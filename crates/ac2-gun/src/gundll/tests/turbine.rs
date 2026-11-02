//! End to end: `TurbineGun` over the real `CGun` (instead of the recording double), a fake DP8 and a
//! fake GUN server.

use std::rc::Rc;

use rsa::traits::PublicKeyParts;

use super::fakes::*;
use super::handshake::server_key;
use crate::gundll::blob;
use crate::gundll::crypto::build_public_key_blob;
use crate::gundll::dp8::Dp8Message;
use crate::gundll::{CGun, GunEnv};
use crate::interfaces::{Guid, GunObject};
use crate::turbine_gun_utils::{ClassFactory, GunLoader, GunModule};
use crate::{Hresult, TurbineGun};

struct Factory(Option<Box<dyn GunObject>>);

impl ClassFactory for Factory {
    fn create_instance(&mut self, _riid: &Guid) -> (Hresult, Option<Box<dyn GunObject>>) {
        (Hresult::S_OK, self.0.take())
    }
}

struct Module(Option<Box<dyn GunObject>>);

impl GunModule for Module {
    fn dll_get_class_object(
        &mut self,
        _clsid: &Guid,
        _riid: &Guid,
    ) -> Option<(Hresult, Option<Box<dyn ClassFactory>>)> {
        Some((Hresult::S_OK, Some(Box::new(Factory(self.0.take())))))
    }
    fn get_last_error(&self) -> u32 {
        0
    }
}

struct Loader(Option<CGun>);

impl GunLoader for Loader {
    fn load_library(&mut self, _path: &str) -> Result<Box<dyn GunModule>, u32> {
        Ok(Box::new(Module(
            self.0.take().map(|g| Box::new(g) as Box<dyn GunObject>),
        )))
    }
}

#[test]
fn turbine_gun_connects_through_the_real_cgun() {
    let dp8 = Dp8::default();
    let reg = Rc::new(Reg::default());
    let mut env = GunEnv::new(Rc::new(dp8.clone()), reg);
    env.random = test_random();
    env.probe_port = Box::new(|| Some(40001));
    let cgun = CGun::new(env);
    let turbine = TurbineGun::new(0x0409);
    assert!(turbine.init("GunDll.dll", &mut Loader(Some(cgun))));
    assert!(turbine.request_connect("gun.example.net", 8610, 0, "1.0", "XXXX-YYYY-ZZZZ", 0));
    assert!(turbine.is_connecting());
    // DP8 connected: the next CheckForMessages sends <Connect>
    dp8.push(Dp8Message::ConnectComplete {
        hresult: Hresult::S_OK,
        reply: vec![],
    });
    assert!(turbine.check_for_messages());
    let first = dp8.sent_text(0);
    assert!(first.starts_with("<Connect Cx=\"0x"));
    assert!(first.ends_with("\" ProtoVer=\"3.3\" />"));
    let cx = first
        .trim_start_matches("<Connect Cx=\"0x")
        .split('"')
        .next()
        .unwrap()
        .to_owned();
    let key = server_key();
    let pubkey = build_public_key_blob(&key.n().to_bytes_be(), 0x10001);
    dp8.push_datagram(datagram(&format!(
        "<ConnectRes Cx=\"0x{cx}\" HR=\"0x0\" Random=\"99\" ><RSAKey Val=\"{}\"/></ConnectRes> ",
        blob::encode_for_attribute(&pubkey).unwrap()
    )));
    assert!(turbine.check_for_messages());
    assert_eq!(dp8.sent_count(), 2, "ConInfo");
    dp8.push_datagram(datagram(&format!(
        "<ConInfoRes Cx=\"0x{cx}\" HR=\"0x0\" />"
    )));
    assert!(turbine.check_for_messages());
    // ConnectEv (0x1300): TurbineGun::RcvConnectEvent marks the connection up
    assert!(turbine.is_connected());
    assert!(!turbine.is_connecting());
}
