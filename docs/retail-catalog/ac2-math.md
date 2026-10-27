# ac2-math: Math and geometry

426 retail functions: 149 done, 11 partial, 127 to do, 139 skipped.
Legend and workflow: [README](README.md).

## `portal/shared/src/geometry/aframe.cpp.c` (2/5)

### AFrame
- [ ] `AFrame::Pack` · 0062ac90 · legions 005e4d70 — needs port (check): implemented as ac2-math frame.rs AFrame::to_packed_bytes, uncited
- [x] `AFrame::IsValid` · 0062acf0 · legions 005e4dd0
- [x] `AFrame::UnPack` · 0062ae10 · legions 005e4ef0
- [-] `AFrame::FromFileNode` · 0062ae80 — text source format (admin tools)
- [-] `AFrame::ToFileNode` · 0062afd0 — text source format (admin tools)

## `portal/shared/src/geometry/animassetkey.cpp.c` (0/13)

### AnimAssetScaleKey
- [-] `AnimAssetScaleKey::AnimAssetScaleKey` · 007365a0 · legions 006b2110 — ctor/dtor: Rust struct construction and Drop

### AnimAssetOffsetKey
- [-] `AnimAssetOffsetKey::AnimAssetOffsetKey` · 007365d0 · legions 006b2140 — ctor/dtor: Rust struct construction and Drop

### AnimAssetQuatKey
- [-] `AnimAssetQuatKey::AnimAssetQuatKey` · 00736600 · legions 006b2170 — ctor/dtor: Rust struct construction and Drop

### AnimAssetWeightKey
- [-] `AnimAssetWeightKey::AnimAssetWeightKey` · 00736640 · legions 006b21b0 — ctor/dtor: Rust struct construction and Drop

### AnimAssetScaleKey
- [-] `AnimAssetScaleKey::operator=` · 00736660 · legions 006b21d0 — operator=: Rust value semantics (glam/ac2-math provides it)

### AnimAssetWeightKey
- [-] `AnimAssetWeightKey::operator=` · 00736690 · legions 006b2230 — operator=: Rust value semantics (glam/ac2-math provides it)

### AnimAssetScaleKey
- [ ] `AnimAssetScaleKey::StreamPack` · 007366b0 · legions 006b2250 — needs port: AnimAsset key stream layout (value, then sTime)

### AnimAssetQuatKey
- [ ] `AnimAssetQuatKey::StreamPack` · 007366f0 · legions 006b2290 — needs port: AnimAsset key stream layout (value, then sTime)

### AnimAssetKey
- [-] `AnimAssetKey::FromFileNode` · 00736730 · legions 006b2310 — text source format (admin tools)

### AnimAssetScaleKey
- [-] `AnimAssetScaleKey::FromFileNode` · 00736810 · legions 006b23f0 — text source format (admin tools)

### AnimAssetOffsetKey
- [-] `AnimAssetOffsetKey::FromFileNode` · 007368d0 · legions 006b24b0 — text source format (admin tools)

### AnimAssetQuatKey
- [-] `AnimAssetQuatKey::FromFileNode` · 00736990 · legions 006b2570 — text source format (admin tools)

### AnimAssetWeightKey
- [-] `AnimAssetWeightKey::FromFileNode` · 00736a50 · legions 006b2630 — text source format (admin tools)

## `portal/shared/src/geometry/bbox.cpp.c` (6/26)

### Plane
- [ ] `Plane::IsValid` · 0061ca70 · legions 005d8660 — needs port: plane validity test (normal length threshold)

### BBox
- [-] `BBox::BBox` · 0061cb10 · legions 005d8700 — ctor/dtor: Rust struct construction and Drop
- [-] `BBox::GetPackSize` · 0061cb20 — stream size helper: Rust decodes by slice length, no size pass
- [-] `BBox::Pack` · 0061cb30 · legions 005d8720 — inline copy of 00426d40
- [ ] `BBox::UnPack` · 0061cbb0 — needs port: 28-byte BBox read (min, max, padding)
- [x] `BBox::InitForAdjustment` · 0061cc10 · legions 005d8800 — ac2-physics objects.rs (BBox::InitForAdjustment: empty box before adjust); ac2-math geometry.rs BBox::for_adjustment
- [x] `BBox::GetPlane` · 0061cc30 · legions 005d8820 — ac2-physics envcell.rs: portal clipping against the cell-search box
- [x] `BBox::localtoglobal` · 0061cd10 · legions 005d8900 — ac2-physics objects.rs: static-object collision dispatch, containment radius, shadow box
- [ ] `BBox::ScaleCentered` · 0061cf40 · legions 005d8b30 — needs port: scale a box about its centre
- [ ] `BBox::BuildBoundingBox` · 0061d040 · legions 005d8c30 — needs port: box built from a point set
- [x] `BBox::GetCenter` · 0061d070 · legions 005d8c60 — ac2-physics envcell.rs: portal clipping against the cell-search box
- [ ] `BBox::Within` · 0061d0b0 · legions 005d8ca0 — needs port (check): ac2-math geometry.rs BBox::contains_point is the analogue, uncited; retail inclusive test
- [ ] `BBox::Intersects` · 0061d110 · legions 005d8d00 — needs port: box-box and box-shape intersection tests
- [ ] `BBox::GetPoints` · 0061d1e0 · legions 005d8dd0 — needs port: the eight corners of a box
- [ ] `BBox::StreamPack` · 0061d2b0 — needs port: BBox stream layout, 28 bytes read and write
- [x] `BBox::InitFromCylSpheres` · 0061d300 · legions 005d8ef0 — ac2-physics objects.rs: static-object collision dispatch, containment radius, shadow box
- [x] `BBox::InitFromSpheres` · 0061d440 · legions 005d9030 — ac2-physics objects.rs: static-object collision dispatch, containment radius, shadow box
- [ ] `BBox::CalculateRadius` · 0061d580 · legions 005d9170 — needs port: box radius (half diagonal)
- [ ] `BBox::Intersects` · 0061d660 · legions 005d9250 — needs port: box-box and box-shape intersection tests
- [ ] `BBox::FindCoverRays` · 0061d750 · legions 005d9340 — needs port: cover rays for a box
- [-] `BBox::ToFileNode` · 0061e280 — text source format (admin tools)
- [-] `BBox::FromFileNode` · 0061e330 — text source format (admin tools)
- [ ] `BBox::FindXYPlaneIntersect` · 0061e530 · legions 005d9e70 — needs port: exact box and plane intersection (XY, XZ, YZ, general plane)
- [ ] `BBox::FindXZPlaneIntersect` · 0061e6c0 · legions 005da000 — needs port: exact box and plane intersection (XY, XZ, YZ, general plane)
- [ ] `BBox::FindYZPlaneIntersect` · 0061e850 · legions 005da190 — needs port: exact box and plane intersection (XY, XZ, YZ, general plane)
- [ ] `BBox::CalcPlaneIntersection` · 0061e9e0 · legions 005da320 — needs port: exact box and plane intersection (XY, XZ, YZ, general plane)

## `portal/shared/src/geometry/bvolume.cpp.c` (0/1)

### CHILDINFO
- [ ] `CHILDINFO::StreamPack` · 00737cb0 · legions 006b3870 — needs port: CHILDINFO stream layout

## `portal/shared/src/geometry/cellportal.cpp.c` (3/11)

### FileNodeName_Int32
- [-] `FileNodeName_Int32::Clone` · 0049c740 · legions 004944a0 — admin file-node source reader (PFileNode tree)

### CCellPortal
- [-] `CCellPortal::CCellPortal` · 0049c770 — ctor/dtor: Rust struct construction and Drop
- [-] `CCellPortal::~CCellPortal` · 0049c790 · legions 004944f0 — ctor/dtor: Rust struct construction and Drop
- [x] `CCellPortal::GetOtherCell` · 0049c7a0 · legions 00494500 — ac2-physics envcell.rs: portal clipping against the cell-search box
- [x] `CCellPortal::GetOtherCell` · 0049c810 · legions 00494560 — ac2-physics envcell.rs: portal clipping against the cell-search box
- [-] `CCellPortal::GetPackSize` · 0049c880 · legions 004945c0 — stream size helper: Rust decodes by slice length, no size pass
- [-] `CCellPortal::Pack` · 0049c8b0 · legions 004945f0 — write side of a DAT/admin layout the client never writes; the read side is ported or cited
- [x] `CCellPortal::UnPack` · 0049c980 · legions 004946c0 — ac2-world ac2_env_cell.rs (CCellPortal::UnPack, counted portals)

### PFileNode
- [-] `PFileNode::GetNameAsInt32` · 0049ca60 · legions 004947a0 — admin file-node source reader (PFileNode tree)
- [-] `PFileNode::GetSubName` · 0049cb20 — admin file-node source reader (PFileNode tree)

### CCellPortal
- [-] `CCellPortal::FromFileNode` · 0049cb40 — text source format (admin tools)

## `portal/shared/src/geometry/cylsphere.cpp.c` (9/9)

### CCylSphere
- [x] `CCylSphere::collides_with_sphere` · 00731160 · legions 006ad6a0 — ac2-physics shapes/cylsphere.rs (asm float order; Legions identical)
- [x] `CCylSphere::normal_of_collision` · 007311c0 · legions 006ad700 — ac2-physics shapes/cylsphere.rs (asm float order; Legions identical)
- [x] `CCylSphere::collide_with_point` · 007312d0 · legions 006ad810 — ac2-physics shapes/cylsphere.rs (asm float order; Legions identical)
- [x] `CCylSphere::slide_sphere` · 00731810 · legions 006add50 — ac2-physics shapes/cylsphere.rs (asm float order; Legions identical)
- [x] `CCylSphere::step_sphere_up` · 007318c0 · legions 006ade00 — ac2-physics shapes/cylsphere.rs (asm float order; Legions identical)
- [x] `CCylSphere::step_sphere_down` · 007319d0 · legions 006adf10 — ac2-physics shapes/cylsphere.rs (asm float order; Legions identical)
- [x] `CCylSphere::land_on_cylinder` · 00731b40 · legions 006ae080 — ac2-physics shapes/cylsphere.rs (asm float order; Legions identical)
- [x] `CCylSphere::intersects_sphere` · 00731bf0 · legions 006ae130 — ac2-physics shapes/cylsphere.rs (asm float order; Legions identical)
- [x] `CCylSphere::intersects_sphere` · 007320f0 · legions 006ae630 — ac2-physics shapes/cylsphere.rs (asm float order; Legions identical)

## `portal/shared/src/geometry/detectionobjray.cpp.c` (0/4)

### DetectionObjRay
- [ ] `DetectionObjRay::StreamPack` · 0062abb0 · legions 005e4c90 — needs port: DetectionObjRay stream layout
- [-] `DetectionObjRay::DetectionObjRay` · 0062ac10 · legions 005e4cf0 — ctor/dtor: Rust struct construction and Drop
- [-] `DetectionObjRay::~DetectionObjRay` · 0062ac40 · legions 005e4d20 — ctor/dtor: Rust struct construction and Drop
- [ ] `DetectionObjRay::AddRay` · 0062ac80 · legions 005e4d60 — needs port: detection ray list append (ordering)

## `portal/shared/src/geometry/frame.cpp.c` (7/19)

### Frame
- [x] `Frame::cache_matrix` · 00619f70 · legions 005d5c10
- [x] `Frame::cache_quaternion` · 00619fe0 · legions 005d5c80
- [ ] `Frame::SetMatrix` · 0061a040 · legions 005d5ce0 — needs port: frame from matrix (rotation extraction)
- [ ] `Frame::GetMatrix` · 0061a170 · legions 005d5e10 — needs port (check): implemented as ac2-math frame.rs Frame::to_mat4, uncited
- [x] `Frame::get_vector_heading` · 0061a220 · legions 005d5ec0
- [x] `Frame::get_heading` · 0061a240 · legions 005d5ee0
- [ ] `Frame::get_pitch` · 0061a270 · legions 005d5f10 — needs port: frame pitch read and set with the retail clamp
- [ ] `Frame::Pack` · 0061a290 · legions 005d5f30 — needs port: frame stream layout (write and read)
- [ ] `Frame::UnPack` · 0061a2a0 · legions 005d5f40 — needs port: frame stream layout (write and read)
- [-] `Frame::FromFileNode` · 0061a2d0 — text source format (admin tools)
- [-] `Frame::ToFileNode` · 0061a300 — text source format (admin tools)
- [x] `Frame::grotate` · 0061a310 · legions 005d5f70
- [~] `Frame::slerp_rotation` · 0061a3c0 · legions 005d6020 — ac2-physics interp (slice 3)
- [ ] `Frame::OrientFrame` · 0061a460 · legions 005d60c0 — needs port (check): ac2-math frame.rs align_axis_to_vector is the analogue, uncited
- [ ] `Frame::subtract2` · 0061a4e0 · legions 005d6140 — needs port (check): ac2-math frame.rs difference_from_frame is the analogue, uncited
- [x] `Frame::set_vector_heading` · 0061a5b0 · legions 005d6210
- [x] `Frame::set_heading` · 0061a810 · legions 005d6470
- [ ] `Frame::set_pitch` · 0061a860 · legions 005d64c0 — needs port: frame pitch read and set with the retail clamp
- [~] `Frame::slerp` · 0061a8a0 · legions 005d6500 — ac2-ui pregame/arms.rs (D3DXQuaternionSlerp in f64)

## `portal/shared/src/geometry/gcylsphere.cpp.c` (4/10)

### Cylsphere
- [ ] `Cylsphere::BuildBoundingBox` · 006211c0 · legions 005dc960 — needs port: cylinder bounding box, containment, stream layout and scale
- [x] `Cylsphere::Intersects` · 00621380 · legions 005dcb20
- [x] `Cylsphere::Intersects` · 00621390 · legions 005dcb30
- [x] `Cylsphere::Intersects` · 00621410 · legions 005dcbb0
- [ ] `Cylsphere::Within` · 00621420 · legions 005dcbc0 — needs port: cylinder bounding box, containment, stream layout and scale
- [ ] `Cylsphere::StreamPack` · 00621480 · legions 005dcc20 — needs port: cylinder bounding box, containment, stream layout and scale
- [ ] `Cylsphere::Scale` · 006214d0 · legions 005dcc70 — needs port: cylinder bounding box, containment, stream layout and scale
- [x] `Cylsphere::TraceRay` · 00621510 · legions 005dccb0 — ac2-physics los (Terrain::los_check)
- [-] `Cylsphere::ToFileNode` · 006218b0 — text source format (admin tools)
- [-] `Cylsphere::FromFileNode` · 006219a0 — text source format (admin tools)

## `portal/shared/src/geometry/gsphere.cpp.c` (3/7)

### Plane
- [-] `Plane::GetPackSize` · 00628450 · legions 005e33a0 — stream size helper: Rust decodes by slice length, no size pass

### Sphere
- [x] `Sphere::Intersects` · 00628460 · legions 005e33b0
- [x] `Sphere::Intersects` · 00628580 · legions 005e34d0
- [x] `Sphere::TraceRay` · 00628590 · legions 005e34e0 — ac2-physics los (Terrain::los_check)
- [ ] `Sphere::FindCoverRays` · 006286a0 · legions 005e35f0 — needs port: sphere cover rays and bounding box
- [-] `Sphere::FromFileNode` · 00628f40 — text source format (admin tools)
- [-] `Sphere::ToFileNode` · 00629040 — text source format (admin tools)

## `portal/shared/src/geometry/heading.cpp.c` (11/21)

### Heading
- [x] `Heading::get_degrees_0_centered` · 0061a8f0 · legions 005d6550
- [x] `Heading::get_radians` · 0061a910 · legions 005d6570
- [ ] `Heading::get_radians_0_centered` · 0061a920 · legions 005d6580 — needs port: heading in radians, centred on zero, as f64
- [x] `Heading::ToUInt8` · 0061a940 · legions 005d65a0
- [-] `Heading::Heading` · 0061a960 · legions 005d65c0 — ctor/dtor: Rust struct construction and Drop
- [-] `Heading::operator=` · 0061a980 · legions 005d65e0 — operator=: Rust value semantics (glam/ac2-math provides it)

### (free functions)
- [ ] `operator+` · 0061a990 · legions 005d65f0 — needs port: heading addition with the 360 degree wrap
- [x] `operator-` · 0061a9e0 · legions 005d6640

### Heading
- [x] `Heading::set` · 0061aa30 · legions 005d6690
- [x] `Heading::set_degrees` · 0061aaa0 · legions 005d6700
- [x] `Heading::FromUInt8` · 0061aaf0 · legions 005d6750
- [-] `Heading::Heading` · 0061ab50 · legions 005d67b0 — ctor/dtor: Rust struct construction and Drop
- [x] `Heading::Heading` · 0061ab60 · legions 005d67c0
- [x] `Heading::abs` · 0061ab80 · legions 005d67e0 — ac2-physics moveto (slice 3)
- [ ] `Heading::lerp` · 0061abf0 · legions 005d6850 — needs port: heading lerp along the shorter arc
- [x] `Heading::is_in_range` · 0061acb0 · legions 005d6910 — ac2-physics moveto (slice 3)
- [x] `Heading::is_inside` · 0061ad30 · legions 005d6990 — ac2-physics moveto (slice 3)
- [~] `Heading::equals` · 0061ad80 · legions 005d69e0 — in ac2-weenie reporter (ac2-math has no Heading yet)
- [ ] `Heading::is_zero` · 0061add0 · legions 005d6a30 — needs port: heading zero test with the retail epsilon
- [ ] `Heading::ToPString` · 0061ae20 · legions 005d6a80 — needs port: heading text form (narrow and wide)
- [ ] `Heading::ToWPString` · 0061b2e0 · legions 005d6f40 — needs port: heading text form (narrow and wide)

## `portal/shared/src/geometry/landdefs.cpp.c` (7/13)

### LandDefs
- [x] `LandDefs::init_misc` · 0061ed40 · legions 005da680 — ac2-world land_lattice.rs; ac2-physics landdefs.rs (LandDefs::init_misc installs the values at startup)
- [x] `LandDefs::set_bitlengths` · 0061ef30 · legions 005da870
- [ ] `LandDefs::set_height_table` · 0061efd0 · legions 005da910 — needs port: LandDefs height table, interpolation, sizing, nearest pixel and block lists
- [ ] `LandDefs::Interpolate` · 0061f020 · legions 005da960 — needs port: LandDefs height table, interpolation, sizing, nearest pixel and block lists
- [x] `LandDefs::MoveVectorToBeInBlock` · 0061f0e0 · legions 005daa20 — ac2-physics los (Terrain::los_check)
- [x] `LandDefs::get_outside_lcoord` · 0061f140 · legions 005daa80
- [x] `LandDefs::GetBlockOffset` · 0061f200 · legions 005dab40 — ac2-physics los (Terrain::los_check)
- [ ] `LandDefs::init_size` · 0061f240 · legions 005dab80 — needs port: LandDefs height table, interpolation, sizing, nearest pixel and block lists
- [x] `LandDefs::adjust_to_outside` · 0061f300 · legions 005dac40
- [x] `LandDefs::BlockRayClip` · 0061f400 · legions 005dad40 — ac2-physics los (Terrain::los_check)
- [ ] `LandDefs::GetNearestPixel` · 0061f880 · legions 005db1c0 — needs port: LandDefs height table, interpolation, sizing, nearest pixel and block lists
- [ ] `LandDefs::InqLandblockIIDFromStaticObjectIID` · 0061fb30 · legions 005db470 — needs port: LandDefs height table, interpolation, sizing, nearest pixel and block lists
- [ ] `LandDefs::get_block_list` · 0061fb90 · legions 005db4d0 — needs port: LandDefs height table, interpolation, sizing, nearest pixel and block lists

## `portal/shared/src/geometry/landscapedefs.cpp.c` (1/14)

### LandscapeDefs
- [-] `LandscapeDefs::LandscapeDefs` · 006297e0 · legions 005e4340 — ctor/dtor: Rust struct construction and Drop
- [-] `LandscapeDefs::GetDBOType` · 00629970 — accessor or type constant: Rust field or enum provides it
- [ ] `LandscapeDefs::InitEnd` · 00629980 · legions 005e44e0 — needs port: LandscapeDefs init, old defs and world extents
- [ ] `LandscapeDefs::InitOldLandDefs` · 00629a30 · legions 005e4590 — needs port: LandscapeDefs init, old defs and world extents
- [-] `LandscapeDefs::GetPackSize` · 00629a90 — stream size helper: Rust decodes by slice length, no size pass
- [-] `LandscapeDefs::Pack` · 00629ab0 — write side of a DAT/admin layout the client never writes; the read side is ported or cited
- [x] `LandscapeDefs::UnPack` · 00629d60
- [-] `LandscapeDefs::Allocate` · 00629ff0 — container/memory helper: Rust Vec and struct ownership
- [-] `LandscapeDefs::SetCurrentLandscapeDefs` · 0062a030 · legions 005e4b90 — singleton setter: Rust passes the landdefs value explicitly, no global
- [ ] `LandscapeDefs::SetWorldExtents` · 0062a070 · legions 005e4bd0 — needs port: LandscapeDefs init, old defs and world extents
- [ ] `LandscapeDefs::Init` · 0062a0c0 · legions 005e4c20 — needs port: LandscapeDefs init, old defs and world extents
- [-] `LandscapeDefs::LoadHeightEntry` · 0062a140 — text source loader, admin build only
- [-] `LandscapeDefs::LoadHeightList` · 0062a2f0 — text source loader, admin build only
- [-] `LandscapeDefs::FromFileNode` · 0062a430 — text source format (admin tools)

## `portal/shared/src/geometry/lightsource.cpp.c` (3/12)

### LightSource
- [-] `LightSource::operator=` · 00621cd0 · legions 005dd0b0 — operator=: Rust value semantics (glam/ac2-math provides it)
- [ ] `LightSource::GetIsRangeAnimating` · 00621eb0 · legions 005dd290 — needs port: light range animation state and animated range
- [ ] `LightSource::GetAnimatedRange` · 00621ed0 · legions 005dd2b0 — needs port: light range animation state and animated range
- [x] `LightSource::RefreshMaxRange` · 00621ef0 · legions 005dd2d0 — ac2-render object::cell / point_lights
- [x] `LightSource::CalculateAttenuation` · 00621f30 · legions 005dd310 — ac2-world ac2_light_info.rs (LightSource::CalculateAttenuation formula)
- [~] `LightSource::ApplyProperty` · 00621fb0 — pregame 3D scenes: no orthographic cameras, volumetric shadows or coronas; avatars without gmCharGenResult::InqAppliedAppearanceVDesc; no tooltips
- [ ] `LightSource::BuildOptionalProperties` · 00622190 — needs port: light optional property build from DAT
- [-] `LightSource::LightSource` · 00622500 · legions 005dd8d0 — ctor/dtor: Rust struct construction and Drop
- [x] `LightSource::ApplyPropertyCollection` · 00622650 — ac2-assets ac2_light_source.rs (LightSource::ApplyPropertyCollection decode)
- [ ] `LightSource::GetDiffuse` · 00622730 · legions 005ddb00 — needs port (check): cited only in render cell.rs comment; confirm GetDiffuse is implemented there
- [~] `LightSource::GetWeight` · 00622760 · legions 005ddb30 — static point lights: weight sort, six cap, range test, indoor/outdoor; sunlight-type and dynamic lights not yet
- [~] `LightSource::FillRenderLight` · 00622900 · legions 005ddcd0 — static point lights: weight sort, six cap, range test, indoor/outdoor; sunlight-type and dynamic lights not yet

## `portal/shared/src/geometry/matrix4.cpp.c` (11/18)

### Matrix4
- [x] `Matrix4::Translate` · 0061b390 · legions 005d6ff0 — ac2-render object::pass / texture_matrix, ac2-gpu object
- [x] `Matrix4::Translate` · 0061b3e0 · legions 005d7040 — ac2-render object::pass / texture_matrix, ac2-gpu object
- [-] `Matrix4::Translate3` · 0061b430 · legions 005d7090 — basic vector/matrix/quaternion operator: the Rust math type (glam) provides it
- [x] `Matrix4::RotateX` · 0061b480 · legions 005d70e0 — ac2-math perlin.rs, matrix.rs; ac2-render portal/tunnel.rs; tests: tunnel_tests, perlin, matrix
- [x] `Matrix4::RotateY` · 0061b4d0 · legions 005d7130 — ac2-math perlin.rs, matrix.rs; ac2-render portal/tunnel.rs; tests: tunnel_tests, perlin, matrix
- [x] `Matrix4::RotateZ` · 0061b520 · legions 005d7180 — ac2-math perlin.rs, matrix.rs; ac2-render portal/tunnel.rs; tests: tunnel_tests, perlin, matrix
- [x] `Matrix4::Scale` · 0061b570 · legions 005d71d0 — ac2-render object::pass / texture_matrix, ac2-gpu object
- [x] `Matrix4::CalcDeterminant` · 0061b5c0 · legions 005d7220 — ac2-render object/skeleton.rs (bone matrices)
- [x] `Matrix4::Adjoint` · 0061b780 · legions 005d73e0 — ac2-render object/skeleton.rs (bone matrices)
- [-] `Matrix4::Inverse3` · 0061bb10 · legions 005d7770 — basic vector/matrix/quaternion operator: the Rust math type (glam) provides it
- [ ] `Matrix4::QuaternionFromRotation` · 0061bc80 · legions 005d78e0 — needs port: matrix to quaternion convention
- [x] `Matrix4::RotateBasis` · 0061bce0 · legions 005d7940 — ac2-render object/skeleton.rs (bone matrices)
- [-] `Matrix4::Transpose` · 0061bd60 · legions 005d79c0 — basic vector/matrix/quaternion operator: the Rust math type (glam) provides it
- [x] `Matrix4::Inverse` · 0061bdd0 · legions 005d7a30 — ac2-render object/skeleton.rs (bone matrices)
- [ ] `Matrix4::DirectionMatrix` · 0061be40 · legions 005d7aa0 — needs port (check): cited only by shadow_map.rs and emitter.rs comments; no implementing function cited
- [x] `Matrix4::RotationFromQuaternion` · 0061c080 · legions 005d7ce0 — ac2-render object/skeleton.rs (bone matrices)
- [~] `Matrix4::RotateAngles` · 0061c0c0 · legions 005d7d20 — the forward row CameraManager::UpdateCamera reads (ac2-client camera.rs)
- [ ] `Matrix4::LookAt` · 0061c1a0 · legions 005d7e00 — needs port: LookAt matrix convention

## `portal/shared/src/geometry/perlinnoise.cpp.c` (2/3)

### PerlinNoise
- [x] `PerlinNoise::Init` · 0062b040 · legions 005e4f60 — ac2-math perlin.rs, matrix.rs; ac2-render portal/tunnel.rs; tests: tunnel_tests, perlin, matrix
- [x] `PerlinNoise::Noise` · 0062b0f0 · legions 005e5010 — ac2-math perlin.rs, matrix.rs; ac2-render portal/tunnel.rs; tests: tunnel_tests, perlin, matrix
- [ ] `PerlinNoise::fBm1` · 0062b180 · legions 005e50a0 — needs port: PerlinNoise::fBm1 for FRACTAL waveforms (not reproduced)

## `portal/shared/src/geometry/plane.cpp.c` (2/16)

### Plane
- [x] `Plane::localtoglobal` · 00623960 · legions 005ded30 — ac2-physics bsp/tree.rs, position.rs: physics-BSP collision in mesh space (asm float order; Legions identical)
- [x] `Plane::intersect_line` · 00623a20 · legions 005dedf0 — ac2-physics los (Terrain::los_check)
- [ ] `Plane::compute_time_of_intersection` · 00623b30 · legions 005def00 — needs port: plane time of intersection and intercept
- [ ] `Plane::FindIntercept` · 00623bb0 · legions 005def80 — needs port: plane time of intersection and intercept
- [-] `Plane::Pack` · 00623c40 · legions 005df010 — write side of a DAT/admin layout the client never writes; the read side is ported or cited
- [ ] `Plane::UnPack` · 00623c90 · legions 005df060 — needs port: plane stream read (normal and distance)
- [ ] `Plane::IntersectLineSegment` · 00623ce0 · legions 005df0b0 — needs port: line segment, clip-to-positive and cut-ray plane tests
- [ ] `Plane::ClipRayToPositive` · 00623e40 · legions 005df210 — needs port: line segment, clip-to-positive and cut-ray plane tests
- [ ] `Plane::OrientFrameToPlane` · 00623fc0 · legions 005df390 — needs port: orient a frame onto a plane
- [ ] `Plane::StreamPack` · 006240c0 · legions 005df490 — needs port: plane stream layout (read and write)
- [ ] `Plane::CutRay` · 006240f0 · legions 005df4c0 — needs port: line segment, clip-to-positive and cut-ray plane tests
- [ ] `Plane::GetString` · 00624310 · legions 005df6e0 — needs port (check): plane text form, compare against Vector::GetStringW format
- [ ] `Plane::GetStringW` · 00624380 · legions 005df750 — needs port (check): plane text form, compare against Vector::GetStringW format
- [-] `Plane::FromFileNode` · 00624440 — text source format (admin tools)
- [-] `Plane::ToFileNode` · 00624550 — text source format (admin tools)
- [ ] `Plane::SplitPolygon` · 006245b0 · legions 005df810 — needs port: split a polygon by a plane

## `portal/shared/src/geometry/polygon.cpp.c` (18/36)

### SPHEREPATH
- [x] `SPHEREPATH::is_walkable_allowable` · 00499290 · legions 004917c0 — ac2-physics transition/localspace.rs, sphere_path.rs

### CPolygon
- [-] `CPolygon::CPolygon` · 004992b0 — ctor/dtor: Rust struct construction and Drop
- [-] `CPolygon::GetVertexLoc` · 004992d0 · legions 00491800 — accessor or type constant: Rust field or enum provides it
- [x] `CPolygon::point_in_poly2D` · 004992f0 · legions 00491820
- [ ] `CPolygon::circle_in_poly2D` · 00499380 · legions 004918b0 — needs port: circle inside a 2D polygon
- [-] `CPolygon::Destroy` · 00499530 · legions 00491a60 — container/memory helper: Rust Vec and struct ownership
- [-] `CPolygon::InitEmpty` · 00499570 · legions 00491aa0 — container/memory helper: Rust Vec and struct ownership
- [-] `CPolygon::GetPackSize` · 004995c0 · legions 00491af0 — stream size helper: Rust decodes by slice length, no size pass
- [-] `CPolygon::Pack` · 004995e0 · legions 00491b10 — write side of a DAT/admin layout the client never writes; the read side is ported or cited
- [x] `CPolygon::adjust_sphere_to_poly` · 00499670 · legions 00491ba0 — ac2-physics bsp/polygon.rs (asm float order; Legions identical)
- [x] `CPolygon::adjust_sphere_to_plane` · 00499700 · legions 00491c30 — ac2-physics bsp/polygon.rs (asm float order; Legions identical)
- [x] `CPolygon::adjust_to_placement_poly` · 00499800 · legions 00491d30 — ac2-physics bsp: placement insert (Legions identical)

### PackObj
- [-] `PackObj::GetPackSize` · 004998b0 — stream size helper: Rust decodes by slice length, no size pass

### CPolygon
- [-] `CPolygon::operator=` · 004998f0 · legions 00491de0 — operator=: Rust value semantics (glam/ac2-math provides it)
- [x] `CPolygon::MakePlane` · 00499940 · legions 00491e30 — ac2-physics bsp/polygon.rs (asm float order; Legions identical)
- [x] `CPolygon::UnPack` · 00499a90 · legions 00491f80 — ac2-assets ac2_physics_mesh.rs (CPolygon::UnPack layout decode)
- [x] `CPolygon::polygon_hits_sphere_slow_but_sure` · 00499b90 · legions 00492080 — ac2-physics bsp/polygon.rs (asm float order; Legions identical)
- [x] `CPolygon::point_in_polygon` · 00499e60 · legions 00492350 — ac2-physics los (Terrain::los_check)
- [x] `CPolygon::check_walkable` · 00499f30 · legions 00492420 — ac2-physics bsp/polygon.rs (asm float order; Legions identical)
- [x] `CPolygon::check_small_walkable` · 0049a160 · legions 00492650 — ac2-physics bsp/polygon.rs (asm float order; Legions identical)
- [x] `CPolygon::find_crossed_edge` · 0049a3a0 · legions 00492890 — ac2-physics bsp/polygon.rs (asm float order; Legions identical)

### WorkerPolygon
- [-] `WorkerPolygon::WorkerPolygon` · 0049a530 · legions 00492a20 — ctor/dtor: Rust struct construction and Drop
- [ ] `WorkerPolygon::RecalculatePlane` · 0049a560 · legions 00492a50 — needs port: worker polygon plane recalculation and area
- [ ] `WorkerPolygon::CalcArea` · 0049a6a0 · legions 00492b90 — needs port: worker polygon plane recalculation and area

### CPolygon
- [x] `CPolygon::pos_hits_sphere` · 0049a860 · legions 00492d50 — ac2-physics bsp/polygon.rs (asm float order; Legions identical)
- [x] `CPolygon::hits_sphere` · 0049a8b0 · legions 00492da0 — ac2-physics bsp/polygon.rs (asm float order; Legions identical)
- [x] `CPolygon::walkable_hits_sphere` · 0049a8d0 · legions 00492dc0 — ac2-physics bsp/polygon.rs (asm float order; Legions identical)
- [x] `CPolygon::TraceRay` · 0049a920 · legions 00492e10 — ac2-physics los (Terrain::los_check)

### SmartArray<CPolygon>
- [-] `SmartArray<CPolygon>::SetNElements` · 0049a9c0 — container/memory helper: Rust Vec and struct ownership

### WorkerPolygon
- [x] `WorkerPolygon::InitFromPolygon` · 0049a9f0 · legions 00492eb0 — ac2-physics envcell.rs: portal clipping against the cell-search box

### CPolygon
- [-] `CPolygon::FromFileNode` · 0049aad0 — text source format (admin tools)

### WorkerPolygon
- [x] `WorkerPolygon::Clip` · 0049acb0 · legions 00492f90 — ac2-physics envcell.rs: portal clipping against the cell-search box

### CPolygon
- [-] `CPolygon::FromFileStream` · 0049afe0 — admin file-node source reader (PFileNode tree)
- [-] `CPolygon::ReadTextPolygonArray` · 0049b1b0 — admin file-node source reader (PFileNode tree)
- [-] `CPolygon::ReadBinaryPolygonArray` · 0049b240 — admin file-node source reader (PFileNode tree)
- [-] `CPolygon::ReadPolygonArray` · 0049b340 — admin file-node source reader (PFileNode tree)

## `portal/shared/src/geometry/portalpoly.cpp.c` (1/3)

### CPortalPoly
- [-] `CPortalPoly::CPortalPoly` · 0072f870 — ctor/dtor: Rust struct construction and Drop
- [x] `CPortalPoly::StreamPack` · 0072f890 · legions 006ad130 — ac2-world ac2_cell_struct.rs (CPortalPoly::StreamPack decode)
- [-] `CPortalPoly::FromFileNode` · 0072f930 — text source format (admin tools)

## `portal/shared/src/geometry/position.cpp.c` (26/36)

### Position
- [-] `Position::Position` · 0061fd70 · legions 005db6b0 — ctor/dtor: Rust struct construction and Drop
- [x] `Position::distance` · 0061fdb0 · legions 005db6f0 — ac2-physics interp (slice 3)
- [x] `Position::distance_squared` · 0061fdf0 · legions 005db730 — ac2-physics interp (slice 3)
- [x] `Position::xy_distance` · 0061fe30 · legions 005db770 — ac2-script glue::position (ported beside the gPosition glue)
- [x] `Position::xy_distance_squared` · 0061fe60 · legions 005db7a0 — ac2-script glue::position (ported beside the gPosition glue)
- [x] `Position::heading` · 0061fe90 · legions 005db7d0 — ac2-physics interp (slice 3)
- [x] `Position::pitch` · 0061ff20 · legions 005db860 — ac2-script glue::position (ported beside the gPosition glue)
- [x] `Position::determine_quadrant` · 0061ffa0 · legions 005db8e0 — ac2-script glue::position (ported beside the gPosition glue)
- [-] `Position::GetPackSize` · 00620060 · legions 005db9a0 — stream size helper: Rust decodes by slice length, no size pass
- [x] `Position::Pack` · 00620070 · legions 005db9b0 — ac2-net protocol::native pack+unpack (Legions layouts; WState graphs round-trip byte-exact); ac2-script still decodes its own copy until it switches
- [x] `Position::UnPack` · 006200b0 · legions 005db9f0
- [x] `Position::cylinder_distance` · 006200f0 · legions 005dba30 — ac2-physics moveto (slice 3)
- [ ] `Position::cylinder_distance_squared` · 00620190 · legions 005dbad0 — needs port (check): ac2-physics moveto Position::cylinder_distance is the non-squared form
- [x] `Position::IsValid` · 00620230 · legions 005dbb70 — ac2-script glue::position (ported beside the gPosition glue)
- [ ] `Position::CreateRay` · 00620260 · legions 005dbba0 — needs port: ray from a position

### LandDefs
- [x] `LandDefs::gid_to_game_map_coords` · 006202c0 · legions 005dbc00 — ac2-physics landdefs.rs + ac2-client ui_sources.rs map_coords_string (the radar) + ac2-script glue
- [x] `LandDefs::game_map_coords_to_gid` · 00620350 · legions 005dbc90 — ac2-script glue::position (ported beside the gPosition glue)
- [x] `LandDefs::GetCellDistanceSquared` · 006203e0 · legions 005dbd20 — ac2-script glue::position (ported beside the gPosition glue)

### LM_UVRotate
- [-] `LM_UVRotate::GetType` · 00620480 · legions 00599340 — accessor or type constant: Rust field or enum provides it

### FileNodeName_CellID
- [-] `FileNodeName_CellID::Clone` · 00620490 — admin file-node source reader (PFileNode tree)

### PFileNode
- [-] `PFileNode::MakeSubNodeAsCellID` · 006204c0 — admin file-node source reader (PFileNode tree)

### Position
- [x] `Position::GetCellDistanceSquared` · 00620520 — ac2-script glue::position (ported beside the gPosition glue)
- [x] `Position::FromWPString` · 00620530 · legions 005dbdd0 — ac2-script glue/position.rs (Position::FromWPString)
- [x] `Position::GetCellX` · 006205f0 · legions 005dbe90 — ac2-script glue::position (ported beside the gPosition glue)
- [x] `Position::GetCellY` · 00620620 · legions 005dbec0 — ac2-script glue::position (ported beside the gPosition glue)
- [x] `Position::FromCellCoords` · 00620650 · legions 005dbef0 — ac2-script glue::position (ported beside the gPosition glue)
- [x] `Position::ToRealMapCoords_Info` · 006206d0 · legions 005dbf70 — ac2-physics landdefs.rs + ac2-client ui_sources.rs map_coords_string (the radar) + ac2-script glue
- [ ] `Position::FromGameMapCoords` · 00620830 · legions 005dc0d0 — needs port: game map coordinates to position
- [x] `Position::FromRealMapCoords` · 006208b0 · legions 005dc150 — ac2-script glue/position.rs (Position::FromRealMapCoords)
- [x] `Position::GetCellDistance` · 00620a40 · legions 005dc2e0 — ac2-script glue::position (ported beside the gPosition glue)
- [x] `Position::FromCellCoordsString` · 00620a60 · legions 005dc300 — ac2-script glue::position (ported beside the gPosition glue)
- [x] `Position::ToWPString` · 00620ab0 · legions 005dc350 — ac2-script glue::position (ported beside the gPosition glue)
- [x] `Position::ToCellCoordsString` · 00620b50 · legions 005dc3f0 — ac2-script glue::position (ported beside the gPosition glue)
- [x] `Position::ToRealMapCoords` · 00620bf0 · legions 005dc490 — ac2-script glue::position (ported beside the gPosition glue)
- [-] `Position::FromFileNode` · 00620d20 — text source format (admin tools)
- [-] `Position::ToFileNode` · 00620db0 — text source format (admin tools)

## `portal/shared/src/geometry/positionoffset.cpp.c` (3/6)

### PositionOffset
- [x] `PositionOffset::ToPosition` · 00625860 · legions 005e0a40 — ac2-client remote.rs (PositionOffset::ToPosition)
- [x] `PositionOffset::FromPosition` · 006258e0 · legions 005e0ac0
- [-] `PositionOffset::PositionOffset` · 00625900 · legions 005e0ae0 — ctor/dtor: Rust struct construction and Drop
- [x] `PositionOffset::StreamPack` · 00625930 · legions 005e0b10
- [-] `PositionOffset::FromFileNode` · 00625980 — text source format (admin tools)
- [-] `PositionOffset::ToFileNode` · 00625b60 — text source format (admin tools)

## `portal/shared/src/geometry/quaternion.cpp.c` (4/11)

### Quaternion
- [x] `Quaternion::rotate` · 0061c210 · legions 005d7e70
- [-] `Quaternion::Transform` · 0061c2e0 · legions 005d7f40 — basic vector/matrix/quaternion operator: the Rust math type (glam) provides it
- [ ] `Quaternion::RotationAxis` · 0061c3c0 · legions 005d8020 — needs port: rotation axes to quaternion and the normalize rule
- [ ] `Quaternion::SetRotationAxesInternal` · 0061c3f0 · legions 005d8050 — needs port: rotation axes to quaternion and the normalize rule
- [x] `Quaternion::InqRotationAxes` · 0061c570 · legions 005d81d0
- [-] `Quaternion::Pack` · 0061c640 · legions 005d82a0 — write side of a DAT/admin layout the client never writes; the read side is ported or cited
- [ ] `Quaternion::SetRotationAxesAndNormalize` · 0061c680 · legions 005d82e0 — needs port: rotation axes to quaternion and the normalize rule
- [x] `Quaternion::SetRotationAxes` · 0061c700 · legions 005d8360
- [x] `Quaternion::UnPack` · 0061c710 · legions 005d8370
- [-] `Quaternion::ToFileNode` · 0061c850 — text source format (admin tools)
- [-] `Quaternion::FromFileNode` · 0061c8c0 · legions 005d84b0 — text source format (admin tools)

## `portal/shared/src/geometry/ray.cpp.c` (0/3)

### Ray
- [ ] `Ray::StreamPack` · 00621b30 — needs port: Ray stream layout
- [-] `Ray::FromFileNode` · 00621b90 — text source format (admin tools)
- [-] `Ray::ToFileNode` · 00621c50 — text source format (admin tools)

## `portal/shared/src/geometry/rgbacolor.cpp.c` (4/7)

### RGBAColor
- [-] `RGBAColor::Pack` · 006290a0 · legions 005e3e90 — write side of a DAT/admin layout the client never writes; the read side is ported or cited
- [x] `RGBAColor::UnPack` · 006290c0 · legions 005e3eb0 — ac2-assets ac2_ui_scene.rs (RGBAColor::UnPack, fog colour)
- [x] `RGBAColor::RGBtoHSB` · 006290f0 · legions 005e3ee0
- [x] `RGBAColor::HSBtoRGB` · 006292a0 · legions 005e4090
- [x] `RGBAColor::StreamPack` · 006294f0 · legions 005e42e0
- [-] `RGBAColor::ToFileNode` · 00629550 — text source format (admin tools)
- [-] `RGBAColor::FromFileNode` · 006295c0 — text source format (admin tools)

## `portal/shared/src/geometry/simd.cpp.c` (0/1)

### SIMD
- [-] `SIMD::Startup` · 0062b360 · legions 005e5280 — CPU feature detection: Rust uses compile-time SIMD

## `portal/shared/src/geometry/simplepolygon.cpp.c` (0/25)

### SmartArray<Vector>
- [-] `SmartArray<Vector>::RemoveOrderedByIndex` · 00625c10 · legions 005e0b60 — container/memory helper: Rust Vec and struct ownership

### SimplePolygon
- [ ] `SimplePolygon::GetEdge` · 00625c60 · legions 005e0bb0 — needs port: retail SimplePolygon clipping, convex hull and edge tests (exact vertex order and epsilons)
- [ ] `SimplePolygon::ReplaceEdgeWithMidpoint` · 00625cd0 · legions 005e0c20 — needs port: retail SimplePolygon clipping, convex hull and edge tests (exact vertex order and epsilons)
- [ ] `SimplePolygon::GetEdgePlane` · 00625da0 · legions 005e0cf0 — needs port: retail SimplePolygon clipping, convex hull and edge tests (exact vertex order and epsilons)
- [ ] `SimplePolygon::RemoveColinearAndConcavePoints` · 00625e70 · legions 005e0dc0 — needs port: retail SimplePolygon clipping, convex hull and edge tests (exact vertex order and epsilons)
- [ ] `SimplePolygon::FindArea` · 006260d0 · legions 005e1020 — needs port: retail SimplePolygon clipping, convex hull and edge tests (exact vertex order and epsilons)
- [ ] `SimplePolygon::FindArea2D` · 00626420 · legions 005e1370 — needs port: retail SimplePolygon clipping, convex hull and edge tests (exact vertex order and epsilons)
- [ ] `SimplePolygon::CheckIfInPlane` · 00626560 · legions 005e14b0 — needs port: retail SimplePolygon clipping, convex hull and edge tests (exact vertex order and epsilons)
- [ ] `SimplePolygon::GetBounds` · 006265b0 · legions 005e1500 — needs port: retail SimplePolygon clipping, convex hull and edge tests (exact vertex order and epsilons)
- [ ] `SimplePolygon::RemoveShortEdges` · 00626610 · legions 005e1560 — needs port: retail SimplePolygon clipping, convex hull and edge tests (exact vertex order and epsilons)
- [ ] `SimplePolygon::IsOutside` · 00626790 · legions 005e16e0 — needs port: retail SimplePolygon clipping, convex hull and edge tests (exact vertex order and epsilons)
- [ ] `SimplePolygon::SelfCutByEdges` · 006268c0 · legions 005e1810 — needs port: retail SimplePolygon clipping, convex hull and edge tests (exact vertex order and epsilons)
- [ ] `SimplePolygon::IsDegenerate` · 00626c90 · legions 005e1be0 — needs port: retail SimplePolygon clipping, convex hull and edge tests (exact vertex order and epsilons)
- [ ] `SimplePolygon::CalculatePlane` · 00626cb0 · legions 005e1c00 — needs port: retail SimplePolygon clipping, convex hull and edge tests (exact vertex order and epsilons)
- [ ] `SimplePolygon::IsConvex` · 00626ff0 · legions 005e1f61 — needs port: retail SimplePolygon clipping, convex hull and edge tests (exact vertex order and epsilons)
- [ ] `SimplePolygon::AdjustToPolygonHeight` · 00627150 · legions 005e20a0 — needs port: retail SimplePolygon clipping, convex hull and edge tests (exact vertex order and epsilons)

### SmartArray<Vector>
- [-] `SmartArray<Vector>::InsertOrderedByIndex` · 00627450 · legions 005e23a0 — container/memory helper: Rust Vec and struct ownership

### SimplePolygon
- [ ] `SimplePolygon::ExtendConvexHull` · 006274f0 · legions 005e2440 — needs port: retail SimplePolygon clipping, convex hull and edge tests (exact vertex order and epsilons)
- [ ] `SimplePolygon::AdjustVertexHeights` · 00627860 · legions 005e27b0 — needs port: retail SimplePolygon clipping, convex hull and edge tests (exact vertex order and epsilons)
- [ ] `SimplePolygon::CleanupConvex` · 00627960 · legions 005e28b0 — needs port: retail SimplePolygon clipping, convex hull and edge tests (exact vertex order and epsilons)
- [ ] `SimplePolygon::ApplyZConstraints` · 00627a20 · legions 005e2970 — needs port: retail SimplePolygon clipping, convex hull and edge tests (exact vertex order and epsilons)

### SmartArray<Ray>
- [-] `SmartArray<Ray>::AddToEnd` · 00627eb0 · legions 005e2e00 — container/memory helper: Rust Vec and struct ownership

### SimplePolygon
- [ ] `SimplePolygon::Convexify` · 00627f20 · legions 005e2e70 — needs port: retail SimplePolygon clipping, convex hull and edge tests (exact vertex order and epsilons)
- [ ] `SimplePolygon::ConvexifyWithZConstraints` · 00628050 · legions 005e2fa0 — needs port: retail SimplePolygon clipping, convex hull and edge tests (exact vertex order and epsilons)
- [ ] `SimplePolygon::Calc2DOverlap` · 006280c0 · legions 005e3010 — needs port: retail SimplePolygon clipping, convex hull and edge tests (exact vertex order and epsilons)

## `portal/shared/src/geometry/sphcone.cpp.c` (0/9)

### Sphere
- [ ] `Sphere::BuildBoundingBox` · 00736b30 · legions 006b2710 — needs port: sphere cover rays and bounding box

### SphCone
- [-] `SphCone::SphCone` · 00736bd0 · legions 006b27b0 — ctor/dtor: Rust struct construction and Drop
- [-] `SphCone::SphCone` · 00736c20 · legions 006b2800 — ctor/dtor: Rust struct construction and Drop
- [-] `SphCone::SphCone` · 00736c60 · legions 006b2840 — ctor/dtor: Rust struct construction and Drop
- [ ] `SphCone::AngleCheck` · 00736cb0 · legions 006b2890 — needs port: sphere-cone angle test, intersections and stream layout
- [ ] `SphCone::Intersects` · 00736d50 · legions 006b2930 — needs port: sphere-cone angle test, intersections and stream layout
- [ ] `SphCone::Intersects` · 00736ee0 · legions 006b2ac0 — needs port: sphere-cone angle test, intersections and stream layout
- [ ] `SphCone::Intersects` · 00737150 · legions 006b2d30 — needs port: sphere-cone angle test, intersections and stream layout
- [ ] `SphCone::StreamPack` · 00737290 · legions 006b2e70 — needs port: sphere-cone angle test, intersections and stream layout

## `portal/shared/src/geometry/sphere.cpp.c` (15/15)

### (free functions)
- [x] `IsFloatZero` · 0049e170 · legions 00495560 — ac2-physics shapes/sphere.rs (asm float order; Legions identical)

### Position
- [x] `Position::localtoglobal` · 0049e190 · legions 00495580 — ac2-physics bsp/tree.rs, position.rs: physics-BSP collision in mesh space (asm float order; Legions identical)

### SPHEREPATH
- [x] `SPHEREPATH::get_curr_pos_check_pos_block_offset` · 0049e230 · legions 00495620 — ac2-physics transition/localspace.rs, sphere_path.rs
- [x] `SPHEREPATH::add_offset_to_check_pos` · 0049e260 · legions 00495650 — ac2-physics transition/localspace.rs, sphere_path.rs

### CSphere
- [x] `CSphere::find_time_of_collision` · 0049e2b0 · legions 004956a0 — ac2-physics shapes/sphere.rs (asm float order; Legions identical)
- [x] `CSphere::collides_with_sphere` · 0049e3a0 · legions 00495790 — ac2-physics shapes/sphere.rs (asm float order; Legions identical)
- [x] `CSphere::step_sphere_down` · 0049e3e0 · legions 004957d0 — ac2-physics shapes/sphere.rs (asm float order; Legions identical)
- [x] `CSphere::collide_with_point` · 0049e620 · legions 00495a10 — ac2-physics shapes/sphere.rs (asm float order; Legions identical)
- [x] `CSphere::slide_sphere` · 0049e860 · legions 00495c50 — ac2-physics shapes/sphere.rs (asm float order; Legions identical)
- [x] `CSphere::slide_sphere` · 0049eb70 · legions 00495f60 — ac2-physics shapes/sphere.rs (asm float order; Legions identical)
- [x] `CSphere::step_sphere_up` · 0049eec0 · legions 004962b0 — ac2-physics shapes/sphere.rs (asm float order; Legions identical)
- [x] `CSphere::land_on_sphere` · 0049ef60 · legions 00496350 — ac2-physics shapes/sphere.rs (asm float order; Legions identical)
- [x] `CSphere::slide_sphere` · 0049f000 · legions 004963f0 — ac2-physics shapes/sphere.rs (asm float order; Legions identical)
- [x] `CSphere::intersects_sphere` · 0049f0a0 · legions 00496490 — ac2-physics shapes/sphere.rs (asm float order; Legions identical)
- [x] `CSphere::intersects_sphere` · 0049f5f0 · legions 004969e0 — ac2-physics shapes/sphere.rs (asm float order; Legions identical)

## `portal/shared/src/geometry/sphplanecone.cpp.c` (0/9)

### SphPlaneCone
- [ ] `SphPlaneCone::BuildBoundingBox` · 00737310 · legions 006b2ef0 — needs port: sphere-plane-cone bounds, angle check, intersections and stream layout
- [ ] `SphPlaneCone::Intersects` · 007373b0 · legions 006b2f90 — needs port: sphere-plane-cone bounds, angle check, intersections and stream layout
- [-] `SphPlaneCone::SphPlaneCone` · 007373f0 · legions 006b2fd0 — ctor/dtor: Rust struct construction and Drop
- [-] `SphPlaneCone::SphPlaneCone` · 00737450 · legions 006b3010 — ctor/dtor: Rust struct construction and Drop
- [-] `SphPlaneCone::SphPlaneCone` · 007374b0 · legions 006b3070 — ctor/dtor: Rust struct construction and Drop
- [ ] `SphPlaneCone::CheckAngle` · 00737510 · legions 006b30d0 — needs port: sphere-plane-cone bounds, angle check, intersections and stream layout
- [ ] `SphPlaneCone::Intersects` · 00737610 · legions 006b31d0 — needs port: sphere-plane-cone bounds, angle check, intersections and stream layout
- [ ] `SphPlaneCone::Intersects` · 007379a0 · legions 006b3560 — needs port: sphere-plane-cone bounds, angle check, intersections and stream layout
- [ ] `SphPlaneCone::StreamPack` · 00737bd0 · legions 006b3790 — needs port: sphere-plane-cone bounds, angle check, intersections and stream layout

## `portal/shared/src/geometry/transform.cpp.c` (4/13)

### TransformBase
- [-] `TransformBase::GetPackSize` · 00624ed0 · legions 005e0130 — stream size helper: Rust decodes by slice length, no size pass
- [-] `TransformBase::Pack` · 00624ee0 · legions 005e0140 — write side of a DAT/admin layout the client never writes; the read side is ported or cited
- [ ] `TransformBase::SetFrame` · 00624f60 · legions 005e01c0 — needs port: transform frame set, validity and normalisation
- [x] `TransformBase::AddWeightedTransform` · 00624fb0 · legions 005e0210 — ac2-anim dbanimator / ac2-physics anim_pose (slice 3)
- [x] `TransformBase::WeightTransform` · 00625070 · legions 005e02d0 — ac2-anim dbanimator / ac2-physics anim_pose (slice 3)
- [ ] `TransformBase::NormalizeEvenInvalid` · 00625110 · legions 005e0370 — needs port: transform frame set, validity and normalisation
- [x] `TransformBase::FillMatrix` · 00625190 · legions 005e03f0 — ac2-render object/skeleton.rs (bone matrices)

### (free functions)
- [-] `STREAMPACK_PACKOBJ` · 00625290 · legions 005e04f0 — helper macro for the stream layout; the Rust readers decode directly

### TransformBase
- [ ] `TransformBase::IsValid` · 00625300 · legions 005e0560 — needs port: transform frame set, validity and normalisation
- [x] `TransformBase::UnPack` · 00625460 · legions 005e06c0 — ac2-anim setup.rs (TransformBase::UnPack: offset, scale, quaternion w,x,y,z)
- [ ] `TransformBase::NormalizeIfInvalid` · 006254e0 · legions 005e0740 — needs port: transform frame set, validity and normalisation
- [-] `TransformBase::FromFileNode` · 00625640 · legions 005e08a0 — text source format (admin tools)
- [-] `TransformBase::ToFileNode` · 006257e0 — text source format (admin tools)

## `portal/shared/src/geometry/vec2d.cpp.c` (0/1)

### Vec2D
- [-] `Vec2D::Vec2D` · 00628430 · legions 005e3380 — ctor/dtor: Rust struct construction and Drop

## `portal/shared/src/geometry/vector.cpp.c` (2/7)

### Vector
- [x] `Vector::FromSpherical` · 00620e20 · legions 005dc5c0
- [~] `Vector::UnitVectorFromUInt24` · 00620e60 · legions 005dc600 — wire quantizer in ac2-net protocol::position
- [-] `Vector::ToFileNode` · 00620ed0 · legions 005dc670 — text source format (admin tools)
- [~] `Vector::UnitVectorToUInt24` · 00620f70 · legions 005dc710 — wire quantizer in ac2-net protocol::position
- [-] `Vector::FromFileNode` · 00621090 · legions 005dc830 — text source format (admin tools)
- [ ] `Vector::GetString` · 00621160 · legions 005dc900 — needs port (check): same format as Vector::GetStringW (ac2-script glue/math.rs)
- [x] `Vector::GetStringW` · 00621190 · legions 005dc930 — ac2-script glue/math.rs (Vector::GetStringW format)

## `portal/shared/src/geometry/vertex.cpp.c` (1/1)

### VertexFormatInfo
- [x] `VertexFormatInfo::GenerateOffsets` · 0062b380 · legions 005e52a0 — ac2-assets ac2_mesh.rs (VertexFormatInfo::GenerateOffsets stride)

## `portal/shared/src/geometry/vertexarray.cpp.c` (0/31)

### CVertexArray
- [-] `CVertexArray::Begin` · 0072fc70 · legions 006ad1d0 — D3D vertex buffer wrapper: Rust vertex buffers
- [-] `CVertexArray::End` · 0072fca0 · legions 006ad200 — D3D vertex buffer wrapper: Rust vertex buffers
- [-] `CVertexArray::RequestDynamicVertices` · 0072fcd0 · legions 006ad230 — D3D vertex buffer wrapper: Rust vertex buffers
- [-] `CVertexArray::RequestStripHWExtraTexCoords` · 0072fd00 · legions 006ad260 — D3D vertex buffer wrapper: Rust vertex buffers
- [-] `CVertexArray::Startup` · 0072fd30 · legions 006ad290 — D3D vertex buffer wrapper: Rust vertex buffers
- [-] `CVertexArray::Shutdown` · 0072fda0 · legions 006ad300 — D3D vertex buffer wrapper: Rust vertex buffers
- [-] `CVertexArray::Lock` · 0072fdf0 · legions 006ad350 — D3D vertex buffer wrapper: Rust vertex buffers
- [ ] `CVertexArray::BuildBoundingBox` · 0072fe40 · legions 006ad3a0 — needs port: vertex array bounds and UnPack (check ac2_mesh decode, uncited)
- [-] `CVertexArray::GetPackSize` · 0072fe90 · legions 006ad3f0 — stream size helper: Rust decodes by slice length, no size pass
- [-] `CVertexArray::Pack` · 0072fed0 · legions 006ad430 — write side of a DAT/admin layout the client never writes; the read side is ported or cited

### (free functions)
- [-] `SetOrigin` · 0072ff90 — vertex builder setter: Rust vertex struct fills the field
- [-] `SetNormal` · 0072ffc0 — vertex builder setter: Rust vertex struct fills the field
- [-] `SetDiffuse` · 0072fff0 — vertex builder setter: Rust vertex struct fills the field
- [-] `SetSpecular` · 00730010 — vertex builder setter: Rust vertex struct fills the field
- [-] `SetVectorS` · 00730030 — vertex builder setter: Rust vertex struct fills the field
- [-] `SetVectorT` · 00730060 — vertex builder setter: Rust vertex struct fills the field
- [-] `SetVectorPt2Light` · 00730090 — vertex builder setter: Rust vertex struct fills the field
- [-] `SetTCPair` · 007300c0 — vertex builder setter: Rust vertex struct fills the field
- [-] `SetMatrixWeight` · 007300f0 — vertex builder setter: Rust vertex struct fills the field

### CVertexArray
- [-] `CVertexArray::CreateVertexArray` · 00730130 · legions 006ad4f0 — D3D vertex buffer wrapper: Rust vertex buffers
- [ ] `CVertexArray::UnPack` · 00730160 · legions 006ad520 — needs port: vertex array bounds and UnPack (check ac2_mesh decode, uncited)

### (free functions)
- [ ] `StreamArbitraryVertex` · 00730280 — needs port (check): arbitrary vertex stream, compare ac2_mesh decode

### CVertexArray
- [-] `CVertexArray::Unlock` · 00730630 · legions 006ad640 — D3D vertex buffer wrapper: Rust vertex buffers

### PSmartArray<LOAD_UVPAIR>
- [ ] `PSmartArray<LOAD_UVPAIR>::StreamPack` · 00730690 — needs port: LOAD_UVPAIR and LOAD_WEIGHTS array stream

### LOAD_UVPAIR
- [-] `LOAD_UVPAIR::LOAD_UVPAIR` · 00730820 — ctor/dtor: Rust struct construction and Drop
- [ ] `LOAD_UVPAIR::UnPack` · 00730830 — needs port: LOAD_UVPAIR and LOAD_WEIGHTS array stream

### PSmartArray<LOAD_WEIGHTS>
- [ ] `PSmartArray<LOAD_WEIGHTS>::StreamPack` · 00730870 — needs port: LOAD_UVPAIR and LOAD_WEIGHTS array stream

### LOAD_WEIGHTS
- [-] `LOAD_WEIGHTS::LOAD_WEIGHTS` · 00730a00 — ctor/dtor: Rust struct construction and Drop

### CLandCell
- [-] `CLandCell::Pack` · 00730a10 · legions 00659b50 — write side of a DAT/admin layout the client never writes; the read side is ported or cited

### (free functions)
- [ ] `LoadArbitraryVertex` · 00730a20 — needs port (check): arbitrary vertex stream, compare ac2_mesh decode

### CVertexArray
- [-] `CVertexArray::FromFileNode` · 00730df0 — text source format (admin tools)

## `portal/shared/src/geometry/waveform.cpp.c` (0/10)

### Waveform
- [ ] `Waveform::SetDefaults` · 00622c40 · legions 005de010 — needs port: waveform defaults and upper bound
- [ ] `Waveform::GetUpperBounds` · 00622c70 · legions 005de040 — needs port: waveform defaults and upper bound
- [~] `Waveform::GetValueForPhase` · 00622cf0 · legions 005de0c0 — tunnel geometry (positions, uvs, index buffer, ring fade) in ac2-render portal/tunnel.rs; not yet: the vertex buffer locks, the portal sounds, the ring-to-alpha mapping (unverified), the outer ring's material
- [~] `Waveform::GetValue` · 00622f10 · legions 005de2e0 — ac2-math waveform; NOISE/PERLIN/FRACTAL return the base (client RNG and PerlinNoise not reproduced)
- [-] `Waveform::Waveform` · 00622f30 · legions 005de300 — ctor/dtor: Rust struct construction and Drop
- [-] `Waveform::Waveform` · 00622f60 · legions 005de330 — ctor/dtor: Rust struct construction and Drop
- [-] `Waveform::Waveform` · 00622fa0 · legions 005de370 — ctor/dtor: Rust struct construction and Drop
- [-] `Waveform::GetTypeString` · 00622fe0 · legions 005de3b0 — text source format (admin tools)
- [-] `Waveform::Parse` · 006230f0 · legions 005de4c0 — text source format (admin tools)
- [-] `Waveform::Output` · 006235e0 · legions 005de9b0 — text source format (admin tools)
