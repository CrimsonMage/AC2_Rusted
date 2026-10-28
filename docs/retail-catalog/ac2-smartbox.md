# ac2-smartbox: SmartBox, camera, world manager, object maintenance

262 retail functions: 94 done, 36 partial, 86 to do, 46 skipped.
Legend and workflow: [README](README.md).

## `portal/client/src/objects/cbookkeep.cpp.c` (12/22)

### CBookKeep
- [-] `CBookKeep::CBookKeep` · 004d9b70 · legions 004cc5e0 — ctor/dtor/operator= or plain data copy (Rust new/Drop/Clone/derive replaces)
- [x] `CBookKeep::HandleOrderedBlob` · 004d9c00 · legions 004cc670
- [ ] `CBookKeep::ChangeCell` · 004d9c50 · legions 004cc6c0 — needs port (check): bookie cell change / creation / parent / destruction; ac2-weenie world.rs has enter, leave and lost-cell bookkeeping but no citation here
- [x] `CBookKeep::GotoLostCell` · 004d9c80 · legions 004cc6f0
- [x] `CBookKeep::RemoveFromLostCell` · 004d9ce0 · legions 004cc750
- [ ] `CBookKeep::HandleCreation` · 004d9d10 · legions 004cc780 — needs port (check): bookie cell change / creation / parent / destruction; ac2-weenie world.rs has enter, leave and lost-cell bookkeeping but no citation here
- [~] `CBookKeep::HandleDestruction` · 004d9d50 · legions 004cc7c0 — the pending deletion only
- [x] `CBookKeep::ReEnterVisibility` · 004d9d80 · legions 004cc7f0
- [ ] `CBookKeep::HasParent` · 004d9da0 · legions 004cc810 — needs port (check): bookie cell change / creation / parent / destruction; ac2-weenie world.rs has enter, leave and lost-cell bookkeeping but no citation here
- [-] `CBookKeep::SyncPointers` · 004d9dc0 · legions 004cc830 — raw back-pointer maintenance or notice-handler teardown; Rust keeps ids and drops owners
- [-] `CBookKeep::~CBookKeep` · 004d9e20 · legions 004cc890 — ctor/dtor/operator= or plain data copy (Rust new/Drop/Clone/derive replaces)
- [x] `CBookKeep::FlushVisualOrderedEvents` · 004d9e90 · legions 004cc900
- [x] `CBookKeep::Notice_CreateEventReceived` · 004d9f10 · legions 004cc980
- [x] `CBookKeep::FlushOrderedEvents` · 004d9f30 · legions 004cc9a0
- [-] `CBookKeep::SetPhysObj` · 004d9f80 · legions 004cc9f0 — raw back-pointer maintenance or notice-handler teardown; Rust keeps ids and drops owners
- [-] `CBookKeep::SetWeenieObj` · 004d9f90 · legions 004cca00 — raw back-pointer maintenance or notice-handler teardown; Rust keeps ids and drops owners
- [x] `CBookKeep::AddObjectToBeDestroyed` · 004d9fa0 · legions 004cca10 — OnlineSession::add_object_to_be_destroyed (children too)
- [x] `CBookKeep::RemoveObjectToBeDestroyed` · 004da060 · legions 004ccad0
- [x] `CBookKeep::HandleEnterWorld` · 004da0f0 · legions 004ccb60
- [x] `CBookKeep::HandleLeaveWorld` · 004da120 · legions 004ccb90
- [~] `CBookKeep::ObjectBeingDestructed` · 004da150 · legions 004ccbc0 — CBookKeep::ObjectBeingDestructed (004da150): the destroyed object's children are queued for destruction in ac2-weenie world.rs destroy_object_internal (add_object_to_be_destroyed, 25 s). Not modelled: the physobj child detach (vtable 0x3f4) and parent teardown (0x160). CWInterface::ObjectBeingDestructed (0052e3f0) also purges ClientTriggerScheduler events, the ac2-client row ClientTriggerScheduler::PurgeEventsForInterface, not ported
- [x] `CBookKeep::HandleVisualOrderedBlob` · 004da200 · legions 004ccc70

## `portal/client/src/objects/clostcell.cpp.c` (2/4)

### CLostCell
- [-] `CLostCell::CLostCell` · 004da230 · legions 004ccca0 — ctor/dtor/operator= or plain data copy (Rust new/Drop/Clone/derive replaces)
- [-] `CLostCell::~CLostCell` · 004da250 · legions 00668630 — ctor/dtor/operator= or plain data copy (Rust new/Drop/Clone/derive replaces)
- [x] `CLostCell::RemoveObject` · 004da260 · legions 004cccc0
- [x] `CLostCell::AddObject` · 004da2a0 · legions 004ccd00

## `portal/client/src/objects/cobjmaint.cpp.c` (20/65)

### CObjectMaint
- [x] `CObjectMaint::CreatePhysicsObj` · 004d4fb0 · legions 004c7930 — crates/ac2-weenie/src/appearance.rs:4 (CObjectMaint::CreatePhysicsObj applies the create's delta)
- [x] `CObjectMaint::GetWeenieObject` · 004d5000 · legions 004c7980 — crates/ac2-script/src/host/interface.rs:299 (CObjectMaint::GetWeenieObject, undefined when none)
- [x] `CObjectMaint::GetPlayerID` · 004d5030 · legions 004c79b0 — crates/ac2-script/src/host/glue.rs:161 (CObjectMaint::GetPlayerID: the player's id for the efun)
- [ ] `CObjectMaint::RecvNotice_SetPlayerID` · 004d5060 — needs port (check): player id / player weenie lookup and PhysicsDesc position; uncited

### _STL::less<InstanceID>
- [-] `_STL::less<InstanceID>::operator()` · 004d50b0 · legions 004c7a30 — STL less/map/rb-tree/deque internals (Rust std replaces)

### CObjectMaint
- [ ] `CObjectMaint::GetPlayerWeenieObject` · 004d50e0 · legions 004c7a60 — needs port (check): player id / player weenie lookup and PhysicsDesc position; uncited
- [x] `CObjectMaint::GetPlayerWeenieObjectReference` · 004d5130 · legions 004c7ab0 — crates/ac2-script/src/host/glue.rs:171 (CObjectMaint::GetPlayerWeenieObjectReference: the player's script instance)

### DBObjGrabber<VisualDesc>
- [-] `DBObjGrabber<VisualDesc>::DBObjGrabber<VisualDesc>` · 004d5190 · legions 004c7b50 — ctor/dtor/operator= or plain data copy (Rust new/Drop/Clone/derive replaces)

### PriorityHash<InstanceID,double>
- [-] `PriorityHash<InstanceID,double>::bubble_up` · 004d55d0 · legions 004c7fd0 — PriorityHash heap sift internals (Rust ordered structure in ac2-weenie destruction.rs)
- [-] `PriorityHash<InstanceID,double>::bubble_down` · 004d5680 · legions 004c8130 — PriorityHash heap sift internals (Rust ordered structure in ac2-weenie destruction.rs)

### FXScalarAndTarget
- [x] `FXScalarAndTarget::StreamPack` · 004d5720 · legions 004c81d0

### PhysicsDesc
- [ ] `PhysicsDesc::get_position` · 004d5770 · legions 004c8220 — needs port (check): player id / player weenie lookup and PhysicsDesc position; uncited

### CObjectMaint
- [~] `CObjectMaint::UpdateObject` · 004d57b0 · legions 004c8260 — SyncPositionSimple(pos, 1) for the same instance

### PriorityHash<InstanceID,double>
- [ ] `PriorityHash<InstanceID,double>::remove_min` · 004d5920 · legions 004c83d0 — needs port (check): PriorityHash remove_min / remove (destruction queue); ac2-weenie destruction.rs has take_due but no citation
- [-] `PriorityHash<InstanceID,double>::bubble_up_or_down` · 004d59a0 · legions 004c8450 — PriorityHash heap sift internals (Rust ordered structure in ac2-weenie destruction.rs)

### CPhysicsObj
- [x] `CPhysicsObj::is_newer` · 004d5a90 · legions 004c8540

### CObjectMaint
- [ ] `CObjectMaint::ClearCreationTable` · 004d5ae0 · legions 004c8590 — needs port (check): object table, creation, temporary ids, event queues and destruction (CObjectMaint); ac2-weenie world.rs covers parts but none is cited here
- [-] `CObjectMaint::UnRegisterAllNoticeHandlers` · 004d5b70 · legions 004c8620 — raw back-pointer maintenance or notice-handler teardown; Rust keeps ids and drops owners
- [ ] `CObjectMaint::IsObjectBeingCreated` · 004d5be0 · legions 004c8690 — needs port (check): object table, creation, temporary ids, event queues and destruction (CObjectMaint); ac2-weenie world.rs covers parts but none is cited here

### PriorityHash<InstanceID,double>
- [ ] `PriorityHash<InstanceID,double>::remove` · 004d5c60 · legions 004c8770 — needs port (check): PriorityHash remove_min / remove (destruction queue); ac2-weenie destruction.rs has take_due but no citation

### CObjectMaint
- [x] `CObjectMaint::RemoveObjectToBeDestroyed` · 004d5da0 · legions 004c8860
- [~] `CObjectMaint::ReleaseObjCell` · 004d5dd0 · legions 004c8890 — for every cell of a released block: objects in the world, unless held, leave visibility
- [ ] `CObjectMaint::FlushLostCellTable` · 004d5ec0 · legions 004c8980 — needs port (check): player id / player weenie lookup and PhysicsDesc position; uncited
- [ ] `CObjectMaint::OnContextFinished` · 004d61d0 — needs port (check): object table, creation, temporary ids, event queues and destruction (CObjectMaint); ac2-weenie world.rs covers parts but none is cited here
- [x] `CObjectMaint::InitObjCell` · 004d6210 · legions 004c8cd0 — ac2-client remote.rs, for every cell of a block made resident
- [-] `CObjectMaint::DumpToString` · 004d6280 · legions 004c8d40 — CObjectMaint::DumpToString: debug dump

### BehaviorParams
- [-] `BehaviorParams::operator=` · 004d6970 · legions 004c93e0 — ctor/dtor/operator= or plain data copy (Rust new/Drop/Clone/derive replaces)

### CObjectMaint
- [ ] `CObjectMaint::AddObject` · 004d6dc0 · legions 004c9830 — needs port (check): object table, creation, temporary ids, event queues and destruction (CObjectMaint); ac2-weenie world.rs covers parts but none is cited here
- [ ] `CObjectMaint::GetObjectA` · 004d6df0 — needs port (check): object table, creation, temporary ids, event queues and destruction (CObjectMaint); ac2-weenie world.rs covers parts but none is cited here
- [x] `CObjectMaint::GetLostCell` · 004d6e20 · legions 004c9890 — ac2-client remote.rs
- [x] `CObjectMaint::RemoveFromLostCell` · 004d6ea0 · legions 004c9910 — ac2-client remote.rs
- [ ] `CObjectMaint::CreateDummyObject` · 004d6f10 · legions 004c9980 — needs port (check): object table, creation, temporary ids, event queues and destruction (CObjectMaint); ac2-weenie world.rs covers parts but none is cited here

### PriorityHash<InstanceID,double>
- [x] `PriorityHash<InstanceID,double>::insert` · 004d7340 · legions 004c9db0 — crates/ac2-weenie/src/destruction.rs:21 (PriorityHash::insert: a second insert replaces its time)

### SmartArray<BehaviorParams>
- [-] `SmartArray<BehaviorParams>::operator=` · 004d7450 · legions 004c9ec0 — ctor/dtor/operator= or plain data copy (Rust new/Drop/Clone/derive replaces)

### CObjectMaint
- [ ] `CObjectMaint::CreateObject` · 004d75a0 · legions 004ca010 — needs port (check): object table, creation, temporary ids, event queues and destruction (CObjectMaint); ac2-weenie world.rs covers parts but none is cited here
- [~] `CObjectMaint::AddObjectToBeDestroyed` · 004d76d0 · legions 004ca140 — 25 s; no temporary ids (0.25 s)
- [x] `CObjectMaint::DestroyObject` · 004d7730 · legions 004ca1a0
- [x] `CObjectMaint::FadeAndDestroyObject` · 004d7770 · legions 004ca1e0 — ac2-weenie World::fade_and_destroy_object (pending deletion at cur_time + destruction, only for an object in the table) with the fade slider in ac2-smartbox missile_follow.rs; FX objects keep their own FxObjects::fade_and_destroy (0.5, 3.0)
- [x] `CObjectMaint::GotoLostCell` · 004d77e0 · legions 004ca250 — ac2-client remote.rs (lost-cell table)
- [ ] `CObjectMaint::InitializeTemporaryObject` · 004d7840 · legions 004ca2b0 — needs port (check): object table, creation, temporary ids, event queues and destruction (CObjectMaint); ac2-weenie world.rs covers parts but none is cited here
- [ ] `CObjectMaint::AllocateTemporaryID` · 004d7c10 · legions 004ca680 — needs port (check): object table, creation, temporary ids, event queues and destruction (CObjectMaint); ac2-weenie world.rs covers parts but none is cited here

### _STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,SliderData>,_STL::_Select1st<_STL::pair<unsigned_long_const_,SliderData>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,SliderData>_>_>
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,SliderData>,_STL::_Select1st<_STL::pair<unsigned_long_const_,SliderData>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,SliderData>_>_>::operator=` · 004d7d00 · legions 004ca770 — STL less/map/rb-tree/deque internals (Rust std replaces)

### _STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,FXScalarAndTarget>,_STL::_Select1st<_STL::pair<unsigned_long_const_,FXScalarAndTarget>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,FXScalarAndTarget>_>_>
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,FXScalarAndTarget>,_STL::_Select1st<_STL::pair<unsigned_long_const_,FXScalarAndTarget>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,FXScalarAndTarget>_>_>::operator=` · 004d7dc0 · legions 004ca830 — STL less/map/rb-tree/deque internals (Rust std replaces)

### ObjectEventQueue
- [x] `ObjectEventQueue::push` · 004d7e80 · legions 004ca8f0

### CObjectMaint
- [ ] `CObjectMaint::DeallocateTemporaryID` · 004d7ef0 · legions 004ca960 — needs port (check): object table, creation, temporary ids, event queues and destruction (CObjectMaint); ac2-weenie world.rs covers parts but none is cited here

### PhysicsDesc
- [-] `PhysicsDesc::operator=` · 004d80e0 · legions 004cab50 — ctor/dtor/operator= or plain data copy (Rust new/Drop/Clone/derive replaces)

### CObjectMaint
- [ ] `CObjectMaint::CreateTemporaryObject` · 004d8310 · legions 004cad80 — needs port (check): object table, creation, temporary ids, event queues and destruction (CObjectMaint); ac2-weenie world.rs covers parts but none is cited here
- [ ] `CObjectMaint::DestroyEventQueue` · 004d8400 · legions 004cae70 — needs port (check): object table, creation, temporary ids, event queues and destruction (CObjectMaint); ac2-weenie world.rs covers parts but none is cited here
- [ ] `CObjectMaint::StartEntityCreation` · 004d84a0 · legions 004caf10 — needs port (check): object table, creation, temporary ids, event queues and destruction (CObjectMaint); ac2-weenie world.rs covers parts but none is cited here
- [~] `CObjectMaint::DestroyObjectInternal` · 004d86f0 · legions 004cb160 — table, both queues and the event queue; no weenie or physics teardown hooks
- [x] `CObjectMaint::HandlePendingDeletion` · 004d87b0 · legions 004cb220
- [x] `CObjectMaint::FlushEventQueueForBookie` · 004d8820 · legions 004cb290
- [ ] `CObjectMaint::FlushObjectTable` · 004d8940 · legions 004cb3b0 — needs port (check): object table, creation, temporary ids, event queues and destruction (CObjectMaint); ac2-weenie world.rs covers parts but none is cited here
- [ ] `CObjectMaint::DestroyEventQueues` · 004d8a00 · legions 004cb470 — needs port (check): CObjectMaint::DestroyEventQueues (event queue teardown); uncited
- [~] `CObjectMaint::FinishEntityCreation` · 004d8a40 · legions 004cb4b0 — ac2-client: the physics object once its Setup loads on a loader thread, then SyncPositionSimple(pos, 1); the table entry is created on receipt
- [ ] `CObjectMaint::DestroyObjects` · 004d8c00 · legions 004cb670 — needs port (check): object table, creation, temporary ids, event queues and destruction (CObjectMaint); ac2-weenie world.rs covers parts but none is cited here
- [-] `CObjectMaint::CObjectMaint` · 004d8de0 · legions 004cb850 — ctor/dtor/operator= or plain data copy (Rust new/Drop/Clone/derive replaces)
- [-] `CObjectMaint::~CObjectMaint` · 004d8f30 · legions 004cb9a0 — ctor/dtor/operator= or plain data copy (Rust new/Drop/Clone/derive replaces)
- [~] `CObjectMaint::FinishEntityCreation` · 004d9150 · legions 004cbbc0 — ac2-client: the physics object once its Setup loads on a loader thread, then SyncPositionSimple(pos, 1); the table entry is created on receipt
- [~] `CObjectMaint::UseTime` · 004d9480 · legions 004cbef0 — destruction queue to pending deletions, the event-queue timeouts, HandlePendingDeletion; creation is not deferred to it (no temporary objects)

### _STL::map<InstanceID,ObjectEventQueue,_STL::less<InstanceID>,_STL::allocator<_STL::pair<InstanceID_const_,ObjectEventQueue>_>_>
- [-] `_STL::map<InstanceID,ObjectEventQueue,_STL::less<InstanceID>,_STL::allocator<_STL::pair<InstanceID_const_,ObjectEventQueue>_>_>::operator[]` · 004d9770 · legions 004cc1e0 — STL less/map/rb-tree/deque internals (Rust std replaces)

### CObjectMaint
- [x] `CObjectMaint::QueueEventForBookie` · 004d98b0 · legions 004cc320
- [x] `CObjectMaint::GetObjectOrQueueEvent` · 004d98e0 · legions 004cc350
- [~] `CObjectMaint::RecvEvt_CreateObject` · 004d99c0 · legions 004cc430 — creation is immediate (no async entity loading); parent/stamp/update rules kept
- [x] `CObjectMaint::RecvEvt_DestroyObject` · 004d9b00 · legions 004cc570 — ac2-weenie World: a pending deletion at cur_time

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
- [ ] `CameraFX::SetDefaults` · 00577d00 · legions 00547ca0 — needs port: camera effects (CameraFX DBO and CameraFxManager: shake, screen flash, fov shift, motion blur, distortion, camera distance); not in Rust
- [ ] `CameraFX::GetDBOType` · 00577d70 — needs port: camera effects (CameraFX DBO and CameraFxManager: shake, screen flash, fov shift, motion blur, distortion, camera distance); not in Rust
- [ ] `CameraFX::get_pack_bitfield` · 00577d80 · legions 00547d20 — needs port: camera effects (CameraFX DBO and CameraFxManager: shake, screen flash, fov shift, motion blur, distortion, camera distance); not in Rust
- [-] `CameraFX::CopyInto` · 00577e90 — ctor/dtor/operator= or plain data copy (Rust new/Drop/Clone/derive replaces)
- [-] `CameraFX::Destroy` · 00577f20 — ctor/dtor/operator= or plain data copy (Rust new/Drop/Clone/derive replaces)
- [-] `CameraFX::CameraFX` · 00577f30 · legions 00547ed0 — ctor/dtor/operator= or plain data copy (Rust new/Drop/Clone/derive replaces)
- [ ] `CameraFX::StreamPack` · 00577f80 — needs port: camera effects (CameraFX DBO and CameraFxManager: shake, screen flash, fov shift, motion blur, distortion, camera distance); not in Rust
- [ ] `CameraFX::Allocate` · 00578270 — needs port: camera effects (CameraFX DBO and CameraFxManager: shake, screen flash, fov shift, motion blur, distortion, camera distance); not in Rust
- [-] `CameraFX::FromFileNode` · 00578280 — text source format (admin tools)
- [-] `CameraFX::ToFileNode` · 00578590 — text source format (admin tools)

## `portal/client/src/smartbox/camera/camerafxmanager.cpp.c` (0/10)

### CameraFxTrack
- [ ] `CameraFxTrack::CameraFxTrack` · 005788c0 · legions 00548220 — needs port: camera effects (CameraFX DBO and CameraFxManager: shake, screen flash, fov shift, motion blur, distortion, camera distance); not in Rust
- [ ] `CameraFxTrack::Update` · 00578910 · legions 00548270 — needs port: camera effects (CameraFX DBO and CameraFxManager: shake, screen flash, fov shift, motion blur, distortion, camera distance); not in Rust

### CameraFxManager
- [ ] `CameraFxManager::GetDistortion` · 00578a10 · legions 00548370 — needs port: camera effects (CameraFX DBO and CameraFxManager: shake, screen flash, fov shift, motion blur, distortion, camera distance); not in Rust
- [ ] `CameraFxManager::UseTime` · 00578d20 · legions 00548680 — needs port: camera effects (CameraFX DBO and CameraFxManager: shake, screen flash, fov shift, motion blur, distortion, camera distance); not in Rust
- [ ] `CameraFxManager::AddMotionBlur` · 00578ed0 · legions 00548830 — needs port: camera effects (CameraFX DBO and CameraFxManager: shake, screen flash, fov shift, motion blur, distortion, camera distance); not in Rust
- [ ] `CameraFxManager::AddScreenFlash` · 00578f50 · legions 005488b0 — needs port: camera effects (CameraFX DBO and CameraFxManager: shake, screen flash, fov shift, motion blur, distortion, camera distance); not in Rust
- [ ] `CameraFxManager::AddFovShift` · 00578fe0 · legions 00548940 — needs port: camera effects (CameraFX DBO and CameraFxManager: shake, screen flash, fov shift, motion blur, distortion, camera distance); not in Rust
- [ ] `CameraFxManager::AddShake` · 00579060 · legions 005489c0 — needs port: camera effects (CameraFX DBO and CameraFxManager: shake, screen flash, fov shift, motion blur, distortion, camera distance); not in Rust
- [ ] `CameraFxManager::AddCameraDistance` · 005790e0 · legions 00548a40 — needs port: camera effects (CameraFX DBO and CameraFxManager: shake, screen flash, fov shift, motion blur, distortion, camera distance); not in Rust
- [ ] `CameraFxManager::StopAllCameraFX` · 00579160 · legions 00548ac0 — needs port: camera effects (CameraFX DBO and CameraFxManager: shake, screen flash, fov shift, motion blur, distortion, camera distance); not in Rust

## `portal/client/src/smartbox/camera/cameramanager.cpp.c` (19/46)

### TargetingModule
- [x] `TargetingModule::TargetingModule` · 00574850 · legions 00544720

### CameraManager
- [-] `CameraManager::GetCameraManager` · 00574970 · legions 00544840 — CameraManager singleton accessor (Rust owns the camera struct)
- [x] `CameraManager::CalcAlignToSlope` · 00574980 · legions 00544850 — ac2-client camera.rs
- [~] `CameraManager::UseTime` · 00574b50 · legions 00544a20 — field of view 65; no camera effects (distortion, blur, flash)
- [x] `CameraManager::GetFirstPersonIsReady` · 00574c50 · legions 00544b20
- [ ] `CameraManager::CheckEnvOnly` · 00574ca0 · legions 00544b70 — needs port: camera rig (CheckEnvOnly, parent/dummy objects, free/targeted camera, restore, anim target/parent, interp viewer and target offsets); ac2-smartbox camera.rs cites other CameraManager methods, not these
- [x] `CameraManager::NoRayTrace` · 00574cc0 · legions 00544b90
- [x] `CameraManager::ZoomIn` · 00574cf0 · legions 00544bc0 — camera action 0x4000002D
- [x] `CameraManager::ZoomOut` · 00574d40 · legions 00544c10 — camera action 0x400000B2
- [ ] `CameraManager::SetParentObject` · 00574d80 · legions 00544c50 — needs port: camera rig (CheckEnvOnly, parent/dummy objects, free/targeted camera, restore, anim target/parent, interp viewer and target offsets); ac2-smartbox camera.rs cites other CameraManager methods, not these
- [ ] `CameraManager::SetupCameraDummyObjects` · 00574db0 · legions 00544c80 — needs port: camera rig (CheckEnvOnly, parent/dummy objects, free/targeted camera, restore, anim target/parent, interp viewer and target offsets); ac2-smartbox camera.rs cites other CameraManager methods, not these
- [~] `CameraManager::ResetCamera` · 00574eb0 · legions 00544d80 — the player target only
- [ ] `CameraManager::SetFreeCamera` · 00574ff0 · legions 00544ec0 — needs port: camera rig (CheckEnvOnly, parent/dummy objects, free/targeted camera, restore, anim target/parent, interp viewer and target offsets); ac2-smartbox camera.rs cites other CameraManager methods, not these
- [ ] `CameraManager::SetTargetedCamera` · 00575040 · legions 00544f10 — needs port: camera rig (CheckEnvOnly, parent/dummy objects, free/targeted camera, restore, anim target/parent, interp viewer and target offsets); ac2-smartbox camera.rs cites other CameraManager methods, not these
- [ ] `CameraManager::RestorePlayerCamera` · 00575090 · legions 00544f60 — needs port: camera rig (CheckEnvOnly, parent/dummy objects, free/targeted camera, restore, anim target/parent, interp viewer and target offsets); ac2-smartbox camera.rs cites other CameraManager methods, not these
- [x] `CameraManager::GetGlobalDirection` · 005750d0 · legions 00544fa0 — CameraManager::global_direction, fed to the movement module
- [ ] `CameraManager::SetCameraAnimTarget` · 005750f0 · legions 00544fc0 — needs port: camera rig (CheckEnvOnly, parent/dummy objects, free/targeted camera, restore, anim target/parent, interp viewer and target offsets); ac2-smartbox camera.rs cites other CameraManager methods, not these
- [ ] `CameraManager::SetCameraAnimParent` · 00575150 · legions 00545020 — needs port: camera rig (CheckEnvOnly, parent/dummy objects, free/targeted camera, restore, anim target/parent, interp viewer and target offsets); ac2-smartbox camera.rs cites other CameraManager methods, not these
- [x] `CameraManager::FakeTransition` · 005751b0 · legions 00546710 — ac2-smartbox camera.rs fake_transition / look_at_self / swing_out_left (action 0x40000028 and 0x40000029 Legions)
- [~] `CameraManager::SetupPrefs` · 005751c0 · legions 00545090 — ac2-smartbox prefs.rs values + camera.rs apply_prefs: default distance and tilt (taken at the camera's next reset, not live), rotate speed, mouse-look sensitivity, Invert_Y, Steer_Player, Align_To_Slope; zoom speed, transition time, auto distance and camera height are registered but not read
- [ ] `CameraManager::StopAllCameraFX` · 00575a20 — needs port: camera effects (CameraFX DBO and CameraFxManager: shake, screen flash, fov shift, motion blur, distortion, camera distance); not in Rust

### CameraFxTrack
- [-] `CameraFxTrack::operator=` · 00575a30 · legions 00545900 — ctor/dtor/operator= or plain data copy (Rust new/Drop/Clone/derive replaces)

### CameraManager
- [x] `CameraManager::GetCameraDistance` · 00575ab0 · legions 00545980 — no camera effects run, so their distance sum is 0
- [x] `CameraManager::ShoulderView` · 00575ad0 · legions 005459a0 — camera action 0x4000002A
- [~] `CameraManager::SetTargetObject` · 00575b40 · legions 00545a10 — the player target only
- [x] `CameraManager::GetInterpViewerOffset` · 00575bc0 · legions 00545a90 — ac2-smartbox targeting.rs interp_viewer_offset (old and new offsets lerped by the blend's time while mIsOn) and camera.rs; read by the player's LookAtManager::Update
- [x] `CameraManager::CalcShoulderOffset` · 00575c00 · legions 00545ad0
- [~] `CameraManager::HandleApply` · 00575ee0 · legions 00545db0 — ac2-smartbox prefs.rs values + camera.rs apply_prefs: default distance and tilt (taken at the camera's next reset, not live), rotate speed, mouse-look sensitivity, Invert_Y, Steer_Player, Align_To_Slope; zoom speed, transition time, auto distance and camera height are registered but not read
- [ ] `CameraManager::DoCameraFX` · 00575f20 · legions 00545df0 — needs port: camera effects (CameraFX DBO and CameraFxManager: shake, screen flash, fov shift, motion blur, distortion, camera distance); not in Rust
- [~] `CameraManager::UpdateCamera` · 005760f0 · legions 00545fb0 — no dynamic first-person head-bone camera, no camera effects
- [x] `CameraManager::LookAtSelf` · 005767a0 · legions 00546660 — ac2-smartbox camera.rs fake_transition / look_at_self / swing_out_left (action 0x40000028 and 0x40000029 Legions)
- [x] `CameraManager::SwingOutLeft` · 00576850 · legions 005467f0 — ac2-smartbox camera.rs fake_transition / look_at_self / swing_out_left (action 0x40000028 and 0x40000029 Legions)
- [-] `CameraManager::SwingOutRight` · 00576930 — no Legions 7160 function: Legions StartAction runs SwingOutLeft for 0x40000029 (camera.rs SWING_OUT_RIGHT); admin-only [vtable-checked: not a slot of any located Legions vtable]
- [~] `CameraManager::SetMapView` · 00576a10 · legions 005468d0 — ac2-smartbox camera.rs set_map_view saves and restores the orbit and distance; SceneTool::m_bDisableDegrades not modelled
- [ ] `CameraManager::GetTargetOffset` · 00576b00 · legions 005469c0 — needs port: camera rig (CheckEnvOnly, parent/dummy objects, free/targeted camera, restore, anim target/parent, interp viewer and target offsets); ac2-smartbox camera.rs cites other CameraManager methods, not these
- [x] `CameraManager::SetDefaults` · 00576b40 · legions 00546a30
- [x] `CameraManager::LookAtGround` · 00576c70 · legions 00546b60 — camera action 0x400000AE (no map view)
- [x] `CameraManager::SetMouseLookState` · 00576d10 · legions 00546c00 — ac2-client camera.rs (driven by gmCameraUI states)
- [x] `CameraManager::SetFirstPersonState` · 00576dc0 · legions 00546cb0 — ac2-client camera.rs (driven by gmCameraUI states)
- [~] `CameraManager::HandleChange` · 00576e30 — applied once with the preference defaults
- [~] `CameraManager::StartAction` · 00577100 · legions 00546ff0 — ac2-smartbox camera.rs start_action / stop_action: every camera action id of Legions StartAction and StopAction; camera effects (FX) and the SceneTool degrade switch not modelled
- [~] `CameraManager::StopAction` · 00577360 · legions 00547250 — ac2-smartbox camera.rs start_action / stop_action: every camera action id of Legions StartAction and StopAction; camera effects (FX) and the SceneTool degrade switch not modelled
- [~] `CameraManager::CameraManager` · 005775a0 · legions 005475f0 — no camera effects manager

### _STL::deque<_STL::pair<long,long>,_STL::allocator<_STL::pair<long,long>_>_>
- [-] `_STL::deque<_STL::pair<long,long>,_STL::allocator<_STL::pair<long,long>_>_>::~deque<_STL::pair<long,long>,_STL::allocator<_STL::pair<long,long>_>_>` · 00577720 · legions 005752f0 — STL less/map/rb-tree/deque internals (Rust std replaces)

### CameraManager
- [-] `CameraManager::~CameraManager` · 00577840 · legions 005477e0 — ctor/dtor/operator= or plain data copy (Rust new/Drop/Clone/derive replaces)
- [x] `CameraManager::MouseMove` · 005778a0 · legions 00547840 — ac2-client camera_mouse.rs / input_handlers

## `portal/client/src/smartbox/camera/targetingmodule.cpp.c` (7/11)

### TargetingModule
- [x] `TargetingModule::SetPlayer` · 00579180 · legions 00548ae0
- [x] `TargetingModule::TurnOn` · 005791a0 · legions 00548b00
- [x] `TargetingModule::CalcCameraDistance` · 00579210 · legions 00548b70
- [x] `TargetingModule::GetInterpViewerOffset` · 00579420 · legions 00548d80 — ac2-smartbox targeting.rs interp_viewer_offset (old and new offsets lerped by the blend's time while mIsOn) and camera.rs; read by the player's LookAtManager::Update
- [x] `TargetingModule::FakeTransition` · 005794b0 · legions 00548e10
- [~] `TargetingModule::GetPosition` · 00579500 · legions 00548e60 — the physics position stands in for the render-slerped drawing position
- [~] `TargetingModule::StoreTargetPos` · 00579670 · legions 00548fd0 — bone 0, the player or no target
- [ ] `TargetingModule::StoreParentPos` · 00579700 · legions 00549060 — needs port: camera rig (CheckEnvOnly, parent/dummy objects, free/targeted camera, restore, anim target/parent, interp viewer and target offsets); ac2-smartbox camera.rs cites other CameraManager methods, not these
- [x] `TargetingModule::Update` · 00579780 · legions 005490e0
- [~] `TargetingModule::InternalCalcViewerPos` · 00579910 · legions 00549270 — a player target without parent or bone (third and first person)
- [x] `TargetingModule::CalcViewerPos` · 0057a320 · legions 00549c80

## `portal/client/src/smartbox/smartbox.cpp.c` (8/30)

### Position
- [-] `Position::operator!=` · 005734b0 · legions 005434b0 — ctor/dtor/operator= or plain data copy (Rust new/Drop/Clone/derive replaces)

### SmartBox
- [-] `SmartBox::RefreshGraphicsResources` · 005734f0 — D3D device resources and render targets (ac2-gpu / wgpu replaces)
- [ ] `SmartBox::GetDisplayWidth` · 00573510 · legions 00543510 — needs port (check): SmartBox display size, region id, notice flag, show/hide, render, reset and init; uncited
- [ ] `SmartBox::GetDisplayHeight` · 00573530 · legions 00543530 — needs port (check): SmartBox display size, region id, notice flag, show/hide, render, reset and init; uncited
- [x] `SmartBox::EnterPortalSpace` · 00573550 · legions 00543550 — ac2-render portal/effect.rs (enter/exit/in_portal_space, Start/Stop) and ac2-client client_teleport.rs; tests: portal_7160, effect_tests
- [x] `SmartBox::ExitPortalSpace` · 00573560 · legions 00543560 — ac2-render portal/effect.rs (enter/exit/in_portal_space, Start/Stop) and ac2-client client_teleport.rs; tests: portal_7160, effect_tests
- [x] `SmartBox::InPortalSpace` · 00573570 · legions 00543570 — ac2-render portal/effect.rs (enter/exit/in_portal_space, Start/Stop) and ac2-client client_teleport.rs; tests: portal_7160, effect_tests
- [~] `SmartBox::RenderScreenshot` · 00573580 · legions 00543580 — the presented frame is copied instead of re-rendering the scene (bin/ac2/screenshot.rs)
- [x] `SmartBox::SetCameraPosition` · 005735e0 · legions 005435e0
- [x] `SmartBox::SetCameraDefaults` · 00573630 · legions 00543630
- [ ] `SmartBox::RecvNotice_SetRegionID` · 005736e0 · legions 005436e0 — needs port (check): SmartBox display size, region id, notice flag, show/hide, render, reset and init; uncited
- [x] `SmartBox::SetWantNoticeOnNextFrameDrawn` · 005736f0 · legions 005436f0
- [ ] `SmartBox::GetWantNoticeOnNextFrameDrawn` · 00573700 · legions 00543700 — needs port (check): SmartBox display size, region id, notice flag, show/hide, render, reset and init; uncited
- [-] `SmartBox::SmartBox` · 00573710 · legions 00543710 — ctor/dtor/operator= or plain data copy (Rust new/Drop/Clone/derive replaces)
- [ ] `SmartBox::IsVisible` · 005737e0 · legions 005437e0 — needs port (check): SmartBox display size, region id, notice flag, show/hide, render, reset and init; uncited
- [ ] `SmartBox::Show` · 005737f0 · legions 005437f0 — needs port (check): SmartBox display size, region id, notice flag, show/hide, render, reset and init; uncited
- [ ] `SmartBox::Hide` · 00573800 · legions 00543800 — needs port (check): SmartBox display size, region id, notice flag, show/hide, render, reset and init; uncited
- [~] `SmartBox::InitPreferences` · 00573810 · legions 00543810 — ac2-client bin/ac2/main.rs reads the preferences; the far radius is applied in ac2-render, uncited
- [x] `SmartBox::UpdatePlayer` · 00573930 · legions 00543930 — cells, ReEnterVisibility (EnterWorldSimple flags 0x11), PlayerReadyToEnterWorld, the entry camera
- [-] `SmartBox::UpdateRenderTargets` · 005739f0 · legions 005439f0 — D3D device resources and render targets (ac2-gpu / wgpu replaces)
- [ ] `SmartBox::Render` · 00573a10 — needs port (check): SmartBox display size, region id, notice flag, show/hide, render, reset and init; uncited
- [ ] `SmartBox::ResetWorldState` · 00573a60 · legions 00543a30 — needs port (check): SmartBox display size, region id, notice flag, show/hide, render, reset and init; uncited
- [~] `SmartBox::InitPlayer` · 00573b50 · legions 00543b20 — SetCameraDefaults once the body's Setup has loaded
- [~] `SmartBox::UpdateCamera` · 00573bd0 · legions 00543ba0 — the line-of-sight pull-in (CObjCell::LOSCheck) is not ported: the desired position stands
- [~] `SmartBox::RecvNotice_CreatePlayerObject` · 00574220 · legions 00544170 — the body loads on a loader thread; m_bBlockingLoad set
- [x] `SmartBox::PlayerPhysicsUpdatedCallback` · 00574280 · legions 005441d0 — after every player physics quantum outside mouse look
- [~] `SmartBox::HandleTeleport` · 005742d0 · legions 00544220 — teleport flow in ac2-client (client_teleport.rs); not yet: run-lock reset, target clear, portal tunnel advancement (GeneratePortals), the portal sounds
- [-] `SmartBox::~SmartBox` · 005743e0 · legions 00544330 — ctor/dtor/operator= or plain data copy (Rust new/Drop/Clone/derive replaces)
- [ ] `SmartBox::Init` · 005744f0 · legions 00544420 — needs port (check): SmartBox display size, region id, notice flag, show/hide, render, reset and init; uncited
- [~] `SmartBox::UseTime` · 005746d0 · legions 005445d0 — UpdatePlayer, WorldManager::Update, the mouse-look camera, SceneCamera::slerp; no camera effects

## `portal/client/src/smartbox/worldmanager/cellmanager.cpp.c` (6/14)

### CellManager
- [-] `CellManager::~CellManager` · 0057ba00 · legions 0054b2e0 — ctor/dtor/operator= or plain data copy (Rust new/Drop/Clone/derive replaces)
- [x] `CellManager::OnContextFinished` · 0057ba10 · legions 0054b2f0
- [x] `CellManager::ReleaseCells` · 0057ba40 · legions 0054b320
- [ ] `CellManager::SetFarRadius` · 0057baa0 · legions 0054b380 — needs port (check): CellManager far radius, region change, outside cell id, reset; uncited
- [ ] `CellManager::NotifyChangeRegion` · 0057bac0 · legions 0054b3a0 — needs port (check): CellManager far radius, region change, outside cell id, reset; uncited

### Position
- [ ] `Position::get_outside_cell_id` · 0057bad0 · legions 0054b3b0 — needs port (check): CellManager far radius, region change, outside cell id, reset; uncited

### CellManager
- [x] `CellManager::CellManager` · 0057bb10 · legions 0054b3f0
- [x] `CellManager::RequestAsyncLoadBlockRadius` · 0057bba0 · legions 0054b480 — ring requests to the loader threads
- [~] `CellManager::RequestAsyncLoadBlockRadiusSceneObjects` · 0057bcd0 · legions 0054b5b0 — the near blocks' objects come with their sub-data, so nothing is left to request
- [ ] `CellManager::ResetWorldState` · 0057be70 · legions 0054b750 — needs port (check): CellManager far radius, region change, outside cell id, reset; uncited
- [~] `CellManager::FinishUpdateVisibleCells` · 0057be90 · legions 0054b770 — an interior keeps the near window around its block
- [x] `CellManager::AsyncGetCells` · 0057bf80 · legions 0054b850
- [~] `CellManager::OnTopLevelRequestFinished` · 0057c040 · legions 0054b910 — an interior's visible cells in other blocks are not requested
- [x] `CellManager::UpdateVisibleCells` · 0057c360 · legions 0054bc30

## `portal/client/src/smartbox/worldmanager/lscapemanager.cpp.c` (2/21)

### LScapeManager
- [ ] `LScapeManager::GetBlockShift` · 0057a510 · legions 00549e70 — needs port (check): LScapeManager block shift, visible cells, reload, refresh, reset and region change; uncited (LScapeRender::SetVertexValue calls SetVertexValue)
- [x] `LScapeManager::SetFarRadius` · 0057a590 · legions 00549ef0 — far radius consumed by FarTerrain::new
- [ ] `LScapeManager::GrabVisibleCells` · 0057a5b0 · legions 00549f10 — needs port (check): LScapeManager block shift, visible cells, reload, refresh, reset and region change; uncited (LScapeRender::SetVertexValue calls SetVertexValue)
- [ ] `LScapeManager::ReleaseVisibleCells` · 0057a640 · legions 00549fa0 — needs port (check): LScapeManager block shift, visible cells, reload, refresh, reset and region change; uncited (LScapeRender::SetVertexValue calls SetVertexValue)
- [~] `LScapeManager::Init` · 0057a780 · legions 0054a060 — near radius 1, far radius 4
- [ ] `LScapeManager::ReloadLandBlocks` · 0057a7b0 · legions 0054a090 — needs port (check): LScapeManager block shift, visible cells, reload, refresh, reset and region change; uncited (LScapeRender::SetVertexValue calls SetVertexValue)
- [-] `LScapeManager::InitFreeBlocks` · 0057a810 · legions 0054a0f0 — landblock free-block pool (ac2-smartbox BlockPool loader threads replace it; AGENTS pool exception)
- [-] `LScapeManager::CleanupFreeBlocks` · 0057a880 · legions 0054a160 — landblock free-block pool (ac2-smartbox BlockPool loader threads replace it; AGENTS pool exception)
- [ ] `LScapeManager::ResetWorldState` · 0057a8d0 · legions 0054a1b0 — needs port (check): LScapeManager block shift, visible cells, reload, refresh, reset and region change; uncited (LScapeRender::SetVertexValue calls SetVertexValue)
- [ ] `LScapeManager::NotifyChangeRegion` · 0057a8e0 · legions 0054a1c0 — needs port (check): LScapeManager block shift, visible cells, reload, refresh, reset and region change; uncited (LScapeRender::SetVertexValue calls SetVertexValue)
- [ ] `LScapeManager::Refresh` · 0057a980 · legions 0054a260 — needs port (check): LScapeManager block shift, visible cells, reload, refresh, reset and region change; uncited (LScapeRender::SetVertexValue calls SetVertexValue)
- [-] `LScapeManager::ReleaseBlock` · 0057a9f0 · legions 0054a2d0 — landblock free-block pool (ac2-smartbox BlockPool loader threads replace it; AGENTS pool exception)
- [x] `LScapeManager::GetLandBlock` · 0057aa60 · legions 0054a340 — crates/ac2-physics/src/los/land.rs:190 (LScapeManager::GetLandBlock: whether the block is resident)
- [ ] `LScapeManager::ReleaseCells` · 0057ab30 · legions 0054a410 — needs port (check): LScapeManager block shift, visible cells, reload, refresh, reset and region change; uncited (LScapeRender::SetVertexValue calls SetVertexValue)
- [ ] `LScapeManager::SetVertexValue` · 0057ac20 · legions 0054a500 — needs port (check): LScapeManager block shift, visible cells, reload, refresh, reset and region change; uncited (LScapeRender::SetVertexValue calls SetVertexValue)
- [-] `LScapeManager::GetFreeBlock` · 0057af50 · legions 0054a830 — landblock free-block pool (ac2-smartbox BlockPool loader threads replace it; AGENTS pool exception)
- [-] `LScapeManager::~LScapeManager` · 0057b1b0 · legions 0054aa90 — ctor/dtor/operator= or plain data copy (Rust new/Drop/Clone/derive replaces)
- [ ] `LScapeManager::AddBlockToRefreshList` · 0057b260 · legions 0054ab40 — needs port (check): LScapeManager block shift, visible cells, reload, refresh, reset and region change; uncited (LScapeRender::SetVertexValue calls SetVertexValue)
- [ ] `LScapeManager::UpdateBlocks` · 0057b270 · legions 0054ab50 — needs port (check): LScapeManager block shift, visible cells, reload, refresh, reset and region change; uncited (LScapeRender::SetVertexValue calls SetVertexValue)
- [-] `LScapeManager::LScapeManager` · 0057b8f0 · legions 0054b1d0 — ctor/dtor/operator= or plain data copy (Rust new/Drop/Clone/derive replaces)
- [~] `LScapeManager::UpdateVisibleCells` · 0057b980 · legions 0054b260 — the near window (radius 1) resident, the rest released

## `portal/client/src/smartbox/worldmanager/worldmanager.cpp.c` (3/9)

### WorldManager
- [-] `WorldManager::WorldManager` · 0057c420 · legions 0054bcf0 — ctor/dtor/operator= or plain data copy (Rust new/Drop/Clone/derive replaces)
- [ ] `WorldManager::Init` · 0057c440 · legions 0054bd10 — needs port (check): WorldManager Init, SetRegion, Refresh, ResetWorldState; uncited
- [x] `WorldManager::IsBlockingForCells` · 0057c500 · legions 0054bdd0
- [x] `WorldManager::Update` · 0057c510 · legions 0054bde0 — GameTime::UseTime, then unless blocking CObjectMaint::UseTime and the physics step
- [x] `WorldManager::UpdateVisibleCells` · 0057c540 · legions 0054be10
- [ ] `WorldManager::SetRegion` · 0057c570 · legions 0054be40 — needs port (check): WorldManager Init, SetRegion, Refresh, ResetWorldState; uncited
- [ ] `WorldManager::Refresh` · 0057c5e0 · legions 0054beb0 — needs port (check): WorldManager Init, SetRegion, Refresh, ResetWorldState; uncited
- [-] `WorldManager::~WorldManager` · 0057c610 · legions 0054bee0 — ctor/dtor/operator= or plain data copy (Rust new/Drop/Clone/derive replaces)
- [ ] `WorldManager::ResetWorldState` · 0057c6b0 · legions 0054bf80 — needs port (check): WorldManager Init, SetRegion, Refresh, ResetWorldState; uncited

## `portal/shared/src/objects/iidutils.cpp.c` (8/13)

### IIDUtils
- [ ] `IIDUtils::CreateUI` · 006bc300 · legions 006512a0 — needs port (check): IIDUtils CreateUI/CreateTemporary/CreateStatic and camera parent/target ids; ac2-net iid.rs has verify/type but no citation of these
- [ ] `IIDUtils::CreateTemporary` · 006bc340 · legions 006512e0 — needs port (check): IIDUtils CreateUI/CreateTemporary/CreateStatic and camera parent/target ids; ac2-net iid.rs has verify/type but no citation of these
- [x] `IIDUtils::GetType` · 006bc380 · legions 00651320 — ac2-net iid: id_type / is_player / is_dynamic / is_static / read_block
- [x] `IIDUtils::VerifyPlayer` · 006bc410 · legions 006513b0 — ac2-net iid: id_type / is_player / is_dynamic / is_static / read_block
- [x] `IIDUtils::VerifyUI` · 006bc430 · legions 006513d0 — ac2-net iid: id_type / is_player / is_dynamic / is_static / read_block
- [x] `IIDUtils::VerifyCamera` · 006bc450 · legions 006513f0 — ac2-net iid: id_type / is_player / is_dynamic / is_static / read_block
- [x] `IIDUtils::VerifyTemporary` · 006bc470 · legions 00651410 — ac2-net iid: id_type / is_player / is_dynamic / is_static / read_block
- [x] `IIDUtils::VerifyStatic` · 006bc490 · legions 00651430 — ac2-net iid: id_type / is_player / is_dynamic / is_static / read_block
- [x] `IIDUtils::VerifyDynamic` · 006bc4c0 · legions 00651460 — ac2-net iid: id_type / is_player / is_dynamic / is_static / read_block
- [x] `IIDUtils::ReadBlock` · 006bc4e0 · legions 00651480 — ac2-net iid: id_type / is_player / is_dynamic / is_static / read_block
- [ ] `IIDUtils::GetCameraParent` · 006bc4f0 · legions 00651490 — needs port (check): IIDUtils CreateUI/CreateTemporary/CreateStatic and camera parent/target ids; ac2-net iid.rs has verify/type but no citation of these
- [ ] `IIDUtils::GetCameraTarget` · 006bc510 · legions 006514b0 — needs port (check): IIDUtils CreateUI/CreateTemporary/CreateStatic and camera parent/target ids; ac2-net iid.rs has verify/type but no citation of these
- [ ] `IIDUtils::CreateStatic` · 006bc530 · legions 006514d0 — needs port (check): IIDUtils CreateUI/CreateTemporary/CreateStatic and camera parent/target ids; ac2-net iid.rs has verify/type but no citation of these
