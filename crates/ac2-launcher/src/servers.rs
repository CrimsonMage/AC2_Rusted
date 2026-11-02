//! The servers the launcher knows, in ThwargLauncher's way: a server is a name, a connect string
//! (`host:port`) and an emulator type (`emu`), and the launcher builds the client's command line
//! for that type itself ([`crate::launch`]). The player's servers and a published server list
//! are both `ServerItem` XML (ThwargLauncher's schema, `docs/server-list.md`): the player's with
//! `connect_string`, a published list with `server_host` and `server_port`.

use std::path::Path;

use serde::{Deserialize, Serialize};

/// The client data version a type of server takes (`--version`).
pub const PROTOCOL_7230: &str = ac2_net::RELEASE_7230_VERSION;
pub const PROTOCOL_RELAUNCH: &str = ac2_net::RELAUNCH_VERSION;

/// The kind of server, which decides the command line ([`crate::launch::args_for`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Emu {
    /// An AC2e server: the 7230 protocol (7230; AC2e serves its 7230 data to it), an account
    /// service for its tickets when
    /// it has one (`LoginServerUrlKey`), else the developer account.
    Ac2e,
    /// A relaunch server (2012-2017 client): protocol 1802, the password as the ticket
    /// (`glsticketdirect`).
    Relaunch,
}

impl Emu {
    pub const ALL: [Emu; 2] = [Emu::Ac2e, Emu::Relaunch];

    /// The `emu` element's text.
    pub fn name(self) -> &'static str {
        match self {
            Emu::Ac2e => "AC2E",
            Emu::Relaunch => "Relaunch",
        }
    }

    pub fn parse(text: &str) -> Option<Emu> {
        Emu::ALL
            .into_iter()
            .find(|e| e.name().eq_ignore_ascii_case(text.trim()))
    }

    pub fn protocol(self) -> &'static str {
        match self {
            Emu::Ac2e => PROTOCOL_7230,
            Emu::Relaunch => PROTOCOL_RELAUNCH,
        }
    }
}

/// Where a server came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    User,
    Published,
}

/// One server (a `ServerItem`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Server {
    pub id: String,
    pub name: String,
    pub description: String,
    pub emu: Emu,
    /// `host:port`.
    pub connect: String,
    /// An AC2e account service (`LoginServerUrlKey`); empty for none.
    pub login_server_url: String,
    pub website_url: String,
    pub discord_url: String,
    pub status: String,
    pub source: Source,
}

impl Server {
    pub fn new(name: &str, connect: &str, emu: Emu) -> Self {
        Self {
            id: new_id(),
            name: name.trim().to_owned(),
            description: String::new(),
            emu,
            connect: connect.trim().to_owned(),
            login_server_url: String::new(),
            website_url: String::new(),
            discord_url: String::new(),
            status: String::new(),
            source: Source::User,
        }
    }

    /// The connect string's host and port.
    pub fn address(&self) -> Result<(String, u16), String> {
        let (host, port) = self
            .connect
            .trim()
            .rsplit_once(':')
            .ok_or_else(|| format!("{}: the connect string needs host:port", self.name))?;
        let port: u16 = port
            .parse()
            .ok()
            .filter(|p| *p != 0)
            .ok_or_else(|| format!("{}: the port {port:?} is not valid", self.name))?;
        if host.is_empty() || host.contains(char::is_whitespace) {
            return Err(format!("{}: the host is not valid", self.name));
        }
        Ok((host.to_owned(), port))
    }

    /// The key a server is remembered by (its connect string).
    pub fn key(&self) -> String {
        self.connect.trim().to_ascii_lowercase()
    }

    /// What is wrong with the entry, if anything.
    pub fn problem(&self) -> Option<String> {
        if self.name.trim().is_empty() {
            return Some("a server needs a name".into());
        }
        if let Err(e) = self.address() {
            return Some(e);
        }
        let url = self.login_server_url.trim();
        if !(url.is_empty() || url.starts_with("https://") || url.starts_with("http://")) {
            return Some(format!(
                "{}: the login server must be a web address",
                self.name
            ));
        }
        if !url.is_empty() && self.emu != Emu::Ac2e {
            return Some(format!(
                "{}: only an AC2E server has a login server",
                self.name
            ));
        }
        None
    }
}

/// A new server id (ThwargLauncher keeps a GUID per server).
pub fn new_id() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let mix = now ^ (u128::from(std::process::id()) << 64);
    let h = format!("{mix:032x}");
    format!(
        "{}-{}-{}-{}-{}",
        &h[0..8],
        &h[8..12],
        &h[12..16],
        &h[16..20],
        &h[20..32]
    )
}

/// The server every launcher starts with: an AC2e server on this computer. Other servers are
/// added by their connect string and type, or come from a published list.
pub fn built_in() -> Vec<Server> {
    let mut local = Server::new("Local AC2e", "127.0.0.1:9000", Emu::Ac2e);
    local.id = "00000000-0000-0000-0000-00000000ac2e".into();
    local.description = "An AC2e server on this computer (scripts/ac2e-server.sh).".into();
    local.website_url = "https://github.com/Thwargle/AC2e".into();
    vec![local]
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// The player's servers as ThwargLauncher writes its user servers (`connect_string`).
pub fn write_servers(servers: &[Server]) -> String {
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<ArrayOfServerItem>\n");
    for s in servers {
        out.push_str("  <ServerItem>\n");
        for (tag, value) in [
            ("id", s.id.as_str()),
            ("name", &s.name),
            ("description", &s.description),
            ("emu", s.emu.name()),
            ("connect_string", &s.connect),
            ("LoginServerUrlKey", &s.login_server_url),
            ("website_url", &s.website_url),
            ("discord_url", &s.discord_url),
            ("status", &s.status),
        ] {
            out.push_str(&format!("    <{tag}>{}</{tag}>\n", escape(value)));
        }
        out.push_str("  </ServerItem>\n");
    }
    out.push_str("</ArrayOfServerItem>\n");
    out
}

/// Reads a `ServerItem` list (the player's or a published one): its usable servers, and what
/// was wrong with the others. Unknown elements are ignored; `connect_string`, else
/// `server_host` and `server_port`, is the address, as ThwargLauncher reads them.
pub fn parse_servers(text: &str, source: Source) -> Result<(Vec<Server>, Vec<String>), String> {
    let doc = roxmltree::Document::parse(text)
        .map_err(|e| format!("the server list does not read: {e}"))?;
    let mut servers: Vec<Server> = Vec::new();
    let mut problems = Vec::new();
    for item in doc.descendants().filter(|n| n.has_tag_name("ServerItem")) {
        let field = |tag: &str| {
            item.children()
                .find(|c| c.has_tag_name(tag))
                .and_then(|c| c.text())
                .unwrap_or("")
                .trim()
                .to_owned()
        };
        let name = field("name");
        let emu_text = field("emu");
        let Some(emu) = Emu::parse(&emu_text) else {
            // Another game's server (ThwargLauncher's ACE or GDLE): not an AC2 server.
            problems.push(format!("{name}: not an AC2 server (emu {emu_text:?})"));
            continue;
        };
        let mut connect = field("connect_string");
        if connect.is_empty() {
            connect = format!("{}:{}", field("server_host"), field("server_port"));
        }
        let server = Server {
            id: Some(field("id"))
                .filter(|i| !i.is_empty())
                .unwrap_or_else(new_id),
            name,
            description: field("description"),
            emu,
            connect,
            login_server_url: field("LoginServerUrlKey"),
            website_url: field("website_url"),
            discord_url: field("discord_url"),
            status: field("status"),
            source,
        };
        match server.problem() {
            Some(problem) => problems.push(problem),
            None if servers.iter().any(|s| s.key() == server.key()) => {
                problems.push(format!("{}: listed twice", server.name));
            }
            None => servers.push(server),
        }
    }
    Ok((servers, problems))
}

/// Fetches the published server list at `address`: a web address, or a file on this computer.
pub fn fetch_list(address: &str) -> Result<(Vec<Server>, Vec<String>), String> {
    let text = if address.starts_with("https://") || address.starts_with("http://") {
        ureq::get(address)
            .timeout(std::time::Duration::from_secs(15))
            .call()
            .map_err(|e| format!("the server list could not be fetched: {e}"))?
            .into_string()
            .map_err(|e| format!("the server list could not be read: {e}"))?
    } else {
        std::fs::read_to_string(address)
            .map_err(|e| format!("the server list could not be read: {e}"))?
    };
    parse_servers(&text, Source::Published)
}

/// The launcher's settings (`launcher.json`) and the player's servers (`launcher-servers.xml`),
/// both in the AC2 configuration directory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    /// The DAT folder the client reads (`--dat-dir`).
    #[serde(default)]
    pub dat_dir: String,
    /// The `ac2` program; empty for the one beside the launcher.
    #[serde(default)]
    pub client: String,
    /// The published server list's address (a web address or a file); empty for none.
    #[serde(default)]
    pub list_url: String,
    /// The account last used on each server, by its key. Passwords are never kept.
    #[serde(default)]
    pub accounts: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    pub selected: Option<String>,
    #[serde(skip, default = "built_in")]
    pub favorites: Vec<Server>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            dat_dir: std::env::var("AC2_DAT_DIR").unwrap_or_default(),
            client: String::new(),
            list_url: String::new(),
            accounts: Default::default(),
            selected: None,
            favorites: built_in(),
        }
    }
}

pub const SETTINGS_FILE: &str = "launcher.json";
pub const SERVERS_FILE: &str = "launcher-servers.xml";

impl Settings {
    /// The saved settings and servers, or the defaults where there are none.
    pub fn load() -> Self {
        Self::load_from(&ac2_prefs::paths::config_dir())
    }

    pub fn load_from(dir: &Path) -> Self {
        let mut settings: Settings = std::fs::read_to_string(dir.join(SETTINGS_FILE))
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default();
        if let Ok(text) = std::fs::read_to_string(dir.join(SERVERS_FILE)) {
            if let Ok((servers, _)) = parse_servers(&text, Source::User) {
                settings.favorites = servers;
            }
        }
        settings
    }

    pub fn save(&self) -> Result<(), String> {
        self.save_to(&ac2_prefs::paths::config_dir())
    }

    pub fn save_to(&self, dir: &Path) -> Result<(), String> {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        let text = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(dir.join(SETTINGS_FILE), text).map_err(|e| e.to_string())?;
        std::fs::write(dir.join(SERVERS_FILE), write_servers(&self.favorites))
            .map_err(|e| e.to_string())
    }

    /// Adds `server` to the player's servers, replacing one at the same connect string.
    pub fn add_favorite(&mut self, mut server: Server) {
        server.source = Source::User;
        let key = server.key();
        match self.favorites.iter_mut().find(|f| f.key() == key) {
            Some(slot) => *slot = server,
            None => self.favorites.push(server),
        }
    }

    pub fn is_favorite(&self, server: &Server) -> bool {
        self.favorites.iter().any(|f| f.key() == server.key())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_built_in_server_is_the_local_ac2e() {
        let servers = built_in();
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].problem(), None);
        assert_eq!(servers[0].address().unwrap(), ("127.0.0.1".into(), 9000));
    }

    /// A relaunch server is its connect string and its type.
    #[test]
    fn a_relaunch_server_is_its_connect_string() {
        let s = Server::new("IceFire", "icefire.portalspace.net:8610", Emu::Relaunch);
        assert_eq!(s.problem(), None);
        assert_eq!(
            s.address().unwrap(),
            ("icefire.portalspace.net".into(), 8610)
        );
        assert_eq!(s.emu.protocol(), "1802");
        assert!(Server::new("x", "nohost", Emu::Ac2e).problem().is_some());
        assert!(Server::new("x", "h:0", Emu::Ac2e).problem().is_some());
    }

    #[test]
    fn the_players_servers_and_settings_round_trip_without_a_password() {
        let dir = tempfile::tempdir().unwrap();
        let mut settings = Settings {
            dat_dir: "/dats".into(),
            ..Settings::default()
        };
        let mut mine = Server::new("Mine & <yours>", "ac2.example.net:9100", Emu::Ac2e);
        mine.login_server_url = "https://ac2.example.net".into();
        settings.add_favorite(mine);
        settings
            .accounts
            .insert("ac2.example.net:9100".into(), "me".into());
        settings.save_to(dir.path()).unwrap();
        let loaded = Settings::load_from(dir.path());
        assert_eq!(loaded, settings);
        for file in [SETTINGS_FILE, SERVERS_FILE] {
            let text = std::fs::read_to_string(dir.path().join(file)).unwrap();
            assert!(!text.to_ascii_lowercase().contains("password"), "{text}");
        }
        assert_eq!(
            Settings::load_from(&dir.path().join("none")).favorites,
            built_in()
        );
    }

    /// A published list in ThwargLauncher's `ServerItem` schema: `server_host` and
    /// `server_port`; other games' servers and broken entries are skipped and named.
    #[test]
    fn a_published_list_reads_its_ac2_servers() {
        let text = r#"<?xml version="1.0" encoding="utf-8"?>
            <ArrayOfServerItem>
              <ServerItem>
                <id>7f3b9c21-4e8a-4d16-b2f7-1a9e5c0d3b84</id><name>Ice</name>
                <description>Relaunch</description><emu>Relaunch</emu>
                <server_host>icefire.portalspace.net</server_host><server_port>8610</server_port>
                <type>PvE</type><status>Stable</status><website_url/><discord_url/>
              </ServerItem>
              <ServerItem>
                <name>Emu</name><emu>AC2E</emu>
                <connect_string>ac2.example.net:9000</connect_string>
                <LoginServerUrlKey>https://ac2.example.net</LoginServerUrlKey>
              </ServerItem>
              <ServerItem><name>Dereth</name><emu>ACE</emu>
                <server_host>ace.example</server_host><server_port>9000</server_port></ServerItem>
              <ServerItem><name>Broken</name><emu>AC2E</emu>
                <server_host></server_host><server_port>9000</server_port></ServerItem>
              <ServerItem><name>Again</name><emu>AC2E</emu>
                <connect_string>AC2.example.net:9000</connect_string></ServerItem>
            </ArrayOfServerItem>"#;
        let (servers, problems) = parse_servers(text, Source::Published).unwrap();
        let names: Vec<&str> = servers.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, ["Ice", "Emu"]);
        assert_eq!(servers[0].emu, Emu::Relaunch);
        assert_eq!(servers[0].status, "Stable");
        assert_eq!(servers[1].login_server_url, "https://ac2.example.net");
        assert_eq!(problems.len(), 3, "{problems:?}");
        assert!(parse_servers("<not xml", Source::Published).is_err());
    }
}
