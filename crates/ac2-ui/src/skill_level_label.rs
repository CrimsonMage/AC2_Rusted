//! A skill's level as a label under its node (AGENTS.md: the skill panel's level labels).
//! Retail's `SkillNodeTemplate` (toolbar layout `0x41000003`) is 38 by 52 pixels: the icon in the
//! top of its frame and `SkillNodeLevelText` (`gmSkillTreeUI::RefreshSkillNode`,
//! `MakeLevelText`) in the frame's foot, 34 by 13 over the frame's art, where the level was hard
//! to read. Here the template is taller by [`LABEL_HEIGHT`] and the level text sits under the
//! frame, the node's full width; the frame's parts keep their size (their bottom edges no longer
//! follow the node), and `SkillTree`'s `node_height` grows by the same amount so the connector
//! lines, which leave a parent at its foot, keep their room. The text, its states (`buff`,
//! `debuff`) and when it shows stay retail's.

use std::sync::Arc;

use crate::layout::{AttributeValue, ElementDesc, LayoutDesc, Rect};
use crate::token::tokenize;

/// The rows added under the node for the label.
pub(crate) const LABEL_HEIGHT: i16 = 16;
/// The gap between the frame's foot and the label.
const GAP: i16 = 2;

fn id(name: &str) -> u32 {
    tokenize(name)
}

/// The template with the label under the frame.
fn node(template: &ElementDesc) -> ElementDesc {
    let mut d = template.clone();
    let foot = d.rect.y1 - d.rect.y0;
    d.rect.y1 += LABEL_HEIGHT;
    let width = d.rect.x1 - d.rect.x0;
    for state in &mut d.states {
        for child in &mut state.children {
            let mut c = (**child).clone();
            if c.id == id("SkillNodeLevelText") {
                c.rect = Rect {
                    x0: 0,
                    y0: foot + GAP,
                    x1: width,
                    y1: foot + LABEL_HEIGHT - 1,
                };
            } else if c.edges[3] == 1 {
                // The frame's parts keep their height as the node grows.
                c.edges[3] = 0;
            }
            *child = Arc::new(c);
        }
    }
    d
}

/// `SkillTree`'s rows taller by the label.
fn tree(d: &ElementDesc) -> ElementDesc {
    let mut d = d.clone();
    if d.id == id("SkillTree") {
        for state in &mut d.states {
            for attribute in &mut state.attributes {
                if let (true, AttributeValue::Int(height)) =
                    (attribute.id == id("node_height"), &mut attribute.value)
                {
                    *height += i32::from(LABEL_HEIGHT);
                }
            }
        }
        return d;
    }
    for state in &mut d.states {
        for child in &mut state.children {
            *child = Arc::new(tree(child));
        }
    }
    d
}

/// The toolbar layout with the skill nodes' level under the frame.
pub(crate) fn restyle(layout: &LayoutDesc) -> LayoutDesc {
    let mut layout = layout.clone();
    for element in &mut layout.elements {
        *element = Arc::new(if element.id == id("SkillNodeTemplate") {
            node(element)
        } else {
            tree(element)
        });
    }
    layout
}
