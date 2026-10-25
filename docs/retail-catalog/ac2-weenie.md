# ac2-weenie: Weenie objects, client events, actions, the session

1904 retail functions: 87 done, 26 partial, 1785 to do, 6 skipped.
Legend and workflow: [README](README.md).

## `ac2/client/src/cfactory/gmcenginefactory.cpp.c` (0/11)

### gmCEngineFactory
- [ ] `gmCEngineFactory::Init` · 004017e0 · legions 00401750
- [ ] `gmCEngineFactory::MakeClientCache_Internal` · 00401810
- [ ] `gmCEngineFactory::MakeUI_Internal` · 00401830

### NoticeHandler
- [ ] `NoticeHandler::RecvNotice_CharGenVerification` · 00401850 · legions 00401bb0
- [ ] `NoticeHandler::RecvNotice_DisplayAdminStringInfo` · 00401860 · legions 00401b50

### ArgumentParser
- [ ] `ArgumentParser::FinishOutputText` · 00401870 · legions 006af780

### NoticeHandler
- [ ] `NoticeHandler::RecvNotice_CharacterSet` · 00401880 · legions 00401ba0 — ac2-weenie SessionEvent::Characters / OnlineSession::characters (the latest set; CharacterChoice::Wait stops there for the selection screen)

### gmCEngineFactory
- [ ] `gmCEngineFactory::MakeClient_Internal` · 00401890
- [ ] `gmCEngineFactory::MakeCEventQueue_Internal` · 004018b0
- [ ] `gmCEngineFactory::MakeGlobalEventHandler_Internal` · 004018d0
- [ ] `gmCEngineFactory::MakeCObjectMaint_Internal` · 004018f0

## `ac2/client/src/events/cm_allegiance.cpp.c` (0/4)

### CM_Allegiance
- [ ] `CM_Allegiance::SendNotice_Patron_SwearDone` · 00727f80 — ac2-weenie GameplayNotice variant + Social/Progress state (adaptor shapes); raised by the script VM's glue, not by a stand-in
- [ ] `CM_Allegiance::SendNotice_RequestVassal` · 00728020 — ac2-weenie GameplayNotice variant + Social/Progress state (adaptor shapes); raised by the script VM's glue, not by a stand-in
- [ ] `CM_Allegiance::SendNotice_UpdateAllegianceProfile` · 007280c0 — ac2-weenie GameplayNotice variant + Social/Progress state (adaptor shapes); raised by the script VM's glue, not by a stand-in
- [ ] `CM_Allegiance::SendNotice_Vassal_SwearDone` · 00728150 — ac2-weenie GameplayNotice variant + Social/Progress state (adaptor shapes); raised by the script VM's glue, not by a stand-in

## `ac2/client/src/events/cm_character.cpp.c` (4/30)

### CM_Character
- [x] `CM_Character::SendNotice_AddMarker` · 007248b0 · legions 006a4270 — ac2-client client_radar.rs + ac2-script host/radar.rs (InqRadarLook), ac2-weenie notice.rs (ChangeRadarLook(obj))
- [ ] `CM_Character::SendNotice_AddPet` · 00724970 · legions 006a42e0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [x] `CM_Character::SendNotice_ChangeRadarLook` · 00724a30 — ac2-client client_radar.rs + ac2-script host/radar.rs (InqRadarLook), ac2-weenie notice.rs (ChangeRadarLook(obj))
- [ ] `CM_Character::SendNotice_ChangeRadarShowCoordsPreference` · 00724ac0
- [x] `CM_Character::SendNotice_ClearMarkers` · 00724b50 · legions 006a43e0 — ac2-client client_radar.rs + ac2-script host/radar.rs (InqRadarLook), ac2-weenie notice.rs (ChangeRadarLook(obj))
- [ ] `CM_Character::SendNotice_DisplayMessage` · 00724be0 · legions 006a4430 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Character::SendNotice_PetIsUnderAttack` · 00724c70 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Character::SendNotice_PulseMissed` · 00724d00 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Character::SendNotice_ReloadGameplayOptions` · 00724d90
- [ ] `CM_Character::SendNotice_ReloadOptions` · 00724e20 — ac2-weenie Progress stand-in: GameplayNotice::ReloadOptions / UpdateSkill / UpdateSkillPanel, set_shortcut (CPlayer 0x87)
- [x] `CM_Character::SendNotice_RemoveMarker` · 00724eb0 · legions 006a45e0 — ac2-client client_radar.rs + ac2-script host/radar.rs (InqRadarLook), ac2-weenie notice.rs (ChangeRadarLook(obj))
- [ ] `CM_Character::SendNotice_RemovePet` · 00724f50 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Character::SendNotice_ResurrectionRequestedByOther` · 00724fe0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Character::SendNotice_Select` · 00725070
- [ ] `CM_Character::SendNotice_SelectionChanged` · 00725100 — ac2-weenie GameplayNotice (plain-data notice raised by the PlayerScript stand-in)
- [ ] `CM_Character::SendNotice_SetShortcutHeight` · 00725190
- [ ] `CM_Character::SendNotice_UpdateDeathState` · 00725220 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Character::SendNotice_UpdateEffect` · 007252b0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Character::SendNotice_UpdateFactionStatus` · 00725340 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Character::SendNotice_UpdateHeroStatus` · 007253d0 — ac2-weenie GameplayNotice::UpdateVital / UpdateHeroStatus raised by the vitals stand-in (CPlayer 0x23/0x49/0x198/0x65/0x85); PlayerVitals on ClientFrame
- [ ] `CM_Character::SendNotice_UpdateMountState` · 00725490 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Character::SendNotice_UpdatePetMode` · 00725520 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Character::SendNotice_UpdatePetName` · 007255c0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Character::SendNotice_UpdateResetTime` · 00725660 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Character::SendNotice_UpdateSelectability` · 007256f0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Character::SendNotice_UpdateSelectionInfo` · 00725780 — ac2-weenie GameplayNotice (plain-data notice raised by the PlayerScript stand-in)
- [ ] `CM_Character::SendNotice_UpdateSkill` · 00725810 — ac2-weenie Progress stand-in: GameplayNotice::ReloadOptions / UpdateSkill / UpdateSkillPanel, set_shortcut (CPlayer 0x87)
- [ ] `CM_Character::SendNotice_UpdateSkillPanel` · 007258a0 — ac2-weenie Progress stand-in: GameplayNotice::ReloadOptions / UpdateSkill / UpdateSkillPanel, set_shortcut (CPlayer 0x87)
- [ ] `CM_Character::SendNotice_UpdateVitae` · 00725950 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Character::SendNotice_UpdateVital` · 007259f0 — ac2-weenie GameplayNotice::UpdateVital / UpdateHeroStatus raised by the vitals stand-in (CPlayer 0x23/0x49/0x198/0x65/0x85); PlayerVitals on ClientFrame

## `ac2/client/src/events/cm_combat.cpp.c` (0/5)

### CM_Combat
- [ ] `CM_Combat::SendNotice_EndSpecialAttack` · 00727cb0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Combat::SendNotice_RequestSpecialAttack` · 00727d40 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Combat::SendNotice_StartSpecialAttack` · 00727dd0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Combat::SendNotice_ToggleSpecialAttack` · 00727e60 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Combat::SendNotice_UpdateAttackState` · 00727ef0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host

## `ac2/client/src/events/cm_communication.cpp.c` (0/3)

### CM_Communication
- [ ] `CM_Communication::SendNotice_DisplayStringInfo` · 00725cc0 — ac2-weenie GameplayNotice::DisplayStringInfo (chat lines from the CCommunicationSystem stand-in)
- [ ] `CM_Communication::SendNotice_DoAssistedTell` · 00725d50 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Communication::SendNotice_ReloadChatSettings` · 00725de0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host

## `ac2/client/src/events/cm_craft.cpp.c` (0/6)

### CM_Craft
- [ ] `CM_Craft::SendNotice_AddIngredient_Done` · 00727900 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Craft::SendNotice_ExecuteCraft_Done` · 007279d0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Craft::SendNotice_OpenIngredientPanel` · 00727a60 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Craft::SendNotice_UpdateCost` · 00727af0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Craft::SendNotice_UpdateCraftPanel` · 00727b90 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Craft::SendNotice_UpdateCraftSkill` · 00727c20 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host

## `ac2/client/src/events/cm_examination.cpp.c` (0/2)

### CM_Examination
- [ ] `CM_Examination::SendNotice_ExaminationRequestChanged` · 007277e0 — ac2-weenie GameplayNotice (plain-data notice raised by the PlayerScript stand-in)
- [ ] `CM_Examination::SendNotice_UpdateExaminationProfile` · 00727870 — ac2-weenie GameplayNotice (plain-data notice raised by the PlayerScript stand-in)

## `ac2/client/src/events/cm_fellowship.cpp.c` (0/13)

### CM_Fellowship
- [ ] `CM_Fellowship::SendNotice_ClearFellow` · 00726fe0 — ac2-weenie GameplayNotice variant + Social/Progress state (adaptor shapes); raised by the script VM's glue, not by a stand-in
- [ ] `CM_Fellowship::SendNotice_ClearFellowship` · 00727070 — ac2-weenie GameplayNotice variant + Social/Progress state (adaptor shapes); raised by the script VM's glue, not by a stand-in
- [ ] `CM_Fellowship::SendNotice_ExpireRecruitment` · 00727100 — ac2-weenie GameplayNotice variant + Social/Progress state (adaptor shapes); raised by the script VM's glue, not by a stand-in
- [ ] `CM_Fellowship::SendNotice_RequestRecruitment` · 00727190 — ac2-weenie GameplayNotice variant + Social/Progress state (adaptor shapes); raised by the script VM's glue, not by a stand-in
- [ ] `CM_Fellowship::SendNotice_UpdateFellow` · 00727240 — ac2-weenie GameplayNotice variant + Social/Progress state (adaptor shapes); raised by the script VM's glue, not by a stand-in
- [ ] `CM_Fellowship::SendNotice_UpdateFellowCellID` · 007272e0 — ac2-weenie GameplayNotice variant + Social/Progress state (adaptor shapes); raised by the script VM's glue, not by a stand-in
- [ ] `CM_Fellowship::SendNotice_UpdateFellowHealth` · 00727380 — ac2-weenie GameplayNotice variant + Social/Progress state (adaptor shapes); raised by the script VM's glue, not by a stand-in
- [ ] `CM_Fellowship::SendNotice_UpdateFellowLevel` · 00727430 — ac2-weenie GameplayNotice variant + Social/Progress state (adaptor shapes); raised by the script VM's glue, not by a stand-in
- [ ] `CM_Fellowship::SendNotice_UpdateFellowMaxHealth` · 007274d0 — ac2-weenie GameplayNotice variant + Social/Progress state (adaptor shapes); raised by the script VM's glue, not by a stand-in
- [ ] `CM_Fellowship::SendNotice_UpdateFellowMaxVigor` · 00727570 — ac2-weenie GameplayNotice variant + Social/Progress state (adaptor shapes); raised by the script VM's glue, not by a stand-in
- [ ] `CM_Fellowship::SendNotice_UpdateFellowVigor` · 00727610 — ac2-weenie GameplayNotice variant + Social/Progress state (adaptor shapes); raised by the script VM's glue, not by a stand-in
- [ ] `CM_Fellowship::SendNotice_UpdateFellowship` · 007276c0 — ac2-weenie GameplayNotice variant + Social/Progress state (adaptor shapes); raised by the script VM's glue, not by a stand-in
- [ ] `CM_Fellowship::SendNotice_UpdateLeader` · 00727750 — ac2-weenie GameplayNotice variant + Social/Progress state (adaptor shapes); raised by the script VM's glue, not by a stand-in

## `ac2/client/src/events/cm_friends.cpp.c` (0/5)

### CM_Friends
- [ ] `CM_Friends::SendNotice_GUNResult_AddFriend` · 00725e70 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Friends::SendNotice_GUNResult_FriendStatus` · 00725f00 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Friends::SendNotice_GUNResult_RemoveFriend` · 00725fb0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Friends::SendNotice_LocalStatus` · 00726040 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Friends::SendNotice_GUNResult_GetFriendsList` · 007260d0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host

## `ac2/client/src/events/cm_inventory.cpp.c` (0/4)

### CM_Inventory
- [ ] `CM_Inventory::SendNotice_ContainerStateUpdate` · 00726c10 — ac2-weenie GameplayNotice::ContainerStateUpdate / ItemUpdate raised by the InventoryState stand-in (CInventorySystem 0xF/0x1B, CItem 0x3)
- [ ] `CM_Inventory::SendNotice_ItemUpdate` · 00726cb0 — ac2-weenie GameplayNotice::ContainerStateUpdate / ItemUpdate raised by the InventoryState stand-in (CInventorySystem 0xF/0x1B, CItem 0x3)
- [ ] `CM_Inventory::SendNotice_UpdateContainer` · 00726d40 — ac2-weenie GameplayNotice::UpdateContainer / UpdateInventory raised by the InventoryState stand-in
- [ ] `CM_Inventory::SendNotice_UpdateInventory` · 00726e10 — ac2-weenie GameplayNotice::UpdateContainer / UpdateInventory raised by the InventoryState stand-in

## `ac2/client/src/events/cm_money.cpp.c` (0/2)

### CM_Money
- [ ] `CM_Money::SendNotice_GhostMoneyBag` · 00726ec0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Money::SendNotice_UpdateMoney` · 00726f50 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host

## `ac2/client/src/events/cm_physics.cpp.c` (0/2)

### CM_Physics
- [ ] `CM_Physics::RecvEvt_CreateObject` · 007269d0 · legions 006a55c0
- [ ] `CM_Physics::SendNotice_Do3DDamage` · 00726b80 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host

## `ac2/client/src/events/cm_quest.cpp.c` (0/3)

### CM_Quest
- [ ] `CM_Quest::SendNotice_UpdateQuests` · 00725ab0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Quest::SendNotice_UpdateStoryQuests` · 00725b40 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Quest::SendNotice_PlaySequence` · 00725bd0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host

## `ac2/client/src/events/cm_shortcut.cpp.c` (0/2)

### CM_Shortcut
- [ ] `CM_Shortcut::SendNotice_ReloadShortcuts` · 007268b0 — ac2-weenie GameplayNotice variant + Social/Progress state (adaptor shapes); raised by the script VM's glue, not by a stand-in
- [ ] `CM_Shortcut::SendNotice_RemoveRecipe` · 00726940

## `ac2/client/src/events/cm_store.cpp.c` (0/12)

### CM_Store
- [ ] `CM_Store::SendNotice_ConfirmPurchase` · 00724000 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Store::SendNotice_InitNewSale` · 007240b0
- [ ] `CM_Store::SendNotice_LeaveCatalog` · 00724150 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Store::SendNotice_LeaveConsignment` · 007241e0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Store::SendNotice_LeaveStore` · 00724270 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Store::SendNotice_ShowSaleReminder` · 00724300 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Store::SendNotice_UpdateSale` · 00724390 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Store::SendNotice_UpdateConsignment` · 00724430 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Store::SendNotice_UpdateStore` · 00724520 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Store::SendNotice_ViewConsignment` · 00724610 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Store::SendNotice_ViewStore` · 00724700 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Store::SendNotice_ViewCatalog` · 007247f0 · legions 006a41f0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host

## `ac2/client/src/events/cm_trade.cpp.c` (0/9)

### CM_Trade
- [~] `CM_Trade::SendNotice_Trade_BeAcceptanceCleared` · 00726310 — raised by the stand-in from WM_Trade handlers (ac2-net gameplay/trade.rs, ac2-weenie session/trade.rs); handler ids traced through CPlayer, arg kinds from the notice signatures; script VM path not wired
- [~] `CM_Trade::SendNotice_Trade_BeAccepted` · 007263a0 — raised by the stand-in from WM_Trade handlers (ac2-net gameplay/trade.rs, ac2-weenie session/trade.rs); handler ids traced through CPlayer, arg kinds from the notice signatures; script VM path not wired
- [~] `CM_Trade::SendNotice_Trade_BeClosed` · 00726430 — raised by the stand-in from WM_Trade handlers (ac2-net gameplay/trade.rs, ac2-weenie session/trade.rs); handler ids traced through CPlayer, arg kinds from the notice signatures; script VM path not wired
- [~] `CM_Trade::SendNotice_Trade_BeDeclined` · 007264d0 — raised by the stand-in from WM_Trade handlers (ac2-net gameplay/trade.rs, ac2-weenie session/trade.rs); handler ids traced through CPlayer, arg kinds from the notice signatures; script VM path not wired
- [~] `CM_Trade::SendNotice_Trade_BeOffered` · 00726560 — raised by the stand-in from WM_Trade handlers (ac2-net gameplay/trade.rs, ac2-weenie session/trade.rs); handler ids traced through CPlayer, arg kinds from the notice signatures; script VM path not wired
- [~] `CM_Trade::SendNotice_Trade_BeOpened` · 00726620 — raised by the stand-in from WM_Trade handlers (ac2-net gameplay/trade.rs, ac2-weenie session/trade.rs); handler ids traced through CPlayer, arg kinds from the notice signatures; script VM path not wired
- [~] `CM_Trade::SendNotice_Trade_BeRegistered` · 007266b0 — raised by the stand-in from WM_Trade handlers (ac2-net gameplay/trade.rs, ac2-weenie session/trade.rs); handler ids traced through CPlayer, arg kinds from the notice signatures; script VM path not wired
- [~] `CM_Trade::SendNotice_Trade_BeReset` · 00726770 — raised by the stand-in from WM_Trade handlers (ac2-net gameplay/trade.rs, ac2-weenie session/trade.rs); handler ids traced through CPlayer, arg kinds from the notice signatures; script VM path not wired
- [~] `CM_Trade::SendNotice_Trade_BeRevoked` · 00726800 — raised by the stand-in from WM_Trade handlers (ac2-net gameplay/trade.rs, ac2-weenie session/trade.rs); handler ids traced through CPlayer, arg kinds from the notice signatures; script VM path not wired

## `ac2/client/src/events/cm_ui.cpp.c` (0/21)

### CM_UI
- [ ] `CM_UI::SendNotice_CloseConfirmationDialog` · 007232e0
- [ ] `CM_UI::SendNotice_CloseMessageDialog` · 00723380
- [ ] `CM_UI::SendNotice_CloseResurrectDialog` · 00723410
- [ ] `CM_UI::SendNotice_CloseStackUI` · 007234a0 · legions 006a36a0
- [ ] `CM_UI::SendNotice_EndCharacterSession` · 00723530
- [ ] `CM_UI::SendNotice_GunAuthenticated` · 007235c0
- [ ] `CM_UI::SendNotice_GunConnected` · 00723650
- [ ] `CM_UI::SendNotice_GunConnectionLost` · 007236e0
- [ ] `CM_UI::SendNotice_GunReconnected` · 00723770
- [ ] `CM_UI::SendNotice_HandleDragFrom` · 00723800 · legions 006a3880
- [ ] `CM_UI::SendNotice_InitStackUI` · 007238a0 · legions 006a38e0
- [ ] `CM_UI::SendNotice_Logoff` · 00723980
- [ ] `CM_UI::SendNotice_OpenExaminationPanel` · 00723a10 · legions 006a39b0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_UI::SendNotice_PortalSpace` · 00723aa0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_UI::SendNotice_PrecludeFromBottom` · 00723b30
- [ ] `CM_UI::SendNotice_PrecludeFromTop` · 00723bc0
- [ ] `CM_UI::SendNotice_SaveLocationsChanged` · 00723c50 · legions 006a3af0
- [ ] `CM_UI::SendNotice_StackUIDone` · 00723ce0 · legions 006a3b40
- [ ] `CM_UI::SendNotice_UpdateCanMouseLook` · 00723d80 · legions 006a3ba0
- [ ] `CM_UI::SendNotice_FinishPlaySequence` · 00723e10
- [ ] `CM_UI::SendNotice_PortalScene` · 00723f00 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host

## `ac2/client/src/events/cm_usage.cpp.c` (0/2)

### CM_Usage
- [ ] `CM_Usage::SendNotice_SetTargetingPointer` · 007261c0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CM_Usage::SendNotice_UseBook` · 00726250 · legions 006a5160 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host

## `ac2/client/src/events/gmceventqueue.cpp.c` (0/2)

### gmCEventQueue
- [ ] `gmCEventQueue::gmCEventQueue` · 00723260
- [ ] `gmCEventQueue::DispatchEvent` · 00723280 · legions 006a3540

## `ac2/client/src/events/gmglobaleventhandler.cpp.c` (0/2)

### gmGlobalEventHandler
- [ ] `gmGlobalEventHandler::gmGlobalEventHandler` · 007232c0 · legions 006a3580
- [ ] `gmGlobalEventHandler::gmGetGlobalEventHandler` · 007232d0 · legions 006a3590

## `ac2/client/src/weenie/interp/glue/gallegiancedataadaptor.cpp.c` (10/10)

### gAllegianceDataAdaptor
- [x] `gAllegianceDataAdaptor::CallEFun_SetFaction` · 0046a980 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)
- [x] `gAllegianceDataAdaptor::CallEFun_SetGender` · 0046a9e0 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)
- [x] `gAllegianceDataAdaptor::CallEFun_SetID` · 0046aa40 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)
- [x] `gAllegianceDataAdaptor::CallEFun_SetLevel` · 0046aaa0 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)
- [x] `gAllegianceDataAdaptor::CallEFun_SetName` · 0046ab00 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)
- [x] `gAllegianceDataAdaptor::CallEFun_SetOfficer` · 0046ab60 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)
- [x] `gAllegianceDataAdaptor::CallEFun_SetRank` · 0046abc0 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)
- [x] `gAllegianceDataAdaptor::CallEFun_SetSpecies` · 0046ac20 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)
- [x] `gAllegianceDataAdaptor::CallEFun_SetTitle` · 0046ac80 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)
- [x] `gAllegianceDataAdaptor::CallEFun_SetXPPassed` · 0046ace0 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)

## `ac2/client/src/weenie/interp/glue/gallegianceprofileadaptor.cpp.c` (9/10)

### gAllegianceProfileAdaptor
- [x] `gAllegianceProfileAdaptor::CallEFun_SetAllegianceName` · 0046a490 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)
- [x] `gAllegianceProfileAdaptor::CallEFun_SetFactionType` · 0046a4f0 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)
- [x] `gAllegianceProfileAdaptor::CallEFun_SetMember` · 0046a550 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)
- [x] `gAllegianceProfileAdaptor::CallEFun_SetMonarch` · 0046a5c0 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)
- [x] `gAllegianceProfileAdaptor::CallEFun_SetMotD` · 0046a630 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)
- [x] `gAllegianceProfileAdaptor::CallEFun_SetPatron` · 0046a6a0 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)
- [x] `gAllegianceProfileAdaptor::CallEFun_SetTotal` · 0046a710 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)
- [x] `gAllegianceProfileAdaptor::CallEFun_AddVassal` · 0046a7f0 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)

### HashSet<InstanceID>
- [ ] `HashSet<InstanceID>::add` · 0046a890 · legions 0046b170

### gAllegianceProfileAdaptor
- [x] `gAllegianceProfileAdaptor::CallEFun_AddOfficer` · 0046a900 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)

## `ac2/client/src/weenie/interp/glue/gcm_allegiance.cpp.c` (0/4)

### gCM_Allegiance
- [ ] `gCM_Allegiance::CallEFun_SendNotice_Patron_SwearDone` · 0046a340 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Allegiance::CallEFun_SendNotice_RequestVassal` · 0046a390 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Allegiance::CallEFun_SendNotice_UpdateAllegianceProfile` · 0046a3f0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Allegiance::CallEFun_SendNotice_Vassal_SwearDone` · 0046a440 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)

## `ac2/client/src/weenie/interp/glue/gcm_character.cpp.c` (4/26)

### gCM_Character
- [x] `gCM_Character::CallEFun_SendNotice_AddMarker` · 00469a90 — ac2-client client_radar.rs + ac2-script host/radar.rs (InqRadarLook), ac2-weenie notice.rs (ChangeRadarLook(obj))
- [ ] `gCM_Character::CallEFun_SendNotice_AddPet` · 00469b10 · legions 0046a500 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [x] `gCM_Character::CallEFun_SendNotice_ChangeRadarLook` · 00469b80 — ac2-client client_radar.rs + ac2-script host/radar.rs (InqRadarLook), ac2-weenie notice.rs (ChangeRadarLook(obj))
- [x] `gCM_Character::CallEFun_SendNotice_ClearMarkers` · 00469bc0 — ac2-client client_radar.rs + ac2-script host/radar.rs (InqRadarLook), ac2-weenie notice.rs (ChangeRadarLook(obj))
- [ ] `gCM_Character::CallEFun_SendNotice_DisplayMessage` · 00469c10 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Character::CallEFun_SendNotice_PetIsUnderAttack` · 00469c60 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Character::CallEFun_SendNotice_PulseMissed` · 00469cb0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Character::CallEFun_SendNotice_ReloadOptions` · 00469d00 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [x] `gCM_Character::CallEFun_SendNotice_RemoveMarker` · 00469d40 — ac2-client client_radar.rs + ac2-script host/radar.rs (InqRadarLook), ac2-weenie notice.rs (ChangeRadarLook(obj))
- [ ] `gCM_Character::CallEFun_SendNotice_RemovePet` · 00469d90 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Character::CallEFun_SendNotice_ResurrectionRequestedByOther` · 00469de0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Character::CallEFun_SendNotice_SelectionChanged` · 00469e30 — ac2-script host notices -> ac2_weenie::GameplayNotice
- [ ] `gCM_Character::CallEFun_SendNotice_UpdateDeathState` · 00469e80 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Character::CallEFun_SendNotice_UpdateEffect` · 00469ed0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Character::CallEFun_SendNotice_UpdateFactionStatus` · 00469f10 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Character::CallEFun_SendNotice_UpdateHeroStatus` · 00469f60 — ac2-script host notices -> ac2_weenie::GameplayNotice
- [ ] `gCM_Character::CallEFun_SendNotice_UpdateMountState` · 00469fd0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Character::CallEFun_SendNotice_UpdatePetMode` · 0046a020 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Character::CallEFun_SendNotice_UpdatePetName` · 0046a070 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Character::CallEFun_SendNotice_UpdateResetTime` · 0046a0d0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Character::CallEFun_SendNotice_UpdateSelectability` · 0046a110 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Character::CallEFun_SendNotice_UpdateSelectionInfo` · 0046a160 — ac2-script host notices -> ac2_weenie::GameplayNotice
- [ ] `gCM_Character::CallEFun_SendNotice_UpdateSkill` · 0046a1b0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Character::CallEFun_SendNotice_UpdateSkillPanel` · 0046a200 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Character::CallEFun_SendNotice_UpdateVitae` · 0046a260 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Character::CallEFun_SendNotice_UpdateVital` · 0046a2d0 — ac2-script host notices -> ac2_weenie::GameplayNotice

## `ac2/client/src/weenie/interp/glue/gcm_combat.cpp.c` (0/5)

### gCM_Combat
- [ ] `gCM_Combat::CallEFun_SendNotice_EndSpecialAttack` · 00469900 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Combat::CallEFun_SendNotice_RequestSpecialAttack` · 00469950 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Combat::CallEFun_SendNotice_StartSpecialAttack` · 004699a0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Combat::CallEFun_SendNotice_ToggleSpecialAttack` · 004699f0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Combat::CallEFun_SendNotice_UpdateAttackState` · 00469a40 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)

## `ac2/client/src/weenie/interp/glue/gcm_communication.cpp.c` (0/3)

### gCM_Communication
- [ ] `gCM_Communication::CallEFun_SendNotice_DisplayStringInfo` · 00469820 — ac2-script host notices -> ac2_weenie::GameplayNotice
- [ ] `gCM_Communication::CallEFun_SendNotice_DoAssistedTell` · 00469870 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Communication::CallEFun_SendNotice_ReloadChatSettings` · 004698c0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)

## `ac2/client/src/weenie/interp/glue/gcm_craft.cpp.c` (0/6)

### gCM_Craft
- [ ] `gCM_Craft::CallEFun_SendNotice_AddIngredient_Done` · 00469620 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Craft::CallEFun_SendNotice_ExecuteCraft_Done` · 004696a0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Craft::CallEFun_SendNotice_OpenIngredientPanel` · 004696f0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Craft::CallEFun_SendNotice_UpdateCost` · 00469740 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Craft::CallEFun_SendNotice_UpdateCraftPanel` · 00469790 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Craft::CallEFun_SendNotice_UpdateCraftSkill` · 004697d0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)

## `ac2/client/src/weenie/interp/glue/gcm_examination.cpp.c` (0/2)

### gCM_Examination
- [ ] `gCM_Examination::CallEFun_SendNotice_ExaminationRequestChanged` · 00469580 — ac2-script host notices -> ac2_weenie::GameplayNotice
- [ ] `gCM_Examination::CallEFun_SendNotice_UpdateExaminationProfile` · 004695d0 — ac2-script host (ScriptHost: process creation, dispatch, SendServerEvt, notices)

## `ac2/client/src/weenie/interp/glue/gcm_fellowship.cpp.c` (0/13)

### gCM_Fellowship
- [ ] `gCM_Fellowship::CallEFun_SendNotice_ClearFellow` · 00469130 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Fellowship::CallEFun_SendNotice_ClearFellowship` · 00469180 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Fellowship::CallEFun_SendNotice_ExpireRecruitment` · 004691c0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Fellowship::CallEFun_SendNotice_RequestRecruitment` · 00469210 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Fellowship::CallEFun_SendNotice_UpdateFellow` · 00469280 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Fellowship::CallEFun_SendNotice_UpdateFellowCellID` · 004692e0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Fellowship::CallEFun_SendNotice_UpdateFellowHealth` · 00469330 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Fellowship::CallEFun_SendNotice_UpdateFellowLevel` · 00469390 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Fellowship::CallEFun_SendNotice_UpdateFellowMaxHealth` · 004693e0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Fellowship::CallEFun_SendNotice_UpdateFellowMaxVigor` · 00469430 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Fellowship::CallEFun_SendNotice_UpdateFellowVigor` · 00469480 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Fellowship::CallEFun_SendNotice_UpdateFellowship` · 004694e0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Fellowship::CallEFun_SendNotice_UpdateLeader` · 00469530 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)

## `ac2/client/src/weenie/interp/glue/gcm_friends.cpp.c` (0/6)

### gCM_Friends
- [ ] `gCM_Friends::CallEFun_SendNotice_GUNResult_AddFriend` · 00467f60 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Friends::CallEFun_SendNotice_GUNResult_FriendStatus` · 00467fb0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Friends::CallEFun_SendNotice_GUNResult_RemoveFriend` · 00468010 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Friends::CallEFun_SendNotice_LocalStatus` · 00468060 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Friends::RListToGUNUserStatusList` · 004680b0 · legions 00468d60
- [ ] `gCM_Friends::CallEFun_SendNotice_GUNResult_GetFriendsList` · 00468150 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)

## `ac2/client/src/weenie/interp/glue/gcm_inventory.cpp.c` (0/4)

### gCM_Inventory
- [ ] `gCM_Inventory::CallEFun_SendNotice_ContainerStateUpdate` · 00468f10 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Inventory::CallEFun_SendNotice_ItemUpdate` · 00468f70 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Inventory::CallEFun_SendNotice_UpdateContainer` · 00468fb0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Inventory::CallEFun_SendNotice_UpdateInventory` · 00469030 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)

## `ac2/client/src/weenie/interp/glue/gcm_money.cpp.c` (0/2)

### gCM_Money
- [ ] `gCM_Money::CallEFun_SendNotice_GhostMoneyBag` · 00469090 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Money::CallEFun_SendNotice_UpdateMoney` · 004690e0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)

## `ac2/client/src/weenie/interp/glue/gcm_physics.cpp.c` (0/1)

### gCM_Physics
- [ ] `gCM_Physics::CallEFun_SendNotice_Do3DDamage` · 00468ec0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)

## `ac2/client/src/weenie/interp/glue/gcm_quest.cpp.c` (0/3)

### gCM_Quest
- [ ] `gCM_Quest::CallEFun_SendNotice_UpdateQuests` · 00468db0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Quest::CallEFun_SendNotice_UpdateStoryQuests` · 00468e00 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Quest::CallEFun_SendNotice_PlaySequence` · 00468e50 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)

## `ac2/client/src/weenie/interp/glue/gcm_shortcut.cpp.c` (0/2)

### gCM_Shortcut
- [ ] `gCM_Shortcut::CallEFun_SendNotice_ReloadShortcuts` · 00468d20 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Shortcut::CallEFun_SendNotice_RemoveRecipe` · 00468d60

## `ac2/client/src/weenie/interp/glue/gcm_store.cpp.c` (0/12)

### gCM_Store
- [ ] `gCM_Store::CallEFun_SendNotice_ConfirmPurchase` · 00467a50 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Store::CallEFun_SendNotice_LeaveCatalog` · 00467ac0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Store::CallEFun_SendNotice_LeaveConsignment` · 00467b00 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Store::CallEFun_SendNotice_LeaveStore` · 00467b40 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Store::CallEFun_SendNotice_ShowSaleReminder` · 00467b80 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Store::CallEFun_SendNotice_UpdateSale` · 00467bd0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Store::CallEFun_SendNotice_UpdateStore` · 00467c20 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Store::CallEFun_SendNotice_ViewConsignment` · 00467c80 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Store::CallEFun_SendNotice_ViewStore` · 00467ce0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Store::CallEFun_SendNotice_UpdateConsignment` · 00467d40 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Store::RListToUIShopList` · 00467e20 · legions 00468ad0
- [ ] `gCM_Store::CallEFun_SendNotice_ViewCatalog` · 00467ec0 · legions 00468b70 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)

## `ac2/client/src/weenie/interp/glue/gcm_trade.cpp.c` (0/9)

### gCM_Trade
- [ ] `gCM_Trade::CallEFun_SendNotice_Trade_BeAcceptanceCleared` · 00468a10 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Trade::CallEFun_SendNotice_Trade_BeAccepted` · 00468a50 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Trade::CallEFun_SendNotice_Trade_BeClosed` · 00468aa0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Trade::CallEFun_SendNotice_Trade_BeDeclined` · 00468af0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Trade::CallEFun_SendNotice_Trade_BeOffered` · 00468b40 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Trade::CallEFun_SendNotice_Trade_BeOpened` · 00468bb0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Trade::CallEFun_SendNotice_Trade_BeRegistered` · 00468bf0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Trade::CallEFun_SendNotice_Trade_BeReset` · 00468c70 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Trade::CallEFun_SendNotice_Trade_BeRevoked` · 00468cb0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)

## `ac2/client/src/weenie/interp/glue/gcm_ui.cpp.c` (0/3)

### gCM_UI
- [ ] `gCM_UI::CallEFun_SendNotice_OpenExaminationPanel` · 00468940 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_UI::CallEFun_SendNotice_PortalSpace` · 00468970 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_UI::CallEFun_SendNotice_PortalScene` · 004689a0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)

## `ac2/client/src/weenie/interp/glue/gcm_usage.cpp.c` (0/2)

### gCM_Usage
- [ ] `gCM_Usage::CallEFun_SendNotice_SetTargetingPointer` · 00468870 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCM_Usage::CallEFun_SendNotice_UseBook` · 004688c0 · legions 00469340 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)

## `ac2/client/src/weenie/interp/glue/gfellowadaptor.cpp.c` (9/9)

### gFellowAdaptor
- [x] `gFellowAdaptor::CallEFun_SetCell` · 00468510 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)
- [x] `gFellowAdaptor::CallEFun_SetHealth` · 00468570 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)
- [x] `gFellowAdaptor::CallEFun_SetLevel` · 004685d0 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)
- [x] `gFellowAdaptor::CallEFun_SetMaxHealth` · 00468630 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)
- [x] `gFellowAdaptor::CallEFun_SetMaxVigor` · 00468690 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)
- [x] `gFellowAdaptor::CallEFun_SetName` · 004686f0 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)
- [x] `gFellowAdaptor::CallEFun_SetPKDamage` · 00468750 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)
- [x] `gFellowAdaptor::CallEFun_SetPKVigorloss` · 004687b0 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)
- [x] `gFellowAdaptor::CallEFun_SetVigor` · 00468810 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)

## `ac2/client/src/weenie/interp/glue/gfellowshipadaptor.cpp.c` (6/6)

### gFellowshipAdaptor
- [x] `gFellowshipAdaptor::CallEFun_SetLeader` · 004682c0 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)
- [x] `gFellowshipAdaptor::CallEFun_SetLootingMethod` · 00468320 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)
- [x] `gFellowshipAdaptor::CallEFun_SetSocial` · 00468380 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)

### FellowshipAdaptor
- [x] `FellowshipAdaptor::SetName` · 004683e0 · legions 00468fa0 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)

### gFellowshipAdaptor
- [x] `gFellowshipAdaptor::CallEFun_AddFellow` · 00468430 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)
- [x] `gFellowshipAdaptor::CallEFun_SetName` · 004684b0 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)

## `ac2/client/src/weenie/interp/glue/ggmcobjmaint.cpp.c` (0/3)

### ggmCObjMaint
- [ ] `ggmCObjMaint::CallEFun_GetPlayerID` · 00468210 — ac2-script host (ScriptHost: process creation, dispatch, SendServerEvt, notices)
- [ ] `ggmCObjMaint::CallEFun_GetPlayerWeenieObjectReference` · 00468250 · legions 00468f00 — ac2-script host (ScriptHost: process creation, dispatch, SendServerEvt, notices)
- [ ] `ggmCObjMaint::CallEFun_DumpToString` · 00468280

## `ac2/client/src/weenie/interp/glue/ggmcwinterface.cpp.c` (8/301)

### PackObj
- [ ] `PackObj::GET_SIZE_LEFT` · 00454060 · legions 004550e0

### HeapObject
- [ ] `HeapObject::InitializeData` · 00454080 · legions 00455100 — ac2-script vm (VM/ReferenceTable/Resolver)

### VMMon
- [ ] `VMMon::iBegin` · 004540e0
- [ ] `VMMon::iEnd` · 00454140

### ReferenceTable
- [ ] `ReferenceTable::GetTimeSinceLastGCRun` · 00454220
- [ ] `ReferenceTable::GetTimeSinceLastGrow` · 00454250

### HeapObject
- [ ] `HeapObject::operator_new` · 004542a0 · legions 004551b0
- [ ] `HeapObject::HeapObject` · 004542d0 · legions 004551e0

### WDataMapper
- [ ] `WDataMapper::WStateIDToPackageID` · 00454300 · legions 00455210 — ac2-script vm (VM/ReferenceTable/Resolver)

### ReferenceTable
- [ ] `ReferenceTable::MarkReference` · 00454350 · legions 00455260

### ContextState
- [ ] `ContextState::Init` · 00454440 · legions 00455320 — ac2-script vm (VM/ReferenceTable/Resolver)

### ggmCWInterface
- [ ] `ggmCWInterface::CallEFun_PopupYesNoDialog` · 004544b0

### ReferenceEntry
- [ ] `ReferenceEntry::GetPackageStateID` · 00454870

### ReferenceTable
- [ ] `ReferenceTable::FindNextReference` · 00454890 · legions 004556f0
- [ ] `ReferenceTable::ScanReservedRefs` · 004548d0 · legions 00455730
- [ ] `ReferenceTable::MarkAndSweepBuffer` · 00454900 · legions 00455760
- [ ] `ReferenceTable::DeleteAllUnUsedReferences` · 00454960 · legions 004557c0

### ggmCWInterface
- [ ] `ggmCWInterface::CallEFun_RequestGUN_CallCreate` · 004549f0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `ggmCWInterface::CallEFun_RequestGUN_FriendAdd` · 00454a90 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `ggmCWInterface::CallEFun_RequestGUN_FriendDelete` · 00454b10 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `ggmCWInterface::CallEFun_RequestGUN_GetFriendsList` · 00454b90 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `ggmCWInterface::CallEFun_RequestGUN_GetIgnoredList` · 00454c10 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `ggmCWInterface::CallEFun_RequestGUN_IgnoredAdd` · 00454c90 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `ggmCWInterface::CallEFun_RequestGUN_IgnoredDelete` · 00454d10 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `ggmCWInterface::CallEFun_RequestGUN_SendToRoom` · 00454d90 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `ggmCWInterface::CallEFun_RequestGUN_SendToUser` · 00454e20 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `ggmCWInterface::CallEFun_RequestGUN_SetLocation` · 00454eb0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `ggmCWInterface::CallEFun_RequestGUN_SetStateFlags` · 00454f30 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)

### VM
- [ ] `VM::DeleteContext` · 00455000 · legions 00455c70

### Resolver
- [ ] `Resolver::GetVirtualFunctionOffset` · 00455060 · legions 00455cd0 — ac2-script vm (VM/ReferenceTable/Resolver)

### ReferenceTable
- [ ] `ReferenceTable::Grow` · 004550b0 · legions 00455d20
- [ ] `ReferenceTable::ScanThreads` · 00455130 · legions 00455d90
- [ ] `ReferenceTable::ScanSingletons` · 004551b0 · legions 00455e10
- [ ] `ReferenceTable::ScanVMData` · 00455210 · legions 00455e70

### ContextState
- [ ] `ContextState::PushDInt` · 00455270 · legions 00455ed0
- [ ] `ContextState::HaltTimers` · 004552c0 · legions 00455f20
- [ ] `ContextState::ResumeTimers` · 00455340 · legions 00455fa0

### VM
- [ ] `VM::Execute` · 004553f0 · legions 00456050 — ac2-script vm (VM/ReferenceTable/Resolver)

### Resolver
- [ ] `Resolver::GetFunctionOffset` · 00455460 · legions 004560c0 — ac2-script vm (VM/ReferenceTable/Resolver)

### ReferenceTable
- [ ] `ReferenceTable::FinishGarbageCollection` · 004554d0 · legions 00456130
- [ ] `ReferenceTable::ScanProcesses` · 00455600 · legions 00456210

### VM
- [ ] `VM::CallIFun` · 00455740 · legions 00456350 — ac2-script vm (VM/ReferenceTable/Resolver)

### ReferenceTable
- [ ] `ReferenceTable::CallPackageInitializer` · 004557b0 · legions 004563c0 — ac2-script vm (VM/ReferenceTable/Resolver)
- [ ] `ReferenceTable::UpdateRelativeReferences` · 004558a0 · legions 004564b0 — ac2-script vm: no GC/reference reuse, no VMMon timers, efun ids resolved per call
- [ ] `ReferenceTable::StartGarbageCollection` · 004558f0 · legions 00456500

### ggmCWInterface
- [x] `ggmCWInterface::LAHashToXPHash` · 004559c0 · legions 00456590 — ac2-client Client::fellow_xp_hash: an empty LAHash out parameter to IFun 0xB400A8 (CPlayer 0x2C -> FellowshipSystem 0x41), read back as (id, share)
- [ ] `ggmCWInterface::RListToCraftSkillAdaptorList` · 00455ae0 · legions 004566b0
- [ ] `ggmCWInterface::RListToIngredientAdaptorList` · 00455b30 · legions 00456700
- [ ] `ggmCWInterface::RListToRecipeAdaptorList` · 00455b80 · legions 00456750
- [ ] `ggmCWInterface::ItemListToLList` · 00455bd0 · legions 004567a0
- [ ] `ggmCWInterface::IngredientOrdinalHashToALHash` · 00455c40 · legions 00456810

### PList<unsigned___int64>
- [ ] `PList<unsigned___int64>::Pack` · 00455fb0 · legions 00456b80

### PList<InstanceID>
- [ ] `PList<InstanceID>::GetPackSize` · 00456000 · legions 00416530

### PList<unsigned___int64>
- [ ] `PList<unsigned___int64>::UnPack` · 00456020 · legions 00456bd0

### ReferenceTable
- [x] `ReferenceTable::AddReference` · 00456190 · legions 00456d40 — ac2-script vm::refs (reserved bit on every new entry)
- [ ] `ReferenceTable::UpdateReferences` · 00456270 · legions 00456e20 — ac2-script vm: no GC/reference reuse, no VMMon timers, efun ids resolved per call

### ggmCWInterface
- [x] `ggmCWInterface::XPHashToLAHash` · 00456330 · legions 00456ee0 — ac2-client Client::fellow_xp_hash: an empty LAHash out parameter to IFun 0xB400A8 (CPlayer 0x2C -> FellowshipSystem 0x41), read back as (id, share)
- [ ] `ggmCWInterface::RListToEffectUINodeList` · 00456440 · legions 00456ff0

### BaseList<unsigned___int64,60>
- [ ] `BaseList<unsigned___int64,60>::~BaseList<unsigned___int64,60>` · 00456490 · legions 00457040

### BaseList<unsigned_long,102>
- [ ] `BaseList<unsigned_long,102>::~BaseList<unsigned_long,102>` · 004564b0 · legions 00457060

### BaseHash<unsigned_long,unsigned___int64,9>
- [ ] `BaseHash<unsigned_long,unsigned___int64,9>::~BaseHash<unsigned_long,unsigned___int64,9>` · 004564d0 · legions 00457080

### PList<gmActInfo>
- [ ] `PList<gmActInfo>::PList<gmActInfo>` · 00456520 · legions 00457120

### ReferenceTable
- [x] `ReferenceTable::CreateNativeObject` · 00456550 · legions 00457150 — ac2-script vm::refs (reserved bit on every new entry)
- [ ] `ReferenceTable::GetSingleton` · 004565c0 · legions 004571c0 — ac2-script vm (VM/ReferenceTable/Resolver)

### RList
- [ ] `RList::RList` · 00456690 · legions 00457290
- [ ] `RList::GetReferences` · 004566d0
- [ ] `RList::UpdateReferences` · 00456710

### LList
- [ ] `LList::LList` · 00456770 · legions 00457370
- [ ] `LList::UnPack` · 004567b0 — ac2-script reference/native unpack

### ggmCWInterface
- [ ] `ggmCWInterface::CraftSkillAdaptorListToRList` · 004567c0 · legions 004573c0
- [ ] `ggmCWInterface::IngredientAdaptorListToRList` · 00456860 · legions 00457460
- [ ] `ggmCWInterface::RecipeAdaptorListToRList` · 00456900 · legions 00457500
- [ ] `ggmCWInterface::IgnoredListToRList` · 004569a0 · legions 00525e50
- [ ] `ggmCWInterface::EffectUINodeListToRList` · 00456a40 · legions 004575a0
- [ ] `ggmCWInterface::ShortcutInfoArrayToRArray` · 00456ae0 · legions 00457640 — ac2-script glue::options / examination (Legions layout and efun switch)
- [ ] `ggmCWInterface::GUNUserStatusListToRList` · 00456b80 · legions 004576e0

### PHashTable<unsigned___int64,unsigned_long>
- [ ] `PHashTable<unsigned___int64,unsigned_long>::StreamPack` · 00456c20 · legions 00457780

### (free functions)
- [ ] `STREAMPACK` · 00456e10 · legions 004e95e0

### PHashTable<unsigned_long,unsigned___int64>
- [ ] `PHashTable<unsigned_long,unsigned___int64>::StreamPack` · 00456f30 · legions 00457a90

### ReferenceTable
- [ ] `ReferenceTable::UnPackReferenceArray` · 00457120 · legions 00457c80 — ac2-script PackedReference::unpack: decode only, no reference table insert yet
- [ ] `ReferenceTable::UnPackReference` · 00457330 · legions 00457e90 — ac2-script PackedReference::unpack: decode only, no reference table insert yet

### LAHash
- [ ] `LAHash::LAHash` · 00457460 · legions 00457fc0
- [ ] `LAHash::StreamPack` · 004574a0

### ALHash
- [ ] `ALHash::ALHash` · 004574d0 · legions 00458030
- [ ] `ALHash::StreamPack` · 00457510

### ReferenceTable
- [ ] `ReferenceTable::FillPackage` · 00457540 · legions 004580a0 — ac2-script vm (VM/ReferenceTable/Resolver)
- [ ] `ReferenceTable::GetReferencePtr` · 00457610 · legions 00458170 — ac2-script vm (VM/ReferenceTable/Resolver)
- [ ] `ReferenceTable::GetPackageID` · 004576d0 · legions 00458230 — ac2-script vm (VM/ReferenceTable/Resolver)

### ggmCWInterface
- [~] `ggmCWInterface::CallIFun_AcceptTrade` · 00457730 · legions 00458290 — IFun body runs on the script VM; its WM_Trade stub id (0xB400B8..BD -> 0x8215_xxxx) traced from the Legions WLib (CWInterface IFun -> stub) and ported in ac2-net gameplay/request.rs
- [ ] `ggmCWInterface::CallIFun_AcceptVassal` · 004577c0 · legions 00458320
- [ ] `ggmCWInterface::CallIFun_AddEffectDataToProfileByEID` · 00457850 · legions 004583b0
- [ ] `ggmCWInterface::CallIFun_BreakAllegiance` · 00457a20 · legions 00458580
- [ ] `ggmCWInterface::CallIFun_CanAutoAttackTarget` · 00457af0 · legions 00458650
- [ ] `ggmCWInterface::CallIFun_CanCraftSkillBeRaised` · 00457bd0 · legions 00458730
- [ ] `ggmCWInterface::CallIFun_CanSkillBeRaisedByEnum` · 00457c90 · legions 004587f0 — ac2-script host::ifun (ScriptHost::query) + ac2-client scripts_queries
- [ ] `ggmCWInterface::CallIFun_CanSkillBeTrainedByEnum` · 00457d50 · legions 004588b0 — ac2-script host::ifun (ScriptHost::query) + ac2-client scripts_queries
- [ ] `ggmCWInterface::CallIFun_CanSkillBeUntrainedByEnum` · 00457e10 · legions 00458970 — ac2-script host::ifun (ScriptHost::query) + ac2-client scripts_queries
- [ ] `ggmCWInterface::CallIFun_CheckIngredient` · 00457ed0 · legions 00458a30
- [ ] `ggmCWInterface::CallIFun_ClearResurrectionRequest` · 00458090 · legions 00458bf0
- [~] `ggmCWInterface::CallIFun_CloseTradeNegotiations` · 00458120 · legions 00458c80 — IFun body runs on the script VM; its WM_Trade stub id (0xB400B8..BD -> 0x8215_xxxx) traced from the Legions WLib (CWInterface IFun -> stub) and ported in ac2-net gameplay/request.rs
- [~] `ggmCWInterface::CallIFun_DeclineTrade` · 004581b0 · legions 00458d10 — IFun body runs on the script VM; its WM_Trade stub id (0xB400B8..BD -> 0x8215_xxxx) traced from the Legions WLib (CWInterface IFun -> stub) and ported in ac2-net gameplay/request.rs
- [ ] `ggmCWInterface::CallIFun_DeclineVassal` · 00458240 · legions 00458da0
- [ ] `ggmCWInterface::CallIFun_Dismount` · 004582f0 · legions 00458f40
- [ ] `ggmCWInterface::CallIFun_DisplaySaleReminders` · 00458380 · legions 00458fd0
- [ ] `ggmCWInterface::CallIFun_DoAssistedTell` · 00458410 · legions 00459060
- [ ] `ggmCWInterface::CallIFun_DoEmote` · 00458500 · legions 00459150
- [ ] `ggmCWInterface::CallIFun_DoFellowshipCreate` · 004585e0 · legions 00459230
- [ ] `ggmCWInterface::CallIFun_DoFellowshipDisband` · 00458710 · legions 00459360
- [ ] `ggmCWInterface::CallIFun_DoFellowshipDismiss` · 004587a0 · legions 004593f0
- [ ] `ggmCWInterface::CallIFun_DoFellowshipLeave` · 00458870 · legions 004594c0
- [ ] `ggmCWInterface::CallIFun_DoFellowshipRecruit` · 00458900 · legions 00459550
- [ ] `ggmCWInterface::CallIFun_DoFellowshipRecruitAccept` · 004589d0 · legions 00459620
- [ ] `ggmCWInterface::CallIFun_DoFellowshipRecruitDecline` · 00458a60 · legions 004596b0
- [ ] `ggmCWInterface::CallIFun_DoIHaveItem` · 00458b10 · legions 00459760
- [ ] `ggmCWInterface::CallIFun_DoUIDamage` · 00458bf0 · legions 00459840
- [ ] `ggmCWInterface::CallIFun_DoesIngredientAutoPopulate` · 00458dd0 · legions 00459a20
- [~] `ggmCWInterface::CallIFun_DragFromMoneyBag` · 00458ec0 · legions 00459b10 — ac2-weenie OnlineSession::drag_to_money_bag / drag_from_money_bag (CMoneySystem 0x4 / 0x5 -> WM_Money 0x7 / 0x8, receive_money for 0x6 / 0x9); MoneySystem 0x6's ownership checks and 0x2's room check not mirrored
- [~] `ggmCWInterface::CallIFun_DragToMoneyBag` · 00458fe0 · legions 00459c30 — ac2-weenie OnlineSession::drag_to_money_bag / drag_from_money_bag (CMoneySystem 0x4 / 0x5 -> WM_Money 0x7 / 0x8, receive_money for 0x6 / 0x9); MoneySystem 0x6's ownership checks and 0x2's room check not mirrored
- [ ] `ggmCWInterface::CallIFun_EquipItem` · 00459140 · legions 00459d90
- [ ] `ggmCWInterface::CallIFun_GUNEvent_FriendStart` · 00459240 · legions 00459e90
- [ ] `ggmCWInterface::CallIFun_GUNEvent_FriendStatus` · 00459340 · legions 00459f90
- [ ] `ggmCWInterface::CallIFun_GUNEvent_HearFromRoom` · 00459420 · legions 0045a070
- [ ] `ggmCWInterface::CallIFun_GUNHandleReauthentication` · 004595a0 · legions 0045a1f0
- [ ] `ggmCWInterface::CallIFun_GetAcceptAllegianceRequest` · 00459630 · legions 0045a280
- [ ] `ggmCWInterface::CallIFun_GetAcceptFellowshipRequest` · 004596d0 · legions 0045a320
- [ ] `ggmCWInterface::CallIFun_GetAcceptTradeRequest` · 00459770 · legions 0045a3c0
- [ ] `ggmCWInterface::CallIFun_GetAutoTarget` · 00459810 · legions 0045a460
- [ ] `ggmCWInterface::CallIFun_GetAvailableCraftExperience` · 004598b0 · legions 0045a500
- [ ] `ggmCWInterface::CallIFun_GetAvailableExperience` · 00459970 · legions 0045a5c0
- [ ] `ggmCWInterface::CallIFun_GetBypassGoldMeltingConfirmationDlg` · 00459a30 · legions 0045a680
- [ ] `ggmCWInterface::CallIFun_GetConsignmentCharge` · 00459ad0 · legions 0045a720
- [x] `ggmCWInterface::CallIFun_GetContainerSegmentInfo` · 00459b70 · legions 0045a7c0 — ac2-client Client::contents_info / container_segment_info (Container 0x3D / 0x3F on the VM); the double sends ContainerSegmentDescriptor segments
- [x] `ggmCWInterface::CallIFun_GetContentsInfo` · 00459d30 · legions 0045a980 — ac2-client Client::contents_info / container_segment_info (Container 0x3D / 0x3F on the VM); the double sends ContainerSegmentDescriptor segments
- [ ] `ggmCWInterface::CallIFun_GetDamageTextRangeOther` · 0045a000 · legions 0045ac50
- [ ] `ggmCWInterface::CallIFun_GetDurabilityLevel` · 0045a0a0 · legions 0045acf0
- [ ] `ggmCWInterface::CallIFun_GetFXAndBehaviorInformationFromWsl` · 0045a140 · legions 0045ad90
- [x] `ggmCWInterface::CallIFun_GetFellowXPHash` · 0045a310 · legions 0045af60 — ac2-client Client::fellow_xp_hash: an empty LAHash out parameter to IFun 0xB400A8 (CPlayer 0x2C -> FellowshipSystem 0x41), read back as (id, share)
- [ ] `ggmCWInterface::CallIFun_GetGameplayOptions` · 0045a4b0 · legions 0045b100 — ac2-script host::ifun (ScriptHost::query) + ac2-client scripts_queries
- [ ] `ggmCWInterface::CallIFun_GetHeroSkillCredits` · 0045a620 · legions 0045b270
- [ ] `ggmCWInterface::CallIFun_GetIconID` · 0045a6c0 · legions 0045b310
- [ ] `ggmCWInterface::CallIFun_GetLastSelection` · 0045a760 · legions 0045b3b0
- [ ] `ggmCWInterface::CallIFun_GetLatencyIndicatorGraph` · 0045a820 · legions 0045b470
- [ ] `ggmCWInterface::CallIFun_GetMaxContainerSize` · 0045a8c0 · legions 0045b510
- [ ] `ggmCWInterface::CallIFun_GetMaxDurabilityLevel` · 0045a960 · legions 0045b5b0
- [ ] `ggmCWInterface::CallIFun_GetMaxPurchaseQuantity` · 0045aa00 · legions 0045b650
- [ ] `ggmCWInterface::CallIFun_GetMaxStackSize` · 0045aae0 · legions 0045b730
- [ ] `ggmCWInterface::CallIFun_GetMissileInformationFromWSL` · 0045ab80 · legions 0045b7d0
- [ ] `ggmCWInterface::CallIFun_GetPotentiallyPluralizedName` · 0045adb0 · legions 0045ba00
- [x] `ggmCWInterface::CallIFun_GetPreferredInventoryLocation` · 0045ae80 · legions 0045bad0 — ac2-weenie ItemStats::preferred_location (Item 0x26: int 282), valid_locations (275), precluded_locations (283)
- [ ] `ggmCWInterface::CallIFun_GetQuantity` · 0045af20 · legions 0045bb70
- [ ] `ggmCWInterface::CallIFun_GetRadarMask` · 0045afc0 · legions 0045bc10
- [ ] `ggmCWInterface::CallIFun_GetSelection` · 0045b060 · legions 0045bcb0
- [ ] `ggmCWInterface::CallIFun_GetShortcutHeight` · 0045b120 · legions 0045bd70
- [ ] `ggmCWInterface::CallIFun_GetShortcutInstaScroll` · 0045b1c0 · legions 0045be10
- [ ] `ggmCWInterface::CallIFun_GetShortcutLocked` · 0045b260 · legions 0045beb0
- [ ] `ggmCWInterface::CallIFun_GetShowDamageText` · 0045b300 · legions 0045bf50
- [ ] `ggmCWInterface::CallIFun_GetShowDamageTextOther` · 0045b3a0 · legions 0045bff0
- [ ] `ggmCWInterface::CallIFun_GetShowFellowsOnMap` · 0045b440 · legions 0045c090
- [ ] `ggmCWInterface::CallIFun_GetShowObjectNames` · 0045b4e0 · legions 0045c130
- [ ] `ggmCWInterface::CallIFun_GetShowPKDamage` · 0045b580 · legions 0045c1d0
- [ ] `ggmCWInterface::CallIFun_GetShowPetUI` · 0045b620 · legions 0045c270
- [ ] `ggmCWInterface::CallIFun_GetShowRadarCoords` · 0045b6c0 · legions 0045c310
- [ ] `ggmCWInterface::CallIFun_GetShowShortcutNumbers` · 0045b760 · legions 0045c3b0
- [ ] `ggmCWInterface::CallIFun_GetSkillCostToTrainByEnum` · 0045b800 · legions 0045c450 — ac2-script host::ifun (ScriptHost::query) + ac2-client scripts_queries
- [ ] `ggmCWInterface::CallIFun_GetSkillCredits` · 0045b8c0 · legions 0045c510
- [ ] `ggmCWInterface::CallIFun_GetSkillExperienceNeededToLevelByEnum` · 0045b960 · legions 0045c5b0 — ac2-script host::ifun (ScriptHost::query) + ac2-client scripts_queries
- [ ] `ggmCWInterface::CallIFun_GetSkillXPByEnum` · 0045ba40 · legions 0045c690
- [ ] `ggmCWInterface::CallIFun_GetStackable` · 0045bb20 · legions 0045c770
- [ ] `ggmCWInterface::CallIFun_GetWhichShortcutSet` · 0045bbc0 · legions 0045c810
- [ ] `ggmCWInterface::CallIFun_HandlePlayScenesDone` · 0045bc60 · legions 0045c8b0
- [ ] `ggmCWInterface::CallIFun_HandleWeenieEmote` · 0045bd60 · legions 0045c9b0
- [ ] `ggmCWInterface::CallIFun_HasDurability` · 0045be20 · legions 0045ca70
- [ ] `ggmCWInterface::CallIFun_HasSomethingSelected` · 0045bec0 · legions 0045cb10
- [ ] `ggmCWInterface::CallIFun_HearTell` · 0045bf60 · legions 0045cbb0
- [ ] `ggmCWInterface::CallIFun_InqActInfoList` · 0045c0f0 · legions 0045cd40
- [ ] `ggmCWInterface::CallIFun_InqAlias` · 0045c2a0 · legions 0045cef0
- [ ] `ggmCWInterface::CallIFun_InqAllInventoryIIDList` · 0045c4a0 · legions 0045d0f0
- [ ] `ggmCWInterface::CallIFun_InqCraftSkillAdaptor` · 0045c630 · legions 0045d280
- [ ] `ggmCWInterface::CallIFun_InqCraftSkillCategoryName` · 0045c7c0 · legions 0045d410
- [ ] `ggmCWInterface::CallIFun_InqCraftSkillLevel` · 0045c950 · legions 0045d5a0
- [ ] `ggmCWInterface::CallIFun_InqCraftSkillList` · 0045cb20 · legions 0045d770
- [~] `ggmCWInterface::CallIFun_InqEffectList` · 0045ccc0 · legions 0045d910 — ac2-client Client::effect_list (IFun 0xB40062, CPlayer 0x52 -> CEffectSystem 0x7); waits on the VM's gEffectUINode efuns
- [ ] `ggmCWInterface::CallIFun_InqNextReplyHistoryItem` · 0045ce60 · legions 0045dab0
- [ ] `ggmCWInterface::CallIFun_InqNextRetellHistoryItem` · 0045d000 · legions 0045dc50
- [ ] `ggmCWInterface::CallIFun_InqPrevReplyHistoryItem` · 0045d1a0 · legions 0045ddf0
- [ ] `ggmCWInterface::CallIFun_InqPrevRetellHistoryItem` · 0045d340 · legions 0045df90
- [ ] `ggmCWInterface::CallIFun_InqQuestInfoList` · 0045d4e0 · legions 0045e130
- [~] `ggmCWInterface::CallIFun_InqRadarLook` · 0045d660 · legions 0045e2b0 — ac2-client client_radar.rs + ac2-script radar.rs: looks asked on each object's script, the ranges from RadarControl; the creature, NPC and player scripts answer no look in the double (no reads, retval 0); no drawing (pixel mapping, tooltip) in ac2-ui
- [ ] `ggmCWInterface::CallIFun_InqRecipeAdaptor` · 0045d870 · legions 0045e4c0
- [ ] `ggmCWInterface::CallIFun_InqRecipeCost` · 0045da10 · legions 0045e660
- [ ] `ggmCWInterface::CallIFun_InqRecipeDifficulty` · 0045db70 · legions 0045e7c0
- [ ] `ggmCWInterface::CallIFun_InqRecipeIngredients` · 0045dd60 · legions 0045e9b0
- [ ] `ggmCWInterface::CallIFun_InqRecipeList` · 0045dfc0 · legions 0045ec10
- [ ] `ggmCWInterface::CallIFun_InqRootPanelFromRace` · 0045e180 · legions 0045edd0 — ac2-script host::ifun (ScriptHost::query) + ac2-client scripts_queries
- [ ] `ggmCWInterface::CallIFun_InqShortcutArray` · 0045e2c0 · legions 0045ef10
- [ ] `ggmCWInterface::CallIFun_InqShortcutInfo` · 0045e450 · legions 0045f0a0 — ac2-script host::ifun (ScriptHost::query) + ac2-client scripts_queries
- [ ] `ggmCWInterface::CallIFun_InqSimpleInvByLocTable` · 0045e660 · legions 0045f2b0
- [ ] `ggmCWInterface::CallIFun_InqSkillLevelByEnum` · 0045e7f0 · legions 0045f440 — ac2-script host::ifun (ScriptHost::query) + ac2-client scripts_queries
- [ ] `ggmCWInterface::CallIFun_InqSkillTimeLeft` · 0045e9c0 · legions 0045f610 — ac2-script host::ifun (ScriptHost::query) + ac2-client scripts_queries
- [ ] `ggmCWInterface::CallIFun_InqUISaveLocations` · 0045eb90 · legions 0045f7e0
- [ ] `ggmCWInterface::CallIFun_IsCraftSkillRaiseable` · 0045ed00 · legions 0045f950
- [ ] `ggmCWInterface::CallIFun_IsFellowshipMember` · 0045edc0 · legions 0045fa10
- [ ] `ggmCWInterface::CallIFun_IsHeroSkillByEnum` · 0045eea0 · legions 0045faf0 — ac2-script host::ifun (ScriptHost::query) + ac2-client scripts_queries
- [ ] `ggmCWInterface::CallIFun_IsOnInventory` · 0045ef60 · legions 0045fbb0
- [ ] `ggmCWInterface::CallIFun_IsSkillBeingUntrainedByEnum` · 0045f040 · legions 0045fc90
- [ ] `ggmCWInterface::CallIFun_IsSkillHarmful` · 0045f100 · legions 0045fd50
- [ ] `ggmCWInterface::CallIFun_IsSkillTrainedByEnum` · 0045f1c0 · legions 0045fe10
- [ ] `ggmCWInterface::CallIFun_IsSkillUsableBasedOnInventoryByEnum` · 0045f280 · legions 0045fed0
- [ ] `ggmCWInterface::CallIFun_LootAllFromContainer` · 0045f340 · legions 0045ff90
- [ ] `ggmCWInterface::CallIFun_MoveItem` · 0045f410 · legions 00460060
- [~] `ggmCWInterface::CallIFun_OfferItemForTrade` · 0045f5e0 · legions 00460230 — IFun body runs on the script VM; its WM_Trade stub id (0xB400B8..BD -> 0x8215_xxxx) traced from the Legions WLib (CWInterface IFun -> stub) and ported in ac2-net gameplay/request.rs
- [ ] `ggmCWInterface::CallIFun_OpenRecipe` · 0045f6e0 · legions 00460330
- [~] `ggmCWInterface::CallIFun_OpenTradeNegotiations` · 0045f790 · legions 004603e0 — IFun body runs on the script VM; its WM_Trade stub id (0xB400B8..BD -> 0x8215_xxxx) traced from the Legions WLib (CWInterface IFun -> stub) and ported in ac2-net gameplay/request.rs
- [ ] `ggmCWInterface::Resume_PopupYesNoDialog` · 0045f860 · legions 004604b0
- [ ] `ggmCWInterface::CallIFun_PurchaseConfirmed` · 0045f8f0 · legions 00460540
- [ ] `ggmCWInterface::CallIFun_PurchaseItem` · 0045f9a0 · legions 004605f0
- [ ] `ggmCWInterface::CallIFun_QueryAllegianceProfile` · 0045fac0 · legions 00460710
- [ ] `ggmCWInterface::CallIFun_RegisterCommands` · 0045fb90 · legions 004607e0
- [ ] `ggmCWInterface::CallIFun_RequestAddIngredient` · 0045fc20 · legions 00460870
- [ ] `ggmCWInterface::CallIFun_RequestAttackStateChange` · 0045fd60 · legions 004609b0
- [ ] `ggmCWInterface::CallIFun_RequestCollect` · 0045fe20 · legions 00460a70
- [ ] `ggmCWInterface::CallIFun_RequestEnterConsignment` · 0045ff00 · legions 00460b50
- [ ] `ggmCWInterface::CallIFun_RequestEnterStore` · 0045ff90 · legions 00460be0
- [ ] `ggmCWInterface::CallIFun_RequestExecuteCraft` · 00460040 · legions 00460c90
- [ ] `ggmCWInterface::CallIFun_RequestLeaveCatalog` · 00460190 · legions 00460de0
- [ ] `ggmCWInterface::CallIFun_RequestLeaveConsignment` · 00460220 · legions 00460e70
- [ ] `ggmCWInterface::CallIFun_RequestLeaveStore` · 004602b0 · legions 00460f00
- [ ] `ggmCWInterface::CallIFun_RequestNextSales` · 00460340 · legions 00460f90
- [ ] `ggmCWInterface::CallIFun_RequestPetAttack` · 004603d0 · legions 00461020
- [ ] `ggmCWInterface::CallIFun_RequestPetBuff` · 004604a0 · legions 004610f0
- [ ] `ggmCWInterface::CallIFun_RequestPetHeal` · 00460530 · legions 00461180
- [ ] `ggmCWInterface::CallIFun_RequestPetMode` · 004605c0 · legions 00461210
- [ ] `ggmCWInterface::CallIFun_RequestPetReturn` · 004606c0 · legions 00461310
- [ ] `ggmCWInterface::CallIFun_RequestPrevSales` · 00460750 · legions 004613a0
- [ ] `ggmCWInterface::CallIFun_RequestRaiseCraftSkill` · 004607e0 · legions 00461430
- [ ] `ggmCWInterface::CallIFun_RequestRaiseSkill` · 00460890 · legions 004614e0
- [ ] `ggmCWInterface::CallIFun_RequestRestock` · 00460940 · legions 00461590
- [ ] `ggmCWInterface::CallIFun_RequestResurrect` · 00460a60 · legions 004616b0
- [ ] `ggmCWInterface::CallIFun_RequestSales` · 00460af0 · legions 00461740
- [ ] `ggmCWInterface::CallIFun_RequestSell` · 00460ba0 · legions 004617f0
- [ ] `ggmCWInterface::CallIFun_RequestSetPetName` · 00460cc0 · legions 00461910
- [ ] `ggmCWInterface::CallIFun_RequestTrainSkill` · 00460df0 · legions 00461a40
- [ ] `ggmCWInterface::CallIFun_RequestUntrainSkill` · 00460ea0 · legions 00461af0
- [ ] `ggmCWInterface::CallIFun_RequestUntrainSkillCancel` · 00460f50 · legions 00461ba0
- [ ] `ggmCWInterface::CallIFun_ResetReplyHistoryIndex` · 00460fe0 · legions 00461c30
- [ ] `ggmCWInterface::CallIFun_ResetRetellHistoryIndex` · 00461070 · legions 00461cc0
- [ ] `ggmCWInterface::CallIFun_Result_CallCreate` · 00461100 · legions 00461d50
- [ ] `ggmCWInterface::CallIFun_Result_FriendAdd` · 004612c0 · legions 00461f10
- [ ] `ggmCWInterface::CallIFun_Result_FriendDelete` · 004613d0 · legions 00462020
- [ ] `ggmCWInterface::CallIFun_Result_GetFriendsList` · 004614e0
- [ ] `ggmCWInterface::CallIFun_Result_GetIgnoredList` · 00461620
- [ ] `ggmCWInterface::CallIFun_Result_IgnoredAdd` · 00461760 · legions 004623b0
- [ ] `ggmCWInterface::CallIFun_Result_IgnoredDelete` · 00461870 · legions 004624c0
- [ ] `ggmCWInterface::CallIFun_Result_SendToRoom` · 00461980 · legions 004625d0
- [ ] `ggmCWInterface::CallIFun_Result_SendToUser` · 00461ab0 · legions 00462700
- [~] `ggmCWInterface::CallIFun_RevokeItemFromTrade` · 00461c10 · legions 00462860 — IFun body runs on the script VM; its WM_Trade stub id (0xB400B8..BD -> 0x8215_xxxx) traced from the Legions WLib (CWInterface IFun -> stub) and ported in ac2-net gameplay/request.rs
- [ ] `ggmCWInterface::CallIFun_SetAcceptAllegianceRequest` · 00461ce0 · legions 00462930
- [ ] `ggmCWInterface::CallIFun_SetAcceptFellowshipRequest` · 00461d90 · legions 004629e0
- [ ] `ggmCWInterface::CallIFun_SetAcceptTradeRequest` · 00461e40 · legions 00462a90
- [ ] `ggmCWInterface::CallIFun_SetAutoTarget` · 00461ef0 · legions 00462b40
- [ ] `ggmCWInterface::CallIFun_SetBypassGoldMeltingConfirmationDlg` · 00461fa0 · legions 00462bf0
- [ ] `ggmCWInterface::CallIFun_SetChatFontColor` · 00462050 · legions 00462ca0
- [ ] `ggmCWInterface::CallIFun_SetChatFontSize` · 00462140 · legions 00462d90
- [ ] `ggmCWInterface::CallIFun_SetChatPopupFlag` · 00462230 · legions 00462e80
- [ ] `ggmCWInterface::CallIFun_SetChatWindowFilter` · 00462320 · legions 00462f70
- [ ] `ggmCWInterface::CallIFun_SetDamageTextRangeOther` · 00462410 · legions 00463060
- [ ] `ggmCWInterface::CallIFun_SetExaminationRequest` · 004624c0 · legions 00463110
- [ ] `ggmCWInterface::CallIFun_SetLatencyIndicatorGraph` · 004625a0 · legions 004631f0
- [ ] `ggmCWInterface::CallIFun_SetRadarMask` · 00462650 · legions 004632a0
- [ ] `ggmCWInterface::CallIFun_SetSelection` · 00462700 · legions 00463350
- [ ] `ggmCWInterface::CallIFun_SetShortcut` · 004627d0 · legions 00463420 — ac2-script host::ifun (ScriptHost::query) + ac2-client scripts_queries
- [ ] `ggmCWInterface::CallIFun_SetShortcutHeight` · 004628e0 · legions 00463530
- [ ] `ggmCWInterface::CallIFun_SetShortcutInstaScroll` · 00462990 · legions 004635e0
- [ ] `ggmCWInterface::CallIFun_SetShortcutLocked` · 00462a40 · legions 00463690
- [ ] `ggmCWInterface::CallIFun_SetShowDamageText` · 00462af0 · legions 00463740
- [ ] `ggmCWInterface::CallIFun_SetShowDamageTextOther` · 00462ba0 · legions 004637f0
- [ ] `ggmCWInterface::CallIFun_SetShowFellowsOnMap` · 00462c50 · legions 004638a0
- [ ] `ggmCWInterface::CallIFun_SetShowObjectNames` · 00462d00 · legions 00463950
- [ ] `ggmCWInterface::CallIFun_SetShowPKDamage` · 00462db0 · legions 00463a00
- [ ] `ggmCWInterface::CallIFun_SetShowPetUI` · 00462e60 · legions 00463ab0
- [ ] `ggmCWInterface::CallIFun_SetShowRadarCoords` · 00462f10 · legions 00463b60
- [ ] `ggmCWInterface::CallIFun_SetShowShortcutNumbers` · 00462fc0 · legions 00463c10
- [ ] `ggmCWInterface::CallIFun_SetUISaveLocations` · 00463070 · legions 00463cc0
- [ ] `ggmCWInterface::CallIFun_SetWhichShortcutSet` · 00463150 · legions 00463da0
- [ ] `ggmCWInterface::CallIFun_SetWindowOpacity` · 00463200 · legions 00463e50
- [ ] `ggmCWInterface::CallIFun_ShouldAutoTarget` · 004632f0 · legions 00463f40
- [ ] `ggmCWInterface::CallIFun_SpecialAttackEnded` · 00463390 · legions 00463fe0
- [ ] `ggmCWInterface::CallIFun_SpecialAttackStarted` · 00463440 · legions 00464090
- [ ] `ggmCWInterface::CallIFun_StartExamination` · 004634f0 · legions 00464140
- [ ] `ggmCWInterface::CallIFun_StopAttacking` · 00463580 · legions 004641f0
- [ ] `ggmCWInterface::CallIFun_StopAttackingIfAutoMoving` · 00463660 · legions 004642d0
- [ ] `ggmCWInterface::CallIFun_StopExamination` · 004636f0 · legions 00464360
- [ ] `ggmCWInterface::CallIFun_SwearAllegiance` · 00463780 · legions 004643f0
- [ ] `ggmCWInterface::CallIFun_TransmuteAllFromContainer` · 00463810 · legions 00464480
- [ ] `ggmCWInterface::CallIFun_UnEquipItem` · 004638e0 · legions 00464550
- [ ] `ggmCWInterface::CallIFun_UpdateOpen` · 004639e0 · legions 00464650
- [ ] `ggmCWInterface::CallIFun_UploadChatOptions` · 00463ae0 · legions 00464750
- [ ] `ggmCWInterface::CallIFun_UploadGameOptions` · 00463b70 · legions 004647e0
- [ ] `ggmCWInterface::CallIFun_UploadUISettings` · 00463c00 · legions 00464870
- [ ] `ggmCWInterface::CallIFun_UseSelection` · 00463c90 · legions 00464900
- [ ] `ggmCWInterface::CallIFun_UseShortcut` · 00463d30 · legions 004649a0 — ac2-script host::ifun (ScriptHost::query) + ac2-client scripts_queries

## `ac2/client/src/weenie/interp/glue/gmcefundispatcher.cpp.c` (0/1)

### gmCEFunDispatcher
- [ ] `gmCEFunDispatcher::GetEFunAddress` · 00463df0 · legions 00464a60 — ac2-script glue: id table + pure-glue registry; engine glue unbound

## `ac2/client/src/weenie/interp/glue/gmcnativefactory.cpp.c` (0/18)

### gmCNativeFactory
- [ ] `gmCNativeFactory::HashNativeObject` · 00465f30 · legions 00466c10
- [ ] `gmCNativeFactory::IsEqual` · 00465f60
- [ ] `gmCNativeFactory::GetReferences` · 00465fc0 · legions 00466c70
- [ ] `gmCNativeFactory::UpdateReferences` · 00465fe0 · legions 00466c90
- [ ] `gmCNativeFactory::GetNativePackSize` · 00466000 · legions 00466cb0
- [ ] `gmCNativeFactory::PackNativeObject` · 004661d0 · legions 00466e80 — ac2-net protocol::NativeObject pack+unpack: every native type the Legions WStates hold (all 39,748 graphs round-trip byte-exact) plus ShortcutInfo, UISaveLocations, SelectionInfo, Examination*, the adaptors; iterators and other engine types not ported
- [ ] `gmCNativeFactory::GetCheckSums` · 004663b0 · legions 00467060
- [ ] `gmCNativeFactory::ValidateCheckSums` · 004663d0 · legions 00467080

### PStringBase<char>
- [ ] `PStringBase<char>::operator!=` · 00466400 · legions 004670b0

### gmCNativeFactory
- [ ] `gmCNativeFactory::GetNativeTypeName` · 004664a0 · legions 00467150

### PStringBase<char>
- [ ] `PStringBase<char>::operator+=` · 00466930 · legions 004675e0

### gmCNativeFactory
- [ ] `gmCNativeFactory::ValidateGlueVersion` · 00466960 · legions 00467610
- [ ] `gmCNativeFactory::NewNativeObject` · 00466c10 · legions 004678c0

### AllegianceProfileAdaptor
- [ ] `AllegianceProfileAdaptor::~AllegianceProfileAdaptor` · 00467040 · legions 00467cf0

### FellowshipAdaptor
- [ ] `FellowshipAdaptor::~FellowshipAdaptor` · 00467100 · legions 00467db0

### gmCNativeFactory
- [ ] `gmCNativeFactory::UnPackNativeObject` · 00467160 · legions 00467e10 — ac2-script native::unpack: the native types Legions WStates hold
- [ ] `gmCNativeFactory::CopyNativeObject` · 00467370 · legions 00468020
- [ ] `gmCNativeFactory::DeleteNativeObject` · 004677e0 · legions 00468490

## `ac2/shared/src/weenie/allocator/gmwallocator.cpp.c` (0/6)

### gmCWAllocator
- [ ] `gmCWAllocator::Create` · 00453ed0 · legions 00454f70
- [ ] `gmCWAllocator::AllocateWeenieObject_Internal` · 00453f20 · legions 00454fc0
- [ ] `gmCWAllocator::AllocateWeenieDesc_Internal` · 00453f60 · legions 00455000
- [ ] `gmCWAllocator::AllocateWeenieDesc_Internal` · 00453f80 · legions 00455020
- [ ] `gmCWAllocator::AllocateEFunDispatcher` · 00454010 · legions 00455090
- [ ] `gmCWAllocator::AllocateNativeFactory` · 00454030 · legions 004550b0

## `ac2/shared/src/weenie/interface/gmwinterface.cpp.c` (36/249)

### gmCWInterface
- [ ] `gmCWInterface::gmCWInterface` · 0044f9a0 · legions 00450ae0

### CWPhysicsObject
- [ ] `CWPhysicsObject::SetPhysObj` · 0044f9c0 · legions 00450b00

### CWRepository
- [ ] `CWRepository::AdoptQualities` · 0044f9d0 · legions 00450b10

### gmCWInterface
- [ ] `gmCWInterface::InqName` · 0044f9e0 · legions 00450b20
- [~] `gmCWInterface::GetPotentiallyPluralizedName` · 0044fa00 · legions 00450b40 — ItemStats::display_name (gmEntity 0x10B/0x9F/0xAD); plural IsValid against the DAT tables left to the resolver
- [ ] `gmCWInterface::GetPhysObjID` · 0044fa70 · legions 00450bb0
- [ ] `gmCWInterface::SetSelection` · 0044faa0 · legions 00450be0 — ac2-weenie OnlineSession::select/start_examination/... over the PlayerScript stand-in for CPlayer 0x6A/0x3D/0x63/0x114/0xF6
- [ ] `gmCWInterface::GetSelection` · 0044fac0 · legions 00450c00 — ac2-weenie OnlineSession::select/start_examination/... over the PlayerScript stand-in for CPlayer 0x6A/0x3D/0x63/0x114/0xF6
- [ ] `gmCWInterface::GetLastSelection` · 0044fb20 · legions 00450c60 — ac2-weenie OnlineSession::select/start_examination/... over the PlayerScript stand-in for CPlayer 0x6A/0x3D/0x63/0x114/0xF6
- [x] `gmCWInterface::ShouldAutoTarget` · 0044fb80 · legions 00450cc0 — ac2-client: the ShouldAutoTarget IFun on the VM before attacking
- [ ] `gmCWInterface::UseSelection` · 0044fbb0 · legions 00450cf0 — ac2-weenie OnlineSession::use_selection (CPlayer 0x79 -> CUsageSystem 0x9 stand-in)
- [ ] `gmCWInterface::UseShortcut` · 0044fbe0 · legions 00450d20
- [ ] `gmCWInterface::StopAttacking` · 0044fc20 · legions 00450d60
- [x] `gmCWInterface::RequestAttackStateChange` · 0044fc60 · legions 00450da0 — ac2-client Command::SetAttacking: ShouldAutoTarget's Select(0x41000024) notice, then the IFun on the VM (stand-in: OnlineSession::request_attack_state -> WM_Combat 0x16 with the selection / 0x14)
- [ ] `gmCWInterface::CanAutoAttackTarget` · 0044fca0 · legions 00450de0
- [ ] `gmCWInterface::InqCraftSkillAdaptor` · 0044fce0 · legions 00450e20
- [ ] `gmCWInterface::InqCraftSkillList` · 0044fd20 · legions 00450e60
- [ ] `gmCWInterface::InqCraftSkillLevel` · 0044fd60 · legions 00450ea0
- [ ] `gmCWInterface::InqRecipeAdaptor` · 0044fda0 · legions 00450ee0
- [ ] `gmCWInterface::InqRecipeList` · 0044fde0 · legions 00450f20
- [ ] `gmCWInterface::InqRecipeIngredients` · 0044fe20 · legions 00450f60
- [ ] `gmCWInterface::InqRecipeDifficulty` · 0044fe70 · legions 00450fb0
- [ ] `gmCWInterface::DoesIngredientAutoPopulate` · 0044fec0 · legions 00451000
- [ ] `gmCWInterface::RequestAddIngredient` · 0044ff00 · legions 00451040
- [ ] `gmCWInterface::OpenRecipe` · 0044ff30 · legions 00451070
- [ ] `gmCWInterface::CheckIngredient` · 0044ff50 · legions 00451090
- [ ] `gmCWInterface::CanCraftSkillBeRaised` · 0044ffa0 · legions 004510e0
- [ ] `gmCWInterface::IsCraftSkillRaiseable` · 0044ffe0 · legions 00451120
- [ ] `gmCWInterface::RequestRaiseCraftSkill` · 00450020 · legions 00451160
- [ ] `gmCWInterface::InqCraftSkillCategoryName` · 00450040 · legions 00451180
- [ ] `gmCWInterface::RequestExecuteCraft` · 00450080 · legions 004511c0
- [ ] `gmCWInterface::InqRecipeCost` · 004500b0 · legions 004511f0
- [ ] `gmCWInterface::GetAvailableCraftExperience` · 004500f0 · legions 00451230
- [ ] `gmCWInterface::DoEmote` · 00450130 · legions 00451270 — ac2-weenie OnlineSession::emote(id): the WM_Communication emote call; the emote-table name lookup (CPlayer 0x15B) is not ported
- [ ] `gmCWInterface::PurchaseItem` · 00450150 · legions 00451290
- [ ] `gmCWInterface::RequestNextSales` · 00450180 · legions 004512c0
- [ ] `gmCWInterface::RequestPrevSales` · 004501a0 · legions 004512e0
- [ ] `gmCWInterface::RequestSales` · 004501c0 · legions 00451300
- [ ] `gmCWInterface::PurchaseConfirmed` · 004501e0 · legions 00451320
- [ ] `gmCWInterface::GetMaxPurchaseQuantity` · 00450200 · legions 00451340
- [ ] `gmCWInterface::RequestLeaveStore` · 00450240 · legions 00451380
- [ ] `gmCWInterface::RequestLeaveCatalog` · 00450260 · legions 004513a0
- [ ] `gmCWInterface::RequestLeaveConsignment` · 00450280 · legions 004513c0
- [ ] `gmCWInterface::RequestEnterConsignment` · 004502a0 · legions 004513e0
- [ ] `gmCWInterface::RequestEnterStore` · 004502c0 · legions 00451400
- [ ] `gmCWInterface::RequestSell` · 004502e0 · legions 00451420
- [ ] `gmCWInterface::RequestRestock` · 00450310 · legions 00451450
- [ ] `gmCWInterface::RequestCollect` · 00450340 · legions 00451480
- [ ] `gmCWInterface::GetConsignmentCharge` · 00450360 · legions 004514a0
- [ ] `gmCWInterface::DisplaySaleReminders` · 00450390 · legions 004514d0
- [ ] `gmCWInterface::RequestPetAttack` · 004503b0 · legions 004514f0
- [ ] `gmCWInterface::RequestPetBuff` · 004503d0 · legions 00451510
- [ ] `gmCWInterface::RequestPetHeal` · 004503f0 · legions 00451530
- [ ] `gmCWInterface::RequestPetReturn` · 00450410 · legions 00451550
- [ ] `gmCWInterface::RequestPetMode` · 00450430 · legions 00451570
- [ ] `gmCWInterface::RequestSetPetName` · 00450460 · legions 004515a0
- [ ] `gmCWInterface::MoveItem` · 00450490 · legions 004515d0 — ac2-weenie OnlineSession::move_item / equip_item / unequip_item / set_container_open: the CContainer/CInventorySystem stand-in builds InvMoveDesc/InvEquipDesc byte-exact; the scripts' local checks and optimistic updates not mirrored
- [ ] `gmCWInterface::EquipItem` · 004504f0 · legions 00451630 — ac2-weenie OnlineSession::move_item / equip_item / unequip_item / set_container_open: the CContainer/CInventorySystem stand-in builds InvMoveDesc/InvEquipDesc byte-exact; the scripts' local checks and optimistic updates not mirrored
- [ ] `gmCWInterface::UnEquipItem` · 00450530 · legions 00451670 — ac2-weenie OnlineSession::move_item / equip_item / unequip_item / set_container_open: the CContainer/CInventorySystem stand-in builds InvMoveDesc/InvEquipDesc byte-exact; the scripts' local checks and optimistic updates not mirrored
- [x] `gmCWInterface::GetContentsInfo` · 00450570 · legions 004516b0 — ac2-client Client::contents_info / container_segment_info (Container 0x3D / 0x3F on the VM); the double sends ContainerSegmentDescriptor segments
- [x] `gmCWInterface::GetContainerSegmentInfo` · 004505a0 · legions 004516e0 — ac2-client Client::contents_info / container_segment_info (Container 0x3D / 0x3F on the VM); the double sends ContainerSegmentDescriptor segments
- [x] `gmCWInterface::GetPreferredInventoryLocation` · 004505d0 · legions 00451710 — ac2-weenie ItemStats::preferred_location (Item 0x26: int 282), valid_locations (275), precluded_locations (283)
- [ ] `gmCWInterface::InqSimpleInvByLocTable` · 00450600 · legions 00451740
- [ ] `gmCWInterface::InqAllInventoryIIDList` · 00450620 · legions 00451760
- [ ] `gmCWInterface::DoIHaveItem` · 00450640 · legions 00451780
- [ ] `gmCWInterface::IsOnInventory` · 00450680 · legions 004517c0
- [~] `gmCWInterface::DragToMoneyBag` · 004506c0 · legions 00451800 — ac2-weenie OnlineSession::drag_to_money_bag / drag_from_money_bag (CMoneySystem 0x4 / 0x5 -> WM_Money 0x7 / 0x8, receive_money for 0x6 / 0x9); MoneySystem 0x6's ownership checks and 0x2's room check not mirrored
- [~] `gmCWInterface::DragFromMoneyBag` · 00450700 · legions 00451840 — ac2-weenie OnlineSession::drag_to_money_bag / drag_from_money_bag (CMoneySystem 0x4 / 0x5 -> WM_Money 0x7 / 0x8, receive_money for 0x6 / 0x9); MoneySystem 0x6's ownership checks and 0x2's room check not mirrored
- [ ] `gmCWInterface::LootAllFromContainer` · 00450730 · legions 00451870 — ac2-weenie OnlineSession::loot_all / transmute_all send InvTakeAllDesc / InvTransmuteAllDesc byte-exact; the scripts' checks (CInventorySystem 0x1E/0x1F) not mirrored
- [ ] `gmCWInterface::TransmuteAllFromContainer` · 00450750 · legions 00451890 — ac2-weenie OnlineSession::loot_all / transmute_all send InvTakeAllDesc / InvTransmuteAllDesc byte-exact; the scripts' checks (CInventorySystem 0x1E/0x1F) not mirrored
- [x] `gmCWInterface::HasDurability` · 00450770 · legions 004518b0 — ac2-weenie ItemStats::of (gmEntity 0x10/0x23/0xB/0x158/0x159/0x15B over the Inq helpers); ac2-client InventoryFrame::items, Client::item_display
- [x] `gmCWInterface::GetDurabilityLevel` · 004507a0 · legions 004518e0 — ac2-weenie ItemStats::of (gmEntity 0x10/0x23/0xB/0x158/0x159/0x15B over the Inq helpers); ac2-client InventoryFrame::items, Client::item_display
- [x] `gmCWInterface::GetMaxDurabilityLevel` · 004507d0 · legions 00451910 — ac2-weenie ItemStats::of (gmEntity 0x10/0x23/0xB/0x158/0x159/0x15B over the Inq helpers); ac2-client InventoryFrame::items, Client::item_display
- [x] `gmCWInterface::GetIconID` · 00450800 · legions 00451940 — ac2-weenie ItemStats::of (gmEntity 0x10/0x23/0xB/0x158/0x159/0x15B over the Inq helpers); ac2-client InventoryFrame::items, Client::item_display
- [~] `gmCWInterface::GetMaxContainerSize` · 00450830 · legions 00451970 — ItemStats::capacity: int 280 default 12; the EffectSystem pass is not applied
- [ ] `gmCWInterface::GetAvailableExperience` · 00450860 · legions 004519a0 — ac2-weenie Experience::available_xp (Agent 0x7), Skills::credits / hero_credits (Agent 0x2C/0xBE)
- [ ] `gmCWInterface::GetSkillCredits` · 004508a0 · legions 004519e0 — ac2-weenie Experience::available_xp (Agent 0x7), Skills::credits / hero_credits (Agent 0x2C/0xBE)
- [ ] `gmCWInterface::GetHeroSkillCredits` · 004508d0 · legions 00451a10 — ac2-weenie Experience::available_xp (Agent 0x7), Skills::credits / hero_credits (Agent 0x2C/0xBE)
- [ ] `gmCWInterface::Dismount` · 00450900 · legions 00451a40
- [ ] `gmCWInterface::InqRootPanelFromRace` · 00450920 · legions 00451a60
- [ ] `gmCWInterface::IsSkillTrainedByEnum` · 00450960 · legions 00451aa0 — ac2-weenie Skills::is_trained / is_being_untrained / xp (SkillSystem 0x14/0x15/0x1F over the repository mirror)
- [ ] `gmCWInterface::IsSkillBeingUntrainedByEnum` · 004509a0 · legions 00451ae0 — ac2-weenie Skills::is_trained / is_being_untrained / xp (SkillSystem 0x14/0x15/0x1F over the repository mirror)
- [ ] `gmCWInterface::IsHeroSkillByEnum` · 004509e0 · legions 00451b20 — ac2-client Client::can_skill_be_trained/untrained/raised, skill_cost_to_train, is_hero_skill: the IFun on the script VM (Client::call_ifun's retval)
- [ ] `gmCWInterface::CanSkillBeTrainedByEnum` · 00450a20 · legions 00451b60 — ac2-client Client::can_skill_be_trained/untrained/raised, skill_cost_to_train, is_hero_skill: the IFun on the script VM (Client::call_ifun's retval)
- [ ] `gmCWInterface::CanSkillBeUntrainedByEnum` · 00450a60 · legions 00451ba0 — ac2-client Client::can_skill_be_trained/untrained/raised, skill_cost_to_train, is_hero_skill: the IFun on the script VM (Client::call_ifun's retval)
- [ ] `gmCWInterface::CanSkillBeRaisedByEnum` · 00450aa0 · legions 00451be0 — ac2-client Client::can_skill_be_trained/untrained/raised, skill_cost_to_train, is_hero_skill: the IFun on the script VM (Client::call_ifun's retval)
- [ ] `gmCWInterface::GetSkillExperienceNeededToLevelByEnum` · 00450ae0 · legions 00451c20
- [ ] `gmCWInterface::InqSkillLevelByEnum` · 00450b30 · legions 00451c70
- [ ] `gmCWInterface::GetSkillCostToTrainByEnum` · 00450b70 · legions 00451cb0 — ac2-client Client::can_skill_be_trained/untrained/raised, skill_cost_to_train, is_hero_skill: the IFun on the script VM (Client::call_ifun's retval)
- [ ] `gmCWInterface::GetSkillXPByEnum` · 00450bb0 · legions 00451cf0 — ac2-weenie Skills::is_trained / is_being_untrained / xp (SkillSystem 0x14/0x15/0x1F over the repository mirror)
- [ ] `gmCWInterface::RequestTrainSkill` · 00450c00 · legions 00451d40 — ac2-weenie OnlineSession::request_skill(SkillRequest) -> WM_Skill 0x13/0x12/0x10/0xF (CPlayer 0x53/0xF3/0x124/0xB1); ac2-client Command::Skill
- [ ] `gmCWInterface::RequestUntrainSkill` · 00450c20 · legions 00451d60 — ac2-weenie OnlineSession::request_skill(SkillRequest) -> WM_Skill 0x13/0x12/0x10/0xF (CPlayer 0x53/0xF3/0x124/0xB1); ac2-client Command::Skill
- [ ] `gmCWInterface::RequestRaiseSkill` · 00450c40 · legions 00451d80 — ac2-weenie OnlineSession::request_skill(SkillRequest) -> WM_Skill 0x13/0x12/0x10/0xF (CPlayer 0x53/0xF3/0x124/0xB1); ac2-client Command::Skill
- [ ] `gmCWInterface::RequestUntrainSkillCancel` · 00450c60 · legions 00451da0 — ac2-weenie OnlineSession::request_skill(SkillRequest) -> WM_Skill 0x13/0x12/0x10/0xF (CPlayer 0x53/0xF3/0x124/0xB1); ac2-client Command::Skill
- [ ] `gmCWInterface::InqSkillTimeLeft` · 00450c80 · legions 00451dc0
- [ ] `gmCWInterface::IsSkillUsableBasedOnInventoryByEnum` · 00450cc0 · legions 00451e00
- [ ] `gmCWInterface::GUNHandleReauthentication` · 00450d00 · legions 00451e40
- [ ] `gmCWInterface::Result_SendToUser` · 00450d10 · legions 00451e50
- [ ] `gmCWInterface::Result_SendToRoom` · 00450d40 · legions 00451e80
- [ ] `gmCWInterface::Result_FriendAdd` · 00450d70 · legions 00451eb0
- [ ] `gmCWInterface::Result_FriendDelete` · 00450d90 · legions 00451ed0
- [ ] `gmCWInterface::GUNEvent_FriendStatus` · 00450db0 · legions 00451ef0
- [ ] `gmCWInterface::Result_IgnoredAdd` · 00450de0 · legions 00451f20 — ac2-weenie ChatState::ignored_added/ignored_deleted/ignored_list (CPlayer 0x32/0x5D/0x7); nothing calls them without a GUN service
- [ ] `gmCWInterface::Result_IgnoredDelete` · 00450e00 · legions 00451f40 — ac2-weenie ChatState::ignored_added/ignored_deleted/ignored_list (CPlayer 0x32/0x5D/0x7); nothing calls them without a GUN service
- [ ] `gmCWInterface::Result_CallCreate` · 00450e20 · legions 00451f60
- [~] `gmCWInterface::InqEffectList` · 00450e60 · legions 00451fa0 — ac2-client Client::effect_list (IFun 0xB40062, CPlayer 0x52 -> CEffectSystem 0x7); waits on the VM's gEffectUINode efuns
- [x] `gmCWInterface::GetStackable` · 00450ea0 · legions 00451fe0 — ac2-weenie ItemStats::of (gmEntity 0x10/0x23/0xB/0x158/0x159/0x15B over the Inq helpers); ac2-client InventoryFrame::items, Client::item_display
- [x] `gmCWInterface::GetQuantity` · 00450ed0 · legions 00452010 — ac2-weenie ItemStats::of (gmEntity 0x10/0x23/0xB/0x158/0x159/0x15B over the Inq helpers); ac2-client InventoryFrame::items, Client::item_display
- [~] `gmCWInterface::GetMaxStackSize` · 00450f00 · legions 00452040 — ItemStats::max_stack: int 261; InventoryGlobals slot-5 default only through the VM
- [ ] `gmCWInterface::SetExaminationRequest` · 00450f30 · legions 00452070 — ac2-weenie OnlineSession::examine (stand-in for CPlayer 0xE6)
- [ ] `gmCWInterface::AddEffectDataToProfileByEID` · 00450f50 · legions 00452090
- [ ] `gmCWInterface::StartExamination` · 00450f80 · legions 004520c0 — ac2-weenie OnlineSession::select/start_examination/... over the PlayerScript stand-in for CPlayer 0x6A/0x3D/0x63/0x114/0xF6
- [ ] `gmCWInterface::StopExamination` · 00450fa0 · legions 004520e0 — ac2-weenie OnlineSession::select/start_examination/... over the PlayerScript stand-in for CPlayer 0x6A/0x3D/0x63/0x114/0xF6
- [ ] `gmCWInterface::HasSomethingSelected` · 00450fc0 · legions 00452100
- [ ] `gmCWInterface::RegisterCommands` · 00450ff0 · legions 00452130
- [ ] `gmCWInterface::RequestResurrect` · 00451010 · legions 00452150
- [ ] `gmCWInterface::ClearResurrectionRequest` · 00451030 · legions 00452170
- [ ] `gmCWInterface::UpdateOpen` · 00451050 · legions 00452190 — ac2-weenie OnlineSession::move_item / equip_item / unequip_item / set_container_open: the CContainer/CInventorySystem stand-in builds InvMoveDesc/InvEquipDesc byte-exact; the scripts' local checks and optimistic updates not mirrored
- [ ] `gmCWInterface::InqShortcutInfo` · 00451090 · legions 004521d0
- [ ] `gmCWInterface::SetShortcut` · 004510d0 · legions 00452210
- [ ] `gmCWInterface::InqShortcutArray` · 00451110 · legions 00452250
- [x] `gmCWInterface::SetWhichShortcutSet` · 00451150 — ac2-client Command::SetShortcutSet: the IFun on the VM, the session's shortcut mirror (a profile write, nothing to the server)
- [ ] `gmCWInterface::GetWhichShortcutSet` · 00451170 · legions 004522b0
- [ ] `gmCWInterface::SetShowShortcutNumbers` · 004511a0 · legions 004522e0
- [ ] `gmCWInterface::GetShowShortcutNumbers` · 004511c0 · legions 00452300
- [ ] `gmCWInterface::SetShortcutLocked` · 004511f0 · legions 00452330
- [ ] `gmCWInterface::GetShortcutLocked` · 00451210 · legions 00452350
- [ ] `gmCWInterface::SetShortcutInstaScroll` · 00451240 · legions 00452380
- [ ] `gmCWInterface::GetShortcutInstaScroll` · 00451260 · legions 004523a0
- [ ] `gmCWInterface::SetShowPKDamage` · 00451290 · legions 004523d0
- [ ] `gmCWInterface::GetShowPKDamage` · 004512b0 · legions 004523f0
- [ ] `gmCWInterface::SetShowDamageText` · 004512e0 · legions 00452420
- [ ] `gmCWInterface::GetShowDamageText` · 00451300 · legions 00452440
- [ ] `gmCWInterface::SetShowDamageTextOther` · 00451330 · legions 00452470
- [ ] `gmCWInterface::GetShowDamageTextOther` · 00451350 · legions 00452490
- [ ] `gmCWInterface::SetDamageTextRangeOther` · 00451380 · legions 004524c0
- [ ] `gmCWInterface::GetDamageTextRangeOther` · 004513a0 · legions 004524e0
- [ ] `gmCWInterface::SetShortcutHeight` · 004513d0 · legions 00452510
- [ ] `gmCWInterface::GetShortcutHeight` · 004513f0 · legions 00452530
- [ ] `gmCWInterface::SetBypassGoldMeltingConfirmationDlg` · 00451420 · legions 00452560
- [x] `gmCWInterface::GetBypassGoldMeltingConfirmationDlg` · 00451440 · legions 00452580 — ac2-ui money_bag.rs / item_display.rs / backpack.rs; ac2-client ui_inventory.rs; ac2-script host/inventory_globals.rs (coin max) and the player's IFUN 0xB400D7 (bypass); Legions 7160 matches admin here (same IFUNs 0xA60001 and 0xB400D7)
- [ ] `gmCWInterface::SetLatencyIndicatorGraph` · 00451470 · legions 004525b0
- [ ] `gmCWInterface::GetLatencyIndicatorGraph` · 00451490 · legions 004525d0
- [ ] `gmCWInterface::SetShowFellowsOnMap` · 004514c0 · legions 00452600
- [ ] `gmCWInterface::GetShowFellowsOnMap` · 004514e0 · legions 00452620
- [ ] `gmCWInterface::SetShowPetUI` · 00451510 · legions 00452650
- [ ] `gmCWInterface::GetShowPetUI` · 00451530 · legions 00452670
- [ ] `gmCWInterface::SetUISaveLocations` · 00451560 · legions 00452290
- [ ] `gmCWInterface::InqUISaveLocations` · 00451580 · legions 004526c0
- [x] `gmCWInterface::SetShowRadarCoords` · 004515c0 · legions 00452700 — ac2-ui radar.rs + ac2-client (Command::SetShowRadarCoords, IFun 0xB4007D; show_radar_coords from the options profile)
- [x] `gmCWInterface::GetShowRadarCoords` · 004515e0 · legions 00452720 — ac2-ui radar.rs + ac2-client (Command::SetShowRadarCoords, IFun 0xB4007D; show_radar_coords from the options profile)
- [x] `gmCWInterface::SetShowObjectNames` · 00451610 · legions 00452750 — ac2-ui name_plates.rs + ac2-client (Command::SetShowObjectNames, IFun 0xB400CE)
- [x] `gmCWInterface::GetShowObjectNames` · 00451630 · legions 00452770 — ac2-ui name_plates.rs + ac2-client (Command::SetShowObjectNames, IFun 0xB400CE)
- [ ] `gmCWInterface::SetRadarMask` · 00451660 · legions 004527a0
- [ ] `gmCWInterface::GetRadarMask` · 00451680 · legions 004527c0
- [ ] `gmCWInterface::SetChatWindowFilter` · 004516b0 · legions 004527f0
- [ ] `gmCWInterface::SetChatFontSize` · 004516f0 · legions 00452830
- [ ] `gmCWInterface::SetChatFontColor` · 00451730 · legions 00452870
- [ ] `gmCWInterface::SetWindowOpacity` · 00451770 · legions 004528f0
- [ ] `gmCWInterface::SetChatPopupFlag` · 004517b0 · legions 00452930
- [ ] `gmCWInterface::InqGameplayOptions` · 004517f0 · legions 00452970
- [ ] `gmCWInterface::SetAutoTarget` · 00451820 · legions 004529a0
- [ ] `gmCWInterface::GetAutoTarget` · 00451840 · legions 004529c0
- [ ] `gmCWInterface::SetAcceptAllegianceRequest` · 00451870 · legions 004529f0
- [ ] `gmCWInterface::GetAcceptAllegianceRequest` · 00451890 · legions 00452a10
- [ ] `gmCWInterface::SetAcceptFellowshipRequest` · 004518c0 · legions 00452a40
- [ ] `gmCWInterface::GetAcceptFellowshipRequest` · 004518e0 · legions 00452a60
- [ ] `gmCWInterface::SetAcceptTradeRequest` · 00451910 · legions 00452a90
- [ ] `gmCWInterface::GetAcceptTradeRequest` · 00451930 · legions 00452ab0
- [ ] `gmCWInterface::InqNextRetellHistoryItem` · 00451960 · legions 00452ae0 — ac2-weenie shell::HistoryList (CPlayer 0x75/0x4B/0x11D/0x46 over HistoryList 0x3/0x8): OnlineSession::chat_history(Retell|Reply, prev)
- [ ] `gmCWInterface::InqPrevRetellHistoryItem` · 00451990 · legions 00452b10 — ac2-weenie shell::HistoryList (CPlayer 0x75/0x4B/0x11D/0x46 over HistoryList 0x3/0x8): OnlineSession::chat_history(Retell|Reply, prev)
- [ ] `gmCWInterface::InqNextReplyHistoryItem` · 004519c0 · legions 00452b40 — ac2-weenie shell::HistoryList (CPlayer 0x75/0x4B/0x11D/0x46 over HistoryList 0x3/0x8): OnlineSession::chat_history(Retell|Reply, prev)
- [ ] `gmCWInterface::InqPrevReplyHistoryItem` · 004519f0 · legions 00452b70 — ac2-weenie shell::HistoryList (CPlayer 0x75/0x4B/0x11D/0x46 over HistoryList 0x3/0x8): OnlineSession::chat_history(Retell|Reply, prev)
- [ ] `gmCWInterface::InqQuestList` · 00451a20 · legions 00452ba0
- [ ] `gmCWInterface::InqActInfoList` · 00451a60 · legions 00452be0
- [ ] `gmCWInterface::HandlePlayScenesDone` · 00451aa0 · legions 00452c20
- [ ] `gmCWInterface::UploadGameOptions` · 00451ac0 · legions 00452c40
- [ ] `gmCWInterface::UploadChatOptions` · 00451ae0 · legions 00452c60
- [ ] `gmCWInterface::UploadUISettings` · 00451b00 · legions 00452c80
- [x] `gmCWInterface::DoFellowshipCreate` · 00451b20 · legions 00452ca0 — ac2-client Command::Social(SocialRequest::Fellowship*): the IFun on the VM, else OnlineSession::social_request (WM_Fellowship stubs, byte-exact)
- [x] `gmCWInterface::DoFellowshipDisband` · 00451b50 · legions 00452cd0 — ac2-client Command::Social(SocialRequest::Fellowship*): the IFun on the VM, else OnlineSession::social_request (WM_Fellowship stubs, byte-exact)
- [x] `gmCWInterface::DoFellowshipLeave` · 00451b70 · legions 00452cf0 — ac2-client Command::Social(SocialRequest::Fellowship*): the IFun on the VM, else OnlineSession::social_request (WM_Fellowship stubs, byte-exact)
- [x] `gmCWInterface::DoFellowshipRecruit` · 00451b90 · legions 00452d10 — ac2-client Command::Social(SocialRequest::Fellowship*): the IFun on the VM, else OnlineSession::social_request (WM_Fellowship stubs, byte-exact)
- [x] `gmCWInterface::DoFellowshipDismiss` · 00451bb0 · legions 00452d30 — ac2-client Command::Social(SocialRequest::Fellowship*): the IFun on the VM, else OnlineSession::social_request (WM_Fellowship stubs, byte-exact)
- [x] `gmCWInterface::DoFellowshipRecruitAccept` · 00451bd0 · legions 00452d50 — ac2-client Command::Social(SocialRequest::Fellowship*): the IFun on the VM, else OnlineSession::social_request (WM_Fellowship stubs, byte-exact)
- [x] `gmCWInterface::DoFellowshipRecruitDecline` · 00451bf0 · legions 00452d70 — ac2-client Command::Social(SocialRequest::Fellowship*): the IFun on the VM, else OnlineSession::social_request (WM_Fellowship stubs, byte-exact)
- [ ] `gmCWInterface::IsFellowshipMember` · 00451c10 · legions 00452d90
- [x] `gmCWInterface::GetFellowXPHash` · 00451c50 · legions 00452dd0 — ac2-client Client::fellow_xp_hash: an empty LAHash out parameter to IFun 0xB400A8 (CPlayer 0x2C -> FellowshipSystem 0x41), read back as (id, share)
- [ ] `gmCWInterface::DoAssistedTell` · 00451c90 · legions 00452e10
- [ ] `gmCWInterface::HandleWeenieEmote` · 00451cb0 · legions 00452e30
- [x] `gmCWInterface::QueryAllegianceProfile` · 00451cf0 · legions 00452e70 — ac2-client Command::Social(SocialRequest allegiance actions): the IFun on the VM, else OnlineSession::social_request (WM_Allegiance stubs)
- [x] `gmCWInterface::SwearAllegiance` · 00451d10 · legions 00452e90 — ac2-client Command::Social(SocialRequest allegiance actions): the IFun on the VM, else OnlineSession::social_request (WM_Allegiance stubs)
- [x] `gmCWInterface::BreakAllegiance` · 00451d30 · legions 00452eb0 — ac2-client Command::Social(SocialRequest allegiance actions): the IFun on the VM, else OnlineSession::social_request (WM_Allegiance stubs)
- [x] `gmCWInterface::AcceptVassal` · 00451d50 · legions 00452ed0 — ac2-client Command::Social(SocialRequest allegiance actions): the IFun on the VM, else OnlineSession::social_request (WM_Allegiance stubs)
- [x] `gmCWInterface::DeclineVassal` · 00451d70 · legions 00452ef0 — ac2-client Command::Social(SocialRequest allegiance actions): the IFun on the VM, else OnlineSession::social_request (WM_Allegiance stubs)
- [x] `gmCWInterface::OpenTradeNegotiations` · 00451d90 · legions 00452f10 — ac2-weenie session/trade.rs: open_trade/close_trade/decline_trade/offer_item_for_trade/revoke_item_from_trade send the WM_Trade stubs (0x2F, 0x3E, 0x23, 0x15, 0x3F) as ScriptRequest; trade_7160
- [x] `gmCWInterface::CloseTradeNegotiations` · 00451db0 · legions 00452f30 — ac2-weenie session/trade.rs: open_trade/close_trade/decline_trade/offer_item_for_trade/revoke_item_from_trade send the WM_Trade stubs (0x2F, 0x3E, 0x23, 0x15, 0x3F) as ScriptRequest; trade_7160
- [~] `gmCWInterface::AcceptTrade` · 00451dd0 · legions 00452f50 — accept_trade sends WM_Trade stub 0x35 with an empty PackedReference; retail packs the accepting side's CWInterface instance graph (not ported)
- [x] `gmCWInterface::DeclineTrade` · 00451df0 · legions 00452f70 — ac2-weenie session/trade.rs: open_trade/close_trade/decline_trade/offer_item_for_trade/revoke_item_from_trade send the WM_Trade stubs (0x2F, 0x3E, 0x23, 0x15, 0x3F) as ScriptRequest; trade_7160
- [x] `gmCWInterface::OfferItemForTrade` · 00451e10 · legions 00452f90 — ac2-weenie session/trade.rs: open_trade/close_trade/decline_trade/offer_item_for_trade/revoke_item_from_trade send the WM_Trade stubs (0x2F, 0x3E, 0x23, 0x15, 0x3F) as ScriptRequest; trade_7160
- [x] `gmCWInterface::RevokeItemFromTrade` · 00451e40 · legions 00452fc0 — ac2-weenie session/trade.rs: open_trade/close_trade/decline_trade/offer_item_for_trade/revoke_item_from_trade send the WM_Trade stubs (0x2F, 0x3E, 0x23, 0x15, 0x3F) as ScriptRequest; trade_7160
- [ ] `gmCWInterface::GetMissileInformationFromWSL` · 00451e60 · legions 00452fe0
- [ ] `gmCWInterface::GetFXAndBehaviorInformationFromWsl` · 00451eb0 · legions 00453030
- [ ] `gmCWInterface::DoUIDamage` · 00451ef0 · legions 00453070
- [ ] `gmCWInterface::SpecialAttackStarted` · 00451f50 · legions 004530d0
- [ ] `gmCWInterface::SpecialAttackEnded` · 00451f70 · legions 004530f0
- [ ] `gmCWInterface::StopAttackingIfAutoMoving` · 00451f90 · legions 00453110
- [x] `gmCWInterface::InqRadarLook` · 00451fb0 · legions 00453130 — ac2-client client_radar.rs + ac2-script host/radar.rs (InqRadarLook), ac2-weenie notice.rs (ChangeRadarLook(obj))
- [ ] `gmCWInterface::IsSkillHarmful` · 00451ff0 · legions 00453170
- [ ] `gmCWInterface::HandleSetWeenieDesc` · 00452030 · legions 004531b0

### GunChatExtraInfo
- [ ] `GunChatExtraInfo::PackIntoBlob` · 004520f0 · legions 00453230 — ac2-net GunChatExtraInfo

### gmCWInterface
- [ ] `gmCWInterface::ProcessText_Done` · 004521b0 · legions 004532f0 — ac2-weenie ChatState::process_text resets the told/heard walks (CPlayer 0x43/0xDF)
- [ ] `gmCWInterface::RequestGUN_FriendAdd` · 004521e0 · legions 00453320 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `gmCWInterface::RequestGUN_FriendDelete` · 00452250 · legions 00453390 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `gmCWInterface::RequestGUN_GetFriendsList` · 004522c0 · legions 00453400 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `gmCWInterface::RequestGUN_IgnoredAdd` · 00452330 · legions 00453470 — ac2-weenie squelch: no TurbineGun in Legions, so the request fails with the squelch-service line
- [ ] `gmCWInterface::RequestGUN_IgnoredDelete` · 004523a0 · legions 004534e0 — ac2-weenie squelch: no TurbineGun in Legions, so the request fails with the squelch-service line
- [ ] `gmCWInterface::RequestGUN_GetIgnoredList` · 00452410 · legions 00453550 — ac2-weenie squelch: no TurbineGun in Legions, so the request fails with the squelch-service line
- [ ] `gmCWInterface::RequestGUN_CallCreate` · 00452480 · legions 004535d2 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host

### GunChatExtraInfo
- [ ] `GunChatExtraInfo::GunChatExtraInfo` · 00452580 · legions 00453690 — ac2-net GunChatExtraInfo::new: version 1, flags; the meta letters (RetrieveMetaLetters) are left empty
- [ ] `GunChatExtraInfo::~GunChatExtraInfo` · 00452660 · legions 00453770

### gmCWInterface
- [ ] `gmCWInterface::RequestGUN_SendToUser` · 00452710 · legions 00453820 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `gmCWInterface::RequestGUN_SendToRoom` · 004527c0 · legions 004538d0 — ac2-weenie OnlineSession::send_to_room: the Turbine chat path (CSendToTurbineRoomByID); the GUN path is dead in Legions

### CCliFlagsResult
- [ ] `CCliFlagsResult::CCliFlagsResult` · 004528d0 · legions 004539e0

### CCliLocationResult
- [ ] `CCliLocationResult::CCliLocationResult` · 00452960 · legions 00453a70

### `anonymous-namespace'::YesNoDialogAwakener
- [ ] ``anonymous-namespace'::YesNoDialogAwakener::YesNoDialogAwakener` · 004529f0 · legions 00453b00
- [ ] ``anonymous-namespace'::YesNoDialogAwakener::RecvNotice_CloseConfirmationDialog` · 00452a70

### PStringBaseIter_Common<unsigned_short>
- [ ] `PStringBaseIter_Common<unsigned_short>::GetSubString` · 00452ae0 · legions 00453bf0

### gmCWInterface
- [ ] `gmCWInterface::DumpWeenieDesc` · 00452bf0 · legions 00453d00
- [ ] `gmCWInterface::GetName` · 00452cd0 · legions 00453de0
- [ ] `gmCWInterface::GetPluralName` · 00452d50 · legions 00453e60
- [ ] `gmCWInterface::RequestGUN_SetStateFlags` · 00452dd0 · legions 00453ee0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `gmCWInterface::RequestGUN_SetLocation` · 00452e40 · legions 00453f50 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host

### PStringBaseIter_Common<unsigned_short>
- [ ] `PStringBaseIter_Common<unsigned_short>::GetSubString` · 00452f50 · legions 00454060

### gmCWInterface
- [ ] `gmCWInterface::HearTell` · 004530f0 · legions 00454200
- [ ] `gmCWInterface::GUNEvent_HearFromRoom` · 00453320 · legions 00454430 — ac2-weenie GameplayNotice::HearFromRoom / SendToRoomResult; the CPlayer script's channel formatting is not mirrored

### GunChatExtraInfo
- [ ] `GunChatExtraInfo::StreamPack` · 004534e0 · legions 004545f0 — ac2-net GunChatExtraInfo

### (free functions)
- [ ] `STREAMPACK` · 00453560 · legions 005e6f90
- [ ] `STREAMPACK_PACKOBJ` · 004535f0 · legions 00454670

### gmCWInterface
- [ ] `gmCWInterface::HandleAlias` · 00453670 · legions 004546f0 — ac2-weenie shell::expand_aliases over the options profile's m_aliasTable (ChatState::login_options)

### PList<PStringBase<unsigned_short>_>
- [ ] `PList<PStringBase<unsigned_short>_>::Pack` · 004537f0 · legions 00454870
- [ ] `PList<PStringBase<unsigned_short>_>::GetPackSize` · 00453840
- [ ] `PList<PStringBase<unsigned_short>_>::UnPack` · 00453880

### gmCWInterface
- [ ] `gmCWInterface::ExpandText` · 004539e0 · legions 00454a60
- [ ] `gmCWInterface::Result_GetFriendsList` · 00453b90
- [ ] `gmCWInterface::GUNEvent_FriendStart` · 00453bd0 · legions 00454c50
- [ ] `gmCWInterface::Result_GetIgnoredList` · 00453c10 — ac2-weenie ChatState::ignored_added/ignored_deleted/ignored_list (CPlayer 0x32/0x5D/0x7); nothing calls them without a GUN service

### `anonymous-namespace'::YesNoDialogAwakener
- [ ] ``anonymous-namespace'::YesNoDialogAwakener::OnTrigger` · 00453de0 · legions 00454e60

### gmCWInterface
- [ ] `gmCWInterface::PopupYesNoDialog` · 00453e30 · legions 00454eb0

## `portal/client/src/action/interp/actioninterp.cpp.c` (0/23)

### ActionInterpreter
- [ ] `ActionInterpreter::Shutdown` · 00478fe0 · legions 00474e20
- [ ] `ActionInterpreter::Get` · 00479010 · legions 00474e50
- [ ] `ActionInterpreter::OnDDDComplete` · 00479020
- [ ] `ActionInterpreter::StopAction` · 00479030 · legions 00474e70 — ac2-ui interpreter.rs
- [ ] `ActionInterpreter::GetDescripValues` · 004790b0 · legions 00474ef0
- [ ] `ActionInterpreter::SetPlayerID` · 004790d0 · legions 00474f10
- [ ] `ActionInterpreter::MouseLookHandler` · 004790f0 · legions 00474f30
- [ ] `ActionInterpreter::HandleDoubleTap` · 00479110 · legions 00474f50 — ac2-ui interpreter.rs
- [ ] `ActionInterpreter::FindAction` · 004791b0 · legions 00474ff0 — ac2-ui interpreter.rs
- [ ] `ActionInterpreter::ActionDown` · 004791e0 · legions 00475020 — ac2-ui interpreter.rs
- [ ] `ActionInterpreter::GetModifierBitfield` · 00479210 · legions 00475050 — ac2-ui interpreter.rs
- [ ] `ActionInterpreter::CtrlKeyDown` · 004792d0 · legions 00475110
- [ ] `ActionInterpreter::ShiftKeyDown` · 004792e0 · legions 00475120
- [ ] `ActionInterpreter::AltKeyDown` · 00479300 · legions 00475140
- [ ] `ActionInterpreter::GetActionValues` · 00479310 · legions 00475150 — ac2-ui interpreter.rs
- [ ] `ActionInterpreter::StartAction` · 004793d0 · legions 00475210 — ac2-ui interpreter.rs
- [ ] `ActionInterpreter::PurgeSetActions` · 004794c0 · legions 00475300 — ac2-ui interpreter.rs
- [ ] `ActionInterpreter::UseTime` · 00479530 · legions 00475370 — ac2-ui interpreter.rs
- [ ] `ActionInterpreter::TurnOffRunLock` · 00479540 · legions 00475380 — ac2-ui interpreter.rs
- [ ] `ActionInterpreter::~ActionInterpreter` · 00479790 · legions 004755d0
- [ ] `ActionInterpreter::Startup` · 004797f0 · legions 00475630 — ac2-ui controls.rs (no ICIDM handler registration)
- [ ] `ActionInterpreter::FocusSwitchHandler` · 00479890 · legions 004756d0 — ac2-ui interpreter.rs
- [ ] `ActionInterpreter::ActionHandler` · 004799c0 · legions 00475800 — ac2-ui interpreter.rs

## `portal/client/src/action/map/actionmap.cpp.c` (0/16)

### ActionMap
- [ ] `ActionMap::GetMapper` · 0072d640 · legions 00613a40

### _STL::less<ActionWithModifiers>
- [ ] `_STL::less<ActionWithModifiers>::operator()` · 0072d6a0 · legions 006ab720

### ActionMap
- [ ] `ActionMap::Pack` · 0072d770
- [ ] `ActionMap::GetToggleType` · 0072da60 · legions 006abae0 — ac2-ui maps.rs
- [ ] `ActionMap::GetActionValues` · 0072da90 · legions 006abb10 — ac2-ui maps.rs
- [ ] `ActionMap::GetDescripValues` · 0072daf0 · legions 006abb70
- [ ] `ActionMap::~ActionMap` · 0072e690 · legions 006ac710
- [ ] `ActionMap::GetDBOType` · 0072e760
- [ ] `ActionMap::Destroy` · 0072e770 · legions 006ac7f0

### _STL::map<unsigned_long,unsigned_long,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>
- [ ] `_STL::map<unsigned_long,unsigned_long,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>::operator[]` · 0072e800 · legions 006ac880

### _STL::map<ActionWithModifiers,ActionMapValue,_STL::less<ActionWithModifiers>,_STL::allocator<_STL::pair<ActionWithModifiers_const_,ActionMapValue>_>_>
- [ ] `_STL::map<ActionWithModifiers,ActionMapValue,_STL::less<ActionWithModifiers>,_STL::allocator<_STL::pair<ActionWithModifiers_const_,ActionMapValue>_>_>::operator[]` · 0072e880 · legions 006ac900

### _STL::map<unsigned_long,DescripValue,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,DescripValue>_>_>
- [ ] `_STL::map<unsigned_long,DescripValue,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,DescripValue>_>_>::operator[]` · 0072e900 · legions 006ac980

### ActionMap
- [ ] `ActionMap::ActionMap` · 0072e980 · legions 006aca00
- [-] `ActionMap::FromFileNode` · 0072ea40 — text source format (admin tools)
- [ ] `ActionMap::UnPack` · 0072f200 — ac2-ui maps.rs
- [ ] `ActionMap::Allocate` · 0072f370

## `portal/client/src/action/movement/playermovementmodule.cpp.c` (0/17)

### PlayerMovementModule
- [ ] `PlayerMovementModule::Startup` · 00568e20 · legions 00538db0
- [ ] `PlayerMovementModule::Get` · 00568e90 · legions 00538e20
- [ ] `PlayerMovementModule::ShouldSendPositionEvent` · 00568ea0 · legions 00538e30
- [ ] `PlayerMovementModule::SendPositionEvent` · 00569020 · legions 00538fb0
- [ ] `PlayerMovementModule::ShouldSendLookAtEvent` · 00569200 · legions 00539190
- [ ] `PlayerMovementModule::SendLookAtEvent` · 005692e0 · legions 00539270
- [ ] `PlayerMovementModule::MouseMove` · 00569380 · legions 00539310 — ac2-client camera_mouse.rs / input_handlers
- [ ] `PlayerMovementModule::DoBehavior` · 005693a0 · legions 00539330 — ac2-client camera_mouse.rs / input_handlers
- [ ] `PlayerMovementModule::StopBehavior` · 005693d0 · legions 00539360 — ac2-client camera_mouse.rs / input_handlers
- [ ] `PlayerMovementModule::StartCameraAction` · 00569400 · legions 00539390 — ac2-client camera_mouse.rs / input_handlers
- [ ] `PlayerMovementModule::StopCameraAction` · 00569420 · legions 005393b0 — ac2-client camera_mouse.rs / input_handlers
- [ ] `PlayerMovementModule::LockActions` · 00569440 · legions 005393d0
- [ ] `PlayerMovementModule::InterruptMoveTo` · 00569450 · legions 005393e0 — ac2-client camera_mouse.rs / input_handlers
- [ ] `PlayerMovementModule::StopAttackingIfAutoMoving` · 00569470 · legions 00539400
- [ ] `PlayerMovementModule::Shutdown` · 005694a0 · legions 00539430
- [ ] `PlayerMovementModule::SetPlayerID` · 005694f0 · legions 00539480
- [ ] `PlayerMovementModule::UseTime` · 00569550 · legions 005394e0

## `portal/client/src/cfactory/cenginefactory.cpp.c` (0/5)

### CEngineFactory
- [ ] `CEngineFactory::MakeSmartBox_Internal` · 004991f0
- [ ] `CEngineFactory::MakeCellManager_Internal` · 00499210
- [ ] `CEngineFactory::MakeFrillManager_Internal` · 00499230
- [ ] `CEngineFactory::MakeLScapeManager_Internal` · 00499250
- [ ] `CEngineFactory::MakeWorldManager_Internal` · 00499270

## `portal/client/src/cfactory/cfactory.cpp.c` (0/11)

### CFactory
- [ ] `CFactory::CleanUp` · 00499090 · legions 004915a0
- [ ] `CFactory::MakeCObjectMaint` · 004990b0 · legions 004915c0
- [ ] `CFactory::MakeClientCache` · 004990d0 · legions 004915e0
- [ ] `CFactory::MakeUI` · 004990f0 · legions 00491600
- [ ] `CFactory::MakeWorldManager` · 00499110 · legions 00491620
- [ ] `CFactory::MakeCellManager` · 00499130 · legions 00491640
- [ ] `CFactory::MakeFrillManager` · 00499150 · legions 00491660
- [ ] `CFactory::MakeLScapeManager` · 00499170 · legions 00491680
- [ ] `CFactory::MakeClient` · 00499190 · legions 004916a0
- [ ] `CFactory::MakeCEventQueue` · 004991b0 · legions 004916c0
- [ ] `CFactory::MakeGlobalEventHandler` · 004991d0 · legions 004916e0

## `portal/client/src/events/ceventqueue.cpp.c` (0/15)

### CEventQueue
- [ ] `CEventQueue::CEventQueue` · 00493f60 · legions 0048c730
- [ ] `CEventQueue::ProcessVisualOrderedEvent` · 00493f80 · legions 0048c750
- [ ] `CEventQueue::DispatchEvent` · 00494010 · legions 0048c7e0 — every opcode routed; quality, interpreter and admin events kept as raw bodies
- [ ] `CEventQueue::PostEvent` · 004946a0 · legions 0048ce70

### InstanceIDWithStamp
- [ ] `InstanceIDWithStamp::StreamPack` · 004946d0 · legions 0048cea0

### CEventQueue
- [ ] `CEventQueue::ProcessOrderedNetBlob` · 00494720 · legions 0048cef0

### NIList<NetBlob_*>
- [ ] `NIList<NetBlob_*>::AddToTail` · 004947c0 · legions 0048cf90

### CEventQueue
- [ ] `CEventQueue::QueueEvent` · 00494810 · legions 0048cfe0
- [ ] `CEventQueue::ConsiderQueueingEvent` · 00494890 · legions 0048d060
- [ ] `CEventQueue::DestroyEventQueues` · 00494960 · legions 0048d130
- [ ] `CEventQueue::~CEventQueue` · 004949f0 · legions 0048d1c0
- [ ] `CEventQueue::ProcessNetBlob` · 00494a00 · legions 0048d1d0
- [ ] `CEventQueue::FlushQueue` · 00494a70 · legions 0048d240
- [ ] `CEventQueue::FinishEnteringWorld` · 00494b00 · legions 0048d2d0
- [ ] `CEventQueue::UseTime` · 00494b10 · legions 0048d2e0

## `portal/client/src/events/ecm_admin.cpp.c` (0/5)

### benchScope
- [ ] `benchScope::benchScope` · 00493150

### ECM_Admin
- [ ] `ECM_Admin::RecvEvt_AccountBooted` · 00493190 · legions 0048bc20
- [ ] `ECM_Admin::RecvEvt_DisplayStringInfo` · 00493200 · legions 0048bc90
- [ ] `ECM_Admin::RecvEvt_WorldName` · 004932b0 · legions 0048bd40
- [ ] `ECM_Admin::SendNotice_DisplayAdminStringInfo` · 00493320

## `portal/client/src/events/ecm_interp.cpp.c` (0/4)

### ECM_Interp
- [ ] `ECM_Interp::RecvEvt_InterpCEvent_Broadcast` · 00498d60 · legions 00491380
- [ ] `ECM_Interp::RecvEvt_InterpCEvent_Cell` · 00498e20 · legions 00491270
- [ ] `ECM_Interp::SendEvt_InterpSEvent` · 00498f30 · legions 00491440
- [ ] `ECM_Interp::SendEvt_InterpSEventEncrypt` · 00498fe0 · legions 004914f0

## `portal/client/src/events/ecm_login.cpp.c` (0/21)

### ECM_Login
- [ ] `ECM_Login::RecvEvt_CharExitGame` · 004933b0 · legions 0048be00
- [ ] `ECM_Login::RecvEvt_CharacterDeletion` · 004933c0 · legions 0048be10
- [ ] `ECM_Login::RecvEvt_CharacterError` · 00493410 · legions 0048be60
- [ ] `ECM_Login::RecvEvt_PlayerDesc` · 00493460 · legions 0048beb0
- [ ] `ECM_Login::SendEvt_ClientSceneRenderingComplete` · 00493500 · legions 0048bf50
- [ ] `ECM_Login::RecvEvt_CharGenVerification` · 00493520 · legions 0048bf70

### CharacterSet
- [ ] `CharacterSet::~CharacterSet` · 00493680 · legions 0048c0d0

### ECM_Login
- [ ] `ECM_Login::SendNotice_CharExitGame` · 00493700
- [ ] `ECM_Login::SendNotice_CharGenVerification` · 00493780
- [ ] `ECM_Login::SendNotice_CharacterDeletion` · 00493810
- [ ] `ECM_Login::SendNotice_CharacterError` · 004938a0 · legions 0048c230
- [ ] `ECM_Login::SendNotice_CharacterSet` · 00493930
- [ ] `ECM_Login::SendNotice_LoginFinished` · 004939b0
- [ ] `ECM_Login::SendNotice_ServerDied` · 00493a30 — SessionEvent::ServerDied, before Disconnected
- [ ] `ECM_Login::SendNotice_WorldName` · 00493ab0
- [ ] `ECM_Login::RecvEvt_CharacterSet` · 00493b30 · legions 0048c380

### MinimalCharacterSet
- [ ] `MinimalCharacterSet::~MinimalCharacterSet` · 00493bd0 · legions 0048c420
- [ ] `MinimalCharacterSet::StreamPack` · 00493c50 · legions 0048c4a0

### (free functions)
- [ ] `STREAMPACK` · 00493cb0 · legions 0064e070

### ECM_Login
- [ ] `ECM_Login::RecvEvt_MinCharSet` · 00493d30 · legions 0048c500

### PSmartArray<PStringBase<unsigned_short>_>
- [ ] `PSmartArray<PStringBase<unsigned_short>_>::StreamPack` · 00493de0

## `portal/client/src/events/ecm_physics.cpp.c` (0/59)

### TargetPosition
- [ ] `TargetPosition::TargetPosition` · 00494c30 · legions 0048d400

### ECM_Physics
- [ ] `ECM_Physics::RecvEvt_AddTargetedImpulse` · 00494cb0 · legions 0048d480
- [ ] `ECM_Physics::RecvEvt_AddTerrainModifier` · 00494e30 · legions 0048d600
- [ ] `ECM_Physics::RecvEvt_ClearTerrainModifiers` · 00494f30 · legions 0048d700
- [ ] `ECM_Physics::RecvEvt_Contain` · 00494fb0 · legions 0048d780 — ac2-weenie World (the player's RecvEvt_Contain on the child)
- [ ] `ECM_Physics::RecvEvt_CreatePlayer` · 00495060 · legions 0048d830
- [ ] `ECM_Physics::RecvEvt_DeParent` · 00495100 · legions 0048d8d0
- [ ] `ECM_Physics::RecvEvt_DestroyObject` · 004951f0 · legions 0048d9c0
- [ ] `ECM_Physics::RecvEvt_DoFX` · 00495260 · legions 0048da30
- [ ] `ECM_Physics::RecvEvt_DoFXScript` · 00495360 · legions 0048db30
- [ ] `ECM_Physics::RecvEvt_DoMode` · 00495420 · legions 0048dbf0
- [ ] `ECM_Physics::RecvEvt_DoSlider` · 004954e0 · legions 0048dcb0
- [ ] `ECM_Physics::RecvEvt_DoStory` · 00495620 · legions 0048ddf0
- [ ] `ECM_Physics::RecvEvt_DoTargettedFX` · 00495710 · legions 0048dee0
- [ ] `ECM_Physics::RecvEvt_InitializeMissile` · 00495850 · legions 0048e020
- [ ] `ECM_Physics::RecvEvt_LeaveWorld` · 00495a00 · legions 0048e1d0
- [ ] `ECM_Physics::RecvEvt_LookAt` · 00495ac0 · legions 0048e290
- [ ] `ECM_Physics::RecvEvt_LookAtDir` · 00495b80 · legions 0048e350
- [ ] `ECM_Physics::RecvEvt_Parent` · 00495c80 · legions 0048e450
- [ ] `ECM_Physics::RecvEvt_Position` · 00495da0 · legions 0048e570
- [ ] `ECM_Physics::RecvEvt_PositionCell` · 00495ea0 · legions 0048e670
- [ ] `ECM_Physics::RecvEvt_SetAccelerationScale` · 00495fa0 · legions 0048e770
- [ ] `ECM_Physics::RecvEvt_SetGlobalModifier` · 00496060 · legions 0048e830
- [ ] `ECM_Physics::RecvEvt_SetHoverHeight` · 00496120 · legions 0048e8f0
- [ ] `ECM_Physics::RecvEvt_SetJumpScale` · 004961e0 · legions 0048e9b0
- [ ] `ECM_Physics::RecvEvt_SetMIDESC` · 004962a0 · legions 0048ea70
- [ ] `ECM_Physics::RecvEvt_SetMode` · 00496360 · legions 0048eb30
- [ ] `ECM_Physics::RecvEvt_SetVelocityScale` · 00496420 · legions 0048ebf0
- [ ] `ECM_Physics::RecvEvt_StopAllWeenieBehaviors` · 004964e0 · legions 0048ecb0
- [ ] `ECM_Physics::RecvEvt_StopBehavior` · 00496560 · legions 0048ed30
- [ ] `ECM_Physics::RecvEvt_StopFX` · 00496620 · legions 0048edf0
- [ ] `ECM_Physics::RecvEvt_StopLayer` · 004966e0 · legions 0048eeb0
- [ ] `ECM_Physics::RecvEvt_StopMoveToCancelsBehaviors` · 004967a0 · legions 0048ef70
- [ ] `ECM_Physics::RecvEvt_UpdateVisualDesc` · 00496820 · legions 0048eff0
- [ ] `ECM_Physics::SendEvt_CForceCreate` · 00496910 · legions 0048f0e0
- [ ] `ECM_Physics::SendEvt_CLookAt` · 00496980 · legions 0048f150
- [ ] `ECM_Physics::SendEvt_CLookAtDir` · 004969f0 · legions 0048f1c0
- [ ] `ECM_Physics::SendEvt_CPosition` · 00496a90 · legions 0048f260
- [ ] `ECM_Physics::RecvEvt_AddImpulse` · 00496b00 · legions 0048f2d0

### TargetPosition
- [ ] `TargetPosition::StreamPack` · 00496c10

### ECM_Physics
- [ ] `ECM_Physics::RecvEvt_DoFX_Private` · 00496cb0 · legions 0048f480
- [ ] `ECM_Physics::SendNotice_BeingDeleted` · 00496d60
- [ ] `ECM_Physics::SendNotice_CreateObject` · 00496de0
- [ ] `ECM_Physics::SendNotice_CreatePlayerObject` · 00496e70
- [ ] `ECM_Physics::SendNotice_E_LastAttacker` · 00496f00
- [ ] `ECM_Physics::SendNotice_LocalVisualDescUpdate` · 00496f90 · legions 0048f660
- [ ] `ECM_Physics::SendNotice_ModeUpdate` · 00497010
- [ ] `ECM_Physics::SendNotice_NextFrameDrawn` · 00497090
- [ ] `ECM_Physics::SendNotice_PlayerReadyToEnterWorld` · 00497110
- [ ] `ECM_Physics::SendNotice_PlayerTeleportStart` · 00497190
- [ ] `ECM_Physics::SendNotice_SetPlayerID` · 00497210
- [ ] `ECM_Physics::SendNotice_SetRegionID` · 004972a0

### PList<Position>
- [ ] `PList<Position>::Pack` · 004974c0 · legions 0048f9d0
- [ ] `PList<Position>::GetPackSize` · 00497510
- [ ] `PList<Position>::UnPack` · 00497550 · legions 0048fa74

### ListNode<Position>
- [ ] `ListNode<Position>::ListNode<Position>` · 004976a0 · legions 0048fbb0

### ECM_Physics
- [ ] `ECM_Physics::RecvEvt_MoveTo` · 00497750 · legions 0048fc60
- [ ] `ECM_Physics::RecvEvt_ReleaseBehavior` · 004978c0 · legions 0048fdd0
- [ ] `ECM_Physics::RecvEvt_DoBehavior` · 00497a00 · legions 0048ff10

## `portal/client/src/events/ecm_qualities.cpp.c` (0/20)

### ECM_Qualities
- [ ] `ECM_Qualities::RecvEvt_UpdateBool_Private` · 00497af0 · legions 00490000
- [ ] `ECM_Qualities::RecvEvt_UpdateBool_Visual` · 00497ba0 · legions 004900b0
- [ ] `ECM_Qualities::RecvEvt_UpdateDataID_Private` · 00497c90 · legions 004901a0
- [ ] `ECM_Qualities::RecvEvt_UpdateDataID_Visual` · 00497d40 · legions 00490250
- [ ] `ECM_Qualities::RecvEvt_UpdateFloat_Private` · 00497e30 · legions 00490340
- [ ] `ECM_Qualities::RecvEvt_UpdateFloat_Visual` · 00497ef0 · legions 00490400
- [ ] `ECM_Qualities::RecvEvt_UpdateInstanceID_Private` · 00497ff0 · legions 00490500
- [ ] `ECM_Qualities::RecvEvt_UpdateInstanceID_Visual` · 004980b0 · legions 004905c0
- [ ] `ECM_Qualities::RecvEvt_UpdateInt_Private` · 004981b0 · legions 004906c0
- [ ] `ECM_Qualities::RecvEvt_UpdateInt_Visual` · 00498260 · legions 00490770
- [ ] `ECM_Qualities::RecvEvt_UpdateLongInt_Private` · 00498350 · legions 00490860
- [ ] `ECM_Qualities::RecvEvt_UpdateLongInt_Visual` · 00498410 · legions 00490920
- [ ] `ECM_Qualities::RecvEvt_UpdatePosition_Private` · 00498510 · legions 00490a20
- [ ] `ECM_Qualities::RecvEvt_UpdatePosition_Visual` · 00498600 · legions 00490b10
- [ ] `ECM_Qualities::RecvEvt_UpdateStringInfo_Private` · 00498740 · legions 00490c50
- [ ] `ECM_Qualities::RecvEvt_UpdateStringInfo_Visual` · 00498800 · legions 00490d10
- [ ] `ECM_Qualities::RecvEvt_UpdateTimestamp_Private` · 00498910 · legions 00490e20
- [ ] `ECM_Qualities::RecvEvt_UpdateTimestamp_Visual` · 004989e0 · legions 00490ef0
- [ ] `ECM_Qualities::RecvEvt_UpdateString_Private` · 00498af0 · legions 00491000
- [ ] `ECM_Qualities::RecvEvt_UpdateString_Visual` · 00498c00 · legions 00491110

## `portal/client/src/events/globaleventhandler.cpp.c` (0/21)

### GlobalEventHandler
- [ ] `GlobalEventHandler::GetGlobalEventHandler` · 00492be0 · legions 0048b590
- [ ] `GlobalEventHandler::RecvEvt_AccountBooted` · 00492bf0 · legions 0048b5a0
- [ ] `GlobalEventHandler::RecvEvt_CharExitGame` · 00492c10 · legions 0048b5c0
- [ ] `GlobalEventHandler::RecvEvt_CharacterSet` · 00492c20 · legions 0048b5d0
- [ ] `GlobalEventHandler::RecvEvt_CharacterDeletion` · 00492c40 · legions 0048b5f0
- [ ] `GlobalEventHandler::RecvEvt_CharacterError` · 00492c60 · legions 0048b610
- [ ] `GlobalEventHandler::RecvEvt_CharGenVerification` · 00492c90 · legions 0048b640
- [ ] `GlobalEventHandler::RecvEvt_CreatePlayer` · 00492cb0 · legions 0048b660
- [ ] `GlobalEventHandler::RecvEvt_CreateObject` · 00492ce0 · legions 0048b690
- [ ] `GlobalEventHandler::RecvEvt_DestroyObject` · 00492d20 · legions 0048b6d0
- [ ] `GlobalEventHandler::RecvEvt_DisplayStringInfo` · 00492d40 · legions 0048b6f0
- [ ] `GlobalEventHandler::RecvEvt_WorldName` · 00492d90 · legions 0048b740

### LongNIHashIter<NIList<NoticeHandler_*>_>
- [ ] `LongNIHashIter<NIList<NoticeHandler_*>_>::LongNIHashIter<NIList<NoticeHandler_*>_>` · 00492db0 · legions 0048b7c0

### LongNIHash<NIList<NoticeHandler_*>_>
- [ ] `LongNIHash<NIList<NoticeHandler_*>_>::add` · 00492e10 · legions 0048b870

### GlobalEventHandler
- [ ] `GlobalEventHandler::GlobalEventHandler` · 00492e60 · legions 0048b8c0
- [ ] `GlobalEventHandler::GetNoticeHandlers` · 00492e80 · legions 0048b8e0
- [ ] `GlobalEventHandler::RegisterNoticeHandler` · 00492ec0 · legions 0048b920

### LongNIHash<NIList<NoticeHandler_*>_>
- [ ] `LongNIHash<NIList<NoticeHandler_*>_>::destroy_contents` · 00492fc0 · legions 0048ba90

### GlobalEventHandler
- [ ] `GlobalEventHandler::~GlobalEventHandler` · 00493060 · legions 0048bb30
- [ ] `GlobalEventHandler::UnRegisterNoticeHandler` · 00493090 · legions 0048bb60
- [ ] `GlobalEventHandler::UnRegisterAllNoticeHandlers` · 004930e0 · legions 0048bbb0

## `portal/client/src/events/noticehandler.cpp.c` (0/1)

### NoticeHandler
- [ ] `NoticeHandler::~NoticeHandler` · 00492bb0 · legions 0048b560

## `portal/client/src/weenie/interp/glue/cefundispatcher.cpp.c` (0/1)

### CEFunDispatcher
- [ ] `CEFunDispatcher::GetEFunAddress` · 0049fec0 · legions 00496f40 — ac2-script glue: id table + pure-glue registry; engine glue unbound

## `portal/client/src/weenie/interp/glue/cnativefactory.cpp.c` (0/268)

### PFileParser
- [ ] `PFileParser::PFileParser` · 004a5eb0 · legions 0049cf30

### CNativeFactory
- [ ] `CNativeFactory::GetCheckSums` · 004a5f00 · legions 0049cf80
- [ ] `CNativeFactory::ValidateCheckSums` · 004a5f20 · legions 0049cfa0

### BaseHashSetIter<unsigned_long,7>
- [ ] `BaseHashSetIter<unsigned_long,7>::UnPack` · 004a5f40 · legions 0049cfc0

### BaseMultiHashIter<unsigned_long,unsigned_long,4>
- [ ] `BaseMultiHashIter<unsigned_long,unsigned_long,4>::UnPack` · 004a5f80 · legions 0049d000

### BaseListIter<unsigned_long,11>
- [ ] `BaseListIter<unsigned_long,11>::UnPack` · 004a5fc0 · legions 0049d040

### RandomSelectionTable<unsigned_long>::ElementStruct
- [ ] `RandomSelectionTable<unsigned_long>::ElementStruct::ElementStruct` · 004a6090

### CollideProperty
- [ ] `CollideProperty::CollideProperty` · 004a60a0 · legions 0049d120

### ObjCollisionProfile
- [ ] `ObjCollisionProfile::ObjCollisionProfile` · 004a60e0 · legions 0049d160
- [ ] `ObjCollisionProfile::StreamPack` · 004a6110 · legions 0049d190
- [ ] `ObjCollisionProfile::ObjCollisionProfile` · 004a6170 · legions 0049d1f0

### DetectionProfile
- [ ] `DetectionProfile::DetectionProfile` · 004a61a0 · legions 0049d220

### LOSParameters
- [ ] `LOSParameters::LOSParameters` · 004a6220 · legions 0049d2a0 — ac2-physics los (Terrain::los_check)

### BaseMultiMapIter<unsigned_long,unsigned_long,13970>
- [ ] `BaseMultiMapIter<unsigned_long,unsigned_long,13970>::BaseMultiMapIter<unsigned_long,unsigned_long,13970>` · 004a62f0 · legions 0049d370

### BaseMultiMapIter<unsigned_long,unsigned___int64,13972>
- [ ] `BaseMultiMapIter<unsigned_long,unsigned___int64,13972>::BaseMultiMapIter<unsigned_long,unsigned___int64,13972>` · 004a6320 · legions 0049d3a0

### BaseListIter<unsigned_long,11>
- [ ] `BaseListIter<unsigned_long,11>::BaseListIter<unsigned_long,11>` · 004a6350 · legions 0049d3d0

### Sphere
- [ ] `Sphere::Sphere` · 004a6380 · legions 004917a0

### BaseMultiMapIter<unsigned___int64,unsigned_long,13974>
- [ ] `BaseMultiMapIter<unsigned___int64,unsigned_long,13974>::BaseMultiMapIter<unsigned___int64,unsigned_long,13974>` · 004a63a0 · legions 0049d400

### BaseMultiMapIter<unsigned___int64,unsigned___int64,13976>
- [ ] `BaseMultiMapIter<unsigned___int64,unsigned___int64,13976>::BaseMultiMapIter<unsigned___int64,unsigned___int64,13976>` · 004a63d0 · legions 0049d430

### BaseListIter<unsigned___int64,60>
- [ ] `BaseListIter<unsigned___int64,60>::BaseListIter<unsigned___int64,60>` · 004a6400 · legions 0049d460

### MissileParameters
- [ ] `MissileParameters::MissileParameters` · 004a6430 · legions 0049d490

### MissileTargetingParameters
- [ ] `MissileTargetingParameters::MissileTargetingParameters` · 004a6550 · legions 0049d5b0

### MotionValues
- [ ] `MotionValues::MotionValues` · 004a66a0 · legions 0049d700

### PathNodeData
- [ ] `PathNodeData::PathNodeData` · 004a6740 · legions 0049d7a0

### BaseListIter<unsigned_long,102>
- [ ] `BaseListIter<unsigned_long,102>::BaseListIter<unsigned_long,102>` · 004a6780 · legions 0049d7e0

### SetPositionStruct
- [ ] `SetPositionStruct::SetPositionStruct` · 004a67b0 · legions 0049d810

### CNativeFactory
- [ ] `CNativeFactory::GetNativePackSize` · 004a6870 · legions 0049d8b0

### CaseInsensitiveStringBase<PStringBase<unsigned_short>_>
- [ ] `CaseInsensitiveStringBase<PStringBase<unsigned_short>_>::case_insensitive_hash` · 004a6c30 · legions 0049dc70

### BaseHashSetIter<unsigned_long,7>
- [ ] `BaseHashSetIter<unsigned_long,7>::BaseHashSetIter<unsigned_long,7>` · 004a6e50 · legions 0049de90

### BaseListIter<unsigned_long,11>
- [ ] `BaseListIter<unsigned_long,11>::BaseListIter<unsigned_long,11>` · 004a6e70 · legions 0049deb0

### BaseHashSetIter<unsigned___int64,11788>
- [ ] `BaseHashSetIter<unsigned___int64,11788>::BaseHashSetIter<unsigned___int64,11788>` · 004a6e90 · legions 0049ded0

### BaseListIter<unsigned___int64,60>
- [ ] `BaseListIter<unsigned___int64,60>::BaseListIter<unsigned___int64,60>` · 004a6eb0 · legions 0049def0

### BaseListIter<unsigned_long,102>
- [ ] `BaseListIter<unsigned_long,102>::BaseListIter<unsigned_long,102>` · 004a6ed0 · legions 0049df10

### RandomSelectionTable<unsigned_long>
- [ ] `RandomSelectionTable<unsigned_long>::RandomSelectionTable<unsigned_long>` · 004a6ef0 · legions 0049df30

### BaseListIter<unsigned___int64,60>
- [ ] `BaseListIter<unsigned___int64,60>::Sync` · 004a7220 · legions 0049e260

### BaseListIter<unsigned_long,11>
- [ ] `BaseListIter<unsigned_long,11>::Sync` · 004a7280 · legions 0049e2c0

### IntrusiveHashIterator<unsigned___int64,HashSetData<unsigned___int64>_*>
- [ ] `IntrusiveHashIterator<unsigned___int64,HashSetData<unsigned___int64>_*>::ComputeOffset` · 004a72e0 · legions 004a1e50

### IntrusiveHashIterator<unsigned_long,HashSetData<unsigned_long>_*>
- [ ] `IntrusiveHashIterator<unsigned_long,HashSetData<unsigned_long>_*>::ComputeOffset` · 004a7340 · legions 004a1ad0

### CollisionProfile
- [ ] `CollisionProfile::CollisionProfile` · 004a7630 · legions 0049e610

### DetectionParameters
- [ ] `DetectionParameters::DetectionParameters` · 004a76c0 · legions 0049e6a0

### BaseHashIter<unsigned_long,unsigned_long,2>
- [ ] `BaseHashIter<unsigned_long,unsigned_long,2>::BaseHashIter<unsigned_long,unsigned_long,2>` · 004a77b0 · legions 0049e790

### BaseHashSetIter<unsigned_long,7>
- [ ] `BaseHashSetIter<unsigned_long,7>::BaseHashSetIter<unsigned_long,7>` · 004a77f0 · legions 0049e7d0

### BaseHashIter<unsigned_long,unsigned___int64,9>
- [ ] `BaseHashIter<unsigned_long,unsigned___int64,9>::BaseHashIter<unsigned_long,unsigned___int64,9>` · 004a7830 · legions 0049e810

### BaseHashIter<unsigned_long,unsigned_long,18>
- [ ] `BaseHashIter<unsigned_long,unsigned_long,18>::BaseHashIter<unsigned_long,unsigned_long,18>` · 004a7870 · legions 0049e850

### BaseHashIter<unsigned___int64,unsigned_long,51>
- [ ] `BaseHashIter<unsigned___int64,unsigned_long,51>::BaseHashIter<unsigned___int64,unsigned_long,51>` · 004a78d0 · legions 0049e8b0

### BaseHashSetIter<unsigned___int64,11788>
- [ ] `BaseHashSetIter<unsigned___int64,11788>::BaseHashSetIter<unsigned___int64,11788>` · 004a7910 · legions 0049e8f0

### BaseHashIter<unsigned___int64,unsigned___int64,58>
- [ ] `BaseHashIter<unsigned___int64,unsigned___int64,58>::BaseHashIter<unsigned___int64,unsigned___int64,58>` · 004a7950 · legions 0049e930

### BaseHashIter<unsigned___int64,unsigned_long,65>
- [ ] `BaseHashIter<unsigned___int64,unsigned_long,65>::BaseHashIter<unsigned___int64,unsigned_long,65>` · 004a7990 · legions 0049e970

### BaseHashIter<ReferenceIDWrapper,unsigned_long,76>
- [ ] `BaseHashIter<ReferenceIDWrapper,unsigned_long,76>::BaseHashIter<ReferenceIDWrapper,unsigned_long,76>` · 004a79d0 · legions 0049e9b0

### BaseHashIter<ReferenceIDWrapper,unsigned_long,78>
- [ ] `BaseHashIter<ReferenceIDWrapper,unsigned_long,78>::BaseHashIter<ReferenceIDWrapper,unsigned_long,78>` · 004a7a10 · legions 0049e9f0

### DBObj
- [ ] `DBObj::DBObj` · 004a7a90 · legions 0049ea70
- [ ] `DBObj::Pack` · 004a7b10
- [ ] `DBObj::UnPack` · 004a7b30

### CNativeFactory
- [ ] `CNativeFactory::HashNativeObject` · 004a7b50 · legions 0049eb30

### CaseInsensitiveStringBase<PStringBase<unsigned_short>_>
- [ ] `CaseInsensitiveStringBase<PStringBase<unsigned_short>_>::operator==` · 004a7bd0 · legions 0049ebb0

### BaseHashIter<unsigned_long,unsigned_long,2>
- [ ] `BaseHashIter<unsigned_long,unsigned_long,2>::BaseHashIter<unsigned_long,unsigned_long,2>` · 004a7c00 · legions 0049ebe0

### BaseHashIter<unsigned_long,unsigned___int64,9>
- [ ] `BaseHashIter<unsigned_long,unsigned___int64,9>::BaseHashIter<unsigned_long,unsigned___int64,9>` · 004a7c20 · legions 0049ec00

### BaseHashIter<unsigned_long,unsigned_long,18>
- [ ] `BaseHashIter<unsigned_long,unsigned_long,18>::BaseHashIter<unsigned_long,unsigned_long,18>` · 004a7c40 · legions 0049ec20

### BaseHashIter<unsigned___int64,unsigned_long,51>
- [ ] `BaseHashIter<unsigned___int64,unsigned_long,51>::BaseHashIter<unsigned___int64,unsigned_long,51>` · 004a7c60 · legions 0049ec40

### BaseHashIter<unsigned___int64,unsigned___int64,58>
- [ ] `BaseHashIter<unsigned___int64,unsigned___int64,58>::BaseHashIter<unsigned___int64,unsigned___int64,58>` · 004a7c80 · legions 0049ec60

### BaseHashIter<unsigned___int64,unsigned_long,65>
- [ ] `BaseHashIter<unsigned___int64,unsigned_long,65>::BaseHashIter<unsigned___int64,unsigned_long,65>` · 004a7ca0 · legions 0049ec80

### BaseHashIter<ReferenceIDWrapper,unsigned_long,76>
- [ ] `BaseHashIter<ReferenceIDWrapper,unsigned_long,76>::BaseHashIter<ReferenceIDWrapper,unsigned_long,76>` · 004a7cc0 · legions 0049eca0

### BaseHashIter<ReferenceIDWrapper,unsigned_long,78>
- [ ] `BaseHashIter<ReferenceIDWrapper,unsigned_long,78>::BaseHashIter<ReferenceIDWrapper,unsigned_long,78>` · 004a7ce0 · legions 0049ecc0

### PSmartArray<CollideProperty>
- [ ] `PSmartArray<CollideProperty>::StreamPack` · 004a7d90 · legions 0049edf0

### SmartArray<InstanceID>
- [ ] `SmartArray<InstanceID>::operator=` · 004a8000 · legions 0049f060

### SmartArray<BaseProperty>
- [ ] `SmartArray<BaseProperty>::operator=` · 004a8060 · legions 0049f0c0

### SmartArray<PropertyGroup>
- [ ] `SmartArray<PropertyGroup>::operator=` · 004a80d0 · legions 0049f130

### SmartArray<TimeOfDay>
- [ ] `SmartArray<TimeOfDay>::operator=` · 004a8140 · legions 0049f1a0

### SmartArray<WeekDay>
- [ ] `SmartArray<WeekDay>::operator=` · 004a81b0 · legions 0049f210

### SmartArray<Season>
- [ ] `SmartArray<Season>::operator=` · 004a8210 · legions 0049f270

### SmartArray<unsigned___int64>
- [ ] `SmartArray<unsigned___int64>::operator=` · 004a8280 · legions 0049f2e0

### BaseMultiMapIter<unsigned_long,unsigned_long,13970>
- [ ] `BaseMultiMapIter<unsigned_long,unsigned_long,13970>::Sync` · 004a8370 · legions 0049f3d0

### SmartArray<RandomSelectionTable<unsigned_long>::ElementStruct>
- [ ] `SmartArray<RandomSelectionTable<unsigned_long>::ElementStruct>::operator=` · 004a83f0 · legions 0049f450

### CNativeFactory
- [ ] `CNativeFactory::IsEqual` · 004a8450 · legions 0049f4b0
- [ ] `CNativeFactory::GetNativeTypeName` · 004a8500 · legions 0049f540

### HashIterator<InstanceID,CBookKeep_*>
- [ ] `HashIterator<InstanceID,CBookKeep_*>::next` · 004a90a0 · legions 0042f1e0

### PSmartArray<ObjCollisionProfile>
- [ ] `PSmartArray<ObjCollisionProfile>::StreamPack` · 004a9230 · legions 004a0230

### PSmartArray<unsigned___int64>
- [ ] `PSmartArray<unsigned___int64>::StreamPack` · 004a93d0

### BaseHashIter<unsigned_long,unsigned_long,2>
- [ ] `BaseHashIter<unsigned_long,unsigned_long,2>::GetReferences` · 004a9470 · legions 004a05a0

### BaseMultiHashIter<unsigned_long,unsigned_long,4>
- [ ] `BaseMultiHashIter<unsigned_long,unsigned_long,4>::GetReferences` · 004a94c0 · legions 004a0550

### BaseListIter<unsigned_long,11>
- [ ] `BaseListIter<unsigned_long,11>::GetReferences` · 004a9510 · legions 004a05f0

### BaseMultiMapIter<unsigned_long,unsigned_long,13970>
- [ ] `BaseMultiMapIter<unsigned_long,unsigned_long,13970>::GetReferences` · 004a9560 · legions 004a04c0

### BaseListIter<unsigned___int64,60>
- [ ] `BaseListIter<unsigned___int64,60>::Pack` · 004a95a0 · legions 004a0500

### BaseListIter<unsigned_long,11>
- [ ] `BaseListIter<unsigned_long,11>::Pack` · 004a95f0 · legions 004a0470

### RandomSelectionTable<unsigned_long>
- [ ] `RandomSelectionTable<unsigned_long>::StreamPack` · 004a9640 · legions 004a0640 — ac2-net protocol::native pack+unpack (Legions layouts; WState graphs round-trip byte-exact); ac2-script still decodes its own copy until it switches

### RandomSelectionTable<unsigned_long>::ElementStruct
- [ ] `RandomSelectionTable<unsigned_long>::ElementStruct::StreamPack` · 004a9760 · legions 004a0760 — ac2-net protocol::native pack+unpack (Legions layouts; WState graphs round-trip byte-exact); ac2-script still decodes its own copy until it switches

### BaseHashSetIter<unsigned_long,7>
- [ ] `BaseHashSetIter<unsigned_long,7>::Sync` · 004a97c0 · legions 004a07c0

### BaseHashSetIter<unsigned___int64,11788>
- [ ] `BaseHashSetIter<unsigned___int64,11788>::Sync` · 004a9870 · legions 004a0870

### LArray
- [ ] `LArray::StreamPack` · 004a9920

### RArray
- [ ] `RArray::GetReferences` · 004a9930 · legions 004a09a0

### PSmartArray<TimeOfDay>
- [ ] `PSmartArray<TimeOfDay>::StreamPack` · 004a99b0

### PSmartArray<WeekDay>
- [ ] `PSmartArray<WeekDay>::StreamPack` · 004a9a50

### PSmartArray<Season>
- [ ] `PSmartArray<Season>::StreamPack` · 004a9af0 · legions 004a0a20

### PSmartArray<PropertyGroup>
- [ ] `PSmartArray<PropertyGroup>::StreamPack` · 004a9b90 · legions 004a0b60 — unpack only, in ac2-net protocol::property / fx (types from a PropertyTypes resolver; Legions BaseProperty::StreamPack 0x0064B4E0 instruction-identical)

### PSmartArray<BaseProperty>
- [ ] `PSmartArray<BaseProperty>::StreamPack` · 004a9c30 · legions 004a0c00 — unpack only, in ac2-net protocol::property / fx (types from a PropertyTypes resolver; Legions BaseProperty::StreamPack 0x0064B4E0 instruction-identical)

### CNativeFactory
- [ ] `CNativeFactory::GetReferences` · 004a9cd0 · legions 004a0d40
- [ ] `CNativeFactory::ValidateGlueVersion` · 004a9f90 · legions 004a1000

### HashTable<unsigned_long,AppearanceInfo>
- [ ] `HashTable<unsigned_long,AppearanceInfo>::~HashTable<unsigned_long,AppearanceInfo>` · 004aa120 · legions 004a1190

### BaseArray<unsigned_long>
- [ ] `BaseArray<unsigned_long>::BaseArray<unsigned_long>` · 004aa160 · legions 004a11d0

### HashTable<unsigned_long,AppearanceInfo>
- [ ] `HashTable<unsigned_long,AppearanceInfo>::operator=` · 004aa2e0 · legions 004a1350

### HashSet<unsigned___int64>
- [ ] `HashSet<unsigned___int64>::operator=` · 004aa3d0 · legions 004a1440

### HashSet<unsigned_long>
- [ ] `HashSet<unsigned_long>::operator=` · 004aa5a0 · legions 004a1610

### HashTable<unsigned___int64,unsigned_long>
- [ ] `HashTable<unsigned___int64,unsigned_long>::operator=` · 004aa680 · legions 004a17e0

### HashTable<unsigned___int64,unsigned___int64>
- [ ] `HashTable<unsigned___int64,unsigned___int64>::operator=` · 004aa7a0 · legions 004a1900

### HashTable<ReferenceIDWrapper,unsigned_long>
- [ ] `HashTable<ReferenceIDWrapper,unsigned_long>::operator=` · 004aa8c0 · legions 004a1a20

### BaseMultiMapIter<unsigned_long,unsigned_long,13970>
- [ ] `BaseMultiMapIter<unsigned_long,unsigned_long,13970>::UpdateReferences` · 004aa9c0 · legions 004a1ff0

### BaseHashSetIter<unsigned_long,7>
- [ ] `BaseHashSetIter<unsigned_long,7>::Pack` · 004aa9e0 · legions 004a1bb0
- [ ] `BaseHashSetIter<unsigned_long,7>::UpdateReferences` · 004aaa20 · legions 004a1bf0

### BaseHashIter<unsigned_long,unsigned_long,2>
- [ ] `BaseHashIter<unsigned_long,unsigned_long,2>::StreamPack` · 004aaa50 · legions 004a1c20

### BaseMultiMapIter<unsigned_long,unsigned_long,13970>
- [ ] `BaseMultiMapIter<unsigned_long,unsigned_long,13970>::StreamPack` · 004aab00 · legions 004a1cd0

### BaseHashIter<unsigned_long,unsigned_long,2>
- [ ] `BaseHashIter<unsigned_long,unsigned_long,2>::UpdateReferences` · 004aab60
- [ ] `BaseHashIter<unsigned_long,unsigned_long,2>::Sync` · 004aab90 · legions 004a1de0

### BaseHashIter<unsigned___int64,unsigned_long,51>
- [ ] `BaseHashIter<unsigned___int64,unsigned_long,51>::StreamPack` · 004aac00

### BaseHashSetIter<unsigned___int64,11788>
- [ ] `BaseHashSetIter<unsigned___int64,11788>::Pack` · 004aac60 · legions 004a1ee0
- [ ] `BaseHashSetIter<unsigned___int64,11788>::UpdateReferences` · 004aaca0 · legions 004a1f20

### BaseMultiMapIter<unsigned_long,unsigned_long,13970>
- [ ] `BaseMultiMapIter<unsigned_long,unsigned_long,13970>::ComputeOffset` · 004aad70 · legions 004a1d30

### BaseListIter<unsigned___int64,60>
- [ ] `BaseListIter<unsigned___int64,60>::UpdateReferences` · 004aadf0 · legions 004a1db0

### BaseHashIter<unsigned___int64,unsigned_long,51>
- [ ] `BaseHashIter<unsigned___int64,unsigned_long,51>::UpdateReferences` · 004aae20 · legions 004a1eb0
- [ ] `BaseHashIter<unsigned___int64,unsigned_long,51>::Sync` · 004aae50 · legions 004a20a0

### BaseHashIter<ReferenceIDWrapper,unsigned_long,76>
- [ ] `BaseHashIter<ReferenceIDWrapper,unsigned_long,76>::StreamPack` · 004aaec0

### IntrusiveHashIterator<ReferenceIDWrapper,HashTableData<ReferenceIDWrapper,unsigned_long>_*>
- [ ] `IntrusiveHashIterator<ReferenceIDWrapper,HashTableData<ReferenceIDWrapper,unsigned_long>_*>::ComputeOffset` · 004aaf20 · legions 004a2110

### BaseHashIter<ReferenceIDWrapper,unsigned_long,76>
- [ ] `BaseHashIter<ReferenceIDWrapper,unsigned_long,76>::UpdateReferences` · 004aaf80 · legions 004a2170
- [ ] `BaseHashIter<ReferenceIDWrapper,unsigned_long,76>::Sync` · 004aafb0 · legions 004a2200

### BaseListIter<unsigned_long,11>
- [ ] `BaseListIter<unsigned_long,11>::UpdateReferences` · 004ab020

### RandomSelectionTable<unsigned_long>
- [ ] `RandomSelectionTable<unsigned_long>::RandomSelectionTable<unsigned_long>` · 004ab050 · legions 004a2270

### BaseMultiHashIter<unsigned___int64,unsigned_long,53>
- [ ] `BaseMultiHashIter<unsigned___int64,unsigned_long,53>::ComputeOffset` · 004ab0f0 · legions 004a2310

### BaseMultiHashIter<unsigned_long,unsigned_long,4>
- [ ] `BaseMultiHashIter<unsigned_long,unsigned_long,4>::ComputeOffset` · 004ab160

### BaseMultiHashIter<unsigned___int64,unsigned_long,53>
- [ ] `BaseMultiHashIter<unsigned___int64,unsigned_long,53>::Sync` · 004ab1d0 · legions 004a2380

### List<unsigned___int64>
- [ ] `List<unsigned___int64>::operator=` · 004ab240 · legions 004a23f0

### RArray
- [ ] `RArray::RArray` · 004ab2b0 · legions 004a2460
- [ ] `RArray::UpdateReferences` · 004ab2f0 · legions 004a24a0

### CollisionParameters
- [ ] `CollisionParameters::CollisionParameters` · 004ab350 · legions 004a2500

### GameTime
- [ ] `GameTime::GameTime` · 004ab460 · legions 004a2610
- [ ] `GameTime::Allocate` · 004ab620
- [ ] `GameTime::Allocator` · 004ab630
- [ ] `GameTime::GetDBOType` · 004ab650

### PropertyCollection
- [ ] `PropertyCollection::PropertyCollection` · 004ab680 · legions 004a2830

### PropertyGroup
- [ ] `PropertyGroup::PropertyGroup` · 004ab6e0 · legions 004a2890

### HashTable<unsigned_long,PStringBase<char>_>
- [ ] `HashTable<unsigned_long,PStringBase<char>_>::~HashTable<unsigned_long,PStringBase<char>_>` · 004ab750 · legions 004a2900
- [ ] `HashTable<unsigned_long,PStringBase<char>_>::operator=` · 004ab790 · legions 004a2940

### BaseHash<unsigned___int64,unsigned_long,51>
- [ ] `BaseHash<unsigned___int64,unsigned_long,51>::~BaseHash<unsigned___int64,unsigned_long,51>` · 004ab8b0 · legions 004a2a60

### PHashSet<unsigned_long>
- [ ] `PHashSet<unsigned_long>::~PHashSet<unsigned_long>` · 004ab900

### HashTable<PStringBase<unsigned_short>,Position>
- [ ] `HashTable<PStringBase<unsigned_short>,Position>::operator=` · 004abc40 · legions 004a2da0

### BaseMultiHashIter<unsigned_long,unsigned_long,4>
- [ ] `BaseMultiHashIter<unsigned_long,unsigned_long,4>::Sync` · 004ac070 · legions 004a3240

### CDBForm
- [ ] `CDBForm::operator=` · 004ac0e0 · legions 004a32b0
- [ ] `CDBForm::~CDBForm` · 004ac150 · legions 004a3320

### EntityLinkDesc
- [ ] `EntityLinkDesc::EntityLinkDesc` · 004ac1b0 · legions 004a3380

### LArray
- [ ] `LArray::LArray` · 004ac240 · legions 004a3410

### BaseHashSet<unsigned___int64,11788>
- [ ] `BaseHashSet<unsigned___int64,11788>::~BaseHashSet<unsigned___int64,11788>` · 004ac280 · legions 004a3450

### BaseHash<unsigned___int64,unsigned___int64,58>
- [ ] `BaseHash<unsigned___int64,unsigned___int64,58>::~BaseHash<unsigned___int64,unsigned___int64,58>` · 004ac2d0 · legions 004a34a0

### PList<IconModAtom>
- [ ] `PList<IconModAtom>::Pack` · 004ac780 · legions 0047e110

### PList<Ray>
- [ ] `PList<Ray>::UnPack` · 004ac7d0 · legions 004a3964

### BaseMultiHashIter<unsigned___int64,unsigned_long,53>
- [ ] `BaseMultiHashIter<unsigned___int64,unsigned_long,53>::Pack` · 004ac990 · legions 004a3b40
- [ ] `BaseMultiHashIter<unsigned___int64,unsigned_long,53>::UpdateReferences` · 004ac9d0 · legions 004a31d0

### BaseMultiHashIter<unsigned_long,unsigned_long,4>
- [ ] `BaseMultiHashIter<unsigned_long,unsigned_long,4>::Pack` · 004acaf0 · legions 004a3c70
- [ ] `BaseMultiHashIter<unsigned_long,unsigned_long,4>::UpdateReferences` · 004acb30 · legions 004a3a30

### Path
- [ ] `Path::Path` · 004ace10 · legions 004a3f60

### CNativeFactory
- [ ] `CNativeFactory::UpdateReferences` · 004acef0 · legions 004a4040
- [ ] `CNativeFactory::PackNativeObject` · 004ad1d0 · legions 004a4320 — ac2-net protocol::NativeObject pack+unpack: every native type the Legions WStates hold (all 39,748 graphs round-trip byte-exact) plus ShortcutInfo, UISaveLocations, SelectionInfo, Examination*, the adaptors; iterators and other engine types not ported

### PHashTable<unsigned_long,float>
- [ ] `PHashTable<unsigned_long,float>::PHashTable<unsigned_long,float>` · 004ad780 · legions 004a48d0

### PHashTable<unsigned_long,AppearanceInfo>
- [ ] `PHashTable<unsigned_long,AppearanceInfo>::PHashTable<unsigned_long,AppearanceInfo>` · 004ad7c0 · legions 004a4910
- [ ] `PHashTable<unsigned_long,AppearanceInfo>::PHashTable<unsigned_long,AppearanceInfo>` · 004ad840 · legions 004a4990
- [ ] `PHashTable<unsigned_long,AppearanceInfo>::StreamPack` · 004ad880

### PHashSet<unsigned___int64>
- [ ] `PHashSet<unsigned___int64>::StreamPack` · 004ada80 · legions 004a4bd0

### PHashTable<unsigned___int64,unsigned___int64>
- [ ] `PHashTable<unsigned___int64,unsigned___int64>::StreamPack` · 004adc10 · legions 004a4d60

### BaseHash<ReferenceIDWrapper,unsigned_long,78>
- [ ] `BaseHash<ReferenceIDWrapper,unsigned_long,78>::~BaseHash<ReferenceIDWrapper,unsigned_long,78>` · 004ade00 · legions 004a4f50

### PHashTable<ReferenceIDWrapper,unsigned_long>
- [ ] `PHashTable<ReferenceIDWrapper,unsigned_long>::StreamPack` · 004ade50

### ReferenceIDWrapper
- [ ] `ReferenceIDWrapper::ReferenceIDWrapper` · 004ae050 · legions 004a51a0

### PHashTable<PStringBase<unsigned_short>,Position>
- [ ] `PHashTable<PStringBase<unsigned_short>,Position>::~PHashTable<PStringBase<unsigned_short>,Position>` · 004ae230 · legions 004a5380
- [ ] `PHashTable<PStringBase<unsigned_short>,Position>::StreamPack` · 004ae280

### PHashSet<unsigned_long>
- [ ] `PHashSet<unsigned_long>::StreamPack` · 004ae520

### _STL::multimap<unsigned_long,unsigned_long,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>
- [ ] `_STL::multimap<unsigned_long,unsigned_long,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>::~multimap<unsigned_long,unsigned_long,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>` · 004ae710 · legions 004a5990

### _STL::multimap<unsigned_long,unsigned___int64,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned___int64>_>_>
- [ ] `_STL::multimap<unsigned_long,unsigned___int64,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned___int64>_>_>::~multimap<unsigned_long,unsigned___int64,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned___int64>_>_>` · 004ae760 · legions 004a59e0

### _STL::multimap<unsigned___int64,unsigned_long,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned_long>_>_>
- [ ] `_STL::multimap<unsigned___int64,unsigned_long,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned_long>_>_>::~multimap<unsigned___int64,unsigned_long,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned_long>_>_>` · 004ae7b0 · legions 004a5a30

### _STL::multimap<unsigned___int64,unsigned___int64,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned___int64>_>_>
- [ ] `_STL::multimap<unsigned___int64,unsigned___int64,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned___int64>_>_>::~multimap<unsigned___int64,unsigned___int64,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned___int64>_>_>` · 004ae800 · legions 004a5a80

### CDBForm
- [ ] `CDBForm::CDBForm` · 004af160 · legions 004a63e0
- [ ] `CDBForm::CDBForm` · 004af1b0 · legions 004a6430

### Path
- [ ] `Path::~Path` · 004af220 · legions 004a64a0

### PlacesTable
- [ ] `PlacesTable::PlacesTable` · 004af270 · legions 004a64f0
- [ ] `PlacesTable::GetDBOType` · 004af2b0

### RList
- [ ] `RList::RList` · 004af2c0 · legions 004a6540

### BaseHashSet<unsigned_long,7>
- [ ] `BaseHashSet<unsigned_long,7>::BaseHashSet<unsigned_long,7>` · 004af300 · legions 004a6580

### BehaviorParams
- [ ] `BehaviorParams::BehaviorParams` · 004af380 · legions 004a6600

### LList
- [ ] `LList::LList` · 004af4b0 · legions 004a6730

### Path
- [ ] `Path::Path` · 004af4f0 · legions 004a6770

### PlacesTable
- [ ] `PlacesTable::PlacesTable` · 004af600 · legions 004a6880

### BaseHash<unsigned_long,unsigned_long,2>
- [ ] `BaseHash<unsigned_long,unsigned_long,2>::BaseHash<unsigned_long,unsigned_long,2>` · 004af660 · legions 004a68e0
- [ ] `BaseHash<unsigned_long,unsigned_long,2>::BaseHash<unsigned_long,unsigned_long,2>` · 004af6d0 · legions 004a6950

### BaseMultiMap<unsigned_long,unsigned_long,13970>
- [ ] `BaseMultiMap<unsigned_long,unsigned_long,13970>::BaseMultiMap<unsigned_long,unsigned_long,13970>` · 004af710 · legions 004a6990
- [ ] `BaseMultiMap<unsigned_long,unsigned_long,13970>::GetReferences` · 004af760

### BaseHashSet<unsigned_long,7>
- [ ] `BaseHashSet<unsigned_long,7>::BaseHashSet<unsigned_long,7>` · 004af770 · legions 004a69f0

### BaseMultiMap<unsigned_long,unsigned___int64,13972>
- [ ] `BaseMultiMap<unsigned_long,unsigned___int64,13972>::BaseMultiMap<unsigned_long,unsigned___int64,13972>` · 004af7a0 · legions 004a6a20

### BaseMultiMap<unsigned___int64,unsigned_long,13974>
- [ ] `BaseMultiMap<unsigned___int64,unsigned_long,13974>::BaseMultiMap<unsigned___int64,unsigned_long,13974>` · 004af7f0 · legions 004a6a70

### BaseMultiMap<unsigned___int64,unsigned___int64,13976>
- [ ] `BaseMultiMap<unsigned___int64,unsigned___int64,13976>::BaseMultiMap<unsigned___int64,unsigned___int64,13976>` · 004af840 · legions 004a6ac0

### BaseMultiMap<unsigned_long,unsigned_long,13970>
- [ ] `BaseMultiMap<unsigned_long,unsigned_long,13970>::StreamPack` · 004af9c0

### (free functions)
- [ ] `STREAMPACK` · 004afaf0 · legions 00409e80

### BaseMultiMap<unsigned_long,unsigned___int64,13972>
- [ ] `BaseMultiMap<unsigned_long,unsigned___int64,13972>::StreamPack` · 004afc80

### BaseMultiMap<unsigned___int64,unsigned_long,13974>
- [ ] `BaseMultiMap<unsigned___int64,unsigned_long,13974>::StreamPack` · 004afec0

### BaseMultiMap<unsigned___int64,unsigned___int64,13976>
- [ ] `BaseMultiMap<unsigned___int64,unsigned___int64,13976>::StreamPack` · 004b0130

### ARHash
- [ ] `ARHash::ARHash` · 004b03b0 · legions 004a75a0

### LRHash
- [ ] `LRHash::LRHash` · 004b03f0 · legions 004a75e0
- [ ] `LRHash::GetReferences` · 004b0430

### BaseHash<unsigned___int64,unsigned_long,65>
- [ ] `BaseHash<unsigned___int64,unsigned_long,65>::AddDataAsReferences` · 004b0450 · legions 004a7a60

### LRHash
- [ ] `LRHash::UpdateReferences` · 004b04d0

### BaseHash<unsigned___int64,unsigned_long,65>
- [ ] `BaseHash<unsigned___int64,unsigned_long,65>::UpdateData` · 004b04f0

### LRHash
- [ ] `LRHash::LRHash` · 004b05b0 · legions 004a77a0

### NAHash
- [ ] `NAHash::NAHash` · 004b05f0 · legions 004a77e0
- [ ] `NAHash::GetReferences` · 004b0630
- [ ] `NAHash::UpdateReferences` · 004b0650
- [ ] `NAHash::NAHash` · 004b06a0 · legions 004a7930

### NRHash
- [ ] `NRHash::NRHash` · 004b06e0 · legions 004a7970
- [ ] `NRHash::GetReferences` · 004b0720

### BaseHash<ReferenceIDWrapper,unsigned_long,76>
- [ ] `BaseHash<ReferenceIDWrapper,unsigned_long,76>::AddKeysAsReferences` · 004b0740 · legions 004a79d0

### BaseHash<ReferenceIDWrapper,unsigned_long,78>
- [ ] `BaseHash<ReferenceIDWrapper,unsigned_long,78>::AddDataAsReferences` · 004b07d0

### NRHash
- [ ] `NRHash::UpdateReferences` · 004b0850

### BaseHash<ReferenceIDWrapper,unsigned_long,78>
- [ ] `BaseHash<ReferenceIDWrapper,unsigned_long,78>::UpdateData` · 004b0880

### (free functions)
- [ ] `UpdateReferenceIDKeys` · 004b0910

### NRHash
- [ ] `NRHash::NRHash` · 004b09e0 · legions 004a7bd0

### PlacesTable
- [ ] `PlacesTable::Allocate` · 004b0a20
- [ ] `PlacesTable::Allocator` · 004b0a30

### LAHashSet
- [ ] `LAHashSet::LAHashSet` · 004b0ab0 · legions 004a7ca0
- [ ] `LAHashSet::StreamPack` · 004b0af0

### LLHash
- [ ] `LLHash::LLHash` · 004b0b20 · legions 004a7d10
- [ ] `LLHash::StreamPack` · 004b0b60

### ALHash
- [ ] `ALHash::ALHash` · 004b0bf0 · legions 004a7de0

### LAHash
- [ ] `LAHash::LAHash` · 004b0c30 · legions 004a7e20

### LAHashSet
- [ ] `LAHashSet::LAHashSet` · 004b0c70 · legions 004a7e60

### LLHash
- [ ] `LLHash::LLHash` · 004b0cb0 · legions 004a7ea0

### QualifiedDataIDArray
- [ ] `QualifiedDataIDArray::QualifiedDataIDArray` · 004b0cf0 · legions 004a7ee0

### BaseMultiMap<unsigned_long,unsigned_long,13970>
- [ ] `BaseMultiMap<unsigned_long,unsigned_long,13970>::BaseMultiMap<unsigned_long,unsigned_long,13970>` · 004b0d20 · legions 004a7f10

### BaseMultiMap<unsigned_long,unsigned___int64,13972>
- [ ] `BaseMultiMap<unsigned_long,unsigned___int64,13972>::BaseMultiMap<unsigned_long,unsigned___int64,13972>` · 004b0d50 · legions 004a7f40

### BaseMultiMap<unsigned___int64,unsigned_long,13974>
- [ ] `BaseMultiMap<unsigned___int64,unsigned_long,13974>::BaseMultiMap<unsigned___int64,unsigned_long,13974>` · 004b0d80 · legions 004a7f70

### BaseMultiMap<unsigned___int64,unsigned___int64,13976>
- [ ] `BaseMultiMap<unsigned___int64,unsigned___int64,13976>::BaseMultiMap<unsigned___int64,unsigned___int64,13976>` · 004b0db0 · legions 004a7fa0

### BaseMultiHash<unsigned_long,unsigned___int64,13>
- [ ] `BaseMultiHash<unsigned_long,unsigned___int64,13>::Pack` · 004b0e80
- [ ] `BaseMultiHash<unsigned_long,unsigned___int64,13>::UnPack` · 004b0f40 · legions 004a8130

### BaseMultiHash<unsigned_long,unsigned_long,20>
- [ ] `BaseMultiHash<unsigned_long,unsigned_long,20>::UnPack` · 004b1270 · legions 004a8520

### BaseMultiHash<unsigned___int64,unsigned_long,53>
- [ ] `BaseMultiHash<unsigned___int64,unsigned_long,53>::UnPack` · 004b1530 · legions 004a87e0

### BaseMultiHash<unsigned___int64,unsigned___int64,62>
- [ ] `BaseMultiHash<unsigned___int64,unsigned___int64,62>::GetPackSize` · 004b1700
- [ ] `BaseMultiHash<unsigned___int64,unsigned___int64,62>::Pack` · 004b1770
- [ ] `BaseMultiHash<unsigned___int64,unsigned___int64,62>::UnPack` · 004b1830 · legions 004a8ae0

### BaseMultiHash<unsigned_long,unsigned___int64,13>
- [ ] `BaseMultiHash<unsigned_long,unsigned___int64,13>::GetPackSize` · 004b1af0 · legions 004a8da0

### BaseMultiHash<unsigned___int64,unsigned_long,53>
- [ ] `BaseMultiHash<unsigned___int64,unsigned_long,53>::Pack` · 004b1b60

### BaseMultiHash<unsigned_long,unsigned_long,4>
- [ ] `BaseMultiHash<unsigned_long,unsigned_long,4>::BaseMultiHash<unsigned_long,unsigned_long,4>` · 004b1c20 · legions 004a8ed0

### BaseMultiHash<unsigned_long,unsigned_long,20>
- [ ] `BaseMultiHash<unsigned_long,unsigned_long,20>::GetPackSize` · 004b1c70 · legions 004a8f20
- [ ] `BaseMultiHash<unsigned_long,unsigned_long,20>::Pack` · 004b1ce0 · legions 004a8460

### BaseMultiHash<unsigned_long,unsigned_long,4>
- [ ] `BaseMultiHash<unsigned_long,unsigned_long,4>::UnPack` · 004b1da0 · legions 004a8f90 — ac2-script reference/native unpack

### CNativeFactory
- [ ] `CNativeFactory::DeleteNativeObject` · 004b1e80 · legions 004a9070

### ALMultiHash
- [ ] `ALMultiHash::ALMultiHash` · 004b2300 · legions 004a94f0
- [ ] `ALMultiHash::UnPack` · 004b2350

### ARMultiHash
- [ ] `ARMultiHash::ARMultiHash` · 004b2360 · legions 004a9550
- [ ] `ARMultiHash::GetReferences` · 004b23b0
- [ ] `ARMultiHash::UpdateReferences` · 004b2480

### LAMultiHash
- [ ] `LAMultiHash::LAMultiHash` · 004b2530 · legions 004a96d0

### LLMultiHash
- [ ] `LLMultiHash::LLMultiHash` · 004b2580 · legions 004a9720
- [ ] `LLMultiHash::UnPack` · 004b25d0

### LRMultiHash
- [ ] `LRMultiHash::LRMultiHash` · 004b25e0 · legions 004a9780
- [ ] `LRMultiHash::GetReferences` · 004b2630
- [ ] `LRMultiHash::UpdateReferences` · 004b26b0

### LAMultiHash
- [ ] `LAMultiHash::UnPack` · 004b2760

### BaseMultiHash<unsigned_long,unsigned_long,4>
- [ ] `BaseMultiHash<unsigned_long,unsigned_long,4>::BaseMultiHash<unsigned_long,unsigned_long,4>` · 004b2770 · legions 004a9960

### ARMultiHash
- [ ] `ARMultiHash::ARMultiHash` · 004b27b0 · legions 004a99a0

### LRMultiHash
- [ ] `LRMultiHash::LRMultiHash` · 004b27f0 · legions 004a99e0

### ALMultiHash
- [ ] `ALMultiHash::ALMultiHash` · 004b2830 · legions 004a9a20

### LAMultiHash
- [ ] `LAMultiHash::LAMultiHash` · 004b2870 · legions 004a9a60

### LLMultiHash
- [ ] `LLMultiHash::LLMultiHash` · 004b28b0 · legions 004a9aa0

### CNativeFactory
- [ ] `CNativeFactory::CopyNativeObject` · 004b28f0 · legions 004a9ae0
- [ ] `CNativeFactory::NewNativeObject` · 004b3980 · legions 004aab70
- [ ] `CNativeFactory::UnPackNativeObject` · 004b4720 · legions 004ab910 — ac2-script native::unpack: the native types Legions WStates hold

## `portal/client/src/weenie/interp/glue/gcwallocator.cpp.c` (0/1)

### gCWAllocator
- [ ] `gCWAllocator::CallEFun_ResetShell` · 004bc8c0

## `portal/client/src/weenie/interp/glue/gcwbookkeeper.cpp.c` (0/2)

### gCWBookKeeper
- [ ] `gCWBookKeeper::CallEFun_AddObjectToBeDestroyed` · 004bc7e0
- [ ] `gCWBookKeeper::CallEFun_RemoveObjectToBeDestroyed` · 004bc850

## `portal/client/src/weenie/interp/glue/gcwdistributor.cpp.c` (0/3)

### gCWDistributor
- [ ] `gCWDistributor::CallEFun_GetEntityDID` · 004bc690
- [ ] `gCWDistributor::CallEFun_GetMyPackageID` · 004bc700
- [ ] `gCWDistributor::CallEFun_GetProcessID` · 004bc770

## `portal/client/src/weenie/interp/glue/gcwinterface.cpp.c` (0/32)

### gCWInterface
- [ ] `gCWInterface::Resume_Sleep` · 004bb120 · legions 004b0340
- [ ] `gCWInterface::CallEFun_DisplayHelpText` · 004bb200
- [ ] `gCWInterface::CallEFun_Sleep` · 004bb260
- [ ] `gCWInterface::CallEFun_AddShellCmd` · 004bb370
- [ ] `gCWInterface::CallEFun_AddShellCmdByStringID` · 004bb3f0
- [ ] `gCWInterface::CallEFun_AddShellHelp` · 004bb480
- [ ] `gCWInterface::CallEFun_AddShellHelpByStringID` · 004bb500
- [ ] `gCWInterface::CallEFun_GetExternalWeenieObject` · 004bb580 — ac2-script glue::options / examination (Legions layout and efun switch)
- [ ] `gCWInterface::CallEFun_GetInternalWeenieObject` · 004bb600 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWInterface::CallEFun_InqStringInfoValue` · 004bb680
- [ ] `gCWInterface::CallEFun_IsThePlayer` · 004bb7a0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWInterface::CallEFun_ProcessText` · 004bb810
- [ ] `gCWInterface::CallEFun_SendServerEvt` · 004bb890 — ac2-script host (ScriptHost: process creation, dispatch, SendServerEvt, notices)
- [ ] `gCWInterface::CallEFun_SendServerEvtEncrypt` · 004bb910 — ac2-script host (ScriptHost: process creation, dispatch, SendServerEvt, notices)
- [ ] `gCWInterface::CallEFun_GetMyPosition` · 004bb990
- [ ] `gCWInterface::CallIFun_ApplyProperties` · 004bba90 · legions 004b09b0
- [ ] `gCWInterface::CallIFun_DisplayStringInfo` · 004bbb80 · legions 004b0aa0
- [ ] `gCWInterface::CallIFun_DisplayText` · 004bbc80 · legions 004b0ba0
- [ ] `gCWInterface::CallEFun_GetOutputFilesPath` · 004bbd90 · legions 004b0cb0
- [ ] `gCWInterface::CallIFun_HandleCharacterSessionEnd` · 004bbe30 · legions 004b0d20
- [ ] `gCWInterface::CallIFun_HandleCharacterSessionStart` · 004bbec0 · legions 004b0db0
- [ ] `gCWInterface::CallIFun_HandleCollision` · 004bbf50 · legions 004b0e40
- [ ] `gCWInterface::CallIFun_HandleCreation` · 004bc030 · legions 004b0f20
- [ ] `gCWInterface::CallIFun_HandleDestruction` · 004bc0c0 · legions 004b0fb0
- [ ] `gCWInterface::CallIFun_HandleEnterWorld` · 004bc150 · legions 004b1040
- [ ] `gCWInterface::CallIFun_HandleLeaveWorld` · 004bc1e0 · legions 004b10d0
- [ ] `gCWInterface::CallIFun_HandlePostLeaveWorld` · 004bc270 · legions 004b1160
- [ ] `gCWInterface::CallIFun_HandleTeleportEnd` · 004bc300 · legions 004b11f0
- [ ] `gCWInterface::CallIFun_HandleTeleportStart` · 004bc3b0 · legions 004b12a0
- [ ] `gCWInterface::Resume_InqStringInfoValue` · 004bc440 · legions 004b1330
- [ ] `gCWInterface::CallIFun_OnStatChanged` · 004bc510 · legions 004b1400
- [ ] `gCWInterface::CallIFun_PreloadWSLSingletons` · 004bc600 · legions 004b14f0

## `portal/client/src/weenie/interp/glue/gcwphysicsobject.cpp.c` (0/85)

### gCWPhysicsObject
- [ ] `gCWPhysicsObject::CallEFun_DoStory` · 004b8050
- [ ] `gCWPhysicsObject::CallEFun_SetAccelerationScale` · 004b80b0
- [ ] `gCWPhysicsObject::CallEFun_SetJumpScale` · 004b8110
- [ ] `gCWPhysicsObject::CallEFun_SetVelocityScale` · 004b8170
- [ ] `gCWPhysicsObject::CallEFun_StopAllCameraFX` · 004b81d0
- [ ] `gCWPhysicsObject::CallEFun_AddImpulse` · 004b8270
- [ ] `gCWPhysicsObject::CallEFun_AddImpulseLocal` · 004b82f0
- [ ] `gCWPhysicsObject::CallEFun_AddTargetedImpulse` · 004b8370 · legions 004ae2a0
- [ ] `gCWPhysicsObject::CallEFun_ApplyAdditiveVisualDesc` · 004b8410
- [ ] `gCWPhysicsObject::CallEFun_ApplySubtractiveVisualDesc` · 004b8490
- [ ] `gCWPhysicsObject::CallEFun_ApplyVisualDescToPhysicsObj` · 004b8510
- [ ] `gCWPhysicsObject::CallEFun_CalculateDistance` · 004b8580
- [ ] `gCWPhysicsObject::CallEFun_CombineAndApplyVisualDesc` · 004b8600
- [ ] `gCWPhysicsObject::CallEFun_DoBehavior` · 004b8680
- [ ] `gCWPhysicsObject::CallEFun_DoBehaviorNonLatent` · 004b87a0
- [ ] `gCWPhysicsObject::CallEFun_DoFX` · 004b8820
- [ ] `gCWPhysicsObject::CallEFun_DoIntersection` · 004b88a0
- [ ] `gCWPhysicsObject::CallEFun_DoMode` · 004b89c0
- [ ] `gCWPhysicsObject::CallEFun_DoModeNonLatent` · 004b8ae0
- [ ] `gCWPhysicsObject::CallEFun_DoSlider` · 004b8b60
- [ ] `gCWPhysicsObject::CallEFun_DoSliderNonLatent` · 004b8c90 · legions 004ae810
- [ ] `gCWPhysicsObject::CallEFun_DoTargettedFX` · 004b8d20
- [ ] `gCWPhysicsObject::CallEFun_ForceVisualDesc` · 004b8db0
- [ ] `gCWPhysicsObject::CallEFun_GetAccelerationScale` · 004b8e30
- [ ] `gCWPhysicsObject::CallEFun_GetDesiredMode` · 004b8ea0
- [ ] `gCWPhysicsObject::CallEFun_GetElement` · 004b8f10
- [ ] `gCWPhysicsObject::CallEFun_GetHeight` · 004b8f80
- [ ] `gCWPhysicsObject::CallEFun_GetJumpScale` · 004b8ff0
- [ ] `gCWPhysicsObject::CallEFun_GetMode` · 004b9060
- [ ] `gCWPhysicsObject::CallEFun_GetRadius` · 004b90d0
- [ ] `gCWPhysicsObject::CallEFun_GetTopLevelHeight` · 004b9140
- [ ] `gCWPhysicsObject::CallEFun_GetTopLevelRadius` · 004b91b0
- [ ] `gCWPhysicsObject::CallEFun_GetVelocityScale` · 004b9220
- [ ] `gCWPhysicsObject::CallEFun_HasMotionInterpreter` · 004b9290
- [ ] `gCWPhysicsObject::CallEFun_HasRemoteDoMotionValues` · 004b9300
- [ ] `gCWPhysicsObject::CallEFun_InitVisualDesc` · 004b9370
- [ ] `gCWPhysicsObject::CallEFun_IsAnimationFrozen` · 004b93e0
- [ ] `gCWPhysicsObject::CallEFun_IsDoingBehavior` · 004b9450
- [ ] `gCWPhysicsObject::CallEFun_IsInWorld` · 004b94d0
- [ ] `gCWPhysicsObject::CallEFun_IsMovementFrozen` · 004b9540
- [ ] `gCWPhysicsObject::CallEFun_IsOutside` · 004b95b0
- [ ] `gCWPhysicsObject::CallEFun_IsUndetectable` · 004b9620
- [ ] `gCWPhysicsObject::CallEFun_LocalToGlobal` · 004b9690
- [ ] `gCWPhysicsObject::CallEFun_LookAt` · 004b9720
- [ ] `gCWPhysicsObject::CallEFun_MissileFireOnly` · 004b97a0
- [ ] `gCWPhysicsObject::CallEFun_MissileInitPlaceAndFire` · 004b9810
- [ ] `gCWPhysicsObject::CallEFun_MissileInitialize` · 004b98a0
- [ ] `gCWPhysicsObject::CallEFun_MissileInquireDirection` · 004b9930
- [ ] `gCWPhysicsObject::CallEFun_MissilePlace` · 004b99c0
- [ ] `gCWPhysicsObject::CallEFun_MissilePlaceAndFire` · 004b9a30
- [ ] `gCWPhysicsObject::CallEFun_MissileRetarget` · 004b9aa0
- [ ] `gCWPhysicsObject::CallEFun_ReleaseBehavior` · 004b9b30
- [ ] `gCWPhysicsObject::CallEFun_SetAnimationFrozen` · 004b9c60
- [ ] `gCWPhysicsObject::CallEFun_SetCloaked` · 004b9ce0
- [ ] `gCWPhysicsObject::CallEFun_SetCollisionState` · 004b9d60
- [ ] `gCWPhysicsObject::CallEFun_SetLocalVisualDesc` · 004b9de0
- [ ] `gCWPhysicsObject::CallEFun_SetMode` · 004b9e60
- [ ] `gCWPhysicsObject::CallEFun_SetMovementFrozen` · 004b9ee0
- [ ] `gCWPhysicsObject::CallEFun_SetNoDrawPhysics` · 004b9f60
- [ ] `gCWPhysicsObject::CallEFun_SetUndetectable` · 004b9fe0
- [ ] `gCWPhysicsObject::CallEFun_StopAllWeenieBehaviors` · 004ba060
- [ ] `gCWPhysicsObject::CallEFun_StopBehavior` · 004ba0d0
- [ ] `gCWPhysicsObject::CallEFun_StopFX` · 004ba150
- [ ] `gCWPhysicsObject::CallEFun_StopLayer` · 004ba1d0
- [ ] `gCWPhysicsObject::CallEFun_StopMoveToCancelsBehaviors` · 004ba250
- [ ] `gCWPhysicsObject::CallEFun_TogglePlaceMarkerObjects` · 004ba2c0
- [ ] `gCWPhysicsObject::CallEFun__SetIgnoreCollisions` · 004ba330
- [ ] `gCWPhysicsObject::CallEFun__SetReportCollisions` · 004ba3b0
- [ ] `gCWPhysicsObject::CallEFun_GetDesiredMotionValues` · 004ba530
- [ ] `gCWPhysicsObject::CallEFun_GetLastDoMotionValues` · 004ba5e0
- [ ] `gCWPhysicsObject::CallEFun_GetLocalVisualDesc` · 004ba690
- [ ] `gCWPhysicsObject::CallEFun_GetRemoteDoMotionValues` · 004ba750
- [ ] `gCWPhysicsObject::CallEFun_GetStandardMotionValues` · 004ba800
- [ ] `gCWPhysicsObject::DetectionTableToLRHash` · 004ba8b0 · legions 004afbe0
- [ ] `gCWPhysicsObject::Resume_DoBehavior` · 004baa50 · legions 004afd80
- [ ] `gCWPhysicsObject::Resume_DoIntersection` · 004baae0 · legions 004afe10
- [ ] `gCWPhysicsObject::Resume_DoMode` · 004babb0 · legions 004afee0
- [ ] `gCWPhysicsObject::Resume_DoSlider` · 004bac40 · legions 004aff70
- [ ] `gCWPhysicsObject::CallEFun_GetCollisionState` · 004bacd0
- [ ] `gCWPhysicsObject::CallEFun_InqCurrentProperties` · 004bad70
- [ ] `gCWPhysicsObject::CallEFun_InqCurrentProperty` · 004bae10
- [ ] `gCWPhysicsObject::CallEFun_InqCylSphere` · 004baeb0 · legions 004b0160
- [ ] `gCWPhysicsObject::CallEFun_MissileInquireCurrentVelocity` · 004baf50
- [ ] `gCWPhysicsObject::CallEFun_PerformLOS` · 004baff0
- [ ] `gCWPhysicsObject::Resume_ReleaseBehavior` · 004bb090 · legions 004b02b0

## `portal/client/src/weenie/interp/glue/gcwrepository.cpp.c` (0/90)

### gCWRepository
- [ ] `gCWRepository::CallEFun_AdjAndTruncateFloat` · 004b4ba0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_AdjAndTruncateInt` · 004b4c50 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_AdjAndTruncateLongInt` · 004b4cf0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_AdjFloat` · 004b4dd0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_AdjInt` · 004b4e60 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_AdjLongInt` · 004b4ee0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_DoesBoolExist` · 004b4f90 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_DoesDataIDExist` · 004b5010 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_DoesFloatExist` · 004b5090 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_DoesInstanceIDExist` · 004b5110 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_DoesIntExist` · 004b5190 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_DoesLongIntExist` · 004b5210 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_DoesPackageIDExist` · 004b5290 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_DoesPositionExist` · 004b5310 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_DoesStringExist` · 004b5390 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_DoesStringInfoExist` · 004b5410 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_DoesTimestampExist` · 004b5490 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_EdgeSlides` · 004b5510
- [ ] `gCWRepository::CallEFun_GetBool` · 004b5580 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_GetDataID` · 004b5600 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_GetFloat` · 004b5680 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_GetID` · 004b5700 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_GetInstanceID` · 004b5770 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_GetInt` · 004b5810 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_GetLongInt` · 004b5890 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_GetPackageID` · 004b5920 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_GetRawBool` · 004b59a0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_GetRawDataID` · 004b5a20 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_GetRawFloat` · 004b5aa0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_GetRawInstanceID` · 004b5b20 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_GetRawInt` · 004b5bc0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_GetRawLongInt` · 004b5c40 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_GetRawPackageID` · 004b5cd0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_GetScale` · 004b5d50
- [ ] `gCWRepository::CallEFun_GetTimestamp` · 004b5dc0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_LoadDataIDW` · 004b5e60
- [ ] `gCWRepository::CallEFun_RemoveBool` · 004b5ee0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_RemoveDataID` · 004b5f60 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_RemoveFloat` · 004b5fe0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_RemoveInstanceID` · 004b6060 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_RemoveInt` · 004b60e0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_RemoveLongInt` · 004b6160 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_RemovePackageID` · 004b61e0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_RemovePosition` · 004b6260 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_RemoveString` · 004b62e0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_RemoveStringInfo` · 004b6360 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_RemoveTimestamp` · 004b63e0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_SetBool` · 004b6460 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_SetDataID` · 004b64e0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_SetEdgeSlides` · 004b6560
- [ ] `gCWRepository::CallEFun_SetEtherealType` · 004b65e0
- [ ] `gCWRepository::CallEFun_SetFloat` · 004b6670 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_SetInstanceID` · 004b66f0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_SetInt` · 004b6770 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_SetLongInt` · 004b67f0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_SetPackageID` · 004b6870 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_SetPosition` · 004b68f0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_SetScale` · 004b6970
- [ ] `gCWRepository::CallEFun_SetStringInfo` · 004b69f0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_SetStringW` · 004b6a70 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_SetTimestamp` · 004b6af0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_DumpBoolsToString` · 004b6b90
- [ ] `gCWRepository::CallEFun_DumpDIDsToString` · 004b6c50
- [ ] `gCWRepository::CallEFun_DumpFloatsToString` · 004b6d10
- [ ] `gCWRepository::CallEFun_DumpIIDsToString` · 004b6dd0
- [ ] `gCWRepository::CallEFun_DumpIntsToString` · 004b6e90
- [ ] `gCWRepository::CallEFun_DumpLongIntsToString` · 004b6f50
- [ ] `gCWRepository::CallEFun_DumpPositionsToString` · 004b7010
- [ ] `gCWRepository::CallEFun_DumpStringInfosToString` · 004b70d0
- [ ] `gCWRepository::CallEFun_DumpStringsToString` · 004b7190
- [ ] `gCWRepository::CallEFun_DumpTSsToString` · 004b7250
- [ ] `gCWRepository::CallEFun_DumpWeenieDescW` · 004b7310
- [ ] `gCWRepository::CallEFun_GetPosition` · 004b73d0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_GetRawPosition` · 004b7480 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_GetRawStringInfo` · 004b7530 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_GetRawStringW` · 004b75f0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_GetStringInfo` · 004b76c0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_GetStringW` · 004b7780 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_InqBool` · 004b7850 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_InqDataID` · 004b7900 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_InqEtherealType` · 004b79b0 · legions 004adb10
- [ ] `gCWRepository::CallEFun_InqFloat` · 004b7a50 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_InqInstanceID` · 004b7b00 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_InqInt` · 004b7bb0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_InqLongInt` · 004b7c60 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_InqPackageID` · 004b7d10 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_InqPosition` · 004b7dc0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_InqStringInfo` · 004b7e60 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_InqStringW` · 004b7f00 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)
- [ ] `gCWRepository::CallEFun_InqTimestamp` · 004b7fa0 — ac2-script host glue (notices, chat requests, gCWRepository, gCWInterface)

## `portal/shared/src/weenie/allocator/wallocator.cpp.c` (0/38)

### DBFile2IDTable
- [ ] `DBFile2IDTable::Get` · 0052cad0 · legions 00511050

### CWAllocator
- [ ] `CWAllocator::CWAllocator` · 0052caf0 · legions 00511070
- [ ] `CWAllocator::Clear` · 0052cb60 · legions 005110e0
- [ ] `CWAllocator::AllocateWeenieObject` · 0052cb80 · legions 00511100
- [ ] `CWAllocator::AllocateWeenieDesc` · 0052cbd0 · legions 00511150
- [ ] `CWAllocator::AllocateWeenieDesc` · 0052cbf0 · legions 00511170
- [ ] `CWAllocator::IsMobileByQualitiesID_` · 0052cc10
- [ ] `CWAllocator::ResetShell` · 0052cc70 · legions 00511190
- [ ] `CWAllocator::ResetShellInternal` · 0052cc80 · legions 005111a0
- [ ] `CWAllocator::~CWAllocator` · 0052ccf0 · legions 00511210
- [ ] `CWAllocator::IsMobileByPackageID` · 0052cd90
- [ ] `CWAllocator::IsMobileByDataID` · 0052cdc0

### DBObjGrabber<DBFile2IDTable>
- [ ] `DBObjGrabber<DBFile2IDTable>::DBObjGrabber<DBFile2IDTable>` · 0052ce10 · legions 00511380

### CWAllocator
- [ ] `CWAllocator::SetNoDBWLibFile` · 0052ce70 · legions 005113e0
- [ ] `CWAllocator::SetDBWLibFile` · 0052cec0 · legions 00511430
- [ ] `CWAllocator::GetDBWLib` · 0052cf10 · legions 00511480
- [ ] `CWAllocator::InitWLibPath` · 0052cfb0 · legions 00511520

### Resolver
- [ ] `Resolver::Clear` · 0052d0f0 · legions 00511660
- [ ] `Resolver::Clear_Internal` · 0052d100 · legions 00511670

### CWAllocator
- [ ] `CWAllocator::AllocateResolver` · 0052d160 · legions 005116d0
- [ ] `CWAllocator::Init_Internal` · 0052d1e0 · legions 00511750

### Resolver
- [ ] `Resolver::Init` · 0052d270 · legions 005117c0
- [ ] `Resolver::LoadByteStream` · 0052d340 · legions 00511890

### CentralDebugInfo
- [ ] `CentralDebugInfo::GetInstance` · 0052d3c0

### CentralDebugByteStream
- [-] `CentralDebugByteStream::LoadDebug_AddLineOffsetInfo` · 0052d430 — text source loader, admin build only
- [-] `CentralDebugByteStream::LoadDebug_AddFunctionLocation` · 0052d4e0 — text source loader, admin build only
- [-] `CentralDebugByteStream::LoadDebug_AddOriginalSourceFileInfo` · 0052d5a0 — text source loader, admin build only
- [-] `CentralDebugByteStream::LoadDebug_AddFrameDebugInfo` · 0052d6f0 — text source loader, admin build only

### FrameDebugInfo
- [ ] `FrameDebugInfo::FrameDebugInfo` · 0052d800 · legions 00511c50

### SmartArray<FrameMemberDebugInfo>
- [ ] `SmartArray<FrameMemberDebugInfo>::operator=` · 0052d890 · legions 00511d30

### FrameMemberDebugInfo
- [ ] `FrameMemberDebugInfo::operator=` · 0052d900 · legions 00511da0
- [ ] `FrameMemberDebugInfo::FrameMemberDebugInfo` · 0052da70 · legions 00511f10

### PSmartArray<FrameMemberDebugInfo>
- [ ] `PSmartArray<FrameMemberDebugInfo>::StreamPack` · 0052dab0 · legions 00511f50

### FrameDebugInfo
- [ ] `FrameDebugInfo::StreamPack` · 0052db50 · legions 00511ff0

### Resolver
- [ ] `Resolver::ProcessByteStream` · 0052dbb0 · legions 00512050 — ac2-script vm: no GC/reference reuse, no VMMon timers, efun ids resolved per call
- [ ] `Resolver::BuildReverseMap` · 0052dc80 · legions 00512120

### IntrusiveHashIterator<PStringBase<char>,HashTableData<PStringBase<char>,unsigned_long>_*>
- [ ] `IntrusiveHashIterator<PStringBase<char>,HashTableData<PStringBase<char>,unsigned_long>_*>::operator++` · 0052dd40 · legions 005121e0

### CWAllocator
- [ ] `CWAllocator::Init` · 0052dd70 · legions 00512210

## `portal/shared/src/weenie/distributor/wdistributor.cpp.c` (0/13)

### CWDistributor
- [ ] `CWDistributor::Construct` · 0052de20 · legions 005122c0 — ac2-script host (ScriptHost: process creation, dispatch, SendServerEvt, notices)
- [ ] `CWDistributor::~CWDistributor` · 0052dee0 · legions 00512380
- [ ] `CWDistributor::GetContextState` · 0052df00 · legions 005123a0
- [ ] `CWDistributor::HandleCrash` · 0052df20
- [ ] `CWDistributor::GetInstanceReference` · 0052df80 · legions 005123c0 — ac2-script host (ScriptHost: process creation, dispatch, SendServerEvt, notices)
- [ ] `CWDistributor::GetMyPackageID` · 0052df90 · legions 005123d0
- [ ] `CWDistributor::GetProcessID` · 0052dfa0 · legions 005123e0

### VMDataTimer
- [ ] `VMDataTimer::~VMDataTimer` · 0052dfb0

### CWDistributor
- [ ] `CWDistributor::CWDistributor` · 0052e010 · legions 005123f0
- [ ] `CWDistributor::ResumeContext` · 0052e080 · legions 00512460 — ac2-script host (ScriptHost: process creation, dispatch, SendServerEvt, notices)
- [ ] `CWDistributor::ResumeContext` · 0052e130 — ac2-script host (ScriptHost: process creation, dispatch, SendServerEvt, notices)
- [ ] `CWDistributor::DispatchInterpMsg` · 0052e150 · legions 005124d0 — ac2-script host (ScriptHost: process creation, dispatch, SendServerEvt, notices)
- [ ] `CWDistributor::RecvEvt_InterpCEvent_Private` · 0052e250 · legions 00512590

## `portal/shared/src/weenie/interface/winterface.cpp.c` (1/52)

### CWInterface
- [ ] `CWInterface::CWInterface` · 0052e280 · legions 005125c0
- [ ] `CWInterface::~CWInterface` · 0052e2b0 · legions 005125f0
- [ ] `CWInterface::HandleCreation` · 0052e2e0
- [ ] `CWInterface::HandleDestruction` · 0052e3c0 · legions 00512700
- [ ] `CWInterface::ObjectBeingDestructed` · 0052e3f0 · legions 00512730
- [ ] `CWInterface::HandleEnterWorld` · 0052e410 · legions 00512750
- [ ] `CWInterface::HandleLeaveWorld` · 0052e440 · legions 00512780
- [ ] `CWInterface::HandlePostLeaveWorld` · 0052e470 · legions 005127b0
- [ ] `CWInterface::HandleTeleportStart` · 0052e490 · legions 005127d0
- [ ] `CWInterface::HandleTeleportEnd` · 0052e4a0 · legions 005127e0
- [ ] `CWInterface::HandleCharacterSessionStart` · 0052e4c0 · legions 00512800
- [ ] `CWInterface::HandleCharacterSessionEnd` · 0052e520 · legions 00512860
- [ ] `CWInterface::ApplyProperties` · 0052e530 · legions 00512870
- [ ] `CWInterface::SendServerEvt` · 0052e570 · legions 005128b0
- [ ] `CWInterface::SendServerEvtEncrypt` · 0052e590 · legions 005128d0
- [ ] `CWInterface::GetInternalWeenieObject` · 0052e5b0 · legions 005128f0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWInterface::IsPlayer` · 0052e5e0 · legions 00512920
- [ ] `CWInterface::IsCreature` · 0052e600 · legions 00512940
- [ ] `CWInterface::IsThePlayer` · 0052e620 · legions 00512960 — ac2-script host::interface
- [x] `CWInterface::OnStatChanged` · 0052e660 · legions 005129a0 — ac2-client stat_changed: OnStatChanged (IFun 0x7F0044: table, quality) on the object's process once the VM has it (CPlayer 0x85: vitals, XP bar, vitae, skill panel); the session's stand-ins otherwise
- [ ] `CWInterface::RecvEvt_PlayerDesc` · 0052e6a0 · legions 005129e0
- [ ] `CWInterface::HandleIntersectionDone` · 0052e6c0 · legions 00512a00
- [ ] `CWInterface::InqDetectionProfile` · 0052e6e0 · legions 00512a20
- [ ] `CWInterface::HandleCollision` · 0052e700 · legions 00512a40
- [ ] `CWInterface::HandleBehaviorDone` · 0052e730 · legions 00512a70
- [ ] `CWInterface::HandleReleaseDone` · 0052e750 · legions 00512a90
- [ ] `CWInterface::HandleSliderDone` · 0052e770 · legions 00512ab0
- [ ] `CWInterface::HandleModeDone` · 0052e790 · legions 00512ad0
- [ ] `CWInterface::InqPrevHistoryItem` · 0052e7b0 · legions 00512af0 — ac2-weenie OnlineSession::chat_history(ChatHistory::Command, prev)
- [ ] `CWInterface::InqNextHistoryItem` · 0052e7c0 · legions 00512b00 — ac2-weenie OnlineSession::chat_history(ChatHistory::Command, prev)
- [ ] `CWInterface::DisplayText` · 0052e7d0 · legions 00512b10
- [ ] `CWInterface::DisplayStringInfo` · 0052e7f0 · legions 00512b30
- [ ] `CWInterface::PreloadWSLSingletons` · 0052e810 · legions 00512b50
- [ ] `CWInterface::CheckDetectionTypeMatch` · 0052e870 · legions 00512bb0
- [ ] `CWInterface::InqStringInfoValue` · 0052e8c0 · legions 00512c00

### `anonymous-namespace'::ClientLatentEfunActivator
- [ ] ``anonymous-namespace'::ClientLatentEfunActivator::OnTrigger` · 0052e960

### CWInterface
- [ ] `CWInterface::ExpandText` · 0052e970 · legions 00512cb0
- [ ] `CWInterface::DisplayHelpText` · 0052e9c0 · legions 00512d00

### `anonymous-namespace'::SleepAwakener
- [ ] ``anonymous-namespace'::SleepAwakener::OnTrigger` · 0052eaf0

### `anonymous-namespace'::InqStringInfoValue_AsyncCacheCallback
- [ ] ``anonymous-namespace'::InqStringInfoValue_AsyncCacheCallback::OnContextFinished` · 0052eb10

### CWInterface
- [ ] `CWInterface::GetOutputFilesPath` · 0052eb80 · legions 00512ec0

### HashTable<PStringBase<unsigned_short>,FunctionID>
- [ ] `HashTable<PStringBase<unsigned_short>,FunctionID>::~HashTable<PStringBase<unsigned_short>,FunctionID>` · 0052ed30 · legions 00512f40

### CWInterface
- [ ] `CWInterface::ProcessText` · 0052ee50 · legions 00513060 — ac2-weenie ChatState::process_text: command history, aliases, the shell, ProcessText_Done; CPlayer 0xDB/0x131/0x112/0xC5/0x5E/0xCF/0x94/0x103/0xE4/0x14C natively, other commands as SessionEvent::ShellCommand
- [ ] `CWInterface::HandleCmdArgs` · 0052ef20 · legions 00513130
- [ ] `CWInterface::ResumeCallback` · 0052f070 · legions 00513280
- [ ] `CWInterface::Sleep` · 0052f100
- [ ] `CWInterface::DoCmdInterp` · 0052f1f0
- [ ] `CWInterface::DoHelpInterp` · 0052f300
- [ ] `CWInterface::AddShellCmd` · 0052f430 · legions 005135f0 — ac2-weenie ChatCommands::register (split ;:, lowercased, first registration wins); ac2-client chat_commands resolves CPlayer 0x121's table
- [ ] `CWInterface::AddShellHelp` · 0052f6e0 · legions 005138a0
- [ ] `CWInterface::AddShellCmdByStringID` · 0052f990 · legions 00513b50 — ac2-weenie ChatCommands::register (split ;:, lowercased, first registration wins); ac2-client chat_commands resolves CPlayer 0x121's table
- [ ] `CWInterface::AddShellHelpByStringID` · 0052fa10 · legions 00513bd0

## `portal/shared/src/weenie/interp/dbresolver.cpp.c` (0/4)

### DBResolver
- [ ] `DBResolver::Clear` · 006b7ef0 · legions 0064f710

### Resolver
- [ ] `Resolver::~Resolver` · 006b7f20 · legions 0064f740

### DBResolver
- [ ] `DBResolver::DBResolver` · 006b7fc0 · legions 0064f7e0
- [ ] `DBResolver::Init` · 006b8010 · legions 0064f830

## `portal/shared/src/weenie/interp/winterpasserts.cpp.c` (0/1)

### WInterpAsserts
- [ ] `WInterpAsserts::GetContextObject` · 006b8070

## `portal/shared/src/weenie/interp/wlib.cpp.c` (0/8)

### WLib
- [ ] `WLib::WLib` · 006b7790 · legions 0064efb0
- [ ] `WLib::GetDBOType` · 006b77c0
- [ ] `WLib::GetPackSize` · 006b77d0
- [ ] `WLib::Pack` · 006b7800 · legions 0064f020
- [ ] `WLib::Allocate` · 006b7840
- [ ] `WLib::ReleaseRefs` · 006b7850 · legions 0064f070
- [ ] `WLib::Clear` · 006b78a0 · legions 0064f0c0
- [ ] `WLib::UnPack` · 006b78f0 — ac2-script WLib/WState/ByteStream decode

## `portal/shared/src/weenie/interp/wstate.cpp.c` (0/16)

### WState
- [ ] `WState::Destroy` · 006b79e0
- [ ] `WState::GetStateByType` · 006b79f0 · legions 0064f210 — ac2-script wstate DID formulas / ByteStream::package_id
- [ ] `WState::GetPackSize` · 006b7a30 · legions 0064f250
- [ ] `WState::Pack` · 006b7a50 · legions 0064f270
- [ ] `WState::UnPack` · 006b7aa0 · legions 0064f2c0 — ac2-script WLib/WState/ByteStream decode
- [ ] `WState::WState` · 006b7b00 · legions 0064f320
- [ ] `WState::GetDBOType` · 006b7b50
- [-] `WState::GetPackSize` · 006b7b70 — inline copy of 006b7a30
- [ ] `WState::Pack` · 006b7b80
- [ ] `WState::UnPack` · 006b7b90
- [ ] `WState::WState` · 006b7ba0 · legions 0064f3c0
- [ ] `WState::Allocate` · 006b7bf0
- [ ] `WState::GetSubDataIDs` · 006b7c60

### ReferenceTable
- [ ] `ReferenceTable::GetNativeType` · 006b7d40 · legions 0064f560

### WState
- [ ] `WState::SaveStateByID` · 006b7da0 · legions 0064f5c0
- [ ] `WState::SaveStateByType` · 006b7ea0 · legions 0064f6c0

## `portal/shared/src/weenie/repository/wrepository.cpp.c` (0/101)

### CWRepository
- [ ] `CWRepository::Construct` · 00530e70 · legions 00514e10 — qualities kept per object in ac2-weenie (WeenieQualities); no DB defaults (InitDefaults) or DID derivation
- [ ] `CWRepository::~CWRepository` · 00530f10 · legions 00514eb0
- [ ] `CWRepository::SetWeenieDesc` · 00530f40 · legions 00514ee0 — qualities kept per object in ac2-weenie (WeenieQualities); no DB defaults (InitDefaults) or DID derivation
- [ ] `CWRepository::SetQualities` · 00530f70 · legions 00514f10
- [ ] `CWRepository::HandleEnterWorld` · 00530f90 · legions 00514f30
- [ ] `CWRepository::HandleLeaveWorld` · 00530fa0 · legions 00514f40
- [ ] `CWRepository::InqWeenieDescription` · 00530fb0
- [ ] `CWRepository::InqInt` · 00530fd0 · legions 00514f70 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::InqLongInt` · 00530ff0 · legions 00514f90 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::InqBool` · 00531010 · legions 00514fb0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::InqFloat` · 00531030 · legions 00514fd0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::InqStringW` · 00531050 · legions 00514ff0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::InqDataID` · 00531070 · legions 00515010 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::InqInstanceID` · 00531090 · legions 00515030 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::InqPosition` · 005310b0 · legions 00515050 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::InqTimestamp` · 005310d0 · legions 00515070 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::InqPackageID` · 005310f0 · legions 00515090 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::InqStringInfo` · 00531110 · legions 005150b0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::GetInt` · 00531130 · legions 005150d0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::GetBool` · 00531160 · legions 00515100 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::GetFloat` · 00531190 · legions 00515130 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::GetDataID` · 005311c0 · legions 00515160 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::GetInstanceID` · 005311f0 · legions 00515190 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::GetTimestamp` · 00531250 · legions 005151f0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::GetStringInfo` · 00531280 · legions 00515290 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::GetPackageID` · 005312f0 · legions 00515220 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::GetLongInt` · 00531320 · legions 00515250 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::DoesIntExist` · 00531360 · legions 00515300 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::DoesLongIntExist` · 00531380 · legions 00515320 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::DoesBoolExist` · 005313a0 · legions 00515340 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::DoesFloatExist` · 005313c0 · legions 00515360 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::DoesDataIDExist` · 005313e0 · legions 00515380 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::DoesInstanceIDExist` · 00531400 · legions 005153a0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::DoesPositionExist` · 00531420 · legions 005153c0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::DoesTimestampExist` · 00531490 · legions 00515430 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::DoesStringInfoExist` · 005314b0 · legions 00515450 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::DoesPackageIDExist` · 005314f0 · legions 00515490 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::SetInt` · 00531510 · legions 005154b0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::SetLongInt` · 00531540 · legions 005154e0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::SetBool` · 00531570 · legions 00515510 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::SetFloat` · 005315a0 · legions 00515540 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::SetDataID` · 005315d0 · legions 00515570 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::SetInstanceID` · 00531600 · legions 005155a0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::SetPosition` · 00531630 · legions 005155d0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::SetTimestamp` · 00531660 · legions 00515600 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::SetPackageID` · 00531690 · legions 00515630 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::SetStringInfo` · 005316c0 · legions 00515660 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::RemoveInt` · 005316f0 · legions 00515690 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::RemoveLongInt` · 00531710 · legions 005156b0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::RemoveBool` · 00531730 · legions 005156d0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::RemoveFloat` · 00531750 · legions 005156f0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::RemoveString` · 00531770 · legions 00515710 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::RemoveDataID` · 00531790 · legions 00515730 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::RemoveInstanceID` · 005317b0 · legions 00515750 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::RemovePosition` · 005317d0 · legions 00515770 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::RemoveTimestamp` · 005317f0 · legions 00515790 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::RemoveStringInfo` · 00531810 · legions 005157b0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::RemovePackageID` · 00531830 · legions 005157d0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::AdjInt` · 00531850 · legions 005157f0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::AdjAndTruncateInt` · 00531880 · legions 00515820 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::AdjLongInt` · 005318c0 · legions 00515860 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::AdjAndTruncateLongInt` · 00531910 · legions 005158b0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::AdjFloat` · 00531970 · legions 00515910 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::AdjAndTruncateFloat` · 005319b0 · legions 00515950 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::RecvEvt_UpdateInt_Private` · 005319f0 · legions 00515990
- [ ] `CWRepository::RecvEvt_UpdateLongInt_Private` · 00531a30 · legions 005159d0
- [ ] `CWRepository::RecvEvt_UpdateBool_Private` · 00531a70 · legions 00515a10
- [ ] `CWRepository::RecvEvt_UpdateFloat_Private` · 00531ab0 · legions 00515a50
- [ ] `CWRepository::RecvEvt_UpdateTimestamp_Private` · 00531af0 · legions 00515a90
- [ ] `CWRepository::RecvEvt_UpdateString_Private` · 00531b30 · legions 00515ad0
- [ ] `CWRepository::RecvEvt_UpdateDataID_Private` · 00531b70 · legions 00515b10
- [ ] `CWRepository::RecvEvt_UpdateInstanceID_Private` · 00531bb0 · legions 00515b50
- [ ] `CWRepository::RecvEvt_UpdatePosition_Private` · 00531bf0 · legions 00515b90
- [ ] `CWRepository::RecvEvt_UpdateStringInfo_Private` · 00531c30 · legions 00515bd0
- [ ] `CWRepository::CWRepository` · 00531c70 · legions 00515c10
- [ ] `CWRepository::GetPhysObjID` · 00531ce0 · legions 00515c80
- [ ] `CWRepository::GetScale` · 00531d10 · legions 00515cb0
- [ ] `CWRepository::EdgeSlides` · 00531d40 · legions 00515ce0
- [ ] `CWRepository::InqEtherealType` · 00531d70 · legions 00515d10
- [ ] `CWRepository::SetScale` · 00531df0 · legions 00515d90
- [ ] `CWRepository::SetEdgeSlides` · 00531e30 · legions 00515dd0
- [ ] `CWRepository::SetEtherealType` · 00531e60 · legions 00515e00
- [ ] `CWRepository::GatherPhysicsState` · 00531ed0 · legions 00515e70
- [ ] `CWRepository::GetStringW` · 00531fc0 · legions 00516020 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::GetRawPosition` · 00532050 · legions 00515f60 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::DoesStringExist` · 00532110 · legions 005160b0 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::SetStringW` · 00532170 · legions 00516110 — ac2-script: reached through the script glue; quality tables only (no weenie-desc/defaults), notices/chat requests handed to the host
- [ ] `CWRepository::LoadDataIDW` · 00532200 · legions 005161a0
- [ ] `CWRepository::BuildPhysObjProfile` · 00532290 · legions 00516230
- [ ] `CWRepository::DumpWeenieDesc` · 00532380 · legions 00516320
- [ ] `CWRepository::DumpWeenieDescW` · 00532510 · legions 005164b0
- [ ] `CWRepository::DumpBoolsToString` · 00532560 · legions 00516500
- [ ] `CWRepository::DumpDIDsToString` · 005327b0 · legions 00516750
- [ ] `CWRepository::DumpFloatsToString` · 005329f0 · legions 00516990
- [ ] `CWRepository::DumpIIDsToString` · 00532c40 · legions 00516be0
- [ ] `CWRepository::DumpIntsToString` · 00532e80 · legions 00516e20
- [ ] `CWRepository::DumpLongIntsToString` · 005330c0 · legions 00517060
- [ ] `CWRepository::DumpStringsToString` · 00533300 · legions 005172a0
- [ ] `CWRepository::DumpTSsToString` · 00533550 · legions 005174f0
- [ ] `CWRepository::DumpPositionsToString` · 00533790 · legions 00517730
- [ ] `CWRepository::DumpStringInfosToString` · 00533a70 · legions 00517a10

## `portal/shared/src/weenie/system/log/wlogsystem.cpp.c` (0/10)

### WLogSystem
- [ ] `WLogSystem::IsFileLogging` · 007332e0 · legions 006af790
- [ ] `WLogSystem::IsPrintLogging` · 00733300 · legions 006af7b0
- [ ] `WLogSystem::SetFileLog` · 00733320 · legions 006af7d0
- [ ] `WLogSystem::SetPrintLog` · 00733330 · legions 006af7e0
- [ ] `WLogSystem::AddFileLog` · 00733340 · legions 006af7f0
- [ ] `WLogSystem::AddPrintLog` · 00733360 · legions 006af810
- [ ] `WLogSystem::RemoveFileLog` · 00733380 · legions 006af830
- [ ] `WLogSystem::RemovePrintLog` · 007333a0 · legions 006af850
- [ ] `WLogSystem::GetFileLog` · 007333c0 · legions 006af870
- [ ] `WLogSystem::GetPrintLog` · 007333d0 · legions 006af880

## `portal/shared/src/weenie/wbookie/wbookkeeper.cpp.c` (0/5)

### CWBookKeeper
- [ ] `CWBookKeeper::CWBookKeeper` · 0052dd90 · legions 00512230
- [ ] `CWBookKeeper::CSendToTurbineRoomByID` · 0052ddb0 · legions 00512250 — ac2-weenie OnlineSession::send_to_room / receive_chat over ac2-net turbine_chat (chatclient.dll's protocol)
- [ ] `CWBookKeeper::AddObjectToBeDestroyed` · 0052ddf0 · legions 00512290
- [ ] `CWBookKeeper::RemoveObjectToBeDestroyed` · 0052de00 · legions 005122a0
- [ ] `CWBookKeeper::~CWBookKeeper` · 0052de10 · legions 005122b0

## `portal/shared/src/weenie/weenieobject.cpp.c` (0/9)

### CWeenieObject
- [ ] `CWeenieObject::IsMobile` · 0052fa90 · legions 00513c50
- [ ] `CWeenieObject::ObjectBeingDestructed` · 0052fab0 · legions 00513c70
- [ ] `CWeenieObject::CWeenieObject` · 0052fad0 · legions 00513c90
- [ ] `CWeenieObject::GetNoticeHandlers` · 0052fb00 · legions 00513cc0
- [ ] `CWeenieObject::RegisterNoticeHandler` · 0052fb40 · legions 00513d00
- [ ] `CWeenieObject::UnRegisterNoticeHandler` · 0052fc40 · legions 00513e00

### GlobalEventHandler
- [ ] `GlobalEventHandler::RemoveHandlerFromList` · 0052fcd0 · legions 0048ba20

### CWeenieObject
- [ ] `CWeenieObject::~CWeenieObject` · 0052fd40 · legions 00513e90
- [ ] `CWeenieObject::UnRegisterAllNoticeHandlers` · 0052fd70 · legions 00513ec0

## `portal/shared/src/weenie/wphysobj/wphysicsobject.cpp.c` (0/91)

### CWPhysicsObject
- [ ] `CWPhysicsObject::CWPhysicsObject` · 0052fe30 · legions 00513f80
- [ ] `CWPhysicsObject::~CWPhysicsObject` · 0052fe50 · legions 00513fa0
- [ ] `CWPhysicsObject::HandleCreation` · 0052fe60 · legions 00513fb0
- [ ] `CWPhysicsObject::HandleDestruction` · 0052fe80 · legions 00513fd0
- [ ] `CWPhysicsObject::DoIntersection` · 0052fe90
- [ ] `CWPhysicsObject::PerformLOS` · 0052ff20 · legions 00514020
- [ ] `CWPhysicsObject::SetCollisionState` · 0052ff40 · legions 00514040
- [ ] `CWPhysicsObject::GetCollisionState` · 0052ff60 · legions 00514060
- [ ] `CWPhysicsObject::MissileInitPlaceAndFire` · 0052ff80 · legions 00514080
- [ ] `CWPhysicsObject::MissileInitialize` · 0052ffa0 · legions 005140a0
- [ ] `CWPhysicsObject::MissilePlaceAndFire` · 0052ffc0 · legions 005140c0
- [ ] `CWPhysicsObject::MissilePlace` · 0052ffe0 · legions 005140e0
- [ ] `CWPhysicsObject::MissileFireOnly` · 00530000 · legions 00514100
- [ ] `CWPhysicsObject::MissileInquireDirection` · 00530020 · legions 00514120
- [ ] `CWPhysicsObject::MissileInquireCurrentVelocity` · 00530040 · legions 00514140
- [ ] `CWPhysicsObject::MissileRetarget` · 00530060 · legions 00514160
- [ ] `CWPhysicsObject::InitVisualDesc` · 00530080 · legions 00514180
- [ ] `CWPhysicsObject::CombineAndApplyVisualDesc` · 005300b0 · legions 005141b0
- [ ] `CWPhysicsObject::SetLocalVisualDesc` · 005300d0 · legions 005141d0
- [ ] `CWPhysicsObject::ApplyAdditiveVisualDesc` · 005300f0 · legions 005141f0
- [ ] `CWPhysicsObject::ApplySubtractiveVisualDesc` · 00530110 · legions 00514210
- [ ] `CWPhysicsObject::ApplyVisualDescToPhysicsObj` · 00530130 · legions 00514230
- [ ] `CWPhysicsObject::RecvEvt_UpdateVisualDesc` · 00530150 · legions 00514250 — ac2-weenie ObjectTable::apply_action
- [ ] `CWPhysicsObject::ForceVisualDesc` · 00530190 · legions 00514290
- [ ] `CWPhysicsObject::DoFX` · 005301b0 · legions 005142b0
- [ ] `CWPhysicsObject::StopFX` · 005301d0 · legions 005142d0
- [ ] `CWPhysicsObject::DoTargettedFX` · 005301f0 · legions 005142f0
- [ ] `CWPhysicsObject::DoBehavior` · 00530220
- [ ] `CWPhysicsObject::DoBehaviorNonLatent` · 005302c0 · legions 00514360
- [ ] `CWPhysicsObject::ReleaseBehavior` · 005302f0
- [ ] `CWPhysicsObject::DoMode` · 00530390
- [ ] `CWPhysicsObject::DoModeNonLatent` · 00530420 · legions 00514420
- [ ] `CWPhysicsObject::SetMode` · 00530440 · legions 00514440
- [ ] `CWPhysicsObject::DoSlider` · 00530460
- [ ] `CWPhysicsObject::DoSliderNonLatent` · 00530500 · legions 005144a0
- [ ] `CWPhysicsObject::StopBehavior` · 00530530 · legions 005144d0
- [ ] `CWPhysicsObject::StopLayer` · 00530550 · legions 005144f0
- [ ] `CWPhysicsObject::StopMoveToCancelsBehaviors` · 00530570 · legions 00514510
- [ ] `CWPhysicsObject::StopAllWeenieBehaviors` · 00530590 · legions 00514530
- [ ] `CWPhysicsObject::LookAtSelection` · 005305b0 · legions 00514550
- [ ] `CWPhysicsObject::LookAt` · 005305e0 · legions 00514580
- [ ] `CWPhysicsObject::AddTargetedImpulse` · 00530610 · legions 005145b0
- [ ] `CWPhysicsObject::GetElement` · 00530650 · legions 005145f0
- [ ] `CWPhysicsObject::GetDesiredMode` · 00530670 · legions 00514610
- [ ] `CWPhysicsObject::GetMode` · 00530690 · legions 00514630
- [ ] `CWPhysicsObject::IsDoingBehavior` · 005306b0 · legions 00514650
- [ ] `CWPhysicsObject::HasMotionInterpreter` · 005306d0 · legions 00514670
- [ ] `CWPhysicsObject::GetLastDoMotionValues` · 005306f0 · legions 00514690
- [ ] `CWPhysicsObject::HasRemoteDoMotionValues` · 00530710 · legions 005146b0
- [ ] `CWPhysicsObject::GetRemoteDoMotionValues` · 00530730 · legions 005146d0
- [ ] `CWPhysicsObject::GetDesiredMotionValues` · 00530750 · legions 005146f0
- [ ] `CWPhysicsObject::GetStandardMotionValues` · 00530770 · legions 00514710
- [ ] `CWPhysicsObject::GetMyPosition` · 00530790 · legions 00514730
- [ ] `CWPhysicsObject::GetDrawingPosition` · 005307b0 · legions 00514750
- [ ] `CWPhysicsObject::GetBoundingBox` · 005307d0 · legions 00514770
- [ ] `CWPhysicsObject::GetWorldMatrix` · 005307f0 · legions 00514790
- [ ] `CWPhysicsObject::IsVisible` · 00530810 · legions 005147b0
- [ ] `CWPhysicsObject::IsOutside` · 00530820 · legions 005147c0
- [ ] `CWPhysicsObject::IsInWorld` · 00530850 · legions 005147f0
- [ ] `CWPhysicsObject::LocalToGlobal` · 00530870 · legions 00514810
- [ ] `CWPhysicsObject::CalculateDistance` · 00530890 · legions 00514830
- [ ] `CWPhysicsObject::GetVelocityScale` · 005308c0 · legions 00514860
- [ ] `CWPhysicsObject::SetVelocityScale` · 005308e0 · legions 00514880
- [ ] `CWPhysicsObject::GetAccelerationScale` · 00530900 · legions 005148a0
- [ ] `CWPhysicsObject::SetAccelerationScale` · 00530920 · legions 005148c0
- [ ] `CWPhysicsObject::GetJumpScale` · 00530940 · legions 005148e0
- [ ] `CWPhysicsObject::SetJumpScale` · 00530960 · legions 00514900
- [ ] `CWPhysicsObject::Notify_DoMode` · 00530980 · legions 00514920
- [ ] `CWPhysicsObject::SetNoDrawPhysics` · 005309a0 · legions 00514940
- [ ] `CWPhysicsObject::SetCloaked` · 005309c0 · legions 00514960
- [ ] `CWPhysicsObject::SetUndetectable` · 005309e0 · legions 00514980
- [ ] `CWPhysicsObject::InqCurrentProperties` · 00530ae0 · legions 00514a80
- [ ] `CWPhysicsObject::InqCurrentProperty` · 00530b00 · legions 00514aa0
- [ ] `CWPhysicsObject::GetHeight` · 00530b20 · legions 00514ac0
- [ ] `CWPhysicsObject::GetRadius` · 00530b40 · legions 00514ae0
- [ ] `CWPhysicsObject::GetTopLevelRadius` · 00530b60 · legions 00514b00
- [ ] `CWPhysicsObject::GetTopLevelHeight` · 00530b80 · legions 00514b20
- [ ] `CWPhysicsObject::IsMoving` · 00530ba0 · legions 00514b40
- [ ] `CWPhysicsObject::IsUndetectable` · 00530bc0 · legions 00514b60
- [ ] `CWPhysicsObject::IsMovementFrozen` · 00530be0 · legions 00514b80
- [ ] `CWPhysicsObject::SetMovementFrozen` · 00530c00 · legions 00514ba0
- [ ] `CWPhysicsObject::IsAnimationFrozen` · 00530c30 · legions 00514bd0
- [ ] `CWPhysicsObject::SetAnimationFrozen` · 00530c50 · legions 00514bf0
- [ ] `CWPhysicsObject::TogglePlaceMarkerObjects` · 00530c80 · legions 00514c20
- [ ] `CWPhysicsObject::StopAllCameraFX` · 00530cb0 · legions 00514c50
- [ ] `CWPhysicsObject::DoStory` · 00530cc0 · legions 00514c60
- [ ] `CWPhysicsObject::InitPhysics` · 00530ce0 · legions 00514c80
- [ ] `CWPhysicsObject::GetLocalVisualDesc` · 00530d50 · legions 00514cf0
- [ ] `CWPhysicsObject::AddImpulse` · 00530da0 · legions 00514d40
- [ ] `CWPhysicsObject::AddImpulseLocal` · 00530de0 · legions 00514d80
- [ ] `CWPhysicsObject::InqCylSphere` · 00530e20 · legions 00514dc0
