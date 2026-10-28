//! NPC conversation trees (`ConversationTree`, DAT type `0x40000057`), Legions 7160 layouts.
//! The client unpacks a tree from the DAT (`DBObj::UnPack`), so these are DAT bytes, not
//! session events. The tree is a `_STL::map<PStringBase<unsigned_short>, ConversationNode>`:
//! `STREAMPACK_STL` writes a u32 count, then each key (a UTF-16 string) and its node. A node
//! holds its name, a `PList<ConversationAction>` and a `PList<ConversationDecision>`, each a u32
//! count and the entries. Every string is a `PStringBase<unsigned_short>` (see `string`).
//!
//! The map keeps the wire order here, not a sorted order: a tree that is decoded and packed again
//! gives the same bytes, and lookups compare keys for equality.

use crate::protocol::reader::{Reader, Writer};
use crate::protocol::ProtocolError;

/// `ConversationAction::StreamPack`: the action id, then its arguments.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConversationAction {
    pub action_id: u32,
    pub args: String,
}

/// `ConversationDecision::StreamPack`: the decision id, the comparison type, the arguments and
/// the destination node's key.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConversationDecision {
    pub decision_id: u32,
    pub comparison_type: u32,
    pub args: String,
    pub destination: String,
}

/// `ConversationNode::StreamPack`: the node's name, its actions, then its decisions.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConversationNode {
    pub name: String,
    pub actions: Vec<ConversationAction>,
    pub decisions: Vec<ConversationDecision>,
}

/// `ConversationTree`: its nodes in wire order, each under its map key.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConversationTree {
    pub nodes: Vec<(String, ConversationNode)>,
}

/// `PList<T>::UnPack`: a u32 count, then that many entries.
fn plist<T>(
    r: &mut Reader<'_>,
    mut one: impl FnMut(&mut Reader<'_>) -> Result<T, ProtocolError>,
) -> Result<Vec<T>, ProtocolError> {
    let count = r.count()?;
    (0..count).map(|_| one(r)).collect()
}

/// `PList<T>::Pack`: the count, then each entry in list order.
fn write_plist<T>(w: &mut Writer, items: &[T], mut one: impl FnMut(&T, &mut Writer)) {
    w.u32(items.len() as u32);
    for item in items {
        one(item, w);
    }
}

impl ConversationAction {
    fn read(r: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        Ok(Self {
            action_id: r.u32()?,
            args: r.wide_string()?,
        })
    }

    fn write(&self, w: &mut Writer) {
        w.u32(self.action_id).wide_string(&self.args);
    }
}

impl ConversationDecision {
    /// `ConversationDecision::WantsResponse`: the decision id `0x80000001` waits for the
    /// player's answer rather than following on its own.
    pub const RESPONSE_ID: u32 = 0x8000_0001;

    /// Whether this decision waits for the player's response.
    pub fn wants_response(&self) -> bool {
        self.decision_id == Self::RESPONSE_ID
    }

    fn read(r: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        Ok(Self {
            decision_id: r.u32()?,
            comparison_type: r.u32()?,
            args: r.wide_string()?,
            destination: r.wide_string()?,
        })
    }

    fn write(&self, w: &mut Writer) {
        w.u32(self.decision_id)
            .u32(self.comparison_type)
            .wide_string(&self.args)
            .wide_string(&self.destination);
    }
}

impl ConversationNode {
    fn read(r: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        Ok(Self {
            name: r.wide_string()?,
            actions: plist(r, ConversationAction::read)?,
            decisions: plist(r, ConversationDecision::read)?,
        })
    }

    fn write(&self, w: &mut Writer) {
        w.wide_string(&self.name);
        write_plist(w, &self.actions, ConversationAction::write);
        write_plist(w, &self.decisions, ConversationDecision::write);
    }
}

impl ConversationTree {
    /// `ConversationTree::StreamPack` read through `STREAMPACK_STL`. A key already present keeps
    /// its first node, as `insert_unique` does; the duplicate's bytes are still consumed.
    pub(crate) fn read(r: &mut Reader<'_>) -> Result<Self, ProtocolError> {
        let count = r.count()?;
        let mut nodes: Vec<(String, ConversationNode)> = Vec::new();
        for _ in 0..count {
            let key = r.wide_string()?;
            let node = ConversationNode::read(r)?;
            if !nodes.iter().any(|(k, _)| *k == key) {
                nodes.push((key, node));
            }
        }
        Ok(Self { nodes })
    }

    pub(crate) fn write(&self, w: &mut Writer) {
        w.u32(self.nodes.len() as u32);
        for (key, node) in &self.nodes {
            w.wide_string(key);
            node.write(w);
        }
    }

    /// Decodes a whole DAT tree record; the bytes must end where the tree does.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let mut r = Reader::new(bytes);
        let tree = Self::read(&mut r)?;
        r.finish()?;
        Ok(tree)
    }

    /// The tree's DAT bytes, as `ConversationTree::StreamPack` writes them.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut w = Writer::new();
        self.write(&mut w);
        w.bytes
    }

    /// The node under `key`.
    pub fn node(&self, key: &str) -> Option<&ConversationNode> {
        self.nodes
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, node)| node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A tree with one node under the empty key, holding one empty decision.
    fn one_empty_decision(decision_id: u32) -> ConversationTree {
        ConversationTree {
            nodes: vec![(
                String::new(),
                ConversationNode {
                    name: String::new(),
                    actions: Vec::new(),
                    decisions: vec![ConversationDecision {
                        decision_id,
                        ..ConversationDecision::default()
                    }],
                },
            )],
        }
    }

    #[test]
    fn an_empty_tree_is_one_zero_count() {
        assert_eq!(ConversationTree::default().to_bytes(), [0, 0, 0, 0]);
        assert_eq!(
            ConversationTree::from_bytes(&[0, 0, 0, 0]).unwrap(),
            ConversationTree::default()
        );
    }

    #[test]
    fn a_decision_packs_its_id_comparison_and_two_strings_exactly() {
        // Count 1; key "" (4 bytes, padding included); node name ""; 0 actions; 1 decision with
        // id 0x80000001, comparison 0, args "" and destination "" (each an empty string of 4).
        let tree = one_empty_decision(ConversationDecision::RESPONSE_ID);
        let expected = [
            1, 0, 0, 0, // nodes
            0, 0, 0, 0, // key ""
            0, 0, 0, 0, // node name ""
            0, 0, 0, 0, // actions
            1, 0, 0, 0, // decisions
            1, 0, 0, 0x80, // decision id
            0, 0, 0, 0, // comparison type
            0, 0, 0, 0, // args ""
            0, 0, 0, 0, // destination ""
        ];
        assert_eq!(tree.to_bytes(), expected);
        let back = ConversationTree::from_bytes(&expected).unwrap();
        assert_eq!(back, tree);
        assert!(back.node("").unwrap().decisions[0].wants_response());
    }

    #[test]
    fn a_short_string_is_a_unit_count_then_the_scrambled_unit_then_padding() {
        // "a" is one unit: count 1, then 0x61 nibble-swapped to 0x16 (a partial word is not XORed),
        // then one byte of padding.
        let action = ConversationAction {
            action_id: 5,
            args: "a".to_string(),
        };
        let mut w = Writer::new();
        action.write(&mut w);
        assert_eq!(w.bytes, [5, 0, 0, 0, 1, 0, 0x16, 0]);
        let mut r = Reader::new(&w.bytes);
        assert_eq!(ConversationAction::read(&mut r).unwrap(), action);
        r.finish().unwrap();
    }

    #[test]
    fn a_node_with_an_action_and_a_decision_round_trips() {
        let tree = ConversationTree {
            nodes: vec![(
                "hi".to_string(),
                ConversationNode {
                    name: "hi".to_string(),
                    actions: vec![ConversationAction {
                        action_id: 7,
                        args: "wave".to_string(),
                    }],
                    decisions: vec![ConversationDecision {
                        decision_id: 2,
                        comparison_type: 3,
                        args: "bye".to_string(),
                        destination: "end".to_string(),
                    }],
                },
            )],
        };
        let bytes = tree.to_bytes();
        assert_eq!(ConversationTree::from_bytes(&bytes).unwrap(), tree);
        assert_eq!(
            ConversationTree::from_bytes(&bytes).unwrap().to_bytes(),
            bytes
        );
        assert!(!tree.node("hi").unwrap().decisions[0].wants_response());
    }

    #[test]
    fn a_repeated_key_keeps_its_first_node() {
        let first = one_empty_decision(0);
        let mut second = first.clone();
        second.nodes[0].1.name = "second".to_string();
        // Two entries under the same key: the second node's bytes are read, then dropped.
        let mut bytes = 2_u32.to_le_bytes().to_vec();
        bytes.extend(&first.to_bytes()[4..]);
        bytes.extend(&second.to_bytes()[4..]);
        let read = ConversationTree::from_bytes(&bytes).unwrap();
        assert_eq!(read.nodes.len(), 1);
        assert_eq!(read.nodes[0].1.name, "");
    }

    #[test]
    fn truncated_or_oversized_trees_are_refused() {
        let bytes = one_empty_decision(0).to_bytes();
        assert!(ConversationTree::from_bytes(&bytes[..bytes.len() - 1]).is_err());
        assert!(ConversationTree::from_bytes(&[0xFF, 0x1F, 0, 0]).is_err());
        let mut trailing = bytes.clone();
        trailing.push(0);
        assert!(ConversationTree::from_bytes(&trailing).is_err());
    }
}
