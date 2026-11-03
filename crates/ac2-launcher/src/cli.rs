//! The launcher's command line, for shortcuts and scripts. It does what the window does, with
//! the same settings and code:
//!
//! ```text
//! ac2-launcher                                    the window
//! ac2-launcher add NAME HOST:PORT Pyrealium|Pyrealium|Relaunch [--login-server URL]
//! ac2-launcher set dat-dir DIR | client PATH | list ADDRESS
//! ac2-launcher list
//! ac2-launcher play NAME|HOST:PORT [--account NAME]
//! ```
//!
//! `play` builds the server's command line and starts the client, as Play does. A server that
//! wants a password reads it from the first line of standard input; it is never kept.

use std::time::Duration;

use ac2_launcher::launch::{self, Started};
use ac2_launcher::servers::{Emu, Server, Settings};

const USAGE: &str = "usage:
  ac2-launcher                                     open the launcher window
  ac2-launcher add NAME HOST:PORT Pyrealium|Pyrealium|Relaunch [--login-server URL]
  ac2-launcher set dat-dir DIR | client PATH | list ADDRESS
  ac2-launcher list
  ac2-launcher play NAME|HOST:PORT [--account NAME]   (a password on standard input)";

/// Runs a command; `None` when there is none (open the window).
pub fn run(args: &[String]) -> Option<Result<String, String>> {
    let (command, rest) = args.split_first()?;
    Some(match command.as_str() {
        "add" => add(rest),
        "set" => set(rest),
        "list" => Ok(list(&Settings::load())),
        "play" => play(rest),
        "help" | "--help" | "-h" => Ok(USAGE.into()),
        other => Err(format!("unknown command {other:?}\n{USAGE}")),
    })
}

/// `--name VALUE` among `args`, and the rest.
fn option(args: &[String], name: &str) -> Result<(Option<String>, Vec<String>), String> {
    let mut rest = Vec::new();
    let mut value = None;
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        if arg == name {
            value = Some(it.next().ok_or(format!("{name} needs a value"))?.clone());
        } else {
            rest.push(arg.clone());
        }
    }
    Ok((value, rest))
}

fn add(args: &[String]) -> Result<String, String> {
    let (login, rest) = option(args, "--login-server")?;
    let [name, connect, kind] = rest.as_slice() else {
        return Err(USAGE.into());
    };
    let emu = Emu::parse(kind).ok_or(format!("the type must be Pyrealium, AC2E or Relaunch, not {kind:?}"))?;
    let mut server = Server::new(name, connect, emu);
    server.login_server_url = login.unwrap_or_default();
    if let Some(problem) = server.problem() {
        return Err(problem);
    }
    let mut settings = Settings::load();
    let text = format!("added {} ({}, {})", server.name, server.connect, emu.name());
    settings.add_favorite(server);
    settings.save()?;
    Ok(text)
}

fn set(args: &[String]) -> Result<String, String> {
    let [key, value] = args else {
        return Err(USAGE.into());
    };
    let mut settings = Settings::load();
    match key.as_str() {
        "dat-dir" => settings.dat_dir = value.clone(),
        "client" => settings.client = value.clone(),
        "list" => settings.list_url = value.clone(),
        other => return Err(format!("unknown setting {other:?}\n{USAGE}")),
    }
    settings.save()?;
    Ok(format!("{key} = {value}"))
}

fn list(settings: &Settings) -> String {
    let mut out = format!(
        "DAT folder: {}\nclient: {}\n",
        if settings.dat_dir.is_empty() {
            "(not set)"
        } else {
            &settings.dat_dir
        },
        launch::client_program(settings).display()
    );
    for s in &settings.favorites {
        out.push_str(&format!("{}  {}  {}\n", s.name, s.connect, s.emu.name()));
    }
    out
}

fn play(args: &[String]) -> Result<String, String> {
    let (account, rest) = option(args, "--account")?;
    let [which] = rest.as_slice() else {
        return Err(USAGE.into());
    };
    let mut settings = Settings::load();
    let server = settings
        .favorites
        .iter()
        .find(|s| s.name.eq_ignore_ascii_case(which) || s.key() == which.to_ascii_lowercase())
        .cloned()
        .ok_or(format!("no server {which:?} (ac2-launcher list)"))?;
    let account = account.unwrap_or_else(|| {
        settings
            .accounts
            .get(&server.key())
            .cloned()
            .unwrap_or_default()
    });
    let mut password = String::new();
    if launch::needs_password(&server) {
        std::io::stdin()
            .read_line(&mut password)
            .map_err(|e| e.to_string())?;
    }
    let plan = launch::prepare(
        &server,
        &settings,
        &account,
        password.trim_end_matches(['\r', '\n']),
    )?;
    if !account.is_empty() {
        settings.accounts.insert(server.key(), account);
        settings.save()?;
    }
    eprintln!(
        "ac2-launcher: {} {}",
        plan.program.display(),
        plan.args.join(" ")
    );
    match launch::start(&plan, Duration::from_secs(60))? {
        Started::Running => Ok(format!("{} is starting in the client", server.name)),
        Started::Exited(why) => Err(format!("the client stopped ({why})")),
        Started::NoHandoff => Err("the client did not take its sign-in; it was removed".into()),
    }
}
