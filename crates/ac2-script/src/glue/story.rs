//! The story scene natives: `ggmKeyframe` (0x9F), `ggmSceneInfo` (0xA3) and `ggmSceneInfoList`
//! (`List<gmSceneInfo>`, 0xA4), over ac2-weenie's [`SceneInfo`] and [`Keyframe`]. The notices
//! `CM_Quest::SendNotice_PlaySequence` and `CM_UI::SendNotice_PortalScene` carry the list, and
//! `ggmCWInterface::CallIFun_HandlePlayScenesDone` hands the played list back to the script.
//!
//! Layouts and rules follow `gmKeyframe` (`gmkeyframe.cpp`: the tracks, `m_definedTracks` bits,
//! `HasDurationDefined`), `gmSceneInfo` (`gmsceneinfo.cpp`: `AddKeyframe` numbers the frame and
//! stamps the scene id, `Set` copies a scene) and `List<gmSceneInfo>` (`insert_sort` keeps the
//! scenes by `_uiSceneNum`, equal ones after). Calls that name a file (`SetVideoTrackByName`,
//! `SetMovieTrack`, ...) and the dump and verify efuns are not bound here.

use ac2_weenie::{Keyframe, SceneInfo, TRACK_AUDIO, TRACK_SUBTITLE, TRACK_VIDEO, TRACK_VOICE};

use super::enums::inq_enum_value;
use super::{word, word64, Call, INT_RESULT};
use crate::native::{ntype, NativeObject};
use crate::vm::{tag, Context, EFuns, InterpStatus, Vm};
use crate::wire::{string_info_from_net, string_info_to_net};

/// The enum map `gmSceneInfo::SetIDByName` looks its name up in (`EnumMapper::InqEnum`).
const SCENE_ID_MAP: u32 = 0x4100_0019;

pub(super) fn register(efuns: &mut EFuns) {
    efuns.register(0x009F_0002, |vm, c| {
        keyframe_do(vm, c, |k| {
            k.scene_id = 0;
            k.frame_num = 0;
            k.subtitle = Default::default();
            k.video = 0;
            k.audio = 0;
            k.voice = 0;
            k.defined_tracks = 0;
        })
    });
    efuns.register(0x009F_0003, |vm, c| get_keyframe(vm, c, |k| k.scene_id));
    efuns.register(0x009F_0004, |vm, c| {
        set_keyframe(vm, c, |k, v| k.scene_id = v)
    });
    efuns.register(0x009F_0005, |vm, c| get_keyframe(vm, c, |k| k.frame_num));
    efuns.register(0x009F_0006, |vm, c| {
        set_keyframe(vm, c, |k, v| k.frame_num = v)
    });
    efuns.register(0x009F_0007, |vm, c| get_keyframe(vm, c, |k| k.video));
    efuns.register(0x009F_0008, |vm, c| {
        set_track(vm, c, TRACK_VIDEO, |k, v| k.video = v)
    });
    efuns.register(0x009F_000A, |vm, c| get_keyframe(vm, c, |k| k.audio));
    efuns.register(0x009F_000B, |vm, c| {
        set_track(vm, c, TRACK_AUDIO, |k, v| k.audio = v)
    });
    efuns.register(0x009F_000D, |vm, c| get_keyframe(vm, c, |k| k.voice));
    efuns.register(0x009F_000E, |vm, c| {
        set_track(vm, c, TRACK_VOICE, |k, v| k.voice = v)
    });
    efuns.register(0x009F_0010, get_subtitle);
    efuns.register(0x009F_0011, set_subtitle);
    efuns.register(0x009F_0014, get_duration);
    efuns.register(0x009F_0015, set_duration);

    efuns.register(0x00A3_0004, |vm, c| {
        scene_do(vm, c, |s| {
            s.id = 0;
            s.name = Default::default();
            s.act_id = 0;
            s.scene_num = 0;
            s.hidden = false;
            s.playable = false;
            s.keyframes.clear();
        })
    });
    efuns.register(0x00A3_0005, |vm, c| get_scene(vm, c, |s| s.id));
    efuns.register(0x00A3_0006, |vm, c| set_scene(vm, c, |s, v| s.id = v));
    efuns.register(0x00A3_0007, set_id_by_name);
    efuns.register(0x00A3_0008, get_name);
    efuns.register(0x00A3_0009, set_name);
    efuns.register(0x00A3_000A, |vm, c| get_scene(vm, c, |s| s.act_id));
    efuns.register(0x00A3_000B, |vm, c| set_scene(vm, c, |s, v| s.act_id = v));
    efuns.register(0x00A3_000C, |vm, c| get_scene(vm, c, |s| s.scene_num));
    efuns.register(0x00A3_000D, |vm, c| {
        set_scene(vm, c, |s, v| s.scene_num = v)
    });
    efuns.register(0x00A3_000E, |vm, c| {
        get_scene(vm, c, |s| u32::from(s.hidden))
    });
    efuns.register(0x00A3_000F, |vm, c| {
        set_scene(vm, c, |s, v| s.hidden = v != 0)
    });
    efuns.register(0x00A3_0010, |vm, c| {
        get_scene(vm, c, |s| u32::from(s.playable))
    });
    efuns.register(0x00A3_0011, |vm, c| {
        set_scene(vm, c, |s, v| s.playable = v != 0)
    });
    efuns.register(0x00A3_0012, |vm, c| {
        scene_do(vm, c, |s| {
            for k in &mut s.keyframes {
                k.video = 0;
                k.audio = 0;
                k.voice = 0;
                k.subtitle = Default::default();
            }
        })
    });
    efuns.register(0x00A3_0013, set_from_model);
    efuns.register(0x00A3_0014, add_keyframe);

    efuns.register(0x00A4_0001, num_elements);
    efuns.register(0x00A4_0002, push_tail);
    efuns.register(0x00A4_0003, pop_front);
    efuns.register(0x00A4_0004, insert_sort);
}

fn keyframe(vm: &Vm, reference: u32) -> Option<&Keyframe> {
    match vm.native(reference)? {
        NativeObject::Keyframe(k) => Some(&**k),
        _ => None,
    }
}

fn keyframe_mut(vm: &mut Vm, reference: u32) -> Option<&mut Keyframe> {
    match vm.native_mut(reference)? {
        NativeObject::Keyframe(k) => Some(&mut **k),
        _ => None,
    }
}

fn scene(vm: &Vm, reference: u32) -> Option<&SceneInfo> {
    match vm.native(reference)? {
        NativeObject::SceneInfo(s) => Some(&**s),
        _ => None,
    }
}

fn scene_mut(vm: &mut Vm, reference: u32) -> Option<&mut SceneInfo> {
    match vm.native_mut(reference)? {
        NativeObject::SceneInfo(s) => Some(&mut **s),
        _ => None,
    }
}

fn scene_list_mut(vm: &mut Vm, reference: u32) -> Option<&mut Vec<SceneInfo>> {
    match vm.native_mut(reference)? {
        NativeObject::SceneList(l) => Some(l),
        _ => None,
    }
}

/// The scene a `StringInfo` native carries, as ac2-net's form (`StringInfo::operator=`).
fn text_arg(vm: &Vm, reference: u32) -> ac2_net::protocol::StringInfo {
    match vm.native(reference) {
        Some(NativeObject::StringInfo(s)) => string_info_to_net(s),
        _ => ac2_net::protocol::StringInfo::default(),
    }
}

/// A new `StringInfo` native holding `info` (`VM::CopyNativeObject(0x6A)`).
fn new_text(vm: &mut Vm, info: &ac2_net::protocol::StringInfo) -> u32 {
    vm.refs.create_native(
        ntype::STRING_INFO,
        NativeObject::StringInfo(string_info_from_net(info)),
    )
}

/// A `gmKeyframe` method that changes `this` and returns nothing.
fn keyframe_do(vm: &mut Vm, ctx: &mut Context, f: impl FnOnce(&mut Keyframe)) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some(k) = keyframe_mut(vm, this) else {
        return vm.bad_this(this);
    };
    f(k);
    InterpStatus::Success
}

/// A `gmKeyframe` getter: the word `f` reads, returned in slot 2.
fn get_keyframe(vm: &mut Vm, ctx: &mut Context, f: fn(&Keyframe) -> u32) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some(v) = keyframe(vm, this).map(f) else {
        return vm.bad_this(this);
    };
    Call::new(ctx).ret(2, v, INT_RESULT);
    InterpStatus::Success
}

/// A `gmKeyframe` setter taking one word.
fn set_keyframe(vm: &mut Vm, ctx: &mut Context, f: fn(&mut Keyframe, u32)) -> InterpStatus {
    let (this, v) = (Call::new(ctx).this(), word(ctx, 2));
    let Some(k) = keyframe_mut(vm, this) else {
        return vm.bad_this(this);
    };
    f(k, v);
    InterpStatus::Success
}

/// `gmKeyframe::SetVideoTrack` / `SetAudioTrack` / `SetVoiceTrack`: the DID and its track bit.
fn set_track(vm: &mut Vm, ctx: &mut Context, bit: u32, f: fn(&mut Keyframe, u32)) -> InterpStatus {
    let (this, did) = (Call::new(ctx).this(), word(ctx, 2));
    let Some(k) = keyframe_mut(vm, this) else {
        return vm.bad_this(this);
    };
    f(k, did);
    k.defined_tracks |= bit;
    InterpStatus::Success
}

/// `gmKeyframe::GetSubtitle`: a copy of the subtitle as a new native.
fn get_subtitle(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some(text) = keyframe(vm, this).map(|k| k.subtitle.clone()) else {
        return vm.bad_this(this);
    };
    let r = new_text(vm, &text);
    Call::new(ctx).ret(2, r, tag::REF);
    InterpStatus::Success
}

/// `gmKeyframe::SetSubtitle(si)`: the subtitle and its track bit.
fn set_subtitle(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (this, si) = (Call::new(ctx).this(), word(ctx, 2));
    let text = text_arg(vm, si);
    let Some(k) = keyframe_mut(vm, this) else {
        return vm.bad_this(this);
    };
    k.subtitle = text;
    k.defined_tracks |= TRACK_SUBTITLE;
    InterpStatus::Success
}

/// `gmKeyframe::GetDuration`: the double in two slots, high word in slot 2.
fn get_duration(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some(bits) = keyframe(vm, this).map(|k| k.duration.to_bits()) else {
        return vm.bad_this(this);
    };
    let mut call = Call::new(ctx);
    call.ret(3, bits as u32, tag::BOOL | tag::LONG);
    call.ret(2, (bits >> 32) as u32, tag::BOOL | tag::LONG);
    InterpStatus::Success
}

/// `gmKeyframe::SetDuration(double)`: the 64-bit argument, low word in slot 3.
fn set_duration(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (this, bits) = (Call::new(ctx).this(), word64(ctx, 3));
    let Some(k) = keyframe_mut(vm, this) else {
        return vm.bad_this(this);
    };
    k.duration = f64::from_bits(bits);
    InterpStatus::Success
}

/// A `gmSceneInfo` method that changes `this` and returns nothing.
fn scene_do(vm: &mut Vm, ctx: &mut Context, f: impl FnOnce(&mut SceneInfo)) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some(s) = scene_mut(vm, this) else {
        return vm.bad_this(this);
    };
    f(s);
    InterpStatus::Success
}

/// A `gmSceneInfo` getter: the word `f` reads, returned in slot 2.
fn get_scene(vm: &mut Vm, ctx: &mut Context, f: fn(&SceneInfo) -> u32) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some(v) = scene(vm, this).map(f) else {
        return vm.bad_this(this);
    };
    Call::new(ctx).ret(2, v, INT_RESULT);
    InterpStatus::Success
}

/// A `gmSceneInfo` setter taking one word.
fn set_scene(vm: &mut Vm, ctx: &mut Context, f: fn(&mut SceneInfo, u32)) -> InterpStatus {
    let (this, v) = (Call::new(ctx).this(), word(ctx, 2));
    let Some(s) = scene_mut(vm, this) else {
        return vm.bad_this(this);
    };
    f(s, v);
    InterpStatus::Success
}

/// `gmSceneInfo::SetIDByName(name)`: the id the enum map gives the name (0 when it has none).
fn set_id_by_name(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (this, name) = (Call::new(ctx).this(), word(ctx, 2));
    let name = vm.wstring(name).map(super::narrow).unwrap_or_default();
    let id = inq_enum_value(vm.data(), SCENE_ID_MAP, &name).unwrap_or(0);
    let Some(s) = scene_mut(vm, this) else {
        return vm.bad_this(this);
    };
    s.id = id;
    InterpStatus::Success
}

/// `gmSceneInfo::GetName`: a copy of the name as a new native.
fn get_name(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some(name) = scene(vm, this).map(|s| s.name.clone()) else {
        return vm.bad_this(this);
    };
    let r = new_text(vm, &name);
    Call::new(ctx).ret(2, r, tag::REF);
    InterpStatus::Success
}

/// `gmSceneInfo::SetName(si)`.
fn set_name(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (this, si) = (Call::new(ctx).this(), word(ctx, 2));
    let text = text_arg(vm, si);
    let Some(s) = scene_mut(vm, this) else {
        return vm.bad_this(this);
    };
    s.name = text;
    InterpStatus::Success
}

/// `gmSceneInfo::Set(model)`: this scene becomes a copy of `model`.
fn set_from_model(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (this, model) = (Call::new(ctx).this(), word(ctx, 2));
    let Some(copy) = scene(vm, model).cloned() else {
        return vm.bad_this(model);
    };
    let Some(s) = scene_mut(vm, this) else {
        return vm.bad_this(this);
    };
    *s = copy;
    InterpStatus::Success
}

/// `gmSceneInfo::AddKeyframe(frame)`: a copy of the keyframe, numbered after the scene's last
/// and stamped with the scene's id; returns 1.
fn add_keyframe(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let (this, frame) = (Call::new(ctx).this(), word(ctx, 2));
    let Some(mut k) = keyframe(vm, frame).cloned() else {
        return vm.bad_this(frame);
    };
    let Some(s) = scene_mut(vm, this) else {
        return vm.bad_this(this);
    };
    k.scene_id = s.id;
    k.frame_num = s.keyframes.len() as u32 + 1;
    s.keyframes.push(k);
    Call::new(ctx).ret(3, 1, INT_RESULT);
    InterpStatus::Success
}

/// Runs `f` on `this` as a scene list; `f` gives the result for `slot`.
fn with_list(
    vm: &mut Vm,
    ctx: &mut Context,
    slot: usize,
    f: impl FnOnce(&mut Vec<SceneInfo>) -> u32,
) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some(list) = scene_list_mut(vm, this) else {
        return vm.bad_this(this);
    };
    let v = f(list);
    Call::new(ctx).ret(slot, v, INT_RESULT);
    InterpStatus::Success
}

/// `ggmSceneInfoList::num_elements`.
fn num_elements(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    with_list(vm, ctx, 2, |l| l.len() as u32)
}

/// `ggmSceneInfoList::push_tail(scene)`: a copy at the end; returns 1.
fn push_tail(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let Some(s) = scene(vm, word(ctx, 2)).cloned() else {
        return vm.bad_this(word(ctx, 2));
    };
    with_list(vm, ctx, 3, |l| {
        l.push(s);
        1
    })
}

/// `ggmSceneInfoList::insert_sort(scene)`: before the first scene with a larger scene number;
/// returns 1.
fn insert_sort(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let Some(s) = scene(vm, word(ctx, 2)).cloned() else {
        return vm.bad_this(word(ctx, 2));
    };
    with_list(vm, ctx, 3, |l| {
        let at = l
            .iter()
            .position(|n| n.scene_num > s.scene_num)
            .unwrap_or(l.len());
        l.insert(at, s);
        1
    })
}

/// `ggmSceneInfoList::pop_front(out scene)`: the first scene copied out and removed; returns 0
/// on an empty list.
fn pop_front(vm: &mut Vm, ctx: &mut Context) -> InterpStatus {
    let out = word(ctx, 2);
    let mut head = None;
    let status = with_list(vm, ctx, 3, |l| {
        if l.is_empty() {
            return 0;
        }
        head = Some(l.remove(0));
        1
    });
    if let Some(head) = head {
        let target = vm.read_through(ctx, out).unwrap_or(u32::MAX);
        if let Some(s) = scene_mut(vm, target) {
            *s = head;
        }
    }
    status
}
