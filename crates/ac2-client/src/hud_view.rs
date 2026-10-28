//! The gameplay HUD over the world (`window` feature): the element tree
//! `gmGamePlayUI::SetupChildren`'s frameworks create, built and its images read on a worker
//! thread, then composed (`UIRegion::DrawHere`, software, as retail) whenever it arrives or the
//! window changes size, and drawn as one layer per root surface. The frame's UI actions go to
//! the tree (`UIElementManager::StartInput`), and it is composed again when they change it.
//! The tree's messages go to the chat (`gmChatUI`) before they are returned.

use crate::ui_scale::UiScale;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::sync::Arc;

use ac2_db::ResourceResult;
use ac2_gpu::{Gpu, UiOverlay};
use ac2_ui::{
    ChatRequest, ChatSettings, ChatUi, GameplayChild, GameplayHud, Modifiers, ReticleOutcome,
    ToolbarChild, ToolbarUi, UiMessage, UiPixels, UiTree, GAMEPLAY_SHOW_KEY,
};

type Built = ResourceResult<(GameplayHud, HashMap<u32, UiPixels>)>;

mod accessors;
mod build;
mod compose;

pub(crate) struct HudView {
    pending: Option<Receiver<Built>>,
    pub(crate) hud: Option<(UiTree, HashMap<u32, UiPixels>)>,
    /// The HUD's panel state: `gm3DTextUI`'s reticle, name plates and damage texts, and the clock.
    pub(crate) panels: ac2_ui::Hud,
    /// `gmGamePlayUI::m_shown`: the HUD is shown (`gmGamePlayUI::Show`, the `0x4100000C` key).
    pub(crate) gameplay_shown: bool,
    /// The gameplay panels' show state before a story hid them (`None` with no story playing).
    pub(crate) story_hid_gameplay: Option<bool>,
    /// Images set after the build, loaded on demand.
    images: ac2_ui::UiImageLoader,
    /// The chat lines being rendered (`StringDownload`).
    chat_text: ac2_ui::chat_text::ChatText,
    overlay: UiOverlay,
    /// The root of each drawn layer, in drawing order (the viewports' scenes go after theirs).
    layer_roots: Vec<usize>,
    /// The window, in pixels.
    size: (u32, u32),
    /// The HUD laid out over the whole window and drawn scaled ([`UiScale::fill`]).
    pub(crate) scale: UiScale,
    pub(crate) dirty: bool,
    /// The sounds the UI's media played (`MD_Data_Sound`) until the audio sink takes them.
    pub(crate) sounds: Vec<u32>,
}

impl HudView {
    /// The window, in pixels.
    pub(crate) fn window_size(&self) -> (u32, u32) {
        self.size
    }

    /// The render device's new size: every root placed again (`UIElement::UpdateSizeAndPosition`).
    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        self.size = (width.max(1), height.max(1));
        self.scale = UiScale::fill(self.size);
        self.overlay.set_smooth(!self.scale.is_identity());
        let display = self.scale.display;
        if let Some((tree, _)) = &mut self.hud {
            tree.set_display(display.0 as i32, display.1 as i32);
        }
        self.dirty = true;
    }

    /// The frame's UI actions (`IUIManager` start / stop: `UIElementManager::StartInput` /
    /// `StopInput`); returns the messages they broadcast.
    pub(crate) fn input(
        &mut self,
        actions: &[(u32, f32, bool)],
        modifiers: Modifiers,
    ) -> Vec<UiMessage> {
        self.panels.shift_held = modifiers.shift;
        let Some((tree, _)) = &mut self.hud else {
            return Vec::new();
        };
        tree.set_shift(modifiers.shift);
        tree.set_alt(modifiers.alt);
        for (action, extent, start) in self.scale.actions_to_ui(actions) {
            if start {
                tree.start_input(action, extent);
            } else {
                tree.stop_input(action);
            }
        }
        let messages = pump(tree, &mut self.panels.chat, &self.panels.toolbar, modifiers);
        self.panels.player_messages.extend(messages.iter().cloned());
        if tree.take_changed() {
            self.dirty = true;
        }
        self.gameplay_keys(&messages);
        messages
    }

    /// `gmGamePlayUI::ListenToMessage` for the frame's global key presses. A child key runs its
    /// `ChildKeyPress` while the gameplay UI is shown; the HUD key is kept for
    /// [`HudView::frameworks`].
    fn gameplay_keys(&mut self, messages: &[UiMessage]) {
        for message in messages {
            if message.element.is_some() || message.message != ac2_ui::GLOBAL_KEY_PRESS {
                continue;
            }
            let key = message.data as u32;
            if key == GAMEPLAY_SHOW_KEY {
                self.panels.hud_key_pending = true;
            } else if key == ac2_ui::USE_KEY {
                self.panels.use_key_pending = true;
            } else if let Some(child) = ac2_ui::child_for_key(key) {
                if self.gameplay_shown {
                    self.set_child_shown(child, !self.child_shown(child));
                }
            }
        }
    }

    /// Whether a gameplay child is shown: the toolbar's bar, the status bar, the radar.
    pub(crate) fn child_shown(&self, child: GameplayChild) -> bool {
        let Some((tree, _)) = &self.hud else {
            return false;
        };
        match child {
            GameplayChild::Toolbar => self
                .panels
                .toolbar
                .as_ref()
                .is_some_and(|t| t.bar_shown(tree)),
            GameplayChild::Statusbar => ac2_ui::statusbar_shown(tree),
            GameplayChild::Radar => self.panels.radar.as_ref().is_some_and(|r| r.shown(tree)),
        }
    }

    /// A gameplay child's `Show`: the toolbar's bar (with its shortcut bar), the status bar or the
    /// radar. Their panels' own cascade is not modelled here.
    pub(crate) fn set_child_shown(&mut self, child: GameplayChild, shown: bool) {
        match child {
            GameplayChild::Toolbar => self.show_toolbar(shown),
            GameplayChild::Statusbar => {
                if let Some((tree, _)) = &mut self.hud {
                    ac2_ui::show_statusbar(tree, shown);
                }
            }
            GameplayChild::Radar => {
                if let (Some((tree, _)), Some(radar)) = (&mut self.hud, &self.panels.radar) {
                    radar.show(tree, shown);
                }
            }
        }
        self.dirty = true;
    }

    /// `gmGamePlayUI::Show(shown)` for the HUD's children with a key: the status bar, the radar
    /// and the toolbar's bar show or hide with the flag.
    pub(crate) fn set_gameplay_shown(&mut self, shown: bool) {
        self.gameplay_shown = shown;
        for child in [
            GameplayChild::Toolbar,
            GameplayChild::Statusbar,
            GameplayChild::Radar,
        ] {
            self.set_child_shown(child, shown);
        }
    }

    /// Whether the use key was pressed since the last call (`gmCWInterface::UseSelection`).
    pub(crate) fn take_use_key(&mut self) -> bool {
        std::mem::take(&mut self.panels.use_key_pending)
    }

    /// `gmGamePlayUI::ListenToMessage` case `0x4100000C`: the HUD toggles unless a death dialog is
    /// open or a logout is requested, and `gmGamePlayUI::Show` ignores it while a please-wait
    /// dialog or a story holds the UI.
    pub(crate) fn gameplay_show_key(&mut self, story_active: bool) {
        if !std::mem::take(&mut self.panels.hud_key_pending) {
            return;
        }
        if self.panels.death.context() != 0 || self.panels.game_dialogs.end_session_requested() {
            return;
        }
        if self.panels.game_dialogs.wait_context() != 0 || story_active {
            return;
        }
        self.set_gameplay_shown(!self.gameplay_shown);
    }

    /// The active text element's actions and characters (`UIElement_Text::ActionHandler` /
    /// `CharacterHandler`); returns the messages they broadcast.
    pub(crate) fn text_input(
        &mut self,
        actions: &[(u32, f32)],
        characters: &[(u16, u32)],
        modifiers: Modifiers,
    ) -> Vec<UiMessage> {
        let Some((tree, _)) = &mut self.hud else {
            return Vec::new();
        };
        for &(action, extent) in actions {
            tree.text_action(action, extent, modifiers);
        }
        for &(ch, key) in characters {
            tree.character(ch, key, modifiers);
        }
        let messages = pump(tree, &mut self.panels.chat, &self.panels.toolbar, modifiers);
        self.panels.player_messages.extend(messages.iter().cloned());
        if tree.take_changed() {
            self.dirty = true;
        }
        messages
    }

    /// What the chat asked since the last call (lines to send, history steps).
    pub(crate) fn take_chat_requests(&mut self) -> Vec<ChatRequest> {
        self.panels
            .chat
            .as_mut()
            .map_or_else(Vec::new, ChatUi::take_requests)
    }

    /// `CM_UI::SendNotice_OpenExaminationPanel` (`gmExaminationUI::RecvNotice_OpenExaminationPanel`).
    pub(crate) fn open_examination(&mut self) {
        if let (Some((tree, _)), Some(toolbar)) = (&mut self.hud, &self.panels.toolbar) {
            toolbar.show(tree, ToolbarChild::Examination, true);
            if tree.take_changed() {
                self.dirty = true;
            }
        }
    }

    /// `ChatWindow::RecvNotice_ReloadChatSettings` with the player's chat options.
    pub(crate) fn reload_chat_settings(&mut self, settings: &ChatSettings) {
        if let (Some((tree, _)), Some(chat)) = (&mut self.hud, &mut self.panels.chat) {
            chat.reload_chat_settings(tree, settings);
            if tree.take_changed() {
                self.dirty = true;
            }
        }
    }

    /// `ChatWindow::DisplayCmdHistory`'s answer: the entry line shows `text`.
    pub(crate) fn set_chat_entry(&mut self, text: &str) {
        if let (Some((tree, _)), Some(chat)) = (&mut self.hud, &mut self.panels.chat) {
            chat.set_entry_text(tree, text);
            if tree.take_changed() {
                self.dirty = true;
            }
        }
    }

    /// `ChatWindow::DoAssistedTell`: the entry line becomes a tell to `target`.
    pub(crate) fn assisted_tell(&mut self, target: &str) {
        if let (Some((tree, _)), Some(chat)) = (&mut self.hud, &mut self.panels.chat) {
            chat.assisted_tell(tree, target);
            if tree.take_changed() {
                self.dirty = true;
            }
        }
    }

    /// `gmToolbarUI::Show`: the toolbar field shows or hides, and its height is the bottom
    /// preclude the reticle and the name plates keep (`CM_UI::SendNotice_PrecludeFromBottom`,
    /// which the next frame's update applies).
    pub(crate) fn show_toolbar(&mut self, shown: bool) {
        // The bar's height is taken only while the reticle the preclude is kept on exists.
        let has_reticle = self.panels.reticle_mut().is_some();
        let bottom = match (&mut self.hud, &self.panels.toolbar) {
            (Some((tree, _)), Some(toolbar)) if has_reticle => Some(toolbar.show_bar(tree, shown)),
            _ => None,
        };
        if let (Some(bottom), Some(reticle)) = (bottom, self.panels.reticle_mut()) {
            reticle.precluded_bottom = bottom;
            self.dirty = true;
        }
        // `gmShortcutUI::Show` from `gmToolbarUI::Show`.
        if let (Some((tree, _)), Some(bar)) = (&mut self.hud, &self.panels.shortcuts) {
            bar.show(tree, shown);
        }
    }

    /// `gmToolbarUI`'s panel show (a toolbar button or key).
    pub(crate) fn show_panel(&mut self, child: ToolbarChild, shown: bool) {
        if let (Some((tree, _)), Some(toolbar)) = (&mut self.hud, &self.panels.toolbar) {
            toolbar.show(tree, child, shown);
            // `gmQuestsUI::Show` when the Quest child changed.
            if let Some(quests) = self.panels.quests.as_mut() {
                quests.sync(tree, toolbar.shown(tree, ToolbarChild::Quest));
            }
            if let Some(doll) = &self.panels.paper_doll {
                doll.follow_panel(tree);
            }
            self.dirty = true;
        }
    }

    /// `CM_Communication::SendNotice_DisplayStringInfo`: the line is rendered on a worker and
    /// reaches the chat windows when it is done.
    pub(crate) fn chat_display_info(&mut self, kind: u32, info: &ac2_net::protocol::StringInfo) {
        self.chat_text.render(kind, info);
    }

    /// `CM_Communication::SendNotice_DisplayStringInfo`'s text for the chat windows.
    pub(crate) fn chat_display(&mut self, kind: u32, text: &str) {
        match &mut self.panels.chat {
            Some(chat) => chat.display(kind, text),
            None => self.panels.early_lines.push((kind, text.to_owned())),
        }
    }

    /// One frame of the HUD at `Timer::cur_time` `now` (seconds).
    pub(crate) fn update(&mut self, gpu: &Gpu, now: f64) {
        self.panels.set_now(now);
        if let Some(pending) = &self.panels.pending_strings {
            if let Ok(tables) = pending.try_recv() {
                self.panels.strings = Some(Arc::new(tables));
                self.panels.pending_strings = None;
            }
        }
        if let Some(pending) = &self.pending {
            match pending.try_recv() {
                Ok(built) => {
                    self.pending = None;
                    self.arrived(built);
                }
                Err(TryRecvError::Disconnected) => self.pending = None,
                Err(TryRecvError::Empty) => {}
            }
        }
        if let Some((_, images)) = &mut self.hud {
            if self.images.take(images) {
                self.dirty = true;
            }
        }
        for (kind, text) in self.chat_text.take() {
            self.chat_display(kind, &text);
        }
        // `UIMESSAGE_GLOBAL_LOOP`: the meters move and the chat windows write what arrived.
        if let Some((tree, _)) = &mut self.hud {
            tree.global_loop(now);
            if let Some(doll) = &self.panels.paper_doll {
                doll.follow_panel(tree);
            }
            // `gmShortcutUI::UpdateTimeLeft`: the skills on reset fill their slots' meters.
            if let Some(bar) = &mut self.panels.shortcuts {
                bar.update_time_left(tree, now);
            }
            self.sounds.extend(tree.take_sounds());
            if let Some(chat) = &mut self.panels.chat {
                chat.global_loop(tree, now);
            }
            if tree.take_changed() {
                self.dirty = true;
            }
        }
        self.compose(gpu);
    }

    /// Waits for the build and the images it shows (headless snapshots), so a snapshot never
    /// races the UI image loader.
    pub(crate) fn settle(&mut self, gpu: &Gpu) {
        if let Some(pending) = self.pending.take() {
            if let Ok(built) = pending.recv() {
                self.arrived(built);
            }
        }
        self.compose(gpu);
        // A composed image may show further images (a newly loaded frame's children), so
        // compose again until nothing new arrives.
        for _ in 0..8 {
            let Some((_, images)) = &mut self.hud else {
                break;
            };
            if !self
                .images
                .wait_all(images, std::time::Duration::from_secs(10))
            {
                break;
            }
            self.dirty = true;
            self.compose(gpu);
        }
    }

    /// `gm3DTextUI::UpdateNames` and `UpdateDamage` for this frame, over the projected boxes.
    pub(crate) fn update_overlays(
        &mut self,
        boxes: &[ac2_ui::OverlayBox],
        blips: &[ac2_ui::RadarBlip],
        selection: u64,
        range: f32,
    ) {
        let Some((tree, _)) = &mut self.hud else {
            return;
        };
        self.panels
            .update_overlays(tree, boxes, blips, selection, range);
    }

    /// `gmExternalContainerUI::PositionWindow` over the container's box, when it wants a place.
    pub(crate) fn position_external_container(&mut self, boxes: &[ac2_ui::OverlayBox]) {
        let container = self.external_container_position();
        let HudView {
            hud, dirty, panels, ..
        } = self;
        if let Some((tree, _)) = hud {
            panels.position_external_container_over(tree, dirty, container, boxes);
        }
    }

    /// `gm3DTextUI::Update`'s reticle for this frame: the selection (id and name) and, when it is
    /// drawn, its box. Then the HUD is composed again or its surfaces moved.
    pub(crate) fn reticle(
        &mut self,
        selection: Option<(u64, Option<&str>)>,
        info: Option<&ac2_net::protocol::SelectionInfo>,
        selected: Option<&ac2_ui::OverlayBox>,
    ) -> ReticleOutcome {
        let Some((tree, _)) = &mut self.hud else {
            return ReticleOutcome::default();
        };
        self.panels
            .update_reticle_over(tree, selection, info, selected)
    }

    /// `gmRadarUI::Update` for this frame; the next compose paints it.
    pub(crate) fn update_radar(&mut self, mut view: ac2_ui::RadarView) {
        view.coordinates = self.panels.radar_coordinates.clone();
        if let (Some((tree, _)), Some(radar)) = (&mut self.hud, &mut self.panels.radar) {
            radar.update(tree, &view);
        }
    }
}

/// The tree's messages to the chat until it broadcasts no more (activating an entry line
/// broadcasts `activated`, which the chat answers); all of them, in order.
pub(crate) fn pump(
    tree: &mut UiTree,
    chat: &mut Option<ChatUi>,
    toolbar: &Option<ToolbarUi>,
    modifiers: Modifiers,
) -> Vec<UiMessage> {
    let mut all = Vec::new();
    loop {
        let messages = tree.take_messages();
        if messages.is_empty() {
            return all;
        }
        for message in &messages {
            if let Some(chat) = chat.as_mut() {
                chat.message(tree, message, modifiers);
            }
            if let Some(toolbar) = toolbar {
                toolbar.message(tree, message);
            }
        }
        all.extend(messages);
    }
}
