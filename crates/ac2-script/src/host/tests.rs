//! The host over the Legions library (`AC2_DAT_DIR`) and over a synthetic program.

use std::sync::Arc;

use ac2_core::DataId;
use ac2_db::ResourceStore;
use ac2_net::protocol::{script_function, PackValue, ScriptMessage, SelectionInfo};

use super::*;
use crate::data::DatScriptData;
use crate::vm::NoData;
use crate::{load_wlib, LEGIONS_WLIB_DID};

mod collision;
mod physics;

fn legions_host() -> Option<ScriptHost> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = ResourceStore::new(dataset.clone());
    let wlib = load_wlib(&store, DataId::new(LEGIONS_WLIB_DID)).ok()??;
    let program = Program::from_stream(&wlib.stream);
    Some(ScriptHost::new(
        Arc::new(program),
        Box::new(DatScriptData::new(dataset, store)),
    ))
}

#[test]
fn dispatch_pushes_retail_frames_and_reports_unknown_objects() {
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    let mut host = ScriptHost::new(Arc::new(program), Box::new(NoData));
    assert_eq!(
        host.dispatch(7, 0x8212_002C, &[]),
        Err(HostError::UnknownObject(7))
    );
    assert!(host.take_server_calls().is_empty());
}

/// The player's script (`CPlayer`, a fresh instance: its class-default WState is in the older
/// layout retail cannot load) takes server calls: `WM_Player` 0x2C (UpdateSelectionInfo) runs
/// to the end, and `WM_Communication` 0x11 raises `DisplayStringInfo` before it reaches a
/// member the server's state would have set.
#[test]
fn the_player_script_takes_server_calls_and_raises_notices() {
    let Some(mut host) = legions_host() else {
        return;
    };
    let pid = host.vm().program.package_id("CPlayer").expect("CPlayer");
    assert!(host.create_weenie(0x5000_0001, pid, 0));
    host.set_player(0x5000_0001);
    let info = SelectionInfo {
        kind: SelectionInfo::CREATURE,
        cur_health: 40,
        pk_damage: 0,
        max_health: 50,
        cur_vigor: 7,
        pk_vigor_loss: 0,
        max_vigor: 9,
    };
    let message = ScriptMessage::UpdateSelectionInfo { info, id: 0x77 };
    let outcome = host
        .dispatch(0x5000_0001, message.function(), &message.arguments())
        .unwrap();
    assert_eq!(outcome.status, InterpStatus::Terminated);
    let text = ac2_net::protocol::StringInfo {
        string_id: 0,
        table_id: 0,
        literal: Some("Hello".into()),
        variables: Vec::new(),
    };
    let message = ScriptMessage::DisplayStringInfo {
        function: script_function::DISPLAY_STRING_INFO,
        text: text.clone(),
        kind: 1,
    };
    host.dispatch(0x5000_0001, message.function(), &message.arguments())
        .unwrap();
    assert_eq!(
        host.take_notices(),
        vec![GameplayNotice::DisplayStringInfo { kind: 1, text }]
    );
    assert!(host.take_dropped().is_empty());
}

/// A script that packs every value kind and sends it: the call comes out in wire form, the
/// package as its packed graph.
#[test]
fn pack_buffer_and_send_server_evt_reach_the_outbox() {
    use crate::native::{ntype, NativeObject};
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    let mut host = ScriptHost::new(Arc::new(program), Box::new(NoData));
    let vm = host.vm_mut();
    let buffer = vm.refs.create_native(
        ntype::PACK_BUFFER,
        NativeObject::new_empty(ntype::PACK_BUFFER),
    );
    let text = vm.new_wstring("hi".encode_utf16().collect());
    let call = |vm: &mut Vm, id: u32, args: &[u32], this: u32| {
        let mut ctx = Context::new(1, None);
        ctx.enter(0).unwrap();
        ctx.push(0, tag::INT).unwrap();
        for &a in args.iter().rev() {
            ctx.push(a, tag::INT).unwrap();
        }
        ctx.push(this, tag::REF).unwrap();
        assert_eq!(vm.run_efun(id, &mut ctx), InterpStatus::Success);
    };
    call(vm, 0x0050_0002, &[7], buffer); // PackInt
    call(vm, 0x0050_0003, &[1, 2], buffer); // PackLInt (high word first)
    call(vm, 0x0050_0004, &[1.5f32.to_bits()], buffer); // PackFloat
    call(vm, 0x0050_0001, &[text], buffer); // PackPackage
    call(vm, 0x007F_0036, &[0x8212_0016, buffer], u32::MAX); // SendServerEvt
    let calls = host.take_server_calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].function, 0x8212_0016);
    assert!(!calls[0].encrypt);
    assert_eq!(
        calls[0].args[..3],
        [
            PackValue::Int(7),
            PackValue::LongInt(0x1_0000_0002),
            PackValue::Float(1.5),
        ]
    );
    assert_eq!(
        calls[0].args[3].as_native(),
        Some(&ac2_net::protocol::NativeObject::WString("hi".into()))
    );
}

/// Runs efun `id` with `this` and `args` (word 2 first) over a frame with a result slot.
fn run_efun(vm: &mut Vm, id: u32, args: &[u32], this: u32) -> InterpStatus {
    let mut ctx = Context::new(1, None);
    ctx.enter(0).unwrap();
    ctx.push(0, tag::INT).unwrap();
    for &a in args.iter().rev() {
        ctx.push(a, tag::INT).unwrap();
    }
    ctx.push(this, tag::REF).unwrap();
    vm.run_efun(id, &mut ctx)
}

/// Runs efun `id` and returns its frame slot `ret` (word 2 is the first argument).
fn efun_result(vm: &mut Vm, id: u32, args: &[u32], this: u32, ret: usize) -> u32 {
    let mut ctx = Context::new(1, None);
    ctx.enter(0).unwrap();
    // The caller reserves the words from the answer slot down to the arguments (the latent
    // efuns answer deeper than the others: DoMode and DoBehavior at slot 4, ReleaseBehavior at 5).
    for _ in 0..ret.saturating_sub(args.len() + 1).max(1) {
        ctx.push(0, tag::INT).unwrap();
    }
    for &a in args.iter().rev() {
        ctx.push(a, tag::INT).unwrap();
    }
    ctx.push(this, tag::REF).unwrap();
    let fp = ctx.sp;
    assert_eq!(vm.run_efun(id, &mut ctx), InterpStatus::Success);
    ctx.data[fp - ret]
}

/// `gGameplayOptionsProfile`: the "show" bits of hidden things invert, the alias list is
/// "alias [%s] == [%s]\n" per alias, and a chat window without a text type answers 0x10.
#[test]
fn the_options_profile_glue_follows_legions() {
    use crate::native::{ntype, NativeObject};
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    let mut host = ScriptHost::new(Arc::new(program), Box::new(NoData));
    let vm = host.vm_mut();
    let p = vm.refs.create_native(
        ntype::GAMEPLAY_OPTIONS_PROFILE,
        NativeObject::new_empty(ntype::GAMEPLAY_OPTIONS_PROFILE),
    );
    assert_eq!(efun_result(vm, 0x009B_0056, &[], p, 2), 1); // GetShowPetUI
    efun_result(vm, 0x009B_0055, &[0x100], p, 2); // SetShowPetUI(low byte 0)
    assert_eq!(efun_result(vm, 0x009B_0056, &[], p, 2), 0);
    assert_eq!(efun_result(vm, 0x009B_002D, &[], p, 2), 0x1_0000); // GetBitfield
    let (a, b) = (
        vm.new_wstring("hi".encode_utf16().collect()),
        vm.new_wstring("/say hello".encode_utf16().collect()),
    );
    assert_eq!(efun_result(vm, 0x009B_0001, &[a, b], p, 4), 1); // SetAlias
    let list = efun_result(vm, 0x009B_0007, &[], p, 2); // BuildAliasList
    assert_eq!(
        String::from_utf16_lossy(vm.wstring(list).unwrap()),
        "alias [hi] == [/say hello]\n"
    );
    assert_eq!(efun_result(vm, 0x009B_0054, &[9], p, 3), 0x10);
    assert_eq!(efun_result(vm, 0x009B_0054, &[2], p, 3), 0x8_0000);
    // AddFilter / RemoveFilter on window 2.
    efun_result(vm, 0x009B_001F, &[2, 0x3], p, 4);
    efun_result(vm, 0x009B_0020, &[2, 0x8_0001], p, 4);
    assert_eq!(efun_result(vm, 0x009B_0022, &[2], p, 3), 0x2);
}

/// `gQualifiedDataIDArray::WslAddToEnd`: a DID joins once; 0 never does.
#[test]
fn data_id_sets_add_each_did_once() {
    use crate::native::{ntype, NativeObject};
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    let mut host = ScriptHost::new(Arc::new(program), Box::new(NoData));
    let vm = host.vm_mut();
    let set = vm.refs.create_native(
        ntype::QUALIFIED_DATA_ID_ARRAY,
        NativeObject::new_empty(ntype::QUALIFIED_DATA_ID_ARRAY),
    );
    assert_eq!(efun_result(vm, 0x005E_0001, &[0x0400_0001], set, 3), 1);
    assert_eq!(efun_result(vm, 0x005E_0001, &[0x0400_0001], set, 3), 0);
    assert_eq!(efun_result(vm, 0x005E_0001, &[0], set, 3), 0);
    assert!(matches!(vm.native(set), Some(NativeObject::DataIds(s)) if s.len() == 1));
}

/// `CM_*` notices without an ac2-weenie type come out decoded by their signature; ones with a
/// type come out typed.
#[test]
fn notices_decode_by_signature() {
    use crate::native::{ntype, NativeObject};
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    let mut host = ScriptHost::new(Arc::new(program), Box::new(NoData));
    let vm = host.vm_mut();
    let obj = vm.new_wstring(Vec::new());
    vm.owners.insert(obj, 0x5000_0001);
    // CM_Quest::SendNotice_PlaySequence(obj, scenes): no ac2-weenie type (a null list here).
    run_efun(vm, 0x008E_0003, &[obj, u32::MAX], u32::MAX);
    // CM_Character::SendNotice_UpdateSkillPanel(obj, availableXP, credits, heroCredits).
    run_efun(vm, 0x0085_0005, &[obj, 0x1, 0x2, 3, 4], u32::MAX);
    // CM_Inventory::SendNotice_UpdateInventory(obj, code, itemID, location).
    run_efun(vm, 0x008B_0001, &[obj, 0x4000_0001, 0, 9, 2], u32::MAX);
    // A fellowship: two fellows added (one twice), then the notice and a fellow update.
    let new = |vm: &mut Vm, t: u32| vm.refs.create_native(t, NativeObject::new_empty(t));
    let fs = new(vm, ntype::FELLOWSHIP_ADAPTOR);
    let fellow = new(vm, ntype::FELLOW_ADAPTOR);
    run_efun(vm, 0x0093_0002, &[7], fellow); // SetLevel
    assert_eq!(efun_result(vm, 0x0094_0001, &[0, 48, fellow], fs, 5), 1);
    assert_eq!(efun_result(vm, 0x0094_0001, &[0, 1, fellow], fs, 5), 1);
    assert_eq!(efun_result(vm, 0x0094_0001, &[0, 48, fellow], fs, 5), 0);
    run_efun(vm, 0x0094_0003, &[0, 1], fs); // SetLeader
    run_efun(vm, 0x008A_0001, &[obj, fs], u32::MAX);
    run_efun(vm, 0x008A_0008, &[obj, 0, 1, 30, 2], u32::MAX);
    // CM_Usage::SendNotice_UseBook(obj, showControls, mode, title, text, image): the title and
    // the text are strings, the rest raw.
    let title = vm.new_wstring("Tome".encode_utf16().collect());
    let body = vm.new_wstring("Words".encode_utf16().collect());
    run_efun(
        vm,
        0x0092_0002,
        &[obj, 1, 5, title, body, 0x7000_0001],
        u32::MAX,
    );
    assert_eq!(
        host.take_script_notices(),
        vec![ScriptNotice {
            name: "CM_Quest::SendNotice_PlaySequence",
            object: 0x5000_0001,
            args: vec![NoticeArg::Null],
        }]
    );
    let notices = host.take_notices();
    assert_eq!(
        notices[..2],
        [
            GameplayNotice::UpdateSkillPanel {
                available_xp: 0x1_0000_0002,
                skill_credits: 3,
                hero_skill_credits: 4
            },
            GameplayNotice::UpdateInventory {
                code: 0x4000_0001,
                item: 9,
                location: 2
            },
        ]
    );
    // Fellows in table order (id % 47: 48 -> bucket 1, 1 -> bucket 1, the later at the head).
    assert!(notices.contains(&GameplayNotice::UseBook {
        show_controls: true,
        mode: 5,
        title: "Tome".to_owned(),
        text: "Words".to_owned(),
        image: 0x7000_0001,
    }));
    let GameplayNotice::UpdateFellowship(f) = &notices[2] else {
        panic!("{notices:?}");
    };
    let ids: Vec<u64> = f.fellows.iter().map(|(id, _)| *id).collect();
    assert_eq!((ids, f.leader, f.fellows[0].1.level), (vec![1, 48], 1, 7));
    assert_eq!(
        notices[3],
        GameplayNotice::UpdateFellowHealth {
            fellow: 1,
            value: 30,
            pk: 2
        }
    );
}

/// The quest journal's data: `gmQuestInfoList::UpdateQuestInfo` keeps the newest bestowal
/// first and updates a known quest in place (`bNew` 0), and `CM_Quest::SendNotice_UpdateQuests`
/// hands the quest on typed.
#[test]
fn quests_sort_newest_first_and_notify_typed() {
    use crate::native::{ntype, NativeObject};
    use ac2_net::protocol::{GmQuestInfo, NativeObject as W};
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    let mut host = ScriptHost::new(Arc::new(program), Box::new(NoData));
    let vm = host.vm_mut();
    let obj = vm.new_wstring(Vec::new());
    vm.owners.insert(obj, 0x5000_0001);
    let new = |vm: &mut Vm, t: u32| vm.refs.create_native(t, NativeObject::new_empty(t));
    let list = new(vm, ntype::GM_QUEST_INFO_LIST);
    let quest = |vm: &mut Vm, id: u32, bestowal: f64| {
        let q = GmQuestInfo {
            id,
            bestowal,
            ..GmQuestInfo::default()
        };
        vm.refs.create_native(
            ntype::GM_QUEST_INFO,
            NativeObject::Wire(Box::new(W::GmQuestInfo(Box::new(q)))),
        )
    };
    for (id, at) in [(1, 10.0), (2, 30.0), (3, 20.0)] {
        let q = quest(vm, id, at);
        assert_eq!(efun_result(vm, 0x00A1_0002, &[q, u32::MAX], list, 4), 1);
    }
    let ids: Vec<u32> = match vm.native(list) {
        Some(NativeObject::Wire(w)) => match &**w {
            W::GmQuestInfoList(l) => l.iter().map(|q| q.id).collect(),
            _ => Vec::new(),
        },
        _ => Vec::new(),
    };
    assert_eq!(ids, [2, 3, 1]);
    assert_eq!(efun_result(vm, 0x00A1_0006, &[], list, 2), 3);
    let q = quest(vm, 3, 20.0);
    assert_eq!(efun_result(vm, 0x00A1_0004, &[q], list, 3), 1); // Erase
    assert_eq!(efun_result(vm, 0x00A1_0006, &[], list, 2), 2);
    // CM_Quest::SendNotice_UpdateQuests(obj, code, quest).
    run_efun(vm, 0x008E_0001, &[obj, 5, q], u32::MAX);
    // The list notice (code 2) names no quest: a null quest still reaches the journal.
    run_efun(vm, 0x008E_0001, &[obj, 0x4000_0002, u32::MAX], u32::MAX);
    assert!(matches!(
        &host.take_notices()[..],
        [
            GameplayNotice::UpdateQuests { code: 5, quest },
            GameplayNotice::UpdateQuests { code: 0x4000_0002, .. },
        ] if quest.id == 3
    ));
}

/// A store page built through `gUIShop` / `gUIShopSale` (sales kept in ordinal order) and a
/// damage text with its colour (`RGBAColor::SetColor32`) reach the UI typed.
#[test]
fn stores_and_damage_text_reach_the_ui_typed() {
    use crate::native::{ntype, NativeObject};
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    let mut host = ScriptHost::new(Arc::new(program), Box::new(NoData));
    let vm = host.vm_mut();
    let obj = vm.new_wstring(Vec::new());
    vm.owners.insert(obj, 0x5000_0001);
    let new = |vm: &mut Vm, t: u32| vm.refs.create_native(t, NativeObject::new_empty(t));
    let shop = new(vm, ntype::UI_SHOP);
    run_efun(vm, 0x380A_0009, &[7], shop); // SetID
    for ordinal in [5, 2, 9] {
        let sale = new(vm, ntype::UI_SHOP_SALE);
        run_efun(vm, 0x380B_0009, &[ordinal], sale); // SetOrdinal
        run_efun(vm, 0x380B_000C, &[], sale); // SetSold
        run_efun(vm, 0x380A_0004, &[sale], shop); // AddSale
    }
    run_efun(vm, 0x3839_0006, &[obj, shop], u32::MAX); // ViewStore
    let color = new(vm, ntype::RGBA_COLOR);
    run_efun(vm, 0x0064_0004, &[0xFF00_8000], color); // SetColor32
    assert_eq!(efun_result(vm, 0x0064_0005, &[], color, 2), 0xFF00_8000);
    let blob = new(vm, ntype::DAMAGE_TEXT_BLOB);
    run_efun(vm, 0x0097_0003, &[color], blob); // SetFontColor
    run_efun(vm, 0x0097_0007, &[1.5f32.to_bits()], blob); // SetScale
    run_efun(vm, 0x008D_0001, &[obj, blob], u32::MAX); // Do3DDamage
    let notices = host.take_notices();
    let GameplayNotice::ViewStore(s) = &notices[0] else {
        panic!("{notices:?}");
    };
    let ordinals: Vec<u32> = s.sales.iter().map(|x| x.ordinal).collect();
    assert_eq!((s.id, ordinals, s.sales[0].status), (7, vec![2, 5, 9], 3));
    let GameplayNotice::Do3DDamage { object, text: d } = &notices[1] else {
        panic!("{notices:?}");
    };
    assert_eq!((*object, d.scale), (0x5000_0001, 1.5));
    assert_eq!(d.color[1], (128.0 * f64::from(0.003_921_569_f32)) as f32);
}

/// `gShortcutInfo` sets every field; the wire form packs the type and the datum it names.
#[test]
fn shortcut_infos_keep_their_fields_and_pack_their_datum() {
    use crate::native::{ntype, NativeObject};
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    let mut host = ScriptHost::new(Arc::new(program), Box::new(NoData));
    let vm = host.vm_mut();
    let s = vm.refs.create_native(
        ntype::SHORTCUT_INFO,
        NativeObject::new_empty(ntype::SHORTCUT_INFO),
    );
    run_efun(vm, 0x00AC_0005, &[0x4000_0002], s); // SetType
    run_efun(vm, 0x00AC_0006, &[0x8000_0001, 0x42], s); // SetDataIID (high word first)
    run_efun(vm, 0x00AC_0007, &[9], s); // SetDataEnum (kept, not packed)
    assert_eq!(efun_result(vm, 0x00AC_0001, &[], s, 2), 0x4000_0002);
    assert_eq!(efun_result(vm, 0x00AC_0003, &[], s, 2), 9);
    let Some(NativeObject::Shortcut(sc)) = vm.native(s) else {
        panic!();
    };
    assert_eq!(
        sc.to_net(),
        ac2_net::protocol::ShortcutInfo::Instance(0x8000_0001_0000_0042)
    );
}

/// `RequestGUN_SendToRoom` goes out only through a chat service; with none, retail returns the
/// room id and sends nothing. GUN-only requests need GUN.
#[test]
fn chat_requests_follow_the_available_services() {
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    let mut host = ScriptHost::new(Arc::new(program), Box::new(NoData));
    let vm = host.vm_mut();
    let this = vm.new_wstring(Vec::new());
    let text = vm.new_wstring("hi".encode_utf16().collect());
    run_efun(vm, 0x00B4_0033, &[5, text, 0x10], this);
    run_efun(vm, 0x00B4_0037, &[1], this);
    assert!(host.take_chat_requests().is_empty());
    host.set_chat_services(ChatServices {
        turbine_chat: true,
        gun: false,
    });
    let vm = host.vm_mut();
    run_efun(vm, 0x00B4_0033, &[5, text, 0x10], this);
    run_efun(vm, 0x00B4_0037, &[1], this);
    assert_eq!(
        run_efun(vm, 0x00B4_0033, &[5, text, 0], u32::MAX),
        InterpStatus::Crash
    );
    assert_eq!(
        host.take_chat_requests(),
        vec![ChatRequest::SendToRoom {
            room: 5,
            text: "hi".into(),
            flags: 0x10
        }]
    );
}

/// `gCWRepository::SetInt` / `AdjInt` write the weenie's quality table; without qualities
/// (`m_qualities == NULL`) nothing is stored.
#[test]
fn repository_efuns_write_the_weenie_qualities() {
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    let mut host = ScriptHost::new(Arc::new(program), Box::new(NoData));
    let this = host.vm_mut().new_wstring(Vec::new());
    host.vm_mut().owners.insert(this, 0x5000_0001);
    run_efun(host.vm_mut(), 0x0081_0006, &[0x14, 7], this);
    assert!(host.qualities(0x5000_0001).is_none());
    host.set_qualities(0x5000_0001, Default::default());
    run_efun(host.vm_mut(), 0x0081_0006, &[0x14, 7], this);
    // AdjInt(stat, delta, default): 7 + 5.
    run_efun(host.vm_mut(), 0x0081_0009, &[0x14, 5, 0], this);
    let q = host.qualities(0x5000_0001).unwrap();
    assert_eq!(q.ints.get(&0x14), Some(&12));
}

/// `CBaseQualities::SetInt` on a description field (`IsInWeenieDesc`, stype 0x106 quantity):
/// the value goes into the weenie description, which `InqInt` reads first. (`AdjInt` works on
/// the table only, as retail's does.)
#[test]
fn repository_efuns_write_description_fields_into_the_description() {
    use ac2_net::protocol::WeenieDesc;
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    let mut host = ScriptHost::new(Arc::new(program), Box::new(NoData));
    let this = host.vm_mut().new_wstring(Vec::new());
    host.vm_mut().owners.insert(this, 0x5000_0002);
    let quantity = WeenieDesc {
        quantity: Some(1),
        data_mask: 0x800,
        ..WeenieDesc::default()
    };
    host.set_qualities(
        0x5000_0002,
        ac2_weenie::WeenieQualities::from_weenie_desc(quantity),
    );
    // SetInt(0x106, 3), then AdjInt(0x106, 2, 0): 3 + 2.
    run_efun(host.vm_mut(), 0x0081_0006, &[0x106, 3], this);
    let q = host.qualities(0x5000_0002).unwrap();
    assert_eq!(q.inq_int(0x106), Some(3));
    assert_eq!(q.weenie.as_ref().and_then(|w| w.quantity), Some(3));
    assert!(!q.ints.contains_key(&0x106));
}

/// `gCWRepository::RemoveInt` clears a weenie description field (`CBaseQualities::RemoveInt`
/// asks `gmWeenieDesc::RemoveInt` first), so the field reads as absent afterwards.
#[test]
fn repository_remove_clears_the_weenie_description_field() {
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    let mut host = ScriptHost::new(Arc::new(program), Box::new(NoData));
    let this = host.vm_mut().new_wstring(Vec::new());
    host.vm_mut().owners.insert(this, 0x5000_0001);
    host.set_qualities(0x5000_0001, Default::default());
    // SetInt(0x106 quantity, 7): a description field.
    run_efun(host.vm_mut(), 0x0081_0006, &[0x106, 7], this);
    let quantity = |host: &ScriptHost| {
        let q = host.qualities(0x5000_0001).unwrap();
        (q.weenie.as_ref().and_then(|w| w.quantity), q.ints.len())
    };
    assert_eq!(quantity(&host), (Some(7), 0));
    // RemoveInt(0x106): the description answers, the field is gone.
    run_efun(host.vm_mut(), 0x0081_0008, &[0x106, 0], this);
    assert_eq!(quantity(&host), (None, 0));
}

/// A data source holding one class default record (`CBaseQualities::m_defaults`).
struct OneClassDefault(u32, ac2_weenie::WeenieQualities);

impl crate::vm::ScriptData for OneClassDefault {
    fn wstate(&self, _did: u32) -> Option<crate::vm::StateRef> {
        None
    }

    fn class_defaults(&self, did: u32) -> Option<Arc<ac2_weenie::WeenieQualities>> {
        (did == self.0).then(|| Arc::new(self.1.clone()))
    }
}

/// `CBaseQualities::AdjInt` on a quality its table lacks starts from the class default
/// (`m_defaults`, `InqInt`), not from 0; the result is stored in the table. A quality with no
/// default starts from 0.
#[test]
fn adj_on_an_absent_quality_starts_from_the_class_default() {
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    let mut class = ac2_weenie::WeenieQualities::default();
    class.ints.insert(0x14, 10);
    let mut host = ScriptHost::new(
        Arc::new(program),
        Box::new(OneClassDefault(0x8000_00AB, class)),
    );
    let this = host.vm_mut().new_wstring(Vec::new());
    host.vm_mut().owners.insert(this, 0x5000_0001);
    host.set_qualities(
        0x5000_0001,
        ac2_weenie::WeenieQualities {
            did: 0x8000_00AB,
            ..Default::default()
        },
    );
    // AdjInt(0x14, 5, 0): the default 10 + 5.
    run_efun(host.vm_mut(), 0x0081_0009, &[0x14, 5, 0], this);
    // AdjInt(0x15, 4, 0): no default, so 0 + 4.
    run_efun(host.vm_mut(), 0x0081_0009, &[0x15, 4, 0], this);
    let q = host.qualities(0x5000_0001).unwrap();
    assert_eq!(
        (q.ints.get(&0x14), q.ints.get(&0x15)),
        (Some(&15), Some(&4))
    );
    // The table holds the value now: the next adjustment works on it, not the default.
    run_efun(host.vm_mut(), 0x0081_0009, &[0x14, 1, 0], this);
    assert_eq!(
        host.qualities(0x5000_0001).unwrap().ints.get(&0x14),
        Some(&16)
    );
}

/// A new native packs the reserved-reference bit retail's `ReservedRefTable::AddReference`
/// sets (0x04000000 on `CreateNativeObject`'s 0x40000100): exact bytes of a packed wpstring.
#[test]
fn new_natives_pack_the_reserved_bit() {
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    let mut host = ScriptHost::new(Arc::new(program), Box::new(NoData));
    let vm = host.vm_mut();
    let r = vm.new_wstring("hi".encode_utf16().collect());
    let bytes = crate::wire::to_net(&vm.refs.pack(r)).unwrap().to_bytes();
    let words: Vec<u32> = bytes[..20]
        .chunks_exact(4)
        .map(|w| u32::from_le_bytes([w[0], w[1], w[2], w[3]]))
        .collect();
    // count, id, flags, m_pkgStateID, m_pkgid << 16 | m_ntype.
    assert_eq!(words, [1, r, 0x4400_0100, 0, 0xFFFF_0076]);
}

/// The `CharacterGenSystem` singleton answers the creation screens' questions from its WState.
/// `CallIFun_InqAppliedAppearanceVDesc(0xFFFFFFFF, cgr)`: the defaults for a race and sex apply
/// a scale and the physique tables' keys to the global group (the appearance the screens show);
/// a description written onto keeps its parent.
#[test]
fn the_applied_appearance_follows_the_physique_values() {
    use crate::host::chargen;
    use crate::native::CharGenResult;
    let Some(mut host) = legions_host() else {
        return;
    };
    let vm = host.vm_mut();
    let (_, races) = chargen::race_sex_data(vm).expect("race/sex data");
    let (race, sexes) = races.first().cloned().expect("a race");
    let mut cgr = CharGenResult {
        race,
        sex: sexes[0],
        ..CharGenResult::default()
    };
    chargen::set_defaults(vm, &mut cgr).expect("defaults");
    let (ret, applied) =
        chargen::applied_appearance(vm, ac2_weenie::Appearance::default(), &cgr, u32::MAX)
            .expect("the applied appearance");
    // The script's answer for the whole mask is not read by the screens.
    let _ = ret;
    assert_ne!(applied.scale, [1.0; 3]);
    assert!(!applied.global.appearance.is_empty());
    assert!(applied.part_groups.is_empty());
    // The start appearance is the one written onto (`VisualDesc::VisualDesc(out)` copies it).
    let start = ac2_weenie::Appearance {
        parent: 0x1F00_0023,
        ..ac2_weenie::Appearance::default()
    };
    let (_, from_start) = chargen::applied_appearance(vm, start, &cgr, u32::MAX).unwrap();
    assert_eq!(from_start.parent, 0x1F00_0023);
    assert_eq!(from_start.scale, applied.scale);
}

#[test]
fn the_character_gen_system_answers_from_its_state() {
    use crate::host::chargen;
    use crate::native::CharGenResult;
    let Some(mut host) = legions_host() else {
        return;
    };
    let vm = host.vm_mut();
    let (ret, races) = chargen::race_sex_data(vm).expect("race/sex data");
    eprintln!("race/sex data {ret}: {races:x?} fault {:?}", vm.last_fault);
    let (ret, infos) = chargen::race_sex_info_data(vm).expect("race/sex info");
    eprintln!(
        "race/sex info {ret}: {} entries, first {:x?}",
        infos.len(),
        infos.first()
    );
    let player = chargen::avatar_entity_did(vm, false);
    let admin = chargen::avatar_entity_did(vm, true);
    eprintln!("avatar entity DIDs: player {player:x?} admin {admin:x?}");
    let Some((race, sexes)) = races.first().cloned() else {
        panic!("no races");
    };
    let sex = sexes.first().copied().unwrap_or(0);
    let mut cgr = CharGenResult {
        race,
        sex,
        ..CharGenResult::default()
    };
    let r = chargen::set_defaults(vm, &mut cgr);
    eprintln!(
        "defaults {r:?}: {:x?} fault {:?}",
        (
            cgr.race,
            cgr.sex,
            cgr.class,
            cgr.entity_did,
            cgr.phys_type_values()
        ),
        vm.last_fault
    );
    let areas = chargen::valid_starting_areas(vm, cgr.race, cgr.class);
    eprintln!("starting areas {areas:x?}");
    if let Some((_, areas)) = &areas {
        if let Some(area) = areas.first() {
            eprintln!(
                "starting position {:?}",
                chargen::starting_position(vm, *area)
            );
        }
    }
    let phys = chargen::phys_obj(vm, race, sex);
    eprintln!("phys obj {phys:x?}");
    assert!(!races.is_empty());
    let pairs: usize = races.iter().map(|(_, s)| s.len()).sum();
    assert_eq!(infos.len(), pairs);
    assert!(player.is_some_and(|d| d != 0) && admin.is_some_and(|d| d != 0));
    assert_eq!(r, Some(1));
    assert!(!cgr.phys_type_values().is_empty());
    assert!(phys.is_some_and(|d| d != 0));
}

/// The player's process (`CPlayer`, from its entity's WState) takes the login message
/// (`WM_Player` 0x27, `CPlayer` 0x65) as the server double sends it - the options profile, the
/// equipment and the contents - and runs it to the end, raising the panels' notices.
#[test]
fn init_player_runs_to_its_end() {
    let Some(mut host) = legions_host() else {
        return;
    };
    let player = 0x5000_0001;
    let pid = host.vm().program.package_id("CPlayer").expect("CPlayer");
    assert!(host.create_weenie(player, pid, 0x4700_110F));
    host.set_player(player);
    host.set_qualities(player, Default::default());
    let outcome = init_player(&mut host, player);
    assert_eq!(outcome.status, InterpStatus::Terminated, "{outcome:?}");
    let notices = host.take_notices();
    assert!(notices
        .iter()
        .any(|n| matches!(n, GameplayNotice::UpdateHeroStatus { .. })));
    assert!(notices
        .iter()
        .any(|n| matches!(n, GameplayNotice::UpdateSkillPanel { .. })));
    assert!(notices.contains(&GameplayNotice::ReloadOptions));
    assert!(notices
        .iter()
        .any(|n| matches!(n, GameplayNotice::UpdateMoney(_))));
    assert!(host.take_dropped().is_empty());
}

fn init_player(host: &mut ScriptHost, player: u64) -> DispatchOutcome {
    use ac2_net::protocol::{pack_id_list, Equipment, NativeObject as W};
    let mut args = vec![PackValue::Int(0); 14];
    args[3] = PackValue::native(W::GameplayOptionsProfile(Box::default()));
    let equipment = Equipment::from_locations(&[]);
    args[6] = equipment.first;
    args[7] = PackValue::Reference(equipment.by_location);
    args[8] = PackValue::Reference(equipment.by_item);
    for arg in &mut args[9..12] {
        *arg = PackValue::Reference(pack_id_list(&[]));
    }
    host.dispatch(player, script_function::INIT_PLAYER, &args)
        .unwrap()
}

/// After the login message the selection runs on the VM: the UI's `SetSelection` IFun
/// (0xB40002) raises SelectionChanged and the examination request and tells the server, and
/// `WM_Player` 0x2C then shows the selected object's info.
#[test]
fn the_selection_runs_on_the_vm_after_login() {
    let Some(mut host) = legions_host() else {
        return;
    };
    let player = 0x5000_0001;
    let pid = host.vm().program.package_id("CPlayer").expect("CPlayer");
    assert!(host.create_weenie(player, pid, 0x4700_110F));
    host.set_player(player);
    host.set_qualities(player, Default::default());
    init_player(&mut host, player);
    host.take_notices();
    host.take_script_notices();
    let r = host
        .call_ifun(player, 0x00B4_0002, &[IFunArg::Long(0x77)])
        .unwrap();
    assert_eq!(r.outcome.status, InterpStatus::Terminated, "{r:?}");
    let notices = host.take_notices();
    assert_eq!(notices[0], GameplayNotice::SelectionChanged(0x77));
    assert!(matches!(
        &notices[1],
        GameplayNotice::ExaminationRequestChanged(e) if e.iid == 0x77
    ));
    let calls = host.take_server_calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].args, vec![PackValue::LongInt(0x77)]);

    let info = SelectionInfo {
        kind: SelectionInfo::CREATURE,
        cur_health: 40,
        pk_damage: 0,
        max_health: 50,
        cur_vigor: 7,
        pk_vigor_loss: 0,
        max_vigor: 9,
    };
    let message = ScriptMessage::UpdateSelectionInfo { info, id: 0x77 };
    let o = host
        .dispatch(player, message.function(), &message.arguments())
        .unwrap();
    assert_eq!(o.status, InterpStatus::Terminated, "{o:?}");
    assert_eq!(
        host.take_notices(),
        vec![GameplayNotice::UpdateSelectionInfo { id: player, info }]
    );
}

/// Prints every bound efun (`cargo test -p ac2-script list_bound_efuns -- --ignored`).
#[test]
#[ignore]
fn list_bound_efuns() {
    let efuns = host_efuns();
    for (id, name) in crate::glue::efun_table() {
        if efuns.is_bound(id) {
            println!("BOUND {id:08x} {name}");
        }
    }
}

/// A UI call (`ggmCWInterface::CallIFun_*`) runs on the object's process and reads back its
/// return slot: here the method returns its int argument.
#[test]
fn ui_ifun_calls_read_back_the_return_slot() {
    use crate::vm::op;
    let w = |code: u32, t: u8, imm: i32| code | u32::from(t) << 8 | (imm as u32) << 16;
    let code = vec![
        // 0: initializer
        w(op::PUSH_FRAME, 0, 0),
        op::POP_FRAME,
        // 2: method 1 (this, arg): ret = arg
        w(op::PUSH_FRAME, 0, 0),
        w(op::LOAD, 0, -2),
        w(op::PUSHV, 2, -3),
        op::STORE,
        op::POP,
        op::POP_FRAME,
    ];
    let mut program = Program::from_parts(code, vec![vec![0, 1]], vec![vec![0, 8]]);
    program.package_sizes = vec![1];
    let mut host = ScriptHost::new(Arc::new(program), Box::new(NoData));
    assert!(host.create_weenie(9, 0, 0));
    let result = host
        .call_ifun(9, FunctionId::new(0, 1, false).0, &[IFunArg::Int(41)])
        .unwrap();
    assert_eq!(result.outcome.status, InterpStatus::Terminated);
    assert_eq!(result.retval, 41);
    assert_eq!(host.call_ifun(8, 0, &[]), Err(HostError::UnknownObject(8)));
}

/// The skill panels come from the `SkillPanelManager` singleton, which reads the `MasterList`
/// singleton's hash; the DATs leave it empty (the server's skill data fills it), so a fresh VM
/// answers every panel with no nodes and no crash.
#[test]
fn skill_panels_answer_through_the_singleton() {
    let Some(mut host) = legions_host() else {
        return;
    };
    for panel in [1, 2, 3, 0x4000_0001] {
        let data = host.skill_panel(panel).unwrap().unwrap();
        assert!(data.nodes.is_empty());
    }
}

/// The login's GUN line: `CPlayer` 0x65 asks GUN for the ignore and friends lists and sets
/// its flags; when `RequestGUN_GetIgnoredList` answers 0 (no `TurbineGun`: no `-gunserver`,
/// or its `Init` failed) it shows "Cannot connect to GUN server ..." (0x73651CBE in
/// 0x25000002) instead.
#[test]
fn the_login_asks_gun_or_says_it_cannot() {
    for gun in [false, true] {
        let Some(mut host) = legions_host() else {
            return;
        };
        host.set_chat_services(ChatServices {
            turbine_chat: true,
            gun,
        });
        let player = 0x5000_0001;
        let pid = host.vm().program.package_id("CPlayer").expect("CPlayer");
        assert!(host.create_weenie(player, pid, 0x4700_110F));
        host.set_player(player);
        init_player(&mut host, player);
        let lines: Vec<u32> = host
            .take_notices()
            .into_iter()
            .filter_map(|n| match n {
                GameplayNotice::DisplayStringInfo { text, .. } => Some(text.string_id),
                _ => None,
            })
            .collect();
        let requests = host.take_chat_requests();
        if gun {
            assert!(lines.is_empty());
            assert_eq!(
                requests,
                [
                    ChatRequest::GetIgnoredList {
                        user_requested: false
                    },
                    ChatRequest::GetFriendsList {
                        user_requested: false
                    },
                    ChatRequest::SetStateFlags(1),
                ]
            );
        } else {
            assert_eq!(lines, [0x7365_1CBE]);
            assert!(requests.is_empty());
        }
    }
}

/// `gCWDistributor::GetEntityDID`: the DID the weenie was made with (`m_initData.m_did`); a
/// null `this` crashes.
#[test]
fn get_entity_did_answers_the_weenies_init_did() {
    let Some(mut host) = legions_host() else {
        return;
    };
    let player = 0x5000_0001;
    let pid = host.vm().program.package_id("CPlayer").expect("CPlayer");
    assert!(host.create_weenie(player, pid, 0x4700_110F));
    let instance = host.instance(player).expect("the process instance");
    let vm = host.vm_mut();
    assert_eq!(efun_result(vm, 0x007E_0003, &[], instance, 2), 0x4700_110F);
    assert_eq!(
        run_efun(vm, 0x007E_0003, &[], u32::MAX),
        InterpStatus::Crash
    );
}

/// `gmCWInterface::HandleWeenieEmote` reaches the player's script as IFun 0xB400AA
/// (`ggmCWInterface::CallIFun_HandleWeenieEmote`): the call runs to its end, and an object with no
/// process has no answer.
#[test]
fn the_player_script_answers_the_weenie_emote_ifun() {
    let Some(mut host) = legions_host() else {
        return;
    };
    let pid = host.vm().program.package_id("CPlayer").expect("CPlayer");
    assert!(host.create_weenie(0x5000_0001, pid, 0));
    host.set_player(0x5000_0001);
    let result = host
        .call_ifun(0x5000_0001, 0x00B4_00AA, &[IFunArg::Int(0x2A)])
        .unwrap();
    assert_eq!(result.outcome.status, InterpStatus::Terminated);
    assert!(result.outcome.fault.is_none());
    assert_eq!(
        host.call_ifun(0x6, 0x00B4_00AA, &[IFunArg::Int(0x2A)]),
        Err(HostError::UnknownObject(0x6))
    );
}

/// `gCWPhysicsObject`'s DoFX, StopFX and scale efuns queue the server event each one is for the
/// weenie's physics object (the client applies them to the body); an undefined `this` crashes.
#[test]
fn physics_object_efuns_queue_their_server_events_for_the_weenie() {
    use ac2_net::protocol::ObjectAction;
    let program = Program::from_parts(vec![crate::vm::op::TERMINATE], Vec::new(), Vec::new());
    let mut host = ScriptHost::new(Arc::new(program), Box::new(NoData));
    host.vm.owners.insert(9, 0x55);
    let fx = 0x4000_0010;
    assert_eq!(
        efun_result(&mut host.vm, 0x0080_0045, &[fx, 0.5f32.to_bits()], 9, 4),
        1,
        "DoFX answers 1 for an owned weenie"
    );
    assert_eq!(
        efun_result(&mut host.vm, 0x0080_0047, &[fx], 9, 3),
        1,
        "StopFX answers 1"
    );
    assert_eq!(
        run_efun(&mut host.vm, 0x0080_0003, &[2.0f32.to_bits()], 9),
        InterpStatus::Success
    );
    assert_eq!(
        run_efun(&mut host.vm, 0x0080_0005, &[0.25f32.to_bits()], 9),
        InterpStatus::Success
    );
    assert_eq!(
        run_efun(&mut host.vm, 0x0080_0007, &[3.0f32.to_bits()], 9),
        InterpStatus::Success
    );
    assert_eq!(
        host.take_physics(),
        vec![
            (0x55, ObjectAction::DoFx { fx, scalar: 0.5 }),
            (0x55, ObjectAction::StopFx(fx)),
            (0x55, ObjectAction::SetVelocityScale(2.0)),
            (0x55, ObjectAction::SetAccelerationScale(0.25)),
            (0x55, ObjectAction::SetJumpScale(3.0)),
        ]
    );
    assert!(host.take_physics().is_empty());
    assert_ne!(
        run_efun(&mut host.vm, 0x0080_0003, &[1.0f32.to_bits()], u32::MAX),
        InterpStatus::Success,
        "an undefined this crashes the script"
    );
}
