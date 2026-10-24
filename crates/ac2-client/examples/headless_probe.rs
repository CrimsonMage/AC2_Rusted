//! The headless client against a live server: log on, enter, walk forward with the DATs'
//! terrain and collision, print the frames, log off.
//!
//! `cargo run -p ac2-client --example headless_probe -- --dat-dir DIR [--server 127.0.0.1:9000]
//!   [--account local] [--version 7148] [--client-exe PATH] [--seconds 4] [--character NAME]
//!   [--walk 0.5]` (the fraction of the seconds spent walking forward)

use std::sync::Arc;
use std::time::{Duration, Instant};

use ac2_client::{Client, DatWorld, FrameInput};
use ac2_net::Credential;
use ac2_weenie::{
    ActiveAction, CharacterChoice, OnlineConfig, OnlineSession, ACTION_CLASS_MOVEMENT,
    ACTION_FORWARD,
};

fn main() -> Result<(), String> {
    let mut server = "127.0.0.1:9000".to_string();
    let mut account = "local".to_string();
    let mut version = "7148".to_string();
    let mut dat_dir = None;
    let mut server_key = None;
    let mut seconds = 4.0f64;
    let mut walk_fraction = 0.5f64;
    let mut character = CharacterChoice::First;
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        let value = args.next().ok_or(format!("missing value for {flag}"))?;
        match flag.as_str() {
            "--server" => server = value,
            "--account" => account = value,
            "--version" => version = value,
            "--dat-dir" => dat_dir = Some(value),
            "--seconds" => seconds = value.parse().map_err(|e| format!("{e}"))?,
            "--walk" => walk_fraction = value.parse().map_err(|e| format!("{e}"))?,
            "--character" => character = CharacterChoice::Name(value),
            "--client-exe" => {
                let exe = std::fs::read(&value).map_err(|e| format!("{value}: {e}"))?;
                server_key = ac2_net::ServerPublicKey::find_in_client(&exe);
            }
            other => return Err(format!("unknown option {other}")),
        }
    }
    let dat_dir = dat_dir.ok_or("--dat-dir is required")?;
    let dataset = ac2_dat::DatDataset::open(ac2_dat::DatOpenOptions::new(&dat_dir))
        .map_err(|e| e.to_string())?;
    let store = Arc::new(ac2_db::ResourceStore::new(Arc::clone(&dataset)));
    ac2_db::install_ac2_property_types(&store).map_err(|e| e.to_string())?;
    let world = DatWorld::new(Arc::clone(&store))?;
    let session = OnlineSession::connect(OnlineConfig {
        server: server.parse().map_err(|e| format!("{e}"))?,
        account,
        credential: Credential::Account,
        version,
        character,
        server_key,
        language: 0,
        dat_iterations: ac2_weenie::dat_iterations(&dataset)?,
    })
    .map_err(|e| e.to_string())?;
    let mut client = Client::new(session, Arc::new(world));
    let idle = FrameInput::default();
    let deadline = Instant::now() + Duration::from_secs(30);
    while !client.frame().in_world {
        if Instant::now() > deadline {
            return Err("no world entry".into());
        }
        client.use_time(&idle)?;
        std::thread::sleep(Duration::from_millis(16));
    }
    let frame = client.frame();
    println!(
        "entered: cell {:08X} at {:?}, {} blocks resident, time of day {:?}, notices {:?}",
        frame.cell_id.unwrap_or(0),
        frame.player.as_ref().map(|p| p.origin),
        frame.blocks.len(),
        frame.time_of_day,
        client.take_notices()
    );
    println!("camera: {:?}", frame.camera);
    let walk = FrameInput {
        actions: vec![ActiveAction {
            class: ACTION_CLASS_MOVEMENT,
            enum_value: ACTION_FORWARD,
            map_extent: 1.0,
            extent: 1.0,
        }],
        ..FrameInput::default()
    };
    let start = Instant::now();
    let mut next_report = 0.0;
    while start.elapsed().as_secs_f64() < seconds {
        if start.elapsed().as_secs_f64() >= next_report {
            next_report += 1.0;
            for o in client.frame().objects {
                println!(
                    "object {:#x} in_world {} active {} cell {:08X} at {:?} mode {:?} motion {:?} \
                     parts {} groups {} holder {:?}",
                    o.id,
                    o.in_world,
                    o.active,
                    o.cell,
                    o.origin,
                    o.mode,
                    o.motion,
                    o.pose.len(),
                    o.appearance.part_groups.len(),
                    o.holder
                );
            }
        }
        let input = if start.elapsed().as_secs_f64() < seconds * walk_fraction {
            &walk
        } else {
            &idle
        };
        for event in client.use_time(input)? {
            if !matches!(
                event,
                ac2_weenie::SessionEvent::Interp { .. }
                    | ac2_weenie::SessionEvent::ObjectMoved { .. }
            ) {
                println!("{event:?}");
            }
        }
        std::thread::sleep(Duration::from_millis(16));
    }
    let frame = client.frame();
    println!(
        "after the walk: cell {:08X} at {:?} (contact {:?}), {} remote objects, {} blocks",
        frame.cell_id.unwrap_or(0),
        frame.player.as_ref().map(|p| p.origin),
        frame.player.as_ref().map(|p| p.contact),
        frame.objects.len(),
        frame.blocks.len()
    );
    println!("camera: {:?}", frame.camera);
    let left = client.session_mut().log_off(Duration::from_secs(3));
    println!("character exit acknowledged: {left}");
    Ok(())
}
