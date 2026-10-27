//! Game data natives (`gmCNativeFactory`): `gmRaceSexInfo` (0xA2), `gmSceneInfo` (0xA3) with
//! its `gmKeyframe`s, and `SkillUINode` (0xAE).

use crate::protocol::reader::{Reader, Writer, MAX_ITEMS};
use crate::protocol::string_info::StringInfo;
use crate::protocol::ProtocolError;

fn count(r: &mut Reader<'_>, item: usize) -> Result<usize, ProtocolError> {
    let n = r.u32()? as usize;
    if n > MAX_ITEMS || n.saturating_mul(item) > r.remaining() {
        return Err(ProtocolError::Invalid("count"));
    }
    Ok(n)
}

/// `gmRaceSexInfo::StreamPack`: three DIDs, then `_objInLocHash` (a `PHashTable` of word
/// pairs; the header keeps the bucket-size index).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GmRaceSexInfo {
    pub phys_obj_did: u32,
    pub apr_file_did: u32,
    pub music_file_did: u32,
    pub obj_in_loc_bucket_index: u8,
    pub obj_in_loc: Vec<(u32, u32)>,
}

impl GmRaceSexInfo {
    pub(super) fn read(r: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        let (phys_obj_did, apr_file_did, music_file_did) = (r.u32()?, r.u32()?, r.u32()?);
        let header = r.u32()?;
        let n = (header & 0x00FF_FFFF) as usize;
        if n > MAX_ITEMS || n * 8 > r.remaining() {
            return Err(ProtocolError::Invalid("count"));
        }
        Ok(Self {
            phys_obj_did,
            apr_file_did,
            music_file_did,
            obj_in_loc_bucket_index: (header >> 24) as u8,
            obj_in_loc: (0..n)
                .map(|_| Ok((r.u32()?, r.u32()?)))
                .collect::<Result<_, ProtocolError>>()?,
        })
    }

    pub(super) fn write(&self, w: &mut Writer) {
        w.u32(self.phys_obj_did)
            .u32(self.apr_file_did)
            .u32(self.music_file_did)
            .u32(
                self.obj_in_loc.len() as u32 & 0x00FF_FFFF
                    | u32::from(self.obj_in_loc_bucket_index) << 24,
            );
        for (k, v) in &self.obj_in_loc {
            w.u32(*k).u32(*v);
        }
    }
}

/// `gmKeyframe::StreamPack`.
#[derive(Debug, Clone, PartialEq)]
pub struct GmKeyframe {
    pub scene_id: u32,
    pub frame_num: u32,
    pub duration: f64,
    pub video_fragment: u32,
    pub audio_fragment: u32,
    pub voice_fragment: u32,
    pub text_fragment: StringInfo,
    pub movie_fragment_filename: String,
    pub defined_tracks: u32,
}

impl GmKeyframe {
    fn read(r: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        Ok(Self {
            scene_id: r.u32()?,
            frame_num: r.u32()?,
            duration: r.f64()?,
            video_fragment: r.u32()?,
            audio_fragment: r.u32()?,
            voice_fragment: r.u32()?,
            text_fragment: StringInfo::read(r)?,
            movie_fragment_filename: r.wide_string()?,
            defined_tracks: r.u32()?,
        })
    }

    fn write(&self, w: &mut Writer) {
        w.u32(self.scene_id)
            .u32(self.frame_num)
            .f64(self.duration)
            .u32(self.video_fragment)
            .u32(self.audio_fragment)
            .u32(self.voice_fragment);
        self.text_fragment.write(w);
        w.wide_string(&self.movie_fragment_filename)
            .u32(self.defined_tracks);
    }
}

/// `gmSceneInfo::StreamPack`: the scene, then its keyframes (`PList`: a count, then each).
#[derive(Debug, Clone, PartialEq)]
pub struct GmSceneInfo {
    pub id: u32,
    pub name: StringInfo,
    pub act_id: u32,
    pub scene_num: u32,
    pub is_hidden: u32,
    pub is_playable: u32,
    pub keyframes: Vec<GmKeyframe>,
}

impl GmSceneInfo {
    pub(super) fn read(r: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        let id = r.u32()?;
        let name = StringInfo::read(r)?;
        let (act_id, scene_num, is_hidden, is_playable) = (r.u32()?, r.u32()?, r.u32()?, r.u32()?);
        let n = count(r, 4)?;
        Ok(Self {
            id,
            name,
            act_id,
            scene_num,
            is_hidden,
            is_playable,
            keyframes: (0..n)
                .map(|_| GmKeyframe::read(r))
                .collect::<Result<_, _>>()?,
        })
    }

    /// `PList<gmSceneInfo>::UnPack`: a count, then each scene.
    pub(super) fn read_list(r: &mut Reader<'_>) -> Result<Vec<Self>, ProtocolError> {
        let n = count(r, 4)?;
        (0..n).map(|_| Self::read(r)).collect()
    }

    /// `PList<gmSceneInfo>::Pack`: a count, then each scene's `StreamPack`.
    pub(super) fn write_list(list: &[Self], w: &mut Writer) {
        w.u32(list.len() as u32);
        for scene in list {
            scene.write(w);
        }
    }

    pub(super) fn write(&self, w: &mut Writer) {
        w.u32(self.id);
        self.name.write(w);
        w.u32(self.act_id)
            .u32(self.scene_num)
            .u32(self.is_hidden)
            .u32(self.is_playable)
            .u32(self.keyframes.len() as u32);
        for frame in &self.keyframes {
            frame.write(w);
        }
    }
}

/// `SkillUINode::StreamPack` (Legions 0x005C8870): the skill, its texts, icon and place in the
/// skill tree, its parents and dependencies (`BaseList<unsigned long>`: a count, then each).
#[derive(Debug, Clone, PartialEq)]
pub struct SkillUiNode {
    pub skill_enum: u32,
    pub skill_type: u32,
    pub name: StringInfo,
    pub desc: StringInfo,
    pub icon: u32,
    pub x: i32,
    pub y: i32,
    pub parents: Vec<u32>,
    pub dependencies: Vec<u32>,
    pub is_passive: u32,
    pub untrainable: u32,
}

impl SkillUiNode {
    pub(super) fn read(r: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        let words = |r: &mut Reader<'_>| -> Result<Vec<u32>, ProtocolError> {
            let n = count(r, 4)?;
            (0..n).map(|_| r.u32()).collect()
        };
        Ok(Self {
            skill_enum: r.u32()?,
            skill_type: r.u32()?,
            name: StringInfo::read(r)?,
            desc: StringInfo::read(r)?,
            icon: r.u32()?,
            x: r.u32()? as i32,
            y: r.u32()? as i32,
            parents: words(r)?,
            dependencies: words(r)?,
            is_passive: r.u32()?,
            untrainable: r.u32()?,
        })
    }

    pub(super) fn write(&self, w: &mut Writer) {
        w.u32(self.skill_enum).u32(self.skill_type);
        self.name.write(w);
        self.desc.write(w);
        w.u32(self.icon).u32(self.x as u32).u32(self.y as u32);
        for list in [&self.parents, &self.dependencies] {
            w.u32(list.len() as u32);
            for v in list {
                w.u32(*v);
            }
        }
        w.u32(self.is_passive).u32(self.untrainable);
    }
}

/// `EffectUINode::StreamPack` (admin 0x00607F50): the effect's id, its flags (1 enabled, 2
/// toggled, 4 pulsed: `EffectUINode::IsPulsed`), name and description (`wpstring`s), icon, then
/// the duration and the time it was cast (doubles; `EffectUINode::Clear` leaves both -1).
#[derive(Debug, Clone, PartialEq)]
pub struct EffectUiNode {
    pub eid: u32,
    pub flags: u32,
    pub name: String,
    pub desc: String,
    pub icon: u32,
    pub duration: f64,
    pub time_cast: f64,
}

impl Default for EffectUiNode {
    /// `EffectUINode::Clear`.
    fn default() -> Self {
        Self {
            eid: 0,
            flags: 0,
            name: String::new(),
            desc: String::new(),
            icon: 0,
            duration: -1.0,
            time_cast: -1.0,
        }
    }
}

impl EffectUiNode {
    /// `EffectUINode::IsPulsed`.
    pub fn is_pulsed(&self) -> bool {
        self.flags & 4 != 0
    }

    pub(super) fn read(r: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        Ok(Self {
            eid: r.u32()?,
            flags: r.u32()?,
            name: r.wide_string()?,
            desc: r.wide_string()?,
            icon: r.u32()?,
            duration: r.f64()?,
            time_cast: r.f64()?,
        })
    }

    pub(super) fn write(&self, w: &mut Writer) {
        w.u32(self.eid).u32(self.flags);
        w.wide_string(&self.name).wide_string(&self.desc);
        w.u32(self.icon).f64(self.duration).f64(self.time_cast);
    }
}

/// `PList<EffectUINode>::Pack` (admin 0x00438EF0; Legions 0x00439740): the node count, then
/// each node's `StreamPack`. Retail writes nothing when the caller's `capacity` is smaller than
/// the list; either way it returns the size the list needs (`GetPackSize`, 0x00438F40).
/// The decompile has no direct caller (the list is reached through its vtable), so no crate
/// packs it yet.
#[allow(dead_code)]
pub(crate) fn pack_effect_list(nodes: &[EffectUiNode], out: &mut Writer, capacity: usize) -> usize {
    let mut needed = Writer::new();
    needed.u32(nodes.len() as u32);
    for node in nodes {
        node.write(&mut needed);
    }
    let size = needed.bytes.len();
    if size <= capacity {
        out.bytes.extend_from_slice(&needed.bytes);
    }
    size
}

/// `PList<EffectUINode>::UnPack` (admin 0x00438F80): a count, then that many nodes. A short
/// buffer or a node that fails to decode fails the whole list (retail returns 0 and keeps no
/// node it had not finished); bytes after the list stay unread for the caller.
#[allow(dead_code)]
pub(crate) fn read_effect_list(r: &mut Reader<'_>) -> Result<Vec<EffectUiNode>, ProtocolError> {
    let n = r.count()?;
    (0..n).map(|_| EffectUiNode::read(r)).collect()
}

#[cfg(test)]
mod effect_list_tests {
    use super::*;

    fn node(eid: u32, name: &str, duration: f64) -> EffectUiNode {
        EffectUiNode {
            eid,
            flags: 1 | 4,
            name: name.to_string(),
            desc: String::new(),
            icon: 0x0600_1234,
            duration,
            time_cast: 10.0,
        }
    }

    #[test]
    fn the_list_packs_its_count_then_each_node_and_unpacks_back() {
        let nodes = vec![node(0x11, "Haste", 30.0), node(0x22, "", -1.0)];
        let mut out = Writer::new();
        let size = pack_effect_list(&nodes, &mut out, usize::MAX);
        assert_eq!(size, out.bytes.len());
        assert_eq!(&out.bytes[..4], &2u32.to_le_bytes());
        // The first node: eid, flags, the name as a wide string, an empty description.
        assert_eq!(&out.bytes[4..8], &0x11u32.to_le_bytes());
        assert_eq!(&out.bytes[8..12], &5u32.to_le_bytes());
        let mut r = Reader::new(&out.bytes);
        assert_eq!(read_effect_list(&mut r).unwrap(), nodes);
        r.finish().unwrap();
    }

    #[test]
    fn an_empty_list_is_one_zero_word() {
        let mut out = Writer::new();
        assert_eq!(pack_effect_list(&[], &mut out, 4), 4);
        assert_eq!(out.bytes, vec![0, 0, 0, 0]);
        let mut r = Reader::new(&out.bytes);
        assert!(read_effect_list(&mut r).unwrap().is_empty());
    }

    #[test]
    fn a_list_that_does_not_fit_is_not_written_but_its_size_is_reported() {
        let nodes = vec![node(1, "x", 1.0)];
        let mut out = Writer::new();
        let needed = pack_effect_list(&nodes, &mut out, 3);
        assert!(out.bytes.is_empty());
        let mut full = Writer::new();
        assert_eq!(pack_effect_list(&nodes, &mut full, needed), needed);
        assert_eq!(full.bytes.len(), needed);
    }

    #[test]
    fn a_truncated_list_fails_and_a_short_count_leaves_the_tail() {
        let nodes = vec![node(1, "x", 1.0), node(2, "y", 2.0)];
        let mut out = Writer::new();
        pack_effect_list(&nodes, &mut out, usize::MAX);
        for end in 0..out.bytes.len() {
            let mut r = Reader::new(&out.bytes[..end]);
            assert!(read_effect_list(&mut r).is_err(), "prefix {end}");
        }
        // One node counted: the second node's bytes are left over.
        let mut one = out.bytes.clone();
        one[..4].copy_from_slice(&1u32.to_le_bytes());
        let mut r = Reader::new(&one);
        assert_eq!(read_effect_list(&mut r).unwrap(), nodes[..1].to_vec());
        assert!(r.finish().is_err());
    }
}

#[cfg(test)]
mod tests {
    use super::super::NativeObject;
    use super::*;

    fn round_trip(object: NativeObject) -> Vec<u8> {
        let mut w = Writer::new();
        object.write(&mut w);
        let mut r = Reader::new(&w.bytes);
        assert_eq!(
            NativeObject::read(object.native_type(), &mut r).unwrap(),
            object
        );
        r.finish().unwrap();
        w.bytes
    }

    fn words(w: &[u32]) -> Vec<u8> {
        w.iter().flat_map(|w| w.to_le_bytes()).collect()
    }

    #[test]
    fn a_new_effect_node_is_cleared_and_a_copy_keeps_every_field() {
        // `EffectUINode::EffectUINode()`: null strings, then `Clear`: id, flags and icon 0, both
        // times -1.0 (the high words 0xBFF00000).
        let node = EffectUiNode::default();
        assert_eq!(
            (node.eid, node.flags, node.icon, &node.name, &node.desc),
            (0, 0, 0, &String::new(), &String::new())
        );
        assert_eq!(node.duration.to_bits() >> 32, 0xBFF0_0000);
        assert_eq!(node.time_cast.to_bits() >> 32, 0xBFF0_0000);
        assert!(!node.is_pulsed());
        // `EffectUINode::EffectUINode(const EffectUINode &)` copies by `operator=`.
        let source = EffectUiNode {
            eid: 7,
            flags: 5,
            name: "Name".into(),
            desc: "Description".into(),
            icon: 0x0600_0042,
            duration: 30.0,
            time_cast: 100.5,
        };
        let copy = source.clone();
        assert_eq!(copy, source);
        assert!(copy.is_pulsed());
    }

    #[test]
    fn skill_nodes_and_race_info_keep_their_field_order() {
        let node = SkillUiNode {
            skill_enum: 0x41,
            skill_type: 2,
            name: StringInfo::default(),
            desc: StringInfo::default(),
            icon: 0x0600_0001,
            x: -1,
            y: 3,
            parents: vec![0x40],
            dependencies: vec![],
            is_passive: 1,
            untrainable: 0,
        };
        let mut expected = words(&[0x41, 2, 0, 0, 0, 0, 0, 0, 0x0600_0001, u32::MAX, 3]);
        expected.extend(words(&[1, 0x40, 0, 1, 0]));
        assert_eq!(
            round_trip(NativeObject::SkillUiNode(Box::new(node))),
            expected
        );
        let info = GmRaceSexInfo {
            phys_obj_did: 1,
            apr_file_did: 2,
            music_file_did: 3,
            obj_in_loc_bucket_index: 2,
            obj_in_loc: vec![(4, 5)],
        };
        assert_eq!(
            round_trip(NativeObject::GmRaceSexInfo(info)),
            words(&[1, 2, 3, 0x0200_0001, 4, 5])
        );
        let scene = GmSceneInfo {
            id: 1,
            name: StringInfo::default(),
            act_id: 2,
            scene_num: 3,
            is_hidden: 0,
            is_playable: 1,
            keyframes: vec![GmKeyframe {
                scene_id: 1,
                frame_num: 0,
                duration: 2.5,
                video_fragment: 0,
                audio_fragment: 0,
                voice_fragment: 0,
                text_fragment: StringInfo::default(),
                movie_fragment_filename: "a.bik".into(),
                defined_tracks: 7,
            }],
        };
        round_trip(NativeObject::GmSceneInfo(Box::new(scene)));
    }

    #[test]
    fn effect_nodes_pack_eid_flags_texts_icon_and_times() {
        let node = EffectUiNode {
            eid: 7,
            flags: 5,
            icon: 0x0600_0002,
            duration: 30.0,
            ..EffectUiNode::default()
        };
        assert!(node.is_pulsed());
        let mut expected = words(&[7, 5]);
        let mut w = Writer::new();
        w.wide_string("").wide_string("");
        expected.extend(w.bytes);
        expected.extend(words(&[0x0600_0002]));
        expected.extend(30.0f64.to_le_bytes());
        expected.extend((-1.0f64).to_le_bytes());
        assert_eq!(
            round_trip(NativeObject::EffectUiNode(Box::new(node))),
            expected
        );
    }
}
