# ac2-smartbox: SmartBox, camera, world manager, object maintenance

262 retail functions: 18 done, 3 partial, 239 to do, 2 skipped.
Legend and workflow: [README](README.md).

## `portal/client/src/objects/cbookkeep.cpp.c` (5/22)

### CBookKeep
- [ ] `CBookKeep::CBookKeep` · 004d9b70 · legions 004cc5e0
- [x] `CBookKeep::HandleOrderedBlob` · 004d9c00 · legions 004cc670
- [ ] `CBookKeep::ChangeCell` · 004d9c50 · legions 004cc6c0
- [ ] `CBookKeep::GotoLostCell` · 004d9c80 · legions 004cc6f0
- [ ] `CBookKeep::RemoveFromLostCell` · 004d9ce0 · legions 004cc750
- [ ] `CBookKeep::HandleCreation` · 004d9d10 · legions 004cc780
- [ ] `CBookKeep::HandleDestruction` · 004d9d50 · legions 004cc7c0
- [ ] `CBookKeep::ReEnterVisibility` · 004d9d80 · legions 004cc7f0
- [ ] `CBookKeep::HasParent` · 004d9da0 · legions 004cc810
- [ ] `CBookKeep::SyncPointers` · 004d9dc0 · legions 004cc830
- [ ] `CBookKeep::~CBookKeep` · 004d9e20 · legions 004cc890
- [x] `CBookKeep::FlushVisualOrderedEvents` · 004d9e90 · legions 004cc900
- [x] `CBookKeep::Notice_CreateEventReceived` · 004d9f10 · legions 004cc980
- [x] `CBookKeep::FlushOrderedEvents` · 004d9f30 · legions 004cc9a0
- [ ] `CBookKeep::SetPhysObj` · 004d9f80 · legions 004cc9f0
- [ ] `CBookKeep::SetWeenieObj` · 004d9f90 · legions 004cca00
- [ ] `CBookKeep::AddObjectToBeDestroyed` · 004d9fa0 · legions 004cca10
- [ ] `CBookKeep::RemoveObjectToBeDestroyed` · 004da060 · legions 004ccad0
- [ ] `CBookKeep::HandleEnterWorld` · 004da0f0 · legions 004ccb60
- [ ] `CBookKeep::HandleLeaveWorld` · 004da120 · legions 004ccb90
- [ ] `CBookKeep::ObjectBeingDestructed` · 004da150 · legions 004ccbc0
- [x] `CBookKeep::HandleVisualOrderedBlob` · 004da200 · legions 004ccc70

## `portal/client/src/objects/clostcell.cpp.c` (0/4)

### CLostCell
- [ ] `CLostCell::CLostCell` · 004da230 · legions 004ccca0
- [ ] `CLostCell::~CLostCell` · 004da250 · legions 00668630
- [ ] `CLostCell::RemoveObject` · 004da260 · legions 004cccc0
- [ ] `CLostCell::AddObject` · 004da2a0 · legions 004ccd00

## `portal/client/src/objects/cobjmaint.cpp.c` (6/65)

### CObjectMaint
- [ ] `CObjectMaint::CreatePhysicsObj` · 004d4fb0 · legions 004c7930
- [ ] `CObjectMaint::GetWeenieObject` · 004d5000 · legions 004c7980
- [ ] `CObjectMaint::GetPlayerID` · 004d5030 · legions 004c79b0
- [ ] `CObjectMaint::RecvNotice_SetPlayerID` · 004d5060

### _STL::less<InstanceID>
- [ ] `_STL::less<InstanceID>::operator()` · 004d50b0 · legions 004c7a30

### CObjectMaint
- [ ] `CObjectMaint::GetPlayerWeenieObject` · 004d50e0 · legions 004c7a60
- [ ] `CObjectMaint::GetPlayerWeenieObjectReference` · 004d5130 · legions 004c7ab0

### DBObjGrabber<VisualDesc>
- [ ] `DBObjGrabber<VisualDesc>::DBObjGrabber<VisualDesc>` · 004d5190 · legions 004c7b50

### PriorityHash<InstanceID,double>
- [ ] `PriorityHash<InstanceID,double>::bubble_up` · 004d55d0 · legions 004c7fd0
- [ ] `PriorityHash<InstanceID,double>::bubble_down` · 004d5680 · legions 004c8130

### FXScalarAndTarget
- [x] `FXScalarAndTarget::StreamPack` · 004d5720 · legions 004c81d0

### PhysicsDesc
- [ ] `PhysicsDesc::get_position` · 004d5770 · legions 004c8220

### CObjectMaint
- [ ] `CObjectMaint::UpdateObject` · 004d57b0 · legions 004c8260

### PriorityHash<InstanceID,double>
- [ ] `PriorityHash<InstanceID,double>::remove_min` · 004d5920 · legions 004c83d0
- [ ] `PriorityHash<InstanceID,double>::bubble_up_or_down` · 004d59a0 · legions 004c8450

### CPhysicsObj
- [ ] `CPhysicsObj::is_newer` · 004d5a90 · legions 004c8540

### CObjectMaint
- [ ] `CObjectMaint::ClearCreationTable` · 004d5ae0 · legions 004c8590
- [ ] `CObjectMaint::UnRegisterAllNoticeHandlers` · 004d5b70 · legions 004c8620
- [ ] `CObjectMaint::IsObjectBeingCreated` · 004d5be0 · legions 004c8690

### PriorityHash<InstanceID,double>
- [ ] `PriorityHash<InstanceID,double>::remove` · 004d5c60 · legions 004c8770

### CObjectMaint
- [ ] `CObjectMaint::RemoveObjectToBeDestroyed` · 004d5da0 · legions 004c8860
- [ ] `CObjectMaint::ReleaseObjCell` · 004d5dd0 · legions 004c8890
- [ ] `CObjectMaint::FlushLostCellTable` · 004d5ec0 · legions 004c8980
- [ ] `CObjectMaint::OnContextFinished` · 004d61d0
- [ ] `CObjectMaint::InitObjCell` · 004d6210 · legions 004c8cd0
- [ ] `CObjectMaint::DumpToString` · 004d6280 · legions 004c8d40

### BehaviorParams
- [ ] `BehaviorParams::operator=` · 004d6970 · legions 004c93e0

### CObjectMaint
- [ ] `CObjectMaint::AddObject` · 004d6dc0 · legions 004c9830
- [ ] `CObjectMaint::GetObjectA` · 004d6df0
- [ ] `CObjectMaint::GetLostCell` · 004d6e20 · legions 004c9890
- [ ] `CObjectMaint::RemoveFromLostCell` · 004d6ea0 · legions 004c9910
- [ ] `CObjectMaint::CreateDummyObject` · 004d6f10 · legions 004c9980

### PriorityHash<InstanceID,double>
- [ ] `PriorityHash<InstanceID,double>::insert` · 004d7340 · legions 004c9db0

### SmartArray<BehaviorParams>
- [ ] `SmartArray<BehaviorParams>::operator=` · 004d7450 · legions 004c9ec0

### CObjectMaint
- [ ] `CObjectMaint::CreateObject` · 004d75a0 · legions 004ca010
- [ ] `CObjectMaint::AddObjectToBeDestroyed` · 004d76d0 · legions 004ca140
- [ ] `CObjectMaint::DestroyObject` · 004d7730 · legions 004ca1a0
- [ ] `CObjectMaint::FadeAndDestroyObject` · 004d7770 · legions 004ca1e0
- [ ] `CObjectMaint::GotoLostCell` · 004d77e0 · legions 004ca250
- [ ] `CObjectMaint::InitializeTemporaryObject` · 004d7840 · legions 004ca2b0
- [ ] `CObjectMaint::AllocateTemporaryID` · 004d7c10 · legions 004ca680

### _STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,SliderData>,_STL::_Select1st<_STL::pair<unsigned_long_const_,SliderData>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,SliderData>_>_>
- [ ] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,SliderData>,_STL::_Select1st<_STL::pair<unsigned_long_const_,SliderData>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,SliderData>_>_>::operator=` · 004d7d00 · legions 004ca770

### _STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,FXScalarAndTarget>,_STL::_Select1st<_STL::pair<unsigned_long_const_,FXScalarAndTarget>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,FXScalarAndTarget>_>_>
- [ ] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,FXScalarAndTarget>,_STL::_Select1st<_STL::pair<unsigned_long_const_,FXScalarAndTarget>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,FXScalarAndTarget>_>_>::operator=` · 004d7dc0 · legions 004ca830

### ObjectEventQueue
- [x] `ObjectEventQueue::push` · 004d7e80 · legions 004ca8f0

### CObjectMaint
- [ ] `CObjectMaint::DeallocateTemporaryID` · 004d7ef0 · legions 004ca960

### PhysicsDesc
- [ ] `PhysicsDesc::operator=` · 004d80e0 · legions 004cab50

### CObjectMaint
- [ ] `CObjectMaint::CreateTemporaryObject` · 004d8310 · legions 004cad80
- [ ] `CObjectMaint::DestroyEventQueue` · 004d8400 · legions 004cae70
- [ ] `CObjectMaint::StartEntityCreation` · 004d84a0 · legions 004caf10
- [ ] `CObjectMaint::DestroyObjectInternal` · 004d86f0 · legions 004cb160
- [ ] `CObjectMaint::HandlePendingDeletion` · 004d87b0 · legions 004cb220
- [x] `CObjectMaint::FlushEventQueueForBookie` · 004d8820 · legions 004cb290
- [ ] `CObjectMaint::FlushObjectTable` · 004d8940 · legions 004cb3b0
- [ ] `CObjectMaint::DestroyEventQueues` · 004d8a00 · legions 004cb470
- [~] `CObjectMaint::FinishEntityCreation` · 004d8a40 · legions 004cb4b0 — visual-queue release and queued-event replay only; no CPhysicsObj built
- [ ] `CObjectMaint::DestroyObjects` · 004d8c00 · legions 004cb670
- [ ] `CObjectMaint::CObjectMaint` · 004d8de0 · legions 004cb850
- [ ] `CObjectMaint::~CObjectMaint` · 004d8f30 · legions 004cb9a0
- [~] `CObjectMaint::FinishEntityCreation` · 004d9150 · legions 004cbbc0 — visual-queue release and queued-event replay only; no CPhysicsObj built
- [ ] `CObjectMaint::UseTime` · 004d9480 · legions 004cbef0

### _STL::map<InstanceID,ObjectEventQueue,_STL::less<InstanceID>,_STL::allocator<_STL::pair<InstanceID_const_,ObjectEventQueue>_>_>
- [ ] `_STL::map<InstanceID,ObjectEventQueue,_STL::less<InstanceID>,_STL::allocator<_STL::pair<InstanceID_const_,ObjectEventQueue>_>_>::operator[]` · 004d9770 · legions 004cc1e0

### CObjectMaint
- [x] `CObjectMaint::QueueEventForBookie` · 004d98b0 · legions 004cc320
- [x] `CObjectMaint::GetObjectOrQueueEvent` · 004d98e0 · legions 004cc350
- [~] `CObjectMaint::RecvEvt_CreateObject` · 004d99c0 · legions 004cc430 — creation is immediate (no async entity loading); parent/stamp/update rules kept
- [x] `CObjectMaint::RecvEvt_DestroyObject` · 004d9b00 · legions 004cc570

## `portal/client/src/objects/visualorderingqueue.cpp.c` (7/7)

### VisualOrderingQueue
- [x] `VisualOrderingQueue::IsHeavilyBlocked` · 004da2e0 · legions 004ccd40
- [x] `VisualOrderingQueue::DiscardEvents` · 004da320 · legions 004ccd80
- [x] `VisualOrderingQueue::VisualOrderingQueue` · 004da550 · legions 004ccfb0
- [x] `VisualOrderingQueue::HandleVisualOrderedEvent` · 004da590 · legions 004ccff0
- [x] `VisualOrderingQueue::ExtractNextEvent` · 004da5d0 · legions 004cd030
- [x] `VisualOrderingQueue::DiscardAllEvents` · 004da700 · legions 004cd160
- [x] `VisualOrderingQueue::~VisualOrderingQueue` · 004da7b0 · legions 004cd210

## `portal/client/src/smartbox/camera/camerafx.cpp.c` (0/10)

### CameraFX
- [ ] `CameraFX::SetDefaults` · 00577d00 · legions 00547ca0
- [ ] `CameraFX::GetDBOType` · 00577d70
- [ ] `CameraFX::get_pack_bitfield` · 00577d80 · legions 00547d20
- [ ] `CameraFX::CopyInto` · 00577e90
- [ ] `CameraFX::Destroy` · 00577f20
- [ ] `CameraFX::CameraFX` · 00577f30 · legions 00547ed0
- [ ] `CameraFX::StreamPack` · 00577f80
- [ ] `CameraFX::Allocate` · 00578270
- [-] `CameraFX::FromFileNode` · 00578280 — text source format (admin tools)
- [-] `CameraFX::ToFileNode` · 00578590 — text source format (admin tools)

## `portal/client/src/smartbox/camera/camerafxmanager.cpp.c` (0/10)

### CameraFxTrack
- [ ] `CameraFxTrack::CameraFxTrack` · 005788c0 · legions 00548220
- [ ] `CameraFxTrack::Update` · 00578910 · legions 00548270

### CameraFxManager
- [ ] `CameraFxManager::GetDistortion` · 00578a10 · legions 00548370
- [ ] `CameraFxManager::UseTime` · 00578d20 · legions 00548680
- [ ] `CameraFxManager::AddMotionBlur` · 00578ed0 · legions 00548830
- [ ] `CameraFxManager::AddScreenFlash` · 00578f50 · legions 005488b0
- [ ] `CameraFxManager::AddFovShift` · 00578fe0 · legions 00548940
- [ ] `CameraFxManager::AddShake` · 00579060 · legions 005489c0
- [ ] `CameraFxManager::AddCameraDistance` · 005790e0 · legions 00548a40
- [ ] `CameraFxManager::StopAllCameraFX` · 00579160 · legions 00548ac0

## `portal/client/src/smartbox/camera/cameramanager.cpp.c` (0/46)

### TargetingModule
- [ ] `TargetingModule::TargetingModule` · 00574850 · legions 00544720

### CameraManager
- [ ] `CameraManager::GetCameraManager` · 00574970 · legions 00544840
- [ ] `CameraManager::CalcAlignToSlope` · 00574980 · legions 00544850
- [ ] `CameraManager::UseTime` · 00574b50 · legions 00544a20
- [ ] `CameraManager::GetFirstPersonIsReady` · 00574c50 · legions 00544b20
- [ ] `CameraManager::CheckEnvOnly` · 00574ca0 · legions 00544b70
- [ ] `CameraManager::NoRayTrace` · 00574cc0 · legions 00544b90
- [ ] `CameraManager::ZoomIn` · 00574cf0 · legions 00544bc0
- [ ] `CameraManager::ZoomOut` · 00574d40 · legions 00544c10
- [ ] `CameraManager::SetParentObject` · 00574d80 · legions 00544c50
- [ ] `CameraManager::SetupCameraDummyObjects` · 00574db0 · legions 00544c80
- [ ] `CameraManager::ResetCamera` · 00574eb0 · legions 00544d80
- [ ] `CameraManager::SetFreeCamera` · 00574ff0 · legions 00544ec0
- [ ] `CameraManager::SetTargetedCamera` · 00575040 · legions 00544f10
- [ ] `CameraManager::RestorePlayerCamera` · 00575090 · legions 00544f60
- [ ] `CameraManager::GetGlobalDirection` · 005750d0 · legions 00544fa0
- [ ] `CameraManager::SetCameraAnimTarget` · 005750f0 · legions 00544fc0
- [ ] `CameraManager::SetCameraAnimParent` · 00575150 · legions 00545020
- [ ] `CameraManager::FakeTransition` · 005751b0 · legions 00546710
- [ ] `CameraManager::SetupPrefs` · 005751c0 · legions 00545090
- [ ] `CameraManager::StopAllCameraFX` · 00575a20

### CameraFxTrack
- [ ] `CameraFxTrack::operator=` · 00575a30 · legions 00545900

### CameraManager
- [ ] `CameraManager::GetCameraDistance` · 00575ab0 · legions 00545980
- [ ] `CameraManager::ShoulderView` · 00575ad0 · legions 005459a0
- [ ] `CameraManager::SetTargetObject` · 00575b40 · legions 00545a10
- [ ] `CameraManager::GetInterpViewerOffset` · 00575bc0 · legions 00545a90
- [ ] `CameraManager::CalcShoulderOffset` · 00575c00 · legions 00545ad0
- [ ] `CameraManager::HandleApply` · 00575ee0 · legions 00545db0
- [ ] `CameraManager::DoCameraFX` · 00575f20 · legions 00545df0
- [ ] `CameraManager::UpdateCamera` · 005760f0 · legions 00545fb0
- [ ] `CameraManager::LookAtSelf` · 005767a0 · legions 00546660
- [ ] `CameraManager::SwingOutLeft` · 00576850 · legions 005467f0
- [ ] `CameraManager::SwingOutRight` · 00576930
- [ ] `CameraManager::SetMapView` · 00576a10 · legions 005468d0
- [ ] `CameraManager::GetTargetOffset` · 00576b00 · legions 005469c0
- [ ] `CameraManager::SetDefaults` · 00576b40 · legions 00546a30
- [ ] `CameraManager::LookAtGround` · 00576c70 · legions 00546b60
- [ ] `CameraManager::SetMouseLookState` · 00576d10 · legions 00546c00
- [ ] `CameraManager::SetFirstPersonState` · 00576dc0 · legions 00546cb0
- [ ] `CameraManager::HandleChange` · 00576e30
- [ ] `CameraManager::StartAction` · 00577100 · legions 00546ff0
- [ ] `CameraManager::StopAction` · 00577360 · legions 00547250
- [ ] `CameraManager::CameraManager` · 005775a0 · legions 005475f0

### _STL::deque<_STL::pair<long,long>,_STL::allocator<_STL::pair<long,long>_>_>
- [ ] `_STL::deque<_STL::pair<long,long>,_STL::allocator<_STL::pair<long,long>_>_>::~deque<_STL::pair<long,long>,_STL::allocator<_STL::pair<long,long>_>_>` · 00577720 · legions 005752f0

### CameraManager
- [ ] `CameraManager::~CameraManager` · 00577840 · legions 005477e0
- [ ] `CameraManager::MouseMove` · 005778a0 · legions 00547840

## `portal/client/src/smartbox/camera/targetingmodule.cpp.c` (0/11)

### TargetingModule
- [ ] `TargetingModule::SetPlayer` · 00579180 · legions 00548ae0
- [ ] `TargetingModule::TurnOn` · 005791a0 · legions 00548b00
- [ ] `TargetingModule::CalcCameraDistance` · 00579210 · legions 00548b70
- [ ] `TargetingModule::GetInterpViewerOffset` · 00579420 · legions 00548d80
- [ ] `TargetingModule::FakeTransition` · 005794b0 · legions 00548e10
- [ ] `TargetingModule::GetPosition` · 00579500 · legions 00548e60
- [ ] `TargetingModule::StoreTargetPos` · 00579670 · legions 00548fd0
- [ ] `TargetingModule::StoreParentPos` · 00579700 · legions 00549060
- [ ] `TargetingModule::Update` · 00579780 · legions 005490e0
- [ ] `TargetingModule::InternalCalcViewerPos` · 00579910 · legions 00549270
- [ ] `TargetingModule::CalcViewerPos` · 0057a320 · legions 00549c80

## `portal/client/src/smartbox/smartbox.cpp.c` (0/30)

### Position
- [ ] `Position::operator!=` · 005734b0 · legions 005434b0

### SmartBox
- [ ] `SmartBox::RefreshGraphicsResources` · 005734f0
- [ ] `SmartBox::GetDisplayWidth` · 00573510 · legions 00543510
- [ ] `SmartBox::GetDisplayHeight` · 00573530 · legions 00543530
- [ ] `SmartBox::EnterPortalSpace` · 00573550 · legions 00543550
- [ ] `SmartBox::ExitPortalSpace` · 00573560 · legions 00543560
- [ ] `SmartBox::InPortalSpace` · 00573570 · legions 00543570
- [ ] `SmartBox::RenderScreenshot` · 00573580 · legions 00543580
- [ ] `SmartBox::SetCameraPosition` · 005735e0 · legions 005435e0
- [ ] `SmartBox::SetCameraDefaults` · 00573630 · legions 00543630
- [ ] `SmartBox::RecvNotice_SetRegionID` · 005736e0 · legions 005436e0
- [ ] `SmartBox::SetWantNoticeOnNextFrameDrawn` · 005736f0 · legions 005436f0
- [ ] `SmartBox::GetWantNoticeOnNextFrameDrawn` · 00573700 · legions 00543700
- [ ] `SmartBox::SmartBox` · 00573710 · legions 00543710
- [ ] `SmartBox::IsVisible` · 005737e0 · legions 005437e0
- [ ] `SmartBox::Show` · 005737f0 · legions 005437f0
- [ ] `SmartBox::Hide` · 00573800 · legions 00543800
- [ ] `SmartBox::InitPreferences` · 00573810 · legions 00543810
- [ ] `SmartBox::UpdatePlayer` · 00573930 · legions 00543930
- [ ] `SmartBox::UpdateRenderTargets` · 005739f0 · legions 005439f0
- [ ] `SmartBox::Render` · 00573a10
- [ ] `SmartBox::ResetWorldState` · 00573a60 · legions 00543a30
- [ ] `SmartBox::InitPlayer` · 00573b50 · legions 00543b20
- [ ] `SmartBox::UpdateCamera` · 00573bd0 · legions 00543ba0
- [ ] `SmartBox::RecvNotice_CreatePlayerObject` · 00574220 · legions 00544170
- [ ] `SmartBox::PlayerPhysicsUpdatedCallback` · 00574280 · legions 005441d0
- [ ] `SmartBox::HandleTeleport` · 005742d0 · legions 00544220
- [ ] `SmartBox::~SmartBox` · 005743e0 · legions 00544330
- [ ] `SmartBox::Init` · 005744f0 · legions 00544420
- [ ] `SmartBox::UseTime` · 005746d0 · legions 005445d0

## `portal/client/src/smartbox/worldmanager/cellmanager.cpp.c` (0/14)

### CellManager
- [ ] `CellManager::~CellManager` · 0057ba00 · legions 0054b2e0
- [ ] `CellManager::OnContextFinished` · 0057ba10 · legions 0054b2f0
- [ ] `CellManager::ReleaseCells` · 0057ba40 · legions 0054b320
- [ ] `CellManager::SetFarRadius` · 0057baa0 · legions 0054b380
- [ ] `CellManager::NotifyChangeRegion` · 0057bac0 · legions 0054b3a0

### Position
- [ ] `Position::get_outside_cell_id` · 0057bad0 · legions 0054b3b0

### CellManager
- [ ] `CellManager::CellManager` · 0057bb10 · legions 0054b3f0
- [ ] `CellManager::RequestAsyncLoadBlockRadius` · 0057bba0 · legions 0054b480
- [ ] `CellManager::RequestAsyncLoadBlockRadiusSceneObjects` · 0057bcd0 · legions 0054b5b0
- [ ] `CellManager::ResetWorldState` · 0057be70 · legions 0054b750
- [ ] `CellManager::FinishUpdateVisibleCells` · 0057be90 · legions 0054b770
- [ ] `CellManager::AsyncGetCells` · 0057bf80 · legions 0054b850
- [ ] `CellManager::OnTopLevelRequestFinished` · 0057c040 · legions 0054b910
- [ ] `CellManager::UpdateVisibleCells` · 0057c360 · legions 0054bc30

## `portal/client/src/smartbox/worldmanager/lscapemanager.cpp.c` (0/21)

### LScapeManager
- [ ] `LScapeManager::GetBlockShift` · 0057a510 · legions 00549e70
- [ ] `LScapeManager::SetFarRadius` · 0057a590 · legions 00549ef0
- [ ] `LScapeManager::GrabVisibleCells` · 0057a5b0 · legions 00549f10
- [ ] `LScapeManager::ReleaseVisibleCells` · 0057a640 · legions 00549fa0
- [ ] `LScapeManager::Init` · 0057a780 · legions 0054a060
- [ ] `LScapeManager::ReloadLandBlocks` · 0057a7b0 · legions 0054a090
- [ ] `LScapeManager::InitFreeBlocks` · 0057a810 · legions 0054a0f0
- [ ] `LScapeManager::CleanupFreeBlocks` · 0057a880 · legions 0054a160
- [ ] `LScapeManager::ResetWorldState` · 0057a8d0 · legions 0054a1b0
- [ ] `LScapeManager::NotifyChangeRegion` · 0057a8e0 · legions 0054a1c0
- [ ] `LScapeManager::Refresh` · 0057a980 · legions 0054a260
- [ ] `LScapeManager::ReleaseBlock` · 0057a9f0 · legions 0054a2d0
- [ ] `LScapeManager::GetLandBlock` · 0057aa60 · legions 0054a340
- [ ] `LScapeManager::ReleaseCells` · 0057ab30 · legions 0054a410
- [ ] `LScapeManager::SetVertexValue` · 0057ac20 · legions 0054a500
- [ ] `LScapeManager::GetFreeBlock` · 0057af50 · legions 0054a830
- [ ] `LScapeManager::~LScapeManager` · 0057b1b0 · legions 0054aa90
- [ ] `LScapeManager::AddBlockToRefreshList` · 0057b260 · legions 0054ab40
- [ ] `LScapeManager::UpdateBlocks` · 0057b270 · legions 0054ab50
- [ ] `LScapeManager::LScapeManager` · 0057b8f0 · legions 0054b1d0
- [ ] `LScapeManager::UpdateVisibleCells` · 0057b980 · legions 0054b260

## `portal/client/src/smartbox/worldmanager/worldmanager.cpp.c` (0/9)

### WorldManager
- [ ] `WorldManager::WorldManager` · 0057c420 · legions 0054bcf0
- [ ] `WorldManager::Init` · 0057c440 · legions 0054bd10
- [ ] `WorldManager::IsBlockingForCells` · 0057c500 · legions 0054bdd0
- [ ] `WorldManager::Update` · 0057c510 · legions 0054bde0
- [ ] `WorldManager::UpdateVisibleCells` · 0057c540 · legions 0054be10
- [ ] `WorldManager::SetRegion` · 0057c570 · legions 0054be40
- [ ] `WorldManager::Refresh` · 0057c5e0 · legions 0054beb0
- [ ] `WorldManager::~WorldManager` · 0057c610 · legions 0054bee0
- [ ] `WorldManager::ResetWorldState` · 0057c6b0 · legions 0054bf80

## `portal/shared/src/objects/iidutils.cpp.c` (0/13)

### IIDUtils
- [ ] `IIDUtils::CreateUI` · 006bc300 · legions 006512a0
- [ ] `IIDUtils::CreateTemporary` · 006bc340 · legions 006512e0
- [ ] `IIDUtils::GetType` · 006bc380 · legions 00651320
- [ ] `IIDUtils::VerifyPlayer` · 006bc410 · legions 006513b0
- [ ] `IIDUtils::VerifyUI` · 006bc430 · legions 006513d0
- [ ] `IIDUtils::VerifyCamera` · 006bc450 · legions 006513f0
- [ ] `IIDUtils::VerifyTemporary` · 006bc470 · legions 00651410
- [ ] `IIDUtils::VerifyStatic` · 006bc490 · legions 00651430
- [ ] `IIDUtils::VerifyDynamic` · 006bc4c0 · legions 00651460
- [ ] `IIDUtils::ReadBlock` · 006bc4e0 · legions 00651480
- [ ] `IIDUtils::GetCameraParent` · 006bc4f0 · legions 00651490
- [ ] `IIDUtils::GetCameraTarget` · 006bc510 · legions 006514b0
- [ ] `IIDUtils::CreateStatic` · 006bc530 · legions 006514d0
