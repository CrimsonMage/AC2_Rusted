# ac2-net: Network transport, RPC, crypto, GUN, chat, HTTP

556 retail functions: 217 done, 15 partial, 324 to do, 0 skipped.
Legend and workflow: [README](README.md).

## `ac2/client/src/gun/clientguninterface.cpp.c` (0/2)

### ClientGunInterface
- [ ] `ClientGunInterface::ClientGunInterface` · 00728240 · legions 006a6520
- [ ] `ClientGunInterface::GetPlayerInterface` · 00728290 · legions 006a6570

## `ac2/client/src/gun/event/cclichatevent.cpp.c` (1/2)

### CCliChatEvent
- [x] `CCliChatEvent::HearRoom` · 007281f0 · legions 006a64d0 — ac2-weenie OnlineSession::send_to_room / receive_chat over ac2-net turbine_chat (chatclient.dll's protocol)
- [ ] `CCliChatEvent::HearUser` · 00728220 · legions 006a6500

## `ac2/client/src/gun/event/ccliconnectevent.cpp.c` (0/1)

### CCliConnectEvent
- [ ] `CCliConnectEvent::HandleConnectEvent` · 00728380

## `ac2/client/src/gun/event/cclifriendsevent.cpp.c` (0/3)

### CCliFriendsEvent
- [ ] `CCliFriendsEvent::FriendStatus` · 007283c0 · legions 006a66d0
- [ ] `CCliFriendsEvent::CCliFriendsEvent` · 00728470 · legions 006a6780
- [ ] `CCliFriendsEvent::FriendStart` · 00728500 · legions 006a6810

## `ac2/client/src/gun/result/cclicallcreateresult.cpp.c` (0/3)

### CCliCallCreateResult
- [ ] `CCliCallCreateResult::HandleCallCreateResult` · 00728ee0 · legions 006a71c0

### CTCallCreateResult
- [ ] `CTCallCreateResult::CTCallCreateResult` · 00728f20 · legions 006a7200

### CCliCallCreateResult
- [ ] `CCliCallCreateResult::CCliCallCreateResult` · 00728fc0 · legions 006a72a0

## `ac2/client/src/gun/result/cclifriendaddresult.cpp.c` (0/2)

### CCliFriendAddResult
- [~] `CCliFriendAddResult::HandleFriendAddResult` · 00728690 — net: the start body raises no list notice in Legions (its status notices are gated by the stored list, CPlayer 0x15a / gmCEntity 0x15, not traced); the panel takes the list from the GUN event; the list conversion is IFunArg::VmList (ac2-script ifun.rs); the add button is a UI path not wired to a frame event
- [ ] `CCliFriendAddResult::CCliFriendAddResult` · 007286d0 · legions 006a69d0

## `ac2/client/src/gun/result/cclifrienddeleteresult.cpp.c` (0/2)

### CCliFriendDeleteResult
- [ ] `CCliFriendDeleteResult::HandleFriendDeleteResult` · 00728780 · legions 006a6ae0
- [ ] `CCliFriendDeleteResult::CCliFriendDeleteResult` · 007287c0 · legions 006a6b10

## `ac2/client/src/gun/result/ccligetfriendslistresult.cpp.c` (0/2)

### CCliGetFriendsListResult
- [ ] `CCliGetFriendsListResult::CCliGetFriendsListResult` · 00728870 · legions 006a6bc0
- [~] `CCliGetFriendsListResult::HandleGetFriendsListResult` · 00728910 — net: the start body raises no list notice in Legions (its status notices are gated by the stored list, CPlayer 0x15a / gmCEntity 0x15, not traced); the panel takes the list from the GUN event; the list conversion is IFunArg::VmList (ac2-script ifun.rs); the add button is a UI path not wired to a frame event

## `ac2/client/src/gun/result/ccligetignoredlistresult.cpp.c` (0/2)

### CCliGetIgnoredListResult
- [ ] `CCliGetIgnoredListResult::CCliGetIgnoredListResult` · 00728d00 · legions 006a7020
- [ ] `CCliGetIgnoredListResult::HandleGetIgnoredListResult` · 00728dd0

## `ac2/client/src/gun/result/ccliignoredaddresult.cpp.c` (0/2)

### CCliIgnoredAddResult
- [ ] `CCliIgnoredAddResult::HandleIgnoredAddResult` · 00728ad0
- [ ] `CCliIgnoredAddResult::CCliIgnoredAddResult` · 00728b10 · legions 006a6e70

## `ac2/client/src/gun/result/ccliignoreddeleteresult.cpp.c` (0/3)

### CCliIgnoredDeleteResult
- [ ] `CCliIgnoredDeleteResult::HandleIgnoredDeleteResult` · 00728bc0 · legions 006a6f20
- [ ] `CCliIgnoredDeleteResult::CCliIgnoredDeleteResult` · 00728c00 · legions 006a6f50

### CCliFriendAddResult
- [ ] `CCliFriendAddResult::`vbase_destructor'` · 00728ce0 · legions 006a7000

## `ac2/client/src/gun/result/cclisendtoroomresult.cpp.c` (0/3)

### CCliSendToRoomResult
- [~] `CCliSendToRoomResult::HandleSendToRoomResult` · 007291f0 · legions 006a74c0 — ac2-weenie GameplayNotice::HearFromRoom / SendToRoomResult; the CPlayer script's channel formatting is not mirrored
- [ ] `CCliSendToRoomResult::CCliSendToRoomResult` · 00729240 · legions 006a7500
- [ ] `CCliSendToRoomResult::`vbase_destructor'` · 00729320 · legions 006a75e0

## `ac2/client/src/gun/result/cclisendtouserresult.cpp.c` (0/3)

### CCliSendToUserResult
- [ ] `CCliSendToUserResult::HandleSendToUserResult` · 00729060
- [ ] `CCliSendToUserResult::CCliSendToUserResult` · 007290b0 · legions 006a7380

### CTCallCreateResult
- [ ] `CTCallCreateResult::~CTCallCreateResult` · 007291a0 · legions 006a7470

## `ac2/client/src/gun/result/ccliticketresult.cpp.c` (0/3)

### CCliTicketResult
- [ ] `CCliTicketResult::HandleTicketArrived` · 007282a0
- [ ] `CCliTicketResult::HandleAuthenticationResult` · 007282c0
- [ ] `CCliTicketResult::CCliTicketResult` · 007282e0 · legions 006a65c0

## `ac2/shared/src/gun/ctbullet.cpp.c` (1/1)

### CTBullet
- [x] `CTBullet::ConvertToWeenieResult` · 00735c60 · legions 006b1900 — net: friends_script_7160 / ac2-ui friends model (the status event runs the script; the native's efuns read name, location, mode; the GUN code converts through the admin table, 23 rows, default 0xc741)

## `ac2/shared/src/gun/helper/turbineuserstatus.cpp.c` (3/10)

### TurbineUserStatus
- [x] `TurbineUserStatus::InqMode` · 00604530 · legions 005c5410 — net: friends_script_7160 / ac2-ui friends model (the status event runs the script; the native's efuns read name, location, mode; the GUN code converts through the admin table, 23 rows, default 0xc741)
- [ ] `TurbineUserStatus::~TurbineUserStatus` · 00604550 · legions 005c5430
- [ ] `TurbineUserStatus::Clear` · 006045a0 · legions 005c5480
- [ ] `TurbineUserStatus::operator=` · 00604630 · legions 005c5510
- [x] `TurbineUserStatus::InqName` · 006046c0 · legions 005c55a0 — net: friends_script_7160 / ac2-ui friends model (the status event runs the script; the native's efuns read name, location, mode; the GUN code converts through the admin table, 23 rows, default 0xc741)
- [ ] `TurbineUserStatus::SetLocation` · 00604720 · legions 005c5600
- [x] `TurbineUserStatus::InqLocation` · 00604770 · legions 005c5650 — net: friends_script_7160 / ac2-ui friends model (the status event runs the script; the native's efuns read name, location, mode; the GUN code converts through the admin table, 23 rows, default 0xc741)
- [ ] `TurbineUserStatus::TurbineUserStatus` · 006047d0 · legions 005c56b0
- [ ] `TurbineUserStatus::TurbineUserStatus` · 00604830 · legions 005c5710
- [ ] `TurbineUserStatus::StreamPack` · 00604880 · legions 005c5760

## `ac2/shared/src/gun/result/ctresult.cpp.c` (0/1)

### CTResult
- [ ] `CTResult::GetHResultString` · 006075b0 · legions 005c8450

## `ac2/shared/src/gun/result/ctticketresult.cpp.c` (0/4)

### CTTicketResult
- [ ] `CTTicketResult::CTTicketResult` · 00607400 · legions 005c82a0
- [ ] `CTTicketResult::~CTTicketResult` · 00607470 · legions 005c8310
- [ ] `CTTicketResult::AdoptTicket` · 006074b0 · legions 005c8350
- [ ] `CTTicketResult::CopyTicket` · 00607500 · legions 005c83a0

## `ac2/shared/src/gun/turbinegun.cpp.c` (0/49)

### Gun::CGetListOfRoomsResult
- [ ] `Gun::CGetListOfRoomsResult::GetRoomList` · 006048d0 · legions 005c5840
- [ ] `Gun::CGetListOfRoomsResult::GetCount` · 00604900 · legions 005c5810

### Gun::CEnterPersonaResult
- [ ] `Gun::CEnterPersonaResult::GetServerId` · 00604930 · legions 005c57b0

### Gun::CBaseResult
- [ ] `Gun::CBaseResult::GetContext` · 00604960 · legions 005c5870

### Gun::CRoomEnterEvent
- [ ] `Gun::CRoomEnterEvent::GetUserNameA` · 00604990 · legions 005c58a0

### Gun::CGetTicketResult
- [ ] `Gun::CGetTicketResult::GetTicket` · 006049c0 · legions 005c57e0

### NSTurbineGun::TurbineGun
- [ ] `NSTurbineGun::TurbineGun::CheckForMessages` · 006049f0 · legions 005c58d0
- [ ] `NSTurbineGun::TurbineGun::GetAuthenticationTicket` · 00604a20 · legions 005c5900
- [ ] `NSTurbineGun::TurbineGun::Read` · 00604a80 · legions 005c5960
- [ ] `NSTurbineGun::TurbineGun::RcvEvent` · 00604ad0 · legions 005c59b0
- [ ] `NSTurbineGun::TurbineGun::RcvAccountsEvent` · 00604b70 · legions 005c5a50
- [ ] `NSTurbineGun::TurbineGun::RcvPersonaEvent` · 00604b90 · legions 005c5a70
- [ ] `NSTurbineGun::TurbineGun::RcvAccountsResult` · 00604bb0 · legions 005c5a90
- [ ] `NSTurbineGun::TurbineGun::RcvSupportResult` · 00604c10 · legions 005c5af0
- [ ] `NSTurbineGun::TurbineGun::RcvNewsResult` · 00604ca0 · legions 005c5b80
- [ ] `NSTurbineGun::TurbineGun::RcvChatResult` · 00604cf0 · legions 005c5bd0
- [ ] `NSTurbineGun::TurbineGun::RcvFriendsResult` · 00605140 · legions 005c6020
- [ ] `NSTurbineGun::TurbineGun::RcvMatchResult` · 00605440 · legions 005c6320
- [ ] `NSTurbineGun::TurbineGun::RcvAuthenticateResult` · 006054d0 · legions 005c63b0
- [ ] `NSTurbineGun::TurbineGun::RequestPresentAuthenticationTicket` · 00605640 · legions 005c6520
- [ ] `NSTurbineGun::TurbineGun::RequestFinishWithTicket` · 006056a0 · legions 005c6580
- [ ] `NSTurbineGun::TurbineGun::RequestSendToRoomEx` · 006056f0 · legions 005c65d0
- [ ] `NSTurbineGun::TurbineGun::RequestGetFriendsList` · 00605770 · legions 005c6650
- [ ] `NSTurbineGun::TurbineGun::RequestGetIgnoredList` · 006057d0 · legions 005c66b0
- [ ] `NSTurbineGun::TurbineGun::RequestFlags` · 00605830 · legions 005c6710
- [ ] `NSTurbineGun::TurbineGun::RequestSetLocation` · 006058a0 · legions 005c6780
- [ ] `NSTurbineGun::TurbineGun::RcvResult` · 00605900 · legions 005c67e0
- [ ] `NSTurbineGun::TurbineGun::UnRegisterEvents` · 00605a10 · legions 005c68f0
- [ ] `NSTurbineGun::TurbineGun::RequestConnect` · 00605ac0 · legions 005c69a0
- [ ] `NSTurbineGun::TurbineGun::SetAuthenticationTicket` · 00605ba0 · legions 005c6a80
- [ ] `NSTurbineGun::TurbineGun::RcvConnectEvent` · 00605ca0 · legions 005c6b80
- [ ] `NSTurbineGun::TurbineGun::RegisterForConnectEvents` · 00605e00 · legions 005c6ce0
- [ ] `NSTurbineGun::TurbineGun::RegisterForChatEvents` · 00605e40 · legions 005c6d20
- [ ] `NSTurbineGun::TurbineGun::RegisterForFriendsEvents` · 00605e80 · legions 005c6d60
- [ ] `NSTurbineGun::TurbineGun::Init` · 00605f00 · legions 005c6da0
- [ ] `NSTurbineGun::TurbineGun::TurbineGun` · 00606250 · legions 005c70f0
- [ ] `NSTurbineGun::TurbineGun::~TurbineGun` · 00606330 · legions 005c71d0
- [ ] `NSTurbineGun::TurbineGun::LocalToGlobalName` · 006064b0 · legions 005c7350
- [ ] `NSTurbineGun::TurbineGun::GlobalToLocalName` · 00606590 · legions 005c7430
- [ ] `NSTurbineGun::TurbineGun::RcvChatEvent` · 00606710
- [ ] `NSTurbineGun::TurbineGun::RcvFriendsEvent` · 00606c20 · legions 005c7ac0

### PList<PStringBase<unsigned_short>_>
- [ ] `PList<PStringBase<unsigned_short>_>::PList<PStringBase<unsigned_short>_>` · 00606d50 · legions 005c7bf0

### NSTurbineGun::TurbineGun
- [ ] `NSTurbineGun::TurbineGun::RequestSendToUserEx` · 00606df0 · legions 005c7c90
- [ ] `NSTurbineGun::TurbineGun::RequestFriendAdd` · 00606ec0 · legions 005c7d60
- [ ] `NSTurbineGun::TurbineGun::RequestFriendDelete` · 00606f50 · legions 005c7df0
- [ ] `NSTurbineGun::TurbineGun::RequestIgnoredAdd` · 00606fe0 · legions 005c7e80
- [ ] `NSTurbineGun::TurbineGun::RequestIgnoredDelete` · 00607070 · legions 005c7f10
- [ ] `NSTurbineGun::TurbineGun::RequestCallCreate` · 00607100 · legions 005c7fa0
- [ ] `NSTurbineGun::TurbineGun::RcvPersonaResult` · 006071f0 · legions 005c8090

## `ac2/shared/src/gun/turbinegunutils.cpp.c` (0/8)

### TurbineGunUtils
- [ ] `TurbineGunUtils::GetModeText` · 006075d0 · legions 005c8470
- [ ] `TurbineGunUtils::GetRoomModeText` · 00607690 · legions 005c8530
- [ ] `TurbineGunUtils::GunCreateInstance` · 00607780 · legions 005c8620
- [ ] `TurbineGunUtils::GetWorldName` · 00607830 · legions 005c86d0
- [ ] `TurbineGunUtils::GetWorldSuffix` · 00607870 · legions 005c8710
- [ ] `TurbineGunUtils::StringFromHR` · 006078b0 · legions 005c8750
- [ ] `TurbineGunUtils::PrintCmdHRESULT` · 00607930 · legions 005c87d0
- [ ] `TurbineGunUtils::GlobalToLocalName` · 006079a0 · legions 005c8840

## `portal/client/src/cchat/ac2clientchatmanager.cpp.c` (0/4)

### AC2ClientChatManager
- [ ] `AC2ClientChatManager::AC2ClientChatManager` · 0072f4f0 · legions 006acda0
- [ ] `AC2ClientChatManager::GetIChatClient` · 0072f510 · legions 006acdc0
- [ ] `AC2ClientChatManager::Init` · 0072f580 · legions 006ace30
- [ ] `AC2ClientChatManager::IncludeFileInWatsonUpload` · 0072f700 · legions 006acfa0

## `portal/client/src/ckeystone/ckeystone.cpp.c` (0/22)

### KeyStone
- [ ] `KeyStone::HasFailedToLoadMSXML4` · 004bdec0 · legions 004b21e0

### (free functions)
- [ ] `CheckForMSXML4` · 004bded0 · legions 004b21f0

### KeyStone
- [ ] `KeyStone::Update` · 004bdf70 · legions 004b2290
- [ ] `KeyStone::Create` · 004bdf80 · legions 004b22a0
- [ ] `KeyStone::Init` · 004be070 · legions 004b2390
- [ ] `KeyStone::DispatchMessageA` · 004be130 · legions 004b2450
- [ ] `KeyStone::IsHelpFocused` · 004be170 · legions 004b2490
- [ ] `KeyStone::IsOverKeystoneWindow` · 004be1a0 · legions 004b24c0
- [ ] `KeyStone::TranslateAcceleratorA` · 004be1d0 · legions 004b24f0
- [ ] `KeyStone::LoseFocus` · 004be200 · legions 004b2520
- [ ] `KeyStone::CloseHelp` · 004be230 · legions 004b2550
- [ ] `KeyStone::ClosePluginManager` · 004be250 · legions 004b2570
- [ ] `KeyStone::OpenPluginManager` · 004be270 · legions 004b2590
- [ ] `KeyStone::Release` · 004be2b0 · legions 004b25d0
- [ ] `KeyStone::Cleanup` · 004be300 · legions 004b2620
- [ ] `KeyStone::OpenHelp` · 004be3b0 · legions 004b26d0
- [ ] `KeyStone::IsHelpOpen` · 004be4c0 · legions 004b27e0
- [ ] `KeyStone::IsPluginManagerOpen` · 004be550 · legions 004b2870

### (free functions)
- [ ] `$L93066` · 0073de10 · legions 006b9cc0
- [ ] `$L93259` · 0073de30 · legions 006b9ce0
- [ ] `$L93260` · 0073de38 · legions 006b9ce8
- [ ] `$L93361` · 0073de50 · legions 006b9d00

## `portal/client/src/ckeystone/gunxmlparse.cpp.c` (0/4)

### CParseXml
- [ ] `CParseXml::CParseXml` · 004be5e0 · legions 004b2900
- [ ] `CParseXml::~CParseXml` · 004be600 · legions 004b2920
- [ ] `CParseXml::Init` · 004be620 · legions 004b2940

### (free functions)
- [ ] `FUN_004be6bb` · 004be6bb · legions 004b29db

## `portal/client/src/clinet/arrivedephinfo.cpp.c` (3/3)

### ArrivedEphInfo
- [x] `ArrivedEphInfo::UpdateNetBlobID` · 004cee60 · legions 004c3430
- [x] `ArrivedEphInfo::fTimedOut` · 004cee90 · legions 004c3460
- [x] `ArrivedEphInfo::ArrivedEphInfo` · 004ceeb0 · legions 004c3480

## `portal/client/src/clinet/cflowqueue.cpp.c` (32/39)

### FlowQueue
- [x] `FlowQueue::DequeuePacket` · 004cbe20 · legions 004c0360
- [x] `FlowQueue::EncryptChecksum` · 004cbe60 · legions 004c03a0
- [x] `FlowQueue::IncrementLocalInterval` · 004cbeb0 · legions 004c03f0

### ClientFlowQueue
- [x] `ClientFlowQueue::RecordBytesSent` · 004cbec0 · legions 004c0400

### NIList<NetBlob_*>
- [ ] `NIList<NetBlob_*>::DeleteContents` · 004cbee0 · legions 004b3360

### CTimeSyncHeader
- [x] `CTimeSyncHeader::UpdateTimeSensitivePayload` · 004cc010 · legions 004c0580

### CEchoRequestHeader
- [x] `CEchoRequestHeader::UpdateTimeSensitivePayload` · 004cc030

### FlowQueue
- [ ] `FlowQueue::SharedStaticCleanup` · 004cc040 · legions 004c05b0
- [x] `FlowQueue::DequeueAck` · 004cc080 · legions 004c05f0
- [x] `FlowQueue::EnqueuePacket` · 004cc140 · legions 004c06b0

### PQueueArray<unsigned_long,void_*>
- [x] `PQueueArray<unsigned_long,void_*>::Insert` · 004cc1b0 · legions 004c0720
- [x] `PQueueArray<unsigned_long,void_*>::RemoveMin` · 004cc210 · legions 004c0780

### CSeqIDListHeader<8192,33>
- [x] `CSeqIDListHeader<8192,33>::CSeqIDListHeader<8192,33>` · 004cc2c0 · legions 004c0830

### CTimeSyncHeader
- [x] `CTimeSyncHeader::CreateFromData` · 004cc320 · legions 004c0890

### CEchoRequestHeader
- [x] `CEchoRequestHeader::CreateFromData` · 004cc370 · legions 004c08e0

### FlowQueue
- [x] `FlowQueue::EnqueueOptionalHeader` · 004cc3c0 · legions 004c0930
- [x] `FlowQueue::FlushSentPackets` · 004cc410 · legions 004c0980
- [x] `FlowQueue::CoalesceData` · 004cc490 · legions 004c0a00
- [x] `FlowQueue::TransmitNewPackets` · 004cc7b0 · legions 004c0d20

### ClientFlowQueue
- [x] `ClientFlowQueue::Enqueue` · 004cca20 · legions 004c0f90
- [x] `ClientFlowQueue::Dequeue` · 004cca70 · legions 004c0fe0

### NIListIter<unsigned_long>
- [x] `NIListIter<unsigned_long>::AddBeforeCur` · 004ccaa0 · legions 004c1010 — FlowQueue::EnqueueAcks inserts into m_ackList before the cursor

### CPackObjHeader<NetError,2097152,2>
- [x] `CPackObjHeader<NetError,2097152,2>::CreateFromData` · 004ccb40 · legions 004c10b0

### COnePrimHeader<4194304,7,CICMDCommandStruct>
- [x] `COnePrimHeader<4194304,7,CICMDCommandStruct>::CreateFromData` · 004ccbc0 · legions 004c1130

### ClientFlowQueue
- [~] `ClientFlowQueue::OnLocalIntervalAck` · 004ccc10 · legions 004c1180 — not called by the client paths ported
- [x] `ClientFlowQueue::IncrementLocalInterval` · 004ccc60 · legions 004c11d0

### FlowQueue
- [ ] `FlowQueue::~FlowQueue` · 004ccde0 · legions 004c1350
- [x] `FlowQueue::DequeueEmptyAck` · 004cce50 · legions 004c13c0
- [x] `FlowQueue::CompileEmptyAcks` · 004ccec0 · legions 004c1430
- [x] `FlowQueue::TransmitNaks` · 004ccf30 · legions 004c14a0
- [ ] `FlowQueue::Destroy` · 004ccfc0 · legions 004c1530
- [x] `FlowQueue::EnqueueEmptyAck` · 004cd140 · legions 004c16b0
- [x] `FlowQueue::TransmitAcks` · 004cd1f0 · legions 004c1760

### ClientFlowQueue
- [ ] `ClientFlowQueue::~ClientFlowQueue` · 004cd340 · legions 004c18b0

### FlowQueue
- [x] `FlowQueue::EnqueueAcks` · 004cd370 · legions 004c18e0
- [x] `FlowQueue::Empty` · 004cd4b0 · legions 004c1a20
- [x] `FlowQueue::FlowQueue` · 004cd5d0 · legions 004c1b40

### ClientFlowQueue
- [x] `ClientFlowQueue::ClientFlowQueue` · 004cd6e0 · legions 004c1c50

### FlowQueue
- [ ] `FlowQueue::SharedStaticInit` · 004cd7a0 · legions 004c1d10

## `portal/client/src/clinet/chargendata.cpp.c` (1/1)

### CharGenData
- [x] `CharGenData::Save` · 004cdf20 · legions 004c2490 — ac2-client chargen_data.rs CharGenCapture::save and capture_image (record stream; written on a worker thread; --chargen names the file)

## `portal/client/src/clinet/cindicator.cpp.c` (6/15)

### HashBaseIter<unsigned___int64>
- [ ] `HashBaseIter<unsigned___int64>::DeleteCurrent` · 004ce190 · legions 004c2700
- [ ] `HashBaseIter<unsigned___int64>::Next` · 004ce220 · legions 004c2790

### HashBase<unsigned___int64>
- [ ] `HashBase<unsigned___int64>::remove` · 004ce270 · legions 004c27e0
- [ ] `HashBase<unsigned___int64>::lookup` · 004ce300 · legions 004c2870

### UI64Hash<NetBlob>
- [ ] `UI64Hash<NetBlob>::add` · 004ce350 · legions 004c2900

### Indicator
- [x] `Indicator::SendBlobToQueue` · 004ce3f0 · legions 004c2960
- [ ] `Indicator::Indicator` · 004ce440 · legions 004c29b0
- [x] `Indicator::AddArrivedEphInfo` · 004ce560 · legions 004c2ad0
- [x] `Indicator::FlushTimedOutEphInfo` · 004ce5c0 · legions 004c2b30
- [x] `Indicator::FragIsObsoleteEmphemeral` · 004ce640 · legions 004c2bb0

### UI64Hash<ArrivedEphInfo>
- [ ] `UI64Hash<ArrivedEphInfo>::destroy_contents` · 004ce710 · legions 004c2c80

### Indicator
- [ ] `Indicator::CleanupWaitingBlobs` · 004ce770 · legions 004c2ce0
- [x] `Indicator::AcceptFrag` · 004ce830 · legions 004c2da0
- [ ] `Indicator::~Indicator` · 004ce980 · legions 004c2ef0
- [x] `Indicator::CheckInPacket` · 004ce9d0 · legions 004c2f40

## `portal/client/src/clinet/clientnet.cpp.c` (56/83)

### CQuickTimer
- [ ] `CQuickTimer::Expired` · 004c52b0 · legions 004b9670

### SharedNet
- [~] `SharedNet::UseTime` · 004c52d0 · legions 004b9690 — frame order and lastDidUseTime_ kept; perf counters and the UseTime time limit not kept
- [x] `SharedNet::ReadAndProcessPackets` · 004c5360 · legions 004b9720
- [x] `SharedNet::ProcessNewestSeqNum` · 004c53e0 · legions 004b97a0
- [x] `SharedNet::ChecksumHeader` · 004c5440 · legions 004b9800
- [x] `SharedNet::ProcessOptionalHeaders` · 004c54a0 · legions 004b9860
- [x] `SharedNet::ProcessBlobFrags` · 004c5500 · legions 004b98c0
- [x] `SharedNet::ProcessConnections` · 004c5520 · legions 004b98e0
- [x] `SharedNet::SendBuff` · 004c5570 · legions 004b9930
- [x] `SharedNet::SendPacket` · 004c5640 · legions 004b9a00
- [x] `SharedNet::SetConnectionState` · 004c5760 · legions 004b9b20

### (free functions)
- [ ] `ParseInAddr` · 004c57b0 · legions 004b9b70

### ClientNet
- [x] `ClientNet::UseTime` · 004c5870 · legions 004b9c30
- [ ] `ClientNet::OnStateBegun` · 004c5890 · legions 004b9c50
- [x] `ClientNet::EnterWorld` · 004c58b0 · legions 004b9c70
- [~] `ClientNet::ExitWorldDisconnect` · 004c58d0 · legions 004b9c90 — logoff path only
- [x] `ClientNet::SendBuff` · 004c5950 · legions 004b9d10

### COnePrimHeader<512,7,sockaddr_in>
- [x] `COnePrimHeader<512,7,sockaddr_in>::InqData` · 004c59b0 · legions 004b9d70

### AVLNode<unsigned_long,unsigned_long>
- [ ] `AVLNode<unsigned_long,unsigned_long>::~AVLNode<unsigned_long,unsigned_long>` · 004c59f0 · legions 004b9db0

### AVL<unsigned_long,unsigned_long>
- [x] `AVL<unsigned_long,unsigned_long>::LeftRightRotate` · 004c5a40 · legions 005d0c70 — ReceiverData::m_SeqIDsWeNAKed as a BTreeMap: same key order for GetNaks, same add/remove results

### CEchoResponseHeader
- [x] `CEchoResponseHeader::UpdateTimeSensitivePayload` · 004c5b80

### CLinkStatusAverages
- [ ] `CLinkStatusAverages::CLinkStatusAverages` · 004c5b90 · legions 004b9f50

### SharedNet
- [x] `SharedNet::ReadNextPacket` · 004c5c80 · legions 004ba040
- [x] `SharedNet::HandlePak` · 004c5d20 · legions 004ba0e0
- [x] `SharedNet::HandleNak` · 004c5d60 · legions 004ba120
- [x] `SharedNet::SetNakState` · 004c5db0 · legions 004ba170
- [ ] `SharedNet::AddToPerfCounter` · 004c5de0 · legions 004ba1a0

### ClientNet
- [ ] `ClientNet::AddToPerfCounter` · 004c5ed0 · legions 004ba290

### TimeStampUtils
- [x] `TimeStampUtils::OverflowCompare` · 004c5fd0 · legions 004ba390

### CEchoResponseHeader
- [x] `CEchoResponseHeader::CreateFromData` · 004c60c0 · legions 004ba480

### NetPacket
- [x] `NetPacket::CreateForSend` · 004c6120 · legions 004ba4e0
- [ ] `NetPacket::~NetPacket` · 004c61c0 · legions 004ba580

### SharedNet
- [ ] `SharedNet::~SharedNet` · 004c6260 · legions 004ba6a0
- [x] `SharedNet::SplitPacketData` · 004c63b0 · legions 004ba7f0
- [x] `SharedNet::ProcessOptionalHeader` · 004c65f0 · legions 004baa30
- [x] `SharedNet::SendOptionalHeader` · 004c66e0 · legions 004bab20

### (free functions)
- [ ] `ParseInterfaceString` · 004c67d0 · legions 004bac10

### ClientNet
- [~] `ClientNet::EndState` · 004c6900 · legions 004bad40 — logon outcome as an event; no async state machine
- [x] `ClientNet::InitCrypto` · 004c6980 · legions 004badc0 — SessionKey::generate + ServerPublicKey: Base-provider 40-bit RC4 with 11-byte zero salt; key-container fallback (InitCliCrypto) not needed
- [x] `ClientNet::VerifyHeader` · 004c6bd0 · legions 004bb010
- [~] `ClientNet::NotifyPluginsOfStatusChange` · 004c6d80 · legions 004bb1c0 — Net_LoginConnectionError / Net_WorldConnectionError surface as NetEvent::ConnectionError; no plugin list
- [ ] `ClientNet::NotifyPluginsOfHeartbeat` · 004c6dd0 · legions 004bb210

### COnePrimHeader<134217728,16,CFlowStruct>
- [x] `COnePrimHeader<134217728,16,CFlowStruct>::CreateFromData` · 004c6e10 · legions 004bb250

### CPackObjHeader<NetError,1048576,7>
- [x] `CPackObjHeader<NetError,1048576,7>::CreateFromData` · 004c6e60 · legions 004bb2a0

### COnePrimHeader<2048,1073741922,CReferralStruct>
- [x] `COnePrimHeader<2048,1073741922,CReferralStruct>::CreateFromData` · 004c6ee0 · legions 004bb320

### AVL<unsigned_long,unsigned_long>
- [x] `AVL<unsigned_long,unsigned_long>::ReBalanceTree` · 004c6f30 · legions 004bb370 — ReceiverData::m_SeqIDsWeNAKed as a BTreeMap: same key order for GetNaks, same add/remove results
- [x] `AVL<unsigned_long,unsigned_long>::FindSuccessor` · 004c6fa0 · legions 004bb3e0 — ReceiverData::m_SeqIDsWeNAKed as a BTreeMap: same key order for GetNaks, same add/remove results

### TimeStampUtils
- [x] `TimeStampUtils::lhs_newer` · 004c7040 · legions 004bb480
- [x] `TimeStampUtils::lhs_newer` · 004c7080 · legions 004bb4c0

### SharedNet
- [x] `SharedNet::SeqIDSanityCheck` · 004c70c0 · legions 004bb500
- [x] `SharedNet::ProcessNewRemoteInterval` · 004c7120 · legions 004bb560
- [x] `SharedNet::EnqueuePak` · 004c71b0 · legions 004bb5f0
- [x] `SharedNet::EnqueueNaks` · 004c7270 · legions 004bb6b0

### ClientNet
- [ ] `ClientNet::OnContextEnded` · 004c7360 · legions 004bb7a0
- [x] `ClientNet::HandleTimeSynch` · 004c73d0 · legions 004bb810
- [x] `ClientNet::HandleServerSwitch` · 004c74c0 · legions 004bb900
- [x] `ClientNet::LogOffServer` · 004c7590 · legions 004bb9d0
- [x] `ClientNet::SendConnectAck` · 004c7630 · legions 004bba70
- [x] `ClientNet::ProcessReferralQueue` · 004c7730 · legions 004bbb70

### AVL<unsigned_long,unsigned_long>
- [x] `AVL<unsigned_long,unsigned_long>::Remove` · 004c7950 · legions 004bbd80 — ReceiverData::m_SeqIDsWeNAKed as a BTreeMap: same key order for GetNaks, same add/remove results

### SmartArray<ClientNet::CReferralQueueEntry>
- [ ] `SmartArray<ClientNet::CReferralQueueEntry>::AddToEnd` · 004c7ab0 · legions 004bbee0

### CClientsideLoginStateHandler
- [ ] `CClientsideLoginStateHandler::OnStateHandlerInit` · 004c7b10 · legions 004bbfd0

### SharedNet
- [x] `SharedNet::ProcessNewSeqNum` · 004c7c00 · legions 004bc0c0
- [x] `SharedNet::ProcessPacket` · 004c7d00 · legions 004bc1c0
- [x] `SharedNet::HandleEmptyAck` · 004c7e60 · legions 004bc320
- [x] `SharedNet::ProcessConnection` · 004c7ec0 · legions 004bc380

### ClientNet
- [ ] `ClientNet::OnContextBegun` · 004c7ef0 · legions 004bc3b0
- [x] `ClientNet::ProcessPacket` · 004c7f80 · legions 004bc440
- [x] `ClientNet::ProcessOptionalHeader` · 004c8020 · legions 004bc4e0
- [x] `ClientNet::RemoveConnection` · 004c8240 · legions 004bc700
- [x] `ClientNet::SendLoginRequest` · 004c82c0 · legions 004bc780
- [x] `ClientNet::ProcessConnections` · 004c8420 · legions 004bc8e0
- [~] `ClientNet::ProcessConnection` · 004c8430 · legions 004bc8f0 — heartbeat link-status snapshots not kept
- [ ] `ClientNet::~ClientNet` · 004c8950 · legions 004bcea0
- [x] `ClientNet::LogonUseTime` · 004c8a50 · legions 004bcfa0
- [ ] `ClientNet::InitAddresses` · 004c8b40 · legions 004bd090
- [~] `ClientNet::Init` · 004c9050 · legions 004bd5a0 — connection sequence from real time and SIMPLEBLOB session-key export; no address/interface parsing
- [x] `ClientNet::HandleConnectionRequest` · 004c9200 · legions 004bd750
- [x] `ClientNet::HandleReferral` · 004c9480 · legions 004bd9d0
- [~] `ClientNet::Init` · 004c95a0 · legions 004bdaf0 — connection sequence from real time; no address/interface parsing or crypto key export

### PerfMonCounterFactory
- [ ] `PerfMonCounterFactory::CreateNewCounter` · 004c9650 · legions 004bdc40

### SharedNet
- [ ] `SharedNet::SharedNet` · 004c96b0 · legions 004bdca0

### ClientNet
- [ ] `ClientNet::ClientNet` · 004c9a70 · legions 004be060

## `portal/client/src/clinet/linkstatus.cpp.c` (0/10)

### CLinkStatusAverages::CAverager<float,4>
- [ ] `CLinkStatusAverages::CAverager<float,4>::AddSample` · 004c9f80 · legions 004be560

### CLinkStatusAverages::CAverager<unsigned_short,40>
- [ ] `CLinkStatusAverages::CAverager<unsigned_short,40>::AddSample` · 004c9ff0 · legions 004be5d0

### CLinkStatusAverages::CAverager<unsigned_long,2>
- [ ] `CLinkStatusAverages::CAverager<unsigned_long,2>::AddSample` · 004ca060 · legions 004be640

### CLinkStatusAverages::CAverager<double,2>
- [ ] `CLinkStatusAverages::CAverager<double,2>::AddSample` · 004ca0e0 · legions 004be6c0

### CLinkStatusAverages
- [ ] `CLinkStatusAverages::GetAveragePacketLoss` · 004ca150 · legions 004be730
- [ ] `CLinkStatusAverages::GetAverageBytesPerSecondReceived` · 004ca190 · legions 004be770
- [ ] `CLinkStatusAverages::AddSnapshot` · 004ca1c0 · legions 004be7a0
- [ ] `CLinkStatusAverages::OnPingResponse` · 004ca2a0 · legions 004be880

### (free functions)
- [ ] `RemoveLinkStatusPlugin` · 004ca2b0 · legions 004be890
- [ ] `AddLinkStatusPlugin` · 004ca2e0 · legions 004be8c0

## `portal/client/src/clinet/pui.cpp.c` (7/8)

### Proto_UI
- [x] `Proto_UI::SendBlob` · 004c9b70 · legions 004be150
- [x] `Proto_UI::SetOrderingStamp` · 004c9c10 · legions 004be1f0
- [x] `Proto_UI::SendBuff` · 004c9c20 · legions 004be200
- [x] `Proto_UI::LogOffCharacter` · 004c9d10 · legions 004be2f0
- [x] `Proto_UI::SendCharGenResult` · 004c9d40 · legions 004be320 — ac2-net ClientEvent::CharGenResult (queue 8) / ac2-weenie OnlineSession::create_character; CharGenData::Save capture in ac2-client
- [x] `Proto_UI::SendDeleteCharacter` · 004c9e00 · legions 004be3e0
- [x] `Proto_UI::SendEnterWorld` · 004c9e70 · legions 004be450
- [ ] `Proto_UI::SendAdminRestoreCharacter` · 004c9ef0 · legions 004be4d0

## `portal/client/src/http/connectioncache.cpp.c` (0/8)

### ConnectionCache::Entry
- [ ] `ConnectionCache::Entry::Entry` · 006fbd20 · legions 0067bad0
- [ ] `ConnectionCache::Entry::Entry` · 006fbd80 · legions 0067bb30

### ConnectionCache
- [ ] `ConnectionCache::GetConnection` · 006fbde0 · legions 0067bb90
- [ ] `ConnectionCache::CacheConnection` · 006fc030 · legions 0067bde0

### _STL::deque<ConnectionCache::Entry,_STL::allocator<ConnectionCache::Entry>_>
- [ ] `_STL::deque<ConnectionCache::Entry,_STL::allocator<ConnectionCache::Entry>_>::~deque<ConnectionCache::Entry,_STL::allocator<ConnectionCache::Entry>_>` · 006fc150 · legions 0067bf00

### ConnectionCache
- [ ] `ConnectionCache::ConnectionCache` · 006fc2f0 · legions 0067c0a0
- [ ] `ConnectionCache::~ConnectionCache` · 006fc320 · legions 0067c0d0
- [ ] `ConnectionCache::Clear` · 006fc330 · legions 0067c0e0

## `portal/client/src/http/httpclient.cpp.c` (0/8)

### HttpClient
- [ ] `HttpClient::HttpClient` · 006fa830 · legions 0067a630
- [ ] `HttpClient::~HttpClient` · 006fa870 · legions 0067a670
- [ ] `HttpClient::Initialize` · 006fa890 · legions 0067a690

### CAsyncStateHandler
- [ ] `CAsyncStateHandler::EndStateOK` · 006fa8b0 · legions 0067a6b0
- [ ] `CAsyncStateHandler::EndStateFailed` · 006fa8c0 · legions 0067a6c0

### HttpClient
- [ ] `HttpClient::Terminate` · 006fa8d0 · legions 0067a6d0
- [ ] `HttpClient::GetToSmartBuffer` · 006fa8e0 · legions 0067a6e0
- [ ] `HttpClient::PostToSmartBuffer` · 006fa900

## `portal/client/src/http/wininetasynchttpclient.cpp.c` (0/25)

### WinInetAsyncHttpClient
- [ ] `WinInetAsyncHttpClient::Initialize` · 006f9d00 · legions 00679c30
- [ ] `WinInetAsyncHttpClient::IsInitialized` · 006f9d60
- [ ] `WinInetAsyncHttpClient::Terminate` · 006f9d70 · legions 00679ca0
- [ ] `WinInetAsyncHttpClient::QueryInterface` · 006f9d90

### WinInetAsyncHttpClient::OnErrorEvent
- [ ] `WinInetAsyncHttpClient::OnErrorEvent::Dispatch` · 006f9df0

### WinInetAsyncHttpClient::OnDownloadStatusEvent
- [ ] `WinInetAsyncHttpClient::OnDownloadStatusEvent::Dispatch` · 006f9e10

### WinInetAsyncHttpClient::OnResponseEvent
- [ ] `WinInetAsyncHttpClient::OnResponseEvent::Dispatch` · 006f9e30

### WinInetAsyncHttpClient
- [ ] `WinInetAsyncHttpClient::UseTime` · 006f9e80 · legions 00679df2

### LFQueue<AsyncCache::CCallbackHandler_*>
- [ ] `LFQueue<AsyncCache::CCallbackHandler_*>::Produce` · 006f9f10 · legions 00659370

### WinInetAsyncHttpClient
- [ ] `WinInetAsyncHttpClient::WinInetAsyncHttpClient` · 006fa070 · legions 00679e70
- [ ] `WinInetAsyncHttpClient::WaitForWorkerThreadToTerminate` · 006fa140
- [ ] `WinInetAsyncHttpClient::~WinInetAsyncHttpClient` · 006fa160 · legions 00679f60
- [ ] `WinInetAsyncHttpClient::Startup` · 006fa230 · legions 0067a030
- [ ] `WinInetAsyncHttpClient::WakeForTheReaper` · 006fa2d0 · legions 0067a0d0
- [ ] `WinInetAsyncHttpClient::OnResponse` · 006fa2f0
- [ ] `WinInetAsyncHttpClient::OnError` · 006fa350
- [ ] `WinInetAsyncHttpClient::OnDownloadStatus` · 006fa3a0
- [ ] `WinInetAsyncHttpClient::OnDownloadStatus` · 006fa3f0
- [ ] `WinInetAsyncHttpClient::OnError` · 006fa450

### WinInetAsyncHttpClient::GetRequest
- [ ] `WinInetAsyncHttpClient::GetRequest::~GetRequest` · 006fa540 · legions 0067a340
- [ ] `WinInetAsyncHttpClient::GetRequest::Dispatch` · 006fa5d0 · legions 0067a3d0

### WinInetAsyncHttpClient::PostRequest
- [ ] `WinInetAsyncHttpClient::PostRequest::PostRequest` · 006fa680 · legions 0067a480
- [ ] `WinInetAsyncHttpClient::PostRequest::Dispatch` · 006fa6d0

### WinInetAsyncHttpClient
- [ ] `WinInetAsyncHttpClient::Get` · 006fa770
- [ ] `WinInetAsyncHttpClient::Post` · 006fa7e0 · legions 0067a5e0

## `portal/client/src/http/wininetimplementation.cpp.c` (0/14)

### WinInetImplementation
- [ ] `WinInetImplementation::WinInetImplementation` · 006fa970 · legions 0067a770
- [ ] `WinInetImplementation::Initialize` · 006fa990 · legions 0067a790

### DiskConBase
- [ ] `DiskConBase::GetDataset` · 006faa10 · legions 0067a810

### GraphicsResource
- [ ] `GraphicsResource::SetResourceSize` · 006faa20 · legions 0067a820

### WinInetImplementation
- [ ] `WinInetImplementation::AddThrottleDelay` · 006faa30 · legions 0067a830
- [ ] `WinInetImplementation::ReadResponseToSmartBuffer` · 006faa80 · legions 0067a880
- [ ] `WinInetImplementation::ReadResponseToFileHandle` · 006fad00 · legions 0067ab00
- [ ] `WinInetImplementation::Terminate` · 006faf90 · legions 0067ad90
- [ ] `WinInetImplementation::SendPost` · 006fb080 · legions 0067ae80
- [ ] `WinInetImplementation::SendGet` · 006fb290 · legions 0067b090
- [ ] `WinInetImplementation::PostToFileHandle` · 006fb670 · legions 0067b470
- [ ] `WinInetImplementation::PostToSmartBuffer` · 006fb6b0 · legions 0067b4b0
- [ ] `WinInetImplementation::GetToFileHandle` · 006fb6f0 · legions 0067b4f0
- [ ] `WinInetImplementation::GetToSmartBuffer` · 006fb740 · legions 0067b540

## `portal/shared/src/crypto/cryptosystem.cpp.c` (7/10)

### CryptoSystem
- [ ] `CryptoSystem::~CryptoSystem` · 0065fd60 · legions 0060e670

### QTIsaac<8,unsigned_long>
- [ ] `QTIsaac<8,unsigned_long>::srand` · 0065fd80 · legions 0060e690

### QTIsaac<8,unsigned_long>::randctx
- [ ] `QTIsaac<8,unsigned_long>::randctx::~randctx` · 0065fec0 · legions 0060e7d0

### QTIsaac<8,unsigned_long>
- [x] `QTIsaac<8,unsigned_long>::QTIsaac<8,unsigned_long>` · 0065fee0 · legions 0060e7f0
- [x] `QTIsaac<8,unsigned_long>::randinit` · 0065ff80 · legions 0060e890
- [x] `QTIsaac<8,unsigned_long>::shuffle` · 00660280 · legions 0060eb90

### CryptoSystem
- [x] `CryptoSystem::CryptoSystem` · 00660390 · legions 0060eca0

### QTIsaac<8,unsigned_long>
- [x] `QTIsaac<8,unsigned_long>::isaac` · 006603e0 · legions 0060ecf0

### CryptoSystem
- [x] `CryptoSystem::GetNextCryptoKey` · 006606a0 · legions 0060efb0
- [x] `CryptoSystem::EncryptData` · 006606e0 · legions 0060eff0

## `portal/shared/src/crypto/portaldh.cpp.c` (0/1)

### PortalDH
- [ ] `PortalDH::Init` · 0065fd30 · legions 0060e640

## `portal/shared/src/newnet/asyncstate.cpp.c` (0/19)

### CAsyncStateHandler
- [ ] `CAsyncStateHandler::CAsyncStateHandler` · 0063a6e0 · legions 005ef330

### CAsyncStateMachine
- [ ] `CAsyncStateMachine::AllocateStateData` · 0063a720 · legions 005ef370

### ListIterator<CAsyncStateHandler_*>
- [ ] `ListIterator<CAsyncStateHandler_*>::insert` · 0063a7f0 · legions 005ef440

### CAsyncStateMachine
- [ ] `CAsyncStateMachine::AddStateHandler` · 0063a940 · legions 005ef470
- [ ] `CAsyncStateMachine::BeginContext` · 0063ab50 · legions 005ef6d0
- [ ] `CAsyncStateMachine::EndAndReleaseContext` · 0063abd0 · legions 005ef750
- [ ] `CAsyncStateMachine::OnHandlerDoneOK` · 0063acb0 · legions 005ef830
- [ ] `CAsyncStateMachine::OnHandlerFailed` · 0063ace0 · legions 005ef860
- [ ] `CAsyncStateMachine::Advance` · 0063acf0 · legions 005ef870

### CAsyncStateHandler
- [ ] `CAsyncStateHandler::GetContextData` · 0063ae50 · legions 005ef9d0
- [ ] `CAsyncStateHandler::SetContextData` · 0063aea0 · legions 005efa20

### CAsyncStateMachine
- [ ] `CAsyncStateMachine::RemoveStateHandler` · 0063aef0 · legions 005efa70
- [ ] `CAsyncStateMachine::AllocateAndBeginContext` · 0063afc0 · legions 005efb40

### CAsyncStateHandler
- [ ] `CAsyncStateHandler::~CAsyncStateHandler` · 0063b090 · legions 005efc10

### CAsyncStateMachine
- [ ] `CAsyncStateMachine::~CAsyncStateMachine` · 0063b0c0 · legions 005efc40
- [ ] `CAsyncStateMachine::CAsyncStateMachine` · 0063b270 · legions 005efda0
- [ ] `CAsyncStateMachine::AllocateContext` · 0063b2d0 · legions 005efe00
- [ ] `CAsyncStateMachine::RegisterContextDataName` · 0063b390 · legions 005efec0

### CAsyncStateHandler
- [ ] `CAsyncStateHandler::RegisterContextDataName` · 0063b400 · legions 005eff30

## `portal/shared/src/newnet/blobfrag.cpp.c` (4/4)

### BlobFrag
- [x] `BlobFrag::BlobFrag` · 004cbc30 · legions 004c0170
- [x] `BlobFrag::BlobFrag` · 004cbd00 · legions 004c0240
- [x] `BlobFrag::CreateForSend` · 004cbd60 · legions 004c02a0
- [x] `BlobFrag::CreateForRecv` · 004cbda0 · legions 004c02e0

## `portal/shared/src/newnet/interfaceenumerator.cpp.c` (0/2)

### InterfaceEnumerator
- [ ] `InterfaceEnumerator::InterfaceEnumerator` · 0063c590 · legions 005f10e0
- [ ] `InterfaceEnumerator::~InterfaceEnumerator` · 0063c630 · legions 005f1180

## `portal/shared/src/newnet/netauthenticator.cpp.c` (6/11)

### NetAuthenticator::CGrowBuffer
- [x] `NetAuthenticator::CGrowBuffer::Grow` · 004c4ed0 · legions 004b92e0

### NetAuthenticator
- [x] `NetAuthenticator::SetCryptoKey` · 004c4f10 · legions 004b9320

### NetAuthenticator::CGrowBuffer
- [x] `NetAuthenticator::CGrowBuffer::StreamPack` · 004c4f80 · legions 004b9390

### NetAuthenticator
- [x] `NetAuthenticator::NetAuthenticator` · 004c4ff0 · legions 004b9400

### StreamPackObj
- [ ] `StreamPackObj::GetPackSize` · 004c5050 · legions 004759b0

### NetAuthenticator
- [ ] `NetAuthenticator::Pack` · 004c5080

### (free functions)
- [ ] `STREAMPACK_STREAMPACKOBJ` · 004c50c0 · legions 004b94a0

### NetAuthenticator
- [ ] `NetAuthenticator::~NetAuthenticator` · 004c5100 · legions 004b94e0
- [x] `NetAuthenticator::SetToAuthType` · 004c5170 · legions 004b9550
- [ ] `NetAuthenticator::SetAdminAccountOverride` · 004c51e0
- [x] `NetAuthenticator::StreamPack` · 004c5200 · legions 004b95c0

## `portal/shared/src/newnet/netblob.cpp.c` (6/10)

### NetBlob
- [x] `NetBlob::ReceiveAddFragment` · 004cd9a0 · legions 004c1f10
- [x] `NetBlob::ReceiveBlobReady` · 004cdab0 · legions 004c2020
- [x] `NetBlob::Send` · 004cdac0 · legions 004c2030
- [ ] `NetBlob::Pack` · 004cdb10
- [ ] `NetBlob::UnPack` · 004cdbe0 · legions 004c2150
- [x] `NetBlob::NetBlob` · 004cdcb0 · legions 004c2220
- [ ] `NetBlob::GetPackSize` · 004cdd50
- [x] `NetBlob::NetBlob` · 004cdd60 · legions 004c22d0
- [ ] `NetBlob::~NetBlob` · 004cddc0 · legions 004c2330
- [x] `NetBlob::Fragmentize` · 004cde10 · legions 004c2380

## `portal/shared/src/newnet/netblobid.cpp.c` (10/10)

### NetBlobIDUtils
- [x] `NetBlobIDUtils::IsCellFlagSet` · 0063a540 · legions 005ef190
- [x] `NetBlobIDUtils::IsEphemeralFlagSet` · 0063a560 · legions 005ef1b0
- [x] `NetBlobIDUtils::IsOutOfWorldFlagSet` · 0063a580 · legions 005ef1d0
- [x] `NetBlobIDUtils::GetOrderingType` · 0063a5a0 · legions 005ef1f0
- [x] `NetBlobIDUtils::GetOrderingStamp` · 0063a5b0 · legions 005ef200
- [x] `NetBlobIDUtils::GetSequenceID` · 0063a5c0 · legions 005ef210
- [x] `NetBlobIDUtils::MakeInitialSequenceID` · 0063a5d0 · legions 005ef220
- [x] `NetBlobIDUtils::MakeNetBlobID` · 0063a600 · legions 005ef250
- [x] `NetBlobIDUtils::NextNonEphemeralSequenceID` · 0063a660 · legions 005ef2b0
- [x] `NetBlobIDUtils::LHSNewerOrderingStamp` · 0063a690 · legions 005ef2e0

## `portal/shared/src/newnet/netinitializer.cpp.c` (0/3)

### NetInitializer
- [ ] `NetInitializer::NetInitializer` · 0063b5e0 · legions 005f0110
- [ ] `NetInitializer::~NetInitializer` · 0063b620 · legions 005f0150

### NetInitializer::AllSocketsCrashCleaner
- [ ] `NetInitializer::AllSocketsCrashCleaner::CrashCleanup` · 0063b630 · legions 005f0160

## `portal/shared/src/newnet/netinterface.cpp.c` (1/2)

### NetInterface
- [ ] `NetInterface::NetInterface` · 004cd980 · legions 004c1ef0
- [x] `NetInterface::GetNonEphemeralID` · 004cd990 · legions 004c1f00

## `portal/shared/src/newnet/netkeyexch.cpp.c` (0/5)

### NetKeyExch
- [ ] `NetKeyExch::SetPrivateRandom` · 004cd840 · legions 004c1db0
- [ ] `NetKeyExch::Pack` · 004cd860
- [ ] `NetKeyExch::UnPack` · 004cd8b0
- [ ] `NetKeyExch::NetKeyExch` · 004cd8f0 · legions 004c1e60
- [ ] `NetKeyExch::GetPackSize` · 004cd920

## `portal/shared/src/newnet/netlayerpacket.cpp.c` (0/3)

### CNetLayerPacket
- [ ] `CNetLayerPacket::CNetLayerPacket` · 0063b410 · legions 005eff40
- [ ] `CNetLayerPacket::Create` · 0063b4c0 · legions 005efff0
- [ ] `CNetLayerPacket::Clear` · 0063b4e0 · legions 005f0010

## `portal/shared/src/newnet/netpacket.cpp.c` (9/9)

### NetPacket
- [x] `NetPacket::ComputeChecksum` · 004cb7d0 · legions 004bfd10
- [x] `NetPacket::ApplySpecialFrags` · 004cb8a0 · legions 004bfde0
- [x] `NetPacket::AddToTail` · 004cb8d0 · legions 004bfe10
- [x] `NetPacket::AddToHead` · 004cb900 · legions 004bfe40
- [x] `NetPacket::Remove` · 004cb950 · legions 004bfe90
- [x] `NetPacket::AddOptionalHeader` · 004cb990 · legions 004bfed0
- [x] `NetPacket::RemoveDisposableOptionalHeaders` · 004cba40 · legions 004bff80
- [x] `NetPacket::UpdateTimeSensitiveHeaders` · 004cbb00 · legions 004c0040
- [x] `NetPacket::AddFrag` · 004cbb70 · legions 004c00b0

## `portal/shared/src/newnet/netutils.cpp.c` (0/3)

### NetUtils
- [ ] `NetUtils::GetSinAddr` · 0063b640 · legions 005f0170
- [ ] `NetUtils::GetIPString` · 0063b6c0 · legions 005f01f0
- [ ] `NetUtils::GetLastErrorString` · 0063b730 · legions 005f0260

## `portal/shared/src/newnet/optionalheader.cpp.c` (28/28)

### COptionalHeaderFactory
- [x] `COptionalHeaderFactory::RegisterOptionalHeaderType` · 0063b7e0 · legions 005f0310

### CBufferIterator
- [x] `CBufferIterator::ReadPrimArray` · 0063b830 · legions 005f0360

### COptionalHeaderFactory
- [x] `COptionalHeaderFactory::AddOptionalHeadersToPacket` · 0063b880 · legions 005f03b0

### CPackObjHeader<NetError,1048576,7>
- [x] `CPackObjHeader<NetError,1048576,7>::CPackObjHeader<NetError,1048576,7>` · 0063b900 · legions 005f0430

### COnePrimHeader<512,7,sockaddr_in>
- [x] `COnePrimHeader<512,7,sockaddr_in>::COnePrimHeader<512,7,sockaddr_in>` · 0063b990 · legions 005f04c0

### CPackObjHeader<NetError,2097152,2>
- [x] `CPackObjHeader<NetError,2097152,2>::CPackObjHeader<NetError,2097152,2>` · 0063b9e0 · legions 005f0510

### CTimeSyncHeader
- [x] `CTimeSyncHeader::CreateFromStream` · 0063ba70

### CEchoRequestHeader
- [x] `CEchoRequestHeader::CreateFromStream` · 0063baf0

### CEchoResponseHeader
- [x] `CEchoResponseHeader::CreateFromStream` · 0063bb50

### CConnectHeader
- [x] `CConnectHeader::CreateFromStream` · 0063bbe0

### COnePrimHeader<134217728,16,CFlowStruct>
- [x] `COnePrimHeader<134217728,16,CFlowStruct>::CreateFromStream` · 0063bc60

### CPackObjHeader<NetError,1048576,7>
- [x] `CPackObjHeader<NetError,1048576,7>::CreateFromStream` · 0063bcd0

### CSeqIDListHeader<4096,33>
- [x] `CSeqIDListHeader<4096,33>::CreateFromStream` · 0063bd00

### COnePrimHeader<16384,1,unsigned_long>
- [x] `COnePrimHeader<16384,1,unsigned_long>::CreateFromStream` · 0063bde0

### CSeqIDListHeader<8192,33>
- [x] `CSeqIDListHeader<8192,33>::CreateFromStream` · 0063be40

### COnePrimHeader<131072,7,unsigned___int64>
- [x] `COnePrimHeader<131072,7,unsigned___int64>::CreateFromStream` · 0063bf20

### CEmptyHeader<32768,3>
- [x] `CEmptyHeader<32768,3>::CreateFromStream` · 0063bf90

### CEmptyHeader<1024,7>
- [x] `CEmptyHeader<1024,7>::CreateFromStream` · 0063bfd0

### COnePrimHeader<524288,536870919,unsigned___int64>
- [x] `COnePrimHeader<524288,536870919,unsigned___int64>::CreateFromStream` · 0063c010

### COnePrimHeader<256,96,CServerSwitchStruct>
- [x] `COnePrimHeader<256,96,CServerSwitchStruct>::CreateFromStream` · 0063c080

### COnePrimHeader<512,7,sockaddr_in>
- [x] `COnePrimHeader<512,7,sockaddr_in>::CreateFromStream` · 0063c0f0

### COnePrimHeader<2048,1073741922,CReferralStruct>
- [x] `COnePrimHeader<2048,1073741922,CReferralStruct>::CreateFromStream` · 0063c150

### COnePrimHeader<4194304,7,CICMDCommandStruct>
- [x] `COnePrimHeader<4194304,7,CICMDCommandStruct>::CreateFromStream` · 0063c1d0

### CPackObjHeader<NetError,2097152,2>
- [x] `CPackObjHeader<NetError,2097152,2>::CreateFromStream` · 0063c240

### CLogonHeader
- [x] `CLogonHeader::CLogonHeader` · 0063c270 · legions 005f0dc0
- [x] `CLogonHeader::CLogonHeader` · 0063c2c0 · legions 005f0e10
- [x] `CLogonHeader::CreateFromData` · 0063c340 · legions 005f0e90
- [x] `CLogonHeader::CreateFromStream` · 0063c410

## `portal/shared/src/newnet/packetcontroller.cpp.c` (15/19)

### PacketController
- [x] `PacketController::Instance` · 004ca310 · legions 004be8f0
- [x] `PacketController::AddReceiveQueue` · 004ca320 · legions 004be900
- [x] `PacketController::EnqueueRecip` · 004ca340 · legions 004be920
- [x] `PacketController::SendBlob` · 004ca370 · legions 004be950
- [x] `PacketController::GetRecipient` · 004ca3a0 · legions 004be980
- [ ] `PacketController::CallNetHook` · 004ca3b0
- [ ] `PacketController::~PacketController` · 004ca530 · legions 004beaf0
- [x] `PacketController::GetNonEphemeralID` · 004ca5e0
- [x] `PacketController::EnqueueOptionalHeader` · 004ca620 · legions 004bebc0

### PQueueArray<double,void_*>
- [x] `PQueueArray<double,void_*>::Insert` · 004ca660 · legions 004bec00
- [x] `PQueueArray<double,void_*>::RemoveMin` · 004ca6e0 · legions 004bec80
- [x] `PQueueArray<double,void_*>::RemoveByData` · 004ca760 · legions 004bed00

### PacketController
- [x] `PacketController::CheckFlowControlTimers` · 004ca880 · legions 004bee20
- [x] `PacketController::UseTime` · 004ca960 · legions 004bef00
- [x] `PacketController::AddRecipient` · 004ca9c0 · legions 004bef60
- [x] `PacketController::DeleteRecipient` · 004caad0 · legions 004bf070

### (free functions)
- [ ] `GetIPFromRecipientID` · 004cab90 · legions 004bf130

### PacketController
- [x] `PacketController::AddReceivedBlobToQueue` · 004cabe0 · legions 004bf180
- [~] `PacketController::PacketController` · 004cacf0 · legions 004bf280 — net hook DLL and perf counters not loaded

## `portal/shared/src/newnet/receiverdata.cpp.c` (12/16)

### ReceiverData
- [x] `ReceiverData::SetConnectionState` · 004caed0 · legions 004bf410
- [x] `ReceiverData::GetConnectionStateDuration` · 004caef0 · legions 004bf430
- [x] `ReceiverData::AddToQueue` · 004caf00 · legions 004bf440
- [x] `ReceiverData::RemoveFromQueue` · 004caf30 · legions 004bf470
- [x] `ReceiverData::Encrypt` · 004cafb0 · legions 004bf4f0
- [x] `ReceiverData::Decrypt` · 004cafc0 · legions 004bf500

### AVLNode<unsigned_long,unsigned_long>
- [ ] `AVLNode<unsigned_long,unsigned_long>::AVLNode<unsigned_long,unsigned_long>` · 004cafd0 · legions 004bf510

### ReceiverData
- [x] `ReceiverData::InitCrypto` · 004cb010 · legions 004bf550
- [ ] `ReceiverData::ReceiverData` · 004cb0a0 · legions 004bf5e0
- [ ] `ReceiverData::~ReceiverData` · 004cb0e0 · legions 004bf620
- [x] `ReceiverData::SharedInit` · 004cb160 · legions 004bf6a0
- [x] `ReceiverData::Init` · 004cb210 · legions 004bf750
- [~] `ReceiverData::Clear` · 004cb270 · legions 004bf7b0 — slot interval and byte count kept; crypto/key exchange freed with the receiver
- [x] `ReceiverData::GetNaks` · 004cb330 · legions 004bf870

### AVL<unsigned_long,unsigned_long>
- [x] `AVL<unsigned_long,unsigned_long>::Add` · 004cb3b0 · legions 004bf8f0 — ReceiverData::m_SeqIDsWeNAKed as a BTreeMap: same key order for GetNaks, same add/remove results

### ReceiverData
- [x] `ReceiverData::AddNakked` · 004cb4b0 · legions 004bf9f0

## `portal/shared/src/newnet/recipientdata.cpp.c` (6/8)

### RecipientData
- [~] `RecipientData::TouchConnection` · 004cb520 · legions 004bfa60 — lastTouched_ not tracked (no reader in the client)
- [x] `RecipientData::IndicateReceivedPacket` · 004cb540 · legions 004bfa80
- [x] `RecipientData::EnqueueBlob` · 004cb570 · legions 004bfab0
- [x] `RecipientData::UseTime` · 004cb590 · legions 004bfad0
- [x] `RecipientData::RecipientData` · 004cb5d0 · legions 004bfb10
- [ ] `RecipientData::~RecipientData` · 004cb6a0 · legions 004bfbe0
- [x] `RecipientData::QueueFlush` · 004cb730 · legions 004bfc70
- [x] `RecipientData::ProcessNaks` · 004cb770 · legions 004bfcb0

## `portal/shared/src/newnet/sentpacket.cpp.c` (3/7)

### SentPacketStore
- [x] `SentPacketStore::Contains` · 004cea50 · legions 004c2fc0
- [x] `SentPacketStore::GenerateAck` · 004cea80 · legions 004c2ff0
- [ ] `SentPacketStore::~SentPacketStore` · 004ceab0 · legions 004c3080
- [~] `SentPacketStore::AddSentPacket` · 004ceb80 · legions 004c3150 — ephemeral blob-id bookkeeping (m_sentNetBlobIDInfo) not kept
- [ ] `SentPacketStore::EraseNetBlobID` · 004cecd0 · legions 004c32a0
- [x] `SentPacketStore::Flush` · 004ced50 · legions 004c3320
- [ ] `SentPacketStore::SentPacketStore` · 004cee30 · legions 004c3400

## `portal/shared/src/rpc/rpcserv.cpp.c` (0/13)

### RpcServer
- [ ] `RpcServer::RegisterInterface` · 0073c360
- [ ] `RpcServer::UnRegisterInterface` · 0073c3e0 · legions 006b8670
- [ ] `RpcServer::StartServer` · 0073c4a0
- [ ] `RpcServer::StopServer` · 0073c4f0 · legions 006b8730
- [ ] `RpcServer::BindOverNamedPipe` · 0073c580
- [ ] `RpcServer::BindOverLpc` · 0073c620
- [ ] `RpcServer::SetSecurityDescriptor` · 0073c6e0
- [ ] `RpcServer::RpcServer` · 0073c730
- [ ] `RpcServer::Initialize` · 0073c790
- [ ] `RpcServer::DynamicBindOverTcp` · 0073c7a0
- [ ] `RpcServer::DynamicBindOverSpx` · 0073c810
- [ ] `RpcServer::~RpcServer` · 0073c880 · legions 006b87c0
- [ ] `RpcServer::AddProtocol` · 0073c8f0

## `portal/shared/src/rpc/rpcutil.c.c` (0/2)

### (free functions)
- [ ] `MIDL_user_allocate` · 0073c950
- [ ] `MIDL_user_free` · 0073c970

## `portal/shared/src/vlong/vlong.cpp.c` (0/21)

### flex_unit
- [ ] `flex_unit::reserve` · 00672a00 · legions 0061cfc0
- [ ] `flex_unit::set` · 00672a60 · legions 0061d020

### (free functions)
- [ ] `do_inner` · 00672ae0 · legions 0061d0a0

### flex_unit
- [ ] `flex_unit::fast_mul` · 00672b20 · legions 0061d0e0

### vlong_value
- [ ] `vlong_value::bits` · 00672c20 · legions 0061d1e0
- [ ] `vlong_value::cf` · 00672c70 · legions 0061d230
- [ ] `vlong_value::add` · 00672d00 · legions 0061d2c0
- [ ] `vlong_value::subtract` · 00672d70 · legions 0061d330
- [ ] `vlong_value::copy` · 00672dd0 · legions 0061d390

### vlong
- [ ] `vlong::store` · 00672e10 · legions 0061d3d0
- [ ] `vlong::docopy` · 00672e40 · legions 0061d400
- [ ] `vlong::GetPackSize` · 00672e80
- [ ] `vlong::load` · 00672e90 · legions 0061d450
- [ ] `vlong::vlong` · 00672f00 · legions 0061d4c0
- [ ] `vlong::operator=` · 00672f50 · legions 0061d510
- [ ] `vlong::~vlong` · 00672fb0 · legions 0061d570
- [ ] `vlong::operator+=` · 00673010 · legions 0061d5d0

### (free functions)
- [ ] `operator*` · 006730f0 · legions 0061d6b0

### vlong
- [ ] `vlong::read_from_hex_string` · 00673190 · legions 0061d750
- [ ] `vlong::Pack` · 006734f0
- [ ] `vlong::UnPack` · 00673540 · legions 0061db10
