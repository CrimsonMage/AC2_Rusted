//! NPC conversations (`ac2/shared/src/conversation`, Legions 7160): the conversation a client
//! walks through a `ConversationTree`. The tree and its nodes are read by `ac2-net`; this
//! module holds the place in the tree and the answer rule.

use ac2_net::protocol::{ConversationNode, ConversationTree};

/// A conversation in progress: the tree and the key of the node being spoken.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conversation {
    tree: ConversationTree,
    current: String,
}

impl Conversation {
    /// Starts at `key`; `None` when the tree has no such node.
    pub fn start(tree: ConversationTree, key: &str) -> Option<Self> {
        if tree.node(key).is_some() {
            Some(Self {
                tree,
                current: key.to_string(),
            })
        } else {
            None
        }
    }

    /// The node being spoken.
    pub fn current(&self) -> Option<&ConversationNode> {
        self.tree.node(&self.current)
    }

    /// Moves to `key` when the tree has that node; otherwise the conversation stays put.
    pub fn goto(&mut self, key: &str) -> bool {
        if self.tree.node(key).is_some() {
            self.current = key.to_string();
            true
        } else {
            false
        }
    }

    /// Whether the node being spoken has a decision that waits for the player's response.
    pub fn waits_for_response(&self) -> bool {
        self.current()
            .is_some_and(|node| node.decisions.iter().any(|d| d.wants_response()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ac2_net::protocol::ConversationDecision;

    fn node(name: &str, decision_id: u32) -> (String, ConversationNode) {
        (
            name.to_string(),
            ConversationNode {
                name: name.to_string(),
                actions: Vec::new(),
                decisions: vec![ConversationDecision {
                    decision_id,
                    comparison_type: 0,
                    args: String::new(),
                    destination: "end".to_string(),
                }],
            },
        )
    }

    #[test]
    fn a_conversation_starts_at_a_known_node_and_moves_only_to_known_nodes() {
        let tree = ConversationTree {
            nodes: vec![node("hello", 0), node("end", 0x8000_0001)],
        };
        assert!(Conversation::start(tree.clone(), "missing").is_none());
        let mut talk = Conversation::start(tree, "hello").unwrap();
        assert!(!talk.waits_for_response());
        assert!(!talk.goto("missing"));
        assert_eq!(talk.current().unwrap().name, "hello");
        assert!(talk.goto("end"));
        assert!(talk.waits_for_response());
    }
}
