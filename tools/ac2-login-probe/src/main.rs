//! `ac2-login-probe`: the network logon alone, against a real or local AC2 server.

use std::process::ExitCode;
use std::time::Duration;

use ac2_login_probe::{resolve_ipv4, retail_account, run_udp, Failure, FailureKind, ProbeConfig};
use ac2_net::{Credential, LEGIONS_VERSION};

const HELP: &str = "\
ac2-login-probe: log on to an AC2 server, report each stage, quit at character selection.

USAGE
  ac2-login-probe --host NAME|IP --port N --account NAME [options]

OPTIONS
  --host NAME|IP          server host (resolved to its first IPv4 address)
  --port N                server logon port (retail icefire: 8610)
  --account NAME          account name (lowercased, as retail does)
  --glsticketdirect PASS  password logon (auth type 0x40000002); omit for a plain account logon
  --version V             data version in the logon (default 7160, the Legions DVD client;
                          1802 for the 2012-2017 relaunch client). Pyrealium accepts only
                          7148 or 7230. The RSA session key is never sent.
  --dat-dir DIR           DAT directory for the interrogation response (default AC2_DAT_DIR)
  --timeout SECS          wait for the character set this long (default 45)
  -h, --help              this text

EXIT CODES
  0 character set reached and logoff sent   1 usage or setup   2 host did not resolve
  3 no reply / timeout   4 refused (server gave a reason or closed)   5 protocol error
";

struct Args {
    host: String,
    port: u16,
    account: String,
    password: Option<String>,
    version: String,
    dat_dir: Option<String>,
    timeout: u64,
}

fn parse(mut args: impl Iterator<Item = String>) -> Result<Option<Args>, String> {
    let (mut host, mut port, mut account) = (None, None, None);
    let mut parsed = Args {
        host: String::new(),
        port: 0,
        account: String::new(),
        password: None,
        version: LEGIONS_VERSION.into(),
        dat_dir: std::env::var("AC2_DAT_DIR").ok().filter(|d| !d.is_empty()),
        timeout: 45,
    };
    while let Some(flag) = args.next() {
        if flag == "-h" || flag == "--help" {
            return Ok(None);
        }
        let mut value = || args.next().ok_or(format!("{flag} needs a value"));
        match flag.as_str() {
            "--host" => host = Some(value()?),
            "--port" => {
                port = Some(
                    value()?
                        .parse::<u16>()
                        .map_err(|e| format!("--port: {e}"))?,
                )
            }
            "--account" => account = Some(value()?),
            "--glsticketdirect" => parsed.password = Some(value()?).filter(|p| !p.is_empty()),
            "--version" => parsed.version = value()?,
            "--dat-dir" => parsed.dat_dir = Some(value()?),
            "--timeout" => {
                parsed.timeout = value()?.parse().map_err(|e| format!("--timeout: {e}"))?
            }
            other => return Err(format!("unknown option {other}")),
        }
    }
    parsed.host = host.ok_or("--host is required")?;
    parsed.port = port.ok_or("--port is required")?;
    parsed.account = account.ok_or("--account is required")?;
    Ok(Some(parsed))
}

fn setup(args: &Args) -> Result<ProbeConfig, Failure> {
    let setup = |m: String| Failure::new(FailureKind::Setup, m);
    let dat_iterations = match &args.dat_dir {
        Some(dir) => {
            let dataset = ac2_dat::DatDataset::open(ac2_dat::DatOpenOptions::new(dir))
                .map_err(|e| setup(format!("--dat-dir: {e}")))?;
            ac2_weenie::dat_iterations(&dataset).map_err(setup)?
        }
        None => Vec::new(),
    };
    let server =
        resolve_ipv4(&args.host, args.port).map_err(|e| Failure::new(FailureKind::Resolve, e))?;
    println!("resolved {} -> {server}", args.host);
    Ok(ProbeConfig {
        server,
        account: retail_account(&args.account),
        credential: match &args.password {
            Some(password) => Credential::Password(password.clone()),
            None => Credential::Account,
        },
        version: args.version.clone(),
        dat_iterations,
        timeout: Duration::from_secs(args.timeout),
    })
}

fn main() -> ExitCode {
    let args = match parse(std::env::args().skip(1)) {
        Ok(Some(args)) => args,
        Ok(None) => {
            print!("{HELP}");
            return ExitCode::SUCCESS;
        }
        Err(message) => {
            eprintln!("error: {message}\n\n{HELP}");
            return ExitCode::from(FailureKind::Setup.exit_code());
        }
    };
    let result = setup(&args).and_then(|config| {
        if config.dat_iterations.is_empty() {
            println!("no DATs: the interrogation response will list no files");
        }
        run_udp(config, &mut |stage| println!("{stage}"))
    });
    match result {
        Ok(set) => {
            println!("ok: {} character(s); disconnected", set.characters.len());
            ExitCode::SUCCESS
        }
        Err(failure) => {
            eprintln!("failed ({}): {}", failure.kind.label(), failure.detail);
            ExitCode::from(failure.kind.exit_code())
        }
    }
}
