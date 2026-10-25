//! `gSkillUINode` (`gSkillUINode.cpp`): the skill-tree node the skill UI reads, over ac2-net's
//! `SkillUiNode` (`SkillUINode`: enum, type, name and description `StringInfo`s, icon, tree
//! coordinates, parent and dependency lists, passive and untrainable flags).

use ac2_net::protocol::{NativeObject as Wire, SkillUiNode};

use super::{word, Call, INT_RESULT};
use crate::glue::lists::{write_out, Kind};
use crate::native::{ntype, NativeObject, ScriptList};
use crate::vm::{tag, Context, EFunFn, EFuns, InterpStatus, Vm};
use crate::wire::{string_info_from_net, string_info_to_net};

pub(super) fn register(efuns: &mut EFuns) {
    // `SetCoordinates` (0x0046EEB0) is the id the dispatcher table leaves unnamed between
    // `InqCoordinates` and `GetX`.
    if let Some(f) = method("SetCoordinates") {
        efuns.register(0x00AE_000C, f);
    }
    for (id, name) in super::efun_table() {
        if let Some(("gSkillUINode", m)) = name.split_once("::CallEFun_") {
            if let Some(f) = method(m) {
                efuns.register(id, f);
            }
        }
    }
}

/// `SkillUINode::SkillUINode` / `Clear`: everything zero, coordinates -1.
pub(crate) fn empty_node() -> SkillUiNode {
    SkillUiNode {
        skill_enum: 0,
        skill_type: 0,
        name: Default::default(),
        desc: Default::default(),
        icon: 0,
        x: -1,
        y: -1,
        parents: Vec::new(),
        dependencies: Vec::new(),
        is_passive: 0,
        untrainable: 0,
    }
}

fn node_mut(vm: &mut Vm, r: u32) -> Option<&mut SkillUiNode> {
    match vm.native_mut(r)? {
        NativeObject::Wire(w) => match &mut **w {
            Wire::SkillUiNode(n) => Some(n),
            _ => None,
        },
        _ => None,
    }
}

fn node(vm: &Vm, r: u32) -> Option<SkillUiNode> {
    match vm.native(r)? {
        NativeObject::Wire(w) => match &**w {
            Wire::SkillUiNode(n) => Some((**n).clone()),
            _ => None,
        },
        _ => None,
    }
}

/// Runs `f` on `this`'s node.
fn edit(
    vm: &mut Vm,
    ctx: &mut Context,
    f: impl FnOnce(&mut SkillUiNode, &Context),
) -> InterpStatus {
    let this = Call::new(ctx).this();
    match node_mut(vm, this) {
        Some(n) => {
            f(n, ctx);
            InterpStatus::Success
        }
        None => vm.bad_this(this),
    }
}

/// A word result in slot 2.
fn get(vm: &mut Vm, ctx: &mut Context, f: fn(&SkillUiNode) -> u32) -> InterpStatus {
    let this = Call::new(ctx).this();
    match node(vm, this) {
        Some(n) => {
            Call::new(ctx).ret(2, f(&n), INT_RESULT);
            InterpStatus::Success
        }
        None => vm.bad_this(this),
    }
}

/// `GetName` / `GetDescription`: a copy as a new `StringInfo`.
fn get_text(vm: &mut Vm, ctx: &mut Context, desc: bool) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some(n) = node(vm, this) else {
        return vm.bad_this(this);
    };
    let info = string_info_from_net(if desc { &n.desc } else { &n.name });
    let r = vm
        .refs
        .create_native(ntype::STRING_INFO, NativeObject::StringInfo(info));
    Call::new(ctx).ret(2, r, tag::REF);
    InterpStatus::Success
}

/// `SetName` / `SetDescription`: `StringInfo::operator=` from the argument.
fn set_text(vm: &mut Vm, ctx: &mut Context, desc: bool) -> InterpStatus {
    let source = word(ctx, 2);
    let Some(NativeObject::StringInfo(info)) = vm.native(source) else {
        return vm.bad_this(source);
    };
    let info = string_info_to_net(info);
    edit(vm, ctx, |n, _| {
        if desc {
            n.desc = info;
        } else {
            n.name = info;
        }
    })
}

/// `GetParentList` / `GetDependencyList`: a new `AList` copy (`ValueListToAList`).
fn get_list(vm: &mut Vm, ctx: &mut Context, dependencies: bool) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some(n) = node(vm, this) else {
        return vm.bad_this(this);
    };
    let items = if dependencies {
        &n.dependencies
    } else {
        &n.parents
    };
    let list = NativeObject::List(ScriptList {
        wide: false,
        items: items.iter().map(|v| u64::from(*v)).collect(),
    });
    let r = vm.refs.create_native(ntype::A_LIST, list);
    Call::new(ctx).ret(2, r, tag::REF);
    InterpStatus::Success
}

fn method(m: &str) -> Option<EFunFn> {
    Some(match m {
        "GetEnum" => |vm, c| get(vm, c, |n| n.skill_enum),
        "GetType" => |vm, c| get(vm, c, |n| n.skill_type),
        "GetIcon" => |vm, c| get(vm, c, |n| n.icon),
        "GetX" => |vm, c| get(vm, c, |n| n.x as u32),
        "GetY" => |vm, c| get(vm, c, |n| n.y as u32),
        "IsPassive" => |vm, c| get(vm, c, |n| n.is_passive),
        "IsUntrainable" => |vm, c| get(vm, c, |n| n.untrainable),
        "SetEnum" => |vm, c| edit(vm, c, |n, c| n.skill_enum = word(c, 2)),
        "SetType" => |vm, c| edit(vm, c, |n, c| n.skill_type = word(c, 2)),
        "SetIcon" => |vm, c| edit(vm, c, |n, c| n.icon = word(c, 2)),
        "SetPassive" => |vm, c| edit(vm, c, |n, c| n.is_passive = word(c, 2)),
        "SetUntrainable" => |vm, c| edit(vm, c, |n, c| n.untrainable = word(c, 2)),
        "SetCoordinates" => |vm, c| {
            edit(vm, c, |n, c| {
                n.x = word(c, 2) as i32;
                n.y = word(c, 3) as i32;
            })
        },
        "GetName" => |vm, c| get_text(vm, c, false),
        "GetDescription" => |vm, c| get_text(vm, c, true),
        "SetName" => |vm, c| set_text(vm, c, false),
        "SetDescription" => |vm, c| set_text(vm, c, true),
        "GetParentList" => |vm, c| get_list(vm, c, false),
        "GetDependencyList" => |vm, c| get_list(vm, c, true),
        // `SkillUINode::AddParent`: appended unless present.
        "AddParent" => |vm, c| {
            edit(vm, c, |n, c| {
                let p = word(c, 2);
                if !n.parents.contains(&p) {
                    n.parents.push(p);
                }
            })
        },
        // `SkillUINode::AddDependency`: appended unless present; 1 when added.
        "AddDependency" => |vm, c| {
            let d = word(c, 2);
            let mut added = false;
            let status = edit(vm, c, |n, _| {
                if !n.dependencies.contains(&d) {
                    n.dependencies.push(d);
                    added = true;
                }
            });
            Call::new(c).ret(3, u32::from(added), INT_RESULT);
            status
        },
        "Clear" => |vm, c| edit(vm, c, |n, _| *n = empty_node()),
        // `SkillUINode::operator=`.
        "Set" => |vm, c| {
            let Some(other) = node(vm, word(c, 2)) else {
                return vm.bad_this(word(c, 2));
            };
            edit(vm, c, |n, _| *n = other)
        },
        // `InqCoordinates(out x, out y)`: both outs tagged 4.
        "InqCoordinates" => |vm, c| {
            let this = Call::new(c).this();
            let Some(n) = node(vm, this) else {
                return vm.bad_this(this);
            };
            let (ox, oy) = (word(c, 2), word(c, 3));
            write_out(vm, c, ox, Some(u64::from(n.x as u32)), Kind::A);
            write_out(vm, c, oy, Some(u64::from(n.y as u32)), Kind::A);
            InterpStatus::Success
        },
        _ => return None,
    })
}
