//! A narrow view of the logon stages for tools: what the net layer saw that the
//! [`SessionEvent`](super::SessionEvent) stream folds away (the logged-on notice, the DAT
//! interrogation, the exact net errors). It changes nothing the session sends.

use ac2_net::NetError;

use super::OnlineSession;

/// One logon stage, reported to the observer as it happens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionTrace {
    /// `ClientNet::HandleConnectionRequest`: a server answered the logon with its connect
    /// header.
    Connected { net_id: u32 },
    /// `Net_LoginConnected` (`ClientNet::EndState`): the first packet after the connect-ack.
    LoggedOn,
    /// `CLCache::OnServerInterrogation`: the server asked for the DAT iterations; its payload
    /// length.
    Interrogation { payload_len: usize },
    /// `CLCache::OnServerInterrogation`'s answer went out: the language and how many DAT files
    /// it lists.
    InterrogationAnswered { language: u32, dat_files: usize },
    /// A queue 1 event that did not decode (an opcode or layout this client does not know):
    /// its opcode, length in bytes and the decoder's reason.
    Undecoded {
        opcode: u32,
        length: usize,
        error: String,
    },
    /// `Net_LoginConnectionError` / `Net_WorldConnectionError`: the server is closing the
    /// connection for this reason.
    ConnectionError { world: bool, error: NetError },
    /// The session ended with this reason (`ClientNet::EndState` for a refused logon,
    /// `RemoveConnection`, the logon timeout).
    Disconnected(NetError),
}

impl OnlineSession {
    /// Reports each [`SessionTrace`] to `observer` from now on (a tool's progress output).
    pub fn set_observer(&mut self, observer: Box<dyn FnMut(&SessionTrace) + Send>) {
        self.observer = Some(observer);
    }

    pub(super) fn trace(&mut self, trace: SessionTrace) {
        if let Some(observer) = self.observer.as_mut() {
            observer(&trace);
        }
    }
}
