//! [`PregameUi`]: `gmUIFlow` with its frameworks, the dialogs and the shared data over one element
//! tree. The session's login notices come in as calls (`RecvNotice_CharacterSet`,
//! `RecvNotice_CharacterError`, `RecvNotice_CharGenVerification`, `RecvNotice_ServerDied`); the
//! frame's UI actions go to the tree and its messages to the dialogs and the current framework
//! (`UIListener`); each UI loop (`UIMESSAGE_GLOBAL_LOOP`) runs the frameworks and switches to a
//! queued mode (`UIFlow::UseNewMode`). The UI opens on the intro, then the data patch
//! (connecting) screen, which the character set ends with character selection
//! (`gmDataPatchUI::Update`).
//!
//! Character generation's screens ask the `CharacterGenSystem` script through the
//! [`CharGenSystem`] the client sets ([`PregameUi::set_char_gen_system`]; without one they make
//! nothing). Their 3D scenes are the client's: it renders the scenes the viewports name
//! ([`PregameUi::viewports`]), answers picks ([`PregameUi::wants_pick`],
//! [`PregameUi::finish_pick`]) and the end of the chosen avatar's animation
//! ([`PregameUi::selected_animation`], [`PregameUi::finish_animation`]).
//!
//! The controls screen (`gmKeyboardUI`) takes the key presses ([`PregameUi::key_presses`]) and
//! hands its saved or restored maps to the client as [`PregameRequest::KeyMaps`].

use std::collections::HashMap;

use ac2_net::protocol::{CharGenResult, CharacterIdentity, CharacterSet};

use crate::compose::{UiPixels, UiSurface};
use crate::dialog::{DialogFactory, LAYOUT_DIALOGS};
use crate::element::UiTree;
use crate::input::KeyPress;
use crate::mouse::UiMessage;
use crate::pregame::assets::PregameAssets;
use crate::pregame::avatar_modification::AvatarModificationUi;
use crate::pregame::avatar_selection::AvatarSelectionUi;
use crate::pregame::character_selection::CharacterSelectionUi;
use crate::pregame::chargen::{CharGenSystem, NoCharGen};
use crate::pregame::confirmation::is_name_character;
use crate::pregame::credits::CreditsUi;
use crate::pregame::data_patch::DataPatchUi;
use crate::pregame::disconnected::DisconnectedUi;
use crate::pregame::flow::{
    UiFlow, MODE_AVATAR_MODIFICATION, MODE_AVATAR_SELECTION, MODE_CHARACTER_SELECTION,
    MODE_CREDITS, MODE_DATA_PATCH, MODE_DISCONNECTED, MODE_INTRO, MODE_KEYBOARD,
};
use crate::pregame::intro::IntroUi;
use crate::pregame::keyboard::KeyboardUi;
use crate::pregame::keyboard_maps::{ActiveKeyMaps, KeyMapSet};
use crate::pregame::persistent::PersistentData;
use crate::pregame::Ctx;

/// What the pregame UI asks of the client.
#[derive(Debug, Clone, PartialEq)]
pub enum PregameRequest {
    /// `Proto_UI::SendEnterWorld`.
    EnterWorld(u64),
    /// `Proto_UI::SendDeleteCharacter`.
    DeleteCharacter(u64),
    /// `Proto_UI::SendAdminRestoreCharacter` (admin accounts).
    RestoreCharacter { character: u64, name: String },
    /// `Proto_UI::SendCharGenResult`.
    CreateCharacter(CharGenResult),
    /// `SoundManager::Play` of a screen's music on `Channel_Main` (the screen's constructor):
    /// the previous screen's music fades out over two seconds first (`PlayingSoundContext::Stop`).
    Music(u32),
    /// `gmCreditsUI::UpdateSound`'s `SoundManager::Play` of the credits' sound on `Channel_Main`.
    CreditSound(u32),
    /// `Device::Done`: close the client.
    Quit,
    /// The input manager's maps are these (`SaveKeyboardList`'s reload, a restore preset).
    KeyMaps(KeyMapSet),
}

/// The current main framework (`UIFlow::_curUI`).
#[derive(Debug, Default)]
enum Framework {
    #[default]
    None,
    Intro(IntroUi),
    DataPatch(DataPatchUi),
    Credits(CreditsUi),
    Keyboard(KeyboardUi),
    CharacterSelection(CharacterSelectionUi),
    AvatarSelection(AvatarSelectionUi),
    AvatarModification(AvatarModificationUi),
    Disconnected(DisconnectedUi),
}

/// The pregame UI.
pub struct PregameUi {
    assets: PregameAssets,
    tree: UiTree,
    dialogs: DialogFactory,
    flow: UiFlow,
    data: PersistentData,
    current: Framework,
    requests: Vec<PregameRequest>,
    system: Box<dyn CharGenSystem>,
    /// `ActionInterpreter::ActionDown(0x40000004 / 0x40000007)`: a left button is held.
    left_down: bool,
    /// The key maps the controls screen edits and the input manager holds.
    keymaps: ActiveKeyMaps,
}

impl std::fmt::Debug for PregameUi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PregameUi")
            .field("flow", &self.flow)
            .field("current", &self.current)
            .finish_non_exhaustive()
    }
}

/// Splits the UI into a framework context and the current framework.
macro_rules! ctx {
    ($ui:expr) => {{
        let PregameUi {
            assets,
            tree,
            dialogs,
            flow,
            data,
            current,
            requests,
            system,
            keymaps,
            ..
        } = $ui;
        (
            Ctx {
                tree,
                dialogs,
                flow,
                data,
                strings: &assets.strings,
                assets,
                requests,
                system: &mut **system,
                keymaps,
            },
            current,
        )
    }};
}

impl PregameUi {
    /// The UI on a `width` x `height` display, the intro queued (`gmUIFlow::gmUIFlow`).
    pub fn new(assets: PregameAssets, width: i32, height: i32) -> Self {
        let tree = assets.tree(width, height);
        let keymaps = assets.keymaps.clone();
        let dialogs = DialogFactory::new(assets.layout(LAYOUT_DIALOGS).cloned());
        let mut flow = UiFlow::new();
        flow.queue_mode(MODE_INTRO, "");
        Self {
            assets,
            tree,
            dialogs,
            flow,
            data: PersistentData::default(),
            current: Framework::None,
            requests: Vec::new(),
            system: Box::new(NoCharGen),
            left_down: false,
            keymaps: ActiveKeyMaps::new(keymaps),
        }
    }

    /// The `CharacterGenSystem` character generation asks.
    pub fn set_char_gen_system(&mut self, system: Box<dyn CharGenSystem>) {
        self.system = system;
    }

    pub fn tree(&self) -> &UiTree {
        &self.tree
    }

    pub fn tree_mut(&mut self) -> &mut UiTree {
        &mut self.tree
    }

    /// The images the pregame layouts show.
    pub fn images(&self) -> &HashMap<u32, UiPixels> {
        &self.assets.images
    }

    /// The tree's roots drawn over the screen (`UiTree::compose` with the pregame images).
    pub fn compose(&mut self) -> Vec<UiSurface> {
        self.tree.compose(&self.assets.images)
    }

    /// The current mode (0 while connecting).
    pub fn mode(&self) -> u32 {
        self.flow.mode()
    }

    /// The flow (current and queued mode).
    pub fn flow(&self) -> &UiFlow {
        &self.flow
    }

    pub fn data(&self) -> &PersistentData {
        &self.data
    }

    /// The character selection framework, when it is up.
    pub fn character_selection(&self) -> Option<&CharacterSelectionUi> {
        match &self.current {
            Framework::CharacterSelection(ui) => Some(ui),
            _ => None,
        }
    }

    /// Avatar selection, when it is up.
    pub fn avatar_selection(&self) -> Option<&AvatarSelectionUi> {
        match &self.current {
            Framework::AvatarSelection(ui) => Some(ui),
            _ => None,
        }
    }

    /// Avatar modification, when it is up.
    pub fn avatar_modification(&self) -> Option<&AvatarModificationUi> {
        match &self.current {
            Framework::AvatarModification(ui) => Some(ui),
            _ => None,
        }
    }

    /// The shown viewport roots and the scenes they show, in the order the roots are drawn
    /// (`UIViewportObject::Render` among the surfaces).
    pub fn viewports(&self) -> Vec<crate::pregame::SceneViewport> {
        crate::scene_viewports(&self.tree)
    }

    /// The dialogs.
    pub fn dialogs(&self) -> &DialogFactory {
        &self.dialogs
    }

    /// What the UI asked since the last call.
    pub fn take_requests(&mut self) -> Vec<PregameRequest> {
        std::mem::take(&mut self.requests)
    }

    /// Whether the credits' sound is still playing (its `PlayingSoundContext` holds a sound that
    /// has not finished); the client answers this before each loop. Other screens ignore it.
    pub fn set_credit_sound_busy(&mut self, busy: bool) {
        if let Framework::Credits(ui) = &mut self.current {
            ui.sound_busy = busy;
        }
    }

    /// `UIPersistantData::RecvNotice_CharacterSet` (`SetCharSet`): the set kept and the shown
    /// list made again; the first set ends connecting with character selection.
    pub fn character_set(&mut self, set: CharacterSet) {
        self.data.set_characters(set);
        let (mut ctx, current) = ctx!(self);
        match current {
            Framework::CharacterSelection(ui) => ui.update(&mut ctx),
            Framework::DataPatch(ui) => ui.update(&mut ctx),
            _ => {}
        }
        self.pump();
    }

    /// The UI reopened after a session ended: the set the session kept is the character list, and
    /// the flow goes to character selection instead of the intro (`UIFlow::QueueUIMode`, as
    /// `DoPleaseWaitCancel` queues it). The next UI loop makes the framework.
    pub fn resume_at_character_selection(&mut self, set: CharacterSet) {
        self.data.set_characters(set);
        self.flow.queue_mode(MODE_CHARACTER_SELECTION, "");
    }

    /// `gmDataPatchUI::OnNetStatusChange(Net_LoginConnected)`: the session connected.
    pub fn connected(&mut self) {
        let (mut ctx, current) = ctx!(self);
        if let Framework::DataPatch(ui) = current {
            ui.connected(&mut ctx);
        }
    }

    /// `gmUIFlow::RecvNotice_CharacterError`, with the server's text when it sent one.
    pub fn character_error(&mut self, error: u32, text: Option<&str>) {
        self.flow.character_error(&self.assets.strings, error, text);
    }

    /// `gmUIFlow::RecvNotice_ServerDied`.
    pub fn server_died(&mut self) {
        self.flow.server_died(&self.assets.strings);
    }

    /// `RecvNotice_CharGenVerification` for the frameworks that wait on it.
    pub fn char_gen_verification(
        &mut self,
        response: u32,
        identity: Option<&CharacterIdentity>,
        weenie_result: u32,
    ) {
        let (mut ctx, current) = ctx!(self);
        match current {
            Framework::CharacterSelection(ui) => {
                ui.char_gen_verification(&mut ctx, response, identity)
            }
            Framework::AvatarModification(ui) => {
                if let Some(dialog) = ui.dialog_mut() {
                    dialog.char_gen_verification(&mut ctx, response, identity, weenie_result);
                }
            }
            Framework::DataPatch(ui) => ui.char_gen_verification(&mut ctx, response, identity),
            _ => {}
        }
        self.pump();
    }

    /// The frame's UI actions (`UIElementManager::StartInput` / `StopInput`): (action, extent,
    /// start). Returns every message they broadcast.
    pub fn input(&mut self, actions: &[(u32, f32, bool)]) -> Vec<UiMessage> {
        for &(action, extent, start) in actions {
            if matches!(action, 0x4000_0004 | 0x4000_0007) {
                self.left_down = start;
            }
            if start {
                self.tree.start_input(action, extent);
            } else {
                self.tree.stop_input(action);
            }
        }
        self.pump()
    }

    /// The active text element's actions and characters; returns the messages.
    pub fn text_input(
        &mut self,
        actions: &[(u32, f32)],
        characters: &[(u16, u32)],
        ctrl: bool,
    ) -> Vec<UiMessage> {
        for &(action, extent) in actions {
            self.tree.text_action(action, extent, ctrl);
        }
        // `NameInputFilter` on the naming dialog's entry.
        let name_entry = match &self.current {
            Framework::AvatarModification(ui) => ui.dialog().and_then(|d| d.name_entry()),
            _ => None,
        };
        for &(ch, key) in characters {
            if name_entry.is_some() && self.tree.active() == name_entry && !is_name_character(ch) {
                continue;
            }
            self.tree.character(ch, key, ctrl);
        }
        self.pump()
    }

    /// The dialogs' notices to the framework that asked (`RecvNotice_CloseConfirmationDialog`).
    fn route_outcomes(&mut self) {
        let outcomes = self.dialogs.take_outcomes();
        let (mut ctx, current) = ctx!(self);
        for outcome in outcomes {
            match current {
                Framework::CharacterSelection(ui) => ui.dialog_outcome(&mut ctx, outcome),
                Framework::Keyboard(ui) => ui.dialog_outcome(&mut ctx, outcome),
                _ => {}
            }
        }
    }

    /// The context of the dialog on screen, if one is (tests answer it).
    #[cfg(test)]
    pub(crate) fn shown_dialog(&self) -> Option<u32> {
        self.dialogs.shown().map(|(context, _)| context)
    }

    /// A confirmation's yes or no, as its button sends it (tests).
    #[cfg(test)]
    pub(crate) fn answer_dialog(&mut self, context: u32, yes: bool) {
        self.dialogs.answer(&mut self.tree, context, yes);
        self.route_outcomes();
    }

    /// The keyboard presses the controls screen's key hit handler takes (`KeyHitHandler`).
    pub fn key_presses(&mut self, presses: &[KeyPress]) {
        for press in presses {
            let (mut ctx, current) = ctx!(self);
            if let Framework::Keyboard(ui) = current {
                ui.key_hit(&mut ctx, press.semantic, press.meta_mode);
            }
        }
    }

    /// The tree's messages to the dialogs and the framework until no more come; all of them.
    fn pump(&mut self) -> Vec<UiMessage> {
        let mut all = Vec::new();
        loop {
            let messages = self.tree.take_messages();
            if messages.is_empty() {
                return all;
            }
            for message in &messages {
                self.dialogs.message(&mut self.tree, message);
                self.route_outcomes();
                let left_down = self.left_down;
                let (mut ctx, current) = ctx!(self);
                match current {
                    Framework::Intro(ui) => ui.message(&mut ctx, message),
                    Framework::DataPatch(ui) => ui.message(&mut ctx, message),
                    Framework::Credits(ui) => ui.message(&mut ctx, message),
                    Framework::Keyboard(ui) => ui.message(&mut ctx, message),
                    Framework::CharacterSelection(ui) => ui.message(&mut ctx, message),
                    Framework::AvatarSelection(ui) => ui.message(&mut ctx, message),
                    Framework::AvatarModification(ui) => ui.message(&mut ctx, message, left_down),
                    Framework::Disconnected(ui) => ui.message(&mut ctx, message),
                    Framework::None => {}
                }
            }
            all.extend(messages);
        }
    }

    /// Where a 3D pick waits (`UIScene::StartSelection`'s point), if one does.
    pub fn wants_pick(&self) -> Option<(i32, i32)> {
        match &self.current {
            Framework::CharacterSelection(ui) => ui.wants_pick(),
            Framework::AvatarSelection(ui) => ui.wants_pick().then(|| self.tree.pointer()),
            _ => None,
        }
    }

    /// The pick's answer: what is under the point, if anything: character selection's slot
    /// (`Finish3DSelection`) or avatar selection's avatar index (its UI loop's selection).
    pub fn finish_pick(&mut self, picked: Option<usize>) {
        let (mut ctx, current) = ctx!(self);
        match current {
            Framework::CharacterSelection(ui) => ui.finish_3d_selection(&mut ctx, picked),
            Framework::AvatarSelection(ui) => ui.finish_pick(&mut ctx, picked),
            _ => {}
        }
        self.pump();
    }

    /// The avatar whose selected animation plays (avatar selection waits for its end).
    pub fn selected_animation(&self) -> Option<usize> {
        match &self.current {
            Framework::AvatarSelection(ui) => ui.selected_animation(),
            _ => None,
        }
    }

    /// The selected animation ended (the avatar's behavior `0x41000005` stopped running).
    pub fn finish_animation(&mut self) {
        let (mut ctx, current) = ctx!(self);
        if let Framework::AvatarSelection(ui) = current {
            ui.animation_done(&mut ctx);
        }
        self.pump();
    }

    /// `UIMESSAGE_GLOBAL_LOOP` at `now` seconds: the elements' media and transitions, the
    /// frameworks' loop, then the flow switches to a queued mode (`UIFlow::ListenToMessage`,
    /// `UseNewMode`).
    pub fn use_time(&mut self, now: f64) {
        self.tree.global_loop(now);
        let (mut ctx, current) = ctx!(self);
        match current {
            Framework::CharacterSelection(ui) => ui.global_loop(now),
            Framework::Credits(ui) => ui.global_loop(&mut ctx, now),
            _ => {}
        }
        if let Some((mode, text)) = self.flow.use_new_mode() {
            self.use_new_mode(mode, &text);
        }
        self.pump();
    }

    /// `UIFlow::UseNewMode`'s framework switch.
    fn use_new_mode(&mut self, mode: u32, text: &str) {
        let (mut ctx, current) = ctx!(self);
        match std::mem::take(current) {
            Framework::Intro(ui) => ui.destroy(&mut ctx),
            Framework::DataPatch(ui) => ui.destroy(&mut ctx),
            Framework::Credits(ui) => ui.destroy(&mut ctx),
            Framework::Keyboard(ui) => ui.destroy(&mut ctx),
            Framework::CharacterSelection(ui) => ui.destroy(&mut ctx),
            Framework::AvatarSelection(ui) => ui.destroy(&mut ctx),
            Framework::AvatarModification(ui) => ui.destroy(&mut ctx),
            Framework::Disconnected(ui) => ui.destroy(&mut ctx),
            Framework::None => {}
        }
        *current = match mode {
            MODE_INTRO => Framework::Intro(IntroUi::create(&mut ctx)),
            MODE_DATA_PATCH => Framework::DataPatch(DataPatchUi::create(&mut ctx)),
            MODE_CREDITS => Framework::Credits(CreditsUi::create(&mut ctx)),
            MODE_KEYBOARD => Framework::Keyboard(KeyboardUi::create(&mut ctx)),
            MODE_CHARACTER_SELECTION => {
                Framework::CharacterSelection(CharacterSelectionUi::create(&mut ctx))
            }
            MODE_AVATAR_SELECTION => {
                Framework::AvatarSelection(AvatarSelectionUi::create(&mut ctx))
            }
            MODE_AVATAR_MODIFICATION => {
                Framework::AvatarModification(AvatarModificationUi::create(&mut ctx))
            }
            MODE_DISCONNECTED => Framework::Disconnected(DisconnectedUi::create(&mut ctx)),
            _ => Framework::None,
        };
        if text.is_empty() {
            return;
        }
        match current {
            Framework::CharacterSelection(ui) => ui.set_error_msg(ctx.tree, text),
            Framework::DataPatch(ui) => ui.set_error_msg(&mut ctx, text),
            Framework::Disconnected(ui) => ui.set_error_msg(&mut ctx, text),
            _ => {}
        }
    }
}
