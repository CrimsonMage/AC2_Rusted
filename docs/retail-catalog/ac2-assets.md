# ac2-assets: Render asset codecs: materials, meshes, appearance, images

785 retail functions: 119 done, 14 partial, 231 to do, 421 skipped.
Legend and workflow: [README](README.md).

## `ac2/shared/src/objdesc/gmweeniedesc.cpp.c` (14/62)

### gmWeenieDesc
- [-] `gmWeenieDesc::~gmWeenieDesc` · 00601b00 · legions 005c29e0 — ctor/dtor of plain data; Rust constructs and drops values
- [-] `gmWeenieDesc::Reset` · 00601b30 · legions 005c2a10 — reset to defaults before decode / reset of plain fields; Rust builds fresh values
- [x] `gmWeenieDesc::QueryIntStat` · 00601c80
- [x] `gmWeenieDesc::QueryBoolStat` · 00601cd0
- [x] `gmWeenieDesc::QueryFloatStat` · 00601d00
- [x] `gmWeenieDesc::QueryDIDStat` · 00601d20
- [x] `gmWeenieDesc::QueryIIDStat` · 00601d40
- [x] `gmWeenieDesc::QueryStringInfoStat` · 00601e70
- [ ] `gmWeenieDesc::GetMyPackageID` · 00601e90 — needs port: weenie property bag (gmWeenieDesc): gmWeenieDesc::GetMyPackageID
- [ ] `gmWeenieDesc::SetMyPackageID` · 00601ea0 — needs port: weenie property bag (gmWeenieDesc): gmWeenieDesc::SetMyPackageID (setter; no direct callers in the admin call graph)
- [ ] `gmWeenieDesc::GetEntityDID` · 00601ed0 — needs port: weenie property bag (gmWeenieDesc): gmWeenieDesc::GetEntityDID
- [ ] `gmWeenieDesc::SetEntityDID` · 00601ee0 — needs port: weenie property bag (gmWeenieDesc): gmWeenieDesc::SetEntityDID (setter; no direct callers in the admin call graph)
- [ ] `gmWeenieDesc::AddInt` · 00601f10 — needs port: weenie property bag (gmWeenieDesc): gmWeenieDesc::AddInt (atom insert rule)
- [ ] `gmWeenieDesc::RemoveInt` · 00601f20 — needs port: weenie property bag (gmWeenieDesc): gmWeenieDesc::RemoveInt (atom removal rule)
- [x] `gmWeenieDesc::RemoveInt` · 00601f40 — ac2-weenie/src/qualities.rs (test ~L438, remove_in_weenie_desc) cites gmWeenieDesc::RemoveInt; answers 1 and clears the field
- [ ] `gmWeenieDesc::InqInt` · 006021b0 — needs port: weenie property bag (gmWeenieDesc): gmWeenieDesc::InqInt (property inquiry)
- [x] `gmWeenieDesc::SetInt` · 006023c0
- [ ] `gmWeenieDesc::AdjInt` · 00602590 — needs port: weenie property bag (gmWeenieDesc): gmWeenieDesc::AdjInt
- [ ] `gmWeenieDesc::AddLongInt` · 006026b0 — needs port: weenie property bag (gmWeenieDesc): gmWeenieDesc::AddLongInt (atom insert rule)
- [ ] `gmWeenieDesc::RemoveLongInt` · 006026d0 — needs port: weenie property bag (gmWeenieDesc): gmWeenieDesc::RemoveLongInt (atom removal rule)

### DBObj
- [-] `DBObj::SaveToFile` · 006026f0 — writer for DAT/pack output; the client only reads (tool-only)

### gmWeenieDesc
- [ ] `gmWeenieDesc::RemoveBool` · 00602700 — needs port: weenie property bag (gmWeenieDesc): gmWeenieDesc::RemoveBool (atom removal rule)
- [ ] `gmWeenieDesc::RemoveBool` · 00602720 — needs port: weenie property bag (gmWeenieDesc): gmWeenieDesc::RemoveBool (atom removal rule)
- [ ] `gmWeenieDesc::InqBool` · 006027f0 — needs port: weenie property bag (gmWeenieDesc): gmWeenieDesc::InqBool (property inquiry)
- [x] `gmWeenieDesc::SetBool` · 00602890
- [ ] `gmWeenieDesc::AddFloat` · 00602990 — needs port: weenie property bag (gmWeenieDesc): gmWeenieDesc::AddFloat (atom insert rule)
- [ ] `gmWeenieDesc::RemoveFloat` · 006029a0 — needs port: weenie property bag (gmWeenieDesc): gmWeenieDesc::RemoveFloat (atom removal rule)
- [ ] `gmWeenieDesc::RemoveFloat` · 006029c0 — needs port: weenie property bag (gmWeenieDesc): gmWeenieDesc::RemoveFloat (atom removal rule)
- [ ] `gmWeenieDesc::InqFloat` · 00602a00 — needs port: weenie property bag (gmWeenieDesc): gmWeenieDesc::InqFloat (property inquiry)
- [x] `gmWeenieDesc::SetFloat` · 00602a30
- [ ] `gmWeenieDesc::AdjFloat` · 00602a60 — needs port: weenie property bag (gmWeenieDesc): gmWeenieDesc::AdjFloat
- [ ] `gmWeenieDesc::AddTimestamp` · 00602a90 — needs port: weenie property bag (gmWeenieDesc): gmWeenieDesc::AddTimestamp (atom insert rule)
- [ ] `gmWeenieDesc::RemoveTimestamp` · 00602ab0 — needs port: weenie property bag (gmWeenieDesc): gmWeenieDesc::RemoveTimestamp (atom removal rule)
- [ ] `gmWeenieDesc::AddDataID` · 00602ad0 — needs port: weenie property bag (gmWeenieDesc): gmWeenieDesc::AddDataID (atom insert rule)
- [ ] `gmWeenieDesc::RemoveDataID` · 00602ae0 — needs port: weenie property bag (gmWeenieDesc): gmWeenieDesc::RemoveDataID (atom removal rule)
- [ ] `gmWeenieDesc::RemoveDataID` · 00602b00 — needs port: weenie property bag (gmWeenieDesc): gmWeenieDesc::RemoveDataID (atom removal rule)
- [ ] `gmWeenieDesc::InqDataID` · 00602b30 — needs port: weenie property bag (gmWeenieDesc): gmWeenieDesc::InqDataID (property inquiry)
- [x] `gmWeenieDesc::SetDataID` · 00602b60
- [ ] `gmWeenieDesc::AddInstanceID` · 00602b90 — needs port: weenie property bag (gmWeenieDesc): gmWeenieDesc::AddInstanceID (atom insert rule)
- [ ] `gmWeenieDesc::RemoveInstanceID` · 00602bb0 — needs port: weenie property bag (gmWeenieDesc): gmWeenieDesc::RemoveInstanceID (atom removal rule)
- [ ] `gmWeenieDesc::RemoveInstanceID` · 00602bd0 — needs port: weenie property bag (gmWeenieDesc): gmWeenieDesc::RemoveInstanceID (atom removal rule)
- [ ] `gmWeenieDesc::InqInstanceID` · 00602e40 — needs port: weenie property bag (gmWeenieDesc): gmWeenieDesc::InqInstanceID (property inquiry)
- [x] `gmWeenieDesc::SetInstanceID` · 00603090
- [ ] `gmWeenieDesc::AddString` · 006032d0 — needs port: weenie property bag (gmWeenieDesc): gmWeenieDesc::AddString (atom insert rule)
- [ ] `gmWeenieDesc::AddPosition` · 006032e0 — needs port: weenie property bag (gmWeenieDesc): gmWeenieDesc::AddPosition (atom insert rule)
- [ ] `gmWeenieDesc::RemovePosition` · 006032f0 — needs port: weenie property bag (gmWeenieDesc): gmWeenieDesc::RemovePosition (atom removal rule)
- [ ] `gmWeenieDesc::AddStringInfo` · 00603370 — needs port: weenie property bag (gmWeenieDesc): gmWeenieDesc::AddStringInfo (atom insert rule)
- [ ] `gmWeenieDesc::RemoveStringInfo` · 00603380 — needs port: weenie property bag (gmWeenieDesc): gmWeenieDesc::RemoveStringInfo (atom removal rule)
- [ ] `gmWeenieDesc::RemoveStringInfo` · 006033c0 — needs port: weenie property bag (gmWeenieDesc): gmWeenieDesc::RemoveStringInfo (atom removal rule)
- [ ] `gmWeenieDesc::InqStringInfo` · 00603420 — needs port: weenie property bag (gmWeenieDesc): gmWeenieDesc::InqStringInfo (property inquiry)
- [x] `gmWeenieDesc::SetStringInfo` · 00603470
- [ ] `gmWeenieDesc::AddPackageID` · 006034c0 — needs port: weenie property bag (gmWeenieDesc): gmWeenieDesc::AddPackageID (atom insert rule)
- [ ] `gmWeenieDesc::RemovePackageID` · 006034d0 — needs port: weenie property bag (gmWeenieDesc): gmWeenieDesc::RemovePackageID (atom removal rule)
- [ ] `gmWeenieDesc::GetSubDataIDs` · 006034f0 · legions 005c43d0 — needs port: dependent DIDs of gmWeenieDesc for sub-object resolution (AGENTS.md exception)
- [-] `gmWeenieDesc::gmWeenieDesc` · 00603530 · legions 005c4410 — ctor/dtor of plain data; Rust constructs and drops values
- [-] `gmWeenieDesc::operator=` · 006035a0 · legions 005c4480 — operator of plain data or container; Rust value semantics / derived PartialEq
- [x] `gmWeenieDesc::StreamPack` · 006037c0 · legions 005c46a0
- [-] `gmWeenieDesc::gmWeenieDesc` · 00603b70 · legions 005c4a50 — ctor/dtor of plain data; Rust constructs and drops values
- [-] `gmWeenieDesc::Copy` · 00603ba0 — copy of plain data; Rust clones values
- [-] `gmWeenieDesc::operator=` · 00603bf0 — operator of plain data or container; Rust value semantics / derived PartialEq
- [ ] `gmWeenieDesc::RemoveString` · 00603c00 — needs port: weenie property bag (gmWeenieDesc): gmWeenieDesc::RemoveString (atom removal rule)
- [ ] `gmWeenieDesc::Dump` · 00603c60 · legions 005c4b40 — needs port: weenie property bag (gmWeenieDesc): gmWeenieDesc::Dump

## `portal/client/src/image/graphic.cpp.c` (4/12)

### RenderSurfaceLocal
- [x] `RenderSurfaceLocal::GetGlobalOrLocal` · 00568050 · legions 00537fe0 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor

### Graphic
- [-] `Graphic::Graphic` · 005680a0 · legions 00538030 — ctor/dtor of plain data; Rust constructs and drops values
- [ ] `Graphic::Incorporate` · 005680c0 · legions 00538050 — needs port: UI graphic: Graphic::Incorporate
- [ ] `Graphic::SetToDefaultGraphic` · 005680f0 · legions 00538080 — needs port: UI graphic: Graphic::SetToDefaultGraphic (runtime setter)
- [-] `Graphic::GetWidth` · 005681e0 · legions 00538170 — accessor of a Rust typed field
- [-] `Graphic::GetHeight` · 00568200 · legions 00538190 — accessor of a Rust typed field
- [x] `Graphic::PutImage` · 00568220 · legions 005381b0 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor

### Box2D
- [x] `Box2D::Intersection` · 005682d0 · legions 00538260 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor

### Graphic
- [~] `Graphic::Graphic` · 00568380 · legions 00538310 — ac2-ui: no transitions, timed media, mouse, listeners or dirty rects; A8R8G8B8 blits only; HUD roots from SetupChildren's radar/selection/statusbar/toolbar
- [-] `Graphic::Graphic` · 00568430 · legions 005383c0 — ctor/dtor of plain data; Rust constructs and drops values
- [x] `Graphic::Draw` · 00568470 · legions 00538400 — ac2-ui compose.rs / element.rs set_image
- [ ] `Graphic::InitFromIconLayerDesc` · 00568920 · legions 005388b0 — needs port: UI graphic: Graphic::InitFromIconLayerDesc (initialisation from the DB record)

## `portal/client/src/render/material/layermodifier.cpp.c` (4/46)

### LM_Origin
- [-] `LM_Origin::LM_Origin` · 005d5a80 — ctor/dtor of plain data; Rust constructs and drops values

### LM_Normal
- [-] `LM_Normal::LM_Normal` · 005d5b50 — ctor/dtor of plain data; Rust constructs and drops values

### LM_Origin
- [-] `LM_Origin::GetSize` · 005d5c20 — accessor of a Rust typed field

### LM_Normal
- [-] `LM_Normal::operator=` · 005d5c30 — operator of plain data or container; Rust value semantics / derived PartialEq

### LM_Diffuse
- [-] `LM_Diffuse::LM_Diffuse` · 005d5fa0 — ctor/dtor of plain data; Rust constructs and drops values
- [-] `LM_Diffuse::GetSize` · 005d6000 — accessor of a Rust typed field
- [-] `LM_Diffuse::operator=` · 005d6010 — operator of plain data or container; Rust value semantics / derived PartialEq

### LM_UVTranslate
- [-] `LM_UVTranslate::LM_UVTranslate` · 005d6170 · legions 00599170 — ctor/dtor of plain data; Rust constructs and drops values
- [x] `LM_UVTranslate::ApplyTextureTransform` · 005d61b0 · legions 00599250 — ac2-render object::pass / texture_matrix, ac2-gpu object

### LM_UVRotate
- [-] `LM_UVRotate::LM_UVRotate` · 005d6270 · legions 00599310 — ctor/dtor of plain data; Rust constructs and drops values
- [-] `LM_UVRotate::GetSize` · 005d62a0 · legions 00599350 — accessor of a Rust typed field
- [-] `LM_UVRotate::operator=` · 005d62b0 · legions 00599360 — operator of plain data or container; Rust value semantics / derived PartialEq
- [x] `LM_UVRotate::ApplyTextureTransform` · 005d6310 · legions 005993c0 — ac2-render object::pass / texture_matrix, ac2-gpu object

### LM_UVScale
- [-] `LM_UVScale::LM_UVScale` · 005d6370 · legions 00599420 — ctor/dtor of plain data; Rust constructs and drops values

### LM_UVTranslate
- [-] `LM_UVTranslate::GetSize` · 005d63c0 · legions 00599470 — accessor of a Rust typed field
- [-] `LM_UVTranslate::operator=` · 005d63d0 · legions 005991b0 — operator of plain data or container; Rust value semantics / derived PartialEq

### LM_UVScale
- [x] `LM_UVScale::ApplyTextureTransform` · 005d6470 — ac2-render object::pass / texture_matrix, ac2-gpu object

### LM_UVTransform
- [-] `LM_UVTransform::LM_UVTransform` · 005d64f0 · legions 00599500 — ctor/dtor of plain data; Rust constructs and drops values

### FileNodeName_Bool
- [-] `FileNodeName_Bool::GetType` · 005d6540 · legions 00599550 — text source format (admin tools)

### LM_UVTransform
- [-] `LM_UVTransform::GetSize` · 005d6550 — accessor of a Rust typed field
- [x] `LM_UVTransform::ApplyTextureTransform` · 005d6560 — ac2-render object::pass / texture_matrix, ac2-gpu object

### (free functions)
- [-] `STREAMPACK_SIZEOF` · 005d6590 · legions 005995a0 — pack size helper (STREAMPACK_SIZEOF) for LM modifier records; Rust reads fields directly

### LM_Origin
- [ ] `LM_Origin::StreamPack` · 005d65e0 — needs port: material record: LM_Origin::StreamPack (stream decode)

### LM_Normal
- [-] `LM_Normal::Copy` · 005d6710 — copy of plain data; Rust clones values

### LM_Diffuse
- [-] `LM_Diffuse::Copy` · 005d6750 — copy of plain data; Rust clones values
- [ ] `LM_Diffuse::StreamPack` · 005d6790 — needs port: material record: LM_Diffuse::StreamPack (stream decode)

### LM_UVTranslate
- [-] `LM_UVTranslate::Copy` · 005d6860 · legions 005995f0 — copy of plain data; Rust clones values
- [ ] `LM_UVTranslate::StreamPack` · 005d68c0 · legions 00599750 — needs port: material record: LM_UVTranslate::StreamPack (stream decode)

### LM_UVRotate
- [-] `LM_UVRotate::Copy` · 005d6940 · legions 00599650 — copy of plain data; Rust clones values
- [ ] `LM_UVRotate::StreamPack` · 005d6990 · legions 005996a0 — needs port: material record: LM_UVRotate::StreamPack (stream decode)

### LM_UVScale
- [-] `LM_UVScale::Copy` · 005d6a00 — copy of plain data; Rust clones values

### LM_UVTransform
- [-] `LM_UVTransform::Copy` · 005d6a40 — copy of plain data; Rust clones values
- [ ] `LM_UVTransform::StreamPack` · 005d6ab0 — needs port: material record: LM_UVTransform::StreamPack (stream decode)

### LM_Origin
- [-] `LM_Origin::Copy` · 005d6b20 — copy of plain data; Rust clones values
- [-] `LM_Origin::LoadFromFileNode` · 005d6b60 — text source loader, admin build only
- [-] `LM_Origin::SaveToFileNode` · 005d6f00 — writer for DAT/pack output; the client only reads (tool-only)

### LM_Normal
- [-] `LM_Normal::LoadFromFileNode` · 005d7400 — text source loader, admin build only
- [-] `LM_Normal::SaveToFileNode` · 005d77a0 — writer for DAT/pack output; the client only reads (tool-only)

### LM_Diffuse
- [-] `LM_Diffuse::LoadFromFileNode` · 005d7ca0 — text source loader, admin build only
- [-] `LM_Diffuse::SaveToFileNode` · 005d7ee0 — writer for DAT/pack output; the client only reads (tool-only)

### LM_UVTranslate
- [-] `LM_UVTranslate::LoadFromFileNode` · 005d8180 · legions 005998b0 — text source format (admin tools)
- [-] `LM_UVTranslate::SaveToFileNode` · 005d8370 · legions 00599aa0 — writer for DAT/pack output; the client only reads (tool-only)

### LM_UVRotate
- [-] `LM_UVRotate::LoadFromFileNode` · 005d85a0 · legions 00599cd0 — text source format (admin tools)
- [-] `LM_UVRotate::SaveToFileNode` · 005d8750 · legions 00599e80 — writer for DAT/pack output; the client only reads (tool-only)

### LM_UVScale
- [-] `LM_UVScale::LoadFromFileNode` · 005d8910 — text source loader, admin build only
- [-] `LM_UVScale::SaveToFileNode` · 005d8b00 — writer for DAT/pack output; the client only reads (tool-only)

## `portal/client/src/render/material/layerstage.cpp.c` (0/13)

### LayerStage
- [-] `LayerStage::ReleaseSubObjects` · 005be2e0 · legions 00587290 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)
- [ ] `LayerStage::StreamPack` · 005be300 · legions 005872b0 — needs port: material record: LayerStage::StreamPack (stream decode)
- [ ] `LayerStage::GetDIDsForFilenames` · 005be460 — needs port: dependent DIDs of LayerStage for sub-object resolution (AGENTS.md exception)
- [-] `LayerStage::Begin` · 005be4b0 · legions 00587410 — reset to defaults before decode / reset of plain fields; Rust builds fresh values
- [-] `LayerStage::CopyInto` · 005be550 · legions 005874b0 — copy of plain data; Rust clones values
- [-] `LayerStage::Destroy` · 005be610 · legions 00587570 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)
- [ ] `LayerStage::SetTexture` · 005be620 · legions 00587580 — needs port: material record: LayerStage::SetTexture (runtime setter)
- [ ] `LayerStage::SetTexture` · 005be6c0 · legions 00587600 — needs port: material record: LayerStage::SetTexture (runtime setter)
- [-] `LayerStage::LayerStage` · 005be760 · legions 005876a0 — ctor/dtor of plain data; Rust constructs and drops values
- [-] `LayerStage::Copy` · 005be790 · legions 005876d0 — copy of plain data; Rust clones values
- [-] `LayerStage::GetSubObjects` · 005be7e0 · legions 00587720 — retail DBO object-graph walk for the cache; Arc replaces it
- [-] `LayerStage::LoadFromFileNode` · 005be850 — text source loader, admin build only
- [-] `LayerStage::SaveToFileNode` · 005bf7a0 — writer for DAT/pack output; the client only reads (tool-only)

## `portal/client/src/render/material/materialinstance.cpp.c` (1/23)

### MaterialInstance
- [ ] `MaterialInstance::Refresh` · 005ba670 — needs port: material record: MaterialInstance::Refresh
- [ ] `MaterialInstance::SetTemplateMaterial` · 005ba680 · legions 00584670 — needs port: material record: MaterialInstance::SetTemplateMaterial (runtime setter)
- [ ] `MaterialInstance::RestoreOriginalTemplateMaterial` · 005ba6c0 · legions 005846b0 — needs port: material record: MaterialInstance::RestoreOriginalTemplateMaterial
- [ ] `MaterialInstance::GetSubDataIDs` · 005ba730 · legions 00584710 — needs port: dependent DIDs of MaterialInstance for sub-object resolution (AGENTS.md exception)
- [-] `MaterialInstance::ReleaseSubObjects` · 005ba780 · legions 00584760 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)
- [ ] `MaterialInstance::GetDIDsForFilenames` · 005ba800 — needs port: dependent DIDs of MaterialInstance for sub-object resolution (AGENTS.md exception)
- [x] `MaterialInstance::RefreshMaterial` · 005ba8e0 · legions 005847e0 — ac2-assets/src/ac2_material_instance.rs (RefreshMaterial): modifiers applied to a copy of the template
- [-] `MaterialInstance::ReleaseModifier` · 005ba9e0 · legions 005848e0 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)
- [-] `MaterialInstance::Begin` · 005baa50 · legions 00584950 — reset to defaults before decode / reset of plain fields; Rust builds fresh values
- [-] `MaterialInstance::CopyInto` · 005baae0 — copy of plain data; Rust clones values
- [-] `MaterialInstance::GetSubObjects` · 005bad00 — retail DBO object-graph walk for the cache; Arc replaces it
- [ ] `MaterialInstance::StreamPack` · 005badf0 — needs port: material record: MaterialInstance::StreamPack (stream decode)
- [-] `MaterialInstance::LoadFromFileNode` · 005baf20 — text source loader, admin build only
- [-] `MaterialInstance::ToFileNode` · 005bb140 — text source format (admin tools)
- [ ] `MaterialInstance::InsertModifier` · 005bb390 · legions 00584de0 — needs port: material record: MaterialInstance::InsertModifier
- [-] `MaterialInstance::MaterialInstance` · 005bb3d0 · legions 00584e20 — ctor/dtor of plain data; Rust constructs and drops values
- [-] `MaterialInstance::GetDBOType` · 005bb430 — DBO type tag; Rust uses typed resource ids
- [ ] `MaterialInstance::End` · 005bb440 · legions 00584e90 — needs port: material record: MaterialInstance::End (post-decode finaliser)
- [-] `MaterialInstance::Destroy` · 005bb4c0 · legions 00584f10 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)
- [-] `MaterialInstance::FromFileNode` · 005bb4d0 — text source format (admin tools)
- [ ] `MaterialInstance::ReadInstanceFilename` · 005bb5e0 — needs port: material record: MaterialInstance::ReadInstanceFilename
- [-] `MaterialInstance::Allocate` · 005bb7c0 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)
- [-] `MaterialInstance::~MaterialInstance` · 005bb7d0 · legions 00584f30 — ctor/dtor of plain data; Rust constructs and drops values

## `portal/client/src/render/material/materiallayer.cpp.c` (3/20)

### MaterialLayer
- [-] `MaterialLayer::Begin` · 005cb660 · legions 00592640 — reset to defaults before decode / reset of plain fields; Rust builds fresh values
- [x] `MaterialLayer::BuildCompilerFlags` · 005cb7e0 · legions 005927c0 — ac2-render object::pass / texture_matrix, ac2-gpu object

### (free functions)
- [-] `STREAMPACK_SIZEOF` · 005cb8b0 · legions 00592890 — pack size helper (STREAMPACK_SIZEOF); Rust reads fields directly

### MaterialLayer
- [ ] `MaterialLayer::End` · 005cb9b0 · legions 005928e0 — needs port: material record: MaterialLayer::End (post-decode finaliser)
- [-] `MaterialLayer::Destroy` · 005cbad0 · legions 00592a00 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)
- [~] `MaterialLayer::GetVertexShader` · 005cbae0 · legions 00592a10 — fragment.dll programs in object.wgsl: unlit, lit, lit vertex diffuse, per-pixel (sun); point-light variants not yet

### MaterialLayer::ShaderInfo
- [-] `MaterialLayer::ShaderInfo::ShaderInfo` · 005cbbd0 — ctor/dtor of plain data; Rust constructs and drops values

### MaterialLayer
- [x] `MaterialLayer::BuildPixelShader` · 005cbbe0 · legions 00592b10 — object::parse_pixel_shader + ac2-gpu pixel_program (ps.1.1 to WGSL)

### SmartArray<MaterialLayer::ShaderInfo>
- [-] `SmartArray<MaterialLayer::ShaderInfo>::~SmartArray<MaterialLayer::ShaderInfo>` · 005cbcb0 · legions 00592be0 — ctor/dtor of plain data; Rust constructs and drops values

### SmartArray<LScapeFarBlockEdge>
- [-] `SmartArray<LScapeFarBlockEdge>::operator=` · 005cbcf0 · legions 00592c20 — operator of plain data or container; Rust value semantics / derived PartialEq

### MaterialLayer
- [-] `MaterialLayer::MaterialLayer` · 005cbd70 · legions 00592ca0 — ctor/dtor of plain data; Rust constructs and drops values
- [-] `MaterialLayer::CopyInto` · 005cbdc0 · legions 00592cf0 — copy of plain data; Rust clones values
- [x] `MaterialLayer::StreamPack` · 005cc0b0 · legions 00592fe0 — ac2-assets/src/ac2_material.rs: MaterialLayer::StreamPack stage decode (4-stage limit, LM modifiers)
- [-] `MaterialLayer::LoadFromFileNode` · 005cc6b0 — text source loader, admin build only
- [-] `MaterialLayer::SaveToFileNode` · 005ce650 — writer for DAT/pack output; the client only reads (tool-only)
- [ ] `MaterialLayer::BuildVertexShader` · 005cfef0 · legions 005935e0 — needs port: material record: MaterialLayer::BuildVertexShader (derived bounding data)
- [-] `MaterialLayer::Copy` · 005d0320 · legions 00593a10 — copy of plain data; Rust clones values

### SmartArray<MaterialLayer::ShaderInfo>
- [ ] `SmartArray<MaterialLayer::ShaderInfo>::AddToEnd` · 005d0360 · legions 00593a50 — needs port: material record: SmartArray::AddToEnd (atom insert rule)

### MaterialLayer
- [~] `MaterialLayer::BuildVertexShaders` · 005d03d0 · legions 00593ac0 — fragment.dll programs in object.wgsl: unlit, lit, lit vertex diffuse, per-pixel (sun); point-light variants not yet
- [~] `MaterialLayer::PrepareShaders` · 005d06e0 · legions 00593dd0 — fragment.dll programs in object.wgsl: unlit, lit, lit vertex diffuse, per-pixel (sun); point-light variants not yet

## `portal/client/src/render/material/materialmodifier.cpp.c` (0/12)

### MaterialModifier
- [-] `MaterialModifier::MaterialModifier` · 005b8fb0 · legions 00583a30 — ctor/dtor of plain data; Rust constructs and drops values
- [-] `MaterialModifier::GetDBOType` · 005b8fe0 — DBO type tag; Rust uses typed resource ids
- [ ] `MaterialModifier::End` · 005b8ff0 · legions 00583a70 — needs port: material record: MaterialModifier::End (post-decode finaliser)
- [-] `MaterialModifier::Destroy` · 005b9050 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)
- [ ] `MaterialModifier::GetSubDataIDs` · 005b9060 · legions 00583ae0 — needs port: dependent DIDs of MaterialModifier for sub-object resolution (AGENTS.md exception)
- [-] `MaterialModifier::Allocate` · 005b90b0 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)
- [-] `MaterialModifier::~MaterialModifier` · 005b90c0 · legions 00583b40 — ctor/dtor of plain data; Rust constructs and drops values
- [-] `MaterialModifier::CopyInto` · 005b9130 — copy of plain data; Rust clones values
- [ ] `MaterialModifier::StreamPack` · 005b91b0 · legions 00583c30 — needs port: material record: MaterialModifier::StreamPack (stream decode)
- [-] `MaterialModifier::LoadFromFileNode` · 005b9290 — text source loader, admin build only
- [-] `MaterialModifier::ToFileNode` · 005b94f0 — text source format (admin tools)
- [-] `MaterialModifier::FromFileNode` · 005b9580 — text source format (admin tools)

## `portal/client/src/render/material/materialproperty.cpp.c` (1/28)

### LM_Origin
- [ ] `LM_Origin::SetYScale` · 005da1a0 · legions 0059b890 — needs port: material record: LM_Origin::SetYScale (runtime setter)
- [ ] `LM_Origin::SetZScale` · 005da210 · legions 0059b900 — needs port: material record: LM_Origin::SetZScale (runtime setter)
- [ ] `LM_Origin::SetOriginPhase` · 005da280 · legions 0059b970 — needs port: material record: LM_Origin::SetOriginPhase (runtime setter)
- [ ] `LM_Origin::SetNormalPhase` · 005da2f0 · legions 0059b9e0 — needs port: material record: LM_Origin::SetNormalPhase (runtime setter)
- [ ] `LM_Origin::SetXTranslate` · 005da360 · legions 0059ba50 — needs port: material record: LM_Origin::SetXTranslate (runtime setter)
- [ ] `LM_Origin::SetYTranslate` · 005da3b0 · legions 0059baa0 — needs port: material record: LM_Origin::SetYTranslate (runtime setter)
- [ ] `LM_Origin::SetZTranslate` · 005da400 · legions 0059baf0 — needs port: material record: LM_Origin::SetZTranslate (runtime setter)
- [ ] `LM_Origin::SetXScale` · 005da450 · legions 0059b820 — needs port: material record: LM_Origin::SetXScale (runtime setter)

### LM_UVTranslate
- [ ] `LM_UVTranslate::SetUTranslate` · 005da4c0 · legions 00557790 — needs port: material record: LM_UVTranslate::SetUTranslate (runtime setter)

### MaterialLayer
- [ ] `MaterialLayer::SetAlphaTestRef` · 005da510 · legions 0059bb40 — needs port: material record: MaterialLayer::SetAlphaTestRef (runtime setter)
- [ ] `MaterialLayer::SetSpecularPower` · 005da560 · legions 0059bb90 — needs port: material record: MaterialLayer::SetSpecularPower (runtime setter)

### MaterialProperty
- [ ] `MaterialProperty::SetWaveformData` · 005da5d0 — needs port: material record: MaterialProperty::SetWaveformData (runtime setter)
- [ ] `MaterialProperty::SetColorData` · 005da650 — needs port: material record: MaterialProperty::SetColorData (runtime setter)
- [ ] `MaterialProperty::SetBoolData` · 005da6a0 — needs port: material record: MaterialProperty::SetBoolData (runtime setter)
- [ ] `MaterialProperty::GetDataTypeForID` · 005da6e0 — needs port: MaterialProperty::GetDataTypeForID

### (free functions)
- [-] `STREAMPACK_BIT32` · 005da720 · legions 004f9b20 — generic 32-bit field pack helper (STREAMPACK_BIT32); ArchiveCursor reads primitives

### MaterialProperty
- [ ] `MaterialProperty::GetDIDsForFilenames` · 005da750 — needs port: dependent DIDs of MaterialProperty for sub-object resolution (AGENTS.md exception)
- [x] `MaterialProperty::Apply` · 005da830 · legions 0059bc00 — ac2-assets/src/ac2_material_instance.rs: MaterialProperty::Apply (RMFieldType handling)
- [-] `MaterialProperty::Begin` · 005dabb0 · legions 0059bf80 — reset to defaults before decode / reset of plain fields; Rust builds fresh values
- [ ] `MaterialProperty::End` · 005dac50 · legions 0059c020 — needs port: material record: MaterialProperty::End (post-decode finaliser)
- [-] `MaterialProperty::Destroy` · 005dacf0 · legions 0059c0c0 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)
- [-] `MaterialProperty::MaterialProperty` · 005dad00 · legions 0059c0d0 — ctor/dtor of plain data; Rust constructs and drops values
- [-] `MaterialProperty::~MaterialProperty` · 005dad50 · legions 0059c120 — ctor/dtor of plain data; Rust constructs and drops values
- [-] `MaterialProperty::CopyInto` · 005dadb0 · legions 0059c180 — copy of plain data; Rust clones values
- [ ] `MaterialProperty::StreamPack` · 005daf10 · legions 0059c2e0 — needs port: material record: MaterialProperty::StreamPack (stream decode)
- [-] `MaterialProperty::LoadFromFileNode` · 005db0b0 — text source loader, admin build only
- [-] `MaterialProperty::SaveToFileNode` · 005db420 — writer for DAT/pack output; the client only reads (tool-only)
- [-] `MaterialProperty::Copy` · 005db680 · legions 0059c4a0 — copy of plain data; Rust clones values

## `portal/client/src/render/material/rendermaterial.cpp.c` (0/17)

### PStringBase<char>
- [-] `PStringBase<char>::operator[]` · 005b9690 — operator of plain data or container; Rust value semantics / derived PartialEq

### RenderMaterial
- [-] `RenderMaterial::RenderMaterial` · 005b96b0 · legions 00583d10 — ctor/dtor of plain data; Rust constructs and drops values
- [-] `RenderMaterial::GetDBOType` · 005b9700 — DBO type tag; Rust uses typed resource ids
- [ ] `RenderMaterial::GetSubDataIDs` · 005b9710 — needs port: dependent DIDs of RenderMaterial for sub-object resolution (AGENTS.md exception)
- [~] `RenderMaterial::GetSubObjects` · 005b97c0 — hardware-class layer filter (object::layer_drawn, class 2 default)
- [-] `RenderMaterial::ReleaseSubObjects` · 005b9ad0 · legions 00584130 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)
- [ ] `RenderMaterial::GetDIDsForFilenames` · 005b9b50 — needs port: dependent DIDs of RenderMaterial for sub-object resolution (AGENTS.md exception)
- [-] `RenderMaterial::Allocate` · 005b9bf0 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)
- [ ] `RenderMaterial::End` · 005b9c00 · legions 005841c0 — needs port: material record: RenderMaterial::End (post-decode finaliser)
- [-] `RenderMaterial::Destroy` · 005b9c70 · legions 00584230 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)
- [-] `RenderMaterial::CheckOutputField` · 005b9c90 · legions 00584250 — text source format (admin tools)
- [-] `RenderMaterial::CopyInto` · 005b9d50 — copy of plain data; Rust clones values
- [ ] `RenderMaterial::StreamPack` · 005b9e00 · legions 005843c0 — needs port: material record: RenderMaterial::StreamPack (stream decode)
- [-] `RenderMaterial::LoadFromFileNode` · 005b9f30 — text source loader, admin build only
- [-] `RenderMaterial::ToFileNode` · 005ba250 — text source format (admin tools)
- [-] `RenderMaterial::FromFileNode` · 005ba430 — text source format (admin tools)
- [-] `RenderMaterial::CheckParseField` · 005ba540 · legions 00584540 — text source format (admin tools)

## `portal/preproc/src/empgen/empgen.cpp.c` (0/27)

### Position
- [-] `Position::get_origin` · 006735b0 · legions 0061db70 — asset preprocessing (admin tools)

### EmpGen
- [-] `EmpGen::MakeFilenameAbsolute` · 006735c0 · legions 0061db80 — asset preprocessing (admin tools)

### PStringBaseIter_Common<char>
- [-] `PStringBaseIter_Common<char>::FindSubStringIter` · 006735d0 · legions 0061db90 — asset preprocessing (admin tools)

### EmpGenIOStreamUtilities
- [-] `EmpGenIOStreamUtilities::MaybeSpewComment` · 00673620 · legions 0061dbe0 — asset preprocessing (admin tools)

### EmpGen
- [-] `EmpGen::Initialize` · 00673720 · legions 0061dce0 — asset preprocessing (admin tools)

### PStringBaseIter_Common<char>
- [-] `PStringBaseIter_Common<char>::FindSubString` · 00673d60 · legions 0061e320 — asset preprocessing (admin tools)

### EmpGen
- [-] `EmpGen::EmpGen` · 00673e40 · legions 0061e400 — asset preprocessing (admin tools)
- [-] `EmpGen::~EmpGen` · 00674090 · legions 0061e650 — asset preprocessing (admin tools)
- [-] `EmpGen::LoadEmp` · 006743b0 · legions 0061e970 — asset preprocessing (admin tools)
- [-] `EmpGen::GetWslTypeName` · 00674540 · legions 0061eae0 — asset preprocessing (admin tools)

### EmpGenIOStreamUtilities
- [-] `EmpGenIOStreamUtilities::SpewTypeHeaderContents` · 006746d0 · legions 0061ec70 — asset preprocessing (admin tools)
- [-] `EmpGenIOStreamUtilities::SpewWTypeHeaderContents` · 00674a00 · legions 0061efa0 — asset preprocessing (admin tools)

### EmpGen
- [-] `EmpGen::GetDefineSymbol` · 00674cd0 · legions 0061f270 — asset preprocessing (admin tools)
- [-] `EmpGen::FindIncludeName` · 00674dc0 · legions 0061f360 — asset preprocessing (admin tools)
- [-] `EmpGen::GenerateSymbol` · 00674f00 · legions 0061f480 — asset preprocessing (admin tools)

### EmpGenIOStreamUtilities
- [-] `EmpGenIOStreamUtilities::SpewListHeaderContents` · 006750f0 · legions 0061f670 — asset preprocessing (admin tools)

### EmpGen
- [-] `EmpGen::GenerateWTypeHeader` · 00675400 — asset preprocessing (admin tools)
- [-] `EmpGen::UpdateEMP` · 00675520 — asset preprocessing (admin tools)

### EmpGenIOStreamUtilities
- [-] `EmpGenIOStreamUtilities::SpewIncludeHeader` · 006757d0 · legions 0061fd50 — asset preprocessing (admin tools)
- [-] `EmpGenIOStreamUtilities::SpewIncludeFooter` · 00675890 · legions 0061fe10 — asset preprocessing (admin tools)

### EmpGen
- [-] `EmpGen::AddEnum` · 00675910 — asset preprocessing (admin tools)
- [-] `EmpGen::AddEnum` · 006759a0 · legions 0061fe90 — asset preprocessing (admin tools)
- [-] `EmpGen::GenerateTypeHeader` · 00675a30 — asset preprocessing (admin tools)
- [-] `EmpGen::GenerateListHeader` · 00675b50 — asset preprocessing (admin tools)
- [-] `EmpGen::FindWslImportName` · 00675c80 · legions 00620170 — asset preprocessing (admin tools)

### EmpGenIOStreamUtilities
- [-] `EmpGenIOStreamUtilities::SpewWListHeaderContents` · 00675eb0 · legions 006203a0 — asset preprocessing (admin tools)

### EmpGen
- [-] `EmpGen::GenerateWListHeader` · 00676170 — asset preprocessing (admin tools)

## `portal/preproc/src/empgen/empgenparse.cpp.c` (0/37)

### EmpGenParse
- [-] `EmpGenParse::EMP_HandleAllCode` · 00676290 — asset preprocessing (admin tools)
- [-] `EmpGenParse::EMP_HandleBitfield` · 006762b0 — asset preprocessing (admin tools)
- [-] `EmpGenParse::SE_HandleNoCode` · 006762e0 — asset preprocessing (admin tools)
- [-] `EmpGenParse::SE_HandleNoMap` · 00676300 — asset preprocessing (admin tools)
- [-] `EmpGenParse::SE_HandleClientOnly` · 00676310 — asset preprocessing (admin tools)
- [-] `EmpGenParse::SE_HandleServerOnly` · 00676330 — asset preprocessing (admin tools)
- [-] `EmpGenParse::SE_HandleInternal` · 00676350 — asset preprocessing (admin tools)

### PEnum
- [-] `PEnum::PEnum` · 00676360 · legions 00620850 — asset preprocessing (admin tools)
- [-] `PEnum::~PEnum` · 006763c0 · legions 006208b0 — asset preprocessing (admin tools)

### EmpGenParse
- [-] `EmpGenParse::EMP_HandleTypeHeader` · 00676440 — asset preprocessing (admin tools)
- [-] `EmpGenParse::EMP_HandleListHeader` · 00676530 — asset preprocessing (admin tools)
- [-] `EmpGenParse::EMP_HandleWTypeHeader` · 00676620 — asset preprocessing (admin tools)
- [-] `EmpGenParse::EMP_HandleWListHeader` · 00676710 — asset preprocessing (admin tools)
- [-] `EmpGenParse::EMP_HandleSymbolSuffix` · 00676800 · legions 00620cf0 — asset preprocessing (admin tools)
- [-] `EmpGenParse::EMP_HandleSymbolPrefix` · 00676890 · legions 00620d80 — asset preprocessing (admin tools)
- [-] `EmpGenParse::EMP_HandleFSymbolSuffix` · 00676920 · legions 00620e10 — asset preprocessing (admin tools)
- [-] `EmpGenParse::EMP_HandleFSymbolPrefix` · 006769b0 · legions 00620ea0 — asset preprocessing (admin tools)
- [-] `EmpGenParse::EMP_HandleNameSpace` · 00676a40 — asset preprocessing (admin tools)
- [-] `EmpGenParse::SE_HandleCode` · 00676b90 · legions 00621080 — asset preprocessing (admin tools)
- [-] `EmpGenParse::SE_HandleFundamental` · 00676c30 · legions 00621120 — asset preprocessing (admin tools)
- [-] `EmpGenParse::SE_HandleFileName` · 00676cd0 · legions 006211c0 — asset preprocessing (admin tools)
- [-] `EmpGenParse::EMP_Setup` · 00676db0 · legions 006213a0 — asset preprocessing (admin tools)
- [-] `EmpGenParse::EMP_HandleBaseEmp` · 00677150 — asset preprocessing (admin tools)
- [-] `EmpGenParse::EMP_HandleTypeDef` · 006775b0 — asset preprocessing (admin tools)
- [-] `EmpGenParse::EMP_HandleExistingTypeDef` · 00677850 · legions 00621e40 — asset preprocessing (admin tools)
- [-] `EmpGenParse::EMP_HandleDefaultInvalid` · 00677970 — asset preprocessing (admin tools)
- [-] `EmpGenParse::EMP_HandleSequential` · 00677b00 · legions 006220f0 — asset preprocessing (admin tools)
- [-] `EmpGenParse::SE_HandleForce` · 00677b80 · legions 00622170 — asset preprocessing (admin tools)
- [-] `EmpGenParse::SE_HandleData` · 00677be0 — asset preprocessing (admin tools)

### SmartArray<PEnum>
- [-] `SmartArray<PEnum>::~SmartArray<PEnum>` · 00677c80 · legions 00622270 — asset preprocessing (admin tools)
- [-] `SmartArray<PEnum>::AddToEnd` · 00677db0 · legions 00622400 — asset preprocessing (admin tools)

### EmpGenParse
- [-] `EmpGenParse::InitPEnumParser` · 00677e00 · legions 00622450 — asset preprocessing (admin tools)
- [-] `EmpGenParse::ParsePEnum` · 00677f40 · legions 00622590 — asset preprocessing (admin tools)
- [-] `EmpGenParse::EMP_HandleEnums` · 00678040 — asset preprocessing (admin tools)
- [-] `EmpGenParse::EMP_HandleGeneratedMappings` · 00678550 — asset preprocessing (admin tools)
- [-] `EmpGenParse::InitEmpParser` · 00678650 · legions 00622ca0 — asset preprocessing (admin tools)
- [-] `EmpGenParse::ParseEmp` · 006788d0 · legions 00622f20 — asset preprocessing (admin tools)

## `portal/preproc/src/imagedec/ilbmdecoder.cpp.c` (0/7)

### ILBMDecoder
- [-] `ILBMDecoder::ILBMDecoder` · 00733d30 — image import in the preprocessor (admin tools); client reads DAT textures
- [-] `ILBMDecoder::readILBMHeader` · 00733d50 — image import in the preprocessor (admin tools); client reads DAT textures
- [-] `ILBMDecoder::expandCompressedLine` · 00733f70 — image import in the preprocessor (admin tools); client reads DAT textures
- [-] `ILBMDecoder::ReadHeader` · 00734030 — image import in the preprocessor (admin tools); client reads DAT textures
- [-] `ILBMDecoder::expandBODY` · 00734080 — image import in the preprocessor (admin tools); client reads DAT textures
- [-] `ILBMDecoder::readData` · 007342b0 — image import in the preprocessor (admin tools); client reads DAT textures
- [-] `ILBMDecoder::Decode` · 00734380 — image import in the preprocessor (admin tools); client reads DAT textures

## `portal/preproc/src/imagedec/imagedecoder.cpp.c` (0/7)

### ImageDecoder
- [-] `ImageDecoder::ImageDecoder` · 00733a20 — image import in the preprocessor (admin tools); client reads DAT textures
- [-] `ImageDecoder::InitInternal` · 00733a60 — image import in the preprocessor (admin tools); client reads DAT textures
- [-] `ImageDecoder::GetLine` · 00733ae0 — image import in the preprocessor (admin tools); client reads DAT textures
- [-] `ImageDecoder::SetRGBAPixel` · 00733af0 — image import in the preprocessor (admin tools); client reads DAT textures
- [-] `ImageDecoder::Load` · 00733b80 — image import in the preprocessor (admin tools); client reads DAT textures
- [-] `ImageDecoder::Cleanup` · 00733bc0 — image import in the preprocessor (admin tools); client reads DAT textures
- [-] `ImageDecoder::Init` · 00733c10 — image import in the preprocessor (admin tools); client reads DAT textures

## `portal/preproc/src/imagedec/tgadecoder.cpp.c` (0/10)

### TGADecoder
- [-] `TGADecoder::TGADecoder` · 007343a0 — image import in the preprocessor (admin tools); client reads DAT textures
- [-] `TGADecoder::readTGAHeader` · 007343c0 — image import in the preprocessor (admin tools); client reads DAT textures
- [-] `TGADecoder::readPixel32` · 00734500 — image import in the preprocessor (admin tools); client reads DAT textures
- [-] `TGADecoder::ReadHeader` · 00734660 — image import in the preprocessor (admin tools); client reads DAT textures
- [-] `TGADecoder::readPalette` · 00734740 — image import in the preprocessor (admin tools); client reads DAT textures
- [-] `TGADecoder::expandUncompressedLine` · 00734790 — image import in the preprocessor (admin tools); client reads DAT textures
- [-] `TGADecoder::expandCompressedLine` · 007347e0 — image import in the preprocessor (admin tools); client reads DAT textures
- [-] `TGADecoder::readData` · 007348e0 — image import in the preprocessor (admin tools); client reads DAT textures
- [-] `TGADecoder::readImage` · 00734990 — image import in the preprocessor (admin tools); client reads DAT textures
- [-] `TGADecoder::Decode` · 007349f0 — image import in the preprocessor (admin tools); client reads DAT textures

## `portal/shared/src/entitydesc/entitydesc.cpp.c` (7/41)

### LM_UVScale
- [-] `LM_UVScale::GetType` · 00615d90 · legions 005d3ac0 — accessor of a Rust typed field

### FileNodeName_InstanceID
- [-] `FileNodeName_InstanceID::Clone` · 00615da0 · legions 005d3ad0 — text source format (admin tools)

### PFileNode
- [ ] `PFileNode::SetNameAsInstanceID` · 00615dd0 · legions 005d3b00 — needs port: file-node tree accessor SetNameAsInstanceID (binary reader in ac2-dat)
- [ ] `PFileNode::MakeSubNodeAsInstanceID` · 00615e20 — needs port: file-node tree accessor MakeSubNodeAsInstanceID (binary reader in ac2-dat)

### EntityDesc
- [ ] `EntityDesc::SetProperties` · 00615e50 · legions 005d3b50 — needs port: entity description: EntityDesc::SetProperties (runtime setter)
- [ ] `EntityDesc::InqBool` · 00615e70 · legions 005d3b70 — needs port: entity description: EntityDesc::InqBool (property inquiry)
- [x] `EntityDesc::InqInteger` · 00615ea0 · legions 005d3ba0 — Integer property of an entity link (name, out long); a property of another type answers 0 (entity_desc.rs). SaleProfile::ComputeMaxStackSize reads its max stack size through it; the sell's rig test runs it.
- [ ] `EntityDesc::InqFloat` · 00615ed0 · legions 005d3bd0 — needs port: entity description: EntityDesc::InqFloat (property inquiry)
- [ ] `EntityDesc::InqVector` · 00615f00 · legions 005d3c00 — needs port: entity description: EntityDesc::InqVector (property inquiry)
- [ ] `EntityDesc::InqEnum` · 00615f30 · legions 005d3c30 — needs port: entity description: EntityDesc::InqEnum (property inquiry)
- [x] `EntityDesc::InqDataID` · 00615f60 · legions 005d3c60 — ac2-script/src/glue/entity_desc.rs inq_data_id cites EntityDesc::InqDataID
- [ ] `EntityDesc::InqPackageID` · 00615f90 · legions 005d3c90 — needs port: entity description: EntityDesc::InqPackageID (property inquiry)
- [x] `EntityDesc::InqStringInfo` · 00615fc0 · legions 005d3cc0 — ac2-script/src/glue/entity_desc.rs cites EntityDesc::InqStringInfo (StringInfo property read)
- [ ] `EntityDesc::GetSubDataIDs` · 00615ff0 · legions 005d3cf0 — needs port: dependent DIDs of EntityDesc for sub-object resolution (AGENTS.md exception)
- [ ] `EntityDesc::SetDataFileName` · 00616030 — needs port: entity description: EntityDesc::SetDataFileName (runtime setter)
- [-] `EntityDesc::GetPackBitfield` · 00616080 · legions 005d3d30 — writer for DAT/pack output; the client only reads (tool-only)

### (free functions)
- [ ] `STREAMPACK_STREAMPACKOBJ` · 006161b0 · legions 005d3e60 — needs port: generic StreamPack op for int STREAMPACK_STREAMPACKOBJ(STREAMTYPE op, Frame * obj, void * * addr, uint * size)

### PFileNode
- [ ] `PFileNode::GetNameAsInstanceID` · 00616200 — needs port: file-node tree accessor GetNameAsInstanceID (binary reader in ac2-dat)

### EntityDesc
- [-] `EntityDesc::EntityDesc` · 006162d0 · legions 005d3eb0 — ctor/dtor of plain data; Rust constructs and drops values
- [-] `EntityDesc::GetDBOType` · 00616390 — DBO type tag; Rust uses typed resource ids
- [-] `EntityDesc::operator=` · 006163a0 · legions 005d3f80 — operator of plain data or container; Rust value semantics / derived PartialEq
- [-] `EntityDesc::~EntityDesc` · 006164c0 · legions 005d40a0 — ctor/dtor of plain data; Rust constructs and drops values
- [ ] `EntityDesc::InqDataFile` · 00616520 — needs port: entity description: EntityDesc::InqDataFile (property inquiry)
- [ ] `EntityDesc::InqDataFileW` · 006165d0 · legions 005d41b0 — needs port: entity description: EntityDesc::InqDataFileW (property inquiry)
- [ ] `EntityDesc::InqDataFilePath` · 00616690 — needs port: entity description: EntityDesc::InqDataFilePath (property inquiry)
- [ ] `EntityDesc::InqDataFilePathW` · 00616720 · legions 005d4300 — needs port: entity description: EntityDesc::InqDataFilePathW (property inquiry)
- [ ] `EntityDesc::InqStringW` · 006167d0 · legions 005d43b0 — needs port: entity description: EntityDesc::InqStringW (property inquiry)
- [-] `EntityDesc::CopyInto` · 00616800 · legions 005d43e0 — copy of plain data; Rust clones values
- [-] `EntityDesc::Destroy` · 00616820 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)
- [ ] `EntityDesc::GetRootPackageID` · 00616960 · legions 005d4540 — needs port: entity description: EntityDesc::GetRootPackageID
- [x] `EntityDesc::GetTotalProperties` · 006169c0 · legions 005d45a0
- [x] `EntityDesc::GetVisualDescID` · 00616a50 · legions 005d4610 — ac2-assets ac2_static_model.rs resolve_ac2_entity_visual: an EntityDesc (0x47) follows its template (m_dataID) to the VisualDesc or CSetup it draws; ac2-client world_data.rs load_body_data resolves FX object DIDs through it on the loader thread (real-data test every_fx_objects_entity_resolves_to_a_drawable_visual)
- [x] `EntityDesc::InqEntityDescFromDataID` · 00616ac0 · legions 005d4680 — ac2-script/src/glue/entity_desc.rs cites EntityDesc::InqEntityDescFromDataID
- [-] `EntityDesc::Allocate` · 00616b20 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)
- [-] `EntityDesc::EntityDesc` · 00616b30 · legions 005d46f0 — ctor/dtor of plain data; Rust constructs and drops values
- [-] `EntityDesc::FromFileNode` · 00616bb0 — text source format (admin tools)
- [-] `EntityDesc::ToFileNode` · 00616fa0 — text source format (admin tools)
- [ ] `EntityDesc::GetEntityNameFromDataIDW` · 00617320 · legions 005d4770 — needs port: entity description: EntityDesc::GetEntityNameFromDataIDW
- [~] `EntityDesc::StreamPack` · 00617370 · legions 005d47c0 — unpack only, in ac2-net protocol::property / fx (types from a PropertyTypes resolver; Legions BaseProperty::StreamPack 0x0064B4E0 instruction-identical)
- [x] `EntityDesc::GetDataIDFromEntityNameW` · 006174d0 · legions 005d4920 — ac2-script/src/glue/enums.rs get_data_id_from_entity_name_w cites EntityDesc::GetDataIDFromEntityNameW
- [ ] `EntityDesc::InqEntityDescFromNameW` · 00617520 · legions 005d4970 — needs port: entity description: EntityDesc::InqEntityDescFromNameW (property inquiry)

## `portal/shared/src/entitydesc/entitygroupdesc.cpp.c` (0/13)

### EntityGroupDesc
- [ ] `EntityGroupDesc::StreamPack` · 00617580 · legions 005d49d0 — needs port: entity description: EntityGroupDesc::StreamPack (stream decode)
- [ ] `EntityGroupDesc::GetSubDataIDs` · 00617600 · legions 005d4a50 — needs port: dependent DIDs of EntityGroupDesc for sub-object resolution (AGENTS.md exception)
- [-] `EntityGroupDesc::Destroy` · 00617640 · legions 005d4a90 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)

### SmartArray<EntityDesc>
- [-] `SmartArray<EntityDesc>::operator=` · 006176b0 · legions 005d4b00 — operator of plain data or container; Rust value semantics / derived PartialEq

### EntityGroupDesc
- [-] `EntityGroupDesc::EntityGroupDesc` · 00617720 · legions 005d4b70 — ctor/dtor of plain data; Rust constructs and drops values
- [-] `EntityGroupDesc::GetDBOType` · 00617760 — DBO type tag; Rust uses typed resource ids
- [-] `EntityGroupDesc::~EntityGroupDesc` · 00617770 · legions 005d4bc0 — ctor/dtor of plain data; Rust constructs and drops values

### SmartArray<EntityLinkDesc>
- [-] `SmartArray<EntityLinkDesc>::operator=` · 006177e0 · legions 005d4c30 — operator of plain data or container; Rust value semantics / derived PartialEq

### PSmartArray<EntityLinkDesc>
- [ ] `PSmartArray<EntityLinkDesc>::StreamPack` · 00617850 · legions 005d4ca0 — needs port: entity description: PSmartArray::StreamPack (stream decode)

### EntityGroupDesc
- [-] `EntityGroupDesc::Allocate` · 006178f0 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)
- [-] `EntityGroupDesc::ToFileNode` · 00617920 — text source format (admin tools)
- [-] `EntityGroupDesc::CopyInto` · 00617b10 · legions 005d4d70 — copy of plain data; Rust clones values
- [-] `EntityGroupDesc::FromFileNode` · 00617b70 — text source format (admin tools)

## `portal/shared/src/entitydesc/entitylinkdesc.cpp.c` (0/10)

### EntityLinkDesc
- [ ] `EntityLinkDesc::GetOppositeID` · 00617e70 · legions 005d4dd0 — needs port: entity description: EntityLinkDesc::GetOppositeID
- [-] `EntityLinkDesc::EntityLinkDesc` · 00617ee0 · legions 005d4e40 — ctor/dtor of plain data; Rust constructs and drops values
- [ ] `EntityLinkDesc::GetWBName_ForWsl` · 00617f40 · legions 005d4ea0 — needs port: entity description: EntityLinkDesc::GetWBName_ForWsl
- [ ] `EntityLinkDesc::InqStringPropertyW` · 00617f60 · legions 005d4ec0 — needs port: entity description: EntityLinkDesc::InqStringPropertyW (property inquiry)

### LinkDesc
- [ ] `LinkDesc::InqLinkType` · 00617fc0 — needs port: entity description: LinkDesc::InqLinkType (property inquiry)
- [-] `LinkDesc::ToFileNode` · 00618000 — text source format (admin tools)
- [-] `LinkDesc::FromFileNode` · 00618110 — text source format (admin tools)

### EntityLinkDesc
- [-] `EntityLinkDesc::ToFileNode` · 006182e0 — text source format (admin tools)
- [-] `EntityLinkDesc::FromFileNode` · 00618400 — text source format (admin tools)
- [ ] `EntityLinkDesc::StreamPack` · 00618540 · legions 005d4f20 — needs port: entity description: EntityLinkDesc::StreamPack (stream decode)

## `portal/shared/src/entitydesc/linkrequirements.cpp.c` (0/3)

### LinkRequirements
- [-] `LinkRequirements::LinkRequirements` · 006185b0 · legions 005d4f90 — ctor/dtor of plain data; Rust constructs and drops values
- [-] `LinkRequirements::LinkRequirements` · 006185f0 · legions 005d4fd0 — ctor/dtor of plain data; Rust constructs and drops values
- [ ] `LinkRequirements::StreamPack` · 00618610 · legions 005d4ff0 — needs port: entity description: LinkRequirements::StreamPack (stream decode)

## `portal/shared/src/mesh/basemesh.cpp.c` (2/50)

### CMeshTriList
- [-] `CMeshTriList::GetMeshFragment` · 004cfbe0 · legions 004c3db0 — accessor of a Rust typed field
- [-] `CMeshTriList::GetIndexBuffer` · 004cfc00 · legions 004c3dd0 — D3D index-buffer plumbing; ac2-gpu owns buffers

### MeshMaterials::MaterialAndDataID
- [-] `MeshMaterials::MaterialAndDataID::MaterialAndDataID` · 004cfc20 — ctor/dtor of plain data; Rust constructs and drops values

### MaterialInstance
- [ ] `MaterialInstance::Get` · 004cfc40 · legions 004c3e10 — needs port (check): cited only from a caller in ac2-render terrain/far.rs; verify and cite, or port

### CBaseMesh
- [-] `CBaseMesh::HasMatrixWeights` · 004cfc60 · legions 004c3e30 — accessor of a Rust typed field
- [ ] `CBaseMesh::BuildBoundingBox` · 004cfc80 · legions 004c3e50 — needs port: mesh record: CBaseMesh::BuildBoundingBox (derived bounding data)
- [ ] `CBaseMesh::BuildDefaultBoundingBox` · 004cfca0 · legions 004c3e70 — needs port: mesh record: CBaseMesh::BuildDefaultBoundingBox (derived bounding data)

### (free functions)
- [ ] `STREAMPACK_PACKOBJ` · 004cfcc0 · legions 004c3e90 — needs port: generic StreamPack op for int STREAMPACK_PACKOBJ(STREAMTYPE op, CVertexArray * obj, void * * addr, uint * size)
- [ ] `STREAMPACK_PACKOBJ` · 004cfd30 · legions 004c3f00 — needs port: generic StreamPack op for int STREAMPACK_PACKOBJ(STREAMTYPE op, CMeshTriList * obj, void * * addr, uint * size)

### CMeshDegradeList
- [-] `CMeshDegradeList::CMeshDegradeList` · 004cfda0 · legions 004c3f70 — ctor/dtor of plain data; Rust constructs and drops values

### CBaseMesh
- [-] `CBaseMesh::GetNumDegradeLevels` · 004cfde0 · legions 004c3fe0 — accessor of a Rust typed field

### MeshMaterials
- [-] `MeshMaterials::GetSubObjects` · 004cfe00 · legions 004c4000 — retail DBO object-graph walk for the cache; Arc replaces it
- [-] `MeshMaterials::ReleaseSubObjects` · 004cfeb0 · legions 004c4090 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)
- [ ] `MeshMaterials::GetSubDataIDs` · 004cfef0 · legions 004c40d0 — needs port: dependent DIDs of MeshMaterials for sub-object resolution (AGENTS.md exception)

### CBaseMesh
- [-] `CBaseMesh::ReleaseVertexArray` · 004cff30 · legions 004c4110 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)
- [-] `CBaseMesh::InitVertexArray` · 004cff60 · legions 004c4140 — D3D vertex buffer plumbing; ac2-gpu owns buffers
- [ ] `CBaseMesh::SetMaterial` · 004cfff0 · legions 004c41d0 — needs port: mesh record: CBaseMesh::SetMaterial (runtime setter)
- [ ] `CBaseMesh::SetMaterialID` · 004d0050 · legions 004c4230 — needs port: mesh record: CBaseMesh::SetMaterialID (runtime setter)
- [ ] `CBaseMesh::DrawMeshDegrade` · 004d0080 · legions 004c4260 — needs port: CBaseMesh::DrawMeshDegrade fragment draw by degrade level (ac2-render; no citation)
- [ ] `CBaseMesh::GenerateLocalIndices` · 004d0320 · legions 004c44d0 — needs port: mesh record: CBaseMesh::GenerateLocalIndices (derived index/triangle/connectivity data)
- [ ] `CBaseMesh::GenerateTriangleData` · 004d0530 · legions 004c46e0 — needs port: mesh record: CBaseMesh::GenerateTriangleData (derived index/triangle/connectivity data)
- [ ] `CBaseMesh::GenerateConnectivityData` · 004d0820 · legions 004c49d0 — needs port: mesh record: CBaseMesh::GenerateConnectivityData (derived index/triangle/connectivity data)
- [-] `CBaseMesh::GetSubObjects` · 004d08c0 · legions 004c4a70 — retail DBO object-graph walk for the cache; Arc replaces it
- [-] `CBaseMesh::ReleaseSubObjects` · 004d08f0 · legions 004c4aa0 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)
- [ ] `CBaseMesh::GetSubDataIDs` · 004d0910 · legions 004c4ac0 — needs port: dependent DIDs of CBaseMesh for sub-object resolution (AGENTS.md exception)
- [ ] `CBaseMesh::GenerateShadowVolume` · 004d0920 · legions 004c4ad0 — needs port: mesh record: CBaseMesh::GenerateShadowVolume (derived index/triangle/connectivity data)
- [-] `CBaseMesh::CBaseMesh` · 004d1090 · legions 004c5240 — ctor/dtor of plain data; Rust constructs and drops values
- [-] `CBaseMesh::Destroy` · 004d10c0 · legions 004c5270 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)

### PSmartArray<CPolygon>
- [x] `PSmartArray<CPolygon>::StreamPack` · 004d1260 · legions 004c5410 — ac2-assets/src/ac2_physics_mesh.rs cites PSmartArray<CPolygon>::StreamPack (CPolygon::UnPack records)

### CBaseMesh
- [-] `CBaseMesh::~CBaseMesh` · 004d1450 · legions 004c5600 — ctor/dtor of plain data; Rust constructs and drops values
- [ ] `CBaseMesh::DrawMesh` · 004d1460 · legions 004c5610 — needs port: CBaseMesh::DrawMesh fragment draw by degrade level (ac2-render; no citation)
- [-] `CBaseMesh::ParseVertexArrayFileNode` · 004d17c0 — text source format (admin tools)

### PSmartArray<MeshMaterials::MaterialAndDataID>
- [ ] `PSmartArray<MeshMaterials::MaterialAndDataID>::StreamPack` · 004d1830 · legions 004c5970 — needs port: mesh record: PSmartArray::StreamPack (stream decode)

### CMeshDegradeList
- [ ] `CMeshDegradeList::Create` · 004d18d0 · legions 004c5a10 — needs port: mesh record: CMeshDegradeList::Create (initialisation from the DB record)
- [ ] `CMeshDegradeList::UpdateDetailLevelIndices` · 004d1930 · legions 004c5a70 — needs port: mesh record: CMeshDegradeList::UpdateDetailLevelIndices
- [-] `CMeshDegradeList::Destroy` · 004d19c0 · legions 004c5b00 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)

### MeshMaterials
- [ ] `MeshMaterials::DynamicCopy` · 004d1a30 · legions 004c5b70 — needs port: MeshMaterials::DynamicCopy (per-instance copy of material instances)

### CMeshDegradeList
- [-] `CMeshDegradeList::~CMeshDegradeList` · 004d1ba0 · legions 004c5ce0 — ctor/dtor of plain data; Rust constructs and drops values

### MeshGeometry
- [-] `MeshGeometry::~MeshGeometry` · 004d1ec0 · legions 004c5f20 — ctor/dtor of plain data; Rust constructs and drops values

### CBaseMesh
- [-] `CBaseMesh::ParseMorphTargetNode` · 004d1fc0 — text source format (admin tools)
- [-] `CBaseMesh::ParseVertexSetNode` · 004d2240 — text source format (admin tools)

### PHashTable<unsigned_long,CMorphTargetData>
- [ ] `PHashTable<unsigned_long,CMorphTargetData>::StreamPack` · 004d24e0 — needs port: mesh record: PHashTable::StreamPack (stream decode)

### PHashTable<unsigned_long,CVertexSetData>
- [ ] `PHashTable<unsigned_long,CVertexSetData>::StreamPack` · 004d26d0 — needs port: mesh record: PHashTable::StreamPack (stream decode)

### MeshGeometry
- [-] `MeshGeometry::MeshGeometry` · 004d28c0 · legions 004c6420 — ctor/dtor of plain data; Rust constructs and drops values

### CBaseMesh
- [ ] `CBaseMesh::InitMesh` · 004d2950 · legions 004c64b0 — needs port: mesh record: CBaseMesh::InitMesh (initialisation from the DB record)
- [ ] `CBaseMesh::SetDegradeList` · 004d29d0 · legions 004c6530 — needs port: mesh record: CBaseMesh::SetDegradeList (runtime setter)
- [-] `CBaseMesh::AllocateMaterials` · 004d2a40 · legions 004c65a0 — allocation of material array; Rust Vec
- [x] `CBaseMesh::StreamPack` · 004d2ab0 · legions 004c6610 — ac2-assets/src/ac2_mesh.rs header: CBaseMesh::StreamPack order (CDataMesh record)
- [-] `CBaseMesh::ParseMaterialArrayNode` · 004d2cd0 — text source format (admin tools)
- [-] `CBaseMesh::ParseRenderFileNode` · 004d2e80 — text source format (admin tools)

## `portal/shared/src/mesh/cellmesh.cpp.c` (2/15)

### CCellMesh
- [-] `CCellMesh::ReleaseMesh` · 004cef30 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)
- [x] `CCellMesh::PointInCell` · 004cef40 · legions 004c3510 — ac2-physics envcell.rs, find_cells.rs: interior collision, containment and portal walk (Legions identical)
- [ ] `CCellMesh::GetSubDataIDs` · 004cef50 — needs port: dependent DIDs of CCellMesh for sub-object resolution (AGENTS.md exception)
- [ ] `CCellMesh::GetCellPlanes` · 004cefc0 · legions 004c3590 — needs port: mesh record: CCellMesh::GetCellPlanes
- [-] `CCellMesh::CCellMesh` · 004ceff0 · legions 004c35c0 — ctor/dtor of plain data; Rust constructs and drops values
- [-] `CCellMesh::DynamicCastCellMesh` · 004cf040 — accessor of a Rust typed field
- [-] `CCellMesh::GetDID` · 004cf050 — accessor of the cell mesh DID; Rust typed field
- [-] `CCellMesh::GetSubObjects` · 004cf060 — retail DBO object-graph walk for the cache; Arc replaces it
- [-] `CCellMesh::ReleaseSubObjects` · 004cf070 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)
- [-] `CCellMesh::~CCellMesh` · 004cf090 · legions 004c3660 — ctor/dtor of plain data; Rust constructs and drops values
- [-] `CCellMesh::Allocate` · 004cf110 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)
- [-] `CCellMesh::LoadFromFile` · 004cf140 — text source loader, admin build only

### SmartArray<CPortalPoly>
- [-] `SmartArray<CPortalPoly>::SetNElements` · 004cf370 · legions 004c3830 — container resize (SmartArray::SetNElements); Rust Vec

### CCellMesh
- [x] `CCellMesh::StreamPack` · 004cf3a0 — ac2-physics/src/loader.rs cell_structure cites CCellMesh::StreamPack (physics BSP, cell BSP, portals)
- [-] `CCellMesh::FromFileNode` · 004cf550 — text source format (admin tools)

## `portal/shared/src/mesh/datamesh.cpp.c` (1/19)

### CDataMesh
- [-] `CDataMesh::CDataMesh` · 004d3930 · legions 004c6bf0 — ctor/dtor of plain data; Rust constructs and drops values
- [ ] `CDataMesh::GetDID` · 004d3960 · legions 004c6c20 — needs port: CDataMesh::GetDID DBO lookup by DID
- [-] `CDataMesh::Destroy` · 004d3970 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)
- [-] `CDataMesh::ReleaseMesh` · 004d3980 · legions 004c6c40 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)
- [ ] `CDataMesh::GetSubDataIDs` · 004d3990 — needs port: dependent DIDs of CDataMesh for sub-object resolution (AGENTS.md exception)
- [-] `CDataMesh::GetSubObjects` · 004d39a0 · legions 004c6c60 — retail DBO object-graph walk for the cache; Arc replaces it
- [-] `CDataMesh::ReleaseSubObjects` · 004d39c0 · legions 004c6c80 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)

### PFileNode
- [ ] `PFileNode::MakeSubNodeAsUInt32` · 004d39f0 · legions 004c6cb0 — needs port: file-node tree accessor MakeSubNodeAsUInt32 (binary reader in ac2-dat)

### CDataMesh
- [-] `CDataMesh::Allocate` · 004d3a50 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)

### FileNodeName_PString
- [-] `FileNodeName_PString::FileNodeName_PString` · 004d3aa0 — ctor/dtor of plain data; Rust constructs and drops values

### PFileNode
- [-] `PFileNode::SetNameAsPString` · 004d3ad0 · legions 004c6d60 — FileNode text-source name setter (admin tools)
- [ ] `PFileNode::MakeSubNodeAsPString` · 004d3b40 · legions 004c6dd0 — needs port: file-node tree accessor MakeSubNodeAsPString (binary reader in ac2-dat)
- [ ] `PFileNode::MakeSubNode` · 004d3bb0 · legions 004c6e40 — needs port: file-node tree accessor MakeSubNode (binary reader in ac2-dat)

### CDataMesh
- [x] `CDataMesh::StreamPack` · 004d3bf0 · legions 004c6e80 — ac2-assets/src/ac2_physics_mesh.rs header: CDataMesh::StreamPack runs CStaticMesh then CBaseMesh
- [-] `CDataMesh::ParseMeshInfoFileNode` · 004d3c70 — text source format (admin tools)
- [-] `CDataMesh::ToMeshInfoFileNode` · 004d3e50 — writer for DAT/pack output; the client only reads (tool-only)
- [-] `CDataMesh::FromFileNode` · 004d3f90 — text source format (admin tools)
- [-] `CDataMesh::LoadFromFile` · 004d4030 — text source loader, admin build only
- [-] `CDataMesh::SaveToFile` · 004d4220 — writer for DAT/pack output; the client only reads (tool-only)

## `portal/shared/src/mesh/dynamicmesh.cpp.c` (1/4)

### CStaticMesh
- [-] `CStaticMesh::ReleaseMesh` · 004d4990 · legions 004c74d0 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)

### CDynamicMesh
- [x] `CDynamicMesh::ApplyMaterialModifier` · 004d49a0 · legions 004c74e0 — ac2-assets/src/ac2_appearance.rs cites CDynamicMesh::ApplyMaterialModifier
- [-] `CDynamicMesh::DeleteMaterialModifiers` · 004d4a40 · legions 004c7580 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)
- [ ] `CDynamicMesh::InitDynamicMesh` · 004d4ab0 · legions 004c75f0 — needs port: mesh record: CDynamicMesh::InitDynamicMesh (initialisation from the DB record)

## `portal/shared/src/mesh/meshgroup.cpp.c` (0/13)

### CDataMesh
- [ ] `CDataMesh::Get` · 004d43b0 · legions 004c6f00 — needs port: CDataMesh::Get DBO lookup by DID

### HashBase<unsigned_long>
- [-] `HashBase<unsigned_long>::~HashBase<unsigned_long>` · 004d4410 · legions 004c6f60 — ctor/dtor of plain data; Rust constructs and drops values
- [-] `HashBase<unsigned_long>::InternalInit` · 004d4430 · legions 004c6fc0 — reset to defaults before decode / reset of plain fields; Rust builds fresh values

### CMeshGroup
- [-] `CMeshGroup::CleanupMesh` · 004d4490 · legions 004c7020 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)
- [ ] `CMeshGroup::InitCellMesh` · 004d44e0 · legions 004c7070 — needs port: mesh record: CMeshGroup::InitCellMesh (initialisation from the DB record)
- [ ] `CMeshGroup::InitStaticMesh` · 004d4580 · legions 004c7100 — needs port: mesh record: CMeshGroup::InitStaticMesh (initialisation from the DB record)
- [ ] `CMeshGroup::GetMesh` · 004d4610 · legions 004c7190 — needs port: CMeshGroup::GetMesh DBO lookup by DID
- [-] `CMeshGroup::DestroyDynamicMesh` · 004d4630 · legions 004c71b0 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)
- [ ] `CMeshGroup::DrawMesh` · 004d4670 · legions 004c71f0 — needs port: CMeshGroup::DrawMesh fragment draw by degrade level (ac2-render; no citation)

### HashBase<unsigned_long>
- [ ] `HashBase<unsigned_long>::remove` · 004d46c0 · legions 004c7240 — needs port: mesh record: HashBase::remove

### CMeshGroup
- [ ] `CMeshGroup::InitDatabaseMesh` · 004d4790 · legions 004c72e0 — needs port: mesh record: CMeshGroup::InitDatabaseMesh (initialisation from the DB record)
- [-] `CMeshGroup::Release` · 004d48e0 · legions 004c7420 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)
- [ ] `CMeshGroup::SpawnDynamicMesh` · 004d4920 · legions 004c7460 — needs port: mesh record: CMeshGroup::SpawnDynamicMesh (initialisation from the DB record)

## `portal/shared/src/mesh/meshinstance.cpp.c` (2/15)

### CMeshInstance
- [-] `CMeshInstance::CMeshInstance` · 004cf840 · legions 004c3a10 — ctor/dtor of plain data; Rust constructs and drops values
- [-] `CMeshInstance::GetStaticMesh` · 004cf860 · legions 004c3a30 — accessor of a Rust typed field
- [ ] `CMeshInstance::GetMesh` · 004cf870 · legions 004c3a40 — needs port: CMeshInstance::GetMesh DBO lookup by DID
- [ ] `CMeshInstance::InitDatabaseMesh` · 004cf880 · legions 004c3a50 — needs port: mesh record: CMeshInstance::InitDatabaseMesh (initialisation from the DB record)
- [ ] `CMeshInstance::InitCellMesh` · 004cf8b0 · legions 004c3a80 — needs port: mesh record: CMeshInstance::InitCellMesh (initialisation from the DB record)
- [ ] `CMeshInstance::InitStaticMesh` · 004cf8e0 · legions 004c3ab0 — needs port: mesh record: CMeshInstance::InitStaticMesh (initialisation from the DB record)
- [-] `CMeshInstance::ReleaseMesh` · 004cf900 · legions 004c3ad0 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)
- [x] `CMeshInstance::PerformMeshReplacement` · 004cf930 · legions 004c3b00 — ac2-render object/model.rs load_object_model_with
- [ ] `CMeshInstance::DrawMesh` · 004cf9a0 · legions 004c3b70 — needs port: CMeshInstance::DrawMesh fragment draw by degrade level (ac2-render; no citation)
- [-] `CMeshInstance::~CMeshInstance` · 004cf9f0 · legions 004c3bc0 — ctor/dtor of plain data; Rust constructs and drops values
- [-] `CMeshInstance::GetCellMesh` · 004cfa20 · legions 004c3bf0 — accessor of a Rust typed field
- [-] `CMeshInstance::GetDynamicMesh` · 004cfa40 · legions 004c3c10 — accessor of a Rust typed field
- [ ] `CMeshInstance::RemoveMaterialModifiers` · 004cfa60 · legions 004c3c30 — needs port: mesh record: CMeshInstance::RemoveMaterialModifiers (atom removal rule)
- [ ] `CMeshInstance::MakeDynamic` · 004cfab0 · legions 004c3c80 — needs port: mesh record: CMeshInstance::MakeDynamic
- [x] `CMeshInstance::ApplyMaterialModifier` · 004cfb60 · legions 004c3d30 — ac2-render object/model.rs load_object_model_with

## `portal/shared/src/mesh/morphtargetdata.cpp.c` (0/5)

### CMorphTargetData
- [ ] `CMorphTargetData::StreamPack` · 004d4c40 · legions 004c7780 — needs port: mesh record: CMorphTargetData::StreamPack (stream decode)
- [-] `CMorphTargetData::CMorphTargetData` · 004d4ca0 · legions 004c77e0 — ctor/dtor of plain data; Rust constructs and drops values
- [-] `CMorphTargetData::~CMorphTargetData` · 004d4cc0 · legions 004c7800 — ctor/dtor of plain data; Rust constructs and drops values
- [-] `CMorphTargetData::FromFileNode` · 004d4d30 — text source format (admin tools)
- [-] `CMorphTargetData::CMorphTargetData` · 004d4da0 — ctor/dtor of plain data; Rust constructs and drops values

## `portal/shared/src/mesh/staticmesh.cpp.c` (3/13)

### CStaticMesh
- [ ] `CStaticMesh::BuildRenderBoundingBox` · 004d3190 · legions 004c6830 — needs port: mesh record: CStaticMesh::BuildRenderBoundingBox (derived bounding data)
- [ ] `CStaticMesh::BuildDefaultRenderBoundingBox` · 004d31a0 · legions 004c6840 — needs port: mesh record: CStaticMesh::BuildDefaultRenderBoundingBox (derived bounding data)
- [x] `CStaticMesh::BuildPhysicsBoundingBox` · 004d31b0 · legions 004c6850 — ac2-physics objects.rs: static-object collision dispatch, containment radius, shadow box
- [x] `CStaticMesh::CalcPhysicsContainmentRadius` · 004d3230 · legions 004c68d0 — ac2-physics objects.rs: static-object collision dispatch, containment radius, shadow box
- [x] `CStaticMesh::StreamPack` · 004d32d0 · legions 004c6970 — ac2-assets/src/ac2_physics_mesh.rs header: CStaticMesh::StreamPack (physics polygons, BSP)
- [ ] `CStaticMesh::GetSubDataIDs` · 004d33b0 · legions 004c6a50 — needs port: dependent DIDs of CStaticMesh for sub-object resolution (AGENTS.md exception)
- [-] `CStaticMesh::CStaticMesh` · 004d3400 · legions 004c6aa0 — ctor/dtor of plain data; Rust constructs and drops values
- [-] `CStaticMesh::Destroy` · 004d3440 · legions 004c6ae0 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)
- [-] `CStaticMesh::~CStaticMesh` · 004d34a0 · legions 004c6b40 — ctor/dtor of plain data; Rust constructs and drops values
- [-] `CStaticMesh::ParsePhysicsMaterialNode` · 004d3500 — text source format (admin tools)
- [-] `CStaticMesh::AllocatePhysicsPolygons` · 004d36b0 · legions 004c6bc0 — allocation of physics polygon array; Rust Vec
- [-] `CStaticMesh::ParsePhysicsFileNode` · 004d36e0 — text source format (admin tools)
- [-] `CStaticMesh::FromFileNode` · 004d37f0 — text source format (admin tools)

## `portal/shared/src/mesh/vertexsetdata.cpp.c` (0/4)

### CVertexSetData
- [-] `CVertexSetData::CVertexSetData` · 004d4e00 · legions 004c78a0 — ctor/dtor of plain data; Rust constructs and drops values
- [-] `CVertexSetData::~CVertexSetData` · 004d4e20 · legions 004c78c0 — ctor/dtor of plain data; Rust constructs and drops values
- [-] `CVertexSetData::FromFileNode` · 004d4e90 — text source format (admin tools)
- [-] `CVertexSetData::CVertexSetData` · 004d4f60 — ctor/dtor of plain data; Rust constructs and drops values

## `portal/shared/src/objdesc/appearance/appearanceinfo.cpp.c` (7/13)

### AppearanceInfo
- [x] `AppearanceInfo::StreamPack` · 0064a5b0
- [-] `AppearanceInfo::GetAppliedAppKeyIter` · 0064a5e0 · legions 005f9c90 — container iterator begin (AppliedAppKey); Rust iterators
- [-] `AppearanceInfo::GetAppliedAppKeyEnd` · 0064a610 · legions 005f9cc0 — container iterator end (AppliedAppKey); Rust iterators
- [x] `AppearanceInfo::AugmentAtomCollection` · 0064a630 · legions 005f9ce0 — ac2-render object/appearance.rs augment_atoms
- [x] `AppearanceInfo::RemoveAppearanceKey` · 0064a730 · legions 005f9dd0 — ac2-weenie appearance_ops.rs (the decompile algebra over Appearance; unit tests in that file)
- [x] `AppearanceInfo::HasAppliedAppKey` · 0064a760 · legions 005f9e00 — ac2-weenie appearance_ops.rs (the decompile algebra over Appearance; unit tests in that file)
- [-] `AppearanceInfo::operator=` · 0064a7c0 · legions 005f9e60 — operator of plain data or container; Rust value semantics / derived PartialEq
- [-] `AppearanceInfo::~AppearanceInfo` · 0064a7e0 · legions 005f9e80 — ctor/dtor of plain data; Rust constructs and drops values
- [x] `AppearanceInfo::AddAppliedAppKey` · 0064a840 · legions 005f9ee0 — ac2-weenie appearance_ops.rs (the decompile algebra over Appearance; unit tests in that file)
- [x] `AppearanceInfo::RemoveAppearanceKeys` · 0064a870 · legions 005f9f10 — ac2-weenie appearance_ops.rs (the decompile algebra over Appearance; unit tests in that file)
- [x] `AppearanceInfo::AugmentExistingKeys` · 0064a970 · legions 005fa010 — ac2-weenie appearance_ops.rs (the decompile algebra over Appearance; unit tests in that file)
- [-] `AppearanceInfo::AppearanceInfo` · 0064aa60 · legions 005fa100 — ctor/dtor of plain data; Rust constructs and drops values
- [-] `AppearanceInfo::AppearanceInfo` · 0064aaa0 · legions 005fa140 — ctor/dtor of plain data; Rust constructs and drops values

## `portal/shared/src/objdesc/appearance/appearancetable.cpp.c` (2/24)

### (free functions)
- [ ] `STREAMPACK` · 00648cf0 · legions 00486a30 — needs port: generic StreamPack op for int STREAMPACK(STREAMTYPE op, PHashTable<unsigned_long,UILocationData> * obj, void * * addr, uint * size)

### AppearanceTable
- [x] `AppearanceTable::StreamPack` · 00648d80 · legions 005f8d00 — ac2-assets ac2_appearance.rs (all atom kinds; FX/icon atoms decoded past)
- [-] `AppearanceTable::Destroy` · 00648f70 · legions 005f8f70 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)
- [-] `AppearanceTable::ToFileNode` · 00648f90 — text source format (admin tools)
- [ ] `AppearanceTable::GetSubDataIDs` · 00649260 — needs port: dependent DIDs of AppearanceTable for sub-object resolution (AGENTS.md exception)
- [ ] `AppearanceTable::AddBaseAppearanceDID` · 00649330 — needs port: appearance/visual record: AppearanceTable::AddBaseAppearanceDID (atom insert rule)
- [x] `AppearanceTable::GetAtoms` · 00649370 · legions 005f9060 — ac2-assets ac2_appearance.rs (all atom kinds; FX/icon atoms decoded past)
- [ ] `AppearanceTable::HasAppearanceKey` · 00649470 · legions 005f9140 — needs port: appearance/visual record: AppearanceTable::HasAppearanceKey
- [ ] `AppearanceTable::InqAppearanceKeyModList` · 00649540 · legions 005f91f0 — needs port: appearance/visual record: AppearanceTable::InqAppearanceKeyModList (property inquiry)
- [ ] `AppearanceTable::InqTsysAppearanceKeyModCount` · 00649650 · legions 005f92e0 — needs port: appearance/visual record: AppearanceTable::InqTsysAppearanceKeyModCount (property inquiry)
- [ ] `AppearanceTable::CalculateAppearanceModsBelowThreshold` · 006497a0 — needs port: appearance/visual record: AppearanceTable::CalculateAppearanceModsBelowThreshold

### PHashTable<unsigned_long,VisualModTable>
- [ ] `PHashTable<unsigned_long,VisualModTable>::StreamPack` · 006499a0 — needs port: appearance/visual record: PHashTable::StreamPack (stream decode)

### AppearanceTable
- [-] `AppearanceTable::AppearanceTable` · 00649ba0 · legions 005f9630 — ctor/dtor of plain data; Rust constructs and drops values
- [-] `AppearanceTable::GetDBOType` · 00649c30 — DBO type tag; Rust uses typed resource ids
- [-] `AppearanceTable::~AppearanceTable` · 00649c60 · legions 005f96f0 — ctor/dtor of plain data; Rust constructs and drops values
- [-] `AppearanceTable::Allocate` · 00649d10 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)

### PList<AtomCollection>
- [-] `PList<AtomCollection>::Pack` · 00649e20 · legions 005f98b0 — writer for DAT/pack output; the client only reads (tool-only)
- [-] `PList<AtomCollection>::GetPackSize` · 00649e70 — writer for DAT/pack output; the client only reads (tool-only)
- [ ] `PList<AtomCollection>::UnPack` · 00649eb0 · legions 005f995a — needs port: appearance/visual record: PList::UnPack (stream decode)

### HashTable<unsigned_long,VisualModTable>
- [-] `HashTable<unsigned_long,VisualModTable>::operator=` · 0064a060 · legions 005f9af0 — operator of plain data or container; Rust value semantics / derived PartialEq

### AppearanceTable
- [-] `AppearanceTable::VMTsFromFileNode` · 0064a210 — text source format (admin tools)
- [-] `AppearanceTable::operator=` · 0064a370 · legions 005f9c10 — operator of plain data or container; Rust value semantics / derived PartialEq
- [-] `AppearanceTable::CopyInto` · 0064a3d0 · legions 005f9c70 — copy of plain data; Rust clones values
- [-] `AppearanceTable::FromFileNode` · 0064a3f0 — text source format (admin tools)

## `portal/shared/src/objdesc/appearance/appearancetableadaptor.cpp.c` (0/3)

### AppearanceTableAdaptor
- [ ] `AppearanceTableAdaptor::HasAppearanceKey` · 0064aaf0 · legions 005fa190 — needs port: appearance/visual record: AppearanceTableAdaptor::HasAppearanceKey
- [ ] `AppearanceTableAdaptor::InqTsysAppearanceKeyModCount` · 0064ab50 · legions 005fa1e0 — needs port: appearance/visual record: AppearanceTableAdaptor::InqTsysAppearanceKeyModCount (property inquiry)
- [ ] `AppearanceTableAdaptor::InqAppearanceKeyModList` · 0064abc0 — needs port: appearance/visual record: AppearanceTableAdaptor::InqAppearanceKeyModList (property inquiry)

## `portal/shared/src/objdesc/appearance/appliedappearancekey.cpp.c` (2/7)

### AppliedAppearanceKey
- [x] `AppliedAppearanceKey::AppliedAppearanceKey` · 00641fc0 · legions 005f57c0 — ac2-script native AppliedAppearanceKey (plain data)
- [x] `AppliedAppearanceKey::operator=` · 00641fe0 · legions 005f57e0 — ac2-script native AppliedAppearanceKey (plain data)

### CAmbientSoundInfo
- [-] `CAmbientSoundInfo::operator==` · 00642000 · legions 005f5800 — operator of plain data or container; Rust value semantics / derived PartialEq

### AppliedAppearanceKey
- [-] `AppliedAppearanceKey::AppliedAppearanceKey` · 00642020 · legions 005f5820 — ctor/dtor of plain data; Rust constructs and drops values
- [ ] `AppliedAppearanceKey::AppearanceKeyFFN` · 00642040 · legions 005f5890 — needs port: appearance/visual record: AppliedAppearanceKey::AppearanceKeyFFN
- [ ] `AppliedAppearanceKey::ValueFFN` · 006421f0 · legions 005f5a40 — needs port: appearance/visual record: AppliedAppearanceKey::ValueFFN
- [~] `AppliedAppearanceKey::FromFileNode` · 00642260 · legions 005f5ab0 — FromFileNode reads a PFileNode (gPFileNode, unbound: no PFileNode native in the port; ac2-dat PFileNode rows unchecked)

## `portal/shared/src/objdesc/appearance/atomcollection.cpp.c` (6/30)

### FXAtom
- [-] `FXAtom::FXAtom` · 00646840 — ctor/dtor of plain data; Rust constructs and drops values
- [-] `FXAtom::~FXAtom` · 00646880 · legions 005f74d0 — ctor/dtor of plain data; Rust constructs and drops values

### AtomCollection
- [ ] `AtomCollection::GetSubDataIDs` · 00646990 · legions 005f75e0 — needs port: dependent DIDs of AtomCollection for sub-object resolution (AGENTS.md exception)
- [-] `AtomCollection::ToFileNode` · 00646aa0 — text source format (admin tools)
- [ ] `AtomCollection::AddIconModAtom` · 00646fa0 — needs port: appearance/visual record: AtomCollection::AddIconModAtom (atom insert rule)
- [ ] `AtomCollection::AddReplacementIconModAtom` · 00646fd0 · legions 005f7870 — needs port: appearance/visual record: AtomCollection::AddReplacementIconModAtom (atom insert rule)

### List<IconModAtom>
- [-] `List<IconModAtom>::operator=` · 00647130 · legions 005f79d0 — operator of plain data or container; Rust value semantics / derived PartialEq

### AtomCollection
- [ ] `AtomCollection::AddOpacityAtom` · 006471d0 — needs port: appearance/visual record: AtomCollection::AddOpacityAtom (atom insert rule)
- [ ] `AtomCollection::AddMeshReplacementAtom` · 00647200 — needs port: appearance/visual record: AtomCollection::AddMeshReplacementAtom (atom insert rule)
- [ ] `AtomCollection::AddPGDReplacementAtom` · 00647230 — needs port: appearance/visual record: AtomCollection::AddPGDReplacementAtom (atom insert rule)
- [x] `AtomCollection::AddReplacementOpacityAtom` · 00647260 · legions 005f7a70 — ac2-assets ac2_appearance.rs (all atom kinds; FX/icon atoms decoded past)
- [x] `AtomCollection::AddReplacementMeshReplacementAtom` · 00647320 · legions 005f7b30 — ac2-assets ac2_appearance.rs (all atom kinds; FX/icon atoms decoded past)
- [x] `AtomCollection::AddReplacementPGDReplacementAtom` · 006473e0 · legions 005f7bf0 — ac2-assets ac2_appearance.rs (all atom kinds; FX/icon atoms decoded past)

### PList<MaterialModifierAtom>
- [-] `PList<MaterialModifierAtom>::operator=` · 006474c0 · legions 005f7cd0 — operator of plain data or container; Rust value semantics / derived PartialEq

### PList<OpacityAtom>
- [-] `PList<OpacityAtom>::operator=` · 00647500 · legions 005f7d10 — operator of plain data or container; Rust value semantics / derived PartialEq

### PList<MeshReplacementAtom>
- [-] `PList<MeshReplacementAtom>::operator=` · 00647540 · legions 005f7d50 — operator of plain data or container; Rust value semantics / derived PartialEq

### PList<PGDReplacementAtom>
- [-] `PList<PGDReplacementAtom>::operator=` · 00647580 · legions 005f7d90 — operator of plain data or container; Rust value semantics / derived PartialEq

### AtomCollection
- [x] `AtomCollection::AddReplacementMatModAtom` · 006475c0 · legions 005f7dd0 — ac2-assets ac2_appearance.rs (all atom kinds; FX/icon atoms decoded past)
- [-] `AtomCollection::Clear` · 006476e0 · legions 005f7ef0 — reset to defaults before decode / reset of plain fields; Rust builds fresh values
- [-] `AtomCollection::AtomCollection` · 00647780 — ctor/dtor of plain data; Rust constructs and drops values

### FXAtom
- [-] `FXAtom::FXAtom` · 006478a0 · legions 005f7f90 — ctor/dtor of plain data; Rust constructs and drops values

### _STL::_Rb_tree<FXAtom,FXAtom,_STL::_Identity<FXAtom>,_STL::less<FXAtom>,_STL::allocator<FXAtom>_>
- [-] `_STL::_Rb_tree<FXAtom,FXAtom,_STL::_Identity<FXAtom>,_STL::less<FXAtom>,_STL::allocator<FXAtom>_>::operator=` · 00647b60 · legions 005f8250 — operator of plain data or container; Rust value semantics / derived PartialEq

### AtomCollection
- [-] `AtomCollection::operator=` · 00647f20 · legions 005f8610 — operator of plain data or container; Rust value semantics / derived PartialEq
- [ ] `AtomCollection::AddFXAtom` · 00647fe0 — needs port: appearance/visual record: AtomCollection::AddFXAtom (atom insert rule)
- [ ] `AtomCollection::AddReplacementFXAtom` · 00648000 · legions 005f86d0 — needs port: appearance/visual record: AtomCollection::AddReplacementFXAtom (atom insert rule)
- [-] `AtomCollection::FromFileNode` · 006480e0 — text source format (admin tools)

### (free functions)
- [ ] `STREAMPACK_STL_SET` · 006487a0 · legions 005f87b0 — needs port: generic StreamPack op for int STREAMPACK_STL_SET(STREAMTYPE op, set<FXAtom,_STL::less<FXAtom>,_STL::allocator<FXAtom>_> * n, void * * ad

### AtomCollection
- [-] `AtomCollection::AtomCollection` · 00648990 · legions 005f89a0 — ctor/dtor of plain data; Rust constructs and drops values
- [x] `AtomCollection::UpdateAtomCollection` · 00648a80 · legions 005f8a90 — ac2-assets ac2_appearance.rs (all atom kinds; FX/icon atoms decoded past)
- [x] `AtomCollection::StreamPack` · 00648c40 · legions 005f8c50 — ac2-assets ac2_appearance.rs (all atom kinds; FX/icon atoms decoded past)

## `portal/shared/src/objdesc/appearance/atoms.cpp.c` (7/43)

### MaterialModifierAtom
- [-] `MaterialModifierAtom::Clear` · 00642410 · legions 005f5c60 — reset to defaults before decode / reset of plain fields; Rust builds fresh values
- [-] `MaterialModifierAtom::operator==` · 00642430 · legions 005f5c80 — operator of plain data or container; Rust value semantics / derived PartialEq
- [x] `MaterialModifierAtom::Replaces` · 00642470 · legions 005f5cc0 — ac2-assets/src/ac2_appearance.rs fn replaces cites MaterialModifierAtom::Replaces (priority and mesh-type rule)
- [ ] `MaterialModifierAtom::GetSubDataIDs` · 006424b0 · legions 005f5d00 — needs port: dependent DIDs of MaterialModifierAtom for sub-object resolution (AGENTS.md exception)

### OpacityAtom
- [-] `OpacityAtom::Clear` · 006424e0 · legions 005f5d30 — reset to defaults before decode / reset of plain fields; Rust builds fresh values
- [x] `OpacityAtom::Replaces` · 006424f0 · legions 005f5d40 — ac2-assets ac2_appearance.rs (all atom kinds; FX/icon atoms decoded past)
- [-] `OpacityAtom::operator==` · 00642510 · legions 005f5d60 — operator of plain data or container; Rust value semantics / derived PartialEq

### MeshReplacementAtom
- [-] `MeshReplacementAtom::Clear` · 00642540 · legions 005f5d90 — reset to defaults before decode / reset of plain fields; Rust builds fresh values
- [x] `MeshReplacementAtom::Replaces` · 00642550 · legions 005f5da0 — ac2-assets ac2_appearance.rs (all atom kinds; FX/icon atoms decoded past)
- [-] `MeshReplacementAtom::operator==` · 00642580 · legions 005f5dd0 — operator of plain data or container; Rust value semantics / derived PartialEq

### PGDReplacementAtom
- [-] `PGDReplacementAtom::Clear` · 006425b0 · legions 005f5e00 — reset to defaults before decode / reset of plain fields; Rust builds fresh values
- [x] `PGDReplacementAtom::Replaces` · 006425c0 · legions 005f5e10 — ac2-assets ac2_appearance.rs (all atom kinds; FX/icon atoms decoded past)
- [-] `PGDReplacementAtom::operator==` · 006425f0 · legions 005f5e40 — operator of plain data or container; Rust value semantics / derived PartialEq
- [ ] `PGDReplacementAtom::GetSubDataIDs` · 00642620 · legions 005f5e70 — needs port: dependent DIDs of PGDReplacementAtom for sub-object resolution (AGENTS.md exception)

### IconModAtom
- [-] `IconModAtom::Clear` · 00642630 · legions 005f5e80 — reset to defaults before decode / reset of plain fields; Rust builds fresh values
- [ ] `IconModAtom::Replaces` · 00642640 · legions 005f5e90 — needs port: appearance/visual record: IconModAtom::Replaces (replacement rule)
- [ ] `IconModAtom::GetSubDataIDs` · 006426b0 · legions 005f5f00 — needs port: dependent DIDs of IconModAtom for sub-object resolution (AGENTS.md exception)
- [-] `IconModAtom::ToFileNode` · 006426c0 — text source format (admin tools)

### MeshMaterials::MaterialAndDataID
- [ ] `MeshMaterials::MaterialAndDataID::StreamPack` · 006426e0 — needs port: mesh record: MeshMaterials::StreamPack (stream decode)

### MaterialModifierAtom
- [ ] `MaterialModifierAtom::StreamPack` · 00642710 — needs port: material record: MaterialModifierAtom::StreamPack (stream decode)

### OpacityAtom
- [x] `OpacityAtom::StreamPack` · 00642790 — ac2-assets ac2_appearance.rs (all atom kinds; FX/icon atoms decoded past)

### MeshReplacementAtom
- [x] `MeshReplacementAtom::StreamPack` · 006427d0 — ac2-assets ac2_appearance.rs (all atom kinds; FX/icon atoms decoded past)

### PGDReplacementAtom
- [x] `PGDReplacementAtom::StreamPack` · 00642830 · legions 005f6030 — ac2-assets ac2_appearance.rs (all atom kinds; FX/icon atoms decoded past)

### FXAtom
- [ ] `FXAtom::Replaces` · 00642870 · legions 005f6070 — needs port: appearance/visual record: FXAtom::Replaces (replacement rule)
- [-] `FXAtom::operator<` · 00642890 · legions 005f6090 — operator of plain data or container; Rust value semantics / derived PartialEq
- [ ] `FXAtom::GetSubDataIDs` · 006428c0 · legions 005f60c0 — needs port: dependent DIDs of FXAtom for sub-object resolution (AGENTS.md exception)

### IconModAtom
- [-] `IconModAtom::operator==` · 00642910 · legions 005f6160 — operator of plain data or container; Rust value semantics / derived PartialEq

### CVertexSetData
- [ ] `CVertexSetData::StreamPack` · 00642950 — needs port: mesh record: CVertexSetData::StreamPack (stream decode)

### FootPrint
- [-] `FootPrint::Reset` · 00642980 · legions 005f61a0 — reset to defaults before decode / reset of plain fields; Rust builds fresh values

### AppearanceAtom
- [ ] `AppearanceAtom::ReadPriority` · 00642990 — needs port: AppearanceAtom::ReadPriority (atom priority read inside the atom StreamPack order; uncited)
- [-] `AppearanceAtom::WritePriority` · 00642a20 — writer for DAT/pack output; the client only reads (tool-only)

### MaterialModifierAtom
- [-] `MaterialModifierAtom::ToFileNode` · 00642a90 — text source format (admin tools)
- [-] `MaterialModifierAtom::FromFileNode` · 00642e40 — text source format (admin tools)

### OpacityAtom
- [-] `OpacityAtom::ToFileNode` · 00643220 — text source format (admin tools)
- [-] `OpacityAtom::FromFileNode` · 00643350 — text source format (admin tools)

### MeshReplacementAtom
- [-] `MeshReplacementAtom::ToFileNode` · 006435e0 — text source format (admin tools)
- [-] `MeshReplacementAtom::FromFileNode` · 006437f0 — text source format (admin tools)

### PGDReplacementAtom
- [-] `PGDReplacementAtom::ToFileNode` · 00643b00 — text source format (admin tools)
- [-] `PGDReplacementAtom::FromFileNode` · 00643c00 — text source format (admin tools)

### FXAtom
- [-] `FXAtom::ToFileNode` · 00643db0 — text source format (admin tools)

### IconModAtom
- [-] `IconModAtom::FromFileNode` · 00643f10 — text source format (admin tools)

### SmartArray<FXData>
- [-] `SmartArray<FXData>::AddToEnd` · 00643f50 — container append (SmartArray::AddToEnd); Rust Vec push

### FXAtom
- [-] `FXAtom::FromFileNode` · 00643fa0 — text source format (admin tools)

## `portal/shared/src/objdesc/appearance/partgroupdatadesc.cpp.c` (11/32)

### UIRegion
- [ ] `UIRegion::SetZLevel_Internal` · 00644090 · legions 005f61b0 — needs port: UI region (ac2-ui): UIRegion::SetZLevel_Internal (runtime setter)

### PartGroupDataDesc
- [-] `PartGroupDataDesc::operator==` · 006440a0 · legions 005f61c0 — operator of plain data or container; Rust value semantics / derived PartialEq

### CMotionInterp
- [ ] `CMotionInterp::GetLocalDesiredVelocity` · 006440f0 · legions 005f6210 — needs port: motion interpreter (ac2-anim): CMotionInterp::GetLocalDesiredVelocity

### PartGroupDataDesc
- [x] `PartGroupDataDesc::RemoveChild` · 00644100 · legions 005f6220 — ac2-weenie appearance_ops.rs (the decompile algebra over Appearance; unit tests in that file)
- [x] `PartGroupDataDesc::IsConnPtEmpty` · 00644160 · legions 005f6280 — ac2-weenie appearance.rs
- [x] `PartGroupDataDesc::FindPGDDWithPartGroupKey` · 006441a0 · legions 005f62c0 — ac2-render object/appearance.rs over ac2-assets Ac2TotalVisualDesc

### SmartArray<PartGroupDataDesc_*>
- [-] `SmartArray<PartGroupDataDesc_*>::RemoveNullsOrdered` · 00644210 · legions 005f6330 — container compaction (RemoveNullsOrdered); Rust collections

### PartGroupDataDesc
- [x] `PartGroupDataDesc::PrepareForRemoval` · 00644260 · legions 005f6380 — ac2-weenie appearance_ops.rs (the decompile algebra over Appearance; unit tests in that file)
- [-] `PartGroupDataDesc::Empty` · 006442b0 · legions 005f63d0 — reset to defaults before decode / reset of plain fields; Rust builds fresh values
- [ ] `PartGroupDataDesc::AddChildPointer` · 006442f0 · legions 005f6410 — needs port: appearance/visual record: PartGroupDataDesc::AddChildPointer (atom insert rule)
- [x] `PartGroupDataDesc::HasFX` · 00644330 · legions 005f6450 — ac2-weenie/src/appearance_ops.rs has_fx cites PartGroupDataDesc::HasFX
- [-] `PartGroupDataDesc::GetPackBitfield` · 00644350 · legions 005f6470 — writer for DAT/pack output; the client only reads (tool-only)
- [~] `PartGroupDataDesc::StreamPack` · 006443c0 · legions 005f64e0 — unpack only, in ac2-net protocol::fx / protocol::visual (Legions FXNode::StreamPack 0x00642410 instruction-identical)
- [x] `PartGroupDataDesc::AugmentAtomCollection` · 00644510 · legions 005f6690 — ac2-render object/appearance.rs augment_atoms
- [-] `PartGroupDataDesc::WriteAppearanceKeys` · 00644590 — writer for DAT/pack output; the client only reads (tool-only)
- [ ] `PartGroupDataDesc::GetSubDataIDs` · 00644950 · legions 005f6710 — needs port: dependent DIDs of PartGroupDataDesc for sub-object resolution (AGENTS.md exception)
- [-] `PartGroupDataDesc::Clear` · 00644a30 · legions 005f67c0 — Clear of PartGroupDataDesc (reset)
- [x] `PartGroupDataDesc::HasAppliedAppKey` · 00644a80 · legions 005f6810 — ac2-weenie appearance_ops.rs (the decompile algebra over Appearance; unit tests in that file)
- [x] `PartGroupDataDesc::RemoveAppearanceKeys` · 00644ad0 · legions 005f6860 — ac2-weenie appearance_ops.rs (the decompile algebra over Appearance; unit tests in that file)
- [x] `PartGroupDataDesc::RemoveAppearanceKey` · 00644bb0 · legions 005f6940 — ac2-weenie appearance_ops.rs (the decompile algebra over Appearance; unit tests in that file)
- [ ] `PartGroupDataDesc::ReadFX` · 00644c40 — needs port: appearance/visual record: PartGroupDataDesc::ReadFX
- [-] `PartGroupDataDesc::ToFileNode` · 00644fe0 — text source format (admin tools)
- [x] `PartGroupDataDesc::AugmentAppearanceInfo` · 00645580 · legions 005f6b20 — ac2-weenie appearance_ops.rs (the decompile algebra over Appearance; unit tests in that file)
- [ ] `PartGroupDataDesc::ReadAppearanceKeys` · 006455e0 — needs port: appearance/visual record: PartGroupDataDesc::ReadAppearanceKeys
- [-] `PartGroupDataDesc::~PartGroupDataDesc` · 00645940 · legions 005f6bf0 — ctor/dtor of plain data; Rust constructs and drops values
- [-] `PartGroupDataDesc::FromFileNode` · 00645a20 — text source format (admin tools)

### PHashTable<unsigned_long,PSmartArray<FXData>_>
- [~] `PHashTable<unsigned_long,PSmartArray<FXData>_>::StreamPack` · 00645f00 — unpack only, in ac2-net protocol::fx / protocol::visual (Legions FXNode::StreamPack 0x00642410 instruction-identical)

### PartGroupDataDesc
- [-] `PartGroupDataDesc::PartGroupDataDesc` · 006460f0 · legions 005f6ec0 — ctor/dtor of plain data; Rust constructs and drops values
- [-] `PartGroupDataDesc::operator=` · 006462a0 · legions 005f7070 — operator of plain data or container; Rust value semantics / derived PartialEq
- [x] `PartGroupDataDesc::ApplyAdditivePGDDesc` · 00646300 · legions 005f70d0 — ac2-weenie appearance.rs (FX override data kept by id)
- [-] `PartGroupDataDesc::PartGroupDataDesc` · 006464f0 · legions 005f72c0 — ctor/dtor of plain data; Rust constructs and drops values
- [ ] `PartGroupDataDesc::FillFXTable` · 006465d0 · legions 005f73a0 — needs port: appearance/visual record: PartGroupDataDesc::FillFXTable

## `portal/shared/src/objdesc/appearance/visualmodtable.cpp.c` (2/8)

### VisualModTable
- [x] `VisualModTable::StreamPack` · 0064ac30 — ac2-assets ac2_appearance.rs (all atom kinds; FX/icon atoms decoded past)
- [x] `VisualModTable::UpdateAtomCollection` · 0064ac60 · legions 005fa2c0 — ac2-assets ac2_appearance.rs (all atom kinds; FX/icon atoms decoded past)
- [ ] `VisualModTable::GetSubDataIDs` · 0064aca0 · legions 005fa300 — needs port: dependent DIDs of VisualModTable for sub-object resolution (AGENTS.md exception)
- [-] `VisualModTable::ToFileNode` · 0064ace0 — text source format (admin tools)
- [ ] `VisualModTable::GetAtomCollection` · 0064adc0 — needs port: appearance/visual record: VisualModTable::GetAtomCollection
- [-] `VisualModTable::FromFileNode` · 0064aee0 — text source format (admin tools)
- [-] `VisualModTable::VisualModTable` · 0064b010 · legions 005fa340 — ctor/dtor of plain data; Rust constructs and drops values
- [-] `VisualModTable::~VisualModTable` · 0064b070 · legions 005fa3a0 — ctor/dtor of plain data; Rust constructs and drops values

## `portal/shared/src/objdesc/icon/icondesc.cpp.c` (2/18)

### RGBAColor
- [-] `RGBAColor::operator==` · 0063faf0 · legions 005f3a20 — operator of plain data or container; Rust value semantics / derived PartialEq

### IconLayerDesc
- [-] `IconLayerDesc::HasColor` · 0063fb60 — accessor of a Rust typed field

### IconDesc
- [x] `IconDesc::StreamPack` · 0063fb80
- [-] `IconDesc::operator==` · 0063fc00 · legions 005f3b10 — operator of plain data or container; Rust value semantics / derived PartialEq
- [-] `IconDesc::IsValid` · 0063fc80 · legions 005f3b90 — accessor of a Rust typed field
- [ ] `IconDesc::GetSubDataIDs` · 0063fcd0 · legions 005f3be0 — needs port: dependent DIDs of IconDesc for sub-object resolution (AGENTS.md exception)
- [-] `IconDesc::ToFileNode` · 0063fea0 — text source format (admin tools)
- [-] `IconDesc::Clear` · 006400e0 · legions 005f3e10 — reset to defaults before decode / reset of plain fields; Rust builds fresh values

### PList<IconLayerDesc>
- [-] `PList<IconLayerDesc>::Pack` · 00640100 · legions 005f3e30 — writer for DAT/pack output; the client only reads (tool-only)
- [-] `PList<IconLayerDesc>::GetPackSize` · 00640150 — writer for DAT/pack output; the client only reads (tool-only)
- [x] `PList<IconLayerDesc>::UnPack` · 00640190 · legions 005f3ed4

### IconDesc
- [-] `IconDesc::IconDesc` · 006402c0 · legions 005f3ff0 — ctor/dtor of plain data; Rust constructs and drops values
- [-] `IconDesc::~IconDesc` · 006402f0 · legions 005f4020 — ctor/dtor of plain data; Rust constructs and drops values
- [-] `IconDesc::operator=` · 00640320 · legions 005f4050 — operator of plain data or container; Rust value semantics / derived PartialEq
- [~] `IconDesc::ApplyAdditiveIconLayer` · 00640360 · legions 005f4090 — ac2-weenie icon layers; AppearanceTable icon modifiers missing
- [-] `IconDesc::IconDesc` · 00640440 · legions 005f4170 — ctor/dtor of plain data; Rust constructs and drops values
- [-] `IconDesc::FromFileNode` · 00640480 — text source format (admin tools)
- [~] `IconDesc::ApplyAdditiveIconDesc` · 00640580 · legions 005f41b0 — ac2-weenie appearance.rs: a layer id replaces

## `portal/shared/src/objdesc/icon/iconlayerdesc.cpp.c` (1/7)

### IconLayerDesc
- [-] `IconLayerDesc::Clear` · 006405c0 · legions 005f41f0 — reset to defaults before decode / reset of plain fields; Rust builds fresh values
- [ ] `IconLayerDesc::ApplyAdditive` · 006405f0 · legions 005f4220 — needs port: IconLayerDesc::ApplyAdditive icon layer compositing

### (free functions)
- [ ] `STREAMPACK_STREAMPACKOBJ` · 00640640 · legions 005f4270 — needs port: generic StreamPack op for int STREAMPACK_STREAMPACKOBJ(STREAMTYPE op, RGBAColor * obj, void * * addr, uint * size)

### IconLayerDesc
- [-] `IconLayerDesc::IconLayerDesc` · 00640660 · legions 005f4290 — ctor/dtor of plain data; Rust constructs and drops values
- [x] `IconLayerDesc::StreamPack` · 006406a0 · legions 005f42d0
- [-] `IconLayerDesc::ToFileNode` · 00640700 — text source format (admin tools)
- [-] `IconLayerDesc::FromFileNode` · 00640890 — text source format (admin tools)

## `portal/shared/src/objdesc/physicsdesc.cpp.c` (3/14)

### PhysicsDesc
- [ ] `PhysicsDesc::get_timestamp` · 00640b30 · legions 005f4330 — needs port: appearance/visual record: PhysicsDesc::get_timestamp
- [ ] `PhysicsDesc::get_parent_id` · 00640b40 · legions 005f4340 — needs port: appearance/visual record: PhysicsDesc::get_parent_id
- [ ] `PhysicsDesc::get_parent_instance_stamp` · 00640b60 · legions 005f4360 — needs port: appearance/visual record: PhysicsDesc::get_parent_instance_stamp
- [ ] `PhysicsDesc::get_location_id` · 00640b70 · legions 005f4370 — needs port: appearance/visual record: PhysicsDesc::get_location_id
- [ ] `PhysicsDesc::get_mode` · 00640b80 · legions 005f4380 — needs port: appearance/visual record: PhysicsDesc::get_mode
- [ ] `PhysicsDesc::get_behaviors` · 00640b90 · legions 005f4390 — needs port: appearance/visual record: PhysicsDesc::get_behaviors
- [ ] `PhysicsDesc::get_sliders` · 00640ba0 · legions 005f43a0 — needs port: appearance/visual record: PhysicsDesc::get_sliders
- [ ] `PhysicsDesc::get_fx` · 00640bb0 · legions 005f43b0 — needs port: appearance/visual record: PhysicsDesc::get_fx

### (free functions)
- [x] `STREAMPACK_STL` · 00640f40 · legions 005f4740
- [x] `STREAMPACK_STL` · 006410e0 · legions 005f48e0

### PhysicsDesc
- [-] `PhysicsDesc::Destroy` · 006412a0 · legions 005f4aa0 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)
- [-] `PhysicsDesc::~PhysicsDesc` · 00641450 · legions 005f4c50 — ctor/dtor of plain data; Rust constructs and drops values
- [x] `PhysicsDesc::StreamPack` · 00641530
- [-] `PhysicsDesc::PhysicsDesc` · 00641c70 · legions 005f5470 — ctor/dtor of plain data; Rust constructs and drops values

## `portal/shared/src/objdesc/visualdesc.cpp.c` (29/58)

### VisualDesc
- [ ] `VisualDesc::FillFXTable` · 0063c910 · legions 005f1440 — needs port: appearance/visual record: VisualDesc::FillFXTable
- [ ] `VisualDesc::GetStartupFX` · 0063c920 · legions 005f1450 — needs port: VisualDesc::GetStartupFX
- [ ] `VisualDesc::HasFX` · 0063c930 · legions 005f1460 — needs port: VisualDesc::HasFX (global FX table test; uncited)

### (free functions)
- [ ] `STREAMPACK_STREAMPACKOBJ` · 0063c940 · legions 0063e9a0 — needs port: generic StreamPack op for int STREAMPACK_STREAMPACKOBJ(STREAMTYPE op, IconDesc * obj, void * * addr, uint * size)
- [ ] `STREAMPACK` · 0063c960 · legions 0050bc60 — needs port: generic StreamPack op for int STREAMPACK(STREAMTYPE op, FXNode * obj, void * * addr, uint * size)

### VisualDesc
- [x] `VisualDesc::Empty` · 0063caf0 · legions 005f1540

### IntrusiveHashIterator<unsigned_long,HashTableData<unsigned_long,PartGroupDataDesc>_*>
- [-] `IntrusiveHashIterator<unsigned_long,HashTableData<unsigned_long,PartGroupDataDesc>_*>::operator++` · 0063cb50 · legions 006003c0 — hash iterator op

### VisualDesc
- [-] `VisualDesc::GetPGDDescTableIter` · 0063cc30 — container iterator (PGD table); Rust iterates the map
- [-] `VisualDesc::GetPackBitfield` · 0063cc60 · legions 005f1600 — writer for DAT/pack output; the client only reads (tool-only)

### HashIterator<unsigned_long,PartGroupDataDesc>
- [-] `HashIterator<unsigned_long,PartGroupDataDesc>::operator++` · 0063cd60 · legions 00600810 — operator of plain data or container; Rust value semantics / derived PartialEq

### VisualDesc
- [-] `VisualDesc::ToFileNode` · 0063ce40 — text source format (admin tools)
- [ ] `VisualDesc::GetSubDataIDs` · 0063d1c0 — needs port: dependent DIDs of VisualDesc for sub-object resolution (AGENTS.md exception)
- [x] `VisualDesc::GetPGDDesc` · 0063d2f0 · legions 005f18e0 — ac2-weenie appearance.rs
- [x] `VisualDesc::ResetRootPointer` · 0063d310 · legions 005f1900 — ac2-weenie appearance_ops.rs (the decompile algebra over Appearance; unit tests in that file)
- [x] `VisualDesc::RemovePGDD` · 0063d3e0 · legions 005f19d0 — ac2-weenie appearance_ops.rs (the decompile algebra over Appearance; unit tests in that file)
- [x] `VisualDesc::ModVDesc_DeleteStragglers` · 0063d450 · legions 005f1a40 — ac2-weenie appearance_ops.rs (the decompile algebra over Appearance; unit tests in that file)
- [x] `VisualDesc::BuildAtomCollection` · 0063d680 · legions 005f1c70 — ac2-render object/appearance.rs over ac2-assets Ac2TotalVisualDesc
- [ ] `VisualDesc::GatherAppearanceKeys` · 0063d740 · legions 005f1d10 — needs port: appearance/visual record: VisualDesc::GatherAppearanceKeys
- [~] `VisualDesc::StreamPack` · 0063d7f0 — part-group FX overrides (FXData/FXNode) refused as unsupported
- [x] `VisualDesc::AddPGDDesc` · 0063db70 · legions 005f2140 — ac2-assets ac2_total_visual.rs (drawn description; ac2-weenie keeps the session copy)
- [x] `VisualDesc::HasAppliedAppKey` · 0063dcc0 · legions 005f2290 — ac2-weenie appearance_ops.rs (the decompile algebra over Appearance; unit tests in that file)
- [x] `VisualDesc::InqAppInfoHashForPartGroup` · 0063dd00 · legions 005f22d0 — ac2-weenie appearance_ops.rs (the decompile algebra over Appearance; unit tests in that file)
- [x] `VisualDesc::RemovePGDDRecursively` · 0063dd40 · legions 005f2310 — ac2-weenie appearance_ops.rs (the decompile algebra over Appearance; unit tests in that file)
- [x] `VisualDesc::RecursiveAdditivePGDDMerge` · 0063ddb0 · legions 005f2380 — ac2-assets ac2_total_visual.rs (drawn description; ac2-weenie keeps the session copy)
- [x] `VisualDesc::ApplyAdditiveVDesc` · 0063dea0 · legions 005f2470 — ac2-weenie appearance_ops.rs (the decompile algebra over Appearance; unit tests in that file)
- [x] `VisualDesc::Clear` · 0063e0d0 · legions 005f26a0 — ac2-weenie appearance_ops.rs (the decompile algebra over Appearance; unit tests in that file)
- [x] `VisualDesc::AddPGDDesc` · 0063e170 · legions 005f2740 — ac2-assets ac2_total_visual.rs (drawn description; ac2-weenie keeps the session copy)
- [x] `VisualDesc::MakeLocalPGDDFromParent` · 0063e1d0 · legions 005f27ba — ac2-weenie appearance_ops.rs (the decompile algebra over Appearance; unit tests in that file)
- [x] `VisualDesc::AddAppearanceKey` · 0063e370 · legions 005f2920 — ac2-weenie appearance_ops.rs (the decompile algebra over Appearance; unit tests in that file)
- [x] `VisualDesc::AddAppInfoHashToPartGroup` · 0063e3e0 · legions 005f2990 — ac2-weenie appearance_ops.rs (the decompile algebra over Appearance; unit tests in that file)
- [x] `VisualDesc::ApplyAdditiveVDesc` · 0063e460 · legions 005f2a10 — ac2-weenie appearance_ops.rs (the decompile algebra over Appearance; unit tests in that file)
- [-] `VisualDesc::FromFileNode` · 0063e4c0 — text source format (admin tools)
- [-] `VisualDesc::Destroy` · 0063eaf0 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)

### PHashTable<unsigned_long,PartGroupDataDesc>
- [ ] `PHashTable<unsigned_long,PartGroupDataDesc>::StreamPack` · 0063eb00 — needs port: appearance/visual record: PHashTable::StreamPack (stream decode)

### VisualDesc
- [-] `VisualDesc::VisualDesc` · 0063ecd0 · legions 005f2c50 — ctor/dtor of plain data; Rust constructs and drops values
- [-] `VisualDesc::GetDBOType` · 0063ed70 — DBO type tag; Rust uses typed resource ids
- [-] `VisualDesc::~VisualDesc` · 0063ed80 · legions 005f2d00 — ctor/dtor of plain data; Rust constructs and drops values
- [x] `VisualDesc::operator=` · 0063ee10 · legions 005f2d90 — ac2-weenie appearance_ops.rs (the decompile algebra over Appearance; unit tests in that file)
- [x] `VisualDesc::GetTotalVisualDescInternal` · 0063ee40 · legions 005f2dc0 — ac2-render object/appearance.rs over ac2-assets Ac2TotalVisualDesc
- [x] `VisualDesc::AddAppearanceKey` · 0063eee0 · legions 005f2e30 — ac2-weenie appearance_ops.rs (the decompile algebra over Appearance; unit tests in that file)
- [x] `VisualDesc::RemoveAppearanceKey` · 0063ef40 · legions 005f2e90 — ac2-weenie appearance_ops.rs (the decompile algebra over Appearance; unit tests in that file)
- [x] `VisualDesc::AttemptToRemoveVDesc` · 0063f070 · legions 005f2fa0 — ac2-weenie appearance_ops.rs (the decompile algebra over Appearance; unit tests in that file)
- [x] `VisualDesc::ReplaceVisualDesc` · 0063f250 · legions 005f3180 — ac2-weenie appearance_ops.rs (the decompile algebra over Appearance; unit tests in that file)
- [-] `VisualDesc::CopyInto` · 0063f340 · legions 005f3270 — copy of plain data; Rust clones values
- [ ] `VisualDesc::AddAppearanceMod` · 0063f380 · legions 005f32b0 — needs port: appearance/visual record: VisualDesc::AddAppearanceMod (atom insert rule)
- [ ] `VisualDesc::RemoveAppearanceMod` · 0063f390 · legions 005f32c0 — needs port: appearance/visual record: VisualDesc::RemoveAppearanceMod (atom removal rule)
- [x] `VisualDesc::GetVisualDescFromDB` · 0063f3a0 · legions 005f32d0 — ac2-weenie appearance_ops.rs (the decompile algebra over Appearance; unit tests in that file)
- [-] `VisualDesc::Allocate` · 0063f410 — retail DBO pool/cache release; Arc drop replaces it (AGENTS.md exception)
- [-] `VisualDesc::VisualDesc` · 0063f440 · legions 005f3370 — ctor/dtor of plain data; Rust constructs and drops values
- [x] `VisualDesc::ApplySubtractiveVDesc` · 0063f4d0 · legions 005f3400 — ac2-weenie appearance_ops.rs (the decompile algebra over Appearance; unit tests in that file)

### FXData
- [-] `FXData::FXData` · 0063f520 · legions 005f3450 — ctor/dtor of plain data; Rust constructs and drops values

### SmartArray<FXData>
- [-] `SmartArray<FXData>::operator=` · 0063f650 · legions 005f3580 — operator of plain data or container; Rust value semantics / derived PartialEq

### PSmartArray<FXData>
- [~] `PSmartArray<FXData>::StreamPack` · 0063f6c0 · legions 005f35f0 — unpack only, in ac2-net protocol::fx / protocol::visual (Legions FXNode::StreamPack 0x00642410 instruction-identical)

### HashTable<unsigned_long,PSmartArray<FXData>_>
- [-] `HashTable<unsigned_long,PSmartArray<FXData>_>::operator=` · 0063f770 · legions 005f36a0 — operator of plain data or container; Rust value semantics / derived PartialEq

### VisualDesc
- [x] `VisualDesc::SwapPGDD` · 0063f880 · legions 005f37b0 — ac2-render object/appearance.rs over ac2-assets Ac2TotalVisualDesc
- [x] `VisualDesc::ModifyPGDTreeBasedOnApr` · 0063f930 · legions 005f3860 — ac2-render object/appearance.rs over ac2-assets Ac2TotalVisualDesc
- [x] `VisualDesc::GetTotalVisualDesc` · 0063f9c0 · legions 005f38f0 — ac2-render object/appearance.rs over ac2-assets Ac2TotalVisualDesc
- [~] `VisualDesc::GetTotalIconDesc` · 0063f9f0 · legions 005f3920 — ac2-weenie total_icon_layers and glue GetTotalIconDesc: parent DAT icons and this icon; the appearance tables' icon modifiers (AugmentAtomCollection) not modelled

## `portal/shared/src/objdesc/weeniedesc.cpp.c` (2/2)

### WeenieDesc
- [x] `WeenieDesc::IsInWeenieDesc` · 00641ea0 · legions 005f56a0
- [x] `WeenieDesc::IsInWeenieDesc` · 00641fa0 · legions 005f57a0
