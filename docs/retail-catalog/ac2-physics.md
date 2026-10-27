# ac2-physics: Physics objects, collision, motion, paths, BSP

2124 retail functions: 484 done, 97 partial, 765 to do, 778 skipped.
Legend and workflow: [README](README.md).

## `portal/client/src/physics/cphysics.cpp.c` (3/9)

### CPhysics
- [-] `CPhysics::CPhysics` · 007321b0 — C++ constructor/destructor; Rust constructor and Drop replace it
- [~] `CPhysics::GetTickCount` · 00732200 — Timer::local_time drives the physics clock
- [-] `CPhysics::~CPhysics` · 00732210 — C++ constructor/destructor; Rust constructor and Drop replace it
- [~] `CPhysics::UpdatePhysics` · 00732240 — the player body only (CPhysics::UpdateAllPhysics over remote objects not simulated)
- [ ] `CPhysics::UpdateAnimation` · 00732280 — needs port (check): cited only loosely (a comment, not the function's body); verify and cite, or port
- [x] `CPhysics::UpdateRenderInterpolation` · 007322c0 · legions 006ae760 — PhysicsClock::render_interpolation
- [x] `CPhysics::GetObjectA` · 00732380 · legions 006ae820 — CPhysics::GetObjectA - CPhysics::GetObjectA: object lookup by id, ac2-client client_objects.rs
- [~] `CPhysics::UpdateAllPhysics` · 00732400 — the player's quantum with PlayerPhysicsUpdatedCallback, then every other object in the world and in range
- [x] `CPhysics::UpdateAllAnimation` · 007325a0 — ac2-client: the player's and every object's body

## `portal/shared/src/bsp/bspleaf.cpp.c` (7/13)

### BSPLEAF
- [-] `BSPLEAF::BSPLEAF` · 0049f670 · legions 00496a60 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `BSPLEAF::GetPackSize` · 0049f690 — DBO/archive serialisation of runtime state
- [-] `BSPLEAF::Pack` · 0049f6e0 — DBO/archive serialisation of runtime state
- [x] `BSPLEAF::sphere_intersects_poly` · 0049f770 — ac2-physics bsp/node.rs (Legions identical)
- [x] `BSPLEAF::sphere_intersects_solid` · 0049f7e0 — ac2-physics bsp/node.rs (Legions identical)
- [ ] `BSPLEAF::point_intersects_solid` · 0049f860 — needs port: BSP point-in-solid query
- [x] `BSPLEAF::hits_walkable` · 0049f870 — ac2-physics bsp/node.rs (Legions identical)
- [x] `BSPLEAF::find_walkable` · 0049f8f0 — ac2-physics bsp/node.rs (Legions identical)
- [x] `BSPLEAF::sphere_intersects_solid_poly` · 0049f980 · legions 00496d70 — ac2-physics bsp: placement insert (Legions identical)
- [x] `BSPLEAF::TraceRay` · 0049fa10 — ac2-physics los (Terrain::los_check)
- [x] `BSPLEAF::UnPack` · 0049fa40 — BSPLEAF::UnPack - BSP 'LEAF'/'NODE' tag decoder, ac2-assets ac2_physics_mesh.rs
- [-] `BSPLEAF::FromFileStream` · 0049fb00 — text source format
- [-] `BSPLEAF::FromFileNode` · 0049fc30 — text source format (admin tools)

## `portal/shared/src/bsp/bspnode.cpp.c` (10/26)

### Sphere
- [x] `Sphere::Intersects` · 0049ce40 · legions 00494860

### BSPNODE
- [-] `BSPNODE::BSPNODE` · 0049cea0 · legions 004948c0 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `BSPNODE::Destroy` · 0049ced0 · legions 004948f0 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [-] `BSPNODE::GetPackSizeChild` · 0049cf10 · legions 00494930 — DBO/archive serialisation of runtime state
- [-] `BSPNODE::PackChild` · 0049cf20 · legions 00494940 — DBO/archive serialisation of runtime state
- [x] `BSPNODE::UnPackChild` · 0049cf50 · legions 00494970 — BSPNODE::UnPackChild - BSP 'LEAF'/'NODE' tag decoder, ac2-assets ac2_physics_mesh.rs
- [x] `BSPNODE::point_inside_cell_bsp` · 0049cff0 · legions 00494a10 — ac2-physics envcell.rs, find_cells.rs: interior collision, containment and portal walk (Legions identical)
- [x] `BSPNODE::sphere_intersects_poly` · 0049d040 — ac2-physics bsp/node.rs (Legions identical)
- [x] `BSPNODE::sphere_intersects_solid` · 0049d100 — ac2-physics bsp/node.rs (Legions identical)
- [ ] `BSPNODE::point_intersects_solid` · 0049d1e0 — needs port: BSP point-in-solid query
- [x] `BSPNODE::hits_walkable` · 0049d230 — ac2-physics bsp/node.rs (Legions identical)
- [x] `BSPNODE::find_walkable` · 0049d2e0 — ac2-physics bsp/node.rs (Legions identical)
- [x] `BSPNODE::sphere_intersects_solid_poly` · 0049d3b0 — ac2-physics bsp: placement insert (Legions identical)
- [ ] `BSPNODE::ClipToCellBSP` · 0049d4b0 · legions 00494ed0 — needs port: BSPNODE::ClipToCellBSP
- [-] `BSPNODE::~BSPNODE` · 0049d500 · legions 00494f20 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `BSPNODE::GetPackSize` · 0049d540 · legions 00494f60 — DBO/archive serialisation of runtime state
- [-] `BSPNODE::Pack` · 0049d580 — DBO/archive serialisation of runtime state
- [x] `BSPNODE::UnPack` · 0049d5f0 · legions 00495010 — BSPNODE::UnPack - BSP 'LEAF'/'NODE' tag decoder, ac2-assets ac2_physics_mesh.rs
- [ ] `BSPNODE::ChildFromFileStream` · 0049d690 — needs port: BSPNODE::ChildFromFileStream
- [-] `BSPNODE::FromFileStream` · 0049d770 — text source format
- [x] `BSPNODE::TraceRay` · 0049d880 — ac2-physics los (Terrain::los_check)

### PFileNode
- [-] `PFileNode::GetLeafAsPString` · 0049dcb0 — admin/debug or text-file tooling

### BSPNODE
- [ ] `BSPNODE::GetNodeType` · 0049dd20 — needs port: BSP node query
- [-] `BSPNODE::ChildFromFileNode` · 0049dec0 — text source format
- [-] `BSPNODE::FromFileNode` · 0049df70 — text source format (admin tools)
- [ ] `BSPNODE::GetPosPlanes` · 0049e140 · legions 00495530 — needs port: BSP node query

## `portal/shared/src/bsp/bsptree.cpp.c` (17/20)

### Plane
- [x] `Plane::Scale` · 0049b420 · legions 004932c0 — ac2-physics bsp/tree.rs, position.rs: physics-BSP collision in mesh space (asm float order; Legions identical)

### Position
- [x] `Position::localtoglobalvec` · 0049b490 · legions 00493330 — ac2-physics bsp/tree.rs, position.rs: physics-BSP collision in mesh space (asm float order; Legions identical)

### SPHEREPATH
- [x] `SPHEREPATH::set_neg_poly_hit` · 0049b4f0 · legions 00493390 — ac2-physics transition/localspace.rs, sphere_path.rs

### COLLISIONINFO
- [x] `COLLISIONINFO::set_contact_plane` · 0049b540 · legions 004933e0

### BSPTREE
- [x] `BSPTREE::GetBoundingSphere` · 0049b580 · legions 00493420 — ac2-physics envcell.rs, find_cells.rs: interior collision, containment and portal walk (Legions identical)
- [x] `BSPTREE::point_inside_cell_bsp` · 0049b590 · legions 00493430 — ac2-physics envcell.rs, find_cells.rs: interior collision, containment and portal walk (Legions identical)
- [x] `BSPTREE::TraceRay` · 0049b5a0 · legions 00493440 — ac2-physics los (Terrain::los_check)
- [x] `BSPTREE::adjust_to_plane` · 0049b5f0 · legions 00493490 — ac2-physics bsp/tree.rs, position.rs: physics-BSP collision in mesh space (asm float order; Legions identical)
- [x] `BSPTREE::slide_sphere` · 0049b800 · legions 004936a0 — ac2-physics bsp/tree.rs, position.rs: physics-BSP collision in mesh space (asm float order; Legions identical)
- [x] `BSPTREE::step_sphere_up` · 0049b880 · legions 00493720 — ac2-physics bsp/tree.rs, position.rs: physics-BSP collision in mesh space (asm float order; Legions identical)
- [-] `BSPTREE::~BSPTREE` · 0049b920 · legions 004937c0 — C++ constructor/destructor; Rust constructor and Drop replace it
- [ ] `BSPTREE::StreamPack` · 0049b940 · legions 004937e0 — needs port: BSP tree read
- [x] `BSPTREE::collide_with_pt` · 0049ba20 · legions 004938c0 — ac2-physics bsp/tree.rs, position.rs: physics-BSP collision in mesh space (asm float order; Legions identical)
- [x] `BSPTREE::check_walkable` · 0049bba0 · legions 00493a40 — ac2-physics bsp/tree.rs, position.rs: physics-BSP collision in mesh space (asm float order; Legions identical)

### SPHEREPATH
- [x] `SPHEREPATH::set_collide` · 0049bbf0 · legions 00493a90 — ac2-physics transition/localspace.rs, sphere_path.rs
- [x] `SPHEREPATH::set_walkable` · 0049bc60 · legions 00493b00 — ac2-physics transition/localspace.rs, sphere_path.rs

### BSPTREE
- [x] `BSPTREE::placement_insert` · 0049bcf0 · legions 00493b90 — ac2-physics bsp: placement insert (Legions identical)
- [x] `BSPTREE::step_sphere_down` · 0049bf70 · legions 00493e10 — ac2-physics bsp/tree.rs, position.rs: physics-BSP collision in mesh space (asm float order; Legions identical)
- [-] `BSPTREE::ReadBSPTree` · 0049c180 — admin/debug or text-file tooling
- [x] `BSPTREE::FindObjCollisions` · 0049c2c0 · legions 00494020 — ac2-physics bsp/tree.rs, position.rs: physics-BSP collision in mesh space (asm float order; Legions identical)

## `portal/shared/src/path/astar.cpp.c` (0/14)

### PathSearchStats
- [-] `PathSearchStats::RecordBegin` · 006324d0 — admin/debug or text-file tooling

### MarkerHeap
- [-] `MarkerHeap::expand` · 00632590 · legions 005e9ed0 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### AStar
- [ ] `AStar::MetaFind` · 00632600 · legions 005e9f40 — needs port: path search

### PathSearchStats
- [-] `PathSearchStats::RecordEnd` · 00632680 · legions 005e9fc0 — admin/debug or text-file tooling

### MarkerHeap
- [-] `MarkerHeap::heapify` · 00632770 · legions 005ea0b0 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it
- [-] `MarkerHeap::insert` · 006327f0 · legions 005ea130 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it
- [-] `MarkerHeap::heapify_up` · 00632870 · legions 005ea1b0 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### AStar
- [ ] `AStar::MetaEnqueue` · 006328d0 · legions 005ea210 — needs port: path search
- [ ] `AStar::MetaDequeue` · 00632920 — needs port: path search
- [ ] `AStar::MetaUpdate` · 006329a0 · legions 005ea2e0 — needs port: path search
- [ ] `AStar::MetaAdd` · 00632a10 · legions 005ea350 — needs port: path search

### MarkerPool
- [-] `MarkerPool::GetNewMarker` · 00632a90 · legions 005ea3d0 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### AStar
- [ ] `AStar::Relax` · 00632b10 · legions 005ea450 — needs port: path search
- [ ] `AStar::FindPath` · 00632bd0 — needs port: path search

## `portal/shared/src/path/bsp/pathbspleaf.cpp.c` (0/22)

### PathBSPLeaf
- [-] `PathBSPLeaf::PathBSPLeaf` · 006357d0 · legions 005ecad0 — C++ constructor/destructor; Rust constructor and Drop replace it
- [ ] `PathBSPLeaf::FindNodeID` · 006357f0 — needs port: path search

### LM_UVTransform
- [-] `LM_UVTransform::LoadFromFileNode` · 00635800 · legions 005ecb00 — admin/debug or text-file tooling

### gmWeenieDesc
- [ ] `gmWeenieDesc::AdjLongInt` · 00635810 — needs port: gmWeenieDesc::AdjLongInt

### PathBSPLeaf
- [-] `PathBSPLeaf::PathBSPLeaf` · 00635820 · legions 005ecb20 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `PathBSPLeaf::Destroy` · 00635840 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [-] `PathBSPLeaf::CopyTo` · 00635850 — path-map builder/editor
- [ ] `PathBSPLeaf::GetNodesWithin` · 00635870 — needs port: path search
- [ ] `PathBSPLeaf::GetNodesWithinXY` · 00635890 — needs port: path search
- [-] `PathBSPLeaf::LabelAllLeaves` · 006358b0 — path-map builder/editor

### PathNodeEdge
- [-] `PathNodeEdge::PathNodeEdge` · 006358d0 — C++ constructor/destructor; Rust constructor and Drop replace it

### PathBSPLeaf
- [ ] `PathBSPLeaf::StreamPack` · 00635900 — needs port: path-map archive read
- [-] `PathBSPLeaf::ToFileNode` · 00635930 — text source format (admin tools)
- [ ] `PathBSPLeaf::CheckNodesVsPlanes` · 00635950 — needs port: path search
- [-] `PathBSPLeaf::FromFileNode` · 006359e0 — text source format (admin tools)

### PathNodeEdge
- [-] `PathNodeEdge::operator=` · 00635a10 · legions 005eccc0 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it

### PathBSPLeaf
- [-] `PathBSPLeaf::SplitBySegment` · 00635b30 — path-map builder/editor
- [-] `PathBSPLeaf::CalcNodesInVolume` · 00635b50 · legions 005ece00 — path-map builder/editor
- [-] `PathBSPLeaf::CalcNodePolygons` · 00635ba0 — path-map builder/editor

### SmartArray<PathNodeEdge>
- [-] `SmartArray<PathNodeEdge>::AddToEnd` · 00635c20 · legions 005eced0 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### PathBSPLeaf
- [ ] `PathBSPLeaf::FindNodeList` · 00635ca0 — needs port: path search
- [-] `PathBSPLeaf::UpdateNodeIDs` · 00636120 — path-map builder/editor

## `portal/shared/src/path/bsp/pathbspnode.cpp.c` (0/45)

### PathBSPNode
- [-] `PathBSPNode::PathBSPNode` · 006331d0 · legions 005ea830 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `PathBSPNode::ClearChildrenWithoutDeleting` · 00633200 — path-map builder/editor
- [-] `PathBSPNode::Destroy` · 00633210 · legions 005ea870 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [-] `PathBSPNode::CopyTo` · 00633240 — path-map builder/editor
- [-] `PathBSPNode::SetPlane` · 006332e0 — path-map builder/editor
- [-] `PathBSPNode::MakePosNode` · 00633300 — path-map builder/editor
- [-] `PathBSPNode::MakeNegNode` · 00633360 — path-map builder/editor
- [-] `PathBSPNode::MakePosLeaf` · 006333c0 · legions 005eaa20 — path-map builder/editor
- [-] `PathBSPNode::MakeNegLeaf` · 00633410 · legions 005eaa70 — path-map builder/editor
- [-] `PathBSPNode::ReplaceLeafWithNode` · 00633460 · legions 005eaac0 — path-map builder/editor
- [ ] `PathBSPNode::FindNodeID` · 00633540 — needs port: path search
- [-] `PathBSPNode::UpdateLeafToNode` · 006335a0 — path-map builder/editor
- [-] `PathBSPNode::Transform` · 006335d0 — path-map builder/editor
- [ ] `PathBSPNode::GetNodesWithin` · 00633700 · legions 005ead60 — needs port: path search
- [-] `PathBSPNode::SplitSubNodeBySegment` · 00633750 — path-map builder/editor
- [-] `PathBSPNode::MakePlaneFromSegment` · 006337d0 — path-map builder/editor
- [-] `PathBSPNode::LabelAllLeaves` · 006338a0 · legions 005eaef0 — path-map builder/editor
- [ ] `PathBSPNode::PosIsLeaf` · 006338e0 — needs port: path graph query
- [ ] `PathBSPNode::NegIsLeaf` · 006338f0 — needs port: path graph query
- [-] `PathBSPNode::FindAndSplitNode` · 00633900 · legions 005eaf50 — path-map builder/editor
- [-] `PathBSPNode::ReplaceLeafWithTree` · 00633a10 · legions 005eb060 — path-map builder/editor
- [-] `PathBSPNode::RemoveLeafAndAddTree` · 00633a80 — path-map builder/editor
- [-] `PathBSPNode::InitChildAsNode` · 00633b30 — path-map builder/editor
- [-] `PathBSPNode::InitChildAsLeaf` · 00633ba0 — path-map builder/editor
- [-] `PathBSPNode::UpdateNodeIDs` · 00633c10 · legions 005eb260 — path-map builder/editor
- [-] `PathBSPNode::CheckCellPlanes` · 00633c40 · legions 005eb290 — path-map builder/editor
- [-] `PathBSPNode::RemoveLeaf` · 00633c80 · legions 005eb2d0 — path-map builder/editor
- [-] `PathBSPNode::SetAsChild` · 00633d90 · legions 005eb3e0 — path-map builder/editor

### (free functions)
- [-] `STREAMPACK_STREAMPACKOBJ` · 00633e30 · legions 005eb480 — STL/MSVC or archive template helper; Rust collections replace it

### PathBSPNode
- [ ] `PathBSPNode::StreamPack` · 00633e50 · legions 005eb4a0 — needs port: path-map archive read
- [-] `PathBSPNode::SplitBySegment` · 00633f50 — path-map builder/editor
- [-] `PathBSPNode::CalcNodePolygons` · 006342e0 — path-map builder/editor
- [ ] `PathBSPNode::GetNodesWithinXY` · 00634440 — needs port: path search
- [ ] `PathBSPNode::FindNodeList` · 00634790 — needs port: path search
- [-] `PathBSPNode::MakeVerticalSplit` · 00634960 — path-map builder/editor
- [-] `PathBSPNode::GetSplitSide` · 00634ac0 — path-map builder/editor
- [ ] `PathBSPNode::ToSubNode` · 00634bc0 — needs port: path-map archive read
- [ ] `PathBSPNode::FromSubNode` · 00634c30 — needs port: path-map archive read

### SmartArray<Plane>
- [-] `SmartArray<Plane>::operator=` · 00634d80 · legions 005ec210 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### PathBSPNode
- [-] `PathBSPNode::ToFileNode` · 00634df0 — text source format (admin tools)
- [-] `PathBSPNode::FromFileNode` · 00634e80 — text source format (admin tools)
- [-] `PathBSPNode::CalcNodesInVolume` · 00634f80 — path-map builder/editor
- [ ] `PathBSPNode::CheckNodesVsPlanes` · 00635380 — needs port: path search
- [ ] `PathBSPNode::GetPlanes` · 00635470 — needs port: path graph query
- [-] `PathBSPNode::FindPlanePolys` · 00635530 — path-map builder/editor

## `portal/shared/src/path/bsp/pathbsptree.cpp.c` (0/6)

### PathBSPTree
- [ ] `PathBSPTree::FindNodeID` · 00632de0 · legions 005ea720 — needs port: path search
- [ ] `PathBSPTree::GetAndRemoveRoot` · 00632e00 · legions 005ea740 — needs port: path search
- [-] `PathBSPTree::~PathBSPTree` · 00632e10 · legions 005ea750 — C++ constructor/destructor; Rust constructor and Drop replace it
- [ ] `PathBSPTree::StreamPack` · 00632e30 · legions 005ea770 — needs port: path-map archive read
- [-] `PathBSPTree::ToFileNode` · 00632ed0 — text source format (admin tools)
- [-] `PathBSPTree::FromFileNode` · 00632f90 — text source format (admin tools)

## `portal/shared/src/path/finder/pathfinder.cpp.c` (0/21)

### PathFinder
- [-] `PathFinder::SetHeight` · 00631090 · legions 005e8ac0 — path-map builder/editor
- [-] `PathFinder::SetWidth` · 00631100 · legions 005e8b30 — path-map builder/editor
- [ ] `PathFinder::SetStart` · 00631170 · legions 005e8ba0 — needs port: path search
- [ ] `PathFinder::SetEnd` · 006311b0 · legions 005e8be0 — needs port: path search
- [ ] `PathFinder::GetPackSize` · 006311f0 · legions 005e8c20 — needs port: path system
- [ ] `PathFinder::Pack` · 00631220 · legions 005e8c50 — needs port: path system
- [ ] `PathFinder::UnPack` · 006312f0 · legions 005e8d20 — needs port: path-map archive read
- [ ] `PathFinder::SetStartPosition` · 00631450 · legions 005e8e80 — needs port: path search
- [ ] `PathFinder::SetEndPosition` · 006314d0 · legions 005e8f00 — needs port: path search
- [ ] `PathFinder::HasAnySolutions` · 00631690 — needs port: path search
- [ ] `PathFinder::HasMarkerForSolution` · 006316a0 · legions 005e90d0 — needs port: path search

### HashTable<unsigned_long,unsigned_long>
- [-] `HashTable<unsigned_long,unsigned_long>::operator=` · 006316e0 · legions 00505890 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### PathFinder
- [ ] `PathFinder::SetMarkerForSolution` · 006318b0 · legions 005e91f0 — needs port: path search

### Path
- [-] `Path::operator=` · 00631960 · legions 005e92a0 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it

### PathFinder
- [ ] `PathFinder::InqSolution` · 00631b70 · legions 005e94b0 — needs port: path search
- [ ] `PathFinder::SetupSolutions` · 00631c30 — needs port: path search

### HashTable<unsigned_long,Path>
- [-] `HashTable<unsigned_long,Path>::operator=` · 00631e80 · legions 005e97c0 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### PHashTable<unsigned_long,Path>
- [-] `PHashTable<unsigned_long,Path>::StreamPack` · 00631fe0 · legions 005e9920 — DBO/archive serialisation of runtime state; the client does not read it from the DAT

### PathFinder
- [-] `PathFinder::PathFinder` · 00632240 · legions 005e9b80 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `PathFinder::operator=` · 00632340 · legions 005e9c80 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [-] `PathFinder::~PathFinder` · 00632400 · legions 005e9d40 — C++ constructor/destructor; Rust constructor and Drop replace it

## `portal/shared/src/path/finder/pathfinder_normal.cpp.c` (0/15)

### PathFinder_Normal
- [-] `PathFinder_Normal::PathFinder_Normal` · 0062fc80 · legions 005e7e70 — C++ constructor/destructor; Rust constructor and Drop replace it
- [ ] `PathFinder_Normal::IsRandom` · 0062fcf0 · legions 005e7ee0 — needs port: path search
- [-] `PathFinder_Normal::operator=` · 0062fd10 · legions 005e7f00 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [ ] `PathFinder_Normal::VerifyData` · 0062fdb0 · legions 005e7fa0 — needs port: path search
- [ ] `PathFinder_Normal::SetStartMarker` · 0062fdf0 · legions 005e7fe0 — needs port: path search
- [ ] `PathFinder_Normal::CheckNode` · 0062fe20 · legions 005e8010 — needs port: path search
- [ ] `PathFinder_Normal::IsAcceptable` · 0062ff60 · legions 005e8150 — needs port: path search
- [ ] `PathFinder_Normal::GetCost` · 006300e0 · legions 005e82d0 — needs port: path search
- [ ] `PathFinder_Normal::GetEstimate` · 00630170 · legions 005e8360 — needs port: path search
- [ ] `PathFinder_Normal::UpdateMarker` · 006301d0 · legions 005e83c0 — needs port: path search
- [ ] `PathFinder_Normal::Pack` · 00630200 · legions 005e83f0 — needs port: path system
- [ ] `PathFinder_Normal::UnPack` · 006302e0 · legions 005e84d0 — needs port: path-map archive read
- [-] `PathFinder_Normal::PathFinder_Normal` · 00630470 · legions 005e8660 — C++ constructor/destructor; Rust constructor and Drop replace it
- [ ] `PathFinder_Normal::GetPackSize` · 00630490 · legions 005e8680 — needs port: path system
- [-] `PathFinder_Normal::ToString` · 006304a0 — admin/debug or text-file tooling

## `portal/shared/src/path/obstacle/footprint.cpp.c` (0/10)

### FootPrint
- [-] `FootPrint::StreamPack` · 00637fb0 · legions 005f6110 — DBO/archive serialisation of runtime state; the client does not read it from the DAT
- [-] `FootPrint::FootPrint` · 00638000 · legions 005ee820 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `FootPrint::~FootPrint` · 00638030 · legions 005ee850 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `FootPrint::ToFileNode` · 006380c0 — text source format (admin tools)

### PSmartArray<BBox>
- [-] `PSmartArray<BBox>::StreamPack` · 006382b0 · legions 005ee8e0 — DBO/archive serialisation of runtime state; the client does not read it from the DAT

### PSmartArray<Cylsphere>
- [-] `PSmartArray<Cylsphere>::StreamPack` · 00638350 · legions 005ee980 — DBO/archive serialisation of runtime state; the client does not read it from the DAT

### SmartArray<Cylsphere>
- [-] `SmartArray<Cylsphere>::AddToEnd` · 006383f0 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### FootPrint
- [-] `FootPrint::LoadBox` · 00638460 — text source loader, admin build only
- [-] `FootPrint::LoadCylinder` · 006384e0 — text source loader, admin build only
- [-] `FootPrint::FromFileNode` · 00638570 — text source format (admin tools)

## `portal/shared/src/path/obstacle/obstacle.cpp.c` (1/9)

### Position
- [x] `Position::localtoglobal` · 00636170 · legions 005ed420 — Position::localtoglobal - Position::local_to_global, ac2-physics position.rs

### Obstacle
- [ ] `Obstacle::CheckIfSolid` · 006361c0 · legions 005ed470 — needs port: path obstacle geometry
- [ ] `Obstacle::Within` · 00636350 · legions 005ed600 — needs port: path obstacle geometry

### PathExternalLink
- [-] `PathExternalLink::operator=` · 00636640 · legions 005ed8f0 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it

### Obstacle
- [ ] `Obstacle::GetGlobalBoundingBox` · 006366a0 · legions 005ed950 — needs port: path obstacle geometry
- [ ] `Obstacle::GetGlobalCornersFromBox` · 00636b10 · legions 005eddc0 — needs port: path obstacle geometry
- [ ] `Obstacle::GetGlobalCornersFromCylinder` · 00636d30 · legions 005edfe0 — needs port: path obstacle geometry
- [ ] `Obstacle::CheckIfSolid` · 00636ec0 · legions 005ee170 — needs port: path obstacle geometry

### SmartArray<PathExternalLink>
- [-] `SmartArray<PathExternalLink>::AddToEnd` · 006370c0 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

## `portal/shared/src/path/obstacle/obstacledesc.cpp.c` (0/9)

### ObstacleDesc
- [-] `ObstacleDesc::ObstacleDesc` · 00637150 · legions 005ee370 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `ObstacleDesc::GetDBOType` · 006371d0 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [-] `ObstacleDesc::Destroy` · 006371e0 · legions 005ee400 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [-] `ObstacleDesc::Allocate` · 00637290 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [-] `ObstacleDesc::~ObstacleDesc` · 006372a0 · legions 005ee4c0 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `ObstacleDesc::ToFileNode` · 00637340 — text source format (admin tools)

### PSmartArray<PathExternalLink>
- [-] `PSmartArray<PathExternalLink>::StreamPack` · 00637690 · legions 005ee560 — DBO/archive serialisation of runtime state; the client does not read it from the DAT

### ObstacleDesc
- [ ] `ObstacleDesc::StreamPack` · 00637750 · legions 005ee620 — needs port: ObstacleDesc DAT record decode
- [-] `ObstacleDesc::FromFileNode` · 00637860 — text source format (admin tools)

## `portal/shared/src/path/obstacle/pathexternallink.cpp.c` (0/5)

### PathExternalLink
- [-] `PathExternalLink::PathExternalLink` · 00637b90 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `PathExternalLink::~PathExternalLink` · 00637be0 · legions 005ee780 — C++ constructor/destructor; Rust constructor and Drop replace it
- [ ] `PathExternalLink::StreamPack` · 00637c10 · legions 005ee7b0 — needs port: path-map archive read
- [-] `PathExternalLink::ToFileNode` · 00637c80 — text source format (admin tools)
- [-] `PathExternalLink::FromFileNode` · 00637db0 — text source format (admin tools)

## `portal/shared/src/path/obstacle/undersideinfo.cpp.c` (0/8)

### UndersideInfo
- [ ] `UndersideInfo::GetNumTotal` · 006386a0 — needs port: underside info
- [-] `UndersideInfo::Destroy` · 006386d0 · legions 005eea20 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [-] `UndersideInfo::StreamPack` · 00638770 · legions 005eeac0 — DBO/archive serialisation of runtime state; the client does not read it from the DAT
- [-] `UndersideInfo::UndersideInfo` · 00638810 · legions 005eeb60 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `UndersideInfo::~UndersideInfo` · 00638880 · legions 005eebd0 — C++ constructor/destructor; Rust constructor and Drop replace it

### PSmartArray<unsigned_long>
- [~] `PSmartArray<unsigned_long>::StreamPack` · 006388c0 · legions 004554e0 — unpack only, in ac2-net protocol::fx / protocol::visual (Legions FXNode::StreamPack 0x00642410 instruction-identical)

### UndersideInfo
- [-] `UndersideInfo::ToFileNode` · 006389b0 — text source format (admin tools)
- [-] `UndersideInfo::FromFileNode` · 00638b60 — text source format (admin tools)

## `portal/shared/src/path/path.cpp.c` (0/6)

### Path
- [ ] `Path::SmoothPositionList` · 0062b640 · legions 005e5560 — needs port: path system
- [ ] `Path::StreamPack` · 0062b720 · legions 005e5640 — needs port: path-map archive read

### SmartArray<Position>
- [-] `SmartArray<Position>::AddToEnd` · 0062b8f0 · legions 005e5810 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### Path
- [-] `Path::ToString` · 0062b950 · legions 005e5870 — admin/debug or text-file tooling
- [-] `Path::ToStringW` · 0062b9f0 · legions 005e5910 — admin/debug or text-file tooling
- [ ] `Path::TranslateToPositionList` · 0062ba40 · legions 005e5960 — needs port: path system

## `portal/shared/src/path/pathlink.cpp.c` (0/5)

### PathLink
- [ ] `PathLink::Link` · 0062f020 · legions 005e7900 — needs port: path-map archive read
- [ ] `PathLink::StreamPack` · 0062f040 · legions 005e7920 — needs port: path-map archive read
- [ ] `PathLink::GetPortalConstrained` · 0062f0b0 — needs port: path search
- [-] `PathLink::ToFileNode` · 0062f430 — text source format (admin tools)
- [-] `PathLink::FromFileNode` · 0062f4e0 — text source format (admin tools)

## `portal/shared/src/path/pathlinkdata.cpp.c` (0/7)

### PathNode
- [ ] `PathNode::FindGridPt` · 0062de20 · legions 005e6e20 — needs port: path search

### PathLinkData
- [ ] `PathLinkData::Compress` · 0062de90 · legions 005e6e90 — needs port: path-map archive read
- [ ] `PathLinkData::Uncompress` · 0062df00 · legions 005e6f00 — needs port: path-map archive read
- [-] `PathLinkData::PathLinkData` · 0062df90 · legions 005e7020 — C++ constructor/destructor; Rust constructor and Drop replace it
- [ ] `PathLinkData::StreamPack` · 0062dfe0 · legions 005e7070 — needs port: path-map archive read
- [-] `PathLinkData::ToFileNode` · 0062e250 — text source format (admin tools)
- [-] `PathLinkData::FromFileNode` · 0062e400 — text source format (admin tools)

## `portal/shared/src/path/pathmap.cpp.c` (0/37)

### FileNodeName_Hex
- [-] `FileNodeName_Hex::Clone` · 0062bd10 · legions 005e5c30 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it

### PFileNode
- [-] `PFileNode::SetNameAsHex` · 0062bd40 · legions 005e5c60 — admin/debug or text-file tooling
- [-] `PFileNode::MakeSubNodeAsHex` · 0062bd90 — admin/debug or text-file tooling

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,SimplePolygon_*>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,SimplePolygon_*>_*>::~IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,SimplePolygon_*>_*>` · 0062bdf0 · legions 005e5cb0 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### PathMap
- [-] `PathMap::ClearNodePolygons` · 0062bee0 · legions 005e5da0 — path-map builder/editor
- [-] `PathMap::CountLinks` · 0062bf80 · legions 005e5e40 — path-map builder/editor
- [-] `PathMap::LinkPathNodes` · 0062bfe0 · legions 005e5ea0 — path-map builder/editor
- [-] `PathMap::LoadFromFile` · 0062c050 — text source loader, admin build only
- [-] `PathMap::SaveToFile` · 0062c190 — admin/debug or text-file tooling
- [-] `PathMap::SavePathPolygon` · 0062c2b0 — admin/debug or text-file tooling
- [-] `PathMap::ClearTable` · 0062c3b0 · legions 005e5f10 — admin/debug or text-file tooling
- [-] `PathMap::ClearBSP` · 0062c430 · legions 005e5f90 — path-map builder/editor
- [ ] `PathMap::GetNumLinks` · 0062c4b0 · legions 005e6010 — needs port: path graph query
- [-] `PathMap::Dump` · 0062c4d0 · legions 005e6030 — admin/debug or text-file tooling
- [-] `PathMap::ToFileNode` · 0062c630 — text source format (admin tools)
- [-] `PathMap::Clear` · 0062ccc0 · legions 005e6230 — path-map builder/editor
- [ ] `PathMap::GetPathNode` · 0062cd20 · legions 005e6290 — needs port: path search
- [ ] `PathMap::GetNodesWithin` · 0062cd60 · legions 005e62d0 — needs port: path search
- [-] `PathMap::~PathMap` · 0062cdd0 · legions 005e6340 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `PathMap::GetDBOType` · 0062ce80 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [ ] `PathMap::QueryPosition` · 0062ce90 · legions 005e6400 — needs port: path search
- [-] `PathMap::AddPathNode` · 0062cf00 · legions 005e6470 — path-map builder/editor
- [-] `PathMap::AddCellID` · 0062cf40 — path-map builder/editor
- [ ] `PathMap::StreamPack` · 0062cf80 — needs port: path-map archive read
- [-] `PathMap::LoadPathNode` · 0062d2e0 — text source loader, admin build only
- [-] `PathMap::LoadCellID` · 0062d370 — text source loader, admin build only
- [-] `PathMap::LoadPathBSP` · 0062d430 — text source loader, admin build only
- [ ] `PathMap::GetBSP` · 0062d4c0 · legions 005e6820 — needs port: path graph query
- [ ] `PathMap::GetNodePolygon` · 0062d530 · legions 005e6890 — needs port: path graph query
- [ ] `PathMap::CheckNodePolygon` · 0062d580 · legions 005e68e0 — needs port: path search
- [ ] `PathMap::GetNodesWithinXY` · 0062d8d0 · legions 005e6c30 — needs port: path search
- [-] `PathMap::PathMap` · 0062d950 · legions 005e6cb0 — C++ constructor/destructor; Rust constructor and Drop replace it
- [ ] `PathMap::GetPathNode` · 0062d9d0 · legions 005e6d30 — needs port: path search
- [-] `PathMap::SetNodePolygon` · 0062da30 · legions 005e6d90 — path-map builder/editor
- [-] `PathMap::Allocate` · 0062dab0 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [-] `PathMap::LoadPathPolygon` · 0062dac0 — text source loader, admin build only
- [-] `PathMap::FromFileNode` · 0062dca0 — text source format (admin tools)

## `portal/shared/src/path/pathmapdb.cpp.c` (0/8)

### PathMapDB
- [ ] `PathMapDB::GetNumLinks` · 00630ca0 — needs port: path graph query
- [ ] `PathMapDB::GetNumNodes` · 00630cc0 — needs port: path graph query
- [-] `PathMapDB::DumpPathMap` · 00630ce0 · legions 005e8710 — admin/debug or text-file tooling
- [ ] `PathMapDB::GetMap` · 00630d30 · legions 005e8760 — needs port: path search
- [ ] `PathMapDB::GetPathNode` · 00630d90 · legions 005e87c0 — needs port: path search
- [ ] `PathMapDB::GetPathNode` · 00630de0 · legions 005e8810 — needs port: path search
- [ ] `PathMapDB::GetPathNodeID` · 00630e60 · legions 005e8890 — needs port: path search
- [ ] `PathMapDB::QueryPosition` · 00630e80 · legions 005e88b0 — needs port: path search

## `portal/shared/src/path/pathnode.cpp.c` (0/13)

### PathNode
- [ ] `PathNode::GetPathMapID` · 0062e6c0 · legions 005e72e0 — needs port: path system
- [ ] `PathNode::NodeWithin` · 0062e6e0 · legions 005e7300 — needs port: path search
- [ ] `PathNode::GetNodeData` · 0062e6f0 · legions 005e7310 — needs port: path search
- [ ] `PathNode::CompressSize` · 0062e720 · legions 005e7340 — needs port: path-map archive read
- [ ] `PathNode::UncompressSize` · 0062e7e0 · legions 005e7400 — needs port: path-map archive read

### (free functions)
- [-] `STREAMPACK_STREAMPACKOBJ` · 0062e880 · legions 005e74a0 — STL/MSVC or archive template helper; Rust collections replace it

### PathNode
- [ ] `PathNode::Link` · 0062e8a0 · legions 005e74c0 — needs port: path-map archive read
- [-] `PathNode::~PathNode` · 0062e8d0 · legions 005e74f0 — C++ constructor/destructor; Rust constructor and Drop replace it
- [ ] `PathNode::Distance` · 0062e900 · legions 005e7520 — needs port: path search
- [-] `PathNode::PathNode` · 0062e940 · legions 005e7560 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `PathNode::ToFileNode` · 0062e990 — text source format (admin tools)
- [ ] `PathNode::StreamPack` · 0062eac0 — needs port: path-map archive read
- [-] `PathNode::FromFileNode` · 0062ee50 — text source format (admin tools)

## `portal/shared/src/path/pathnodedata.cpp.c` (0/7)

### PathNodeData
- [-] `PathNodeData::PathNodeData` · 0062f670 · legions 005e7d10 — C++ constructor/destructor; Rust constructor and Drop replace it

### PackObj
- [-] `PackObj::~PackObj` · 0062f6a0 · legions 004e01b0 — C++ constructor/destructor; Rust constructor and Drop replace it

### PathNodeData
- [ ] `PathNodeData::Compress` · 0062f6b0 · legions 005e7d40 — needs port: path-map archive read
- [ ] `PathNodeData::Uncompress` · 0062f720 · legions 005e7db0 — needs port: path-map archive read
- [ ] `PathNodeData::StreamPack` · 0062f780 · legions 005e7e10 — needs port: path-map archive read
- [-] `PathNodeData::ToFileNode` · 0062f7e0 — text source format (admin tools)
- [-] `PathNodeData::FromFileNode` · 0062f9d0 — text source format (admin tools)

## `portal/shared/src/path/pathsearchstats.cpp.c` (0/1)

### PathSearchStats
- [-] `PathSearchStats::Dump` · 00630570 — admin/debug or text-file tooling

## `portal/shared/src/path/pathsystem.cpp.c` (0/6)

### PathSearchStats
- [-] `PathSearchStats::Clear` · 00630ee0 — path-map builder/editor

### PathSystem
- [ ] `PathSystem::GetPathNode` · 00630f50 · legions 005e8980 — needs port: path search

### PathMapDB
- [-] `PathMapDB::ClearPathMaps` · 00630f70 · legions 005e89a0 — admin/debug or text-file tooling

### PathSystem
- [ ] `PathSystem::FindPath_Internal` · 00630fa0 · legions 00512634 — needs port: path search
- [ ] `PathSystem::InqPathNodeDataByNodeID` · 00630ff0 · legions 005e8a20 — needs port: path search
- [ ] `PathSystem::InqPathNodeDataByPosition` · 00631050 · legions 005e8a80 — needs port: path search

## `portal/shared/src/physics/cellarray.cpp.c` (1/1)

### CELLARRAY
- [x] `CELLARRAY::add_cell` · 004da7e0 · legions 004cd240 — ac2-physics find_cells.rs, terrain.rs: cell search through land cells, touching cells and portals

## `portal/shared/src/physics/collisioninfo.cpp.c` (1/1)

### COLLISIONINFO
- [x] `COLLISIONINFO::AddObject` · 007326e0 · legions 006aeb80 — ac2-physics objects.rs: static-object collision dispatch, containment radius, shadow box

## `portal/shared/src/physics/material/physicsmaterial.cpp.c` (0/12)

### PhysicsMaterial
- [-] `PhysicsMaterial::PhysicsMaterial` · 0063a270 · legions 005ef040 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `PhysicsMaterial::GetDBOType` · 0063a290 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [-] `PhysicsMaterial::GetSubDataIDs` · 0063a2a0 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [-] `PhysicsMaterial::CopyInto` · 0063a2b0 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [-] `PhysicsMaterial::Destroy` · 0063a2d0 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [ ] `PhysicsMaterial::UpdateProperty` · 0063a2e0 · legions 005ef0b0 — needs port: PhysicsMaterial DAT record
- [ ] `PhysicsMaterial::UpdateCollection` · 0063a2f0 · legions 005ef0c0 — needs port: PhysicsMaterial DAT record
- [ ] `PhysicsMaterial::GetProperty` · 0063a300 · legions 005ef0d0 — needs port: PhysicsMaterial DAT record
- [-] `PhysicsMaterial::Allocate` · 0063a310 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [ ] `PhysicsMaterial::StreamPack` · 0063a320 · legions 005ef0f0 — needs port: PhysicsMaterial DAT record
- [-] `PhysicsMaterial::FromFileNode` · 0063a3c0 — text source format (admin tools)
- [-] `PhysicsMaterial::ToFileNode` · 0063a4d0 — text source format (admin tools)

## `portal/shared/src/physics/objectinfo.cpp.c` (5/5)

### OBJECTINFO
- [x] `OBJECTINFO::is_valid_walkable` · 004db340 · legions 004cdda0
- [x] `OBJECTINFO::kill_velocity` · 004db350 · legions 004cddb0
- [x] `OBJECTINFO::get_walkable_z` · 004db380 · legions 004cdde0
- [x] `OBJECTINFO::init` · 004db390 · legions 004cddf0
- [x] `OBJECTINFO::validate_walkable` · 004db410 · legions 004cde70

## `portal/shared/src/physics/physicsmodule.cpp.c` (2/4)

### PhysicsModule
- [-] `PhysicsModule::PhysicsModule` · 00737cc0 · legions 006b3880 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `PhysicsModule::~PhysicsModule` · 00737ce0 · legions 006b38a0 — C++ constructor/destructor; Rust constructor and Drop replace it
- [x] `PhysicsModule::UpdatePhysics` · 00737cf0 · legions 006b38b0
- [x] `PhysicsModule::UpdateAnimation` · 00737dd0 · legions 006b3990 — ac2-physics anim_rep / motion_interp_behaviors (slice 3)

## `portal/shared/src/physics/spherepath.cpp.c` (10/13)

### Position
- [ ] `Position::globaltolocalvec` · 004da840 · legions 004cd2a0 — needs port: Position::globaltolocalvec

### SPHEREPATH
- [x] `SPHEREPATH::init` · 004da8a0 · legions 004cd300
- [x] `SPHEREPATH::init_sphere` · 004da930 · legions 004cd390
- [x] `SPHEREPATH::cache_global_curr_center` · 004da9e0 · legions 004cd440
- [x] `SPHEREPATH::cache_global_sphere` · 004daaa0 · legions 004cd500
- [x] `SPHEREPATH::adjust_check_pos` · 004dac40 · legions 004cd6a0
- [x] `SPHEREPATH::step_up_slide` · 004dacc0 · legions 004cd720 — ac2-physics transition/localspace.rs, sphere_path.rs
- [x] `SPHEREPATH::check_walkables` · 004dad00 · legions 004cd760 — ac2-physics transition/localspace.rs, sphere_path.rs
- [x] `SPHEREPATH::precipice_slide` · 004dad40 · legions 004cd7a0 — ac2-physics transition/localspace.rs, sphere_path.rs
- [-] `SPHEREPATH::SPHEREPATH` · 004daed0 · legions 004cd930 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `SPHEREPATH::~SPHEREPATH` · 004db030 · legions 004cda90 — C++ constructor/destructor; Rust constructor and Drop replace it
- [x] `SPHEREPATH::cache_localspace_sphere` · 004db0b0 · legions 004cdb10 — ac2-physics transition/localspace.rs, sphere_path.rs
- [x] `SPHEREPATH::init_path` · 004db2a0 · legions 004cdd00

## `portal/shared/src/physics/transition.cpp.c` (33/35)

### AFrame
- [x] `AFrame::operator==` · 004db790 · legions 004ce1f0

### Frame
- [x] `Frame::set_rotate` · 004db850 · legions 004ce2b0

### Plane
- [x] `Plane::snap_to_plane` · 004db8f0 · legions 004ce350

### COLLISIONINFO
- [x] `COLLISIONINFO::set_sliding_normal` · 004db950 · legions 004ce3b0

### CTransition
- [-] `CTransition::cleanupTransition` · 004db9d0 · legions 004ce430 — static CTransition pool; one Rust value per transition
- [x] `CTransition::init_object` · 004db9e0 · legions 004ce440
- [x] `CTransition::init_sphere` · 004db9f0 · legions 004ce450
- [x] `CTransition::init_path` · 004dba00 · legions 004ce460
- [x] `CTransition::calc_num_steps` · 004dba10 · legions 004ce470

### COLLISIONINFO
- [x] `COLLISIONINFO::init` · 004dbc00 · legions 004ce660

### CTransition
- [x] `CTransition::init` · 004dbc30 · legions 004ce690
- [x] `CTransition::adjust_offset` · 004dbcb0 · legions 004ce710
- [x] `CTransition::build_cell_array` · 004dbff0 · legions 004cea50 — ac2-physics find_cells.rs, terrain.rs: cell search through land cells, touching cells and portals
- [x] `CTransition::cliff_slide` · 004dc030 · legions 004cea90

### SPHEREPATH
- [x] `SPHEREPATH::save_check_pos` · 004dc230 · legions 004cec90
- [x] `SPHEREPATH::set_check_pos` · 004dc260 · legions 004cecc0
- [x] `SPHEREPATH::set_walkable_check_pos` · 004dc2a0 · legions 004ced00 — ac2-physics transition/localspace.rs, sphere_path.rs
- [x] `SPHEREPATH::get_walkable_pos` · 004dc2d0 · legions 004ced30 — ac2-physics transition/localspace.rs, sphere_path.rs

### CTransition
- [x] `CTransition::validate_transition` · 004dc320 · legions 004ced80
- [x] `CTransition::validate_placement_transition` · 004dc690 · legions 004cf0f0 — ac2-physics transition/placement.rs (asm float order; Legions identical)
- [x] `CTransition::check_other_cells` · 004dc720 · legions 004cf180 — ac2-physics find_cells.rs, terrain.rs: cell search through land cells, touching cells and portals
- [x] `CTransition::check_walkable` · 004dc910 · legions 004cf370 — ac2-physics transition: step_up, check_walkable, neg-poly and precipice branches (Legions identical)

### SPHEREPATH
- [x] `SPHEREPATH::restore_check_pos` · 004dca10 · legions 004cf470

### CTransition
- [x] `CTransition::CTransition` · 004dca60 · legions 004cf4c0
- [-] `CTransition::makeTransition` · 004dcb20 · legions 004cf580 — static CTransition pool; one Rust value per transition
- [x] `CTransition::placement_insert` · 004dcba0 · legions 004cf600 — ac2-physics transition/placement.rs (asm float order; Legions identical)
- [x] `CTransition::validate_placement` · 004dcc40 · legions 004cf6a0 — ac2-physics transition/placement.rs (asm float order; Legions identical)
- [x] `CTransition::step_down` · 004dccd0 · legions 004cf730
- [x] `CTransition::edge_slide` · 004dce00 · legions 004cf860 — ac2-physics transition: step_up, check_walkable, neg-poly and precipice branches (Legions identical)
- [x] `CTransition::step_up` · 004dd060 · legions 004cfac0 — ac2-physics transition: step_up, check_walkable, neg-poly and precipice branches (Legions identical)
- [x] `CTransition::transitional_insert` · 004dd150 · legions 004cfbb0 — ac2-physics transition: step_up, check_walkable, neg-poly and precipice branches (Legions identical)
- [x] `CTransition::find_placement_pos` · 004dd4d0 · legions 004cff30 — ac2-physics transition/placement.rs (asm float order; Legions identical)
- [x] `CTransition::find_transitional_position` · 004dd900 · legions 004d0360 — ac2-physics transition/placement.rs (asm float order; Legions identical)
- [x] `CTransition::find_placement_position` · 004ddc20 · legions 004d0680 — ac2-physics transition/placement.rs (asm float order; Legions identical)
- [x] `CTransition::find_valid_position` · 004ddda0 · legions 004d0800 — ac2-physics transition/placement.rs (asm float order; Legions identical)

## `portal/shared/src/physobj/childlist.cpp.c` (0/9)

### CHILDINFO
- [-] `CHILDINFO::CHILDINFO` · 004ed850 · legions 004e01c0 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `CHILDINFO::CHILDINFO` · 004ed8a0 · legions 004e0210 — C++ constructor/destructor; Rust constructor and Drop replace it

### CHILDLIST
- [-] `CHILDLIST::CHILDLIST` · 004ed900 · legions 004e0270 — intrusive child-list container; Rust Vec of children replaces it
- [-] `CHILDLIST::~CHILDLIST` · 004ed920 · legions 004e0290 — intrusive child-list container; Rust Vec of children replaces it

### CHILDINFO
- [-] `CHILDINFO::operator=` · 004ed960 · legions 004e02d0 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it

### PSmartArray<CHILDINFO>
- [-] `PSmartArray<CHILDINFO>::StreamPack` · 004edad0 · legions 004e0440 — DBO/archive serialisation of runtime state; the client does not read it from the DAT

### CHILDLIST
- [-] `CHILDLIST::RemoveChild` · 004edb70 · legions 004e04e0 — intrusive child-list container; Rust Vec of children replaces it

### SmartArray<CHILDINFO>
- [-] `SmartArray<CHILDINFO>::AddToEnd` · 004edc00 · legions 004e0570 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### CHILDLIST
- [-] `CHILDLIST::AddChild` · 004edc80 · legions 004e05f0 — intrusive child-list container; Rust Vec of children replaces it

## `portal/shared/src/physobj/collision/collideproperty.cpp.c` (1/16)

### CollideProperty
- [-] `CollideProperty::GetPropertyName` · 0069fe80 · legions 0063ccf0 — admin/debug property name text

### CBookKeep
- [-] `CBookKeep::IsDummyObject` · 0069fe90 · legions 0063cd00 — admin/debug or text-file tooling

### CollideProperty
- [ ] `CollideProperty::IsEnd` · 0069fea0 · legions 0063cd10 — needs port: collision reports and properties
- [ ] `CollideProperty::IsChanged` · 0069feb0 · legions 0063cd20 — needs port: collision reports and properties
- [ ] `CollideProperty::IsSame` · 0069fec0 · legions 0063cd30 — needs port: collision reports and properties

### CEventQueue
- [x] `CEventQueue::StartEnteringWorld` · 0069fed0 · legions 0063cd40

### CollideProperty
- [ ] `CollideProperty::SetAsChange` · 0069fef0 · legions 0063cd60 — needs port: collision reports and properties
- [ ] `CollideProperty::SetAsSame` · 0069ff00 · legions 0063cd70 — needs port: collision reports and properties
- [ ] `CollideProperty::IsReportedToWeenie` · 0069ff10 · legions 0063cd80 — needs port: collision reports and properties

### BaseProperty
- [-] `BaseProperty::Set` · 0069ff20 — plain field accessor on a runtime record; Rust struct field replaces it

### CollideProperty
- [-] `CollideProperty::CollideProperty` · 0069ff60 · legions 0063cd90 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `CollideProperty::~CollideProperty` · 0069ff90 · legions 0063cdc0 — C++ constructor/destructor; Rust constructor and Drop replace it
- [ ] `CollideProperty::SetProperty` · 0069ffb0 · legions 0063cde0 — needs port: collision reports and properties
- [ ] `CollideProperty::UpdateVal` · 0069ffe0 · legions 0063ce10 — needs port: collision reports and properties
- [-] `CollideProperty::GetPropertyOutputStringW` · 006a0040 · legions 0063ce70 — admin/debug or text-file tooling
- [-] `CollideProperty::StreamPack` · 006a0100 · legions 0063cf30 — DBO/archive serialisation of runtime state; the client does not read it from the DAT

## `portal/shared/src/physobj/collision/collidepropertylist.cpp.c` (0/7)

### CollidePropertyList
- [-] `CollidePropertyList::StreamPack` · 006a0980 · legions 0063d7b0 — DBO/archive serialisation of runtime state; the client does not read it from the DAT
- [ ] `CollidePropertyList::GetProperty` · 006a0a20 · legions 0063d850 — needs port: collision reports and properties
- [-] `CollidePropertyList::CollidePropertyList` · 006a0a60 · legions 0063d890 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `CollidePropertyList::~CollidePropertyList` · 006a0a90 · legions 0063d8c0 — C++ constructor/destructor; Rust constructor and Drop replace it
- [ ] `CollidePropertyList::Add` · 006a0ad0 · legions 0063d900 — needs port: collision reports and properties
- [ ] `CollidePropertyList::BeginPropertyCollision` · 006a0ae0 · legions 0063d910 — needs port: collision reports and properties
- [ ] `CollidePropertyList::EndPropertyCollision` · 006a0b80 · legions 0063d9b0 — needs port: collision reports and properties

## `portal/shared/src/physobj/collision/collisionmanager.cpp.c` (3/61)

### CollisionParameters
- [ ] `CollisionParameters::SetWeeniePlacementType` · 004f6330 · legions 004e8b20 — needs port: ethereal/collision parameter state
- [ ] `CollisionParameters::SetWeenieMovementType` · 004f6350 · legions 004e8b40 — needs port: ethereal/collision parameter state

### ObjCollisionType
- [ ] `ObjCollisionType::UpdateIgnoreCollide` · 004f6370 · legions 004e8b60 — needs port: collision reports and properties

### CCollisionManager
- [x] `CCollisionManager::SetMinReportedImpactZ` · 004f6390 · legions 004e8b80 — ac2-physics physics_obj.rs init_object_end and collision_impact.rs (the XY minimum from the hover height and max forward speed, the Z from the jump impulse and scale, both plus 0.1); the body keeps them
- [x] `CCollisionManager::SetMinReportedImpactXY` · 004f63a0 · legions 004e8b90 — ac2-physics physics_obj.rs init_object_end and collision_impact.rs (the XY minimum from the hover height and max forward speed, the Z from the jump impulse and scale, both plus 0.1); the body keeps them
- [ ] `CCollisionManager::GetCollisionState` · 004f63b0 · legions 004e8ba0 — needs port: collision reports and properties
- [ ] `CCollisionManager::HasConflict` · 004f63f0 · legions 004e8be0 — needs port: collision reports and properties
- [ ] `CCollisionManager::AddToTempEtherealList` · 004f64d0 · legions 004e8cc0 — needs port: collision reports and properties
- [ ] `CCollisionManager::UpdateCollidePhysicsMask` · 004f64f0 · legions 004e8ce0 — needs port: collision reports and properties
- [~] `CCollisionManager::SetCollisionNormal` · 004f6520 · legions 004e8d10 — gate modelled (ac2-physics collision_impact.rs); collision manager sounds and reports not ported
- [ ] `CCollisionManager::SetNoCollisionNormal` · 004f65a0 · legions 004e8d90 — needs port: collision reports and properties
- [ ] `CCollisionManager::SetContactPlane` · 004f65b0 · legions 004e8da0 — needs port: collision reports and properties
- [ ] `CCollisionManager::SetInWater` · 004f65f0 · legions 004e8de0 — needs port: collision reports and properties
- [ ] `CCollisionManager::SetOutOfWater` · 004f6610 · legions 004e8e00 — needs port: collision reports and properties
- [ ] `CCollisionManager::SetOnLandscape` · 004f6620 · legions 004e8e10 — needs port: collision reports and properties
- [ ] `CCollisionManager::SetOffLandscape` · 004f6640 · legions 004e8e30 — needs port: collision reports and properties
- [ ] `CCollisionManager::SetTerrainType` · 004f6650 · legions 004e8e40 — needs port: collision reports and properties

### Vector
- [x] `Vector::StreamPack` · 004f6690 · legions 004e8e80 — ac2-net protocol::native pack+unpack (Legions layouts; WState graphs round-trip byte-exact); ac2-script still decodes its own copy until it switches

### PusherValues
- [-] `PusherValues::PusherValues` · 004f66c0 · legions 004e8eb0 — C++ constructor/destructor; Rust constructor and Drop replace it

### CCollisionManager
- [ ] `CCollisionManager::RemoveEndElements` · 004f67f0 · legions 004e8fe0 — needs port: collision reports and properties
- [ ] `CCollisionManager::SetProperties` · 004f6860 · legions 004e9050 — needs port: collision reports and properties
- [ ] `CCollisionManager::UpdateWaterTerrain` · 004f6940 · legions 004e9130 — needs port: collision reports and properties
- [ ] `CCollisionManager::UpdateLandTerrain` · 004f69e0 · legions 004e91d0 — needs port: collision reports and properties

### PusherValues
- [-] `PusherValues::PusherValues` · 004f6b20 · legions 004e9310 — C++ constructor/destructor; Rust constructor and Drop replace it

### IntrusiveHashIterator<unsigned___int64,HashTableData<unsigned___int64,unsigned_long>_*>
- [-] `IntrusiveHashIterator<unsigned___int64,HashTableData<unsigned___int64,unsigned_long>_*>::operator++` · 004f6c00 · legions 00455480 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### IntrusiveHashIterator<unsigned_long,HashTableData<unsigned_long,unsigned_long>_*>
- [-] `IntrusiveHashIterator<unsigned_long,HashTableData<unsigned_long,unsigned_long>_*>::operator++` · 004f6c80 · legions 004e93f0 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### (free functions)
- [-] `STREAMPACK_STREAMPACKOBJ` · 004f6d00 · legions 004e9470 — STL/MSVC or archive template helper; Rust collections replace it

### ObjCollisionType
- [-] `ObjCollisionType::ObjCollisionType` · 004f6d30 · legions 004e94a0 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `ObjCollisionType::StreamPack` · 004f6db0 · legions 004e9540 — DBO/archive serialisation of runtime state; the client does not read it from the DAT

### CCollisionManager
- [-] `CCollisionManager::StreamPack` · 004f6e50 · legions 004e9680 — DBO/archive serialisation of runtime state; the client does not read it from the DAT
- [ ] `CCollisionManager::IsEmpty` · 004f7170 · legions 004e99a0 — needs port: collision reports and properties

### PusherValues
- [-] `PusherValues::StreamPack` · 004f7190 · legions 004e99c0 — DBO/archive serialisation of runtime state; the client does not read it from the DAT

### SmartArray<CollideProperty>
- [-] `SmartArray<CollideProperty>::operator=` · 004f73c0 · legions 004e9bf0 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### CCollisionManager
- [ ] `CCollisionManager::UpdateCurrentPhysicsMask` · 004f7440 · legions 004e9c70 — needs port: collision reports and properties

### SmartArray<CollideProperty>
- [-] `SmartArray<CollideProperty>::AddToEnd` · 004f74b0 · legions 004e9ce0 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### CollidePropertyList
- [-] `CollidePropertyList::operator=` · 004f7510 · legions 004e9d40 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it

### CCollisionManager
- [ ] `CCollisionManager::ReportEnvironmentalProperties` · 004f7570 · legions 004e9da0 — needs port: collision reports and properties
- [ ] `CCollisionManager::EndEnvironmentalProperties` · 004f75d0 · legions 004e9e00 — needs port: collision reports and properties
- [ ] `CCollisionManager::ReportWaterProperties` · 004f7690 · legions 004e9ec0 — needs port: collision reports and properties
- [ ] `CCollisionManager::ReportSurfaceProperties` · 004f7700 · legions 004e9f30 — needs port: collision reports and properties
- [ ] `CCollisionManager::UpdatePropertyArray` · 004f7770 · legions 004e9fa0 — needs port: collision reports and properties
- [ ] `CCollisionManager::UpdatePusherValues` · 004f7930 · legions 004ea160 — needs port: collision reports and properties
- [ ] `CCollisionManager::UpdateConflictList` · 004f7b10 · legions 004ea340 — needs port: collision reports and properties
- [~] `CCollisionManager::SetCollisionState` · 004f7ca0 · legions 004ea510 — minimums and the sets reach the player body (ac2-client apply_collision_minimums/apply_collision_sets); the sets replace the body's rather than merging as CCollisionManager does; the physics and placement masks of the manager and its temp lists are not ported
- [ ] `CCollisionManager::AddEnvironmentalProperties` · 004f7db0 · legions 004ea620 — needs port: collision reports and properties
- [ ] `CCollisionManager::ChangePhysicsMaterial` · 004f7e30 · legions 004ea6a0 — needs port: collision reports and properties
- [ ] `CCollisionManager::RemovePropertiesAndErase` · 004f7ed0 · legions 004ea720 — needs port: collision reports and properties
- [ ] `CCollisionManager::RemoveObjectCollision` · 004f7f60 · legions 004ea7b0 — needs port: collision reports and properties
- [ ] `CCollisionManager::FindInPropertyList` · 004f8050 · legions 004ea8a0 — needs port: collision reports and properties
- [-] `CCollisionManager::~CCollisionManager` · 004f8420 · legions 004eac70 — C++ constructor/destructor; Rust constructor and Drop replace it
- [ ] `CCollisionManager::EndInWater` · 004f85b0 · legions 004eae00 — needs port: collision reports and properties
- [ ] `CCollisionManager::EndOnLandscape` · 004f8610 · legions 004eae60 — needs port: collision reports and properties
- [ ] `CCollisionManager::AddPusherProperties` · 004f8670 · legions 004eaec0 — needs port: collision reports and properties
- [ ] `CCollisionManager::AddToCollideList` · 004f9260 · legions 004ebab0 — needs port: collision reports and properties
- [~] `CCollisionManager::ReportCollision` · 004f9390 · legions 004ebbe0 — impact reports only: a landing impact (normal and speed, taken once by ac2-physics take_impact_report) is handed to the player's HandleCollision IFun (ac2-client collision_reported); object, environment, surface and water properties, begin/end reports and missiles are not ported
- [ ] `CCollisionManager::AddToPropertyList` · 004f9a00 · legions 004ec250 — needs port: collision reports and properties

### PHashTable<InstanceID,ObjCollisionType>
- [-] `PHashTable<InstanceID,ObjCollisionType>::StreamPack` · 004f9a30 — DBO/archive serialisation of runtime state; the client does not read it from the DAT

### PHashTable<unsigned_long,CollidePropertyList>
- [-] `PHashTable<unsigned_long,CollidePropertyList>::StreamPack` · 004f9d30 — DBO/archive serialisation of runtime state; the client does not read it from the DAT

### PHashTable<unsigned_long,PusherValues>
- [-] `PHashTable<unsigned_long,PusherValues>::StreamPack` · 004f9f10 — DBO/archive serialisation of runtime state; the client does not read it from the DAT

### CCollisionManager
- [-] `CCollisionManager::CCollisionManager` · 004fa110 · legions 004ec960 — C++ constructor/destructor; Rust constructor and Drop replace it
- [ ] `CCollisionManager::AddObjectCollision` · 004fa2f0 · legions 004ecb40 — needs port: collision reports and properties

## `portal/shared/src/physobj/collision/collisionparameters.cpp.c` (7/15)

### CollisionParameters
- [~] `CollisionParameters::SetBit` · 0069f9b0 · legions 0063c820 — no Rust caller: the sets are set through update_*/add_to_ethereal_list (collision_types.rs)
- [x] `CollisionParameters::UpdatePlacementEthereal` · 0069f9c0 · legions 0063c830 — ac2-physics collision_types.rs (CollisionParameters sets, IsBlocking); DAT existence via the master property table in glue
- [x] `CollisionParameters::UpdatePlacementSolid` · 0069f9f0 · legions 0063c860 — ac2-physics collision_types.rs (CollisionParameters sets, IsBlocking); DAT existence via the master property table in glue
- [x] `CollisionParameters::UpdateMovementEthereal` · 0069fa20 · legions 0063c890 — ac2-physics collision_types.rs (CollisionParameters sets, IsBlocking); DAT existence via the master property table in glue
- [x] `CollisionParameters::UpdateMovementSolid` · 0069fa50 · legions 0063c8c0 — ac2-physics collision_types.rs (CollisionParameters sets, IsBlocking); DAT existence via the master property table in glue
- [x] `CollisionParameters::UpdatePhysicsType` · 0069fa80 · legions 0063c8f0 — ac2-physics collision_types.rs (CollisionParameters sets, IsBlocking); DAT existence via the master property table in glue
- [x] `CollisionParameters::RemovePhysicsType` · 0069fab0 · legions 0063c920 — ac2-physics collision_types.rs (CollisionParameters sets, IsBlocking); DAT existence via the master property table in glue
- [x] `CollisionParameters::AddToEtherealList` · 0069fae0 · legions 0063c950 — ac2-physics collision_types.rs (CollisionParameters sets, IsBlocking); DAT existence via the master property table in glue
- [ ] `CollisionParameters::FindInTempEtherealList` · 0069fb90 · legions 0063ca00 — needs port: ethereal/collision parameter state
- [-] `CollisionParameters::CollisionParameters` · 0069fbe0 · legions 0063ca50 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `CollisionParameters::~CollisionParameters` · 0069fc70 · legions 0063cae0 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `CollisionParameters::StreamPack` · 0069fcd0 · legions 0063cb40 — DBO/archive serialisation of runtime state; the client does not read it from the DAT
- [ ] `CollisionParameters::RemoveFromTempEtherealList` · 0069fdc0 · legions 0063cc30 — needs port: ethereal/collision parameter state
- [ ] `CollisionParameters::InTempEtherealList` · 0069fe20 · legions 0063cc90 — needs port: ethereal/collision parameter state
- [ ] `CollisionParameters::AddToTempEtherealList` · 0069fe40 · legions 0063ccb0 — needs port: ethereal/collision parameter state

## `portal/shared/src/physobj/collision/collisionprofile.cpp.c` (0/20)

### CollisionProfile
- [ ] `CollisionProfile::HasCollided` · 006a0170 · legions 0063cfa0 — needs port: collision reports and properties
- [ ] `CollisionProfile::IsEndCollision` · 006a01b0 · legions 0063cfe0 — needs port: collision reports and properties
- [ ] `CollisionProfile::HasContact` · 006a01c0 · legions 0063cff0 — needs port: collision reports and properties
- [ ] `CollisionProfile::IsEndContact` · 006a0200 · legions 0063d030 — needs port: collision reports and properties
- [ ] `CollisionProfile::GetObjectA` · 006a0210 · legions 0063d040 — needs port: collision reports and properties
- [ ] `CollisionProfile::GetObjectA` · 006a0230 · legions 0063d060 — needs port: collision reports and properties
- [ ] `CollisionProfile::GetObjectProperty` · 006a0270 · legions 0063d0a0 — needs port: collision reports and properties
- [ ] `CollisionProfile::GetEnvProperty` · 006a02b0 · legions 0063d0e0 — needs port: collision reports and properties

### CWRepository
- [ ] `CWRepository::GetQualities` · 006a02f0 · legions 0063d120 — needs port: CWRepository::GetQualities

### CollisionProfile
- [ ] `CollisionProfile::GetSurfaceProperty` · 006a0300 · legions 0063d130 — needs port: collision reports and properties
- [ ] `CollisionProfile::GetWaterProperty` · 006a0340 · legions 0063d170 — needs port: collision reports and properties
- [ ] `CollisionProfile::FindObject` · 006a0380 · legions 0063d1b0 — needs port: collision reports and properties
- [ ] `CollisionProfile::NonEmpty` · 006a03f0 · legions 0063d220 — needs port: collision reports and properties
- [-] `CollisionProfile::CollisionProfile` · 006a0430 · legions 0063d260 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `CollisionProfile::~CollisionProfile` · 006a0490 · legions 0063d2c0 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `CollisionProfile::StreamPack` · 006a0560 · legions 0063d390 — DBO/archive serialisation of runtime state; the client does not read it from the DAT

### SmartArray<ObjCollisionProfile>
- [-] `SmartArray<ObjCollisionProfile>::AddToEnd` · 006a0650 · legions 0063d480 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### CollisionProfile
- [ ] `CollisionProfile::AddAsBeginCollision` · 006a06b0 · legions 0063d4e0 — needs port: collision reports and properties
- [ ] `CollisionProfile::AddAsEndCollision` · 006a06d0 · legions 0063d500 — needs port: collision reports and properties
- [-] `CollisionProfile::operator=` · 006a06f0 · legions 0063d520 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it

## `portal/shared/src/physobj/collision/objcollisiontype.cpp.c` (0/4)

### ObjCollisionType
- [ ] `ObjCollisionType::GetID` · 006a0c20 · legions 0063da50 — needs port: collision reports and properties
- [ ] `ObjCollisionType::SetAsEndCollide` · 006a0c40 · legions 0063da70 — needs port: collision reports and properties
- [ ] `ObjCollisionType::SetAsBeginCollide` · 006a0c50 · legions 0063da80 — needs port: collision reports and properties
- [-] `ObjCollisionType::ObjCollisionType` · 006a0c60 · legions 0063da90 — C++ constructor/destructor; Rust constructor and Drop replace it

## `portal/shared/src/physobj/detect/detectionblocktype.cpp.c` (0/7)

### DetectionBlockType
- [-] `DetectionBlockType::StreamPack` · 006a2d80 · legions 0063fc60 — DBO/archive serialisation of runtime state; the client does not read it from the DAT

### DetectionObjRay
- [-] `DetectionObjRay::DetectionObjRay` · 006a2e70 · legions 0063fd50 — C++ constructor/destructor; Rust constructor and Drop replace it

### DetectionBlockType
- [-] `DetectionBlockType::~DetectionBlockType` · 006a2fc0 · legions 0063fea0 — C++ constructor/destructor; Rust constructor and Drop replace it
- [ ] `DetectionBlockType::AddObject` · 006a3090 · legions 0063ff70 — needs port: object detection
- [ ] `DetectionBlockType::AddRayInfo` · 006a3100 · legions 0063ffe0 — needs port: object detection

### PHashTable<InstanceID,DetectionObjRay>
- [-] `PHashTable<InstanceID,DetectionObjRay>::StreamPack` · 006a3190 — DBO/archive serialisation of runtime state; the client does not read it from the DAT

### DetectionBlockType
- [-] `DetectionBlockType::DetectionBlockType` · 006a33d0 · legions 00640330 — C++ constructor/destructor; Rust constructor and Drop replace it

## `portal/shared/src/physobj/detect/detectioncylsphere.cpp.c` (0/5)

### PackObj
- [-] `PackObj::Pack` · 004fd590 — DBO/archive serialisation of runtime state; the client does not read it from the DAT
- [-] `PackObj::UnPack` · 004fd5c0 · legions 004efdb0 — DBO/archive serialisation of runtime state

### DetectionCylsphere
- [-] `DetectionCylsphere::~DetectionCylsphere` · 004fd5f0 · legions 004efde0 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `DetectionCylsphere::StreamPack` · 004fd600 · legions 004efdf0 — DBO/archive serialisation of runtime state; the client does not read it from the DAT
- [-] `DetectionCylsphere::DetectionCylsphere` · 004fd690 · legions 004efe80 — C++ constructor/destructor; Rust constructor and Drop replace it

## `portal/shared/src/physobj/detect/detectionmanager.cpp.c` (1/49)

### PackObj
- [x] `PackObj::UNPACK_TYPE` · 004fa540 · legions 004ecd90 — PackObj::UNPACK_TYPE - PackObj type-15 check, ac2-script bytestream/mod.rs
- [-] `PackObj::Pack` · 004fa580 — DBO/archive serialisation of runtime state; the client does not read it from the DAT

### CDetectionManager
- [ ] `CDetectionManager::InqDetectors` · 004fa5b0 · legions 004ecdd0 — needs port: object detection
- [ ] `CDetectionManager::CheckDetection` · 004fa5e0 · legions 004ece00 — needs port: object detection

### DetectionCylsphere
- [-] `DetectionCylsphere::DetectionCylsphere` · 004fa6a0 · legions 004ecec0 — C++ constructor/destructor; Rust constructor and Drop replace it

### CDetectionManager
- [-] `CDetectionManager::StreamPack` · 004fa6f0 · legions 004ecef0 — DBO/archive serialisation of runtime state; the client does not read it from the DAT
- [ ] `CDetectionManager::FindCellList` · 004fa750 · legions 004ecf50 — needs port: object detection
- [ ] `CDetectionManager::CheckLOSToOrigin` · 004fa7b0 · legions 004ecfb0 — needs port: object detection

### IntrusiveHashTable<InstanceID,HashTableData<InstanceID,DetectionInfo>_*>
- [-] `IntrusiveHashTable<InstanceID,HashTableData<InstanceID,DetectionInfo>_*>::~IntrusiveHashTable<InstanceID,HashTableData<InstanceID,DetectionInfo>_*>` · 004fa990 · legions 004ed190 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### InstantDetectionState
- [-] `InstantDetectionState::InstantDetectionState` · 004faa80 · legions 004ed280 — C++ constructor/destructor; Rust constructor and Drop replace it

### CDetectionManager
- [ ] `CDetectionManager::CleanupGlobalDetection` · 004fac30 · legions 004ed430 — needs port: object detection
- [ ] `CDetectionManager::UpdateCellListInternal` · 004facd0 · legions 004ed4d0 — needs port: object detection
- [ ] `CDetectionManager::NumTargetedDetectionCylSpheres` · 004fae20 · legions 004ed620 — needs port: object detection
- [ ] `CDetectionManager::NumTargetedDetectionCylSpheres` · 004faed0 · legions 004ed6d0 — needs port: object detection
- [ ] `CDetectionManager::NumGlobalDetectionCylSpheres` · 004faf80 — needs port: object detection
- [ ] `CDetectionManager::HandleLeaveWorld` · 004fb030 · legions 004ed830 — needs port: object detection
- [ ] `CDetectionManager::GetTargetBasePos` · 004fb0a0 · legions 004ed8a0 — needs port: object detection
- [ ] `CDetectionManager::CheckOrigin` · 004fb220 · legions 004eda20 — needs port: object detection
- [ ] `CDetectionManager::DoLOSCheck` · 004fb360 · legions 004edb60 — needs port: object detection
- [ ] `CDetectionManager::IsActive` · 004fb8a0 · legions 004ee0a0 — needs port: object detection
- [ ] `CDetectionManager::HasDetectionCylsphere` · 004fb8c0 — needs port: object detection
- [ ] `CDetectionManager::DestroyDetectionCylsphere` · 004fb910 · legions 004ee110 — needs port: object detection
- [ ] `CDetectionManager::DeleteAllTargetedDetectionCylspheres` · 004fb970 — needs port: object detection
- [ ] `CDetectionManager::CleanupDetectionCylsphere` · 004fb9f0 · legions 004ee1f0 — needs port: object detection
- [ ] `CDetectionManager::ClearPendingDeletions` · 004fba80 · legions 004ee280 — needs port: object detection
- [ ] `CDetectionManager::ReportGlobalDetection` · 004fbad0 · legions 004ee2d0 — needs port: object detection
- [ ] `CDetectionManager::RemoveContext` · 004fbc80 · legions 004ee480 — needs port: object detection
- [ ] `CDetectionManager::CalcPositions` · 004fbcb0 · legions 004ee4b0 — needs port: object detection
- [ ] `CDetectionManager::DoDetect` · 004fbe20 · legions 004ee620 — needs port: object detection
- [-] `CDetectionManager::~CDetectionManager` · 004fc380 · legions 004eeb80 — C++ constructor/destructor; Rust constructor and Drop replace it
- [ ] `CDetectionManager::RemoveTargetedDetectionSpheres` · 004fc450 · legions 004eec50 — needs port: object detection
- [ ] `CDetectionManager::ReceiveDetectionUpdate` · 004fc4d0 · legions 004eecd0 — needs port: object detection
- [ ] `CDetectionManager::GetState` · 004fc5a0 · legions 004eeda0 — needs port: object detection
- [ ] `CDetectionManager::InitIntersection` · 004fc640 · legions 004eee40 — needs port: object detection
- [ ] `CDetectionManager::GetProfileList` · 004fc680 · legions 004eee80 — needs port: object detection
- [ ] `CDetectionManager::DoIntersection` · 004fc6a0 · legions 004eeea0 — needs port: object detection

### PHashTable<unsigned_long,DetectionCylsphere>
- [-] `PHashTable<unsigned_long,DetectionCylsphere>::StreamPack` · 004fc7f0 — DBO/archive serialisation of runtime state; the client does not read it from the DAT

### PHashTable<unsigned_long,InstantDetectionState>
- [-] `PHashTable<unsigned_long,InstantDetectionState>::StreamPack` · 004fca50 — DBO/archive serialisation of runtime state; the client does not read it from the DAT

### CDetectionManager
- [-] `CDetectionManager::CDetectionManager` · 004fcc80 · legions 004ef4a0 — C++ constructor/destructor; Rust constructor and Drop replace it
- [ ] `CDetectionManager::InitializeGlobalDetection` · 004fcd60 · legions 004ef580 — needs port: object detection
- [ ] `CDetectionManager::UpdateCellList` · 004fcdf0 · legions 004ef610 — needs port: object detection
- [ ] `CDetectionManager::CreateDetectionCylsphere` · 004fce40 · legions 004ef660 — needs port: object detection
- [ ] `CDetectionManager::CheckDetection` · 004fd010 · legions 004ef830 — needs port: object detection
- [ ] `CDetectionManager::CheckDetection` · 004fd090 · legions 004ef8b0 — needs port: object detection

### PSmartArray<DetectionBlockType>
- [-] `PSmartArray<DetectionBlockType>::StreamPack` · 004fd120 · legions 004ef940 — DBO/archive serialisation of runtime state; the client does not read it from the DAT

### DetectionBlockType
- [-] `DetectionBlockType::operator=` · 004fd2a0 · legions 004efac0 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it

### HashTable<InstanceID,DetectionObjRay>
- [-] `HashTable<InstanceID,DetectionObjRay>::operator=` · 004fd2d0 · legions 004efaf0 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### PSmartArray<Ray>
- [-] `PSmartArray<Ray>::StreamPack` · 004fd4b0 · legions 004efcd0 — DBO/archive serialisation of runtime state; the client does not read it from the DAT

### DetectionObjRay
- [-] `DetectionObjRay::operator=` · 004fd550 · legions 004efd70 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it

## `portal/shared/src/physobj/detect/detectionparameters.cpp.c` (0/23)

### DetectionParameters
- [-] `DetectionParameters::SetShapeBox` · 006a0ca0 · legions 0063dad0 — plain detection parameter setter/getter; Rust struct field replaces it
- [-] `DetectionParameters::SetShapeSphere` · 006a0cb0 · legions 0063dae0 — plain detection parameter setter/getter; Rust struct field replaces it
- [-] `DetectionParameters::SetShapeSphereCone` · 006a0cc0 · legions 0063daf0 — plain detection parameter setter/getter; Rust struct field replaces it
- [-] `DetectionParameters::SetShapeAttackCone` · 006a0cd0 · legions 0063db00 — plain detection parameter setter/getter; Rust struct field replaces it
- [-] `DetectionParameters::UseDetectingObjectForBasePosition` · 006a0e30 · legions 0063dc60 — plain detection parameter setter/getter; Rust struct field replaces it
- [-] `DetectionParameters::SetDetectOffset` · 006a0e60 · legions 0063dc90 — plain detection parameter setter/getter; Rust struct field replaces it
- [-] `DetectionParameters::SetLOSBlockingMasks` · 006a0ef0 · legions 0063dd20 — plain detection parameter setter/getter; Rust struct field replaces it
- [-] `DetectionParameters::GetOriginPoint` · 006a0f30 · legions 0063dd60 — plain detection parameter setter/getter; Rust struct field replaces it
- [-] `DetectionParameters::GetGlobalBox` · 006a0f40 · legions 0063dd70 — plain detection parameter setter/getter; Rust struct field replaces it
- [-] `DetectionParameters::SetShapeCylinder` · 006a0f50 · legions 0063dd80 — plain detection parameter setter/getter; Rust struct field replaces it
- [-] `DetectionParameters::SetBasePosition` · 006a0fa0 · legions 0063ddd0 — plain detection parameter setter/getter; Rust struct field replaces it
- [-] `DetectionParameters::GetOriginPos` · 006a0fc0 · legions 0063ddf0 — plain detection parameter setter/getter; Rust struct field replaces it
- [ ] `DetectionParameters::InTargetList` · 006a0fe0 · legions 0063de10 — needs port: detection parameters
- [ ] `DetectionParameters::InExcludeList` · 006a1030 · legions 0063de60 — needs port: detection parameters
- [ ] `DetectionParameters::SingleTargetedObject` · 006a1080 · legions 0063deb0 — needs port: detection parameters
- [-] `DetectionParameters::DetectionParameters` · 006a10d0 · legions 0063df00 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `DetectionParameters::~DetectionParameters` · 006a1240 · legions 0063e070 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `DetectionParameters::StreamPack` · 006a12c0 · legions 0063e0f0 — DBO/archive serialisation of runtime state; the client does not read it from the DAT
- [-] `DetectionParameters::SetDetectionObjectType` · 006a14f0 · legions 0063e320 — plain detection parameter setter/getter; Rust struct field replaces it
- [-] `DetectionParameters::SetOriginPoint` · 006a1510 · legions 0063e340 — plain detection parameter setter/getter; Rust struct field replaces it
- [-] `DetectionParameters::CopyInto` · 006a1530 · legions 0063e360 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [ ] `DetectionParameters::AddTarget` · 006a1770 · legions 0063e5a0 — needs port: detection parameters
- [ ] `DetectionParameters::AddExcludeObject` · 006a17b0 · legions 0063e5e0 — needs port: detection parameters

## `portal/shared/src/physobj/detect/detectionprofile.cpp.c` (0/10)

### DetectionProfile
- [-] `DetectionProfile::~DetectionProfile` · 006a17f0 · legions 0063e620 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `DetectionProfile::Init` · 006a1800 · legions 0063e630 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `DetectionProfile::GetIntApp` · 006a1820 · legions 0063e650 — plain profile app-value accessor; Rust struct field replaces it
- [-] `DetectionProfile::SetIntApp` · 006a1830 · legions 0063e660 — plain profile app-value accessor; Rust struct field replaces it
- [-] `DetectionProfile::GetFloatApp` · 006a1840 · legions 0063e670 — plain profile app-value accessor; Rust struct field replaces it
- [-] `DetectionProfile::SetFloatApp` · 006a1850 · legions 0063e680 — plain profile app-value accessor; Rust struct field replaces it

### (free functions)
- [-] `STREAMPACK_BYTES` · 006a1870 · legions 00486ac0 — STL/MSVC or archive template helper; Rust collections replace it

### DetectionProfile
- [-] `DetectionProfile::DetectionProfile` · 006a1900 · legions 0063e6a0 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `DetectionProfile::StreamPack` · 006a1980 · legions 0063e720 — DBO/archive serialisation of runtime state; the client does not read it from the DAT
- [-] `DetectionProfile::CopyInto` · 006a1a40 · legions 0063e7e0 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it

## `portal/shared/src/physobj/detect/detectionvolume.cpp.c` (0/19)

### DetectionVolume
- [-] `DetectionVolume::DetectionVolume` · 006a1ae0 · legions 0063e880 — C++ constructor/destructor; Rust constructor and Drop replace it
- [ ] `DetectionVolume::GetGlobalBox` · 006a1b30 · legions 0063e8d0 — needs port: object detection

### (free functions)
- [-] `STREAMPACK_STREAMPACKOBJ` · 006a1c00 · legions 0063e9c0 — STL/MSVC or archive template helper; Rust collections replace it
- [-] `STREAMPACK_STREAMPACKOBJ` · 006a1c20 · legions 0063e9e0 — STL/MSVC or archive template helper; Rust collections replace it
- [-] `STREAMPACK_PACKOBJ` · 006a1c40 · legions 0063ea00 — STL/MSVC or archive template helper; Rust collections replace it
- [-] `STREAMPACK_PACKOBJ` · 006a1cb0 · legions 0063ea70 — STL/MSVC or archive template helper; Rust collections replace it

### Cylsphere
- [-] `Cylsphere::Cylsphere` · 006a1d20 · legions 0063eae0 — C++ constructor/destructor; Rust constructor and Drop replace it

### DetectionVolume
- [ ] `DetectionVolume::IsWithin` · 006a1d50 · legions 0063eb10 — needs port: object detection
- [ ] `DetectionVolume::IsWithin` · 006a1f40 · legions 0063ed00 — needs port: object detection
- [ ] `DetectionVolume::IsWithin` · 006a2070 · legions 0063ee30 — needs port: object detection
- [-] `DetectionVolume::StreamPack` · 006a21b0 · legions 0063ef70 — DBO/archive serialisation of runtime state; the client does not read it from the DAT
- [-] `DetectionVolume::CopyInto` · 006a23e0 · legions 0063f190 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [-] `DetectionVolume::Destroy` · 006a2540 · legions 0063f2f0 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [ ] `DetectionVolume::CreateBBox` · 006a2650 · legions 0063f400 — needs port: object detection
- [ ] `DetectionVolume::CreateCylinder` · 006a26f0 · legions 0063f4a0 — needs port: object detection
- [ ] `DetectionVolume::CreateSphere` · 006a2750 · legions 0063f500 — needs port: object detection
- [ ] `DetectionVolume::CreateSphereCone` · 006a27b0 · legions 0063f560 — needs port: object detection
- [ ] `DetectionVolume::CreateSpherePlaneCone` · 006a2810 · legions 0063f5c0 — needs port: object detection
- [-] `DetectionVolume::~DetectionVolume` · 006a2880 · legions 0063f630 — C++ constructor/destructor; Rust constructor and Drop replace it

## `portal/shared/src/physobj/detect/instantdetectionstate.cpp.c` (0/14)

### InstantDetectionState
- [-] `InstantDetectionState::StreamPack` · 006a3410 · legions 00640370 — DBO/archive serialisation of runtime state; the client does not read it from the DAT
- [ ] `InstantDetectionState::GetBlock` · 006a34a0 · legions 00640400 — needs port: object detection
- [-] `InstantDetectionState::InstantDetectionState` · 006a34e0 · legions 00640440 — C++ constructor/destructor; Rust constructor and Drop replace it
- [ ] `InstantDetectionState::Init` · 006a3550 · legions 006404b0 — needs port: object detection
- [-] `InstantDetectionState::~InstantDetectionState` · 006a3590 · legions 006404f0 — C++ constructor/destructor; Rust constructor and Drop replace it
- [ ] `InstantDetectionState::RemoveObjectsInUnloadedBlock` · 006a3650 · legions 006405b0 — needs port: object detection
- [ ] `InstantDetectionState::PruneObjects` · 006a3760 · legions 006406c0 — needs port: object detection
- [ ] `InstantDetectionState::UpdateProfiles` · 006a38b0 · legions 00640810 — needs port: object detection

### SmartArray<DetectionBlockType>
- [-] `SmartArray<DetectionBlockType>::operator=` · 006a3a10 · legions 00640970 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it
- [-] `SmartArray<DetectionBlockType>::AddToEnd` · 006a3a90 · legions 006409f0 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### InstantDetectionState
- [-] `InstantDetectionState::operator=` · 006a3af0 · legions 00640a50 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [ ] `InstantDetectionState::CreateBlockListInfo` · 006a3b70 · legions 00640ad0 — needs port: object detection
- [ ] `InstantDetectionState::FindCreateBlockRay` · 006a3be0 · legions 00640b40 — needs port: object detection
- [ ] `InstantDetectionState::AddBlockRay` · 006a3cc0 · legions 00640c20 — needs port: object detection

## `portal/shared/src/physobj/detect/profilelisttype.cpp.c` (0/3)

### ProfileListType
- [-] `ProfileListType::~ProfileListType` · 006a2a70 · legions 0063f850 — C++ constructor/destructor; Rust constructor and Drop replace it

### PHashTable<InstanceID,DetectionProfile>
- [-] `PHashTable<InstanceID,DetectionProfile>::StreamPack` · 006a2af0 — DBO/archive serialisation of runtime state; the client does not read it from the DAT

### ProfileListType
- [-] `ProfileListType::ProfileListType` · 006a2d40 · legions 0063fc20 — C++ constructor/destructor; Rust constructor and Drop replace it

## `portal/shared/src/physobj/ethereal/etherealphysicstype.cpp.c` (1/6)

### EtherealPhysicsType
- [x] `EtherealPhysicsType::SetProperty` · 006a44d0 · legions 00641430 — ac2-physics collision_types.rs (CollisionParameters sets, IsBlocking); DAT existence via the master property table in glue
- [ ] `EtherealPhysicsType::UpdateEthereality` · 006a4510 · legions 00641470 — needs port: ethereal/collision parameter state
- [ ] `EtherealPhysicsType::SetSolidOnlyToEnvironment` · 006a4590 · legions 006414f0 — needs port: ethereal/collision parameter state
- [ ] `EtherealPhysicsType::SetCameraEthereal` · 006a45e0 · legions 00641540 — needs port: ethereal/collision parameter state
- [ ] `EtherealPhysicsType::SetEtherealToWaterLayer` · 006a4630 · legions 00641590 — needs port: ethereal/collision parameter state
- [-] `EtherealPhysicsType::EtherealPhysicsType` · 006a4690 · legions 006415f0 — C++ constructor/destructor; Rust constructor and Drop replace it

## `portal/shared/src/physobj/ethereal/etherealweenietype.cpp.c` (7/14)

### EtherealWeenieType
- [~] `EtherealWeenieType::SetToEthereal` · 006a46b0 · legions 00641610 — no Rust caller: the sets are set through update_*/add_to_ethereal_list (collision_types.rs)
- [x] `EtherealWeenieType::SetToSolid` · 006a46c0 · legions 00641620 — ac2-physics collision_types.rs (CollisionParameters sets, IsBlocking); DAT existence via the master property table in glue
- [ ] `EtherealWeenieType::NeedLOSCheck` · 006a46d0 · legions 00641630 — needs port: ethereal/collision parameter state
- [x] `EtherealWeenieType::FindInEtherealList` · 006a46f0 · legions 00641650 — ac2-physics collision_types.rs (CollisionParameters sets, IsBlocking); DAT existence via the master property table in glue
- [x] `EtherealWeenieType::FindInSolidList` · 006a4740 · legions 006416a0 — ac2-physics collision_types.rs (CollisionParameters sets, IsBlocking); DAT existence via the master property table in glue
- [-] `EtherealWeenieType::EtherealWeenieType` · 006a4790 · legions 006416f0 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `EtherealWeenieType::~EtherealWeenieType` · 006a47c0 · legions 00641720 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `EtherealWeenieType::StreamPack` · 006a4800 · legions 00641760 — DBO/archive serialisation of runtime state; the client does not read it from the DAT
- [x] `EtherealWeenieType::InEtherealList` · 006a4860 · legions 006417c0 — ac2-physics collision_types.rs (CollisionParameters sets, IsBlocking); DAT existence via the master property table in glue
- [x] `EtherealWeenieType::InSolidList` · 006a4880 · legions 006417e0 — ac2-physics collision_types.rs (CollisionParameters sets, IsBlocking); DAT existence via the master property table in glue
- [-] `EtherealWeenieType::CopyInto` · 006a48a0 · legions 00641800 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [x] `EtherealWeenieType::AddToEtherealList` · 006a49b0 · legions 00641910 — ac2-physics collision_types.rs (CollisionParameters sets, IsBlocking); DAT existence via the master property table in glue
- [x] `EtherealWeenieType::AddToSolidList` · 006a49f0 · legions 00641950 — ac2-physics collision_types.rs (CollisionParameters sets, IsBlocking); DAT existence via the master property table in glue
- [ ] `EtherealWeenieType::Update` · 006a4a30 · legions 00641990 — needs port: ethereal/collision parameter state

## `portal/shared/src/physobj/ethereal/losparameters.cpp.c` (5/10)

### UIPersistantData
- [x] `UIPersistantData::SetCurrentCGR` · 006a3cf0 · legions 00640c50 — ac2-ui pregame/persistent.rs (plain data)

### LOSParameters
- [-] `LOSParameters::SetBlockedByWater` · 006a3d00 · legions 00640c60 — plain field accessor on a runtime record; Rust struct field replaces it
- [x] `LOSParameters::LOSParameters` · 006a3d20 · legions 00640c80 — ac2-physics los (Terrain::los_check)
- [-] `LOSParameters::~LOSParameters` · 006a3e10 · legions 00640d70 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `LOSParameters::SetLOSParams` · 006a3e50 · legions 00640db0 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `LOSParameters::StreamPack` · 006a3e80 · legions 00640de0 — DBO/archive serialisation of runtime state; the client does not read it from the DAT
- [-] `LOSParameters::operator=` · 006a4040 · legions 00640fa0 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [x] `LOSParameters::SetRay` · 006a41d0 · legions 00641130 — ac2-physics los (Terrain::los_check)
- [x] `LOSParameters::ResetState` · 006a4210 · legions 00641170 — ac2-physics los (Terrain::los_check)
- [x] `LOSParameters::CalcFinalRay` · 006a4270 · legions 006411d0 — ac2-physics los (Terrain::los_check)

## `portal/shared/src/physobj/fx/dbfxtable.cpp.c` (2/12)

### DBFXTable
- [-] `DBFXTable::GetSubDataIDs` · 006a8dc0 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [-] `DBFXTable::ToFileNode` · 006a8e10 — text source format (admin tools)
- [-] `DBFXTable::InitSubTableCache` · 006a8fb0 · legions 006432a0 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [x] `DBFXTable::StreamPack` · 006a8ff0 · legions 006432e0 — DBFXTable::StreamPack - DBFXTable/FXNode/FXData decode and defaults, ac2-assets ac2_fx.rs, ac2-world ac2_particle_sources.rs
- [-] `DBFXTable::Destroy` · 006a9080 · legions 00643370 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [x] `DBFXTable::GetFXData` · 006a90c0 · legions 006433b0 — ac2-audio/ac2-client: 3D emitters and FX sounds
- [-] `DBFXTable::FromFileNode` · 006a9200 — text source format (admin tools)
- [-] `DBFXTable::~DBFXTable` · 006a9450 · legions 006434d0 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `DBFXTable::GetDBOType` · 006a94c0 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [-] `DBFXTable::DBFXTable` · 006a94f0 · legions 00643570 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `DBFXTable::Allocate` · 006a9550 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [-] `DBFXTable::CopyInto` · 006a9560 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it

## `portal/shared/src/physobj/fx/decalfx.cpp.c` (0/5)

### DecalFX
- [-] `DecalFX::SetDefaults` · 006a9890 · legions 00643910 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `DecalFX::DecalFX` · 006a9900 · legions 006439a0 — C++ constructor/destructor; Rust constructor and Drop replace it
- [~] `DecalFX::StreamPack` · 006a9930 · legions 006439d0 — unpack only, in ac2-net protocol::fx / protocol::visual (Legions FXNode::StreamPack 0x00642410 instruction-identical)
- [-] `DecalFX::FromFileNode` · 006a99e0 — text source format (admin tools)
- [-] `DecalFX::ToFileNode` · 006a9d20 — text source format (admin tools)

## `portal/shared/src/physobj/fx/fxdefs.cpp.c` (4/25)

### FXNode
- [-] `FXNode::GetSubDataIDs` · 006a4c10 · legions 00641b70 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it

### FXData
- [~] `FXData::StreamPack` · 006a4ce0 · legions 00641c40 — unpack only, in ac2-net protocol::fx / protocol::visual (Legions FXNode::StreamPack 0x00642410 instruction-identical)

### StreakFX
- [-] `StreakFX::operator=` · 006a4d30 · legions 00641c90 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it

### FXNode
- [x] `FXNode::GetSoundID` · 006a4db0 · legions 00641d10 — ac2-audio/ac2-client: 3D emitters and FX sounds
- [x] `FXNode::GetParticleID` · 006a4e00 · legions 00641d60 — FXNode::GetParticleID - DBFXTable/FXNode/FXData decode and defaults, ac2-assets ac2_fx.rs, ac2-world ac2_particle_sources.rs
- [~] `FXNode::get_pack_bitfield` · 006a4e50 · legions 00641db0 — unpack only, in ac2-net protocol::fx / protocol::visual (Legions FXNode::StreamPack 0x00642410 instruction-identical)

### DecalFX
- [-] `DecalFX::operator=` · 006a51a0 · legions 00642100 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it

### FXNode
- [-] `FXNode::ToFileNode` · 006a52f0 — text source format (admin tools)
- [-] `FXNode::ReadColor` · 006a5f00 — admin/debug or text-file tooling
- [x] `FXNode::SetDefaults` · 006a5fc0 · legions 00642250 — FXNode::SetDefaults - DBFXTable/FXNode/FXData decode and defaults, ac2-assets ac2_fx.rs, ac2-world ac2_particle_sources.rs
- [~] `FXNode::StreamPack` · 006a6180 — unpack only, in ac2-net protocol::fx / protocol::visual (Legions FXNode::StreamPack 0x00642410 instruction-identical)

### FXData
- [-] `FXData::ToFileNode` · 006a68d0 — text source format (admin tools)

### (free functions)
- [-] `FXMap_ToFileNode` · 006a6a30 — admin FXMap text tool, not used by the client

### FXData
- [-] `FXData::GetSubDataIDs` · 006a6c60 · legions 00642b60 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it

### (free functions)
- [-] `FXMap_GetSubDataIDs` · 006a6ce0 · legions 00642be0 — admin FXMap text tool, not used by the client

### FXData
- [x] `FXData::GetData` · 006a6d80 · legions 00642c80 — FXData::GetData - DBFXTable/FXNode/FXData decode and defaults, ac2-assets ac2_fx.rs, ac2-world ac2_particle_sources.rs

### FXNode
- [-] `FXNode::ParseChannel` · 006a6dd0 — admin/debug or text-file tooling
- [-] `FXNode::operator=` · 006a7050 · legions 00642cd0 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [-] `FXNode::FromFileNode` · 006a7310 — text source format (admin tools)
- [-] `FXNode::FXNode` · 006a8190 · legions 00642f90 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `FXNode::FXNode` · 006a8250 · legions 00643050 — C++ constructor/destructor; Rust constructor and Drop replace it

### HashTable<unsigned_long,FXNode>
- [-] `HashTable<unsigned_long,FXNode>::operator=` · 006a8310 · legions 00643110 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### FXData
- [-] `FXData::FromFileNode` · 006a8410 — text source format (admin tools)
- [-] `FXData::operator=` · 006a8ac0 · legions 00643210 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it

### (free functions)
- [-] `FXMap_FromFileNode` · 006a8b00 — admin FXMap text tool, not used by the client

## `portal/shared/src/physobj/fx/fxmanager.cpp.c` (9/44)

### DecalEffectDesc
- [-] `DecalEffectDesc::DecalEffectDesc` · 005051c0 · legions 004f3500 — C++ constructor/destructor; Rust constructor and Drop replace it

### FXManager
- [x] `FXManager::DoColorFX` · 00505210 · legions 004f3550 — ac2-client fx_objects.rs set_colour: the emitter's colour ramps over the node's ramp time (PHYSICSOBJREP property ramp); drawn through FxLooks tints; restored by StopFX (node restore, 0.1 s) and by the colour lifetime
- [ ] `FXManager::DoSliderFX` · 005052d0 · legions 004f3610 — needs port: FX manager effect
- [ ] `FXManager::DoRepulseFX` · 00505310 · legions 004f3650 — needs port: FX manager effect
- [ ] `FXManager::DoStreakFX` · 00505340 · legions 004f3680 — needs port: FX manager effect
- [ ] `FXManager::DoCameraFX` · 00505380 · legions 004f36c0 — needs port: FX manager effect
- [ ] `FXManager::DestroyObject` · 005053b0 · legions 004f36f0 — needs port: FX manager effect

### ChannelPrefs
- [-] `ChannelPrefs::LoadChannelPrefs` · 005053f0 · legions 004f3730 — admin/debug or text-file tooling

### FXNode
- [ ] `FXNode::GetAppearanceKey` · 00505850 · legions 004f3b20 — needs port: FX appearance key

### PlayingSoundContext
- [x] `PlayingSoundContext::SetVolume` · 00505870 · legions 004f3b40 — ac2-audio/ac2-client: 3D emitters and FX sounds

### FXState
- [-] `FXState::FXState` · 005058b0 · legions 004f3b80 — C++ constructor/destructor; Rust constructor and Drop replace it

### FXManager
- [ ] `FXManager::DoDecalFX` · 00505960 · legions 004f3c30 — needs port: FX manager effect
- [ ] `FXManager::DoAppearanceFX` · 00505c30 · legions 004f3f00 — needs port: FX manager effect

### DBObjGrabber<FxScript>
- [x] `DBObjGrabber<FxScript>::DBObjGrabber<FxScript>` · 00505ca0 · legions 004f3f70 — ac2-anim fxscript.rs load_ac2_fx_script (the 0x57 DBO read)

### _STL::pair<unsigned_long_const_,FXState>
- [-] `_STL::pair<unsigned_long_const_,FXState>::~pair<unsigned_long_const_,FXState>` · 00505ce0 · legions 004f3fb0 — STL/MSVC or archive template helper; Rust collections replace it

### FXManager
- [~] `FXManager::FireParticleFX` · 00505dd0 · legions 004f3ff0 — fire-and-forget FX nodes: ac2-client keeps no handle; ac2-render scene/particles.rs starts the system unfollowed, placed once where its object first stood (PSManager::CreateAutonomousParticleSystem); water-plane riding and the holding and bone attachment of the autonomous frame not ported
- [x] `FXManager::SoundIsLooping` · 005060e0 · legions 004f4300 — ac2-audio/ac2-client: 3D emitters and FX sounds
- [x] `FXManager::ParticleIsLooping` · 00506190 · legions 004f43b0 — ac2-assets Ac2ParticleSystemDesc::is_looping, flagged per PSDesc on the loader thread (ac2_visual_particles.rs looping_flags)
- [ ] `FXManager::SoundIsDefaultChannelMusic` · 005061f0 · legions 004f4410 — needs port: FX manager effect

### SmartArray<PlayingSoundContext>
- [-] `SmartArray<PlayingSoundContext>::operator=` · 00506260 · legions 004f4480 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### FXManager
- [x] `FXManager::DoSoundFX` · 005062c0 · legions 004f44e0 — ac2-audio/ac2-client: 3D emitters and FX sounds
- [x] `FXManager::DoParticleFX` · 00506360 · legions 004f4580 — ac2-client client_audio.rs: a handle is kept only for a looping PSDesc (fx_particles ObjectParticleStart.looping), so StopFX destroys only those; the random PSDesc pick is unchanged
- [x] `FXManager::DoLightSourceFX` · 00506430 · legions 004f4650 — ac2-client client_audio.rs: light_starts only while Render::m_Prefs.LightSourceEffects is on (ac2-render prefs light_source_effects, applied by apply_light_preferences); particle systems still start with the light off
- [~] `FXManager::CalcDuration` · 005064f0 · legions 004f4710 — ac2-client FX sounds: sound branch only (no particles/appearance/channel prefs, day assumed); DirectSound 3D pan law and doppler approximated

### FXState
- [-] `FXState::operator=` · 00506520 · legions 004f4740 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [-] `FXState::FXState` · 005065e0 · legions 004f4800 — C++ constructor/destructor; Rust constructor and Drop replace it

### FXManager
- [~] `FXManager::DoTargetAndCloningFXs` · 00506680 · legions 004f48a0 — ac2-client fx_objects.rs start: target hide (m_hide_target, unhidden on stop) and destruction (m_destroy_target). Not ported: retarget clones, parent/unparent to the holding location, target FX (m_target_fx_id)
- [x] `FXManager::ChannelMissmatch` · 005069b0 · legions 004f4bd0 — FXManager::ChannelMissmatch - FXManager::ChannelMissmatch, ac2-assets ac2_fx.rs and ac2-client client_audio.rs

### ChannelPrefs
- [-] `ChannelPrefs::HandleApply` · 00506b00 — admin/debug or text-file tooling
- [-] `ChannelPrefs::HandleChange` · 00506be0 — admin/debug or text-file tooling

### FXManager
- [~] `FXManager::DoCreateObjectFX` · 00507010 · legions 004f5230 — ac2-client fx_objects.rs: the node's object is created at its emitter's position (CreateTemporaryObject at the rep's position, id 0x13FF...), handed to the FX (state object ids, target), faded and destroyed on StopFX/expiry (FadeAndDestroyObject 0.5/3.0). Not ported: bone and holding-location placement, water-plane placement, the behaviour start (its object is kept as the failed-behaviour fallback), ethereal physics flags
- [~] `FXManager::StopFX` · 00507650 · legions 004f5870 — ac2-client fx_objects.rs stop: startup FX kept (m_bStartupFX), objects faded, targets unhidden, colour restored; sound and particle stop as before. Not ported: streak stop, appearance removal, camera FX, target FX on the target
- [-] `FXManager::FXManager` · 00507d10 · legions 004f5e80 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `FXManager::Destroy` · 00507db0 · legions 004f5f20 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [~] `FXManager::UseTime` · 00507df0 · legions 004f5f60 — ac2-client fx_objects.rs expire: an FX whose end came tears down as StopFX does (objects, hides, colour); sound and particle ends keep their existing approximation

### _STL::map<unsigned_long,FXScalarAndTarget,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,FXScalarAndTarget>_>_>
- [-] `_STL::map<unsigned_long,FXScalarAndTarget,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,FXScalarAndTarget>_>_>::operator[]` · 00508360 · legions 004f6490 — STL/MSVC or archive template helper; Rust collections replace it

### (free functions)
- [-] `STREAMPACK_STL` · 005083f0 · legions 004f6520 — STL/MSVC or archive template helper; Rust collections replace it

### FXManager
- [-] `FXManager::~FXManager` · 00508660 · legions 004f6790 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `FXManager::StreamPack` · 00508750 · legions 004f6880 — DBO/archive serialisation of runtime state; the client does not read it from the DAT
- [ ] `FXManager::GetCurrentFX` · 005087c0 · legions 004f68f0 — needs port: FX manager effect

### _STL::map<unsigned_long,FXState,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,FXState>_>_>
- [-] `_STL::map<unsigned_long,FXState,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,FXState>_>_>::operator[]` · 00508860 · legions 004f6990 — STL/MSVC or archive template helper; Rust collections replace it

### FXManager
- [~] `FXManager::DoTargettedFX` · 005089d0 · legions 004f6b00 — ac2-client FX sounds: sound branch only (no particles/appearance/channel prefs, day assumed); DirectSound 3D pan law and doppler approximated
- [~] `FXManager::DoStartupFX` · 00509220 · legions 004f7270 — ac2-client client_audio.rs start_startup_lights: each object's startup FX place their lights (startup_light_starts, LightSourceEffects gated), play their node sounds on the object (DoSoundFX, no day/night filter), and create their nodes' objects (fx_objects start_startup, recorded as startup so StopFX leaves them). Not ported: streak (PHYSICSOBJREP::CreateStreakEffect), decal (DoDecalFX) and repulsor (CreateParticleRepulsor) nodes, which need render support; the light-fails-skips-vdesc branch; the shared rep's particle branch (ApplyStartupFX with no object)
- [~] `FXManager::DoFX` · 00509440 · legions 004f7490 — ac2-client FX sounds: sound branch only (no particles/appearance/channel prefs, day assumed); DirectSound 3D pan law and doppler approximated

## `portal/shared/src/physobj/fx/fxtable.cpp.c` (1/8)

### FXTable
- [ ] `FXTable::SetStaticTable` · 006a95d0 · legions 00643650 — needs port: FX table registration

### (free functions)
- [-] `STREAMPACK_DBOBJ` · 006a9610 · legions 00643690 — STL/MSVC or archive template helper; Rust collections replace it

### FXTable
- [-] `FXTable::StreamPack` · 006a9680 · legions 00643700 — DBO/archive serialisation of runtime state; the client does not read it from the DAT
- [ ] `FXTable::Clear` · 006a96d0 · legions 00643750 — needs port: FX table registration
- [x] `FXTable::GetFXData` · 006a96e0 · legions 00643760 — ac2-audio/ac2-client: 3D emitters and FX sounds
- [-] `FXTable::~FXTable` · 006a97a0 · legions 00643820 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `FXTable::FXTable` · 006a9830 · legions 006438b0 — C++ constructor/destructor; Rust constructor and Drop replace it
- [ ] `FXTable::AddEntry` · 006a9870 · legions 006438f0 — needs port: FX table registration

## `portal/shared/src/physobj/fx/lightsourcefx.cpp.c` (0/4)

### LightSourceFX
- [-] `LightSourceFX::LightSourceFX` · 006aaa30 · legions 00643c70 — C++ constructor/destructor; Rust constructor and Drop replace it
- [~] `LightSourceFX::StreamPack` · 006aaa90 · legions 00643cd0 — unpack only, in ac2-net protocol::property / fx (types from a PropertyTypes resolver; Legions BaseProperty::StreamPack 0x0064B4E0 instruction-identical)
- [-] `LightSourceFX::FromFileNode` · 006aab00 — text source format (admin tools)
- [-] `LightSourceFX::ToFileNode` · 006aad00 — text source format (admin tools)

## `portal/shared/src/physobj/fx/psdesc.cpp.c` (2/42)

### FileNodeName_Float
- [-] `FileNodeName_Float::Clone` · 004fe120 · legions 004f0910 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it

### WaveformVector3
- [-] `WaveformVector3::operator=` · 004fe150 · legions 004f0940 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it

### PSDesc
- [-] `PSDesc::GetSubDataIDs` · 004fe230 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [-] `PSDesc::GetSubObjects` · 004fe250 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [ ] `PSDesc::Refresh` · 004fe2c0 · legions 004f0a90 — needs port: particle descriptor derived data
- [-] `PSDesc::ReleaseSubObjects` · 004fe2e0 · legions 004f0ab0 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it

### (free functions)
- [-] `STREAMPACK_SIZEOF` · 004fe310 · legions 004f0ae0 — STL/MSVC or archive template helper; Rust collections replace it
- [-] `STREAMPACK_SIZEOF` · 004fe350 · legions 004f0b20 — STL/MSVC or archive template helper; Rust collections replace it
- [-] `STREAMPACK_SIZEOF` · 004fe3a0 · legions 004f0b70 — STL/MSVC or archive template helper; Rust collections replace it
- [-] `STREAMPACK_SIZEOF` · 004fe3f0 · legions 004f0bc0 — STL/MSVC or archive template helper; Rust collections replace it

### PEKeyframeDesc
- [-] `PEKeyframeDesc::PEKeyframeDesc` · 004fe460 · legions 004f0c30 — C++ constructor/destructor; Rust constructor and Drop replace it

### PFileNode
- [-] `PFileNode::SetNameAsFloat` · 004fe520 · legions 004f0cf0 — admin/debug or text-file tooling

### PSDesc
- [x] `PSDesc::ApproximateLifetime` · 004fe570 · legions 004f0d40 — ac2-assets ac2_particle_desc.rs approximate_lifetime (checked against the admin disassembly: emitter stride 0x444, count = emission limit / blast count, max(count*birth, time limit), start and fast-forward); -1 when physics duration > 0 or an emitter has no limit

### PEKeyframeDesc
- [-] `PEKeyframeDesc::~PEKeyframeDesc` · 004fe650 — C++ constructor/destructor; Rust constructor and Drop replace it

### PFileNode
- [-] `PFileNode::SetName` · 004fe750 · legions 004f0f20 — admin/debug or text-file tooling
- [-] `PFileNode::GetNameAsFloat` · 004fe7a0 · legions 004f0f70 — admin/debug or text-file tooling

### SmartArray<PEKeyframeDesc>
- [-] `SmartArray<PEKeyframeDesc>::~SmartArray<PEKeyframeDesc>` · 004fe860 · legions 004f1030 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### PFileNode
- [-] `PFileNode::SetName` · 004fe8c0 · legions 004f1090 — admin/debug or text-file tooling

### PEmitterDesc
- [-] `PEmitterDesc::PEmitterDesc` · 004fe900 · legions 004f10d0 — C++ constructor/destructor; Rust constructor and Drop replace it

### PSDesc
- [-] `PSDesc::LoadKeyframeFromFileNode` · 004ff460 — text source loader, admin build only
- [-] `PSDesc::SaveKeyframeToFileNode` · 004ffbe0 — admin/debug or text-file tooling

### PEKeyframeDesc
- [-] `PEKeyframeDesc::operator=` · 005008c0 · legions 004f1cd0 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it

### PSDesc
- [-] `PSDesc::Begin` · 00500a00 · legions 004f1e10 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [-] `PSDesc::Destroy` · 00500a60 · legions 004f1e70 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [-] `PSDesc::SaveEmitterToFileNode` · 00500aa0 — admin/debug or text-file tooling

### SmartArray<PEKeyframeDesc>
- [-] `SmartArray<PEKeyframeDesc>::SetNElements` · 00501d30 · legions 004f1eb0 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it
- [-] `SmartArray<PEKeyframeDesc>::operator=` · 00501d60 · legions 004f1ee0 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### PSDesc
- [-] `PSDesc::PSDesc` · 00501dd0 · legions 004f1f50 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `PSDesc::GetDBOType` · 00501e20 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [-] `PSDesc::~PSDesc` · 00501e30 · legions 004f1fb0 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `PSDesc::ToFileNode` · 00501ea0 — text source format (admin tools)

### SmartArray<PEKeyframeDesc>
- [-] `SmartArray<PEKeyframeDesc>::AddToEnd` · 00502590 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### PEmitterDesc
- [-] `PEmitterDesc::operator=` · 005025e0 · legions 004f2020 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it

### PSDesc
- [-] `PSDesc::Allocate` · 00502c10 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it

### PEmitterDesc
- [-] `PEmitterDesc::CopyInto` · 00502c20 · legions 004f2660 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it

### PSDesc
- [-] `PSDesc::LoadEmitterFromFileNode` · 00503280 — text source loader, admin build only

### SmartArray<PEmitterDesc>
- [-] `SmartArray<PEmitterDesc>::SetNElements` · 005041f0 · legions 004f2cc0 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### PSDesc
- [x] `PSDesc::StreamPack` · 00504220 — PSDesc::StreamPack - PSDesc::StreamPack decoder, ac2-assets ac2_particle_desc.rs

### SmartArray<PEmitterDesc>
- [-] `SmartArray<PEmitterDesc>::AddToEnd` · 005048f0 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### PSDesc
- [-] `PSDesc::CopyInto` · 00504940 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [-] `PSDesc::LoadFromFileNode` · 00504a80 — text source loader, admin build only
- [-] `PSDesc::FromFileNode` · 005050b0 — text source format (admin tools)

## `portal/shared/src/physobj/fx/repulsorfx.cpp.c` (0/4)

### RepulsorFX
- [-] `RepulsorFX::RepulsorFX` · 006aa5b0 · legions 00643bc0 — C++ constructor/destructor; Rust constructor and Drop replace it
- [~] `RepulsorFX::StreamPack` · 006aa5d0 · legions 00643be0 — unpack only, in ac2-net protocol::fx / protocol::visual (Legions FXNode::StreamPack 0x00642410 instruction-identical)
- [-] `RepulsorFX::FromFileNode` · 006aa660 — text source format (admin tools)
- [-] `RepulsorFX::ToFileNode` · 006aa8c0 — text source format (admin tools)

## `portal/shared/src/physobj/fx/streakfx.cpp.c` (0/5)

### StreakFX
- [-] `StreakFX::GetSubDataIDs` · 006a9f00 · legions 00643a80 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [-] `StreakFX::StreakFX` · 006a9f20 · legions 00643aa0 — C++ constructor/destructor; Rust constructor and Drop replace it
- [~] `StreakFX::StreamPack` · 006a9f80 · legions 00643b00 — unpack only, in ac2-net protocol::fx / protocol::visual (Legions FXNode::StreamPack 0x00642410 instruction-identical)
- [-] `StreakFX::FromFileNode` · 006aa040 — text source format (admin tools)
- [-] `StreakFX::ToFileNode` · 006aa3a0 — text source format (admin tools)

## `portal/shared/src/physobj/interface/iphysicsobj.cpp.c` (1/10)

### ICPhysicsObj
- [-] `ICPhysicsObj::~ICPhysicsObj` · 00509480 · legions 004f74d0 — C++ constructor/destructor; Rust constructor and Drop replace it
- [ ] `ICPhysicsObj::SetPlayerID` · 005094a0 · legions 004f74f0 — needs port: cite-only physics-object method
- [ ] `ICPhysicsObj::SetPlayerPhysicsEventHandler` · 005094c0 · legions 004f7510 — needs port: cite-only physics-object method
- [ ] `ICPhysicsObj::makeObject` · 005094d0 · legions 004f7520 — needs port: cite-only physics-object method
- [ ] `ICPhysicsObj::makeObject` · 005094f0 · legions 004f7540 — needs port: cite-only physics-object method
- [ ] `ICPhysicsObj::makeObject` · 00509520 · legions 004f7570 — needs port: cite-only physics-object method
- [ ] `ICPhysicsObj::makePartGroupObject` · 00509550 · legions 004f75a0 — needs port: cite-only physics-object method
- [ ] `ICPhysicsObj::GetObjectA` · 00509580 · legions 004f75d0 — needs port: cite-only physics-object method
- [x] `ICPhysicsObj::IncrementGlobalPhysicsTimeStamp` · 005095a0 · legions 004f75f0 — ICPhysicsObj::IncrementGlobalPhysicsTimeStamp - ICPhysicsObj::IncrementGlobalPhysicsTimeStamp, ac2-physics transition/mod.rs
- [-] `ICPhysicsObj::ICPhysicsObj` · 005095b0 · legions 004f7600 — C++ constructor/destructor; Rust constructor and Drop replace it

## `portal/shared/src/physobj/lookat/lookatmanager.cpp.c` (6/6)

### LookAtManager
- [x] `LookAtManager::Release` · 004fd6f0 · legions 004efee0 — LookAt chain: ac2-physics look_at.rs (LookAtManager), physics_obj_look.rs (CPhysicsObj::LookAt/LookAtDir), ac2-script efun 0x0080002A, ac2-client motion.rs
- [x] `LookAtManager::LookAtDir` · 004fd750 · legions 004eff40 — LookAt chain: ac2-physics look_at.rs (LookAtManager), physics_obj_look.rs (CPhysicsObj::LookAt/LookAtDir), ac2-script efun 0x0080002A, ac2-client motion.rs
- [x] `LookAtManager::LookAt` · 004fd780 · legions 004eff70 — LookAt chain: ac2-physics look_at.rs (LookAtManager), physics_obj_look.rs (CPhysicsObj::LookAt/LookAtDir), ac2-script efun 0x0080002A, ac2-client motion.rs
- [x] `LookAtManager::LookAtManager` · 004fd870 · legions 004f0060 — LookAt chain: ac2-physics look_at.rs (LookAtManager), physics_obj_look.rs (CPhysicsObj::LookAt/LookAtDir), ac2-script efun 0x0080002A, ac2-client motion.rs
- [x] `LookAtManager::LookAtLogic` · 004fd930 · legions 004f0120 — LookAt chain: ac2-physics look_at.rs (LookAtManager), physics_obj_look.rs (CPhysicsObj::LookAt/LookAtDir), ac2-script efun 0x0080002A, ac2-client motion.rs
- [x] `LookAtManager::Update` · 004fdb70 · legions 004f0360 — LookAtManager::Update - LookAtManager::Update, ac2-physics physics_obj_look.rs

## `portal/shared/src/physobj/minterp/constraintmanager.cpp.c` (6/6)

### ConstraintManager
- [x] `ConstraintManager::ConstraintManager` · 0050c9d0 · legions 004faa40 — ac2-physics constraint.rs (slice 4): ConstraintManager, PhysicsBody::constrain_to/unconstrain, the factor in ConvertDoMotionToLocalDesired, DoConstraint's enable
- [x] `ConstraintManager::~ConstraintManager` · 0050ca30 · legions 004faaa0 — ac2-physics constraint.rs (slice 4): ConstraintManager, PhysicsBody::constrain_to/unconstrain, the factor in ConvertDoMotionToLocalDesired, DoConstraint's enable
- [x] `ConstraintManager::SetConstraintDistance` · 0050ca40 · legions 004faab0 — ac2-physics constraint.rs (slice 4): ConstraintManager, PhysicsBody::constrain_to/unconstrain, the factor in ConvertDoMotionToLocalDesired, DoConstraint's enable
- [x] `ConstraintManager::GetConstraintFactor` · 0050ca60 · legions 004faad0 — ac2-physics constraint.rs (slice 4): ConstraintManager, PhysicsBody::constrain_to/unconstrain, the factor in ConvertDoMotionToLocalDesired, DoConstraint's enable
- [x] `ConstraintManager::Create` · 0050cad0 · legions 004fab40 — ac2-physics constraint.rs (slice 4): ConstraintManager, PhysicsBody::constrain_to/unconstrain, the factor in ConvertDoMotionToLocalDesired, DoConstraint's enable
- [x] `ConstraintManager::ConstrainTo` · 0050cb00 · legions 004fab70 — ac2-physics constraint.rs (slice 4): ConstraintManager, PhysicsBody::constrain_to/unconstrain, the factor in ConvertDoMotionToLocalDesired, DoConstraint's enable

## `portal/shared/src/physobj/minterp/imotioninterp.cpp.c` (0/1)

### IMotionInterp
- [-] `IMotionInterp::IMotionInterp` · 0069f9a0 · legions 0063c810 — C++ constructor/destructor; Rust constructor and Drop replace it

## `portal/shared/src/physobj/minterp/interpolationmanager.cpp.c` (25/33)

### InterpolationManager
- [ ] `InterpolationManager::SetPhysicsObject` · 0050cb20 — needs port: interpolation manager
- [x] `InterpolationManager::StopInterpolating` · 0050cb30 — ac2-physics interp (slice 3)
- [-] `InterpolationManager::GetDebugDisplayValues` · 0050cb60 — admin/debug or text-file tooling
- [-] `InterpolationManager::StreamPack` · 0050cbb0 · legions 004fac20 — DBO/archive serialisation of runtime state; the client does not read it from the DAT

### (free functions)
- [x] `clamp` · 0050cc10 · legions 004fac80 — ac2-physics interp (slice 3)

### InterpolationManager::QueuedPosition
- [x] `InterpolationManager::QueuedPosition::QueuedPosition` · 0050cc40 · legions 004facb0 — ac2-physics interp (slice 3)

### _STL::_Deque_iterator<_STL::pair<InterpolationManager::QueuedPosition,float>,_STL::_Nonconst_traits<_STL::pair<InterpolationManager::QueuedPosition,float>_>_>
- [-] `_STL::_Deque_iterator<_STL::pair<InterpolationManager::QueuedPosition,float>,_STL::_Nonconst_traits<_STL::pair<InterpolationManager::QueuedPosition,float>_>_>::operator+` · 0050ccd0 · legions 004fad40 — STL/MSVC or archive template helper; Rust collections replace it
- [-] `_STL::_Deque_iterator<_STL::pair<InterpolationManager::QueuedPosition,float>,_STL::_Nonconst_traits<_STL::pair<InterpolationManager::QueuedPosition,float>_>_>::operator-` · 0050cd20 · legions 004fad90 — STL/MSVC or archive template helper; Rust collections replace it

### InterpolationManager::InterpolationPath
- [x] `InterpolationManager::InterpolationPath::FindKeyframesForDistance` · 0050cfe0 · legions 004fb050 — ac2-physics interp (slice 3)
- [x] `InterpolationManager::InterpolationPath::GetPointFromDistance` · 0050d150 · legions 004fb1c0 — ac2-physics interp (slice 3)
- [x] `InterpolationManager::InterpolationPath::HasContact` · 0050d380 · legions 004fb3f0 — ac2-physics interp (slice 3)
- [x] `InterpolationManager::InterpolationPath::ShouldImpulseHere` · 0050d3c0 · legions 004fb430 — ac2-physics interp (slice 3)
- [x] `InterpolationManager::InterpolationPath::GetInterpolatedValuesForDistance` · 0050d500 · legions 004fb570 — ac2-physics interp (slice 3)
- [x] `InterpolationManager::InterpolationPath::FindClosestPoint` · 0050d660 · legions 004fb6d0 — ac2-physics interp (slice 3)
- [x] `InterpolationManager::InterpolationPath::GetLast` · 0050e090 · legions 004fc100 — ac2-physics interp (slice 3)
- [x] `InterpolationManager::InterpolationPath::GetTotalDistance` · 0050e0c0 · legions 004fc130 — ac2-physics interp (slice 3)
- [x] `InterpolationManager::InterpolationPath::GetInterpolatedDistance` · 0050e100 · legions 004fc170 — ac2-physics interp (slice 3)

### InterpolationManager
- [x] `InterpolationManager::DoConstraint` · 0050e1c0 · legions 004fc230 — ac2-physics constraint.rs (slice 4): ConstraintManager, PhysicsBody::constrain_to/unconstrain, the factor in ConvertDoMotionToLocalDesired, DoConstraint's enable
- [x] `InterpolationManager::Blip` · 0050e340 · legions 004fc3b0 — ac2-physics interp (slice 3)
- [ ] `InterpolationManager::GetLastReceivedMotionVector` · 0050e430 — needs port: interpolation manager
- [x] `InterpolationManager::DoImpulseLogic` · 0050e8b0 · legions 004fc920 — ac2-physics interp (slice 3)

### _STL::deque<_STL::pair<InterpolationManager::QueuedPosition,float>,_STL::allocator<_STL::pair<InterpolationManager::QueuedPosition,float>_>_>
- [-] `_STL::deque<_STL::pair<InterpolationManager::QueuedPosition,float>,_STL::allocator<_STL::pair<InterpolationManager::QueuedPosition,float>_>_>::~deque<_STL::pair<InterpolationManager::QueuedPosition,float>,_STL::allocator<_STL::pair<InterpolationManager::QueuedPosition,float>_>_>` · 0050ed60 · legions 004fcdd0 — STL/MSVC or archive template helper; Rust collections replace it

### InterpolationManager::InterpolationPath
- [x] `InterpolationManager::InterpolationPath::ThrowAway` · 0050f7d0 · legions 004fd840 — ac2-physics interp (slice 3)

### InterpolationManager
- [x] `InterpolationManager::Seek` · 0050fa00 · legions 004fda70 — ac2-physics interp (slice 3)
- [x] `InterpolationManager::CheckProgress` · 0050fe00 · legions 004fde70 — ac2-physics interp (slice 3)
- [x] `InterpolationManager::PerformInterpolation` · 0050fec0 · legions 004fdf30 — ac2-physics interp (slice 3)

### InterpolationManager::InterpolationPath
- [x] `InterpolationManager::InterpolationPath::Add` · 005103a0 · legions 004fe410 — ac2-physics interp (slice 3)

### InterpolationManager
- [x] `InterpolationManager::InterpolationManager` · 005105a0 · legions 004fe610 — ac2-physics interp (slice 3)
- [x] `InterpolationManager::Create` · 005106d0 · legions 004fe740 — ac2-physics interp (slice 3)
- [x] `InterpolationManager::AddImpulseEvent` · 00510700 · legions 004fe770 — ac2-physics interp (slice 3)
- [x] `InterpolationManager::AddEvent` · 00510950 · legions 004fe9c0 — ac2-physics interp (slice 3)
- [x] `InterpolationManager::Update` · 00510b60 — ac2-physics interp (slice 3)

### InterpolationManager::InterpolationPath
- [-] `InterpolationManager::InterpolationPath::StreamPack` · 00510cf0 · legions 004fed00 — DBO/archive serialisation of runtime state; the client does not read it from the DAT

## `portal/shared/src/physobj/minterp/motioninterp.cpp.c` (40/62)

### CMotionInterp
- [ ] `CMotionInterp::Init` · 00509630 · legions 004f7680 — needs port: cite-only physics-object method
- [x] `CMotionInterp::SetMIDESC` · 005096c0 — SetMIDESC: loader thread reads the MotionInterpDesc (Request::MotionDesc, world_data motion_desc); client_objects request_motion_desc / motion_desc_loaded swap the body's table (MotionInterp::set_desc); missing DBObj keeps the table
- [ ] `CMotionInterp::GetBehaviorTableID` · 00509720 — needs port: cite-only physics-object method
- [ ] `CMotionInterp::SetPhysobjRep` · 00509730 — needs port: cite-only physics-object method
- [ ] `CMotionInterp::SetTerrainType` · 00509770 — needs port: cite-only physics-object method
- [ ] `CMotionInterp::GetPhysObj` · 00509780 — needs port: cite-only physics-object method
- [x] `CMotionInterp::GetSeaFloorMultiplier` · 00509790
- [x] `CMotionInterp::GetWaterMultiplier` · 005097b0
- [x] `CMotionInterp::IsMovingTo` · 005097d0 — ac2-physics moveto (slice 3)
- [x] `CMotionInterp::IsTurningTo` · 005097e0 — ac2-physics MovementState turning_to / turn_to_degrees
- [ ] `CMotionInterp::ReachedPersistentTarget` · 005097f0 — needs port: cite-only physics-object method
- [x] `CMotionInterp::ConvertDoMotionToLocalDesired` · 00509800 — ac2-physics interp (slice 3)
- [x] `CMotionInterp::GetDesiredModeState` · 00509910 — CMotionInterp::GetDesiredModeState - CMotionInterp method: ac2-physics motion_interp.rs, moveto/manager.rs; ac2-world ac2_motion_interp.rs
- [x] `CMotionInterp::UpdateTargetInfo` · 00509930 — ac2-physics moveto (slice 3)
- [x] `CMotionInterp::MoveToEvent` · 00509950 — ac2-physics moveto (slice 3)
- [x] `CMotionInterp::InterruptMoveTo` · 00509960 — ac2-physics moveto (slice 3)
- [x] `CMotionInterp::HandleMoveToCallback` · 00509970 — CMotionInterp::HandleMoveToCallback - CMotionInterp method: ac2-physics motion_interp.rs, moveto/manager.rs; ac2-world ac2_motion_interp.rs
- [x] `CMotionInterp::HitGround` · 00509980 — ac2-physics anim_rep / motion_interp_behaviors (slice 3)
- [x] `CMotionInterp::LeaveGround` · 00509a10 — ac2-physics anim_rep / motion_interp_behaviors (slice 3)
- [ ] `CMotionInterp::GetMaxSpeed` · 00509a60 — needs port: cite-only physics-object method
- [ ] `CMotionInterp::SetLocalDesiredVelocity` · 00509a70 — needs port: cite-only physics-object method
- [x] `CMotionInterp::IsAffectedByGravity` · 00509a90
- [x] `CMotionInterp::ResetHoverHeight` · 00509aa0 — CMotionInterp::ResetHoverHeight - CMotionInterp method: ac2-physics motion_interp.rs, moveto/manager.rs; ac2-world ac2_motion_interp.rs
- [x] `CMotionInterp::GetHoverHeight` · 00509ab0
- [ ] `CMotionInterp::GetDefaultHoverHeight` · 00509ae0 — needs port: cite-only physics-object method
- [x] `CMotionInterp::TurnTo` · 00509af0
- [x] `CMotionInterp::StopTurnTo` · 00509b40 · legions 004f7b80
- [x] `CMotionInterp::StopInterpolating` · 00509b70 — ac2-physics interp (slice 3)
- [ ] `CMotionInterp::IsInterpolating` · 00509b80 — needs port: cite-only physics-object method
- [ ] `CMotionInterp::GetRemoteDoMotionValues` · 00509ba0 — needs port: cite-only physics-object method
- [ ] `CMotionInterp::MakeInterpolationManager` · 00509bc0 · legions 004f7c00 — needs port: cite-only physics-object method
- [x] `CMotionInterp::MakeConstraintManager` · 00509bf0 · legions 004f7c30 — ac2-physics constraint.rs (slice 4): ConstraintManager, PhysicsBody::constrain_to/unconstrain, the factor in ConvertDoMotionToLocalDesired, DoConstraint's enable
- [ ] `CMotionInterp::HandleLeaveWorld` · 00509c90 · legions 004f7cd0 — needs port: cite-only physics-object method
- [x] `CMotionInterp::ConstrainTo` · 00509cb0 · legions 004f7cf0 — ac2-physics constraint.rs (slice 4): ConstraintManager, PhysicsBody::constrain_to/unconstrain, the factor in ConvertDoMotionToLocalDesired, DoConstraint's enable
- [x] `CMotionInterp::UnConstrain` · 00509cd0 — ac2-physics constraint.rs (slice 4): ConstraintManager, PhysicsBody::constrain_to/unconstrain, the factor in ConvertDoMotionToLocalDesired, DoConstraint's enable
- [x] `CMotionInterp::GetConstraintFactor` · 00509ce0 — ac2-physics constraint.rs (slice 4): ConstraintManager, PhysicsBody::constrain_to/unconstrain, the factor in ConvertDoMotionToLocalDesired, DoConstraint's enable
- [-] `CMotionInterp::GetInterpolationValuesForDebugDisplay` · 00509d00 — admin/debug or text-file tooling

### (free functions)
- [-] `STREAMPACK_PACKOBJ` · 00509d20 · legions 004f7d60 — STL/MSVC or archive template helper; Rust collections replace it

### CMotionInterp
- [-] `CMotionInterp::Destroy` · 00509de0 · legions 004f7e20 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [x] `CMotionInterp::SetOmegaVelocity` · 00509ed0 · legions 004f7f10
- [x] `CMotionInterp::CalculateOmegaAcceleration` · 00509f20 · legions 004f7f60
- [x] `CMotionInterp::DoMotion` · 0050a1b0
- [x] `CMotionInterp::ClampVelocity` · 0050a290 — ac2-physics interp (slice 3)
- [x] `CMotionInterp::InterpolateTo` · 0050a420 · legions 004f8460 — ac2-physics interp (slice 3)
- [x] `CMotionInterp::ImpulseAt` · 0050a480 · legions 004f84c0 — ac2-physics interp (slice 3)
- [~] `CMotionInterp::UpdateMovement` · 0050a4f0 · legions 004f8530 — animation behaviours/FX (run, skid, land, fall, swim) and the Interpolation/MoveTo managers not ported
- [-] `CMotionInterp::GetPackBitfield` · 0050b080 · legions 004f90c0 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [-] `CMotionInterp::StreamPack` · 0050b230 · legions 004f9270 — DBO/archive serialisation of runtime state; the client does not read it from the DAT
- [x] `CMotionInterp::FindMotionValues` · 0050b470 · legions 004f94b0
- [x] `CMotionInterp::GetMotionValuesForElement` · 0050b540
- [x] `CMotionInterp::GetMotionValues` · 0050b5d0
- [x] `CMotionInterp::GetDesiredMotionValues` · 0050b660 · legions 004f96a0
- [ ] `CMotionInterp::ClearTerrainModifiers` · 0050b710 — needs port: cite-only physics-object method
- [x] `CMotionInterp::AddTerrainModifier` · 0050b720 — CMotionInterp::AddTerrainModifier - CMotionInterp method: ac2-physics motion_interp.rs, moveto/manager.rs; ac2-world ac2_motion_interp.rs
- [-] `CMotionInterp::~CMotionInterp` · 0050b740 · legions 004f9780 — C++ constructor/destructor; Rust constructor and Drop replace it
- [x] `CMotionInterp::GetTerrainType` · 0050b7b0 — CMotionInterp::GetTerrainType - CMotionInterp method: ac2-physics motion_interp.rs, moveto/manager.rs; ac2-world ac2_motion_interp.rs
- [x] `CMotionInterp::GetConstraintManager` · 0050b7c0 — ac2-physics constraint.rs (slice 4): ConstraintManager, PhysicsBody::constrain_to/unconstrain, the factor in ConvertDoMotionToLocalDesired, DoConstraint's enable
- [x] `CMotionInterp::GetTurnToDirection` · 0050b7d0 · legions 004f9810 — ac2-physics MovementState turning_to / turn_to_degrees
- [x] `CMotionInterp::GetAcceleration` · 0050b7f0
- [x] `CMotionInterp::GetOmegaAcceleration` · 0050b800
- [ ] `CMotionInterp::GetAccelerationScale` · 0050b810 — needs port: cite-only physics-object method
- [x] `CMotionInterp::CMotionInterp` · 0050b820 · legions 004f9860

## `portal/shared/src/physobj/minterp/motioninterpdesc.cpp.c` (4/25)

### MotionInterpDesc
- [x] `MotionInterpDesc::StreamPack` · 0069dad0 · legions 0063b9b0

### TerrainList
- [x] `TerrainList::StreamPack` · 0069db60 · legions 0063ba40

### MSList
- [x] `MSList::StreamPack` · 0069dba0 · legions 0063ba80

### MotionValues
- [-] `MotionValues::operator=` · 0069dbf0 · legions 0063bad0 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it

### MotionInterpDesc
- [-] `MotionInterpDesc::ToFileNode` · 0069dd10 — text source format (admin tools)

### HashTable<unsigned_long,MotionValues>
- [-] `HashTable<unsigned_long,MotionValues>::operator=` · 0069dea0 · legions 0063bc20 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### MotionInterpDesc
- [x] `MotionInterpDesc::GetMotionValues` · 0069df90 · legions 0063bd10
- [-] `MotionInterpDesc::Destroy` · 0069e240 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it

### TerrainList
- [-] `TerrainList::FromFileNode` · 0069e250 — text source format (admin tools)

### PHashTable<ModeStateKey,unsigned_long>
- [-] `PHashTable<ModeStateKey,unsigned_long>::~PHashTable<ModeStateKey,unsigned_long>` · 0069e490 · legions 0063c050 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it
- [-] `PHashTable<ModeStateKey,unsigned_long>::StreamPack` · 0069e4e0 — DBO/archive serialisation of runtime state; the client does not read it from the DAT

### TerrainList
- [-] `TerrainList::TerrainList` · 0069e7b0 — C++ constructor/destructor; Rust constructor and Drop replace it

### MotionInterpDesc
- [-] `MotionInterpDesc::MotionInterpDesc` · 0069e810 · legions 0063c300 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `MotionInterpDesc::GetDBOType` · 0069e880 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it

### HashTable<unsigned_long,TerrainList>
- [-] `HashTable<unsigned_long,TerrainList>::operator=` · 0069e890 · legions 0063c380 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### MotionInterpDesc
- [-] `MotionInterpDesc::Allocate` · 0069e9c0 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it

### MSList
- [-] `MSList::FromFileNode` · 0069ea70 — text source format (admin tools)
- [-] `MSList::operator=` · 0069ee60 · legions 0063c4f0 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [-] `MSList::MSList` · 0069eea0 · legions 0063c530 — C++ constructor/destructor; Rust constructor and Drop replace it

### SmartArray<MSList>
- [-] `SmartArray<MSList>::operator=` · 0069eff0 · legions 0063c680 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### PSmartArray<MSList>
- [-] `PSmartArray<MSList>::StreamPack` · 0069f080 · legions 0063c710 — DBO/archive serialisation of runtime state; the client does not read it from the DAT

### SmartArray<MSList>
- [-] `SmartArray<MSList>::AddToEnd` · 0069f130 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### MotionInterpDesc
- [-] `MotionInterpDesc::CopyInto` · 0069f1b0 · legions 0063c7c0 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [-] `MotionInterpDesc::ParseValuesNode` · 0069f200 — admin/debug or text-file tooling
- [-] `MotionInterpDesc::FromFileNode` · 0069f6f0 — text source format (admin tools)

## `portal/shared/src/physobj/minterp/motioninterpfactory.cpp.c` (0/1)

### CMotionInterpFactory
- [ ] `CMotionInterpFactory::Create` · 00509610 · legions 004f7660 — needs port: motion table decode

## `portal/shared/src/physobj/minterp/motionvalues.cpp.c` (3/23)

### MotionValues
- [x] `MotionValues::MotionValues` · 0069cc40 · legions 0063b6d0

### Heading
- [-] `Heading::get_degrees` · 0069cca0 · legions 0063b730 — angle conversion

### MotionValues
- [-] `MotionValues::GetDrag` · 0069ccb0 · legions 0063b740 — plain motion-table field read
- [-] `MotionValues::GetTerminalVel` · 0069ccc0 · legions 0063b750 — plain motion-table field read
- [-] `MotionValues::GetSinkOffset` · 0069ccd0 · legions 0063b760 — plain motion-table field read
- [-] `MotionValues::GetMaxForwardVel` · 0069cce0 · legions 0063b770 — plain motion-table field read
- [-] `MotionValues::GetMaxBackwardVel` · 0069ccf0 · legions 0063b780 — plain motion-table field read
- [-] `MotionValues::GetMaxStrafeVel` · 0069cd00 · legions 0063b790 — plain motion-table field read
- [-] `MotionValues::GetStillTurnVel` · 0069cd10 · legions 0063b7a0 — plain motion-table field read
- [-] `MotionValues::GetForwardAcl` · 0069cd20 · legions 0063b7b0 — plain motion-table field read
- [-] `MotionValues::GetBackwardAcl` · 0069cd30 · legions 0063b7c0 — plain motion-table field read
- [-] `MotionValues::GetStrafeAcl` · 0069cd40 · legions 0063b7d0 — plain motion-table field read

### GameplayOptionsProfile
- [x] `GameplayOptionsProfile::GetDamageTextRangeOther` · 0069cd50 · legions 0063b7e0 — ac2-script glue::options / examination (Legions layout and efun switch)

### MotionValues
- [-] `MotionValues::GetVertAcl` · 0069cd60 · legions 0063b7f0 — plain motion-table field read

### DirectXPlayingSound
- [-] `DirectXPlayingSound::GetPriority` · 0069cd70 · legions 0063b800 — DirectSound plumbing; ac2-audio owns it

### MotionValues
- [-] `MotionValues::GetBackwardSB` · 0069cd80 · legions 0063b810 — plain motion-table field read
- [-] `MotionValues::GetStrafeSB` · 0069cd90 · legions 0063b820 — plain motion-table field read
- [-] `MotionValues::GetTurnSB` · 0069cda0 · legions 0063b830 — plain motion-table field read

### CPartArray
- [-] `CPartArray::GetOpacity` · 0069cdb0 · legions 0063b850 — render plumbing

### MotionValues
- [-] `MotionValues::GetHoverHeight` · 0069cdc0 · legions 0063b860 — plain motion-table field read
- [x] `MotionValues::StreamPack` · 0069cdd0 · legions 0063b870
- [-] `MotionValues::ToFileNode` · 0069cf10 — text source format (admin tools)
- [-] `MotionValues::FromFileNode` · 0069d320 — text source format (admin tools)

## `portal/shared/src/physobj/minterp/movementparameters.cpp.c` (3/43)

### MovementParameters
- [-] `MovementParameters::set_persistent_flag` · 0069c1f0 · legions 0063ac10 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `MovementParameters::set_persistent_fail_exits_flag` · 0069c210 · legions 0063ac30 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `MovementParameters::set_move_towards_flag` · 0069c230 · legions 0063ac50 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `MovementParameters::set_move_away_flag` · 0069c250 · legions 0063ac70 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `MovementParameters::set_turn_flag` · 0069c270 · legions 0063ac90 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `MovementParameters::set_object_flag` · 0069c290 · legions 0063acb0 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `MovementParameters::set_use_cylinders` · 0069c2b0 · legions 0063acd0 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `MovementParameters::set_target_object_id` · 0069c2d0 · legions 0063acf0 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `MovementParameters::set_desired_distance` · 0069c2f0 · legions 0063ad10 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `MovementParameters::set_max_success_distance` · 0069c300 · legions 0063ad20 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `MovementParameters::set_min_success_distance` · 0069c320 · legions 0063ad40 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `MovementParameters::set_target_heading` · 0069c340 · legions 0063ad60 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `MovementParameters::set_offset_vector` · 0069c390 · legions 0063adb0 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `MovementParameters::set_context_id` · 0069c420 · legions 0063ae40 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `MovementParameters::get_move_flag` · 0069c430 · legions 0063ae50 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `MovementParameters::get_turn_flag` · 0069c440 · legions 0063ae60 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `MovementParameters::get_away_flag` · 0069c450 · legions 0063ae70 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `MovementParameters::get_persistent_flag` · 0069c460 · legions 0063ae80 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `MovementParameters::get_object_flag` · 0069c470 · legions 0063ae90 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `MovementParameters::get_use_cylinders` · 0069c480 · legions 0063aea0 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `MovementParameters::get_target_object_id` · 0069c490 · legions 0063aeb0 — plain field accessor on a runtime record; Rust struct field replaces it

### FellowInfo
- [x] `FellowInfo::GetName` · 0069c4b0 · legions 0063aed0 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs

### MovementParameters
- [-] `MovementParameters::get_target_heading` · 0069c4c0 · legions 0063aee0 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `MovementParameters::get_offset_vector` · 0069c4e0 · legions 0063af00 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `MovementParameters::get_context_id` · 0069c4f0 · legions 0063af10 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `MovementParameters::set_callback_when_in_range` · 0069c500 · legions 0063af20 — plain field accessor on a runtime record; Rust struct field replaces it

### MovementReturn
- [-] `MovementReturn::SetError` · 0069c520 · legions 0063af40 — plain field accessor on a runtime record; Rust struct field replaces it

### UISceneEntity
- [-] `UISceneEntity::SetInstanceID` · 0069c530 · legions 00579750 — UI accessor; ac2-ui owns it

### MovementParameters
- [~] `MovementParameters::StreamPack` · 0069c550 · legions 0063afe0 — unpack only, in ac2-net protocol::movement (Legions instruction-identical)
- [-] `MovementParameters::set_target_position` · 0069c6e0 · legions 0063b170 — plain field accessor on a runtime record; Rust struct field replaces it

### GamePlayUI
- [-] `GamePlayUI::GetSmartBox` · 0069c700 · legions 0063b190 — UI accessor; ac2-ui owns it

### MovementReturn
- [-] `MovementReturn::SetPosition` · 0069c710 · legions 0063b1a0 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `MovementReturn::StreamPack` · 0069c730 · legions 0063b1c0 — DBO/archive serialisation of runtime state; the client does not read it from the DAT

### MovementParameters
- [-] `MovementParameters::pop_position_list` · 0069c7f0 · legions 0063b280 — plain field accessor on a runtime record; Rust struct field replaces it

### List<Position>
- [-] `List<Position>::operator=` · 0069c800 · legions 0063b290 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### MovementParameters
- [x] `MovementParameters::MovementParameters` · 0069c8a0 · legions 0063b330 — ac2-physics moveto (slice 3)
- [-] `MovementParameters::set_position_list` · 0069c9c0 · legions 0063b450 — plain field accessor on a runtime record; Rust struct field replaces it

### MovementReturn
- [-] `MovementReturn::MovementReturn` · 0069c9d0 · legions 0063b460 — C++ constructor/destructor; Rust constructor and Drop replace it

### MovementParameters
- [-] `MovementParameters::operator=` · 0069ca40 · legions 0063b4d0 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it

### MovementReturn
- [-] `MovementReturn::operator=` · 0069cae0 · legions 0063b570 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [-] `MovementReturn::SetMovementParameters` · 0069cb60 · legions 0063b5f0 — plain field accessor on a runtime record; Rust struct field replaces it

### MovementParameters
- [x] `MovementParameters::MovementParameters` · 0069cb70 · legions 0063b600 — ac2-physics moveto (slice 3)

### MovementReturn
- [-] `MovementReturn::MovementReturn` · 0069cbe0 · legions 0063b670 — C++ constructor/destructor; Rust constructor and Drop replace it

## `portal/shared/src/physobj/minterp/movetomanager.cpp.c` (13/19)

### Position
- [x] `Position::add_local_offset` · 0050b990 · legions 004f99d0 — ac2-physics interp (slice 3)

### MoveToManager
- [-] `MoveToManager::Destroy` · 0050ba00 · legions 004f9a40 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [ ] `MoveToManager::GetID` · 0050ba30 · legions 004f9a70 — needs port: move-to manager
- [~] `MoveToManager::GetTargetObj` · 0050ba70 · legions 004f9ab0 — object targets come from update_move_to_target / lose_move_to_target, not a CPhysics lookup
- [x] `MoveToManager::UpdateTargetInfo` · 0050bae0 — ac2-physics moveto (slice 3)
- [ ] `MoveToManager::SetMotionInterp` · 0050bb30 · legions 004f9ba0 — needs port: move-to manager
- [x] `MoveToManager::StartMoveTo` · 0050bbb0 · legions 004f9c20 — ac2-physics moveto (slice 3)
- [x] `MoveToManager::StartTurnTo` · 0050bd60 · legions 004f9dd0 — ac2-physics moveto (slice 3)
- [x] `MoveToManager::Stop` · 0050bde0 · legions 004f9e50 — ac2-physics moveto (slice 3)
- [x] `MoveToManager::TranslateForHovering` · 0050bec0 · legions 004f9f30 — ac2-physics moveto (slice 3)
- [-] `MoveToManager::StreamPack` · 0050bf30 — DBO/archive serialisation of runtime state; the client does not read it from the DAT
- [x] `MoveToManager::MoveToManager` · 0050c110 · legions 004fa180 — ac2-physics moveto (slice 3)
- [-] `MoveToManager::~MoveToManager` · 0050c1c0 · legions 004fa230 — C++ constructor/destructor; Rust constructor and Drop replace it
- [x] `MoveToManager::Stop` · 0050c2a0 · legions 004fa310 — ac2-physics moveto (slice 3)
- [x] `MoveToManager::Interrupt` · 0050c310 — ac2-physics moveto (slice 3)
- [x] `MoveToManager::MoveToEvent` · 0050c330 — ac2-physics moveto (slice 3)
- [x] `MoveToManager::UpdateMoveTo` · 0050c450 · legions 004fa4c0 — ac2-physics moveto (slice 3)
- [x] `MoveToManager::UpdateTurnTo` · 0050c790 · legions 004fa800 — ac2-physics moveto (slice 3)
- [x] `MoveToManager::Update` · 0050c960 · legions 004fa9d0 — ac2-physics moveto (slice 3)

## `portal/shared/src/physobj/missile/missilemanager.cpp.c` (1/58)

### Random
- [ ] `Random::Check` · 00510ef0 · legions 004fef00 — needs port: Random::Check

### CMissileManager
- [ ] `CMissileManager::Place` · 00510f20 · legions 004fef30 — needs port: missile flight
- [ ] `CMissileManager::Fire` · 00510f30 · legions 004fef40 — needs port: missile flight
- [ ] `CMissileManager::ClientInitMissileParameters` · 00510f40 · legions 004fef50 — needs port: missile flight
- [ ] `CMissileManager::IsInWorld` · 00510fb0 · legions 004fefc0 — needs port: missile flight
- [ ] `CMissileManager::GetMinTime` · 00510fd0 · legions 004fefe0 — needs port: missile flight
- [ ] `CMissileManager::GetMaxTime` · 00511050 · legions 004ff060 — needs port: missile flight
- [ ] `CMissileManager::GetGravitationalVelocity` · 005110f0 · legions 004ff100 — needs port: missile flight
- [ ] `CMissileManager::AdjustSpeed` · 005111b0 · legions 004ff1c0 — needs port: missile flight
- [ ] `CMissileManager::UpdateFuelUse` · 005112d0 · legions 004ff2e0 — needs port: missile flight
- [ ] `CMissileManager::OutOfFuel` · 00511330 · legions 004ff340 — needs port: missile flight
- [ ] `CMissileManager::ReportOutOfFuel` · 00511360 · legions 004ff370 — needs port: missile flight
- [ ] `CMissileManager::SetActivated` · 00511380 · legions 004ff390 — needs port: missile flight
- [ ] `CMissileManager::SetMoving` · 00511390 · legions 004ff3a0 — needs port: missile flight
- [ ] `CMissileManager::ShouldDetonate` · 005113f0 · legions 004ff400 — needs port: missile flight

### Position
- [-] `Position::ForwardVector` · 00511560 · legions 004ff570 — vector/frame math
- [x] `Position::GetLocalHeading` · 005115a0 · legions 004ff5b0 — ac2-script glue::position (ported beside the gPosition glue)

### CMissileManager
- [-] `CMissileManager::CMissileManager` · 00511670 · legions 004ff680 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `CMissileManager::~CMissileManager` · 005117b0 · legions 004ff7c0 — C++ constructor/destructor; Rust constructor and Drop replace it
- [ ] `CMissileManager::PlaceAndFire` · 005117f0 · legions 004ff800 — needs port: missile flight

### MissileTargetingParameters
- [-] `MissileTargetingParameters::operator=` · 00511810 · legions 004ff820 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it

### CMissileManager
- [ ] `CMissileManager::CalcGravitationalClosest` · 00511900 · legions 004ff910 — needs port: missile flight
- [ ] `CMissileManager::CalcMinMaxTime` · 00511bb0 · legions 004ffbc0 — needs port: missile flight
- [ ] `CMissileManager::CalcGravitationalPitch` · 00511d10 · legions 004ffd20 — needs port: missile flight
- [ ] `CMissileManager::GreaterPitch` · 00511f90 · legions 004fffa0 — needs port: missile flight
- [ ] `CMissileManager::ChaseVelocity` · 00512000 · legions 00500010 — needs port: missile flight
- [ ] `CMissileManager::AdjustHeight` · 00512180 · legions 00500190 — needs port: missile flight
- [ ] `CMissileManager::GetDesiredVelocity` · 005121d0 · legions 005001e0 — needs port: missile flight
- [ ] `CMissileManager::CollideVelocity` · 005122f0 · legions 00500300 — needs port: missile flight
- [ ] `CMissileManager::UpdateMissilePhysics` · 005124b0 · legions 005004c0 — needs port: missile flight
- [ ] `CMissileManager::GetCurrentVelocity` · 00512790 · legions 005007a0 — needs port: missile flight
- [ ] `CMissileManager::InitHeadingTowardTarget` · 005127c0 · legions 005007d0 — needs port: missile flight
- [ ] `CMissileManager::HandleCollision` · 00512890 · legions 005008a0 — needs port: missile flight

### TargetPosition
- [-] `TargetPosition::Reset` · 00512940 · legions 00500950 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `TargetPosition::SetTargetInfo` · 00512990 · legions 005009a0 — plain field accessor on a runtime record; Rust struct field replaces it

### CMissileManager
- [ ] `CMissileManager::GetStartPos` · 005129d0 · legions 005009e0 — needs port: missile flight
- [-] `CMissileManager::StreamPack` · 00512a70 · legions 00500a80 — DBO/archive serialisation of runtime state; the client does not read it from the DAT
- [ ] `CMissileManager::InitPosition` · 00512cc0 · legions 00500cd0 — needs port: missile flight
- [ ] `CMissileManager::CalcGravitationalSpeed` · 00512dc0 · legions 00500dd0 — needs port: missile flight
- [ ] `CMissileManager::PerpAdjust` · 00513020 · legions 00501030 — needs port: missile flight
- [ ] `CMissileManager::CalcAcceleration` · 00513120 · legions 00501130 — needs port: missile flight
- [ ] `CMissileManager::TargetUpdate` · 005135e0 · legions 005015f0 — needs port: missile flight
- [ ] `CMissileManager::ClientActivate` · 00513760 · legions 00501770 — needs port: missile flight
- [ ] `CMissileManager::AttemptWorldPlacement` · 00513850 · legions 00501860 — needs port: missile flight
- [ ] `CMissileManager::CalcGravitationalVelocity` · 00513920 · legions 00501930 — needs port: missile flight
- [ ] `CMissileManager::MaintDistVelocity` · 00513b20 · legions 00501b30 — needs port: missile flight
- [ ] `CMissileManager::SetupTarget` · 00513cc0 · legions 00501cd0 — needs port: missile flight
- [ ] `CMissileManager::Initialize` · 00513d30 · legions 00501d40 — needs port: missile flight
- [ ] `CMissileManager::Retarget` · 00513d80 · legions 00501d90 — needs port: missile flight
- [ ] `CMissileManager::ClientInitTargetingParameters` · 00513e60 · legions 00501e70 — needs port: missile flight
- [ ] `CMissileManager::GetInitialGravitationalFlight` · 00513e90 · legions 00501ea0 — needs port: missile flight
- [ ] `CMissileManager::InterceptVelocity` · 00513fe0 · legions 00501ff0 — needs port: missile flight
- [ ] `CMissileManager::MissileInfoUpdate` · 00514070 · legions 00502080 — needs port: missile flight
- [ ] `CMissileManager::CalcInitialVelocity` · 00514330 · legions 00502340 — needs port: missile flight
- [ ] `CMissileManager::StartMovement` · 005144d0 · legions 005024e0 — needs port: missile flight
- [ ] `CMissileManager::InquireDirection` · 005145c0 · legions 005025d0 — needs port: missile flight
- [ ] `CMissileManager::Update` · 00514740 · legions 00502750 — needs port: missile flight
- [ ] `CMissileManager::ClientRetarget` · 00514830 · legions 00502840 — needs port: missile flight

## `portal/shared/src/physobj/missile/missileparameters.cpp.c` (2/25)

### MissileParameters
- [-] `MissileParameters::IsSet` · 006aae20 · legions 00643d40 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [-] `MissileParameters::SetDetonatesOnMiss` · 006aae30 · legions 00643d50 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [-] `MissileParameters::SetDetonationProximity` · 006aae50 · legions 00643d70 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [x] `MissileParameters::MissileParameters` · 006aae80 · legions 00643da0 — MissileParameters::MissileParameters - script struct values decoded in ac2-script native/structs.rs (structs.rs:55, 104)
- [-] `MissileParameters::CopyInto` · 006aaf40 · legions 00643e60 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [x] `MissileParameters::StreamPack` · 006ab140 · legions 00644060 — ac2-net protocol::native pack+unpack (Legions layouts; WState graphs round-trip byte-exact); ac2-script still decodes its own copy until it switches
- [-] `MissileParameters::SetSpeedLimits` · 006ab580 · legions 006444a0 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [-] `MissileParameters::SetMaxAcceleration` · 006ab5a0 · legions 006444c0 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [-] `MissileParameters::SetTotalFuel` · 006ab5e0 · legions 00644500 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [-] `MissileParameters::SetUseNoFuel` · 006ab610 · legions 00644530 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [-] `MissileParameters::SetTotalLifespan` · 006ab630 · legions 00644550 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [-] `MissileParameters::SetHasNoLifespan` · 006ab650 · legions 00644570 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [-] `MissileParameters::SetDragFactor` · 006ab670 · legions 00644590 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [-] `MissileParameters::SetAffectedByGravity` · 006ab680 · legions 006445a0 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [-] `MissileParameters::SetFollowGroundType` · 006ab690 · legions 006445b0 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [-] `MissileParameters::SetConserveFuel` · 006ab6b0 · legions 006445d0 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [-] `MissileParameters::SetMaintainDistance` · 006ab6e0 · legions 00644600 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [-] `MissileParameters::SetMissileAsHoming` · 006ab700 · legions 00644620 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [-] `MissileParameters::SetMissileReaction` · 006ab720 · legions 00644640 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [-] `MissileParameters::SetTurnMethod` · 006ab750 · legions 00644670 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [-] `MissileParameters::UseEnvironmentalMaximumVelocities` · 006ab770 · legions 00644690 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [-] `MissileParameters::SetAlignType` · 006ab790 · legions 006446b0 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [-] `MissileParameters::SetMissileSticky` · 006ab7b0 · legions 006446d0 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [-] `MissileParameters::SetCanLand` · 006ab7d0 · legions 006446f0 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [-] `MissileParameters::SetMaxAccelerationUniform` · 006ab7f0 · legions 00644710 — plain missile parameter field; ac2-script glue/missile.rs holds the values

## `portal/shared/src/physobj/missile/missiletargetingparameters.cpp.c` (1/17)

### MissileTargetingParameters
- [-] `MissileTargetingParameters::~MissileTargetingParameters` · 006ab820 · legions 00644740 — C++ constructor/destructor; Rust constructor and Drop replace it

### CContinSoundInfo
- [x] `CContinSoundInfo::TestBitMask` · 006ab830 · legions 00644750 — CContinSoundInfo::TestBitMask - script struct decode ac2-script native/structs.rs; ambient_data.rs TestBitMask

### MissileTargetingParameters
- [-] `MissileTargetingParameters::MissileTargetingParameters` · 006ab840 · legions 00644760 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `MissileTargetingParameters::StreamPack` · 006ab940 · legions 00644860 — DBO/archive serialisation of runtime state; the client does not read it from the DAT
- [-] `MissileTargetingParameters::SetTargetID` · 006abad0 · legions 006449f0 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [-] `MissileTargetingParameters::SetTargetPos` · 006abb10 · legions 00644a30 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [-] `MissileTargetingParameters::SetInitialSpeed` · 006abb40 · legions 00644a60 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [-] `MissileTargetingParameters::SetInitialPitch` · 006abb60 · legions 00644a80 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [-] `MissileTargetingParameters::SetBasePosition` · 006abb80 · legions 00644aa0 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [-] `MissileTargetingParameters::ClientUseParentForBasePosition` · 006abbb0 · legions 00644ad0 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [-] `MissileTargetingParameters::UseSourceObjectForBasePosition` · 006abbc0 · legions 00644ae0 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [-] `MissileTargetingParameters::SetNoFireIfTooFar` · 006abc10 · legions 00644b30 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [-] `MissileTargetingParameters::SetMaxInitialTargetRange` · 006abc20 · legions 00644b40 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [-] `MissileTargetingParameters::SetMissileDetonationChance` · 006abc40 · legions 00644b60 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [-] `MissileTargetingParameters::SetStartTowardTarget` · 006abc60 · legions 00644b80 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [-] `MissileTargetingParameters::SetAlwaysStartTowardTarget` · 006abc70 · legions 00644b90 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [-] `MissileTargetingParameters::CopyInto` · 006abc80 · legions 00644ba0 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it

## `portal/shared/src/physobj/packclasses.cpp.c` (4/4)

### PositionPack
- [x] `PositionPack::PositionPack` · 0069bd90 · legions 0063a7b0
- [x] `PositionPack::StreamPack` · 0069be20 · legions 0063a840

### CPositionPack
- [x] `CPositionPack::CPositionPack` · 0069bfd0 · legions 0063a9f0
- [x] `CPositionPack::StreamPack` · 0069c060 · legions 0063aa80

## `portal/shared/src/physobj/partarray.cpp.c` (17/72)

### CPartArray
- [-] `CPartArray::SetOwnerID` · 004edfc0 · legions 004e0930 — render plumbing
- [-] `CPartArray::SetupCompositor` · 004ee000 · legions 004e0970 — render plumbing
- [-] `CPartArray::SetAnimMapID` · 004ee050 · legions 004e09c0 — render plumbing
- [x] `CPartArray::UpdateBones` · 004ee0c0 · legions 004e0a20 — ac2-render object/skeleton.rs (render-side bones)
- [ ] `CPartArray::SetBoundingObject` · 004ee1e0 · legions 004e0b40 — needs port: cite-only physics-object method
- [~] `CPartArray::SetPlacementFrameID` · 004ee200 · legions 004e0b60 — ac2-render skeleton: placement frame 0 only
- [-] `CPartArray::InitParts` · 004ee2b0 · legions 004e0c10 — render plumbing
- [-] `CPartArray::DestroyParts` · 004ee380 · legions 004e0ce0 — render plumbing
- [~] `CPartArray::InitBones` · 004ee3e0 · legions 004e0d40 — ac2-render skeleton: placement frame 0 only
- [-] `CPartArray::DestroyBones` · 004ee4d0 · legions 004e0e30 — render plumbing
- [ ] `CPartArray::GetSelectionSphere` · 004ee540 · legions 004e0ea0 — needs port: cite-only physics-object method
- [ ] `CPartArray::GetNumSphere` · 004ee580 — needs port: cite-only physics-object method
- [x] `CPartArray::GetSphere` · 004ee590 — CPartArray::GetSphere - CPartArray sphere/height/step reads, ac2-assets ac2_physics_mesh setup.rs and loader.rs
- [ ] `CPartArray::GetNumCylsphere` · 004ee5a0 — needs port: cite-only physics-object method
- [ ] `CPartArray::GetCylsphere` · 004ee5b0 — needs port: cite-only physics-object method
- [ ] `CPartArray::GetRadius` · 004ee5c0 · legions 004e0f20 — needs port: cite-only physics-object method
- [x] `CPartArray::GetHeight` · 004ee5d0 · legions 004e0f30 — CPartArray::GetHeight - CPartArray sphere/height/step reads, ac2-assets ac2_physics_mesh setup.rs and loader.rs
- [ ] `CPartArray::AllowsFreeHeading` · 004ee5e0 · legions 004e0f40 — needs port: cite-only physics-object method
- [x] `CPartArray::GetStepUpHeight` · 004ee5f0 · legions 004e0f50 — CPartArray::GetStepUpHeight - CPartArray sphere/height/step reads, ac2-assets ac2_physics_mesh setup.rs and loader.rs
- [x] `CPartArray::GetStepDownHeight` · 004ee610 · legions 004e0f70 — CPartArray::GetStepDownHeight - CPartArray sphere/height/step reads, ac2-assets ac2_physics_mesh setup.rs and loader.rs
- [ ] `CPartArray::SetStepDownHeight` · 004ee630 · legions 004e0f90 — needs port: cite-only physics-object method
- [ ] `CPartArray::GetPhysicsBoundingBox` · 004ee650 · legions 004e0fb0 — needs port: cite-only physics-object method
- [ ] `CPartArray::GetPhysicsContainmentRadius` · 004ee660 · legions 004e0fc0 — needs port: cite-only physics-object method
- [x] `CPartArray::CacheHasMatrixWeights` · 004ee670 · legions 004e0fd0 — ac2-render object/skeleton.rs (render-side bones)
- [-] `CPartArray::BuildRenderBoundingBox` · 004ee6c0 · legions 004e1020 — render plumbing
- [-] `CPartArray::CacheRenderBoundingBox` · 004ee700 · legions 004e1060 — render plumbing
- [-] `CPartArray::BuildDefaultRenderBoundingBox` · 004ee720 · legions 004e1080 — render plumbing
- [-] `CPartArray::CacheDefaultRenderBoundingBox` · 004ee760 · legions 004e10c0 — render plumbing
- [ ] `CPartArray::CacheHasPhysicsBSP` · 004ee780 · legions 004e10e0 — needs port: cite-only physics-object method
- [x] `CPartArray::FindObjCollisions` · 004ee7d0 · legions 004e1130 — ac2-physics objects.rs: static-object collision dispatch, containment radius, shadow box
- [x] `CPartArray::TraceRayThroughPartsPhysics` · 004ee860 · legions 004e11c0 — ac2-physics los (Terrain::los_check)
- [x] `CPartArray::RefreshMatrices` · 004ee8f0 · legions 004e1250 — ac2-render object/skeleton.rs (render-side bones)
- [x] `CPartArray::ApplyMeshReplacement` · 004ee970 · legions 004e12d0 — ac2-render object/model.rs load_object_model_with
- [-] `CPartArray::RemoveMeshReplacements` · 004ee9b0 · legions 004e1310 — render plumbing
- [x] `CPartArray::ApplyMaterialModifier` · 004ee9f0 · legions 004e1350 — ac2-render object/model.rs load_object_model_with
- [-] `CPartArray::RemoveMaterialModifiers` · 004eea40 · legions 004e13a0 — render plumbing
- [-] `CPartArray::CacheAnimations` · 004eea70 · legions 004e13d0 — render plumbing
- [ ] `CPartArray::HasHoldingLocation` · 004eea80 · legions 004e13e0 — needs port: cite-only physics-object method
- [ ] `CPartArray::FindBoneType` · 004eeaa0 · legions 004e1400 — needs port: cite-only physics-object method
- [ ] `CPartArray::FindMatrixForBoneType` · 004eeac0 · legions 004e1420 — needs port: cite-only physics-object method
- [ ] `CPartArray::CalculateBoneIndexAndOffset` · 004eeb10 · legions 004e1470 — needs port: cite-only physics-object method
- [ ] `CPartArray::InitInternal` · 004eec30 · legions 004e1590 — needs port: cite-only physics-object method
- [~] `CPartArray::DestroyAllTransientFX` · 004eec50 · legions 004e15b0 — ac2-render WorldScene::remove_object_lights on object gone (client_audio object_lights_gone); FX systems are stopped by handle; the streak, decal and repulsor lists are not part of this port
- [ ] `CPartArray::ActivateStreakEffects` · 004eed30 · legions 004e1690 — needs port: cite-only physics-object method
- [ ] `CPartArray::DeactivateStreakEffects` · 004eed80 · legions 004e16e0 — needs port: cite-only physics-object method
- [x] `CPartArray::DestroyParticleSystem` · 004eedd0 · legions 004e1730 — CPartArray::DestroyParticleSystem - CPartArray particle systems, draw and setup id, ac2-render scene/particles.rs and model.rs
- [ ] `CPartArray::SetSetupID` · 004eee60 · legions 004e17c0 — needs port (check): cited only loosely (a comment, not the function's body); verify and cite, or port
- [ ] `CPartArray::GetBoundingSphere` · 004eef20 · legions 004e1860 — needs port: cite-only physics-object method
- [x] `CPartArray::QueueParticleSystems` · 004eef90 · legions 004e18d0 — CPartArray::QueueParticleSystems - CPartArray particle systems, draw and setup id, ac2-render scene/particles.rs and model.rs
- [ ] `CPartArray::TraceRayThroughParts` · 004ef290 · legions 004e1bd0 — needs port: cite-only physics-object method
- [-] `CPartArray::SetPartGroupData` · 004ef8f0 · legions 004e2230 — render plumbing

### LocationType
- [-] `LocationType::operator=` · 004ef920 · legions 004e2260 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it

### CPartArray
- [-] `CPartArray::CalculateSkeletonPoints` · 004ef9c0 · legions 004e2300 — render plumbing
- [x] `CPartArray::Draw` · 004efb00 · legions 004e2440 — CPartArray::Draw - CPartArray particle systems, draw and setup id, ac2-render scene/particles.rs and model.rs
- [ ] `CPartArray::GetHoldingLocation` · 004f0140 · legions 004e2a80 — needs port: cite-only physics-object method

### SmartArray<PSystemInstance>
- [-] `SmartArray<PSystemInstance>::AddToEnd` · 004f0190 · legions 004e2ad0 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### SmartArray<LightSourceInstance>
- [-] `SmartArray<LightSourceInstance>::AddToEnd` · 004f01f0 · legions 004e2b30 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### CPartArray
- [x] `CPartArray::CreateParticleSystem` · 004f0260 · legions 004e2ba0 — CPartArray::CreateParticleSystem - CPartArray particle systems, draw and setup id, ac2-render scene/particles.rs and model.rs
- [ ] `CPartArray::CreateRepulsorEffect` · 004f0300 · legions 004e2c40 — needs port: cite-only physics-object method
- [ ] `CPartArray::CreateStreakEffect` · 004f0380 · legions 004e2cc0 — needs port: cite-only physics-object method
- [~] `CPartArray::CreateLightSource` · 004f0410 · legions 004e2d50 — ac2-render scene/object_lights.rs: instance keeps the offset, lifespan, fade time and creation time; the light is an Ac2LightSource subset, not a full LightSource copy
- [-] `CPartArray::GetAnimator` · 004f0500 · legions 004e2e40 — render plumbing
- [-] `CPartArray::ClearUnusedAnimations` · 004f05b0 · legions 004e2ee0 — render plumbing
- [-] `CPartArray::DestroyAnimCache` · 004f0710 · legions 004e3040 — render plumbing
- [-] `CPartArray::Destroy` · 004f0900 · legions 004e3230 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [x] `CPartArray::GetComboCVel` · 004f0a10 · legions 004e3340 — ac2-physics anim_rep / motion_interp_behaviors (slice 3)
- [x] `CPartArray::GetHoldLocations` · 004f0a80 · legions 004e33b0 — ac2-anim dbanimator / ac2-physics anim_pose (slice 3)
- [~] `CPartArray::AddAnimation` · 004f0af0 · legions 004e3420 — all triggers every frame, compositor only on refresh frames; no slider-driven layer weights; one part group
- [-] `CPartArray::CPartArray` · 004f0c10 · legions 004e3540 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `CPartArray::SetColor` · 004f0d10 · legions 004e3640 — render plumbing
- [-] `CPartArray::~CPartArray` · 004f0d40 · legions 004e3670 — C++ constructor/destructor; Rust constructor and Drop replace it
- [~] `CPartArray::DoAnimation` · 004f0de0 · legions 004e3710 — impulse triggers only (effect triggers and weenie hooks not fired); no slider-driven layer weights; one part group

## `portal/shared/src/physobj/partgroup.cpp.c` (13/44)

### CPartGroup
- [ ] `CPartGroup::InitSetup` · 004f45e0 · legions 004e6e00 — needs port: cite-only physics-object method
- [x] `CPartGroup::SetParentOffset` · 004f4610 · legions 004e6e30 — ac2-render object/skeleton.rs (render-side bones)
- [x] `CPartGroup::TraceRayPhysics` · 004f4670 · legions 004e6e90 — ac2-physics los (Terrain::los_check)
- [ ] `CPartGroup::GetBoneMatrix` · 004f46d0 · legions 004e6ef0 — needs port: cite-only physics-object method
- [-] `CPartGroup::CPartGroup` · 004f46e0 · legions 004e6f00 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `CPartGroup::~CPartGroup` · 004f47a0 · legions 004e6fc0 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `CPartGroup::SetOwnerID` · 004f47e0 · legions 004e7000 — render plumbing
- [-] `CPartGroup::SetupCompositor` · 004f4830 · legions 004e7050 — render plumbing
- [ ] `CPartGroup::RemoveChild` · 004f48d0 · legions 004e70f0 — needs port: cite-only physics-object method
- [ ] `CPartGroup::GetPartGroup` · 004f4980 · legions 004e71a0 — needs port: cite-only physics-object method
- [ ] `CPartGroup::GetTransitionGTime` · 004f49d0 · legions 004e71f0 — needs port: cite-only physics-object method
- [x] `CPartGroup::DoAnimation` · 004f4a40 · legions 004e7260 — ac2-anim dbanimator / ac2-physics anim_pose (slice 3)
- [~] `CPartGroup::GetComboCVel` · 004f4aa0 · legions 004e72c0 — ac2-physics anim_rep: no look-at, FX scripts, triggers, wind slider or AnimInfo hold loops; DoFX is an event; one part group
- [ ] `CPartGroup::GetHoldLocations` · 004f4b20 · legions 004e7340 — needs port: cite-only physics-object method
- [ ] `CPartGroup::SetParentConnection` · 004f4bb0 · legions 004e73d0 — needs port: cite-only physics-object method
- [ ] `CPartGroup::DestroyChildren` · 004f4bf0 · legions 004e7410 — needs port: cite-only physics-object method
- [ ] `CPartGroup::SetCellID` · 004f4c90 · legions 004e74b0 — needs port: cite-only physics-object method
- [x] `CPartGroup::UpdateBones` · 004f4ce0 · legions 004e7500 — ac2-render object/skeleton.rs (render-side bones)
- [ ] `CPartGroup::TraceRay` · 004f4d20 · legions 004e7540 — needs port: cite-only physics-object method
- [-] `CPartGroup::Draw` · 004f4e20 · legions 004e7640 — render plumbing
- [x] `CPartGroup::RefreshMatrices` · 004f4f10 · legions 004e7730 — ac2-render object/skeleton.rs (render-side bones)
- [ ] `CPartGroup::SetPlacementFrameID` · 004f4fc0 · legions 004e77e0 — needs port: cite-only physics-object method
- [-] `CPartGroup::BuildRenderBoundingBox` · 004f5020 · legions 004e7840 — render plumbing
- [-] `CPartGroup::BuildDefaultRenderBoundingBox` · 004f5070 · legions 004e7890 — render plumbing
- [ ] `CPartGroup::CacheHasPhysicsBSP` · 004f50c0 · legions 004e78e0 — needs port: cite-only physics-object method
- [x] `CPartGroup::FindObjCollisions` · 004f5110 · legions 004e7930 — ac2-physics objects.rs: static-object collision dispatch, containment radius, shadow box
- [ ] `CPartGroup::HasHoldingLocation` · 004f5180 · legions 004e79a0 — needs port: cite-only physics-object method
- [ ] `CPartGroup::GetHoldingLocation` · 004f51f0 · legions 004e7a10 — needs port: cite-only physics-object method
- [-] `CPartGroup::CacheAnimations` · 004f5270 · legions 004e7a90 — render plumbing
- [ ] `CPartGroup::FindMatrixForBoneType` · 004f52b0 · legions 004e7ad0 — needs port: cite-only physics-object method
- [x] `CPartGroup::FindBoneType` · 004f5320 · legions 004e7b40 — CPartGroup::FindBoneType - CPartGroup FindBoneType (ac2_fx.rs) and AddChild (ac2-render model.rs)
- [ ] `CPartGroup::DestroyParticleSystem` · 004f53a0 · legions 004e7bc0 — needs port: cite-only physics-object method
- [~] `CPartGroup::ApplyAtomCollection` · 004f5400 · legions 004e7c20 — ac2-render model.rs: mesh + material atoms; opacity atoms not applied (none in the DATs)
- [ ] `CPartGroup::UpdatePartGroupFromPGDD` · 004f5540 · legions 004e7d30 — needs port: cite-only physics-object method
- [ ] `CPartGroup::ResetParentConnection` · 004f5590 · legions 004e7d80 — needs port: cite-only physics-object method
- [x] `CPartGroup::CheckRefreshMatrices` · 004f5660 · legions 004e7e50 — ac2-render object/skeleton.rs (render-side bones)
- [x] `CPartGroup::AddChild` · 004f5710 · legions 004e7f00 — CPartGroup::AddChild - CPartGroup FindBoneType (ac2_fx.rs) and AddChild (ac2-render model.rs)
- [ ] `CPartGroup::HandleResidualChildren` · 004f5770 · legions 004e7f60 — needs port: cite-only physics-object method
- [x] `CPartGroup::BuildAndApplyAtomCollection` · 004f5910 · legions 004e8100 — ac2-render object/model.rs load_object_model_with
- [x] `CPartGroup::SubInit` · 004f5980 · legions 004e8170 — ac2-render object/model.rs load_object_model_with
- [x] `CPartGroup::Init` · 004f59f0 · legions 004e81e0 — ac2-render object/model.rs load_object_model_with
- [x] `CPartGroup::InitVisualDesc` · 004f5aa0 · legions 004e8290 — ac2-render object/model.rs load_object_model_with
- [ ] `CPartGroup::CompareAndModify` · 004f5ac0 · legions 004e82b0 — needs port: cite-only physics-object method
- [ ] `CPartGroup::CompareAndModifyChildren` · 004f5b80 · legions 004e8370 — needs port: cite-only physics-object method

## `portal/shared/src/physobj/path/progresschecker.cpp.c` (4/8)

### ProgressData
- [-] `ProgressData::StreamPack` · 006ac470 · legions 00645420 — DBO/archive serialisation of runtime state; the client does not read it from the DAT

### ProgressChecker
- [x] `ProgressChecker::reset` · 006ac660 · legions 00645820 — ac2-physics moveto (slice 3)
- [-] `ProgressChecker::~ProgressChecker` · 006ac720 · legions 00645990 — C++ constructor/destructor; Rust constructor and Drop replace it

### (free functions)
- [-] `STREAMPACK_STL_VECTOR` · 006ac750 · legions 006459c0 — STL/MSVC or archive template helper; Rust collections replace it

### ProgressChecker
- [x] `ProgressChecker::ProgressChecker` · 006ac900 · legions 00645b70 — ac2-physics moveto (slice 3)
- [x] `ProgressChecker::push` · 006ac9b0 · legions 00645c20 — ac2-physics moveto (slice 3)
- [x] `ProgressChecker::check_progress` · 006aca30 · legions 00645ca0 — ac2-physics moveto (slice 3)
- [-] `ProgressChecker::StreamPack` · 006acf30 · legions 006461a0 — DBO/archive serialisation of runtime state; the client does not read it from the DAT

## `portal/shared/src/physobj/path/simplepathmodule.cpp.c` (7/8)

### SimplePathModule
- [x] `SimplePathModule::SimplePathModule` · 006abf90 · legions 00644eb0 — ac2-physics moveto (slice 3)
- [x] `SimplePathModule::set_parameters` · 006ac010 · legions 00644f30 — ac2-physics moveto (slice 3)
- [x] `SimplePathModule::set_always_turn` · 006ac040 · legions 00644f60 — ac2-physics moveto (slice 3)
- [x] `SimplePathModule::set_use_cylinders` · 006ac050 · legions 00644f70 — ac2-physics moveto (slice 3)
- [-] `SimplePathModule::StreamPack` · 006ac060 · legions 00644f80 — DBO/archive serialisation of runtime state; the client does not read it from the DAT
- [x] `SimplePathModule::set_destination` · 006ac110 · legions 00645030 — ac2-physics moveto (slice 3)
- [x] `SimplePathModule::CalculateXYDistanceToDestination` · 006ac140 · legions 00645060 — ac2-physics moveto (slice 3)
- [x] `SimplePathModule::update` · 006ac1e0 · legions 00645100 — ac2-physics moveto (slice 3)

## `portal/shared/src/physobj/path/turnmodule.cpp.c` (3/4)

### TurnModule
- [x] `TurnModule::TurnModule` · 006abe20 · legions 00644d40 — ac2-physics moveto (slice 3)
- [x] `TurnModule::update` · 006abe50 · legions 00644d70 — ac2-physics moveto (slice 3)
- [x] `TurnModule::set_desired_heading` · 006abf00 · legions 00644e20 — ac2-physics moveto (slice 3)
- [-] `TurnModule::StreamPack` · 006abf60 — DBO/archive serialisation of runtime state; the client does not read it from the DAT

## `portal/shared/src/physobj/physicsobj.cpp.c` (146/526)

### Quaternion
- [x] `Quaternion::multiply` · 004dddc0 · legions 004d0820

### Position
- [-] `Position::set_heading` · 004dde50 · legions 004d08b0 — vector/frame math
- [x] `Position::lerp_origin` · 004dde80 · legions 004d08e0 — ac2_physics::Position::lerp_origin (across blocks the z sum is stored first)
- [x] `Position::slerp` · 004ddf50 · legions 004d09b0 — ac2-physics PhysicsBody::{drawing_position, set_mobile, is_mobile}

### EtherealPhysicsType
- [x] `EtherealPhysicsType::SetBit` · 004ddf90 · legions 004d09f0 — ac2-physics collision_types.rs (CollisionParameters sets, IsBlocking); DAT existence via the master property table in glue

### CollisionParameters
- [ ] `CollisionParameters::SetPhysicsPlacementType` · 004ddfe0 · legions 004d0a40 — needs port: ethereal/collision parameter state
- [ ] `CollisionParameters::SetPhysicsMovementType` · 004de000 · legions 004d0a60 — needs port: ethereal/collision parameter state

### CTransition
- [x] `CTransition::init_contact_plane` · 004de020 · legions 004d0a80
- [x] `CTransition::get_contact_plane` · 004de0d0 · legions 004d0b30

### CMissileManager
- [ ] `CMissileManager::DetonatesOnProximityTo` · 004de130 · legions 004d0b90 — needs port: missile flight

### CPhysicsObj
- [ ] `CPhysicsObj::InitDataIDObject` · 004de1a0 · legions 004d0c00 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::SetBoundingObject` · 004de1f0 — needs port: cite-only physics-object method
- [-] `CPhysicsObj::GetDataID` · 004de210 — admin/debug helper
- [ ] `CPhysicsObj::GetPartGroup` · 004de220 · legions 004d0c80 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::RecvEvt_SetGlobalModifier` · 004de230 — ac2-physics body.rs set_global_modifier -> motion_interp.rs global_mod (keys the motion table lookup); ac2-client motion.rs apply; test motion_interp_tests a_global_modifier_selects_its_motion_values
- [ ] `CPhysicsObj::RecvEvt_AddTerrainModifier` · 004de250 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::RecvEvt_ClearTerrainModifiers` · 004de270 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::RecvEvt_AddTargetedImpulse` · 004de280 — ac2-physics physics_obj_impulse.rs add_impulse / add_targeted_impulse (velocity add, contact cleared, push from the target's position); ac2-net physics_event.rs decodes 0x10098/0x10099; ac2-client motion.rs apply_from; tests physics_obj_tests impulse cases, physics_event tests, tests/impulse_7160.rs. The contact argument is passed through as a velocity-setter note, not separated
- [x] `CPhysicsObj::RecvEvt_DoFXScript` · 004de2b0 — ac2-client motion.rs DoFxScript -> ac2-physics body do_fx_script; the tracker in ac2-anim fxscript_tracker.rs; client test fx_effects_tests.rs (sound and particle ids); rain: client rain_sounds.rs (tests rain_sounds_tests.rs)
- [x] `CPhysicsObj::RecvEvt_LookAt` · 004de2e0 — LookAt chain: ac2-physics look_at.rs (LookAtManager), physics_obj_look.rs (CPhysicsObj::LookAt/LookAtDir), ac2-script efun 0x0080002A, ac2-client motion.rs
- [x] `CPhysicsObj::RecvEvt_LookAtDir` · 004de300 — LookAt chain: ac2-physics look_at.rs (LookAtManager), physics_obj_look.rs (CPhysicsObj::LookAt/LookAtDir), ac2-script efun 0x0080002A, ac2-client motion.rs
- [x] `CPhysicsObj::RecvEvt_StopBehavior` · 004de320 — ac2-client motion.rs
- [x] `CPhysicsObj::RecvEvt_StopLayer` · 004de330 — ac2-client motion.rs forwards SessionEvent::ObjectAction to the PhysicsBody entry point (players and other objects)
- [x] `CPhysicsObj::RecvEvt_StopMoveToCancelsBehaviors` · 004de340 — ac2-client motion.rs forwards SessionEvent::ObjectAction to the PhysicsBody entry point (players and other objects)
- [x] `CPhysicsObj::RecvEvt_StopAllWeenieBehaviors` · 004de350 — ac2-client motion.rs forwards SessionEvent::ObjectAction to the PhysicsBody entry point (players and other objects)
- [x] `CPhysicsObj::RecvEvt_DoMode` · 004de360 — ac2-client motion.rs forwards SessionEvent::ObjectAction to the PhysicsBody entry point (players and other objects)
- [x] `CPhysicsObj::RecvEvt_DoSlider` · 004de380 · legions 004d0de0 — ac2-physics physics_obj_anim / body_anim (slice 4): PhysicsBody animation entry points
- [x] `CPhysicsObj::RecvEvt_SetMIDESC` · 004de3f0 — SetMIDESC: loader thread reads the MotionInterpDesc (Request::MotionDesc, world_data motion_desc); client_objects request_motion_desc / motion_desc_loaded swap the body's table (MotionInterp::set_desc); missing DBObj keeps the table
- [x] `CPhysicsObj::RecvEvt_SetAccelerationScale` · 004de410 — ac2-physics body.rs set_velocity_scale / set_acceleration_scale / set_jump_scale / set_hover_height / do_fx / stop_fx; client motion.rs apply; tests physics_obj_tests and motion_tests
- [x] `CPhysicsObj::RecvEvt_SetVelocityScale` · 004de430 — ac2-physics body.rs set_velocity_scale / set_acceleration_scale / set_jump_scale / set_hover_height / do_fx / stop_fx; client motion.rs apply; tests physics_obj_tests and motion_tests
- [x] `CPhysicsObj::RecvEvt_SetJumpScale` · 004de450 — ac2-physics body.rs set_velocity_scale / set_acceleration_scale / set_jump_scale / set_hover_height / do_fx / stop_fx; client motion.rs apply; tests physics_obj_tests and motion_tests
- [x] `CPhysicsObj::RecvEvt_SetHoverHeight` · 004de470 — ac2-physics body.rs set_velocity_scale / set_acceleration_scale / set_jump_scale / set_hover_height / do_fx / stop_fx; client motion.rs apply; tests physics_obj_tests and motion_tests
- [x] `CPhysicsObj::RecvEvt_MoveTo` · 004de490 · legions 004d0ef0 — ac2-client motion.rs -> PhysicsBody::recv_move_to (player only); stamp in ac2-weenie; no path planner (retail passes none)
- [ ] `CPhysicsObj::RecvEvt_PositionCell` · 004de4e0 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::RecvEvt_DoFX` · 004de4f0 — CPhysicsObj::RecvEvt_DoFX - CPhysicsObj method cited in ac2-physics body.rs, frame.rs, objects.rs, movement_module.rs, selection.rs, client_audio.rs
- [x] `CPhysicsObj::RecvEvt_DoTargettedFX` · 004de500 — CPhysicsObj::RecvEvt_DoTargettedFX - CPhysicsObj method cited in ac2-physics body.rs, frame.rs, objects.rs, movement_module.rs, selection.rs, client_audio.rs
- [x] `CPhysicsObj::RecvEvt_StopFX` · 004de520 — CPhysicsObj::RecvEvt_StopFX - CPhysicsObj method cited in ac2-physics body.rs, frame.rs, objects.rs, movement_module.rs, selection.rs, client_audio.rs
- [ ] `CPhysicsObj::ApplyVisualDescToPhysicsObj` · 004de530 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::GetNumSphere` · 004de550 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::GetSphere` · 004de560 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::GetNumCylsphere` · 004de570 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::GetCylsphere` · 004de580 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::GetRadius` · 004de590 · legions 004d0ff0 — ac2-physics moveto (slice 3)
- [x] `CPhysicsObj::GetHeight` · 004de5b0 · legions 004d1010
- [x] `CPhysicsObj::GetStepUpHeight` · 004de5d0
- [ ] `CPhysicsObj::SetStepDownHeight` · 004de5f0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::GetDepth` · 004de610 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::AllowsFreeHeading` · 004de620 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::GetSortingSphere` · 004de640 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::IsVisible` · 004de660 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::GetSelectionSphere` · 004de680 · legions 004d10e0 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::SetGlobalModifier` · 004de6d0 — ac2-physics body.rs set_global_modifier -> motion_interp.rs global_mod (keys the motion table lookup); ac2-client motion.rs apply; test motion_interp_tests a_global_modifier_selects_its_motion_values
- [ ] `CPhysicsObj::AddTerrainModifier` · 004de6e0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::ClearTerrainModifiers` · 004de6f0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::AddImpulseLocal` · 004de700 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::DoFXScript` · 004de780 · legions 004d11e0 — ac2-client motion.rs DoFxScript -> ac2-physics body do_fx_script; the tracker in ac2-anim fxscript_tracker.rs; client test fx_effects_tests.rs (sound and particle ids); rain: client rain_sounds.rs (tests rain_sounds_tests.rs)
- [x] `CPhysicsObj::DoElement` · 004de7b0 · legions 004d1210 — ac2-physics anim_rep / motion_interp_behaviors (slice 3)
- [x] `CPhysicsObj::SetMode` · 004de800 · legions 004d1260 — ac2-physics physics_obj_anim / body_anim (slice 4): PhysicsBody animation entry points
- [x] `CPhysicsObj::DoSlider` · 004de840 · legions 004d12a0 — ac2-physics physics_obj_anim / body_anim (slice 4): PhysicsBody animation entry points
- [x] `CPhysicsObj::ReleaseBehavior` · 004de890 · legions 004d12f0 — ac2-physics physics_obj_anim / body_anim (slice 4): PhysicsBody animation entry points
- [x] `CPhysicsObj::StopBehavior` · 004de8d0 · legions 004d1330 — ac2-physics anim_rep / motion_interp_behaviors (slice 3)
- [x] `CPhysicsObj::StopMoveToCancelsBehaviors` · 004de900 — ac2-physics physics_obj_anim / body_anim (slice 4): PhysicsBody animation entry points
- [x] `CPhysicsObj::StopAllWeenieBehaviors` · 004de910 — ac2-physics physics_obj_anim / body_anim (slice 4): PhysicsBody animation entry points
- [x] `CPhysicsObj::StopLayer` · 004de920 · legions 004d1380 — ac2-physics physics_obj_anim / body_anim (slice 4): PhysicsBody animation entry points
- [x] `CPhysicsObj::DoMotion` · 004de940 · legions 004d13a0
- [x] `CPhysicsObj::GetElement` · 004de980 · legions 004d13f0 — ac2-physics anim_rep / motion_interp_behaviors (slice 3)
- [x] `CPhysicsObj::GetDesiredElement` · 004de990 — ac2-physics anim_rep / motion_interp_behaviors (slice 3)
- [x] `CPhysicsObj::GetMode` · 004de9a0 — ac2-physics physics_obj_anim / body_anim (slice 4): PhysicsBody animation entry points
- [ ] `CPhysicsObj::GetDesiredMode` · 004de9b0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::IsDoingBehavior` · 004de9c0 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::GetSliderCurrentValue` · 004de9d0 — ac2-physics physics_obj_anim / body_anim (slice 4): PhysicsBody animation entry points
- [x] `CPhysicsObj::GetSliderDesiredValue` · 004de9e0 — ac2-physics physics_obj_anim / body_anim (slice 4): PhysicsBody animation entry points
- [ ] `CPhysicsObj::GetDesiredMotionValues` · 004de9f0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::GetStandardMotionValues` · 004dea00 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::IsMovingTo` · 004dea10 · legions 004d1470 — CPhysicsObj::IsMovingTo - CPhysicsObj method cited in ac2-physics body.rs, frame.rs, objects.rs, movement_module.rs, selection.rs, client_audio.rs
- [x] `CPhysicsObj::InterruptMoveTo` · 004dea20 — ac2-physics physics_obj_anim / body_anim (slice 4): PhysicsBody animation entry points
- [ ] `CPhysicsObj::GetLastDoMotionValues` · 004dea30 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::GetRemoteDoMotionValues` · 004dea50 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::SetMotionInterpreter` · 004dea70 · legions 004d14d0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::GetMotionInterpreter` · 004deab0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::GetModeTracker` · 004deac0 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::InterpolateAndImpulse` · 004dead0 · legions 004d1530 — ac2-physics interp (slice 3)
- [x] `CPhysicsObj::InterpolateTo` · 004deb70 · legions 004d15d0 — ac2-physics interp (slice 3)
- [x] `CPhysicsObj::StopInterpolating` · 004dec30 · legions 004d1690 — ac2-physics interp (slice 3)
- [x] `CPhysicsObj::IsInterpolating` · 004dec50 · legions 004d16b0 — ac2-physics interp (slice 3)
- [x] `CPhysicsObj::GetConstraintFactor` · 004dec70 · legions 004d16d0 — ac2-physics constraint.rs (slice 4): ConstraintManager, PhysicsBody::constrain_to/unconstrain, the factor in ConvertDoMotionToLocalDesired, DoConstraint's enable
- [ ] `CPhysicsObj::MissileInitPlaceAndFire` · 004decb0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::MissileInitEnd` · 004dece0 · legions 004d1740 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::MissileReportOutOfFuel` · 004ded10 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::MissileReportExpired` · 004ded30 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::NumTargetedDetectionCylSpheres` · 004ded50 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::NumTargetedDetectionCylSpheres` · 004ded80 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::NumGlobalDetectionCylSpheres` · 004deda0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::RemoveTargetedDetectionSpheres` · 004dedc0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::HasDetectionCylsphere` · 004dedf0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::DeleteDetectionCylsphere` · 004dee10 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::DeleteAllTargetedDetectionCylspheres` · 004dee30 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::InqDetectors` · 004dee50 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::ReceiveDetectionUpdate` · 004dee70 · legions 004d18d0 — needs port: cite-only physics-object method
- [~] `CPhysicsObj::SetCollisionState` · 004deea0 · legions 004d1900 — minimums only: the non-zero XY and Z minimums reach the body (ac2-physics set_collision_minimums); the physics and placement masks and ethereal lists are not ported
- [ ] `CPhysicsObj::GetCollisionState` · 004deef0 · legions 004d1950 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::GetCollisionStatePtr` · 004def40 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::GetCollisionProfile` · 004def50 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::Interpermiable` · 004defc0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::InitDetectionList` · 004df050 · legions 004d1ab0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::DoIntersection` · 004df0b0 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::IsBlocking` · 004df190 · legions 004d1bf0 — ac2-physics collision_types.rs (CollisionParameters sets, IsBlocking); DAT existence via the master property table in glue
- [ ] `CPhysicsObj::WithinDetectVolume` · 004df1f0 · legions 004d1c50 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::SetTarget` · 004df300 · legions 004d1d60 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::ClearTarget` · 004df360 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::GetTargetID` · 004df370 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::SetTargetQuantum` · 004df3b0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::GetTargetQuantum` · 004df3d0 — needs port: cite-only physics-object method

### TargetInfo
- [-] `TargetInfo::operator=` · 004df3f0 · legions 004d1e50 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it

### CPhysicsObj
- [ ] `CPhysicsObj::ReceiveTargetUpdate` · 004df4f0 · legions 004d1f50 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::AddVoyeur` · 004df510 · legions 004d1f70 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::RemoveVoyeur` · 004df570 · legions 004d1fd0 — needs port: cite-only physics-object method
- [~] `CPhysicsObj::can_jump` · 004df5a0 · legions 004d2000 — no IsDoingBehavior(jump) check (behaviours not ported)
- [x] `CPhysicsObj::IsSlidingDown` · 004df670 · legions 004d20d0 — ac2-physics anim_rep / motion_interp_behaviors (slice 3)
- [x] `CPhysicsObj::ClampHeading` · 004df6a0 · legions 004d2100
- [x] `CPhysicsObj::UnclampHeading` · 004df6d0 · legions 004d2130
- [x] `CPhysicsObj::ClampOmegaVelocity` · 004df6e0 · legions 004d2140
- [x] `CPhysicsObj::UnclampOmegaVelocity` · 004df700 · legions 004d2160
- [x] `CPhysicsObj::SetVelocityRail` · 004df710 · legions 004d2170 — ac2-physics interp (slice 3)
- [ ] `CPhysicsObj::DisableVelocityRail` · 004df740 · legions 004d21a0 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::UpdateRotation` · 004df750 · legions 004d21b0
- [x] `CPhysicsObj::CullVelocity` · 004dfa20 · legions 004d2480
- [x] `CPhysicsObj::CalcAcceleration` · 004dfa90 · legions 004d24f0
- [x] `CPhysicsObj::CalcFriction` · 004dfd10 · legions 004d2770
- [x] `CPhysicsObj::GetCurrentGravity` · 004dffc0
- [ ] `CPhysicsObj::GetCurrentWind` · 004dffd0 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::CanCollide` · 004dffe0 · legions 004d2a40 — ac2-physics objects.rs: static-object collision dispatch, containment radius, shadow box
- [ ] `CPhysicsObj::AddToTempEtherealList` · 004e0050 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::UpdateCollidePhysicsMask` · 004e0080 · legions 004d2ae0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::SetContactInfo` · 004e00a0 · legions 004d2b00 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::PerformLOS` · 004e0190 — ac2-physics los (Terrain::los_check)
- [-] `CPhysicsObj::EasyDraw` · 004e01a0 — admin/debug or text-file tooling
- [~] `CPhysicsObj::TraceRay` · 004e0200 · legions 004d2c60 — ac2-render selection.rs: drawn-mesh triangles (skinned parts at their group matrix), no potential list, no selectability / LookAtSelection / own-item checks
- [-] `CPhysicsObj::get_cell_position` · 004e0380 — state-bit flag accessor
- [ ] `CPhysicsObj::GetObjectA` · 004e0410 · legions 004d2e70 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::SetWeenieObj` · 004e0430 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::GetTopLevelObject` · 004e0440 — needs port: cite-only physics-object method
- [-] `CPhysicsObj::set_cell_id` · 004e0450 — state-bit flag accessor
- [ ] `CPhysicsObj::AddStaticObjToCell` · 004e0470 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::change_cell` · 004e04c0 · legions 004d2f20 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::enter_cell` · 004e0510 · legions 004d2f70 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::leave_cell` · 004e0560 · legions 004d2fc0 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::is_valid_walkable` · 004e05a0
- [x] `CPhysicsObj::get_walkable_z` · 004e05d0
- [x] `CPhysicsObj::set_contact` · 004e05e0 · legions 004d3040 — ac2-physics interp (slice 3)
- [x] `CPhysicsObj::HandleHitGround` · 004e0620 · legions 004d3080 — ac2-physics anim_rep / motion_interp_behaviors (slice 3)
- [x] `CPhysicsObj::HandleHitWater` · 004e06c0 · legions 004d3120
- [-] `CPhysicsObj::set_frame` · 004e0720 — state-bit flag accessor
- [ ] `CPhysicsObj::SetNoCellPosition` · 004e0730 · legions 004d3190 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::SetInitialFrame` · 004e0760 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::RefreshWorldMatrix` · 004e0770 · legions 004d31d0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::IsMoving` · 004e07e0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::set_missile_velocity` · 004e0820 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::set_omega_velocity` · 004e0860 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::set_local_velocity` · 004e0890 · legions 004d32f0 — ac2-physics anim_pose fire_impulse
- [-] `CPhysicsObj::set_local_omega_velocity` · 004e08d0 — state-bit flag accessor
- [-] `CPhysicsObj::get_distance_to_position` · 004e0950 — state-bit flag accessor
- [-] `CPhysicsObj::get_distance_to_object` · 004e09a0 — state-bit flag accessor
- [-] `CPhysicsObj::get_distance_to_object_squared` · 004e09f0 — state-bit flag accessor
- [-] `CPhysicsObj::get_global_velocity` · 004e0a40 · legions 004d34a0 — state-bit flag accessor
- [x] `CPhysicsObj::get_local_velocity` · 004e0a70 · legions 004d34d0
- [x] `CPhysicsObj::get_heading` · 004e0af0
- [-] `CPhysicsObj::get_vector_heading` · 004e0b10 — state-bit flag accessor
- [ ] `CPhysicsObj::calc_heading` · 004e0b30 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::HasBoneType` · 004e0b50 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::HasHoldingLocation` · 004e0b60 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::FXSetParent` · 004e0b90 · legions 004d35f0 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::set_parent` · 004e0c00 · legions 004d3660 — CPhysicsObj::set_parent - CPhysicsObj method cited in ac2-physics body.rs, frame.rs, objects.rs, movement_module.rs, selection.rs, client_audio.rs
- [x] `CPhysicsObj::set_parent` · 004e0ca0 — CPhysicsObj::set_parent - CPhysicsObj method cited in ac2-physics body.rs, frame.rs, objects.rs, movement_module.rs, selection.rs, client_audio.rs
- [x] `CPhysicsObj::set_parent` · 004e0cd0 · legions 004d3730 — CPhysicsObj::set_parent - CPhysicsObj method cited in ac2-physics body.rs, frame.rs, objects.rs, movement_module.rs, selection.rs, client_audio.rs
- [ ] `CPhysicsObj::ApplyPhysObjProfile` · 004e0d10 · legions 004d3770 — needs port: cite-only physics-object method
- [~] `CPhysicsObj::ApplyVisualDesc` · 004e0dc0 · legions 004d3820 — ac2-weenie: the delta becomes the local description; the part rebuild (CompareAndModify) is the renderer's
- [-] `CPhysicsObj::SetDiffuseColor` · 004e0df0 — render plumbing
- [-] `CPhysicsObj::SetOpacity` · 004e0e00 — render plumbing
- [x] `CPhysicsObj::GetParticleScale` · 004e0e10 · legions 004d3870 — CPhysicsObj::GetParticleScale - CPhysicsObj method cited in ac2-physics body.rs, frame.rs, objects.rs, movement_module.rs, selection.rs, client_audio.rs
- [x] `CPhysicsObj::SetPhysicsContainmentRadius` · 004e0e30 · legions 004d3890 — ac2-physics objects.rs: static-object collision dispatch, containment radius, shadow box
- [ ] `CPhysicsObj::movement_can_be_inactive` · 004e0e90 · legions 004d38f0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::animation_can_be_inactive` · 004e1150 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::set_movement_active` · 004e1170 · legions 004d3bd0 — CPhysicsObj::set_movement_active - CPhysicsObj method cited in ac2-physics body.rs, frame.rs, objects.rs, movement_module.rs, selection.rs, client_audio.rs
- [x] `CPhysicsObj::set_animation_active` · 004e11f0 · legions 004d3c50 — ac2-physics physics_obj_anim / body_anim (slice 4): PhysicsBody animation entry points
- [-] `CPhysicsObj::set_misc_active` · 004e1250 · legions 004d3cb0 — state-bit flag accessor
- [-] `CPhysicsObj::set_edge_slide` · 004e12b0 — state-bit flag accessor
- [x] `CPhysicsObj::set_synchronize_transition` · 004e12f0 · legions 004d3d50 — ac2-physics interp (slice 3)
- [-] `CPhysicsObj::set_ignore_collisions` · 004e1330 — state-bit flag accessor
- [-] `CPhysicsObj::set_gamedraw` · 004e1360 · legions 004d3dc0 — render plumbing
- [-] `CPhysicsObj::set_toolsdraw` · 004e13a0 — render plumbing
- [x] `CPhysicsObj::set_nodraw` · 004e13e0 — ac2-script host::interface set_no_draw_physics (efun 0x800057 -> CPhysicsObj::set_nodraw): queued for the client, which keeps it per object (Client::is_no_draw) and gives the camera subject its no_draw; combat_7160 (the respawn clears it via CPlayer 0x85). The renderer does not read it for other objects
- [-] `CPhysicsObj::set_cloaked` · 004e1410 — render plumbing
- [ ] `CPhysicsObj::set_missile` · 004e1450 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::set_mobile` · 004e1480 — ac2-physics PhysicsBody::{drawing_position, set_mobile, is_mobile}
- [ ] `CPhysicsObj::align_to_path` · 004e14b0 — needs port: cite-only physics-object method
- [-] `CPhysicsObj::set_path_clipping` · 004e14e0 — state-bit flag accessor
- [x] `CPhysicsObj::set_affected_by_gravity` · 004e1520
- [-] `CPhysicsObj::set_undetectable` · 004e1550 — render plumbing
- [-] `CPhysicsObj::set_inelastic` · 004e15d0 — state-bit flag accessor
- [-] `CPhysicsObj::set_lights` · 004e1600 — render plumbing
- [-] `CPhysicsObj::set_sticky` · 004e1630 — state-bit flag accessor
- [-] `CPhysicsObj::set_pushable` · 004e1640 — state-bit flag accessor
- [-] `CPhysicsObj::set_movement_frozen` · 004e1670 — state-bit flag accessor
- [-] `CPhysicsObj::set_animation_frozen` · 004e16b0 — state-bit flag accessor
- [ ] `CPhysicsObj::SetInWorkspace` · 004e16f0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::GetSoundPosition` · 004e1720 · legions 004d4180 — needs port: cite-only physics-object method
- [-] `CPhysicsObj::SetGraphicsState` · 004e1740 — render plumbing
- [-] `CPhysicsObj::GetGraphicsState` · 004e1780 — render plumbing
- [-] `CPhysicsObj::set_state` · 004e17a0 — state-bit flag accessor
- [ ] `CPhysicsObj::ReApplyPropertyCollection` · 004e1810 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::ApplyEtherealProperty` · 004e1820 · legions 004d4280 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::ApplyRenderProperty` · 004e1880 · legions 004d42e0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::ApplyPhysicsProperty` · 004e1a40 · legions 004d44a0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::ApplyPusherProperty` · 004e1ad0 · legions 004d4530 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::ApplyGameplayProperty` · 004e1b30 · legions 004d4590 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::BuildOptionalProperties` · 004e1b80 · legions 004d45e0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::GetObjectProperties` · 004e1bf0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::InqCurrentProperties` · 004e1c30 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::InqCurrentProperty` · 004e1c60 — needs port: cite-only physics-object method
- [~] `CPhysicsObj::get_object_info` · 004e1c90 · legions 004d46f0 — no admin moves or missiles
- [x] `CPhysicsObj::AdjustPosition` · 004e1d70 · legions 004d47d0 — ac2-physics physics_obj_place.rs: SetPosition placement check (Legions identical)
- [x] `CPhysicsObj::CheckPositionInternal` · 004e1eb0 · legions 004d4910 — ac2-physics physics_obj_place.rs: SetPosition placement check (Legions identical)
- [ ] `CPhysicsObj::UpdateGravity` · 004e1f70 · legions 004d49c0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::find_inside_position` · 004e1fb0 — needs port: cite-only physics-object method
- [~] `CPhysicsObj::SetPlacementPosition` · 004e2030 · legions 004d4a80 — ac2-physics: the drawn position slerp / previous position only (PhysicsBody::drawing_position)
- [~] `CPhysicsObj::EnterWorld` · 004e20b0 — ac2-client remote.rs
- [~] `CPhysicsObj::EnterWorldSimple` · 004e2100 — ac2-client remote.rs over PhysicsBody::place (flags 0x11)
- [~] `CPhysicsObj::HandleTeleportStart` · 004e2160 · legions 004d4bb0 — teleport flow in ac2-client (client_teleport.rs); not yet: run-lock reset, target clear, portal tunnel advancement (GeneratePortals), the portal sounds
- [ ] `CPhysicsObj::OnCellAddedToFreelist` · 004e21c0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::OnCellRemovedFromFreelist` · 004e21d0 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::LeaveVisibility` · 004e21e0 · legions 004d4c30 — ac2-client remote.rs
- [x] `CPhysicsObj::ReEnterVisibility` · 004e2220 · legions 004d4c70 — ac2-client remote.rs
- [x] `CPhysicsObj::ReEnterVisibility` · 004e2240 — ac2-client remote.rs
- [x] `CPhysicsObj::IsThePlayer` · 004e2260 — ac2-physics physics_obj_anim / body_anim (slice 4): PhysicsBody animation entry points
- [x] `CPhysicsObj::SetHoverHeight` · 004e2290 — CPhysicsObj::SetHoverHeight - CPhysicsObj method cited in ac2-physics body.rs, frame.rs, objects.rs, movement_module.rs, selection.rs, client_audio.rs
- [x] `CPhysicsObj::ResetHoverHeight` · 004e22a0 — CPhysicsObj::ResetHoverHeight - CPhysicsObj method cited in ac2-physics body.rs, frame.rs, objects.rs, movement_module.rs, selection.rs, client_audio.rs
- [ ] `CPhysicsObj::GetHoverHeight` · 004e22b0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::GetDefaultHoverHeight` · 004e22c0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::GetVelocityScale` · 004e22d0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::GetAccelerationScale` · 004e22e0 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::SetVelocityScale` · 004e22f0 — Collision impact minimum (CCollisionManager::SetMinReportedImpactXY): scale times the standard values' mMaxForwardVel plus 0.1 (SetVelocityScale), and Z (SetMinReportedImpactZ): jump scale times mJumpImpulse plus 0.1 (SetJumpScale, the field read through CPartArray::GetOpacity). ac2-physics collision_impact.rs, body.rs set_velocity_scale/set_jump_scale. Tests: collision_impact tests. Retail skips the minimum when the object has no collision manager; here it is always kept (no manager).
- [ ] `CPhysicsObj::SetAccelerationScale` · 004e2340 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::SetJumpScale` · 004e2350 — CPhysicsObj::SetJumpScale - CPhysicsObj method cited in ac2-physics body.rs, frame.rs, objects.rs, movement_module.rs, selection.rs, client_audio.rs
- [ ] `CPhysicsObj::GetJumpScale` · 004e23a0 — needs port: cite-only physics-object method
- [~] `CPhysicsObj::get_instance_stamp` · 004e23b0 · legions 004d4e00 — instance stamp kept per object in ac2-weenie ObjectTable
- [ ] `CPhysicsObj::RestoreCamera` · 004e23d0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::SetCameraAnimParent` · 004e23f0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::SetCameraAnimTarget` · 004e2400 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::SetTargetedCamera` · 004e2410 · legions 004d4e60 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::DoBehaviorOnObject` · 004e2440 · legions 004d4e90 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::InitLookAtManager` · 004e2480 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::GetMouseLookState` · 004e24f0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::SetMouseLookState` · 004e2500 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::GetFirstPersonState` · 004e2510 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::SetFirstPersonState` · 004e2520 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::SetCameraLookAround` · 004e2530 · legions 004d4f80 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::SetCameraLookAtCamera` · 004e2570 · legions 004d4fc0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::LookAtSelection` · 004e25b0 · legions 004d5000 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::SetLookAtSelection` · 004e2630 · legions 004d5080 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::StopAllCameraFX` · 004e2650 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::LookAtDir` · 004e2670 · legions 004d50c0 — LookAt chain: ac2-physics look_at.rs (LookAtManager), physics_obj_look.rs (CPhysicsObj::LookAt/LookAtDir), ac2-script efun 0x0080002A, ac2-client motion.rs
- [ ] `CPhysicsObj::SetParentObject` · 004e26c0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::SetTargetObject` · 004e26f0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::SetFreeCamera` · 004e2720 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::RestorePlayerCamera` · 004e2750 — needs port: cite-only physics-object method
- [-] `CPhysicsObj::IsPlacingMarkerObjects` · 004e2760 — admin/debug or text-file tooling
- [ ] `CPhysicsObj::RecvEvt_DoStory` · 004e2770 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::LockActions` · 004e2790 · legions 004d51e0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::UnLockActions` · 004e27e0 · legions 004d5230 — needs port: cite-only physics-object method
- [-] `CPhysicsObj::CacheAnimations` · 004e2800 — render plumbing
- [ ] `CPhysicsObj::DeleteSelf` · 004e2810 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::ClearFXTarget` · 004e2860 · legions 004d52b0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::SetFXTarget` · 004e2890 · legions 004d52e0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::InqCellObjects` · 004e28b0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::InqBlockObjects` · 004e28d0 — needs port: cite-only physics-object method
- [~] `CPhysicsObj::HandleOutOfPhysicsRange` · 004e2900 · legions 004d5350 — ac2-client: the object stops stepping physics
- [~] `CPhysicsObj::HandleInPhysicsRange` · 004e2980 · legions 004d53d0 — ac2-client: the object steps physics again

### Frame
- [ ] `Frame::combine` · 004e2ae0 · legions 004d5530 — needs port: frame combine
- [ ] `Frame::combine` · 004e2b70 · legions 004d55c0 — needs port: frame combine

### Position
- [-] `Position::subtract2` · 004e2c20 · legions 004d5670 — vector/frame math

### LOSParameters
- [x] `LOSParameters::GetEndPoint` · 004e2cb0 · legions 004d5700 — ac2-physics los (Terrain::los_check)

### BBox
- [-] `BBox::BBox` · 004e2d40 · legions 004d5790 — C++ constructor/destructor; Rust constructor and Drop replace it

### SceneTool
- [x] `SceneTool::GetLocalSelectionRay` · 004e2d80 · legions 004d57d0 — ac2-render selection.rs + ac2-ui selection_ui.rs: clicking the world selects

### CPhysicsObj
- [ ] `CPhysicsObj::InitObjectBegin` · 004e2ed0 · legions 004d5920 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::InitObjectEnd` · 004e2fe0 · legions 004d5a30 — ac2-physics physics_obj.rs init_object_end and collision_impact.rs (the XY minimum from the hover height and max forward speed, the Z from the jump impulse and scale, both plus 0.1); the body keeps them
- [x] `CPhysicsObj::RecvEvt_AddImpulse` · 004e30b0 — ac2-physics physics_obj_impulse.rs add_impulse / add_targeted_impulse (velocity add, contact cleared, push from the target's position); ac2-net physics_event.rs decodes 0x10098/0x10099; ac2-client motion.rs apply_from; tests physics_obj_tests impulse cases, physics_event tests, tests/impulse_7160.rs. The contact argument is passed through as a velocity-setter note, not separated
- [x] `CPhysicsObj::GetStepDownHeight` · 004e30e0
- [-] `CPhysicsObj::GetRenderBoundingBox` · 004e3140 · legions 004d5b90 — render plumbing
- [-] `CPhysicsObj::GetDefaultRenderBoundingBox` · 004e31a0 · legions 004d5bf0 — render plumbing
- [-] `CPhysicsObj::GetScaledDefaultRenderBoundingBox` · 004e3200 — render plumbing
- [x] `CPhysicsObj::CalcPhysicsBoundingBox` · 004e3280 — ac2-physics objects.rs: static-object collision dispatch, containment radius, shadow box
- [x] `CPhysicsObj::ApplyImpulse` · 004e3310 · legions 004d5d60 — ac2-physics interp (slice 3)
- [x] `CPhysicsObj::AddImpulse` · 004e3350 — ac2-physics physics_obj_impulse.rs add_impulse / add_targeted_impulse (velocity add, contact cleared, push from the target's position); ac2-net physics_event.rs decodes 0x10098/0x10099; ac2-client motion.rs apply_from; tests physics_obj_tests impulse cases, physics_event tests, tests/impulse_7160.rs. The contact argument is passed through as a velocity-setter note, not separated
- [x] `CPhysicsObj::AddTargetedImpulse` · 004e33d0 — ac2-physics physics_obj_impulse.rs add_impulse / add_targeted_impulse (velocity add, contact cleared, push from the target's position); ac2-net physics_event.rs decodes 0x10098/0x10099; ac2-client motion.rs apply_from; tests physics_obj_tests impulse cases, physics_event tests, tests/impulse_7160.rs. The contact argument is passed through as a velocity-setter note, not separated
- [ ] `CPhysicsObj::MoveTo` · 004e3510 · legions 004d5f60 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::CreateDetectionCylsphere` · 004e3550 · legions 004d5fa0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::CheckIfSolid` · 004e35f0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::GetObjectOffsetPos` · 004e36a0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::CheckTypeMatch` · 004e39b0 · legions 004d6400 — needs port: cite-only physics-object method
- [~] `CPhysicsObj::IsLOSBlocking` · 004e3a50 — ac2-physics los: static objects only (no dynamic shadow list, blocking masks or exact drawing rep)
- [ ] `CPhysicsObj::GetTargetInfo` · 004e3a70 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::Jump` · 004e3aa0 · legions 004d64f0 — ac2-physics anim_rep / motion_interp_behaviors (slice 3)
- [~] `CPhysicsObj::CalcElement` · 004e3b90 · legions 004d65e0 — elements go through ModeTracker::DoElement; the no-cell DoElement(GROUND) is not ported
- [~] `CPhysicsObj::CalcExternalForces` · 004e3e20 · legions 004d6870 — no pushers (CCollisionManager::UpdatePusherValues)
- [~] `CPhysicsObj::transition` · 004e4170 · legions 004d6bc0 — no admin moves or missiles
- [-] `CPhysicsObj::ResetLightCache` · 004e4280 · legions 004d6cd0 — render plumbing
- [-] `CPhysicsObj::get_landscape_coord` · 004e42f0 — state-bit flag accessor
- [ ] `CPhysicsObj::remove_shadows_from_cells` · 004e4310 · legions 004d6d60 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::AreShadowObjectsComplete` · 004e4360 · legions 004d6db0 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::set_on_walkable` · 004e43b0 · legions 004d6e00 — ac2-physics anim_rep / motion_interp_behaviors (slice 3)
- [x] `CPhysicsObj::SetPlacementFrame` · 004e4410 — CPhysicsObj::SetPlacementFrame - CPhysicsObj method cited in ac2-physics body.rs, frame.rs, objects.rs, movement_module.rs, selection.rs, client_audio.rs
- [~] `CPhysicsObj::set_velocity` · 004e4430 — the F_EPSILON change test and CullVelocity in anim_pose fire_impulse; kill_velocity and PhysicsBody::set_velocity assign directly
- [x] `CPhysicsObj::set_heading` · 004e44e0
- [ ] `CPhysicsObj::SetParent` · 004e4580 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::TopParent` · 004e4590 — needs port: cite-only physics-object method
- [~] `CPhysicsObj::ApplyVisualDesc` · 004e45b0 — ac2-weenie: the delta becomes the local description; the part rebuild (CompareAndModify) is the renderer's
- [~] `CPhysicsObj::GetTotalScale` · 004e4640 — ac2-physics: vdesc scale x placement scale; the body uses the root z scale
- [ ] `CPhysicsObj::SetScaleInternal` · 004e4730 · legions 004d7180 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::clear_transient_states` · 004e47d0 · legions 004d7220 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::misc_can_be_inactive` · 004e4880 · legions 004d72d0 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::ShouldMuteSound` · 004e48e0 · legions 004d7330 — CPhysicsObj::ShouldMuteSound - CPhysicsObj method cited in ac2-physics body.rs, frame.rs, objects.rs, movement_module.rs, selection.rs, client_audio.rs
- [ ] `CPhysicsObj::ApplyPropertyGroup` · 004e4960 · legions 004d73b0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::ApplyProperty` · 004e49b0 · legions 004d7400 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::UpdateCellMaterial` · 004e4a60 · legions 004d74b0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::local_to_global` · 004e4af0 — needs port: cite-only physics-object method
- [~] `CPhysicsObj::StorePosition` · 004e4b60 — no adjust_to_outside
- [~] `CPhysicsObj::LeaveWorld` · 004e4c10 · legions 004d7660 — ac2-client remote.rs: out of the world, queued for destruction
- [~] `CPhysicsObj::HandleEnterWorld` · 004e4ca0 · legions 004d76f0 — the bookie part only
- [x] `CPhysicsObj::LeaveVisibility` · 004e4d00 — ac2-client remote.rs
- [x] `CPhysicsObj::DoFX` · 004e4d80 · legions 004d77d0 — ac2-physics body.rs set_velocity_scale / set_acceleration_scale / set_jump_scale / set_hover_height / do_fx / stop_fx; client motion.rs apply; tests physics_obj_tests and motion_tests
- [ ] `CPhysicsObj::DoTargettedFX` · 004e4dc0 · legions 004d7810 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::StopFX` · 004e4e10 · legions 004d7860 — ac2-physics body.rs set_velocity_scale / set_acceleration_scale / set_jump_scale / set_hover_height / do_fx / stop_fx; client motion.rs apply; tests physics_obj_tests and motion_tests
- [ ] `CPhysicsObj::GetGroundPosition` · 004e4e50 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::GetBonePosition` · 004e4f60 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::RecvEvt_DoFX_Private` · 004e50c0 · legions 004d7b10 — needs port: cite-only physics-object method

### DBObjGrabber<EntityDesc>
- [-] `DBObjGrabber<EntityDesc>::DBObjGrabber<EntityDesc>` · 004e5110 · legions 004d7b60 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### IntrusiveSmartPointer<SmartPointerData<CHILDLIST>_>
- [-] `IntrusiveSmartPointer<SmartPointerData<CHILDLIST>_>::create` · 004e5190 · legions 004d7be0 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### IntrusiveSmartPointer<SmartPointerData<StoryManager>_>
- [-] `IntrusiveSmartPointer<SmartPointerData<StoryManager>_>::create` · 004e51f0 · legions 004d7c40 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### IntrusiveSmartPointer<SmartPointerData<CMissileManager>_>
- [-] `IntrusiveSmartPointer<SmartPointerData<CMissileManager>_>::create` · 004e5250 · legions 004d7ca0 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### LocationType
- [-] `LocationType::LocationType` · 004e53b0 · legions 004d7e00 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `LocationType::~LocationType` · 004e53f0 · legions 004d7e40 — C++ constructor/destructor; Rust constructor and Drop replace it

### CPhysicsObj
- [ ] `CPhysicsObj::AssemblePartGroupObject` · 004e5490 · legions 004d7ee0 — needs port: cite-only physics-object method
- [-] `CPhysicsObj::Destroy` · 004e5540 · legions 004d7f90 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [-] `CPhysicsObj::GetScaledRenderBoundingBox` · 004e5660 — render plumbing
- [x] `CPhysicsObj::JumpLogic` · 004e56e0 — ac2-physics anim_rep / motion_interp_behaviors (slice 3)
- [~] `CPhysicsObj::GetHeightAboveWalkable` · 004e5780 — vertical distance to the terrain polygon instead of CObjCell::LOSCheck
- [ ] `CPhysicsObj::UpdateChild` · 004e5980 · legions 004d83d0 — needs port: cite-only physics-object method
- [~] `CPhysicsObj::handle_all_collisions` · 004e5d10 · legions 004d8760 — environment response only; object collisions and the collision manager not ported
- [~] `CPhysicsObj::CheckRayIntersect` · 004e60f0 — ac2-physics los: static objects only (no dynamic shadow list, blocking masks or exact drawing rep)
- [ ] `CPhysicsObj::SetVisualDesc` · 004e6650 · legions 004d90a0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::SetScale` · 004e6690 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::ApplyPropertyCollection` · 004e66a0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::HasMissileManager` · 004e6710 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::is_newer_then_update` · 004e6720 · legions 004d9170 — CPhysicsObj::is_newer_then_update - CPhysicsObj method cited in ac2-physics body.rs, frame.rs, objects.rs, movement_module.rs, selection.rs, client_audio.rs
- [x] `CPhysicsObj::RecvEvt_LeaveWorld` · 004e6780 — ac2-weenie ObjectTable::apply_action (newer position stamp), then LeaveWorld
- [~] `CPhysicsObj::RecvEvt_Parent` · 004e67c0 — ac2-weenie: newer position stamp, the holder, location and placement frame
- [x] `CPhysicsObj::RecvEvt_DeParent` · 004e6810 — ac2-weenie ObjectTable::apply_action
- [x] `CPhysicsObj::RecvEvt_Contain` · 004e6890 — ac2-weenie ObjectTable::apply_contain, ac2-client remote.rs
- [ ] `CPhysicsObj::DoAppearanceFX` · 004e6920 · legions 004d9370 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::RemoveAppearanceFX` · 004e6990 · legions 004d93e0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::StartStreak` · 004e6a30 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::StopStreak` · 004e6aa0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::InitVisualDesc` · 004e6b10 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::SetLocalVisualDesc` · 004e6bd0 — ac2-weenie Appearance::from_delta (the effects description is empty)
- [ ] `CPhysicsObj::ApplyAdditiveVisualDesc` · 004e6c30 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::ApplySubtractiveVisualDesc` · 004e6c40 — needs port: cite-only physics-object method
- [~] `CPhysicsObj::DoMode` · 004e6c50 · legions 004d96a0 — ac2-physics do_mode: frozen check, set_animation_active, ModeTracker::DoMode; no weenie mode hook or child propagation
- [ ] `CPhysicsObj::HandleBehaviorDone` · 004e6d00 · legions 004d9750 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::MissilePlaceAndFire` · 004e6d20 · legions 004d9770 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::MissilePlace` · 004e6d60 · legions 004d97b0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::MissileFire` · 004e6da0 · legions 004d97f0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::MissileInquireDirection` · 004e6de0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::MissileInquireCurrentVelocity` · 004e6e60 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::MissileRetarget` · 004e6eb0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::UpdateAnimationRecursive` · 004e6f30 · legions 004d9980 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::UpdateFXRecursive` · 004e6f80 · legions 004d99d0 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::UpdateMovementInternal` · 004e6ff0 · legions 004d9a40
- [~] `CPhysicsObj::FindObjCollisions` · 004e7580 — ac2-physics objects.rs find_object_collisions: the ethereal filter via IsBlocking (collision_types.rs); the admin branch on the 0x250 slot, step/missile/temp-ignore lists not ported
- [~] `CPhysicsObj::HandleCollision` · 004e79a0 · legions 004da3f0 — impact reports only: a landing impact (normal and speed, taken once by ac2-physics take_impact_report) is handed to the player's HandleCollision IFun (ac2-client collision_reported); object, environment, surface and water properties, begin/end reports and missiles are not ported
- [ ] `CPhysicsObj::CollisionRemoveObject` · 004e79e0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::AlwaysPrepareObject` · 004e7a60 — needs port: cite-only physics-object method
- [~] `CPhysicsObj::UpdateViewerDistance` · 004e7af0 — PhysicsBody::set_view_distance takes the camera distance from the caller; no scene stamp or children
- [~] `CPhysicsObj::PrepareObjectForDrawing` · 004e7c80 — ac2-physics PhysicsBody::drawing_position (slerp of the previous and current position at the render interpolation); ac2-client draws the player and every remote at it; the pose blend is not retail (Client::set_pose_blend)
- [ ] `CPhysicsObj::Draw` · 004e7db0 — needs port (check): cited only loosely (a comment, not the function's body); verify and cite, or port
- [ ] `CPhysicsObj::AddChildInternal` · 004e86c0 · legions 004db0a0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::StickySetParent` · 004e8750 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::unset_parent` · 004e8850 · legions 004db230 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::unparent_children` · 004e88a0 · legions 004db280 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::GetTotalVisualDesc` · 004e88e0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::GetAvailableModes` · 004e88f0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::IsModeAvailable` · 004e8950 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::LookUpInChildren` · 004e89c0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::LookUpInAllChildren` · 004e8a40 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::SetMIDESC` · 004e8ac0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::SetVDescScale` · 004e8ae0 — needs port: cite-only physics-object method
- [-] `CPhysicsObj::SetChildrenOpacity` · 004e8b00 · legions 004db4e0 — render plumbing
- [ ] `CPhysicsObj::DoSliderOnChild` · 004e8b60 · legions 004db540 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::LookAt` · 004e8bc0 — LookAt chain: ac2-physics look_at.rs (LookAtManager), physics_obj_look.rs (CPhysicsObj::LookAt/LookAtDir), ac2-script efun 0x0080002A, ac2-client motion.rs
- [ ] `CPhysicsObj::DoStory` · 004e8d10 · legions 004db6f0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::GetLocalVisualDesc` · 004e8d50 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::RecvEvt_InitializeMissile` · 004e8d80 · legions 004db760 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::RecvEvt_RetargetMissile` · 004e8de0 · legions 004db7c0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::MissileInitialize` · 004e8e40 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::MissileInitMissileParameters` · 004e8ec0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::MissileInitTargetingParameters` · 004e8f20 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::UpdatePositionInternal` · 004e8f80 · legions 004db960
- [~] `CPhysicsObj::ReportCollisions` · 004e9040 · legions 004dba20 — impact reports only: a landing impact (normal and speed, taken once by ac2-physics take_impact_report) is handed to the player's HandleCollision IFun (ac2-client collision_reported); object, environment, surface and water properties, begin/end reports and missiles are not ported
- [ ] `CPhysicsObj::EndAllCollisions` · 004e90c0 · legions 004dbaa0 — needs port: cite-only physics-object method
- [~] `CPhysicsObj::add_shadows_to_cells` · 004e9140 · legions 004dbb20 — ac2-physics Terrain::insert_object: shadow cells from the shadow box through land cells and portals; residency order and moving objects not ported
- [ ] `CPhysicsObj::AddChild` · 004e9200 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::RemoveChild` · 004e92b0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::set_report_collisions` · 004e92e0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::AddToLOSList` · 004e9510 · legions 004dbd90 — needs port: cite-only physics-object method
- [~] `CPhysicsObj::UpdateAnimation` · 004e9700 — ac2-physics anim_pose: distance-degraded update frames, TriggerUpdate fired (FX and weenie hooks as events, impulses); no look-at, FX scripts or FX manager
- [ ] `CPhysicsObj::UpdateMisc` · 004e9bb0 · legions 004dc430 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::calc_cross_cells_static` · 004e9be0 — needs port: cite-only physics-object method
- [~] `CPhysicsObj::calc_cross_cells` · 004e9d30 · legions 004dc5b0 — ac2-physics Terrain::insert_object: shadow cells from the shadow box through land cells and portals; residency order and moving objects not ported
- [~] `CPhysicsObj::SetPositionInternal` · 004e9f50 · legions 004dc7d0 — no cell residency, shadow or material updates; LeaveGround FX not ported
- [ ] `CPhysicsObj::CombineAndApplyVisualDesc` · 004ea290 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::DoBehavior` · 004ea490 — ac2-physics anim_rep / motion_interp_behaviors (slice 3)
- [~] `CPhysicsObj::DoBehavior` · 004ea500 — ac2-physics do_behavior: frozen check, params mode, ModeTracker::DoBehavior; no story manager, children, FX target or cloned visual
- [ ] `CPhysicsObj::AddToTargetList` · 004ea670 · legions 004dcfa0 — needs port: cite-only physics-object method
- [~] `CPhysicsObj::UpdateMovementInternal` · 004ea780 · legions 004dd0b0 — no path-aligned heading, targeting or missiles
- [~] `CPhysicsObj::ForceIntoCell` · 004ea990 · legions 004dd2c0 — ac2-physics: FORCE_SPF sets the position only (no cell entry, shadows or cell material)
- [ ] `CPhysicsObj::recalc_cross_cells` · 004ea9f0 · legions 004dd320 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::SetPositionInternal` · 004eaa30 · legions 004dd360 — ac2-physics physics_obj_place.rs: SetPosition placement check (Legions identical)
- [-] `CPhysicsObj::~CPhysicsObj` · 004eab90 · legions 004dd4c0 — C++ constructor/destructor; Rust constructor and Drop replace it
- [ ] `CPhysicsObj::GetID` · 004ead40 · legions 004dd670 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::GetPhysicsCollideMask` · 004ead60 — needs port: cite-only physics-object method
- [-] `CPhysicsObj::SetBookie` · 004ead70 — admin/debug or text-file tooling
- [-] `CPhysicsObj::GetBookie` · 004ead80 — admin/debug or text-file tooling
- [ ] `CPhysicsObj::GetWeenieObj` · 004ead90 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::HasPhysicsBSP` · 004eada0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::HasPhysicsPresence` · 004eadb0 · legions 004dd6e0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::GetLookAtIID` · 004eade0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::GetLookAtX` · 004eae00 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::GetLookAtZ` · 004eae10 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::IsInWorld` · 004eae20 — CPhysicsObj::IsInWorld - CPhysicsObj method cited in ac2-physics body.rs, frame.rs, objects.rs, movement_module.rs, selection.rs, client_audio.rs
- [ ] `CPhysicsObj::IsInWorkspace` · 004eae30 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::GetCell` · 004eae40 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::GetFrame` · 004eae50 — needs port: cite-only physics-object method
- [-] `CPhysicsObj::get_state` · 004eae60 — state-bit flag accessor
- [-] `CPhysicsObj::is_dynamic` · 004eae70 — state-bit flag accessor
- [-] `CPhysicsObj::is_static` · 004eae80 — state-bit flag accessor
- [-] `CPhysicsObj::is_env_obj` · 004eae90 — state-bit flag accessor
- [ ] `CPhysicsObj::on_ground` · 004eaea0 · legions 004dd7d0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::off_ground` · 004eaed0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::reports_collisions` · 004eaee0 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::do_edge_sliding` · 004eaef0
- [ ] `CPhysicsObj::ignore_collisions` · 004eaf00 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::is_mobile` · 004eaf10 — ac2-physics PhysicsBody::{drawing_position, set_mobile, is_mobile}
- [x] `CPhysicsObj::is_nodraw` · 004eaf20 — CPhysicsObj::is_nodraw - CPhysicsObj method cited in ac2-physics body.rs, frame.rs, objects.rs, movement_module.rs, selection.rs, client_audio.rs
- [-] `CPhysicsObj::is_cloaked` · 004eaf30 — render plumbing
- [-] `CPhysicsObj::is_pushable` · 004eaf40 — state-bit flag accessor
- [-] `CPhysicsObj::is_path_clipped` · 004eaf60 — state-bit flag accessor
- [-] `CPhysicsObj::is_path_aligned` · 004eaf70 — state-bit flag accessor
- [-] `CPhysicsObj::is_movement_active` · 004eaf80 — state-bit flag accessor
- [-] `CPhysicsObj::is_animation_active` · 004eaf90 — state-bit flag accessor
- [-] `CPhysicsObj::is_misc_active` · 004eafa0 — state-bit flag accessor
- [-] `CPhysicsObj::is_active` · 004eafb0 · legions 004dd8e0 — state-bit flag accessor
- [-] `CPhysicsObj::is_inactive` · 004eaff0 — state-bit flag accessor
- [x] `CPhysicsObj::on_walkable` · 004eb000
- [x] `CPhysicsObj::reset_stationary_stuck_fall` · 004eb010
- [x] `CPhysicsObj::activate_stationary_fall` · 004eb020
- [x] `CPhysicsObj::activate_stationary_stop` · 004eb030
- [x] `CPhysicsObj::activate_stationary_stuck` · 004eb040
- [-] `CPhysicsObj::is_stationary_fall` · 004eb050 — state-bit flag accessor
- [-] `CPhysicsObj::is_stationary_stuck` · 004eb060 — state-bit flag accessor
- [-] `CPhysicsObj::is_stationary_stop` · 004eb070 — state-bit flag accessor
- [x] `CPhysicsObj::has_contact` · 004eb080
- [ ] `CPhysicsObj::contact_is_water` · 004eb090 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::has_sliding` · 004eb0a0 — needs port: cite-only physics-object method
- [-] `CPhysicsObj::get_contact_plane_cell_id` · 004eb0b0 — state-bit flag accessor
- [-] `CPhysicsObj::get_contact_plane` · 004eb0c0 — state-bit flag accessor
- [x] `CPhysicsObj::is_affected_by_gravity` · 004eb0d0
- [-] `CPhysicsObj::lights_on` · 004eb0e0 — render plumbing
- [-] `CPhysicsObj::is_undetectable` · 004eb0f0 — render plumbing
- [x] `CPhysicsObj::is_inelastic` · 004eb100
- [-] `CPhysicsObj::is_movement_frozen` · 004eb110 — state-bit flag accessor
- [x] `CPhysicsObj::is_animation_frozen` · 004eb120 — ac2-physics physics_obj_anim / body_anim (slice 4): PhysicsBody animation entry points
- [ ] `CPhysicsObj::SetScale` · 004eb130 — needs port: cite-only physics-object method
- [-] `CPhysicsObj::get_omega_velocity` · 004eb160 — state-bit flag accessor
- [-] `CPhysicsObj::get_omega_accleration` · 004eb170 — state-bit flag accessor
- [-] `CPhysicsObj::IsDummyObject` · 004eb180 — admin/debug or text-file tooling
- [ ] `CPhysicsObj::GetDistanceToViewer` · 004eb190 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::HasParent` · 004eb1a0 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::get_update_time` · 004eb1b0
- [ ] `CPhysicsObj::inc_update_time` · 004eb1c0 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::GetPosition` · 004eb1d0 — CPhysicsObj::GetPosition - CPhysicsObj method cited in ac2-physics body.rs, frame.rs, objects.rs, movement_module.rs, selection.rs, client_audio.rs
- [x] `CPhysicsObj::GetDrawingPosition` · 004eb1e0 — ac2-physics PhysicsBody::{drawing_position, set_mobile, is_mobile}
- [ ] `CPhysicsObj::SetTerrainType` · 004eb1f0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::GetWorldMatrix` · 004eb210 · legions 004ddb50 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::GetNumShadowObjects` · 004eb220 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::SetPhysicsCollideMask` · 004eb230 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::GetNumChildren` · 004eb250 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::GetChild` · 004eb260 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::AddToIntersection` · 004eb280 · legions 004ddbc0 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::AddToIntersection_Targeted` · 004eb320 · legions 004ddc60 — needs port: cite-only physics-object method
- [~] `CPhysicsObj::UpdateMovement` · 004eb370 — no player-distance range checks, parent or missile paths
- [ ] `CPhysicsObj::SetScatterPositionInternal` · 004eb4c0 · legions 004dde00 — needs port: cite-only physics-object method
- [-] `CPhysicsObj::CleanupMarkerObjects` · 004eb600 · legions 004ddf40 — admin/debug or text-file tooling

### PHashTable<unsigned_long,Trigger>
- [-] `PHashTable<unsigned_long,Trigger>::StreamPack` · 004eb6d0 — DBO/archive serialisation of runtime state; the client does not read it from the DAT

### PSmartArray<BehaviorParams>
- [x] `PSmartArray<BehaviorParams>::StreamPack` · 004eb8b0 · legions 004de220

### TriggerUpdate
- [x] `TriggerUpdate::TriggerUpdate` · 004eb960 · legions 004de2d0 — ac2-anim behavior/triggers + dbanimator/triggers (slice 4): effect, hook and impulse triggers through TriggerUpdate
- [-] `TriggerUpdate::~TriggerUpdate` · 004eba30 · legions 004de3a0 — C++ constructor/destructor; Rust constructor and Drop replace it

### CPhysicsObj
- [x] `CPhysicsObj::CPhysicsObj` · 004ebb50 · legions 004de4c0 — CPhysicsObj::CPhysicsObj - CPhysicsObj method cited in ac2-physics body.rs, frame.rs, objects.rs, movement_module.rs, selection.rs, client_audio.rs
- [ ] `CPhysicsObj::makeObjectInternal` · 004ec1a0 · legions 004deb10 — needs port: cite-only physics-object method
- [ ] `CPhysicsObj::makePartGroupObject` · 004ec230 · legions 004deba0 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::RecvEvt_ReleaseBehavior` · 004ec290 — ac2-client motion.rs forwards SessionEvent::ObjectAction to the PhysicsBody entry point (players and other objects)
- [~] `CPhysicsObj::GetTriggerUpdate` · 004ec310 · legions 004dec80 — the TriggerUpdate lives in the mode tracker (ac2-anim), bound lazily as ModeTracker::mTriggers
- [~] `CPhysicsObj::SetPositionInternal` · 004ec350 · legions 004decc0 — ac2-physics PhysicsBody::place: no scatter (SCATTER/RANDOMSCATTER), no SetPlacementPosition
- [~] `CPhysicsObj::HandleLeaveWorld` · 004ec400 · legions 004ded70 — the bookie part only

### _STL::map<unsigned_long,FXScalarAndTarget,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,FXScalarAndTarget>_>_>
- [-] `_STL::map<unsigned_long,FXScalarAndTarget,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,FXScalarAndTarget>_>_>::~map<unsigned_long,FXScalarAndTarget,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,FXScalarAndTarget>_>_>` · 004ec4a0 · legions 004dee10 — STL/MSVC or archive template helper; Rust collections replace it

### CPhysicsObj
- [-] `CPhysicsObj::TogglePlaceMarkerObjects` · 004ec4f0 · legions 004dee60 — admin/debug or text-file tooling
- [ ] `CPhysicsObj::makeObjectInternal` · 004ec520 · legions 004dee90 — needs port: cite-only physics-object method
- [x] `CPhysicsObj::RecvEvt_DoBehavior` · 004ec680 — ac2-client client_objects.rs apply_remote: the weenie emote gate (answer_weenie_emote); the copy with context 1 is motion.rs behavior()
- [~] `CPhysicsObj::SetPosition` · 004ec730 · legions 004df090 — ac2-physics PhysicsBody::place: no scatter (SCATTER/RANDOMSCATTER), no SetPlacementPosition
- [~] `CPhysicsObj::EnterWorld` · 004ec7d0 — ac2-client remote.rs
- [~] `CPhysicsObj::set_description` · 004ec830 — ac2-physics set_description_animation: mode, network behaviours, sliders; velocity, omega, scales, look-at, FX, missile and stamps not ported
- [ ] `CPhysicsObj::makeObject` · 004ed160 · legions 004dfac0 — needs port: cite-only physics-object method
- [~] `CPhysicsObj::SetPositionSimple` · 004ed190 · legions 004dfaf0 — flags 0x1010 into PhysicsBody::place
- [~] `CPhysicsObj::SyncPositionSimple` · 004ed1f0 · legions 004dfb50 — interp blips use set_position_checked(SLIDE); parent/placement-position bookkeeping not ported
- [ ] `CPhysicsObj::makeObject` · 004ed270 · legions 004dfbd0 — needs port: cite-only physics-object method
- [-] `CPhysicsObj::PlaceMarkerObject` · 004ed2e0 · legions 004dfc40 — admin/debug or text-file tooling
- [x] `CPhysicsObj::RecvEvt_Position` · 004ed470 — the non-player branch over PhysicsBody::interpolate_to / interpolate_and_impulse (ac2-client remote.rs)

## `portal/shared/src/physobj/physicsobjrep.cpp.c` (44/171)

### BehaviorTable
- [ ] `BehaviorTable::Get` · 004f10c0 · legions 004e39f0 — needs port: BehaviorTable::Get

### PHYSICSOBJREP
- [ ] `PHYSICSOBJREP::SetMotionInterpreter` · 004f10e0 · legions 004e3a10 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::HandleEnterWorld` · 004f1140 · legions 004e3a70 — needs port: cite-only physics-object method
- [x] `PHYSICSOBJREP::SetMode` · 004f1170 · legions 004e3aa0 — ac2-physics physics_obj_anim / body_anim (slice 4): PhysicsBody animation entry points
- [x] `PHYSICSOBJREP::DoElement` · 004f1190 · legions 004e3ac0 — ac2-physics anim_rep / motion_interp_behaviors (slice 3)
- [x] `PHYSICSOBJREP::DoFXScript` · 004f11b0 · legions 004e3ae0 — ac2-client motion.rs DoFxScript -> ac2-physics body do_fx_script; the tracker in ac2-anim fxscript_tracker.rs; client test fx_effects_tests.rs (sound and particle ids); rain: client rain_sounds.rs (tests rain_sounds_tests.rs)
- [x] `PHYSICSOBJREP::DoMode` · 004f11d0 — ac2-physics anim_rep / motion_interp_behaviors (slice 3)
- [x] `PHYSICSOBJREP::DoSlider` · 004f11f0 · legions 004e3b20 — ac2-physics physics_obj_anim / body_anim (slice 4): PhysicsBody animation entry points
- [x] `PHYSICSOBJREP::DoBehavior` · 004f12f0 · legions 004e3c20 — ac2-physics anim_rep / motion_interp_behaviors (slice 3)
- [x] `PHYSICSOBJREP::DoAutonomousBehavior` · 004f1310 — ac2-physics anim_rep / motion_interp_behaviors (slice 3)
- [x] `PHYSICSOBJREP::ReleaseBehavior` · 004f1330 · legions 004e3c60 — ac2-physics physics_obj_anim / body_anim (slice 4): PhysicsBody animation entry points
- [x] `PHYSICSOBJREP::StopBehavior` · 004f1350 — ac2-physics anim_rep / motion_interp_behaviors (slice 3)
- [x] `PHYSICSOBJREP::StopLayer` · 004f1370 · legions 004e3ca0 — ac2-physics physics_obj_anim / body_anim (slice 4): PhysicsBody animation entry points
- [x] `PHYSICSOBJREP::StopAllWeenieBehaviors` · 004f1390 — ac2-physics physics_obj_anim / body_anim (slice 4): PhysicsBody animation entry points
- [x] `PHYSICSOBJREP::StopMoveToCancelsBehaviors` · 004f13b0 · legions 004e3ce0 — ac2-physics physics_obj_anim / body_anim (slice 4): PhysicsBody animation entry points
- [ ] `PHYSICSOBJREP::DoMotion` · 004f13d0 · legions 004e3d00 — needs port: cite-only physics-object method
- [x] `PHYSICSOBJREP::GetElement` · 004f1420 · legions 004e3d50 — ac2-physics anim_rep / motion_interp_behaviors (slice 3)
- [x] `PHYSICSOBJREP::GetDesiredElement` · 004f1440 · legions 004e3d70 — ac2-physics anim_rep / motion_interp_behaviors (slice 3)
- [x] `PHYSICSOBJREP::GetMode` · 004f1460 · legions 004e3d90 — ac2-physics anim_rep / motion_interp_behaviors (slice 3)
- [x] `PHYSICSOBJREP::IsDoingBehavior` · 004f1480 — ac2-physics anim_rep / motion_interp_behaviors (slice 3)
- [x] `PHYSICSOBJREP::GetSliderCurrentValue` · 004f14a0 — ac2-physics physics_obj_anim / body_anim (slice 4): PhysicsBody animation entry points
- [x] `PHYSICSOBJREP::GetSliderDesiredValue` · 004f14c0 — ac2-physics physics_obj_anim / body_anim (slice 4): PhysicsBody animation entry points
- [ ] `PHYSICSOBJREP::GetMotionInterpreter` · 004f14e0 · legions 004e3e30 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetModeTracker` · 004f14f0 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetSliderTracker` · 004f1500 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetIID` · 004f1510 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetSeaFloorMultiplier` · 004f1540 · legions 004e3e70 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetWaterMultiplier` · 004f1560 · legions 004e3e90 — needs port: cite-only physics-object method
- [x] `PHYSICSOBJREP::MoveToEvent` · 004f1580 · legions 004e3eb0 — ac2-physics physics_obj_anim / body_anim (slice 4): PhysicsBody animation entry points
- [x] `PHYSICSOBJREP::InterruptMoveTo` · 004f15a0 · legions 004e3ed0 — ac2-physics physics_obj_anim / body_anim (slice 4): PhysicsBody animation entry points
- [ ] `PHYSICSOBJREP::HandleBehaviorDone` · 004f15c0 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::HandleReleaseDone` · 004f15d0 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::HandleMoveToCallback` · 004f15e0 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::IsMovingTo` · 004f15f0 · legions 004e3f20 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::IsTurningTo` · 004f1610 · legions 004e3f40 — needs port: cite-only physics-object method
- [x] `PHYSICSOBJREP::SetGlobalModifier` · 004f1630 — ac2-physics body.rs set_global_modifier -> motion_interp.rs global_mod (keys the motion table lookup); ac2-client motion.rs apply; test motion_interp_tests a_global_modifier_selects_its_motion_values
- [ ] `PHYSICSOBJREP::AddTerrainModifier` · 004f1650 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::ClearTerrainModifiers` · 004f1670 — needs port: cite-only physics-object method
- [x] `PHYSICSOBJREP::SetMIDESC` · 004f1690 · legions 004e3fc0 — SetMIDESC: loader thread reads the MotionInterpDesc (Request::MotionDesc, world_data motion_desc); client_objects request_motion_desc / motion_desc_loaded swap the body's table (MotionInterp::set_desc); missing DBObj keeps the table
- [x] `PHYSICSOBJREP::SetHoverHeight` · 004f16b0 · legions 004e3fe0 — PHYSICSOBJREP::SetHoverHeight - PHYSICSOBJREP method cited in ac2-physics motion_interp.rs, body.rs, ac2-assets visual particles, loader_anim.rs
- [ ] `PHYSICSOBJREP::ResetHoverHeight` · 004f16d0 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetHoverHeight` · 004f16f0 · legions 004e4060 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetDefaultHoverHeight` · 004f1710 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetVelocityScale` · 004f1730 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetAccelerationScale` · 004f1750 — needs port: cite-only physics-object method
- [x] `PHYSICSOBJREP::SetVelocityScale` · 004f1770 · legions 004e40a0 — PHYSICSOBJREP::SetVelocityScale - PHYSICSOBJREP method cited in ac2-physics motion_interp.rs, body.rs, ac2-assets visual particles, loader_anim.rs
- [ ] `PHYSICSOBJREP::SetAccelerationScale` · 004f1790 · legions 004e40c0 — needs port: cite-only physics-object method
- [x] `PHYSICSOBJREP::HitGround` · 004f17b0 · legions 004e40e0 — ac2-physics anim_rep / motion_interp_behaviors (slice 3)
- [x] `PHYSICSOBJREP::LeaveGround` · 004f17c0 · legions 004e40f0 — ac2-physics anim_rep / motion_interp_behaviors (slice 3)
- [ ] `PHYSICSOBJREP::RestoreCamera` · 004f17d0 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::SetCameraAnimParent` · 004f17e0 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::SetCameraAnimTarget` · 004f17f0 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::SetTargetedCamera` · 004f1800 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::DoBehaviorOnObject` · 004f1830 — needs port: cite-only physics-object method
- [-] `PHYSICSOBJREP::SetChildrenOpacity` · 004f1860 — render plumbing
- [ ] `PHYSICSOBJREP::DoSliderOnChild` · 004f1870 · legions 004e41a0 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetPosition` · 004f1880 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetOmegaVelocity` · 004f18a0 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetHeightAboveWalkable` · 004f18f0 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetDepth` · 004f1910 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::HasWater` · 004f1930 · legions 004e4260 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetCell` · 004f1950 · legions 004e4280 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetVectorHeading` · 004f1970 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetHeading` · 004f19d0 · legions 004e4300 — needs port: cite-only physics-object method
- [~] `PHYSICSOBJREP::GetTriggerUpdate` · 004f1a00 — the TriggerUpdate lives in the mode tracker (ac2-anim), bound lazily as ModeTracker::mTriggers
- [x] `PHYSICSOBJREP::GetConstraintFactor` · 004f1a20 — ac2-physics constraint.rs (slice 4): ConstraintManager, PhysicsBody::constrain_to/unconstrain, the factor in ConvertDoMotionToLocalDesired, DoConstraint's enable
- [ ] `PHYSICSOBJREP::GetLocalVelocity` · 004f1a40 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetTransitionCVel` · 004f1a70 · legions 004e43a0 — needs port: cite-only physics-object method
- [x] `PHYSICSOBJREP::DoAnimation` · 004f1ad0 — ac2-anim dbanimator / ac2-physics anim_pose (slice 3)
- [x] `PHYSICSOBJREP::GetComboCVel` · 004f1b00 — ac2-physics anim_rep / motion_interp_behaviors (slice 3)
- [x] `PHYSICSOBJREP::GetHoldLocations` · 004f1b90 — ac2-anim dbanimator / ac2-physics anim_pose (slice 3)
- [~] `PHYSICSOBJREP::SetupSliderTracker` · 004f1c40 · legions 004e4540 — ac2-physics anim_rep: no look-at, FX scripts, triggers, wind slider or AnimInfo hold loops; DoFX is an event; one part group
- [x] `PHYSICSOBJREP::SetupFXScriptTracker` · 004f1cb0 · legions 004e45b0 — ac2-client motion.rs DoFxScript -> ac2-physics body do_fx_script; the tracker in ac2-anim fxscript_tracker.rs; client test fx_effects_tests.rs (sound and particle ids); rain: client rain_sounds.rs (tests rain_sounds_tests.rs)
- [x] `PHYSICSOBJREP::SetupModeTracker` · 004f1d10 · legions 004e4610 — ac2-physics anim_rep / motion_interp_behaviors (slice 3)
- [ ] `PHYSICSOBJREP::SetTerrainType` · 004f1d70 · legions 004e4670 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetBoneTypeMatrix` · 004f1da0 · legions 004e46a0 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::HasBoneType` · 004f1db0 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::HasHoldingLocation` · 004f1dd0 · legions 004e46d0 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetHoldingLocation` · 004f1de0 · legions 004e46e0 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetPartGroup` · 004f1df0 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetNumSphere` · 004f1e00 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetSphere` · 004f1e10 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetNumCylsphere` · 004f1e20 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetCylsphere` · 004f1e30 · legions 004e46f0 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetRadius` · 004f1e40 · legions 004e4740 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetHeight` · 004f1e50 · legions 004e4750 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::AllowsFreeHeading` · 004f1e60 · legions 004e4780 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetStepUpHeight` · 004f1e70 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetStepDownHeight` · 004f1e80 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetBoundingSphere` · 004f1e90 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetSelectionSphere` · 004f1ea0 · legions 004e47a0 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::SetStepDownHeight` · 004f1ef0 · legions 004e4810 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::SetBoundingObject` · 004f1f00 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetPhysicsBoundingBox` · 004f1f10 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetPhysicsContainmentRadius` · 004f1f20 · legions 004e4820 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::CacheHasPhysicsBSP` · 004f1f30 · legions 004e4830 — needs port: cite-only physics-object method
- [x] `PHYSICSOBJREP::FindObjCollisions` · 004f1f40 · legions 004e4840 — ac2-physics objects.rs: static-object collision dispatch, containment radius, shadow box
- [ ] `PHYSICSOBJREP::SetCellID` · 004f1f50 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetEffectsScale` · 004f1f60 — needs port: cite-only physics-object method
- [x] `PHYSICSOBJREP::GetVdescScale` · 004f1f90 — ac2-physics objects.rs: static-object collision dispatch, containment radius, shadow box
- [ ] `PHYSICSOBJREP::SetPlacementFrame` · 004f1fb0 · legions 004e48b0 — needs port: cite-only physics-object method
- [-] `PHYSICSOBJREP::Draw` · 004f1fe0 · legions 004e48e0 — render plumbing
- [ ] `PHYSICSOBJREP::StartStreakInternal` · 004f20f0 · legions 004e49f0 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::StopStreakInternal` · 004f2100 · legions 004e4a00 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::StartStreak` · 004f2110 · legions 004e4a10 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::StopStreak` · 004f2140 · legions 004e4a40 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::DoAppearanceFX` · 004f2170 · legions 004e4a70 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::RemoveAppearanceFX` · 004f2180 · legions 004e4a80 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::CreateStreakEffect` · 004f2190 · legions 004e4a90 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::CreateParticleRepulsor` · 004f21a0 · legions 004e4aa0 — needs port: cite-only physics-object method
- [x] `PHYSICSOBJREP::CreateParticleSystem` · 004f21b0 · legions 004e4ab0 — PHYSICSOBJREP::CreateParticleSystem - PHYSICSOBJREP method cited in ac2-physics motion_interp.rs, body.rs, ac2-assets visual particles, loader_anim.rs
- [x] `PHYSICSOBJREP::CreateParticleSystem` · 004f22c0 · legions 004e4bc0 — PHYSICSOBJREP::CreateParticleSystem - PHYSICSOBJREP method cited in ac2-physics motion_interp.rs, body.rs, ac2-assets visual particles, loader_anim.rs
- [ ] `PHYSICSOBJREP::DestroyParticleSystem` · 004f2410 · legions 004e4d10 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::UpdateMovement` · 004f2420 · legions 004e4d20 — needs port: cite-only physics-object method
- [~] `PHYSICSOBJREP::UpdateSliderTracker` · 004f2440 · legions 004e4d40 — ac2-physics anim_rep update_sliders: wind slider 0x4000000B to windmag over 1 s; colour/opacity read back; FXScript tracker separate
- [x] `PHYSICSOBJREP::UpdateFXScriptTracker` · 004f2540 · legions 004e4e40 — ac2-client motion.rs DoFxScript -> ac2-physics body do_fx_script; the tracker in ac2-anim fxscript_tracker.rs; client test fx_effects_tests.rs (sound and particle ids); rain: client rain_sounds.rs (tests rain_sounds_tests.rs)
- [x] `PHYSICSOBJREP::UpdateAnimation` · 004f2560 · legions 004e4e60 — ac2-physics anim_rep / motion_interp_behaviors (slice 3)
- [ ] `PHYSICSOBJREP::GetTerrainType` · 004f2590 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::HasContact` · 004f25b0 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::IsSlidingDown` · 004f25d0 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetContactPlane` · 004f25e0 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetAnimVelocity` · 004f2620 — needs port: cite-only physics-object method
- [x] `PHYSICSOBJREP::IsThePlayer` · 004f2670 — ac2-physics anim_rep / motion_interp_behaviors (slice 3)
- [~] `PHYSICSOBJREP::DoFX` · 004f2690 — ac2-physics anim_rep: no look-at, FX scripts, triggers, wind slider or AnimInfo hold loops; DoFX is an event; one part group
- [ ] `PHYSICSOBJREP::DoTargettedFX` · 004f26c0 · legions 004e4fc0 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetVDescToClone` · 004f26f0 · legions 004e4ff0 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetClonedAppearanceID` · 004f2710 · legions 004e5010 — needs port: cite-only physics-object method
- [x] `PHYSICSOBJREP::StopFX` · 004f2730 — PHYSICSOBJREP::StopFX - PHYSICSOBJREP method cited in ac2-physics motion_interp.rs, body.rs, ac2-assets visual particles, loader_anim.rs
- [ ] `PHYSICSOBJREP::UpdateFX` · 004f2750 · legions 004e5050 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetFXManager` · 004f2770 · legions 004e5070 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::UnlockActions` · 004f2780 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::LockActions` · 004f2790 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::DeleteSelf` · 004f27a0 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::ClearFXTarget` · 004f27d0 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::SetFXTarget` · 004f27e0 · legions 004e50e0 — needs port: cite-only physics-object method
- [-] `PHYSICSOBJREP::CacheAnimations` · 004f2800 · legions 004e5100 — render plumbing

### SmartArray<Ray>
- [-] `SmartArray<Ray>::~SmartArray<Ray>` · 004f2810 · legions 005ae4c0 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### SliderInterpolator
- [-] `SliderInterpolator::SliderInterpolator` · 004f2830 · legions 004e5110 — C++ constructor/destructor; Rust constructor and Drop replace it

### PHYSICSOBJREP
- [ ] `PHYSICSOBJREP::GetModeState` · 004f2860 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetDesiredModeState` · 004f2880 · legions 004e5160 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetAcceleration` · 004f28a0 · legions 004e5220 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetDesiredVelocity` · 004f28f0 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetOmegaAcceleration` · 004f2940 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::AlwaysPrepareObject` · 004f2990 · legions 004e5270 — needs port: cite-only physics-object method

### TransformBase
- [-] `TransformBase::TransformBase` · 004f2a00 · legions 004e52e0 — C++ constructor/destructor; Rust constructor and Drop replace it

### SmartArray<LightSourceInstance>
- [-] `SmartArray<LightSourceInstance>::RemoveOrderedByIndex` · 004f2ad0 · legions 004e53b0 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### PHYSICSOBJREP
- [ ] `PHYSICSOBJREP::GetSliderInterpolator` · 004f2c30 · legions 004e5420 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetMotionValuesForElement` · 004f2cc0 · legions 004e54b0 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetMotionValues` · 004f2d20 · legions 004e5510 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetDesiredMotionValues` · 004f2d70 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::GetStandardMotionValues` · 004f2df0 — needs port: cite-only physics-object method
- [~] `PHYSICSOBJREP::PrepareObjectForDrawing` · 004f2e70 · legions 004e5660 — ac2-render scene/object_lights.rs place: light fade in/out and lifespan (opacity), origin = object frame times offset, spent lights dropped; object particle systems follow the object's frame (scene/particles.rs place_objects); cell and water-plane placement not ported
- [x] `PHYSICSOBJREP::CreateParticleSystem` · 004f3280 · legions 004e5a40 — PHYSICSOBJREP::CreateParticleSystem - PHYSICSOBJREP method cited in ac2-physics motion_interp.rs, body.rs, ac2-assets visual particles, loader_anim.rs
- [ ] `PHYSICSOBJREP::GetBaseSetupName` · 004f3460 — needs port: cite-only physics-object method
- [~] `PHYSICSOBJREP::ApplyStartupFX` · 004f3510 · legions 004e5d70 — ac2-client client_audio.rs start_startup_lights: each object's startup FX place their lights once at creation (startup_light_starts, LightSourceEffects gated); startup sounds, streaks, decals, repulsors and vdesc objects of the object branch, and the shared rep's particle branch, are not ported here
- [ ] `PHYSICSOBJREP::InitInternal` · 004f35f0 · legions 004e5ee0 — needs port: cite-only physics-object method
- [~] `PHYSICSOBJREP::SetupCompositor` · 004f36a0 · legions 004e5f90 — impulse triggers only (effect triggers and weenie hooks not fired); no slider-driven layer weights; one part group

### HashTable<ModeStateKey,int>
- [-] `HashTable<ModeStateKey,int>::operator=` · 004f3800 · legions 0063bed0 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### PHYSICSOBJREP
- [-] `PHYSICSOBJREP::~PHYSICSOBJREP` · 004f3a00 · legions 004e6240 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `PHYSICSOBJREP::Release` · 004f3ad0 · legions 004e6310 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it

### PHashTable<ModeStateKey,int>
- [-] `PHashTable<ModeStateKey,int>::~PHashTable<ModeStateKey,int>` · 004f3b10 · legions 004e6350 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it
- [-] `PHashTable<ModeStateKey,int>::StreamPack` · 004f3c60 — DBO/archive serialisation of runtime state; the client does not read it from the DAT

### PHYSICSOBJREP
- [-] `PHYSICSOBJREP::PHYSICSOBJREP` · 004f3f60 · legions 004e67a0 — C++ constructor/destructor; Rust constructor and Drop replace it
- [ ] `PHYSICSOBJREP::GetClonedAppKeyHash` · 004f4020 · legions 004e6860 — needs port: cite-only physics-object method
- [x] `PHYSICSOBJREP::InitFXManager` · 004f4090 · legions 004e68d0 — PHYSICSOBJREP::InitFXManager - PHYSICSOBJREP method cited in ac2-physics motion_interp.rs, body.rs, ac2-assets visual particles, loader_anim.rs
- [x] `PHYSICSOBJREP::InitVisualDescObject` · 004f4240 · legions 004e6a80 — PHYSICSOBJREP::InitVisualDescObject - PHYSICSOBJREP method cited in ac2-physics motion_interp.rs, body.rs, ac2-assets visual particles, loader_anim.rs
- [ ] `PHYSICSOBJREP::CreateVisualDesc` · 004f4300 · legions 004e6b40 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::CompareAndModify` · 004f4360 · legions 004e6ba0 — needs port: cite-only physics-object method
- [x] `PHYSICSOBJREP::InitVisualDescObject` · 004f4400 · legions 004e6c40 — PHYSICSOBJREP::InitVisualDescObject - PHYSICSOBJREP method cited in ac2-physics motion_interp.rs, body.rs, ac2-assets visual particles, loader_anim.rs
- [ ] `PHYSICSOBJREP::InitDataIDObject` · 004f44c0 · legions 004e6ce0 — needs port: cite-only physics-object method
- [ ] `PHYSICSOBJREP::CreateDataID` · 004f4510 · legions 004e6d30 — needs port: cite-only physics-object method

## `portal/shared/src/physobj/physicspart.cpp.c` (4/11)

### CPhysicsPart
- [-] `CPhysicsPart::CPhysicsPart` · 004f5c30 · legions 004e8420 — C++ constructor/destructor; Rust constructor and Drop replace it
- [ ] `CPhysicsPart::HasMatrixWeights` · 004f5c90 · legions 004e8480 — needs port: cite-only physics-object method
- [-] `CPhysicsPart::BuildRenderBoundingBox` · 004f5cb0 · legions 004e84a0 — render plumbing
- [-] `CPhysicsPart::BuildDefaultRenderBoundingBox` · 004f5e40 · legions 004e8630 — render plumbing
- [ ] `CPhysicsPart::HasPhysicsBSP` · 004f5fd0 · legions 004e87c0 — needs port: cite-only physics-object method
- [ ] `CPhysicsPart::SetMesh` · 004f5ff0 · legions 004e87e0 — needs port: cite-only physics-object method
- [x] `CPhysicsPart::RefreshMatrix` · 004f6010 · legions 004e8800 — ac2-gpu ObjectInstance.pose (part matrices, bone storage buffer)
- [x] `CPhysicsPart::DrawPart` · 004f6030 · legions 004e8820 — ac2-gpu ObjectInstance.pose (part matrices, bone storage buffer)
- [x] `CPhysicsPart::FindObjCollisions` · 004f6090 · legions 004e8880 — ac2-physics objects.rs: static-object collision dispatch, containment radius, shadow box
- [ ] `CPhysicsPart::makePhysicsPart` · 004f6110 · legions 004e8900 — needs port: cite-only physics-object method
- [x] `CPhysicsPart::TraceRayPhysics` · 004f6160 · legions 004e8950 — ac2-physics los (Terrain::los_check)

## `portal/shared/src/physobj/setpositiondefs.cpp.c` (0/12)

### SetPositionStruct
- [-] `SetPositionStruct::SetPositionStruct` · 004edcd0 · legions 004e0640 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `SetPositionStruct::~SetPositionStruct` · 004edd20 · legions 004e0690 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `SetPositionStruct::GetPackSize` · 004edd30 — DBO/archive serialisation of runtime state; the client does not read it from the DAT
- [-] `SetPositionStruct::Pack` · 004edd40 — DBO/archive serialisation of runtime state; the client does not read it from the DAT
- [-] `SetPositionStruct::UnPack` · 004eddd0 — DBO/archive serialisation of runtime state
- [-] `SetPositionStruct::AddFlags` · 004ede70 · legions 004e07e0 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `SetPositionStruct::RemoveFlags` · 004ede80 · legions 004e07f0 — state-bit flag accessor; Rust struct field replaces it
- [-] `SetPositionStruct::SetScatterRadius` · 004edea0 · legions 004e0810 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `SetPositionStruct::SetNumTries` · 004eded0 · legions 004e0840 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `SetPositionStruct::SetLine` · 004edee0 · legions 004e0850 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `SetPositionStruct::SetLineByPos` · 004edf70 · legions 004e08e0 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `SetPositionStruct::SetPosition` · 004edfa0 · legions 004e0910 — plain field accessor on a runtime record; Rust struct field replaces it

## `portal/shared/src/physobj/story/fxandbehaviorinfo.cpp.c` (0/9)

### FrillRender
- [-] `FrillRender::SetOutsideRadius` · 006acf90 · legions 00646200 — render plumbing

### FXAndBehaviorInfo
- [-] `FXAndBehaviorInfo::GetWeaponBehavior` · 006acfa0 · legions 00646210 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `FXAndBehaviorInfo::StreamPack` · 006acfb0 · legions 00646220 — DBO/archive serialisation of runtime state; the client does not read it from the DAT

### PhysicsStory
- [-] `PhysicsStory::SetAttackerBehavior` · 006ad040 · legions 006462b0 — plain field accessor on a runtime record; Rust struct field replaces it

### FXAndBehaviorInfo
- [-] `FXAndBehaviorInfo::SetWeaponBehavior` · 006ad050 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `FXAndBehaviorInfo::~FXAndBehaviorInfo` · 006ad060 · legions 006462c0 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `FXAndBehaviorInfo::operator=` · 006ad110 · legions 00646370 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [-] `FXAndBehaviorInfo::FXAndBehaviorInfo` · 006ad150 · legions 006463b0 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `FXAndBehaviorInfo::FXAndBehaviorInfo` · 006ad1a0 · legions 00646400 — C++ constructor/destructor; Rust constructor and Drop replace it

## `portal/shared/src/physobj/story/missileinfo.cpp.c` (0/17)

### MissileInfo
- [-] `MissileInfo::MissileInfo` · 006ad200 · legions 00646460 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `MissileInfo::~MissileInfo` · 006ad250 · legions 006464b0 — C++ constructor/destructor; Rust constructor and Drop replace it

### FrillRender
- [-] `FrillRender::SetInsideRadius` · 006ad280 · legions 006464e0 — render plumbing

### MissileInfo
- [-] `MissileInfo::SetThrown` · 006ad290 · legions 006464f0 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [-] `MissileInfo::GetTargetingParameters` · 006ad2a0 · legions 00646500 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [-] `MissileInfo::IsThrown` · 006ad2b0 · legions 00646520 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [-] `MissileInfo::GetCollisionParameters` · 006ad2c0 · legions 00646530 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [-] `MissileInfo::SetClone` · 006ad2d0 · legions 00646540 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [-] `MissileInfo::ShouldClone` · 006ad2e0 · legions 00646550 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [-] `MissileInfo::StreamPack` · 006ad340 · legions 006465b0 — DBO/archive serialisation of runtime state; the client does not read it from the DAT

### MissileParameters
- [-] `MissileParameters::operator=` · 006ad400 · legions 00646670 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it

### MissileInfo
- [-] `MissileInfo::SetMissileParameters` · 006ad510 · legions 00646780 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [-] `MissileInfo::SetTargetingParameters` · 006ad520 · legions 00646790 — plain missile parameter field; ac2-script glue/missile.rs holds the values

### CollisionParameters
- [-] `CollisionParameters::operator=` · 006ad530 · legions 006467a0 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it

### MissileInfo
- [-] `MissileInfo::SetCollisionParameters` · 006ad5c0 · legions 00646830 — plain missile parameter field; ac2-script glue/missile.rs holds the values
- [-] `MissileInfo::operator=` · 006ad5d0 · legions 00646840 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [-] `MissileInfo::MissileInfo` · 006ad640 · legions 006468b0 — C++ constructor/destructor; Rust constructor and Drop replace it

## `portal/shared/src/physobj/story/physicsstory.cpp.c` (2/35)

### StoryHookData
- [-] `StoryHookData::StoryHookData` · 006ad680 · legions 006468f0 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `StoryHookData::Clear` · 006ad6d0 · legions 00646940 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `StoryHookData::get_pack_bitfield` · 006ad740 · legions 006469b0 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [-] `StoryHookData::SetWeaponID` · 006ad7d0 · legions 00646a40 — plain field accessor on a runtime record; Rust struct field replaces it

### ParticleSystem
- [x] `ParticleSystem::SetNoFade` · 006ad7f0 · legions 00646a60 — ac2-render particles ParticleSystem::set_no_fade

### StoryHookData
- [-] `StoryHookData::SetTargetPKDamageChange` · 006ad800 · legions 00646a70 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `StoryHookData::SetAttackResult` · 006ad810 · legions 00646a80 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `StoryHookData::SetAnimHookNumber` · 006ad820 · legions 00646a90 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `StoryHookData::SetAttackHookNumber` · 006ad830 · legions 00646aa0 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `StoryHookData::SetTypeToDamage` · 006ad840 · legions 00646ab0 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `StoryHookData::SetTypeToCreateMissiles` · 006ad850 · legions 00646ac0 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `StoryHookData::SetTypeToFireMissile` · 006ad860 · legions 00646ad0 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `StoryHookData::GetWeaponID` · 006ad870 · legions 00646b00 — plain field accessor on a runtime record; Rust struct field replaces it

### CWBookKeeper
- [-] `CWBookKeeper::GetBookie` · 006ad890 · legions 00646b20 — admin/debug or text-file tooling

### PhysicsStory
- [-] `PhysicsStory::get_pack_bitfield` · 006ad8a0 · legions 00646b30 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it

### CMotionInterp
- [x] `CMotionInterp::SetGlobalModifier` · 006ad900 · legions 00646b90 — ac2-physics body.rs set_global_modifier -> motion_interp.rs global_mod (keys the motion table lookup); ac2-client motion.rs apply; test motion_interp_tests a_global_modifier_selects_its_motion_values

### PhysicsStory
- [-] `PhysicsStory::SetSkillID` · 006ad910 · legions 00646ba0 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `PhysicsStory::SetBasicAttack` · 006ad920 · legions 00646bb0 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `PhysicsStory::GetHooks` · 006ad930 · legions 00646bc0 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `PhysicsStory::GetClientAttackContext` · 006ad940 · legions 00646be0 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `PhysicsStory::GetSkillID` · 006ad950 · legions 00646bf0 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `PhysicsStory::IsBasicAttack` · 006ad960 · legions 00646c00 — plain field accessor on a runtime record; Rust struct field replaces it

### StoryHookData
- [-] `StoryHookData::GetTargetID` · 006ad970 · legions 00646ae0 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `StoryHookData::StoryHookData` · 006ad990 · legions 00646c10 — C++ constructor/destructor; Rust constructor and Drop replace it
- [~] `StoryHookData::StreamPack` · 006ad9f0 · legions 00646c70 — unpack only, in ac2-net protocol::movement (Legions instruction-identical)

### PhysicsStory
- [-] `PhysicsStory::IsNull` · 006adb60 · legions 00646de0 — plain field accessor on a runtime record; Rust struct field replaces it
- [-] `PhysicsStory::GetOverallAttackResult` · 006adb80 · legions 00646e00 — plain field accessor on a runtime record; Rust struct field replaces it

### _STL::_Rb_tree<StoryHookData,StoryHookData,_STL::_Identity<StoryHookData>,_STL::less<StoryHookData>,_STL::allocator<StoryHookData>_>
- [-] `_STL::_Rb_tree<StoryHookData,StoryHookData,_STL::_Identity<StoryHookData>,_STL::less<StoryHookData>,_STL::allocator<StoryHookData>_>::operator=` · 006ade30 · legions 006470c0 — STL/MSVC or archive template helper; Rust collections replace it

### PhysicsStory
- [-] `PhysicsStory::~PhysicsStory` · 006ae140 · legions 006473d0 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `PhysicsStory::operator=` · 006ae1e0 · legions 00647470 — C++ assignment/allocation/teardown plumbing; Rust ownership replaces it
- [-] `PhysicsStory::AddHook` · 006ae240 · legions 006474d0 — plain field accessor on a runtime record; Rust struct field replaces it

### (free functions)
- [~] `STREAMPACK_STL_SET` · 006ae260 · legions 006474f0 — unpack only, in ac2-net protocol::movement (Legions instruction-identical)

### PhysicsStory
- [-] `PhysicsStory::PhysicsStory` · 006ae420 · legions 006476b0 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `PhysicsStory::PhysicsStory` · 006ae4d0 · legions 00647760 — C++ constructor/destructor; Rust constructor and Drop replace it
- [~] `PhysicsStory::StreamPack` · 006ae550 · legions 006477e0 — unpack only, in ac2-net protocol::movement (Legions instruction-identical)

## `portal/shared/src/physobj/story/storymanager.cpp.c` (2/15)

### StoryManager
- [ ] `StoryManager::CreateMissile` · 005148d0 · legions 005028e0 — needs port: story missile creation

### SmartArray<InstanceID>
- [-] `SmartArray<InstanceID>::RemoveOrderedByIndex` · 005149b0 · legions 005029c0 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### IntrusiveSmartPointer<SmartPointerData<PhysicsStory>_>
- [-] `IntrusiveSmartPointer<SmartPointerData<PhysicsStory>_>::create` · 00514a30 · legions 00502a40 — STL/SmartArray/HashTable container internals; Rust Vec/HashMap replaces it

### StoryManager
- [ ] `StoryManager::FireMissile` · 00514a90 · legions 00502aa0 — needs port: story missile creation
- [ ] `StoryManager::CleanupStory` · 00514b90 · legions 00502ba0 — needs port: story hook dispatch
- [-] `StoryManager::StoryManager` · 00514ca0 · legions 00502cb0 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `StoryManager::~StoryManager` · 00514cd0 · legions 00502ce0 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `StoryManager::StreamPack` · 00514d20 — DBO/archive serialisation of runtime state; the client does not read it from the DAT
- [ ] `StoryManager::CreateAndParentMissiles` · 00514dd0 · legions 00502de0 — needs port: story missile creation
- [x] `StoryManager::DoStoryHook` · 00514ff0 · legions 00503000 — StoryManager::DoStoryHook - StoryManager hook cited in ac2-client client_story.rs and ac2-anim behavior/mod.rs
- [ ] `StoryManager::DoCurrentStoryHooks` · 005152e0 · legions 005032f0 — needs port: story hook dispatch
- [ ] `StoryManager::FinishStory` · 00515370 · legions 00503380 — needs port: story hook dispatch
- [ ] `StoryManager::DoStory` · 005153b0 · legions 005033c0 — needs port: story hook dispatch
- [x] `StoryManager::HandleAnimHook` · 005154b0 · legions 005034c0 — StoryManager::HandleAnimHook - StoryManager hook cited in ac2-client client_story.rs and ac2-anim behavior/mod.rs
- [ ] `StoryManager::HandleBehaviorDone` · 005154f0 · legions 00503500 — needs port: story hook dispatch

## `portal/shared/src/physobj/target/targetmanager.cpp.c` (0/25)

### TargetVoyeur
- [-] `TargetVoyeur::TargetVoyeur` · 00515520 · legions 00503530 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `TargetVoyeur::GetPackSize` · 00515580 · legions 00503590 — DBO/archive serialisation of runtime state; the client does not read it from the DAT
- [-] `TargetVoyeur::Pack` · 005155a0 · legions 005035b0 — DBO/archive serialisation of runtime state; the client does not read it from the DAT
- [ ] `TargetVoyeur::UnPack` · 00515600 · legions 00503610 — needs port: target tracking and voyeurs

### TargetManager
- [ ] `TargetManager::GetInterpolatedPosition` · 00515690 · legions 005036a0 — needs port: target tracking and voyeurs

### TargetInfo
- [-] `TargetInfo::TargetInfo` · 00515710 · legions 00503720 — C++ constructor/destructor; Rust constructor and Drop replace it
- [-] `TargetInfo::GetPackSize` · 005157e0 — DBO/archive serialisation of runtime state; the client does not read it from the DAT
- [-] `TargetInfo::Pack` · 00515820 · legions 00503830 — DBO/archive serialisation of runtime state; the client does not read it from the DAT
- [-] `TargetInfo::UnPack` · 00515950 — DBO/archive serialisation of runtime state

### TargetManager
- [ ] `TargetManager::ClearTarget` · 00515ab0 · legions 00503ac0 — needs port: target tracking and voyeurs
- [ ] `TargetManager::SetTargetQuantum` · 00515b00 · legions 00503b10 — needs port: target tracking and voyeurs
- [~] `TargetManager::ReceiveUpdate` · 00515b60 · legions 00503b70 — ac2-client track_move_to_target: the player's move-to object target's position, radius and height each physics quantum; lost when it leaves the world; no voyeur update thresholds or interpolation
- [ ] `TargetManager::SendVoyeurUpdate` · 00515d50 · legions 00503d60 — needs port: target tracking and voyeurs
- [ ] `TargetManager::CheckAndUpdateVoyeur` · 00515e90 · legions 00503ea0 — needs port: target tracking and voyeurs
- [ ] `TargetManager::SetTarget` · 00515f70 · legions 00503f80 — needs port: target tracking and voyeurs

### TargetVoyeur
- [-] `TargetVoyeur::TargetVoyeur` · 005160e0 · legions 005040f0 — C++ constructor/destructor; Rust constructor and Drop replace it

### TargetManager
- [~] `TargetManager::HandleTargetting` · 005161e0 · legions 005041f0 — ac2-client track_move_to_target: the player's move-to object target's position, radius and height each physics quantum; lost when it leaves the world; no voyeur update thresholds or interpolation
- [ ] `TargetManager::NotifyVoyeurOfEvent` · 005162b0 · legions 005042c0 — needs port: target tracking and voyeurs
- [~] `TargetManager::HandleLeaveWorld` · 00516330 · legions 00504340 — ac2-client track_move_to_target: the player's move-to object target's position, radius and height each physics quantum; lost when it leaves the world; no voyeur update thresholds or interpolation
- [ ] `TargetManager::RemoveVoyeur` · 005163d0 · legions 005043e0 — needs port: target tracking and voyeurs
- [-] `TargetManager::~TargetManager` · 00516580 · legions 00504590 — C++ constructor/destructor; Rust constructor and Drop replace it
- [ ] `TargetManager::AddVoyeur` · 005165e0 · legions 005045f0 — needs port: target tracking and voyeurs

### PHashTable<InstanceID,TargetVoyeur>
- [-] `PHashTable<InstanceID,TargetVoyeur>::StreamPack` · 005166a0 — DBO/archive serialisation of runtime state; the client does not read it from the DAT

### TargetVoyeur
- [-] `TargetVoyeur::TargetVoyeur` · 005168e0 · legions 005048f0 — C++ constructor/destructor; Rust constructor and Drop replace it

### TargetManager
- [-] `TargetManager::TargetManager` · 00516940 · legions 00504970 — C++ constructor/destructor; Rust constructor and Drop replace it
