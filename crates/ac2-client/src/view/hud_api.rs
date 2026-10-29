//! The HUD's and the frameworks' entry points: the frame's UI input, the chat, the panels and
//! pages the gameplay code reads and drives. Each forwards to `HudView`.

use super::ClientView;

impl ClientView {
    /// The sounds the HUD's media played since the last call (`MD_Data_Sound`), for
    /// [`crate::play_ui_sounds`].
    pub fn take_ui_sounds(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.hud.sounds)
    }

    /// The frame's UI actions for the HUD ([`ac2_ui::Controls::take_ui_input`]); returns the
    /// messages they broadcast, for [`ac2_ui::Controls::ui_messages`].
    pub fn ui_input(
        &mut self,
        actions: &[(u32, f32, bool)],
        modifiers: ac2_ui::Modifiers,
    ) -> Vec<ac2_ui::UiMessage> {
        self.hud.input(actions, modifiers)
    }

    /// The typing for the HUD's active text element ([`ac2_ui::Controls::take_text_input`]);
    /// returns the messages it broadcast.
    pub fn text_input(
        &mut self,
        actions: &[(u32, f32)],
        characters: &[(u16, u32)],
        modifiers: ac2_ui::Modifiers,
    ) -> Vec<ac2_ui::UiMessage> {
        self.hud.text_input(actions, characters, modifiers)
    }

    /// What the HUD's chat asked since the last call ([`ac2_ui::ChatRequest`]).
    pub fn take_chat_requests(&mut self) -> Vec<ac2_ui::ChatRequest> {
        self.hud.take_chat_requests()
    }

    /// The HUD frameworks that ask the player (skills panel, shortcut bar, chat options), once a
    /// frame after `Client::use_time`; returns the gameplay commands they ask for.
    pub fn frameworks(
        &mut self,
        client: &mut crate::Client,
        frame: &crate::ClientFrame,
        events: &[ac2_weenie::SessionEvent],
        script_notices: &[ac2_script::host::ScriptNotice],
    ) -> Vec<crate::Command> {
        // `gmGamePlayUI::ListenToMessage` 0x4100000C: the HUD toggle, with the story's state.
        let story_active = client.story.is_active();
        self.hud.gameplay_show_key(story_active);
        // The paper doll's pick from the scene as last drawn (`FinishPaperDollClick` runs on
        // the loop after the selection render).
        let doll_pick = self.hud.doll_wants_pick().map(|point| {
            let viewport = self.hud.doll_viewport()?;
            self.doll.pick(point, &viewport)
        });
        // `gm3DItemsUI`: a drop's target search finishes with the object under the pointer.
        let world_pick = self.hud.world_pick_point().map(|point| {
            let size = self.hud.window_size();
            self.pick(frame, size, point)
        });
        // `gmGamePlayUI::ListenToMessage` 0x4100000D: `gmCWInterface::UseSelection`.
        let mut commands: Vec<crate::Command> = self
            .hud
            .take_use_key()
            .then_some(crate::Command::UseSelection)
            .into_iter()
            .collect();
        commands.extend(self.hud.frameworks(
            client,
            frame,
            events,
            script_notices,
            (doll_pick, world_pick),
        ));
        commands
    }

    /// The modern additions' frame without a session (the world viewer): the quest guide, the
    /// world map and the Tracker over the frame's objects. Their gameplay commands (a Tracker
    /// pick's selection) have no session to go to and are dropped.
    pub fn offline_frameworks(&mut self, frame: &crate::ClientFrame) {
        self.hud.offline_frameworks(frame);
    }

    /// Opens or closes the quest guide (the In-Game Wiki button).
    pub fn toggle_guide(&mut self) {
        self.hud.toggle_addition(ac2_ui::Hud::toggle_guide);
    }

    /// Opens or closes the Tracker's window (its radar eyeball).
    pub fn toggle_tracker(&mut self) {
        self.hud.toggle_addition(ac2_ui::Hud::toggle_tracker);
    }

    /// Whether the player confirmed the quit question (`gmGamePlayUI::RecvNotice_Logoff`); the
    /// window closes after the frame that sends `UploadGameOptions`.
    pub fn take_quit_request(&mut self) -> bool {
        self.hud.take_quit_request()
    }

    /// `CM_UI::SendNotice_OpenExaminationPanel`: the examination panel shows.
    pub fn open_examination(&mut self) {
        self.hud.open_examination();
    }

    /// `RecvNotice_ReloadChatSettings`: the player's chat options (filters, fonts, popups).
    pub fn reload_chat_settings(&mut self, settings: &ac2_ui::ChatSettings) {
        self.hud.reload_chat_settings(settings);
    }

    /// The chat's command history answer: the entry line shows `text`.
    pub fn set_chat_entry(&mut self, text: &str) {
        self.hud.set_chat_entry(text);
    }

    /// The chat's retell / reply history answer (`ChatWindow::DoAssistedTell`).
    pub fn assisted_tell(&mut self, target: &str) {
        self.hud.assisted_tell(target);
    }

    /// The skills panel's panel and selected skill.
    pub fn skills_current(&self) -> Option<(u32, u32)> {
        self.hud.skills_current()
    }

    /// The HUD's text element `name` (`IUIManager` lookup by element id).
    pub fn element_text(&self, name: &str) -> Option<String> {
        self.hud.element_text(name)
    }

    /// The screen box of the HUD's element `name` (the first one with that name).
    pub fn element_box(&self, name: &str) -> Option<ac2_ui::UiBox> {
        self.hud.element_box(name)
    }

    /// The help window's text, while it shows (`KeyStone::OpenHelp`).
    pub fn help_window_text(&self) -> Option<String> {
        self.hud.help_window_text()
    }

    /// The screen box of the help window's element `name`.
    pub fn help_element_box(&self, name: &str) -> Option<ac2_ui::UiBox> {
        self.hud.help_element_box(name)
    }

    /// The secure trade panel: whether it shows, the player's offers and the partner's.
    pub fn trade(&self) -> Option<(bool, Vec<u64>, Vec<u64>)> {
        let panel = self.hud.panels.secure_trade.as_ref()?;
        Some((panel.is_shown(), panel.own_items(), panel.partner_items()))
    }

    /// The objects with a name plate (`gm3DTextUI`).
    pub fn named(&self) -> Vec<u64> {
        self.hud.named()
    }

    /// The character options page: its tab opens.
    pub fn character_open(&mut self) {
        self.hud.character_open();
    }

    /// The character options page: an option's checkbox is ticked or unticked.
    pub fn character_tick(&mut self, option: &str, on: bool) {
        self.hud.character_tick(option, on);
    }

    /// The character options page: one of its buttons is pressed (`ApplyButton`, ...); its
    /// requests become commands on the next [`View::frameworks`].
    pub fn character_press(&mut self, button: &str) {
        self.hud.character_press(button);
    }

    /// The character options page: an option's value as the page shows it.
    pub fn character_value(&self, option: &str) -> Option<f64> {
        self.hud.character_value(option)
    }

    /// A press on one of the navigation panel's buttons (`LockPositionButton`, `NavigateLeftButton`,
    /// ...), with Shift held or not (the short pan step). Returns whether the panel took it.
    pub fn navigation_press(&mut self, button: &str, shift: bool) -> bool {
        self.hud.navigation_press(button, shift)
    }

    /// A press on the navigation panel's map (`MapField`, `MapParentField`, `PlayerMarker`):
    /// the element's data code and the pixel relative to the map field.
    pub fn navigation_map_press(&mut self, element: &str, data: i32, pixel: (i32, i32)) -> bool {
        self.hud.navigation_map_press(element, data, pixel)
    }

    /// The navigation panel's lock to the character (`gmNavigationUI::ListenToMessage`'s lock):
    /// locking zooms to the character, unlocking leaves the window where it is.
    pub fn navigation_lock(&mut self, locked: bool) {
        self.hud.navigation_lock(locked);
    }

    /// The navigation panel's world map window (its centre in land cells and its zoom), when the
    /// panel draws the world map.
    pub fn world_map_window(&self) -> Option<ac2_ui::world_map_tiles::MapWindow> {
        self.hud.world_map_window()
    }

    /// The navigation panel and the tree it lives in, once the HUD is built.
    pub fn navigation(&mut self) -> Option<(&mut ac2_ui::UiTree, &mut ac2_ui::NavigationUi)> {
        self.hud.navigation_parts()
    }

    /// The radar's coordinates text.
    pub fn radar_coordinates(&self) -> Option<String> {
        self.hud.radar_coordinates()
    }

    /// The backpack's first page: each item and its slot's tooltip.
    pub fn backpack_items(&self) -> Vec<(u64, String)> {
        self.hud.backpack_items()
    }

    /// The external container panel's container and items, while it shows one.
    pub fn external_container_items(&self) -> Option<(u64, Vec<u64>)> {
        self.hud.external_container_items()
    }

    /// The item the paper doll shows worn at an inventory location.
    pub fn paper_doll_item(&self, location: u32) -> Option<u64> {
        self.hud.panels.paper_doll.as_ref()?.item_at(location)
    }

    /// Shows or hides a toolbar panel, as its button does.
    pub fn show_panel(&mut self, child: ac2_ui::ToolbarChild, shown: bool) {
        self.hud.show_panel(child, shown);
    }

    /// The paper doll's viewport while it shows.
    pub fn paper_doll_viewport(&self) -> Option<ac2_ui::SceneViewport> {
        self.hud.doll_viewport()
    }

    /// The paper doll scene's pick at a screen point: the mesh type of the avatar's part there.
    pub fn paper_doll_pick(&self, point: (i32, i32)) -> Option<u32> {
        self.doll.pick(point, &self.hud.doll_viewport()?)
    }

    /// The allegiance page's member list texts, in order.
    pub fn allegiance_texts(&self) -> Vec<String> {
        self.hud.allegiance_texts()
    }

    /// The fellowship page's rows, top first: each member's id and name.
    pub fn fellowship_rows(&self) -> Vec<(u64, String)> {
        self.hud.fellowship_rows()
    }

    /// The examination panel's texts, in order.
    pub fn examination_texts(&self) -> Vec<String> {
        self.hud.examination_texts()
    }

    /// The net indicator's tooltip and state (`gmNetStatusUI`), once a heartbeat has set them.
    pub fn net_indicator(&self) -> Option<(String, u32)> {
        self.hud.net_indicator()
    }

    /// The gameplay HUD's element tree, once it is built (the windows' places).
    pub fn hud_tree(&self) -> Option<&ac2_ui::UiTree> {
        self.hud.tree()
    }

    /// The HUD panels the build made (`mini_fellow`, `pets`, `strings`), by name.
    pub fn built_panels(&self) -> Vec<&'static str> {
        self.hud.built_panels()
    }

    /// The shortcut bar's slots.
    pub fn shortcut_slots(&self) -> usize {
        self.hud.shortcut_slots()
    }

    /// The first chat window's log text.
    pub fn chat_log(&self) -> Option<String> {
        self.hud.chat_log()
    }

    /// Whether a toolbar panel is shown.
    pub fn panel_shown(&self, child: ac2_ui::ToolbarChild) -> bool {
        self.hud.panel_shown(child)
    }

    /// Whether the gameplay HUD is shown (`gmGamePlayUI::m_shown`, the `0x4100000C` key).
    pub fn gameplay_shown(&self) -> bool {
        self.hud.gameplay_shown()
    }

    /// Whether a gameplay child with a key is shown: the toolbar's bar, the status bar, the radar.
    pub fn child_visible(&self, child: ac2_ui::GameplayChild) -> bool {
        self.hud.child_visible(child)
    }

    /// `RecvNotice_DisplayStringInfo`: a `StringInfo` line for the chat windows, rendered off the
    /// frame thread.
    pub fn chat_display_info(&mut self, kind: u32, info: &ac2_net::protocol::StringInfo) {
        self.hud.chat_display_info(kind, info);
    }

    /// A line for the chat windows, with its display type (`RecvNotice_DisplayStringInfo`).
    pub fn chat_display(&mut self, kind: u32, text: &str) {
        self.hud.chat_display(kind, text);
    }

    /// `gmGamePlayUI::SetErrorMsg`: the line reaches the chat as an error (the error text type,
    /// through `chat_display`, as `ECM_Admin::SendNotice_DisplayAdminStringInfo` does).
    pub fn set_error_msg(&mut self, text: &str) {
        self.chat_display(ac2_ui::ERROR_TEXT_TYPE, text);
    }

    /// Whether the HUD's active element keeps the arrow keys from the camera
    /// (`gmCameraUI::SendKey`).
    pub fn camera_keys_blocked(&self) -> bool {
        self.hud.camera_keys_blocked()
    }

    /// Whether the HUD takes typing (`CInputManager::SetTextMode`).
    pub fn text_mode(&self) -> bool {
        self.hud.text_mode()
    }

    /// Whether a read-only HUD text holds the copy focus (Ctrl+A / Ctrl+C reach it).
    pub fn text_copy_focus(&self) -> bool {
        self.hud.text_copy_focus()
    }

    /// The pointer as the HUD last saw it (`ICIDM` mouse position), in window pixels.
    pub fn pointer(&self) -> (i32, i32) {
        self.hud.pointer()
    }

    /// How many HUD surfaces are shown.
    pub fn hud_shown_layers(&self) -> usize {
        self.hud.shown_layers()
    }

    /// How many HUD surfaces are drawn.
    pub fn hud_layers(&self) -> usize {
        self.hud.layers()
    }
}
