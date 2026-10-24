# ac2-world: Cells, landblocks, regions, terrain, sky, water, game time

989 retail functions: 116 done, 10 partial, 720 to do, 143 skipped.
Legend and workflow: [README](README.md).

## `portal/shared/src/cell/envcell.cpp.c` (12/140)

### PackObj
- [ ] `PackObj::Pack` · 0047f1a0

### (free functions)
- [ ] `STREAMPACK_ALIGN` · 0047f1c0 · legions 004791d0

### Frame
- [x] `Frame::localtoglobalvec` · 0047f1f0 · legions 00479200

### FileNodeName_Bool
- [ ] `FileNodeName_Bool::Clone` · 0047f250 · legions 00479260

### BBox
- [ ] `BBox::DoesOverlap` · 0047f280 · legions 00479290

### RGBAColor
- [ ] `RGBAColor::GetColor32` · 0047f3c0 · legions 004793d0
- [x] `RGBAColor::SetColor32` · 0047f410 · legions 00479420
- [x] `RGBAColor::CapLow` · 0047f4b0 · legions 004794c0
- [x] `RGBAColor::CapHigh` · 0047f500 · legions 00479510

### Plane
- [ ] `Plane::which_side` · 0047f550 · legions 00479560

### Waveform
- [ ] `Waveform::operator=` · 0047f5a0 · legions 004795b0

### LightSource
- [ ] `LightSource::SetDiffuseIntensity` · 0047f5f0 · legions 00479600

### AppliedAppearanceKey
- [ ] `AppliedAppearanceKey::AppliedAppearanceKey` · 0047f650 · legions 00479660

### IconLayerDesc
- [ ] `IconLayerDesc::operator=` · 0047f670 · legions 00479680

### IconModAtom
- [ ] `IconModAtom::IconModAtom` · 0047f6b0
- [ ] `IconModAtom::~IconModAtom` · 0047f6d0

### CEnvCell
- [ ] `CEnvCell::CullingPlaneCheck` · 0047f730 · legions 00479700

### Frustum
- [ ] `Frustum::operator=` · 0047f9b0 · legions 00479980

### CEnvCell
- [x] `CEnvCell::point_in_cell` · 0047fa10 — ac2-physics envcell.rs, find_cells.rs: interior collision, containment and portal walk (Legions identical)
- [x] `CEnvCell::find_collisions` · 0047fa50 — ac2-physics envcell.rs, find_cells.rs: interior collision, containment and portal walk (Legions identical)
- [x] `CEnvCell::find_env_collisions` · 0047fa80 — ac2-physics envcell.rs, find_cells.rs: interior collision, containment and portal walk (Legions identical)
- [ ] `CEnvCell::GetSubDataIDs` · 0047fb70 · legions 00479b10
- [ ] `CEnvCell::SumBumpMapVectors` · 0047fbd0 · legions 00479b70
- [ ] `CEnvCell::NormalizeBumpMapVectors` · 0047fd00 · legions 00479ca0
- [ ] `CEnvCell::BuildOptionalProperties` · 00480180
- [ ] `CEnvCell::GetPhysicsMaterialDID` · 004801b0

### SmartArray<unsigned_long>
- [ ] `SmartArray<unsigned_long>::SetNElements` · 004801c0 · legions 0047a160

### LandDefs
- [ ] `LandDefs::inbound_valid_blockid` · 00480320 · legions 0047a290
- [ ] `LandDefs::blockpart_to_blockcoord` · 00480390 · legions 0047a300

### FileNodeName_UInt32
- [ ] `FileNodeName_UInt32::Clone` · 00480400 · legions 0047a370

### Ray
- [ ] `Ray::Ray` · 00480430 · legions 0047a3a0

### CellFrustum
- [ ] `CellFrustum::CellFrustum` · 00480470 · legions 0047a3e0

### OpacityAtom
- [ ] `OpacityAtom::~OpacityAtom` · 004804f0

### MeshReplacementAtom
- [ ] `MeshReplacementAtom::~MeshReplacementAtom` · 00480550

### PGDReplacementAtom
- [ ] `PGDReplacementAtom::PGDReplacementAtom` · 004805b0
- [ ] `PGDReplacementAtom::~PGDReplacementAtom` · 004805e0

### CEnvCell
- [ ] `CEnvCell::Destroy` · 00480650 · legions 0047a520
- [ ] `CEnvCell::RecoverFromFreelist` · 004807b0
- [ ] `CEnvCell::ClearForFreelist` · 00480800 · legions 0047a6d0
- [x] `CEnvCell::GetTouchesOutside` · 00480860 · legions 0047a730 — ac2-physics physics_obj_place.rs: SetPosition placement check (Legions identical)
- [ ] `CEnvCell::DestroyStaticLights` · 00480890 · legions 0047a760
- [~] `CEnvCell::DrawCellMesh` · 00480900 · legions 0047a7d0 — cell mesh with appearance atoms at the cell frame, point lights cached at 1.5x bbox radius; degrade selection fixed at top
- [ ] `CEnvCell::CullObjects` · 00480a80 · legions 0047a940
- [ ] `CEnvCell::LinkStaticLights` · 00480bc0 · legions 0047aa80
- [ ] `CEnvCell::LinkDynamicLights` · 00480c00 · legions 0047aac0
- [ ] `CEnvCell::RenderLightCoronas` · 00480c40 · legions 0047ab00
- [x] `CEnvCell::FindTransitCellsInternal` · 00480cb0 · legions 0047ab70 — ac2-physics envcell.rs, find_cells.rs: interior collision, containment and portal walk (Legions identical)
- [ ] `CEnvCell::ReleaseSubObjects` · 00480f30 · legions 0047adf0
- [ ] `CEnvCell::CopyDiffuseToVB` · 00480fa0 · legions 0047ae60
- [ ] `CEnvCell::CacheDiffuseFromVB` · 00480ff0 · legions 0047aeb0
- [x] `CEnvCell::LightIndices` · 00481030 · legions 0047aef0 — ac2-render object::cell / point_lights
- [ ] `CEnvCell::TraceRayAgainstMesh` · 004813a0 · legions 0047b260
- [ ] `CEnvCell::ApplyPropertyCollection` · 00481580
- [ ] `CEnvCell::InqVisibleCells` · 00481630 · legions 0047b4f0
- [ ] `CEnvCell::GetCellPlanes` · 00481680

### IntrusiveSmartPointer<LightSource>
- [ ] `IntrusiveSmartPointer<LightSource>::~IntrusiveSmartPointer<LightSource>` · 00481710 · legions 0047b5d0
- [ ] `IntrusiveSmartPointer<LightSource>::create` · 00481740 · legions 0047b600

### FileNodeName_PString
- [ ] `FileNodeName_PString::Clone` · 00481cf0 · legions 0047bc60
- [ ] `FileNodeName_PString::FromPString` · 00481d20 · legions 0047bc90
- [ ] `FileNodeName_PString::ToPString` · 00481d70 · legions 0047bce0

### PFileNode
- [ ] `PFileNode::GetNameAsBool` · 00481dc0 · legions 0047bd30
- [ ] `PFileNode::GetLeafAsBool` · 00481e80
- [ ] `PFileNode::GetNameAsUInt32` · 00481ea0 · legions 0047bdf0
- [ ] `PFileNode::GetNameAsPString` · 00481f60 · legions 0047beb0
- [ ] `PFileNode::GetSubNameAsPString` · 00482040

### LOSParameters
- [ ] `LOSParameters::SetFinalRay` · 00482060 · legions 0047bf90

### SceneTool
- [ ] `SceneTool::GetSunlight` · 004820b0 · legions 0047bfe0

### CEnvCell
- [ ] `CEnvCell::CEnvCell` · 004820d0 · legions 0047c000
- [ ] `CEnvCell::HasLighting` · 004822c0
- [ ] `CEnvCell::SetGatherTimestamp` · 004822d0
- [ ] `CEnvCell::GetGatherTimestamp` · 004822e0
- [ ] `CEnvCell::GetDBOType` · 004822f0
- [ ] `CEnvCell::~CEnvCell` · 00482310 · legions 0047c240
- [x] `CEnvCell::FindTransitCells` · 00482450 — ac2-physics envcell.rs, find_cells.rs: interior collision, containment and portal walk (Legions identical)
- [ ] `CEnvCell::GetSubObjects` · 00482460
- [ ] `CEnvCell::ApplyAtomCollection` · 004825c0 · legions 0047c490
- [ ] `CEnvCell::UpdateDynamicLights` · 00482710 · legions 0047c5b0
- [ ] `CEnvCell::UpdateCellMaterial` · 00482a80

### SmartArray<CCellPortal>
- [ ] `SmartArray<CCellPortal>::SetNElements` · 00482b40 · legions 0047c9e0

### PFileNode
- [ ] `PFileNode::GetSubName` · 00482c10 · legions 0047cab0

### CEnvCell
- [ ] `CEnvCell::Allocate` · 00482c30
- [ ] `CEnvCell::Allocator` · 00482c40
- [ ] `CEnvCell::DynamicLightVisibleCells` · 00482c80 · legions 0047cb20
- [ ] `CEnvCell::find_in_visible_cells` · 00482d00 · legions 0047cba0
- [ ] `CEnvCell::StreamPack` · 00482da0

### SmartArray<Vector>
- [ ] `SmartArray<Vector>::AddToEnd` · 00483190 · legions 0047d030

### SmartArray<PhysObjBrush>
- [ ] `SmartArray<PhysObjBrush>::AddToEnd` · 004831f0 · legions 0047d090

### SmartArray<CellFrustum>
- [ ] `SmartArray<CellFrustum>::AddToEnd` · 00483250 · legions 0047d0f0

### SmartArray<Frustum>
- [ ] `SmartArray<Frustum>::AddToEnd` · 004832c0 · legions 0047d160

### FileNodeName_PString
- [ ] `FileNodeName_PString::GetPackSize` · 00483410 · legions 0047d2b0
- [ ] `FileNodeName_PString::Pack` · 00483430 · legions 0047d2d0
- [ ] `FileNodeName_PString::UnPack` · 00483440 · legions 0047d2e0

### CEnvCell
- [ ] `CEnvCell::CreatePhysicsObj` · 00483460 · legions 0047d300
- [ ] `CEnvCell::CreateLightSource` · 004834f0 · legions 0047d390
- [ ] `CEnvCell::CheckVisibility` · 00483680 · legions 0047d520
- [ ] `CEnvCell::GatherInsideObjects` · 00483970 · legions 0047d810
- [-] `CEnvCell::FromFileNode` · 00483a60 — text source format (admin tools)
- [ ] `CEnvCell::AddStaticLights` · 00484090 · legions 0047d900
- [ ] `CEnvCell::GetNearbyGeometry` · 00484200 · legions 0047da70
- [~] `CEnvCell::InitStaticObjects` · 00484680 · legions 0047def0 — ac2-physics load_block_interiors: cell entities as static objects (no ObjectClearance/CollisionParameters)
- [ ] `CEnvCell::InitLightSources` · 00484710 · legions 0047df80
- [ ] `CEnvCell::GatherInsideObjects` · 00484760 · legions 0047dfd0
- [ ] `CEnvCell::InitLoad` · 00484800

### PList<IconModAtom>
- [ ] `PList<IconModAtom>::GetPackSize` · 004848a0
- [ ] `PList<IconModAtom>::UnPack` · 004848e0 · legions 0047e1b4

### ListNode<IconModAtom>
- [ ] `ListNode<IconModAtom>::ListNode<IconModAtom>` · 00484a30 · legions 0047e2f0

### CEnvCell
- [ ] `CEnvCell::GetVisible` · 00484e30 · legions 0047e710
- [ ] `CEnvCell::CullCellGroupInternal` · 00484e70 · legions 0047e750
- [ ] `CEnvCell::CullCellsInternal` · 00485290 · legions 0047eb70
- [ ] `CEnvCell::DrawEnvironment` · 004856b0
- [ ] `CEnvCell::DrawPostEffects` · 00485770
- [ ] `CEnvCell::GatherVisibleInsideObjects` · 00485800
- [x] `CEnvCell::find_visible_child_cell` · 00485890 · legions 0047f140 — ac2-physics physics_obj_place.rs: SetPosition placement check (Legions identical)
- [ ] `CEnvCell::TraceRay` · 00485930 · legions 0047f1e0

### PList<MaterialModifierAtom>
- [ ] `PList<MaterialModifierAtom>::Pack` · 00486180 · legions 0047fa20
- [ ] `PList<MaterialModifierAtom>::GetPackSize` · 004861d0
- [ ] `PList<MaterialModifierAtom>::UnPack` · 00486210 · legions 0047fac4

### MaterialModifierAtom
- [ ] `MaterialModifierAtom::MaterialModifierAtom` · 00486360 · legions 0047fc00

### PList<OpacityAtom>
- [ ] `PList<OpacityAtom>::Pack` · 004863b0 · legions 0064fdb0
- [ ] `PList<OpacityAtom>::UnPack` · 00486400 · legions 0047fc64

### PList<MeshReplacementAtom>
- [ ] `PList<MeshReplacementAtom>::UnPack` · 00486560 · legions 0047fe04

### ListNode<MeshReplacementAtom>
- [ ] `ListNode<MeshReplacementAtom>::ListNode<MeshReplacementAtom>` · 004866a0 · legions 0047ff30

### PList<PGDReplacementAtom>
- [ ] `PList<PGDReplacementAtom>::Pack` · 004866e0 · legions 0047ff70
- [ ] `PList<PGDReplacementAtom>::GetPackSize` · 00486730
- [ ] `PList<PGDReplacementAtom>::UnPack` · 00486770

### ListNode<PGDReplacementAtom>
- [ ] `ListNode<PGDReplacementAtom>::ListNode<PGDReplacementAtom>` · 004868f0 · legions 00480180

### CEnvCell
- [ ] `CEnvCell::Trace` · 00486af0
- [ ] `CEnvCell::release_visible` · 00486b00 · legions 00480390
- [ ] `CEnvCell::ReleaseVisibleCells` · 00486bd0
- [ ] `CEnvCell::add_visible_cell` · 00486d10 · legions 004805a0
- [ ] `CEnvCell::CullCells` · 00486dd0
- [ ] `CEnvCell::grab_visible` · 00486ee0 · legions 00480740
- [ ] `CEnvCell::GrabVisibleCells` · 00486f10 · legions 00480770

### AtomCollection
- [ ] `AtomCollection::AtomCollection` · 00486f70 · legions 004807d0
- [ ] `AtomCollection::~AtomCollection` · 00487060 · legions 004808c0

### CEnvCell
- [ ] `CEnvCell::BuildAndApplyAtomCollection` · 00487180 · legions 004809e0
- [ ] `CEnvCell::ApplyProperty` · 00487270
- [ ] `CEnvCell::ComputeCellGroups` · 004873b0 · legions 00480be0
- [ ] `CEnvCell::FlushCellGroups` · 004876a0 · legions 00480ed0
- [~] `CEnvCell::flush_cells` · 00487750 · legions 00480f80 — static diffuse bake (base colour + LightIndices); SumBumpMapVectors not yet

## `portal/shared/src/cell/landblock.cpp.c` (12/72)

### LandDefs
- [ ] `LandDefs::GetBlockOffset` · 00488140 · legions 00481970
- [ ] `LandDefs::within_block` · 00488270 · legions 00481aa0

### EtherealPhysicsType
- [ ] `EtherealPhysicsType::SetPropertyEthereal` · 004882c0 · legions 00481af0

### IError
- [ ] `IError::ReportDataError` · 004882d0

### Plane
- [ ] `Plane::set_height` · 004882f0 · legions 00481b00

### Random
- [ ] `Random::pseudorand2` · 00488330 · legions 00481b40

### SceneGroup
- [ ] `SceneGroup::HasRadiusCheck` · 00488390 · legions 00481ba0

### CLandBlock
- [ ] `CLandBlock::Init` · 004883b0 · legions 00481bc0
- [ ] `CLandBlock::SetVertexValue` · 004883c0 · legions 00481bd0
- [x] `CLandBlock::OnRoad` · 004883d0 · legions 00481be0
- [ ] `CLandBlock::GrabVisibleCells` · 004886f0 · legions 00481f00
- [ ] `CLandBlock::ReleaseVisibleCells` · 00488720 · legions 00481f30
- [ ] `CLandBlock::FindInteriorCell` · 00488750 · legions 00481f60
- [ ] `CLandBlock::InitLBI` · 004887f0 · legions 00482000

### LandDefs
- [ ] `LandDefs::get_block_gid` · 00488b70 · legions 004823f0
- [ ] `LandDefs::GetBlockID` · 00488bc0 · legions 00482440

### CLandBlockMesh
- [ ] `CLandBlockMesh::GetLandCell` · 00488c10 · legions 00482490

### CLandBlock
- [ ] `CLandBlock::RecoverFromFreelist` · 00488c60 · legions 004824e0
- [ ] `CLandBlock::ClearForFreelist` · 00488d00 · legions 00482580
- [ ] `CLandBlock::GenerateMesh` · 00488d90 · legions 00482610
- [ ] `CLandBlock::GetNearestPixel` · 00488e10 · legions 00482690
- [ ] `CLandBlock::InitDynamicObjects` · 00488e70 · legions 004826f0
- [ ] `CLandBlock::ReleaseDynamicObjects` · 00488ec0 · legions 00482740
- [x] `CLandBlock::InqClearanceInfo` · 00488f10 · legions 00482790
- [ ] `CLandBlock::InitializeQuadtree` · 00488f70 · legions 004827f0
- [ ] `CLandBlock::ClearEncounterList` · 00489110 · legions 00482990
- [x] `CLandBlock::ObjectCollides` · 00489160 · legions 004829e0
- [ ] `CLandBlock::ProcFrill` · 004891c0 · legions 00482a40
- [~] `CLandBlock::CheckRadius` · 00489230 · legions 00482ab0 — ported; CLandCell::CheckRadius per-cell test not re-verified
- [ ] `CLandBlock::CullObjects` · 00489520 · legions 00482d70
- [ ] `CLandBlock::TraceRay` · 00489820 · legions 00483060
- [ ] `CLandBlock::InqObjects` · 00489a70 · legions 004832b0

### SmartArray<CLandBlock::RenderEntityInfo>
- [ ] `SmartArray<CLandBlock::RenderEntityInfo>::~SmartArray<CLandBlock::RenderEntityInfo>` · 00489b10 · legions 00483350

### DBObjGrabber<CWaterDesc>
- [ ] `DBObjGrabber<CWaterDesc>::DBObjGrabber<CWaterDesc>` · 00489b60 · legions 004833a0

### DBObjGrabber<BlockMap>
- [ ] `DBObjGrabber<BlockMap>::DBObjGrabber<BlockMap>` · 00489ba0 · legions 004833e0

### LandDefs
- [ ] `LandDefs::GetBlockID` · 00489d70 · legions 00483680

### EtherealWeenieType
- [ ] `EtherealWeenieType::EtherealWeenieType` · 00489de0 · legions 004836f0

### CLandBlock
- [ ] `CLandBlock::CLandBlock` · 00489e40 · legions 00483750
- [ ] `CLandBlock::DestroyStaticObjects` · 00489f70 · legions 00483880
- [ ] `CLandBlock::GetPhysicsMaterial` · 0048a060
- [ ] `CLandBlock::GetWaterPhysicsMaterial` · 0048a0e0 · legions 004839f0
- [x] `CLandBlock::CheckGroupPlacement` · 0048a160 · legions 00483a50
- [ ] `CLandBlock::RemoveDynamicObject` · 0048a330 · legions 00483c20

### SmartArray<AmbientSoundPoint>
- [ ] `SmartArray<AmbientSoundPoint>::AddToEnd` · 0048a390 · legions 00483c80

### CELLARRAY
- [ ] `CELLARRAY::CELLARRAY` · 0048a3f0 · legions 00483ce0

### CLandBlock
- [ ] `CLandBlock::Clear` · 0048a420 · legions 00483d10
- [x] `CLandBlock::InitCellConnections` · 0048a460 · legions 00483d50 — ac2-physics find_cells.rs, terrain.rs: cell search through land cells, touching cells and portals
- [ ] `CLandBlock::CreateAmbientSoundPoints` · 0048a620 · legions 00483f10
- [ ] `CLandBlock::DoIntersection` · 0048a7e0 · legions 00484090

### PSmartArray<InstanceID>
- [x] `PSmartArray<InstanceID>::StreamPack` · 0048a9c0

### CLandBlock
- [ ] `CLandBlock::~CLandBlock` · 0048ab10 · legions 004843c0
- [x] `CLandBlock::AddEntityToClearList` · 0048abb0 · legions 00484460
- [x] `CLandBlock::StartSceneGroup` · 0048ace0 · legions 00484590
- [x] `CLandBlock::ProcEntity` · 0048ad70 · legions 00484620
- [~] `CLandBlock::CreateStaticObjects` · 0048ae10 · legions 004846c0 — ac2-physics load_block_objects: colliding entity markers as static objects (no ObjectClearance/weenie filter, no CollisionParameters)
- [ ] `CLandBlock::AddDynamicObject` · 0048afe0 · legions 00484890
- [ ] `CLandBlock::AddToBrushList` · 0048b010 · legions 004848c0
- [ ] `CLandBlock::GetAmbientSoundPoints` · 0048b170 · legions 00484a20
- [ ] `CLandBlock::DoDetect` · 0048b190 · legions 00484a40

### LScapeManager
- [ ] `LScapeManager::GetLandBlock` · 0048b3d0 · legions 00484c80

### CLandBlock
- [ ] `CLandBlock::GetVisible` · 0048b430 · legions 00484ce0
- [x] `CLandBlock::GenerateLBIObjectList` · 0048b450 · legions 00484d00
- [x] `CLandBlock::ProcScene` · 0048b4c0 · legions 00484d70
- [x] `CLandBlock::ProcGroup` · 0048b700 · legions 00484f60
- [ ] `CLandBlock::RemoveLandCellsFromHash` · 0048b990 · legions 004851f0
- [ ] `CLandBlock::GatherObjects` · 0048ba30 · legions 00485290
- [ ] `CLandBlock::TraceRayCrossBlock` · 0048bcd0 · legions 00485500
- [ ] `CLandBlock::GenerateSceneObjectList` · 0048bfe0 · legions 00485810
- [ ] `CLandBlock::AddLandCellsToHash` · 0048c180 · legions 004859b0
- [ ] `CLandBlock::InitStaticObjects` · 0048c240 · legions 00485a60
- [ ] `CLandBlock::Regenerate` · 0048c2f0 · legions 00485b10
- [ ] `CLandBlock::Refresh` · 0048c320 · legions 00485b40

## `portal/shared/src/cell/landblockdata.cpp.c` (8/31)

### SurfaceInfo
- [ ] `SurfaceInfo::operator=` · 0048ee70 · legions 00487c90
- [x] `SurfaceInfo::SetToDefault` · 0048eef0 · legions 00487d10

### CLandBlockData
- [ ] `CLandBlockData::GetSubDataIDs` · 0048ef60

### SurfaceInfo
- [ ] `SurfaceInfo::SurfaceInfo` · 0048ef90

### CLandBlockData
- [ ] `CLandBlockData::SetHeight` · 0048efc0 · legions 00487de0
- [ ] `CLandBlockData::SetTerrain` · 0048efe0 · legions 00487e00
- [ ] `CLandBlockData::SetScene` · 0048f040 · legions 00487e60
- [ ] `CLandBlockData::SetEncounter` · 0048f0a0 · legions 00487ec0
- [ ] `CLandBlockData::SetRoad` · 0048f100 · legions 00487f20
- [x] `CLandBlockData::GetTerrain` · 0048f160 · legions 00487f80
- [ ] `CLandBlockData::GetScene` · 0048f1b0 · legions 00487fd0
- [x] `CLandBlockData::GetColor` · 0048f200 · legions 00488020
- [x] `CLandBlockData::GetRoad` · 0048f250 · legions 00488070
- [ ] `CLandBlockData::InqAsyncLoadDIDs` · 0048f2a0 · legions 004880c0
- [ ] `CLandBlockData::StreamPack` · 0048f310 · legions 00488130

### DBObjGrabber<CSurfaceDesc>
- [ ] `DBObjGrabber<CSurfaceDesc>::DBObjGrabber<CSurfaceDesc>` · 0048f3d0 · legions 004881f0

### CLandBlockData
- [ ] `CLandBlockData::~CLandBlockData` · 0048f560 · legions 00488380
- [ ] `CLandBlockData::GetDBOType` · 0048f600
- [x] `CLandBlockData::GetNeighborIsSWtoNECut` · 0048f610 · legions 00488430
- [x] `CLandBlockData::GetNeighborHeightValue` · 0048f720 · legions 00488540
- [x] `CLandBlockData::GetNormalFromTriangle` · 0048f830 · legions 00488650
- [x] `CLandBlockData::CachePackValues` · 0048f980 · legions 004887a0
- [ ] `CLandBlockData::InitLoad` · 0048fa90

### PSmartArray<unsigned_char>
- [ ] `PSmartArray<unsigned_char>::StreamPack` · 0048faa0

### (free functions)
- [ ] `STREAMPACK` · 0048fb40 · legions 00488960

### SmartArray<Vector>
- [ ] `SmartArray<Vector>::SetNElements` · 0048fbd0 · legions 004889f0

### CLandBlockData
- [ ] `CLandBlockData::CLandBlockData` · 0048fc00
- [~] `CLandBlockData::CacheFarValues` · 0048fe10 · legions 00488c30 — vertex normals and terrain SurfaceInfo; not far colours or water
- [ ] `CLandBlockData::Allocate` · 00490770
- [ ] `CLandBlockData::Allocator` · 00490780
- [~] `CLandBlockData::CacheNearValues` · 004907a0 · legions 00489500 — road SurfaceInfo; not physics materials or scene DIDs

## `portal/shared/src/cell/landblockinfo.cpp.c` (0/13)

### CLandBlockInfo
- [ ] `CLandBlockInfo::GetSubDataIDs` · 004913b0 · legions 0048a0b0
- [ ] `CLandBlockInfo::ApplyProperty` · 00491430 · legions 0048a130
- [ ] `CLandBlockInfo::CLandBlockInfo` · 00491530
- [ ] `CLandBlockInfo::GetDBOType` · 00491610
- [ ] `CLandBlockInfo::Destroy` · 00491620 · legions 0048a320
- [ ] `CLandBlockInfo::InqVisibleCells` · 004916d0 · legions 0048a3d0
- [ ] `CLandBlockInfo::StreamPack` · 00491720
- [ ] `CLandBlockInfo::ApplyPropertyCollection` · 00491810 · legions 0048a510

### EntityLinkDesc
- [ ] `EntityLinkDesc::operator=` · 004918c0 · legions 0048a5c0

### CLandBlockInfo
- [ ] `CLandBlockInfo::Allocate` · 00491a20
- [ ] `CLandBlockInfo::Allocator` · 00491a30
- [ ] `CLandBlockInfo::InitLoad` · 00491a50 · legions 0048a750
- [-] `CLandBlockInfo::FromFileNode` · 00491b10 — text source format (admin tools)

## `portal/shared/src/cell/landblockmesh.cpp.c` (4/15)

### CLandBlockMesh
- [ ] `CLandBlockMesh::CLandBlockMesh` · 0048c390 · legions 00485bb0
- [ ] `CLandBlockMesh::Clear` · 0048c440 · legions 00485c60
- [ ] `CLandBlockMesh::HasWater` · 0048c490 · legions 00485cb0
- [ ] `CLandBlockMesh::GetWaterIndex` · 0048c4a0 · legions 00485cc0
- [ ] `CLandBlockMesh::GetWaterPlane` · 0048c4b0 · legions 00485cd0
- [x] `CLandBlockMesh::InqWaterPlane` · 0048c4d0 · legions 00485cf0
- [ ] `CLandBlockMesh::SetVertexValueInternal` · 0048c510 · legions 00485d30
- [ ] `CLandBlockMesh::~CLandBlockMesh` · 0048c690 · legions 00485eb0
- [ ] `CLandBlockMesh::Init` · 0048c6b0 · legions 00485ed0
- [ ] `CLandBlockMesh::SetMinMax` · 0048c7d0 · legions 00485ff0
- [x] `CLandBlockMesh::ConstructVertices` · 0048ca50 · legions 00486270
- [x] `CLandBlockMesh::SetPolygon` · 0048cb40 · legions 00486360
- [ ] `CLandBlockMesh::Refresh` · 0048cbf0 · legions 00486410
- [x] `CLandBlockMesh::ConstructPolygons` · 0048cca0 · legions 004864c0
- [ ] `CLandBlockMesh::GenerateMesh` · 0048d0a0 · legions 004868c0

## `portal/shared/src/cell/landblockquadtree.cpp.c` (0/22)

### QuadTree<ICPhysicsObj_*>
- [ ] `QuadTree<ICPhysicsObj_*>::GetChildCenter` · 00491e60 · legions 0048a810
- [ ] `QuadTree<ICPhysicsObj_*>::DestroyTree` · 00491ef0 · legions 0048a8a0
- [ ] `QuadTree<ICPhysicsObj_*>::GetChildQuad` · 00491f50 · legions 0048a900

### CLandBlockQuadTree
- [ ] `CLandBlockQuadTree::UpdateMinMax` · 00491ff0 · legions 0048a9a0
- [ ] `CLandBlockQuadTree::DoCull` · 004920e0 · legions 0048aa90
- [ ] `CLandBlockQuadTree::Cull` · 004922b0 · legions 0048ac60

### QuadTree<ICPhysicsObj_*>
- [ ] `QuadTree<ICPhysicsObj_*>::Reset` · 004922e0 · legions 0048ac90
- [ ] `QuadTree<ICPhysicsObj_*>::~QuadTree<ICPhysicsObj_*>` · 00492310 · legions 0048acc0

### CLandBlockQuadTree
- [ ] `CLandBlockQuadTree::CLandBlockQuadTree` · 004923c0 · legions 0048ad70
- [ ] `CLandBlockQuadTree::~CLandBlockQuadTree` · 00492400 · legions 0048adb0

### QuadTree<ICPhysicsObj_*>::QuadNode
- [ ] `QuadTree<ICPhysicsObj_*>::QuadNode::~QuadNode` · 00492410 · legions 0048adc0

### SmartArray<_STL::pair<ICPhysicsObj*,BBox>_>
- [ ] `SmartArray<_STL::pair<ICPhysicsObj*,BBox>_>::operator=` · 00492590 · legions 0048af40

### CLandBlockQuadTree
- [ ] `CLandBlockQuadTree::CreateNode` · 00492650 · legions 0048b000
- [ ] `CLandBlockQuadTree::DoDraw` · 00492680 · legions 0048b030

### QuadTree<ICPhysicsObj_*>
- [ ] `QuadTree<ICPhysicsObj_*>::CreateNode` · 00492750 · legions 0048b100
- [ ] `QuadTree<ICPhysicsObj_*>::Initialize` · 00492780 · legions 0048b130

### SmartArray<_STL::pair<ICPhysicsObj_*,BBox>_>
- [ ] `SmartArray<_STL::pair<ICPhysicsObj_*,BBox>_>::AddToEnd` · 004927b0 · legions 0048b160

### QuadTree<ICPhysicsObj_*>
- [ ] `QuadTree<ICPhysicsObj_*>::DoGenerate` · 00492820 · legions 0048b1d0

### CLandBlockQuadTree
- [ ] `CLandBlockQuadTree::Draw` · 004929e0 · legions 0048b390

### QuadTree<ICPhysicsObj_*>
- [ ] `QuadTree<ICPhysicsObj_*>::AddObject` · 00492a00 · legions 0048b3b0
- [ ] `QuadTree<ICPhysicsObj_*>::Generate` · 00492a50 · legions 0048b400

### CLandBlockQuadTree
- [ ] `CLandBlockQuadTree::Generate` · 00492b60 · legions 0048b510

## `portal/shared/src/cell/landcell.cpp.c` (9/38)

### (free functions)
- [ ] `AreFloatEqual` · 0047d8e0 · legions 00477910

### Position
- [ ] `Position::adjust_to_outside` · 0047d900 · legions 00477930

### PositionOffset
- [x] `PositionOffset::GetOffset` · 0047d920 · legions 00489c10

### Ray
- [ ] `Ray::Ray` · 0047d990 · legions 00477950
- [ ] `Ray::InitFromOffset` · 0047d9c0 · legions 00477980
- [ ] `Ray::InitLineSegment` · 0047daa0 · legions 00477a60
- [ ] `Ray::GetEndPoint` · 0047dae0 · legions 00477aa0

### LOSParameters
- [ ] `LOSParameters::SetImpactNormal` · 0047db20 · legions 00477ae0

### Plane
- [x] `Plane::Plane` · 0047db50 · legions 00477b10 — ac2-physics shapes/sphere.rs (asm float order; Legions identical)

### SPHEREPATH
- [x] `SPHEREPATH::add_offset_to_check_pos` · 0047db90 · legions 00477b50

### COLLISIONINFO
- [x] `COLLISIONINFO::set_collision_normal` · 0047dbe0 · legions 00477ba0

### CLandCell
- [ ] `CLandCell::CLandCell` · 0047dc60 · legions 00477c20
- [ ] `CLandCell::~CLandCell` · 0047dca0 · legions 00477c60
- [ ] `CLandCell::SetPolygon` · 0047dcc0 · legions 00477c80
- [x] `CLandCell::InqWaterPlane` · 0047dce0
- [x] `CLandCell::find_terrain_poly` · 0047dd00 · legions 00477cc0
- [ ] `CLandCell::find_terrain_plane` · 0047dd60 · legions 00477d20
- [x] `CLandCell::point_in_cell` · 0047ddc0
- [ ] `CLandCell::GetViewerDistance` · 0047de50
- [ ] `CLandCell::GrabVisibleCells` · 0047dea0
- [ ] `CLandCell::ReleaseVisibleCells` · 0047deb0
- [ ] `CLandCell::UpdateCellMaterial` · 0047dec0
- [ ] `CLandCell::GetPhysicsMaterialDID` · 0047df20

### Ray
- [ ] `Ray::operator=` · 0047df40 · legions 00477f60

### CLandCell
- [ ] `CLandCell::Allocate` · 0047df80
- [ ] `CLandCell::CheckRadius` · 0047dfe0 · legions 00478000
- [x] `CLandCell::AddOutsideCell` · 0047e160 · legions 00478180 — ac2-physics find_cells.rs, terrain.rs: cell search through land cells, touching cells and portals
- [ ] `CLandCell::handle_move_restriction` · 0047e2c0 · legions 004782e0

### DBObjGrabber<CLandBlockData>
- [ ] `DBObjGrabber<CLandBlockData>::DBObjGrabber<CLandBlockData>` · 0047e400 · legions 00478420

### LOSParameters
- [ ] `LOSParameters::SetCurrentRay` · 0047e520 · legions 00478560

### CLandCell
- [x] `CLandCell::FindOutsideCells` · 0047e570 · legions 004785b0 — ac2-physics find_cells.rs, terrain.rs: cell search through land cells, touching cells and portals
- [~] `CLandCell::find_env_collisions` · 0047e670 — unloaded-landblock move restriction not ported
- [ ] `CLandCell::TraceRay` · 0047e7d0 · legions 00478810

### SmartArray<Plane>
- [ ] `SmartArray<Plane>::AddToEnd` · 0047ecb0 · legions 00478ce0

### CLandCell
- [ ] `CLandCell::GetCellPlanes` · 0047ed10

### LScapeManager
- [ ] `LScapeManager::GetLandCell` · 0047ee60 · legions 00478e90

### CLandCell
- [ ] `CLandCell::Get` · 0047eea0 · legions 00478ed0
- [ ] `CLandCell::Trace` · 0047eed0

## `portal/shared/src/cell/lightinfo.cpp.c` (0/26)

### PackObj
- [ ] `PackObj::UNPACK_TYPE` · 0048d160 · legions 00486970
- [ ] `PackObj::UnPack` · 0048d190 · legions 004869a0

### (free functions)
- [ ] `STREAMPACK_SIZEOF` · 0048d1c0 · legions 004869d0

### CLightInfo
- [ ] `CLightInfo::GetLitIndices` · 0048d220 · legions 00486b50
- [ ] `CLightInfo::GetSunLitIndices` · 0048d290 · legions 00486bc0

### CLightInfo::LightCellInfo
- [ ] `CLightInfo::LightCellInfo::LightCellInfo` · 0048d300
- [ ] `CLightInfo::LightCellInfo::~LightCellInfo` · 0048d310 · legions 00486c40

### RLE
- [ ] `RLE::Decode` · 0048d3d0 · legions 00486d00
- [ ] `RLE::Encode` · 0048d4c0 · legions 00486df0

### PFileNode
- [ ] `PFileNode::GetLeafAsUInt32` · 0048d660 · legions 00486f90

### SmartArray<short>
- [ ] `SmartArray<short>::SetNElements` · 0048d710 · legions 00487040

### CLightInfo
- [-] `CLightInfo::LoadFromFile` · 0048d740 — text source loader, admin build only
- [ ] `CLightInfo::StreamPackCell` · 0048d870 · legions 00487070
- [ ] `CLightInfo::Destroy` · 0048da40 · legions 00487240
- [ ] `CLightInfo::GetNumLights` · 0048db70 · legions 00487370
- [ ] `CLightInfo::GetLight` · 0048dbc0 · legions 004873c0
- [ ] `CLightInfo::StreamPack` · 0048dc30 · legions 00487430
- [ ] `CLightInfo::~CLightInfo` · 0048e110 · legions 00487910
- [ ] `CLightInfo::GetDBOType` · 0048e1c0
- [ ] `CLightInfo::CellInfoFromFileNode` · 0048e290

### PHashTable<unsigned_long,PSmartArray<unsigned_long>_>
- [ ] `PHashTable<unsigned_long,PSmartArray<unsigned_long>_>::StreamPack` · 0048e5d0

### CLightInfo
- [ ] `CLightInfo::CLightInfo` · 0048e7d0
- [ ] `CLightInfo::LightsFromFileNode` · 0048e840
- [ ] `CLightInfo::Allocate` · 0048ed60
- [ ] `CLightInfo::Allocator` · 0048ed70
- [-] `CLightInfo::FromFileNode` · 0048ed90 — text source format (admin tools)

## `portal/shared/src/cell/objcell.cpp.c` (10/53)

### DBObj
- [ ] `DBObj::SaveToFile` · 0047b860

### StreamDBObj
- [ ] `StreamDBObj::StreamPack` · 0047b870 · legions 00475960
- [ ] `StreamDBObj::GetPackSize` · 0047b8c0
- [ ] `StreamDBObj::Pack` · 0047b8f0
- [ ] `StreamDBObj::UnPack` · 0047b930

### Cylsphere
- [ ] `Cylsphere::Cylsphere` · 0047b950

### FrillEntry
- [ ] `FrillEntry::FrillEntry` · 0047b970

### CObjCell
- [ ] `CObjCell::Get` · 0047b9c0 · legions 00475af0
- [ ] `CObjCell::GetVisible` · 0047ba10 · legions 00475b40
- [ ] `CObjCell::SendObjectDetection` · 0047ba40 · legions 00475b70
- [ ] `CObjCell::SetObjectMaintainer` · 0047bb10 · legions 00475c40
- [ ] `CObjCell::GetViewerDistance` · 0047bb20
- [x] `CObjCell::FindCells` · 0047bb40 · legions 00475c70 — ac2-physics find_cells.rs, terrain.rs: cell search through land cells, touching cells and portals
- [x] `CObjCell::FindCellsLocal` · 0047bbd0 · legions 00475d00 — ac2-physics find_cells.rs, terrain.rs: cell search through land cells, touching cells and portals
- [x] `CObjCell::FindCellsLocal` · 0047bcb0 · legions 00475de0 — ac2-physics find_cells.rs, terrain.rs: cell search through land cells, touching cells and portals
- [x] `CObjCell::FindCells` · 0047bd90 · legions 00475ec0 — ac2-physics find_cells.rs, terrain.rs: cell search through land cells, touching cells and portals
- [ ] `CObjCell::LOSCheck` · 0047be70 · legions 00475fa0

### _STL::pair<ICPhysicsObj_*,BBox>
- [ ] `_STL::pair<ICPhysicsObj_*,BBox>::~pair<ICPhysicsObj_*,BBox>` · 0047bf50

### CObjCell
- [ ] `CObjCell::ClearForFreelist` · 0047c120 · legions 004761f0
- [ ] `CObjCell::MakeFrillList` · 0047c150 · legions 00476220
- [ ] `CObjCell::RemoveAllFrillObjects` · 0047c320 · legions 004763f0
- [ ] `CObjCell::RemoveShadowObject` · 0047c330 · legions 00476400
- [ ] `CObjCell::InitDynamicObjects` · 0047c390 · legions 00476460
- [ ] `CObjCell::DoIntersection` · 0047c3f0 · legions 004764c0
- [x] `CObjCell::FindCellsLocal` · 0047c440 · legions 00476510 — ac2-physics find_cells.rs, terrain.rs: cell search through land cells, touching cells and portals
- [x] `CObjCell::FindCellsLocal` · 0047c460 · legions 00476530 — ac2-physics find_cells.rs, terrain.rs: cell search through land cells, touching cells and portals
- [x] `CObjCell::find_obj_collisions` · 0047c510 · legions 004765e0 — ac2-physics objects.rs: static-object collision dispatch, containment radius, shadow box
- [ ] `CObjCell::TraceRayObjects` · 0047c560 · legions 00476630

### SmartArray<ObjectClearance>
- [ ] `SmartArray<ObjectClearance>::~SmartArray<ObjectClearance>` · 0047c600 · legions 004766c0

### IntrusiveHashTable<InstanceID,HashTableData<InstanceID,DetectionVoyeur>_*>
- [ ] `IntrusiveHashTable<InstanceID,HashTableData<InstanceID,DetectionVoyeur>_*>::~IntrusiveHashTable<InstanceID,HashTableData<InstanceID,DetectionVoyeur>_*>` · 0047c640 · legions 00476700

### DetectionReportType
- [ ] `DetectionReportType::StreamPack` · 0047c8c0

### CObjCell
- [ ] `CObjCell::CObjCell` · 0047c8f0 · legions 004769d0
- [ ] `CObjCell::GetCellID` · 0047c9d0 · legions 00476ac0
- [ ] `CObjCell::~CObjCell` · 0047c9e0 · legions 00476ad0
- [ ] `CObjCell::InvalidateShadowObjects` · 0047ca70 · legions 00476b60
- [ ] `CObjCell::ReleaseDynamicObjects` · 0047cb40 · legions 00476c30
- [x] `CObjCell::find_obj_collisions` · 0047cb70 · legions 00476c60 — ac2-physics objects.rs: static-object collision dispatch, containment radius, shadow box

### SmartArray<BBox>
- [ ] `SmartArray<BBox>::operator=` · 0047cbb0 · legions 00476ca0

### SmartArray<Cylsphere>
- [ ] `SmartArray<Cylsphere>::operator=` · 0047cc40 · legions 00476d30

### ObjectClearance
- [x] `ObjectClearance::operator=` · 0047cd90 · legions 00476e80

### CObjCell
- [x] `CObjCell::AddTouchingCell` · 0047ce60 · legions 00476f50 — ac2-physics find_cells.rs, terrain.rs: cell search through land cells, touching cells and portals
- [ ] `CObjCell::AddShadowObject` · 0047cfc0 · legions 00477050
- [ ] `CObjCell::UpdateAllVoyeur` · 0047d020 · legions 004770b0
- [ ] `CObjCell::HideObject` · 0047d0c0 · legions 00477150
- [ ] `CObjCell::UnhideObject` · 0047d0d0 · legions 00477160
- [ ] `CObjCell::AddObject` · 0047d170 · legions 004771a0
- [ ] `CObjCell::RemoveObject` · 0047d1f0 · legions 00477220
- [ ] `CObjCell::RemoveVoyeur` · 0047d260 · legions 00477290
- [ ] `CObjCell::ObjectWatchedBy` · 0047d330 · legions 00477360
- [ ] `CObjCell::InqObjects` · 0047d3e0 · legions 00477410
- [ ] `CObjCell::AddFrill` · 0047d500 · legions 00477530
- [ ] `CObjCell::InqVoyeurs` · 0047d5a0 · legions 004775d0
- [ ] `CObjCell::AddVoyeur` · 0047d7b0 · legions 004777e0

## `portal/shared/src/gametime/gametime.cpp.c` (4/29)

### GameTime
- [x] `GameTime::SetYearLength` · 00618860 · legions 005d50d0 — GameClock::new: day_length * days_per_year as a double

### WeekDay
- [ ] `WeekDay::WeekDay` · 00618880

### TimeOfDay
- [ ] `TimeOfDay::TimeOfDay` · 00618890

### GameTime
- [ ] `GameTime::GetTimeDilation` · 006188a0 · legions 005d5110

### Season
- [ ] `Season::Season` · 006188c0
- [ ] `Season::StreamPack` · 006188d0 · legions 005d5140

### GameTime
- [ ] `GameTime::Destroy` · 00618920 · legions 005d5190
- [ ] `GameTime::StreamPack` · 00618a40 · legions 005d52b0
- [ ] `GameTime::Init` · 00618b50 · legions 005d53c0
- [x] `GameTime::CalcDayBegin` · 00618bd0 · legions 005d5420 — ac2_world::GameClock (f64 intermediates; the year and day floors)
- [x] `GameTime::CalcTimeOfDay` · 00618c90 · legions 005d54e0 — ac2_world::GameClock (the last time of day runs to day_length)
- [ ] `GameTime::GetDateTimeString` · 00618d40 · legions 005d5590
- [ ] `GameTime::GetGameTimeFromTimeStamp` · 00618e40 · legions 005d5690
- [ ] `GameTime::InqGameDateTimeString` · 00618f30 · legions 005d5780
- [ ] `GameTime::GameTime` · 00618fb0 · legions 005d5800
- [ ] `GameTime::~GameTime` · 00619090 · legions 005d58e0
- [x] `GameTime::UseTime` · 00619130 · legions 005d5980 — ac2_world::GameClock::use_time
- [ ] `GameTime::InqCurrentGameDateTimeString` · 00619200 · legions 005d5a50
- [ ] `GameTime::TemporalDilationInternal` · 00619250 · legions 005d5aa0
- [ ] `GameTime::AlterTimeOfDayImmediate` · 00619320 · legions 005d5b70

### Season
- [-] `Season::FromFileNode` · 006193a0 — text source format (admin tools)

### WeekDay
- [-] `WeekDay::FromFileNode` · 00619540 — text source format (admin tools)

### TimeOfDay
- [-] `TimeOfDay::FromFileNode` · 006196a0 — text source format (admin tools)

### GameTime
- [ ] `GameTime::TemporalDilation` · 00619880 · legions 005d5bf0

### SmartArray<TimeOfDay>
- [ ] `SmartArray<TimeOfDay>::AddToEnd` · 006198a0

### GameTime
- [-] `GameTime::LoadSeason` · 00619900 — text source loader, admin build only
- [-] `GameTime::LoadWeek` · 00619980 — text source loader, admin build only
- [-] `GameTime::LoadHour` · 006199f0 — text source loader, admin build only
- [-] `GameTime::FromFileNode` · 00619a40 — text source format (admin tools)

## `portal/shared/src/places/placestable.cpp.c` (0/11)

### PlacesTable
- [ ] `PlacesTable::StreamPack` · 00638ee0 · legions 005eec60

### _STL::less<PStringBase<unsigned_short>_>
- [ ] `_STL::less<PStringBase<unsigned_short>_>::operator()` · 00638f40 · legions 005eecc0

### PlacesTable
- [ ] `PlacesTable::Dump` · 006395a0 · legions 005eecf0
- [ ] `PlacesTable::WslDump` · 00639690 · legions 005eede0
- [ ] `PlacesTable::GetPlace` · 00639850 · legions 005eee60
- [ ] `PlacesTable::InqPlace` · 006398d0 · legions 005eeee0
- [ ] `PlacesTable::InqPlaceStr` · 00639960 · legions 005eef40
- [ ] `PlacesTable::AddPlace` · 00639bb0
- [ ] `PlacesTable::InqPlacesHash` · 00639c40 · legions 005ef000
- [-] `PlacesTable::FromFileNode` · 00639cb0 — text source format (admin tools)
- [-] `PlacesTable::ToFileNode` · 00639f70 — text source format (admin tools)

## `portal/shared/src/region/blockmaps/blockmap.cpp.c` (0/8)

### BlockMap
- [ ] `BlockMap::UnPack` · 006cf150
- [ ] `BlockMap::Pack` · 006cf200
- [ ] `BlockMap::GetValue` · 006cf2c0 · legions 00658ee0
- [ ] `BlockMap::BlockMap` · 006cf300 · legions 00658f20
- [ ] `BlockMap::GetDBOType` · 006cf350
- [ ] `BlockMap::Destroy` · 006cf360 · legions 00658f80
- [ ] `BlockMap::Allocate` · 006cf400
- [ ] `BlockMap::~BlockMap` · 006cf410 · legions 00659030

## `portal/shared/src/region/encounter/encounterdesc.cpp.c` (0/16)

### CEncounterDesc
- [ ] `CEncounterDesc::Destroy` · 00520430
- [ ] `CEncounterDesc::StreamPack` · 00520440 · legions 0050b5b0
- [ ] `CEncounterDesc::GetSubDataIDs` · 005204b0 · legions 0050b620
- [ ] `CEncounterDesc::CEncounterDesc` · 00520500 · legions 0050b670
- [ ] `CEncounterDesc::GetDBOType` · 00520530
- [ ] `CEncounterDesc::~CEncounterDesc` · 00520540 · legions 0050b6b0
- [ ] `CEncounterDesc::Allocate` · 00520680
- [-] `CEncounterDesc::ToFileNode` · 005206b0 — text source format (admin tools)
- [-] `CEncounterDesc::LoadMap` · 005207c0 — text source loader, admin build only

### SmartArray<EncounterExcept>
- [ ] `SmartArray<EncounterExcept>::AddToEnd` · 00520900

### EncounterElem
- [ ] `EncounterElem::operator=` · 00520950 · legions 0050b820

### EncounterRow
- [ ] `EncounterRow::operator=` · 00520b40 · legions 0050ba10

### PSmartArray<EncounterRow>
- [ ] `PSmartArray<EncounterRow>::StreamPack` · 00520cf0 · legions 0050bbc0

### CEncounterDesc
- [-] `CEncounterDesc::CheckDupAdd` · 00520d90 — text source format (admin tools)
- [-] `CEncounterDesc::LoadRow` · 00520e10 — text source loader, admin build only
- [-] `CEncounterDesc::FromFileNode` · 00520ea0 — text source format (admin tools)

## `portal/shared/src/region/encounter/encounterelem.cpp.c` (0/13)

### EncounterElem
- [ ] `EncounterElem::StreamPack` · 006c6c70 · legions 00656470
- [ ] `EncounterElem::GetSubDataIDs` · 006c6d10 · legions 00656510
- [ ] `EncounterElem::EncounterElem` · 006c6d90
- [ ] `EncounterElem::~EncounterElem` · 006c6dd0 · legions 006565d0
- [ ] `EncounterElem::PutSubObject` · 006c6e60
- [-] `EncounterElem::LoadEncounter` · 006c6f10 — text source loader, admin build only

### PSmartArray<EncounterExcept>
- [ ] `PSmartArray<EncounterExcept>::StreamPack` · 006c7070 · legions 00656660

### EncounterElem
- [ ] `EncounterElem::PutTerrainException` · 006c7110
- [ ] `EncounterElem::PutWaterException` · 006c71f0
- [-] `EncounterElem::ToFileNode` · 006c72d0 — text source format (admin tools)
- [-] `EncounterElem::LoadTerrainExcept` · 006c7510 — text source loader, admin build only
- [-] `EncounterElem::LoadWaterExcept` · 006c7720 — text source loader, admin build only
- [-] `EncounterElem::FromFileNode` · 006c7930 — text source format (admin tools)

## `portal/shared/src/region/encounter/encounterexcept.cpp.c` (0/1)

### EncounterExcept
- [ ] `EncounterExcept::EncounterExcept` · 006c6c50

## `portal/shared/src/region/encounter/encounterrow.cpp.c` (0/10)

### EncounterRow
- [ ] `EncounterRow::GetSubDataIDs` · 006c7c10 · legions 00656700
- [ ] `EncounterRow::EncounterRow` · 006c7c50 · legions 00656740
- [ ] `EncounterRow::Destroy` · 006c7c90 · legions 00656780
- [ ] `EncounterRow::~EncounterRow` · 006c7ce0 · legions 006567d0
- [-] `EncounterRow::ToFileNode` · 006c7d40 — text source format (admin tools)
- [ ] `EncounterRow::StreamPack` · 006c7e40 · legions 00656830

### PSmartArray<EncounterElem>
- [ ] `PSmartArray<EncounterElem>::StreamPack` · 006c7ea0 · legions 00656890

### EncounterRow
- [-] `EncounterRow::CheckDupAdd` · 006c7f40 — text source format (admin tools)
- [-] `EncounterRow::LoadElement` · 006c7fc0 — text source loader, admin build only
- [-] `EncounterRow::FromFileNode` · 006c8020 — text source format (admin tools)

## `portal/shared/src/region/filenamedesc.cpp.c` (0/2)

### FileNameDesc
- [ ] `FileNameDesc::FileNameDesc` · 0052bd90
- [ ] `FileNameDesc::~FileNameDesc` · 0052be40 · legions 00510a30

## `portal/shared/src/region/fog/fogdesc.cpp.c` (0/15)

### CFogDesc
- [ ] `CFogDesc::CFogDesc` · 00524ce0 · legions 0050e1b0
- [ ] `CFogDesc::GetDBOType` · 00524d20
- [ ] `CFogDesc::Destroy` · 00524d30 · legions 0050e200
- [ ] `CFogDesc::Pack` · 00524db0
- [ ] `CFogDesc::GetPackSize` · 00524f10 · legions 0050e3e0
- [ ] `CFogDesc::GetSubDataIDs` · 00524f70 · legions 0050e440
- [ ] `CFogDesc::FindFogType` · 00524fe0 · legions 0050e4b0
- [ ] `CFogDesc::HasFog` · 00525030 · legions 0050e500
- [ ] `CFogDesc::GetFogging` · 00525050 · legions 0050e520
- [ ] `CFogDesc::Allocate` · 00525080
- [ ] `CFogDesc::UnPack` · 005250d0
- [-] `CFogDesc::ToFileNode` · 00525280 — text source format (admin tools)
- [-] `CFogDesc::CheckDupAdd` · 005254d0 — text source format (admin tools)
- [-] `CFogDesc::LoadFog` · 00525540 — text source loader, admin build only
- [-] `CFogDesc::FromFileNode` · 005255e0 — text source format (admin tools)

## `portal/shared/src/region/fog/fogtype.cpp.c` (0/8)

### FogType
- [ ] `FogType::FogType` · 006ca330 · legions 006570a0
- [ ] `FogType::unpack` · 006ca350 · legions 006570c0
- [ ] `FogType::GetPackSize` · 006ca3a0 · legions 006580a0
- [ ] `FogType::HasFog` · 006ca3b0 · legions 00657110
- [ ] `FogType::GetFogging` · 006ca3c0 · legions 00657120
- [ ] `FogType::~FogType` · 006ca3e0 · legions 00657140
- [-] `FogType::FromFileNode` · 006ca410 — text source format (admin tools)
- [-] `FogType::ToFileNode` · 006ca590 — text source format (admin tools)

## `portal/shared/src/region/fog/lightfoginfo.cpp.c` (0/19)

### LightInfo
- [ ] `LightInfo::Destroy` · 006ca670 · legions 00657170

### LightFogInfo
- [ ] `LightFogInfo::LightFogInfo` · 006ca6f0 · legions 006571f0
- [ ] `LightFogInfo::Destroy` · 006ca790 · legions 00657290
- [ ] `LightFogInfo::GetCodePart` · 006ca810 · legions 00657310
- [ ] `LightFogInfo::FindCodeType` · 006ca870 · legions 00657370
- [ ] `LightFogInfo::SetCombine` · 006ca8c0 · legions 006573c0
- [ ] `LightFogInfo::~LightFogInfo` · 006ca940 · legions 00657440
- [ ] `LightFogInfo::unpack` · 006ca950 · legions 00657450
- [ ] `LightFogInfo::GetCode` · 006cabd0 · legions 006576d0
- [ ] `LightFogInfo::pack` · 006cac30 · legions 00657730
- [ ] `LightFogInfo::GetPackSize` · 006caea0 · legions 006579a0
- [-] `LightFogInfo::LoadFog` · 006caf10 — text source loader, admin build only
- [-] `LightFogInfo::LoadDirectLight` · 006cb290 — text source loader, admin build only
- [-] `LightFogInfo::LoadAmbientLight` · 006cb430 — text source loader, admin build only
- [-] `LightFogInfo::LoadShadowColor` · 006cb5d0 — text source loader, admin build only
- [-] `LightFogInfo::LoadCombine` · 006cb700 — text source loader, admin build only
- [ ] `LightFogInfo::SaveCombine` · 006cb930
- [-] `LightFogInfo::FromFileNode` · 006cba90 — text source format (admin tools)
- [-] `LightFogInfo::ToFileNode` · 006cbcc0 — text source format (admin tools)

## `portal/shared/src/region/properties/blockproperty.cpp.c` (0/16)

### BlockProperty
- [ ] `BlockProperty::GetPropertyType` · 006cd220 · legions 00658100
- [ ] `BlockProperty::StreamPack` · 006cd240 · legions 00658120
- [ ] `BlockProperty::GetProperty` · 006cd2b0 · legions 00658190
- [ ] `BlockProperty::GetInteger` · 006cd2e0 · legions 006581c0
- [ ] `BlockProperty::GetFloat` · 006cd320 · legions 00658240
- [ ] `BlockProperty::GetVector` · 006cd360 · legions 00658280
- [ ] `BlockProperty::GetColor` · 006cd3a0 · legions 006582c0
- [ ] `BlockProperty::GetEnum` · 006cd3e0
- [ ] `BlockProperty::BlockProperty` · 006cd420
- [ ] `BlockProperty::~BlockProperty` · 006cd450 · legions 00658330
- [-] `BlockProperty::LoadName` · 006cd4a0 — text source loader, admin build only
- [ ] `BlockProperty::CheckGrow` · 006cd580
- [-] `BlockProperty::LoadBlockMap` · 006cd640 — text source loader, admin build only
- [-] `BlockProperty::LoadValue` · 006cd710 — text source loader, admin build only
- [-] `BlockProperty::LoadLinear` · 006cd950 — text source loader, admin build only
- [ ] `BlockProperty::Load` · 006ce070

## `portal/shared/src/region/properties/propertydesc.cpp.c` (0/18)

### PropertyDesc
- [ ] `PropertyDesc::IntegerCornerInterp` · 006ce4a0 · legions 00658380
- [ ] `PropertyDesc::FloatCornerInterp` · 006ce4f0 · legions 006583d0
- [ ] `PropertyDesc::VectorCornerInterp` · 006ce520 · legions 00658400
- [ ] `PropertyDesc::ColorCornerInterp` · 006ce5b0 · legions 00658490
- [ ] `PropertyDesc::GetSubDataIDs` · 006ce660 · legions 00658540
- [ ] `PropertyDesc::PropertyDesc` · 006ce6a0 · legions 00658580
- [ ] `PropertyDesc::GetDBOType` · 006ce6d0
- [ ] `PropertyDesc::~PropertyDesc` · 006ce6e0 · legions 006585c0
- [ ] `PropertyDesc::GetBlockPropertyInternal` · 006ce760 · legions 00658640
- [ ] `PropertyDesc::GetPassiblity` · 006ceb80 · legions 00658a20
- [ ] `PropertyDesc::Allocate` · 006cec80
- [ ] `PropertyDesc::UpdateBlockMaterial` · 006cecb0 · legions 00658b30

### BlockProperty
- [ ] `BlockProperty::operator=` · 006ced20 · legions 00658ba0

### PSmartArray<BlockProperty>
- [ ] `PSmartArray<BlockProperty>::StreamPack` · 006cee50 · legions 00658cd0

### SmartArray<BlockProperty>
- [ ] `SmartArray<BlockProperty>::AddToEnd` · 006ceef0

### PropertyDesc
- [-] `PropertyDesc::CheckDupAdd` · 006cef60 — text source format (admin tools)
- [-] `PropertyDesc::LoadProperty` · 006cefb0 — text source loader, admin build only
- [-] `PropertyDesc::FromFileNode` · 006cf040 — text source format (admin tools)

## `portal/shared/src/region/regiondesc.cpp.c` (0/47)

### PackObj
- [ ] `PackObj::ALIGN_PTR` · 0051d6e0 · legions 0050a0e0

### CRegionDesc
- [ ] `CRegionDesc::SetLandDefs` · 0051d730 · legions 0050a130
- [ ] `CRegionDesc::RemoveCurrentRegion` · 0051d7a0 · legions 0050a1a0
- [-] `CRegionDesc::LoadHeader` · 0051d800 — text source loader, admin build only
- [ ] `CRegionDesc::GetGameMap` · 0051d850 · legions 0050a200
- [ ] `CRegionDesc::Destroy` · 0051d920 · legions 0050a2d0
- [ ] `CRegionDesc::PackInfoList` · 0051db20 · legions 0050a4d0
- [ ] `CRegionDesc::GetSubDataIDs` · 0051dbd0 · legions 0050a580
- [ ] `CRegionDesc::GetCurrentSoundInfo` · 0051de10 · legions 0050a7c0
- [ ] `CRegionDesc::GetCurrentSceneInfo` · 0051de40 · legions 0050a7e0
- [ ] `CRegionDesc::GetCurrentTerrainInfo` · 0051de70 · legions 0050a810
- [ ] `CRegionDesc::GetCurrentSurfaceInfo` · 0051deb0 · legions 0050a840
- [ ] `CRegionDesc::GetCurrentDetailMapTable` · 0051def0 · legions 0050a870
- [ ] `CRegionDesc::GetCurrentWaterInfo` · 0051df30 · legions 0050a8a0
- [ ] `CRegionDesc::GetCurrentFogInfo` · 0051df70 · legions 0050a8d0
- [ ] `CRegionDesc::GetCurrentPropertyInfo` · 0051dfb0 · legions 0050a900

### PFileNode
- [ ] `PFileNode::GetLeafAsFloat` · 0051dff0 · legions 0050a930

### CRegionDesc
- [ ] `CRegionDesc::CRegionDesc` · 0051e010 · legions 0050a950
- [ ] `CRegionDesc::GetDBOType` · 0051e150
- [ ] `CRegionDesc::~CRegionDesc` · 0051e160 · legions 0050aaa0
- [ ] `CRegionDesc::GetPackSize` · 0051e290 · legions 0050abd0
- [ ] `CRegionDesc::SetGameTime` · 0051e480 · legions 0050adc0
- [ ] `CRegionDesc::GetCurrentSkyInfo` · 0051e540
- [ ] `CRegionDesc::Allocate` · 0051e580
- [ ] `CRegionDesc::UnPackInfoList` · 0051e5b0 · legions 0050aee0
- [ ] `CRegionDesc::SetRegionByGID` · 0051e6c0
- [ ] `CRegionDesc::SetRegion` · 0051e7a0
- [-] `CRegionDesc::LoadCellMaps` · 0051e800 — text source loader, admin build only
- [-] `CRegionDesc::LoadGameMap` · 0051ec30 — text source loader, admin build only
- [-] `CRegionDesc::LoadFile` · 0051eeb0 — text source loader, admin build only
- [-] `CRegionDesc::CheckDupAdd` · 0051ef90 — text source format (admin tools)
- [ ] `CRegionDesc::Pack` · 0051f030
- [ ] `CRegionDesc::UnPack` · 0051f260
- [-] `CRegionDesc::LoadGeneral` · 0051f4f0 — text source loader, admin build only
- [-] `CRegionDesc::LoadNameAndFile` · 0051f9e0 — text source loader, admin build only
- [-] `CRegionDesc::LoadSoundInfo` · 0051fc60 — text source loader, admin build only
- [-] `CRegionDesc::LoadSceneInfo` · 0051fcb0 — text source loader, admin build only
- [-] `CRegionDesc::LoadTerrainInfo` · 0051fd00 — text source loader, admin build only
- [-] `CRegionDesc::LoadSurfaceInfo` · 0051fd50 — text source loader, admin build only
- [-] `CRegionDesc::LoadEncounterTable` · 0051fda0 — text source loader, admin build only
- [-] `CRegionDesc::LoadDetailMapTable` · 0051fdf0 — text source loader, admin build only
- [-] `CRegionDesc::LoadSkyInfo` · 0051fe40 — text source loader, admin build only
- [-] `CRegionDesc::LoadWaterInfo` · 0051fe90 — text source loader, admin build only
- [-] `CRegionDesc::LoadFogInfo` · 0051fee0 — text source loader, admin build only
- [-] `CRegionDesc::LoadPropertyInfo` · 0051ff30 — text source loader, admin build only
- [-] `CRegionDesc::LoadWorldMap` · 0051ff80 — text source loader, admin build only
- [-] `CRegionDesc::FromFileNode` · 0051ffd0 — text source format (admin tools)

## `portal/shared/src/region/regionemap.cpp.c` (0/4)

### RegionEmap
- [ ] `RegionEmap::GetSceneEnum` · 006bc6f0
- [ ] `RegionEmap::GetSurfaceEnum` · 006bc770
- [ ] `RegionEmap::GetSkyObjectEnum` · 006bc7f0
- [ ] `RegionEmap::GetSkyObjectName` · 006bc870

## `portal/shared/src/region/scene/lscapescene.cpp.c` (0/8)

### LScapeScene
- [ ] `LScapeScene::LScapeScene` · 006c32e0 · legions 00654960
- [ ] `LScapeScene::pack` · 006c3300 · legions 00654980
- [ ] `LScapeScene::unpack` · 006c3360 · legions 006549e0

### FileNodeName_Int64
- [ ] `FileNodeName_Int64::GetType` · 006c33b0 · legions 00654a30

### LScapeScene
- [ ] `LScapeScene::GetSubDataIDs` · 006c33c0 · legions 00654a40
- [ ] `LScapeScene::~LScapeScene` · 006c33e0 · legions 00654a60
- [-] `LScapeScene::LoadSceneFile` · 006c3400 — text source loader, admin build only
- [ ] `LScapeScene::Load` · 006c34d0

## `portal/shared/src/region/scene/objectclearance.cpp.c` (17/17)

### ObjectClearance
- [x] `ObjectClearance::InitAsEntity` · 006bc900 · legions 00651690
- [x] `ObjectClearance::InitAsGroup` · 006bc940 · legions 006516d0
- [x] `ObjectClearance::IntersectsCyl` · 006bc960 · legions 006516f0
- [x] `ObjectClearance::SetEnd` · 006bca20 · legions 006517b0
- [x] `ObjectClearance::InSuperGroup` · 006bca40 · legions 006517d0
- [x] `ObjectClearance::SetClearance` · 006bca50 · legions 006517e0
- [x] `ObjectClearance::ObjectClearance` · 006bca80 · legions 00651810
- [x] `ObjectClearance::~ObjectClearance` · 006bcaf0 · legions 00651880
- [x] `ObjectClearance::IntersectsBox` · 006bcb50 · legions 006518e0
- [x] `ObjectClearance::CheckIntersection` · 006bccb0 · legions 00651a40
- [x] `ObjectClearance::InitAsEntity` · 006bce10 · legions 00651ba0
- [x] `ObjectClearance::SetClearBBoxMin` · 006bce60 · legions 00651bf0
- [x] `ObjectClearance::SetClearBBoxMax` · 006bcf20 · legions 00651cb0
- [x] `ObjectClearance::SetClearCylCenter` · 006bcfe0 · legions 00651d70
- [x] `ObjectClearance::SetClearCylHeight` · 006bd0c0 · legions 00651e50
- [x] `ObjectClearance::SetClearCylRadius` · 006bd190 · legions 00651f20
- [x] `ObjectClearance::ApplyClearanceProperty` · 006bd260 · legions 00651ff0

## `portal/shared/src/region/scene/scenedegrade.cpp.c` (0/10)

### SceneDegrade
- [ ] `SceneDegrade::GetDegradeDist` · 006c2ba0 · legions 00654630
- [ ] `SceneDegrade::SceneDegrade` · 006c2be0 · legions 00654670
- [ ] `SceneDegrade::Destroy` · 006c2c00 · legions 00654690
- [ ] `SceneDegrade::GetSubDataIDs` · 006c2c70 · legions 00654700
- [ ] `SceneDegrade::~SceneDegrade` · 006c2cb0 · legions 00654740
- [ ] `SceneDegrade::CopyInto` · 006c2cd0 · legions 00654760
- [ ] `SceneDegrade::StreamPack` · 006c2d80 · legions 00654810
- [-] `SceneDegrade::ToFileNode` · 006c2ed0 — text source format (admin tools)
- [-] `SceneDegrade::LoadEntity` · 006c2ff0 — text source loader, admin build only
- [-] `SceneDegrade::FromFileNode` · 006c3070 — text source format (admin tools)

## `portal/shared/src/region/scene/scenedesc.cpp.c` (0/14)

### CSceneDesc
- [ ] `CSceneDesc::CSceneDesc` · 00523b20 · legions 0050d8a0
- [ ] `CSceneDesc::GetDBOType` · 00523b50
- [ ] `CSceneDesc::Destroy` · 00523b60 · legions 0050d8e0
- [ ] `CSceneDesc::GetPackSize` · 00523bb0 · legions 0050d930
- [ ] `CSceneDesc::Pack` · 00523bf0
- [ ] `CSceneDesc::GetSubDataIDs` · 00523cb0 · legions 0050da30
- [ ] `CSceneDesc::FindSceneType` · 00523cf0 · legions 0050da70
- [ ] `CSceneDesc::GetSceneID` · 00523d30 · legions 0050dab0
- [ ] `CSceneDesc::GetSoundType` · 00523d70 · legions 0050dad0
- [ ] `CSceneDesc::Allocate` · 00523dc0
- [ ] `CSceneDesc::UnPack` · 00523e10
- [-] `CSceneDesc::CheckDupAdd` · 00523ef0 — text source format (admin tools)
- [-] `CSceneDesc::LoadSceneType` · 00523f60 — text source loader, admin build only
- [-] `CSceneDesc::FromFileNode` · 00523fe0 — text source format (admin tools)

## `portal/shared/src/region/scene/sceneentity.cpp.c` (0/10)

### SceneEntity
- [ ] `SceneEntity::CopyInto` · 006c17d0 · legions 00653ce0
- [ ] `SceneEntity::GetSubDataIDs` · 006c1850 · legions 00653d50
- [ ] `SceneEntity::GetDataID` · 006c1890 · legions 00653d80
- [ ] `SceneEntity::GetSubGroupPtr` · 006c18b0 · legions 00653da0
- [ ] `SceneEntity::StreamPack` · 006c18c0 · legions 00653db0
- [ ] `SceneEntity::~SceneEntity` · 006c1970 · legions 00653e50
- [ ] `SceneEntity::PutSubObject` · 006c19c0
- [ ] `SceneEntity::GetSubObject` · 006c1a90
- [-] `SceneEntity::ToFileNode` · 006c1be0 — text source format (admin tools)
- [-] `SceneEntity::FromFileNode` · 006c1c90 — text source format (admin tools)

## `portal/shared/src/region/scene/scenefile.cpp.c` (0/12)

### SceneFile
- [ ] `SceneFile::SceneFile` · 006c1d10 · legions 00653e90

### AMLook
- [ ] `AMLook::GetType` · 006c1d40

### SceneFile
- [ ] `SceneFile::Destroy` · 006c1d50 · legions 00653ed0
- [ ] `SceneFile::GetSubDataIDs` · 006c1dc0 · legions 00653f40
- [ ] `SceneFile::GetCorrectDegrade` · 006c1e10 · legions 00653f90
- [ ] `SceneFile::Allocate` · 006c1e60
- [ ] `SceneFile::CopyInto` · 006c1eb0
- [ ] `SceneFile::StreamPack` · 006c1fa0
- [-] `SceneFile::ToFileNode` · 006c2150 — text source format (admin tools)
- [-] `SceneFile::LoadSceneDegrade` · 006c2250 — text source loader, admin build only
- [-] `SceneFile::LoadClientFrills` · 006c22d0 — text source loader, admin build only
- [-] `SceneFile::FromFileNode` · 006c2350 — text source format (admin tools)

## `portal/shared/src/region/scene/scenegroup.cpp.c` (13/45)

### SceneGroup
- [ ] `SceneGroup::SetMinRot` · 006bd400
- [ ] `SceneGroup::SetMaxRot` · 006bd430
- [ ] `SceneGroup::SetFrequency` · 006bd460
- [ ] `SceneGroup::SetMaxAboveDryLand` · 006bd490
- [ ] `SceneGroup::SetMinAboveDryLand` · 006bd4c0
- [ ] `SceneGroup::SetMaxAboveWater` · 006bd4f0
- [ ] `SceneGroup::SetMinAboveWater` · 006bd520
- [ ] `SceneGroup::SetMaxBelowWater` · 006bd550
- [ ] `SceneGroup::SetMinBelowWater` · 006bd580
- [ ] `SceneGroup::SetMinWaterDepth` · 006bd5b0
- [ ] `SceneGroup::SetMaxWaterDepth` · 006bd5e0
- [ ] `SceneGroup::SetMaxBelowGround` · 006bd610
- [ ] `SceneGroup::SetMinBelowGround` · 006bd640
- [ ] `SceneGroup::SetDayFreq` · 006bd670
- [ ] `SceneGroup::SetMaxSlope` · 006bd6a0
- [ ] `SceneGroup::SetMinSlope` · 006bd6e0
- [ ] `SceneGroup::SetClearanceRadius` · 006bd720
- [ ] `SceneGroup::SetMaxSlopeRadius` · 006bd750
- [ ] `SceneGroup::SetMinSlopeRadius` · 006bd7a0
- [x] `SceneGroup::CheckFreq` · 006bd7f0 · legions 00652190
- [x] `SceneGroup::GetScale` · 006bd870 · legions 00652210
- [x] `SceneGroup::HaveMaxObjects` · 006bdc00 · legions 006525a0
- [x] `SceneGroup::UpdateZLoc` · 006bdc20 · legions 006525c0
- [x] `SceneGroup::GetCheckRadius` · 006bdcb0 · legions 00652640
- [x] `SceneGroup::ScaleByLandSlope` · 006bdcf0 · legions 00652680
- [x] `SceneGroup::GetClearanceRadius` · 006bdda0 · legions 00652730
- [x] `SceneGroup::CheckSlope` · 006bde20 · legions 006527b0
- [x] `SceneGroup::GetAlignedLimits` · 006bde50 · legions 006527e0
- [ ] `SceneGroup::SceneGroup` · 006bded0 · legions 00652860
- [ ] `SceneGroup::Destroy` · 006be020 · legions 006529b0
- [ ] `SceneGroup::GetSubDataIDs` · 006be1b0 · legions 00652b40
- [x] `SceneGroup::UpdateXYLoc` · 006be200 · legions 00652b90
- [~] `SceneGroup::UpdateFrame` · 006be3c0 · legions 00652d50 — heading, orient and rot-sub paths; align (Plane::OrientFrameToPlane) not re-verified
- [x] `SceneGroup::ScaleByWaterDepth` · 006be680 · legions 00653000
- [x] `SceneGroup::CheckFinalHeight` · 006be780 · legions 00653100
- [ ] `SceneGroup::~SceneGroup` · 006be9f0 · legions 00653380
- [x] `SceneGroup::FinishPlaceAndCheck` · 006bea10 · legions 006533a0
- [ ] `SceneGroup::CopyInto` · 006beb80 · legions 00653510
- [ ] `SceneGroup::StreamPack` · 006bee00 · legions 00653790
- [ ] `SceneGroup::GetCorner` · 006bf350
- [ ] `SceneGroup::PutRadiusInfo` · 006bf3e0
- [ ] `SceneGroup::GetRadiusInfo` · 006bf560
- [-] `SceneGroup::LoadEntity` · 006bf840 — text source loader, admin build only
- [-] `SceneGroup::ToFileNode` · 006bf8c0 — text source format (admin tools)
- [-] `SceneGroup::FromFileNode` · 006c03f0 — text source format (admin tools)

## `portal/shared/src/region/scene/scenetype.cpp.c` (0/12)

### SceneType
- [ ] `SceneType::Destroy` · 006c24b0 · legions 006542d0
- [ ] `SceneType::pack` · 006c2500 · legions 00654320
- [ ] `SceneType::pack_size` · 006c2590 · legions 006543b0
- [ ] `SceneType::GetSubDataIDs` · 006c25d0 · legions 006543f0
- [ ] `SceneType::GetScene` · 006c2610 · legions 00654430
- [ ] `SceneType::SceneType` · 006c2690 · legions 006544b0
- [ ] `SceneType::~SceneType` · 006c26c0 · legions 006544e0
- [ ] `SceneType::unpack` · 006c2700 · legions 00654520
- [-] `SceneType::LoadName` · 006c2810 — text source loader, admin build only
- [-] `SceneType::LoadSound` · 006c2880 — text source loader, admin build only
- [-] `SceneType::LoadLScapeScene` · 006c2970 — text source loader, admin build only
- [ ] `SceneType::Load` · 006c29f0

## `portal/shared/src/region/sky/daydesc.cpp.c` (10/38)

### CelestialPosition
- [x] `CelestialPosition::CelestialPosition` · 005257a0 · legions 0050e750 — ac2-world region::sky_objects, ac2-render sky

### CDayDesc
- [x] `CDayDesc::GetTimeRatio` · 00525800 · legions 0050e7b0
- [x] `CDayDesc::InterpAngleClosest` · 005258b0 · legions 0050e860
- [x] `CDayDesc::InterpColor` · 00525950 · legions 0050e900
- [x] `CDayDesc::CalcLightVector` · 005259e0 · legions 0050e990
- [ ] `CDayDesc::Destroy` · 00525a90 · legions 0050ea40
- [ ] `CDayDesc::StreamPack` · 00525af0 · legions 0050eaa0
- [ ] `CDayDesc::GetSubDataIDs` · 00525b60
- [-] `CDayDesc::CheckDupAdd` · 00525bd0 — text source format (admin tools)
- [ ] `CDayDesc::SetInterpDir` · 00525c10
- [x] `CDayDesc::GetBeforeTime` · 00525cb0 · legions 0050eb80
- [x] `CDayDesc::GetAfterTime` · 00525d30 · legions 0050ec00
- [x] `CDayDesc::GetLighting` · 00525d90 · legions 0050ec60
- [x] `CDayDesc::GetWorldFog` · 00525f70 · legions 0050ee40
- [ ] `CDayDesc::GetRain` · 00526060 · legions 0050ef30
- [ ] `CDayDesc::CDayDesc` · 005262f0 · legions 0050f1c0
- [ ] `CDayDesc::GetDBOType` · 00526330
- [ ] `CDayDesc::~CDayDesc` · 00526340 · legions 0050f210

### PSmartArray<SkyTimeOfDay>
- [ ] `PSmartArray<SkyTimeOfDay>::StreamPack` · 005263b0 · legions 0050f280

### SkyObjectReplace
- [ ] `SkyObjectReplace::operator=` · 00526460 · legions 0050f330

### SmartArray<LensFlarePoint>
- [ ] `SmartArray<LensFlarePoint>::operator=` · 00526590 · legions 0050f460

### CDayDesc
- [ ] `CDayDesc::Allocate` · 00526600
- [-] `CDayDesc::ToFileNode` · 00526630 — text source format (admin tools)

### SmartArray<SkyTimeOfDay>
- [ ] `SmartArray<SkyTimeOfDay>::AddToEnd` · 00526820

### CDayDesc
- [-] `CDayDesc::LoadTimeOfDay` · 00526870 — text source loader, admin build only

### SmartArray<SkyObjectReplace>
- [ ] `SmartArray<SkyObjectReplace>::AddToEnd` · 005268f0

### SkyObject
- [ ] `SkyObject::operator=` · 00526980 · legions 0050f500

### CelestialPosition
- [ ] `CelestialPosition::operator=` · 00526a20 · legions 0050f5a0

### CDayDesc
- [-] `CDayDesc::LoadAllTimes` · 00526ba0 — text source loader, admin build only
- [ ] `CDayDesc::SetInterpDiffuse` · 00526c80
- [ ] `CDayDesc::SetInterpOpacity` · 00526d40

### SmartArray<SkyObject>
- [ ] `SmartArray<SkyObject>::RemoveOrderedByIndex` · 00526dd0

### PSmartArray<SkyObject>
- [ ] `PSmartArray<SkyObject>::StreamPack` · 00526e20 · legions 0050f720

### CDayDesc
- [ ] `CDayDesc::AddMissingEntries` · 00526fb0
- [-] `CDayDesc::LoadObject` · 00528780 — text source loader, admin build only
- [-] `CDayDesc::LoadSkyObjects` · 00528820 — text source loader, admin build only
- [x] `CDayDesc::GetSky` · 00528900 · legions 0050f8b0 — ac2-world region::sky_objects, ac2-render sky
- [-] `CDayDesc::FromFileNode` · 00528d10 — text source format (admin tools)

## `portal/shared/src/region/sky/skyday.cpp.c` (0/5)

### DecalFX
- [ ] `DecalFX::GetSubDataIDs` · 006c8960 · legions 00643980

### SkyDay
- [ ] `SkyDay::SkyDay` · 006c8980
- [ ] `SkyDay::~SkyDay` · 006c89a0 · legions 00656a20
- [-] `SkyDay::LoadName` · 006c89c0 — text source loader, admin build only
- [-] `SkyDay::FromFileNode` · 006c8aa0 — text source format (admin tools)

## `portal/shared/src/region/sky/skydesc.cpp.c` (1/17)

### CSkyDesc
- [ ] `CSkyDesc::GetTickSize` · 00524140 · legions 0050dc30
- [ ] `CSkyDesc::GetLightTickSize` · 00524180 · legions 0050dc70
- [ ] `CSkyDesc::Destroy` · 005241c0 · legions 0050dcb0
- [ ] `CSkyDesc::GetSubDataIDs` · 00524220 · legions 0050dd10
- [ ] `CSkyDesc::GetDayByIndex` · 00524270 · legions 0050dd60
- [x] `CSkyDesc::GetCurrentDay` · 00524290 · legions 0050dd80
- [ ] `CSkyDesc::RecalcFreq` · 00524350 · legions 0050de40
- [ ] `CSkyDesc::CSkyDesc` · 005243d0 · legions 0050de70
- [ ] `CSkyDesc::GetDBOType` · 00524430
- [ ] `CSkyDesc::StreamPack` · 00524440 · legions 0050dee0
- [ ] `CSkyDesc::Allocate` · 00524600
- [-] `CSkyDesc::LoadGameTime` · 00524670 — text source loader, admin build only

### PSmartArray<SkyDay>
- [ ] `PSmartArray<SkyDay>::StreamPack` · 00524740 · legions 0050e110

### CSkyDesc
- [-] `CSkyDesc::CheckDupAdd` · 005247e0 — text source format (admin tools)
- [-] `CSkyDesc::LoadDay` · 00524860 — text source loader, admin build only
- [-] `CSkyDesc::LoadWeatherInfo` · 005248f0 — text source loader, admin build only
- [-] `CSkyDesc::FromFileNode` · 005249d0 — text source format (admin tools)

## `portal/shared/src/region/sky/skyobject.cpp.c` (1/8)

### SkyObject
- [ ] `SkyObject::Destroy` · 0052b0b0 · legions 00510790
- [ ] `SkyObject::GetSubDataIDs` · 0052b0f0 · legions 005107d0
- [ ] `SkyObject::SkyObject` · 0052b120
- [ ] `SkyObject::~SkyObject` · 0052b170 · legions 00510850
- [x] `SkyObject::StreamPack` · 0052b1d0 · legions 005108b0 — ac2-world region::sky_objects, ac2-render sky
- [-] `SkyObject::ToFileNode` · 0052b350 — text source format (admin tools)
- [-] `SkyObject::LoadName` · 0052b7b0 — text source loader, admin build only
- [-] `SkyObject::FromFileNode` · 0052b880 — text source format (admin tools)

## `portal/shared/src/region/sky/skyobjectreplace.cpp.c` (1/9)

### CMapNoteRow
- [ ] `CMapNoteRow::GetSubDataIDs` · 006c81f0 · legions 00509bd0

### SkyObjectReplace
- [ ] `SkyObjectReplace::GetDiffuseColor` · 006c8210

### MotionValues
- [ ] `MotionValues::GetMaxVertVel` · 006c8250

### SkyObjectReplace
- [ ] `SkyObjectReplace::SkyObjectReplace` · 006c8260
- [x] `SkyObjectReplace::StreamPack` · 006c82b0 · legions 00656980 — ac2-world region::sky_objects, ac2-render sky
- [-] `SkyObjectReplace::ToFileNode` · 006c8330 — text source format (admin tools)
- [-] `SkyObjectReplace::LoadName` · 006c8540 — text source loader, admin build only

### SkyObject
- [-] `SkyObject::LoadGfx` · 006c8610 — text source loader, admin build only

### SkyObjectReplace
- [-] `SkyObjectReplace::FromFileNode` · 006c86e0 — text source format (admin tools)

## `portal/shared/src/region/sky/skytimeofday.cpp.c` (0/17)

### SkyTimeOfDay
- [ ] `SkyTimeOfDay::Destroy` · 006c8c90 · legions 00656a40
- [~] `SkyTimeOfDay::StreamPack` · 006c8d90 · legions 00656b40 — decoded; field order after m_nDirectHeading not re-verified
- [ ] `SkyTimeOfDay::GetSubDataIDs` · 006c8f10 · legions 00656cc0
- [ ] `SkyTimeOfDay::FindReplacement` · 006c8f50 · legions 00656d00
- [ ] `SkyTimeOfDay::SkyTimeOfDay` · 006c8f90
- [ ] `SkyTimeOfDay::~SkyTimeOfDay` · 006c9070 · legions 00656e20

### SmartArray<SkyObjectReplace>
- [ ] `SmartArray<SkyObjectReplace>::operator=` · 006c90c0 · legions 00656e70

### PSmartArray<SkyObjectReplace>
- [ ] `PSmartArray<SkyObjectReplace>::StreamPack` · 006c9160 · legions 00656f10

### SkyTimeOfDay
- [-] `SkyTimeOfDay::ToFileNode` · 006c9200 — text source format (admin tools)
- [-] `SkyTimeOfDay::LoadRain` · 006c9660 — text source loader, admin build only
- [-] `SkyTimeOfDay::LoadWorldFog` · 006c9820 — text source loader, admin build only
- [-] `SkyTimeOfDay::LoadDirectLight` · 006c9b10 — text source loader, admin build only
- [-] `SkyTimeOfDay::LoadAmbientLight` · 006c9d80 — text source loader, admin build only
- [ ] `SkyTimeOfDay::operator=` · 006c9f30 · legions 00656fb0
- [-] `SkyTimeOfDay::CheckDupAdd` · 006ca020 — text source format (admin tools)
- [-] `SkyTimeOfDay::LoadReplace` · 006ca070 — text source loader, admin build only
- [-] `SkyTimeOfDay::FromFileNode` · 006ca0e0 — text source format (admin tools)

## `portal/shared/src/region/surface/lscapealphadesc.cpp.c` (0/5)

### LScapeAlphaDesc
- [ ] `LScapeAlphaDesc::LScapeAlphaDesc` · 006c4160

### LScapeMaterialDesc
- [ ] `LScapeMaterialDesc::~LScapeMaterialDesc` · 006c4180 · legions 00654f80

### TimeOfDay
- [ ] `TimeOfDay::StreamPack` · 006c41a0 · legions 00632ab0

### LScapeAlphaDesc
- [-] `LScapeAlphaDesc::LoadTexture` · 006c4200 — text source loader, admin build only
- [ ] `LScapeAlphaDesc::Load` · 006c42e0

## `portal/shared/src/region/surface/lscapematerialdesc.cpp.c` (1/4)

### LScapeMaterialDesc
- [ ] `LScapeMaterialDesc::LScapeMaterialDesc` · 006c3e30
- [x] `LScapeMaterialDesc::StreamPack` · 006c3e50 · legions 00654f00
- [-] `LScapeMaterialDesc::LoadMaterial` · 006c3eb0 — text source loader, admin build only
- [ ] `LScapeMaterialDesc::Load` · 006c3f80

## `portal/shared/src/region/surface/lscapesurfacetype.cpp.c` (2/11)

### LScapeSurfaceType
- [x] `LScapeSurfaceType::StreamPack` · 006c5910 · legions 00655c10
- [ ] `LScapeSurfaceType::GetSubDataIDs` · 006c5990 · legions 00655c90
- [x] `LScapeSurfaceType::GetVertexInfo` · 006c59d0 · legions 00655cd0
- [ ] `LScapeSurfaceType::GetPhysicsMaterial` · 006c5a50 · legions 00655d50
- [ ] `LScapeSurfaceType::LScapeSurfaceType` · 006c5ab0
- [ ] `LScapeSurfaceType::~LScapeSurfaceType` · 006c5ae0 · legions 00655de0
- [-] `LScapeSurfaceType::LoadName` · 006c5b30 — text source loader, admin build only

### PSmartArray<MaterialGroup>
- [ ] `PSmartArray<MaterialGroup>::StreamPack` · 006c5ca0 · legions 00655e30

### SmartArray<MaterialGroup>
- [ ] `SmartArray<MaterialGroup>::AddToEnd` · 006c5d50

### LScapeSurfaceType
- [-] `LScapeSurfaceType::LoadMatGroup` · 006c5da0 — text source loader, admin build only
- [ ] `LScapeSurfaceType::Load` · 006c5e10

## `portal/shared/src/region/surface/lscapevertexcolordesc.cpp.c` (2/4)

### LScapeVertexColorDesc
- [x] `LScapeVertexColorDesc::CalcVertexColor` · 006c36d0 · legions 00654a80
- [ ] `LScapeVertexColorDesc::LScapeVertexColorDesc` · 006c39a0
- [x] `LScapeVertexColorDesc::StreamPack` · 006c3a20 · legions 00654dd0
- [ ] `LScapeVertexColorDesc::Load` · 006c3b30

## `portal/shared/src/region/surface/materialgroup.cpp.c` (6/20)

### MaterialGroup
- [x] `MaterialGroup::GetRandom` · 006c44d0 · legions 00654fa0
- [ ] `MaterialGroup::Destroy` · 006c4530 · legions 00655000
- [x] `MaterialGroup::StreamPack` · 006c45a0 · legions 00655070
- [ ] `MaterialGroup::GetSubDataIDs` · 006c47d0 · legions 006552a0
- [x] `MaterialGroup::GetColor` · 006c4970 · legions 00655440
- [x] `MaterialGroup::GetAlpha` · 006c4b30 · legions 00655570
- [x] `MaterialGroup::GetTexture` · 006c4bd0 · legions 00655610
- [ ] `MaterialGroup::MaterialGroup` · 006c4c80
- [ ] `MaterialGroup::~MaterialGroup` · 006c4d70 · legions 006557b0
- [x] `MaterialGroup::GetVertexInfo` · 006c4ed0 · legions 00655910
- [-] `MaterialGroup::LoadPhysicsMaterial` · 006c4ff0 — text source loader, admin build only

### PSmartArray<LScapeVertexColorDesc>
- [ ] `PSmartArray<LScapeVertexColorDesc>::StreamPack` · 006c50f0 · legions 00655a30

### PSmartArray<LScapeMaterialDesc>
- [ ] `PSmartArray<LScapeMaterialDesc>::StreamPack` · 006c5190 · legions 00655ad0

### PSmartArray<LScapeAlphaDesc>
- [ ] `PSmartArray<LScapeAlphaDesc>::StreamPack` · 006c5230 · legions 00655b70

### SmartArray<LScapeMaterialDesc>
- [ ] `SmartArray<LScapeMaterialDesc>::AddToEnd` · 006c52d0

### SmartArray<LScapeAlphaDesc>
- [ ] `SmartArray<LScapeAlphaDesc>::AddToEnd` · 006c5330

### MaterialGroup
- [-] `MaterialGroup::LoadVertexColor` · 006c5390 — text source loader, admin build only
- [-] `MaterialGroup::LoadMaterial` · 006c5450 — text source loader, admin build only
- [-] `MaterialGroup::LoadAlpha` · 006c54d0 — text source loader, admin build only
- [ ] `MaterialGroup::Load` · 006c5550

## `portal/shared/src/region/surface/surfacedesc.cpp.c` (3/24)

### CSurfaceDesc
- [ ] `CSurfaceDesc::Destroy` · 00522360
- [x] `CSurfaceDesc::StreamPack` · 00522380 · legions 0050c820
- [ ] `CSurfaceDesc::GetSubDataIDs` · 00522480
- [x] `CSurfaceDesc::GetVertexInfo` · 00522520 · legions 0050c9c0
- [x] `CSurfaceDesc::GetRoadInfo` · 00522580 · legions 0050ca20
- [ ] `CSurfaceDesc::GetPhysicsMaterial` · 005225e0 · legions 0050ca80
- [ ] `CSurfaceDesc::GetRoadPhysicsMaterial` · 00522630 · legions 0050cad0

### LScapeVertexColorDesc
- [ ] `LScapeVertexColorDesc::operator=` · 00522680 · legions 0050cb20

### CSurfaceDesc
- [ ] `CSurfaceDesc::CSurfaceDesc` · 00522710 · legions 0050cbb0
- [ ] `CSurfaceDesc::GetDBOType` · 00522770
- [ ] `CSurfaceDesc::~CSurfaceDesc` · 00522780 · legions 0050cc20

### PFileNode
- [ ] `PFileNode::GetSubName` · 00522ad0 · legions 0050cf70

### CSurfaceDesc
- [ ] `CSurfaceDesc::Allocate` · 00522af0

### SmartArray<LScapeVertexColorDesc>
- [ ] `SmartArray<LScapeVertexColorDesc>::operator=` · 00522b20 · legions 0050cfc0

### SmartArray<LScapeMaterialDesc>
- [ ] `SmartArray<LScapeMaterialDesc>::operator=` · 00522b90 · legions 0050d030

### SmartArray<LScapeAlphaDesc>
- [ ] `SmartArray<LScapeAlphaDesc>::operator=` · 00522c00 · legions 0050d0a0

### MaterialGroup
- [ ] `MaterialGroup::operator=` · 00522c70 · legions 0050d110

### SmartArray<MaterialGroup>
- [ ] `SmartArray<MaterialGroup>::operator=` · 00522e90 · legions 0050d330

### LScapeSurfaceType
- [ ] `LScapeSurfaceType::operator=` · 00522f00 · legions 0050d3a0

### PSmartArray<LScapeSurfaceType>
- [ ] `PSmartArray<LScapeSurfaceType>::StreamPack` · 00523020 · legions 0050d4c0

### SmartArray<LScapeSurfaceType>
- [ ] `SmartArray<LScapeSurfaceType>::AddToEnd` · 005230c0

### CSurfaceDesc
- [-] `CSurfaceDesc::LoadSurface` · 00523130 — text source loader, admin build only
- [-] `CSurfaceDesc::LoadRoad` · 00523190 — text source loader, admin build only
- [-] `CSurfaceDesc::FromFileNode` · 005231f0 — text source format (admin tools)

## `portal/shared/src/region/terrain/terraindesc.cpp.c` (0/15)

### CTerrainDesc
- [ ] `CTerrainDesc::CTerrainDesc` · 00521ad0 · legions 0050c260
- [ ] `CTerrainDesc::GetDBOType` · 00521b00
- [ ] `CTerrainDesc::Destroy` · 00521b10 · legions 0050c2a0
- [ ] `CTerrainDesc::pack_size` · 00521b80 · legions 0050c310
- [ ] `CTerrainDesc::GetSubDataIDs` · 00521bf0 · legions 0050c380
- [ ] `CTerrainDesc::FindTerrainType` · 00521c40 · legions 0050c3d0
- [ ] `CTerrainDesc::GetSurfaceType` · 00521c90 · legions 0050c420
- [ ] `CTerrainDesc::Allocate` · 00521cc0
- [ ] `CTerrainDesc::Pack` · 00521cd0
- [ ] `CTerrainDesc::GetScene` · 00521de0
- [ ] `CTerrainDesc::GetSoundType` · 00521ed0
- [ ] `CTerrainDesc::UnPack` · 00522000
- [-] `CTerrainDesc::CheckDupAdd` · 00522140 — text source format (admin tools)
- [-] `CTerrainDesc::LoadTerrainType` · 005221b0 — text source loader, admin build only
- [-] `CTerrainDesc::FromFileNode` · 00522240 — text source format (admin tools)

## `portal/shared/src/region/terrain/terrainsceneinfo.cpp.c` (0/5)

### TerrainSceneInfo
- [ ] `TerrainSceneInfo::pack` · 006c6900 · legions 006563a0
- [ ] `TerrainSceneInfo::unpack` · 006c6950 · legions 006563f0
- [ ] `TerrainSceneInfo::~TerrainSceneInfo` · 006c69a0 · legions 00656440
- [-] `TerrainSceneInfo::LoadSceneName` · 006c69b0 — text source loader, admin build only
- [ ] `TerrainSceneInfo::Load` · 006c6a70

## `portal/shared/src/region/terrain/terraintype.cpp.c` (0/10)

### TerrainType
- [ ] `TerrainType::pack_size` · 006c5fe0 · legions 00655ee0
- [ ] `TerrainType::GetSceneType` · 006c6080 · legions 00655f80
- [ ] `TerrainType::TerrainType` · 006c60c0 · legions 00655fc0
- [ ] `TerrainType::Destroy` · 006c6100 · legions 00656000
- [ ] `TerrainType::~TerrainType` · 006c61b0 · legions 006560b0
- [-] `TerrainType::LoadScene` · 006c61f0 — text source loader, admin build only
- [-] `TerrainType::LoadSurface` · 006c6280 — text source loader, admin build only
- [ ] `TerrainType::pack` · 006c6340 · legions 006560f0
- [ ] `TerrainType::unpack` · 006c6460 · legions 00656210
- [ ] `TerrainType::Load` · 006c65f0

## `portal/shared/src/region/water/waterdesc.cpp.c` (0/21)

### CWaterDesc
- [ ] `CWaterDesc::CWaterDesc` · 00521060 · legions 0050bcd0
- [ ] `CWaterDesc::GetDBOType` · 00521090
- [ ] `CWaterDesc::Destroy` · 005210a0 · legions 0050bd10
- [ ] `CWaterDesc::Pack` · 00521110
- [ ] `CWaterDesc::GetPackSize` · 00521240 · legions 0050beb0
- [ ] `CWaterDesc::GetSubDataIDs` · 005212a0 · legions 0050bf10
- [ ] `CWaterDesc::FindWaterType` · 005212f0 · legions 0050bf60
- [ ] `CWaterDesc::HasWater` · 00521340 · legions 0050bfb0
- [ ] `CWaterDesc::GetTileWidth` · 00521360 · legions 0050bfd0
- [ ] `CWaterDesc::GetAboveMaterial` · 00521380 · legions 0050bff0
- [ ] `CWaterDesc::GetBelowMaterial` · 005213a0 · legions 0050c010
- [ ] `CWaterDesc::GetWaterLighting` · 005213c0 · legions 0050c030
- [ ] `CWaterDesc::GetVertexColor` · 005213f0 · legions 0050c060
- [ ] `CWaterDesc::GetPhysicsMaterial` · 00521410 · legions 0050c080
- [ ] `CWaterDesc::Allocate` · 00521430
- [ ] `CWaterDesc::UnPack` · 00521480
- [-] `CWaterDesc::ToFileNode` · 005215f0 — text source format (admin tools)
- [-] `CWaterDesc::CheckDupAdd` · 00521780 — text source format (admin tools)
- [-] `CWaterDesc::LoadID` · 005217f0 — text source loader, admin build only
- [-] `CWaterDesc::LoadWater` · 005218d0 — text source loader, admin build only
- [-] `CWaterDesc::FromFileNode` · 00521960 — text source format (admin tools)

## `portal/shared/src/region/water/waterlighting.cpp.c` (0/7)

### WaterLighting
- [ ] `WaterLighting::WaterLighting` · 006ccf30 · legions 00658020

### FogType
- [ ] `FogType::pack` · 006ccf40 · legions 00658030

### WaterLighting
- [ ] `WaterLighting::unpack` · 006ccf70 · legions 00658060
- [ ] `WaterLighting::GetLight` · 006ccfb0 · legions 006580b0
- [ ] `WaterLighting::~WaterLighting` · 006ccfe0 · legions 006580e0
- [-] `WaterLighting::FromFileNode` · 006cd000 — text source format (admin tools)
- [-] `WaterLighting::ToFileNode` · 006cd160 — text source format (admin tools)

## `portal/shared/src/region/water/watertype.cpp.c` (0/14)

### WaterLightInfo
- [ ] `WaterLightInfo::WaterLightInfo` · 006cc080 · legions 00657a10

### WaterType
- [ ] `WaterType::GetSubDataIDs` · 006cc0a0 · legions 00657a30
- [ ] `WaterType::WaterType` · 006cc140 · legions 00657ad0
- [ ] `WaterType::Destroy` · 006cc1a0 · legions 00657b30
- [ ] `WaterType::pack` · 006cc240 · legions 00657bd0
- [ ] `WaterType::GetPackSize` · 006cc350 · legions 00657ce0
- [ ] `WaterType::~WaterType` · 006cc450 · legions 00657de0
- [ ] `WaterType::unpack` · 006cc470 · legions 00657e00
- [-] `WaterType::ToFileNode` · 006cc5e0 — text source format (admin tools)
- [-] `WaterType::LoadMaterial` · 006cc8a0 — text source loader, admin build only
- [-] `WaterType::LoadPhysMat` · 006cc9a0 — text source loader, admin build only
- [-] `WaterType::LoadLight` · 006ccad0 — text source loader, admin build only
- [-] `WaterType::FromFileNode` · 006ccb30 — text source format (admin tools)
- [ ] `WaterType::GetWaterLighting` · 006cce80 · legions 00657f70
