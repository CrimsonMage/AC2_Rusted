//! A fake DirectPlay 8 (client and thread pool), a fake registry and a fake GUN server.

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

use crate::gundll::crypto::rc4;
use crate::gundll::dp8::*;
use crate::gundll::{CGun, GunEnv, GunRegistry, SharedThreadPool};
use crate::interfaces::*;
use crate::message::GunQueueMessage;
use crate::Hresult;

#[derive(Default)]
pub struct Dp8State {
    pub handler: Option<Rc<dyn Dp8MessageHandler>>,
    pub init_flags: Vec<u32>,
    pub sent: Vec<(Vec<u8>, u32, u32)>,
    pub connects: Vec<(Dp8ApplicationDesc, Dp8Address, Dp8Address)>,
    pub caps_log: Vec<String>,
    pub caps: Dp8Caps,
    pub connect_result: Option<Hresult>,
    pub send_result: Option<Hresult>,
    pub inbound: VecDeque<Dp8Message>,
    pub returned: Vec<u32>,
    pub closed: usize,
    pub do_work: Vec<(u32, u32)>,
    pub thread_counts: Vec<(u32, u32, u32)>,
    pub pool_created: usize,
    pub client_fails: bool,
    pub cancels: Vec<(u32, u32)>,
    /// The DP8 calls in order: `create_client`, `initialize`, `close`, `cancel`, `connect`, `send`.
    pub log: Vec<&'static str>,
    pub pool_fails: bool,
    /// `GetConnectionInfo`: the round-trip time answered (`None`: the call fails).
    pub rtt: Option<u32>,
    pub conn_info_sizes: Vec<u32>,
    /// Runs inside `return_buffer` (a DP8 call made from the Q1 drain).
    pub on_return_buffer: RefCell<Option<Rc<dyn Fn()>>>,
}

#[derive(Clone, Default)]
pub struct Dp8(pub Rc<RefCell<Dp8State>>);

impl IDirectPlay8Client for Dp8 {
    fn initialize(&self, handler: Rc<dyn Dp8MessageHandler>, flags: u32) -> Hresult {
        let mut s = self.0.borrow_mut();
        s.init_flags.push(flags);
        s.log.push("initialize");
        s.handler = Some(handler);
        Hresult::S_OK
    }
    fn connect(&self, a: &Dp8ApplicationDesc, h: &Dp8Address, d: &Dp8Address) -> (Hresult, u32) {
        let mut s = self.0.borrow_mut();
        s.connects.push((a.clone(), h.clone(), d.clone()));
        s.log.push("connect");
        (s.connect_result.unwrap_or(DPNSUCCESS_PENDING), 7)
    }
    fn send(&self, buffer: &[u8], timeout: u32, context: u32, flags: u32) -> Hresult {
        let mut s = self.0.borrow_mut();
        s.sent.push((buffer.to_vec(), timeout, flags));
        s.log.push("send");
        let _ = context;
        s.send_result.unwrap_or(DPNSUCCESS_PENDING)
    }
    fn cancel_async_operation(&self, handle: u32, flags: u32) -> Hresult {
        let mut s = self.0.borrow_mut();
        s.cancels.push((handle, flags));
        s.log.push("cancel");
        Hresult::S_OK
    }
    fn close(&self, _flags: u32) -> Hresult {
        let mut s = self.0.borrow_mut();
        s.closed += 1;
        s.log.push("close");
        Hresult::S_OK
    }
    fn return_buffer(&self, h: u32, _flags: u32) -> Hresult {
        let hook = {
            let mut s = self.0.borrow_mut();
            s.returned.push(h);
            let hook = s.on_return_buffer.borrow().clone();
            hook
        };
        if let Some(f) = hook {
            f();
        }
        Hresult::S_OK
    }
    fn get_caps(&self, caps: &mut Dp8Caps, _flags: u32) -> Hresult {
        let mut s = self.0.borrow_mut();
        s.caps_log.push(format!("get size={:#x}", caps.size));
        *caps = Dp8Caps {
            size: caps.size,
            ..s.caps
        };
        Hresult::S_OK
    }
    fn get_connection_info(&self, info: &mut Dp8ConnectionInfo, _flags: u32) -> Hresult {
        let mut s = self.0.borrow_mut();
        s.conn_info_sizes.push(info.size);
        match s.rtt {
            Some(r) => {
                info.round_trip_latency_ms = r;
                Hresult::S_OK
            }
            None => Hresult(0x8015_8480),
        }
    }
    fn set_caps(&self, caps: &Dp8Caps, _flags: u32) -> Hresult {
        let mut s = self.0.borrow_mut();
        s.caps_log
            .push(format!("set keepalive={}", caps.timeout_until_keep_alive));
        s.caps = *caps;
        Hresult::S_OK
    }
}

impl IDirectPlay8ThreadPool for Dp8 {
    fn set_thread_count(&self, p: u32, n: u32, f: u32) -> Hresult {
        self.0.borrow_mut().thread_counts.push((p, n, f));
        Hresult::S_OK
    }
    /// Delivers the queued messages to the handler on this thread, as DP8 does inside `DoWork`.
    fn do_work(&self, slice: u32, flags: u32) -> Hresult {
        self.0.borrow_mut().do_work.push((slice, flags));
        loop {
            let (msg, handler) = {
                let mut s = self.0.borrow_mut();
                (s.inbound.pop_front(), s.handler.clone())
            };
            match (msg, handler) {
                (Some(m), Some(h)) => {
                    h.message(&m);
                }
                _ => break,
            }
        }
        Hresult::S_OK
    }
}

impl Dp8Factory for Dp8 {
    fn create_client(&self) -> Result<Rc<dyn IDirectPlay8Client>, Hresult> {
        self.0.borrow_mut().log.push("create_client");
        if self.0.borrow().client_fails {
            return Err(Hresult(0x8004_0154));
        }
        Ok(Rc::new(self.clone()))
    }
    fn create_thread_pool(&self) -> Result<Rc<dyn IDirectPlay8ThreadPool>, Hresult> {
        if self.0.borrow().pool_fails {
            return Err(Hresult(0x8004_0154));
        }
        self.0.borrow_mut().pool_created += 1;
        Ok(Rc::new(self.clone()))
    }
}

impl Dp8 {
    /// Queues a datagram for the handler (delivered by the next `do_work`).
    pub fn push_datagram(&self, data: Vec<u8>) {
        self.0.borrow_mut().inbound.push_back(Dp8Message::Receive {
            data,
            buffer_handle: 99,
        });
    }
    pub fn push(&self, m: Dp8Message) {
        self.0.borrow_mut().inbound.push_back(m);
    }
    /// The UTF-8 text of the n-th sent buffer after its `50 C3` type word and before its NUL.
    pub fn sent_text(&self, n: usize) -> String {
        let s = self.0.borrow();
        let b = &s.sent[n].0;
        assert_eq!(&b[..2], &[0x50, 0xC3], "u16 type 0xC350");
        assert_eq!(*b.last().unwrap(), 0, "NUL terminated");
        String::from_utf8(b[2..b.len() - 1].to_vec()).unwrap()
    }
    pub fn sent_count(&self) -> usize {
        self.0.borrow().sent.len()
    }
}

/// `50 C3` + text + NUL: a server datagram (`0a_receive_messages.md` framing resolution).
pub fn datagram(text: &str) -> Vec<u8> {
    let mut v = vec![0x50, 0xC3];
    v.extend_from_slice(text.as_bytes());
    v.push(0);
    v
}

#[derive(Default)]
pub struct Reg {
    pub values: RefCell<HashMap<String, Vec<u8>>>,
    pub fail_writes: RefCell<bool>,
}

impl GunRegistry for Reg {
    fn query_value(&self, name: &str, max_len: usize) -> Option<Vec<u8>> {
        self.values
            .borrow()
            .get(name)
            .filter(|v| v.len() <= max_len)
            .cloned()
    }
    fn set_binary(&self, name: &str, data: &[u8]) -> bool {
        if *self.fail_writes.borrow() {
            return false;
        }
        self.values.borrow_mut().insert(name.into(), data.to_vec());
        true
    }
    fn set_dword(&self, name: &str, value: u32) -> bool {
        if *self.fail_writes.borrow() {
            return false;
        }
        self.values
            .borrow_mut()
            .insert(name.into(), value.to_le_bytes().to_vec());
        true
    }
    fn delete_value(&self, name: &str) -> bool {
        self.values.borrow_mut().remove(name).is_some()
    }
}

/// Counter-based "randomness" (never zero, so the PKCS #1 padding terminates).
pub fn test_random() -> Box<dyn FnMut(&mut [u8])> {
    let mut n = 0u8;
    Box::new(move |buf: &mut [u8]| {
        for b in buf {
            n = if n == 255 { 1 } else { n + 1 };
            *b = n;
        }
    })
}

pub struct Rig {
    pub gun: Rc<CGun>,
    pub dp8: Dp8,
    pub reg: Rc<Reg>,
}

pub const APP_GUID: Guid = Guid {
    data1: 0x2280767d,
    data2: 0x1689,
    data3: 0x4d6b,
    data4: [0x81, 0xdd, 0x3a, 0xd9, 0xe8, 0x0f, 0xb6, 0x55],
};

pub fn rig() -> Rig {
    let dp8 = Dp8::default();
    let reg = Rc::new(Reg::default());
    let mut env = GunEnv::new(Rc::new(dp8.clone()), reg.clone());
    env.random = test_random();
    env.pool = SharedThreadPool::new();
    env.probe_port = Box::new(|| Some(40123));
    Rig {
        gun: Rc::new(CGun::new(env)),
        dp8,
        reg,
    }
}

/// A sink that records the messages and returns scripted results.
#[derive(Default)]
pub struct Sink {
    pub got: RefCell<Vec<GunQueueMessage>>,
    pub results: RefCell<VecDeque<Hresult>>,
    pub on_read: RefCell<Option<Box<dyn Fn(&GunQueueMessage)>>>,
}

impl IGunQueueRead for Sink {
    fn read(&self, m: &GunQueueMessage) -> Hresult {
        self.got.borrow_mut().push(m.clone());
        if let Some(f) = self.on_read.borrow().as_ref() {
            f(m);
        }
        self.results
            .borrow_mut()
            .pop_front()
            .unwrap_or(Hresult::S_OK)
    }
}

impl Sink {
    pub fn ids(&self) -> Vec<u32> {
        self.got.borrow().iter().map(|m| m.message.id.0).collect()
    }
}

/// The server's view of the session key: it imports the SIMPLEBLOB and keys RC4 with the 5 key bytes
/// and the 11 zero salt bytes (Microsoft's documentation, `msdocs/salt-value-functionality.md`).
pub fn server_session_key(key5: &[u8]) -> [u8; 16] {
    let mut k = [0u8; 16];
    k[..5].copy_from_slice(key5);
    k
}

pub fn rc4_apply(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut d = data.to_vec();
    rc4(key, &mut d);
    d
}
