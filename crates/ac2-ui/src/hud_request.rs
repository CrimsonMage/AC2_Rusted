//! What the HUD's panels ask of the player, one request per panel request kind. The panels'
//! passes ([`crate::message_pass`] and its siblings) return these, in order, and the client maps
//! each to its gameplay command. Each variant carries the panel's own request type.

use crate::{
    CatalogRequest, CharacterRequest, ChatOptionRequest, ConsignmentRequest,
    ExternalContainerRequest, FriendsRequest, ItemRequest, NewSaleRequest, PetAction, RadarRequest,
    SelectionCall,
};

/// One request a HUD panel raised in a pass.
pub enum HudRequest {
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
    /// A slot's request (`gmInventoryUI`, the money bag, the paper doll, the 3D items).
    Item(ItemRequest),
    /// `gmExternalContainerUI`'s close, loot-all and transmute-all (its item requests are
    /// [`HudRequest::Item`]).
    ExternalContainer(ExternalContainerRequest),
    /// `gmShopNewSaleUI`: the sale's buttons.
    NewSale(NewSaleRequest),
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
}
