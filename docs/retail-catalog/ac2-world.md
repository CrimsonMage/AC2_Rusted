# ac2-world: Cells, landblocks, regions, terrain, sky, water, game time

989 retail functions: 190 done, 25 partial, 323 to do, 451 skipped.
Legend and workflow: [README](README.md).

## `portal/shared/src/cell/envcell.cpp.c` (19/140)

### PackObj
- [-] `PackObj::Pack` · 0047f1a0 — write path (client reads DATs only)

### (free functions)
- [-] `STREAMPACK_ALIGN` · 0047f1c0 · legions 004791d0 — retail StreamPack macro dispatch (Rust decoders)

### Frame
- [x] `Frame::localtoglobalvec` · 0047f1f0 · legions 00479200

### FileNodeName_Bool
- [-] `FileNodeName_Bool::Clone` · 0047f250 · legions 00479260 — FileNode name wrapper (Rust strings)

### BBox
- [ ] `BBox::DoesOverlap` · 0047f280 · legions 00479290 — needs port: bounding box math

### RGBAColor
- [x] `RGBAColor::GetColor32` · 0047f3c0 · legions 004793d0 — ac2-ui tree_view.rs
- [x] `RGBAColor::SetColor32` · 0047f410 · legions 00479420
- [x] `RGBAColor::CapLow` · 0047f4b0 · legions 004794c0
- [x] `RGBAColor::CapHigh` · 0047f500 · legions 00479510

### Plane
- [ ] `Plane::which_side` · 0047f550 · legions 00479560 — needs port: plane math (terrain and cell planes)

### Waveform
- [-] `Waveform::operator=` · 0047f5a0 · legions 004795b0 — copy assignment of plain data (Rust clone/move)

### LightSource
- [ ] `LightSource::SetDiffuseIntensity` · 0047f5f0 · legions 00479600 — needs port: light sources

### AppliedAppearanceKey
- [-] `AppliedAppearanceKey::AppliedAppearanceKey` · 0047f650 · legions 00479660 — constructor (struct init; Rust builds it directly)

### IconLayerDesc
- [-] `IconLayerDesc::operator=` · 0047f670 · legions 00479680 — copy assignment of plain data (Rust clone/move)

### IconModAtom
- [-] `IconModAtom::IconModAtom` · 0047f6b0 — constructor (struct init; Rust builds it directly)
- [-] `IconModAtom::~IconModAtom` · 0047f6d0 — destructor (Rust Drop)

### CEnvCell
- [ ] `CEnvCell::CullingPlaneCheck` · 0047f730 · legions 00479700 — needs port: indoor env cell: visibility, lights, portals, rendering

### Frustum
- [-] `Frustum::operator=` · 0047f9b0 · legions 00479980 — copy assignment of plain data (Rust clone/move)

### CEnvCell
- [x] `CEnvCell::point_in_cell` · 0047fa10 — ac2-physics envcell.rs, find_cells.rs: interior collision, containment and portal walk (Legions identical)
- [x] `CEnvCell::find_collisions` · 0047fa50 — ac2-physics envcell.rs, find_cells.rs: interior collision, containment and portal walk (Legions identical)
- [x] `CEnvCell::find_env_collisions` · 0047fa80 — ac2-physics envcell.rs, find_cells.rs: interior collision, containment and portal walk (Legions identical)
- [ ] `CEnvCell::GetSubDataIDs` · 0047fb70 · legions 00479b10 — needs port: indoor env cell: visibility, lights, portals, rendering (sub-object DID list for async sub-object resolution, AGENTS.md)
- [ ] `CEnvCell::SumBumpMapVectors` · 0047fbd0 · legions 00479b70 — needs port: indoor env cell: visibility, lights, portals, rendering
- [ ] `CEnvCell::NormalizeBumpMapVectors` · 0047fd00 · legions 00479ca0 — needs port: indoor env cell: visibility, lights, portals, rendering
- [ ] `CEnvCell::BuildOptionalProperties` · 00480180 — needs port: indoor env cell: visibility, lights, portals, rendering
- [x] `CEnvCell::GetPhysicsMaterialDID` · 004801b0 — Physics materials: ac2-world reads each vertex's surface or road band material (LScapeSurfaceType band by pitch) and each water type's material; ac2-physics Terrain returns the land vertex, block water and interior cell materials, read by PhysicsBody::contact_materials.

### SmartArray<unsigned_long>
- [-] `SmartArray<unsigned_long>::SetNElements` · 004801c0 · legions 0047a160 — SmartArray container operation (Vec replaces it)

### LandDefs
- [ ] `LandDefs::inbound_valid_blockid` · 00480320 · legions 0047a290 — needs port: landblock id and coordinate arithmetic
- [ ] `LandDefs::blockpart_to_blockcoord` · 00480390 · legions 0047a300 — needs port: landblock id and coordinate arithmetic

### FileNodeName_UInt32
- [-] `FileNodeName_UInt32::Clone` · 00480400 · legions 0047a370 — FileNode name wrapper (Rust strings)

### Ray
- [x] `Ray::Ray` · 00480430 · legions 0047a3a0 — crates/ac2-physics/src/los/mod.rs Ray::Ray (zero point, direction, length)

### CellFrustum
- [-] `CellFrustum::CellFrustum` · 00480470 · legions 0047a3e0 — constructor (struct init; Rust builds it directly)

### OpacityAtom
- [-] `OpacityAtom::~OpacityAtom` · 004804f0 — destructor (Rust Drop)

### MeshReplacementAtom
- [-] `MeshReplacementAtom::~MeshReplacementAtom` · 00480550 — destructor (Rust Drop)

### PGDReplacementAtom
- [-] `PGDReplacementAtom::PGDReplacementAtom` · 004805b0 — constructor (struct init; Rust builds it directly)
- [-] `PGDReplacementAtom::~PGDReplacementAtom` · 004805e0 — destructor (Rust Drop)

### CEnvCell
- [-] `CEnvCell::Destroy` · 00480650 · legions 0047a520 — DBO release (Drop frees on last drop, AGENTS.md)
- [-] `CEnvCell::RecoverFromFreelist` · 004807b0 — retail free-object pool (DBOCache exception, AGENTS.md)
- [-] `CEnvCell::ClearForFreelist` · 00480800 · legions 0047a6d0 — retail free-object pool (DBOCache exception, AGENTS.md)
- [x] `CEnvCell::GetTouchesOutside` · 00480860 · legions 0047a730 — ac2-physics physics_obj_place.rs: SetPosition placement check (Legions identical)
- [ ] `CEnvCell::DestroyStaticLights` · 00480890 · legions 0047a760 — needs port: indoor env cell: visibility, lights, portals, rendering
- [~] `CEnvCell::DrawCellMesh` · 00480900 · legions 0047a7d0 — cell mesh with appearance atoms at the cell frame, point lights cached at 1.5x bbox radius; degrade selection fixed at top
- [ ] `CEnvCell::CullObjects` · 00480a80 · legions 0047a940 — needs port: indoor env cell: visibility, lights, portals, rendering
- [ ] `CEnvCell::LinkStaticLights` · 00480bc0 · legions 0047aa80 — needs port: indoor env cell: visibility, lights, portals, rendering
- [ ] `CEnvCell::LinkDynamicLights` · 00480c00 · legions 0047aac0 — needs port: indoor env cell: visibility, lights, portals, rendering
- [ ] `CEnvCell::RenderLightCoronas` · 00480c40 · legions 0047ab00 — needs port: indoor env cell: visibility, lights, portals, rendering
- [x] `CEnvCell::FindTransitCellsInternal` · 00480cb0 · legions 0047ab70 — ac2-physics envcell.rs, find_cells.rs: interior collision, containment and portal walk (Legions identical)
- [ ] `CEnvCell::ReleaseSubObjects` · 00480f30 · legions 0047adf0 — needs port: indoor env cell: visibility, lights, portals, rendering
- [ ] `CEnvCell::CopyDiffuseToVB` · 00480fa0 · legions 0047ae60 — needs port: indoor env cell: visibility, lights, portals, rendering
- [ ] `CEnvCell::CacheDiffuseFromVB` · 00480ff0 · legions 0047aeb0 — needs port: indoor env cell: visibility, lights, portals, rendering
- [x] `CEnvCell::LightIndices` · 00481030 · legions 0047aef0 — ac2-render object::cell / point_lights
- [ ] `CEnvCell::TraceRayAgainstMesh` · 004813a0 · legions 0047b260 — needs port: CEnvCell::TraceRayAgainstMesh (exact render-mesh ray trace; physics holds no render mesh)
- [ ] `CEnvCell::ApplyPropertyCollection` · 00481580 — needs port: indoor env cell: visibility, lights, portals, rendering
- [ ] `CEnvCell::InqVisibleCells` · 00481630 · legions 0047b4f0 — needs port: indoor env cell: visibility, lights, portals, rendering
- [ ] `CEnvCell::GetCellPlanes` · 00481680 — needs port: indoor env cell: visibility, lights, portals, rendering

### IntrusiveSmartPointer<LightSource>
- [-] `IntrusiveSmartPointer<LightSource>::~IntrusiveSmartPointer<LightSource>` · 00481710 · legions 0047b5d0 — intrusive pointer; Arc replaces it
- [-] `IntrusiveSmartPointer<LightSource>::create` · 00481740 · legions 0047b600 — intrusive pointer; Arc replaces it

### FileNodeName_PString
- [-] `FileNodeName_PString::Clone` · 00481cf0 · legions 0047bc60 — FileNode name wrapper (Rust strings)
- [-] `FileNodeName_PString::FromPString` · 00481d20 · legions 0047bc90 — FileNode name wrapper (Rust strings)
- [-] `FileNodeName_PString::ToPString` · 00481d70 · legions 0047bce0 — FileNode name wrapper (Rust strings)

### PFileNode
- [ ] `PFileNode::GetNameAsBool` · 00481dc0 · legions 0047bd30 — needs port: FileNode accessors (used by FromFileNode loaders; confirm DAT path)
- [ ] `PFileNode::GetLeafAsBool` · 00481e80 — needs port: FileNode accessors (used by FromFileNode loaders; confirm DAT path)
- [ ] `PFileNode::GetNameAsUInt32` · 00481ea0 · legions 0047bdf0 — needs port: FileNode accessors (used by FromFileNode loaders; confirm DAT path)
- [ ] `PFileNode::GetNameAsPString` · 00481f60 · legions 0047beb0 — needs port: FileNode accessors (used by FromFileNode loaders; confirm DAT path)
- [ ] `PFileNode::GetSubNameAsPString` · 00482040 — needs port: FileNode accessors (used by FromFileNode loaders; confirm DAT path)

### LOSParameters
- [x] `LOSParameters::SetFinalRay` · 00482060 · legions 0047bf90 — ac2-physics los (Terrain::los_check)

### SceneTool
- [-] `SceneTool::GetSunlight` · 004820b0 · legions 0047bfe0 — scene tool (admin editor)

### CEnvCell
- [-] `CEnvCell::CEnvCell` · 004820d0 · legions 0047c000 — constructor (struct init; Rust builds it directly)
- [ ] `CEnvCell::HasLighting` · 004822c0 — needs port: indoor env cell: visibility, lights, portals, rendering
- [ ] `CEnvCell::SetGatherTimestamp` · 004822d0 — needs port: indoor env cell: visibility, lights, portals, rendering
- [ ] `CEnvCell::GetGatherTimestamp` · 004822e0 — needs port: indoor env cell: visibility, lights, portals, rendering
- [-] `CEnvCell::GetDBOType` · 004822f0 — DBO type tag (Rust typed decoders)
- [ ] `CEnvCell::~CEnvCell` · 00482310 · legions 0047c240 — needs port: CEnvCell destructor unload (world hash and resident state)
- [x] `CEnvCell::FindTransitCells` · 00482450 — ac2-physics envcell.rs, find_cells.rs: interior collision, containment and portal walk (Legions identical)
- [ ] `CEnvCell::GetSubObjects` · 00482460 — needs port: indoor env cell: visibility, lights, portals, rendering
- [ ] `CEnvCell::ApplyAtomCollection` · 004825c0 · legions 0047c490 — needs port: indoor env cell: visibility, lights, portals, rendering
- [~] `CEnvCell::UpdateDynamicLights` · 00482710 · legions 0047c5b0 — Cell lights here are the block's static CLightInfo lights (ac2-render scene/interiors.rs object_point_lights). Object FX lights never reach a cell: retail links them with SceneTool::LinkLight(bDynamic=1) into the object-only dynamic list, and CEnvCell lighting reads only CLightInfo. Per-vertex sun and time-of-day baking (CopyDiffuseToVB, LightIndices on the sun indices) is not reproduced by this row
- [ ] `CEnvCell::UpdateCellMaterial` · 00482a80 — needs port: indoor env cell: visibility, lights, portals, rendering

### SmartArray<CCellPortal>
- [-] `SmartArray<CCellPortal>::SetNElements` · 00482b40 · legions 0047c9e0 — SmartArray container operation (Vec replaces it)

### PFileNode
- [ ] `PFileNode::GetSubName` · 00482c10 · legions 0047cab0 — needs port: FileNode accessors (used by FromFileNode loaders; confirm DAT path)

### CEnvCell
- [-] `CEnvCell::Allocate` · 00482c30 — retail free-object pool (DBOCache exception, AGENTS.md)
- [-] `CEnvCell::Allocator` · 00482c40 — retail free-object pool (DBOCache exception, AGENTS.md)
- [~] `CEnvCell::DynamicLightVisibleCells` · 00482c80 · legions 0047cb20 — Cell lights here are the block's static CLightInfo lights (ac2-render scene/interiors.rs object_point_lights). Object FX lights never reach a cell: retail links them with SceneTool::LinkLight(bDynamic=1) into the object-only dynamic list, and CEnvCell lighting reads only CLightInfo. Per-vertex sun and time-of-day baking (CopyDiffuseToVB, LightIndices on the sun indices) is not reproduced by this row
- [ ] `CEnvCell::find_in_visible_cells` · 00482d00 · legions 0047cba0 — needs port: indoor env cell: visibility, lights, portals, rendering
- [x] `CEnvCell::StreamPack` · 00482da0 — crates/ac2-world/src/ac2_env_cell.rs module decode in CEnvCell::StreamPack order

### SmartArray<Vector>
- [-] `SmartArray<Vector>::AddToEnd` · 00483190 · legions 0047d030 — SmartArray container operation (Vec replaces it)

### SmartArray<PhysObjBrush>
- [-] `SmartArray<PhysObjBrush>::AddToEnd` · 004831f0 · legions 0047d090 — SmartArray container operation (Vec replaces it)

### SmartArray<CellFrustum>
- [-] `SmartArray<CellFrustum>::AddToEnd` · 00483250 · legions 0047d0f0 — SmartArray container operation (Vec replaces it)

### SmartArray<Frustum>
- [-] `SmartArray<Frustum>::AddToEnd` · 004832c0 · legions 0047d160 — SmartArray container operation (Vec replaces it)

### FileNodeName_PString
- [-] `FileNodeName_PString::GetPackSize` · 00483410 · legions 0047d2b0 — write path (client reads DATs only)
- [-] `FileNodeName_PString::Pack` · 00483430 · legions 0047d2d0 — write path (client reads DATs only)
- [-] `FileNodeName_PString::UnPack` · 00483440 · legions 0047d2e0 — FileNode name wrapper (Rust strings)

### CEnvCell
- [~] `CEnvCell::CreatePhysicsObj` · 00483460 · legions 0047d300 — ac2-world env_cell_markers: drawn placement only (no physics object, no collision)
- [ ] `CEnvCell::CreateLightSource` · 004834f0 · legions 0047d390 — needs port: indoor env cell: visibility, lights, portals, rendering
- [ ] `CEnvCell::CheckVisibility` · 00483680 · legions 0047d520 — needs port: indoor env cell: visibility, lights, portals, rendering
- [ ] `CEnvCell::GatherInsideObjects` · 00483970 · legions 0047d810 — needs port: indoor env cell: visibility, lights, portals, rendering
- [-] `CEnvCell::FromFileNode` · 00483a60 — text source format (admin tools)
- [ ] `CEnvCell::AddStaticLights` · 00484090 · legions 0047d900 — needs port: indoor env cell: visibility, lights, portals, rendering
- [ ] `CEnvCell::GetNearbyGeometry` · 00484200 · legions 0047da70 — needs port: indoor env cell: visibility, lights, portals, rendering
- [x] `CEnvCell::InitStaticObjects` · 00484680 · legions 0047def0 — ac2-world env_cell_markers; drawn by ac2-render scene interiors
- [ ] `CEnvCell::InitLightSources` · 00484710 · legions 0047df80 — needs port: indoor env cell: visibility, lights, portals, rendering
- [ ] `CEnvCell::GatherInsideObjects` · 00484760 · legions 0047dfd0 — needs port: indoor env cell: visibility, lights, portals, rendering
- [ ] `CEnvCell::InitLoad` · 00484800 — needs port: indoor env cell: visibility, lights, portals, rendering (post-load init)

### PList<IconModAtom>
- [-] `PList<IconModAtom>::GetPackSize` · 004848a0 — write path (client reads DATs only)
- [ ] `PList<IconModAtom>::UnPack` · 004848e0 · legions 0047e1b4 — needs port: counted pointer lists in DAT records (DAT decode)

### ListNode<IconModAtom>
- [-] `ListNode<IconModAtom>::ListNode<IconModAtom>` · 00484a30 · legions 0047e2f0 — constructor (struct init; Rust builds it directly)

### CEnvCell
- [ ] `CEnvCell::GetVisible` · 00484e30 · legions 0047e710 — needs port: indoor env cell: visibility, lights, portals, rendering
- [ ] `CEnvCell::CullCellGroupInternal` · 00484e70 · legions 0047e750 — needs port: indoor env cell: visibility, lights, portals, rendering
- [ ] `CEnvCell::CullCellsInternal` · 00485290 · legions 0047eb70 — needs port: indoor env cell: visibility, lights, portals, rendering
- [ ] `CEnvCell::DrawEnvironment` · 004856b0 — needs port: indoor env cell: visibility, lights, portals, rendering
- [ ] `CEnvCell::DrawPostEffects` · 00485770 — needs port: indoor env cell: visibility, lights, portals, rendering
- [ ] `CEnvCell::GatherVisibleInsideObjects` · 00485800 — needs port: indoor env cell: visibility, lights, portals, rendering
- [x] `CEnvCell::find_visible_child_cell` · 00485890 · legions 0047f140 — ac2-physics physics_obj_place.rs: SetPosition placement check (Legions identical)
- [~] `CEnvCell::TraceRay` · 00485930 · legions 0047f1e0 — ac2-physics los: static objects only (no dynamic shadow list, blocking masks or exact drawing rep)

### PList<MaterialModifierAtom>
- [-] `PList<MaterialModifierAtom>::Pack` · 00486180 · legions 0047fa20 — write path (client reads DATs only)
- [-] `PList<MaterialModifierAtom>::GetPackSize` · 004861d0 — write path (client reads DATs only)
- [ ] `PList<MaterialModifierAtom>::UnPack` · 00486210 · legions 0047fac4 — needs port: counted pointer lists in DAT records (DAT decode)

### MaterialModifierAtom
- [-] `MaterialModifierAtom::MaterialModifierAtom` · 00486360 · legions 0047fc00 — constructor (struct init; Rust builds it directly)

### PList<OpacityAtom>
- [-] `PList<OpacityAtom>::Pack` · 004863b0 · legions 0064fdb0 — write path (client reads DATs only)
- [ ] `PList<OpacityAtom>::UnPack` · 00486400 · legions 0047fc64 — needs port: counted pointer lists in DAT records (DAT decode)

### PList<MeshReplacementAtom>
- [ ] `PList<MeshReplacementAtom>::UnPack` · 00486560 · legions 0047fe04 — needs port: counted pointer lists in DAT records (DAT decode)

### ListNode<MeshReplacementAtom>
- [-] `ListNode<MeshReplacementAtom>::ListNode<MeshReplacementAtom>` · 004866a0 · legions 0047ff30 — constructor (struct init; Rust builds it directly)

### PList<PGDReplacementAtom>
- [-] `PList<PGDReplacementAtom>::Pack` · 004866e0 · legions 0047ff70 — write path (client reads DATs only)
- [-] `PList<PGDReplacementAtom>::GetPackSize` · 00486730 — write path (client reads DATs only)
- [ ] `PList<PGDReplacementAtom>::UnPack` · 00486770 — needs port: counted pointer lists in DAT records (DAT decode)

### ListNode<PGDReplacementAtom>
- [-] `ListNode<PGDReplacementAtom>::ListNode<PGDReplacementAtom>` · 004868f0 · legions 00480180 — constructor (struct init; Rust builds it directly)

### CEnvCell
- [x] `CEnvCell::Trace` · 00486af0 — ac2-physics los (Terrain::los_check)
- [ ] `CEnvCell::release_visible` · 00486b00 · legions 00480390 — needs port: indoor env cell: visibility, lights, portals, rendering
- [ ] `CEnvCell::ReleaseVisibleCells` · 00486bd0 — needs port: indoor env cell: visibility, lights, portals, rendering
- [ ] `CEnvCell::add_visible_cell` · 00486d10 · legions 004805a0 — needs port: indoor env cell: visibility, lights, portals, rendering
- [ ] `CEnvCell::CullCells` · 00486dd0 — needs port: indoor env cell: visibility, lights, portals, rendering
- [ ] `CEnvCell::grab_visible` · 00486ee0 · legions 00480740 — needs port: indoor env cell: visibility, lights, portals, rendering
- [ ] `CEnvCell::GrabVisibleCells` · 00486f10 · legions 00480770 — needs port: indoor env cell: visibility, lights, portals, rendering

### AtomCollection
- [-] `AtomCollection::AtomCollection` · 00486f70 · legions 004807d0 — constructor (struct init; Rust builds it directly)
- [-] `AtomCollection::~AtomCollection` · 00487060 · legions 004808c0 — destructor (Rust Drop)

### CEnvCell
- [ ] `CEnvCell::BuildAndApplyAtomCollection` · 00487180 · legions 004809e0 — needs port: CEnvCell::BuildAndApplyAtomCollection (only its constants are present)
- [ ] `CEnvCell::ApplyProperty` · 00487270 — needs port: CEnvCell::ApplyProperty (property ids named only)
- [ ] `CEnvCell::ComputeCellGroups` · 004873b0 · legions 00480be0 — needs port: indoor env cell: visibility, lights, portals, rendering
- [ ] `CEnvCell::FlushCellGroups` · 004876a0 · legions 00480ed0 — needs port: indoor env cell: visibility, lights, portals, rendering
- [~] `CEnvCell::flush_cells` · 00487750 · legions 00480f80 — static diffuse bake (base colour + LightIndices); SumBumpMapVectors not yet

## `portal/shared/src/cell/landblock.cpp.c` (24/72)

### LandDefs
- [x] `LandDefs::GetBlockOffset` · 00488140 · legions 00481970 — ac2-physics los (Terrain::los_check)
- [ ] `LandDefs::within_block` · 00488270 · legions 00481aa0 — needs port: landblock id and coordinate arithmetic

### EtherealPhysicsType
- [x] `EtherealPhysicsType::SetPropertyEthereal` · 004882c0 · legions 00481af0 — ac2-physics collision_types.rs: PhysicsTypeMask::set_property (master-table gate, bit below 64) and WeenieTypes::update_from (EtherealWeenieType::Update merge, copy when the source lists are empty)

### IError
- [-] `IError::ReportDataError` · 004882d0 — admin-only dump, save or error report

### Plane
- [x] `Plane::set_height` · 004882f0 · legions 00481b00 — crates/ac2-render/src/terrain/heights.rs (Plane::set_height over terrain heights)

### Random
- [ ] `Random::pseudorand2` · 00488330 · legions 00481b40 — needs port: seeded random for procedural placement

### SceneGroup
- [ ] `SceneGroup::HasRadiusCheck` · 00488390 · legions 00481ba0 — needs port: procedural scenery groups (placement rules)

### CLandBlock
- [ ] `CLandBlock::Init` · 004883b0 · legions 00481bc0 — needs port: landblock streaming: scenery, objects, cells, water, sound points
- [ ] `CLandBlock::SetVertexValue` · 004883c0 · legions 00481bd0 — needs port: landblock streaming: scenery, objects, cells, water, sound points
- [x] `CLandBlock::OnRoad` · 004883d0 · legions 00481be0 — ac2-world ac2_scene/road.rs: the sixteen corner-road cases match Legions 0x00481BE0 (admin 0x004883D0 is the same body); the index is x times the reciprocal pixel length, the intermediates are float-rounded, and the road field comes from the DAT LandDefs bit lengths (CLandBlockData::GetRoad). Out-of-block vertices read as no road where retail reads neighbour memory
- [ ] `CLandBlock::GrabVisibleCells` · 004886f0 · legions 00481f00 — needs port: landblock streaming: scenery, objects, cells, water, sound points
- [ ] `CLandBlock::ReleaseVisibleCells` · 00488720 · legions 00481f30 — needs port: landblock streaming: scenery, objects, cells, water, sound points
- [ ] `CLandBlock::FindInteriorCell` · 00488750 · legions 00481f60 — needs port: landblock streaming: scenery, objects, cells, water, sound points
- [ ] `CLandBlock::InitLBI` · 004887f0 · legions 00482000 — needs port: landblock streaming: scenery, objects, cells, water, sound points

### LandDefs
- [ ] `LandDefs::get_block_gid` · 00488b70 · legions 004823f0 — needs port: landblock id and coordinate arithmetic
- [x] `LandDefs::GetBlockID` · 00488bc0 · legions 00482440 — crates/ac2-physics/src/los/land.rs LandDefs::GetBlockID neighbour id

### CLandBlockMesh
- [ ] `CLandBlockMesh::GetLandCell` · 00488c10 · legions 00482490 — needs port: landblock terrain mesh build and water

### CLandBlock
- [-] `CLandBlock::RecoverFromFreelist` · 00488c60 · legions 004824e0 — retail free-object pool (DBOCache exception, AGENTS.md)
- [-] `CLandBlock::ClearForFreelist` · 00488d00 · legions 00482580 — retail free-object pool (DBOCache exception, AGENTS.md)
- [ ] `CLandBlock::GenerateMesh` · 00488d90 · legions 00482610 — needs port: landblock streaming: scenery, objects, cells, water, sound points
- [ ] `CLandBlock::GetNearestPixel` · 00488e10 · legions 00482690 — needs port: landblock streaming: scenery, objects, cells, water, sound points
- [ ] `CLandBlock::InitDynamicObjects` · 00488e70 · legions 004826f0 — needs port: landblock streaming: scenery, objects, cells, water, sound points
- [ ] `CLandBlock::ReleaseDynamicObjects` · 00488ec0 · legions 00482740 — needs port: landblock streaming: scenery, objects, cells, water, sound points
- [x] `CLandBlock::InqClearanceInfo` · 00488f10 · legions 00482790
- [ ] `CLandBlock::InitializeQuadtree` · 00488f70 · legions 004827f0 — needs port: landblock streaming: scenery, objects, cells, water, sound points
- [ ] `CLandBlock::ClearEncounterList` · 00489110 · legions 00482990 — needs port: landblock streaming: scenery, objects, cells, water, sound points
- [x] `CLandBlock::ObjectCollides` · 00489160 · legions 004829e0
- [x] `CLandBlock::ProcFrill` · 004891c0 · legions 00482a40 — crates/ac2-world/src/ac2_landblock_info.rs frill placements (CLandBlock::ProcFrill)
- [~] `CLandBlock::CheckRadius` · 00489230 · legions 00482ab0 — ported; CLandCell::CheckRadius per-cell test not re-verified
- [ ] `CLandBlock::CullObjects` · 00489520 · legions 00482d70 — needs port: landblock streaming: scenery, objects, cells, water, sound points
- [x] `CLandBlock::TraceRay` · 00489820 · legions 00483060 — ac2-physics los (Terrain::los_check)
- [ ] `CLandBlock::InqObjects` · 00489a70 · legions 004832b0 — needs port: landblock streaming: scenery, objects, cells, water, sound points

### SmartArray<CLandBlock::RenderEntityInfo>
- [-] `SmartArray<CLandBlock::RenderEntityInfo>::~SmartArray<CLandBlock::RenderEntityInfo>` · 00489b10 · legions 00483350 — destructor (Rust Drop)

### DBObjGrabber<CWaterDesc>
- [-] `DBObjGrabber<CWaterDesc>::DBObjGrabber<CWaterDesc>` · 00489b60 · legions 004833a0 — DBO cache grabber (DBOCache exception, AGENTS.md)

### DBObjGrabber<BlockMap>
- [-] `DBObjGrabber<BlockMap>::DBObjGrabber<BlockMap>` · 00489ba0 · legions 004833e0 — DBO cache grabber (DBOCache exception, AGENTS.md)

### LandDefs
- [x] `LandDefs::GetBlockID` · 00489d70 · legions 00483680 — ac2-physics los (Terrain::los_check)

### EtherealWeenieType
- [-] `EtherealWeenieType::EtherealWeenieType` · 00489de0 · legions 004836f0 — constructor (struct init; Rust builds it directly)

### CLandBlock
- [-] `CLandBlock::CLandBlock` · 00489e40 · legions 00483750 — constructor (struct init; Rust builds it directly)
- [ ] `CLandBlock::DestroyStaticObjects` · 00489f70 · legions 00483880 — needs port: landblock streaming: scenery, objects, cells, water, sound points
- [~] `CLandBlock::GetPhysicsMaterial` · 0048a060 — Nearest vertex in the block's material table (LandDefs::GetNearestPixel). Road or terrain is chosen per vertex (road where the vertex has a road band), not by CLandBlock::OnRoad's polygon test. Not verified against retail on a road edge.
- [x] `CLandBlock::GetWaterPhysicsMaterial` · 0048a0e0 · legions 004839f0 — Physics materials: ac2-world reads each vertex's surface or road band material (LScapeSurfaceType band by pitch) and each water type's material; ac2-physics Terrain returns the land vertex, block water and interior cell materials, read by PhysicsBody::contact_materials.
- [x] `CLandBlock::CheckGroupPlacement` · 0048a160 · legions 00483a50
- [ ] `CLandBlock::RemoveDynamicObject` · 0048a330 · legions 00483c20 — needs port: landblock streaming: scenery, objects, cells, water, sound points

### SmartArray<AmbientSoundPoint>
- [-] `SmartArray<AmbientSoundPoint>::AddToEnd` · 0048a390 · legions 00483c80 — SmartArray container operation (Vec replaces it)

### CELLARRAY
- [-] `CELLARRAY::CELLARRAY` · 0048a3f0 · legions 00483ce0 — constructor (struct init; Rust builds it directly)

### CLandBlock
- [ ] `CLandBlock::Clear` · 0048a420 · legions 00483d10 — needs port: landblock streaming: scenery, objects, cells, water, sound points
- [x] `CLandBlock::InitCellConnections` · 0048a460 · legions 00483d50 — ac2-physics find_cells.rs, terrain.rs: cell search through land cells, touching cells and portals
- [x] `CLandBlock::CreateAmbientSoundPoints` · 0048a620 · legions 00483f10 — ac2-audio/ac2-client: ambient environments, groove
- [ ] `CLandBlock::DoIntersection` · 0048a7e0 · legions 00484090 — needs port: landblock streaming: scenery, objects, cells, water, sound points

### PSmartArray<InstanceID>
- [x] `PSmartArray<InstanceID>::StreamPack` · 0048a9c0

### CLandBlock
- [ ] `CLandBlock::~CLandBlock` · 0048ab10 · legions 004843c0 — needs port: CLandBlock destructor unload (world hash and resident state)
- [x] `CLandBlock::AddEntityToClearList` · 0048abb0 · legions 00484460
- [x] `CLandBlock::StartSceneGroup` · 0048ace0 · legions 00484590
- [x] `CLandBlock::ProcEntity` · 0048ad70 · legions 00484620
- [~] `CLandBlock::CreateStaticObjects` · 0048ae10 · legions 004846c0 — ac2-physics loader: each landblock entity's blocking mask from its ethereal properties (ac2_db::ac2_ethereal_mask). Retail's SetProperty(0x40000038) is a no-op (bit 0x40000038 is past 63) and the SetCollisionState rewrite of the same mask is the identity. Not ported: the ObjectClearance group/weenie filter and the scene/lbi object split
- [ ] `CLandBlock::AddDynamicObject` · 0048afe0 · legions 00484890 — needs port: landblock streaming: scenery, objects, cells, water, sound points
- [ ] `CLandBlock::AddToBrushList` · 0048b010 · legions 004848c0 — needs port: landblock streaming: scenery, objects, cells, water, sound points
- [ ] `CLandBlock::GetAmbientSoundPoints` · 0048b170 · legions 00484a20 — needs port: landblock streaming: scenery, objects, cells, water, sound points
- [ ] `CLandBlock::DoDetect` · 0048b190 · legions 00484a40 — needs port: landblock streaming: scenery, objects, cells, water, sound points

### LScapeManager
- [x] `LScapeManager::GetLandBlock` · 0048b3d0 · legions 00484c80 — crates/ac2-physics/src/los/land.rs LScapeManager::GetLandBlock residency query

### CLandBlock
- [ ] `CLandBlock::GetVisible` · 0048b430 · legions 00484ce0 — needs port: landblock streaming: scenery, objects, cells, water, sound points
- [x] `CLandBlock::GenerateLBIObjectList` · 0048b450 · legions 00484d00
- [x] `CLandBlock::ProcScene` · 0048b4c0 · legions 00484d70
- [x] `CLandBlock::ProcGroup` · 0048b700 · legions 00484f60
- [ ] `CLandBlock::RemoveLandCellsFromHash` · 0048b990 · legions 004851f0 — needs port: landblock streaming: scenery, objects, cells, water, sound points
- [ ] `CLandBlock::GatherObjects` · 0048ba30 · legions 00485290 — needs port: CLandBlock::GatherObjects (object gather for culling and shadow map)
- [x] `CLandBlock::TraceRayCrossBlock` · 0048bcd0 · legions 00485500 — ac2-physics los (Terrain::los_check)
- [x] `CLandBlock::GenerateSceneObjectList` · 0048bfe0 · legions 00485810 — crates/ac2-world/src/ac2_scene/generate.rs procedural scenery and grass
- [ ] `CLandBlock::AddLandCellsToHash` · 0048c180 · legions 004859b0 — needs port: landblock streaming: scenery, objects, cells, water, sound points
- [ ] `CLandBlock::InitStaticObjects` · 0048c240 · legions 00485a60 — needs port: landblock streaming: scenery, objects, cells, water, sound points
- [ ] `CLandBlock::Regenerate` · 0048c2f0 · legions 00485b10 — needs port: landblock streaming: scenery, objects, cells, water, sound points
- [ ] `CLandBlock::Refresh` · 0048c320 · legions 00485b40 — needs port: landblock streaming: scenery, objects, cells, water, sound points

## `portal/shared/src/cell/landblockdata.cpp.c` (9/31)

### SurfaceInfo
- [-] `SurfaceInfo::operator=` · 0048ee70 · legions 00487c90 — copy assignment of plain data (Rust clone/move)
- [x] `SurfaceInfo::SetToDefault` · 0048eef0 · legions 00487d10

### CLandBlockData
- [ ] `CLandBlockData::GetSubDataIDs` · 0048ef60 — needs port: landblock data: heights, terrain, scene, encounters (sub-object DID list for async sub-object resolution, AGENTS.md)

### SurfaceInfo
- [-] `SurfaceInfo::SurfaceInfo` · 0048ef90 — constructor (struct init; Rust builds it directly)

### CLandBlockData
- [ ] `CLandBlockData::SetHeight` · 0048efc0 · legions 00487de0 — needs port: landblock data: heights, terrain, scene, encounters
- [ ] `CLandBlockData::SetTerrain` · 0048efe0 · legions 00487e00 — needs port: landblock data: heights, terrain, scene, encounters
- [ ] `CLandBlockData::SetScene` · 0048f040 · legions 00487e60 — needs port: landblock data: heights, terrain, scene, encounters
- [ ] `CLandBlockData::SetEncounter` · 0048f0a0 · legions 00487ec0 — needs port: landblock data: heights, terrain, scene, encounters
- [ ] `CLandBlockData::SetRoad` · 0048f100 · legions 00487f20 — needs port: landblock data: heights, terrain, scene, encounters
- [x] `CLandBlockData::GetTerrain` · 0048f160 · legions 00487f80
- [ ] `CLandBlockData::GetScene` · 0048f1b0 · legions 00487fd0 — needs port: landblock data: heights, terrain, scene, encounters
- [x] `CLandBlockData::GetColor` · 0048f200 · legions 00488020
- [x] `CLandBlockData::GetRoad` · 0048f250 · legions 00488070
- [ ] `CLandBlockData::InqAsyncLoadDIDs` · 0048f2a0 · legions 004880c0 — needs port: landblock data: heights, terrain, scene, encounters
- [x] `CLandBlockData::StreamPack` · 0048f310 · legions 00488130 — crates/ac2-world/src/ac2_scene/block.rs CLandBlockData::StreamPack

### DBObjGrabber<CSurfaceDesc>
- [-] `DBObjGrabber<CSurfaceDesc>::DBObjGrabber<CSurfaceDesc>` · 0048f3d0 · legions 004881f0 — DBO cache grabber (DBOCache exception, AGENTS.md)

### CLandBlockData
- [ ] `CLandBlockData::~CLandBlockData` · 0048f560 · legions 00488380 — needs port: CLandBlockData destructor unload (world hash and resident state)
- [-] `CLandBlockData::GetDBOType` · 0048f600 — DBO type tag (Rust typed decoders)
- [x] `CLandBlockData::GetNeighborIsSWtoNECut` · 0048f610 · legions 00488430
- [x] `CLandBlockData::GetNeighborHeightValue` · 0048f720 · legions 00488540
- [x] `CLandBlockData::GetNormalFromTriangle` · 0048f830 · legions 00488650
- [x] `CLandBlockData::CachePackValues` · 0048f980 · legions 004887a0
- [ ] `CLandBlockData::InitLoad` · 0048fa90 — needs port: landblock data: heights, terrain, scene, encounters (post-load init)

### PSmartArray<unsigned_char>
- [ ] `PSmartArray<unsigned_char>::StreamPack` · 0048faa0 — needs port: DAT counted list (DAT decode)

### (free functions)
- [ ] `STREAMPACK` · 0048fb40 · legions 00488960 — no call site in the Legions wlib (scan: host::tests::efun_coverage); lower priority

### SmartArray<Vector>
- [-] `SmartArray<Vector>::SetNElements` · 0048fbd0 · legions 004889f0 — SmartArray container operation (Vec replaces it)

### CLandBlockData
- [-] `CLandBlockData::CLandBlockData` · 0048fc00 — constructor (struct init; Rust builds it directly)
- [~] `CLandBlockData::CacheFarValues` · 0048fe10 · legions 00488c30 — vertex normals and terrain SurfaceInfo; not far colours or water
- [-] `CLandBlockData::Allocate` · 00490770 — retail free-object pool (DBOCache exception, AGENTS.md)
- [-] `CLandBlockData::Allocator` · 00490780 — retail free-object pool (DBOCache exception, AGENTS.md)
- [~] `CLandBlockData::CacheNearValues` · 004907a0 · legions 00489500 — road SurfaceInfo; not physics materials or scene DIDs

## `portal/shared/src/cell/landblockinfo.cpp.c` (2/13)

### CLandBlockInfo
- [ ] `CLandBlockInfo::GetSubDataIDs` · 004913b0 · legions 0048a0b0 — needs port: landblock interior info: cells, lights, properties (sub-object DID list for async sub-object resolution, AGENTS.md)
- [x] `CLandBlockInfo::ApplyProperty` · 00491430 · legions 0048a130 — crates/ac2-world/src/ac2_landblock_interior.rs interior properties (CLandBlockInfo::ApplyProperty)
- [-] `CLandBlockInfo::CLandBlockInfo` · 00491530 — constructor (struct init; Rust builds it directly)
- [-] `CLandBlockInfo::GetDBOType` · 00491610 — DBO type tag (Rust typed decoders)
- [-] `CLandBlockInfo::Destroy` · 00491620 · legions 0048a320 — DBO release (Drop frees on last drop, AGENTS.md)
- [ ] `CLandBlockInfo::InqVisibleCells` · 004916d0 · legions 0048a3d0 — needs port: CLandBlockInfo::InqVisibleCells (cells drawn from outside)
- [x] `CLandBlockInfo::StreamPack` · 00491720 — crates/ac2-world/src/ac2_landblock_info.rs CLandBlockInfo::StreamPack
- [ ] `CLandBlockInfo::ApplyPropertyCollection` · 00491810 · legions 0048a510 — needs port: landblock interior info: cells, lights, properties

### EntityLinkDesc
- [-] `EntityLinkDesc::operator=` · 004918c0 · legions 0048a5c0 — copy assignment of plain data (Rust clone/move)

### CLandBlockInfo
- [-] `CLandBlockInfo::Allocate` · 00491a20 — retail free-object pool (DBOCache exception, AGENTS.md)
- [-] `CLandBlockInfo::Allocator` · 00491a30 — retail free-object pool (DBOCache exception, AGENTS.md)
- [ ] `CLandBlockInfo::InitLoad` · 00491a50 · legions 0048a750 — needs port: landblock interior info: cells, lights, properties (post-load init)
- [-] `CLandBlockInfo::FromFileNode` · 00491b10 — text source format (admin tools)

## `portal/shared/src/cell/landblockmesh.cpp.c` (4/15)

### CLandBlockMesh
- [-] `CLandBlockMesh::CLandBlockMesh` · 0048c390 · legions 00485bb0 — constructor (struct init; Rust builds it directly)
- [ ] `CLandBlockMesh::Clear` · 0048c440 · legions 00485c60 — needs port: landblock terrain mesh build and water
- [ ] `CLandBlockMesh::HasWater` · 0048c490 · legions 00485cb0 — needs port: landblock terrain mesh build and water
- [ ] `CLandBlockMesh::GetWaterIndex` · 0048c4a0 · legions 00485cc0 — needs port: landblock terrain mesh build and water
- [ ] `CLandBlockMesh::GetWaterPlane` · 0048c4b0 · legions 00485cd0 — needs port: landblock terrain mesh build and water
- [x] `CLandBlockMesh::InqWaterPlane` · 0048c4d0 · legions 00485cf0
- [ ] `CLandBlockMesh::SetVertexValueInternal` · 0048c510 · legions 00485d30 — needs port: landblock terrain mesh build and water
- [ ] `CLandBlockMesh::~CLandBlockMesh` · 0048c690 · legions 00485eb0 — needs port: CLandBlockMesh destructor unload (world hash and resident state)
- [ ] `CLandBlockMesh::Init` · 0048c6b0 · legions 00485ed0 — needs port: landblock terrain mesh build and water
- [ ] `CLandBlockMesh::SetMinMax` · 0048c7d0 · legions 00485ff0 — needs port: landblock terrain mesh build and water
- [x] `CLandBlockMesh::ConstructVertices` · 0048ca50 · legions 00486270
- [x] `CLandBlockMesh::SetPolygon` · 0048cb40 · legions 00486360
- [ ] `CLandBlockMesh::Refresh` · 0048cbf0 · legions 00486410 — needs port: landblock terrain mesh build and water
- [x] `CLandBlockMesh::ConstructPolygons` · 0048cca0 · legions 004864c0
- [ ] `CLandBlockMesh::GenerateMesh` · 0048d0a0 · legions 004868c0 — needs port: landblock terrain mesh build and water

## `portal/shared/src/cell/landblockquadtree.cpp.c` (0/22)

### QuadTree<ICPhysicsObj_*>
- [ ] `QuadTree<ICPhysicsObj_*>::GetChildCenter` · 00491e60 · legions 0048a810 — needs port: landblock object quadtree generation and culling
- [ ] `QuadTree<ICPhysicsObj_*>::DestroyTree` · 00491ef0 · legions 0048a8a0 — needs port: landblock object quadtree generation and culling
- [ ] `QuadTree<ICPhysicsObj_*>::GetChildQuad` · 00491f50 · legions 0048a900 — needs port: landblock object quadtree generation and culling

### CLandBlockQuadTree
- [ ] `CLandBlockQuadTree::UpdateMinMax` · 00491ff0 · legions 0048a9a0 — needs port: landblock object quadtree culling and draw
- [ ] `CLandBlockQuadTree::DoCull` · 004920e0 · legions 0048aa90 — needs port: landblock object quadtree culling and draw
- [ ] `CLandBlockQuadTree::Cull` · 004922b0 · legions 0048ac60 — needs port: landblock object quadtree culling and draw

### QuadTree<ICPhysicsObj_*>
- [ ] `QuadTree<ICPhysicsObj_*>::Reset` · 004922e0 · legions 0048ac90 — needs port: landblock object quadtree generation and culling
- [-] `QuadTree<ICPhysicsObj_*>::~QuadTree<ICPhysicsObj_*>` · 00492310 · legions 0048acc0 — destructor (Rust Drop)

### CLandBlockQuadTree
- [-] `CLandBlockQuadTree::CLandBlockQuadTree` · 004923c0 · legions 0048ad70 — constructor (struct init; Rust builds it directly)
- [-] `CLandBlockQuadTree::~CLandBlockQuadTree` · 00492400 · legions 0048adb0 — destructor (Rust Drop)

### QuadTree<ICPhysicsObj_*>::QuadNode
- [-] `QuadTree<ICPhysicsObj_*>::QuadNode::~QuadNode` · 00492410 · legions 0048adc0 — destructor (Rust Drop)

### SmartArray<_STL::pair<ICPhysicsObj*,BBox>_>
- [-] `SmartArray<_STL::pair<ICPhysicsObj*,BBox>_>::operator=` · 00492590 · legions 0048af40 — copy assignment of plain data (Rust clone/move)

### CLandBlockQuadTree
- [ ] `CLandBlockQuadTree::CreateNode` · 00492650 · legions 0048b000 — needs port: landblock object quadtree culling and draw
- [ ] `CLandBlockQuadTree::DoDraw` · 00492680 · legions 0048b030 — needs port: landblock object quadtree culling and draw

### QuadTree<ICPhysicsObj_*>
- [ ] `QuadTree<ICPhysicsObj_*>::CreateNode` · 00492750 · legions 0048b100 — needs port: landblock object quadtree generation and culling
- [ ] `QuadTree<ICPhysicsObj_*>::Initialize` · 00492780 · legions 0048b130 — needs port: landblock object quadtree generation and culling

### SmartArray<_STL::pair<ICPhysicsObj_*,BBox>_>
- [-] `SmartArray<_STL::pair<ICPhysicsObj_*,BBox>_>::AddToEnd` · 004927b0 · legions 0048b160 — STL internals

### QuadTree<ICPhysicsObj_*>
- [ ] `QuadTree<ICPhysicsObj_*>::DoGenerate` · 00492820 · legions 0048b1d0 — needs port: landblock object quadtree generation and culling

### CLandBlockQuadTree
- [ ] `CLandBlockQuadTree::Draw` · 004929e0 · legions 0048b390 — needs port: landblock object quadtree culling and draw

### QuadTree<ICPhysicsObj_*>
- [ ] `QuadTree<ICPhysicsObj_*>::AddObject` · 00492a00 · legions 0048b3b0 — needs port: landblock object quadtree generation and culling
- [ ] `QuadTree<ICPhysicsObj_*>::Generate` · 00492a50 · legions 0048b400 — needs port: landblock object quadtree generation and culling

### CLandBlockQuadTree
- [ ] `CLandBlockQuadTree::Generate` · 00492b60 · legions 0048b510 — needs port: landblock object quadtree culling and draw

## `portal/shared/src/cell/landcell.cpp.c` (20/38)

### (free functions)
- [ ] `AreFloatEqual` · 0047d8e0 · legions 00477910 — needs port: float comparison helper

### Position
- [ ] `Position::adjust_to_outside` · 0047d900 · legions 00477930 — needs port: position math

### PositionOffset
- [x] `PositionOffset::GetOffset` · 0047d920 · legions 00489c10

### Ray
- [x] `Ray::Ray` · 0047d990 · legions 00477950 — ac2-physics los (Terrain::los_check)
- [x] `Ray::InitFromOffset` · 0047d9c0 · legions 00477980 — ac2-physics los (Terrain::los_check)
- [x] `Ray::InitLineSegment` · 0047daa0 · legions 00477a60 — ac2-physics los (Terrain::los_check)
- [x] `Ray::GetEndPoint` · 0047dae0 · legions 00477aa0 — ac2-physics los (Terrain::los_check)

### LOSParameters
- [x] `LOSParameters::SetImpactNormal` · 0047db20 · legions 00477ae0 — ac2-physics los (Terrain::los_check)

### Plane
- [x] `Plane::Plane` · 0047db50 · legions 00477b10 — ac2-physics shapes/sphere.rs (asm float order; Legions identical)

### SPHEREPATH
- [x] `SPHEREPATH::add_offset_to_check_pos` · 0047db90 · legions 00477b50

### COLLISIONINFO
- [x] `COLLISIONINFO::set_collision_normal` · 0047dbe0 · legions 00477ba0

### CLandCell
- [-] `CLandCell::CLandCell` · 0047dc60 · legions 00477c20 — constructor (struct init; Rust builds it directly)
- [ ] `CLandCell::~CLandCell` · 0047dca0 · legions 00477c60 — needs port: CLandCell destructor unload (world hash and resident state)
- [ ] `CLandCell::SetPolygon` · 0047dcc0 · legions 00477c80 — needs port: landscape cell (terrain polygon) queries and visibility
- [x] `CLandCell::InqWaterPlane` · 0047dce0
- [x] `CLandCell::find_terrain_poly` · 0047dd00 · legions 00477cc0
- [x] `CLandCell::find_terrain_plane` · 0047dd60 · legions 00477d20 — crates/ac2-world/src/ac2_scene/geometry.rs CLandCell::find_terrain_plane
- [x] `CLandCell::point_in_cell` · 0047ddc0
- [ ] `CLandCell::GetViewerDistance` · 0047de50 — needs port: landscape cell (terrain polygon) queries and visibility
- [ ] `CLandCell::GrabVisibleCells` · 0047dea0 — needs port: landscape cell (terrain polygon) queries and visibility
- [ ] `CLandCell::ReleaseVisibleCells` · 0047deb0 — needs port: landscape cell (terrain polygon) queries and visibility
- [ ] `CLandCell::UpdateCellMaterial` · 0047dec0 — needs port: landscape cell (terrain polygon) queries and visibility
- [x] `CLandCell::GetPhysicsMaterialDID` · 0047df20 — Physics materials: ac2-world reads each vertex's surface or road band material (LScapeSurfaceType band by pitch) and each water type's material; ac2-physics Terrain returns the land vertex, block water and interior cell materials, read by PhysicsBody::contact_materials.

### Ray
- [x] `Ray::operator=` · 0047df40 · legions 00477f60 — ac2-physics los (Terrain::los_check)

### CLandCell
- [-] `CLandCell::Allocate` · 0047df80 — retail free-object pool (DBOCache exception, AGENTS.md)
- [ ] `CLandCell::CheckRadius` · 0047dfe0 · legions 00478000 — needs port: landscape cell (terrain polygon) queries and visibility
- [x] `CLandCell::AddOutsideCell` · 0047e160 · legions 00478180 — ac2-physics find_cells.rs, terrain.rs: cell search through land cells, touching cells and portals
- [ ] `CLandCell::handle_move_restriction` · 0047e2c0 · legions 004782e0 — needs port: landscape cell (terrain polygon) queries and visibility

### DBObjGrabber<CLandBlockData>
- [-] `DBObjGrabber<CLandBlockData>::DBObjGrabber<CLandBlockData>` · 0047e400 · legions 00478420 — DBO cache grabber (DBOCache exception, AGENTS.md)

### LOSParameters
- [x] `LOSParameters::SetCurrentRay` · 0047e520 · legions 00478560 — ac2-physics los (Terrain::los_check)

### CLandCell
- [x] `CLandCell::FindOutsideCells` · 0047e570 · legions 004785b0 — ac2-physics find_cells.rs, terrain.rs: cell search through land cells, touching cells and portals
- [~] `CLandCell::find_env_collisions` · 0047e670 — unloaded-landblock move restriction not ported
- [x] `CLandCell::TraceRay` · 0047e7d0 · legions 00478810 — ac2-physics los (Terrain::los_check)

### SmartArray<Plane>
- [-] `SmartArray<Plane>::AddToEnd` · 0047ecb0 · legions 00478ce0 — SmartArray container operation (Vec replaces it)

### CLandCell
- [ ] `CLandCell::GetCellPlanes` · 0047ed10 — needs port: landscape cell (terrain polygon) queries and visibility

### LScapeManager
- [ ] `LScapeManager::GetLandCell` · 0047ee60 · legions 00478e90 — needs port: landscape scene, surfaces, materials and vertex colours

### CLandCell
- [ ] `CLandCell::Get` · 0047eea0 · legions 00478ed0 — needs port: landscape cell (terrain polygon) queries and visibility
- [x] `CLandCell::Trace` · 0047eed0 — ac2-physics los (Terrain::los_check)

## `portal/shared/src/cell/lightinfo.cpp.c` (3/26)

### PackObj
- [x] `PackObj::UNPACK_TYPE` · 0048d160 · legions 00486970 — crates/ac2-script/src/bytestream/mod.rs PackObj::UNPACK_TYPE check
- [ ] `PackObj::UnPack` · 0048d190 · legions 004869a0 — needs port: world content (PackObj) (DAT decode)

### (free functions)
- [-] `STREAMPACK_SIZEOF` · 0048d1c0 · legions 004869d0 — retail StreamPack macro dispatch (Rust decoders)

### CLightInfo
- [ ] `CLightInfo::GetLitIndices` · 0048d220 · legions 00486b50 — needs port: per-cell light lists (RLE cell sets)
- [ ] `CLightInfo::GetSunLitIndices` · 0048d290 · legions 00486bc0 — needs port: per-cell light lists (RLE cell sets)

### CLightInfo::LightCellInfo
- [ ] `CLightInfo::LightCellInfo::LightCellInfo` · 0048d300 — needs port: per-cell light lists (RLE cell sets)
- [-] `CLightInfo::LightCellInfo::~LightCellInfo` · 0048d310 · legions 00486c40 — destructor (Rust Drop)

### RLE
- [x] `RLE::Decode` · 0048d3d0 · legions 00486d00 — crates/ac2-world/src/ac2_light_info.rs RLE::Decode
- [-] `RLE::Encode` · 0048d4c0 · legions 00486df0 — write or admin-only FileNode import

### PFileNode
- [ ] `PFileNode::GetLeafAsUInt32` · 0048d660 · legions 00486f90 — needs port: FileNode accessors (used by FromFileNode loaders; confirm DAT path)

### SmartArray<short>
- [-] `SmartArray<short>::SetNElements` · 0048d710 · legions 00487040 — SmartArray container operation (Vec replaces it)

### CLightInfo
- [-] `CLightInfo::LoadFromFile` · 0048d740 — text source loader, admin build only
- [ ] `CLightInfo::StreamPackCell` · 0048d870 · legions 00487070 — needs port: per-cell light lists (RLE cell sets) (DAT decode)
- [-] `CLightInfo::Destroy` · 0048da40 · legions 00487240 — DBO release (Drop frees on last drop, AGENTS.md)
- [ ] `CLightInfo::GetNumLights` · 0048db70 · legions 00487370 — needs port: per-cell light lists (RLE cell sets)
- [ ] `CLightInfo::GetLight` · 0048dbc0 · legions 004873c0 — needs port: per-cell light lists (RLE cell sets)
- [x] `CLightInfo::StreamPack` · 0048dc30 · legions 00487430 — crates/ac2-world/src/ac2_light_info.rs CLightInfo::StreamPack order
- [ ] `CLightInfo::~CLightInfo` · 0048e110 · legions 00487910 — needs port: CLightInfo destructor unload (world hash and resident state)
- [-] `CLightInfo::GetDBOType` · 0048e1c0 — DBO type tag (Rust typed decoders)
- [-] `CLightInfo::CellInfoFromFileNode` · 0048e290 — write or admin-only FileNode import

### PHashTable<unsigned_long,PSmartArray<unsigned_long>_>
- [-] `PHashTable<unsigned_long,PSmartArray<unsigned_long>_>::StreamPack` · 0048e5d0 — hash table container; Rust map replaces it

### CLightInfo
- [-] `CLightInfo::CLightInfo` · 0048e7d0 — constructor (struct init; Rust builds it directly)
- [-] `CLightInfo::LightsFromFileNode` · 0048e840 — write or admin-only FileNode import
- [-] `CLightInfo::Allocate` · 0048ed60 — retail free-object pool (DBOCache exception, AGENTS.md)
- [-] `CLightInfo::Allocator` · 0048ed70 — retail free-object pool (DBOCache exception, AGENTS.md)
- [-] `CLightInfo::FromFileNode` · 0048ed90 — text source format (admin tools)

## `portal/shared/src/cell/objcell.cpp.c` (10/53)

### DBObj
- [-] `DBObj::SaveToFile` · 0047b860 — admin-only dump, save or error report

### StreamDBObj
- [ ] `StreamDBObj::StreamPack` · 0047b870 · legions 00475960 — needs port: StreamDBObj::StreamPack (DID base read; only a net-field comment cites it)
- [-] `StreamDBObj::GetPackSize` · 0047b8c0 — write path (client reads DATs only)
- [-] `StreamDBObj::Pack` · 0047b8f0 — write path (client reads DATs only)
- [ ] `StreamDBObj::UnPack` · 0047b930 — needs port: world content (StreamDBObj) (DAT decode)

### Cylsphere
- [-] `Cylsphere::Cylsphere` · 0047b950 — constructor (struct init; Rust builds it directly)

### FrillEntry
- [-] `FrillEntry::FrillEntry` · 0047b970 — constructor (struct init; Rust builds it directly)

### CObjCell
- [ ] `CObjCell::Get` · 0047b9c0 · legions 00475af0 — needs port: object cells: membership, detection voyeurs, frills, shadows
- [x] `CObjCell::GetVisible` · 0047ba10 · legions 00475b40 — crates/ac2-physics/src/cells.rs CObjCell::GetVisible resident cell
- [ ] `CObjCell::SendObjectDetection` · 0047ba40 · legions 00475b70 — needs port: object cells: membership, detection voyeurs, frills, shadows
- [ ] `CObjCell::SetObjectMaintainer` · 0047bb10 · legions 00475c40 — needs port: object cells: membership, detection voyeurs, frills, shadows
- [ ] `CObjCell::GetViewerDistance` · 0047bb20 — needs port: object cells: membership, detection voyeurs, frills, shadows
- [x] `CObjCell::FindCells` · 0047bb40 · legions 00475c70 — ac2-physics find_cells.rs, terrain.rs: cell search through land cells, touching cells and portals
- [x] `CObjCell::FindCellsLocal` · 0047bbd0 · legions 00475d00 — ac2-physics find_cells.rs, terrain.rs: cell search through land cells, touching cells and portals
- [x] `CObjCell::FindCellsLocal` · 0047bcb0 · legions 00475de0 — ac2-physics find_cells.rs, terrain.rs: cell search through land cells, touching cells and portals
- [x] `CObjCell::FindCells` · 0047bd90 · legions 00475ec0 — ac2-physics find_cells.rs, terrain.rs: cell search through land cells, touching cells and portals
- [x] `CObjCell::LOSCheck` · 0047be70 · legions 00475fa0 — ac2-physics los (Terrain::los_check)

### _STL::pair<ICPhysicsObj_*,BBox>
- [-] `_STL::pair<ICPhysicsObj_*,BBox>::~pair<ICPhysicsObj_*,BBox>` · 0047bf50 — STL internals

### CObjCell
- [-] `CObjCell::ClearForFreelist` · 0047c120 · legions 004761f0 — retail free-object pool (DBOCache exception, AGENTS.md)
- [ ] `CObjCell::MakeFrillList` · 0047c150 · legions 00476220 — needs port: object cells: membership, detection voyeurs, frills, shadows
- [ ] `CObjCell::RemoveAllFrillObjects` · 0047c320 · legions 004763f0 — needs port: object cells: membership, detection voyeurs, frills, shadows
- [~] `CObjCell::RemoveShadowObject` · 0047c330 · legions 00476400 — ac2-physics terrain.rs insert_object/remove_object/list_in: a server object is listed in the shadow lists of the cells its shadow box touches (static and dynamic lists merged into one per cell); CObjCell::m_objectList and the voyeur and detection bookkeeping (UpdateAllVoyeur, CLandBlock::AddDynamicObject) are not ported
- [~] `CObjCell::InitDynamicObjects` · 0047c390 · legions 00476460 — ac2-client: CellManager's block changes
- [ ] `CObjCell::DoIntersection` · 0047c3f0 · legions 004764c0 — needs port: object cells: membership, detection voyeurs, frills, shadows
- [x] `CObjCell::FindCellsLocal` · 0047c440 · legions 00476510 — ac2-physics find_cells.rs, terrain.rs: cell search through land cells, touching cells and portals
- [x] `CObjCell::FindCellsLocal` · 0047c460 · legions 00476530 — ac2-physics find_cells.rs, terrain.rs: cell search through land cells, touching cells and portals
- [~] `CObjCell::find_obj_collisions` · 0047c510 · legions 004765e0 — ac2-physics objects.rs find_object_collisions: the mover's own entry is skipped (object_info.object test), server objects are tested in the dynamic shadow list, FX objects are ethereal (DoCreateObjectFX); the admin 0x250 branch and the step/missile/temp-ignore lists are not ported
- [~] `CObjCell::TraceRayObjects` · 0047c560 · legions 00476630 — ac2-physics los: static objects only (no dynamic shadow list, blocking masks or exact drawing rep)

### SmartArray<ObjectClearance>
- [-] `SmartArray<ObjectClearance>::~SmartArray<ObjectClearance>` · 0047c600 · legions 004766c0 — SmartArray container destructor (Vec drop)

### IntrusiveHashTable<InstanceID,HashTableData<InstanceID,DetectionVoyeur>_*>
- [-] `IntrusiveHashTable<InstanceID,HashTableData<InstanceID,DetectionVoyeur>_*>::~IntrusiveHashTable<InstanceID,HashTableData<InstanceID,DetectionVoyeur>_*>` · 0047c640 · legions 00476700 — hash table container; Rust map replaces it

### DetectionReportType
- [ ] `DetectionReportType::StreamPack` · 0047c8c0 — needs port: object detection reports (DAT decode)

### CObjCell
- [-] `CObjCell::CObjCell` · 0047c8f0 · legions 004769d0 — constructor (struct init; Rust builds it directly)
- [ ] `CObjCell::GetCellID` · 0047c9d0 · legions 00476ac0 — needs port: object cells: membership, detection voyeurs, frills, shadows
- [ ] `CObjCell::~CObjCell` · 0047c9e0 · legions 00476ad0 — needs port: CObjCell destructor unload (world hash and resident state)
- [ ] `CObjCell::InvalidateShadowObjects` · 0047ca70 · legions 00476b60 — needs port: object cells: membership, detection voyeurs, frills, shadows
- [~] `CObjCell::ReleaseDynamicObjects` · 0047cb40 · legions 00476c30 — ac2-client: CellManager's block changes
- [~] `CObjCell::find_obj_collisions` · 0047cb70 · legions 00476c60 — ac2-physics objects.rs find_object_collisions: the mover's own entry is skipped (object_info.object test), server objects are tested in the dynamic shadow list, FX objects are ethereal (DoCreateObjectFX); the admin 0x250 branch and the step/missile/temp-ignore lists are not ported

### SmartArray<BBox>
- [-] `SmartArray<BBox>::operator=` · 0047cbb0 · legions 00476ca0 — copy assignment of plain data (Rust clone/move)

### SmartArray<Cylsphere>
- [-] `SmartArray<Cylsphere>::operator=` · 0047cc40 · legions 00476d30 — copy assignment of plain data (Rust clone/move)

### ObjectClearance
- [x] `ObjectClearance::operator=` · 0047cd90 · legions 00476e80

### CObjCell
- [x] `CObjCell::AddTouchingCell` · 0047ce60 · legions 00476f50 — ac2-physics find_cells.rs, terrain.rs: cell search through land cells, touching cells and portals
- [~] `CObjCell::AddShadowObject` · 0047cfc0 · legions 00477050 — ac2-physics terrain.rs insert_object/remove_object/list_in: a server object is listed in the shadow lists of the cells its shadow box touches (static and dynamic lists merged into one per cell); CObjCell::m_objectList and the voyeur and detection bookkeeping (UpdateAllVoyeur, CLandBlock::AddDynamicObject) are not ported
- [ ] `CObjCell::UpdateAllVoyeur` · 0047d020 · legions 004770b0 — needs port: object cells: membership, detection voyeurs, frills, shadows
- [ ] `CObjCell::HideObject` · 0047d0c0 · legions 00477150 — needs port: object cells: membership, detection voyeurs, frills, shadows
- [ ] `CObjCell::UnhideObject` · 0047d0d0 · legions 00477160 — needs port: object cells: membership, detection voyeurs, frills, shadows
- [~] `CObjCell::AddObject` · 0047d170 · legions 004771a0 — ac2-physics terrain.rs insert_object/remove_object/list_in: a server object is listed in the shadow lists of the cells its shadow box touches (static and dynamic lists merged into one per cell); CObjCell::m_objectList and the voyeur and detection bookkeeping (UpdateAllVoyeur, CLandBlock::AddDynamicObject) are not ported
- [~] `CObjCell::RemoveObject` · 0047d1f0 · legions 00477220 — ac2-physics terrain.rs insert_object/remove_object/list_in: a server object is listed in the shadow lists of the cells its shadow box touches (static and dynamic lists merged into one per cell); CObjCell::m_objectList and the voyeur and detection bookkeeping (UpdateAllVoyeur, CLandBlock::AddDynamicObject) are not ported
- [ ] `CObjCell::RemoveVoyeur` · 0047d260 · legions 00477290 — needs port: object cells: membership, detection voyeurs, frills, shadows
- [ ] `CObjCell::ObjectWatchedBy` · 0047d330 · legions 00477360 — needs port: object cells: membership, detection voyeurs, frills, shadows
- [ ] `CObjCell::InqObjects` · 0047d3e0 · legions 00477410 — needs port: object cells: membership, detection voyeurs, frills, shadows
- [ ] `CObjCell::AddFrill` · 0047d500 · legions 00477530 — needs port: CObjCell::AddFrill (frill entities as cell objects)
- [ ] `CObjCell::InqVoyeurs` · 0047d5a0 · legions 004775d0 — needs port: object cells: membership, detection voyeurs, frills, shadows
- [ ] `CObjCell::AddVoyeur` · 0047d7b0 · legions 004777e0 — needs port: object cells: membership, detection voyeurs, frills, shadows

## `portal/shared/src/gametime/gametime.cpp.c` (13/29)

### GameTime
- [x] `GameTime::SetYearLength` · 00618860 · legions 005d50d0 — GameClock::new: day_length * days_per_year as a double

### WeekDay
- [-] `WeekDay::WeekDay` · 00618880 — constructor (struct init; Rust builds it directly)

### TimeOfDay
- [-] `TimeOfDay::TimeOfDay` · 00618890 — constructor (struct init; Rust builds it directly)

### GameTime
- [ ] `GameTime::GetTimeDilation` · 006188a0 · legions 005d5110 — needs port: game time, day/night and time dilation

### Season
- [-] `Season::Season` · 006188c0 — constructor (struct init; Rust builds it directly)
- [x] `Season::StreamPack` · 006188d0 · legions 005d5140 — ac2-world region/game_time.rs (GameClock::date_time: YEAR, DAY, YEARSPEC, SEASON, TIME ids), ac2-client ui_sources::game_date_time rendered for the frame, ac2-ui navigation.rs (DateText and TimeText in the macro view, CoordinatesText); tests game_time_real_tests.rs, navigation_tests, navigation_detail_7160.rs

### GameTime
- [-] `GameTime::Destroy` · 00618920 · legions 005d5190 — DBO release (Drop frees on last drop, AGENTS.md)
- [x] `GameTime::StreamPack` · 00618a40 · legions 005d52b0 — ac2-world region/game_time.rs (GameClock::date_time: YEAR, DAY, YEARSPEC, SEASON, TIME ids), ac2-client ui_sources::game_date_time rendered for the frame, ac2-ui navigation.rs (DateText and TimeText in the macro view, CoordinatesText); tests game_time_real_tests.rs, navigation_tests, navigation_detail_7160.rs
- [x] `GameTime::Init` · 00618b50 · legions 005d53c0 — ac2-world region/game_time.rs (GameClock::date_time: YEAR, DAY, YEARSPEC, SEASON, TIME ids), ac2-client ui_sources::game_date_time rendered for the frame, ac2-ui navigation.rs (DateText and TimeText in the macro view, CoordinatesText); tests game_time_real_tests.rs, navigation_tests, navigation_detail_7160.rs
- [x] `GameTime::CalcDayBegin` · 00618bd0 · legions 005d5420 — ac2-world region/game_time.rs (GameClock::date_time: YEAR, DAY, YEARSPEC, SEASON, TIME ids), ac2-client ui_sources::game_date_time rendered for the frame, ac2-ui navigation.rs (DateText and TimeText in the macro view, CoordinatesText); tests game_time_real_tests.rs, navigation_tests, navigation_detail_7160.rs
- [x] `GameTime::CalcTimeOfDay` · 00618c90 · legions 005d54e0 — ac2-world region/game_time.rs (GameClock::date_time: YEAR, DAY, YEARSPEC, SEASON, TIME ids), ac2-client ui_sources::game_date_time rendered for the frame, ac2-ui navigation.rs (DateText and TimeText in the macro view, CoordinatesText); tests game_time_real_tests.rs, navigation_tests, navigation_detail_7160.rs
- [x] `GameTime::GetDateTimeString` · 00618d40 · legions 005d5590 — ac2-world region/game_time.rs (GameClock::date_time: YEAR, DAY, YEARSPEC, SEASON, TIME ids), ac2-client ui_sources::game_date_time rendered for the frame, ac2-ui navigation.rs (DateText and TimeText in the macro view, CoordinatesText); tests game_time_real_tests.rs, navigation_tests, navigation_detail_7160.rs
- [ ] `GameTime::GetGameTimeFromTimeStamp` · 00618e40 · legions 005d5690 — needs port: game time, day/night and time dilation
- [ ] `GameTime::InqGameDateTimeString` · 00618f30 · legions 005d5780 — needs port: game time, day/night and time dilation
- [x] `GameTime::GameTime` · 00618fb0 · legions 005d5800 — crates/ac2-world/src/region/game_time.rs GameTime::GameTime clock setup
- [-] `GameTime::~GameTime` · 00619090 · legions 005d58e0 — destructor (Rust Drop)
- [x] `GameTime::UseTime` · 00619130 · legions 005d5980 — ac2-world region/game_time.rs (GameClock::date_time: YEAR, DAY, YEARSPEC, SEASON, TIME ids), ac2-client ui_sources::game_date_time rendered for the frame, ac2-ui navigation.rs (DateText and TimeText in the macro view, CoordinatesText); tests game_time_real_tests.rs, navigation_tests, navigation_detail_7160.rs
- [x] `GameTime::InqCurrentGameDateTimeString` · 00619200 · legions 005d5a50 — ac2-world GameClock::alter_time_of_day_immediate / temporal_dilation (x87 double arithmetic, objdump-verified) and need_lighting_refresh
- [x] `GameTime::TemporalDilationInternal` · 00619250 · legions 005d5aa0 — ac2-world GameClock::alter_time_of_day_immediate / temporal_dilation (x87 double arithmetic, objdump-verified) and need_lighting_refresh
- [x] `GameTime::AlterTimeOfDayImmediate` · 00619320 · legions 005d5b70 — ac2-world GameClock::alter_time_of_day_immediate / temporal_dilation (x87 double arithmetic, objdump-verified) and need_lighting_refresh

### Season
- [-] `Season::FromFileNode` · 006193a0 — text source format (admin tools)

### WeekDay
- [-] `WeekDay::FromFileNode` · 00619540 — text source format (admin tools)

### TimeOfDay
- [-] `TimeOfDay::FromFileNode` · 006196a0 — text source format (admin tools)

### GameTime
- [x] `GameTime::TemporalDilation` · 00619880 · legions 005d5bf0 — ac2-world GameClock::alter_time_of_day_immediate / temporal_dilation (x87 double arithmetic, objdump-verified) and need_lighting_refresh

### SmartArray<TimeOfDay>
- [-] `SmartArray<TimeOfDay>::AddToEnd` · 006198a0 — SmartArray container operation (Vec replaces it)

### GameTime
- [-] `GameTime::LoadSeason` · 00619900 — text source loader, admin build only
- [-] `GameTime::LoadWeek` · 00619980 — text source loader, admin build only
- [-] `GameTime::LoadHour` · 006199f0 — text source loader, admin build only
- [-] `GameTime::FromFileNode` · 00619a40 — text source format (admin tools)

## `portal/shared/src/places/placestable.cpp.c` (4/11)

### PlacesTable
- [x] `PlacesTable::StreamPack` · 00638ee0 · legions 005eec60 — ac2-script glue::places / enums / chargen (PlacesTable data::places)

### _STL::less<PStringBase<unsigned_short>_>
- [-] `_STL::less<PStringBase<unsigned_short>_>::operator()` · 00638f40 · legions 005eecc0 — STL internals

### PlacesTable
- [-] `PlacesTable::Dump` · 006395a0 · legions 005eecf0 — admin-only dump, save or error report
- [-] `PlacesTable::WslDump` · 00639690 · legions 005eede0 — admin-only dump, save or error report
- [x] `PlacesTable::GetPlace` · 00639850 · legions 005eee60 — ac2-script glue::places / enums / chargen (PlacesTable data::places)
- [x] `PlacesTable::InqPlace` · 006398d0 · legions 005eeee0 — ac2-script glue::places / enums / chargen (PlacesTable data::places)
- [ ] `PlacesTable::InqPlaceStr` · 00639960 · legions 005eef40 — needs port: named places table
- [-] `PlacesTable::AddPlace` · 00639bb0 — admin-only dump, save or error report
- [x] `PlacesTable::InqPlacesHash` · 00639c40 · legions 005ef000 — ac2-script glue::places / enums / chargen (PlacesTable data::places)
- [-] `PlacesTable::FromFileNode` · 00639cb0 — text source format (admin tools)
- [-] `PlacesTable::ToFileNode` · 00639f70 — text source format (admin tools)

## `portal/shared/src/region/blockmaps/blockmap.cpp.c` (1/8)

### BlockMap
- [ ] `BlockMap::UnPack` · 006cf150 — needs port: block maps (DAT decode)
- [-] `BlockMap::Pack` · 006cf200 — write path (client reads DATs only)
- [x] `BlockMap::GetValue` · 006cf2c0 · legions 00658ee0 — ac2-world region/detail_map.rs (DetailMap::row_from_value / row_from_block, MapNoteRow) and ac2_region.rs; real-data tests in region/detail_map_tests.rs
- [-] `BlockMap::BlockMap` · 006cf300 · legions 00658f20 — constructor (struct init; Rust builds it directly)
- [-] `BlockMap::GetDBOType` · 006cf350 — DBO type tag (Rust typed decoders)
- [-] `BlockMap::Destroy` · 006cf360 · legions 00658f80 — DBO release (Drop frees on last drop, AGENTS.md)
- [-] `BlockMap::Allocate` · 006cf400 — retail free-object pool (DBOCache exception, AGENTS.md)
- [-] `BlockMap::~BlockMap` · 006cf410 · legions 00659030 — destructor (Rust Drop)

## `portal/shared/src/region/encounter/encounterdesc.cpp.c` (0/16)

### CEncounterDesc
- [-] `CEncounterDesc::Destroy` · 00520430 — DBO release (Drop frees on last drop, AGENTS.md)
- [ ] `CEncounterDesc::StreamPack` · 00520440 · legions 0050b5b0 — needs port: encounter tables (monster spawn) (DAT decode)
- [ ] `CEncounterDesc::GetSubDataIDs` · 005204b0 · legions 0050b620 — needs port: encounter tables (monster spawn) (sub-object DID list for async sub-object resolution, AGENTS.md)
- [-] `CEncounterDesc::CEncounterDesc` · 00520500 · legions 0050b670 — constructor (struct init; Rust builds it directly)
- [-] `CEncounterDesc::GetDBOType` · 00520530 — DBO type tag (Rust typed decoders)
- [-] `CEncounterDesc::~CEncounterDesc` · 00520540 · legions 0050b6b0 — destructor (Rust Drop)
- [-] `CEncounterDesc::Allocate` · 00520680 — retail free-object pool (DBOCache exception, AGENTS.md)
- [-] `CEncounterDesc::ToFileNode` · 005206b0 — text source format (admin tools)
- [-] `CEncounterDesc::LoadMap` · 005207c0 — text source loader, admin build only

### SmartArray<EncounterExcept>
- [-] `SmartArray<EncounterExcept>::AddToEnd` · 00520900 — SmartArray container operation (Vec replaces it)

### EncounterElem
- [-] `EncounterElem::operator=` · 00520950 · legions 0050b820 — copy assignment of plain data (Rust clone/move)

### EncounterRow
- [-] `EncounterRow::operator=` · 00520b40 · legions 0050ba10 — copy assignment of plain data (Rust clone/move)

### PSmartArray<EncounterRow>
- [ ] `PSmartArray<EncounterRow>::StreamPack` · 00520cf0 · legions 0050bbc0 — needs port: DAT counted list (DAT decode)

### CEncounterDesc
- [-] `CEncounterDesc::CheckDupAdd` · 00520d90 — text source format (admin tools)
- [-] `CEncounterDesc::LoadRow` · 00520e10 — text source loader, admin build only
- [-] `CEncounterDesc::FromFileNode` · 00520ea0 — text source format (admin tools)

## `portal/shared/src/region/encounter/encounterelem.cpp.c` (0/13)

### EncounterElem
- [ ] `EncounterElem::StreamPack` · 006c6c70 · legions 00656470 — needs port: encounter elements (DAT decode)
- [ ] `EncounterElem::GetSubDataIDs` · 006c6d10 · legions 00656510 — needs port: encounter elements (sub-object DID list for async sub-object resolution, AGENTS.md)
- [-] `EncounterElem::EncounterElem` · 006c6d90 — constructor (struct init; Rust builds it directly)
- [-] `EncounterElem::~EncounterElem` · 006c6dd0 · legions 006565d0 — destructor (Rust Drop)
- [ ] `EncounterElem::PutSubObject` · 006c6e60 — needs port: encounter elements
- [-] `EncounterElem::LoadEncounter` · 006c6f10 — text source loader, admin build only

### PSmartArray<EncounterExcept>
- [ ] `PSmartArray<EncounterExcept>::StreamPack` · 006c7070 · legions 00656660 — needs port: DAT counted list (DAT decode)

### EncounterElem
- [ ] `EncounterElem::PutTerrainException` · 006c7110 — needs port: encounter elements
- [ ] `EncounterElem::PutWaterException` · 006c71f0 — needs port: encounter elements
- [-] `EncounterElem::ToFileNode` · 006c72d0 — text source format (admin tools)
- [-] `EncounterElem::LoadTerrainExcept` · 006c7510 — text source loader, admin build only
- [-] `EncounterElem::LoadWaterExcept` · 006c7720 — text source loader, admin build only
- [-] `EncounterElem::FromFileNode` · 006c7930 — text source format (admin tools)

## `portal/shared/src/region/encounter/encounterexcept.cpp.c` (0/1)

### EncounterExcept
- [-] `EncounterExcept::EncounterExcept` · 006c6c50 — constructor (struct init; Rust builds it directly)

## `portal/shared/src/region/encounter/encounterrow.cpp.c` (0/10)

### EncounterRow
- [ ] `EncounterRow::GetSubDataIDs` · 006c7c10 · legions 00656700 — needs port: encounter rows (sub-object DID list for async sub-object resolution, AGENTS.md)
- [-] `EncounterRow::EncounterRow` · 006c7c50 · legions 00656740 — constructor (struct init; Rust builds it directly)
- [-] `EncounterRow::Destroy` · 006c7c90 · legions 00656780 — DBO release (Drop frees on last drop, AGENTS.md)
- [-] `EncounterRow::~EncounterRow` · 006c7ce0 · legions 006567d0 — destructor (Rust Drop)
- [-] `EncounterRow::ToFileNode` · 006c7d40 — text source format (admin tools)
- [ ] `EncounterRow::StreamPack` · 006c7e40 · legions 00656830 — needs port: encounter rows (DAT decode)

### PSmartArray<EncounterElem>
- [ ] `PSmartArray<EncounterElem>::StreamPack` · 006c7ea0 · legions 00656890 — needs port: DAT counted list (DAT decode)

### EncounterRow
- [-] `EncounterRow::CheckDupAdd` · 006c7f40 — text source format (admin tools)
- [-] `EncounterRow::LoadElement` · 006c7fc0 — text source loader, admin build only
- [-] `EncounterRow::FromFileNode` · 006c8020 — text source format (admin tools)

## `portal/shared/src/region/filenamedesc.cpp.c` (0/2)

### FileNameDesc
- [-] `FileNameDesc::FileNameDesc` · 0052bd90 — constructor (struct init; Rust builds it directly)
- [-] `FileNameDesc::~FileNameDesc` · 0052be40 · legions 00510a30 — destructor (Rust Drop)

## `portal/shared/src/region/fog/fogdesc.cpp.c` (0/15)

### CFogDesc
- [-] `CFogDesc::CFogDesc` · 00524ce0 · legions 0050e1b0 — constructor (struct init; Rust builds it directly)
- [-] `CFogDesc::GetDBOType` · 00524d20 — DBO type tag (Rust typed decoders)
- [-] `CFogDesc::Destroy` · 00524d30 · legions 0050e200 — DBO release (Drop frees on last drop, AGENTS.md)
- [-] `CFogDesc::Pack` · 00524db0 — write path (client reads DATs only)
- [-] `CFogDesc::GetPackSize` · 00524f10 · legions 0050e3e0 — write path (client reads DATs only)
- [ ] `CFogDesc::GetSubDataIDs` · 00524f70 · legions 0050e440 — needs port: fog descriptors (sub-object DID list for async sub-object resolution, AGENTS.md)
- [ ] `CFogDesc::FindFogType` · 00524fe0 · legions 0050e4b0 — needs port: fog descriptors
- [ ] `CFogDesc::HasFog` · 00525030 · legions 0050e500 — needs port: fog descriptors
- [ ] `CFogDesc::GetFogging` · 00525050 · legions 0050e520 — needs port: fog descriptors
- [-] `CFogDesc::Allocate` · 00525080 — retail free-object pool (DBOCache exception, AGENTS.md)
- [ ] `CFogDesc::UnPack` · 005250d0 — needs port: fog descriptors (DAT decode)
- [-] `CFogDesc::ToFileNode` · 00525280 — text source format (admin tools)
- [-] `CFogDesc::CheckDupAdd` · 005254d0 — text source format (admin tools)
- [-] `CFogDesc::LoadFog` · 00525540 — text source loader, admin build only
- [-] `CFogDesc::FromFileNode` · 005255e0 — text source format (admin tools)

## `portal/shared/src/region/fog/fogtype.cpp.c` (0/8)

### FogType
- [-] `FogType::FogType` · 006ca330 · legions 006570a0 — constructor (struct init; Rust builds it directly)
- [ ] `FogType::unpack` · 006ca350 · legions 006570c0 — needs port: fog types (DAT decode)
- [-] `FogType::GetPackSize` · 006ca3a0 · legions 006580a0 — write path (client reads DATs only)
- [ ] `FogType::HasFog` · 006ca3b0 · legions 00657110 — needs port: fog types
- [ ] `FogType::GetFogging` · 006ca3c0 · legions 00657120 — needs port: fog types
- [-] `FogType::~FogType` · 006ca3e0 · legions 00657140 — destructor (Rust Drop)
- [-] `FogType::FromFileNode` · 006ca410 — text source format (admin tools)
- [-] `FogType::ToFileNode` · 006ca590 — text source format (admin tools)

## `portal/shared/src/region/fog/lightfoginfo.cpp.c` (0/19)

### LightInfo
- [-] `LightInfo::Destroy` · 006ca670 · legions 00657170 — DBO release (Drop frees on last drop, AGENTS.md)

### LightFogInfo
- [-] `LightFogInfo::LightFogInfo` · 006ca6f0 · legions 006571f0 — constructor (struct init; Rust builds it directly)
- [-] `LightFogInfo::Destroy` · 006ca790 · legions 00657290 — DBO release (Drop frees on last drop, AGENTS.md)
- [ ] `LightFogInfo::GetCodePart` · 006ca810 · legions 00657310 — needs port: light and fog info
- [ ] `LightFogInfo::FindCodeType` · 006ca870 · legions 00657370 — needs port: light and fog info
- [ ] `LightFogInfo::SetCombine` · 006ca8c0 · legions 006573c0 — needs port: light and fog info
- [-] `LightFogInfo::~LightFogInfo` · 006ca940 · legions 00657440 — destructor (Rust Drop)
- [ ] `LightFogInfo::unpack` · 006ca950 · legions 00657450 — needs port: light and fog info (DAT decode)
- [ ] `LightFogInfo::GetCode` · 006cabd0 · legions 006576d0 — needs port: light and fog info
- [-] `LightFogInfo::pack` · 006cac30 · legions 00657730 — write path (client reads DATs only)
- [-] `LightFogInfo::GetPackSize` · 006caea0 · legions 006579a0 — write path (client reads DATs only)
- [-] `LightFogInfo::LoadFog` · 006caf10 — text source loader, admin build only
- [-] `LightFogInfo::LoadDirectLight` · 006cb290 — text source loader, admin build only
- [-] `LightFogInfo::LoadAmbientLight` · 006cb430 — text source loader, admin build only
- [-] `LightFogInfo::LoadShadowColor` · 006cb5d0 — text source loader, admin build only
- [-] `LightFogInfo::LoadCombine` · 006cb700 — text source loader, admin build only
- [ ] `LightFogInfo::SaveCombine` · 006cb930 — needs port: light and fog info
- [-] `LightFogInfo::FromFileNode` · 006cba90 — text source format (admin tools)
- [-] `LightFogInfo::ToFileNode` · 006cbcc0 — text source format (admin tools)

## `portal/shared/src/region/properties/blockproperty.cpp.c` (0/16)

### BlockProperty
- [ ] `BlockProperty::GetPropertyType` · 006cd220 · legions 00658100 — needs port: block properties
- [ ] `BlockProperty::StreamPack` · 006cd240 · legions 00658120 — needs port: block properties (DAT decode)
- [ ] `BlockProperty::GetProperty` · 006cd2b0 · legions 00658190 — needs port: block properties
- [ ] `BlockProperty::GetInteger` · 006cd2e0 · legions 006581c0 — needs port: block properties
- [ ] `BlockProperty::GetFloat` · 006cd320 · legions 00658240 — needs port: block properties
- [ ] `BlockProperty::GetVector` · 006cd360 · legions 00658280 — needs port: block properties
- [ ] `BlockProperty::GetColor` · 006cd3a0 · legions 006582c0 — needs port: block properties
- [ ] `BlockProperty::GetEnum` · 006cd3e0 — needs port: block properties
- [-] `BlockProperty::BlockProperty` · 006cd420 — constructor (struct init; Rust builds it directly)
- [-] `BlockProperty::~BlockProperty` · 006cd450 · legions 00658330 — destructor (Rust Drop)
- [-] `BlockProperty::LoadName` · 006cd4a0 — text source loader, admin build only
- [ ] `BlockProperty::CheckGrow` · 006cd580 — needs port: block properties
- [-] `BlockProperty::LoadBlockMap` · 006cd640 — text source loader, admin build only
- [-] `BlockProperty::LoadValue` · 006cd710 — text source loader, admin build only
- [-] `BlockProperty::LoadLinear` · 006cd950 — text source loader, admin build only
- [-] `BlockProperty::Load` · 006ce070 — admin-only text/file loader (no Legions match)

## `portal/shared/src/region/properties/propertydesc.cpp.c` (0/18)

### PropertyDesc
- [ ] `PropertyDesc::IntegerCornerInterp` · 006ce4a0 · legions 00658380 — needs port: block property descriptors (interpolated per corner)
- [ ] `PropertyDesc::FloatCornerInterp` · 006ce4f0 · legions 006583d0 — needs port: block property descriptors (interpolated per corner)
- [ ] `PropertyDesc::VectorCornerInterp` · 006ce520 · legions 00658400 — needs port: block property descriptors (interpolated per corner)
- [ ] `PropertyDesc::ColorCornerInterp` · 006ce5b0 · legions 00658490 — needs port: block property descriptors (interpolated per corner)
- [ ] `PropertyDesc::GetSubDataIDs` · 006ce660 · legions 00658540 — needs port: block property descriptors (interpolated per corner) (sub-object DID list for async sub-object resolution, AGENTS.md)
- [-] `PropertyDesc::PropertyDesc` · 006ce6a0 · legions 00658580 — constructor (struct init; Rust builds it directly)
- [-] `PropertyDesc::GetDBOType` · 006ce6d0 — DBO type tag (Rust typed decoders)
- [-] `PropertyDesc::~PropertyDesc` · 006ce6e0 · legions 006585c0 — destructor (Rust Drop)
- [ ] `PropertyDesc::GetBlockPropertyInternal` · 006ce760 · legions 00658640 — needs port: block property descriptors (interpolated per corner)
- [ ] `PropertyDesc::GetPassiblity` · 006ceb80 · legions 00658a20 — needs port: block property descriptors (interpolated per corner)
- [-] `PropertyDesc::Allocate` · 006cec80 — retail free-object pool (DBOCache exception, AGENTS.md)
- [ ] `PropertyDesc::UpdateBlockMaterial` · 006cecb0 · legions 00658b30 — needs port: block property descriptors (interpolated per corner)

### BlockProperty
- [-] `BlockProperty::operator=` · 006ced20 · legions 00658ba0 — copy assignment of plain data (Rust clone/move)

### PSmartArray<BlockProperty>
- [ ] `PSmartArray<BlockProperty>::StreamPack` · 006cee50 · legions 00658cd0 — needs port: DAT counted list (DAT decode)

### SmartArray<BlockProperty>
- [-] `SmartArray<BlockProperty>::AddToEnd` · 006ceef0 — SmartArray container operation (Vec replaces it)

### PropertyDesc
- [-] `PropertyDesc::CheckDupAdd` · 006cef60 — text source format (admin tools)
- [-] `PropertyDesc::LoadProperty` · 006cefb0 — text source loader, admin build only
- [-] `PropertyDesc::FromFileNode` · 006cf040 — text source format (admin tools)

## `portal/shared/src/region/regiondesc.cpp.c` (7/47)

### PackObj
- [x] `PackObj::ALIGN_PTR` · 0051d6e0 · legions 0050a0e0 — crates/ac2-script/src/unpack.rs PackObj::ALIGN_PTR

### CRegionDesc
- [ ] `CRegionDesc::SetLandDefs` · 0051d730 · legions 0050a130 — needs port: region data: scene, terrain, water, sky, fog lookups
- [ ] `CRegionDesc::RemoveCurrentRegion` · 0051d7a0 · legions 0050a1a0 — needs port: region data: scene, terrain, water, sky, fog lookups
- [-] `CRegionDesc::LoadHeader` · 0051d800 — text source loader, admin build only
- [x] `CRegionDesc::GetGameMap` · 0051d850 · legions 0050a200 — crates/ac2-world/src/region/ac2_region.rs CRegionDesc::GetGameMap
- [-] `CRegionDesc::Destroy` · 0051d920 · legions 0050a2d0 — DBO release (Drop frees on last drop, AGENTS.md)
- [-] `CRegionDesc::PackInfoList` · 0051db20 · legions 0050a4d0 — write path (client reads DATs only)
- [ ] `CRegionDesc::GetSubDataIDs` · 0051dbd0 · legions 0050a580 — needs port: region data: scene, terrain, water, sky, fog lookups (sub-object DID list for async sub-object resolution, AGENTS.md)
- [x] `CRegionDesc::GetCurrentSoundInfo` · 0051de10 · legions 0050a7c0 — ac2-audio/ac2-client: ambient environments, groove
- [ ] `CRegionDesc::GetCurrentSceneInfo` · 0051de40 · legions 0050a7e0 — needs port: region data: scene, terrain, water, sky, fog lookups
- [ ] `CRegionDesc::GetCurrentTerrainInfo` · 0051de70 · legions 0050a810 — needs port: region data: scene, terrain, water, sky, fog lookups
- [x] `CRegionDesc::GetCurrentSurfaceInfo` · 0051deb0 · legions 0050a840 — crates/ac2-world/src/ac2_world.rs CRegionDesc::GetCurrentSurfaceInfo
- [x] `CRegionDesc::GetCurrentDetailMapTable` · 0051def0 · legions 0050a870 — ac2-world region/detail_map.rs (DetailMap::row_from_value / row_from_block, MapNoteRow) and ac2_region.rs; real-data tests in region/detail_map_tests.rs
- [x] `CRegionDesc::GetCurrentWaterInfo` · 0051df30 · legions 0050a8a0 — crates/ac2-world/src/region/ac2_region.rs CRegionDesc::GetCurrentWaterInfo
- [ ] `CRegionDesc::GetCurrentFogInfo` · 0051df70 · legions 0050a8d0 — needs port: region data: scene, terrain, water, sky, fog lookups
- [ ] `CRegionDesc::GetCurrentPropertyInfo` · 0051dfb0 · legions 0050a900 — needs port: region data: scene, terrain, water, sky, fog lookups

### PFileNode
- [ ] `PFileNode::GetLeafAsFloat` · 0051dff0 · legions 0050a930 — needs port: FileNode accessors (used by FromFileNode loaders; confirm DAT path)

### CRegionDesc
- [-] `CRegionDesc::CRegionDesc` · 0051e010 · legions 0050a950 — constructor (struct init; Rust builds it directly)
- [-] `CRegionDesc::GetDBOType` · 0051e150 — DBO type tag (Rust typed decoders)
- [ ] `CRegionDesc::~CRegionDesc` · 0051e160 · legions 0050aaa0 — needs port: CRegionDesc destructor unload (world hash and resident state)
- [-] `CRegionDesc::GetPackSize` · 0051e290 · legions 0050abd0 — write path (client reads DATs only)
- [~] `CRegionDesc::SetGameTime` · 0051e480 · legions 0050adc0 — ac2-world ac2_region: the first sky's game_time DID (sky_info[current_sky_info] 0), loaded with the region; the script's game-time glue is not wired
- [ ] `CRegionDesc::GetCurrentSkyInfo` · 0051e540 — needs port: region data: scene, terrain, water, sky, fog lookups
- [-] `CRegionDesc::Allocate` · 0051e580 — retail free-object pool (DBOCache exception, AGENTS.md)
- [ ] `CRegionDesc::UnPackInfoList` · 0051e5b0 · legions 0050aee0 — needs port: region data: scene, terrain, water, sky, fog lookups (DAT decode)
- [ ] `CRegionDesc::SetRegionByGID` · 0051e6c0 — needs port: region data: scene, terrain, water, sky, fog lookups
- [ ] `CRegionDesc::SetRegion` · 0051e7a0 — needs port: region data: scene, terrain, water, sky, fog lookups
- [-] `CRegionDesc::LoadCellMaps` · 0051e800 — text source loader, admin build only
- [-] `CRegionDesc::LoadGameMap` · 0051ec30 — text source loader, admin build only
- [-] `CRegionDesc::LoadFile` · 0051eeb0 — text source loader, admin build only
- [-] `CRegionDesc::CheckDupAdd` · 0051ef90 — text source format (admin tools)
- [-] `CRegionDesc::Pack` · 0051f030 — write path (client reads DATs only)
- [x] `CRegionDesc::UnPack` · 0051f260 — crates/ac2-world/src/region/ac2_region.rs CRegionDesc::UnPack
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
- [ ] `RegionEmap::GetSceneEnum` · 006bc6f0 — needs port: world content (RegionEmap)
- [ ] `RegionEmap::GetSurfaceEnum` · 006bc770 — needs port: world content (RegionEmap)
- [ ] `RegionEmap::GetSkyObjectEnum` · 006bc7f0 — needs port: world content (RegionEmap)
- [ ] `RegionEmap::GetSkyObjectName` · 006bc870 — needs port: world content (RegionEmap)

## `portal/shared/src/region/scene/lscapescene.cpp.c` (0/8)

### LScapeScene
- [-] `LScapeScene::LScapeScene` · 006c32e0 · legions 00654960 — constructor (struct init; Rust builds it directly)
- [-] `LScapeScene::pack` · 006c3300 · legions 00654980 — write path (client reads DATs only)
- [ ] `LScapeScene::unpack` · 006c3360 · legions 006549e0 — needs port: landscape scene, surfaces, materials and vertex colours (DAT decode)

### FileNodeName_Int64
- [-] `FileNodeName_Int64::GetType` · 006c33b0 · legions 00654a30 — FileNode name wrapper (Rust strings)

### LScapeScene
- [ ] `LScapeScene::GetSubDataIDs` · 006c33c0 · legions 00654a40 — needs port: landscape scene, surfaces, materials and vertex colours (sub-object DID list for async sub-object resolution, AGENTS.md)
- [ ] `LScapeScene::~LScapeScene` · 006c33e0 · legions 00654a60 — needs port: LScapeScene destructor unload (world hash and resident state)
- [-] `LScapeScene::LoadSceneFile` · 006c3400 — text source loader, admin build only
- [-] `LScapeScene::Load` · 006c34d0 — admin-only text/file loader (no Legions match)

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
- [ ] `SceneDegrade::GetDegradeDist` · 006c2ba0 · legions 00654630 — needs port: scene LOD degrade
- [-] `SceneDegrade::SceneDegrade` · 006c2be0 · legions 00654670 — constructor (struct init; Rust builds it directly)
- [-] `SceneDegrade::Destroy` · 006c2c00 · legions 00654690 — DBO release (Drop frees on last drop, AGENTS.md)
- [ ] `SceneDegrade::GetSubDataIDs` · 006c2c70 · legions 00654700 — needs port: scene LOD degrade (sub-object DID list for async sub-object resolution, AGENTS.md)
- [-] `SceneDegrade::~SceneDegrade` · 006c2cb0 · legions 00654740 — destructor (Rust Drop)
- [ ] `SceneDegrade::CopyInto` · 006c2cd0 · legions 00654760 — needs port: scene LOD degrade
- [ ] `SceneDegrade::StreamPack` · 006c2d80 · legions 00654810 — needs port: scene LOD degrade (DAT decode)
- [-] `SceneDegrade::ToFileNode` · 006c2ed0 — text source format (admin tools)
- [-] `SceneDegrade::LoadEntity` · 006c2ff0 — text source loader, admin build only
- [-] `SceneDegrade::FromFileNode` · 006c3070 — text source format (admin tools)

## `portal/shared/src/region/scene/scenedesc.cpp.c` (1/14)

### CSceneDesc
- [-] `CSceneDesc::CSceneDesc` · 00523b20 · legions 0050d8a0 — constructor (struct init; Rust builds it directly)
- [-] `CSceneDesc::GetDBOType` · 00523b50 — DBO type tag (Rust typed decoders)
- [-] `CSceneDesc::Destroy` · 00523b60 · legions 0050d8e0 — DBO release (Drop frees on last drop, AGENTS.md)
- [-] `CSceneDesc::GetPackSize` · 00523bb0 · legions 0050d930 — write path (client reads DATs only)
- [-] `CSceneDesc::Pack` · 00523bf0 — write path (client reads DATs only)
- [ ] `CSceneDesc::GetSubDataIDs` · 00523cb0 · legions 0050da30 — needs port: scene descriptors (scenery id and sound type) (sub-object DID list for async sub-object resolution, AGENTS.md)
- [ ] `CSceneDesc::FindSceneType` · 00523cf0 · legions 0050da70 — needs port: scene descriptors (scenery id and sound type)
- [ ] `CSceneDesc::GetSceneID` · 00523d30 · legions 0050dab0 — needs port: scene descriptors (scenery id and sound type)
- [x] `CSceneDesc::GetSoundType` · 00523d70 · legions 0050dad0 — ac2-audio/ac2-client: ambient environments, groove
- [-] `CSceneDesc::Allocate` · 00523dc0 — retail free-object pool (DBOCache exception, AGENTS.md)
- [ ] `CSceneDesc::UnPack` · 00523e10 — needs port: scene descriptors (scenery id and sound type) (DAT decode)
- [-] `CSceneDesc::CheckDupAdd` · 00523ef0 — text source format (admin tools)
- [-] `CSceneDesc::LoadSceneType` · 00523f60 — text source loader, admin build only
- [-] `CSceneDesc::FromFileNode` · 00523fe0 — text source format (admin tools)

## `portal/shared/src/region/scene/sceneentity.cpp.c` (0/10)

### SceneEntity
- [ ] `SceneEntity::CopyInto` · 006c17d0 · legions 00653ce0 — needs port: scene entities
- [ ] `SceneEntity::GetSubDataIDs` · 006c1850 · legions 00653d50 — needs port: scene entities (sub-object DID list for async sub-object resolution, AGENTS.md)
- [ ] `SceneEntity::GetDataID` · 006c1890 · legions 00653d80 — needs port: scene entities
- [ ] `SceneEntity::GetSubGroupPtr` · 006c18b0 · legions 00653da0 — needs port: scene entities
- [ ] `SceneEntity::StreamPack` · 006c18c0 · legions 00653db0 — needs port: scene entities (DAT decode)
- [-] `SceneEntity::~SceneEntity` · 006c1970 · legions 00653e50 — destructor (Rust Drop)
- [ ] `SceneEntity::PutSubObject` · 006c19c0 — needs port: scene entities
- [ ] `SceneEntity::GetSubObject` · 006c1a90 — needs port: scene entities
- [-] `SceneEntity::ToFileNode` · 006c1be0 — text source format (admin tools)
- [-] `SceneEntity::FromFileNode` · 006c1c90 — text source format (admin tools)

## `portal/shared/src/region/scene/scenefile.cpp.c` (1/12)

### SceneFile
- [-] `SceneFile::SceneFile` · 006c1d10 · legions 00653e90 — constructor (struct init; Rust builds it directly)

### AMLook
- [ ] `AMLook::GetType` · 006c1d40 — needs port: world content (AMLook)

### SceneFile
- [-] `SceneFile::Destroy` · 006c1d50 · legions 00653ed0 — DBO release (Drop frees on last drop, AGENTS.md)
- [ ] `SceneFile::GetSubDataIDs` · 006c1dc0 · legions 00653f40 — needs port: scene files (frill and scenery groups) (sub-object DID list for async sub-object resolution, AGENTS.md)
- [ ] `SceneFile::GetCorrectDegrade` · 006c1e10 · legions 00653f90 — needs port: scene files (frill and scenery groups)
- [-] `SceneFile::Allocate` · 006c1e60 — retail free-object pool (DBOCache exception, AGENTS.md)
- [ ] `SceneFile::CopyInto` · 006c1eb0 — needs port: scene files (frill and scenery groups)
- [x] `SceneFile::StreamPack` · 006c1fa0 — crates/ac2-world/src/ac2_scene/format.rs SceneFile::StreamPack
- [-] `SceneFile::ToFileNode` · 006c2150 — text source format (admin tools)
- [-] `SceneFile::LoadSceneDegrade` · 006c2250 — text source loader, admin build only
- [-] `SceneFile::LoadClientFrills` · 006c22d0 — text source loader, admin build only
- [-] `SceneFile::FromFileNode` · 006c2350 — text source format (admin tools)

## `portal/shared/src/region/scene/scenegroup.cpp.c` (14/45)

### SceneGroup
- [-] `SceneGroup::SetMinRot` · 006bd400 — scene editor setter (client reads the DAT values)
- [-] `SceneGroup::SetMaxRot` · 006bd430 — scene editor setter (client reads the DAT values)
- [-] `SceneGroup::SetFrequency` · 006bd460 — scene editor setter (client reads the DAT values)
- [-] `SceneGroup::SetMaxAboveDryLand` · 006bd490 — scene editor setter (client reads the DAT values)
- [-] `SceneGroup::SetMinAboveDryLand` · 006bd4c0 — scene editor setter (client reads the DAT values)
- [-] `SceneGroup::SetMaxAboveWater` · 006bd4f0 — scene editor setter (client reads the DAT values)
- [-] `SceneGroup::SetMinAboveWater` · 006bd520 — scene editor setter (client reads the DAT values)
- [-] `SceneGroup::SetMaxBelowWater` · 006bd550 — scene editor setter (client reads the DAT values)
- [-] `SceneGroup::SetMinBelowWater` · 006bd580 — scene editor setter (client reads the DAT values)
- [-] `SceneGroup::SetMinWaterDepth` · 006bd5b0 — scene editor setter (client reads the DAT values)
- [-] `SceneGroup::SetMaxWaterDepth` · 006bd5e0 — scene editor setter (client reads the DAT values)
- [-] `SceneGroup::SetMaxBelowGround` · 006bd610 — scene editor setter (client reads the DAT values)
- [-] `SceneGroup::SetMinBelowGround` · 006bd640 — scene editor setter (client reads the DAT values)
- [-] `SceneGroup::SetDayFreq` · 006bd670 — scene editor setter (client reads the DAT values)
- [-] `SceneGroup::SetMaxSlope` · 006bd6a0 — scene editor setter (client reads the DAT values)
- [-] `SceneGroup::SetMinSlope` · 006bd6e0 — scene editor setter (client reads the DAT values)
- [-] `SceneGroup::SetClearanceRadius` · 006bd720 — scene editor setter (client reads the DAT values)
- [-] `SceneGroup::SetMaxSlopeRadius` · 006bd750 — scene editor setter (client reads the DAT values)
- [-] `SceneGroup::SetMinSlopeRadius` · 006bd7a0 — scene editor setter (client reads the DAT values)
- [x] `SceneGroup::CheckFreq` · 006bd7f0 · legions 00652190
- [x] `SceneGroup::GetScale` · 006bd870 · legions 00652210
- [x] `SceneGroup::HaveMaxObjects` · 006bdc00 · legions 006525a0
- [x] `SceneGroup::UpdateZLoc` · 006bdc20 · legions 006525c0
- [x] `SceneGroup::GetCheckRadius` · 006bdcb0 · legions 00652640
- [x] `SceneGroup::ScaleByLandSlope` · 006bdcf0 · legions 00652680
- [x] `SceneGroup::GetClearanceRadius` · 006bdda0 · legions 00652730
- [x] `SceneGroup::CheckSlope` · 006bde20 · legions 006527b0
- [x] `SceneGroup::GetAlignedLimits` · 006bde50 · legions 006527e0
- [-] `SceneGroup::SceneGroup` · 006bded0 · legions 00652860 — constructor (struct init; Rust builds it directly)
- [-] `SceneGroup::Destroy` · 006be020 · legions 006529b0 — DBO release (Drop frees on last drop, AGENTS.md)
- [ ] `SceneGroup::GetSubDataIDs` · 006be1b0 · legions 00652b40 — needs port: procedural scenery groups (placement rules) (sub-object DID list for async sub-object resolution, AGENTS.md)
- [x] `SceneGroup::UpdateXYLoc` · 006be200 · legions 00652b90
- [~] `SceneGroup::UpdateFrame` · 006be3c0 · legions 00652d50 — heading, orient and rot-sub paths; align (Plane::OrientFrameToPlane) not re-verified
- [x] `SceneGroup::ScaleByWaterDepth` · 006be680 · legions 00653000
- [x] `SceneGroup::CheckFinalHeight` · 006be780 · legions 00653100
- [-] `SceneGroup::~SceneGroup` · 006be9f0 · legions 00653380 — destructor (Rust Drop)
- [x] `SceneGroup::FinishPlaceAndCheck` · 006bea10 · legions 006533a0
- [ ] `SceneGroup::CopyInto` · 006beb80 · legions 00653510 — needs port: procedural scenery groups (placement rules)
- [x] `SceneGroup::StreamPack` · 006bee00 · legions 00653790 — crates/ac2-world/src/ac2_scene/format.rs SceneGroup::StreamPack
- [ ] `SceneGroup::GetCorner` · 006bf350 — needs port: procedural scenery groups (placement rules)
- [-] `SceneGroup::PutRadiusInfo` · 006bf3e0 — scene editor writer (admin-only)
- [ ] `SceneGroup::GetRadiusInfo` · 006bf560 — needs port: procedural scenery groups (placement rules)
- [-] `SceneGroup::LoadEntity` · 006bf840 — text source loader, admin build only
- [-] `SceneGroup::ToFileNode` · 006bf8c0 — text source format (admin tools)
- [-] `SceneGroup::FromFileNode` · 006c03f0 — text source format (admin tools)

## `portal/shared/src/region/scene/scenetype.cpp.c` (0/12)

### SceneType
- [-] `SceneType::Destroy` · 006c24b0 · legions 006542d0 — DBO release (Drop frees on last drop, AGENTS.md)
- [-] `SceneType::pack` · 006c2500 · legions 00654320 — write path (client reads DATs only)
- [-] `SceneType::pack_size` · 006c2590 · legions 006543b0 — write path (client reads DATs only)
- [ ] `SceneType::GetSubDataIDs` · 006c25d0 · legions 006543f0 — needs port: scene types (sub-object DID list for async sub-object resolution, AGENTS.md)
- [ ] `SceneType::GetScene` · 006c2610 · legions 00654430 — needs port: scene types
- [-] `SceneType::SceneType` · 006c2690 · legions 006544b0 — constructor (struct init; Rust builds it directly)
- [-] `SceneType::~SceneType` · 006c26c0 · legions 006544e0 — destructor (Rust Drop)
- [ ] `SceneType::unpack` · 006c2700 · legions 00654520 — needs port: scene types (DAT decode)
- [-] `SceneType::LoadName` · 006c2810 — text source loader, admin build only
- [-] `SceneType::LoadSound` · 006c2880 — text source loader, admin build only
- [-] `SceneType::LoadLScapeScene` · 006c2970 — text source loader, admin build only
- [-] `SceneType::Load` · 006c29f0 — admin-only text/file loader (no Legions match)

## `portal/shared/src/region/sky/daydesc.cpp.c` (12/38)

### CelestialPosition
- [x] `CelestialPosition::CelestialPosition` · 005257a0 · legions 0050e750 — ac2-world region::sky_objects, ac2-render sky

### CDayDesc
- [x] `CDayDesc::GetTimeRatio` · 00525800 · legions 0050e7b0
- [x] `CDayDesc::InterpAngleClosest` · 005258b0 · legions 0050e860
- [x] `CDayDesc::InterpColor` · 00525950 · legions 0050e900
- [x] `CDayDesc::CalcLightVector` · 005259e0 · legions 0050e990
- [-] `CDayDesc::Destroy` · 00525a90 · legions 0050ea40 — DBO release (Drop frees on last drop, AGENTS.md)
- [x] `CDayDesc::StreamPack` · 00525af0 · legions 0050eaa0 — crates/ac2-world/src/region/ac2_region.rs CDayDesc::StreamPack
- [ ] `CDayDesc::GetSubDataIDs` · 00525b60 — needs port: day descriptors (sky per time of day) (sub-object DID list for async sub-object resolution, AGENTS.md)
- [-] `CDayDesc::CheckDupAdd` · 00525bd0 — text source format (admin tools)
- [-] `CDayDesc::SetInterpDir` · 00525c10 — admin day-table editor (no client path)
- [x] `CDayDesc::GetBeforeTime` · 00525cb0 · legions 0050eb80
- [x] `CDayDesc::GetAfterTime` · 00525d30 · legions 0050ec00
- [x] `CDayDesc::GetLighting` · 00525d90 · legions 0050ec60
- [x] `CDayDesc::GetWorldFog` · 00525f70 · legions 0050ee40
- [x] `CDayDesc::GetRain` · 00526060 · legions 0050ef30 — rain amount and wind; streaks drawn as the rain material's particle batch above 0.4
- [-] `CDayDesc::CDayDesc` · 005262f0 · legions 0050f1c0 — constructor (struct init; Rust builds it directly)
- [-] `CDayDesc::GetDBOType` · 00526330 — DBO type tag (Rust typed decoders)
- [-] `CDayDesc::~CDayDesc` · 00526340 · legions 0050f210 — destructor (Rust Drop)

### PSmartArray<SkyTimeOfDay>
- [ ] `PSmartArray<SkyTimeOfDay>::StreamPack` · 005263b0 · legions 0050f280 — needs port: DAT counted list (DAT decode)

### SkyObjectReplace
- [-] `SkyObjectReplace::operator=` · 00526460 · legions 0050f330 — copy assignment of plain data (Rust clone/move)

### SmartArray<LensFlarePoint>
- [-] `SmartArray<LensFlarePoint>::operator=` · 00526590 · legions 0050f460 — copy assignment of plain data (Rust clone/move)

### CDayDesc
- [-] `CDayDesc::Allocate` · 00526600 — retail free-object pool (DBOCache exception, AGENTS.md)
- [-] `CDayDesc::ToFileNode` · 00526630 — text source format (admin tools)

### SmartArray<SkyTimeOfDay>
- [-] `SmartArray<SkyTimeOfDay>::AddToEnd` · 00526820 — SmartArray container operation (Vec replaces it)

### CDayDesc
- [-] `CDayDesc::LoadTimeOfDay` · 00526870 — text source loader, admin build only

### SmartArray<SkyObjectReplace>
- [-] `SmartArray<SkyObjectReplace>::AddToEnd` · 005268f0 — SmartArray container operation (Vec replaces it)

### SkyObject
- [-] `SkyObject::operator=` · 00526980 · legions 0050f500 — copy assignment of plain data (Rust clone/move)

### CelestialPosition
- [-] `CelestialPosition::operator=` · 00526a20 · legions 0050f5a0 — copy assignment of plain data (Rust clone/move)

### CDayDesc
- [-] `CDayDesc::LoadAllTimes` · 00526ba0 — text source loader, admin build only
- [-] `CDayDesc::SetInterpDiffuse` · 00526c80 — admin day-table editor (no client path)
- [-] `CDayDesc::SetInterpOpacity` · 00526d40 — admin day-table editor (no client path)

### SmartArray<SkyObject>
- [-] `SmartArray<SkyObject>::RemoveOrderedByIndex` · 00526dd0 — SmartArray container operation (Vec replaces it)

### PSmartArray<SkyObject>
- [ ] `PSmartArray<SkyObject>::StreamPack` · 00526e20 · legions 0050f720 — needs port: DAT counted list (DAT decode)

### CDayDesc
- [-] `CDayDesc::AddMissingEntries` · 00526fb0 — admin day-table editor (no client path)
- [-] `CDayDesc::LoadObject` · 00528780 — text source loader, admin build only
- [-] `CDayDesc::LoadSkyObjects` · 00528820 — text source loader, admin build only
- [x] `CDayDesc::GetSky` · 00528900 · legions 0050f8b0 — ac2-world region::sky_objects, ac2-render sky
- [-] `CDayDesc::FromFileNode` · 00528d10 — text source format (admin tools)

## `portal/shared/src/region/sky/skyday.cpp.c` (0/5)

### DecalFX
- [~] `DecalFX::GetSubDataIDs` · 006c8960 · legions 00643980 — DecalFX sub-object DIDs: the material is not resolved through the async sub-object path yet

### SkyDay
- [-] `SkyDay::SkyDay` · 006c8980 — constructor (struct init; Rust builds it directly)
- [-] `SkyDay::~SkyDay` · 006c89a0 · legions 00656a20 — destructor (Rust Drop)
- [-] `SkyDay::LoadName` · 006c89c0 — text source loader, admin build only
- [-] `SkyDay::FromFileNode` · 006c8aa0 — text source format (admin tools)

## `portal/shared/src/region/sky/skydesc.cpp.c` (2/17)

### CSkyDesc
- [ ] `CSkyDesc::GetTickSize` · 00524140 · legions 0050dc30 — needs port: sky descriptors and tick sizes
- [ ] `CSkyDesc::GetLightTickSize` · 00524180 · legions 0050dc70 — needs port: sky descriptors and tick sizes
- [-] `CSkyDesc::Destroy` · 005241c0 · legions 0050dcb0 — DBO release (Drop frees on last drop, AGENTS.md)
- [ ] `CSkyDesc::GetSubDataIDs` · 00524220 · legions 0050dd10 — needs port: sky descriptors and tick sizes (sub-object DID list for async sub-object resolution, AGENTS.md)
- [ ] `CSkyDesc::GetDayByIndex` · 00524270 · legions 0050dd60 — needs port: sky descriptors and tick sizes
- [x] `CSkyDesc::GetCurrentDay` · 00524290 · legions 0050dd80
- [ ] `CSkyDesc::RecalcFreq` · 00524350 · legions 0050de40 — needs port: sky descriptors and tick sizes
- [-] `CSkyDesc::CSkyDesc` · 005243d0 · legions 0050de70 — constructor (struct init; Rust builds it directly)
- [-] `CSkyDesc::GetDBOType` · 00524430 — DBO type tag (Rust typed decoders)
- [x] `CSkyDesc::StreamPack` · 00524440 · legions 0050dee0 — crates/ac2-world/src/region/ac2_region.rs CSkyDesc::StreamPack
- [-] `CSkyDesc::Allocate` · 00524600 — retail free-object pool (DBOCache exception, AGENTS.md)
- [-] `CSkyDesc::LoadGameTime` · 00524670 — text source loader, admin build only

### PSmartArray<SkyDay>
- [ ] `PSmartArray<SkyDay>::StreamPack` · 00524740 · legions 0050e110 — needs port: DAT counted list (DAT decode)

### CSkyDesc
- [-] `CSkyDesc::CheckDupAdd` · 005247e0 — text source format (admin tools)
- [-] `CSkyDesc::LoadDay` · 00524860 — text source loader, admin build only
- [-] `CSkyDesc::LoadWeatherInfo` · 005248f0 — text source loader, admin build only
- [-] `CSkyDesc::FromFileNode` · 005249d0 — text source format (admin tools)

## `portal/shared/src/region/sky/skyobject.cpp.c` (1/8)

### SkyObject
- [-] `SkyObject::Destroy` · 0052b0b0 · legions 00510790 — DBO release (Drop frees on last drop, AGENTS.md)
- [ ] `SkyObject::GetSubDataIDs` · 0052b0f0 · legions 005107d0 — needs port: sky objects (sub-object DID list for async sub-object resolution, AGENTS.md)
- [-] `SkyObject::SkyObject` · 0052b120 — constructor (struct init; Rust builds it directly)
- [-] `SkyObject::~SkyObject` · 0052b170 · legions 00510850 — destructor (Rust Drop)
- [x] `SkyObject::StreamPack` · 0052b1d0 · legions 005108b0 — ac2-world region::sky_objects, ac2-render sky
- [-] `SkyObject::ToFileNode` · 0052b350 — text source format (admin tools)
- [-] `SkyObject::LoadName` · 0052b7b0 — text source loader, admin build only
- [-] `SkyObject::FromFileNode` · 0052b880 — text source format (admin tools)

## `portal/shared/src/region/sky/skyobjectreplace.cpp.c` (1/9)

### CMapNoteRow
- [ ] `CMapNoteRow::GetSubDataIDs` · 006c81f0 · legions 00509bd0 — needs port: map note rows (sub-object DID list for async sub-object resolution, AGENTS.md)

### SkyObjectReplace
- [ ] `SkyObjectReplace::GetDiffuseColor` · 006c8210 — needs port: sky objects

### MotionValues
- [ ] `MotionValues::GetMaxVertVel` · 006c8250 — needs port: world content (MotionValues)

### SkyObjectReplace
- [-] `SkyObjectReplace::SkyObjectReplace` · 006c8260 — constructor (struct init; Rust builds it directly)
- [x] `SkyObjectReplace::StreamPack` · 006c82b0 · legions 00656980 — ac2-world region::sky_objects, ac2-render sky
- [-] `SkyObjectReplace::ToFileNode` · 006c8330 — text source format (admin tools)
- [-] `SkyObjectReplace::LoadName` · 006c8540 — text source loader, admin build only

### SkyObject
- [-] `SkyObject::LoadGfx` · 006c8610 — text source loader, admin build only

### SkyObjectReplace
- [-] `SkyObjectReplace::FromFileNode` · 006c86e0 — text source format (admin tools)

## `portal/shared/src/region/sky/skytimeofday.cpp.c` (0/17)

### SkyTimeOfDay
- [-] `SkyTimeOfDay::Destroy` · 006c8c90 · legions 00656a40 — DBO release (Drop frees on last drop, AGENTS.md)
- [~] `SkyTimeOfDay::StreamPack` · 006c8d90 · legions 00656b40 — decoded; field order after m_nDirectHeading not re-verified
- [ ] `SkyTimeOfDay::GetSubDataIDs` · 006c8f10 · legions 00656cc0 — needs port: sky time-of-day keys (sub-object DID list for async sub-object resolution, AGENTS.md)
- [ ] `SkyTimeOfDay::FindReplacement` · 006c8f50 · legions 00656d00 — needs port: sky time-of-day keys
- [-] `SkyTimeOfDay::SkyTimeOfDay` · 006c8f90 — constructor (struct init; Rust builds it directly)
- [-] `SkyTimeOfDay::~SkyTimeOfDay` · 006c9070 · legions 00656e20 — destructor (Rust Drop)

### SmartArray<SkyObjectReplace>
- [-] `SmartArray<SkyObjectReplace>::operator=` · 006c90c0 · legions 00656e70 — copy assignment of plain data (Rust clone/move)

### PSmartArray<SkyObjectReplace>
- [ ] `PSmartArray<SkyObjectReplace>::StreamPack` · 006c9160 · legions 00656f10 — needs port: DAT counted list (DAT decode)

### SkyTimeOfDay
- [-] `SkyTimeOfDay::ToFileNode` · 006c9200 — text source format (admin tools)
- [-] `SkyTimeOfDay::LoadRain` · 006c9660 — text source loader, admin build only
- [-] `SkyTimeOfDay::LoadWorldFog` · 006c9820 — text source loader, admin build only
- [-] `SkyTimeOfDay::LoadDirectLight` · 006c9b10 — text source loader, admin build only
- [-] `SkyTimeOfDay::LoadAmbientLight` · 006c9d80 — text source loader, admin build only
- [-] `SkyTimeOfDay::operator=` · 006c9f30 · legions 00656fb0 — copy assignment of plain data (Rust clone/move)
- [-] `SkyTimeOfDay::CheckDupAdd` · 006ca020 — text source format (admin tools)
- [-] `SkyTimeOfDay::LoadReplace` · 006ca070 — text source loader, admin build only
- [-] `SkyTimeOfDay::FromFileNode` · 006ca0e0 — text source format (admin tools)

## `portal/shared/src/region/surface/lscapealphadesc.cpp.c` (0/5)

### LScapeAlphaDesc
- [-] `LScapeAlphaDesc::LScapeAlphaDesc` · 006c4160 — constructor (struct init; Rust builds it directly)

### LScapeMaterialDesc
- [-] `LScapeMaterialDesc::~LScapeMaterialDesc` · 006c4180 · legions 00654f80 — destructor (Rust Drop)

### TimeOfDay
- [ ] `TimeOfDay::StreamPack` · 006c41a0 · legions 00632ab0 — needs port: time of day (DAT decode)

### LScapeAlphaDesc
- [-] `LScapeAlphaDesc::LoadTexture` · 006c4200 — text source loader, admin build only
- [-] `LScapeAlphaDesc::Load` · 006c42e0 — admin-only text/file loader (no Legions match)

## `portal/shared/src/region/surface/lscapematerialdesc.cpp.c` (1/4)

### LScapeMaterialDesc
- [-] `LScapeMaterialDesc::LScapeMaterialDesc` · 006c3e30 — constructor (struct init; Rust builds it directly)
- [x] `LScapeMaterialDesc::StreamPack` · 006c3e50 · legions 00654f00
- [-] `LScapeMaterialDesc::LoadMaterial` · 006c3eb0 — text source loader, admin build only
- [-] `LScapeMaterialDesc::Load` · 006c3f80 — admin-only text/file loader (no Legions match)

## `portal/shared/src/region/surface/lscapesurfacetype.cpp.c` (3/11)

### LScapeSurfaceType
- [x] `LScapeSurfaceType::StreamPack` · 006c5910 · legions 00655c10
- [ ] `LScapeSurfaceType::GetSubDataIDs` · 006c5990 · legions 00655c90 — needs port: landscape scene, surfaces, materials and vertex colours (sub-object DID list for async sub-object resolution, AGENTS.md)
- [x] `LScapeSurfaceType::GetVertexInfo` · 006c59d0 · legions 00655cd0
- [x] `LScapeSurfaceType::GetPhysicsMaterial` · 006c5a50 · legions 00655d50 — Physics materials: ac2-world reads each vertex's surface or road band material (LScapeSurfaceType band by pitch) and each water type's material; ac2-physics Terrain returns the land vertex, block water and interior cell materials, read by PhysicsBody::contact_materials.
- [-] `LScapeSurfaceType::LScapeSurfaceType` · 006c5ab0 — constructor (struct init; Rust builds it directly)
- [-] `LScapeSurfaceType::~LScapeSurfaceType` · 006c5ae0 · legions 00655de0 — destructor (Rust Drop)
- [-] `LScapeSurfaceType::LoadName` · 006c5b30 — text source loader, admin build only

### PSmartArray<MaterialGroup>
- [ ] `PSmartArray<MaterialGroup>::StreamPack` · 006c5ca0 · legions 00655e30 — needs port: DAT counted list (DAT decode)

### SmartArray<MaterialGroup>
- [-] `SmartArray<MaterialGroup>::AddToEnd` · 006c5d50 — SmartArray container operation (Vec replaces it)

### LScapeSurfaceType
- [-] `LScapeSurfaceType::LoadMatGroup` · 006c5da0 — text source loader, admin build only
- [-] `LScapeSurfaceType::Load` · 006c5e10 — admin-only text/file loader (no Legions match)

## `portal/shared/src/region/surface/lscapevertexcolordesc.cpp.c` (2/4)

### LScapeVertexColorDesc
- [x] `LScapeVertexColorDesc::CalcVertexColor` · 006c36d0 · legions 00654a80
- [-] `LScapeVertexColorDesc::LScapeVertexColorDesc` · 006c39a0 — constructor (struct init; Rust builds it directly)
- [x] `LScapeVertexColorDesc::StreamPack` · 006c3a20 · legions 00654dd0
- [-] `LScapeVertexColorDesc::Load` · 006c3b30 — admin-only text/file loader (no Legions match)

## `portal/shared/src/region/surface/materialgroup.cpp.c` (6/20)

### MaterialGroup
- [x] `MaterialGroup::GetRandom` · 006c44d0 · legions 00654fa0
- [-] `MaterialGroup::Destroy` · 006c4530 · legions 00655000 — DBO release (Drop frees on last drop, AGENTS.md)
- [x] `MaterialGroup::StreamPack` · 006c45a0 · legions 00655070
- [ ] `MaterialGroup::GetSubDataIDs` · 006c47d0 · legions 006552a0 — needs port: landscape material groups (sub-object DID list for async sub-object resolution, AGENTS.md)
- [x] `MaterialGroup::GetColor` · 006c4970 · legions 00655440
- [x] `MaterialGroup::GetAlpha` · 006c4b30 · legions 00655570
- [x] `MaterialGroup::GetTexture` · 006c4bd0 · legions 00655610
- [-] `MaterialGroup::MaterialGroup` · 006c4c80 — constructor (struct init; Rust builds it directly)
- [-] `MaterialGroup::~MaterialGroup` · 006c4d70 · legions 006557b0 — destructor (Rust Drop)
- [x] `MaterialGroup::GetVertexInfo` · 006c4ed0 · legions 00655910
- [-] `MaterialGroup::LoadPhysicsMaterial` · 006c4ff0 — text source loader, admin build only

### PSmartArray<LScapeVertexColorDesc>
- [ ] `PSmartArray<LScapeVertexColorDesc>::StreamPack` · 006c50f0 · legions 00655a30 — needs port: DAT counted list (DAT decode)

### PSmartArray<LScapeMaterialDesc>
- [ ] `PSmartArray<LScapeMaterialDesc>::StreamPack` · 006c5190 · legions 00655ad0 — needs port: DAT counted list (DAT decode)

### PSmartArray<LScapeAlphaDesc>
- [ ] `PSmartArray<LScapeAlphaDesc>::StreamPack` · 006c5230 · legions 00655b70 — needs port: DAT counted list (DAT decode)

### SmartArray<LScapeMaterialDesc>
- [-] `SmartArray<LScapeMaterialDesc>::AddToEnd` · 006c52d0 — SmartArray container operation (Vec replaces it)

### SmartArray<LScapeAlphaDesc>
- [-] `SmartArray<LScapeAlphaDesc>::AddToEnd` · 006c5330 — SmartArray container operation (Vec replaces it)

### MaterialGroup
- [-] `MaterialGroup::LoadVertexColor` · 006c5390 — text source loader, admin build only
- [-] `MaterialGroup::LoadMaterial` · 006c5450 — text source loader, admin build only
- [-] `MaterialGroup::LoadAlpha` · 006c54d0 — text source loader, admin build only
- [-] `MaterialGroup::Load` · 006c5550 — admin-only text/file loader (no Legions match)

## `portal/shared/src/region/surface/surfacedesc.cpp.c` (5/24)

### CSurfaceDesc
- [-] `CSurfaceDesc::Destroy` · 00522360 — DBO release (Drop frees on last drop, AGENTS.md)
- [x] `CSurfaceDesc::StreamPack` · 00522380 · legions 0050c820
- [ ] `CSurfaceDesc::GetSubDataIDs` · 00522480 — needs port: surface descriptors (terrain surface materials, road) (sub-object DID list for async sub-object resolution, AGENTS.md)
- [x] `CSurfaceDesc::GetVertexInfo` · 00522520 · legions 0050c9c0
- [x] `CSurfaceDesc::GetRoadInfo` · 00522580 · legions 0050ca20
- [x] `CSurfaceDesc::GetPhysicsMaterial` · 005225e0 · legions 0050ca80 — Physics materials: ac2-world reads each vertex's surface or road band material (LScapeSurfaceType band by pitch) and each water type's material; ac2-physics Terrain returns the land vertex, block water and interior cell materials, read by PhysicsBody::contact_materials.
- [x] `CSurfaceDesc::GetRoadPhysicsMaterial` · 00522630 · legions 0050cad0 — Physics materials: ac2-world reads each vertex's surface or road band material (LScapeSurfaceType band by pitch) and each water type's material; ac2-physics Terrain returns the land vertex, block water and interior cell materials, read by PhysicsBody::contact_materials.

### LScapeVertexColorDesc
- [-] `LScapeVertexColorDesc::operator=` · 00522680 · legions 0050cb20 — copy assignment of plain data (Rust clone/move)

### CSurfaceDesc
- [-] `CSurfaceDesc::CSurfaceDesc` · 00522710 · legions 0050cbb0 — constructor (struct init; Rust builds it directly)
- [-] `CSurfaceDesc::GetDBOType` · 00522770 — DBO type tag (Rust typed decoders)
- [-] `CSurfaceDesc::~CSurfaceDesc` · 00522780 · legions 0050cc20 — destructor (Rust Drop)

### PFileNode
- [ ] `PFileNode::GetSubName` · 00522ad0 · legions 0050cf70 — needs port: FileNode accessors (used by FromFileNode loaders; confirm DAT path)

### CSurfaceDesc
- [-] `CSurfaceDesc::Allocate` · 00522af0 — retail free-object pool (DBOCache exception, AGENTS.md)

### SmartArray<LScapeVertexColorDesc>
- [-] `SmartArray<LScapeVertexColorDesc>::operator=` · 00522b20 · legions 0050cfc0 — copy assignment of plain data (Rust clone/move)

### SmartArray<LScapeMaterialDesc>
- [-] `SmartArray<LScapeMaterialDesc>::operator=` · 00522b90 · legions 0050d030 — copy assignment of plain data (Rust clone/move)

### SmartArray<LScapeAlphaDesc>
- [-] `SmartArray<LScapeAlphaDesc>::operator=` · 00522c00 · legions 0050d0a0 — copy assignment of plain data (Rust clone/move)

### MaterialGroup
- [-] `MaterialGroup::operator=` · 00522c70 · legions 0050d110 — copy assignment of plain data (Rust clone/move)

### SmartArray<MaterialGroup>
- [-] `SmartArray<MaterialGroup>::operator=` · 00522e90 · legions 0050d330 — copy assignment of plain data (Rust clone/move)

### LScapeSurfaceType
- [-] `LScapeSurfaceType::operator=` · 00522f00 · legions 0050d3a0 — copy assignment of plain data (Rust clone/move)

### PSmartArray<LScapeSurfaceType>
- [ ] `PSmartArray<LScapeSurfaceType>::StreamPack` · 00523020 · legions 0050d4c0 — needs port: DAT counted list (DAT decode)

### SmartArray<LScapeSurfaceType>
- [-] `SmartArray<LScapeSurfaceType>::AddToEnd` · 005230c0 — SmartArray container operation (Vec replaces it)

### CSurfaceDesc
- [-] `CSurfaceDesc::LoadSurface` · 00523130 — text source loader, admin build only
- [-] `CSurfaceDesc::LoadRoad` · 00523190 — text source loader, admin build only
- [-] `CSurfaceDesc::FromFileNode` · 005231f0 — text source format (admin tools)

## `portal/shared/src/region/terrain/terraindesc.cpp.c` (4/15)

### CTerrainDesc
- [-] `CTerrainDesc::CTerrainDesc` · 00521ad0 · legions 0050c260 — constructor (struct init; Rust builds it directly)
- [-] `CTerrainDesc::GetDBOType` · 00521b00 — DBO type tag (Rust typed decoders)
- [-] `CTerrainDesc::Destroy` · 00521b10 · legions 0050c2a0 — DBO release (Drop frees on last drop, AGENTS.md)
- [-] `CTerrainDesc::pack_size` · 00521b80 · legions 0050c310 — write path (client reads DATs only)
- [ ] `CTerrainDesc::GetSubDataIDs` · 00521bf0 · legions 0050c380 — needs port: terrain descriptors and surface lookups (sub-object DID list for async sub-object resolution, AGENTS.md)
- [x] `CTerrainDesc::FindTerrainType` · 00521c40 · legions 0050c3d0 — crates/ac2-world/src/ac2_scene/format.rs CTerrainDesc::FindTerrainType
- [x] `CTerrainDesc::GetSurfaceType` · 00521c90 · legions 0050c420 — crates/ac2-world/src/ac2_terrain_desc.rs CTerrainDesc::GetSurfaceType
- [-] `CTerrainDesc::Allocate` · 00521cc0 — retail free-object pool (DBOCache exception, AGENTS.md)
- [-] `CTerrainDesc::Pack` · 00521cd0 — write path (client reads DATs only)
- [ ] `CTerrainDesc::GetScene` · 00521de0 — needs port: terrain descriptors and surface lookups
- [x] `CTerrainDesc::GetSoundType` · 00521ed0 — ac2-audio/ac2-client: ambient environments, groove
- [x] `CTerrainDesc::UnPack` · 00522000 — crates/ac2-world/src/ac2_terrain_desc.rs CTerrainDesc::UnPack
- [-] `CTerrainDesc::CheckDupAdd` · 00522140 — text source format (admin tools)
- [-] `CTerrainDesc::LoadTerrainType` · 005221b0 — text source loader, admin build only
- [-] `CTerrainDesc::FromFileNode` · 00522240 — text source format (admin tools)

## `portal/shared/src/region/terrain/terrainsceneinfo.cpp.c` (0/5)

### TerrainSceneInfo
- [-] `TerrainSceneInfo::pack` · 006c6900 · legions 006563a0 — write path (client reads DATs only)
- [ ] `TerrainSceneInfo::unpack` · 006c6950 · legions 006563f0 — needs port: terrain scene info (DAT decode)
- [-] `TerrainSceneInfo::~TerrainSceneInfo` · 006c69a0 · legions 00656440 — destructor (Rust Drop)
- [-] `TerrainSceneInfo::LoadSceneName` · 006c69b0 — text source loader, admin build only
- [-] `TerrainSceneInfo::Load` · 006c6a70 — admin-only text/file loader (no Legions match)

## `portal/shared/src/region/terrain/terraintype.cpp.c` (2/10)

### TerrainType
- [-] `TerrainType::pack_size` · 006c5fe0 · legions 00655ee0 — write path (client reads DATs only)
- [x] `TerrainType::GetSceneType` · 006c6080 · legions 00655f80 — crates/ac2-world/src/ac2_scene/sound.rs TerrainType::GetSceneType chain
- [-] `TerrainType::TerrainType` · 006c60c0 · legions 00655fc0 — constructor (struct init; Rust builds it directly)
- [-] `TerrainType::Destroy` · 006c6100 · legions 00656000 — DBO release (Drop frees on last drop, AGENTS.md)
- [-] `TerrainType::~TerrainType` · 006c61b0 · legions 006560b0 — destructor (Rust Drop)
- [-] `TerrainType::LoadScene` · 006c61f0 — text source loader, admin build only
- [-] `TerrainType::LoadSurface` · 006c6280 — text source loader, admin build only
- [-] `TerrainType::pack` · 006c6340 · legions 006560f0 — write path (client reads DATs only)
- [x] `TerrainType::unpack` · 006c6460 · legions 00656210 — crates/ac2-world/src/ac2_terrain_desc.rs TerrainType::unpack
- [-] `TerrainType::Load` · 006c65f0 — admin-only text/file loader (no Legions match)

## `portal/shared/src/region/water/waterdesc.cpp.c` (2/21)

### CWaterDesc
- [-] `CWaterDesc::CWaterDesc` · 00521060 · legions 0050bcd0 — constructor (struct init; Rust builds it directly)
- [-] `CWaterDesc::GetDBOType` · 00521090 — DBO type tag (Rust typed decoders)
- [-] `CWaterDesc::Destroy` · 005210a0 · legions 0050bd10 — DBO release (Drop frees on last drop, AGENTS.md)
- [-] `CWaterDesc::Pack` · 00521110 — write path (client reads DATs only)
- [-] `CWaterDesc::GetPackSize` · 00521240 · legions 0050beb0 — write path (client reads DATs only)
- [ ] `CWaterDesc::GetSubDataIDs` · 005212a0 · legions 0050bf10 — needs port: water descriptors (sub-object DID list for async sub-object resolution, AGENTS.md)
- [x] `CWaterDesc::FindWaterType` · 005212f0 · legions 0050bf60 — crates/ac2-world/src/ac2_water.rs CWaterDesc::FindWaterType
- [ ] `CWaterDesc::HasWater` · 00521340 · legions 0050bfb0 — needs port: water descriptors
- [ ] `CWaterDesc::GetTileWidth` · 00521360 · legions 0050bfd0 — needs port: water descriptors
- [ ] `CWaterDesc::GetAboveMaterial` · 00521380 · legions 0050bff0 — needs port: water descriptors
- [ ] `CWaterDesc::GetBelowMaterial` · 005213a0 · legions 0050c010 — needs port: water descriptors
- [ ] `CWaterDesc::GetWaterLighting` · 005213c0 · legions 0050c030 — needs port: water descriptors
- [ ] `CWaterDesc::GetVertexColor` · 005213f0 · legions 0050c060 — needs port: water descriptors
- [x] `CWaterDesc::GetPhysicsMaterial` · 00521410 · legions 0050c080 — Physics materials: ac2-world reads each vertex's surface or road band material (LScapeSurfaceType band by pitch) and each water type's material; ac2-physics Terrain returns the land vertex, block water and interior cell materials, read by PhysicsBody::contact_materials.
- [-] `CWaterDesc::Allocate` · 00521430 — retail free-object pool (DBOCache exception, AGENTS.md)
- [ ] `CWaterDesc::UnPack` · 00521480 — needs port: water descriptors (DAT decode)
- [-] `CWaterDesc::ToFileNode` · 005215f0 — text source format (admin tools)
- [-] `CWaterDesc::CheckDupAdd` · 00521780 — text source format (admin tools)
- [-] `CWaterDesc::LoadID` · 005217f0 — text source loader, admin build only
- [-] `CWaterDesc::LoadWater` · 005218d0 — text source loader, admin build only
- [-] `CWaterDesc::FromFileNode` · 00521960 — text source format (admin tools)

## `portal/shared/src/region/water/waterlighting.cpp.c` (0/7)

### WaterLighting
- [-] `WaterLighting::WaterLighting` · 006ccf30 · legions 00658020 — constructor (struct init; Rust builds it directly)

### FogType
- [-] `FogType::pack` · 006ccf40 · legions 00658030 — write path (client reads DATs only)

### WaterLighting
- [ ] `WaterLighting::unpack` · 006ccf70 · legions 00658060 — needs port: water lighting (DAT decode)
- [ ] `WaterLighting::GetLight` · 006ccfb0 · legions 006580b0 — needs port: water lighting
- [-] `WaterLighting::~WaterLighting` · 006ccfe0 · legions 006580e0 — destructor (Rust Drop)
- [-] `WaterLighting::FromFileNode` · 006cd000 — text source format (admin tools)
- [-] `WaterLighting::ToFileNode` · 006cd160 — text source format (admin tools)

## `portal/shared/src/region/water/watertype.cpp.c` (0/14)

### WaterLightInfo
- [-] `WaterLightInfo::WaterLightInfo` · 006cc080 · legions 00657a10 — constructor (struct init; Rust builds it directly)

### WaterType
- [ ] `WaterType::GetSubDataIDs` · 006cc0a0 · legions 00657a30 — needs port: water types (sub-object DID list for async sub-object resolution, AGENTS.md)
- [-] `WaterType::WaterType` · 006cc140 · legions 00657ad0 — constructor (struct init; Rust builds it directly)
- [-] `WaterType::Destroy` · 006cc1a0 · legions 00657b30 — DBO release (Drop frees on last drop, AGENTS.md)
- [-] `WaterType::pack` · 006cc240 · legions 00657bd0 — write path (client reads DATs only)
- [-] `WaterType::GetPackSize` · 006cc350 · legions 00657ce0 — write path (client reads DATs only)
- [-] `WaterType::~WaterType` · 006cc450 · legions 00657de0 — destructor (Rust Drop)
- [ ] `WaterType::unpack` · 006cc470 · legions 00657e00 — needs port: water types (DAT decode)
- [-] `WaterType::ToFileNode` · 006cc5e0 — text source format (admin tools)
- [-] `WaterType::LoadMaterial` · 006cc8a0 — text source loader, admin build only
- [-] `WaterType::LoadPhysMat` · 006cc9a0 — text source loader, admin build only
- [-] `WaterType::LoadLight` · 006ccad0 — text source loader, admin build only
- [-] `WaterType::FromFileNode` · 006ccb30 — text source format (admin tools)
- [ ] `WaterType::GetWaterLighting` · 006cce80 · legions 00657f70 — needs port: water types
