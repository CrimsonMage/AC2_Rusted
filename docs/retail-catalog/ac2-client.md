# ac2-client: Client entry, shell, preferences, platform devices

354 retail functions: 187 done, 0 partial, 0 to do, 167 skipped.
Legend and workflow: [README](README.md).

## `ac2/client/src/main/billingmigration.cpp.c` (0/6)

### Listener
- [-] `Listener::Listener` · 006f89c0 · legions 006788f0 — not needed: Retail billing-migration web check (gmclient.cpp.c:1904 URL acbm.turbinegames.com, called from gmClient::Init at gmclient.cpp.c:2804); defunct Turbine billing service; no supported server (Pyrealium, 7230) uses it
- [-] `Listener::OnError` · 006f89f0 — not needed: Retail billing-migration web check (gmclient.cpp.c:1904 URL acbm.turbinegames.com, called from gmClient::Init at gmclient.cpp.c:2804); defunct Turbine billing service; no supported server (Pyrealium, 7230) uses it
- [-] `Listener::Reset` · 006f8a00 — not needed: Retail billing-migration web check (gmclient.cpp.c:1904 URL acbm.turbinegames.com, called from gmClient::Init at gmclient.cpp.c:2804); defunct Turbine billing service; no supported server (Pyrealium, 7230) uses it
- [-] `Listener::OnResponse` · 006f8a10 — not needed: Retail billing-migration web check (gmclient.cpp.c:1904 URL acbm.turbinegames.com, called from gmClient::Init at gmclient.cpp.c:2804); defunct Turbine billing service; no supported server (Pyrealium, 7230) uses it

### (free functions)
- [-] `CheckForBillingMigrationInternal` · 006f8aa0 · legions 006789d0 — not needed: Retail billing-migration web check (gmclient.cpp.c:1904 URL acbm.turbinegames.com, called from gmClient::Init at gmclient.cpp.c:2804); defunct Turbine billing service; no supported server (Pyrealium, 7230) uses it
- [-] `CheckForBillingMigration` · 006f96e0 · legions 00679610 — not needed: Retail billing-migration web check (gmclient.cpp.c:1904 URL acbm.turbinegames.com, called from gmClient::Init at gmclient.cpp.c:2804); defunct Turbine billing service; no supported server (Pyrealium, 7230) uses it

## `ac2/client/src/main/climain.cpp.c` (0/6)

### PackObj
- [-] `PackObj::GetPackSize` · 00401000 · legions 00401000 — replaced by design: Rust packers build a Vec<u8> (authenticator() etc.); no retained length field needed [crates/ac2-net/src/handshake.rs:116]

### (free functions)
- [-] ``vector_constructor_iterator'` · 00401010 · legions 00401010 — STL vector construct/destruct iterator helpers (MSVC internals)
- [-] ``vector_destructor_iterator'` · 00401040 · legions 00401040 — STL vector construct/destruct iterator helpers (MSVC internals)

### ReferenceCountTemplate<268435456,0>
- [-] `ReferenceCountTemplate<268435456,0>::Release` · 004010a0 · legions 00401dc0 — reference-count release; Rust Arc/Drop

### CMostlyConsecutiveIntSet
- [-] `CMostlyConsecutiveIntSet::~CMostlyConsecutiveIntSet` · 004010f0 · legions 004010c0 — ctor/dtor of plain data

### (free functions)
- [-] `WinMain` · 004013a0 · legions 00401370 — WinMain: Win32 entry point, Rust main

## `ac2/client/src/main/gmclient.cpp.c` (57/92)

### PackObj
- [x] `PackObj::ALIGN_PTR` · 00401a60 · legions 00401980 — crates/ac2-script/src/unpack.rs `align` cites PackObj::ALIGN_PTR (4-byte alignment)

### NetError
- [-] `NetError::operator!=` · 00401aa0 · legions 004019c0 — operator==/=/+/!= of plain data (string, id, error); Rust derives or uses String
- [x] `NetError::operator_StringInfo` · 00401ad0 · legions 004019f0 — audit: covered: NetError -> retail string id -> text (net_error_id) for the disconnected screen [crates/ac2-ui/src/pregame/connect_failed.rs:1]
- [x] `NetError::GetPackSize` · 00401b20 · legions 00401a40
- [x] `NetError::Pack` · 00401b30 · legions 00401a50
- [x] `NetError::UnPack` · 00401b80 · legions 00401aa0

### ArgumentParser
- [x] `ArgumentParser::OnCommandLineEvaluationDone` · 00401bd0 · legions 00401af0 — audit: covered:  [crates/ac2-client/src/bin/ac2/cmdline.rs:243]

### Turbine::Client::ChatInterfaceProvider
- [x] `Turbine::Client::ChatInterfaceProvider::UseTime` · 00401be0 · legions 00401b00 — Legions body is empty / returns 0: the DVD client has no room-by-name result or per-frame chat work; rooms are set by server calls (shell/state.rs)
- [x] `Turbine::Client::ChatInterfaceProvider::QueryInterface` · 00401bf0 · legions 00401b10 — Legions returns E_NOINTERFACE: the port has no COM interface query

### gmNoticeHandler
- [x] `gmNoticeHandler::RecvNotice_Patron_SwearDone` · 00401c30 · legions 00401b70 — base virtual is empty; the allegiance page overrides it: ui_social.rs N::PatronSwearDone -> patron_swear_done (hud_player (frame_passes notices))
- [x] `gmNoticeHandler::RecvNotice_AddMarker` · 00401c40 — base virtual is empty; the navigation UI overrides it: client_radar.rs radar_notice -> add_map_note / marker

### Turbine::Client::ChatInterfaceProvider
- [x] `Turbine::Client::ChatInterfaceProvider::OnSendToRoom` · 00401c50 · legions 00401b90 — ac2-weenie OnlineSession::send_to_room; the Legions vtable stub is empty (room sends go through gmChatInterfaceProvider::OnSendToRoom)
- [x] `Turbine::Client::ChatInterfaceProvider::OnEnterRoomByNameResult` · 00401c60 · legions 00401bc0 — Legions body is empty / returns 0: the DVD client has no room-by-name result or per-frame chat work; rooms are set by server calls (shell/state.rs)

### gmNoticeHandler
- [-] `gmNoticeHandler::RecvNotice_GUNResult_FriendStatus` · 00401c70 — GUN service result (GUN is gone); the friends list is not fed by a GUN answer here (ac2-weenie friends.rs)
- [x] `gmNoticeHandler::RecvNotice_InitStackUI` · 00401c80 · legions 00401be0 — base virtual is empty; the stack UI is set up by ShopStackCall::Init (hud_shop.rs shop_stack_calls)

### gmCharNameCharacterValidator
- [x] `gmCharNameCharacterValidator::SetLanguage` · 00401c90 · legions 00401bf0 — ac2-db char_names.rs (LanguageLetters from Legions 0x00A3B7D0; language set from the local DAT's name in mapper 0x4000002A, g_validator->SetLanguage at 0x004B43E2); ac2-ui confirmation.rs is_name_character = NameInputFilter; tests each_language_takes_ascii_and_its_own_letters, only_korean_takes_the_jamo..., the_language_names_map_to_the_name_validators_ids
- [x] `gmCharNameCharacterValidator::GetLanguage` · 00401ca0 · legions 00401c00 — ac2-db char_names.rs (LanguageLetters from Legions 0x00A3B7D0; language set from the local DAT's name in mapper 0x4000002A, g_validator->SetLanguage at 0x004B43E2); ac2-ui confirmation.rs is_name_character = NameInputFilter; tests each_language_takes_ascii_and_its_own_letters, only_korean_takes_the_jamo..., the_language_names_map_to_the_name_validators_ids

### IError
- [-] `IError::ReportGenericError` · 00401cb0 — Win32 message-box error reporter; Rust client reports through its own UI

### AppliedAppearanceKey
- [-] `AppliedAppearanceKey::~AppliedAppearanceKey` · 00401cd0 · legions 00401c10 — ctor/dtor of plain data

### StreamPackObj
- [-] `StreamPackObj::Pack` · 00401cf0 — replaced by design: Abstract base virtual; each type has its own Rust encode/decode [crates/ac2-ui/src/layout_media.rs:171; crates/ac2-net/src/protocol/]
- [-] `StreamPackObj::UnPack` · 00401d30 · legions 00401ca0 — replaced by design: Abstract base virtual; each type has its own Rust encode/decode [crates/ac2-ui/src/layout_media.rs:171; crates/ac2-net/src/protocol/]

### gmClient
- [-] `gmClient::IsShuttingDown` · 00401d80 · legions 00401cf0 — replaced by design: winit event loop exit (CloseRequested -> log_off -> exit) replaces the shutdown flag [crates/ac2-client/src/bin/ac2/main.rs:619]

### PackObj
- [-] `PackObj::PackObj` · 00401d90 · legions 00401d00 — ctor/dtor of plain data

### gmClient
- [-] `gmClient::InitWeenie` · 00401da0 · legions 00401d10 — replaced by design: No Client object; the weenie session is built in run() (OnlineSession) [crates/ac2-client/src/client.rs:403]
- [-] `gmClient::OnDDDEvent` · 00401dc0 · legions 00401d30 — not needed: Data-patch progress UI/events. Client answers the DAT interrogation (session.rs:561) but never downloads (data_patch.rs header: "the DATs never patch here"); supported servers serve data matching the mounted DATs. Revisit only if a server pushes DDD patches [crates/ac2-weenie/src/ddd.rs:
- [x] `gmClient::RecvNotice_CharacterSet` · 00401e00 — ac2-weenie gun/client.rs recv_notice_character_set (Legions 0x00401D70, wired by ac2-client gun.rs gun_session_event): the two flags; the trailing slot +0x9C is Turbine::Client::StartupTurbineChat (Legions 0x006A2A30, gmClient vtable 0x009C60A0), whose chat manager is the built-in Turbine chat (set_chat_client); AGENTS.md GUN entry updated
- [x] `gmClient::UseTime` · 00401e30 · legions 00401da0 — GUN phase 3 (gun-phase3)

### PStringBase<char>
- [x] `PStringBase<char>::GetPackSize` · 00401e70 · legions 00401e10
- [-] `PStringBase<char>::operator==` · 00401ea0 · legions 00401e40 — operator==/=/+/!= of plain data (string, id, error); Rust derives or uses String

### PSmartArray<unsigned_long>
- [-] `PSmartArray<unsigned_long>::~PSmartArray<unsigned_long>` · 00401f00 · legions 00401ea0 — ctor/dtor of plain data

### Turbine::Client::ChatInterfaceProvider
- [-] `Turbine::Client::ChatInterfaceProvider::QueryInterface` · 00402170 — COM QueryInterface plumbing
- [-] `Turbine::Client::ChatInterfaceProvider::QueryInterface` · 00402180 — inline copy of 00402170

### CTBullet
- [x] `CTBullet::NumEventsHandled` · 00402190 · legions 00402130 — ported in ac2-gun (gun-phase2), cited per item
- [x] `CTBullet::IncEventsHandled` · 004021a0 · legions 00402140 — ported in ac2-gun (gun-phase2), cited per item

### gmClient::gmChatInterfaceProvider
- [-] `gmClient::gmChatInterfaceProvider::~gmChatInterfaceProvider` · 004021d0 · legions 00402170 — COM refcount and object lifetime: Rust ownership (the chat provider keeps no GUN state here)
- [x] `gmClient::gmChatInterfaceProvider::OnSendToRoomByIDResult` · 00402240 — GunShared::answer_turbine_send: ctx.m_hr = hr, Result_SendToRoom

### PStringBase<unsigned_short>
- [-] `PStringBase<unsigned_short>::operator=` · 00402380 · legions 00402300 — operator==/=/+/!= of plain data (string, id, error); Rust derives or uses String

### StringInfo
- [-] `StringInfo::IsOnDisk` · 004025c0 · legions 00402540 — string-table cache flag; strings read on demand (AGENTS.md cache rules)

### gmClient
- [x] `gmClient::~gmClient` · 00402650 · legions 004025a0 — GUN phase 3 (gun-phase3)

### Turbine::Client
- [x] `Turbine::Client::GetWidth` · 00402830 · legions 00402780 — Args (account, host, port, width, height) and Args::server_address, the window size
- [x] `Turbine::Client::GetHeight` · 00402840 · legions 00402790 — Args (account, host, port, width, height) and Args::server_address, the window size
- [x] `Turbine::Client::GetAccount` · 00402850 · legions 004027a0 — Args (account, host, port, width, height) and Args::server_address, the window size
- [x] `Turbine::Client::GetHostName` · 00402860 · legions 004027b0 — Args (account, host, port, width, height) and Args::server_address, the window size
- [x] `Turbine::Client::GetPort` · 00402870 · legions 004027c0 — Args (account, host, port, width, height) and Args::server_address, the window size
- [x] `Turbine::Client::SetPortA` · 00402880 · legions 004027d0 — Args (account, host, port, width, height) and Args::server_address, the window size
- [x] `Turbine::Client::GetWorldName` · 00402890 · legions 004027e0 — Turbine::Client world name: set by the WORLD_NAME notice (Client::set_world_name), read by the efun and the assistance page
- [x] `Turbine::Client::SetHostName` · 004028a0 · legions 004027f0 — audit: covered:  [crates/ac2-net/src/resolve.rs:3]
- [x] `Turbine::Client::SetWorldName` · 004028f0 · legions 00402840 — audit: covered: show_world_names to window title [crates/ac2-client/src/bin/ac2/main.rs:325]

### CCliChatEvent
- [-] `CCliChatEvent::CCliChatEvent` · 00402940 · legions 00402890 — COM refcount and object lifetime: Rust ownership (CCliChatEvent is a GUN event class; GUN is gone)

### gmClient
- [x] `gmClient::RecvNotice_GunAuthenticated` · 004029d0 — GUN phase 3 fix round
- [x] `gmClient::AuthenticateWithGUN` · 00402ae0 · legions 00402a30 — GUN phase 3 (gun-phase3)

### RefCountIUnknown<gmClient::gmChatInterfaceProvider>
- [-] `RefCountIUnknown<gmClient::gmChatInterfaceProvider>::RefCountIUnknown<gmClient::gmChatInterfaceProvider>` · 00402d00 · legions 00402c50 — COM refcount and object lifetime: Rust ownership (CCliChatEvent is a GUN event class; GUN is gone)
- [-] `RefCountIUnknown<gmClient::gmChatInterfaceProvider>::AddRef` · 00402d70 · legions 00402cc0 — COM refcount and object lifetime: Rust ownership (CCliChatEvent is a GUN event class; GUN is gone)
- [-] `RefCountIUnknown<gmClient::gmChatInterfaceProvider>::Release` · 00402d90 · legions 00402ce0 — COM refcount and object lifetime: Rust ownership (CCliChatEvent is a GUN event class; GUN is gone)
- [-] `RefCountIUnknown<gmClient::gmChatInterfaceProvider>::AddRef` · 00402dd0 — inline copy of 00402d70
- [-] `RefCountIUnknown<gmClient::gmChatInterfaceProvider>::Release` · 00402de0 — inline copy of 00402d90
- [-] `RefCountIUnknown<gmClient::gmChatInterfaceProvider>::AddRef` · 00402df0 — inline copy of 00402d70
- [-] `RefCountIUnknown<gmClient::gmChatInterfaceProvider>::Release` · 00402e00 — inline copy of 00402d90
- [-] `RefCountIUnknown<gmClient::gmChatInterfaceProvider>::~RefCountIUnknown<gmClient::gmChatInterfaceProvider>` · 00402e40 · legions 00402d90 — COM refcount and object lifetime: Rust ownership (CCliChatEvent is a GUN event class; GUN is gone)

### PStringBase<unsigned_short>
- [-] `PStringBase<unsigned_short>::operator+` · 00402ef0 · legions 00402e60 — operator==/=/+/!= of plain data (string, id, error); Rust derives or uses String

### NetError
- [-] `NetError::GetLogString` · 00403280 · legions 004031f0 — not needed: Log text only (callers: OnConnectFailed, GunReconnected log); Rust NetError Display logs [crates/ac2-net/src/error.rs:35]

### ArgumentParser::CommandLineArgList
- [-] `ArgumentParser::CommandLineArgList::AddCmd` · 004032f0 · legions 00403260 — replaced by design: Rust argument parser replaces the retail arg list object [crates/ac2-client/src/bin/ac2/cmdline.rs:92]

### NSTurbineGun::TurbineGun
- [x] `NSTurbineGun::TurbineGun::SetWorldName` · 004033a0 · legions 00403310 — ported in ac2-gun (gun-phase2), cited per item

### gmClient
- [x] `gmClient::gmClient` · 004034a0 · legions 00403410 — GUN phase 3 (gun-phase3)

### CCliConnectEvent
- [x] `CCliConnectEvent::CCliConnectEvent` · 004035c0 · legions 00403530 — GUN phase 3 (gun-phase3)

### gmClient
- [x] `gmClient::BuildCommandLineArgs` · 00403650 · legions 004035c0 — GUN phase 3 fix round
- [x] `gmClient::EvaluateCommandLineArg` · 00403ae0 · legions 00403a50 — GUN phase 3 fix round
- [x] `gmClient::OnCommandLineEvaluationDone` · 00403b60 · legions 00403ad0 — GUN phase 3 (gun-phase3)
- [x] `gmClient::RegisterForGunEvents` · 00403c10 · legions 00403b80 — GUN phase 3 (gun-phase3)
- [x] `gmClient::InitKeymap` · 00403d30 · legions 00403c80 — audit: covered: Catalog note stale: keymap_file module (KeymapFiles, MAIN_KEY_UPDATES) reads the player .keymap files [crates/ac2-ui/src/controls.rs:84]
- [x] `gmClient::RecvNotice_WorldName` · 00403ff0 · legions 00403f40 — GUN phase 3 (gun-phase3)
- [x] `gmClient::RecvNotice_GunReconnected` · 00404080 — GUN phase 3 (gun-phase3)

### gmClient::gmChatInterfaceProvider
- [x] `gmClient::gmChatInterfaceProvider::OnSendToRoom` · 00404100 · legions 00404050 — ac2-weenie OnlineSession::send_to_room / receive_chat over ac2-net turbine_chat (chatclient.dll's protocol)

### gmClient
- [x] `gmClient::CreateChatInterfaceProvider` · 00404190 · legions 004040e0 — the session is the chat provider (ac2-weenie OnlineSession send_to_room / receive_chat over ac2-net turbine_chat)

### PStringBase<char>
- [x] `PStringBase<char>::Pack` · 004042f0 · legions 00404240

### accountID
- [-] `accountID::operator=` · 004043d0 — operator==/=/+/!= of plain data (string, id, error); Rust derives or uses String

### Turbine::Client
- [x] `Turbine::Client::SetAccount` · 004043f0 · legions 00404320 — audit: covered:  [crates/ac2-client/src/bin/ac2/cmdline.rs:13]

### gmClient
- [x] `gmClient::Init` · 00404410 · legions 00404340 — audit: covered: Only the billing-migration step (Init step 5) is unported; see Listener rows (not-needed). ForceDisplayResolution(1,0x400,0x300) in Init is replaced by winit window [crates/ac2-client/src/bin/ac2/main.rs:763]
- [x] `gmClient::StartupGUN` · 00404a40 · legions 00404820 — audit: covered: KeepUIAlive failure branches covered [crates/ac2-client/src/gun.rs:113]
- [x] `gmClient::InitAuth` · 00404e10 · legions 00404bb0 — GUN phase 3 (gun-phase3)
- [x] `gmClient::Connect` · 00404f80 · legions 00404d20 — GUN phase 3 fix round

### gmNoticeHandler
- [-] `gmNoticeHandler::RecvNotice_GUNResult_GetFriendsList` · 00405050 — GUN service result (GUN is gone); the friends list is not fed by a GUN answer here (ac2-weenie friends.rs)
- [-] `gmNoticeHandler::RecvNotice_FinishPlaySequence` · 00405210 — the base gmNoticeHandler body only frees the list it was passed (as NoticeHandler::RecvNotice_CharacterSet, skipped); the story UI's own receiver is the live one (ac2-client client_story.rs)
- [x] `gmNoticeHandler::RecvNotice_PlaySequence` · 00405230 — base virtual flushes its list; the story slot takes the scenes: client_story.rs GameplayNotice::PlaySequence

### FXNode
- [-] `FXNode::~FXNode` · 00405250 · legions 00404ff0 — ctor/dtor of plain data

### gmNoticeHandler
- [x] `gmNoticeHandler::RecvNotice_UpdateConsignment` · 00405330 — base virtual releases its arguments; the panel is the override: hud_shop.rs consignment_notice / catalog_notice via hud_player (frame_passes notices)

### UIShop
- [-] `UIShop::~UIShop` · 00405340 · legions 004050e0 — ctor/dtor of plain data

### gmNoticeHandler
- [x] `gmNoticeHandler::RecvNotice_UpdateStore` · 004054b0 — base virtual releases its arguments; hud_shop.rs shop_notice -> recv_update_store (hud_player (frame_passes notices))

### PHashTable<unsigned_long,unsigned_long>
- [x] `PHashTable<unsigned_long,unsigned_long>::StreamPack` · 00405500 — audit: covered: Client only unpacks; the pack side is never run by the client [crates/ac2-net/src/protocol/qualities.rs; crates/ac2-net/src/protocol/fx.rs]

### PHashTable<unsigned_long,FXNode>
- [x] `PHashTable<unsigned_long,FXNode>::StreamPack` · 004058e0 · legions 004055c0 — audit: covered: Client only unpacks; the pack side is never run by the client [crates/ac2-net/src/protocol/qualities.rs; crates/ac2-net/src/protocol/fx.rs]

### gmNoticeHandler
- [x] `gmNoticeHandler::RecvNotice_ViewCatalog` · 00405b40 — base virtual releases its arguments; the panel is the override: hud_shop.rs consignment_notice / catalog_notice via hud_player (frame_passes notices)

### FXData
- [-] `FXData::~FXData` · 00405c30 · legions 00405910 — ctor/dtor of plain data

## `portal/client/src/clienttriggers/clienttriggers.cpp.c` (3/4)

### ClientTrigger
- [x] `ClientTrigger::IsAborted` · 004c4a50 · legions 004b8e20 — ac2-script host/latent.rs wake_sleepers (named there): the due SleepAwakeners fire earliest first; a purged or cleared trigger is the woken sleep finding no process (resume_sleep None); the YesNoDialogAwakeners are the dialog answers (ac2-ui game_dialogs.rs)

### _STL::vector<GenericTriggerScheduler<ClientTrigger>::TriggerInfo,_STL::allocator<GenericTriggerScheduler<ClientTrigger>::TriggerInfo>_>
- [-] `_STL::vector<GenericTriggerScheduler<ClientTrigger>::TriggerInfo,_STL::allocator<GenericTriggerScheduler<ClientTrigger>::TriggerInfo>_>::operator=` · 004c4bd0 · legions 004b8fe0 — operator==/=/+/!= of plain data (string, id, error); Rust derives or uses String

### GenericTriggerScheduler<ClientTrigger>
- [x] `GenericTriggerScheduler<ClientTrigger>::ClearTriggers` · 004c4cc0 · legions 004b90d0 — ac2-script host/latent.rs wake_sleepers (named there): the due SleepAwakeners fire earliest first; a purged or cleared trigger is the woken sleep finding no process (resume_sleep None); the YesNoDialogAwakeners are the dialog answers (ac2-ui game_dialogs.rs)

### ClientTriggerScheduler
- [x] `ClientTriggerScheduler::PurgeEventsForInterface` · 004c4d90 · legions 004b91a0 — ac2-script host/latent.rs wake_sleepers (named there): the due SleepAwakeners fire earliest first; a purged or cleared trigger is the woken sleep finding no process (resume_sleep None); the YesNoDialogAwakeners are the dialog answers (ac2-ui game_dialogs.rs)

## `portal/client/src/device/win95/devavi.cpp.c` (0/5)

### DevAVI
- [-] `DevAVI::DrawNextFrame` · 007333e0 · legions 006af890 — not needed: Only caller chain is MD_Data_AVI::Update; media.rs says no layout on the DVD has an AVI media; intro (pictures) is skipped by AGENTS exception [crates/ac2-ui/src/media.rs:230; AGENTS.md:655]
- [-] `DevAVI::Play` · 00733480 · legions 006af930 — not needed: Only caller chain is MD_Data_AVI::Update; media.rs says no layout on the DVD has an AVI media; intro (pictures) is skipped by AGENTS exception [crates/ac2-ui/src/media.rs:230; AGENTS.md:655]
- [-] `DevAVI::Cleanup` · 007334f0 · legions 006af9a0 — not needed: Only caller chain is MD_Data_AVI::Update; media.rs says no layout on the DVD has an AVI media; intro (pictures) is skipped by AGENTS exception [crates/ac2-ui/src/media.rs:230; AGENTS.md:655]
- [-] `DevAVI::DevAVI` · 00733590 · legions 006afa40 — not needed: Only caller chain is MD_Data_AVI::Update; media.rs says no layout on the DVD has an AVI media; intro (pictures) is skipped by AGENTS exception [crates/ac2-ui/src/media.rs:230; AGENTS.md:655]
- [-] `DevAVI::~DevAVI` · 007337a0 · legions 006afc50 — not needed: Only caller chain is MD_Data_AVI::Update; media.rs says no layout on the DVD has an AVI media; intro (pictures) is skipped by AGENTS exception [crates/ac2-ui/src/media.rs:230; AGENTS.md:655]

## `portal/client/src/device/win95/wdevice.cpp.c` (5/26)

### RenderDevicePresentation
- [-] `RenderDevicePresentation::RenderDevicePresentation` · 00533d40 · legions 00517ce0 — ctor of plain data (presentation, event, dialog gateway)

### Device::DeviceDialogBoxGateway
- [-] `Device::DeviceDialogBoxGateway::ReportThread_OnDialogBoxBegin` · 00533d70 · legions 00517d10 — Win32 dialog-box thread marshalling; Rust draws its own dialogs
- [-] `Device::DeviceDialogBoxGateway::ReportThread_OnDialogBoxEnd` · 00533dc0 · legions 00517d60 — Win32 dialog-box thread marshalling; Rust draws its own dialogs
- [-] `Device::DeviceDialogBoxGateway::DeviceDialogBoxGateway` · 00533df0 · legions 00517d90 — ctor of plain data (presentation, event, dialog gateway)

### DialogBoxGateway
- [-] `DialogBoxGateway::DialogThread_OnDialogBoxBegin` · 00533e10 · legions 00517db0 — Win32 dialog-box thread marshalling; Rust draws its own dialogs

### Device
- [x] `Device::Done` · 00533e20 · legions 00517dc0 — crates/ac2-client/src/pregame.rs PregameOutcome::quit cites Device::Done (the done flag closes the client)
- [x] `Device::Unfocus` · 00533e30 · legions 00517dd0 — audit: covered: WindowEvent::Focused -> controls.focus releases input [crates/ac2-client/src/bin/ac2/main.rs:636]
- [x] `Device::LoadRenderingPreferences` · 00533e80 · legions 00517e20 — ac2-client window_presentation.rs: FullScreen, TripleBuffering and SyncToDisplayRefresh read from the bank into a WindowPresentation (from_bank, from_flags); tests in window_presentation.rs
- [-] `Device::ShutdownGraphicsEngine` · 00533f20 · legions 00517ec0 — D3D graphics-engine teardown; wgpu drop
- [-] `Device::ToggleFullScreenMode` · 00533f80 · legions 00517f20 — replaced by design: Borderless fullscreen via winit (apply_presentation); window resizes freely; exclusive display-mode change not wanted [crates/ac2-client/src/bin/ac2/main.rs:170; AGENTS.md:42]
- [-] `Device::GetPath` · 00533fe0 · legions 00517f80 — registry install path (Win32); Rust takes --dat-dir and the config directory
- [-] `Device::Cleanup` · 00534130 · legions 005180d0 — Win32/D3D device cleanup; Rust Drop
- [-] `Device::WndProc` · 00534190 — Win32 window procedure; winit delivers the events
- [-] `Device::DoEventLoop` · 00534810 · legions 005187b0 — Win32 message pump; winit event loop in ac2 main.rs
- [-] `Device::SendStringToClipboard` · 00534a50 · legions 005189f0 — replaced by design: arboard clipboard; AGENTS UI modern exceptions (text select + clipboard) [crates/ac2-client/src/clipboard.rs:2]
- [-] `Device::SetPath` · 00534b50 · legions 00518af0 — registry install path (Win32); Rust takes --dat-dir and the config directory
- [-] `Device::GetStringFromClipboard` · 00534cd0 — replaced by design: arboard clipboard; AGENTS UI modern exceptions (text select + clipboard) [crates/ac2-client/src/clipboard.rs:2]
- [-] `Device::ChangePresentation` · 00534de0 · legions 00518d80 — replaced by design: Borderless fullscreen via winit (apply_presentation); window resizes freely; exclusive display-mode change not wanted [crates/ac2-client/src/bin/ac2/main.rs:170; AGENTS.md:42]
- [-] `Device::Show16BitWarning` · 00534fc0 · legions 00518f60 — Win32 16-bit display warning dialog; not a modern display case
- [-] `Device::ForceDisplayResolution` · 005350a0 · legions 00518fa0 — replaced by design: Borderless fullscreen via winit (apply_presentation); window resizes freely; exclusive display-mode change not wanted [crates/ac2-client/src/bin/ac2/main.rs:170; AGENTS.md:42]
- [-] `Device::StartupGraphicsEngine` · 00535180 · legions 00519080 — replaced by design: winit window + wgpu device replace D3D init; --width/--height in cmdline.rs:234 [crates/ac2-client/src/bin/ac2/window_setup.rs; main.rs:480]
- [-] `Device::InitWindow` · 005352d0 · legions 005191d0 — Win32 window creation; winit
- [x] `Device::SaveScreenshot` · 005356e0 · legions 005195e0 — ac2-gpu screenshot.rs (named there): the first free capN.jpg of 100000, JPEG; the key in bin/ac2/keys.rs. The frame it captures differs (SmartBox::RenderScreenshot, partial: retail shot the world without the HUD)
- [-] `Device::RunConfigTool` · 00535aa0 · legions 005199a0 — launches retail's external config tool (ShellExecute); prefs edited in-client
- [x] `Device::InitializeRenderingPreferences` · 00535e20 · legions 00519d20 — ac2-render prefs::register_device (five display flags, section 0x4000000B) and register_display_resolution: the monitor's modes over 799x599, 1024x768 always listed and the default, each valued by nModeDesc (w<<16|h), labelled 'W x H'; the client registers it at window creation and sizes the window (AGENTS.md exception)
- [-] `Device::Init` · 005362f0 · legions 0051a1f0 — replaced by design: winit window + wgpu device replace D3D init; --width/--height in cmdline.rs:234 [crates/ac2-client/src/bin/ac2/window_setup.rs; main.rs:480]

## `portal/client/src/initcli/initcli.cpp.c` (2/6)

### InitCliCrypto
- [-] `InitCliCrypto::InitCliCrypto` · 00568a20 · legions 005389b0 — CryptoAPI provider handle plumbing and its error log; Rust crypto crates
- [-] `InitCliCrypto::GetLastErrorString` · 00568a30 · legions 005389c0 — CryptoAPI provider handle plumbing and its error log; Rust crypto crates
- [-] `InitCliCrypto::Log` · 00568b00 · legions 00538a90 — CryptoAPI provider handle plumbing and its error log; Rust crypto crates
- [x] `InitCliCrypto::Init` · 00568b70 · legions 00538b00 — audit: covered: CryptoAPI RC4 session key exported under the embedded 512-bit RSA key (SIMPLEBLOB); this is the real login crypto (not NetKeyExch) [crates/ac2-net/src/session_key.rs:1]
- [-] `InitCliCrypto::Delete` · 00568db0 · legions 00538d40 — CryptoAPI provider handle plumbing and its error log; Rust crypto crates
- [x] `InitCliCrypto::Init` · 00568dd0 · legions 00538d60 — audit: covered: CryptoAPI RC4 session key exported under the embedded 512-bit RSA key (SIMPLEBLOB); this is the real login crypto (not NetKeyExch) [crates/ac2-net/src/session_key.rs:1]

## `portal/client/src/main/client.cpp.c` (19/37)

### CChatEvent
- [x] `CChatEvent::CreateForSend` · 0071f6b0 · legions 0069ffb0

### Turbine::Client
- [-] `Turbine::Client::GetClientPtr` · 0071f730 · legions 006a0030 — singleton accessor; Rust owns the Client
- [-] `Turbine::Client::CleanupUI` · 0071f740 · legions 006a0040 — replaced by design: CloseRequested -> log_off (ClientNet::LogOffServer) -> exit; Drop of ownership trees replaces explicit cleanup order [crates/ac2-client/src/bin/ac2/main.rs:619,474]
- [-] `Turbine::Client::CleanupDatabase` · 0071f790 · legions 006a0090 — replaced by design: CloseRequested -> log_off (ClientNet::LogOffServer) -> exit; Drop of ownership trees replaces explicit cleanup order [crates/ac2-client/src/bin/ac2/main.rs:619,474]

### Turbine
- [-] `Turbine::portal_new_handler` · 0071f7b0 — allocator new-failure handler

### Turbine::Client
- [x] `Turbine::Client::InitAuth` · 0071f7d0 · legions 006a00c0 — crates/ac2-net/src/handshake.rs Credential::from_ticket cites Turbine::Client::InitAuth (auth type 1 with the account)
- [-] `Turbine::Client::InitUI` · 0071f810 · legions 006a0100 — replaced by design: Window resizes freely; UiScale replaces fixed 1024x768 device [crates/ac2-client/src/ui_scale.rs:3; AGENTS.md:42]
- [-] `Turbine::Client::SetUICallback` · 0071f940 · legions 006a0200 — replaced by design: Rust calls the UI directly [crates/ac2-client/src/client.rs:403]
- [-] `Turbine::Client::Run` · 0071f950 · legions 006a0210 — replaced by design: winit event loop replaces the retail frame loop [crates/ac2-client/src/bin/ac2/main.rs (ApplicationHandler)]
- [-] `Turbine::Client::OnDDDEvent` · 0071f990 · legions 006a0250 — not needed: Data-patch progress UI/events. Client answers the DAT interrogation (session.rs:561) but never downloads (data_patch.rs header: "the DATs never patch here"); supported servers serve data matching the mounted DATs. Revisit only if a server pushes DDD patches [crates/ac2-weenie/src/ddd.rs:
- [-] `Turbine::Client::FinishOutputText` · 0071f9f0 — replaced by design: Console/log text plumbing; debug-only output [crates/ac2-script/src/glue/wlog.rs:3]
- [-] `Turbine::Client::GetCharactersToWrapUsageTo` · 0071fa00 · legions 006a02b0 — console usage-text wrap width; Rust CLI help is generated
- [-] `Turbine::Client::GetLastCryptoError` · 0071fa10 · legions 006a02c0 — crypto error getter; Rust Result
- [x] `Turbine::Client::AddNetQueue` · 0071fa40 · legions 006a02f0
- [x] `Turbine::Client::ProcessControlEventQueue` · 0071fab0 · legions 006a0360
- [x] `Turbine::Client::ProcessLogonEventQueue` · 0071fb10 · legions 006a03c0

### Turbine::Client::ChatInterfaceProvider
- [x] `Turbine::Client::ChatInterfaceProvider::SendData` · 0071fba0 · legions 006a0450

### CharGenData
- [x] `CharGenData::SetFileName` · 0071fc00 · legions 006a04b0 — ac2-client chargen_data.rs CharGenCapture::new (the --chargen file; relative names under the AC2 config directory)

### ArgumentParser
- [-] `ArgumentParser::GetCharactersToWrapUsageTo` · 0071fc40 · legions 006a04f0 — console usage-text wrap width; Rust CLI help is generated

### Turbine::Client
- [x] `Turbine::Client::InitNet` · 0071fc50 · legions 006a0500 — audit: covered: Queues 1,2,4,5 routed by the session [crates/ac2-weenie/src/session.rs:553-585]
- [x] `Turbine::Client::SetLanguage` · 0071fde0 · legions 006a0640 — audit: covered:  [crates/ac2-client/src/bin/ac2/main.rs:763; crates/ac2-dat/src/paths.rs:13]
- [-] `Turbine::Client::~Client` · 0071fe30 · legions 006a0690 — destructor: Rust drop order in run()
- [-] `Turbine::Client::CleanupNet` · 00720090 · legions 006a08c0 — replaced by design: CloseRequested -> log_off (ClientNet::LogOffServer) -> exit; Drop of ownership trees replaces explicit cleanup order [crates/ac2-client/src/bin/ac2/main.rs:619,474]
- [x] `Turbine::Client::ParseArgs` · 007201b0 · legions 006a09e0 — cmdline.rs parse_args; ClientPrefs::load reads Preferences.ini before the frame loop (ac2 run())
- [x] `Turbine::Client::IsAlreadyRunning` · 00720230 · legions 006a0a60 — bin/ac2/instance.rs: exclusive lock on Turbine Client.lock in the AC2 config directory; a second client stops with a message (retail dialog text not known)
- [x] `Turbine::Client::Connect` · 007202b0 · legions 006a0ae0 — audit: covered: Auto-unique-port/interface prefs (0x40000070/71) only choose a local bind; not needed to log in [crates/ac2-net/src/net.rs:117]
- [x] `Turbine::Client::Disconnect` · 00720450 · legions 006a0c40 — audit: covered: log_off + Disconnected UI mode with ID_NetErr_ConnectionLost [crates/ac2-weenie/src/session.rs:937; crates/ac2-client/src/client.rs:85,981]
- [x] `Turbine::Client::BuildCommandLineArgs` · 007205d0 · legions 006a0db0 — audit: covered: tchatdll has no DLL to load: Turbine chat runs on the game connection (ac2-net protocol/turbine_chat.rs) [crates/ac2-client/src/bin/ac2/cmdline.rs:227]
- [x] `Turbine::Client::EvaluateCommandLineArg` · 007218f0 · legions 006a1eb0 — ported (bin/ac2/cmdline.rs, main.rs) from BuildCommandLineArgs (Legions 0x006A0DB0: each AddCmd type, field, default) and EvaluateCommandLineArg (0x006A1EB0): ff/sb/sw Preferences::SetBool(0x40000002, 0x4000000D/0x40000008/0x4000000C, 1), lb/nolb SetBool(0x40000004, 0x40000014, 1/0) into the session preferences (PrefsFile::set_bool); imeon (Client+0x68) or the preference (0x40000010, 0x40000077) is CIme::Init enable (g_UseIme), without which the input method never turns on; outport (Client+0x54, 1-65535 else Cli port must be between 1 and 65535) is the game socket local port (UdpTransport::bind_port). Accepted with no effect as in the release build: farview, frill, dropmip (values nothing reads), ls, refreshrate, regserver, fullscreen (Device::Init replaces Client+0x28 by the FullScreen preference). The debug/admin ones stay unported. Not traced: ConnectionOptions bAutoGenerateUniquePort (a busy outport fails the bind). Test the_device_and_network_options_follow_retail
- [x] `Turbine::Client::StartupTurbineChat` · 00722600 · legions 006a2a30 — audit: covered:  [crates/ac2-client/src/gun.rs:94; crates/ac2-weenie/src/gun/shared.rs:116]
- [-] `Turbine::Client::Client` · 00722720 · legions 006a2b30 — replaced by design: No Client object: Args, ClientPrefs, OnlineSession built in run() [crates/ac2-client/src/bin/ac2/main.rs]
- [-] `Turbine::Client::GetLanguage` · 00722900 · legions 006a2d10 — not needed: Preference-string path for the language id; --language is the only source here [crates/ac2-client/src/bin/ac2/cmdline.rs:39]
- [-] `Turbine::Client::AppendOutputText` · 00722a20 · legions 006a2e10 — replaced by design: Console/log text plumbing; debug-only output [crates/ac2-script/src/glue/wlog.rs:3]
- [-] `Turbine::Client::InitDatabase` · 00722a80 · legions 006a2e70 — replaced by design: DATs opened on demand by ac2-dat (DBOCache exception) [AGENTS.md:20; crates/ac2-client/src/bin/ac2/main.rs:763]

### GenericTriggerScheduler<ClientTrigger>
- [x] `GenericTriggerScheduler<ClientTrigger>::FireTriggers` · 00722b90 · legions 006a2f60 — ac2-script host/latent.rs wake_sleepers (named there): the due SleepAwakeners fire earliest first; a purged or cleared trigger is the woken sleep finding no process (resume_sleep None); the YesNoDialogAwakeners are the dialog answers (ac2-ui game_dialogs.rs)

### Turbine::Client
- [x] `Turbine::Client::Init` · 00722c90 · legions 006a3060 — audit: covered: Order not one-to-one (DAT open, prefs, keymap, session, window) but every step exists [crates/ac2-client/src/bin/ac2/main.rs:763]
- [x] `Turbine::Client::UseTime` · 00722f20 — audit: covered: gun, net, script clock, sleepers, db cache, input, commands, movement, smartbox; render/sound/UI run from the winit loop and client_audio/view [crates/ac2-client/src/client.rs:403-440]

## `portal/client/src/main/realclient.cpp.c` (0/3)

### RealClient
- [-] `RealClient::RealClient` · 007230f0 · legions 006a33b0 — replaced by design: KeepUIAlive = the frame keeps running while GUN waits (one iteration per frame) [crates/ac2-client/src/gun.rs:8,113]
- [-] `RealClient::~RealClient` · 00723110 · legions 006a33d0 — replaced by design: KeepUIAlive = the frame keeps running while GUN waits (one iteration per frame) [crates/ac2-client/src/gun.rs:8,113]
- [-] `RealClient::KeepUIAlive` · 00723160 · legions 006a3420 — replaced by design: KeepUIAlive = the frame keeps running while GUN waits (one iteration per frame) [crates/ac2-client/src/gun.rs:8,113]

## `portal/client/src/shell/cplayershell.cpp.c` (6/9)

### PStringBaseIter_Common<unsigned_short>
- [-] `PStringBaseIter_Common<unsigned_short>::FindSubStringIter` · 00732780 · legions 006aec20 — string search helper; Rust str/String find

### CPlayerShell
- [x] `CPlayerShell::ProcessCommand` · 007327d0 — ac2-weenie shell::process_text: the / prefix, ID_DefaultCommand speech, ExpandableText

### PStringBaseIter_Common<unsigned_short>
- [-] `PStringBaseIter_Common<unsigned_short>::FindSubString` · 00732810 · legions 006aecb0 — string search helper; Rust str/String find

### CPlayerShell
- [x] `CPlayerShell::CPlayerShell` · 00732940 · legions 006aede0 — ac2-weenie shell/mod.rs ChatCommands::new (named there): empty command and help tables, the / prefix in CPlayerShell::ProcessText, an empty CShell history (history.rs CommandHistory::default); recreated at the end of a session (ChatState::reset_shell)
- [x] `CPlayerShell::GetHelpText` · 007329d0 — audit: covered:  [crates/ac2-weenie/src/shell/handlers.rs:161; mod.rs:224]
- [x] `CPlayerShell::ExpandableText` · 00732a90 — ac2-weenie shell::process_text: the / prefix, ID_DefaultCommand speech, ExpandableText
- [x] `CPlayerShell::ProcessSpeech` · 00732c60 · legions 006af100 — ac2-weenie shell::process_text: the / prefix, ID_DefaultCommand speech, ExpandableText
- [-] `CPlayerShell::StreamPack` · 00732d50 · legions 006af1f0 — no caller packs a shell: CWAllocator (wallocator.cpp) only makes, resets and frees CPlayerShell, and nothing in the weenie/interface code packs it; the history and prefix do not persist (a new shell at every EndSession)
- [x] `CPlayerShell::ProcessText` · 00732da0 — ac2-weenie shell::process_text: the / prefix, ID_DefaultCommand speech, ExpandableText

## `portal/client/src/shell/cplayershellcmdtable.cpp.c` (1/2)

### CPlayerShellCmdTable
- [-] `CPlayerShellCmdTable::~CPlayerShellCmdTable` · 00732750 · legions 006aebf0 — ctor/dtor of plain data
- [x] `CPlayerShellCmdTable::CPlayerShellCmdTable` · 00732760 · legions 006aec00 — ac2-weenie shell/mod.rs ChatCommands::new (named there): empty command and help tables, the / prefix in CPlayerShell::ProcessText, an empty CShell history (history.rs CommandHistory::default); recreated at the end of a session (ChatState::reset_shell)

## `portal/shared/src/prefs/preference.cpp.c` (7/14)

### Preference
- [-] `Preference::Preference` · 006af6e0 · legions 00648930 — C++ constructor, destructor or singleton teardown: Rust ownership (the store and ClientPrefs drop; no global)
- [-] `Preference::~Preference` · 006af700 · legions 00648950 — C++ constructor, destructor or singleton teardown: Rust ownership (the store and ClientPrefs drop; no global)
- [-] `Preference::Compare` · 006af710 · legions 00648960 — derived Clone or PartialEq (the Rust types derive them), not a separate port
- [x] `Preference::Copy` · 006af770 · legions 006489c0 — ac2-prefs Pref::copy_from (value, default, range, choices, map)
- [-] `Preference::Describe` · 006af7a0 · legions 006489f0 — replaced by design: Option pages own their labels (ID_Sound_<name> etc.) [crates/ac2-ui/src/client_options_page.rs:22]
- [-] `Preference::Describe` · 006af7c0 · legions 00648a10 — replaced by design: Option pages own their labels (ID_Sound_<name> etc.) [crates/ac2-ui/src/client_options_page.rs:22]
- [-] `Preference::GetString` · 006af7e0 · legions 0044a270 — not a preference function (misattributed by the decompile: craft-skill text, a DiskConBase subset, or another class getter)

### DiskConBase
- [-] `DiskConBase::GetSubset` · 006af830 · legions 00648a80 — not a preference function (misattributed by the decompile: craft-skill text, a DiskConBase subset, or another class getter)

### Preference
- [x] `Preference::Create` · 006af840 · legions 00648a90 — ac2-prefs PreferenceBank::register: loaded from the file on first registration (Preference::Load), copied when the key exists
- [x] `Preference::Bless` · 006af890 · legions 00648ae0 — ac2-prefs PreferenceBank::register: loaded from the file on first registration (Preference::Load), copied when the key exists
- [x] `Preference::Identify` · 006af8d0 — ac2-prefs Pref: the section and key are fields (Pref::section, Pref::key)
- [x] `Preference::Identify` · 006af8f0 — ac2-prefs Pref: the section and key are fields (Pref::section, Pref::key)
- [x] `Preference::Save` · 006af940 · legions 00648b90 — ac2-prefs PreferenceBank::save: WritePrivateProfileString of each preference in list order (ini.rs set)
- [x] `Preference::Load` · 006afa50 · legions 00648ca0 — ac2-prefs bank load_into: the text of the file (or the default text) cut to 99 bytes, SetValue, then apply (the loaded value is the saved one)

## `portal/shared/src/prefs/preferencebank.cpp.c` (12/25)

### PreferenceBank
- [-] `PreferenceBank::GetInstance` · 006afba0 · legions 00648df0 — no global bank in the port: ClientPrefs owns the bank (retail keeps the mBank singleton)
- [x] `PreferenceBank::ChooseFile` · 006afbb0 · legions 00648e00 — ac2-prefs paths::resolve (PSUtils::full_path, relative to the working directory); --prefs names the file; ClientPrefs::load reads it before the frame loop
- [x] `PreferenceBank::Save` · 006afc10 · legions 00648e60 — ac2-prefs PreferenceBank::save: WritePrivateProfileString of each preference in list order (ini.rs set)
- [x] `PreferenceBank::Find` · 006afc50 · legions 00648ea0 — ac2-prefs PreferenceBank::find
- [-] `PreferenceBank::Synchronize` · 006afcb0 · legions 00648f00 — not needed: Unconditional apply form; nothing calls it; apply(section) applies changed ones [crates/ac2-prefs/src/bank.rs:131]
- [-] `PreferenceBank::ConnectSource` · 006afd00 · legions 00648f50 — replaced by design: Source/target copies and handler objects replaced by one value per key and submit/apply/recall/reload returning keys [crates/ac2-prefs/src/bank.rs:121-177; pref.rs:87]
- [-] `PreferenceBank::ConnectTarget` · 006afd50 · legions 00648fa0 — replaced by design: Source/target copies and handler objects replaced by one value per key and submit/apply/recall/reload returning keys [crates/ac2-prefs/src/bank.rs:121-177; pref.rs:87]
- [x] `PreferenceBank::List` · 006afda0 · legions 00648ff0 — ac2-prefs PreferenceBank::list
- [-] `PreferenceBank::ListHandlers` · 006afe20 · legions 00649070 — replaced by design: Source/target copies and handler objects replaced by one value per key and submit/apply/recall/reload returning keys [crates/ac2-prefs/src/bank.rs:121-177; pref.rs:87]
- [-] `PreferenceBank::Unlink` · 006aff40 · legions 00649190 — replaced by design: Source/target copies and handler objects replaced by one value per key and submit/apply/recall/reload returning keys [crates/ac2-prefs/src/bank.rs:121-177; pref.rs:87]
- [-] `PreferenceBank::Detach` · 006aff70 · legions 006491c0 — replaced by design: Source/target copies and handler objects replaced by one value per key and submit/apply/recall/reload returning keys [crates/ac2-prefs/src/bank.rs:121-177; pref.rs:87]
- [-] `PreferenceBank::~PreferenceBank` · 006affa0 · legions 006491f0 — C++ constructor, destructor or singleton teardown: Rust ownership (the store and ClientPrefs drop; no global)
- [x] `PreferenceBank::Link` · 006b0070 · legions 006492c0 — ac2-prefs PreferenceBank::link: records the owner subsystem of a key (SoundManager registers its keys)
- [-] `PreferenceBank::Change` · 006b00e0 · legions 00649330 — replaced by design: Source/target copies and handler objects replaced by one value per key and submit/apply/recall/reload returning keys [crates/ac2-prefs/src/bank.rs:121-177; pref.rs:87]
- [x] `PreferenceBank::Submit` · 006b0190 · legions 006493e0 — ac2-prefs PreferenceBank::submit (keys whose value changed)
- [x] `PreferenceBank::Recall` · 006b0300 · legions 00649550 — ac2-prefs PreferenceBank::recall (every preference of the section back to its applied value)
- [x] `PreferenceBank::Reload` · 006b0490 · legions 006496e0 — ac2-prefs PreferenceBank::reload (the keys of the section)
- [x] `PreferenceBank::Apply` · 006b0530 · legions 00649780 — ac2-prefs PreferenceBank::apply (section: the changed ones; key: one)
- [x] `PreferenceBank::Apply` · 006b0600 · legions 00649850 — ac2-prefs PreferenceBank::apply (section: the changed ones; key: one)
- [-] `PreferenceBank::Shutdown` · 006b0800 · legions 00649a50 — C++ constructor, destructor or singleton teardown: Rust ownership (the store and ClientPrefs drop; no global)
- [x] `PreferenceBank::Set` · 006b0830 · legions 00649a80 — ac2-prefs PreferenceBank::register: loaded from the file on first registration (Preference::Load), copied when the key exists
- [-] `PreferenceBank::Destroy` · 006b08e0 · legions 00649b30 — replaced by design: Source/target copies and handler objects replaced by one value per key and submit/apply/recall/reload returning keys [crates/ac2-prefs/src/bank.rs:121-177; pref.rs:87]
- [-] `PreferenceBank::Attach` · 006b0990 · legions 00649be0 — replaced by design: Source/target copies and handler objects replaced by one value per key and submit/apply/recall/reload returning keys [crates/ac2-prefs/src/bank.rs:121-177; pref.rs:87]
- [-] `PreferenceBank::PreferenceBank` · 006b0a00 · legions 00649c50 — C++ constructor, destructor or singleton teardown: Rust ownership (the store and ClientPrefs drop; no global)
- [x] `PreferenceBank::Startup` · 006b0ae0 · legions 00649d30 — ac2-prefs paths::resolve (PSUtils::full_path, relative to the working directory); --prefs names the file; ClientPrefs::load reads it before the frame loop

## `portal/shared/src/prefs/preferencebool.cpp.c` (12/18)

### PreferenceBool
- [-] `PreferenceBool::PreferenceBool` · 006b1680 · legions 0064a8d0 — C++ constructor, destructor or singleton teardown: Rust ownership (the store and ClientPrefs drop; no global)
- [-] `PreferenceBool::~PreferenceBool` · 006b16b0 · legions 0064a900 — C++ constructor, destructor or singleton teardown: Rust ownership (the store and ClientPrefs drop; no global)
- [-] `PreferenceBool::Clone` · 006b16c0 · legions 0064a910 — derived Clone or PartialEq (the Rust types derive them), not a separate port
- [-] `PreferenceBool::Compare` · 006b1710 — derived Clone or PartialEq (the Rust types derive them), not a separate port
- [x] `PreferenceBool::Copy` · 006b1760 · legions 0064a9b0 — ac2-prefs Pref::copy_from (value, default, range, choices, map)
- [-] `PreferenceBool::ConnectTarget` · 006b17c0 · legions 0064aa10 — replaced by design: Source/target copies and handler objects replaced by one value per key and submit/apply/recall/reload returning keys [crates/ac2-prefs/src/bank.rs:121-177; pref.rs:87]
- [x] `PreferenceBool::SetValue` · 006b1800 · legions 0064aa50 — ac2-prefs Pref::set_bool, set_default, changed, apply, recall, reset (the bool value and default)
- [x] `PreferenceBool::SetDefault` · 006b1820 · legions 0064aa70 — ac2-prefs Pref::set_bool, set_default, changed, apply, recall, reset (the bool value and default)
- [x] `PreferenceBool::Changed` · 006b1840 · legions 0064aa90 — ac2-prefs Pref::set_bool, set_default, changed, apply, recall, reset (the bool value and default)
- [x] `PreferenceBool::Apply` · 006b1860 · legions 0064aab0 — ac2-prefs Pref::set_bool, set_default, changed, apply, recall, reset (the bool value and default)
- [x] `PreferenceBool::Recall` · 006b1870 — ac2-prefs Pref::set_bool, set_default, changed, apply, recall, reset (the bool value and default)
- [x] `PreferenceBool::Reset` · 006b1880 — ac2-prefs Pref::set_bool, set_default, changed, apply, recall, reset (the bool value and default)

### AMHuman
- [-] `AMHuman::GetType` · 006b1890 — not a preference function (misattributed by the decompile: craft-skill text, a DiskConBase subset, or another class getter)

### PreferenceBool
- [x] `PreferenceBool::GetBool` · 006b18a0 · legions 0064aaf0 — audit: covered: Reads its own value [crates/ac2-prefs/src/pref.rs]
- [x] `PreferenceBool::SetValue` · 006b18f0 — ac2-prefs Pref::set_text for a bool: 0 or 1 in any case, other text keeps the value (SetValue(PString))
- [x] `PreferenceBool::SetDefault` · 006b19b0 — ac2-prefs Pref::set_default_text: a bool resets to its default even when the text names none
- [x] `PreferenceBool::InqValue` · 006b1a60 — ac2-prefs Pref::value_text for a bool: 1 or 0
- [x] `PreferenceBool::InqDefault` · 006b1aa0 — ac2-prefs Pref::default_text for a bool: 1 or 0

## `portal/shared/src/prefs/preferenceenum.cpp.c` (16/27)

### PreferenceEnum
- [-] `PreferenceEnum::Compare` · 006b0b10 — C++ constructor, destructor or singleton teardown: Rust ownership (the store and ClientPrefs drop; no global)
- [-] `PreferenceEnum::ConnectTarget` · 006b0b60 · legions 00649db0 — replaced by design: Source/target copies and handler objects replaced by one value per key and submit/apply/recall/reload returning keys [crates/ac2-prefs/src/bank.rs:121-177; pref.rs:87]
- [x] `PreferenceEnum::Map` · 006b0ba0 · legions 00649df0 — ac2-prefs enum: Pref::enumeration takes the map (EnumMapper of the names)
- [x] `PreferenceEnum::Changed` · 006b0bb0 · legions 00649e00 — ac2-prefs Pref::changed, apply, reset for an enum
- [x] `PreferenceEnum::Apply` · 006b0bd0 · legions 00649e20 — ac2-prefs Pref::changed, apply, reset for an enum
- [x] `PreferenceEnum::Reset` · 006b0be0 — ac2-prefs Pref::changed, apply, reset for an enum

### AnimAsset
- [-] `AnimAsset::GetType` · 006b0bf0 — not a preference function (misattributed by the decompile: craft-skill text, a DiskConBase subset, or another class getter)

### PreferenceEnum
- [x] `PreferenceEnum::GetInt` · 006b0c00 · legions 00649e50 — audit: covered: Reads its own value [crates/ac2-prefs/src/pref.rs]

### StoryHookData
- [-] `StoryHookData::GetTargetPKDamageChange` · 006b0c20 · legions 00649e70 — not a preference function (misattributed by the decompile: craft-skill text, a DiskConBase subset, or another class getter)

### PreferenceEnum
- [x] `PreferenceEnum::FindValue` · 006b0c70 · legions 00649ec0 — ac2-prefs Pref::parse for an enum name: its label, then the choice of that label (FindValue; no choice gives the default, through the not-found vtable call)
- [-] `PreferenceEnum::ClearChoices` · 006b0d30 · legions 00649f80 — replaced by design: Source/target copies and handler objects replaced by one value per key and submit/apply/recall/reload returning keys [crates/ac2-prefs/src/bank.rs:121-177; pref.rs:87]

### HashTable<unsigned_long,StringInfo>
- [-] `HashTable<unsigned_long,StringInfo>::operator=` · 006b0d50 · legions 00649fa0 — C++ constructor, destructor or singleton teardown: Rust ownership (the store and ClientPrefs drop; no global)

### PreferenceEnum
- [-] `PreferenceEnum::~PreferenceEnum` · 006b0f20 · legions 0064a170 — C++ constructor, destructor or singleton teardown: Rust ownership (the store and ClientPrefs drop; no global)
- [x] `PreferenceEnum::Copy` · 006b0fb0 · legions 0064a200 — ac2-prefs Pref::copy_from (value, default, range, choices, map)
- [x] `PreferenceEnum::SetValue` · 006b1040 — ac2-prefs Pref::set_text for an enum: an integer (is_int32, base 0) or a mapped name, that must be a choice
- [x] `PreferenceEnum::SetValue` · 006b1140 · legions 0064a390 — ac2-prefs Pref::set_choice (SetValue(long): a choice only)
- [x] `PreferenceEnum::SetDefault` · 006b11c0 — ac2-prefs Pref::set_default_text for an enum (a choice only)
- [x] `PreferenceEnum::SetDefault` · 006b12c0 · legions 0064a510 — ac2-prefs Pref::set_default (SetDefault(long): a choice only)
- [x] `PreferenceEnum::InqValue` · 006b1340 · legions 0064a590 — ac2-prefs Pref::value_text for an enum: the label name, else the integer in %d
- [x] `PreferenceEnum::InqDefault` · 006b13c0 — ac2-prefs Pref::default_text for an enum
- [-] `PreferenceEnum::AssignChoice` · 006b1430 — replaced by design: Source/target copies and handler objects replaced by one value per key and submit/apply/recall/reload returning keys [crates/ac2-prefs/src/bank.rs:121-177; pref.rs:87]
- [-] `PreferenceEnum::AssignChoice` · 006b1450 · legions 0064a6a0 — replaced by design: Source/target copies and handler objects replaced by one value per key and submit/apply/recall/reload returning keys [crates/ac2-prefs/src/bank.rs:121-177; pref.rs:87]
- [x] `PreferenceEnum::ExamineChoice` · 006b14a0 — ac2-prefs Pref::label_of and Pref::choices (ExamineChoice and ListChoices)
- [x] `PreferenceEnum::ExamineChoice` · 006b14c0 · legions 0064a710 — ac2-prefs Pref::label_of and Pref::choices (ExamineChoice and ListChoices)
- [x] `PreferenceEnum::ListChoices` · 006b1530 — ac2-prefs Pref::label_of and Pref::choices (ExamineChoice and ListChoices)
- [-] `PreferenceEnum::PreferenceEnum` · 006b15f0 · legions 0064a840 — C++ constructor, destructor or singleton teardown: Rust ownership (the store and ClientPrefs drop; no global)
- [-] `PreferenceEnum::Clone` · 006b1650 — derived Clone or PartialEq (the Rust types derive them), not a separate port

## `portal/shared/src/prefs/preferencefloat.cpp.c` (14/21)

### PreferenceFloat
- [-] `PreferenceFloat::PreferenceFloat` · 006b1ad0 · legions 0064ad20 — C++ constructor, destructor or singleton teardown: Rust ownership (the store and ClientPrefs drop; no global)
- [-] `PreferenceFloat::~PreferenceFloat` · 006b1b10 · legions 0064ad60 — C++ constructor, destructor or singleton teardown: Rust ownership (the store and ClientPrefs drop; no global)
- [-] `PreferenceFloat::Clone` · 006b1b20 — derived Clone or PartialEq (the Rust types derive them), not a separate port
- [-] `PreferenceFloat::Compare` · 006b1b80 — derived Clone or PartialEq (the Rust types derive them), not a separate port
- [x] `PreferenceFloat::Copy` · 006b1bf0 · legions 0064ae40 — ac2-prefs Pref::copy_from (value, default, range, choices, map)
- [-] `PreferenceFloat::ConnectTarget` · 006b1c60 · legions 0064aef0 — replaced by design: Source/target copies and handler objects replaced by one value per key and submit/apply/recall/reload returning keys [crates/ac2-prefs/src/bank.rs:121-177; pref.rs:87]
- [x] `PreferenceFloat::SetRange` · 006b1ca0 · legions 0064af30 — ac2-prefs Pref::set_range: refused when max is below min (no clamp elsewhere)
- [x] `PreferenceFloat::SetValue` · 006b1cd0 · legions 0064af60 — ac2-prefs Pref::set_float: stores what it is given (no clamp)
- [x] `PreferenceFloat::SetDefault` · 006b1cf0 · legions 0064af80 — ac2-prefs Pref::set_default (float default and value)
- [x] `PreferenceFloat::InqRange` · 006b1d10 · legions 0064afa0 — ac2-prefs Pref::set_range: refused when max is below min (no clamp elsewhere)
- [x] `PreferenceFloat::Changed` · 006b1d30 · legions 0064afc0 — ac2-prefs Pref::changed, apply, recall, reset (float, string)
- [x] `PreferenceFloat::Apply` · 006b1d50 · legions 0064afe0 — ac2-prefs Pref::changed, apply, recall, reset (float, string)
- [x] `PreferenceFloat::Recall` · 006b1d60 — ac2-prefs Pref::changed, apply, recall, reset (float, string)
- [x] `PreferenceFloat::Reset` · 006b1d70 — ac2-prefs Pref::changed, apply, recall, reset (float, string)

### AnimCube
- [-] `AnimCube::GetType` · 006b1d80 — not a preference function (misattributed by the decompile: craft-skill text, a DiskConBase subset, or another class getter)

### PreferenceFloat
- [x] `PreferenceFloat::GetFloat` · 006b1d90 · legions 0064b020 — audit: covered: Reads its own value [crates/ac2-prefs/src/pref.rs]

### MotionValues
- [-] `MotionValues::GetRunningTurnVel` · 006b1db0 · legions 0064b040 — not a preference function (misattributed by the decompile: craft-skill text, a DiskConBase subset, or another class getter)

### PreferenceFloat
- [x] `PreferenceFloat::SetValue` · 006b1df0 · legions 0064b080 — ac2-prefs Pref::set_text for a float: is_double then strtod, cast to f32
- [x] `PreferenceFloat::SetDefault` · 006b1e60 · legions 0064b0f0 — ac2-prefs Pref::set_default_text for a float (is_double first)
- [x] `PreferenceFloat::InqValue` · 006b1ec0 — ac2-prefs Pref::value_text for a float: %f (six decimals)
- [x] `PreferenceFloat::InqDefault` · 006b1f20 — ac2-prefs Pref::default_text for a float: %f

## `portal/shared/src/prefs/preferences.cpp.c` (10/13)

### Preferences
- [x] `Preferences::SetPreferencesFile` · 006ae650 · legions 006478e0 — ac2-prefs paths::resolve (PSUtils::full_path, relative to the working directory); --prefs names the file; ClientPrefs::load reads it before the frame loop
- [x] `Preferences::LoadValue_Internal` · 006ae6f0 · legions 00647980 — ac2-prefs store.rs and ini.rs: GetPrivateProfileString semantics (trimmed value, first match, comments skipped); section and key names come from the DAT (ac2-client DatNames)
- [x] `Preferences::LoadValue_Internal` · 006ae8c0 · legions 00647b50 — ac2-prefs store.rs and ini.rs: GetPrivateProfileString semantics (trimmed value, first match, comments skipped); section and key names come from the DAT (ac2-client DatNames)
- [x] `Preferences::LookupInHash_Internal` · 006ae9d0 · legions 00647c60 — ac2-prefs store.rs and ini.rs: GetPrivateProfileString semantics (trimmed value, first match, comments skipped); section and key names come from the DAT (ac2-client DatNames)
- [x] `Preferences::InqBool` · 006aebf0 · legions 00647e80 — ac2-prefs store.rs and ini.rs: GetPrivateProfileString semantics (trimmed value, first match, comments skipped); section and key names come from the DAT (ac2-client DatNames)
- [x] `Preferences::InqString` · 006aecd0 · legions 00647f60 — ac2-prefs store.rs and ini.rs: GetPrivateProfileString semantics (trimmed value, first match, comments skipped); section and key names come from the DAT (ac2-client DatNames)
- [x] `Preferences::StoreInHash_Internal` · 006aedc0 · legions 00648050 — ac2-prefs store.rs: the session cache of (section, key) text; SetBool and SetUInt32 are session-only, as retail (nothing is written to the file)
- [-] `Preferences::Preferences` · 006aeee0 · legions 00648170 — C++ constructor, destructor or singleton teardown: Rust ownership (the store and ClientPrefs drop; no global)
- [-] `Preferences::~Preferences` · 006aef40 · legions 006481d0 — C++ constructor, destructor or singleton teardown: Rust ownership (the store and ClientPrefs drop; no global)
- [-] `Preferences::Shutdown` · 006aeff0 · legions 00648280 — C++ constructor, destructor or singleton teardown: Rust ownership (the store and ClientPrefs drop; no global)
- [x] `Preferences::SetBool` · 006af060 · legions 006482f0 — ac2-prefs store.rs: the session cache of (section, key) text; SetBool and SetUInt32 are session-only, as retail (nothing is written to the file)
- [x] `Preferences::SetUInt32` · 006af0f0 · legions 00648380 — ac2-prefs store.rs: the session cache of (section, key) text; SetBool and SetUInt32 are session-only, as retail (nothing is written to the file)
- [x] `Preferences::Startup` · 006af180 · legions 00648410 — ac2-prefs paths::resolve (PSUtils::full_path, relative to the working directory); --prefs names the file; ClientPrefs::load reads it before the frame loop

## `portal/shared/src/prefs/preferencestring.cpp.c` (9/16)

### PreferenceString
- [x] `PreferenceString::Copy` · 006af1d0 · legions 00648460 — ac2-prefs Pref::copy_from (value, default, range, choices, map)

### PreferenceBool
- [-] `PreferenceBool::ConnectSource` · 006af230 — replaced by design: Source/target copies and handler objects replaced by one value per key and submit/apply/recall/reload returning keys [crates/ac2-prefs/src/bank.rs:121-177; pref.rs:87]

### PreferenceString
- [-] `PreferenceString::ConnectTarget` · 006af270 · legions 006484c0 — replaced by design: Source/target copies and handler objects replaced by one value per key and submit/apply/recall/reload returning keys [crates/ac2-prefs/src/bank.rs:121-177; pref.rs:87]
- [x] `PreferenceString::Recall` · 006af2c0 — ac2-prefs Pref::changed, apply, recall, reset (float, string)
- [x] `PreferenceString::Reset` · 006af2d0 — ac2-prefs Pref::changed, apply, recall, reset (float, string)

### CCellMesh
- [-] `CCellMesh::GetDBOType` · 006af2e0 — not a preference function (misattributed by the decompile: craft-skill text, a DiskConBase subset, or another class getter)

### PreferenceString
- [-] `PreferenceString::~PreferenceString` · 006af2f0 · legions 00648540 — C++ constructor, destructor or singleton teardown: Rust ownership (the store and ClientPrefs drop; no global)
- [-] `PreferenceString::Compare` · 006af360 — derived Clone or PartialEq (the Rust types derive them), not a separate port
- [x] `PreferenceString::SetValue` · 006af410 · legions 00648660 — ac2-prefs Pref::set_text for a string: the text as it is
- [x] `PreferenceString::SetDefault` · 006af470 · legions 006486c0 — ac2-prefs Pref::set_default_text for a string: the default and the value
- [x] `PreferenceString::InqValue` · 006af4f0 · legions 00648740 — ac2-prefs Pref::value_text for a string: the value as it is
- [x] `PreferenceString::InqDefault` · 006af550 · legions 006487a0 — ac2-prefs Pref::default_text for a string
- [x] `PreferenceString::Changed` · 006af5a0 — ac2-prefs Pref::changed, apply, recall, reset (float, string)
- [x] `PreferenceString::Apply` · 006af5c0 · legions 00648810 — ac2-prefs Pref::changed, apply, recall, reset (float, string)
- [-] `PreferenceString::PreferenceString` · 006af620 · legions 00648870 — C++ constructor, destructor or singleton teardown: Rust ownership (the store and ClientPrefs drop; no global)
- [-] `PreferenceString::Clone` · 006af6b0 — derived Clone or PartialEq (the Rust types derive them), not a separate port

## `portal/shared/src/shell/shell.cpp.c` (6/8)

### CShell
- [-] `CShell::StreamPack` · 00732ec0 · legions 006af360 — no caller packs a shell: CWAllocator (wallocator.cpp) only makes, resets and frees CPlayerShell, and nothing in the weenie/interface code packs it; the history and prefix do not persist (a new shell at every EndSession)
- [x] `CShell::InqHistoryItem` · 00732fc0 · legions 006af460 — ac2-weenie shell::CommandHistory (newest first; ResizeCmdHistory never trims, as retail's compare runs)
- [x] `CShell::InqNextHistoryItem` · 007330d0 · legions 006af570 — ac2-weenie shell::CommandHistory (newest first; ResizeCmdHistory never trims, as retail's compare runs)
- [x] `CShell::InqPrevHistoryItem` · 007330e0 · legions 006af580 — ac2-weenie shell::CommandHistory (newest first; ResizeCmdHistory never trims, as retail's compare runs)
- [x] `CShell::ResizeCmdHistory` · 007330f0 · legions 006af590 — ac2-weenie shell::CommandHistory (newest first; ResizeCmdHistory never trims, as retail's compare runs)
- [-] `CShell::~CShell` · 00733160 · legions 006af600 — ctor/dtor of plain data
- [x] `CShell::AddCmdToHistory` · 00733190 · legions 006af630 — ac2-weenie shell::CommandHistory (newest first; ResizeCmdHistory never trims, as retail's compare runs)
- [x] `CShell::CShell` · 00733230 · legions 006af6d0 — ac2-weenie shell::CommandHistory (newest first; ResizeCmdHistory never trims, as retail's compare runs)

## `portal/shared/src/shell/shellcmdtable.cpp.c` (8/16)

### AVL<unsigned_long,unsigned_long>
- [-] `AVL<unsigned_long,unsigned_long>::LeftRotate` · 00612a10 · legions 005d0ad0 — AVL container (rebalance, node, add); Rust ordered map
- [-] `AVL<unsigned_long,unsigned_long>::RightRotate` · 00612ae0 · legions 005d0ba0 — AVL container (rebalance, node, add); Rust ordered map
- [-] `AVL<unsigned_long,unsigned_long>::RightLeftRotate` · 00612bb0 · legions 004b9e00 — AVL container (rebalance, node, add); Rust ordered map
- [-] `AVL<unsigned_long,unsigned_long>::Rebalance` · 00612d10 · legions 005d0dd0 — AVL container (rebalance, node, add); Rust ordered map

### AVLNode<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>
- [-] `AVLNode<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>::AVLNode<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>` · 00612d80 · legions 005d0e40 — AVL container (rebalance, node, add); Rust ordered map
- [-] `AVLNode<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>::~AVLNode<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>` · 00612df0 · legions 005d0eb0 — AVL container (rebalance, node, add); Rust ordered map

### AVL<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>
- [-] `AVL<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>::Add` · 00612e50 · legions 005d0f10 — AVL container (rebalance, node, add); Rust ordered map

### ShellCmdTable
- [x] `ShellCmdTable::GetAllCmdHelpText` · 00613010 · legions 005d1080 — Script help words answered on the VM through CWInterface::DoHelpInterp (return slot read as a wpstring; list mode for the full list); native help keeps its word (AddHelpHandler adds only new words); full list formatted "  %s -- %s" (Legions 0x009EB834) in name order. Test help_texts_come_from_the_scripts_help_functions.
- [x] `ShellCmdTable::ProcessCommand` · 006132f0 · legions 005d1360 — ac2-weenie shell::process_command: quoted tokens, set-trim of the command, lowercased lookup; ExpandableCommand
- [x] `ShellCmdTable::GetCmdHelpText` · 00613710 · legions 005d1780 — Script help words answered on the VM through CWInterface::DoHelpInterp (return slot read as a wpstring; list mode for the full list); native help keeps its word (AddHelpHandler adds only new words); full list formatted "  %s -- %s" (Legions 0x009EB834) in name order. Test help_texts_come_from_the_scripts_help_functions.
- [x] `ShellCmdTable::ExpandableCommand` · 00613880 · legions 005d18f0 — ac2-weenie shell::process_command: quoted tokens, set-trim of the command, lowercased lookup; ExpandableCommand
- [-] `ShellCmdTable::~ShellCmdTable` · 00613c40 · legions 005d1cb0 — ctor/dtor of plain data
- [x] `ShellCmdTable::AddCmdHandler` · 00613cc0 · legions 005d1d30 — ac2-weenie shell/mod.rs ChatCommands::register (named there): lowered, the first handler kept, now also refusing a name holding a space. Test a_command_name_holding_a_space_is_refused_and_the_first_handler_stays
- [x] `ShellCmdTable::AddHelpHandler` · 00613e00 · legions 005d1e70 — ac2-weenie shell/mod.rs ChatCommands::set_help (named there): retail keyed a help function by command name; here every name of a handler shares its text
- [x] `ShellCmdTable::GetHelpText` · 00613f40 · legions 005d1fb0 — audit: covered:  [crates/ac2-weenie/src/shell/handlers.rs:161; mod.rs:224]
- [x] `ShellCmdTable::ShellCmdTable` · 00613f80 · legions 005d1ff0 — ac2-weenie shell/mod.rs ChatCommands::new (named there): empty command and help tables, the / prefix in CPlayerShell::ProcessText, an empty CShell history (history.rs CommandHistory::default); recreated at the end of a session (ChatState::reset_shell)
