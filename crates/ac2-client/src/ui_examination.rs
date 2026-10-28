//! The examination panel's effect nodes (`gmExaminationUI::ProcessEffectIDNode`): an effect id
//! expands into the nodes the player's script adds for it
//! (`gmCWInterface::AddEffectDataToProfileByEID`, `ggmCWInterface::CallIFun_AddEffectDataToProfileByEID`),
//! which take the effect node's place in the profile. The expansion's own effect nodes are not
//! expanded again.

use ac2_net::protocol::{ExaminationDataNode, ExaminationProfile, ExaminationValue, NativeObject};
use ac2_script::host::{IFunArg, IFunReturn, OutValue};
use ac2_script::native::{ntype, NativeObject as ScriptNative};

use crate::client::Client;
use ac2_ui::splice_effects;

/// `ggmCWInterface::CallIFun_AddEffectDataToProfileByEID`'s function (`0xB40123`, called on the
/// player's process with the context, order and effect id, then the profile the script fills).
pub const IFUN_ADD_EFFECT_DATA_TO_PROFILE: u32 = 0x00B4_0123;

impl Client {
    /// `gmExaminationUI::ProcessEffectIDNode` for each effect node of a profile. A profile without
    /// effects is returned as it is; a call that fails (no player, or no script) expands to
    /// nothing.
    pub(crate) fn expand_examination_effects(
        &mut self,
        profile: &ExaminationProfile,
    ) -> ExaminationProfile {
        if !profile
            .nodes
            .iter()
            .any(|n| matches!(n.value, ExaminationValue::Effect(_)))
        {
            return profile.clone();
        }
        splice_effects(profile, |context, order, eid| {
            self.add_effect_data(context, order, eid)
        })
    }

    /// `gmCWInterface::AddEffectDataToProfileByEID`: the script fills an empty profile (the
    /// fourth parameter) with the effect's nodes.
    pub(crate) fn add_effect_data(
        &mut self,
        context: u32,
        order: u32,
        eid: u32,
    ) -> Vec<ExaminationDataNode> {
        let args = [
            IFunArg::Int(context),
            IFunArg::Int(order),
            IFunArg::Int(eid),
            IFunArg::Filled(ntype::EXAMINATION_PROFILE),
        ];
        let Ok(result) = self.query(IFUN_ADD_EFFECT_DATA_TO_PROFILE, &args, IFunReturn::Word)
        else {
            return Vec::new();
        };
        match result.outs.get(3) {
            Some(Some(OutValue::Vm(ScriptNative::Wire(profile)))) => match &**profile {
                NativeObject::ExaminationProfile(p) => p.nodes.clone(),
                _ => Vec::new(),
            },
            _ => Vec::new(),
        }
    }
}
