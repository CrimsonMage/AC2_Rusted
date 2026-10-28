//! The gameplay HUD's panel state, owned by [`Hud`]: the world overlays `gm3DTextUI` keeps (the
//! targeting reticle, the name plates and the floating damage texts) and the clock they run on.
//! The client owns the element tree and the GPU surfaces; this state takes the tree as an argument
//! and reads no client type, so the overlays' logic runs on plain frame data.

use ac2_net::protocol::SelectionInfo;

use crate::damage_text::DamageTexts;
use crate::element::UiTree;
use crate::name_plates::NamePlates;
use crate::reticle::{Reticle, ReticleOutcome, ReticleTarget};

/// The HUD's panel state (`gmGamePlayUI`'s children that the client drives each frame).
#[derive(Debug, Clone, Default)]
pub struct Hud {
    reticle: Option<Reticle>,
    names: Option<NamePlates>,
    damage: Option<DamageTexts>,
    /// The HUD's clock (`Timer::cur_time`) at the last frame: a damage text starts at it.
    now: f64,
    /// The selection info the reticle's meters last took.
    reticle_info: Option<SelectionInfo>,
}

impl Hud {
    /// The overlays the HUD's build made (`gm3DTextUI`'s reticle, name plates and damage texts).
    pub fn new(
        reticle: Option<Reticle>,
        names: Option<NamePlates>,
        damage: Option<DamageTexts>,
    ) -> Self {
        Self {
            reticle,
            names,
            damage,
            ..Self::default()
        }
    }

    /// The HUD's clock for this frame (`Timer::cur_time`, seconds).
    pub fn set_now(&mut self, now: f64) {
        self.now = now;
    }

    /// The HUD's clock (`Timer::cur_time`, seconds) as the last frame set it.
    pub fn now(&self) -> f64 {
        self.now
    }

    /// The name plates' slot, for the passes that take it (`MessagePanels::names`).
    pub fn names_slot(&mut self) -> &mut Option<NamePlates> {
        &mut self.names
    }

    /// The damage texts' slot, for the notice pass (`NoticePanels::damage`).
    pub fn damage_slot(&mut self) -> &mut Option<DamageTexts> {
        &mut self.damage
    }

    /// The targeting reticle, while the HUD has one.
    pub fn reticle_mut(&mut self) -> Option<&mut Reticle> {
        self.reticle.as_mut()
    }

    /// The name plates, while the HUD has them.
    pub fn names_mut(&mut self) -> Option<&mut NamePlates> {
        self.names.as_mut()
    }

    /// The floating damage texts, while the HUD has them.
    pub fn damage_mut(&mut self) -> Option<&mut DamageTexts> {
        self.damage.as_mut()
    }

    /// The objects with a name plate.
    pub fn named(&self) -> Vec<u64> {
        self.names.as_ref().map_or(Vec::new(), |n| n.named())
    }

    /// The objects with a floating damage text in the air.
    pub fn damage_objects(&self) -> Vec<u64> {
        self.damage.as_ref().map_or(Vec::new(), |d| d.objects())
    }

    /// `gm3DTextUI::Update`'s reticle for this frame: the selection (id and name) and, when it is
    /// drawn, its box and projection. `info` is taken once per new selection info
    /// (`RecvNotice_UpdateSelectionInfo`).
    pub fn update_reticle(
        &mut self,
        tree: &mut UiTree,
        selection: Option<(u64, Option<&str>)>,
        info: Option<&SelectionInfo>,
        target: Option<ReticleTarget<'_>>,
    ) -> ReticleOutcome {
        let Some(reticle) = self.reticle.as_mut() else {
            return ReticleOutcome::default();
        };
        let id = selection.map(|(id, _)| id);
        if reticle.current() != id {
            reticle.selection_changed(tree, id, selection.and_then(|(_, name)| name));
            self.reticle_info = None;
        }
        if let Some(info) = info.filter(|i| self.reticle_info.as_ref() != Some(i)) {
            reticle.selection_info(tree, info);
            self.reticle_info = Some(*info);
        }
        reticle.update(tree, target, false)
    }

    /// `gm3DTextUI::UpdateDamage`: the damage texts over `places` at the HUD's clock, rising above
    /// the selection's reticle and name when `selection` is the object they are over.
    pub fn update_damage(
        &mut self,
        tree: &mut UiTree,
        places: &[crate::DamagePlace<'_>],
        selection: u64,
    ) {
        let (Some(damage), reticle) = (self.damage.as_mut(), self.reticle.as_ref()) else {
            return;
        };
        let offset = reticle.map_or(0, |r| r.selection_offset(tree));
        let precluded = reticle.map_or((0, 0), |r| (r.precluded_top, r.precluded_bottom));
        damage.update(
            tree,
            places,
            (selection, offset),
            precluded,
            self.now,
            false,
        );
    }

    /// `gm3DTextUI::UpdateNames` after the reticle: the plates over `places`, clear of the
    /// selection's reticle, within `range` (the radar's, squared as a float).
    pub fn update_names(
        &mut self,
        tree: &mut UiTree,
        places: &[crate::NamePlace<'_>],
        selection: u64,
        range: f32,
    ) {
        let Some(names) = self.names.as_mut() else {
            return;
        };
        let reticle = self.reticle.as_ref();
        let rect = reticle.and_then(|r| r.selection_rect());
        let precluded = reticle.map_or((0, 0), |r| (r.precluded_top, r.precluded_bottom));
        names.update(
            tree,
            places,
            selection,
            rect,
            range * range,
            precluded,
            false,
        );
    }
}
