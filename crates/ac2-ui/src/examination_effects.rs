//! The examination panel's effect nodes (`gmExaminationUI::ProcessEffectIDNode`): each effect node
//! of a profile is replaced in place by the nodes its expansion gives. The expansion's own effect
//! nodes are dropped, not expanded again. The client supplies the expansion
//! (`gmCWInterface::AddEffectDataToProfileByEID`).

use ac2_net::protocol::{ExaminationDataNode, ExaminationProfile, ExaminationValue};

use crate::examination::ExaminationItem;

/// The player's answers to the examination panel: an effect's expansion (the player's script,
/// `gmCWInterface::AddEffectDataToProfileByEID`) and the profile's text items
/// (`gmExaminationUI::MakeString` and `ProcessNode`).
pub trait ExaminationSource {
    /// The nodes an effect adds at its place, for the effect node's `context`, `order` and `eid`.
    fn effect_nodes(&self, context: u32, order: u32, eid: u32) -> Vec<ExaminationDataNode>;
    /// The panel's items for a profile whose effects are already expanded.
    fn examination_items(&self, profile: &ExaminationProfile) -> Vec<ExaminationItem>;
}

/// `gmExaminationUI::ProcessEffectIDNode` for each effect node of a profile: the source's
/// expansion (`ExaminationSource::effect_nodes`) takes each effect node's place. A profile without
/// effects is returned as it is; a call that fails expands to nothing.
pub fn expand_examination_effects<S: ExaminationSource + ?Sized>(
    source: &S,
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
        source.effect_nodes(context, order, eid)
    })
}

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
