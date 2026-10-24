//! Event opcodes: the first u32 of every event blob.
//!
//! Server events are listed with their Legions 7160 handler: `CEventQueue::DispatchEvent`
//! (0x0048C7E0, queue 1) with `gmCEventQueue::DispatchEvent` (0x006A3540) taking `0x10058`;
//! `Turbine::Client::ProcessLogonEventQueue` (queue 4); and the data-download handler
//! `CLCache::UseTime` (Legions 0x004B53A0, queue 5). Client events are listed with their
//! sender and the queue `Proto_UI::SendBuff` gives them. Opcodes are identical in admin 7148.

/// Event opcodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Opcode;

impl Opcode {
    // Login (`ECM_Login`).
    /// S: `ECM_Login::RecvEvt_CharExitGame` (no body). C: `Proto_UI::LogOffCharacter`, queue 4.
    pub const CHAR_EXIT_GAME: u32 = 0x0001_0027;
    /// S: `ECM_Login::RecvEvt_CharacterDeletion`. C: `Proto_UI::SendDeleteCharacter`, queue 4.
    pub const CHARACTER_DELETION: u32 = 0x0001_0029;
    /// C: `Proto_UI::SendCharGenResult`, queue 8 (4 without a result).
    pub const CHAR_GEN_RESULT: u32 = 0x0001_002A;
    /// C: `Proto_UI::SendEnterWorld`, queue 4.
    pub const ENTER_WORLD: u32 = 0x0001_002B;
    /// C: `Proto_UI::SendAdminRestoreCharacter`, queue 6.
    pub const ADMIN_RESTORE_CHARACTER: u32 = 0x0001_0051;
    /// S: `ECM_Login::RecvEvt_CharacterSet`.
    pub const CHARACTER_SET: u32 = 0x0001_0055;
    /// S: `ECM_Login::RecvEvt_CharacterError`.
    pub const CHARACTER_ERROR: u32 = 0x0001_0056;
    /// S: `ECM_Login::RecvEvt_CharGenVerification`.
    pub const CHAR_GEN_VERIFICATION: u32 = 0x0001_0057;
    /// S: `ECM_Login::RecvEvt_PlayerDesc` (`CBaseQualities`).
    pub const PLAYER_DESCRIPTION: u32 = 0x0001_0068;
    /// S: `ECM_Login::RecvEvt_MinCharSet`.
    pub const MIN_CHARACTER_SET: u32 = 0x0001_00B8;
    /// C: `ECM_Login::SendEvt_ClientSceneRenderingComplete`, queue 3, sent by
    /// `gmGamePlayUI::RecvNotice_NextFrameDrawn`.
    pub const SCENE_RENDERING_COMPLETE: u32 = 0x0001_00D1;

    // Data download (`CCliDatEvent`, queue 5, unordered out-of-world ids).
    /// C: `CCliDatRequestEvent::CreateForSend`: data type and id.
    pub const DAT_REQUEST: u32 = 0x0001_0034;
    /// S: data for a request (`CCliDatDataEvent`).
    pub const DAT_DATA: u32 = 0x0001_0035;
    /// S: a request failed (`AsyncCache::OnAsyncGetFromOtherSourcesFailed`).
    pub const DAT_REQUEST_FAILED: u32 = 0x0001_0036;
    /// S: `CLCache::OnServerInterrogation`.
    pub const DAT_INTERROGATION: u32 = 0x0001_0037;
    /// C: `CCliDatInterrogationResponseEvent::CreateForSend`: language and iteration lists.
    pub const DAT_INTERROGATION_RESPONSE: u32 = 0x0001_0038;
    /// S: `NotifyDDDEvent(DDD_PatchtimePending)`.
    pub const DAT_PATCHTIME_PENDING: u32 = 0x0001_0039;
    /// S: `CLCache::OnBeginDDD`: missing iterations.
    pub const BEGIN_DDD: u32 = 0x0001_003A;
    /// S: `CLCache::OnEndDDD`. C: `CCliDatEndDDDEvent::CreateForSend`.
    pub const END_DDD: u32 = 0x0001_003B;

    // Interpreter (`ECM_Interp`).
    /// C: `ECM_Interp::SendEvt_InterpSEvent`, queue 3.
    pub const INTERP_S_EVENT: u32 = 0x0001_0053;
    /// S: `ECM_Interp::RecvEvt_InterpCEvent_Broadcast` (also `0x100CA`).
    pub const INTERP_C_EVENT_BROADCAST: u32 = 0x0001_0066;
    /// S: `ECM_Interp::RecvEvt_InterpCEvent_Cell` (also `0x100C4`).
    pub const INTERP_C_EVENT_CELL: u32 = 0x0001_0067;
    pub const INTERP_C_EVENT_CELL_2: u32 = 0x0001_00C4;
    pub const INTERP_C_EVENT_BROADCAST_2: u32 = 0x0001_00CA;
    /// C: `ECM_Interp::SendEvt_InterpSEventEncrypt`, queue 7.
    pub const INTERP_S_EVENT_ENCRYPT: u32 = 0x0001_00D0;

    // Qualities (`ECM_Qualities`): private/visual pairs.
    pub const UPDATE_INT_PRIVATE: u32 = 0x0001_0069;
    pub const UPDATE_LONG_INT_VISUAL: u32 = 0x0001_00D5;

    // Physics (`ECM_Physics`, `CM_Physics`).
    /// S: `CM_Physics::RecvEvt_CreateObject`.
    pub const CREATE_OBJECT: u32 = 0x0001_0058;
    /// S: `ECM_Physics::RecvEvt_CreatePlayer`.
    pub const CREATE_PLAYER: u32 = 0x0001_0060;
    pub const DO_BEHAVIOR: u32 = 0x0001_007F;
    pub const STOP_BEHAVIOR: u32 = 0x0001_0080;
    pub const DO_SLIDER: u32 = 0x0001_0081;
    pub const DO_MODE: u32 = 0x0001_0082;
    /// S: `ECM_Physics::RecvEvt_MoveTo`.
    pub const MOVE_TO: u32 = 0x0001_0083;
    /// S: `ECM_Physics::RecvEvt_Position`.
    pub const UPDATE_POSITION: u32 = 0x0001_0086;
    pub const PARENT: u32 = 0x0001_0087;
    /// C: `ECM_Physics::SendEvt_CPosition`, queue 3.
    pub const CLIENT_POSITION: u32 = 0x0001_008E;
    /// C: `ECM_Physics::SendEvt_CForceCreate`, queue 3.
    pub const CLIENT_FORCE_CREATE: u32 = 0x0001_008F;
    pub const SET_HOVER_HEIGHT: u32 = 0x0001_0091;
    pub const SET_VELOCITY_SCALE: u32 = 0x0001_0092;
    pub const SET_ACCELERATION_SCALE: u32 = 0x0001_0093;
    pub const SET_MIDESC: u32 = 0x0001_0094;
    pub const STOP_LAYER: u32 = 0x0001_0095;
    /// S: `ECM_Physics::RecvEvt_LookAt`.
    pub const LOOK_AT: u32 = 0x0001_0096;
    pub const ADD_IMPULSE: u32 = 0x0001_0098;
    /// S: `ECM_Physics::RecvEvt_AddTargetedImpulse`.
    pub const ADD_TARGETED_IMPULSE: u32 = 0x0001_0099;
    /// S: `ECM_Physics::RecvEvt_ReleaseBehavior`.
    pub const RELEASE_BEHAVIOR: u32 = 0x0001_009B;
    pub const UPDATE_VISUAL_DESC: u32 = 0x0001_009F;
    pub const DO_FX: u32 = 0x0001_00A0;
    pub const STOP_FX: u32 = 0x0001_00A1;
    pub const DO_TARGETTED_FX: u32 = 0x0001_00A2;
    /// S: `ECM_Physics::RecvEvt_AddTerrainModifier`.
    pub const ADD_TERRAIN_MODIFIER: u32 = 0x0001_00A5;
    pub const CLEAR_TERRAIN_MODIFIERS: u32 = 0x0001_00A6;
    pub const SET_GLOBAL_MODIFIER: u32 = 0x0001_00A7;
    /// S: `ECM_Physics::RecvEvt_DoStory`.
    pub const DO_STORY: u32 = 0x0001_00A8;
    pub const SET_MODE: u32 = 0x0001_00A9;
    /// S: `ECM_Physics::RecvEvt_InitializeMissile`.
    pub const INITIALIZE_MISSILE: u32 = 0x0001_00B5;
    pub const SET_JUMP_SCALE: u32 = 0x0001_00BA;
    pub const DO_FX_SCRIPT: u32 = 0x0001_00BE;
    pub const LEAVE_WORLD: u32 = 0x0001_00BF;
    pub const POSITION_CELL: u32 = 0x0001_00C1;
    /// S: `ECM_Physics::RecvEvt_DestroyObject`.
    pub const DESTROY_OBJECT: u32 = 0x0001_00C2;
    pub const DE_PARENT: u32 = 0x0001_00C3;
    pub const STOP_ALL_WEENIE_BEHAVIORS: u32 = 0x0001_00C5;
    /// C: `ECM_Physics::SendEvt_CLookAt`, queue 3.
    pub const CLIENT_LOOK_AT: u32 = 0x0001_00C7;
    /// C: `ECM_Physics::SendEvt_CLookAtDir`, queue 3.
    pub const CLIENT_LOOK: u32 = 0x0001_00C8;
    /// S: `ECM_Physics::RecvEvt_LookAtDir`.
    pub const LOOK: u32 = 0x0001_00C9;
    pub const CONTAIN: u32 = 0x0001_00CB;
    pub const STOP_MOVE_TO_CANCELS_BEHAVIORS: u32 = 0x0001_00CF;
    /// S: `ECM_Physics::RecvEvt_DoFX_Private`.
    pub const DO_FX_PRIVATE: u32 = 0x0001_00D2;

    // Admin (`ECM_Admin`).
    pub const ACCOUNT_BOOTED: u32 = 0x0001_00B4;
    pub const DISPLAY_STRING_INFO: u32 = 0x0001_00C6;
    pub const WORLD_NAME: u32 = 0x0001_00CE;
    /// S, queue 4: chat (`ProcessLogonEventQueue` hands it to the chat client).
    pub const CHAT: u32 = 0x0001_00D3;
}
