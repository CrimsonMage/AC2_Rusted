//! End to end: `TurbineGun` -> the real `CGun` -> this bridge -> the DX9 DirectPlay client ->
//! a scripted fake UDP host on a fake socket and clock. The host answers the DirectPlay handshake
//! (CONNECTED, 0xC2) and then the GUN handshake (`<ConnectRes>` with an RSA key generated here,
//! `<ConInfoRes>`); every DirectPlay frame the client sends up to the local ConnectEv (0x1300) is
//! compared with bytes written out from the specs (`spec_client_delta.md` 2.x / 3.1 and its
//! verification: v1.6 CFRAME `80|08 op mid rsp 06 00 01 00 sid tick`, DFRAME `cmd ctl seq nrcv`,
//! the `_EX` 0xC1 of 0x5c bytes, the first KeepAlive `3F 02 00 00 sid`), never from the encoder.

use std::cell::RefCell;
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::rc::Rc;

use rsa::traits::PublicKeyParts;
use rsa::RsaPrivateKey;

use super::*;
use crate::dplay8::clock::{FakeClock, FixedRandom};
use crate::dplay8::tests::{
    cframe, FakeAdapters, FakeOpener, Net, NoResolver, ADAPTER, APP_GUID, HOST,
};
use crate::gundll::blob;
use crate::gundll::crypto::build_public_key_blob;
use crate::gundll::{CGun, GunEnv, GunRegistry};
use crate::interfaces::GunObject;
use crate::turbine_gun_utils::{ClassFactory, GunLoader, GunModule};
use crate::{Hresult, TurbineGun};

const SID: u32 = 0x1122_3344;
const T0: u32 = 10_000;

struct TestRng(u64);
impl rsa::rand_core::RngCore for TestRng {
    fn next_u32(&mut self) -> u32 {
        self.next_u64() as u32
    }
    fn next_u64(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        for b in dest {
            *b = (self.next_u64() >> 24) as u8;
        }
    }
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rsa::rand_core::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}
impl rsa::rand_core::CryptoRng for TestRng {}

#[derive(Default)]
struct Reg(RefCell<HashMap<String, Vec<u8>>>);
impl GunRegistry for Reg {
    fn query_value(&self, name: &str, max_len: usize) -> Option<Vec<u8>> {
        self.0
            .borrow()
            .get(name)
            .filter(|v| v.len() <= max_len)
            .cloned()
    }
    fn set_binary(&self, name: &str, data: &[u8]) -> bool {
        self.0.borrow_mut().insert(name.into(), data.to_vec());
        true
    }
    fn set_dword(&self, name: &str, value: u32) -> bool {
        self.0
            .borrow_mut()
            .insert(name.into(), value.to_le_bytes().to_vec());
        true
    }
    fn delete_value(&self, name: &str) -> bool {
        self.0.borrow_mut().remove(name).is_some()
    }
}

struct Factory(Option<Box<dyn GunObject>>);
impl ClassFactory for Factory {
    fn create_instance(
        &mut self,
        _riid: &crate::interfaces::Guid,
    ) -> (Hresult, Option<Box<dyn GunObject>>) {
        (Hresult::S_OK, self.0.take())
    }
}
struct Module(Option<Box<dyn GunObject>>);
impl GunModule for Module {
    fn dll_get_class_object(
        &mut self,
        _c: &crate::interfaces::Guid,
        _r: &crate::interfaces::Guid,
    ) -> Option<(Hresult, Option<Box<dyn ClassFactory>>)> {
        Some((Hresult::S_OK, Some(Box::new(Factory(self.0.take())))))
    }
    fn get_last_error(&self) -> u32 {
        0
    }
}
struct Loader(Option<CGun>);
impl GunLoader for Loader {
    fn load_library(&mut self, _p: &str) -> Result<Box<dyn GunModule>, u32> {
        Ok(Box::new(Module(
            self.0.take().map(|g| Box::new(g) as Box<dyn GunObject>),
        )))
    }
}

fn counter_random() -> Box<dyn FnMut(&mut [u8])> {
    let mut n = 0u8;
    Box::new(move |buf: &mut [u8]| {
        for b in buf {
            n = if n == 255 { 1 } else { n + 1 };
            *b = n;
        }
    })
}

/// `cmd ctl seq nrcv` + payload.
fn dframe(cmd: u8, seq: u8, nrcv: u8, payload: &[u8]) -> Vec<u8> {
    let mut v = vec![cmd, 0, seq, nrcv];
    v.extend_from_slice(payload);
    v
}

/// A GUN datagram (`50 C3`, UTF-8 text, NUL; 0a_receive_messages.md framing).
fn gun(text: &str) -> Vec<u8> {
    let mut v = vec![0x50, 0xC3];
    v.extend_from_slice(text.as_bytes());
    v.push(0);
    v
}

/// The 0xC1 `_EX` payload (verify_spec_client_delta 3.1): type, dwFlags 2 (CLIENT), dwDNETVersion
/// 7, ten zero dwords (name, data, password, connect data, URL, instance GUID's offsets/sizes
/// and the instance GUID), the application GUID, 8 zero bytes: 0x5c bytes.
fn c1_payload() -> Vec<u8> {
    let mut p = vec![0xC1, 0, 0, 0, 2, 0, 0, 0, 7, 0, 0, 0];
    p.extend_from_slice(&[0; 40]);
    p.extend_from_slice(&[0; 16]);
    p.extend_from_slice(&APP_GUID);
    p.extend_from_slice(&[0; 8]);
    assert_eq!(p.len(), 0x5c);
    p
}

struct Rig {
    turbine: TurbineGun,
    clock: FakeClock,
    net: Rc<RefCell<Net>>,
}

impl Rig {
    fn tick(&self) -> Vec<Vec<u8>> {
        self.clock.advance(20);
        self.turbine.check_for_messages();
        std::mem::take(&mut self.net.borrow_mut().to_host)
    }
    /// Tick until the client sends something (at most 3 ticks: CGun queues the send after the
    /// DoWork that raised the message and the protocol sends it on the next pump).
    fn tick_out(&self) -> Vec<Vec<u8>> {
        for _ in 0..3 {
            let out = self.tick();
            if !out.is_empty() {
                return out;
            }
        }
        panic!("the client sent nothing");
    }
    fn host(&self, d: Vec<u8>) {
        self.net.borrow_mut().from_host.push_back(d);
    }
}

#[test]
fn turbine_gun_to_dx9_client_to_scripted_host_up_to_connect_ev() {
    let clock = FakeClock::new(T0);
    let net = Rc::new(RefCell::new(Net::default()));
    let (c2, n2) = (clock.clone(), net.clone());
    let factory = Dx9Dp8Factory::new(Box::new(move || {
        Dplay8Client::new(
            Box::new(c2.clone()),
            Box::new(FakeOpener(n2.clone())),
            Box::new(NoResolver),
            Box::new(FakeAdapters(vec![ADAPTER])),
            Box::new(FixedRandom(vec![SID], 0)),
        )
    }));
    let mut env = GunEnv::new(Rc::new(factory), Rc::new(Reg::default()));
    env.random = counter_random();
    env.probe_port = Box::new(|| Some(40001));
    env.pool = crate::gundll::SharedThreadPool::new();
    let cgun = CGun::new(env);
    let turbine = TurbineGun::new(0x0409);
    assert!(turbine.init("GunDll.dll", &mut Loader(Some(cgun))));
    let rig = Rig {
        turbine,
        clock,
        net,
    };
    assert!(rig
        .turbine
        .request_connect("10.0.0.5", 2300, 0, "1.0", "XXXX-YYYY-ZZZZ", 0));
    assert!(rig.turbine.is_connecting());
    assert_eq!(*rig.net.borrow().binds.first().unwrap(), 40001);
    assert_eq!(HOST, SocketAddrV4::new(Ipv4Addr::new(10, 0, 0, 5), 2300));

    // Tick 1 (+20 ms): the CONNECT CFRAME, POLL set, bMsgID 0 (spec_client_delta 2.2).
    assert_eq!(rig.tick(), vec![cframe(true, 1, 0, 0, SID, T0 + 20)]);
    assert!(rig.tick().is_empty());
    // The host's CONNECTED (bMsgID 5, POLL) is answered by the client's CONNECTED (bMsgID 1,
    // bRspId 5, no POLL), the first KeepAlive (v1.6 form, seq 0) and the lone reliable 0xC1
    // (USER_1 0x77|POLL = 0x7F, seq 1, nrcv 0).
    rig.host(cframe(true, 2, 5, 0, SID, 77));
    assert_eq!(
        rig.tick(),
        vec![
            cframe(false, 2, 1, 5, SID, T0 + 60),
            vec![0x3F, 0x02, 0x00, 0x00, 0x44, 0x33, 0x22, 0x11],
            dframe(0x7F, 1, 0, &c1_payload()),
        ]
    );
    // The host's 0xC2 (seq 0) acknowledges both (nrcv 2); the client answers 0xC3 (seq 2, nrcv 1).
    let mut c2 = vec![0xC2, 0, 0, 0];
    c2.extend_from_slice(&[0; 12]);
    rig.host(dframe(0x77, 0, 2, &c2));
    let out = rig.tick();
    assert_eq!(out, vec![dframe(0x7F, 2, 1, &[0xC3, 0, 0, 0])]);
    assert!(!rig.turbine.is_connected());
    // CGun saw CONNECT_COMPLETE through the bridge: the next CheckForMessages sends <Connect>
    // as a GUN datagram in a lone reliable DFRAME (flags 0x9 never coalesce), seq 3, nrcv 1.
    let out = rig.tick_out();
    assert_eq!(out.len(), 1, "{out:?}");
    let f = &out[0];
    assert_eq!(&f[..4], &[0x3F, 0x00, 0x03, 0x01]);
    assert_eq!(&f[4..6], &[0x50, 0xC3]);
    assert_eq!(*f.last().unwrap(), 0);
    let text = String::from_utf8(f[6..f.len() - 1].to_vec()).unwrap();
    assert!(
        text.starts_with("<Connect Cx=\"0x") && text.ends_with("\" ProtoVer=\"3.3\" />"),
        "{text}"
    );
    let cx = text["<Connect Cx=\"0x".len()..]
        .split('"')
        .next()
        .unwrap()
        .to_owned();

    // The host acknowledges (SACK-less: its next DFRAME carries nrcv 4) with <ConnectRes>.
    let key = RsaPrivateKey::new(&mut TestRng(0x1234_5678_9abc_def1), 512).unwrap();
    let pubkey = build_public_key_blob(&key.n().to_bytes_be(), 0x10001);
    let res = gun(&format!(
        "<ConnectRes Cx=\"0x{cx}\" HR=\"0x0\" Random=\"99\" ><RSAKey Val=\"{}\"/></ConnectRes> ",
        blob::encode_for_attribute(&pubkey).unwrap()
    ));
    rig.host(dframe(0x37, 1, 4, &res));
    let out = rig.tick_out();
    assert_eq!(out.len(), 1, "{out:?}");
    let f = &out[0];
    // <ConInfo ...>: seq 4, nrcv 2 (the host's seq 0 and 1 are in).
    assert_eq!(&f[..4], &[0x3F, 0x00, 0x04, 0x02]);
    assert_eq!(&f[4..6], &[0x50, 0xC3]);
    assert!(f[6..].starts_with(b"<ConInfo "));
    let ci = String::from_utf8_lossy(&f[6..]).to_string();
    assert!(ci.contains(&format!("Cx=\"0x{cx}\"")), "{ci}");

    // <ConInfoRes> (host seq 2, nrcv 5) completes the GUN handshake: ConnectEv (0x1300).
    rig.host(dframe(
        0x37,
        2,
        5,
        &gun(&format!("<ConInfoRes Cx=\"0x{cx}\" HR=\"0x0\" />")),
    ));
    rig.tick();
    rig.tick();
    assert!(rig.turbine.is_connected());
    assert!(!rig.turbine.is_connecting());
}
