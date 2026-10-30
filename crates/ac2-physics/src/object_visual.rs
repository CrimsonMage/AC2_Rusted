//! The object's visual description state and the `CPhysicsObj` family that edits it:
//! `m_local_vdesc` (what the server said the object looks like) and `m_effects_vdesc` (the
//! appearance keys FX applied, merged back onto every new local description).
//!
//! The merge algebra is [`Appearance`]'s (`appearance_ops`); this wraps the two descriptions
//! the way `CPhysicsObj` holds them. The part rebuild that retail's `ApplyVisualDesc` runs after
//! a change (`CPartGroup::CompareAndModify` against `GetTotalVisualDesc`) is the caller's:
//! `PartGroupTree::compare_and_modify` over the total description `ac2_assets::total_visual_desc`
//! builds from the DAT chain and [`ObjectVisualDesc::local`].

use crate::appearance::Appearance;

/// `MasterDBMap::DivineType` of a `VisualDesc` DID.
pub const VISUAL_DESC_TYPE: u32 = 0x4000_0021;
/// The part group key `VisualDesc::GatherAppearanceKeys` reads the global tables under
/// (`CombineAndApplyVisualDesc` passes `0x40000002`).
const GLOBAL_KEY: u32 = crate::appearance::GLOBAL_PART_GROUP;

/// `CPhysicsObj::m_local_vdesc` and `m_effects_vdesc`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ObjectVisualDesc {
    local: Appearance,
    effects: Appearance,
}

impl ObjectVisualDesc {
    pub fn new() -> Self {
        Self::default()
    }

    /// An object whose local description is `local` and which has no effects.
    pub fn from_local(local: Appearance) -> Self {
        Self {
            local,
            effects: Appearance::default(),
        }
    }

    /// The local description, consuming the wrapper.
    pub fn into_local(self) -> Appearance {
        self.local
    }

    /// `CPhysicsObj::GetLocalVisualDesc`.
    pub fn local(&self) -> &Appearance {
        &self.local
    }

    /// `CPhysicsObj::m_effects_vdesc`.
    pub fn effects(&self) -> &Appearance {
        &self.effects
    }

    /// `CPhysicsObj::InitVisualDesc` for a `VisualDesc` DID: the local description is cleared
    /// and its parent becomes `parent_did`. (An `EntityDesc` DID resolves to its visual DID
    /// first, which needs the DAT, so the caller does that; `0` is retail's failure.) False for
    /// a zero DID.
    pub fn init_visual_desc(&mut self, parent_did: u32) -> bool {
        if parent_did == 0 {
            return false;
        }
        self.local.clear();
        self.local.parent = parent_did;
        true
    }

    /// `CPhysicsObj::SetLocalVisualDesc`: the local description becomes `delta`
    /// (`VisualDesc::operator=`) with the effects merged on top. Returns the new description,
    /// which retail sends to the weenie as `LocalVisualDescUpdate`.
    pub fn set_local_visual_desc(&mut self, delta: &Appearance) -> &Appearance {
        self.local.load_from(delta);
        let effects = self.effects.clone();
        self.local.apply_additive_appearance(&effects);
        &self.local
    }

    /// `CPhysicsObj::ApplyAdditiveVisualDesc`: `VisualDesc::ApplyAdditiveVDesc` onto the local
    /// description.
    pub fn apply_additive_visual_desc(&mut self, delta: &Appearance) -> bool {
        self.local.apply_additive_appearance(delta)
    }

    /// `CPhysicsObj::ApplySubtractiveVisualDesc`: `VisualDesc::ApplySubtractiveVDesc` from the
    /// local description; `parent` is the DAT description its parent DID names.
    pub fn apply_subtractive_visual_desc(
        &mut self,
        delta: &Appearance,
        parent: Option<&Appearance>,
    ) -> bool {
        self.local.apply_subtractive(delta, parent)
    }

    /// `CPhysicsObj::SetVDescScale`: the local description's scale.
    pub fn set_vdesc_scale(&mut self, scale: [f32; 3]) {
        self.local.scale = scale;
    }

    /// `CPhysicsObj::DoAppearanceFX`: the key joins the effects and, when it did, merges onto
    /// the local description. True when the local description changed.
    pub fn do_appearance_fx(&mut self, table: u32, key: (u32, f32)) -> bool {
        self.local.do_appearance_fx(&mut self.effects, table, key)
    }

    /// `CPhysicsObj::RemoveAppearanceFX`.
    pub fn remove_appearance_fx(&mut self, table: u32, key: u32) {
        self.local
            .remove_appearance_fx(&mut self.effects, table, key);
    }

    /// `CPhysicsObj::CombineAndApplyVisualDesc(vdescID)`: the local description becomes the DAT
    /// description `vdesc` with the effects merged on top, and the global appearance tables the
    /// object had in its total description (`total`: `GetTotalVisualDesc` before the change,
    /// from `ac2_assets::total_visual_desc`) are added back as global keys
    /// (`GatherAppearanceKeys` then `AddAppearanceKey`). `parent` is the description `vdesc`'s
    /// own parent names, for `MakeLocalPGDDFromParent`. True; retail raises
    /// `LocalVisualDescUpdate` and refreshes the parts after.
    pub fn combine_and_apply_visual_desc(
        &mut self,
        vdesc: &Appearance,
        total: &Appearance,
        parent: Option<&Appearance>,
    ) -> bool {
        let gathered = total.global.appearance.clone();
        self.local.load_from(vdesc);
        let effects = self.effects.clone();
        self.local.apply_additive_appearance(&effects);
        for (table, keys) in &gathered {
            self.local
                .add_appearance_key(GLOBAL_KEY, *table, keys, parent);
        }
        true
    }

    /// `CPhysicsObj::ApplyVisualDesc(delta)`: `SetLocalVisualDesc` of `delta`; true when the
    /// parts must be rebuilt (retail's second call, the part-array rebuild with an FX reset).
    pub fn apply_visual_desc(&mut self, delta: &Appearance) -> bool {
        self.set_local_visual_desc(delta);
        true
    }
}

/// `CPhysicsObj::IsVisible`: an object is hidden only while its animation opacity
/// (`PHYSICSOBJREP::m_animopacity`) is under 0.0002; one without a representation is visible.
pub fn is_visible(anim_opacity: Option<f32>) -> bool {
    anim_opacity.is_none_or(|opacity| opacity >= 0.0002)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_global_key(table: u32, key: u32, scalar: f32) -> Appearance {
        let mut a = Appearance::default();
        a.global.appearance = vec![(table, vec![(key, scalar)])];
        a
    }

    #[test]
    fn set_local_takes_the_delta_and_the_effects() {
        let mut v = ObjectVisualDesc::new();
        assert!(v.do_appearance_fx(0x2000_0001, (7, 0.5)));
        let mut delta = with_global_key(0x2000_0002, 1, 1.0);
        delta.parent = 0x2300_0001;
        v.set_local_visual_desc(&delta);
        assert_eq!(v.local().parent, 0x2300_0001);
        assert_eq!(
            v.local().applied_scalar(GLOBAL_KEY, 0x2000_0002, 1),
            Some(1.0)
        );
        assert_eq!(
            v.local().applied_scalar(GLOBAL_KEY, 0x2000_0001, 7),
            Some(0.5)
        );
        v.remove_appearance_fx(0x2000_0001, 7);
        assert_eq!(v.local().applied_scalar(GLOBAL_KEY, 0x2000_0001, 7), None);
        assert!(v.effects().is_empty() || v.effects().global.appearance.is_empty());
    }

    #[test]
    fn additive_then_subtractive_leaves_the_key_out() {
        let mut v = ObjectVisualDesc::new();
        let gear = with_global_key(0x2000_0003, 4, 1.0);
        assert!(v.apply_additive_visual_desc(&gear));
        assert_eq!(
            v.local().applied_scalar(GLOBAL_KEY, 0x2000_0003, 4),
            Some(1.0)
        );
        assert!(v.apply_subtractive_visual_desc(&gear, None));
        assert_eq!(v.local().applied_scalar(GLOBAL_KEY, 0x2000_0003, 4), None);
    }

    #[test]
    fn combine_keeps_the_totals_global_tables() {
        let mut v = ObjectVisualDesc::new();
        let total = with_global_key(0x2000_0009, 2, 0.25);
        let dat = Appearance::default();
        assert!(v.combine_and_apply_visual_desc(&dat, &total, None));
        assert_eq!(
            v.local().applied_scalar(GLOBAL_KEY, 0x2000_0009, 2),
            Some(0.25)
        );
    }

    #[test]
    fn init_clears_and_names_the_parent_and_scale_is_set() {
        let mut v = ObjectVisualDesc::new();
        assert!(!v.init_visual_desc(0));
        v.apply_additive_visual_desc(&with_global_key(1, 1, 1.0));
        assert!(v.init_visual_desc(0x2300_0042));
        assert_eq!(v.local().parent, 0x2300_0042);
        assert!(v.local().global.appearance.is_empty());
        v.set_vdesc_scale([2.0, 2.0, 3.0]);
        assert_eq!(v.local().scale, [2.0, 2.0, 3.0]);
    }

    #[test]
    fn visibility_follows_the_animation_opacity() {
        assert!(is_visible(None));
        assert!(is_visible(Some(1.0)));
        assert!(!is_visible(Some(0.0001)));
    }
}
