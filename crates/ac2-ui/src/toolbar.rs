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
//! In the modern style the same elements are laid out as the design's strip and dock
//! (`modern_toolbar.rs`, [`ToolbarUi::follow`]); the classic style keeps retail's bar.
//!
//! Not modelled yet: the panels' contents (each child's own frameworks), the help buttons
//! (`KeyStone::OpenHelp`), the key stone and plug-in keys.

use std::cell::Cell;

use crate::element::UiTree;
use crate::layout::AttributeValue;
use crate::mouse::{UiMessage, GLOBAL_KEY_PRESS, GLOBAL_MOUSE_DOWN, GLOBAL_MOUSE_MOVE};
use crate::skill_tree::SkillVar;
use crate::token::{tokenize, MSG_BUTTON_RELEASE_ON, MSG_MOUSE_RELEASE};
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
    /// The modern style's layout of the same elements (`modern_toolbar.rs`), and what it was last
    /// laid out for.
    dock: Option<crate::modern_toolbar::Dock>,
    /// The modern inventory drawer (`modern_inventory`), laid out by the display.
    drawer: Option<crate::modern_inventory::Drawer>,
    laid_out: Cell<Option<Layout>>,
}

/// What the modern toolbar is laid out for: the panels shown, the fan open, the field's and the
/// shortcut bar's boxes, the status buttons shown.
type Layout = (
    [bool; 7],
    bool,
    crate::element::UiBox,
    Option<crate::element::UiBox>,
    [bool; 2],
);

impl ToolbarUi {
    /// `gmToolbarUI::SetupChildren`: each child finds its field (`IUIManager::GetElement`) and
    /// hides it.
    pub fn create(tree: &mut UiTree) -> Self {
        Self::create_styled(tree, crate::HudStyle::Classic)
    }

    /// [`Self::create`] in `style`: the modern style lays the same elements out as the strip and
    /// the dock (`modern_toolbar.rs`); the classic one keeps retail's bar.
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
        let toolbar = Self {
            children,
            field,
            dismount: button("DismountButton"),
            consignments: button("ConsignmentsButton"),
            tooltip_shown: Cell::new([None; 7]),
            dock: None,
            drawer: None,
            laid_out: Cell::new(None),
        };
        for child in ToolbarChild::ALL {
            toolbar.set_field(tree, child, false);
        }
        let mut toolbar = toolbar;
        if style == crate::HudStyle::Modern {
            toolbar.dock = field.map(|field| crate::modern_toolbar::Dock::adopt(tree, field));
            toolbar.follow(tree);
        }
        toolbar
    }

    /// The modern toolbar laid out again when what it is laid out for changed (a panel shown or
    /// hidden, the fan opened or folded, the display resized, the shortcut bar moved, a status
    /// button shown or hidden). Run every frame; nothing in the classic style.
    pub fn follow(&self, tree: &mut UiTree) {
        if let Some(drawer) = &self.drawer {
            drawer.follow(tree);
        }
        let Some(dock) = &self.dock else {
            return;
        };
        let shown = ToolbarChild::ALL.map(|c| self.shown(tree, c));
        let bar = tree
            .roots()
            .iter()
            .copied()
            .find(|&r| {
                tree.get(r)
                    .is_some_and(|e| e.id() == tokenize("ShortcutField"))
            })
            .map(|r| tree.screen_box(r));
        let field = self.field.map(|f| tree.screen_box(f)).unwrap_or_default();
        let visible = |e: Option<usize>| e.and_then(|e| tree.get(e)).is_some_and(|e| e.visible);
        let now = (
            shown,
            dock.is_open(),
            field,
            bar,
            [visible(self.consignments), visible(self.dismount)],
        );
        if self.laid_out.get() != Some(now) {
            dock.arrange(tree, shown);
            // The layout may move the shortcut bar: what it was laid out for is taken after.
            let bar = tree
                .roots()
                .iter()
                .copied()
                .find(|&r| {
                    tree.get(r)
                        .is_some_and(|e| e.id() == tokenize("ShortcutField"))
                })
                .map(|r| tree.screen_box(r));
            self.laid_out.set(Some((now.0, now.1, now.2, bar, now.4)));
        }
    }

    /// The modern drawer's own parts set for the backpack (every frame; nothing in classic).
    pub(crate) fn drawer_tick(
        &self,
        tree: &mut UiTree,
        backpack: &crate::BackpackUi,
        doll: Option<&crate::PaperDollUi>,
        source: &dyn crate::item_display::InventorySource,
    ) {
        if let Some(drawer) = &self.drawer {
            drawer.tick(tree, backpack, doll, source);
        }
    }

    /// A press on the modern drawer's item ring: the action and the item (`modern_item_ring`).
    pub(crate) fn drawer_ring_press(
        &self,
        message: &UiMessage,
    ) -> Option<(crate::modern_item_ring::RingAction, u64)> {
        if message.message != crate::token::MSG_MOUSE_PRESS_TOP {
            return None;
        }
        self.drawer.as_ref()?.ring_press(message.element?)
    }

    /// The modern inventory drawer, laid out with the toolbar.
    pub(crate) fn set_drawer(&mut self, drawer: Option<crate::modern_inventory::Drawer>) {
        self.drawer = drawer;
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
            // A wheel turn over the modern drawer's All view scrolls it.
            if message.message == GLOBAL_MOUSE_DOWN {
                if let Some(drawer) = &self.drawer {
                    if drawer.wheel(tree, message.data as u32) {
                        return;
                    }
                }
            }
            // The modern dock's fan follows the pointer; a press on its orb pins it.
            if let Some(dock) = &self.dock {
                let pressed = message.message == GLOBAL_MOUSE_DOWN;
                if (message.message == GLOBAL_MOUSE_MOVE || pressed)
                    && dock.pointer_at(tree, pressed)
                {
                    self.follow(tree);
                }
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
        // A press on the modern drawer's own parts (its filter chips).
        if message.message == crate::token::MSG_MOUSE_PRESS_TOP {
            if let (Some(drawer), Some(element)) = (&self.drawer, message.element) {
                if drawer.press(tree, element) {
                    return;
                }
            }
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
