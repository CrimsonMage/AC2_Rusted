//! The command line: our own options and the retail ones `Turbine::Client::Init` registers.
//!
//! Retail's `ArgumentParser::ParseArgs` (Legions 0x0060AFF0) calls a token an argument when it
//! starts with `-` or `/` (`ArgumentParser::IsCommandLineArgument`, 0x0060A2C0). A second
//! identical prefix character (`--host`) starts a long name; otherwise a one-character name is
//! the short form (`-h`) and anything longer is a long name too (`-host`, `/host`).

use std::net::SocketAddr;

pub(crate) struct Args {
    pub dat_dir: String,
    pub server: String,
    /// `Turbine::Client::EvaluateCommandLineArg` `account` (Legions 0x006A1EB0): stored in
    /// lower case (`_strlwr`).
    pub account: String,
    pub version: String,
    /// `gmClient::m_startChar`: enter with this character, skipping character selection.
    pub character: Option<String>,
    pub radius: i32,
    /// `Preferences.ini` (`-prefs` in retail): the default is in the user's configuration directory.
    pub prefs: Option<String>,
    /// `chargen <file>` (`CharGenData::SetFileName`): every created character is also captured
    /// there (a relative name goes to the configuration directory).
    pub chargen: Option<String>,
    /// `host` (`h`): the server's IPv4 address or DNS name.
    pub host: Option<String>,
    /// `port` (`p`, `<1-65535>`): the server port, [`ac2_net::DEFAULT_PORT`] without it.
    pub port: Option<String>,
    /// `glsticketdirect` (`z`): the ticket text stored straight into `Client+0x3c`. Empty means
    /// no password (`gmClient::InitAuth`, Legions 0x00404BB0). It is the password: never print it.
    pub ticket: String,
}

impl Args {
    /// The server to log on to: `--host` (resolved the way `ClientNet::InitAddresses` does) and
    /// `--port` when either is given, else `--server`.
    pub fn server_address(&self) -> Result<SocketAddr, String> {
        let port = match &self.port {
            Some(text) => Some(ac2_net::parse_port(text).map_err(|e| format!("--port: {e}"))?),
            None => None,
        };
        match &self.host {
            Some(host) => {
                let port = port.unwrap_or(ac2_net::DEFAULT_PORT);
                let address =
                    ac2_net::resolve_server(host, port).map_err(|e| format!("--host: {e}"))?;
                Ok(SocketAddr::V4(address))
            }
            None => {
                let mut server: SocketAddr =
                    self.server.parse().map_err(|e| format!("--server: {e}"))?;
                if let Some(port) = port {
                    server.set_port(port);
                }
                Ok(server)
            }
        }
    }
}

/// The name behind a retail-style argument token (`--name`, `-name`, `/name`, `-h`), with the
/// short forms of the arguments we accept expanded; `None` for a token that is not an argument.
fn argument_name(token: &str) -> Option<String> {
    let mut chars = token.chars();
    let prefix = chars.next().filter(|c| matches!(c, '-' | '/'))?;
    let rest = chars.as_str();
    let name = rest.strip_prefix(prefix).unwrap_or(rest);
    if !rest.starts_with(prefix) && name.chars().count() == 1 {
        return Some(
            match name {
                "h" => "host",
                "p" => "port",
                "z" => "glsticketdirect",
                other => other,
            }
            .to_owned(),
        );
    }
    Some(name.to_owned())
}

pub(crate) fn parse_args() -> Result<Args, String> {
    parse_from(
        std::env::args().skip(1),
        std::env::var("AC2_DAT_DIR").unwrap_or_default(),
    )
}

fn parse_from(tokens: impl IntoIterator<Item = String>, dat_dir: String) -> Result<Args, String> {
    let mut args = Args {
        dat_dir,
        server: "127.0.0.1:9000".into(),
        account: "local".into(),
        version: ac2_net::LEGIONS_VERSION.into(),
        character: None,
        radius: 1,
        prefs: None,
        chargen: None,
        host: None,
        port: None,
        ticket: String::new(),
    };
    let mut it = tokens.into_iter().peekable();
    while let Some(token) = it.next() {
        let name = argument_name(&token).ok_or_else(|| format!("unrecognized argument {token}"))?;
        if name == "rodat" {
            // `rodat` (Legions 0x006A1EB0 sets `Client+0x30`): open the DATs read-only. Our DAT
            // reader never writes, so the value is accepted and has no effect. The value is
            // optional: a following argument is not taken as one.
            if it.peek().is_some_and(|next| argument_name(next).is_none()) {
                it.next();
            }
            continue;
        }
        let known = [
            "dat-dir",
            "server",
            "account",
            "version",
            "character",
            "prefs",
            "chargen",
            "radius",
            "host",
            "port",
            "glsticketdirect",
        ];
        if !known.contains(&name.as_str()) {
            return Err(format!("unknown option {token}"));
        }
        let value = it.next().ok_or(format!("{token} needs a value"))?;
        match name.as_str() {
            "dat-dir" => args.dat_dir = value,
            "server" => args.server = value,
            "account" => args.account = value.to_ascii_lowercase(),
            "version" => args.version = value,
            "character" => args.character = Some(value),
            "prefs" => args.prefs = Some(value),
            "chargen" => args.chargen = Some(value),
            "radius" => args.radius = value.parse().map_err(|e| format!("--radius: {e}"))?,
            "host" => args.host = Some(value),
            "port" => args.port = Some(value),
            _ => args.ticket = value,
        }
    }
    if args.dat_dir.is_empty() {
        return Err("set AC2_DAT_DIR or pass --dat-dir".into());
    }
    Ok(args)
}

#[cfg(test)]
mod tests {
    use super::parse_from;
    use ac2_net::Credential;

    fn parse(line: &str) -> Result<super::Args, String> {
        parse_from(line.split_whitespace().map(String::from), "/dat".into())
    }

    #[test]
    fn the_retail_command_line_is_accepted() {
        let args = parse(
            "--host 10.1.2.3 --port 8610 --rodat on --account MyName --glsticketdirect Secret",
        )
        .unwrap();
        assert_eq!(args.account, "myname");
        assert_eq!(args.server_address().unwrap().to_string(), "10.1.2.3:8610");
        assert_eq!(
            Credential::from_ticket(&args.ticket),
            Credential::Password("Secret".into())
        );
    }

    #[test]
    fn retail_prefixes_and_short_names() {
        let args = parse("-host 10.1.2.3 /port 0x2182 -z pw --dat-dir /x").unwrap();
        assert_eq!(args.server_address().unwrap().to_string(), "10.1.2.3:8578");
        assert_eq!((args.ticket.as_str(), args.dat_dir.as_str()), ("pw", "/x"));
        let args = parse("-h 10.1.2.3 -p 99").unwrap();
        assert_eq!(args.server_address().unwrap().to_string(), "10.1.2.3:99");
    }

    #[test]
    fn host_defaults_to_the_retail_port_and_beats_server() {
        let args = parse("--server 127.0.0.1:9000 --host 10.1.2.3").unwrap();
        assert_eq!(args.server_address().unwrap().to_string(), "10.1.2.3:7304");
    }

    #[test]
    fn our_options_still_work() {
        let args = parse("--server 127.0.0.1:9000 --account local --version 7148").unwrap();
        assert_eq!(args.server_address().unwrap().to_string(), "127.0.0.1:9000");
        assert_eq!(args.version, "7148");
        assert_eq!(Credential::from_ticket(&args.ticket), Credential::Account);
        let args = parse("--server 127.0.0.1:9000 --port 9100").unwrap();
        assert_eq!(args.server_address().unwrap().to_string(), "127.0.0.1:9100");
    }

    #[test]
    fn rodat_value_is_optional_and_ignored() {
        assert!(parse("--rodat --account A").is_ok());
        assert!(parse("--rodat").is_ok());
        assert_eq!(parse("--rodat on --account A").unwrap().account, "a");
    }

    #[test]
    fn bad_input_is_an_error_that_never_shows_the_ticket() {
        assert!(parse("--bogus 1").is_err());
        assert!(parse("--host").is_err());
        assert!(parse("--port 70000 --host 1.2.3.4")
            .unwrap()
            .server_address()
            .is_err());
        let error = parse("--glsticketdirect hunter2 --bogus 1").err().unwrap();
        assert!(!error.contains("hunter2"), "{error}");
    }
}
