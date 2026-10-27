//! The weenies' scripts on the script VM (`ac2_script::host::ScriptHost`): each object the
//! session reports gets its script process (`CWDistributor::Construct`, from its WState, read
//! into the cache on a loader thread first), the server's script calls are dispatched to it
//! (`CWDistributor::DispatchInterpMsg`), and what the scripts produce goes out: their server
//! calls as `InterpSEvent`s (`gCWInterface::SendServerEvt`), their notices among the frame's
//! events (`CM_*::SendNotice_*`).
//!
//! The session's native stand-ins (`PlayerScript`, the inventory, skills) still answer the
//! calls they know; a call is the VM's once it is handed over ([`Client::attach_scripts`]'s
//! `functions`), or when no stand-in knows it. For a call a stand-in still answers, the VM runs
//! too (its state follows the server's) but what it produces is set aside, so nothing is sent
//! or shown twice. UI commands still go to the stand-ins.

use std::collections::{BTreeMap, BTreeSet};

use ac2_net::protocol::{unpack_values, PackValue};
use ac2_script::host::{ChatServices, IFunArg, ScriptHost, ScriptNotice};
use ac2_weenie::SessionEvent;

use crate::client::Client;
use crate::loader::Request;

/// The server calls and UI IFuns the script VM answers as the stand-ins did (verified against
/// them, `scripts_7160`), and so takes over: `WM_Communication` 0x11, 0x23
/// (`DisplayStringInfo`) and 0x12 (`DisplayText`); `WM_Player` 0x27 (the login, `CPlayer`
/// 0x65: vitals, inventory refresh; the session's mirrors still follow it) and 0x2C
/// (`UpdateSelectionInfo`, `CPlayer` 0x34); `SetSelection` (`CPlayer` 0x6A),
/// `RequestAttackStateChange` (`CPlayer` 0x4F), `SetWhichShortcutSet` (`CPlayer` 0x5), `SetShowRadarCoords`, and
/// the fellowship and allegiance actions.
pub const SCRIPT_VM_FUNCTIONS: &[u32] = &[
    ac2_net::protocol::script_function::DISPLAY_STRING_INFO,
    ac2_net::protocol::script_function::DISPLAY_STRING_INFO_2,
    ac2_net::protocol::script_function::DISPLAY_TEXT,
    ac2_net::protocol::script_function::INIT_PLAYER,
    ac2_net::protocol::script_function::UPDATE_SELECTION_INFO,
    IFUN_SET_SELECTION,
    // `gmCWInterface::UseSelection` (`CPlayer` 0x79 -> `CUsageSystem` 0x9) and the server's
    // outcome and failure (`WM_Usage` 0x21 -> `CPlayer` 0x7D, 0x23 -> `CPlayer` 0x67), whose
    // `CUsageSystem` 0xA / 0x2 name the error text (see [`crate::gameplay`]).
    IFUN_USE_SELECTION,
    ac2_net::protocol::script_function::USAGE_DONE,
    ac2_net::protocol::script_function::USAGE_ERROR,
    IFUN_REQUEST_ATTACK_STATE_CHANGE,
    IFUN_SET_WHICH_SHORTCUT_SET,
    IFUN_SET_SHOW_RADAR_COORDS,
    IFUN_SET_SHOW_OBJECT_NAMES,
    // `gmCWInterface::DoEmote` (the friends status's emote).
    IFUN_DO_EMOTE,
    // The combat IFuns the gameplay UI calls with no stand-in (`gmCWInterface::Dismount`,
    // `StopAttacking`).
    IFUN_DISMOUNT,
    IFUN_STOP_ATTACKING,
    IFUN_STOP_ATTACKING_IF_AUTO_MOVING,
    // The pet panel's requests (`gmCWInterface::RequestPet*`, `RequestSetPetName`).
    IFUN_REQUEST_PET_ATTACK,
    IFUN_REQUEST_PET_BUFF,
    IFUN_REQUEST_PET_HEAL,
    IFUN_REQUEST_PET_RETURN,
    IFUN_REQUEST_PET_MODE,
    IFUN_REQUEST_SET_PET_NAME,
    // The options page's setters and the upload (`gmCharacterUI::Apply`).
    IFUN_SET_AUTO_TARGET,
    IFUN_SET_ACCEPT_ALLEGIANCE,
    IFUN_SET_ACCEPT_FELLOWSHIP,
    IFUN_SET_ACCEPT_TRADE,
    IFUN_SET_SHOW_DAMAGE_TEXT,
    IFUN_SET_SHOW_DAMAGE_TEXT_OTHER,
    IFUN_SET_SHOW_SHORTCUT_NUMBERS,
    IFUN_SET_SHOW_PK_DAMAGE,
    IFUN_SET_SHORTCUT_LOCKED,
    IFUN_SET_SHORTCUT_INSTA_SCROLL,
    IFUN_SET_BYPASS_GOLD_MELTING,
    IFUN_SET_LATENCY_INDICATOR_GRAPH,
    IFUN_SET_SHOW_FELLOWS_ON_MAP,
    IFUN_SET_SHOW_PET_UI,
    IFUN_SET_RADAR_MASK,
    IFUN_SET_DAMAGE_TEXT_RANGE_OTHER,
    IFUN_SET_SHORTCUT_HEIGHT,
    IFUN_UPLOAD_GAME_OPTIONS,
    // The window locations (`gmCWInterface::SetUISaveLocations`, `InqUISaveLocations`,
    // `UploadUISettings`).
    IFUN_SET_UI_SAVE_LOCATIONS,
    IFUN_INQ_UI_SAVE_LOCATIONS,
    IFUN_UPLOAD_UI_SETTINGS,
    IFUN_SET_WINDOW_OPACITY,
    // The chat options page's setters and the upload (`gmChatOptionsUI::Apply`).
    IFUN_SET_CHAT_WINDOW_FILTER,
    IFUN_SET_CHAT_FONT_SIZE,
    IFUN_SET_CHAT_FONT_COLOR,
    IFUN_SET_CHAT_POPUP_FLAG,
    IFUN_SET_CHAT_WINDOW_TEXT_TYPE,
    IFUN_UPLOAD_CHAT_OPTIONS,
    // `CWInterface::OnStatChanged` (a quality update: vitals, the XP bar, the skill panel).
    ac2_weenie::IFUN_ON_STAT_CHANGED,
    // `CWInterface::HandleCollision` (a collision report: the impact a body recorded).
    ac2_weenie::IFUN_HANDLE_COLLISION,
    // `WM_Combat` 0x1B -> `CPlayer` 0x2 (`SendNotice_UpdateAttackState`) and 0x1A -> `CPlayer`
    // 0xE0 (a combat action's failure: `CCombatSystem` 0x11 makes the chat line).
    0x8204_001B,
    0x8204_001A,
    // The secure trade's `WM_Trade` forwarders (`CPlayer` -> `CM_Trade::SendNotice_Trade_*`).
    ac2_net::protocol::script_function::TRADE_REGISTERED,
    ac2_net::protocol::script_function::TRADE_OPENED,
    ac2_net::protocol::script_function::TRADE_CLOSED,
    ac2_net::protocol::script_function::TRADE_ACCEPTED,
    ac2_net::protocol::script_function::TRADE_DECLINED,
    ac2_net::protocol::script_function::TRADE_RESET,
    ac2_net::protocol::script_function::TRADE_ACCEPTANCE_CLEARED,
    ac2_net::protocol::script_function::TRADE_ACCEPTANCE_CLEARED_ALT,
    ac2_net::protocol::script_function::TRADE_OFFERED,
    ac2_net::protocol::script_function::TRADE_REVOKED,
    // `gmCWInterface::AcceptTrade` (its IFun packs the accepting instance for the stub).
    IFUN_ACCEPT_TRADE,
    // The store's `WM_Store` forwarders (`CPlayer` -> `CStoreSystem` -> `CM_Store::SendNotice_*`).
    ac2_net::protocol::script_function::STORE_VIEW,
    ac2_net::protocol::script_function::STORE_UPDATE,
    ac2_net::protocol::script_function::STORE_LEFT,
    ac2_net::protocol::script_function::STORE_UPDATE_SALE,
    // The craft answers (`WM_Craft` 0x16 -> `CPlayer` 0x30, 0x1C -> `CPlayer` 0xF7), the panel
    // update 0x18, and the registry: 0x19 the whole registry, 0x1B a recipe learned by DID, 0x27
    // and 0x34 skill records set, 0x1A and 0x37 known recipes' records replaced. `UpdateCost` has
    // no forwarder in the Legions wlib.
    ac2_net::protocol::script_function::CRAFT_ADD_INGREDIENT_DONE,
    ac2_net::protocol::script_function::CRAFT_EXECUTE_DONE,
    ac2_net::protocol::script_function::CRAFT_UPDATE_PANEL_18,
    ac2_net::protocol::script_function::CRAFT_SET_REGISTRY,
    ac2_net::protocol::script_function::CRAFT_LEARN_RECIPE,
    ac2_net::protocol::script_function::CRAFT_SET_SKILL,
    ac2_net::protocol::script_function::CRAFT_SET_SKILLS,
    ac2_net::protocol::script_function::CRAFT_UPDATE_RECIPE,
    ac2_net::protocol::script_function::CRAFT_UPDATE_RECIPES,
    // `gmCWInterface`'s store calls (their IFuns on the player's script).
    IFUN_REQUEST_ENTER_STORE,
    IFUN_REQUEST_LEAVE_STORE,
    // The consignment answers and calls (`gmCWInterface`'s consignment IFuns).
    ac2_net::protocol::script_function::STORE_VIEW_CATALOG,
    ac2_net::protocol::script_function::STORE_VIEW_CONSIGNMENT,
    ac2_net::protocol::script_function::STORE_UPDATE_CONSIGNMENT,
    IFUN_REQUEST_ENTER_CONSIGNMENT,
    // The friends' GUN results (`ggmCWInterface::CallIFun_Result_Friend*`, see [`crate::friends`]).
    IFUN_RESULT_FRIEND_ADD,
    IFUN_RESULT_FRIEND_DELETE,
    IFUN_GUN_EVENT_FRIEND_START,
    IFUN_GUN_EVENT_FRIEND_STATUS,
    IFUN_REQUEST_LEAVE_CONSIGNMENT,
    IFUN_GET_CONSIGNMENT_CHARGE,
    IFUN_PURCHASE_ITEM,
    IFUN_PURCHASE_CONFIRMED,
    IFUN_REQUEST_SELL,
    IFUN_GET_MAX_PURCHASE_QUANTITY,
    IFUN_REQUEST_SALES,
    IFUN_REQUEST_NEXT_SALES,
    IFUN_REQUEST_PREV_SALES,
    IFUN_REQUEST_LEAVE_CATALOG,
    IFUN_REQUEST_COLLECT,
    IFUN_REQUEST_RESTOCK,
    IFUN_DISPLAY_SALE_REMINDERS,
    // The skill requests (`ggmCWInterface::CallIFun_Request*Skill`, see [`crate::skills`]).
    IFUN_REQUEST_TRAIN_SKILL,
    IFUN_REQUEST_UNTRAIN_SKILL,
    IFUN_REQUEST_UNTRAIN_SKILL_CANCEL,
    IFUN_REQUEST_RAISE_SKILL,
    // The loot-all (`gmCWInterface::LootAllFromContainer`, see [`crate::gameplay`]).
    IFUN_LOOT_ALL_FROM_CONTAINER,
    // The inventory queries (`gmCWInterface::DoIHaveItem`, `InqSimpleInvByLocTable`,
    // `InqAllInventoryIIDList`, `IsOnInventory`, see [`crate::inventory_queries`]).
    crate::inventory_queries::DO_I_HAVE_ITEM,
    crate::inventory_queries::INQ_SIMPLE_INV_BY_LOC_TABLE,
    crate::inventory_queries::INQ_ALL_INVENTORY_IID_LIST,
    crate::inventory_queries::IS_ON_INVENTORY,
    // The crafting queries and requests (`ggmCWInterface::CallIFun_*`, see [`crate::craft`]).
    IFUN_CAN_CRAFT_SKILL_BE_RAISED,
    IFUN_CHECK_INGREDIENT,
    IFUN_DOES_INGREDIENT_AUTO_POPULATE,
    IFUN_GET_AVAILABLE_CRAFT_EXPERIENCE,
    IFUN_INQ_CRAFT_SKILL_ADAPTOR,
    IFUN_INQ_CRAFT_SKILL_CATEGORY_NAME,
    IFUN_INQ_CRAFT_SKILL_LEVEL,
    IFUN_INQ_CRAFT_SKILL_LIST,
    IFUN_INQ_RECIPE_ADAPTOR,
    IFUN_INQ_RECIPE_COST,
    IFUN_INQ_RECIPE_DIFFICULTY,
    IFUN_INQ_RECIPE_INGREDIENTS,
    IFUN_INQ_RECIPE_LIST,
    IFUN_IS_CRAFT_SKILL_RAISEABLE,
    IFUN_OPEN_RECIPE,
    IFUN_REQUEST_ADD_INGREDIENT,
    IFUN_REQUEST_EXECUTE_CRAFT,
    IFUN_REQUEST_RAISE_CRAFT_SKILL,
    // The fellowship and allegiance actions (`ac2_weenie::SocialRequest::ifun`).
    0x00B4_00A0,
    0x00B4_00A1,
    0x00B4_00A2,
    0x00B4_00A3,
    0x00B4_00A4,
    0x00B4_00A5,
    0x00B4_00A6,
    0x00B4_00B3,
    0x00B4_00B4,
    0x00B4_00B5,
    0x00B4_00B6,
    0x00B4_00B7,
];

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

/// The IFun and argument of an options page setting (`ggmCWInterface::CallIFun_Set*`).
pub fn option_call(setting: &ac2_ui::OptionSetting) -> (u32, ac2_net::protocol::PackValue) {
    use ac2_net::protocol::PackValue;
    use ac2_ui::{OptionSetting, Setter};
    let setter = match setting {
        OptionSetting::Int { setter, .. } | OptionSetting::Float { setter, .. } => *setter,
    };
    let ifun = match setter {
        Setter::AutoTarget => IFUN_SET_AUTO_TARGET,
        Setter::AcceptAllegiance => IFUN_SET_ACCEPT_ALLEGIANCE,
        Setter::AcceptFellowship => IFUN_SET_ACCEPT_FELLOWSHIP,
        Setter::AcceptTrade => IFUN_SET_ACCEPT_TRADE,
        Setter::ShowDamageText => IFUN_SET_SHOW_DAMAGE_TEXT,
        Setter::ShowDamageTextOther => IFUN_SET_SHOW_DAMAGE_TEXT_OTHER,
        Setter::ShowShortcutNumbers => IFUN_SET_SHOW_SHORTCUT_NUMBERS,
        Setter::ShowPkDamage => IFUN_SET_SHOW_PK_DAMAGE,
        Setter::ShortcutLocked => IFUN_SET_SHORTCUT_LOCKED,
        Setter::ShortcutInstaScroll => IFUN_SET_SHORTCUT_INSTA_SCROLL,
        Setter::BypassGoldMeltingConfirmation => IFUN_SET_BYPASS_GOLD_MELTING,
        Setter::LatencyIndicatorGraph => IFUN_SET_LATENCY_INDICATOR_GRAPH,
        Setter::ShowFellowsOnMap => IFUN_SET_SHOW_FELLOWS_ON_MAP,
        Setter::ShowPetUi => IFUN_SET_SHOW_PET_UI,
        Setter::ShowRadarCoords => IFUN_SET_SHOW_RADAR_COORDS,
        Setter::RadarMask => IFUN_SET_RADAR_MASK,
        Setter::DamageTextRangeOther => IFUN_SET_DAMAGE_TEXT_RANGE_OTHER,
        Setter::ShortcutHeight => IFUN_SET_SHORTCUT_HEIGHT,
    };
    let arg = match setting {
        OptionSetting::Int { value, .. } => PackValue::Int(*value),
        OptionSetting::Float { value, .. } => PackValue::Float(*value),
    };
    (ifun, arg)
}

/// The IFun and arguments of a chat options setting (`ggmCWInterface::CallIFun_SetChat*`).
pub fn chat_option_call(setting: &ac2_ui::ChatOptionSetting) -> (u32, Vec<PackValue>) {
    use ac2_ui::ChatOptionSetting as S;
    match *setting {
        S::FontSize { text_type, size } => (
            IFUN_SET_CHAT_FONT_SIZE,
            vec![PackValue::Int(text_type), PackValue::Int(size)],
        ),
        S::FontColor { text_type, color } => (
            IFUN_SET_CHAT_FONT_COLOR,
            vec![PackValue::Int(text_type), PackValue::Int(color)],
        ),
        S::PopupFlag { text_type, flag } => (
            IFUN_SET_CHAT_POPUP_FLAG,
            vec![PackValue::Int(text_type), PackValue::Int(flag)],
        ),
        S::WindowFilter { window, filter } => (
            IFUN_SET_CHAT_WINDOW_FILTER,
            vec![PackValue::Int(window), PackValue::Int(filter)],
        ),
        S::WindowTextType { window, text_type } => (
            IFUN_SET_CHAT_WINDOW_TEXT_TYPE,
            vec![PackValue::Int(window), PackValue::Int(text_type)],
        ),
    }
}

/// The script VM and the objects waiting for their process.
pub(crate) struct Scripts {
    pub(crate) host: ScriptHost,
    /// Objects whose WState is being read, and the script calls that arrived meanwhile
    /// (`CObjectMaint::GetObjectOrQueueEvent` keeps an object's events until it exists).
    pub(crate) waiting: BTreeMap<u64, Vec<(u32, Vec<u8>)>>,
    /// Objects with a process.
    pub(crate) running: BTreeSet<u64>,
    /// What the boundary dropped and dispatches that failed, for diagnostics.
    pub(crate) problems: Vec<String>,
    /// The server calls and UI IFuns handed to the VM.
    pub(crate) owned: BTreeSet<u32>,
    /// Events produced between frames (a process started on a loader result), for the next
    /// frame's events.
    pub(crate) produced: Vec<SessionEvent>,
    /// Contexts stopped for data the loader is reading: context id -> (object, the call it
    /// runs; `None` for a UI IFun, always the VM's).
    pub(crate) data_waits: BTreeMap<u32, (u64, Option<u32>)>,
    /// Script data reads handed to the loader and not back yet.
    pub(crate) data_pending: usize,
    /// The `CM_*` notices ac2-weenie has no type for, since the last take.
    pub(crate) script_notices: Vec<ScriptNotice>,
    /// GUN requests for the GUN service ([`Client::set_gun_service`]).
    pub(crate) gun_requests: Vec<ac2_script::host::ChatRequest>,
    /// Whether the client has a GUN service (`Client::set_gun_service`); the UI's requests are
    /// queued only with one.
    pub(crate) gun_service: bool,
    /// The objects whose physics objects the scripts set no-draw (`SetNoDrawPhysics`).
    pub(crate) no_draw: BTreeSet<u64>,
}

impl Client {
    /// Runs the weenies' scripts on `host` from now on, the server calls in `functions`
    /// handed over from the session's stand-ins (`OnlineSession::set_script_functions`). Build
    /// the host off the frame thread (`ScriptHost::new` over the WLib and a `DatScriptData`
    /// sharing the client's resource store, so the loader's WState reads land in its cache).
    ///
    /// Retail reads script data synchronously; here the frame thread must not, so give the
    /// host a `DatScriptData` that was preloaded on a worker (`preload_tables`,
    /// `preload_states`) and set cache-only, and the client's `DatWorld` a clone of it
    /// (`DatWorld::with_script_data`): reads the VM misses go to the loader threads and the
    /// waiting calls resume when they land.
    pub fn attach_scripts(&mut self, mut host: ScriptHost, functions: &[u32]) {
        self.session.set_script_functions(functions);
        host.set_world_name(&self.world_name);
        // Room chat goes through Turbine chat; a GUN server only when the client has one
        // (`Client::set_gun_service`), as `gmClient::Init` builds `TurbineGun` for `-gunserver`.
        host.set_chat_services(ChatServices {
            turbine_chat: true,
            gun: false,
        });
        self.scripts = Some(Scripts {
            host,
            waiting: BTreeMap::new(),
            running: BTreeSet::new(),
            problems: Vec::new(),
            owned: functions.iter().copied().collect(),
            produced: Vec::new(),
            data_waits: BTreeMap::new(),
            data_pending: 0,
            script_notices: Vec::new(),
            gun_requests: Vec::new(),
            gun_service: false,
            no_draw: BTreeSet::new(),
        });
        // Objects already known get their processes too.
        let known: Vec<u64> = self.session.objects().iter().map(|o| o.id).collect();
        for id in known {
            self.request_script(id);
        }
    }

    /// The script VM, when attached.
    pub fn script_host(&self) -> Option<&ScriptHost> {
        self.scripts.as_ref().map(|s| &s.host)
    }

    /// What the script boundary dropped and the dispatches that failed since the last call.
    pub fn take_script_problems(&mut self) -> Vec<String> {
        self.scripts
            .as_mut()
            .map(|s| std::mem::take(&mut s.problems))
            .unwrap_or_default()
    }

    /// Asks a loader thread for `id`'s script state (its weenie's package and entity DID).
    fn request_script(&mut self, id: u64) {
        let Some(scripts) = self.scripts.as_mut() else {
            return;
        };
        if scripts.running.contains(&id) || scripts.waiting.contains_key(&id) {
            return;
        }
        let object = self.session.objects().get(id);
        let Some(pid) = object.and_then(|o| o.script_package) else {
            return;
        };
        let did = object.and_then(|o| o.entity).unwrap_or(0);
        scripts.waiting.insert(id, Vec::new());
        self.loader.request(Request::ScriptState(id, pid, did));
    }

    /// The loader read `id`'s script state: its process starts, then the calls it missed run.
    pub(crate) fn script_state_loaded(&mut self, id: u64, pid: u32, did: u32) {
        let Some(scripts) = self.scripts.as_mut() else {
            return;
        };
        let Some(missed) = scripts.waiting.remove(&id) else {
            return;
        };
        if !scripts.host.create_weenie(id, pid, did) {
            if scripts.host.take_data_requests().is_empty() {
                scripts.problems.push(format!(
                    "{id:#018x}: no script process for package {pid:#x}"
                ));
            } else {
                // Its WState left the cache before the process started: read it again.
                scripts.waiting.insert(id, missed);
                self.loader.request(Request::ScriptState(id, pid, did));
            }
            return;
        }
        scripts.running.insert(id);
        if Some(id) == self.session.player() {
            scripts.host.set_player(id);
        }
        self.share_qualities(id);
        self.share_appearance(id);
        let mut events = Vec::new();
        for (function, arguments) in missed {
            self.dispatch_script(id, function, &arguments, &mut events);
        }
        if let Some(scripts) = self.scripts.as_mut() {
            scripts.produced.extend(events);
        }
    }

    /// A script data read landed; once none is outstanding, the calls waiting on data run on.
    pub(crate) fn script_data_loaded(&mut self) {
        let Some(scripts) = self.scripts.as_mut() else {
            return;
        };
        scripts.data_pending = scripts.data_pending.saturating_sub(1);
        if scripts.data_pending > 0 {
            return;
        }
        let waits = std::mem::take(&mut scripts.data_waits);
        let mut events = Vec::new();
        for (context, (id, function)) in waits {
            let Some(scripts) = self.scripts.as_mut() else {
                return;
            };
            let Some(outcome) = scripts.host.resume(id, context) else {
                continue;
            };
            self.script_outcome(id, function, Ok(outcome), &mut events);
        }
        if let Some(scripts) = self.scripts.as_mut() {
            scripts.produced.extend(events);
        }
    }

    /// The frame's session events through the scripts (after the session dispatched them).
    pub(crate) fn run_scripts(&mut self, events: &mut Vec<SessionEvent>) {
        if self.scripts.is_none() {
            return;
        }
        let mut produced = std::mem::take(&mut self.scripts.as_mut().expect("attached").produced);
        for event in events.iter() {
            match event {
                SessionEvent::ObjectAdded(id) => self.request_script(*id),
                SessionEvent::QualityChanged {
                    object,
                    table,
                    quality,
                } => {
                    self.share_quality(*object, *table, *quality);
                    self.stat_changed(*object, *table, *quality, &mut produced);
                }
                SessionEvent::ObjectAction {
                    object,
                    action: ac2_net::protocol::ObjectAction::UpdateVisualDesc(_),
                } => self.share_appearance(*object),
                SessionEvent::PlayerQualities(_) => {
                    if let Some(player) = self.session.player() {
                        self.share_qualities(player);
                    }
                }
                SessionEvent::EnteredWorld(entry) => self.request_script(entry.player),
                SessionEvent::ObjectRemoved(id) => {
                    if let Some(scripts) = self.scripts.as_mut() {
                        scripts.host.remove_weenie(*id);
                        scripts.running.remove(id);
                        scripts.waiting.remove(id);
                    }
                }
                SessionEvent::Interp {
                    object,
                    function,
                    arguments,
                } => {
                    let scripts = self.scripts.as_mut().expect("attached");
                    if let Some(queue) = scripts.waiting.get_mut(object) {
                        queue.push((*function, arguments.clone()));
                    } else if scripts.running.contains(object) {
                        self.dispatch_script(*object, *function, arguments, &mut produced);
                    }
                }
                _ => {}
            }
        }
        events.extend(produced);
    }

    /// `CWPhysicsObject::SetLocalVisualDesc` for a running process: the scripts copy the object's
    /// local description (`gCWPhysicsObject::GetLocalVisualDesc`) as the session keeps it from the
    /// server (`RecvEvt_UpdateVisualDesc`, the object's create).
    fn share_appearance(&mut self, id: u64) {
        let Some(scripts) = self.scripts.as_mut() else {
            return;
        };
        if !scripts.running.contains(&id) {
            return;
        }
        if let Some(object) = self.session.objects().get(id) {
            scripts
                .host
                .set_appearance(id, (*object.appearance).clone());
        }
    }

    /// `CWRepository::SetQualities` / `SetWeenieDesc` for a running process: the scripts read
    /// the object's qualities (`gCWRepository`) as the session keeps them from the server
    /// (the player description, the weenie description, the quality updates).
    fn share_qualities(&mut self, id: u64) {
        let Some(scripts) = self.scripts.as_mut() else {
            return;
        };
        if !scripts.running.contains(&id) {
            return;
        }
        if let Some(object) = self.session.objects().get(id) {
            scripts.host.set_qualities(id, object.qualities.clone());
        }
    }

    /// One quality update for a running process: that quality alone is copied into the
    /// scripts' repository, so what the scripts wrote since (`Agent` 0x3F's dead flag, bool 289)
    /// survives it, as it does in retail's single `CWRepository`.
    fn share_quality(&mut self, id: u64, table: u32, quality: u32) {
        let Some(scripts) = self.scripts.as_mut() else {
            return;
        };
        if !scripts.running.contains(&id) {
            return;
        }
        let Some(object) = self.session.objects().get(id) else {
            return;
        };
        match scripts.host.qualities_mut(id) {
            Some(mine) => mine.copy_quality(&object.qualities, table, quality),
            None => scripts.host.set_qualities(id, object.qualities.clone()),
        }
    }

    /// `CWRepository::RecvEvt_Update*_Private` -> `CWInterface::OnStatChanged`: once the VM
    /// has the call, the object's script hears which quality changed (`CPlayer` 0x85: the
    /// vitals, the XP bar, the skill panel; `CItem` 0x3: the item update).
    fn stat_changed(&mut self, id: u64, table: u32, quality: u32, out: &mut Vec<SessionEvent>) {
        let function = ac2_weenie::IFUN_ON_STAT_CHANGED;
        let Some(scripts) = self.scripts.as_mut() else {
            return;
        };
        if !scripts.owned.contains(&function) || !scripts.running.contains(&id) {
            return;
        }
        let args = [IFunArg::Int(table), IFunArg::Int(quality)];
        let result = scripts
            .host
            .call_ifun(id, function, &args)
            .map(|r| r.outcome)
            .map_err(|e| format!("{e:?}"));
        self.script_outcome(id, None, result, out);
    }

    /// `CPhysicsObj::ReportCollisions` -> `CWInterface::HandleCollision`: the object's script
    /// hears the impact it recorded (its contact normal and speed, as a `CollisionProfile`).
    /// Only an object whose script runs and owns the IFun hears it.
    pub(crate) fn collision_reported(
        &mut self,
        id: u64,
        report: ac2_physics::ImpactReport,
        out: &mut Vec<SessionEvent>,
    ) {
        let function = ac2_weenie::IFUN_HANDLE_COLLISION;
        let Some(scripts) = self.scripts.as_mut() else {
            return;
        };
        if !scripts.owned.contains(&function) || !scripts.running.contains(&id) {
            return;
        }
        let normal = [report.normal.x, report.normal.y, report.normal.z];
        let profile = ac2_script::native::CollisionProfile::impact(normal, report.speed);
        let args = [IFunArg::Vm(
            ac2_script::native::ntype::COLLISION_PROFILE,
            ac2_script::native::NativeObject::CollisionProfile(profile),
        )];
        let result = scripts
            .host
            .call_ifun(id, function, &args)
            .map(|r| r.outcome)
            .map_err(|e| format!("{e:?}"));
        self.script_outcome(id, None, result, out);
    }

    /// Whether the scripts set `id`'s physics object no-draw (`CPhysicsObj::is_nodraw`).
    pub fn is_no_draw(&self, id: u64) -> bool {
        self.scripts
            .as_ref()
            .is_some_and(|s| s.no_draw.contains(&id))
    }

    /// Whether the UI's IFun (or server call) `function` is the script VM's.
    pub(crate) fn scripts_own(&self, function: u32) -> bool {
        self.scripts
            .as_ref()
            .is_some_and(|s| s.owned.contains(&function))
            && self.session.player().is_some_and(|p| {
                self.scripts
                    .as_ref()
                    .is_some_and(|s| s.running.contains(&p))
            })
    }

    /// `ggmCWInterface::CallIFun_*`: the IFun on the player's process (a 0 return slot, the
    /// arguments, the instance, `VM::CallIFun`), its output sent and returned.
    pub(crate) fn script_ifun(&mut self, function: u32, args: &[PackValue]) -> Vec<SessionEvent> {
        let args: Vec<IFunArg> = args
            .iter()
            .filter_map(|a| match a {
                PackValue::Int(v) => Some(IFunArg::Int(*v)),
                PackValue::LongInt(v) => Some(IFunArg::Long(*v)),
                other => other.as_native().cloned().map(IFunArg::Native),
            })
            .collect();
        self.script_ifun_args(function, &args)
    }

    /// [`Client::script_ifun`] with the arguments given directly, native objects included (a
    /// native the client makes, as `gmCWInterface::SetUISaveLocations` passes its save).
    pub(crate) fn script_ifun_args(
        &mut self,
        function: u32,
        args: &[IFunArg],
    ) -> Vec<SessionEvent> {
        let mut out = Vec::new();
        if let Err(error) = self.ifun_into(function, args, &mut out) {
            if let Some(scripts) = self.scripts.as_mut() {
                scripts.problems.push(error);
            }
        }
        out
    }

    /// `CWDistributor::DispatchInterpMsg`, then the scripts' output (`script_outcome`).
    fn dispatch_script(
        &mut self,
        id: u64,
        function: u32,
        arguments: &[u8],
        out: &mut Vec<SessionEvent>,
    ) {
        let scripts = self.scripts.as_mut().expect("attached");
        let result = unpack_values(arguments)
            .map_err(|e| e.to_string())
            .and_then(|values| {
                scripts
                    .host
                    .dispatch(id, function, &values)
                    .map_err(|e| format!("{e:?}"))
            });
        self.script_outcome(id, Some(function), result, out);
    }
}
