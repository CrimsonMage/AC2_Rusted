# ac2-client: Client entry, shell, preferences, platform devices

354 retail functions: 125 done, 49 partial, 73 to do, 107 skipped.
Legend and workflow: [README](README.md).

## `ac2/client/src/main/billingmigration.cpp.c` (0/6)

### Listener
- [ ] `Listener::Listener` · 006f89c0 · legions 006788f0 — needs port: billing-migration check at logon (Listener and CheckForBillingMigration); no billing code in crates
- [ ] `Listener::OnError` · 006f89f0 — needs port: billing-migration check at logon (Listener and CheckForBillingMigration); no billing code in crates
- [ ] `Listener::Reset` · 006f8a00 — needs port: billing-migration check at logon (Listener and CheckForBillingMigration); no billing code in crates
- [ ] `Listener::OnResponse` · 006f8a10 — needs port: billing-migration check at logon (Listener and CheckForBillingMigration); no billing code in crates

### (free functions)
- [ ] `CheckForBillingMigrationInternal` · 006f8aa0 · legions 006789d0 — needs port: billing-migration check at logon (Listener and CheckForBillingMigration); no billing code in crates
- [ ] `CheckForBillingMigration` · 006f96e0 · legions 00679610 — needs port: billing-migration check at logon (Listener and CheckForBillingMigration); no billing code in crates

## `ac2/client/src/main/climain.cpp.c` (0/6)

### PackObj
- [ ] `PackObj::GetPackSize` · 00401000 · legions 00401000 — needs port (check): PackObj size helper; confirm the Rust packers size their output without a retained length field

### (free functions)
- [-] ``vector_constructor_iterator'` · 00401010 · legions 00401010 — STL vector construct/destruct iterator helpers (MSVC internals)
- [-] ``vector_destructor_iterator'` · 00401040 · legions 00401040 — STL vector construct/destruct iterator helpers (MSVC internals)

### ReferenceCountTemplate<268435456,0>
- [-] `ReferenceCountTemplate<268435456,0>::Release` · 004010a0 · legions 00401dc0 — reference-count release; Rust Arc/Drop

### CMostlyConsecutiveIntSet
- [-] `CMostlyConsecutiveIntSet::~CMostlyConsecutiveIntSet` · 004010f0 · legions 004010c0 — ctor/dtor of plain data

### (free functions)
- [-] `WinMain` · 004013a0 · legions 00401370 — WinMain: Win32 entry point, Rust main

## `ac2/client/src/main/gmclient.cpp.c` (26/92)

### PackObj
- [x] `PackObj::ALIGN_PTR` · 00401a60 · legions 00401980 — crates/ac2-script/src/unpack.rs `align` cites PackObj::ALIGN_PTR (4-byte alignment)

### NetError
- [-] `NetError::operator!=` · 00401aa0 · legions 004019c0 — operator==/=/+/!= of plain data (string, id, error); Rust derives or uses String
- [ ] `NetError::operator_StringInfo` · 00401ad0 · legions 004019f0 — needs port (check): NetError string and log text; implemented as NetError Display in crates/ac2-net but uncited
- [x] `NetError::GetPackSize` · 00401b20 · legions 00401a40
- [x] `NetError::Pack` · 00401b30 · legions 00401a50
- [x] `NetError::UnPack` · 00401b80 · legions 00401aa0

### ArgumentParser
- [ ] `ArgumentParser::OnCommandLineEvaluationDone` · 00401bd0 · legions 00401af0 — needs port (check): command-line evaluation; implemented as crates/ac2-client/src/bin/ac2/cmdline.rs but these overloads uncited

### Turbine::Client::ChatInterfaceProvider
- [x] `Turbine::Client::ChatInterfaceProvider::UseTime` · 00401be0 · legions 00401b00 — Legions body is empty / returns 0: the DVD client has no room-by-name result or per-frame chat work; rooms are set by server calls (shell/state.rs)
- [x] `Turbine::Client::ChatInterfaceProvider::QueryInterface` · 00401bf0 · legions 00401b10 — Legions returns E_NOINTERFACE: the port has no COM interface query

### gmNoticeHandler
- [x] `gmNoticeHandler::RecvNotice_Patron_SwearDone` · 00401c30 · legions 00401b70 — base virtual is empty; the allegiance page overrides it: ui_social.rs N::PatronSwearDone -> patron_swear_done (hud_player_notice)
- [x] `gmNoticeHandler::RecvNotice_AddMarker` · 00401c40 — base virtual is empty; the navigation UI overrides it: client_radar.rs radar_notice -> add_map_note / marker

### Turbine::Client::ChatInterfaceProvider
- [x] `Turbine::Client::ChatInterfaceProvider::OnSendToRoom` · 00401c50 · legions 00401b90 — ac2-weenie OnlineSession::send_to_room; the Legions vtable stub is empty (room sends go through gmChatInterfaceProvider::OnSendToRoom)
- [x] `Turbine::Client::ChatInterfaceProvider::OnEnterRoomByNameResult` · 00401c60 · legions 00401bc0 — Legions body is empty / returns 0: the DVD client has no room-by-name result or per-frame chat work; rooms are set by server calls (shell/state.rs)

### gmNoticeHandler
- [-] `gmNoticeHandler::RecvNotice_GUNResult_FriendStatus` · 00401c70 — GUN service result (GUN is gone); the friends list is not fed by a GUN answer here (ac2-weenie friends.rs)
- [x] `gmNoticeHandler::RecvNotice_InitStackUI` · 00401c80 · legions 00401be0 — base virtual is empty; the stack UI is set up by ShopStackCall::Init (hud_shop.rs shop_stack_calls)

### gmCharNameCharacterValidator
- [ ] `gmCharNameCharacterValidator::SetLanguage` · 00401c90 · legions 00401bf0 — needs port (check): gmCharNameCharacterValidator language; implemented as crates/ac2-ui/src/pregame/confirmation.rs (English only) but uncited
- [ ] `gmCharNameCharacterValidator::GetLanguage` · 00401ca0 · legions 00401c00 — needs port (check): gmCharNameCharacterValidator language; implemented as crates/ac2-ui/src/pregame/confirmation.rs (English only) but uncited

### IError
- [-] `IError::ReportGenericError` · 00401cb0 — Win32 message-box error reporter; Rust client reports through its own UI

### AppliedAppearanceKey
- [-] `AppliedAppearanceKey::~AppliedAppearanceKey` · 00401cd0 · legions 00401c10 — ctor/dtor of plain data

### StreamPackObj
- [ ] `StreamPackObj::Pack` · 00401cf0 — needs port (check): StreamPackObj Pack/UnPack base; implemented per type (crates/ac2-ui/src/layout_media.rs, crates/ac2-net) but uncited
- [ ] `StreamPackObj::UnPack` · 00401d30 · legions 00401ca0 — needs port (check): StreamPackObj Pack/UnPack base; implemented per type (crates/ac2-ui/src/layout_media.rs, crates/ac2-net) but uncited

### gmClient
- [ ] `gmClient::IsShuttingDown` · 00401d80 · legions 00401cf0 — needs port (check): gmClient::IsShuttingDown; check the shutdown path in crates/ac2-client/src/bin/ac2/main.rs

### PackObj
- [-] `PackObj::PackObj` · 00401d90 · legions 00401d00 — ctor/dtor of plain data

### gmClient
- [ ] `gmClient::InitWeenie` · 00401da0 · legions 00401d10 — needs port (check): gmClient::InitWeenie, weenie session setup at startup; uncited
- [ ] `gmClient::OnDDDEvent` · 00401dc0 · legions 00401d30 — needs port: DDD event handling (data-patch progress, OnDDDEvent); crates/ac2-ui/src/pregame/data_patch.rs says the patch is not modelled
- [~] `gmClient::RecvNotice_CharacterSet` · 00401e00 — CharacterSet.turbine_chat is decoded (ac2-net) but the port keeps Turbine chat on: the double sends 0 and room chat must still work
- [~] `gmClient::UseTime` · 00401e30 · legions 00401da0 — no TurbineGun (GUN) step

### PStringBase<char>
- [x] `PStringBase<char>::GetPackSize` · 00401e70 · legions 00401e10
- [-] `PStringBase<char>::operator==` · 00401ea0 · legions 00401e40 — operator==/=/+/!= of plain data (string, id, error); Rust derives or uses String

### PSmartArray<unsigned_long>
- [-] `PSmartArray<unsigned_long>::~PSmartArray<unsigned_long>` · 00401f00 · legions 00401ea0 — ctor/dtor of plain data

### Turbine::Client::ChatInterfaceProvider
- [-] `Turbine::Client::ChatInterfaceProvider::QueryInterface` · 00402170 — COM QueryInterface plumbing
- [-] `Turbine::Client::ChatInterfaceProvider::QueryInterface` · 00402180 — inline copy of 00402170

### CTBullet
- [-] `CTBullet::NumEventsHandled` · 00402190 · legions 00402130 — GUN bullet event counter; statistics only, no client behaviour
- [-] `CTBullet::IncEventsHandled` · 004021a0 · legions 00402140 — GUN bullet event counter; statistics only, no client behaviour

### gmClient::gmChatInterfaceProvider
- [-] `gmClient::gmChatInterfaceProvider::~gmChatInterfaceProvider` · 004021d0 · legions 00402170 — COM refcount and object lifetime: Rust ownership (CCliChatEvent is a GUN event class; GUN is gone)
- [~] `gmClient::gmChatInterfaceProvider::OnSendToRoomByIDResult` · 00402240 — admin-only forwarder of the async result; Legions answers through the chat response (CCliSendToRoomResult::HandleSendToRoomResult -> GameplayNotice::SendToRoomResult)

### PStringBase<unsigned_short>
- [-] `PStringBase<unsigned_short>::operator=` · 00402380 · legions 00402300 — operator==/=/+/!= of plain data (string, id, error); Rust derives or uses String

### StringInfo
- [-] `StringInfo::IsOnDisk` · 004025c0 · legions 00402540 — string-table cache flag; strings read on demand (AGENTS.md cache rules)

### gmClient
- [ ] `gmClient::~gmClient` · 00402650 · legions 004025a0 — needs port (check): client construction and teardown (gmClient ctor/dtor); Client::new and Drop in crates/ac2-client/src/client.rs uncited

### Turbine::Client
- [x] `Turbine::Client::GetWidth` · 00402830 · legions 00402780 — Args (account, host, port, width, height) and Args::server_address, the window size
- [x] `Turbine::Client::GetHeight` · 00402840 · legions 00402790 — Args (account, host, port, width, height) and Args::server_address, the window size
- [x] `Turbine::Client::GetAccount` · 00402850 · legions 004027a0 — Args (account, host, port, width, height) and Args::server_address, the window size
- [x] `Turbine::Client::GetHostName` · 00402860 · legions 004027b0 — Args (account, host, port, width, height) and Args::server_address, the window size
- [x] `Turbine::Client::GetPort` · 00402870 · legions 004027c0 — Args (account, host, port, width, height) and Args::server_address, the window size
- [x] `Turbine::Client::SetPortA` · 00402880 · legions 004027d0 — Args (account, host, port, width, height) and Args::server_address, the window size
- [x] `Turbine::Client::GetWorldName` · 00402890 · legions 004027e0 — Turbine::Client world name: set by the WORLD_NAME notice (Client::set_world_name), read by the efun and the assistance page
- [ ] `Turbine::Client::SetHostName` · 004028a0 · legions 004027f0 — needs port (check): Turbine::Client::SetHostName; implemented as crates/ac2-net/src/resolve.rs but uncited
- [~] `Turbine::Client::SetWorldName` · 004028f0 · legions 00402840 — the virtual slot +0x70 gmClient calls (Turbine::Client vtable 0x9c60a0) is this setter; the world-name path reaches the window title (bin/ac2 show_world_name)

### CCliChatEvent
- [-] `CCliChatEvent::CCliChatEvent` · 00402940 · legions 00402890 — COM refcount and object lifetime: Rust ownership (CCliChatEvent is a GUN event class; GUN is gone)

### gmClient
- [ ] `gmClient::RecvNotice_GunAuthenticated` · 004029d0 — needs port: GUN (MSN Gaming Zone) startup, authentication and notices; the live path uses glsticket (crates/ac2-prefs/src/gls_ticket.rs); decide whether GUN is kept
- [ ] `gmClient::AuthenticateWithGUN` · 00402ae0 · legions 00402a30 — needs port: GUN (MSN Gaming Zone) startup, authentication and notices; the live path uses glsticket (crates/ac2-prefs/src/gls_ticket.rs); decide whether GUN is kept

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
- [ ] `NetError::GetLogString` · 00403280 · legions 004031f0 — needs port (check): NetError string and log text; implemented as NetError Display in crates/ac2-net but uncited

### ArgumentParser::CommandLineArgList
- [ ] `ArgumentParser::CommandLineArgList::AddCmd` · 004032f0 · legions 00403260 — needs port (check): command-line evaluation; implemented as crates/ac2-client/src/bin/ac2/cmdline.rs but these overloads uncited

### NSTurbineGun::TurbineGun
- [ ] `NSTurbineGun::TurbineGun::SetWorldName` · 004033a0 · legions 00403310 — needs port: GUN (MSN Gaming Zone) startup, authentication and notices; the live path uses glsticket (crates/ac2-prefs/src/gls_ticket.rs); decide whether GUN is kept

### gmClient
- [ ] `gmClient::gmClient` · 004034a0 · legions 00403410 — needs port (check): client construction and teardown (gmClient ctor/dtor); Client::new and Drop in crates/ac2-client/src/client.rs uncited

### CCliConnectEvent
- [-] `CCliConnectEvent::CCliConnectEvent` · 004035c0 · legions 00403530 — ctor of plain data (presentation, event, dialog gateway)

### gmClient
- [ ] `gmClient::BuildCommandLineArgs` · 00403650 · legions 004035c0 — needs port (check): command-line evaluation; implemented as crates/ac2-client/src/bin/ac2/cmdline.rs but these overloads uncited
- [ ] `gmClient::EvaluateCommandLineArg` · 00403ae0 · legions 00403a50 — needs port (check): command-line evaluation; implemented as crates/ac2-client/src/bin/ac2/cmdline.rs but these overloads uncited
- [ ] `gmClient::OnCommandLineEvaluationDone` · 00403b60 · legions 00403ad0 — needs port (check): command-line evaluation; implemented as crates/ac2-client/src/bin/ac2/cmdline.rs but these overloads uncited
- [ ] `gmClient::RegisterForGunEvents` · 00403c10 · legions 00403b80 — needs port: GUN (MSN Gaming Zone) startup, authentication and notices; the live path uses glsticket (crates/ac2-prefs/src/gls_ticket.rs); decide whether GUN is kept
- [~] `gmClient::InitKeymap` · 00403d30 · legions 00403c80 — ac2-ui controls.rs: the DAT key maps only; saved .keymap files are not read
- [~] `gmClient::RecvNotice_WorldName` · 00403ff0 · legions 00403f40 — window title (main.rs) and the script copy (Client::set_world_name); the GUN copy (NSTurbineGun::SetWorldName) is not kept, no GUN service
- [ ] `gmClient::RecvNotice_GunReconnected` · 00404080 — needs port: GUN (MSN Gaming Zone) startup, authentication and notices; the live path uses glsticket (crates/ac2-prefs/src/gls_ticket.rs); decide whether GUN is kept

### gmClient::gmChatInterfaceProvider
- [x] `gmClient::gmChatInterfaceProvider::OnSendToRoom` · 00404100 · legions 00404050 — ac2-weenie OnlineSession::send_to_room / receive_chat over ac2-net turbine_chat (chatclient.dll's protocol)

### gmClient
- [x] `gmClient::CreateChatInterfaceProvider` · 00404190 · legions 004040e0 — the session is the chat provider (ac2-weenie OnlineSession send_to_room / receive_chat over ac2-net turbine_chat)

### PStringBase<char>
- [x] `PStringBase<char>::Pack` · 004042f0 · legions 00404240

### accountID
- [-] `accountID::operator=` · 004043d0 — operator==/=/+/!= of plain data (string, id, error); Rust derives or uses String

### Turbine::Client
- [ ] `Turbine::Client::SetAccount` · 004043f0 · legions 00404320 — needs port (check): Turbine::Client::SetAccount stores the account; implemented as cmdline.rs Args::account but uncited

### gmClient
- [~] `gmClient::Init` · 00404410 · legions 00404340 — gmClient::Init: same start-up as ac2 run(); see Turbine::Client::Init
- [ ] `gmClient::StartupGUN` · 00404a40 · legions 00404820 — needs port: GUN (MSN Gaming Zone) startup, authentication and notices; the live path uses glsticket (crates/ac2-prefs/src/gls_ticket.rs); decide whether GUN is kept
- [~] `gmClient::InitAuth` · 00404e10 · legions 00404bb0 — ac2-net Credential::from_ticket (empty -> Account, else Password); GUN ticket handler path elsewhere
- [~] `gmClient::Connect` · 00404f80 · legions 00404d20 — ac2-net connect to Args::server_address; the auto-unique-port and interface prefs (section 0x4000000F keys 0x40000070/71) are not read

### gmNoticeHandler
- [-] `gmNoticeHandler::RecvNotice_GUNResult_GetFriendsList` · 00405050 — GUN service result (GUN is gone); the friends list is not fed by a GUN answer here (ac2-weenie friends.rs)
- [~] `gmNoticeHandler::RecvNotice_FinishPlaySequence` · 00405210 — base virtual flushes the played list; the port sends no finish notice, the story slot ends its own sequence (client_story.rs)
- [x] `gmNoticeHandler::RecvNotice_PlaySequence` · 00405230 — base virtual flushes its list; the story slot takes the scenes: client_story.rs GameplayNotice::PlaySequence

### FXNode
- [-] `FXNode::~FXNode` · 00405250 · legions 00404ff0 — ctor/dtor of plain data

### gmNoticeHandler
- [x] `gmNoticeHandler::RecvNotice_UpdateConsignment` · 00405330 — base virtual releases its arguments; the panel is the override: hud_shop.rs consignment_notice / catalog_notice via hud_player_notice

### UIShop
- [-] `UIShop::~UIShop` · 00405340 · legions 004050e0 — ctor/dtor of plain data

### gmNoticeHandler
- [x] `gmNoticeHandler::RecvNotice_UpdateStore` · 004054b0 — base virtual releases its arguments; hud_shop.rs shop_notice -> recv_update_store (hud_player_notice)

### PHashTable<unsigned_long,unsigned_long>
- [~] `PHashTable<unsigned_long,unsigned_long>::StreamPack` · 00405500 — unpack only, in ac2-net protocol::qualities / visual (Legions CBaseQualities::UnPack instruction-identical)

### PHashTable<unsigned_long,FXNode>
- [~] `PHashTable<unsigned_long,FXNode>::StreamPack` · 004058e0 · legions 004055c0 — unpack only, in ac2-net protocol::fx / protocol::visual (Legions FXNode::StreamPack 0x00642410 instruction-identical)

### gmNoticeHandler
- [x] `gmNoticeHandler::RecvNotice_ViewCatalog` · 00405b40 — base virtual releases its arguments; the panel is the override: hud_shop.rs consignment_notice / catalog_notice via hud_player_notice

### FXData
- [-] `FXData::~FXData` · 00405c30 · legions 00405910 — ctor/dtor of plain data

## `portal/client/src/clienttriggers/clienttriggers.cpp.c` (0/4)

### ClientTrigger
- [ ] `ClientTrigger::IsAborted` · 004c4a50 · legions 004b8e20 — needs port: client trigger scheduler (delayed and recurring client events, purge per interface); no equivalent in crates

### _STL::vector<GenericTriggerScheduler<ClientTrigger>::TriggerInfo,_STL::allocator<GenericTriggerScheduler<ClientTrigger>::TriggerInfo>_>
- [-] `_STL::vector<GenericTriggerScheduler<ClientTrigger>::TriggerInfo,_STL::allocator<GenericTriggerScheduler<ClientTrigger>::TriggerInfo>_>::operator=` · 004c4bd0 · legions 004b8fe0 — operator==/=/+/!= of plain data (string, id, error); Rust derives or uses String

### GenericTriggerScheduler<ClientTrigger>
- [ ] `GenericTriggerScheduler<ClientTrigger>::ClearTriggers` · 004c4cc0 · legions 004b90d0 — needs port: client trigger scheduler (delayed and recurring client events, purge per interface); no equivalent in crates

### ClientTriggerScheduler
- [ ] `ClientTriggerScheduler::PurgeEventsForInterface` · 004c4d90 · legions 004b91a0 — needs port: client trigger scheduler (delayed and recurring client events, purge per interface); no equivalent in crates

## `portal/client/src/device/win95/devavi.cpp.c` (0/5)

### DevAVI
- [ ] `DevAVI::DrawNextFrame` · 007333e0 · legions 006af890 — needs port: intro AVI playback (DevAVI); crates/ac2-ui/src/media.rs skips Avi media
- [ ] `DevAVI::Play` · 00733480 · legions 006af930 — needs port: intro AVI playback (DevAVI); crates/ac2-ui/src/media.rs skips Avi media
- [ ] `DevAVI::Cleanup` · 007334f0 · legions 006af9a0 — needs port: intro AVI playback (DevAVI); crates/ac2-ui/src/media.rs skips Avi media
- [ ] `DevAVI::DevAVI` · 00733590 · legions 006afa40 — needs port: intro AVI playback (DevAVI); crates/ac2-ui/src/media.rs skips Avi media
- [ ] `DevAVI::~DevAVI` · 007337a0 · legions 006afc50 — needs port: intro AVI playback (DevAVI); crates/ac2-ui/src/media.rs skips Avi media

## `portal/client/src/device/win95/wdevice.cpp.c` (1/26)

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
- [ ] `Device::Unfocus` · 00533e30 · legions 00517dd0 — needs port (check): Device::Unfocus releases input on focus loss; implemented in crates/ac2-client/src/bin/ac2/main.rs (WindowEvent::Focused) but uncited
- [ ] `Device::LoadRenderingPreferences` · 00533e80 · legions 00517e20 — needs port (check): Device::LoadRenderingPreferences; crates/ac2-render/src/prefs.rs covers InitializeRenderingPreferences, this overload uncited
- [-] `Device::ShutdownGraphicsEngine` · 00533f20 · legions 00517ec0 — D3D graphics-engine teardown; wgpu drop
- [ ] `Device::ToggleFullScreenMode` · 00533f80 · legions 00517f20 — needs port (check): Device::ToggleFullScreenMode; main.rs OptionsEffect::Fullscreen sets borderless fullscreen but uncited
- [-] `Device::GetPath` · 00533fe0 · legions 00517f80 — registry install path (Win32); Rust takes --dat-dir and the config directory
- [-] `Device::Cleanup` · 00534130 · legions 005180d0 — Win32/D3D device cleanup; Rust Drop
- [-] `Device::WndProc` · 00534190 — Win32 window procedure; winit delivers the events
- [-] `Device::DoEventLoop` · 00534810 · legions 005187b0 — Win32 message pump; winit event loop in ac2 main.rs
- [~] `Device::SendStringToClipboard` · 00534a50 · legions 005189f0 — ac2-ui text_edit.rs / text_clip.rs / text.rs: drag and Shift selection, clipboard keys; not modelled: wheel on text (HandleMouseWheel), page up/down, IME, fallback fonts; clipboard is arboard, not CF_UNICODETEXT (ac2-client clipboard.rs)
- [-] `Device::SetPath` · 00534b50 · legions 00518af0 — registry install path (Win32); Rust takes --dat-dir and the config directory
- [~] `Device::GetStringFromClipboard` · 00534cd0 — ac2-ui text_edit.rs / text_clip.rs / text.rs: drag and Shift selection, clipboard keys; not modelled: wheel on text (HandleMouseWheel), page up/down, IME, fallback fonts; clipboard is arboard, not CF_UNICODETEXT (ac2-client clipboard.rs)
- [ ] `Device::ChangePresentation` · 00534de0 · legions 00518d80 — needs port: display mode change (ChangePresentation, ForceDisplayResolution); only borderless fullscreen exists
- [-] `Device::Show16BitWarning` · 00534fc0 · legions 00518f60 — Win32 16-bit display warning dialog; not a modern display case
- [ ] `Device::ForceDisplayResolution` · 005350a0 · legions 00518fa0 — needs port: display mode change (ChangePresentation, ForceDisplayResolution); only borderless fullscreen exists
- [ ] `Device::StartupGraphicsEngine` · 00535180 · legions 00519080 — needs port (check): Device::StartupGraphicsEngine; wgpu device creation in crates/ac2-gpu uncited
- [-] `Device::InitWindow` · 005352d0 · legions 005191d0 — Win32 window creation; winit
- [~] `Device::SaveScreenshot` · 005356e0 · legions 005195e0 — bin/ac2/screenshot.rs: copies the presented frame to the AC2 dir as capN.jpg (first free N, create_new); not re-rendered
- [-] `Device::RunConfigTool` · 00535aa0 · legions 005199a0 — launches retail's external config tool (ShellExecute); prefs edited in-client
- [~] `Device::InitializeRenderingPreferences` · 00535e20 · legions 00519d20 — ac2-render prefs::register_device: five display flags (section 0x4000000B); DisplayResolution (0x40000047, a float-free enum whose default comes from the device's display modes at run time) is not registered
- [ ] `Device::Init` · 005362f0 · legions 0051a1f0 — needs port (check): Device::Init window and size (--width/--height); main.rs window setup uncited

## `portal/client/src/initcli/initcli.cpp.c` (0/6)

### InitCliCrypto
- [-] `InitCliCrypto::InitCliCrypto` · 00568a20 · legions 005389b0 — CryptoAPI provider handle plumbing and its error log; Rust crypto crates
- [-] `InitCliCrypto::GetLastErrorString` · 00568a30 · legions 005389c0 — CryptoAPI provider handle plumbing and its error log; Rust crypto crates
- [-] `InitCliCrypto::Log` · 00568b00 · legions 00538a90 — CryptoAPI provider handle plumbing and its error log; Rust crypto crates
- [ ] `InitCliCrypto::Init` · 00568b70 · legions 00538b00 — needs port (check): InitCliCrypto::Init RSA key exchange; implemented in crates/ac2-net/src/session_key.rs but uncited
- [-] `InitCliCrypto::Delete` · 00568db0 · legions 00538d40 — CryptoAPI provider handle plumbing and its error log; Rust crypto crates
- [ ] `InitCliCrypto::Init` · 00568dd0 · legions 00538d60 — needs port (check): InitCliCrypto::Init RSA key exchange; implemented in crates/ac2-net/src/session_key.rs but uncited

## `portal/client/src/main/client.cpp.c` (9/37)

### CChatEvent
- [x] `CChatEvent::CreateForSend` · 0071f6b0 · legions 0069ffb0

### Turbine::Client
- [-] `Turbine::Client::GetClientPtr` · 0071f730 · legions 006a0030 — singleton accessor; Rust owns the Client
- [ ] `Turbine::Client::CleanupUI` · 0071f740 · legions 006a0040 — needs port (check): client shutdown order (CleanupUI, CleanupDatabase, CleanupNet); check the exit path in main.rs
- [ ] `Turbine::Client::CleanupDatabase` · 0071f790 · legions 006a0090 — needs port (check): client shutdown order (CleanupUI, CleanupDatabase, CleanupNet); check the exit path in main.rs

### Turbine
- [-] `Turbine::portal_new_handler` · 0071f7b0 — allocator new-failure handler

### Turbine::Client
- [x] `Turbine::Client::InitAuth` · 0071f7d0 · legions 006a00c0 — crates/ac2-net/src/handshake.rs Credential::from_ticket cites Turbine::Client::InitAuth (auth type 1 with the account)
- [ ] `Turbine::Client::InitUI` · 0071f810 · legions 006a0100 — needs port (check): Turbine::Client::InitUI (1024x768 logical window); crates/ac2-client/src/ui_scale.rs is uncited for this function
- [ ] `Turbine::Client::SetUICallback` · 0071f940 · legions 006a0200 — needs port (check): Turbine::Client::SetUICallback; Rust calls the UI directly, confirm no callback behaviour is lost
- [ ] `Turbine::Client::Run` · 0071f950 · legions 006a0210 — needs port (check): Turbine::Client::Run, the frame loop; main.rs frame loop uncited
- [ ] `Turbine::Client::OnDDDEvent` · 0071f990 · legions 006a0250 — needs port: DDD event handling (data-patch progress, OnDDDEvent); crates/ac2-ui/src/pregame/data_patch.rs says the patch is not modelled
- [ ] `Turbine::Client::FinishOutputText` · 0071f9f0 — needs port (check): text output to the log and console (FinishOutputText, AppendOutputText); wlog.rs cites ArgumentParser::FinishOutputText only
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
- [~] `Turbine::Client::InitNet` · 0071fc50 · legions 006a0500 — queues 1, 2, 4, 5 registered by ClientNet/OnlineSession routing
- [~] `Turbine::Client::SetLanguage` · 0071fde0 · legions 006a0640 — --language mounts local_<name>.dat: ac2_dat paths.rs local_dat_for_language, bin/ac2 run(); the stored string is not otherwise read
- [-] `Turbine::Client::~Client` · 0071fe30 · legions 006a0690 — destructor: Rust drop order in run()
- [ ] `Turbine::Client::CleanupNet` · 00720090 · legions 006a08c0 — needs port (check): client shutdown order (CleanupUI, CleanupDatabase, CleanupNet); check the exit path in main.rs
- [x] `Turbine::Client::ParseArgs` · 007201b0 · legions 006a09e0 — cmdline.rs parse_args; ClientPrefs::load reads Preferences.ini before the frame loop (ac2 run())
- [x] `Turbine::Client::IsAlreadyRunning` · 00720230 · legions 006a0a60 — bin/ac2/instance.rs: exclusive lock on Turbine Client.lock in the AC2 config directory; a second client stops with a message (retail dialog text not known)
- [~] `Turbine::Client::Connect` · 007202b0 · legions 006a0ae0 — ac2-net connect to Args::server_address; the auto-unique-port and interface prefs (section 0x4000000F keys 0x40000070/71) are not read
- [~] `Turbine::Client::Disconnect` · 00720450 · legions 006a0c40 — ac2-net NetClient::disconnect sends the logoff; leave order is session.rs (LogOffCharacter, CharExitGame, transport logoff); PacketController cleanup and error dialog not ported
- [~] `Turbine::Client::BuildCommandLineArgs` · 007205d0 · legions 006a0db0 — cmdline.rs option table (retail names and short forms); no launch-line builder or help text
- [~] `Turbine::Client::EvaluateCommandLineArg` · 007218f0 · legions 006a1eb0 — ac2 bin cmdline.rs: account (lowercased), host, port, width, height, glsticketdirect, glsticket (flag or FILE), rodat, language (kept, not applied), chargen, prefs; other retail options not ported
- [~] `Turbine::Client::StartupTurbineChat` · 00722600 · legions 006a2a30 — Legions: a vtable slot, no direct caller in the DVD client. The port has the Turbine chat service on the game connection from logon (no chatclient.dll load, no AC2ClientChatManager)
- [~] `Turbine::Client::Client` · 00722720 · legions 006a2b30 — no Client object: state is Args, ClientPrefs and OnlineSession built in run()
- [~] `Turbine::Client::GetLanguage` · 00722900 · legions 006a2d10 — --language's local DAT is mounted; the preference-string id path (GetEnum 0x4000002a) is not ported
- [ ] `Turbine::Client::AppendOutputText` · 00722a20 · legions 006a2e10 — needs port (check): text output to the log and console (FinishOutputText, AppendOutputText); wlog.rs cites ArgumentParser::FinishOutputText only
- [ ] `Turbine::Client::InitDatabase` · 00722a80 · legions 006a2e70 — needs port (check): Turbine::Client::InitDatabase; database startup uncited

### GenericTriggerScheduler<ClientTrigger>
- [ ] `GenericTriggerScheduler<ClientTrigger>::FireTriggers` · 00722b90 · legions 006a2f60 — needs port: client trigger scheduler (delayed and recurring client events, purge per interface); no equivalent in crates

### Turbine::Client
- [~] `Turbine::Client::Init` · 00722c90 · legions 006a3060 — ac2 run(): DAT open, prefs, keymap, session, window; retail order (InitAuth, InitUI, InitNet, InitDatabase) is not one-to-one
- [~] `Turbine::Client::UseTime` · 00722f20 — headless frame order in ac2-client Client::use_time; no render, sound, UI, triggers or DB cache step

## `portal/client/src/main/realclient.cpp.c` (0/3)

### RealClient
- [ ] `RealClient::RealClient` · 007230f0 · legions 006a33b0 — needs port (check): RealClient ctor, dtor and KeepUIAlive (UI kept alive during loads); client.rs and loader.rs uncited
- [ ] `RealClient::~RealClient` · 00723110 · legions 006a33d0 — needs port (check): RealClient ctor, dtor and KeepUIAlive (UI kept alive during loads); client.rs and loader.rs uncited
- [ ] `RealClient::KeepUIAlive` · 00723160 · legions 006a3420 — needs port (check): RealClient ctor, dtor and KeepUIAlive (UI kept alive during loads); client.rs and loader.rs uncited

## `portal/client/src/shell/cplayershell.cpp.c` (4/9)

### PStringBaseIter_Common<unsigned_short>
- [-] `PStringBaseIter_Common<unsigned_short>::FindSubStringIter` · 00732780 · legions 006aec20 — string search helper; Rust str/String find

### CPlayerShell
- [x] `CPlayerShell::ProcessCommand` · 007327d0 — ac2-weenie shell::process_text: the / prefix, ID_DefaultCommand speech, ExpandableText

### PStringBaseIter_Common<unsigned_short>
- [-] `PStringBaseIter_Common<unsigned_short>::FindSubString` · 00732810 · legions 006aecb0 — string search helper; Rust str/String find

### CPlayerShell
- [ ] `CPlayerShell::CPlayerShell` · 00732940 · legions 006aede0 — needs port (check): CPlayerShell and CPlayerShellCmdTable construction; crates/ac2-weenie/src/shell/mod.rs builds the table uncited
- [ ] `CPlayerShell::GetHelpText` · 007329d0 — needs port: command help text (GetHelpText, GetAllCmdHelpText, GetCmdHelpText); shell/mod.rs keeps a help map, uncited
- [x] `CPlayerShell::ExpandableText` · 00732a90 — ac2-weenie shell::process_text: the / prefix, ID_DefaultCommand speech, ExpandableText
- [x] `CPlayerShell::ProcessSpeech` · 00732c60 · legions 006af100 — ac2-weenie shell::process_text: the / prefix, ID_DefaultCommand speech, ExpandableText
- [ ] `CPlayerShell::StreamPack` · 00732d50 · legions 006af1f0 — needs port (check): CPlayerShell and CShell StreamPack (command-history state); shell/history.rs has the histories, StreamPack uncited
- [x] `CPlayerShell::ProcessText` · 00732da0 — ac2-weenie shell::process_text: the / prefix, ID_DefaultCommand speech, ExpandableText

## `portal/client/src/shell/cplayershellcmdtable.cpp.c` (0/2)

### CPlayerShellCmdTable
- [-] `CPlayerShellCmdTable::~CPlayerShellCmdTable` · 00732750 · legions 006aebf0 — ctor/dtor of plain data
- [ ] `CPlayerShellCmdTable::CPlayerShellCmdTable` · 00732760 · legions 006aec00 — needs port (check): CPlayerShell and CPlayerShellCmdTable construction; crates/ac2-weenie/src/shell/mod.rs builds the table uncited

## `portal/shared/src/prefs/preference.cpp.c` (7/14)

### Preference
- [-] `Preference::Preference` · 006af6e0 · legions 00648930 — C++ constructor, destructor or singleton teardown: Rust ownership (the store and ClientPrefs drop; no global)
- [-] `Preference::~Preference` · 006af700 · legions 00648950 — C++ constructor, destructor or singleton teardown: Rust ownership (the store and ClientPrefs drop; no global)
- [-] `Preference::Compare` · 006af710 · legions 00648960 — derived Clone or PartialEq (the Rust types derive them), not a separate port
- [x] `Preference::Copy` · 006af770 · legions 006489c0 — ac2-prefs Pref::copy_from (value, default, range, choices, map)
- [~] `Preference::Describe` · 006af7a0 · legions 006489f0 — display string ids are UI state (the option pages own their labels); ac2-prefs keeps none
- [~] `Preference::Describe` · 006af7c0 · legions 00648a10 — display string ids are UI state (the option pages own their labels); ac2-prefs keeps none
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
- [~] `PreferenceBank::Synchronize` · 006afcb0 · legions 00648f00 — applies every preference of the section (+0x58); the port apply(section) applies the changed ones, and nothing needs the unconditional form yet
- [~] `PreferenceBank::ConnectSource` · 006afd00 · legions 00648f50 — source and target copies are not modelled: the bank holds one value per key
- [~] `PreferenceBank::ConnectTarget` · 006afd50 · legions 00648fa0 — source and target copies are not modelled: the bank holds one value per key
- [x] `PreferenceBank::List` · 006afda0 · legions 00648ff0 — ac2-prefs PreferenceBank::list
- [~] `PreferenceBank::ListHandlers` · 006afe20 · legions 00649070 — handlers are not objects in the port: submit, apply, recall and reload return the keys and the caller dispatches them
- [~] `PreferenceBank::Unlink` · 006aff40 · legions 00649190 — handlers are not objects in the port: submit, apply, recall and reload return the keys and the caller dispatches them
- [~] `PreferenceBank::Detach` · 006aff70 · legions 006491c0 — handlers are not objects in the port: submit, apply, recall and reload return the keys and the caller dispatches them
- [-] `PreferenceBank::~PreferenceBank` · 006affa0 · legions 006491f0 — C++ constructor, destructor or singleton teardown: Rust ownership (the store and ClientPrefs drop; no global)
- [x] `PreferenceBank::Link` · 006b0070 · legions 006492c0 — ac2-prefs PreferenceBank::link: records the owner subsystem of a key (SoundManager registers its keys)
- [~] `PreferenceBank::Change` · 006b00e0 · legions 00649330 — changes notify the key handler and the section monitor; the port returns the keys (submit, apply, recall, reload) instead
- [x] `PreferenceBank::Submit` · 006b0190 · legions 006493e0 — ac2-prefs PreferenceBank::submit (keys whose value changed)
- [x] `PreferenceBank::Recall` · 006b0300 · legions 00649550 — ac2-prefs PreferenceBank::recall (every preference of the section back to its applied value)
- [x] `PreferenceBank::Reload` · 006b0490 · legions 006496e0 — ac2-prefs PreferenceBank::reload (the keys of the section)
- [x] `PreferenceBank::Apply` · 006b0530 · legions 00649780 — ac2-prefs PreferenceBank::apply (section: the changed ones; key: one)
- [x] `PreferenceBank::Apply` · 006b0600 · legions 00649850 — ac2-prefs PreferenceBank::apply (section: the changed ones; key: one)
- [-] `PreferenceBank::Shutdown` · 006b0800 · legions 00649a50 — C++ constructor, destructor or singleton teardown: Rust ownership (the store and ClientPrefs drop; no global)
- [x] `PreferenceBank::Set` · 006b0830 · legions 00649a80 — ac2-prefs PreferenceBank::register: loaded from the file on first registration (Preference::Load), copied when the key exists
- [~] `PreferenceBank::Destroy` · 006b08e0 · legions 00649b30 — no removal and no section monitors in the port: the bank keeps every preference registered this session
- [~] `PreferenceBank::Attach` · 006b0990 · legions 00649be0 — no removal and no section monitors in the port: the bank keeps every preference registered this session
- [-] `PreferenceBank::PreferenceBank` · 006b0a00 · legions 00649c50 — C++ constructor, destructor or singleton teardown: Rust ownership (the store and ClientPrefs drop; no global)
- [x] `PreferenceBank::Startup` · 006b0ae0 · legions 00649d30 — ac2-prefs paths::resolve (PSUtils::full_path, relative to the working directory); --prefs names the file; ClientPrefs::load reads it before the frame loop

## `portal/shared/src/prefs/preferencebool.cpp.c` (11/18)

### PreferenceBool
- [-] `PreferenceBool::PreferenceBool` · 006b1680 · legions 0064a8d0 — C++ constructor, destructor or singleton teardown: Rust ownership (the store and ClientPrefs drop; no global)
- [-] `PreferenceBool::~PreferenceBool` · 006b16b0 · legions 0064a900 — C++ constructor, destructor or singleton teardown: Rust ownership (the store and ClientPrefs drop; no global)
- [-] `PreferenceBool::Clone` · 006b16c0 · legions 0064a910 — derived Clone or PartialEq (the Rust types derive them), not a separate port
- [-] `PreferenceBool::Compare` · 006b1710 — derived Clone or PartialEq (the Rust types derive them), not a separate port
- [x] `PreferenceBool::Copy` · 006b1760 · legions 0064a9b0 — ac2-prefs Pref::copy_from (value, default, range, choices, map)
- [~] `PreferenceBool::ConnectTarget` · 006b17c0 · legions 0064aa10 — source and target copies are not modelled: the bank holds one value per key
- [x] `PreferenceBool::SetValue` · 006b1800 · legions 0064aa50 — ac2-prefs Pref::set_bool, set_default, changed, apply, recall, reset (the bool value and default)
- [x] `PreferenceBool::SetDefault` · 006b1820 · legions 0064aa70 — ac2-prefs Pref::set_bool, set_default, changed, apply, recall, reset (the bool value and default)
- [x] `PreferenceBool::Changed` · 006b1840 · legions 0064aa90 — ac2-prefs Pref::set_bool, set_default, changed, apply, recall, reset (the bool value and default)
- [x] `PreferenceBool::Apply` · 006b1860 · legions 0064aab0 — ac2-prefs Pref::set_bool, set_default, changed, apply, recall, reset (the bool value and default)
- [x] `PreferenceBool::Recall` · 006b1870 — ac2-prefs Pref::set_bool, set_default, changed, apply, recall, reset (the bool value and default)
- [x] `PreferenceBool::Reset` · 006b1880 — ac2-prefs Pref::set_bool, set_default, changed, apply, recall, reset (the bool value and default)

### AMHuman
- [-] `AMHuman::GetType` · 006b1890 — not a preference function (misattributed by the decompile: craft-skill text, a DiskConBase subset, or another class getter)

### PreferenceBool
- [~] `PreferenceBool::GetBool` · 006b18a0 · legions 0064aaf0 — reads its own value; the source copy is not modelled
- [x] `PreferenceBool::SetValue` · 006b18f0 — ac2-prefs Pref::set_text for a bool: 0 or 1 in any case, other text keeps the value (SetValue(PString))
- [x] `PreferenceBool::SetDefault` · 006b19b0 — ac2-prefs Pref::set_default_text: a bool resets to its default even when the text names none
- [x] `PreferenceBool::InqValue` · 006b1a60 — ac2-prefs Pref::value_text for a bool: 1 or 0
- [x] `PreferenceBool::InqDefault` · 006b1aa0 — ac2-prefs Pref::default_text for a bool: 1 or 0

## `portal/shared/src/prefs/preferenceenum.cpp.c` (15/27)

### PreferenceEnum
- [-] `PreferenceEnum::Compare` · 006b0b10 — C++ constructor, destructor or singleton teardown: Rust ownership (the store and ClientPrefs drop; no global)
- [~] `PreferenceEnum::ConnectTarget` · 006b0b60 · legions 00649db0 — source and target copies are not modelled: the bank holds one value per key
- [x] `PreferenceEnum::Map` · 006b0ba0 · legions 00649df0 — ac2-prefs enum: Pref::enumeration takes the map (EnumMapper of the names)
- [x] `PreferenceEnum::Changed` · 006b0bb0 · legions 00649e00 — ac2-prefs Pref::changed, apply, reset for an enum
- [x] `PreferenceEnum::Apply` · 006b0bd0 · legions 00649e20 — ac2-prefs Pref::changed, apply, reset for an enum
- [x] `PreferenceEnum::Reset` · 006b0be0 — ac2-prefs Pref::changed, apply, reset for an enum

### AnimAsset
- [-] `AnimAsset::GetType` · 006b0bf0 — not a preference function (misattributed by the decompile: craft-skill text, a DiskConBase subset, or another class getter)

### PreferenceEnum
- [~] `PreferenceEnum::GetInt` · 006b0c00 · legions 00649e50 — reads its own value; the source copy is not modelled

### StoryHookData
- [-] `StoryHookData::GetTargetPKDamageChange` · 006b0c20 · legions 00649e70 — not a preference function (misattributed by the decompile: craft-skill text, a DiskConBase subset, or another class getter)

### PreferenceEnum
- [x] `PreferenceEnum::FindValue` · 006b0c70 · legions 00649ec0 — ac2-prefs Pref::parse for an enum name: its label, then the choice of that label (FindValue; no choice gives the default, through the not-found vtable call)
- [~] `PreferenceEnum::ClearChoices` · 006b0d30 · legions 00649f80 — choices are given to Pref::enumeration; there is no separate AssignChoice call

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
- [~] `PreferenceEnum::AssignChoice` · 006b1430 — choices are given to Pref::enumeration; there is no separate AssignChoice call
- [~] `PreferenceEnum::AssignChoice` · 006b1450 · legions 0064a6a0 — choices are given to Pref::enumeration; there is no separate AssignChoice call
- [x] `PreferenceEnum::ExamineChoice` · 006b14a0 — ac2-prefs Pref::label_of and Pref::choices (ExamineChoice and ListChoices)
- [x] `PreferenceEnum::ExamineChoice` · 006b14c0 · legions 0064a710 — ac2-prefs Pref::label_of and Pref::choices (ExamineChoice and ListChoices)
- [x] `PreferenceEnum::ListChoices` · 006b1530 — ac2-prefs Pref::label_of and Pref::choices (ExamineChoice and ListChoices)
- [-] `PreferenceEnum::PreferenceEnum` · 006b15f0 · legions 0064a840 — C++ constructor, destructor or singleton teardown: Rust ownership (the store and ClientPrefs drop; no global)
- [-] `PreferenceEnum::Clone` · 006b1650 — derived Clone or PartialEq (the Rust types derive them), not a separate port

## `portal/shared/src/prefs/preferencefloat.cpp.c` (13/21)

### PreferenceFloat
- [-] `PreferenceFloat::PreferenceFloat` · 006b1ad0 · legions 0064ad20 — C++ constructor, destructor or singleton teardown: Rust ownership (the store and ClientPrefs drop; no global)
- [-] `PreferenceFloat::~PreferenceFloat` · 006b1b10 · legions 0064ad60 — C++ constructor, destructor or singleton teardown: Rust ownership (the store and ClientPrefs drop; no global)
- [-] `PreferenceFloat::Clone` · 006b1b20 — derived Clone or PartialEq (the Rust types derive them), not a separate port
- [-] `PreferenceFloat::Compare` · 006b1b80 — derived Clone or PartialEq (the Rust types derive them), not a separate port
- [x] `PreferenceFloat::Copy` · 006b1bf0 · legions 0064ae40 — ac2-prefs Pref::copy_from (value, default, range, choices, map)
- [~] `PreferenceFloat::ConnectTarget` · 006b1c60 · legions 0064aef0 — source and target copies are not modelled: the bank holds one value per key
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
- [~] `PreferenceFloat::GetFloat` · 006b1d90 · legions 0064b020 — reads its own value; the source copy is not modelled

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
- [~] `PreferenceBool::ConnectSource` · 006af230 — source and target copies are not modelled: the bank holds one value per key

### PreferenceString
- [~] `PreferenceString::ConnectTarget` · 006af270 · legions 006484c0 — source and target copies are not modelled: the bank holds one value per key
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
- [ ] `CShell::StreamPack` · 00732ec0 · legions 006af360 — needs port (check): CPlayerShell and CShell StreamPack (command-history state); shell/history.rs has the histories, StreamPack uncited
- [x] `CShell::InqHistoryItem` · 00732fc0 · legions 006af460 — ac2-weenie shell::CommandHistory (newest first; ResizeCmdHistory never trims, as retail's compare runs)
- [x] `CShell::InqNextHistoryItem` · 007330d0 · legions 006af570 — ac2-weenie shell::CommandHistory (newest first; ResizeCmdHistory never trims, as retail's compare runs)
- [x] `CShell::InqPrevHistoryItem` · 007330e0 · legions 006af580 — ac2-weenie shell::CommandHistory (newest first; ResizeCmdHistory never trims, as retail's compare runs)
- [x] `CShell::ResizeCmdHistory` · 007330f0 · legions 006af590 — ac2-weenie shell::CommandHistory (newest first; ResizeCmdHistory never trims, as retail's compare runs)
- [-] `CShell::~CShell` · 00733160 · legions 006af600 — ctor/dtor of plain data
- [x] `CShell::AddCmdToHistory` · 00733190 · legions 006af630 — ac2-weenie shell::CommandHistory (newest first; ResizeCmdHistory never trims, as retail's compare runs)
- [x] `CShell::CShell` · 00733230 · legions 006af6d0 — ac2-weenie shell::CommandHistory (newest first; ResizeCmdHistory never trims, as retail's compare runs)

## `portal/shared/src/shell/shellcmdtable.cpp.c` (2/16)

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
- [ ] `ShellCmdTable::GetAllCmdHelpText` · 00613010 · legions 005d1080 — needs port: command help text (GetHelpText, GetAllCmdHelpText, GetCmdHelpText); shell/mod.rs keeps a help map, uncited
- [x] `ShellCmdTable::ProcessCommand` · 006132f0 · legions 005d1360 — ac2-weenie shell::process_command: quoted tokens, set-trim of the command, lowercased lookup; ExpandableCommand
- [ ] `ShellCmdTable::GetCmdHelpText` · 00613710 · legions 005d1780 — needs port: command help text (GetHelpText, GetAllCmdHelpText, GetCmdHelpText); shell/mod.rs keeps a help map, uncited
- [x] `ShellCmdTable::ExpandableCommand` · 00613880 · legions 005d18f0 — ac2-weenie shell::process_command: quoted tokens, set-trim of the command, lowercased lookup; ExpandableCommand
- [-] `ShellCmdTable::~ShellCmdTable` · 00613c40 · legions 005d1cb0 — ctor/dtor of plain data
- [ ] `ShellCmdTable::AddCmdHandler` · 00613cc0 · legions 005d1d30 — needs port (check): ShellCmdTable AddCmdHandler and AddHelpHandler; shell/handlers.rs registers commands, uncited
- [ ] `ShellCmdTable::AddHelpHandler` · 00613e00 · legions 005d1e70 — needs port (check): ShellCmdTable AddCmdHandler and AddHelpHandler; shell/handlers.rs registers commands, uncited
- [ ] `ShellCmdTable::GetHelpText` · 00613f40 · legions 005d1fb0 — needs port: command help text (GetHelpText, GetAllCmdHelpText, GetCmdHelpText); shell/mod.rs keeps a help map, uncited
- [ ] `ShellCmdTable::ShellCmdTable` · 00613f80 · legions 005d1ff0 — needs port (check): ShellCmdTable constructor; shell/mod.rs builds the table uncited
