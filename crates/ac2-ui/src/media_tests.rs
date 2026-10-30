//! Media timelines and state transitions on hand-built layouts.

use std::sync::Arc;

use crate::layout::{ElementDesc, LayoutDesc, MediaDesc, Rect, StateDesc};
use crate::layout_media::{TransitionData, TransitionDesc};
use crate::token::{tokenize, STATELESS};
use crate::UiTree;

fn state(name: &str, media: Vec<MediaDesc>, transitions: Vec<TransitionDesc>) -> StateDesc {
    StateDesc {
        id: if name == "stateless" {
            STATELESS
        } else {
            tokenize(name)
        },
        flags: 0,
        rect: Rect {
            x0: -1,
            y0: -1,
            x1: -1,
            y1: -1,
        },
        z_level: 0,
        attributes: Vec::new(),
        media,
        transitions,
        children: Vec::new(),
    }
}

/// A 20 x 20 root with these states, starting in the first.
fn tree(states: Vec<StateDesc>) -> (UiTree, usize) {
    let root = ElementDesc {
        id: tokenize("Root"),
        kind: tokenize("field"),
        default_state: states[0].id,
        rect: Rect {
            x0: 0,
            y0: 0,
            x1: 19,
            y1: 19,
        },
        z_level: 0,
        edges: [0; 4],
        states,
    };
    let layout = Arc::new(LayoutDesc {
        did: 0,
        sub_ids: Vec::new(),
        string_table: 0,
        width: 20,
        height: 20,
        elements: vec![Arc::new(root)],
    });
    let mut tree = UiTree::new(100, 100);
    let root = tree.create_root(layout, tokenize("Root")).unwrap();
    (tree, root)
}

fn image(tree: &UiTree, index: usize) -> Option<u32> {
    tree.get(index).unwrap().image().map(|i| i.file)
}

fn anim(frames: &[u32], duration: f32) -> MediaDesc {
    MediaDesc::Anim {
        duration,
        draw_mode: 0,
        offset: (0, 0),
        frames: frames.to_vec(),
    }
}

#[test]
fn an_animation_shows_its_frames_over_its_duration_and_a_jump_loops_it() {
    let (mut tree, root) = tree(vec![state(
        "spin",
        vec![
            anim(&[1, 2, 3], 0.3),
            MediaDesc::Jump {
                index: 0,
                probability: 1.0,
            },
        ],
        vec![],
    )]);
    // Made at time 0: the first frame.
    assert_eq!(image(&tree, root), Some(1));
    tree.global_loop(0.15);
    assert_eq!(image(&tree, root), Some(2));
    // The last frame shows and the run goes on at once: the jump starts the animation again
    // from this time.
    tree.global_loop(0.25);
    assert_eq!(image(&tree, root), Some(1));
    tree.global_loop(0.36);
    assert_eq!(image(&tree, root), Some(2));
}

#[test]
fn a_pause_holds_the_run_then_a_state_item_switches_state() {
    let (mut tree, root) = tree(vec![
        state(
            "intro",
            vec![
                MediaDesc::Pause { min: 1.0, max: 1.0 },
                MediaDesc::State {
                    state: tokenize("idle"),
                },
            ],
            vec![],
        ),
        state(
            "idle",
            vec![
                MediaDesc::Image {
                    file: 7,
                    draw_mode: 0,
                    offset: (0, 0),
                },
                MediaDesc::Sound { sound: 0x0A00_0001 },
            ],
            vec![],
        ),
    ]);
    tree.global_loop(0.5);
    assert_eq!(tree.get(root).unwrap().state(), tokenize("intro"));
    tree.global_loop(1.0);
    assert_eq!(tree.get(root).unwrap().state(), tokenize("idle"));
    assert_eq!(image(&tree, root), Some(7));
    assert_eq!(tree.take_sounds(), vec![0x0A00_0001]);
}

#[test]
fn a_message_item_broadcasts_from_the_element() {
    let ping = tokenize("ping");
    let (mut tree, root) = tree(vec![state(
        "stateless",
        vec![MediaDesc::Message {
            message: ping,
            probability: 1.0,
        }],
        vec![],
    )]);
    let messages = tree.take_messages();
    assert!(messages
        .iter()
        .any(|m| m.message == ping && m.element == Some(root)));
}

fn fade(when: u32, state: &str, start: f32, end: f32) -> TransitionDesc {
    TransitionDesc {
        when,
        state: tokenize(state),
        data: TransitionData::Fade {
            end_alpha: end,
            start_alpha: start,
            duration: 1.0,
            flags: 1,
        },
    }
}

fn opacity(tree: &UiTree, root: usize) -> f32 {
    tree.placements()
        .iter()
        .find(|p| p.root == root)
        .unwrap()
        .opacity
}

#[test]
fn an_exit_fade_holds_the_old_state_until_it_ends_and_an_enter_fade_follows() {
    let (mut tree, root) = tree(vec![
        state(
            "open",
            vec![],
            vec![fade(2, "closed", 1.0, 0.0), fade(1, "all_states", 0.0, 1.0)],
        ),
        state("closed", vec![], vec![]),
    ]);
    tree.global_loop(10.0);
    tree.set_state(root, tokenize("closed"));
    // Still open while it fades out.
    assert_eq!(tree.get(root).unwrap().state(), tokenize("open"));
    tree.global_loop(10.5);
    assert_eq!(opacity(&tree, root), 0.5);
    // Asking again for the state it is heading to changes nothing.
    tree.set_state(root, tokenize("closed"));
    tree.global_loop(11.0);
    assert_eq!(tree.get(root).unwrap().state(), tokenize("closed"));
    assert_eq!(opacity(&tree, root), 0.0);
    // Opening fades in from 0.
    tree.set_state(root, tokenize("open"));
    assert_eq!(tree.get(root).unwrap().state(), tokenize("open"));
    tree.global_loop(11.25);
    assert_eq!(opacity(&tree, root), 0.25);
}

fn still(file: u32) -> MediaDesc {
    MediaDesc::Image {
        file,
        draw_mode: 0,
        offset: (0, 0),
    }
}

/// `UIElement::AddImageToState`: an image added to the stateless state is shown and comes back
/// when the element leaves a state with its own image for one without media; a state the
/// description lacks takes nothing; an image added to another state waits for it.
#[test]
fn an_image_added_to_a_state_returns_with_that_state() {
    let (mut tree, root) = tree(vec![
        state("stateless", vec![], vec![]),
        state("lit", vec![still(7)], vec![]),
        state("dim", vec![], vec![]),
    ]);
    assert!(tree.add_image_to_state(root, 5, 2, STATELESS));
    assert_eq!(image(&tree, root), Some(5));
    tree.set_state(root, tokenize("lit"));
    assert_eq!(image(&tree, root), Some(7));
    tree.set_state(root, tokenize("dim"));
    assert_eq!(image(&tree, root), Some(5), "the stateless media run again");
    assert!(!tree.add_image_to_state(root, 9, 2, tokenize("absent")));
    assert_eq!(image(&tree, root), Some(5));
    assert!(tree.add_image_to_state(root, 8, 2, tokenize("lit")));
    assert_eq!(image(&tree, root), Some(5), "not the current state");
    tree.set_state(root, tokenize("lit"));
    assert_eq!(image(&tree, root), Some(8), "after the state's own image");
}
