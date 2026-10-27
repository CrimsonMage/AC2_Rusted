//! `gmActInfo` (0x9C) and `List<gmActInfo>` (0x9D): an act of the story quests, with its scenes
//! (`gmActInfo::StreamPack`), the list the player's script fills for the Quest panel's act list
//! (`gmCWInterface::InqActInfoList`).

use super::gm::GmSceneInfo;
use crate::protocol::reader::{Reader, Writer};
use crate::protocol::string_info::StringInfo;
use crate::protocol::ProtocolError;

/// `gmActInfo`: the act's id and name, its number (the order `insert_sort` keeps), whether it is
/// visible and playable, and its scenes in `_uiSceneNum` order.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GmActInfo {
    /// `_actID`.
    pub id: u32,
    /// `_siActName`.
    pub name: StringInfo,
    /// `_uiActNum`.
    pub act_num: u32,
    /// `_bVisible`.
    pub visible: u32,
    /// `_bPlayable`.
    pub playable: u32,
    /// `_sceneList`.
    pub scenes: Vec<GmSceneInfo>,
}

impl GmActInfo {
    /// `gmActInfo::StreamPack` read: the id, name, number, the two flags, then the scene list
    /// (`PList<gmSceneInfo>`: a count, then each scene).
    pub(super) fn read(r: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        let id = r.u32()?;
        let name = StringInfo::read(r)?;
        let (act_num, visible, playable) = (r.u32()?, r.u32()?, r.u32()?);
        let scenes = GmSceneInfo::read_list(r)?;
        Ok(Self {
            id,
            name,
            act_num,
            visible,
            playable,
            scenes,
        })
    }

    /// `List<gmActInfo>::UnPack`: a count, then each act.
    pub(super) fn read_list(r: &mut Reader<'_>) -> Result<Vec<Self>, ProtocolError> {
        let n = r.u32()? as usize;
        if n > crate::protocol::reader::MAX_ITEMS || n.saturating_mul(4) > r.remaining() {
            return Err(ProtocolError::Invalid("count"));
        }
        (0..n).map(|_| Self::read(r)).collect()
    }

    /// `gmActInfo::StreamPack` write, the mirror of [`GmActInfo::read`].
    pub(super) fn write(&self, w: &mut Writer) {
        w.u32(self.id);
        self.name.write(w);
        w.u32(self.act_num).u32(self.visible).u32(self.playable);
        GmSceneInfo::write_list(&self.scenes, w);
    }

    /// `List<gmActInfo>::Pack`: a count, then each act.
    pub(super) fn write_list(list: &[Self], w: &mut Writer) {
        w.u32(list.len() as u32);
        for act in list {
            act.write(w);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::native::NativeObject;

    fn scene(id: u32, num: u32) -> GmSceneInfo {
        GmSceneInfo {
            id,
            name: StringInfo::default(),
            act_id: 7,
            scene_num: num,
            is_hidden: 0,
            is_playable: 1,
            keyframes: Vec::new(),
        }
    }

    #[test]
    fn an_act_and_its_scene_list_round_trip() {
        let act = GmActInfo {
            id: 7,
            name: StringInfo::default(),
            act_num: 2,
            visible: 1,
            playable: 1,
            scenes: vec![scene(70, 1), scene(71, 2)],
        };
        for object in [
            NativeObject::GmActInfo(Box::new(act.clone())),
            NativeObject::GmActInfoList(vec![act.clone(), GmActInfo::default()]),
        ] {
            let mut w = Writer::new();
            object.write(&mut w);
            let mut r = Reader::new(&w.bytes);
            assert_eq!(
                NativeObject::read(object.native_type(), &mut r).unwrap(),
                object
            );
            r.finish().unwrap();
        }
    }
}
