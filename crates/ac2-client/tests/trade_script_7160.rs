//! The secure trade with the player's script on the VM: the trade window's commands go out
//! through `gmCWInterface::AcceptTrade`'s IFun and the stubs, and the double's `WM_Trade`
//! forwarders run on `CPlayer`, which raises the `CM_Trade` notices (`AC2_DAT_DIR`).

mod common;

use std::sync::Arc;

use ac2_client::{Command, FrameInput};
use ac2_core::DataId;
use ac2_net::protocol::{
    pack_values, unpack_values, HashKey, NativeObject, PackValue, PackedEntry, ScriptRequest,
};
use ac2_script::data::DatScriptData;
use ac2_script::host::ScriptHost;
use ac2_script::vm::Program;
use ac2_script::{load_wlib, LEGIONS_WLIB_DID};
use ac2_weenie::{GameplayNotice, SessionEvent, INVENTORY_REFRESH};
use common::*;
use legions_server_double::{Spawn, FELLOW, PLAYER, TRADE_COMPLETED};

const SWORD: u64 = 0x8000_0002_0000_0021;
const ROBE: u64 = 0x8000_0002_0000_0022;
const SHIELD: u64 = 0x8000_0002_0000_0031;
const HELM: u64 = 0x8000_0002_0000_0032;
/// `CWInterface`'s package in the Legions library (the stub packs the accepting instance).
const CWINTERFACE_PACKAGE: u32 = 0x235;

fn partner_name() -> ac2_net::protocol::StringInfo {
    ac2_net::protocol::StringInfo {
        literal: Some("Birch".into()),
        ..Default::default()
    }
}

fn spawn() -> Spawn {
    Spawn {
        cell: CELL,
        origin: [70.0, 70.0, GROUND],
        rotation: [1.0, 0.0, 0.0, 0.0],
    }
}

/// One frame with `commands`, then the notices it and the frames after it raised.
fn act(
    client: &mut ac2_client::Client,
    server: &std::rc::Rc<std::cell::RefCell<legions_server_double::State>>,
    commands: Vec<Command>,
) -> Vec<GameplayNotice> {
    let mut notices = Vec::new();
    let input = FrameInput {
        commands,
        ..FrameInput::default()
    };
    let idle = FrameInput::default();
    let mut events = step(client, server, &input);
    for _ in 0..4 {
        events.extend(step(client, server, &idle));
    }
    for event in events {
        if let SessionEvent::Notice(notice) = event {
            notices.push(notice);
        }
    }
    notices
}

/// A client with the player's script attached, in the trade world: the player holds SWORD and
/// ROBE, FELLOW holds SHIELD and HELM. `None` without `AC2_DAT_DIR`.
fn trade_client() -> Option<(
    ac2_client::Client,
    std::rc::Rc<std::cell::RefCell<legions_server_double::State>>,
)> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = ac2_db::ResourceStore::new(dataset.clone());
    let wlib = load_wlib(&store, DataId::new(LEGIONS_WLIB_DID))
        .unwrap()
        .unwrap();
    let program = Arc::new(Program::from_stream(&wlib.stream));
    let data = DatScriptData::new(dataset, store);
    data.preload_tables();
    let (mut client, server) = entered(spawn());
    {
        let mut s = server.borrow_mut();
        s.script_world
            .contents
            .insert(PLAYER, (Vec::new(), vec![SWORD, ROBE]));
        s.script_world
            .contents
            .insert(FELLOW, (Vec::new(), vec![SHIELD, HELM]));
    }
    client.attach_scripts(
        ScriptHost::new(program, Box::new(data)),
        ac2_client::SCRIPT_VM_FUNCTIONS,
    );
    let idle = FrameInput::default();
    for _ in 0..10 {
        step(&mut client, &server, &idle);
    }
    Some((client, server))
}

/// `CPlayer` 0x10F (the `WM_Trade` 0x48 forwarder) raises `CM_Trade::SendNotice_Trade_BeRevoked`
/// on the VM when the partner takes an offered item back.
#[test]
fn the_partner_taking_an_offer_back_raises_be_revoked() {
    let Some((mut client, server)) = trade_client() else {
        return;
    };
    act(&mut client, &server, vec![Command::OpenTrade(FELLOW)]);
    server.borrow_mut().partner_offer(SHIELD, 1);
    act(&mut client, &server, Vec::new());
    server.borrow_mut().partner_revoke(SHIELD);
    let revoked = act(&mut client, &server, Vec::new());
    assert_eq!(
        revoked,
        [GameplayNotice::TradeRevoked {
            source: FELLOW,
            item: SHIELD
        }]
    );
    assert!(client.take_script_problems().is_empty());
}

#[test]
fn the_trade_window_runs_through_the_player_script() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ac2_db::ResourceStore::new(dataset.clone());
    let wlib = load_wlib(&store, DataId::new(LEGIONS_WLIB_DID))
        .unwrap()
        .unwrap();
    let program = Arc::new(Program::from_stream(&wlib.stream));
    let data = DatScriptData::new(dataset, store);
    data.preload_tables();
    let (mut client, server) = entered(spawn());
    {
        let mut s = server.borrow_mut();
        s.script_world
            .contents
            .insert(PLAYER, (Vec::new(), vec![SWORD, ROBE]));
        s.script_world
            .contents
            .insert(FELLOW, (Vec::new(), vec![SHIELD, HELM]));
    }
    client.attach_scripts(
        ScriptHost::new(program, Box::new(data)),
        ac2_client::SCRIPT_VM_FUNCTIONS,
    );
    let idle = FrameInput::default();
    for _ in 0..10 {
        step(&mut client, &server, &idle);
    }

    let opened = act(&mut client, &server, vec![Command::OpenTrade(FELLOW)]);
    assert_eq!(
        opened,
        [
            GameplayNotice::TradeRegistered {
                initiator: PLAYER,
                partner: FELLOW,
                partner_name: partner_name(),
            },
            GameplayNotice::TradeOpened,
        ]
    );
    let offer = act(
        &mut client,
        &server,
        vec![Command::OfferItemForTrade {
            item: SWORD,
            amount: 1,
        }],
    );
    assert_eq!(
        offer,
        [GameplayNotice::TradeOffered {
            source: PLAYER,
            item: SWORD,
            amount: 1
        }]
    );
    server.borrow_mut().partner_offer(SHIELD, 1);
    let partner = act(&mut client, &server, Vec::new());
    assert_eq!(
        partner,
        [GameplayNotice::TradeOffered {
            source: FELLOW,
            item: SHIELD,
            amount: 1
        }]
    );
    // The player's accept goes out as the IFun's stub packed it: the VM's reference.
    let accept = act(&mut client, &server, vec![Command::AcceptTrade]);
    assert_eq!(accept, [GameplayNotice::TradeAccepted(PLAYER)]);
    server.borrow_mut().partner_accept();
    let closed = act(&mut client, &server, Vec::new());
    // The VM's notices join the frame after the session's own events, so the contents' refresh
    // comes before the partner's accept (the stand-in path puts it after).
    assert_eq!(
        closed,
        [
            GameplayNotice::UpdateContainer {
                container: PLAYER,
                code: INVENTORY_REFRESH,
                item: 0,
                slot: 0
            },
            GameplayNotice::TradeAccepted(FELLOW),
            GameplayNotice::TradeClosed {
                source: PLAYER,
                kind: TRADE_COMPLETED
            },
        ]
    );
    assert!(client.take_script_problems().is_empty());
    let s = server.borrow();
    assert!(s.script_errors.is_empty(), "{:?}", s.script_errors);
    assert_eq!(s.script_world.contents[&PLAYER].1, [ROBE, SHIELD]);
    assert_eq!(s.script_world.contents[&FELLOW].1, [HELM, SWORD]);

    // The requests: the open, the offer, then the accept as one `AcceptTrade` whose reference is
    // the packed CWInterface graph (the root is the interface's package, 0x235; its hash holds
    // the offered item). Its event is the stub's `InterpSEvent`, and the bytes round-trip.
    let requests = &s.script_requests;
    assert_eq!(requests.len(), 3, "{requests:?}");
    assert_eq!(requests[0], ScriptRequest::OpenTrade(FELLOW));
    assert_eq!(
        requests[1],
        ScriptRequest::OfferItem {
            item: SWORD,
            amount: 1
        }
    );
    let ScriptRequest::AcceptTrade(reference) = &requests[2] else {
        panic!("{:?}", requests[2]);
    };
    assert_eq!(reference.entries.len(), 3);
    let PackedEntry::Heap { object, .. } = &reference.entries[0] else {
        panic!("{:?}", reference.entries[0]);
    };
    assert_eq!(object.pkg_id, CWINTERFACE_PACKAGE);
    let hashed = reference.entries.iter().any(|e| {
        matches!(e, PackedEntry::Heap { object, .. }
            if matches!(&object.native, Some(NativeObject::Hash(h))
                if h.keys == [HashKey::Value(SWORD)]))
    });
    assert!(hashed, "{reference:?}");
    let (queue, bytes) = ScriptRequest::AcceptTrade(reference.clone())
        .to_event()
        .encode();
    assert_eq!(queue, 3);
    let arguments = pack_values(&[PackValue::Reference(reference.clone())]);
    assert_eq!(
        bytes[..12],
        [0x0001_0053u32, 0x8215_0035, arguments.len() as u32]
            .iter()
            .flat_map(|w| w.to_le_bytes())
            .collect::<Vec<u8>>()[..]
    );
    assert_eq!(bytes[12..], arguments[..]);
    assert_eq!(
        unpack_values(&arguments).unwrap()[0],
        PackValue::Reference(reference.clone())
    );
}
