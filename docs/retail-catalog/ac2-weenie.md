# ac2-weenie: Weenie objects, client events, actions, the session

1904 retail functions: 825 done, 181 partial, 390 to do, 508 skipped.
Legend and workflow: [README](README.md).

## `ac2/client/src/cfactory/gmcenginefactory.cpp.c` (1/11)

### gmCEngineFactory
- [-] `gmCEngineFactory::Init` · 004017e0 · legions 00401750 — engine factory / singleton allocator: Rust builds and owns these directly
- [-] `gmCEngineFactory::MakeClientCache_Internal` · 00401810 — engine factory: Rust builds the subsystems directly
- [-] `gmCEngineFactory::MakeUI_Internal` · 00401830 — engine factory: Rust builds the subsystems directly

### NoticeHandler
- [x] `NoticeHandler::RecvNotice_CharGenVerification` · 00401850 · legions 00401bb0 — NoticeHandler base virtual: retail body returns (Legions 0x00401BB0 is empty); the pregame subclasses dispatch in ac2-ui pregame/ui.rs
- [-] `NoticeHandler::RecvNotice_DisplayAdminStringInfo` · 00401860 · legions 00401b50 — admin-only: admin string info display (no admin tools in the client)

### ArgumentParser
- [-] `ArgumentParser::FinishOutputText` · 00401870 · legions 006af780 — admin/debug/logging tooling: not in the client

### NoticeHandler
- [ ] `NoticeHandler::RecvNotice_CharacterSet` · 00401880 · legions 00401ba0 — needs port: notice handler::recv notice character set; no citation in crates/

### gmCEngineFactory
- [-] `gmCEngineFactory::MakeClient_Internal` · 00401890 — engine factory: Rust builds the subsystems directly
- [-] `gmCEngineFactory::MakeCEventQueue_Internal` · 004018b0 — engine factory: Rust builds the subsystems directly
- [-] `gmCEngineFactory::MakeGlobalEventHandler_Internal` · 004018d0 — engine factory: Rust builds the subsystems directly
- [-] `gmCEngineFactory::MakeCObjectMaint_Internal` · 004018f0 — engine factory: Rust builds the subsystems directly

## `ac2/client/src/events/cm_allegiance.cpp.c` (4/4)

### CM_Allegiance
- [x] `CM_Allegiance::SendNotice_Patron_SwearDone` · 00727f80 — implemented: cited at crates/ac2-weenie/src/notice.rs:120
- [x] `CM_Allegiance::SendNotice_RequestVassal` · 00728020 — implemented: cited at crates/ac2-weenie/src/notice.rs:118
- [x] `CM_Allegiance::SendNotice_UpdateAllegianceProfile` · 007280c0 — implemented: cited at crates/ac2-weenie/src/notice.rs:116
- [x] `CM_Allegiance::SendNotice_Vassal_SwearDone` · 00728150 — implemented: cited at crates/ac2-weenie/src/notice.rs:122

## `ac2/client/src/events/cm_character.cpp.c` (22/30)

### CM_Character
- [x] `CM_Character::SendNotice_AddMarker` · 007248b0 · legions 006a4270 — ac2-client client_radar.rs + ac2-script host/radar.rs (InqRadarLook), ac2-weenie notice.rs (ChangeRadarLook(obj))
- [~] `CM_Character::SendNotice_AddPet` · 00724970 · legions 006a42e0 — ac2-ui pets.rs: AddPet and PetIsUnderAttack reach the panel; RemovePet and UpdatePetName have no identified Legions handler, so the panel leaves them unchanged
- [x] `CM_Character::SendNotice_ChangeRadarLook` · 00724a30 — ac2-client client_radar.rs + ac2-script host/radar.rs (InqRadarLook), ac2-weenie notice.rs (ChangeRadarLook(obj))
- [-] `CM_Character::SendNotice_ChangeRadarShowCoordsPreference` · 00724ac0 — no Legions 7160 counterpart: the Legions script table (glue/efun_names.txt) has no efun that raises it; admin 7148 only. Shortcut height is set through gGameplayOptionsProfile (glue/options.rs)
- [x] `CM_Character::SendNotice_ClearMarkers` · 00724b50 · legions 006a43e0 — ac2-client client_radar.rs + ac2-script host/radar.rs (InqRadarLook), ac2-weenie notice.rs (ChangeRadarLook(obj))
- [x] `CM_Character::SendNotice_DisplayMessage` · 00724be0 · legions 006a4430 — retail handler registered by gmGamePlayUI::SetupChildren (Init at login); boxes raised in the world wait in HudView::display_backlog until the HUD dialogs exist (tests/message_box_hud_7160.rs)
- [~] `CM_Character::SendNotice_PetIsUnderAttack` · 00724c70 — ac2-ui pets.rs: AddPet and PetIsUnderAttack reach the panel; RemovePet and UpdatePetName have no identified Legions handler, so the panel leaves them unchanged
- [x] `CM_Character::SendNotice_PulseMissed` · 00724d00 — typed notice (ac2-script notices_typed) reaches its Legions panel in ac2-client (hud_player (frame_passes notices), hud_status, hud_death, hud_game_dialogs, target_select, hud_view reticle, hud_shortcut_checks)
- [-] `CM_Character::SendNotice_ReloadGameplayOptions` · 00724d90 — no Legions 7160 counterpart: the Legions script table (glue/efun_names.txt) has no efun that raises it; admin 7148 only. Shortcut height is set through gGameplayOptionsProfile (glue/options.rs)
- [x] `CM_Character::SendNotice_ReloadOptions` · 00724e20 — typed notice (ac2-script notices_typed) reaches its Legions panel in ac2-client (hud_player (frame_passes notices), hud_status, hud_death, hud_game_dialogs, target_select, hud_view reticle, hud_shortcut_checks)
- [x] `CM_Character::SendNotice_RemoveMarker` · 00724eb0 · legions 006a45e0 — ac2-client client_radar.rs + ac2-script host/radar.rs (InqRadarLook), ac2-weenie notice.rs (ChangeRadarLook(obj))
- [~] `CM_Character::SendNotice_RemovePet` · 00724f50 — ac2-ui pets.rs: AddPet and PetIsUnderAttack reach the panel; RemovePet and UpdatePetName have no identified Legions handler, so the panel leaves them unchanged
- [x] `CM_Character::SendNotice_ResurrectionRequestedByOther` · 00724fe0 — typed notice (ac2-script notices_typed) reaches its Legions panel in ac2-client (hud_player (frame_passes notices), hud_status, hud_death, hud_game_dialogs, target_select, hud_view reticle, hud_shortcut_checks)
- [-] `CM_Character::SendNotice_Select` · 00725070 — no Legions 7160 counterpart: the Legions script table (glue/efun_names.txt) has no efun that raises it; admin 7148 only. Shortcut height is set through gGameplayOptionsProfile (glue/options.rs)
- [x] `CM_Character::SendNotice_SelectionChanged` · 00725100 — typed notice (ac2-script notices_typed) reaches its Legions panel in ac2-client (hud_player (frame_passes notices), hud_status, hud_death, hud_game_dialogs, target_select, hud_view reticle, hud_shortcut_checks)
- [-] `CM_Character::SendNotice_SetShortcutHeight` · 00725190 — no Legions 7160 counterpart: the Legions script table (glue/efun_names.txt) has no efun that raises it; admin 7148 only. Shortcut height is set through gGameplayOptionsProfile (glue/options.rs)
- [x] `CM_Character::SendNotice_UpdateDeathState` · 00725220 — ac2-ui death.rs (DeathPanel: the resurrect dialog, its text, its button's respawn); client hud_death.rs wires the notice and the answer; death_panel_7160 and combat_7160 (death, respawn)
- [x] `CM_Character::SendNotice_UpdateEffect` · 007252b0 — typed notice (ac2-script notices_typed) reaches its Legions panel in ac2-client (hud_player (frame_passes notices), hud_status, hud_death, hud_game_dialogs, target_select, hud_view reticle, hud_shortcut_checks)
- [x] `CM_Character::SendNotice_UpdateFactionStatus` · 00725340 — PK field fed: Legions 0x0044E290 is gmPKStatusUI::RecvNotice_UpdateFactionStatus (the index missed its match; 1 peace, 2 control, 3 factional, 4 ffa, else stateless), raised by CM_Character::SendNotice_UpdateFactionStatus; routed through ac2-client hud_status::pk_notice
- [x] `CM_Character::SendNotice_UpdateHeroStatus` · 007253d0 — typed notice (ac2-script notices_typed) reaches its Legions panel in ac2-client (hud_player (frame_passes notices), hud_status, hud_death, hud_game_dialogs, target_select, hud_view reticle, hud_shortcut_checks)
- [x] `CM_Character::SendNotice_UpdateMountState` · 00725490 — typed notice (ac2-script notices_typed) reaches its Legions panel in ac2-client (hud_player (frame_passes notices), hud_status, hud_death, hud_game_dialogs, target_select, hud_view reticle, hud_shortcut_checks)
- [x] `CM_Character::SendNotice_UpdatePetMode` · 00725520 — ac2-ui pets.rs update_pet_mode (gmPetUI::RecvNotice_UpdatePetMode: server mode; a row following the old mode moves its wanted mode and text); ac2-client hud_pets.rs
- [~] `CM_Character::SendNotice_UpdatePetName` · 007255c0 — ac2-ui pets.rs: AddPet and PetIsUnderAttack reach the panel; RemovePet and UpdatePetName have no identified Legions handler, so the panel leaves them unchanged
- [x] `CM_Character::SendNotice_UpdateResetTime` · 00725660 — typed notice (ac2-script notices_typed) reaches its Legions panel in ac2-client (hud_player (frame_passes notices), hud_status, hud_death, hud_game_dialogs, target_select, hud_view reticle, hud_shortcut_checks)
- [x] `CM_Character::SendNotice_UpdateSelectability` · 007256f0 — typed notice (ac2-script notices_typed) reaches its Legions panel in ac2-client (hud_player (frame_passes notices), hud_status, hud_death, hud_game_dialogs, target_select, hud_view reticle, hud_shortcut_checks)
- [x] `CM_Character::SendNotice_UpdateSelectionInfo` · 00725780 — typed notice (ac2-script notices_typed) reaches its Legions panel in ac2-client (hud_player (frame_passes notices), hud_status, hud_death, hud_game_dialogs, target_select, hud_view reticle, hud_shortcut_checks)
- [x] `CM_Character::SendNotice_UpdateSkill` · 00725810 — typed notice (ac2-script notices_typed) reaches its Legions panel in ac2-client (hud_player (frame_passes notices), hud_status, hud_death, hud_game_dialogs, target_select, hud_view reticle, hud_shortcut_checks)
- [x] `CM_Character::SendNotice_UpdateSkillPanel` · 007258a0 — typed notice (ac2-script notices_typed) reaches its Legions panel in ac2-client (hud_player (frame_passes notices), hud_status, hud_death, hud_game_dialogs, target_select, hud_view reticle, hud_shortcut_checks)
- [x] `CM_Character::SendNotice_UpdateVitae` · 00725950 — PlayerVitals.vitae; reached through CPlayer 0x127 on the VM, tested with the double's vitae qualities
- [x] `CM_Character::SendNotice_UpdateVital` · 007259f0 — implemented: cited at crates/ac2-ui/src/vitals.rs:23

## `ac2/client/src/events/cm_combat.cpp.c` (4/5)

### CM_Combat
- [x] `CM_Combat::SendNotice_EndSpecialAttack` · 00727cb0 — implemented: cited at crates/ac2-weenie/src/notice.rs:207
- [x] `CM_Combat::SendNotice_RequestSpecialAttack` · 00727d40 — implemented: cited at crates/ac2-net/src/protocol/gameplay/combat.rs:4
- [-] `CM_Combat::SendNotice_StartSpecialAttack` · 00727dd0 — no route found: CPlayer 0xD (0x1AA4000D) is called by no wlib function (CALL and CALLSTATIC scans, raw word scan); its only callee is CCombatSystem 0x2, which calls the gCM_Combat start efun; no WM_* package, CAgent forwarder or ECM_* event reaches it. A server InterpCEvent naming 0x9AA4000D would still reach it; no retail server message does
- [x] `CM_Combat::SendNotice_ToggleSpecialAttack` · 00727e60 — implemented: cited at crates/ac2-weenie/src/notice.rs:209
- [x] `CM_Combat::SendNotice_UpdateAttackState` · 00727ef0 — implemented: cited at crates/ac2-weenie/src/notice.rs:19

## `ac2/client/src/events/cm_communication.cpp.c` (3/3)

### CM_Communication
- [x] `CM_Communication::SendNotice_DisplayStringInfo` · 00725cc0 — implemented: cited at crates/ac2-net/src/protocol/gameplay/mod.rs:149
- [x] `CM_Communication::SendNotice_DoAssistedTell` · 00725d50 — implemented: cited at crates/ac2-weenie/src/notice.rs:211
- [x] `CM_Communication::SendNotice_ReloadChatSettings` · 00725de0 — implemented: cited at crates/ac2-weenie/src/notice.rs:213

## `ac2/client/src/events/cm_craft.cpp.c` (5/6)

### CM_Craft
- [x] `CM_Craft::SendNotice_AddIngredient_Done` · 00727900 — net: client call over the script VM (craft_script_7160); DAT-backed answers and the request bytes tested
- [x] `CM_Craft::SendNotice_ExecuteCraft_Done` · 007279d0 — net: client call over the script VM (craft_script_7160); DAT-backed answers and the request bytes tested
- [x] `CM_Craft::SendNotice_OpenIngredientPanel` · 00727a60 — net: client call over the script VM (craft_script_7160); DAT-backed answers and the request bytes tested
- [~] `CM_Craft::SendNotice_UpdateCost` · 00727af0 — net: bound; no wlib function calls efun 0x00880004 (scanned the Legions wlib's CALL_EFUN, THIS/LOAD/RLOAD_CALL_EFUN sites), so no WM_Craft answer reaches it
- [x] `CM_Craft::SendNotice_UpdateCraftPanel` · 00727b90 — craft_script_7160. Forwarders (wlib): WM_Craft 0x19 (the registry) -> CPlayer 0xC3, 0x1B (recipe learned by DID) -> CPlayer 0x105, 0x1A/0x37 (recipe records) -> CPlayer 0x61/0x1C5 and 0x27/0x34 (skill records) -> CPlayer 0x1AE/0x1C4 each raise it; the double sends 0x1A after a craft. 0x18 -> CPlayer 0xF9 calls efun 0x8F0003 (unbound) and is not driven.
- [x] `CM_Craft::SendNotice_UpdateCraftSkill` · 00727c20 — lead: WM_Craft 0x27/0x34 -> CPlayer 0x1AE/0x1C4 set a CraftSkillRecord, then this notice with the skill's adaptor and UpdateCraftPanel; craft_script_7160 updates_change_only_known_records asserts it.

## `ac2/client/src/events/cm_examination.cpp.c` (2/2)

### CM_Examination
- [x] `CM_Examination::SendNotice_ExaminationRequestChanged` · 007277e0 — implemented: cited at crates/ac2-weenie/src/notice.rs:28
- [x] `CM_Examination::SendNotice_UpdateExaminationProfile` · 00727870 — implemented: cited at crates/ac2-weenie/src/notice.rs:30

## `ac2/client/src/events/cm_fellowship.cpp.c` (13/13)

### CM_Fellowship
- [x] `CM_Fellowship::SendNotice_ClearFellow` · 00726fe0 — implemented: cited at crates/ac2-weenie/src/notice.rs:90
- [x] `CM_Fellowship::SendNotice_ClearFellowship` · 00727070 — implemented: cited at crates/ac2-weenie/src/notice.rs:92
- [x] `CM_Fellowship::SendNotice_ExpireRecruitment` · 00727100 — implemented: cited at crates/ac2-weenie/src/notice.rs:114
- [x] `CM_Fellowship::SendNotice_RequestRecruitment` · 00727190 — implemented: cited at crates/ac2-weenie/src/notice.rs:108
- [x] `CM_Fellowship::SendNotice_UpdateFellow` · 00727240 — implemented: cited at crates/ac2-weenie/src/notice.rs:88
- [x] `CM_Fellowship::SendNotice_UpdateFellowCellID` · 007272e0 — implemented: cited at crates/ac2-weenie/src/notice.rs:106
- [x] `CM_Fellowship::SendNotice_UpdateFellowHealth` · 00727380 — implemented: cited at crates/ac2-weenie/src/notice.rs:96
- [x] `CM_Fellowship::SendNotice_UpdateFellowLevel` · 00727430 — implemented: cited at crates/ac2-weenie/src/notice.rs:104
- [x] `CM_Fellowship::SendNotice_UpdateFellowMaxHealth` · 007274d0 — implemented: cited at crates/ac2-weenie/src/notice.rs:100
- [x] `CM_Fellowship::SendNotice_UpdateFellowMaxVigor` · 00727570 — implemented: cited at crates/ac2-weenie/src/notice.rs:102
- [x] `CM_Fellowship::SendNotice_UpdateFellowVigor` · 00727610 — implemented: cited at crates/ac2-weenie/src/notice.rs:98
- [x] `CM_Fellowship::SendNotice_UpdateFellowship` · 007276c0 — implemented: cited at crates/ac2-weenie/src/notice.rs:86
- [x] `CM_Fellowship::SendNotice_UpdateLeader` · 00727750 — implemented: cited at crates/ac2-weenie/src/notice.rs:94

## `ac2/client/src/events/cm_friends.cpp.c` (5/5)

### CM_Friends
- [x] `CM_Friends::SendNotice_GUNResult_AddFriend` · 00725e70 — net: friends_script_7160 (the login's GUN requests are the VM's exact values; Result_FriendAdd/Delete IFuns (ids 0xB40040/41, match Legions) raise FriendAdded/FriendRemoved with the name)
- [x] `CM_Friends::SendNotice_GUNResult_FriendStatus` · 00725f00 — implemented: cited at crates/ac2-weenie/src/notice.rs:280
- [x] `CM_Friends::SendNotice_GUNResult_RemoveFriend` · 00725fb0 — net: friends_script_7160 (the login's GUN requests are the VM's exact values; Result_FriendAdd/Delete IFuns (ids 0xB40040/41, match Legions) raise FriendAdded/FriendRemoved with the name)
- [x] `CM_Friends::SendNotice_LocalStatus` · 00726040 — implemented: cited at crates/ac2-weenie/src/notice.rs:275
- [x] `CM_Friends::SendNotice_GUNResult_GetFriendsList` · 007260d0 — implemented: cited at crates/ac2-weenie/src/notice.rs:277

## `ac2/client/src/events/cm_inventory.cpp.c` (4/4)

### CM_Inventory
- [x] `CM_Inventory::SendNotice_ContainerStateUpdate` · 00726c10 — implemented: cited at crates/ac2-weenie/src/notice.rs:55
- [x] `CM_Inventory::SendNotice_ItemUpdate` · 00726cb0 — implemented: cited at crates/ac2-weenie/src/notice.rs:57
- [x] `CM_Inventory::SendNotice_UpdateContainer` · 00726d40 — implemented: cited at crates/ac2-weenie/src/notice.rs:47
- [x] `CM_Inventory::SendNotice_UpdateInventory` · 00726e10 — implemented: cited at crates/ac2-weenie/src/notice.rs:60

## `ac2/client/src/events/cm_money.cpp.c` (2/2)

### CM_Money
- [x] `CM_Money::SendNotice_GhostMoneyBag` · 00726ec0 — implemented: cited at crates/ac2-weenie/src/notice.rs:137
- [x] `CM_Money::SendNotice_UpdateMoney` · 00726f50 — implemented: cited at crates/ac2-weenie/src/notice.rs:135

## `ac2/client/src/events/cm_physics.cpp.c` (2/2)

### CM_Physics
- [x] `CM_Physics::RecvEvt_CreateObject` · 007269d0 · legions 006a55c0 — implemented: cited at crates/ac2-net/src/protocol/opcode.rs:93
- [x] `CM_Physics::SendNotice_Do3DDamage` · 00726b80 — implemented: cited at crates/ac2-weenie/src/notice.rs:314

## `ac2/client/src/events/cm_quest.cpp.c` (3/3)

### CM_Quest
- [x] `CM_Quest::SendNotice_UpdateQuests` · 00725ab0 — implemented: cited at crates/ac2-weenie/src/notice.rs:125
- [x] `CM_Quest::SendNotice_UpdateStoryQuests` · 00725b40 — implemented: cited at crates/ac2-ui/src/story_quests.rs:6
- [x] `CM_Quest::SendNotice_PlaySequence` · 00725bd0 — implemented: cited at crates/ac2-ui/src/story_quests.rs:34

## `ac2/client/src/events/cm_shortcut.cpp.c` (1/2)

### CM_Shortcut
- [x] `CM_Shortcut::SendNotice_ReloadShortcuts` · 007268b0 — implemented: cited at crates/ac2-weenie/src/notice.rs:81
- [ ] `CM_Shortcut::SendNotice_RemoveRecipe` · 00726940 — needs port: notice RemoveRecipe to the UI/client; no citation in crates/

## `ac2/client/src/events/cm_store.cpp.c` (1/12)

### CM_Store
- [~] `CM_Store::SendNotice_ConfirmPurchase` · 00724000 — Typed as GameplayNotice (ac2-script host/notices_typed.rs) with the argument kinds of each SendNotice signature and its gCM_Store glue; LeaveStore raised on the VM (WM_Store 0x13 path). ConfirmPurchase is raised locally by CStoreSystem 0x3 (PurchaseItem), not by a server message. Not yet run on the VM with a store open.
- [-] `CM_Store::SendNotice_InitNewSale` · 007240b0 — No gCM_Store efun and no WLib path reaches it (not script-raised).
- [~] `CM_Store::SendNotice_LeaveCatalog` · 00724150 — Typed as GameplayNotice (ac2-script host/notices_typed.rs) with the argument kinds of each SendNotice signature and its gCM_Store glue; LeaveStore raised on the VM (WM_Store 0x13 path). ConfirmPurchase is raised locally by CStoreSystem 0x3 (PurchaseItem), not by a server message. Not yet run on the VM with a store open.
- [~] `CM_Store::SendNotice_LeaveConsignment` · 007241e0 — net: LeaveConsignment is raised by the close-all body (CStoreSystem 0x1C, from ViewStore and the consignment view), not by a WM_Store opcode of its own. DisplaySaleReminders (IFun 0xB4013D, id matches Legions) is not exercised
- [~] `CM_Store::SendNotice_LeaveStore` · 00724270 — Typed as GameplayNotice (ac2-script host/notices_typed.rs) with the argument kinds of each SendNotice signature and its gCM_Store glue; LeaveStore raised on the VM (WM_Store 0x13 path). ConfirmPurchase is raised locally by CStoreSystem 0x3 (PurchaseItem), not by a server message. Not yet run on the VM with a store open.
- [~] `CM_Store::SendNotice_ShowSaleReminder` · 00724300 — Typed as GameplayNotice (ac2-script host/notices_typed.rs) with the argument kinds of each SendNotice signature and its gCM_Store glue; LeaveStore raised on the VM (WM_Store 0x13 path). ConfirmPurchase is raised locally by CStoreSystem 0x3 (PurchaseItem), not by a server message. Not yet run on the VM with a store open.
- [~] `CM_Store::SendNotice_UpdateSale` · 00724390 — Typed as GameplayNotice (ac2-script host/notices_typed.rs) with the argument kinds of each SendNotice signature and its gCM_Store glue; LeaveStore raised on the VM (WM_Store 0x13 path). ConfirmPurchase is raised locally by CStoreSystem 0x3 (PurchaseItem), not by a server message. Not yet run on the VM with a store open.
- [~] `CM_Store::SendNotice_UpdateConsignment` · 00724430 — lead: WM_Store 0x53 (CPlayer 0x1F1 -> CStoreSystem 0x17) / 0x57 (CPlayer 0x1F6 -> CStoreSystem 0x16) take an RList of Consignments (package 0x3898: slot 0 the GameSaleProfile, 3 ordinal, 4 stock, 8 bit 0 expired) and the storekeeper; the build at 0x118008 makes a UIShopSale per consignment with stock (sale build 0x118530). ac2-net ConsignmentList; consignment_script_7160 asserts both notices' shops.
- [~] `CM_Store::SendNotice_UpdateStore` · 00724520 — Typed as GameplayNotice (ac2-script host/notices_typed.rs) with the argument kinds of each SendNotice signature and its gCM_Store glue; LeaveStore raised on the VM (WM_Store 0x13 path). ConfirmPurchase is raised locally by CStoreSystem 0x3 (PurchaseItem), not by a server message. Not yet run on the VM with a store open.
- [~] `CM_Store::SendNotice_ViewConsignment` · 00724610 — lead: WM_Store 0x53 (CPlayer 0x1F1 -> CStoreSystem 0x17) / 0x57 (CPlayer 0x1F6 -> CStoreSystem 0x16) take an RList of Consignments (package 0x3898: slot 0 the GameSaleProfile, 3 ordinal, 4 stock, 8 bit 0 expired) and the storekeeper; the build at 0x118008 makes a UIShopSale per consignment with stock (sale build 0x118530). ac2-net ConsignmentList; consignment_script_7160 asserts both notices' shops.
- [~] `CM_Store::SendNotice_ViewStore` · 00724700 — Typed as GameplayNotice (ac2-script host/notices_typed.rs) with the argument kinds of each SendNotice signature and its gCM_Store glue; LeaveStore raised on the VM (WM_Store 0x13 path). ConfirmPurchase is raised locally by CStoreSystem 0x3 (PurchaseItem), not by a server message. Not yet run on the VM with a store open.
- [x] `CM_Store::SendNotice_ViewCatalog` · 007247f0 · legions 006a41f0 — lead: WM_Store 0x52 -> CPlayer 0x1F3 -> CStoreSystem 0xF (vendor, StoreGroup DID 0x3858, AAHash store DID -> size): closes store/consignment/catalog, sets the shopper's group (CShopperContext 0x2), lists the group's StoreTemplates (build 0x117914) and raises it; ac2-net StoreCatalog; consignment_script_7160 view_catalog asserts the shops and sizes.

## `ac2/client/src/events/cm_trade.cpp.c` (0/9)

### CM_Trade
- [~] `CM_Trade::SendNotice_Trade_BeAcceptanceCleared` · 00726310 — raised by CPlayer's WM_Trade forwarders on the VM (fids 0x46/0x4E/0x53/0x50/0x51/0x52/0x44/0x56 in package 0x215, ac2-net gameplay/trade.rs); Offered/Opened/Registered/Accepted/Closed/Reset/Cleared/Declined verified on the VM, Revoked not
- [~] `CM_Trade::SendNotice_Trade_BeAccepted` · 007263a0 — raised by CPlayer's WM_Trade forwarders on the VM (fids 0x46/0x4E/0x53/0x50/0x51/0x52/0x44/0x56 in package 0x215, ac2-net gameplay/trade.rs); Offered/Opened/Registered/Accepted/Closed/Reset/Cleared/Declined verified on the VM, Revoked not
- [~] `CM_Trade::SendNotice_Trade_BeClosed` · 00726430 — raised by CPlayer's WM_Trade forwarders on the VM (fids 0x46/0x4E/0x53/0x50/0x51/0x52/0x44/0x56 in package 0x215, ac2-net gameplay/trade.rs); Offered/Opened/Registered/Accepted/Closed/Reset/Cleared/Declined verified on the VM, Revoked not
- [~] `CM_Trade::SendNotice_Trade_BeDeclined` · 007264d0 — raised by CPlayer's WM_Trade forwarders on the VM (fids 0x46/0x4E/0x53/0x50/0x51/0x52/0x44/0x56 in package 0x215, ac2-net gameplay/trade.rs); Offered/Opened/Registered/Accepted/Closed/Reset/Cleared/Declined verified on the VM, Revoked not
- [~] `CM_Trade::SendNotice_Trade_BeOffered` · 00726560 — raised by CPlayer's WM_Trade forwarders on the VM (fids 0x46/0x4E/0x53/0x50/0x51/0x52/0x44/0x56 in package 0x215, ac2-net gameplay/trade.rs); Offered/Opened/Registered/Accepted/Closed/Reset/Cleared/Declined verified on the VM, Revoked not
- [~] `CM_Trade::SendNotice_Trade_BeOpened` · 00726620 — raised by CPlayer's WM_Trade forwarders on the VM (fids 0x46/0x4E/0x53/0x50/0x51/0x52/0x44/0x56 in package 0x215, ac2-net gameplay/trade.rs); Offered/Opened/Registered/Accepted/Closed/Reset/Cleared/Declined verified on the VM, Revoked not
- [~] `CM_Trade::SendNotice_Trade_BeRegistered` · 007266b0 — raised by CPlayer's WM_Trade forwarders on the VM (fids 0x46/0x4E/0x53/0x50/0x51/0x52/0x44/0x56 in package 0x215, ac2-net gameplay/trade.rs); Offered/Opened/Registered/Accepted/Closed/Reset/Cleared/Declined verified on the VM, Revoked not
- [~] `CM_Trade::SendNotice_Trade_BeReset` · 00726770 — raised by CPlayer's WM_Trade forwarders on the VM (fids 0x46/0x4E/0x53/0x50/0x51/0x52/0x44/0x56 in package 0x215, ac2-net gameplay/trade.rs); Offered/Opened/Registered/Accepted/Closed/Reset/Cleared/Declined verified on the VM, Revoked not
- [~] `CM_Trade::SendNotice_Trade_BeRevoked` · 00726800 — raised by CPlayer's WM_Trade forwarders on the VM (fids 0x46/0x4E/0x53/0x50/0x51/0x52/0x44/0x56 in package 0x215, ac2-net gameplay/trade.rs); Offered/Opened/Registered/Accepted/Closed/Reset/Cleared/Declined verified on the VM, Revoked not

## `ac2/client/src/events/cm_ui.cpp.c` (14/21)

### CM_UI
- [x] `CM_UI::SendNotice_CloseConfirmationDialog` · 007232e0 — implemented: cited at crates/ac2-ui/src/dialog.rs:10
- [x] `CM_UI::SendNotice_CloseMessageDialog` · 00723380 — implemented: cited at crates/ac2-ui/src/dialog.rs:46
- [x] `CM_UI::SendNotice_CloseResurrectDialog` · 00723410 — ac2-ui death.rs (DeathPanel: the resurrect dialog, its text, its button's respawn); client hud_death.rs wires the notice and the answer; death_panel_7160 and combat_7160 (death, respawn)
- [x] `CM_UI::SendNotice_CloseStackUI` · 007234a0 · legions 006a36a0 — implemented: cited at crates/ac2-ui/src/item_display.rs:533
- [ ] `CM_UI::SendNotice_EndCharacterSession` · 00723530 — needs port: notice EndCharacterSession to the UI/client; no citation in crates/
- [ ] `CM_UI::SendNotice_GunAuthenticated` · 007235c0 — needs port: notice GunAuthenticated to the UI/client; no citation in crates/
- [ ] `CM_UI::SendNotice_GunConnected` · 00723650 — needs port: notice GunConnected to the UI/client; no citation in crates/
- [ ] `CM_UI::SendNotice_GunConnectionLost` · 007236e0 — needs port: notice GunConnectionLost to the UI/client; no citation in crates/
- [ ] `CM_UI::SendNotice_GunReconnected` · 00723770 — needs port: notice GunReconnected to the UI/client; no citation in crates/
- [ ] `CM_UI::SendNotice_HandleDragFrom` · 00723800 · legions 006a3880 — needs port: notice HandleDragFrom to the UI/client; no citation in crates/
- [x] `CM_UI::SendNotice_InitStackUI` · 007238a0 · legions 006a38e0 — implemented: cited at crates/ac2-ui/src/money_bag.rs:4
- [ ] `CM_UI::SendNotice_Logoff` · 00723980 — needs port: notice Logoff to the UI/client; no citation in crates/
- [x] `CM_UI::SendNotice_OpenExaminationPanel` · 00723a10 · legions 006a39b0 — implemented: cited at crates/ac2-ui/src/consignment.rs:63
- [x] `CM_UI::SendNotice_PortalSpace` · 00723aa0 — implemented: cited at crates/ac2-weenie/src/notice.rs:257
- [x] `CM_UI::SendNotice_PrecludeFromBottom` · 00723b30 — implemented: cited at crates/ac2-client/src/hud_view.rs:374
- [x] `CM_UI::SendNotice_PrecludeFromTop` · 00723bc0 — implemented: cited at crates/ac2-ui/src/statusbar.rs:25
- [x] `CM_UI::SendNotice_SaveLocationsChanged` · 00723c50 · legions 006a3af0 — implemented: cited at crates/ac2-ui/src/window_locations.rs:272
- [x] `CM_UI::SendNotice_StackUIDone` · 00723ce0 · legions 006a3b40 — implemented: cited at crates/ac2-ui/src/stack.rs:6
- [x] `CM_UI::SendNotice_UpdateCanMouseLook` · 00723d80 · legions 006a3ba0 — implemented: cited at crates/ac2-ui/src/gameplay_story.rs:42
- [x] `CM_UI::SendNotice_FinishPlaySequence` · 00723e10 — implemented: cited at crates/ac2-ui/src/story.rs:11
- [x] `CM_UI::SendNotice_PortalScene` · 00723f00 — implemented: cited at crates/ac2-weenie/src/notice.rs:131

## `ac2/client/src/events/cm_usage.cpp.c` (0/2)

### CM_Usage
- [~] `CM_Usage::SendNotice_SetTargetingPointer` · 007261c0 — ac2-ui selection_notices.rs set_targeting_pointer and ac2-client target_select.rs take the pointer when the player's VM raises it; without the VM, ac2-weenie usage.rs use_item sends the use with its target directly (no multi-item collection; usage_7160 test)
- [~] `CM_Usage::SendNotice_UseBook` · 00726250 · legions 006a5160 — ac2-script host/notices.rs (oiiwwi) and notices_typed.rs UseBook -> GameplayNotice::UseBook -> ac2-client hud_book.rs; the handler list fan-out is the frame's event list

## `ac2/client/src/events/gmceventqueue.cpp.c` (1/2)

### gmCEventQueue
- [-] `gmCEventQueue::gmCEventQueue` · 00723260 — constructor of plain data: Rust struct init replaces it
- [x] `gmCEventQueue::DispatchEvent` · 00723280 · legions 006a3540 — implemented: cited at crates/ac2-net/src/protocol/opcode.rs:4

## `ac2/client/src/events/gmglobaleventhandler.cpp.c` (0/2)

### gmGlobalEventHandler
- [-] `gmGlobalEventHandler::gmGlobalEventHandler` · 007232c0 · legions 006a3580 — constructor of plain data: Rust struct init replaces it
- [-] `gmGlobalEventHandler::gmGetGlobalEventHandler` · 007232d0 · legions 006a3590 — singleton accessor: Rust owns the handler directly

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
- [-] `HashSet<InstanceID>::add` · 0046a890 · legions 0046b170 — STL/container internals: a Rust collection replaces it

### gAllegianceProfileAdaptor
- [x] `gAllegianceProfileAdaptor::CallEFun_AddOfficer` · 0046a900 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)

## `ac2/client/src/weenie/interp/glue/gcm_allegiance.cpp.c` (4/4)

### gCM_Allegiance
- [x] `gCM_Allegiance::CallEFun_SendNotice_Patron_SwearDone` · 0046a340 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs
- [x] `gCM_Allegiance::CallEFun_SendNotice_RequestVassal` · 0046a390 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs
- [x] `gCM_Allegiance::CallEFun_SendNotice_UpdateAllegianceProfile` · 0046a3f0 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs
- [x] `gCM_Allegiance::CallEFun_SendNotice_Vassal_SwearDone` · 0046a440 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs

## `ac2/client/src/weenie/interp/glue/gcm_character.cpp.c` (22/26)

### gCM_Character
- [x] `gCM_Character::CallEFun_SendNotice_AddMarker` · 00469a90 — ac2-client client_radar.rs + ac2-script host/radar.rs (InqRadarLook), ac2-weenie notice.rs (ChangeRadarLook(obj))
- [~] `gCM_Character::CallEFun_SendNotice_AddPet` · 00469b10 · legions 0046a500 — ac2-ui pets.rs: AddPet and PetIsUnderAttack reach the panel; RemovePet and UpdatePetName have no identified Legions handler, so the panel leaves them unchanged
- [x] `gCM_Character::CallEFun_SendNotice_ChangeRadarLook` · 00469b80 — ac2-client client_radar.rs + ac2-script host/radar.rs (InqRadarLook), ac2-weenie notice.rs (ChangeRadarLook(obj))
- [x] `gCM_Character::CallEFun_SendNotice_ClearMarkers` · 00469bc0 — ac2-client client_radar.rs + ac2-script host/radar.rs (InqRadarLook), ac2-weenie notice.rs (ChangeRadarLook(obj))
- [x] `gCM_Character::CallEFun_SendNotice_DisplayMessage` · 00469c10 — retail handler registered by gmGamePlayUI::SetupChildren (Init at login); boxes raised in the world wait in HudView::display_backlog until the HUD dialogs exist (tests/message_box_hud_7160.rs)
- [~] `gCM_Character::CallEFun_SendNotice_PetIsUnderAttack` · 00469c60 — ac2-ui pets.rs: AddPet and PetIsUnderAttack reach the panel; RemovePet and UpdatePetName have no identified Legions handler, so the panel leaves them unchanged
- [x] `gCM_Character::CallEFun_SendNotice_PulseMissed` · 00469cb0 — typed notice (ac2-script notices_typed) reaches its Legions panel in ac2-client (hud_player (frame_passes notices), hud_status, hud_death, hud_game_dialogs, target_select, hud_view reticle, hud_shortcut_checks)
- [x] `gCM_Character::CallEFun_SendNotice_ReloadOptions` · 00469d00 — typed notice (ac2-script notices_typed) reaches its Legions panel in ac2-client (hud_player (frame_passes notices), hud_status, hud_death, hud_game_dialogs, target_select, hud_view reticle, hud_shortcut_checks)
- [x] `gCM_Character::CallEFun_SendNotice_RemoveMarker` · 00469d40 — ac2-client client_radar.rs + ac2-script host/radar.rs (InqRadarLook), ac2-weenie notice.rs (ChangeRadarLook(obj))
- [~] `gCM_Character::CallEFun_SendNotice_RemovePet` · 00469d90 — ac2-ui pets.rs: AddPet and PetIsUnderAttack reach the panel; RemovePet and UpdatePetName have no identified Legions handler, so the panel leaves them unchanged
- [x] `gCM_Character::CallEFun_SendNotice_ResurrectionRequestedByOther` · 00469de0 — typed notice (ac2-script notices_typed) reaches its Legions panel in ac2-client (hud_player (frame_passes notices), hud_status, hud_death, hud_game_dialogs, target_select, hud_view reticle, hud_shortcut_checks)
- [x] `gCM_Character::CallEFun_SendNotice_SelectionChanged` · 00469e30 — typed notice (ac2-script notices_typed) reaches its Legions panel in ac2-client (hud_player (frame_passes notices), hud_status, hud_death, hud_game_dialogs, target_select, hud_view reticle, hud_shortcut_checks)
- [x] `gCM_Character::CallEFun_SendNotice_UpdateDeathState` · 00469e80 — ac2-ui death.rs (DeathPanel: the resurrect dialog, its text, its button's respawn); client hud_death.rs wires the notice and the answer; death_panel_7160 and combat_7160 (death, respawn)
- [x] `gCM_Character::CallEFun_SendNotice_UpdateEffect` · 00469ed0 — typed notice (ac2-script notices_typed) reaches its Legions panel in ac2-client (hud_player (frame_passes notices), hud_status, hud_death, hud_game_dialogs, target_select, hud_view reticle, hud_shortcut_checks)
- [x] `gCM_Character::CallEFun_SendNotice_UpdateFactionStatus` · 00469f10 — PK field fed: Legions 0x0044E290 is gmPKStatusUI::RecvNotice_UpdateFactionStatus (the index missed its match; 1 peace, 2 control, 3 factional, 4 ffa, else stateless), raised by CM_Character::SendNotice_UpdateFactionStatus; routed through ac2-client hud_status::pk_notice
- [x] `gCM_Character::CallEFun_SendNotice_UpdateHeroStatus` · 00469f60 — typed notice (ac2-script notices_typed) reaches its Legions panel in ac2-client (hud_player (frame_passes notices), hud_status, hud_death, hud_game_dialogs, target_select, hud_view reticle, hud_shortcut_checks)
- [x] `gCM_Character::CallEFun_SendNotice_UpdateMountState` · 00469fd0 — typed notice (ac2-script notices_typed) reaches its Legions panel in ac2-client (hud_player (frame_passes notices), hud_status, hud_death, hud_game_dialogs, target_select, hud_view reticle, hud_shortcut_checks)
- [x] `gCM_Character::CallEFun_SendNotice_UpdatePetMode` · 0046a020 — ac2-ui pets.rs update_pet_mode (gmPetUI::RecvNotice_UpdatePetMode: server mode; a row following the old mode moves its wanted mode and text); ac2-client hud_pets.rs
- [~] `gCM_Character::CallEFun_SendNotice_UpdatePetName` · 0046a070 — ac2-ui pets.rs: AddPet and PetIsUnderAttack reach the panel; RemovePet and UpdatePetName have no identified Legions handler, so the panel leaves them unchanged
- [x] `gCM_Character::CallEFun_SendNotice_UpdateResetTime` · 0046a0d0 — typed notice (ac2-script notices_typed) reaches its Legions panel in ac2-client (hud_player (frame_passes notices), hud_status, hud_death, hud_game_dialogs, target_select, hud_view reticle, hud_shortcut_checks)
- [x] `gCM_Character::CallEFun_SendNotice_UpdateSelectability` · 0046a110 — typed notice (ac2-script notices_typed) reaches its Legions panel in ac2-client (hud_player (frame_passes notices), hud_status, hud_death, hud_game_dialogs, target_select, hud_view reticle, hud_shortcut_checks)
- [x] `gCM_Character::CallEFun_SendNotice_UpdateSelectionInfo` · 0046a160 — typed notice (ac2-script notices_typed) reaches its Legions panel in ac2-client (hud_player (frame_passes notices), hud_status, hud_death, hud_game_dialogs, target_select, hud_view reticle, hud_shortcut_checks)
- [x] `gCM_Character::CallEFun_SendNotice_UpdateSkill` · 0046a1b0 — typed notice (ac2-script notices_typed) reaches its Legions panel in ac2-client (hud_player (frame_passes notices), hud_status, hud_death, hud_game_dialogs, target_select, hud_view reticle, hud_shortcut_checks)
- [x] `gCM_Character::CallEFun_SendNotice_UpdateSkillPanel` · 0046a200 — typed notice (ac2-script notices_typed) reaches its Legions panel in ac2-client (hud_player (frame_passes notices), hud_status, hud_death, hud_game_dialogs, target_select, hud_view reticle, hud_shortcut_checks)
- [x] `gCM_Character::CallEFun_SendNotice_UpdateVitae` · 0046a260 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs
- [x] `gCM_Character::CallEFun_SendNotice_UpdateVital` · 0046a2d0 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs

## `ac2/client/src/weenie/interp/glue/gcm_combat.cpp.c` (4/5)

### gCM_Combat
- [x] `gCM_Combat::CallEFun_SendNotice_EndSpecialAttack` · 00469900 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs
- [x] `gCM_Combat::CallEFun_SendNotice_RequestSpecialAttack` · 00469950 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs
- [-] `gCM_Combat::CallEFun_SendNotice_StartSpecialAttack` · 004699a0 — no route found: CPlayer 0xD (0x1AA4000D) is called by no wlib function (CALL and CALLSTATIC scans, raw word scan); its only callee is CCombatSystem 0x2, which calls the gCM_Combat start efun; no WM_* package, CAgent forwarder or ECM_* event reaches it. A server InterpCEvent naming 0x9AA4000D would still reach it; no retail server message does
- [x] `gCM_Combat::CallEFun_SendNotice_ToggleSpecialAttack` · 004699f0 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs
- [x] `gCM_Combat::CallEFun_SendNotice_UpdateAttackState` · 00469a40 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs

## `ac2/client/src/weenie/interp/glue/gcm_communication.cpp.c` (3/3)

### gCM_Communication
- [x] `gCM_Communication::CallEFun_SendNotice_DisplayStringInfo` · 00469820 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs
- [x] `gCM_Communication::CallEFun_SendNotice_DoAssistedTell` · 00469870 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs
- [x] `gCM_Communication::CallEFun_SendNotice_ReloadChatSettings` · 004698c0 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs

## `ac2/client/src/weenie/interp/glue/gcm_craft.cpp.c` (5/6)

### gCM_Craft
- [x] `gCM_Craft::CallEFun_SendNotice_AddIngredient_Done` · 00469620 — net: client call over the script VM (craft_script_7160); DAT-backed answers and the request bytes tested
- [x] `gCM_Craft::CallEFun_SendNotice_ExecuteCraft_Done` · 004696a0 — net: client call over the script VM (craft_script_7160); DAT-backed answers and the request bytes tested
- [x] `gCM_Craft::CallEFun_SendNotice_OpenIngredientPanel` · 004696f0 — net: client call over the script VM (craft_script_7160); DAT-backed answers and the request bytes tested
- [~] `gCM_Craft::CallEFun_SendNotice_UpdateCost` · 00469740 — net: bound; no wlib function calls efun 0x00880004 (scanned the Legions wlib's CALL_EFUN, THIS/LOAD/RLOAD_CALL_EFUN sites), so no WM_Craft answer reaches it
- [x] `gCM_Craft::CallEFun_SendNotice_UpdateCraftPanel` · 00469790 — craft_script_7160. Forwarders (wlib): WM_Craft 0x19 (the registry) -> CPlayer 0xC3, 0x1B (recipe learned by DID) -> CPlayer 0x105, 0x1A/0x37 (recipe records) -> CPlayer 0x61/0x1C5 and 0x27/0x34 (skill records) -> CPlayer 0x1AE/0x1C4 each raise it; the double sends 0x1A after a craft. 0x18 -> CPlayer 0xF9 calls efun 0x8F0003 (unbound) and is not driven.
- [x] `gCM_Craft::CallEFun_SendNotice_UpdateCraftSkill` · 004697d0 — lead: WM_Craft 0x27/0x34 -> CPlayer 0x1AE/0x1C4 set a CraftSkillRecord, then this notice with the skill's adaptor and UpdateCraftPanel; craft_script_7160 updates_change_only_known_records asserts it.

## `ac2/client/src/weenie/interp/glue/gcm_examination.cpp.c` (2/2)

### gCM_Examination
- [x] `gCM_Examination::CallEFun_SendNotice_ExaminationRequestChanged` · 00469580 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs
- [x] `gCM_Examination::CallEFun_SendNotice_UpdateExaminationProfile` · 004695d0 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs

## `ac2/client/src/weenie/interp/glue/gcm_fellowship.cpp.c` (13/13)

### gCM_Fellowship
- [x] `gCM_Fellowship::CallEFun_SendNotice_ClearFellow` · 00469130 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs
- [x] `gCM_Fellowship::CallEFun_SendNotice_ClearFellowship` · 00469180 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs
- [x] `gCM_Fellowship::CallEFun_SendNotice_ExpireRecruitment` · 004691c0 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs
- [x] `gCM_Fellowship::CallEFun_SendNotice_RequestRecruitment` · 00469210 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs
- [x] `gCM_Fellowship::CallEFun_SendNotice_UpdateFellow` · 00469280 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs
- [x] `gCM_Fellowship::CallEFun_SendNotice_UpdateFellowCellID` · 004692e0 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs
- [x] `gCM_Fellowship::CallEFun_SendNotice_UpdateFellowHealth` · 00469330 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs
- [x] `gCM_Fellowship::CallEFun_SendNotice_UpdateFellowLevel` · 00469390 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs
- [x] `gCM_Fellowship::CallEFun_SendNotice_UpdateFellowMaxHealth` · 004693e0 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs
- [x] `gCM_Fellowship::CallEFun_SendNotice_UpdateFellowMaxVigor` · 00469430 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs
- [x] `gCM_Fellowship::CallEFun_SendNotice_UpdateFellowVigor` · 00469480 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs
- [x] `gCM_Fellowship::CallEFun_SendNotice_UpdateFellowship` · 004694e0 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs
- [x] `gCM_Fellowship::CallEFun_SendNotice_UpdateLeader` · 00469530 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs

## `ac2/client/src/weenie/interp/glue/gcm_friends.cpp.c` (5/6)

### gCM_Friends
- [x] `gCM_Friends::CallEFun_SendNotice_GUNResult_AddFriend` · 00467f60 — net: friends_script_7160 (the login's GUN requests are the VM's exact values; Result_FriendAdd/Delete IFuns (ids 0xB40040/41, match Legions) raise FriendAdded/FriendRemoved with the name)
- [x] `gCM_Friends::CallEFun_SendNotice_GUNResult_FriendStatus` · 00467fb0 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs
- [x] `gCM_Friends::CallEFun_SendNotice_GUNResult_RemoveFriend` · 00468010 — net: friends_script_7160 (the login's GUN requests are the VM's exact values; Result_FriendAdd/Delete IFuns (ids 0xB40040/41, match Legions) raise FriendAdded/FriendRemoved with the name)
- [x] `gCM_Friends::CallEFun_SendNotice_LocalStatus` · 00468060 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs
- [-] `gCM_Friends::RListToGUNUserStatusList` · 004680b0 · legions 00468d60 — STL/container internals: a Rust collection replaces it
- [x] `gCM_Friends::CallEFun_SendNotice_GUNResult_GetFriendsList` · 00468150 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs

## `ac2/client/src/weenie/interp/glue/gcm_inventory.cpp.c` (4/4)

### gCM_Inventory
- [x] `gCM_Inventory::CallEFun_SendNotice_ContainerStateUpdate` · 00468f10 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs
- [x] `gCM_Inventory::CallEFun_SendNotice_ItemUpdate` · 00468f70 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs
- [x] `gCM_Inventory::CallEFun_SendNotice_UpdateContainer` · 00468fb0 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs
- [x] `gCM_Inventory::CallEFun_SendNotice_UpdateInventory` · 00469030 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs

## `ac2/client/src/weenie/interp/glue/gcm_money.cpp.c` (2/2)

### gCM_Money
- [x] `gCM_Money::CallEFun_SendNotice_GhostMoneyBag` · 00469090 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs
- [x] `gCM_Money::CallEFun_SendNotice_UpdateMoney` · 004690e0 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs

## `ac2/client/src/weenie/interp/glue/gcm_physics.cpp.c` (1/1)

### gCM_Physics
- [x] `gCM_Physics::CallEFun_SendNotice_Do3DDamage` · 00468ec0 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs

## `ac2/client/src/weenie/interp/glue/gcm_quest.cpp.c` (2/3)

### gCM_Quest
- [x] `gCM_Quest::CallEFun_SendNotice_UpdateQuests` · 00468db0 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs
- [x] `gCM_Quest::CallEFun_SendNotice_UpdateStoryQuests` · 00468e00 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs
- [~] `gCM_Quest::CallEFun_SendNotice_PlaySequence` · 00468e50 — ac2-client client_story.rs: story slot routed; drawing, audio and camera FX are known gaps

## `ac2/client/src/weenie/interp/glue/gcm_shortcut.cpp.c` (1/2)

### gCM_Shortcut
- [x] `gCM_Shortcut::CallEFun_SendNotice_ReloadShortcuts` · 00468d20 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs
- [-] `gCM_Shortcut::CallEFun_SendNotice_RemoveRecipe` · 00468d60 — No Legions 7160 counterpart: glue/efun_names.txt has no efun for it; the Legions 0x00468D60 is a different function

## `ac2/client/src/weenie/interp/glue/gcm_store.cpp.c` (5/12)

### gCM_Store
- [x] `gCM_Store::CallEFun_SendNotice_ConfirmPurchase` · 00467a50 — ac2-script store round trip (store_script_7160): ViewStore with a StoreView's sales, UpdateStore after it, the ConfirmPurchase notice of PurchaseItem, LeaveStore in the shop body's prologue
- [~] `gCM_Store::CallEFun_SendNotice_LeaveCatalog` · 00467ac0 — Efun ids 0x3839xxxx in the notice table (ac2-script host/notices.rs); arguments as the glue reads them (frame in signature order). LeaveStore reached on the VM.
- [~] `gCM_Store::CallEFun_SendNotice_LeaveConsignment` · 00467b00 — net: LeaveConsignment is raised by the close-all body (CStoreSystem 0x1C, from ViewStore and the consignment view), not by a WM_Store opcode of its own. DisplaySaleReminders (IFun 0xB4013D, id matches Legions) is not exercised
- [x] `gCM_Store::CallEFun_SendNotice_LeaveStore` · 00467b40 — ac2-script store round trip (store_script_7160): ViewStore with a StoreView's sales, UpdateStore after it, the ConfirmPurchase notice of PurchaseItem, LeaveStore in the shop body's prologue
- [~] `gCM_Store::CallEFun_SendNotice_ShowSaleReminder` · 00467b80 — Efun ids 0x3839xxxx in the notice table (ac2-script host/notices.rs); arguments as the glue reads them (frame in signature order). LeaveStore reached on the VM.
- [~] `gCM_Store::CallEFun_SendNotice_UpdateSale` · 00467bd0 — Efun ids 0x3839xxxx in the notice table (ac2-script host/notices.rs); arguments as the glue reads them (frame in signature order). LeaveStore reached on the VM.
- [x] `gCM_Store::CallEFun_SendNotice_UpdateStore` · 00467c20 — ac2-script store round trip (store_script_7160): ViewStore with a StoreView's sales, UpdateStore after it, the ConfirmPurchase notice of PurchaseItem, LeaveStore in the shop body's prologue
- [~] `gCM_Store::CallEFun_SendNotice_ViewConsignment` · 00467c80 — lead: WM_Store 0x53 (CPlayer 0x1F1 -> CStoreSystem 0x17) / 0x57 (CPlayer 0x1F6 -> CStoreSystem 0x16) take an RList of Consignments (package 0x3898: slot 0 the GameSaleProfile, 3 ordinal, 4 stock, 8 bit 0 expired) and the storekeeper; the build at 0x118008 makes a UIShopSale per consignment with stock (sale build 0x118530). ac2-net ConsignmentList; consignment_script_7160 asserts both notices' shops.
- [x] `gCM_Store::CallEFun_SendNotice_ViewStore` · 00467ce0 — ac2-script store round trip (store_script_7160): ViewStore with a StoreView's sales, UpdateStore after it, the ConfirmPurchase notice of PurchaseItem, LeaveStore in the shop body's prologue
- [~] `gCM_Store::CallEFun_SendNotice_UpdateConsignment` · 00467d40 — lead: WM_Store 0x53 (CPlayer 0x1F1 -> CStoreSystem 0x17) / 0x57 (CPlayer 0x1F6 -> CStoreSystem 0x16) take an RList of Consignments (package 0x3898: slot 0 the GameSaleProfile, 3 ordinal, 4 stock, 8 bit 0 expired) and the storekeeper; the build at 0x118008 makes a UIShopSale per consignment with stock (sale build 0x118530). ac2-net ConsignmentList; consignment_script_7160 asserts both notices' shops.
- [-] `gCM_Store::RListToUIShopList` · 00467e20 · legions 00468ad0 — STL/container internals: a Rust collection replaces it
- [x] `gCM_Store::CallEFun_SendNotice_ViewCatalog` · 00467ec0 · legions 00468b70 — lead: WM_Store 0x52 -> CPlayer 0x1F3 -> CStoreSystem 0xF (vendor, StoreGroup DID 0x3858, AAHash store DID -> size): closes store/consignment/catalog, sets the shopper's group (CShopperContext 0x2), lists the group's StoreTemplates (build 0x117914) and raises it; ac2-net StoreCatalog; consignment_script_7160 view_catalog asserts the shops and sizes.

## `ac2/client/src/weenie/interp/glue/gcm_trade.cpp.c` (8/9)

### gCM_Trade
- [x] `gCM_Trade::CallEFun_SendNotice_Trade_BeAcceptanceCleared` · 00468a10 — ac2-script host notices: the efun raises its typed notice on the VM (WM_Trade forwarders, ac2-net gameplay/trade.rs ids); exercised by trade_script_7160 and the probe
- [x] `gCM_Trade::CallEFun_SendNotice_Trade_BeAccepted` · 00468a50 — ac2-script host notices: the efun raises its typed notice on the VM (WM_Trade forwarders, ac2-net gameplay/trade.rs ids); exercised by trade_script_7160 and the probe
- [x] `gCM_Trade::CallEFun_SendNotice_Trade_BeClosed` · 00468aa0 — ac2-script host notices: the efun raises its typed notice on the VM (WM_Trade forwarders, ac2-net gameplay/trade.rs ids); exercised by trade_script_7160 and the probe
- [x] `gCM_Trade::CallEFun_SendNotice_Trade_BeDeclined` · 00468af0 — ac2-script host notices: the efun raises its typed notice on the VM (WM_Trade forwarders, ac2-net gameplay/trade.rs ids); exercised by trade_script_7160 and the probe
- [x] `gCM_Trade::CallEFun_SendNotice_Trade_BeOffered` · 00468b40 — ac2-script host notices: the efun raises its typed notice on the VM (WM_Trade forwarders, ac2-net gameplay/trade.rs ids); exercised by trade_script_7160 and the probe
- [x] `gCM_Trade::CallEFun_SendNotice_Trade_BeOpened` · 00468bb0 — ac2-script host notices: the efun raises its typed notice on the VM (WM_Trade forwarders, ac2-net gameplay/trade.rs ids); exercised by trade_script_7160 and the probe
- [x] `gCM_Trade::CallEFun_SendNotice_Trade_BeRegistered` · 00468bf0 — ac2-script host notices: the efun raises its typed notice on the VM (WM_Trade forwarders, ac2-net gameplay/trade.rs ids); exercised by trade_script_7160 and the probe
- [x] `gCM_Trade::CallEFun_SendNotice_Trade_BeReset` · 00468c70 — ac2-script host notices: the efun raises its typed notice on the VM (WM_Trade forwarders, ac2-net gameplay/trade.rs ids); exercised by trade_script_7160 and the probe
- [~] `gCM_Trade::CallEFun_SendNotice_Trade_BeRevoked` · 00468cb0 — BeRevoked efun: raised on the VM only through forwarder 0x48 (CPlayer 0x10F), not exercised on the VM yet

## `ac2/client/src/weenie/interp/glue/gcm_ui.cpp.c` (2/3)

### gCM_UI
- [x] `gCM_UI::CallEFun_SendNotice_OpenExaminationPanel` · 00468940 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs
- [x] `gCM_UI::CallEFun_SendNotice_PortalSpace` · 00468970 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs
- [~] `gCM_UI::CallEFun_SendNotice_PortalScene` · 004689a0 — ac2-client client_story.rs: story slot routed; drawing, audio and camera FX are known gaps

## `ac2/client/src/weenie/interp/glue/gcm_usage.cpp.c` (2/2)

### gCM_Usage
- [x] `gCM_Usage::CallEFun_SendNotice_SetTargetingPointer` · 00468870 — Script notice glue: the efun is bound in ac2-script host/notices.rs (table of CM_*::SendNotice_* targets with argument kinds) and carries its efun id from glue/efun_names.txt; the raise reaches the UI through host/notices_typed.rs
- [x] `gCM_Usage::CallEFun_SendNotice_UseBook` · 004688c0 · legions 00469340 — ac2-script host/notices.rs entry 0x00920002 'oiiwwi' (argument order checked against the glue's stack slots)

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
- [ ] `ggmCObjMaint::CallEFun_GetPlayerID` · 00468210 — needs port: script efun get player id on the object maintenance; no citation in crates/
- [ ] `ggmCObjMaint::CallEFun_GetPlayerWeenieObjectReference` · 00468250 · legions 00468f00 — needs port: script efun get player weenie object reference on the object maintenance; no citation in crates/
- [-] `ggmCObjMaint::CallEFun_DumpToString` · 00468280 — admin dump-to-string / dump debug output: not in the client

## `ac2/client/src/weenie/interp/glue/ggmcwinterface.cpp.c` (124/301)

### PackObj
- [x] `PackObj::GET_SIZE_LEFT` · 00454060 · legions 004550e0 — implemented: cited at crates/ac2-script/src/bytestream/mod.rs:93

### HeapObject
- [x] `HeapObject::InitializeData` · 00454080 · legions 00455100 — implemented: cited at crates/ac2-script/src/vm/refs.rs:121

### VMMon
- [-] `VMMon::iBegin` · 004540e0 — admin/debug/logging tooling: not in the client
- [-] `VMMon::iEnd` · 00454140 — admin/debug/logging tooling: not in the client

### ReferenceTable
- [-] `ReferenceTable::GetTimeSinceLastGCRun` · 00454220 — VM garbage collector and timers: Rust ownership replaces the reference-table GC (AGENTS: no GC/reference reuse)
- [-] `ReferenceTable::GetTimeSinceLastGrow` · 00454250 — VM garbage collector and timers: Rust ownership replaces the reference-table GC (AGENTS: no GC/reference reuse)

### HeapObject
- [-] `HeapObject::operator_new` · 004542a0 · legions 004551b0 — operator= / copy of plain data: Rust value semantics replace it
- [-] `HeapObject::HeapObject` · 004542d0 · legions 004551e0 — constructor of plain data: Rust struct init replaces it

### WDataMapper
- [x] `WDataMapper::WStateIDToPackageID` · 00454300 · legions 00455210 — implemented: cited at crates/ac2-script/src/vm/mod.rs:392

### ReferenceTable
- [-] `ReferenceTable::MarkReference` · 00454350 · legions 00455260 — VM garbage collector and timers: Rust ownership replaces the reference-table GC (AGENTS: no GC/reference reuse)

### ContextState
- [x] `ContextState::Init` · 00454440 · legions 00455320 — implemented: cited at crates/ac2-script/src/vm/context.rs:58

### ggmCWInterface
- [x] `ggmCWInterface::CallEFun_PopupYesNoDialog` · 004544b0 — ac2-client hud_game_dialogs.rs: questions queued in HudView::popup_backlog until the HUD dialogs exist (tests/popup_hud_7160.rs)

### ReferenceEntry
- [-] `ReferenceEntry::GetPackageStateID` · 00454870 — VM garbage collector and timers: Rust ownership replaces the reference-table GC (AGENTS: no GC/reference reuse)

### ReferenceTable
- [-] `ReferenceTable::FindNextReference` · 00454890 · legions 004556f0 — VM garbage collector and timers: Rust ownership replaces the reference-table GC (AGENTS: no GC/reference reuse)
- [-] `ReferenceTable::ScanReservedRefs` · 004548d0 · legions 00455730 — VM garbage collector and timers: Rust ownership replaces the reference-table GC (AGENTS: no GC/reference reuse)
- [-] `ReferenceTable::MarkAndSweepBuffer` · 00454900 · legions 00455760 — VM garbage collector and timers: Rust ownership replaces the reference-table GC (AGENTS: no GC/reference reuse)
- [-] `ReferenceTable::DeleteAllUnUsedReferences` · 00454960 · legions 004557c0 — VM garbage collector and timers: Rust ownership replaces the reference-table GC (AGENTS: no GC/reference reuse)

### ggmCWInterface
- [~] `ggmCWInterface::CallEFun_RequestGUN_CallCreate` · 004549f0 — GUN-only (no Turbine GUN in this client): the script's request answers 0 without a GUN service, as RequestGUN_* does when TurbineGun::S_pTurbineGun is NULL (ac2-script host/gun.rs gun(); the page and friend requests gate the same way in ac2-client friends.rs); with a service the request is queued but the GUN answers and wire are not modelled; the Result and HearTell paths need those answers
- [~] `ggmCWInterface::CallEFun_RequestGUN_FriendAdd` · 00454a90 — net: the client's friend add and delete are queued for the GUN service (ChatRequest; friends.rs), with the double's FriendsBook as the GUN answer. The GUN wire is not modelled. The GUN-to-weenie result conversion is only its success case (0); the other codes are not traced
- [~] `ggmCWInterface::CallEFun_RequestGUN_FriendDelete` · 00454b10 — net: the client's friend add and delete are queued for the GUN service (ChatRequest; friends.rs), with the double's FriendsBook as the GUN answer. The GUN wire is not modelled. The GUN-to-weenie result conversion is only its success case (0); the other codes are not traced
- [x] `ggmCWInterface::CallEFun_RequestGUN_GetFriendsList` · 00454b90 — net: friends_script_7160 (the login's GUN requests are the VM's exact values; Result_FriendAdd/Delete IFuns (ids 0xB40040/41, match Legions) raise FriendAdded/FriendRemoved with the name)
- [~] `ggmCWInterface::CallEFun_RequestGUN_GetIgnoredList` · 00454c10 — GUN-only (no Turbine GUN in this client): the script's request answers 0 without a GUN service, as RequestGUN_* does when TurbineGun::S_pTurbineGun is NULL (ac2-script host/gun.rs gun(); the page and friend requests gate the same way in ac2-client friends.rs); with a service the request is queued but the GUN answers and wire are not modelled; the Result and HearTell paths need those answers
- [~] `ggmCWInterface::CallEFun_RequestGUN_IgnoredAdd` · 00454c90 — GUN-only (no Turbine GUN in this client): the script's request answers 0 without a GUN service, as RequestGUN_* does when TurbineGun::S_pTurbineGun is NULL (ac2-script host/gun.rs gun(); the page and friend requests gate the same way in ac2-client friends.rs); with a service the request is queued but the GUN answers and wire are not modelled; the Result and HearTell paths need those answers
- [~] `ggmCWInterface::CallEFun_RequestGUN_IgnoredDelete` · 00454d10 — GUN-only (no Turbine GUN in this client): the script's request answers 0 without a GUN service, as RequestGUN_* does when TurbineGun::S_pTurbineGun is NULL (ac2-script host/gun.rs gun(); the page and friend requests gate the same way in ac2-client friends.rs); with a service the request is queued but the GUN answers and wire are not modelled; the Result and HearTell paths need those answers
- [~] `ggmCWInterface::CallEFun_RequestGUN_SendToRoom` · 00454d90 — GUN-only (no Turbine GUN in this client): the script's request answers 0 without a GUN service, as RequestGUN_* does when TurbineGun::S_pTurbineGun is NULL (ac2-script host/gun.rs gun(); the page and friend requests gate the same way in ac2-client friends.rs); with a service the request is queued but the GUN answers and wire are not modelled; the Result and HearTell paths need those answers
- [~] `ggmCWInterface::CallEFun_RequestGUN_SendToUser` · 00454e20 — GUN-only (no Turbine GUN in this client): the script's request answers 0 without a GUN service, as RequestGUN_* does when TurbineGun::S_pTurbineGun is NULL (ac2-script host/gun.rs gun(); the page and friend requests gate the same way in ac2-client friends.rs); with a service the request is queued but the GUN answers and wire are not modelled; the Result and HearTell paths need those answers
- [~] `ggmCWInterface::CallEFun_RequestGUN_SetLocation` · 00454eb0 — GUN-only (no Turbine GUN in this client): the script's request answers 0 without a GUN service, as RequestGUN_* does when TurbineGun::S_pTurbineGun is NULL (ac2-script host/gun.rs gun(); the page and friend requests gate the same way in ac2-client friends.rs); with a service the request is queued but the GUN answers and wire are not modelled; the Result and HearTell paths need those answers
- [~] `ggmCWInterface::CallEFun_RequestGUN_SetStateFlags` · 00454f30 — GUN-only (no Turbine GUN in this client): the script's request answers 0 without a GUN service, as RequestGUN_* does when TurbineGun::S_pTurbineGun is NULL (ac2-script host/gun.rs gun(); the page and friend requests gate the same way in ac2-client friends.rs); with a service the request is queued but the GUN answers and wire are not modelled; the Result and HearTell paths need those answers

### VM
- [-] `VM::DeleteContext` · 00455000 · legions 00455c70 — VM garbage collector and timers: Rust ownership replaces the reference-table GC (AGENTS: no GC/reference reuse)

### Resolver
- [x] `Resolver::GetVirtualFunctionOffset` · 00455060 · legions 00455cd0 — implemented: cited at crates/ac2-script/src/vm/program.rs:8

### ReferenceTable
- [-] `ReferenceTable::Grow` · 004550b0 · legions 00455d20 — VM garbage collector and timers: Rust ownership replaces the reference-table GC (AGENTS: no GC/reference reuse)
- [-] `ReferenceTable::ScanThreads` · 00455130 · legions 00455d90 — VM garbage collector and timers: Rust ownership replaces the reference-table GC (AGENTS: no GC/reference reuse)
- [-] `ReferenceTable::ScanSingletons` · 004551b0 · legions 00455e10 — VM garbage collector and timers: Rust ownership replaces the reference-table GC (AGENTS: no GC/reference reuse)
- [-] `ReferenceTable::ScanVMData` · 00455210 · legions 00455e70 — VM garbage collector and timers: Rust ownership replaces the reference-table GC (AGENTS: no GC/reference reuse)

### ContextState
- [x] `ContextState::PushDInt` · 00455270 · legions 00455ed0 — implemented: cited at crates/ac2-script/src/vm/context.rs:112
- [-] `ContextState::HaltTimers` · 004552c0 · legions 00455f20 — VM garbage collector and timers: Rust ownership replaces the reference-table GC (AGENTS: no GC/reference reuse)
- [-] `ContextState::ResumeTimers` · 00455340 · legions 00455fa0 — VM garbage collector and timers: Rust ownership replaces the reference-table GC (AGENTS: no GC/reference reuse)

### VM
- [x] `VM::Execute` · 004553f0 · legions 00456050 — implemented: cited at crates/ac2-script/src/vm/mod.rs:330

### Resolver
- [x] `Resolver::GetFunctionOffset` · 00455460 · legions 004560c0 — implemented: cited at crates/ac2-net/src/protocol/gameplay/trade.rs:6

### ReferenceTable
- [-] `ReferenceTable::FinishGarbageCollection` · 004554d0 · legions 00456130 — VM garbage collector and timers: Rust ownership replaces the reference-table GC (AGENTS: no GC/reference reuse)
- [-] `ReferenceTable::ScanProcesses` · 00455600 · legions 00456210 — VM garbage collector and timers: Rust ownership replaces the reference-table GC (AGENTS: no GC/reference reuse)

### VM
- [x] `VM::CallIFun` · 00455740 · legions 00456350 — implemented: cited at crates/ac2-script/src/host/ifun.rs:3

### ReferenceTable
- [x] `ReferenceTable::CallPackageInitializer` · 004557b0 · legions 004563c0 — implemented: cited at crates/ac2-script/src/vm/mod.rs:418
- [-] `ReferenceTable::UpdateRelativeReferences` · 004558a0 · legions 004564b0 — VM garbage collector and timers: Rust ownership replaces the reference-table GC (AGENTS: no GC/reference reuse)
- [-] `ReferenceTable::StartGarbageCollection` · 004558f0 · legions 00456500 — VM garbage collector and timers: Rust ownership replaces the reference-table GC (AGENTS: no GC/reference reuse)

### ggmCWInterface
- [x] `ggmCWInterface::LAHashToXPHash` · 004559c0 · legions 00456590 — ac2-client Client::fellow_xp_hash: an empty LAHash out parameter to IFun 0xB400A8 (CPlayer 0x2C -> FellowshipSystem 0x41), read back as (id, share)
- [x] `ggmCWInterface::RListToCraftSkillAdaptorList` · 00455ae0 · legions 004566b0 — lead: the lists read the player's CraftRegistry (CPlayer 0x23, slot 0x21; skill ARHash slot 5, recipe ARHash slot 6, keyed by DID; InqRecipeList keeps the recipes whose Recipe 0xA7 is the skill). Filled by WM_Craft 0x19 (the registry: CPlayer 0xC3 -> CPlayer 0x1B), 0x1B (learn by DID: CraftSystem 0x1B -> CraftRegistry 0x5 makes a RecipeRecord), 0x27/0x34 (CraftSkillRecord set), 0x1A/0x37 (known RecipeRecord replaced, else 0x9C51). craft_script_7160 asserts the lists.
- [~] `ggmCWInterface::RListToIngredientAdaptorList` · 00455b30 · legions 00456700 — net: bound or read generically (ifun.rs read_filled / text_out); no test reads it back
- [x] `ggmCWInterface::RListToRecipeAdaptorList` · 00455b80 · legions 00456750 — lead: the lists read the player's CraftRegistry (CPlayer 0x23, slot 0x21; skill ARHash slot 5, recipe ARHash slot 6, keyed by DID; InqRecipeList keeps the recipes whose Recipe 0xA7 is the skill). Filled by WM_Craft 0x19 (the registry: CPlayer 0xC3 -> CPlayer 0x1B), 0x1B (learn by DID: CraftSystem 0x1B -> CraftRegistry 0x5 makes a RecipeRecord), 0x27/0x34 (CraftSkillRecord set), 0x1A/0x37 (known RecipeRecord replaced, else 0x9C51). craft_script_7160 asserts the lists.
- [-] `ggmCWInterface::ItemListToLList` · 00455bd0 · legions 004567a0 — STL/container internals: a Rust collection replaces it
- [x] `ggmCWInterface::IngredientOrdinalHashToALHash` · 00455c40 · legions 00456810 — net: craft_script_7160 (client call over the script VM, DAT-backed answer or request bytes)

### PList<unsigned___int64>
- [-] `PList<unsigned___int64>::Pack` · 00455fb0 · legions 00456b80 — STL/container internals: a Rust collection replaces it

### PList<InstanceID>
- [-] `PList<InstanceID>::GetPackSize` · 00456000 · legions 00416530 — STL/container internals: a Rust collection replaces it

### PList<unsigned___int64>
- [-] `PList<unsigned___int64>::UnPack` · 00456020 · legions 00456bd0 — STL/container internals: a Rust collection replaces it

### ReferenceTable
- [x] `ReferenceTable::AddReference` · 00456190 · legions 00456d40 — ac2-script vm::refs (reserved bit on every new entry)
- [-] `ReferenceTable::UpdateReferences` · 00456270 · legions 00456e20 — serialization / reference-tracking bookkeeping: Rust codecs and owned values replace it

### ggmCWInterface
- [x] `ggmCWInterface::XPHashToLAHash` · 00456330 · legions 00456ee0 — ac2-client Client::fellow_xp_hash: an empty LAHash out parameter to IFun 0xB400A8 (CPlayer 0x2C -> FellowshipSystem 0x41), read back as (id, share)
- [-] `ggmCWInterface::RListToEffectUINodeList` · 00456440 · legions 00456ff0 — STL/container internals: a Rust collection replaces it

### BaseList<unsigned___int64,60>
- [-] `BaseList<unsigned___int64,60>::~BaseList<unsigned___int64,60>` · 00456490 · legions 00457040 — STL/container internals: a Rust collection replaces it

### BaseList<unsigned_long,102>
- [-] `BaseList<unsigned_long,102>::~BaseList<unsigned_long,102>` · 004564b0 · legions 00457060 — STL/container internals: a Rust collection replaces it

### BaseHash<unsigned_long,unsigned___int64,9>
- [-] `BaseHash<unsigned_long,unsigned___int64,9>::~BaseHash<unsigned_long,unsigned___int64,9>` · 004564d0 · legions 00457080 — STL/container internals: a Rust collection replaces it

### PList<gmActInfo>
- [-] `PList<gmActInfo>::PList<gmActInfo>` · 00456520 · legions 00457120 — STL/container internals: a Rust collection replaces it

### ReferenceTable
- [x] `ReferenceTable::CreateNativeObject` · 00456550 · legions 00457150 — ac2-script vm::refs (reserved bit on every new entry)
- [x] `ReferenceTable::GetSingleton` · 004565c0 · legions 004571c0 — implemented: cited at crates/ac2-script/src/vm/refs.rs:149

### RList
- [-] `RList::RList` · 00456690 · legions 00457290 — STL/container internals: a Rust collection replaces it
- [-] `RList::GetReferences` · 004566d0 — STL/container internals: a Rust collection replaces it
- [-] `RList::UpdateReferences` · 00456710 — STL/container internals: a Rust collection replaces it

### LList
- [-] `LList::LList` · 00456770 · legions 00457370 — STL/container internals: a Rust collection replaces it
- [-] `LList::UnPack` · 004567b0 — STL/container internals: a Rust collection replaces it

### ggmCWInterface
- [-] `ggmCWInterface::CraftSkillAdaptorListToRList` · 004567c0 · legions 004573c0 — STL/container internals: a Rust collection replaces it
- [-] `ggmCWInterface::IngredientAdaptorListToRList` · 00456860 · legions 00457460 — STL/container internals: a Rust collection replaces it
- [-] `ggmCWInterface::RecipeAdaptorListToRList` · 00456900 · legions 00457500 — STL/container internals: a Rust collection replaces it
- [-] `ggmCWInterface::IgnoredListToRList` · 004569a0 · legions 00525e50 — STL/container internals: a Rust collection replaces it
- [-] `ggmCWInterface::EffectUINodeListToRList` · 00456a40 · legions 004575a0 — STL/container internals: a Rust collection replaces it
- [-] `ggmCWInterface::ShortcutInfoArrayToRArray` · 00456ae0 · legions 00457640 — STL/container internals: a Rust collection replaces it
- [-] `ggmCWInterface::GUNUserStatusListToRList` · 00456b80 · legions 004576e0 — STL/container internals: a Rust collection replaces it

### PHashTable<unsigned___int64,unsigned_long>
- [-] `PHashTable<unsigned___int64,unsigned_long>::StreamPack` · 00456c20 · legions 00457780 — STL/container internals: a Rust collection replaces it

### (free functions)
- [ ] `STREAMPACK` · 00456e10 · legions 004e95e0 — no call site in the Legions wlib (scan: host::tests::efun_coverage); lower priority

### PHashTable<unsigned_long,unsigned___int64>
- [-] `PHashTable<unsigned_long,unsigned___int64>::StreamPack` · 00456f30 · legions 00457a90 — STL/container internals: a Rust collection replaces it

### ReferenceTable
- [x] `ReferenceTable::UnPackReferenceArray` · 00457120 · legions 00457c80 — implemented: cited at crates/ac2-net/src/protocol/pack.rs:6
- [ ] `ReferenceTable::UnPackReference` · 00457330 · legions 00457e90 — needs port: packed reference decode only; the reference-table insert is missing (ac2-script refs)

### LAHash
- [-] `LAHash::LAHash` · 00457460 · legions 00457fc0 — STL/container internals: a Rust collection replaces it
- [-] `LAHash::StreamPack` · 004574a0 — STL/container internals: a Rust collection replaces it

### ALHash
- [-] `ALHash::ALHash` · 004574d0 · legions 00458030 — STL/container internals: a Rust collection replaces it
- [-] `ALHash::StreamPack` · 00457510 — STL/container internals: a Rust collection replaces it

### ReferenceTable
- [x] `ReferenceTable::FillPackage` · 00457540 · legions 004580a0 — a cache-only miss on the WState no longer fills the singleton with a fresh instance: it stays unfilled and the efun waits for the record (vm/mod.rs fill_package, peek_missed); a fault after a miss waits too (vm/efun.rs finish_efun)
- [-] `ReferenceTable::GetReferencePtr` · 00457610 · legions 00458170 — VM garbage collector and timers: Rust ownership replaces the reference-table GC (AGENTS: no GC/reference reuse)
- [x] `ReferenceTable::GetPackageID` · 004576d0 · legions 00458230 — implemented: cited at crates/ac2-script/src/vm/mod.rs:352

### ggmCWInterface
- [x] `ggmCWInterface::CallIFun_AcceptTrade` · 00457730 · legions 00458290 — ac2-client gameplay.rs Command::AcceptTrade: the IFun on the player's VM script (CPlayer's CWInterface graph packed by stub 0x45 -> 0x35); stand-in only without a VM (empty reference)
- [ ] `ggmCWInterface::CallIFun_AcceptVassal` · 004577c0 · legions 00458320 — needs port: script IFun accept vassal on the client interface; no citation in crates/
- [x] `ggmCWInterface::CallIFun_AddEffectDataToProfileByEID` · 00457850 · legions 004583b0 — ui_examination.rs add_effect_data: IFun 0xB40123 with context, order, id and the profile; the script's own Add* efuns other than AddEffectID stay unbound
- [ ] `ggmCWInterface::CallIFun_BreakAllegiance` · 00457a20 · legions 00458580 — needs port: script IFun break allegiance on the client interface; no citation in crates/
- [~] `ggmCWInterface::CallIFun_CanAutoAttackTarget` · 00457af0 · legions 00458650 — ac2-client scripts_queries.rs can_auto_attack_target queries IFun 0xB4000A (combat_7160). Not used by the cycling keys yet: the script answered no for the double's creature (qualities not traced), so target_select.rs candidates keep the creature-not-fellow rule
- [x] `ggmCWInterface::CallIFun_CanCraftSkillBeRaised` · 00457bd0 · legions 00458730 — net: craft_script_7160 (the VM runs the IFun or the forwarder; the packet or notice is asserted)
- [x] `ggmCWInterface::CallIFun_CanSkillBeRaisedByEnum` · 00457c90 · legions 004587f0 — ac2-client scripts_queries: the IFun's query on the player's script; asserted in skill_script_7160
- [x] `ggmCWInterface::CallIFun_CanSkillBeTrainedByEnum` · 00457d50 · legions 004588b0 — ac2-client scripts_queries: the IFun's query on the player's script; asserted in skill_script_7160
- [~] `ggmCWInterface::CallIFun_CanSkillBeUntrainedByEnum` · 00457e10 · legions 00458970 — query or request on the IFun, asserted by skill_script_7160 (untrain and cancel are answered with the repository, whose slot 5 names the skill being untrained; the double completes no untrain timer)
- [x] `ggmCWInterface::CallIFun_CheckIngredient` · 00457ed0 · legions 00458a30 — net: craft_script_7160 (the VM runs the IFun or the forwarder; the packet or notice is asserted)
- [x] `ggmCWInterface::CallIFun_ClearResurrectionRequest` · 00458090 · legions 00458bf0 — implemented: cited at crates/ac2-client/src/scripts_queries.rs:27
- [~] `ggmCWInterface::CallIFun_CloseTradeNegotiations` · 00458120 · legions 00458c80 — IFun body runs on the script VM; its WM_Trade stub id (0xB400B8..BD -> 0x8215_xxxx) traced from the Legions WLib (CWInterface IFun -> stub) and ported in ac2-net gameplay/request.rs
- [~] `ggmCWInterface::CallIFun_DeclineTrade` · 004581b0 · legions 00458d10 — IFun body runs on the script VM; its WM_Trade stub id (0xB400B8..BD -> 0x8215_xxxx) traced from the Legions WLib (CWInterface IFun -> stub) and ported in ac2-net gameplay/request.rs
- [ ] `ggmCWInterface::CallIFun_DeclineVassal` · 00458240 · legions 00458da0 — needs port: script IFun decline vassal on the client interface; no citation in crates/
- [x] `ggmCWInterface::CallIFun_Dismount` · 004582f0 · legions 00458f40 — ac2-client target_select.rs run (SelectionAction::StopAttacking -> Command::StopAttacking) and gameplay.rs own_ifun -> IFun 0xB40104 on the VM; toolbar.rs is_dismount_press -> hud_player.rs Command::Dismount -> IFun 0xB400E5; the VM's WM_Combat 0x25 / WM_Player 0x33 calls (ac2-net calls.rs); tests combat_7160 stop_attacking_runs_on_the_vm_for_the_target, dismount_runs_on_the_vm; the double answers the targeted stop
- [~] `ggmCWInterface::CallIFun_DisplaySaleReminders` · 00458380 · legions 00458fd0 — Command::DisplaySaleReminders sends IFun 0xB4013D (scripts.rs); the stub sends nothing with a shopper open, and the sale reminder notice (gmToolbarUI::RecvNotice_ShowSaleReminder) is not shown
- [ ] `ggmCWInterface::CallIFun_DoAssistedTell` · 00458410 · legions 00459060 — needs port: script IFun do assisted tell on the client interface; no citation in crates/
- [x] `ggmCWInterface::CallIFun_DoEmote` · 00458500 · legions 00459150 — ac2-client friends.rs do_emote: the status's emote is the IFun with the emote StringInfo (table 0x4100001C); gStringUtils IsWordInString efun in ac2-script glue/string_utils.rs (friends_script_7160)
- [ ] `ggmCWInterface::CallIFun_DoFellowshipCreate` · 004585e0 · legions 00459230 — needs port: script IFun do fellowship create on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_DoFellowshipDisband` · 00458710 · legions 00459360 — needs port: script IFun do fellowship disband on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_DoFellowshipDismiss` · 004587a0 · legions 004593f0 — needs port: script IFun do fellowship dismiss on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_DoFellowshipLeave` · 00458870 · legions 004594c0 — needs port: script IFun do fellowship leave on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_DoFellowshipRecruit` · 00458900 · legions 00459550 — needs port: script IFun do fellowship recruit on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_DoFellowshipRecruitAccept` · 004589d0 · legions 00459620 — needs port: script IFun do fellowship recruit accept on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_DoFellowshipRecruitDecline` · 00458a60 · legions 004596b0 — needs port: script IFun do fellowship recruit decline on the client interface; no citation in crates/
- [~] `ggmCWInterface::CallIFun_DoIHaveItem` · 00458b10 · legions 00459760 — answered by the player's script (inventory_queries.rs over the WM_Inventory messages); the UI unions the script's list with the frame mirror, as a move into a side pack is not yet reflected by the script (inventory_items_7160 Known Gap)
- [x] `ggmCWInterface::CallIFun_DoUIDamage` · 00458bf0 · legions 00459840 — implemented: cited at crates/ac2-client/src/scripts_queries.rs:19
- [x] `ggmCWInterface::CallIFun_DoesIngredientAutoPopulate` · 00458dd0 · legions 00459a20 — net: craft_script_7160 (the VM runs the IFun or the forwarder; the packet or notice is asserted)
- [~] `ggmCWInterface::CallIFun_DragFromMoneyBag` · 00458ec0 · legions 00459b10 — ac2-weenie OnlineSession::drag_to_money_bag / drag_from_money_bag (CMoneySystem 0x4 / 0x5 -> WM_Money 0x7 / 0x8, receive_money for 0x6 / 0x9); MoneySystem 0x6's ownership checks and 0x2's room check not mirrored
- [~] `ggmCWInterface::CallIFun_DragToMoneyBag` · 00458fe0 · legions 00459c30 — ac2-weenie OnlineSession::drag_to_money_bag / drag_from_money_bag (CMoneySystem 0x4 / 0x5 -> WM_Money 0x7 / 0x8, receive_money for 0x6 / 0x9); MoneySystem 0x6's ownership checks and 0x2's room check not mirrored
- [~] `ggmCWInterface::CallIFun_EquipItem` · 00459140 · legions 00459d90 — ac2-weenie session/inventory.rs equip_item: InvEquipDesc.equip byte-exact (inventory_7160); client EquipItem (inventory_items_7160). Script checks not mirrored
- [ ] `ggmCWInterface::CallIFun_GUNEvent_FriendStart` · 00459240 · legions 00459e90 — needs port: script IFun gunevent friend start on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_GUNEvent_FriendStatus` · 00459340 · legions 00459f90 — needs port: script IFun gunevent friend status on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_GUNEvent_HearFromRoom` · 00459420 · legions 0045a070 — needs port: script IFun gunevent hear from room on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_GUNHandleReauthentication` · 004595a0 · legions 0045a1f0 — needs port: script IFun gunhandle reauthentication on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_GetAcceptAllegianceRequest` · 00459630 · legions 0045a280 — needs port: script IFun get accept allegiance request on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_GetAcceptFellowshipRequest` · 004596d0 · legions 0045a320 — needs port: script IFun get accept fellowship request on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_GetAcceptTradeRequest` · 00459770 · legions 0045a3c0 — needs port: script IFun get accept trade request on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_GetAutoTarget` · 00459810 · legions 0045a460 — needs port: script IFun get auto target on the client interface; no citation in crates/
- [x] `ggmCWInterface::CallIFun_GetAvailableCraftExperience` · 004598b0 · legions 0045a500 — net: craft_script_7160 (the VM runs the IFun or the forwarder; the packet or notice is asserted)
- [ ] `ggmCWInterface::CallIFun_GetAvailableExperience` · 00459970 · legions 0045a5c0 — needs port: script IFun get available experience on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_GetBypassGoldMeltingConfirmationDlg` · 00459a30 · legions 0045a680 — needs port: script IFun get bypass gold melting confirmation dlg on the client interface; no citation in crates/
- [x] `ggmCWInterface::CallIFun_GetConsignmentCharge` · 00459ad0 · legions 0045a720 — ac2-client gameplay.rs Command to the scripts.rs IFun; ac2-net request.rs ScriptRequest; tests/store_requests_7160.rs and consignment_script_7160.rs
- [x] `ggmCWInterface::CallIFun_GetContainerSegmentInfo` · 00459b70 · legions 0045a7c0 — ac2-client Client::contents_info / container_segment_info (Container 0x3D / 0x3F on the VM); the double sends ContainerSegmentDescriptor segments
- [x] `ggmCWInterface::CallIFun_GetContentsInfo` · 00459d30 · legions 0045a980 — ac2-client Client::contents_info / container_segment_info (Container 0x3D / 0x3F on the VM); the double sends ContainerSegmentDescriptor segments
- [ ] `ggmCWInterface::CallIFun_GetDamageTextRangeOther` · 0045a000 · legions 0045ac50 — needs port: script IFun get damage text range other on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_GetDurabilityLevel` · 0045a0a0 · legions 0045acf0 — needs port: script IFun get durability level on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_GetFXAndBehaviorInformationFromWsl` · 0045a140 · legions 0045ad90 — needs port: script IFun get fxand behavior information from wsl on the client interface; no citation in crates/
- [x] `ggmCWInterface::CallIFun_GetFellowXPHash` · 0045a310 · legions 0045af60 — ac2-client Client::fellow_xp_hash: an empty LAHash out parameter to IFun 0xB400A8 (CPlayer 0x2C -> FellowshipSystem 0x41), read back as (id, share)
- [ ] `ggmCWInterface::CallIFun_GetGameplayOptions` · 0045a4b0 · legions 0045b100 — needs port: script IFun get gameplay options on the client interface; no citation in crates/
- [x] `ggmCWInterface::CallIFun_GetHeroSkillCredits` · 0045a620 · legions 0045b270 — ac2-client scripts_queries: the IFun's query on the player's script; asserted in skill_script_7160
- [ ] `ggmCWInterface::CallIFun_GetIconID` · 0045a6c0 · legions 0045b310 — needs port: script IFun get icon id on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_GetLastSelection` · 0045a760 · legions 0045b3b0 — needs port: script IFun get last selection on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_GetLatencyIndicatorGraph` · 0045a820 · legions 0045b470 — needs port: script IFun get latency indicator graph on the client interface; no citation in crates/
- [~] `ggmCWInterface::CallIFun_GetMaxContainerSize` · 0045a8c0 · legions 0045b510 — ac2-script item_queries::ItemQueries::container_capacity; ItemStats capacity default 12, the EffectSystem pass is not applied
- [ ] `ggmCWInterface::CallIFun_GetMaxDurabilityLevel` · 0045a960 · legions 0045b5b0 — needs port: script IFun get max durability level on the client interface; no citation in crates/
- [~] `ggmCWInterface::CallIFun_GetMaxPurchaseQuantity` · 0045aa00 · legions 0045b650 — IFun ids 0xB40137/31/2C/30/38/2D (RequestSell is 0xB40138 in admin and Legions; the earlier 0xB4013E was wrong) and the push order (ac2-client scripts.rs; the host pushes the arguments last first, as the glue's declared order). Run on the VM; the stubs' fids and packs traced (WM_Store 0x4D, 0x12, 0x14, 0x4E: the sell's pack is an int, price, quantity, a long item, a long).
- [ ] `ggmCWInterface::CallIFun_GetMaxStackSize` · 0045aae0 · legions 0045b730 — needs port: script IFun get max stack size on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_GetMissileInformationFromWSL` · 0045ab80 · legions 0045b7d0 — needs port: script IFun get missile information from wsl on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_GetPotentiallyPluralizedName` · 0045adb0 · legions 0045ba00 — needs port: script IFun get potentially pluralized name on the client interface; no citation in crates/
- [x] `ggmCWInterface::CallIFun_GetPreferredInventoryLocation` · 0045ae80 · legions 0045bad0 — ac2-weenie ItemStats::preferred_location (Item 0x26: int 282), valid_locations (275), precluded_locations (283)
- [x] `ggmCWInterface::CallIFun_GetQuantity` · 0045af20 · legions 0045bb70 — ac2-script item_queries::ItemQueries::item_display (IFun 0xB40064 on the item's process)
- [ ] `ggmCWInterface::CallIFun_GetRadarMask` · 0045afc0 · legions 0045bc10 — needs port: script IFun get radar mask on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_GetSelection` · 0045b060 · legions 0045bcb0 — needs port: script IFun get selection on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_GetShortcutHeight` · 0045b120 · legions 0045bd70 — needs port: script IFun get shortcut height on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_GetShortcutInstaScroll` · 0045b1c0 · legions 0045be10 — needs port: script IFun get shortcut insta scroll on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_GetShortcutLocked` · 0045b260 · legions 0045beb0 — needs port: script IFun get shortcut locked on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_GetShowDamageText` · 0045b300 · legions 0045bf50 — needs port: script IFun get show damage text on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_GetShowDamageTextOther` · 0045b3a0 · legions 0045bff0 — needs port: script IFun get show damage text other on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_GetShowFellowsOnMap` · 0045b440 · legions 0045c090 — needs port: script IFun get show fellows on map on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_GetShowObjectNames` · 0045b4e0 · legions 0045c130 — needs port: script IFun get show object names on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_GetShowPKDamage` · 0045b580 · legions 0045c1d0 — needs port: script IFun get show pkdamage on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_GetShowPetUI` · 0045b620 · legions 0045c270 — needs port: script IFun get show pet ui on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_GetShowRadarCoords` · 0045b6c0 · legions 0045c310 — needs port: script IFun get show radar coords on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_GetShowShortcutNumbers` · 0045b760 · legions 0045c3b0 — needs port: script IFun get show shortcut numbers on the client interface; no citation in crates/
- [x] `ggmCWInterface::CallIFun_GetSkillCostToTrainByEnum` · 0045b800 · legions 0045c450 — ac2-client scripts_queries: the IFun's query on the player's script; asserted in skill_script_7160
- [x] `ggmCWInterface::CallIFun_GetSkillCredits` · 0045b8c0 · legions 0045c510 — ac2-client scripts_queries: the IFun's query on the player's script; asserted in skill_script_7160
- [x] `ggmCWInterface::CallIFun_GetSkillExperienceNeededToLevelByEnum` · 0045b960 · legions 0045c5b0 — ac2-client scripts_queries: the IFun's query on the player's script; asserted in skill_script_7160
- [x] `ggmCWInterface::CallIFun_GetSkillXPByEnum` · 0045ba40 · legions 0045c690 — ac2-client scripts_queries: the IFun's query on the player's script; asserted in skill_script_7160
- [ ] `ggmCWInterface::CallIFun_GetStackable` · 0045bb20 · legions 0045c770 — needs port: script IFun get stackable on the client interface; no citation in crates/
- [~] `ggmCWInterface::CallIFun_GetWhichShortcutSet` · 0045bbc0 · legions 0045c810 — read from the same options profile (shortcut_settings.which_set), not IFun 0xB40072 itself
- [~] `ggmCWInterface::CallIFun_HandlePlayScenesDone` · 0045bc60 · legions 0045c8b0 — ac2-client client_story.rs: story slot routed; drawing, audio and camera FX are known gaps
- [x] `ggmCWInterface::CallIFun_HandleWeenieEmote` · 0045bd60 · legions 0045c9b0 — gmCWInterface::HandleWeenieEmote caller is CPhysicsObj::RecvEvt_DoBehavior (admin 004ec680, weenie slot +0x78): ac2-client scripts_io.rs answer_weenie_emote calls IFun 0xB400AA via ScriptHost::call_ifun; ac2-client client_objects.rs apply_remote gates the behaviour on it
- [ ] `ggmCWInterface::CallIFun_HasDurability` · 0045be20 · legions 0045ca70 — needs port: script IFun has durability on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_HasSomethingSelected` · 0045bec0 · legions 0045cb10 — needs port: script IFun has something selected on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_HearTell` · 0045bf60 · legions 0045cbb0 — needs port: script IFun hear tell on the client interface; no citation in crates/
- [x] `ggmCWInterface::CallIFun_InqActInfoList` · 0045c0f0 · legions 0045cd40 — ac2-script glue/act_info.rs (ggmActInfo, ggmActInfoList efuns); ac2-net native/act.rs (gmActInfo StreamPack); client ui_quests.rs inq_act_info_list (InqActInfoList IFun 0xB4008F)
- [ ] `ggmCWInterface::CallIFun_InqAlias` · 0045c2a0 · legions 0045cef0 — needs port: script IFun inq alias on the client interface; no citation in crates/
- [~] `ggmCWInterface::CallIFun_InqAllInventoryIIDList` · 0045c4a0 · legions 0045d0f0 — answered by the player's script (inventory_queries.rs over the WM_Inventory messages); the UI unions the script's list with the frame mirror, as a move into a side pack is not yet reflected by the script (inventory_items_7160 Known Gap)
- [x] `ggmCWInterface::CallIFun_InqCraftSkillAdaptor` · 0045c630 · legions 0045d280 — net: client call over the script VM (craft_script_7160); DAT-backed answers and the request bytes tested
- [x] `ggmCWInterface::CallIFun_InqCraftSkillCategoryName` · 0045c7c0 · legions 0045d410 — net: craft_script_7160 (client call over the script VM, DAT-backed answer or request bytes)
- [x] `ggmCWInterface::CallIFun_InqCraftSkillLevel` · 0045c950 · legions 0045d5a0 — net: craft_script_7160 (client call over the script VM, DAT-backed answer or request bytes)
- [x] `ggmCWInterface::CallIFun_InqCraftSkillList` · 0045cb20 · legions 0045d770 — lead: the lists read the player's CraftRegistry (CPlayer 0x23, slot 0x21; skill ARHash slot 5, recipe ARHash slot 6, keyed by DID; InqRecipeList keeps the recipes whose Recipe 0xA7 is the skill). Filled by WM_Craft 0x19 (the registry: CPlayer 0xC3 -> CPlayer 0x1B), 0x1B (learn by DID: CraftSystem 0x1B -> CraftRegistry 0x5 makes a RecipeRecord), 0x27/0x34 (CraftSkillRecord set), 0x1A/0x37 (known RecipeRecord replaced, else 0x9C51). craft_script_7160 asserts the lists.
- [x] `ggmCWInterface::CallIFun_InqEffectList` · 0045ccc0 · legions 0045d910 — ac2-client Client::effect_list (IFun 0xB40062, CPlayer 0x52 -> CEffectSystem 0x7); the VM's gEffectUINode efuns fill it, tested with the login registry and WM_Effect 0x17/0x16
- [ ] `ggmCWInterface::CallIFun_InqNextReplyHistoryItem` · 0045ce60 · legions 0045dab0 — needs port: script IFun inq next reply history item on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_InqNextRetellHistoryItem` · 0045d000 · legions 0045dc50 — needs port: script IFun inq next retell history item on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_InqPrevReplyHistoryItem` · 0045d1a0 · legions 0045ddf0 — needs port: script IFun inq prev reply history item on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_InqPrevRetellHistoryItem` · 0045d340 · legions 0045df90 — needs port: script IFun inq prev retell history item on the client interface; no citation in crates/
- [~] `ggmCWInterface::CallIFun_InqQuestInfoList` · 0045d4e0 · legions 0045e130 — client ui_quests.rs Client::inq_quest_list (IFun 0xB4008E, list as IFunArg::Filled: the out list is approximated, not run against a live login)
- [~] `ggmCWInterface::CallIFun_InqRadarLook` · 0045d660 · legions 0045e2b0 — ac2-client client_radar.rs + ac2-script radar.rs: looks asked on each object's script, the ranges from RadarControl; the creature, NPC and player scripts answer no look in the double (no reads, retval 0); no drawing (pixel mapping, tooltip) in ac2-ui
- [x] `ggmCWInterface::CallIFun_InqRecipeAdaptor` · 0045d870 · legions 0045e4c0 — net: client call over the script VM (craft_script_7160); DAT-backed answers and the request bytes tested
- [x] `ggmCWInterface::CallIFun_InqRecipeCost` · 0045da10 · legions 0045e660 — net: craft_script_7160 (client call over the script VM, DAT-backed answer or request bytes)
- [x] `ggmCWInterface::CallIFun_InqRecipeDifficulty` · 0045db70 · legions 0045e7c0 — net: craft_script_7160 (the VM runs the IFun or the forwarder; the packet or notice is asserted)
- [x] `ggmCWInterface::CallIFun_InqRecipeIngredients` · 0045dd60 · legions 0045e9b0 — net: craft_script_7160 (the VM runs the IFun or the forwarder; the packet or notice is asserted)
- [x] `ggmCWInterface::CallIFun_InqRecipeList` · 0045dfc0 · legions 0045ec10 — lead: the lists read the player's CraftRegistry (CPlayer 0x23, slot 0x21; skill ARHash slot 5, recipe ARHash slot 6, keyed by DID; InqRecipeList keeps the recipes whose Recipe 0xA7 is the skill). Filled by WM_Craft 0x19 (the registry: CPlayer 0xC3 -> CPlayer 0x1B), 0x1B (learn by DID: CraftSystem 0x1B -> CraftRegistry 0x5 makes a RecipeRecord), 0x27/0x34 (CraftSkillRecord set), 0x1A/0x37 (known RecipeRecord replaced, else 0x9C51). craft_script_7160 asserts the lists.
- [ ] `ggmCWInterface::CallIFun_InqRootPanelFromRace` · 0045e180 · legions 0045edd0 — needs port: script IFun inq root panel from race on the client interface; no citation in crates/
- [~] `ggmCWInterface::CallIFun_InqShortcutArray` · 0045e2c0 · legions 0045ef10 — read from GetGameplayOptions' profile (scripts_queries.rs shortcut_array, progress_7160), not IFun 0xB40070 itself
- [x] `ggmCWInterface::CallIFun_InqShortcutInfo` · 0045e450 · legions 0045f0a0 — client shortcut_info (scripts_queries.rs, IFun 0xB4006D); icon in ui_sources ShortcutSource::icon; shortcut_drag_7160 and view tests
- [~] `ggmCWInterface::CallIFun_InqSimpleInvByLocTable` · 0045e660 · legions 0045f2b0 — answered by the player's script (inventory_queries.rs over the WM_Inventory messages); the UI unions the script's list with the frame mirror, as a move into a side pack is not yet reflected by the script (inventory_items_7160 Known Gap)
- [~] `ggmCWInterface::CallIFun_InqSkillLevelByEnum` · 0045e7f0 · legions 0045f440 — query or request on the IFun, asserted by skill_script_7160 (untrain and cancel are answered with the repository, whose slot 5 names the skill being untrained; the double completes no untrain timer)
- [~] `ggmCWInterface::CallIFun_InqSkillTimeLeft` · 0045e9c0 · legions 0045f610 — query or request on the IFun, asserted by skill_script_7160 (untrain and cancel are answered with the repository, whose slot 5 names the skill being untrained; the double completes no untrain timer)
- [x] `ggmCWInterface::CallIFun_InqUISaveLocations` · 0045eb90 · legions 0045f7e0 — ac2-client client_locations.rs: ReloadOptions and the refresh apply the profile's locations (hud_player.rs); a moved window sends SaveWindowLocations: IFun 0xB4007B then 0xB400DB (WM_Player 0x32); AddLocation bucket order in ac2-script native/options.rs; tests window_locations_7160.rs
- [x] `ggmCWInterface::CallIFun_IsCraftSkillRaiseable` · 0045ed00 · legions 0045f950 — net: craft_script_7160 (the VM runs the IFun or the forwarder; the packet or notice is asserted)
- [ ] `ggmCWInterface::CallIFun_IsFellowshipMember` · 0045edc0 · legions 0045fa10 — needs port: script IFun is fellowship member on the client interface; no citation in crates/
- [~] `ggmCWInterface::CallIFun_IsHeroSkillByEnum` · 0045eea0 · legions 0045faf0 — query or request on the IFun, asserted by skill_script_7160 (untrain and cancel are answered with the repository, whose slot 5 names the skill being untrained; the double completes no untrain timer)
- [x] `ggmCWInterface::CallIFun_IsOnInventory` · 0045ef60 · legions 0045fbb0 — the worn test: true for an item the player has equipped, false for pack, side-pack and unknown items (inventory_items_7160); the money bag drop callback is admin-only
- [x] `ggmCWInterface::CallIFun_IsSkillBeingUntrainedByEnum` · 0045f040 · legions 0045fc90 — ac2-client scripts_queries: the IFun's query on the player's script; asserted in skill_script_7160
- [x] `ggmCWInterface::CallIFun_IsSkillHarmful` · 0045f100 · legions 0045fd50 — ac2-client scripts_queries: the IFun's query on the player's script; asserted in skill_script_7160
- [x] `ggmCWInterface::CallIFun_IsSkillTrainedByEnum` · 0045f1c0 · legions 0045fe10 — ac2-client scripts_queries: the IFun's query on the player's script; asserted in skill_script_7160
- [~] `ggmCWInterface::CallIFun_IsSkillUsableBasedOnInventoryByEnum` · 0045f280 · legions 0045fed0 — query or request on the IFun, asserted by skill_script_7160 (untrain and cancel are answered with the repository, whose slot 5 names the skill being untrained; the double completes no untrain timer)
- [~] `ggmCWInterface::CallIFun_LootAllFromContainer` · 0045f340 · legions 0045ff90 — Loot All runs on the player's script: IFun 0xB400E8 sends WM_Inventory 0x68 after the wlib counts the corpse's lists (corpse_script_7160)
- [~] `ggmCWInterface::CallIFun_MoveItem` · 0045f410 · legions 00460060 — ac2-weenie session/inventory.rs move_item: InvMoveDesc / ReorganizeContents byte-exact (inventory_7160, chest_7160); client Command::MoveItem (item_drag_7160, inventory_items_7160). Script checks and optimistic updates not mirrored
- [~] `ggmCWInterface::CallIFun_OfferItemForTrade` · 0045f5e0 · legions 00460230 — IFun body runs on the script VM; its WM_Trade stub id (0xB400B8..BD -> 0x8215_xxxx) traced from the Legions WLib (CWInterface IFun -> stub) and ported in ac2-net gameplay/request.rs
- [x] `ggmCWInterface::CallIFun_OpenRecipe` · 0045f6e0 · legions 00460330 — net: client call over the script VM (craft_script_7160); DAT-backed answers and the request bytes tested
- [~] `ggmCWInterface::CallIFun_OpenTradeNegotiations` · 0045f790 · legions 004603e0 — IFun body runs on the script VM; its WM_Trade stub id (0xB400B8..BD -> 0x8215_xxxx) traced from the Legions WLib (CWInterface IFun -> stub) and ported in ac2-net gameplay/request.rs
- [x] `ggmCWInterface::Resume_PopupYesNoDialog` · 0045f860 · legions 004604b0 — ac2-client hud_game_dialogs.rs: questions queued in HudView::popup_backlog until the HUD dialogs exist (tests/popup_hud_7160.rs)
- [~] `ggmCWInterface::CallIFun_PurchaseConfirmed` · 0045f8f0 · legions 00460540 — IFun ids 0xB40137/31/2C/30/38/2D (RequestSell is 0xB40138 in admin and Legions; the earlier 0xB4013E was wrong) and the push order (ac2-client scripts.rs; the host pushes the arguments last first, as the glue's declared order). Run on the VM; the stubs' fids and packs traced (WM_Store 0x4D, 0x12, 0x14, 0x4E: the sell's pack is an int, price, quantity, a long item, a long).
- [~] `ggmCWInterface::CallIFun_PurchaseItem` · 0045f9a0 · legions 004605f0 — IFun ids 0xB40137/31/2C/30/38/2D (RequestSell is 0xB40138 in admin and Legions; the earlier 0xB4013E was wrong) and the push order (ac2-client scripts.rs; the host pushes the arguments last first, as the glue's declared order). Run on the VM; the stubs' fids and packs traced (WM_Store 0x4D, 0x12, 0x14, 0x4E: the sell's pack is an int, price, quantity, a long item, a long).
- [ ] `ggmCWInterface::CallIFun_QueryAllegianceProfile` · 0045fac0 · legions 00460710 — needs port: script IFun query allegiance profile on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_RegisterCommands` · 0045fb90 · legions 004607e0 — needs port: script IFun register commands on the client interface; no citation in crates/
- [x] `ggmCWInterface::CallIFun_RequestAddIngredient` · 0045fc20 · legions 00460870 — net: client call over the script VM (craft_script_7160); DAT-backed answers and the request bytes tested
- [ ] `ggmCWInterface::CallIFun_RequestAttackStateChange` · 0045fd60 · legions 004609b0 — needs port: script IFun request attack state change on the client interface; no citation in crates/
- [x] `ggmCWInterface::CallIFun_RequestCollect` · 0045fe20 · legions 00460a70 — ac2-client gameplay.rs Command to the scripts.rs IFun; ac2-net request.rs ScriptRequest; tests/store_requests_7160.rs and consignment_script_7160.rs
- [x] `ggmCWInterface::CallIFun_RequestEnterConsignment` · 0045ff00 · legions 00460b50 — ac2-client gameplay.rs Command to the scripts.rs IFun; ac2-net request.rs ScriptRequest; tests/store_requests_7160.rs and consignment_script_7160.rs
- [~] `ggmCWInterface::CallIFun_RequestEnterStore` · 0045ff90 · legions 00460be0 — IFun ids 0xB40137/31/2C/30/38/2D (RequestSell is 0xB40138 in admin and Legions; the earlier 0xB4013E was wrong) and the push order (ac2-client scripts.rs; the host pushes the arguments last first, as the glue's declared order). Run on the VM; the stubs' fids and packs traced (WM_Store 0x4D, 0x12, 0x14, 0x4E: the sell's pack is an int, price, quantity, a long item, a long).
- [x] `ggmCWInterface::CallIFun_RequestExecuteCraft` · 00460040 · legions 00460c90 — net: client call over the script VM (craft_script_7160); DAT-backed answers and the request bytes tested
- [x] `ggmCWInterface::CallIFun_RequestLeaveCatalog` · 00460190 · legions 00460de0 — ac2-client gameplay.rs Command to the scripts.rs IFun; ac2-net request.rs ScriptRequest; tests/store_requests_7160.rs and consignment_script_7160.rs
- [x] `ggmCWInterface::CallIFun_RequestLeaveConsignment` · 00460220 · legions 00460e70 — ac2-client gameplay.rs Command to the scripts.rs IFun; ac2-net request.rs ScriptRequest; tests/store_requests_7160.rs and consignment_script_7160.rs
- [~] `ggmCWInterface::CallIFun_RequestLeaveStore` · 004602b0 · legions 00460f00 — IFun ids 0xB40137/31/2C/30/38/2D (RequestSell is 0xB40138 in admin and Legions; the earlier 0xB4013E was wrong) and the push order (ac2-client scripts.rs; the host pushes the arguments last first, as the glue's declared order). Run on the VM; the stubs' fids and packs traced (WM_Store 0x4D, 0x12, 0x14, 0x4E: the sell's pack is an int, price, quantity, a long item, a long).
- [x] `ggmCWInterface::CallIFun_RequestNextSales` · 00460340 · legions 00460f90 — ac2-client gameplay.rs Command to the scripts.rs IFun; ac2-net request.rs ScriptRequest; tests/store_requests_7160.rs and consignment_script_7160.rs
- [x] `ggmCWInterface::CallIFun_RequestPetAttack` · 004603d0 · legions 00461020 — ac2-client scripts.rs IFUN_REQUEST_PET_* (0xB40144-0xB40149) on the VM; the argument order follows the glue's pushes; tests/pets_7160.rs
- [x] `ggmCWInterface::CallIFun_RequestPetBuff` · 004604a0 · legions 004610f0 — ac2-client scripts.rs IFUN_REQUEST_PET_* (0xB40144-0xB40149) on the VM; the argument order follows the glue's pushes; tests/pets_7160.rs
- [x] `ggmCWInterface::CallIFun_RequestPetHeal` · 00460530 · legions 00461180 — ac2-client scripts.rs IFUN_REQUEST_PET_* (0xB40144-0xB40149) on the VM; the argument order follows the glue's pushes; tests/pets_7160.rs
- [x] `ggmCWInterface::CallIFun_RequestPetMode` · 004605c0 · legions 00461210 — ac2-client scripts.rs IFUN_REQUEST_PET_* (0xB40144-0xB40149) on the VM; the argument order follows the glue's pushes; tests/pets_7160.rs
- [x] `ggmCWInterface::CallIFun_RequestPetReturn` · 004606c0 · legions 00461310 — ac2-client scripts.rs IFUN_REQUEST_PET_* (0xB40144-0xB40149) on the VM; the argument order follows the glue's pushes; tests/pets_7160.rs
- [x] `ggmCWInterface::CallIFun_RequestPrevSales` · 00460750 · legions 004613a0 — ac2-client gameplay.rs Command to the scripts.rs IFun; ac2-net request.rs ScriptRequest; tests/store_requests_7160.rs and consignment_script_7160.rs
- [x] `ggmCWInterface::CallIFun_RequestRaiseCraftSkill` · 004607e0 · legions 00461430 — net: craft_script_7160 (the VM runs the IFun or the forwarder; the packet or notice is asserted)
- [x] `ggmCWInterface::CallIFun_RequestRaiseSkill` · 00460890 · legions 004614e0 — ac2-client skills.rs: the request's IFun on the player's script (VM stub sends WM_Skill 0x13 / 0x10, bytes asserted in skill_script_7160); the double answers (character.rs answer_skill)
- [x] `ggmCWInterface::CallIFun_RequestRestock` · 00460940 · legions 00461590 — ac2-client gameplay.rs Command to the scripts.rs IFun; ac2-net request.rs ScriptRequest; tests/store_requests_7160.rs and consignment_script_7160.rs
- [x] `ggmCWInterface::CallIFun_RequestResurrect` · 00460a60 · legions 004616b0 — ac2-ui death.rs (DeathPanel: the resurrect dialog, its text, its button's respawn); client hud_death.rs wires the notice and the answer; death_panel_7160 and combat_7160 (death, respawn)
- [x] `ggmCWInterface::CallIFun_RequestSales` · 00460af0 · legions 00461740 — ac2-client gameplay.rs Command to the scripts.rs IFun; ac2-net request.rs ScriptRequest; tests/store_requests_7160.rs and consignment_script_7160.rs
- [x] `ggmCWInterface::CallIFun_RequestSell` · 00460ba0 · legions 004617f0 — The sell body (Legions 0x4617F0 IFun; stub WM_Store 0x4E sends WM_Store 0x43) runs on the VM: the sell's packed values are an int (the catalog's one store, the body's head of StoreGroup 11), the price, the quantity, the item (long), the storekeeper (long); sell_inside_a_consignment_sends_the_request_sell_bytes (consignment_script_7160) asserts those bytes. Needs the catalog open and the consignment entered; no catalog refuses (sell_without_a_catalog_is_refused_by_the_script).
- [x] `ggmCWInterface::CallIFun_RequestSetPetName` · 00460cc0 · legions 00461910 — ac2-client scripts.rs IFUN_REQUEST_PET_* (0xB40144-0xB40149) on the VM; the argument order follows the glue's pushes; tests/pets_7160.rs
- [x] `ggmCWInterface::CallIFun_RequestTrainSkill` · 00460df0 · legions 00461a40 — ac2-client skills.rs: the request's IFun on the player's script (VM stub sends WM_Skill 0x13 / 0x10, bytes asserted in skill_script_7160); the double answers (character.rs answer_skill)
- [~] `ggmCWInterface::CallIFun_RequestUntrainSkill` · 00460ea0 · legions 00461af0 — query or request on the IFun, asserted by skill_script_7160 (untrain and cancel are answered with the repository, whose slot 5 names the skill being untrained; the double completes no untrain timer)
- [~] `ggmCWInterface::CallIFun_RequestUntrainSkillCancel` · 00460f50 · legions 00461ba0 — query or request on the IFun, asserted by skill_script_7160 (untrain and cancel are answered with the repository, whose slot 5 names the skill being untrained; the double completes no untrain timer)
- [ ] `ggmCWInterface::CallIFun_ResetReplyHistoryIndex` · 00460fe0 · legions 00461c30 — needs port: script IFun reset reply history index on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_ResetRetellHistoryIndex` · 00461070 · legions 00461cc0 — needs port: script IFun reset retell history index on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_Result_CallCreate` · 00461100 · legions 00461d50 — needs port: script IFun result call create on the client interface; no citation in crates/
- [x] `ggmCWInterface::CallIFun_Result_FriendAdd` · 004612c0 · legions 00461f10 — net: friends_script_7160 (the login's GUN requests are the VM's exact values; Result_FriendAdd/Delete IFuns (ids 0xB40040/41, match Legions) raise FriendAdded/FriendRemoved with the name)
- [x] `ggmCWInterface::CallIFun_Result_FriendDelete` · 004613d0 · legions 00462020 — net: friends_script_7160 (the login's GUN requests are the VM's exact values; Result_FriendAdd/Delete IFuns (ids 0xB40040/41, match Legions) raise FriendAdded/FriendRemoved with the name)
- [-] `ggmCWInterface::CallIFun_Result_GetFriendsList` · 004614e0 — admin 7148 only: Legions 7160 has no Result_GetFriendsList wrapper and no IFun 0xB40042; the list arrives in GUNEvent_FriendStart
- [ ] `ggmCWInterface::CallIFun_Result_GetIgnoredList` · 00461620 — needs port: script IFun result get ignored list on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_Result_IgnoredAdd` · 00461760 · legions 004623b0 — needs port: script IFun result ignored add on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_Result_IgnoredDelete` · 00461870 · legions 004624c0 — needs port: script IFun result ignored delete on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_Result_SendToRoom` · 00461980 · legions 004625d0 — needs port: script IFun result send to room on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_Result_SendToUser` · 00461ab0 · legions 00462700 — needs port: script IFun result send to user on the client interface; no citation in crates/
- [~] `ggmCWInterface::CallIFun_RevokeItemFromTrade` · 00461c10 · legions 00462860 — IFun body runs on the script VM; its WM_Trade stub id (0xB400B8..BD -> 0x8215_xxxx) traced from the Legions WLib (CWInterface IFun -> stub) and ported in ac2-net gameplay/request.rs
- [x] `ggmCWInterface::CallIFun_SetAcceptAllegianceRequest` · 00461ce0 · legions 00462930 — ac2-client scripts.rs: the IFun ids of these glue calls (the same ids in Legions 7160), run on the VM by Command::SetGameplayOption / UploadGameOptions
- [x] `ggmCWInterface::CallIFun_SetAcceptFellowshipRequest` · 00461d90 · legions 004629e0 — ac2-client scripts.rs: the IFun ids of these glue calls (the same ids in Legions 7160), run on the VM by Command::SetGameplayOption / UploadGameOptions
- [x] `ggmCWInterface::CallIFun_SetAcceptTradeRequest` · 00461e40 · legions 00462a90 — ac2-client scripts.rs: the IFun ids of these glue calls (the same ids in Legions 7160), run on the VM by Command::SetGameplayOption / UploadGameOptions
- [x] `ggmCWInterface::CallIFun_SetAutoTarget` · 00461ef0 · legions 00462b40 — ac2-client scripts.rs: the IFun ids of these glue calls (the same ids in Legions 7160), run on the VM by Command::SetGameplayOption / UploadGameOptions
- [x] `ggmCWInterface::CallIFun_SetBypassGoldMeltingConfirmationDlg` · 00461fa0 · legions 00462bf0 — ac2-client scripts.rs: the IFun ids of these glue calls (the same ids in Legions 7160), run on the VM by Command::SetGameplayOption / UploadGameOptions
- [x] `ggmCWInterface::CallIFun_SetChatFontColor` · 00462050 · legions 00462ca0 — ac2-client: Command::SetChatOption / SetWindowOpacity (IFuns 0xB400D3/D4/DD/83, DF; SetWindowOpacity then UploadUISettings), UploadChatOptions (IFun 0xB400D5, WM_Player 0x2E asserted in chat_options_7160), InqGameplayOptions = GetGameplayOptions IFun 0xB400DE (scripts_queries gameplay_options); ac2-ui chat_window.rs opacity slider, apply_opacity, reload (window_locations_tests.rs, window_locations_7160.rs a_window_opacity...)
- [x] `ggmCWInterface::CallIFun_SetChatFontSize` · 00462140 · legions 00462d90 — ac2-client: Command::SetChatOption / SetWindowOpacity (IFuns 0xB400D3/D4/DD/83, DF; SetWindowOpacity then UploadUISettings), UploadChatOptions (IFun 0xB400D5, WM_Player 0x2E asserted in chat_options_7160), InqGameplayOptions = GetGameplayOptions IFun 0xB400DE (scripts_queries gameplay_options); ac2-ui chat_window.rs opacity slider, apply_opacity, reload (window_locations_tests.rs, window_locations_7160.rs a_window_opacity...)
- [x] `ggmCWInterface::CallIFun_SetChatPopupFlag` · 00462230 · legions 00462e80 — ac2-client: Command::SetChatOption / SetWindowOpacity (IFuns 0xB400D3/D4/DD/83, DF; SetWindowOpacity then UploadUISettings), UploadChatOptions (IFun 0xB400D5, WM_Player 0x2E asserted in chat_options_7160), InqGameplayOptions = GetGameplayOptions IFun 0xB400DE (scripts_queries gameplay_options); ac2-ui chat_window.rs opacity slider, apply_opacity, reload (window_locations_tests.rs, window_locations_7160.rs a_window_opacity...)
- [x] `ggmCWInterface::CallIFun_SetChatWindowFilter` · 00462320 · legions 00462f70 — ac2-client: Command::SetChatOption / SetWindowOpacity (IFuns 0xB400D3/D4/DD/83, DF; SetWindowOpacity then UploadUISettings), UploadChatOptions (IFun 0xB400D5, WM_Player 0x2E asserted in chat_options_7160), InqGameplayOptions = GetGameplayOptions IFun 0xB400DE (scripts_queries gameplay_options); ac2-ui chat_window.rs opacity slider, apply_opacity, reload (window_locations_tests.rs, window_locations_7160.rs a_window_opacity...)
- [x] `ggmCWInterface::CallIFun_SetDamageTextRangeOther` · 00462410 · legions 00463060 — ac2-client scripts.rs: the IFun ids of these glue calls (the same ids in Legions 7160), run on the VM by Command::SetGameplayOption / UploadGameOptions
- [ ] `ggmCWInterface::CallIFun_SetExaminationRequest` · 004624c0 · legions 00463110 — needs port: script IFun set examination request on the client interface; no citation in crates/
- [x] `ggmCWInterface::CallIFun_SetLatencyIndicatorGraph` · 004625a0 · legions 004631f0 — ac2-client scripts.rs: the IFun ids of these glue calls (the same ids in Legions 7160), run on the VM by Command::SetGameplayOption / UploadGameOptions
- [x] `ggmCWInterface::CallIFun_SetRadarMask` · 00462650 · legions 004632a0 — ac2-client scripts.rs: the IFun ids of these glue calls (the same ids in Legions 7160), run on the VM by Command::SetGameplayOption / UploadGameOptions
- [x] `ggmCWInterface::CallIFun_SetSelection` · 00462700 · legions 00463350 — implemented: cited at crates/ac2-client/src/scripts.rs:329
- [x] `ggmCWInterface::CallIFun_SetShortcut` · 004627d0 · legions 00463420 — client set_shortcut (scripts_queries.rs, IFun 0xB4006F); shortcut_drag_7160 asserts the SetShortcut bytes (ShortcutInfo native, slot)
- [x] `ggmCWInterface::CallIFun_SetShortcutHeight` · 004628e0 · legions 00463530 — ac2-client scripts.rs: the IFun ids of these glue calls (the same ids in Legions 7160), run on the VM by Command::SetGameplayOption / UploadGameOptions
- [x] `ggmCWInterface::CallIFun_SetShortcutInstaScroll` · 00462990 · legions 004635e0 — ac2-client scripts.rs: the IFun ids of these glue calls (the same ids in Legions 7160), run on the VM by Command::SetGameplayOption / UploadGameOptions
- [x] `ggmCWInterface::CallIFun_SetShortcutLocked` · 00462a40 · legions 00463690 — ac2-client scripts.rs: the IFun ids of these glue calls (the same ids in Legions 7160), run on the VM by Command::SetGameplayOption / UploadGameOptions
- [x] `ggmCWInterface::CallIFun_SetShowDamageText` · 00462af0 · legions 00463740 — ac2-client scripts.rs: the IFun ids of these glue calls (the same ids in Legions 7160), run on the VM by Command::SetGameplayOption / UploadGameOptions
- [x] `ggmCWInterface::CallIFun_SetShowDamageTextOther` · 00462ba0 · legions 004637f0 — ac2-client scripts.rs: the IFun ids of these glue calls (the same ids in Legions 7160), run on the VM by Command::SetGameplayOption / UploadGameOptions
- [x] `ggmCWInterface::CallIFun_SetShowFellowsOnMap` · 00462c50 · legions 004638a0 — ac2-client scripts.rs: the IFun ids of these glue calls (the same ids in Legions 7160), run on the VM by Command::SetGameplayOption / UploadGameOptions
- [x] `ggmCWInterface::CallIFun_SetShowObjectNames` · 00462d00 · legions 00463950 — implemented: cited at crates/ac2-client/src/scripts.rs:212
- [x] `ggmCWInterface::CallIFun_SetShowPKDamage` · 00462db0 · legions 00463a00 — ac2-client scripts.rs: the IFun ids of these glue calls (the same ids in Legions 7160), run on the VM by Command::SetGameplayOption / UploadGameOptions
- [x] `ggmCWInterface::CallIFun_SetShowPetUI` · 00462e60 · legions 00463ab0 — ac2-client scripts.rs: the IFun ids of these glue calls (the same ids in Legions 7160), run on the VM by Command::SetGameplayOption / UploadGameOptions
- [x] `ggmCWInterface::CallIFun_SetShowRadarCoords` · 00462f10 · legions 00463b60 — implemented: cited at crates/ac2-client/src/scripts.rs:210
- [x] `ggmCWInterface::CallIFun_SetShowShortcutNumbers` · 00462fc0 · legions 00463c10 — ac2-client scripts.rs: the IFun ids of these glue calls (the same ids in Legions 7160), run on the VM by Command::SetGameplayOption / UploadGameOptions
- [x] `ggmCWInterface::CallIFun_SetUISaveLocations` · 00463070 · legions 00463cc0 — ac2-client client_locations.rs: ReloadOptions and the refresh apply the profile's locations (hud_player.rs); a moved window sends SaveWindowLocations: IFun 0xB4007B then 0xB400DB (WM_Player 0x32); AddLocation bucket order in ac2-script native/options.rs; tests window_locations_7160.rs
- [ ] `ggmCWInterface::CallIFun_SetWhichShortcutSet` · 00463150 · legions 00463da0 — needs port: script IFun set which shortcut set on the client interface; no citation in crates/
- [x] `ggmCWInterface::CallIFun_SetWindowOpacity` · 00463200 · legions 00463e50 — ac2-client: Command::SetChatOption / SetWindowOpacity (IFuns 0xB400D3/D4/DD/83, DF; SetWindowOpacity then UploadUISettings), UploadChatOptions (IFun 0xB400D5, WM_Player 0x2E asserted in chat_options_7160), InqGameplayOptions = GetGameplayOptions IFun 0xB400DE (scripts_queries gameplay_options); ac2-ui chat_window.rs opacity slider, apply_opacity, reload (window_locations_tests.rs, window_locations_7160.rs a_window_opacity...)
- [ ] `ggmCWInterface::CallIFun_ShouldAutoTarget` · 004632f0 · legions 00463f40 — needs port: script IFun should auto target on the client interface; no citation in crates/
- [~] `ggmCWInterface::CallIFun_SpecialAttackEnded` · 00463390 · legions 00463fe0 — no native caller in the call graph (the VM glue raises it); ac2-client hud_player.rs shows EndSpecialAttack notices; IFun 0xB400C2 not called from native
- [-] `ggmCWInterface::CallIFun_SpecialAttackStarted` · 00463440 · legions 00464090 — no route found: CPlayer 0xD (0x1AA4000D) is called by no wlib function (CALL and CALLSTATIC scans, raw word scan); its only callee is CCombatSystem 0x2, which calls the gCM_Combat start efun; no WM_* package, CAgent forwarder or ECM_* event reaches it. A server InterpCEvent naming 0x9AA4000D would still reach it; no retail server message does
- [ ] `ggmCWInterface::CallIFun_StartExamination` · 004634f0 · legions 00464140 — needs port: script IFun start examination on the client interface; no citation in crates/
- [x] `ggmCWInterface::CallIFun_StopAttacking` · 00463580 · legions 004641f0 — ac2-client target_select.rs run (SelectionAction::StopAttacking -> Command::StopAttacking) and gameplay.rs own_ifun -> IFun 0xB40104 on the VM; toolbar.rs is_dismount_press -> hud_player.rs Command::Dismount -> IFun 0xB400E5; the VM's WM_Combat 0x25 / WM_Player 0x33 calls (ac2-net calls.rs); tests combat_7160 stop_attacking_runs_on_the_vm_for_the_target, dismount_runs_on_the_vm; the double answers the targeted stop
- [x] `ggmCWInterface::CallIFun_StopAttackingIfAutoMoving` · 00463660 · legions 004642d0
- [ ] `ggmCWInterface::CallIFun_StopExamination` · 004636f0 · legions 00464360 — needs port: script IFun stop examination on the client interface; no citation in crates/
- [ ] `ggmCWInterface::CallIFun_SwearAllegiance` · 00463780 · legions 004643f0 — needs port: script IFun swear allegiance on the client interface; no citation in crates/
- [~] `ggmCWInterface::CallIFun_TransmuteAllFromContainer` · 00463810 · legions 00464480 — ac2-weenie session/inventory.rs transmute_all: InvTransmuteAllDesc (client test transmute_all_asks_the_server_to_transmute_the_container). The double clears the container without resending its lists, so the frame refresh is the server's; checks (CInventorySystem 0x1F) not mirrored
- [~] `ggmCWInterface::CallIFun_UnEquipItem` · 004638e0 · legions 00464550 — ac2-weenie session/inventory.rs unequip_item: InvEquipDesc.unequip byte-exact (inventory_7160); client UnequipItem (inventory_items_7160). Script checks not mirrored
- [ ] `ggmCWInterface::CallIFun_UpdateOpen` · 004639e0 · legions 00464650 — needs port: script IFun update open on the client interface; no citation in crates/
- [x] `ggmCWInterface::CallIFun_UploadChatOptions` · 00463ae0 · legions 00464750 — ac2-client: Command::SetChatOption / SetWindowOpacity (IFuns 0xB400D3/D4/DD/83, DF; SetWindowOpacity then UploadUISettings), UploadChatOptions (IFun 0xB400D5, WM_Player 0x2E asserted in chat_options_7160), InqGameplayOptions = GetGameplayOptions IFun 0xB400DE (scripts_queries gameplay_options); ac2-ui chat_window.rs opacity slider, apply_opacity, reload (window_locations_tests.rs, window_locations_7160.rs a_window_opacity...)
- [x] `ggmCWInterface::CallIFun_UploadGameOptions` · 00463b70 · legions 004647e0 — ac2-client scripts.rs: the IFun ids of these glue calls (the same ids in Legions 7160), run on the VM by Command::SetGameplayOption / UploadGameOptions
- [x] `ggmCWInterface::CallIFun_UploadUISettings` · 00463c00 · legions 00464870 — ac2-client client_locations.rs: ReloadOptions and the refresh apply the profile's locations (hud_player.rs); a moved window sends SaveWindowLocations: IFun 0xB4007B then 0xB400DB (WM_Player 0x32); AddLocation bucket order in ac2-script native/options.rs; tests window_locations_7160.rs
- [x] `ggmCWInterface::CallIFun_UseSelection` · 00463c90 · legions 00464900 — ac2-client Command::UseSelection runs IFun 0xB40006 on the player's script when the VM owns it (CPlayer 0x79 -> CUsageSystem 0x9: the targeting pointer, the usage list, then WM_Usage 0x22); butchery_7160 asserts it. The session's use_selection stand-in stays for a client without scripts
- [x] `ggmCWInterface::CallIFun_UseShortcut` · 00463d30 · legions 004649a0 — client use_shortcut (scripts_queries.rs, IFun 0xB40007); skill and text shortcuts run on the VM; vitals_7160 use_shortcut

## `ac2/client/src/weenie/interp/glue/gmcefundispatcher.cpp.c` (1/1)

### gmCEFunDispatcher
- [x] `gmCEFunDispatcher::GetEFunAddress` · 00463df0 · legions 00464a60 — implemented: cited at crates/ac2-script/src/vm/efun.rs:4

## `ac2/client/src/weenie/interp/glue/gmcnativefactory.cpp.c` (2/18)

### gmCNativeFactory
- [-] `gmCNativeFactory::HashNativeObject` · 00465f30 · legions 00466c10 — serialization / reference-tracking bookkeeping: Rust codecs and owned values replace it
- [ ] `gmCNativeFactory::IsEqual` · 00465f60 — needs port: gm cnative factory::is equal; no citation in crates/
- [-] `gmCNativeFactory::GetReferences` · 00465fc0 · legions 00466c70 — serialization / reference-tracking bookkeeping: Rust codecs and owned values replace it
- [-] `gmCNativeFactory::UpdateReferences` · 00465fe0 · legions 00466c90 — serialization / reference-tracking bookkeeping: Rust codecs and owned values replace it
- [-] `gmCNativeFactory::GetNativePackSize` · 00466000 · legions 00466cb0 — serialization / reference-tracking bookkeeping: Rust codecs and owned values replace it
- [x] `gmCNativeFactory::PackNativeObject` · 004661d0 · legions 00466e80 — implemented: cited at crates/ac2-net/src/protocol/native/mod.rs:4
- [ ] `gmCNativeFactory::GetCheckSums` · 004663b0 · legions 00467060 — needs port: gm cnative factory::get check sums; no citation in crates/
- [ ] `gmCNativeFactory::ValidateCheckSums` · 004663d0 · legions 00467080 — needs port: gm cnative factory::validate check sums; no citation in crates/

### PStringBase<char>
- [-] `PStringBase<char>::operator!=` · 00466400 · legions 004670b0 — STL/container internals: a Rust collection replaces it

### gmCNativeFactory
- [x] `gmCNativeFactory::GetNativeTypeName` · 004664a0 · legions 00467150 — implemented: cited at crates/ac2-script/src/native/mod.rs:2

### PStringBase<char>
- [-] `PStringBase<char>::operator+=` · 00466930 · legions 004675e0 — STL/container internals: a Rust collection replaces it

### gmCNativeFactory
- [ ] `gmCNativeFactory::ValidateGlueVersion` · 00466960 · legions 00467610 — needs port: gm cnative factory::validate glue version; no citation in crates/
- [ ] `gmCNativeFactory::NewNativeObject` · 00466c10 · legions 004678c0 — needs port: gm cnative factory::new native object; no citation in crates/

### AllegianceProfileAdaptor
- [-] `AllegianceProfileAdaptor::~AllegianceProfileAdaptor` · 00467040 · legions 00467cf0 — destructor: Rust Drop replaces it

### FellowshipAdaptor
- [-] `FellowshipAdaptor::~FellowshipAdaptor` · 00467100 · legions 00467db0 — destructor: Rust Drop replaces it

### gmCNativeFactory
- [ ] `gmCNativeFactory::UnPackNativeObject` · 00467160 · legions 00467e10 — needs port: gm cnative factory::un pack native object; no citation in crates/
- [-] `gmCNativeFactory::CopyNativeObject` · 00467370 · legions 00468020 — copy / delete of plain native data: Rust clone and Drop replace it
- [-] `gmCNativeFactory::DeleteNativeObject` · 004677e0 · legions 00468490 — copy / delete of plain native data: Rust clone and Drop replace it

## `ac2/shared/src/weenie/allocator/gmwallocator.cpp.c` (0/6)

### gmCWAllocator
- [ ] `gmCWAllocator::Create` · 00453ed0 · legions 00454f70 — needs port: weenie allocator create; no citation in crates/
- [ ] `gmCWAllocator::AllocateWeenieObject_Internal` · 00453f20 · legions 00454fc0 — needs port: weenie allocator allocate weenie object internal; no citation in crates/
- [ ] `gmCWAllocator::AllocateWeenieDesc_Internal` · 00453f60 · legions 00455000 — needs port: weenie desc allocation by type is not cited (gmCWAllocator::AllocateWeenieDesc_Internal)
- [ ] `gmCWAllocator::AllocateWeenieDesc_Internal` · 00453f80 · legions 00455020 — needs port: weenie desc allocation by type is not cited (gmCWAllocator::AllocateWeenieDesc_Internal)
- [ ] `gmCWAllocator::AllocateEFunDispatcher` · 00454010 · legions 00455090 — needs port: weenie allocator allocate efun dispatcher; no citation in crates/
- [ ] `gmCWAllocator::AllocateNativeFactory` · 00454030 · legions 004550b0 — needs port: weenie allocator allocate native factory; no citation in crates/

## `ac2/shared/src/weenie/interface/gmwinterface.cpp.c` (165/249)

### gmCWInterface
- [-] `gmCWInterface::gmCWInterface` · 0044f9a0 · legions 00450ae0 — constructor of plain data: Rust struct init replaces it

### CWPhysicsObject
- [ ] `CWPhysicsObject::SetPhysObj` · 0044f9c0 · legions 00450b00 — needs port: physics object set phys obj; no citation in crates/

### CWRepository
- [ ] `CWRepository::AdoptQualities` · 0044f9d0 · legions 00450b10 — needs port: weenie repository adopt qualities; no citation in crates/

### gmCWInterface
- [ ] `gmCWInterface::InqName` · 0044f9e0 · legions 00450b20 — needs port: interface inq name; no citation in crates/
- [~] `gmCWInterface::GetPotentiallyPluralizedName` · 0044fa00 · legions 00450b40 — ItemStats::display_name (gmEntity 0x10B/0x9F/0xAD); plural IsValid against the DAT tables left to the resolver
- [ ] `gmCWInterface::GetPhysObjID` · 0044fa70 · legions 00450bb0 — needs port: interface get phys obj id; no citation in crates/
- [x] `gmCWInterface::SetSelection` · 0044faa0 · legions 00450be0 — ac2-weenie player_script.rs (PlayerScript: selection, last selection, examination request and panel) and session/gameplay.rs select/examine/start_examination/stop_examination; ac2-client gameplay.rs Command::Select/Examine/StartExamination/StopExamination, target_select.rs snapshot; SetSelection's IFun 0xB40002 runs on the VM (SCRIPT_VM_FUNCTIONS); GetSelection/HasSomethingSelected answered from the mirror, their IFuns are not queried
- [x] `gmCWInterface::GetSelection` · 0044fac0 · legions 00450c00 — ac2-weenie player_script.rs (PlayerScript: selection, last selection, examination request and panel) and session/gameplay.rs select/examine/start_examination/stop_examination; ac2-client gameplay.rs Command::Select/Examine/StartExamination/StopExamination, target_select.rs snapshot; SetSelection's IFun 0xB40002 runs on the VM (SCRIPT_VM_FUNCTIONS); GetSelection/HasSomethingSelected answered from the mirror, their IFuns are not queried
- [x] `gmCWInterface::GetLastSelection` · 0044fb20 · legions 00450c60 — ac2-weenie player_script.rs (PlayerScript: selection, last selection, examination request and panel) and session/gameplay.rs select/examine/start_examination/stop_examination; ac2-client gameplay.rs Command::Select/Examine/StartExamination/StopExamination, target_select.rs snapshot; SetSelection's IFun 0xB40002 runs on the VM (SCRIPT_VM_FUNCTIONS); GetSelection/HasSomethingSelected answered from the mirror, their IFuns are not queried
- [x] `gmCWInterface::ShouldAutoTarget` · 0044fb80 · legions 00450cc0 — ac2-client: the ShouldAutoTarget IFun on the VM before attacking
- [x] `gmCWInterface::UseSelection` · 0044fbb0 · legions 00450cf0 — ac2-client Command::UseSelection runs IFun 0xB40006 on the player's script when the VM owns it (CPlayer 0x79 -> CUsageSystem 0x9: the targeting pointer, the usage list, then WM_Usage 0x22); butchery_7160 asserts it. The session's use_selection stand-in stays for a client without scripts
- [x] `gmCWInterface::UseShortcut` · 0044fbe0 · legions 00450d20 — client use_shortcut (scripts_queries.rs, IFun 0xB40007); skill and text shortcuts run on the VM; vitals_7160 use_shortcut
- [x] `gmCWInterface::StopAttacking` · 0044fc20 · legions 00450d60 — ac2-client target_select.rs run (SelectionAction::StopAttacking -> Command::StopAttacking) and gameplay.rs own_ifun -> IFun 0xB40104 on the VM; toolbar.rs is_dismount_press -> hud_player.rs Command::Dismount -> IFun 0xB400E5; the VM's WM_Combat 0x25 / WM_Player 0x33 calls (ac2-net calls.rs); tests combat_7160 stop_attacking_runs_on_the_vm_for_the_target, dismount_runs_on_the_vm; the double answers the targeted stop
- [x] `gmCWInterface::RequestAttackStateChange` · 0044fc60 · legions 00450da0 — ac2-client Command::SetAttacking: ShouldAutoTarget's Select(0x41000024) notice, then the IFun on the VM (stand-in: OnlineSession::request_attack_state -> WM_Combat 0x16 with the selection / 0x14)
- [~] `gmCWInterface::CanAutoAttackTarget` · 0044fca0 · legions 00450de0 — ac2-client scripts_queries.rs can_auto_attack_target queries IFun 0xB4000A (combat_7160). Not used by the cycling keys yet: the script answered no for the double's creature (qualities not traced), so target_select.rs candidates keep the creature-not-fellow rule
- [x] `gmCWInterface::InqCraftSkillAdaptor` · 0044fce0 · legions 00450e20 — net: craft_script_7160 (client call over the script VM, DAT-backed answer or request bytes)
- [x] `gmCWInterface::InqCraftSkillList` · 0044fd20 · legions 00450e60 — lead: the lists read the player's CraftRegistry (CPlayer 0x23, slot 0x21; skill ARHash slot 5, recipe ARHash slot 6, keyed by DID; InqRecipeList keeps the recipes whose Recipe 0xA7 is the skill). Filled by WM_Craft 0x19 (the registry: CPlayer 0xC3 -> CPlayer 0x1B), 0x1B (learn by DID: CraftSystem 0x1B -> CraftRegistry 0x5 makes a RecipeRecord), 0x27/0x34 (CraftSkillRecord set), 0x1A/0x37 (known RecipeRecord replaced, else 0x9C51). craft_script_7160 asserts the lists.
- [x] `gmCWInterface::InqCraftSkillLevel` · 0044fd60 · legions 00450ea0 — net: craft_script_7160 (client call over the script VM, DAT-backed answer or request bytes)
- [x] `gmCWInterface::InqRecipeAdaptor` · 0044fda0 · legions 00450ee0 — net: craft_script_7160 (client call over the script VM, DAT-backed answer or request bytes)
- [x] `gmCWInterface::InqRecipeList` · 0044fde0 · legions 00450f20 — lead: the lists read the player's CraftRegistry (CPlayer 0x23, slot 0x21; skill ARHash slot 5, recipe ARHash slot 6, keyed by DID; InqRecipeList keeps the recipes whose Recipe 0xA7 is the skill). Filled by WM_Craft 0x19 (the registry: CPlayer 0xC3 -> CPlayer 0x1B), 0x1B (learn by DID: CraftSystem 0x1B -> CraftRegistry 0x5 makes a RecipeRecord), 0x27/0x34 (CraftSkillRecord set), 0x1A/0x37 (known RecipeRecord replaced, else 0x9C51). craft_script_7160 asserts the lists.
- [x] `gmCWInterface::InqRecipeIngredients` · 0044fe20 · legions 00450f60 — net: craft_script_7160 (the VM runs the IFun or the forwarder; the packet or notice is asserted)
- [x] `gmCWInterface::InqRecipeDifficulty` · 0044fe70 · legions 00450fb0 — net: craft_script_7160 (the VM runs the IFun or the forwarder; the packet or notice is asserted)
- [x] `gmCWInterface::DoesIngredientAutoPopulate` · 0044fec0 · legions 00451000 — net: craft_script_7160 (the VM runs the IFun or the forwarder; the packet or notice is asserted)
- [x] `gmCWInterface::RequestAddIngredient` · 0044ff00 · legions 00451040 — net: craft_script_7160 (client call over the script VM, DAT-backed answer or request bytes)
- [x] `gmCWInterface::OpenRecipe` · 0044ff30 · legions 00451070 — net: craft_script_7160 (client call over the script VM, DAT-backed answer or request bytes)
- [x] `gmCWInterface::CheckIngredient` · 0044ff50 · legions 00451090 — net: craft_script_7160 (the VM runs the IFun or the forwarder; the packet or notice is asserted)
- [x] `gmCWInterface::CanCraftSkillBeRaised` · 0044ffa0 · legions 004510e0 — net: craft_script_7160 (the VM runs the IFun or the forwarder; the packet or notice is asserted)
- [x] `gmCWInterface::IsCraftSkillRaiseable` · 0044ffe0 · legions 00451120 — net: craft_script_7160 (the VM runs the IFun or the forwarder; the packet or notice is asserted)
- [x] `gmCWInterface::RequestRaiseCraftSkill` · 00450020 · legions 00451160 — net: craft_script_7160 (the VM runs the IFun or the forwarder; the packet or notice is asserted)
- [x] `gmCWInterface::InqCraftSkillCategoryName` · 00450040 · legions 00451180 — net: craft_script_7160 (client call over the script VM, DAT-backed answer or request bytes)
- [x] `gmCWInterface::RequestExecuteCraft` · 00450080 · legions 004511c0 — net: craft_script_7160 (client call over the script VM, DAT-backed answer or request bytes)
- [x] `gmCWInterface::InqRecipeCost` · 004500b0 · legions 004511f0 — net: craft_script_7160 (client call over the script VM, DAT-backed answer or request bytes)
- [x] `gmCWInterface::GetAvailableCraftExperience` · 004500f0 · legions 00451230 — net: craft_script_7160 (the VM runs the IFun or the forwarder; the packet or notice is asserted)
- [x] `gmCWInterface::DoEmote` · 00450130 · legions 00451270 — ac2-client friends.rs do_emote: the status's emote is the IFun with the emote StringInfo (table 0x4100001C); gStringUtils IsWordInString efun in ac2-script glue/string_utils.rs (friends_script_7160)
- [~] `gmCWInterface::PurchaseItem` · 00450150 · legions 00451290 — Command runs its IFun on the player's script (VM only; no session stand-in). Stub packs and the forwarders traced. The requests need an open shopper (a server ViewStore); ViewStore with a StoreTemplate reference stops at the shop build (store_script_7160 pins the notices). RequestSell's pack is traced (WM_Store 0x4E) but its locals are not.
- [x] `gmCWInterface::RequestNextSales` · 00450180 · legions 004512c0 — ac2-client gameplay.rs Command to the scripts.rs IFun; ac2-net request.rs ScriptRequest; tests/store_requests_7160.rs and consignment_script_7160.rs
- [x] `gmCWInterface::RequestPrevSales` · 004501a0 · legions 004512e0 — ac2-client gameplay.rs Command to the scripts.rs IFun; ac2-net request.rs ScriptRequest; tests/store_requests_7160.rs and consignment_script_7160.rs
- [x] `gmCWInterface::RequestSales` · 004501c0 · legions 00451300 — ac2-client gameplay.rs Command to the scripts.rs IFun; ac2-net request.rs ScriptRequest; tests/store_requests_7160.rs and consignment_script_7160.rs
- [~] `gmCWInterface::PurchaseConfirmed` · 004501e0 · legions 00451320 — Command runs its IFun on the player's script (VM only; no session stand-in). Stub packs and the forwarders traced. The requests need an open shopper (a server ViewStore); ViewStore with a StoreTemplate reference stops at the shop build (store_script_7160 pins the notices). RequestSell's pack is traced (WM_Store 0x4E) but its locals are not.
- [~] `gmCWInterface::GetMaxPurchaseQuantity` · 00450200 · legions 00451340 — Command runs its IFun on the player's script (VM only; no session stand-in). Stub packs and the forwarders traced; the requests need an open shopper (a server ViewStore), so no store request is byte-verified end to end yet (store_script_7160). RequestSell's three ints are not decoded.
- [~] `gmCWInterface::RequestLeaveStore` · 00450240 · legions 00451380 — Command runs its IFun on the player's script (VM only; no session stand-in). Stub packs and the forwarders traced; the requests need an open shopper (a server ViewStore), so no store request is byte-verified end to end yet (store_script_7160). RequestSell's three ints are not decoded.
- [x] `gmCWInterface::RequestLeaveCatalog` · 00450260 · legions 004513a0 — ac2-client gameplay.rs Command to the scripts.rs IFun; ac2-net request.rs ScriptRequest; tests/store_requests_7160.rs and consignment_script_7160.rs
- [x] `gmCWInterface::RequestLeaveConsignment` · 00450280 · legions 004513c0 — ac2-client gameplay.rs Command to the scripts.rs IFun; ac2-net request.rs ScriptRequest; tests/store_requests_7160.rs and consignment_script_7160.rs
- [x] `gmCWInterface::RequestEnterConsignment` · 004502a0 · legions 004513e0 — ac2-client gameplay.rs Command to the scripts.rs IFun; ac2-net request.rs ScriptRequest; tests/store_requests_7160.rs and consignment_script_7160.rs
- [~] `gmCWInterface::RequestEnterStore` · 004502c0 · legions 00451400 — Command runs its IFun on the player's script (VM only; no session stand-in). Stub packs and the forwarders traced. The requests need an open shopper (a server ViewStore); ViewStore with a StoreTemplate reference stops at the shop build (store_script_7160 pins the notices). RequestSell's pack is traced (WM_Store 0x4E) but its locals are not.
- [x] `gmCWInterface::RequestSell` · 004502e0 · legions 00451420 — The sell body (Legions 0x4617F0 IFun; stub WM_Store 0x4E sends WM_Store 0x43) runs on the VM: the sell's packed values are an int (the catalog's one store, the body's head of StoreGroup 11), the price, the quantity, the item (long), the storekeeper (long); sell_inside_a_consignment_sends_the_request_sell_bytes (consignment_script_7160) asserts those bytes. Needs the catalog open and the consignment entered; no catalog refuses (sell_without_a_catalog_is_refused_by_the_script).
- [x] `gmCWInterface::RequestRestock` · 00450310 · legions 00451450 — ac2-client gameplay.rs Command to the scripts.rs IFun; ac2-net request.rs ScriptRequest; tests/store_requests_7160.rs and consignment_script_7160.rs
- [x] `gmCWInterface::RequestCollect` · 00450340 · legions 00451480 — ac2-client gameplay.rs Command to the scripts.rs IFun; ac2-net request.rs ScriptRequest; tests/store_requests_7160.rs and consignment_script_7160.rs
- [x] `gmCWInterface::GetConsignmentCharge` · 00450360 · legions 004514a0 — ac2-client gameplay.rs Command to the scripts.rs IFun; ac2-net request.rs ScriptRequest; tests/store_requests_7160.rs and consignment_script_7160.rs
- [~] `gmCWInterface::DisplaySaleReminders` · 00450390 · legions 004514d0 — Command::DisplaySaleReminders sends IFun 0xB4013D (scripts.rs); the stub sends nothing with a shopper open, and the sale reminder notice (gmToolbarUI::RecvNotice_ShowSaleReminder) is not shown
- [x] `gmCWInterface::RequestPetAttack` · 004503b0 · legions 004514f0 — ac2-client Command::PetAttack / PetBuff / PetHeal / PetReturn / PetMode / PetRename run the IFun on the VM (gameplay.rs, scripts.rs IFUN_REQUEST_PET_*); tests/pets_7160.rs asserts the packed calls
- [x] `gmCWInterface::RequestPetBuff` · 004503d0 · legions 00451510 — ac2-client Command::PetAttack / PetBuff / PetHeal / PetReturn / PetMode / PetRename run the IFun on the VM (gameplay.rs, scripts.rs IFUN_REQUEST_PET_*); tests/pets_7160.rs asserts the packed calls
- [x] `gmCWInterface::RequestPetHeal` · 004503f0 · legions 00451530 — ac2-client Command::PetAttack / PetBuff / PetHeal / PetReturn / PetMode / PetRename run the IFun on the VM (gameplay.rs, scripts.rs IFUN_REQUEST_PET_*); tests/pets_7160.rs asserts the packed calls
- [x] `gmCWInterface::RequestPetReturn` · 00450410 · legions 00451550 — ac2-client Command::PetAttack / PetBuff / PetHeal / PetReturn / PetMode / PetRename run the IFun on the VM (gameplay.rs, scripts.rs IFUN_REQUEST_PET_*); tests/pets_7160.rs asserts the packed calls
- [x] `gmCWInterface::RequestPetMode` · 00450430 · legions 00451570 — ac2-client Command::PetAttack / PetBuff / PetHeal / PetReturn / PetMode / PetRename run the IFun on the VM (gameplay.rs, scripts.rs IFUN_REQUEST_PET_*); tests/pets_7160.rs asserts the packed calls
- [x] `gmCWInterface::RequestSetPetName` · 00450460 · legions 004515a0 — ac2-client Command::PetAttack / PetBuff / PetHeal / PetReturn / PetMode / PetRename run the IFun on the VM (gameplay.rs, scripts.rs IFUN_REQUEST_PET_*); tests/pets_7160.rs asserts the packed calls
- [~] `gmCWInterface::MoveItem` · 00450490 · legions 004515d0 — ac2-weenie session/inventory.rs move_item: InvMoveDesc / ReorganizeContents byte-exact (inventory_7160, chest_7160); client Command::MoveItem (item_drag_7160, inventory_items_7160). Script checks and optimistic updates not mirrored
- [~] `gmCWInterface::EquipItem` · 004504f0 · legions 00451630 — ac2-weenie session/inventory.rs equip_item: InvEquipDesc.equip byte-exact (inventory_7160); client EquipItem (inventory_items_7160). Script checks not mirrored
- [~] `gmCWInterface::UnEquipItem` · 00450530 · legions 00451670 — ac2-weenie session/inventory.rs unequip_item: InvEquipDesc.unequip byte-exact (inventory_7160); client UnequipItem (inventory_items_7160). Script checks not mirrored
- [x] `gmCWInterface::GetContentsInfo` · 00450570 · legions 004516b0 — ac2-client Client::contents_info / container_segment_info (Container 0x3D / 0x3F on the VM); the double sends ContainerSegmentDescriptor segments
- [x] `gmCWInterface::GetContainerSegmentInfo` · 004505a0 · legions 004516e0 — ac2-client Client::contents_info / container_segment_info (Container 0x3D / 0x3F on the VM); the double sends ContainerSegmentDescriptor segments
- [x] `gmCWInterface::GetPreferredInventoryLocation` · 004505d0 · legions 00451710 — ac2-weenie ItemStats::preferred_location (Item 0x26: int 282), valid_locations (275), precluded_locations (283)
- [~] `gmCWInterface::InqSimpleInvByLocTable` · 00450600 · legions 00451740 — answered by the player's script (inventory_queries.rs over the WM_Inventory messages); the UI unions the script's list with the frame mirror, as a move into a side pack is not yet reflected by the script (inventory_items_7160 Known Gap)
- [~] `gmCWInterface::InqAllInventoryIIDList` · 00450620 · legions 00451760 — answered by the player's script (inventory_queries.rs over the WM_Inventory messages); the UI unions the script's list with the frame mirror, as a move into a side pack is not yet reflected by the script (inventory_items_7160 Known Gap)
- [~] `gmCWInterface::DoIHaveItem` · 00450640 · legions 00451780 — answered by the player's script (inventory_queries.rs over the WM_Inventory messages); the UI unions the script's list with the frame mirror, as a move into a side pack is not yet reflected by the script (inventory_items_7160 Known Gap)
- [x] `gmCWInterface::IsOnInventory` · 00450680 · legions 004517c0 — the worn test: true for an item the player has equipped, false for pack, side-pack and unknown items (inventory_items_7160); the money bag drop callback is admin-only
- [~] `gmCWInterface::DragToMoneyBag` · 004506c0 · legions 00451800 — ac2-weenie OnlineSession::drag_to_money_bag / drag_from_money_bag (CMoneySystem 0x4 / 0x5 -> WM_Money 0x7 / 0x8, receive_money for 0x6 / 0x9); MoneySystem 0x6's ownership checks and 0x2's room check not mirrored
- [~] `gmCWInterface::DragFromMoneyBag` · 00450700 · legions 00451840 — ac2-weenie OnlineSession::drag_to_money_bag / drag_from_money_bag (CMoneySystem 0x4 / 0x5 -> WM_Money 0x7 / 0x8, receive_money for 0x6 / 0x9); MoneySystem 0x6's ownership checks and 0x2's room check not mirrored
- [~] `gmCWInterface::LootAllFromContainer` · 00450730 · legions 00451870 — Loot All runs on the player's script: IFun 0xB400E8 sends WM_Inventory 0x68 after the wlib counts the corpse's lists, which the open answer fills (CPlayer 0x8C -> CInventorySystem); corpse_script_7160. Another character's corpse is refused with NotMine (1077, consumed by CInventorySystem's open handler); the CorpseUsagePermission (0x26A) fields and the CorpsePermissionBlob message (0x1BDF) stay untraced
- [~] `gmCWInterface::TransmuteAllFromContainer` · 00450750 · legions 00451890 — ac2-weenie session/inventory.rs transmute_all: InvTransmuteAllDesc (client test transmute_all_asks_the_server_to_transmute_the_container). The double clears the container without resending its lists, so the frame refresh is the server's; checks (CInventorySystem 0x1F) not mirrored
- [x] `gmCWInterface::HasDurability` · 00450770 · legions 004518b0 — ac2-weenie ItemStats::of (gmEntity 0x10/0x23/0xB/0x158/0x159/0x15B over the Inq helpers); ac2-client InventoryFrame::items, Client::item_display
- [x] `gmCWInterface::GetDurabilityLevel` · 004507a0 · legions 004518e0 — ac2-weenie ItemStats::of (gmEntity 0x10/0x23/0xB/0x158/0x159/0x15B over the Inq helpers); ac2-client InventoryFrame::items, Client::item_display
- [x] `gmCWInterface::GetMaxDurabilityLevel` · 004507d0 · legions 00451910 — ac2-weenie ItemStats::of (gmEntity 0x10/0x23/0xB/0x158/0x159/0x15B over the Inq helpers); ac2-client InventoryFrame::items, Client::item_display
- [x] `gmCWInterface::GetIconID` · 00450800 · legions 00451940 — ac2-weenie ItemStats::of (gmEntity 0x10/0x23/0xB/0x158/0x159/0x15B over the Inq helpers); ac2-client InventoryFrame::items, Client::item_display
- [~] `gmCWInterface::GetMaxContainerSize` · 00450830 · legions 00451970 — ac2-script item_queries::ItemQueries::container_capacity; ItemStats capacity default 12, the EffectSystem pass is not applied
- [ ] `gmCWInterface::GetAvailableExperience` · 00450860 · legions 004519a0 — needs port: interface get available experience; no citation in crates/
- [x] `gmCWInterface::GetSkillCredits` · 004508a0 · legions 004519e0 — ac2-client scripts_queries: the IFun's query on the player's script; asserted in skill_script_7160
- [x] `gmCWInterface::GetHeroSkillCredits` · 004508d0 · legions 00451a10 — ac2-client scripts_queries: the IFun's query on the player's script; asserted in skill_script_7160
- [x] `gmCWInterface::Dismount` · 00450900 · legions 00451a40 — ac2-client target_select.rs run (SelectionAction::StopAttacking -> Command::StopAttacking) and gameplay.rs own_ifun -> IFun 0xB40104 on the VM; toolbar.rs is_dismount_press -> hud_player.rs Command::Dismount -> IFun 0xB400E5; the VM's WM_Combat 0x25 / WM_Player 0x33 calls (ac2-net calls.rs); tests combat_7160 stop_attacking_runs_on_the_vm_for_the_target, dismount_runs_on_the_vm; the double answers the targeted stop
- [x] `gmCWInterface::InqRootPanelFromRace` · 00450920 · legions 00451a60 — implemented: cited at crates/ac2-ui/src/skill_tree.rs:114
- [x] `gmCWInterface::IsSkillTrainedByEnum` · 00450960 · legions 00451aa0 — ac2-client scripts_queries: the IFun's query on the player's script; asserted in skill_script_7160
- [x] `gmCWInterface::IsSkillBeingUntrainedByEnum` · 004509a0 · legions 00451ae0 — ac2-client scripts_queries: the IFun's query on the player's script; asserted in skill_script_7160
- [~] `gmCWInterface::IsHeroSkillByEnum` · 004509e0 · legions 00451b20 — query or request on the IFun, asserted by skill_script_7160 (untrain and cancel are answered with the repository, whose slot 5 names the skill being untrained; the double completes no untrain timer)
- [x] `gmCWInterface::CanSkillBeTrainedByEnum` · 00450a20 · legions 00451b60 — ac2-client scripts_queries: the IFun's query on the player's script; asserted in skill_script_7160
- [~] `gmCWInterface::CanSkillBeUntrainedByEnum` · 00450a60 · legions 00451ba0 — query or request on the IFun, asserted by skill_script_7160 (untrain and cancel are answered with the repository, whose slot 5 names the skill being untrained; the double completes no untrain timer)
- [x] `gmCWInterface::CanSkillBeRaisedByEnum` · 00450aa0 · legions 00451be0 — ac2-client scripts_queries: the IFun's query on the player's script; asserted in skill_script_7160
- [x] `gmCWInterface::GetSkillExperienceNeededToLevelByEnum` · 00450ae0 · legions 00451c20 — ac2-client scripts_queries: the IFun's query on the player's script; asserted in skill_script_7160
- [~] `gmCWInterface::InqSkillLevelByEnum` · 00450b30 · legions 00451c70 — query or request on the IFun, asserted by skill_script_7160 (untrain and cancel are answered with the repository, whose slot 5 names the skill being untrained; the double completes no untrain timer)
- [x] `gmCWInterface::GetSkillCostToTrainByEnum` · 00450b70 · legions 00451cb0 — ac2-client scripts_queries: the IFun's query on the player's script; asserted in skill_script_7160
- [x] `gmCWInterface::GetSkillXPByEnum` · 00450bb0 · legions 00451cf0 — ac2-client scripts_queries: the IFun's query on the player's script; asserted in skill_script_7160
- [x] `gmCWInterface::RequestTrainSkill` · 00450c00 · legions 00451d40 — ac2-client skills.rs: the request's IFun on the player's script (VM stub sends WM_Skill 0x13 / 0x10, bytes asserted in skill_script_7160); the double answers (character.rs answer_skill)
- [~] `gmCWInterface::RequestUntrainSkill` · 00450c20 · legions 00451d60 — query or request on the IFun, asserted by skill_script_7160 (untrain and cancel are answered with the repository, whose slot 5 names the skill being untrained; the double completes no untrain timer)
- [x] `gmCWInterface::RequestRaiseSkill` · 00450c40 · legions 00451d80 — ac2-client skills.rs: the request's IFun on the player's script (VM stub sends WM_Skill 0x13 / 0x10, bytes asserted in skill_script_7160); the double answers (character.rs answer_skill)
- [~] `gmCWInterface::RequestUntrainSkillCancel` · 00450c60 · legions 00451da0 — query or request on the IFun, asserted by skill_script_7160 (untrain and cancel are answered with the repository, whose slot 5 names the skill being untrained; the double completes no untrain timer)
- [~] `gmCWInterface::InqSkillTimeLeft` · 00450c80 · legions 00451dc0 — query or request on the IFun, asserted by skill_script_7160 (untrain and cancel are answered with the repository, whose slot 5 names the skill being untrained; the double completes no untrain timer)
- [~] `gmCWInterface::IsSkillUsableBasedOnInventoryByEnum` · 00450cc0 · legions 00451e00 — query or request on the IFun, asserted by skill_script_7160 (untrain and cancel are answered with the repository, whose slot 5 names the skill being untrained; the double completes no untrain timer)
- [ ] `gmCWInterface::GUNHandleReauthentication` · 00450d00 · legions 00451e40 — needs port: interface gunhandle reauthentication; no citation in crates/
- [~] `gmCWInterface::Result_SendToUser` · 00450d10 · legions 00451e50 — GUN-only (no Turbine GUN in this client): the script's request answers 0 without a GUN service, as RequestGUN_* does when TurbineGun::S_pTurbineGun is NULL (ac2-script host/gun.rs gun(); the page and friend requests gate the same way in ac2-client friends.rs); with a service the request is queued but the GUN answers and wire are not modelled; the Result and HearTell paths need those answers
- [~] `gmCWInterface::Result_SendToRoom` · 00450d40 · legions 00451e80 — GUN-only (no Turbine GUN in this client): the script's request answers 0 without a GUN service, as RequestGUN_* does when TurbineGun::S_pTurbineGun is NULL (ac2-script host/gun.rs gun(); the page and friend requests gate the same way in ac2-client friends.rs); with a service the request is queued but the GUN answers and wire are not modelled; the Result and HearTell paths need those answers
- [x] `gmCWInterface::Result_FriendAdd` · 00450d70 · legions 00451eb0 — net: friends_script_7160 (the login's GUN requests are the VM's exact values; Result_FriendAdd/Delete IFuns (ids 0xB40040/41, match Legions) raise FriendAdded/FriendRemoved with the name)
- [x] `gmCWInterface::Result_FriendDelete` · 00450d90 · legions 00451ed0 — net: friends_script_7160 (the login's GUN requests are the VM's exact values; Result_FriendAdd/Delete IFuns (ids 0xB40040/41, match Legions) raise FriendAdded/FriendRemoved with the name)
- [x] `gmCWInterface::GUNEvent_FriendStatus` · 00450db0 · legions 00451ef0 — ac2-client friends.rs gun_friend_event (list and status to the page through take_friend_events; the script IFuns run first); ac2-ui friends_panel.rs; tests friends_script_7160, friends_page_7160
- [~] `gmCWInterface::Result_IgnoredAdd` · 00450de0 · legions 00451f20 — GUN-only (no Turbine GUN in this client): the script's request answers 0 without a GUN service, as RequestGUN_* does when TurbineGun::S_pTurbineGun is NULL (ac2-script host/gun.rs gun(); the page and friend requests gate the same way in ac2-client friends.rs); with a service the request is queued but the GUN answers and wire are not modelled; the Result and HearTell paths need those answers
- [~] `gmCWInterface::Result_IgnoredDelete` · 00450e00 · legions 00451f40 — GUN-only (no Turbine GUN in this client): the script's request answers 0 without a GUN service, as RequestGUN_* does when TurbineGun::S_pTurbineGun is NULL (ac2-script host/gun.rs gun(); the page and friend requests gate the same way in ac2-client friends.rs); with a service the request is queued but the GUN answers and wire are not modelled; the Result and HearTell paths need those answers
- [~] `gmCWInterface::Result_CallCreate` · 00450e20 · legions 00451f60 — GUN-only (no Turbine GUN in this client): the script's request answers 0 without a GUN service, as RequestGUN_* does when TurbineGun::S_pTurbineGun is NULL (ac2-script host/gun.rs gun(); the page and friend requests gate the same way in ac2-client friends.rs); with a service the request is queued but the GUN answers and wire are not modelled; the Result and HearTell paths need those answers
- [~] `gmCWInterface::InqEffectList` · 00450e60 · legions 00451fa0 — ac2-client Client::effect_list (IFun 0xB40062, CPlayer 0x52 -> CEffectSystem 0x7); waits on the VM's gEffectUINode efuns
- [x] `gmCWInterface::GetStackable` · 00450ea0 · legions 00451fe0 — ac2-weenie ItemStats::of (gmEntity 0x10/0x23/0xB/0x158/0x159/0x15B over the Inq helpers); ac2-client InventoryFrame::items, Client::item_display
- [x] `gmCWInterface::GetQuantity` · 00450ed0 · legions 00452010 — ac2-script item_queries::ItemQueries::item_display (IFun 0xB40064 on the item's process)
- [~] `gmCWInterface::GetMaxStackSize` · 00450f00 · legions 00452040 — ItemStats::max_stack: int 261; InventoryGlobals slot-5 default only through the VM
- [x] `gmCWInterface::SetExaminationRequest` · 00450f30 · legions 00452070 — ac2-weenie player_script.rs (PlayerScript: selection, last selection, examination request and panel) and session/gameplay.rs select/examine/start_examination/stop_examination; ac2-client gameplay.rs Command::Select/Examine/StartExamination/StopExamination, target_select.rs snapshot; SetSelection's IFun 0xB40002 runs on the VM (SCRIPT_VM_FUNCTIONS); GetSelection/HasSomethingSelected answered from the mirror, their IFuns are not queried
- [x] `gmCWInterface::AddEffectDataToProfileByEID` · 00450f50 · legions 00452090 — ui_examination.rs add_effect_data: IFun 0xB40123 with context, order, id and the profile; the script's own Add* efuns other than AddEffectID stay unbound
- [x] `gmCWInterface::StartExamination` · 00450f80 · legions 004520c0 — ac2-weenie player_script.rs (PlayerScript: selection, last selection, examination request and panel) and session/gameplay.rs select/examine/start_examination/stop_examination; ac2-client gameplay.rs Command::Select/Examine/StartExamination/StopExamination, target_select.rs snapshot; SetSelection's IFun 0xB40002 runs on the VM (SCRIPT_VM_FUNCTIONS); GetSelection/HasSomethingSelected answered from the mirror, their IFuns are not queried
- [x] `gmCWInterface::StopExamination` · 00450fa0 · legions 004520e0 — ac2-weenie player_script.rs (PlayerScript: selection, last selection, examination request and panel) and session/gameplay.rs select/examine/start_examination/stop_examination; ac2-client gameplay.rs Command::Select/Examine/StartExamination/StopExamination, target_select.rs snapshot; SetSelection's IFun 0xB40002 runs on the VM (SCRIPT_VM_FUNCTIONS); GetSelection/HasSomethingSelected answered from the mirror, their IFuns are not queried
- [x] `gmCWInterface::HasSomethingSelected` · 00450fc0 · legions 00452100 — ac2-weenie player_script.rs (PlayerScript: selection, last selection, examination request and panel) and session/gameplay.rs select/examine/start_examination/stop_examination; ac2-client gameplay.rs Command::Select/Examine/StartExamination/StopExamination, target_select.rs snapshot; SetSelection's IFun 0xB40002 runs on the VM (SCRIPT_VM_FUNCTIONS); GetSelection/HasSomethingSelected answered from the mirror, their IFuns are not queried
- [ ] `gmCWInterface::RegisterCommands` · 00450ff0 · legions 00452130 — needs port: interface register commands; no citation in crates/
- [x] `gmCWInterface::RequestResurrect` · 00451010 · legions 00452150 — ac2-ui death.rs (DeathPanel: the resurrect dialog, its text, its button's respawn); client hud_death.rs wires the notice and the answer; death_panel_7160 and combat_7160 (death, respawn)
- [x] `gmCWInterface::ClearResurrectionRequest` · 00451030 · legions 00452170 — implemented: cited at crates/ac2-ui/src/death.rs:12
- [x] `gmCWInterface::UpdateOpen` · 00451050 · legions 00452190 — implemented: cited at crates/ac2-ui/src/external_container.rs:6
- [x] `gmCWInterface::InqShortcutInfo` · 00451090 · legions 004521d0 — client shortcut_info (scripts_queries.rs, IFun 0xB4006D); icon in ui_sources ShortcutSource::icon; shortcut_drag_7160 and view tests
- [x] `gmCWInterface::SetShortcut` · 004510d0 · legions 00452210 — client set_shortcut (scripts_queries.rs, IFun 0xB4006F); shortcut_drag_7160 asserts the SetShortcut bytes (ShortcutInfo native, slot)
- [~] `gmCWInterface::InqShortcutArray` · 00451110 · legions 00452250 — read from GetGameplayOptions' profile (scripts_queries.rs shortcut_array, progress_7160), not IFun 0xB40070 itself
- [x] `gmCWInterface::SetWhichShortcutSet` · 00451150 — ac2-client Command::SetShortcutSet: the IFun on the VM, the session's shortcut mirror (a profile write, nothing to the server)
- [~] `gmCWInterface::GetWhichShortcutSet` · 00451170 · legions 004522b0 — read from the same options profile (shortcut_settings.which_set), not IFun 0xB40072 itself
- [x] `gmCWInterface::SetShowShortcutNumbers` · 004511a0 · legions 004522e0 — ac2-client Command::SetGameplayOption / UploadGameOptions: the IFun on the player's script (scripts::option_call, IFun ids in SCRIPT_VM_FUNCTIONS); the page's requests (ac2-ui character.rs)
- [x] `gmCWInterface::GetShowShortcutNumbers` · 004511c0 · legions 00452300 — ac2-ui character_options.rs read(): the profile's bit, radar mask, shortcut height and range as the page reads them (the profile from ac2-client options_settings)
- [x] `gmCWInterface::SetShortcutLocked` · 004511f0 · legions 00452330 — ac2-client Command::SetGameplayOption / UploadGameOptions: the IFun on the player's script (scripts::option_call, IFun ids in SCRIPT_VM_FUNCTIONS); the page's requests (ac2-ui character.rs)
- [x] `gmCWInterface::GetShortcutLocked` · 00451210 · legions 00452350 — ac2-ui character_options.rs read(): the profile's bit, radar mask, shortcut height and range as the page reads them (the profile from ac2-client options_settings)
- [x] `gmCWInterface::SetShortcutInstaScroll` · 00451240 · legions 00452380 — ac2-client Command::SetGameplayOption / UploadGameOptions: the IFun on the player's script (scripts::option_call, IFun ids in SCRIPT_VM_FUNCTIONS); the page's requests (ac2-ui character.rs)
- [x] `gmCWInterface::GetShortcutInstaScroll` · 00451260 · legions 004523a0 — ac2-ui character_options.rs read(): the profile's bit, radar mask, shortcut height and range as the page reads them (the profile from ac2-client options_settings)
- [x] `gmCWInterface::SetShowPKDamage` · 00451290 · legions 004523d0 — ac2-client Command::SetGameplayOption / UploadGameOptions: the IFun on the player's script (scripts::option_call, IFun ids in SCRIPT_VM_FUNCTIONS); the page's requests (ac2-ui character.rs)
- [x] `gmCWInterface::GetShowPKDamage` · 004512b0 · legions 004523f0 — ac2-ui character_options.rs read(): the profile's bit, radar mask, shortcut height and range as the page reads them (the profile from ac2-client options_settings)
- [x] `gmCWInterface::SetShowDamageText` · 004512e0 · legions 00452420 — ac2-client Command::SetGameplayOption / UploadGameOptions: the IFun on the player's script (scripts::option_call, IFun ids in SCRIPT_VM_FUNCTIONS); the page's requests (ac2-ui character.rs)
- [x] `gmCWInterface::GetShowDamageText` · 00451300 · legions 00452440 — ac2-ui character_options.rs read(): the profile's bit, radar mask, shortcut height and range as the page reads them (the profile from ac2-client options_settings)
- [x] `gmCWInterface::SetShowDamageTextOther` · 00451330 · legions 00452470 — ac2-client Command::SetGameplayOption / UploadGameOptions: the IFun on the player's script (scripts::option_call, IFun ids in SCRIPT_VM_FUNCTIONS); the page's requests (ac2-ui character.rs)
- [x] `gmCWInterface::GetShowDamageTextOther` · 00451350 · legions 00452490 — ac2-ui character_options.rs read(): the profile's bit, radar mask, shortcut height and range as the page reads them (the profile from ac2-client options_settings)
- [x] `gmCWInterface::SetDamageTextRangeOther` · 00451380 · legions 004524c0 — ac2-client Command::SetGameplayOption / UploadGameOptions: the IFun on the player's script (scripts::option_call, IFun ids in SCRIPT_VM_FUNCTIONS); the page's requests (ac2-ui character.rs)
- [x] `gmCWInterface::GetDamageTextRangeOther` · 004513a0 · legions 004524e0 — ac2-ui character_options.rs read(): the profile's bit, radar mask, shortcut height and range as the page reads them (the profile from ac2-client options_settings)
- [x] `gmCWInterface::SetShortcutHeight` · 004513d0 · legions 00452510 — ac2-client Command::SetGameplayOption / UploadGameOptions: the IFun on the player's script (scripts::option_call, IFun ids in SCRIPT_VM_FUNCTIONS); the page's requests (ac2-ui character.rs)
- [x] `gmCWInterface::GetShortcutHeight` · 004513f0 · legions 00452530 — ac2-ui character_options.rs read(): the profile's bit, radar mask, shortcut height and range as the page reads them (the profile from ac2-client options_settings)
- [x] `gmCWInterface::SetBypassGoldMeltingConfirmationDlg` · 00451420 · legions 00452560 — ac2-client Command::SetGameplayOption / UploadGameOptions: the IFun on the player's script (scripts::option_call, IFun ids in SCRIPT_VM_FUNCTIONS); the page's requests (ac2-ui character.rs)
- [x] `gmCWInterface::GetBypassGoldMeltingConfirmationDlg` · 00451440 · legions 00452580 — ac2-ui money_bag.rs / item_display.rs / backpack.rs; ac2-client ui_inventory.rs; ac2-script host/inventory_globals.rs (coin max) and the player's IFUN 0xB400D7 (bypass); Legions 7160 matches admin here (same IFUNs 0xA60001 and 0xB400D7)
- [x] `gmCWInterface::SetLatencyIndicatorGraph` · 00451470 · legions 004525b0 — ac2-client Command::SetGameplayOption / UploadGameOptions: the IFun on the player's script (scripts::option_call, IFun ids in SCRIPT_VM_FUNCTIONS); the page's requests (ac2-ui character.rs)
- [x] `gmCWInterface::GetLatencyIndicatorGraph` · 00451490 · legions 004525d0 — ac2-ui character_options.rs read(): the profile's bit, radar mask, shortcut height and range as the page reads them (the profile from ac2-client options_settings)
- [x] `gmCWInterface::SetShowFellowsOnMap` · 004514c0 · legions 00452600 — ac2-client Command::SetGameplayOption / UploadGameOptions: the IFun on the player's script (scripts::option_call, IFun ids in SCRIPT_VM_FUNCTIONS); the page's requests (ac2-ui character.rs)
- [x] `gmCWInterface::GetShowFellowsOnMap` · 004514e0 · legions 00452620 — ac2-ui character_options.rs read(): the profile's bit, radar mask, shortcut height and range as the page reads them (the profile from ac2-client options_settings)
- [x] `gmCWInterface::SetShowPetUI` · 00451510 · legions 00452650 — ac2-client Command::SetGameplayOption / UploadGameOptions: the IFun on the player's script (scripts::option_call, IFun ids in SCRIPT_VM_FUNCTIONS); the page's requests (ac2-ui character.rs)
- [x] `gmCWInterface::GetShowPetUI` · 00451530 · legions 00452670 — ac2-ui character_options.rs read(): the profile's bit, radar mask, shortcut height and range as the page reads them (the profile from ac2-client options_settings)
- [x] `gmCWInterface::SetUISaveLocations` · 00451560 · legions 00452290 — ac2-client client_locations.rs: ReloadOptions and the refresh apply the profile's locations (hud_player.rs); a moved window sends SaveWindowLocations: IFun 0xB4007B then 0xB400DB (WM_Player 0x32); AddLocation bucket order in ac2-script native/options.rs; tests window_locations_7160.rs
- [x] `gmCWInterface::InqUISaveLocations` · 00451580 · legions 004526c0 — ac2-client client_locations.rs: ReloadOptions and the refresh apply the profile's locations (hud_player.rs); a moved window sends SaveWindowLocations: IFun 0xB4007B then 0xB400DB (WM_Player 0x32); AddLocation bucket order in ac2-script native/options.rs; tests window_locations_7160.rs
- [x] `gmCWInterface::SetShowRadarCoords` · 004515c0 · legions 00452700 — ac2-ui radar.rs + ac2-client (Command::SetShowRadarCoords, IFun 0xB4007D; show_radar_coords from the options profile)
- [x] `gmCWInterface::GetShowRadarCoords` · 004515e0 · legions 00452720 — ac2-ui radar.rs + ac2-client (Command::SetShowRadarCoords, IFun 0xB4007D; show_radar_coords from the options profile)
- [x] `gmCWInterface::SetShowObjectNames` · 00451610 · legions 00452750 — ac2-ui name_plates.rs + ac2-client (Command::SetShowObjectNames, IFun 0xB400CE)
- [x] `gmCWInterface::GetShowObjectNames` · 00451630 · legions 00452770 — ac2-ui name_plates.rs + ac2-client (Command::SetShowObjectNames, IFun 0xB400CE)
- [x] `gmCWInterface::SetRadarMask` · 00451660 · legions 004527a0 — ac2-client Command::SetGameplayOption / UploadGameOptions: the IFun on the player's script (scripts::option_call, IFun ids in SCRIPT_VM_FUNCTIONS); the page's requests (ac2-ui character.rs)
- [x] `gmCWInterface::GetRadarMask` · 00451680 · legions 004527c0 — ac2-ui character_options.rs read(): the profile's bit, radar mask, shortcut height and range as the page reads them (the profile from ac2-client options_settings)
- [x] `gmCWInterface::SetChatWindowFilter` · 004516b0 · legions 004527f0 — ac2-client: Command::SetChatOption / SetWindowOpacity (IFuns 0xB400D3/D4/DD/83, DF; SetWindowOpacity then UploadUISettings), UploadChatOptions (IFun 0xB400D5, WM_Player 0x2E asserted in chat_options_7160), InqGameplayOptions = GetGameplayOptions IFun 0xB400DE (scripts_queries gameplay_options); ac2-ui chat_window.rs opacity slider, apply_opacity, reload (window_locations_tests.rs, window_locations_7160.rs a_window_opacity...)
- [x] `gmCWInterface::SetChatFontSize` · 004516f0 · legions 00452830 — ac2-client: Command::SetChatOption / SetWindowOpacity (IFuns 0xB400D3/D4/DD/83, DF; SetWindowOpacity then UploadUISettings), UploadChatOptions (IFun 0xB400D5, WM_Player 0x2E asserted in chat_options_7160), InqGameplayOptions = GetGameplayOptions IFun 0xB400DE (scripts_queries gameplay_options); ac2-ui chat_window.rs opacity slider, apply_opacity, reload (window_locations_tests.rs, window_locations_7160.rs a_window_opacity...)
- [x] `gmCWInterface::SetChatFontColor` · 00451730 · legions 00452870 — ac2-client: Command::SetChatOption / SetWindowOpacity (IFuns 0xB400D3/D4/DD/83, DF; SetWindowOpacity then UploadUISettings), UploadChatOptions (IFun 0xB400D5, WM_Player 0x2E asserted in chat_options_7160), InqGameplayOptions = GetGameplayOptions IFun 0xB400DE (scripts_queries gameplay_options); ac2-ui chat_window.rs opacity slider, apply_opacity, reload (window_locations_tests.rs, window_locations_7160.rs a_window_opacity...)
- [x] `gmCWInterface::SetWindowOpacity` · 00451770 · legions 004528f0 — ac2-client: Command::SetChatOption / SetWindowOpacity (IFuns 0xB400D3/D4/DD/83, DF; SetWindowOpacity then UploadUISettings), UploadChatOptions (IFun 0xB400D5, WM_Player 0x2E asserted in chat_options_7160), InqGameplayOptions = GetGameplayOptions IFun 0xB400DE (scripts_queries gameplay_options); ac2-ui chat_window.rs opacity slider, apply_opacity, reload (window_locations_tests.rs, window_locations_7160.rs a_window_opacity...)
- [x] `gmCWInterface::SetChatPopupFlag` · 004517b0 · legions 00452930 — ac2-client: Command::SetChatOption / SetWindowOpacity (IFuns 0xB400D3/D4/DD/83, DF; SetWindowOpacity then UploadUISettings), UploadChatOptions (IFun 0xB400D5, WM_Player 0x2E asserted in chat_options_7160), InqGameplayOptions = GetGameplayOptions IFun 0xB400DE (scripts_queries gameplay_options); ac2-ui chat_window.rs opacity slider, apply_opacity, reload (window_locations_tests.rs, window_locations_7160.rs a_window_opacity...)
- [x] `gmCWInterface::InqGameplayOptions` · 004517f0 · legions 00452970 — ac2-client: Command::SetChatOption / SetWindowOpacity (IFuns 0xB400D3/D4/DD/83, DF; SetWindowOpacity then UploadUISettings), UploadChatOptions (IFun 0xB400D5, WM_Player 0x2E asserted in chat_options_7160), InqGameplayOptions = GetGameplayOptions IFun 0xB400DE (scripts_queries gameplay_options); ac2-ui chat_window.rs opacity slider, apply_opacity, reload (window_locations_tests.rs, window_locations_7160.rs a_window_opacity...)
- [x] `gmCWInterface::SetAutoTarget` · 00451820 · legions 004529a0 — ac2-client Command::SetGameplayOption / UploadGameOptions: the IFun on the player's script (scripts::option_call, IFun ids in SCRIPT_VM_FUNCTIONS); the page's requests (ac2-ui character.rs)
- [x] `gmCWInterface::GetAutoTarget` · 00451840 · legions 004529c0 — ac2-ui character_options.rs read(): the profile's bit, radar mask, shortcut height and range as the page reads them (the profile from ac2-client options_settings)
- [x] `gmCWInterface::SetAcceptAllegianceRequest` · 00451870 · legions 004529f0 — ac2-client Command::SetGameplayOption / UploadGameOptions: the IFun on the player's script (scripts::option_call, IFun ids in SCRIPT_VM_FUNCTIONS); the page's requests (ac2-ui character.rs)
- [x] `gmCWInterface::GetAcceptAllegianceRequest` · 00451890 · legions 00452a10 — ac2-ui character_options.rs read(): the profile's bit, radar mask, shortcut height and range as the page reads them (the profile from ac2-client options_settings)
- [x] `gmCWInterface::SetAcceptFellowshipRequest` · 004518c0 · legions 00452a40 — ac2-client Command::SetGameplayOption / UploadGameOptions: the IFun on the player's script (scripts::option_call, IFun ids in SCRIPT_VM_FUNCTIONS); the page's requests (ac2-ui character.rs)
- [x] `gmCWInterface::GetAcceptFellowshipRequest` · 004518e0 · legions 00452a60 — ac2-ui character_options.rs read(): the profile's bit, radar mask, shortcut height and range as the page reads them (the profile from ac2-client options_settings)
- [x] `gmCWInterface::SetAcceptTradeRequest` · 00451910 · legions 00452a90 — ac2-client Command::SetGameplayOption / UploadGameOptions: the IFun on the player's script (scripts::option_call, IFun ids in SCRIPT_VM_FUNCTIONS); the page's requests (ac2-ui character.rs)
- [x] `gmCWInterface::GetAcceptTradeRequest` · 00451930 · legions 00452ab0 — ac2-ui character_options.rs read(): the profile's bit, radar mask, shortcut height and range as the page reads them (the profile from ac2-client options_settings)
- [ ] `gmCWInterface::InqNextRetellHistoryItem` · 00451960 · legions 00452ae0 — needs port: interface inq next retell history item; no citation in crates/
- [ ] `gmCWInterface::InqPrevRetellHistoryItem` · 00451990 · legions 00452b10 — needs port: interface inq prev retell history item; no citation in crates/
- [ ] `gmCWInterface::InqNextReplyHistoryItem` · 004519c0 · legions 00452b40 — needs port: interface inq next reply history item; no citation in crates/
- [ ] `gmCWInterface::InqPrevReplyHistoryItem` · 004519f0 · legions 00452b70 — needs port: interface inq prev reply history item; no citation in crates/
- [~] `gmCWInterface::InqQuestList` · 00451a20 · legions 00452ba0 — client ui_quests.rs Client::inq_quest_list (IFun 0xB4008E, list as IFunArg::Filled: the out list is approximated, not run against a live login)
- [x] `gmCWInterface::InqActInfoList` · 00451a60 · legions 00452be0 — ac2-script glue/act_info.rs (ggmActInfo, ggmActInfoList efuns); ac2-net native/act.rs (gmActInfo StreamPack); client ui_quests.rs inq_act_info_list (InqActInfoList IFun 0xB4008F)
- [x] `gmCWInterface::HandlePlayScenesDone` · 00451aa0 · legions 00452c20 — implemented: cited at crates/ac2-ui/src/gameplay_story.rs:9
- [x] `gmCWInterface::UploadGameOptions` · 00451ac0 · legions 00452c40 — ac2-client Command::SetGameplayOption / UploadGameOptions: the IFun on the player's script (scripts::option_call, IFun ids in SCRIPT_VM_FUNCTIONS); the page's requests (ac2-ui character.rs)
- [x] `gmCWInterface::UploadChatOptions` · 00451ae0 · legions 00452c60 — ac2-client: Command::SetChatOption / SetWindowOpacity (IFuns 0xB400D3/D4/DD/83, DF; SetWindowOpacity then UploadUISettings), UploadChatOptions (IFun 0xB400D5, WM_Player 0x2E asserted in chat_options_7160), InqGameplayOptions = GetGameplayOptions IFun 0xB400DE (scripts_queries gameplay_options); ac2-ui chat_window.rs opacity slider, apply_opacity, reload (window_locations_tests.rs, window_locations_7160.rs a_window_opacity...)
- [x] `gmCWInterface::UploadUISettings` · 00451b00 · legions 00452c80 — ac2-client client_locations.rs: ReloadOptions and the refresh apply the profile's locations (hud_player.rs); a moved window sends SaveWindowLocations: IFun 0xB4007B then 0xB400DB (WM_Player 0x32); AddLocation bucket order in ac2-script native/options.rs; tests window_locations_7160.rs
- [x] `gmCWInterface::DoFellowshipCreate` · 00451b20 · legions 00452ca0 — ac2-client Command::Social(SocialRequest::Fellowship*): the IFun on the VM, else OnlineSession::social_request (WM_Fellowship stubs, byte-exact)
- [x] `gmCWInterface::DoFellowshipDisband` · 00451b50 · legions 00452cd0 — ac2-client Command::Social(SocialRequest::Fellowship*): the IFun on the VM, else OnlineSession::social_request (WM_Fellowship stubs, byte-exact)
- [x] `gmCWInterface::DoFellowshipLeave` · 00451b70 · legions 00452cf0 — ac2-client Command::Social(SocialRequest::Fellowship*): the IFun on the VM, else OnlineSession::social_request (WM_Fellowship stubs, byte-exact)
- [x] `gmCWInterface::DoFellowshipRecruit` · 00451b90 · legions 00452d10 — ac2-client Command::Social(SocialRequest::Fellowship*): the IFun on the VM, else OnlineSession::social_request (WM_Fellowship stubs, byte-exact)
- [x] `gmCWInterface::DoFellowshipDismiss` · 00451bb0 · legions 00452d30 — ac2-client Command::Social(SocialRequest::Fellowship*): the IFun on the VM, else OnlineSession::social_request (WM_Fellowship stubs, byte-exact)
- [x] `gmCWInterface::DoFellowshipRecruitAccept` · 00451bd0 · legions 00452d50 — ac2-client Command::Social(SocialRequest::Fellowship*): the IFun on the VM, else OnlineSession::social_request (WM_Fellowship stubs, byte-exact)
- [x] `gmCWInterface::DoFellowshipRecruitDecline` · 00451bf0 · legions 00452d70 — ac2-client Command::Social(SocialRequest::Fellowship*): the IFun on the VM, else OnlineSession::social_request (WM_Fellowship stubs, byte-exact)
- [x] `gmCWInterface::IsFellowshipMember` · 00451c10 · legions 00452d90 — implemented: cited at crates/ac2-ui/src/target_cycle.rs:61
- [x] `gmCWInterface::GetFellowXPHash` · 00451c50 · legions 00452dd0 — ac2-client Client::fellow_xp_hash: an empty LAHash out parameter to IFun 0xB400A8 (CPlayer 0x2C -> FellowshipSystem 0x41), read back as (id, share)
- [ ] `gmCWInterface::DoAssistedTell` · 00451c90 · legions 00452e10 — needs port: interface do assisted tell; no citation in crates/
- [x] `gmCWInterface::HandleWeenieEmote` · 00451cb0 · legions 00452e30 — gmCWInterface::HandleWeenieEmote caller is CPhysicsObj::RecvEvt_DoBehavior (admin 004ec680, weenie slot +0x78): ac2-client scripts_io.rs answer_weenie_emote calls IFun 0xB400AA via ScriptHost::call_ifun; ac2-client client_objects.rs apply_remote gates the behaviour on it
- [x] `gmCWInterface::QueryAllegianceProfile` · 00451cf0 · legions 00452e70 — ac2-client Command::Social(SocialRequest allegiance actions): the IFun on the VM, else OnlineSession::social_request (WM_Allegiance stubs)
- [x] `gmCWInterface::SwearAllegiance` · 00451d10 · legions 00452e90 — ac2-client Command::Social(SocialRequest allegiance actions): the IFun on the VM, else OnlineSession::social_request (WM_Allegiance stubs)
- [x] `gmCWInterface::BreakAllegiance` · 00451d30 · legions 00452eb0 — ac2-client Command::Social(SocialRequest allegiance actions): the IFun on the VM, else OnlineSession::social_request (WM_Allegiance stubs)
- [x] `gmCWInterface::AcceptVassal` · 00451d50 · legions 00452ed0 — ac2-client Command::Social(SocialRequest allegiance actions): the IFun on the VM, else OnlineSession::social_request (WM_Allegiance stubs)
- [x] `gmCWInterface::DeclineVassal` · 00451d70 · legions 00452ef0 — ac2-client Command::Social(SocialRequest allegiance actions): the IFun on the VM, else OnlineSession::social_request (WM_Allegiance stubs)
- [x] `gmCWInterface::OpenTradeNegotiations` · 00451d90 · legions 00452f10 — ac2-weenie session/trade.rs: open_trade/close_trade/decline_trade/offer_item_for_trade/revoke_item_from_trade send the WM_Trade stubs (0x2F, 0x3E, 0x23, 0x15, 0x3F) as ScriptRequest; trade_7160
- [x] `gmCWInterface::CloseTradeNegotiations` · 00451db0 · legions 00452f30 — ac2-weenie session/trade.rs: open_trade/close_trade/decline_trade/offer_item_for_trade/revoke_item_from_trade send the WM_Trade stubs (0x2F, 0x3E, 0x23, 0x15, 0x3F) as ScriptRequest; trade_7160
- [x] `gmCWInterface::AcceptTrade` · 00451dd0 · legions 00452f50 — ac2-client Command::AcceptTrade runs IFUN_ACCEPT_TRADE (0x00B400BA) on the VM; ac2-weenie accept_trade is the no-VM stand-in (empty reference)
- [x] `gmCWInterface::DeclineTrade` · 00451df0 · legions 00452f70 — ac2-weenie session/trade.rs: open_trade/close_trade/decline_trade/offer_item_for_trade/revoke_item_from_trade send the WM_Trade stubs (0x2F, 0x3E, 0x23, 0x15, 0x3F) as ScriptRequest; trade_7160
- [x] `gmCWInterface::OfferItemForTrade` · 00451e10 · legions 00452f90 — ac2-weenie session/trade.rs: open_trade/close_trade/decline_trade/offer_item_for_trade/revoke_item_from_trade send the WM_Trade stubs (0x2F, 0x3E, 0x23, 0x15, 0x3F) as ScriptRequest; trade_7160
- [x] `gmCWInterface::RevokeItemFromTrade` · 00451e40 · legions 00452fc0 — ac2-weenie session/trade.rs: open_trade/close_trade/decline_trade/offer_item_for_trade/revoke_item_from_trade send the WM_Trade stubs (0x2F, 0x3E, 0x23, 0x15, 0x3F) as ScriptRequest; trade_7160
- [ ] `gmCWInterface::GetMissileInformationFromWSL` · 00451e60 · legions 00452fe0 — needs port: interface get missile information from wsl; no citation in crates/
- [ ] `gmCWInterface::GetFXAndBehaviorInformationFromWsl` · 00451eb0 · legions 00453030 — needs port: interface get fxand behavior information from wsl; no citation in crates/
- [x] `gmCWInterface::DoUIDamage` · 00451ef0 · legions 00453070 — ac2-client client_story.rs calls IFun 0xB400C0 (scripts_queries ifun::DO_UI_DAMAGE) per damage hook; combat_7160 damage tests. Not run: the native StoryManager::DoStoryHook body
- [-] `gmCWInterface::SpecialAttackStarted` · 00451f50 · legions 004530d0 — no route found: CPlayer 0xD (0x1AA4000D) is called by no wlib function (CALL and CALLSTATIC scans, raw word scan); its only callee is CCombatSystem 0x2, which calls the gCM_Combat start efun; no WM_* package, CAgent forwarder or ECM_* event reaches it. A server InterpCEvent naming 0x9AA4000D would still reach it; no retail server message does
- [~] `gmCWInterface::SpecialAttackEnded` · 00451f70 · legions 004530f0 — no native caller in the call graph (the VM glue raises it); ac2-client hud_player.rs shows EndSpecialAttack notices; IFun 0xB400C2 not called from native
- [x] `gmCWInterface::StopAttackingIfAutoMoving` · 00451f90 · legions 00453110 — gmCWInterface::StopAttackingIfAutoMoving (vtable slot 0xC0 of the player's weenie): the IFun 0xB400C3 call (ggmCWInterface::CallIFun_StopAttackingIfAutoMoving, admin 00463660) runs in ac2-client Command::StopAttackingIfAutoMoving, no arguments
- [x] `gmCWInterface::InqRadarLook` · 00451fb0 · legions 00453130 — ac2-client client_radar.rs + ac2-script host/radar.rs (InqRadarLook), ac2-weenie notice.rs (ChangeRadarLook(obj))
- [x] `gmCWInterface::IsSkillHarmful` · 00451ff0 · legions 00453170 — ac2-client scripts_queries: the IFun's query on the player's script; asserted in skill_script_7160
- [ ] `gmCWInterface::HandleSetWeenieDesc` · 00452030 · legions 004531b0 — needs port: interface handle set weenie desc; no citation in crates/

### GunChatExtraInfo
- [x] `GunChatExtraInfo::PackIntoBlob` · 004520f0 · legions 00453230 — implemented: cited at crates/ac2-net/src/protocol/turbine_chat.rs:94

### gmCWInterface
- [x] `gmCWInterface::ProcessText_Done` · 004521b0 · legions 004532f0 — implemented: cited at crates/ac2-weenie/src/shell/handlers.rs:109
- [~] `gmCWInterface::RequestGUN_FriendAdd` · 004521e0 · legions 00453320 — net: the client's friend add and delete are queued for the GUN service (ChatRequest; friends.rs), with the double's FriendsBook as the GUN answer. The GUN wire is not modelled. The GUN-to-weenie result conversion is only its success case (0); the other codes are not traced
- [~] `gmCWInterface::RequestGUN_FriendDelete` · 00452250 · legions 00453390 — net: the client's friend add and delete are queued for the GUN service (ChatRequest; friends.rs), with the double's FriendsBook as the GUN answer. The GUN wire is not modelled. The GUN-to-weenie result conversion is only its success case (0); the other codes are not traced
- [~] `gmCWInterface::RequestGUN_GetFriendsList` · 004522c0 · legions 00453400 — net: the client's friend add and delete are queued for the GUN service (ChatRequest; friends.rs), with the double's FriendsBook as the GUN answer. The GUN wire is not modelled. The GUN-to-weenie result conversion is only its success case (0); the other codes are not traced
- [~] `gmCWInterface::RequestGUN_IgnoredAdd` · 00452330 · legions 00453470 — GUN-only (no Turbine GUN in this client): the script's request answers 0 without a GUN service, as RequestGUN_* does when TurbineGun::S_pTurbineGun is NULL (ac2-script host/gun.rs gun(); the page and friend requests gate the same way in ac2-client friends.rs); with a service the request is queued but the GUN answers and wire are not modelled; the Result and HearTell paths need those answers
- [~] `gmCWInterface::RequestGUN_IgnoredDelete` · 004523a0 · legions 004534e0 — GUN-only (no Turbine GUN in this client): the script's request answers 0 without a GUN service, as RequestGUN_* does when TurbineGun::S_pTurbineGun is NULL (ac2-script host/gun.rs gun(); the page and friend requests gate the same way in ac2-client friends.rs); with a service the request is queued but the GUN answers and wire are not modelled; the Result and HearTell paths need those answers
- [~] `gmCWInterface::RequestGUN_GetIgnoredList` · 00452410 · legions 00453550 — GUN-only (no Turbine GUN in this client): the script's request answers 0 without a GUN service, as RequestGUN_* does when TurbineGun::S_pTurbineGun is NULL (ac2-script host/gun.rs gun(); the page and friend requests gate the same way in ac2-client friends.rs); with a service the request is queued but the GUN answers and wire are not modelled; the Result and HearTell paths need those answers
- [~] `gmCWInterface::RequestGUN_CallCreate` · 00452480 · legions 004535d2 — GUN-only (no Turbine GUN in this client): the script's request answers 0 without a GUN service, as RequestGUN_* does when TurbineGun::S_pTurbineGun is NULL (ac2-script host/gun.rs gun(); the page and friend requests gate the same way in ac2-client friends.rs); with a service the request is queued but the GUN answers and wire are not modelled; the Result and HearTell paths need those answers

### GunChatExtraInfo
- [-] `GunChatExtraInfo::GunChatExtraInfo` · 00452580 · legions 00453690 — constructor of plain data: Rust struct init replaces it
- [-] `GunChatExtraInfo::~GunChatExtraInfo` · 00452660 · legions 00453770 — destructor: Rust Drop replaces it

### gmCWInterface
- [~] `gmCWInterface::RequestGUN_SendToUser` · 00452710 · legions 00453820 — GUN-only (no Turbine GUN in this client): the script's request answers 0 without a GUN service, as RequestGUN_* does when TurbineGun::S_pTurbineGun is NULL (ac2-script host/gun.rs gun(); the page and friend requests gate the same way in ac2-client friends.rs); with a service the request is queued but the GUN answers and wire are not modelled; the Result and HearTell paths need those answers
- [~] `gmCWInterface::RequestGUN_SendToRoom` · 004527c0 · legions 004538d0 — GUN-only (no Turbine GUN in this client): the script's request answers 0 without a GUN service, as RequestGUN_* does when TurbineGun::S_pTurbineGun is NULL (ac2-script host/gun.rs gun(); the page and friend requests gate the same way in ac2-client friends.rs); with a service the request is queued but the GUN answers and wire are not modelled; the Result and HearTell paths need those answers

### CCliFlagsResult
- [-] `CCliFlagsResult::CCliFlagsResult` · 004528d0 · legions 004539e0 — constructor of plain data: Rust struct init replaces it

### CCliLocationResult
- [-] `CCliLocationResult::CCliLocationResult` · 00452960 · legions 00453a70 — constructor of plain data: Rust struct init replaces it

### `anonymous-namespace'::YesNoDialogAwakener
- [x] ``anonymous-namespace'::YesNoDialogAwakener::YesNoDialogAwakener` · 004529f0 · legions 00453b00 — ac2-client hud_game_dialogs.rs: questions queued in HudView::popup_backlog until the HUD dialogs exist (tests/popup_hud_7160.rs)
- [x] ``anonymous-namespace'::YesNoDialogAwakener::RecvNotice_CloseConfirmationDialog` · 00452a70 — ac2-ui game_dialogs.rs ask_popup/take_popup; ac2-script host/popup.rs efun 0xB400F5 + answer_popup; ac2-client hud_game_dialogs.rs + scripts_io.rs answer_popup

### PStringBaseIter_Common<unsigned_short>
- [-] `PStringBaseIter_Common<unsigned_short>::GetSubString` · 00452ae0 · legions 00453bf0 — STL/container internals: a Rust collection replaces it

### gmCWInterface
- [-] `gmCWInterface::DumpWeenieDesc` · 00452bf0 · legions 00453d00 — admin dump-to-string / dump debug output: not in the client
- [ ] `gmCWInterface::GetName` · 00452cd0 · legions 00453de0 — needs port: interface get name; no citation in crates/
- [ ] `gmCWInterface::GetPluralName` · 00452d50 · legions 00453e60 — needs port: interface get plural name; no citation in crates/
- [~] `gmCWInterface::RequestGUN_SetStateFlags` · 00452dd0 · legions 00453ee0 — GUN-only (no Turbine GUN in this client): the script's request answers 0 without a GUN service, as RequestGUN_* does when TurbineGun::S_pTurbineGun is NULL (ac2-script host/gun.rs gun(); the page and friend requests gate the same way in ac2-client friends.rs); with a service the request is queued but the GUN answers and wire are not modelled; the Result and HearTell paths need those answers
- [~] `gmCWInterface::RequestGUN_SetLocation` · 00452e40 · legions 00453f50 — GUN-only (no Turbine GUN in this client): the script's request answers 0 without a GUN service, as RequestGUN_* does when TurbineGun::S_pTurbineGun is NULL (ac2-script host/gun.rs gun(); the page and friend requests gate the same way in ac2-client friends.rs); with a service the request is queued but the GUN answers and wire are not modelled; the Result and HearTell paths need those answers

### PStringBaseIter_Common<unsigned_short>
- [-] `PStringBaseIter_Common<unsigned_short>::GetSubString` · 00452f50 · legions 00454060 — STL/container internals: a Rust collection replaces it

### gmCWInterface
- [~] `gmCWInterface::HearTell` · 004530f0 · legions 00454200 — GUN-only (no Turbine GUN in this client): the script's request answers 0 without a GUN service, as RequestGUN_* does when TurbineGun::S_pTurbineGun is NULL (ac2-script host/gun.rs gun(); the page and friend requests gate the same way in ac2-client friends.rs); with a service the request is queued but the GUN answers and wire are not modelled; the Result and HearTell paths need those answers
- [~] `gmCWInterface::GUNEvent_HearFromRoom` · 00453320 · legions 00454430 — GUN-only (no Turbine GUN in this client): the script's request answers 0 without a GUN service, as RequestGUN_* does when TurbineGun::S_pTurbineGun is NULL (ac2-script host/gun.rs gun(); the page and friend requests gate the same way in ac2-client friends.rs); with a service the request is queued but the GUN answers and wire are not modelled; the Result and HearTell paths need those answers

### GunChatExtraInfo
- [-] `GunChatExtraInfo::StreamPack` · 004534e0 · legions 004545f0 — serialization / reference-tracking bookkeeping: Rust codecs and owned values replace it

### (free functions)
- [ ] `STREAMPACK` · 00453560 · legions 005e6f90 — no call site in the Legions wlib (scan: host::tests::efun_coverage); lower priority
- [-] `STREAMPACK_PACKOBJ` · 004535f0 · legions 00454670 — serialization / reference-tracking bookkeeping: Rust codecs and owned values replace it

### gmCWInterface
- [x] `gmCWInterface::HandleAlias` · 00453670 · legions 004546f0 — implemented: cited at crates/ac2-weenie/src/shell/mod.rs:230

### PList<PStringBase<unsigned_short>_>
- [-] `PList<PStringBase<unsigned_short>_>::Pack` · 004537f0 · legions 00454870 — STL/container internals: a Rust collection replaces it
- [-] `PList<PStringBase<unsigned_short>_>::GetPackSize` · 00453840 — STL/container internals: a Rust collection replaces it
- [-] `PList<PStringBase<unsigned_short>_>::UnPack` · 00453880 — STL/container internals: a Rust collection replaces it

### gmCWInterface
- [ ] `gmCWInterface::ExpandText` · 004539e0 · legions 00454a60 — needs port: interface expand text; no citation in crates/
- [-] `gmCWInterface::Result_GetFriendsList` · 00453b90 — admin 7148 only: Legions 7160 has no Result_GetFriendsList wrapper and no IFun 0xB40042; the list arrives in GUNEvent_FriendStart
- [x] `gmCWInterface::GUNEvent_FriendStart` · 00453bd0 · legions 00454c50 — ac2-client friends.rs gun_friend_event (list and status to the page through take_friend_events; the script IFuns run first); ac2-ui friends_panel.rs; tests friends_script_7160, friends_page_7160
- [x] `gmCWInterface::Result_GetIgnoredList` · 00453c10 — implemented: cited at crates/ac2-weenie/src/shell/state.rs:191

### `anonymous-namespace'::YesNoDialogAwakener
- [x] ``anonymous-namespace'::YesNoDialogAwakener::OnTrigger` · 00453de0 · legions 00454e60 — no timer: the question is held by GameplayDialogs popups until its answer (the awakener's keep-alive trigger has no Rust counterpart)

### gmCWInterface
- [x] `gmCWInterface::PopupYesNoDialog` · 00453e30 · legions 00454eb0 — ac2-client hud_game_dialogs.rs: questions queued in HudView::popup_backlog until the HUD dialogs exist (tests/popup_hud_7160.rs)

## `portal/client/src/action/interp/actioninterp.cpp.c` (16/23)

### ActionInterpreter
- [-] `ActionInterpreter::Shutdown` · 00478fe0 · legions 00474e20 — Rust Drop / singleton accessor / synchronous DAT load: no retail state to tear down or DDD callback
- [-] `ActionInterpreter::Get` · 00479010 · legions 00474e50 — Rust Drop / singleton accessor / synchronous DAT load: no retail state to tear down or DDD callback
- [-] `ActionInterpreter::OnDDDComplete` · 00479020 — Rust Drop / singleton accessor / synchronous DAT load: no retail state to tear down or DDD callback
- [x] `ActionInterpreter::StopAction` · 00479030 · legions 00474e70 — ac2-ui interpreter.rs (ActionHandler, StartAction/StopAction dispatch, double tap, run lock, one-shot purge) and controls.rs (focus, dispatch to movement, behaviours, camera, UI); tests in ac2-ui
- [ ] `ActionInterpreter::GetDescripValues` · 004790b0 · legions 00474ef0 — needs port: action interpreter::get descrip values; no citation in crates/
- [ ] `ActionInterpreter::SetPlayerID` · 004790d0 · legions 00474f10 — needs port: action interpreter::set player id; no citation in crates/
- [ ] `ActionInterpreter::MouseLookHandler` · 004790f0 · legions 00474f30 — needs port: action interpreter::mouse look handler; no citation in crates/
- [x] `ActionInterpreter::HandleDoubleTap` · 00479110 · legions 00474f50 — ac2-ui interpreter.rs (ActionHandler, StartAction/StopAction dispatch, double tap, run lock, one-shot purge) and controls.rs (focus, dispatch to movement, behaviours, camera, UI); tests in ac2-ui
- [x] `ActionInterpreter::FindAction` · 004791b0 · legions 00474ff0 — ac2-ui interpreter.rs (ActionHandler, StartAction/StopAction dispatch, double tap, run lock, one-shot purge) and controls.rs (focus, dispatch to movement, behaviours, camera, UI); tests in ac2-ui
- [x] `ActionInterpreter::ActionDown` · 004791e0 · legions 00475020 — ac2-ui interpreter.rs (ActionHandler, StartAction/StopAction dispatch, double tap, run lock, one-shot purge) and controls.rs (focus, dispatch to movement, behaviours, camera, UI); tests in ac2-ui
- [x] `ActionInterpreter::GetModifierBitfield` · 00479210 · legions 00475050 — ac2-ui interpreter.rs (ActionHandler, StartAction/StopAction dispatch, double tap, run lock, one-shot purge) and controls.rs (focus, dispatch to movement, behaviours, camera, UI); tests in ac2-ui
- [x] `ActionInterpreter::CtrlKeyDown` · 004792d0 · legions 00475110 — ac2-ui interpreter.rs (ActionHandler, StartAction/StopAction dispatch, double tap, run lock, one-shot purge) and controls.rs (focus, dispatch to movement, behaviours, camera, UI); tests in ac2-ui
- [x] `ActionInterpreter::ShiftKeyDown` · 004792e0 · legions 00475120 — ac2-ui interpreter.rs (ActionHandler, StartAction/StopAction dispatch, double tap, run lock, one-shot purge) and controls.rs (focus, dispatch to movement, behaviours, camera, UI); tests in ac2-ui
- [x] `ActionInterpreter::AltKeyDown` · 00479300 · legions 00475140 — ac2-ui chat.rs Modifiers: alt is never held (retail's input manager has no Alt meta key, see the chat.rs note)
- [x] `ActionInterpreter::GetActionValues` · 00479310 · legions 00475150 — ac2-ui interpreter.rs (ActionHandler, StartAction/StopAction dispatch, double tap, run lock, one-shot purge) and controls.rs (focus, dispatch to movement, behaviours, camera, UI); tests in ac2-ui
- [x] `ActionInterpreter::StartAction` · 004793d0 · legions 00475210 — ac2-ui interpreter.rs (ActionHandler, StartAction/StopAction dispatch, double tap, run lock, one-shot purge) and controls.rs (focus, dispatch to movement, behaviours, camera, UI); tests in ac2-ui
- [x] `ActionInterpreter::PurgeSetActions` · 004794c0 · legions 00475300 — ac2-ui interpreter.rs (ActionHandler, StartAction/StopAction dispatch, double tap, run lock, one-shot purge) and controls.rs (focus, dispatch to movement, behaviours, camera, UI); tests in ac2-ui
- [x] `ActionInterpreter::UseTime` · 00479530 · legions 00475370 — ac2-ui interpreter.rs (ActionHandler, StartAction/StopAction dispatch, double tap, run lock, one-shot purge) and controls.rs (focus, dispatch to movement, behaviours, camera, UI); tests in ac2-ui
- [x] `ActionInterpreter::TurnOffRunLock` · 00479540 · legions 00475380 — ac2-ui interpreter.rs (ActionHandler, StartAction/StopAction dispatch, double tap, run lock, one-shot purge) and controls.rs (focus, dispatch to movement, behaviours, camera, UI); tests in ac2-ui
- [-] `ActionInterpreter::~ActionInterpreter` · 00479790 · legions 004755d0 — Rust Drop / singleton accessor / synchronous DAT load: no retail state to tear down or DDD callback
- [x] `ActionInterpreter::Startup` · 004797f0 · legions 00475630 — ac2-ui interpreter.rs (ActionHandler, StartAction/StopAction dispatch, double tap, run lock, one-shot purge) and controls.rs (focus, dispatch to movement, behaviours, camera, UI); tests in ac2-ui
- [x] `ActionInterpreter::FocusSwitchHandler` · 00479890 · legions 004756d0 — ac2-ui interpreter.rs (ActionHandler, StartAction/StopAction dispatch, double tap, run lock, one-shot purge) and controls.rs (focus, dispatch to movement, behaviours, camera, UI); tests in ac2-ui
- [x] `ActionInterpreter::ActionHandler` · 004799c0 · legions 00475800 — ac2-ui interpreter.rs (ActionHandler, StartAction/StopAction dispatch, double tap, run lock, one-shot purge) and controls.rs (focus, dispatch to movement, behaviours, camera, UI); tests in ac2-ui

## `portal/client/src/action/map/actionmap.cpp.c` (3/16)

### ActionMap
- [ ] `ActionMap::GetMapper` · 0072d640 · legions 00613a40 — needs port: action map::get mapper; no citation in crates/

### _STL::less<ActionWithModifiers>
- [-] `_STL::less<ActionWithModifiers>::operator()` · 0072d6a0 · legions 006ab720 — STL/container internals: a Rust collection replaces it

### ActionMap
- [ ] `ActionMap::Pack` · 0072d770 — needs port: action map::pack; no citation in crates/
- [x] `ActionMap::GetToggleType` · 0072da60 · legions 006abae0 — ac2-ui maps.rs (ActionMap values by action, modifiers and toggle type, UnPack from the DAT)
- [x] `ActionMap::GetActionValues` · 0072da90 · legions 006abb10 — ac2-ui maps.rs (ActionMap values by action, modifiers and toggle type, UnPack from the DAT)
- [ ] `ActionMap::GetDescripValues` · 0072daf0 · legions 006abb70 — needs port: action map::get descrip values; no citation in crates/
- [-] `ActionMap::~ActionMap` · 0072e690 · legions 006ac710 — destructor: Rust Drop replaces it
- [-] `ActionMap::GetDBOType` · 0072e760 — serialization / reference-tracking bookkeeping: Rust codecs and owned values replace it
- [ ] `ActionMap::Destroy` · 0072e770 · legions 006ac7f0 — needs port: action map::destroy; no citation in crates/

### _STL::map<unsigned_long,unsigned_long,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>
- [-] `_STL::map<unsigned_long,unsigned_long,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>::operator[]` · 0072e800 · legions 006ac880 — STL/container internals: a Rust collection replaces it

### _STL::map<ActionWithModifiers,ActionMapValue,_STL::less<ActionWithModifiers>,_STL::allocator<_STL::pair<ActionWithModifiers_const_,ActionMapValue>_>_>
- [-] `_STL::map<ActionWithModifiers,ActionMapValue,_STL::less<ActionWithModifiers>,_STL::allocator<_STL::pair<ActionWithModifiers_const_,ActionMapValue>_>_>::operator[]` · 0072e880 · legions 006ac900 — STL/container internals: a Rust collection replaces it

### _STL::map<unsigned_long,DescripValue,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,DescripValue>_>_>
- [-] `_STL::map<unsigned_long,DescripValue,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,DescripValue>_>_>::operator[]` · 0072e900 · legions 006ac980 — STL/container internals: a Rust collection replaces it

### ActionMap
- [-] `ActionMap::ActionMap` · 0072e980 · legions 006aca00 — constructor of plain data: Rust struct init replaces it
- [-] `ActionMap::FromFileNode` · 0072ea40 — text source format (admin tools)
- [x] `ActionMap::UnPack` · 0072f200 — ac2-ui maps.rs (ActionMap values by action, modifiers and toggle type, UnPack from the DAT)
- [-] `ActionMap::Allocate` · 0072f370 — serialization / reference-tracking bookkeeping: Rust codecs and owned values replace it

## `portal/client/src/action/movement/playermovementmodule.cpp.c` (15/17)

### PlayerMovementModule
- [x] `PlayerMovementModule::Startup` · 00568e20 · legions 00538db0 — ac2-weenie movement_module.rs (UseTime: do-motion, turn, steer, jump count) and reporter.rs (position and look-at reports, tests in ac2-weenie physics_adapter); ac2-client client.rs input_handlers (DoBehavior/StopBehavior, InterruptMoveTo), camera_actions, camera_mouse.rs (MouseMove); ac2-client motion_tests
- [-] `PlayerMovementModule::Get` · 00568e90 · legions 00538e20 — Rust owns the module inside Client (no singleton accessor, no Shutdown ordering)
- [x] `PlayerMovementModule::ShouldSendPositionEvent` · 00568ea0 · legions 00538e30 — ac2-weenie movement_module.rs (UseTime: do-motion, turn, steer, jump count) and reporter.rs (position and look-at reports, tests in ac2-weenie physics_adapter); ac2-client client.rs input_handlers (DoBehavior/StopBehavior, InterruptMoveTo), camera_actions, camera_mouse.rs (MouseMove); ac2-client motion_tests
- [x] `PlayerMovementModule::SendPositionEvent` · 00569020 · legions 00538fb0 — ac2-weenie movement_module.rs (UseTime: do-motion, turn, steer, jump count) and reporter.rs (position and look-at reports, tests in ac2-weenie physics_adapter); ac2-client client.rs input_handlers (DoBehavior/StopBehavior, InterruptMoveTo), camera_actions, camera_mouse.rs (MouseMove); ac2-client motion_tests
- [x] `PlayerMovementModule::ShouldSendLookAtEvent` · 00569200 · legions 00539190 — ac2-weenie movement_module.rs (UseTime: do-motion, turn, steer, jump count) and reporter.rs (position and look-at reports, tests in ac2-weenie physics_adapter); ac2-client client.rs input_handlers (DoBehavior/StopBehavior, InterruptMoveTo), camera_actions, camera_mouse.rs (MouseMove); ac2-client motion_tests
- [x] `PlayerMovementModule::SendLookAtEvent` · 005692e0 · legions 00539270 — ac2-weenie movement_module.rs (UseTime: do-motion, turn, steer, jump count) and reporter.rs (position and look-at reports, tests in ac2-weenie physics_adapter); ac2-client client.rs input_handlers (DoBehavior/StopBehavior, InterruptMoveTo), camera_actions, camera_mouse.rs (MouseMove); ac2-client motion_tests
- [x] `PlayerMovementModule::MouseMove` · 00569380 · legions 00539310 — ac2-weenie movement_module.rs (UseTime: do-motion, turn, steer, jump count) and reporter.rs (position and look-at reports, tests in ac2-weenie physics_adapter); ac2-client client.rs input_handlers (DoBehavior/StopBehavior, InterruptMoveTo), camera_actions, camera_mouse.rs (MouseMove); ac2-client motion_tests
- [x] `PlayerMovementModule::DoBehavior` · 005693a0 · legions 00539330
- [x] `PlayerMovementModule::StopBehavior` · 005693d0 · legions 00539360 — ac2-weenie movement_module.rs (UseTime: do-motion, turn, steer, jump count) and reporter.rs (position and look-at reports, tests in ac2-weenie physics_adapter); ac2-client client.rs input_handlers (DoBehavior/StopBehavior, InterruptMoveTo), camera_actions, camera_mouse.rs (MouseMove); ac2-client motion_tests
- [x] `PlayerMovementModule::StartCameraAction` · 00569400 · legions 00539390 — ac2-weenie movement_module.rs (UseTime: do-motion, turn, steer, jump count) and reporter.rs (position and look-at reports, tests in ac2-weenie physics_adapter); ac2-client client.rs input_handlers (DoBehavior/StopBehavior, InterruptMoveTo), camera_actions, camera_mouse.rs (MouseMove); ac2-client motion_tests
- [x] `PlayerMovementModule::StopCameraAction` · 00569420 · legions 005393b0 — ac2-weenie movement_module.rs (UseTime: do-motion, turn, steer, jump count) and reporter.rs (position and look-at reports, tests in ac2-weenie physics_adapter); ac2-client client.rs input_handlers (DoBehavior/StopBehavior, InterruptMoveTo), camera_actions, camera_mouse.rs (MouseMove); ac2-client motion_tests
- [x] `PlayerMovementModule::LockActions` · 00569440 · legions 005393d0 — ac2-client client.rs lock_player_actions, set from the player's LockActions/UnlockActions animation events (client_audio animation_sounds); movement_module.rs locked; motion_tests a_locking_behaviour_on_the_player_emits_lock_events
- [x] `PlayerMovementModule::InterruptMoveTo` · 00569450 · legions 005393e0 — ac2-weenie movement_module.rs (UseTime: do-motion, turn, steer, jump count) and reporter.rs (position and look-at reports, tests in ac2-weenie physics_adapter); ac2-client client.rs input_handlers (DoBehavior/StopBehavior, InterruptMoveTo), camera_actions, camera_mouse.rs (MouseMove); ac2-client motion_tests
- [x] `PlayerMovementModule::StopAttackingIfAutoMoving` · 00569470 · legions 00539400 — PlayerMovementModule::StopAttackingIfAutoMoving (admin 00569470): CPhysicsObj vtable slot 0x54c = CPhysicsObj::GetWeenieObj (004ead90), then the weenie's slot 0xC0 = gmCWInterface::StopAttackingIfAutoMoving (admin 00451f90, objdump-confirmed; the brief's note that slot 0xC0 is GetAcceleration is the CMotionInterp vtable, not this object). Ported: ac2-ui turn keys set ControlFrame.stop_attacking_if_auto_moving, ac2-client Command::StopAttackingIfAutoMoving calls IFun 0xB400C3 (the script decides; no native auto-move state exists to check). Tests: ac2-ui a_turn_key_asks_for_the_auto_move_attack_stop, ac2-client controls a_turn_key_start..., combat_7160 stop_attacking_if_auto_moving_runs_on_the_vm
- [-] `PlayerMovementModule::Shutdown` · 005694a0 · legions 00539430 — Rust owns the module inside Client (no singleton accessor, no Shutdown ordering)
- [x] `PlayerMovementModule::SetPlayerID` · 005694f0 · legions 00539480 — ac2-weenie movement_module.rs (UseTime: do-motion, turn, steer, jump count) and reporter.rs (position and look-at reports, tests in ac2-weenie physics_adapter); ac2-client client.rs input_handlers (DoBehavior/StopBehavior, InterruptMoveTo), camera_actions, camera_mouse.rs (MouseMove); ac2-client motion_tests
- [x] `PlayerMovementModule::UseTime` · 00569550 · legions 005394e0 — ac2-weenie movement_module.rs (UseTime: do-motion, turn, steer, jump count) and reporter.rs (position and look-at reports, tests in ac2-weenie physics_adapter); ac2-client client.rs input_handlers (DoBehavior/StopBehavior, InterruptMoveTo), camera_actions, camera_mouse.rs (MouseMove); ac2-client motion_tests

## `portal/client/src/cfactory/cenginefactory.cpp.c` (0/5)

### CEngineFactory
- [-] `CEngineFactory::MakeSmartBox_Internal` · 004991f0 — engine factory: Rust builds the subsystems directly
- [-] `CEngineFactory::MakeCellManager_Internal` · 00499210 — engine factory: Rust builds the subsystems directly
- [-] `CEngineFactory::MakeFrillManager_Internal` · 00499230 — engine factory: Rust builds the subsystems directly
- [-] `CEngineFactory::MakeLScapeManager_Internal` · 00499250 — engine factory: Rust builds the subsystems directly
- [-] `CEngineFactory::MakeWorldManager_Internal` · 00499270 — engine factory: Rust builds the subsystems directly

## `portal/client/src/cfactory/cfactory.cpp.c` (0/11)

### CFactory
- [-] `CFactory::CleanUp` · 00499090 · legions 004915a0 — engine factory / singleton allocator: Rust builds and owns these directly
- [-] `CFactory::MakeCObjectMaint` · 004990b0 · legions 004915c0 — engine factory: Rust builds the subsystems directly
- [-] `CFactory::MakeClientCache` · 004990d0 · legions 004915e0 — engine factory: Rust builds the subsystems directly
- [-] `CFactory::MakeUI` · 004990f0 · legions 00491600 — engine factory: Rust builds the subsystems directly
- [-] `CFactory::MakeWorldManager` · 00499110 · legions 00491620 — engine factory: Rust builds the subsystems directly
- [-] `CFactory::MakeCellManager` · 00499130 · legions 00491640 — engine factory: Rust builds the subsystems directly
- [-] `CFactory::MakeFrillManager` · 00499150 · legions 00491660 — engine factory: Rust builds the subsystems directly
- [-] `CFactory::MakeLScapeManager` · 00499170 · legions 00491680 — engine factory: Rust builds the subsystems directly
- [-] `CFactory::MakeClient` · 00499190 · legions 004916a0 — engine factory: Rust builds the subsystems directly
- [-] `CFactory::MakeCEventQueue` · 004991b0 · legions 004916c0 — engine factory: Rust builds the subsystems directly
- [-] `CFactory::MakeGlobalEventHandler` · 004991d0 · legions 004916e0 — engine factory: Rust builds the subsystems directly

## `portal/client/src/events/ceventqueue.cpp.c` (11/15)

### CEventQueue
- [-] `CEventQueue::CEventQueue` · 00493f60 · legions 0048c730 — constructor of plain data: Rust struct init replaces it
- [x] `CEventQueue::ProcessVisualOrderedEvent` · 00493f80 · legions 0048c750 — implemented: cited at crates/ac2-weenie/src/world.rs:271
- [x] `CEventQueue::DispatchEvent` · 00494010 · legions 0048c7e0 — implemented: cited at crates/ac2-net/src/protocol/opcode.rs:3
- [x] `CEventQueue::PostEvent` · 004946a0 · legions 0048ce70 — implemented: cited at crates/ac2-net/src/protocol/mod.rs:3

### InstanceIDWithStamp
- [-] `InstanceIDWithStamp::StreamPack` · 004946d0 · legions 0048cea0 — serialization / reference-tracking bookkeeping: Rust codecs and owned values replace it

### CEventQueue
- [x] `CEventQueue::ProcessOrderedNetBlob` · 00494720 · legions 0048cef0 — implemented: cited at crates/ac2-weenie/src/world.rs:248

### NIList<NetBlob_*>
- [-] `NIList<NetBlob_*>::AddToTail` · 004947c0 · legions 0048cf90 — STL/container internals: a Rust collection replaces it

### CEventQueue
- [x] `CEventQueue::QueueEvent` · 00494810 · legions 0048cfe0 — CEventQueue queue semantics: World::queued (QueueEvent, FIFO Vec) held in EnteringWorld by World::consider_queueing (ConsiderQueueingEvent); World::finish_entering_world (FinishEnteringWorld + FlushQueue) sets InWorld then replays in arrival order; leave_world = DestroyEventQueues. Tests: world_tests entering_* and entering_queue_replays_in_arrival_order_after_create_player
- [x] `CEventQueue::ConsiderQueueingEvent` · 00494890 · legions 0048d060 — CEventQueue queue semantics: World::queued (QueueEvent, FIFO Vec) held in EnteringWorld by World::consider_queueing (ConsiderQueueingEvent); World::finish_entering_world (FinishEnteringWorld + FlushQueue) sets InWorld then replays in arrival order; leave_world = DestroyEventQueues. Tests: world_tests entering_* and entering_queue_replays_in_arrival_order_after_create_player
- [x] `CEventQueue::DestroyEventQueues` · 00494960 · legions 0048d130 — implemented: cited at crates/ac2-weenie/src/world.rs:143
- [-] `CEventQueue::~CEventQueue` · 004949f0 · legions 0048d1c0 — destructor: Rust Drop replaces it
- [x] `CEventQueue::ProcessNetBlob` · 00494a00 · legions 0048d1d0 — implemented: cited at crates/ac2-weenie/src/world.rs:235
- [x] `CEventQueue::FlushQueue` · 00494a70 · legions 0048d240 — CEventQueue queue semantics: World::queued (QueueEvent, FIFO Vec) held in EnteringWorld by World::consider_queueing (ConsiderQueueingEvent); World::finish_entering_world (FinishEnteringWorld + FlushQueue) sets InWorld then replays in arrival order; leave_world = DestroyEventQueues. Tests: world_tests entering_* and entering_queue_replays_in_arrival_order_after_create_player
- [x] `CEventQueue::FinishEnteringWorld` · 00494b00 · legions 0048d2d0 — CEventQueue queue semantics: World::queued (QueueEvent, FIFO Vec) held in EnteringWorld by World::consider_queueing (ConsiderQueueingEvent); World::finish_entering_world (FinishEnteringWorld + FlushQueue) sets InWorld then replays in arrival order; leave_world = DestroyEventQueues. Tests: world_tests entering_* and entering_queue_replays_in_arrival_order_after_create_player
- [x] `CEventQueue::UseTime` · 00494b10 · legions 0048d2e0 — implemented: cited at crates/ac2-weenie/src/world.rs:194

## `portal/client/src/events/ecm_admin.cpp.c` (0/5)

### benchScope
- [-] `benchScope::benchScope` · 00493150 — constructor of plain data: Rust struct init replaces it

### ECM_Admin
- [-] `ECM_Admin::RecvEvt_AccountBooted` · 00493190 · legions 0048bc20 — admin/debug/logging tooling: not in the client
- [-] `ECM_Admin::RecvEvt_DisplayStringInfo` · 00493200 · legions 0048bc90 — admin/debug/logging tooling: not in the client
- [-] `ECM_Admin::RecvEvt_WorldName` · 004932b0 · legions 0048bd40 — admin/debug/logging tooling: not in the client
- [-] `ECM_Admin::SendNotice_DisplayAdminStringInfo` · 00493320 — admin/debug/logging tooling: not in the client

## `portal/client/src/events/ecm_interp.cpp.c` (4/4)

### ECM_Interp
- [x] `ECM_Interp::RecvEvt_InterpCEvent_Broadcast` · 00498d60 · legions 00491380 — implemented: cited at crates/ac2-net/src/protocol/opcode.rs:60
- [x] `ECM_Interp::RecvEvt_InterpCEvent_Cell` · 00498e20 · legions 00491270 — implemented: cited at crates/ac2-net/src/protocol/opcode.rs:62
- [x] `ECM_Interp::SendEvt_InterpSEvent` · 00498f30 · legions 00491440 — implemented: cited at crates/ac2-net/src/protocol/opcode.rs:58
- [x] `ECM_Interp::SendEvt_InterpSEventEncrypt` · 00498fe0 · legions 004914f0 — implemented: cited at crates/ac2-net/src/protocol/opcode.rs:66

## `portal/client/src/events/ecm_login.cpp.c` (11/21)

### ECM_Login
- [~] `ECM_Login::RecvEvt_CharExitGame` · 004933b0 · legions 0048be00 — ac2-weenie session.rs ServerEvent::CharExitGame leaves the world and emits SessionEvent::LeftWorld (logoff_7160, logout_hud_7160); the notice fan-out FUN_0048c150 is not named or ported
- [x] `ECM_Login::RecvEvt_CharacterDeletion` · 004933c0 · legions 0048be10 — implemented: cited at crates/ac2-net/src/protocol/opcode.rs:17
- [x] `ECM_Login::RecvEvt_CharacterError` · 00493410 · legions 0048be60 — audit: decoded in ac2-net server_event/physics_event and applied in ac2-weenie objects.rs apply_action / session.rs handle_world
- [x] `ECM_Login::RecvEvt_PlayerDesc` · 00493460 · legions 0048beb0 — implemented: cited at crates/ac2-net/src/protocol/opcode.rs:31
- [x] `ECM_Login::SendEvt_ClientSceneRenderingComplete` · 00493500 · legions 0048bf50 — implemented: cited at crates/ac2-net/src/protocol/opcode.rs:35
- [x] `ECM_Login::RecvEvt_CharGenVerification` · 00493520 · legions 0048bf70 — implemented: cited at crates/ac2-net/src/protocol/opcode.rs:29

### CharacterSet
- [-] `CharacterSet::~CharacterSet` · 00493680 · legions 0048c0d0 — destructor: Rust Drop replaces it

### ECM_Login
- [x] `ECM_Login::SendNotice_CharExitGame` · 00493700 — ECM_Login::SendNotice_CharExitGame broadcasts notice 0x10405: no registrant in either build. Session LeftWorld; Proto_UI::LogOffCharacter byte test (ac2-net client_event) and logoff_7160 / logout_hud_7160
- [~] `ECM_Login::SendNotice_CharGenVerification` · 00493780 — CharGenVerification dispatches notice 0x10404 to gmGenerationConfirmationUI. Legions body of its RecvNotice_CharGenVerification is not in the decompile (admin 7148 0x0042D080 adds the identity on response 1); session AddIdentity mirrors admin. pregame_7160 char_gen_7160 tests
- [x] `ECM_Login::SendNotice_CharacterDeletion` · 00493810 — GlobalEventHandler::RecvEvt_CharacterDeletion dispatches notice 0x10406; no handler registers it in admin 7148 or Legions 7160, so retail reacts with nothing. Session emits SessionEvent::CharacterDeleted; pregame UI leaves the list alone (as retail). Request side: Proto_UI::SendDeleteCharacter byte test in ac2-net client_event senders_use_their_retail_queues_and_layouts; pregame_7160 delete_is_confirmed_then_sent
- [ ] `ECM_Login::SendNotice_CharacterError` · 004938a0 · legions 0048c230 — needs port: notice CharacterError to the UI/client; no citation in crates/
- [ ] `ECM_Login::SendNotice_CharacterSet` · 00493930 — needs port: notice CharacterSet to the UI/client; no citation in crates/
- [x] `ECM_Login::SendNotice_LoginFinished` · 004939b0 — implemented: cited at crates/ac2-weenie/src/session.rs:130
- [x] `ECM_Login::SendNotice_ServerDied` · 00493a30 — implemented: cited at crates/ac2-net/src/net_connections.rs:177
- [ ] `ECM_Login::SendNotice_WorldName` · 00493ab0 — needs port: notice WorldName to the UI/client; no citation in crates/
- [x] `ECM_Login::RecvEvt_CharacterSet` · 00493b30 · legions 0048c380 — implemented: cited at crates/ac2-net/src/protocol/opcode.rs:25

### MinimalCharacterSet
- [-] `MinimalCharacterSet::~MinimalCharacterSet` · 00493bd0 · legions 0048c420 — destructor: Rust Drop replaces it
- [-] `MinimalCharacterSet::StreamPack` · 00493c50 · legions 0048c4a0 — serialization / reference-tracking bookkeeping: Rust codecs and owned values replace it

### (free functions)
- [ ] `STREAMPACK` · 00493cb0 · legions 0064e070 — no call site in the Legions wlib (scan: host::tests::efun_coverage); lower priority

### ECM_Login
- [x] `ECM_Login::RecvEvt_MinCharSet` · 00493d30 · legions 0048c500 — implemented: cited at crates/ac2-net/src/protocol/opcode.rs:33

### PSmartArray<PStringBase<unsigned_short>_>
- [-] `PSmartArray<PStringBase<unsigned_short>_>::StreamPack` · 00493de0 — STL/container internals: a Rust collection replaces it

## `portal/client/src/events/ecm_physics.cpp.c` (44/59)

### TargetPosition
- [-] `TargetPosition::TargetPosition` · 00494c30 · legions 0048d400 — constructor of plain data: Rust struct init replaces it

### ECM_Physics
- [x] `ECM_Physics::RecvEvt_AddTargetedImpulse` · 00494cb0 · legions 0048d480 — ac2-physics physics_obj_impulse.rs add_impulse / add_targeted_impulse (velocity add, contact cleared, push from the target's position); ac2-net physics_event.rs decodes 0x10098/0x10099; ac2-client motion.rs apply_from; tests physics_obj_tests impulse cases, physics_event tests, tests/impulse_7160.rs. The contact argument is passed through as a velocity-setter note, not separated
- [x] `ECM_Physics::RecvEvt_AddTerrainModifier` · 00494e30 · legions 0048d600 — ECM_Physics terrain modifier events: decoded in ac2-net (ObjectAction), applied by ac2-client motion.rs to ac2-physics MotionInterp (add_terrain_modifier / clear_terrain_modifiers); tests in ac2-physics motion_interp_tests and ac2-client motion_tests
- [x] `ECM_Physics::RecvEvt_ClearTerrainModifiers` · 00494f30 · legions 0048d700 — ECM_Physics terrain modifier events: decoded in ac2-net (ObjectAction), applied by ac2-client motion.rs to ac2-physics MotionInterp (add_terrain_modifier / clear_terrain_modifiers); tests in ac2-physics motion_interp_tests and ac2-client motion_tests
- [x] `ECM_Physics::RecvEvt_Contain` · 00494fb0 · legions 0048d780 — ac2-weenie ObjectTable::apply_contain (position stamp newer); double world_objects.rs answer_pickup sends CONTAIN; test ac2-client/tests/world_objects_7160.rs a_ground_item_used_is_taken_into_the_main_pack
- [x] `ECM_Physics::RecvEvt_CreatePlayer` · 00495060 · legions 0048d830 — implemented: cited at crates/ac2-net/src/protocol/opcode.rs:95
- [x] `ECM_Physics::RecvEvt_DeParent` · 00495100 · legions 0048d8d0 — audit: decoded in ac2-net server_event/physics_event and applied in ac2-weenie objects.rs apply_action / session.rs handle_world
- [x] `ECM_Physics::RecvEvt_DestroyObject` · 004951f0 · legions 0048d9c0 — implemented: cited at crates/ac2-net/src/protocol/opcode.rs:139
- [x] `ECM_Physics::RecvEvt_DoFX` · 00495260 · legions 0048da30 — ac2-client motion.rs apply -> ac2-physics body.rs (set_velocity_scale, set_acceleration_scale, set_jump_scale, set_hover_height, do_fx, stop_fx; FX as StartFx/StopFx animation events for the FX sound part); ac2-net physics_event.rs decode; tests: motion_tests, physics_obj_tests, physics_event tests
- [x] `ECM_Physics::RecvEvt_DoFXScript` · 00495360 · legions 0048db30 — ac2-client motion.rs DoFxScript -> ac2-physics body do_fx_script; the tracker in ac2-anim fxscript_tracker.rs; client test fx_effects_tests.rs (sound and particle ids); rain: client rain_sounds.rs (tests rain_sounds_tests.rs)
- [x] `ECM_Physics::RecvEvt_DoMode` · 00495420 · legions 0048dbf0 — ac2-client motion.rs apply (CPhysicsObj::RecvEvt_* on the player and remote bodies); motion_tests modes_behaviours_and_layers_reach_the_body, a_death_behavior_lays_the_body_in_the_dead_mode, only_the_player_moves_to; door_lever_7160: a door's DoMode transitions (opening, closing) and a lever's DoBehavior flip (with its FX sound) play from the DAT setups' behaviour tables
- [x] `ECM_Physics::RecvEvt_DoSlider` · 004954e0 · legions 0048dcb0 — ac2-client motion.rs apply (CPhysicsObj::RecvEvt_* on the player and remote bodies); motion_tests modes_behaviours_and_layers_reach_the_body, a_death_behavior_lays_the_body_in_the_dead_mode, only_the_player_moves_to
- [x] `ECM_Physics::RecvEvt_DoStory` · 00495620 · legions 0048ddf0 — implemented: cited at crates/ac2-net/src/protocol/opcode.rs:130
- [~] `ECM_Physics::RecvEvt_DoTargettedFX` · 00495710 · legions 0048dee0 — sound branch only: the target is ignored (FXManager::DoTargettedFX sound path = untargeted DoFX); the targeted visual part is not drawn
- [x] `ECM_Physics::RecvEvt_InitializeMissile` · 00495850 · legions 0048e020 — implemented: cited at crates/ac2-net/src/protocol/opcode.rs:133
- [x] `ECM_Physics::RecvEvt_LeaveWorld` · 00495a00 · legions 0048e1d0 — audit: decoded in ac2-net server_event/physics_event and applied in ac2-weenie objects.rs apply_action / session.rs handle_world
- [x] `ECM_Physics::RecvEvt_LookAt` · 00495ac0 · legions 0048e290 — implemented: cited at crates/ac2-net/src/protocol/opcode.rs:115
- [x] `ECM_Physics::RecvEvt_LookAtDir` · 00495b80 · legions 0048e350 — implemented: cited at crates/ac2-net/src/protocol/opcode.rs:147
- [x] `ECM_Physics::RecvEvt_Parent` · 00495c80 · legions 0048e450 — audit: decoded in ac2-net server_event/physics_event and applied in ac2-weenie objects.rs apply_action / session.rs handle_world
- [x] `ECM_Physics::RecvEvt_Position` · 00495da0 · legions 0048e570 — implemented: cited at crates/ac2-net/src/protocol/opcode.rs:103
- [x] `ECM_Physics::RecvEvt_PositionCell` · 00495ea0 · legions 0048e670 — audit: decoded in ac2-net server_event/physics_event and applied in ac2-weenie objects.rs apply_action / session.rs handle_world
- [x] `ECM_Physics::RecvEvt_SetAccelerationScale` · 00495fa0 · legions 0048e770 — ac2-client motion.rs apply -> ac2-physics body.rs (set_velocity_scale, set_acceleration_scale, set_jump_scale, set_hover_height, do_fx, stop_fx; FX as StartFx/StopFx animation events for the FX sound part); ac2-net physics_event.rs decode; tests: motion_tests, physics_obj_tests, physics_event tests
- [x] `ECM_Physics::RecvEvt_SetGlobalModifier` · 00496060 · legions 0048e830 — ac2-physics body.rs set_global_modifier -> motion_interp.rs global_mod (keys the motion table lookup); ac2-client motion.rs apply; test motion_interp_tests a_global_modifier_selects_its_motion_values
- [x] `ECM_Physics::RecvEvt_SetHoverHeight` · 00496120 · legions 0048e8f0 — ac2-client motion.rs apply -> ac2-physics body.rs (set_velocity_scale, set_acceleration_scale, set_jump_scale, set_hover_height, do_fx, stop_fx; FX as StartFx/StopFx animation events for the FX sound part); ac2-net physics_event.rs decode; tests: motion_tests, physics_obj_tests, physics_event tests
- [x] `ECM_Physics::RecvEvt_SetJumpScale` · 004961e0 · legions 0048e9b0 — ac2-client motion.rs apply -> ac2-physics body.rs (set_velocity_scale, set_acceleration_scale, set_jump_scale, set_hover_height, do_fx, stop_fx; FX as StartFx/StopFx animation events for the FX sound part); ac2-net physics_event.rs decode; tests: motion_tests, physics_obj_tests, physics_event tests
- [x] `ECM_Physics::RecvEvt_SetMIDESC` · 004962a0 · legions 0048ea70 — SetMIDESC: loader thread reads the MotionInterpDesc (Request::MotionDesc, world_data motion_desc); client_objects request_motion_desc / motion_desc_loaded swap the body's table (MotionInterp::set_desc); missing DBObj keeps the table
- [x] `ECM_Physics::RecvEvt_SetMode` · 00496360 · legions 0048eb30 — ac2-client motion.rs apply (CPhysicsObj::RecvEvt_* on the player and remote bodies); motion_tests modes_behaviours_and_layers_reach_the_body, a_death_behavior_lays_the_body_in_the_dead_mode, only_the_player_moves_to; door_lever_7160: a door's DoMode transitions (opening, closing) and a lever's DoBehavior flip (with its FX sound) play from the DAT setups' behaviour tables
- [x] `ECM_Physics::RecvEvt_SetVelocityScale` · 00496420 · legions 0048ebf0 — ac2-client motion.rs apply -> ac2-physics body.rs (set_velocity_scale, set_acceleration_scale, set_jump_scale, set_hover_height, do_fx, stop_fx; FX as StartFx/StopFx animation events for the FX sound part); ac2-net physics_event.rs decode; tests: motion_tests, physics_obj_tests, physics_event tests
- [x] `ECM_Physics::RecvEvt_StopAllWeenieBehaviors` · 004964e0 · legions 0048ecb0 — ac2-client motion.rs apply (CPhysicsObj::RecvEvt_* on the player and remote bodies); motion_tests modes_behaviours_and_layers_reach_the_body, a_death_behavior_lays_the_body_in_the_dead_mode, only_the_player_moves_to
- [x] `ECM_Physics::RecvEvt_StopBehavior` · 00496560 · legions 0048ed30 — ac2-client motion.rs apply (CPhysicsObj::RecvEvt_* on the player and remote bodies); motion_tests modes_behaviours_and_layers_reach_the_body, a_death_behavior_lays_the_body_in_the_dead_mode, only_the_player_moves_to
- [x] `ECM_Physics::RecvEvt_StopFX` · 00496620 · legions 0048edf0 — ac2-client motion.rs apply -> ac2-physics body.rs (set_velocity_scale, set_acceleration_scale, set_jump_scale, set_hover_height, do_fx, stop_fx; FX as StartFx/StopFx animation events for the FX sound part); ac2-net physics_event.rs decode; tests: motion_tests, physics_obj_tests, physics_event tests
- [x] `ECM_Physics::RecvEvt_StopLayer` · 004966e0 · legions 0048eeb0 — ac2-client motion.rs apply (CPhysicsObj::RecvEvt_* on the player and remote bodies); motion_tests modes_behaviours_and_layers_reach_the_body, a_death_behavior_lays_the_body_in_the_dead_mode, only_the_player_moves_to
- [x] `ECM_Physics::RecvEvt_StopMoveToCancelsBehaviors` · 004967a0 · legions 0048ef70 — ac2-client motion.rs apply (CPhysicsObj::RecvEvt_* on the player and remote bodies); motion_tests modes_behaviours_and_layers_reach_the_body, a_death_behavior_lays_the_body_in_the_dead_mode, only_the_player_moves_to
- [x] `ECM_Physics::RecvEvt_UpdateVisualDesc` · 00496820 · legions 0048eff0 — audit: decoded in ac2-net server_event/physics_event and applied in ac2-weenie objects.rs apply_action / session.rs handle_world
- [x] `ECM_Physics::SendEvt_CForceCreate` · 00496910 · legions 0048f0e0 — implemented: cited at crates/ac2-net/src/protocol/opcode.rs:108
- [x] `ECM_Physics::SendEvt_CLookAt` · 00496980 · legions 0048f150 — implemented: cited at crates/ac2-net/src/protocol/opcode.rs:143
- [x] `ECM_Physics::SendEvt_CLookAtDir` · 004969f0 · legions 0048f1c0 — implemented: cited at crates/ac2-net/src/protocol/opcode.rs:145
- [x] `ECM_Physics::SendEvt_CPosition` · 00496a90 · legions 0048f260 — implemented: cited at crates/ac2-net/src/protocol/opcode.rs:106
- [x] `ECM_Physics::RecvEvt_AddImpulse` · 00496b00 · legions 0048f2d0 — ac2-physics physics_obj_impulse.rs add_impulse / add_targeted_impulse (velocity add, contact cleared, push from the target's position); ac2-net physics_event.rs decodes 0x10098/0x10099; ac2-client motion.rs apply_from; tests physics_obj_tests impulse cases, physics_event tests, tests/impulse_7160.rs. The contact argument is passed through as a velocity-setter note, not separated

### TargetPosition
- [-] `TargetPosition::StreamPack` · 00496c10 — serialization / reference-tracking bookkeeping: Rust codecs and owned values replace it

### ECM_Physics
- [x] `ECM_Physics::RecvEvt_DoFX_Private` · 00496cb0 · legions 0048f480 — implemented: cited at crates/ac2-net/src/protocol/opcode.rs:151
- [x] `ECM_Physics::SendNotice_BeingDeleted` · 00496d60 — CWeenieObject::ObjectBeingDestructed -> FUN_0048f530 dispatches notice 0x1040c to the weenie's handlers (radar, selection, stack, ingredient, external container, fellow). Here: WorldEvent::Destroyed -> SessionEvent::ObjectRemoved, consumed by those UI pieces. The shutdown argument is unread by the Legions handlers (radar, fellowinfo). ECM_Physics::SendNotice_BeingDeleted is the same dispatch
- [ ] `ECM_Physics::SendNotice_CreateObject` · 00496de0 — needs port: notice CreateObject to the UI/client; no citation in crates/
- [ ] `ECM_Physics::SendNotice_CreatePlayerObject` · 00496e70 — needs port: notice CreatePlayerObject to the UI/client; no citation in crates/
- [x] `ECM_Physics::SendNotice_E_LastAttacker` · 00496f00 — implemented: cited at crates/ac2-weenie/src/notice.rs:150
- [ ] `ECM_Physics::SendNotice_LocalVisualDescUpdate` · 00496f90 · legions 0048f660 — needs port: notice LocalVisualDescUpdate to the UI/client; no citation in crates/
- [ ] `ECM_Physics::SendNotice_ModeUpdate` · 00497010 — needs port: notice ModeUpdate to the UI/client; no citation in crates/
- [ ] `ECM_Physics::SendNotice_NextFrameDrawn` · 00497090 — needs port: notice NextFrameDrawn to the UI/client; no citation in crates/
- [ ] `ECM_Physics::SendNotice_PlayerReadyToEnterWorld` · 00497110 — needs port: notice PlayerReadyToEnterWorld to the UI/client; no citation in crates/
- [x] `ECM_Physics::SendNotice_PlayerTeleportStart` · 00497190 — implemented: cited at crates/ac2-client/src/client_teleport.rs:10
- [ ] `ECM_Physics::SendNotice_SetPlayerID` · 00497210 — needs port: notice SetPlayerID to the UI/client; no citation in crates/
- [ ] `ECM_Physics::SendNotice_SetRegionID` · 004972a0 — needs port: notice SetRegionID to the UI/client; no citation in crates/

### PList<Position>
- [-] `PList<Position>::Pack` · 004974c0 · legions 0048f9d0 — STL/container internals: a Rust collection replaces it
- [-] `PList<Position>::GetPackSize` · 00497510 — STL/container internals: a Rust collection replaces it
- [-] `PList<Position>::UnPack` · 00497550 · legions 0048fa74 — STL/container internals: a Rust collection replaces it

### ListNode<Position>
- [-] `ListNode<Position>::ListNode<Position>` · 004976a0 · legions 0048fbb0 — STL/container internals: a Rust collection replaces it

### ECM_Physics
- [x] `ECM_Physics::RecvEvt_MoveTo` · 00497750 · legions 0048fc60 — ac2-client motion.rs apply (CPhysicsObj::RecvEvt_* on the player and remote bodies); motion_tests modes_behaviours_and_layers_reach_the_body, a_death_behavior_lays_the_body_in_the_dead_mode, only_the_player_moves_to
- [x] `ECM_Physics::RecvEvt_ReleaseBehavior` · 004978c0 · legions 0048fdd0 — ac2-client motion.rs apply (CPhysicsObj::RecvEvt_* on the player and remote bodies); motion_tests modes_behaviours_and_layers_reach_the_body, a_death_behavior_lays_the_body_in_the_dead_mode, only_the_player_moves_to
- [x] `ECM_Physics::RecvEvt_DoBehavior` · 00497a00 · legions 0048ff10 — ac2-client motion.rs apply (CPhysicsObj::RecvEvt_* on the player and remote bodies); motion_tests modes_behaviours_and_layers_reach_the_body, a_death_behavior_lays_the_body_in_the_dead_mode, only_the_player_moves_to; door_lever_7160: a door's DoMode transitions (opening, closing) and a lever's DoBehavior flip (with its FX sound) play from the DAT setups' behaviour tables

## `portal/client/src/events/ecm_qualities.cpp.c` (20/20)

### ECM_Qualities
- [x] `ECM_Qualities::RecvEvt_UpdateBool_Private` · 00497af0 · legions 00490000 — ac2-net protocol/quality_event.rs decodes and encodes each kind (private: no object, visual: InstanceIdWithStamp first), world.rs queues visual updates to their object (GetObjectOrQueueEvent)
- [x] `ECM_Qualities::RecvEvt_UpdateBool_Visual` · 00497ba0 · legions 004900b0 — ac2-net protocol/quality_event.rs decodes and encodes each kind (private: no object, visual: InstanceIdWithStamp first), world.rs queues visual updates to their object (GetObjectOrQueueEvent)
- [x] `ECM_Qualities::RecvEvt_UpdateDataID_Private` · 00497c90 · legions 004901a0 — ac2-net protocol/quality_event.rs decodes and encodes each kind (private: no object, visual: InstanceIdWithStamp first), world.rs queues visual updates to their object (GetObjectOrQueueEvent)
- [x] `ECM_Qualities::RecvEvt_UpdateDataID_Visual` · 00497d40 · legions 00490250 — ac2-net protocol/quality_event.rs decodes and encodes each kind (private: no object, visual: InstanceIdWithStamp first), world.rs queues visual updates to their object (GetObjectOrQueueEvent)
- [x] `ECM_Qualities::RecvEvt_UpdateFloat_Private` · 00497e30 · legions 00490340 — ac2-net protocol/quality_event.rs decodes and encodes each kind (private: no object, visual: InstanceIdWithStamp first), world.rs queues visual updates to their object (GetObjectOrQueueEvent)
- [x] `ECM_Qualities::RecvEvt_UpdateFloat_Visual` · 00497ef0 · legions 00490400 — ac2-net protocol/quality_event.rs decodes and encodes each kind (private: no object, visual: InstanceIdWithStamp first), world.rs queues visual updates to their object (GetObjectOrQueueEvent)
- [x] `ECM_Qualities::RecvEvt_UpdateInstanceID_Private` · 00497ff0 · legions 00490500 — ac2-net protocol/quality_event.rs decodes and encodes each kind (private: no object, visual: InstanceIdWithStamp first), world.rs queues visual updates to their object (GetObjectOrQueueEvent)
- [x] `ECM_Qualities::RecvEvt_UpdateInstanceID_Visual` · 004980b0 · legions 004905c0 — ac2-net protocol/quality_event.rs decodes and encodes each kind (private: no object, visual: InstanceIdWithStamp first), world.rs queues visual updates to their object (GetObjectOrQueueEvent)
- [x] `ECM_Qualities::RecvEvt_UpdateInt_Private` · 004981b0 · legions 004906c0 — ac2-net protocol/quality_event.rs decodes and encodes each kind (private: no object, visual: InstanceIdWithStamp first), world.rs queues visual updates to their object (GetObjectOrQueueEvent)
- [x] `ECM_Qualities::RecvEvt_UpdateInt_Visual` · 00498260 · legions 00490770 — ac2-net protocol/quality_event.rs decodes and encodes each kind (private: no object, visual: InstanceIdWithStamp first), world.rs queues visual updates to their object (GetObjectOrQueueEvent)
- [x] `ECM_Qualities::RecvEvt_UpdateLongInt_Private` · 00498350 · legions 00490860 — ac2-net protocol/quality_event.rs decodes and encodes each kind (private: no object, visual: InstanceIdWithStamp first), world.rs queues visual updates to their object (GetObjectOrQueueEvent)
- [x] `ECM_Qualities::RecvEvt_UpdateLongInt_Visual` · 00498410 · legions 00490920 — ac2-net protocol/quality_event.rs decodes and encodes each kind (private: no object, visual: InstanceIdWithStamp first), world.rs queues visual updates to their object (GetObjectOrQueueEvent)
- [x] `ECM_Qualities::RecvEvt_UpdatePosition_Private` · 00498510 · legions 00490a20 — ac2-net protocol/quality_event.rs decodes and encodes each kind (private: no object, visual: InstanceIdWithStamp first), world.rs queues visual updates to their object (GetObjectOrQueueEvent)
- [x] `ECM_Qualities::RecvEvt_UpdatePosition_Visual` · 00498600 · legions 00490b10 — ac2-net protocol/quality_event.rs decodes and encodes each kind (private: no object, visual: InstanceIdWithStamp first), world.rs queues visual updates to their object (GetObjectOrQueueEvent)
- [x] `ECM_Qualities::RecvEvt_UpdateStringInfo_Private` · 00498740 · legions 00490c50 — ac2-net protocol/quality_event.rs decodes and encodes each kind (private: no object, visual: InstanceIdWithStamp first), world.rs queues visual updates to their object (GetObjectOrQueueEvent)
- [x] `ECM_Qualities::RecvEvt_UpdateStringInfo_Visual` · 00498800 · legions 00490d10 — ac2-net protocol/quality_event.rs decodes and encodes each kind (private: no object, visual: InstanceIdWithStamp first), world.rs queues visual updates to their object (GetObjectOrQueueEvent)
- [x] `ECM_Qualities::RecvEvt_UpdateTimestamp_Private` · 00498910 · legions 00490e20 — ac2-net protocol/quality_event.rs decodes and encodes each kind (private: no object, visual: InstanceIdWithStamp first), world.rs queues visual updates to their object (GetObjectOrQueueEvent)
- [x] `ECM_Qualities::RecvEvt_UpdateTimestamp_Visual` · 004989e0 · legions 00490ef0 — ac2-net protocol/quality_event.rs decodes and encodes each kind (private: no object, visual: InstanceIdWithStamp first), world.rs queues visual updates to their object (GetObjectOrQueueEvent)
- [x] `ECM_Qualities::RecvEvt_UpdateString_Private` · 00498af0 · legions 00491000 — ac2-net protocol/quality_event.rs decodes and encodes each kind (private: no object, visual: InstanceIdWithStamp first), world.rs queues visual updates to their object (GetObjectOrQueueEvent)
- [x] `ECM_Qualities::RecvEvt_UpdateString_Visual` · 00498c00 · legions 00491110 — ac2-net protocol/quality_event.rs decodes and encodes each kind (private: no object, visual: InstanceIdWithStamp first), world.rs queues visual updates to their object (GetObjectOrQueueEvent)

## `portal/client/src/events/globaleventhandler.cpp.c` (9/21)

### GlobalEventHandler
- [-] `GlobalEventHandler::GetGlobalEventHandler` · 00492be0 · legions 0048b590 — singleton accessor: Rust owns the handler directly
- [x] `GlobalEventHandler::RecvEvt_AccountBooted` · 00492bf0 · legions 0048b5a0 — implemented: cited at crates/ac2-weenie/src/session.rs:173
- [~] `GlobalEventHandler::RecvEvt_CharExitGame` · 00492c10 · legions 0048b5c0 — ac2-weenie session.rs ServerEvent::CharExitGame leaves the world and emits SessionEvent::LeftWorld (logoff_7160, logout_hud_7160); the notice fan-out FUN_0048c150 is not named or ported
- [x] `GlobalEventHandler::RecvEvt_CharacterSet` · 00492c20 · legions 0048b5d0 — audit: decoded in ac2-net server_event/physics_event and applied in ac2-weenie objects.rs apply_action / session.rs handle_world
- [x] `GlobalEventHandler::RecvEvt_CharacterDeletion` · 00492c40 · legions 0048b5f0 — GlobalEventHandler::RecvEvt_CharacterDeletion dispatches notice 0x10406; no handler registers it in admin 7148 or Legions 7160, so retail reacts with nothing. Session emits SessionEvent::CharacterDeleted; pregame UI leaves the list alone (as retail). Request side: Proto_UI::SendDeleteCharacter byte test in ac2-net client_event senders_use_their_retail_queues_and_layouts; pregame_7160 delete_is_confirmed_then_sent
- [x] `GlobalEventHandler::RecvEvt_CharacterError` · 00492c60 · legions 0048b610 — audit: decoded in ac2-net server_event/physics_event and applied in ac2-weenie objects.rs apply_action / session.rs handle_world
- [~] `GlobalEventHandler::RecvEvt_CharGenVerification` · 00492c90 · legions 0048b640 — CharGenVerification dispatches notice 0x10404 to gmGenerationConfirmationUI. Legions body of its RecvNotice_CharGenVerification is not in the decompile (admin 7148 0x0042D080 adds the identity on response 1); session AddIdentity mirrors admin. pregame_7160 char_gen_7160 tests
- [x] `GlobalEventHandler::RecvEvt_CreatePlayer` · 00492cb0 · legions 0048b660 — implemented: cited at crates/ac2-weenie/src/world.rs:377
- [x] `GlobalEventHandler::RecvEvt_CreateObject` · 00492ce0 · legions 0048b690 — audit: decoded in ac2-net server_event/physics_event and applied in ac2-weenie objects.rs apply_action / session.rs handle_world
- [x] `GlobalEventHandler::RecvEvt_DestroyObject` · 00492d20 · legions 0048b6d0 — audit: decoded in ac2-net server_event/physics_event and applied in ac2-weenie objects.rs apply_action / session.rs handle_world
- [x] `GlobalEventHandler::RecvEvt_DisplayStringInfo` · 00492d40 · legions 0048b6f0 — implemented: cited at crates/ac2-weenie/src/session.rs:166
- [x] `GlobalEventHandler::RecvEvt_WorldName` · 00492d90 · legions 0048b740 — implemented: cited at crates/ac2-net/src/protocol/string_info.rs:140

### LongNIHashIter<NIList<NoticeHandler_*>_>
- [-] `LongNIHashIter<NIList<NoticeHandler_*>_>::LongNIHashIter<NIList<NoticeHandler_*>_>` · 00492db0 · legions 0048b7c0 — STL/container internals: a Rust collection replaces it

### LongNIHash<NIList<NoticeHandler_*>_>
- [-] `LongNIHash<NIList<NoticeHandler_*>_>::add` · 00492e10 · legions 0048b870 — STL/container internals: a Rust collection replaces it

### GlobalEventHandler
- [-] `GlobalEventHandler::GlobalEventHandler` · 00492e60 · legions 0048b8c0 — constructor of plain data: Rust struct init replaces it
- [-] `GlobalEventHandler::GetNoticeHandlers` · 00492e80 · legions 0048b8e0 — notice-handler registry: Rust dispatches notices by enum, no handler list
- [-] `GlobalEventHandler::RegisterNoticeHandler` · 00492ec0 · legions 0048b920 — notice-handler registry: Rust dispatches notices by enum, no handler list

### LongNIHash<NIList<NoticeHandler_*>_>
- [-] `LongNIHash<NIList<NoticeHandler_*>_>::destroy_contents` · 00492fc0 · legions 0048ba90 — STL/container internals: a Rust collection replaces it

### GlobalEventHandler
- [-] `GlobalEventHandler::~GlobalEventHandler` · 00493060 · legions 0048bb30 — destructor: Rust Drop replaces it
- [-] `GlobalEventHandler::UnRegisterNoticeHandler` · 00493090 · legions 0048bb60 — notice-handler registry: Rust dispatches notices by enum, no handler list
- [-] `GlobalEventHandler::UnRegisterAllNoticeHandlers` · 004930e0 · legions 0048bbb0 — notice-handler registry: Rust dispatches notices by enum, no handler list

## `portal/client/src/events/noticehandler.cpp.c` (0/1)

### NoticeHandler
- [-] `NoticeHandler::~NoticeHandler` · 00492bb0 · legions 0048b560 — destructor: Rust Drop replaces it

## `portal/client/src/weenie/interp/glue/cefundispatcher.cpp.c` (1/1)

### CEFunDispatcher
- [x] `CEFunDispatcher::GetEFunAddress` · 0049fec0 · legions 00496f40 — implemented: cited at crates/ac2-script/src/vm/efun.rs:3

## `portal/client/src/weenie/interp/glue/cnativefactory.cpp.c` (4/268)

### PFileParser
- [-] `PFileParser::PFileParser` · 004a5eb0 · legions 0049cf30 — constructor of plain data: Rust struct init replaces it

### CNativeFactory
- [ ] `CNativeFactory::GetCheckSums` · 004a5f00 · legions 0049cf80 — needs port: cnative factory::get check sums; no citation in crates/
- [ ] `CNativeFactory::ValidateCheckSums` · 004a5f20 · legions 0049cfa0 — needs port: cnative factory::validate check sums; no citation in crates/

### BaseHashSetIter<unsigned_long,7>
- [-] `BaseHashSetIter<unsigned_long,7>::UnPack` · 004a5f40 · legions 0049cfc0 — STL/container internals: a Rust collection replaces it

### BaseMultiHashIter<unsigned_long,unsigned_long,4>
- [-] `BaseMultiHashIter<unsigned_long,unsigned_long,4>::UnPack` · 004a5f80 · legions 0049d000 — STL/container internals: a Rust collection replaces it

### BaseListIter<unsigned_long,11>
- [-] `BaseListIter<unsigned_long,11>::UnPack` · 004a5fc0 · legions 0049d040 — STL/container internals: a Rust collection replaces it

### RandomSelectionTable<unsigned_long>::ElementStruct
- [-] `RandomSelectionTable<unsigned_long>::ElementStruct::ElementStruct` · 004a6090 — constructor of plain data: Rust struct init replaces it

### CollideProperty
- [-] `CollideProperty::CollideProperty` · 004a60a0 · legions 0049d120 — constructor of plain data: Rust struct init replaces it

### ObjCollisionProfile
- [-] `ObjCollisionProfile::ObjCollisionProfile` · 004a60e0 · legions 0049d160 — constructor of plain data: Rust struct init replaces it
- [-] `ObjCollisionProfile::StreamPack` · 004a6110 · legions 0049d190 — serialization / reference-tracking bookkeeping: Rust codecs and owned values replace it
- [-] `ObjCollisionProfile::ObjCollisionProfile` · 004a6170 · legions 0049d1f0 — constructor of plain data: Rust struct init replaces it

### DetectionProfile
- [-] `DetectionProfile::DetectionProfile` · 004a61a0 · legions 0049d220 — constructor of plain data: Rust struct init replaces it

### LOSParameters
- [-] `LOSParameters::LOSParameters` · 004a6220 · legions 0049d2a0 — constructor of plain data: Rust struct init replaces it

### BaseMultiMapIter<unsigned_long,unsigned_long,13970>
- [-] `BaseMultiMapIter<unsigned_long,unsigned_long,13970>::BaseMultiMapIter<unsigned_long,unsigned_long,13970>` · 004a62f0 · legions 0049d370 — STL/container internals: a Rust collection replaces it

### BaseMultiMapIter<unsigned_long,unsigned___int64,13972>
- [-] `BaseMultiMapIter<unsigned_long,unsigned___int64,13972>::BaseMultiMapIter<unsigned_long,unsigned___int64,13972>` · 004a6320 · legions 0049d3a0 — STL/container internals: a Rust collection replaces it

### BaseListIter<unsigned_long,11>
- [-] `BaseListIter<unsigned_long,11>::BaseListIter<unsigned_long,11>` · 004a6350 · legions 0049d3d0 — STL/container internals: a Rust collection replaces it

### Sphere
- [-] `Sphere::Sphere` · 004a6380 · legions 004917a0 — constructor of plain data: Rust struct init replaces it

### BaseMultiMapIter<unsigned___int64,unsigned_long,13974>
- [-] `BaseMultiMapIter<unsigned___int64,unsigned_long,13974>::BaseMultiMapIter<unsigned___int64,unsigned_long,13974>` · 004a63a0 · legions 0049d400 — STL/container internals: a Rust collection replaces it

### BaseMultiMapIter<unsigned___int64,unsigned___int64,13976>
- [-] `BaseMultiMapIter<unsigned___int64,unsigned___int64,13976>::BaseMultiMapIter<unsigned___int64,unsigned___int64,13976>` · 004a63d0 · legions 0049d430 — STL/container internals: a Rust collection replaces it

### BaseListIter<unsigned___int64,60>
- [-] `BaseListIter<unsigned___int64,60>::BaseListIter<unsigned___int64,60>` · 004a6400 · legions 0049d460 — STL/container internals: a Rust collection replaces it

### MissileParameters
- [-] `MissileParameters::MissileParameters` · 004a6430 · legions 0049d490 — constructor of plain data: Rust struct init replaces it

### MissileTargetingParameters
- [-] `MissileTargetingParameters::MissileTargetingParameters` · 004a6550 · legions 0049d5b0 — constructor of plain data: Rust struct init replaces it

### MotionValues
- [-] `MotionValues::MotionValues` · 004a66a0 · legions 0049d700 — constructor of plain data: Rust struct init replaces it

### PathNodeData
- [-] `PathNodeData::PathNodeData` · 004a6740 · legions 0049d7a0 — constructor of plain data: Rust struct init replaces it

### BaseListIter<unsigned_long,102>
- [-] `BaseListIter<unsigned_long,102>::BaseListIter<unsigned_long,102>` · 004a6780 · legions 0049d7e0 — STL/container internals: a Rust collection replaces it

### SetPositionStruct
- [-] `SetPositionStruct::SetPositionStruct` · 004a67b0 · legions 0049d810 — constructor of plain data: Rust struct init replaces it

### CNativeFactory
- [-] `CNativeFactory::GetNativePackSize` · 004a6870 · legions 0049d8b0 — serialization / reference-tracking bookkeeping: Rust codecs and owned values replace it

### CaseInsensitiveStringBase<PStringBase<unsigned_short>_>
- [-] `CaseInsensitiveStringBase<PStringBase<unsigned_short>_>::case_insensitive_hash` · 004a6c30 · legions 0049dc70 — STL/container internals: a Rust collection replaces it

### BaseHashSetIter<unsigned_long,7>
- [-] `BaseHashSetIter<unsigned_long,7>::BaseHashSetIter<unsigned_long,7>` · 004a6e50 · legions 0049de90 — STL/container internals: a Rust collection replaces it

### BaseListIter<unsigned_long,11>
- [-] `BaseListIter<unsigned_long,11>::BaseListIter<unsigned_long,11>` · 004a6e70 · legions 0049deb0 — STL/container internals: a Rust collection replaces it

### BaseHashSetIter<unsigned___int64,11788>
- [-] `BaseHashSetIter<unsigned___int64,11788>::BaseHashSetIter<unsigned___int64,11788>` · 004a6e90 · legions 0049ded0 — STL/container internals: a Rust collection replaces it

### BaseListIter<unsigned___int64,60>
- [-] `BaseListIter<unsigned___int64,60>::BaseListIter<unsigned___int64,60>` · 004a6eb0 · legions 0049def0 — STL/container internals: a Rust collection replaces it

### BaseListIter<unsigned_long,102>
- [-] `BaseListIter<unsigned_long,102>::BaseListIter<unsigned_long,102>` · 004a6ed0 · legions 0049df10 — STL/container internals: a Rust collection replaces it

### RandomSelectionTable<unsigned_long>
- [-] `RandomSelectionTable<unsigned_long>::RandomSelectionTable<unsigned_long>` · 004a6ef0 · legions 0049df30 — constructor of plain data: Rust struct init replaces it

### BaseListIter<unsigned___int64,60>
- [-] `BaseListIter<unsigned___int64,60>::Sync` · 004a7220 · legions 0049e260 — STL/container internals: a Rust collection replaces it

### BaseListIter<unsigned_long,11>
- [-] `BaseListIter<unsigned_long,11>::Sync` · 004a7280 · legions 0049e2c0 — STL/container internals: a Rust collection replaces it

### IntrusiveHashIterator<unsigned___int64,HashSetData<unsigned___int64>_*>
- [-] `IntrusiveHashIterator<unsigned___int64,HashSetData<unsigned___int64>_*>::ComputeOffset` · 004a72e0 · legions 004a1e50 — STL/container internals: a Rust collection replaces it

### IntrusiveHashIterator<unsigned_long,HashSetData<unsigned_long>_*>
- [-] `IntrusiveHashIterator<unsigned_long,HashSetData<unsigned_long>_*>::ComputeOffset` · 004a7340 · legions 004a1ad0 — STL/container internals: a Rust collection replaces it

### CollisionProfile
- [-] `CollisionProfile::CollisionProfile` · 004a7630 · legions 0049e610 — constructor of plain data: Rust struct init replaces it

### DetectionParameters
- [-] `DetectionParameters::DetectionParameters` · 004a76c0 · legions 0049e6a0 — constructor of plain data: Rust struct init replaces it

### BaseHashIter<unsigned_long,unsigned_long,2>
- [-] `BaseHashIter<unsigned_long,unsigned_long,2>::BaseHashIter<unsigned_long,unsigned_long,2>` · 004a77b0 · legions 0049e790 — STL/container internals: a Rust collection replaces it

### BaseHashSetIter<unsigned_long,7>
- [-] `BaseHashSetIter<unsigned_long,7>::BaseHashSetIter<unsigned_long,7>` · 004a77f0 · legions 0049e7d0 — STL/container internals: a Rust collection replaces it

### BaseHashIter<unsigned_long,unsigned___int64,9>
- [-] `BaseHashIter<unsigned_long,unsigned___int64,9>::BaseHashIter<unsigned_long,unsigned___int64,9>` · 004a7830 · legions 0049e810 — STL/container internals: a Rust collection replaces it

### BaseHashIter<unsigned_long,unsigned_long,18>
- [-] `BaseHashIter<unsigned_long,unsigned_long,18>::BaseHashIter<unsigned_long,unsigned_long,18>` · 004a7870 · legions 0049e850 — STL/container internals: a Rust collection replaces it

### BaseHashIter<unsigned___int64,unsigned_long,51>
- [-] `BaseHashIter<unsigned___int64,unsigned_long,51>::BaseHashIter<unsigned___int64,unsigned_long,51>` · 004a78d0 · legions 0049e8b0 — STL/container internals: a Rust collection replaces it

### BaseHashSetIter<unsigned___int64,11788>
- [-] `BaseHashSetIter<unsigned___int64,11788>::BaseHashSetIter<unsigned___int64,11788>` · 004a7910 · legions 0049e8f0 — STL/container internals: a Rust collection replaces it

### BaseHashIter<unsigned___int64,unsigned___int64,58>
- [-] `BaseHashIter<unsigned___int64,unsigned___int64,58>::BaseHashIter<unsigned___int64,unsigned___int64,58>` · 004a7950 · legions 0049e930 — STL/container internals: a Rust collection replaces it

### BaseHashIter<unsigned___int64,unsigned_long,65>
- [-] `BaseHashIter<unsigned___int64,unsigned_long,65>::BaseHashIter<unsigned___int64,unsigned_long,65>` · 004a7990 · legions 0049e970 — STL/container internals: a Rust collection replaces it

### BaseHashIter<ReferenceIDWrapper,unsigned_long,76>
- [-] `BaseHashIter<ReferenceIDWrapper,unsigned_long,76>::BaseHashIter<ReferenceIDWrapper,unsigned_long,76>` · 004a79d0 · legions 0049e9b0 — STL/container internals: a Rust collection replaces it

### BaseHashIter<ReferenceIDWrapper,unsigned_long,78>
- [-] `BaseHashIter<ReferenceIDWrapper,unsigned_long,78>::BaseHashIter<ReferenceIDWrapper,unsigned_long,78>` · 004a7a10 · legions 0049e9f0 — STL/container internals: a Rust collection replaces it

### DBObj
- [-] `DBObj::DBObj` · 004a7a90 · legions 0049ea70 — constructor of plain data: Rust struct init replaces it
- [ ] `DBObj::Pack` · 004a7b10 — needs port: dbobj::pack; no citation in crates/
- [ ] `DBObj::UnPack` · 004a7b30 — needs port: dbobj::un pack; no citation in crates/

### CNativeFactory
- [-] `CNativeFactory::HashNativeObject` · 004a7b50 · legions 0049eb30 — serialization / reference-tracking bookkeeping: Rust codecs and owned values replace it

### CaseInsensitiveStringBase<PStringBase<unsigned_short>_>
- [-] `CaseInsensitiveStringBase<PStringBase<unsigned_short>_>::operator==` · 004a7bd0 · legions 0049ebb0 — STL/container internals: a Rust collection replaces it

### BaseHashIter<unsigned_long,unsigned_long,2>
- [-] `BaseHashIter<unsigned_long,unsigned_long,2>::BaseHashIter<unsigned_long,unsigned_long,2>` · 004a7c00 · legions 0049ebe0 — STL/container internals: a Rust collection replaces it

### BaseHashIter<unsigned_long,unsigned___int64,9>
- [-] `BaseHashIter<unsigned_long,unsigned___int64,9>::BaseHashIter<unsigned_long,unsigned___int64,9>` · 004a7c20 · legions 0049ec00 — STL/container internals: a Rust collection replaces it

### BaseHashIter<unsigned_long,unsigned_long,18>
- [-] `BaseHashIter<unsigned_long,unsigned_long,18>::BaseHashIter<unsigned_long,unsigned_long,18>` · 004a7c40 · legions 0049ec20 — STL/container internals: a Rust collection replaces it

### BaseHashIter<unsigned___int64,unsigned_long,51>
- [-] `BaseHashIter<unsigned___int64,unsigned_long,51>::BaseHashIter<unsigned___int64,unsigned_long,51>` · 004a7c60 · legions 0049ec40 — STL/container internals: a Rust collection replaces it

### BaseHashIter<unsigned___int64,unsigned___int64,58>
- [-] `BaseHashIter<unsigned___int64,unsigned___int64,58>::BaseHashIter<unsigned___int64,unsigned___int64,58>` · 004a7c80 · legions 0049ec60 — STL/container internals: a Rust collection replaces it

### BaseHashIter<unsigned___int64,unsigned_long,65>
- [-] `BaseHashIter<unsigned___int64,unsigned_long,65>::BaseHashIter<unsigned___int64,unsigned_long,65>` · 004a7ca0 · legions 0049ec80 — STL/container internals: a Rust collection replaces it

### BaseHashIter<ReferenceIDWrapper,unsigned_long,76>
- [-] `BaseHashIter<ReferenceIDWrapper,unsigned_long,76>::BaseHashIter<ReferenceIDWrapper,unsigned_long,76>` · 004a7cc0 · legions 0049eca0 — STL/container internals: a Rust collection replaces it

### BaseHashIter<ReferenceIDWrapper,unsigned_long,78>
- [-] `BaseHashIter<ReferenceIDWrapper,unsigned_long,78>::BaseHashIter<ReferenceIDWrapper,unsigned_long,78>` · 004a7ce0 · legions 0049ecc0 — STL/container internals: a Rust collection replaces it

### PSmartArray<CollideProperty>
- [-] `PSmartArray<CollideProperty>::StreamPack` · 004a7d90 · legions 0049edf0 — STL/container internals: a Rust collection replaces it

### SmartArray<InstanceID>
- [-] `SmartArray<InstanceID>::operator=` · 004a8000 · legions 0049f060 — STL/container internals: a Rust collection replaces it

### SmartArray<BaseProperty>
- [-] `SmartArray<BaseProperty>::operator=` · 004a8060 · legions 0049f0c0 — STL/container internals: a Rust collection replaces it

### SmartArray<PropertyGroup>
- [-] `SmartArray<PropertyGroup>::operator=` · 004a80d0 · legions 0049f130 — STL/container internals: a Rust collection replaces it

### SmartArray<TimeOfDay>
- [-] `SmartArray<TimeOfDay>::operator=` · 004a8140 · legions 0049f1a0 — STL/container internals: a Rust collection replaces it

### SmartArray<WeekDay>
- [-] `SmartArray<WeekDay>::operator=` · 004a81b0 · legions 0049f210 — STL/container internals: a Rust collection replaces it

### SmartArray<Season>
- [-] `SmartArray<Season>::operator=` · 004a8210 · legions 0049f270 — STL/container internals: a Rust collection replaces it

### SmartArray<unsigned___int64>
- [-] `SmartArray<unsigned___int64>::operator=` · 004a8280 · legions 0049f2e0 — STL/container internals: a Rust collection replaces it

### BaseMultiMapIter<unsigned_long,unsigned_long,13970>
- [-] `BaseMultiMapIter<unsigned_long,unsigned_long,13970>::Sync` · 004a8370 · legions 0049f3d0 — STL/container internals: a Rust collection replaces it

### SmartArray<RandomSelectionTable<unsigned_long>::ElementStruct>
- [-] `SmartArray<RandomSelectionTable<unsigned_long>::ElementStruct>::operator=` · 004a83f0 · legions 0049f450 — STL/container internals: a Rust collection replaces it

### CNativeFactory
- [ ] `CNativeFactory::IsEqual` · 004a8450 · legions 0049f4b0 — needs port: cnative factory::is equal; no citation in crates/
- [x] `CNativeFactory::GetNativeTypeName` · 004a8500 · legions 0049f540 — implemented: cited at crates/ac2-net/src/protocol/native/mod.rs:2

### HashIterator<InstanceID,CBookKeep_*>
- [-] `HashIterator<InstanceID,CBookKeep_*>::next` · 004a90a0 · legions 0042f1e0 — STL/container internals: a Rust collection replaces it

### PSmartArray<ObjCollisionProfile>
- [-] `PSmartArray<ObjCollisionProfile>::StreamPack` · 004a9230 · legions 004a0230 — STL/container internals: a Rust collection replaces it

### PSmartArray<unsigned___int64>
- [-] `PSmartArray<unsigned___int64>::StreamPack` · 004a93d0 — STL/container internals: a Rust collection replaces it

### BaseHashIter<unsigned_long,unsigned_long,2>
- [-] `BaseHashIter<unsigned_long,unsigned_long,2>::GetReferences` · 004a9470 · legions 004a05a0 — STL/container internals: a Rust collection replaces it

### BaseMultiHashIter<unsigned_long,unsigned_long,4>
- [-] `BaseMultiHashIter<unsigned_long,unsigned_long,4>::GetReferences` · 004a94c0 · legions 004a0550 — STL/container internals: a Rust collection replaces it

### BaseListIter<unsigned_long,11>
- [-] `BaseListIter<unsigned_long,11>::GetReferences` · 004a9510 · legions 004a05f0 — STL/container internals: a Rust collection replaces it

### BaseMultiMapIter<unsigned_long,unsigned_long,13970>
- [-] `BaseMultiMapIter<unsigned_long,unsigned_long,13970>::GetReferences` · 004a9560 · legions 004a04c0 — STL/container internals: a Rust collection replaces it

### BaseListIter<unsigned___int64,60>
- [-] `BaseListIter<unsigned___int64,60>::Pack` · 004a95a0 · legions 004a0500 — STL/container internals: a Rust collection replaces it

### BaseListIter<unsigned_long,11>
- [-] `BaseListIter<unsigned_long,11>::Pack` · 004a95f0 · legions 004a0470 — STL/container internals: a Rust collection replaces it

### RandomSelectionTable<unsigned_long>
- [-] `RandomSelectionTable<unsigned_long>::StreamPack` · 004a9640 · legions 004a0640 — serialization / reference-tracking bookkeeping: Rust codecs and owned values replace it

### RandomSelectionTable<unsigned_long>::ElementStruct
- [-] `RandomSelectionTable<unsigned_long>::ElementStruct::StreamPack` · 004a9760 · legions 004a0760 — serialization / reference-tracking bookkeeping: Rust codecs and owned values replace it

### BaseHashSetIter<unsigned_long,7>
- [-] `BaseHashSetIter<unsigned_long,7>::Sync` · 004a97c0 · legions 004a07c0 — STL/container internals: a Rust collection replaces it

### BaseHashSetIter<unsigned___int64,11788>
- [-] `BaseHashSetIter<unsigned___int64,11788>::Sync` · 004a9870 · legions 004a0870 — STL/container internals: a Rust collection replaces it

### LArray
- [-] `LArray::StreamPack` · 004a9920 — STL/container internals: a Rust collection replaces it

### RArray
- [-] `RArray::GetReferences` · 004a9930 · legions 004a09a0 — STL/container internals: a Rust collection replaces it

### PSmartArray<TimeOfDay>
- [-] `PSmartArray<TimeOfDay>::StreamPack` · 004a99b0 — STL/container internals: a Rust collection replaces it

### PSmartArray<WeekDay>
- [-] `PSmartArray<WeekDay>::StreamPack` · 004a9a50 — STL/container internals: a Rust collection replaces it

### PSmartArray<Season>
- [-] `PSmartArray<Season>::StreamPack` · 004a9af0 · legions 004a0a20 — STL/container internals: a Rust collection replaces it

### PSmartArray<PropertyGroup>
- [-] `PSmartArray<PropertyGroup>::StreamPack` · 004a9b90 · legions 004a0b60 — STL/container internals: a Rust collection replaces it

### PSmartArray<BaseProperty>
- [-] `PSmartArray<BaseProperty>::StreamPack` · 004a9c30 · legions 004a0c00 — STL/container internals: a Rust collection replaces it

### CNativeFactory
- [-] `CNativeFactory::GetReferences` · 004a9cd0 · legions 004a0d40 — serialization / reference-tracking bookkeeping: Rust codecs and owned values replace it
- [ ] `CNativeFactory::ValidateGlueVersion` · 004a9f90 · legions 004a1000 — needs port: cnative factory::validate glue version; no citation in crates/

### HashTable<unsigned_long,AppearanceInfo>
- [-] `HashTable<unsigned_long,AppearanceInfo>::~HashTable<unsigned_long,AppearanceInfo>` · 004aa120 · legions 004a1190 — STL/container internals: a Rust collection replaces it

### BaseArray<unsigned_long>
- [-] `BaseArray<unsigned_long>::BaseArray<unsigned_long>` · 004aa160 · legions 004a11d0 — STL/container internals: a Rust collection replaces it

### HashTable<unsigned_long,AppearanceInfo>
- [-] `HashTable<unsigned_long,AppearanceInfo>::operator=` · 004aa2e0 · legions 004a1350 — STL/container internals: a Rust collection replaces it

### HashSet<unsigned___int64>
- [-] `HashSet<unsigned___int64>::operator=` · 004aa3d0 · legions 004a1440 — STL/container internals: a Rust collection replaces it

### HashSet<unsigned_long>
- [-] `HashSet<unsigned_long>::operator=` · 004aa5a0 · legions 004a1610 — STL/container internals: a Rust collection replaces it

### HashTable<unsigned___int64,unsigned_long>
- [-] `HashTable<unsigned___int64,unsigned_long>::operator=` · 004aa680 · legions 004a17e0 — STL/container internals: a Rust collection replaces it

### HashTable<unsigned___int64,unsigned___int64>
- [-] `HashTable<unsigned___int64,unsigned___int64>::operator=` · 004aa7a0 · legions 004a1900 — STL/container internals: a Rust collection replaces it

### HashTable<ReferenceIDWrapper,unsigned_long>
- [-] `HashTable<ReferenceIDWrapper,unsigned_long>::operator=` · 004aa8c0 · legions 004a1a20 — STL/container internals: a Rust collection replaces it

### BaseMultiMapIter<unsigned_long,unsigned_long,13970>
- [-] `BaseMultiMapIter<unsigned_long,unsigned_long,13970>::UpdateReferences` · 004aa9c0 · legions 004a1ff0 — STL/container internals: a Rust collection replaces it

### BaseHashSetIter<unsigned_long,7>
- [-] `BaseHashSetIter<unsigned_long,7>::Pack` · 004aa9e0 · legions 004a1bb0 — STL/container internals: a Rust collection replaces it
- [-] `BaseHashSetIter<unsigned_long,7>::UpdateReferences` · 004aaa20 · legions 004a1bf0 — STL/container internals: a Rust collection replaces it

### BaseHashIter<unsigned_long,unsigned_long,2>
- [-] `BaseHashIter<unsigned_long,unsigned_long,2>::StreamPack` · 004aaa50 · legions 004a1c20 — STL/container internals: a Rust collection replaces it

### BaseMultiMapIter<unsigned_long,unsigned_long,13970>
- [-] `BaseMultiMapIter<unsigned_long,unsigned_long,13970>::StreamPack` · 004aab00 · legions 004a1cd0 — STL/container internals: a Rust collection replaces it

### BaseHashIter<unsigned_long,unsigned_long,2>
- [-] `BaseHashIter<unsigned_long,unsigned_long,2>::UpdateReferences` · 004aab60 — STL/container internals: a Rust collection replaces it
- [-] `BaseHashIter<unsigned_long,unsigned_long,2>::Sync` · 004aab90 · legions 004a1de0 — STL/container internals: a Rust collection replaces it

### BaseHashIter<unsigned___int64,unsigned_long,51>
- [-] `BaseHashIter<unsigned___int64,unsigned_long,51>::StreamPack` · 004aac00 — STL/container internals: a Rust collection replaces it

### BaseHashSetIter<unsigned___int64,11788>
- [-] `BaseHashSetIter<unsigned___int64,11788>::Pack` · 004aac60 · legions 004a1ee0 — STL/container internals: a Rust collection replaces it
- [-] `BaseHashSetIter<unsigned___int64,11788>::UpdateReferences` · 004aaca0 · legions 004a1f20 — STL/container internals: a Rust collection replaces it

### BaseMultiMapIter<unsigned_long,unsigned_long,13970>
- [-] `BaseMultiMapIter<unsigned_long,unsigned_long,13970>::ComputeOffset` · 004aad70 · legions 004a1d30 — STL/container internals: a Rust collection replaces it

### BaseListIter<unsigned___int64,60>
- [-] `BaseListIter<unsigned___int64,60>::UpdateReferences` · 004aadf0 · legions 004a1db0 — STL/container internals: a Rust collection replaces it

### BaseHashIter<unsigned___int64,unsigned_long,51>
- [-] `BaseHashIter<unsigned___int64,unsigned_long,51>::UpdateReferences` · 004aae20 · legions 004a1eb0 — STL/container internals: a Rust collection replaces it
- [-] `BaseHashIter<unsigned___int64,unsigned_long,51>::Sync` · 004aae50 · legions 004a20a0 — STL/container internals: a Rust collection replaces it

### BaseHashIter<ReferenceIDWrapper,unsigned_long,76>
- [-] `BaseHashIter<ReferenceIDWrapper,unsigned_long,76>::StreamPack` · 004aaec0 — STL/container internals: a Rust collection replaces it

### IntrusiveHashIterator<ReferenceIDWrapper,HashTableData<ReferenceIDWrapper,unsigned_long>_*>
- [-] `IntrusiveHashIterator<ReferenceIDWrapper,HashTableData<ReferenceIDWrapper,unsigned_long>_*>::ComputeOffset` · 004aaf20 · legions 004a2110 — STL/container internals: a Rust collection replaces it

### BaseHashIter<ReferenceIDWrapper,unsigned_long,76>
- [-] `BaseHashIter<ReferenceIDWrapper,unsigned_long,76>::UpdateReferences` · 004aaf80 · legions 004a2170 — STL/container internals: a Rust collection replaces it
- [-] `BaseHashIter<ReferenceIDWrapper,unsigned_long,76>::Sync` · 004aafb0 · legions 004a2200 — STL/container internals: a Rust collection replaces it

### BaseListIter<unsigned_long,11>
- [-] `BaseListIter<unsigned_long,11>::UpdateReferences` · 004ab020 — STL/container internals: a Rust collection replaces it

### RandomSelectionTable<unsigned_long>
- [-] `RandomSelectionTable<unsigned_long>::RandomSelectionTable<unsigned_long>` · 004ab050 · legions 004a2270 — constructor of plain data: Rust struct init replaces it

### BaseMultiHashIter<unsigned___int64,unsigned_long,53>
- [-] `BaseMultiHashIter<unsigned___int64,unsigned_long,53>::ComputeOffset` · 004ab0f0 · legions 004a2310 — STL/container internals: a Rust collection replaces it

### BaseMultiHashIter<unsigned_long,unsigned_long,4>
- [-] `BaseMultiHashIter<unsigned_long,unsigned_long,4>::ComputeOffset` · 004ab160 — STL/container internals: a Rust collection replaces it

### BaseMultiHashIter<unsigned___int64,unsigned_long,53>
- [-] `BaseMultiHashIter<unsigned___int64,unsigned_long,53>::Sync` · 004ab1d0 · legions 004a2380 — STL/container internals: a Rust collection replaces it

### List<unsigned___int64>
- [-] `List<unsigned___int64>::operator=` · 004ab240 · legions 004a23f0 — STL/container internals: a Rust collection replaces it

### RArray
- [-] `RArray::RArray` · 004ab2b0 · legions 004a2460 — STL/container internals: a Rust collection replaces it
- [-] `RArray::UpdateReferences` · 004ab2f0 · legions 004a24a0 — STL/container internals: a Rust collection replaces it

### CollisionParameters
- [-] `CollisionParameters::CollisionParameters` · 004ab350 · legions 004a2500 — constructor of plain data: Rust struct init replaces it

### GameTime
- [-] `GameTime::GameTime` · 004ab460 · legions 004a2610 — constructor of plain data: Rust struct init replaces it
- [-] `GameTime::Allocate` · 004ab620 — serialization / reference-tracking bookkeeping: Rust codecs and owned values replace it
- [-] `GameTime::Allocator` · 004ab630 — serialization / reference-tracking bookkeeping: Rust codecs and owned values replace it
- [-] `GameTime::GetDBOType` · 004ab650 — serialization / reference-tracking bookkeeping: Rust codecs and owned values replace it

### PropertyCollection
- [-] `PropertyCollection::PropertyCollection` · 004ab680 · legions 004a2830 — constructor of plain data: Rust struct init replaces it

### PropertyGroup
- [-] `PropertyGroup::PropertyGroup` · 004ab6e0 · legions 004a2890 — constructor of plain data: Rust struct init replaces it

### HashTable<unsigned_long,PStringBase<char>_>
- [-] `HashTable<unsigned_long,PStringBase<char>_>::~HashTable<unsigned_long,PStringBase<char>_>` · 004ab750 · legions 004a2900 — STL/container internals: a Rust collection replaces it
- [-] `HashTable<unsigned_long,PStringBase<char>_>::operator=` · 004ab790 · legions 004a2940 — STL/container internals: a Rust collection replaces it

### BaseHash<unsigned___int64,unsigned_long,51>
- [-] `BaseHash<unsigned___int64,unsigned_long,51>::~BaseHash<unsigned___int64,unsigned_long,51>` · 004ab8b0 · legions 004a2a60 — STL/container internals: a Rust collection replaces it

### PHashSet<unsigned_long>
- [-] `PHashSet<unsigned_long>::~PHashSet<unsigned_long>` · 004ab900 — STL/container internals: a Rust collection replaces it

### HashTable<PStringBase<unsigned_short>,Position>
- [-] `HashTable<PStringBase<unsigned_short>,Position>::operator=` · 004abc40 · legions 004a2da0 — STL/container internals: a Rust collection replaces it

### BaseMultiHashIter<unsigned_long,unsigned_long,4>
- [-] `BaseMultiHashIter<unsigned_long,unsigned_long,4>::Sync` · 004ac070 · legions 004a3240 — STL/container internals: a Rust collection replaces it

### CDBForm
- [-] `CDBForm::operator=` · 004ac0e0 · legions 004a32b0 — operator= / copy of plain data: Rust value semantics replace it
- [-] `CDBForm::~CDBForm` · 004ac150 · legions 004a3320 — destructor: Rust Drop replaces it

### EntityLinkDesc
- [-] `EntityLinkDesc::EntityLinkDesc` · 004ac1b0 · legions 004a3380 — constructor of plain data: Rust struct init replaces it

### LArray
- [-] `LArray::LArray` · 004ac240 · legions 004a3410 — STL/container internals: a Rust collection replaces it

### BaseHashSet<unsigned___int64,11788>
- [-] `BaseHashSet<unsigned___int64,11788>::~BaseHashSet<unsigned___int64,11788>` · 004ac280 · legions 004a3450 — STL/container internals: a Rust collection replaces it

### BaseHash<unsigned___int64,unsigned___int64,58>
- [-] `BaseHash<unsigned___int64,unsigned___int64,58>::~BaseHash<unsigned___int64,unsigned___int64,58>` · 004ac2d0 · legions 004a34a0 — STL/container internals: a Rust collection replaces it

### PList<IconModAtom>
- [-] `PList<IconModAtom>::Pack` · 004ac780 · legions 0047e110 — STL/container internals: a Rust collection replaces it

### PList<Ray>
- [-] `PList<Ray>::UnPack` · 004ac7d0 · legions 004a3964 — STL/container internals: a Rust collection replaces it

### BaseMultiHashIter<unsigned___int64,unsigned_long,53>
- [-] `BaseMultiHashIter<unsigned___int64,unsigned_long,53>::Pack` · 004ac990 · legions 004a3b40 — STL/container internals: a Rust collection replaces it
- [-] `BaseMultiHashIter<unsigned___int64,unsigned_long,53>::UpdateReferences` · 004ac9d0 · legions 004a31d0 — STL/container internals: a Rust collection replaces it

### BaseMultiHashIter<unsigned_long,unsigned_long,4>
- [-] `BaseMultiHashIter<unsigned_long,unsigned_long,4>::Pack` · 004acaf0 · legions 004a3c70 — STL/container internals: a Rust collection replaces it
- [-] `BaseMultiHashIter<unsigned_long,unsigned_long,4>::UpdateReferences` · 004acb30 · legions 004a3a30 — STL/container internals: a Rust collection replaces it

### Path
- [-] `Path::Path` · 004ace10 · legions 004a3f60 — constructor of plain data: Rust struct init replaces it

### CNativeFactory
- [-] `CNativeFactory::UpdateReferences` · 004acef0 · legions 004a4040 — serialization / reference-tracking bookkeeping: Rust codecs and owned values replace it
- [x] `CNativeFactory::PackNativeObject` · 004ad1d0 · legions 004a4320 — implemented: cited at crates/ac2-net/src/protocol/native/mod.rs:4

### PHashTable<unsigned_long,float>
- [-] `PHashTable<unsigned_long,float>::PHashTable<unsigned_long,float>` · 004ad780 · legions 004a48d0 — STL/container internals: a Rust collection replaces it

### PHashTable<unsigned_long,AppearanceInfo>
- [-] `PHashTable<unsigned_long,AppearanceInfo>::PHashTable<unsigned_long,AppearanceInfo>` · 004ad7c0 · legions 004a4910 — STL/container internals: a Rust collection replaces it
- [-] `PHashTable<unsigned_long,AppearanceInfo>::PHashTable<unsigned_long,AppearanceInfo>` · 004ad840 · legions 004a4990 — STL/container internals: a Rust collection replaces it
- [-] `PHashTable<unsigned_long,AppearanceInfo>::StreamPack` · 004ad880 — STL/container internals: a Rust collection replaces it

### PHashSet<unsigned___int64>
- [-] `PHashSet<unsigned___int64>::StreamPack` · 004ada80 · legions 004a4bd0 — STL/container internals: a Rust collection replaces it

### PHashTable<unsigned___int64,unsigned___int64>
- [-] `PHashTable<unsigned___int64,unsigned___int64>::StreamPack` · 004adc10 · legions 004a4d60 — STL/container internals: a Rust collection replaces it

### BaseHash<ReferenceIDWrapper,unsigned_long,78>
- [-] `BaseHash<ReferenceIDWrapper,unsigned_long,78>::~BaseHash<ReferenceIDWrapper,unsigned_long,78>` · 004ade00 · legions 004a4f50 — STL/container internals: a Rust collection replaces it

### PHashTable<ReferenceIDWrapper,unsigned_long>
- [-] `PHashTable<ReferenceIDWrapper,unsigned_long>::StreamPack` · 004ade50 — STL/container internals: a Rust collection replaces it

### ReferenceIDWrapper
- [-] `ReferenceIDWrapper::ReferenceIDWrapper` · 004ae050 · legions 004a51a0 — STL/container internals: a Rust collection replaces it

### PHashTable<PStringBase<unsigned_short>,Position>
- [-] `PHashTable<PStringBase<unsigned_short>,Position>::~PHashTable<PStringBase<unsigned_short>,Position>` · 004ae230 · legions 004a5380 — STL/container internals: a Rust collection replaces it
- [-] `PHashTable<PStringBase<unsigned_short>,Position>::StreamPack` · 004ae280 — STL/container internals: a Rust collection replaces it

### PHashSet<unsigned_long>
- [-] `PHashSet<unsigned_long>::StreamPack` · 004ae520 — STL/container internals: a Rust collection replaces it

### _STL::multimap<unsigned_long,unsigned_long,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>
- [-] `_STL::multimap<unsigned_long,unsigned_long,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>::~multimap<unsigned_long,unsigned_long,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>` · 004ae710 · legions 004a5990 — STL/container internals: a Rust collection replaces it

### _STL::multimap<unsigned_long,unsigned___int64,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned___int64>_>_>
- [-] `_STL::multimap<unsigned_long,unsigned___int64,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned___int64>_>_>::~multimap<unsigned_long,unsigned___int64,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned___int64>_>_>` · 004ae760 · legions 004a59e0 — STL/container internals: a Rust collection replaces it

### _STL::multimap<unsigned___int64,unsigned_long,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned_long>_>_>
- [-] `_STL::multimap<unsigned___int64,unsigned_long,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned_long>_>_>::~multimap<unsigned___int64,unsigned_long,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned_long>_>_>` · 004ae7b0 · legions 004a5a30 — STL/container internals: a Rust collection replaces it

### _STL::multimap<unsigned___int64,unsigned___int64,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned___int64>_>_>
- [-] `_STL::multimap<unsigned___int64,unsigned___int64,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned___int64>_>_>::~multimap<unsigned___int64,unsigned___int64,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned___int64>_>_>` · 004ae800 · legions 004a5a80 — STL/container internals: a Rust collection replaces it

### CDBForm
- [-] `CDBForm::CDBForm` · 004af160 · legions 004a63e0 — constructor of plain data: Rust struct init replaces it
- [-] `CDBForm::CDBForm` · 004af1b0 · legions 004a6430 — constructor of plain data: Rust struct init replaces it

### Path
- [-] `Path::~Path` · 004af220 · legions 004a64a0 — destructor: Rust Drop replaces it

### PlacesTable
- [-] `PlacesTable::PlacesTable` · 004af270 · legions 004a64f0 — constructor of plain data: Rust struct init replaces it
- [-] `PlacesTable::GetDBOType` · 004af2b0 — serialization / reference-tracking bookkeeping: Rust codecs and owned values replace it

### RList
- [-] `RList::RList` · 004af2c0 · legions 004a6540 — STL/container internals: a Rust collection replaces it

### BaseHashSet<unsigned_long,7>
- [-] `BaseHashSet<unsigned_long,7>::BaseHashSet<unsigned_long,7>` · 004af300 · legions 004a6580 — STL/container internals: a Rust collection replaces it

### BehaviorParams
- [-] `BehaviorParams::BehaviorParams` · 004af380 · legions 004a6600 — constructor of plain data: Rust struct init replaces it

### LList
- [-] `LList::LList` · 004af4b0 · legions 004a6730 — STL/container internals: a Rust collection replaces it

### Path
- [-] `Path::Path` · 004af4f0 · legions 004a6770 — constructor of plain data: Rust struct init replaces it

### PlacesTable
- [-] `PlacesTable::PlacesTable` · 004af600 · legions 004a6880 — constructor of plain data: Rust struct init replaces it

### BaseHash<unsigned_long,unsigned_long,2>
- [-] `BaseHash<unsigned_long,unsigned_long,2>::BaseHash<unsigned_long,unsigned_long,2>` · 004af660 · legions 004a68e0 — STL/container internals: a Rust collection replaces it
- [-] `BaseHash<unsigned_long,unsigned_long,2>::BaseHash<unsigned_long,unsigned_long,2>` · 004af6d0 · legions 004a6950 — STL/container internals: a Rust collection replaces it

### BaseMultiMap<unsigned_long,unsigned_long,13970>
- [-] `BaseMultiMap<unsigned_long,unsigned_long,13970>::BaseMultiMap<unsigned_long,unsigned_long,13970>` · 004af710 · legions 004a6990 — STL/container internals: a Rust collection replaces it
- [-] `BaseMultiMap<unsigned_long,unsigned_long,13970>::GetReferences` · 004af760 — STL/container internals: a Rust collection replaces it

### BaseHashSet<unsigned_long,7>
- [-] `BaseHashSet<unsigned_long,7>::BaseHashSet<unsigned_long,7>` · 004af770 · legions 004a69f0 — STL/container internals: a Rust collection replaces it

### BaseMultiMap<unsigned_long,unsigned___int64,13972>
- [-] `BaseMultiMap<unsigned_long,unsigned___int64,13972>::BaseMultiMap<unsigned_long,unsigned___int64,13972>` · 004af7a0 · legions 004a6a20 — STL/container internals: a Rust collection replaces it

### BaseMultiMap<unsigned___int64,unsigned_long,13974>
- [-] `BaseMultiMap<unsigned___int64,unsigned_long,13974>::BaseMultiMap<unsigned___int64,unsigned_long,13974>` · 004af7f0 · legions 004a6a70 — STL/container internals: a Rust collection replaces it

### BaseMultiMap<unsigned___int64,unsigned___int64,13976>
- [-] `BaseMultiMap<unsigned___int64,unsigned___int64,13976>::BaseMultiMap<unsigned___int64,unsigned___int64,13976>` · 004af840 · legions 004a6ac0 — STL/container internals: a Rust collection replaces it

### BaseMultiMap<unsigned_long,unsigned_long,13970>
- [-] `BaseMultiMap<unsigned_long,unsigned_long,13970>::StreamPack` · 004af9c0 — STL/container internals: a Rust collection replaces it

### (free functions)
- [ ] `STREAMPACK` · 004afaf0 · legions 00409e80 — no call site in the Legions wlib (scan: host::tests::efun_coverage); lower priority

### BaseMultiMap<unsigned_long,unsigned___int64,13972>
- [-] `BaseMultiMap<unsigned_long,unsigned___int64,13972>::StreamPack` · 004afc80 — STL/container internals: a Rust collection replaces it

### BaseMultiMap<unsigned___int64,unsigned_long,13974>
- [-] `BaseMultiMap<unsigned___int64,unsigned_long,13974>::StreamPack` · 004afec0 — STL/container internals: a Rust collection replaces it

### BaseMultiMap<unsigned___int64,unsigned___int64,13976>
- [-] `BaseMultiMap<unsigned___int64,unsigned___int64,13976>::StreamPack` · 004b0130 — STL/container internals: a Rust collection replaces it

### ARHash
- [-] `ARHash::ARHash` · 004b03b0 · legions 004a75a0 — constructor of plain data: Rust struct init replaces it

### LRHash
- [-] `LRHash::LRHash` · 004b03f0 · legions 004a75e0 — STL/container internals: a Rust collection replaces it
- [-] `LRHash::GetReferences` · 004b0430 — STL/container internals: a Rust collection replaces it

### BaseHash<unsigned___int64,unsigned_long,65>
- [-] `BaseHash<unsigned___int64,unsigned_long,65>::AddDataAsReferences` · 004b0450 · legions 004a7a60 — STL/container internals: a Rust collection replaces it

### LRHash
- [-] `LRHash::UpdateReferences` · 004b04d0 — STL/container internals: a Rust collection replaces it

### BaseHash<unsigned___int64,unsigned_long,65>
- [-] `BaseHash<unsigned___int64,unsigned_long,65>::UpdateData` · 004b04f0 — STL/container internals: a Rust collection replaces it

### LRHash
- [-] `LRHash::LRHash` · 004b05b0 · legions 004a77a0 — STL/container internals: a Rust collection replaces it

### NAHash
- [-] `NAHash::NAHash` · 004b05f0 · legions 004a77e0 — STL/container internals: a Rust collection replaces it
- [-] `NAHash::GetReferences` · 004b0630 — STL/container internals: a Rust collection replaces it
- [-] `NAHash::UpdateReferences` · 004b0650 — STL/container internals: a Rust collection replaces it
- [-] `NAHash::NAHash` · 004b06a0 · legions 004a7930 — STL/container internals: a Rust collection replaces it

### NRHash
- [-] `NRHash::NRHash` · 004b06e0 · legions 004a7970 — STL/container internals: a Rust collection replaces it
- [-] `NRHash::GetReferences` · 004b0720 — STL/container internals: a Rust collection replaces it

### BaseHash<ReferenceIDWrapper,unsigned_long,76>
- [-] `BaseHash<ReferenceIDWrapper,unsigned_long,76>::AddKeysAsReferences` · 004b0740 · legions 004a79d0 — STL/container internals: a Rust collection replaces it

### BaseHash<ReferenceIDWrapper,unsigned_long,78>
- [-] `BaseHash<ReferenceIDWrapper,unsigned_long,78>::AddDataAsReferences` · 004b07d0 — STL/container internals: a Rust collection replaces it

### NRHash
- [-] `NRHash::UpdateReferences` · 004b0850 — STL/container internals: a Rust collection replaces it

### BaseHash<ReferenceIDWrapper,unsigned_long,78>
- [-] `BaseHash<ReferenceIDWrapper,unsigned_long,78>::UpdateData` · 004b0880 — STL/container internals: a Rust collection replaces it

### (free functions)
- [-] `UpdateReferenceIDKeys` · 004b0910 — serialization / reference-tracking bookkeeping: Rust codecs and owned values replace it

### NRHash
- [-] `NRHash::NRHash` · 004b09e0 · legions 004a7bd0 — STL/container internals: a Rust collection replaces it

### PlacesTable
- [-] `PlacesTable::Allocate` · 004b0a20 — serialization / reference-tracking bookkeeping: Rust codecs and owned values replace it
- [-] `PlacesTable::Allocator` · 004b0a30 — serialization / reference-tracking bookkeeping: Rust codecs and owned values replace it

### LAHashSet
- [-] `LAHashSet::LAHashSet` · 004b0ab0 · legions 004a7ca0 — STL/container internals: a Rust collection replaces it
- [-] `LAHashSet::StreamPack` · 004b0af0 — STL/container internals: a Rust collection replaces it

### LLHash
- [-] `LLHash::LLHash` · 004b0b20 · legions 004a7d10 — STL/container internals: a Rust collection replaces it
- [-] `LLHash::StreamPack` · 004b0b60 — STL/container internals: a Rust collection replaces it

### ALHash
- [-] `ALHash::ALHash` · 004b0bf0 · legions 004a7de0 — STL/container internals: a Rust collection replaces it

### LAHash
- [-] `LAHash::LAHash` · 004b0c30 · legions 004a7e20 — STL/container internals: a Rust collection replaces it

### LAHashSet
- [-] `LAHashSet::LAHashSet` · 004b0c70 · legions 004a7e60 — STL/container internals: a Rust collection replaces it

### LLHash
- [-] `LLHash::LLHash` · 004b0cb0 · legions 004a7ea0 — STL/container internals: a Rust collection replaces it

### QualifiedDataIDArray
- [-] `QualifiedDataIDArray::QualifiedDataIDArray` · 004b0cf0 · legions 004a7ee0 — constructor of plain data: Rust struct init replaces it

### BaseMultiMap<unsigned_long,unsigned_long,13970>
- [-] `BaseMultiMap<unsigned_long,unsigned_long,13970>::BaseMultiMap<unsigned_long,unsigned_long,13970>` · 004b0d20 · legions 004a7f10 — STL/container internals: a Rust collection replaces it

### BaseMultiMap<unsigned_long,unsigned___int64,13972>
- [-] `BaseMultiMap<unsigned_long,unsigned___int64,13972>::BaseMultiMap<unsigned_long,unsigned___int64,13972>` · 004b0d50 · legions 004a7f40 — STL/container internals: a Rust collection replaces it

### BaseMultiMap<unsigned___int64,unsigned_long,13974>
- [-] `BaseMultiMap<unsigned___int64,unsigned_long,13974>::BaseMultiMap<unsigned___int64,unsigned_long,13974>` · 004b0d80 · legions 004a7f70 — STL/container internals: a Rust collection replaces it

### BaseMultiMap<unsigned___int64,unsigned___int64,13976>
- [-] `BaseMultiMap<unsigned___int64,unsigned___int64,13976>::BaseMultiMap<unsigned___int64,unsigned___int64,13976>` · 004b0db0 · legions 004a7fa0 — STL/container internals: a Rust collection replaces it

### BaseMultiHash<unsigned_long,unsigned___int64,13>
- [-] `BaseMultiHash<unsigned_long,unsigned___int64,13>::Pack` · 004b0e80 — STL/container internals: a Rust collection replaces it
- [-] `BaseMultiHash<unsigned_long,unsigned___int64,13>::UnPack` · 004b0f40 · legions 004a8130 — STL/container internals: a Rust collection replaces it

### BaseMultiHash<unsigned_long,unsigned_long,20>
- [-] `BaseMultiHash<unsigned_long,unsigned_long,20>::UnPack` · 004b1270 · legions 004a8520 — STL/container internals: a Rust collection replaces it

### BaseMultiHash<unsigned___int64,unsigned_long,53>
- [-] `BaseMultiHash<unsigned___int64,unsigned_long,53>::UnPack` · 004b1530 · legions 004a87e0 — STL/container internals: a Rust collection replaces it

### BaseMultiHash<unsigned___int64,unsigned___int64,62>
- [-] `BaseMultiHash<unsigned___int64,unsigned___int64,62>::GetPackSize` · 004b1700 — STL/container internals: a Rust collection replaces it
- [-] `BaseMultiHash<unsigned___int64,unsigned___int64,62>::Pack` · 004b1770 — STL/container internals: a Rust collection replaces it
- [-] `BaseMultiHash<unsigned___int64,unsigned___int64,62>::UnPack` · 004b1830 · legions 004a8ae0 — STL/container internals: a Rust collection replaces it

### BaseMultiHash<unsigned_long,unsigned___int64,13>
- [-] `BaseMultiHash<unsigned_long,unsigned___int64,13>::GetPackSize` · 004b1af0 · legions 004a8da0 — STL/container internals: a Rust collection replaces it

### BaseMultiHash<unsigned___int64,unsigned_long,53>
- [-] `BaseMultiHash<unsigned___int64,unsigned_long,53>::Pack` · 004b1b60 — STL/container internals: a Rust collection replaces it

### BaseMultiHash<unsigned_long,unsigned_long,4>
- [-] `BaseMultiHash<unsigned_long,unsigned_long,4>::BaseMultiHash<unsigned_long,unsigned_long,4>` · 004b1c20 · legions 004a8ed0 — STL/container internals: a Rust collection replaces it

### BaseMultiHash<unsigned_long,unsigned_long,20>
- [-] `BaseMultiHash<unsigned_long,unsigned_long,20>::GetPackSize` · 004b1c70 · legions 004a8f20 — STL/container internals: a Rust collection replaces it
- [-] `BaseMultiHash<unsigned_long,unsigned_long,20>::Pack` · 004b1ce0 · legions 004a8460 — STL/container internals: a Rust collection replaces it

### BaseMultiHash<unsigned_long,unsigned_long,4>
- [-] `BaseMultiHash<unsigned_long,unsigned_long,4>::UnPack` · 004b1da0 · legions 004a8f90 — STL/container internals: a Rust collection replaces it

### CNativeFactory
- [-] `CNativeFactory::DeleteNativeObject` · 004b1e80 · legions 004a9070 — copy / delete of plain native data: Rust clone and Drop replace it

### ALMultiHash
- [-] `ALMultiHash::ALMultiHash` · 004b2300 · legions 004a94f0 — STL/container internals: a Rust collection replaces it
- [-] `ALMultiHash::UnPack` · 004b2350 — STL/container internals: a Rust collection replaces it

### ARMultiHash
- [-] `ARMultiHash::ARMultiHash` · 004b2360 · legions 004a9550 — STL/container internals: a Rust collection replaces it
- [-] `ARMultiHash::GetReferences` · 004b23b0 — STL/container internals: a Rust collection replaces it
- [-] `ARMultiHash::UpdateReferences` · 004b2480 — STL/container internals: a Rust collection replaces it

### LAMultiHash
- [-] `LAMultiHash::LAMultiHash` · 004b2530 · legions 004a96d0 — STL/container internals: a Rust collection replaces it

### LLMultiHash
- [-] `LLMultiHash::LLMultiHash` · 004b2580 · legions 004a9720 — STL/container internals: a Rust collection replaces it
- [-] `LLMultiHash::UnPack` · 004b25d0 — STL/container internals: a Rust collection replaces it

### LRMultiHash
- [-] `LRMultiHash::LRMultiHash` · 004b25e0 · legions 004a9780 — STL/container internals: a Rust collection replaces it
- [-] `LRMultiHash::GetReferences` · 004b2630 — STL/container internals: a Rust collection replaces it
- [-] `LRMultiHash::UpdateReferences` · 004b26b0 — STL/container internals: a Rust collection replaces it

### LAMultiHash
- [-] `LAMultiHash::UnPack` · 004b2760 — STL/container internals: a Rust collection replaces it

### BaseMultiHash<unsigned_long,unsigned_long,4>
- [-] `BaseMultiHash<unsigned_long,unsigned_long,4>::BaseMultiHash<unsigned_long,unsigned_long,4>` · 004b2770 · legions 004a9960 — STL/container internals: a Rust collection replaces it

### ARMultiHash
- [-] `ARMultiHash::ARMultiHash` · 004b27b0 · legions 004a99a0 — STL/container internals: a Rust collection replaces it

### LRMultiHash
- [-] `LRMultiHash::LRMultiHash` · 004b27f0 · legions 004a99e0 — STL/container internals: a Rust collection replaces it

### ALMultiHash
- [-] `ALMultiHash::ALMultiHash` · 004b2830 · legions 004a9a20 — STL/container internals: a Rust collection replaces it

### LAMultiHash
- [-] `LAMultiHash::LAMultiHash` · 004b2870 · legions 004a9a60 — STL/container internals: a Rust collection replaces it

### LLMultiHash
- [-] `LLMultiHash::LLMultiHash` · 004b28b0 · legions 004a9aa0 — STL/container internals: a Rust collection replaces it

### CNativeFactory
- [-] `CNativeFactory::CopyNativeObject` · 004b28f0 · legions 004a9ae0 — copy / delete of plain native data: Rust clone and Drop replace it
- [x] `CNativeFactory::NewNativeObject` · 004b3980 · legions 004aab70 — implemented: cited at crates/ac2-script/src/native/mod.rs:257
- [x] `CNativeFactory::UnPackNativeObject` · 004b4720 · legions 004ab910 — implemented: cited at crates/ac2-script/src/native/mod.rs:5

## `portal/client/src/weenie/interp/glue/gcwallocator.cpp.c` (0/1)

### gCWAllocator
- [ ] `gCWAllocator::CallEFun_ResetShell` · 004bc8c0 — needs port: script efun reset shell on the allocator; no citation in crates/

## `portal/client/src/weenie/interp/glue/gcwbookkeeper.cpp.c` (0/2)

### gCWBookKeeper
- [ ] `gCWBookKeeper::CallEFun_AddObjectToBeDestroyed` · 004bc7e0 — needs port: script efun add object to be destroyed on the book keeper; no citation in crates/
- [ ] `gCWBookKeeper::CallEFun_RemoveObjectToBeDestroyed` · 004bc850 — needs port: script efun remove object to be destroyed on the book keeper; no citation in crates/

## `portal/client/src/weenie/interp/glue/gcwdistributor.cpp.c` (2/3)

### gCWDistributor
- [x] `gCWDistributor::CallEFun_GetEntityDID` · 004bc690 — lead: ac2-script host/glue.rs get_entity_did: the DID the weenie was made with (CWDistributor::m_initData.m_did, +0x14), tag 4; null this crashes; host tests.
- [x] `gCWDistributor::CallEFun_GetMyPackageID` · 004bc700 — ac2-script glue::engine_misc get_my_package_id
- [~] `gCWDistributor::CallEFun_GetProcessID` · 004bc770 — ac2-script glue::engine_misc: process id numbered in weenie creation order from 1; the singleton processes retail registers first are not modelled

## `portal/client/src/weenie/interp/glue/gcwinterface.cpp.c` (9/32)

### gCWInterface
- [ ] `gCWInterface::Resume_Sleep` · 004bb120 · legions 004b0340 — needs port: resume continuation of latent sleep (the script waits and resumes here); no citation in crates/
- [ ] `gCWInterface::CallEFun_DisplayHelpText` · 004bb200 — needs port: script efun display help text on the interface; no citation in crates/
- [ ] `gCWInterface::CallEFun_Sleep` · 004bb260 — needs port: script efun sleep on the interface; no citation in crates/
- [x] `gCWInterface::CallEFun_AddShellCmd` · 004bb370 — ac2-script glue::engine_misc add_shell (Vm::shell table: words split at ;:, lower-cased, trimmed; the chat shell reads Vm::shell)
- [x] `gCWInterface::CallEFun_AddShellCmdByStringID` · 004bb3f0 — ac2-script glue::engine_misc add_shell (Vm::shell table: words split at ;:, lower-cased, trimmed; the chat shell reads Vm::shell)
- [x] `gCWInterface::CallEFun_AddShellHelp` · 004bb480 — ac2-script glue::engine_misc add_shell (Vm::shell table: words split at ;:, lower-cased, trimmed; the chat shell reads Vm::shell)
- [x] `gCWInterface::CallEFun_AddShellHelpByStringID` · 004bb500 — ac2-script glue::engine_misc add_shell (Vm::shell table: words split at ;:, lower-cased, trimmed; the chat shell reads Vm::shell)
- [ ] `gCWInterface::CallEFun_GetExternalWeenieObject` · 004bb580 — needs port: script efun get external weenie object on the interface; no citation in crates/
- [ ] `gCWInterface::CallEFun_GetInternalWeenieObject` · 004bb600 — needs port: script efun get internal weenie object on the interface; no citation in crates/
- [ ] `gCWInterface::CallEFun_InqStringInfoValue` · 004bb680 — needs port: script efun inq string info value on the interface; no citation in crates/
- [ ] `gCWInterface::CallEFun_IsThePlayer` · 004bb7a0 — needs port: script efun is the player on the interface; no citation in crates/
- [x] `gCWInterface::CallEFun_ProcessText` · 004bb810 — implemented: cited at crates/ac2-script/src/host/interface.rs:226
- [x] `gCWInterface::CallEFun_SendServerEvt` · 004bb890 — implemented: cited at crates/ac2-script/src/host/glue.rs:139
- [ ] `gCWInterface::CallEFun_SendServerEvtEncrypt` · 004bb910 — needs port: script efun send server evt encrypt on the interface; no citation in crates/
- [~] `gCWInterface::CallEFun_GetMyPosition` · 004bb990 — player position set each frame by the client (Client::sync_script_clock); vtable slot assumed to be the player's position
- [ ] `gCWInterface::CallIFun_ApplyProperties` · 004bba90 · legions 004b09b0 — needs port: script IFun apply properties on the client interface; no citation in crates/
- [ ] `gCWInterface::CallIFun_DisplayStringInfo` · 004bbb80 · legions 004b0aa0 — needs port: script IFun display string info on the client interface; no citation in crates/
- [ ] `gCWInterface::CallIFun_DisplayText` · 004bbc80 · legions 004b0ba0 — needs port: script IFun display text on the client interface; no citation in crates/
- [ ] `gCWInterface::CallEFun_GetOutputFilesPath` · 004bbd90 · legions 004b0cb0 — needs port: script efun get output files path on the interface; no citation in crates/
- [ ] `gCWInterface::CallIFun_HandleCharacterSessionEnd` · 004bbe30 · legions 004b0d20 — needs port: script IFun handle character session end on the client interface; no citation in crates/
- [ ] `gCWInterface::CallIFun_HandleCharacterSessionStart` · 004bbec0 · legions 004b0db0 — needs port: script IFun handle character session start on the client interface; no citation in crates/
- [~] `gCWInterface::CallIFun_HandleCollision` · 004bbf50 · legions 004b0e40 — impact reports only: a landing impact (normal and speed, taken once by ac2-physics take_impact_report) is handed to the player's HandleCollision IFun (ac2-client collision_reported); object, environment, surface and water properties, begin/end reports and missiles are not ported
- [ ] `gCWInterface::CallIFun_HandleCreation` · 004bc030 · legions 004b0f20 — needs port: script IFun handle creation on the client interface; no citation in crates/
- [ ] `gCWInterface::CallIFun_HandleDestruction` · 004bc0c0 · legions 004b0fb0 — needs port: script IFun handle destruction on the client interface; no citation in crates/
- [ ] `gCWInterface::CallIFun_HandleEnterWorld` · 004bc150 · legions 004b1040 — needs port: script IFun handle enter world on the client interface; no citation in crates/
- [ ] `gCWInterface::CallIFun_HandleLeaveWorld` · 004bc1e0 · legions 004b10d0 — needs port: script IFun handle leave world on the client interface; no citation in crates/
- [ ] `gCWInterface::CallIFun_HandlePostLeaveWorld` · 004bc270 · legions 004b1160 — needs port: script IFun handle post leave world on the client interface; no citation in crates/
- [x] `gCWInterface::CallIFun_HandleTeleportEnd` · 004bc300 · legions 004b11f0 — implemented: cited at crates/ac2-client/src/client_teleport.rs:29
- [x] `gCWInterface::CallIFun_HandleTeleportStart` · 004bc3b0 · legions 004b12a0 — implemented: cited at crates/ac2-client/src/client_teleport.rs:12
- [ ] `gCWInterface::Resume_InqStringInfoValue` · 004bc440 · legions 004b1330 — needs port: resume continuation of latent inq string info value (the script waits and resumes here); no citation in crates/
- [x] `gCWInterface::CallIFun_OnStatChanged` · 004bc510 · legions 004b1400 — implemented: cited at crates/ac2-weenie/src/session.rs:82
- [~] `gCWInterface::CallIFun_PreloadWSLSingletons` · 004bc600 · legions 004b14f0 — retail's script-driven singleton warm-up at interface start; not ported: singletons fill on first use (ReferenceTable::GetSingleton) and their WStates are preloaded as the 0x70000000 range. Its vtable caller is not identified in the decompile

## `portal/client/src/weenie/interp/glue/gcwphysicsobject.cpp.c` (15/85)

### gCWPhysicsObject
- [ ] `gCWPhysicsObject::CallEFun_DoStory` · 004b8050 — needs port: script efun do story on the physics object; no citation in crates/
- [x] `gCWPhysicsObject::CallEFun_SetAccelerationScale` · 004b80b0 — ac2-script host/interface.rs queues the server event (vm.physics) for the weenie; ac2-client scripts_io apply_script_physics; host/tests physics_object_efuns_queue_their_server_events_for_the_weenie
- [x] `gCWPhysicsObject::CallEFun_SetJumpScale` · 004b8110 — ac2-script host/interface.rs queues the server event (vm.physics) for the weenie; ac2-client scripts_io apply_script_physics; host/tests physics_object_efuns_queue_their_server_events_for_the_weenie
- [x] `gCWPhysicsObject::CallEFun_SetVelocityScale` · 004b8170 — ac2-script host/interface.rs queues the server event (vm.physics) for the weenie; ac2-client scripts_io apply_script_physics; host/tests physics_object_efuns_queue_their_server_events_for_the_weenie
- [~] `gCWPhysicsObject::CallEFun_StopAllCameraFX` · 004b81d0 — ac2-client client_story.rs: story slot routed; drawing, audio and camera FX are known gaps
- [ ] `gCWPhysicsObject::CallEFun_AddImpulse` · 004b8270 — needs port: script efun add impulse on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_AddImpulseLocal` · 004b82f0 — needs port: script efun add impulse local on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_AddTargetedImpulse` · 004b8370 · legions 004ae2a0 — needs port: script efun add targeted impulse on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_ApplyAdditiveVisualDesc` · 004b8410 — needs port: script efun apply additive visual desc on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_ApplySubtractiveVisualDesc` · 004b8490 — needs port: script efun apply subtractive visual desc on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_ApplyVisualDescToPhysicsObj` · 004b8510 — needs port: script efun apply visual desc to physics obj on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_CalculateDistance` · 004b8580 — needs port: script efun calculate distance on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_CombineAndApplyVisualDesc` · 004b8600 — needs port: script efun combine and apply visual desc on the physics object; no citation in crates/
- [x] `gCWPhysicsObject::CallEFun_DoBehavior` · 004b8680 — Latent DoMode/DoBehavior: the efun queues the start with its context (ac2-script latent.rs, interface.rs do_mode/do_behavior: Interrupted until the body reports done); the body's ModeDone/BehaviorDone(context, error) resumes it through Resume_DoMode/Resume_DoBehavior (ScriptHost::resume_latent: out param = error). Client: client_latent.rs, client_audio.rs animation_sounds. Tests: ac2-script host tests/physics.rs a_latent_mode_resumes_the_context_when_the_body_reports_it_done, physics_mode_and_behavior_efuns_queue_their_server_events
- [ ] `gCWPhysicsObject::CallEFun_DoBehaviorNonLatent` · 004b87a0 — needs port: script efun do behavior non latent on the physics object; no citation in crates/
- [x] `gCWPhysicsObject::CallEFun_DoFX` · 004b8820 — ac2-script host/interface.rs queues the server event (vm.physics) for the weenie; ac2-client scripts_io apply_script_physics; host/tests physics_object_efuns_queue_their_server_events_for_the_weenie
- [ ] `gCWPhysicsObject::CallEFun_DoIntersection` · 004b88a0 — needs port: script efun do intersection on the physics object; no citation in crates/
- [x] `gCWPhysicsObject::CallEFun_DoMode` · 004b89c0 — Latent DoMode/DoBehavior: the efun queues the start with its context (ac2-script latent.rs, interface.rs do_mode/do_behavior: Interrupted until the body reports done); the body's ModeDone/BehaviorDone(context, error) resumes it through Resume_DoMode/Resume_DoBehavior (ScriptHost::resume_latent: out param = error). Client: client_latent.rs, client_audio.rs animation_sounds. Tests: ac2-script host tests/physics.rs a_latent_mode_resumes_the_context_when_the_body_reports_it_done, physics_mode_and_behavior_efuns_queue_their_server_events
- [ ] `gCWPhysicsObject::CallEFun_DoModeNonLatent` · 004b8ae0 — needs port: script efun do mode non latent on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_DoSlider` · 004b8b60 — needs port: script efun do slider on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_DoSliderNonLatent` · 004b8c90 · legions 004ae810 — needs port: script efun do slider non latent on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_DoTargettedFX` · 004b8d20 — needs port: script efun do targetted fx on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_ForceVisualDesc` · 004b8db0 — needs port: script efun force visual desc on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_GetAccelerationScale` · 004b8e30 — needs port: script efun get acceleration scale on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_GetDesiredMode` · 004b8ea0 — needs port: script efun get desired mode on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_GetElement` · 004b8f10 — needs port: script efun get element on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_GetHeight` · 004b8f80 — needs port: script efun get height on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_GetJumpScale` · 004b8ff0 — needs port: script efun get jump scale on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_GetMode` · 004b9060 — needs port: script efun get mode on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_GetRadius` · 004b90d0 — needs port: script efun get radius on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_GetTopLevelHeight` · 004b9140 — needs port: script efun get top level height on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_GetTopLevelRadius` · 004b91b0 — needs port: script efun get top level radius on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_GetVelocityScale` · 004b9220 — needs port: script efun get velocity scale on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_HasMotionInterpreter` · 004b9290 — needs port: script efun has motion interpreter on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_HasRemoteDoMotionValues` · 004b9300 — needs port: script efun has remote do motion values on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_InitVisualDesc` · 004b9370 — needs port: script efun init visual desc on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_IsAnimationFrozen` · 004b93e0 — needs port: script efun is animation frozen on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_IsDoingBehavior` · 004b9450 — needs port: script efun is doing behavior on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_IsInWorld` · 004b94d0 — needs port: script efun is in world on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_IsMovementFrozen` · 004b9540 — needs port: script efun is movement frozen on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_IsOutside` · 004b95b0 — needs port: script efun is outside on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_IsUndetectable` · 004b9620 — needs port: script efun is undetectable on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_LocalToGlobal` · 004b9690 — needs port: script efun local to global on the physics object; no citation in crates/
- [x] `gCWPhysicsObject::CallEFun_LookAt` · 004b9720 — LookAt chain: ac2-physics look_at.rs (LookAtManager), physics_obj_look.rs (CPhysicsObj::LookAt/LookAtDir), ac2-script efun 0x0080002A, ac2-client motion.rs
- [ ] `gCWPhysicsObject::CallEFun_MissileFireOnly` · 004b97a0 — needs port: script efun missile fire only on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_MissileInitPlaceAndFire` · 004b9810 — needs port: script efun missile init place and fire on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_MissileInitialize` · 004b98a0 — needs port: script efun missile initialize on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_MissileInquireDirection` · 004b9930 — needs port: script efun missile inquire direction on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_MissilePlace` · 004b99c0 — needs port: script efun missile place on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_MissilePlaceAndFire` · 004b9a30 — needs port: script efun missile place and fire on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_MissileRetarget` · 004b9aa0 — needs port: script efun missile retarget on the physics object; no citation in crates/
- [~] `gCWPhysicsObject::CallEFun_ReleaseBehavior` · 004b9b30 — ac2-script host/interface.rs do_mode, do_behavior and release_behavior queue the server event (DoMode, DoBehavior, ReleaseBehavior with the BehaviorParams native). Not modelled: the latent wait (retail returns Interrupted until the mode or behaviour is reached, then resumes the context on ModeDone); the script continues at once
- [ ] `gCWPhysicsObject::CallEFun_SetAnimationFrozen` · 004b9c60 — needs port: script efun set animation frozen on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_SetCloaked` · 004b9ce0 — needs port: script efun set cloaked on the physics object; no citation in crates/
- [x] `gCWPhysicsObject::CallEFun_SetCollisionState` · 004b9d60 — ac2-script glue/collision.rs and host/interface.rs: the CollisionParameters native (native/collision.rs) holds the minimums; SetCollisionState queues the non-zero ones for the weenie's body; HasCollided and IsEndCollision read the CollisionProfile native
- [ ] `gCWPhysicsObject::CallEFun_SetLocalVisualDesc` · 004b9de0 — needs port: script efun set local visual desc on the physics object; no citation in crates/
- [x] `gCWPhysicsObject::CallEFun_SetMode` · 004b9e60 — ac2-script host/interface.rs set_mode and stop_behavior queue the server event for the weenie's physics object (answer 1 with an owner); tests host/tests.rs physics_mode_and_behavior_efuns_queue_their_server_events
- [ ] `gCWPhysicsObject::CallEFun_SetMovementFrozen` · 004b9ee0 — needs port: script efun set movement frozen on the physics object; no citation in crates/
- [x] `gCWPhysicsObject::CallEFun_SetNoDrawPhysics` · 004b9f60 — ac2-script host::interface set_no_draw_physics (efun 0x800057 -> CPhysicsObj::set_nodraw): queued for the client, which keeps it per object (Client::is_no_draw) and gives the camera subject its no_draw; combat_7160 (the respawn clears it via CPlayer 0x85). The renderer does not read it for other objects
- [ ] `gCWPhysicsObject::CallEFun_SetUndetectable` · 004b9fe0 — needs port: script efun set undetectable on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_StopAllWeenieBehaviors` · 004ba060 — needs port: script efun stop all weenie behaviors on the physics object; no citation in crates/
- [x] `gCWPhysicsObject::CallEFun_StopBehavior` · 004ba0d0 — ac2-script host/interface.rs set_mode and stop_behavior queue the server event for the weenie's physics object (answer 1 with an owner); tests host/tests.rs physics_mode_and_behavior_efuns_queue_their_server_events
- [x] `gCWPhysicsObject::CallEFun_StopFX` · 004ba150 — ac2-script host/interface.rs queues the server event (vm.physics) for the weenie; ac2-client scripts_io apply_script_physics; host/tests physics_object_efuns_queue_their_server_events_for_the_weenie
- [ ] `gCWPhysicsObject::CallEFun_StopLayer` · 004ba1d0 — needs port: script efun stop layer on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_StopMoveToCancelsBehaviors` · 004ba250 — needs port: script efun stop move to cancels behaviors on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_TogglePlaceMarkerObjects` · 004ba2c0 — needs port: script efun toggle place marker objects on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun__SetIgnoreCollisions` · 004ba330 — needs port: script efun  set ignore collisions on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun__SetReportCollisions` · 004ba3b0 — needs port: script efun  set report collisions on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_GetDesiredMotionValues` · 004ba530 — needs port: script efun get desired motion values on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_GetLastDoMotionValues` · 004ba5e0 — needs port: script efun get last do motion values on the physics object; no citation in crates/
- [x] `gCWPhysicsObject::CallEFun_GetLocalVisualDesc` · 004ba690 — Returns a new VisualDesc native (word 2, REF) copied from the host's shared appearance (ScriptHost::set_appearance from the session's object table); a weenie never shared has the default description. The sell body reads it through gVisualDesc::GetTotalIconDesc.
- [ ] `gCWPhysicsObject::CallEFun_GetRemoteDoMotionValues` · 004ba750 — needs port: script efun get remote do motion values on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_GetStandardMotionValues` · 004ba800 — needs port: script efun get standard motion values on the physics object; no citation in crates/
- [-] `gCWPhysicsObject::DetectionTableToLRHash` · 004ba8b0 · legions 004afbe0 — STL/container internals: a Rust collection replaces it
- [x] `gCWPhysicsObject::Resume_DoBehavior` · 004baa50 · legions 004afd80 — Latent DoMode/DoBehavior: the efun queues the start with its context (ac2-script latent.rs, interface.rs do_mode/do_behavior: Interrupted until the body reports done); the body's ModeDone/BehaviorDone(context, error) resumes it through Resume_DoMode/Resume_DoBehavior (ScriptHost::resume_latent: out param = error). Client: client_latent.rs, client_audio.rs animation_sounds. Tests: ac2-script host tests/physics.rs a_latent_mode_resumes_the_context_when_the_body_reports_it_done, physics_mode_and_behavior_efuns_queue_their_server_events
- [ ] `gCWPhysicsObject::Resume_DoIntersection` · 004baae0 · legions 004afe10 — needs port: resume continuation of latent do intersection (the script waits and resumes here); no citation in crates/
- [x] `gCWPhysicsObject::Resume_DoMode` · 004babb0 · legions 004afee0 — Latent DoMode/DoBehavior: the efun queues the start with its context (ac2-script latent.rs, interface.rs do_mode/do_behavior: Interrupted until the body reports done); the body's ModeDone/BehaviorDone(context, error) resumes it through Resume_DoMode/Resume_DoBehavior (ScriptHost::resume_latent: out param = error). Client: client_latent.rs, client_audio.rs animation_sounds. Tests: ac2-script host tests/physics.rs a_latent_mode_resumes_the_context_when_the_body_reports_it_done, physics_mode_and_behavior_efuns_queue_their_server_events
- [ ] `gCWPhysicsObject::Resume_DoSlider` · 004bac40 · legions 004aff70 — needs port: resume continuation of latent do slider (the script waits and resumes here); no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_GetCollisionState` · 004bacd0 — needs port: script efun get collision state on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_InqCurrentProperties` · 004bad70 — needs port: script efun inq current properties on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_InqCurrentProperty` · 004bae10 — needs port: script efun inq current property on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_InqCylSphere` · 004baeb0 · legions 004b0160 — needs port: script efun inq cyl sphere on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_MissileInquireCurrentVelocity` · 004baf50 — needs port: script efun missile inquire current velocity on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::CallEFun_PerformLOS` · 004baff0 — needs port: script efun perform los on the physics object; no citation in crates/
- [ ] `gCWPhysicsObject::Resume_ReleaseBehavior` · 004bb090 · legions 004b02b0 — needs port: resume continuation of latent release behavior (the script waits and resumes here); no citation in crates/

## `portal/client/src/weenie/interp/glue/gcwrepository.cpp.c` (64/90)

### gCWRepository
- [~] `gCWRepository::CallEFun_AdjAndTruncateFloat` · 004b4ba0 — ac2-script glue/repository.rs adjust: delta on the description/table value, no class defaults (m_defaults), so an absent quality starts at 0 where retail returns the default
- [x] `gCWRepository::CallEFun_AdjAndTruncateInt` · 004b4c50 — ac2-script glue repository.rs adjust (table only, class default start via ScriptData::class_defaults); ac2-script data DatScriptData::class_defaults (qualities record at the DID, lazy)
- [~] `gCWRepository::CallEFun_AdjAndTruncateLongInt` · 004b4cf0 — ac2-script glue/repository.rs adjust: delta on the description/table value, no class defaults (m_defaults), so an absent quality starts at 0 where retail returns the default
- [x] `gCWRepository::CallEFun_AdjFloat` · 004b4dd0 — ac2-script glue repository.rs adjust (table only, class default start via ScriptData::class_defaults); ac2-script data DatScriptData::class_defaults (qualities record at the DID, lazy)
- [x] `gCWRepository::CallEFun_AdjInt` · 004b4e60 — ac2-script glue repository.rs adjust (table only, class default start via ScriptData::class_defaults); ac2-script data DatScriptData::class_defaults (qualities record at the DID, lazy)
- [x] `gCWRepository::CallEFun_AdjLongInt` · 004b4ee0 — ac2-script glue repository.rs adjust (table only, class default start via ScriptData::class_defaults); ac2-script data DatScriptData::class_defaults (qualities record at the DID, lazy)
- [x] `gCWRepository::CallEFun_DoesBoolExist` · 004b4f90 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_DoesDataIDExist` · 004b5010 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_DoesFloatExist` · 004b5090 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_DoesInstanceIDExist` · 004b5110 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_DoesIntExist` · 004b5190 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_DoesLongIntExist` · 004b5210 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_DoesPackageIDExist` · 004b5290 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_DoesPositionExist` · 004b5310 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_DoesStringExist` · 004b5390 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_DoesStringInfoExist` · 004b5410 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_DoesTimestampExist` · 004b5490 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [ ] `gCWRepository::CallEFun_EdgeSlides` · 004b5510 — needs port: script efun edge slides on the weenie repository; no citation in crates/
- [x] `gCWRepository::CallEFun_GetBool` · 004b5580 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_GetDataID` · 004b5600 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_GetFloat` · 004b5680 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_GetID` · 004b5700 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_GetInstanceID` · 004b5770 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_GetInt` · 004b5810 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_GetLongInt` · 004b5890 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_GetPackageID` · 004b5920 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_GetRawBool` · 004b59a0 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_GetRawDataID` · 004b5a20 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_GetRawFloat` · 004b5aa0 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_GetRawInstanceID` · 004b5b20 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_GetRawInt` · 004b5bc0 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_GetRawLongInt` · 004b5c40 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_GetRawPackageID` · 004b5cd0 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [ ] `gCWRepository::CallEFun_GetScale` · 004b5d50 — needs port: script efun get scale on the weenie repository; no citation in crates/
- [x] `gCWRepository::CallEFun_GetTimestamp` · 004b5dc0 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [~] `gCWRepository::CallEFun_LoadDataIDW` · 004b5e60 — glue/repository_physics.rs: qualities done; SetScale/SetEdgeSlides physics side effects not applied; LoadDataIDW needs the data source's file_did
- [x] `gCWRepository::CallEFun_RemoveBool` · 004b5ee0 — ac2-weenie qualities::remove_in_weenie_desc; ac2-script glue repository.rs remove_value
- [x] `gCWRepository::CallEFun_RemoveDataID` · 004b5f60 — ac2-weenie qualities::remove_in_weenie_desc (gmWeenieDesc::Remove* first, then table); ac2-script glue repository.rs remove_value
- [x] `gCWRepository::CallEFun_RemoveFloat` · 004b5fe0 — ac2-weenie qualities::remove_in_weenie_desc (gmWeenieDesc::Remove* first, then table); ac2-script glue repository.rs remove_value
- [x] `gCWRepository::CallEFun_RemoveInstanceID` · 004b6060 — ac2-weenie qualities::remove_in_weenie_desc (gmWeenieDesc::Remove* first, then table); ac2-script glue repository.rs remove_value
- [x] `gCWRepository::CallEFun_RemoveInt` · 004b60e0 — ac2-weenie qualities::remove_in_weenie_desc (gmWeenieDesc::Remove* first, then table); ac2-script glue repository.rs remove_value
- [~] `gCWRepository::CallEFun_RemoveLongInt` · 004b6160 — ac2-script glue/repository.rs removes the table entry only; a description field is not cleared (gmWeenieDesc::Remove* not modelled)
- [~] `gCWRepository::CallEFun_RemovePackageID` · 004b61e0 — ac2-script glue/repository.rs removes the table entry only; a description field is not cleared (gmWeenieDesc::Remove* not modelled)
- [~] `gCWRepository::CallEFun_RemovePosition` · 004b6260 — ac2-script glue/repository.rs removes the table entry only; a description field is not cleared (gmWeenieDesc::Remove* not modelled)
- [~] `gCWRepository::CallEFun_RemoveString` · 004b62e0 — ac2-script glue/repository.rs removes the table entry only; a description field is not cleared (gmWeenieDesc::Remove* not modelled)
- [~] `gCWRepository::CallEFun_RemoveStringInfo` · 004b6360 — ac2-script glue/repository.rs removes the table entry only; a description field is not cleared (gmWeenieDesc::Remove* not modelled)
- [~] `gCWRepository::CallEFun_RemoveTimestamp` · 004b63e0 — ac2-script glue/repository.rs removes the table entry only; a description field is not cleared (gmWeenieDesc::Remove* not modelled)
- [x] `gCWRepository::CallEFun_SetBool` · 004b6460 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_SetDataID` · 004b64e0 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [~] `gCWRepository::CallEFun_SetEdgeSlides` · 004b6560 — glue/repository_physics.rs: qualities done; SetScale/SetEdgeSlides physics side effects not applied; LoadDataIDW needs the data source's file_did
- [~] `gCWRepository::CallEFun_SetEtherealType` · 004b65e0 — glue/repository_physics.rs: qualities done; SetScale/SetEdgeSlides physics side effects not applied; LoadDataIDW needs the data source's file_did
- [x] `gCWRepository::CallEFun_SetFloat` · 004b6670 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_SetInstanceID` · 004b66f0 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_SetInt` · 004b6770 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_SetLongInt` · 004b67f0 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_SetPackageID` · 004b6870 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_SetPosition` · 004b68f0 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [~] `gCWRepository::CallEFun_SetScale` · 004b6970 — glue/repository_physics.rs: qualities done; SetScale/SetEdgeSlides physics side effects not applied; LoadDataIDW needs the data source's file_did
- [x] `gCWRepository::CallEFun_SetStringInfo` · 004b69f0 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_SetStringW` · 004b6a70 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_SetTimestamp` · 004b6af0 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [-] `gCWRepository::CallEFun_DumpBoolsToString` · 004b6b90 — admin dump-to-string / dump debug output: not in the client
- [-] `gCWRepository::CallEFun_DumpDIDsToString` · 004b6c50 — admin dump-to-string / dump debug output: not in the client
- [-] `gCWRepository::CallEFun_DumpFloatsToString` · 004b6d10 — admin dump-to-string / dump debug output: not in the client
- [-] `gCWRepository::CallEFun_DumpIIDsToString` · 004b6dd0 — admin dump-to-string / dump debug output: not in the client
- [-] `gCWRepository::CallEFun_DumpIntsToString` · 004b6e90 — admin dump-to-string / dump debug output: not in the client
- [-] `gCWRepository::CallEFun_DumpLongIntsToString` · 004b6f50 — admin dump-to-string / dump debug output: not in the client
- [-] `gCWRepository::CallEFun_DumpPositionsToString` · 004b7010 — admin dump-to-string / dump debug output: not in the client
- [-] `gCWRepository::CallEFun_DumpStringInfosToString` · 004b70d0 — admin dump-to-string / dump debug output: not in the client
- [-] `gCWRepository::CallEFun_DumpStringsToString` · 004b7190 — admin dump-to-string / dump debug output: not in the client
- [-] `gCWRepository::CallEFun_DumpTSsToString` · 004b7250 — admin dump-to-string / dump debug output: not in the client
- [-] `gCWRepository::CallEFun_DumpWeenieDescW` · 004b7310 — admin dump-to-string / dump debug output: not in the client
- [x] `gCWRepository::CallEFun_GetPosition` · 004b73d0 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_GetRawPosition` · 004b7480 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_GetRawStringInfo` · 004b7530 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_GetRawStringW` · 004b75f0 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_GetStringInfo` · 004b76c0 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_GetStringW` · 004b7780 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_InqBool` · 004b7850 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_InqDataID` · 004b7900 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [~] `gCWRepository::CallEFun_InqEtherealType` · 004b79b0 · legions 004adb10 — glue/repository_physics.rs: qualities done; SetScale/SetEdgeSlides physics side effects not applied; LoadDataIDW needs the data source's file_did
- [x] `gCWRepository::CallEFun_InqFloat` · 004b7a50 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_InqInstanceID` · 004b7b00 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_InqInt` · 004b7bb0 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_InqLongInt` · 004b7c60 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_InqPackageID` · 004b7d10 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_InqPosition` · 004b7dc0 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_InqStringInfo` · 004b7e60 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_InqStringW` · 004b7f00 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)
- [x] `gCWRepository::CallEFun_InqTimestamp` · 004b7fa0 — ac2-script glue/repository.rs: Inq/Get/Set/Does over WeenieQualities; a description field (IsInWeenieDesc) goes to the description (host/tests.rs repository_efuns_write_description_fields_into_the_description)

## `portal/shared/src/weenie/allocator/wallocator.cpp.c` (2/38)

### DBFile2IDTable
- [ ] `DBFile2IDTable::Get` · 0052cad0 · legions 00511050 — needs port: dbfile2 idtable::get; no citation in crates/

### CWAllocator
- [-] `CWAllocator::CWAllocator` · 0052caf0 · legions 00511070 — constructor of plain data: Rust struct init replaces it
- [ ] `CWAllocator::Clear` · 0052cb60 · legions 005110e0 — needs port: weenie allocator clear; no citation in crates/
- [ ] `CWAllocator::AllocateWeenieObject` · 0052cb80 · legions 00511100 — needs port: weenie allocator allocate weenie object; no citation in crates/
- [ ] `CWAllocator::AllocateWeenieDesc` · 0052cbd0 · legions 00511150 — needs port: weenie allocator allocate weenie desc; no citation in crates/
- [ ] `CWAllocator::AllocateWeenieDesc` · 0052cbf0 · legions 00511170 — needs port: weenie allocator allocate weenie desc; no citation in crates/
- [ ] `CWAllocator::IsMobileByQualitiesID_` · 0052cc10 — needs port: weenie allocator is mobile by qualities id ; no citation in crates/
- [ ] `CWAllocator::ResetShell` · 0052cc70 · legions 00511190 — needs port: shell reset on teardown: CWAllocator::ResetShell is not modelled (ac2-ui game_dialogs.rs says so)
- [ ] `CWAllocator::ResetShellInternal` · 0052cc80 · legions 005111a0 — needs port: weenie allocator reset shell internal; no citation in crates/
- [-] `CWAllocator::~CWAllocator` · 0052ccf0 · legions 00511210 — destructor: Rust Drop replaces it
- [ ] `CWAllocator::IsMobileByPackageID` · 0052cd90 — needs port: weenie allocator is mobile by package id; no citation in crates/
- [ ] `CWAllocator::IsMobileByDataID` · 0052cdc0 — needs port: weenie allocator is mobile by data id; no citation in crates/

### DBObjGrabber<DBFile2IDTable>
- [-] `DBObjGrabber<DBFile2IDTable>::DBObjGrabber<DBFile2IDTable>` · 0052ce10 · legions 00511380 — constructor of plain data: Rust struct init replaces it

### CWAllocator
- [ ] `CWAllocator::SetNoDBWLibFile` · 0052ce70 · legions 005113e0 — needs port: weenie allocator set no dbwlib file; no citation in crates/
- [ ] `CWAllocator::SetDBWLibFile` · 0052cec0 · legions 00511430 — needs port: weenie allocator set dbwlib file; no citation in crates/
- [x] `CWAllocator::GetDBWLib` · 0052cf10 · legions 00511480 — implemented: cited at crates/ac2-script/src/wlib.rs:17
- [ ] `CWAllocator::InitWLibPath` · 0052cfb0 · legions 00511520 — needs port: weenie allocator init wlib path; no citation in crates/

### Resolver
- [ ] `Resolver::Clear` · 0052d0f0 · legions 00511660 — needs port: script VM clear (Resolver); no citation in crates/
- [ ] `Resolver::Clear_Internal` · 0052d100 · legions 00511670 — needs port: script VM clear internal (Resolver); no citation in crates/

### CWAllocator
- [ ] `CWAllocator::AllocateResolver` · 0052d160 · legions 005116d0 — needs port: weenie allocator allocate resolver; no citation in crates/
- [ ] `CWAllocator::Init_Internal` · 0052d1e0 · legions 00511750 — needs port: weenie allocator init internal; no citation in crates/

### Resolver
- [ ] `Resolver::Init` · 0052d270 · legions 005117c0 — needs port: script VM init (Resolver); no citation in crates/
- [ ] `Resolver::LoadByteStream` · 0052d340 · legions 00511890 — needs port: script VM load byte stream (Resolver); no citation in crates/

### CentralDebugInfo
- [-] `CentralDebugInfo::GetInstance` · 0052d3c0 — admin/debug/logging tooling: not in the client

### CentralDebugByteStream
- [-] `CentralDebugByteStream::LoadDebug_AddLineOffsetInfo` · 0052d430 — text source loader, admin build only
- [-] `CentralDebugByteStream::LoadDebug_AddFunctionLocation` · 0052d4e0 — text source loader, admin build only
- [-] `CentralDebugByteStream::LoadDebug_AddOriginalSourceFileInfo` · 0052d5a0 — text source loader, admin build only
- [-] `CentralDebugByteStream::LoadDebug_AddFrameDebugInfo` · 0052d6f0 — text source loader, admin build only

### FrameDebugInfo
- [-] `FrameDebugInfo::FrameDebugInfo` · 0052d800 · legions 00511c50 — constructor of plain data: Rust struct init replaces it

### SmartArray<FrameMemberDebugInfo>
- [-] `SmartArray<FrameMemberDebugInfo>::operator=` · 0052d890 · legions 00511d30 — STL/container internals: a Rust collection replaces it

### FrameMemberDebugInfo
- [-] `FrameMemberDebugInfo::operator=` · 0052d900 · legions 00511da0 — operator= / copy of plain data: Rust value semantics replace it
- [-] `FrameMemberDebugInfo::FrameMemberDebugInfo` · 0052da70 · legions 00511f10 — constructor of plain data: Rust struct init replaces it

### PSmartArray<FrameMemberDebugInfo>
- [-] `PSmartArray<FrameMemberDebugInfo>::StreamPack` · 0052dab0 · legions 00511f50 — STL/container internals: a Rust collection replaces it

### FrameDebugInfo
- [-] `FrameDebugInfo::StreamPack` · 0052db50 · legions 00511ff0 — serialization / reference-tracking bookkeeping: Rust codecs and owned values replace it

### Resolver
- [x] `Resolver::ProcessByteStream` · 0052dbb0 · legions 00512050 — implemented: cited at crates/ac2-script/src/vm/program.rs:1
- [ ] `Resolver::BuildReverseMap` · 0052dc80 · legions 00512120 — needs port: script VM build reverse map (Resolver); no citation in crates/

### IntrusiveHashIterator<PStringBase<char>,HashTableData<PStringBase<char>,unsigned_long>_*>
- [-] `IntrusiveHashIterator<PStringBase<char>,HashTableData<PStringBase<char>,unsigned_long>_*>::operator++` · 0052dd40 · legions 005121e0 — STL/container internals: a Rust collection replaces it

### CWAllocator
- [ ] `CWAllocator::Init` · 0052dd70 · legions 00512210 — needs port: weenie allocator init; no citation in crates/

## `portal/shared/src/weenie/distributor/wdistributor.cpp.c` (6/13)

### CWDistributor
- [x] `CWDistributor::Construct` · 0052de20 · legions 005122c0 — entity WState read on demand (DBObj::Get, blocking); a cache-only miss waits for the loader and retries create_weenie (host/mod.rs create_weenie, host/tests/create.rs); package WStates preloaded before the frame loop (main.rs)
- [-] `CWDistributor::~CWDistributor` · 0052dee0 · legions 00512380 — destructor: Rust Drop replaces it
- [ ] `CWDistributor::GetContextState` · 0052df00 · legions 005123a0 — needs port: script VM get context state (CWDistributor); no citation in crates/
- [-] `CWDistributor::HandleCrash` · 0052df20 — crash reporting: admin/debug tooling
- [x] `CWDistributor::GetInstanceReference` · 0052df80 · legions 005123c0 — implemented: cited at crates/ac2-script/src/host/mod.rs:221
- [ ] `CWDistributor::GetMyPackageID` · 0052df90 · legions 005123d0 — needs port: script VM get my package id (CWDistributor); no citation in crates/
- [ ] `CWDistributor::GetProcessID` · 0052dfa0 · legions 005123e0 — needs port: script VM get process id (CWDistributor); no citation in crates/

### VMDataTimer
- [-] `VMDataTimer::~VMDataTimer` · 0052dfb0 — destructor: Rust Drop replaces it

### CWDistributor
- [-] `CWDistributor::CWDistributor` · 0052e010 · legions 005123f0 — constructor of plain data: Rust struct init replaces it
- [x] `CWDistributor::ResumeContext` · 0052e080 · legions 00512460 — implemented: cited at crates/ac2-script/src/host/mod.rs:258
- [x] `CWDistributor::ResumeContext` · 0052e130 — implemented: cited at crates/ac2-script/src/host/mod.rs:258
- [x] `CWDistributor::DispatchInterpMsg` · 0052e150 · legions 005124d0 — implemented: cited at crates/ac2-script/src/host/mod.rs:2
- [x] `CWDistributor::RecvEvt_InterpCEvent_Private` · 0052e250 · legions 00512590 — implemented: cited at crates/ac2-net/src/protocol/server_event.rs:59

## `portal/shared/src/weenie/interface/winterface.cpp.c` (13/52)

### CWInterface
- [-] `CWInterface::CWInterface` · 0052e280 · legions 005125c0 — constructor of plain data: Rust struct init replaces it
- [-] `CWInterface::~CWInterface` · 0052e2b0 · legions 005125f0 — destructor: Rust Drop replaces it
- [ ] `CWInterface::HandleCreation` · 0052e2e0 — needs port: interface handle creation; no citation in crates/
- [ ] `CWInterface::HandleDestruction` · 0052e3c0 · legions 00512700 — needs port: interface handle destruction; no citation in crates/
- [~] `CWInterface::ObjectBeingDestructed` · 0052e3f0 · legions 00512730 — CBookKeep::ObjectBeingDestructed (004da150): the destroyed object's children are queued for destruction in ac2-weenie world.rs destroy_object_internal (add_object_to_be_destroyed, 25 s). Not modelled: the physobj child detach (vtable 0x3f4) and parent teardown (0x160). CWInterface::ObjectBeingDestructed (0052e3f0) also purges ClientTriggerScheduler events, the ac2-client row ClientTriggerScheduler::PurgeEventsForInterface, not ported
- [ ] `CWInterface::HandleEnterWorld` · 0052e410 · legions 00512750 — needs port: interface handle enter world; no citation in crates/
- [ ] `CWInterface::HandleLeaveWorld` · 0052e440 · legions 00512780 — needs port: interface handle leave world; no citation in crates/
- [ ] `CWInterface::HandlePostLeaveWorld` · 0052e470 · legions 005127b0 — needs port: interface handle post leave world; no citation in crates/
- [x] `CWInterface::HandleTeleportStart` · 0052e490 · legions 005127d0 — implemented: cited at crates/ac2-client/src/client_teleport.rs:12
- [x] `CWInterface::HandleTeleportEnd` · 0052e4a0 · legions 005127e0 — implemented: cited at crates/ac2-client/src/client_teleport.rs:17
- [ ] `CWInterface::HandleCharacterSessionStart` · 0052e4c0 · legions 00512800 — needs port: interface handle character session start; no citation in crates/
- [ ] `CWInterface::HandleCharacterSessionEnd` · 0052e520 · legions 00512860 — needs port: interface handle character session end; no citation in crates/
- [ ] `CWInterface::ApplyProperties` · 0052e530 · legions 00512870 — needs port: interface apply properties; no citation in crates/
- [x] `CWInterface::SendServerEvt` · 0052e570 · legions 005128b0 — implemented: cited at crates/ac2-net/src/protocol/gameplay/mod.rs:5
- [ ] `CWInterface::SendServerEvtEncrypt` · 0052e590 · legions 005128d0 — needs port: interface send server evt encrypt; no citation in crates/
- [x] `CWInterface::GetInternalWeenieObject` · 0052e5b0 · legions 005128f0 — implemented: cited at crates/ac2-script/src/host/interface.rs:277
- [x] `CWInterface::IsPlayer` · 0052e5e0 · legions 00512920 — implemented: cited at crates/ac2-weenie/src/objects.rs:83
- [x] `CWInterface::IsCreature` · 0052e600 · legions 00512940 — implemented: cited at crates/ac2-weenie/src/session/gameplay.rs:21
- [x] `CWInterface::IsThePlayer` · 0052e620 · legions 00512960 — implemented: cited at crates/ac2-script/src/host/interface.rs:266
- [x] `CWInterface::OnStatChanged` · 0052e660 · legions 005129a0 — ac2-client stat_changed: OnStatChanged (IFun 0x7F0044: table, quality) on the object's process once the VM has it (CPlayer 0x85: vitals, XP bar, vitae, skill panel); the session's stand-ins otherwise
- [x] `CWInterface::RecvEvt_PlayerDesc` · 0052e6a0 · legions 005129e0 — implemented: cited at crates/ac2-net/src/protocol/qualities.rs:2
- [ ] `CWInterface::HandleIntersectionDone` · 0052e6c0 · legions 00512a00 — needs port: interface handle intersection done; no citation in crates/
- [ ] `CWInterface::InqDetectionProfile` · 0052e6e0 · legions 00512a20 — needs port: interface inq detection profile; no citation in crates/
- [~] `CWInterface::HandleCollision` · 0052e700 · legions 00512a40 — impact reports only: a landing impact (normal and speed, taken once by ac2-physics take_impact_report) is handed to the player's HandleCollision IFun (ac2-client collision_reported); object, environment, surface and water properties, begin/end reports and missiles are not ported
- [ ] `CWInterface::HandleBehaviorDone` · 0052e730 · legions 00512a70 — needs port: interface handle behavior done; no citation in crates/
- [ ] `CWInterface::HandleReleaseDone` · 0052e750 · legions 00512a90 — needs port: interface handle release done; no citation in crates/
- [ ] `CWInterface::HandleSliderDone` · 0052e770 · legions 00512ab0 — needs port: interface handle slider done; no citation in crates/
- [ ] `CWInterface::HandleModeDone` · 0052e790 · legions 00512ad0 — needs port: interface handle mode done; no citation in crates/
- [ ] `CWInterface::InqPrevHistoryItem` · 0052e7b0 · legions 00512af0 — needs port: interface inq prev history item; no citation in crates/
- [ ] `CWInterface::InqNextHistoryItem` · 0052e7c0 · legions 00512b00 — needs port: interface inq next history item; no citation in crates/
- [ ] `CWInterface::DisplayText` · 0052e7d0 · legions 00512b10 — needs port: interface display text; no citation in crates/
- [ ] `CWInterface::DisplayStringInfo` · 0052e7f0 · legions 00512b30 — needs port: interface display string info; no citation in crates/
- [~] `CWInterface::PreloadWSLSingletons` · 0052e810 · legions 00512b50 — retail's script-driven singleton warm-up at interface start; not ported: singletons fill on first use (ReferenceTable::GetSingleton) and their WStates are preloaded as the 0x70000000 range. Its vtable caller is not identified in the decompile
- [ ] `CWInterface::CheckDetectionTypeMatch` · 0052e870 · legions 00512bb0 — needs port: interface check detection type match; no citation in crates/
- [ ] `CWInterface::InqStringInfoValue` · 0052e8c0 · legions 00512c00 — needs port: interface inq string info value; no citation in crates/

### `anonymous-namespace'::ClientLatentEfunActivator
- [ ] ``anonymous-namespace'::ClientLatentEfunActivator::OnTrigger` · 0052e960 — needs port: `anonymous-namespace'::client latent efun activator::on trigger; no citation in crates/

### CWInterface
- [ ] `CWInterface::ExpandText` · 0052e970 · legions 00512cb0 — needs port: interface expand text; no citation in crates/
- [ ] `CWInterface::DisplayHelpText` · 0052e9c0 · legions 00512d00 — needs port: interface display help text; no citation in crates/

### `anonymous-namespace'::SleepAwakener
- [ ] ``anonymous-namespace'::SleepAwakener::OnTrigger` · 0052eaf0 — needs port: `anonymous-namespace'::sleep awakener::on trigger; no citation in crates/

### `anonymous-namespace'::InqStringInfoValue_AsyncCacheCallback
- [ ] ``anonymous-namespace'::InqStringInfoValue_AsyncCacheCallback::OnContextFinished` · 0052eb10 — needs port: `anonymous-namespace'::inq string info value async cache callback::on context finished; no citation in crates/

### CWInterface
- [ ] `CWInterface::GetOutputFilesPath` · 0052eb80 · legions 00512ec0 — needs port: interface get output files path; no citation in crates/

### HashTable<PStringBase<unsigned_short>,FunctionID>
- [-] `HashTable<PStringBase<unsigned_short>,FunctionID>::~HashTable<PStringBase<unsigned_short>,FunctionID>` · 0052ed30 · legions 00512f40 — STL/container internals: a Rust collection replaces it

### CWInterface
- [ ] `CWInterface::ProcessText` · 0052ee50 · legions 00513060 — needs port: citation only in a test; port and cite in the client code
- [x] `CWInterface::HandleCmdArgs` · 0052ef20 · legions 00513130 — ac2-script host/shell.rs run_script_commands: a ShellCommand event whose word a script registered runs that function on the player's process with (text, RList of tokens)
- [ ] `CWInterface::ResumeCallback` · 0052f070 · legions 00513280 — needs port: interface resume callback; no citation in crates/
- [ ] `CWInterface::Sleep` · 0052f100 — needs port: interface sleep; no citation in crates/
- [x] `CWInterface::DoCmdInterp` · 0052f1f0 — ac2-script host/shell.rs run_script_commands: a ShellCommand event whose word a script registered runs that function on the player's process with (text, RList of tokens)
- [ ] `CWInterface::DoHelpInterp` · 0052f300 — needs port: interface do help interp; no citation in crates/
- [x] `CWInterface::AddShellCmd` · 0052f430 · legions 005135f0 — implemented: cited at crates/ac2-weenie/src/shell/mod.rs:160
- [ ] `CWInterface::AddShellHelp` · 0052f6e0 · legions 005138a0 — needs port: interface add shell help; no citation in crates/
- [x] `CWInterface::AddShellCmdByStringID` · 0052f990 · legions 00513b50 — implemented: cited at crates/ac2-client/src/chat_commands.rs:3
- [ ] `CWInterface::AddShellHelpByStringID` · 0052fa10 · legions 00513bd0 — needs port: interface add shell help by string id; no citation in crates/

## `portal/shared/src/weenie/interp/dbresolver.cpp.c` (0/4)

### DBResolver
- [ ] `DBResolver::Clear` · 006b7ef0 · legions 0064f710 — needs port: script VM clear (DBResolver); no citation in crates/

### Resolver
- [-] `Resolver::~Resolver` · 006b7f20 · legions 0064f740 — destructor: Rust Drop replaces it

### DBResolver
- [-] `DBResolver::DBResolver` · 006b7fc0 · legions 0064f7e0 — constructor of plain data: Rust struct init replaces it
- [ ] `DBResolver::Init` · 006b8010 · legions 0064f830 — needs port: script VM init (DBResolver); no citation in crates/

## `portal/shared/src/weenie/interp/winterpasserts.cpp.c` (0/1)

### WInterpAsserts
- [-] `WInterpAsserts::GetContextObject` · 006b8070 — admin/debug/logging tooling: not in the client

## `portal/shared/src/weenie/interp/wlib.cpp.c` (1/8)

### WLib
- [-] `WLib::WLib` · 006b7790 · legions 0064efb0 — constructor of plain data: Rust struct init replaces it
- [-] `WLib::GetDBOType` · 006b77c0 — serialization / reference-tracking bookkeeping: Rust codecs and owned values replace it
- [-] `WLib::GetPackSize` · 006b77d0 — serialization / reference-tracking bookkeeping: Rust codecs and owned values replace it
- [ ] `WLib::Pack` · 006b7800 · legions 0064f020 — needs port: script VM pack (WLib); no citation in crates/
- [-] `WLib::Allocate` · 006b7840 — serialization / reference-tracking bookkeeping: Rust codecs and owned values replace it
- [-] `WLib::ReleaseRefs` · 006b7850 · legions 0064f070 — teardown and reference release: Rust Drop replaces it
- [-] `WLib::Clear` · 006b78a0 · legions 0064f0c0 — teardown and reference release: Rust Drop replaces it
- [x] `WLib::UnPack` · 006b78f0 — implemented: cited at crates/ac2-script/src/wlib.rs:31

## `portal/shared/src/weenie/interp/wstate.cpp.c` (3/16)

### WState
- [-] `WState::Destroy` · 006b79e0 — teardown and reference release: Rust Drop replaces it
- [x] `WState::GetStateByType` · 006b79f0 · legions 0064f210 — implemented: cited at crates/ac2-script/src/wstate.rs:28
- [-] `WState::GetPackSize` · 006b7a30 · legions 0064f250 — serialization / reference-tracking bookkeeping: Rust codecs and owned values replace it
- [ ] `WState::Pack` · 006b7a50 · legions 0064f270 — needs port: script VM pack (WState); no citation in crates/
- [x] `WState::UnPack` · 006b7aa0 · legions 0064f2c0 — implemented: cited at crates/ac2-script/src/wstate.rs:4
- [-] `WState::WState` · 006b7b00 · legions 0064f320 — constructor of plain data: Rust struct init replaces it
- [-] `WState::GetDBOType` · 006b7b50 — serialization / reference-tracking bookkeeping: Rust codecs and owned values replace it
- [-] `WState::GetPackSize` · 006b7b70 — inline copy of 006b7a30
- [ ] `WState::Pack` · 006b7b80 — needs port: script VM pack (WState); no citation in crates/
- [x] `WState::UnPack` · 006b7b90 — implemented: cited at crates/ac2-script/src/wstate.rs:4
- [-] `WState::WState` · 006b7ba0 · legions 0064f3c0 — constructor of plain data: Rust struct init replaces it
- [-] `WState::Allocate` · 006b7bf0 — serialization / reference-tracking bookkeeping: Rust codecs and owned values replace it
- [-] `WState::GetSubDataIDs` · 006b7c60 — serialization / reference-tracking bookkeeping: Rust codecs and owned values replace it

### ReferenceTable
- [ ] `ReferenceTable::GetNativeType` · 006b7d40 · legions 0064f560 — needs port: script VM get native type (ReferenceTable); no citation in crates/

### WState
- [ ] `WState::SaveStateByID` · 006b7da0 · legions 0064f5c0 — needs port: script VM save state by id (WState); no citation in crates/
- [ ] `WState::SaveStateByType` · 006b7ea0 · legions 0064f6c0 — needs port: script VM save state by type (WState); no citation in crates/

## `portal/shared/src/weenie/repository/wrepository.cpp.c` (60/101)

### CWRepository
- [-] `CWRepository::Construct` · 00530e70 · legions 00514e10 — singleton constructor: Rust builds the object directly
- [-] `CWRepository::~CWRepository` · 00530f10 · legions 00514eb0 — destructor: Rust Drop replaces it
- [x] `CWRepository::SetWeenieDesc` · 00530f40 · legions 00514ee0 — implemented: cited at crates/ac2-weenie/src/qualities.rs:39
- [x] `CWRepository::SetQualities` · 00530f70 · legions 00514f10 — implemented: cited at crates/ac2-weenie/src/session.rs:127
- [ ] `CWRepository::HandleEnterWorld` · 00530f90 · legions 00514f30 — needs port: weenie repository handle enter world; no citation in crates/
- [ ] `CWRepository::HandleLeaveWorld` · 00530fa0 · legions 00514f40 — needs port: weenie repository handle leave world; no citation in crates/
- [ ] `CWRepository::InqWeenieDescription` · 00530fb0 — needs port: weenie repository inq weenie description; no citation in crates/
- [x] `CWRepository::InqInt` · 00530fd0 · legions 00514f70 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::InqLongInt` · 00530ff0 · legions 00514f90 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::InqBool` · 00531010 · legions 00514fb0 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::InqFloat` · 00531030 · legions 00514fd0 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::InqStringW` · 00531050 · legions 00514ff0 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::InqDataID` · 00531070 · legions 00515010 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::InqInstanceID` · 00531090 · legions 00515030 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::InqPosition` · 005310b0 · legions 00515050 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::InqTimestamp` · 005310d0 · legions 00515070 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::InqPackageID` · 005310f0 · legions 00515090 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::InqStringInfo` · 00531110 · legions 005150b0 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::GetInt` · 00531130 · legions 005150d0 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::GetBool` · 00531160 · legions 00515100 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::GetFloat` · 00531190 · legions 00515130 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::GetDataID` · 005311c0 · legions 00515160 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::GetInstanceID` · 005311f0 · legions 00515190 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::GetTimestamp` · 00531250 · legions 005151f0 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::GetStringInfo` · 00531280 · legions 00515290 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::GetPackageID` · 005312f0 · legions 00515220 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::GetLongInt` · 00531320 · legions 00515250 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::DoesIntExist` · 00531360 · legions 00515300 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::DoesLongIntExist` · 00531380 · legions 00515320 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::DoesBoolExist` · 005313a0 · legions 00515340 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::DoesFloatExist` · 005313c0 · legions 00515360 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::DoesDataIDExist` · 005313e0 · legions 00515380 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::DoesInstanceIDExist` · 00531400 · legions 005153a0 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::DoesPositionExist` · 00531420 · legions 005153c0 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::DoesTimestampExist` · 00531490 · legions 00515430 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::DoesStringInfoExist` · 005314b0 · legions 00515450 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::DoesPackageIDExist` · 005314f0 · legions 00515490 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::SetInt` · 00531510 · legions 005154b0 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::SetLongInt` · 00531540 · legions 005154e0 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::SetBool` · 00531570 · legions 00515510 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::SetFloat` · 005315a0 · legions 00515540 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::SetDataID` · 005315d0 · legions 00515570 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::SetInstanceID` · 00531600 · legions 005155a0 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::SetPosition` · 00531630 · legions 005155d0 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::SetTimestamp` · 00531660 · legions 00515600 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::SetPackageID` · 00531690 · legions 00515630 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::SetStringInfo` · 005316c0 · legions 00515660 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [~] `CWRepository::RemoveInt` · 005316f0 · legions 00515690 — glue/repository.rs: no class defaults (m_defaults) for Adj and Remove; Remove does not clear a description field
- [~] `CWRepository::RemoveLongInt` · 00531710 · legions 005156b0 — glue/repository.rs: no class defaults (m_defaults) for Adj and Remove; Remove does not clear a description field
- [~] `CWRepository::RemoveBool` · 00531730 · legions 005156d0 — glue/repository.rs: no class defaults (m_defaults) for Adj and Remove; Remove does not clear a description field
- [~] `CWRepository::RemoveFloat` · 00531750 · legions 005156f0 — glue/repository.rs: no class defaults (m_defaults) for Adj and Remove; Remove does not clear a description field
- [~] `CWRepository::RemoveString` · 00531770 · legions 00515710 — glue/repository.rs: no class defaults (m_defaults) for Adj and Remove; Remove does not clear a description field
- [~] `CWRepository::RemoveDataID` · 00531790 · legions 00515730 — glue/repository.rs: no class defaults (m_defaults) for Adj and Remove; Remove does not clear a description field
- [~] `CWRepository::RemoveInstanceID` · 005317b0 · legions 00515750 — glue/repository.rs: no class defaults (m_defaults) for Adj and Remove; Remove does not clear a description field
- [~] `CWRepository::RemovePosition` · 005317d0 · legions 00515770 — glue/repository.rs: no class defaults (m_defaults) for Adj and Remove; Remove does not clear a description field
- [~] `CWRepository::RemoveTimestamp` · 005317f0 · legions 00515790 — glue/repository.rs: no class defaults (m_defaults) for Adj and Remove; Remove does not clear a description field
- [~] `CWRepository::RemoveStringInfo` · 00531810 · legions 005157b0 — glue/repository.rs: no class defaults (m_defaults) for Adj and Remove; Remove does not clear a description field
- [~] `CWRepository::RemovePackageID` · 00531830 · legions 005157d0 — glue/repository.rs: no class defaults (m_defaults) for Adj and Remove; Remove does not clear a description field
- [x] `CWRepository::AdjInt` · 00531850 · legions 005157f0 — ac2-script glue repository.rs adjust (table only, class default start via ScriptData::class_defaults); ac2-script data DatScriptData::class_defaults (qualities record at the DID, lazy)
- [x] `CWRepository::AdjAndTruncateInt` · 00531880 · legions 00515820 — ac2-script glue repository.rs adjust (table only, class default start via ScriptData::class_defaults); ac2-script data DatScriptData::class_defaults (qualities record at the DID, lazy)
- [x] `CWRepository::AdjLongInt` · 005318c0 · legions 00515860 — ac2-script glue repository.rs adjust (table only, class default start via ScriptData::class_defaults); ac2-script data DatScriptData::class_defaults (qualities record at the DID, lazy)
- [~] `CWRepository::AdjAndTruncateLongInt` · 00531910 · legions 005158b0 — glue/repository.rs: no class defaults (m_defaults) for Adj and Remove; Remove does not clear a description field
- [x] `CWRepository::AdjFloat` · 00531970 · legions 00515910 — ac2-script glue repository.rs adjust (table only, class default start via ScriptData::class_defaults); ac2-script data DatScriptData::class_defaults (qualities record at the DID, lazy)
- [~] `CWRepository::AdjAndTruncateFloat` · 005319b0 · legions 00515950 — glue/repository.rs: no class defaults (m_defaults) for Adj and Remove; Remove does not clear a description field
- [x] `CWRepository::RecvEvt_UpdateInt_Private` · 005319f0 · legions 00515990 — ac2-net protocol/quality_event.rs (private and visual, all ten kinds); ac2-weenie world.rs applies it (visual ones queue to their object); session raises QualityChanged for OnStatChanged (ac2-client scripts.rs)
- [x] `CWRepository::RecvEvt_UpdateLongInt_Private` · 00531a30 · legions 005159d0 — ac2-net protocol/quality_event.rs (private and visual, all ten kinds); ac2-weenie world.rs applies it (visual ones queue to their object); session raises QualityChanged for OnStatChanged (ac2-client scripts.rs)
- [x] `CWRepository::RecvEvt_UpdateBool_Private` · 00531a70 · legions 00515a10 — ac2-net protocol/quality_event.rs (private and visual, all ten kinds); ac2-weenie world.rs applies it (visual ones queue to their object); session raises QualityChanged for OnStatChanged (ac2-client scripts.rs)
- [x] `CWRepository::RecvEvt_UpdateFloat_Private` · 00531ab0 · legions 00515a50 — ac2-net protocol/quality_event.rs (private and visual, all ten kinds); ac2-weenie world.rs applies it (visual ones queue to their object); session raises QualityChanged for OnStatChanged (ac2-client scripts.rs)
- [x] `CWRepository::RecvEvt_UpdateTimestamp_Private` · 00531af0 · legions 00515a90 — ac2-net protocol/quality_event.rs (private and visual, all ten kinds); ac2-weenie world.rs applies it (visual ones queue to their object); session raises QualityChanged for OnStatChanged (ac2-client scripts.rs)
- [x] `CWRepository::RecvEvt_UpdateString_Private` · 00531b30 · legions 00515ad0 — ac2-net protocol/quality_event.rs (private and visual, all ten kinds); ac2-weenie world.rs applies it (visual ones queue to their object); session raises QualityChanged for OnStatChanged (ac2-client scripts.rs)
- [x] `CWRepository::RecvEvt_UpdateDataID_Private` · 00531b70 · legions 00515b10 — ac2-net protocol/quality_event.rs (private and visual, all ten kinds); ac2-weenie world.rs applies it (visual ones queue to their object); session raises QualityChanged for OnStatChanged (ac2-client scripts.rs)
- [x] `CWRepository::RecvEvt_UpdateInstanceID_Private` · 00531bb0 · legions 00515b50 — ac2-net protocol/quality_event.rs (private and visual, all ten kinds); ac2-weenie world.rs applies it (visual ones queue to their object); session raises QualityChanged for OnStatChanged (ac2-client scripts.rs)
- [x] `CWRepository::RecvEvt_UpdatePosition_Private` · 00531bf0 · legions 00515b90 — ac2-net protocol/quality_event.rs (private and visual, all ten kinds); ac2-weenie world.rs applies it (visual ones queue to their object); session raises QualityChanged for OnStatChanged (ac2-client scripts.rs)
- [x] `CWRepository::RecvEvt_UpdateStringInfo_Private` · 00531c30 · legions 00515bd0 — ac2-net protocol/quality_event.rs (private and visual, all ten kinds); ac2-weenie world.rs applies it (visual ones queue to their object); session raises QualityChanged for OnStatChanged (ac2-client scripts.rs)
- [-] `CWRepository::CWRepository` · 00531c70 · legions 00515c10 — constructor of plain data: Rust struct init replaces it
- [ ] `CWRepository::GetPhysObjID` · 00531ce0 · legions 00515c80 — needs port: weenie repository get phys obj id; no citation in crates/
- [ ] `CWRepository::GetScale` · 00531d10 · legions 00515cb0 — needs port: weenie repository get scale; no citation in crates/
- [ ] `CWRepository::EdgeSlides` · 00531d40 · legions 00515ce0 — needs port: weenie repository edge slides; no citation in crates/
- [ ] `CWRepository::InqEtherealType` · 00531d70 · legions 00515d10 — needs port: weenie repository inq ethereal type; no citation in crates/
- [ ] `CWRepository::SetScale` · 00531df0 · legions 00515d90 — needs port: weenie repository set scale; no citation in crates/
- [ ] `CWRepository::SetEdgeSlides` · 00531e30 · legions 00515dd0 — needs port: weenie repository set edge slides; no citation in crates/
- [ ] `CWRepository::SetEtherealType` · 00531e60 · legions 00515e00 — needs port: weenie repository set ethereal type; no citation in crates/
- [ ] `CWRepository::GatherPhysicsState` · 00531ed0 · legions 00515e70 — needs port: weenie repository gather physics state; no citation in crates/
- [x] `CWRepository::GetStringW` · 00531fc0 · legions 00516020 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::GetRawPosition` · 00532050 · legions 00515f60 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::DoesStringExist` · 00532110 · legions 005160b0 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [x] `CWRepository::SetStringW` · 00532170 · legions 00516110 — ac2-script glue/repository.rs over WeenieQualities (description fields first, the gCWRepository efun behind it)
- [ ] `CWRepository::LoadDataIDW` · 00532200 · legions 005161a0 — needs port: weenie repository load data idw; no citation in crates/
- [ ] `CWRepository::BuildPhysObjProfile` · 00532290 · legions 00516230 — needs port: weenie repository build phys obj profile; no citation in crates/
- [-] `CWRepository::DumpWeenieDesc` · 00532380 · legions 00516320 — admin dump-to-string / dump debug output: not in the client
- [-] `CWRepository::DumpWeenieDescW` · 00532510 · legions 005164b0 — admin dump-to-string / dump debug output: not in the client
- [-] `CWRepository::DumpBoolsToString` · 00532560 · legions 00516500 — admin dump-to-string / dump debug output: not in the client
- [-] `CWRepository::DumpDIDsToString` · 005327b0 · legions 00516750 — admin dump-to-string / dump debug output: not in the client
- [-] `CWRepository::DumpFloatsToString` · 005329f0 · legions 00516990 — admin dump-to-string / dump debug output: not in the client
- [-] `CWRepository::DumpIIDsToString` · 00532c40 · legions 00516be0 — admin dump-to-string / dump debug output: not in the client
- [-] `CWRepository::DumpIntsToString` · 00532e80 · legions 00516e20 — admin dump-to-string / dump debug output: not in the client
- [-] `CWRepository::DumpLongIntsToString` · 005330c0 · legions 00517060 — admin dump-to-string / dump debug output: not in the client
- [-] `CWRepository::DumpStringsToString` · 00533300 · legions 005172a0 — admin dump-to-string / dump debug output: not in the client
- [-] `CWRepository::DumpTSsToString` · 00533550 · legions 005174f0 — admin dump-to-string / dump debug output: not in the client
- [-] `CWRepository::DumpPositionsToString` · 00533790 · legions 00517730 — admin dump-to-string / dump debug output: not in the client
- [-] `CWRepository::DumpStringInfosToString` · 00533a70 · legions 00517a10 — admin dump-to-string / dump debug output: not in the client

## `portal/shared/src/weenie/system/log/wlogsystem.cpp.c` (0/10)

### WLogSystem
- [-] `WLogSystem::IsFileLogging` · 007332e0 · legions 006af790 — admin/debug/logging tooling: not in the client
- [-] `WLogSystem::IsPrintLogging` · 00733300 · legions 006af7b0 — admin/debug/logging tooling: not in the client
- [-] `WLogSystem::SetFileLog` · 00733320 · legions 006af7d0 — admin/debug/logging tooling: not in the client
- [-] `WLogSystem::SetPrintLog` · 00733330 · legions 006af7e0 — admin/debug/logging tooling: not in the client
- [-] `WLogSystem::AddFileLog` · 00733340 · legions 006af7f0 — admin/debug/logging tooling: not in the client
- [-] `WLogSystem::AddPrintLog` · 00733360 · legions 006af810 — admin/debug/logging tooling: not in the client
- [-] `WLogSystem::RemoveFileLog` · 00733380 · legions 006af830 — admin/debug/logging tooling: not in the client
- [-] `WLogSystem::RemovePrintLog` · 007333a0 · legions 006af850 — admin/debug/logging tooling: not in the client
- [-] `WLogSystem::GetFileLog` · 007333c0 · legions 006af870 — admin/debug/logging tooling: not in the client
- [-] `WLogSystem::GetPrintLog` · 007333d0 · legions 006af880 — admin/debug/logging tooling: not in the client

## `portal/shared/src/weenie/wbookie/wbookkeeper.cpp.c` (1/5)

### CWBookKeeper
- [-] `CWBookKeeper::CWBookKeeper` · 0052dd90 · legions 00512230 — constructor of plain data: Rust struct init replaces it
- [x] `CWBookKeeper::CSendToTurbineRoomByID` · 0052ddb0 · legions 00512250 — implemented: cited at crates/ac2-weenie/src/session/gameplay.rs:159
- [ ] `CWBookKeeper::AddObjectToBeDestroyed` · 0052ddf0 · legions 00512290 — needs port: cwbook keeper::add object to be destroyed; no citation in crates/
- [ ] `CWBookKeeper::RemoveObjectToBeDestroyed` · 0052de00 · legions 005122a0 — needs port: cwbook keeper::remove object to be destroyed; no citation in crates/
- [-] `CWBookKeeper::~CWBookKeeper` · 0052de10 · legions 005122b0 — destructor: Rust Drop replaces it

## `portal/shared/src/weenie/weenieobject.cpp.c` (2/9)

### CWeenieObject
- [x] `CWeenieObject::IsMobile` · 0052fa90 · legions 00513c50 — implemented: cited at crates/ac2-physics/src/body.rs:218
- [x] `CWeenieObject::ObjectBeingDestructed` · 0052fab0 · legions 00513c70 — CWeenieObject::ObjectBeingDestructed -> FUN_0048f530 dispatches notice 0x1040c to the weenie's handlers (radar, selection, stack, ingredient, external container, fellow). Here: WorldEvent::Destroyed -> SessionEvent::ObjectRemoved, consumed by those UI pieces. The shutdown argument is unread by the Legions handlers (radar, fellowinfo). ECM_Physics::SendNotice_BeingDeleted is the same dispatch
- [-] `CWeenieObject::CWeenieObject` · 0052fad0 · legions 00513c90 — constructor of plain data: Rust struct init replaces it
- [-] `CWeenieObject::GetNoticeHandlers` · 0052fb00 · legions 00513cc0 — notice-handler registry: Rust dispatches notices by enum, no handler list
- [-] `CWeenieObject::RegisterNoticeHandler` · 0052fb40 · legions 00513d00 — notice-handler registry: Rust dispatches notices by enum, no handler list
- [-] `CWeenieObject::UnRegisterNoticeHandler` · 0052fc40 · legions 00513e00 — notice-handler registry: Rust dispatches notices by enum, no handler list

### GlobalEventHandler
- [-] `GlobalEventHandler::RemoveHandlerFromList` · 0052fcd0 · legions 0048ba20 — notice-handler registry: Rust dispatches notices by enum, no handler list

### CWeenieObject
- [-] `CWeenieObject::~CWeenieObject` · 0052fd40 · legions 00513e90 — destructor: Rust Drop replaces it
- [-] `CWeenieObject::UnRegisterAllNoticeHandlers` · 0052fd70 · legions 00513ec0 — notice-handler registry: Rust dispatches notices by enum, no handler list

## `portal/shared/src/weenie/wphysobj/wphysicsobject.cpp.c` (11/91)

### CWPhysicsObject
- [-] `CWPhysicsObject::CWPhysicsObject` · 0052fe30 · legions 00513f80 — constructor of plain data: Rust struct init replaces it
- [-] `CWPhysicsObject::~CWPhysicsObject` · 0052fe50 · legions 00513fa0 — destructor: Rust Drop replaces it
- [ ] `CWPhysicsObject::HandleCreation` · 0052fe60 · legions 00513fb0 — needs port: physics object handle creation; no citation in crates/
- [ ] `CWPhysicsObject::HandleDestruction` · 0052fe80 · legions 00513fd0 — needs port: physics object handle destruction; no citation in crates/
- [ ] `CWPhysicsObject::DoIntersection` · 0052fe90 — needs port: physics object do intersection; no citation in crates/
- [ ] `CWPhysicsObject::PerformLOS` · 0052ff20 · legions 00514020 — needs port: physics object perform los; no citation in crates/
- [~] `CWPhysicsObject::SetCollisionState` · 0052ff40 · legions 00514040 — minimums only: the non-zero XY and Z minimums reach the body (ac2-physics set_collision_minimums); the physics and placement masks and ethereal lists are not ported
- [ ] `CWPhysicsObject::GetCollisionState` · 0052ff60 · legions 00514060 — needs port: physics object get collision state; no citation in crates/
- [ ] `CWPhysicsObject::MissileInitPlaceAndFire` · 0052ff80 · legions 00514080 — needs port: physics object missile init place and fire; no citation in crates/
- [ ] `CWPhysicsObject::MissileInitialize` · 0052ffa0 · legions 005140a0 — needs port: physics object missile initialize; no citation in crates/
- [ ] `CWPhysicsObject::MissilePlaceAndFire` · 0052ffc0 · legions 005140c0 — needs port: physics object missile place and fire; no citation in crates/
- [ ] `CWPhysicsObject::MissilePlace` · 0052ffe0 · legions 005140e0 — needs port: physics object missile place; no citation in crates/
- [ ] `CWPhysicsObject::MissileFireOnly` · 00530000 · legions 00514100 — needs port: physics object missile fire only; no citation in crates/
- [ ] `CWPhysicsObject::MissileInquireDirection` · 00530020 · legions 00514120 — needs port: physics object missile inquire direction; no citation in crates/
- [ ] `CWPhysicsObject::MissileInquireCurrentVelocity` · 00530040 · legions 00514140 — needs port: physics object missile inquire current velocity; no citation in crates/
- [ ] `CWPhysicsObject::MissileRetarget` · 00530060 · legions 00514160 — needs port: physics object missile retarget; no citation in crates/
- [ ] `CWPhysicsObject::InitVisualDesc` · 00530080 · legions 00514180 — needs port: physics object init visual desc; no citation in crates/
- [ ] `CWPhysicsObject::CombineAndApplyVisualDesc` · 005300b0 · legions 005141b0 — needs port: physics object combine and apply visual desc; no citation in crates/
- [ ] `CWPhysicsObject::SetLocalVisualDesc` · 005300d0 · legions 005141d0 — needs port: physics object set local visual desc; no citation in crates/
- [ ] `CWPhysicsObject::ApplyAdditiveVisualDesc` · 005300f0 · legions 005141f0 — needs port: physics object apply additive visual desc; no citation in crates/
- [ ] `CWPhysicsObject::ApplySubtractiveVisualDesc` · 00530110 · legions 00514210 — needs port: physics object apply subtractive visual desc; no citation in crates/
- [ ] `CWPhysicsObject::ApplyVisualDescToPhysicsObj` · 00530130 · legions 00514230 — needs port: physics object apply visual desc to physics obj; no citation in crates/
- [x] `CWPhysicsObject::RecvEvt_UpdateVisualDesc` · 00530150 · legions 00514250 — implemented: cited at crates/ac2-weenie/src/objects.rs:213
- [ ] `CWPhysicsObject::ForceVisualDesc` · 00530190 · legions 00514290 — needs port: physics object force visual desc; no citation in crates/
- [x] `CWPhysicsObject::DoFX` · 005301b0 · legions 005142b0 — ac2-script host/interface.rs -> ac2-client scripts_io apply_script_physics -> ac2-physics PhysicsBody setters/do_fx/stop_fx; tests physics_obj_tests, host/tests
- [x] `CWPhysicsObject::StopFX` · 005301d0 · legions 005142d0 — ac2-script host/interface.rs -> ac2-client scripts_io apply_script_physics -> ac2-physics PhysicsBody setters/do_fx/stop_fx; tests physics_obj_tests, host/tests
- [ ] `CWPhysicsObject::DoTargettedFX` · 005301f0 · legions 005142f0 — needs port: physics object do targetted fx; no citation in crates/
- [~] `CWPhysicsObject::DoBehavior` · 00530220 — Player body only: the latent start reaches the player's body with its context; other objects finish at once (error 0, 0x5d for a refused behaviour). The story manager's HandleBehaviorDone path is not modelled. Rig: the door/lever rig's DoMode arrives as a server event, not through this efun, so no rig covers the latent wait; the host test does.
- [ ] `CWPhysicsObject::DoBehaviorNonLatent` · 005302c0 · legions 00514360 — needs port: physics object do behavior non latent; no citation in crates/
- [~] `CWPhysicsObject::ReleaseBehavior` · 005302f0 — ac2-script host/interface.rs do_mode, do_behavior and release_behavior queue the server event (DoMode, DoBehavior, ReleaseBehavior with the BehaviorParams native). Not modelled: the latent wait (retail returns Interrupted until the mode or behaviour is reached, then resumes the context on ModeDone); the script continues at once
- [~] `CWPhysicsObject::DoMode` · 00530390 — Player body only: the latent start reaches the player's body with its context; other objects finish at once (error 0, 0x5d for a refused behaviour). The story manager's HandleBehaviorDone path is not modelled. Rig: the door/lever rig's DoMode arrives as a server event, not through this efun, so no rig covers the latent wait; the host test does.
- [ ] `CWPhysicsObject::DoModeNonLatent` · 00530420 · legions 00514420 — needs port: physics object do mode non latent; no citation in crates/
- [x] `CWPhysicsObject::SetMode` · 00530440 · legions 00514440 — ac2-script host/interface.rs set_mode and stop_behavior queue the server event for the weenie's physics object (answer 1 with an owner); tests host/tests.rs physics_mode_and_behavior_efuns_queue_their_server_events
- [ ] `CWPhysicsObject::DoSlider` · 00530460 — needs port: physics object do slider; no citation in crates/
- [ ] `CWPhysicsObject::DoSliderNonLatent` · 00530500 · legions 005144a0 — needs port: physics object do slider non latent; no citation in crates/
- [x] `CWPhysicsObject::StopBehavior` · 00530530 · legions 005144d0 — ac2-script host/interface.rs set_mode and stop_behavior queue the server event for the weenie's physics object (answer 1 with an owner); tests host/tests.rs physics_mode_and_behavior_efuns_queue_their_server_events
- [ ] `CWPhysicsObject::StopLayer` · 00530550 · legions 005144f0 — needs port: physics object stop layer; no citation in crates/
- [ ] `CWPhysicsObject::StopMoveToCancelsBehaviors` · 00530570 · legions 00514510 — needs port: physics object stop move to cancels behaviors; no citation in crates/
- [ ] `CWPhysicsObject::StopAllWeenieBehaviors` · 00530590 · legions 00514530 — needs port: physics object stop all weenie behaviors; no citation in crates/
- [ ] `CWPhysicsObject::LookAtSelection` · 005305b0 · legions 00514550 — needs port: selection look-at not modelled: CWPhysicsObject::LookAtSelection (ac2-ui selection_notices.rs says so)
- [x] `CWPhysicsObject::LookAt` · 005305e0 · legions 00514580 — LookAt chain: ac2-physics look_at.rs (LookAtManager), physics_obj_look.rs (CPhysicsObj::LookAt/LookAtDir), ac2-script efun 0x0080002A, ac2-client motion.rs
- [ ] `CWPhysicsObject::AddTargetedImpulse` · 00530610 · legions 005145b0 — needs port: physics object add targeted impulse; no citation in crates/
- [ ] `CWPhysicsObject::GetElement` · 00530650 · legions 005145f0 — needs port: physics object get element; no citation in crates/
- [ ] `CWPhysicsObject::GetDesiredMode` · 00530670 · legions 00514610 — needs port: physics object get desired mode; no citation in crates/
- [ ] `CWPhysicsObject::GetMode` · 00530690 · legions 00514630 — needs port: physics object get mode; no citation in crates/
- [ ] `CWPhysicsObject::IsDoingBehavior` · 005306b0 · legions 00514650 — needs port: physics object is doing behavior; no citation in crates/
- [ ] `CWPhysicsObject::HasMotionInterpreter` · 005306d0 · legions 00514670 — needs port: physics object has motion interpreter; no citation in crates/
- [ ] `CWPhysicsObject::GetLastDoMotionValues` · 005306f0 · legions 00514690 — needs port: physics object get last do motion values; no citation in crates/
- [ ] `CWPhysicsObject::HasRemoteDoMotionValues` · 00530710 · legions 005146b0 — needs port: physics object has remote do motion values; no citation in crates/
- [ ] `CWPhysicsObject::GetRemoteDoMotionValues` · 00530730 · legions 005146d0 — needs port: physics object get remote do motion values; no citation in crates/
- [ ] `CWPhysicsObject::GetDesiredMotionValues` · 00530750 · legions 005146f0 — needs port: physics object get desired motion values; no citation in crates/
- [ ] `CWPhysicsObject::GetStandardMotionValues` · 00530770 · legions 00514710 — needs port: physics object get standard motion values; no citation in crates/
- [ ] `CWPhysicsObject::GetMyPosition` · 00530790 · legions 00514730 — needs port: physics object get my position; no citation in crates/
- [ ] `CWPhysicsObject::GetDrawingPosition` · 005307b0 · legions 00514750 — needs port: physics object get drawing position; no citation in crates/
- [ ] `CWPhysicsObject::GetBoundingBox` · 005307d0 · legions 00514770 — needs port: physics object get bounding box; no citation in crates/
- [ ] `CWPhysicsObject::GetWorldMatrix` · 005307f0 · legions 00514790 — needs port: physics object get world matrix; no citation in crates/
- [ ] `CWPhysicsObject::IsVisible` · 00530810 · legions 005147b0 — needs port: physics object is visible; no citation in crates/
- [ ] `CWPhysicsObject::IsOutside` · 00530820 · legions 005147c0 — needs port: physics object is outside; no citation in crates/
- [ ] `CWPhysicsObject::IsInWorld` · 00530850 · legions 005147f0 — needs port: physics object is in world; no citation in crates/
- [ ] `CWPhysicsObject::LocalToGlobal` · 00530870 · legions 00514810 — needs port: physics object local to global; no citation in crates/
- [ ] `CWPhysicsObject::CalculateDistance` · 00530890 · legions 00514830 — needs port: physics object calculate distance; no citation in crates/
- [ ] `CWPhysicsObject::GetVelocityScale` · 005308c0 · legions 00514860 — needs port: physics object get velocity scale; no citation in crates/
- [x] `CWPhysicsObject::SetVelocityScale` · 005308e0 · legions 00514880 — ac2-script host/interface.rs -> ac2-client scripts_io apply_script_physics -> ac2-physics PhysicsBody setters/do_fx/stop_fx; tests physics_obj_tests, host/tests
- [ ] `CWPhysicsObject::GetAccelerationScale` · 00530900 · legions 005148a0 — needs port: physics object get acceleration scale; no citation in crates/
- [x] `CWPhysicsObject::SetAccelerationScale` · 00530920 · legions 005148c0 — ac2-script host/interface.rs -> ac2-client scripts_io apply_script_physics -> ac2-physics PhysicsBody setters/do_fx/stop_fx; tests physics_obj_tests, host/tests
- [ ] `CWPhysicsObject::GetJumpScale` · 00530940 · legions 005148e0 — needs port: physics object get jump scale; no citation in crates/
- [x] `CWPhysicsObject::SetJumpScale` · 00530960 · legions 00514900 — ac2-script host/interface.rs -> ac2-client scripts_io apply_script_physics -> ac2-physics PhysicsBody setters/do_fx/stop_fx; tests physics_obj_tests, host/tests
- [ ] `CWPhysicsObject::Notify_DoMode` · 00530980 · legions 00514920 — needs port: physics object notify do mode; no citation in crates/
- [x] `CWPhysicsObject::SetNoDrawPhysics` · 005309a0 · legions 00514940 — ac2-script host::interface set_no_draw_physics (efun 0x800057 -> CPhysicsObj::set_nodraw): queued for the client, which keeps it per object (Client::is_no_draw) and gives the camera subject its no_draw; combat_7160 (the respawn clears it via CPlayer 0x85). The renderer does not read it for other objects
- [ ] `CWPhysicsObject::SetCloaked` · 005309c0 · legions 00514960 — needs port: physics object set cloaked; no citation in crates/
- [ ] `CWPhysicsObject::SetUndetectable` · 005309e0 · legions 00514980 — needs port: physics object set undetectable; no citation in crates/
- [ ] `CWPhysicsObject::InqCurrentProperties` · 00530ae0 · legions 00514a80 — needs port: physics object inq current properties; no citation in crates/
- [ ] `CWPhysicsObject::InqCurrentProperty` · 00530b00 · legions 00514aa0 — needs port: physics object inq current property; no citation in crates/
- [ ] `CWPhysicsObject::GetHeight` · 00530b20 · legions 00514ac0 — needs port: physics object get height; no citation in crates/
- [ ] `CWPhysicsObject::GetRadius` · 00530b40 · legions 00514ae0 — needs port: physics object get radius; no citation in crates/
- [ ] `CWPhysicsObject::GetTopLevelRadius` · 00530b60 · legions 00514b00 — needs port: physics object get top level radius; no citation in crates/
- [ ] `CWPhysicsObject::GetTopLevelHeight` · 00530b80 · legions 00514b20 — needs port: physics object get top level height; no citation in crates/
- [ ] `CWPhysicsObject::IsMoving` · 00530ba0 · legions 00514b40 — needs port: physics object is moving; no citation in crates/
- [ ] `CWPhysicsObject::IsUndetectable` · 00530bc0 · legions 00514b60 — needs port: physics object is undetectable; no citation in crates/
- [ ] `CWPhysicsObject::IsMovementFrozen` · 00530be0 · legions 00514b80 — needs port: physics object is movement frozen; no citation in crates/
- [ ] `CWPhysicsObject::SetMovementFrozen` · 00530c00 · legions 00514ba0 — needs port: physics object set movement frozen; no citation in crates/
- [ ] `CWPhysicsObject::IsAnimationFrozen` · 00530c30 · legions 00514bd0 — needs port: physics object is animation frozen; no citation in crates/
- [ ] `CWPhysicsObject::SetAnimationFrozen` · 00530c50 · legions 00514bf0 — needs port: physics object set animation frozen; no citation in crates/
- [ ] `CWPhysicsObject::TogglePlaceMarkerObjects` · 00530c80 · legions 00514c20 — needs port: physics object toggle place marker objects; no citation in crates/
- [ ] `CWPhysicsObject::StopAllCameraFX` · 00530cb0 · legions 00514c50 — needs port: camera FX stop: the client has no camera effect manager (ac2-script host/interface.rs says a known gap)
- [ ] `CWPhysicsObject::DoStory` · 00530cc0 · legions 00514c60 — needs port: physics object do story; no citation in crates/
- [ ] `CWPhysicsObject::InitPhysics` · 00530ce0 · legions 00514c80 — needs port: physics object init physics; no citation in crates/
- [x] `CWPhysicsObject::GetLocalVisualDesc` · 00530d50 · legions 00514cf0 — Returns a new VisualDesc native (word 2, REF) copied from the host's shared appearance (ScriptHost::set_appearance from the session's object table); a weenie never shared has the default description. The sell body reads it through gVisualDesc::GetTotalIconDesc.
- [ ] `CWPhysicsObject::AddImpulse` · 00530da0 · legions 00514d40 — needs port: physics object add impulse; no citation in crates/
- [ ] `CWPhysicsObject::AddImpulseLocal` · 00530de0 · legions 00514d80 — needs port: physics object add impulse local; no citation in crates/
- [ ] `CWPhysicsObject::InqCylSphere` · 00530e20 · legions 00514dc0 — needs port: physics object inq cyl sphere; no citation in crates/
