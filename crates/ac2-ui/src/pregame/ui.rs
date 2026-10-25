//! [`PregameUi`]: `gmUIFlow` with its frameworks, the dialogs and the shared data over one element
//! tree. The session's login notices come in as calls (`RecvNotice_CharacterSet`,
//! `RecvNotice_CharacterError`, `RecvNotice_CharGenVerification`, `RecvNotice_ServerDied`); the
//! frame's UI actions go to the tree and its messages to the dialogs and the current framework
//! (`UIListener`); each UI loop (`UIMESSAGE_GLOBAL_LOOP`) runs the frameworks and switches to a
//! queued mode (`UIFlow::UseNewMode`). Before the first character set the UI is in the data patch
//! (connecting) mode, which the client stands in for: the first set switches to character
//! selection (`gmDataPatchUI::Update`).
//!
//! Not modelled yet: the intro, data patch, keyboard and credits frameworks (their modes show
//! nothing), and character generation's screens.

use std::collections::HashMap;

use ac2_net::protocol::{CharGenResult, CharacterIdentity, CharacterSet};

use crate::compose::UiPixels;
use crate::dialog::{DialogFactory, LAYOUT_DIALOGS};
use crate::element::UiTree;
use crate::mouse::UiMessage;
use crate::pregame::assets::PregameAssets;
use crate::pregame::character_selection::CharacterSelectionUi;
use crate::pregame::disconnected::DisconnectedUi;
use crate::pregame::flow::{UiFlow, MODE_CHARACTER_SELECTION, MODE_DISCONNECTED};
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
    /// `Device::Done`: close the client.
    Quit,
}

/// The current main framework (`UIFlow::_curUI`).
#[derive(Debug, Default)]
enum Framework {
    #[default]
    None,
    CharacterSelection(CharacterSelectionUi),
    Disconnected(DisconnectedUi),
}

/// The pregame UI.
#[derive(Debug)]
pub struct PregameUi {
    assets: PregameAssets,
    tree: UiTree,
    dialogs: DialogFactory,
    flow: UiFlow,
    data: PersistentData,
    current: Framework,
    requests: Vec<PregameRequest>,
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
            },
            current,
        )
    }};
}

impl PregameUi {
    /// The UI on a `width` x `height` display, connecting (no framework yet).
    pub fn new(assets: PregameAssets, width: i32, height: i32) -> Self {
        let tree = assets.tree(width, height);
        let dialogs = DialogFactory::new(assets.layout(LAYOUT_DIALOGS).cloned());
        Self {
            assets,
            tree,
            dialogs,
            flow: UiFlow::new(),
            data: PersistentData::default(),
            current: Framework::None,
            requests: Vec::new(),
        }
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

    /// The dialogs.
    pub fn dialogs(&self) -> &DialogFactory {
        &self.dialogs
    }

    /// What the UI asked since the last call.
    pub fn take_requests(&mut self) -> Vec<PregameRequest> {
        std::mem::take(&mut self.requests)
    }

    /// `UIPersistantData::RecvNotice_CharacterSet` (`SetCharSet`): the set kept and the shown
    /// list made again; the first set ends connecting with character selection.
    pub fn character_set(&mut self, set: CharacterSet) {
        self.data.set_characters(set);
        let (mut ctx, current) = ctx!(self);
        match current {
            Framework::CharacterSelection(ui) => ui.update(&mut ctx),
            Framework::None if ctx.flow.mode() == 0 && ctx.flow.next_mode().is_none() => {
                ctx.flow.queue_mode(MODE_CHARACTER_SELECTION, "");
            }
            _ => {}
        }
        self.pump();
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
    pub fn char_gen_verification(&mut self, response: u32, identity: Option<&CharacterIdentity>) {
        let (mut ctx, current) = ctx!(self);
        if let Framework::CharacterSelection(ui) = current {
            ui.char_gen_verification(&mut ctx, response, identity);
        }
        self.pump();
    }

    /// The frame's UI actions (`UIElementManager::StartInput` / `StopInput`): (action, extent,
    /// start). Returns every message they broadcast.
    pub fn input(&mut self, actions: &[(u32, f32, bool)]) -> Vec<UiMessage> {
        for &(action, extent, start) in actions {
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
        for &(ch, key) in characters {
            self.tree.character(ch, key, ctrl);
        }
        self.pump()
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
                let outcomes = self.dialogs.take_outcomes();
                let (mut ctx, current) = ctx!(self);
                for outcome in outcomes {
                    if let Framework::CharacterSelection(ui) = current {
                        ui.dialog_outcome(&mut ctx, outcome);
                    }
                }
                match current {
                    Framework::CharacterSelection(ui) => ui.message(&mut ctx, message),
                    Framework::Disconnected(ui) => ui.message(&mut ctx, message),
                    Framework::None => {}
                }
            }
            all.extend(messages);
        }
    }

    /// Where a 3D pick waits (`UIScene::StartSelection`'s point), if one does.
    pub fn wants_pick(&self) -> Option<(i32, i32)> {
        self.character_selection().and_then(|ui| ui.wants_pick())
    }

    /// The pick's answer: the slot of the figure under the point, if any
    /// (`Finish3DSelection`).
    pub fn finish_pick(&mut self, slot: Option<usize>) {
        let (mut ctx, current) = ctx!(self);
        if let Framework::CharacterSelection(ui) = current {
            ui.finish_3d_selection(&mut ctx, slot);
        }
        self.pump();
    }

    /// `UIMESSAGE_GLOBAL_LOOP` at `now` seconds: the frameworks' loop, then the flow switches
    /// to a queued mode (`UIFlow::ListenToMessage`, `UseNewMode`).
    pub fn use_time(&mut self, now: f64) {
        if let Framework::CharacterSelection(ui) = &mut self.current {
            ui.global_loop(now);
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
            Framework::CharacterSelection(ui) => ui.destroy(&mut ctx),
            Framework::Disconnected(ui) => ui.destroy(&mut ctx),
            Framework::None => {}
        }
        *current = match mode {
            MODE_CHARACTER_SELECTION => {
                Framework::CharacterSelection(CharacterSelectionUi::create(&mut ctx))
            }
            MODE_DISCONNECTED => Framework::Disconnected(DisconnectedUi::create(&mut ctx)),
            _ => Framework::None,
        };
        if text.is_empty() {
            return;
        }
        match current {
            Framework::CharacterSelection(ui) => ui.set_error_msg(ctx.tree, text),
            Framework::Disconnected(ui) => ui.set_error_msg(&mut ctx, text),
            Framework::None => {}
        }
    }
}
