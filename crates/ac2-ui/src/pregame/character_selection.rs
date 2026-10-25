//! `gmCharacterSelectionUI`: the account's characters, by name in `CharacterListBox` and as
//! figures standing on the arms of the 3D scene in `Viewport`, with enter, create, delete,
//! options, credits and quit buttons.
//!
//! `Update` lists the character set (`list_box_item`, or `deleted_list_box_item` for one pending
//! deletion), makes one arm per character slot (the allowed count, 20 when unlimited, at least
//! the characters listed) with each character on its arm, enables create while a slot is free and
//! selects the slot character generation left (`_genSlot`). Selecting a character turns the arms
//! to it and enables enter and delete, or restore for one pending deletion. Entering sends
//! `Proto_UI::SendEnterWorld`, shows "please wait" and queues gameplay; create queues avatar
//! selection; delete asks for confirmation first (`MakeDeleteDialog`, `DoDelete`). A press on
//! the scene picks the figure under the pointer (`Start3DSelection`, `Finish3DSelection`); a
//! double click enters with it.
//!
//! Not modelled yet: the 3D scene itself (the client renders [`CharacterSelectionUi::occupants`]
//! on [`CharacterSelectionUi::arms`] and answers the pick), the name's ellipsis
//! (`TruncateText`), the background music and the portal and torch viewports' scenes.

use ac2_net::protocol::CharacterIdentity;

use crate::dialog::{DialogOutcome, DIALOG_CONFIRMATION, DIALOG_WAIT};
use crate::element::UiTree;
use crate::layout::AttributeValue;
use crate::list_box_select::{CLICK_SELECT, MSG_NEW_SELECTION};
use crate::mouse::UiMessage;
use crate::pregame::arms::CharacterArms;
use crate::pregame::flow::{MODE_AVATAR_SELECTION, MODE_CREDITS, MODE_GAMEPLAY, MODE_KEYBOARD};
use crate::pregame::persistent::{greyed_out_for, identity, set_greyed_out_for};
use crate::pregame::strings::{int_var, text_var};
use crate::pregame::{Ctx, PregameRequest, LAYOUT_CHARACTER_SELECTION};
use crate::token::{tokenize, BUTTON_GHOSTED, MSG_BUTTON_RELEASE_ON, MSG_MOUSE_PRESS_TOP};

const LEFT: u32 = 0x4000_0004;
const DOUBLE: u32 = 0x4000_0007;

fn id(name: &str) -> u32 {
    tokenize(name)
}

/// What stands on an arm (`AddPhysObjToArm`): a character's figure from its appearance
/// (`VisualDesc::GetTotalVisualDesc`, `makePartGroupObject`), or the viewport's `nologininfo`
/// object when the appearance cannot be made.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArmOccupant {
    /// The character's slot in the set (and its arm).
    pub slot: usize,
    /// `nologininfo`, the stand-in object.
    pub stand_in: Option<u32>,
}

/// `gmCharacterSelectionUI`'s state.
#[derive(Debug, Clone, Default)]
pub struct CharacterSelectionUi {
    background: Option<usize>,
    enter_field: Option<usize>,
    portal_viewport: Option<usize>,
    torch_viewport: Option<usize>,
    viewport: Option<usize>,
    list_box: Option<usize>,
    arms: CharacterArms,
    occupants: Vec<ArmOccupant>,
    /// `_doSelect`, `_doubleClick`.
    do_select: bool,
    double_click: bool,
    delete_context: u32,
    delete_index: usize,
    wait_context: u32,
    /// The pointer when a pick started (`UIScene::StartSelection`'s point).
    pick: Option<(i32, i32)>,
}

impl CharacterSelectionUi {
    /// `gmCharacterSelectionUI::gmCharacterSelectionUI`: the roots, ghosted buttons, then
    /// `Update`.
    pub(crate) fn create(ctx: &mut Ctx<'_>) -> Self {
        let mut ui = Self::default();
        if let Some(layout) = ctx.assets.layout(LAYOUT_CHARACTER_SELECTION) {
            let mut root = |name: &str| ctx.tree.create_root(layout.clone(), id(name));
            ui.background = root("BackgroundField");
            ui.enter_field = root("EnterButtonField");
            ui.portal_viewport = root("PortalViewport");
            ui.torch_viewport = root("TorchViewport");
            ui.viewport = root("Viewport");
        }
        ui.list_box = ui.child(ctx.tree, ui.background, "CharacterListBox");
        ui.ghost_buttons(ctx.tree, true);
        ui.update(ctx);
        ui
    }

    /// `~gmCharacterSelectionUI`: buttons ghosted, arms and items gone, roots removed, its
    /// dialogs closed.
    pub(crate) fn destroy(mut self, ctx: &mut Ctx<'_>) {
        self.ghost_buttons(ctx.tree, true);
        if let Some(list) = self.list_box {
            ctx.tree.list_flush(list);
        }
        for root in [
            self.background,
            self.enter_field,
            self.portal_viewport,
            self.torch_viewport,
            self.viewport,
        ]
        .into_iter()
        .flatten()
        {
            ctx.tree.remove_root(root);
        }
        ctx.dialogs.close_dialog(ctx.tree, self.delete_context);
        ctx.dialogs.close_dialog(ctx.tree, self.wait_context);
        self.delete_context = 0;
        self.wait_context = 0;
    }

    fn child(&self, tree: &UiTree, root: Option<usize>, name: &str) -> Option<usize> {
        tree.child_recursive(root?, id(name))
    }

    fn set_ghosted(&self, tree: &mut UiTree, root: Option<usize>, name: &str, ghosted: bool) {
        if let Some(button) = self.child(tree, root, name) {
            tree.set_bool(button, BUTTON_GHOSTED, ghosted);
        }
    }

    /// `gmCharacterSelectionUI::GhostButtons`: ghosting ghosts every button; unghosting frees
    /// only options and credits. The list takes clicks while not ghosted.
    fn ghost_buttons(&self, tree: &mut UiTree, ghost: bool) {
        if ghost {
            self.set_ghosted(tree, self.background, "CreateButton", true);
            self.set_ghosted(tree, self.enter_field, "EnterButton", true);
            self.set_ghosted(tree, self.background, "DeleteButton", true);
            self.set_ghosted(tree, self.background, "RestoreButton", true);
        }
        self.free_buttons(tree, ghost);
    }

    /// The tail every path that gives control back shares: options and credits, and the list's
    /// clicks.
    fn free_buttons(&self, tree: &mut UiTree, ghost: bool) {
        self.set_ghosted(tree, self.background, "OptionsButton", ghost);
        self.set_ghosted(tree, self.background, "CreditsButton", ghost);
        if let Some(list) = self.list_box {
            tree.set_bool(list, CLICK_SELECT, !ghost);
        }
    }

    /// `gmCharacterSelectionUI::SetErrorMsg`: the text under the screen.
    pub(crate) fn set_error_msg(&self, tree: &mut UiTree, text: &str) {
        if let Some(error) = self.child(tree, self.background, "ErrorMsg") {
            tree.set_text(error, text);
        }
    }

    fn data_id(&self, tree: &UiTree, element: Option<usize>, name: &str) -> Option<u32> {
        match tree.get(element?)?.attribute(id(name)) {
            Some(AttributeValue::DataId(did)) => Some(*did),
            _ => None,
        }
    }

    /// `gmCharacterSelectionUI::Update`: the list, the arms and the figures from the character
    /// set.
    pub(crate) fn update(&mut self, ctx: &mut Ctx<'_>) {
        let Some(set) = ctx.data.characters().cloned() else {
            return;
        };
        self.set_error_msg(ctx.tree, "");
        self.ghost_buttons(ctx.tree, false);
        let allowed = set.allowed as i32;
        let slots = if allowed < 0 { 20 } else { allowed as usize };
        let slots = slots.max(set.characters.len());
        self.arms.init(slots);
        self.occupants.clear();
        let Some(list) = self.list_box else {
            return;
        };
        ctx.tree.list_flush(list);
        let layout = ctx.assets.layout(LAYOUT_CHARACTER_SELECTION);
        for slot in 0..set.characters.len() {
            let deleted = greyed_out_for(&set, slot) != 0;
            if !deleted && set.characters[slot].id == 0 {
                continue;
            }
            let template = if deleted {
                "deleted_list_box_item"
            } else {
                "list_box_item"
            };
            let Some(item) = layout.and_then(|layout| ctx.tree.create_from(layout, id(template)))
            else {
                continue;
            };
            let name = &set.characters[slot].name;
            ctx.tree
                .set_text(item, if name.is_empty() { "GARBAGE!" } else { name });
            ctx.tree.list_add_item(list, item);
            if identity(&set, slot).is_some() {
                self.add_phys_obj_to_arm(ctx.tree, slot, &set.characters[slot]);
            }
        }
        if set.characters.len() < slots {
            if self
                .child(ctx.tree, self.background, "CreateButton")
                .is_none()
            {
                return;
            }
            self.set_ghosted(ctx.tree, self.background, "CreateButton", false);
        }
        self.arms.set_current(0);
        ctx.tree
            .list_set_selected_index(list, Some(ctx.data.gen_slot()), true);
        ctx.data.set_gen_slot(0);
        self.arms.set_move_immediate(true);
    }

    /// `gmCharacterSelectionUI::AddPhysObjToArm`: the character's figure on its arm (the
    /// appearance is the client's to build; an empty one stands in with `nologininfo`).
    fn add_phys_obj_to_arm(&mut self, tree: &UiTree, slot: usize, identity: &CharacterIdentity) {
        if self.viewport.is_none() {
            return;
        }
        let stand_in = if identity.visual == Default::default() {
            match self.data_id(tree, self.viewport, "nologininfo") {
                Some(did) => Some(did),
                None => return,
            }
        } else {
            None
        };
        self.occupants.push(ArmOccupant { slot, stand_in });
    }

    /// `gmCharacterSelectionUI::MakePleaseWaitDialog`.
    fn make_please_wait_dialog(&mut self, ctx: &mut Ctx<'_>) {
        ctx.dialogs.close_dialog(ctx.tree, self.wait_context);
        let text = ctx.strings.get("ID_Wait_PleaseWait");
        self.wait_context = ctx.dialogs.make_dialog(ctx.tree, &text, DIALOG_WAIT, false);
    }

    /// `gmCharacterSelectionUI::MakeDeleteDialog`.
    fn make_delete_dialog(&mut self, ctx: &mut Ctx<'_>, index: usize, name: &str) {
        if self.delete_context != 0 {
            return;
        }
        self.ghost_buttons(ctx.tree, true);
        let text = ctx.strings.render(
            id("ID_CharSelect_DeleteConfirm"),
            vec![text_var("name", name)],
        );
        self.delete_index = index;
        self.delete_context = ctx
            .dialogs
            .make_dialog(ctx.tree, &text, DIALOG_CONFIRMATION, false);
    }

    /// `gmCharacterSelectionUI::RecvNotice_CloseConfirmationDialog` / `DoDelete`.
    pub(crate) fn dialog_outcome(&mut self, ctx: &mut Ctx<'_>, outcome: DialogOutcome) {
        let DialogOutcome::Confirmation { context, yes } = outcome else {
            return;
        };
        if context != self.delete_context {
            return;
        }
        let index = self.delete_index;
        self.delete_index = 0;
        self.delete_context = 0;
        if !yes {
            self.free_buttons(ctx.tree, false);
            self.update(ctx);
            return;
        }
        if let Some(c) = ctx.data.characters().and_then(|set| identity(set, index)) {
            ctx.requests.push(PregameRequest::DeleteCharacter(c.id));
        }
    }

    /// `gmCharacterSelectionUI::Start3DSelection`.
    fn start_3d_selection(&mut self, tree: &UiTree) {
        self.do_select = true;
        if self.viewport.is_some() {
            self.pick = Some(tree.pointer());
        }
    }

    /// The point a pick waits on (`UIScene::StartSelection`), until
    /// [`CharacterSelectionUi::finish_3d_selection`].
    pub fn wants_pick(&self) -> Option<(i32, i32)> {
        self.pick.filter(|_| self.do_select)
    }

    /// `gmCharacterSelectionUI::Finish3DSelection`: the figure the scene picked (its slot)
    /// becomes the selection; after a double click, entering follows.
    pub(crate) fn finish_3d_selection(&mut self, ctx: &mut Ctx<'_>, slot: Option<usize>) {
        let double = self.double_click;
        self.do_select = false;
        self.double_click = false;
        self.pick = None;
        let Some(slot) = slot.filter(|s| self.occupants.iter().any(|o| o.slot == *s)) else {
            return;
        };
        let Some(list) = self.list_box else {
            return;
        };
        ctx.tree.list_set_selected_index(list, Some(slot), true);
        self.arms.set_move_immediate(double);
        if double {
            self.enter(ctx);
        }
    }

    /// `ListenToMessage`'s enter path: the selected character enters the world.
    fn enter(&mut self, ctx: &mut Ctx<'_>) {
        let Some(set) = ctx.data.characters().cloned() else {
            return;
        };
        let index = self.list_box.and_then(|l| ctx.tree.list_selected_index(l));
        let error = match index.and_then(|i| set.characters.get(i).map(|c| (i, c))) {
            None => "ID_Character_Err_NoChacter",
            Some((_, c)) if c.id == 0 => "ID_Character_Err_InvalidID",
            Some((_, c)) if c.name.is_empty() => "ID_Character_Err_NoName",
            Some((i, _)) if greyed_out_for(&set, i) != 0 => "ID_Character_Err_EnterWDel",
            Some((_, c)) => {
                self.ghost_buttons(ctx.tree, true);
                ctx.requests.push(PregameRequest::EnterWorld(c.id));
                self.make_please_wait_dialog(ctx);
                ctx.flow.queue_mode(MODE_GAMEPLAY, "");
                return;
            }
        };
        let text = ctx.strings.get(error);
        self.set_error_msg(ctx.tree, &text);
    }

    /// `gmCharacterSelectionUI::ListenToMessage` for an element's message.
    pub(crate) fn message(&mut self, ctx: &mut Ctx<'_>, message: &UiMessage) {
        let (element, kind) = (message.element_id, message.message);
        if kind == MSG_BUTTON_RELEASE_ON && element == id("QuitButton") {
            ctx.requests.push(PregameRequest::Quit);
            return;
        }
        if element == id("Viewport") && kind == MSG_MOUSE_PRESS_TOP {
            let action = message.data as u32;
            if action != LEFT && action != DOUBLE {
                return;
            }
            if !self.do_select {
                self.start_3d_selection(ctx.tree);
            }
            if self.double_click || !self.do_select {
                return;
            }
            self.double_click = action == DOUBLE;
            return;
        }
        if ctx.data.characters().is_none() {
            return;
        }
        if element == id("list_box_item") && kind == MSG_MOUSE_PRESS_TOP {
            if message.data as u32 != DOUBLE {
                return;
            }
            let (Some(item), Some(list)) = (message.element, self.list_box) else {
                return;
            };
            self.arms.set_move_immediate(true);
            ctx.tree.list_set_selected_item(list, Some(item), true);
            self.enter(ctx);
            return;
        }
        if kind == MSG_BUTTON_RELEASE_ON {
            self.button(ctx, element);
        } else if element == id("CharacterListBox") && kind == MSG_NEW_SELECTION {
            self.new_selection(ctx);
        }
    }

    /// The buttons' releases.
    fn button(&mut self, ctx: &mut Ctx<'_>, element: u32) {
        let mode = [
            ("CreateButton", MODE_AVATAR_SELECTION),
            ("OptionsButton", MODE_KEYBOARD),
            ("CreditsButton", MODE_CREDITS),
        ]
        .into_iter()
        .find(|(name, _)| element == id(name));
        if let Some((_, mode)) = mode {
            self.ghost_buttons(ctx.tree, true);
            self.make_please_wait_dialog(ctx);
            ctx.flow.queue_mode(mode, "");
            return;
        }
        if element == id("EnterButton") {
            self.enter(ctx);
            return;
        }
        let selected = self.list_box.and_then(|l| ctx.tree.list_selected_index(l));
        let chosen = selected.and_then(|i| {
            let c = ctx.data.characters()?.characters.get(i)?;
            Some((i, c.id, c.name.clone()))
        });
        if element == id("DeleteButton") {
            match chosen {
                Some((index, _, name)) => self.make_delete_dialog(ctx, index, &name),
                None => {
                    let text = ctx.strings.get("ID_Character_Err_NoChacter");
                    self.set_error_msg(ctx.tree, &text);
                }
            }
        } else if element == id("RestoreButton") {
            match chosen {
                Some((_, character, name)) => {
                    self.ghost_buttons(ctx.tree, true);
                    ctx.requests
                        .push(PregameRequest::RestoreCharacter { character, name });
                }
                None => {
                    let text = ctx.strings.get("ID_Character_Err_NoChacter");
                    self.set_error_msg(ctx.tree, &text);
                }
            }
        }
    }

    /// `ListenToMessage`'s `new_selection`: the arms turn to the selection; enter and delete
    /// follow whether it is pending deletion (then restore instead).
    fn new_selection(&mut self, ctx: &mut Ctx<'_>) {
        let Some(list) = self.list_box else {
            return;
        };
        let selected = ctx.tree.list_selected_index(list);
        // Retail stores -1 for no selection, which no slot matches; the arms stay.
        if let Some(index) = selected {
            self.arms.set_target(index);
        }
        let greyed = match (ctx.data.characters(), selected) {
            (Some(set), Some(index)) => greyed_out_for(set, index) != 0,
            _ => false,
        };
        let Some(enter) = self.child(ctx.tree, self.enter_field, "EnterButton") else {
            return;
        };
        ctx.tree.set_bool(enter, BUTTON_GHOSTED, greyed);
        let Some(field) = self.child(ctx.tree, self.background, "DeleteRestoreField") else {
            return;
        };
        ctx.tree
            .set_state(field, id(if greyed { "restore" } else { "delete" }));
        self.set_ghosted(ctx.tree, self.background, "DeleteButton", greyed);
        self.set_ghosted(ctx.tree, self.background, "RestoreButton", !greyed);
    }

    /// `gmCharacterSelectionUI::RecvNotice_CharGenVerification` (an answer to a restore): a
    /// restored character's deletion time is taken from the answer; any answer is shown, then
    /// the list made again.
    pub(crate) fn char_gen_verification(
        &mut self,
        ctx: &mut Ctx<'_>,
        response: u32,
        restored: Option<&CharacterIdentity>,
    ) {
        if ctx.data.characters().is_none() {
            return;
        }
        let selected = self.list_box.and_then(|l| ctx.tree.list_selected_index(l));
        let error = match response {
            1 => {
                if let (Some(restored), Some(index)) = (restored, selected) {
                    if let Some(set) = ctx.data.characters_mut() {
                        let listed = set.characters.iter().any(|c| c.id == restored.id);
                        let here = identity(set, index)
                            .is_some_and(|c| c.id == restored.id && c.seconds_greyed_out != 0);
                        if listed && here {
                            set_greyed_out_for(set, index, restored.seconds_greyed_out);
                        }
                    }
                }
                "ID_Character_Err_OK"
            }
            2 => "ID_Character_Err_Pending",
            3 => "ID_Character_Err_NameInUse",
            5 => "ID_Character_Err_Corrupt",
            6 => "ID_Character_Err_DatabaseDown",
            7 => "ID_Character_Err_AdminPriv",
            _ => "ID_Character_Err_Unknown",
        };
        let text = ctx
            .strings
            .render(id(error), vec![int_var("response", response as i32)]);
        self.set_error_msg(ctx.tree, &text);
        self.free_buttons(ctx.tree, false);
        self.update(ctx);
    }

    /// `ListenToMessage(UIMESSAGE_GLOBAL_LOOP)`'s arms (a pick is finished by the client).
    pub(crate) fn global_loop(&mut self, now: f64) {
        self.arms.global_loop(now);
    }

    /// The arms, one per character slot.
    pub fn arms(&self) -> &CharacterArms {
        &self.arms
    }

    /// The figures standing on the arms.
    pub fn occupants(&self) -> &[ArmOccupant] {
        &self.occupants
    }

    /// The scene `Viewport` shows (`uiscene`), its camera entity's name (`camera`) and the arm
    /// object (`armdid`).
    pub fn scene(&self, tree: &UiTree) -> (Option<u32>, Option<u32>) {
        (
            self.data_id(tree, self.viewport, "uiscene"),
            self.data_id(tree, self.viewport, "armdid"),
        )
    }
}
