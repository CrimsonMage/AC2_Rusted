//! Headless online check: log on, enter the world, report a short walk, and print what the
//! server sends.
//!
//! `cargo run -p ac2-weenie --example online_probe -- --server 127.0.0.1:9000 \
//!   --account local [--password PW] [--character NAME] [--seconds 10] [--walk 2.0] \
//!   [--version 7160] [--client-exe PATH] [--dat-dir PATH] [--language N]`
//!
//! The Legions DVD client logs on as `7160`; servers that only accept the admin build need
//! `--version 7148` (the default here, for the local test server). `--client-exe` reads the RSA
//! key a retail `ac2client.exe` carries and logs on with an exported session key, as retail
//! does; without it the logon carries no session key. `--dat-dir` answers the DAT interrogation
//! with those DATs' iterations, as retail does; without it the answer lists none.

use std::time::{Duration, Instant};

use ac2_weenie::{
    BodyState, CellPosition, CharacterChoice, Credential, OnlineConfig, OnlineSession, SessionEvent,
};

fn main() -> Result<(), String> {
    let mut server = "127.0.0.1:9000".to_string();
    let mut account = "local".to_string();
    let mut credential = Credential::Account;
    let mut character = CharacterChoice::First;
    let mut seconds = 10.0f32;
    let mut walk_speed = 2.0f32;
    let mut version = "7148".to_string();
    let mut server_key = None;
    let mut language = 0;
    let mut dat_iterations = Vec::new();
    let mut property_types = false;
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
            "--version" => version = value,
            "--language" => language = value.parse().map_err(|e| format!("{e}"))?,
            "--dat-dir" => {
                let dataset = ac2_dat::DatDataset::open(ac2_dat::DatOpenOptions::new(&value))
                    .map_err(|e| format!("{value}: {e}"))?;
                dat_iterations = ac2_weenie::dat_iterations(&dataset)?;
                let store = ac2_db::ResourceStore::new(dataset);
                property_types =
                    ac2_db::install_ac2_property_types(&store).map_err(|e| e.to_string())?;
                println!(
                    "interrogation answer: {:?}",
                    dat_iterations
                        .iter()
                        .map(|(id, its)| (format!("{id:#018X}"), its.len()))
                        .collect::<Vec<_>>()
                );
            }
            "--client-exe" => {
                let exe = std::fs::read(&value).map_err(|e| format!("{value}: {e}"))?;
                let key = ac2_net::ServerPublicKey::find_in_client(&exe)
                    .ok_or(format!("{value}: no 0x54-byte PUBLICKEYBLOB"))?;
                println!("session key exported under a {}-bit RSA key", key.bits());
                server_key = Some(key);
            }
            other => return Err(format!("unknown option {other}")),
        }
    }
    let mut session = OnlineSession::connect(OnlineConfig {
        server: server.parse().map_err(|e| format!("{e}"))?,
        account,
        credential,
        version,
        character,
        server_key,
        language,
        dat_iterations,
    })
    .map_err(|e| e.to_string())?;
    if property_types {
        session.set_property_types(Box::new(ac2_db::ac2_property_type));
    }
    let entry = session.wait_for_world(Duration::from_secs(45))?;
    println!(
        "entered world: player {:#x} instance {} cell {:08X} at {:?} visual {:?}",
        entry.player, entry.instance, entry.position.cell, entry.position.origin, entry.visual
    );
    session.scene_ready();
    let start = Instant::now();
    let origin = entry.position.origin;
    let mut reports = 0;
    while start.elapsed().as_secs_f32() < seconds {
        let t = start.elapsed().as_secs_f32();
        // Walk north for the first half, then stand.
        let walking = t < seconds / 2.0;
        let distance = walk_speed * t.min(seconds / 2.0);
        session.report_body(&BodyState {
            position: CellPosition {
                cell: entry.position.cell,
                origin: [origin[0], origin[1] + distance, origin[2]],
            },
            heading_degrees: 0.0,
            turn_to_heading: None,
            motion: if walking { [0.0, 1.0, 0.0] } else { [0.0; 3] },
            times_jumped: 0,
            velocity: [0.0; 3],
            actively_moving: walking,
        });
        reports += 1;
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
                SessionEvent::PlayerQualities(q) => println!(
                    "player qualities {:#010x}: {} ints, {} floats, {} strings, {} DIDs, weenie {}",
                    q.did,
                    q.ints.len(),
                    q.floats.len(),
                    q.strings.len(),
                    q.dids.len(),
                    q.weenie.is_some()
                ),
                SessionEvent::Characters(set) => println!(
                    "characters: {:?}",
                    set.characters.iter().map(|c| &c.name).collect::<Vec<_>>()
                ),
                other => println!("{other:?}"),
            }
        }
        std::thread::sleep(Duration::from_millis(16));
    }
    println!(
        "{} objects live after {reports} frames; logging off",
        session.objects().len()
    );
    let left = session.log_off(Duration::from_secs(3));
    println!("character exit acknowledged: {left}");
    Ok(())
}
