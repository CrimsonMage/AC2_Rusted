//! `gmToolbarUI`'s panel children (`SetupChildren`): examination, navigation and options on the
//! left side panel, inventory, quests, skills and social on the right. Each child's constructor
//! hides its field (`gmSkillUI::Show(0)` and the like: `Show` sets `XxxField` visible as the
//! child is shown); its toolbar button (`XxxButton`, `mouse_release_on`; the quest button is a
//! field and sends `mouse_release`) toggles it, its close button (`XxxCloseButton`) hides it, and
//! the child keys (`0x41000001`..`07`, `gmToolbarUI::ChildKeyPress`) toggle it.
//! `ManageChildren` hides the children that share a side panel when one shows (their
//! `MExList`).
//!
//! Not modelled yet: the panels' contents (each child's own frameworks), the help buttons
//! (`KeyStone::OpenHelp`), the buttons' show/hide tooltips, the key stone and plug-in keys.

use crate::element::UiTree;
use crate::layout::AttributeValue;
use crate::mouse::{UiMessage, GLOBAL_KEY_PRESS};
use crate::token::{tokenize, MSG_BUTTON_RELEASE_ON, MSG_MOUSE_RELEASE};

/// A toolbar panel child.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolbarChild {
    Examination,
    Navigation,
    Option,
    Inventory,
    Quest,
    Skill,
    Social,
}

impl ToolbarChild {
    pub const ALL: [Self; 7] = [
        Self::Examination,
        Self::Navigation,
        Self::Option,
        Self::Inventory,
        Self::Quest,
        Self::Skill,
        Self::Social,
    ];

    fn name(self) -> &'static str {
        match self {
            Self::Examination => "Examination",
            Self::Navigation => "Navigation",
            Self::Option => "Option",
            Self::Inventory => "Inventory",
            Self::Quest => "Quest",
            Self::Skill => "Skill",
            Self::Social => "Social",
        }
    }

    /// `ToolbarChildInfo::keypress`.
    pub fn key(self) -> u32 {
        match self {
            Self::Examination => 0x4100_0001,
            Self::Inventory => 0x4100_0002,
            Self::Navigation => 0x4100_0003,
            Self::Option => 0x4100_0004,
            Self::Quest => 0x4100_0005,
            Self::Skill => 0x4100_0006,
            Self::Social => 0x4100_0007,
        }
    }

    /// The left side panel's children exclude each other, as do the right's.
    fn left(self) -> bool {
        matches!(self, Self::Examination | Self::Navigation | Self::Option)
    }
}

#[derive(Debug, Clone)]
struct Child {
    child: ToolbarChild,
    field: Option<usize>,
    button: u32,
    close: u32,
}

/// `gmToolbarUI`'s children, and the mount and sale buttons on its field.
#[derive(Debug, Clone)]
pub struct ToolbarUi {
    children: Vec<Child>,
    /// `ToolbarField` (the toolbar's root) and its `PrecludeHeight`.
    field: Option<usize>,
    /// `DismountButton` and `ConsignmentsButton`: hidden until a mount or a sale needs them.
    dismount: Option<usize>,
    consignments: Option<usize>,
}

impl ToolbarUi {
    /// `gmToolbarUI::SetupChildren`: each child finds its field (`IUIManager::GetElement`) and
    /// hides it.
    pub fn create(tree: &mut UiTree) -> Self {
        let children = ToolbarChild::ALL
            .iter()
            .map(|&child| {
                let name = child.name();
                let id = tokenize(&format!("{name}Field"));
                let field = tree
                    .roots()
                    .iter()
                    .find_map(|&root| tree.child_recursive(root, id));
                Child {
                    child,
                    field,
                    button: tokenize(&format!("{name}Button")),
                    close: tokenize(&format!("{name}CloseButton")),
                }
            })
            .collect();
        let field = tree.roots().iter().copied().find(|&r| {
            tree.get(r)
                .is_some_and(|e| e.id() == tokenize("ToolbarField"))
        });
        let button = |name: &str| field.and_then(|f| tree.child_recursive(f, tokenize(name)));
        let toolbar = Self {
            children,
            field,
            dismount: button("DismountButton"),
            consignments: button("ConsignmentsButton"),
        };
        for child in ToolbarChild::ALL {
            toolbar.set_field(tree, child, false);
        }
        toolbar
    }

    fn get(&self, child: ToolbarChild) -> &Child {
        self.children
            .iter()
            .find(|c| c.child == child)
            .expect("every child")
    }

    fn set_field(&self, tree: &mut UiTree, child: ToolbarChild, shown: bool) {
        if let Some(field) = self.get(child).field {
            if tree.get(field).is_some_and(|e| e.visible != shown) {
                tree.set_visible(field, shown);
            }
        }
    }

    /// Whether a child is shown (its field visible).
    pub fn shown(&self, tree: &UiTree, child: ToolbarChild) -> bool {
        self.get(child)
            .field
            .and_then(|f| tree.get(f))
            .is_some_and(|e| e.visible)
    }

    /// The child's `Show`, then `ManageChildren`: showing one hides the others on its side
    /// (`gmExaminationUI::RecvNotice_OpenExaminationPanel` shows the examination panel).
    pub fn show(&self, tree: &mut UiTree, child: ToolbarChild, shown: bool) {
        self.set_field(tree, child, shown);
        if shown {
            for other in ToolbarChild::ALL {
                if other != child && other.left() == child.left() {
                    self.set_field(tree, other, false);
                }
            }
        }
    }

    /// `gmToolbarUI::RecvNotice_ShowSaleReminder`: the consignments button shows in its
    /// consignments state, or hides in the stateless one.
    pub fn sale_reminder(&self, tree: &mut UiTree, show: bool) {
        button_state(tree, self.consignments, "consignments", show);
    }

    /// `gmToolbarUI::RecvNotice_UpdateMountState`: the dismount button shows in its dismount
    /// state while the player is on a mount.
    pub fn mount_state(&self, tree: &mut UiTree, on_mount: bool) {
        button_state(tree, self.dismount, "dismount", on_mount);
    }

    /// `gmToolbarUI::ListenToMessage`: the dismount button's release (`DismountButton`,
    /// `mouse_release_on`) calls `gmCWInterface::Dismount`. True for that message.
    pub fn is_dismount_press(&self, message: &UiMessage) -> bool {
        self.dismount.is_some()
            && message.element_id == tokenize("DismountButton")
            && message.message == MSG_BUTTON_RELEASE_ON
    }

    /// Whether the toolbar bar is shown (its `ToolbarField` is visible; `gmToolbarUI::Shown`).
    pub fn bar_shown(&self, tree: &UiTree) -> bool {
        self.field
            .and_then(|f| tree.get(f))
            .is_some_and(|e| e.visible)
    }

    /// `gmToolbarUI::Show`'s part: the toolbar field shows or hides, and the bottom preclude the
    /// reticle keeps is its `PrecludeHeight` when it shows (0 when it hides).
    pub fn show_bar(&self, tree: &mut UiTree, shown: bool) -> i32 {
        let Some(field) = self.field else {
            return 0;
        };
        tree.set_visible(field, shown);
        match tree
            .get(field)
            .and_then(|e| e.attribute(tokenize("precludeheight")))
        {
            Some(AttributeValue::Int(height)) if shown => *height,
            _ => 0,
        }
    }

    /// `gmToolbarUI::CleanupChildren`'s part here: the children's fields hide (their frameworks
    /// are not made, so there is nothing to delete).
    pub fn cleanup_children(&self, tree: &mut UiTree) {
        for child in ToolbarChild::ALL {
            self.set_field(tree, child, false);
        }
    }

    fn toggle(&self, tree: &mut UiTree, child: ToolbarChild) {
        let shown = self.shown(tree, child);
        self.show(tree, child, !shown);
    }

    /// One UI message: `gmToolbarUI::ListenToMessage`'s child keys and each child's
    /// `ListenToMessage` for its buttons.
    pub fn message(&self, tree: &mut UiTree, message: &UiMessage) {
        if message.element.is_none() {
            if message.message == GLOBAL_KEY_PRESS {
                let key = match message.data as u32 {
                    // Action 0x40000005 is the examination key too.
                    0x4000_0005 => 0x4100_0001,
                    key => key,
                };
                if let Some(&child) = ToolbarChild::ALL.iter().find(|c| c.key() == key) {
                    self.toggle(tree, child);
                }
            }
            return;
        }
        for c in &self.children {
            let release = if c.child == ToolbarChild::Quest {
                MSG_MOUSE_RELEASE
            } else {
                MSG_BUTTON_RELEASE_ON
            };
            if message.element_id == c.button && message.message == release {
                self.toggle(tree, c.child);
            } else if message.element_id == c.close && message.message == MSG_BUTTON_RELEASE_ON {
                self.show(tree, c.child, false);
            }
        }
    }
}

/// A mount or sale button: shown in its `state` (the button is shown first), or hidden in the
/// stateless one (the state first), as `gmToolbarUI`'s two notices do.
fn button_state(tree: &mut UiTree, button: Option<usize>, state: &str, shown: bool) {
    let Some(button) = button else {
        return;
    };
    if shown {
        tree.set_visible(button, true);
        tree.set_state(button, tokenize(state));
    } else {
        tree.set_state(button, tokenize("stateless"));
        tree.set_visible(button, false);
    }
}
