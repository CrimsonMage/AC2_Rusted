//! `NSTurbineGun::TurbineGun` (Legions 0x005C58D0 to 0x005C8090): the request wrappers, the
//! `Rcv*Result` / `Rcv*Event` decoders and the `Name@World` mapping, sans IO.

use std::collections::{BTreeMap, VecDeque};

use super::xml::{frame, Elem, FrameReader};

/// `ZT_E_ALREADY_IN_LIST`.
pub const ZT_E_ALREADY_IN_LIST: u32 = 0xC112_1009;
/// `ZT_E_NOT_IN_LIST`.
pub const ZT_E_NOT_IN_LIST: u32 = 0xC112_1401;
/// `ZT_E_ACCOUNT_BANNED`: `gmClient::StartupGUN` stops with "your account is banned".
pub const ZT_E_ACCOUNT_BANNED: u32 = 0xC112_1005;

/// The ProtoVer of AC2e's GUN transport.
const PROTO_VERSION: &str = "1";

#[derive(Debug)]
pub enum GunError {
    Io(std::io::Error),
    /// A frame that is not a well-formed element, or a length out of range.
    Protocol(&'static str),
}

impl std::fmt::Display for GunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "GUN I/O: {e}"),
            Self::Protocol(why) => write!(f, "GUN protocol: {why}"),
        }
    }
}

impl std::error::Error for GunError {}

impl From<std::io::Error> for GunError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

/// `Gun::UserStatus`: a friend and what the service says of them. `mode` is the status bit set
/// (1 logged in, 2 away, 4 do not disturb, 8 in game, 16 in chat).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GunUserStatus {
    pub name: String,
    pub mode: u32,
    pub location: String,
}

/// What `TurbineGun::CheckForMessages` hands the client's listeners (`CCli*Event`, `CCli*Result`).
/// Names are local: `@World` is gone when it is the client's world (`GlobalToLocalName`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GunEvent {
    /// `CTConnectEvent`: the service accepted the connection (`m_fConnected`).
    Connected,
    /// The connection failed or the service refused it.
    ConnectFailed,
    /// The connection dropped after it had been made.
    Disconnected,
    /// `RcvAuthenticateResult`: the answer to the presented ticket; `hr` 0 is authenticated,
    /// `ZT_E_ACCOUNT_BANNED` is banned.
    TicketResult {
        hr: u32,
    },
    /// `RcvFriendsEvent` `FriendStart`: the friends snapshot after the ticket.
    FriendStart(Vec<GunUserStatus>),
    /// `RcvFriendsEvent` `FriendStatus`: one friend's new mode.
    FriendStatus(GunUserStatus),
    /// `RcvFriendsResult` `GetFriendsList`.
    FriendsList {
        user_requested: bool,
        hr: u32,
        friends: Vec<GunUserStatus>,
    },
    FriendAdd {
        hr: u32,
        name: String,
    },
    FriendDelete {
        hr: u32,
        name: String,
    },
    /// `RcvFriendsResult` `GetIgnoredList`.
    IgnoredList {
        user_requested: bool,
        hr: u32,
        names: Vec<String>,
    },
    IgnoredAdd {
        hr: u32,
        name: String,
    },
    IgnoredDelete {
        hr: u32,
        name: String,
    },
    /// `RcvFriendsResult` `Flags`: the word asked about and its value.
    Flags {
        hr: u32,
        which: u32,
        flags: u32,
    },
    /// `RcvFriendsResult` `SetLocation`.
    Location {
        hr: u32,
    },
    /// `RcvChatResult` `SendToUserEx`.
    TellResult {
        hr: u32,
    },
    /// `RcvChatEvent` `UserSpeak`: a tell from `from`.
    UserTell {
        from: String,
        text: String,
        flags: u32,
    },
}

/// What a request in flight is, to turn its answer into the event.
#[derive(Debug, Clone)]
enum Pending {
    Ticket,
    ReleaseTicket,
    GetFriends { user_requested: bool },
    FriendAdd(String),
    FriendDelete(String),
    GetIgnored { user_requested: bool },
    IgnoredAdd(String),
    IgnoredDelete(String),
    Flags,
    SetLocation,
    Tell,
}

/// `NSTurbineGun::TurbineGun`.
#[derive(Debug)]
pub struct TurbineGun {
    /// `m_worldName`, set once (`SetWorldName`); empty until then.
    world_name: String,
    /// `m_worldSuffix`: `@` and the world's name.
    world_suffix: String,
    /// `m_fConnected`.
    connected: bool,
    connecting: bool,
    ticket: Option<Vec<u8>>,
    reader: FrameReader,
    output: Vec<u8>,
    next_seq: u32,
    pending: BTreeMap<u32, Pending>,
    events: VecDeque<GunEvent>,
}

impl Default for TurbineGun {
    fn default() -> Self {
        Self::new()
    }
}

impl TurbineGun {
    pub fn new() -> Self {
        Self {
            world_name: String::new(),
            world_suffix: String::new(),
            connected: false,
            connecting: false,
            ticket: None,
            reader: FrameReader::default(),
            output: Vec::new(),
            next_seq: 0,
            pending: BTreeMap::new(),
            events: VecDeque::new(),
        }
    }

    /// `TurbineGun::SetWorldName`: only the first name sticks (retail returns 0 once the name
    /// is set); the suffix is `@` and the name. Returns whether it was set.
    pub fn set_world_name(&mut self, world: &str) -> bool {
        if !self.world_name.is_empty() {
            return false;
        }
        self.world_name = world.to_owned();
        self.world_suffix = format!("@{world}");
        true
    }

    pub fn world_name(&self) -> &str {
        &self.world_name
    }

    /// `TurbineGun::LocalToGlobalName`: a name with no `@` gets the world's suffix.
    pub fn local_to_global_name(&self, name: &str) -> String {
        if name.contains('@') {
            name.to_owned()
        } else {
            format!("{name}{}", self.world_suffix)
        }
    }

    /// `TurbineGun::GlobalToLocalName`: a name with an `@` loses it when what follows is this
    /// client's world; other worlds keep their suffix.
    pub fn global_to_local_name(&self, name: &str) -> String {
        match name.split_once('@') {
            Some((local, world)) if world == self.world_name => local.to_owned(),
            _ => name.to_owned(),
        }
    }

    /// `m_fConnected`: the service has accepted the connection.
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// `TurbineGun::SetAuthenticationTicket`: the ticket bytes (`-gunticketoverride`).
    pub fn set_authentication_ticket(&mut self, ticket: &[u8]) {
        self.ticket = Some(ticket.to_vec());
    }

    /// `TurbineGun::GetAuthenticationTicket`: the ticket, when there is one.
    pub fn authentication_ticket(&self) -> Option<&[u8]> {
        self.ticket.as_deref()
    }

    /// The bytes to send since the last call.
    pub fn take_output(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.output)
    }

    /// The events since the last call, in order.
    pub fn take_events(&mut self) -> Vec<GunEvent> {
        self.events.drain(..).collect()
    }

    fn send(&mut self, elem: &Elem) {
        self.output.extend_from_slice(&frame(elem));
    }

    fn request(&mut self, elem: Elem, pending: Pending) -> bool {
        if !self.connected {
            return false;
        }
        self.next_seq += 1;
        let seq = self.next_seq;
        self.pending.insert(seq, pending);
        self.send(&elem.attr("seq", seq.to_string()));
        true
    }

    /// `TurbineGun::RequestConnect`: the first frame. The result is [`GunEvent::Connected`]
    /// (retail's `ConnectEv`) or [`GunEvent::ConnectFailed`].
    pub fn request_connect(&mut self) -> bool {
        if self.connecting || self.connected {
            return false;
        }
        self.connecting = true;
        self.send(&Elem::new("Connect").attr("ProtoVer", PROTO_VERSION));
        true
    }

    /// The transport could not be made (or ended before `ConnectRes`).
    pub fn connection_failed(&mut self) {
        if self.connecting {
            self.connecting = false;
            self.events.push_back(GunEvent::ConnectFailed);
        }
    }

    /// The transport ended: `DisconnectEv` once it had connected.
    pub fn connection_lost(&mut self) {
        if self.connected {
            self.connected = false;
            self.pending.clear();
            self.events.push_back(GunEvent::Disconnected);
        } else {
            self.connection_failed();
        }
    }

    /// `TurbineGun::RequestPresentAuthenticationTicket`: presents the stored ticket.
    pub fn request_present_authentication_ticket(&mut self) -> bool {
        let Some(ticket) = self.ticket.clone() else {
            return false;
        };
        let hex: String = ticket.iter().map(|b| format!("{b:02x}")).collect();
        self.request(
            Elem::new("PresentTicketCmd").attr("Ticket", hex),
            Pending::Ticket,
        )
    }

    /// `TurbineGun::RequestFinishWithTicket`.
    pub fn request_finish_with_ticket(&mut self) -> bool {
        self.request(Elem::new("ReleaseTicketCmd"), Pending::ReleaseTicket)
    }

    /// `TurbineGun::RequestGetFriendsList(userRequested)`.
    pub fn request_get_friends_list(&mut self, user_requested: bool) -> bool {
        self.request(
            Elem::new("GetFriendsList"),
            Pending::GetFriends { user_requested },
        )
    }

    /// `TurbineGun::RequestGetIgnoredList(userRequested)`.
    pub fn request_get_ignored_list(&mut self, user_requested: bool) -> bool {
        self.request(
            Elem::new("GetIgnoredList"),
            Pending::GetIgnored { user_requested },
        )
    }

    /// `TurbineGun::RequestFriendAdd` (the name goes out as `LocalToGlobalName` makes it).
    pub fn request_friend_add(&mut self, name: &str) -> bool {
        let global = self.local_to_global_name(name);
        self.request(
            Elem::new("FriendAdd").attr("Name", global),
            Pending::FriendAdd(name.to_owned()),
        )
    }

    pub fn request_friend_delete(&mut self, name: &str) -> bool {
        let global = self.local_to_global_name(name);
        self.request(
            Elem::new("FriendDelete").attr("Name", global),
            Pending::FriendDelete(name.to_owned()),
        )
    }

    pub fn request_ignored_add(&mut self, name: &str) -> bool {
        let global = self.local_to_global_name(name);
        self.request(
            Elem::new("IgnoredAdd").attr("Name", global),
            Pending::IgnoredAdd(name.to_owned()),
        )
    }

    pub fn request_ignored_delete(&mut self, name: &str) -> bool {
        let global = self.local_to_global_name(name);
        self.request(
            Elem::new("IgnoredDelete").attr("Name", global),
            Pending::IgnoredDelete(name.to_owned()),
        )
    }

    /// `TurbineGun::RequestFlags(which, op, flags)` (see [`super::flag_op`], [`super::flag_word`]).
    pub fn request_flags(&mut self, which: u32, op: u32, flags: u32) -> bool {
        self.request(
            Elem::new("Flags")
                .attr("Op", op.to_string())
                .attr("Which", which.to_string())
                .attr("Flags", flags.to_string()),
            Pending::Flags,
        )
    }

    /// `TurbineGun::RequestSetLocation(flags, location)`.
    pub fn request_set_location(&mut self, flags: u32, location: &str) -> bool {
        self.request(
            Elem::new("SetLoc")
                .attr("Flags", flags.to_string())
                .attr("Location", location),
            Pending::SetLocation,
        )
    }

    /// `TurbineGun::RequestSendToUserEx`.
    pub fn request_send_to_user(&mut self, user: &str, text: &str, flags: u32) -> bool {
        let global = self.local_to_global_name(user);
        self.request(
            Elem::new("SendToUserEx")
                .attr("Name", global)
                .attr("Text", text)
                .attr("Flags", flags.to_string()),
            Pending::Tell,
        )
    }

    /// Bytes from the service. A frame that does not decode ends the link ([`GunError::Protocol`]).
    pub fn receive(&mut self, bytes: &[u8]) -> Result<(), GunError> {
        self.reader.push(bytes);
        while let Some(elem) = self.reader.next_element().map_err(GunError::Protocol)? {
            self.on_message(&elem);
        }
        Ok(())
    }

    fn status(&self, elem: &Elem) -> GunUserStatus {
        GunUserStatus {
            name: self.global_to_local_name(elem.get("Name").unwrap_or("")),
            mode: number(elem.get("Mode")),
            location: elem.get("Location").unwrap_or("").to_owned(),
        }
    }

    fn on_message(&mut self, elem: &Elem) {
        match elem.name.as_str() {
            "ConnectRes" => {
                if self.connecting {
                    self.connecting = false;
                    if hr_of(elem) == 0 {
                        self.connected = true;
                        self.events.push_back(GunEvent::Connected);
                    } else {
                        self.events.push_back(GunEvent::ConnectFailed);
                    }
                }
            }
            "FStart" => {
                let friends = elem.children.iter().map(|c| self.status(c)).collect();
                self.events.push_back(GunEvent::FriendStart(friends));
            }
            "FriendStat" => {
                let status = self.status(elem);
                self.events.push_back(GunEvent::FriendStatus(status));
            }
            "UTellEv" => {
                let from = self.global_to_local_name(elem.get("From").unwrap_or(""));
                self.events.push_back(GunEvent::UserTell {
                    from,
                    text: elem.get("Text").unwrap_or("").to_owned(),
                    flags: number(elem.get("Flags")),
                });
            }
            _ => {
                let Some(seq) = elem.get("seq").and_then(|s| s.parse::<u32>().ok()) else {
                    return;
                };
                let Some(pending) = self.pending.remove(&seq) else {
                    return;
                };
                let hr = hr_of(elem);
                let event = match pending {
                    Pending::Ticket => GunEvent::TicketResult { hr },
                    Pending::ReleaseTicket => return,
                    Pending::GetFriends { user_requested } => GunEvent::FriendsList {
                        user_requested,
                        hr,
                        friends: elem.children.iter().map(|c| self.status(c)).collect(),
                    },
                    Pending::FriendAdd(name) => GunEvent::FriendAdd { hr, name },
                    Pending::FriendDelete(name) => GunEvent::FriendDelete { hr, name },
                    Pending::GetIgnored { user_requested } => GunEvent::IgnoredList {
                        user_requested,
                        hr,
                        names: elem
                            .children
                            .iter()
                            .map(|c| self.global_to_local_name(c.get("Name").unwrap_or("")))
                            .collect(),
                    },
                    Pending::IgnoredAdd(name) => GunEvent::IgnoredAdd { hr, name },
                    Pending::IgnoredDelete(name) => GunEvent::IgnoredDelete { hr, name },
                    Pending::Flags => GunEvent::Flags {
                        hr,
                        which: number(elem.get("WhichFlag")),
                        flags: number(elem.get("Flags")),
                    },
                    Pending::SetLocation => GunEvent::Location { hr },
                    Pending::Tell => GunEvent::TellResult { hr },
                };
                self.events.push_back(event);
            }
        }
    }
}

fn number(text: Option<&str>) -> u32 {
    text.and_then(|t| t.parse().ok()).unwrap_or(0)
}

/// The `hr` attribute (`0x%08X`); a missing or malformed one is a failure.
fn hr_of(elem: &Elem) -> u32 {
    elem.get("hr")
        .and_then(|t| t.strip_prefix("0x"))
        .and_then(|t| u32::from_str_radix(t, 16).ok())
        .unwrap_or(0xC112_0003)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bytes(text: &str) -> Vec<u8> {
        let mut out = (text.len() as u32).to_be_bytes().to_vec();
        out.extend_from_slice(text.as_bytes());
        out
    }

    fn connected() -> TurbineGun {
        let mut gun = TurbineGun::new();
        gun.set_world_name("AC2e");
        assert!(gun.request_connect());
        gun.take_output();
        gun.receive(&bytes(r#"<ConnectRes hr="0x00000000" Random="00"/>"#))
            .unwrap();
        assert_eq!(gun.take_events(), vec![GunEvent::Connected]);
        gun
    }

    #[test]
    fn connect_frame_is_exact() {
        let mut gun = TurbineGun::new();
        assert!(gun.request_connect());
        assert_eq!(gun.take_output(), bytes(r#"<Connect ProtoVer="1"/>"#));
        assert!(!gun.request_connect(), "already connecting");
    }

    #[test]
    fn requests_refuse_before_the_connect_event() {
        let mut gun = TurbineGun::new();
        assert!(!gun.request_get_friends_list(true));
        assert!(!gun.request_friend_add("Bob"));
        assert!(gun.take_output().is_empty());
    }

    #[test]
    fn ticket_and_friends_frames_are_exact() {
        let mut gun = connected();
        assert!(
            !gun.request_present_authentication_ticket(),
            "no ticket yet"
        );
        gun.set_authentication_ticket(b"alice");
        assert_eq!(gun.authentication_ticket(), Some(&b"alice"[..]));
        assert!(gun.request_present_authentication_ticket());
        assert!(gun.request_friend_add("Bob"));
        assert!(gun.request_friend_delete("Zed@Other"));
        assert!(gun.request_get_friends_list(true));
        assert!(gun.request_get_ignored_list(false));
        assert!(gun.request_ignored_add("Pest"));
        assert!(gun.request_ignored_delete("Pest"));
        assert!(gun.request_flags(1, 1, 6));
        assert!(gun.request_set_location(2, "Holtburg"));
        assert!(gun.request_send_to_user("Bob", "hi & <bye>", 0));
        let mut expect = Vec::new();
        for text in [
            r#"<PresentTicketCmd Ticket="616c696365" seq="1"/>"#,
            r#"<FriendAdd Name="Bob@AC2e" seq="2"/>"#,
            r#"<FriendDelete Name="Zed@Other" seq="3"/>"#,
            r#"<GetFriendsList seq="4"/>"#,
            r#"<GetIgnoredList seq="5"/>"#,
            r#"<IgnoredAdd Name="Pest@AC2e" seq="6"/>"#,
            r#"<IgnoredDelete Name="Pest@AC2e" seq="7"/>"#,
            r#"<Flags Op="1" Which="1" Flags="6" seq="8"/>"#,
            r#"<SetLoc Flags="2" Location="Holtburg" seq="9"/>"#,
            r#"<SendToUserEx Name="Bob@AC2e" Text="hi &amp; &lt;bye&gt;" Flags="0" seq="10"/>"#,
        ] {
            expect.extend(bytes(text));
        }
        assert_eq!(gun.take_output(), expect);
    }

    #[test]
    fn results_and_events_decode() {
        let mut gun = connected();
        gun.set_authentication_ticket(b"a");
        gun.request_present_authentication_ticket();
        gun.request_friend_add("Bob");
        gun.request_get_friends_list(true);
        gun.request_flags(1, 1, 6);
        gun.take_output();
        let wire = [
            r#"<PresentTicketRes seq="1" hr="0x00000000" Status="0" Account="a"/>"#,
            r#"<FStart><Friend Name="Bob@AC2e" Mode="1" Location=""/><Friend Name="Cy@Elsewhere" Mode="0" Location=""/></FStart>"#,
            r#"<FriendAddRes seq="2" hr="0xC1121009"/>"#,
            r#"<GetFriendsListRes seq="3" hr="0x00000000"><Friend Name="Bob@AC2e" Mode="9" Location="Holtburg"/></GetFriendsListRes>"#,
            r#"<FlagsRes seq="4" hr="0x00000000" WhichFlag="1" Flags="6"/>"#,
            r#"<FriendStat Name="Bob@AC2e" Mode="3" Location="x"/>"#,
            r#"<UTellEv From="Bob@AC2e" Text="hi" Flags="0"/>"#,
        ]
        .map(bytes)
        .concat();
        // Delivered in two pieces: a frame may span reads.
        gun.receive(&wire[..30]).unwrap();
        assert!(gun.take_events().is_empty());
        gun.receive(&wire[30..]).unwrap();
        let status = |name: &str, mode, location: &str| GunUserStatus {
            name: name.into(),
            mode,
            location: location.into(),
        };
        assert_eq!(
            gun.take_events(),
            vec![
                GunEvent::TicketResult { hr: 0 },
                GunEvent::FriendStart(vec![status("Bob", 1, ""), status("Cy@Elsewhere", 0, "")]),
                GunEvent::FriendAdd {
                    hr: ZT_E_ALREADY_IN_LIST,
                    name: "Bob".into()
                },
                GunEvent::FriendsList {
                    user_requested: true,
                    hr: 0,
                    friends: vec![status("Bob", 9, "Holtburg")]
                },
                GunEvent::Flags {
                    hr: 0,
                    which: 1,
                    flags: 6
                },
                GunEvent::FriendStatus(status("Bob", 3, "x")),
                GunEvent::UserTell {
                    from: "Bob".into(),
                    text: "hi".into(),
                    flags: 0
                },
            ]
        );
    }

    #[test]
    fn world_name_sticks_once_and_maps_names() {
        let mut gun = TurbineGun::new();
        assert_eq!(gun.local_to_global_name("Bob"), "Bob", "no world yet");
        assert!(gun.set_world_name("AC2e"));
        assert!(!gun.set_world_name("Other"));
        assert_eq!(gun.local_to_global_name("Bob"), "Bob@AC2e");
        assert_eq!(gun.local_to_global_name("Bob@Other"), "Bob@Other");
        assert_eq!(gun.global_to_local_name("Bob@AC2e"), "Bob");
        assert_eq!(gun.global_to_local_name("Bob@Other"), "Bob@Other");
        assert_eq!(gun.global_to_local_name("Bob"), "Bob");
    }

    #[test]
    fn connect_refusal_loss_and_bad_frames() {
        let mut gun = TurbineGun::new();
        gun.request_connect();
        gun.receive(&bytes(r#"<ConnectRes hr="0xC1120405" Random=""/>"#))
            .unwrap();
        assert_eq!(gun.take_events(), vec![GunEvent::ConnectFailed]);
        let mut gun = connected();
        gun.request_get_friends_list(false);
        gun.connection_lost();
        assert_eq!(gun.take_events(), vec![GunEvent::Disconnected]);
        assert!(!gun.is_connected());
        assert!(gun.receive(&[0, 0, 0, 0]).is_err(), "zero length");
        let mut gun = TurbineGun::new();
        assert!(gun.receive(&bytes("not xml")).is_err());
    }
}
