//! The pregame screens' music on the real layouts (`AC2_DAT_DIR`), headless: the intro opens
//! playing its `music`, and the character selection that follows plays its own while the intro's
//! fades out (`gmIntroUI` and `gmCharacterSelectionUI` constructors, `Channel_Main`). The sound
//! manager's commands go to a tee that records them and mixes them into a [`MixingSink`].

mod common;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use ac2_audio::{MixingSink, RecordingSink, SinkCommand, SoundSink};
use ac2_client::{pregame_event, pregame_requests, Client};
use ac2_ui::pregame::{PregameUi, MODE_CHARACTER_SELECTION};
use ac2_weenie::{CharacterChoice, OnlineSession};
use common::*;
use legions_server_double::{double, Spawn};

type Server = Rc<RefCell<legions_server_double::State>>;

/// Forwards every command to a mixer and keeps a copy of each one.
#[derive(Clone)]
struct TeeSink {
    recorded: RecordingSink,
    mixer: MixingSink,
}

impl SoundSink for TeeSink {
    fn send(&mut self, command: SinkCommand) {
        self.recorded.send(command.clone());
        self.mixer.send(command);
    }
}

fn connect() -> Option<(Client, Server, PregameUi, RecordingSink, MixingSink)> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = Arc::new(ac2_db::ResourceStore::new(dataset));
    let assets = ac2_ui::pregame::load_pregame(&store).ok()?;
    let ui = PregameUi::new(assets, 1024, 768);
    let (transport, server) = double(Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    });
    let mut config = config();
    config.character = CharacterChoice::Wait;
    let session = OnlineSession::with_transport(config, Box::new(transport)).ok()?;
    let mut client = Client::new(session, Arc::new(Synthetic));
    client.set_load_budget(Duration::from_secs(5));
    let recorded = RecordingSink::new();
    let mixer = MixingSink::new(44_100);
    let tee = TeeSink {
        recorded: recorded.clone(),
        mixer: mixer.clone(),
    };
    client.attach_audio(ac2_client::sound_manager(store, Box::new(tee)));
    Some((client, server, ui, recorded, mixer))
}

/// The intro's `MusicInfo` (`gmIntroUI`'s `IntroField`) and the character selection's (its
/// `BackgroundField`), on the Legions layouts.
const INTRO_MUSIC: u32 = 0x2C00_039B;
const SELECTION_MUSIC: u32 = 0x2C00_039A;

/// One frame of the pregame loop, as the window program runs it: the session's events into the
/// UI, the UI's loop, its requests out (the music played on the client). Returns the music the
/// frame asked for, each with the handle it now plays on (`None` when it did not start).
fn frame(
    client: &mut Client,
    server: &Server,
    ui: &mut PregameUi,
    t: f64,
) -> Vec<(u32, Option<u32>)> {
    let events = step(client, server, &ac2_client::FrameInput::default());
    for event in &events {
        pregame_event(ui, event);
    }
    ui.use_time(t);
    let outcome = pregame_requests(ui, client.session_mut());
    assert!(!outcome.quit);
    let mut played = Vec::new();
    for &did in &outcome.music {
        client.play_pregame_music(did);
        // The music loads on the sound loader; the test takes it as it finishes.
        client
            .audio_mut()
            .unwrap()
            .wait_for_loads(Duration::from_secs(30));
        played.push((did, client.pregame_music_handle()));
    }
    played
}

/// Whether `voice` was stopped or faded out after the command `from`.
fn faded_after(recorded: &RecordingSink, from: usize, voice: u32) -> bool {
    recorded.commands().iter().skip(from).any(|c| match c {
        SinkCommand::Volume { voice: v, .. }
        | SinkCommand::Stop { voice: v }
        | SinkCommand::StopMusic { voice: v, .. } => *v == voice,
        _ => false,
    })
}

#[test]
fn the_intro_and_then_the_character_selection_play_their_music() {
    let Some((mut client, server, mut ui, recorded, mixer)) = connect() else {
        return;
    };
    let mut played = Vec::new();
    let mut t = 0.0;
    // The commands before the selection's music starts (its frame's own are after this mark).
    let mut before_selection = None;
    while ui.mode() != MODE_CHARACTER_SELECTION {
        let mark = recorded.commands().len();
        let now = frame(&mut client, &server, &mut ui, t);
        if now.iter().any(|&(did, _)| did == SELECTION_MUSIC) {
            before_selection = Some(mark);
        }
        played.extend(now);
        t += STEP;
        assert!(t < 30.0, "no character selection");
    }
    eprintln!("screen music {played:08X?}");
    let dids: Vec<u32> = played.iter().map(|&(did, _)| did).collect();
    assert_eq!(
        dids,
        vec![INTRO_MUSIC, SELECTION_MUSIC],
        "the intro, then the selection"
    );
    let intro = played[0].1.expect("the intro's music plays");
    let selection = played[1].1.expect("the selection's music plays");
    assert_ne!(selection, intro, "the selection plays its own music");

    // The intro's music is heard while it plays: its wave or segment reaches the mix.
    let mut peak = 0.0f32;
    for _ in 0..64 {
        frame(&mut client, &server, &mut ui, t);
        t += STEP;
        let samples = mixer.render(689, 2);
        assert!(samples.iter().all(|s| s.is_finite()));
        peak = peak.max(samples.iter().fold(0.0f32, |p, s| p.max(s.abs())));
    }
    assert!(peak > 0.0001, "the music is heard (peak {peak})");

    // The selection's music replaces the intro's: the intro fades out over two seconds.
    for _ in 0..(3 * 64) {
        frame(&mut client, &server, &mut ui, t);
        t += STEP;
        mixer.render(689, 2);
    }
    let before_selection = before_selection.expect("the selection's frame");
    assert!(
        faded_after(&recorded, before_selection, intro),
        "the intro's music fades out when the selection replaces it"
    );
}
