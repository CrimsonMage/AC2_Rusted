//! What the HUD's panels ask of the player, one request per panel request kind. The panels'
//! passes ([`crate::message_pass`] and its siblings) return these, in order, and the client maps
//! each to its gameplay command. Each variant carries the panel's own request type.

use crate::{
    CatalogRequest, CharacterRequest, ChatOptionRequest, ConsignmentRequest, CraftCommand,
    DialogRequest, ExaminationCall, ExternalContainerRequest, FriendsRequest, IngredientCommand,
    ItemRequest, NewSaleRequest, PetAction, RadarRequest, SavedWindow, SecureTradeRequest,
    SelectionCall, ShopRequest, SkillRequest, SocialCall,
};

/// One request a HUD panel raised in a pass.
pub enum HudRequest {
    /// `gmGamePlayUI::SaveWindowLocations`: the windows' places, after a window moved.
    SaveWindowLocations(Vec<SavedWindow>),
    /// `gmChatWindow`'s opacity slider released: the window's opacity is saved.
    SetWindowOpacity { window: u32, opacity: f32 },
    /// `gmCWInterface::Dismount`: the toolbar's dismount button.
    Dismount,
    /// `gmCWInterface::DisplaySaleReminders`: the consignments button.
    DisplaySaleReminders,
    /// `gmSelectionUI::Select`: a target key's or the selection's calls.
    Selection(SelectionCall),
    /// `Command::Select`: an object picked by a panel (a name plate, a list row, a slot).
    Select(u64),
    /// `gm3DTextUI`: the names key toggled the name plates.
    SetShowObjectNames(bool),
    /// `gmCharacterUI`: the options page's settings and upload.
    Character(CharacterRequest),
    /// `gmChatOptionsUI`: the chat options page's settings and upload.
    ChatOption(ChatOptionRequest),
    /// `gmPetUI`: the pet panel's selection, attacks and modes.
    Pet(PetAction),
    /// `gmFriendsUI`: the friends page's list and status requests.
    Friends(FriendsRequest),
    /// `gmRadarUI`: a blip selected, or the coordinates shown or hidden.
    Radar(RadarRequest),
    /// A map note added, removed or cleared (`FellowshipHelper`'s fellow markers).
    Marker(crate::MarkerRequest),
    /// The Tracker window: a pattern added or removed, a match picked or dismissed, a reload.
    Tracker(crate::tracker_ui::TrackerRequest),
    /// A press on a map coordinate in chat: the floating arrow points there (north, east).
    PointAt { north: f32, east: f32 },
    /// A press on read-only text (the chat log): the line and the column it landed on.
    TextPress(crate::text_edit::TextPress),
    /// A slot's request (`gmInventoryUI`, the money bag, the paper doll, the 3D items).
    Item(ItemRequest),
    /// `gmExternalContainerUI`'s close, loot-all and transmute-all (its item requests are
    /// [`HudRequest::Item`]).
    ExternalContainer(ExternalContainerRequest),
    /// `gmShopNewSaleUI`: the sale's buttons.
    NewSale(NewSaleRequest),
    /// `gmCraftUI`: the craft panel's examination, recipe, raise and selection requests.
    Craft(CraftCommand),
    /// `gmIngredientUI`: the ingredient panel's add and execute requests.
    Ingredient(IngredientCommand),
    /// `gmStoryUI`: the gameplay panels leave (`false`) at a story's begin and return (`true`) at
    /// its hide; the client holds the show state.
    StoryGameplay(bool),
    /// `gmShopCatalogUI`: the catalog's buttons and rows.
    Catalog(CatalogRequest),
    /// `gmShopConsignmentUI`: the consignment's requests the panel answers with the player
    /// (its new-sale dialog is opened in the pass).
    Consignment(ConsignmentRequest),
    /// The examination panel opens on the request already made (no examination call of its own).
    OpenExamination,
    /// `gmCWInterface::SetExaminationRequest` with the request, and the item it selects (0 for
    /// none); the examination panel opens.
    Examine { kind: u32, data: u32, item: u64 },
    /// `KeyStone::OpenHelp` for a help topic.
    OpenHelp(u32),
    /// `gmSecureTradeUI`'s calls on the player (`gmCWInterface::*Trade*`).
    SecureTrade(SecureTradeRequest),
    /// `gmShopUI`'s requests on the player (its examination is [`HudRequest::OpenExamination`]).
    Shop(ShopRequest),
    /// `gmSkillTreeUI`'s requests: an examination, a train, raise or untrain.
    Skill(SkillRequest),
    /// `gmExaminationUI`'s start and stop calls.
    Examination(ExaminationCall),
    /// The allegiance and fellowship pages' calls: a `CPlayer` call, a selection, or an assisted
    /// tell's name for the chat.
    Social(SocialCall),
    /// A dialog's answer the gameplay panel takes (`GameplayDialogs::frame_answers`).
    Dialog(DialogRequest),
    /// `gmGamePlayUI`'s logout: the game options go up, then the character logs off.
    LogOff,
    /// `CM_UI::SendNotice_Logoff` (`gmGamePlayUI::RecvNotice_Logoff`): the game options go to
    /// the server and the client closes (`Device::Done`).
    Quit,
}
