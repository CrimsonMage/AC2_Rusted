# ac2-client: Client entry, shell, preferences, platform devices

354 retail functions: 10 done, 5 partial, 334 to do, 5 skipped.
Legend and workflow: [README](README.md).

## `ac2/client/src/main/billingmigration.cpp.c` (0/6)

### Listener
- [ ] `Listener::Listener` · 006f89c0 · legions 006788f0
- [ ] `Listener::OnError` · 006f89f0
- [ ] `Listener::Reset` · 006f8a00
- [ ] `Listener::OnResponse` · 006f8a10

### (free functions)
- [ ] `CheckForBillingMigrationInternal` · 006f8aa0 · legions 006789d0
- [ ] `CheckForBillingMigration` · 006f96e0 · legions 00679610

## `ac2/client/src/main/climain.cpp.c` (0/6)

### PackObj
- [ ] `PackObj::GetPackSize` · 00401000 · legions 00401000

### (free functions)
- [ ] ``vector_constructor_iterator'` · 00401010 · legions 00401010
- [ ] ``vector_destructor_iterator'` · 00401040 · legions 00401040

### ReferenceCountTemplate<268435456,0>
- [ ] `ReferenceCountTemplate<268435456,0>::Release` · 004010a0 · legions 00401dc0

### CMostlyConsecutiveIntSet
- [ ] `CMostlyConsecutiveIntSet::~CMostlyConsecutiveIntSet` · 004010f0 · legions 004010c0

### (free functions)
- [ ] `WinMain` · 004013a0 · legions 00401370

## `ac2/client/src/main/gmclient.cpp.c` (5/92)

### PackObj
- [ ] `PackObj::ALIGN_PTR` · 00401a60 · legions 00401980

### NetError
- [ ] `NetError::operator!=` · 00401aa0 · legions 004019c0
- [ ] `NetError::operator_StringInfo` · 00401ad0 · legions 004019f0
- [x] `NetError::GetPackSize` · 00401b20 · legions 00401a40
- [x] `NetError::Pack` · 00401b30 · legions 00401a50
- [x] `NetError::UnPack` · 00401b80 · legions 00401aa0

### ArgumentParser
- [ ] `ArgumentParser::OnCommandLineEvaluationDone` · 00401bd0 · legions 00401af0

### Turbine::Client::ChatInterfaceProvider
- [ ] `Turbine::Client::ChatInterfaceProvider::UseTime` · 00401be0 · legions 00401b00
- [ ] `Turbine::Client::ChatInterfaceProvider::QueryInterface` · 00401bf0 · legions 00401b10

### gmNoticeHandler
- [ ] `gmNoticeHandler::RecvNotice_Patron_SwearDone` · 00401c30 · legions 00401b70
- [ ] `gmNoticeHandler::RecvNotice_AddMarker` · 00401c40

### Turbine::Client::ChatInterfaceProvider
- [ ] `Turbine::Client::ChatInterfaceProvider::OnSendToRoom` · 00401c50 · legions 00401b90
- [ ] `Turbine::Client::ChatInterfaceProvider::OnEnterRoomByNameResult` · 00401c60 · legions 00401bc0

### gmNoticeHandler
- [ ] `gmNoticeHandler::RecvNotice_GUNResult_FriendStatus` · 00401c70
- [ ] `gmNoticeHandler::RecvNotice_InitStackUI` · 00401c80 · legions 00401be0

### gmCharNameCharacterValidator
- [ ] `gmCharNameCharacterValidator::SetLanguage` · 00401c90 · legions 00401bf0
- [ ] `gmCharNameCharacterValidator::GetLanguage` · 00401ca0 · legions 00401c00

### IError
- [ ] `IError::ReportGenericError` · 00401cb0

### AppliedAppearanceKey
- [ ] `AppliedAppearanceKey::~AppliedAppearanceKey` · 00401cd0 · legions 00401c10

### StreamPackObj
- [ ] `StreamPackObj::Pack` · 00401cf0
- [ ] `StreamPackObj::UnPack` · 00401d30 · legions 00401ca0

### gmClient
- [ ] `gmClient::IsShuttingDown` · 00401d80 · legions 00401cf0

### PackObj
- [ ] `PackObj::PackObj` · 00401d90 · legions 00401d00

### gmClient
- [ ] `gmClient::InitWeenie` · 00401da0 · legions 00401d10
- [ ] `gmClient::OnDDDEvent` · 00401dc0 · legions 00401d30
- [ ] `gmClient::RecvNotice_CharacterSet` · 00401e00
- [~] `gmClient::UseTime` · 00401e30 · legions 00401da0 — no TurbineGun (GUN) step

### PStringBase<char>
- [x] `PStringBase<char>::GetPackSize` · 00401e70 · legions 00401e10
- [ ] `PStringBase<char>::operator==` · 00401ea0 · legions 00401e40

### PSmartArray<unsigned_long>
- [ ] `PSmartArray<unsigned_long>::~PSmartArray<unsigned_long>` · 00401f00 · legions 00401ea0

### Turbine::Client::ChatInterfaceProvider
- [ ] `Turbine::Client::ChatInterfaceProvider::QueryInterface` · 00402170
- [-] `Turbine::Client::ChatInterfaceProvider::QueryInterface` · 00402180 — inline copy of 00402170

### CTBullet
- [ ] `CTBullet::NumEventsHandled` · 00402190 · legions 00402130
- [ ] `CTBullet::IncEventsHandled` · 004021a0 · legions 00402140

### gmClient::gmChatInterfaceProvider
- [ ] `gmClient::gmChatInterfaceProvider::~gmChatInterfaceProvider` · 004021d0 · legions 00402170
- [ ] `gmClient::gmChatInterfaceProvider::OnSendToRoomByIDResult` · 00402240

### PStringBase<unsigned_short>
- [ ] `PStringBase<unsigned_short>::operator=` · 00402380 · legions 00402300

### StringInfo
- [ ] `StringInfo::IsOnDisk` · 004025c0 · legions 00402540

### gmClient
- [ ] `gmClient::~gmClient` · 00402650 · legions 004025a0

### Turbine::Client
- [ ] `Turbine::Client::GetWidth` · 00402830 · legions 00402780
- [ ] `Turbine::Client::GetHeight` · 00402840 · legions 00402790
- [ ] `Turbine::Client::GetAccount` · 00402850 · legions 004027a0
- [ ] `Turbine::Client::GetHostName` · 00402860 · legions 004027b0
- [ ] `Turbine::Client::GetPort` · 00402870 · legions 004027c0
- [ ] `Turbine::Client::SetPortA` · 00402880 · legions 004027d0
- [ ] `Turbine::Client::GetWorldName` · 00402890 · legions 004027e0
- [ ] `Turbine::Client::SetHostName` · 004028a0 · legions 004027f0
- [ ] `Turbine::Client::SetWorldName` · 004028f0 · legions 00402840

### CCliChatEvent
- [ ] `CCliChatEvent::CCliChatEvent` · 00402940 · legions 00402890

### gmClient
- [ ] `gmClient::RecvNotice_GunAuthenticated` · 004029d0
- [ ] `gmClient::AuthenticateWithGUN` · 00402ae0 · legions 00402a30

### RefCountIUnknown<gmClient::gmChatInterfaceProvider>
- [ ] `RefCountIUnknown<gmClient::gmChatInterfaceProvider>::RefCountIUnknown<gmClient::gmChatInterfaceProvider>` · 00402d00 · legions 00402c50
- [ ] `RefCountIUnknown<gmClient::gmChatInterfaceProvider>::AddRef` · 00402d70 · legions 00402cc0
- [ ] `RefCountIUnknown<gmClient::gmChatInterfaceProvider>::Release` · 00402d90 · legions 00402ce0
- [-] `RefCountIUnknown<gmClient::gmChatInterfaceProvider>::AddRef` · 00402dd0 — inline copy of 00402d70
- [-] `RefCountIUnknown<gmClient::gmChatInterfaceProvider>::Release` · 00402de0 — inline copy of 00402d90
- [-] `RefCountIUnknown<gmClient::gmChatInterfaceProvider>::AddRef` · 00402df0 — inline copy of 00402d70
- [-] `RefCountIUnknown<gmClient::gmChatInterfaceProvider>::Release` · 00402e00 — inline copy of 00402d90
- [ ] `RefCountIUnknown<gmClient::gmChatInterfaceProvider>::~RefCountIUnknown<gmClient::gmChatInterfaceProvider>` · 00402e40 · legions 00402d90

### PStringBase<unsigned_short>
- [ ] `PStringBase<unsigned_short>::operator+` · 00402ef0 · legions 00402e60

### NetError
- [ ] `NetError::GetLogString` · 00403280 · legions 004031f0

### ArgumentParser::CommandLineArgList
- [ ] `ArgumentParser::CommandLineArgList::AddCmd` · 004032f0 · legions 00403260

### NSTurbineGun::TurbineGun
- [ ] `NSTurbineGun::TurbineGun::SetWorldName` · 004033a0 · legions 00403310

### gmClient
- [ ] `gmClient::gmClient` · 004034a0 · legions 00403410

### CCliConnectEvent
- [ ] `CCliConnectEvent::CCliConnectEvent` · 004035c0 · legions 00403530

### gmClient
- [ ] `gmClient::BuildCommandLineArgs` · 00403650 · legions 004035c0
- [ ] `gmClient::EvaluateCommandLineArg` · 00403ae0 · legions 00403a50
- [ ] `gmClient::OnCommandLineEvaluationDone` · 00403b60 · legions 00403ad0
- [ ] `gmClient::RegisterForGunEvents` · 00403c10 · legions 00403b80
- [ ] `gmClient::InitKeymap` · 00403d30 · legions 00403c80
- [ ] `gmClient::RecvNotice_WorldName` · 00403ff0 · legions 00403f40
- [ ] `gmClient::RecvNotice_GunReconnected` · 00404080

### gmClient::gmChatInterfaceProvider
- [ ] `gmClient::gmChatInterfaceProvider::OnSendToRoom` · 00404100 · legions 00404050

### gmClient
- [ ] `gmClient::CreateChatInterfaceProvider` · 00404190 · legions 004040e0

### PStringBase<char>
- [x] `PStringBase<char>::Pack` · 004042f0 · legions 00404240

### accountID
- [ ] `accountID::operator=` · 004043d0

### Turbine::Client
- [ ] `Turbine::Client::SetAccount` · 004043f0 · legions 00404320

### gmClient
- [ ] `gmClient::Init` · 00404410 · legions 00404340
- [ ] `gmClient::StartupGUN` · 00404a40 · legions 00404820
- [ ] `gmClient::InitAuth` · 00404e10 · legions 00404bb0
- [ ] `gmClient::Connect` · 00404f80 · legions 00404d20

### gmNoticeHandler
- [ ] `gmNoticeHandler::RecvNotice_GUNResult_GetFriendsList` · 00405050
- [ ] `gmNoticeHandler::RecvNotice_FinishPlaySequence` · 00405210
- [ ] `gmNoticeHandler::RecvNotice_PlaySequence` · 00405230

### FXNode
- [ ] `FXNode::~FXNode` · 00405250 · legions 00404ff0

### gmNoticeHandler
- [ ] `gmNoticeHandler::RecvNotice_UpdateConsignment` · 00405330

### UIShop
- [ ] `UIShop::~UIShop` · 00405340 · legions 004050e0

### gmNoticeHandler
- [ ] `gmNoticeHandler::RecvNotice_UpdateStore` · 004054b0

### PHashTable<unsigned_long,unsigned_long>
- [~] `PHashTable<unsigned_long,unsigned_long>::StreamPack` · 00405500 — unpack only, in ac2-net protocol::qualities / visual (Legions CBaseQualities::UnPack instruction-identical)

### PHashTable<unsigned_long,FXNode>
- [~] `PHashTable<unsigned_long,FXNode>::StreamPack` · 004058e0 · legions 004055c0 — unpack only, in ac2-net protocol::fx / protocol::visual (Legions FXNode::StreamPack 0x00642410 instruction-identical)

### gmNoticeHandler
- [ ] `gmNoticeHandler::RecvNotice_ViewCatalog` · 00405b40

### FXData
- [ ] `FXData::~FXData` · 00405c30 · legions 00405910

## `portal/client/src/clienttriggers/clienttriggers.cpp.c` (0/4)

### ClientTrigger
- [ ] `ClientTrigger::IsAborted` · 004c4a50 · legions 004b8e20

### _STL::vector<GenericTriggerScheduler<ClientTrigger>::TriggerInfo,_STL::allocator<GenericTriggerScheduler<ClientTrigger>::TriggerInfo>_>
- [ ] `_STL::vector<GenericTriggerScheduler<ClientTrigger>::TriggerInfo,_STL::allocator<GenericTriggerScheduler<ClientTrigger>::TriggerInfo>_>::operator=` · 004c4bd0 · legions 004b8fe0

### GenericTriggerScheduler<ClientTrigger>
- [ ] `GenericTriggerScheduler<ClientTrigger>::ClearTriggers` · 004c4cc0 · legions 004b90d0

### ClientTriggerScheduler
- [ ] `ClientTriggerScheduler::PurgeEventsForInterface` · 004c4d90 · legions 004b91a0

## `portal/client/src/device/win95/devavi.cpp.c` (0/5)

### DevAVI
- [ ] `DevAVI::DrawNextFrame` · 007333e0 · legions 006af890
- [ ] `DevAVI::Play` · 00733480 · legions 006af930
- [ ] `DevAVI::Cleanup` · 007334f0 · legions 006af9a0
- [ ] `DevAVI::DevAVI` · 00733590 · legions 006afa40
- [ ] `DevAVI::~DevAVI` · 007337a0 · legions 006afc50

## `portal/client/src/device/win95/wdevice.cpp.c` (0/26)

### RenderDevicePresentation
- [ ] `RenderDevicePresentation::RenderDevicePresentation` · 00533d40 · legions 00517ce0

### Device::DeviceDialogBoxGateway
- [ ] `Device::DeviceDialogBoxGateway::ReportThread_OnDialogBoxBegin` · 00533d70 · legions 00517d10
- [ ] `Device::DeviceDialogBoxGateway::ReportThread_OnDialogBoxEnd` · 00533dc0 · legions 00517d60
- [ ] `Device::DeviceDialogBoxGateway::DeviceDialogBoxGateway` · 00533df0 · legions 00517d90

### DialogBoxGateway
- [ ] `DialogBoxGateway::DialogThread_OnDialogBoxBegin` · 00533e10 · legions 00517db0

### Device
- [ ] `Device::Done` · 00533e20 · legions 00517dc0
- [ ] `Device::Unfocus` · 00533e30 · legions 00517dd0
- [ ] `Device::LoadRenderingPreferences` · 00533e80 · legions 00517e20
- [ ] `Device::ShutdownGraphicsEngine` · 00533f20 · legions 00517ec0
- [ ] `Device::ToggleFullScreenMode` · 00533f80 · legions 00517f20
- [ ] `Device::GetPath` · 00533fe0 · legions 00517f80
- [ ] `Device::Cleanup` · 00534130 · legions 005180d0
- [ ] `Device::WndProc` · 00534190
- [ ] `Device::DoEventLoop` · 00534810 · legions 005187b0
- [ ] `Device::SendStringToClipboard` · 00534a50 · legions 005189f0
- [ ] `Device::SetPath` · 00534b50 · legions 00518af0
- [ ] `Device::GetStringFromClipboard` · 00534cd0
- [ ] `Device::ChangePresentation` · 00534de0 · legions 00518d80
- [ ] `Device::Show16BitWarning` · 00534fc0 · legions 00518f60
- [ ] `Device::ForceDisplayResolution` · 005350a0 · legions 00518fa0
- [ ] `Device::StartupGraphicsEngine` · 00535180 · legions 00519080
- [ ] `Device::InitWindow` · 005352d0 · legions 005191d0
- [ ] `Device::SaveScreenshot` · 005356e0 · legions 005195e0
- [ ] `Device::RunConfigTool` · 00535aa0 · legions 005199a0
- [ ] `Device::InitializeRenderingPreferences` · 00535e20 · legions 00519d20
- [ ] `Device::Init` · 005362f0 · legions 0051a1f0

## `portal/client/src/initcli/initcli.cpp.c` (0/6)

### InitCliCrypto
- [ ] `InitCliCrypto::InitCliCrypto` · 00568a20 · legions 005389b0
- [ ] `InitCliCrypto::GetLastErrorString` · 00568a30 · legions 005389c0
- [ ] `InitCliCrypto::Log` · 00568b00 · legions 00538a90
- [ ] `InitCliCrypto::Init` · 00568b70 · legions 00538b00
- [ ] `InitCliCrypto::Delete` · 00568db0 · legions 00538d40
- [ ] `InitCliCrypto::Init` · 00568dd0 · legions 00538d60

## `portal/client/src/main/client.cpp.c` (5/37)

### CChatEvent
- [x] `CChatEvent::CreateForSend` · 0071f6b0 · legions 0069ffb0

### Turbine::Client
- [ ] `Turbine::Client::GetClientPtr` · 0071f730 · legions 006a0030
- [ ] `Turbine::Client::CleanupUI` · 0071f740 · legions 006a0040
- [ ] `Turbine::Client::CleanupDatabase` · 0071f790 · legions 006a0090

### Turbine
- [ ] `Turbine::portal_new_handler` · 0071f7b0

### Turbine::Client
- [ ] `Turbine::Client::InitAuth` · 0071f7d0 · legions 006a00c0
- [ ] `Turbine::Client::InitUI` · 0071f810 · legions 006a0100
- [ ] `Turbine::Client::SetUICallback` · 0071f940 · legions 006a0200
- [ ] `Turbine::Client::Run` · 0071f950 · legions 006a0210
- [ ] `Turbine::Client::OnDDDEvent` · 0071f990 · legions 006a0250
- [ ] `Turbine::Client::FinishOutputText` · 0071f9f0
- [ ] `Turbine::Client::GetCharactersToWrapUsageTo` · 0071fa00 · legions 006a02b0
- [ ] `Turbine::Client::GetLastCryptoError` · 0071fa10 · legions 006a02c0
- [x] `Turbine::Client::AddNetQueue` · 0071fa40 · legions 006a02f0
- [x] `Turbine::Client::ProcessControlEventQueue` · 0071fab0 · legions 006a0360
- [x] `Turbine::Client::ProcessLogonEventQueue` · 0071fb10 · legions 006a03c0

### Turbine::Client::ChatInterfaceProvider
- [x] `Turbine::Client::ChatInterfaceProvider::SendData` · 0071fba0 · legions 006a0450

### CharGenData
- [ ] `CharGenData::SetFileName` · 0071fc00 · legions 006a04b0

### ArgumentParser
- [ ] `ArgumentParser::GetCharactersToWrapUsageTo` · 0071fc40 · legions 006a04f0

### Turbine::Client
- [~] `Turbine::Client::InitNet` · 0071fc50 · legions 006a0500 — queues 1, 2, 4, 5 registered by ClientNet/OnlineSession routing
- [ ] `Turbine::Client::SetLanguage` · 0071fde0 · legions 006a0640
- [ ] `Turbine::Client::~Client` · 0071fe30 · legions 006a0690
- [ ] `Turbine::Client::CleanupNet` · 00720090 · legions 006a08c0
- [ ] `Turbine::Client::ParseArgs` · 007201b0 · legions 006a09e0
- [ ] `Turbine::Client::IsAlreadyRunning` · 00720230 · legions 006a0a60
- [ ] `Turbine::Client::Connect` · 007202b0 · legions 006a0ae0
- [ ] `Turbine::Client::Disconnect` · 00720450 · legions 006a0c40
- [ ] `Turbine::Client::BuildCommandLineArgs` · 007205d0 · legions 006a0db0
- [ ] `Turbine::Client::EvaluateCommandLineArg` · 007218f0 · legions 006a1eb0
- [ ] `Turbine::Client::StartupTurbineChat` · 00722600 · legions 006a2a30
- [ ] `Turbine::Client::Client` · 00722720 · legions 006a2b30
- [ ] `Turbine::Client::GetLanguage` · 00722900 · legions 006a2d10
- [ ] `Turbine::Client::AppendOutputText` · 00722a20 · legions 006a2e10
- [ ] `Turbine::Client::InitDatabase` · 00722a80 · legions 006a2e70

### GenericTriggerScheduler<ClientTrigger>
- [ ] `GenericTriggerScheduler<ClientTrigger>::FireTriggers` · 00722b90 · legions 006a2f60

### Turbine::Client
- [ ] `Turbine::Client::Init` · 00722c90 · legions 006a3060
- [~] `Turbine::Client::UseTime` · 00722f20 — headless frame order in ac2-client Client::use_time; no render, sound, UI, triggers or DB cache step

## `portal/client/src/main/realclient.cpp.c` (0/3)

### RealClient
- [ ] `RealClient::RealClient` · 007230f0 · legions 006a33b0
- [ ] `RealClient::~RealClient` · 00723110 · legions 006a33d0
- [ ] `RealClient::KeepUIAlive` · 00723160 · legions 006a3420

## `portal/client/src/shell/cplayershell.cpp.c` (0/9)

### PStringBaseIter_Common<unsigned_short>
- [ ] `PStringBaseIter_Common<unsigned_short>::FindSubStringIter` · 00732780 · legions 006aec20

### CPlayerShell
- [ ] `CPlayerShell::ProcessCommand` · 007327d0

### PStringBaseIter_Common<unsigned_short>
- [ ] `PStringBaseIter_Common<unsigned_short>::FindSubString` · 00732810 · legions 006aecb0

### CPlayerShell
- [ ] `CPlayerShell::CPlayerShell` · 00732940 · legions 006aede0
- [ ] `CPlayerShell::GetHelpText` · 007329d0
- [ ] `CPlayerShell::ExpandableText` · 00732a90
- [ ] `CPlayerShell::ProcessSpeech` · 00732c60 · legions 006af100
- [ ] `CPlayerShell::StreamPack` · 00732d50 · legions 006af1f0
- [ ] `CPlayerShell::ProcessText` · 00732da0

## `portal/client/src/shell/cplayershellcmdtable.cpp.c` (0/2)

### CPlayerShellCmdTable
- [ ] `CPlayerShellCmdTable::~CPlayerShellCmdTable` · 00732750 · legions 006aebf0
- [ ] `CPlayerShellCmdTable::CPlayerShellCmdTable` · 00732760 · legions 006aec00

## `portal/shared/src/prefs/preference.cpp.c` (0/14)

### Preference
- [ ] `Preference::Preference` · 006af6e0 · legions 00648930
- [ ] `Preference::~Preference` · 006af700 · legions 00648950
- [ ] `Preference::Compare` · 006af710 · legions 00648960
- [ ] `Preference::Copy` · 006af770 · legions 006489c0
- [ ] `Preference::Describe` · 006af7a0 · legions 006489f0
- [ ] `Preference::Describe` · 006af7c0 · legions 00648a10
- [ ] `Preference::GetString` · 006af7e0 · legions 0044a270

### DiskConBase
- [ ] `DiskConBase::GetSubset` · 006af830 · legions 00648a80

### Preference
- [ ] `Preference::Create` · 006af840 · legions 00648a90
- [ ] `Preference::Bless` · 006af890 · legions 00648ae0
- [ ] `Preference::Identify` · 006af8d0
- [ ] `Preference::Identify` · 006af8f0
- [ ] `Preference::Save` · 006af940 · legions 00648b90
- [ ] `Preference::Load` · 006afa50 · legions 00648ca0

## `portal/shared/src/prefs/preferencebank.cpp.c` (0/25)

### PreferenceBank
- [ ] `PreferenceBank::GetInstance` · 006afba0 · legions 00648df0
- [ ] `PreferenceBank::ChooseFile` · 006afbb0 · legions 00648e00
- [ ] `PreferenceBank::Save` · 006afc10 · legions 00648e60
- [ ] `PreferenceBank::Find` · 006afc50 · legions 00648ea0
- [ ] `PreferenceBank::Synchronize` · 006afcb0 · legions 00648f00
- [ ] `PreferenceBank::ConnectSource` · 006afd00 · legions 00648f50
- [ ] `PreferenceBank::ConnectTarget` · 006afd50 · legions 00648fa0
- [ ] `PreferenceBank::List` · 006afda0 · legions 00648ff0
- [ ] `PreferenceBank::ListHandlers` · 006afe20 · legions 00649070
- [ ] `PreferenceBank::Unlink` · 006aff40 · legions 00649190
- [ ] `PreferenceBank::Detach` · 006aff70 · legions 006491c0
- [ ] `PreferenceBank::~PreferenceBank` · 006affa0 · legions 006491f0
- [ ] `PreferenceBank::Link` · 006b0070 · legions 006492c0
- [ ] `PreferenceBank::Change` · 006b00e0 · legions 00649330
- [ ] `PreferenceBank::Submit` · 006b0190 · legions 006493e0
- [ ] `PreferenceBank::Recall` · 006b0300 · legions 00649550
- [ ] `PreferenceBank::Reload` · 006b0490 · legions 006496e0
- [ ] `PreferenceBank::Apply` · 006b0530 · legions 00649780
- [ ] `PreferenceBank::Apply` · 006b0600 · legions 00649850
- [ ] `PreferenceBank::Shutdown` · 006b0800 · legions 00649a50
- [ ] `PreferenceBank::Set` · 006b0830 · legions 00649a80
- [ ] `PreferenceBank::Destroy` · 006b08e0 · legions 00649b30
- [ ] `PreferenceBank::Attach` · 006b0990 · legions 00649be0
- [ ] `PreferenceBank::PreferenceBank` · 006b0a00 · legions 00649c50
- [ ] `PreferenceBank::Startup` · 006b0ae0 · legions 00649d30

## `portal/shared/src/prefs/preferencebool.cpp.c` (0/18)

### PreferenceBool
- [ ] `PreferenceBool::PreferenceBool` · 006b1680 · legions 0064a8d0
- [ ] `PreferenceBool::~PreferenceBool` · 006b16b0 · legions 0064a900
- [ ] `PreferenceBool::Clone` · 006b16c0 · legions 0064a910
- [ ] `PreferenceBool::Compare` · 006b1710
- [ ] `PreferenceBool::Copy` · 006b1760 · legions 0064a9b0
- [ ] `PreferenceBool::ConnectTarget` · 006b17c0 · legions 0064aa10
- [ ] `PreferenceBool::SetValue` · 006b1800 · legions 0064aa50
- [ ] `PreferenceBool::SetDefault` · 006b1820 · legions 0064aa70
- [ ] `PreferenceBool::Changed` · 006b1840 · legions 0064aa90
- [ ] `PreferenceBool::Apply` · 006b1860 · legions 0064aab0
- [ ] `PreferenceBool::Recall` · 006b1870
- [ ] `PreferenceBool::Reset` · 006b1880

### AMHuman
- [ ] `AMHuman::GetType` · 006b1890

### PreferenceBool
- [ ] `PreferenceBool::GetBool` · 006b18a0 · legions 0064aaf0
- [ ] `PreferenceBool::SetValue` · 006b18f0
- [ ] `PreferenceBool::SetDefault` · 006b19b0
- [ ] `PreferenceBool::InqValue` · 006b1a60
- [ ] `PreferenceBool::InqDefault` · 006b1aa0

## `portal/shared/src/prefs/preferenceenum.cpp.c` (0/27)

### PreferenceEnum
- [ ] `PreferenceEnum::Compare` · 006b0b10
- [ ] `PreferenceEnum::ConnectTarget` · 006b0b60 · legions 00649db0
- [ ] `PreferenceEnum::Map` · 006b0ba0 · legions 00649df0
- [ ] `PreferenceEnum::Changed` · 006b0bb0 · legions 00649e00
- [ ] `PreferenceEnum::Apply` · 006b0bd0 · legions 00649e20
- [ ] `PreferenceEnum::Reset` · 006b0be0

### AnimAsset
- [ ] `AnimAsset::GetType` · 006b0bf0

### PreferenceEnum
- [ ] `PreferenceEnum::GetInt` · 006b0c00 · legions 00649e50

### StoryHookData
- [ ] `StoryHookData::GetTargetPKDamageChange` · 006b0c20 · legions 00649e70

### PreferenceEnum
- [ ] `PreferenceEnum::FindValue` · 006b0c70 · legions 00649ec0
- [ ] `PreferenceEnum::ClearChoices` · 006b0d30 · legions 00649f80

### HashTable<unsigned_long,StringInfo>
- [ ] `HashTable<unsigned_long,StringInfo>::operator=` · 006b0d50 · legions 00649fa0

### PreferenceEnum
- [ ] `PreferenceEnum::~PreferenceEnum` · 006b0f20 · legions 0064a170
- [ ] `PreferenceEnum::Copy` · 006b0fb0 · legions 0064a200
- [ ] `PreferenceEnum::SetValue` · 006b1040
- [ ] `PreferenceEnum::SetValue` · 006b1140 · legions 0064a390
- [ ] `PreferenceEnum::SetDefault` · 006b11c0
- [ ] `PreferenceEnum::SetDefault` · 006b12c0 · legions 0064a510
- [ ] `PreferenceEnum::InqValue` · 006b1340 · legions 0064a590
- [ ] `PreferenceEnum::InqDefault` · 006b13c0
- [ ] `PreferenceEnum::AssignChoice` · 006b1430
- [ ] `PreferenceEnum::AssignChoice` · 006b1450 · legions 0064a6a0
- [ ] `PreferenceEnum::ExamineChoice` · 006b14a0
- [ ] `PreferenceEnum::ExamineChoice` · 006b14c0 · legions 0064a710
- [ ] `PreferenceEnum::ListChoices` · 006b1530
- [ ] `PreferenceEnum::PreferenceEnum` · 006b15f0 · legions 0064a840
- [ ] `PreferenceEnum::Clone` · 006b1650

## `portal/shared/src/prefs/preferencefloat.cpp.c` (0/21)

### PreferenceFloat
- [ ] `PreferenceFloat::PreferenceFloat` · 006b1ad0 · legions 0064ad20
- [ ] `PreferenceFloat::~PreferenceFloat` · 006b1b10 · legions 0064ad60
- [ ] `PreferenceFloat::Clone` · 006b1b20
- [ ] `PreferenceFloat::Compare` · 006b1b80
- [ ] `PreferenceFloat::Copy` · 006b1bf0 · legions 0064ae40
- [ ] `PreferenceFloat::ConnectTarget` · 006b1c60 · legions 0064aef0
- [ ] `PreferenceFloat::SetRange` · 006b1ca0 · legions 0064af30
- [ ] `PreferenceFloat::SetValue` · 006b1cd0 · legions 0064af60
- [ ] `PreferenceFloat::SetDefault` · 006b1cf0 · legions 0064af80
- [ ] `PreferenceFloat::InqRange` · 006b1d10 · legions 0064afa0
- [ ] `PreferenceFloat::Changed` · 006b1d30 · legions 0064afc0
- [ ] `PreferenceFloat::Apply` · 006b1d50 · legions 0064afe0
- [ ] `PreferenceFloat::Recall` · 006b1d60
- [ ] `PreferenceFloat::Reset` · 006b1d70

### AnimCube
- [ ] `AnimCube::GetType` · 006b1d80

### PreferenceFloat
- [ ] `PreferenceFloat::GetFloat` · 006b1d90 · legions 0064b020

### MotionValues
- [ ] `MotionValues::GetRunningTurnVel` · 006b1db0 · legions 0064b040

### PreferenceFloat
- [ ] `PreferenceFloat::SetValue` · 006b1df0 · legions 0064b080
- [ ] `PreferenceFloat::SetDefault` · 006b1e60 · legions 0064b0f0
- [ ] `PreferenceFloat::InqValue` · 006b1ec0
- [ ] `PreferenceFloat::InqDefault` · 006b1f20

## `portal/shared/src/prefs/preferences.cpp.c` (0/13)

### Preferences
- [ ] `Preferences::SetPreferencesFile` · 006ae650 · legions 006478e0
- [ ] `Preferences::LoadValue_Internal` · 006ae6f0 · legions 00647980
- [ ] `Preferences::LoadValue_Internal` · 006ae8c0 · legions 00647b50
- [ ] `Preferences::LookupInHash_Internal` · 006ae9d0 · legions 00647c60
- [ ] `Preferences::InqBool` · 006aebf0 · legions 00647e80
- [ ] `Preferences::InqString` · 006aecd0 · legions 00647f60
- [ ] `Preferences::StoreInHash_Internal` · 006aedc0 · legions 00648050
- [ ] `Preferences::Preferences` · 006aeee0 · legions 00648170
- [ ] `Preferences::~Preferences` · 006aef40 · legions 006481d0
- [ ] `Preferences::Shutdown` · 006aeff0 · legions 00648280
- [ ] `Preferences::SetBool` · 006af060 · legions 006482f0
- [ ] `Preferences::SetUInt32` · 006af0f0 · legions 00648380
- [ ] `Preferences::Startup` · 006af180 · legions 00648410

## `portal/shared/src/prefs/preferencestring.cpp.c` (0/16)

### PreferenceString
- [ ] `PreferenceString::Copy` · 006af1d0 · legions 00648460

### PreferenceBool
- [ ] `PreferenceBool::ConnectSource` · 006af230

### PreferenceString
- [ ] `PreferenceString::ConnectTarget` · 006af270 · legions 006484c0
- [ ] `PreferenceString::Recall` · 006af2c0
- [ ] `PreferenceString::Reset` · 006af2d0

### CCellMesh
- [ ] `CCellMesh::GetDBOType` · 006af2e0

### PreferenceString
- [ ] `PreferenceString::~PreferenceString` · 006af2f0 · legions 00648540
- [ ] `PreferenceString::Compare` · 006af360
- [ ] `PreferenceString::SetValue` · 006af410 · legions 00648660
- [ ] `PreferenceString::SetDefault` · 006af470 · legions 006486c0
- [ ] `PreferenceString::InqValue` · 006af4f0 · legions 00648740
- [ ] `PreferenceString::InqDefault` · 006af550 · legions 006487a0
- [ ] `PreferenceString::Changed` · 006af5a0
- [ ] `PreferenceString::Apply` · 006af5c0 · legions 00648810
- [ ] `PreferenceString::PreferenceString` · 006af620 · legions 00648870
- [ ] `PreferenceString::Clone` · 006af6b0

## `portal/shared/src/shell/shell.cpp.c` (0/8)

### CShell
- [ ] `CShell::StreamPack` · 00732ec0 · legions 006af360
- [ ] `CShell::InqHistoryItem` · 00732fc0 · legions 006af460
- [ ] `CShell::InqNextHistoryItem` · 007330d0 · legions 006af570
- [ ] `CShell::InqPrevHistoryItem` · 007330e0 · legions 006af580
- [ ] `CShell::ResizeCmdHistory` · 007330f0 · legions 006af590
- [ ] `CShell::~CShell` · 00733160 · legions 006af600
- [ ] `CShell::AddCmdToHistory` · 00733190 · legions 006af630
- [ ] `CShell::CShell` · 00733230 · legions 006af6d0

## `portal/shared/src/shell/shellcmdtable.cpp.c` (0/16)

### AVL<unsigned_long,unsigned_long>
- [ ] `AVL<unsigned_long,unsigned_long>::LeftRotate` · 00612a10 · legions 005d0ad0
- [ ] `AVL<unsigned_long,unsigned_long>::RightRotate` · 00612ae0 · legions 005d0ba0
- [ ] `AVL<unsigned_long,unsigned_long>::RightLeftRotate` · 00612bb0 · legions 004b9e00
- [ ] `AVL<unsigned_long,unsigned_long>::Rebalance` · 00612d10 · legions 005d0dd0

### AVLNode<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>
- [ ] `AVLNode<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>::AVLNode<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>` · 00612d80 · legions 005d0e40
- [ ] `AVLNode<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>::~AVLNode<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>` · 00612df0 · legions 005d0eb0

### AVL<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>
- [ ] `AVL<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>::Add` · 00612e50 · legions 005d0f10

### ShellCmdTable
- [ ] `ShellCmdTable::GetAllCmdHelpText` · 00613010 · legions 005d1080
- [ ] `ShellCmdTable::ProcessCommand` · 006132f0 · legions 005d1360
- [ ] `ShellCmdTable::GetCmdHelpText` · 00613710 · legions 005d1780
- [ ] `ShellCmdTable::ExpandableCommand` · 00613880 · legions 005d18f0
- [ ] `ShellCmdTable::~ShellCmdTable` · 00613c40 · legions 005d1cb0
- [ ] `ShellCmdTable::AddCmdHandler` · 00613cc0 · legions 005d1d30
- [ ] `ShellCmdTable::AddHelpHandler` · 00613e00 · legions 005d1e70
- [ ] `ShellCmdTable::GetHelpText` · 00613f40 · legions 005d1fb0
- [ ] `ShellCmdTable::ShellCmdTable` · 00613f80 · legions 005d1ff0
