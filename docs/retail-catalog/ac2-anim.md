# ac2-anim: Animation system

532 retail functions: 172 done, 1 partial, 2 to do, 357 skipped.
Legend and workflow: [README](README.md).

## `portal/shared/src/anim/charactersetup.cpp.c` (8/67)

### PlacementFrame
- [-] `PlacementFrame::Pack` · 0060d540 · legions 005ce310 — DAT writer half of the DBObj pack (CharacterSetup::Pack and the PlacementFrame/DegradeInfo packs it calls): the Legions client only unpacks setups (Legions vtable slot 3 of CharacterSetup at 0x005cf460 is the writer; callers: DBObj::SaveToFile and tool paths). The read side is ac2-anim setup.rs (Ac2Setup decode)
- [x] `PlacementFrame::UnPack` · 0060d5a0 · legions 005ce370 — ac2-anim setup.rs Ac2Setup::decode_packed placement loop (PlacementFrame::UnPack: kind, bone count, a TransformBase per bone); setup::retail_setups_decode_ordered_degrade_distances decodes every Setup of the dataset to the last byte
- [-] `PlacementFrame::SetNumBones` · 0060d630 — trivial setter / list append on plain struct during decode
- [-] `PlacementFrame::SetTransform` · 0060d690 — trivial setter / list append on plain struct during decode

### DegradeInfo
- [-] `DegradeInfo::Pack` · 0060d700 · legions 005ce400 — DAT writer half of the DBObj pack (CharacterSetup::Pack and the PlacementFrame/DegradeInfo packs it calls): the Legions client only unpacks setups (Legions vtable slot 3 of CharacterSetup at 0x005cf460 is the writer; callers: DBObj::SaveToFile and tool paths). The read side is ac2-anim setup.rs (Ac2Setup decode)
- [x] `DegradeInfo::UnPack` · 0060d800 · legions 005ce500 — ac2-anim Ac2Setup::degrade (read_degrade_info): 18 distances, low/medium/high
- [-] `DegradeInfo::DegradeInfo` · 0060d8f0 · legions 005ce5f0 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)

### CharacterSetup
- [-] `CharacterSetup::GetSubDataIDs` · 0060d940 · legions 005ce640 — GetSubDataIDs lists a record's dependent DIDs for the data-graph prefetch (DBObj::AddToDataGraph); DBOCache::GetIfInMemory does not call it, it calls an int-returning vtable slot (+0x24) once when !m_bLoaded, not this void (type, array) slot. The client reads every DID on demand from the local DATs and Ac2DbAnimator's loader recurses by DID (dbanimator/tree.rs Loader::node), so no consumer exists
- [-] `CharacterSetup::SetBoundingObject` · 0060d980 — trivial setter / list append on plain struct during decode
- [-] `CharacterSetup::SetNumSphere` · 0060db10 — trivial setter / list append on plain struct during decode
- [-] `CharacterSetup::SetNumCylsphere` · 0060db70 — trivial setter / list append on plain struct during decode

### gmCharGenResult
- [x] `gmCharGenResult::SetImplement` · 0060dbd0 · legions 005ce810 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)

### CharacterSetup
- [-] `CharacterSetup::IsBoneType` · 0060dbe0 — CharacterSetup is the authoring tool's setup object: no Legions address, no callers in the calls table (virtual slots of the tool's skeleton helpers). IsBoneType/GetBoneType/GetParentIndex read bone_info id and parent_index, which Ac2Setup holds as Ac2Bone name and parent; InitLoad builds the mesh-name render_info hash the tool needs. The client reads bones from the DAT (setup.rs)
- [-] `CharacterSetup::GetBoneType` · 0060dc10 — CharacterSetup is the authoring tool's setup object: no Legions address, no callers in the calls table (virtual slots of the tool's skeleton helpers). IsBoneType/GetBoneType/GetParentIndex read bone_info id and parent_index, which Ac2Setup holds as Ac2Bone name and parent; InitLoad builds the mesh-name render_info hash the tool needs. The client reads bones from the DAT (setup.rs)

### HashBaseIter<unsigned_long>
- [-] `HashBaseIter<unsigned_long>::Next` · 0060dc30 · legions 005ce870 — hash/list iterator internals (Rust iterators)
- [-] `HashBaseIter<unsigned_long>::DeleteCurrent` · 0060dc80 · legions 005ce8c0 — hash/list iterator internals (Rust iterators)

### CharacterSetup
- [-] `CharacterSetup::SetSphere` · 0060dd10 — trivial setter / list append on plain struct during decode

### LocationType
- [-] `LocationType::LocationType` · 0060dd50 · legions 005ce950 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)

### PlacementFrame
- [-] `PlacementFrame::~PlacementFrame` · 0060de10 · legions 005cea10 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)
- [-] `PlacementFrame::PlacementFrame` · 0060de70 · legions 005cea70 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)

### MeshInfo
- [-] `MeshInfo::MeshInfo` · 0060dfa0 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)

### BoneInfo
- [-] `BoneInfo::BoneInfo` · 0060dfd0 · legions 005cebd0 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)

### LocationType
- [-] `LocationType::SetTypeName` · 0060e000 — trivial setter / list append on plain struct during decode

### PlacementFrame
- [-] `PlacementFrame::SetTypeName` · 0060e060 — trivial setter / list append on plain struct during decode

### BoneInfo
- [-] `BoneInfo::SetTypeName` · 0060e0c0 — trivial setter / list append on plain struct during decode

### CharacterSetup
- [-] `CharacterSetup::SetNumMesh` · 0060e270 · legions 005ced50 — trivial setter / list append on plain struct during decode

### MeshInfo
- [-] `MeshInfo::operator=` · 0060e2d0 · legions 005cedb0 — operator= of plain data (Rust value assignment)

### CharacterSetup
- [-] `CharacterSetup::SetNumBones` · 0060e320 · legions 005cee00 — trivial setter / list append on plain struct during decode

### BoneInfo
- [-] `BoneInfo::operator=` · 0060e380 · legions 005cee60 — operator= of plain data (Rust value assignment)

### CharacterSetup
- [-] `CharacterSetup::InitFileLoad` · 0060e3d0 — only caller is CharacterSetup::LoadFromFile (the authoring tool's text .setup load; calls table), no Legions body. It computes the height, radius, selection sphere, step heights and physics bounds that the DAT stores and CharacterSetup::UnPack reads back (ac2-assets ac2_physics_mesh/setup.rs); ac2-physics ObjectShape::new derives the containment radius and box for objects from the BSPs, spheres and cylinders

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
- [-] `CharacterSetup::AddPlacementFrame` · 0060fad0 · legions 005cef20 — trivial setter / list append on plain struct during decode
- [-] `CharacterSetup::AddHoldingLocation` · 0060fb70 · legions 005cefc0 — trivial setter / list append on plain struct during decode
- [-] `CharacterSetup::AddConnectionPoint` · 0060fc30 · legions 005cf080 — trivial setter / list append on plain struct during decode
- [-] `CharacterSetup::SetCylsphere` · 0060fcf0 — trivial setter / list append on plain struct during decode
- [-] `CharacterSetup::GetHoldingLocation` · 0060fd40 · legions 005cf140 — trivial accessor / write-side pack size on setup records
- [-] `CharacterSetup::GetConnectionPoint` · 0060fd80 · legions 005cf180 — trivial accessor / write-side pack size on setup records
- [x] `CharacterSetup::GetPlacementFrame` · 0060fdc0 · legions 005cf1c0 — ac2-render object/skeleton.rs placement(): CharacterSetup::GetPlacementFrame bone transforms
- [-] `CharacterSetup::GetPackSize` · 0060fe10 — trivial accessor / write-side pack size on setup records
- [-] `CharacterSetup::Pack` · 00610060 — DAT writer half of the DBObj pack (CharacterSetup::Pack and the PlacementFrame/DegradeInfo packs it calls): the Legions client only unpacks setups (Legions vtable slot 3 of CharacterSetup at 0x005cf460 is the writer; callers: DBObj::SaveToFile and tool paths). The read side is ac2-anim setup.rs (Ac2Setup decode)
- [x] `CharacterSetup::UnPack` · 006105a0 · legions 005cf9a0 — ac2-anim setup.rs: CharacterSetup::UnPack decode order (file doc cites it)
- [-] `CharacterSetup::ToFileNode` · 00610b60 — text source format (admin tools)
- [-] `CharacterSetup::ToFileNodeSetinfo` · 00611390 — admin text source format (ToFileNode), admin tools
- [-] `CharacterSetup::FromFileNode` · 00611440 — text source format (admin tools)
- [-] `CharacterSetup::CopyInto` · 00611b20 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing
- [-] `CharacterSetup::SaveToFile` · 00612030 — admin text source format (ToFileNode/FromFileNode/Parse*Node), admin tools
- [-] `CharacterSetup::LoadFromFile` · 00612190 — text source loader, admin build only
- [-] `CharacterSetup::Destroy` · 00612410 · legions 005d04d0 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing
- [x] `CharacterSetup::SetRestBones` · 006124f0 · legions 005d05b0 — ac2-render object/skeleton.rs rest_matrices
- [x] `CharacterSetup::InitRestBones` · 00612670 · legions 005d0730 — ac2-render object/skeleton.rs rest_matrices
- [-] `CharacterSetup::~CharacterSetup` · 006126d0 · legions 005d0790 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)
- [-] `CharacterSetup::GetParentIndex` · 00612760 — CharacterSetup is the authoring tool's setup object: no Legions address, no callers in the calls table (virtual slots of the tool's skeleton helpers). IsBoneType/GetBoneType/GetParentIndex read bone_info id and parent_index, which Ac2Setup holds as Ac2Bone name and parent; InitLoad builds the mesh-name render_info hash the tool needs. The client reads bones from the DAT (setup.rs)
- [x] `CharacterSetup::FindBoneType` · 00612780 · legions 005d0840 — ObjectSkeleton::named_bones (ac2-render object/skeleton.rs): every part group's bones by name in CPartGroup::FindBoneType order (own bones, then each child group's search), the first match wins; scene/particles.rs attaches a bone node to it (Bones::named). Real-data test object/fx_bone_tests.rs
- [-] `CharacterSetup::InitLoad` · 006127e0 — CharacterSetup is the authoring tool's setup object: no Legions address, no callers in the calls table (virtual slots of the tool's skeleton helpers). IsBoneType/GetBoneType/GetParentIndex read bone_info id and parent_index, which Ac2Setup holds as Ac2Bone name and parent; InitLoad builds the mesh-name render_info hash the tool needs. The client reads bones from the DAT (setup.rs)
- [-] `CharacterSetup::CharacterSetup` · 00612860 · legions 005d0920 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)

## `portal/shared/src/anim/setup.cpp.c` (0/4)

### CSetup
- [-] `CSetup::CSetup` · 0072f380 · legions 006acc40 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)
- [-] `CSetup::GetDBOType` · 0072f3a0 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing
- [-] `CSetup::CachePhysicsInformation` · 0072f3b0 — only caller is CharacterSetup::LoadFromFile (the authoring tool's text .setup load; calls table), no Legions body. It computes the height, radius, selection sphere, step heights and physics bounds that the DAT stores and CharacterSetup::UnPack reads back (ac2-assets ac2_physics_mesh/setup.rs); ac2-physics ObjectShape::new derives the containment radius and box for objects from the BSPs, spheres and cylinders
- [-] `CSetup::Allocate` · 0072f4b0 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing

## `portal/shared/src/animsys/animator.cpp.c` (1/5)

### AMLook
- [-] `AMLook::AMLook` · 0068ebe0 · legions 0062fc00 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)

### AMRandom
- [-] `AMRandom::AMRandom` · 0068ec20 · legions 0062fc40 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)

### Animator
- [x] `Animator::Create` · 0068ec70 · legions 0062fc90 — ac2-anim dbanimator/tree.rs AnimatorRecord: Animator::Create type word then per-type StreamPack
- [-] `Animator::FromFile` · 0068ed70 — admin text/file source format, admin tools
- [-] `Animator::SaveToFile` · 0068f1a0 — admin text source format (ToFileNode/FromFileNode/Parse*Node), admin tools

## `portal/shared/src/animsys/animblender.cpp.c` (3/4)

### AnimBlender
- [x] `AnimBlender::Animate` · 0068fde0 · legions 00630960 — ac2-anim dbanimator / ac2-physics anim_pose (slice 3)
- [-] `AnimBlender::~AnimBlender` · 00690040 · legions 00630bc0 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)
- [x] `AnimBlender::SetNumAnimations` · 00690090 · legions 00630c10 — ac2-anim dbanimator / ac2-physics anim_pose (slice 3)
- [x] `AnimBlender::AddAnimation` · 006900c0 · legions 00630c40 — ac2-anim dbanimator / ac2-physics anim_pose (slice 3)

## `portal/shared/src/animsys/animcompositor.cpp.c` (4/7)

### AnimCompositor
- [-] `AnimCompositor::Destroy` · 0068b6c0 · legions 0062c6f0 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing
- [x] `AnimCompositor::Clear` · 0068b730 · legions 0062c760 — ac2-anim dbanimator / ac2-physics anim_pose (slice 3)
- [x] `AnimCompositor::AddAnimation` · 0068b770 · legions 0062c7a0 — ac2-anim dbanimator / ac2-physics anim_pose (slice 3)
- [x] `AnimCompositor::Init` · 0068b7a0 · legions 0062c7d0 — ac2-anim dbanimator/compose.rs Ac2Compositor::pose: one AnimBlender per layer of the table, layer weights 1.0, tweak flags stored in reverse layer order, per-bone temp arrays; effect triggers and weenie hooks go through Ac2TriggerUpdate (behavior/triggers.rs); layering tests in dbanimator/tests.rs
- [x] `AnimCompositor::Animate` · 0068b890 · legions 0062c8c0 — ac2-anim dbanimator / ac2-physics anim_pose (slice 3)
- [-] `AnimCompositor::~AnimCompositor` · 0068bad0 · legions 0062cb00 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)
- [-] `AnimCompositor::AnimCompositor` · 0068bb40 · legions 0062cb70 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)

## `portal/shared/src/animsys/animinfo.cpp.c` (3/12)

### BaseList<unsigned_long,11>
- [-] `BaseList<unsigned_long,11>::GetReferences` · 00739420 · legions 006b4f60 — BaseList<ulong,11>::GetReferences is 'return 1' (empty body; the index name is a folded duplicate)

### AnimInfo
- [x] `AnimInfo::GetHoldLoop` · 00739430 · legions 006b4f70 — ac2-anim dbanimator / ac2-physics anim_pose (slice 3)
- [x] `AnimInfo::StreamPack` · 00739460 · legions 006b4fa0 — ac2-anim dbanimator / ac2-physics anim_pose (slice 3)
- [-] `AnimInfo::ParseLoopNode` · 007397a0 — admin text source format (ToFileNode/FromFileNode/Parse*Node), admin tools
- [-] `AnimInfo::ParseComboAttackNode` · 00739990 · legions 006b5570 — EMP text parsers (AnimInfo::ParseComboAttackNode and ParseHelper<EmpGenParse>::Parse): callers are EmpGenParse::ParseEmp and the FromFileNode loaders (tool text sources); the DVD client loads the binary DAT records (Animator::Create + StreamPack)
- [-] `AnimInfo::ToFileNode` · 00739a90 · legions 006b5670 — text source format (admin tools)
- [x] `AnimInfo::GetTriggers` · 0073ab60 · legions 006b6db0 — ac2-anim behavior/triggers + dbanimator/triggers (slice 4): effect, hook and impulse triggers through TriggerUpdate
- [-] `AnimInfo::ParseImpulseNode` · 0073b010 — admin text source format (ToFileNode/FromFileNode/Parse*Node), admin tools
- [-] `AnimInfo::ParseEffectsNode` · 0073b320 — admin text source format (ToFileNode/FromFileNode/Parse*Node), admin tools
- [-] `AnimInfo::ParseStopEffectsNode` · 0073b860 — admin text source format (ToFileNode/FromFileNode/Parse*Node), admin tools
- [-] `AnimInfo::ParseWeenieHookNode` · 0073bbe0 — admin text source format (ToFileNode/FromFileNode/Parse*Node), admin tools
- [-] `AnimInfo::FromFileNode` · 0073c0a0 · legions 006b83b0 — text source format (admin tools)

## `portal/shared/src/animsys/animmap.cpp.c` (2/13)

### AnimMap
- [x] `AnimMap::StreamPack` · 006796f0 · legions 00623ac0 — ac2-anim animation.rs Ac2AnimMap::decode_packed (AnimMap::StreamPack: DBObj identity, then mMapping); loaded by the human AnimMap in dbanimator::tests::retail_human_animators_load_and_pose_finitely
- [-] `AnimMap::GetSubDataIDs` · 006797f0 — GetSubDataIDs lists a record's dependent DIDs for the data-graph prefetch (DBObj::AddToDataGraph); DBOCache::GetIfInMemory does not call it, it calls an int-returning vtable slot (+0x24) once when !m_bLoaded, not this void (type, array) slot. The client reads every DID on demand from the local DATs and Ac2DbAnimator's loader recurses by DID (dbanimator/tree.rs Loader::node), so no consumer exists
- [-] `AnimMap::Destroy` · 00679860 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing
- [-] `AnimMap::ToFileNode` · 00679870 — text source format (admin tools)
- [-] `AnimMap::CacheAnimations` · 00679aa0 · legions 00623ba0 — retail DB cache prefetch/flush (AGENTS exception: read on demand, free on last drop)
- [x] `AnimMap::GetAnimator` · 00679ca0 · legions 00623d70 — ac2-anim dbanimator/tree.rs load_ac2_db_animator: AnimMap::GetAnimator
- [-] `AnimMap::~AnimMap` · 00679ce0 · legions 00623db0 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)
- [-] `AnimMap::ParseMappingNode` · 00679d60 — admin text source format (ToFileNode/FromFileNode/Parse*Node), admin tools
- [-] `AnimMap::ParseMapsNode` · 0067a010 — admin text source format (ToFileNode/FromFileNode/Parse*Node), admin tools
- [-] `AnimMap::CopyInto` · 0067a210 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing
- [-] `AnimMap::FromFileNode` · 0067a230 — text source format (admin tools)
- [-] `AnimMap::AnimMap` · 0067a3e0 · legions 00623e70 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)
- [-] `AnimMap::Allocate` · 0067a430 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing

## `portal/shared/src/animsys/animmodule.cpp.c` (0/4)

### AnimModule
- [-] `AnimModule::AnimModule` · 00690330 · legions 00630ea0 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)
- [-] `AnimModule::FromFileNode` · 00690340 — text source format (admin tools)
- [-] `AnimModule::FromFile` · 00690630 — admin file loader/writer (no DVD client path), admin tools
- [-] `AnimModule::ToFileNode` · 00690b00 — text source format (admin tools)

## `portal/shared/src/animsys/animmodules/amangle.cpp.c` (2/13)

### BSPLEAF
- [-] `BSPLEAF::ClipToCellBSP` · 00692890 — returns 1: AMLook::CheckAnimationBoneNum under a folded name. tree-validation query (HasAnimator / CheckAnimationBoneNum fills an error-string list): its only callers are the same query on the sibling animators (calls table), no root caller in admin 7148 or Legions, so it is reached only from the authoring tools' checks; the client never validates animators

### AMLook
- [x] `AMLook::GetPeriod` · 006928a0 — ac2-anim dbanimator/look.rs (AMLook): GetPeriod 1.0, GetTriggers adds nothing, Animate turns the neck and head bones by the 0x40000007-9 look sliders (x87 order, f32 stores); tests in dbanimator/interpolator_tests.rs. The record 0x0500002a decodes to the last byte (retail_animator_records_decode...)

### CDataMesh
- [-] `CDataMesh::GetDBOType` · 006928b0 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing

### AMLook
- [x] `AMLook::GetTriggers` · 006928c0 — ac2-anim dbanimator/look.rs (AMLook): GetPeriod 1.0, GetTriggers adds nothing, Animate turns the neck and head bones by the 0x40000007-9 look sliders (x87 order, f32 stores); tests in dbanimator/interpolator_tests.rs. The record 0x0500002a decodes to the last byte (retail_animator_records_decode...)

### AMAngle
- [ ] `AMAngle::StreamPack` · 006928d0 · legions 00632a20 — AMAngle (type 0x4000000a, one DAT record 0x050002fd) decodes to Ac2DbAnimator::Other: no pose. Not ported: Animate reads the body's frame and a ground plane (PHYSICSOBJREP position/plane slots) and tilts the bones of its BoneAngleMod list; StreamPack is BoneAngleMod list, min/max x and y
- [ ] `AMAngle::Animate` · 00692960 · legions 00632b10 — AMAngle (type 0x4000000a, one DAT record 0x050002fd) decodes to Ac2DbAnimator::Other: no pose. Not ported: Animate reads the body's frame and a ground plane (PHYSICSOBJREP position/plane slots) and tilts the bones of its BoneAngleMod list; StreamPack is BoneAngleMod list, min/max x and y
- [-] `AMAngle::DataFromFileNode` · 00692e50 — admin text source format (ToFileNode/FromFileNode/Parse*Node), admin tools
- [-] `AMAngle::Destroy` · 00693210 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing

### PList<TurbineUserStatus>
- [-] `PList<TurbineUserStatus>::GetPackSize` · 00693230 — container serialization/helper (Rust collections; elements decoded by their owning class)

### PList<BoneAngleMod>
- [-] `PList<BoneAngleMod>::UnPack` · 00693270 · legions 006333f4 — container serialization/helper (Rust collections; elements decoded by their owning class)

### AMAngle
- [-] `AMAngle::AMAngle` · 00693390 · legions 00633500 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)

### PList<BoneAngleMod>
- [-] `PList<BoneAngleMod>::operator=` · 006933e0 · legions 00633550 — container operator= (Rust collections)

### AMAngle
- [-] `AMAngle::CopyMe` · 00693470 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing

## `portal/shared/src/animsys/animmodules/amhuman.cpp.c` (4/13)

### AMHuman
- [-] `AMHuman::AMHuman` · 00694960 · legions 006349f0 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)

### CContinSoundInfo
- [-] `CContinSoundInfo::GetSubDataIDs` · 00694980 · legions 00634a10 — GetSubDataIDs lists a record's dependent DIDs for the data-graph prefetch (DBObj::AddToDataGraph); DBOCache::GetIfInMemory does not call it, it calls an int-returning vtable slot (+0x24) once when !m_bLoaded, not this void (type, array) slot. The client reads every DID on demand from the local DATs and Ac2DbAnimator's loader recurses by DID (dbanimator/tree.rs Loader::node), so no consumer exists

### AMHuman
- [-] `AMHuman::HasAnimator` · 006949a0 — tree-validation query (HasAnimator / CheckAnimationBoneNum fills an error-string list): its only callers are the same query on the sibling animators (calls table), no root caller in admin 7148 or Legions, so it is reached only from the authoring tools' checks; the client never validates animators
- [-] `AMHuman::CheckAnimationBoneNum` · 00694a10 — tree-validation query (HasAnimator / CheckAnimationBoneNum fills an error-string list): its only callers are the same query on the sibling animators (calls table), no root caller in admin 7148 or Legions, so it is reached only from the authoring tools' checks; the client never validates animators
- [x] `AMHuman::GetPeriod` · 00694a80 — ac2-anim dbanimator / ac2-physics anim_pose (slice 3)
- [-] `AMHuman::CopyMe` · 00694af0 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing
- [-] `AMHuman::Destroy` · 00694b30 · legions 00634b70 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing
- [x] `AMHuman::GetTriggers` · 00694b50 — ac2-anim dbanimator / ac2-physics anim_pose (slice 3)
- [x] `AMHuman::StreamPack` · 00694bd0 — ac2-anim dbanimator/tree.rs AnimatorRecord (AMHuman::StreamPack: the one dbAnimatorID); every human AnimMap animator loads in retail_human_animators_load_and_pose_finitely
- [x] `AMHuman::Animate` · 00694c00 — ac2-anim dbanimator / ac2-physics anim_pose (slice 3)
- [-] `AMHuman::DataToFileNode` · 00695020 — admin text source format (ToFileNode/FromFileNode/Parse*Node), admin tools
- [-] `AMHuman::GetRecursiveSubDataIDs` · 006950e0 — GetSubDataIDs lists a record's dependent DIDs for the data-graph prefetch (DBObj::AddToDataGraph); DBOCache::GetIfInMemory does not call it, it calls an int-returning vtable slot (+0x24) once when !m_bLoaded, not this void (type, array) slot. The client reads every DID on demand from the local DATs and Ac2DbAnimator's loader recurses by DID (dbanimator/tree.rs Loader::node), so no consumer exists
- [-] `AMHuman::DataFromFileNode` · 00695170 — admin text source format (ToFileNode/FromFileNode/Parse*Node), admin tools

## `portal/shared/src/animsys/animmodules/amlook.cpp.c` (1/3)

### AMLook
- [-] `AMLook::CopyMe` · 00690c60 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing
- [-] `AMLook::~AMLook` · 00690c90 · legions 00630ef0 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)
- [x] `AMLook::Animate` · 00690ca0 · legions 00630f00 — ac2-anim dbanimator/look.rs (AMLook): GetPeriod 1.0, GetTriggers adds nothing, Animate turns the neck and head bones by the 0x40000007-9 look sliders (x87 order, f32 stores); tests in dbanimator/interpolator_tests.rs. The record 0x0500002a decodes to the last byte (retail_animator_records_decode...)

## `portal/shared/src/animsys/animmodules/amrandom.cpp.c` (5/14)

### AMRandom
- [-] `AMRandom::GetSubDataIDs` · 00691090 — GetSubDataIDs lists a record's dependent DIDs for the data-graph prefetch (DBObj::AddToDataGraph); DBOCache::GetIfInMemory does not call it, it calls an int-returning vtable slot (+0x24) once when !m_bLoaded, not this void (type, array) slot. The client reads every DID on demand from the local DATs and Ac2DbAnimator's loader recurses by DID (dbanimator/tree.rs Loader::node), so no consumer exists
- [-] `AMRandom::GetRecursiveSubDataIDs` · 006910a0 — GetSubDataIDs lists a record's dependent DIDs for the data-graph prefetch (DBObj::AddToDataGraph); DBOCache::GetIfInMemory does not call it, it calls an int-returning vtable slot (+0x24) once when !m_bLoaded, not this void (type, array) slot. The client reads every DID on demand from the local DATs and Ac2DbAnimator's loader recurses by DID (dbanimator/tree.rs Loader::node), so no consumer exists
- [-] `AMRandom::HasAnimator` · 006910b0 — tree-validation query (HasAnimator / CheckAnimationBoneNum fills an error-string list): its only callers are the same query on the sibling animators (calls table), no root caller in admin 7148 or Legions, so it is reached only from the authoring tools' checks; the client never validates animators
- [-] `AMRandom::CheckAnimationBoneNum` · 006910c0 — tree-validation query (HasAnimator / CheckAnimationBoneNum fills an error-string list): its only callers are the same query on the sibling animators (calls table), no root caller in admin 7148 or Legions, so it is reached only from the authoring tools' checks; the client never validates animators
- [x] `AMRandom::GetPeriod` · 006910d0 — ac2-anim dbanimator/interpolator.rs Ac2Random (AMRandom::StreamPack, GetType 0x40000007, GetPeriod, GetTriggers and Animate delegate to the AnimInterpolator; Animate rolls Random::RollDice(float) and (long) through the context and starts the SliderInterpolator); the two DAT records load in retail_animator_records_decode_and_amrandom_loads_its_animators; tests amrandom_starts_a_ramp...
- [x] `AMRandom::GetType` · 006910e0 — ac2-anim dbanimator/interpolator.rs Ac2Random (AMRandom::StreamPack, GetType 0x40000007, GetPeriod, GetTriggers and Animate delegate to the AnimInterpolator; Animate rolls Random::RollDice(float) and (long) through the context and starts the SliderInterpolator); the two DAT records load in retail_animator_records_decode_and_amrandom_loads_its_animators; tests amrandom_starts_a_ramp...
- [-] `AMRandom::Destroy` · 006910f0 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing
- [x] `AMRandom::GetTriggers` · 00691100 — ac2-anim dbanimator/interpolator.rs Ac2Random (AMRandom::StreamPack, GetType 0x40000007, GetPeriod, GetTriggers and Animate delegate to the AnimInterpolator; Animate rolls Random::RollDice(float) and (long) through the context and starts the SliderInterpolator); the two DAT records load in retail_animator_records_decode_and_amrandom_loads_its_animators; tests amrandom_starts_a_ramp...
- [-] `AMRandom::~AMRandom` · 00691130 · legions 00631390 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)
- [x] `AMRandom::StreamPack` · 00691160 · legions 006313c0 — ac2-anim dbanimator/interpolator.rs Ac2Random (AMRandom::StreamPack, GetType 0x40000007, GetPeriod, GetTriggers and Animate delegate to the AnimInterpolator; Animate rolls Random::RollDice(float) and (long) through the context and starts the SliderInterpolator); the two DAT records load in retail_animator_records_decode_and_amrandom_loads_its_animators; tests amrandom_starts_a_ramp...
- [x] `AMRandom::Animate` · 006911c0 · legions 00631420 — ac2-anim dbanimator/interpolator.rs Ac2Random (AMRandom::StreamPack, GetType 0x40000007, GetPeriod, GetTriggers and Animate delegate to the AnimInterpolator; Animate rolls Random::RollDice(float) and (long) through the context and starts the SliderInterpolator); the two DAT records load in retail_animator_records_decode_and_amrandom_loads_its_animators; tests amrandom_starts_a_ramp...
- [-] `AMRandom::DataToFileNode` · 00691270 — admin text source format (ToFileNode/FromFileNode/Parse*Node), admin tools
- [-] `AMRandom::CopyMe` · 00691590 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing
- [-] `AMRandom::DataFromFileNode` · 00691610 — admin text source format (ToFileNode/FromFileNode/Parse*Node), admin tools

## `portal/shared/src/animsys/animmodules/amswitchme.cpp.c` (6/16)

### AMSwitchME
- [x] `AMSwitchME::StreamPack` · 006934c0 · legions 00633630 — ac2-anim dbanimator / ac2-physics anim_pose (slice 3)
- [-] `AMSwitchME::GetSubDataIDs` · 00693560 — GetSubDataIDs lists a record's dependent DIDs for the data-graph prefetch (DBObj::AddToDataGraph); DBOCache::GetIfInMemory does not call it, it calls an int-returning vtable slot (+0x24) once when !m_bLoaded, not this void (type, array) slot. The client reads every DID on demand from the local DATs and Ac2DbAnimator's loader recurses by DID (dbanimator/tree.rs Loader::node), so no consumer exists
- [-] `AMSwitchME::CheckAnimationBoneNum` · 006935d0 — tree-validation query (HasAnimator / CheckAnimationBoneNum fills an error-string list): its only callers are the same query on the sibling animators (calls table), no root caller in admin 7148 or Legions, so it is reached only from the authoring tools' checks; the client never validates animators
- [-] `AMSwitchME::DataToFileNode` · 00693700 — admin text source format (ToFileNode/FromFileNode/Parse*Node), admin tools
- [-] `AMSwitchME::GetRecursiveSubDataIDs` · 006939f0 — GetSubDataIDs lists a record's dependent DIDs for the data-graph prefetch (DBObj::AddToDataGraph); DBOCache::GetIfInMemory does not call it, it calls an int-returning vtable slot (+0x24) once when !m_bLoaded, not this void (type, array) slot. The client reads every DID on demand from the local DATs and Ac2DbAnimator's loader recurses by DID (dbanimator/tree.rs Loader::node), so no consumer exists
- [-] `AMSwitchME::Destroy` · 00693b50 · legions 00633c30 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing
- [-] `AMSwitchME::HasAnimator` · 00693c20 — tree-validation query (HasAnimator / CheckAnimationBoneNum fills an error-string list): its only callers are the same query on the sibling animators (calls table), no root caller in admin 7148 or Legions, so it is reached only from the authoring tools' checks; the client never validates animators
- [-] `AMSwitchME::~AMSwitchME` · 00693f70 · legions 00634020 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)
- [x] `AMSwitchME::GetAnimator` · 00694000 · legions 006340b0 — ac2-anim dbanimator / ac2-physics anim_pose (slice 3)
- [x] `AMSwitchME::GetPeriod` · 00694120 · legions 006341b0 — ac2-anim dbanimator / ac2-physics anim_pose (slice 3)
- [-] `AMSwitchME::DataFromFileNode` · 00694150 — admin text source format (ToFileNode/FromFileNode/Parse*Node), admin tools
- [x] `AMSwitchME::GetTriggers` · 00694770 · legions 00634800 — ac2-anim dbanimator / ac2-physics anim_pose (slice 3)
- [x] `AMSwitchME::GetHoldLoop` · 006947a0 · legions 00634830 — ac2-anim dbanimator / ac2-physics anim_pose (slice 3)
- [x] `AMSwitchME::Animate` · 00694880 · legions 00634910 — ac2-anim dbanimator / ac2-physics anim_pose (slice 3)
- [-] `AMSwitchME::AMSwitchME` · 006948c0 · legions 00634950 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)
- [-] `AMSwitchME::CopyMe` · 00694910 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing

## `portal/shared/src/animsys/animmodules/animasset.cpp.c` (12/53)

### AnimAssetOffsetKey
- [-] `AnimAssetOffsetKey::AnimAssetOffsetKey` · 00698830 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)

### AnimAssetQuatKey
- [-] `AnimAssetQuatKey::AnimAssetQuatKey` · 00698840 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)

### AnimAssetWeightKey
- [-] `AnimAssetWeightKey::AnimAssetWeightKey` · 00698850 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)

### AnimAsset
- [-] `AnimAsset::Destroy` · 00698860 · legions 006374d0 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing
- [x] `AnimAsset::GetPeriod` · 00698920 — ac2-anim dbanimator / ac2-physics anim_pose (slice 3)
- [x] `AnimAsset::GetTriggers` · 00698930 — ac2-anim dbanimator / ac2-physics anim_pose (slice 3)
- [x] `AnimAsset::GetHoldLoop` · 00698990 — ac2-anim dbanimator / ac2-physics anim_pose (slice 3)

### AnimAssetScaleKey
- [-] `AnimAssetScaleKey::AnimAssetScaleKey` · 00698cb0 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)

### AnimAsset
- [-] `AnimAsset::AnimAsset` · 00698cc0 · legions 00637930 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)

### KeyframeLookupTable<AnimAssetScaleKey>
- [x] `KeyframeLookupTable<AnimAssetScaleKey>::FindLargestLTEQ` · 00698ce0 · legions 00637950 — ac2-anim dbanimator / ac2-physics anim_pose (slice 3)

### KeyframeLookupTable<AnimAssetQuatKey>
- [x] `KeyframeLookupTable<AnimAssetQuatKey>::FindLargestLTEQ` · 00698d80 · legions 006379f0 — ac2-anim dbanimator/asset.rs KeyframeLookupTable::FindLargestLTEQ (one generic template for quat, weight and scale keys)

### KeyframeLookupTable<AnimAssetWeightKey>
- [x] `KeyframeLookupTable<AnimAssetWeightKey>::FindLargestLTEQ` · 00698e20 · legions 00637a90 — ac2-anim dbanimator/asset.rs KeyframeLookupTable::FindLargestLTEQ (one generic template for quat, weight and scale keys)

### SmartArray<AnimAssetOffsetKey>
- [-] `SmartArray<AnimAssetOffsetKey>::operator=` · 00698ec0 · legions 00637b30 — container operator= (Rust collections)

### PSmartArray<AnimAssetOffsetKey>
- [-] `PSmartArray<AnimAssetOffsetKey>::StreamPack` · 00698f30 · legions 00637ba0 — container serialization/helper (Rust collections; elements decoded by their owning class)

### SmartArray<AnimAssetQuatKey>
- [-] `SmartArray<AnimAssetQuatKey>::operator=` · 00698fd0 · legions 00637c40 — container operator= (Rust collections)

### PSmartArray<AnimAssetQuatKey>
- [-] `PSmartArray<AnimAssetQuatKey>::StreamPack` · 00699040 · legions 00637cb0 — container serialization/helper (Rust collections; elements decoded by their owning class)

### SmartArray<AnimAssetWeightKey>
- [-] `SmartArray<AnimAssetWeightKey>::operator=` · 006990e0 · legions 00637d50 — container operator= (Rust collections)

### PSmartArray<AnimAssetWeightKey>
- [-] `PSmartArray<AnimAssetWeightKey>::StreamPack` · 00699150 · legions 00637dc0 — container serialization/helper (Rust collections; elements decoded by their owning class)

### AnimInfo
- [-] `AnimInfo::AnimInfo` · 006992d0 · legions 00637f40 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)
- [-] `AnimInfo::~AnimInfo` · 00699340 · legions 00637fb0 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)

### AnimAsset
- [x] `AnimAsset::Animate` · 00699400 — ac2-anim dbanimator / ac2-physics anim_pose (slice 3)
- [-] `AnimAsset::SetKeyFrame` · 00699990 · legions 00638600 — trivial setter / list append on plain struct during decode
- [-] `AnimAsset::SetScaleKeyFrame` · 00699a90 · legions 00638700 — trivial setter / list append on plain struct during decode

### KeyframeLookupTable<AnimAssetScaleKey>
- [-] `KeyframeLookupTable<AnimAssetScaleKey>::KeyframeLookupTable<AnimAssetScaleKey>` · 00699ae0 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)
- [-] `KeyframeLookupTable<AnimAssetScaleKey>::~KeyframeLookupTable<AnimAssetScaleKey>` · 00699b10 · legions 00638780 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)

### KeyframeLookupTable<AnimAssetOffsetKey>
- [-] `KeyframeLookupTable<AnimAssetOffsetKey>::KeyframeLookupTable<AnimAssetOffsetKey>` · 00699b70 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)
- [-] `KeyframeLookupTable<AnimAssetOffsetKey>::~KeyframeLookupTable<AnimAssetOffsetKey>` · 00699ba0 · legions 00638810 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)

### KeyframeLookupTable<AnimAssetQuatKey>
- [-] `KeyframeLookupTable<AnimAssetQuatKey>::KeyframeLookupTable<AnimAssetQuatKey>` · 00699c00 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)
- [-] `KeyframeLookupTable<AnimAssetQuatKey>::~KeyframeLookupTable<AnimAssetQuatKey>` · 00699c30 · legions 006388a0 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)

### KeyframeLookupTable<AnimAssetWeightKey>
- [-] `KeyframeLookupTable<AnimAssetWeightKey>::KeyframeLookupTable<AnimAssetWeightKey>` · 00699c90 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)
- [-] `KeyframeLookupTable<AnimAssetWeightKey>::~KeyframeLookupTable<AnimAssetWeightKey>` · 00699cc0 · legions 00638930 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)

### SmartArray<AnimAssetScaleKey>
- [-] `SmartArray<AnimAssetScaleKey>::operator=` · 00699d20 · legions 00638990 — container operator= (Rust collections)

### PSmartArray<AnimAssetScaleKey>
- [-] `PSmartArray<AnimAssetScaleKey>::StreamPack` · 00699d90 · legions 00638a00 — container serialization/helper (Rust collections; elements decoded by their owning class)

### AnimAsset
- [-] `AnimAsset::SetNumBones` · 00699e30 · legions 00638aa0 — trivial setter / list append on plain struct during decode
- [-] `AnimAsset::LoadFromFile` · 0069a270 — text source loader, admin build only
- [-] `AnimAsset::SaveToFile` · 0069a4e0 — admin text source format (ToFileNode/FromFileNode/Parse*Node), admin tools

### KeyframeLookupTable<AnimAssetScaleKey>
- [-] `KeyframeLookupTable<AnimAssetScaleKey>::InitLookupTable` · 0069a630 · legions 00639030 — keyframe lookup-table build (acceleration); FindLargestLTEQ cited searches the keys directly

### KeyframeLookupTable<AnimAssetQuatKey>
- [-] `KeyframeLookupTable<AnimAssetQuatKey>::InitLookupTable` · 0069a7f0 · legions 006391f0 — keyframe lookup-table build (acceleration); FindLargestLTEQ cited searches the keys directly

### KeyframeLookupTable<AnimAssetWeightKey>
- [-] `KeyframeLookupTable<AnimAssetWeightKey>::InitLookupTable` · 0069a9b0 · legions 006393b0 — keyframe lookup-table build (acceleration); FindLargestLTEQ cited searches the keys directly

### AnimAsset
- [x] `AnimAsset::StreamPackInternal` · 0069ab70 · legions 00639570 — ac2-anim dbanimator / ac2-physics anim_pose (slice 3)
- [-] `AnimAsset::SetNumScaleKeys` · 0069aca0 · legions 006396a0 — trivial setter / list append on plain struct during decode
- [-] `AnimAsset::SetNumKeyFrames` · 0069ace0 · legions 006396e0 — trivial setter / list append on plain struct during decode
- [-] `AnimAsset::ReadBinaryAnim` · 0069ad90 · legions 00639790 — AnimAsset::ReadBinaryAnim: only caller is AnimAsset::FromFileNode (tool import of binary animations); the client reads the DAT record (AnimAsset::StreamPackInternal)
- [-] `AnimAsset::WriteBinaryAnim` · 0069af10 — write-side/admin path with no Legions match (setups and anims are read-only in the DVD client) [vtable-checked: not a slot of any located Legions vtable]

### KeyframeLookupTable<AnimAssetScaleKey>
- [x] `KeyframeLookupTable<AnimAssetScaleKey>::StreamPack` · 0069afe0 — ac2-anim dbanimator/asset.rs Ac2AnimAsset::decode (AnimAsset::StreamPack calls StreamPackInternal; the KeyframeLookupTable StreamPacks are the per-bone scale, quaternion and weight key tables: a count then (value, time) keys); every human animator decodes in retail_human_animators_load_and_pose_finitely

### KeyframeLookupTable<AnimAssetQuatKey>
- [x] `KeyframeLookupTable<AnimAssetQuatKey>::StreamPack` · 0069b020 — ac2-anim dbanimator/asset.rs Ac2AnimAsset::decode (AnimAsset::StreamPack calls StreamPackInternal; the KeyframeLookupTable StreamPacks are the per-bone scale, quaternion and weight key tables: a count then (value, time) keys); every human animator decodes in retail_human_animators_load_and_pose_finitely

### KeyframeLookupTable<AnimAssetWeightKey>
- [x] `KeyframeLookupTable<AnimAssetWeightKey>::StreamPack` · 0069b060 — ac2-anim dbanimator/asset.rs Ac2AnimAsset::decode (AnimAsset::StreamPack calls StreamPackInternal; the KeyframeLookupTable StreamPacks are the per-bone scale, quaternion and weight key tables: a count then (value, time) keys); every human animator decodes in retail_human_animators_load_and_pose_finitely

### AnimInfo
- [-] `AnimInfo::AnimInfo` · 0069b0a0 · legions 006399d0 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)

### AnimAsset
- [-] `AnimAsset::CheckAnimationBoneNum` · 0069b140 · legions 00639a70 — tree-validation query (HasAnimator / CheckAnimationBoneNum fills an error-string list): its only callers are the same query on the sibling animators (calls table), no root caller in admin 7148 or Legions, so it is reached only from the authoring tools' checks; the client never validates animators
- [x] `AnimAsset::StreamPack` · 0069b1f0 — ac2-anim dbanimator/asset.rs Ac2AnimAsset::decode (AnimAsset::StreamPack calls StreamPackInternal; the KeyframeLookupTable StreamPacks are the per-bone scale, quaternion and weight key tables: a count then (value, time) keys); every human animator decodes in retail_human_animators_load_and_pose_finitely
- [-] `AnimAsset::FromFileNode` · 0069b210 · legions 00639b40 — text source format (admin tools)
- [-] `AnimAsset::ToFileNode` · 0069ba90 — text source format (admin tools)
- [-] `AnimAsset::CopyMe` · 0069bbf0 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing

## `portal/shared/src/animsys/animmodules/animcube.cpp.c` (6/22)

### AnimCube
- [-] `AnimCube::FlushCache` · 006965a0 · legions 00636140 — retail DB cache prefetch/flush (AGENTS exception: read on demand, free on last drop)
- [-] `AnimCube::CheckAnimationBoneNum` · 006965e0 — tree-validation query (HasAnimator / CheckAnimationBoneNum fills an error-string list): its only callers are the same query on the sibling animators (calls table), no root caller in admin 7148 or Legions, so it is reached only from the authoring tools' checks; the client never validates animators

### AnimCube::SliderAndTickList
- [x] `AnimCube::SliderAndTickList::StreamPack` · 00696670 · legions 00636200 — ac2-anim dbanimator/tree.rs AnimatorRecord::Cube (AnimCube::SliderAndTickList::StreamPack: the slider id then its tick list); animators of the human AnimMap load in retail_human_animators_load_and_pose_finitely

### AnimCube
- [-] `AnimCube::GetFromLocalCache` · 006966c0 — retail DB cache prefetch/flush (AGENTS exception: read on demand, free on last drop)
- [-] `AnimCube::AnimCube` · 00696770 · legions 00636290 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)
- [-] `AnimCube::Destroy` · 006967b0 · legions 006362d0 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing
- [x] `AnimCube::GetAnimatorWeights` · 006967e0 · legions 00636300 — ac2-anim dbanimator / ac2-physics anim_pose (slice 3)
- [-] `AnimCube::~AnimCube` · 00696a00 · legions 00636520 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)
- [-] `AnimCube::GetRecursiveSubDataIDs` · 00696a90 — GetSubDataIDs lists a record's dependent DIDs for the data-graph prefetch (DBObj::AddToDataGraph); DBOCache::GetIfInMemory does not call it, it calls an int-returning vtable slot (+0x24) once when !m_bLoaded, not this void (type, array) slot. The client reads every DID on demand from the local DATs and Ac2DbAnimator's loader recurses by DID (dbanimator/tree.rs Loader::node), so no consumer exists
- [-] `AnimCube::HasAnimator` · 00696b70 — tree-validation query (HasAnimator / CheckAnimationBoneNum fills an error-string list): its only callers are the same query on the sibling animators (calls table), no root caller in admin 7148 or Legions, so it is reached only from the authoring tools' checks; the client never validates animators
- [x] `AnimCube::StreamPack` · 00696ce0 · legions 006367e0 — ac2-anim animator.rs / dbanimator/tree.rs AnimatorRecord::Cube: AnimCube::StreamPack sliders then children
- [x] `AnimCube::Animate` · 00696d80 — ac2-anim dbanimator / ac2-physics anim_pose (slice 3)
- [x] `AnimCube::GetPeriod` · 00696e10 — ac2-anim dbanimator / ac2-physics anim_pose (slice 3)
- [-] `AnimCube::GetAnimatorWeights` · 00696ea0 — the AnimCube::GetAnimatorWeights(x, y) overload has no Legions body and its only caller is AnimCube::ToFileNode (tool); the client uses the AnimationObj overload (cube_weights, done at 006967e0)
- [-] `AnimCube::ToFileNode` · 00697180 — text source format (admin tools)

### AnimCube::SliderAndTickList
- [-] `AnimCube::SliderAndTickList::SliderAndTickList` · 00697860 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)

### AnimCube
- [x] `AnimCube::GetTriggers` · 00697a20 — ac2-anim behavior/triggers + dbanimator/triggers (slice 4): effect, hook and impulse triggers through TriggerUpdate

### SmartArray<AnimCube::SliderAndTickList>
- [-] `SmartArray<AnimCube::SliderAndTickList>::operator=` · 00697ff0 · legions 006372c0 — container operator= (Rust collections)

### PSmartArray<AnimCube::SliderAndTickList>
- [-] `PSmartArray<AnimCube::SliderAndTickList>::StreamPack` · 00698060 · legions 00637330 — container serialization/helper (Rust collections; elements decoded by their owning class)

### SmartArray<AnimCube::SliderAndTickList>
- [-] `SmartArray<AnimCube::SliderAndTickList>::AddToEnd` · 00698100 — container serialization/helper (Rust collections; elements decoded by their owning class)

### AnimCube
- [-] `AnimCube::CopyMe` · 00698160 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing
- [-] `AnimCube::FromFileNode` · 00698230 — text source format (admin tools)

## `portal/shared/src/animsys/animmodules/animinterpolator.cpp.c` (5/16)

### AMLook
- [-] `AMLook::DataFromFileNode` · 006919c0 — admin text source format (ToFileNode/FromFileNode/Parse*Node), admin tools

### DBAnimator
- [-] `DBAnimator::GetDBOType` · 006919d0 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing

### AnimInterpolator
- [-] `AnimInterpolator::GetSubDataIDs` · 006919e0 · legions 00631c10 — GetSubDataIDs lists a record's dependent DIDs for the data-graph prefetch (DBObj::AddToDataGraph); DBOCache::GetIfInMemory does not call it, it calls an int-returning vtable slot (+0x24) once when !m_bLoaded, not this void (type, array) slot. The client reads every DID on demand from the local DATs and Ac2DbAnimator's loader recurses by DID (dbanimator/tree.rs Loader::node), so no consumer exists
- [-] `AnimInterpolator::CheckAnimationBoneNum` · 00691a10 · legions 00631c40 — tree-validation query (HasAnimator / CheckAnimationBoneNum fills an error-string list): its only callers are the same query on the sibling animators (calls table), no root caller in admin 7148 or Legions, so it is reached only from the authoring tools' checks; the client never validates animators
- [x] `AnimInterpolator::StreamPack` · 00691a90 · legions 00631cc0 — ac2-anim dbanimator/interpolator.rs Ac2Interpolator (AnimInterpolator::StreamPack slider and ids, GetAnimatorWeights from the slider interpolator, Animate/GetPeriod crossfade (1-t) and t, GetTriggers merges the effects by weight like AnimCube); tests in dbanimator/interpolator_tests.rs
- [x] `AnimInterpolator::GetAnimatorWeights` · 00691ad0 · legions 00631d00 — ac2-anim dbanimator/interpolator.rs Ac2Interpolator (AnimInterpolator::StreamPack slider and ids, GetAnimatorWeights from the slider interpolator, Animate/GetPeriod crossfade (1-t) and t, GetTriggers merges the effects by weight like AnimCube); tests in dbanimator/interpolator_tests.rs
- [x] `AnimInterpolator::Animate` · 00691b70 — ac2-anim dbanimator/interpolator.rs Ac2Interpolator (AnimInterpolator::StreamPack slider and ids, GetAnimatorWeights from the slider interpolator, Animate/GetPeriod crossfade (1-t) and t, GetTriggers merges the effects by weight like AnimCube); tests in dbanimator/interpolator_tests.rs
- [x] `AnimInterpolator::GetPeriod` · 00691ca0 — ac2-anim dbanimator/interpolator.rs Ac2Interpolator (AnimInterpolator::StreamPack slider and ids, GetAnimatorWeights from the slider interpolator, Animate/GetPeriod crossfade (1-t) and t, GetTriggers merges the effects by weight like AnimCube); tests in dbanimator/interpolator_tests.rs
- [-] `AnimInterpolator::Destroy` · 00691dc0 · legions 00631f90 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing
- [-] `AnimInterpolator::AnimInterpolator` · 00691df0 · legions 00631fc0 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)
- [-] `AnimInterpolator::~AnimInterpolator` · 00691e10 · legions 00631fe0 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)
- [-] `AnimInterpolator::GetRecursiveSubDataIDs` · 00691ec0 — GetSubDataIDs lists a record's dependent DIDs for the data-graph prefetch (DBObj::AddToDataGraph); DBOCache::GetIfInMemory does not call it, it calls an int-returning vtable slot (+0x24) once when !m_bLoaded, not this void (type, array) slot. The client reads every DID on demand from the local DATs and Ac2DbAnimator's loader recurses by DID (dbanimator/tree.rs Loader::node), so no consumer exists
- [-] `AnimInterpolator::HasAnimator` · 00691f90 — tree-validation query (HasAnimator / CheckAnimationBoneNum fills an error-string list): its only callers are the same query on the sibling animators (calls table), no root caller in admin 7148 or Legions, so it is reached only from the authoring tools' checks; the client never validates animators
- [-] `AnimInterpolator::CopyMe` · 00692100 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing

### PList<Trigger>
- [-] `PList<Trigger>::UnPack` · 006921b0 · legions 00632334 — container serialization/helper (Rust collections; elements decoded by their owning class)

### AnimInterpolator
- [x] `AnimInterpolator::GetTriggers` · 00692360 — ac2-anim dbanimator/interpolator.rs Ac2Interpolator (AnimInterpolator::StreamPack slider and ids, GetAnimatorWeights from the slider interpolator, Animate/GetPeriod crossfade (1-t) and t, GetTriggers merges the effects by weight like AnimCube); tests in dbanimator/interpolator_tests.rs

## `portal/shared/src/animsys/animmodules/axiscube.cpp.c` (0/15)

### AnimMap
- [-] `AnimMap::GetDBOType` · 00695320 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing

### AnimCube
- [-] `AnimCube::GetSubDataIDs` · 00695330 · legions 00635320 — GetSubDataIDs lists a record's dependent DIDs for the data-graph prefetch (DBObj::AddToDataGraph); DBOCache::GetIfInMemory does not call it, it calls an int-returning vtable slot (+0x24) once when !m_bLoaded, not this void (type, array) slot. The client reads every DID on demand from the local DATs and Ac2DbAnimator's loader recurses by DID (dbanimator/tree.rs Loader::node), so no consumer exists

### AxisCube
- [-] `AxisCube::HasAnimator` · 00695360 · legions 00635350 — tree-validation query (HasAnimator / CheckAnimationBoneNum fills an error-string list): its only callers are the same query on the sibling animators (calls table), no root caller in admin 7148 or Legions, so it is reached only from the authoring tools' checks; the client never validates animators
- [-] `AxisCube::CheckAnimationBoneNum` · 006953e0 — tree-validation query (HasAnimator / CheckAnimationBoneNum fills an error-string list): its only callers are the same query on the sibling animators (calls table), no root caller in admin 7148 or Legions, so it is reached only from the authoring tools' checks; the client never validates animators
- [-] `AxisCube::StreamPack` · 00695470 · legions 00635460 — AxisCube (type 0x40000006): no DBAnimator record of the Legions portal DAT uses it (all 5,480 records at 0x0500xxxx are types 1, 2, 3, 5, 7, 8 and 10; retail_animator_records_decode... asserts none is 0x40000006), so nothing the client loads reaches it
- [-] `AxisCube::GetAnimatorWeights` · 006954c0 · legions 006354b0 — AxisCube (type 0x40000006): no DBAnimator record of the Legions portal DAT uses it (all 5,480 records at 0x0500xxxx are types 1, 2, 3, 5, 7, 8 and 10; retail_animator_records_decode... asserts none is 0x40000006), so nothing the client loads reaches it
- [-] `AxisCube::Destroy` · 006955c0 · legions 006355b0 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing
- [-] `AxisCube::AxisCube` · 006955f0 · legions 006355e0 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)
- [-] `AxisCube::~AxisCube` · 00695620 · legions 00635610 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)
- [-] `AxisCube::GetRecursiveSubDataIDs` · 006956a0 — GetSubDataIDs lists a record's dependent DIDs for the data-graph prefetch (DBObj::AddToDataGraph); DBOCache::GetIfInMemory does not call it, it calls an int-returning vtable slot (+0x24) once when !m_bLoaded, not this void (type, array) slot. The client reads every DID on demand from the local DATs and Ac2DbAnimator's loader recurses by DID (dbanimator/tree.rs Loader::node), so no consumer exists
- [-] `AxisCube::Animate` · 00695770 — AxisCube (type 0x40000006): no DBAnimator record of the Legions portal DAT uses it (all 5,480 records at 0x0500xxxx are types 1, 2, 3, 5, 7, 8 and 10; retail_animator_records_decode... asserts none is 0x40000006), so nothing the client loads reaches it
- [-] `AxisCube::GetPeriod` · 00695920 — AxisCube (type 0x40000006): no DBAnimator record of the Legions portal DAT uses it (all 5,480 records at 0x0500xxxx are types 1, 2, 3, 5, 7, 8 and 10; retail_animator_records_decode... asserts none is 0x40000006), so nothing the client loads reaches it
- [-] `AxisCube::CopyMe` · 00695a80 · legions 006359e0 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing
- [-] `AxisCube::FromFileNode` · 00695af0 — text source format (admin tools)
- [-] `AxisCube::GetTriggers` · 00695e90 — AxisCube (type 0x40000006): no DBAnimator record of the Legions portal DAT uses it (all 5,480 records at 0x0500xxxx are types 1, 2, 3, 5, 7, 8 and 10; retail_animator_records_decode... asserts none is 0x40000006), so nothing the client loads reaches it

## `portal/shared/src/animsys/behavior.cpp.c` (1/2)

### Behavior
- [-] `Behavior::Behavior` · 0068f360 · legions 0062fee0 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)
- [x] `Behavior::StreamPack` · 0068f380 — ac2-anim behavior_table.rs decode_template / transition_key (Behavior::StreamPack 64-bit flags, Mode::StreamPack isCombat, Slider::StreamPack controller, ModeTransitionKey::StreamPack); test the_char_template_keeps_modes_and_user_controlled_sliders and retail_behavior_tables_decode

## `portal/shared/src/animsys/behaviorparams.cpp.c` (4/6)

### BehaviorParams
- [-] `BehaviorParams::IsValid` · 00678fe0 · legions 006233b0 — BehaviorParams::IsValid (mBehaviorID != INVALID): callers are PhysicsStory::get_pack_bitfield/IsNull and StoryManager::DoStory/DoStoryHook (the story manager, a physics-catalog row); ac2-anim reads an unpacked behaviour as id 0 and DoBehavior refuses it as unknown
- [x] `BehaviorParams::get_pack_bitfield` · 00679000 · legions 006233d0 — ac2-script host (ScriptHost: process creation, dispatch, SendServerEvt, notices)
- [x] `BehaviorParams::StreamPack` · 00679220 · legions 006235f0 — ac2-anim; ac2-script native::BehaviorParams also unpacks it
- [x] `BehaviorParams::SetDefaults` · 006795b0 · legions 00623980 — ac2-net protocol/physics.rs Default for BehaviorParams: BehaviorParams::SetDefaults values
- [-] `BehaviorParams::BehaviorParams` · 00679660 · legions 00623a30 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)
- [x] `BehaviorParams::BehaviorParams` · 006796a0 · legions 00623a70 — ac2-anim behavior/track.rs Ac2BehaviorParams::new: BehaviorParams::BehaviorParams(ulong) over SetDefaults

## `portal/shared/src/animsys/behaviortable.cpp.c` (10/49)

### BehaviorTable
- [x] `BehaviorTable::GetInitBehaviors` · 0067cfa0 · legions 006253d0 — ac2-anim behavior_table.rs (Legions identical; Legions GetBehavior inlines GetCharTemplate)

### SliderData
- [x] `SliderData::StreamPack` · 0067cfb0 · legions 005f5840

### BehaviorTable
- [x] `BehaviorTable::StreamPack` · 0067d000 · legions 006253e0 — ac2-anim behavior_table.rs (Legions identical; Legions GetBehavior inlines GetCharTemplate)
- [x] `BehaviorTable::GetLayerNum` · 0067d140 · legions 00625520 — ac2-anim behavior_table.rs (Legions identical; Legions GetBehavior inlines GetCharTemplate)

### HashIterator<ModeTransitionKey,float>
- [-] `HashIterator<ModeTransitionKey,float>::operator++` · 0067d320 · legions 00625700 — hash/list iterator internals (Rust iterators)

### IntrusiveHashIterator<ModeTransitionKey,HashTableData<ModeTransitionKey,float>_*>
- [-] `IntrusiveHashIterator<ModeTransitionKey,HashTableData<ModeTransitionKey,float>_*>::operator++` · 0067d360 — hash/list iterator internals (Rust iterators)

### HashIterator<ReferenceIDWrapper,unsigned_long>
- [-] `HashIterator<ReferenceIDWrapper,unsigned_long>::operator++` · 0067d3b0 · legions 00625740 — hash/list iterator internals (Rust iterators)

### IntrusiveHashIterator<ComboKey,HashTableData<ComboKey,ComboInfo>_*>
- [-] `IntrusiveHashIterator<ComboKey,HashTableData<ComboKey,ComboInfo>_*>::operator++` · 0067d3f0 — hash/list iterator internals (Rust iterators)

### HashIterator<ModeStateKey,int>
- [-] `HashIterator<ModeStateKey,int>::operator++` · 0067d440 · legions 00625780 — hash/list iterator internals (Rust iterators)

### BehaviorTable
- [-] `BehaviorTable::ParseLayersNode` · 0067d660 — admin text source format (ToFileNode/FromFileNode/Parse*Node), admin tools
- [-] `BehaviorTable::ParseInitBehaviorsNode` · 0067d820 — admin text source format (ToFileNode/FromFileNode/Parse*Node), admin tools
- [-] `BehaviorTable::ParseInitModeNode` · 0067d980 — admin text source format (ToFileNode/FromFileNode/Parse*Node), admin tools

### HashIterator<ModeTransitionKey,float>
- [-] `HashIterator<ModeTransitionKey,float>::operator++` · 0067da40 — hash/list iterator internals (Rust iterators)

### HashIterator<ComboKey,ComboInfo>
- [-] `HashIterator<ComboKey,ComboInfo>::operator++` · 0067da70 — hash/list iterator internals (Rust iterators)

### ParseHelper<EmpGenParse>
- [-] `ParseHelper<EmpGenParse>::~ParseHelper<EmpGenParse>` · 0067dc80 · legions 006223a0 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)

### SmartArray<PStringBase<char>_>
- [-] `SmartArray<PStringBase<char>_>::SetNElements` · 0067dda0 — container serialization/helper (Rust collections; elements decoded by their owning class)

### BehaviorTable
- [x] `BehaviorTable::GetModeTransitionRampTime` · 0067ddd0 · legions 00625b10 — ac2-anim behavior_table.rs (Legions identical; Legions GetBehavior inlines GetCharTemplate)
- [x] `BehaviorTable::GetComboInfo` · 0067de10 · legions 00625b50 — ac2-anim behavior_table.rs (Legions identical; Legions GetBehavior inlines GetCharTemplate)
- [x] `BehaviorTable::GetBehaviorRampTime` · 0067de90 · legions 00625bd0 — ac2-anim behavior_table.rs (Legions identical; Legions GetBehavior inlines GetCharTemplate)
- [x] `BehaviorTable::GetBehaviorInfo` · 0067def0 · legions 00625c30 — ac2-anim behavior_table.rs (Legions identical; Legions GetBehavior inlines GetCharTemplate)
- [x] `BehaviorTable::GetTransitionInfo` · 0067df60 · legions 00625ca0 — ac2-anim behavior_table.rs (Legions identical; Legions GetBehavior inlines GetCharTemplate)

### HashTable<ModeTransitionKey,float>
- [-] `HashTable<ModeTransitionKey,float>::operator=` · 0067df90 · legions 00625cd0 — container operator= (Rust collections)

### HashTable<ComboKey,ComboInfo>
- [-] `HashTable<ComboKey,ComboInfo>::operator=` · 0067e130 · legions 00625df0 — container operator= (Rust collections)

### HashTable<unsigned_long,BehaviorInfo>
- [-] `HashTable<unsigned_long,BehaviorInfo>::operator=` · 0067e390 · legions 00625fb0 — container operator= (Rust collections)

### BehaviorTable
- [-] `BehaviorTable::ToFileNode` · 0067e9b0 — text source format (admin tools)
- [-] `BehaviorTable::ParseBehaviorInfoNode` · 00680180 — admin text source format (ToFileNode/FromFileNode/Parse*Node), admin tools

### PList<ModeStateKey>
- [-] `PList<ModeStateKey>::PList<ModeStateKey>` · 00680690 · legions 006266f0 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)

### PList<OpacityAtom>
- [-] `PList<OpacityAtom>::GetPackSize` · 006806d0 — container serialization/helper (Rust collections; elements decoded by their owning class)

### PList<ModeStateKey>
- [-] `PList<ModeStateKey>::UnPack` · 00680710 — container serialization/helper (Rust collections; elements decoded by their owning class)

### BehaviorTable
- [-] `BehaviorTable::ParseComboNode` · 00680830 — admin text source format (ToFileNode/FromFileNode/Parse*Node), admin tools
- [-] `BehaviorTable::ParseBehaviorsDefinition` · 00681460 — admin text source format (ToFileNode/FromFileNode/Parse*Node), admin tools

### PHashTable<ModeTransitionKey,float>
- [-] `PHashTable<ModeTransitionKey,float>::StreamPack` · 00681eb0 — container serialization/helper (Rust collections; elements decoded by their owning class)

### PHashTable<ComboKey,ComboInfo>
- [-] `PHashTable<ComboKey,ComboInfo>::StreamPack` · 006821d0 — container serialization/helper (Rust collections; elements decoded by their owning class)

### PHashTable<ModeTransitionKey,TransitionInfo>
- [-] `PHashTable<ModeTransitionKey,TransitionInfo>::StreamPack` · 00682570 — container serialization/helper (Rust collections; elements decoded by their owning class)

### IntrusiveHashIterator<ModeTransitionKey,HashTableData<ModeTransitionKey,TransitionInfo>_*>
- [-] `IntrusiveHashIterator<ModeTransitionKey,HashTableData<ModeTransitionKey,TransitionInfo>_*>::operator++` · 00682750 · legions 00627040 — hash/list iterator internals (Rust iterators)

### PHashTable<unsigned_long,BehaviorInfo>
- [-] `PHashTable<unsigned_long,BehaviorInfo>::StreamPack` · 00682850 — container serialization/helper (Rust collections; elements decoded by their owning class)

### PList<ModeStateKey>
- [-] `PList<ModeStateKey>::operator=` · 00682a50 · legions 006273d0 — container operator= (Rust collections)

### BehaviorTable
- [-] `BehaviorTable::Destroy` · 00682eb0 · legions 006277e0 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing

### HashTable<ModeStateKey,PList<ModeStateKey>_>
- [-] `HashTable<ModeStateKey,PList<ModeStateKey>_>::operator=` · 00682f00 · legions 00627830 — container operator= (Rust collections)

### BehaviorTable
- [-] `BehaviorTable::ParseModeTransitionNode` · 00683140 — admin text source format (ToFileNode/FromFileNode/Parse*Node), admin tools

### PHashTable<ModeStateKey,PList<ModeStateKey>_>
- [-] `PHashTable<ModeStateKey,PList<ModeStateKey>_>::StreamPack` · 006844d0 · legions 00627a00 — container serialization/helper (Rust collections; elements decoded by their owning class)
- [-] `PHashTable<ModeStateKey,PList<ModeStateKey>_>::~PHashTable<ModeStateKey,PList<ModeStateKey>_>` · 006846d0 · legions 00627c70 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)

### BehaviorTable
- [-] `BehaviorTable::BehaviorTable` · 00684780 · legions 00627d20 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)
- [-] `BehaviorTable::GetDBOType` · 00684910 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing
- [-] `BehaviorTable::~BehaviorTable` · 00684920 · legions 00627ec0 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)
- [x] `BehaviorTable::GetNextTransitionAlongModePath` · 00684b00 · legions 006280a0 — ac2-anim behavior_table.rs (Legions identical; Legions GetBehavior inlines GetCharTemplate)
- [-] `BehaviorTable::CopyInto` · 006851d0 · legions 00628770 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing
- [-] `BehaviorTable::FromFileNode` · 00685260 — text source format (admin tools)
- [-] `BehaviorTable::Allocate` · 00685560 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing

## `portal/shared/src/animsys/behaviortrack.cpp.c` (1/4)

### BehaviorTrack
- [x] `BehaviorTrack::Update` · 0068f520 · legions 006300a0 — ac2-anim behavior/track.rs Ac2BehaviorTrack::update (clip time wrap at 64, weight ramp, action ramp in and out, hold loop; a plain behaviour clears its effect triggers each cycle); behavior/tests.rs ramp tests
- [-] `BehaviorTrack::~BehaviorTrack` · 0068f980 · legions 00630500 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)

### (free functions)
- [-] `STREAMPACK_STL_SET` · 0068fa20 · legions 006305a0 — STL set/tree serialization (Rust collections)

### BehaviorTrack
- [-] `BehaviorTrack::StreamPack` · 0068fb60 · legions 006306e0 — StreamPack of runtime tracker state (BehaviorTrack, FXTrack, BehaviorTracker, ModeTracker, SliderTracker, TriggerUpdate): nothing calls it: no direct caller in the calls table and no other function in the index refers to these classes' StreamPack; the live state is Ac2ModeTracker/Ac2BehaviorTracker/Ac2SliderTracker/Ac2TriggerUpdate, rebuilt from the server's network behaviour params

## `portal/shared/src/animsys/behaviortracker.cpp.c` (20/34)

### BehaviorTracker
- [x] `BehaviorTracker::SendAndClearCallback` · 0068bba0 · legions 0062cbd0 — ac2-anim Ac2BehaviorTracker
- [x] `BehaviorTracker::ProcessParamFlags` · 0068bbf0 · legions 0062cc20 — ac2-anim behavior/tracker.rs process_param_flags: children fade out (ChildrenOpacity), FX starts, and for the player the action lock and camera parent/target/restore/targeted camera as Ac2AnimEvents (the client turns them into CameraManager calls); test a_behaviour_fades_the_children_and_drives_the_players_camera
- [~] `BehaviorTracker::ProcessRampingDownParamFlags` · 0068bd50 · legions 0062cd80 — ac2-anim behavior/tracker.rs process_ramping_down_param_flags: children fade back in, the player's camera is restored, the FX stops, weenie behaviours count down. Not modelled: mDestroyOnCompletion (an AnimationObj slot-1 call with 0,0,0,3.0) and the clone end (slot 2, when mTargetID or mVDescToClone is set)
- [-] `BehaviorTracker::StreamPack` · 0068be30 · legions 0062ce60 — StreamPack of runtime tracker state (BehaviorTrack, FXTrack, BehaviorTracker, ModeTracker, SliderTracker, TriggerUpdate): nothing calls it: no direct caller in the calls table and no other function in the index refers to these classes' StreamPack; the live state is Ac2ModeTracker/Ac2BehaviorTracker/Ac2SliderTracker/Ac2TriggerUpdate, rebuilt from the server's network behaviour params

### SmartArray<WeenieHook>
- [-] `SmartArray<WeenieHook>::operator=` · 0068c0c0 · legions 0062d0f0 — container operator= (Rust collections)

### PSmartArray<WeenieHook>
- [-] `PSmartArray<WeenieHook>::StreamPack` · 0068c140 · legions 0062d170 — container serialization/helper (Rust collections; elements decoded by their owning class)

### BehaviorTracker
- [x] `BehaviorTracker::Destroy` · 0068c1e0 · legions 0062d210 — ac2-anim behavior/tracker.rs destroy: weight -1, AddTrackTriggers, and the children fade back in (ChildrenOpacity 1.0 over 0.25 s) for a track that faded them
- [x] `BehaviorTracker::AddTriggers` · 0068c290 · legions 0062d2c0 — ac2-anim behavior/triggers + dbanimator/triggers (slice 4): effect, hook and impulse triggers through TriggerUpdate
- [x] `BehaviorTracker::CheckActiveBehaviorsForDynamicFPCamera` · 0068c300 · legions 0062d330 — ac2-anim behavior/mode_tracker.rs check_for_dynamic_fp_camera and tracker.rs check_active_behaviors_for_dynamic_fp_camera (incoming tracker else outgoing; any track flagged DYNAMIC_FP_CAMERA); test a_dynamic_first_person_camera_behaviour_is_found_while_it_plays. The consumer, CameraManager::UpdateCamera, is the camera's (ac2-smartbox)
- [x] `BehaviorTracker::ReleaseBehavior` · 0068c400 · legions 0062d430 — ac2-anim Ac2BehaviorTracker
- [x] `BehaviorTracker::GetTrack` · 0068c4c0 · legions 0062d4f0 — ac2-anim Ac2BehaviorTracker
- [x] `BehaviorTracker::IsDoingBehavior` · 0068c500 · legions 0062d530 — ac2-anim Ac2BehaviorTracker
- [-] `BehaviorTracker::~BehaviorTracker` · 0068c640 · legions 0062d670 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)
- [x] `BehaviorTracker::Update` · 0068c720 · legions 0062d750 — ac2-anim behavior/tracker.rs Ac2BehaviorTracker (update, init, do_behavior); their param handling is process_param_flags (done) and process_ramping_down_param_flags (partial: destroy and clone-end calls)
- [x] `BehaviorTracker::StopBehavior` · 0068cbd0 · legions 0062dbf0 — ac2-anim Ac2BehaviorTracker
- [x] `BehaviorTracker::StopAllWeenieBehaviors` · 0068cda0 · legions 0062ddc0 — ac2-anim Ac2BehaviorTracker
- [x] `BehaviorTracker::StopLayer` · 0068ce80 · legions 0062dea0 — ac2-anim Ac2BehaviorTracker

### _STL::_Rb_tree<unsigned_long,unsigned_long,_STL::_Identity<unsigned_long>,_STL::less<unsigned_long>,_STL::allocator<unsigned_long>_>
- [-] `_STL::_Rb_tree<unsigned_long,unsigned_long,_STL::_Identity<unsigned_long>,_STL::less<unsigned_long>,_STL::allocator<unsigned_long>_>::operator=` · 0068cfd0 · legions 0062dff0 — STL set/tree serialization (Rust collections)

### BehaviorTracker
- [x] `BehaviorTracker::StopMoveToCancelsBehaviors` · 0068d0b0 · legions 0062e0d0 — ac2-anim Ac2BehaviorTracker

### AnimTrack
- [-] `AnimTrack::~AnimTrack` · 0068d270 · legions 0062e290 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)

### BehaviorTracker
- [x] `BehaviorTracker::BehaviorTracker` · 0068d340 · legions 0062e360 — ac2-anim Ac2BehaviorTracker

### AnimTrack
- [-] `AnimTrack::operator=` · 0068d3e0 · legions 0062e400 — operator= of plain data (Rust value assignment)
- [-] `AnimTrack::AnimTrack` · 0068d460 · legions 0062e480 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)
- [-] `AnimTrack::IsRampingDown` · 0068d4e0 — trivial getter on plain struct (Rust is_ramping_down field)

### BehaviorTrack
- [x] `BehaviorTrack::BehaviorTrack` · 0068d520 · legions 0062e540 — ac2-anim Ac2BehaviorTracker
- [-] `BehaviorTrack::IsRampingDown` · 0068d5b0 — trivial getter on plain struct (Rust is_ramping_down field)

### AnimTrack
- [-] `AnimTrack::AnimTrack` · 0068d5e0 · legions 0062e600 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)

### BehaviorTracker
- [x] `BehaviorTracker::GetNetworkBehaviorParams` · 0068d690 · legions 0062e6b0 — ac2-anim: network behaviours and sliders, CharTemplate slider controllers (slice 4)

### BehaviorTrack
- [-] `BehaviorTrack::operator=` · 0068d710 · legions 0062e730 — operator= of plain data (Rust value assignment)
- [x] `BehaviorTrack::BehaviorTrack` · 0068d800 · legions 0062e820 — ac2-anim Ac2BehaviorTracker

### PHashTable<unsigned_long,BehaviorTrack>
- [-] `PHashTable<unsigned_long,BehaviorTrack>::StreamPack` · 0068d900 — container serialization/helper (Rust collections; elements decoded by their owning class)

### BehaviorTracker
- [x] `BehaviorTracker::Init` · 0068db80 · legions 0062eba0 — ac2-anim behavior/tracker.rs Ac2BehaviorTracker (update, init, do_behavior); their param handling is process_param_flags (done) and process_ramping_down_param_flags (partial: destroy and clone-end calls)
- [x] `BehaviorTracker::DoBehavior` · 0068dfc0 · legions 0062efe0 — ac2-anim behavior/tracker.rs Ac2BehaviorTracker (update, init, do_behavior); their param handling is process_param_flags (done) and process_ramping_down_param_flags (partial: destroy and clone-end calls)
- [x] `BehaviorTracker::DoAutonomousBehavior` · 0068e410 · legions 0062f430 — ac2-anim Ac2BehaviorTracker

## `portal/shared/src/animsys/chartemplate.cpp.c` (4/26)

### CharTemplate
- [-] `CharTemplate::GetCharTemplate` · 0067a6a0 — retail DB cache lookup/release (AGENTS exception); Rust loads the template by DID
- [-] `CharTemplate::ReleaseCharTemplate` · 0067a700 · legions 006240f0 — retail DB cache lookup/release (AGENTS exception); Rust loads the template by DID
- [-] `CharTemplate::GetDefaultElement` · 0067a720 — trivial getter on plain struct (Rust default_element / default_mode fields)
- [-] `CharTemplate::GetDefaultMode` · 0067a730 — trivial getter on plain struct (Rust default_element / default_mode fields)
- [-] `CharTemplate::FillDataGraph` · 0067a740 — CharTemplate::FillDataGraph feeds DBObj::AddToDataGraph (the data-graph prefetch); the only direct caller of AddToDataGraph is admin-only DBOCache::AddObj_Internal (no Legions body), and Legions DBObj::AddToDataGraph (0x006613C0) has none. The client reads each DID on demand
- [x] `CharTemplate::StreamPack` · 0067a7d0 · legions 00624200 — ac2-anim behavior_table.rs decode_template: behaviours, sliders, modes (isCombat, now kept), defaults and userControlledSliders (now kept); test the_char_template_keeps_modes_and_user_controlled_sliders
- [-] `CharTemplate::SaveToFile` · 0067a9f0 — admin text source format (ToFileNode/FromFileNode/Parse*Node), admin tools
- [-] `CharTemplate::Destroy` · 0067ad30 · legions 00624390 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing

### SmartArray<ParseHelper<EmpGenParse>::ParseHandler>
- [-] `SmartArray<ParseHelper<EmpGenParse>::ParseHandler>::AddToEnd` · 0067ae40 · legions 006b6b60 — container serialization/helper (Rust collections; elements decoded by their owning class)

### CharTemplate
- [x] `CharTemplate::GetBehavior` · 0067ae90 · legions 006244a0 — ac2-anim behavior_table.rs (Legions identical; Legions GetBehavior inlines GetCharTemplate)
- [x] `CharTemplate::GetSlider` · 0067aed0 — ac2-anim: network behaviours and sliders, CharTemplate slider controllers (slice 4)
- [x] `CharTemplate::IsUserControlled` · 0067af10 — ac2-anim: network behaviours and sliders, CharTemplate slider controllers (slice 4)

### HashTable<unsigned_long,Behavior>
- [-] `HashTable<unsigned_long,Behavior>::operator=` · 0067af70 · legions 006245a0 — container operator= (Rust collections)

### HashTable<unsigned_long,Slider>
- [-] `HashTable<unsigned_long,Slider>::operator=` · 0067b0f0 · legions 006246b0 — container operator= (Rust collections)

### HashTable<unsigned_long,Mode>
- [-] `HashTable<unsigned_long,Mode>::operator=` · 0067b260 · legions 006247b0 — container operator= (Rust collections)

### CharTemplate
- [-] `CharTemplate::~CharTemplate` · 0067b690 · legions 00624b70 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)
- [-] `CharTemplate::GetDBOType` · 0067b780 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing
- [-] `CharTemplate::ParseEMPsNode` · 0067b790 — admin text source format (ToFileNode/FromFileNode/Parse*Node), admin tools

### ParseHelper<EmpGenParse>
- [-] `ParseHelper<EmpGenParse>::AddHandler` · 0067c640 · legions 006b6ff0 — trivial setter / list append on plain struct during decode

### CharTemplate
- [-] `CharTemplate::CopyInto` · 0067c720 · legions 00624c90 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing
- [-] `CharTemplate::FromFileNode` · 0067c780 — text source format (admin tools)

### PHashTable<unsigned_long,Behavior>
- [-] `PHashTable<unsigned_long,Behavior>::StreamPack` · 0067c8c0 · legions 00624cf0 — container serialization/helper (Rust collections; elements decoded by their owning class)

### PHashTable<unsigned_long,Slider>
- [-] `PHashTable<unsigned_long,Slider>::StreamPack` · 0067cac0 — container serialization/helper (Rust collections; elements decoded by their owning class)

### PHashTable<unsigned_long,Mode>
- [-] `PHashTable<unsigned_long,Mode>::StreamPack` · 0067ccc0 — container serialization/helper (Rust collections; elements decoded by their owning class)

### CharTemplate
- [-] `CharTemplate::CharTemplate` · 0067cec0 · legions 006252f0 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)
- [-] `CharTemplate::Allocate` · 0067cf90 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing

## `portal/shared/src/animsys/combokey.cpp.c` (3/13)

### ComboKey
- [-] `ComboKey::operator%` · 0068e9f0 · legions 0062f9d0 — hash key equality/hash (Rust HashMap keys)
- [-] `ComboKey::operator==` · 0068ea10 · legions 0062f9f0 — hash key equality/hash (Rust HashMap keys)

### ModeTransitionKey
- [-] `ModeTransitionKey::operator%` · 0068ea40 · legions 0062fa20 — hash key equality/hash (Rust HashMap keys)
- [-] `ModeTransitionKey::operator==` · 0068ea60 · legions 0062fa40 — hash key equality/hash (Rust HashMap keys)

### ModeStateKey
- [-] `ModeStateKey::operator%` · 0068eaa0 · legions 0062fa80 — hash key equality/hash (Rust HashMap keys)
- [-] `ModeStateKey::operator==` · 0068eac0 · legions 0062faa0 — hash key equality/hash (Rust HashMap keys)

### ComboInfo
- [x] `ComboInfo::IsValid` · 0068eaf0 · legions 0062fad0 — ac2-anim behavior_table.rs (Legions identical; Legions GetBehavior inlines GetCharTemplate)
- [x] `ComboInfo::SetInvalid` · 0068eb00 · legions 0062fae0 — ac2-anim behavior_table.rs (Legions identical; Legions GetBehavior inlines GetCharTemplate)

### ComboKey
- [-] `ComboKey::ComboKey` · 0068eb10 · legions 0062faf0 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)

### ModeTransitionKey
- [-] `ModeTransitionKey::ModeTransitionKey` · 0068eb30 · legions 0062fb10 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)

### ModeStateKey
- [-] `ModeStateKey::ModeStateKey` · 0068eb50 · legions 0062fb30 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)

### TransitionInfo
- [-] `TransitionInfo::TransitionInfo` · 0068eb70 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)

### ComboKey
- [x] `ComboKey::StreamPack` · 0068eb90 · legions 0062fb50 — ac2-anim behavior_table.rs (Legions identical; Legions GetBehavior inlines GetCharTemplate)

## `portal/shared/src/animsys/dbanimator.cpp.c` (5/16)

### DBAnimator
- [-] `DBAnimator::DBAnimator` · 0067a440 · legions 00623ed0 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)
- [-] `DBAnimator::GetSubDataIDs` · 0067a460 — GetSubDataIDs lists a record's dependent DIDs for the data-graph prefetch (DBObj::AddToDataGraph); DBOCache::GetIfInMemory does not call it, it calls an int-returning vtable slot (+0x24) once when !m_bLoaded, not this void (type, array) slot. The client reads every DID on demand from the local DATs and Ac2DbAnimator's loader recurses by DID (dbanimator/tree.rs Loader::node), so no consumer exists
- [-] `DBAnimator::GetRecursiveSubDataIDs` · 0067a470 · legions 00623f00 — GetSubDataIDs lists a record's dependent DIDs for the data-graph prefetch (DBObj::AddToDataGraph); DBOCache::GetIfInMemory does not call it, it calls an int-returning vtable slot (+0x24) once when !m_bLoaded, not this void (type, array) slot. The client reads every DID on demand from the local DATs and Ac2DbAnimator's loader recurses by DID (dbanimator/tree.rs Loader::node), so no consumer exists
- [x] `DBAnimator::Animate` · 0067a480 · legions 00623f10 — ac2-anim dbanimator / ac2-physics anim_pose (slice 3)
- [x] `DBAnimator::GetTriggers` · 0067a490 · legions 00623f20 — ac2-anim dbanimator / ac2-physics anim_pose (slice 3)
- [x] `DBAnimator::GetHoldLoop` · 0067a4a0 · legions 00623f30 — ac2-anim dbanimator / ac2-physics anim_pose (slice 3)
- [x] `DBAnimator::GetPeriod` · 0067a4b0 · legions 00623f40 — ac2-anim dbanimator / ac2-physics anim_pose (slice 3)
- [-] `DBAnimator::Refresh` · 0067a4c0 — write-side/admin path with no Legions match (setups and anims are read-only in the DVD client) [vtable-checked: not a slot of any located Legions vtable]
- [-] `DBAnimator::CopyInto` · 0067a4d0 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing
- [-] `DBAnimator::Destroy` · 0067a4f0 · legions 00623f80 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing
- [-] `DBAnimator::LoadFromFile` · 0067a510 — text source loader, admin build only
- [-] `DBAnimator::SaveToFile` · 0067a530 — admin text source format (ToFileNode/FromFileNode/Parse*Node), admin tools
- [-] `DBAnimator::HasAnimator` · 0067a550 · legions 00623fa0 — tree-validation query (HasAnimator / CheckAnimationBoneNum fills an error-string list): its only callers are the same query on the sibling animators (calls table), no root caller in admin 7148 or Legions, so it is reached only from the authoring tools' checks; the client never validates animators
- [-] `DBAnimator::CheckAnimationBoneNum` · 0067a570 · legions 00623fc0 — tree-validation query (HasAnimator / CheckAnimationBoneNum fills an error-string list): its only callers are the same query on the sibling animators (calls table), no root caller in admin 7148 or Legions, so it is reached only from the authoring tools' checks; the client never validates animators
- [-] `DBAnimator::Allocate` · 0067a5a0 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing
- [x] `DBAnimator::StreamPack` · 0067a5f0 · legions 00624040 — ac2-anim dbanimator/tree.rs AnimatorRecord::decode_packed (DBAnimator::StreamPack: the DBObj identity, the animator's type word, Animator::Create, then the type's StreamPack)

## `portal/shared/src/animsys/fxscript.cpp.c` (2/16)

### FxScript
- [x] `FxScript::StreamPack` · 006859d0 · legions 00628970 — ac2-anim fxscript.rs Ac2FxScript (decode, real-data test over the 22 DAT records)

### PetInfo
- [-] `PetInfo::operator=` · 00685a40 · legions 00418c10 — operator= of plain data (Rust value assignment)

### FxScript
- [-] `FxScript::ParsePeriodNode` · 00685e20 — editor text source / cache allocator (admin tools or Rust value type)

### SmartArray<ImpulseTrigger>
- [-] `SmartArray<ImpulseTrigger>::operator=` · 00685e50 · legions 00628d60 — container operator= (Rust collections)

### Trigger
- [-] `Trigger::Trigger` · 00685ee0 · legions 00628df0 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)

### FxScript
- [-] `FxScript::ToFileNode` · 00686360 — editor text source / cache allocator (admin tools or Rust value type)

### ParseHelper<EmpGenParse>
- [-] `ParseHelper<EmpGenParse>::Parse` · 00686db0 · legions 006b63b0 — EMP text parsers (AnimInfo::ParseComboAttackNode and ParseHelper<EmpGenParse>::Parse): callers are EmpGenParse::ParseEmp and the FromFileNode loaders (tool text sources); the DVD client loads the binary DAT records (Animator::Create + StreamPack)

### SmartArray<Trigger>
- [-] `SmartArray<Trigger>::operator=` · 00687440 · legions 00629270 — container operator= (Rust collections)

### SmartArray<ImpulseTrigger>
- [-] `SmartArray<ImpulseTrigger>::AddToEnd` · 006874d0 · legions 00629300 — container serialization/helper (Rust collections; elements decoded by their owning class)

### FxScript
- [-] `FxScript::CopyInto` · 006877a0 · legions 006295d0 — editor text source / cache allocator (admin tools or Rust value type)

### SmartArray<Trigger>
- [-] `SmartArray<Trigger>::AddToEnd` · 006877f0 · legions 00629620 — container serialization/helper (Rust collections; elements decoded by their owning class)

### FxScript
- [x] `FxScript::GetTriggers` · 00687be0 · legions 00629a10 — ac2-anim fxscript_tracker.rs Ac2FxScriptTracker (timeline, rest at -4, triggers via add_effects_and_impulses); tests in fxscript_tracker_tests.rs
- [-] `FxScript::ParseImpulseNode` · 00688190 — editor text source / cache allocator (admin tools or Rust value type)
- [-] `FxScript::ParseEffectsNode` · 006884a0 — editor text source / cache allocator (admin tools or Rust value type)
- [-] `FxScript::ParseStopEffectsNode` · 006889e0 — editor text source / cache allocator (admin tools or Rust value type)
- [-] `FxScript::FromFileNode` · 00688d60 — editor text source / cache allocator (admin tools or Rust value type)

## `portal/shared/src/animsys/fxscripttracker.cpp.c` (4/7)

### (free functions)
- [-] `STREAMPACK` · 0068e480 · legions 0063af50 — STREAMPACK template wrapper for FXScriptTracker (calls its virtual StreamPack, runtime state); no call site in the Legions wlib scan (host::tests::efun_coverage)

### FXScriptTracker
- [x] `FXScriptTracker::StreamPack` · 0068e510 — ac2-anim fxscript_tracker.rs Ac2FxScriptTracker (timeline, rest at -4, triggers via add_effects_and_impulses); tests in fxscript_tracker_tests.rs
- [x] `FXScriptTracker::Update` · 0068e620 · legions 0062f5b0 — ac2-anim fxscript_tracker.rs Ac2FxScriptTracker (timeline, rest at -4, triggers via add_effects_and_impulses); tests in fxscript_tracker_tests.rs

### PList<gmActInfo>
- [-] `PList<gmActInfo>::GetPackSize` · 0068e700 — container serialization/helper (Rust collections; elements decoded by their owning class)

### FXScriptTracker
- [x] `FXScriptTracker::FXScriptTracker` · 0068e7b0 · legions 0062f790 — ac2-anim fxscript_tracker.rs Ac2FxScriptTracker (timeline, rest at -4, triggers via add_effects_and_impulses); tests in fxscript_tracker_tests.rs

### PList<FXTrack>
- [-] `PList<FXTrack>::UnPack` · 0068e8d0 · legions 0062f8c4 — container serialization/helper (Rust collections; elements decoded by their owning class)

### FXScriptTracker
- [x] `FXScriptTracker::DoFXScript` · 0068e990 · legions 0062f970 — ac2-anim fxscript_tracker.rs Ac2FxScriptTracker (timeline, rest at -4, triggers via add_effects_and_impulses); tests in fxscript_tracker_tests.rs

## `portal/shared/src/animsys/fxtrack.cpp.c` (5/6)

### FXTrack
- [x] `FXTrack::FillCache` · 00690130 · legions 00630cb0 — ac2-anim fxscript_tracker.rs Ac2FxScriptTracker::resolve: the FX script is fetched by DID and cached on the track (FXTrack::FillCache); the frame thread never reads, so the caller preloads scripts (fx_scripts.rs load_fx_scripts)
- [-] `FXTrack::StreamPack` · 00690190 · legions 00630d00 — StreamPack of runtime tracker state (BehaviorTrack, FXTrack, BehaviorTracker, ModeTracker, SliderTracker, TriggerUpdate): nothing calls it: no direct caller in the calls table and no other function in the index refers to these classes' StreamPack; the live state is Ac2ModeTracker/Ac2BehaviorTracker/Ac2SliderTracker/Ac2TriggerUpdate, rebuilt from the server's network behaviour params
- [x] `FXTrack::operator=` · 00690220 · legions 00630d90 — ac2-anim fxscript_tracker.rs Ac2FxScriptTracker (timeline, rest at -4, triggers via add_effects_and_impulses); tests in fxscript_tracker_tests.rs
- [x] `FXTrack::~FXTrack` · 006902a0 · legions 00630e10 — ac2-anim fxscript_tracker.rs Ac2FxScriptTracker (timeline, rest at -4, triggers via add_effects_and_impulses); tests in fxscript_tracker_tests.rs
- [x] `FXTrack::FXTrack` · 006902d0 · legions 00630e40 — ac2-anim fxscript_tracker.rs Ac2FxScriptTracker (timeline, rest at -4, triggers via add_effects_and_impulses); tests in fxscript_tracker_tests.rs
- [x] `FXTrack::FXTrack` · 00690300 · legions 00630e70 — ac2-anim fxscript_tracker.rs Ac2FxScriptTracker (timeline, rest at -4, triggers via add_effects_and_impulses); tests in fxscript_tracker_tests.rs

## `portal/shared/src/animsys/impulsetrigger.cpp.c` (2/3)

### ImpulseTrigger
- [x] `ImpulseTrigger::operator==` · 00689180 · legions 0062a1a0 — ac2-anim behavior/triggers + dbanimator/triggers (slice 4): effect, hook and impulse triggers through TriggerUpdate
- [-] `ImpulseTrigger::operator<` · 006891f0 · legions 0062a210 — Trigger/ImpulseTrigger/WeenieHook operator< (order by mTime): used only by the STL sorts in AnimInfo::Parse*Node and FxScript::Parse*Node (EMP text parsers, tool paths; calls table). The DAT stores the lists already sorted and the decoders keep stored order
- [x] `ImpulseTrigger::StreamPack` · 00689210 · legions 0062a230 — ac2-anim dbanimator / ac2-physics anim_pose (slice 3)

## `portal/shared/src/animsys/mode.cpp.c` (1/2)

### Mode
- [-] `Mode::Mode` · 0068f400 · legions 0062ff80 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)
- [x] `Mode::StreamPack` · 0068f410 — ac2-anim behavior_table.rs decode_template / transition_key (Behavior::StreamPack 64-bit flags, Mode::StreamPack isCombat, Slider::StreamPack controller, ModeTransitionKey::StreamPack); test the_char_template_keeps_modes_and_user_controlled_sliders and retail_behavior_tables_decode

## `portal/shared/src/animsys/modetracker.cpp.c` (21/23)

### ModeTracker
- [x] `ModeTracker::GetTriggerUpdate` · 0068a8b0 · legions 0062b8f0 — ac2-anim behavior/mode_tracker.rs Ac2ModeTracker::triggers_mut: the TriggerUpdate belongs to the mode tracker (ModeTracker::GetTriggerUpdate), fired by physics update_animation
- [-] `ModeTracker::SetAnimationObj` · 0068a8d0 · legions 0062b910 — ModeTracker::SetAnimationObj binds the AnimationObj pointer; Ac2ModeTracker receives the object's queries per call (the Ac2AnimationSource argument of update, do_behavior and the rest) instead of storing a pointer
- [x] `ModeTracker::Init` · 0068a900 · legions 0062b940 — ac2-anim Ac2ModeTracker
- [x] `ModeTracker::SetMode` · 0068a9f0 · legions 0062ba20 — ac2-anim Ac2ModeTracker
- [x] `ModeTracker::DoAutonomousBehavior` · 0068aac0 · legions 0062baf0 — ac2-anim Ac2ModeTracker
- [x] `ModeTracker::ReleaseBehavior` · 0068ab10 · legions 0062bb40 — ac2-anim Ac2ModeTracker
- [x] `ModeTracker::StopBehavior` · 0068ab90 · legions 0062bbc0 — ac2-anim Ac2ModeTracker
- [x] `ModeTracker::StopMoveToCancelsBehaviors` · 0068ac50 · legions 0062bc80 — ac2-anim Ac2ModeTracker
- [x] `ModeTracker::StopLayer` · 0068acd0 · legions 0062bd00 — ac2-anim Ac2ModeTracker
- [x] `ModeTracker::StopAllWeenieBehaviors` · 0068ad20 · legions 0062bd50 — ac2-anim Ac2ModeTracker
- [x] `ModeTracker::GetElement` · 0068ad50 · legions 0062bd80 — ac2-anim Ac2ModeTracker
- [x] `ModeTracker::IsDoingBehavior` · 0068ad70 · legions 0062bda0 — ac2-anim Ac2ModeTracker
- [x] `ModeTracker::CheckForDynamicFPCamera` · 0068adb0 · legions 0062bde0 — ac2-anim behavior/mode_tracker.rs check_for_dynamic_fp_camera and tracker.rs check_active_behaviors_for_dynamic_fp_camera (incoming tracker else outgoing; any track flagged DYNAMIC_FP_CAMERA); test a_dynamic_first_person_camera_behaviour_is_found_while_it_plays. The consumer, CameraManager::UpdateCamera, is the camera's (ac2-smartbox)
- [x] `ModeTracker::GetNetworkBehaviorParams` · 0068add0 · legions 0062be00 — ac2-anim: network behaviours and sliders, CharTemplate slider controllers (slice 4)
- [x] `ModeTracker::ModeTracker` · 0068adf0 · legions 0062be20 — ac2-anim Ac2ModeTracker
- [x] `ModeTracker::GetDesiredModeState` · 0068ae30 · legions 0062be60 — ac2-anim Ac2ModeTracker
- [-] `ModeTracker::StreamPack` · 0068ae60 · legions 0062be90 — StreamPack of runtime tracker state (BehaviorTrack, FXTrack, BehaviorTracker, ModeTracker, SliderTracker, TriggerUpdate): nothing calls it: no direct caller in the calls table and no other function in the index refers to these classes' StreamPack; the live state is Ac2ModeTracker/Ac2BehaviorTracker/Ac2SliderTracker/Ac2TriggerUpdate, rebuilt from the server's network behaviour params
- [x] `ModeTracker::GetModeState` · 0068b060 · legions 0062c090 — ac2-anim Ac2ModeTracker
- [x] `ModeTracker::DoBehavior` · 0068b0b0 · legions 0062c0e0 — ac2-anim Ac2ModeTracker
- [x] `ModeTracker::StartTransitioning` · 0068b1d0 · legions 0062c200 — ac2-anim behavior/mode_tracker.rs Ac2ModeTracker (start_transitioning: transition behaviour, effect and stop effect through fire_and_forget; update: transition time, swap, AddTriggers after the caller's DoAnimation); the body of the retail function has no camera, fade or clone use
- [x] `ModeTracker::DoElement` · 0068b400 · legions 0062c430 — ac2-anim Ac2ModeTracker
- [x] `ModeTracker::DoMode` · 0068b450 · legions 0062c480 — ac2-anim Ac2ModeTracker
- [x] `ModeTracker::Update` · 0068b510 · legions 0062c540 — ac2-anim behavior/mode_tracker.rs Ac2ModeTracker (start_transitioning: transition behaviour, effect and stop effect through fire_and_forget; update: transition time, swap, AddTriggers after the caller's DoAnimation); the body of the retail function has no camera, fade or clone use

## `portal/shared/src/animsys/slider.cpp.c` (1/2)

### Slider
- [-] `Slider::Slider` · 0068f3b0 · legions 0062ff30 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)
- [x] `Slider::StreamPack` · 0068f3c0 · legions 0062ff40 — ac2-anim behavior_table.rs decode_template / transition_key (Behavior::StreamPack 64-bit flags, Mode::StreamPack isCombat, Slider::StreamPack controller, ModeTransitionKey::StreamPack); test the_char_template_keeps_modes_and_user_controlled_sliders and retail_behavior_tables_decode

## `portal/shared/src/animsys/slidertracker.cpp.c` (11/16)

### SliderInterpolator
- [x] `SliderInterpolator::Activate` · 006898e0 · legions 0062a900 — ac2-anim behavior/slider.rs Ac2SliderTracker (SliderInterpolator::Activate, GetSliderInterpolator creating the resting interpolator, Update advancing interpolators after the sliders: time += quantum, an active one past its length rests at its end value); test a_slider_interpolator_ramps_for_its_length_then_rests_at_its_end

### ModeTransitionKey
- [x] `ModeTransitionKey::StreamPack` · 00689910 · legions 0062fba0 — ac2-anim behavior_table.rs decode_template / transition_key (Behavior::StreamPack 64-bit flags, Mode::StreamPack isCombat, Slider::StreamPack controller, ModeTransitionKey::StreamPack); test the_char_template_keeps_modes_and_user_controlled_sliders and retail_behavior_tables_decode

### SliderTracker
- [-] `SliderTracker::StreamPack` · 00689970 · legions 0062a930 — StreamPack of runtime tracker state (BehaviorTrack, FXTrack, BehaviorTracker, ModeTracker, SliderTracker, TriggerUpdate): nothing calls it: no direct caller in the calls table and no other function in the index refers to these classes' StreamPack; the live state is Ac2ModeTracker/Ac2BehaviorTracker/Ac2SliderTracker/Ac2TriggerUpdate, rebuilt from the server's network behaviour params

### SliderInterpolator
- [-] `SliderInterpolator::StreamPack` · 006899c0 · legions 0062a980 — StreamPack of runtime tracker state (BehaviorTrack, FXTrack, BehaviorTracker, ModeTracker, SliderTracker, TriggerUpdate): nothing calls it: no direct caller in the calls table and no other function in the index refers to these classes' StreamPack; the live state is Ac2ModeTracker/Ac2BehaviorTracker/Ac2SliderTracker/Ac2TriggerUpdate, rebuilt from the server's network behaviour params

### SliderTracker
- [x] `SliderTracker::Update` · 00689ab0 · legions 0062aab0 — ac2-anim behavior/slider.rs Ac2SliderTracker (SliderInterpolator::Activate, GetSliderInterpolator creating the resting interpolator, Update advancing interpolators after the sliders: time += quantum, an active one past its length rests at its end value); test a_slider_interpolator_ramps_for_its_length_then_rests_at_its_end
- [x] `SliderTracker::GetNetworkSliders` · 00689c20 · legions 0062ac20 — ac2-anim: network behaviours and sliders, CharTemplate slider controllers (slice 4)
- [x] `SliderTracker::GetSliderCurrentValue` · 00689d60 · legions 0062ada0 — ac2-anim Ac2SliderTracker
- [x] `SliderTracker::GetSliderDesiredValue` · 00689da0 · legions 0062ade0 — ac2-anim Ac2SliderTracker
- [x] `SliderTracker::GetSliderVelocity` · 00689de0 · legions 0062ae20 — ac2-anim Ac2SliderTracker
- [-] `SliderTracker::~SliderTracker` · 0068a130 · legions 0062b170 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)
- [x] `SliderTracker::DoSlider` · 0068a1d0 · legions 0062b210 — ac2-anim Ac2SliderTracker
- [x] `SliderTracker::GetSliderInterpolator` · 0068a320 · legions 0062b360 — ac2-anim behavior/slider.rs Ac2SliderTracker (SliderInterpolator::Activate, GetSliderInterpolator creating the resting interpolator, Update advancing interpolators after the sliders: time += quantum, an active one past its length rests at its end value); test a_slider_interpolator_ramps_for_its_length_then_rests_at_its_end
- [x] `SliderTracker::Init` · 0068a400 · legions 0062b440 — ac2-anim Ac2SliderTracker

### PHashTable<unsigned_long,SliderInfo>
- [-] `PHashTable<unsigned_long,SliderInfo>::StreamPack` · 0068a490 — container serialization/helper (Rust collections; elements decoded by their owning class)

### PHashTable<unsigned_long,SliderInterpolator>
- [-] `PHashTable<unsigned_long,SliderInterpolator>::StreamPack` · 0068a660 · legions 0062b6a0 — container serialization/helper (Rust collections; elements decoded by their owning class)

### SliderTracker
- [x] `SliderTracker::SliderTracker` · 0068a840 · legions 0062b880 — ac2-anim Ac2SliderTracker

## `portal/shared/src/animsys/trigger.cpp.c` (6/7)

### Trigger
- [x] `Trigger::operator==` · 00688fa0 · legions 00629fc0 — ac2-anim behavior/triggers + dbanimator/triggers (slice 4): effect, hook and impulse triggers through TriggerUpdate
- [-] `Trigger::operator<` · 00688fe0 · legions 0062a000 — Trigger/ImpulseTrigger/WeenieHook operator< (order by mTime): used only by the STL sorts in AnimInfo::Parse*Node and FxScript::Parse*Node (EMP text parsers, tool paths; calls table). The DAT stores the lists already sorted and the decoders keep stored order
- [x] `Trigger::Trigger` · 00689000 · legions 0062a020 — ac2-anim behavior/triggers + dbanimator/triggers (slice 4): effect, hook and impulse triggers through TriggerUpdate
- [x] `Trigger::Trigger` · 00689040 · legions 0062a060 — ac2-anim behavior/triggers + dbanimator/triggers (slice 4): effect, hook and impulse triggers through TriggerUpdate
- [x] `Trigger::Trigger` · 00689080 · legions 0062a0a0 — ac2-anim behavior/triggers + dbanimator/triggers (slice 4): effect, hook and impulse triggers through TriggerUpdate
- [x] `Trigger::Trigger` · 006890c0 · legions 0062a0e0 — ac2-anim behavior/triggers + dbanimator/triggers (slice 4): effect, hook and impulse triggers through TriggerUpdate
- [x] `Trigger::StreamPack` · 00689100 — ac2-anim dbanimator / ac2-physics anim_pose (slice 3)

## `portal/shared/src/animsys/triggerupdate.cpp.c` (7/8)

### TriggerUpdate
- [-] `TriggerUpdate::StreamPack` · 00689290 · legions 0062a2b0 — StreamPack of runtime tracker state (BehaviorTrack, FXTrack, BehaviorTracker, ModeTracker, SliderTracker, TriggerUpdate): nothing calls it: no direct caller in the calls table and no other function in the index refers to these classes' StreamPack; the live state is Ac2ModeTracker/Ac2BehaviorTracker/Ac2SliderTracker/Ac2TriggerUpdate, rebuilt from the server's network behaviour params
- [x] `TriggerUpdate::IsEmpty` · 006892f0 · legions 0062a310 — ac2-anim behavior/triggers + dbanimator/triggers (slice 4): effect, hook and impulse triggers through TriggerUpdate
- [x] `TriggerUpdate::AddWeenieHook` · 00689330 · legions 0062a350 — ac2-anim behavior/triggers + dbanimator/triggers (slice 4): effect, hook and impulse triggers through TriggerUpdate
- [x] `TriggerUpdate::AddImpulseTrigger` · 006893b0 · legions 0062a3d0 — ac2-anim behavior/triggers + dbanimator/triggers (slice 4): effect, hook and impulse triggers through TriggerUpdate
- [x] `TriggerUpdate::OnTrackTerminated` · 006894d0 · legions 0062a4f0 — ac2-anim behavior/triggers + dbanimator/triggers (slice 4): effect, hook and impulse triggers through TriggerUpdate
- [x] `TriggerUpdate::CleanUp` · 00689680 · legions 0062a6a0 — ac2-anim behavior/triggers + dbanimator/triggers (slice 4): effect, hook and impulse triggers through TriggerUpdate
- [x] `TriggerUpdate::AddEffectsTrigger` · 00689710 · legions 0062a730 — ac2-anim behavior/triggers.rs Ac2TriggerUpdate::add_effects_trigger records the probability and intensity rolls in add order and draws them on the shared Random when the triggers fire; AMRandom's roll (animate) comes before the adds in both orders (DoAnimation, then AddTriggers), so the draw sequence is retail's; trigger_tests.rs
- [x] `TriggerUpdate::AddTrackTriggers` · 00689880 · legions 0062a8a0 — ac2-anim behavior/triggers + dbanimator/triggers (slice 4): effect, hook and impulse triggers through TriggerUpdate

## `portal/shared/src/animsys/validmodes.cpp.c` (0/7)

### ValidModes
- [-] `ValidModes::StreamPack` · 00685590 — ValidModes (m_available_modes list) has no reader: no function in the index refers to it besides its allocator and own members, so the client never loads it
- [-] `ValidModes::Destroy` · 006855c0 · legions 00628860 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing
- [-] `ValidModes::ValidModes` · 006855f0 · legions 00628890 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)
- [-] `ValidModes::GetDBOType` · 00685620 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing
- [-] `ValidModes::Allocate` · 00685630 — DBO plumbing (Destroy/Allocate/GetDBOType/CopyMe/CopyInto): Rust drop, Clone or Arc sharing
- [-] `ValidModes::ToFileNode` · 00685680 — text source format (admin tools)
- [-] `ValidModes::FromFileNode` · 00685790 — text source format (admin tools)

## `portal/shared/src/animsys/weeniehook.cpp.c` (2/4)

### WeenieHook
- [x] `WeenieHook::operator==` · 0068f440 · legions 0062ffc0 — ac2-anim behavior/triggers + dbanimator/triggers (slice 4): effect, hook and impulse triggers through TriggerUpdate
- [-] `WeenieHook::operator<` · 0068f470 · legions 0062fff0 — Trigger/ImpulseTrigger/WeenieHook operator< (order by mTime): used only by the STL sorts in AnimInfo::Parse*Node and FxScript::Parse*Node (EMP text parsers, tool paths; calls table). The DAT stores the lists already sorted and the decoders keep stored order
- [-] `WeenieHook::WeenieHook` · 0068f490 · legions 00630010 — ctor/dtor of plain data (Rust owned values, no retail object lifetime)
- [x] `WeenieHook::StreamPack` · 0068f4c0 · legions 00630040 — ac2-anim dbanimator / ac2-physics anim_pose (slice 3)
