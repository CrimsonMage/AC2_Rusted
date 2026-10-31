//! Test tooling: signs in at an AC2e account service with the launcher's own code and prints
//! `ACCOUNT TICKET HOST PORT` (used by `scripts/ac2e-launcher-test.sh`; the ticket is single-use).
//!
//! `cargo run -p ac2-launcher --example sign_in -- URL ACCOUNT` with the password on stdin.

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [url, account] = args.as_slice() else {
        eprintln!("usage: sign_in URL ACCOUNT  (password on stdin)");
        std::process::exit(2);
    };
    let mut password = String::new();
    std::io::stdin().read_line(&mut password).ok();
    match ac2_launcher::account::sign_in(url, account, password.trim_end()) {
        Ok(signed) => println!(
            "{} {} {} {}",
            signed.account,
            signed.ticket,
            signed.game_host.unwrap_or_default(),
            signed.game_port.unwrap_or_default()
        ),
        Err(e) => {
            eprintln!("sign-in failed: {e}");
            std::process::exit(1);
        }
    }
}
