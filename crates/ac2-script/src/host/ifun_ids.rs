//! The retail IFun ids the script host, the UI and the client name (`ggmCWInterface::CallIFun_*`,
//! `gmCWInterface::*` IFuns). Pure ids: the glue that answers them lives with each subsystem.

/// `gmCWInterface::RequestAttackStateChange` (`CPlayer` 0x4F: start attacking the selection,
/// or stop).
pub const IFUN_REQUEST_ATTACK_STATE_CHANGE: u32 = 0x00B4_0008;
/// `gmCWInterface::ShouldAutoTarget` (`CPlayer` 0x7A: the auto-target option, and the
/// selection not attackable).
pub const IFUN_SHOULD_AUTO_TARGET: u32 = 0x00B4_0005;
/// `gmCWInterface::SetWhichShortcutSet` (`CPlayer` 0x5).
pub const IFUN_SET_WHICH_SHORTCUT_SET: u32 = 0x00B4_0071;
/// `gmCWInterface::SetShowRadarCoords` (`ggmCWInterface::CallIFun_SetShowRadarCoords`).
pub const IFUN_SET_SHOW_RADAR_COORDS: u32 = 0x00B4_007D;
/// `gmCWInterface::SetShowObjectNames` (`ggmCWInterface::CallIFun_SetShowObjectNames`).
pub const IFUN_SET_SHOW_OBJECT_NAMES: u32 = 0x00B4_00CE;
/// `gmCWInterface::Dismount` (`ggmCWInterface::CallIFun_Dismount`: no arguments; the toolbar's
/// dismount button).
pub const IFUN_DISMOUNT: u32 = 0x00B4_00E5;
/// `gmCWInterface::StopAttacking(target)` (`ggmCWInterface::CallIFun_StopAttacking`: the
/// `InstanceID`). `gmSelectionUI` calls it when the target is no longer selectable or is deleted.
pub const IFUN_STOP_ATTACKING: u32 = 0x00B4_0104;
/// `gmCWInterface::StopAttackingIfAutoMoving` (`ggmCWInterface::CallIFun_StopAttackingIfAutoMoving`:
/// the return slot and the player's instance). A turn key starting calls it through
/// `PlayerMovementModule::StopAttackingIfAutoMoving`.
pub const IFUN_STOP_ATTACKING_IF_AUTO_MOVING: u32 = 0x00B4_00C3;
/// `gmCWInterface::RequestPetAttack(target)` (`ggmCWInterface::CallIFun_RequestPetAttack`: the
/// pet's `InstanceID`).
pub const IFUN_REQUEST_PET_ATTACK: u32 = 0x00B4_0144;
/// `gmCWInterface::RequestPetBuff` (`CallIFun_RequestPetBuff`: no arguments).
pub const IFUN_REQUEST_PET_BUFF: u32 = 0x00B4_0145;
/// `gmCWInterface::RequestPetHeal` (`CallIFun_RequestPetHeal`: no arguments).
pub const IFUN_REQUEST_PET_HEAL: u32 = 0x00B4_0146;
/// `gmCWInterface::RequestPetReturn` (`CallIFun_RequestPetReturn`: no arguments).
pub const IFUN_REQUEST_PET_RETURN: u32 = 0x00B4_0147;
/// `gmCWInterface::HandleWeenieEmote(emoteID)` (`ggmCWInterface::CallIFun_HandleWeenieEmote`: the
/// emote's id, and the int answer). A behaviour that names a weenie emote runs only when it says yes.
pub const IFUN_HANDLE_WEENIE_EMOTE: u32 = 0x00B4_00AA;
/// `gmCWInterface::RequestPetMode(target, mode)` (`CallIFun_RequestPetMode`: the mode, then the
/// pet's `InstanceID`).
pub const IFUN_REQUEST_PET_MODE: u32 = 0x00B4_0148;
/// `gmCWInterface::RequestSetPetName(target, name)` (`CallIFun_RequestSetPetName`: the name, a
/// `PStringBase<unsigned short>`, then the pet's `InstanceID`).
pub const IFUN_REQUEST_SET_PET_NAME: u32 = 0x00B4_0149;

/// `gmCWInterface::AcceptTrade`'s IFun (`ggmCWInterface::CallIFun_AcceptTrade`: no
/// arguments; `WM_Trade` stub 0x45 sends 0x35 with the instance packed).
pub const IFUN_ACCEPT_TRADE: u32 = 0x00B4_00BA;

/// `gmCWInterface::RequestEnterStore` (`ggmCWInterface::CallIFun_RequestEnterStore`: the store).
pub const IFUN_REQUEST_ENTER_STORE: u32 = 0x00B4_0137;
/// `gmCWInterface::Result_FriendAdd` (`CallIFun_Result_FriendAdd`: the weenie result, the name).
/// The same id in admin 7148 and Legions 7160.
pub const IFUN_RESULT_FRIEND_ADD: u32 = 0x00B4_0040;
/// `gmCWInterface::GUNEvent_FriendStart` (`CallIFun_GUNEvent_FriendStart`: the friends list, an
/// `RList` of `TurbineUserStatus`). The same id in admin 7148 and Legions 7160.
pub const IFUN_GUN_EVENT_FRIEND_START: u32 = 0x00B4_0043;
/// `gmCWInterface::GUNEvent_FriendStatus` (`CallIFun_GUNEvent_FriendStatus`: one
/// `TurbineUserStatus`).
pub const IFUN_GUN_EVENT_FRIEND_STATUS: u32 = 0x00B4_0044;
/// `gmCWInterface::Result_FriendDelete` (`CallIFun_Result_FriendDelete`: the result, the name).
pub const IFUN_RESULT_FRIEND_DELETE: u32 = 0x00B4_0041;
/// `gmCWInterface::RequestEnterConsignment` (`CallIFun_RequestEnterConsignment`: no arguments).
pub const IFUN_REQUEST_ENTER_CONSIGNMENT: u32 = 0x00B4_0136;
/// `gmCWInterface::RequestLeaveConsignment` (`CallIFun_RequestLeaveConsignment`: no arguments).
pub const IFUN_REQUEST_LEAVE_CONSIGNMENT: u32 = 0x00B4_0135;
/// `gmCWInterface::GetConsignmentCharge` (`CallIFun_GetConsignmentCharge`: a float out value).
pub const IFUN_GET_CONSIGNMENT_CHARGE: u32 = 0x00B4_013B;
/// `gmCWInterface::RequestLeaveStore` (`CallIFun_RequestLeaveStore`: no arguments).
pub const IFUN_REQUEST_LEAVE_STORE: u32 = 0x00B4_0131;
/// `gmCWInterface::PurchaseItem` (`CallIFun_PurchaseItem`: the item, the quantity, the slot).
pub const IFUN_PURCHASE_ITEM: u32 = 0x00B4_012C;
/// `gmCWInterface::PurchaseConfirmed` (`CallIFun_PurchaseConfirmed`: the answer, an int).
pub const IFUN_PURCHASE_CONFIRMED: u32 = 0x00B4_0130;
/// `gmCWInterface::RequestSell` (`ggmCWInterface::CallIFun_RequestSell`, IFun 0xB40138 in admin
/// and Legions; the item (long), the quantity, the price).
pub const IFUN_REQUEST_SELL: u32 = 0x00B4_0138;
/// `gmCWInterface::GetMaxPurchaseQuantity` (`CallIFun_GetMaxPurchaseQuantity`: the item, and an
/// out quantity).
pub const IFUN_GET_MAX_PURCHASE_QUANTITY: u32 = 0x00B4_012D;
/// `gmCWInterface::RequestSales(index)` (`ggmCWInterface::CallIFun_RequestSales`: the page, an int).
pub const IFUN_REQUEST_SALES: u32 = 0x00B4_013E;
/// `gmCWInterface::RequestNextSales` (`CallIFun_RequestNextSales`, no arguments).
pub const IFUN_REQUEST_NEXT_SALES: u32 = 0x00B4_0132;
/// `gmCWInterface::RequestPrevSales` (`CallIFun_RequestPrevSales`, no arguments).
pub const IFUN_REQUEST_PREV_SALES: u32 = 0x00B4_0133;
/// `gmCWInterface::RequestLeaveCatalog` (`CallIFun_RequestLeaveCatalog`, no arguments).
pub const IFUN_REQUEST_LEAVE_CATALOG: u32 = 0x00B4_0134;
/// `gmCWInterface::RequestCollect` (`CallIFun_RequestCollect`: the target (ulong), whether the
/// profits are taken (int)).
pub const IFUN_REQUEST_COLLECT: u32 = 0x00B4_0139;
/// `gmCWInterface::RequestRestock` (`CallIFun_RequestRestock`: the item (long), the quantity, the
/// target (ulong)).
pub const IFUN_REQUEST_RESTOCK: u32 = 0x00B4_013A;
/// `gmCWInterface::DisplaySaleReminders` (`CallIFun_DisplaySaleReminders`, no arguments).
pub const IFUN_DISPLAY_SALE_REMINDERS: u32 = 0x00B4_013D;

/// The crafting IFuns (`ggmCWInterface::CallIFun_*`, the same ids in Legions 7160). The
/// queries' parameters are in the order the glue declares them, the out values after them.
pub const IFUN_CAN_CRAFT_SKILL_BE_RAISED: u32 = 0x00B4_011B;
pub const IFUN_CHECK_INGREDIENT: u32 = 0x00B4_00F4;
pub const IFUN_DOES_INGREDIENT_AUTO_POPULATE: u32 = 0x00B4_0119;
pub const IFUN_GET_AVAILABLE_CRAFT_EXPERIENCE: u32 = 0x00B4_0112;
pub const IFUN_INQ_CRAFT_SKILL_ADAPTOR: u32 = 0x00B4_00F9;
pub const IFUN_INQ_CRAFT_SKILL_CATEGORY_NAME: u32 = 0x00B4_0110;
pub const IFUN_INQ_CRAFT_SKILL_LEVEL: u32 = 0x00B4_0121;
pub const IFUN_INQ_CRAFT_SKILL_LIST: u32 = 0x00B4_00FA;
pub const IFUN_INQ_RECIPE_ADAPTOR: u32 = 0x00B4_009A;
pub const IFUN_INQ_RECIPE_COST: u32 = 0x00B4_0106;
pub const IFUN_INQ_RECIPE_DIFFICULTY: u32 = 0x00B4_011D;
pub const IFUN_INQ_RECIPE_INGREDIENTS: u32 = 0x00B4_0108;
pub const IFUN_INQ_RECIPE_LIST: u32 = 0x00B4_009B;
pub const IFUN_IS_CRAFT_SKILL_RAISEABLE: u32 = 0x00B4_0128;
/// `gmCWInterface::OpenRecipe` (`CPlayer` 0x1B7: raises `OpenIngredientPanel`, no request).
pub const IFUN_OPEN_RECIPE: u32 = 0x00B4_0118;
/// `gmCWInterface::RequestAddIngredient` (the stub sends `WM_Craft` 0x06).
pub const IFUN_REQUEST_ADD_INGREDIENT: u32 = 0x00B4_009D;
/// `gmCWInterface::RequestExecuteCraft` (the stub sends `WM_Craft` 0x12).
pub const IFUN_REQUEST_EXECUTE_CRAFT: u32 = 0x00B4_009E;
/// `gmCWInterface::RequestRaiseCraftSkill` (`CallIFun_RequestRaiseCraftSkill`).
pub const IFUN_REQUEST_RAISE_CRAFT_SKILL: u32 = 0x00B4_011C;

/// The skill requests (`ggmCWInterface::CallIFun_Request*Skill`, the same ids in Legions 7160):
/// `RequestTrainSkill` (the skill), `RequestUntrainSkill` (the skill), `RequestUntrainSkillCancel`
/// (none) and `RequestRaiseSkill` (the skill). Their stubs send `WM_Skill` 0x13, 0x12, 0xF and 0x10.
pub const IFUN_REQUEST_TRAIN_SKILL: u32 = 0x00B4_002B;
pub const IFUN_REQUEST_UNTRAIN_SKILL: u32 = 0x00B4_002C;
pub const IFUN_REQUEST_UNTRAIN_SKILL_CANCEL: u32 = 0x00B4_002D;
pub const IFUN_REQUEST_RAISE_SKILL: u32 = 0x00B4_002E;

/// `gmCWInterface::LootAllFromContainer` (`ggmCWInterface::CallIFun_LootAllFromContainer`, the
/// same id in Legions 7160): the container, an instance id (a long); its stub sends `WM_Inventory`
/// 0x68 with the take-all descriptor.
pub const IFUN_LOOT_ALL_FROM_CONTAINER: u32 = 0x00B4_00E8;

/// `gmCWInterface::SetSelection`'s IFun (`ggmCWInterface::CallIFun_SetSelection`: the
/// object, then `CPlayer` 0x6A).
pub const IFUN_SET_SELECTION: u32 = 0x00B4_0002;

/// `gmCWInterface::UseSelection` (`ggmCWInterface::CallIFun_UseSelection`, FunctionID 0xB40006, the
/// same in Legions 7160): no arguments; `CPlayer` 0x79 uses the selection (`CUsageSystem` 0x9).
pub const IFUN_USE_SELECTION: u32 = 0x00B4_0006;

// The options page's setters (`ggmCWInterface::CallIFun_Set*`, the same ids in Legions 7160).
pub const IFUN_SET_AUTO_TARGET: u32 = 0x00B4_0085;
pub const IFUN_SET_ACCEPT_ALLEGIANCE: u32 = 0x00B4_0087;
pub const IFUN_SET_ACCEPT_FELLOWSHIP: u32 = 0x00B4_0089;
pub const IFUN_SET_ACCEPT_TRADE: u32 = 0x00B4_008B;
pub const IFUN_SET_SHOW_DAMAGE_TEXT: u32 = 0x00B4_0075;
pub const IFUN_SET_SHOW_DAMAGE_TEXT_OTHER: u32 = 0x00B4_0077;
pub const IFUN_SET_SHOW_SHORTCUT_NUMBERS: u32 = 0x00B4_0073;
pub const IFUN_SET_SHOW_PK_DAMAGE: u32 = 0x00B4_00FD;
pub const IFUN_SET_SHORTCUT_LOCKED: u32 = 0x00B4_00FB;
pub const IFUN_SET_SHORTCUT_INSTA_SCROLL: u32 = 0x00B4_010B;
pub const IFUN_SET_BYPASS_GOLD_MELTING: u32 = 0x00B4_00D6;
pub const IFUN_SET_LATENCY_INDICATOR_GRAPH: u32 = 0x00B4_010A;
pub const IFUN_SET_SHOW_FELLOWS_ON_MAP: u32 = 0x00B4_0140;
pub const IFUN_SET_SHOW_PET_UI: u32 = 0x00B4_014B;
pub const IFUN_SET_RADAR_MASK: u32 = 0x00B4_007F;
pub const IFUN_SET_DAMAGE_TEXT_RANGE_OTHER: u32 = 0x00B4_0079;
pub const IFUN_SET_SHORTCUT_HEIGHT: u32 = 0x00B4_0142;
/// `ggmCWInterface::CallIFun_SetUISaveLocations(UISaveLocations)` (Legions 0x00463CC0): the
/// saved locations go into the player's profile.
pub const IFUN_SET_UI_SAVE_LOCATIONS: u32 = 0x00B4_007B;
/// `ggmCWInterface::CallIFun_InqUISaveLocations(UISaveLocations out)` (Legions 0x0045F7E0).
pub const IFUN_INQ_UI_SAVE_LOCATIONS: u32 = 0x00B4_007C;
/// `ggmCWInterface::CallIFun_SetWindowOpacity(windowID, opacity)` (Legions 0x00463E50): the
/// opacity is a float, passed as its bits in the word slot (tag 4, as the glue pushes it).
pub const IFUN_SET_WINDOW_OPACITY: u32 = 0x00B4_00DF;
/// `ggmCWInterface::CallIFun_UploadUISettings` (Legions 0x00464870): the profile to the server
/// (`WM_Player` 0x32, `UploadUISettings`).
pub const IFUN_UPLOAD_UI_SETTINGS: u32 = 0x00B4_00DB;
/// `gmCWInterface::UploadGameOptions` (`ggmCWInterface::CallIFun_UploadGameOptions`: the script
/// sends the profile, `WM_Player` 0x1D).
pub const IFUN_UPLOAD_GAME_OPTIONS: u32 = 0x00B4_0093;

// The chat options page's setters (`ggmCWInterface::CallIFun_SetChat*`, Legions 7160): the text
// type or window first, then the value.
/// `CallIFun_SetChatWindowFilter(window, bitfield)`.
pub const IFUN_SET_CHAT_WINDOW_FILTER: u32 = 0x00B4_0083;
/// `CallIFun_SetChatFontSize(textType, fontSize)`.
pub const IFUN_SET_CHAT_FONT_SIZE: u32 = 0x00B4_00D3;
/// `CallIFun_SetChatFontColor(textType, fontColor)`.
pub const IFUN_SET_CHAT_FONT_COLOR: u32 = 0x00B4_00D4;
/// `CallIFun_UploadChatOptions` (no arguments; the profile to the server).
pub const IFUN_UPLOAD_CHAT_OPTIONS: u32 = 0x00B4_00D5;
/// `gmCWInterface::DoEmote(emoteName)` (`ggmCWInterface::CallIFun_DoEmote`: the emote's
/// `StringInfo`, the emote table's string). Its one caller is the friends status drop-down.
pub const IFUN_DO_EMOTE: u32 = 0x00B4_00EC;
/// `CallIFun_SetChatPopupFlag(textType, flag)`.
pub const IFUN_SET_CHAT_POPUP_FLAG: u32 = 0x00B4_00DD;
/// The window's text type (`FUN_00458e50`'s IFun in Legions 7160): the window, then the type.
pub const IFUN_SET_CHAT_WINDOW_TEXT_TYPE: u32 = 0x00B4_0143;
