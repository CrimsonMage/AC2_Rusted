//! The HUD frame's passes that read the player: the options reload (`RecvNotice_ReloadOptions`),
//! the map (`gm3DTextUI`, `gmRadarUI::Update`, `gmNavigationUI::UpdateMacroMap`), the first frame
//! in the world (`!player_loaded`) and the pick (`gmPaperDollUI`, `gmSelectionUI`, `gmFriendsUI`).
//! The panels read the player through [`HudSource`]; what only the client knows (the saved
//! options, the window places, the object table's race, the GUN events) comes through
//! [`HudHost`]; the frame's own values come as plain [`FrameInputs`]. The requests the passes
//! raise return as [`HudRequest`]s, in order, for the client to map to its commands.

use ac2_net::protocol::StringInfo;
use ac2_script::host::ScriptNotice;
use ac2_weenie::{GameplayNotice, PlayerVitals, SessionEvent};
use ac2_world::GameDateTime;

use crate::element::UiTree;
use crate::friends_panel::{friend_event, FriendEvent};
use crate::gameplay_story::StoryEvent;
use crate::hud_style::HudStyle;
use crate::target_select::frame_selection_calls;
use crate::token::tokenize;
use crate::{
    book_pass, item_dialog, BackpackUi, BookRequest, ChatSettings, FellowshipSource, Hud,
    HudRequest, HudSource, InventorySource, Items3dUi, MapMarker, MessageInputs, MoneyBagUi,
    NameLook, OptionsHost, OptionsSettings, PaperDollUi, SavedWindow, ShortcutCalls, StatusBar,
    TickInputs, UiMessage, UiStrings, WindowPosition, GLOBAL_KEY_PRESS, PROFILE_HIDE_PET_UI,
    PROFILE_LATENCY_INDICATOR_GRAPH, SKIP_STORY_MESSAGE,
};

/// What the client answers the frame's passes with: its options and saved places, the player's
/// race, the GUN friend events and the worn items. Reads take `&self` (the client's session is
/// behind its own cell); the saves and the friend events are the client's to keep.
pub trait HudHost {
    /// `gmRadarUI`'s coordinates option (`CM_Character`'s `ShowRadarCoords`).
    fn show_radar_coords(&self) -> bool;
    /// `gm3DTextUI`'s names option (`ShowObjectNames`).
    fn show_object_names(&self) -> bool;
    /// The player's chat settings, as the chat windows and the chat options page take them.
    fn chat_settings(&self) -> Option<ChatSettings>;
    /// The options profile as the options page reads it (`gmCWInterface::Get*`).
    fn options_settings(&self) -> Option<OptionsSettings>;
    /// The server's saved window locations (`gmGamePlayUI::RecvNotice_ReloadOptions`).
    fn saved_window_locations(&self) -> Result<Vec<SavedWindow>, String>;
    /// The player's own window places, kept on this machine for the character and the HUD's
    /// style.
    fn local_window_positions(
        &self,
        player: u64,
        style: HudStyle,
    ) -> Result<Vec<WindowPosition>, String>;
    /// Saves the player's window places on this machine.
    fn save_local_window_positions(
        &self,
        player: u64,
        style: HudStyle,
        positions: &[WindowPosition],
    );
    /// The player's race (int 290) from the object table.
    fn player_race(&self, player: u64) -> Option<i32>;
    /// The GUN friend events taken since the last call (`FriendStart`, `FriendStatus`).
    fn take_friend_events(&self) -> Vec<FriendEvent>;
    /// The worn items as `(location, item)`.
    fn worn_items(&self) -> Vec<(u32, u64)>;
    /// A tooltip for a map note (`gmNavigationUI::AddMarker`), rendered from the string tables.
    fn marker_tooltip(&self, message: &StringInfo, cell: u32) -> String;
    /// The shop's purchase limit for an item (`max_purchase_quantity`, 0 when unknown).
    fn max_purchase_quantity(&self, item: u64) -> u32;
    /// The consignment charge the script returns (`consignment_charge`, 0 when unknown).
    fn consignment_charge(&self) -> f32;
    /// `gmStoryUI::ListenToMessage` 0xffff0003: the skip key reaches the story slot.
    fn story_key(&self, data: u32);
    /// Whether the story background fades by its media (the slot then waits for its end).
    fn set_story_fades_on_hud(&self, fades: bool);
    /// The background's fade is done (`token_play`): the story starts or moves on.
    fn story_fade_done(&self);
    /// The story slot's drawn events since the last call (`Show`, `Hide`, `StopAudio`).
    fn take_story_events(&self) -> Vec<StoryEvent>;
}

/// The player and the client's sinks for the frame's option and shortcut calls (`gmClientUI`'s
/// preferences and applies, `gmShortcutUI`'s use and set), as the panels take them.
pub struct FrameHosts<'a, P: ?Sized> {
    pub player: &'a P,
    pub options: &'a mut dyn OptionsHost,
    pub shortcuts: &'a mut dyn ShortcutCalls,
}

/// The frame's plain inputs to the passes: what the frame says about the player, the selection,
/// the map and the events, with no client type in it.
pub struct FrameInputs<'a> {
    /// `Timer::cur_time` (server time).
    pub server_time: f64,
    /// The player's id, when its body exists.
    pub player: Option<u64>,
    /// The player's cell, when its body exists.
    pub player_cell: Option<u32>,
    /// `CPhysicsObj::IsInWorld` for the player: the frame's passes run only in the world.
    pub in_world: bool,
    /// `CWPhysicsObject::IsMoving` for the player (`ac2_physics::is_moving_velocity`).
    pub player_moving: bool,
    /// The selected object's id (`gmSelectionUI`), for the fellowship and pet panels.
    pub selection: Option<u64>,
    pub player_vitals: &'a PlayerVitals,
    /// The player's money, as the money bag shows it.
    pub money: u32,
    /// The map notes, in arrival order.
    pub markers: &'a [MapMarker],
    /// The game's calendar at this frame.
    pub date_time: Option<GameDateTime>,
    /// `gm3DTextUI::AddName`'s looks: taken only when the names are shown or toggled.
    pub name_looks: &'a dyn Fn() -> Vec<NameLook>,
    /// The friends page's target (another player's name, when it is selected).
    pub friend_target: Option<&'a str>,
    /// The session's events this frame.
    pub events: &'a [SessionEvent],
    /// The script's notices this frame (the `CM_*` notice names).
    pub script_notices: &'a [ScriptNotice],
    /// The paper doll's pick from the scene as last drawn: `Some(mesh type)` once the pick is
    /// answered (`None` for no part).
    pub doll_pick: Option<Option<u32>>,
    /// The object under the pointer for `gm3DItemsUI`'s target search (`Some` once the search the
    /// last frame started is answered; `None` for no object inside).
    pub world_pick: Option<Option<u64>>,
}

/// The reload pass's outputs for the passes after it.
#[derive(Clone, Copy, Default)]
pub struct Reloads {
    /// The radar's options reloaded this frame (the shortcut bar takes it too).
    pub radar: bool,
    pub show_radar_coords: Option<bool>,
    pub show_names: Option<bool>,
}

impl Hud {
    /// One frame of the HUD's frameworks that ask the player, in the order they get it: the
    /// popup and display backlogs, then (in the world, with the player and the HUD) the reload,
    /// the map, the first frame, the pick, the notices, the messages, the help pages and the
    /// ticks. Returns the requests the panels raised, in order, for the client to map.
    pub fn frame<P: HudSource + HudHost>(
        &mut self,
        tree: Option<&mut UiTree>,
        dirty: &mut bool,
        hosts: Option<FrameHosts<'_, P>>,
        input: &FrameInputs<'_>,
    ) -> Vec<HudRequest> {
        let mut messages = std::mem::take(&mut self.player_messages);
        // `gmCWInterface::PopupYesNoDialog` raises its question at once, whether or not the HUD
        // is up yet: the dialogs are the game's own, always there in retail. A question that
        // comes before the HUD's dialogs waits in the backlog and is asked when they are.
        self.popup_backlog
            .extend(crate::popup_questions(input.events));
        // `gmGamePlayUI::RecvNotice_DisplayMessage` is registered with the HUD's children
        // (`gmGamePlayUI::SetupChildren`), so a box raised in the world waits for the HUD; before
        // the world there is no handler and the notice is gone, as in retail.
        if input.in_world {
            self.display_backlog
                .extend(crate::display_notices(input.events));
        }
        let (Some(tree), Some(hosts)) = (tree, hosts) else {
            return Vec::new();
        };
        let FrameHosts {
            player,
            options,
            shortcuts,
        } = hosts;
        let mut requests = Vec::new();
        // `gmStoryUI`: the skip key, the background's fade, then the slot's events.
        self.story_pass(tree, player, &messages, &mut requests);
        if !input.in_world {
            return requests;
        }
        // `gm3DItemsUI`: a drop's target search finishes with `world_pick`, then the messages;
        // the drops' stack dialogs open here and the rest are requests.
        if let Some(items) = self.items_3d.as_mut() {
            for request in items.frame_pass(tree, player, input.world_pick, &messages) {
                if let Some(request) = item_dialog(request, tree, self.stack.as_mut(), player) {
                    requests.push(HudRequest::Item(request));
                }
            }
        }
        // `gmCraftUI`: the craft notices and messages, then its requests (`CraftUi::frame_pass`).
        if let Some(craft) = self.craft.as_mut() {
            let commands = craft.frame_pass(tree, player, input.events, &messages);
            requests.extend(commands.into_iter().map(HudRequest::Craft));
        }
        // `gmBookUI`: the use notices show the book, then its messages turn the pages. Its help
        // button opens the help topic; the copy button sends the text to the clipboard inside
        // `book_pass`.
        if let Some(book) = self.book.as_mut() {
            for request in book_pass(book, tree, player, input.events, &messages) {
                if let BookRequest::OpenHelp = request {
                    requests.push(HudRequest::OpenHelp(tokenize("ID_Help_Book")));
                }
            }
        }
        // `gmIngredientUI`: the recipe's notices and messages, then its requests.
        if let Some(ingredient) = self.ingredient.as_mut() {
            let commands =
                ingredient.frame_pass(tree, player, input.events, &messages, input.server_time);
            requests.extend(commands.into_iter().map(HudRequest::Ingredient));
        }
        let (reload, reloads) = self.reload_pass(tree, dirty, player, input);
        requests.extend(reload);
        self.map_pass(tree, player, input, reloads);
        self.init_pass(tree, player, input);
        requests.extend(self.pick_pass(tree, player, input));
        // `notice_pass`: the frame's notices, then the shortcut bar's checks.
        let friends = crate::notice_pass(
            self,
            tree,
            dirty,
            player,
            input.events,
            input.server_time,
            reloads.radar,
        );
        requests.extend(friends.into_iter().map(HudRequest::Friends));
        // `message_pass`: the quests' global loop, then the messages to the panels.
        let mut most = |item| player.max_purchase_quantity(item);
        let mut charge = || player.consignment_charge();
        let message_inputs = MessageInputs {
            server_time: input.server_time,
            selection: input.selection,
            friend_target: input.friend_target,
            name_looks: input.name_looks,
            max_purchase: &mut most,
            consignment_charge: &mut charge,
            options_host: &mut *options,
        };
        self.tracker_enter(tree, dirty, &mut messages, &mut requests);
        requests.extend(crate::message_pass(
            self,
            tree,
            message_inputs,
            player,
            &messages,
        ));
        // `gmHelpUI`: the help frameworks' messages.
        if let Some(help) = self.help.as_mut() {
            if crate::help_pass(help, tree, self.help_pages.as_ref(), &messages) {
                *dirty = true;
            }
        }
        self.addition_pass(tree, dirty, &messages, &mut requests);
        // `gmFriendsUI`'s global loop: away from the keyboard ends when the player moves.
        if let Some(friends) = self.friends.as_mut() {
            friends.global_loop(tree, input.player_moving);
        }
        // `gmSecureTradeUI`, `gmPetUI`, `gmShopUI`, `gmSkillTreeUI`, `gmShortcutUI`'s ticks.
        let tick_inputs = TickInputs {
            server_time: input.server_time,
            selection: input.selection,
            options_host: &mut *options,
            shortcut_calls: &mut *shortcuts,
        };
        requests.extend(crate::tick_pass(
            self,
            tree,
            tick_inputs,
            player,
            input.events,
        ));
        // The fellows' map notes the notices and the reload asked for.
        requests.extend(
            self.fellow_markers
                .take_requests()
                .into_iter()
                .map(HudRequest::Marker),
        );
        if tree.take_changed() {
            *dirty = true;
        }
        requests
    }

    /// `gmStoryUI` on the frame: the skip key first, then the background's fade (its `token_play`
    /// starts the story), then the events the slot made (the key's own teardown included), so a
    /// story skipped this frame hides this frame. The gameplay panels leave at a story's begin
    /// and return at its hide ([`HudRequest::StoryGameplay`]).
    fn story_pass<P: HudHost>(
        &mut self,
        tree: &mut UiTree,
        host: &P,
        messages: &[UiMessage],
        requests: &mut Vec<HudRequest>,
    ) {
        for message in messages {
            if message.message == GLOBAL_KEY_PRESS && message.data as u32 == SKIP_STORY_MESSAGE {
                host.story_key(SKIP_STORY_MESSAGE);
            }
        }
        let Some(story) = self.story.as_mut() else {
            return;
        };
        host.set_story_fades_on_hud(story.fades_by_media());
        if messages.iter().any(|message| story.is_fade_done(message)) {
            host.story_fade_done();
        }
        let events = host.take_story_events();
        let strings = self.strings.clone();
        for shown in story.frame_events(tree, events, strings.as_deref()) {
            requests.push(HudRequest::StoryGameplay(shown));
        }
    }

    /// The options and saved layout that reload this frame (`gmChatOptionsUI`, `gmGamePlayUI`,
    /// `gmCharacterUI`, `gmNetStatusUI`, `gm3DTextUI`, `gmRadarUI` `RecvNotice_ReloadOptions`).
    /// Returns the requests the panels raised and the reload's flags for the passes after it.
    pub fn reload_pass<P: HudHost>(
        &mut self,
        tree: &mut UiTree,
        dirty: &mut bool,
        host: &P,
        input: &FrameInputs<'_>,
    ) -> (Vec<HudRequest>, Reloads) {
        let mut requests = Vec::new();
        // `gmGamePlayUI::RecvNotice_SaveLocationsChanged`: a window moved since the last frame.
        // The modern style's windows are kept on this machine only: their places are not
        // retail's, so they are neither uploaded nor taken from the server (`AGENTS.md`).
        if tree.take_locations_changed() && self.style == HudStyle::Classic {
            requests.push(HudRequest::SaveWindowLocations(tree.inq_locations()));
        }
        // The window exception (`AGENTS.md`): a window moved and its drag ended, so every
        // window's place is kept on this machine for the character.
        if tree.take_positions_changed() {
            if let Some(player) = input.player {
                host.save_local_window_positions(player, self.style, &tree.window_positions());
            }
        }
        // `gmChatOptionsUI`'s Apply sent the reload notice; its values refresh on the options'
        // reload (`RecvNotice_ReloadOptions`), as the page's Refresh does.
        let applied = self
            .chat_options
            .as_mut()
            .is_some_and(|page| page.take_reload());
        let reload_chat = applied || !self.player_loaded || notice(input, CHAT_RELOAD_NOTICES);
        if reload_chat {
            if let Some(settings) = host.chat_settings() {
                if let Some(chat) = self.chat.as_mut() {
                    chat.reload_chat_settings(tree, &settings);
                }
                let reload_page = !self.player_loaded
                    || input
                        .script_notices
                        .iter()
                        .any(|n| n.name == "CM_Character::SendNotice_ReloadOptions");
                if let (true, Some(page)) = (reload_page, self.chat_options.as_mut()) {
                    page.refresh(tree, &settings);
                }
            }
        }
        // The chat windows' opacity sliders released since the last frame.
        if let Some(chat) = self.chat.as_mut() {
            for (window, opacity) in chat.take_opacity_requests() {
                requests.push(HudRequest::SetWindowOpacity { window, opacity });
            }
        }
        // `gmRadarUI::ReloadOptions` (at login, `Init`, and on `RecvNotice_ReloadOptions`).
        let reload_radar = !self.player_loaded
            || input
                .script_notices
                .iter()
                .any(|n| n.name == "CM_Character::SendNotice_ReloadOptions");
        let show_radar_coords = reload_radar.then(|| host.show_radar_coords());
        // `gmGamePlayUI::RecvNotice_ReloadOptions` (and the refresh at login, `ListenToMessage`
        // 0xffff0007): the player's saved window locations apply to the windows.
        if reload_radar {
            if let Some(saved) = (self.style == HudStyle::Classic)
                .then(|| host.saved_window_locations().ok())
                .flatten()
            {
                tree.apply_locations(&saved);
                if let Some(radar) = self.radar.as_ref() {
                    radar.location_applied(tree);
                }
            }
            // The player's own places, per character, override the server's (`AGENTS.md`).
            if let Some(player) = input.player {
                match host.local_window_positions(player, self.style) {
                    Ok(local) => tree.set_window_positions(&local),
                    Err(error) => eprintln!("ac2: window positions: {error}"),
                }
            }
        }
        // `gmCharacterUI::RecvNotice_ReloadOptions`: the page's values as the profile has them.
        let reload_page = reload_radar
            || input
                .events
                .iter()
                .any(|e| matches!(e, SessionEvent::Notice(GameplayNotice::ReloadOptions)));
        if let (true, Some(page)) = (reload_page, self.character.as_mut()) {
            if let Some(settings) = host.options_settings() {
                page.refresh(tree, &settings);
            }
        }
        if let Some(page) = self.character.as_mut() {
            for request in page.take_requests() {
                requests.push(HudRequest::Character(request));
            }
        }
        // `gmNetStatusUI::RecvNotice_ReloadOptions`: the latency graph's option.
        if let (true, Some(net)) = (reload_radar, self.net.as_mut()) {
            if let Some(settings) = host.options_settings() {
                net.set_graph_enabled(settings.bits & PROFILE_LATENCY_INDICATOR_GRAPH != 0);
                *dirty = true;
            }
        }
        // `FellowshipHelper::RecvNotice_ReloadGameplayOptions`: the fellows' map notes as the
        // options now say.
        if reload_radar {
            let hidden = host.options_settings().map(|settings| {
                settings.bits & crate::character_options::PROFILE_HIDE_FELLOWS_ON_MAP != 0
            });
            let members = crate::hud_notices::fellowship_members(self);
            let player = input.player.unwrap_or(0);
            self.fellow_markers.reload(hidden, player, &members);
        }
        // `gmPetUI::RecvNotice_ReloadGameplayOptions`: `UpdateState` with `GetShowPetUI`.
        if let (true, Some(pets)) = (reload_radar, self.pets.as_mut()) {
            if let Some(settings) = host.options_settings() {
                pets.set_show_pet_ui(tree, settings.bits & PROFILE_HIDE_PET_UI == 0);
                *dirty = true;
            }
        }
        // `gm3DTextUI::RecvNotice_ReloadOptions`: the names as the options say.
        let show_names = reload_radar.then(|| host.show_object_names());
        self.player_race = input.player.and_then(|id| host.player_race(id));
        let reloads = Reloads {
            radar: reload_radar,
            show_radar_coords,
            show_names,
        };
        (requests, reloads)
    }

    /// The name plates, the radar's options and position, and the macro map's date and time
    /// (`gm3DTextUI`, `gmRadarUI::Update`, `gmNavigationUI::UpdateMacroMap`).
    pub fn map_pass<P: HudSource + HudHost>(
        &mut self,
        tree: &mut UiTree,
        player: &P,
        input: &FrameInputs<'_>,
        reloads: Reloads,
    ) {
        let Reloads {
            show_names,
            show_radar_coords,
            ..
        } = reloads;
        if let Some(plates) = self.names_mut() {
            let looks = (input.name_looks)();
            match show_names {
                Some(show) => plates.set_showing(tree, show, &looks),
                None => plates.sync(tree, &looks),
            }
        }
        if let (Some(show), Some(radar)) = (show_radar_coords, self.radar.as_mut()) {
            radar.reload_options(tree, player, show);
        }
        // `gmRadarUI::Update`'s `Position::ToRealMapCoords_Info`, again when the cell changes.
        let cell = input.player_cell;
        if cell.is_some() && cell != self.radar_cell {
            self.radar_cell = cell;
            let string = crate::map_coords_string(cell.unwrap_or(0));
            self.radar_coordinates = Some(UiStrings::render_string(player, &string));
        }
        // `gmNavigationUI::AddMarker`'s tooltips, rendered with the frame's markers.
        self.navigation_tooltips = input
            .markers
            .iter()
            .map(|m| player.marker_tooltip(&m.message, m.cell))
            .collect();
        // `gmNavigationUI::UpdateMacroMap`'s date and time: rendered again when the calendar moves.
        if let Some(date_time) = input.date_time {
            if self.navigation_time.as_ref().map(|(known, _, _)| *known) != Some(date_time) {
                let (date, time) = crate::game_date_time(date_time);
                self.navigation_time = Some((
                    date_time,
                    UiStrings::render_string(player, &date),
                    UiStrings::render_string(player, &time),
                ));
            }
        }
    }

    /// The first frame in the world (`!player_loaded`): the panels read the player once.
    pub fn init_pass<P: HudSource + HudHost>(
        &mut self,
        tree: &mut UiTree,
        player: &P,
        input: &FrameInputs<'_>,
    ) {
        if self.player_loaded {
            return;
        }
        self.player_loaded = true;
        // `gmSecureTradeUI::gmSecureTradeUI` reads the player.
        if let (Some(panel), Some(id)) = (self.secure_trade.as_mut(), input.player) {
            panel.set_player(id);
        }
        if let Some(skills) = self.skills.as_mut() {
            skills.update_skill_panel(tree, player);
        }
        // `gmJournalQuestsUI` registers for the player's quest notices when it is made, before the
        // login message's quest list (`WM_Player` 0x27) raises the list notice. The HUD here is
        // built on workers and can come after that notice, so the journal takes the list the
        // player's script holds now, as that notice's case 2 would (`InqQuestInfoList`).
        if let Some(quests) = self.quests.as_mut() {
            let list = GameplayNotice::UpdateQuests {
                code: crate::QUEST_CODE_LIST,
                quest: Box::default(),
            };
            quests.recv_gameplay_notice(tree, player, &list);
        }
        if let Some(page) = self.allegiance.as_mut() {
            page.set_player(tree, FellowshipSource::player(player));
        }
        // `gmVitalsUI`, `gmEffectsUI` (`Init`) and `gmVitaeStatusUI` read the player.
        StatusBar {
            vitals: &mut self.vitals,
            effects: &mut self.effects,
            vitae: &mut self.vitae,
            net: &mut self.net,
            pk: &mut self.pk,
        }
        .player_loaded(tree, player, input.player_vitals, input.server_time);
        // `gmInventoryUI::SetupChildren`'s backpack over the player.
        if let (Some(layout), Some(id)) = (self.toolbar_layout.as_ref(), input.player) {
            self.item_selection = InventorySource::selection(player);
            self.backpack = Some(BackpackUi::create(tree, layout, player, id));
            self.items_3d = Some(Items3dUi::create(tree, id));
            // `gmMoneyBagUI::gmMoneyBagUI` and its amount (`RecvNotice_UpdateMoney`).
            let mut bag = MoneyBagUi::create(tree);
            bag.set_money(tree, player, input.money);
            self.money = Some(bag);
            let equipped = player.worn_items();
            let doll = PaperDollUi::create(tree, layout, player, id, &equipped);
            doll.follow_panel(tree);
            self.paper_doll = Some(doll);
        }
    }

    /// The paper doll's pick, the target selection's notices and the GUN friend events, ahead of
    /// the notices. Returns the requests in the order the panels raised them.
    pub fn pick_pass<P: HudSource + HudHost>(
        &mut self,
        tree: &mut UiTree,
        player: &P,
        input: &FrameInputs<'_>,
    ) -> Vec<HudRequest> {
        let mut requests = Vec::new();
        // `gmPaperDollUI::FinishPaperDollClick` with the scene's answer.
        if let (Some(doll), Some(mesh_type)) = (self.paper_doll.as_mut(), input.doll_pick) {
            if let Some(request) = doll.finish_pick(tree, mesh_type) {
                requests.push(HudRequest::Item(request));
            }
        }
        // `gmSelectionUI`: the selection keys, its notices and `RecvNotice_BeingDeleted`.
        let typing = tree.text_mode();
        requests.extend(
            frame_selection_calls(player, tree, input.events, typing)
                .into_iter()
                .map(HudRequest::Selection),
        );
        // `gmFriendsUI`: the GUN list and status the client took since the last frame, then the
        // add and remove buttons for the game selection (`UpdateButtonStates`, every frame).
        let friend_events = player.take_friend_events();
        if let Some(panel) = self.friends.as_mut() {
            for event in friend_events {
                friend_event(panel, tree, event);
            }
            panel.set_target(tree, input.friend_target);
        }
        requests
    }
}

/// The notices that reload the chat settings (`CM_Communication`, `CM_Character`).
const CHAT_RELOAD_NOTICES: &[&str] = &[
    "CM_Communication::SendNotice_ReloadChatSettings",
    "CM_Character::SendNotice_ReloadOptions",
];

/// Whether the frame's script notices include one of `names`.
fn notice(input: &FrameInputs<'_>, names: &[&str]) -> bool {
    input.script_notices.iter().any(|n| names.contains(&n.name))
}
