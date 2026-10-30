//! `gmStoryQuestsUI`: the Quest panel's Story page (`StoryPage`), a list of the story's acts. Each
//! act row (`ActTemplate`) shows its name, a strip of its scenes' icons (`SceneIconTemplate`,
//! completed or not) and, when the act is open, a row per visible scene (`SceneTemplate`). A
//! scene row pressed asks for the scene to play ([`StoryQuestAction::PlayScene`]). The act list
//! comes from the player's script (`gmCWInterface::InqActInfoList`), which the global loop asks
//! for after a notice (`CM_Quest::SendNotice_UpdateStoryQuests`, `RecvNotice_UpdateStoryQuests`).
//!
//! The Quest panel (`gmQuestsUI`, [`crate::QuestsUi`]) owns this page: a notice while the panel is
//! hidden flashes the quest button and makes the Story tab active. Retail's `m_updatedActs` flags
//! only feed a lookup whose result `ReloadActs` never reads, so they are not kept. The element
//! hash is the list's own items. `ReloadActs` lists only the acts whose `_bVisible` is set (the
//! decompile names that read `GetPhysObj`, a getter folded onto the same offset).

use std::collections::BTreeSet;
use std::sync::Arc;

use ac2_net::protocol::{GmActInfo, GmSceneInfo};

use crate::element::UiTree;
use crate::journal_quests::{find_named, JournalSource};
use crate::layout::{AttributeValue, LayoutDesc};
use crate::mouse::UiMessage;
use crate::token::{tokenize, MSG_BUTTON_RELEASE_ON, MSG_MOUSE_PRESS};

/// `RecvNotice_UpdateStoryQuests`' list code (`0x40000002`): the acts reload, no act is updated.
pub const STORY_CODE_LIST: u32 = 0x4000_0002;

/// The data of a scene row's press (`ListenToMessage` on `SceneTemplate`, `0x40000007`).
const SCENE_PRESS_DATA: i32 = 0x4000_0007;

/// What a press on the Story page asks of the player.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoryQuestAction {
    /// The scene of the act plays (`CM_Quest::SendNotice_PlaySequence` with the scene alone).
    PlayScene { act: u32, scene: u32 },
}

/// `gmStoryQuestsUI`'s state.
#[derive(Debug, Clone)]
pub struct StoryQuestsUi {
    layout: Arc<LayoutDesc>,
    /// `m_actListBox`, under the `StoryPage`.
    list: Option<usize>,
    /// `m_actList`: the acts in the script's order.
    acts: Vec<GmActInfo>,
    /// `m_expandedActs`: the acts open.
    expanded: BTreeSet<u32>,
    /// `m_updatedActID`: the act the last notice named.
    updated: u32,
    /// `m_update`: the list waits for the global loop to reload it.
    update: bool,
    /// The panel's own shown state (`UIFramework::IsShown`).
    shown: bool,
}

/// `gmStoryQuestsUI::gmStoryQuestsUI`: the act list under the Story page, if the page is there.
fn story_list(tree: &UiTree) -> Option<usize> {
    let page = find_named(tree, "StoryPage")?;
    tree.child_recursive(page, tokenize("ActListBox"))
}

/// The element `name` of the layout, made as `UIFramework::CreateElement` makes it.
fn make(tree: &mut UiTree, layout: &Arc<LayoutDesc>, name: &str) -> Option<usize> {
    let desc = layout.element(tokenize(name))?.clone();
    tree.create_element(Arc::clone(layout), desc, false)
}

/// The child of `element` named `name`, as `UIElement::GetChildRecursive`.
fn child(tree: &UiTree, element: usize, name: &str) -> Option<usize> {
    tree.child_recursive(element, tokenize(name))
}

/// The id an element carries in attribute `name`, as `UIElement::GetAttribute_ID`.
fn attribute_id(tree: &UiTree, element: Option<usize>, name: &str) -> Option<u32> {
    match tree.get(element?)?.attribute(tokenize(name)) {
        Some(AttributeValue::Id(id)) => Some(*id),
        _ => None,
    }
}

/// The state of a scene's row or icon: completed when playable, else not.
fn completion(scene: &GmSceneInfo) -> u32 {
    if scene.is_playable != 0 {
        tokenize("completed")
    } else {
        tokenize("notcompleted")
    }
}

impl StoryQuestsUi {
    /// `gmStoryQuestsUI::gmStoryQuestsUI` over the toolbar's layout (the page's templates are
    /// there). The list is missing when the page is not in the tree; the panel then does nothing.
    pub fn create(tree: &UiTree, layout: &Arc<LayoutDesc>) -> Self {
        Self {
            layout: Arc::clone(layout),
            list: story_list(tree),
            acts: Vec::new(),
            expanded: BTreeSet::new(),
            updated: 0,
            update: false,
            shown: false,
        }
    }

    /// The acts the list shows, in the script's order.
    pub fn acts(&self) -> &[GmActInfo] {
        &self.acts
    }

    /// `gmStoryQuestsUI::RecvNotice_UpdateStoryQuests`: the act the notice names is the updated
    /// one, and the list reloads on the global loop. A notice other than the list one, while the
    /// Quest panel is hidden (`panel_shown` false), flashes the quest button; the caller then makes
    /// the Story tab active, which `true` asks for.
    pub fn recv_notice(
        &mut self,
        tree: &mut UiTree,
        code: u32,
        act: u32,
        panel_shown: bool,
    ) -> bool {
        self.updated = act;
        self.update = true;
        if code == STORY_CODE_LIST || panel_shown {
            return false;
        }
        if let Some(button) = find_named(tree, "QuestButton") {
            tree.set_state(button, tokenize("changed"));
        }
        true
    }

    /// Whether a reload waits for the global loop.
    pub fn reload_pending(&self) -> bool {
        self.update
    }

    /// The global loop's check (`ListenToMessage` 0xffff0005): a pending reload, cleared.
    pub fn take_reload(&mut self) -> bool {
        std::mem::take(&mut self.update)
    }

    /// `gmStoryQuestsUI::ReloadActs`: the list flushed and laid out again from `acts`. An act
    /// that is open (or the updated one, which opens) shows its scene rows, and the updated act
    /// scrolls into view by its index, as retail does. An act that is not visible is not listed.
    pub fn reload(&mut self, tree: &mut UiTree, source: &dyn JournalSource, acts: Vec<GmActInfo>) {
        let Some(list) = self.list else {
            return;
        };
        tree.list_flush(list);
        self.acts = acts;
        let layout = Arc::clone(&self.layout);
        let mut scroll = None;
        tree.list_multi_op_start(list);
        for (index, act) in self.acts.iter().enumerate() {
            // Retail's check is the act's `_bVisible` word (`CWPhysicsObject::GetPhysObj`, an
            // identical-code-folded getter, reads the same offset): a hidden act has no row, though
            // it still counts in the index the updated act scrolls to.
            if act.visible == 0 {
                continue;
            }
            if act.id == self.updated {
                self.expanded.insert(act.id);
                scroll = Some(index);
            }
            let Some(row) = make(tree, &layout, "ActTemplate") else {
                continue;
            };
            tree.list_add_item(list, row);
            if let Some(name) = child(tree, row, "ActName") {
                let text = source.render_info(&act.name);
                tree.set_text(name, &text);
            }
            let open = self.expanded.contains(&act.id);
            tree.set_state(row, tokenize(if open { "highlight" } else { "normal" }));
            tree.set_attribute_id(row, tokenize("actid"), act.id);
            let visible: Vec<&GmSceneInfo> =
                act.scenes.iter().filter(|s| s.is_hidden == 0).collect();
            if let Some(icons) = child(tree, row, "SceneIconListBox") {
                tree.list_multi_op_start(icons);
                for scene in &visible {
                    if let Some(icon) = make(tree, &layout, "SceneIconTemplate") {
                        tree.list_add_item(icons, icon);
                        tree.set_state(icon, completion(scene));
                    }
                }
                tree.list_multi_op_complete(icons);
            }
            if !open {
                continue;
            }
            for scene in &visible {
                let Some(scene_row) = make(tree, &layout, "SceneTemplate") else {
                    continue;
                };
                tree.list_add_item(list, scene_row);
                tree.set_state(scene_row, completion(scene));
                tree.set_attribute_id(scene_row, tokenize("sceneid"), scene.id);
                tree.set_attribute_id(scene_row, tokenize("actid"), act.id);
                if let Some(name) = child(tree, scene_row, "SceneName") {
                    let text = source.render_info(&scene.name);
                    tree.set_text(name, &text);
                }
            }
        }
        tree.list_multi_op_complete(list);
        if let Some(index) = scroll {
            tree.list_scroll_to_row(list, index as i32);
        }
    }

    /// `gmStoryQuestsUI::Show`: the page's shown state. Hiding the panel with an updated act
    /// pending closes that act and reloads (`m_updatedActID` cleared).
    pub fn show(&mut self, shown: bool) {
        let was_shown = self.shown;
        self.shown = shown;
        if was_shown && !shown && self.updated != 0 {
            self.expanded.remove(&self.updated);
            self.updated = 0;
            self.update = true;
        }
    }

    /// `gmStoryQuestsUI::ListenToMessage` for the page's elements. A release on an act row
    /// toggles it open or closed; a press on a scene row returns the scene to play when it is
    /// playable.
    pub fn message(&mut self, tree: &UiTree, message: &UiMessage) -> Option<StoryQuestAction> {
        if message.element_id == tokenize("ActTemplate") && message.message == MSG_BUTTON_RELEASE_ON
        {
            let act = attribute_id(tree, message.element, "actid")?;
            self.updated = 0;
            if !self.expanded.remove(&act) {
                self.expanded.insert(act);
            }
            self.update = true;
            return None;
        }
        if message.element_id == tokenize("SceneTemplate")
            && message.message == MSG_MOUSE_PRESS
            && message.data == SCENE_PRESS_DATA
        {
            let act = attribute_id(tree, message.element, "actid")?;
            let scene = attribute_id(tree, message.element, "sceneid")?;
            let playable = self
                .acts
                .iter()
                .filter(|a| a.id == act)
                .flat_map(|a| a.scenes.iter())
                .any(|s| s.id == scene && s.is_playable != 0);
            if playable {
                return Some(StoryQuestAction::PlayScene { act, scene });
            }
        }
        None
    }

    /// The act `id` as the script listed it (`ggmActInfo` scenes are read from it).
    pub fn act(&self, id: u32) -> Option<&GmActInfo> {
        self.acts.iter().find(|a| a.id == id)
    }
}
