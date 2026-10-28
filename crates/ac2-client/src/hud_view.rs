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
    ChatRequest, ChatSettings, ChatUi, CraftUi, GameplayChild, GameplayHud, IngredientUi,
    Modifiers, Reticle, ReticleOutcome, ReticleTarget, ShortcutUi, SkillTreeUi, ToolbarChild,
    ToolbarUi, UiMessage, UiPixels, UiTree, GAMEPLAY_SHOW_KEY,
};

type Built = ResourceResult<(GameplayHud, HashMap<u32, UiPixels>)>;

mod accessors;
mod build;
mod compose;

pub(crate) struct HudView {
    pending: Option<Receiver<Built>>,
    pub(crate) hud: Option<(UiTree, HashMap<u32, UiPixels>)>,
    reticle: Option<Reticle>,
    /// `gm3DTextUI`'s name plates.
    pub(crate) names: Option<ac2_ui::NamePlates>,
    /// `gm3DTextUI`'s floating damage numbers.
    pub(crate) damage: Option<ac2_ui::DamageTexts>,
    /// The HUD's clock (`Timer::cur_time`) at the last frame: a damage text starts at it.
    pub(crate) now: f64,
    pub(crate) chat: Option<ChatUi>,
    pub(crate) toolbar: Option<ToolbarUi>,
    pub(crate) shortcuts: Option<ShortcutUi>,
    pub(crate) skills: Option<SkillTreeUi>,
    /// `gmCraftUI`: the craft panel on the skill toolbar.
    pub(crate) craft: Option<CraftUi>,
    /// `gmIngredientUI`: the ingredient panel of a recipe.
    pub(crate) ingredient: Option<IngredientUi>,
    pub(crate) examination: Option<ac2_ui::ExaminationUi>,
    pub(crate) allegiance: Option<ac2_ui::AllegianceUi>,
    pub(crate) fellowship: Option<ac2_ui::FellowshipUi>,
    /// `gmFriendsUI`: the friends page of the social toolbar.
    pub(crate) friends: Option<ac2_ui::FriendsPanel>,
    /// `gmAssistanceUI`: the assistance page of the option toolbar.
    pub(crate) assistance: Option<ac2_ui::AssistancePanel>,
    /// `gmSocialUI`: the social panel's buttons.
    pub(crate) social: Option<ac2_ui::SocialPanel>,
    /// `gmMiniFellowUI`: the fellowship mini panel on the HUD.
    pub(crate) mini_fellow: Option<ac2_ui::MiniFellowUi>,
    /// `gmPetUI`: the pet panel on the HUD.
    pub(crate) pets: Option<ac2_ui::PetUi>,
    /// `gmQuestsUI`: the Quest panel and its quest journal.
    pub(crate) quests: Option<ac2_ui::QuestsUi>,
    pub(crate) vitals: Option<ac2_ui::VitalsUi>,
    pub(crate) effects: Option<ac2_ui::EffectsUi>,
    pub(crate) vitae: Option<ac2_ui::VitaeStatusUi>,
    /// `gmPKStatusUI`, the PK indicator, fed by `CM_Character::SendNotice_UpdateFactionStatus`.
    pub(crate) pk: Option<ac2_ui::PkStatusUi>,
    /// `gmNetStatusUI`, the net indicator, fed by each world heartbeat's link averages.
    pub(crate) net: Option<ac2_ui::NetStatusUi>,
    /// `gmInventoryUI`'s backpack, made once the player is in the world, over this layout.
    pub(crate) backpack: Option<ac2_ui::BackpackUi>,
    /// `gmInventoryUI`'s paper doll, made with the backpack.
    pub(crate) paper_doll: Option<ac2_ui::PaperDollUi>,
    /// The player's race (int 290) as the frameworks last read it (the doll's height).
    pub(crate) player_race: Option<i32>,
    pub(crate) toolbar_layout: Option<Arc<ac2_ui::LayoutDesc>>,
    /// The selection the item slots last saw.
    pub(crate) item_selection: u64,
    /// `gm3DItemsUI`, made with the backpack.
    pub(crate) items_3d: Option<ac2_ui::Items3dUi>,
    /// The selection info the reticle's meters last took.
    reticle_info: Option<ac2_net::protocol::SelectionInfo>,
    pub(crate) dialogs: Option<ac2_ui::DialogFactory>,
    /// `gmGamePlayUI::m_shown`: the HUD is shown (`gmGamePlayUI::Show`, the `0x4100000C` key).
    pub(crate) gameplay_shown: bool,
    /// The gameplay panels' show state before a story hid them (`None` with no story playing).
    pub(crate) story_hid_gameplay: Option<bool>,
    /// The `0x4100000C` key was pressed this frame; [`HudView::frameworks`] runs it, where the
    /// client's story state is.
    hud_key_pending: bool,
    /// The `0x4100000D` use key was pressed this frame ([`HudView::take_use_key`]).
    use_key_pending: bool,
    /// `gmGamePlayUI`'s death dialog (`_deathContext`).
    pub(crate) death: ac2_ui::DeathPanel,
    /// `gmGamePlayUI`'s wait, message and offer contexts (`_waitContext`, `_messageContexts`).
    pub(crate) game_dialogs: ac2_ui::GameplayDialogs,
    /// The script questions raised and not yet asked: they wait for the HUD's dialogs (see
    /// [`HudView::frameworks`](crate::hud_player)).
    pub(crate) popup_backlog: Vec<ac2_ui::PopupQuestion>,
    /// The message boxes raised in the world and not yet shown (`RecvNotice_DisplayMessage`
    /// before the HUD's dialogs exist); retail's handler is registered with the HUD's children,
    /// so these wait for the HUD rather than being dropped.
    pub(crate) display_backlog: Vec<ac2_ui::DisplayNotice>,
    /// `gmStackUI`.
    pub(crate) stack: Option<ac2_ui::StackUi>,
    /// `gmExternalContainerUI`.
    pub(crate) external_container: Option<ac2_ui::ExternalContainerUi>,
    /// `gmSecureTradeUI`, hidden until a trade opens (its requests wait in the panel).
    pub(crate) secure_trade: Option<ac2_ui::SecureTradeUi>,
    /// `gmCharacterUI`, the character options page (its requests wait in the page).
    pub(crate) character: Option<ac2_ui::CharacterUi>,
    /// `gmChatOptionsUI`, the chat options page (its requests wait in the page).
    pub(crate) chat_options: Option<ac2_ui::ChatOptionsUi>,
    /// `gmClientUI`, the client options page, and the effects its applies left.
    pub(crate) options: ac2_ui::OptionsPages,
    /// `gmShopUI`, the vendor's panel (its requests wait in the panel).
    pub(crate) shop: Option<ac2_ui::ShopUi>,
    /// `gmShopCatalogUI`, the storekeeper's catalog (its requests wait in the panel).
    pub(crate) catalog: Option<ac2_ui::CatalogUi>,
    /// `gmShopNewSaleUI`, the consignment's new-sale dialog (its requests wait in the panel).
    pub(crate) new_sale: Option<ac2_ui::NewSaleUi>,
    /// `gmShopConsignmentUI`, the storekeeper's consignment (its requests wait in the panel).
    pub(crate) consignment: Option<ac2_ui::ConsignmentUi>,
    /// `gmRadarUI`'s blips, painted over the radar field each compose.
    pub(crate) radar: Option<ac2_ui::RadarUi>,
    /// The player's cell and its map coordinates as the radar shows them (rendered by the
    /// frameworks, which have the string tables).
    pub(crate) radar_cell: Option<u32>,
    pub(crate) radar_coordinates: Option<String>,
    /// The blips last uploaded over the radar field's layer. The field is composed and uploaded
    /// again only when these change (or the whole HUD is composed).
    pub(crate) radar_uploaded: Option<Vec<(i32, i32, [u8; 4])>>,
    /// The navigation panel's game date and time, rendered for the calendar they were rendered
    /// for (`GameTime::InqCurrentGameDateTimeString`).
    pub(crate) navigation_time: Option<(ac2_world::GameDateTime, String, String)>,
    /// `gmNavigationUI::AddMarker`'s tooltips, one per frame marker (rendered on the frame).
    pub(crate) navigation_tooltips: Vec<String>,
    /// `gmNavigationUI`'s macro map (its map image is set on the worker) and `gmOverviewMapUI`.
    pub(crate) navigation: Option<ac2_ui::NavigationUi>,
    pub(crate) overview: Option<ac2_ui::OverviewMapUi>,
    /// `gmBookUI`, shown by the book's use notice.
    pub(crate) book: Option<ac2_ui::BookUi>,
    /// The help window (`KeyStone::OpenHelp`), hidden until a help page opens.
    pub(crate) help: Option<ac2_ui::HelpWindow>,
    /// The help pages, read once on the HUD's worker.
    pub(crate) help_pages: Option<ac2_ui::HelpPages>,
    /// `gmStoryUI`'s background and story fields, shown while a story holds the slot.
    pub(crate) story: Option<ac2_ui::StoryView>,
    /// `gmMoneyBagUI`, made with the backpack.
    pub(crate) money: Option<ac2_ui::MoneyBagUi>,
    /// Whether shift is held (`GetKeyState(VK_SHIFT)` for the melting confirmation), as the
    /// frame's input last said.
    pub(crate) shift_held: bool,
    /// The tree's messages for the frameworks that ask the player
    /// ([`HudView::frameworks`](crate::hud_player)).
    pub(crate) player_messages: Vec<UiMessage>,
    /// Every string table, loaded on a worker, and the UI table's DID (enum `0x41000018`).
    pub(crate) strings: Option<Arc<ac2_ui::UiStringTables>>,
    pending_strings: Option<Receiver<ac2_ui::UiStringTables>>,
    /// The frameworks that ask the player have been loaded from it.
    pub(crate) player_loaded: bool,
    /// Lines displayed before the HUD arrived (`RecvNotice_DisplayStringInfo`).
    early_lines: Vec<(u32, String)>,
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
        self.shift_held = modifiers.shift;
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
        let messages = pump(tree, &mut self.chat, &self.toolbar, modifiers);
        self.player_messages.extend(messages.iter().cloned());
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
                self.hud_key_pending = true;
            } else if key == ac2_ui::USE_KEY {
                self.use_key_pending = true;
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
            GameplayChild::Toolbar => self.toolbar.as_ref().is_some_and(|t| t.bar_shown(tree)),
            GameplayChild::Statusbar => ac2_ui::statusbar_shown(tree),
            GameplayChild::Radar => self.radar.as_ref().is_some_and(|r| r.shown(tree)),
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
                if let (Some((tree, _)), Some(radar)) = (&mut self.hud, &self.radar) {
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
        std::mem::take(&mut self.use_key_pending)
    }

    /// `gmGamePlayUI::ListenToMessage` case `0x4100000C`: the HUD toggles unless a death dialog is
    /// open or a logout is requested, and `gmGamePlayUI::Show` ignores it while a please-wait
    /// dialog or a story holds the UI.
    pub(crate) fn gameplay_show_key(&mut self, story_active: bool) {
        if !std::mem::take(&mut self.hud_key_pending) {
            return;
        }
        if self.death.context() != 0 || self.game_dialogs.end_session_requested() {
            return;
        }
        if self.game_dialogs.wait_context() != 0 || story_active {
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

    /// `gmToolbarUI::Show`: the toolbar field shows or hides, and its height is the bottom
    /// preclude the reticle and the name plates keep (`CM_UI::SendNotice_PrecludeFromBottom`,
    /// which the next frame's update applies).
    pub(crate) fn show_toolbar(&mut self, shown: bool) {
        if let (Some((tree, _)), Some(toolbar), Some(reticle)) =
            (&mut self.hud, &self.toolbar, &mut self.reticle)
        {
            reticle.precluded_bottom = toolbar.show_bar(tree, shown);
            self.dirty = true;
        }
        // `gmShortcutUI::Show` from `gmToolbarUI::Show`.
        if let (Some((tree, _)), Some(bar)) = (&mut self.hud, &self.shortcuts) {
            bar.show(tree, shown);
        }
    }

    /// `gmToolbarUI`'s panel show (a toolbar button or key).
    pub(crate) fn show_panel(&mut self, child: ToolbarChild, shown: bool) {
        if let (Some((tree, _)), Some(toolbar)) = (&mut self.hud, &self.toolbar) {
            toolbar.show(tree, child, shown);
            // `gmQuestsUI::Show` when the Quest child changed.
            if let Some(quests) = self.quests.as_mut() {
                quests.sync(tree, toolbar.shown(tree, ToolbarChild::Quest));
            }
            if let Some(doll) = &self.paper_doll {
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
        match &mut self.chat {
            Some(chat) => chat.display(kind, text),
            None => self.early_lines.push((kind, text.to_owned())),
        }
    }

    /// One frame of the HUD at `Timer::cur_time` `now` (seconds).
    pub(crate) fn update(&mut self, gpu: &Gpu, now: f64) {
        self.now = now;
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
            if let Some(doll) = &self.paper_doll {
                doll.follow_panel(tree);
            }
            // `gmShortcutUI::UpdateTimeLeft`: the skills on reset fill their slots' meters.
            if let Some(bar) = &mut self.shortcuts {
                bar.update_time_left(tree, now);
            }
            self.sounds.extend(tree.take_sounds());
            if let Some(chat) = &mut self.chat {
                chat.global_loop(tree);
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

    /// `gm3DTextUI::UpdateDamage`: the damage texts over `places` at the HUD's clock, rising
    /// above the selection's reticle and name when `selection` is the object they are over.
    pub(crate) fn update_damage(&mut self, places: &[ac2_ui::DamagePlace<'_>], selection: u64) {
        let (Some((tree, _)), Some(damage)) = (&mut self.hud, &mut self.damage) else {
            return;
        };
        let offset = self
            .reticle
            .as_ref()
            .map_or(0, |r| r.selection_offset(tree));
        let precluded = self
            .reticle
            .as_ref()
            .map_or((0, 0), |r| (r.precluded_top, r.precluded_bottom));
        damage.update(
            tree,
            places,
            (selection, offset),
            precluded,
            self.now,
            false,
        );
        if tree.take_changed() {
            self.dirty = true;
        }
    }

    /// `gm3DTextUI::UpdateNames` after the reticle: the plates over `places`, clear of the
    /// selection's reticle, within `range` (the radar's, squared as a float).
    pub(crate) fn update_names(
        &mut self,
        places: &[ac2_ui::NamePlace<'_>],
        selection: u64,
        range: f32,
    ) {
        let (Some((tree, _)), Some(names)) = (&mut self.hud, &mut self.names) else {
            return;
        };
        let rect = self.reticle.as_ref().and_then(|r| r.selection_rect());
        let precluded = self
            .reticle
            .as_ref()
            .map_or((0, 0), |r| (r.precluded_top, r.precluded_bottom));
        names.update(
            tree,
            places,
            selection,
            rect,
            range * range,
            precluded,
            false,
        );
    }

    /// `gmExternalContainerUI::PositionWindow` with the container's projected centre.
    pub(crate) fn position_external_container(&mut self, clip: Option<[f32; 2]>) {
        if let (Some((tree, _)), Some(panel)) = (&mut self.hud, &mut self.external_container) {
            panel.position_window_clip(tree, clip);
            self.dirty = true;
        }
    }

    /// `gm3DTextUI::Update`'s reticle for this frame: the selection (id and name) and, when it
    /// is drawn, its box and projection. Then the HUD is composed again or its surfaces moved.
    pub(crate) fn reticle(
        &mut self,
        selection: Option<(u64, Option<&str>)>,
        info: Option<&ac2_net::protocol::SelectionInfo>,
        target: Option<ReticleTarget<'_>>,
    ) -> ReticleOutcome {
        let (Some((tree, _)), Some(reticle)) = (&mut self.hud, &mut self.reticle) else {
            return ReticleOutcome::default();
        };
        let id = selection.map(|(id, _)| id);
        if reticle.current() != id {
            reticle.selection_changed(tree, id, selection.and_then(|(_, name)| name));
            self.reticle_info = None;
        }
        // `RecvNotice_UpdateSelectionInfo`, once per new info.
        if let Some(info) = info.filter(|i| self.reticle_info.as_ref() != Some(i)) {
            reticle.selection_info(tree, info);
            self.reticle_info = Some(*info);
        }
        reticle.update(tree, target, false)
    }

    /// `gmRadarUI::Update` for this frame; the next compose paints it.
    pub(crate) fn update_radar(&mut self, mut view: ac2_ui::RadarView) {
        view.coordinates = self.radar_coordinates.clone();
        if let (Some((tree, _)), Some(radar)) = (&mut self.hud, &mut self.radar) {
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
