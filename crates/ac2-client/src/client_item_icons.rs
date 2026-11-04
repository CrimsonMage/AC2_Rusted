//! `VisualDesc::GetTotalIconDesc` for the item slots (`ItemDisplay::Update`, which asks it of the
//! object's local description, `CWPhysicsObject::GetLocalVisualDesc`): the visual's DAT chain's
//! icon layers, the local description's layers over them, then the icon atoms of the local
//! description's global appearance keys (`PartGroupDataDesc::AugmentAtomCollection` of
//! `m_global_app_mods`, `IconDesc::ApplyAdditiveIconLayer` of each). The chain and the atoms read
//! the DATs, so a loader thread reads them (`Request::TotalIcon`); the local layers are merged as
//! the slot is drawn. The local description is the drawn one: the server's with the appearance
//! FX's keys on it, as `CPhysicsObj::DoAppearanceFX` puts them on the local description.

use std::sync::Arc;

use ac2_physics::Appearance;

use crate::client::Client;
use crate::loader::Request;

/// One icon layer: layer id, image, shift colour.
type Layer = (u32, u32, u32);

/// What the loader read for an object's icon, and the global keys it was read for.
#[derive(Debug, Default)]
pub(crate) struct ItemIcon {
    /// The global appearance tables and keys (not their scalars) the atoms were asked for.
    asked: Vec<(u32, Vec<u32>)>,
    /// The DAT chain's layers (without atoms).
    chain: Vec<Layer>,
    /// The atoms of the asked keys, in `AugmentAtomCollection` order.
    atoms: Vec<Layer>,
}

/// The tables and key ids of a description's global appearance.
fn global_keys(appearance: &Appearance) -> Vec<(u32, Vec<u32>)> {
    appearance
        .global
        .appearance
        .iter()
        .map(|(table, keys)| (*table, keys.iter().map(|(key, _)| *key).collect()))
        .collect()
}

impl Client {
    /// The object's local description as `GetLocalVisualDesc` holds it (the appearance FX on it).
    fn icon_local(&self, id: u64) -> Option<(u32, Arc<Appearance>)> {
        let object = self.session.objects().get(id)?;
        let local = self.drawn_appearance(id, object.appearance.clone());
        Some((object.appearance.parent, local))
    }

    /// Asks the loader for object `id`'s chain layers and global icon atoms.
    pub(crate) fn request_item_icon(&mut self, id: u64) {
        let Some((visual, local)) = self.icon_local(id) else {
            return;
        };
        if visual == 0 {
            return;
        }
        self.item_icons.entry(id).or_default().asked = global_keys(&local);
        self.loader.request(Request::TotalIcon(
            id,
            visual,
            local.global.appearance.clone(),
        ));
    }

    /// Asks again for each object whose global appearance keys changed since its icon was asked
    /// for (an appearance FX came or went), so its atoms follow.
    pub(crate) fn refresh_item_icons(&mut self) {
        let changed: Vec<u64> = self
            .item_icons
            .iter()
            .filter(|(id, icon)| {
                self.icon_local(**id)
                    .is_some_and(|(_, local)| global_keys(&local) != icon.asked)
            })
            .map(|(id, _)| *id)
            .collect();
        for id in changed {
            self.request_item_icon(id);
        }
    }

    /// The loader's answer: kept when it is for the keys asked last; the slots redraw the item
    /// (`RecvNotice_ItemUpdate`) when what they draw changed.
    pub(crate) fn item_icon_loaded(
        &mut self,
        id: u64,
        global: &[(u32, Vec<(u32, f32)>)],
        chain: Vec<Layer>,
        atoms: Vec<Layer>,
    ) {
        let asked: Vec<(u32, Vec<u32>)> = global
            .iter()
            .map(|(table, keys)| (*table, keys.iter().map(|(key, _)| *key).collect()))
            .collect();
        let Some(icon) = self.item_icons.get_mut(&id) else {
            return;
        };
        if icon.asked != asked || (icon.chain == chain && icon.atoms == atoms) {
            return;
        }
        icon.chain = chain;
        icon.atoms = atoms;
        self.script_outbox.push(ac2_weenie::SessionEvent::Notice(
            ac2_weenie::GameplayNotice::ItemUpdate(id),
        ));
    }

    /// `VisualDesc::GetTotalIconDesc` of object `id`'s local description: the chain's layers,
    /// the local layers on top, then the global icon atoms (empty until the loader read them).
    pub(crate) fn item_icon_layers(&self, id: u64) -> Vec<Layer> {
        let Some((_, local)) = self.icon_local(id) else {
            return Vec::new();
        };
        let icon = self.item_icons.get(&id);
        let chain = Appearance {
            icon: icon.map(|i| i.chain.clone()).unwrap_or_default(),
            ..Appearance::default()
        };
        let merged = Appearance {
            icon: local.total_icon_layers(Some(&chain)),
            ..Appearance::default()
        };
        let atoms = Appearance {
            icon: icon.map(|i| i.atoms.clone()).unwrap_or_default(),
            ..Appearance::default()
        };
        atoms.total_icon_layers(Some(&merged))
    }
}
