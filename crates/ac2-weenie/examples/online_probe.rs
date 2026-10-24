//! Headless online check: log on, enter the world, report a short walk, and print what the
//! server sends.
//!
//! `cargo run -p ac2-session --example online_probe -- --server 127.0.0.1:9000 \
//!   --account local [--password PW] [--character NAME] [--seconds 10] [--walk 2.0]`

use std::time::{Duration, Instant};

use ac2_weenie::{
    CellPosition, CharacterChoice, Credential, OnlineConfig, OnlineSession, SessionEvent,
};

fn main() -> Result<(), String> {
    let mut server = "127.0.0.1:9000".to_string();
    let mut account = "local".to_string();
    let mut credential = Credential::Account;
    let mut character = CharacterChoice::First;
    let mut seconds = 10.0f32;
    let mut walk_speed = 2.0f32;
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        let value = args.next().ok_or(format!("missing value for {flag}"))?;
        match flag.as_str() {
            "--server" => server = value,
            "--account" => account = value,
            "--password" => credential = Credential::Password(value),
            "--character" => character = CharacterChoice::Name(value),
            "--seconds" => seconds = value.parse().map_err(|e| format!("{e}"))?,
            "--walk" => walk_speed = value.parse().map_err(|e| format!("{e}"))?,
            other => return Err(format!("unknown option {other}")),
        }
    }
    let mut session = OnlineSession::connect(OnlineConfig {
        server: server.parse().map_err(|e| format!("{e}"))?,
        account,
        credential,
        version: "7148".into(),
        character,
        connect_timeout: Duration::from_secs(10),
    })
    .map_err(|e| e.to_string())?;
    let entry = session.wait_for_world(Duration::from_secs(15))?;
    println!(
        "entered world: player {:#x} instance {} cell {:08X} at {:?} visual {:?}",
        entry.player, entry.instance, entry.position.cell, entry.position.origin, entry.visual
    );
    session.scene_ready();
    let start = Instant::now();
    let origin = entry.position.origin;
    while start.elapsed().as_secs_f32() < seconds {
        let t = start.elapsed().as_secs_f32();
        // Walk north for the first half, then stand.
        let walking = t < seconds / 2.0;
        let distance = walk_speed * t.min(seconds / 2.0);
        session.report_position(
            CellPosition {
                cell: entry.position.cell,
                origin: [origin[0], origin[1] + distance, origin[2]],
            },
            0.0,
            if walking { [0.0, 1.0, 0.0] } else { [0.0; 3] },
            None,
        );
        for event in session.poll() {
            match event {
                SessionEvent::ObjectAdded(id) => {
                    let object = session.objects().get(id).cloned();
                    println!("object added {id:#x}: {object:?}");
                }
                SessionEvent::ObjectMoved(id) => {
                    let object = session.objects().get(id);
                    println!(
                        "object moved {id:#x}: {:?}",
                        object.and_then(|o| o.position)
                    );
                }
                other => println!("{other:?}"),
            }
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    println!("{} objects live; logging off", session.objects().len());
    session.log_off();
    Ok(())
}
