//! Starting the client for a server, as ThwargLauncher does: the launcher builds the command
//! line for the server's type (`emu`) from its connect string and the player's account and
//! password, and starts the client with it; the player never writes it. The command lines
//! (`docs/server-list.md`):
//!
//! - AC2E: `--host HOST --port PORT --version 7230 --account ACCOUNT`, with `--glsticket FILE`
//!   when the server has a login service (its ticket).
//! - Relaunch: `--host HOST --port PORT --version 1802 --account ACCOUNT --glsticket FILE`
//!   (the password, as `glsticketdirect`).
//!
//! The ticket is a file only the player can read (`glsticket FILE`), which the client reads and
//! deletes, as retail's launcher waited for its registry value to go.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use crate::account;
use crate::servers::{Emu, Server, Settings};

/// A prepared start: the program, its arguments and the ticket file waiting for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Launch {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub ticket_file: Option<PathBuf>,
}

/// The `ac2` program: the settings' choice, else the one beside the launcher.
pub fn client_program(settings: &Settings) -> PathBuf {
    if !settings.client.trim().is_empty() {
        return PathBuf::from(settings.client.trim());
    }
    let name = if cfg!(windows) { "ac2.exe" } else { "ac2" };
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join(name)))
        .unwrap_or_else(|| PathBuf::from(name))
}

/// Whether the server wants a password: a relaunch server always, an AC2E server with a login
/// service.
pub fn needs_password(server: &Server) -> bool {
    match server.emu {
        Emu::Relaunch => true,
        Emu::Ac2e => !server.login_server_url.trim().is_empty(),
    }
}

/// The account to sign in with: what the player typed, else an AC2E server's developer
/// account `local` when it has no login service.
pub fn account_for(server: &Server, typed: &str) -> Option<String> {
    let typed = typed.trim();
    if !typed.is_empty() {
        return Some(typed.to_owned());
    }
    (!needs_password(server)).then(|| "local".to_owned())
}

/// The command line for `server` (see the module docs).
pub fn args_for(
    server: &Server,
    dat_dir: &str,
    account: &str,
    address: (&str, u16),
    ticket_file: Option<&Path>,
) -> Vec<String> {
    let mut args = vec![
        "--dat-dir".to_owned(),
        dat_dir.to_owned(),
        "--host".to_owned(),
        address.0.to_owned(),
        "--port".to_owned(),
        address.1.to_string(),
        "--version".to_owned(),
        server.emu.protocol().to_owned(),
        "--account".to_owned(),
        account.to_owned(),
    ];
    if let Some(file) = ticket_file {
        args.push("--glsticket".into());
        args.push(file.display().to_string());
    }
    args
}

/// Signs in where the server wants it and prepares the start. `password` is used for this
/// start only.
pub fn prepare(
    server: &Server,
    settings: &Settings,
    typed_account: &str,
    password: &str,
) -> Result<Launch, String> {
    if let Some(problem) = server.problem() {
        return Err(problem);
    }
    let dat_dir = settings.dat_dir.trim();
    if dat_dir.is_empty() {
        return Err("choose the DAT folder first".into());
    }
    let account = account_for(server, typed_account).ok_or("enter your account name")?;
    if needs_password(server) && password.is_empty() {
        return Err("enter your password".into());
    }
    let mut address = server.address()?;
    let login = server.login_server_url.trim();
    let ticket = match server.emu {
        Emu::Relaunch => Some(password.to_owned()),
        Emu::Ac2e if login.is_empty() => None,
        Emu::Ac2e => {
            let signed = account::sign_in(login, &account, password)?;
            if let Some(host) = signed.game_host.clone().filter(|h| !h.is_empty()) {
                address.0 = host;
            }
            if let Some(port) = signed.game_port.filter(|p| *p != 0) {
                address.1 = port;
            }
            Some(signed.ticket)
        }
    };
    let ticket_file = match ticket {
        Some(ticket) => Some(write_ticket(&ticket)?),
        None => None,
    };
    let args = args_for(
        server,
        dat_dir,
        &account,
        (&address.0, address.1),
        ticket_file.as_deref(),
    );
    Ok(Launch {
        program: client_program(settings),
        args,
        ticket_file,
    })
}

/// Writes `ticket` to a new file only the player can read, in the AC2 configuration
/// directory's `launch` folder.
pub fn write_ticket(ticket: &str) -> Result<PathBuf, String> {
    write_ticket_in(&ac2_prefs::paths::config_dir().join("launch"), ticket)
}

pub fn write_ticket_in(dir: &Path, ticket: &str) -> Result<PathBuf, String> {
    use std::io::Write;
    let io = |e: std::io::Error| format!("the ticket file could not be written: {e}");
    std::fs::create_dir_all(dir).map_err(io)?;
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
        std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700)).map_err(io)?;
        options.mode(0o600);
    }
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let path = dir.join(format!("GLSTicket-{}-{stamp}", std::process::id()));
    let mut file = options.open(&path).map_err(io)?;
    file.write_all(ticket.as_bytes()).map_err(io)?;
    Ok(path)
}

/// How a start ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Started {
    /// The client is running (and took its ticket, if it had one).
    Running,
    /// The client stopped before taking its ticket; its exit status.
    Exited(String),
    /// The client did not take its ticket in time; the ticket was removed.
    NoHandoff,
}

/// Starts the client and waits up to `wait` for it to take its ticket. A ticket left behind is
/// removed, so it never stays on disk.
pub fn start(launch: &Launch, wait: Duration) -> Result<Started, String> {
    let mut child = std::process::Command::new(&launch.program)
        .args(&launch.args)
        .spawn()
        .map_err(|e| {
            if let Some(file) = &launch.ticket_file {
                let _ = std::fs::remove_file(file);
            }
            format!("{} could not be started: {e}", launch.program.display())
        })?;
    let Some(file) = &launch.ticket_file else {
        return Ok(Started::Running);
    };
    let deadline = Instant::now() + wait;
    loop {
        if !file.exists() {
            return Ok(Started::Running);
        }
        if let Ok(Some(status)) = child.try_wait() {
            let _ = std::fs::remove_file(file);
            return Ok(Started::Exited(status.to_string()));
        }
        if Instant::now() >= deadline {
            let _ = std::fs::remove_file(file);
            return Ok(Started::NoHandoff);
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::servers::built_in;

    fn local() -> Server {
        built_in().remove(0)
    }

    fn relaunch() -> Server {
        Server::new("IceFire", "icefire.portalspace.net:8610", Emu::Relaunch)
    }

    #[test]
    fn an_ac2e_server_without_a_login_service_uses_the_developer_account() {
        let s = local();
        let account = account_for(&s, "").unwrap();
        let args = args_for(&s, "/dats", &account, ("127.0.0.1", 9000), None);
        assert_eq!(
            args,
            [
                "--dat-dir",
                "/dats",
                "--host",
                "127.0.0.1",
                "--port",
                "9000",
                "--version",
                "7230",
                "--account",
                "local"
            ]
        );
    }

    #[test]
    fn a_relaunch_server_takes_the_password_ticket_and_its_protocol() {
        let s = relaunch();
        assert!(needs_password(&s));
        assert_eq!(account_for(&s, ""), None, "an account must be typed");
        let file = Path::new("/cfg/launch/GLSTicket-1");
        let args = args_for(
            &s,
            "/dats",
            "player",
            ("icefire.portalspace.net", 8610),
            Some(file),
        );
        assert_eq!(
            args,
            [
                "--dat-dir",
                "/dats",
                "--host",
                "icefire.portalspace.net",
                "--port",
                "8610",
                "--version",
                "1802",
                "--account",
                "player",
                "--glsticket",
                "/cfg/launch/GLSTicket-1"
            ]
        );
    }

    #[test]
    fn a_ticket_file_is_new_and_only_the_player_can_read_it() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_ticket_in(&dir.path().join("launch"), "t0k3n").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "t0k3n");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o600);
        }
        // The client's own reader takes it and removes it.
        assert_eq!(
            ac2_prefs::gls_ticket::take_from_file(&path).as_deref(),
            Ok("t0k3n")
        );
        assert!(!path.exists());
    }

    #[test]
    fn a_missing_dat_folder_or_password_stops_the_start_first() {
        let settings = Settings {
            dat_dir: String::new(),
            ..Settings::default()
        };
        assert!(prepare(&local(), &settings, "", "").is_err());
        let settings = Settings {
            dat_dir: "/dats".into(),
            ..Settings::default()
        };
        let error = prepare(&relaunch(), &settings, "player", "").unwrap_err();
        assert!(error.contains("password"), "{error}");
    }
}
