//! The gameplay HUD over the world (`window` feature): the element tree
//! `gmGamePlayUI::SetupChildren`'s frameworks create, built and its images read on a worker
//! thread, then composed (`UIRegion::DrawHere`, software, as retail) whenever it arrives or the
//! window changes size, and drawn as one layer per root surface. The frame's UI actions go to
//! the tree (`UIElementManager::StartInput`), and it is composed again when they change it.
//! The tree's messages go to the chat (`gmChatUI`) before they are returned.

use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::sync::Arc;

use ac2_db::{ResourceResult, ResourceStore};
use ac2_gpu::{Gpu, UiLayer, UiOverlay};
use ac2_ui::{
    ChatRequest, ChatSettings, ChatUi, GameplayHud, Modifiers, Reticle, ReticleOutcome,
    ReticleTarget, ShortcutUi, SkillTreeUi, ToolbarChild, ToolbarUi, UiMessage, UiPixels, UiTree,
};

type Built = ResourceResult<(GameplayHud, HashMap<u32, UiPixels>)>;

pub(crate) struct HudView {
    pending: Option<Receiver<Built>>,
    pub(crate) hud: Option<(UiTree, HashMap<u32, UiPixels>)>,
    reticle: Option<Reticle>,
    pub(crate) chat: Option<ChatUi>,
    toolbar: Option<ToolbarUi>,
    pub(crate) shortcuts: Option<ShortcutUi>,
    pub(crate) skills: Option<SkillTreeUi>,
    pub(crate) examination: Option<ac2_ui::ExaminationUi>,
    pub(crate) allegiance: Option<ac2_ui::AllegianceUi>,
    pub(crate) fellowship: Option<ac2_ui::FellowshipUi>,
    pub(crate) dialogs: Option<ac2_ui::DialogFactory>,
    /// The tree's messages for the frameworks that ask the player
    /// ([`HudView::frameworks`](crate::hud_player)).
    pub(crate) player_messages: Vec<UiMessage>,
    /// Every string table, loaded on a worker, and the UI table's DID (enum `0x41000018`).
    pub(crate) strings: Option<Arc<crate::ui_sources::UiStringTables>>,
    pending_strings: Option<Receiver<crate::ui_sources::UiStringTables>>,
    /// The frameworks that ask the player have been loaded from it.
    pub(crate) player_loaded: bool,
    /// Lines displayed before the HUD arrived (`RecvNotice_DisplayStringInfo`).
    early_lines: Vec<(u32, String)>,
    /// Images set after the build, loaded on demand.
    images: crate::ui_images::UiImageLoader,
    /// The chat lines being rendered (`StringDownload`).
    chat_text: crate::chat_text::ChatText,
    overlay: UiOverlay,
    size: (u32, u32),
    pub(crate) dirty: bool,
    /// The sounds the UI's media played this frame (`MD_Data_Sound`), for an audio sink.
    pub(crate) sounds: Vec<u32>,
}

impl HudView {
    pub(crate) fn new(
        store: Arc<ResourceStore>,
        gpu: &Gpu,
        format: wgpu::TextureFormat,
        size: (u32, u32),
    ) -> Self {
        let chat_text = crate::chat_text::ChatText::new((*store).clone());
        let images = crate::ui_images::UiImageLoader::new(Arc::clone(&store));
        let (send_strings, pending_strings) = mpsc::channel();
        let strings_store = Arc::clone(&store);
        std::thread::Builder::new()
            .name("ac2 ui strings".into())
            .spawn(move || {
                let tables = ac2_db::PreloadedStringTables::load_all(&strings_store, 0x5800_0000);
                let _ = send_strings.send(crate::ui_sources::UiStringTables::new(
                    &strings_store,
                    tables,
                ));
            })
            .expect("spawn the UI strings loader");
        let (send, receive) = mpsc::channel();
        let (width, height) = (size.0 as i32, size.1 as i32);
        std::thread::Builder::new()
            .name("ac2 hud".into())
            .spawn(move || {
                let built = ac2_ui::build_gameplay_hud(&store, width, height).and_then(|hud| {
                    let images = ac2_ui::load_ui_images(&store, &hud.tree)?;
                    Ok((hud, images))
                });
                let _ = send.send(built);
            })
            .expect("spawn the HUD loader");
        Self {
            pending: Some(receive),
            hud: None,
            reticle: None,
            chat: None,
            toolbar: None,
            shortcuts: None,
            skills: None,
            examination: None,
            allegiance: None,
            fellowship: None,
            dialogs: None,
            player_messages: Vec::new(),
            strings: None,
            pending_strings: Some(pending_strings),
            player_loaded: false,
            early_lines: Vec::new(),
            chat_text,
            images,
            overlay: UiOverlay::new(gpu, format),
            size,
            dirty: false,
            sounds: Vec::new(),
        }
    }

    /// The render device's new size: every root placed again (`UIElement::UpdateSizeAndPosition`).
    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        self.size = (width.max(1), height.max(1));
        if let Some((tree, _)) = &mut self.hud {
            tree.set_display(self.size.0 as i32, self.size.1 as i32);
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
        let Some((tree, _)) = &mut self.hud else {
            return Vec::new();
        };
        for &(action, extent, start) in actions {
            if start {
                tree.start_input(action, extent);
            } else {
                tree.stop_input(action);
            }
        }
        let messages = pump(tree, &mut self.chat, &self.toolbar, modifiers);
        self.player_messages.extend(messages.iter().cloned());
        if tree.take_changed() {
            self.dirty = true;
        }
        messages
    }

    /// The active text element's actions and characters (`UIElement_Text::ActionHandler` /
    /// `CharacterHandler`); returns the messages they broadcast.
    pub(crate) fn text_input(
        &mut self,
        actions: &[(u32, f32)],
        characters: &[(u16, u32)],
        modifiers: Modifiers,
    ) -> Vec<UiMessage> {
        let ctrl = modifiers.ctrl;
        let Some((tree, _)) = &mut self.hud else {
            return Vec::new();
        };
        for &(action, extent) in actions {
            tree.text_action(action, extent, ctrl);
        }
        for &(ch, key) in characters {
            tree.character(ch, key, ctrl);
        }
        let messages = pump(tree, &mut self.chat, &self.toolbar, modifiers);
        self.player_messages.extend(messages.iter().cloned());
        if tree.take_changed() {
            self.dirty = true;
        }
        messages
    }

    /// What the chat asked since the last call (lines to send, history steps).
    pub(crate) fn take_chat_requests(&mut self) -> Vec<ChatRequest> {
        self.chat
            .as_mut()
            .map_or_else(Vec::new, ChatUi::take_requests)
    }

    /// `CM_UI::SendNotice_OpenExaminationPanel` (`gmExaminationUI::RecvNotice_OpenExaminationPanel`).
    pub(crate) fn open_examination(&mut self) {
        if let (Some((tree, _)), Some(toolbar)) = (&mut self.hud, &self.toolbar) {
            toolbar.show(tree, ToolbarChild::Examination, true);
            if tree.take_changed() {
                self.dirty = true;
            }
        }
    }

    /// `ChatWindow::RecvNotice_ReloadChatSettings` with the player's chat options.
    pub(crate) fn reload_chat_settings(&mut self, settings: &ChatSettings) {
        if let (Some((tree, _)), Some(chat)) = (&mut self.hud, &mut self.chat) {
            chat.reload_chat_settings(tree, settings);
            if tree.take_changed() {
                self.dirty = true;
            }
        }
    }

    /// `ChatWindow::DisplayCmdHistory`'s answer: the entry line shows `text`.
    pub(crate) fn set_chat_entry(&mut self, text: &str) {
        if let (Some((tree, _)), Some(chat)) = (&mut self.hud, &mut self.chat) {
            chat.set_entry_text(tree, text);
            if tree.take_changed() {
                self.dirty = true;
            }
        }
    }

    /// `ChatWindow::DoAssistedTell`: the entry line becomes a tell to `target`.
    pub(crate) fn assisted_tell(&mut self, target: &str) {
        if let (Some((tree, _)), Some(chat)) = (&mut self.hud, &mut self.chat) {
            chat.assisted_tell(tree, target);
            if tree.take_changed() {
                self.dirty = true;
            }
        }
    }

    /// The skills panel's panel and selected skill.
    pub(crate) fn skills_current(&self) -> Option<(u32, u32)> {
        self.skills.as_ref().map(ac2_ui::SkillTreeUi::current)
    }

    /// The examination panel's texts, in order.
    pub(crate) fn allegiance_texts(&self) -> Vec<String> {
        let (Some((tree, _)), Some(page)) = (&self.hud, &self.allegiance) else {
            return Vec::new();
        };
        fn walk(tree: &UiTree, index: usize, out: &mut Vec<String>) {
            if let Some(text) = tree.get(index).and_then(|e| e.text()) {
                if !text.is_empty() {
                    out.push(text);
                }
            }
            for &child in tree.children(index) {
                walk(tree, child, out);
            }
        }
        let mut out = Vec::new();
        for row in page.rows(tree) {
            walk(tree, row, &mut out);
        }
        out
    }

    pub(crate) fn fellowship_rows(&self) -> Vec<(u64, String)> {
        let (Some((tree, _)), Some(panel)) = (&self.hud, &self.fellowship) else {
            return Vec::new();
        };
        let name = ac2_ui::tokenize("FellowTemplateNameText");
        panel
            .rows(tree)
            .into_iter()
            .map(|(id, row)| {
                let text = tree
                    .child_recursive(row, name)
                    .and_then(|t| tree.get(t))
                    .and_then(|e| e.text());
                (id, text.unwrap_or_default())
            })
            .collect()
    }

    pub(crate) fn examination_texts(&self) -> Vec<String> {
        let (Some((tree, _)), Some(panel)) = (&self.hud, &self.examination) else {
            return Vec::new();
        };
        fn walk(tree: &UiTree, index: usize, out: &mut Vec<String>) {
            if let Some(text) = tree.get(index).and_then(|e| e.text()) {
                if !text.is_empty() {
                    out.push(text);
                }
            }
            for &child in tree.children(index) {
                walk(tree, child, out);
            }
        }
        let mut out = Vec::new();
        if let Some(list) = panel.list() {
            for &row in tree.list_items(list) {
                walk(tree, row, &mut out);
            }
        }
        out
    }

    /// The shortcut bar's slots.
    pub(crate) fn shortcut_slots(&self) -> usize {
        let (Some((tree, _)), Some(bar)) = (&self.hud, &self.shortcuts) else {
            return 0;
        };
        bar.elements()
            .1
            .map_or(0, |list| tree.list_items(list).len())
    }

    /// The first chat window's log text.
    pub(crate) fn chat_log(&self) -> Option<String> {
        let (tree, _) = self.hud.as_ref()?;
        let log = self.chat.as_ref()?.logs().first().copied().flatten()?;
        tree.get(log)?.text()
    }

    /// Whether a toolbar panel is shown.
    pub(crate) fn panel_shown(&self, child: ToolbarChild) -> bool {
        match (&self.hud, &self.toolbar) {
            (Some((tree, _)), Some(toolbar)) => toolbar.shown(tree, child),
            _ => false,
        }
    }

    /// `CM_Communication::SendNotice_DisplayStringInfo`: the line is rendered on a worker and
    /// reaches the chat windows when it is done.
    pub(crate) fn chat_display_info(&mut self, kind: u32, info: &ac2_net::protocol::StringInfo) {
        self.chat_text.render(kind, info);
    }

    /// `CM_Communication::SendNotice_DisplayStringInfo`'s text for the chat windows.
    pub(crate) fn chat_display(&mut self, kind: u32, text: &str) {
        match &mut self.chat {
            Some(chat) => chat.display(kind, text),
            None => self.early_lines.push((kind, text.to_owned())),
        }
    }

    /// Whether an editable text element is active (the input manager's text mode).
    pub(crate) fn text_mode(&self) -> bool {
        self.hud.as_ref().is_some_and(|(tree, _)| tree.text_mode())
    }

    fn arrived(&mut self, built: Built) {
        match built {
            Ok((
                GameplayHud {
                    mut tree,
                    reticle,
                    mut chat,
                    toolbar,
                    shortcuts,
                    skills,
                    examination,
                    allegiance,
                    fellowship,
                    dialogs,
                },
                images,
            )) => {
                tree.set_display(self.size.0 as i32, self.size.1 as i32);
                if let Some(chat) = &mut chat {
                    // The player's options are not read yet: the chat starts on
                    // `GameplayOptionsProfile::Clear`'s (no filters, default fonts).
                    chat.reload_chat_settings(&mut tree, &ChatSettings::default());
                    for (kind, text) in self.early_lines.drain(..) {
                        chat.display(kind, &text);
                    }
                }
                self.hud = Some((tree, images));
                self.reticle = reticle;
                self.chat = chat;
                self.toolbar = Some(toolbar);
                self.shortcuts = shortcuts;
                self.skills = skills;
                self.examination = examination;
                self.allegiance = allegiance;
                self.fellowship = fellowship;
                self.dialogs = Some(dialogs);
                self.dirty = true;
            }
            Err(error) => eprintln!("ac2: the HUD did not load: {error}"),
        }
    }

    /// Takes a finished build, then recomposes if anything changed.
    /// One frame of the HUD at `Timer::cur_time` `now` (seconds).
    pub(crate) fn update(&mut self, gpu: &Gpu, now: f64) {
        if let Some(pending) = &self.pending_strings {
            if let Ok(tables) = pending.try_recv() {
                self.strings = Some(Arc::new(tables));
                self.pending_strings = None;
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
            self.sounds = tree.take_sounds();
            if let Some(chat) = &mut self.chat {
                chat.global_loop(tree);
            }
            if tree.take_changed() {
                self.dirty = true;
            }
        }
        self.compose(gpu);
    }

    /// Waits for the build (headless snapshots).
    pub(crate) fn settle(&mut self, gpu: &Gpu) {
        if let Some(pending) = self.pending.take() {
            if let Ok(built) = pending.recv() {
                self.arrived(built);
            }
        }
        self.compose(gpu);
    }

    /// `gm3DTextUI::Update`'s reticle for this frame: the selection (id and name) and, when it
    /// is drawn, its box and projection. Then the HUD is composed again or its surfaces moved.
    pub(crate) fn reticle(
        &mut self,
        gpu: &Gpu,
        selection: Option<(u64, Option<&str>)>,
        target: Option<ReticleTarget<'_>>,
    ) -> ReticleOutcome {
        let (Some((tree, _)), Some(reticle)) = (&mut self.hud, &mut self.reticle) else {
            return ReticleOutcome::default();
        };
        let id = selection.map(|(id, _)| id);
        if reticle.current() != id {
            reticle.selection_changed(tree, id, selection.and_then(|(_, name)| name));
        }
        let outcome = reticle.update(tree, target, false);
        if tree.take_changed() {
            self.dirty = true;
        }
        self.compose(gpu);
        outcome
    }

    fn compose(&mut self, gpu: &Gpu) {
        let Some((tree, images)) = &mut self.hud else {
            return;
        };
        if !self.dirty {
            if tree.take_moved() {
                let placements = tree.placements();
                self.overlay.set_placements(
                    placements
                        .into_iter()
                        .map(|p| (p.x, p.y, p.rotation, p.shown, p.opacity)),
                );
            }
            return;
        }
        tree.take_moved();
        self.dirty = false;
        self.images.request_missing(tree, images);
        tree.take_changed();
        let surfaces = tree.compose(images);
        self.overlay.set_layers(
            gpu,
            surfaces.iter().map(|surface| UiLayer {
                x: surface.x,
                y: surface.y,
                width: surface.width,
                height: surface.height,
                rotation: surface.rotation,
                shown: surface.shown,
                opacity: surface.opacity,
                rgba: &surface.rgba,
            }),
        );
    }

    /// The pointer as the UI last saw it.
    pub(crate) fn pointer(&self) -> (i32, i32) {
        self.hud.as_ref().map_or((0, 0), |(tree, _)| tree.pointer())
    }

    pub(crate) fn shown_layers(&self) -> usize {
        self.overlay.shown_layers()
    }

    pub(crate) fn layers(&self) -> usize {
        self.overlay.layers()
    }

    pub(crate) fn prepare(&self, queue: &wgpu::Queue) {
        self.overlay.prepare(queue, self.size.0, self.size.1);
    }

    pub(crate) fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        self.overlay.draw(pass);
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
