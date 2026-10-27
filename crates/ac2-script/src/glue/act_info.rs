//! `ggmActInfo` (0x9C) and `ggmActInfoList` (`List<gmActInfo>`, 0x9D): the acts the story quests
//! list, over ac2-net's [`GmActInfo`]. The player's script builds them (`gmActInfo::Init` and
//! `AddScene`) and sorts them (`List<gmActInfo>::insert_sort`), and `gmCWInterface::InqActInfoList`
//! hands the list to the Quest panel's `ReloadActs`. `DumpW` (a debug dump) is not bound.
//!
//! Layouts and rules follow `gmActInfo` (`gmactinfo.cpp`: `Init` copies an already sorted scene
//! list or inserts each scene by `_uiSceneNum`, `AddScene` keeps the scenes in that order) and
//! `List<gmActInfo>` (`insert_sort` keeps the acts by `_uiActNum`, equal ones after).

use ac2_net::protocol::{GmActInfo, GmSceneInfo, NativeObject as Wire, StringInfo};

use super::{word, Call, INT_RESULT};
use crate::native::NativeObject;
use crate::vm::{Context, EFuns, InterpStatus, Vm};
use crate::wire::{scene_to_net, string_info_to_net};

pub(super) fn register(efuns: &mut EFuns) {
    efuns.register(0x009C_0001, init);
    efuns.register(0x009C_0002, add_scene);
    efuns.register(0x009C_0003, |vm, c| {
        let this = Call::new(c).this();
        let Some(id) = act(vm, this).map(|a| a.id) else {
            return vm.bad_this(this);
        };
        Call::new(c).ret(2, id, INT_RESULT);
        InterpStatus::Success
    });
    efuns.register(0x009D_0001, insert_sort);
    efuns.register(0x009D_0002, pop_front);
}

fn act(vm: &Vm, reference: u32) -> Option<&GmActInfo> {
    match vm.native(reference)? {
        NativeObject::Wire(w) => match &**w {
            Wire::GmActInfo(a) => Some(&**a),
            _ => None,
        },
        _ => None,
    }
}

fn act_mut(vm: &mut Vm, reference: u32) -> Option<&mut GmActInfo> {
    match vm.native_mut(reference)? {
        NativeObject::Wire(w) => match &mut **w {
            Wire::GmActInfo(a) => Some(&mut **a),
            _ => None,
        },
        _ => None,
    }
}

fn act_list_mut(vm: &mut Vm, reference: u32) -> Option<&mut Vec<GmActInfo>> {
    match vm.native_mut(reference)? {
        NativeObject::Wire(w) => match &mut **w {
            Wire::GmActInfoList(l) => Some(l),
            _ => None,
        },
        _ => None,
    }
}

/// A `gmActInfo` argument, copied (`gmActInfo::operator=`).
fn act_arg(vm: &Vm, reference: u32) -> Option<GmActInfo> {
    act(vm, reference).cloned()
}

/// A `gmSceneInfo` argument as ac2-net's form.
fn scene_arg(vm: &Vm, reference: u32) -> Option<GmSceneInfo> {
    match vm.native(reference)? {
        NativeObject::SceneInfo(s) => Some(scene_to_net(s)),
        _ => None,
    }
}

/// A `List<gmSceneInfo>` argument as ac2-net's form, in list order.
fn scene_list_arg(vm: &Vm, reference: u32) -> Vec<GmSceneInfo> {
    match vm.native(reference) {
        Some(NativeObject::SceneList(l)) => l.iter().map(scene_to_net).collect(),
        _ => Vec::new(),
    }
}

/// A `StringInfo` argument as ac2-net's form.
fn text_arg(vm: &Vm, reference: u32) -> StringInfo {
    match vm.native(reference) {
        Some(NativeObject::StringInfo(s)) => string_info_to_net(s),
        _ => StringInfo::default(),
    }
}

/// `List<gmSceneInfo>::insert`-by-number: before the first scene numbered after `scene`, else
/// last (`gmActInfo::AddScene`).
fn insert_scene(scenes: &mut Vec<GmSceneInfo>, scene: GmSceneInfo) {
    let at = scenes
        .iter()
        .position(|s| s.scene_num > scene.scene_num)
        .unwrap_or(scenes.len());
    scenes.insert(at, scene);
}

/// `gmActInfo::Init(actID, actName, actNum, bVisible, bPlayable, sceneList, bSorted)`: a sorted
/// scene list is copied as it is (`PList::operator=`); otherwise each scene is inserted into the
/// act's list by its number.
fn init(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    let incoming = scene_list_arg(vm, word(ctx, 7));
    let sorted = word(ctx, 8) != 0;
    let name = text_arg(vm, word(ctx, 3));
    let Some(slot) = act_mut(vm, this) else {
        return vm.bad_this(this);
    };
    slot.id = word(ctx, 2);
    slot.name = name;
    slot.act_num = word(ctx, 4);
    slot.visible = u32::from(word(ctx, 5) != 0);
    slot.playable = u32::from(word(ctx, 6) != 0);
    if sorted {
        slot.scenes = incoming;
    } else {
        for scene in incoming {
            insert_scene(&mut slot.scenes, scene);
        }
    }
    InterpStatus::Success
}

/// `gmActInfo::AddScene(scene)`: the scene inserted in number order; 1 when it was.
fn add_scene(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some(scene) = scene_arg(vm, word(ctx, 2)) else {
        Call::new(ctx).ret(3, 0, INT_RESULT);
        return InterpStatus::Success;
    };
    let Some(slot) = act_mut(vm, this) else {
        return vm.bad_this(this);
    };
    insert_scene(&mut slot.scenes, scene);
    Call::new(ctx).ret(3, 1, INT_RESULT);
    InterpStatus::Success
}

/// `List<gmActInfo>::insert_sort(act)`: before the first act numbered after it, else last.
fn insert_sort(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some(act) = act_arg(vm, word(ctx, 2)) else {
        Call::new(ctx).ret(3, 0, INT_RESULT);
        return InterpStatus::Success;
    };
    let Some(list) = act_list_mut(vm, this) else {
        return vm.bad_this(this);
    };
    let at = list
        .iter()
        .position(|a| a.act_num > act.act_num)
        .unwrap_or(list.len());
    list.insert(at, act);
    Call::new(ctx).ret(3, 1, INT_RESULT);
    InterpStatus::Success
}

/// `List<gmActInfo>::pop_front(out act)`: the head copied out and removed; 0 on an empty list.
fn pop_front(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let out = word(ctx, 2);
    let this = Call::new(ctx).this();
    let Some(list) = act_list_mut(vm, this) else {
        return vm.bad_this(this);
    };
    let head = if list.is_empty() {
        None
    } else {
        Some(list.remove(0))
    };
    let status = u32::from(head.is_some());
    if let Some(head) = head {
        let target = vm.read_through(ctx, out).unwrap_or(u32::MAX);
        if let Some(slot) = act_mut(vm, target) {
            *slot = head;
        }
    }
    Call::new(ctx).ret(3, status, INT_RESULT);
    InterpStatus::Success
}
