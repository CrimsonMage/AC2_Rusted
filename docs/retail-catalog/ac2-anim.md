# ac2-anim: Animation system

532 retail functions: 2 done, 2 partial, 492 to do, 36 skipped.
Legend and workflow: [README](README.md).

## `portal/shared/src/anim/charactersetup.cpp.c` (0/67)

### PlacementFrame
- [ ] `PlacementFrame::Pack` · 0060d540 · legions 005ce310
- [ ] `PlacementFrame::UnPack` · 0060d5a0 · legions 005ce370
- [ ] `PlacementFrame::SetNumBones` · 0060d630
- [ ] `PlacementFrame::SetTransform` · 0060d690

### DegradeInfo
- [ ] `DegradeInfo::Pack` · 0060d700 · legions 005ce400
- [ ] `DegradeInfo::UnPack` · 0060d800 · legions 005ce500
- [ ] `DegradeInfo::DegradeInfo` · 0060d8f0 · legions 005ce5f0

### CharacterSetup
- [ ] `CharacterSetup::GetSubDataIDs` · 0060d940 · legions 005ce640
- [ ] `CharacterSetup::SetBoundingObject` · 0060d980
- [ ] `CharacterSetup::SetNumSphere` · 0060db10
- [ ] `CharacterSetup::SetNumCylsphere` · 0060db70

### gmCharGenResult
- [ ] `gmCharGenResult::SetImplement` · 0060dbd0 · legions 005ce810

### CharacterSetup
- [ ] `CharacterSetup::IsBoneType` · 0060dbe0
- [ ] `CharacterSetup::GetBoneType` · 0060dc10

### HashBaseIter<unsigned_long>
- [ ] `HashBaseIter<unsigned_long>::Next` · 0060dc30 · legions 005ce870
- [ ] `HashBaseIter<unsigned_long>::DeleteCurrent` · 0060dc80 · legions 005ce8c0

### CharacterSetup
- [ ] `CharacterSetup::SetSphere` · 0060dd10

### LocationType
- [ ] `LocationType::LocationType` · 0060dd50 · legions 005ce950

### PlacementFrame
- [ ] `PlacementFrame::~PlacementFrame` · 0060de10 · legions 005cea10
- [ ] `PlacementFrame::PlacementFrame` · 0060de70 · legions 005cea70

### MeshInfo
- [ ] `MeshInfo::MeshInfo` · 0060dfa0

### BoneInfo
- [ ] `BoneInfo::BoneInfo` · 0060dfd0 · legions 005cebd0

### LocationType
- [ ] `LocationType::SetTypeName` · 0060e000

### PlacementFrame
- [ ] `PlacementFrame::SetTypeName` · 0060e060

### BoneInfo
- [ ] `BoneInfo::SetTypeName` · 0060e0c0

### CharacterSetup
- [ ] `CharacterSetup::SetNumMesh` · 0060e270 · legions 005ced50

### MeshInfo
- [ ] `MeshInfo::operator=` · 0060e2d0 · legions 005cedb0

### CharacterSetup
- [ ] `CharacterSetup::SetNumBones` · 0060e320 · legions 005cee00

### BoneInfo
- [ ] `BoneInfo::operator=` · 0060e380 · legions 005cee60

### CharacterSetup
- [~] `CharacterSetup::InitFileLoad` · 0060e3d0 — ac2-physics ObjectShape::new: physics containment radius and box only

### LocationType
- [-] `LocationType::ToFileNode` · 0060ea80 — text source format (admin tools)
- [-] `LocationType::FromFileNode` · 0060eb70 — text source format (admin tools)

### PlacementFrame
- [-] `PlacementFrame::ToFileNode` · 0060ed50 — text source format (admin tools)
- [-] `PlacementFrame::FromFileNode` · 0060ee40 — text source format (admin tools)

### MeshInfo
- [-] `MeshInfo::ToFileNode` · 0060f040 — text source format (admin tools)
- [-] `MeshInfo::FromFileNode` · 0060f0d0 — text source format (admin tools)

### BoneInfo
- [-] `BoneInfo::ToFileNode` · 0060f1b0 — text source format (admin tools)
- [-] `BoneInfo::FromFileNode` · 0060f270 — text source format (admin tools)

### DegradeInfo
- [-] `DegradeInfo::ToFileNode` · 0060f430 — text source format (admin tools)

### DLDegradeInfo
- [-] `DLDegradeInfo::FromFileNode` · 0060f690 — text source format (admin tools)

### DegradeInfo
- [-] `DegradeInfo::FromFileNode` · 0060f800 — text source format (admin tools)

### RenderInfo
- [-] `RenderInfo::ToFileNode` · 0060f8f0 — text source format (admin tools)
- [-] `RenderInfo::FromFileNode` · 0060f9d0 — text source format (admin tools)

### CharacterSetup
- [ ] `CharacterSetup::AddPlacementFrame` · 0060fad0 · legions 005cef20
- [ ] `CharacterSetup::AddHoldingLocation` · 0060fb70 · legions 005cefc0
- [ ] `CharacterSetup::AddConnectionPoint` · 0060fc30 · legions 005cf080
- [ ] `CharacterSetup::SetCylsphere` · 0060fcf0
- [ ] `CharacterSetup::GetHoldingLocation` · 0060fd40 · legions 005cf140
- [ ] `CharacterSetup::GetConnectionPoint` · 0060fd80 · legions 005cf180
- [ ] `CharacterSetup::GetPlacementFrame` · 0060fdc0 · legions 005cf1c0
- [ ] `CharacterSetup::GetPackSize` · 0060fe10
- [ ] `CharacterSetup::Pack` · 00610060
- [ ] `CharacterSetup::UnPack` · 006105a0 · legions 005cf9a0
- [-] `CharacterSetup::ToFileNode` · 00610b60 — text source format (admin tools)
- [ ] `CharacterSetup::ToFileNodeSetinfo` · 00611390
- [-] `CharacterSetup::FromFileNode` · 00611440 — text source format (admin tools)
- [ ] `CharacterSetup::CopyInto` · 00611b20
- [ ] `CharacterSetup::SaveToFile` · 00612030
- [-] `CharacterSetup::LoadFromFile` · 00612190 — text source loader, admin build only
- [ ] `CharacterSetup::Destroy` · 00612410 · legions 005d04d0
- [ ] `CharacterSetup::SetRestBones` · 006124f0 · legions 005d05b0
- [ ] `CharacterSetup::InitRestBones` · 00612670 · legions 005d0730
- [ ] `CharacterSetup::~CharacterSetup` · 006126d0 · legions 005d0790
- [ ] `CharacterSetup::GetParentIndex` · 00612760
- [ ] `CharacterSetup::FindBoneType` · 00612780 · legions 005d0840
- [ ] `CharacterSetup::InitLoad` · 006127e0
- [ ] `CharacterSetup::CharacterSetup` · 00612860 · legions 005d0920

## `portal/shared/src/anim/setup.cpp.c` (0/4)

### CSetup
- [ ] `CSetup::CSetup` · 0072f380 · legions 006acc40
- [ ] `CSetup::GetDBOType` · 0072f3a0
- [~] `CSetup::CachePhysicsInformation` · 0072f3b0 — ac2-physics ObjectShape::new: physics containment radius and box only
- [ ] `CSetup::Allocate` · 0072f4b0

## `portal/shared/src/animsys/animator.cpp.c` (0/5)

### AMLook
- [ ] `AMLook::AMLook` · 0068ebe0 · legions 0062fc00

### AMRandom
- [ ] `AMRandom::AMRandom` · 0068ec20 · legions 0062fc40

### Animator
- [ ] `Animator::Create` · 0068ec70 · legions 0062fc90
- [ ] `Animator::FromFile` · 0068ed70
- [ ] `Animator::SaveToFile` · 0068f1a0

## `portal/shared/src/animsys/animblender.cpp.c` (0/4)

### AnimBlender
- [ ] `AnimBlender::Animate` · 0068fde0 · legions 00630960
- [ ] `AnimBlender::~AnimBlender` · 00690040 · legions 00630bc0
- [ ] `AnimBlender::SetNumAnimations` · 00690090 · legions 00630c10
- [ ] `AnimBlender::AddAnimation` · 006900c0 · legions 00630c40

## `portal/shared/src/animsys/animcompositor.cpp.c` (0/7)

### AnimCompositor
- [ ] `AnimCompositor::Destroy` · 0068b6c0 · legions 0062c6f0
- [ ] `AnimCompositor::Clear` · 0068b730 · legions 0062c760
- [ ] `AnimCompositor::AddAnimation` · 0068b770 · legions 0062c7a0
- [ ] `AnimCompositor::Init` · 0068b7a0 · legions 0062c7d0
- [ ] `AnimCompositor::Animate` · 0068b890 · legions 0062c8c0
- [ ] `AnimCompositor::~AnimCompositor` · 0068bad0 · legions 0062cb00
- [ ] `AnimCompositor::AnimCompositor` · 0068bb40 · legions 0062cb70

## `portal/shared/src/animsys/animinfo.cpp.c` (0/12)

### BaseList<unsigned_long,11>
- [ ] `BaseList<unsigned_long,11>::GetReferences` · 00739420 · legions 006b4f60

### AnimInfo
- [ ] `AnimInfo::GetHoldLoop` · 00739430 · legions 006b4f70
- [ ] `AnimInfo::StreamPack` · 00739460 · legions 006b4fa0
- [ ] `AnimInfo::ParseLoopNode` · 007397a0
- [ ] `AnimInfo::ParseComboAttackNode` · 00739990 · legions 006b5570
- [-] `AnimInfo::ToFileNode` · 00739a90 · legions 006b5670 — text source format (admin tools)
- [ ] `AnimInfo::GetTriggers` · 0073ab60 · legions 006b6db0
- [ ] `AnimInfo::ParseImpulseNode` · 0073b010
- [ ] `AnimInfo::ParseEffectsNode` · 0073b320
- [ ] `AnimInfo::ParseStopEffectsNode` · 0073b860
- [ ] `AnimInfo::ParseWeenieHookNode` · 0073bbe0
- [-] `AnimInfo::FromFileNode` · 0073c0a0 · legions 006b83b0 — text source format (admin tools)

## `portal/shared/src/animsys/animmap.cpp.c` (0/13)

### AnimMap
- [ ] `AnimMap::StreamPack` · 006796f0 · legions 00623ac0
- [ ] `AnimMap::GetSubDataIDs` · 006797f0
- [ ] `AnimMap::Destroy` · 00679860
- [-] `AnimMap::ToFileNode` · 00679870 — text source format (admin tools)
- [ ] `AnimMap::CacheAnimations` · 00679aa0 · legions 00623ba0
- [ ] `AnimMap::GetAnimator` · 00679ca0 · legions 00623d70
- [ ] `AnimMap::~AnimMap` · 00679ce0 · legions 00623db0
- [ ] `AnimMap::ParseMappingNode` · 00679d60
- [ ] `AnimMap::ParseMapsNode` · 0067a010
- [ ] `AnimMap::CopyInto` · 0067a210
- [-] `AnimMap::FromFileNode` · 0067a230 — text source format (admin tools)
- [ ] `AnimMap::AnimMap` · 0067a3e0 · legions 00623e70
- [ ] `AnimMap::Allocate` · 0067a430

## `portal/shared/src/animsys/animmodule.cpp.c` (0/4)

### AnimModule
- [ ] `AnimModule::AnimModule` · 00690330 · legions 00630ea0
- [-] `AnimModule::FromFileNode` · 00690340 — text source format (admin tools)
- [ ] `AnimModule::FromFile` · 00690630
- [-] `AnimModule::ToFileNode` · 00690b00 — text source format (admin tools)

## `portal/shared/src/animsys/animmodules/amangle.cpp.c` (0/13)

### BSPLEAF
- [ ] `BSPLEAF::ClipToCellBSP` · 00692890

### AMLook
- [ ] `AMLook::GetPeriod` · 006928a0

### CDataMesh
- [ ] `CDataMesh::GetDBOType` · 006928b0

### AMLook
- [ ] `AMLook::GetTriggers` · 006928c0

### AMAngle
- [ ] `AMAngle::StreamPack` · 006928d0 · legions 00632a20
- [ ] `AMAngle::Animate` · 00692960 · legions 00632b10
- [ ] `AMAngle::DataFromFileNode` · 00692e50
- [ ] `AMAngle::Destroy` · 00693210

### PList<TurbineUserStatus>
- [ ] `PList<TurbineUserStatus>::GetPackSize` · 00693230

### PList<BoneAngleMod>
- [ ] `PList<BoneAngleMod>::UnPack` · 00693270 · legions 006333f4

### AMAngle
- [ ] `AMAngle::AMAngle` · 00693390 · legions 00633500

### PList<BoneAngleMod>
- [ ] `PList<BoneAngleMod>::operator=` · 006933e0 · legions 00633550

### AMAngle
- [ ] `AMAngle::CopyMe` · 00693470

## `portal/shared/src/animsys/animmodules/amhuman.cpp.c` (0/13)

### AMHuman
- [ ] `AMHuman::AMHuman` · 00694960 · legions 006349f0

### CContinSoundInfo
- [ ] `CContinSoundInfo::GetSubDataIDs` · 00694980 · legions 00634a10

### AMHuman
- [ ] `AMHuman::HasAnimator` · 006949a0
- [ ] `AMHuman::CheckAnimationBoneNum` · 00694a10
- [ ] `AMHuman::GetPeriod` · 00694a80
- [ ] `AMHuman::CopyMe` · 00694af0
- [ ] `AMHuman::Destroy` · 00694b30 · legions 00634b70
- [ ] `AMHuman::GetTriggers` · 00694b50
- [ ] `AMHuman::StreamPack` · 00694bd0
- [ ] `AMHuman::Animate` · 00694c00
- [ ] `AMHuman::DataToFileNode` · 00695020
- [ ] `AMHuman::GetRecursiveSubDataIDs` · 006950e0
- [ ] `AMHuman::DataFromFileNode` · 00695170

## `portal/shared/src/animsys/animmodules/amlook.cpp.c` (0/3)

### AMLook
- [ ] `AMLook::CopyMe` · 00690c60
- [ ] `AMLook::~AMLook` · 00690c90 · legions 00630ef0
- [ ] `AMLook::Animate` · 00690ca0 · legions 00630f00

## `portal/shared/src/animsys/animmodules/amrandom.cpp.c` (0/14)

### AMRandom
- [ ] `AMRandom::GetSubDataIDs` · 00691090
- [ ] `AMRandom::GetRecursiveSubDataIDs` · 006910a0
- [ ] `AMRandom::HasAnimator` · 006910b0
- [ ] `AMRandom::CheckAnimationBoneNum` · 006910c0
- [ ] `AMRandom::GetPeriod` · 006910d0
- [ ] `AMRandom::GetType` · 006910e0
- [ ] `AMRandom::Destroy` · 006910f0
- [ ] `AMRandom::GetTriggers` · 00691100
- [ ] `AMRandom::~AMRandom` · 00691130 · legions 00631390
- [ ] `AMRandom::StreamPack` · 00691160 · legions 006313c0
- [ ] `AMRandom::Animate` · 006911c0 · legions 00631420
- [ ] `AMRandom::DataToFileNode` · 00691270
- [ ] `AMRandom::CopyMe` · 00691590
- [ ] `AMRandom::DataFromFileNode` · 00691610

## `portal/shared/src/animsys/animmodules/amswitchme.cpp.c` (0/16)

### AMSwitchME
- [ ] `AMSwitchME::StreamPack` · 006934c0 · legions 00633630
- [ ] `AMSwitchME::GetSubDataIDs` · 00693560
- [ ] `AMSwitchME::CheckAnimationBoneNum` · 006935d0
- [ ] `AMSwitchME::DataToFileNode` · 00693700
- [ ] `AMSwitchME::GetRecursiveSubDataIDs` · 006939f0
- [ ] `AMSwitchME::Destroy` · 00693b50 · legions 00633c30
- [ ] `AMSwitchME::HasAnimator` · 00693c20
- [ ] `AMSwitchME::~AMSwitchME` · 00693f70 · legions 00634020
- [ ] `AMSwitchME::GetAnimator` · 00694000 · legions 006340b0
- [ ] `AMSwitchME::GetPeriod` · 00694120 · legions 006341b0
- [ ] `AMSwitchME::DataFromFileNode` · 00694150
- [ ] `AMSwitchME::GetTriggers` · 00694770 · legions 00634800
- [ ] `AMSwitchME::GetHoldLoop` · 006947a0 · legions 00634830
- [ ] `AMSwitchME::Animate` · 00694880 · legions 00634910
- [ ] `AMSwitchME::AMSwitchME` · 006948c0 · legions 00634950
- [ ] `AMSwitchME::CopyMe` · 00694910

## `portal/shared/src/animsys/animmodules/animasset.cpp.c` (0/53)

### AnimAssetOffsetKey
- [ ] `AnimAssetOffsetKey::AnimAssetOffsetKey` · 00698830

### AnimAssetQuatKey
- [ ] `AnimAssetQuatKey::AnimAssetQuatKey` · 00698840

### AnimAssetWeightKey
- [ ] `AnimAssetWeightKey::AnimAssetWeightKey` · 00698850

### AnimAsset
- [ ] `AnimAsset::Destroy` · 00698860 · legions 006374d0
- [ ] `AnimAsset::GetPeriod` · 00698920
- [ ] `AnimAsset::GetTriggers` · 00698930
- [ ] `AnimAsset::GetHoldLoop` · 00698990

### AnimAssetScaleKey
- [ ] `AnimAssetScaleKey::AnimAssetScaleKey` · 00698cb0

### AnimAsset
- [ ] `AnimAsset::AnimAsset` · 00698cc0 · legions 00637930

### KeyframeLookupTable<AnimAssetScaleKey>
- [ ] `KeyframeLookupTable<AnimAssetScaleKey>::FindLargestLTEQ` · 00698ce0 · legions 00637950

### KeyframeLookupTable<AnimAssetQuatKey>
- [ ] `KeyframeLookupTable<AnimAssetQuatKey>::FindLargestLTEQ` · 00698d80 · legions 006379f0

### KeyframeLookupTable<AnimAssetWeightKey>
- [ ] `KeyframeLookupTable<AnimAssetWeightKey>::FindLargestLTEQ` · 00698e20 · legions 00637a90

### SmartArray<AnimAssetOffsetKey>
- [ ] `SmartArray<AnimAssetOffsetKey>::operator=` · 00698ec0 · legions 00637b30

### PSmartArray<AnimAssetOffsetKey>
- [ ] `PSmartArray<AnimAssetOffsetKey>::StreamPack` · 00698f30 · legions 00637ba0

### SmartArray<AnimAssetQuatKey>
- [ ] `SmartArray<AnimAssetQuatKey>::operator=` · 00698fd0 · legions 00637c40

### PSmartArray<AnimAssetQuatKey>
- [ ] `PSmartArray<AnimAssetQuatKey>::StreamPack` · 00699040 · legions 00637cb0

### SmartArray<AnimAssetWeightKey>
- [ ] `SmartArray<AnimAssetWeightKey>::operator=` · 006990e0 · legions 00637d50

### PSmartArray<AnimAssetWeightKey>
- [ ] `PSmartArray<AnimAssetWeightKey>::StreamPack` · 00699150 · legions 00637dc0

### AnimInfo
- [ ] `AnimInfo::AnimInfo` · 006992d0 · legions 00637f40
- [ ] `AnimInfo::~AnimInfo` · 00699340 · legions 00637fb0

### AnimAsset
- [ ] `AnimAsset::Animate` · 00699400
- [ ] `AnimAsset::SetKeyFrame` · 00699990 · legions 00638600
- [ ] `AnimAsset::SetScaleKeyFrame` · 00699a90 · legions 00638700

### KeyframeLookupTable<AnimAssetScaleKey>
- [ ] `KeyframeLookupTable<AnimAssetScaleKey>::KeyframeLookupTable<AnimAssetScaleKey>` · 00699ae0
- [ ] `KeyframeLookupTable<AnimAssetScaleKey>::~KeyframeLookupTable<AnimAssetScaleKey>` · 00699b10 · legions 00638780

### KeyframeLookupTable<AnimAssetOffsetKey>
- [ ] `KeyframeLookupTable<AnimAssetOffsetKey>::KeyframeLookupTable<AnimAssetOffsetKey>` · 00699b70
- [ ] `KeyframeLookupTable<AnimAssetOffsetKey>::~KeyframeLookupTable<AnimAssetOffsetKey>` · 00699ba0 · legions 00638810

### KeyframeLookupTable<AnimAssetQuatKey>
- [ ] `KeyframeLookupTable<AnimAssetQuatKey>::KeyframeLookupTable<AnimAssetQuatKey>` · 00699c00
- [ ] `KeyframeLookupTable<AnimAssetQuatKey>::~KeyframeLookupTable<AnimAssetQuatKey>` · 00699c30 · legions 006388a0

### KeyframeLookupTable<AnimAssetWeightKey>
- [ ] `KeyframeLookupTable<AnimAssetWeightKey>::KeyframeLookupTable<AnimAssetWeightKey>` · 00699c90
- [ ] `KeyframeLookupTable<AnimAssetWeightKey>::~KeyframeLookupTable<AnimAssetWeightKey>` · 00699cc0 · legions 00638930

### SmartArray<AnimAssetScaleKey>
- [ ] `SmartArray<AnimAssetScaleKey>::operator=` · 00699d20 · legions 00638990

### PSmartArray<AnimAssetScaleKey>
- [ ] `PSmartArray<AnimAssetScaleKey>::StreamPack` · 00699d90 · legions 00638a00

### AnimAsset
- [ ] `AnimAsset::SetNumBones` · 00699e30 · legions 00638aa0
- [-] `AnimAsset::LoadFromFile` · 0069a270 — text source loader, admin build only
- [ ] `AnimAsset::SaveToFile` · 0069a4e0

### KeyframeLookupTable<AnimAssetScaleKey>
- [ ] `KeyframeLookupTable<AnimAssetScaleKey>::InitLookupTable` · 0069a630 · legions 00639030

### KeyframeLookupTable<AnimAssetQuatKey>
- [ ] `KeyframeLookupTable<AnimAssetQuatKey>::InitLookupTable` · 0069a7f0 · legions 006391f0

### KeyframeLookupTable<AnimAssetWeightKey>
- [ ] `KeyframeLookupTable<AnimAssetWeightKey>::InitLookupTable` · 0069a9b0 · legions 006393b0

### AnimAsset
- [ ] `AnimAsset::StreamPackInternal` · 0069ab70 · legions 00639570
- [ ] `AnimAsset::SetNumScaleKeys` · 0069aca0 · legions 006396a0
- [ ] `AnimAsset::SetNumKeyFrames` · 0069ace0 · legions 006396e0
- [ ] `AnimAsset::ReadBinaryAnim` · 0069ad90 · legions 00639790
- [ ] `AnimAsset::WriteBinaryAnim` · 0069af10

### KeyframeLookupTable<AnimAssetScaleKey>
- [ ] `KeyframeLookupTable<AnimAssetScaleKey>::StreamPack` · 0069afe0

### KeyframeLookupTable<AnimAssetQuatKey>
- [ ] `KeyframeLookupTable<AnimAssetQuatKey>::StreamPack` · 0069b020

### KeyframeLookupTable<AnimAssetWeightKey>
- [ ] `KeyframeLookupTable<AnimAssetWeightKey>::StreamPack` · 0069b060

### AnimInfo
- [ ] `AnimInfo::AnimInfo` · 0069b0a0 · legions 006399d0

### AnimAsset
- [ ] `AnimAsset::CheckAnimationBoneNum` · 0069b140 · legions 00639a70
- [ ] `AnimAsset::StreamPack` · 0069b1f0
- [-] `AnimAsset::FromFileNode` · 0069b210 · legions 00639b40 — text source format (admin tools)
- [-] `AnimAsset::ToFileNode` · 0069ba90 — text source format (admin tools)
- [ ] `AnimAsset::CopyMe` · 0069bbf0

## `portal/shared/src/animsys/animmodules/animcube.cpp.c` (0/22)

### AnimCube
- [ ] `AnimCube::FlushCache` · 006965a0 · legions 00636140
- [ ] `AnimCube::CheckAnimationBoneNum` · 006965e0

### AnimCube::SliderAndTickList
- [ ] `AnimCube::SliderAndTickList::StreamPack` · 00696670 · legions 00636200

### AnimCube
- [ ] `AnimCube::GetFromLocalCache` · 006966c0
- [ ] `AnimCube::AnimCube` · 00696770 · legions 00636290
- [ ] `AnimCube::Destroy` · 006967b0 · legions 006362d0
- [ ] `AnimCube::GetAnimatorWeights` · 006967e0 · legions 00636300
- [ ] `AnimCube::~AnimCube` · 00696a00 · legions 00636520
- [ ] `AnimCube::GetRecursiveSubDataIDs` · 00696a90
- [ ] `AnimCube::HasAnimator` · 00696b70
- [ ] `AnimCube::StreamPack` · 00696ce0 · legions 006367e0
- [ ] `AnimCube::Animate` · 00696d80
- [ ] `AnimCube::GetPeriod` · 00696e10
- [ ] `AnimCube::GetAnimatorWeights` · 00696ea0
- [-] `AnimCube::ToFileNode` · 00697180 — text source format (admin tools)

### AnimCube::SliderAndTickList
- [ ] `AnimCube::SliderAndTickList::SliderAndTickList` · 00697860

### AnimCube
- [ ] `AnimCube::GetTriggers` · 00697a20

### SmartArray<AnimCube::SliderAndTickList>
- [ ] `SmartArray<AnimCube::SliderAndTickList>::operator=` · 00697ff0 · legions 006372c0

### PSmartArray<AnimCube::SliderAndTickList>
- [ ] `PSmartArray<AnimCube::SliderAndTickList>::StreamPack` · 00698060 · legions 00637330

### SmartArray<AnimCube::SliderAndTickList>
- [ ] `SmartArray<AnimCube::SliderAndTickList>::AddToEnd` · 00698100

### AnimCube
- [ ] `AnimCube::CopyMe` · 00698160
- [-] `AnimCube::FromFileNode` · 00698230 — text source format (admin tools)

## `portal/shared/src/animsys/animmodules/animinterpolator.cpp.c` (0/16)

### AMLook
- [ ] `AMLook::DataFromFileNode` · 006919c0

### DBAnimator
- [ ] `DBAnimator::GetDBOType` · 006919d0

### AnimInterpolator
- [ ] `AnimInterpolator::GetSubDataIDs` · 006919e0 · legions 00631c10
- [ ] `AnimInterpolator::CheckAnimationBoneNum` · 00691a10 · legions 00631c40
- [ ] `AnimInterpolator::StreamPack` · 00691a90 · legions 00631cc0
- [ ] `AnimInterpolator::GetAnimatorWeights` · 00691ad0 · legions 00631d00
- [ ] `AnimInterpolator::Animate` · 00691b70
- [ ] `AnimInterpolator::GetPeriod` · 00691ca0
- [ ] `AnimInterpolator::Destroy` · 00691dc0 · legions 00631f90
- [ ] `AnimInterpolator::AnimInterpolator` · 00691df0 · legions 00631fc0
- [ ] `AnimInterpolator::~AnimInterpolator` · 00691e10 · legions 00631fe0
- [ ] `AnimInterpolator::GetRecursiveSubDataIDs` · 00691ec0
- [ ] `AnimInterpolator::HasAnimator` · 00691f90
- [ ] `AnimInterpolator::CopyMe` · 00692100

### PList<Trigger>
- [ ] `PList<Trigger>::UnPack` · 006921b0 · legions 00632334

### AnimInterpolator
- [ ] `AnimInterpolator::GetTriggers` · 00692360

## `portal/shared/src/animsys/animmodules/axiscube.cpp.c` (0/15)

### AnimMap
- [ ] `AnimMap::GetDBOType` · 00695320

### AnimCube
- [ ] `AnimCube::GetSubDataIDs` · 00695330 · legions 00635320

### AxisCube
- [ ] `AxisCube::HasAnimator` · 00695360 · legions 00635350
- [ ] `AxisCube::CheckAnimationBoneNum` · 006953e0
- [ ] `AxisCube::StreamPack` · 00695470 · legions 00635460
- [ ] `AxisCube::GetAnimatorWeights` · 006954c0 · legions 006354b0
- [ ] `AxisCube::Destroy` · 006955c0 · legions 006355b0
- [ ] `AxisCube::AxisCube` · 006955f0 · legions 006355e0
- [ ] `AxisCube::~AxisCube` · 00695620 · legions 00635610
- [ ] `AxisCube::GetRecursiveSubDataIDs` · 006956a0
- [ ] `AxisCube::Animate` · 00695770
- [ ] `AxisCube::GetPeriod` · 00695920
- [ ] `AxisCube::CopyMe` · 00695a80 · legions 006359e0
- [-] `AxisCube::FromFileNode` · 00695af0 — text source format (admin tools)
- [ ] `AxisCube::GetTriggers` · 00695e90

## `portal/shared/src/animsys/behavior.cpp.c` (0/2)

### Behavior
- [ ] `Behavior::Behavior` · 0068f360 · legions 0062fee0
- [ ] `Behavior::StreamPack` · 0068f380

## `portal/shared/src/animsys/behaviorparams.cpp.c` (1/6)

### BehaviorParams
- [ ] `BehaviorParams::IsValid` · 00678fe0 · legions 006233b0
- [ ] `BehaviorParams::get_pack_bitfield` · 00679000 · legions 006233d0
- [x] `BehaviorParams::StreamPack` · 00679220 · legions 006235f0
- [ ] `BehaviorParams::SetDefaults` · 006795b0 · legions 00623980
- [ ] `BehaviorParams::BehaviorParams` · 00679660 · legions 00623a30
- [ ] `BehaviorParams::BehaviorParams` · 006796a0 · legions 00623a70

## `portal/shared/src/animsys/behaviortable.cpp.c` (1/49)

### BehaviorTable
- [ ] `BehaviorTable::GetInitBehaviors` · 0067cfa0 · legions 006253d0

### SliderData
- [x] `SliderData::StreamPack` · 0067cfb0 · legions 005f5840

### BehaviorTable
- [ ] `BehaviorTable::StreamPack` · 0067d000 · legions 006253e0
- [ ] `BehaviorTable::GetLayerNum` · 0067d140 · legions 00625520

### HashIterator<ModeTransitionKey,float>
- [ ] `HashIterator<ModeTransitionKey,float>::operator++` · 0067d320 · legions 00625700

### IntrusiveHashIterator<ModeTransitionKey,HashTableData<ModeTransitionKey,float>_*>
- [ ] `IntrusiveHashIterator<ModeTransitionKey,HashTableData<ModeTransitionKey,float>_*>::operator++` · 0067d360

### HashIterator<ReferenceIDWrapper,unsigned_long>
- [ ] `HashIterator<ReferenceIDWrapper,unsigned_long>::operator++` · 0067d3b0 · legions 00625740

### IntrusiveHashIterator<ComboKey,HashTableData<ComboKey,ComboInfo>_*>
- [ ] `IntrusiveHashIterator<ComboKey,HashTableData<ComboKey,ComboInfo>_*>::operator++` · 0067d3f0

### HashIterator<ModeStateKey,int>
- [ ] `HashIterator<ModeStateKey,int>::operator++` · 0067d440 · legions 00625780

### BehaviorTable
- [ ] `BehaviorTable::ParseLayersNode` · 0067d660
- [ ] `BehaviorTable::ParseInitBehaviorsNode` · 0067d820
- [ ] `BehaviorTable::ParseInitModeNode` · 0067d980

### HashIterator<ModeTransitionKey,float>
- [ ] `HashIterator<ModeTransitionKey,float>::operator++` · 0067da40

### HashIterator<ComboKey,ComboInfo>
- [ ] `HashIterator<ComboKey,ComboInfo>::operator++` · 0067da70

### ParseHelper<EmpGenParse>
- [ ] `ParseHelper<EmpGenParse>::~ParseHelper<EmpGenParse>` · 0067dc80 · legions 006223a0

### SmartArray<PStringBase<char>_>
- [ ] `SmartArray<PStringBase<char>_>::SetNElements` · 0067dda0

### BehaviorTable
- [ ] `BehaviorTable::GetModeTransitionRampTime` · 0067ddd0 · legions 00625b10
- [ ] `BehaviorTable::GetComboInfo` · 0067de10 · legions 00625b50
- [ ] `BehaviorTable::GetBehaviorRampTime` · 0067de90 · legions 00625bd0
- [ ] `BehaviorTable::GetBehaviorInfo` · 0067def0 · legions 00625c30
- [ ] `BehaviorTable::GetTransitionInfo` · 0067df60 · legions 00625ca0

### HashTable<ModeTransitionKey,float>
- [ ] `HashTable<ModeTransitionKey,float>::operator=` · 0067df90 · legions 00625cd0

### HashTable<ComboKey,ComboInfo>
- [ ] `HashTable<ComboKey,ComboInfo>::operator=` · 0067e130 · legions 00625df0

### HashTable<unsigned_long,BehaviorInfo>
- [ ] `HashTable<unsigned_long,BehaviorInfo>::operator=` · 0067e390 · legions 00625fb0

### BehaviorTable
- [-] `BehaviorTable::ToFileNode` · 0067e9b0 — text source format (admin tools)
- [ ] `BehaviorTable::ParseBehaviorInfoNode` · 00680180

### PList<ModeStateKey>
- [ ] `PList<ModeStateKey>::PList<ModeStateKey>` · 00680690 · legions 006266f0

### PList<OpacityAtom>
- [ ] `PList<OpacityAtom>::GetPackSize` · 006806d0

### PList<ModeStateKey>
- [ ] `PList<ModeStateKey>::UnPack` · 00680710

### BehaviorTable
- [ ] `BehaviorTable::ParseComboNode` · 00680830
- [ ] `BehaviorTable::ParseBehaviorsDefinition` · 00681460

### PHashTable<ModeTransitionKey,float>
- [ ] `PHashTable<ModeTransitionKey,float>::StreamPack` · 00681eb0

### PHashTable<ComboKey,ComboInfo>
- [ ] `PHashTable<ComboKey,ComboInfo>::StreamPack` · 006821d0

### PHashTable<ModeTransitionKey,TransitionInfo>
- [ ] `PHashTable<ModeTransitionKey,TransitionInfo>::StreamPack` · 00682570

### IntrusiveHashIterator<ModeTransitionKey,HashTableData<ModeTransitionKey,TransitionInfo>_*>
- [ ] `IntrusiveHashIterator<ModeTransitionKey,HashTableData<ModeTransitionKey,TransitionInfo>_*>::operator++` · 00682750 · legions 00627040

### PHashTable<unsigned_long,BehaviorInfo>
- [ ] `PHashTable<unsigned_long,BehaviorInfo>::StreamPack` · 00682850

### PList<ModeStateKey>
- [ ] `PList<ModeStateKey>::operator=` · 00682a50 · legions 006273d0

### BehaviorTable
- [ ] `BehaviorTable::Destroy` · 00682eb0 · legions 006277e0

### HashTable<ModeStateKey,PList<ModeStateKey>_>
- [ ] `HashTable<ModeStateKey,PList<ModeStateKey>_>::operator=` · 00682f00 · legions 00627830

### BehaviorTable
- [ ] `BehaviorTable::ParseModeTransitionNode` · 00683140

### PHashTable<ModeStateKey,PList<ModeStateKey>_>
- [ ] `PHashTable<ModeStateKey,PList<ModeStateKey>_>::StreamPack` · 006844d0 · legions 00627a00
- [ ] `PHashTable<ModeStateKey,PList<ModeStateKey>_>::~PHashTable<ModeStateKey,PList<ModeStateKey>_>` · 006846d0 · legions 00627c70

### BehaviorTable
- [ ] `BehaviorTable::BehaviorTable` · 00684780 · legions 00627d20
- [ ] `BehaviorTable::GetDBOType` · 00684910
- [ ] `BehaviorTable::~BehaviorTable` · 00684920 · legions 00627ec0
- [ ] `BehaviorTable::GetNextTransitionAlongModePath` · 00684b00 · legions 006280a0
- [ ] `BehaviorTable::CopyInto` · 006851d0 · legions 00628770
- [-] `BehaviorTable::FromFileNode` · 00685260 — text source format (admin tools)
- [ ] `BehaviorTable::Allocate` · 00685560

## `portal/shared/src/animsys/behaviortrack.cpp.c` (0/4)

### BehaviorTrack
- [ ] `BehaviorTrack::Update` · 0068f520 · legions 006300a0
- [ ] `BehaviorTrack::~BehaviorTrack` · 0068f980 · legions 00630500

### (free functions)
- [ ] `STREAMPACK_STL_SET` · 0068fa20 · legions 006305a0

### BehaviorTrack
- [ ] `BehaviorTrack::StreamPack` · 0068fb60 · legions 006306e0

## `portal/shared/src/animsys/behaviortracker.cpp.c` (0/34)

### BehaviorTracker
- [ ] `BehaviorTracker::SendAndClearCallback` · 0068bba0 · legions 0062cbd0
- [ ] `BehaviorTracker::ProcessParamFlags` · 0068bbf0 · legions 0062cc20
- [ ] `BehaviorTracker::ProcessRampingDownParamFlags` · 0068bd50 · legions 0062cd80
- [ ] `BehaviorTracker::StreamPack` · 0068be30 · legions 0062ce60

### SmartArray<WeenieHook>
- [ ] `SmartArray<WeenieHook>::operator=` · 0068c0c0 · legions 0062d0f0

### PSmartArray<WeenieHook>
- [ ] `PSmartArray<WeenieHook>::StreamPack` · 0068c140 · legions 0062d170

### BehaviorTracker
- [ ] `BehaviorTracker::Destroy` · 0068c1e0 · legions 0062d210
- [ ] `BehaviorTracker::AddTriggers` · 0068c290 · legions 0062d2c0
- [ ] `BehaviorTracker::CheckActiveBehaviorsForDynamicFPCamera` · 0068c300 · legions 0062d330
- [ ] `BehaviorTracker::ReleaseBehavior` · 0068c400 · legions 0062d430
- [ ] `BehaviorTracker::GetTrack` · 0068c4c0 · legions 0062d4f0
- [ ] `BehaviorTracker::IsDoingBehavior` · 0068c500 · legions 0062d530
- [ ] `BehaviorTracker::~BehaviorTracker` · 0068c640 · legions 0062d670
- [ ] `BehaviorTracker::Update` · 0068c720 · legions 0062d750
- [ ] `BehaviorTracker::StopBehavior` · 0068cbd0 · legions 0062dbf0
- [ ] `BehaviorTracker::StopAllWeenieBehaviors` · 0068cda0 · legions 0062ddc0
- [ ] `BehaviorTracker::StopLayer` · 0068ce80 · legions 0062dea0

### _STL::_Rb_tree<unsigned_long,unsigned_long,_STL::_Identity<unsigned_long>,_STL::less<unsigned_long>,_STL::allocator<unsigned_long>_>
- [ ] `_STL::_Rb_tree<unsigned_long,unsigned_long,_STL::_Identity<unsigned_long>,_STL::less<unsigned_long>,_STL::allocator<unsigned_long>_>::operator=` · 0068cfd0 · legions 0062dff0

### BehaviorTracker
- [ ] `BehaviorTracker::StopMoveToCancelsBehaviors` · 0068d0b0 · legions 0062e0d0

### AnimTrack
- [ ] `AnimTrack::~AnimTrack` · 0068d270 · legions 0062e290

### BehaviorTracker
- [ ] `BehaviorTracker::BehaviorTracker` · 0068d340 · legions 0062e360

### AnimTrack
- [ ] `AnimTrack::operator=` · 0068d3e0 · legions 0062e400
- [ ] `AnimTrack::AnimTrack` · 0068d460 · legions 0062e480
- [ ] `AnimTrack::IsRampingDown` · 0068d4e0

### BehaviorTrack
- [ ] `BehaviorTrack::BehaviorTrack` · 0068d520 · legions 0062e540
- [ ] `BehaviorTrack::IsRampingDown` · 0068d5b0

### AnimTrack
- [ ] `AnimTrack::AnimTrack` · 0068d5e0 · legions 0062e600

### BehaviorTracker
- [ ] `BehaviorTracker::GetNetworkBehaviorParams` · 0068d690 · legions 0062e6b0

### BehaviorTrack
- [ ] `BehaviorTrack::operator=` · 0068d710 · legions 0062e730
- [ ] `BehaviorTrack::BehaviorTrack` · 0068d800 · legions 0062e820

### PHashTable<unsigned_long,BehaviorTrack>
- [ ] `PHashTable<unsigned_long,BehaviorTrack>::StreamPack` · 0068d900

### BehaviorTracker
- [ ] `BehaviorTracker::Init` · 0068db80 · legions 0062eba0
- [ ] `BehaviorTracker::DoBehavior` · 0068dfc0 · legions 0062efe0
- [ ] `BehaviorTracker::DoAutonomousBehavior` · 0068e410 · legions 0062f430

## `portal/shared/src/animsys/chartemplate.cpp.c` (0/26)

### CharTemplate
- [ ] `CharTemplate::GetCharTemplate` · 0067a6a0
- [ ] `CharTemplate::ReleaseCharTemplate` · 0067a700 · legions 006240f0
- [ ] `CharTemplate::GetDefaultElement` · 0067a720
- [ ] `CharTemplate::GetDefaultMode` · 0067a730
- [ ] `CharTemplate::FillDataGraph` · 0067a740
- [ ] `CharTemplate::StreamPack` · 0067a7d0 · legions 00624200
- [ ] `CharTemplate::SaveToFile` · 0067a9f0
- [ ] `CharTemplate::Destroy` · 0067ad30 · legions 00624390

### SmartArray<ParseHelper<EmpGenParse>::ParseHandler>
- [ ] `SmartArray<ParseHelper<EmpGenParse>::ParseHandler>::AddToEnd` · 0067ae40 · legions 006b6b60

### CharTemplate
- [ ] `CharTemplate::GetBehavior` · 0067ae90 · legions 006244a0
- [ ] `CharTemplate::GetSlider` · 0067aed0
- [ ] `CharTemplate::IsUserControlled` · 0067af10

### HashTable<unsigned_long,Behavior>
- [ ] `HashTable<unsigned_long,Behavior>::operator=` · 0067af70 · legions 006245a0

### HashTable<unsigned_long,Slider>
- [ ] `HashTable<unsigned_long,Slider>::operator=` · 0067b0f0 · legions 006246b0

### HashTable<unsigned_long,Mode>
- [ ] `HashTable<unsigned_long,Mode>::operator=` · 0067b260 · legions 006247b0

### CharTemplate
- [ ] `CharTemplate::~CharTemplate` · 0067b690 · legions 00624b70
- [ ] `CharTemplate::GetDBOType` · 0067b780
- [ ] `CharTemplate::ParseEMPsNode` · 0067b790

### ParseHelper<EmpGenParse>
- [ ] `ParseHelper<EmpGenParse>::AddHandler` · 0067c640 · legions 006b6ff0

### CharTemplate
- [ ] `CharTemplate::CopyInto` · 0067c720 · legions 00624c90
- [-] `CharTemplate::FromFileNode` · 0067c780 — text source format (admin tools)

### PHashTable<unsigned_long,Behavior>
- [ ] `PHashTable<unsigned_long,Behavior>::StreamPack` · 0067c8c0 · legions 00624cf0

### PHashTable<unsigned_long,Slider>
- [ ] `PHashTable<unsigned_long,Slider>::StreamPack` · 0067cac0

### PHashTable<unsigned_long,Mode>
- [ ] `PHashTable<unsigned_long,Mode>::StreamPack` · 0067ccc0

### CharTemplate
- [ ] `CharTemplate::CharTemplate` · 0067cec0 · legions 006252f0
- [ ] `CharTemplate::Allocate` · 0067cf90

## `portal/shared/src/animsys/combokey.cpp.c` (0/13)

### ComboKey
- [ ] `ComboKey::operator%` · 0068e9f0 · legions 0062f9d0
- [ ] `ComboKey::operator==` · 0068ea10 · legions 0062f9f0

### ModeTransitionKey
- [ ] `ModeTransitionKey::operator%` · 0068ea40 · legions 0062fa20
- [ ] `ModeTransitionKey::operator==` · 0068ea60 · legions 0062fa40

### ModeStateKey
- [ ] `ModeStateKey::operator%` · 0068eaa0 · legions 0062fa80
- [ ] `ModeStateKey::operator==` · 0068eac0 · legions 0062faa0

### ComboInfo
- [ ] `ComboInfo::IsValid` · 0068eaf0 · legions 0062fad0
- [ ] `ComboInfo::SetInvalid` · 0068eb00 · legions 0062fae0

### ComboKey
- [ ] `ComboKey::ComboKey` · 0068eb10 · legions 0062faf0

### ModeTransitionKey
- [ ] `ModeTransitionKey::ModeTransitionKey` · 0068eb30 · legions 0062fb10

### ModeStateKey
- [ ] `ModeStateKey::ModeStateKey` · 0068eb50 · legions 0062fb30

### TransitionInfo
- [ ] `TransitionInfo::TransitionInfo` · 0068eb70

### ComboKey
- [ ] `ComboKey::StreamPack` · 0068eb90 · legions 0062fb50

## `portal/shared/src/animsys/dbanimator.cpp.c` (0/16)

### DBAnimator
- [ ] `DBAnimator::DBAnimator` · 0067a440 · legions 00623ed0
- [ ] `DBAnimator::GetSubDataIDs` · 0067a460
- [ ] `DBAnimator::GetRecursiveSubDataIDs` · 0067a470 · legions 00623f00
- [ ] `DBAnimator::Animate` · 0067a480 · legions 00623f10
- [ ] `DBAnimator::GetTriggers` · 0067a490 · legions 00623f20
- [ ] `DBAnimator::GetHoldLoop` · 0067a4a0 · legions 00623f30
- [ ] `DBAnimator::GetPeriod` · 0067a4b0 · legions 00623f40
- [ ] `DBAnimator::Refresh` · 0067a4c0
- [ ] `DBAnimator::CopyInto` · 0067a4d0
- [ ] `DBAnimator::Destroy` · 0067a4f0 · legions 00623f80
- [-] `DBAnimator::LoadFromFile` · 0067a510 — text source loader, admin build only
- [ ] `DBAnimator::SaveToFile` · 0067a530
- [ ] `DBAnimator::HasAnimator` · 0067a550 · legions 00623fa0
- [ ] `DBAnimator::CheckAnimationBoneNum` · 0067a570 · legions 00623fc0
- [ ] `DBAnimator::Allocate` · 0067a5a0
- [ ] `DBAnimator::StreamPack` · 0067a5f0 · legions 00624040

## `portal/shared/src/animsys/fxscript.cpp.c` (0/16)

### FxScript
- [ ] `FxScript::StreamPack` · 006859d0 · legions 00628970

### PetInfo
- [ ] `PetInfo::operator=` · 00685a40 · legions 00418c10

### FxScript
- [ ] `FxScript::ParsePeriodNode` · 00685e20

### SmartArray<ImpulseTrigger>
- [ ] `SmartArray<ImpulseTrigger>::operator=` · 00685e50 · legions 00628d60

### Trigger
- [ ] `Trigger::Trigger` · 00685ee0 · legions 00628df0

### FxScript
- [-] `FxScript::ToFileNode` · 00686360 — text source format (admin tools)

### ParseHelper<EmpGenParse>
- [ ] `ParseHelper<EmpGenParse>::Parse` · 00686db0 · legions 006b63b0

### SmartArray<Trigger>
- [ ] `SmartArray<Trigger>::operator=` · 00687440 · legions 00629270

### SmartArray<ImpulseTrigger>
- [ ] `SmartArray<ImpulseTrigger>::AddToEnd` · 006874d0 · legions 00629300

### FxScript
- [ ] `FxScript::CopyInto` · 006877a0 · legions 006295d0

### SmartArray<Trigger>
- [ ] `SmartArray<Trigger>::AddToEnd` · 006877f0 · legions 00629620

### FxScript
- [ ] `FxScript::GetTriggers` · 00687be0 · legions 00629a10
- [ ] `FxScript::ParseImpulseNode` · 00688190
- [ ] `FxScript::ParseEffectsNode` · 006884a0
- [ ] `FxScript::ParseStopEffectsNode` · 006889e0
- [-] `FxScript::FromFileNode` · 00688d60 — text source format (admin tools)

## `portal/shared/src/animsys/fxscripttracker.cpp.c` (0/7)

### (free functions)
- [ ] `STREAMPACK` · 0068e480 · legions 0063af50

### FXScriptTracker
- [ ] `FXScriptTracker::StreamPack` · 0068e510
- [ ] `FXScriptTracker::Update` · 0068e620 · legions 0062f5b0

### PList<gmActInfo>
- [ ] `PList<gmActInfo>::GetPackSize` · 0068e700

### FXScriptTracker
- [ ] `FXScriptTracker::FXScriptTracker` · 0068e7b0 · legions 0062f790

### PList<FXTrack>
- [ ] `PList<FXTrack>::UnPack` · 0068e8d0 · legions 0062f8c4

### FXScriptTracker
- [ ] `FXScriptTracker::DoFXScript` · 0068e990 · legions 0062f970

## `portal/shared/src/animsys/fxtrack.cpp.c` (0/6)

### FXTrack
- [ ] `FXTrack::FillCache` · 00690130 · legions 00630cb0
- [ ] `FXTrack::StreamPack` · 00690190 · legions 00630d00
- [ ] `FXTrack::operator=` · 00690220 · legions 00630d90
- [ ] `FXTrack::~FXTrack` · 006902a0 · legions 00630e10
- [ ] `FXTrack::FXTrack` · 006902d0 · legions 00630e40
- [ ] `FXTrack::FXTrack` · 00690300 · legions 00630e70

## `portal/shared/src/animsys/impulsetrigger.cpp.c` (0/3)

### ImpulseTrigger
- [ ] `ImpulseTrigger::operator==` · 00689180 · legions 0062a1a0
- [ ] `ImpulseTrigger::operator<` · 006891f0 · legions 0062a210
- [ ] `ImpulseTrigger::StreamPack` · 00689210 · legions 0062a230

## `portal/shared/src/animsys/mode.cpp.c` (0/2)

### Mode
- [ ] `Mode::Mode` · 0068f400 · legions 0062ff80
- [ ] `Mode::StreamPack` · 0068f410

## `portal/shared/src/animsys/modetracker.cpp.c` (0/23)

### ModeTracker
- [ ] `ModeTracker::GetTriggerUpdate` · 0068a8b0 · legions 0062b8f0
- [ ] `ModeTracker::SetAnimationObj` · 0068a8d0 · legions 0062b910
- [ ] `ModeTracker::Init` · 0068a900 · legions 0062b940
- [ ] `ModeTracker::SetMode` · 0068a9f0 · legions 0062ba20
- [ ] `ModeTracker::DoAutonomousBehavior` · 0068aac0 · legions 0062baf0
- [ ] `ModeTracker::ReleaseBehavior` · 0068ab10 · legions 0062bb40
- [ ] `ModeTracker::StopBehavior` · 0068ab90 · legions 0062bbc0
- [ ] `ModeTracker::StopMoveToCancelsBehaviors` · 0068ac50 · legions 0062bc80
- [ ] `ModeTracker::StopLayer` · 0068acd0 · legions 0062bd00
- [ ] `ModeTracker::StopAllWeenieBehaviors` · 0068ad20 · legions 0062bd50
- [ ] `ModeTracker::GetElement` · 0068ad50 · legions 0062bd80
- [ ] `ModeTracker::IsDoingBehavior` · 0068ad70 · legions 0062bda0
- [ ] `ModeTracker::CheckForDynamicFPCamera` · 0068adb0 · legions 0062bde0
- [ ] `ModeTracker::GetNetworkBehaviorParams` · 0068add0 · legions 0062be00
- [ ] `ModeTracker::ModeTracker` · 0068adf0 · legions 0062be20
- [ ] `ModeTracker::GetDesiredModeState` · 0068ae30 · legions 0062be60
- [ ] `ModeTracker::StreamPack` · 0068ae60 · legions 0062be90
- [ ] `ModeTracker::GetModeState` · 0068b060 · legions 0062c090
- [ ] `ModeTracker::DoBehavior` · 0068b0b0 · legions 0062c0e0
- [ ] `ModeTracker::StartTransitioning` · 0068b1d0 · legions 0062c200
- [ ] `ModeTracker::DoElement` · 0068b400 · legions 0062c430
- [ ] `ModeTracker::DoMode` · 0068b450 · legions 0062c480
- [ ] `ModeTracker::Update` · 0068b510 · legions 0062c540

## `portal/shared/src/animsys/slider.cpp.c` (0/2)

### Slider
- [ ] `Slider::Slider` · 0068f3b0 · legions 0062ff30
- [ ] `Slider::StreamPack` · 0068f3c0 · legions 0062ff40

## `portal/shared/src/animsys/slidertracker.cpp.c` (0/16)

### SliderInterpolator
- [ ] `SliderInterpolator::Activate` · 006898e0 · legions 0062a900

### ModeTransitionKey
- [ ] `ModeTransitionKey::StreamPack` · 00689910 · legions 0062fba0

### SliderTracker
- [ ] `SliderTracker::StreamPack` · 00689970 · legions 0062a930

### SliderInterpolator
- [ ] `SliderInterpolator::StreamPack` · 006899c0 · legions 0062a980

### SliderTracker
- [ ] `SliderTracker::Update` · 00689ab0 · legions 0062aab0
- [ ] `SliderTracker::GetNetworkSliders` · 00689c20 · legions 0062ac20
- [ ] `SliderTracker::GetSliderCurrentValue` · 00689d60 · legions 0062ada0
- [ ] `SliderTracker::GetSliderDesiredValue` · 00689da0 · legions 0062ade0
- [ ] `SliderTracker::GetSliderVelocity` · 00689de0 · legions 0062ae20
- [ ] `SliderTracker::~SliderTracker` · 0068a130 · legions 0062b170
- [ ] `SliderTracker::DoSlider` · 0068a1d0 · legions 0062b210
- [ ] `SliderTracker::GetSliderInterpolator` · 0068a320 · legions 0062b360
- [ ] `SliderTracker::Init` · 0068a400 · legions 0062b440

### PHashTable<unsigned_long,SliderInfo>
- [ ] `PHashTable<unsigned_long,SliderInfo>::StreamPack` · 0068a490

### PHashTable<unsigned_long,SliderInterpolator>
- [ ] `PHashTable<unsigned_long,SliderInterpolator>::StreamPack` · 0068a660 · legions 0062b6a0

### SliderTracker
- [ ] `SliderTracker::SliderTracker` · 0068a840 · legions 0062b880

## `portal/shared/src/animsys/trigger.cpp.c` (0/7)

### Trigger
- [ ] `Trigger::operator==` · 00688fa0 · legions 00629fc0
- [ ] `Trigger::operator<` · 00688fe0 · legions 0062a000
- [ ] `Trigger::Trigger` · 00689000 · legions 0062a020
- [ ] `Trigger::Trigger` · 00689040 · legions 0062a060
- [ ] `Trigger::Trigger` · 00689080 · legions 0062a0a0
- [ ] `Trigger::Trigger` · 006890c0 · legions 0062a0e0
- [ ] `Trigger::StreamPack` · 00689100

## `portal/shared/src/animsys/triggerupdate.cpp.c` (0/8)

### TriggerUpdate
- [ ] `TriggerUpdate::StreamPack` · 00689290 · legions 0062a2b0
- [ ] `TriggerUpdate::IsEmpty` · 006892f0 · legions 0062a310
- [ ] `TriggerUpdate::AddWeenieHook` · 00689330 · legions 0062a350
- [ ] `TriggerUpdate::AddImpulseTrigger` · 006893b0 · legions 0062a3d0
- [ ] `TriggerUpdate::OnTrackTerminated` · 006894d0 · legions 0062a4f0
- [ ] `TriggerUpdate::CleanUp` · 00689680 · legions 0062a6a0
- [ ] `TriggerUpdate::AddEffectsTrigger` · 00689710 · legions 0062a730
- [ ] `TriggerUpdate::AddTrackTriggers` · 00689880 · legions 0062a8a0

## `portal/shared/src/animsys/validmodes.cpp.c` (0/7)

### ValidModes
- [ ] `ValidModes::StreamPack` · 00685590
- [ ] `ValidModes::Destroy` · 006855c0 · legions 00628860
- [ ] `ValidModes::ValidModes` · 006855f0 · legions 00628890
- [ ] `ValidModes::GetDBOType` · 00685620
- [ ] `ValidModes::Allocate` · 00685630
- [-] `ValidModes::ToFileNode` · 00685680 — text source format (admin tools)
- [-] `ValidModes::FromFileNode` · 00685790 — text source format (admin tools)

## `portal/shared/src/animsys/weeniehook.cpp.c` (0/4)

### WeenieHook
- [ ] `WeenieHook::operator==` · 0068f440 · legions 0062ffc0
- [ ] `WeenieHook::operator<` · 0068f470 · legions 0062fff0
- [ ] `WeenieHook::WeenieHook` · 0068f490 · legions 00630010
- [ ] `WeenieHook::StreamPack` · 0068f4c0 · legions 00630040
