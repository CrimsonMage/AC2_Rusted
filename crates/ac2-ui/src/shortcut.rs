//! `gmShortcutUI`: the shortcut bar over the toolbar (`ShortcutField` of the toolbar layout).
//! Its list box (`ShortcutListBox`) holds one `ShortcutTemplate` per slot of the player's
//! shortcut array, padded to the slots the bar's rows need (`UpdatePadding`); ten slots make a
//! set, and the bar shows one set a row. A slot shows its icon (`InqShortcutInfo`), or for an
//! alias its text, and an empty slot its key number when numbers are on (`MakeElement`).
//!
//! The shortcut keys use a slot of the set shown (`0x4100002B`..`34`, and `0x4100005F`..`68` for
//! the second row), pick a set (`0x41000035`..`3E`) or step through them (`3F` / `40`); the
//! attack key and buttons ask to attack or stop. Pressing a slot uses it, and a right press asks
//! to examine it. Double-clicking a trainable skill's icon puts it in the first empty slot of the
//! set shown. A shortcut whose skill is untrained or whose item is gone is removed
//! (`CheckShortcut`).
//!
//! The notices reach the bar as `check_all_items`, `check_all_skills`, `check_skill` and
//! `check_item` (`RecvNotice_UpdateInventory` / `UpdateContainer` / `UpdateSkill` /
//! `UpdateResetTime`), so each keeps retail's split: a skill's time left, an unusable skill's
//! long meter, and a pending item's look (`CheckItem`'s `_pendingCreate`). An item slot shows
//! the item's icon description (or icon) and its name at its quantity; an item the client does
//! not hold yet waits pending until its object is created (`FIXMESetupItemElement`,
//! `RecvNotice_CreateObject`). Not modelled yet: dragging the bar's edge to change its rows
//! (`UIMESSAGE_GLOBAL_LOOP`'s resize) and the special attacks' drag state.

use std::sync::Arc;

use crate::drag::DragCheck;
use crate::element::UiTree;
use crate::layout::{AttributeValue, LayoutDesc};
use crate::mouse::{UiMessage, GLOBAL_KEY_PRESS};
use crate::token::{tokenize, BUTTON_GHOSTED, MSG_BUTTON_RELEASE_ON, MSG_MOUSE_PRESS_TOP};

/// `ShortcutInfo::_type`.
pub const SHORTCUT_EMPTY: u32 = 0;
pub const SHORTCUT_SKILL: u32 = 0x4000_0001;
pub const SHORTCUT_ITEM: u32 = 0x4000_0002;
pub const SHORTCUT_ALIAS: u32 = 0x4000_0003;
pub const SHORTCUT_RECIPE: u32 = 0x4000_0006;
const SHORTCUT_OBSOLETE: u32 = 0x4000_0004;

/// The slots a bar holds (`index < 100`).
const SLOTS: usize = 100;

/// One shortcut (`ShortcutInfo`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Shortcut {
    pub kind: u32,
    /// `_data_enum`: a skill.
    pub skill: u32,
    /// `_data_iid`: an item.
    pub item: u64,
    /// `_data_dataid`: a recipe.
    pub data_id: u32,
    /// `GetDataStr`: an alias's text.
    pub text: String,
}

/// The bar's options from the player's options profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ShortcutOptions {
    /// `GetWhichShortcutSet`, from 1.
    pub which_set: u32,
    pub show_numbers: bool,
    /// `GetShortcutInstaScroll`.
    pub insta_scroll: bool,
    /// `GetShortcutLocked`: slots stay put unless shift is held (`gmShortcutUI_DOCB`).
    pub locked: bool,
    /// `GetShortcutHeight`: the bar's rows (1 to 5).
    pub rows: i32,
}

/// What the bar asks of the player.
pub trait ShortcutSource {
    /// `gmCWInterface::InqShortcutArray`.
    fn shortcuts(&self) -> Vec<Shortcut>;
    /// `gmCWInterface::InqShortcutInfo`: the slot's icon (0 for none).
    fn icon(&self, index: usize, shortcut: &Shortcut) -> u32;
    /// `gmCWInterface::InqShortcutInfo`: the slot's name, its tooltip (none by default).
    fn tooltip(&self, _index: usize) -> Option<String> {
        None
    }
    fn options(&self) -> ShortcutOptions;
    /// `IsSkillTrainedByEnum`.
    fn skill_trained(&self, skill: u32) -> bool;
    /// `gmCWInterface::InqSkillTimeLeft`: the skill's reset total and time left, when it is on
    /// reset (none by default).
    fn skill_reset(&self, _skill: u32) -> Option<(f64, f64)> {
        None
    }
    /// `DoIHaveItem`.
    fn has_item(&self, item: u64) -> bool;
    /// `CObjectMaint::GetWeenieObject`: the item's icon and name, if the client knows it (none
    /// by default).
    fn item(&self, _item: u64) -> Option<crate::ItemSlot> {
        None
    }
    /// `ID_Item_TT_WaitingForServer` when the UI table holds it (`IsReady`; none by default).
    fn waiting_tooltip(&self) -> Option<String> {
        None
    }
    /// `gmCWInterface::IsSkillUsableBasedOnInventoryByEnum`: a skill the inventory does not
    /// allow is kept on a long meter (`CheckSkill`).
    fn skill_usable(&self, _skill: u32) -> bool {
        true
    }
}

/// What the bar asks of the client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShortcutRequest {
    /// `gmCWInterface::UseShortcut`.
    Use(usize),
    /// `gmCWInterface::SetShortcut` (an empty shortcut removes one).
    Set(usize, Shortcut),
    /// `SetSelection(item)`, `SetExaminationRequest`, `SendNotice_OpenExaminationPanel`.
    Examine { kind: u32, data: u32, item: u64 },
    /// `RequestAttackStateChange`.
    Attack(bool),
    /// `SetWhichShortcutSet`.
    WhichSet(u32),
    /// `CM_Character::SendNotice_SetShortcutHeight`: the bar was resized to these rows.
    Height(u32),
}

fn id(name: &str) -> u32 {
    tokenize(name)
}

/// `gmShortcutUI`'s state.
#[derive(Debug, Clone)]
pub struct ShortcutUi {
    layout: Arc<LayoutDesc>,
    pub(crate) field: Option<usize>,
    pub(crate) list: Option<usize>,
    start_attack: Option<usize>,
    stop_attack: Option<usize>,
    pub(crate) shortcuts: Vec<Shortcut>,
    /// `m_specialAttacks`: attack id to maneuver, while the attack lasts.
    pub(crate) specials: Vec<(u32, u32)>,
    /// `_timeLeftArray`: the skills on reset.
    pub(crate) timers: Vec<crate::shortcut_states::TimeLeft>,
    /// `_pendingCreate`: the item slots that were checked while their item is pending.
    pub(crate) pending: Vec<(u64, usize)>,
    /// The clock (`Timer::cur_time`) at the last `update_time_left`.
    pub(crate) now: f64,
    /// `_offset`: the first slot of the set shown.
    offset: usize,
    show_numbers: bool,
    attacking: bool,
    pub(crate) requests: Vec<ShortcutRequest>,
    /// The field's box after the bar last placed it itself; a different box is the
    /// `location_changed` that sets `_bDirty` (`shortcut_states.rs` `resize_loop`).
    pub(crate) own_box: Option<crate::element::UiBox>,
}

impl ShortcutUi {
    /// `gmShortcutUI::gmShortcutUI`: the bar's root from the toolbar layout and the toolbar's
    /// attack buttons.
    pub fn create(tree: &mut UiTree, layout: &Arc<LayoutDesc>, toolbar_field: usize) -> Self {
        let field = tree.create_root(Arc::clone(layout), id("ShortcutField"));
        let ui = Self {
            layout: Arc::clone(layout),
            field,
            list: field.and_then(|f| tree.child_recursive(f, id("ShortcutListBox"))),
            start_attack: tree.child_recursive(toolbar_field, id("StartAttackButton")),
            stop_attack: tree.child_recursive(toolbar_field, id("StopAttackButton")),
            shortcuts: Vec::new(),
            specials: Vec::new(),
            timers: Vec::new(),
            pending: Vec::new(),
            now: 0.0,
            offset: 0,
            show_numbers: false,
            attacking: false,
            requests: Vec::new(),
            own_box: None,
        };
        ui.check_attack_buttons(tree);
        if let Some(list) = ui.list {
            // `gmShortcutUI::gmShortcutUI`: `gmShortcutUI_DOCB` and `gmShortcutUI_DDCB`.
            tree.set_drag_out_check(list, Some(DragCheck::ShortcutOut));
            tree.set_drag_drop_check(list, Some(DragCheck::ShortcutDrop));
        }
        ui
    }

    /// `gmShortcutUI::Show`: the bar's field shows or hides.
    pub fn show(&self, tree: &mut UiTree, shown: bool) {
        if let Some(field) = self.field {
            tree.set_visible(field, shown);
        }
    }

    pub fn take_requests(&mut self) -> Vec<ShortcutRequest> {
        std::mem::take(&mut self.requests)
    }

    /// The bar's root and list box.
    pub fn elements(&self) -> (Option<usize>, Option<usize>) {
        (self.field, self.list)
    }

    /// The first slot of the set shown.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// `CheckAttackButtons`.
    fn check_attack_buttons(&self, tree: &mut UiTree) {
        if let Some(start) = self.start_attack {
            tree.set_bool(start, BUTTON_GHOSTED, self.attacking);
        }
        if let Some(stop) = self.stop_attack {
            tree.set_bool(stop, BUTTON_GHOSTED, !self.attacking);
        }
    }

    /// `RecvNotice_UpdateAttackState`.
    pub fn update_attack_state(&mut self, tree: &mut UiTree, attacking: bool) {
        self.attacking = attacking;
        self.check_attack_buttons(tree);
    }

    /// `gmShortcutUI::RefreshShortcutHeight`: the bar as tall as its rows (`row_height` each and
    /// `border_size`), its bottom edge kept, and the list given the rows.
    fn refresh_shortcut_height(&mut self, tree: &mut UiTree, rows: i32) {
        let (Some(field), Some(list)) = (self.field, self.list) else {
            return;
        };
        let rows = rows.clamp(1, 5);
        let int =
            |tree: &UiTree, index: usize, name: &str| match tree.get(index)?.attribute(id(name)) {
                Some(AttributeValue::Int(v)) => Some(*v),
                _ => None,
            };
        if int(tree, list, "max_rows") == Some(rows) {
            return;
        }
        let (Some(border), Some(row_height)) = (
            int(tree, field, "border_size"),
            int(tree, field, "row_height"),
        ) else {
            return;
        };
        let height = row_height * rows + border;
        let rect = tree.get(field).map(|e| e.rect).unwrap_or_default();
        let (x, y) = tree.get(field).and_then(|e| e.object).unwrap_or((0, 0));
        tree.resize_to(field, rect.width(), height);
        tree.move_to(field, x, y + rect.height() - height);
        tree.el_mut(list)
            .attributes
            .insert(id("max_rows"), AttributeValue::Int(rows));
        let width = tree.get(list).map_or(0, |e| e.rect.width());
        tree.resize_to(list, width, row_height * rows);
        self.own_box = Some(tree.screen_box(field));
    }

    /// `RecvNotice_ReloadShortcuts` / `Init`: the set from the options, every slot made again.
    pub fn reload(&mut self, tree: &mut UiTree, source: &dyn ShortcutSource) {
        let options = source.options();
        tree.set_shortcut_locked(options.locked);
        self.refresh_shortcut_height(tree, options.rows);
        self.offset = (options.which_set as usize * 10).saturating_sub(10);
        self.show_numbers = options.show_numbers;
        let Some(list) = self.list else {
            return;
        };
        let offset = self.offset;
        tree.list_flush(list);
        tree.list_home(list);
        self.offset = offset;
        self.pending.clear();
        self.shortcuts = source.shortcuts();
        tree.list_multi_op_start(list);
        for index in 0..self.shortcuts.len() {
            let shortcut = self.shortcuts[index].clone();
            if let Some(item) = self.make_element(tree, source, index, &shortcut) {
                tree.list_add_item(list, item);
            }
        }
        self.update_padding(tree, source);
        tree.list_multi_op_complete(list);
        for index in 0..self.shortcuts.len() {
            let shortcut = self.shortcuts[index].clone();
            self.check_shortcut(tree, source, index, &shortcut);
        }
        tree.list_scroll_to_row(list, (self.offset / 10) as i32);
        self.update_set_number(tree);
    }

    /// `gmShortcutUI::MakeElement`.
    fn make_element(
        &mut self,
        tree: &mut UiTree,
        source: &dyn ShortcutSource,
        index: usize,
        shortcut: &Shortcut,
    ) -> Option<usize> {
        let template = self.layout.element(id("ShortcutTemplate")).cloned()?;
        let item = tree.create_element(Arc::clone(&self.layout), template, false)?;
        let alias = tree.child_recursive(item, id("AliasText"));
        if let Some(alias) = alias {
            tree.clear_text(alias);
        }
        let attrs = &mut tree.el_mut(item).attributes;
        attrs.insert(id("Token_ShortcutType"), AttributeValue::Id(shortcut.kind));
        attrs.insert(id("ShortcutIndex"), AttributeValue::Int(index as i32));
        tree.set_bool(item, id("dragable"), shortcut.kind != SHORTCUT_EMPTY);
        if let Some(number) = tree.child_recursive(item, id("NumberDisplay")) {
            tree.clear_text(number);
            if shortcut.kind == SHORTCUT_EMPTY && self.show_numbers {
                tree.set_text(number, &((index + 1) % 10).to_string());
            }
        }
        let icon = tree.child_recursive(item, id("ShortcutIcon"))?;
        if index >= SLOTS {
            if let Some(button) = tree.child_recursive(item, id("ShortcutButton")) {
                tree.set_bool(button, BUTTON_GHOSTED, true);
                tree.set_bool(button, id("broadcast_when_ghosted"), false);
            }
            return Some(item);
        }
        let data = match shortcut.kind {
            SHORTCUT_SKILL | SHORTCUT_EMPTY => Some(AttributeValue::Id(shortcut.skill)),
            SHORTCUT_RECIPE => Some(AttributeValue::Id(shortcut.data_id)),
            SHORTCUT_ITEM => Some(AttributeValue::InstanceId(shortcut.item)),
            _ => None,
        };
        if let Some(data) = data {
            tree.el_mut(item)
                .attributes
                .insert(id("Token_ShortcutData"), data);
        }
        if shortcut.kind == SHORTCUT_ALIAS {
            if let Some(alias) = alias {
                tree.set_text(alias, &shortcut.text);
            }
        }
        tree.set_tooltip(item, source.tooltip(index));
        let image = source.icon(index, shortcut);
        if image != 0 {
            tree.add_image_to_state(icon, image, 2, crate::token::STATELESS);
        }
        if shortcut.kind == SHORTCUT_ITEM {
            self.setup_item_element(tree, source, shortcut.item, index, item);
        }
        Some(item)
    }

    /// `gmShortcutUI::FIXMESetupItemElement`: an item the client does not know yet waits for
    /// its object (`_pendingCreate`, `pending`, tooltip `ID_Item_TT_WaitingForServer`; the DVD's
    /// UI table lacks it, so `StartTooltip` shows the template's `tooltip_entry`); a known one shows its icon description, else its icon, and
    /// its name at its quantity, `stateless`.
    fn setup_item_element(
        &mut self,
        tree: &mut UiTree,
        source: &dyn ShortcutSource,
        object: u64,
        index: usize,
        element: usize,
    ) {
        let Some(icon) = tree.child_recursive(element, id("ShortcutIcon")) else {
            return;
        };
        let state = match source.item(object) {
            None => {
                self.pending.push((object, index));
                tree.set_tooltip(element, source.waiting_tooltip());
                "pending"
            }
            Some(slot) => {
                if slot.icon_layers.is_empty() {
                    tree.set_image(icon, slot.icon, 0);
                } else {
                    tree.set_icon_desc(icon, &slot.icon_layers);
                }
                tree.set_tooltip(element, Some(slot.name));
                "stateless"
            }
        };
        tree.set_state(element, id(state));
    }

    /// `gmShortcutUI::RecvNotice_CreateObject`: each slot waiting on `object` is set up again
    /// and no longer waits.
    pub fn create_object(&mut self, tree: &mut UiTree, source: &dyn ShortcutSource, object: u64) {
        if source.item(object).is_none() {
            return;
        }
        let (waiting, rest) = std::mem::take(&mut self.pending)
            .into_iter()
            .partition(|&(item, _)| item == object);
        self.pending = rest;
        for (_, index) in waiting {
            if let Some(element) = self.slot_element(tree, index) {
                self.setup_item_element(tree, source, object, index, element);
            }
        }
    }

    /// `gmShortcutUI::UpdatePadding`: empty slots to `(rows * 5 + 45) * 2`.
    pub(crate) fn update_padding(&mut self, tree: &mut UiTree, source: &dyn ShortcutSource) {
        let Some(list) = self.list else {
            return;
        };
        let rows = match tree.get(list).and_then(|e| e.attribute(id("max_rows"))) {
            Some(AttributeValue::Int(rows)) => *rows,
            _ => 0,
        };
        let total = ((rows * 5 + 45) * 2).max(0) as usize;
        while tree.list_items(list).len() < total {
            let index = tree.list_items(list).len();
            match self.make_element(tree, source, index, &Shortcut::default()) {
                Some(item) => tree.list_add_item(list, item),
                None => break,
            }
        }
        while tree.list_items(list).len() > total {
            let last = tree.list_items(list).len() - 1;
            if let Some(item) = tree.list_remove_item(list, last) {
                tree.delete(item);
            }
        }
    }

    /// `gmShortcutUI::SetShortcut`: the slot made again; `requested` asks the player to keep it.
    fn set_shortcut(
        &mut self,
        tree: &mut UiTree,
        source: &dyn ShortcutSource,
        index: usize,
        shortcut: Shortcut,
        requested: bool,
    ) {
        if requested {
            self.requests
                .push(ShortcutRequest::Set(index, shortcut.clone()));
            if let Some(slot) = self.shortcuts.get_mut(index) {
                *slot = shortcut.clone();
            }
        }
        let Some(list) = self.list else {
            return;
        };
        if let Some(item) = self.make_element(tree, source, index, &shortcut) {
            tree.list_set_item(list, item, index);
        }
    }

    /// `gmShortcutUI::RemoveShortcut`.
    fn remove_shortcut(&mut self, tree: &mut UiTree, source: &dyn ShortcutSource, index: usize) {
        self.set_shortcut(tree, source, index, Shortcut::default(), true);
    }

    /// `gmShortcutUI::CheckShortcut`: a skill is checked by `CheckSkill`, an item by `CheckItem`,
    /// an obsolete entry is removed and a recipe goes when it has no DID (`CheckRecipe`).
    fn check_shortcut(
        &mut self,
        tree: &mut UiTree,
        source: &dyn ShortcutSource,
        index: usize,
        shortcut: &Shortcut,
    ) {
        match shortcut.kind {
            SHORTCUT_SKILL => self.check_skill_slot(tree, source, index, shortcut),
            SHORTCUT_ITEM => {
                if source.has_item(shortcut.item) {
                    self.clear_pending_state(tree, index, shortcut.item);
                } else {
                    self.remove_shortcut(tree, source, index);
                }
            }
            SHORTCUT_OBSOLETE => self.remove_shortcut(tree, source, index),
            SHORTCUT_RECIPE if shortcut.data_id == 0 => {
                self.remove_shortcut(tree, source, index);
            }
            _ => {}
        }
    }

    /// `gmShortcutUI::CheckSkill` (the slot's): an untrained skill is removed; an unusable one
    /// shows a long meter; a skill on reset keeps its timer, and one ready again has its timer
    /// ended (`UpdateTimeLeft` drops it on the next frame).
    fn check_skill_slot(
        &mut self,
        tree: &mut UiTree,
        source: &dyn ShortcutSource,
        index: usize,
        shortcut: &Shortcut,
    ) {
        let skill = shortcut.skill;
        if !source.skill_trained(skill) {
            self.remove_shortcut(tree, source, index);
            return;
        }
        let usable = source.skill_usable(skill);
        let (total, left) = match (usable, source.skill_reset(skill)) {
            // `InqSkillTimeLeft` reports no reset for a usable skill: its entry ends.
            (true, None) => {
                if let Some(at) = self.time_left_position(skill, index) {
                    self.timers[at].end = 0.0;
                }
                return;
            }
            (true, Some(reset)) => reset,
            // `1e6` seconds (`0x49742400`) for a skill the inventory does not allow.
            (false, _) => (f64::from(1.0e6_f32), f64::from(1.0e6_f32)),
        };
        self.reset_timer(tree, index, skill, total, left);
    }

    /// `CheckItem(index, ...)`'s pending part: an item slot whose item was pending is stateless
    /// again once the item is held.
    fn clear_pending_state(&mut self, tree: &mut UiTree, index: usize, item: u64) {
        if !self.pending.iter().any(|&(pending, _)| pending == item) {
            return;
        }
        if let Some(slot) = self.slot_element(tree, index) {
            tree.set_state(slot, id("stateless"));
        }
    }

    /// The bar's item element for `index`.
    fn slot_element(&self, tree: &UiTree, index: usize) -> Option<usize> {
        self.list
            .and_then(|list| tree.list_items(list).get(index).copied())
    }

    /// `gmShortcutUI::CheckAllItems`: the item slots checked.
    pub fn check_all_items(&mut self, tree: &mut UiTree, source: &dyn ShortcutSource) {
        self.check_matching(tree, source, |s| s.kind == SHORTCUT_ITEM);
    }

    /// `gmShortcutUI::CheckAllSkills`: the skill slots checked.
    pub fn check_all_skills(&mut self, tree: &mut UiTree, source: &dyn ShortcutSource) {
        self.check_matching(tree, source, |s| s.kind == SHORTCUT_SKILL);
    }

    /// `gmShortcutUI::CheckSkill(skill)`: the slots holding that skill checked.
    pub fn check_skill(&mut self, tree: &mut UiTree, source: &dyn ShortcutSource, skill: u32) {
        self.check_matching(tree, source, |s| {
            s.kind == SHORTCUT_SKILL && s.skill == skill
        });
    }

    /// `gmShortcutUI::CheckItem(item, setPending)`: the slots holding `item` checked, then each
    /// is pending (`pending` state) or stateless, and remembered as pending.
    pub fn check_item(
        &mut self,
        tree: &mut UiTree,
        source: &dyn ShortcutSource,
        item: u64,
        set_pending: bool,
    ) {
        for index in 0..self.shortcuts.len() {
            let shortcut = self.shortcuts[index].clone();
            if shortcut.kind == SHORTCUT_ITEM && shortcut.item == item {
                self.check_shortcut(tree, source, index, &shortcut);
                if let Some(slot) = self.slot_element(tree, index) {
                    self.pending.push((item, index));
                    let state = if set_pending { "pending" } else { "stateless" };
                    tree.set_state(slot, id(state));
                }
            }
        }
    }

    /// The loops of `CheckAllItems` and `CheckAllSkills`: the slots `keep` selects.
    fn check_matching(
        &mut self,
        tree: &mut UiTree,
        source: &dyn ShortcutSource,
        keep: impl Fn(&Shortcut) -> bool,
    ) {
        for index in 0..self.shortcuts.len() {
            let shortcut = self.shortcuts[index].clone();
            if keep(&shortcut) {
                self.check_shortcut(tree, source, index, &shortcut);
            }
        }
    }

    /// `gmShortcutUI::UpdateSetNumber`: the set's number shown and kept.
    fn update_set_number(&mut self, tree: &mut UiTree) {
        let set = self.offset / 10 + 1;
        let display = tree
            .roots()
            .iter()
            .find_map(|&r| tree.child_recursive(r, id("SetNumberDisplay")));
        if let Some(display) = display {
            if self.show_numbers {
                tree.set_text(display, &set.to_string());
            } else {
                tree.clear_text(display);
            }
            self.requests.push(ShortcutRequest::WhichSet(set as u32));
        }
    }

    fn show_set(&mut self, tree: &mut UiTree, set: usize) {
        if let Some(list) = self.list {
            tree.list_scroll_to_row(list, set as i32);
        }
    }

    /// `gmShortcutUI::FromElement`: the shortcut a (dragged) element carries.
    fn from_element(tree: &UiTree, element: usize) -> Option<Shortcut> {
        let el = tree.get(element)?;
        let kind = match el.attribute(id("Token_ShortcutType")) {
            Some(AttributeValue::Id(kind)) => *kind,
            _ => return None,
        };
        let data = el.attribute(id("Token_ShortcutData"));
        let mut shortcut = Shortcut {
            kind,
            ..Shortcut::default()
        };
        match (kind, data) {
            (SHORTCUT_SKILL, Some(AttributeValue::Id(skill))) => shortcut.skill = *skill,
            (SHORTCUT_ITEM, Some(AttributeValue::InstanceId(item))) => shortcut.item = *item,
            (SHORTCUT_RECIPE, Some(AttributeValue::Id(did))) => shortcut.data_id = *did,
            (SHORTCUT_ALIAS, _) => {
                let alias = tree.child_recursive(element, id("AliasText"))?;
                shortcut.text = tree.get(alias)?.text().unwrap_or_default();
            }
            (SHORTCUT_SKILL | SHORTCUT_ITEM | SHORTCUT_RECIPE, _) => return None,
            _ => {}
        }
        Some(shortcut)
    }

    /// `ShortcutIndex` of a bar slot (or its copy).
    fn shortcut_index(tree: &UiTree, element: usize) -> Option<usize> {
        match tree.get(element)?.attribute(id("ShortcutIndex")) {
            Some(AttributeValue::Int(index)) => usize::try_from(*index).ok(),
            _ => None,
        }
    }

    /// `ListenToMessage`'s `stop_drag_item` on the list: a drop puts the dragged shortcut in the
    /// slot under the pointer (`gmShortcutUI_DDCB`: one carrying a shortcut, on a slot below 100)
    /// and empties the slot it came from; a bar shortcut dragged off the bar is removed.
    fn drop(&mut self, tree: &mut UiTree, source: &dyn ShortcutSource, message: &UiMessage) {
        let (Some(list), Some(copy)) = (self.list, message.item) else {
            return;
        };
        let from = Self::shortcut_index(tree, copy).filter(|&i| i < SLOTS);
        if message.data != 0 {
            let Some(target) = tree.list_item_under_mouse(list) else {
                return;
            };
            let Some(index) = tree.list_what_num(list, target).filter(|&i| i < SLOTS) else {
                return;
            };
            if let Some(from) = from {
                self.remove_shortcut(tree, source, from);
            }
            let Some(shortcut) = Self::from_element(tree, copy) else {
                return;
            };
            self.set_shortcut(tree, source, index, shortcut, true);
        } else if let Some(from) = from {
            self.remove_shortcut(tree, source, from);
        }
    }

    /// The first empty slot of the set shown (`ListenToMessage`'s double-click on a skill).
    fn first_empty_in_set(&self) -> Option<usize> {
        let end = (self.offset + 10).min(self.shortcuts.len());
        (self.offset..end).find(|&i| self.shortcuts[i].kind == SHORTCUT_EMPTY)
    }

    /// `gmShortcutUI::ListenToMessage` (both overloads).
    pub fn message(&mut self, tree: &mut UiTree, source: &dyn ShortcutSource, message: &UiMessage) {
        if message.element.is_none() {
            if message.message == GLOBAL_KEY_PRESS {
                self.key_press(tree, message.data as u32);
            }
            return;
        }
        let element_id = message.element_id;
        let action = message.data as u32;
        if element_id == id("SkillNodeIcon") && message.message == MSG_MOUSE_PRESS_TOP {
            // A double click (`0x40000007`) on a draggable (trained) skill.
            if action != 0x4000_0007 {
                return;
            }
            let Some(element) = message.element else {
                return;
            };
            if !tree.bool_attribute(element, id("dragable")) {
                return;
            }
            let skill = match tree
                .get(element)
                .and_then(|e| e.attribute(id("Token_ShortcutData")))
            {
                Some(AttributeValue::Id(skill)) => *skill,
                _ => return,
            };
            if let Some(index) = self.first_empty_in_set() {
                let shortcut = Shortcut {
                    kind: SHORTCUT_SKILL,
                    skill,
                    ..Shortcut::default()
                };
                self.set_shortcut(tree, source, index, shortcut, true);
            }
            return;
        }
        if element_id == id("ShortcutListBox") && message.message == crate::drag::MSG_STOP_DRAG_ITEM
        {
            self.drop(tree, source, message);
            return;
        }
        if self.list.is_none() || message.message != MSG_BUTTON_RELEASE_ON {
            if element_id == id("ShortcutListBox") && message.message == id("new_view") {
                self.new_view(tree);
            }
            return;
        }
        if element_id == id("StartAttackButton") {
            self.requests.push(ShortcutRequest::Attack(true));
        } else if element_id == id("StopAttackButton") {
            self.requests.push(ShortcutRequest::Attack(false));
        } else if element_id == id("ShortcutButton") {
            let Some(slot) = message.element.and_then(|e| tree.get(e)?.parent) else {
                return;
            };
            let Some(index) = self.list.and_then(|l| tree.list_what_num(l, slot)) else {
                return;
            };
            if index >= SLOTS {
                return;
            }
            if action == 0x4000_0004 {
                self.requests.push(ShortcutRequest::Use(index));
            } else if action == 0x4000_0005 {
                let shortcut = self.shortcuts.get(index).cloned().unwrap_or_default();
                let request = match shortcut.kind {
                    SHORTCUT_SKILL => Some((0x4000_0003, shortcut.skill, 0)),
                    SHORTCUT_ITEM => Some((0x4000_0002, 0, shortcut.item)),
                    SHORTCUT_RECIPE => Some((0x4000_0007, shortcut.data_id, 0)),
                    _ => None,
                };
                if let Some((kind, data, item)) = request {
                    self.requests
                        .push(ShortcutRequest::Examine { kind, data, item });
                }
            }
        }
    }

    /// `ListenToMessage`'s `new_view`: the set at the list's corner becomes the one shown.
    fn new_view(&mut self, tree: &mut UiTree) {
        let Some(list) = self.list else {
            return;
        };
        let corner = tree.screen_box(list);
        self.offset = tree.list_index_at(list, corner.x0, corner.y0).unwrap_or(0);
        self.update_set_number(tree);
    }

    /// `ListenToMessage`'s key presses.
    fn key_press(&mut self, tree: &mut UiTree, key: u32) {
        match key {
            0x4100_0013 => self.requests.push(ShortcutRequest::Attack(!self.attacking)),
            0x4100_002B..=0x4100_0034 => {
                let index = self.offset + (key - 0x4100_002B) as usize;
                self.requests.push(ShortcutRequest::Use(index));
            }
            0x4100_0035..=0x4100_003E => self.show_set(tree, (key - 0x4100_0035) as usize),
            0x4100_003F | 0x4100_0040 => {
                let step: i64 = if key == 0x4100_003F { 1 } else { -1 };
                let set = (self.offset / 10) as i64 + step;
                let set = if set < 0 {
                    9
                } else if set > 9 {
                    0
                } else {
                    set
                };
                self.show_set(tree, set as usize);
            }
            0x4100_005F..=0x4100_0068 => {
                let index = self.offset + 10 + (key - 0x4100_005F) as usize;
                let rows = self
                    .list
                    .and_then(|l| tree.get(l))
                    .and_then(|e| match e.attribute(id("max_rows")) {
                        Some(AttributeValue::Int(rows)) => Some(*rows),
                        _ => None,
                    })
                    .unwrap_or(0);
                if index < SLOTS && rows > 1 {
                    self.requests.push(ShortcutRequest::Use(index));
                }
            }
            _ => {}
        }
    }
}
