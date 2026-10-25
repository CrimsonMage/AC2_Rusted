//! An in-process lossy proxy (test tooling): wraps any client [`Transport`] (a UDP socket to a
//! live server, or the double) and passes every datagram, both ways, through a seeded
//! [`Channel`], so a live probe sees loss, duplicates and reordering without touching the
//! system's network.

use std::cell::RefCell;
use std::net::SocketAddr;
use std::rc::Rc;
use std::time::Duration;

use ac2_net::{NetError, Transport};

use crate::channel::{Channel, Direction};

/// The proxy: the client's transport behind a lossy channel.
pub struct LossyTransport {
    inner: Box<dyn Transport>,
    channel: Rc<RefCell<Channel>>,
    ready: Vec<(SocketAddr, Vec<u8>)>,
}

impl LossyTransport {
    /// Wraps `inner`; the returned handle reads the channel's stats while the client runs.
    pub fn new(inner: Box<dyn Transport>, channel: Channel) -> (Self, Rc<RefCell<Channel>>) {
        let channel = Rc::new(RefCell::new(channel));
        (
            Self {
                inner,
                channel: Rc::clone(&channel),
                ready: Vec::new(),
            },
            channel,
        )
    }
}

impl Transport for LossyTransport {
    /// Held datagrams now due go on (to the server through the inner transport, to the client
    /// in this batch), then what the inner transport received passes the channel.
    fn receive(&mut self) -> Result<Vec<(SocketAddr, Vec<u8>)>, NetError> {
        let now = self.inner.clock().0;
        let due = self.channel.borrow_mut().release(now);
        for flight in due {
            match flight.direction {
                Direction::ToServer => self.inner.send(flight.addr, &flight.bytes)?,
                Direction::ToClient => self.ready.push((flight.addr, flight.bytes)),
            }
        }
        for (from, bytes) in self.inner.receive()? {
            let passed = self
                .channel
                .borrow_mut()
                .send(now, Direction::ToClient, from, bytes);
            self.ready
                .extend(passed.into_iter().map(|flight| (flight.addr, flight.bytes)));
        }
        Ok(std::mem::take(&mut self.ready))
    }

    fn send(&mut self, to: SocketAddr, datagram: &[u8]) -> Result<(), NetError> {
        let now = self.inner.clock().0;
        let passed =
            self.channel
                .borrow_mut()
                .send(now, Direction::ToServer, to, datagram.to_vec());
        for flight in passed {
            self.inner.send(flight.addr, &flight.bytes)?;
        }
        Ok(())
    }

    fn clock(&mut self) -> (f64, i64) {
        self.inner.clock()
    }

    fn sleep(&mut self, duration: Duration) {
        self.inner.sleep(duration);
    }
}
