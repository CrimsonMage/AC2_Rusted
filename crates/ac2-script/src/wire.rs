//! The boundary between the VM's run-time objects and ac2-net's codecs (`PackedReference`,
//! `NativeObject`): every packed graph the VM reads (WStates, `PackBuffer::UnPackBuffer`) and
//! writes (`PackBuffer::PackPackage`) goes through here.
//!
//! The glue's natives convert both ways: containers into the bucket models
//! (`PHashTable::UnPack` order, `BaseMultiHash::UnPack` inserts), `StringInfo` into its
//! semantic variables, `wpstring` into UTF-16, `MissileParameters` into named fields,
//! `BehaviorParams` from retail's defaults. Every other native stays in its wire form
//! ([`NativeObject::Wire`]) and goes back out unchanged.

use ac2_net::protocol as net;

use crate::native::{
    behavior_defaults, behavior_pack_bitfield, bucket_size_index, ntype, GameplayOptions,
    MissileParameters, NativeObject, RandomSelectionTable, RefKey, ScriptHash, ScriptList,
    ScriptMultiHash, Shortcut, StringInfo, StringVar, StringVarValue,
};
use crate::reference::{HeapObject, PackedEntry, PackedReference};

/// A VM object the wire cannot carry yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Unsupported(pub u32);

/// A packed reference graph from the wire.
pub fn from_net(packed: &net::PackedReference) -> PackedReference {
    let entries = packed
        .entries
        .iter()
        .map(|e| match e {
            net::PackedEntry::Heap { flags, object } => PackedEntry::Heap {
                flags: *flags,
                object: HeapObject {
                    pkg_state_id: object.pkg_state_id,
                    pkg_id: object.pkg_id,
                    native_type: object.native.as_ref().map_or(0, |n| n.native_type()),
                    native: object.native.as_ref().map(native_from_net),
                    slots: object.slots.clone(),
                    tags: object.tags.clone(),
                },
            },
            net::PackedEntry::Relative { flags } => PackedEntry::Relative { flags: *flags },
            net::PackedEntry::Singleton { flags, did } => PackedEntry::Package {
                flags: *flags,
                did: *did,
            },
            net::PackedEntry::Value { flags, value } => PackedEntry::Value {
                flags: *flags,
                value: *value,
            },
        })
        .collect();
    PackedReference {
        ids: packed.ids.clone(),
        entries,
    }
}

/// A packed reference graph for the wire.
pub fn to_net(packed: &PackedReference) -> Result<net::PackedReference, Unsupported> {
    let entries = packed
        .entries
        .iter()
        .map(|e| {
            Ok(match e {
                PackedEntry::Heap { flags, object } => net::PackedEntry::Heap {
                    flags: *flags,
                    object: net::HeapObject {
                        pkg_state_id: object.pkg_state_id,
                        pkg_id: object.pkg_id,
                        native: match &object.native {
                            Some(n) => Some(native_to_net(object.native_type, n)?),
                            None => None,
                        },
                        slots: object.slots.clone(),
                        tags: object.tags.clone(),
                    },
                },
                PackedEntry::Relative { flags } => net::PackedEntry::Relative { flags: *flags },
                PackedEntry::Package { flags, did } => net::PackedEntry::Singleton {
                    flags: *flags,
                    did: *did,
                },
                PackedEntry::Value { flags, value } => net::PackedEntry::Value {
                    flags: *flags,
                    value: *value,
                },
            })
        })
        .collect::<Result<Vec<_>, Unsupported>>()?;
    Ok(net::PackedReference {
        ids: packed.ids.clone(),
        entries,
    })
}

/// A wire native as the VM holds it.
pub fn native_from_net(native: &net::NativeObject) -> NativeObject {
    use net::NativeObject as W;
    match native {
        W::List(list) => NativeObject::List(ScriptList {
            wide: matches!(list.native_type, ntype::L_LIST | ntype::L_ARRAY),
            items: list.items.clone(),
        }),
        W::Hash(hash) if hash.native_type == ntype::AA_MULTI_HASH => {
            let mut multi = ScriptMultiHash::default();
            for (k, v) in hash.keys.iter().zip(&hash.values) {
                if let RefKey::Value(k) = k {
                    multi.add(*k, *v);
                }
            }
            NativeObject::MultiHash(multi)
        }
        W::Hash(hash) => {
            let set = hash.values.is_empty() && !hash.keys.is_empty()
                || matches!(hash.native_type, ntype::A_HASH_SET | ntype::LA_HASH_SET);
            let items = hash
                .keys
                .iter()
                .enumerate()
                .map(|(i, k)| {
                    (
                        *k,
                        if set {
                            0
                        } else {
                            hash.values.get(i).copied().unwrap_or(0)
                        },
                    )
                })
                .collect();
            NativeObject::Hash(ScriptHash::from_packed(hash.bucket_index, items))
        }
        W::StringInfo(info) => NativeObject::StringInfo(string_info_from_net(info)),
        W::WString(s) => NativeObject::WString(s.encode_utf16().collect()),
        W::Vector(v) => NativeObject::Vector(*v),
        W::Position(p) => NativeObject::Position(*p),
        W::MissileParameters(m) => {
            NativeObject::MissileParameters(MissileParameters::from_packed(m.bitfield, &m.words))
        }
        W::BehaviorParams(p) => NativeObject::BehaviorParams(Box::new(behavior_from_net(p))),
        // The adaptors pack nothing (`gmCNativeFactory`); a fresh one each.
        W::Empty(t) if !matches!(NativeObject::new_empty(*t), NativeObject::Unbound(_)) => {
            NativeObject::new_empty(*t)
        }
        W::ShortcutInfo(s) => NativeObject::Shortcut(Box::new(Shortcut::from_net(s))),
        W::GameplayOptionsProfile(p) => {
            NativeObject::GameplayOptions(Box::new(GameplayOptions::from_net(p)))
        }
        W::RandomSelectionTable(t) => NativeObject::RandomSelectionTable(RandomSelectionTable {
            unused: t.unused.clone(),
            used: t.used.clone(),
        }),
        other => NativeObject::Wire(Box::new(other.clone())),
    }
}

/// A VM native for the wire.
pub fn native_to_net(
    native_type: u32,
    native: &NativeObject,
) -> Result<net::NativeObject, Unsupported> {
    use net::NativeObject as W;
    Ok(match native {
        NativeObject::List(list) => W::List(net::ScriptList {
            native_type,
            items: list.items.clone(),
        }),
        NativeObject::Hash(hash) => {
            let set = matches!(native_type, ntype::A_HASH_SET | ntype::LA_HASH_SET);
            W::Hash(net::ScriptHash {
                native_type,
                bucket_index: bucket_size_index(hash.num_buckets()) as u8,
                keys: hash.nodes().map(|n| n.key).collect(),
                values: if set {
                    Vec::new()
                } else {
                    hash.nodes().map(|n| n.value).collect()
                },
            })
        }
        NativeObject::MultiHash(multi) if native_type == ntype::AA_MULTI_HASH => {
            W::Hash(net::ScriptHash {
                native_type,
                bucket_index: 0,
                keys: multi.nodes().map(|n| RefKey::Value(n.key)).collect(),
                values: multi.nodes().map(|n| n.value).collect(),
            })
        }
        NativeObject::StringInfo(info) => W::StringInfo(string_info_to_net(info)),
        NativeObject::WString(s) => W::WString(String::from_utf16_lossy(s)),
        NativeObject::Vector(v) => W::Vector(*v),
        NativeObject::Position(p) => W::Position(*p),
        NativeObject::MissileParameters(m) => W::MissileParameters(net::MissileParameters {
            bitfield: m.bitfield,
            words: m.packed_words(),
        }),
        NativeObject::BehaviorParams(p) => {
            let mut p = (**p).clone();
            p.bitfield = behavior_pack_bitfield(&p);
            W::BehaviorParams(Box::new(p))
        }
        NativeObject::GameplayOptions(p) => W::GameplayOptionsProfile(Box::new(p.to_net())),
        NativeObject::Shortcut(s) => W::ShortcutInfo(s.to_net()),
        NativeObject::Fellow(_)
        | NativeObject::Fellowship(_)
        | NativeObject::AllegianceData(_)
        | NativeObject::AllegianceProfile(_) => W::Empty(native_type),
        NativeObject::RandomSelectionTable(t) => {
            W::RandomSelectionTable(net::RandomSelectionTable {
                unused: t.unused.clone(),
                used: t.used.clone(),
            })
        }
        NativeObject::SceneInfo(scene) => W::GmSceneInfo(Box::new(scene_to_net(scene))),
        NativeObject::SceneList(scenes) => {
            W::GmSceneInfoList(scenes.iter().map(scene_to_net).collect())
        }
        NativeObject::Wire(w) => (**w).clone(),
        _ => return Err(Unsupported(native_type)),
    })
}

/// `BehaviorParams` as the VM holds it: `SetDefaults`, then the fields the bitfield names.
fn behavior_from_net(p: &net::BehaviorParams) -> net::BehaviorParams {
    let mut out = behavior_defaults();
    let has = |bit: u32| p.bitfield & bit != 0;
    out.bitfield = p.bitfield;
    if has(0x1) {
        out.behavior = p.behavior;
    }
    if has(0x4) {
        out.time_scale = p.time_scale;
    }
    if has(0x2) {
        out.mode = p.mode;
    }
    if has(0x10) {
        out.hold_cycles = p.hold_cycles;
    }
    if has(0x1000) {
        out.fx_script = p.fx_script;
    }
    if has(0x10_0000) {
        out.fx = p.fx;
    }
    if has(0x2000) {
        out.target = p.target;
    }
    if has(0x20_0000) {
        out.camera_target = p.camera_target;
        out.camera_behavior = p.camera_behavior;
    }
    if has(0x1_0000) {
        out.vdesc_to_clone = p.vdesc_to_clone;
        out.cloned_apr = p.cloned_apr;
        out.cloned_app_apr = p.cloned_app_apr.clone();
        out.cloned_app_apr_buckets = p.cloned_app_apr_buckets;
    }
    if has(0x4000) {
        out.impulse = p.impulse;
    }
    for (bit, from, to) in [
        (0x8, p.early_callback, &mut out.early_callback),
        (0x20, p.moveto_cancels, &mut out.moveto_cancels),
        (0x80, p.camera_parent, &mut out.camera_parent),
        (0x40, p.camera_target_flag, &mut out.camera_target_flag),
        (0x4_0000, p.camera_hold, &mut out.camera_hold),
        (0x8_0000, p.camera_restore, &mut out.camera_restore),
        (0x100, p.fade_children, &mut out.fade_children),
        (0x400, p.propagate, &mut out.propagate),
        (0x800, p.lock_actions, &mut out.lock_actions),
        (0x200, p.send_event, &mut out.send_event),
        (0x2_0000, p.leave_idle_alone, &mut out.leave_idle_alone),
        (
            0x8000,
            p.destroy_on_completion,
            &mut out.destroy_on_completion,
        ),
        (0x80_0000, p.weenie_emote, &mut out.weenie_emote),
        (0x40_0000, p.context, &mut out.context),
    ] {
        if has(bit) {
            *to = from;
        }
    }
    out
}

/// `StringInfo` from the wire layout (`StringInfo::UnPack`: an unknown variable type carries no
/// value and is dropped).
pub fn string_info_from_net(info: &net::StringInfo) -> StringInfo {
    use net::StringVariable as W;
    use StringVarValue as V;
    let variables = info
        .variables
        .iter()
        .filter_map(|(id, var)| {
            let (kind, value) = match var {
                W::Word { kind: 2, value } => (2, V::Int(*value as i32)),
                W::Word { kind, value } => (*kind, V::UInt(*value)),
                W::WordsShort { kind: 3, a, b, c } => (
                    3,
                    V::FormattedInt {
                        value: *a as i32,
                        grouping: *b as i32,
                        base: *c,
                    },
                ),
                W::WordsShort { kind, a, b, c } => (
                    *kind,
                    V::FormattedUInt {
                        value: *a,
                        grouping: *b as i32,
                        base: *c,
                    },
                ),
                W::LongShort { value, c } => (
                    4,
                    V::Float {
                        value: f64::from_bits(*value),
                        precision: *c,
                    },
                ),
                W::Long5 { value, c, d, e } => (
                    5,
                    V::FormattedFloat {
                        value: f64::from_bits(*value),
                        precision: *c,
                        grouping: *d as i32,
                        base: *e,
                    },
                ),
                W::Nested(inner) => (9, V::StringInfo(Box::new(string_info_from_net(inner)))),
                W::Long { kind: 10, value } => (10, V::LInt(*value as i64)),
                W::Long { kind, value } => (*kind, V::ULInt(*value)),
                W::LongWordShort {
                    kind: 11,
                    value,
                    d,
                    e,
                } => (
                    11,
                    V::FormattedLInt {
                        value: *value as i64,
                        grouping: *d as i32,
                        base: *e,
                    },
                ),
                W::LongWordShort { kind, value, d, e } => (
                    *kind,
                    V::FormattedULInt {
                        value: *value,
                        grouping: *d as i32,
                        base: *e,
                    },
                ),
                W::Empty { .. } => return None,
            };
            Some(StringVar {
                id: *id,
                kind,
                value,
            })
        })
        .collect();
    StringInfo {
        string_id: info.string_id,
        table_id: info.table_id,
        variables,
        literal: info.literal.as_ref().map(|s| s.encode_utf16().collect()),
    }
}

/// `StringInfo` in the wire layout (`StringInfo::Pack`; a `SID_String` variable packs no value).
pub fn string_info_to_net(info: &StringInfo) -> net::StringInfo {
    use net::StringVariable as W;
    use StringVarValue as V;
    let variables = info
        .variables
        .iter()
        .map(|v| {
            let w = match &v.value {
                V::Int(x) => W::Word {
                    kind: v.kind,
                    value: *x as u32,
                },
                V::UInt(x) => W::Word {
                    kind: v.kind,
                    value: *x,
                },
                V::FormattedInt {
                    value,
                    grouping,
                    base,
                } => W::WordsShort {
                    kind: v.kind,
                    a: *value as u32,
                    b: *grouping as u32,
                    c: *base,
                },
                V::FormattedUInt {
                    value,
                    grouping,
                    base,
                } => W::WordsShort {
                    kind: v.kind,
                    a: *value,
                    b: *grouping as u32,
                    c: *base,
                },
                V::Float { value, precision } => W::LongShort {
                    value: value.to_bits(),
                    c: *precision,
                },
                V::FormattedFloat {
                    value,
                    precision,
                    grouping,
                    base,
                } => W::Long5 {
                    value: value.to_bits(),
                    c: *precision,
                    d: *grouping as u32,
                    e: *base,
                },
                V::StringInfo(inner) => W::Nested(Box::new(string_info_to_net(inner))),
                V::LInt(x) => W::Long {
                    kind: v.kind,
                    value: *x as u64,
                },
                V::ULInt(x) => W::Long {
                    kind: v.kind,
                    value: *x,
                },
                V::FormattedLInt {
                    value,
                    grouping,
                    base,
                } => W::LongWordShort {
                    kind: v.kind,
                    value: *value as u64,
                    d: *grouping as u32,
                    e: *base,
                },
                V::FormattedULInt {
                    value,
                    grouping,
                    base,
                } => W::LongWordShort {
                    kind: v.kind,
                    value: *value,
                    d: *grouping as u32,
                    e: *base,
                },
                V::String(_) | V::Skipped => W::Empty { kind: v.kind },
            };
            (v.id, w)
        })
        .collect();
    net::StringInfo {
        string_id: info.string_id,
        table_id: info.table_id,
        literal: info.literal.as_ref().map(|s| String::from_utf16_lossy(s)),
        variables,
    }
}

/// `gmSceneInfo::StreamPack` of a scene: its keyframes' `gmKeyframe::StreamPack`. The movie
/// track's file name is not set by the scripts this client runs (`SetMovieTrack` is unbound), so
/// it stays empty.
/// A `gmSceneInfo` from its `StreamPack` form: the inverse of [`scene_to_net`]. The movie
/// track's file name is not kept (the scripts do not set one).
pub fn scene_from_net(scene: &net::GmSceneInfo) -> ac2_weenie::SceneInfo {
    ac2_weenie::SceneInfo {
        id: scene.id,
        name: scene.name.clone(),
        act_id: scene.act_id,
        scene_num: scene.scene_num,
        hidden: scene.is_hidden != 0,
        playable: scene.is_playable != 0,
        keyframes: scene
            .keyframes
            .iter()
            .map(|k| ac2_weenie::Keyframe {
                scene_id: k.scene_id,
                frame_num: k.frame_num,
                duration: k.duration,
                video: k.video_fragment,
                audio: k.audio_fragment,
                voice: k.voice_fragment,
                subtitle: k.text_fragment.clone(),
                defined_tracks: k.defined_tracks,
            })
            .collect(),
    }
}

pub(crate) fn scene_to_net(scene: &ac2_weenie::SceneInfo) -> net::GmSceneInfo {
    net::GmSceneInfo {
        id: scene.id,
        name: scene.name.clone(),
        act_id: scene.act_id,
        scene_num: scene.scene_num,
        is_hidden: u32::from(scene.hidden),
        is_playable: u32::from(scene.playable),
        keyframes: scene
            .keyframes
            .iter()
            .map(|k| net::GmKeyframe {
                scene_id: k.scene_id,
                frame_num: k.frame_num,
                duration: k.duration,
                video_fragment: k.video,
                audio_fragment: k.audio,
                voice_fragment: k.voice,
                text_fragment: k.subtitle.clone(),
                movie_fragment_filename: String::new(),
                defined_tracks: k.defined_tracks,
            })
            .collect(),
    }
}
