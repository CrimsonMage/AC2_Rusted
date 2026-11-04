# ac2-render: Scene and world rendering

781 retail functions: 300 done, 0 partial, 0 to do, 481 skipped.
Legend and workflow: [README](README.md).

## `portal/client/src/ctrilist/ctrianglelist.cpp.c` (1/12)

### CMeshFragment
- [-] `CMeshFragment::CMeshFragment` · 0052bf30 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [-] `CMeshFragment::InitIndexed` · 0052bf60 · legions 00510b50 — replaced by design: in-memory builder for code-made meshes (sphere/cylinder shadow caps, frill mesh, terrain ApplyList); Rust builds its own vertex/index vectors [AC2_Rusted/crates/ac2-render/src/terrain/mesh.rs:1]
- [-] `CMeshFragment::Init` · 0052bfd0 · legions 00510bc0 — replaced by design: in-memory builder for code-made meshes (sphere/cylinder shadow caps, frill mesh, terrain ApplyList); Rust builds its own vertex/index vectors [AC2_Rusted/crates/ac2-render/src/terrain/mesh.rs:1]

### CMeshTriList
- [-] `CMeshTriList::CMeshTriList` · 0052c000 · legions 00510bf0 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [-] `CMeshTriList::~CMeshTriList` · 0052c030 · legions 00510c20 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [-] `CMeshTriList::GetPackSize` · 0052c120 · legions 00510d10 — save/write path; the client reads DATs only
- [-] `CMeshTriList::Pack` · 0052c190 · legions 00510d80 — save/write path; the client reads DATs only
- [x] `CMeshTriList::UnPack` · 0052c260 · legions 00510e50 — audit: covered: DAT degrade lists (detail-level mask ac2_mesh.rs:30) are read by ac2-assets [AC2_Rusted/crates/ac2-assets/src/ac2_mesh.rs:2]
- [-] `CMeshTriList::Init` · 0052c390 · legions 00510f80 — replaced by design: in-memory trilist builder for code-made meshes (sphere/cylinder, terrain, frills); Rust builds vectors directly [AC2_Rusted/crates/ac2-render/src/terrain/mesh.rs:1]

### DegradeType
- [-] `DegradeType::ParseString` · 0052c460 — not needed: text mesh source loader (admin tools); DAT meshes carry the numeric type [decompiled-pdb: only caller CMeshTriList::FromFileNode]

### CMeshFragment
- [-] `CMeshFragment::FromFileNode` · 0052c600 — text source format (admin tools)

### CMeshTriList
- [-] `CMeshTriList::FromFileNode` · 0052c830 — text source format (admin tools)

## `portal/client/src/render/core/font/font.cpp.c` (8/13)

### Font
- [-] `Font::Begin` · 005b88d0 · legions 00583530 — D3D device/resource plumbing; wgpu replaces
- [-] `Font::End` · 005b8900 · legions 00583560 — D3D device/resource plumbing; wgpu replaces
- [-] `Font::Destroy` · 005b8940 · legions 005835a0 — retail DB object cache handle/release; Arc per DID replaces (AGENTS.md exception)
- [x] `Font::GetCharDesc` · 005b89a0 · legions 00583600 — ac2-assets font.rs: all 55 portal fonts decode
- [x] `Font::GetCharWidthA` · 005b8a30 · legions 00583690 — ac2-ui text.rs / text_attr.rs: UIElement_Text layout and drawing
- [x] `Font::ContainsString` · 005b8a50 · legions 005836b0 — ac2-assets font.rs Ac2Font::contains per character (a glyph of its own, not the ? stand-in char_desc gives) and ac2-ui text.rs font_contains over the string, newline skipped
- [x] `Font::GenerateCharacterMap` · 005b8ad0 · legions 00583730 — ac2-assets font.rs: all 55 portal fonts decode
- [x] `Font::GetStringWidthInternal` · 005b8bc0 · legions 00583820 — ac2-assets font.rs: all 55 portal fonts decode
- [x] `Font::InitLoad` · 005b8c20 — audit: covered: GenerateCharacterMap equivalent (character_map / lookup) is built at load [AC2_Rusted/crates/ac2-assets/src/font.rs:8]
- [-] `Font::LoadFromFile` · 005b8c40 — text source loader, admin build only
- [-] `Font::SaveToFile` · 005b8d40 — admin-only/tool/debug code, not in the client (font save tool)
- [x] `Font::StreamPack` · 005b8e10 · legions 00583890 — ac2-assets font.rs: all 55 portal fonts decode
- [x] `Font::GetStringWidth` · 005b8f70 · legions 005839f0 — audit: covered: string_width = GetStringWidthInternal; the wrapper only adds null/-1 length handling [AC2_Rusted/crates/ac2-assets/src/font.rs:89]

## `portal/client/src/render/core/font/fontmapper.cpp.c` (2/4)

### FontLocal
- [x] `FontLocal::GetGlobalOrLocalFont` · 005c4800 · legions 0058b8c0 — cited in ac2-ui text.rs:29 (UiTextData::font, font by DID with fallback)

### FontMapper
- [-] `FontMapper::Startup` · 005c4850 · legions 0058b910 — retail DB object cache handle/release; Arc per DID replaces (AGENTS.md exception)
- [-] `FontMapper::Shutdown` · 005c4890 · legions 0058b950 — retail DB object cache handle/release; Arc per DID replaces (AGENTS.md exception)
- [x] `FontMapper::GetFontByDataID` · 005c48f0 · legions 0058b9b0 — covered: ac2-assets font.rs load_ac2_font (portal Font below 0x42001000, FontLocal above; one Arc per DID stands in for the mapper list, AGENTS.md cache rule); doc names it

## `portal/client/src/render/core/graphicsresource.cpp.c` (0/10)

### GraphicsResource
- [-] `GraphicsResource::CopyInto` · 005bc290 · legions 005859e0 — D3D device/resource plumbing; wgpu replaces
- [-] `GraphicsResource::RestoreResource` · 005bc2c0 · legions 00585a10 — D3D device/resource plumbing; wgpu replaces
- [-] `GraphicsResource::ShutdownResourceManager` · 005bc330 · legions 00585a80 — D3D device/resource plumbing; wgpu replaces
- [-] `GraphicsResource::UnlinkResource` · 005bc360 · legions 00585ab0 — D3D device/resource plumbing; wgpu replaces
- [-] `GraphicsResource::PurgeResources` · 005bc3b0 · legions 00585b20 — D3D device/resource plumbing; wgpu replaces
- [-] `GraphicsResource::PurgeOldResources` · 005bc400 · legions 00585b70 — D3D device/resource plumbing; wgpu replaces
- [-] `GraphicsResource::~GraphicsResource` · 005bc530 · legions 00585ca0 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [-] `GraphicsResource::LinkResource` · 005bc620 · legions 00585d90 — D3D device/resource plumbing; wgpu replaces
- [-] `GraphicsResource::GraphicsResource` · 005bc690 · legions 00585df0 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [-] `GraphicsResource::DiscardResourceBytes` · 005bca00 · legions 00586140 — D3D device/resource plumbing; wgpu replaces

## `portal/client/src/render/core/indexbuffer/d3d/renderindexbufferd3d.cpp.c` (0/10)

### RenderIndexBufferD3D
- [-] `RenderIndexBufferD3D::Begin` · 005db9a0 · legions 0059c7b0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderIndexBufferD3D::RefreshD3DIndexBuffer` · 005db9b0 · legions 0059c7c0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderIndexBufferD3D::GetD3DIndexBuffer` · 005dba90 · legions 0059c8a0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderIndexBufferD3D::CreateD3DIndexBuffer` · 005dbad0 · legions 0059c8e0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderIndexBufferD3D::PurgeResource` · 005dbbc0 · legions 0059c9d0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderIndexBufferD3D::RestoreResource` · 005dbc00 · legions 0059ca10 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderIndexBufferD3D::End` · 005dbc20 · legions 0059ca30 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderIndexBufferD3D::Startup` · 005dbc50 · legions 0059ca60 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderIndexBufferD3D::Shutdown` · 005dbc90 · legions 0059caa0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderIndexBufferD3D::Duplicate` · 005dbcd0 · legions 0059cae0 — D3D device/resource plumbing; wgpu replaces

## `portal/client/src/render/core/indexbuffer/renderindexbuffer.cpp.c` (0/11)

### RenderIndexBuffer
- [-] `RenderIndexBuffer::Begin` · 005bdcd0 · legions 00586e70 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderIndexBuffer::End` · 005bdd00 · legions 00586ea0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderIndexBuffer::Startup` · 005bdd20 · legions 00586ec0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderIndexBuffer::Shutdown` · 005bdd90 · legions 00586f30 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderIndexBuffer::Lock` · 005bddd0 · legions 00586f70 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderIndexBuffer::Unlock` · 005bdde0 · legions 00586f80 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderIndexBuffer::RecalculateMinMaxIndices` · 005bde00 · legions 00586fa0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderIndexBuffer::Stream` · 005bdea0 · legions 00587040 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderIndexBuffer::SetRenderIndexBuffer` · 005bdfe0 · legions 00587180 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderIndexBuffer::Duplicate` · 005be070 · legions 00587210 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderIndexBuffer::FromFileNode` · 005be0f0 — text source format (admin tools)

## `portal/client/src/render/core/null/renderdevicenull.cpp.c` (0/8)

### RenderDeviceNull
- [-] `RenderDeviceNull::Begin` · 005dcfb0 · legions 0059dca0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderDeviceNull::End` · 005dcfc0 · legions 0059dcb0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderDeviceNull::Startup` · 005dcfd0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderDeviceNull::Shutdown` · 005dd020 · legions 0059dd10 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderDeviceNull::CreateSurface` · 005dd040 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderDeviceNull::CreateIndexBuffer` · 005dd060 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderDeviceNull::CreateVertexBuffer` · 005dd090 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderDeviceNull::CreateTexture` · 005dd0e0 · legions 00586960 — D3D device/resource plumbing; wgpu replaces

## `portal/client/src/render/core/null/rendernull.cpp.c` (0/11)

### RenderNull
- [-] `RenderNull::Startup` · 005d9410 · legions 0059aaa0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderNull::Shutdown` · 005d9430 · legions 0059aac0 — D3D device/resource plumbing; wgpu replaces

### RenderDevice
- [-] `RenderDevice::RenderDevice` · 005d9440 · legions 0059aad0 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [-] `RenderDevice::SetupState` · 005d9590 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderDevice::SetMaterialDiffuseColor` · 005d95a0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderDevice::IsResetPossible` · 005d95d0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderDevice::~RenderDevice` · 005d95e0 · legions 0059ac80 — plain data ctor/dtor/operator=; Rust struct or Drop replaces

### RenderDeviceNull
- [-] `RenderDeviceNull::RenderDeviceNull` · 005d9640 · legions 0059ace0 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [-] `RenderDeviceNull::BeginScene` · 005d9710 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderDeviceNull::EndScene` · 005d9720 — D3D device/resource plumbing; wgpu replaces

### RenderNull
- [-] `RenderNull::CreateRenderDevice` · 005d9760 · legions 0059ae00 — D3D device/resource plumbing; wgpu replaces

## `portal/client/src/render/core/pixelformat.cpp.c` (0/5)

### PixelFormatDesc
- [-] `PixelFormatDesc::CalcBitOffsets` · 005ac250 · legions 00578550 — D3D device/resource plumbing; wgpu replaces
- [-] `PixelFormatDesc::ConvertColor` · 005ac390 — D3D device/resource plumbing; wgpu replaces
- [-] `PixelFormatDesc::SetFormat` · 005ac470 · legions 00578690 — D3D device/resource plumbing; wgpu replaces
- [-] `PixelFormatDesc::DetermineFormat` · 005ac870 — D3D device/resource plumbing; wgpu replaces
- [-] `PixelFormatDesc::PixelFormatDesc` · 005acba0 · legions 00578a90 — D3D device/resource plumbing; wgpu replaces

## `portal/client/src/render/core/render.cpp.c` (6/27)

### Render
- [x] `Render::HandleChange` · 005b30c0 · legions 0057da60 — audit: covered: bank values are re-read into SceneOptions / sound / camera / fx on every apply [AC2_Rusted/crates/ac2-client/src/client_prefs.rs:234]

### GraphicsPreferences
- [-] `GraphicsPreferences::GraphicsPreferences` · 005b30e0 · legions 0057da80 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [-] `GraphicsPreferences::~GraphicsPreferences` · 005b31f0 · legions 0057db90 — plain data ctor/dtor/operator=; Rust struct or Drop replaces

### Render
- [-] `Render::StartupDevice` · 005b3300 · legions 0057dca0 — D3D device lifecycle/restore; wgpu replaces
- [-] `Render::ShutdownDevice` · 005b3380 · legions 0057dd20 — D3D device lifecycle/restore; wgpu replaces
- [x] `Render::SetOverallGraphicsQuality` · 005b33b0 · legions 0057dd50 — audit: covered: apply_quality + ClientPrefs::set_overall_graphics_quality (client_prefs.rs:173); live options read through scene_options (catalog [~] text is stale) [AC2_Rusted/crates/ac2-render/src/prefs.rs:403]
- [-] `Render::ShutdownRenderingSystem` · 005b3c40 · legions 0057e5e0 — D3D device lifecycle/restore; wgpu replaces
- [-] `Render::RestartDevice` · 005b3ca0 · legions 0057e640 — D3D device lifecycle/restore; wgpu replaces
- [-] `Render::CreateIndexBuffer` · 005b3d90 · legions 0057e730 — D3D device/resource plumbing; wgpu replaces
- [-] `Render::Startup` · 005b3de0 · legions 0057e780 — D3D device lifecycle/restore; wgpu replaces
- [-] `Render::Shutdown` · 005b3fb0 · legions 0057e950 — D3D device lifecycle/restore; wgpu replaces
- [x] `Render::InitRenderingPreferences` · 005b4160 · legions 0057eb00 — audit: covered: register_graphics + apply_startup_quality (client_prefs.rs:162); first-run GPU level is AGENTS.md:560 [AC2_Rusted/crates/ac2-render/src/prefs.rs:238]
- [-] `Render::StartupRenderingDevice` · 005b5200 · legions 0057fba0 — D3D device lifecycle/restore; wgpu replaces
- [-] `Render::RestartRenderingSystem` · 005b52b0 · legions 0057fc50 — D3D device lifecycle/restore; wgpu replaces
- [-] `Render::FlushGraphicsResources` · 005b5310 · legions 0057fcb0 — D3D device lifecycle/restore; wgpu replaces
- [-] `Render::RestartRenderingSystem` · 005b5360 · legions 0057fd00 — D3D device lifecycle/restore; wgpu replaces
- [-] `Render::UnlinkRGRCallback` · 005b53b0 · legions 0057fd50 — D3D device lifecycle/restore; wgpu replaces
- [-] `Render::CreateVertexBuffer` · 005b5400 · legions 0057fda0 — D3D device/resource plumbing; wgpu replaces
- [-] `Render::~Render` · 005b5460 · legions 0057fe00 — plain data ctor/dtor/operator=; Rust struct or Drop replaces

### RenderNull
- [-] `RenderNull::RenderNull` · 005b54f0 · legions 0057fe90 — plain data ctor/dtor/operator=; Rust struct or Drop replaces

### RenderD3D
- [-] `RenderD3D::RenderD3D` · 005b5570 · legions 0057ff10 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [-] `RenderD3D::~RenderD3D` · 005b5610 · legions 0057ffb0 — plain data ctor/dtor/operator=; Rust struct or Drop replaces

### Render
- [-] `Render::StartupRenderingSystem` · 005b5670 · legions 00580010 — D3D device lifecycle/restore; wgpu replaces
- [x] `Render::SaveOverallGraphicsQuality` · 005b5710 · legions 005800b0 — ac2-render prefs::save_overall_graphics_quality over ac2-prefs PrefsFile::save_uint_now; ac2-client ClientPrefs wraps the file
- [x] `Render::UpdateOverallGraphicsQuality` · 005b5860 — ac2-render prefs.rs matching_quality / update_overall_graphics_quality (Legions FUN_00580200, the tail of Render::HandleApply); ac2-ui options_apply puts the level in the file before the page's save, or writes it at once during a render verification; tests each_quality_preset_matches_its_own_level..., an_options_apply_saves_the_preset_level_the_graphics_now_match
- [-] `Render::LinkRGRCallback` · 005b5e70 · legions 00580810 — D3D device lifecycle/restore; wgpu replaces
- [x] `Render::HandleApply` · 005b5ee0 · legions 00580880 — audit: covered: apply_live_preferences re-derives scene options; presentation/AA changes via ac2-client window_presentation.rs; gamma = AGENTS.md:138 [AC2_Rusted/crates/ac2-client/src/client_prefs.rs:234]

## `portal/client/src/render/core/renderdevice.cpp.c` (1/17)

### RDTemporaryState
- [-] `RDTemporaryState::Invalidate` · 005ae490 · legions 00579f30 — D3D device/resource plumbing; wgpu replaces

### RDPersistentState
- [-] `RDPersistentState::Invalidate` · 005ae670 · legions 0057a110 — D3D device/resource plumbing; wgpu replaces

### RenderDevice
- [-] `RenderDevice::Begin` · 005ae710 · legions 0057a1b0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderDevice::End` · 005ae750 · legions 0057a1f0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderDevice::Startup` · 005ae7b0 · legions 0057a250 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderDevice::Shutdown` · 005ae860 · legions 0057a300 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderDevice::GetDisplayWidth` · 005ae8a0 · legions 0057a340 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderDevice::GetDisplayHeight` · 005ae8b0 · legions 0057a350 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderDevice::CreateLocalSurface` · 005ae8c0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderDevice::SetViewport` · 005ae8e0 · legions 0057a380 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderDevice::GetUISurfaceFormat` · 005ae950 · legions 005cbab0 — D3D device/resource plumbing; wgpu replaces
- [x] `RenderDevice::SetBlendMatrices` · 005ae960 — ac2-gpu ObjectInstance.pose (part matrices, bone storage buffer)
- [-] `RenderDevice::SetRenderTargetTexture` · 005ae980 · legions 0057a410 — D3D device/resource plumbing; wgpu replaces

### VertexShader
- [-] `VertexShader::VertexShader` · 005ae9d0 · legions 0057a450 — D3D device/resource plumbing; wgpu replaces

### PixelShader
- [-] `PixelShader::PixelShader` · 005aea20 · legions 0057a4a0 — D3D device/resource plumbing; wgpu replaces

### RDVolatileState
- [-] `RDVolatileState::Invalidate` · 005aea70 · legions 0057a4f0 — D3D device/resource plumbing; wgpu replaces

### RenderDeviceState
- [-] `RenderDeviceState::Invalidate` · 005aeab0 · legions 0057a530 — D3D device/resource plumbing; wgpu replaces

## `portal/client/src/render/core/surface/d3d/rendersurfaced3d.cpp.c` (0/26)

### NormalMap
- [-] `NormalMap::Kill` · 005ca740 · legions 00591730 — retail DB object cache handle/release; Arc per DID replaces (AGENTS.md exception)

### RenderSurfaceD3D
- [-] `RenderSurfaceD3D::CreateFromD3DSurface` · 005ca770 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderSurfaceD3D::CreateDepthStencil` · 005ca800 · legions 005917f0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderSurfaceD3D::GetDirect3DSurface` · 005ca840 · legions 00591830 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderSurfaceD3D::CreateD3DSurface` · 005ca880 · legions 00591870 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderSurfaceD3D::PurgeResource` · 005ca8f0 · legions 005918e0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderSurfaceD3D::RestoreResource` · 005ca940 · legions 00591930 — D3D device/resource plumbing; wgpu replaces

### NormalMap::GrayScale
- [-] `NormalMap::GrayScale::Calculate` · 005ca980 — not needed: height->normal conversion only runs for loose source images whose file name contains '_bump' (bWantNormalMap); DAT textures never take it; water bump is procedural (AGENTS.md:174) [decompiled-pdb rendersurface.cpp.c:352,777]

### NormalMap::Average
- [-] `NormalMap::Average::Calculate` · 005ca9d0 — not needed: height->normal conversion only runs for loose source images whose file name contains '_bump' (bWantNormalMap); DAT textures never take it; water bump is procedural (AGENTS.md:174) [decompiled-pdb rendersurface.cpp.c:352,777]

### NormalMap::Red
- [-] `NormalMap::Red::Calculate` · 005caa30 — not needed: height->normal conversion only runs for loose source images whose file name contains '_bump' (bWantNormalMap); DAT textures never take it; water bump is procedural (AGENTS.md:174) [decompiled-pdb rendersurface.cpp.c:352,777]

### NormalMap::Alpha
- [-] `NormalMap::Alpha::Calculate` · 005caa50 — not needed: height->normal conversion only runs for loose source images whose file name contains '_bump' (bWantNormalMap); DAT textures never take it; water bump is procedural (AGENTS.md:174) [decompiled-pdb rendersurface.cpp.c:352,777]

### NormalMap::Green
- [-] `NormalMap::Green::Calculate` · 005caa70 — not needed: height->normal conversion only runs for loose source images whose file name contains '_bump' (bWantNormalMap); DAT textures never take it; water bump is procedural (AGENTS.md:174) [decompiled-pdb rendersurface.cpp.c:352,777]

### NormalMap::Blue
- [-] `NormalMap::Blue::Calculate` · 005caa90 — not needed: height->normal conversion only runs for loose source images whose file name contains '_bump' (bWantNormalMap); DAT textures never take it; water bump is procedural (AGENTS.md:174) [decompiled-pdb rendersurface.cpp.c:352,777]

### NormalMap::Maximum
- [-] `NormalMap::Maximum::Calculate` · 005caac0 — not needed: height->normal conversion only runs for loose source images whose file name contains '_bump' (bWantNormalMap); DAT textures never take it; water bump is procedural (AGENTS.md:174) [decompiled-pdb rendersurface.cpp.c:352,777]

### NormalMap::Lightness
- [-] `NormalMap::Lightness::Calculate` · 005cab40 — not needed: height->normal conversion only runs for loose source images whose file name contains '_bump' (bWantNormalMap); DAT textures never take it; water bump is procedural (AGENTS.md:174) [decompiled-pdb rendersurface.cpp.c:352,777]

### NormalMap::ColorSpace
- [-] `NormalMap::ColorSpace::Calculate` · 005cac00 — not needed: height->normal conversion only runs for loose source images whose file name contains '_bump' (bWantNormalMap); DAT textures never take it; water bump is procedural (AGENTS.md:174) [decompiled-pdb rendersurface.cpp.c:352,777]

### RenderSurfaceD3D
- [-] `RenderSurfaceD3D::RenderSurfaceD3D` · 005cac70 · legions 00591c60 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [-] `RenderSurfaceD3D::Create` · 005cacb0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderSurfaceD3D::Destroy` · 005cad10 · legions 00591d00 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderSurfaceD3D::Lock` · 005cad70 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderSurfaceD3D::Unlock` · 005cae30 · legions 00591e20 — D3D device/resource plumbing; wgpu replaces

### NormalMap
- [-] `NormalMap::Init` · 005caea0 · legions 00591e90 — not needed: only reached for '_bump' loose-file images (admin/tool path); no DAT material uses it; water bump map is procedural (AGENTS.md:174) [decompiled-pdb rendersurface.cpp.c:352,777]
- [-] `NormalMap::Convert24BitToNormalMap` · 005cafe0 · legions 00591fd0 — not needed: only reached for '_bump' loose-file images (admin/tool path); no DAT material uses it; water bump map is procedural (AGENTS.md:174) [decompiled-pdb rendersurface.cpp.c:352,777]

### RenderSurfaceD3D
- [-] `RenderSurfaceD3D::~RenderSurfaceD3D` · 005cb3f0 · legions 005923e0 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [-] `RenderSurfaceD3D::ConvertToNormalMap` · 005cb460 — not needed: only reached for '_bump' loose-file images (admin/tool path); no DAT material uses it; water bump map is procedural (AGENTS.md:174) [decompiled-pdb rendersurface.cpp.c:352,777]
- [-] `RenderSurfaceD3D::Convert` · 005cb5b0 — D3D device/resource plumbing; wgpu replaces

## `portal/client/src/render/core/surface/rendersurface.cpp.c` (4/18)

### RenderSurface
- [-] `RenderSurface::Begin` · 005b13f0 · legions 0057cea0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderSurface::Create` · 005b1450 · legions 0057cf00 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderSurface::Destroy` · 005b14f0 · legions 0057cfa0 — D3D device/resource plumbing; wgpu replaces
- [x] `RenderSurface::InitLoad` · 005b1530 · legions 0057cfe0 — audit: covered: DAT surface payloads (R8G8B8, A8R8G8B8, A8, DXT1-5, JFIF) decoded to GPU-ready pixels [AC2_Rusted/crates/ac2-assets/src/ac2_surface_pixels.rs:1]
- [-] `RenderSurface::Lock` · 005b1570 · legions 0057d010 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderSurface::SelectSurfaceFormat` · 005b15e0 · legions 0057d080 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderSurface::Convert` · 005b1690 · legions 0057d130 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderSurface::RenderSurface` · 005b1860 · legions 0057d300 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [-] `RenderSurface::GetDBOType` · 005b18c0 — DBO type tag; ac2-db owns type ids
- [-] `RenderSurface::~RenderSurface` · 005b18d0 · legions 0057d370 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [x] `RenderSurface::CreateFromSourceData` · 005b1930 · legions 0057d3d0 — audit: covered: DAT surface payloads (R8G8B8, A8R8G8B8, A8, DXT1-5, JFIF) decoded to GPU-ready pixels [AC2_Rusted/crates/ac2-assets/src/ac2_surface_pixels.rs:1]
- [x] `RenderSurface::StreamPack` · 005b1c40 · legions 0057d6d0 — RenderSurface::StreamPack cited in ac2-assets ac2_texture.rs decode_level (DID, category, size, format, bytes)
- [-] `RenderSurface::LoadDDS` · 005b1d60 — text source loader, admin build only
- [-] `RenderSurface::LoadJPGAsRawData` · 005b1eb0 — text source loader, admin build only
- [x] `RenderSurface::SaveJPG` · 005b1f70 · legions 0057d7f0 — audit: covered: JPEG via image crate, fixed quality 90 (retail's IJL quality sits in an undecoded jprops block) [AC2_Rusted/crates/ac2-gpu/src/screenshot.rs:19]
- [-] `RenderSurface::Allocator` · 005b2130 — custom allocator; Rust ownership
- [-] `RenderSurface::LoadFromFile` · 005b2170 — text source loader, admin build only
- [-] `RenderSurface::Unlock` · 005b2370 · legions 0057d9f0 — D3D device/resource plumbing; wgpu replaces

## `portal/client/src/render/core/surface/surfacewindow.cpp.c` (7/25)

### SurfaceWindow
- [-] `SurfaceWindow::End` · 005aed30 · legions 0057a7b0 — D3D device/resource plumbing; wgpu replaces
- [-] `SurfaceWindow::Attach` · 005aed40 · legions 0057a7c0 — D3D device/resource plumbing; wgpu replaces
- [-] `SurfaceWindow::Attach` · 005aede0 · legions 0057a860 — D3D device/resource plumbing; wgpu replaces
- [-] `SurfaceWindow::WindowToClip` · 005aee20 · legions 0057a8a0 — replaced by design: UI is placed by UiOverlay layers in pixels, no software window->clip transform (ac2-ui.md:2305 [-]) [AC2_Rusted/crates/ac2-gpu/src/ui.rs:1]
- [x] `SurfaceWindow::ClipToWindow` · 005aee60 · legions 0057a8e0 — SurfaceWindow::ClipToWindow cited in ac2-ui external_container.rs:159 (object centre to window pixels)
- [-] `SurfaceWindow::Lock` · 005aeeb0 · legions 0057a930 — D3D device/resource plumbing; wgpu replaces

### PixelOp<unsigned_long>
- [-] `PixelOp<unsigned_long>::ConvertPixel` · 005aef30 · legions 0057a9b0 — D3D device/resource plumbing; wgpu replaces

### PixelOp<unsigned_short>
- [-] `PixelOp<unsigned_short>::ConvertPixel` · 005af0b0 · legions 0057ab30 — D3D device/resource plumbing; wgpu replaces

### SurfaceWindow
- [-] `SurfaceWindow::SurfaceWindow` · 005af230 · legions 0057acb0 — plain data ctor/dtor/operator=; Rust struct or Drop replaces (SurfaceWindow rect ctor; edge convention in ac2-ui tree_view.rs:443)
- [-] `SurfaceWindow::SurfaceWindow` · 005af260 · legions 0057ace0 — plain data ctor/dtor/operator=; Rust struct or Drop replaces (SurfaceWindow rect ctor; edge convention in ac2-ui tree_view.rs:443)
- [-] `SurfaceWindow::SurfaceWindow` · 005af2a0 · legions 0057ad20 — plain data ctor/dtor/operator=; Rust struct or Drop replaces (SurfaceWindow rect ctor; edge convention in ac2-ui tree_view.rs:443)
- [-] `SurfaceWindow::SurfaceWindow` · 005af310 · legions 0057ad90 — plain data ctor/dtor/operator=; Rust struct or Drop replaces (SurfaceWindow rect ctor; edge convention in ac2-ui tree_view.rs:443)
- [x] `SurfaceWindow::DrawCharacter` · 005af390 · legions 0057ae10 — audit: covered: draw_character blends glyph coverage into RGBA surfaces; editing/selection are AGENTS.md:540 [AC2_Rusted/crates/ac2-ui/src/text.rs:415]

### SmartArray<PrefDisplayModeType>
- [-] `SmartArray<PrefDisplayModeType>::AddToEnd` · 005af8f0 · legions 0057b430 — SmartArray/STL container internals; Vec replaces

### SurfaceWindow
- [-] `SurfaceWindow::Unlock` · 005af950 · legions 0057b490 — D3D device/resource plumbing; wgpu replaces
- [x] `SurfaceWindow::Blit` · 005af9c0 · legions 0057b500 — audit: covered: blit with draw modes; surfaces are RGBA so colour key/format conversion do not arise [AC2_Rusted/crates/ac2-ui/src/compose.rs:113]
- [x] `SurfaceWindow::Multiply` · 005b00a0 · legions 0057bbd0 — ac2-ui icon_desc.rs multiply: each channel as byte/255 (RGBAColor::SetColor32's float) times the ARGB shift colour's, stored trunc(x * 255)
- [-] `SurfaceWindow::Fill` · 005b0350 · legions 0057be70 — replaced by design: callers are debug HUD/bench, IME, cursor build, default graphic; UI pixel fills live in ac2-ui [AC2_Rusted/crates/ac2-ui/src/radar.rs:293]
- [x] `SurfaceWindow::FillArea` · 005b0530 · legions 0057c050 — SurfaceWindow::FillArea cited in ac2-ui radar.rs:9 (blip pixels filled as areas)
- [x] `SurfaceWindow::FillArea` · 005b05e0 · legions 0057c100 — SurfaceWindow::FillArea cited in ac2-ui radar.rs:9 (blip pixels filled as areas)
- [x] `SurfaceWindow::DrawLine` · 005b0620 · legions 0057c140 — ac2-ui tree_view.rs
- [-] `SurfaceWindow::DrawBox` · 005b0910 — admin-only/tool/debug code, not in the client (no legions match; admin drawing) [vtable-checked: not a slot of any located Legions vtable]
- [-] `SurfaceWindow::DrawTextA` · 005b09a0 · legions 0057c430 — not needed: debug HUD / bench / IME wrapper only; UI text goes through ac2-ui text.rs [decompiled-pdb: callers benchGraphDraw, benchStat::Render, UpdateDebugHUD, CImeFontWrapper]
- [-] `SurfaceWindow::NotColorBits` · 005b0b40 · legions 0057c5c0 — D3D device/resource plumbing; wgpu replaces
- [-] `SurfaceWindow::GetColorFromPoint` · 005b0d80 · legions 0057c800 — replaced by design: callers: pixel scan (AGENTS.md:162) and GetShadowColor, which Rust computes from alphas on the CPU [AC2_Rusted/crates/ac2-render/src/terrain/shadow.rs:77]

## `portal/client/src/render/core/texture/d3d/rendertextured3d.cpp.c` (0/19)

### RenderTextureD3D
- [-] `RenderTextureD3D::GetSubObjects` · 005bb860 · legions 00584fc0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderTextureD3D::Get2DTextureD3D` · 005bb890 · legions 00584ff0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderTextureD3D::GetCubeTextureD3D` · 005bb8d0 · legions 00585030 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderTextureD3D::GetLevelRef` · 005bb910 · legions 00585070 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderTextureD3D::LoadTexture` · 005bb9d0 · legions 00585130 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderTextureD3D::ReleaseD3DSurfaces` · 005bba50 · legions 005851b0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderTextureD3D::PurgeResource` · 005bbac0 · legions 00585220 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderTextureD3D::CopyInto` · 005bbb20 · legions 00585280 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderTextureD3D::ReleaseSubObjects` · 005bbbe0 · legions 00585340 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderTextureD3D::CreateD3DTexture` · 005bbc40 · legions 005853a0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderTextureD3D::RestoreResource` · 005bbee0 · legions 00585640 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderTextureD3D::Create2D` · 005bbff0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderTextureD3D::CreateCube` · 005bc030 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderTextureD3D::Destroy` · 005bc070 · legions 005857d0 — D3D device/resource plumbing; wgpu replaces

### RenderTexture
- [-] `RenderTexture::RenderTexture` · 005bc0c0 · legions 00585820 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [-] `RenderTexture::Allocate` · 005bc110 — custom allocator; Rust ownership
- [-] `RenderTexture::GetDBOType` · 005bc120 — DBO type tag; ac2-db owns type ids

### RenderTextureD3D
- [-] `RenderTextureD3D::RenderTextureD3D` · 005bc190 · legions 005858f0 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [-] `RenderTextureD3D::~RenderTextureD3D` · 005bc230 · legions 00585980 — plain data ctor/dtor/operator=; Rust struct or Drop replaces

## `portal/client/src/render/core/texture/rendertexture.cpp.c` (7/29)

### RenderTexture
- [-] `RenderTexture::Create2D` · 005bcaa0 · legions 00586200 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderTexture::CreateCube` · 005bcaf0 · legions 00586260 — D3D device/resource plumbing; wgpu replaces
- [x] `RenderTexture::ShouldDropHighDetail` · 005bcb40 · legions 005862b0 — ResourceStore::dropped_texture_levels from TextureDetail (load_ac2_texture_chain); highres.dat mounted via for_retail_install
- [-] `RenderTexture::Refresh` · 005bcb60 · legions 005862d0 — replaced by design: D3D texture re-upload/refresh hook; wgpu textures made on demand [AC2_Rusted/AGENTS.md:20]
- [-] `RenderTexture::SelectTextureFormat` · 005bcb90 · legions 00586300 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderTexture::ReleaseSourceLevelReferences` · 005bcc20 · legions 00586390 — retail DB object cache handle/release; Arc per DID replaces (AGENTS.md exception)
- [-] `RenderTexture::ReleaseSubObjects` · 005bcc60 · legions 005863d0 — retail DB object cache handle/release; Arc per DID replaces (AGENTS.md exception)
- [-] `RenderTexture::GetDIDsForFilenames` · 005bcc80 — not needed: text source loader (admin tools) [decompiled-pdb: only caller RenderTexture::FromFileNode]
- [-] `RenderTexture::LoadLevelResources` · 005bcd20 — text source loader, admin build only
- [x] `RenderTexture::ConstructTexture` · 005bcdd0 · legions 00586490 — audit: covered: source levels loaded by DID from portal/local/highres DATs (279-326) [AC2_Rusted/crates/ac2-assets/src/ac2_texture.rs:279]

### DBLevelInfo
- [-] `DBLevelInfo::DBLevelInfo` · 005bcf80 — plain data ctor/dtor/operator=; Rust struct or Drop replaces

### RenderTexture
- [x] `RenderTexture::GetSubObjects` · 005bcfb0 · legions 00586650 — audit: covered: source levels loaded by DID from portal/local/highres DATs (279-326) [AC2_Rusted/crates/ac2-assets/src/ac2_texture.rs:279]

### _anon_1CBD2415::AssertionDialogConfig
- [-] `_anon_1CBD2415::AssertionDialogConfig::operator=` · 005bcff0 · legions 00586690 — MSVC/Windows assertion or iterator internals

### RenderTexture
- [-] `RenderTexture::Begin` · 005bd110 · legions 005867b0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderTexture::DestroySourceLevels` · 005bd160 · legions 00586810 — retail DB object cache handle/release; Arc per DID replaces (AGENTS.md exception)
- [-] `RenderTexture::ToFileNode` · 005bd1c0 — text source format (admin tools)

### SmartArray<DBLevelInfo>
- [-] `SmartArray<DBLevelInfo>::operator=` · 005bd360 · legions 00586870 — SmartArray/STL container internals; Vec replaces
- [-] `SmartArray<DBLevelInfo>::SetNElements` · 005bd400 · legions 00586910 — SmartArray/STL container internals; Vec replaces

### RenderTexture
- [-] `RenderTexture::End` · 005bd430 · legions 00586940 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderTexture::Allocator` · 005bd450 — custom allocator; Rust ownership
- [-] `RenderTexture::Destroy` · 005bd490 · legions 005869a0 — D3D device/resource plumbing; wgpu replaces
- [x] `RenderTexture::SetNumSourceLevels` · 005bd4c0 · legions 005869d0 — audit: covered: level count bound at StreamPack (ac2_texture.rs:371) [AC2_Rusted/crates/ac2-assets/src/ac2_texture.rs:22]
- [x] `RenderTexture::DropUnwantedLevels` · 005bd4f0 · legions 00586a00 — ResourceStore::dropped_texture_levels from TextureDetail (load_ac2_texture_chain); highres.dat mounted via for_retail_install
- [-] `RenderTexture::CopyInto` · 005bd600 · legions 00586b10 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderTexture::GetSubDataIDs` · 005bd750 — not needed: sub-DID enumeration for tools/preload; Rust loads levels on demand [decompiled-pdb: no callers, no Legions address]
- [x] `RenderTexture::InitLoad` · 005bd820 · legions 00586d40 — audit: covered: InitLoad = DropUnwantedLevels (Texture Detail pref) [AC2_Rusted/crates/ac2-db/src/store/texture_detail.rs:1]
- [x] `RenderTexture::StreamPack` · 005bd850 · legions 00586d70 — RenderTexture::StreamPack cited in ac2-assets ac2_texture.rs decode_chain (DID, texture type, counted level DIDs)
- [-] `RenderTexture::LoadFromFileNode` · 005bd940 — text source loader, admin build only
- [-] `RenderTexture::FromFileNode` · 005bdb90 — text source format (admin tools)

## `portal/client/src/render/core/vertexbuffer/d3d/rendervertexbufferd3d.cpp.c` (0/12)

### RenderVertexBufferD3D
- [-] `RenderVertexBufferD3D::Lock` · 005dc1c0 · legions 0059cfc0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderVertexBufferD3D::Begin` · 005dc210 · legions 0059d010 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderVertexBufferD3D::SetupVertexStreams` · 005dc260 · legions 0059d060 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderVertexBufferD3D::ReleaseVertexStreams` · 005dc2d0 · legions 0059d0d0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderVertexBufferD3D::End` · 005dc360 · legions 0059d160 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderVertexBufferD3D::Shutdown` · 005dc370 · legions 0059d170 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderVertexBufferD3D::UpdateVertexBuffer` · 005dc3d0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderVertexBufferD3D::AddDirtyRange` · 005dc520 · legions 0059d2b0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderVertexBufferD3D::CreateVertexStreams` · 005dc5f0 · legions 0059d380 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderVertexBufferD3D::RenderIndexedPrimitives` · 005dc6c0 · legions 0059d450 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderVertexBufferD3D::RenderPrimitives` · 005dc8b0 · legions 0059d5f0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderVertexBufferD3D::Startup` · 005dca40 · legions 0059d730 — D3D device/resource plumbing; wgpu replaces

## `portal/client/src/render/core/vertexbuffer/d3d/rendervertexstreamd3d.cpp.c` (0/10)

### RenderVertexStreamD3D
- [-] `RenderVertexStreamD3D::Begin` · 005dbd30 · legions 0059cb40 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderVertexStreamD3D::CreateD3DVertexBuffer` · 005dbd70 · legions 0059cb80 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderVertexStreamD3D::GetD3DVertexBuffer` · 005dbea0 · legions 0059ccb0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderVertexStreamD3D::FillData` · 005dbed0 · legions 0059cce0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderVertexStreamD3D::ResetVertexIndices` · 005dc040 · legions 0059ce40 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderVertexStreamD3D::PurgeResource` · 005dc060 · legions 0059ce60 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderVertexStreamD3D::RestoreResource` · 005dc0a0 · legions 0059cea0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderVertexStreamD3D::End` · 005dc0c0 · legions 0059cec0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderVertexStreamD3D::Create` · 005dc0f0 · legions 0059cef0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderVertexStreamD3D::Destroy` · 005dc160 · legions 0059cf60 — D3D device/resource plumbing; wgpu replaces

## `portal/client/src/render/core/vertexbuffer/rendervertexbuffer.cpp.c` (6/17)

### RenderVertexBuffer
- [-] `RenderVertexBuffer::Begin` · 005b6f90 · legions 005813c0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderVertexBuffer::End` · 005b6fb0 · legions 005813e0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderVertexBuffer::Startup` · 005b6fd0 · legions 00581400 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderVertexBuffer::Shutdown` · 005b7080 · legions 005814b0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderVertexBuffer::UnlockAnimVertices` · 005b70c0 · legions 005814f0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderVertexBuffer::AnimateVertex` · 005b70e0 · legions 00581510 — replaced by design: copies source vertices into the animated (blend) vertex buffer for CPU skinning; skinning runs on the GPU in object.wgsl [AC2_Rusted/crates/ac2-render/src/object/skeleton.rs:11]
- [-] `RenderVertexBuffer::BlendVertices` · 005b7260 · legions 00581690 — modern pass: GPU skinning always blends normals (BlendLit); retail's ModelDetail<2 path skipped normal blending for speed and bounds update is not drawn-relevant (AGENTS.md:116) [AC2_Rusted/crates/ac2-gpu/src/object.wgsl]
- [-] `RenderVertexBuffer::AnimateAllVertices` · 005b7530 · legions 00581c00 — replaced by design: copies source vertices into the animated (blend) vertex buffer for CPU skinning; skinning runs on the GPU in object.wgsl [AC2_Rusted/crates/ac2-render/src/object/skeleton.rs:11]
- [-] `RenderVertexBuffer::Unlock` · 005b7590 · legions 00581930 — D3D device/resource plumbing; wgpu replaces
- [x] `RenderVertexBuffer::IlluminateVertices` · 005b75b0 · legions 00581950
- [x] `RenderVertexBuffer::RenderIndexBufferPrimitives` · 005b77a0 · legions 00581b40 — audit: covered: material layer/stage order, blend, depth, detail and fog gating are laid out by object::pass and drawn by ac2-gpu object pipelines [AC2_Rusted/crates/ac2-render/src/object/pass.rs:3]
- [-] `RenderVertexBuffer::AnimateVertices` · 005b7860 — replaced by design: copies source vertices into the animated (blend) vertex buffer for CPU skinning; skinning runs on the GPU in object.wgsl [AC2_Rusted/crates/ac2-render/src/object/skeleton.rs:11]
- [x] `RenderVertexBuffer::RenderIndexBufferUsingMaterial` · 005b7880 · legions 00581c50 — audit: covered: material layer/stage order, blend, depth, detail and fog gating are laid out by object::pass and drawn by ac2-gpu object pipelines [AC2_Rusted/crates/ac2-render/src/object/pass.rs:3]
- [x] `RenderVertexBuffer::RenderIndexedUsingMaterial` · 005b7a50 · legions 00581e10 — ac2-render object::pass / texture_matrix, ac2-gpu object
- [x] `RenderVertexBuffer::RenderUsingMaterial` · 005b7c40 · legions 00581ff0 — audit: covered: material layer/stage order, blend, depth, detail and fog gating are laid out by object::pass and drawn by ac2-gpu object pipelines [AC2_Rusted/crates/ac2-render/src/object/pass.rs:3]
- [x] `RenderVertexBuffer::RayIntersectsTriList` · 005b7e00 · legions 005821b0 — audit: covered: Moller-Trumbore ray vs tri list in ac2-physics (los/mesh.rs:43) [AC2_Rusted/crates/ac2-physics/src/part_array.rs:434]
- [-] `RenderVertexBuffer::LockAnimVertices` · 005b8450 · legions 00582800 — D3D device/resource plumbing; wgpu replaces

## `portal/client/src/render/decal/decaleffect.cpp.c` (8/9)

### DecalEffect
- [x] `DecalEffect::ClipPolygonAgainstPlane` · 005dd100 · legions 0059ddf0 — decal FX: ac2-render decal.rs (clip, fade, geometry per material), fx_decals.rs (spawn), scene/decals.rs (ground from resident terrain); tests: decal_tests, fx_decals_tests, real-data decode, headless draw
- [x] `DecalEffect::DecalEffect` · 005de3e0 · legions 0059f170 — decal FX: ac2-render decal.rs (clip, fade, geometry per material), fx_decals.rs (spawn), scene/decals.rs (ground from resident terrain); tests: decal_tests, fx_decals_tests, real-data decode, headless draw
- [x] `DecalEffect::Think` · 005de430 · legions 0059f1c0 — decal FX: ac2-render decal.rs (clip, fade, geometry per material), fx_decals.rs (spawn), scene/decals.rs (ground from resident terrain); tests: decal_tests, fx_decals_tests, real-data decode, headless draw
- [x] `DecalEffect::CreateGeometry` · 005de500 · legions 0059f290 — decal FX: ac2-render decal.rs (clip, fade, geometry per material), fx_decals.rs (spawn), scene/decals.rs (ground from resident terrain); tests: decal_tests, fx_decals_tests, real-data decode, headless draw
- [-] `DecalEffect::~DecalEffect` · 005de7b0 · legions 0059f540 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [x] `DecalEffect::ConstructDecal` · 005de7e0 · legions 0059f570 — decal FX: ac2-render decal.rs (clip, fade, geometry per material), fx_decals.rs (spawn), scene/decals.rs (ground from resident terrain); tests: decal_tests, fx_decals_tests, real-data decode, headless draw
- [x] `DecalEffect::ConstructLandscapeDecal` · 005dee50 · legions 0059fbe0 — decal FX: ac2-render decal.rs (clip, fade, geometry per material), fx_decals.rs (spawn), scene/decals.rs (ground from resident terrain); tests: decal_tests, fx_decals_tests, real-data decode, headless draw
- [x] `DecalEffect::ConstructDungeonDecal` · 005df1e0 · legions 0059ff70 — ac2-render scene/decals.rs dungeon_decal_ground + cell_geometry.rs nearby_geometry: the decal carries its cell (DecalSpec::cell); the block's resident CullCell (CObjCell::GetVisible) gives the cell mesh first degrade (CellGeometry, Arcs shared with the decoded mesh) and its portal polygons; triangles whose cell-local bounds overlap the box of the largest edge, plus one ring of portal neighbours whose portal bounds overlap, go through construct_decal. Not resident: no decal. Tests: nearby_geometry unit test, dungeon_decal_tests real data
- [x] `DecalEffect::Init` · 005df350 · legions 005a00e0 — decal FX: DecalEffect::new builds from the ground triangles; WorldScene::start_object_decal picks the landscape ground (cell below 0x100) or the env cell geometry (DecalEffect::Init split); tests: decal_tests, fx_decals_tests, dungeon_decal_tests

## `portal/client/src/render/decal/decaleffectgroup.cpp.c` (3/5)

### DecalEffectGroup
- [-] `DecalEffectGroup::Init` · 005db6c0 — plain data defaults, container growth or render vertex buffer; Rust structs and Vec replace them
- [-] `DecalEffectGroup::~DecalEffectGroup` · 005db730 · legions 0059c540 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [x] `DecalEffectGroup::Think` · 005db7b0 · legions 0059c5c0 — decal FX: ac2-render decal.rs (clip, fade, geometry per material), fx_decals.rs (spawn), scene/decals.rs (ground from resident terrain); tests: decal_tests, fx_decals_tests, real-data decode, headless draw
- [x] `DecalEffectGroup::Render` · 005db850 · legions 0059c660 — decal FX: ac2-render decal.rs (clip, fade, geometry per material), fx_decals.rs (spawn), scene/decals.rs (ground from resident terrain); tests: decal_tests, fx_decals_tests, real-data decode, headless draw
- [x] `DecalEffectGroup::CreateDecalEffect` · 005db920 · legions 0059c730 — decal FX: ac2-render decal.rs (clip, fade, geometry per material), fx_decals.rs (spawn), scene/decals.rs (ground from resident terrain); tests: decal_tests, fx_decals_tests, real-data decode, headless draw

## `portal/client/src/render/decal/decaleffectmanager.cpp.c` (3/6)

### DecalEffectManager
- [-] `DecalEffectManager::Init` · 005c38c0 · legions 0058a9d0 — plain data defaults, container growth or render vertex buffer; Rust structs and Vec replace them
- [-] `DecalEffectManager::DecalEffectManager` · 005c3900 · legions 0058aa10 — plain data defaults, container growth or render vertex buffer; Rust structs and Vec replace them
- [-] `DecalEffectManager::~DecalEffectManager` · 005c3920 · legions 0058aa30 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [x] `DecalEffectManager::Think` · 005c39b0 · legions 0058aac0 — decal FX: ac2-render decal.rs (clip, fade, geometry per material), fx_decals.rs (spawn), scene/decals.rs (ground from resident terrain); tests: decal_tests, fx_decals_tests, real-data decode, headless draw
- [x] `DecalEffectManager::Render` · 005c3a30 · legions 0058ab40 — decal FX: ac2-render decal.rs (clip, fade, geometry per material), fx_decals.rs (spawn), scene/decals.rs (ground from resident terrain); tests: decal_tests, fx_decals_tests, real-data decode, headless draw
- [x] `DecalEffectManager::CreateDecalEffect` · 005c3c10 · legions 0058ad20 — decal FX: ac2-render decal.rs (clip, fade, geometry per material), fx_decals.rs (spawn), scene/decals.rs (ground from resident terrain); tests: decal_tests, fx_decals_tests, real-data decode, headless draw

## `portal/client/src/render/flare/lensflareeffect.cpp.c` (6/14)

### LensFlarePoint
- [-] `LensFlarePoint::LensFlarePoint` · 005b6050 · legions 005809f0 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [-] `LensFlarePoint::~LensFlarePoint` · 005b6070 · legions 00580a10 — plain data ctor/dtor/operator=; Rust struct or Drop replaces

### LensFlareEffect
- [x] `LensFlareEffect::LensFlareEffect` · 005b6090 · legions 00580a30 — ac2-gpu lens_flare.rs: sky flares (LScapeSky::Draw(1)) with fade by a depth scan; sprite textures requested by ac2-render WorldScene; AGENTS.md exception
- [-] `LensFlareEffect::~LensFlareEffect` · 005b60c0 · legions 00580a60 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [-] `LensFlareEffect::Destroy` · 005b60f0 · legions 00580a90 — retail DB object cache handle/release; Arc per DID replaces (AGENTS.md exception)
- [x] `LensFlareEffect::InitNew` · 005b6130 · legions 00580ad0 — ac2-render scene/interiors.rs CellCorona: m_fCoronaOpacity start, m_fCoronaFadeSpeed, one point at axis -1 (CEnvCell::CreateLightSource's corona); corona properties decoded in ac2-world ac2_light_info.rs
- [x] `LensFlareEffect::GetSubDataIDs` · 005b6160 · legions 00580b00 — ac2-gpu lens_flare.rs: sky flares (LScapeSky::Draw(1)) with fade by a depth scan; sprite textures requested by ac2-render WorldScene; AGENTS.md exception
- [x] `LensFlareEffect::StreamPack` · 005b61f0 · legions 00580b90 — ac2-world ac2_region.rs read_lens_flare: fade speed and counted points kept on SkyObject/CelestialPosition
- [-] `LensFlareEffect::CopyInto` · 005b62f0 · legions 00580c90 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [x] `LensFlareEffect::Draw` · 005b63d0 · legions 00580d70 — ac2-gpu lens_flare.rs: sky flares (LScapeSky::Draw(1)) with fade by a depth scan; sprite textures requested by ac2-render WorldScene; AGENTS.md exception

### SmartArray<LensFlarePoint>
- [-] `SmartArray<LensFlarePoint>::AddToEnd` · 005b6980 · legions 00581320 — SmartArray/STL container internals; Vec replaces

### LensFlareEffect
- [x] `LensFlareEffect::AddPoint` · 005b69e0 · legions 00581380 — ac2-render scene/interiors.rs CellCorona: m_fCoronaOpacity start, m_fCoronaFadeSpeed, one point at axis -1 (CEnvCell::CreateLightSource's corona); corona properties decoded in ac2-world ac2_light_info.rs
- [-] `LensFlareEffect::FromFileNode` · 005b6a20 — text source format (admin tools)
- [-] `LensFlareEffect::ToFileNode` · 005b6d30 — text source format (admin tools)

## `portal/client/src/render/gamerender.cpp.c` (0/2)

### GameRender
- [-] `GameRender::Init` · 005b84e0 — replaced by design: device/render subsystem start-up (Device::StartupGraphicsEngine); wgpu replaces (ac2-render.md Render::Startup [-]) [AC2_Rusted/crates/ac2-gpu/src/lib.rs]
- [-] `GameRender::Cleanup` · 005b8740 — D3D device lifecycle/restore; wgpu replaces

## `portal/client/src/render/particle/particle.cpp.c` (0/41)

### ParticleWrapper
- [-] `ParticleWrapper::ParticleWrapper` · 005dca80 · legions 0059d770 — plain data ctor/dtor/operator=; Rust struct or Drop replaces

### ParticleFormatInfo
- [-] `ParticleFormatInfo::ParticleFormatInfo` · 005dcaa0 · legions 0059d790 — plain data ctor/dtor/operator=; Rust struct or Drop replaces

### ParticleWrapper
- [-] `ParticleWrapper::GetScale` · 005dcc70 · legions 0059d960 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [-] `ParticleWrapper::SetScale` · 005dcc80 · legions 0059d970 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [-] `ParticleWrapper::GetTimeCreated` · 005dcc90 · legions 0059d980 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [-] `ParticleWrapper::SetTimeCreated` · 005dcca0 · legions 0059d990 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [-] `ParticleWrapper::GetStartOrigin` · 005dccb0 · legions 0059d9a0 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [-] `ParticleWrapper::SetStartOrigin` · 005dccc0 · legions 0059d9b0 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [-] `ParticleWrapper::GetOrigin` · 005dcce0 · legions 0059d9d0 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [-] `ParticleWrapper::SetOrigin` · 005dccf0 · legions 0059d9e0 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [-] `ParticleWrapper::GetWorldOrigin` · 005dcd10 · legions 0059da00 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [-] `ParticleWrapper::SetWorldOrigin` · 005dcd20 · legions 0059da10 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [-] `ParticleWrapper::GetPreviousOrigin` · 005dcd40 · legions 0059da30 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [-] `ParticleWrapper::SetPreviousOrigin` · 005dcd50 · legions 0059da40 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [-] `ParticleWrapper::GetVelocity` · 005dcd70 · legions 0059da60 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [-] `ParticleWrapper::SetVelocity` · 005dcd80 · legions 0059da70 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [-] `ParticleWrapper::GetStreak` · 005dcda0 · legions 0059da90 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [-] `ParticleWrapper::SetStreak` · 005dcdb0 · legions 0059daa0 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [-] `ParticleWrapper::GetStreakOrigin` · 005dcdc0 · legions 0059dab0 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [-] `ParticleWrapper::SetStreakOrigin` · 005dcdd0 · legions 0059dac0 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [-] `ParticleWrapper::GetPreviousStreakOrigin` · 005dcdf0 · legions 0059dae0 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [-] `ParticleWrapper::SetPreviousStreakOrigin` · 005dce00 · legions 0059daf0 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [-] `ParticleWrapper::GetRotation` · 005dce20 · legions 0059db10 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [-] `ParticleWrapper::SetRotation` · 005dce30 · legions 0059db20 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [-] `ParticleWrapper::GetWorldRotation` · 005dce50 · legions 0059db40 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [-] `ParticleWrapper::SetWorldRotation` · 005dce60 · legions 0059db50 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [-] `ParticleWrapper::GetStartRotation` · 005dce80 · legions 0059db70 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [-] `ParticleWrapper::SetStartRotation` · 005dce90 · legions 0059db80 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [-] `ParticleWrapper::GetStartWorldRotation` · 005dceb0 · legions 0059dba0 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [-] `ParticleWrapper::SetStartWorldRotation` · 005dcec0 · legions 0059dbb0 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [-] `ParticleWrapper::GetRotateVelocity` · 005dcee0 · legions 0059dbd0 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [-] `ParticleWrapper::SetRotateVelocity` · 005dcef0 · legions 0059dbe0 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [-] `ParticleWrapper::GetKeyframeFlags` · 005dcf10 · legions 0059dc00 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [-] `ParticleWrapper::SetKeyframeFlags` · 005dcf20 · legions 0059dc10 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [-] `ParticleWrapper::GetKeyframeInfo` · 005dcf30 · legions 0059dc20 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [-] `ParticleWrapper::GetWaveformVector` · 005dcf40 · legions 0059dc30 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [-] `ParticleWrapper::SetWaveformVector` · 005dcf50 · legions 0059dc40 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [-] `ParticleWrapper::GetPointIndex` · 005dcf60 · legions 0059dc50 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [-] `ParticleWrapper::SetPointIndex` · 005dcf70 · legions 0059dc60 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [-] `ParticleWrapper::GetPointSpeed` · 005dcf90 · legions 0059dc80 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [-] `ParticleWrapper::SetPointSpeed` · 005dcfa0 · legions 0059dc90 — trivial accessor of plain particle/record field; Rust struct field replaces it

## `portal/client/src/render/particle/particleemitter.cpp.c` (36/40)

### ParticleEmitter
- [x] `ParticleEmitter::SetLastDischargeTime` · 005d07a0 · legions 00593e90 — ac2-render particles

### FillVerts
- [x] `FillVerts::GetNullRotationMatrix` · 005d07b0 — ac2-render particles
- [x] `FillVerts::GetWorldRotationMatrix` · 005d07f0 — ac2-render particles
- [x] `FillVerts::GetRotationMatrix` · 005d0840 — ac2-render particles
- [x] `FillVerts::GetVertexPositions` · 005d0880 · legions 00593f70 — ac2-render particles
- [x] `FillVerts::GetStreakedVertexPositions` · 005d0940 — ported: ac2-render particles/fill.rs streaked; fixed to the shipped branch (Legions 0x00594030: |u.x|<=0.9238795 and |u.x|>0.3826834 both jump to the axis layouts, so the diagonal layouts are dead); test a_diagonal_streak_takes_the_axis_layout
- [x] `FillVerts::RotateVertex` · 005d0f50 — ac2-render particles

### ParticleBuffer
- [-] `ParticleBuffer::Allocate` · 005d1000 · legions 005946f0 — D3D device/resource plumbing; wgpu replaces

### ParticleEmitter
- [x] `ParticleEmitter::ParticleEmitter` · 005d1060 · legions 00594750 — ParticleEmitter::ParticleEmitter cited in ac2-render particles/emitter.rs:41 (Emitter::new: keys, flags, max, zeroed timers)
- [-] `ParticleEmitter::~ParticleEmitter` · 005d1150 · legions 00594840 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [-] `ParticleEmitter::GetDesc` · 005d11b0 · legions 005948a0 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [x] `ParticleEmitter::GetReadyToDie` · 005d11d0 · legions 005948c0 — ac2-render particles/emitter.rs ready_to_die and particles/mod.rs ready_to_die: the emitter's inactive, time-limit and emission-limit tests and the system's ten-thousand second wait, checked against the decompile
- [x] `ParticleEmitter::DetermineDataFlags` · 005d1250 · legions 00594940 — ac2-render particles
- [x] `ParticleEmitter::GetParticleNumVerts` · 005d1540 · legions 00594c30 — ac2-render particles
- [x] `ParticleEmitter::CalculateInitialOrigin` · 005d1580 · legions 00594c70 — ac2-render particles
- [x] `ParticleEmitter::DoWaveform` · 005d1850 · legions 00594f40 — ac2-render particles
- [x] `ParticleEmitter::GetMass` · 005d1980 · legions 00595070 — ac2-render particles
- [x] `ParticleEmitter::DoPhysics` · 005d1a20 · legions 00595110 — ac2-render particles
- [x] `ParticleEmitter::DoRepulse` · 005d1cb0 · legions 005953a0 — ac2-render particles/controllers.rs do_repulse (x87 order kept in f64, rounded at float stores); nearby repulsors per scene stamp (Repulsors::nearby); VolumetricFogEffects off holds repulse systems back as ParticleSystem::Think/Render do
- [x] `ParticleEmitter::DoPointFile` · 005d2150 · legions 00595840 — ac2-render particles
- [x] `ParticleEmitter::DoPointFileLerp` · 005d2290 · legions 00595980 — ac2-render particles

### FillVerts
- [x] `FillVerts::GetOrigin` · 005d2550 · legions 00595c40 — ac2-render particles
- [x] `FillVerts::GetBlendedOrigin` · 005d2570 — ac2-render particles
- [x] `FillVerts::GetNullStreakOrigin` · 005d2600 — ac2-render particles
- [x] `FillVerts::GetStreakOrigin` · 005d2620 — ac2-render particles
- [x] `FillVerts::GetBlendedStreakOrigin` · 005d2640 — ac2-render particles
- [x] `FillVerts::GetLighting` · 005d26d0 — ported: ac2-render particles/lighting.rs light_particle (inlined into Legions 0x00595F90 FillVertexBuffer): rgb times the sum over the first five lights of GetDiffuse x CalculateAttenuation (ambient never set, Black is opaque); 121 7230 PSDescs (templates 0x2B000073/9C/9D) are lit (Ac2ParticleSystemDesc::particle_lighting); lights from scene/particle_lights.rs (GetLightsForPosition radius 0: CacheStaticLights, AddDynamicLights, SortLights, then the directional sun outside)

### ParticleEmitter
- [x] `ParticleEmitter::FillVertexBuffer` · 005d28a0 · legions 00595f90 — ported: particles/fill.rs Emitter::fill (opacity fade, keyframe size/colour, world-origin scale, streak/rotation/quad corners, GetLighting, clamp); vertex budget not modelled
- [x] `ParticleEmitter::SetPosition` · 005d3690 · legions 00596d80 — covered: m_cellID side is ParticleSystem::set_indoor; the last-discharge positions only feed EmitDistanceParticles, which particles/mod.rs documents as not modelled (placed systems never move); cited on ParticleSystem::place
- [x] `ParticleEmitter::ApplyKeyframe` · 005d3850 · legions 00596f40 — ported: particles/keyframes.rs apply_keyframe/finish_controller; fixed the rotation carry to the new key own time (Legions 0x00596F40 adds rotvel x m_Time, not the controller span); test a_particle_turns_over_its_life_and_a_key_carries_its_time
- [x] `ParticleEmitter::ApplyKeyframes` · 005d3d10 · legions 00597400 — ac2-render particles
- [x] `ParticleEmitter::ParticleThink` · 005d3de0 · legions 005974d0 — ported: particles/emitter.rs particle_think; fixed the turn to rotvel x the whole time alive (Legions 0x005974D0), not the time since the controller key
- [x] `ParticleEmitter::Think` · 005d40a0 · legions 00597790 — ported: particles/emitter.rs Emitter::think (physics-step gate, fQuantum denormal after the first live particle)
- [x] `ParticleEmitter::DischargeParticle` · 005d42a0 · legions 00597990 — ported: particles/emitter.rs discharge_particle (with discharge / discharge_at for the outside callers)
- [x] `ParticleEmitter::SetupKeyframes` · 005d4bb0 · legions 005982a0 — ac2-render particles
- [x] `ParticleEmitter::Initialize` · 005d4e30 · legions 00598520 — ported: particles/emitter.rs Emitter::new (ParticleEmitter::ParticleEmitter, Initialize, Reset; DetermineDataFlags in data_flags)
- [x] `ParticleEmitter::Reset` · 005d4ed0 · legions 005985c0 — ac2-render particles
- [x] `ParticleEmitter::EmitTimedParticles` · 005d4f00 · legions 005985f0 — ported: particles/emitter.rs emit_timed_particles
- [-] `ParticleEmitter::EmitDistanceParticles` · 005d5280 · legions 00598970 — placed systems never move: emission by distance travelled needs a moving system
- [x] `ParticleEmitter::EmitParticles` · 005d58a0 · legions 00598f90 — ported: particles/emitter.rs emit_particles (timed emission; distance emission not modelled, documented in particles/mod.rs)

## `portal/client/src/render/particle/particlerepulsor.cpp.c` (2/4)

### ParticleRepulsor
- [x] `ParticleRepulsor::ParticleRepulsor` · 005c3790 · legions 0058a8a0 — ac2-render particles/repulsor.rs: ParticleRepulsor (new, place = SetPosition + stamp + speed, is_active, is_ready_to_die) and Repulsors (PSManager::CreateRepulsor/DestroyRepulsor, s_Repulsors)
- [-] `ParticleRepulsor::GetIsReadyToDie` · 005c37f0 · legions 0058a900 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [-] `ParticleRepulsor::GetIsActive` · 005c3820 · legions 0058a930 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [x] `ParticleRepulsor::SetPosition` · 005c3870 · legions 0058a980 — ac2-render particles/repulsor.rs: ParticleRepulsor (new, place = SetPosition + stamp + speed, is_active, is_ready_to_die) and Repulsors (PSManager::CreateRepulsor/DestroyRepulsor, s_Repulsors)

## `portal/client/src/render/particle/particlesystem.cpp.c` (13/22)

### ParticleSystem
- [-] `ParticleSystem::AddRef` · 005c22a0 · legions 005894e0 — retail DB object cache handle/release; Arc per DID replaces (AGENTS.md exception)
- [-] `ParticleSystem::SetCanBeShared` · 005c22b0 · legions 005894f0 — replaced by design: shared systems are a pooling optimisation (one system drawn at many positions); each source runs its own, only difference is identical vs independent random streams [AC2_Rusted/crates/ac2-render/src/particles/mod.rs:15]
- [x] `ParticleSystem::GetForceDraw` · 005c22c0 · legions 00589500 — ac2-render particles
- [x] `ParticleSystem::Begin` · 005c22e0 · legions 00589520 — covered: ParticleSystem::new defaults (unit scales, not disabled, fading, not placed); cited on new()
- [x] `ParticleSystem::GetExternalForces` · 005c2350 · legions 00589590 — ParticleSystem::GetExternalForces cited in ac2-render particles/emitter.rs:60 (world gravity, no wind)

### Position
- [-] `Position::Position` · 005c2520 · legions 00475a40 — plain data ctor/dtor/operator=; Rust struct or Drop replaces

### ParticleSystem
- [-] `ParticleSystem::CanBeShared` · 005c25b0 · legions 005897b0 — replaced by design: shared systems are a pooling optimisation (one system drawn at many positions); each source runs its own, only difference is identical vs independent random streams [AC2_Rusted/crates/ac2-render/src/particles/mod.rs:15]
- [x] `ParticleSystem::GetReadyToDie` · 005c25f0 · legions 005897f0 — ac2-render particles/emitter.rs ready_to_die and particles/mod.rs ready_to_die: the emitter's inactive, time-limit and emission-limit tests and the system's ten-thousand second wait, checked against the decompile
- [-] `ParticleSystem::Destroy` · 005c2680 · legions 00589880 — retail DB object cache handle/release; Arc per DID replaces (AGENTS.md exception)
- [x] `ParticleSystem::Start` · 005c2700 · legions 00589900 — audit: covered: ported (particles/*); catalog caveat remains: rotation carry and streak diagonal branch not compared with retail output [AC2_Rusted/crates/ac2-render/src/particles/mod.rs:155]
- [x] `ParticleSystem::Think` · 005c2770 · legions 00589970 — ported: particles/mod.rs ParticleSystem::think / think_with
- [x] `ParticleSystem::GetLightingOrigin` · 005c28b0 · legions 00589a80 — ported: particles/fill.rs adds the placement origin of a non-world-space system to the particle origin for the lighting distance (FillContext::placement, else zero)

### ParticleSystem::ParticleRenderPos
- [-] `ParticleSystem::ParticleRenderPos::operator=` · 005c2910 · legions 00589ae0 — plain data ctor/dtor/operator=; Rust struct or Drop replaces

### ParticleSystem
- [x] `ParticleSystem::ParticleSystem` · 005c2a40 · legions 00589c10 — covered: particles/mod.rs ParticleSystem::new
- [-] `ParticleSystem::~ParticleSystem` · 005c2ad0 · legions 00589ca0 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [-] `ParticleSystem::Release` · 005c2b30 · legions 00589d00 — retail DB object cache handle/release; Arc per DID replaces (AGENTS.md exception)
- [x] `ParticleSystem::FillVertexBuffer` · 005c2bf0 · legions 00589dc0 — ported: ParticleSystem::fill (ac2-render particles/mod.rs): four-corner emitters first, then triangles; GetLightsForPosition at the first render position when layer 0 of the template has m_bUseParticleLighting (PE_LightFlag Light=0); vertex budget is not modelled (geometry grows)
- [x] `ParticleSystem::RenderAtPosition` · 005c2e80 · legions 0058a050 — ported: ParticleSystem::fade (stop/start fade distance), set_disabled and fill placement (scene/particles.rs draw)
- [x] `ParticleSystem::Render` · 005c3250 · legions 0058a420 — ported: scene/particles.rs draw (scene stamp, VolumetricFogEffects gate for repulse systems, scale type 2 particle scale, disabled when not drawn)

### SmartArray<ParticleSystem::ParticleRenderPos>
- [-] `SmartArray<ParticleSystem::ParticleRenderPos>::AddToEnd` · 005c3450 · legions 0058a560 — SmartArray/STL container internals; Vec replaces

### ParticleSystem
- [x] `ParticleSystem::Create` · 005c34e0 · legions 0058a5f0 — ported: ParticleSystem::new (Create + Start, fast-forward)
- [x] `ParticleSystem::Queue` · 005c3600 · legions 0058a710 — ported: ParticleSystem::place (ParticleSystem::Queue)

## `portal/client/src/render/particle/psmanager.cpp.c` (7/15)

### PSManager
- [-] `PSManager::Startup` · 005c0540 · legions 005877e0 — replaced by design: shared quad vertex buffer and registry; Rust merges quads per material per frame [AC2_Rusted/crates/ac2-render/src/scene/particles.rs:1]

### PSManager::AutonomousSystemInfo
- [-] `PSManager::AutonomousSystemInfo::AutonomousSystemInfo` · 005c0670 — plain data ctor/dtor/operator=; Rust struct or Drop replaces

### PSManager
- [x] `PSManager::DestroyRepulsor` · 005c0700 · legions 005879a0 — ac2-render particles/repulsor.rs: ParticleRepulsor (new, place = SetPosition + stamp + speed, is_active, is_ready_to_die) and Repulsors (PSManager::CreateRepulsor/DestroyRepulsor, s_Repulsors)
- [x] `PSManager::CreateParticleSystem` · 005c0760 · legions 00587a00 — audit: covered: systems are created per source/object FX incl. anchor, water-plane frame and autonomous follow=false path (ParticleAnchor 30-40) [AC2_Rusted/crates/ac2-render/src/scene/particles.rs:1]
- [-] `PSManager::UnregisterSystem` · 005c0850 · legions 00587ae0 — replaced by design: global s_Systems registry only serves sharing/render iteration; the scene owns its systems [AC2_Rusted/crates/ac2-render/src/scene/particles.rs:1]

### PSManager::AutonomousSystemInfo
- [-] `PSManager::AutonomousSystemInfo::operator=` · 005c0890 · legions 00587b20 — plain data ctor/dtor/operator=; Rust struct or Drop replaces

### SmartArray<PSManager::AutonomousSystemInfo>
- [-] `SmartArray<PSManager::AutonomousSystemInfo>::operator=` · 005c09c0 · legions 00587c50 — SmartArray/STL container internals; Vec replaces

### PSManager
- [x] `PSManager::UnregisterAutonomousSystem` · 005c0ae0 · legions 00587d70 — audit: covered: released systems keep their last place until ready to die (RegisterAutonomousSystem) [AC2_Rusted/crates/ac2-render/src/scene/particles.rs:5]

### SmartArray<PSManager::AutonomousSystemInfo>
- [-] `SmartArray<PSManager::AutonomousSystemInfo>::AddToEnd` · 005c0b70 · legions 00587e00 — SmartArray/STL container internals; Vec replaces

### PSManager
- [-] `PSManager::Shutdown` · 005c0c00 · legions 00587e90 — replaced by design: shared quad vertex buffer and registry; Rust merges quads per material per frame [AC2_Rusted/crates/ac2-render/src/scene/particles.rs:1]
- [x] `PSManager::CreateRepulsor` · 005c0d20 · legions 00587fb0 — ac2-render particles/repulsor.rs: ParticleRepulsor (new, place = SetPosition + stamp + speed, is_active, is_ready_to_die) and Repulsors (PSManager::CreateRepulsor/DestroyRepulsor, s_Repulsors)
- [x] `PSManager::RenderParticleSystems` · 005c0db0 · legions 00588040 — PSManager::RenderParticleSystems cited in ac2-render scene/mod.rs:83 (particles merged per material)
- [-] `PSManager::RegisterSystem` · 005c0e60 · legions 005880f0 — replaced by design: global s_Systems registry only serves sharing/render iteration; the scene owns its systems [AC2_Rusted/crates/ac2-render/src/scene/particles.rs:1]
- [x] `PSManager::RegisterAutonomousSystem` · 005c0ec0 · legions 00588150 — PSManager::RegisterAutonomousSystem cited in ac2-render scene/particles.rs:6 (keep last place until ready to die)
- [x] `PSManager::CreateAutonomousParticleSystem` · 005c0f80 · legions 00588210 — audit: covered: systems are created per source/object FX incl. anchor, water-plane frame and autonomous follow=false path (ParticleAnchor 30-40) [AC2_Rusted/crates/ac2-render/src/scene/particles.rs:1]

## `portal/client/src/render/streak/streakeffect.cpp.c` (7/10)

### StreakEffect
- [x] `StreakEffect::Activate` · 005c13d0 · legions 00588640 — ac2-render streak.rs: StreakEffect state (Activate/Deactivate, SetPosition stamps, Think cut/merge/fade, CreateGeometry ribbon strip) and the manager's create/think/geometry; StreakFX stream decode in ac2-assets ac2_fx.rs (real-data test pending the visual override scan)
- [x] `StreakEffect::Deactivate` · 005c13f0 · legions 00588660 — ac2-render streak.rs: StreakEffect state (Activate/Deactivate, SetPosition stamps, Think cut/merge/fade, CreateGeometry ribbon strip) and the manager's create/think/geometry; StreakFX stream decode in ac2-assets ac2_fx.rs (real-data test pending the visual override scan)
- [x] `StreakEffect::Render` · 005c1410 · legions 00588680 — ac2-render fx_streaks.rs + streak_objects.rs: the emitter streaks start/stop/draw, holding-point children walked (CPhysicsObj::StartStreak recursion), node gate (day/night/player/channel) applied; the ribbon draws as a triangle list through the particle material path

### SmartArray<StreakSegment>
- [-] `SmartArray<StreakSegment>::RemoveOrderedByIndex` · 005c1560 · legions 005887c0 — SmartArray/STL container internals; Vec replaces

### StreakEffect
- [x] `StreakEffect::StreakEffect` · 005c15e0 · legions 00588840 — ac2-render streak.rs: StreakEffect state (Activate/Deactivate, SetPosition stamps, Think cut/merge/fade, CreateGeometry ribbon strip) and the manager's create/think/geometry; StreakFX stream decode in ac2-assets ac2_fx.rs (real-data test pending the visual override scan)
- [-] `StreakEffect::~StreakEffect` · 005c1850 · legions 00588a90 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [x] `StreakEffect::CreateGeometry` · 005c1900 · legions 00588b40 — ac2-render streak.rs: StreakEffect state (Activate/Deactivate, SetPosition stamps, Think cut/merge/fade, CreateGeometry ribbon strip) and the manager's create/think/geometry; StreakFX stream decode in ac2-assets ac2_fx.rs (real-data test pending the visual override scan)
- [x] `StreakEffect::SetPosition` · 005c1ed0 · legions 00589110 — ac2-render streak.rs: StreakEffect state (Activate/Deactivate, SetPosition stamps, Think cut/merge/fade, CreateGeometry ribbon strip) and the manager's create/think/geometry; StreakFX stream decode in ac2-assets ac2_fx.rs (real-data test pending the visual override scan)

### SmartArray<StreakSegment>
- [-] `SmartArray<StreakSegment>::AddToEnd` · 005c1f60 · legions 005891a0 — SmartArray/STL container internals; Vec replaces

### StreakEffect
- [x] `StreakEffect::Think` · 005c1f90 · legions 005891d0 — ac2-render streak.rs: StreakEffect state (Activate/Deactivate, SetPosition stamps, Think cut/merge/fade, CreateGeometry ribbon strip) and the manager's create/think/geometry; StreakFX stream decode in ac2-assets ac2_fx.rs (real-data test pending the visual override scan)

## `portal/client/src/render/streak/streakeffectmanager.cpp.c` (5/6)

### StreakEffectManager
- [-] `StreakEffectManager::~StreakEffectManager` · 005c1040 · legions 005882b0 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [x] `StreakEffectManager::DestroyStreakEffect` · 005c10d0 · legions 00588340 — ac2-render streak.rs StreakEffects: destroy (also from ObjectStreaks::remove_emitter) and default construction
- [x] `StreakEffectManager::UpdateStreakEffects` · 005c1150 · legions 005883c0 — ac2-render streak.rs: StreakEffect state (Activate/Deactivate, SetPosition stamps, Think cut/merge/fade, CreateGeometry ribbon strip) and the manager's create/think/geometry; StreakFX stream decode in ac2-assets ac2_fx.rs (real-data test pending the visual override scan)
- [x] `StreakEffectManager::RenderStreakEffects` · 005c11a0 · legions 00588410 — ac2-render fx_streaks.rs (DoStreakFX, DoTargettedFX's start/stop flags, running-FX guard, pref); ac2-render streak_objects.rs (emitter store, placement, think); drawn in the one particle pass
- [x] `StreakEffectManager::CreateStreakEffect` · 005c1240 · legions 005884b0 — ac2-render streak.rs: StreakEffect state (Activate/Deactivate, SetPosition stamps, Think cut/merge/fade, CreateGeometry ribbon strip) and the manager's create/think/geometry; StreakFX stream decode in ac2-assets ac2_fx.rs (real-data test pending the visual override scan)
- [x] `StreakEffectManager::StreakEffectManager` · 005c12d0 · legions 00588540 — ac2-render streak.rs StreakEffects: destroy (also from ObjectStreaks::remove_emitter) and default construction

## `portal/client/src/scenerender/frustum.cpp.c` (4/7)

### Frustum
- [x] `Frustum::Generate` · 00572cd0 · legions 00542cd0 — ac2-render frustum.rs
- [x] `Frustum::CheckRelationship` · 00572f40 · legions 00542f40 — ac2-render frustum.rs

### CellFrustum
- [-] `CellFrustum::operator=` · 005730b0 · legions 005430b0 — plain data ctor/dtor/operator=; Rust struct or Drop replaces

### Frustum
- [-] `Frustum::Frustum` · 00573110 · legions 00543110 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [x] `Frustum::Generate` · 00573150 · legions 00543150 — Frustum::Generate cited in ac2-render frustum.rs:42 (planes from camera basis, FOVs, near plane)

### CellFrustum
- [-] `CellFrustum::CellFrustum` · 00573390 · legions 00543390 — plain data ctor/dtor/operator=; Rust struct or Drop replaces

### Frustum
- [x] `Frustum::CheckPolygon` · 005733e0 · legions 005433e0 — audit: covered: check_polygon [AC2_Rusted/crates/ac2-render/src/frustum.rs:179]

## `portal/client/src/scenerender/scenecamera.cpp.c` (9/11)

### SceneCamera
- [-] `SceneCamera::~SceneCamera` · 00572820 · legions 0053a560 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [x] `SceneCamera::ApplyProperty` · 00572840 — pregame 3D scenes: ac2-assets ac2_ui_scene.rs, ac2-client pregame_scene/, ac2-ui pregame/scene.rs
- [-] `SceneCamera::BuildOptionalProperties` · 005728f0 — not needed: writes camera props for layout editing (admin tools); DAT read side is ui_scene/data.rs:45 [decompiled-pdb: no callers, no Legions address]
- [x] `SceneCamera::SetFOV` · 00572aa0 · legions 0053a7e0
- [x] `SceneCamera::SetVerticalScale` · 00572ab0 · legions 0053a7f0 — audit: covered: orthographic vertical scale for paper doll/portrait cameras [AC2_Rusted/crates/ac2-render/src/ui_scene/camera.rs:6]
- [x] `SceneCamera::slerp` · 00572ac0 · legions 0053a800
- [x] `SceneCamera::SceneCamera` · 00572b00 · legions 0053a840 — SceneCamera::SceneCamera defaults cited in ac2-client ui_scene/data.rs:51 (perspective, 65, 1, 0.1, 2000)
- [x] `SceneCamera::ApplyPropertyCollection` · 00572bc0 — audit: covered: orthographic, FOV, vertical scale, near/far read from the layout properties [AC2_Rusted/crates/ac2-render/src/ui_scene/data.rs:45]
- [x] `SceneCamera::SetPreviousPosition` · 00572c70 · legions 0053a9b0
- [x] `SceneCamera::SetRenderPosition` · 00572c90 · legions 0053a9d0 — audit: covered: render position used by portal space and UI scene cameras (ui_scene/camera.rs:52) [AC2_Rusted/crates/ac2-render/src/portal/effect.rs:359]
- [x] `SceneCamera::SetCurrentPosition` · 00572cb0 · legions 0053a9f0

## `portal/client/src/scenerender/scenetool.cpp.c` (39/100)

### SceneTool
- [-] `SceneTool::UsingSceneMotionBlur` · 0056a5d0 · legions 0053aa10 — scene motion blur (m_fBlurAmount from CameraFX mBlur): no shipped FX node names a blur CameraFX (blur.cfx 0x55000002 is unreferenced; ac2-assets ac2_fx_tests lists the 22 camera FX nodes), so the blend never runs with the DATs

### MaterialLayer
- [-] `MaterialLayer::SetDiffuse` · 0056a600 · legions 0053aa40 — replaced by design: builders for code-made materials (default, water bump, streak, far terrain) and Vec pushes; DAT materials arrive complete from ac2-assets, code-made ones are WGSL pipelines [AC2_Rusted/crates/ac2-assets/src/ac2_material.rs:69]

### SceneTool
- [-] `SceneTool::StartupDebugHUD` · 0056a630 · legions 0053aa70 — admin-only/tool/debug code, not in the client (debug HUD)
- [-] `SceneTool::ShowDebugHUD` · 0056a7f0 — admin-only/tool/debug code, not in the client (SceneTool::ShowDebugHUD: test notes no handler in Legions) [vtable-checked: not a slot of any located Legions vtable]
- [x] `SceneTool::SetGammaBrightness` · 0056a810 · legions 0053ac30 — ac2-gpu ToneMap::set_brightness: the world's display levels times 1 + 2 * ScreenBrightness (RenderDeviceD3D::SetGamma's ramp), set each frame from SceneOptions::brightness; the UI is left as authored (AGENTS.md)
- [-] `SceneTool::InitFromGlobalPreferences` · 0056a830 · legions 0053ac50 — replaced by design: reads N-patch (key 0x4000001e, not registered, default off), terrain blending (terrain/layers.rs:73), texture targets (replaced by SSR/post chain), tiled-screenshot size ([-] ac2-render.md:809) and applies ScreenBrightness gamma (AGENTS.md:138) [AC2_Rusted/AGENTS.md:138]
- [-] `SceneTool::InitTextureTargets` · 0056a910 · legions 0053ad30 — D3D device/resource plumbing; wgpu replaces
- [-] `SceneTool::SetWorldCellID` · 0056aac0 · legions 0053aee0 — modern pass: world-offset bookkeeping replaced by eye-relative drawing in f64 [AC2_Rusted/AGENTS.md:721]

### (free functions)
- [-] `SwapWorldMatrixForwardAndUp` · 0056ab00 · legions 0053af20 — replaced by design: AC axes (z up) go straight into the wgpu camera; no D3D forward/up swap needed [AC2_Rusted/crates/ac2-render/src/view.rs:2]
- [-] `SwapViewMatrixForwardAndUp` · 0056ab90 · legions 0053afb0 — replaced by design: AC axes (z up) go straight into the wgpu camera; no D3D forward/up swap needed [AC2_Rusted/crates/ac2-render/src/view.rs:2]

### SceneTool
- [-] `SceneTool::SetWorldMatrix` · 0056ac20 · legions 0053b040 — D3D device/resource plumbing; wgpu replaces
- [-] `SceneTool::SetWorldTransform` · 0056ac70 · legions 0053b090 — D3D device/resource plumbing; wgpu replaces
- [x] `SceneTool::SetMatrixStack` · 0056ad20 · legions 0053b140 — ac2-gpu ObjectInstance.pose (part matrices, bone storage buffer)
- [x] `SceneTool::TransformVector` · 0056ad40 · legions 0053b160 — SceneTool::TransformVector cited in ac2-client view.rs:443 (box centre projected through the view)
- [x] `SceneTool::TransformPixelToClipPoint` · 0056af60 · legions 0053b380 — SceneTool::TransformPixelToClipPoint cited in ac2-ui window_locations.rs:5 (pixel_to_clip)
- [x] `SceneTool::TransformClipPointToPixel` · 0056afb0 · legions 0053b3d0 — ac2-ui reticle.rs + ac2-gpu ui.rs: the targeting reticle
- [x] `SceneTool::ViewportToClip` · 0056b010 · legions 0053b430 — audit: covered: only caller is StartSelection (pointer -> selection point) [AC2_Rusted/crates/ac2-render/src/selection.rs:1]
- [-] `SceneTool::ClipToViewport` · 0056b080 · legions 0053b4a0 — modern pass: only caller LensFlareEffect::Draw; flare placement is on the GPU (ac2-gpu lens_flare.rs) [AC2_Rusted/AGENTS.md:162]

### LightSorterPredicate
- [x] `LightSorterPredicate::operator()` · 0056b100 · legions 0053b520 — ac2-render object::point_lights object_point_lights_in and scene object_lit_by: candidates sorted heaviest first by GetWeight

### SceneTool
- [x] `SceneTool::SetAmbientLight` · 0056b150 · legions 0053b570 — ac2-render object::pass / texture_matrix, ac2-gpu object
- [x] `SceneTool::SetOutdoorAmbient` · 0056b170 · legions 0053b590 — ac2-render object::pass / texture_matrix, ac2-gpu object
- [x] `SceneTool::SetFogProperties` · 0056b1a0 · legions 0053b5c0 — SceneTool::SetFogProperties cited in ac2-client ui_scene/data.rs:92 (want_fog, colour, near, far applied in camera.rs)
- [x] `SceneTool::IlluminateMeshVertices` · 0056b1c0 · legions 0053b5e0
- [-] `SceneTool::PrepareGenericRenderTarget0` · 0056b1e0 · legions 0053b600 — D3D device/resource plumbing; wgpu replaces
- [-] `SceneTool::PrepareGenericRenderTarget1` · 0056b250 · legions 0053b670 — D3D device/resource plumbing; wgpu replaces
- [-] `SceneTool::PrepareGenericRenderTarget2` · 0056b2c0 · legions 0053b6e0 — D3D device/resource plumbing; wgpu replaces
- [-] `SceneTool::PrepareGenericDepthTarget` · 0056b330 · legions 0053b750 — D3D device/resource plumbing; wgpu replaces
- [-] `SceneTool::GenerateGlobalShadowVolume` · 0056b390 · legions 0053b7b0 — modern pass: stencil volumes / disc decals replaced by cascaded sun depth maps from every level above 0 (ac2-gpu sun_shadow.rs, object_sun_casters.rs) [AC2_Rusted/AGENTS.md:141]
- [-] `SceneTool::StartScreenshotCapture` · 0056b6b0 · legions 0053ba90 — tiled re-render capture not ported; the presented frame is copied instead
- [-] `SceneTool::FinishScreenshotCapture` · 0056b7a0 · legions 0053bb80 — tiled re-render capture not ported; the presented frame is copied instead
- [-] `SceneTool::GetNumSubScreenshots` · 0056b7c0 · legions 0053bba0 — tiled re-render capture not ported; the presented frame is copied instead
- [-] `SceneTool::FinishSubScreenshot` · 0056b7d0 · legions 0053bbb0 — tiled re-render capture not ported; the presented frame is copied instead
- [-] `SceneTool::StartReflectionMap` · 0056ba00 · legions 0053bde0 — modern pass: reflection/bump render targets replaced by the screen-space water reflection with procedural ripples (ac2-gpu water_reflection.rs) [AC2_Rusted/AGENTS.md:174]
- [-] `SceneTool::FinishReflectionMap` · 0056ba60 · legions 0053be40 — modern pass: reflection/bump render targets replaced by the screen-space water reflection with procedural ripples (ac2-gpu water_reflection.rs) [AC2_Rusted/AGENTS.md:174]
- [-] `SceneTool::StartWaterBumpMap` · 0056baa0 · legions 0053be80 — modern pass: reflection/bump render targets replaced by the screen-space water reflection with procedural ripples (ac2-gpu water_reflection.rs) [AC2_Rusted/AGENTS.md:174]
- [-] `SceneTool::FinishWaterBumpMap` · 0056bb20 · legions 0053bf00 — modern pass: reflection/bump render targets replaced by the screen-space water reflection with procedural ripples (ac2-gpu water_reflection.rs) [AC2_Rusted/AGENTS.md:174]
- [-] `SceneTool::IdentityMatrices` · 0056bb50 · legions 0053bf30 — D3D device/resource plumbing; wgpu replaces
- [-] `SceneTool::StoreMatrices` · 0056bc90 · legions 0053c070 — D3D device/resource plumbing; wgpu replaces
- [-] `SceneTool::RestoreMatrices` · 0056bcd0 · legions 0053c0b0 — D3D device/resource plumbing; wgpu replaces
- [-] `SceneTool::SetBackgroundColor` · 0056bd90 · legions 0053c170 — replaced by design: only caller sets black every frame; wgpu clears the HDR target [AC2_Rusted/crates/ac2-gpu/src/frame.rs]
- [-] `SceneTool::StartFrame` · 0056bdd0 · legions 0053c1b0 — replaced by design: D3D BeginScene/EndScene, texture targets and scene timestamps; wgpu frame replaces (pixel scan = AGENTS.md:162) [AC2_Rusted/crates/ac2-gpu/src/frame.rs]
- [x] `SceneTool::CheckObjectSelection` · 0056be60 · legions 0053c240 — ac2-render selection.rs + ac2-ui selection_ui.rs: clicking the world selects
- [-] `SceneTool::ShouldSkipRenderNow` · 0056bf90 · legions 0053c370 — replaced by design: sceneTimestamp de-dupe of objects reached through several cells; the scene gathers each object once per pass [AC2_Rusted/crates/ac2-render/src/scene/mod.rs:840]
- [-] `SceneTool::SetSelectionID` · 0056bfb0 · legions 0053c390 — not needed: m_selectionID/m_selectionTime are written but never read anywhere [decompiled-pdb scenetool.cpp.c:1302]
- [-] `SceneTool::GetDistortionMap` · 0056bfe0 · legions 0053c3c0 — no shipped material names the Distortion special texture or a BUMPENVMAP stage (Legions DAT: 0 of 2401 materials); one samples the frame buffer (0x31000323)
- [-] `SceneTool::GetFrameBufferTexture` · 0056c270 · legions 0053c640 — D3D device/resource plumbing; wgpu replaces
- [-] `SceneTool::PurgeOldGraphicsResources` · 0056c290 · legions 0053c660 — D3D device lifecycle/restore; wgpu replaces

### BBox
- [-] `BBox::MakeValid` · 0056c570 · legions 0053c950 — not needed: StartSelection normalises its clip-space selection rectangle (y flips in ViewportToClip); the port picks with the cursor point (ac2-render selection.rs check_object_selection), which has no corner order to fix

### SceneTool
- [x] `SceneTool::SetupView` · 0056c5d0 · legions 0053c9b0 — SceneTool::SetupView cited in ac2-client ui_scene/camera.rs:4 (view from entity frame along +Y)
- [x] `SceneTool::SetupPerspectiveProjection` · 0056c730 · legions 0053cb10 — audit: covered: Camera::view_projection / ui_scene/camera.rs:52 [AC2_Rusted/crates/ac2-render/src/view.rs:20]
- [-] `SceneTool::SetupPerpspectiveFromFrustum` · 0056c880 · legions 0053cc60 — replaced by design: only used by tiled screenshot capture (m_bCapturingScreenshot), which is not ported [docs/retail-catalog/ac2-render.md:809]
- [x] `SceneTool::SetupOrthographicProjection` · 0056c980 · legions 0053cd60 — audit: covered: orthographic camera spanning the vertical scale [AC2_Rusted/crates/ac2-render/src/ui_scene/camera.rs:178]
- [x] `SceneTool::SetupOrthographicProjection` · 0056ca70 · legions 0053ce50 — audit: covered: min/max box ortho projection of StartShadowMap (landscape static shadow map) [AC2_Rusted/crates/ac2-gpu/src/shadow_map.rs:3]
- [x] `SceneTool::InverseRotate` · 0056cb90 · legions 0053cf70 — audit: covered: only caller StartSelection; selection ray is mapped into model space (GetLocalSelectionRay) [AC2_Rusted/crates/ac2-render/src/selection.rs:1]
- [x] `SceneTool::InverseTransform` · 0056cd40 · legions 0053d120 — audit: covered: only caller StartSelection; selection ray is mapped into model space (GetLocalSelectionRay) [AC2_Rusted/crates/ac2-render/src/selection.rs:1]
- [x] `SceneTool::GetLight` · 0056cee0 · legions 0053d2c0 — No list of lights to index or unlink: each resident block owns its static lights (BlockInfo.scene_lights, entity lights in ObjectLights) and drops them when the block leaves (WorldScene remove path)
- [x] `SceneTool::UnlinkAllLights` · 0056cf20 · legions 0053d300 — No list of lights to index or unlink: each resident block owns its static lights (BlockInfo.scene_lights, entity lights in ObjectLights) and drops them when the block leaves (WorldScene remove path)
- [-] `SceneTool::DrawLine` · 0056d000 · legions 0053d3e0 — not needed: only real caller is UIRectangleObject::Render (UI element debug outline; [-] in ac2-ui.md:2305) [decompiled-pdb uirectangleobject.cpp.c:77-98]
- [-] `SceneTool::DrawTexturedTriangle` · 0056d0b0 · legions 0053d490 — modern pass: only LensFlareEffect::Draw (scan triangle) and IME; flare scan replaced by GPU depth test [AC2_Rusted/AGENTS.md:162]
- [-] `SceneTool::DrawTriangle` · 0056d1e0 · legions 0053d5c0 — modern pass: only LensFlareEffect::Draw (scan triangle) and IME; flare scan replaced by GPU depth test [AC2_Rusted/AGENTS.md:162]
- [-] `SceneTool::DrawTexturedPolygon` · 0056d240 · legions 0053d620 — modern pass: flare sprites drawn by lens_flare pass (AGENTS.md:162) [AC2_Rusted/crates/ac2-gpu/src/lens_flare.rs:186]
- [-] `SceneTool::BuildCylinderMesh` · 0056d2b0 · legions 0053d690 — replaced by design: sphere/cylinder meshes are caps for stencil shadow volumes; replaced by sun cascades [AC2_Rusted/AGENTS.md:141]
- [-] `SceneTool::BuildSphereMesh` · 0056dc60 · legions 0053e040 — replaced by design: sphere/cylinder meshes are caps for stencil shadow volumes; replaced by sun cascades [AC2_Rusted/AGENTS.md:141]
- [x] `SceneTool::StartSelection` · 0056e450 · legions 0053e830 — audit: covered: pick by projected box then ray trace through parts, nearest first, misses stay potential; selectability/LookAtSelection live with the caller (gmSelectionUI) [AC2_Rusted/crates/ac2-render/src/selection.rs:1]
- [-] `SceneTool::DrawScreenQuadUsingMaterial` · 0056e5f0 · legions 0053e9d0 — modern pass: callers all have equivalents: flash (tonemap.rs:450), sky fog quad, FinishShadowMap (shadow_map.rs:197), volumetric shadows (AGENTS.md:141), water (AGENTS.md:174) [AC2_Rusted/crates/ac2-gpu/src/screen_quad.rs:1]
- [x] `SceneTool::Think` · 0056e860 · legions 0053ec40 — audit: covered: DecalEffectManager::Think; PurgeOldGraphicsResources is D3D plumbing [AC2_Rusted/crates/ac2-render/src/decal.rs:362]
- [x] `SceneTool::GetPixelScanResult` · 0056e880 · legions 0053ec60 — only LensFlareEffect::Draw scanned pixels; replaced by ac2-gpu lens_flare.rs's depth scan pass (AGENTS.md exception)
- [x] `SceneTool::SetupCamera` · 0056ea20 · legions 0053ee00 — SceneTool::SetupCamera cited in ac2-client ui_scene/camera.rs:3 (viewport renders through camera entity)
- [-] `SceneTool::GetPointLightShadowVector` · 0056ed50 · legions 0053f130 — modern pass: point/sun/FX lights done (object_lit_by); per-object shadow direction replaced by the global sun cascades, one direction for all [AC2_Rusted/AGENTS.md:141]
- [x] `SceneTool::DisableSunlight` · 0056eef0 · legions 0053f2d0 — audit: covered: portal space and UI scenes draw without the sun (ui_scene/camera.rs:25) [AC2_Rusted/crates/ac2-gpu/src/portal_space.rs:12]
- [x] `SceneTool::SetSunlight` · 0056ef40 · legions 0053f320 — object::ObjectLights; static shadow strength SceneLighting::shadow_strength drives the landscape map and the sun shadow cascades (ac2_gpu sun_shadow.rs); m_cVolumetricShadowStrength's undimmed colour is not used (cascades take the static strength)
- [-] `SceneTool::DrawLine` · 0056f230 · legions 0053f610 — not needed: only real caller is UIRectangleObject::Render (UI element debug outline; [-] in ac2-ui.md:2305) [decompiled-pdb uirectangleobject.cpp.c:77-98]
- [x] `SceneTool::DrawScreenQuad` · 0056f260 · legions 0053f640 — audit: covered: untextured blended quad; textured callers are texture-target presents and reflection blends (replaced). Interior fog-cull quad is tracked under WorldRender::Render [AC2_Rusted/crates/ac2-gpu/src/screen_quad.rs:1]
- [-] `SceneTool::SwitchTextureTargets` · 0056f2e0 · legions 0053f6c0 — scene motion blur (m_fBlurAmount from CameraFX mBlur): no shipped FX node names a blur CameraFX (blur.cfx 0x55000002 is unreferenced; ac2-assets ac2_fx_tests lists the 22 camera FX nodes), so the blend never runs with the DATs
- [x] `SceneTool::Render` · 0056f440 · legions 0053f820 — audit: covered: streaks (ac2-render streak.rs:279) and the camera flash (set_flash) both present [AC2_Rusted/crates/ac2-gpu/src/tonemap.rs:450]

### SmartArray<ObjectSelection>
- [-] `SmartArray<ObjectSelection>::AddToEnd` · 0056f4f0 · legions 0053f930 — SmartArray/STL container internals; Vec replaces
- [-] `SmartArray<ObjectSelection>::InsertOrderedByIndex` · 0056f560 · legions 0053f9a0 — SmartArray/STL container internals; Vec replaces

### SceneTool
- [-] `SceneTool::UpdateDebugHUD` · 0056f770 · legions 0053fbb0 — admin-only/tool/debug code, not in the client (debug HUD)
- [-] `SceneTool::RefreshGraphicsResources` · 0056fb70 — D3D device lifecycle/restore; wgpu replaces
- [-] `SceneTool::Shutdown` · 0056fc00 · legions 00540040 — D3D device lifecycle/restore; wgpu replaces
- [x] `SceneTool::SetupCamera` · 0056fe60 · legions 005402a0 — SceneTool::SetupCamera cited in ac2-client ui_scene/camera.rs:3 (viewport renders through camera entity)
- [x] `SceneTool::StartShadowMap` · 0056fea0 · legions 005402e0 — ac2_gpu::StaticShadowMap (shadow_map.rs, object_shadow.rs); redraws on block/caster/sun>1deg/strength change
- [x] `SceneTool::FinishShadowMap` · 00570300 · legions 00540740 — ac2_gpu::StaticShadowMap (shadow_map.rs, object_shadow.rs); redraws on block/caster/sun>1deg/strength change
- [-] `SceneTool::RenderGlobalVolumetricShadows` · 005703e0 · legions 00540820 — modern pass: stencil volumes / disc decals replaced by cascaded sun depth maps from every level above 0 (ac2-gpu sun_shadow.rs, object_sun_casters.rs) [AC2_Rusted/AGENTS.md:141]
- [x] `SceneTool::FinishSelection` · 005706d0 · legions 00540b10 — audit: covered: pick by projected box then ray trace through parts, nearest first, misses stay potential; selectability/LookAtSelection live with the caller (gmSelectionUI) [AC2_Rusted/crates/ac2-render/src/selection.rs:1]
- [x] `SceneTool::SelectObject` · 00570730 · legions 00540b70 — audit: covered: pick by projected box then ray trace through parts, nearest first, misses stay potential; selectability/LookAtSelection live with the caller (gmSelectionUI) [AC2_Rusted/crates/ac2-render/src/selection.rs:1]

### SmartArray<PixelScanRequest>
- [-] `SmartArray<PixelScanRequest>::AddToEnd` · 005708c0 · legions 00540d00 — SmartArray/STL container internals; Vec replaces

### MaterialLayer
- [-] `MaterialLayer::InsertStage` · 00570970 · legions 00540e20 — replaced by design: builders for code-made materials (default, water bump, streak, far terrain) and Vec pushes; DAT materials arrive complete from ac2-assets, code-made ones are WGSL pipelines [AC2_Rusted/crates/ac2-assets/src/ac2_material.rs:69]

### RenderMaterial
- [-] `RenderMaterial::InsertLayer` · 00570990 · legions 00540e40 — replaced by design: builders for code-made materials (default, water bump, streak, far terrain) and Vec pushes; DAT materials arrive complete from ac2-assets, code-made ones are WGSL pipelines [AC2_Rusted/crates/ac2-assets/src/ac2_material.rs:69]

### SceneTool
- [-] `SceneTool::CreateDefaultMaterial` · 005709b0 · legions 00540e60 — not needed: m_pDefaultMaterial is created and released but never used for drawing [decompiled-pdb scenetool.cpp.c:3906,4518]
- [-] `SceneTool::Startup` · 00570f00 · legions 005413b0 — D3D device lifecycle/restore; wgpu replaces
- [x] `SceneTool::LinkLight` · 00571630 · legions 00541ae0 — SceneTool::LinkLight cited in ac2-client ui_scene/camera.rs:13 (scene lights linked, sun disabled)
- [x] `SceneTool::AddDynamicLights` · 005716c0 · legions 00541b70 — ac2-render scene/object_lights.rs point_lights: FX and landblock-entity lights on the focus side (light cell vs object cell), in range by RefreshMaxRange (Waveform::upper_bound), weight-sorted
- [x] `SceneTool::RequestPixelScan` · 00571860 · legions 00541d10 — only LensFlareEffect::Draw scanned pixels; replaced by ac2-gpu lens_flare.rs's depth scan pass (AGENTS.md exception)
- [-] `SceneTool::ScanRequestedPixels` · 005718c0 · legions 00541d70 — modern pass: pixel scan replaced by the GPU 5x5 depth patch (lens_flare.rs) [AC2_Rusted/AGENTS.md:162]
- [-] `SceneTool::EndFrame` · 00571a40 · legions 00541ef0 — replaced by design: D3D BeginScene/EndScene, texture targets and scene timestamps; wgpu frame replaces (pixel scan = AGENTS.md:162) [AC2_Rusted/crates/ac2-gpu/src/frame.rs]
- [x] `SceneTool::SortLights` · 00571da0 · legions 00542250 — ac2-render object::cell / point_lights
- [x] `SceneTool::CacheStaticLights` · 00571ec0 · legions 00542370 — object_point_lights_in: weight sort, six cap, range test by max range, indoor/outdoor side, sunlight-type lights follow the sun inside; the DATs hold no outdoor static light (every CLightInfo light is an env cell light); the 2 directional lights of the DATs are left to the sun
- [-] `SceneTool::ApplyLights` · 00572110 · legions 005425c0 — modern pass: point/sun/FX lights done (object_lit_by); per-object shadow direction replaced by the global sun cascades, one direction for all [AC2_Rusted/AGENTS.md:141]
- [x] `SceneTool::GetLightsForPosition` · 005726b0 · legions 00542b60 — ac2-render WorldScene::object_lit_by: static pick plus dynamic lights sorted together, at most four; the sunlight term is the scene sun (ClientView)

## `portal/client/src/smartbox/worldrender/cellrender.cpp.c` (1/2)

### CellRender
- [-] `CellRender::CellRender` · 00583480 · legions 00552bf0 — replaced by design: constructors; the scene owns this state [AC2_Rusted/crates/ac2-render/src/scene/mod.rs]
- [x] `CellRender::AddCellSounds` · 005834d0 · legions 00552c40 — audit: covered: visible interior cells feed ambient sounds [AC2_Rusted/crates/ac2-audio/src/ambient_driver.rs:119]

## `portal/client/src/smartbox/worldrender/frillrender.cpp.c` (4/24)

### FrillRender
- [-] `FrillRender::OnContextFinished` · 00580990 — D3D device lifecycle/restore; wgpu replaces

### FrillEntry
- [-] `FrillEntry::operator=` · 005809e0 · legions 00550290 — plain data ctor/dtor/operator=; Rust struct or Drop replaces

### FrillRender
- [-] `FrillRender::DestroyFrillMesh` · 00580c30 · legions 00550430 — retail DB object cache handle/release; Arc per DID replaces (AGENTS.md exception)
- [-] `FrillRender::ClearFrillMesh` · 00580cb0 · legions 005504b0 — replaced by design: retail's frill object lists / merged-mesh bookkeeping are replaced by select_frills + instanced grass (ac2-render frill.rs, scene/grass.rs) [AC2_Rusted/AGENTS.md:593]
- [-] `FrillRender::InitFrillMesh` · 005810b0 · legions 005508b0 — replaced by design: retail's frill object lists / merged-mesh bookkeeping are replaced by select_frills + instanced grass (ac2-render frill.rs, scene/grass.rs) [AC2_Rusted/AGENTS.md:593]
- [-] `FrillRender::RenderFrillMesh` · 00581180 · legions 00550980 — modern pass: frills drawn as instances with wind/dither; the 12-20 m fade is documented as not drawn [AC2_Rusted/AGENTS.md:593]

### FrillListType
- [-] `FrillListType::~FrillListType` · 00581800 · legions 00551000 — plain data ctor/dtor/operator=; Rust struct or Drop replaces

### HashIterator<unsigned_long,BaseList<unsigned_long,11>_>
- [-] `HashIterator<unsigned_long,BaseList<unsigned_long,11>_>::next` · 00581840 · legions 00536680 — MSVC/Windows assertion or iterator internals

### SmartArray<RGBAColor>
- [-] `SmartArray<RGBAColor>::AddToEnd` · 00581880 · legions 00551040 — SmartArray/STL container internals; Vec replaces

### FrillRender
- [-] `FrillRender::GetSubDataIDs` · 005818e0 · legions 005510a0 — replaced by design: retail's frill object lists / merged-mesh bookkeeping are replaced by select_frills + instanced grass (ac2-render frill.rs, scene/grass.rs) [AC2_Rusted/AGENTS.md:593]
- [-] `FrillRender::ClearAllRenderLists` · 00581990 · legions 00551150 — replaced by design: retail's frill object lists / merged-mesh bookkeeping are replaced by select_frills + instanced grass (ac2-render frill.rs, scene/grass.rs) [AC2_Rusted/AGENTS.md:593]

### FrillRenderType
- [x] `FrillRenderType::AddPos` · 00581a00 · legions 005511c0 — covered: frill.rs FrillCandidate::world (frame with scale) and the frill landscape shadow (terrain/shadow.rs self term + landscape_shadow map term for CalculateShadowColor); doc names it

### SmartArray<FrillEntry>
- [-] `SmartArray<FrillEntry>::AddToEnd` · 00581be0 · legions 005513a0 — SmartArray/STL container internals; Vec replaces

### FrillRender
- [-] `FrillRender::Clear` · 00581d00 · legions 005514c0 — replaced by design: retail's frill object lists / merged-mesh bookkeeping are replaced by select_frills + instanced grass (ac2-render frill.rs, scene/grass.rs) [AC2_Rusted/AGENTS.md:593]
- [-] `FrillRender::RemoveOldFrillObjects` · 00581e90 · legions 00551650 — replaced by design: retail's frill object lists / merged-mesh bookkeeping are replaced by select_frills + instanced grass (ac2-render frill.rs, scene/grass.rs) [AC2_Rusted/AGENTS.md:593]

### FrillListType
- [-] `FrillListType::Add` · 00582010 · legions 005517d0 — replaced by design: retail's frill object lists / merged-mesh bookkeeping are replaced by select_frills + instanced grass (ac2-render frill.rs, scene/grass.rs) [AC2_Rusted/AGENTS.md:593]

### FrillRender
- [-] `FrillRender::~FrillRender` · 00582170 · legions 00551930 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [-] `FrillRender::EnableFrillObjects` · 00582220 · legions 005519d0 — replaced by design: retail's frill object lists / merged-mesh bookkeeping are replaced by select_frills + instanced grass (ac2-render frill.rs, scene/grass.rs) [AC2_Rusted/AGENTS.md:593]
- [x] `FrillRender::AddNewFrillObjects` · 005823e0 · legions 00551b90 — audit: covered: select_frills picks frills in (radius+1) blocks around the camera cell; far frills added by select_far_frills (AGENTS.md:593) [AC2_Rusted/crates/ac2-render/src/frill.rs]
- [-] `FrillRender::FrillRender` · 00582670 · legions 00551e20 — replaced by design: retail's frill object lists / merged-mesh bookkeeping are replaced by select_frills + instanced grass (ac2-render frill.rs, scene/grass.rs) [AC2_Rusted/AGENTS.md:593]
- [-] `FrillRender::UpdateFrillPosition` · 00582760 · legions 00551f00 — replaced by design: retail's frill object lists / merged-mesh bookkeeping are replaced by select_frills + instanced grass (ac2-render frill.rs, scene/grass.rs) [AC2_Rusted/AGENTS.md:593]
- [-] `FrillRender::Render` · 00582890 · legions 00552030 — replaced by design: retail's frill object lists / merged-mesh bookkeeping are replaced by select_frills + instanced grass (ac2-render frill.rs, scene/grass.rs) [AC2_Rusted/AGENTS.md:593]
- [x] `FrillRender::RefreshFrillMesh` · 005828e0 · legions 00552050 — audit: covered: select_frills picks frills in (radius+1) blocks around the camera cell; far frills added by select_far_frills (AGENTS.md:593) [AC2_Rusted/crates/ac2-render/src/frill.rs]
- [x] `FrillRender::UpdateRenderList` · 005832a0 · legions 00552a10 — audit: covered: select_frills picks frills in (radius+1) blocks around the camera cell; far frills added by select_far_frills (AGENTS.md:593) [AC2_Rusted/crates/ac2-render/src/frill.rs]

## `portal/client/src/smartbox/worldrender/lscapelighting.cpp.c` (15/19)

### RGBAColor
- [x] `RGBAColor::Normalize` · 0058f080 · legions 0055e6f0 — audit: covered: CalcDayLight normalises by the largest channel (IsValid/Normalize) [AC2_Rusted/crates/ac2-render/src/landscape_lighting.rs:178]
- [-] `RGBAColor::IsValid` · 0058f130 · legions 0055e7a0 — debug validity check; no visible behaviour

### LScapeLighting
- [x] `LScapeLighting::SetDayLighting` · 0058f1d0 · legions 0055e840
- [x] `LScapeLighting::SetDayFog` · 0058f210 — ac2-render WorldScene::lighting_at: the viewer's block picks the fog corners each frame (no hysteresis cache); the day's fog comes from SceneLighting::from_sky
- [x] `LScapeLighting::CornerInterp` · 0058f230 · legions 0055e8a0 — ac2-render landscape_lighting.rs (with_region_fog): each fog-area corner over the day's light and fog, blended by the viewer's place in the block
- [x] `LScapeLighting::Combine` · 0058f890 · legions 0055ef00 — ac2-render landscape_lighting.rs (with_region_fog): each fog-area corner over the day's light and fog, blended by the viewer's place in the block
- [x] `LScapeLighting::Combine` · 0058f910 · legions 0055ef80 — ac2-render landscape_lighting.rs (with_region_fog): each fog-area corner over the day's light and fog, blended by the viewer's place in the block
- [x] `LScapeLighting::CalcDayLight` · 0058f9a0 · legions 0055f010 — ac2-render landscape_lighting.rs (with_region_fog): each fog-area corner over the day's light and fog, blended by the viewer's place in the block
- [x] `LScapeLighting::CalcCombineFog` · 0058fac0 · legions 0055f130 — ac2-render landscape_lighting.rs (with_region_fog): each fog-area corner over the day's light and fog, blended by the viewer's place in the block
- [x] `LScapeLighting::CalcWaterLight` · 0058fc10 · legions 0055f280 — ac2-render landscape_lighting.rs update_lighting: day light and fog, the fog areas at the viewer's block corners (CalcCombineFog, CornerInterp), then under the block's water its depth-keyed fog (CalcWaterLight; the search keeps retail's last match); water layers from ac2-world Ac2WorldSource::block_water_lighting

### DBObjGrabber<CSoundDesc>
- [-] `DBObjGrabber<CSoundDesc>::~DBObjGrabber<CSoundDesc>` · 00590150 · legions 00478460 — retail DB object cache handle/release; Arc per DID replaces (AGENTS.md exception)

### LScapeLighting
- [x] `LScapeLighting::UpdateLighting` · 005901f0 · legions 0055f840 — ac2-render landscape_lighting.rs update_lighting: day light and fog, the fog areas at the viewer's block corners (CalcCombineFog, CornerInterp), then under the block's water its depth-keyed fog (CalcWaterLight; the search keeps retail's last match); water layers from ac2-world Ac2WorldSource::block_water_lighting
- [x] `LScapeLighting::GetFogCorners` · 00590670 · legions 0055fcc0 — ac2-world region/block_fog.rs RegionFog::corners: the current CFogDesc's fog type map at the block's four corners
- [x] `LScapeLighting::GetWaterInfo` · 00590840 · legions 0055fe90 — ac2-render landscape_lighting.rs update_lighting: day light and fog, the fog areas at the viewer's block corners (CalcCombineFog, CornerInterp), then under the block's water its depth-keyed fog (CalcWaterLight; the search keeps retail's last match); water layers from ac2-world Ac2WorldSource::block_water_lighting

### SmartArray<WaterLightInfo>
- [-] `SmartArray<WaterLightInfo>::~SmartArray<WaterLightInfo>` · 00590900 · legions 0055ff50 — SmartArray/STL container internals; Vec replaces

### LScapeLighting
- [x] `LScapeLighting::LScapeLighting` · 00590940 · legions 0055ff90 — ac2-render WorldScene::lighting_at: the viewer's block picks the fog corners each frame (no hysteresis cache); the day's fog comes from SceneLighting::from_sky
- [-] `LScapeLighting::~LScapeLighting` · 00590b10 · legions 00560160 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [x] `LScapeLighting::UpdateInfo` · 00590b60 · legions 005601b0 — ac2-render WorldScene::lighting_at: the viewer's block picks the fog corners each frame (no hysteresis cache); the day's fog comes from SceneLighting::from_sky
- [x] `LScapeLighting::ChangeCell` · 00590bc0 · legions 00560210 — ac2-render WorldScene::lighting_at: the viewer's block picks the fog corners each frame (no hysteresis cache); the day's fog comes from SceneLighting::from_sky

## `portal/client/src/smartbox/worldrender/lscapematerialinfo.cpp.c` (13/14)

### RenderTextureLocal
- [x] `RenderTextureLocal::GetGlobalOrLocal` · 00590c10 · legions 00560260 — audit: covered: portal vs local DAT texture by DID type [AC2_Rusted/crates/ac2-assets/src/ac2_texture.rs:318]

### LScapeMaterialInfo
- [-] `LScapeMaterialInfo::~LScapeMaterialInfo` · 00590c60 · legions 005602b0 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [x] `LScapeMaterialInfo::CornerMatch` · 00590c80 · legions 005602d0
- [x] `LScapeMaterialInfo::GetCombineMaterials` · 00590cb0 · legions 00560300 — base, alpha and detail materials; the detail one, unused by CombineMaterials, is drawn near the eye (AGENTS.md)
- [x] `LScapeMaterialInfo::GetAlphaTexture` · 00590d80 · legions 005603d0
- [x] `LScapeMaterialInfo::CalcUVInfo` · 00590de0 · legions 00560410
- [x] `LScapeMaterialInfo::Match` · 00590ea0 · legions 005604d0
- [x] `LScapeMaterialInfo::CalcRotUVs` · 00591020 · legions 00560650
- [x] `LScapeMaterialInfo::LScapeMaterialInfo` · 005910d0 · legions 00560700 — audit: covered: terrain layer/material set (CornerMatch, detail AGENTS.md:575) [AC2_Rusted/crates/ac2-render/src/terrain/layers.rs:73]
- [x] `LScapeMaterialInfo::SetUVs` · 005911e0 · legions 00560810
- [x] `LScapeMaterialInfo::Apply` · 00591270 · legions 005608a0
- [x] `LScapeMaterialInfo::CombineMaterials` · 005912d0 · legions 00560900 — terrain.wgsl: base/alpha stages, shadow-map and cloud-shadow modulate before fog
- [x] `LScapeMaterialInfo::MultiCombine` · 005919d0 · legions 00561000
- [x] `LScapeMaterialInfo::Create` · 00591a40 · legions 00561070

## `portal/client/src/smartbox/worldrender/lscaperender.cpp.c` (19/39)

### LScapeLighting
- [x] `LScapeLighting::GetAmbientLight` · 005835c0 · legions 00552d30

### LScapeRender
- [-] `LScapeRender::RefreshGraphicsResources` · 00583630 · legions 00552da0 — D3D device lifecycle/restore; wgpu replaces
- [-] `LScapeRender::ReinitializeTerrain` · 00583650 · legions 00552dc0 — replaced by design: terrain mesh teardown/rebuild on draw-distance or region change; scene options re-applied (set_scene_options) and meshes rebuilt from resident blocks [AC2_Rusted/crates/ac2-client/src/view.rs:304]
- [-] `LScapeRender::ResetWorldState` · 005836b0 · legions 00552e20 — replaced by design: terrain mesh teardown/rebuild on draw-distance or region change; scene options re-applied (set_scene_options) and meshes rebuilt from resident blocks [AC2_Rusted/crates/ac2-client/src/view.rs:304]
- [x] `LScapeRender::IsCameraInWater` · 005836d0 · legions 00552e40 — audit: covered: viewer under its block's water plane picks the water fog/light [AC2_Rusted/crates/ac2-render/src/scene/lighting.rs:43]
- [x] `LScapeRender::DrawRain` · 00583700 · legions 00552e70 — rain amount and wind; streaks drawn as the rain material's particle batch above 0.4
- [-] `LScapeRender::DrawTerrain` · 00583850 — replaced by design: forwarders to LScapeTerrainMesh::DrawTerrain/DrawWater (see those rows) [AC2_Rusted/crates/ac2-gpu/src/scene.rs:2]
- [-] `LScapeRender::DrawWater` · 00583860 — replaced by design: forwarders to LScapeTerrainMesh::DrawTerrain/DrawWater (see those rows) [AC2_Rusted/crates/ac2-gpu/src/scene.rs:2]
- [x] `LScapeRender::SetDayLighting` · 00583870 · legions 00552fe0 — covered: forwarder to LScapeLighting::SetDayLighting; landscape_lighting.rs Final::from_sky reads the day values each frame (doc names it)
- [x] `LScapeRender::SetDayFog` · 00583880 — covered: forwarder to LScapeLighting::SetDayFog; landscape_lighting.rs Final::from_sky (doc names it)
- [x] `LScapeRender::SetDefaultLighting` · 00583890 · legions 00553000
- [-] `LScapeRender::ReloadLandBlocks` · 00583920 · legions 00553090 — replaced by design: draw-list caches and dirty flags; lighting recomputed per frame, blocks drawn depth-tested and eye-relative (AGENTS.md:721), cells released in ac2-smartbox cells.rs:156 [AC2_Rusted/crates/ac2-render/src/scene/mod.rs:840]
- [-] `LScapeRender::SetVertexValue` · 00583950 · legions 005530c0 — not needed: terrain vertex edit for the world builder/admin; the client never edits terrain [decompiled-pdb: no callers (virtual) ; AGENTS 'no admin tools']
- [-] `LScapeRender::Refresh` · 005839b0 · legions 00553120 — replaced by design: draw-list caches and dirty flags; lighting recomputed per frame, blocks drawn depth-tested and eye-relative (AGENTS.md:721), cells released in ac2-smartbox cells.rs:156 [AC2_Rusted/crates/ac2-render/src/scene/mod.rs:840]
- [-] `LScapeRender::DirtyLighting` · 005839e0 · legions 00553150 — replaced by design: draw-list caches and dirty flags; lighting recomputed per frame, blocks drawn depth-tested and eye-relative (AGENTS.md:721), cells released in ac2-smartbox cells.rs:156 [AC2_Rusted/crates/ac2-render/src/scene/mod.rs:840]
- [x] `LScapeRender::UpdateLightingInfo` · 005839f0 · legions 00553160 — covered: forwarder to LScapeLighting::UpdateInfo; landscape_lighting.rs Final::from_sky / update_lighting each frame (doc names it)
- [-] `LScapeRender::FlagRefreshDrawLists` · 00583a00 · legions 00553170 — replaced by design: draw-list caches and dirty flags; lighting recomputed per frame, blocks drawn depth-tested and eye-relative (AGENTS.md:721), cells released in ac2-smartbox cells.rs:156 [AC2_Rusted/crates/ac2-render/src/scene/mod.rs:840]
- [-] `LScapeRender::CalcBlockDrawFrame` · 00583a40 · legions 005531b0 — replaced by design: draw-list caches and dirty flags; lighting recomputed per frame, blocks drawn depth-tested and eye-relative (AGENTS.md:721), cells released in ac2-smartbox cells.rs:156 [AC2_Rusted/crates/ac2-render/src/scene/mod.rs:840]

### RainSegment
- [-] `RainSegment::operator=` · 00583aa0 · legions 00553210 — plain data ctor/dtor/operator=; Rust struct or Drop replaces

### LScapeRender
- [-] `LScapeRender::LScapeRender` · 00583b00 · legions 00553270 — replaced by design: draw-list caches and dirty flags; lighting recomputed per frame, blocks drawn depth-tested and eye-relative (AGENTS.md:721), cells released in ac2-smartbox cells.rs:156 [AC2_Rusted/crates/ac2-render/src/scene/mod.rs:840]

### CPhysicsObj
- [-] `CPhysicsObj::get_acceleration` · 00583c10 · legions 00553380 — not needed: trivial physics accessor; ac2-physics owns acceleration (motion_interp.rs:158) [decompiled-pdb: no callers]

### LScapeRender
- [-] `LScapeRender::~LScapeRender` · 00583c20 · legions 00553390 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [-] `LScapeRender::NotifyChangeRegion` · 00583d00 · legions 00553470 — replaced by design: terrain mesh teardown/rebuild on draw-distance or region change; scene options re-applied (set_scene_options) and meshes rebuilt from resident blocks [AC2_Rusted/crates/ac2-client/src/view.rs:304]
- [-] `LScapeRender::ReleaseCells` · 00583d20 · legions 00553490 — replaced by design: draw-list caches and dirty flags; lighting recomputed per frame, blocks drawn depth-tested and eye-relative (AGENTS.md:721), cells released in ac2-smartbox cells.rs:156 [AC2_Rusted/crates/ac2-render/src/scene/mod.rs:840]
- [x] `LScapeRender::AddCellSounds` · 00583d40 · legions 005534b0 — ac2-audio/ac2-client: ambient environments, groove
- [-] `LScapeRender::CullLandBlocks` · 00583e70 · legions 005535e0 — not needed: only loops the resident blocks calling CLandBlock::CullObjects, itself skipped (culling optimisation, depth test hides what retail culled)
- [x] `LScapeRender::FindReflectionPlane` · 00583f40 · legions 00553680 — ac2-render scene/lighting.rs find_reflection_plane: the viewer's block's water below it, else the first resident block's; none in water
- [x] `LScapeRender::GatherVisibleOutsideObjects` · 00584040 · legions 00553780 — covered: scene/mod.rs frame: each resident block placements (CLandBlock::GatherObjects, done) gathered only when the viewer can see outside; comment names it
- [x] `LScapeRender::GetSunLightDirectionBits` · 00584090 · legions 005537d0 — self term: frill shade from its terrain cell's vertex colour bits (ac2-render terrain/shadow.rs, heights.rs self_shadow); map term in object.wgsl landscape_shadow
- [x] `LScapeRender::UpdateLighting` · 005841c0 · legions 00553900 — audit: covered: sun light-direction bits for the landscape self shadow, plus scene/lighting.rs [AC2_Rusted/crates/ac2-render/src/terrain/shadow.rs:5]
- [x] `LScapeRender::CalculateShadowColor` · 00584290 · legions 005539d0 — self term: frill shade from its terrain cell's vertex colour bits (ac2-render terrain/shadow.rs, heights.rs self_shadow); map term in object.wgsl landscape_shadow
- [x] `LScapeRender::GetTerrainGeometry` · 005843b0 · legions 00553af0 — audit: covered: terrain polygon geometry for landscape decals [AC2_Rusted/crates/ac2-render/src/scene/decals.rs:2]
- [x] `LScapeRender::RefreshShadowMap` · 00584540 · legions 00553c80 — ac2_gpu::StaticShadowMap (shadow_map.rs, object_shadow.rs); redraws on block/caster/sun>1deg/strength change. SceneTool::m_bRenderingToShadowMap (set in SceneTool::StartShadowMap, cleared in FinishShadowMap) clears CPartArray::Draw bVar7 (like m_bDisableDegrades): casters = SceneFrame::shadow_placements (SceneDegrades::shadow_placements), every scenery placement undegraded, unfaded, not distance-culled; the sun cascades static casters use the same list
- [x] `LScapeRender::Init` · 005847b0 · legions 00553ee0 — audit: covered: rain pool, rain material; other state replaced [AC2_Rusted/crates/ac2-render/src/rain.rs:6]
- [-] `LScapeRender::CalcBlockDrawOrderInternal` · 00584950 · legions 00554080 — replaced by design: draw-list caches and dirty flags; lighting recomputed per frame, blocks drawn depth-tested and eye-relative (AGENTS.md:721), cells released in ac2-smartbox cells.rs:156 [AC2_Rusted/crates/ac2-render/src/scene/mod.rs:840]
- [-] `LScapeRender::CalcBlockDrawOrder` · 00584b10 · legions 00554240 — replaced by design: draw-list caches and dirty flags; lighting recomputed per frame, blocks drawn depth-tested and eye-relative (AGENTS.md:721), cells released in ac2-smartbox cells.rs:156 [AC2_Rusted/crates/ac2-render/src/scene/mod.rs:840]
- [x] `LScapeRender::SetViewerPosition` · 00584c80 · legions 005543b0 — audit: covered: water-plane / fog-area choice at the viewer [AC2_Rusted/crates/ac2-render/src/scene/lighting.rs:20]
- [x] `LScapeRender::FillRainVertices` · 00584d70 · legions 00554490 — ac2-render rain.rs RainField (streaks, splashes and near-water ripples) + scene rain_frame discharging WorldRender's rain particle systems
- [x] `LScapeRender::UpdateRain` · 005856f0 · legions 00554de0 — ac2-render rain.rs RainField (streaks, splashes and near-water ripples) + scene rain_frame discharging WorldRender's rain particle systems

## `portal/client/src/smartbox/worldrender/lscapesky.cpp.c` (6/9)

### LScapeSky
- [x] `LScapeSky::SetSkyFog` · 005857b0 · legions 00554ea0 — ac2-world region::sky_objects, ac2-render sky
- [x] `LScapeSky::CalcFrame` · 005857f0 · legions 00554ee0 — ac2-world region::sky_objects, ac2-render sky
- [-] `LScapeSky::Clear` · 00585870 · legions 00554f60 — replaced by design: sky object list ctor/clear; Rust rebuilds from the day sample each frame [AC2_Rusted/crates/ac2-render/src/sky.rs:1]
- [x] `LScapeSky::SetViewerPosition` · 005858b0 · legions 00554fa0 — ac2-world region::sky_objects, ac2-render sky
- [x] `LScapeSky::Draw` · 00585980 · legions 00555070 — sky objects drawn as tinted object instances before/after the world in CDayDesc::GetSky list order with the fog_sky quad, then each up object's lens flare (ac2-gpu lens_flare.rs); only viewer-centred domes tint the modern atmosphere (ac2-gpu SkyRole); fixed_pos/height_mod are carried but retail's sky drawing never reads them (CalcFrame, SetViewerPosition)
- [-] `LScapeSky::LScapeSky` · 00585b30 · legions 00555220 — replaced by design: sky object list ctor/clear; Rust rebuilds from the day sample each frame [AC2_Rusted/crates/ac2-render/src/sky.rs:1]
- [-] `LScapeSky::~LScapeSky` · 00585b50 · legions 00555240 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [x] `LScapeSky::CreateDeletePhysicsObjects` · 00585ba0 · legions 00555290 — audit: covered: objects get CalcFrame heading/pitch, colour tint and alpha; flares (AGENTS.md:162) now done; retail does not read fixed_pos/height_mod here (catalog [~] stale) [AC2_Rusted/crates/ac2-render/src/sky.rs:3]
- [x] `LScapeSky::UpdateTimeOfDay` · 00585cf0 · legions 005553e0 — audit: covered: objects get CalcFrame heading/pitch, colour tint and alpha; flares (AGENTS.md:162) now done; retail does not read fixed_pos/height_mod here (catalog [~] stale) [AC2_Rusted/crates/ac2-render/src/sky.rs:3]

## `portal/client/src/smartbox/worldrender/lscapeterrainmesh.cpp.c` (20/48)

### LM_UVTranslate
- [x] `LM_UVTranslate::SetVTranslate` · 005880a0 · legions 005577e0 — audit: covered: UvTranslate layer modifier waveforms [AC2_Rusted/crates/ac2-assets/src/ac2_material.rs:54]

### LScapeTerrainMesh
- [-] `LScapeTerrainMesh::CreateLocalMemBumpMap` · 005880f0 · legions 00557830 — modern pass: retail built a 256x256 V8U8 bump map from cos waves; ripples are procedural in water_reflection.wgsl [AC2_Rusted/AGENTS.md:174]
- [x] `LScapeTerrainMesh::SetSelfShadowBit` · 00588300 · legions 00557a30 — self term: frill shade from its terrain cell's vertex colour bits (ac2-render terrain/shadow.rs, heights.rs self_shadow); map term in object.wgsl landscape_shadow
- [x] `LScapeTerrainMesh::IlluminateVertices` · 00588320 · legions 00557a50

### LScapeVertex
- [-] `LScapeVertex::operator=` · 00588370 · legions 0059f0d0 — plain data ctor/dtor/operator=; Rust struct or Drop replaces

### LScapeTerrainMesh
- [x] `LScapeTerrainMesh::CalculatePointsOfIntersection` · 00588410 · legions 00557aa0 — ac2-render terrain/shore.rs generate_shore over the near blocks' ground triangles (made again when the near blocks or centre change, scene/shore_strips.rs)

### ShorePointType
- [-] `ShorePointType::operator=` · 005884d0 · legions 00557b60 — plain data ctor/dtor/operator=; Rust struct or Drop replaces

### LandDefs
- [x] `LandDefs::blockcoord_to_blockid` · 00588530 · legions 00557bc0 — audit: covered: get_block_gid [AC2_Rusted/crates/ac2-world/src/land_blocks.rs:48]

### CVertexArray
- [-] `CVertexArray::CVertexArray` · 00588590 · legions 00557c20 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [-] `CVertexArray::~CVertexArray` · 005885e0 · legions 00557c70 — plain data ctor/dtor/operator=; Rust struct or Drop replaces

### LScapeTerrainMesh
- [x] `LScapeTerrainMesh::DrawTerrain` · 00588610 — covered: near mesh plus far chunks from FIRST_FAR_CHUNK (nLastFragment 2) kept when the camera frustum does not exclude their box (SceneFrame::far_chunks, named there); N-patches pref 0x4000001e unregistered (off)
- [x] `LScapeTerrainMesh::DrawWaterMesh` · 00588840 · legions 00557ee0 — audit: covered: near water mesh plus far water chunks; reflection AGENTS.md:174, shore strips terrain/shore.rs (catalog [~] stale) [AC2_Rusted/crates/ac2-render/src/terrain/far.rs:15]
- [-] `LScapeTerrainMesh::ApplyList` · 00588a30 · legions 005580d0 — replaced by design: near-ring vertex array shifting/ordering and mesh lifecycle; Rust builds a per-block near mesh and far chunks (terrain/far.rs) [AC2_Rusted/crates/ac2-render/src/terrain/mesh.rs:1]
- [-] `LScapeTerrainMesh::InqNearBlockIndex` · 00588aa0 · legions 00558140 — replaced by design: near-ring vertex array shifting/ordering and mesh lifecycle; Rust builds a per-block near mesh and far chunks (terrain/far.rs) [AC2_Rusted/crates/ac2-render/src/terrain/mesh.rs:1]

### (free functions)
- [x] `G_CalculateNormal` · 00588ae0 · legions 00558180 — ac2-render terrain/shore.rs generate_shore over the near blocks' ground triangles (made again when the near blocks or centre change, scene/shore_strips.rs)

### LScapeTerrainMesh
- [x] `LScapeTerrainMesh::FindShorePointIndexByOrigin` · 00588b70 · legions 00558210 — ac2-render terrain/shore.rs generate_shore over the near blocks' ground triangles (made again when the near blocks or centre change, scene/shore_strips.rs)
- [-] `LScapeTerrainMesh::DrawShore` · 00588be0 · legions 00558280 — modern pass: DrawShore(true) at LandscapeShoreEffects 2 drawn (terrain/shore.rs:72); the dest-alpha pass (false) only attenuated retail's reflection blend near shores, replaced by Fresnel; minor residual [AC2_Rusted/AGENTS.md:174]

### DBObjGrabber<RenderTexture>
- [-] `DBObjGrabber<RenderTexture>::DBObjGrabber<RenderTexture>` · 00588e10 · legions 005584b0 — retail DB object cache handle/release; Arc per DID replaces (AGENTS.md exception)

### LScapeFarChunk
- [-] `LScapeFarChunk::LScapeFarChunk` · 005890f0 · legions 00558840 — replaced by design: near-ring vertex array shifting/ordering and mesh lifecycle; Rust builds a per-block near mesh and far chunks (terrain/far.rs) [AC2_Rusted/crates/ac2-render/src/terrain/mesh.rs:1]

### LScapeTerrainMesh
- [x] `LScapeTerrainMesh::BuildFarTerrainMesh` · 00589130 · legions 00558880 — indices with RollDice diagonals and LOD crack fill, vertices, far water quads (terrain/far.rs)
- [x] `LScapeTerrainMesh::SetVertex` · 00589ec0 · legions 00559610 — LScapeTerrainMesh::SetVertex cited in ac2-render terrain/mesh.rs:2 (four vertices per polygon, slots SW/SE/NW/NE)
- [x] `LScapeTerrainMesh::GetPixelGeometry` · 0058a000 · legions 00559750 — audit: covered: quad a decal gets at a point [AC2_Rusted/crates/ac2-render/src/terrain/heights.rs:101]
- [x] `LScapeTerrainMesh::CleanupShore` · 0058a1f0 · legions 00559940 — ac2-render terrain/shore.rs generate_shore over the near blocks' ground triangles (made again when the near blocks or centre change, scene/shore_strips.rs)

### SmartArray<ShoreStripType>
- [-] `SmartArray<ShoreStripType>::~SmartArray<ShoreStripType>` · 0058a270 · legions 005599f0 — SmartArray/STL container internals; Vec replaces

### SmartArray<short>
- [-] `SmartArray<short>::operator=` · 0058a400 · legions 00559af0 — SmartArray/STL container internals; Vec replaces

### SmartArray<LScapeFarBlock>
- [-] `SmartArray<LScapeFarBlock>::operator=` · 0058a460 · legions 00559b50 — SmartArray/STL container internals; Vec replaces

### LScapeTerrainMesh
- [x] `LScapeTerrainMesh::GetShadowColor` · 0058a4d0 · legions 00559bc0 — self term: frill shade from its terrain cell's vertex colour bits (ac2-render terrain/shadow.rs, heights.rs self_shadow); map term in object.wgsl landscape_shadow
- [-] `LScapeTerrainMesh::SetVertexValue` · 0058a870 · legions 00559f60 — not needed: terrain vertex edit for the world builder/admin; the client never edits terrain [decompiled-pdb: no callers (virtual) ; AGENTS 'no admin tools']

### SmartArray<LayerStage_*>
- [-] `SmartArray<LayerStage_*>::InsertOrderedByIndex` · 0058aab0 · legions 00540d30 — SmartArray/STL container internals; Vec replaces

### LScapeFarChunk
- [-] `LScapeFarChunk::operator=` · 0058ac60 · legions 0055a2e0 — plain data ctor/dtor/operator=; Rust struct or Drop replaces

### MaterialLayer
- [-] `MaterialLayer::InsertModifier` · 0058ad10 · legions 0055a390 — replaced by design: builders for code-made materials (default, water bump, streak, far terrain) and Vec pushes; DAT materials arrive complete from ac2-assets, code-made ones are WGSL pipelines [AC2_Rusted/crates/ac2-assets/src/ac2_material.rs:69]

### LScapeTerrainMesh
- [-] `LScapeTerrainMesh::DestroyLScapeMaterialInfo` · 0058ad30 · legions 0055a3b0 — retail DB object cache handle/release; Arc per DID replaces (AGENTS.md exception)
- [-] `LScapeTerrainMesh::DrawWater` · 0058ad90 — modern pass: reflection blend over water by SSR + Fresnel with procedural ripples; near/far meshes and shore drawn (ac2-gpu scene.rs:475) [AC2_Rusted/AGENTS.md:174]
- [-] `LScapeTerrainMesh::ReOrderNearVertices` · 0058b460 · legions 0055aad0 — replaced by design: near-ring vertex array shifting/ordering and mesh lifecycle; Rust builds a per-block near mesh and far chunks (terrain/far.rs) [AC2_Rusted/crates/ac2-render/src/terrain/mesh.rs:1]
- [x] `LScapeTerrainMesh::BuildNearWaterMesh` · 0058b600 · legions 0055ac70 — ac2-render object::load_water_model
- [-] `LScapeTerrainMesh::Destroy` · 0058b8a0 · legions 0055af10 — retail DB object cache handle/release; Arc per DID replaces (AGENTS.md exception)
- [x] `LScapeTerrainMesh::BuildNearTerrainMesh` · 0058b9e0 · legions 0055b050 — audit: covered: per-block near mesh with baked vertex light and colour bits; cloud shadows via the static shadow map (shadow_map.rs, loader.rs:85); shared near vertex array is plumbing [AC2_Rusted/crates/ac2-render/src/terrain/mesh.rs:1]
- [-] `LScapeTerrainMesh::~LScapeTerrainMesh` · 0058bc10 · legions 0055b280 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [x] `LScapeTerrainMesh::InitFarMesh` · 0058bce0 · legions 0055b350 — far chunks, LOD, block edges, far material with ADDSIGNED + MODULATE2X cloud stage (far_terrain.wgsl)
- [x] `LScapeTerrainMesh::UpdateNearTerrain` · 0058c8c0 · legions 0055bf30 — audit: covered: per-block near mesh with baked vertex light and colour bits; cloud shadows via the static shadow map (shadow_map.rs, loader.rs:85); shared near vertex array is plumbing [AC2_Rusted/crates/ac2-render/src/terrain/mesh.rs:1]
- [x] `LScapeTerrainMesh::UpdateNearWater` · 0058d700 · legions 0055cd70 — ac2-render object::load_water_model
- [x] `LScapeTerrainMesh::GenerateShore` · 0058dcb0 · legions 0055d320 — ac2-render terrain/shore.rs generate_shore over the near blocks' ground triangles (made again when the near blocks or centre change, scene/shore_strips.rs)
- [-] `LScapeTerrainMesh::Init` · 0058e940 · legions 0055dfb0 — replaced by design: near-ring vertex array shifting/ordering and mesh lifecycle; Rust builds a per-block near mesh and far chunks (terrain/far.rs) [AC2_Rusted/crates/ac2-render/src/terrain/mesh.rs:1]
- [-] `LScapeTerrainMesh::LScapeTerrainMesh` · 0058eb20 · legions 0055e190 — replaced by design: near-ring vertex array shifting/ordering and mesh lifecycle; Rust builds a per-block near mesh and far chunks (terrain/far.rs) [AC2_Rusted/crates/ac2-render/src/terrain/mesh.rs:1]
- [-] `LScapeTerrainMesh::RefreshWorldBuilderTerrain` · 0058ec60 · legions 0055e2d0 — not needed: terrain vertex edit for the world builder/admin; the client never edits terrain [decompiled-pdb: no callers (virtual) ; AGENTS 'no admin tools']
- [-] `LScapeTerrainMesh::Update` · 0058ee40 · legions 0055e4b0 — replaced by design: near-ring vertex array shifting/ordering and mesh lifecycle; Rust builds a per-block near mesh and far chunks (terrain/far.rs) [AC2_Rusted/crates/ac2-render/src/terrain/mesh.rs:1]
- [-] `LScapeTerrainMesh::Refresh` · 0058f030 · legions 0055e6a0 — replaced by design: near-ring vertex array shifting/ordering and mesh lifecycle; Rust builds a per-block near mesh and far chunks (terrain/far.rs) [AC2_Rusted/crates/ac2-render/src/terrain/mesh.rs:1]
- [-] `LScapeTerrainMesh::Refresh` · 0058f070 · legions 0055e6e0 — replaced by design: near-ring vertex array shifting/ordering and mesh lifecycle; Rust builds a per-block near mesh and far chunks (terrain/far.rs) [AC2_Rusted/crates/ac2-render/src/terrain/mesh.rs:1]

## `portal/client/src/smartbox/worldrender/portalspace.cpp.c` (14/14)

### CDataMesh
- [x] `CDataMesh::GetByEnum` · 00585e60 · legions 00555550 — audit: covered: portal background mesh by enum [AC2_Rusted/crates/ac2-render/src/portal/assets.rs:2]

### PortalSpace::Effect
- [x] `PortalSpace::Effect::Effect` · 00585e90 · legions 00555580 — audit: covered: tunnel geometry/rings/fade (AGENTS.md:220); ring-to-alpha mapping flagged unverified in the catalog (tunnel_tests.rs) [AC2_Rusted/crates/ac2-render/src/portal/tunnel.rs]
- [x] `PortalSpace::Effect::MakeSafePortalIndex` · 00585f40 · legions 00555630 — ac2-math perlin.rs, matrix.rs; ac2-render portal/tunnel.rs; tests: tunnel_tests, perlin, matrix
- [x] `PortalSpace::Effect::GeneratePortals` · 00585f90 · legions 00555680 — ac2-render portal/tunnel.rs and effect.rs: GeneratePortals' local matrix carries (0,3,0); Think's phase machine, fader, tunnel advance; StartNow/StopNow sounds via ac2-client play_portal_sounds
- [x] `PortalSpace::Effect::StartNow` · 00586220 · legions 00555910 — ac2-render portal/tunnel.rs and effect.rs: GeneratePortals' local matrix carries (0,3,0); Think's phase machine, fader, tunnel advance; StartNow/StopNow sounds via ac2-client play_portal_sounds
- [x] `PortalSpace::Effect::Destroy` · 00586650 · legions 00555d40 — materials 0x40000013/0x40000014, the clouds' PSDesc 0x40000015 (ac2-render portal/assets.rs load_clouds_desc, Client::set_portal_clouds), the background CDataMesh 0x40000016 (load_portal_models) and the sounds load at setup (client_teleport.rs); freed on last drop
- [x] `PortalSpace::Effect::Start` · 00586720 — ac2-render portal/effect.rs (enter/exit/in_portal_space, Start/Stop) and ac2-client client_teleport.rs; tests: portal_7160, effect_tests
- [x] `PortalSpace::Effect::UpdateGeometry` · 00586740 · legions 00555e30 — ac2-render portal/tunnel.rs update_geometry(current, start, scale): ring base is m_nLastPortalPassed, start is the transparent depth, memory order reversed (closing vertex first), x stored as float, 1/w in f64; alpha ramp verified (0.8+0.2(1-X/15), 0.8(1-X/149)^2)
- [x] `PortalSpace::Effect::StopNow` · 00586eb0 · legions 005565a0 — ac2-render portal/tunnel.rs and effect.rs: GeneratePortals' local matrix carries (0,3,0); Think's phase machine, fader, tunnel advance; StartNow/StopNow sounds via ac2-client play_portal_sounds
- [x] `PortalSpace::Effect::Think` · 00586f50 · legions 00556640 — ac2-render portal/effect.rs: phase machine (1 s phases), startup fader black (0,0,0,t), shutdown fader white (1,1,1,t) (Legions dTimeNow 0x3F800000_3F800000), FOV scales 1-0.8t^2 / 1+1.4t^2 / 1+1.4(1-t)^2, tunnel advance 120 units/s of local_time, regeneration; tests: effect_tests, portal_motion_7160
- [x] `PortalSpace::Effect::Render` · 00587360 · legions 00556a50 — ac2-gpu portal_space.rs, ac2-render portal/: fader, background mesh 0x40000016 at the eye with the opacity as alpha, outer then inner rings, discharge loop (portal/clouds.rs) and cloud particles; camera = slerped/lerped portal frames, backed (1-vis)*100 along row 1; SetupView's rolled basis drawn by turning the geometry about the eye (PortalCamera::upright), particles face that basis; SetFOV(65 * m_fCameraFOVScale); unlit, unfogged; tests: camera tests, portal_view_7160, portal_motion_7160
- [x] `PortalSpace::Effect::~Effect` · 00587da0 · legions 00557490 — materials 0x40000013/0x40000014, the clouds' PSDesc 0x40000015 (ac2-render portal/assets.rs load_clouds_desc, Client::set_portal_clouds), the background CDataMesh 0x40000016 (load_portal_models) and the sounds load at setup (client_teleport.rs); freed on last drop
- [x] `PortalSpace::Effect::Initialize` · 00587db0 · legions 005574a0 — materials 0x40000013/0x40000014, the clouds' PSDesc 0x40000015 (ac2-render portal/assets.rs load_clouds_desc, Client::set_portal_clouds), the background CDataMesh 0x40000016 (load_portal_models) and the sounds load at setup (client_teleport.rs); freed on last drop
- [x] `PortalSpace::Effect::Stop` · 00588050 — ac2-render portal/effect.rs (enter/exit/in_portal_space, Start/Stop) and ac2-client client_teleport.rs; tests: portal_7160, effect_tests

## `portal/client/src/smartbox/worldrender/worldrender.cpp.c` (24/36)

### GameTime
- [-] `GameTime::GetTimeTilTomorrow` · 0057c720 · legions 0054bff0 — not needed: only the retry deadline when the region's sky/day desc is missing; Rust samples the sky from server time (ac2-world region_environment.rs) [decompiled-pdb worldrender.cpp.c:1127-1170]
- [x] `GameTime::GetNeedLightingRefresh` · 0057c740 · legions 0054c010 — ac2-world GameClock::alter_time_of_day_immediate / temporal_dilation (x87 double arithmetic, objdump-verified) and need_lighting_refresh
- [x] `GameTime::NotifyLightingRefreshed` · 0057c760 · legions 0054c030 — ac2-world GameClock::alter_time_of_day_immediate / temporal_dilation (x87 double arithmetic, objdump-verified) and need_lighting_refresh

### PhysicsGlobals
- [x] `PhysicsGlobals::SetWind` · 0057c780 · legions 0054c050 — ac2-world RegionDesc::physics_wind (rain wind or the Perlin breeze); ac2-client sets it on the bodies each animation step

### FogInfo
- [-] `FogInfo::FogInfo` · 0057c7d0 · legions 0054c0a0 — plain data ctor/dtor/operator=; Rust struct or Drop replaces

### LightInfo
- [-] `LightInfo::LightInfo` · 0057c820 · legions 0054c0f0 — plain data ctor/dtor/operator=; Rust struct or Drop replaces

### LScapeLighting
- [x] `LScapeLighting::GetShadowColor` · 0057c8b0 · legions 0054c180 — audit: covered: day light shadow colour -> static_shadow_strength [AC2_Rusted/crates/ac2-render/src/view.rs:85]

### WorldRender
- [x] `WorldRender::Init` · 0057c8e0 · legions 0054c1b0 — audit: covered: rain vapour/ripples/bubbles systems, sky; ambient sound init in ac2-audio [AC2_Rusted/crates/ac2-render/src/rain.rs:23]
- [x] `WorldRender::EnterPortalSpace` · 0057ca90 — ac2-render portal/effect.rs (enter/exit/in_portal_space, Start/Stop) and ac2-client client_teleport.rs; tests: portal_7160, effect_tests
- [x] `WorldRender::ExitPortalSpace` · 0057cab0 — ac2-render portal/effect.rs (enter/exit/in_portal_space, Start/Stop) and ac2-client client_teleport.rs; tests: portal_7160, effect_tests
- [x] `WorldRender::InPortalSpace` · 0057cad0 · legions 0054c3a0 — ac2-render portal/effect.rs (enter/exit/in_portal_space, Start/Stop) and ac2-client client_teleport.rs; tests: portal_7160, effect_tests
- [x] `WorldRender::ForceShadowRefresh` · 0057caf0 · legions 0054c3c0 — audit: covered: static shadow map redraws on block/caster/sun/strength change [AC2_Rusted/crates/ac2-gpu/src/scene.rs:503]
- [x] `WorldRender::UpdateAmbientSounds` · 0057cb00 · legions 0054c3d0 — ac2-audio: rain.rs (RainSounds, the loop and thunder), ambient_driver.rs (AmbientDriver over the SoundSpace trait); ac2-client sound_ids.rs (DIDs), client_ambient.rs and sound_space.rs (physics adapter); tests sound_ids_tests.rs and ac2-audio unit tests
- [-] `WorldRender::InitAmbientSounds` · 0057cb40 — not needed: empty/admin stub; ambient sounds are ac2-audio ambient_driver.rs [decompiled-pdb: no callers, no Legions address]
- [x] `WorldRender::SetupEnvironment` · 0057cb50 · legions 0054c420 — audit: covered: outside, sky and inside environments (interior ambient + linear fog); catalog [~] stale [AC2_Rusted/crates/ac2-render/src/object/lights.rs:55]
- [x] `WorldRender::UpdateRainSounds` · 0057cf80 · legions 0054c850 — ac2-audio: rain.rs (RainSounds, the loop and thunder), ambient_driver.rs (AmbientDriver over the SoundSpace trait); ac2-client sound_ids.rs (DIDs), client_ambient.rs and sound_space.rs (physics adapter); tests sound_ids_tests.rs and ac2-audio unit tests

### SceneCamera
- [-] `SceneCamera::SceneCamera` · 0057d190 · legions 0054ca60 — replaced by design: copy constructor [AC2_Rusted/crates/ac2-render/src/view.rs:6]

### WorldRender
- [x] `WorldRender::DrawBrushes` · 0057d290 · legions 0054cb60 — audit: covered: objects in env cells draw under the inside environment; phases collapse into the one pass [AC2_Rusted/crates/ac2-gpu/src/scene.rs:884]
- [-] `WorldRender::CreateDiscShadows` · 0057d3d0 · legions 0054cca0 — modern pass: stencil volumes / disc decals replaced by cascaded sun depth maps from every level above 0 (ac2-gpu sun_shadow.rs, object_sun_casters.rs) [AC2_Rusted/AGENTS.md:141]
- [x] `WorldRender::StartSelection` · 0057d680 · legions 0054cf50 — audit: covered: pick by projected box then ray trace through parts, nearest first, misses stay potential; selectability/LookAtSelection live with the caller (gmSelectionUI) [AC2_Rusted/crates/ac2-render/src/selection.rs:1]

### DBObjGrabber<CDayDesc>
- [-] `DBObjGrabber<CDayDesc>::DBObjGrabber<CDayDesc>` · 0057d6d0 · legions 0054cfa0 — retail DB object cache handle/release; Arc per DID replaces (AGENTS.md exception)

### WorldRender
- [-] `WorldRender::WorldRender` · 0057d870 · legions 0054d140 — replaced by design: constructors; the scene owns this state [AC2_Rusted/crates/ac2-render/src/scene/mod.rs]
- [x] `WorldRender::SetWantNoticeOnNextFrameDrawn` · 0057da00 · legions 0054d2d0 — audit: covered: NextFrameDrawn notice (client.rs:83) [AC2_Rusted/crates/ac2-client/src/client.rs:155]
- [x] `WorldRender::GetWantNoticeOnNextFrameDrawn` · 0057da10 · legions 0054d2e0 — audit: covered: NextFrameDrawn notice (client.rs:83) [AC2_Rusted/crates/ac2-client/src/client.rs:155]
- [-] `WorldRender::~WorldRender` · 0057da20 · legions 0054d2f0 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [-] `WorldRender::ResetWorldState` · 0057db20 · legions 0054d3f0 — replaced by design: terrain mesh teardown/rebuild on draw-distance or region change; scene options re-applied (set_scene_options) and meshes rebuilt from resident blocks [AC2_Rusted/crates/ac2-client/src/view.rs:304]
- [-] `WorldRender::DrawWorldObjects` · 0057dba0 · legions 0054d470 — modern pass: opaque/shadow-volume/translucent phases collapse into WorldRender::Draw order with sun cascades (AGENTS.md:141) [AC2_Rusted/crates/ac2-gpu/src/scene.rs:2]
- [x] `WorldRender::UpdateTimeOfDay` · 0057dcc0 · legions 0054d590 — audit: covered: tick cadence, day pick, rain/wind (region.rs:123), sky, lighting [AC2_Rusted/crates/ac2-world/src/region_environment.rs:22]
- [x] `WorldRender::Update` · 0057e1b0 · legions 0054da80 — audit: covered: time of day (ac2-world), rain sounds (ac2-audio rain.rs), ambient (ambient_driver.rs), decals/streaks thinks; catalog [~] stale [AC2_Rusted/crates/ac2-client/src/client.rs:785]
- [x] `WorldRender::RenderReflectionMap` · 0057e270 · legions 0054db40 — replaced by ac2-gpu water_reflection.rs's screen-space reflection over the same plane (AGENTS.md exception)
- [-] `WorldRender::UpdateRenderTargets` · 0057e750 · legions 0054e020 — D3D device/resource plumbing; wgpu replaces
- [x] `WorldRender::GatherIndoorObjects` · 0057e830 · legions 0054e100 — visible_interior_cells (scene/mod.rs): the walk from the camera's own cell, the outside portals' viewports, and the bDrawSky and bFogCull out flags (Sight); test a_fog_culled_dungeon_view_paints_the_interior_fog_colour_first
- [x] `WorldRender::GatherOutdoorObjects` · 0057ea20 · legions 0054e2f0 — covered: scene/mod.rs Sight::can_see_outside (m_bCanSeeOutside) and the outside gather (GatherVisibleOutsideObjects) only then; named in Sight and frame()
- [x] `WorldRender::CreateLandscapePointLightDecals` · 0057eb10 · legions 0054e3e0 — ac2-render scene/light_decals.rs: each frame the FX point lights and the drawn cells' own lights within reach over the ground pool there as draw-once decals of the day/dusk/night material (enums 0x4000000D/2B/2C), gated by LightSourceEffects
- [x] `WorldRender::RenderSkeletonEffects` · 0057ef60 · legions 0054e830 — ac2-render scene/skeleton_effects.rs: the outdoor bodies' bones discharge rain vapour (rain over 0.4, bones across the rain), water bubbles and ripples into the shared systems' first emitter; the bubbles system (enum 0x40000011) loads with the rain's
- [x] `WorldRender::Render` · 0057f960 · legions 0054f230 — covered in parts: interior fog-cull quad (SceneFrame::interior_fog), portal gate and world FOV scale (ac2-client view.rs), sky before/after (LScapeSky::Draw), objects/terrain/water/interiors in ac2-gpu scene.rs WorldRender order, decals (DecalEffectManager::Render), selection (selection.rs), rain and water bubbles queued (scene/mod.rs rain_frame), particles; volumetric shadows and the reflection target replaced by the sun cascades and screen-space reflection (AGENTS.md). The DynamicLightVisibleCells sun pass stays deferred under its own row
