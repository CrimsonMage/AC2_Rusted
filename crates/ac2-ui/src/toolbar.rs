//! `gmToolbarUI`'s panel children (`SetupChildren`): examination, navigation and options on the
//! left side panel, inventory, quests, skills and social on the right. Each child's constructor
//! hides its field (`gmSkillUI::Show(0)` and the like: `Show` sets `XxxField` visible as the
//! child is shown); its toolbar button (`XxxButton`, `mouse_release_on`; the quest button is a
//! field and sends `mouse_release`) toggles it, its close button (`XxxCloseButton`) hides it, and
//! the child keys (`0x41000001`..`07`, `gmToolbarUI::ChildKeyPress`) toggle it.
//! `ManageChildren` hides the children that share a side panel when one shows (their
//! `MExList`).
//!
//! Each child's button carries its tooltip (`ManageChildren`): `ID_Toolbar_TT_<child>` with the
//! `ID_Toolbar_TT_Hide` text while the child shows and `ID_Toolbar_TT_Show` while it is hidden,
//! refreshed when a child's shown state changes ([`ToolbarUi::refresh_tooltips`]).
//!
//! In the modern style the toolbar's look is the client's own (`toolbar_cluster.rs`): the same
//! elements laid out as a corner cluster whose groups open under the pointer
//! ([`ToolbarUi::arrange`]). The classic style keeps retail's bar.
//!
//! Not modelled yet: the panels' contents (each child's own frameworks), the help buttons
//! (`KeyStone::OpenHelp`), the key stone and plug-in keys.

use std::cell::Cell;

use crate::element::UiTree;
use crate::layout::AttributeValue;
use crate::mouse::{UiMessage, GLOBAL_KEY_PRESS, GLOBAL_MOUSE_DOWN, GLOBAL_MOUSE_MOVE};
use crate::skill_tree::SkillVar;
use crate::token::{tokenize, MSG_BUTTON_RELEASE_ON, MSG_MOUSE_RELEASE};
use crate::toolbar_cluster::{hover, Cluster};
use crate::ui_string::{UiString, UiStrings};

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

    /// The child a panel name names (`Inventory`, `skill`, ...; any case), as tools pass it.
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|child| child.name().eq_ignore_ascii_case(name))
    }

    pub(crate) fn name(self) -> &'static str {
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
    /// The toolbar button's element (`ManageChildren`'s tooltip target).
    button_element: Option<usize>,
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
    /// Each child's shown state when its tooltip was last set (`None`: not yet).
    tooltip_shown: Cell<[Option<bool>; 7]>,
    /// The field's parts as the corner cluster lays them out, and its open groups (left, right).
    cluster: Option<Cluster>,
    open: Cell<[bool; 2]>,
    /// The shortcut bar's screen box when the cluster was last laid out beside it.
    shortcut_box: Cell<Option<crate::element::UiBox>>,
}

impl ToolbarUi {
    /// `gmToolbarUI::SetupChildren`: each child finds its field (`IUIManager::GetElement`) and
    /// hides it.
    pub fn create(tree: &mut UiTree) -> Self {
        Self::create_styled(tree, crate::HudStyle::Classic)
    }

    /// [`Self::create`] in `style`: the modern style lays the same elements out as the corner
    /// cluster (`toolbar_cluster.rs`); the classic one keeps retail's bar.
    pub fn create_styled(tree: &mut UiTree, style: crate::HudStyle) -> Self {
        let field = tree.roots().iter().copied().find(|&r| {
            tree.get(r)
                .is_some_and(|e| e.id() == tokenize("ToolbarField"))
        });
        let button = |name: &str| field.and_then(|f| tree.child_recursive(f, tokenize(name)));
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
                    button_element: button(&format!("{name}Button")),
                    close: tokenize(&format!("{name}CloseButton")),
                }
            })
            .collect();
        let dismount = button("DismountButton");
        let consignments = button("ConsignmentsButton");
        let cluster = field
            .filter(|_| style == crate::HudStyle::Modern)
            .map(|field| Cluster::adopt(tree, field));
        let toolbar = Self {
            children,
            field,
            dismount,
            consignments,
            tooltip_shown: Cell::new([None; 7]),
            cluster,
            open: Cell::new([false; 2]),
            shortcut_box: Cell::new(None),
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
        self.arrange(tree);
    }

    /// `gmToolbarUI::RecvNotice_ShowSaleReminder`: the consignments button shows in its
    /// consignments state, or hides in the stateless one.
    pub fn sale_reminder(&self, tree: &mut UiTree, show: bool) {
        button_state(tree, self.consignments, "consignments", show);
        self.arrange(tree);
    }

    /// `gmToolbarUI::RecvNotice_UpdateMountState`: the dismount button shows in its dismount
    /// state while the player is on a mount.
    pub fn mount_state(&self, tree: &mut UiTree, on_mount: bool) {
        button_state(tree, self.dismount, "dismount", on_mount);
        self.arrange(tree);
    }

    /// The toolbar laid out for the display as it is (after `UiTree::set_display`, or when the
    /// shortcut bar moved): the corner cluster with its open groups, the parts beside the
    /// shortcut bar, the status buttons shown.
    pub fn arrange(&self, tree: &mut UiTree) {
        if let Some(cluster) = &self.cluster {
            cluster.arrange(tree, self.open.get());
        }
        self.light(tree);
    }

    /// The modern cluster laid out again when the shortcut bar's box differs from the one it was
    /// last laid out beside (the bar's rows, options and saved place move it after the build).
    pub fn follow_shortcut_bar(&self, tree: &mut UiTree) {
        if self.cluster.is_none() {
            return;
        }
        let bar = tree
            .roots()
            .iter()
            .copied()
            .find(|&r| {
                tree.get(r)
                    .is_some_and(|e| e.id() == tokenize("ShortcutField"))
            })
            .map(|r| tree.screen_box(r));
        if bar != self.shortcut_box.get() {
            self.shortcut_box.set(bar);
            self.arrange(tree);
        }
    }

    /// Each shown child's button lit: drawn in its rollover art while its panel shows (the
    /// client's own cue; retail's buttons looked the same either way).
    fn light(&self, tree: &mut UiTree) {
        if self.cluster.is_none() {
            return;
        }
        for c in &self.children {
            let shown = self.shown(tree, c.child);
            if let Some(button) = c.button_element {
                crate::toolbar_cluster::light(tree, button, shown);
            }
        }
    }

    /// The groups open (left panel's, right panel's).
    pub fn open_groups(&self) -> [bool; 2] {
        self.open.get()
    }

    /// The pointer moved or pressed: the group under it opens, and both fold once it leaves the
    /// plate ([`crate::toolbar_cluster::hover`]).
    fn pointer(&self, tree: &mut UiTree) {
        let Some(cluster) = &self.cluster else {
            return;
        };
        let open = self.open.get();
        let next = hover(&cluster.plan(tree, open), open, cluster.pointer(tree));
        if next != open {
            self.open.set(next);
        }
        // Laid out on every move, so a new display size or a moved shortcut bar is followed too.
        self.arrange(tree);
    }

    /// `gmToolbarUI::ListenToMessage`: the dismount button's release (`DismountButton`,
    /// `mouse_release_on`) calls `gmCWInterface::Dismount`. True for that message.
    pub fn is_dismount_press(&self, message: &UiMessage) -> bool {
        self.dismount.is_some()
            && message.element_id == tokenize("DismountButton")
            && message.message == MSG_BUTTON_RELEASE_ON
    }

    /// `gmToolbarUI::ListenToMessage`: the consignments button's release is the sale reminder's
    /// press, which calls `gmCWInterface::DisplaySaleReminders` (no state gate in retail; the
    /// button is only shown while a sale needs it). True for that message.
    pub fn is_sale_reminder_press(&self, message: &UiMessage) -> bool {
        self.consignments.is_some()
            && message.element_id == tokenize("ConsignmentsButton")
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

    /// `gmToolbarUI::ManageChildren`'s tooltips: each child's button names its panel and says
    /// whether a click hides it (`ID_Toolbar_TT_Hide`, shown) or shows it. Set again only when
    /// a child's shown state moved since the last call.
    pub fn refresh_tooltips(&self, tree: &mut UiTree, strings: &dyn UiStrings) {
        let mut last = self.tooltip_shown.get();
        for (index, c) in self.children.iter().enumerate() {
            let shown = self.shown(tree, c.child);
            if last[index] == Some(shown) {
                continue;
            }
            last[index] = Some(shown);
            let Some(button) = c.button_element else {
                continue;
            };
            let action = if shown {
                "ID_Toolbar_TT_Hide"
            } else {
                "ID_Toolbar_TT_Show"
            };
            let string = UiString::ui(
                tokenize(&format!("ID_Toolbar_TT_{}", c.child.name())),
                vec![(
                    tokenize("value"),
                    SkillVar::Info(Box::new(UiString::ui(tokenize(action), Vec::new()))),
                )],
            );
            tree.set_tooltip(button, Some(strings.render_string(&string)));
        }
        self.tooltip_shown.set(last);
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
            if message.message == GLOBAL_MOUSE_MOVE || message.message == GLOBAL_MOUSE_DOWN {
                self.pointer(tree);
            }
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
