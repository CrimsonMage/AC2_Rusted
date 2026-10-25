# ac2-ui: UI framework, input maps, IME, game panels, maps

2247 retail functions: 346 done, 166 partial, 1716 to do, 19 skipped.
Legend and workflow: [README](README.md).

## `ac2/client/src/ui/gameplay/gm3dtextui.cpp.c` (5/53)

### BBox
- [x] `BBox::AdjustBBox` · 00426720 · legions 00426510 — ac2-physics objects.rs: static-object collision dispatch, containment radius, shadow box

### Box2D
- [ ] `Box2D::Box2D` · 004267a0 · legions 00426590
- [ ] `Box2D::Box2D` · 004267d0 · legions 005a35f0
- [ ] `Box2D::Collide` · 00426800 · legions 004265c0

### gm3DTextUI
- [ ] `gm3DTextUI::ListenToMessage` · 004268a0 · legions 00426660
- [ ] `gm3DTextUI::RecvNotice_ItemUpdate` · 00426920
- [x] `gm3DTextUI::ShowReticle` · 00426930 · legions 004266f0 — ac2-ui reticle.rs + ac2-gpu ui.rs: the targeting reticle
- [~] `gm3DTextUI::CheckName` · 00426970 · legions 00426730 — ac2-ui reticle.rs: no off-screen indicator, meters, radar-look colour or quantity; name plates and damage text not ported
- [ ] `gm3DTextUI::DoStuffMeter` · 00426ae0 · legions 004268a0
- [ ] `gm3DTextUI::MakeNewData` · 00426b70 · legions 00426930

### BBox
- [ ] `BBox::BBox` · 00426d00 · legions 00426ac0
- [ ] `BBox::Pack` · 00426d40

### Position
- [ ] `Position::Position` · 00426d50 · legions 00426b10

### gm3DTextUI
- [~] `gm3DTextUI::InitReticle` · 00426d90 · legions 00426b50 — ac2-ui reticle.rs: no off-screen indicator, meters, radar-look colour or quantity; name plates and damage text not ported
- [ ] `gm3DTextUI::RecvNotice_UpdateSelectionInfo` · 00426ee0 · legions 00426ca0
- [ ] `gm3DTextUI::PlaceHere` · 00426f60 · legions 00426d20

### _STL::_Deque_iterator<BBox,_STL::_Nonconst_traits<BBox>_>
- [ ] `_STL::_Deque_iterator<BBox,_STL::_Nonconst_traits<BBox>_>::operator-` · 00427090 · legions 00426eb0
- [ ] `_STL::_Deque_iterator<BBox,_STL::_Nonconst_traits<BBox>_>::operator+` · 004270f0 · legions 00426e50

### SmartArray<Box2D>
- [ ] `SmartArray<Box2D>::AddToEnd` · 00427460 · legions 00427220

### _STL::deque<BBox,_STL::allocator<BBox>_>
- [ ] `_STL::deque<BBox,_STL::allocator<BBox>_>::~deque<BBox,_STL::allocator<BBox>_>` · 004280e0 · legions 00427ea0

### gm3DTextUI::NameTextInfo
- [ ] `gm3DTextUI::NameTextInfo::NameTextInfo` · 00428e00 · legions 00428bc0

### gm3DTextUI::DamageTextInfo
- [ ] `gm3DTextUI::DamageTextInfo::DamageTextInfo` · 00428e30

### gm3DTextUI
- [x] `gm3DTextUI::AverageBoundingBox` · 00428e60 · legions 00428c20 — ac2-ui reticle.rs + ac2-gpu ui.rs: the targeting reticle
- [x] `gm3DTextUI::ConvertObjectToWindow` · 004291a0 · legions 00428f60 — ac2-ui reticle.rs + ac2-gpu ui.rs: the targeting reticle
- [~] `gm3DTextUI::UpdateReticleInternal` · 00429320 · legions 004290e0 — ac2-ui reticle.rs: no off-screen indicator, meters, radar-look colour or quantity; name plates and damage text not ported
- [ ] `gm3DTextUI::CalcScreenPosition` · 00429ab0 · legions 00429870

### _STL::deque<BBox,_STL::allocator<BBox>_>
- [ ] `_STL::deque<BBox,_STL::allocator<BBox>_>::operator=` · 00429bd0 · legions 00429990

### gm3DTextUI
- [ ] `gm3DTextUI::UpdateNames` · 00429e20 · legions 00429be0
- [x] `gm3DTextUI::UpdateReticle` · 0042a1d0 · legions 00429f90 — ac2-ui reticle.rs + ac2-gpu ui.rs: the targeting reticle

### gm3DTextUI::DamageTextInfo
- [ ] `gm3DTextUI::DamageTextInfo::operator=` · 0042a290 · legions 0042a050

### SmartArray<gm3DTextUI::DamageTextInfo>
- [ ] `SmartArray<gm3DTextUI::DamageTextInfo>::RemoveOrderedByIndex` · 0042a4f0 · legions 0042a2b0

### gm3DTextUI
- [ ] `gm3DTextUI::UpdateDamage` · 0042a560 · legions 0042a320
- [ ] `gm3DTextUI::ClearNames` · 0042a980 · legions 0042a740
- [ ] `gm3DTextUI::RemoveName` · 0042aa40 · legions 0042a800
- [ ] `gm3DTextUI::RecvNotice_BeingDeleted` · 0042ab30 · legions 0042a8f0

### SmartArray<gm3DTextUI::DamageTextInfo>
- [ ] `SmartArray<gm3DTextUI::DamageTextInfo>::AddToEnd` · 0042ac30 · legions 0042a9f0

### SmartArray<gm3DTextUI::NameTextInfo>
- [ ] `SmartArray<gm3DTextUI::NameTextInfo>::AddToEnd` · 0042aca0 · legions 0042aa60

### gm3DTextUI
- [ ] `gm3DTextUI::CleanUp` · 0042ad00 · legions 0042aac0
- [~] `gm3DTextUI::Update` · 0042adf0 · legions 0042abb0 — ac2-ui reticle.rs: no off-screen indicator, meters, radar-look colour or quantity; name plates and damage text not ported
- [ ] `gm3DTextUI::AddName` · 0042ae40 · legions 0042ac00
- [ ] `gm3DTextUI::RecvNotice_CreateObject` · 0042b0c0
- [ ] `gm3DTextUI::RecvNotice_Do3DDamage` · 0042b150
- [ ] `gm3DTextUI::RecvNotice_PrecludeFromTop` · 0042b340
- [ ] `gm3DTextUI::RecvNotice_PrecludeFromBottom` · 0042b3a0
- [ ] `gm3DTextUI::~gm3DTextUI` · 0042b400 · legions 0042b1c0
- [ ] `gm3DTextUI::Init` · 0042b510 · legions 0042b2d0
- [ ] `gm3DTextUI::InitNames` · 0042b610 · legions 0042b3d0
- [ ] `gm3DTextUI::RecvNotice_ChangeRadarLook` · 0042b6a0
- [~] `gm3DTextUI::RecvNotice_SelectionChanged` · 0042b730 — ac2-ui reticle.rs: no off-screen indicator, meters, radar-look colour or quantity; name plates and damage text not ported
- [ ] `gm3DTextUI::ToggleNames` · 0042b7e0 · legions 0042b5a0
- [ ] `gm3DTextUI::gm3DTextUI` · 0042b840 · legions 0042b600
- [ ] `gm3DTextUI::ListenToMessage` · 0042ba40 · legions 0042b800
- [ ] `gm3DTextUI::RecvNotice_ReloadOptions` · 0042ba70

## `ac2/client/src/ui/gameplay/gmbookui.cpp.c` (0/19)

### gmBookUI
- [ ] `gmBookUI::CreateManagedElements` · 0041eaa0 · legions 0041e8c0
- [ ] `gmBookUI::Show` · 0041eb50 · legions 0041e970
- [ ] `gmBookUI::SynchSliderBar` · 0041ebc0 · legions 0041e9e0
- [ ] `gmBookUI::PreviousPage` · 0041ec50 · legions 0041ea70
- [ ] `gmBookUI::NextPage` · 0041ec70 · legions 0041ea90
- [ ] `gmBookUI::SetPageFromSlider` · 0041ec90 · legions 0041eab0
- [ ] `gmBookUI::SetMode` · 0041ed20 · legions 0041eb40
- [ ] `gmBookUI::Reset` · 0041ed90 · legions 0041ebb0

### PStringBase<unsigned_short>
- [ ] `PStringBase<unsigned_short>::operator[]` · 0041edf0 · legions 0041ec10

### gmBookUI
- [ ] `gmBookUI::gmBookUI` · 0041ee70 · legions 0041ec90
- [ ] `gmBookUI::~gmBookUI` · 0041ef90 · legions 0041edb0
- [ ] `gmBookUI::UpdateBook` · 0041f010 · legions 0041ee30

### PStringBaseIter<unsigned_short>
- [ ] `PStringBaseIter<unsigned_short>::GetStrLen` · 0041f0d0 · legions 0041eeb0

### gmBookUI
- [ ] `gmBookUI::AddNewlines` · 0041f480 · legions 0041f260
- [ ] `gmBookUI::TruncateText` · 0041f510 · legions 0041f2f0
- [ ] `gmBookUI::ParseText` · 0041f7c0 · legions 0041f5a0
- [ ] `gmBookUI::RecvNotice_UseBook` · 0041ff30 · legions 0041fd30
- [ ] `gmBookUI::SendBookToClipboard` · 00420010 · legions 0041fe10
- [ ] `gmBookUI::ListenToMessage` · 00420320 · legions 00420120

## `ac2/client/src/ui/gameplay/gmcameraui.cpp.c` (3/7)

### gmCameraUI
- [ ] `gmCameraUI::~gmCameraUI` · 0042baa0 · legions 0042b860
- [x] `gmCameraUI::ListenToMessage` · 0042bb00 · legions 0042b8c0 — ac2-ui mouse.rs / button.rs: UI hit tests, presses and buttons
- [ ] `gmCameraUI::SendKey` · 0042bb50
- [x] `gmCameraUI::UpdateCamera` · 0042bbb0 · legions 0042b970 — ac2-ui camera_ui.rs (no UI element hit test: the wheel always reaches the world)
- [ ] `gmCameraUI::gmCameraUI` · 0042bc20 · legions 0042b9e0
- [x] `gmCameraUI::ListenToMessage` · 0042bce0 · legions 0042baa0 — ac2-ui camera_ui.rs (no UI element hit test: the wheel always reaches the world)
- [ ] `gmCameraUI::RecvNotice_UpdateCanMouseLook` · 0042bf50

## `ac2/client/src/ui/gameplay/gmexternalcontainerui.cpp.c` (0/8)

### gmExternalContainerUI
- [ ] `gmExternalContainerUI::~gmExternalContainerUI` · 00420400 · legions 00420200
- [ ] `gmExternalContainerUI::ListenToMessage` · 00420460 · legions 00420290
- [ ] `gmExternalContainerUI::RecvNotice_CreateObject` · 00420520
- [ ] `gmExternalContainerUI::RecvNotice_BeingDeleted` · 00420570
- [ ] `gmExternalContainerUI::PositionWindow` · 004205e0 · legions 00420410
- [ ] `gmExternalContainerUI::RecvNotice_ContainerStateUpdate` · 00420860 · legions 00420690
- [ ] `gmExternalContainerUI::Init` · 004208d0 · legions 00420700
- [ ] `gmExternalContainerUI::gmExternalContainerUI` · 00420980 · legions 004207b0

## `ac2/client/src/ui/gameplay/gmgameplayui.cpp.c` (7/58)

### gmGamePlayUI
- [ ] `gmGamePlayUI::Show` · 00407910 · legions 00407620
- [ ] `gmGamePlayUI::SetErrorMsg` · 00407940 · legions 00407650
- [ ] `gmGamePlayUI::RecvNotice_EndCharacterSession` · 00407980
- [ ] `gmGamePlayUI::RecvNotice_Logoff` · 00407990
- [x] `gmGamePlayUI::RecvNotice_NextFrameDrawn` · 004079b0 — OnlineSession::scene_ready (SceneRenderingComplete) on the frame drawn after each PlayerReadyToEnterWorld
- [ ] `gmGamePlayUI::RecvNotice_BeingDeleted` · 004079c0
- [ ] `gmGamePlayUI::MakePleaseWaitDialog` · 004079e0 · legions 004076f0
- [ ] `gmGamePlayUI::MakeResurrectDialog` · 00407a40 · legions 00407750

### PlayingSoundContext
- [ ] `PlayingSoundContext::Stop` · 00407b80 · legions 004078b0

### gmGamePlayUI
- [ ] `gmGamePlayUI::RecvNotice_PlayerTeleportStart` · 00407bf0
- [ ] `gmGamePlayUI::RecvNotice_PortalSpace` · 00407c30
- [ ] `gmGamePlayUI::RecvNotice_UpdateDeathState` · 00407c80
- [ ] `gmGamePlayUI::RecvNotice_ResurrectionRequestedByOther` · 00407cf0
- [ ] `gmGamePlayUI::RecvNotice_CloseConfirmationDialog` · 00407d00 · legions 00407a30
- [ ] `gmGamePlayUI::RecvNotice_CloseResurrectDialog` · 00407d70
- [ ] `gmGamePlayUI::ChildKeyPress` · 00407f70 · legions 00407ca0
- [ ] `gmGamePlayUI::Update` · 00408020 · legions 00407ce0
- [ ] `gmGamePlayUI::RecvNotice_DisplayMessage` · 00408080
- [~] `gmGamePlayUI::SetupChildren` · 004080c0 · legions 00407d80 — ac2-ui: no transitions, timed media, mouse, listeners or dirty rects; A8R8G8B8 blits only; HUD roots from SetupChildren's radar/selection/statusbar/toolbar
- [ ] `gmGamePlayUI::DoPleaseWaitCancel` · 004084a0 · legions 00408160
- [ ] `gmGamePlayUI::Init` · 00408500 · legions 004081c0
- [ ] `gmGamePlayUI::RecvNotice_CloseWaitDialog` · 00408640

### HashTable<unsigned_long,int>
- [ ] `HashTable<unsigned_long,int>::~HashTable<unsigned_long,int>` · 00408660 · legions 00408320

### gmGamePlayUI
- [ ] `gmGamePlayUI::RecvNotice_LoginFinished` · 00408840 · legions 00408580
- [ ] `gmGamePlayUI::RecvNotice_CloseMessageDialog` · 00408860
- [ ] `gmGamePlayUI::CleanupChildren` · 00408890 · legions 004085d0

### PHashTable<unsigned_long,UILocationData>
- [ ] `PHashTable<unsigned_long,UILocationData>::~PHashTable<unsigned_long,UILocationData>` · 004088f0 · legions 00408630

### PList<gmSceneInfo>
- [ ] `PList<gmSceneInfo>::PList<gmSceneInfo>` · 00408a20 · legions 00408760
- [ ] `PList<gmSceneInfo>::Pack` · 00408a60 · legions 004087a0
- [ ] `PList<gmSceneInfo>::UnPack` · 00408ab0 · legions 00408804

### gmGamePlayUI
- [ ] `gmGamePlayUI::gmGamePlayUI` · 00408c60 · legions 004089a0

### UIFramework
- [ ] `UIFramework::Shown` · 00408e60 · legions 00408ba0

### gmGamePlayUI
- [ ] `gmGamePlayUI::Create` · 00408e70
- [~] `gmGamePlayUI::EndSession` · 00408e90 · legions 00408bd0 — LogOffCharacter only; no UI or game-option upload
- [ ] `gmGamePlayUI::RecvNotice_FinishPlaySequence` · 00408f50 · legions 00408c90
- [ ] `gmGamePlayUI::RecvNotice_PlaySequence` · 00408fa0 · legions 00408ce0
- [ ] `gmGamePlayUI::CheckPortalScene` · 00409000 · legions 00408d40

### PList<gmSceneInfo>
- [ ] `PList<gmSceneInfo>::PList<gmSceneInfo>` · 004090e0 · legions 00408e20

### gmGamePlayUI
- [ ] `gmGamePlayUI::~gmGamePlayUI` · 00409110 · legions 00408e50
- [ ] `gmGamePlayUI::Register` · 00409250 · legions 00408f90
- [~] `gmGamePlayUI::RecvNotice_PlayerReadyToEnterWorld` · 00409270 — asks for the next-frame-drawn notice (SetWantNoticeOnNextFrameDrawn); no portal scenes
- [ ] `gmGamePlayUI::RecvNotice_PortalScene` · 00409330

### PHashTable<unsigned_long,UILocationData>
- [x] `PHashTable<unsigned_long,UILocationData>::StreamPack` · 004093a0 — ac2-net protocol::native pack+unpack (Legions layouts; WState graphs round-trip byte-exact); ac2-script still decodes its own copy until it switches

### PHashTable<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>
- [ ] `PHashTable<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>::StreamPack` · 004095e0 · legions 00409320

### IntrusiveHashData<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>_*>
- [ ] `IntrusiveHashData<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>_*>::IntrusiveHashData<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>_*>` · 004098b0 · legions 004095f0

### (free functions)
- [ ] `STREAMPACK` · 004098e0 · legions 00409650

### PStringBase<unsigned_short>
- [x] `PStringBase<unsigned_short>::GetPackSize` · 00409960 · legions 004096d0
- [x] `PStringBase<unsigned_short>::Pack` · 00409990 · legions 00409700
- [x] `PStringBase<unsigned_short>::UnPack` · 00409be0 · legions 00409950 — ac2-core pstring.rs: the DAT string descramble (ac2-ui, ac2-db use it)

### PHashTable<unsigned_long,float>
- [x] `PHashTable<unsigned_long,float>::StreamPack` · 00409e00

### PHashTable<unsigned_long,PHashTable<unsigned_long,UILocationData>_>
- [ ] `PHashTable<unsigned_long,PHashTable<unsigned_long,UILocationData>_>::~PHashTable<unsigned_long,PHashTable<unsigned_long,UILocationData>_>` · 0040a190 · legions 0040a020
- [x] `PHashTable<unsigned_long,PHashTable<unsigned_long,UILocationData>_>::StreamPack` · 0040a1f0 · legions 0040a080 — ac2-net protocol::native pack+unpack (Legions layouts; WState graphs round-trip byte-exact); ac2-script still decodes its own copy until it switches

### PHashTable<unsigned_long,UILocationData>
- [ ] `PHashTable<unsigned_long,UILocationData>::PHashTable<unsigned_long,UILocationData>` · 0040a440 · legions 0040a2c0

### UISaveLocations
- [ ] `UISaveLocations::~UISaveLocations` · 0040a540 · legions 0040a3c0

### GameplayOptionsProfile
- [ ] `GameplayOptionsProfile::~GameplayOptionsProfile` · 0040a590 · legions 0040a410

### gmGamePlayUI
- [ ] `gmGamePlayUI::RecvNotice_SaveLocationsChanged` · 0040a790
- [ ] `gmGamePlayUI::ListenToMessage` · 0040a810 · legions 0040a6d0
- [ ] `gmGamePlayUI::RecvNotice_ReloadOptions` · 0040a960

## `ac2/client/src/ui/gameplay/gmingredientui.cpp.c` (0/39)

### gmCObjectMaint
- [ ] `gmCObjectMaint::GetWeenieObject` · 00415870 · legions 00415660

### gmIngredientUI
- [ ] `gmIngredientUI::DestroyManagedElements` · 00415890 · legions 00415680
- [ ] `gmIngredientUI::UpdateCraftButton` · 00415900 · legions 004156f0

### (free functions)
- [ ] `gmIngredientUI_DDCB` · 00415b70

### gmIngredientUI
- [ ] `gmIngredientUI::UpdateCost` · 00415be0 · legions 004159d0
- [ ] `gmIngredientUI::Scale_` · 00415c50 · legions 00415a40
- [ ] `gmIngredientUI::CreateManagedElements` · 00415cd0 · legions 00415ac0
- [ ] `gmIngredientUI::Show` · 00415e60 · legions 00415c50
- [ ] `gmIngredientUI::RecvNotice_UpdateMountState` · 00415ee0 · legions 00415cd0
- [ ] `gmIngredientUI::AutoSizeIngredientUI` · 00415f10 · legions 00415d00
- [ ] `gmIngredientUI::SetPending` · 00415fc0 · legions 00415db0
- [ ] `gmIngredientUI::UpdateApparentDifficulty` · 004160b0 · legions 00415ea0
- [ ] `gmIngredientUI::RecvNotice_UpdateCraftSkill` · 00416280 · legions 00416070
- [ ] `gmIngredientUI::~gmIngredientUI` · 004164b0 · legions 004162b0

### HashTable<unsigned_long,InstanceID>
- [ ] `HashTable<unsigned_long,InstanceID>::operator=` · 00416640 · legions 004a16f0

### PList<InstanceID>
- [ ] `PList<InstanceID>::Pack` · 00416740 · legions 004164e0
- [ ] `PList<InstanceID>::UnPack` · 00416790

### gmIngredientUI
- [ ] `gmIngredientUI::MultiOpAddIngredientSlot` · 00416900 · legions 004166c0
- [ ] `gmIngredientUI::HandleItemRemoval` · 00416ad0 · legions 00416890
- [ ] `gmIngredientUI::CheckItem` · 00416be0 · legions 004169a0
- [ ] `gmIngredientUI::RecvNotice_BeingDeleted` · 00416d20
- [ ] `gmIngredientUI::RecvNotice_UpdateInventory` · 00416da0
- [ ] `gmIngredientUI::RecvNotice_UpdateContainer` · 00416df0 · legions 00416bb0
- [ ] `gmIngredientUI::RecvNotice_ItemUpdate` · 00416e40 · legions 00416c00
- [ ] `gmIngredientUI::Init` · 00416ea0 · legions 00416c60
- [ ] `gmIngredientUI::MultiOpRemoveIngredientSlot` · 00417290 · legions 00417050
- [ ] `gmIngredientUI::UpdateIngredientSlot` · 004173f0 · legions 004171b0
- [ ] `gmIngredientUI::HandleListBoxDrop` · 00417530 · legions 004172f0
- [ ] `gmIngredientUI::UpdateIngredients` · 004176c0 · legions 00417480
- [ ] `gmIngredientUI::gmIngredientUI` · 00417800 · legions 004175c0
- [ ] `gmIngredientUI::RecvNotice_OpenIngredientPanel` · 00417ac0 · legions 00417880
- [ ] `gmIngredientUI::UpdateRecipeFromSpinnerChange` · 00417b00 · legions 004178c0
- [ ] `gmIngredientUI::AutoPopulateIngredients` · 00417d20 · legions 00417ae0
- [ ] `gmIngredientUI::DoClose` · 00418060 · legions 00417e20
- [ ] `gmIngredientUI::ListenToMessage` · 00418150 · legions 00417f10
- [ ] `gmIngredientUI::ListenToMessage` · 00418210 · legions 00417fd0
- [ ] `gmIngredientUI::RecvNotice_AddIngredient_Done` · 004185d0
- [ ] `gmIngredientUI::RecvNotice_ExecuteCraft_Done` · 00418780
- [ ] `gmIngredientUI::RecvNotice_CloseConfirmationDialog` · 00418880

## `ac2/client/src/ui/gameplay/gmminifellowui.cpp.c` (0/6)

### gmMiniFellowUI
- [ ] `gmMiniFellowUI::~gmMiniFellowUI` · 00421c50 · legions 00421ac0
- [ ] `gmMiniFellowUI::LocationApplied` · 00421ca0 · legions 0043eb50
- [ ] `gmMiniFellowUI::ListenToMessage` · 00421cc0 · legions 00421b10
- [ ] `gmMiniFellowUI::RecvNotice_UpdateFellowship` · 00421d10 · legions 00421b60
- [ ] `gmMiniFellowUI::RecvNotice_ClearFellowship` · 00421da0 · legions 00421bf0
- [ ] `gmMiniFellowUI::gmMiniFellowUI` · 00421dd0 · legions 00421c20

## `ac2/client/src/ui/gameplay/gmoverviewmapui.cpp.c` (0/6)

### gmOverviewMapUI
- [ ] `gmOverviewMapUI::CreateManagedElements` · 0043e150 · legions 0043e9a0
- [ ] `gmOverviewMapUI::~gmOverviewMapUI` · 0043e1c0 · legions 0043ea10
- [ ] `gmOverviewMapUI::Show` · 0043e230 · legions 0043ea80
- [ ] `gmOverviewMapUI::ShowMap` · 0043e290 · legions 0043eae0
- [ ] `gmOverviewMapUI::ListenToMessage` · 0043e2c0 · legions 0043eb10
- [ ] `gmOverviewMapUI::gmOverviewMapUI` · 0043e300 · legions 0043eb70

## `ac2/client/src/ui/gameplay/gmpetui.cpp.c` (0/18)

### gmPetUI
- [ ] `gmPetUI::Show` · 004188d0 · legions 00418690
- [ ] `gmPetUI::GetNextMode` · 00418920 · legions 004186e0
- [ ] `gmPetUI::GetPreviousMode` · 00418950 · legions 00418710

### _STL::_Node_Alloc_Lock<1,0>
- [ ] `_STL::_Node_Alloc_Lock<1,0>::~_Node_Alloc_Lock<1,0>` · 004189c0 · legions 00418780

### gmPetUI
- [ ] `gmPetUI::UpdateState` · 00418cb0 · legions 00418a70
- [ ] `gmPetUI::RecvNotice_ReloadGameplayOptions` · 00418e20
- [ ] `gmPetUI::RecvNotice_RemovePet` · 00419130
- [ ] `gmPetUI::RecvNotice_SelectionChanged` · 004191d0
- [ ] `gmPetUI::~gmPetUI` · 00419430 · legions 00419270
- [ ] `gmPetUI::Refresh` · 00419580 · legions 004193c0
- [ ] `gmPetUI::RecvNotice_PetIsUnderAttack` · 00419730
- [ ] `gmPetUI::ListenToMessage` · 00419810 · legions 00419650
- [ ] `gmPetUI::RequestSetPetMode` · 00419840 · legions 00419680
- [ ] `gmPetUI::RecvNotice_AddPet` · 004199d0
- [ ] `gmPetUI::RecvNotice_UpdatePetMode` · 00419b70 · legions 004199b0
- [ ] `gmPetUI::ListenToMessage` · 00419c20 · legions 00419a60
- [ ] `gmPetUI::gmPetUI` · 0041a040 · legions 00419e80
- [ ] `gmPetUI::RecvNotice_UpdatePetName` · 0041a3a0

## `ac2/client/src/ui/gameplay/gmradarui.cpp.c` (0/35)

### Position
- [ ] `Position::get_heading` · 00424fe0 · legions 00424dd0

### RadarInfo
- [ ] `RadarInfo::RadarInfo` · 00425000

### gmRadarUI
- [ ] `gmRadarUI::Show` · 00425040 · legions 00424e30
- [ ] `gmRadarUI::LocationApplied` · 00425090 · legions 00424e80
- [ ] `gmRadarUI::RecvNotice_SelectionChanged` · 004250b0
- [ ] `gmRadarUI::DrawPoint` · 004250d0 · legions 00424ec0
- [ ] `gmRadarUI::DrawL` · 00425110 · legions 00424f00
- [ ] `gmRadarUI::DrawEdges` · 00425180 · legions 00424f70
- [ ] `gmRadarUI::DrawCorners` · 00425240 · legions 00425030
- [ ] `gmRadarUI::DrawT` · 00425310 · legions 00425100
- [ ] `gmRadarUI::DrawFace` · 00425380 · legions 00425170
- [ ] `gmRadarUI::DrawSelected` · 00425420 · legions 00425210
- [ ] `gmRadarUI::SetCoordButtonToolTip` · 00425500 · legions 004252f0

### SmartArray<RadarInfo>
- [ ] `SmartArray<RadarInfo>::RemoveUnOrdered` · 00425560 · legions 00425350

### SmartArray<GrooveInfo>
- [ ] `SmartArray<GrooveInfo>::RemoveUnOrdered` · 004255b0 · legions 004253a0

### gmRadarUI
- [ ] `gmRadarUI::~gmRadarUI` · 004256b0 · legions 004254a0
- [ ] `gmRadarUI::UpdateGroove` · 00425740 · legions 00425530
- [ ] `gmRadarUI::RemoveObject` · 00425850 · legions 00425640
- [ ] `gmRadarUI::RecvNotice_BeingDeleted` · 004258d0 · legions 004256c0
- [ ] `gmRadarUI::DrawCross` · 00425900 · legions 004256f0
- [ ] `gmRadarUI::DrawX` · 00425950 · legions 00425740
- [ ] `gmRadarUI::DrawHollow` · 004259a0 · legions 00425790
- [ ] `gmRadarUI::ListenToMessage` · 004259f0 · legions 004257e0
- [ ] `gmRadarUI::DrawFill` · 00425a30 · legions 00425820

### SmartArray<RadarInfo>
- [ ] `SmartArray<RadarInfo>::AddToEnd` · 00425a90 · legions 00425880

### gmRadarUI
- [ ] `gmRadarUI::DrawBlip` · 00425af0 · legions 004258e0
- [ ] `gmRadarUI::Update` · 00425c00 · legions 004259f0
- [ ] `gmRadarUI::ListenToMessage` · 00426060 · legions 00425e50
- [ ] `gmRadarUI::AddObject` · 00426190 · legions 00425f80
- [ ] `gmRadarUI::RecvNotice_CreateObject` · 00426270
- [ ] `gmRadarUI::Init` · 004262a0 · legions 00426090
- [ ] `gmRadarUI::RecvNotice_ChangeRadarLook` · 004263a0
- [ ] `gmRadarUI::ReloadOptions` · 00426400
- [~] `gmRadarUI::gmRadarUI` · 00426450 · legions 00426240 — ac2-ui hud.rs + ac2-gpu ui.rs + ac2-client hud_view.rs: HUD roots made and drawn; no framework behaviour beyond hiding the toolbar's dismount/consignment buttons
- [ ] `gmRadarUI::RecvNotice_ReloadOptions` · 00426710

## `ac2/client/src/ui/gameplay/gmsecuretradeui.cpp.c` (0/27)

### gmSecureTradeUI
- [ ] `gmSecureTradeUI::Show` · 00420a90 · legions 004208c0
- [ ] `gmSecureTradeUI::CloseTrade` · 00420b10 · legions 00420940
- [ ] `gmSecureTradeUI::FinishListBoxDrop` · 00420b50 · legions 00420980
- [ ] `gmSecureTradeUI::UpdateButton` · 00420ba0 · legions 004209d0
- [ ] `gmSecureTradeUI::HandleItemRemoval` · 00420c20 · legions 00420a50

### (free functions)
- [ ] `gmSecureTradeUI_DDCB` · 00420c70

### gmSecureTradeUI
- [ ] `gmSecureTradeUI::ListenToMessage` · 00420ca0 · legions 00420ad0
- [ ] `gmSecureTradeUI::RecvNotice_StackUIDone` · 00420cd0
- [ ] `gmSecureTradeUI::RecvNotice_Trade_BeAcceptanceCleared` · 00420d10 · legions 00420b40
- [ ] `gmSecureTradeUI::RecvNotice_Trade_BeAccepted` · 00420d50 · legions 00420b80
- [ ] `gmSecureTradeUI::RecvNotice_Trade_BeDeclined` · 00420dd0 · legions 00420c00
- [ ] `gmSecureTradeUI::RecvNotice_Trade_BeOpened` · 00420e50 · legions 00420c80
- [ ] `gmSecureTradeUI::DoAcceptTrade` · 00420f00 · legions 00420d30
- [ ] `gmSecureTradeUI::HandleListBoxDrop` · 00420f60 · legions 00420d90
- [ ] `gmSecureTradeUI::ListenToMessage` · 00421100 · legions 00420f30
- [ ] `gmSecureTradeUI::RecvNotice_CloseConfirmationDialog` · 00421230
- [ ] `gmSecureTradeUI::MakeAcceptTradeDialog` · 00421250 · legions 00421080
- [ ] `gmSecureTradeUI::RecvNotice_Trade_BeRegistered` · 004212d0
- [ ] `gmSecureTradeUI::AddItem` · 00421380 · legions 004211b0
- [ ] `gmSecureTradeUI::RecvNotice_Trade_BeOffered` · 00421550
- [ ] `gmSecureTradeUI::RemoveItem` · 00421590 · legions 004213c0
- [ ] `gmSecureTradeUI::RecvNotice_Trade_BeRevoked` · 00421690
- [ ] `gmSecureTradeUI::ResetInternal` · 004216b0 · legions 00421520
- [ ] `gmSecureTradeUI::gmSecureTradeUI` · 004217a0 · legions 00421610
- [ ] `gmSecureTradeUI::~gmSecureTradeUI` · 004219d0 · legions 00421840
- [ ] `gmSecureTradeUI::RecvNotice_Trade_BeClosed` · 00421a90
- [ ] `gmSecureTradeUI::RecvNotice_Trade_BeReset` · 00421bf0

## `ac2/client/src/ui/gameplay/gmselectionui.cpp.c` (7/24)

### LandDefs
- [ ] `LandDefs::in_bounds` · 004233a0 · legions 00423170
- [x] `LandDefs::blockid_to_lcoord` · 004233d0 · legions 004231a0
- [ ] `LandDefs::in_same_block` · 00423450 · legions 00423220
- [x] `LandDefs::get_block_offset` · 00423490 · legions 00423260

### Frame
- [x] `Frame::globaltolocalvec` · 00423550 · legions 00423320
- [ ] `Frame::localtoglobal` · 004235b0 · legions 00423380
- [x] `Frame::globaltolocal` · 00423610 · legions 004233e0 — ac2-physics bsp/tree.rs, position.rs: physics-BSP collision in mesh space (asm float order; Legions identical)

### Position
- [x] `Position::localtolocal` · 00423690 · legions 00423460 — ac2-physics bsp/tree.rs, position.rs: physics-BSP collision in mesh space (asm float order; Legions identical)

### gmSelectionUI
- [ ] `gmSelectionUI::RegisterObjectMessages` · 00423750 · legions 00423520
- [ ] `gmSelectionUI::RecvNotice_CreateObject` · 004237a0
- [ ] `gmSelectionUI::RecvNotice_E_LastAttacker` · 004237d0 · legions 004235a0
- [ ] `gmSelectionUI::RecvNotice_SetTargetingPointer` · 00423820
- [ ] `gmSelectionUI::Farther` · 004238f0 · legions 004236c0
- [ ] `gmSelectionUI::TargetTypeVsKey` · 00423970 · legions 00423740
- [x] `gmSelectionUI::Start3DSelection` · 00423a50 · legions 00423820 — ac2-render selection.rs + ac2-ui selection_ui.rs: clicking the world selects
- [x] `gmSelectionUI::ListenToMessage` · 00423b20 · legions 004238f0 — ac2-render selection.rs + ac2-ui selection_ui.rs: clicking the world selects
- [~] `gmSelectionUI::Finish3DSelection` · 00423bb0 · legions 00423980 — ac2-render selection.rs: drawn-mesh triangles (skinned parts at their group matrix), no potential list, no selectability / LookAtSelection / own-item checks
- [ ] `gmSelectionUI::Init` · 00423d20 · legions 00423af0
- [ ] `gmSelectionUI::Select` · 00423da0
- [ ] `gmSelectionUI::RecvNotice_Select` · 00424500
- [~] `gmSelectionUI::gmSelectionUI` · 00424510 · legions 004242e0 — ac2-ui hud.rs + ac2-gpu ui.rs + ac2-client hud_view.rs: HUD roots made and drawn; no framework behaviour beyond hiding the toolbar's dismount/consignment buttons
- [ ] `gmSelectionUI::ListenToMessage` · 00424610 · legions 004243e0
- [ ] `gmSelectionUI::RecvNotice_UpdateSelectability` · 00424670
- [ ] `gmSelectionUI::RecvNotice_BeingDeleted` · 00424720

## `ac2/client/src/ui/gameplay/gmshopcatalogui.cpp.c` (0/11)

### gmShopCatalogUI
- [ ] `gmShopCatalogUI::DestroyManagedElements` · 0041cc80 · legions 0041caa0
- [ ] `gmShopCatalogUI::Show` · 0041ccb0 · legions 0041cad0
- [ ] `gmShopCatalogUI::Select` · 0041cd30 · legions 0041cb50
- [ ] `gmShopCatalogUI::ListenToMessage` · 0041cda0 · legions 0041cbc0
- [ ] `gmShopCatalogUI::EnterStore` · 0041cde0 · legions 0041cc00
- [ ] `gmShopCatalogUI::gmShopCatalogUI` · 0041ce30 · legions 0041cc50
- [ ] `gmShopCatalogUI::~gmShopCatalogUI` · 0041cf10 · legions 0041cd30
- [ ] `gmShopCatalogUI::RecvNotice_LeaveCatalog` · 0041cf80
- [ ] `gmShopCatalogUI::ListenToMessage` · 0041cff0 · legions 0041ce10
- [ ] `gmShopCatalogUI::PostShop` · 0041d0f0 · legions 0041cf10
- [ ] `gmShopCatalogUI::RecvNotice_ViewCatalog` · 0041d230

## `ac2/client/src/ui/gameplay/gmshopconsignmentui.cpp.c` (0/28)

### UIShopSale
- [ ] `UIShopSale::~UIShopSale` · 0041b090 · legions 0041aef0

### gmShopConsignmentUI
- [ ] `gmShopConsignmentUI::UpdatePortraitPosition` · 0041b0b0 · legions 004424c0
- [ ] `gmShopConsignmentUI::RecvNotice_UpdateMountState` · 0041b240

### gmShopUI
- [ ] `gmShopUI::RecvNotice_LeaveStore` · 0041b250

### gmShopConsignmentUI
- [ ] `gmShopConsignmentUI::SyncPortrait` · 0041b270 · legions 0041b0d0

### (free functions)
- [ ] `gmShopConsignmentUI_DDCB` · 0041b320

### gmShopConsignmentUI
- [ ] `gmShopConsignmentUI::RecvNotice_StackUIDone` · 0041b390
- [ ] `gmShopConsignmentUI::RecvNotice_CloseConfirmationDialog` · 0041b550 · legions 0041b3b0
- [ ] `gmShopConsignmentUI::CanStackWith` · 0041b5d0 · legions 0041b430
- [ ] `gmShopConsignmentUI::InitPortrait` · 0041b6a0 · legions 0041b500
- [ ] `gmShopConsignmentUI::UpdatePortrait` · 0041b760 · legions 0041b5c0
- [ ] `gmShopConsignmentUI::RecvNotice_LocalVisualDescUpdate` · 0041b7c0
- [ ] `gmShopConsignmentUI::UpdatePosition` · 0041b7e0 · legions 0041b640
- [ ] `gmShopConsignmentUI::CreateManagedElements` · 0041b8b0 · legions 0041b710
- [ ] `gmShopConsignmentUI::Show` · 0041b950 · legions 0041b7b0
- [ ] `gmShopConsignmentUI::Select` · 0041ba10 · legions 0041b870
- [ ] `gmShopConsignmentUI::DestroyManagedElements` · 0041bc00 · legions 0041ba20
- [ ] `gmShopConsignmentUI::RecvNotice_HandleDragFrom` · 0041bc90
- [ ] `gmShopConsignmentUI::RecvNotice_SelectionChanged` · 0041bd20
- [ ] `gmShopConsignmentUI::ListenToMessage` · 0041bd90 · legions 0041bbb0
- [ ] `gmShopConsignmentUI::HandleListBoxDrop` · 0041bde0 · legions 0041bc00
- [ ] `gmShopConsignmentUI::~gmShopConsignmentUI` · 0041bff0 · legions 0041be10
- [ ] `gmShopConsignmentUI::PostSale` · 0041c0f0 · legions 0041bf10
- [ ] `gmShopConsignmentUI::ListenToMessage` · 0041c480 · legions 0041c2a0
- [ ] `gmShopConsignmentUI::UpdateSales` · 0041c820 · legions 0041c640
- [ ] `gmShopConsignmentUI::gmShopConsignmentUI` · 0041c980 · legions 0041c7a0
- [ ] `gmShopConsignmentUI::RecvNotice_UpdateConsignment` · 0041cb60
- [ ] `gmShopConsignmentUI::RecvNotice_ViewConsignment` · 0041cbd0 · legions 0041c9f0

## `ac2/client/src/ui/gameplay/gmshopnewsaleui.cpp.c` (0/14)

### gmShopNewSaleUI
- [ ] `gmShopNewSaleUI::DestroyManagedElements` · 0041a600 · legions 0041a440
- [ ] `gmShopNewSaleUI::Update` · 0041a640 · legions 0041a4a0
- [ ] `gmShopNewSaleUI::ListenToMessage` · 0041a760 · legions 0041a5c0
- [ ] `gmShopNewSaleUI::RecvNotice_CloseConfirmationDialog` · 0041a7b0 · legions 0041a610
- [ ] `gmShopNewSaleUI::HandleDragDrop` · 0041a830 · legions 0041a690

### (free functions)
- [ ] `gmShopNewSaleUI_DDCB` · 0041a950

### gmShopNewSaleUI
- [ ] `gmShopNewSaleUI::~gmShopNewSaleUI` · 0041a9c0 · legions 0041a820
- [ ] `gmShopNewSaleUI::CreateManagedElements` · 0041aa70 · legions 0041a8d0
- [ ] `gmShopNewSaleUI::RecvNotice_StackUIDone` · 0041ab00 · legions 0041a960
- [ ] `gmShopNewSaleUI::RecvNotice_InitNewSale` · 0041ab40
- [ ] `gmShopNewSaleUI::Show` · 0041ab60 · legions 0041a9c0
- [ ] `gmShopNewSaleUI::gmShopNewSaleUI` · 0041ac00 · legions 0041aa60
- [ ] `gmShopNewSaleUI::AddSale` · 0041ad40 · legions 0041aba0
- [ ] `gmShopNewSaleUI::ListenToMessage` · 0041af80 · legions 0041ade0

## `ac2/client/src/ui/gameplay/gmshopui.cpp.c` (0/22)

### gmShopUI
- [ ] `gmShopUI::CreateManagedElements` · 0041d3d0 · legions 0041d1f0
- [ ] `gmShopUI::UpdatePosition` · 0041d4a0 · legions 0041d2c0
- [ ] `gmShopUI::Show` · 0041d590 · legions 0041d3b0
- [ ] `gmShopUI::RecvNotice_ConfirmPurchase` · 0041d630
- [ ] `gmShopUI::RecvNotice_CloseConfirmationDialog` · 0041d700 · legions 0041d520
- [ ] `gmShopUI::Select` · 0041d740 · legions 0041d560
- [ ] `gmShopUI::FinishDragFromShop` · 0041d800 · legions 0041d620
- [ ] `gmShopUI::RecvNotice_StackUIDone` · 0041d850 · legions 0041d670
- [ ] `gmShopUI::RecvNotice_UpdatePosition` · 0041d880
- [ ] `gmShopUI::RecvNotice_SelectionChanged` · 0041d8b0
- [ ] `gmShopUI::DestroyManagedElements` · 0041d960 · legions 0041d780
- [ ] `gmShopUI::RecvNotice_UpdateSale` · 0041d9b0
- [ ] `gmShopUI::ListenToMessage` · 0041dac0 · legions 0041d8e0
- [ ] `gmShopUI::PurchaseItem` · 0041dc40 · legions 0041da60
- [ ] `gmShopUI::~gmShopUI` · 0041de50 · legions 0041dc70
- [ ] `gmShopUI::RecvNotice_HandleDragFrom` · 0041df30
- [ ] `gmShopUI::PostSale` · 0041df90 · legions 0041ddb0
- [ ] `gmShopUI::ListenToMessage` · 0041e2c0 · legions 0041e0e0
- [ ] `gmShopUI::UpdateSales` · 0041e540 · legions 0041e360
- [ ] `gmShopUI::gmShopUI` · 0041e6a0 · legions 0041e4c0
- [ ] `gmShopUI::RecvNotice_UpdateStore` · 0041e8d0
- [ ] `gmShopUI::RecvNotice_ViewStore` · 0041e940

## `ac2/client/src/ui/gameplay/gmstackui.cpp.c` (0/13)

### gmStackUI
- [ ] `gmStackUI::Show` · 004247f0 · legions 004245e0
- [ ] `gmStackUI::PositionWindow` · 00424810 · legions 00424600
- [ ] `gmStackUI::Close` · 004248f0 · legions 004246e0
- [ ] `gmStackUI::gmStackUI` · 004249b0 · legions 004247a0
- [ ] `gmStackUI::~gmStackUI` · 00424b00 · legions 004248f0
- [ ] `gmStackUI::RecvNotice_CloseStackUI` · 00424b50
- [ ] `gmStackUI::RecvNotice_ContainerStateUpdate` · 00424b70
- [ ] `gmStackUI::RecvNotice_BeingDeleted` · 00424ba0

### PSUtils
- [ ] `PSUtils::is_uint32` · 00424be0 · legions 004249d0

### gmStackUI
- [ ] `gmStackUI::HandleOkButton` · 00424c70 · legions 00424a60
- [ ] `gmStackUI::SetAmount` · 00424da0 · legions 00424b90
- [ ] `gmStackUI::ListenToMessage` · 00424e20 · legions 00424c10
- [ ] `gmStackUI::RecvNotice_InitStackUI` · 00424ed0 · legions 00424cc0

## `ac2/client/src/ui/gameplay/gmstoryui.cpp.c` (0/11)

### gmKeyframe
- [ ] `gmKeyframe::GetSubtitle` · 0042bf80 · legions 0042bd40

### gmStoryUI
- [ ] `gmStoryUI::Finish` · 0042c030 · legions 0042bdf0
- [ ] `gmStoryUI::~gmStoryUI` · 0042c1e0 · legions 0042bfa0
- [ ] `gmStoryUI::Play` · 0042c2d0 · legions 0042c090
- [ ] `gmStoryUI::CheckStart` · 0042c5a0 · legions 0042c360

### QualifiedDataIDArray
- [ ] `QualifiedDataIDArray::~QualifiedDataIDArray` · 0042c660 · legions 0042c420

### gmStoryUI
- [ ] `gmStoryUI::ListenToMessage` · 0042c6b0 · legions 0042c470
- [ ] `gmStoryUI::ListenToMessage` · 0042c6f0 · legions 0042c4b0
- [ ] `gmStoryUI::OnContextFinished` · 0042c780

### QualifiedDataIDArray
- [ ] `QualifiedDataIDArray::QualifiedDataIDArray` · 0042c7a0 · legions 0042c560

### gmStoryUI
- [ ] `gmStoryUI::gmStoryUI` · 0042c7f0 · legions 0042c5b0

## `ac2/client/src/ui/gameplay/helper/chatwindow.cpp.c` (14/32)

### ChatWindow
- [x] `ChatWindow::IsActiveRoot` · 0044afd0 · legions 0044bc30 — ac2-ui chat.rs / chat_window.rs
- [x] `ChatWindow::IsActive` · 0044b000 · legions 0044bc60 — ac2-ui chat.rs / chat_window.rs
- [x] `ChatWindow::HandleActivation` · 0044b030 · legions 0044bcd0 — ac2-ui chat.rs / chat_window.rs
- [~] `ChatWindow::Show` · 0044b090 · legions 0044bd30 — ac2-ui chat: no opacity, scroll lock, save loc, spew box, selection
- [ ] `ChatWindow::LocationApplied` · 0044b120 · legions 0044bdc0
- [x] `ChatWindow::SetZLevel` · 0044b140 · legions 0044bde0 — ac2-ui chat.rs / chat_window.rs

### gmChatUI
- [ ] `gmChatUI::RecvNotice_CreateObject` · 0044b150

### ChatWindow::ChatInfo
- [x] `ChatWindow::ChatInfo::ChatInfo` · 0044b1a0 · legions 0044bdf0 — ac2-ui chat.rs / chat_window.rs

### ChatWindow
- [ ] `ChatWindow::SetOpacity` · 0044b1b0 · legions 0044be20
- [~] `ChatWindow::SetActiveWindow` · 0044b1f0 · legions 0044be60 — ac2-ui chat: no opacity, scroll lock, save loc, spew box, selection
- [~] `ChatWindow::ListenToMessage` · 0044b220 · legions 0044c570 — ac2-ui chat: no opacity, scroll lock, save loc, spew box, selection

### PStringBaseIter_Common<unsigned_short>
- [ ] `PStringBaseIter_Common<unsigned_short>::FindChar` · 0044b5b0 · legions 0044c090

### ChatWindow
- [~] `ChatWindow::ProcessCommand` · 0044b650 · legions 0044da90 — ac2-ui chat: no opacity, scroll lock, save loc, spew box, selection

### PStringBaseIter_Common<unsigned_short>
- [ ] `PStringBaseIter_Common<unsigned_short>::FindChar` · 0044b730 · legions 0044c130

### ChatWindow
- [x] `ChatWindow::HandleEnterKey` · 0044b810 · legions 0044db60 — ac2-ui chat.rs / chat_window.rs

### SmartArray<ChatWindow::ChatInfo>
- [ ] `SmartArray<ChatWindow::ChatInfo>::AddToEnd` · 0044b880 · legions 0044c210

### ChatWindow
- [x] `ChatWindow::RecvNotice_DisplayStringInfo` · 0044b8e0 — ac2-ui chat.rs / chat_window.rs
- [x] `ChatWindow::GetChatMessageInfo` · 0044ba70 · legions 0044c790 — ac2-ui chat.rs / chat_window.rs
- [~] `ChatWindow::DoAssistedTell` · 0044bad0 · legions 0044c7f0 — ac2-ui chat: no opacity, scroll lock, save loc, spew box, selection

### PSUtils
- [ ] `PSUtils::FindAllWords` · 0044bc80 · legions 0044c990

### ChatWindow
- [ ] `ChatWindow::~ChatWindow` · 0044beb0 · legions 0044cbc0
- [x] `ChatWindow::HandleCommandOrAliasKey` · 0044bf80 · legions 0044cc90 — ac2-ui chat.rs / chat_window.rs
- [x] `ChatWindow::AllTextToLog` · 0044c020 · legions 0044cd30 — ac2-ui chat + ac2-client chat_text
- [x] `ChatWindow::DisplayCmdHistory` · 0044c110 · legions 0044ce30 — ac2-ui chat + ac2-client chat_text
- [x] `ChatWindow::DisplayRetellHistory` · 0044c1f0 · legions 0044cf10 — ac2-ui chat + ac2-client chat_text
- [x] `ChatWindow::DisplayReplyHistory` · 0044c2a0 · legions 0044cfc0 — ac2-ui chat + ac2-client chat_text
- [x] `ChatWindow::HandleReplyKey` · 0044c400 · legions 0044d120 — ac2-ui chat.rs / chat_window.rs
- [~] `ChatWindow::ListenToMessage` · 0044c5b0 · legions 0044d2d0 — ac2-ui chat: no opacity, scroll lock, save loc, spew box, selection
- [~] `ChatWindow::KeyPress` · 0044c5f0 · legions 0044d310 — ac2-ui chat: no opacity, scroll lock, save loc, spew box, selection
- [~] `ChatWindow::ChatWindow` · 0044c7d0 · legions 0044d700 — ac2-ui chat: no opacity, scroll lock, save loc, spew box, selection
- [ ] `ChatWindow::RecvNotice_ReloadOptions` · 0044cb10
- [~] `ChatWindow::RecvNotice_ReloadChatSettings` · 0044cbc0 — ac2-ui chat: no opacity, scroll lock, save loc, spew box, selection

## `ac2/client/src/ui/gameplay/helper/containerdisplay.cpp.c` (1/22)

### ContainerDisplay
- [ ] `ContainerDisplay::FinishListBoxDrop` · 0042d910 · legions 0042d6e0

### (free functions)
- [ ] `ContainerDisplay_DDCB` · 0042da30

### ContainerDisplay
- [ ] `ContainerDisplay::Show` · 0042da90 · legions 0042d860
- [ ] `ContainerDisplay::RequestContext` · 0042dab0 · legions 0042d880

### UIElement_ListBox
- [x] `UIElement_ListBox::WhatNum` · 0042dac0 · legions 0042d890 — ac2-ui list_box.rs

### ContainerDisplay
- [ ] `ContainerDisplay::RecvNotice_StackUIDone` · 0042daf0 · legions 0042d8c0
- [ ] `ContainerDisplay::GetNewIndex` · 0042db40 · legions 0042d910
- [ ] `ContainerDisplay::HandleDragFrom` · 0042dbc0 · legions 0042d990
- [ ] `ContainerDisplay::HandleListBoxDrop` · 0042dc20 · legions 0042d9f0
- [ ] `ContainerDisplay::HandleDragFromMoneyBag` · 0042de90 · legions 0042dc60
- [ ] `ContainerDisplay::ListenToMessage` · 0042dfb0 · legions 0042de40
- [ ] `ContainerDisplay::AddItem` · 0042e070 · legions 0042df00
- [ ] `ContainerDisplay::FindItem` · 0042e150 · legions 0042dfe0
- [ ] `ContainerDisplay::RemoveItem` · 0042e1e0 · legions 0042e070
- [ ] `ContainerDisplay::BeginRequest` · 0042e2a0 · legions 0042e130
- [ ] `ContainerDisplay::RequestFailed` · 0042e2d0 · legions 0042e160
- [ ] `ContainerDisplay::Flush` · 0042e300 · legions 0042e190
- [ ] `ContainerDisplay::ContainerDisplay` · 0042e3a0 · legions 0042e230
- [ ] `ContainerDisplay::Cleanup` · 0042e470 · legions 0042e300
- [ ] `ContainerDisplay::GetContents` · 0042e560 · legions 0042e3f0
- [ ] `ContainerDisplay::Init` · 0042e650 · legions 0042e4e0
- [ ] `ContainerDisplay::RecvNotice_UpdateContainer` · 0042e7b0

## `ac2/client/src/ui/gameplay/helper/fellowinfo.cpp.c` (0/15)

### FellowInfo
- [ ] `FellowInfo::~FellowInfo` · 00439ea0 · legions 0043a6f0
- [ ] `FellowInfo::UpdateCellID` · 00439f30 · legions 0043a780
- [ ] `FellowInfo::UpdateLevel` · 00439f40 · legions 0043a790
- [ ] `FellowInfo::UpdatePercentage` · 00439fa0 · legions 0043a7f0
- [ ] `FellowInfo::ToggleTextState` · 0043a030 · legions 0043a880
- [ ] `FellowInfo::SetLeader` · 0043a060 · legions 0043a8b0
- [ ] `FellowInfo::ShowSelectionHighlight` · 0043a090 · legions 0043a8e0
- [ ] `FellowInfo::RecvNotice_UpdateFellowHealth` · 0043a0d0 · legions 0043a920
- [ ] `FellowInfo::RecvNotice_UpdateFellowMaxHealth` · 0043a110
- [ ] `FellowInfo::RecvNotice_UpdateFellowVigor` · 0043a150 · legions 0043a9a0
- [ ] `FellowInfo::RecvNotice_UpdateFellowMaxVigor` · 0043a190
- [ ] `FellowInfo::RecvNotice_UpdateFellowLevel` · 0043a1d0
- [ ] `FellowInfo::RecvNotice_BeingDeleted` · 0043a1f0 · legions 0043aa40
- [ ] `FellowInfo::FellowInfo` · 0043a200 · legions 0043aa50
- [ ] `FellowInfo::UpdateInfo` · 0043a4d0 · legions 0043ad20

## `ac2/client/src/ui/gameplay/helper/fellowshiphelper.cpp.c` (0/24)

### FellowshipHelper
- [ ] `FellowshipHelper::SelectFellowByIdx` · 0042f0f0 · legions 0042eff0

### ItemDisplay
- [ ] `ItemDisplay::RecvNotice_BeingDeleted` · 0042f180

### FellowshipHelper
- [ ] `FellowshipHelper::ListenToMessage` · 0042f1a0 · legions 0042f080

### HashIterator<InstanceID,float>
- [ ] `HashIterator<InstanceID,float>::operator++` · 0042f2c0 · legions 0042f1a0

### FellowshipHelper
- [ ] `FellowshipHelper::ToggleTextState` · 0042f300 · legions 0042f220

### PHashTable<InstanceID,float>
- [ ] `PHashTable<InstanceID,float>::~PHashTable<InstanceID,float>` · 0042f4a0 · legions 0042f360

### FellowshipHelper
- [ ] `FellowshipHelper::ListenToMessage` · 0042f5f0 · legions 0042f4b0
- [ ] `FellowshipHelper::SetLeader` · 0042f6e0 · legions 0042f5a0
- [ ] `FellowshipHelper::RecvNotice_UpdateLeader` · 0042f7a0
- [ ] `FellowshipHelper::RecvNotice_SelectionChanged` · 0042f7c0

### PHashTable<InstanceID,float>
- [ ] `PHashTable<InstanceID,float>::StreamPack` · 0042f8e0

### (free functions)
- [ ] `STREAMPACK` · 0042fb10 · legions 0042f9d0

### FellowshipHelper
- [ ] `FellowshipHelper::UpdateXP` · 0042fca0 · legions 0042fa60
- [ ] `FellowshipHelper::RecvNotice_UpdateFellowLevel` · 0042fdc0
- [ ] `FellowshipHelper::RemoveFellow` · 0042fdd0 · legions 0042fb90
- [ ] `FellowshipHelper::RemoveAllFellows` · 0042feb0 · legions 0042fc70
- [ ] `FellowshipHelper::RecvNotice_UpdateFellowCellID` · 0042ff90
- [ ] `FellowshipHelper::RecvNotice_ClearFellow` · 00430060
- [ ] `FellowshipHelper::RecvNotice_ReloadGameplayOptions` · 00430080
- [ ] `FellowshipHelper::~FellowshipHelper` · 00430180 · legions 0042ff40
- [ ] `FellowshipHelper::AddFellow` · 00430210 · legions 0042ffd0
- [ ] `FellowshipHelper::AddAllFellows` · 00430390 · legions 00430150
- [ ] `FellowshipHelper::RecvNotice_UpdateFellow` · 00430430
- [ ] `FellowshipHelper::FellowshipHelper` · 004304a0 · legions 00430260

## `ac2/client/src/ui/gameplay/helper/itemdisplay.cpp.c` (0/15)

### ItemDisplay
- [ ] `ItemDisplay::ItemDisplay` · 0042e900 · legions 0042e7e0
- [ ] `ItemDisplay::~ItemDisplay` · 0042e9d0 · legions 0042e8b0
- [ ] `ItemDisplay::ListenToMessage` · 0042eb30 · legions 0042ea10
- [ ] `ItemDisplay::RecvNotice_SelectionChanged` · 0042ebb0 · legions 0042ea90
- [ ] `ItemDisplay::RecvNotice_SetTargetingPointer` · 0042ec10
- [ ] `ItemDisplay::Update` · 0042ece0 · legions 0042ebe0
- [ ] `ItemDisplay::SetItem` · 0042eed0 · legions 0042edd0
- [ ] `ItemDisplay::SetPending` · 0042ef20 · legions 0042ee20
- [ ] `ItemDisplay::SetDraggable` · 0042ef30 · legions 0042ee30
- [ ] `ItemDisplay::SetQuantity` · 0042ef40 · legions 0042ee40
- [ ] `ItemDisplay::SetShortcutType` · 0042ef60 · legions 0042ee60
- [ ] `ItemDisplay::RecvNotice_ItemUpdate` · 0042ef90
- [ ] `ItemDisplay::RecvNotice_LocalVisualDescUpdate` · 0042efc0
- [ ] `ItemDisplay::RecvNotice_CreateObject` · 0042eff0
- [ ] `ItemDisplay::Init` · 0042f020 · legions 0042ef20

## `ac2/client/src/ui/gameplay/helper/optionbase.cpp.c` (0/7)

### OptionBase
- [ ] `OptionBase::OptionBase` · 0044f7b0 · legions 004508f0
- [ ] `OptionBase::~OptionBase` · 0044f7e0 · legions 00450920
- [ ] `OptionBase::Create` · 0044f810 · legions 00450950
- [ ] `OptionBase::Retrieve` · 0044f850 · legions 00450990
- [ ] `OptionBase::Describe` · 0044f870 · legions 004509b0
- [ ] `OptionBase::Describe` · 0044f890 · legions 004509d0
- [ ] `OptionBase::Identify` · 0044f940 · legions 00450a80

## `ac2/client/src/ui/gameplay/helper/optionchoice.cpp.c` (0/15)

### OptionChoice
- [ ] `OptionChoice::OptionChoice` · 0044df20 · legions 0044f0f0
- [ ] `OptionChoice::Cast` · 0044dfa0 · legions 0044f170
- [ ] `OptionChoice::AddChoice` · 0044dfe0 · legions 0044f1b0
- [ ] `OptionChoice::AssignChoice` · 0044e060 · legions 0044f230
- [ ] `OptionChoice::SetCurrent` · 0044e090 · legions 0044f260
- [ ] `OptionChoice::SetDefault` · 0044e140 · legions 0044f310
- [ ] `OptionChoice::GetCurrent` · 0044e180 · legions 0044f350
- [ ] `OptionChoice::Changed` · 0044e1c0 · legions 0044f390
- [ ] `OptionChoice::Apply` · 0044e210 · legions 0044f3e0
- [ ] `OptionChoice::Recall` · 0044e260 · legions 0044f430
- [ ] `OptionChoice::Default` · 0044e280 · legions 0044f450
- [ ] `OptionChoice::ListenToMessage` · 0044e2a0 · legions 0044f470
- [ ] `OptionChoice::Refresh` · 0044e3a0 · legions 0044f570
- [ ] `OptionChoice::Refresh` · 0044e3c0 · legions 0044f590
- [ ] `OptionChoice::Merge` · 0044e590 · legions 0044f6d0

## `ac2/client/src/ui/gameplay/helper/optionexpander.cpp.c` (0/16)

### OptionExpander
- [ ] `OptionExpander::CountOptions` · 0044eb80 · legions 0044fcc0
- [ ] `OptionExpander::Refit` · 0044eb90 · legions 0044fcd0
- [ ] `OptionExpander::Refresh` · 0044ec30 · legions 0044fd70
- [ ] `OptionExpander::Refresh` · 0044ec80 · legions 0044fdc0
- [ ] `OptionExpander::GetOption` · 0044ed10 · legions 0044fe50
- [ ] `OptionExpander::Construct` · 0044ed30 · legions 0044fe70
- [ ] `OptionExpander::Dismantle` · 0044edb0 · legions 0044fef0
- [ ] `OptionExpander::Changed` · 0044ee10 · legions 0044ff50
- [ ] `OptionExpander::Apply` · 0044ee50 · legions 0044ff90
- [ ] `OptionExpander::Recall` · 0044ee80 · legions 0044ffc0
- [ ] `OptionExpander::Default` · 0044eeb0 · legions 0044fff0
- [ ] `OptionExpander::ListenToMessage` · 0044eee0 · legions 00450020
- [ ] `OptionExpander::OptionExpander` · 0044ef30 · legions 00450070
- [ ] `OptionExpander::~OptionExpander` · 0044efc0 · legions 00450100
- [ ] `OptionExpander::Clear` · 0044f030 · legions 00450170
- [ ] `OptionExpander::AddOption` · 0044f0b0 · legions 004501f0

## `ac2/client/src/ui/gameplay/helper/optionflag.cpp.c` (0/13)

### OptionFlag
- [ ] `OptionFlag::OptionFlag` · 0044e6d0 · legions 0044f810
- [ ] `OptionFlag::Cast` · 0044e760 · legions 0044f8a0
- [ ] `OptionFlag::Merge` · 0044e7a0 · legions 0044f8e0
- [ ] `OptionFlag::SetCurrent` · 0044e800 · legions 0044f940
- [ ] `OptionFlag::SetDefault` · 0044e850 · legions 0044f990
- [ ] `OptionFlag::GetCurrent` · 0044e8b0 · legions 0044f9f0
- [ ] `OptionFlag::Changed` · 0044e8e0 · legions 0044fa20
- [ ] `OptionFlag::Apply` · 0044e920 · legions 0044fa60
- [ ] `OptionFlag::Recall` · 0044e960 · legions 0044faa0
- [ ] `OptionFlag::Default` · 0044e9b0 · legions 0044faf0
- [ ] `OptionFlag::ListenToMessage` · 0044ea00 · legions 0044fb40
- [ ] `OptionFlag::Refresh` · 0044eae0 · legions 0044fc20
- [ ] `OptionFlag::Refresh` · 0044eb30 · legions 0044fc70

## `ac2/client/src/ui/gameplay/helper/optionvalue.cpp.c` (0/14)

### OptionValue
- [ ] `OptionValue::OptionValue` · 0044f100 · legions 00450240
- [ ] `OptionValue::Cast` · 0044f1f0 · legions 00450330
- [ ] `OptionValue::Merge` · 0044f230 · legions 00450370
- [ ] `OptionValue::SetRange` · 0044f2a0 · legions 004503e0
- [ ] `OptionValue::SetCurrent` · 0044f2f0 · legions 00450430
- [ ] `OptionValue::SetDefault` · 0044f3d0 · legions 00450510
- [ ] `OptionValue::GetCurrent` · 0044f460 · legions 004505a0
- [ ] `OptionValue::Changed` · 0044f4d0 · legions 00450610
- [ ] `OptionValue::Apply` · 0044f510 · legions 00450650
- [ ] `OptionValue::Recall` · 0044f540 · legions 00450680
- [ ] `OptionValue::Default` · 0044f570 · legions 004506b0
- [ ] `OptionValue::ListenToMessage` · 0044f5a0 · legions 004506e0
- [ ] `OptionValue::Refresh` · 0044f750 · legions 00450890
- [ ] `OptionValue::Refresh` · 0044f780 · legions 004508c0

## `ac2/client/src/ui/gameplay/helper/timedisplay.cpp.c` (0/4)

### TimeDisplay
- [ ] `TimeDisplay::TimeDisplay` · 0043dd20 · legions 0043e570
- [ ] `TimeDisplay::ComputeTimeLeft` · 0043ddb0 · legions 0043e600
- [ ] `TimeDisplay::ListenToMessage` · 0043e080 · legions 0043e8d0
- [ ] `TimeDisplay::Init` · 0043e0d0 · legions 0043e920

## `ac2/client/src/ui/gameplay/helper/vitaldisplay.cpp.c` (7/7)

### VitalDisplay
- [x] `VitalDisplay::VitalDisplay` · 0044dc80 · legions 0044ee50 — ac2-ui meter.rs
- [x] `VitalDisplay::Update` · 0044dcd0 · legions 0044eea0 — ac2-ui meter.rs
- [x] `VitalDisplay::UpdateCurrentVal` · 0044de30 · legions 0044f000 — ac2-ui meter.rs
- [x] `VitalDisplay::UpdateMaxVal` · 0044de50 · legions 0044f020 — ac2-ui meter.rs
- [x] `VitalDisplay::ToggleTextState` · 0044de70 · legions 0044f040 — ac2-ui meter.rs
- [x] `VitalDisplay::SetTextState` · 0044dea0 · legions 0044f070 — ac2-ui meter.rs
- [x] `VitalDisplay::SetMeterState` · 0044ded0 · legions 0044f0a0 — ac2-ui meter.rs

## `ac2/client/src/ui/gameplay/statusbar/gmeffectsui.cpp.c` (0/10)

### gmEffectsUI
- [ ] `gmEffectsUI::SelectElement` · 00438a20 · legions 00439270
- [ ] `gmEffectsUI::ListenToMessage` · 00438b80
- [ ] `gmEffectsUI::RecvNotice_PulseMissed` · 00438c30

### PList<EffectUINode>
- [ ] `PList<EffectUINode>::Pack` · 00438ef0 · legions 00439740
- [ ] `PList<EffectUINode>::GetPackSize` · 00438f40
- [ ] `PList<EffectUINode>::UnPack` · 00438f80

### gmEffectsUI
- [ ] `gmEffectsUI::Init` · 004391c0 · legions 00439a10
- [ ] `gmEffectsUI::RecvNotice_ExaminationRequestChanged` · 004396b0
- [ ] `gmEffectsUI::RecvNotice_UpdateEffect` · 00439700
- [ ] `gmEffectsUI::gmEffectsUI` · 00439710 · legions 00439f60

## `ac2/client/src/ui/gameplay/statusbar/gmstatusbarui.cpp.c` (0/3)

### gmStatusbarUI
- [ ] `gmStatusbarUI::Show` · 004231b0 · legions 00422f80
- [ ] `gmStatusbarUI::SetupChildren` · 00423210 · legions 00422fe0
- [~] `gmStatusbarUI::gmStatusbarUI` · 004232f0 · legions 004230c0 — ac2-ui hud.rs + ac2-gpu ui.rs + ac2-client hud_view.rs: HUD roots made and drawn; no framework behaviour beyond hiding the toolbar's dismount/consignment buttons

## `ac2/client/src/ui/gameplay/statusbar/gmvitalsui.cpp.c` (0/6)

### gmVitalsUI
- [ ] `gmVitalsUI::~gmVitalsUI` · 004397d0 · legions 0043a020
- [ ] `gmVitalsUI::RecvNotice_UpdateVital` · 004398c0
- [ ] `gmVitalsUI::SetTooltip` · 00439930 · legions 0043a180
- [ ] `gmVitalsUI::gmVitalsUI` · 004399a0 · legions 0043a1f0
- [ ] `gmVitalsUI::ListenToMessage` · 00439d30 · legions 0043a580
- [ ] `gmVitalsUI::RecvNotice_UpdateHeroStatus` · 00439e00 · legions 0043a650

## `ac2/client/src/ui/gameplay/statusbar/indicators/gmindicatorsui.cpp.c` (0/4)

### gmIndicatorsUI
- [ ] `gmIndicatorsUI::Show` · 00438850 · legions 004390a0
- [ ] `gmIndicatorsUI::SetupChildren` · 00438860 · legions 004390b0
- [ ] `gmIndicatorsUI::CleanupChildren` · 00438910 · legions 00439160
- [ ] `gmIndicatorsUI::gmIndicatorsUI` · 004389a0 · legions 004391f0

## `ac2/client/src/ui/gameplay/statusbar/indicators/gmnetstatusui.cpp.c` (0/11)

### GraphTool
- [ ] `GraphTool::GraphTool` · 0044d1a0 · legions 0044e370
- [ ] `GraphTool::BlendColors` · 0044d220 · legions 0044e3f0

### gmNetStatusUI
- [ ] `gmNetStatusUI::RecvNotice_ReloadGameplayOptions` · 0044d2a0 · legions 0044e470
- [ ] `gmNetStatusUI::RecvNotice_ReloadOptions` · 0044d2c0 · legions 0044e490

### GraphTool
- [ ] `GraphTool::DrawBar` · 0044d2e0 · legions 0044e4b0

### gmNetStatusUI
- [ ] `gmNetStatusUI::DrawBandwidthGraph` · 0044d3d0 · legions 0044e5a0

### SmartArray<gmNetStatusUI::GraphHistoryColumn>
- [ ] `SmartArray<gmNetStatusUI::GraphHistoryColumn>::AddNElements` · 0044d720 · legions 0044e8f0

### gmNetStatusUI
- [ ] `gmNetStatusUI::~gmNetStatusUI` · 0044d750 · legions 0044e920
- [ ] `gmNetStatusUI::OnHeartbeat` · 0044d800 · legions 0044e9d0
- [ ] `gmNetStatusUI::ListenToMessage` · 0044da90 · legions 0044ec60
- [ ] `gmNetStatusUI::gmNetStatusUI` · 0044dab0 · legions 0044ec80

## `ac2/client/src/ui/gameplay/statusbar/indicators/gmpkstatusui.cpp.c` (0/2)

### gmPKStatusUI
- [ ] `gmPKStatusUI::gmPKStatusUI` · 0044d040 · legions 0044e210
- [ ] `gmPKStatusUI::RecvNotice_UpdateFactionStatus` · 0044d0c0

## `ac2/client/src/ui/gameplay/statusbar/indicators/gmvitaestatusui.cpp.c` (0/4)

### gmVitaeStatusUI
- [ ] `gmVitaeStatusUI::gmVitaeStatusUI` · 0044cd80 · legions 0044df50
- [ ] `gmVitaeStatusUI::RecvNotice_ExaminationRequestChanged` · 0044ce40 · legions 0044e010
- [ ] `gmVitaeStatusUI::RecvNotice_UpdateVitae` · 0044ce90
- [ ] `gmVitaeStatusUI::ListenToMessage` · 0044cfc0

## `ac2/client/src/ui/gameplay/toolbar/gmchatui.cpp.c` (6/14)

### gmChatUI
- [x] `gmChatUI::SetLastFocus` · 004351c0 · legions 00435990 — ac2-ui chat.rs / chat_window.rs
- [ ] `gmChatUI::~gmChatUI` · 004351d0 · legions 004359f0
- [x] `gmChatUI::DetermineTargetWindow` · 00435280 · legions 00435aa0 — ac2-ui chat.rs / chat_window.rs
- [x] `gmChatUI::UpdateZLevels` · 00435320 · legions 00435b40 — ac2-ui chat.rs / chat_window.rs
- [x] `gmChatUI::SetActiveWindow` · 00435370 · legions 00435b90 — ac2-ui chat.rs / chat_window.rs
- [x] `gmChatUI::RecvNotice_DoAssistedTell` · 004353c0 — ac2-ui chat.rs / chat_window.rs
- [x] `gmChatUI::DispatchKeypressToWindow` · 00435550 · legions 00435d70 — ac2-ui chat.rs / chat_window.rs

### SmartArray<StringInfo>
- [ ] `SmartArray<StringInfo>::AddToEnd` · 004355d0 · legions 00435e70

### gmChatUI
- [~] `gmChatUI::Init` · 00435620 · legions 00435ec0 — ac2-ui chat: no opacity, scroll lock, save loc, spew box, selection
- [~] `gmChatUI::RecvNotice_DisplayStringInfo` · 004357d0 — ac2-ui chat: no opacity, scroll lock, save loc, spew box, selection
- [~] `gmChatUI::gmChatUI` · 004357f0 · legions 00436080 — ac2-ui chat: no opacity, scroll lock, save loc, spew box, selection
- [ ] `gmChatUI::Update` · 004358d0 · legions 00436160
- [ ] `gmChatUI::Show` · 00435b80 · legions 00436410
- [~] `gmChatUI::ListenToMessage` · 00435bc0 · legions 00436450 — ac2-ui chat: no opacity, scroll lock, save loc, spew box, selection

## `ac2/client/src/ui/gameplay/toolbar/gmexaminationui.cpp.c` (6/14)

### gmExaminationUI
- [x] `gmExaminationUI::RecvNotice_OpenExaminationPanel` · 00430810 — ac2-ui toolbar.rs

### gmMiniFellowUI
- [ ] `gmMiniFellowUI::RecvNotice_BeingDeleted` · 00430820

### gmExaminationUI
- [x] `gmExaminationUI::MakeString` · 00430840 · legions 00430690 — ac2-ui examination.rs + ac2-client ui_sources
- [x] `gmExaminationUI::DoBreak` · 004309a0 · legions 004307f0 — ac2-ui examination.rs + ac2-client ui_sources
- [ ] `gmExaminationUI::CleanupArray` · 00430a00 · legions 00430850
- [~] `gmExaminationUI::ProcessEffectIDNode` · 00430a50 · legions 004308e0 — no TimeDisplay, icon layers, effect expansion, Legions filter tabs
- [ ] `gmExaminationUI::~gmExaminationUI` · 00430ad0 · legions 004317b0
- [x] `gmExaminationUI::Update` · 00430b60 · legions 004312c0 — ac2-ui examination.rs + ac2-client ui_sources
- [~] `gmExaminationUI::Show` · 00430c00 · legions 00431880 — no TimeDisplay, icon layers, effect expansion, Legions filter tabs
- [~] `gmExaminationUI::ListenToMessage` · 00430c80 — no TimeDisplay, icon layers, effect expansion, Legions filter tabs
- [x] `gmExaminationUI::RecvNotice_ExaminationRequestChanged` · 00430d80 — ac2-ui examination.rs + ac2-client ui_sources
- [~] `gmExaminationUI::ProcessNode` · 00430db0 · legions 00430ae0 — no TimeDisplay, icon layers, effect expansion, Legions filter tabs
- [~] `gmExaminationUI::gmExaminationUI` · 004312e0 — no TimeDisplay, icon layers, effect expansion, Legions filter tabs
- [x] `gmExaminationUI::RecvNotice_UpdateExaminationProfile` · 00431410 — ac2-ui examination.rs + ac2-client ui_sources

## `ac2/client/src/ui/gameplay/toolbar/gmnavigationui.cpp.c` (3/37)

### LandDefs
- [x] `LandDefs::inbound_valid_cellid` · 004314c0 · legions 00431dc0 — ac2-script glue::position (ported beside the gPosition glue)

### gmNavigationUI
- [ ] `gmNavigationUI::AutoShowScrollButtons` · 00431550 · legions 00431e50
- [ ] `gmNavigationUI::UpdateMacroMap` · 004315f0 · legions 00431ef0
- [ ] `gmNavigationUI::UpdateLockPositionButtonToolTip` · 004316d0 · legions 00431fd0
- [ ] `gmNavigationUI::ConvertLandscapeToPixel` · 00431760 · legions 00432060
- [ ] `gmNavigationUI::ConvertPixelToLandscape` · 00431860 · legions 00432160
- [ ] `gmNavigationUI::CenterMarker` · 00431950 · legions 00432250
- [ ] `gmNavigationUI::SizeMap` · 004319b0 · legions 004322b0
- [ ] `gmNavigationUI::ReadMarkerInfo` · 00431bd0 · legions 004324d0

### LandDefs
- [x] `LandDefs::lcoord_to_gid` · 00431e20 · legions 004326b0
- [x] `LandDefs::gid_to_lcoord` · 00431e90 · legions 00432720

### Position
- [ ] `Position::Position` · 00431f60 · legions 004327f0

### gmNavigationUI
- [ ] `gmNavigationUI::Update` · 00431fa0 · legions 00432830
- [ ] `gmNavigationUI::PlaceMarker` · 00432090 · legions 00432920
- [ ] `gmNavigationUI::Show` · 00432330 · legions 00432bc0

### SmartArray<CMapNoteRow>
- [ ] `SmartArray<CMapNoteRow>::operator=` · 00432460 · legions 00432cf0

### PSmartArray<CMapNoteRow>
- [ ] `PSmartArray<CMapNoteRow>::StreamPack` · 00432510 · legions 00432da0

### gmNavigationUI
- [ ] `gmNavigationUI::PlaceMarkers` · 004325b0 · legions 00432e40
- [ ] `gmNavigationUI::FlushMarkers` · 00432640 · legions 00432ed0
- [ ] `gmNavigationUI::FlushAllMarkers` · 004326a0 · legions 00432f30
- [ ] `gmNavigationUI::~gmNavigationUI` · 004327f0 · legions 00433080
- [ ] `gmNavigationUI::SetupElementsForMapMode` · 004328b0 · legions 00433140
- [ ] `gmNavigationUI::InitMacroMap` · 004329c0 · legions 00433250
- [ ] `gmNavigationUI::RecvNotice_ClearMarkers` · 00432b30
- [ ] `gmNavigationUI::AddMarker` · 00432b90 · legions 00433420
- [ ] `gmNavigationUI::FlushMarkerByIID` · 00432e40 · legions 004336d0
- [ ] `gmNavigationUI::RecvNotice_AddMarker` · 00432ef0
- [ ] `gmNavigationUI::RecvNotice_RemoveMarker` · 004330e0
- [ ] `gmNavigationUI::AddMapNotes` · 00433100 · legions 00433990
- [ ] `gmNavigationUI::InitDetailMapFromCellCoords` · 004335f0 · legions 00433e50
- [ ] `gmNavigationUI::ZoomToCharPos` · 00433810 · legions 00434070
- [ ] `gmNavigationUI::OnPlayerBlockChanged` · 00433830 · legions 00434090
- [ ] `gmNavigationUI::gmNavigationUI` · 00433880 · legions 004340e0
- [ ] `gmNavigationUI::ListenToMessage` · 00433cf0 · legions 00434570
- [ ] `gmNavigationUI::InitDetailMapFromLastClick` · 00434160 · legions 00434930
- [ ] `gmNavigationUI::UpdatePlayerPos` · 004341b0 · legions 00434980
- [ ] `gmNavigationUI::ListenToMessage` · 00434280 · legions 00434a50

## `ac2/client/src/ui/gameplay/toolbar/gmshortcutui.cpp.c` (11/46)

### gmShortcutUI
- [ ] `gmShortcutUI::Show` · 00435cd0 · legions 00436520
- [x] `gmShortcutUI::CheckAttackButtons` · 00435ce0 · legions 00436530 — ac2-ui shortcut.rs
- [x] `gmShortcutUI::RecvNotice_UpdateAttackState` · 00435d30 — ac2-ui shortcut.rs
- [x] `gmShortcutUI::RequestAttackStateChange` · 00435d80 · legions 004365d0 — ac2-client Command::SetAttacking: ShouldAutoTarget's Select(0x41000024) notice, then the IFun on the VM (stand-in: OnlineSession::request_attack_state -> WM_Combat 0x16 with the selection / 0x14)

### (free functions)
- [ ] `gmShortcutUI_DDCB` · 00435e10 · legions 00436660
- [ ] `gmShortcutUI_DOCB` · 00435e70

### SmartArray<TimeLeftInfo>
- [ ] `SmartArray<TimeLeftInfo>::RemoveOrderedByIndex` · 00435ec0 · legions 00436710

### gmShortcutUI
- [x] `gmShortcutUI::RemoveShortcut` · 00435fc0 · legions 00436810 — ac2-ui shortcut.rs
- [ ] `gmShortcutUI::RecvNotice_RemoveRecipe` · 00436060
- [x] `gmShortcutUI::CheckRecipe` · 004360e0 · legions 00436930 — ac2-ui shortcut.rs
- [ ] `gmShortcutUI::CheckTimeLeftArray` · 00436120 · legions 00436970
- [ ] `gmShortcutUI::UpdateTimeLeft` · 00436180 · legions 004369d0
- [ ] `gmShortcutUI::RecvNotice_ToggleSpecialAttack` · 00436330

### ShortcutInfo
- [x] `ShortcutInfo::SetDataStr` · 004363e0 · legions 00436c30 — ac2-script glue::shortcut / native::shortcut

### gmShortcutUI
- [~] `gmShortcutUI::CheckItem` · 00436440 · legions 00436c90 — ac2-ui shortcut.rs: no drag/drop, time-left meters, pending look
- [~] `gmShortcutUI::FromElement` · 004364f0 · legions 00436d40 — ac2-ui shortcut.rs: no drag/drop, time-left meters, pending look

### SmartArray<TimeLeftInfo>
- [ ] `SmartArray<TimeLeftInfo>::AddToEnd` · 00436660 · legions 00436eb0

### gmShortcutUI
- [~] `gmShortcutUI::CheckSkill` · 004367a0 · legions 00436ff0 — ac2-ui shortcut.rs: no drag/drop, time-left meters, pending look
- [~] `gmShortcutUI::UpdateSetNumber` · 00436940 · legions 00437190 — ac2-ui shortcut.rs: no drag/drop, time-left meters, pending look

### PSmartArray<ShortcutInfo>
- [ ] `PSmartArray<ShortcutInfo>::StreamPack` · 00436a00 · legions 00437250

### gmShortcutUI
- [x] `gmShortcutUI::CheckShortcut` · 00436bc0 · legions 00437410 — ac2-ui shortcut.rs
- [x] `gmShortcutUI::CheckAllItems` · 00436c20 · legions 00437470 — ac2-ui shortcut.rs
- [~] `gmShortcutUI::CheckItem` · 00436ca0 · legions 004374f0 — ac2-ui shortcut.rs: no drag/drop, time-left meters, pending look
- [x] `gmShortcutUI::CheckAllSkills` · 00436da0 · legions 004375f0 — ac2-ui shortcut.rs
- [~] `gmShortcutUI::CheckSkill` · 00436e20 · legions 00437670 — ac2-ui shortcut.rs: no drag/drop, time-left meters, pending look
- [ ] `gmShortcutUI::RecvNotice_UpdateInventory` · 00436ea0
- [ ] `gmShortcutUI::RecvNotice_UpdateContainer` · 00436f00
- [ ] `gmShortcutUI::RecvNotice_UpdateSkill` · 00436f40
- [ ] `gmShortcutUI::RecvNotice_UpdateResetTime` · 00436f60
- [ ] `gmShortcutUI::RecvNotice_BeingDeleted` · 00436f70
- [ ] `gmShortcutUI::FIXMESetupItemElement` · 00436fe0 · legions 00437830
- [ ] `gmShortcutUI::~gmShortcutUI` · 004371c0 · legions 00437a10
- [ ] `gmShortcutUI::RecvNotice_CreateObject` · 00437290
- [~] `gmShortcutUI::MakeElement` · 00437320 · legions 00437b70 — ac2-ui shortcut.rs: no drag/drop, time-left meters, pending look
- [~] `gmShortcutUI::SetShortcut` · 00437620 · legions 00437e70 — ac2-ui shortcut.rs: no drag/drop, time-left meters, pending look
- [ ] `gmShortcutUI::RecvNotice_RequestSpecialAttack` · 004376c0
- [ ] `gmShortcutUI::RecvNotice_StartSpecialAttack` · 00437770
- [ ] `gmShortcutUI::RecvNotice_EndSpecialAttack` · 00437820
- [~] `gmShortcutUI::ListenToMessage` · 004379a0 · legions 004381f0 — ac2-ui shortcut.rs: no drag/drop, time-left meters, pending look
- [x] `gmShortcutUI::UpdatePadding` · 00437d90 · legions 004385e0 — ac2-ui shortcut.rs
- [~] `gmShortcutUI::RefreshShortcutHeight` · 00437e50 · legions 004386a0 — ac2-ui shortcut.rs: no drag/drop, time-left meters, pending look
- [~] `gmShortcutUI::gmShortcutUI` · 00437fb0 · legions 00438800 — ac2-ui shortcut.rs: no drag/drop, time-left meters, pending look
- [~] `gmShortcutUI::ListenToMessage` · 00438270 · legions 00438ac0 — ac2-ui shortcut.rs: no drag/drop, time-left meters, pending look
- [~] `gmShortcutUI::Init` · 004386c0 · legions 00438f10 — ac2-ui shortcut.rs: no drag/drop, time-left meters, pending look
- [x] `gmShortcutUI::RecvNotice_ReloadShortcuts` · 004387f0 — ac2-ui shortcut.rs
- [ ] `gmShortcutUI::RecvNotice_ReloadOptions` · 00438830 · legions 00439080

## `ac2/client/src/ui/gameplay/toolbar/gmtoolbarui.cpp.c` (1/16)

### PlayingSoundContext
- [ ] `PlayingSoundContext::PlayingSoundContext` · 00421ec0 · legions 006b07e0

### gmToolbarUI
- [ ] `gmToolbarUI::ShowKeyStone` · 00421ed0 · legions 00421d10
- [ ] `gmToolbarUI::ShowPlugin` · 00421f60 · legions 00421da0
- [ ] `gmToolbarUI::RecvNotice_ShowSaleReminder` · 00421fe0
- [ ] `gmToolbarUI::RecvNotice_UpdateMountState` · 00422080 · legions 00421ec0
- [ ] `gmToolbarUI::Show` · 004220d0 · legions 00421f10
- [~] `gmToolbarUI::ManageChildren` · 00422160 · legions 00421fa0 — ac2-ui toolbar.rs: fields shown/hidden by side; contents, help, tooltips not yet
- [x] `gmToolbarUI::ChildKeyPress` · 00422230 · legions 00422070 — ac2-ui toolbar.rs
- [~] `gmToolbarUI::ListenToMessage` · 00422270 · legions 004220b0 — ac2-ui toolbar.rs: fields shown/hidden by side; contents, help, tooltips not yet
- [~] `gmToolbarUI::ListenToMessage` · 00422300 · legions 00422140 — ac2-ui toolbar.rs: fields shown/hidden by side; contents, help, tooltips not yet

### List<unsigned_long>
- [ ] `List<unsigned_long>::operator=` · 00422360 · legions 0040d980

### ListNode<ToolbarChildInfo>
- [ ] `ListNode<ToolbarChildInfo>::ListNode<ToolbarChildInfo>` · 00422400 · legions 004221d0

### gmToolbarUI
- [~] `gmToolbarUI::SetupChildren` · 00422550 · legions 00422320 — ac2-ui toolbar.rs: fields shown/hidden by side; contents, help, tooltips not yet
- [ ] `gmToolbarUI::CleanupChildren` · 00422f00 · legions 00422cd0
- [~] `gmToolbarUI::gmToolbarUI` · 00422f80 · legions 00422d50 — ac2-ui hud.rs + ac2-gpu ui.rs + ac2-client hud_view.rs: HUD roots made and drawn; no framework behaviour beyond hiding the toolbar's dismount/consignment buttons
- [ ] `gmToolbarUI::~gmToolbarUI` · 00423120 · legions 00422ef0

## `ac2/client/src/ui/gameplay/toolbar/inventory/gm3ditemsui.cpp.c` (0/14)

### gm3DItemsUI
- [ ] `gm3DItemsUI::FinishItemDrop` · 00440240 · legions 00440aa0
- [ ] `gm3DItemsUI::StartFind3DTarget` · 004402c0 · legions 00440b20
- [ ] `gm3DItemsUI::Start3DDrag` · 00440320 · legions 00440b80

### (free functions)
- [ ] `gm3DItems_DDCB` · 00440490

### gm3DItemsUI
- [ ] `gm3DItemsUI::gm3DItemsUI` · 00440520 · legions 00440d80
- [ ] `gm3DItemsUI::~gm3DItemsUI` · 00440650 · legions 00440eb0
- [ ] `gm3DItemsUI::RecvNotice_StackUIDone` · 004406b0 · legions 00440f10
- [ ] `gm3DItemsUI::StartItemDrop` · 00440700 · legions 00440f60
- [ ] `gm3DItemsUI::StartMoneyDrop` · 004407f0 · legions 00441050
- [ ] `gm3DItemsUI::FinishFind3DTarget` · 00440810 · legions 00441070
- [ ] `gm3DItemsUI::ListenToMessage` · 004408e0 · legions 00441140
- [ ] `gm3DItemsUI::MidItemDrop` · 00440960 · legions 004411c0
- [ ] `gm3DItemsUI::MidMoneyDrop` · 00440a60 · legions 004412c0
- [ ] `gm3DItemsUI::ListenToMessage` · 00440b40 · legions 004413a0

## `ac2/client/src/ui/gameplay/toolbar/inventory/gmbackpackui.cpp.c` (0/7)

### gmBackpackUI
- [ ] `gmBackpackUI::FinishItemDrop` · 004412f0 · legions 00441b50
- [ ] `gmBackpackUI::FinishMoneyBagDrop` · 00441390 · legions 00441bf0
- [ ] `gmBackpackUI::RecvNotice_StackUIDone` · 00441410
- [ ] `gmBackpackUI::ShowPack` · 00441480 · legions 00441d70
- [ ] `gmBackpackUI::ListenToMessage` · 004414e0 · legions 00441de0
- [ ] `gmBackpackUI::~gmBackpackUI` · 00441860 · legions 00442160
- [ ] `gmBackpackUI::gmBackpackUI` · 00441950 · legions 00442250

## `ac2/client/src/ui/gameplay/toolbar/inventory/gminventoryui.cpp.c` (0/5)

### gmInventoryUI
- [ ] `gmInventoryUI::Show` · 00434500 · legions 00434cd0
- [ ] `gmInventoryUI::ListenToMessage` · 00434530 · legions 00434d00
- [ ] `gmInventoryUI::SetupChildren` · 00434590 · legions 00434d60

### gmStatusbarUI
- [ ] `gmStatusbarUI::CleanupChildren` · 00434670 · legions 00434e40

### gmInventoryUI
- [ ] `gmInventoryUI::gmInventoryUI` · 00434700 · legions 00434ed0

## `ac2/client/src/ui/gameplay/toolbar/inventory/gmmoneybagui.cpp.c` (0/11)

### gmMoneyBagUI
- [ ] `gmMoneyBagUI::RecvNotice_UpdateMoney` · 00440b70
- [ ] `gmMoneyBagUI::CloseDialog` · 00440be0 · legions 00441440
- [ ] `gmMoneyBagUI::IsConfirmationDialogAppropriate` · 00440c00 · legions 00441460
- [ ] `gmMoneyBagUI::MakeConfirmationDialog` · 00440c30 · legions 00441490
- [ ] `gmMoneyBagUI::DoDragTo` · 00440cd0 · legions 00441530

### (free functions)
- [ ] `gmMoneyBagUI_DDCB` · 00440d50

### gmMoneyBagUI
- [ ] `gmMoneyBagUI::gmMoneyBagUI` · 00440df0 · legions 00441650
- [ ] `gmMoneyBagUI::RecvNotice_CloseConfirmationDialog` · 00440f70
- [ ] `gmMoneyBagUI::RecvNotice_StackUIDone` · 00440f90
- [ ] `gmMoneyBagUI::HandleMoneyBagDrop` · 00440fe0 · legions 00441840
- [ ] `gmMoneyBagUI::ListenToMessage` · 00441280 · legions 00441ae0

## `ac2/client/src/ui/gameplay/toolbar/inventory/gmpaperdollui.cpp.c` (0/31)

### gmMiniFellowUI
- [ ] `gmMiniFellowUI::Show` · 00441b40 · legions 00420260

### gmPaperDollUI
- [ ] `gmPaperDollUI::RecvNotice_OpenIngredientPanel` · 00441b70 · legions 00442490
- [ ] `gmPaperDollUI::UpdatePaperDollPosition` · 00441ba0 · legions 0041af10
- [ ] `gmPaperDollUI::MeshTypeToInvLoc` · 00441d20 · legions 00442670
- [ ] `gmPaperDollUI::HandlePaperDollClick` · 00441db0 · legions 00442700

### (free functions)
- [ ] `gmPaperDollUI_DDCB` · 00441e50

### gmPaperDollUI
- [ ] `gmPaperDollUI::RecvNotice_UpdateMountState` · 00441e90
- [ ] `gmPaperDollUI::SetDurabilityTooltip` · 00441ea0 · legions 004427f0
- [ ] `gmPaperDollUI::InitPaperDoll` · 00441f10 · legions 00442860
- [ ] `gmPaperDollUI::UpdatePaperDoll` · 00441fd0 · legions 00442920
- [ ] `gmPaperDollUI::UpdateDurabilityMeter` · 00442030 · legions 00442980
- [ ] `gmPaperDollUI::RecvNotice_LocalVisualDescUpdate` · 004420e0
- [ ] `gmPaperDollUI::TokenToInvLoc` · 00442100 · legions 00442a50
- [ ] `gmPaperDollUI::HandlePaperDollDrop` · 00442180 · legions 00442ad0
- [ ] `gmPaperDollUI::InqItemInvLoc` · 00442300 · legions 00442c50
- [ ] `gmPaperDollUI::FinishPaperDollClick` · 00442380 · legions 00442cd0
- [ ] `gmPaperDollUI::HandlePaperDollDrag` · 004424b0 · legions 00442e00
- [ ] `gmPaperDollUI::~gmPaperDollUI` · 00442580 · legions 00442ed0
- [ ] `gmPaperDollUI::ListenToMessage` · 00442870 · legions 004431c0
- [ ] `gmPaperDollUI::ListenToMessage` · 00442890 · legions 004431e0
- [ ] `gmPaperDollUI::RecvNotice_CreateObject` · 00442900
- [ ] `gmPaperDollUI::RecvNotice_ItemUpdate` · 004429e0
- [ ] `gmPaperDollUI::AddItem` · 00442af0 · legions 00443440
- [ ] `gmPaperDollUI::RemoveItem` · 00442c70 · legions 004435c0
- [ ] `gmPaperDollUI::BeginRequest` · 00442dc0 · legions 00443710
- [ ] `gmPaperDollUI::RequestFailed` · 00442e10 · legions 00443760
- [ ] `gmPaperDollUI::InitSlots` · 00442e60 · legions 004437b0

### PHashTable<unsigned_long,InstanceID>
- [~] `PHashTable<unsigned_long,InstanceID>::StreamPack` · 004431d0 — unpack only, in ac2-net protocol::qualities / visual (Legions CBaseQualities::UnPack instruction-identical)

### gmPaperDollUI
- [ ] `gmPaperDollUI::GetInventory` · 00443400 · legions 00443d50
- [ ] `gmPaperDollUI::gmPaperDollUI` · 004434d0 · legions 00443e20
- [ ] `gmPaperDollUI::RecvNotice_UpdateInventory` · 00443670

## `ac2/client/src/ui/gameplay/toolbar/option/gmassistanceui.cpp.c` (0/6)

### gmAssistanceUI
- [ ] `gmAssistanceUI::Reset` · 00443720 · legions 00444070
- [ ] `gmAssistanceUI::RecvNotice_CloseMessageDialog` · 004437c0
- [ ] `gmAssistanceUI::InitCallTypeMenu` · 004437e0 · legions 00444130
- [ ] `gmAssistanceUI::gmAssistanceUI` · 00443990 · legions 004442e0
- [ ] `gmAssistanceUI::HandleSendButton` · 00443b00 · legions 00444450
- [ ] `gmAssistanceUI::ListenToMessage` · 00443cc0 · legions 00444610

## `ac2/client/src/ui/gameplay/toolbar/option/gmcharacterui.cpp.c` (0/22)

### gmCharacterUI
- [ ] `gmCharacterUI::ConfirmRecall` · 00444e00 · legions 00445940
- [ ] `gmCharacterUI::ConfirmDefault` · 00444e60 · legions 004459a0
- [ ] `gmCharacterUI::ClearConfirmations` · 00444ec0 · legions 00445a00
- [ ] `gmCharacterUI::RefreshFlag` · 00444f20 · legions 00445a60
- [ ] `gmCharacterUI::ApplyFlag` · 004451b0 · legions 00445cf0
- [ ] `gmCharacterUI::ApplyValue` · 00445450 · legions 00445f90
- [ ] `gmCharacterUI::ApplyChoice` · 004454a0 · legions 00445fe0
- [ ] `gmCharacterUI::~gmCharacterUI` · 004454f0 · legions 00446030
- [ ] `gmCharacterUI::Refresh` · 004455d0 · legions 00446110
- [ ] `gmCharacterUI::Changed` · 004456d0 · legions 00446210
- [ ] `gmCharacterUI::RecvNotice_ReloadOptions` · 00445710
- [ ] `gmCharacterUI::RecvNotice_SetShortcutHeight` · 00445720
- [ ] `gmCharacterUI::BuildList` · 004457b0 · legions 004462f0
- [ ] `gmCharacterUI::UpdateButtons` · 00445800 · legions 00446340
- [ ] `gmCharacterUI::Apply` · 004458f0 · legions 00446430
- [ ] `gmCharacterUI::Recall` · 004459e0 · legions 00446520
- [ ] `gmCharacterUI::Default` · 00445a10 · legions 00446550
- [ ] `gmCharacterUI::RecvNotice_CloseConfirmationDialog` · 00445a50
- [ ] `gmCharacterUI::ListenToMessage` · 00445ac0 · legions 00446600
- [ ] `gmCharacterUI::AddOption` · 00445c20 · legions 00446760
- [ ] `gmCharacterUI::Setup` · 00445c80 · legions 004467c0
- [ ] `gmCharacterUI::gmCharacterUI` · 00446610 · legions 00447150

## `ac2/client/src/ui/gameplay/toolbar/option/gmchatoptionsui.cpp.c` (0/18)

### gmChatOptionsUI
- [ ] `gmChatOptionsUI::ConfirmApply` · 00443d20 · legions 00444670
- [ ] `gmChatOptionsUI::ConfirmRecall` · 00443d80 · legions 004446d0
- [ ] `gmChatOptionsUI::ConfirmDefault` · 00443de0 · legions 00444730
- [ ] `gmChatOptionsUI::CreateChatFontSizeChoice` · 00443e40 · legions 00444790
- [ ] `gmChatOptionsUI::CreateChatFontColorChoice` · 00443ee0 · legions 00444830
- [ ] `gmChatOptionsUI::PopulateExpander` · 00443f60
- [ ] `gmChatOptionsUI::~gmChatOptionsUI` · 00444100 · legions 00444b40
- [ ] `gmChatOptionsUI::Changed` · 004442a0 · legions 00444ce0
- [ ] `gmChatOptionsUI::UpdateButtons` · 00444340 · legions 00444d80
- [ ] `gmChatOptionsUI::Recall` · 00444450 · legions 00444e90
- [ ] `gmChatOptionsUI::ListenToMessage` · 004444f0 · legions 00444f30
- [ ] `gmChatOptionsUI::Setup` · 00444640 · legions 00445080
- [ ] `gmChatOptionsUI::Apply` · 00444770 · legions 004451b0
- [ ] `gmChatOptionsUI::Default` · 004449b0 · legions 00445470
- [ ] `gmChatOptionsUI::RecvNotice_CloseConfirmationDialog` · 00444a50
- [ ] `gmChatOptionsUI::Refresh` · 00444af0
- [ ] `gmChatOptionsUI::RecvNotice_ReloadOptions` · 00444cb0
- [ ] `gmChatOptionsUI::gmChatOptionsUI` · 00444cc0 · legions 00445800

## `ac2/client/src/ui/gameplay/toolbar/option/gmclientui.cpp.c` (0/18)

### gmClientUI
- [ ] `gmClientUI::ConfirmRecall` · 00446710 · legions 00447250
- [ ] `gmClientUI::ConfirmDefault` · 00446770 · legions 004472b0
- [ ] `gmClientUI::ClearConfirmations` · 004467d0 · legions 00447310
- [ ] `gmClientUI::VerifyRenderOptions` · 00446830 · legions 00447370
- [ ] `gmClientUI::MonitorChange` · 00446890
- [ ] `gmClientUI::Changed` · 004468d0 · legions 00447410
- [ ] `gmClientUI::LookupExpander` · 00446910 · legions 00447450
- [ ] `gmClientUI::Dismantle` · 004469d0 · legions 00447510
- [ ] `gmClientUI::UpdateButtons` · 00446a70 · legions 004475b0
- [ ] `gmClientUI::~gmClientUI` · 00446b40 · legions 00447680
- [ ] `gmClientUI::Apply` · 00446bf0 · legions 00447730
- [ ] `gmClientUI::Recall` · 00446ce0 · legions 00447820
- [ ] `gmClientUI::Default` · 00446d10 · legions 00447850
- [ ] `gmClientUI::MonitorReload` · 00446d50
- [ ] `gmClientUI::RecvNotice_CloseConfirmationDialog` · 00446e80
- [ ] `gmClientUI::ListenToMessage` · 00446f40 · legions 00447a80
- [ ] `gmClientUI::Construct` · 004470c0 · legions 00447c00
- [ ] `gmClientUI::gmClientUI` · 00447450 · legions 00447f90

## `ac2/client/src/ui/gameplay/toolbar/option/gmoptionui.cpp.c` (0/11)

### gmOptionUI
- [ ] `gmOptionUI::gmOptionUI` · 004347e0 · legions 00434fb0
- [ ] `gmOptionUI::UpdateButtons` · 00434a20 · legions 004351f0
- [ ] `gmOptionUI::MakeConfirmDialog` · 00434aa0 · legions 00435270
- [ ] `gmOptionUI::DoConfirm` · 00434af0 · legions 004352c0
- [ ] `gmOptionUI::MakeQuitDialog` · 00434b40 · legions 00435310
- [ ] `gmOptionUI::MakeLogoutDialog` · 00434ba0 · legions 00435370
- [ ] `gmOptionUI::CloseDialog` · 00434c00 · legions 004353d0
- [ ] `gmOptionUI::~gmOptionUI` · 00434c30 · legions 00435400
- [ ] `gmOptionUI::Show` · 00434d10 · legions 004354e0
- [ ] `gmOptionUI::ListenToMessage` · 00434dd0 · legions 004355a0
- [ ] `gmOptionUI::RecvNotice_CloseConfirmationDialog` · 00434f70

## `ac2/client/src/ui/gameplay/toolbar/quest/gmjournalquestsui.cpp.c` (0/14)

### gmJournalQuestsUI
- [ ] `gmJournalQuestsUI::RemoveQuestElement` · 0043e3a0 · legions 0043ec10
- [ ] `gmJournalQuestsUI::DoQuestTime` · 0043e430 · legions 0043eca0

### gmJournalQuestsUI::QuestDescriptor
- [ ] `gmJournalQuestsUI::QuestDescriptor::QuestDescriptor` · 0043e520 · legions 0043ed90

### UIElement_ListBox
- [ ] `UIElement_ListBox::GetRow` · 0043e580 · legions 0043edf0

### gmJournalQuestsUI
- [ ] `gmJournalQuestsUI::AddQuestElement` · 0043e5c0 · legions 0043ee30
- [ ] `gmJournalQuestsUI::UpdateQuestElement` · 0043e720 · legions 0043ef90
- [ ] `gmJournalQuestsUI::Update` · 0043e9d0 · legions 0043f240
- [ ] `gmJournalQuestsUI::FindQuestDescriptor` · 0043eae0 · legions 0043f350
- [ ] `gmJournalQuestsUI::ListenToMessage` · 0043ebd0 · legions 0043f440
- [ ] `gmJournalQuestsUI::ListenToMessage` · 0043ecb0 · legions 0043f520
- [ ] `gmJournalQuestsUI::Show` · 0043ecd0 · legions 0043f540
- [ ] `gmJournalQuestsUI::gmJournalQuestsUI` · 0043f000 · legions 0043f8b0
- [ ] `gmJournalQuestsUI::~gmJournalQuestsUI` · 0043f130 · legions 0043f9e0
- [ ] `gmJournalQuestsUI::RecvNotice_UpdateQuests` · 0043f1e0

## `ac2/client/src/ui/gameplay/toolbar/quest/gmquestsui.cpp.c` (1/4)

### gmQuestsUI
- [~] `gmQuestsUI::Show` · 004342d0 · legions 00434aa0 — ac2-ui toolbar.rs: fields shown/hidden by side; contents, help, tooltips not yet
- [x] `gmQuestsUI::ListenToMessage` · 00434330 · legions 00434b00 — ac2-ui toolbar.rs
- [ ] `gmQuestsUI::SetActivePanel` · 00434390 · legions 00434b60
- [~] `gmQuestsUI::gmQuestsUI` · 004343c0 · legions 00434b90 — ac2-ui toolbar.rs: fields shown/hidden by side; contents, help, tooltips not yet

## `ac2/client/src/ui/gameplay/toolbar/quest/gmstoryquestsui.cpp.c` (0/11)

### gmSceneInfo
- [ ] `gmSceneInfo::GetName` · 0043f590 · legions 0043fe40

### PList<gmActInfo>
- [ ] `PList<gmActInfo>::PList<gmActInfo>` · 0043f680 · legions 0043ff30
- [ ] `PList<gmActInfo>::Pack` · 0043f6c0 · legions 0062f690
- [ ] `PList<gmActInfo>::UnPack` · 0043f710 · legions 0043ff84

### gmStoryQuestsUI
- [ ] `gmStoryQuestsUI::~gmStoryQuestsUI` · 0043f8b0 · legions 00440110
- [ ] `gmStoryQuestsUI::RecvNotice_UpdateStoryQuests` · 0043f9a0 · legions 00440200
- [ ] `gmStoryQuestsUI::ReloadActs` · 0043fa30 · legions 00440290
- [ ] `gmStoryQuestsUI::Show` · 0043fd60 · legions 004405c0
- [ ] `gmStoryQuestsUI::ListenToMessage` · 0043fea0 · legions 00440700
- [ ] `gmStoryQuestsUI::ListenToMessage` · 004400d0 · legions 00440930
- [ ] `gmStoryQuestsUI::gmStoryQuestsUI` · 00440110 · legions 00440970

## `ac2/client/src/ui/gameplay/toolbar/skill/gmcraftui.cpp.c` (0/22)

### gmCraftUI
- [ ] `gmCraftUI::SelectElement` · 004494b0 · legions 0044a110
- [ ] `gmCraftUI::SelectCraftSkillCategory` · 004494f0 · legions 0044a150
- [ ] `gmCraftUI::SelectRecipe` · 00449550 · legions 0044a1b0
- [ ] `gmCraftUI::GetRecipeIcon` · 004495b0 · legions 0044a210
- [ ] `gmCraftUI::MakeLevelText` · 00449610
- [ ] `gmCraftUI::RefreshCraftSkillDisplay` · 00449700 · legions 0044a360
- [ ] `gmCraftUI::MakeCraftSkill` · 004499e0 · legions 0044a640
- [ ] `gmCraftUI::MakeCraftSkillCategory` · 00449bb0 · legions 0044a810
- [ ] `gmCraftUI::MakeRecipe` · 00449ce0 · legions 0044a940
- [ ] `gmCraftUI::ClearSelectionInfo` · 00449eb0 · legions 0044ab10
- [ ] `gmCraftUI::UpdateCategories` · 0044a040 · legions 0044aca0
- [ ] `gmCraftUI::~gmCraftUI` · 0044a1a0 · legions 0044ae00
- [ ] `gmCraftUI::RecvNotice_ExaminationRequestChanged` · 0044a2d0
- [ ] `gmCraftUI::SelectCraftSkill` · 0044a3b0 · legions 0044b010
- [ ] `gmCraftUI::RefreshCraftSkillPage` · 0044a440 · legions 0044b0a0
- [ ] `gmCraftUI::RefreshCraftSkillCategoryPage` · 0044a550 · legions 0044b1b0
- [ ] `gmCraftUI::RefreshRecipePage` · 0044a6e0 · legions 0044b340
- [ ] `gmCraftUI::RefreshCraftPanel` · 0044a8e0 · legions 0044b540
- [ ] `gmCraftUI::gmCraftUI` · 0044a960 · legions 0044b5c0
- [ ] `gmCraftUI::SetState` · 0044ab30 · legions 0044b790
- [ ] `gmCraftUI::RecvNotice_UpdateCraftPanel` · 0044ab60
- [ ] `gmCraftUI::ListenToMessage` · 0044ab70 · legions 0044b7d0

## `ac2/client/src/ui/gameplay/toolbar/skill/gmskilltreeui.cpp.c` (11/26)

### gmSkillTreeUI
- [~] `gmSkillTreeUI::MakeUntrainCancelDialog` · 00447550 · legions 00448090 — ac2-ui skill_tree.rs: no tooltips, dialogs as requests, no truncation
- [~] `gmSkillTreeUI::MakeTrainDialog` · 004475b0 · legions 004480f0 — ac2-ui skill_tree.rs: no tooltips, dialogs as requests, no truncation
- [x] `gmSkillTreeUI::MakeLevelText` · 00447610 · legions 00448150 — ac2-ui skill_tree.rs
- [x] `gmSkillTreeUI::SelectNode` · 00447700 · legions 00448240 — ac2-ui skill_tree.rs
- [ ] `gmSkillTreeUI::Show` · 00447760 · legions 004482a0
- [ ] `gmSkillTreeUI::RecvNotice_CloseConfirmationDialog` · 004477c0 · legions 00448300
- [~] `gmSkillTreeUI::SetStringAndTruncate` · 00447890 · legions 004483d0 — ac2-ui skill_tree.rs: no tooltips, dialogs as requests, no truncation

### HashIterator<unsigned_long,BaseList<unsigned_long,11>_>
- [ ] `HashIterator<unsigned_long,BaseList<unsigned_long,11>_>::operator++` · 004478f0 · legions 0062aa70

### gmSkillTreeUI
- [~] `gmSkillTreeUI::UpdateSelectedSkillDisplay` · 00447930 · legions 00448450 — ac2-ui skill_tree.rs: no tooltips, dialogs as requests, no truncation
- [~] `gmSkillTreeUI::RefreshSkillNode` · 00447d50 · legions 00448870 — ac2-ui skill_tree.rs: no tooltips, dialogs as requests, no truncation
- [x] `gmSkillTreeUI::FastRefreshSkillNode` · 00448040 · legions 00448bd0 — ac2-ui skill_tree.rs
- [x] `gmSkillTreeUI::HandleElement` · 00448110 · legions 00448ca0 — ac2-ui skill_tree.rs
- [x] `gmSkillTreeUI::RecvNotice_UpdateSkillPanel` · 00448250 — ac2-ui skill_tree.rs
- [x] `gmSkillTreeUI::SelectSkill` · 00448310 · legions 00448eb0 — ac2-ui skill_tree.rs
- [~] `gmSkillTreeUI::MakeUntrainDialog` · 00448430 · legions 00448fd0 — ac2-ui skill_tree.rs: no tooltips, dialogs as requests, no truncation
- [x] `gmSkillTreeUI::ClearSelectedInfo` · 004484c0 · legions 00449060 — ac2-ui skill_tree.rs
- [~] `gmSkillTreeUI::ListenToMessage` · 00448630 · legions 00449230 — ac2-ui skill_tree.rs: no tooltips, dialogs as requests, no truncation
- [x] `gmSkillTreeUI::InitElements` · 00448810 · legions 004496b0 — ac2-ui skill_tree.rs

### SkillUINode
- [ ] `SkillUINode::~SkillUINode` · 00448860 · legions 00449140

### gmSkillTreeUI
- [x] `gmSkillTreeUI::RecvNotice_UpdateSkill` · 004488c0 — ac2-ui skill_tree.rs
- [x] `gmSkillTreeUI::AddNode` · 004489c0 · legions 00449550 — ac2-ui skill_tree.rs
- [~] `gmSkillTreeUI::LoadPanel` · 00448c70 — ac2-ui skill_tree.rs: no tooltips, dialogs as requests, no truncation
- [x] `gmSkillTreeUI::RecvNotice_ExaminationRequestChanged` · 00448e50 — ac2-ui skill_tree.rs

### PHashTable<unsigned_long,SkillUINode>
- [ ] `PHashTable<unsigned_long,SkillUINode>::StreamPack` · 00448eb0 · legions 00449ae0

### gmSkillTreeUI
- [~] `gmSkillTreeUI::gmSkillTreeUI` · 00449110 · legions 00449d40 — ac2-ui skill_tree.rs: no tooltips, dialogs as requests, no truncation
- [ ] `gmSkillTreeUI::~gmSkillTreeUI` · 00449370 · legions 00449fc0

## `ac2/client/src/ui/gameplay/toolbar/skill/gmskillui.cpp.c` (0/3)

### gmSkillUI
- [~] `gmSkillUI::Show` · 00435000 · legions 004357d0 — ac2-ui toolbar.rs: fields shown/hidden by side; contents, help, tooltips not yet
- [~] `gmSkillUI::ListenToMessage` · 00435030 · legions 00435800 — ac2-ui toolbar.rs: fields shown/hidden by side; contents, help, tooltips not yet
- [~] `gmSkillUI::gmSkillUI` · 00435090 · legions 00435860 — ac2-ui toolbar.rs: fields shown/hidden by side; contents, help, tooltips not yet

## `ac2/client/src/ui/gameplay/toolbar/social/gmallegianceui.cpp.c` (0/20)

### AllegianceDataAdaptor
- [ ] `AllegianceDataAdaptor::~AllegianceDataAdaptor` · 0043c7a0 · legions 0043cff0

### gmAllegianceUI
- [ ] `gmAllegianceUI::MakeCurrent` · 0043c7c0 · legions 0043d010
- [ ] `gmAllegianceUI::MakeVassal` · 0043ca20 · legions 0043d270
- [ ] `gmAllegianceUI::SendRequest` · 0043cbc0 · legions 0043d410
- [ ] `gmAllegianceUI::UpdateButtons` · 0043cce0 · legions 0043d530
- [ ] `gmAllegianceUI::FocusTree` · 0043ce50 · legions 0043d6a0

### HashSet<InstanceID>
- [ ] `HashSet<InstanceID>::operator=` · 0043ce80 · legions 0043d6d0

### gmAllegianceUI
- [ ] `gmAllegianceUI::RecvNotice_SelectionChanged` · 0043cfe0
- [ ] `gmAllegianceUI::UpdateShowState` · 0043d000 · legions 0043d850
- [ ] `gmAllegianceUI::MakeAcceptSwearDialog` · 0043d080 · legions 0043d8d0
- [ ] `gmAllegianceUI::MakeBreakDialog` · 0043d100 · legions 0043d950
- [ ] `gmAllegianceUI::DoBreak` · 0043d180 · legions 0043d9d0
- [ ] `gmAllegianceUI::~gmAllegianceUI` · 0043d290 · legions 0043dae0
- [ ] `gmAllegianceUI::ListenToMessage` · 0043d350 · legions 0043dba0
- [ ] `gmAllegianceUI::RecvNotice_CloseConfirmationDialog` · 0043d440 · legions 0043dc90
- [ ] `gmAllegianceUI::RecvNotice_RequestVassal` · 0043d4a0
- [ ] `gmAllegianceUI::RecvNotice_Patron_SwearDone` · 0043d4b0
- [ ] `gmAllegianceUI::gmAllegianceUI` · 0043d4e0 · legions 0043dd30
- [ ] `gmAllegianceUI::Update` · 0043d790 · legions 0043dfe0
- [ ] `gmAllegianceUI::RecvNotice_UpdateAllegianceProfile` · 0043dcb0

## `ac2/client/src/ui/gameplay/toolbar/social/gmfellowshipui.cpp.c` (0/17)

### gmFellowshipUI
- [ ] `gmFellowshipUI::InitLootingMenu` · 0043bde0 · legions 0043c630
- [ ] `gmFellowshipUI::HandleRecruitButton` · 0043bea0 · legions 0043c6f0
- [ ] `gmFellowshipUI::HandleDismissButton` · 0043bef0 · legions 0043c740
- [ ] `gmFellowshipUI::SetLeaderState` · 0043bf40 · legions 0043c790
- [ ] `gmFellowshipUI::MakeRecruitDialog` · 0043bfd0 · legions 0043c820
- [ ] `gmFellowshipUI::RecvNotice_CloseConfirmationDialog` · 0043c050 · legions 0043c8a0
- [ ] `gmFellowshipUI::RecvNotice_RequestRecruitment` · 0043c0a0
- [ ] `gmFellowshipUI::RecvNotice_ExpireRecruitment` · 0043c0c0 · legions 0043c910
- [ ] `gmFellowshipUI::RecvNotice_UpdateLeader` · 0043c0e0
- [ ] `gmFellowshipUI::~gmFellowshipUI` · 0043c130 · legions 0043c980
- [ ] `gmFellowshipUI::HandleCreateButton` · 0043c1d0 · legions 0043ca20
- [ ] `gmFellowshipUI::SetFellowshipState` · 0043c280 · legions 0043cad0
- [ ] `gmFellowshipUI::RecvNotice_UpdateFellowship` · 0043c3c0
- [ ] `gmFellowshipUI::ListenToMessage` · 0043c4e0 · legions 0043cd30
- [ ] `gmFellowshipUI::SetNoFellowshipState` · 0043c570 · legions 0043cdc0
- [ ] `gmFellowshipUI::RecvNotice_ClearFellowship` · 0043c5c0 · legions 0043ce10
- [ ] `gmFellowshipUI::gmFellowshipUI` · 0043c630 · legions 0043ce80

## `ac2/client/src/ui/gameplay/toolbar/social/gmfriendsui.cpp.c` (0/20)

### gmFriendsUI
- [ ] `gmFriendsUI::ListenToMessage` · 0043a570 · legions 0043adc0
- [ ] `gmFriendsUI::RecvNotice_GUNResult_AddFriend` · 0043a5c0
- [ ] `gmFriendsUI::RecvNotice_LocalStatus` · 0043a5e0 · legions 0043ae30

### PStringBase<unsigned_short>
- [ ] `PStringBase<unsigned_short>::operator==` · 0043a620 · legions 0043ae70
- [ ] `PStringBase<unsigned_short>::operator!=` · 0043a650 · legions 0043aea0

### TextPredicate
- [ ] `TextPredicate::TextPredicate` · 0043a680 · legions 0043aed0
- [ ] `TextPredicate::TextPredicate` · 0043a6f0 · legions 0043af40

### gmFriendsUI
- [ ] `gmFriendsUI::HandleAddButton` · 0043a710 · legions 0043af60
- [ ] `gmFriendsUI::HandleRemoveButton` · 0043a7a0 · legions 0043aff0
- [ ] `gmFriendsUI::HandleStatusChange` · 0043a820 · legions 0043b070
- [ ] `gmFriendsUI::UpdateButtonStates` · 0043abe0 · legions 0043b430
- [ ] `gmFriendsUI::ListenToMessage` · 0043ae30 · legions 0043b680
- [ ] `gmFriendsUI::RecvNotice_SelectionChanged` · 0043ae90

### PList<TurbineUserStatus>
- [ ] `PList<TurbineUserStatus>::UnPack` · 0043b020

### gmFriendsUI
- [ ] `gmFriendsUI::BuildUIList` · 0043b190 · legions 0043b9e0
- [ ] `gmFriendsUI::RecvNotice_GUNResult_RemoveFriend` · 0043b4a0
- [ ] `gmFriendsUI::gmFriendsUI` · 0043b6f0 · legions 0043bf40
- [ ] `gmFriendsUI::~gmFriendsUI` · 0043b8d0 · legions 0043c120
- [ ] `gmFriendsUI::RecvNotice_GUNResult_GetFriendsList` · 0043ba00
- [ ] `gmFriendsUI::RecvNotice_GUNResult_FriendStatus` · 0043bb90

## `ac2/client/src/ui/gameplay/toolbar/social/gmsocialui.cpp.c` (0/3)

### gmSocialUI
- [ ] `gmSocialUI::Show` · 00430620 · legions 004303e0
- [ ] `gmSocialUI::ListenToMessage` · 00430650 · legions 00430410
- [ ] `gmSocialUI::gmSocialUI` · 004306b0 · legions 00430470

## `ac2/client/src/ui/gmuiflow.cpp.c` (9/37)

### gmUIFlow
- [x] `gmUIFlow::~gmUIFlow` · 00405d80 · legions 00405a60 — ac2-ui pregame/flow.rs, pregame/ui.rs

### UIFlow
- [ ] `UIFlow::GetSmartBox` · 00405df0 · legions 00405ae0
- [x] `UIFlow::GetUIMainFramework` · 00405e00 — ac2-ui pregame/flow.rs, pregame/ui.rs

### gmUIFlow
- [-] `gmUIFlow::ListenToMessage` · 00405e10 · legions 00405af0 — debug keys (bench mode, debug HUD, screenshot, sound debug)

### PLineOffsetList
- [ ] `PLineOffsetList::~PLineOffsetList` · 004060b0 · legions 00405dc0

### POriginalSourceFileInfo
- [ ] `POriginalSourceFileInfo::~POriginalSourceFileInfo` · 00406100 · legions 00405e10

### FrameDebugInfo
- [ ] `FrameDebugInfo::~FrameDebugInfo` · 00406150 · legions 00405e60

### ImportData
- [ ] `ImportData::~ImportData` · 004061b0 · legions 00405ec0

### gmUIFlow
- [x] `gmUIFlow::RecvNotice_CharacterError` · 00406220 — ac2-ui pregame/flow.rs, pregame/ui.rs
- [x] `gmUIFlow::RecvNotice_ServerDied` · 00406410 — ac2-ui pregame/flow.rs, pregame/ui.rs
- [-] `gmUIFlow::RecvNotice_GunConnectionLost` · 004064a0 — GUN chat servers are not used (Turbine chat)
- [x] `gmUIFlow::RecvNotice_CharacterError` · 00406af0 — ac2-ui pregame/flow.rs, pregame/ui.rs
- [x] `gmUIFlow::RecvNotice_ServerDied` · 00406b00 — ac2-ui pregame/flow.rs, pregame/ui.rs
- [~] `gmUIFlow::gmUIFlow` · 00406b10 — pregame/ui.rs: character selection and disconnected made; intro, data patch (the client connects), keyboard, credits not ported

### ImportData
- [~] `ImportData::StreamPack` · 00406c60 · legions 00406970 — ac2-script: unpack direction only

### PStringBase<char>
- [x] `PStringBase<char>::UnPack` · 00406cb0 · legions 004069c0 — ac2-core pstring.rs: the DAT string descramble (ac2-ui, ac2-db use it)

### DualHash<PStringBase<char>,CheckpointExportData>
- [ ] `DualHash<PStringBase<char>,CheckpointExportData>::clear` · 00406d90 · legions 00406aa0
- [ ] `DualHash<PStringBase<char>,CheckpointExportData>::destroy` · 00406ea0 · legions 00406bb0
- [ ] `DualHash<PStringBase<char>,CheckpointExportData>::~DualHash<PStringBase<char>,CheckpointExportData>` · 00406ef0 · legions 00406c00
- [x] `DualHash<PStringBase<char>,CheckpointExportData>::UnPack` · 00406f50 — ac2-script reference/native unpack

### CheckpointExportData
- [ ] `CheckpointExportData::Pack` · 00407010 · legions 00406d20
- [x] `CheckpointExportData::UnPack` · 00407050 · legions 00406d60 — ac2-script reference/native unpack

### DualHash<PStringBase<char>,CheckpointExportData>
- [ ] `DualHash<PStringBase<char>,CheckpointExportData>::add_internal` · 004070b0 · legions 00406dc0

### DualHashData<PStringBase<char>,CheckpointExportData>
- [ ] `DualHashData<PStringBase<char>,CheckpointExportData>::DualHashData<PStringBase<char>,CheckpointExportData>` · 004071a0 · legions 00406eb0

### DualHash<PStringBase<char>,CheckpointExportData>
- [ ] `DualHash<PStringBase<char>,CheckpointExportData>::GetPackSize` · 004071e0
- [ ] `DualHash<PStringBase<char>,CheckpointExportData>::pack_size` · 004071f0 · legions 00406f00

### DualHashIter<PStringBase<char>,CheckpointExportData>
- [ ] `DualHashIter<PStringBase<char>,CheckpointExportData>::DualHashIter<PStringBase<char>,CheckpointExportData>` · 004072d0 · legions 00406fe0
- [ ] `DualHashIter<PStringBase<char>,CheckpointExportData>::operator=` · 00407300 · legions 00407010
- [ ] `DualHashIter<PStringBase<char>,CheckpointExportData>::DualHashIter<PStringBase<char>,CheckpointExportData>` · 00407350 · legions 00407060
- [ ] `DualHashIter<PStringBase<char>,CheckpointExportData>::DualHashIter<PStringBase<char>,CheckpointExportData>` · 004073b0 · legions 004070c0
- [ ] `DualHashIter<PStringBase<char>,CheckpointExportData>::operator==` · 004073f0 · legions 00407100
- [ ] `DualHashIter<PStringBase<char>,CheckpointExportData>::operator++` · 00407460 · legions 00407170

### DualHash<PStringBase<char>,CheckpointExportData>
- [ ] `DualHash<PStringBase<char>,CheckpointExportData>::Pack` · 004074a0

### ListIterator<DualHashData<PStringBase<char>,CheckpointExportData>*>
- [ ] `ListIterator<DualHashData<PStringBase<char>,CheckpointExportData>*>::operator=` · 004075f0 · legions 00407300

### ExportPackageArgs
- [ ] `ExportPackageArgs::~ExportPackageArgs` · 004076d0 · legions 004073e0

### ExportData
- [ ] `ExportData::~ExportData` · 00407720 · legions 00407430
- [~] `ExportData::StreamPack` · 00407770 · legions 00407480 — ac2-script: unpack direction only

## `ac2/client/src/ui/pregame/generation/gmavatarmodificationui.cpp.c` (10/27)

### AFrame
- [ ] `AFrame::AFrame` · 0040a9e0 · legions 0040a8a0

### Frame
- [ ] `Frame::Frame` · 0040aa00 · legions 0040a8c0
- [ ] `Frame::operator=` · 0040aa30 · legions 0040a8f0

### UIRegion
- [ ] `UIRegion::IsVisible` · 0040aaa0 · legions 0040a960

### gmAvatarModificationUI
- [~] `gmAvatarModificationUI::GetCamera` · 0040aad0 · legions 0040a990 — avatar_modification.rs: sliders, parts, randomize, zoom/rotation state, naming dialog; tooltips, music, the avatar's scene and camera are the client's / not modelled
- [x] `gmAvatarModificationUI::MakeCurrentMask` · 0040ab10 · legions 0040a9d0 — ac2-ui pregame/avatar_modification.rs
- [x] `gmAvatarModificationUI::MakePleaseWaitDialog` · 0040ab70 · legions 0040aa30 — ac2-ui pregame/avatar_modification.rs

### Frame
- [ ] `Frame::Frame` · 0040ac80 · legions 0040ab40

### UISceneEntity
- [ ] `UISceneEntity::GetFrame` · 0040acb0 · legions 0040ab70

### gmAvatarModificationUI
- [~] `gmAvatarModificationUI::UpdateCamera` · 0040ace0 · legions 0040aba0 — avatar_modification.rs: sliders, parts, randomize, zoom/rotation state, naming dialog; tooltips, music, the avatar's scene and camera are the client's / not modelled
- [~] `gmAvatarModificationUI::InitCamera` · 0040b0b0 · legions 0040af70 — avatar_modification.rs: sliders, parts, randomize, zoom/rotation state, naming dialog; tooltips, music, the avatar's scene and camera are the client's / not modelled

### PStringBase<char>
- [ ] `PStringBase<char>::operator+` · 0040b300 · legions 0040b200

### gmAvatarModificationUI
- [~] `gmAvatarModificationUI::Init` · 0040b420 · legions 0040b320 — avatar_modification.rs: sliders, parts, randomize, zoom/rotation state, naming dialog; tooltips, music, the avatar's scene and camera are the client's / not modelled
- [~] `gmAvatarModificationUI::UpdateByMask` · 0040b640 · legions 0040b540 — avatar_modification.rs: sliders, parts, randomize, zoom/rotation state, naming dialog; tooltips, music, the avatar's scene and camera are the client's / not modelled
- [x] `gmAvatarModificationUI::Temp` · 0040b6e0 · legions 0040b5e0 — ac2-ui pregame/avatar_modification.rs
- [x] `gmAvatarModificationUI::InitUIFromData` · 0040b760 · legions 0040b660 — ac2-ui pregame/avatar_modification.rs
- [~] `gmAvatarModificationUI::InitElements` · 0040b8c0 · legions 0040b7c0 — avatar_modification.rs: sliders, parts, randomize, zoom/rotation state, naming dialog; tooltips, music, the avatar's scene and camera are the client's / not modelled

### PList<unsigned_long>
- [ ] `PList<unsigned_long>::GetPackSize` · 0040bb40

### PList<float>
- [ ] `PList<float>::UnPack` · 0040bb60

### gmAvatarModificationUI
- [~] `gmAvatarModificationUI::ListenToMessage` · 0040bc90 · legions 0040bc60 — avatar_modification.rs: sliders, parts, randomize, zoom/rotation state, naming dialog; tooltips, music, the avatar's scene and camera are the client's / not modelled
- [x] `gmAvatarModificationUI::InitUITables` · 0040c0d0 — ac2-ui pregame/avatar_modification.rs
- [x] `gmAvatarModificationUI::ClearDataTable` · 0040c7e0 · legions 0040c800 — ac2-ui pregame/avatar_modification.rs
- [x] `gmAvatarModificationUI::GetDataTable` · 0040c880 · legions 0040c8a0 — ac2-ui pregame/avatar_modification.rs
- [~] `gmAvatarModificationUI::gmAvatarModificationUI` · 0040cc00 — avatar_modification.rs: sliders, parts, randomize, zoom/rotation state, naming dialog; tooltips, music, the avatar's scene and camera are the client's / not modelled
- [x] `gmAvatarModificationUI::~gmAvatarModificationUI` · 0040cff0 · legions 0040d010 — ac2-ui pregame/avatar_modification.rs
- [x] `gmAvatarModificationUI::Create` · 0040d290 — ac2-ui pregame/avatar_modification.rs
- [x] `gmAvatarModificationUI::Register` · 0040d2d0 · legions 0040d2f0 — ac2-ui pregame/avatar_modification.rs

## `ac2/client/src/ui/pregame/generation/gmavatarselectionui.cpp.c` (6/25)

### _InstanceID
- [ ] `_InstanceID::operator==` · 0040d2f0 · legions 0040d310
- [ ] `_InstanceID::operator!=` · 0040d310 · legions 0040d330

### ObjectSelection
- [ ] `ObjectSelection::ObjectSelection` · 0040d330 · legions 0040d350
- [ ] `ObjectSelection::operator=` · 0040d360 · legions 0040d380

### gmAvatarSelectionUI
- [x] `gmAvatarSelectionUI::MakePleaseWaitDialog` · 0040d390 · legions 0040d3b0 — ac2-ui pregame/avatar_selection.rs

### SmartArray<UISceneEntity_*>
- [ ] `SmartArray<UISceneEntity_*>::AddToEnd` · 0040d690 · legions 0040ff80

### PList<unsigned_long>
- [ ] `PList<unsigned_long>::Pack` · 0040d9d0 · legions 0040bad0
- [ ] `PList<unsigned_long>::UnPack` · 0040da30 · legions 0040da32

### BehaviorParams
- [ ] `BehaviorParams::~BehaviorParams` · 0040daf0 · legions 0040dae0

### gmAvatarSelectionUI
- [~] `gmAvatarSelectionUI::MouseOffObject` · 0040db40 · legions 0040db30 — avatar_selection.rs: avatars (phys obj, held objects, location, desc state), pick and selected-animation protocol; the 3D scene, behaviors, tooltip and music are the client's / not modelled
- [~] `gmAvatarSelectionUI::MouseOnObject` · 0040dbd0 · legions 0040dbc0 — avatar_selection.rs: avatars (phys obj, held objects, location, desc state), pick and selected-animation protocol; the 3D scene, behaviors, tooltip and music are the client's / not modelled

### BaseList<unsigned_long,11>
- [ ] `BaseList<unsigned_long,11>::BaseList<unsigned_long,11>` · 0040dca0 · legions 0040dc90

### gmAvatarSelectionUI
- [~] `gmAvatarSelectionUI::ListenToMessage` · 0040de00 · legions 0040dde0 — avatar_selection.rs: avatars (phys obj, held objects, location, desc state), pick and selected-animation protocol; the 3D scene, behaviors, tooltip and music are the client's / not modelled
- [~] `gmAvatarSelectionUI::ListenToMessage` · 0040dfe0 · legions 0040dfc0 — avatar_selection.rs: avatars (phys obj, held objects, location, desc state), pick and selected-animation protocol; the 3D scene, behaviors, tooltip and music are the client's / not modelled

### BaseList<unsigned_long,11>
- [ ] `BaseList<unsigned_long,11>::BaseList<unsigned_long,11>` · 0040e180 · legions 0040e160

### PHashTable<unsigned_long,gmRaceSexInfo>
- [ ] `PHashTable<unsigned_long,gmRaceSexInfo>::StreamPack` · 0040e200

### PHashTable<unsigned_long,unsigned_long>
- [ ] `PHashTable<unsigned_long,unsigned_long>::PHashTable<unsigned_long,unsigned_long>` · 0040e3f0 · legions 0040e3f0

### gmAvatarSelectionUI
- [~] `gmAvatarSelectionUI::InitEntities` · 0040e510 · legions 0040e510 — avatar_selection.rs: avatars (phys obj, held objects, location, desc state), pick and selected-animation protocol; the 3D scene, behaviors, tooltip and music are the client's / not modelled
- [x] `gmAvatarSelectionUI::InitGenders` · 0040eaa0 · legions 0040ea70 — ac2-ui pregame/avatar_selection.rs
- [x] `gmAvatarSelectionUI::InitRaces` · 0040ebf0 · legions 0040ebc0 — ac2-ui pregame/avatar_selection.rs

### PHashTable<unsigned_long,BaseList<unsigned_long,11>_>
- [ ] `PHashTable<unsigned_long,BaseList<unsigned_long,11>_>::StreamPack` · 0040ecc0 · legions 0040ec90

### gmAvatarSelectionUI
- [~] `gmAvatarSelectionUI::gmAvatarSelectionUI` · 0040f020 — avatar_selection.rs: avatars (phys obj, held objects, location, desc state), pick and selected-animation protocol; the 3D scene, behaviors, tooltip and music are the client's / not modelled
- [x] `gmAvatarSelectionUI::~gmAvatarSelectionUI` · 0040f270 · legions 0040f240 — ac2-ui pregame/avatar_selection.rs
- [x] `gmAvatarSelectionUI::Create` · 0040f500 — ac2-ui pregame/avatar_selection.rs
- [x] `gmAvatarSelectionUI::Register` · 0040f540 · legions 0040f510 — ac2-ui pregame/avatar_selection.rs

## `ac2/client/src/ui/pregame/generation/gmgenerationconfirmationui.cpp.c` (5/6)

### (free functions)
- [~] `NameInputFilter` · 0042cb60 — pregame/confirmation.rs is_name_character: English letters, ' - and space; other languages' letters not modelled

### gmGenerationConfirmationUI
- [x] `gmGenerationConfirmationUI::gmGenerationConfirmationUI` · 0042cb90 · legions 0042c950 — ac2-ui pregame/confirmation.rs (Legions: expansion/drudge entitlement checks, nonadminclient buttons)
- [x] `gmGenerationConfirmationUI::GhostButtons` · 0042cc90 · legions 0042ca50 — ac2-ui pregame/confirmation.rs (Legions: expansion/drudge entitlement checks, nonadminclient buttons)
- [x] `gmGenerationConfirmationUI::SetErrorMsg` · 0042cda0 · legions 0042cb60 — ac2-ui pregame/confirmation.rs (Legions: expansion/drudge entitlement checks, nonadminclient buttons)
- [x] `gmGenerationConfirmationUI::ListenToMessage` · 0042cde0 · legions 0042cba0 — ac2-ui pregame/confirmation.rs (Legions: expansion/drudge entitlement checks, nonadminclient buttons)
- [x] `gmGenerationConfirmationUI::RecvNotice_CharGenVerification` · 0042d080 — ac2-ui pregame/confirmation.rs (Legions: expansion/drudge entitlement checks, nonadminclient buttons)

## `ac2/client/src/ui/pregame/gmcharacterselectionui.cpp.c` (17/23)

### CharacterIdentity
- [x] `CharacterIdentity::Invalid` · 0040f560 · legions 0040f530 — ac2-ui pregame/character_selection.rs, pregame/arms.rs

### gmCharacterSelectionUI
- [x] `gmCharacterSelectionUI::GhostButtons` · 0040f590 · legions 0040f560 — ac2-ui pregame/character_selection.rs, pregame/arms.rs
- [~] `gmCharacterSelectionUI::Start3DSelection` · 0040f6c0 · legions 0040f690 — character_selection.rs: occupants per arm and the pick protocol; the 3D figures and the scene pick are the client's to render (ArmOccupant, wants_pick/finish_pick)
- [x] `gmCharacterSelectionUI::DoDelete` · 0040f720 · legions 0040f6f0 — ac2-ui pregame/character_selection.rs, pregame/arms.rs
- [x] `gmCharacterSelectionUI::MakePleaseWaitDialog` · 0040f810 · legions 0040f7e0 — ac2-ui pregame/character_selection.rs, pregame/arms.rs

### UIElement_ListBox
- [x] `UIElement_ListBox::GetSelectedIndex` · 0040f880 · legions 0040f850 — ac2-ui list_box_select.rs

### gmCharacterSelectionUI
- [x] `gmCharacterSelectionUI::MoveArm` · 0040f8c0 · legions 0040f890 — ac2-ui pregame/character_selection.rs, pregame/arms.rs
- [x] `gmCharacterSelectionUI::UnParentandDeleteArms` · 0040fa40 · legions 0040fa10 — ac2-ui pregame/character_selection.rs, pregame/arms.rs
- [x] `gmCharacterSelectionUI::RecvNotice_CloseConfirmationDialog` · 0040fae0 — ac2-ui pregame/character_selection.rs, pregame/arms.rs
- [x] `gmCharacterSelectionUI::SetErrorMsg` · 0040fba0 · legions 0040fb70 — ac2-ui pregame/character_selection.rs, pregame/arms.rs
- [x] `gmCharacterSelectionUI::MoveArms` · 0040fbe0 · legions 0040fbb0 — ac2-ui pregame/character_selection.rs, pregame/arms.rs
- [x] `gmCharacterSelectionUI::RecvNotice_CharGenVerification` · 0040fca0 — ac2-ui pregame/character_selection.rs, pregame/arms.rs
- [x] `gmCharacterSelectionUI::MakeDeleteDialog` · 0040ff20 · legions 0040fef0 — ac2-ui pregame/character_selection.rs, pregame/arms.rs
- [~] `gmCharacterSelectionUI::ListenToMessage` · 0040ffb0 · legions 0040ffd0 — character_selection.rs: all buttons, list, scene presses; the name's ellipsis (TruncateText on download_done) not modelled
- [x] `gmCharacterSelectionUI::InitArms` · 00410510 · legions 00410530 — ac2-ui pregame/character_selection.rs, pregame/arms.rs
- [x] `gmCharacterSelectionUI::~gmCharacterSelectionUI` · 00410750 · legions 00410770 — ac2-ui pregame/character_selection.rs, pregame/arms.rs
- [~] `gmCharacterSelectionUI::AddPhysObjToArm` · 004108e0 · legions 004108e0 — character_selection.rs: occupants per arm and the pick protocol; the 3D figures and the scene pick are the client's to render (ArmOccupant, wants_pick/finish_pick)
- [~] `gmCharacterSelectionUI::Finish3DSelection` · 00410a10 · legions 00410a10 — character_selection.rs: occupants per arm and the pick protocol; the 3D figures and the scene pick are the client's to render (ArmOccupant, wants_pick/finish_pick)
- [~] `gmCharacterSelectionUI::ListenToMessage` · 00410ad0 · legions 00410ad0 — character_selection.rs: all buttons, list, scene presses; the name's ellipsis (TruncateText on download_done) not modelled
- [x] `gmCharacterSelectionUI::Update` · 00410c00 · legions 00410c00 — ac2-ui pregame/character_selection.rs, pregame/arms.rs
- [~] `gmCharacterSelectionUI::gmCharacterSelectionUI` · 00410f00 — character_selection.rs: roots, handlers, Update; background music not played
- [x] `gmCharacterSelectionUI::Create` · 00411220 — ac2-ui pregame/character_selection.rs, pregame/arms.rs
- [x] `gmCharacterSelectionUI::Register` · 00411240 · legions 00411220 — ac2-ui pregame/character_selection.rs, pregame/arms.rs

## `ac2/client/src/ui/pregame/gmcreditsui.cpp.c` (0/15)

### gmCreditsUI
- [ ] `gmCreditsUI::MakePleaseWaitDialog` · 00412900 · legions 00412780
- [ ] `gmCreditsUI::~gmCreditsUI` · 00412950 · legions 004127d0
- [ ] `gmCreditsUI::GetX` · 004129c0 · legions 00412840
- [ ] `gmCreditsUI::GetY` · 004129f0 · legions 00412870
- [ ] `gmCreditsUI::UpdatePicture` · 00412a20 · legions 004128a0
- [ ] `gmCreditsUI::Close` · 00412b10 · legions 00412990
- [ ] `gmCreditsUI::GetSoundID` · 00412b90 · legions 00412a10
- [ ] `gmCreditsUI::UpdateSound` · 00412c50 · legions 00412ad0
- [ ] `gmCreditsUI::UpdateText` · 00412d00 · legions 00412b80
- [ ] `gmCreditsUI::ListenToMessage` · 00412e30 · legions 00412cb0
- [ ] `gmCreditsUI::Update` · 00412e60 · legions 00412ce0
- [ ] `gmCreditsUI::Initialize` · 00412e80 · legions 00412d00
- [ ] `gmCreditsUI::gmCreditsUI` · 00413360
- [ ] `gmCreditsUI::Create` · 004134b0
- [ ] `gmCreditsUI::Register` · 004134d0 · legions 00413350

## `ac2/client/src/ui/pregame/gmdatapatchui.cpp.c` (0/19)

### gmDataPatchUI
- [ ] `gmDataPatchUI::ListenToMessage` · 00411260 · legions 00411240
- [ ] `gmDataPatchUI::Refresh` · 004112a0 · legions 00411280
- [ ] `gmDataPatchUI::OnHeartbeat` · 004113b0

### CPluginManager
- [ ] `CPluginManager::RemovePluginPrototype` · 00411500 · legions 004114e0

### gmClient
- [ ] `gmClient::SetStartChar` · 00411550 · legions 00411530
- [ ] `gmClient::SetCreateChar` · 004115a0 · legions 00411580

### gmDataPatchUI
- [ ] `gmDataPatchUI::SetErrorMsg` · 004115f0 · legions 004115d0
- [ ] `gmDataPatchUI::OnConnectFailed` · 00411630 · legions 00411610
- [ ] `gmDataPatchUI::Connect` · 00411890 · legions 00411870

### (free functions)
- [ ] `operator+` · 00411950 · legions 00411930

### gmDataPatchUI
- [ ] `gmDataPatchUI::~gmDataPatchUI` · 004119e0 · legions 004119c0
- [~] `gmDataPatchUI::Update` · 00411ac0 · legions 00411aa0 — ac2-ui PregameUi::character_set (first set -> character selection) and bin/ac2 --character (start character enters at once); patching not ported
- [ ] `gmDataPatchUI::RecvNotice_CharGenVerification` · 00411f10
- [ ] `gmDataPatchUI::OnDDDEvent` · 00411fd0
- [ ] `gmDataPatchUI::gmDataPatchUI` · 004121d0 · legions 00412050
- [ ] `gmDataPatchUI::Create` · 00412330

### CPluginManager
- [ ] `CPluginManager::AddPluginPrototype` · 00412350 · legions 004121d0

### gmDataPatchUI
- [ ] `gmDataPatchUI::Register` · 004123b0 · legions 00412230
- [ ] `gmDataPatchUI::OnNetStatusChange` · 004123d0

## `ac2/client/src/ui/pregame/gmdisconnectedui.cpp.c` (5/5)

### gmDisconnectedUI
- [x] `gmDisconnectedUI::gmDisconnectedUI` · 004125c0 · legions 00412440 — ac2-ui pregame/disconnected.rs
- [x] `gmDisconnectedUI::Create` · 00412630 — ac2-ui pregame/disconnected.rs
- [x] `gmDisconnectedUI::ListenToMessage` · 00412650 · legions 004124d0 — ac2-ui pregame/disconnected.rs
- [x] `gmDisconnectedUI::Register` · 004126b0 · legions 00412530 — ac2-ui pregame/disconnected.rs
- [x] `gmDisconnectedUI::SetErrorMsg` · 004126d0 · legions 00412550 — ac2-ui pregame/disconnected.rs

## `ac2/client/src/ui/pregame/gmintroui.cpp.c` (0/6)

### gmIntroUI
- [ ] `gmIntroUI::gmIntroUI` · 00412710
- [ ] `gmIntroUI::Create` · 00412830
- [ ] `gmIntroUI::Register` · 00412850 · legions 004126d0
- [ ] `gmIntroUI::Close` · 00412870 · legions 004126f0
- [ ] `gmIntroUI::ListenToMessage` · 004128c0 · legions 00412740
- [ ] `gmIntroUI::ListenToMessage` · 004128e0 · legions 00412760

## `ac2/client/src/ui/pregame/gmkeyboardui.cpp.c` (0/19)

### gmKeyboardUI
- [ ] `gmKeyboardUI::ConfirmationBoxesOpen` · 004134f0 · legions 00413370
- [ ] `gmKeyboardUI::UpdateButtons` · 00413530 · legions 004133b0
- [ ] `gmKeyboardUI::DoClearAll` · 004135c0 · legions 00413440
- [ ] `gmKeyboardUI::CloseAllDialogs` · 004136a0 · legions 00413520
- [ ] `gmKeyboardUI::~gmKeyboardUI` · 00413720 · legions 004135a0
- [ ] `gmKeyboardUI::InqKeyStringInfo` · 004137d0 · legions 00413650

### IntrusiveSmartPointer<PFileNode>
- [ ] `IntrusiveSmartPointer<PFileNode>::~IntrusiveSmartPointer<PFileNode>` · 00413930 · legions 004137b0
- [ ] `IntrusiveSmartPointer<PFileNode>::create` · 00413960 · legions 004137e0

### gmKeyboardUI
- [ ] `gmKeyboardUI::Close` · 004139d0 · legions 00413850
- [ ] `gmKeyboardUI::FinishSet` · 00413a70 · legions 004138f0
- [ ] `gmKeyboardUI::StartSet` · 00413d10 · legions 00413b50
- [ ] `gmKeyboardUI::InitializeKeyboardList` · 00414140 · legions 00413f80
- [ ] `gmKeyboardUI::SaveKeyboardList` · 00414510 · legions 00414350
- [ ] `gmKeyboardUI::RecvNotice_CloseConfirmationDialog` · 004149f0
- [ ] `gmKeyboardUI::gmKeyboardUI` · 00414bb0 · legions 004149f0
- [ ] `gmKeyboardUI::Create` · 00414e50
- [ ] `gmKeyboardUI::ListenToMessage` · 00414e70 · legions 00414c60
- [ ] `gmKeyboardUI::KeyHitHandler` · 00415140 · legions 00414f30
- [ ] `gmKeyboardUI::Register` · 004151f0 · legions 00414fe0

## `ac2/client/src/ui/util/dialog/confirmationdialog.cpp.c` (3/3)

### ConfirmationDialog
- [x] `ConfirmationDialog::ListenToMessage` · 0042d7f0 · legions 0042d5c0 — ac2-ui dialog.rs
- [x] `ConfirmationDialog::SetText` · 0042d830 · legions 0042d600 — ac2-ui dialog.rs
- [x] `ConfirmationDialog::ConfirmationDialog` · 0042d850 · legions 0042d620 — ac2-ui dialog.rs

## `ac2/client/src/ui/util/dialog/dialogfactory.cpp.c` (3/4)

### DialogFactory
- [~] `DialogFactory::CreateDialogInternal` · 00415210 · legions 00415000 — dialog.rs: confirmation, wait, message; resurrect and timed dialogs not made
- [x] `DialogFactory::MakeDialog` · 004155b0 · legions 004153a0 — ac2-ui dialog.rs
- [x] `DialogFactory::DialogDone` · 00415740 · legions 00415530 — ac2-ui dialog.rs
- [x] `DialogFactory::CloseDialog` · 00415810 · legions 00415600 — ac2-ui dialog.rs

## `ac2/client/src/ui/util/dialog/messagedialog.cpp.c` (3/3)

### MessageDialog
- [x] `MessageDialog::ListenToMessage` · 0042d300 · legions 0042d0d0 — ac2-ui dialog.rs
- [x] `MessageDialog::SetText` · 0042d330 · legions 0042d100 — ac2-ui dialog.rs
- [x] `MessageDialog::MessageDialog` · 0042d350 · legions 0042d120 — ac2-ui dialog.rs

## `ac2/client/src/ui/util/dialog/resurrectdialog.cpp.c` (0/3)

### ResurrectDialog
- [ ] `ResurrectDialog::ListenToMessage` · 0042d3f0 · legions 0042d1c0
- [ ] `ResurrectDialog::SetText` · 0042d420 · legions 0042d1f0
- [ ] `ResurrectDialog::ResurrectDialog` · 0042d440 · legions 0042d210

## `ac2/client/src/ui/util/dialog/timeddialog.cpp.c` (0/5)

### TimedDialog
- [ ] `TimedDialog::ListenToMessage` · 0042d590 · legions 0042d360
- [ ] `TimedDialog::ListenToMessage` · 0042d5d0 · legions 0042d3a0
- [ ] `TimedDialog::SetDelay` · 0042d670 · legions 0042d440
- [ ] `TimedDialog::TimedDialog` · 0042d6e0 · legions 0042d4b0
- [ ] `TimedDialog::SetText` · 0042d7b0 · legions 0042d580

## `ac2/client/src/ui/util/dialog/waitdialog.cpp.c` (2/2)

### WaitDialog
- [x] `WaitDialog::SetText` · 0042d4e0 · legions 0042d2b0 — ac2-ui dialog.rs
- [x] `WaitDialog::WaitDialog` · 0042d500 · legions 0042d2d0 — ac2-ui dialog.rs

## `portal/client/src/cimelib/cime.cpp.c` (0/23)

### MultibyteProcessor
- [ ] `MultibyteProcessor::MultibyteProcessor` · 004bd3b0 · legions 004b16d0
- [ ] `MultibyteProcessor::UpdateInternals` · 004bd3c0 · legions 004b16e0
- [ ] `MultibyteProcessor::GetUnicodeChar` · 004bd420 · legions 004b1740

### CIme
- [ ] `CIme::DrawRect` · 004bd4a0
- [ ] `CIme::SurfaceToClip` · 004bd4f0 · legions 004b1810
- [ ] `CIme::DrawTriangle` · 004bd5a0 · legions 004b18c0
- [ ] `CIme::DrawFans` · 004bd6a0

### CImeFontWrapper
- [ ] `CImeFontWrapper::SetPosition` · 004bd740 · legions 004b1a60
- [ ] `CImeFontWrapper::SetFont` · 004bd760 · legions 004b1a80

### CIme
- [ ] `CIme::Deactivate` · 004bd790 · legions 004b1ab0
- [ ] `CIme::Render` · 004bd810 · legions 004b1b30
- [ ] `CIme::IsImeEnabled` · 004bd860 · legions 004b1b80
- [ ] `CIme::IgnoreHotKey` · 004bd870 · legions 004b1b90
- [ ] `CIme::ProcessMessage` · 004bd880
- [ ] `CIme::SetCaretPosition` · 004bd890 · legions 004b1bb0
- [ ] `CIme::ToggleLanguageBar` · 004bd8e0
- [ ] `CIme::IsSendingKeyMessage` · 004bd8f0 · legions 004b1c10
- [ ] `CIme::Activate` · 004bd900 · legions 004b1c20
- [ ] `CIme::SetAppearanceToDefault` · 004bdb70 · legions 004b1e90
- [ ] `CIme::Init` · 004bdc70 · legions 004b1f90

### MultibyteProcessor
- [ ] `MultibyteProcessor::GetUnicodeString` · 004bdd00 · legions 004b2020

### CImeFontWrapper
- [ ] `CImeFontWrapper::GetTextExtent` · 004bddb0 · legions 004b20d0
- [ ] `CImeFontWrapper::DrawTextA` · 004bde10 · legions 004b2130

## `portal/client/src/cinputmap/cinputmanager.cpp.c` (4/16)

### CInputManager
- [ ] `CInputManager::CInputManager` · 00599540 · legions 005665e0

### NIListIter<NoticeHandler_*>
- [ ] `NIListIter<NoticeHandler_*>::RemoveCur` · 00599570 · legions 004c0420

### ICIDM
- [ ] `ICIDM::PushKeyMap` · 005995e0 · legions 00566610
- [ ] `ICIDM::GetNameFromKey` · 00599610 · legions 00566640

### CInputManager
- [ ] `CInputManager::~CInputManager` · 00599630 · legions 00566660
- [ ] `CInputManager::AddHandlerToList` · 00599930 · legions 00566960
- [ ] `CInputManager::RemoveHandlerFromList` · 00599980 · legions 005669b0
- [ ] `CInputManager::RegisterInputHandler` · 005999d0 · legions 00566a00
- [ ] `CInputManager::UnregisterInputHandler` · 00599ae0 · legions 00566b10
- [x] `CInputManager::CallCharacterHandler` · 00599b50 · legions 00566b80 — ac2-ui input.rs / text_edit.rs: typing into text elements
- [x] `CInputManager::CallActionHandler` · 00599b90 · legions 00566bc0 — ac2-ui input.rs
- [x] `CInputManager::CallMouseLookHandler` · 00599c00 · legions 00566c30 — ac2-ui input.rs
- [ ] `CInputManager::CallKeyHitHandler` · 00599c40 · legions 00566c70
- [ ] `CInputManager::CallFocusSwitchHandler` · 00599c80 · legions 00566cb0
- [~] `CInputManager::SetMouseLookMode` · 00599ce0 · legions 00566d10 — ac2-ui input.rs: keys, buttons, wheel, axes, cursor; no text/keystone mode, no device acquisition
- [x] `CInputManager::SetTextMode` · 00599d40 · legions 00566d70 — ac2-ui input.rs / text_edit.rs: typing into text elements

## `portal/client/src/cinputmap/ckeymap.cpp.c` (3/23)

### MapGUID
- [x] `MapGUID::StreamPack` · 00599d60 · legions 00566d90 — ac2-ui maps.rs

### HashIterator<unsigned_long,unsigned_long>
- [ ] `HashIterator<unsigned_long,unsigned_long>::HashIterator<unsigned_long,unsigned_long>` · 00599e10 · legions 004b66a0

### PSUtils
- [ ] `PSUtils::is_uint32` · 00599e30 · legions 00566e40

### (free functions)
- [ ] `compute_hash` · 00599ea0 · legions 00566eb0

### ActionMapping
- [ ] `ActionMapping::operator=` · 00599f10 · legions 00566f20

### CKeyMap
- [ ] `CKeyMap::GetNameBySemantic` · 0059a000 · legions 00566fc0

### ActionMapping
- [x] `ActionMapping::StreamPack` · 0059a1c0 · legions 00567180 — ac2-ui maps.rs

### CKeyMap
- [ ] `CKeyMap::AddAction` · 0059a210 · legions 005671d0
- [x] `CKeyMap::StreamPack` · 0059a2b0 · legions 00567270 — ac2-ui maps.rs
- [ ] `CKeyMap::GetSemanticByName` · 0059a320 · legions 005672e0
- [ ] `CKeyMap::UpdateVersion` · 0059a360 · legions 00567320
- [ ] `CKeyMap::Destroy` · 0059a680
- [ ] `CKeyMap::LoadSemantics` · 0059a690 · legions 00567650

### PList<TurbineUserStatus>
- [ ] `PList<TurbineUserStatus>::Pack` · 0059d4a0 · legions 0056a460

### PList<ActionMapping>
- [ ] `PList<ActionMapping>::UnPack` · 0059d4f0 · legions 0056a4c4

### CKeyMap
- [ ] `CKeyMap::CKeyMap` · 0059d680 · legions 0056a640
- [ ] `CKeyMap::GetDBOType` · 0059d710
- [ ] `CKeyMap::~CKeyMap` · 0059d720 · legions 0056a6e0
- [-] `CKeyMap::FromFileNode` · 0059d820 · legions 0056a7e0 — text source format (admin tools)
- [-] `CKeyMap::ToFileNode` · 0059de20 · legions 0056ade0 — text source format (admin tools)
- [ ] `CKeyMap::CopyInto` · 0059e140 · legions 0056b100
- [ ] `CKeyMap::Allocate` · 0059e210
- [-] `CKeyMap::FromFileNode` · 0059e240 — text source format (admin tools)

## `portal/client/src/cinputmap/directx/cinputmanager_win32.cpp.c` (11/35)

### CInputManager_WIN32
- [x] `CInputManager_WIN32::IsMetaKey` · 0059e260 · legions 0056b200 — ac2-ui input.rs
- [x] `CInputManager_WIN32::MetaModeFromKey` · 0059e320 · legions 0056b2c0 — ac2-ui input.rs
- [ ] `CInputManager_WIN32::PollDevice` · 0059e400 · legions 0056b3a0
- [x] `CInputManager_WIN32::IsUIAction` · 0059e470 · legions 0056b410 — ac2-ui input.rs / text_edit.rs: typing into text elements
- [x] `CInputManager_WIN32::UpdateCharacter` · 0059e500 · legions 0056b4a0 — ac2-ui input.rs / text_edit.rs: typing into text elements
- [ ] `CInputManager_WIN32::OurMapVirtualKey` · 0059e520 · legions 0056b4c0

### _STL::_Rb_tree_iterator<_STL::pair<ActionWithModifiers_const_,ActionMapValue>,_STL::_Nonconst_traits<_STL::pair<ActionWithModifiers_const_,ActionMapValue>_>_>
- [ ] `_STL::_Rb_tree_iterator<_STL::pair<ActionWithModifiers_const_,ActionMapValue>,_STL::_Nonconst_traits<_STL::pair<ActionWithModifiers_const_,ActionMapValue>_>_>::operator++` · 0059e680 · legions 004c7b10

### CInputManager_WIN32
- [ ] `CInputManager_WIN32::AcquireAll` · 0059e6c0 · legions 0056b590
- [ ] `CInputManager_WIN32::UnacquireAll` · 0059e700 · legions 0056b5d0
- [ ] `CInputManager_WIN32::ReleaseDevices` · 0059e730 · legions 0056b600
- [ ] `CInputManager_WIN32::SwitchMouseMode` · 0059e770 · legions 0056b640
- [ ] `CInputManager_WIN32::SetFocus` · 0059e850 · legions 0056b720
- [ ] `CInputManager_WIN32::GetNameFromKey` · 0059e970 · legions 0056b890

### SmartArray<CInputManager_WIN32::InputDevice>
- [ ] `SmartArray<CInputManager_WIN32::InputDevice>::AddToEnd` · 0059eb80 · legions 0056ba40

### CInputManager_WIN32
- [ ] `CInputManager_WIN32::Cleanup` · 0059ebe0 · legions 0056baa0
- [ ] `CInputManager_WIN32::AddDevice` · 0059ec20 · legions 0056bae0
- [ ] `CInputManager_WIN32::EnumSuitableDevicesCB` · 0059ee30

### HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>
- [ ] `HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>::operator=` · 0059ef60 · legions 0056bed0

### CInputManager_WIN32
- [ ] `CInputManager_WIN32::Init` · 0059f210 · legions 0056bfc0
- [x] `CInputManager_WIN32::GetInputActionFromKey` · 0059f5d0 · legions 0056c380 — ac2-ui input.rs
- [x] `CInputManager_WIN32::ReleaseKey` · 0059f690 · legions 0056c440 — ac2-ui input.rs
- [~] `CInputManager_WIN32::SwitchTextMode` · 0059f750 · legions 0056c500 — ac2-ui text_edit.rs: no selection, clipboard, IME, page up/down, cursor flashing or view scrolling
- [x] `CInputManager_WIN32::SetKey` · 0059f980 · legions 0056c730 — ac2-ui input.rs
- [~] `CInputManager_WIN32::UpdateVirtualKey` · 0059fa30 · legions 0056c7e0 — ac2-ui text_edit.rs: no selection, clipboard, IME, page up/down, cursor flashing or view scrolling

### _STL::deque<HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>,_STL::allocator<HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>_>_>
- [ ] `_STL::deque<HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>,_STL::allocator<HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>_>_>::~deque<HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>,_STL::allocator<HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>_>_>` · 0059fbf0 · legions 00645850

### CInputManager_WIN32
- [x] `CInputManager_WIN32::PopKeyMap` · 0059ff00 · legions 0056cc00 — ac2-ui input.rs
- [ ] `CInputManager_WIN32::GetKeyFromInputAction` · 0059ff60 · legions 0056cc60
- [ ] `CInputManager_WIN32::~CInputManager_WIN32` · 005a01f0 · legions 0056cef0
- [ ] `CInputManager_WIN32::CInputManager_WIN32` · 005a03a0 · legions 0056d0a0
- [x] `CInputManager_WIN32::CheckDoubleClick` · 005a04a0 · legions 0056d1a0 — ac2-ui input.rs
- [~] `CInputManager_WIN32::ProcessDeviceData` · 005a05e0 · legions 0056d2e0 — ac2-ui input.rs: keys, buttons, wheel, axes, cursor; no text/keystone mode, no device acquisition
- [x] `CInputManager_WIN32::PushKeyMap_Internal` · 005a09b0 · legions 0056d6b0 — ac2-ui input.rs
- [~] `CInputManager_WIN32::GetInput` · 005a0ac0 · legions 0056d7c0 — ac2-ui input.rs: keys, buttons, wheel, axes, cursor; no text/keystone mode, no device acquisition
- [~] `CInputManager_WIN32::PushKeyMap` · 005a0da0 · legions 0056daa0 — ac2-ui controls.rs: the DAT key maps only; saved .keymap files are not read
- [x] `CInputManager_WIN32::PushKeyMap` · 005a0f80 — ac2-ui input.rs

## `portal/client/src/cregion/cdetailmap/cdetailmapdesc.cpp.c` (0/22)

### MyRect
- [ ] `MyRect::MyRect` · 0051c390
- [ ] `MyRect::Include` · 0051c3b0
- [ ] `MyRect::Include` · 0051c400

### CDetailMapDesc
- [ ] `CDetailMapDesc::StreamPack` · 0051c4e0 · legions 00509c80
- [ ] `CDetailMapDesc::GetSubDataIDs` · 0051c560 · legions 00509d00
- [ ] `CDetailMapDesc::CDetailMapDesc` · 0051c6b0
- [ ] `CDetailMapDesc::GetDBOType` · 0051c6e0

### SmartArray<MyRect>
- [ ] `SmartArray<MyRect>::operator=` · 0051c780

### CDetailMapDesc
- [ ] `CDetailMapDesc::Allocate` · 0051c7f0
- [-] `CDetailMapDesc::ToFileNode` · 0051c860 — text source format (admin tools)
- [-] `CDetailMapDesc::LoadMap` · 0051c970 — text source loader, admin build only

### CDetailMapRow
- [ ] `CDetailMapRow::operator=` · 0051cab0 · legions 00509e00

### MapInfo
- [ ] `MapInfo::MapInfo` · 0051cb10
- [ ] `MapInfo::InqPString` · 0051cbe0

### CDetailMapDesc
- [ ] `CDetailMapDesc::InqRowFromValue` · 0051cc70 · legions 00509e60
- [ ] `CDetailMapDesc::InqRowFromXY` · 0051ce30 · legions 00509fd0

### PSmartArray<CDetailMapRow>
- [ ] `PSmartArray<CDetailMapRow>::StreamPack` · 0051cef0 · legions 0050a040

### SmartArray<CDetailMapRow>
- [ ] `SmartArray<CDetailMapRow>::AddToEnd` · 0051d0b0

### CDetailMapDesc
- [-] `CDetailMapDesc::CheckDupAdd` · 0051d100 — text source format (admin tools)
- [-] `CDetailMapDesc::LoadRow` · 0051d160 — text source loader, admin build only
- [ ] `CDetailMapDesc::CheckAll` · 0051d1f0
- [-] `CDetailMapDesc::FromFileNode` · 0051d570 — text source format (admin tools)

## `portal/client/src/cregion/cdetailmap/cdetailmaprow.cpp.c` (0/7)

### CDetailMapRow
- [ ] `CDetailMapRow::CheckXY` · 0051b8c0
- [ ] `CDetailMapRow::Destroy` · 0051b900 · legions 00509a10
- [ ] `CDetailMapRow::StreamPack` · 0051b930 · legions 00509a40
- [ ] `CDetailMapRow::GetSubDataIDs` · 0051b9e0 · legions 00509af0
- [ ] `CDetailMapRow::CDetailMapRow` · 0051ba20 · legions 00509b30
- [ ] `CDetailMapRow::~CDetailMapRow` · 0051ba50 · legions 00509b60
- [-] `CDetailMapRow::FromFileNode` · 0051baa0 — text source format (admin tools)

## `portal/client/src/cregion/cmapnote/cmapnotedesc.cpp.c` (0/10)

### CDetailMapDesc
- [ ] `CDetailMapDesc::Destroy` · 00528e60

### CMapNoteDesc
- [ ] `CMapNoteDesc::GetSubDataIDs` · 00528e70 · legions 0050fcd0
- [ ] `CMapNoteDesc::CMapNoteDesc` · 00528ec0
- [ ] `CMapNoteDesc::GetDBOType` · 00528ef0
- [ ] `CMapNoteDesc::Allocate` · 00528f00
- [-] `CMapNoteDesc::ToFileNode` · 00528f70 — text source format (admin tools)
- [ ] `CMapNoteDesc::CheckStrings` · 00529080

### SmartArray<CMapNoteRow>
- [ ] `SmartArray<CMapNoteRow>::AddToEnd` · 005291b0

### CMapNoteDesc
- [-] `CMapNoteDesc::LoadRow` · 00529210 — text source loader, admin build only
- [-] `CMapNoteDesc::FromFileNode` · 00529280 — text source format (admin tools)

## `portal/client/src/cregion/cmapnote/cmapnoterow.cpp.c` (0/4)

### CMapNoteRow
- [ ] `CMapNoteRow::CMapNoteRow` · 0051bf70
- [ ] `CMapNoteRow::~CMapNoteRow` · 0051bf90 · legions 00509bf0
- [ ] `CMapNoteRow::StreamPack` · 0051bfb0 · legions 00509c10
- [-] `CMapNoteRow::FromFileNode` · 0051c020 — text source format (admin tools)

## `portal/client/src/render/ui/renderui.cpp.c` (0/17)

### RenderUI
- [ ] `RenderUI::CreateUISurfaceIndexBuffer` · 005c3d00 · legions 0058ae10
- [ ] `RenderUI::SetCursorProperties` · 005c3d80 · legions 0058ae90
- [ ] `RenderUI::SetDefaultCursor` · 005c3f20 · legions 0058b030
- [ ] `RenderUI::ShowCursor` · 005c3f80 · legions 0058b090
- [ ] `RenderUI::LockList` · 005c3fc0 · legions 0058b0d0
- [ ] `RenderUI::InitCursor` · 005c4000 · legions 0058b110
- [ ] `RenderUI::SetCursorPosition` · 005c4160 · legions 0058b270
- [ ] `RenderUI::Startup` · 005c42f0 · legions 0058b400
- [ ] `RenderUI::RenderObjects` · 005c4340 · legions 0058b430
- [ ] `RenderUI::GetLastObject` · 005c4460 · legions 0058b520
- [ ] `RenderUI::GetPreviousObject` · 005c4480 · legions 0058b540
- [ ] `RenderUI::LinkObject` · 005c4510 · legions 0058b560
- [ ] `RenderUI::Shutdown` · 005c45d0 · legions 0058b660
- [ ] `RenderUI::UnlinkObject` · 005c4660 · legions 0058b6e0
- [ ] `RenderUI::CleanupObjectList` · 005c46e0 · legions 0058b760
- [ ] `RenderUI::SetObjectDepth` · 005c4750 · legions 0058b7d0
- [ ] `RenderUI::UnlockList` · 005c4780 · legions 0058b800

## `portal/client/src/render/ui/uinosurfaceobject.cpp.c` (0/9)

### UINoSurfaceObject
- [ ] `UINoSurfaceObject::UINoSurfaceObject` · 00735650 · legions 006b1350

### PHYSICSOBJREP
- [ ] `PHYSICSOBJREP::GetBehaviorTableID` · 00735680 · legions 006b1260

### UINoSurfaceObject
- [ ] `UINoSurfaceObject::GetObjectYPos` · 00735690 · legions 006b1270

### CPhysicsObj
- [ ] `CPhysicsObj::get_cell_id` · 007356a0 · legions 006b1280

### UINoSurfaceObject
- [ ] `UINoSurfaceObject::GetObjectHeight` · 007356b0 · legions 006b0fd0

### Vector
- [ ] `Vector::Vector` · 007356c0 · legions 006b0fe0

### UINoSurfaceObject
- [ ] `UINoSurfaceObject::SetScreenPosition` · 007356d0 · legions 006b1380
- [ ] `UINoSurfaceObject::SetScreenPosition` · 007356f0 · legions 006b13a0
- [ ] `UINoSurfaceObject::ResizeSurface` · 00735710 · legions 006b13c0

## `portal/client/src/render/ui/uiobject.cpp.c` (0/2)

### UIObject
- [ ] `UIObject::UIObject` · 005d9370 · legions 0059b780
- [ ] `UIObject::~UIObject` · 005d93c0 · legions 0059b7d0

## `portal/client/src/render/ui/uirectangleobject.cpp.c` (0/4)

### UIRectangleObject
- [ ] `UIRectangleObject::UIRectangleObject` · 00735270 · legions 006b0f70

### UINoSurfaceObject
- [ ] `UINoSurfaceObject::RayTest` · 007352d0 · legions 006b0ff0
- [ ] `UINoSurfaceObject::GetScreenLocalPosition` · 00735340 · legions 006b13e0

### UIRectangleObject
- [ ] `UIRectangleObject::Render` · 00735390 · legions 006b1060

## `portal/client/src/render/ui/uiscene.cpp.c` (0/23)

### (free functions)
- [ ] `STREAMPACK_PACKOBJ` · 005acbc0 · legions 00578b20

### UIScene
- [ ] `UIScene::GetEntity` · 005acd10 · legions 00578c70
- [ ] `UIScene::RemoveEntity` · 005acd50 · legions 00578cb0
- [ ] `UIScene::DeleteEntity` · 005acdd0 · legions 00578d30
- [ ] `UIScene::DeleteAllEntities` · 005ace50 · legions 00578db0
- [ ] `UIScene::StartSelection` · 005acea0 · legions 00578e00
- [ ] `UIScene::Think` · 005acee0 · legions 00578e40
- [ ] `UIScene::Destroy` · 005acf10 · legions 00578e70
- [ ] `UIScene::Cleanup` · 005acf50
- [ ] `UIScene::StreamPack` · 005acf60 · legions 00578ec0
- [ ] `UIScene::GetSubDataIDs` · 005ad010 · legions 00578f70

### PSmartArray<EntityDesc>
- [ ] `PSmartArray<EntityDesc>::StreamPack` · 005ad050 · legions 00578fb0

### UIScene
- [ ] `UIScene::UIScene` · 005ad100 · legions 00579060
- [ ] `UIScene::GetDBOType` · 005ad1c0
- [ ] `UIScene::~UIScene` · 005ad1d0 · legions 00579130

### SmartArray<EntityDesc>
- [ ] `SmartArray<EntityDesc>::AddToEnd` · 005ad270

### UIScene
- [ ] `UIScene::Allocate` · 005ad2c0
- [ ] `UIScene::AddEntity` · 005ad2f0 · legions 00579200
- [ ] `UIScene::AddEntity` · 005ad330 · legions 00579240
- [ ] `UIScene::Init` · 005ad410 · legions 00579320
- [-] `UIScene::FromFileNode` · 005ad450 — text source format (admin tools)
- [-] `UIScene::ToFileNode` · 005ad820 — text source format (admin tools)
- [ ] `UIScene::Render` · 005ad8e0

## `portal/client/src/render/ui/uiscenecameraentity.cpp.c` (0/3)

### UISceneCameraEntity
- [ ] `UISceneCameraEntity::UISceneCameraEntity` · 005d8d30 · legions 0059a460

### UIScenePhysObjEntity
- [ ] `UIScenePhysObjEntity::GetPhysObj` · 005d8d80 · legions 0059a4b0

### UISceneCameraEntity
- [ ] `UISceneCameraEntity::SetFrame` · 005d8d90 · legions 0059a500

## `portal/client/src/render/ui/uisceneentity.cpp.c` (2/6)

### UISceneEntity
- [~] `UISceneEntity::UISceneEntity` · 005d8ff0 · legions 0059a720 — pregame/arms.rs: frames and slerp; no instance id/scene
- [ ] `UISceneEntity::~UISceneEntity` · 005d90a0 · legions 0059a7d0
- [x] `UISceneEntity::Think` · 005d9120 · legions 0059a850 — ac2-ui pregame/arms.rs SceneEntity
- [x] `UISceneEntity::SLERPTo` · 005d9210 · legions 0059a940 — ac2-ui pregame/arms.rs SceneEntity
- [ ] `UISceneEntity::InitFromDesc` · 005d92e0 · legions 0059aa10
- [ ] `UISceneEntity::SetID` · 005d9330 · legions 0059aa60

## `portal/client/src/render/ui/uiscenelightentity.cpp.c` (0/3)

### UISceneLightEntity
- [ ] `UISceneLightEntity::UISceneLightEntity` · 005d8ed0 · legions 0059a640
- [ ] `UISceneLightEntity::InitFromDesc` · 005d8f10 · legions 0059a4c0
- [ ] `UISceneLightEntity::SetFrame` · 005d8f50 · legions 0059a680

## `portal/client/src/render/ui/uiscenemanager.cpp.c` (0/7)

### UISceneManager
- [ ] `UISceneManager::UseTime` · 005ae180 · legions 00579c20
- [ ] `UISceneManager::ReleaseUIScene` · 005ae1b0 · legions 00579c50
- [ ] `UISceneManager::Cleanup` · 005ae240 · legions 00579ce0
- [ ] `UISceneManager::FreeID` · 005ae290 · legions 00579d30
- [ ] `UISceneManager::Init` · 005ae2c0 · legions 00579d60
- [ ] `UISceneManager::GetUIScene` · 005ae370 · legions 00579e10
- [ ] `UISceneManager::GetFreeID` · 005ae420 · legions 00579ec0

## `portal/client/src/render/ui/uiscenephysobjentity.cpp.c` (0/13)

### UIScenePhysObjEntity
- [ ] `UIScenePhysObjEntity::InitFromDID` · 005adcd0 · legions 00579770
- [ ] `UIScenePhysObjEntity::InitFromVDesc` · 005add20 · legions 005797c0
- [ ] `UIScenePhysObjEntity::InitFromDesc` · 005add60 · legions 00579800
- [ ] `UIScenePhysObjEntity::Think` · 005addb0 · legions 00579850
- [ ] `UIScenePhysObjEntity::SetInstanceID` · 005ade20 · legions 005798c0
- [ ] `UIScenePhysObjEntity::ApplyVisualDesc` · 005ade60 · legions 00579900
- [ ] `UIScenePhysObjEntity::ApplyAdditiveVisualDesc` · 005ade80 · legions 00579920
- [ ] `UIScenePhysObjEntity::UIScenePhysObjEntity` · 005adec0 · legions 00579960

### UISceneEntity
- [ ] `UISceneEntity::SetFrame` · 005adf30 · legions 005799d0

### UIScenePhysObjEntity
- [ ] `UIScenePhysObjEntity::Animate` · 005adf40 · legions 005799e0
- [ ] `UIScenePhysObjEntity::~UIScenePhysObjEntity` · 005adf50 · legions 005799f0
- [ ] `UIScenePhysObjEntity::Render` · 005ae030 · legions 00579ad0
- [ ] `UIScenePhysObjEntity::AddChild` · 005ae130 · legions 00579bd0

## `portal/client/src/render/ui/uisurface.cpp.c` (0/9)

### UISurface
- [ ] `UISurface::GetBestWidthHeight` · 005b0ec0 · legions 0057c940

### RenderSurface
- [ ] `RenderSurface::SelectDirtyRectList` · 005b0f20 · legions 0057c9d0

### UISurface
- [ ] `UISurface::UISurface` · 005b0f40 · legions 0057c9f0
- [ ] `UISurface::SetupVertexBuffer` · 005b0f90 · legions 0057ca40
- [ ] `UISurface::DestroySurface` · 005b10a0 · legions 0057cb50
- [ ] `UISurface::CreateSurface` · 005b1140 · legions 0057cbf0
- [ ] `UISurface::PrepareSurface` · 005b1270 · legions 0057cd20
- [ ] `UISurface::Resize` · 005b1320 · legions 0057cdd0
- [ ] `UISurface::RefreshHardware` · 005b13b0 · legions 0057ce60

## `portal/client/src/render/ui/uisurfaceobject.cpp.c` (2/28)

### UISurfaceObject
- [ ] `UISurfaceObject::UISurfaceObject` · 005b23b0 · legions 00582870
- [ ] `UISurfaceObject::GetSurface` · 005b2500 · legions 005829c0
- [ ] `UISurfaceObject::GetMaterialOpacity` · 005b2510 · legions 005829d0
- [ ] `UISurfaceObject::GetObjectXPos` · 005b2520 · legions 005829e0
- [ ] `UISurfaceObject::GetObjectYPos` · 005b2530 · legions 005829f0
- [ ] `UISurfaceObject::GetObjectHeight` · 005b2540 · legions 00582a00
- [ ] `UISurfaceObject::GetObjectViewHeight` · 005b2550 · legions 00582a10
- [ ] `UISurfaceObject::GetRotateVals` · 005b2560 · legions 00582a20
- [ ] `UISurfaceObject::SetTransform` · 005b2590 · legions 00582a50
- [ ] `UISurfaceObject::SetSurface` · 005b25c0 · legions 00582a80
- [ ] `UISurfaceObject::RayTest` · 005b2610 · legions 00582ad0
- [ ] `UISurfaceObject::GetScreenLocalPosition` · 005b2630 · legions 00582af0
- [ ] `UISurfaceObject::ObjectToSurface` · 005b2680 · legions 00582b40
- [ ] `UISurfaceObject::ClipToSurface` · 005b26f0 · legions 00582bb0
- [ ] `UISurfaceObject::BuildClipMatrix` · 005b2790 · legions 00582c50
- [x] `UISurfaceObject::BuildRotateMatrix` · 005b2860 · legions 00582d20 — ac2-ui reticle.rs + ac2-gpu ui.rs: the targeting reticle
- [ ] `UISurfaceObject::BuildScreenMatrix` · 005b2930 · legions 00582df0
- [ ] `UISurfaceObject::SetScreenPosition` · 005b29f0 · legions 00582eb0
- [ ] `UISurfaceObject::ResizeSurface` · 005b2a70 · legions 00582f30
- [ ] `UISurfaceObject::SetScreenPosition` · 005b2ab0 · legions 00582f70
- [ ] `UISurfaceObject::SetMaterialColor` · 005b2b60 · legions 00583020
- [ ] `UISurfaceObject::SetMaterialOpacity` · 005b2bc0 · legions 00583080
- [ ] `UISurfaceObject::SetMaterialFiltering` · 005b2c30 · legions 005830f0
- [~] `UISurfaceObject::Render` · 005b2c80 · legions 00583140 — ac2-ui hud.rs + ac2-gpu ui.rs + ac2-client hud_view.rs: HUD roots made and drawn; no framework behaviour beyond hiding the toolbar's dismount/consignment buttons
- [ ] `UISurfaceObject::GetLocalPosition` · 005b2dc0 · legions 00583280
- [x] `UISurfaceObject::SetRotateVals` · 005b2e10 · legions 005832d0 — ac2-ui reticle.rs + ac2-gpu ui.rs: the targeting reticle
- [~] `UISurfaceObject::GenerateMaterial` · 005b2ea0 · legions 00583360 — ac2-ui hud.rs + ac2-gpu ui.rs + ac2-client hud_view.rs: HUD roots made and drawn; no framework behaviour beyond hiding the toolbar's dismount/consignment buttons
- [ ] `UISurfaceObject::SetMaterial` · 005b3000 · legions 005834c0

## `portal/client/src/render/ui/uiviewportobject.cpp.c` (0/3)

### UIViewportObject
- [ ] `UIViewportObject::UIViewportObject` · 00735550 · legions 006b1220
- [ ] `UIViewportObject::SetSceneID` · 00735590 · legions 006b1290
- [ ] `UIViewportObject::Render` · 007355a0 · legions 006b12a0

## `portal/client/src/uielement/desc/attributedesc.cpp.c` (1/6)

### AttributeDesc
- [ ] `AttributeDesc::GetSubDataIDs` · 005e10f0 · legions 005a1e80
- [ ] `AttributeDesc::AttributeDesc` · 005e1110 · legions 005a1ea0
- [ ] `AttributeDesc::~AttributeDesc` · 005e1130 · legions 005a1ec0
- [ ] `AttributeDesc::GetPackSize` · 005e1210
- [ ] `AttributeDesc::Pack` · 005e1280
- [x] `AttributeDesc::UnPack` · 005e1390 — ac2-ui layout.rs / layout_media.rs: LayoutDesc decode; all 14 Legions layouts decode to the last byte

## `portal/client/src/uielement/desc/elementdesc.cpp.c` (2/8)

### ElementDesc
- [x] `ElementDesc::GetReferentType` · 005dfaf0 · legions 005a0880 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [ ] `ElementDesc::GetPackSize` · 005dfb50 · legions 005a08e0
- [ ] `ElementDesc::Pack` · 005dfbd0 · legions 005a0960
- [ ] `ElementDesc::GetSubDataIDs` · 005dfd30 · legions 005a0ac0
- [ ] `ElementDesc::~ElementDesc` · 005dfde0 · legions 005a0b70
- [x] `ElementDesc::UnPack` · 005dff30 · legions 005a0cc0 — ac2-ui layout.rs / layout_media.rs: LayoutDesc decode; all 14 Legions layouts decode to the last byte
- [ ] `ElementDesc::Copy` · 005e0110 · legions 005a0ea0
- [ ] `ElementDesc::ElementDesc` · 005e0310 · legions 005a10a0

## `portal/client/src/uielement/desc/layoutdesc.cpp.c` (3/10)

### LayoutDesc
- [ ] `LayoutDesc::GetPackSize` · 005df410
- [ ] `LayoutDesc::Pack` · 005df4a0
- [ ] `LayoutDesc::GetSubDataIDs` · 005df590
- [ ] `LayoutDesc::~LayoutDesc` · 005df660 · legions 005a03f0
- [ ] `LayoutDesc::GetDBOType` · 005df770
- [x] `LayoutDesc::GetRootElementDesc` · 005df7a0 · legions 005a0530 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [x] `LayoutDesc::DereferenceElementDesc` · 005df7e0 · legions 005a0570 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [x] `LayoutDesc::UnPack` · 005df8a0 — ac2-ui layout.rs / layout_media.rs: LayoutDesc decode; all 14 Legions layouts decode to the last byte
- [ ] `LayoutDesc::LayoutDesc` · 005dfa80 · legions 005a0810
- [ ] `LayoutDesc::Allocate` · 005dfae0

## `portal/client/src/uielement/desc/media/md_data_alpha.cpp.c` (2/4)

### MD_Data_Alpha
- [ ] `MD_Data_Alpha::MD_Data_Alpha` · 005e2190 · legions 005a2f80
- [x] `MD_Data_Alpha::Update` · 005e21b0 — ac2-ui media.rs / transition.rs
- [ ] `MD_Data_Alpha::Copy` · 005e21d0 · legions 005a2ff0
- [x] `MD_Data_Alpha::StreamPack` · 005e2230 · legions 005a2770 — ac2-ui layout.rs / layout_media.rs: LayoutDesc decode; all 14 Legions layouts decode to the last byte

## `portal/client/src/uielement/desc/media/md_data_anim.cpp.c` (3/6)

### MD_Data_Anim
- [x] `MD_Data_Anim::Reset` · 005e22a0 — ac2-ui media.rs / transition.rs
- [ ] `MD_Data_Anim::GetSubDataIDs` · 005e22c0
- [x] `MD_Data_Anim::Update` · 005e2300 — ac2-ui media.rs / transition.rs
- [x] `MD_Data_Anim::StreamPack` · 005e23b0 · legions 005a3140 — ac2-ui layout.rs / layout_media.rs: LayoutDesc decode; all 14 Legions layouts decode to the last byte
- [ ] `MD_Data_Anim::MD_Data_Anim` · 005e2440 · legions 005a31d0
- [ ] `MD_Data_Anim::Copy` · 005e24d0 · legions 005a3260

## `portal/client/src/uielement/desc/media/md_data_avi.cpp.c` (1/6)

### MD_Data_AVI
- [-] `MD_Data_AVI::Reset` · 005e19c0 · legions 005a27e0 — no layout on the DVD has an AVI item
- [-] `MD_Data_AVI::Update` · 005e19f0 · legions 005a2810 — no layout on the DVD has an AVI item
- [ ] `MD_Data_AVI::MD_Data_AVI` · 005e1ab0 · legions 005a28d0
- [ ] `MD_Data_AVI::~MD_Data_AVI` · 005e1af0 · legions 005a2910
- [ ] `MD_Data_AVI::Copy` · 005e1b40 · legions 005a2960
- [x] `MD_Data_AVI::StreamPack` · 005e1bd0 · legions 005a29f0 — ac2-ui layout.rs / layout_media.rs: LayoutDesc decode; all 14 Legions layouts decode to the last byte

## `portal/client/src/uielement/desc/media/md_data_cursor.cpp.c` (1/3)

### MD_Data_Cursor
- [ ] `MD_Data_Cursor::MD_Data_Cursor` · 005e1930 · legions 005a26c0
- [x] `MD_Data_Cursor::Update` · 005e1950 — ac2-ui media.rs / transition.rs
- [ ] `MD_Data_Cursor::Copy` · 005e1980 · legions 005a2710

## `portal/client/src/uielement/desc/media/md_data_image.cpp.c` (2/5)

### MD_Data_Image
- [ ] `MD_Data_Image::MD_Data_Image` · 005e1f60 · legions 005a2d80

### MD_Data_Alpha
- [ ] `MD_Data_Alpha::GetSubDataIDs` · 005e1f90

### MD_Data_Image
- [x] `MD_Data_Image::Update` · 005e1fc0 — ac2-ui media.rs / transition.rs
- [ ] `MD_Data_Image::Copy` · 005e1fe0 · legions 005a2dd0
- [x] `MD_Data_Image::StreamPack` · 005e2020 · legions 005a2e10 — ac2-ui layout.rs / layout_media.rs: LayoutDesc decode; all 14 Legions layouts decode to the last byte

## `portal/client/src/uielement/desc/media/md_data_jump.cpp.c` (2/4)

### MD_Data_Jump
- [ ] `MD_Data_Jump::MD_Data_Jump` · 005e1da0 · legions 005a2c20
- [x] `MD_Data_Jump::Update` · 005e1dc0 — ac2-ui media.rs / transition.rs
- [ ] `MD_Data_Jump::Copy` · 005e1e10 · legions 005a2c90
- [x] `MD_Data_Jump::StreamPack` · 005e1e40 · legions 005a2bc0 — ac2-ui layout.rs / layout_media.rs: LayoutDesc decode; all 14 Legions layouts decode to the last byte

## `portal/client/src/uielement/desc/media/md_data_message.cpp.c` (1/3)

### MD_Data_Message
- [ ] `MD_Data_Message::MD_Data_Message` · 005e1d00 · legions 005a2b20
- [x] `MD_Data_Message::Update` · 005e1d20 — ac2-ui media.rs / transition.rs
- [ ] `MD_Data_Message::Copy` · 005e1d70 · legions 005a2b90

## `portal/client/src/uielement/desc/media/md_data_pause.cpp.c` (2/4)

### MD_Data_Pause
- [ ] `MD_Data_Pause::MD_Data_Pause` · 005e20a0 · legions 005a2e90
- [x] `MD_Data_Pause::Reset` · 005e20d0 — ac2-ui media.rs / transition.rs
- [x] `MD_Data_Pause::Update` · 005e20f0 — ac2-ui media.rs / transition.rs
- [ ] `MD_Data_Pause::Copy` · 005e2160 · legions 005a2f50

## `portal/client/src/uielement/desc/media/md_data_sound.cpp.c` (0/3)

### MD_Data_Sound
- [ ] `MD_Data_Sound::MD_Data_Sound` · 005e1ea0 · legions 005a2cc0
- [~] `MD_Data_Sound::Update` · 005e1ec0 — fades only (no rotate); sounds queued for an audio sink; cursor image not drawn by the client yet
- [ ] `MD_Data_Sound::Copy` · 005e1f30 · legions 005a2d50

## `portal/client/src/uielement/desc/media/md_data_state.cpp.c` (2/4)

### MD_Data_State
- [ ] `MD_Data_State::MD_Data_State` · 005e1c40 · legions 005a2a60
- [x] `MD_Data_State::Update` · 005e1c60 — ac2-ui media.rs / transition.rs
- [ ] `MD_Data_State::Copy` · 005e1c80 · legions 005a2aa0

### MD_Data_Sound
- [x] `MD_Data_Sound::StreamPack` · 005e1cb0 · legions 005a2ad0 — ac2-ui layout.rs / layout_media.rs: LayoutDesc decode; all 14 Legions layouts decode to the last byte

## `portal/client/src/uielement/desc/mediadesc.cpp.c` (2/4)

### MediaDesc
- [x] `MediaDesc::CreateMediaType` · 005e1700 · legions 005a2490 — ac2-ui layout.rs / layout_media.rs: LayoutDesc decode; all 14 Legions layouts decode to the last byte
- [ ] `MediaDesc::MediaDesc` · 005e1830 · legions 005a25c0
- [x] `MediaDesc::StreamPack` · 005e1850 · legions 005a25e0 — ac2-ui layout.rs / layout_media.rs: LayoutDesc decode; all 14 Legions layouts decode to the last byte
- [ ] `MediaDesc::~MediaDesc` · 005e1880 · legions 005a2610

## `portal/client/src/uielement/desc/statedesc.cpp.c` (1/7)

### StateDesc
- [ ] `StateDesc::GetPackSize` · 005e0380 · legions 005a1110
- [ ] `StateDesc::Pack` · 005e04a0 · legions 005a1230
- [ ] `StateDesc::GetSubDataIDs` · 005e0690 · legions 005a1420
- [ ] `StateDesc::~StateDesc` · 005e0750 · legions 005a14e0
- [x] `StateDesc::UnPack` · 005e0950 · legions 005a16e0 — ac2-ui layout.rs / layout_media.rs: LayoutDesc decode; all 14 Legions layouts decode to the last byte
- [ ] `StateDesc::Copy` · 005e0cd0 · legions 005a1a60
- [ ] `StateDesc::StateDesc` · 005e1060 · legions 005a1df0

## `portal/client/src/uielement/desc/transitiondesc.cpp.c` (1/2)

### TransitionDesc
- [ ] `TransitionDesc::TransitionDesc` · 005e1490 · legions 005a2220
- [x] `TransitionDesc::StreamPack` · 005e14b0 · legions 005a2240 — ac2-ui layout.rs / layout_media.rs: LayoutDesc decode; all 14 Legions layouts decode to the last byte

## `portal/client/src/uielement/file/filetree.cpp.c` (0/1)

### FileTree
- [ ] `FileTree::~FileTree` · 005fd250 · legions 005bdd80

## `portal/client/src/uielement/framework/gameplay/gameplayui.cpp.c` (0/2)

### GamePlayUI
- [ ] `GamePlayUI::GamePlayUI` · 005fd9c0 · legions 005be4f0
- [ ] `GamePlayUI::~GamePlayUI` · 005fda00 · legions 005be530

## `portal/client/src/uielement/framework/uichildframework.cpp.c` (1/6)

### UIChildFramework
- [ ] `UIChildFramework::UIChildFramework` · 005fe110 · legions 005bec10
- [ ] `UIChildFramework::~UIChildFramework` · 005fe140 · legions 005bec40
- [x] `UIChildFramework::CreateAndAddRootElement` · 005fe170 · legions 005bec70 — ac2-ui hud.rs
- [ ] `UIChildFramework::RemoveRootElement` · 005fe190 · legions 005bec90
- [ ] `UIChildFramework::GetUIFlow` · 005fe1a0 · legions 005beca0
- [ ] `UIChildFramework::Shown` · 005fe1b0 · legions 005becb0

## `portal/client/src/uielement/framework/uiflow.cpp.c` (4/8)

### UIFlow
- [ ] `UIFlow::GetUIFlowPtr` · 005fd2c0 · legions 005bddf0
- [ ] `UIFlow::Update` · 005fd2d0
- [~] `UIFlow::UIFlow` · 005fd320 · legions 005bde50 — pregame/ui.rs: character selection and disconnected made; intro, data patch (the client connects), keyboard, credits not ported
- [ ] `UIFlow::~UIFlow` · 005fd410 · legions 005bdf40
- [x] `UIFlow::QueueUIMode` · 005fd4c0 · legions 005bdff0 — ac2-ui pregame/flow.rs, pregame/ui.rs
- [x] `UIFlow::RegisterFrameworkClass` · 005fd5d0 · legions 005be100 — ac2-ui pregame/flow.rs, pregame/ui.rs
- [x] `UIFlow::UseNewMode` · 005fd620 · legions 005be150 — ac2-ui pregame/flow.rs, pregame/ui.rs
- [x] `UIFlow::ListenToMessage` · 005fd720 · legions 005be250 — ac2-ui pregame/flow.rs, pregame/ui.rs

## `portal/client/src/uielement/framework/uiframework.cpp.c` (0/9)

### UIFramework
- [ ] `UIFramework::GetUIPersistantData` · 005fd7d0 · legions 005be300
- [ ] `UIFramework::CreateElement` · 005fd7f0 · legions 005be320
- [ ] `UIFramework::UIFramework` · 005fd810 · legions 005be340
- [ ] `UIFramework::~UIFramework` · 005fd840 · legions 005be370
- [ ] `UIFramework::Show` · 005fd860 · legions 005be390
- [ ] `UIFramework::FindChild` · 005fd8a0 · legions 005be3d0
- [ ] `UIFramework::LocationApplied` · 005fd8d0 · legions 005be400
- [ ] `UIFramework::RemoveChild` · 005fd940 · legions 005be470
- [ ] `UIFramework::AddChild` · 005fd970 · legions 005be4a0

## `portal/client/src/uielement/framework/uimainframework.cpp.c` (0/11)

### UIMainFramework
- [ ] `UIMainFramework::MakeNoSurface` · 005fda70 · legions 005be5a0
- [ ] `UIMainFramework::MakeSurfaceObject` · 005fdaa0 · legions 005be5d0
- [ ] `UIMainFramework::GetUIFlow` · 005fdba0 · legions 005be6d0
- [ ] `UIMainFramework::InqLocations` · 005fdbb0 · legions 005be6e0
- [ ] `UIMainFramework::ApplyLocations` · 005fdc40 · legions 005be770
- [ ] `UIMainFramework::DeleteUISurfaceList` · 005fdd40 · legions 005be870
- [ ] `UIMainFramework::RemoveRootElement` · 005fddc0 · legions 005be8f0
- [ ] `UIMainFramework::UIMainFramework` · 005fde50 · legions 005be980
- [ ] `UIMainFramework::~UIMainFramework` · 005fde90 · legions 005be9c0
- [~] `UIMainFramework::CreateAndAddRootElementByDataID` · 005fded0 · legions 005bea00 — ac2-ui: no transitions, timed media, mouse, listeners or dirty rects; A8R8G8B8 blits only; HUD roots from SetupChildren's radar/selection/statusbar/toolbar
- [~] `UIMainFramework::CreateAndAddRootElement` · 005fe0b0 · legions 005bebe0 — ac2-ui: no transitions, timed media, mouse, listeners or dirty rects; A8R8G8B8 blits only; HUD roots from SetupChildren's radar/selection/statusbar/toolbar

## `portal/client/src/uielement/framework/uipersistantdata.cpp.c` (9/13)

### UIPersistantData
- [x] `UIPersistantData::SetCharSet` · 005fe220 — ac2-ui pregame/persistent.rs (plain data)
- [x] `UIPersistantData::InqCurrentCGR` · 005fe250 — ac2-ui pregame/persistent.rs (plain data)
- [x] `UIPersistantData::ClearGenerationResultTable` · 005fe2b0 · legions 005bedb0 — ac2-ui pregame/persistent.rs (plain data)
- [x] `UIPersistantData::~UIPersistantData` · 005fe380 · legions 005bee80 — ac2-ui pregame/persistent.rs (plain data)

### DirectXPlayingSound
- [ ] `DirectXPlayingSound::GetIsInWorld` · 005fe400

### UIPersistantData
- [x] `UIPersistantData::InCharGen` · 005fe410 — ac2-ui pregame/persistent.rs (plain data)

### CameraManager
- [x] `CameraManager::GetFirstPersonState` · 005fe420 · legions 005bef00

### UIPersistantData
- [ ] `UIPersistantData::SetPlayingSoundContext` · 005fe430

### CPhysicsObj
- [ ] `CPhysicsObj::GetScale` · 005fe440

### UIPersistantData
- [x] `UIPersistantData::SetCharGenResult` · 005fe450 — ac2-ui pregame/persistent.rs (plain data)
- [x] `UIPersistantData::InqCharGenResult` · 005fe470 · legions 005bef50 — ac2-ui pregame/persistent.rs (plain data)
- [ ] `UIPersistantData::InqGenerationResultTable` · 005fe4c0
- [x] `UIPersistantData::UIPersistantData` · 005fe640 · legions 005bf120 — ac2-ui pregame/persistent.rs (plain data)

## `portal/client/src/uielement/prim/node.cpp.c` (7/7)

### UIElement_Node
- [x] `UIElement_Node::GetToken` · 00735760 · legions 006b1460 — ac2-ui tree_view.rs
- [x] `UIElement_Node::GetConnection` · 00735770 · legions 006b1470 — ac2-ui tree_view.rs
- [x] `UIElement_Node::GetChildLines` · 007358c0 · legions 006b1560 — ac2-ui tree_view.rs
- [x] `UIElement_Node::UIElement_Node` · 00735a60 · legions 006b1700 — ac2-ui tree_view.rs
- [x] `UIElement_Node::~UIElement_Node` · 00735a90 · legions 006b1730 — ac2-ui tree_view.rs
- [x] `UIElement_Node::AddParent` · 00735c00 · legions 006b18a0 — ac2-ui tree_view.rs
- [x] `UIElement_Node::AddChild` · 00735c30 · legions 006b18d0 — ac2-ui tree_view.rs

## `portal/client/src/uielement/prim/uiregion.cpp.c` (5/31)

### UIRegion
- [ ] `UIRegion::SetObject` · 005e2540 · legions 005a32d0
- [ ] `UIRegion::SetAlphaImage` · 005e2580 · legions 005a3310
- [ ] `UIRegion::ClearAlphaImage` · 005e2590 · legions 005a3320

### CMotionInterp
- [ ] `CMotionInterp::SetAccelerationScale` · 005e25b0 · legions 005a3340

### UIRegion
- [~] `UIRegion::NotifyMouseMove` · 005e25c0 · legions 005a3350 — ac2-ui mouse.rs / button.rs: no tooltips, cursors, drag and drop, moving/resizing, activation, hot clicks or listener registry (messages are taken by the caller)
- [ ] `UIRegion::GetObjectA` · 005e26d0 · legions 005a3460
- [ ] `UIRegion::GetObjectX0` · 005e26f0 · legions 005a3480
- [ ] `UIRegion::GetObjectY0` · 005e2720 · legions 005a34b0
- [ ] `UIRegion::GetScreenX0` · 005e2750 · legions 005a34e0
- [ ] `UIRegion::GetScreenY0` · 005e2790 · legions 005a3520
- [ ] `UIRegion::GetObjectBox` · 005e27d0 · legions 005a3560
- [ ] `UIRegion::GetSurface` · 005e2860 · legions 005a3620
- [ ] `UIRegion::GetSurfaceObjAbsX` · 005e2890 · legions 005a3650
- [ ] `UIRegion::GetSurfaceObjAbsY` · 005e28c0 · legions 005a3680
- [ ] `UIRegion::MakeRootDirtyHere` · 005e28f0 · legions 005a36b0
- [ ] `UIRegion::ForceDraw` · 005e2930 · legions 005a36f0
- [ ] `UIRegion::SetParent_Internal` · 005e2980 · legions 005a3740
- [x] `UIRegion::GetClipBox` · 005e2a30 · legions 005a37f0 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [ ] `UIRegion::GetScreenClipBox` · 005e2b50 · legions 005a3910
- [ ] `UIRegion::SetVisible` · 005e2c30 · legions 005a39f0
- [ ] `UIRegion::SetImage` · 005e2cc0 · legions 005a3a80
- [ ] `UIRegion::ClearImage` · 005e2d30 · legions 005a3af0
- [x] `UIRegion::MoveTo` · 005e2da0 · legions 005a3b60 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [x] `UIRegion::ResizeTo` · 005e2e70 · legions 005a3c30 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [~] `UIRegion::DrawHere` · 005e2f50 · legions 005a3d10 — A8R8G8B8 only; colour key and format conversion not modelled
- [ ] `UIRegion::~UIRegion` · 005e3140 · legions 005a3f00
- [ ] `UIRegion::MouseOver` · 005e31e0 · legions 005a3fa0
- [ ] `UIRegion::MouseOverTop` · 005e31f0 · legions 005a3fb0
- [x] `UIRegion::NotifyMouseDown` · 005e3220 — ac2-ui mouse.rs / button.rs: UI hit tests, presses and buttons
- [x] `UIRegion::NotifyMouseUp` · 005e3310 · legions 005a40d0 — ac2-ui mouse.rs / button.rs: UI hit tests, presses and buttons
- [ ] `UIRegion::UIRegion` · 005e33c0 · legions 005a4180

## `portal/client/src/uielement/system/uielement.cpp.c` (32/104)

### UIRectangleObject
- [ ] `UIRectangleObject::SetColor` · 005e8b30 · legions 005a9790

### UIElement
- [ ] `UIElement::GetType` · 005e8b60 · legions 005a97c0

### UISurfaceObject
- [ ] `UISurfaceObject::GetObjectWidth` · 005e8b70 · legions 005a97d0

### UIElement
- [ ] `UIElement::GetParent` · 005e8b80 · legions 005a97e0
- [ ] `UIElement::GetID` · 005e8b90 · legions 005a97f0
- [x] `UIElement::SetZLevel` · 005e8ba0 · legions 005a9800 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor

### UISurfaceObject
- [ ] `UISurfaceObject::GetObjectViewWidth` · 005e8c20 · legions 005a9880

### UIElement
- [ ] `UIElement::Initialized` · 005e8c30 · legions 005a9890
- [ ] `UIElement::GetRootElement` · 005e8c40 · legions 005a98a0
- [x] `UIElement::MouseMove` · 005e8c60 · legions 005a98c0 — ac2-ui mouse.rs / button.rs: UI hit tests, presses and buttons
- [x] `UIElement::MouseUp` · 005e8c90 · legions 005a98f0 — ac2-ui mouse.rs / button.rs: UI hit tests, presses and buttons
- [ ] `UIElement::StartMovement` · 005e8ce0 · legions 005a9940
- [ ] `UIElement::StopMovement` · 005e8d40 · legions 005a99a0
- [ ] `UIElement::MouseMoveElement` · 005e8d60 · legions 005a99c0
- [ ] `UIElement::GetChild` · 005e8e00 · legions 005a9a60
- [x] `UIElement::GetChildRecursive` · 005e8e30 · legions 005a9a90 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [x] `UIElement::GetStringTableID` · 005e8e70 · legions 005a9ad0 — ac2-ui text.rs / text_attr.rs: UIElement_Text layout and drawing
- [x] `UIElement::SetMediaIndex` · 005e8e80 · legions 005a9ae0 — ac2-ui media.rs / transition.rs
- [x] `UIElement::AddChild` · 005e8e90 · legions 005a9af0 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [ ] `UIElement::SetAlphaImage` · 005e8f30 · legions 005a9b90
- [x] `UIElement::SetCursor` · 005e8f70 · legions 005a9bd0 — ac2-ui media.rs / transition.rs
- [ ] `UIElement::RestoreSavedCursor` · 005e8fd0 · legions 005a9c30
- [ ] `UIElement::SaveCursor` · 005e9010 · legions 005a9c70
- [x] `UIElement::MouseOver` · 005e9040 · legions 005a9ca0 — ac2-ui mouse.rs / button.rs: UI hit tests, presses and buttons
- [x] `UIElement::MouseOverTop` · 005e9060 · legions 005a9cc0 — ac2-ui media.rs / transition.rs
- [x] `UIElement::SetAttributeFromDesc` · 005e90b0 · legions 005a9d10 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [ ] `UIElement::CopyElem` · 005e9180 · legions 005a9de0
- [ ] `UIElement::MatchGraphics` · 005e91b0 · legions 005a9e10
- [ ] `UIElement::DragAndDropComplete` · 005e9250 · legions 005a9eb0
- [ ] `UIElement::SetDefaultLocation` · 005e92b0 · legions 005a9f10
- [ ] `UIElement::InqImageSize` · 005e92c0 · legions 005a9f20
- [ ] `UIElement::SetTooltip` · 005e9300 · legions 005a9f60
- [x] `UIElement::GetParentCurrentPosition` · 005e9340 · legions 005a9fa0 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [ ] `UIElement::UpdateActivatable` · 005e93e0 · legions 005aa040
- [ ] `UIElement::UpdateActivateOrder` · 005e9420 · legions 005aa080
- [x] `UIElement::Activate` · 005e9470 · legions 005aa0d0 — ac2-ui input.rs / text_edit.rs: typing into text elements
- [x] `UIElement::Deactivate` · 005e94c0 · legions 005aa120 — ac2-ui input.rs / text_edit.rs: typing into text elements

### UIRegion
- [ ] `UIRegion::GetScreenX1` · 005e94f0 · legions 005aa150
- [ ] `UIRegion::GetScreenY1` · 005e9510 · legions 005aa170

### UIElement
- [~] `UIElement::UIElement` · 005e9530 · legions 005aa190 — ac2-ui: no transitions, timed media, mouse, listeners or dirty rects; A8R8G8B8 blits only; HUD roots from SetupChildren's radar/selection/statusbar/toolbar
- [x] `UIElement::HasCursor` · 005e9710 · legions 005aa370 — ac2-ui media.rs / transition.rs
- [x] `UIElement::SetParent` · 005e9720 · legions 005aa380 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [~] `UIElement::BroadcastMessage` · 005e9790 · legions 005aa3f0 — ac2-ui mouse.rs / button.rs: no tooltips, cursors, drag and drop, moving/resizing, activation, hot clicks or listener registry (messages are taken by the caller)
- [x] `UIElement::UpdateMedia` · 005e97c0 · legions 005aa420 — ac2-ui media.rs / transition.rs
- [ ] `UIElement::ClearTooltip` · 005e9810 · legions 005aa470

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PStringBase<char>_*>_*>
- [ ] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PStringBase<char>_*>_*>::~IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PStringBase<char>_*>_*>` · 005e9860 · legions 005aa4c0

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PStringBase<unsigned_short>_>_*>
- [ ] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PStringBase<unsigned_short>_>_*>::~IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PStringBase<unsigned_short>_>_*>` · 005e9890 · legions 005aa4f0

### UIElement
- [~] `UIElement::StopTransition` · 005e9940 · legions 005aa5a0 — m_delOnTransStop (delete after the exit transition) not modelled
- [x] `UIElement::StopAllTransitions` · 005e99e0 · legions 005aa640 — ac2-ui media.rs / transition.rs
- [ ] `UIElement::SetIconDesc` · 005e9ab0 · legions 005aa710
- [~] `UIElement::UpdateTransitions` · 005e9b80 · legions 005aa7e0 — fades only (no rotate); sounds queued for an audio sink; cursor image not drawn by the client yet
- [ ] `UIElement::GetAttribute_ID` · 005e9f00 · legions 005aaad0
- [ ] `UIElement::GetAttribute_InstanceID` · 005e9f60 · legions 005aab30
- [ ] `UIElement::GetAttribute_StringID` · 005e9fa0 · legions 005aab70
- [ ] `UIElement::GetAttribute_Int` · 005ea000 · legions 005aabd0
- [ ] `UIElement::GetAttribute_Float` · 005ea060 · legions 005aac30
- [ ] `UIElement::GetAttribute_Bool` · 005ea0c0 · legions 005aac90
- [ ] `UIElement::GetAttribute_DataID` · 005ea120 · legions 005aacf0
- [ ] `UIElement::GetAttribute_Unicode` · 005ea180 · legions 005aad50
- [x] `UIElement::MouseDown` · 005ea220 · legions 005aadf0 — ac2-ui mouse.rs / button.rs: UI hit tests, presses and buttons
- [x] `UIElement::ResetMedia` · 005ea2c0 · legions 005aae90 — ac2-ui media.rs / transition.rs
- [x] `UIElement::SetImage` · 005ea430 · legions 005ab000 — ac2-ui compose.rs / element.rs set_image
- [x] `UIElement::AddAttributesFromDesc` · 005ea510 · legions 005ab0e0 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [ ] `UIElement::MatchAttributes` · 005ea5c0 · legions 005ab190
- [ ] `UIElement::DragItem` · 005eab10 · legions 005ab6e0
- [ ] `UIElement::CatchDroppedItem` · 005eabd0 · legions 005ab7a0
- [ ] `UIElement::IsTooltipOn` · 005eac90 · legions 005ab860
- [ ] `UIElement::IsActivatable` · 005eacb0 · legions 005ab880
- [ ] `UIElement::GetActivateOrder` · 005eacd0 · legions 005ab8a0
- [~] `UIElement::MoveTo` · 005eacf0 · legions 005ab8c0 — ac2-ui: no transitions, timed media, mouse, listeners or dirty rects; A8R8G8B8 blits only; HUD roots from SetupChildren's radar/selection/statusbar/toolbar
- [x] `UIElement::GetPositionFromState` · 005eae20 · legions 005ab9f0 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [~] `UIElement::SetVisible` · 005eafc0 · legions 005abb90 — ac2-ui: no transitions, timed media, mouse, listeners or dirty rects; A8R8G8B8 blits only; HUD roots from SetupChildren's radar/selection/statusbar/toolbar
- [x] `UIElement::StartTransition` · 005eb100 · legions 005abcd0 — ac2-ui media.rs / transition.rs
- [ ] `UIElement::StartResizing` · 005eb1f0 · legions 005abdc0
- [ ] `UIElement::StopResizing` · 005eb370 · legions 005abf40
- [ ] `UIElement::MouseResizeElement` · 005eb430 · legions 005ac000
- [ ] `UIElement::AddImageToState` · 005eb9c0 · legions 005ac590
- [x] `UIElement::AddAttributesForState` · 005ebaf0 · legions 005ac6c0 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [ ] `UIElement::InqLocation` · 005ebb80 · legions 005ac750
- [ ] `UIElement::CheckLocation` · 005ebc80 · legions 005ac850
- [ ] `UIElement::ApplyLocation` · 005ebcb0 · legions 005ac880
- [ ] `UIElement::DrawStart` · 005ebdd0 · legions 005ac9a0
- [ ] `UIElement::DrawDone` · 005ebe10 · legions 005ac9e0
- [x] `UIElement::GetPositionFromDesc` · 005ebe50 · legions 005aca20 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [ ] `UIElement::~UIElement` · 005ec020 · legions 005acbf0
- [ ] `UIElement::SetAttribute_ID` · 005ec240 · legions 005ace10
- [ ] `UIElement::SetAttribute_InstanceID` · 005ec2e0 · legions 005aceb0
- [ ] `UIElement::SetAttribute_StringID` · 005ec380 · legions 005acf50
- [ ] `UIElement::SetAttribute_Int` · 005ec420 · legions 005acff0
- [ ] `UIElement::SetAttribute_Float` · 005ec4d0 · legions 005ad0a0
- [~] `UIElement::SetAttribute_Bool` · 005ec570 · legions 005ad140 — ac2-ui: no transitions, timed media, mouse, listeners or dirty rects; A8R8G8B8 blits only; HUD roots from SetupChildren's radar/selection/statusbar/toolbar
- [ ] `UIElement::SetAttribute_DataID` · 005ec660 · legions 005ad230
- [ ] `UIElement::SetAttribute_ASCII` · 005ec700 · legions 005ad2d0
- [ ] `UIElement::SetAttribute_Unicode` · 005ec7a0 · legions 005ad370
- [x] `UIElement::GetOriginalPosition` · 005ec870 · legions 005ad440 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [x] `UIElement::GetParentOriginalPosition` · 005ec8a0 · legions 005ad470 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [x] `UIElement::ListenToMessage` · 005ec9d0 · legions 005ad5a0 — ac2-ui media.rs / transition.rs
- [x] `UIElement::UpdateSizeAndPosition` · 005eca20 · legions 005ad5f0 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [~] `UIElement::Update_Internal` · 005ecd60 · legions 005ad930 — ac2-ui: no transitions, timed media, mouse, listeners or dirty rects; A8R8G8B8 blits only; HUD roots from SetupChildren's radar/selection/statusbar/toolbar
- [~] `UIElement::ResizeTo` · 005ed150 · legions 005add20 — ac2-ui: no transitions, timed media, mouse, listeners or dirty rects; A8R8G8B8 blits only; HUD roots from SetupChildren's radar/selection/statusbar/toolbar
- [x] `UIElement::SwitchDescription` · 005ed2c0 · legions 005ade90 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [ ] `UIElement::Update` · 005ed330 · legions 005adf00
- [x] `UIElement::UpdateChild` · 005ed340 · legions 005adf10 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [~] `UIElement::SetState` · 005ed450 · legions 005ae020 — m_delOnTransStop (delete after the exit transition) not modelled

## `portal/client/src/uielement/system/uielementmanager.cpp.c` (2/69)

### Box2D
- [ ] `Box2D::Includes` · 005e3490 · legions 005a4250

### UIElementManager
- [ ] `UIElementManager::CreateRootElement` · 005e3520
- [ ] `UIElementManager::RemoveRootElement` · 005e3580 · legions 005a4320
- [~] `UIElementManager::SetCursor` · 005e35f0 — fades only (no rotate); sounds queued for an audio sink; cursor image not drawn by the client yet
- [x] `UIElementManager::UseDefaultCursor` · 005e3690 · legions 005a4420 — ac2-ui media.rs / transition.rs
- [ ] `UIElementManager::KeyReleaseEvent` · 005e36b0
- [ ] `UIElementManager::ClearTooltip` · 005e3740 · legions 005a4570
- [ ] `UIElementManager::CheckDragandDrop` · 005e37c0 · legions 005a45f0
- [ ] `UIElementManager::ClearDragandDrop` · 005e3820 · legions 005a4650
- [~] `UIElementManager::ActivationAlert` · 005e38b0 · legions 005a46e0 — ac2-ui text_edit.rs: no selection, clipboard, IME, page up/down, cursor flashing or view scrolling

### SmartArray<CPartGroup_*>
- [ ] `SmartArray<CPartGroup_*>::RemoveOrdered` · 005e3910 · legions 005a4740

### UIElementManager
- [ ] `UIElementManager::AddRootElement` · 005e3960
- [ ] `UIElementManager::ChangingElementVisibility` · 005e3aa0 · legions 005a48d0
- [~] `UIElementManager::MouseMoveEvent` · 005e3af0 · legions 005a4920 — ac2-ui mouse.rs / button.rs: no tooltips, cursors, drag and drop, moving/resizing, activation, hot clicks or listener registry (messages are taken by the caller)
- [~] `UIElementManager::UpdateMouse` · 005e3b70 · legions 005a49a0 — ac2-ui mouse.rs / button.rs: no tooltips, cursors, drag and drop, moving/resizing, activation, hot clicks or listener registry (messages are taken by the caller)
- [~] `UIElementManager::MouseDownEvent` · 005e3cc0 — ac2-ui text_edit.rs: no selection, clipboard, IME, page up/down, cursor flashing or view scrolling
- [ ] `UIElementManager::CleanDeleteQueue` · 005e3de0 · legions 005a4c10
- [ ] `UIElementManager::StartTooltip` · 005e3e90 · legions 005a4cc0
- [ ] `UIElementManager::StartDragandDrop` · 005e42e0
- [ ] `UIElementManager::StopDragandDrop` · 005e46d0 · legions 005a5500
- [ ] `UIElementManager::UnregisterActivatable` · 005e4720 · legions 005a5550
- [ ] `UIElementManager::ActivateNext` · 005e4740
- [ ] `UIElementManager::DrawDirtyRegions` · 005e4970
- [ ] `UIElementManager::GetObjectRegionInfo` · 005e4b10
- [~] `UIElementManager::MouseUpEvent` · 005e4b60 — ac2-ui mouse.rs / button.rs: no tooltips, cursors, drag and drop, moving/resizing, activation, hot clicks or listener registry (messages are taken by the caller)
- [~] `UIElementManager::KeyPressEvent` · 005e4c70 · legions 005a4440 — ac2-ui mouse.rs / button.rs: no tooltips, cursors, drag and drop, moving/resizing, activation, hot clicks or listener registry (messages are taken by the caller)
- [ ] `UIElementManager::ResetTooltip` · 005e4d20 · legions 005a5bb0
- [ ] `UIElementManager::AddtoListCleanQue` · 005e4d30 · legions 005a5bc0
- [ ] `UIElementManager::HandleApply` · 005e4d70 · legions 005a5c00
- [~] `UIElementManager::StartInput` · 005e4e50 — ac2-ui mouse.rs / button.rs: no tooltips, cursors, drag and drop, moving/resizing, activation, hot clicks or listener registry (messages are taken by the caller)
- [~] `UIElementManager::StopInput` · 005e4fc0 — ac2-ui mouse.rs / button.rs: no tooltips, cursors, drag and drop, moving/resizing, activation, hot clicks or listener registry (messages are taken by the caller)
- [ ] `UIElementManager::RegisterActivatable` · 005e50d0 · legions 005a5df0

### Box2D
- [ ] `Box2D::Subtract` · 005e5150 · legions 005a5e70

### UIElementManager
- [ ] `UIElementManager::MakeObjectTopmost` · 005e5260

### UIObjectRegionInfo
- [ ] `UIObjectRegionInfo::AddDirtyRect` · 005e53a0 · legions 005a60c0

### UIElementManager
- [ ] `UIElementManager::MakeRootDirtyHere` · 005e5510
- [ ] `UIElementManager::ReleaseLayoutDesc` · 005e5830 · legions 005a6580
- [ ] `UIElementManager::RefreshEvent` · 005e5870 · legions 005a65c0
- [ ] `UIElementManager::AddElementToDeleteQueue` · 005e5950 · legions 005a66a0
- [~] `UIElementManager::CreateRootElementByDataID` · 005e59b0 — ac2-ui: no transitions, timed media, mouse, listeners or dirty rects; A8R8G8B8 blits only; HUD roots from SetupChildren's radar/selection/statusbar/toolbar
- [~] `UIElementManager::CreateElement` · 005e5a80 — ac2-ui: no transitions, timed media, mouse, listeners or dirty rects; A8R8G8B8 blits only; HUD roots from SetupChildren's radar/selection/statusbar/toolbar
- [ ] `UIElementManager::RemoveAndDeleteRootElement` · 005e5bf0 · legions 005a6930
- [ ] `UIElementManager::UnregisterForMessage` · 005e5c30 · legions 005a6970
- [ ] `UIElementManager::CleanElementListCleanQue` · 005e5cb0 · legions 005a69f0
- [ ] `UIElementManager::RegisterForObjectMessage` · 005e5ea0
- [ ] `UIElementManager::UnregisterForObjectMessage` · 005e5fd0 · legions 005a6d10
- [ ] `UIElementManager::DeleteListenerFromElementCleanQue` · 005e6050 · legions 005a6d90
- [ ] `UIElementManager::DeleteListenerFromCleanQue` · 005e6360 · legions 005a70a0
- [ ] `UIElementManager::CleanObjectListCleanQue` · 005e65d0 · legions 005a7310
- [ ] `UIElementManager::RegisterForGlobalMessage` · 005e6720 · legions 005a7460
- [ ] `UIElementManager::UnregisterForGlobalMessage` · 005e6800 · legions 005a7540
- [ ] `UIElementManager::CleanGlobalListCleanQue` · 005e6870 · legions 005a75b0
- [ ] `UIElementManager::GetElement` · 005e69f0 · legions 005a7730
- [ ] `UIElementManager::GetUniqueMatchElement` · 005e6a40 · legions 005a7780
- [ ] `UIElementManager::DeletingElement` · 005e6ab0 · legions 005a77f0
- [ ] `UIElementManager::RegisterElementClass` · 005e6bb0

### (free functions)
- [ ] `RefreshEvent_g` · 005e6be0

### UIElementManager
- [ ] `UIElementManager::DeleteUIObjectRegionInfo` · 005e6c00
- [ ] `UIElementManager::Init` · 005e6f70
- [ ] `UIElementManager::BroadcastMessage` · 005e7130
- [ ] `UIElementManager::BroadcastGlobalMessage` · 005e7280
- [ ] `UIElementManager::UIElementManager` · 005e7370 · legions 005a8000

### MoveToManager
- [ ] `MoveToManager::ReachedPersistentTarget` · 005e7590 · legions 005a8230
- [x] `MoveToManager::IsMovingTo` · 005e75a0 — ac2-physics moveto (slice 3)

### UIElementManager
- [ ] `UIElementManager::UseTime` · 005e75b0 · legions 005a8240
- [ ] `UIElementManager::Cleanup` · 005e7780 · legions 005a83e0
- [ ] `UIElementManager::RegisterForMessage` · 005e7b30
- [ ] `UIElementManager::~UIElementManager` · 005e7cd0 · legions 005a8930
- [ ] `UIElementManager::CreateUIObjectRegionInfo` · 005e7e70 · legions 005a8ad0

## `portal/client/src/uielement/system/uilistener.cpp.c` (0/12)

### UIListener
- [ ] `UIListener::GetListenerElementNumberRegistered` · 005e7f80 · legions 005a8be0
- [ ] `UIListener::GetListenerNumberRegistered` · 005e7fd0 · legions 005a8c30
- [ ] `UIListener::UnregisterForAllGlobalMessages` · 005e8010 · legions 005a8c70
- [ ] `UIListener::IncrementMessage` · 005e8110 · legions 005a8d70
- [ ] `UIListener::DecrementMessage` · 005e8180 · legions 005a8de0
- [ ] `UIListener::UnregisterForAllMessages` · 005e81f0 · legions 005a8e50
- [ ] `UIListener::DecrementElementMessage` · 005e8340 · legions 005a8fa0
- [ ] `UIListener::UIListener` · 005e8510 · legions 005a9170
- [ ] `UIListener::~UIListener` · 005e8560 · legions 005a91c0
- [ ] `UIListener::IncrementElementMessage` · 005e89b0 · legions 005a9610
- [ ] `UIListener::RegisterForMessage` · 005e8a60 · legions 005a96c0
- [ ] `UIListener::RegisterForGlobalMessage` · 005e8ae0 · legions 005a9740

## `portal/client/src/uielement/types/text/glyph.cpp.c` (3/7)

### Glyph
- [ ] `Glyph::Glyph` · 005fcd50 · legions 005bd880
- [x] `Glyph::Glyph` · 005fcd70 · legions 005bd8a0 — ac2-ui text.rs / text_attr.rs: UIElement_Text layout and drawing
- [ ] `Glyph::~Glyph` · 005fcdf0 · legions 005bd920
- [ ] `Glyph::operator=` · 005fce10 · legions 005bd940
- [x] `Glyph::IsWhiteSpace` · 005fce80 · legions 005bd9b0 — ac2-ui text.rs / text_attr.rs: UIElement_Text layout and drawing
- [x] `Glyph::IsNewLine` · 005fceb0 · legions 005bd9e0 — ac2-ui text.rs / text_attr.rs: UIElement_Text layout and drawing
- [ ] `Glyph::Glyph` · 005fcec0 · legions 005bd9f0

## `portal/client/src/uielement/types/text/glyphlist.cpp.c` (6/24)

### LinebreakUtils
- [x] `LinebreakUtils::IsWhiteSpace` · 005fbe40 · legions 005bc970 — ac2-ui text.rs / text_attr.rs: UIElement_Text layout and drawing
- [x] `LinebreakUtils::IsNonBeginningChar` · 005fbe60 · legions 005bc990 — ac2-ui text.rs / text_attr.rs: UIElement_Text layout and drawing
- [x] `LinebreakUtils::IsNonEndingChar` · 005fbed0 · legions 005bca00 — ac2-ui text.rs / text_attr.rs: UIElement_Text layout and drawing
- [x] `LinebreakUtils::IsEastAsianChar` · 005fbef0 · legions 005bca20 — ac2-ui text.rs / text_attr.rs: UIElement_Text layout and drawing
- [x] `LinebreakUtils::CanBreakLineAt` · 005fbf30 · legions 005bca60 — ac2-ui text.rs / text_attr.rs: UIElement_Text layout and drawing

### GlyphList
- [ ] `GlyphList::FindCurrentLine` · 005fbfb0 · legions 005bcae0
- [ ] `GlyphList::FindLineFromY` · 005fc040 · legions 005bcb70
- [ ] `GlyphList::FindNextLine` · 005fc0d0 · legions 005bcc00
- [ ] `GlyphList::FindPrevLine` · 005fc130 · legions 005bcc60
- [ ] `GlyphList::GetGlyphLineHeight` · 005fc190 · legions 005bccc0
- [ ] `GlyphList::GetGlyphLineWidth` · 005fc1d0 · legions 005bcd00
- [ ] `GlyphList::FindNextWord` · 005fc240 · legions 005bcd70
- [ ] `GlyphList::FindPrevWord` · 005fc2b0 · legions 005bcde0
- [ ] `GlyphList::FindPixelsFromPos` · 005fc320 · legions 005bce50
- [ ] `GlyphList::FindPosFromLineAndPixels` · 005fc3a0 · legions 005bced0
- [ ] `GlyphList::FindXYFromPosition` · 005fc5f0 · legions 005bd120
- [ ] `GlyphList::Flush` · 005fc6d0 · legions 005bd200
- [ ] `GlyphList::Delete` · 005fc710 · legions 005bd240
- [~] `GlyphList::AddText` · 005fc7e0 · legions 005bd310 — ac2-ui text.rs: no editing, cursor, selection, scrolling, fallback fonts, string variables or server downloads; A8R8G8B8 surfaces
- [ ] `GlyphList::GlyphList` · 005fc930 · legions 005bd460
- [ ] `GlyphList::~GlyphList` · 005fc960 · legions 005bd490
- [ ] `GlyphList::InqText` · 005fc9f0 · legions 005bd520
- [x] `GlyphList::Recalculate` · 005fcaf0 · legions 005bd620 — ac2-ui text.rs / text_attr.rs: UIElement_Text layout and drawing
- [ ] `GlyphList::InqText` · 005fcd20 · legions 005bd850

## `portal/client/src/uielement/types/text/stringdownload.cpp.c` (0/4)

### StringDownload
- [ ] `StringDownload::OnContextFinished` · 005fcef0
- [ ] `StringDownload::Init` · 005fcf50 · legions 005bda80
- [ ] `StringDownload::OnTopLevelRequestFinished` · 005fd150
- [ ] `StringDownload::StringDownload` · 005fd210 · legions 005bdd40

## `portal/client/src/uielement/types/text/uielement_text.cpp.c` (13/92)

### (free functions)
- [ ] `NumberInputFilter` · 005ed7c0

### UIElement_Text
- [ ] `UIElement_Text::GetScrollbar` · 005ed7d0 · legions 005ae3a0
- [ ] `UIElement_Text::HandleMouseWheel` · 005ed840 · legions 005ae410
- [x] `UIElement_Text::CalcJustification` · 005ed8b0 · legions 005ae480 — ac2-ui text.rs / text_attr.rs: UIElement_Text layout and drawing
- [ ] `UIElement_Text::IMETurnOn` · 005ed950 · legions 005ae540
- [~] `UIElement_Text::Activate` · 005ed9a0 · legions 005ae590 — ac2-ui text_edit.rs: no selection, clipboard, IME, page up/down, cursor flashing or view scrolling
- [~] `UIElement_Text::Deactivate` · 005eda10 · legions 005ae600 — ac2-ui text_edit.rs: no selection, clipboard, IME, page up/down, cursor flashing or view scrolling
- [ ] `UIElement_Text::CleanupStringDownloads` · 005eda80 · legions 005ae670
- [x] `UIElement_Text::SetAtrb_Editable` · 005edaf0 · legions 005ae6e0 — ac2-ui input.rs / text_edit.rs: typing into text elements
- [x] `UIElement_Text::DetermineMarginValues` · 005edb30 · legions 005ae720 — ac2-ui text.rs / text_attr.rs: UIElement_Text layout and drawing
- [ ] `UIElement_Text::SetAtrb_OneLine` · 005edbf0 · legions 005ae7e0
- [ ] `UIElement_Text::GetPercentageV` · 005edc40 · legions 005ae830
- [ ] `UIElement_Text::GetPercentageH` · 005edcc0 · legions 005ae8b0
- [ ] `UIElement_Text::ClampPosition` · 005edd40 · legions 005ae930
- [ ] `UIElement_Text::MoveUpDown` · 005edd80 · legions 005ae970
- [ ] `UIElement_Text::MoveBeginEndLine` · 005edf30 · legions 005aeb20
- [ ] `UIElement_Text::SetSelecting` · 005edfd0 · legions 005aebc0
- [ ] `UIElement_Text::SetSelectionStart` · 005ee010 · legions 005aec00
- [ ] `UIElement_Text::SetSelectionEnd` · 005ee070 · legions 005aec60
- [ ] `UIElement_Text::GetSelection` · 005ee0d0 · legions 005aecc0
- [ ] `UIElement_Text::GetTotalLines` · 005ee130 · legions 005aed20

### GlyphLine
- [ ] `GlyphLine::GlyphLine` · 005ee140

### UIElement_Text
- [ ] `UIElement_Text::~UIElement_Text` · 005ee160 · legions 005aed50
- [x] `UIElement_Text::ClearAllText` · 005ee340 · legions 005aef30 — ac2-ui text.rs / text_attr.rs: UIElement_Text layout and drawing
- [ ] `UIElement_Text::SelectAll` · 005ee3a0 · legions 005aef90
- [ ] `UIElement_Text::Deselect` · 005ee400 · legions 005aeff0
- [ ] `UIElement_Text::GetText` · 005ee420 · legions 005af010
- [ ] `UIElement_Text::GetSelectedText` · 005ee490
- [ ] `UIElement_Text::UpdateScrollbar` · 005ee550 · legions 005af140
- [x] `UIElement_Text::ResizeTo` · 005ee6a0 · legions 005af290 — ac2-ui text.rs / text_attr.rs: UIElement_Text layout and drawing
- [ ] `UIElement_Text::DetermineFont` · 005ee6f0 · legions 005af2e0
- [~] `UIElement_Text::SetCursorPosition` · 005ee850 · legions 005af3e0 — ac2-ui text_edit.rs: no selection, clipboard, IME, page up/down, cursor flashing or view scrolling
- [~] `UIElement_Text::UIElement_Text` · 005eea40 · legions 005af5d0 — ac2-ui text.rs: no editing, cursor, selection, scrolling, fallback fonts, string variables or server downloads; A8R8G8B8 surfaces
- [~] `UIElement_Text::SetAttribute_Int` · 005eec20 · legions 005af780 — ac2-ui text.rs: no editing, cursor, selection, scrolling, fallback fonts, string variables or server downloads; A8R8G8B8 surfaces
- [ ] `UIElement_Text::Create` · 005eed10
- [~] `UIElement_Text::InqSizewMargins` · 005eed40 · legions 005af8a0 — ac2-ui reticle.rs: no off-screen indicator, meters, radar-look colour or quantity; name plates and damage text not ported
- [ ] `UIElement_Text::UpdateScrollbars` · 005eef10 · legions 005afa70

### SmartArray<GlyphLine>
- [ ] `SmartArray<GlyphLine>::operator=` · 005eef60 · legions 005afac0

### UIElement_Text
- [ ] `UIElement_Text::SetAttribute_ID` · 005eefd0 · legions 005afb30
- [ ] `UIElement_Text::Register` · 005ef100 · legions 005afc60
- [ ] `UIElement_Text::Initialized` · 005ef120 · legions 005afc80
- [ ] `UIElement_Text::Copy` · 005ef170
- [~] `UIElement_Text::KeepInBounds` · 005ef1c0 · legions 005afd20 — ac2-ui text.rs: no editing, cursor, selection, scrolling, fallback fonts, string variables or server downloads; A8R8G8B8 surfaces
- [~] `UIElement_Text::PostBlit` · 005ef2f0 · legions 005afe50 — ac2-ui text.rs: no editing, cursor, selection, scrolling, fallback fonts, string variables or server downloads; A8R8G8B8 surfaces
- [ ] `UIElement_Text::SetAtrb_CenterVertically` · 005ef8d0 · legions 005b0430
- [~] `UIElement_Text::SetFontNum` · 005ef940 · legions 005b04a0 — ac2-ui text.rs: no editing, cursor, selection, scrolling, fallback fonts, string variables or server downloads; A8R8G8B8 surfaces
- [x] `UIElement_Text::SetFontNumAndColor` · 005efc10 · legions 005b0750 — ac2-ui text.rs / text_attr.rs: UIElement_Text layout and drawing
- [~] `UIElement_Text::RecalculateGlyphList` · 005eff20 · legions 005b0a50 — ac2-ui text.rs: no editing, cursor, selection, scrolling, fallback fonts, string variables or server downloads; A8R8G8B8 surfaces
- [ ] `UIElement_Text::SetViewY` · 005f0080 · legions 005b0bb0
- [x] `UIElement_Text::DrawStart` · 005f00a0 · legions 005b0bd0 — ac2-ui text.rs / text_attr.rs: UIElement_Text layout and drawing
- [ ] `UIElement_Text::DeterminePositionFromXY` · 005f00b0 · legions 005b0be0
- [~] `UIElement_Text::SetCursorPosition` · 005f01d0 · legions 005b0d00 — ac2-ui text_edit.rs: no selection, clipboard, IME, page up/down, cursor flashing or view scrolling
- [~] `UIElement_Text::MouseDown` · 005f0200 · legions 005b0d30 — ac2-ui text_edit.rs: no selection, clipboard, IME, page up/down, cursor flashing or view scrolling
- [ ] `UIElement_Text::MouseUp` · 005f02e0 · legions 005b0e10
- [ ] `UIElement_Text::Global_MouseMove` · 005f0370 · legions 005b0ea0
- [ ] `UIElement_Text::DetermineAndSetNewLine` · 005f03e0 · legions 005b0f10
- [ ] `UIElement_Text::DetermineAndSetNewPage` · 005f04f0 · legions 005b1020
- [ ] `UIElement_Text::ScrollToPosition` · 005f0590 · legions 005b10c0
- [ ] `UIElement_Text::ScrollHorizontalView` · 005f05f0 · legions 005b1120
- [ ] `UIElement_Text::DeleteSection` · 005f06a0 · legions 005b11d0
- [ ] `UIElement_Text::VerticalPageDecrement` · 005f0730 · legions 005b1260
- [ ] `UIElement_Text::VerticalPageIncrement` · 005f0740 · legions 005b1270
- [ ] `UIElement_Text::ScrollToHome` · 005f0750 · legions 005b1280
- [x] `UIElement_Text::ScrollToEnd` · 005f0770 · legions 005b12a0 — ac2-ui chat.rs / chat_window.rs
- [ ] `UIElement_Text::HandleScrollBarMessage` · 005f0790 · legions 005b12c0
- [~] `UIElement_Text::MoveCursor` · 005f0940 · legions 005b1470 — ac2-ui text_edit.rs: no selection, clipboard, IME, page up/down, cursor flashing or view scrolling
- [ ] `UIElement_Text::MoveCursorToEnd` · 005f0ad0 · legions 005b1600
- [ ] `UIElement_Text::DeleteChar` · 005f0b10 · legions 005b1640
- [ ] `UIElement_Text::DeleteSelection` · 005f0b30 · legions 005b1660
- [~] `UIElement_Text::AddText_Internal` · 005f0ba0 · legions 005b16d0 — ac2-ui text.rs: no editing, cursor, selection, scrolling, fallback fonts, string variables or server downloads; A8R8G8B8 surfaces
- [ ] `UIElement_Text::Cut` · 005f0cf0
- [x] `UIElement_Text::ActionHandler` · 005f0d50 — ac2-ui input.rs / text_edit.rs: typing into text elements
- [~] `UIElement_Text::CheckStringDownloads` · 005f0e90 · legions 005b19c0 — ac2-ui text.rs: no editing, cursor, selection, scrolling, fallback fonts, string variables or server downloads; A8R8G8B8 surfaces
- [ ] `UIElement_Text::Paste` · 005f0f70
- [~] `UIElement_Text::CharacterHandler` · 005f0ff0 — ac2-ui text_edit.rs: no selection, clipboard, IME, page up/down, cursor flashing or view scrolling
- [ ] `UIElement_Text::Global_Loop` · 005f12a0 · legions 005b1dd0
- [x] `UIElement_Text::AddStringInfowFont` · 005f1350 · legions 005b1e80 — ac2-ui chat.rs / chat_window.rs
- [x] `UIElement_Text::AddStringInfowFont` · 005f13e0 · legions 005b1f10 — ac2-ui chat.rs / chat_window.rs
- [ ] `UIElement_Text::ListenToMessage` · 005f1470 · legions 005b1fa0
- [ ] `UIElement_Text::AddStringInfo` · 005f14b0 · legions 005b1fe0
- [ ] `UIElement_Text::SetStringInfo` · 005f14c0 · legions 005b1ff0
- [ ] `UIElement_Text::SetStringInfo` · 005f14e0 · legions 005b2010
- [ ] `UIElement_Text::AddText` · 005f1510 · legions 005b2040
- [ ] `UIElement_Text::TruncateText` · 005f1570 · legions 005b20a0
- [~] `UIElement_Text::SetText` · 005f16e0 · legions 005b2210 — ac2-ui reticle.rs: no off-screen indicator, meters, radar-look colour or quantity; name plates and damage text not ported
- [~] `UIElement_Text::SetText` · 005f1750 · legions 005b2280 — ac2-ui reticle.rs: no off-screen indicator, meters, radar-look colour or quantity; name plates and damage text not ported
- [~] `UIElement_Text::SetString` · 005f17c0 · legions 005b22f0 — ac2-ui text.rs: no editing, cursor, selection, scrolling, fallback fonts, string variables or server downloads; A8R8G8B8 surfaces
- [ ] `UIElement_Text::MatchAttributes` · 005f1810 · legions 005b2340
- [~] `UIElement_Text::SetAttribute_Bool` · 005f18c0 · legions 005b23f0 — ac2-ui text.rs: no editing, cursor, selection, scrolling, fallback fonts, string variables or server downloads; A8R8G8B8 surfaces
- [x] `UIElement_Text::SetAttribute_StringID` · 005f1a10 · legions 005b2540 — ac2-ui text.rs / text_attr.rs: UIElement_Text layout and drawing
- [x] `UIElement_Text::SetAttribute_DataID` · 005f1a50 · legions 005b2580 — ac2-ui text.rs / text_attr.rs: UIElement_Text layout and drawing
- [ ] `UIElement_Text::ListenToMessage` · 005f1aa0 · legions 005b25d0

## `portal/client/src/uielement/types/uielement_button.cpp.c` (4/16)

### UIElement_Button
- [~] `UIElement_Button::UIElement_Button` · 005fb4e0 · legions 005bc010 — ac2-ui mouse.rs / button.rs: no tooltips, cursors, drag and drop, moving/resizing, activation, hot clicks or listener registry (messages are taken by the caller)
- [ ] `UIElement_Button::Create` · 005fb540
- [ ] `UIElement_Button::Global_Loop` · 005fb570 · legions 005bc0a0
- [ ] `UIElement_Button::ListenToMessage` · 005fb5f0 · legions 005bc120
- [x] `UIElement_Button::UpdateState` · 005fb620 · legions 005bc150 — ac2-ui mouse.rs / button.rs: UI hit tests, presses and buttons
- [x] `UIElement_Button::SetState` · 005fb710 · legions 005bc240 — ac2-ui mouse.rs / button.rs: UI hit tests, presses and buttons
- [ ] `UIElement_Button::IsActivatable` · 005fb770 · legions 005bc2a0
- [ ] `UIElement_Button::Activate` · 005fb7a0 · legions 005bc2d0
- [ ] `UIElement_Button::Deactivate` · 005fb7c0 · legions 005bc2f0
- [ ] `UIElement_Button::StartUseActive` · 005fb7e0 · legions 005bc310
- [ ] `UIElement_Button::EndUseActive` · 005fb870 · legions 005bc3a0
- [~] `UIElement_Button::SetAttribute_Bool` · 005fb940 · legions 005bc470 — ac2-ui mouse.rs / button.rs: no tooltips, cursors, drag and drop, moving/resizing, activation, hot clicks or listener registry (messages are taken by the caller)
- [ ] `UIElement_Button::Register` · 005fb9b0 · legions 005bc4e0
- [~] `UIElement_Button::MouseDown` · 005fb9d0 · legions 005bc500 — ac2-ui mouse.rs / button.rs: no tooltips, cursors, drag and drop, moving/resizing, activation, hot clicks or listener registry (messages are taken by the caller)
- [x] `UIElement_Button::MouseUp` · 005fbb10 · legions 005bc640 — ac2-ui mouse.rs / button.rs: UI hit tests, presses and buttons
- [x] `UIElement_Button::MouseOverTop` · 005fbc50 · legions 005bc780 — ac2-ui mouse.rs / button.rs: UI hit tests, presses and buttons

## `portal/client/src/uielement/types/uielement_dragbar.cpp.c` (0/8)

### UIElement_Dragbar
- [ ] `UIElement_Dragbar::MouseDown` · 005f8b80 · legions 005b96b0
- [ ] `UIElement_Dragbar::MouseUp` · 005f8c30 · legions 005b9760
- [ ] `UIElement_Dragbar::UpdateLayout` · 005f8c90 · legions 005b97c0
- [ ] `UIElement_Dragbar::SetState` · 005f8d60 · legions 005b9890
- [ ] `UIElement_Dragbar::SetAttribute_Bool` · 005f8d80 · legions 005b98b0
- [ ] `UIElement_Dragbar::UIElement_Dragbar` · 005f8dc0 · legions 005b98f0
- [ ] `UIElement_Dragbar::Create` · 005f8e20
- [ ] `UIElement_Dragbar::Register` · 005f8e50 · legions 005b9980

## `portal/client/src/uielement/types/uielement_field.cpp.c` (0/4)

### UIElement_Field
- [ ] `UIElement_Field::Create` · 005fbc70
- [ ] `UIElement_Field::MouseOverTop` · 005fbcb0 · legions 005bc7e0
- [ ] `UIElement_Field::CatchDroppedItem` · 005fbdb0 · legions 005bc8e0
- [ ] `UIElement_Field::Register` · 005fbe20 · legions 005bc950

## `portal/client/src/uielement/types/uielement_listbox.cpp.c` (20/82)

### UIElement_ListBox
- [x] `UIElement_ListBox::Home` · 005f1b90 · legions 005b26c0 — ac2-ui list_box.rs
- [x] `UIElement_ListBox::End` · 005f1bc0 · legions 005b26f0 — ac2-ui list_box.rs
- [x] `UIElement_ListBox::ScrollToRow` · 005f1bf0 · legions 005b2720 — ac2-ui list_box.rs
- [ ] `UIElement_ListBox::ScrollToCol` · 005f1c20 · legions 005b2750
- [ ] `UIElement_ListBox::GetLeftCol` · 005f1c50 · legions 005b2780
- [x] `UIElement_ListBox::GetTopRow` · 005f1c70 · legions 005b27a0 — ac2-ui list_box.rs
- [x] `UIElement_ListBox::KeepViewInBounds` · 005f1c90 · legions 005b27c0 — ac2-ui list_box.rs
- [ ] `UIElement_ListBox::HandleButtonGhosting` · 005f1d40 · legions 005b2870
- [ ] `UIElement_ListBox::GetScrollBar` · 005f1db0 · legions 005b28e0
- [ ] `UIElement_ListBox::UnregisterForScrollbar` · 005f1e50 · legions 005b2980
- [ ] `UIElement_ListBox::TestBitMask` · 005f1f60 · legions 005b2a90
- [~] `UIElement_ListBox::ScrollToShow` · 005f1f80 · legions 005b2ab0 — list_box_select.rs: left-press selection; wheel/scroll bar and smooth (goal) scrolling not modelled
- [~] `UIElement_ListBox::SetAttribute_Bool` · 005f1fc0 · legions 005b2af0 — ac2-ui list_box.rs: no smooth scrolling, controls or selection
- [ ] `UIElement_ListBox::DetermineAndSetNewGoal` · 005f2180 · legions 005b2cb0
- [x] `UIElement_ListBox::UpdateLayout` · 005f21f0 · legions 005b2d20 — ac2-ui list_box.rs
- [ ] `UIElement_ListBox::StartAnimation` · 005f23f0 · legions 005b2f20
- [ ] `UIElement_ListBox::StopAnimation` · 005f2500 · legions 005b3030
- [x] `UIElement_ListBox::GetItem` · 005f2540 · legions 005b3070 — ac2-ui list_box.rs
- [x] `UIElement_ListBox::SetSelectedItem` · 005f2560 · legions 005b3090 — ac2-ui list_box_select.rs
- [x] `UIElement_ListBox::SetSelectedIndex` · 005f2650 · legions 005b3180 — ac2-ui list_box_select.rs
- [x] `UIElement_ListBox::GetItemIndexAtPoint` · 005f26b0 · legions 005b31e0 — ac2-ui list_box.rs
- [~] `UIElement_ListBox::InsertItem` · 005f27c0 · legions 005b32f0 — ac2-ui list_box.rs: no smooth scrolling, controls or selection
- [ ] `UIElement_ListBox::MultiOpAddItem` · 005f2820 · legions 005b3350
- [ ] `UIElement_ListBox::MultiOpInsertItem` · 005f2840 · legions 005b3370
- [ ] `UIElement_ListBox::MultiOpInsertItem` · 005f2850 · legions 005b3380
- [ ] `UIElement_ListBox::GetCol` · 005f2870 · legions 005b33a0
- [ ] `UIElement_ListBox::GetRow` · 005f28d0 · legions 005b3400
- [~] `UIElement_ListBox::ScrollToShow` · 005f2930 · legions 005b3460 — list_box_select.rs: left-press selection; wheel/scroll bar and smooth (goal) scrolling not modelled
- [ ] `UIElement_ListBox::StartDragSelect` · 005f2a50 · legions 005b3580
- [ ] `UIElement_ListBox::EndDragSelect` · 005f2ac0 · legions 005b35f0
- [ ] `UIElement_ListBox::StartRollover` · 005f2b10 · legions 005b3640
- [ ] `UIElement_ListBox::EndRollover` · 005f2b60 · legions 005b3690
- [x] `UIElement_ListBox::ConstrainViewValue` · 005f2bc0 · legions 005b36f0 — ac2-ui list_box.rs
- [ ] `UIElement_ListBox::GetPercentageV` · 005f2c30 · legions 005b3760
- [ ] `UIElement_ListBox::GetPercentageH` · 005f2cc0 · legions 005b37f0
- [ ] `UIElement_ListBox::HandleMouseWheel` · 005f2d50 · legions 005b3880
- [ ] `UIElement_ListBox::HandleScrollBarMessage` · 005f2d90 · legions 005b38c0
- [ ] `UIElement_ListBox::HandleScrollbarWidgetRelease` · 005f2f60 · legions 005b3a90
- [ ] `UIElement_ListBox::~UIElement_ListBox` · 005f3070 · legions 005b3ba0
- [ ] `UIElement_ListBox::MouseOverTop` · 005f3190 · legions 005b3cc0
- [ ] `UIElement_ListBox::ListenToMessage` · 005f31f0 · legions 005b3d20
- [ ] `UIElement_ListBox::Global_Loop` · 005f3320 · legions 005b3e50
- [ ] `UIElement_ListBox::StartActive` · 005f3430 · legions 005b3f60
- [ ] `UIElement_ListBox::VerticalPageDecrement` · 005f34d0 · legions 005b4000
- [ ] `UIElement_ListBox::VerticalPageIncrement` · 005f3550 · legions 005b4080
- [ ] `UIElement_ListBox::HorizontalPageDecrement` · 005f35e0 · legions 005b4110
- [ ] `UIElement_ListBox::HorizontalPageIncrement` · 005f3660 · legions 005b4190
- [ ] `UIElement_ListBox::VerticalLineDecrement` · 005f36f0 · legions 005b4220
- [ ] `UIElement_ListBox::VerticalLineIncrement` · 005f3760 · legions 005b4290
- [ ] `UIElement_ListBox::HorizontalLineDecrement` · 005f37e0 · legions 005b4310
- [ ] `UIElement_ListBox::HorizontalLineIncrement` · 005f3850 · legions 005b4380
- [ ] `UIElement_ListBox::UpdateControls` · 005f38d0 · legions 005b4400
- [ ] `UIElement_ListBox::UpdateAndDraw` · 005f3af0 · legions 005b4620
- [ ] `UIElement_ListBox::ResizeTo` · 005f3b20 · legions 005b4650
- [ ] `UIElement_ListBox::SetSelectedItemByID` · 005f3b60 · legions 005b4690
- [x] `UIElement_ListBox::GetItemUnderMouse` · 005f3c20 · legions 005b4750 — ac2-ui list_box.rs
- [x] `UIElement_ListBox::RemoveItem` · 005f3c90 · legions 005b47c0 — ac2-ui list_box.rs
- [ ] `UIElement_ListBox::MultiOpRemoveItem` · 005f3d60 · legions 005b4890
- [x] `UIElement_ListBox::MultiOpComplete` · 005f3d70 · legions 005b48a0 — ac2-ui list_box.rs
- [x] `UIElement_ListBox::Flush` · 005f3db0 · legions 005b48e0 — ac2-ui list_box.rs
- [ ] `UIElement_ListBox::Rollover` · 005f3e70 · legions 005b49a0
- [x] `UIElement_ListBox::ViewChanged` · 005f3f30 · legions 005b4a60 — ac2-ui list_box.rs
- [x] `UIElement_ListBox::HandleSelection` · 005f3fd0 · legions 005b4b00 — ac2-ui list_box_select.rs
- [ ] `UIElement_ListBox::SetState` · 005f4030 · legions 005b4b60
- [~] `UIElement_ListBox::SetAttribute_Int` · 005f4170 · legions 005b4ca0 — ac2-ui list_box.rs: no smooth scrolling, controls or selection
- [ ] `UIElement_ListBox::SetAttribute_ID` · 005f4270 · legions 005b4da0
- [~] `UIElement_ListBox::MouseDown` · 005f4420 · legions 005b4f50 — list_box_select.rs: left-press selection; wheel/scroll bar and smooth (goal) scrolling not modelled
- [ ] `UIElement_ListBox::Global_MouseMove` · 005f44c0 · legions 005b4ff0
- [ ] `UIElement_ListBox::StartUpActive` · 005f4500 · legions 005b5030
- [ ] `UIElement_ListBox::StartDownActive` · 005f4540 · legions 005b5070
- [ ] `UIElement_ListBox::StartLeftActive` · 005f4570 · legions 005b50a0
- [ ] `UIElement_ListBox::StartRightActive` · 005f45b0 · legions 005b50e0
- [x] `UIElement_ListBox::AddItem` · 005f45e0 · legions 005b5110 — ac2-ui list_box.rs
- [~] `UIElement_ListBox::InsertItem` · 005f4680 · legions 005b51b0 — ac2-ui list_box.rs: no smooth scrolling, controls or selection
- [x] `UIElement_ListBox::DeleteItem` · 005f46f0 · legions 005b5220 — ac2-ui list_box.rs
- [ ] `UIElement_ListBox::MultiOpRemoveItem` · 005f4750 · legions 005b5280
- [~] `UIElement_ListBox::UIElement_ListBox` · 005f47a0 · legions 005b52d0 — ac2-ui list_box.rs: no smooth scrolling, controls or selection
- [ ] `UIElement_ListBox::Create` · 005f4a80
- [ ] `UIElement_ListBox::ListenToMessage` · 005f4ab0 · legions 005b55e0
- [ ] `UIElement_ListBox::ListenToMessage` · 005f4b00 · legions 005b5630
- [x] `UIElement_ListBox::SetItem` · 005f4b90 · legions 005b56c0 — ac2-ui list_box.rs
- [ ] `UIElement_ListBox::Register` · 005f4c30 · legions 005b5760

## `portal/client/src/uielement/types/uielement_listboxmsize.cpp.c` (0/20)

### UIElement_ListBoxMSize
- [ ] `UIElement_ListBoxMSize::ConstrainViewValue` · 005f8e70 · legions 005b99a0
- [ ] `UIElement_ListBoxMSize::InqItemIndex` · 005f8ed0 · legions 005b9a00
- [ ] `UIElement_ListBoxMSize::InqItemSize` · 005f8f20 · legions 005b9a50
- [ ] `UIElement_ListBoxMSize::ScrollToShow` · 005f8f60 · legions 005b9a90
- [ ] `UIElement_ListBoxMSize::ScrollToRow` · 005f90d0 · legions 005b9c00
- [ ] `UIElement_ListBoxMSize::ScrollToCol` · 005f9110 · legions 005b9c40
- [ ] `UIElement_ListBoxMSize::VerticalPageDecrement` · 005f91a0 · legions 005b9cd0
- [ ] `UIElement_ListBoxMSize::VerticalPageIncrement` · 005f9200 · legions 005b9d30
- [ ] `UIElement_ListBoxMSize::HorizontalPageDecrement` · 005f9270 · legions 005b9da0
- [ ] `UIElement_ListBoxMSize::HorizontalPageIncrement` · 005f92d0 · legions 005b9e00
- [ ] `UIElement_ListBoxMSize::VerticalLineDecrement` · 005f9340 · legions 005b9e70
- [ ] `UIElement_ListBoxMSize::VerticalLineIncrement` · 005f93a0 · legions 005b9ed0
- [ ] `UIElement_ListBoxMSize::HorizontalLineDecrement` · 005f9410 · legions 005b9f40
- [ ] `UIElement_ListBoxMSize::HorizontalLineIncrement` · 005f9470 · legions 005b9fa0
- [ ] `UIElement_ListBoxMSize::GetItemIndexAtPoint` · 005f94e0 · legions 005ba010
- [ ] `UIElement_ListBoxMSize::UIElement_ListBoxMSize` · 005f95f0 · legions 005ba120
- [ ] `UIElement_ListBoxMSize::Create` · 005f9640
- [ ] `UIElement_ListBoxMSize::CalculateItemWidthsHeights` · 005f9670 · legions 005ba1a0
- [ ] `UIElement_ListBoxMSize::Register` · 005f98a0 · legions 005ba3d0
- [ ] `UIElement_ListBoxMSize::UpdateLayout` · 005f98c0 · legions 005ba3f0

## `portal/client/src/uielement/types/uielement_menu.cpp.c` (0/22)

### UIElement_Menu
- [ ] `UIElement_Menu::Create` · 005f5420
- [ ] `UIElement_Menu::Flush` · 005f5470 · legions 005b5fa0
- [ ] `UIElement_Menu::GetItem` · 005f5480 · legions 005b5fb0
- [ ] `UIElement_Menu::GetSelectedItem` · 005f54a0 · legions 005b5fd0
- [ ] `UIElement_Menu::SetSelectedIndex` · 005f54c0 · legions 005b5ff0
- [ ] `UIElement_Menu::SetSelectedItem` · 005f54e0 · legions 005b6010
- [ ] `UIElement_Menu::Register` · 005f5500 · legions 005b6030
- [ ] `UIElement_Menu::InsertTextItem` · 005f5520 · legions 005b6050
- [ ] `UIElement_Menu::GetNumItems` · 005f55e0 · legions 005b6110
- [ ] `UIElement_Menu::GetSelectedIndex` · 005f5600 · legions 005b6130
- [ ] `UIElement_Menu::Initialized` · 005f5640 · legions 005b6170
- [ ] `UIElement_Menu::NewSelection` · 005f5760 · legions 005b6290
- [ ] `UIElement_Menu::UpdateState` · 005f5810 · legions 005b6340
- [ ] `UIElement_Menu::MouseOverTop` · 005f58a0 · legions 005b63d0
- [ ] `UIElement_Menu::ListenToMessage` · 005f58c0 · legions 005b63f0
- [ ] `UIElement_Menu::Open` · 005f58e0 · legions 005b6410
- [ ] `UIElement_Menu::Close` · 005f5ad0 · legions 005b6600
- [ ] `UIElement_Menu::~UIElement_Menu` · 005f5b40 · legions 005b6670
- [ ] `UIElement_Menu::SetVisible` · 005f5bd0 · legions 005b6700
- [ ] `UIElement_Menu::SetAttribute_Bool` · 005f5c10 · legions 005b6740
- [ ] `UIElement_Menu::MouseDown` · 005f5c60 · legions 005b6790
- [ ] `UIElement_Menu::ListenToMessage` · 005f5cf0 · legions 005b6820

## `portal/client/src/uielement/types/uielement_meter.cpp.c` (13/14)

### UIElement_Meter
- [x] `UIElement_Meter::UpdateLayout` · 005f4c50 · legions 005b5780 — ac2-ui meter.rs
- [x] `UIElement_Meter::SetParent` · 005f4eb0 · legions 005b59e0 — ac2-ui meter.rs
- [x] `UIElement_Meter::SetState` · 005f4ed0 · legions 005b5a00 — ac2-ui meter.rs
- [x] `UIElement_Meter::MoveTo` · 005f4ef0 · legions 005b5a20 — ac2-ui meter.rs
- [x] `UIElement_Meter::ResizeTo` · 005f4f10 · legions 005b5a40 — ac2-ui meter.rs
- [~] `UIElement_Meter::CalculateNextStateChange` · 005f4f30 · legions 005b5a60 — ties between equal percentages follow insertion order, not hash bucket order
- [x] `UIElement_Meter::Global_Loop` · 005f4fe0 · legions 005b5b10 — ac2-ui meter.rs
- [x] `UIElement_Meter::ListenToMessage` · 005f5110 · legions 005b5c40 — ac2-ui meter.rs
- [x] `UIElement_Meter::StartAnimation` · 005f5140 · legions 005b5c70 — ac2-ui meter.rs
- [x] `UIElement_Meter::SetStateAtPercentage` · 005f5270 · legions 005b5da0 — ac2-ui meter.rs
- [x] `UIElement_Meter::SetAttribute_Float` · 005f52a0 · legions 005b5dd0 — ac2-ui meter.rs
- [x] `UIElement_Meter::UIElement_Meter` · 005f5320 · legions 005b5e50 — ac2-ui meter.rs
- [x] `UIElement_Meter::Create` · 005f53d0 — ac2-ui meter.rs
- [x] `UIElement_Meter::Register` · 005f5400 · legions 005b5f30 — ac2-ui meter.rs

## `portal/client/src/uielement/types/uielement_panel.cpp.c` (2/6)

### UIElement_Panel
- [ ] `UIElement_Panel::~UIElement_Panel` · 005f89a0 · legions 005b94d0
- [ ] `UIElement_Panel::Create` · 005f8a30
- [ ] `UIElement_Panel::SetAttribute_Bool` · 005f8a60 · legions 005b9590
- [x] `UIElement_Panel::Initialized` · 005f8a70 · legions 005b95a0 — ac2-ui panel.rs
- [x] `UIElement_Panel::ListenToMessage` · 005f8ae0 · legions 005b9610 — ac2-ui panel.rs
- [ ] `UIElement_Panel::Register` · 005f8b60 · legions 005b9690

## `portal/client/src/uielement/types/uielement_panelpage.cpp.c` (5/9)

### UIElement_PanelPage
- [ ] `UIElement_PanelPage::Create` · 005f8660
- [x] `UIElement_PanelPage::Initialized` · 005f86a0 · legions 005b91d0 — ac2-ui panel.rs
- [x] `UIElement_PanelPage::SetState` · 005f8760 · legions 005b9290 — ac2-ui panel.rs
- [x] `UIElement_PanelPage::MouseDown` · 005f87e0 · legions 005b9310 — ac2-ui panel.rs
- [x] `UIElement_PanelPage::MouseOverTop` · 005f8830 · legions 005b9360 — ac2-ui panel.rs
- [ ] `UIElement_PanelPage::Activate` · 005f8890 · legions 005b93c0
- [ ] `UIElement_PanelPage::Deactivate` · 005f88d0 · legions 005b9400
- [x] `UIElement_PanelPage::StartUseActive` · 005f8910 · legions 005b9440 — ac2-ui panel.rs
- [ ] `UIElement_PanelPage::Register` · 005f8980 · legions 005b94b0

## `portal/client/src/uielement/types/uielement_resizebar.cpp.c` (0/8)

### UIElement_Resizebar
- [ ] `UIElement_Resizebar::MouseDown` · 005f8050 · legions 005b8b80
- [ ] `UIElement_Resizebar::MouseUp` · 005f81d0 · legions 005b8d00
- [ ] `UIElement_Resizebar::UpdateLayout` · 005f8230 · legions 005b8d60
- [ ] `UIElement_Resizebar::SetState` · 005f8300 · legions 005b8e30
- [ ] `UIElement_Resizebar::SetAttribute_Bool` · 005f8320 · legions 005b8e50
- [ ] `UIElement_Resizebar::UIElement_Resizebar` · 005f8360 · legions 005b8e90
- [ ] `UIElement_Resizebar::Create` · 005f83c0
- [ ] `UIElement_Resizebar::Register` · 005f83f0 · legions 005b8f20

## `portal/client/src/uielement/types/uielement_scrollbar.cpp.c` (10/28)

### UIElement_Scrollbar
- [ ] `UIElement_Scrollbar::SetAttribute_ID` · 005f9a20 · legions 005ba550
- [x] `UIElement_Scrollbar::BroadcastMessageOnRoot` · 005f9a80 · legions 005ba5b0 — ac2-ui scrollbar.rs
- [ ] `UIElement_Scrollbar::HandleHideButton` · 005f9aa0 · legions 005ba5d0
- [ ] `UIElement_Scrollbar::HandleFillChild` · 005f9ae0 · legions 005ba610
- [~] `UIElement_Scrollbar::UpdateLayout` · 005f9b30 · legions 005ba660 — scrollbar.rs: widget placement, drag, touched position, stops, step buttons; proportional widgets, fills, paging, smooth movement, wheel and disabling not modelled
- [ ] `UIElement_Scrollbar::StartAnimation` · 005fa050 · legions 005bab80
- [ ] `UIElement_Scrollbar::StopAnimation` · 005fa170 · legions 005baca0
- [x] `UIElement_Scrollbar::ResizeTo` · 005fa1b0 · legions 005bace0 — ac2-ui scrollbar.rs
- [x] `UIElement_Scrollbar::RegisterForMouseMove` · 005fa1d0 · legions 005bad00 — ac2-ui scrollbar.rs
- [x] `UIElement_Scrollbar::HandleMoveTo` · 005fa200 · legions 005bad30 — ac2-ui scrollbar.rs
- [x] `UIElement_Scrollbar::HandleMouseMove` · 005fa330 · legions 005bae60 — ac2-ui scrollbar.rs
- [x] `UIElement_Scrollbar::HandleMoveSteps` · 005fa3a0 · legions 005baed0 — ac2-ui scrollbar.rs
- [ ] `UIElement_Scrollbar::~UIElement_Scrollbar` · 005fa4a0 · legions 005bafd0
- [x] `UIElement_Scrollbar::SetState` · 005fa5b0 · legions 005bb0e0 — ac2-ui scrollbar.rs
- [ ] `UIElement_Scrollbar::SetAttribute_Bool` · 005fa5d0 · legions 005bb100
- [~] `UIElement_Scrollbar::SetAttribute_Float` · 005fa780 · legions 005bb2b0 — scrollbar.rs: widget placement, drag, touched position, stops, step buttons; proportional widgets, fills, paging, smooth movement, wheel and disabling not modelled
- [ ] `UIElement_Scrollbar::SetAttribute_Int` · 005fa820 · legions 005bb350
- [x] `UIElement_Scrollbar::MouseUp` · 005fa860 · legions 005bb390 — ac2-ui scrollbar.rs
- [ ] `UIElement_Scrollbar::Global_Loop` · 005fa910 · legions 005bb440
- [x] `UIElement_Scrollbar::Global_MouseMove` · 005fabe0 · legions 005bb710 — ac2-ui scrollbar.rs
- [~] `UIElement_Scrollbar::ListenToMessage` · 005facd0 · legions 005bb800 — scrollbar.rs: widget placement, drag, touched position, stops, step buttons; proportional widgets, fills, paging, smooth movement, wheel and disabling not modelled
- [ ] `UIElement_Scrollbar::HandleMouseWheel` · 005fad20 · legions 005bb850
- [x] `UIElement_Scrollbar::HandleButtonHotclick` · 005fadd0 · legions 005bb900 — ac2-ui scrollbar.rs
- [~] `UIElement_Scrollbar::UIElement_Scrollbar` · 005fae40 · legions 005bb970 — scrollbar.rs: widget placement, drag, touched position, stops, step buttons; proportional widgets, fills, paging, smooth movement, wheel and disabling not modelled
- [ ] `UIElement_Scrollbar::Create` · 005fb0a0
- [~] `UIElement_Scrollbar::MouseDown` · 005fb0d0 · legions 005bbc00 — scrollbar.rs: widget placement, drag, touched position, stops, step buttons; proportional widgets, fills, paging, smooth movement, wheel and disabling not modelled
- [~] `UIElement_Scrollbar::ListenToMessage` · 005fb400 · legions 005bbf30 — scrollbar.rs: widget placement, drag, touched position, stops, step buttons; proportional widgets, fills, paging, smooth movement, wheel and disabling not modelled
- [ ] `UIElement_Scrollbar::Register` · 005fb4c0 · legions 005bbff0

## `portal/client/src/uielement/types/uielement_tree.cpp.c` (16/39)

### UIElement_Tree
- [x] `UIElement_Tree::CalculatePaperDimen` · 005f5d90 · legions 005b68c0 — ac2-ui tree_view.rs
- [x] `UIElement_Tree::MultiOpStart` · 005f5e20 · legions 005b6950 — ac2-ui tree_view.rs
- [ ] `UIElement_Tree::DrawMeBotToTop` · 005f5e30 · legions 005b6960
- [ ] `UIElement_Tree::GetPercentageV` · 005f5e50 · legions 005b6980
- [ ] `UIElement_Tree::GetPercentageH` · 005f5ee0 · legions 005b6a10
- [ ] `UIElement_Tree::VerticalPageDecrement` · 005f5f70 · legions 005b6aa0
- [ ] `UIElement_Tree::VerticalPageIncrement` · 005f5fc0 · legions 005b6af0
- [ ] `UIElement_Tree::HorizontalPageDecrement` · 005f6020 · legions 005b6b50
- [ ] `UIElement_Tree::HorizontalPageIncrement` · 005f6070 · legions 005b6ba0
- [ ] `UIElement_Tree::VerticalLineDecrement` · 005f60c0 · legions 005b6bf0
- [ ] `UIElement_Tree::VerticalLineIncrement` · 005f6100 · legions 005b6c30
- [ ] `UIElement_Tree::HorizontalLineDecrement` · 005f6150 · legions 005b6c80
- [ ] `UIElement_Tree::HorizontalLineIncrement` · 005f6190 · legions 005b6cc0
- [x] `UIElement_Tree::End` · 005f61e0 · legions 005b6d10 — ac2-ui tree_view.rs
- [x] `UIElement_Tree::DrawLine` · 005f6270 · legions 005b6da0 — ac2-ui tree_view.rs
- [x] `UIElement_Tree::GetNodeInGrid` · 005f6410 · legions 005b6f40 — ac2-ui tree_view.rs
- [x] `UIElement_Tree::StaticLayoutUpdate` · 005f6450 · legions 005b6f80 — ac2-ui tree_view.rs
- [x] `UIElement_Tree::GridLayoutUpdate` · 005f6500 · legions 005b7030 — ac2-ui tree_view.rs
- [ ] `UIElement_Tree::UpdateControls` · 005f65f0 · legions 005b7120
- [ ] `UIElement_Tree::ListenToMessage` · 005f6a50 · legions 005b7580
- [x] `UIElement_Tree::DrawDone` · 005f6d10 · legions 005b7840 — ac2-ui tree_view.rs
- [x] `UIElement_Tree::GetItem` · 005f6f50 · legions 005b7a80 — ac2-ui tree_view.rs
- [ ] `UIElement_Tree::SetAttribute_ID` · 005f6f70 · legions 005b7aa0
- [x] `UIElement_Tree::AddGridNode` · 005f70d0 · legions 005b7c00 — ac2-ui tree_view.rs
- [x] `UIElement_Tree::IsOccupied` · 005f71f0 · legions 005b7d20 — ac2-ui tree_view.rs
- [x] `UIElement_Tree::UpdateLayout` · 005f7220 · legions 005b7d50 — ac2-ui tree_view.rs
- [ ] `UIElement_Tree::SetSelectedItem` · 005f73e0 · legions 005b7f10
- [x] `UIElement_Tree::SetItemsParent` · 005f74f0 · legions 005b8020 — ac2-ui tree_view.rs
- [x] `UIElement_Tree::MultiOpComplete` · 005f7590 · legions 005b80c0 — ac2-ui tree_view.rs
- [x] `UIElement_Tree::AddItem` · 005f75c0 · legions 005b80f0 — ac2-ui tree_view.rs
- [x] `UIElement_Tree::FlushTree` · 005f76e0 · legions 005b8210 — ac2-ui tree_view.rs
- [ ] `UIElement_Tree::~UIElement_Tree` · 005f7830 · legions 005b8360
- [~] `UIElement_Tree::SetAttribute_Bool` · 005f7b40 · legions 005b8670 — ac2-ui tree_view.rs: view via tree_end/flush only
- [~] `UIElement_Tree::SetAttribute_Int` · 005f7b90 · legions 005b86c0 — ac2-ui tree_view.rs: view via tree_end/flush only
- [ ] `UIElement_Tree::MouseDown` · 005f7c50 · legions 005b8780
- [ ] `UIElement_Tree::SetState` · 005f7dd0 · legions 005b8900
- [ ] `UIElement_Tree::UIElement_Tree` · 005f7f30 · legions 005b8a60
- [ ] `UIElement_Tree::Create` · 005f8000
- [ ] `UIElement_Tree::Register` · 005f8030 · legions 005b8b60

## `portal/client/src/uielement/types/uielement_viewport.cpp.c` (0/10)

### UIElement_Viewport
- [ ] `UIElement_Viewport::GetUISceneID` · 005f8410 · legions 005b8f40
- [ ] `UIElement_Viewport::Create` · 005f8420
- [ ] `UIElement_Viewport::SetCameraID` · 005f8480 · legions 005b8fb0
- [ ] `UIElement_Viewport::GetCameraID` · 005f84d0 · legions 005b9000
- [ ] `UIElement_Viewport::GetUIScene` · 005f84f0 · legions 005b9020
- [ ] `UIElement_Viewport::ReleaseUIScene` · 005f8510 · legions 005b9040
- [ ] `UIElement_Viewport::Register` · 005f8560 · legions 005b9090
- [ ] `UIElement_Viewport::SetAttribute_DataID` · 005f8580 · legions 005b90b0
- [ ] `UIElement_Viewport::SetAttribute_ASCII` · 005f85e0 · legions 005b9110
- [ ] `UIElement_Viewport::SetObject` · 005f8620 · legions 005b9150
