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

use ac2_net::protocol::{unpack_values, PackValue};
use ac2_script::host::{ChatServices, IFunArg, ScriptHost, ScriptState};
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

pub use ac2_script::host::ifun_ids::*;

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
        self.scripts = Some(ScriptState::new(host, functions));
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
            let reads = scripts.host.take_data_requests();
            if reads.is_empty() {
                scripts.problems.push(format!(
                    "{id:#018x}: no script process for package {pid:#x}"
                ));
                return;
            }
            // The WState is not resident yet: the loader reads what create_weenie missed, and
            // the process is tried again once those answer (absent records answer too, and the
            // retry then starts the process from the fresh instance, as retail's null does).
            scripts.waiting.insert(id, missed);
            scripts.state_retries.insert(id, (pid, did));
            scripts.data_pending += reads.len();
            for read in reads {
                self.loader.request(Request::ScriptData(read));
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

    /// A script data read landed (`answered`: the world resolved it, see `Loaded::ScriptData`);
    /// once none is outstanding, the calls waiting on data run on and the objects whose process
    /// waited for a WState start again. A batch with an unanswered read stops here: those waits
    /// stay until the world answers, and nothing is asked again in a loop.
    pub(crate) fn script_data_loaded(&mut self, answered: bool) {
        let Some(scripts) = self.scripts.as_mut() else {
            return;
        };
        scripts.data_pending = scripts.data_pending.saturating_sub(1);
        scripts.data_unanswered |= !answered;
        if scripts.data_pending > 0 {
            return;
        }
        if std::mem::take(&mut scripts.data_unanswered) {
            return;
        }
        let retries = std::mem::take(&mut scripts.state_retries);
        let waits = std::mem::take(&mut scripts.data_waits);
        for (id, (pid, did)) in retries {
            self.loader.request(Request::ScriptState(id, pid, did));
        }
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
            ac2_script::native::NativeObject::CollisionProfile(Box::new(profile)),
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

    /// Whether the UI's IFun (or server call) `function` is the script VM's
    /// (`ac2_script::host::ui_calls::scripts_own`).
    pub(crate) fn scripts_own(&mut self, function: u32) -> bool {
        ac2_script::host::ui_calls::scripts_own(self, function)
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
