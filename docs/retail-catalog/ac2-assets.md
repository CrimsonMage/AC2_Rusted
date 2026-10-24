# ac2-assets: Render asset codecs: materials, meshes, appearance, images

785 retail functions: 7 done, 8 partial, 648 to do, 122 skipped.
Legend and workflow: [README](README.md).

## `ac2/shared/src/objdesc/gmweeniedesc.cpp.c` (0/62)

### gmWeenieDesc
- [ ] `gmWeenieDesc::~gmWeenieDesc` · 00601b00 · legions 005c29e0
- [ ] `gmWeenieDesc::Reset` · 00601b30 · legions 005c2a10
- [ ] `gmWeenieDesc::QueryIntStat` · 00601c80
- [ ] `gmWeenieDesc::QueryBoolStat` · 00601cd0
- [ ] `gmWeenieDesc::QueryFloatStat` · 00601d00
- [ ] `gmWeenieDesc::QueryDIDStat` · 00601d20
- [ ] `gmWeenieDesc::QueryIIDStat` · 00601d40
- [ ] `gmWeenieDesc::QueryStringInfoStat` · 00601e70
- [ ] `gmWeenieDesc::GetMyPackageID` · 00601e90
- [ ] `gmWeenieDesc::SetMyPackageID` · 00601ea0
- [ ] `gmWeenieDesc::GetEntityDID` · 00601ed0
- [ ] `gmWeenieDesc::SetEntityDID` · 00601ee0
- [ ] `gmWeenieDesc::AddInt` · 00601f10
- [ ] `gmWeenieDesc::RemoveInt` · 00601f20
- [ ] `gmWeenieDesc::RemoveInt` · 00601f40
- [ ] `gmWeenieDesc::InqInt` · 006021b0
- [ ] `gmWeenieDesc::SetInt` · 006023c0
- [ ] `gmWeenieDesc::AdjInt` · 00602590
- [ ] `gmWeenieDesc::AddLongInt` · 006026b0
- [ ] `gmWeenieDesc::RemoveLongInt` · 006026d0

### DBObj
- [ ] `DBObj::SaveToFile` · 006026f0

### gmWeenieDesc
- [ ] `gmWeenieDesc::RemoveBool` · 00602700
- [ ] `gmWeenieDesc::RemoveBool` · 00602720
- [ ] `gmWeenieDesc::InqBool` · 006027f0
- [ ] `gmWeenieDesc::SetBool` · 00602890
- [ ] `gmWeenieDesc::AddFloat` · 00602990
- [ ] `gmWeenieDesc::RemoveFloat` · 006029a0
- [ ] `gmWeenieDesc::RemoveFloat` · 006029c0
- [ ] `gmWeenieDesc::InqFloat` · 00602a00
- [ ] `gmWeenieDesc::SetFloat` · 00602a30
- [ ] `gmWeenieDesc::AdjFloat` · 00602a60
- [ ] `gmWeenieDesc::AddTimestamp` · 00602a90
- [ ] `gmWeenieDesc::RemoveTimestamp` · 00602ab0
- [ ] `gmWeenieDesc::AddDataID` · 00602ad0
- [ ] `gmWeenieDesc::RemoveDataID` · 00602ae0
- [ ] `gmWeenieDesc::RemoveDataID` · 00602b00
- [ ] `gmWeenieDesc::InqDataID` · 00602b30
- [ ] `gmWeenieDesc::SetDataID` · 00602b60
- [ ] `gmWeenieDesc::AddInstanceID` · 00602b90
- [ ] `gmWeenieDesc::RemoveInstanceID` · 00602bb0
- [ ] `gmWeenieDesc::RemoveInstanceID` · 00602bd0
- [ ] `gmWeenieDesc::InqInstanceID` · 00602e40
- [ ] `gmWeenieDesc::SetInstanceID` · 00603090
- [ ] `gmWeenieDesc::AddString` · 006032d0
- [ ] `gmWeenieDesc::AddPosition` · 006032e0
- [ ] `gmWeenieDesc::RemovePosition` · 006032f0
- [ ] `gmWeenieDesc::AddStringInfo` · 00603370
- [ ] `gmWeenieDesc::RemoveStringInfo` · 00603380
- [ ] `gmWeenieDesc::RemoveStringInfo` · 006033c0
- [ ] `gmWeenieDesc::InqStringInfo` · 00603420
- [ ] `gmWeenieDesc::SetStringInfo` · 00603470
- [ ] `gmWeenieDesc::AddPackageID` · 006034c0
- [ ] `gmWeenieDesc::RemovePackageID` · 006034d0
- [ ] `gmWeenieDesc::GetSubDataIDs` · 006034f0 · legions 005c43d0
- [ ] `gmWeenieDesc::gmWeenieDesc` · 00603530 · legions 005c4410
- [ ] `gmWeenieDesc::operator=` · 006035a0 · legions 005c4480
- [ ] `gmWeenieDesc::StreamPack` · 006037c0 · legions 005c46a0
- [ ] `gmWeenieDesc::gmWeenieDesc` · 00603b70 · legions 005c4a50
- [ ] `gmWeenieDesc::Copy` · 00603ba0
- [ ] `gmWeenieDesc::operator=` · 00603bf0
- [ ] `gmWeenieDesc::RemoveString` · 00603c00
- [ ] `gmWeenieDesc::Dump` · 00603c60 · legions 005c4b40

## `portal/client/src/image/graphic.cpp.c` (0/12)

### RenderSurfaceLocal
- [ ] `RenderSurfaceLocal::GetGlobalOrLocal` · 00568050 · legions 00537fe0

### Graphic
- [ ] `Graphic::Graphic` · 005680a0 · legions 00538030
- [ ] `Graphic::Incorporate` · 005680c0 · legions 00538050
- [ ] `Graphic::SetToDefaultGraphic` · 005680f0 · legions 00538080
- [ ] `Graphic::GetWidth` · 005681e0 · legions 00538170
- [ ] `Graphic::GetHeight` · 00568200 · legions 00538190
- [ ] `Graphic::PutImage` · 00568220 · legions 005381b0

### Box2D
- [ ] `Box2D::Intersection` · 005682d0 · legions 00538260

### Graphic
- [ ] `Graphic::Graphic` · 00568380 · legions 00538310
- [ ] `Graphic::Graphic` · 00568430 · legions 005383c0
- [ ] `Graphic::Draw` · 00568470 · legions 00538400
- [ ] `Graphic::InitFromIconLayerDesc` · 00568920 · legions 005388b0

## `portal/client/src/render/material/layermodifier.cpp.c` (4/46)

### LM_Origin
- [ ] `LM_Origin::LM_Origin` · 005d5a80

### LM_Normal
- [ ] `LM_Normal::LM_Normal` · 005d5b50

### LM_Origin
- [ ] `LM_Origin::GetSize` · 005d5c20

### LM_Normal
- [ ] `LM_Normal::operator=` · 005d5c30

### LM_Diffuse
- [ ] `LM_Diffuse::LM_Diffuse` · 005d5fa0
- [ ] `LM_Diffuse::GetSize` · 005d6000
- [ ] `LM_Diffuse::operator=` · 005d6010

### LM_UVTranslate
- [ ] `LM_UVTranslate::LM_UVTranslate` · 005d6170 · legions 00599170
- [x] `LM_UVTranslate::ApplyTextureTransform` · 005d61b0 · legions 00599250 — ac2-render object::pass / texture_matrix, ac2-gpu object

### LM_UVRotate
- [ ] `LM_UVRotate::LM_UVRotate` · 005d6270 · legions 00599310
- [ ] `LM_UVRotate::GetSize` · 005d62a0 · legions 00599350
- [ ] `LM_UVRotate::operator=` · 005d62b0 · legions 00599360
- [x] `LM_UVRotate::ApplyTextureTransform` · 005d6310 · legions 005993c0 — ac2-render object::pass / texture_matrix, ac2-gpu object

### LM_UVScale
- [ ] `LM_UVScale::LM_UVScale` · 005d6370 · legions 00599420

### LM_UVTranslate
- [ ] `LM_UVTranslate::GetSize` · 005d63c0 · legions 00599470
- [ ] `LM_UVTranslate::operator=` · 005d63d0 · legions 005991b0

### LM_UVScale
- [x] `LM_UVScale::ApplyTextureTransform` · 005d6470 — ac2-render object::pass / texture_matrix, ac2-gpu object

### LM_UVTransform
- [ ] `LM_UVTransform::LM_UVTransform` · 005d64f0 · legions 00599500

### FileNodeName_Bool
- [ ] `FileNodeName_Bool::GetType` · 005d6540 · legions 00599550

### LM_UVTransform
- [ ] `LM_UVTransform::GetSize` · 005d6550
- [x] `LM_UVTransform::ApplyTextureTransform` · 005d6560 — ac2-render object::pass / texture_matrix, ac2-gpu object

### (free functions)
- [ ] `STREAMPACK_SIZEOF` · 005d6590 · legions 005995a0

### LM_Origin
- [ ] `LM_Origin::StreamPack` · 005d65e0

### LM_Normal
- [ ] `LM_Normal::Copy` · 005d6710

### LM_Diffuse
- [ ] `LM_Diffuse::Copy` · 005d6750
- [ ] `LM_Diffuse::StreamPack` · 005d6790

### LM_UVTranslate
- [ ] `LM_UVTranslate::Copy` · 005d6860 · legions 005995f0
- [ ] `LM_UVTranslate::StreamPack` · 005d68c0 · legions 00599750

### LM_UVRotate
- [ ] `LM_UVRotate::Copy` · 005d6940 · legions 00599650
- [ ] `LM_UVRotate::StreamPack` · 005d6990 · legions 005996a0

### LM_UVScale
- [ ] `LM_UVScale::Copy` · 005d6a00

### LM_UVTransform
- [ ] `LM_UVTransform::Copy` · 005d6a40
- [ ] `LM_UVTransform::StreamPack` · 005d6ab0

### LM_Origin
- [ ] `LM_Origin::Copy` · 005d6b20
- [-] `LM_Origin::LoadFromFileNode` · 005d6b60 — text source loader, admin build only
- [ ] `LM_Origin::SaveToFileNode` · 005d6f00

### LM_Normal
- [-] `LM_Normal::LoadFromFileNode` · 005d7400 — text source loader, admin build only
- [ ] `LM_Normal::SaveToFileNode` · 005d77a0

### LM_Diffuse
- [-] `LM_Diffuse::LoadFromFileNode` · 005d7ca0 — text source loader, admin build only
- [ ] `LM_Diffuse::SaveToFileNode` · 005d7ee0

### LM_UVTranslate
- [ ] `LM_UVTranslate::LoadFromFileNode` · 005d8180 · legions 005998b0
- [ ] `LM_UVTranslate::SaveToFileNode` · 005d8370 · legions 00599aa0

### LM_UVRotate
- [ ] `LM_UVRotate::LoadFromFileNode` · 005d85a0 · legions 00599cd0
- [ ] `LM_UVRotate::SaveToFileNode` · 005d8750 · legions 00599e80

### LM_UVScale
- [-] `LM_UVScale::LoadFromFileNode` · 005d8910 — text source loader, admin build only
- [ ] `LM_UVScale::SaveToFileNode` · 005d8b00

## `portal/client/src/render/material/layerstage.cpp.c` (0/13)

### LayerStage
- [ ] `LayerStage::ReleaseSubObjects` · 005be2e0 · legions 00587290
- [ ] `LayerStage::StreamPack` · 005be300 · legions 005872b0
- [ ] `LayerStage::GetDIDsForFilenames` · 005be460
- [ ] `LayerStage::Begin` · 005be4b0 · legions 00587410
- [ ] `LayerStage::CopyInto` · 005be550 · legions 005874b0
- [ ] `LayerStage::Destroy` · 005be610 · legions 00587570
- [ ] `LayerStage::SetTexture` · 005be620 · legions 00587580
- [ ] `LayerStage::SetTexture` · 005be6c0 · legions 00587600
- [ ] `LayerStage::LayerStage` · 005be760 · legions 005876a0
- [ ] `LayerStage::Copy` · 005be790 · legions 005876d0
- [ ] `LayerStage::GetSubObjects` · 005be7e0 · legions 00587720
- [-] `LayerStage::LoadFromFileNode` · 005be850 — text source loader, admin build only
- [ ] `LayerStage::SaveToFileNode` · 005bf7a0

## `portal/client/src/render/material/materialinstance.cpp.c` (0/23)

### MaterialInstance
- [ ] `MaterialInstance::Refresh` · 005ba670
- [ ] `MaterialInstance::SetTemplateMaterial` · 005ba680 · legions 00584670
- [ ] `MaterialInstance::RestoreOriginalTemplateMaterial` · 005ba6c0 · legions 005846b0
- [ ] `MaterialInstance::GetSubDataIDs` · 005ba730 · legions 00584710
- [ ] `MaterialInstance::ReleaseSubObjects` · 005ba780 · legions 00584760
- [ ] `MaterialInstance::GetDIDsForFilenames` · 005ba800
- [ ] `MaterialInstance::RefreshMaterial` · 005ba8e0 · legions 005847e0
- [ ] `MaterialInstance::ReleaseModifier` · 005ba9e0 · legions 005848e0
- [ ] `MaterialInstance::Begin` · 005baa50 · legions 00584950
- [ ] `MaterialInstance::CopyInto` · 005baae0
- [ ] `MaterialInstance::GetSubObjects` · 005bad00
- [ ] `MaterialInstance::StreamPack` · 005badf0
- [-] `MaterialInstance::LoadFromFileNode` · 005baf20 — text source loader, admin build only
- [-] `MaterialInstance::ToFileNode` · 005bb140 — text source format (admin tools)
- [ ] `MaterialInstance::InsertModifier` · 005bb390 · legions 00584de0
- [ ] `MaterialInstance::MaterialInstance` · 005bb3d0 · legions 00584e20
- [ ] `MaterialInstance::GetDBOType` · 005bb430
- [ ] `MaterialInstance::End` · 005bb440 · legions 00584e90
- [ ] `MaterialInstance::Destroy` · 005bb4c0 · legions 00584f10
- [-] `MaterialInstance::FromFileNode` · 005bb4d0 — text source format (admin tools)
- [ ] `MaterialInstance::ReadInstanceFilename` · 005bb5e0
- [ ] `MaterialInstance::Allocate` · 005bb7c0
- [ ] `MaterialInstance::~MaterialInstance` · 005bb7d0 · legions 00584f30

## `portal/client/src/render/material/materiallayer.cpp.c` (2/20)

### MaterialLayer
- [ ] `MaterialLayer::Begin` · 005cb660 · legions 00592640
- [x] `MaterialLayer::BuildCompilerFlags` · 005cb7e0 · legions 005927c0 — ac2-render object::pass / texture_matrix, ac2-gpu object

### (free functions)
- [ ] `STREAMPACK_SIZEOF` · 005cb8b0 · legions 00592890

### MaterialLayer
- [ ] `MaterialLayer::End` · 005cb9b0 · legions 005928e0
- [ ] `MaterialLayer::Destroy` · 005cbad0 · legions 00592a00
- [~] `MaterialLayer::GetVertexShader` · 005cbae0 · legions 00592a10 — fragment.dll programs in object.wgsl: unlit, lit, lit vertex diffuse, per-pixel (sun); point-light variants not yet

### MaterialLayer::ShaderInfo
- [ ] `MaterialLayer::ShaderInfo::ShaderInfo` · 005cbbd0

### MaterialLayer
- [x] `MaterialLayer::BuildPixelShader` · 005cbbe0 · legions 00592b10 — object::parse_pixel_shader + ac2-gpu pixel_program (ps.1.1 to WGSL)

### SmartArray<MaterialLayer::ShaderInfo>
- [ ] `SmartArray<MaterialLayer::ShaderInfo>::~SmartArray<MaterialLayer::ShaderInfo>` · 005cbcb0 · legions 00592be0

### SmartArray<LScapeFarBlockEdge>
- [ ] `SmartArray<LScapeFarBlockEdge>::operator=` · 005cbcf0 · legions 00592c20

### MaterialLayer
- [ ] `MaterialLayer::MaterialLayer` · 005cbd70 · legions 00592ca0
- [ ] `MaterialLayer::CopyInto` · 005cbdc0 · legions 00592cf0
- [ ] `MaterialLayer::StreamPack` · 005cc0b0 · legions 00592fe0
- [-] `MaterialLayer::LoadFromFileNode` · 005cc6b0 — text source loader, admin build only
- [ ] `MaterialLayer::SaveToFileNode` · 005ce650
- [ ] `MaterialLayer::BuildVertexShader` · 005cfef0 · legions 005935e0
- [ ] `MaterialLayer::Copy` · 005d0320 · legions 00593a10

### SmartArray<MaterialLayer::ShaderInfo>
- [ ] `SmartArray<MaterialLayer::ShaderInfo>::AddToEnd` · 005d0360 · legions 00593a50

### MaterialLayer
- [~] `MaterialLayer::BuildVertexShaders` · 005d03d0 · legions 00593ac0 — fragment.dll programs in object.wgsl: unlit, lit, lit vertex diffuse, per-pixel (sun); point-light variants not yet
- [~] `MaterialLayer::PrepareShaders` · 005d06e0 · legions 00593dd0 — fragment.dll programs in object.wgsl: unlit, lit, lit vertex diffuse, per-pixel (sun); point-light variants not yet

## `portal/client/src/render/material/materialmodifier.cpp.c` (0/12)

### MaterialModifier
- [ ] `MaterialModifier::MaterialModifier` · 005b8fb0 · legions 00583a30
- [ ] `MaterialModifier::GetDBOType` · 005b8fe0
- [ ] `MaterialModifier::End` · 005b8ff0 · legions 00583a70
- [ ] `MaterialModifier::Destroy` · 005b9050
- [ ] `MaterialModifier::GetSubDataIDs` · 005b9060 · legions 00583ae0
- [ ] `MaterialModifier::Allocate` · 005b90b0
- [ ] `MaterialModifier::~MaterialModifier` · 005b90c0 · legions 00583b40
- [ ] `MaterialModifier::CopyInto` · 005b9130
- [ ] `MaterialModifier::StreamPack` · 005b91b0 · legions 00583c30
- [-] `MaterialModifier::LoadFromFileNode` · 005b9290 — text source loader, admin build only
- [-] `MaterialModifier::ToFileNode` · 005b94f0 — text source format (admin tools)
- [-] `MaterialModifier::FromFileNode` · 005b9580 — text source format (admin tools)

## `portal/client/src/render/material/materialproperty.cpp.c` (0/28)

### LM_Origin
- [ ] `LM_Origin::SetYScale` · 005da1a0 · legions 0059b890
- [ ] `LM_Origin::SetZScale` · 005da210 · legions 0059b900
- [ ] `LM_Origin::SetOriginPhase` · 005da280 · legions 0059b970
- [ ] `LM_Origin::SetNormalPhase` · 005da2f0 · legions 0059b9e0
- [ ] `LM_Origin::SetXTranslate` · 005da360 · legions 0059ba50
- [ ] `LM_Origin::SetYTranslate` · 005da3b0 · legions 0059baa0
- [ ] `LM_Origin::SetZTranslate` · 005da400 · legions 0059baf0
- [ ] `LM_Origin::SetXScale` · 005da450 · legions 0059b820

### LM_UVTranslate
- [ ] `LM_UVTranslate::SetUTranslate` · 005da4c0 · legions 00557790

### MaterialLayer
- [ ] `MaterialLayer::SetAlphaTestRef` · 005da510 · legions 0059bb40
- [ ] `MaterialLayer::SetSpecularPower` · 005da560 · legions 0059bb90

### MaterialProperty
- [ ] `MaterialProperty::SetWaveformData` · 005da5d0
- [ ] `MaterialProperty::SetColorData` · 005da650
- [ ] `MaterialProperty::SetBoolData` · 005da6a0
- [ ] `MaterialProperty::GetDataTypeForID` · 005da6e0

### (free functions)
- [ ] `STREAMPACK_BIT32` · 005da720 · legions 004f9b20

### MaterialProperty
- [ ] `MaterialProperty::GetDIDsForFilenames` · 005da750
- [ ] `MaterialProperty::Apply` · 005da830 · legions 0059bc00
- [ ] `MaterialProperty::Begin` · 005dabb0 · legions 0059bf80
- [ ] `MaterialProperty::End` · 005dac50 · legions 0059c020
- [ ] `MaterialProperty::Destroy` · 005dacf0 · legions 0059c0c0
- [ ] `MaterialProperty::MaterialProperty` · 005dad00 · legions 0059c0d0
- [ ] `MaterialProperty::~MaterialProperty` · 005dad50 · legions 0059c120
- [ ] `MaterialProperty::CopyInto` · 005dadb0 · legions 0059c180
- [ ] `MaterialProperty::StreamPack` · 005daf10 · legions 0059c2e0
- [-] `MaterialProperty::LoadFromFileNode` · 005db0b0 — text source loader, admin build only
- [ ] `MaterialProperty::SaveToFileNode` · 005db420
- [ ] `MaterialProperty::Copy` · 005db680 · legions 0059c4a0

## `portal/client/src/render/material/rendermaterial.cpp.c` (0/17)

### PStringBase<char>
- [ ] `PStringBase<char>::operator[]` · 005b9690

### RenderMaterial
- [ ] `RenderMaterial::RenderMaterial` · 005b96b0 · legions 00583d10
- [ ] `RenderMaterial::GetDBOType` · 005b9700
- [ ] `RenderMaterial::GetSubDataIDs` · 005b9710
- [~] `RenderMaterial::GetSubObjects` · 005b97c0 — hardware-class layer filter (object::layer_drawn, class 2 default)
- [ ] `RenderMaterial::ReleaseSubObjects` · 005b9ad0 · legions 00584130
- [ ] `RenderMaterial::GetDIDsForFilenames` · 005b9b50
- [ ] `RenderMaterial::Allocate` · 005b9bf0
- [ ] `RenderMaterial::End` · 005b9c00 · legions 005841c0
- [ ] `RenderMaterial::Destroy` · 005b9c70 · legions 00584230
- [ ] `RenderMaterial::CheckOutputField` · 005b9c90 · legions 00584250
- [ ] `RenderMaterial::CopyInto` · 005b9d50
- [ ] `RenderMaterial::StreamPack` · 005b9e00 · legions 005843c0
- [-] `RenderMaterial::LoadFromFileNode` · 005b9f30 — text source loader, admin build only
- [-] `RenderMaterial::ToFileNode` · 005ba250 — text source format (admin tools)
- [-] `RenderMaterial::FromFileNode` · 005ba430 — text source format (admin tools)
- [ ] `RenderMaterial::CheckParseField` · 005ba540 · legions 00584540

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
- [ ] `ILBMDecoder::ILBMDecoder` · 00733d30
- [ ] `ILBMDecoder::readILBMHeader` · 00733d50
- [ ] `ILBMDecoder::expandCompressedLine` · 00733f70
- [ ] `ILBMDecoder::ReadHeader` · 00734030
- [ ] `ILBMDecoder::expandBODY` · 00734080
- [ ] `ILBMDecoder::readData` · 007342b0
- [ ] `ILBMDecoder::Decode` · 00734380

## `portal/preproc/src/imagedec/imagedecoder.cpp.c` (0/7)

### ImageDecoder
- [ ] `ImageDecoder::ImageDecoder` · 00733a20
- [ ] `ImageDecoder::InitInternal` · 00733a60
- [ ] `ImageDecoder::GetLine` · 00733ae0
- [ ] `ImageDecoder::SetRGBAPixel` · 00733af0
- [ ] `ImageDecoder::Load` · 00733b80
- [ ] `ImageDecoder::Cleanup` · 00733bc0
- [ ] `ImageDecoder::Init` · 00733c10

## `portal/preproc/src/imagedec/tgadecoder.cpp.c` (0/10)

### TGADecoder
- [ ] `TGADecoder::TGADecoder` · 007343a0
- [ ] `TGADecoder::readTGAHeader` · 007343c0
- [ ] `TGADecoder::readPixel32` · 00734500
- [ ] `TGADecoder::ReadHeader` · 00734660
- [ ] `TGADecoder::readPalette` · 00734740
- [ ] `TGADecoder::expandUncompressedLine` · 00734790
- [ ] `TGADecoder::expandCompressedLine` · 007347e0
- [ ] `TGADecoder::readData` · 007348e0
- [ ] `TGADecoder::readImage` · 00734990
- [ ] `TGADecoder::Decode` · 007349f0

## `portal/shared/src/entitydesc/entitydesc.cpp.c` (1/41)

### LM_UVScale
- [ ] `LM_UVScale::GetType` · 00615d90 · legions 005d3ac0

### FileNodeName_InstanceID
- [ ] `FileNodeName_InstanceID::Clone` · 00615da0 · legions 005d3ad0

### PFileNode
- [ ] `PFileNode::SetNameAsInstanceID` · 00615dd0 · legions 005d3b00
- [ ] `PFileNode::MakeSubNodeAsInstanceID` · 00615e20

### EntityDesc
- [ ] `EntityDesc::SetProperties` · 00615e50 · legions 005d3b50
- [ ] `EntityDesc::InqBool` · 00615e70 · legions 005d3b70
- [ ] `EntityDesc::InqInteger` · 00615ea0 · legions 005d3ba0
- [ ] `EntityDesc::InqFloat` · 00615ed0 · legions 005d3bd0
- [ ] `EntityDesc::InqVector` · 00615f00 · legions 005d3c00
- [ ] `EntityDesc::InqEnum` · 00615f30 · legions 005d3c30
- [ ] `EntityDesc::InqDataID` · 00615f60 · legions 005d3c60
- [ ] `EntityDesc::InqPackageID` · 00615f90 · legions 005d3c90
- [ ] `EntityDesc::InqStringInfo` · 00615fc0 · legions 005d3cc0
- [ ] `EntityDesc::GetSubDataIDs` · 00615ff0 · legions 005d3cf0
- [ ] `EntityDesc::SetDataFileName` · 00616030
- [ ] `EntityDesc::GetPackBitfield` · 00616080 · legions 005d3d30

### (free functions)
- [ ] `STREAMPACK_STREAMPACKOBJ` · 006161b0 · legions 005d3e60

### PFileNode
- [ ] `PFileNode::GetNameAsInstanceID` · 00616200

### EntityDesc
- [ ] `EntityDesc::EntityDesc` · 006162d0 · legions 005d3eb0
- [ ] `EntityDesc::GetDBOType` · 00616390
- [ ] `EntityDesc::operator=` · 006163a0 · legions 005d3f80
- [ ] `EntityDesc::~EntityDesc` · 006164c0 · legions 005d40a0
- [ ] `EntityDesc::InqDataFile` · 00616520
- [ ] `EntityDesc::InqDataFileW` · 006165d0 · legions 005d41b0
- [ ] `EntityDesc::InqDataFilePath` · 00616690
- [ ] `EntityDesc::InqDataFilePathW` · 00616720 · legions 005d4300
- [ ] `EntityDesc::InqStringW` · 006167d0 · legions 005d43b0
- [ ] `EntityDesc::CopyInto` · 00616800 · legions 005d43e0
- [ ] `EntityDesc::Destroy` · 00616820
- [ ] `EntityDesc::GetRootPackageID` · 00616960 · legions 005d4540
- [x] `EntityDesc::GetTotalProperties` · 006169c0 · legions 005d45a0
- [ ] `EntityDesc::GetVisualDescID` · 00616a50 · legions 005d4610
- [ ] `EntityDesc::InqEntityDescFromDataID` · 00616ac0 · legions 005d4680
- [ ] `EntityDesc::Allocate` · 00616b20
- [ ] `EntityDesc::EntityDesc` · 00616b30 · legions 005d46f0
- [-] `EntityDesc::FromFileNode` · 00616bb0 — text source format (admin tools)
- [-] `EntityDesc::ToFileNode` · 00616fa0 — text source format (admin tools)
- [ ] `EntityDesc::GetEntityNameFromDataIDW` · 00617320 · legions 005d4770
- [~] `EntityDesc::StreamPack` · 00617370 · legions 005d47c0 — unpack only, in ac2-net protocol::fx; property collection (bit 0x80) refused: value types come from the MasterProperty DAT table
- [ ] `EntityDesc::GetDataIDFromEntityNameW` · 006174d0 · legions 005d4920
- [ ] `EntityDesc::InqEntityDescFromNameW` · 00617520 · legions 005d4970

## `portal/shared/src/entitydesc/entitygroupdesc.cpp.c` (0/13)

### EntityGroupDesc
- [ ] `EntityGroupDesc::StreamPack` · 00617580 · legions 005d49d0
- [ ] `EntityGroupDesc::GetSubDataIDs` · 00617600 · legions 005d4a50
- [ ] `EntityGroupDesc::Destroy` · 00617640 · legions 005d4a90

### SmartArray<EntityDesc>
- [ ] `SmartArray<EntityDesc>::operator=` · 006176b0 · legions 005d4b00

### EntityGroupDesc
- [ ] `EntityGroupDesc::EntityGroupDesc` · 00617720 · legions 005d4b70
- [ ] `EntityGroupDesc::GetDBOType` · 00617760
- [ ] `EntityGroupDesc::~EntityGroupDesc` · 00617770 · legions 005d4bc0

### SmartArray<EntityLinkDesc>
- [ ] `SmartArray<EntityLinkDesc>::operator=` · 006177e0 · legions 005d4c30

### PSmartArray<EntityLinkDesc>
- [ ] `PSmartArray<EntityLinkDesc>::StreamPack` · 00617850 · legions 005d4ca0

### EntityGroupDesc
- [ ] `EntityGroupDesc::Allocate` · 006178f0
- [-] `EntityGroupDesc::ToFileNode` · 00617920 — text source format (admin tools)
- [ ] `EntityGroupDesc::CopyInto` · 00617b10 · legions 005d4d70
- [-] `EntityGroupDesc::FromFileNode` · 00617b70 — text source format (admin tools)

## `portal/shared/src/entitydesc/entitylinkdesc.cpp.c` (0/10)

### EntityLinkDesc
- [ ] `EntityLinkDesc::GetOppositeID` · 00617e70 · legions 005d4dd0
- [ ] `EntityLinkDesc::EntityLinkDesc` · 00617ee0 · legions 005d4e40
- [ ] `EntityLinkDesc::GetWBName_ForWsl` · 00617f40 · legions 005d4ea0
- [ ] `EntityLinkDesc::InqStringPropertyW` · 00617f60 · legions 005d4ec0

### LinkDesc
- [ ] `LinkDesc::InqLinkType` · 00617fc0
- [-] `LinkDesc::ToFileNode` · 00618000 — text source format (admin tools)
- [-] `LinkDesc::FromFileNode` · 00618110 — text source format (admin tools)

### EntityLinkDesc
- [-] `EntityLinkDesc::ToFileNode` · 006182e0 — text source format (admin tools)
- [-] `EntityLinkDesc::FromFileNode` · 00618400 — text source format (admin tools)
- [ ] `EntityLinkDesc::StreamPack` · 00618540 · legions 005d4f20

## `portal/shared/src/entitydesc/linkrequirements.cpp.c` (0/3)

### LinkRequirements
- [ ] `LinkRequirements::LinkRequirements` · 006185b0 · legions 005d4f90
- [ ] `LinkRequirements::LinkRequirements` · 006185f0 · legions 005d4fd0
- [ ] `LinkRequirements::StreamPack` · 00618610 · legions 005d4ff0

## `portal/shared/src/mesh/basemesh.cpp.c` (0/50)

### CMeshTriList
- [ ] `CMeshTriList::GetMeshFragment` · 004cfbe0 · legions 004c3db0
- [ ] `CMeshTriList::GetIndexBuffer` · 004cfc00 · legions 004c3dd0

### MeshMaterials::MaterialAndDataID
- [ ] `MeshMaterials::MaterialAndDataID::MaterialAndDataID` · 004cfc20

### MaterialInstance
- [ ] `MaterialInstance::Get` · 004cfc40 · legions 004c3e10

### CBaseMesh
- [ ] `CBaseMesh::HasMatrixWeights` · 004cfc60 · legions 004c3e30
- [ ] `CBaseMesh::BuildBoundingBox` · 004cfc80 · legions 004c3e50
- [ ] `CBaseMesh::BuildDefaultBoundingBox` · 004cfca0 · legions 004c3e70

### (free functions)
- [ ] `STREAMPACK_PACKOBJ` · 004cfcc0 · legions 004c3e90
- [ ] `STREAMPACK_PACKOBJ` · 004cfd30 · legions 004c3f00

### CMeshDegradeList
- [ ] `CMeshDegradeList::CMeshDegradeList` · 004cfda0 · legions 004c3f70

### CBaseMesh
- [ ] `CBaseMesh::GetNumDegradeLevels` · 004cfde0 · legions 004c3fe0

### MeshMaterials
- [ ] `MeshMaterials::GetSubObjects` · 004cfe00 · legions 004c4000
- [ ] `MeshMaterials::ReleaseSubObjects` · 004cfeb0 · legions 004c4090
- [ ] `MeshMaterials::GetSubDataIDs` · 004cfef0 · legions 004c40d0

### CBaseMesh
- [ ] `CBaseMesh::ReleaseVertexArray` · 004cff30 · legions 004c4110
- [ ] `CBaseMesh::InitVertexArray` · 004cff60 · legions 004c4140
- [ ] `CBaseMesh::SetMaterial` · 004cfff0 · legions 004c41d0
- [ ] `CBaseMesh::SetMaterialID` · 004d0050 · legions 004c4230
- [ ] `CBaseMesh::DrawMeshDegrade` · 004d0080 · legions 004c4260
- [ ] `CBaseMesh::GenerateLocalIndices` · 004d0320 · legions 004c44d0
- [ ] `CBaseMesh::GenerateTriangleData` · 004d0530 · legions 004c46e0
- [ ] `CBaseMesh::GenerateConnectivityData` · 004d0820 · legions 004c49d0
- [ ] `CBaseMesh::GetSubObjects` · 004d08c0 · legions 004c4a70
- [ ] `CBaseMesh::ReleaseSubObjects` · 004d08f0 · legions 004c4aa0
- [ ] `CBaseMesh::GetSubDataIDs` · 004d0910 · legions 004c4ac0
- [ ] `CBaseMesh::GenerateShadowVolume` · 004d0920 · legions 004c4ad0
- [ ] `CBaseMesh::CBaseMesh` · 004d1090 · legions 004c5240
- [ ] `CBaseMesh::Destroy` · 004d10c0 · legions 004c5270

### PSmartArray<CPolygon>
- [ ] `PSmartArray<CPolygon>::StreamPack` · 004d1260 · legions 004c5410

### CBaseMesh
- [ ] `CBaseMesh::~CBaseMesh` · 004d1450 · legions 004c5600
- [ ] `CBaseMesh::DrawMesh` · 004d1460 · legions 004c5610
- [ ] `CBaseMesh::ParseVertexArrayFileNode` · 004d17c0

### PSmartArray<MeshMaterials::MaterialAndDataID>
- [ ] `PSmartArray<MeshMaterials::MaterialAndDataID>::StreamPack` · 004d1830 · legions 004c5970

### CMeshDegradeList
- [ ] `CMeshDegradeList::Create` · 004d18d0 · legions 004c5a10
- [ ] `CMeshDegradeList::UpdateDetailLevelIndices` · 004d1930 · legions 004c5a70
- [ ] `CMeshDegradeList::Destroy` · 004d19c0 · legions 004c5b00

### MeshMaterials
- [ ] `MeshMaterials::DynamicCopy` · 004d1a30 · legions 004c5b70

### CMeshDegradeList
- [ ] `CMeshDegradeList::~CMeshDegradeList` · 004d1ba0 · legions 004c5ce0

### MeshGeometry
- [ ] `MeshGeometry::~MeshGeometry` · 004d1ec0 · legions 004c5f20

### CBaseMesh
- [ ] `CBaseMesh::ParseMorphTargetNode` · 004d1fc0
- [ ] `CBaseMesh::ParseVertexSetNode` · 004d2240

### PHashTable<unsigned_long,CMorphTargetData>
- [ ] `PHashTable<unsigned_long,CMorphTargetData>::StreamPack` · 004d24e0

### PHashTable<unsigned_long,CVertexSetData>
- [ ] `PHashTable<unsigned_long,CVertexSetData>::StreamPack` · 004d26d0

### MeshGeometry
- [ ] `MeshGeometry::MeshGeometry` · 004d28c0 · legions 004c6420

### CBaseMesh
- [ ] `CBaseMesh::InitMesh` · 004d2950 · legions 004c64b0
- [ ] `CBaseMesh::SetDegradeList` · 004d29d0 · legions 004c6530
- [ ] `CBaseMesh::AllocateMaterials` · 004d2a40 · legions 004c65a0
- [ ] `CBaseMesh::StreamPack` · 004d2ab0 · legions 004c6610
- [ ] `CBaseMesh::ParseMaterialArrayNode` · 004d2cd0
- [ ] `CBaseMesh::ParseRenderFileNode` · 004d2e80

## `portal/shared/src/mesh/cellmesh.cpp.c` (0/15)

### CCellMesh
- [ ] `CCellMesh::ReleaseMesh` · 004cef30
- [ ] `CCellMesh::PointInCell` · 004cef40 · legions 004c3510
- [ ] `CCellMesh::GetSubDataIDs` · 004cef50
- [ ] `CCellMesh::GetCellPlanes` · 004cefc0 · legions 004c3590
- [ ] `CCellMesh::CCellMesh` · 004ceff0 · legions 004c35c0
- [ ] `CCellMesh::DynamicCastCellMesh` · 004cf040
- [ ] `CCellMesh::GetDID` · 004cf050
- [ ] `CCellMesh::GetSubObjects` · 004cf060
- [ ] `CCellMesh::ReleaseSubObjects` · 004cf070
- [ ] `CCellMesh::~CCellMesh` · 004cf090 · legions 004c3660
- [ ] `CCellMesh::Allocate` · 004cf110
- [-] `CCellMesh::LoadFromFile` · 004cf140 — text source loader, admin build only

### SmartArray<CPortalPoly>
- [ ] `SmartArray<CPortalPoly>::SetNElements` · 004cf370 · legions 004c3830

### CCellMesh
- [ ] `CCellMesh::StreamPack` · 004cf3a0
- [-] `CCellMesh::FromFileNode` · 004cf550 — text source format (admin tools)

## `portal/shared/src/mesh/datamesh.cpp.c` (0/19)

### CDataMesh
- [ ] `CDataMesh::CDataMesh` · 004d3930 · legions 004c6bf0
- [ ] `CDataMesh::GetDID` · 004d3960 · legions 004c6c20
- [ ] `CDataMesh::Destroy` · 004d3970
- [ ] `CDataMesh::ReleaseMesh` · 004d3980 · legions 004c6c40
- [ ] `CDataMesh::GetSubDataIDs` · 004d3990
- [ ] `CDataMesh::GetSubObjects` · 004d39a0 · legions 004c6c60
- [ ] `CDataMesh::ReleaseSubObjects` · 004d39c0 · legions 004c6c80

### PFileNode
- [ ] `PFileNode::MakeSubNodeAsUInt32` · 004d39f0 · legions 004c6cb0

### CDataMesh
- [ ] `CDataMesh::Allocate` · 004d3a50

### FileNodeName_PString
- [ ] `FileNodeName_PString::FileNodeName_PString` · 004d3aa0

### PFileNode
- [ ] `PFileNode::SetNameAsPString` · 004d3ad0 · legions 004c6d60
- [ ] `PFileNode::MakeSubNodeAsPString` · 004d3b40 · legions 004c6dd0
- [ ] `PFileNode::MakeSubNode` · 004d3bb0 · legions 004c6e40

### CDataMesh
- [ ] `CDataMesh::StreamPack` · 004d3bf0 · legions 004c6e80
- [ ] `CDataMesh::ParseMeshInfoFileNode` · 004d3c70
- [ ] `CDataMesh::ToMeshInfoFileNode` · 004d3e50
- [-] `CDataMesh::FromFileNode` · 004d3f90 — text source format (admin tools)
- [-] `CDataMesh::LoadFromFile` · 004d4030 — text source loader, admin build only
- [ ] `CDataMesh::SaveToFile` · 004d4220

## `portal/shared/src/mesh/dynamicmesh.cpp.c` (0/4)

### CStaticMesh
- [ ] `CStaticMesh::ReleaseMesh` · 004d4990 · legions 004c74d0

### CDynamicMesh
- [ ] `CDynamicMesh::ApplyMaterialModifier` · 004d49a0 · legions 004c74e0
- [ ] `CDynamicMesh::DeleteMaterialModifiers` · 004d4a40 · legions 004c7580
- [ ] `CDynamicMesh::InitDynamicMesh` · 004d4ab0 · legions 004c75f0

## `portal/shared/src/mesh/meshgroup.cpp.c` (0/13)

### CDataMesh
- [ ] `CDataMesh::Get` · 004d43b0 · legions 004c6f00

### HashBase<unsigned_long>
- [ ] `HashBase<unsigned_long>::~HashBase<unsigned_long>` · 004d4410 · legions 004c6f60
- [ ] `HashBase<unsigned_long>::InternalInit` · 004d4430 · legions 004c6fc0

### CMeshGroup
- [ ] `CMeshGroup::CleanupMesh` · 004d4490 · legions 004c7020
- [ ] `CMeshGroup::InitCellMesh` · 004d44e0 · legions 004c7070
- [ ] `CMeshGroup::InitStaticMesh` · 004d4580 · legions 004c7100
- [ ] `CMeshGroup::GetMesh` · 004d4610 · legions 004c7190
- [ ] `CMeshGroup::DestroyDynamicMesh` · 004d4630 · legions 004c71b0
- [ ] `CMeshGroup::DrawMesh` · 004d4670 · legions 004c71f0

### HashBase<unsigned_long>
- [ ] `HashBase<unsigned_long>::remove` · 004d46c0 · legions 004c7240

### CMeshGroup
- [ ] `CMeshGroup::InitDatabaseMesh` · 004d4790 · legions 004c72e0
- [ ] `CMeshGroup::Release` · 004d48e0 · legions 004c7420
- [ ] `CMeshGroup::SpawnDynamicMesh` · 004d4920 · legions 004c7460

## `portal/shared/src/mesh/meshinstance.cpp.c` (0/15)

### CMeshInstance
- [ ] `CMeshInstance::CMeshInstance` · 004cf840 · legions 004c3a10
- [ ] `CMeshInstance::GetStaticMesh` · 004cf860 · legions 004c3a30
- [ ] `CMeshInstance::GetMesh` · 004cf870 · legions 004c3a40
- [ ] `CMeshInstance::InitDatabaseMesh` · 004cf880 · legions 004c3a50
- [ ] `CMeshInstance::InitCellMesh` · 004cf8b0 · legions 004c3a80
- [ ] `CMeshInstance::InitStaticMesh` · 004cf8e0 · legions 004c3ab0
- [ ] `CMeshInstance::ReleaseMesh` · 004cf900 · legions 004c3ad0
- [ ] `CMeshInstance::PerformMeshReplacement` · 004cf930 · legions 004c3b00
- [ ] `CMeshInstance::DrawMesh` · 004cf9a0 · legions 004c3b70
- [ ] `CMeshInstance::~CMeshInstance` · 004cf9f0 · legions 004c3bc0
- [ ] `CMeshInstance::GetCellMesh` · 004cfa20 · legions 004c3bf0
- [ ] `CMeshInstance::GetDynamicMesh` · 004cfa40 · legions 004c3c10
- [ ] `CMeshInstance::RemoveMaterialModifiers` · 004cfa60 · legions 004c3c30
- [ ] `CMeshInstance::MakeDynamic` · 004cfab0 · legions 004c3c80
- [ ] `CMeshInstance::ApplyMaterialModifier` · 004cfb60 · legions 004c3d30

## `portal/shared/src/mesh/morphtargetdata.cpp.c` (0/5)

### CMorphTargetData
- [ ] `CMorphTargetData::StreamPack` · 004d4c40 · legions 004c7780
- [ ] `CMorphTargetData::CMorphTargetData` · 004d4ca0 · legions 004c77e0
- [ ] `CMorphTargetData::~CMorphTargetData` · 004d4cc0 · legions 004c7800
- [-] `CMorphTargetData::FromFileNode` · 004d4d30 — text source format (admin tools)
- [ ] `CMorphTargetData::CMorphTargetData` · 004d4da0

## `portal/shared/src/mesh/staticmesh.cpp.c` (0/13)

### CStaticMesh
- [ ] `CStaticMesh::BuildRenderBoundingBox` · 004d3190 · legions 004c6830
- [ ] `CStaticMesh::BuildDefaultRenderBoundingBox` · 004d31a0 · legions 004c6840
- [ ] `CStaticMesh::BuildPhysicsBoundingBox` · 004d31b0 · legions 004c6850
- [ ] `CStaticMesh::CalcPhysicsContainmentRadius` · 004d3230 · legions 004c68d0
- [ ] `CStaticMesh::StreamPack` · 004d32d0 · legions 004c6970
- [ ] `CStaticMesh::GetSubDataIDs` · 004d33b0 · legions 004c6a50
- [ ] `CStaticMesh::CStaticMesh` · 004d3400 · legions 004c6aa0
- [ ] `CStaticMesh::Destroy` · 004d3440 · legions 004c6ae0
- [ ] `CStaticMesh::~CStaticMesh` · 004d34a0 · legions 004c6b40
- [ ] `CStaticMesh::ParsePhysicsMaterialNode` · 004d3500
- [ ] `CStaticMesh::AllocatePhysicsPolygons` · 004d36b0 · legions 004c6bc0
- [ ] `CStaticMesh::ParsePhysicsFileNode` · 004d36e0
- [-] `CStaticMesh::FromFileNode` · 004d37f0 — text source format (admin tools)

## `portal/shared/src/mesh/vertexsetdata.cpp.c` (0/4)

### CVertexSetData
- [ ] `CVertexSetData::CVertexSetData` · 004d4e00 · legions 004c78a0
- [ ] `CVertexSetData::~CVertexSetData` · 004d4e20 · legions 004c78c0
- [-] `CVertexSetData::FromFileNode` · 004d4e90 — text source format (admin tools)
- [ ] `CVertexSetData::CVertexSetData` · 004d4f60

## `portal/shared/src/objdesc/appearance/appearanceinfo.cpp.c` (0/13)

### AppearanceInfo
- [ ] `AppearanceInfo::StreamPack` · 0064a5b0
- [ ] `AppearanceInfo::GetAppliedAppKeyIter` · 0064a5e0 · legions 005f9c90
- [ ] `AppearanceInfo::GetAppliedAppKeyEnd` · 0064a610 · legions 005f9cc0
- [ ] `AppearanceInfo::AugmentAtomCollection` · 0064a630 · legions 005f9ce0
- [ ] `AppearanceInfo::RemoveAppearanceKey` · 0064a730 · legions 005f9dd0
- [ ] `AppearanceInfo::HasAppliedAppKey` · 0064a760 · legions 005f9e00
- [ ] `AppearanceInfo::operator=` · 0064a7c0 · legions 005f9e60
- [ ] `AppearanceInfo::~AppearanceInfo` · 0064a7e0 · legions 005f9e80
- [ ] `AppearanceInfo::AddAppliedAppKey` · 0064a840 · legions 005f9ee0
- [ ] `AppearanceInfo::RemoveAppearanceKeys` · 0064a870 · legions 005f9f10
- [ ] `AppearanceInfo::AugmentExistingKeys` · 0064a970 · legions 005fa010
- [ ] `AppearanceInfo::AppearanceInfo` · 0064aa60 · legions 005fa100
- [ ] `AppearanceInfo::AppearanceInfo` · 0064aaa0 · legions 005fa140

## `portal/shared/src/objdesc/appearance/appearancetable.cpp.c` (0/24)

### (free functions)
- [ ] `STREAMPACK` · 00648cf0 · legions 00486a30

### AppearanceTable
- [ ] `AppearanceTable::StreamPack` · 00648d80 · legions 005f8d00
- [ ] `AppearanceTable::Destroy` · 00648f70 · legions 005f8f70
- [-] `AppearanceTable::ToFileNode` · 00648f90 — text source format (admin tools)
- [ ] `AppearanceTable::GetSubDataIDs` · 00649260
- [ ] `AppearanceTable::AddBaseAppearanceDID` · 00649330
- [ ] `AppearanceTable::GetAtoms` · 00649370 · legions 005f9060
- [ ] `AppearanceTable::HasAppearanceKey` · 00649470 · legions 005f9140
- [ ] `AppearanceTable::InqAppearanceKeyModList` · 00649540 · legions 005f91f0
- [ ] `AppearanceTable::InqTsysAppearanceKeyModCount` · 00649650 · legions 005f92e0
- [ ] `AppearanceTable::CalculateAppearanceModsBelowThreshold` · 006497a0

### PHashTable<unsigned_long,VisualModTable>
- [ ] `PHashTable<unsigned_long,VisualModTable>::StreamPack` · 006499a0

### AppearanceTable
- [ ] `AppearanceTable::AppearanceTable` · 00649ba0 · legions 005f9630
- [ ] `AppearanceTable::GetDBOType` · 00649c30
- [ ] `AppearanceTable::~AppearanceTable` · 00649c60 · legions 005f96f0
- [ ] `AppearanceTable::Allocate` · 00649d10

### PList<AtomCollection>
- [ ] `PList<AtomCollection>::Pack` · 00649e20 · legions 005f98b0
- [ ] `PList<AtomCollection>::GetPackSize` · 00649e70
- [ ] `PList<AtomCollection>::UnPack` · 00649eb0 · legions 005f995a

### HashTable<unsigned_long,VisualModTable>
- [ ] `HashTable<unsigned_long,VisualModTable>::operator=` · 0064a060 · legions 005f9af0

### AppearanceTable
- [ ] `AppearanceTable::VMTsFromFileNode` · 0064a210
- [ ] `AppearanceTable::operator=` · 0064a370 · legions 005f9c10
- [ ] `AppearanceTable::CopyInto` · 0064a3d0 · legions 005f9c70
- [-] `AppearanceTable::FromFileNode` · 0064a3f0 — text source format (admin tools)

## `portal/shared/src/objdesc/appearance/appearancetableadaptor.cpp.c` (0/3)

### AppearanceTableAdaptor
- [ ] `AppearanceTableAdaptor::HasAppearanceKey` · 0064aaf0 · legions 005fa190
- [ ] `AppearanceTableAdaptor::InqTsysAppearanceKeyModCount` · 0064ab50 · legions 005fa1e0
- [ ] `AppearanceTableAdaptor::InqAppearanceKeyModList` · 0064abc0

## `portal/shared/src/objdesc/appearance/appliedappearancekey.cpp.c` (0/7)

### AppliedAppearanceKey
- [ ] `AppliedAppearanceKey::AppliedAppearanceKey` · 00641fc0 · legions 005f57c0
- [ ] `AppliedAppearanceKey::operator=` · 00641fe0 · legions 005f57e0

### CAmbientSoundInfo
- [ ] `CAmbientSoundInfo::operator==` · 00642000 · legions 005f5800

### AppliedAppearanceKey
- [ ] `AppliedAppearanceKey::AppliedAppearanceKey` · 00642020 · legions 005f5820
- [ ] `AppliedAppearanceKey::AppearanceKeyFFN` · 00642040 · legions 005f5890
- [ ] `AppliedAppearanceKey::ValueFFN` · 006421f0 · legions 005f5a40
- [-] `AppliedAppearanceKey::FromFileNode` · 00642260 · legions 005f5ab0 — text source format (admin tools)

## `portal/shared/src/objdesc/appearance/atomcollection.cpp.c` (0/30)

### FXAtom
- [ ] `FXAtom::FXAtom` · 00646840
- [ ] `FXAtom::~FXAtom` · 00646880 · legions 005f74d0

### AtomCollection
- [ ] `AtomCollection::GetSubDataIDs` · 00646990 · legions 005f75e0
- [-] `AtomCollection::ToFileNode` · 00646aa0 — text source format (admin tools)
- [ ] `AtomCollection::AddIconModAtom` · 00646fa0
- [ ] `AtomCollection::AddReplacementIconModAtom` · 00646fd0 · legions 005f7870

### List<IconModAtom>
- [ ] `List<IconModAtom>::operator=` · 00647130 · legions 005f79d0

### AtomCollection
- [ ] `AtomCollection::AddOpacityAtom` · 006471d0
- [ ] `AtomCollection::AddMeshReplacementAtom` · 00647200
- [ ] `AtomCollection::AddPGDReplacementAtom` · 00647230
- [ ] `AtomCollection::AddReplacementOpacityAtom` · 00647260 · legions 005f7a70
- [ ] `AtomCollection::AddReplacementMeshReplacementAtom` · 00647320 · legions 005f7b30
- [ ] `AtomCollection::AddReplacementPGDReplacementAtom` · 006473e0 · legions 005f7bf0

### PList<MaterialModifierAtom>
- [ ] `PList<MaterialModifierAtom>::operator=` · 006474c0 · legions 005f7cd0

### PList<OpacityAtom>
- [ ] `PList<OpacityAtom>::operator=` · 00647500 · legions 005f7d10

### PList<MeshReplacementAtom>
- [ ] `PList<MeshReplacementAtom>::operator=` · 00647540 · legions 005f7d50

### PList<PGDReplacementAtom>
- [ ] `PList<PGDReplacementAtom>::operator=` · 00647580 · legions 005f7d90

### AtomCollection
- [ ] `AtomCollection::AddReplacementMatModAtom` · 006475c0 · legions 005f7dd0
- [ ] `AtomCollection::Clear` · 006476e0 · legions 005f7ef0
- [ ] `AtomCollection::AtomCollection` · 00647780

### FXAtom
- [ ] `FXAtom::FXAtom` · 006478a0 · legions 005f7f90

### _STL::_Rb_tree<FXAtom,FXAtom,_STL::_Identity<FXAtom>,_STL::less<FXAtom>,_STL::allocator<FXAtom>_>
- [ ] `_STL::_Rb_tree<FXAtom,FXAtom,_STL::_Identity<FXAtom>,_STL::less<FXAtom>,_STL::allocator<FXAtom>_>::operator=` · 00647b60 · legions 005f8250

### AtomCollection
- [ ] `AtomCollection::operator=` · 00647f20 · legions 005f8610
- [ ] `AtomCollection::AddFXAtom` · 00647fe0
- [ ] `AtomCollection::AddReplacementFXAtom` · 00648000 · legions 005f86d0
- [-] `AtomCollection::FromFileNode` · 006480e0 — text source format (admin tools)

### (free functions)
- [ ] `STREAMPACK_STL_SET` · 006487a0 · legions 005f87b0

### AtomCollection
- [ ] `AtomCollection::AtomCollection` · 00648990 · legions 005f89a0
- [ ] `AtomCollection::UpdateAtomCollection` · 00648a80 · legions 005f8a90
- [ ] `AtomCollection::StreamPack` · 00648c40 · legions 005f8c50

## `portal/shared/src/objdesc/appearance/atoms.cpp.c` (0/43)

### MaterialModifierAtom
- [ ] `MaterialModifierAtom::Clear` · 00642410 · legions 005f5c60
- [ ] `MaterialModifierAtom::operator==` · 00642430 · legions 005f5c80
- [ ] `MaterialModifierAtom::Replaces` · 00642470 · legions 005f5cc0
- [ ] `MaterialModifierAtom::GetSubDataIDs` · 006424b0 · legions 005f5d00

### OpacityAtom
- [ ] `OpacityAtom::Clear` · 006424e0 · legions 005f5d30
- [ ] `OpacityAtom::Replaces` · 006424f0 · legions 005f5d40
- [ ] `OpacityAtom::operator==` · 00642510 · legions 005f5d60

### MeshReplacementAtom
- [ ] `MeshReplacementAtom::Clear` · 00642540 · legions 005f5d90
- [ ] `MeshReplacementAtom::Replaces` · 00642550 · legions 005f5da0
- [ ] `MeshReplacementAtom::operator==` · 00642580 · legions 005f5dd0

### PGDReplacementAtom
- [ ] `PGDReplacementAtom::Clear` · 006425b0 · legions 005f5e00
- [ ] `PGDReplacementAtom::Replaces` · 006425c0 · legions 005f5e10
- [ ] `PGDReplacementAtom::operator==` · 006425f0 · legions 005f5e40
- [ ] `PGDReplacementAtom::GetSubDataIDs` · 00642620 · legions 005f5e70

### IconModAtom
- [ ] `IconModAtom::Clear` · 00642630 · legions 005f5e80
- [ ] `IconModAtom::Replaces` · 00642640 · legions 005f5e90
- [ ] `IconModAtom::GetSubDataIDs` · 006426b0 · legions 005f5f00
- [-] `IconModAtom::ToFileNode` · 006426c0 — text source format (admin tools)

### MeshMaterials::MaterialAndDataID
- [ ] `MeshMaterials::MaterialAndDataID::StreamPack` · 006426e0

### MaterialModifierAtom
- [ ] `MaterialModifierAtom::StreamPack` · 00642710

### OpacityAtom
- [ ] `OpacityAtom::StreamPack` · 00642790

### MeshReplacementAtom
- [ ] `MeshReplacementAtom::StreamPack` · 006427d0

### PGDReplacementAtom
- [ ] `PGDReplacementAtom::StreamPack` · 00642830 · legions 005f6030

### FXAtom
- [ ] `FXAtom::Replaces` · 00642870 · legions 005f6070
- [ ] `FXAtom::operator<` · 00642890 · legions 005f6090
- [ ] `FXAtom::GetSubDataIDs` · 006428c0 · legions 005f60c0

### IconModAtom
- [ ] `IconModAtom::operator==` · 00642910 · legions 005f6160

### CVertexSetData
- [ ] `CVertexSetData::StreamPack` · 00642950

### FootPrint
- [ ] `FootPrint::Reset` · 00642980 · legions 005f61a0

### AppearanceAtom
- [ ] `AppearanceAtom::ReadPriority` · 00642990
- [ ] `AppearanceAtom::WritePriority` · 00642a20

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
- [ ] `SmartArray<FXData>::AddToEnd` · 00643f50

### FXAtom
- [-] `FXAtom::FromFileNode` · 00643fa0 — text source format (admin tools)

## `portal/shared/src/objdesc/appearance/partgroupdatadesc.cpp.c` (0/32)

### UIRegion
- [ ] `UIRegion::SetZLevel_Internal` · 00644090 · legions 005f61b0

### PartGroupDataDesc
- [ ] `PartGroupDataDesc::operator==` · 006440a0 · legions 005f61c0

### CMotionInterp
- [ ] `CMotionInterp::GetLocalDesiredVelocity` · 006440f0 · legions 005f6210

### PartGroupDataDesc
- [ ] `PartGroupDataDesc::RemoveChild` · 00644100 · legions 005f6220
- [ ] `PartGroupDataDesc::IsConnPtEmpty` · 00644160 · legions 005f6280
- [ ] `PartGroupDataDesc::FindPGDDWithPartGroupKey` · 006441a0 · legions 005f62c0

### SmartArray<PartGroupDataDesc_*>
- [ ] `SmartArray<PartGroupDataDesc_*>::RemoveNullsOrdered` · 00644210 · legions 005f6330

### PartGroupDataDesc
- [ ] `PartGroupDataDesc::PrepareForRemoval` · 00644260 · legions 005f6380
- [ ] `PartGroupDataDesc::Empty` · 006442b0 · legions 005f63d0
- [ ] `PartGroupDataDesc::AddChildPointer` · 006442f0 · legions 005f6410
- [ ] `PartGroupDataDesc::HasFX` · 00644330 · legions 005f6450
- [ ] `PartGroupDataDesc::GetPackBitfield` · 00644350 · legions 005f6470
- [~] `PartGroupDataDesc::StreamPack` · 006443c0 · legions 005f64e0 — unpack only, in ac2-net protocol::fx / protocol::visual (Legions FXNode::StreamPack 0x00642410 instruction-identical)
- [ ] `PartGroupDataDesc::AugmentAtomCollection` · 00644510 · legions 005f6690
- [ ] `PartGroupDataDesc::WriteAppearanceKeys` · 00644590
- [ ] `PartGroupDataDesc::GetSubDataIDs` · 00644950 · legions 005f6710
- [ ] `PartGroupDataDesc::Clear` · 00644a30 · legions 005f67c0
- [ ] `PartGroupDataDesc::HasAppliedAppKey` · 00644a80 · legions 005f6810
- [ ] `PartGroupDataDesc::RemoveAppearanceKeys` · 00644ad0 · legions 005f6860
- [ ] `PartGroupDataDesc::RemoveAppearanceKey` · 00644bb0 · legions 005f6940
- [ ] `PartGroupDataDesc::ReadFX` · 00644c40
- [-] `PartGroupDataDesc::ToFileNode` · 00644fe0 — text source format (admin tools)
- [ ] `PartGroupDataDesc::AugmentAppearanceInfo` · 00645580 · legions 005f6b20
- [ ] `PartGroupDataDesc::ReadAppearanceKeys` · 006455e0
- [ ] `PartGroupDataDesc::~PartGroupDataDesc` · 00645940 · legions 005f6bf0
- [-] `PartGroupDataDesc::FromFileNode` · 00645a20 — text source format (admin tools)

### PHashTable<unsigned_long,PSmartArray<FXData>_>
- [~] `PHashTable<unsigned_long,PSmartArray<FXData>_>::StreamPack` · 00645f00 — unpack only, in ac2-net protocol::fx / protocol::visual (Legions FXNode::StreamPack 0x00642410 instruction-identical)

### PartGroupDataDesc
- [ ] `PartGroupDataDesc::PartGroupDataDesc` · 006460f0 · legions 005f6ec0
- [ ] `PartGroupDataDesc::operator=` · 006462a0 · legions 005f7070
- [ ] `PartGroupDataDesc::ApplyAdditivePGDDesc` · 00646300 · legions 005f70d0
- [ ] `PartGroupDataDesc::PartGroupDataDesc` · 006464f0 · legions 005f72c0
- [ ] `PartGroupDataDesc::FillFXTable` · 006465d0 · legions 005f73a0

## `portal/shared/src/objdesc/appearance/visualmodtable.cpp.c` (0/8)

### VisualModTable
- [ ] `VisualModTable::StreamPack` · 0064ac30
- [ ] `VisualModTable::UpdateAtomCollection` · 0064ac60 · legions 005fa2c0
- [ ] `VisualModTable::GetSubDataIDs` · 0064aca0 · legions 005fa300
- [-] `VisualModTable::ToFileNode` · 0064ace0 — text source format (admin tools)
- [ ] `VisualModTable::GetAtomCollection` · 0064adc0
- [-] `VisualModTable::FromFileNode` · 0064aee0 — text source format (admin tools)
- [ ] `VisualModTable::VisualModTable` · 0064b010 · legions 005fa340
- [ ] `VisualModTable::~VisualModTable` · 0064b070 · legions 005fa3a0

## `portal/shared/src/objdesc/icon/icondesc.cpp.c` (0/18)

### RGBAColor
- [ ] `RGBAColor::operator==` · 0063faf0 · legions 005f3a20

### IconLayerDesc
- [ ] `IconLayerDesc::HasColor` · 0063fb60

### IconDesc
- [ ] `IconDesc::StreamPack` · 0063fb80
- [ ] `IconDesc::operator==` · 0063fc00 · legions 005f3b10
- [ ] `IconDesc::IsValid` · 0063fc80 · legions 005f3b90
- [ ] `IconDesc::GetSubDataIDs` · 0063fcd0 · legions 005f3be0
- [-] `IconDesc::ToFileNode` · 0063fea0 — text source format (admin tools)
- [ ] `IconDesc::Clear` · 006400e0 · legions 005f3e10

### PList<IconLayerDesc>
- [ ] `PList<IconLayerDesc>::Pack` · 00640100 · legions 005f3e30
- [ ] `PList<IconLayerDesc>::GetPackSize` · 00640150
- [ ] `PList<IconLayerDesc>::UnPack` · 00640190 · legions 005f3ed4

### IconDesc
- [ ] `IconDesc::IconDesc` · 006402c0 · legions 005f3ff0
- [ ] `IconDesc::~IconDesc` · 006402f0 · legions 005f4020
- [ ] `IconDesc::operator=` · 00640320 · legions 005f4050
- [ ] `IconDesc::ApplyAdditiveIconLayer` · 00640360 · legions 005f4090
- [ ] `IconDesc::IconDesc` · 00640440 · legions 005f4170
- [-] `IconDesc::FromFileNode` · 00640480 — text source format (admin tools)
- [ ] `IconDesc::ApplyAdditiveIconDesc` · 00640580 · legions 005f41b0

## `portal/shared/src/objdesc/icon/iconlayerdesc.cpp.c` (0/7)

### IconLayerDesc
- [ ] `IconLayerDesc::Clear` · 006405c0 · legions 005f41f0
- [ ] `IconLayerDesc::ApplyAdditive` · 006405f0 · legions 005f4220

### (free functions)
- [ ] `STREAMPACK_STREAMPACKOBJ` · 00640640 · legions 005f4270

### IconLayerDesc
- [ ] `IconLayerDesc::IconLayerDesc` · 00640660 · legions 005f4290
- [ ] `IconLayerDesc::StreamPack` · 006406a0 · legions 005f42d0
- [-] `IconLayerDesc::ToFileNode` · 00640700 — text source format (admin tools)
- [-] `IconLayerDesc::FromFileNode` · 00640890 — text source format (admin tools)

## `portal/shared/src/objdesc/physicsdesc.cpp.c` (0/14)

### PhysicsDesc
- [ ] `PhysicsDesc::get_timestamp` · 00640b30 · legions 005f4330
- [ ] `PhysicsDesc::get_parent_id` · 00640b40 · legions 005f4340
- [ ] `PhysicsDesc::get_parent_instance_stamp` · 00640b60 · legions 005f4360
- [ ] `PhysicsDesc::get_location_id` · 00640b70 · legions 005f4370
- [ ] `PhysicsDesc::get_mode` · 00640b80 · legions 005f4380
- [ ] `PhysicsDesc::get_behaviors` · 00640b90 · legions 005f4390
- [ ] `PhysicsDesc::get_sliders` · 00640ba0 · legions 005f43a0
- [ ] `PhysicsDesc::get_fx` · 00640bb0 · legions 005f43b0

### (free functions)
- [ ] `STREAMPACK_STL` · 00640f40 · legions 005f4740
- [ ] `STREAMPACK_STL` · 006410e0 · legions 005f48e0

### PhysicsDesc
- [ ] `PhysicsDesc::Destroy` · 006412a0 · legions 005f4aa0
- [ ] `PhysicsDesc::~PhysicsDesc` · 00641450 · legions 005f4c50
- [ ] `PhysicsDesc::StreamPack` · 00641530
- [ ] `PhysicsDesc::PhysicsDesc` · 00641c70 · legions 005f5470

## `portal/shared/src/objdesc/visualdesc.cpp.c` (0/58)

### VisualDesc
- [ ] `VisualDesc::FillFXTable` · 0063c910 · legions 005f1440
- [ ] `VisualDesc::GetStartupFX` · 0063c920 · legions 005f1450
- [ ] `VisualDesc::HasFX` · 0063c930 · legions 005f1460

### (free functions)
- [ ] `STREAMPACK_STREAMPACKOBJ` · 0063c940 · legions 0063e9a0
- [ ] `STREAMPACK` · 0063c960 · legions 0050bc60

### VisualDesc
- [ ] `VisualDesc::Empty` · 0063caf0 · legions 005f1540

### IntrusiveHashIterator<unsigned_long,HashTableData<unsigned_long,PartGroupDataDesc>_*>
- [ ] `IntrusiveHashIterator<unsigned_long,HashTableData<unsigned_long,PartGroupDataDesc>_*>::operator++` · 0063cb50 · legions 006003c0

### VisualDesc
- [ ] `VisualDesc::GetPGDDescTableIter` · 0063cc30
- [ ] `VisualDesc::GetPackBitfield` · 0063cc60 · legions 005f1600

### HashIterator<unsigned_long,PartGroupDataDesc>
- [ ] `HashIterator<unsigned_long,PartGroupDataDesc>::operator++` · 0063cd60 · legions 00600810

### VisualDesc
- [-] `VisualDesc::ToFileNode` · 0063ce40 — text source format (admin tools)
- [ ] `VisualDesc::GetSubDataIDs` · 0063d1c0
- [ ] `VisualDesc::GetPGDDesc` · 0063d2f0 · legions 005f18e0
- [ ] `VisualDesc::ResetRootPointer` · 0063d310 · legions 005f1900
- [ ] `VisualDesc::RemovePGDD` · 0063d3e0 · legions 005f19d0
- [ ] `VisualDesc::ModVDesc_DeleteStragglers` · 0063d450 · legions 005f1a40
- [ ] `VisualDesc::BuildAtomCollection` · 0063d680 · legions 005f1c70
- [ ] `VisualDesc::GatherAppearanceKeys` · 0063d740 · legions 005f1d10
- [ ] `VisualDesc::StreamPack` · 0063d7f0 — part-group FX overrides (FXData/FXNode) refused as unsupported
- [ ] `VisualDesc::AddPGDDesc` · 0063db70 · legions 005f2140
- [ ] `VisualDesc::HasAppliedAppKey` · 0063dcc0 · legions 005f2290
- [ ] `VisualDesc::InqAppInfoHashForPartGroup` · 0063dd00 · legions 005f22d0
- [ ] `VisualDesc::RemovePGDDRecursively` · 0063dd40 · legions 005f2310
- [ ] `VisualDesc::RecursiveAdditivePGDDMerge` · 0063ddb0 · legions 005f2380
- [ ] `VisualDesc::ApplyAdditiveVDesc` · 0063dea0 · legions 005f2470
- [ ] `VisualDesc::Clear` · 0063e0d0 · legions 005f26a0
- [ ] `VisualDesc::AddPGDDesc` · 0063e170 · legions 005f2740
- [ ] `VisualDesc::MakeLocalPGDDFromParent` · 0063e1d0 · legions 005f27ba
- [ ] `VisualDesc::AddAppearanceKey` · 0063e370 · legions 005f2920
- [ ] `VisualDesc::AddAppInfoHashToPartGroup` · 0063e3e0 · legions 005f2990
- [ ] `VisualDesc::ApplyAdditiveVDesc` · 0063e460 · legions 005f2a10
- [-] `VisualDesc::FromFileNode` · 0063e4c0 — text source format (admin tools)
- [ ] `VisualDesc::Destroy` · 0063eaf0

### PHashTable<unsigned_long,PartGroupDataDesc>
- [ ] `PHashTable<unsigned_long,PartGroupDataDesc>::StreamPack` · 0063eb00

### VisualDesc
- [ ] `VisualDesc::VisualDesc` · 0063ecd0 · legions 005f2c50
- [ ] `VisualDesc::GetDBOType` · 0063ed70
- [ ] `VisualDesc::~VisualDesc` · 0063ed80 · legions 005f2d00
- [ ] `VisualDesc::operator=` · 0063ee10 · legions 005f2d90
- [ ] `VisualDesc::GetTotalVisualDescInternal` · 0063ee40 · legions 005f2dc0
- [ ] `VisualDesc::AddAppearanceKey` · 0063eee0 · legions 005f2e30
- [ ] `VisualDesc::RemoveAppearanceKey` · 0063ef40 · legions 005f2e90
- [ ] `VisualDesc::AttemptToRemoveVDesc` · 0063f070 · legions 005f2fa0
- [ ] `VisualDesc::ReplaceVisualDesc` · 0063f250 · legions 005f3180
- [ ] `VisualDesc::CopyInto` · 0063f340 · legions 005f3270
- [ ] `VisualDesc::AddAppearanceMod` · 0063f380 · legions 005f32b0
- [ ] `VisualDesc::RemoveAppearanceMod` · 0063f390 · legions 005f32c0
- [ ] `VisualDesc::GetVisualDescFromDB` · 0063f3a0 · legions 005f32d0
- [ ] `VisualDesc::Allocate` · 0063f410
- [ ] `VisualDesc::VisualDesc` · 0063f440 · legions 005f3370
- [ ] `VisualDesc::ApplySubtractiveVDesc` · 0063f4d0 · legions 005f3400

### FXData
- [ ] `FXData::FXData` · 0063f520 · legions 005f3450

### SmartArray<FXData>
- [ ] `SmartArray<FXData>::operator=` · 0063f650 · legions 005f3580

### PSmartArray<FXData>
- [~] `PSmartArray<FXData>::StreamPack` · 0063f6c0 · legions 005f35f0 — unpack only, in ac2-net protocol::fx / protocol::visual (Legions FXNode::StreamPack 0x00642410 instruction-identical)

### HashTable<unsigned_long,PSmartArray<FXData>_>
- [ ] `HashTable<unsigned_long,PSmartArray<FXData>_>::operator=` · 0063f770 · legions 005f36a0

### VisualDesc
- [ ] `VisualDesc::SwapPGDD` · 0063f880 · legions 005f37b0
- [ ] `VisualDesc::ModifyPGDTreeBasedOnApr` · 0063f930 · legions 005f3860
- [ ] `VisualDesc::GetTotalVisualDesc` · 0063f9c0 · legions 005f38f0
- [ ] `VisualDesc::GetTotalIconDesc` · 0063f9f0 · legions 005f3920

## `portal/shared/src/objdesc/weeniedesc.cpp.c` (0/2)

### WeenieDesc
- [ ] `WeenieDesc::IsInWeenieDesc` · 00641ea0 · legions 005f56a0
- [ ] `WeenieDesc::IsInWeenieDesc` · 00641fa0 · legions 005f57a0
