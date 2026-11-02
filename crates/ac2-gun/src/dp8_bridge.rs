//! The DirectX 9.0b DirectPlay 8 client ([`crate::dplay8`]) behind the traits GunDll's `CGun`
//! uses ([`crate::gundll::dp8`]): [`Dx9Dp8Factory`] is `CoCreateInstance`, [`Dx9Client`] the
//! `IDirectPlay8Client` and [`Dx9ThreadPool`] the `IDirectPlay8ThreadPool`.
//!
//! # Shape
//! * `CGun`'s traits are `&self` and single threaded; the dplay8 client is `&mut self`, so
//!   [`Dx9Client`] keeps it in a `RefCell`. **No borrow is held across a handler call**: `DoWork`
//!   borrows the client only to pump it and to take the indications, releases it, then calls the
//!   handler once per indication and settles each `RECEIVE` with a second, short borrow. A
//!   handler may therefore call `Send`, `ReturnBuffer`, `Close`, `CancelAsyncOperation` and so
//!   on, which is what `CGun` does (`ReturnBuffer` from its dispatch, `Close` from a teardown).
//! * The thread pool (DX9's process-wide `IDirectPlay8ThreadPool`) holds the clients created by
//!   the factory (weakly) and `DoWork` pumps each in creation order; `GunDll` sets zero worker
//!   threads, so no other pump exists.
//! * Conversions: `Dp8Address` -> `DpAddress` (SP then components in call order), `Dp8ApplicationDesc`
//!   -> `DpnApplicationDesc` (the fields GunDll fills; `max_players` 0, no name or password),
//!   `Guid` -> its 16 COM-order bytes; `Connect`'s `Ok(handle)` -> `DPNSUCCESS_PENDING` and the
//!   handle, an error -> that error and handle 0; `Send`'s single buffer -> one descriptor.
//! * Messages: `CONNECT_COMPLETE`, `RECEIVE`, `SEND_COMPLETE`, `TERMINATE_SESSION` map one to
//!   one. The client never raises `CREATE_PLAYER`/`RETURN_BUFFER`/`ASYNC_OP_COMPLETE`/
//!   `ENUM_HOSTS_RESPONSE` (the client path of dpnet does not: see `dplay8::messages`).
//!
//! Unbridgeable: none known. A handler that re-enters `DoWork` is not guarded (the second pump
//! sees only what is left), as dpnet's pool would run the callbacks on its own thread.

use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

use crate::dplay8::address::{Component, DpAddress};
use crate::dplay8::client::{Dplay8Client, DpnApplicationDesc, DpnCaps, DpnConnectionInfo};
use crate::dplay8::core::Indication;
use crate::dplay8::messages::DpnHandle;
use crate::gundll::dp8::{
    Dp8AddrData, Dp8Address, Dp8ApplicationDesc, Dp8Caps, Dp8ConnectionInfo, Dp8Factory,
    Dp8Message, Dp8MessageHandler, IDirectPlay8Client, IDirectPlay8ThreadPool, DPNSUCCESS_PENDING,
};
use crate::interfaces::Guid;
use crate::Hresult;

/// A `Guid` as the 16 bytes COM stores (data1 to data3 little endian).
pub fn guid_bytes(g: &Guid) -> [u8; 16] {
    let mut b = [0u8; 16];
    b[0..4].copy_from_slice(&g.data1.to_le_bytes());
    b[4..6].copy_from_slice(&g.data2.to_le_bytes());
    b[6..8].copy_from_slice(&g.data3.to_le_bytes());
    b[8..16].copy_from_slice(&g.data4);
    b
}

/// `Dp8Address` as an `IDirectPlay8Address` after `SetSP` and the `AddComponent` calls.
pub fn to_dp_address(a: &Dp8Address) -> DpAddress {
    let mut d = DpAddress::new();
    d.set_sp(guid_bytes(&a.sp));
    for (name, data) in &a.components {
        let c = match data {
            Dp8AddrData::String(s) => Component::String(s.clone()),
            Dp8AddrData::Dword(v) => Component::Dword(*v),
            Dp8AddrData::Guid(g) => Component::Guid(guid_bytes(g)),
        };
        d.add_component(name, c);
    }
    d
}

fn to_app_desc(a: &Dp8ApplicationDesc) -> DpnApplicationDesc {
    DpnApplicationDesc {
        size: a.size,
        flags: a.flags,
        guid_instance: guid_bytes(&a.guid_instance),
        guid_application: guid_bytes(&a.guid_application),
        max_players: 0,
        current_players: 0,
        session_name: None,
        password: None,
    }
}

/// The process-wide thread pool state: the clients whose work `DoWork` runs.
#[derive(Default)]
struct PoolShared {
    clients: RefCell<Vec<Weak<Dx9Client>>>,
    /// Impl `+4` bit 0 (set by `Initialize`, FUN_00428e62 dpnet9.dll.c:35322).
    initialized: Cell<bool>,
    /// Impl `+0x14`: the explicit thread count, `u32::MAX` while unset (-1).
    thread_count: Cell<u32>,
    /// Impl `+4` bit 1: a `DoWork` is running (so are the callbacks it makes).
    in_work: Cell<bool>,
    /// Handlers running on this thread (`FUN_00407c66` lists the threads inside a callback).
    in_callback: Cell<u32>,
}

/// `CoCreateInstance` for the DX9 client and thread pool.
pub struct Dx9Dp8Factory {
    make: Box<dyn Fn() -> Dplay8Client>,
    pool: Rc<PoolShared>,
}

impl Dx9Dp8Factory {
    /// A factory whose clients come from `make` (tests give fake sockets and a fake clock).
    pub fn new(make: Box<dyn Fn() -> Dplay8Client>) -> Self {
        Dx9Dp8Factory {
            make,
            pool: Rc::new(PoolShared {
                thread_count: Cell::new(u32::MAX),
                ..PoolShared::default()
            }),
        }
    }

    /// The real clock, UDP sockets, resolver and adapters.
    pub fn system() -> Self {
        Self::new(Box::new(Dplay8Client::with_system))
    }
}

impl Dp8Factory for Dx9Dp8Factory {
    fn create_client(&self) -> Result<Rc<dyn IDirectPlay8Client>, Hresult> {
        let c = Rc::new(Dx9Client {
            inner: RefCell::new((self.make)()),
            handler: RefCell::new(None),
            pool: self.pool.clone(),
        });
        self.pool.clients.borrow_mut().push(Rc::downgrade(&c));
        Ok(c)
    }

    fn create_thread_pool(&self) -> Result<Rc<dyn IDirectPlay8ThreadPool>, Hresult> {
        // CoCreateInstance then `Initialize(0, pfn, 0)`; `DPNERR_ALREADYINITIALIZED` (the
        // singleton, 0x80158080) counts as success for GunDll.
        self.pool.initialized.set(true);
        Ok(Rc::new(Dx9ThreadPool {
            shared: self.pool.clone(),
        }))
    }
}

/// `IDirectPlay8Client` for `CGun`.
pub struct Dx9Client {
    inner: RefCell<Dplay8Client>,
    handler: RefCell<Option<Rc<dyn Dp8MessageHandler>>>,
    pool: Rc<PoolShared>,
}

impl Dx9Client {
    /// Hand the indications to the handler with no borrow of the client held, settling each
    /// `RECEIVE` afterwards (a handler answer of `DPNSUCCESS_PENDING` keeps the buffer).
    fn deliver(&self, inds: Vec<Indication>) {
        let handler = self.handler.borrow().clone();
        self.pool.in_callback.set(self.pool.in_callback.get() + 1);
        for ind in inds {
            let Some(h) = handler.as_ref() else {
                continue;
            };
            match ind {
                Indication::ConnectComplete(m) => {
                    h.message(&Dp8Message::ConnectComplete {
                        hresult: m.result,
                        reply: m.reply_data,
                    });
                }
                Indication::SendComplete(m) => {
                    h.message(&Dp8Message::SendComplete {
                        context: m.user_context as u32,
                        hresult: m.result,
                    });
                }
                Indication::TerminateSession(m) => {
                    h.message(&Dp8Message::TerminateSession {
                        hresult: m.result,
                        data: m.data,
                    });
                }
                Indication::Receive {
                    data,
                    buffer,
                    e0_context,
                } => {
                    let r = h.message(&Dp8Message::Receive {
                        data: data.clone(),
                        buffer_handle: buffer.0,
                    });
                    self.inner
                        .borrow_mut()
                        .finish_receive(buffer, data, e0_context, r);
                }
            }
        }
        self.pool.in_callback.set(self.pool.in_callback.get() - 1);
    }

    /// One pump of this client: the thread pool's work for it.
    fn pump(&self) {
        let inds = self.inner.borrow_mut().do_work_collect();
        self.deliver(inds);
    }
}

impl IDirectPlay8Client for Dx9Client {
    fn initialize(&self, handler: Rc<dyn Dp8MessageHandler>, flags: u32) -> Hresult {
        let hr = self.inner.borrow_mut().initialize_external(flags);
        if hr == Hresult::S_OK {
            *self.handler.borrow_mut() = Some(handler);
        }
        hr
    }

    fn cancel_async_operation(&self, handle: u32, flags: u32) -> Hresult {
        self.inner
            .borrow_mut()
            .cancel_async_operation(DpnHandle(handle), flags)
    }

    fn connect(
        &self,
        app_desc: &Dp8ApplicationDesc,
        host: &Dp8Address,
        device: &Dp8Address,
    ) -> (Hresult, u32) {
        let r = self.inner.borrow_mut().connect(
            &to_app_desc(app_desc),
            &to_dp_address(host),
            &to_dp_address(device),
            false,
            false,
            None,
            0,
            0,
        );
        match r {
            Ok(h) => (DPNSUCCESS_PENDING, h.0),
            Err(e) => (e, 0),
        }
    }

    fn send(&self, buffer: &[u8], timeout: u32, context: u32, flags: u32) -> Hresult {
        match self
            .inner
            .borrow_mut()
            .send(&[buffer], timeout, context as usize, flags)
        {
            Ok(_) => DPNSUCCESS_PENDING,
            Err(e) => e,
        }
    }

    fn close(&self, flags: u32) -> Hresult {
        // DX9 0x0040b856 (dpnet9.dll.c:7594-7599): a Close from inside a callback of the
        // calling thread is `DPNERR_NOTALLOWED`.
        if self.pool.in_work.get() || self.pool.in_callback.get() > 0 {
            return Hresult(0x8015_8520);
        }
        let (hr, inds) = {
            let mut c = self.inner.borrow_mut();
            let hr = c.close(flags);
            (hr, c.take_indications())
        };
        self.deliver(inds);
        hr
    }

    fn return_buffer(&self, buffer_handle: u32, flags: u32) -> Hresult {
        self.inner
            .borrow_mut()
            .return_buffer(DpnHandle(buffer_handle), flags)
    }

    fn get_caps(&self, caps: &mut Dp8Caps, _flags: u32) -> Hresult {
        let mut c = DpnCaps {
            size: caps.size,
            flags: caps.flags,
            ..DpnCaps::default()
        };
        let hr = self.inner.borrow().get_caps(&mut c, 0);
        if hr == Hresult::S_OK {
            caps.connect_timeout = c.connect_timeout;
            caps.connect_retries = c.connect_retries;
            caps.timeout_until_keep_alive = c.timeout_until_keep_alive;
        }
        hr
    }

    fn get_connection_info(&self, info: &mut Dp8ConnectionInfo, _flags: u32) -> Hresult {
        let mut c = DpnConnectionInfo {
            size: info.size,
            ..DpnConnectionInfo::default()
        };
        let hr = self.inner.borrow().get_connection_info(&mut c, 0);
        if hr == Hresult::S_OK {
            info.round_trip_latency_ms = c.round_trip_latency_ms;
        }
        hr
    }

    fn set_caps(&self, caps: &Dp8Caps, _flags: u32) -> Hresult {
        self.inner.borrow_mut().set_caps(
            &DpnCaps {
                size: caps.size,
                flags: caps.flags,
                connect_timeout: caps.connect_timeout,
                connect_retries: caps.connect_retries,
                timeout_until_keep_alive: caps.timeout_until_keep_alive,
            },
            0,
        )
    }
}

/// `IDirectPlay8ThreadPool` for `CGun`.
pub struct Dx9ThreadPool {
    shared: Rc<PoolShared>,
}

impl IDirectPlay8ThreadPool for Dx9ThreadPool {
    /// `SetThreadCount` (FUN_00429796, dpnet9.dll.c:35716): validator FUN_0042a6b0 (count -1 or
    /// above 5000: 0x80070057; flags: 0x80158340), not initialised 0x80158640, a `DoWork` running
    /// 0x80158520; processor -1 stores the explicit count (.c:35764 for 0).
    // open: per-processor counts and real worker threads are not modelled (GunDll passes -1, 0).
    fn set_thread_count(&self, _processor_num: u32, num_threads: u32, flags: u32) -> Hresult {
        if num_threads == u32::MAX || num_threads > 5000 {
            return Hresult(0x8007_0057);
        }
        if flags != 0 {
            return Hresult(0x8015_8340);
        }
        if !self.shared.initialized.get() {
            return Hresult(0x8015_8640);
        }
        if self.shared.in_work.get() {
            return Hresult(0x8015_8520);
        }
        self.shared.thread_count.set(num_threads);
        Hresult::S_OK
    }

    /// `DoWork` (vtable 0x00428f5d, asm only; validator FUN_0042a706 .c:36774): a slice of -1 or
    /// below 60001 (else 0x80070057), flags 0 (else 0x80158340), initialised (0x80158640), an
    /// explicit thread count of 0 (else 0x80158540 NOTREADY), and no `DoWork` running: a
    /// re-entrant one, e.g. from a callback, is 0x80158520, which is the only NOTALLOWED. Runs
    /// every client's work on the calling thread.
    // open: DX9 returns DPNSUCCESS_PENDING (0x0015800E) while any queue still holds work; this
    // returns S_OK once the pump drained (it loops until idle).
    fn do_work(&self, allowed_time_slice: u32, flags: u32) -> Hresult {
        if allowed_time_slice != u32::MAX && allowed_time_slice >= 60001 {
            return Hresult(0x8007_0057);
        }
        if flags != 0 {
            return Hresult(0x8015_8340);
        }
        if !self.shared.initialized.get() {
            return Hresult(0x8015_8640);
        }
        if self.shared.thread_count.get() != 0 {
            return Hresult(0x8015_8540);
        }
        if self.shared.in_work.replace(true) {
            return Hresult(0x8015_8520);
        }
        let clients: Vec<Rc<Dx9Client>> = {
            let mut list = self.shared.clients.borrow_mut();
            list.retain(|w| w.strong_count() > 0);
            list.iter().filter_map(Weak::upgrade).collect()
        };
        for c in clients {
            c.pump();
        }
        self.shared.in_work.set(false);
        Hresult::S_OK
    }
}

#[cfg(test)]
mod tests;
