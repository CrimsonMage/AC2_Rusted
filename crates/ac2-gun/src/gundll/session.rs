//! The connect handshake and the session crypto helpers: `FUN_10009e10` (0x10009e10, the handler
//! for the internal connection events), `FUN_1000a100` (0x1000a100, `<ConnectRes>`), `FUN_10009fd0`
//! (0x10009fd0, `<ConInfoRes>`), `FUN_10001a40` and `FUN_10001b00` (the field encryption).
//! `0a_send_session.md` sections 3-5 and `0a_receive_messages.md` 1.4-1.5 (both verified).

use super::catalog::{by_id, CONNECT_FAILED_EV};
use super::cgun::{CGun, CryptContext, RandomSource};
use super::ids::*;
use super::msg::GunMsg;
use crate::zt::*;
use crate::Hresult;

/// `ProtoVer` of `<Connect>`: the constant `L"3.3"` at GunDll 0x1005b3d8 (the `Init` version string
/// is not used).
pub const PROTO_VER: &str = "3.3";

impl CGun {
    /// `FUN_10009e10(CGun, {type, H, hr})` (0x10009e10).
    ///
    /// * type 0 (DP8 connect completed): sends `<Connect Cx=CGun ProtoVer="3.3">`
    ///   (`FUN_10013180` + `SendRequest`); a negative result is returned (Read answers 1).
    /// * type 1 (connect failed): queues a `ConnectFailedEv`, frees `conn` if it is `H`, sets
    ///   `+0x54 = hr` and status 4.
    /// * type 2 (terminated): queues a `DisconnectEv`, frees `conn` if it is `H`, runs the host
    ///   cleanup and sets status 3.
    /// * types 3 and 4: nothing.
    pub(super) fn connection_event(&self, kind: u32, conn_id: u64, hr: Hresult) -> Hresult {
        match kind {
            0 => {
                let def = by_id(CONNECT).expect("Connect is catalogued");
                let mut m = GunMsg::new(def);
                m.cx = self.connect_cx();
                m.set_str("ProtoVer", Some(PROTO_VER));
                let r = self.send_request(&m);
                if !r.succeeded() {
                    return r;
                }
                Hresult::S_OK
            }
            1 => {
                self.free_conn_if(conn_id);
                self.post(GunMsg::new(&CONNECT_FAILED_EV));
                let mut st = self.st.borrow_mut();
                st.last_error = hr;
                st.status = 4;
                Hresult::S_OK
            }
            2 => {
                self.free_conn_if(conn_id);
                self.post(GunMsg::new(
                    by_id(DISCONNECT_EV).expect("DisconnectEv is catalogued"),
                ));
                self.st.borrow_mut().status = 3;
                Hresult::S_OK
            }
            _ => Hresult::S_OK,
        }
    }

    /// `if (conn != NULL && H == conn) { delete H; conn = NULL; }`. The handler's destructor
    /// (`FUN_10035d50` -> `FUN_10003860`) frees `H+4`, which the failed-connect and TERMINATE sink slots
    /// (0x100035d0, 0x10003660) cleared before they destroyed the sink object `A` themselves
    /// (`A.vtable[0](A, 1)`): the transport is already gone, nothing is closed here.
    fn free_conn_if(&self, conn_id: u64) {
        let mut st = self.st.borrow_mut();
        if st.conn.as_ref().is_some_and(|c| c.id == conn_id) {
            st.conn = None;
        }
    }

    /// `FUN_1000a100(CGun, ConnectRes)` (0x1000a100): the key exchange.
    ///
    /// `HR < 0` -> status 4 and the error saved, returned. Otherwise: the crypto object is created
    /// (`GunKey`), the server's `RSAKey` imported, a 4-byte client random generated, the session key
    /// derived from `MD5(clientRandom || Random)`, exported as a SIMPLEBLOB under the server key, stored
    /// at `CGun+0x80`, the CD key encrypted and `<ConInfo>` sent with the `Cx` of the `ConnectRes`.
    /// On a failure after the key exists the key is dropped (`+0x80 = 0`).
    pub(super) fn on_connect_res(&self, res: &GunMsg) -> Hresult {
        let hr = Hresult(res.hr);
        if !hr.succeeded() {
            let mut st = self.st.borrow_mut();
            st.status = 4;
            st.last_error = hr;
            return hr;
        }
        let server_random = res.dec_field("Random") as u32;
        let rsa_key = res.blob_field("RSAKey").unwrap_or(&[]).to_vec();
        // FUN_10004170 / FUN_100041c0(obj, "GunKey", 1)
        let (client_random_key, simple) = {
            let mut env = self.env.borrow_mut();
            let mut st = self.st.borrow_mut();
            let crypt = st.crypt.get_or_insert_with(CryptContext::new);
            import_and_derive(crypt, &mut env.random, &rsa_key, server_random)
        };
        let (key, simple) = match (client_random_key, simple) {
            (Ok(k), Ok(s)) => (k, s),
            // the key object exists but the export failed: the cleanup clears CGun+0x80 (a previous
            // session key does not survive)
            (Ok(_), Err(e)) => {
                self.st.borrow_mut().session_key = None;
                return e;
            }
            (Err(e), _) => return e,
        };
        self.st.borrow_mut().session_key = Some(key);
        let result = (|| {
            let (cd_key, locale, guid_text) = {
                let st = self.st.borrow();
                (st.cd_key.clone(), st.locale, st.guid_text.clone())
            };
            // FUN_10001a40(CGun, &blob, CGun+0x74)
            let cd_blob = self.encrypt_text(&cd_key)?;
            // FUN_100137c0(ConInfo, Cx, Locale, CdKey, AppGuid, Random)
            let def = by_id(CON_INFO).expect("ConInfo is catalogued");
            let mut m = GunMsg::new(def);
            m.cx = res.cx;
            m.set_dec("Locale", locale as i32);
            m.set_str("AppGuid", Some(&guid_text));
            m.set_blob("CdKey", Some(&cd_blob));
            m.set_blob("Random", Some(&simple));
            let r = self.send_request(&m);
            if r.succeeded() {
                Ok(())
            } else {
                Err(r)
            }
        })();
        match result {
            Ok(()) => Hresult::S_OK,
            Err(e) => {
                self.st.borrow_mut().session_key = None;
                e
            }
        }
    }

    /// `FUN_10009fd0(CGun, ConInfoRes)` (0x10009fd0): `HR < 0` -> status 4, error saved and returned
    /// (no `ConnectEv`); else a `ConnectEv` is queued and the status set to 2 (4 if the queue write
    /// failed). The hosted-game hook `FUN_1000c460` (`CGun+0xac`) is not present without a game.
    pub(super) fn on_con_info_res(&self, res: &GunMsg) -> Hresult {
        let hr = Hresult(res.hr);
        if !hr.succeeded() {
            let mut st = self.st.borrow_mut();
            st.status = 4;
            st.last_error = hr;
            return hr;
        }
        let r = self.post(GunMsg::new(
            by_id(CONNECT_EV).expect("ConnectEv is catalogued"),
        ));
        self.st.borrow_mut().status = if r.succeeded() { 2 } else { 4 };
        r
    }

    /// `FUN_10001a40(CGun, &blob, wstr)` (0x10001a40): UTF-16 to UTF-8 including the NUL
    /// (`FUN_10002550`), then `CryptEncrypt` with the session key; no key is `ZT_E_NOT_CONNECTED`.
    pub(super) fn encrypt_text(&self, s: &str) -> Result<Vec<u8>, Hresult> {
        let mut bytes = s.as_bytes().to_vec();
        bytes.push(0);
        let st = self.st.borrow();
        let key = st.session_key.as_ref().ok_or(ZT_E_NOT_CONNECTED)?;
        key.encrypt(&mut bytes);
        Ok(bytes)
    }

    /// `FUN_10001b00(CGun, &out, blob)` (0x10001b00): the blob copied and `CryptEncrypt`ed.
    pub(super) fn encrypt_blob(&self, blob: &[u8]) -> Result<Vec<u8>, Hresult> {
        let st = self.st.borrow();
        let key = st.session_key.as_ref().ok_or(ZT_E_NOT_CONNECTED)?;
        let mut out = blob.to_vec();
        key.encrypt(&mut out);
        Ok(out)
    }
}

fn import_and_derive(
    crypt: &mut CryptContext,
    random: &mut RandomSource,
    rsa_key: &[u8],
    server_random: u32,
) -> (
    Result<super::cgun::SessionKey, Hresult>,
    Result<Vec<u8>, Hresult>,
) {
    // FUN_100048d0: CryptImportKey
    let r = crypt.import_public_key(rsa_key);
    if !r.succeeded() {
        return (Err(r), Err(r));
    }
    // FUN_10004940: CryptGenRandom(4); FUN_10004cd0: MD5 + CryptDeriveKey
    let client_random = crypt.gen_random_u32(random);
    let key = crypt.derive_session_key(client_random, server_random);
    // FUN_10004a30: CryptExportKey(SIMPLEBLOB)
    let blob = crypt.export_simple_blob(&key, random);
    (Ok(key), blob)
}
