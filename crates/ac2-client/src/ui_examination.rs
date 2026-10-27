//! The examination panel's effect nodes (`gmExaminationUI::ProcessEffectIDNode`): an effect id
//! expands into the nodes the player's script adds for it
//! (`gmCWInterface::AddEffectDataToProfileByEID`, `ggmCWInterface::CallIFun_AddEffectDataToProfileByEID`),
//! which take the effect node's place in the profile. The expansion's own effect nodes are not
//! expanded again.

use ac2_net::protocol::{ExaminationDataNode, ExaminationProfile, ExaminationValue, NativeObject};
use ac2_script::host::{IFunArg, IFunReturn, OutValue};
use ac2_script::native::{ntype, NativeObject as ScriptNative};

use crate::client::Client;

/// `ggmCWInterface::CallIFun_AddEffectDataToProfileByEID`'s function (`0xB40123`, called on the
/// player's process with the context, order and effect id, then the profile the script fills).
pub const IFUN_ADD_EFFECT_DATA_TO_PROFILE: u32 = 0x00B4_0123;

/// Replaces each effect node in `profile` by what `expand` gives for it (its appearance as the
/// context, its order and its effect id). Nodes other than effects stay where they are; effect
/// nodes in an expansion are dropped (`ProcessEffectIDNode` skips type `0x4000000C` there).
pub fn splice_effects(
    profile: &ExaminationProfile,
    mut expand: impl FnMut(u32, u32, u32) -> Vec<ExaminationDataNode>,
) -> ExaminationProfile {
    let mut nodes = Vec::with_capacity(profile.nodes.len());
    for node in &profile.nodes {
        match node.value {
            ExaminationValue::Effect(eid) => {
                nodes.extend(
                    expand(node.appearance, node.order, eid)
                        .into_iter()
                        .filter(|n| !matches!(n.value, ExaminationValue::Effect(_))),
                );
            }
            _ => nodes.push(node.clone()),
        }
    }
    ExaminationProfile {
        request: profile.request,
        nodes,
    }
}

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
    fn add_effect_data(&mut self, context: u32, order: u32, eid: u32) -> Vec<ExaminationDataNode> {
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

#[cfg(test)]
mod tests {
    use ac2_net::protocol::{ExaminationRequest, StringInfo};

    use super::*;

    fn node(order: u32, appearance: u32, value: ExaminationValue) -> ExaminationDataNode {
        ExaminationDataNode {
            order,
            appearance,
            value,
        }
    }

    #[test]
    fn an_effect_is_replaced_in_place_and_nested_effects_drop() {
        let profile = ExaminationProfile {
            request: ExaminationRequest::default(),
            nodes: vec![
                node(1, 7, ExaminationValue::Int(3)),
                node(2, 8, ExaminationValue::Effect(55)),
                node(3, 9, ExaminationValue::Break),
            ],
        };
        let mut calls = Vec::new();
        let spliced = splice_effects(&profile, |context, order, eid| {
            calls.push((context, order, eid));
            vec![
                node(10, 1, ExaminationValue::Text(StringInfo::default())),
                node(11, 2, ExaminationValue::Effect(99)),
            ]
        });
        // The expansion gets the node's appearance (context), order and effect id.
        assert_eq!(calls, vec![(8, 2, 55)]);
        let orders: Vec<u32> = spliced.nodes.iter().map(|n| n.order).collect();
        assert_eq!(orders, vec![1, 10, 3]);
    }

    #[test]
    fn an_empty_expansion_removes_the_effect_node() {
        let profile = ExaminationProfile {
            request: ExaminationRequest::default(),
            nodes: vec![node(1, 0, ExaminationValue::Effect(4))],
        };
        let spliced = splice_effects(&profile, |_, _, _| Vec::new());
        assert!(spliced.nodes.is_empty());
    }
}
