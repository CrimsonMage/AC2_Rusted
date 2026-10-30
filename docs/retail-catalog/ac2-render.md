# ac2-render: Scene and world rendering

781 retail functions: 179 done, 54 partial, 188 to do, 360 skipped.
Legend and workflow: [README](README.md).

## `portal/client/src/ctrilist/ctrianglelist.cpp.c` (0/12)

### CMeshFragment
- [-] `CMeshFragment::CMeshFragment` · 0052bf30 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [ ] `CMeshFragment::InitIndexed` · 0052bf60 · legions 00510b50 — needs port: mesh fragment init (vertex/index arrays of a mesh part)
- [ ] `CMeshFragment::Init` · 0052bfd0 · legions 00510bc0 — needs port: mesh fragment init (vertex/index arrays of a mesh part)

### CMeshTriList
- [-] `CMeshTriList::CMeshTriList` · 0052c000 · legions 00510bf0 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [-] `CMeshTriList::~CMeshTriList` · 0052c030 · legions 00510c20 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [-] `CMeshTriList::GetPackSize` · 0052c120 · legions 00510d10 — save/write path; the client reads DATs only
- [-] `CMeshTriList::Pack` · 0052c190 · legions 00510d80 — save/write path; the client reads DATs only
- [ ] `CMeshTriList::UnPack` · 0052c260 · legions 00510e50 — needs port: mesh triangle list (detail levels, DAT read)
- [ ] `CMeshTriList::Init` · 0052c390 · legions 00510f80 — needs port: mesh triangle list (detail levels, DAT read)

### DegradeType
- [ ] `DegradeType::ParseString` · 0052c460 — needs port: degrade level name parsing (detail settings)

### CMeshFragment
- [-] `CMeshFragment::FromFileNode` · 0052c600 — text source format (admin tools)

### CMeshTriList
- [-] `CMeshTriList::FromFileNode` · 0052c830 — text source format (admin tools)

## `portal/client/src/render/core/font/font.cpp.c` (6/13)

### Font
- [-] `Font::Begin` · 005b88d0 · legions 00583530 — D3D device/resource plumbing; wgpu replaces
- [-] `Font::End` · 005b8900 · legions 00583560 — D3D device/resource plumbing; wgpu replaces
- [-] `Font::Destroy` · 005b8940 · legions 005835a0 — retail DB object cache handle/release; Arc per DID replaces (AGENTS.md exception)
- [x] `Font::GetCharDesc` · 005b89a0 · legions 00583600 — ac2-assets font.rs: all 55 portal fonts decode
- [x] `Font::GetCharWidthA` · 005b8a30 · legions 00583690 — ac2-ui text.rs / text_attr.rs: UIElement_Text layout and drawing
- [x] `Font::ContainsString` · 005b8a50 · legions 005836b0 — ac2-assets font.rs Ac2Font::contains per character (a glyph of its own, not the ? stand-in char_desc gives) and ac2-ui text.rs font_contains over the string, newline skipped
- [x] `Font::GenerateCharacterMap` · 005b8ad0 · legions 00583730 — ac2-assets font.rs: all 55 portal fonts decode
- [x] `Font::GetStringWidthInternal` · 005b8bc0 · legions 00583820 — ac2-assets font.rs: all 55 portal fonts decode
- [ ] `Font::InitLoad` · 005b8c20 — needs port: font data (glyph presence, DBO load)
- [-] `Font::LoadFromFile` · 005b8c40 — text source loader, admin build only
- [-] `Font::SaveToFile` · 005b8d40 — admin-only/tool/debug code, not in the client (font save tool)
- [x] `Font::StreamPack` · 005b8e10 · legions 00583890 — ac2-assets font.rs: all 55 portal fonts decode
- [ ] `Font::GetStringWidth` · 005b8f70 · legions 005839f0 — needs port (check): Rust code with a similar name exists but does not cite this function; verify and cite, or port

## `portal/client/src/render/core/font/fontmapper.cpp.c` (1/4)

### FontLocal
- [x] `FontLocal::GetGlobalOrLocalFont` · 005c4800 · legions 0058b8c0 — cited in ac2-ui text.rs:29 (UiTextData::font, font by DID with fallback)

### FontMapper
- [-] `FontMapper::Startup` · 005c4850 · legions 0058b910 — retail DB object cache handle/release; Arc per DID replaces (AGENTS.md exception)
- [-] `FontMapper::Shutdown` · 005c4890 · legions 0058b950 — retail DB object cache handle/release; Arc per DID replaces (AGENTS.md exception)
- [ ] `FontMapper::GetFontByDataID` · 005c48f0 · legions 0058b9b0 — needs port: font lookup by DID

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

## `portal/client/src/render/core/render.cpp.c` (1/27)

### Render
- [ ] `Render::HandleChange` · 005b30c0 · legions 0057da60 — needs port: graphics preference change handling (quality, degrade, device-independent effects)

### GraphicsPreferences
- [-] `GraphicsPreferences::GraphicsPreferences` · 005b30e0 · legions 0057da80 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [-] `GraphicsPreferences::~GraphicsPreferences` · 005b31f0 · legions 0057db90 — plain data ctor/dtor/operator=; Rust struct or Drop replaces

### Render
- [-] `Render::StartupDevice` · 005b3300 · legions 0057dca0 — D3D device lifecycle/restore; wgpu replaces
- [-] `Render::ShutdownDevice` · 005b3380 · legions 0057dd20 — D3D device lifecycle/restore; wgpu replaces
- [~] `Render::SetOverallGraphicsQuality` · 005b33b0 · legions 0057dd50 — ac2-render prefs::apply_quality: the level's values and defaults (levels 1-5, from the recovered preset table) reach the bank and LandscapeDrawDistance is readable; the renderer reads only the draw distance (far_radius_for_draw_distance), not wired into the live terrain loop; SaveOverallGraphicsQuality is separate
- [-] `Render::ShutdownRenderingSystem` · 005b3c40 · legions 0057e5e0 — D3D device lifecycle/restore; wgpu replaces
- [-] `Render::RestartDevice` · 005b3ca0 · legions 0057e640 — D3D device lifecycle/restore; wgpu replaces
- [-] `Render::CreateIndexBuffer` · 005b3d90 · legions 0057e730 — D3D device/resource plumbing; wgpu replaces
- [-] `Render::Startup` · 005b3de0 · legions 0057e780 — D3D device lifecycle/restore; wgpu replaces
- [-] `Render::Shutdown` · 005b3fb0 · legions 0057e950 — D3D device lifecycle/restore; wgpu replaces
- [~] `Render::InitRenderingPreferences` · 005b4160 · legions 0057eb00 — ac2-render prefs::register_graphics: 21 keys in section 0x40000004 and 2 in 0x4000000D, in Create order, with their enum labels; HighQualityUI has no Create here so it is not registered. The saved OverallGraphicsQuality (1-5) is applied at startup (ClientPrefs::apply_startup_quality); without one the client applies a GPU-chosen level (AGENTS.md exception) where retail kept the registered defaults
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
- [ ] `Render::UpdateOverallGraphicsQuality` · 005b5860 — needs port: overall graphics quality preset applied to the degrade levels
- [-] `Render::LinkRGRCallback` · 005b5e70 · legions 00580810 — D3D device lifecycle/restore; wgpu replaces
- [ ] `Render::HandleApply` · 005b5ee0 · legions 00580880 — needs port: graphics preference apply (quality, degrade, device-independent effects)

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
- [ ] `NormalMap::GrayScale::Calculate` · 005ca980 — needs port: normal map generation from a colour/height source (bump)

### NormalMap::Average
- [ ] `NormalMap::Average::Calculate` · 005ca9d0 — needs port: normal map generation from a colour/height source (bump)

### NormalMap::Red
- [ ] `NormalMap::Red::Calculate` · 005caa30 — needs port: normal map generation from a colour/height source (bump)

### NormalMap::Alpha
- [ ] `NormalMap::Alpha::Calculate` · 005caa50 — needs port: normal map generation from a colour/height source (bump)

### NormalMap::Green
- [ ] `NormalMap::Green::Calculate` · 005caa70 — needs port: normal map generation from a colour/height source (bump)

### NormalMap::Blue
- [ ] `NormalMap::Blue::Calculate` · 005caa90 — needs port: normal map generation from a colour/height source (bump)

### NormalMap::Maximum
- [ ] `NormalMap::Maximum::Calculate` · 005caac0 — needs port: normal map generation from a colour/height source (bump)

### NormalMap::Lightness
- [ ] `NormalMap::Lightness::Calculate` · 005cab40 — needs port: normal map generation from a colour/height source (bump)

### NormalMap::ColorSpace
- [ ] `NormalMap::ColorSpace::Calculate` · 005cac00 — needs port: normal map generation from a colour/height source (bump)

### RenderSurfaceD3D
- [-] `RenderSurfaceD3D::RenderSurfaceD3D` · 005cac70 · legions 00591c60 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [-] `RenderSurfaceD3D::Create` · 005cacb0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderSurfaceD3D::Destroy` · 005cad10 · legions 00591d00 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderSurfaceD3D::Lock` · 005cad70 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderSurfaceD3D::Unlock` · 005cae30 · legions 00591e20 — D3D device/resource plumbing; wgpu replaces

### NormalMap
- [ ] `NormalMap::Init` · 005caea0 · legions 00591e90 — needs port: normal map generation from a colour/height source (bump)
- [ ] `NormalMap::Convert24BitToNormalMap` · 005cafe0 · legions 00591fd0 — needs port: normal map generation from a colour/height source (bump)

### RenderSurfaceD3D
- [-] `RenderSurfaceD3D::~RenderSurfaceD3D` · 005cb3f0 · legions 005923e0 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [ ] `RenderSurfaceD3D::ConvertToNormalMap` · 005cb460 — needs port: surface converted to a normal map (bump)
- [-] `RenderSurfaceD3D::Convert` · 005cb5b0 — D3D device/resource plumbing; wgpu replaces

## `portal/client/src/render/core/surface/rendersurface.cpp.c` (1/18)

### RenderSurface
- [-] `RenderSurface::Begin` · 005b13f0 · legions 0057cea0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderSurface::Create` · 005b1450 · legions 0057cf00 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderSurface::Destroy` · 005b14f0 · legions 0057cfa0 — D3D device/resource plumbing; wgpu replaces
- [ ] `RenderSurface::InitLoad` · 005b1530 · legions 0057cfe0 — needs port: render surface pixel load/build from DAT bytes
- [-] `RenderSurface::Lock` · 005b1570 · legions 0057d010 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderSurface::SelectSurfaceFormat` · 005b15e0 · legions 0057d080 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderSurface::Convert` · 005b1690 · legions 0057d130 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderSurface::RenderSurface` · 005b1860 · legions 0057d300 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [-] `RenderSurface::GetDBOType` · 005b18c0 — DBO type tag; ac2-db owns type ids
- [-] `RenderSurface::~RenderSurface` · 005b18d0 · legions 0057d370 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [ ] `RenderSurface::CreateFromSourceData` · 005b1930 · legions 0057d3d0 — needs port: render surface pixel load/build from DAT bytes
- [x] `RenderSurface::StreamPack` · 005b1c40 · legions 0057d6d0 — RenderSurface::StreamPack cited in ac2-assets ac2_texture.rs decode_level (DID, category, size, format, bytes)
- [-] `RenderSurface::LoadDDS` · 005b1d60 — text source loader, admin build only
- [-] `RenderSurface::LoadJPGAsRawData` · 005b1eb0 — text source loader, admin build only
- [~] `RenderSurface::SaveJPG` · 005b1f70 · legions 0057d7f0 — JPEG via the image crate encoder (screenshot.rs encode_jpeg), fixed quality 90; retail's IJL quality sits in an undecoded jprops block
- [-] `RenderSurface::Allocator` · 005b2130 — custom allocator; Rust ownership
- [-] `RenderSurface::LoadFromFile` · 005b2170 — text source loader, admin build only
- [-] `RenderSurface::Unlock` · 005b2370 · legions 0057d9f0 — D3D device/resource plumbing; wgpu replaces

## `portal/client/src/render/core/surface/surfacewindow.cpp.c` (5/25)

### SurfaceWindow
- [-] `SurfaceWindow::End` · 005aed30 · legions 0057a7b0 — D3D device/resource plumbing; wgpu replaces
- [-] `SurfaceWindow::Attach` · 005aed40 · legions 0057a7c0 — D3D device/resource plumbing; wgpu replaces
- [-] `SurfaceWindow::Attach` · 005aede0 · legions 0057a860 — D3D device/resource plumbing; wgpu replaces
- [ ] `SurfaceWindow::WindowToClip` · 005aee20 · legions 0057a8a0 — needs port: window pixel to clip conversion (SurfaceWindow::WindowToClip)
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
- [~] `SurfaceWindow::DrawCharacter` · 005af390 · legions 0057ae10 — ac2-ui text.rs: no editing, cursor, selection, scrolling, fallback fonts, string variables or server downloads; A8R8G8B8 surfaces

### SmartArray<PrefDisplayModeType>
- [-] `SmartArray<PrefDisplayModeType>::AddToEnd` · 005af8f0 · legions 0057b430 — SmartArray/STL container internals; Vec replaces

### SurfaceWindow
- [-] `SurfaceWindow::Unlock` · 005af950 · legions 0057b490 — D3D device/resource plumbing; wgpu replaces
- [~] `SurfaceWindow::Blit` · 005af9c0 · legions 0057b500 — A8R8G8B8 only; colour key and format conversion not modelled
- [x] `SurfaceWindow::Multiply` · 005b00a0 · legions 0057bbd0 — ac2-ui icon_desc.rs multiply: each channel as byte/255 (RGBAColor::SetColor32's float) times the ARGB shift colour's, stored trunc(x * 255)
- [ ] `SurfaceWindow::Fill` · 005b0350 · legions 0057be70 — needs port: whole-surface colour fill
- [x] `SurfaceWindow::FillArea` · 005b0530 · legions 0057c050 — SurfaceWindow::FillArea cited in ac2-ui radar.rs:9 (blip pixels filled as areas)
- [x] `SurfaceWindow::FillArea` · 005b05e0 · legions 0057c100 — SurfaceWindow::FillArea cited in ac2-ui radar.rs:9 (blip pixels filled as areas)
- [x] `SurfaceWindow::DrawLine` · 005b0620 · legions 0057c140 — ac2-ui tree_view.rs
- [-] `SurfaceWindow::DrawBox` · 005b0910 — admin-only/tool/debug code, not in the client (no legions match; admin drawing) [vtable-checked: not a slot of any located Legions vtable]
- [ ] `SurfaceWindow::DrawTextA` · 005b09a0 · legions 0057c430 — needs port: text drawn onto a software surface (glyph coverage, colour)
- [-] `SurfaceWindow::NotColorBits` · 005b0b40 · legions 0057c5c0 — D3D device/resource plumbing; wgpu replaces
- [ ] `SurfaceWindow::GetColorFromPoint` · 005b0d80 · legions 0057c800 — needs port: read surface colour at a pixel

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

## `portal/client/src/render/core/texture/rendertexture.cpp.c` (1/29)

### RenderTexture
- [-] `RenderTexture::Create2D` · 005bcaa0 · legions 00586200 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderTexture::CreateCube` · 005bcaf0 · legions 00586260 — D3D device/resource plumbing; wgpu replaces
- [ ] `RenderTexture::ShouldDropHighDetail` · 005bcb40 · legions 005862b0 — needs port: render texture level handling (detail level, sub-objects)
- [ ] `RenderTexture::Refresh` · 005bcb60 · legions 005862d0 — needs port: render texture level handling (detail level, sub-objects)
- [-] `RenderTexture::SelectTextureFormat` · 005bcb90 · legions 00586300 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderTexture::ReleaseSourceLevelReferences` · 005bcc20 · legions 00586390 — retail DB object cache handle/release; Arc per DID replaces (AGENTS.md exception)
- [-] `RenderTexture::ReleaseSubObjects` · 005bcc60 · legions 005863d0 — retail DB object cache handle/release; Arc per DID replaces (AGENTS.md exception)
- [ ] `RenderTexture::GetDIDsForFilenames` · 005bcc80 — needs port: render texture level handling (detail level, sub-objects)
- [-] `RenderTexture::LoadLevelResources` · 005bcd20 — text source loader, admin build only
- [ ] `RenderTexture::ConstructTexture` · 005bcdd0 · legions 00586490 — needs port: render texture level handling (detail level, sub-objects)

### DBLevelInfo
- [-] `DBLevelInfo::DBLevelInfo` · 005bcf80 — plain data ctor/dtor/operator=; Rust struct or Drop replaces

### RenderTexture
- [ ] `RenderTexture::GetSubObjects` · 005bcfb0 · legions 00586650 — needs port: render texture level handling (detail level, sub-objects)

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
- [ ] `RenderTexture::SetNumSourceLevels` · 005bd4c0 · legions 005869d0 — needs port: render texture level handling (detail level, sub-objects)
- [ ] `RenderTexture::DropUnwantedLevels` · 005bd4f0 · legions 00586a00 — needs port: render texture level handling (detail level, sub-objects)
- [-] `RenderTexture::CopyInto` · 005bd600 · legions 00586b10 — D3D device/resource plumbing; wgpu replaces
- [ ] `RenderTexture::GetSubDataIDs` · 005bd750 — needs port: render texture level handling (detail level, sub-objects)
- [ ] `RenderTexture::InitLoad` · 005bd820 · legions 00586d40 — needs port: render texture level handling (detail level, sub-objects)
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

## `portal/client/src/render/core/vertexbuffer/rendervertexbuffer.cpp.c` (2/17)

### RenderVertexBuffer
- [-] `RenderVertexBuffer::Begin` · 005b6f90 · legions 005813c0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderVertexBuffer::End` · 005b6fb0 · legions 005813e0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderVertexBuffer::Startup` · 005b6fd0 · legions 00581400 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderVertexBuffer::Shutdown` · 005b7080 · legions 005814b0 — D3D device/resource plumbing; wgpu replaces
- [-] `RenderVertexBuffer::UnlockAnimVertices` · 005b70c0 · legions 005814f0 — D3D device/resource plumbing; wgpu replaces
- [ ] `RenderVertexBuffer::AnimateVertex` · 005b70e0 · legions 00581510 — needs port: CPU vertex animation of a vertex buffer
- [~] `RenderVertexBuffer::BlendVertices` · 005b7260 · legions 00581690 — BlendLitVertices on the GPU (ac2-gpu object.wgsl); no bounds update, ModelDetail<2 path not reproduced
- [ ] `RenderVertexBuffer::AnimateAllVertices` · 005b7530 · legions 00581c00 — needs port: CPU vertex animation of a vertex buffer
- [-] `RenderVertexBuffer::Unlock` · 005b7590 · legions 00581930 — D3D device/resource plumbing; wgpu replaces
- [x] `RenderVertexBuffer::IlluminateVertices` · 005b75b0 · legions 00581950
- [ ] `RenderVertexBuffer::RenderIndexBufferPrimitives` · 005b77a0 · legions 00581b40 — needs port: material stage draw of a vertex buffer (stage order, blend, fog)
- [ ] `RenderVertexBuffer::AnimateVertices` · 005b7860 — needs port: CPU vertex animation of a vertex buffer
- [ ] `RenderVertexBuffer::RenderIndexBufferUsingMaterial` · 005b7880 · legions 00581c50 — needs port: material stage draw of a vertex buffer (stage order, blend, fog)
- [x] `RenderVertexBuffer::RenderIndexedUsingMaterial` · 005b7a50 · legions 00581e10 — ac2-render object::pass / texture_matrix, ac2-gpu object
- [ ] `RenderVertexBuffer::RenderUsingMaterial` · 005b7c40 · legions 00581ff0 — needs port: material stage draw of a vertex buffer (stage order, blend, fog)
- [ ] `RenderVertexBuffer::RayIntersectsTriList` · 005b7e00 · legions 005821b0 — needs port: ray vs triangle list (picking)
- [-] `RenderVertexBuffer::LockAnimVertices` · 005b8450 · legions 00582800 — D3D device/resource plumbing; wgpu replaces

## `portal/client/src/render/decal/decaleffect.cpp.c` (7/9)

### DecalEffect
- [x] `DecalEffect::ClipPolygonAgainstPlane` · 005dd100 · legions 0059ddf0 — decal FX: ac2-render decal.rs (clip, fade, geometry per material), fx_decals.rs (spawn), scene/decals.rs (ground from resident terrain); tests: decal_tests, fx_decals_tests, real-data decode, headless draw
- [x] `DecalEffect::DecalEffect` · 005de3e0 · legions 0059f170 — decal FX: ac2-render decal.rs (clip, fade, geometry per material), fx_decals.rs (spawn), scene/decals.rs (ground from resident terrain); tests: decal_tests, fx_decals_tests, real-data decode, headless draw
- [x] `DecalEffect::Think` · 005de430 · legions 0059f1c0 — decal FX: ac2-render decal.rs (clip, fade, geometry per material), fx_decals.rs (spawn), scene/decals.rs (ground from resident terrain); tests: decal_tests, fx_decals_tests, real-data decode, headless draw
- [x] `DecalEffect::CreateGeometry` · 005de500 · legions 0059f290 — decal FX: ac2-render decal.rs (clip, fade, geometry per material), fx_decals.rs (spawn), scene/decals.rs (ground from resident terrain); tests: decal_tests, fx_decals_tests, real-data decode, headless draw
- [-] `DecalEffect::~DecalEffect` · 005de7b0 · legions 0059f540 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [x] `DecalEffect::ConstructDecal` · 005de7e0 · legions 0059f570 — decal FX: ac2-render decal.rs (clip, fade, geometry per material), fx_decals.rs (spawn), scene/decals.rs (ground from resident terrain); tests: decal_tests, fx_decals_tests, real-data decode, headless draw
- [x] `DecalEffect::ConstructLandscapeDecal` · 005dee50 · legions 0059fbe0 — decal FX: ac2-render decal.rs (clip, fade, geometry per material), fx_decals.rs (spawn), scene/decals.rs (ground from resident terrain); tests: decal_tests, fx_decals_tests, real-data decode, headless draw
- [~] `DecalEffect::ConstructDungeonDecal` · 005df1e0 · legions 0059ff70 — dungeon decal needs CEnvCell::GetNearbyGeometry; not ported, such nodes make no decal
- [x] `DecalEffect::Init` · 005df350 · legions 005a00e0 — decal FX: ac2-render decal.rs (clip, fade, geometry per material), fx_decals.rs (spawn), scene/decals.rs (ground from resident terrain); tests: decal_tests, fx_decals_tests, real-data decode, headless draw

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
- [ ] `GameRender::Init` · 005b84e0 — needs port: game render init
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

## `portal/client/src/render/particle/particleemitter.cpp.c` (24/40)

### ParticleEmitter
- [x] `ParticleEmitter::SetLastDischargeTime` · 005d07a0 · legions 00593e90 — ac2-render particles

### FillVerts
- [x] `FillVerts::GetNullRotationMatrix` · 005d07b0 — ac2-render particles
- [x] `FillVerts::GetWorldRotationMatrix` · 005d07f0 — ac2-render particles
- [x] `FillVerts::GetRotationMatrix` · 005d0840 — ac2-render particles
- [x] `FillVerts::GetVertexPositions` · 005d0880 · legions 00593f70 — ac2-render particles
- [~] `FillVerts::GetStreakedVertexPositions` · 005d0940 — ac2-render particles: unlit, no repulsors, no shared systems; the rotation carried across controller keys and the streak's diagonal branch are read from the decompile, not checked against retail output
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
- [-] `ParticleEmitter::DoRepulse` · 005d1cb0 · legions 005953a0 — placed systems never move; repulsors are not modelled
- [x] `ParticleEmitter::DoPointFile` · 005d2150 · legions 00595840 — ac2-render particles
- [x] `ParticleEmitter::DoPointFileLerp` · 005d2290 · legions 00595980 — ac2-render particles

### FillVerts
- [x] `FillVerts::GetOrigin` · 005d2550 · legions 00595c40 — ac2-render particles
- [x] `FillVerts::GetBlendedOrigin` · 005d2570 — ac2-render particles
- [x] `FillVerts::GetNullStreakOrigin` · 005d2600 — ac2-render particles
- [x] `FillVerts::GetStreakOrigin` · 005d2620 — ac2-render particles
- [x] `FillVerts::GetBlendedStreakOrigin` · 005d2640 — ac2-render particles
- [~] `FillVerts::GetLighting` · 005d26d0 — ac2-render particles: unlit, no repulsors, no shared systems; the rotation carried across controller keys and the streak's diagonal branch are read from the decompile, not checked against retail output

### ParticleEmitter
- [~] `ParticleEmitter::FillVertexBuffer` · 005d28a0 · legions 00595f90 — ac2-render particles: unlit, no repulsors, no shared systems; the rotation carried across controller keys and the streak's diagonal branch are read from the decompile, not checked against retail output
- [ ] `ParticleEmitter::SetPosition` · 005d3690 · legions 00596d80 — needs port: particle emitter (position, desc)
- [~] `ParticleEmitter::ApplyKeyframe` · 005d3850 · legions 00596f40 — ac2-render particles: unlit, no repulsors, no shared systems; the rotation carried across controller keys and the streak's diagonal branch are read from the decompile, not checked against retail output
- [x] `ParticleEmitter::ApplyKeyframes` · 005d3d10 · legions 00597400 — ac2-render particles
- [~] `ParticleEmitter::ParticleThink` · 005d3de0 · legions 005974d0 — ac2-render particles: unlit, no repulsors, no shared systems; the rotation carried across controller keys and the streak's diagonal branch are read from the decompile, not checked against retail output
- [~] `ParticleEmitter::Think` · 005d40a0 · legions 00597790 — ac2-render particles: unlit, no repulsors, no shared systems; the rotation carried across controller keys and the streak's diagonal branch are read from the decompile, not checked against retail output
- [~] `ParticleEmitter::DischargeParticle` · 005d42a0 · legions 00597990 — ac2-render particles: unlit, no repulsors, no shared systems; the rotation carried across controller keys and the streak's diagonal branch are read from the decompile, not checked against retail output
- [x] `ParticleEmitter::SetupKeyframes` · 005d4bb0 · legions 005982a0 — ac2-render particles
- [~] `ParticleEmitter::Initialize` · 005d4e30 · legions 00598520 — ac2-render particles: unlit, no repulsors, no shared systems; the rotation carried across controller keys and the streak's diagonal branch are read from the decompile, not checked against retail output
- [x] `ParticleEmitter::Reset` · 005d4ed0 · legions 005985c0 — ac2-render particles
- [~] `ParticleEmitter::EmitTimedParticles` · 005d4f00 · legions 005985f0 — ac2-render particles: unlit, no repulsors, no shared systems; the rotation carried across controller keys and the streak's diagonal branch are read from the decompile, not checked against retail output
- [-] `ParticleEmitter::EmitDistanceParticles` · 005d5280 · legions 00598970 — placed systems never move; repulsors are not modelled
- [~] `ParticleEmitter::EmitParticles` · 005d58a0 · legions 00598f90 — ac2-render particles: unlit, no repulsors, no shared systems; the rotation carried across controller keys and the streak's diagonal branch are read from the decompile, not checked against retail output

## `portal/client/src/render/particle/particlerepulsor.cpp.c` (0/4)

### ParticleRepulsor
- [ ] `ParticleRepulsor::ParticleRepulsor` · 005c3790 · legions 0058a8a0 — needs port: particle repulsor (push-away field, active state, position)
- [-] `ParticleRepulsor::GetIsReadyToDie` · 005c37f0 · legions 0058a900 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [-] `ParticleRepulsor::GetIsActive` · 005c3820 · legions 0058a930 — trivial accessor of plain particle/record field; Rust struct field replaces it
- [ ] `ParticleRepulsor::SetPosition` · 005c3870 · legions 0058a980 — needs port: particle repulsor (push-away field, active state, position)

## `portal/client/src/render/particle/particlesystem.cpp.c` (3/22)

### ParticleSystem
- [-] `ParticleSystem::AddRef` · 005c22a0 · legions 005894e0 — retail DB object cache handle/release; Arc per DID replaces (AGENTS.md exception)
- [ ] `ParticleSystem::SetCanBeShared` · 005c22b0 · legions 005894f0 — needs port: particle system (lifecycle, lighting origin, shared/queue behaviour)
- [x] `ParticleSystem::GetForceDraw` · 005c22c0 · legions 00589500 — ac2-render particles
- [ ] `ParticleSystem::Begin` · 005c22e0 · legions 00589520 — needs port: particle system (lifecycle, lighting origin, shared/queue behaviour)
- [x] `ParticleSystem::GetExternalForces` · 005c2350 · legions 00589590 — ParticleSystem::GetExternalForces cited in ac2-render particles/emitter.rs:60 (world gravity, no wind)

### Position
- [-] `Position::Position` · 005c2520 · legions 00475a40 — plain data ctor/dtor/operator=; Rust struct or Drop replaces

### ParticleSystem
- [ ] `ParticleSystem::CanBeShared` · 005c25b0 · legions 005897b0 — needs port: shared particle system check
- [x] `ParticleSystem::GetReadyToDie` · 005c25f0 · legions 005897f0 — ac2-render particles/emitter.rs ready_to_die and particles/mod.rs ready_to_die: the emitter's inactive, time-limit and emission-limit tests and the system's ten-thousand second wait, checked against the decompile
- [-] `ParticleSystem::Destroy` · 005c2680 · legions 00589880 — retail DB object cache handle/release; Arc per DID replaces (AGENTS.md exception)
- [~] `ParticleSystem::Start` · 005c2700 · legions 00589900 — ac2-render particles: unlit, no repulsors, no shared systems; the rotation carried across controller keys and the streak's diagonal branch are read from the decompile, not checked against retail output
- [~] `ParticleSystem::Think` · 005c2770 · legions 00589970 — ac2-render particles: unlit, no repulsors, no shared systems; the rotation carried across controller keys and the streak's diagonal branch are read from the decompile, not checked against retail output
- [ ] `ParticleSystem::GetLightingOrigin` · 005c28b0 · legions 00589a80 — needs port: particle system (lifecycle, lighting origin, shared/queue behaviour)

### ParticleSystem::ParticleRenderPos
- [-] `ParticleSystem::ParticleRenderPos::operator=` · 005c2910 · legions 00589ae0 — plain data ctor/dtor/operator=; Rust struct or Drop replaces

### ParticleSystem
- [ ] `ParticleSystem::ParticleSystem` · 005c2a40 · legions 00589c10 — needs port: particle system (lifecycle, lighting origin, shared/queue behaviour)
- [-] `ParticleSystem::~ParticleSystem` · 005c2ad0 · legions 00589ca0 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [-] `ParticleSystem::Release` · 005c2b30 · legions 00589d00 — retail DB object cache handle/release; Arc per DID replaces (AGENTS.md exception)
- [~] `ParticleSystem::FillVertexBuffer` · 005c2bf0 · legions 00589dc0 — ac2-render particles: unlit, no repulsors, no shared systems; the rotation carried across controller keys and the streak's diagonal branch are read from the decompile, not checked against retail output
- [~] `ParticleSystem::RenderAtPosition` · 005c2e80 · legions 0058a050 — ac2-render particles: unlit, no repulsors, no shared systems; the rotation carried across controller keys and the streak's diagonal branch are read from the decompile, not checked against retail output
- [~] `ParticleSystem::Render` · 005c3250 · legions 0058a420 — ac2-render particles: unlit, no repulsors, no shared systems; the rotation carried across controller keys and the streak's diagonal branch are read from the decompile, not checked against retail output

### SmartArray<ParticleSystem::ParticleRenderPos>
- [-] `SmartArray<ParticleSystem::ParticleRenderPos>::AddToEnd` · 005c3450 · legions 0058a560 — SmartArray/STL container internals; Vec replaces

### ParticleSystem
- [~] `ParticleSystem::Create` · 005c34e0 · legions 0058a5f0 — ac2-render particles: unlit, no repulsors, no shared systems; the rotation carried across controller keys and the streak's diagonal branch are read from the decompile, not checked against retail output
- [~] `ParticleSystem::Queue` · 005c3600 · legions 0058a710 — ac2-render particles: unlit, no repulsors, no shared systems; the rotation carried across controller keys and the streak's diagonal branch are read from the decompile, not checked against retail output

## `portal/client/src/render/particle/psmanager.cpp.c` (2/15)

### PSManager
- [ ] `PSManager::Startup` · 005c0540 · legions 005877e0 — needs port: particle system manager (register/create/destroy systems and repulsors)

### PSManager::AutonomousSystemInfo
- [-] `PSManager::AutonomousSystemInfo::AutonomousSystemInfo` · 005c0670 — plain data ctor/dtor/operator=; Rust struct or Drop replaces

### PSManager
- [ ] `PSManager::DestroyRepulsor` · 005c0700 · legions 005879a0 — needs port: particle system manager (register/create/destroy systems and repulsors)
- [ ] `PSManager::CreateParticleSystem` · 005c0760 · legions 00587a00 — needs port: particle system manager (register/create/destroy systems and repulsors)
- [ ] `PSManager::UnregisterSystem` · 005c0850 · legions 00587ae0 — needs port: particle system manager (register/create/destroy systems and repulsors)

### PSManager::AutonomousSystemInfo
- [-] `PSManager::AutonomousSystemInfo::operator=` · 005c0890 · legions 00587b20 — plain data ctor/dtor/operator=; Rust struct or Drop replaces

### SmartArray<PSManager::AutonomousSystemInfo>
- [-] `SmartArray<PSManager::AutonomousSystemInfo>::operator=` · 005c09c0 · legions 00587c50 — SmartArray/STL container internals; Vec replaces

### PSManager
- [ ] `PSManager::UnregisterAutonomousSystem` · 005c0ae0 · legions 00587d70 — needs port: particle system manager (register/create/destroy systems and repulsors)

### SmartArray<PSManager::AutonomousSystemInfo>
- [-] `SmartArray<PSManager::AutonomousSystemInfo>::AddToEnd` · 005c0b70 · legions 00587e00 — SmartArray/STL container internals; Vec replaces

### PSManager
- [ ] `PSManager::Shutdown` · 005c0c00 · legions 00587e90 — needs port: particle system manager (register/create/destroy systems and repulsors)
- [ ] `PSManager::CreateRepulsor` · 005c0d20 · legions 00587fb0 — needs port: particle system manager (register/create/destroy systems and repulsors)
- [x] `PSManager::RenderParticleSystems` · 005c0db0 · legions 00588040 — PSManager::RenderParticleSystems cited in ac2-render scene/mod.rs:83 (particles merged per material)
- [ ] `PSManager::RegisterSystem` · 005c0e60 · legions 005880f0 — needs port: particle system manager (register/create/destroy systems and repulsors)
- [x] `PSManager::RegisterAutonomousSystem` · 005c0ec0 · legions 00588150 — PSManager::RegisterAutonomousSystem cited in ac2-render scene/particles.rs:6 (keep last place until ready to die)
- [~] `PSManager::CreateAutonomousParticleSystem` · 005c0f80 · legions 00588210 — ac2-render scene/particles.rs follow=false path: a system placed once at the object's frame when first drawn; the autonomous registry and the water-plane frame are not ported

## `portal/client/src/render/streak/streakeffect.cpp.c` (6/10)

### StreakEffect
- [x] `StreakEffect::Activate` · 005c13d0 · legions 00588640 — ac2-render streak.rs: StreakEffect state (Activate/Deactivate, SetPosition stamps, Think cut/merge/fade, CreateGeometry ribbon strip) and the manager's create/think/geometry; StreakFX stream decode in ac2-assets ac2_fx.rs (real-data test pending the visual override scan)
- [x] `StreakEffect::Deactivate` · 005c13f0 · legions 00588660 — ac2-render streak.rs: StreakEffect state (Activate/Deactivate, SetPosition stamps, Think cut/merge/fade, CreateGeometry ribbon strip) and the manager's create/think/geometry; StreakFX stream decode in ac2-assets ac2_fx.rs (real-data test pending the visual override scan)
- [~] `StreakEffect::Render` · 005c1410 · legions 00588680 — emitter's own streaks start/stop/draw; holding-point child objects (CPhysicsObj children by holding id) not walked; ribbon drawn as a triangle list through the particle material path (not a strip draw); channel and day/night filters of DoTargettedFX not applied to streak nodes

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

## `portal/client/src/render/streak/streakeffectmanager.cpp.c` (3/6)

### StreakEffectManager
- [-] `StreakEffectManager::~StreakEffectManager` · 005c1040 · legions 005882b0 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [ ] `StreakEffectManager::DestroyStreakEffect` · 005c10d0 · legions 00588340 — needs port: streak effect (streak segments, fade, geometry)
- [x] `StreakEffectManager::UpdateStreakEffects` · 005c1150 · legions 005883c0 — ac2-render streak.rs: StreakEffect state (Activate/Deactivate, SetPosition stamps, Think cut/merge/fade, CreateGeometry ribbon strip) and the manager's create/think/geometry; StreakFX stream decode in ac2-assets ac2_fx.rs (real-data test pending the visual override scan)
- [x] `StreakEffectManager::RenderStreakEffects` · 005c11a0 · legions 00588410 — ac2-render fx_streaks.rs (DoStreakFX, DoTargettedFX's start/stop flags, running-FX guard, pref); ac2-render streak_objects.rs (emitter store, placement, think); drawn in the one particle pass
- [x] `StreakEffectManager::CreateStreakEffect` · 005c1240 · legions 005884b0 — ac2-render streak.rs: StreakEffect state (Activate/Deactivate, SetPosition stamps, Think cut/merge/fade, CreateGeometry ribbon strip) and the manager's create/think/geometry; StreakFX stream decode in ac2-assets ac2_fx.rs (real-data test pending the visual override scan)
- [ ] `StreakEffectManager::StreakEffectManager` · 005c12d0 · legions 00588540 — needs port: streak effect manager init

## `portal/client/src/scenerender/frustum.cpp.c` (3/7)

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
- [ ] `Frustum::CheckPolygon` · 005733e0 · legions 005433e0 — needs port: frustum vs polygon test

## `portal/client/src/scenerender/scenecamera.cpp.c` (6/11)

### SceneCamera
- [-] `SceneCamera::~SceneCamera` · 00572820 · legions 0053a560 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [x] `SceneCamera::ApplyProperty` · 00572840 — pregame 3D scenes: ac2-assets ac2_ui_scene.rs, ac2-client pregame_scene/, ac2-ui pregame/scene.rs
- [ ] `SceneCamera::BuildOptionalProperties` · 005728f0 — needs port: scene camera properties
- [x] `SceneCamera::SetFOV` · 00572aa0 · legions 0053a7e0
- [ ] `SceneCamera::SetVerticalScale` · 00572ab0 · legions 0053a7f0 — needs port: scene camera properties
- [x] `SceneCamera::slerp` · 00572ac0 · legions 0053a800
- [x] `SceneCamera::SceneCamera` · 00572b00 · legions 0053a840 — SceneCamera::SceneCamera defaults cited in ac2-client ui_scene/data.rs:51 (perspective, 65, 1, 0.1, 2000)
- [ ] `SceneCamera::ApplyPropertyCollection` · 00572bc0 — needs port: scene camera properties
- [x] `SceneCamera::SetPreviousPosition` · 00572c70 · legions 0053a9b0
- [ ] `SceneCamera::SetRenderPosition` · 00572c90 · legions 0053a9d0 — needs port: scene camera properties
- [x] `SceneCamera::SetCurrentPosition` · 00572cb0 · legions 0053a9f0

## `portal/client/src/scenerender/scenetool.cpp.c` (20/100)

### SceneTool
- [-] `SceneTool::UsingSceneMotionBlur` · 0056a5d0 · legions 0053aa10 — scene motion blur (m_fBlurAmount from CameraFX mBlur): no shipped FX node names a blur CameraFX (blur.cfx 0x55000002 is unreferenced; ac2-assets ac2_fx_tests lists the 22 camera FX nodes), so the blend never runs with the DATs

### MaterialLayer
- [ ] `MaterialLayer::SetDiffuse` · 0056a600 · legions 0053aa40 — needs port: material layer stage order and diffuse colour

### SceneTool
- [-] `SceneTool::StartupDebugHUD` · 0056a630 · legions 0053aa70 — admin-only/tool/debug code, not in the client (debug HUD)
- [-] `SceneTool::ShowDebugHUD` · 0056a7f0 — admin-only/tool/debug code, not in the client (SceneTool::ShowDebugHUD: test notes no handler in Legions) [vtable-checked: not a slot of any located Legions vtable]
- [x] `SceneTool::SetGammaBrightness` · 0056a810 · legions 0053ac30 — ac2-gpu ToneMap::set_brightness: the world's display levels times 1 + 2 * ScreenBrightness (RenderDeviceD3D::SetGamma's ramp), set each frame from SceneOptions::brightness; the UI is left as authored (AGENTS.md)
- [ ] `SceneTool::InitFromGlobalPreferences` · 0056a830 · legions 0053ac50 — needs port: scene tool InitFromGlobalPreferences
- [-] `SceneTool::InitTextureTargets` · 0056a910 · legions 0053ad30 — D3D device/resource plumbing; wgpu replaces
- [ ] `SceneTool::SetWorldCellID` · 0056aac0 · legions 0053aee0 — needs port: scene SetWorldCellID step

### (free functions)
- [ ] `SwapWorldMatrixForwardAndUp` · 0056ab00 · legions 0053af20 — needs port: forward/up axis swap of matrices
- [ ] `SwapViewMatrixForwardAndUp` · 0056ab90 · legions 0053afb0 — needs port: forward/up axis swap of matrices

### SceneTool
- [-] `SceneTool::SetWorldMatrix` · 0056ac20 · legions 0053b040 — D3D device/resource plumbing; wgpu replaces
- [-] `SceneTool::SetWorldTransform` · 0056ac70 · legions 0053b090 — D3D device/resource plumbing; wgpu replaces
- [x] `SceneTool::SetMatrixStack` · 0056ad20 · legions 0053b140 — ac2-gpu ObjectInstance.pose (part matrices, bone storage buffer)
- [x] `SceneTool::TransformVector` · 0056ad40 · legions 0053b160 — SceneTool::TransformVector cited in ac2-client view.rs:443 (box centre projected through the view)
- [x] `SceneTool::TransformPixelToClipPoint` · 0056af60 · legions 0053b380 — SceneTool::TransformPixelToClipPoint cited in ac2-ui window_locations.rs:5 (pixel_to_clip)
- [x] `SceneTool::TransformClipPointToPixel` · 0056afb0 · legions 0053b3d0 — ac2-ui reticle.rs + ac2-gpu ui.rs: the targeting reticle
- [ ] `SceneTool::ViewportToClip` · 0056b010 · legions 0053b430 — needs port: scene ViewportToClip step
- [ ] `SceneTool::ClipToViewport` · 0056b080 · legions 0053b4a0 — needs port: scene ClipToViewport step

### LightSorterPredicate
- [ ] `LightSorterPredicate::operator()` · 0056b100 · legions 0053b520 — needs port: light sort predicate (which lights are kept)

### SceneTool
- [x] `SceneTool::SetAmbientLight` · 0056b150 · legions 0053b570 — ac2-render object::pass / texture_matrix, ac2-gpu object
- [x] `SceneTool::SetOutdoorAmbient` · 0056b170 · legions 0053b590 — ac2-render object::pass / texture_matrix, ac2-gpu object
- [x] `SceneTool::SetFogProperties` · 0056b1a0 · legions 0053b5c0 — SceneTool::SetFogProperties cited in ac2-client ui_scene/data.rs:92 (want_fog, colour, near, far applied in camera.rs)
- [x] `SceneTool::IlluminateMeshVertices` · 0056b1c0 · legions 0053b5e0
- [-] `SceneTool::PrepareGenericRenderTarget0` · 0056b1e0 · legions 0053b600 — D3D device/resource plumbing; wgpu replaces
- [-] `SceneTool::PrepareGenericRenderTarget1` · 0056b250 · legions 0053b670 — D3D device/resource plumbing; wgpu replaces
- [-] `SceneTool::PrepareGenericRenderTarget2` · 0056b2c0 · legions 0053b6e0 — D3D device/resource plumbing; wgpu replaces
- [-] `SceneTool::PrepareGenericDepthTarget` · 0056b330 · legions 0053b750 — D3D device/resource plumbing; wgpu replaces
- [~] `SceneTool::GenerateGlobalShadowVolume` · 0056b390 · legions 0053b7b0 — ac2_gpu sun_shadow.rs + object_sun_casters.rs: DynamicObjectShadows levels 1-3 draw cascaded sun depth maps (bodies and near static objects cast, posed and skinned; terrain/objects/bodies receive with PCF at the static strength) in place of disc decals and stencil volumes (AGENTS.md exception)
- [-] `SceneTool::StartScreenshotCapture` · 0056b6b0 · legions 0053ba90 — tiled re-render capture not ported; the presented frame is copied instead
- [-] `SceneTool::FinishScreenshotCapture` · 0056b7a0 · legions 0053bb80 — tiled re-render capture not ported; the presented frame is copied instead
- [-] `SceneTool::GetNumSubScreenshots` · 0056b7c0 · legions 0053bba0 — tiled re-render capture not ported; the presented frame is copied instead
- [-] `SceneTool::FinishSubScreenshot` · 0056b7d0 · legions 0053bbb0 — tiled re-render capture not ported; the presented frame is copied instead
- [ ] `SceneTool::StartReflectionMap` · 0056ba00 · legions 0053bde0 — needs port: water reflection map render
- [ ] `SceneTool::FinishReflectionMap` · 0056ba60 · legions 0053be40 — needs port: water reflection map render
- [ ] `SceneTool::StartWaterBumpMap` · 0056baa0 · legions 0053be80 — needs port: water bump map render
- [ ] `SceneTool::FinishWaterBumpMap` · 0056bb20 · legions 0053bf00 — needs port: water bump map render
- [-] `SceneTool::IdentityMatrices` · 0056bb50 · legions 0053bf30 — D3D device/resource plumbing; wgpu replaces
- [-] `SceneTool::StoreMatrices` · 0056bc90 · legions 0053c070 — D3D device/resource plumbing; wgpu replaces
- [-] `SceneTool::RestoreMatrices` · 0056bcd0 · legions 0053c0b0 — D3D device/resource plumbing; wgpu replaces
- [ ] `SceneTool::SetBackgroundColor` · 0056bd90 · legions 0053c170 — needs port: background clear colour
- [ ] `SceneTool::StartFrame` · 0056bdd0 · legions 0053c1b0 — needs port: scene StartFrame step
- [x] `SceneTool::CheckObjectSelection` · 0056be60 · legions 0053c240 — ac2-render selection.rs + ac2-ui selection_ui.rs: clicking the world selects
- [ ] `SceneTool::ShouldSkipRenderNow` · 0056bf90 · legions 0053c370 — needs port: frame skip rule while the scene is busy
- [ ] `SceneTool::SetSelectionID` · 0056bfb0 · legions 0053c390 — needs port: selection id drawn for picking
- [-] `SceneTool::GetDistortionMap` · 0056bfe0 · legions 0053c3c0 — no shipped material names the Distortion special texture or a BUMPENVMAP stage (Legions DAT: 0 of 2401 materials); one samples the frame buffer (0x31000323)
- [-] `SceneTool::GetFrameBufferTexture` · 0056c270 · legions 0053c640 — D3D device/resource plumbing; wgpu replaces
- [-] `SceneTool::PurgeOldGraphicsResources` · 0056c290 · legions 0053c660 — D3D device lifecycle/restore; wgpu replaces

### BBox
- [ ] `BBox::MakeValid` · 0056c570 · legions 0053c950 — needs port: bounding box validity (MakeValid)

### SceneTool
- [x] `SceneTool::SetupView` · 0056c5d0 · legions 0053c9b0 — SceneTool::SetupView cited in ac2-client ui_scene/camera.rs:4 (view from entity frame along +Y)
- [ ] `SceneTool::SetupPerspectiveProjection` · 0056c730 · legions 0053cb10 — needs port (check): Rust code with a similar name exists but does not cite this function; verify and cite, or port
- [ ] `SceneTool::SetupPerpspectiveFromFrustum` · 0056c880 · legions 0053cc60 — needs port: scene SetupPerpspectiveFromFrustum step
- [ ] `SceneTool::SetupOrthographicProjection` · 0056c980 · legions 0053cd60 — needs port (check): Rust code with a similar name exists but does not cite this function; verify and cite, or port
- [ ] `SceneTool::SetupOrthographicProjection` · 0056ca70 · legions 0053ce50 — needs port (check): Rust code with a similar name exists but does not cite this function; verify and cite, or port
- [ ] `SceneTool::InverseRotate` · 0056cb90 · legions 0053cf70 — needs port: scene InverseRotate step
- [ ] `SceneTool::InverseTransform` · 0056cd40 · legions 0053d120 — needs port: scene InverseTransform step
- [ ] `SceneTool::GetLight` · 0056cee0 · legions 0053d2c0 — needs port: scene GetLight step
- [ ] `SceneTool::UnlinkAllLights` · 0056cf20 · legions 0053d300 — needs port: scene UnlinkAllLights step
- [ ] `SceneTool::DrawLine` · 0056d000 · legions 0053d3e0 — needs port: scene DrawLine primitive (verify client use before porting)
- [ ] `SceneTool::DrawTexturedTriangle` · 0056d0b0 · legions 0053d490 — needs port: scene DrawTexturedTriangle primitive (verify client use before porting)
- [ ] `SceneTool::DrawTriangle` · 0056d1e0 · legions 0053d5c0 — needs port: scene DrawTriangle primitive (verify client use before porting)
- [ ] `SceneTool::DrawTexturedPolygon` · 0056d240 · legions 0053d620 — needs port: DrawTexturedPolygon draw
- [ ] `SceneTool::BuildCylinderMesh` · 0056d2b0 · legions 0053d690 — needs port: BuildCylinderMesh (scene shape mesh)
- [ ] `SceneTool::BuildSphereMesh` · 0056dc60 · legions 0053e040 — needs port: BuildSphereMesh (scene shape mesh)
- [~] `SceneTool::StartSelection` · 0056e450 · legions 0053e830 — ac2-render selection.rs: drawn-mesh triangles (skinned parts at their group matrix), no potential list, no selectability / LookAtSelection / own-item checks
- [ ] `SceneTool::DrawScreenQuadUsingMaterial` · 0056e5f0 · legions 0053e9d0 — needs port: DrawScreenQuadUsingMaterial draw
- [ ] `SceneTool::Think` · 0056e860 · legions 0053ec40 — needs port: scene Think step
- [x] `SceneTool::GetPixelScanResult` · 0056e880 · legions 0053ec60 — only LensFlareEffect::Draw scanned pixels; replaced by ac2-gpu lens_flare.rs's depth scan pass (AGENTS.md exception)
- [x] `SceneTool::SetupCamera` · 0056ea20 · legions 0053ee00 — SceneTool::SetupCamera cited in ac2-client ui_scene/camera.rs:3 (viewport renders through camera entity)
- [ ] `SceneTool::GetPointLightShadowVector` · 0056ed50 · legions 0053f130 — needs port: shadow volumes and sunlight disable (shadow mapping of the scene)
- [ ] `SceneTool::DisableSunlight` · 0056eef0 · legions 0053f2d0 — needs port: shadow volumes and sunlight disable (shadow mapping of the scene)
- [x] `SceneTool::SetSunlight` · 0056ef40 · legions 0053f320 — object::ObjectLights; static shadow strength SceneLighting::shadow_strength drives the landscape map and the sun shadow cascades (ac2_gpu sun_shadow.rs); m_cVolumetricShadowStrength's undimmed colour is not used (cascades take the static strength)
- [ ] `SceneTool::DrawLine` · 0056f230 · legions 0053f610 — needs port: scene DrawLine primitive (verify client use before porting)
- [~] `SceneTool::DrawScreenQuad` · 0056f260 · legions 0053f640 — untextured colour quad (ac2-gpu ScreenQuad) only
- [-] `SceneTool::SwitchTextureTargets` · 0056f2e0 · legions 0053f6c0 — scene motion blur (m_fBlurAmount from CameraFX mBlur): no shipped FX node names a blur CameraFX (blur.cfx 0x55000002 is unreferenced; ac2-assets ac2_fx_tests lists the 22 camera FX nodes), so the blend never runs with the DATs
- [~] `SceneTool::Render` · 0056f440 · legions 0053f820 — ac2-gpu tonemap.wgsl/tonemap.rs set_flash: the camera effects' flash (m_cFlashColor) blended over the world's display levels by its alpha before the brightness ramp and the UI; streaks are ac2-render fx_streaks

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
- [~] `SceneTool::RenderGlobalVolumetricShadows` · 005703e0 · legions 00540820 — ac2_gpu sun_shadow.rs + object_sun_casters.rs: DynamicObjectShadows levels 1-3 draw cascaded sun depth maps (bodies and near static objects cast, posed and skinned; terrain/objects/bodies receive with PCF at the static strength) in place of disc decals and stencil volumes (AGENTS.md exception)
- [~] `SceneTool::FinishSelection` · 005706d0 · legions 00540b10 — ac2-render selection.rs: drawn-mesh triangles (skinned parts at their group matrix), no potential list, no selectability / LookAtSelection / own-item checks
- [~] `SceneTool::SelectObject` · 00570730 · legions 00540b70 — ac2-render selection.rs: drawn-mesh triangles (skinned parts at their group matrix), no potential list, no selectability / LookAtSelection / own-item checks

### SmartArray<PixelScanRequest>
- [-] `SmartArray<PixelScanRequest>::AddToEnd` · 005708c0 · legions 00540d00 — SmartArray/STL container internals; Vec replaces

### MaterialLayer
- [ ] `MaterialLayer::InsertStage` · 00570970 · legions 00540e20 — needs port: material layer stage order and diffuse colour

### RenderMaterial
- [ ] `RenderMaterial::InsertLayer` · 00570990 · legions 00540e40 — needs port: material layer insertion order

### SceneTool
- [ ] `SceneTool::CreateDefaultMaterial` · 005709b0 · legions 00540e60 — needs port: scene CreateDefaultMaterial step
- [-] `SceneTool::Startup` · 00570f00 · legions 005413b0 — D3D device lifecycle/restore; wgpu replaces
- [x] `SceneTool::LinkLight` · 00571630 · legions 00541ae0 — SceneTool::LinkLight cited in ac2-client ui_scene/camera.rs:13 (scene lights linked, sun disabled)
- [ ] `SceneTool::AddDynamicLights` · 005716c0 · legions 00541b70 — needs port: scene AddDynamicLights step
- [x] `SceneTool::RequestPixelScan` · 00571860 · legions 00541d10 — only LensFlareEffect::Draw scanned pixels; replaced by ac2-gpu lens_flare.rs's depth scan pass (AGENTS.md exception)
- [ ] `SceneTool::ScanRequestedPixels` · 005718c0 · legions 00541d70 — needs port: pixel scan (colour/ID under a screen pixel)
- [ ] `SceneTool::EndFrame` · 00571a40 · legions 00541ef0 — needs port: scene EndFrame step
- [x] `SceneTool::SortLights` · 00571da0 · legions 00542250 — ac2-render object::cell / point_lights
- [~] `SceneTool::CacheStaticLights` · 00571ec0 · legions 00542370 — static point lights: weight sort, six cap, range test, indoor/outdoor; sunlight-type and dynamic lights not yet
- [~] `SceneTool::ApplyLights` · 00572110 · legions 005425c0 — outdoor sun only; dynamic/point lights and shadow direction not yet
- [ ] `SceneTool::GetLightsForPosition` · 005726b0 · legions 00542b60 — needs port: scene GetLightsForPosition step

## `portal/client/src/smartbox/worldrender/cellrender.cpp.c` (0/2)

### CellRender
- [ ] `CellRender::CellRender` · 00583480 · legions 00552bf0 — needs port: cell render init
- [~] `CellRender::AddCellSounds` · 005834d0 · legions 00552c40 — ac2-audio/ac2-client ambient: four-copy contin spread unused by Legions; interior visibility from the player's stab list; groove objects read every update

## `portal/client/src/smartbox/worldrender/frillrender.cpp.c` (0/24)

### FrillRender
- [-] `FrillRender::OnContextFinished` · 00580990 — D3D device lifecycle/restore; wgpu replaces

### FrillEntry
- [-] `FrillEntry::operator=` · 005809e0 · legions 00550290 — plain data ctor/dtor/operator=; Rust struct or Drop replaces

### FrillRender
- [-] `FrillRender::DestroyFrillMesh` · 00580c30 · legions 00550430 — retail DB object cache handle/release; Arc per DID replaces (AGENTS.md exception)
- [ ] `FrillRender::ClearFrillMesh` · 00580cb0 · legions 005504b0 — needs port: frill mesh render (frill entries, positions, lists)
- [ ] `FrillRender::InitFrillMesh` · 005810b0 · legions 005508b0 — needs port: frill mesh render (frill entries, positions, lists)
- [ ] `FrillRender::RenderFrillMesh` · 00581180 · legions 00550980 — needs port: frill mesh render (frill entries, positions, lists)

### FrillListType
- [-] `FrillListType::~FrillListType` · 00581800 · legions 00551000 — plain data ctor/dtor/operator=; Rust struct or Drop replaces

### HashIterator<unsigned_long,BaseList<unsigned_long,11>_>
- [-] `HashIterator<unsigned_long,BaseList<unsigned_long,11>_>::next` · 00581840 · legions 00536680 — MSVC/Windows assertion or iterator internals

### SmartArray<RGBAColor>
- [-] `SmartArray<RGBAColor>::AddToEnd` · 00581880 · legions 00551040 — SmartArray/STL container internals; Vec replaces

### FrillRender
- [ ] `FrillRender::GetSubDataIDs` · 005818e0 · legions 005510a0 — needs port: frill mesh render (frill entries, positions, lists)
- [ ] `FrillRender::ClearAllRenderLists` · 00581990 · legions 00551150 — needs port: frill render lists cleared per frame

### FrillRenderType
- [~] `FrillRenderType::AddPos` · 00581a00 · legions 005511c0 — frills selected by radius around the camera cell; radius is the constructor default (script sets it)

### SmartArray<FrillEntry>
- [-] `SmartArray<FrillEntry>::AddToEnd` · 00581be0 · legions 005513a0 — SmartArray/STL container internals; Vec replaces

### FrillRender
- [ ] `FrillRender::Clear` · 00581d00 · legions 005514c0 — needs port: frill mesh render (frill entries, positions, lists)
- [ ] `FrillRender::RemoveOldFrillObjects` · 00581e90 · legions 00551650 — needs port: frill mesh render (frill entries, positions, lists)

### FrillListType
- [ ] `FrillListType::Add` · 00582010 · legions 005517d0 — needs port: frill list add

### FrillRender
- [-] `FrillRender::~FrillRender` · 00582170 · legions 00551930 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [ ] `FrillRender::EnableFrillObjects` · 00582220 · legions 005519d0 — needs port: frill mesh render (frill entries, positions, lists)
- [~] `FrillRender::AddNewFrillObjects` · 005823e0 · legions 00551b90 — frills selected by radius around the camera cell; radius is the constructor default (script sets it)
- [ ] `FrillRender::FrillRender` · 00582670 · legions 00551e20 — needs port: frill render init
- [ ] `FrillRender::UpdateFrillPosition` · 00582760 · legions 00551f00 — needs port: frill mesh render (frill entries, positions, lists)
- [ ] `FrillRender::Render` · 00582890 · legions 00552030 — needs port: frill mesh render (frill entries, positions, lists)
- [~] `FrillRender::RefreshFrillMesh` · 005828e0 · legions 00552050 — ac2-render frill::select_frills + per-instance frill normal and landscape self-shadow; drawn as instances, each near frill jittered FrillSettings::copies times (scene/grass.rs; AGENTS.md exception)
- [~] `FrillRender::UpdateRenderList` · 005832a0 · legions 00552a10 — frills selected by radius around the camera cell; radius is the constructor default (script sets it)

## `portal/client/src/smartbox/worldrender/lscapelighting.cpp.c` (14/19)

### RGBAColor
- [ ] `RGBAColor::Normalize` · 0058f080 · legions 0055e6f0 — needs port: colour normalise (clamp channels)
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

## `portal/client/src/smartbox/worldrender/lscapematerialinfo.cpp.c` (11/14)

### RenderTextureLocal
- [ ] `RenderTextureLocal::GetGlobalOrLocal` · 00590c10 · legions 00560260 — needs port: render texture global-or-local lookup

### LScapeMaterialInfo
- [-] `LScapeMaterialInfo::~LScapeMaterialInfo` · 00590c60 · legions 005602b0 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [x] `LScapeMaterialInfo::CornerMatch` · 00590c80 · legions 005602d0
- [x] `LScapeMaterialInfo::GetCombineMaterials` · 00590cb0 · legions 00560300
- [x] `LScapeMaterialInfo::GetAlphaTexture` · 00590d80 · legions 005603d0
- [x] `LScapeMaterialInfo::CalcUVInfo` · 00590de0 · legions 00560410
- [x] `LScapeMaterialInfo::Match` · 00590ea0 · legions 005604d0
- [x] `LScapeMaterialInfo::CalcRotUVs` · 00591020 · legions 00560650
- [ ] `LScapeMaterialInfo::LScapeMaterialInfo` · 005910d0 · legions 00560700 — needs port: landscape material info (layers, UVs)
- [x] `LScapeMaterialInfo::SetUVs` · 005911e0 · legions 00560810
- [x] `LScapeMaterialInfo::Apply` · 00591270 · legions 005608a0
- [x] `LScapeMaterialInfo::CombineMaterials` · 005912d0 · legions 00560900 — terrain.wgsl: base/alpha stages, shadow-map and cloud-shadow modulate before fog
- [x] `LScapeMaterialInfo::MultiCombine` · 005919d0 · legions 00561000
- [x] `LScapeMaterialInfo::Create` · 00591a40 · legions 00561070

## `portal/client/src/smartbox/worldrender/lscaperender.cpp.c` (10/39)

### LScapeLighting
- [x] `LScapeLighting::GetAmbientLight` · 005835c0 · legions 00552d30

### LScapeRender
- [-] `LScapeRender::RefreshGraphicsResources` · 00583630 · legions 00552da0 — D3D device lifecycle/restore; wgpu replaces
- [ ] `LScapeRender::ReinitializeTerrain` · 00583650 · legions 00552dc0 — needs port: landscape render (terrain draw, water, cell and block culling, lighting)
- [ ] `LScapeRender::ResetWorldState` · 005836b0 · legions 00552e20 — needs port: landscape render (terrain draw, water, cell and block culling, lighting)
- [ ] `LScapeRender::IsCameraInWater` · 005836d0 · legions 00552e40 — needs port: landscape render (terrain draw, water, cell and block culling, lighting)
- [x] `LScapeRender::DrawRain` · 00583700 · legions 00552e70 — rain amount and wind; streaks drawn as the rain material's particle batch above 0.4
- [ ] `LScapeRender::DrawTerrain` · 00583850 — needs port: landscape render (terrain draw, water, cell and block culling, lighting)
- [ ] `LScapeRender::DrawWater` · 00583860 — needs port: landscape render (terrain draw, water, cell and block culling, lighting)
- [ ] `LScapeRender::SetDayLighting` · 00583870 · legions 00552fe0 — needs port: landscape render (terrain draw, water, cell and block culling, lighting)
- [ ] `LScapeRender::SetDayFog` · 00583880 — needs port: landscape render (terrain draw, water, cell and block culling, lighting)
- [x] `LScapeRender::SetDefaultLighting` · 00583890 · legions 00553000
- [ ] `LScapeRender::ReloadLandBlocks` · 00583920 · legions 00553090 — needs port: landscape render (terrain draw, water, cell and block culling, lighting)
- [ ] `LScapeRender::SetVertexValue` · 00583950 · legions 005530c0 — needs port: landscape render (terrain draw, water, cell and block culling, lighting)
- [ ] `LScapeRender::Refresh` · 005839b0 · legions 00553120 — needs port: landscape render (terrain draw, water, cell and block culling, lighting)
- [ ] `LScapeRender::DirtyLighting` · 005839e0 · legions 00553150 — needs port: landscape render (terrain draw, water, cell and block culling, lighting)
- [ ] `LScapeRender::UpdateLightingInfo` · 005839f0 · legions 00553160 — needs port: landscape render (terrain draw, water, cell and block culling, lighting)
- [ ] `LScapeRender::FlagRefreshDrawLists` · 00583a00 · legions 00553170 — needs port: landscape render (terrain draw, water, cell and block culling, lighting)
- [ ] `LScapeRender::CalcBlockDrawFrame` · 00583a40 · legions 005531b0 — needs port: landscape render (terrain draw, water, cell and block culling, lighting)

### RainSegment
- [-] `RainSegment::operator=` · 00583aa0 · legions 00553210 — plain data ctor/dtor/operator=; Rust struct or Drop replaces

### LScapeRender
- [ ] `LScapeRender::LScapeRender` · 00583b00 · legions 00553270 — needs port: landscape render (terrain draw, water, cell and block culling, lighting)

### CPhysicsObj
- [ ] `CPhysicsObj::get_acceleration` · 00583c10 · legions 00553380 — needs port: object acceleration read for render (physics accessor)

### LScapeRender
- [-] `LScapeRender::~LScapeRender` · 00583c20 · legions 00553390 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [ ] `LScapeRender::NotifyChangeRegion` · 00583d00 · legions 00553470 — needs port: landscape render (terrain draw, water, cell and block culling, lighting)
- [ ] `LScapeRender::ReleaseCells` · 00583d20 · legions 00553490 — needs port: landscape render (terrain draw, water, cell and block culling, lighting)
- [x] `LScapeRender::AddCellSounds` · 00583d40 · legions 005534b0 — ac2-audio/ac2-client: ambient environments, groove
- [ ] `LScapeRender::CullLandBlocks` · 00583e70 · legions 005535e0 — needs port: landscape render (terrain draw, water, cell and block culling, lighting)
- [x] `LScapeRender::FindReflectionPlane` · 00583f40 · legions 00553680 — ac2-render scene/lighting.rs find_reflection_plane: the viewer's block's water below it, else the first resident block's; none in water
- [ ] `LScapeRender::GatherVisibleOutsideObjects` · 00584040 · legions 00553780 — needs port: landscape render (terrain draw, water, cell and block culling, lighting)
- [x] `LScapeRender::GetSunLightDirectionBits` · 00584090 · legions 005537d0 — self term: frill shade from its terrain cell's vertex colour bits (ac2-render terrain/shadow.rs, heights.rs self_shadow); map term in object.wgsl landscape_shadow
- [ ] `LScapeRender::UpdateLighting` · 005841c0 · legions 00553900 — needs port: landscape render (terrain draw, water, cell and block culling, lighting)
- [x] `LScapeRender::CalculateShadowColor` · 00584290 · legions 005539d0 — self term: frill shade from its terrain cell's vertex colour bits (ac2-render terrain/shadow.rs, heights.rs self_shadow); map term in object.wgsl landscape_shadow
- [ ] `LScapeRender::GetTerrainGeometry` · 005843b0 · legions 00553af0 — needs port: landscape render (terrain draw, water, cell and block culling, lighting)
- [x] `LScapeRender::RefreshShadowMap` · 00584540 · legions 00553c80 — ac2_gpu::StaticShadowMap (shadow_map.rs, object_shadow.rs); redraws on block/caster/sun>1deg/strength change
- [ ] `LScapeRender::Init` · 005847b0 · legions 00553ee0 — needs port: landscape render (terrain draw, water, cell and block culling, lighting)
- [ ] `LScapeRender::CalcBlockDrawOrderInternal` · 00584950 · legions 00554080 — needs port: landscape render (terrain draw, water, cell and block culling, lighting)
- [ ] `LScapeRender::CalcBlockDrawOrder` · 00584b10 · legions 00554240 — needs port: landscape render (terrain draw, water, cell and block culling, lighting)
- [ ] `LScapeRender::SetViewerPosition` · 00584c80 · legions 005543b0 — needs port: landscape render (terrain draw, water, cell and block culling, lighting)
- [x] `LScapeRender::FillRainVertices` · 00584d70 · legions 00554490 — ac2-render rain.rs RainField (streaks, splashes and near-water ripples) + scene rain_frame discharging WorldRender's rain particle systems
- [x] `LScapeRender::UpdateRain` · 005856f0 · legions 00554de0 — ac2-render rain.rs RainField (streaks, splashes and near-water ripples) + scene rain_frame discharging WorldRender's rain particle systems

## `portal/client/src/smartbox/worldrender/lscapesky.cpp.c` (4/9)

### LScapeSky
- [x] `LScapeSky::SetSkyFog` · 005857b0 · legions 00554ea0 — ac2-world region::sky_objects, ac2-render sky
- [x] `LScapeSky::CalcFrame` · 005857f0 · legions 00554ee0 — ac2-world region::sky_objects, ac2-render sky
- [ ] `LScapeSky::Clear` · 00585870 · legions 00554f60 — needs port: landscape sky state (sky clear/fog)
- [x] `LScapeSky::SetViewerPosition` · 005858b0 · legions 00554fa0 — ac2-world region::sky_objects, ac2-render sky
- [x] `LScapeSky::Draw` · 00585980 · legions 00555070 — sky objects drawn as tinted object instances before/after the world in CDayDesc::GetSky list order with the fog_sky quad, then each up object's lens flare (ac2-gpu lens_flare.rs); only viewer-centred domes tint the modern atmosphere (ac2-gpu SkyRole); fixed_pos/height_mod are carried but retail's sky drawing never reads them (CalcFrame, SetViewerPosition)
- [ ] `LScapeSky::LScapeSky` · 00585b30 · legions 00555220 — needs port: landscape sky state (sky clear/fog)
- [-] `LScapeSky::~LScapeSky` · 00585b50 · legions 00555240 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [~] `LScapeSky::CreateDeletePhysicsObjects` · 00585ba0 · legions 00555290 — sky objects drawn as tinted object instances before/after the world with the fog_sky quad; lens flares and fixed_pos/height_mod not yet
- [~] `LScapeSky::UpdateTimeOfDay` · 00585cf0 · legions 005553e0 — sky objects drawn as tinted object instances before/after the world with the fog_sky quad; lens flares and fixed_pos/height_mod not yet

## `portal/client/src/smartbox/worldrender/lscapeterrainmesh.cpp.c` (13/48)

### LM_UVTranslate
- [ ] `LM_UVTranslate::SetVTranslate` · 005880a0 · legions 005577e0 — needs port: UV translate (scrolling texture coordinates)

### LScapeTerrainMesh
- [ ] `LScapeTerrainMesh::CreateLocalMemBumpMap` · 005880f0 · legions 00557830 — needs port: landscape terrain mesh (vertices, shore, near/far mesh build)
- [x] `LScapeTerrainMesh::SetSelfShadowBit` · 00588300 · legions 00557a30 — self term: frill shade from its terrain cell's vertex colour bits (ac2-render terrain/shadow.rs, heights.rs self_shadow); map term in object.wgsl landscape_shadow
- [x] `LScapeTerrainMesh::IlluminateVertices` · 00588320 · legions 00557a50

### LScapeVertex
- [-] `LScapeVertex::operator=` · 00588370 · legions 0059f0d0 — plain data ctor/dtor/operator=; Rust struct or Drop replaces

### LScapeTerrainMesh
- [x] `LScapeTerrainMesh::CalculatePointsOfIntersection` · 00588410 · legions 00557aa0 — ac2-render terrain/shore.rs generate_shore over the near blocks' ground triangles (made again when the near blocks or centre change, scene/shore_strips.rs)

### ShorePointType
- [-] `ShorePointType::operator=` · 005884d0 · legions 00557b60 — plain data ctor/dtor/operator=; Rust struct or Drop replaces

### LandDefs
- [ ] `LandDefs::blockcoord_to_blockid` · 00588530 · legions 00557bc0 — needs port: landblock id conversion

### CVertexArray
- [-] `CVertexArray::CVertexArray` · 00588590 · legions 00557c20 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [-] `CVertexArray::~CVertexArray` · 005885e0 · legions 00557c70 — plain data ctor/dtor/operator=; Rust struct or Drop replaces

### LScapeTerrainMesh
- [~] `LScapeTerrainMesh::DrawTerrain` · 00588610 — ac2-render scene: far chunk culling by m_BoundingBox (near mesh drawn whole; no N-patches)
- [~] `LScapeTerrainMesh::DrawWaterMesh` · 00588840 · legions 00557ee0 — near water mesh with its material; far water chunks, bump map, reflection and shore passes not yet
- [ ] `LScapeTerrainMesh::ApplyList` · 00588a30 · legions 005580d0 — needs port: landscape terrain mesh (vertices, shore, near/far mesh build)
- [ ] `LScapeTerrainMesh::InqNearBlockIndex` · 00588aa0 · legions 00558140 — needs port: landscape terrain mesh (vertices, shore, near/far mesh build)

### (free functions)
- [x] `G_CalculateNormal` · 00588ae0 · legions 00558180 — ac2-render terrain/shore.rs generate_shore over the near blocks' ground triangles (made again when the near blocks or centre change, scene/shore_strips.rs)

### LScapeTerrainMesh
- [x] `LScapeTerrainMesh::FindShorePointIndexByOrigin` · 00588b70 · legions 00558210 — ac2-render terrain/shore.rs generate_shore over the near blocks' ground triangles (made again when the near blocks or centre change, scene/shore_strips.rs)
- [~] `LScapeTerrainMesh::DrawShore` · 00588be0 · legions 00558280 — DrawShore(true) at LandscapeShoreEffects 2: the strips (closed ones under 20 m left out) with the 0x40000017 material after the water (ac2-gpu scene shore group); the destination-alpha pass DrawShore(false) (0x40000024) is not drawn: the HDR target keeps no destination alpha for the water to blend by

### DBObjGrabber<RenderTexture>
- [-] `DBObjGrabber<RenderTexture>::DBObjGrabber<RenderTexture>` · 00588e10 · legions 005584b0 — retail DB object cache handle/release; Arc per DID replaces (AGENTS.md exception)

### LScapeFarChunk
- [ ] `LScapeFarChunk::LScapeFarChunk` · 005890f0 · legions 00558840 — needs port: far terrain chunk init

### LScapeTerrainMesh
- [x] `LScapeTerrainMesh::BuildFarTerrainMesh` · 00589130 · legions 00558880 — indices with RollDice diagonals and LOD crack fill, vertices, far water quads (terrain/far.rs)
- [x] `LScapeTerrainMesh::SetVertex` · 00589ec0 · legions 00559610 — LScapeTerrainMesh::SetVertex cited in ac2-render terrain/mesh.rs:2 (four vertices per polygon, slots SW/SE/NW/NE)
- [ ] `LScapeTerrainMesh::GetPixelGeometry` · 0058a000 · legions 00559750 — needs port: landscape terrain mesh (vertices, shore, near/far mesh build)
- [x] `LScapeTerrainMesh::CleanupShore` · 0058a1f0 · legions 00559940 — ac2-render terrain/shore.rs generate_shore over the near blocks' ground triangles (made again when the near blocks or centre change, scene/shore_strips.rs)

### SmartArray<ShoreStripType>
- [-] `SmartArray<ShoreStripType>::~SmartArray<ShoreStripType>` · 0058a270 · legions 005599f0 — SmartArray/STL container internals; Vec replaces

### SmartArray<short>
- [-] `SmartArray<short>::operator=` · 0058a400 · legions 00559af0 — SmartArray/STL container internals; Vec replaces

### SmartArray<LScapeFarBlock>
- [-] `SmartArray<LScapeFarBlock>::operator=` · 0058a460 · legions 00559b50 — SmartArray/STL container internals; Vec replaces

### LScapeTerrainMesh
- [x] `LScapeTerrainMesh::GetShadowColor` · 0058a4d0 · legions 00559bc0 — self term: frill shade from its terrain cell's vertex colour bits (ac2-render terrain/shadow.rs, heights.rs self_shadow); map term in object.wgsl landscape_shadow
- [ ] `LScapeTerrainMesh::SetVertexValue` · 0058a870 · legions 00559f60 — needs port: landscape terrain mesh (vertices, shore, near/far mesh build)

### SmartArray<LayerStage_*>
- [-] `SmartArray<LayerStage_*>::InsertOrderedByIndex` · 0058aab0 · legions 00540d30 — SmartArray/STL container internals; Vec replaces

### LScapeFarChunk
- [-] `LScapeFarChunk::operator=` · 0058ac60 · legions 0055a2e0 — plain data ctor/dtor/operator=; Rust struct or Drop replaces

### MaterialLayer
- [ ] `MaterialLayer::InsertModifier` · 0058ad10 · legions 0055a390 — needs port: material layer stage order and diffuse colour

### LScapeTerrainMesh
- [-] `LScapeTerrainMesh::DestroyLScapeMaterialInfo` · 0058ad30 · legions 0055a3b0 — retail DB object cache handle/release; Arc per DID replaces (AGENTS.md exception)
- [~] `LScapeTerrainMesh::DrawWater` · 0058ad90 — near water mesh with its material; far water chunks, bump map, reflection and shore passes not yet
- [ ] `LScapeTerrainMesh::ReOrderNearVertices` · 0058b460 · legions 0055aad0 — needs port: landscape terrain mesh (vertices, shore, near/far mesh build)
- [x] `LScapeTerrainMesh::BuildNearWaterMesh` · 0058b600 · legions 0055ac70 — ac2-render object::load_water_model
- [-] `LScapeTerrainMesh::Destroy` · 0058b8a0 · legions 0055af10 — retail DB object cache handle/release; Arc per DID replaces (AGENTS.md exception)
- [~] `LScapeTerrainMesh::BuildNearTerrainMesh` · 0058b9e0 · legions 0055b050 — per-block near mesh; no cloud-shadow UVs or shared near vertex array
- [-] `LScapeTerrainMesh::~LScapeTerrainMesh` · 0058bc10 · legions 0055b280 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [x] `LScapeTerrainMesh::InitFarMesh` · 0058bce0 · legions 0055b350 — far chunks, LOD, block edges, far material with ADDSIGNED + MODULATE2X cloud stage (far_terrain.wgsl)
- [~] `LScapeTerrainMesh::UpdateNearTerrain` · 0058c8c0 · legions 0055bf30 — vertex colour bits kept (ac2-world GetColor field); the near-mesh alpha is only read by the frill shade, not drawn
- [x] `LScapeTerrainMesh::UpdateNearWater` · 0058d700 · legions 0055cd70 — ac2-render object::load_water_model
- [x] `LScapeTerrainMesh::GenerateShore` · 0058dcb0 · legions 0055d320 — ac2-render terrain/shore.rs generate_shore over the near blocks' ground triangles (made again when the near blocks or centre change, scene/shore_strips.rs)
- [ ] `LScapeTerrainMesh::Init` · 0058e940 · legions 0055dfb0 — needs port: landscape terrain mesh (vertices, shore, near/far mesh build)
- [ ] `LScapeTerrainMesh::LScapeTerrainMesh` · 0058eb20 · legions 0055e190 — needs port: landscape terrain mesh (vertices, shore, near/far mesh build)
- [ ] `LScapeTerrainMesh::RefreshWorldBuilderTerrain` · 0058ec60 · legions 0055e2d0 — needs port: landscape terrain mesh (vertices, shore, near/far mesh build)
- [ ] `LScapeTerrainMesh::Update` · 0058ee40 · legions 0055e4b0 — needs port: landscape terrain mesh (vertices, shore, near/far mesh build)
- [ ] `LScapeTerrainMesh::Refresh` · 0058f030 · legions 0055e6a0 — needs port: landscape terrain mesh (vertices, shore, near/far mesh build)
- [ ] `LScapeTerrainMesh::Refresh` · 0058f070 · legions 0055e6e0 — needs port: landscape terrain mesh (vertices, shore, near/far mesh build)

## `portal/client/src/smartbox/worldrender/portalspace.cpp.c` (12/14)

### CDataMesh
- [ ] `CDataMesh::GetByEnum` · 00585e60 · legions 00555550 — needs port: data mesh lookup by enum

### PortalSpace::Effect
- [~] `PortalSpace::Effect::Effect` · 00585e90 · legions 00555580 — tunnel geometry (positions, uvs, index buffer, ring fade) in ac2-render portal/tunnel.rs; not yet: the vertex buffer locks, the portal sounds, the ring-to-alpha mapping (unverified), the outer ring's material
- [x] `PortalSpace::Effect::MakeSafePortalIndex` · 00585f40 · legions 00555630 — ac2-math perlin.rs, matrix.rs; ac2-render portal/tunnel.rs; tests: tunnel_tests, perlin, matrix
- [x] `PortalSpace::Effect::GeneratePortals` · 00585f90 · legions 00555680 — ac2-render portal/tunnel.rs and effect.rs: GeneratePortals' local matrix carries (0,3,0); Think's phase machine, fader, tunnel advance; StartNow/StopNow sounds via ac2-client play_portal_sounds
- [x] `PortalSpace::Effect::StartNow` · 00586220 · legions 00555910 — ac2-render portal/tunnel.rs and effect.rs: GeneratePortals' local matrix carries (0,3,0); Think's phase machine, fader, tunnel advance; StartNow/StopNow sounds via ac2-client play_portal_sounds
- [x] `PortalSpace::Effect::Destroy` · 00586650 · legions 00555d40 — materials 0x40000013/0x40000014, the clouds' PSDesc 0x40000015 (ac2-render portal/assets.rs load_clouds_desc, Client::set_portal_clouds), the background CDataMesh 0x40000016 (load_portal_models) and the sounds load at setup (client_teleport.rs); freed on last drop
- [x] `PortalSpace::Effect::Start` · 00586720 — ac2-render portal/effect.rs (enter/exit/in_portal_space, Start/Stop) and ac2-client client_teleport.rs; tests: portal_7160, effect_tests
- [x] `PortalSpace::Effect::UpdateGeometry` · 00586740 · legions 00555e30 — ac2-render portal/tunnel.rs update_geometry(current, start, scale): ring base is m_nLastPortalPassed, start is the transparent depth, memory order reversed (closing vertex first), x stored as float, 1/w in f64; alpha ramp verified (0.8+0.2(1-X/15), 0.8(1-X/149)^2)
- [x] `PortalSpace::Effect::StopNow` · 00586eb0 · legions 005565a0 — ac2-render portal/tunnel.rs and effect.rs: GeneratePortals' local matrix carries (0,3,0); Think's phase machine, fader, tunnel advance; StartNow/StopNow sounds via ac2-client play_portal_sounds
- [x] `PortalSpace::Effect::Think` · 00586f50 · legions 00556640 — ac2-render portal/tunnel.rs and effect.rs: GeneratePortals' local matrix carries (0,3,0); Think's phase machine, fader, tunnel advance; StartNow/StopNow sounds via ac2-client play_portal_sounds
- [x] `PortalSpace::Effect::Render` · 00587360 · legions 00556a50 — ac2-gpu portal_space.rs, ac2-render portal/: fader, background mesh 0x40000016 at the eye with the opacity as alpha, outer then inner rings, discharge loop (portal/clouds.rs; has_particle reset in GeneratePortals) and cloud particles, into the HDR scene target from the portal camera; SetFOV(mFOV * m_fCameraFOVScale) on SceneCamera's defaults (65 degrees, 0.1, 2000; the decompile's mFarDistance is the recovered export's mFOV); UnlinkAllLights + DisableSunlight (SetAmbient black, vtable +0x4c) + FOGMODE_NONE as a no-light, black-ambient, unfogged environment
- [x] `PortalSpace::Effect::~Effect` · 00587da0 · legions 00557490 — materials 0x40000013/0x40000014, the clouds' PSDesc 0x40000015 (ac2-render portal/assets.rs load_clouds_desc, Client::set_portal_clouds), the background CDataMesh 0x40000016 (load_portal_models) and the sounds load at setup (client_teleport.rs); freed on last drop
- [x] `PortalSpace::Effect::Initialize` · 00587db0 · legions 005574a0 — materials 0x40000013/0x40000014, the clouds' PSDesc 0x40000015 (ac2-render portal/assets.rs load_clouds_desc, Client::set_portal_clouds), the background CDataMesh 0x40000016 (load_portal_models) and the sounds load at setup (client_teleport.rs); freed on last drop
- [x] `PortalSpace::Effect::Stop` · 00588050 — ac2-render portal/effect.rs (enter/exit/in_portal_space, Start/Stop) and ac2-client client_teleport.rs; tests: portal_7160, effect_tests

## `portal/client/src/smartbox/worldrender/worldrender.cpp.c` (11/36)

### GameTime
- [ ] `GameTime::GetTimeTilTomorrow` · 0057c720 · legions 0054bff0 — needs port: game time lighting refresh cadence
- [x] `GameTime::GetNeedLightingRefresh` · 0057c740 · legions 0054c010 — ac2-world GameClock::alter_time_of_day_immediate / temporal_dilation (x87 double arithmetic, objdump-verified) and need_lighting_refresh
- [x] `GameTime::NotifyLightingRefreshed` · 0057c760 · legions 0054c030 — ac2-world GameClock::alter_time_of_day_immediate / temporal_dilation (x87 double arithmetic, objdump-verified) and need_lighting_refresh

### PhysicsGlobals
- [x] `PhysicsGlobals::SetWind` · 0057c780 · legions 0054c050 — ac2-world RegionDesc::physics_wind (rain wind or the Perlin breeze); ac2-client sets it on the bodies each animation step

### FogInfo
- [-] `FogInfo::FogInfo` · 0057c7d0 · legions 0054c0a0 — plain data ctor/dtor/operator=; Rust struct or Drop replaces

### LightInfo
- [-] `LightInfo::LightInfo` · 0057c820 · legions 0054c0f0 — plain data ctor/dtor/operator=; Rust struct or Drop replaces

### LScapeLighting
- [ ] `LScapeLighting::GetShadowColor` · 0057c8b0 · legions 0054c180 — needs port: landscape lighting (day light, fog corners, water light, combine, shadow colour)

### WorldRender
- [ ] `WorldRender::Init` · 0057c8e0 · legions 0054c1b0 — needs port: world render (frame draw, shadows, reflections, objects, time of day)
- [x] `WorldRender::EnterPortalSpace` · 0057ca90 — ac2-render portal/effect.rs (enter/exit/in_portal_space, Start/Stop) and ac2-client client_teleport.rs; tests: portal_7160, effect_tests
- [x] `WorldRender::ExitPortalSpace` · 0057cab0 — ac2-render portal/effect.rs (enter/exit/in_portal_space, Start/Stop) and ac2-client client_teleport.rs; tests: portal_7160, effect_tests
- [x] `WorldRender::InPortalSpace` · 0057cad0 · legions 0054c3a0 — ac2-render portal/effect.rs (enter/exit/in_portal_space, Start/Stop) and ac2-client client_teleport.rs; tests: portal_7160, effect_tests
- [ ] `WorldRender::ForceShadowRefresh` · 0057caf0 · legions 0054c3c0 — needs port: world render (frame draw, shadows, reflections, objects, time of day)
- [x] `WorldRender::UpdateAmbientSounds` · 0057cb00 · legions 0054c3d0 — ac2-audio: rain.rs (RainSounds, the loop and thunder), ambient_driver.rs (AmbientDriver over the SoundSpace trait); ac2-client sound_ids.rs (DIDs), client_ambient.rs and sound_space.rs (physics adapter); tests sound_ids_tests.rs and ac2-audio unit tests
- [ ] `WorldRender::InitAmbientSounds` · 0057cb40 — needs port: world render (frame draw, shadows, reflections, objects, time of day)
- [~] `WorldRender::SetupEnvironment` · 0057cb50 · legions 0054c420 — outdoor sun, ambient and fog; indoor and sky environments not yet
- [x] `WorldRender::UpdateRainSounds` · 0057cf80 · legions 0054c850 — ac2-audio: rain.rs (RainSounds, the loop and thunder), ambient_driver.rs (AmbientDriver over the SoundSpace trait); ac2-client sound_ids.rs (DIDs), client_ambient.rs and sound_space.rs (physics adapter); tests sound_ids_tests.rs and ac2-audio unit tests

### SceneCamera
- [ ] `SceneCamera::SceneCamera` · 0057d190 · legions 0054ca60 — needs port: world camera defaults (only UI scene defaults are cited)

### WorldRender
- [ ] `WorldRender::DrawBrushes` · 0057d290 · legions 0054cb60 — needs port: world render (frame draw, shadows, reflections, objects, time of day)
- [~] `WorldRender::CreateDiscShadows` · 0057d3d0 · legions 0054cca0 — ac2_gpu sun_shadow.rs + object_sun_casters.rs: DynamicObjectShadows levels 1-3 draw cascaded sun depth maps (bodies and near static objects cast, posed and skinned; terrain/objects/bodies receive with PCF at the static strength) in place of disc decals and stencil volumes (AGENTS.md exception)
- [~] `WorldRender::StartSelection` · 0057d680 · legions 0054cf50 — ac2-render selection.rs: drawn-mesh triangles (skinned parts at their group matrix), no potential list, no selectability / LookAtSelection / own-item checks

### DBObjGrabber<CDayDesc>
- [-] `DBObjGrabber<CDayDesc>::DBObjGrabber<CDayDesc>` · 0057d6d0 · legions 0054cfa0 — retail DB object cache handle/release; Arc per DID replaces (AGENTS.md exception)

### WorldRender
- [ ] `WorldRender::WorldRender` · 0057d870 · legions 0054d140 — needs port: world render (frame draw, shadows, reflections, objects, time of day)
- [ ] `WorldRender::SetWantNoticeOnNextFrameDrawn` · 0057da00 · legions 0054d2d0 — needs port: world render (frame draw, shadows, reflections, objects, time of day)
- [ ] `WorldRender::GetWantNoticeOnNextFrameDrawn` · 0057da10 · legions 0054d2e0 — needs port: world render (frame draw, shadows, reflections, objects, time of day)
- [-] `WorldRender::~WorldRender` · 0057da20 · legions 0054d2f0 — plain data ctor/dtor/operator=; Rust struct or Drop replaces
- [ ] `WorldRender::ResetWorldState` · 0057db20 · legions 0054d3f0 — needs port: world render (frame draw, shadows, reflections, objects, time of day)
- [ ] `WorldRender::DrawWorldObjects` · 0057dba0 · legions 0054d470 — needs port: world render (frame draw, shadows, reflections, objects, time of day)
- [ ] `WorldRender::UpdateTimeOfDay` · 0057dcc0 · legions 0054d590 — needs port: world render (frame draw, shadows, reflections, objects, time of day)
- [~] `WorldRender::Update` · 0057e1b0 · legions 0054da80 — ac2-client world_manager_update: WorldManager::Update, UpdateRenderInterpolation, UpdateAnimation; no time of day, rain, ambient sound or scene tools
- [x] `WorldRender::RenderReflectionMap` · 0057e270 · legions 0054db40 — replaced by ac2-gpu water_reflection.rs's screen-space reflection over the same plane (AGENTS.md exception)
- [-] `WorldRender::UpdateRenderTargets` · 0057e750 · legions 0054e020 — D3D device/resource plumbing; wgpu replaces
- [ ] `WorldRender::GatherIndoorObjects` · 0057e830 · legions 0054e100 — needs port: world render (frame draw, shadows, reflections, objects, time of day)
- [ ] `WorldRender::GatherOutdoorObjects` · 0057ea20 · legions 0054e2f0 — needs port: world render (frame draw, shadows, reflections, objects, time of day)
- [x] `WorldRender::CreateLandscapePointLightDecals` · 0057eb10 · legions 0054e3e0 — ac2-render scene/light_decals.rs: each frame the FX point lights and the drawn cells' own lights within reach over the ground pool there as draw-once decals of the day/dusk/night material (enums 0x4000000D/2B/2C), gated by LightSourceEffects
- [x] `WorldRender::RenderSkeletonEffects` · 0057ef60 · legions 0054e830 — ac2-render scene/skeleton_effects.rs: the outdoor bodies' bones discharge rain vapour (rain over 0.4, bones across the rain), water bubbles and ripples into the shared systems' first emitter; the bubbles system (enum 0x40000011) loads with the rain's
- [~] `WorldRender::Render` · 0057f960 · legions 0054f230 — the portal gate (InPortalSpace), the world's SetFOV(m_fWorldFOVScale * mFOV) in portal space (ac2-client view.rs camera), and PortalSpace::Effect::Render after the world (ac2-client ui_scene/render.rs); not all of the world pass is traced here
