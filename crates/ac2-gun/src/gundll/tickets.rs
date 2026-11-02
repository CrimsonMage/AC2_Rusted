//! `IGunTicket`: `SimulateAuthentication` (0x1000b5f0), `GetTicket` (0x1000ba80), `GetTicketEx`
//! (0x1000b6d0), `SetTicket` (0x1000bc00), `PresentTicket` (0x1000be40), `ReleaseTicket` (0x1000b6e0),
//! `CheckTicket` (0x1000b780), `BootUser` (0x1000b9b0); `0a_send_session.md` section 6 (verified).
//! The registry is the [`GunRegistry`](super::registry::GunRegistry) the caller supplies
//! (deviation: GunDll calls the Win32 registry directly).

use super::blob::{base64_decode, extract_tag};
use super::cgun::CGun;
use super::registry::{EMAIL, LAUNCH_TICKET_DATA, LAUNCH_TICKET_SIZE};
use super::requests::wcslen;
use crate::interfaces::*;
use crate::zt::*;
use crate::Hresult;

const E_INVALIDARG: Hresult = Hresult(0x8007_0057);
/// `-0x7fff0001`: `E_FAIL` as GunDll returns it for a registry failure.
const E_FAIL: Hresult = Hresult(0x8000_FFFF);
/// `-0x7fffbfff`: `E_NOTIMPL`, the answer of the calls GunDll leaves unimplemented.
pub const E_NOTIMPL: Hresult = Hresult(0x8000_4001);

impl IGunTicket for CGun {
    /// `CGun::SimulateAuthentication` (0x1000b5f0). Not initialised: `cbSize = 0xcc` and
    /// `ZT_E_NOT_INITED`. A name or password (NULL is `L""`) of 51 or more characters, `cbSize = 0xcc`
    /// and `0x80070057`. A blob smaller than 0xcc: `cbSize = 0xcc` and `ZT_E_BUFFER_TOO_SMALL`.
    /// Otherwise the 0xcc bytes are written: the name as `wcsncpy(dest, name, 0x32)` (padded with NULs)
    /// at wide characters 0..0x32 with `dest[0x32] = 0` (byte 100), the password the same at
    /// `dest + 0x33` with `dest[0x65] = 0` (byte 0xca), `cbSize = 0xcc`.
    ///
    /// Those two NUL terminators, the wide characters at byte offsets 100 and 0xCA, are the zero
    /// markers `PresentTicket` tests: a 0xcc-byte simulated ticket is exactly what its encrypt branch
    /// detects.
    // open: a NULL `pBlobData` makes retail write through NULL; the port allocates the buffer.
    fn simulate_authentication(&self, a: &str, b: &str, c: &mut TagBlob) -> Hresult {
        if !self.inited() {
            c.cb_size = 0xcc;
            return ZT_E_NOT_INITED;
        }
        if wcslen(a) >= 0x33 || wcslen(b) >= 0x33 {
            c.cb_size = 0xcc;
            return E_INVALIDARG;
        }
        if c.cb_size < 0xcc {
            c.cb_size = 0xcc;
            return ZT_E_BUFFER_TOO_SMALL;
        }
        let mut out = [0u16; 0x66];
        for (i, u) in a.encode_utf16().take(0x32).enumerate() {
            out[i] = u;
        }
        for (i, u) in b.encode_utf16().take(0x32).enumerate() {
            out[0x33 + i] = u;
        }
        out[0x32] = 0;
        out[0x65] = 0;
        let bytes: Vec<u8> = out.iter().flat_map(|u| u.to_le_bytes()).collect();
        let data = c.data.get_or_insert_with(Vec::new);
        if data.len() < bytes.len() {
            data.resize(bytes.len(), 0);
        }
        data[..bytes.len()].copy_from_slice(&bytes);
        c.cb_size = 0xcc;
        Hresult::S_OK
    }

    /// `CGun::GetTicket` (0x1000ba80). With no cached ticket: reads `LaunchTicketSize` (a DWORD) and
    /// `LaunchTicketData` (`ZT_E_AUTH_INVALID_TICKET` if either read fails), caches the blob at
    /// `CGun+0x60` and deletes both values. The cached ticket is encrypted with the session key
    /// (`FUN_10001b00`, `ZT_E_NOT_CONNECTED` before the key exists); the caller's `cbSize` becomes the
    /// ticket's size; too small a buffer is `ZT_E_BUFFER_TOO_SMALL`, a NULL buffer `0x80070057`,
    /// otherwise the bytes are copied.
    ///
    /// Deviation: if `LaunchTicketSize` reads but `LaunchTicketData` does not, retail has already
    /// stored the half-filled cache (`CGun+0x60`: the size and an UNINITIALISED buffer) and the next
    /// call encrypts those uninitialised bytes. Uninitialised memory is not reproducible; the port
    /// caches only after both reads succeed, so the next call retries the registry.
    fn get_ticket(&self, blob: &mut TagBlob) -> Hresult {
        let have = self.st.borrow().ticket.is_some();
        if !have {
            let reg = self.env.borrow().registry.clone();
            let Some(size) = reg
                .query_value(LAUNCH_TICKET_SIZE, 4)
                .filter(|v| v.len() >= 4)
                .map(|v| u32::from_le_bytes([v[0], v[1], v[2], v[3]]))
            else {
                return ZT_E_AUTH_INVALID_TICKET;
            };
            let Some(mut data) = reg.query_value(LAUNCH_TICKET_DATA, size as usize) else {
                return ZT_E_AUTH_INVALID_TICKET;
            };
            data.resize(size as usize, 0);
            reg.delete_value(LAUNCH_TICKET_DATA);
            reg.delete_value(LAUNCH_TICKET_SIZE);
            self.st.borrow_mut().ticket = Some(data);
        }
        let cached = self.st.borrow().ticket.clone().unwrap_or_default();
        let enc = match self.encrypt_blob(&cached) {
            Ok(e) => e,
            Err(e) => return e,
        };
        let cap = blob.cb_size;
        blob.cb_size = enc.len() as u32;
        if cap < enc.len() as u32 {
            return ZT_E_BUFFER_TOO_SMALL;
        }
        match blob.data.as_mut() {
            None => E_INVALIDARG,
            Some(d) => {
                if d.len() < enc.len() {
                    d.resize(enc.len(), 0);
                }
                d[..enc.len()].copy_from_slice(&enc);
                Hresult::S_OK
            }
        }
    }

    /// `CGun::GetTicketEx` (0x1000b6d0): unimplemented, `E_NOTIMPL`.
    fn get_ticket_ex(&self, _context: Context) -> Hresult {
        E_NOTIMPL
    }

    /// `CGun::SetTicket` (0x1000bc00). A ticket over 300 bytes (`cbSize`) is XML text: the text after
    /// `<iv>` and after `<encryptedText>` (case-insensitive, up to the next `<`) is Base64 decoded
    /// (`FUN_10002e90`), the first 16 decoded IV bytes go to bytes 0..16 of the caller's buffer and the
    /// decoded text after them, and the caller's `cbSize` becomes `decoded + 0x10`; a missing tag is
    /// `ZT_E_AUTH_INVALID_TICKET`. Then `LaunchTicketData` (REG_BINARY, or a delete when the data
    /// pointer is NULL) and `LaunchTicketSize` (REG_DWORD) are written; a registry failure is `E_FAIL`
    /// (0x8000FFFF).
    fn set_ticket(&self, ticket: &mut TagBlob) -> Hresult {
        if ticket.cb_size > 300 {
            let src = ticket.bytes().to_vec();
            let Ok((iv_text, iv_len)) = extract_tag("<iv>", &src) else {
                return ZT_E_AUTH_INVALID_TICKET;
            };
            let (iv, _) = base64_decode(&iv_text, iv_len);
            let Ok((text, text_len)) = extract_tag("<encryptedText>", &src) else {
                return ZT_E_AUTH_INVALID_TICKET;
            };
            let (dec, dec_len) = base64_decode(&text, text_len);
            // bytes 0..16 = the first 16 bytes of the IV buffer (4 dwords, whatever was decoded)
            let mut full = vec![0u8; 16];
            for (i, b) in iv.iter().take(16).enumerate() {
                full[i] = *b;
            }
            let n = dec_len as usize;
            let mut body = dec;
            body.resize(n, 0);
            full.extend_from_slice(&body[..n]);
            let data = ticket.data.get_or_insert_with(Vec::new);
            if data.len() < full.len() {
                data.resize(full.len(), 0);
            }
            data[..full.len()].copy_from_slice(&full);
            ticket.cb_size = full.len() as u32;
        }
        let reg = self.env.borrow().registry.clone();
        let stored = match ticket.data.as_ref() {
            Some(_) => reg.set_binary(LAUNCH_TICKET_DATA, ticket.bytes()),
            // FUN_1000b900 with NULL data deletes the value
            None => reg.delete_value(LAUNCH_TICKET_DATA),
        };
        if stored && reg.set_dword(LAUNCH_TICKET_SIZE, ticket.cb_size) {
            Hresult::S_OK
        } else {
            E_FAIL
        }
    }

    /// `CGun::PresentTicket` (0x1000be40): not initialised -> `ZT_E_NOT_INITED`; no connection ->
    /// `ZT_E_NOT_CONNECTED`; an empty ticket -> `0x80070057`. The `Email` value is read (at most 100
    /// bytes, as a wide string) and deleted. If the two UTF-16 units at ticket offsets 100 and 0xCA
    /// are both zero the ticket is encrypted with the session key and the Email is NULL; otherwise the
    /// ticket is sent as it is with the registry's Email. `<PresentTicketCmd Email><Ticket/>` (0xf101).
    /// The two zero markers are the NUL terminators of [`IGunTicket::simulate_authentication`]'s
    /// output (`_Dest[0x32]`, `_Dest[0x65]`): a 0xCC-byte simulated ticket takes the encrypt branch.
    // open: a ticket shorter than 0xCC bytes: retail reads past the buffer; the port reads zero.
    fn present_ticket(&self, ticket: &[u8], context: Context) -> Hresult {
        if !self.inited() {
            return ZT_E_NOT_INITED;
        }
        if !self.has_conn() {
            return ZT_E_NOT_CONNECTED;
        }
        if ticket.is_empty() {
            return E_INVALIDARG;
        }
        let reg = self.env.borrow().registry.clone();
        // the 100-byte buffer is zeroed; a failed read leaves an empty string
        let email_bytes = reg.query_value(EMAIL, 100).unwrap_or_default();
        reg.delete_value(EMAIL);
        let unit = |at: usize| -> u16 {
            match (ticket.get(at), ticket.get(at + 1)) {
                (Some(a), Some(b)) => u16::from_le_bytes([*a, *b]),
                _ => 0,
            }
        };
        let (blob, email) = if unit(100) == 0 && unit(0xca) == 0 {
            match self.encrypt_blob(ticket) {
                Ok(enc) => (enc, None),
                Err(e) => return e,
            }
        } else {
            let units: Vec<u16> = email_bytes
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .take_while(|&u| u != 0)
                .collect();
            (ticket.to_vec(), Some(String::from_utf16_lossy(&units)))
        };
        let mut m = self.new_request(0xf101, context);
        m.set_str("Email", email.as_deref());
        m.set_blob("Ticket", Some(&blob));
        self.send_request(&m)
    }

    /// `CGun::ReleaseTicket` (0x1000b6e0): not initialised, then not connected; `<ReleaseTicketCmd>`
    /// (0xf102).
    fn release_ticket(&self, context: Context) -> Hresult {
        if !self.inited() {
            return ZT_E_NOT_INITED;
        }
        let m = self.new_request(0xf102, context);
        self.send_if_connected(m)
    }

    /// `CGun::CheckTicket` (0x1000b780): not initialised -> `ZT_E_NOT_INITED`, not connected ->
    /// `ZT_E_NOT_CONNECTED`; `<CheckTicketCmd AccountName=""><Ticket/>` (0xf100) with the constant
    /// empty account name at 0x10059be0. Retail's only argument test is `p1 == NULL`
    /// (`0x80070057`), which a slice cannot be; there is no empty-ticket test, an empty slice is sent.
    fn check_ticket(&self, a: &[u8], context: Context) -> Hresult {
        if !self.inited() {
            return ZT_E_NOT_INITED;
        }
        if !self.has_conn() {
            return ZT_E_NOT_CONNECTED;
        }
        let mut m = self.new_request(0xf100, context);
        m.set_str("AccountName", Some(""));
        m.set_blob("Ticket", Some(a));
        self.send_request(&m)
    }

    /// `CGun::BootUser` (0x1000b9b0): not connected; an empty name `0x80070057`, one over 50
    /// characters `ZT_E_STR_TOO_LONG`; `<Boot User Reason ReasonText>` (0xf104) with Reason 2.
    fn boot_user(&self, a: &str, b: &str, context: Context) -> Hresult {
        if !self.has_conn() {
            return ZT_E_NOT_CONNECTED;
        }
        if wcslen(a) == 0 {
            return E_INVALIDARG;
        }
        if wcslen(a) >= 0x33 {
            return ZT_E_STR_TOO_LONG;
        }
        let mut m = self.new_request(0xf104, context);
        m.set_str("User", Some(a));
        m.set_dec("Reason", 2);
        m.set_str("ReasonText", Some(b));
        self.send_request(&m)
    }
}
