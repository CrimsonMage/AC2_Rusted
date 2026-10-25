# ac2-render: Scene and world rendering

781 retail functions: 33 done, 21 partial, 714 to do, 13 skipped.
Legend and workflow: [README](README.md).

## `portal/client/src/ctrilist/ctrianglelist.cpp.c` (0/12)

### CMeshFragment
- [ ] `CMeshFragment::CMeshFragment` · 0052bf30
- [ ] `CMeshFragment::InitIndexed` · 0052bf60 · legions 00510b50
- [ ] `CMeshFragment::Init` · 0052bfd0 · legions 00510bc0

### CMeshTriList
- [ ] `CMeshTriList::CMeshTriList` · 0052c000 · legions 00510bf0
- [ ] `CMeshTriList::~CMeshTriList` · 0052c030 · legions 00510c20
- [ ] `CMeshTriList::GetPackSize` · 0052c120 · legions 00510d10
- [ ] `CMeshTriList::Pack` · 0052c190 · legions 00510d80
- [ ] `CMeshTriList::UnPack` · 0052c260 · legions 00510e50
- [ ] `CMeshTriList::Init` · 0052c390 · legions 00510f80

### DegradeType
- [ ] `DegradeType::ParseString` · 0052c460

### CMeshFragment
- [-] `CMeshFragment::FromFileNode` · 0052c600 — text source format (admin tools)

### CMeshTriList
- [-] `CMeshTriList::FromFileNode` · 0052c830 — text source format (admin tools)

## `portal/client/src/render/core/font/font.cpp.c` (0/13)

### Font
- [ ] `Font::Begin` · 005b88d0 · legions 00583530
- [ ] `Font::End` · 005b8900 · legions 00583560
- [ ] `Font::Destroy` · 005b8940 · legions 005835a0
- [ ] `Font::GetCharDesc` · 005b89a0 · legions 00583600
- [ ] `Font::GetCharWidthA` · 005b8a30 · legions 00583690
- [ ] `Font::ContainsString` · 005b8a50 · legions 005836b0
- [ ] `Font::GenerateCharacterMap` · 005b8ad0 · legions 00583730
- [ ] `Font::GetStringWidthInternal` · 005b8bc0 · legions 00583820
- [ ] `Font::InitLoad` · 005b8c20
- [-] `Font::LoadFromFile` · 005b8c40 — text source loader, admin build only
- [ ] `Font::SaveToFile` · 005b8d40
- [ ] `Font::StreamPack` · 005b8e10 · legions 00583890
- [ ] `Font::GetStringWidth` · 005b8f70 · legions 005839f0

## `portal/client/src/render/core/font/fontmapper.cpp.c` (0/4)

### FontLocal
- [ ] `FontLocal::GetGlobalOrLocalFont` · 005c4800 · legions 0058b8c0

### FontMapper
- [ ] `FontMapper::Startup` · 005c4850 · legions 0058b910
- [ ] `FontMapper::Shutdown` · 005c4890 · legions 0058b950
- [ ] `FontMapper::GetFontByDataID` · 005c48f0 · legions 0058b9b0

## `portal/client/src/render/core/graphicsresource.cpp.c` (0/10)

### GraphicsResource
- [ ] `GraphicsResource::CopyInto` · 005bc290 · legions 005859e0
- [ ] `GraphicsResource::RestoreResource` · 005bc2c0 · legions 00585a10
- [ ] `GraphicsResource::ShutdownResourceManager` · 005bc330 · legions 00585a80
- [ ] `GraphicsResource::UnlinkResource` · 005bc360 · legions 00585ab0
- [ ] `GraphicsResource::PurgeResources` · 005bc3b0 · legions 00585b20
- [ ] `GraphicsResource::PurgeOldResources` · 005bc400 · legions 00585b70
- [ ] `GraphicsResource::~GraphicsResource` · 005bc530 · legions 00585ca0
- [ ] `GraphicsResource::LinkResource` · 005bc620 · legions 00585d90
- [ ] `GraphicsResource::GraphicsResource` · 005bc690 · legions 00585df0
- [ ] `GraphicsResource::DiscardResourceBytes` · 005bca00 · legions 00586140

## `portal/client/src/render/core/indexbuffer/d3d/renderindexbufferd3d.cpp.c` (0/10)

### RenderIndexBufferD3D
- [ ] `RenderIndexBufferD3D::Begin` · 005db9a0 · legions 0059c7b0
- [ ] `RenderIndexBufferD3D::RefreshD3DIndexBuffer` · 005db9b0 · legions 0059c7c0
- [ ] `RenderIndexBufferD3D::GetD3DIndexBuffer` · 005dba90 · legions 0059c8a0
- [ ] `RenderIndexBufferD3D::CreateD3DIndexBuffer` · 005dbad0 · legions 0059c8e0
- [ ] `RenderIndexBufferD3D::PurgeResource` · 005dbbc0 · legions 0059c9d0
- [ ] `RenderIndexBufferD3D::RestoreResource` · 005dbc00 · legions 0059ca10
- [ ] `RenderIndexBufferD3D::End` · 005dbc20 · legions 0059ca30
- [ ] `RenderIndexBufferD3D::Startup` · 005dbc50 · legions 0059ca60
- [ ] `RenderIndexBufferD3D::Shutdown` · 005dbc90 · legions 0059caa0
- [ ] `RenderIndexBufferD3D::Duplicate` · 005dbcd0 · legions 0059cae0

## `portal/client/src/render/core/indexbuffer/renderindexbuffer.cpp.c` (0/11)

### RenderIndexBuffer
- [ ] `RenderIndexBuffer::Begin` · 005bdcd0 · legions 00586e70
- [ ] `RenderIndexBuffer::End` · 005bdd00 · legions 00586ea0
- [ ] `RenderIndexBuffer::Startup` · 005bdd20 · legions 00586ec0
- [ ] `RenderIndexBuffer::Shutdown` · 005bdd90 · legions 00586f30
- [ ] `RenderIndexBuffer::Lock` · 005bddd0 · legions 00586f70
- [ ] `RenderIndexBuffer::Unlock` · 005bdde0 · legions 00586f80
- [ ] `RenderIndexBuffer::RecalculateMinMaxIndices` · 005bde00 · legions 00586fa0
- [ ] `RenderIndexBuffer::Stream` · 005bdea0 · legions 00587040
- [ ] `RenderIndexBuffer::SetRenderIndexBuffer` · 005bdfe0 · legions 00587180
- [ ] `RenderIndexBuffer::Duplicate` · 005be070 · legions 00587210
- [-] `RenderIndexBuffer::FromFileNode` · 005be0f0 — text source format (admin tools)

## `portal/client/src/render/core/null/renderdevicenull.cpp.c` (0/8)

### RenderDeviceNull
- [ ] `RenderDeviceNull::Begin` · 005dcfb0 · legions 0059dca0
- [ ] `RenderDeviceNull::End` · 005dcfc0 · legions 0059dcb0
- [ ] `RenderDeviceNull::Startup` · 005dcfd0
- [ ] `RenderDeviceNull::Shutdown` · 005dd020 · legions 0059dd10
- [ ] `RenderDeviceNull::CreateSurface` · 005dd040
- [ ] `RenderDeviceNull::CreateIndexBuffer` · 005dd060
- [ ] `RenderDeviceNull::CreateVertexBuffer` · 005dd090
- [ ] `RenderDeviceNull::CreateTexture` · 005dd0e0 · legions 00586960

## `portal/client/src/render/core/null/rendernull.cpp.c` (0/11)

### RenderNull
- [ ] `RenderNull::Startup` · 005d9410 · legions 0059aaa0
- [ ] `RenderNull::Shutdown` · 005d9430 · legions 0059aac0

### RenderDevice
- [ ] `RenderDevice::RenderDevice` · 005d9440 · legions 0059aad0
- [ ] `RenderDevice::SetupState` · 005d9590
- [ ] `RenderDevice::SetMaterialDiffuseColor` · 005d95a0
- [ ] `RenderDevice::IsResetPossible` · 005d95d0
- [ ] `RenderDevice::~RenderDevice` · 005d95e0 · legions 0059ac80

### RenderDeviceNull
- [ ] `RenderDeviceNull::RenderDeviceNull` · 005d9640 · legions 0059ace0
- [ ] `RenderDeviceNull::BeginScene` · 005d9710
- [ ] `RenderDeviceNull::EndScene` · 005d9720

### RenderNull
- [ ] `RenderNull::CreateRenderDevice` · 005d9760 · legions 0059ae00

## `portal/client/src/render/core/pixelformat.cpp.c` (0/5)

### PixelFormatDesc
- [ ] `PixelFormatDesc::CalcBitOffsets` · 005ac250 · legions 00578550
- [ ] `PixelFormatDesc::ConvertColor` · 005ac390
- [ ] `PixelFormatDesc::SetFormat` · 005ac470 · legions 00578690
- [ ] `PixelFormatDesc::DetermineFormat` · 005ac870
- [ ] `PixelFormatDesc::PixelFormatDesc` · 005acba0 · legions 00578a90

## `portal/client/src/render/core/render.cpp.c` (0/27)

### Render
- [ ] `Render::HandleChange` · 005b30c0 · legions 0057da60

### GraphicsPreferences
- [ ] `GraphicsPreferences::GraphicsPreferences` · 005b30e0 · legions 0057da80
- [ ] `GraphicsPreferences::~GraphicsPreferences` · 005b31f0 · legions 0057db90

### Render
- [ ] `Render::StartupDevice` · 005b3300 · legions 0057dca0
- [ ] `Render::ShutdownDevice` · 005b3380 · legions 0057dd20
- [ ] `Render::SetOverallGraphicsQuality` · 005b33b0 · legions 0057dd50
- [ ] `Render::ShutdownRenderingSystem` · 005b3c40 · legions 0057e5e0
- [ ] `Render::RestartDevice` · 005b3ca0 · legions 0057e640
- [ ] `Render::CreateIndexBuffer` · 005b3d90 · legions 0057e730
- [ ] `Render::Startup` · 005b3de0 · legions 0057e780
- [ ] `Render::Shutdown` · 005b3fb0 · legions 0057e950
- [ ] `Render::InitRenderingPreferences` · 005b4160 · legions 0057eb00
- [ ] `Render::StartupRenderingDevice` · 005b5200 · legions 0057fba0
- [ ] `Render::RestartRenderingSystem` · 005b52b0 · legions 0057fc50
- [ ] `Render::FlushGraphicsResources` · 005b5310 · legions 0057fcb0
- [ ] `Render::RestartRenderingSystem` · 005b5360 · legions 0057fd00
- [ ] `Render::UnlinkRGRCallback` · 005b53b0 · legions 0057fd50
- [ ] `Render::CreateVertexBuffer` · 005b5400 · legions 0057fda0
- [ ] `Render::~Render` · 005b5460 · legions 0057fe00

### RenderNull
- [ ] `RenderNull::RenderNull` · 005b54f0 · legions 0057fe90

### RenderD3D
- [ ] `RenderD3D::RenderD3D` · 005b5570 · legions 0057ff10
- [ ] `RenderD3D::~RenderD3D` · 005b5610 · legions 0057ffb0

### Render
- [ ] `Render::StartupRenderingSystem` · 005b5670 · legions 00580010
- [ ] `Render::SaveOverallGraphicsQuality` · 005b5710 · legions 005800b0
- [ ] `Render::UpdateOverallGraphicsQuality` · 005b5860
- [ ] `Render::LinkRGRCallback` · 005b5e70 · legions 00580810
- [ ] `Render::HandleApply` · 005b5ee0 · legions 00580880

## `portal/client/src/render/core/renderdevice.cpp.c` (1/17)

### RDTemporaryState
- [ ] `RDTemporaryState::Invalidate` · 005ae490 · legions 00579f30

### RDPersistentState
- [ ] `RDPersistentState::Invalidate` · 005ae670 · legions 0057a110

### RenderDevice
- [ ] `RenderDevice::Begin` · 005ae710 · legions 0057a1b0
- [ ] `RenderDevice::End` · 005ae750 · legions 0057a1f0
- [ ] `RenderDevice::Startup` · 005ae7b0 · legions 0057a250
- [ ] `RenderDevice::Shutdown` · 005ae860 · legions 0057a300
- [ ] `RenderDevice::GetDisplayWidth` · 005ae8a0 · legions 0057a340
- [ ] `RenderDevice::GetDisplayHeight` · 005ae8b0 · legions 0057a350
- [ ] `RenderDevice::CreateLocalSurface` · 005ae8c0
- [ ] `RenderDevice::SetViewport` · 005ae8e0 · legions 0057a380
- [ ] `RenderDevice::GetUISurfaceFormat` · 005ae950 · legions 005cbab0
- [x] `RenderDevice::SetBlendMatrices` · 005ae960 — ac2-gpu ObjectInstance.pose (part matrices, bone storage buffer)
- [ ] `RenderDevice::SetRenderTargetTexture` · 005ae980 · legions 0057a410

### VertexShader
- [ ] `VertexShader::VertexShader` · 005ae9d0 · legions 0057a450

### PixelShader
- [ ] `PixelShader::PixelShader` · 005aea20 · legions 0057a4a0

### RDVolatileState
- [ ] `RDVolatileState::Invalidate` · 005aea70 · legions 0057a4f0

### RenderDeviceState
- [ ] `RenderDeviceState::Invalidate` · 005aeab0 · legions 0057a530

## `portal/client/src/render/core/surface/d3d/rendersurfaced3d.cpp.c` (0/26)

### NormalMap
- [ ] `NormalMap::Kill` · 005ca740 · legions 00591730

### RenderSurfaceD3D
- [ ] `RenderSurfaceD3D::CreateFromD3DSurface` · 005ca770
- [ ] `RenderSurfaceD3D::CreateDepthStencil` · 005ca800 · legions 005917f0
- [ ] `RenderSurfaceD3D::GetDirect3DSurface` · 005ca840 · legions 00591830
- [ ] `RenderSurfaceD3D::CreateD3DSurface` · 005ca880 · legions 00591870
- [ ] `RenderSurfaceD3D::PurgeResource` · 005ca8f0 · legions 005918e0
- [ ] `RenderSurfaceD3D::RestoreResource` · 005ca940 · legions 00591930

### NormalMap::GrayScale
- [ ] `NormalMap::GrayScale::Calculate` · 005ca980

### NormalMap::Average
- [ ] `NormalMap::Average::Calculate` · 005ca9d0

### NormalMap::Red
- [ ] `NormalMap::Red::Calculate` · 005caa30

### NormalMap::Alpha
- [ ] `NormalMap::Alpha::Calculate` · 005caa50

### NormalMap::Green
- [ ] `NormalMap::Green::Calculate` · 005caa70

### NormalMap::Blue
- [ ] `NormalMap::Blue::Calculate` · 005caa90

### NormalMap::Maximum
- [ ] `NormalMap::Maximum::Calculate` · 005caac0

### NormalMap::Lightness
- [ ] `NormalMap::Lightness::Calculate` · 005cab40

### NormalMap::ColorSpace
- [ ] `NormalMap::ColorSpace::Calculate` · 005cac00

### RenderSurfaceD3D
- [ ] `RenderSurfaceD3D::RenderSurfaceD3D` · 005cac70 · legions 00591c60
- [ ] `RenderSurfaceD3D::Create` · 005cacb0
- [ ] `RenderSurfaceD3D::Destroy` · 005cad10 · legions 00591d00
- [ ] `RenderSurfaceD3D::Lock` · 005cad70
- [ ] `RenderSurfaceD3D::Unlock` · 005cae30 · legions 00591e20

### NormalMap
- [ ] `NormalMap::Init` · 005caea0 · legions 00591e90
- [ ] `NormalMap::Convert24BitToNormalMap` · 005cafe0 · legions 00591fd0

### RenderSurfaceD3D
- [ ] `RenderSurfaceD3D::~RenderSurfaceD3D` · 005cb3f0 · legions 005923e0
- [ ] `RenderSurfaceD3D::ConvertToNormalMap` · 005cb460
- [ ] `RenderSurfaceD3D::Convert` · 005cb5b0

## `portal/client/src/render/core/surface/rendersurface.cpp.c` (0/18)

### RenderSurface
- [ ] `RenderSurface::Begin` · 005b13f0 · legions 0057cea0
- [ ] `RenderSurface::Create` · 005b1450 · legions 0057cf00
- [ ] `RenderSurface::Destroy` · 005b14f0 · legions 0057cfa0
- [ ] `RenderSurface::InitLoad` · 005b1530 · legions 0057cfe0
- [ ] `RenderSurface::Lock` · 005b1570 · legions 0057d010
- [ ] `RenderSurface::SelectSurfaceFormat` · 005b15e0 · legions 0057d080
- [ ] `RenderSurface::Convert` · 005b1690 · legions 0057d130
- [ ] `RenderSurface::RenderSurface` · 005b1860 · legions 0057d300
- [ ] `RenderSurface::GetDBOType` · 005b18c0
- [ ] `RenderSurface::~RenderSurface` · 005b18d0 · legions 0057d370
- [ ] `RenderSurface::CreateFromSourceData` · 005b1930 · legions 0057d3d0
- [ ] `RenderSurface::StreamPack` · 005b1c40 · legions 0057d6d0
- [-] `RenderSurface::LoadDDS` · 005b1d60 — text source loader, admin build only
- [-] `RenderSurface::LoadJPGAsRawData` · 005b1eb0 — text source loader, admin build only
- [ ] `RenderSurface::SaveJPG` · 005b1f70 · legions 0057d7f0
- [ ] `RenderSurface::Allocator` · 005b2130
- [-] `RenderSurface::LoadFromFile` · 005b2170 — text source loader, admin build only
- [ ] `RenderSurface::Unlock` · 005b2370 · legions 0057d9f0

## `portal/client/src/render/core/surface/surfacewindow.cpp.c` (0/25)

### SurfaceWindow
- [ ] `SurfaceWindow::End` · 005aed30 · legions 0057a7b0
- [ ] `SurfaceWindow::Attach` · 005aed40 · legions 0057a7c0
- [ ] `SurfaceWindow::Attach` · 005aede0 · legions 0057a860
- [ ] `SurfaceWindow::WindowToClip` · 005aee20 · legions 0057a8a0
- [ ] `SurfaceWindow::ClipToWindow` · 005aee60 · legions 0057a8e0
- [ ] `SurfaceWindow::Lock` · 005aeeb0 · legions 0057a930

### PixelOp<unsigned_long>
- [ ] `PixelOp<unsigned_long>::ConvertPixel` · 005aef30 · legions 0057a9b0

### PixelOp<unsigned_short>
- [ ] `PixelOp<unsigned_short>::ConvertPixel` · 005af0b0 · legions 0057ab30

### SurfaceWindow
- [ ] `SurfaceWindow::SurfaceWindow` · 005af230 · legions 0057acb0
- [ ] `SurfaceWindow::SurfaceWindow` · 005af260 · legions 0057ace0
- [ ] `SurfaceWindow::SurfaceWindow` · 005af2a0 · legions 0057ad20
- [ ] `SurfaceWindow::SurfaceWindow` · 005af310 · legions 0057ad90
- [ ] `SurfaceWindow::DrawCharacter` · 005af390 · legions 0057ae10

### SmartArray<PrefDisplayModeType>
- [ ] `SmartArray<PrefDisplayModeType>::AddToEnd` · 005af8f0 · legions 0057b430

### SurfaceWindow
- [ ] `SurfaceWindow::Unlock` · 005af950 · legions 0057b490
- [ ] `SurfaceWindow::Blit` · 005af9c0 · legions 0057b500
- [ ] `SurfaceWindow::Multiply` · 005b00a0 · legions 0057bbd0
- [ ] `SurfaceWindow::Fill` · 005b0350 · legions 0057be70
- [ ] `SurfaceWindow::FillArea` · 005b0530 · legions 0057c050
- [ ] `SurfaceWindow::FillArea` · 005b05e0 · legions 0057c100
- [ ] `SurfaceWindow::DrawLine` · 005b0620 · legions 0057c140
- [ ] `SurfaceWindow::DrawBox` · 005b0910
- [ ] `SurfaceWindow::DrawTextA` · 005b09a0 · legions 0057c430
- [ ] `SurfaceWindow::NotColorBits` · 005b0b40 · legions 0057c5c0
- [ ] `SurfaceWindow::GetColorFromPoint` · 005b0d80 · legions 0057c800

## `portal/client/src/render/core/texture/d3d/rendertextured3d.cpp.c` (0/19)

### RenderTextureD3D
- [ ] `RenderTextureD3D::GetSubObjects` · 005bb860 · legions 00584fc0
- [ ] `RenderTextureD3D::Get2DTextureD3D` · 005bb890 · legions 00584ff0
- [ ] `RenderTextureD3D::GetCubeTextureD3D` · 005bb8d0 · legions 00585030
- [ ] `RenderTextureD3D::GetLevelRef` · 005bb910 · legions 00585070
- [ ] `RenderTextureD3D::LoadTexture` · 005bb9d0 · legions 00585130
- [ ] `RenderTextureD3D::ReleaseD3DSurfaces` · 005bba50 · legions 005851b0
- [ ] `RenderTextureD3D::PurgeResource` · 005bbac0 · legions 00585220
- [ ] `RenderTextureD3D::CopyInto` · 005bbb20 · legions 00585280
- [ ] `RenderTextureD3D::ReleaseSubObjects` · 005bbbe0 · legions 00585340
- [ ] `RenderTextureD3D::CreateD3DTexture` · 005bbc40 · legions 005853a0
- [ ] `RenderTextureD3D::RestoreResource` · 005bbee0 · legions 00585640
- [ ] `RenderTextureD3D::Create2D` · 005bbff0
- [ ] `RenderTextureD3D::CreateCube` · 005bc030
- [ ] `RenderTextureD3D::Destroy` · 005bc070 · legions 005857d0

### RenderTexture
- [ ] `RenderTexture::RenderTexture` · 005bc0c0 · legions 00585820
- [ ] `RenderTexture::Allocate` · 005bc110
- [ ] `RenderTexture::GetDBOType` · 005bc120

### RenderTextureD3D
- [ ] `RenderTextureD3D::RenderTextureD3D` · 005bc190 · legions 005858f0
- [ ] `RenderTextureD3D::~RenderTextureD3D` · 005bc230 · legions 00585980

## `portal/client/src/render/core/texture/rendertexture.cpp.c` (0/29)

### RenderTexture
- [ ] `RenderTexture::Create2D` · 005bcaa0 · legions 00586200
- [ ] `RenderTexture::CreateCube` · 005bcaf0 · legions 00586260
- [ ] `RenderTexture::ShouldDropHighDetail` · 005bcb40 · legions 005862b0
- [ ] `RenderTexture::Refresh` · 005bcb60 · legions 005862d0
- [ ] `RenderTexture::SelectTextureFormat` · 005bcb90 · legions 00586300
- [ ] `RenderTexture::ReleaseSourceLevelReferences` · 005bcc20 · legions 00586390
- [ ] `RenderTexture::ReleaseSubObjects` · 005bcc60 · legions 005863d0
- [ ] `RenderTexture::GetDIDsForFilenames` · 005bcc80
- [-] `RenderTexture::LoadLevelResources` · 005bcd20 — text source loader, admin build only
- [ ] `RenderTexture::ConstructTexture` · 005bcdd0 · legions 00586490

### DBLevelInfo
- [ ] `DBLevelInfo::DBLevelInfo` · 005bcf80

### RenderTexture
- [ ] `RenderTexture::GetSubObjects` · 005bcfb0 · legions 00586650

### _anon_1CBD2415::AssertionDialogConfig
- [ ] `_anon_1CBD2415::AssertionDialogConfig::operator=` · 005bcff0 · legions 00586690

### RenderTexture
- [ ] `RenderTexture::Begin` · 005bd110 · legions 005867b0
- [ ] `RenderTexture::DestroySourceLevels` · 005bd160 · legions 00586810
- [-] `RenderTexture::ToFileNode` · 005bd1c0 — text source format (admin tools)

### SmartArray<DBLevelInfo>
- [ ] `SmartArray<DBLevelInfo>::operator=` · 005bd360 · legions 00586870
- [ ] `SmartArray<DBLevelInfo>::SetNElements` · 005bd400 · legions 00586910

### RenderTexture
- [ ] `RenderTexture::End` · 005bd430 · legions 00586940
- [ ] `RenderTexture::Allocator` · 005bd450
- [ ] `RenderTexture::Destroy` · 005bd490 · legions 005869a0
- [ ] `RenderTexture::SetNumSourceLevels` · 005bd4c0 · legions 005869d0
- [ ] `RenderTexture::DropUnwantedLevels` · 005bd4f0 · legions 00586a00
- [ ] `RenderTexture::CopyInto` · 005bd600 · legions 00586b10
- [ ] `RenderTexture::GetSubDataIDs` · 005bd750
- [ ] `RenderTexture::InitLoad` · 005bd820 · legions 00586d40
- [ ] `RenderTexture::StreamPack` · 005bd850 · legions 00586d70
- [-] `RenderTexture::LoadFromFileNode` · 005bd940 — text source loader, admin build only
- [-] `RenderTexture::FromFileNode` · 005bdb90 — text source format (admin tools)

## `portal/client/src/render/core/vertexbuffer/d3d/rendervertexbufferd3d.cpp.c` (0/12)

### RenderVertexBufferD3D
- [ ] `RenderVertexBufferD3D::Lock` · 005dc1c0 · legions 0059cfc0
- [ ] `RenderVertexBufferD3D::Begin` · 005dc210 · legions 0059d010
- [ ] `RenderVertexBufferD3D::SetupVertexStreams` · 005dc260 · legions 0059d060
- [ ] `RenderVertexBufferD3D::ReleaseVertexStreams` · 005dc2d0 · legions 0059d0d0
- [ ] `RenderVertexBufferD3D::End` · 005dc360 · legions 0059d160
- [ ] `RenderVertexBufferD3D::Shutdown` · 005dc370 · legions 0059d170
- [ ] `RenderVertexBufferD3D::UpdateVertexBuffer` · 005dc3d0
- [ ] `RenderVertexBufferD3D::AddDirtyRange` · 005dc520 · legions 0059d2b0
- [ ] `RenderVertexBufferD3D::CreateVertexStreams` · 005dc5f0 · legions 0059d380
- [ ] `RenderVertexBufferD3D::RenderIndexedPrimitives` · 005dc6c0 · legions 0059d450
- [ ] `RenderVertexBufferD3D::RenderPrimitives` · 005dc8b0 · legions 0059d5f0
- [ ] `RenderVertexBufferD3D::Startup` · 005dca40 · legions 0059d730

## `portal/client/src/render/core/vertexbuffer/d3d/rendervertexstreamd3d.cpp.c` (0/10)

### RenderVertexStreamD3D
- [ ] `RenderVertexStreamD3D::Begin` · 005dbd30 · legions 0059cb40
- [ ] `RenderVertexStreamD3D::CreateD3DVertexBuffer` · 005dbd70 · legions 0059cb80
- [ ] `RenderVertexStreamD3D::GetD3DVertexBuffer` · 005dbea0 · legions 0059ccb0
- [ ] `RenderVertexStreamD3D::FillData` · 005dbed0 · legions 0059cce0
- [ ] `RenderVertexStreamD3D::ResetVertexIndices` · 005dc040 · legions 0059ce40
- [ ] `RenderVertexStreamD3D::PurgeResource` · 005dc060 · legions 0059ce60
- [ ] `RenderVertexStreamD3D::RestoreResource` · 005dc0a0 · legions 0059cea0
- [ ] `RenderVertexStreamD3D::End` · 005dc0c0 · legions 0059cec0
- [ ] `RenderVertexStreamD3D::Create` · 005dc0f0 · legions 0059cef0
- [ ] `RenderVertexStreamD3D::Destroy` · 005dc160 · legions 0059cf60

## `portal/client/src/render/core/vertexbuffer/rendervertexbuffer.cpp.c` (2/17)

### RenderVertexBuffer
- [ ] `RenderVertexBuffer::Begin` · 005b6f90 · legions 005813c0
- [ ] `RenderVertexBuffer::End` · 005b6fb0 · legions 005813e0
- [ ] `RenderVertexBuffer::Startup` · 005b6fd0 · legions 00581400
- [ ] `RenderVertexBuffer::Shutdown` · 005b7080 · legions 005814b0
- [ ] `RenderVertexBuffer::UnlockAnimVertices` · 005b70c0 · legions 005814f0
- [ ] `RenderVertexBuffer::AnimateVertex` · 005b70e0 · legions 00581510
- [~] `RenderVertexBuffer::BlendVertices` · 005b7260 · legions 00581690 — BlendLitVertices on the GPU (ac2-gpu object.wgsl); no bounds update, ModelDetail<2 path not reproduced
- [ ] `RenderVertexBuffer::AnimateAllVertices` · 005b7530 · legions 00581c00
- [ ] `RenderVertexBuffer::Unlock` · 005b7590 · legions 00581930
- [x] `RenderVertexBuffer::IlluminateVertices` · 005b75b0 · legions 00581950
- [ ] `RenderVertexBuffer::RenderIndexBufferPrimitives` · 005b77a0 · legions 00581b40
- [ ] `RenderVertexBuffer::AnimateVertices` · 005b7860
- [ ] `RenderVertexBuffer::RenderIndexBufferUsingMaterial` · 005b7880 · legions 00581c50
- [x] `RenderVertexBuffer::RenderIndexedUsingMaterial` · 005b7a50 · legions 00581e10 — ac2-render object::pass / texture_matrix, ac2-gpu object
- [ ] `RenderVertexBuffer::RenderUsingMaterial` · 005b7c40 · legions 00581ff0
- [ ] `RenderVertexBuffer::RayIntersectsTriList` · 005b7e00 · legions 005821b0
- [ ] `RenderVertexBuffer::LockAnimVertices` · 005b8450 · legions 00582800

## `portal/client/src/render/decal/decaleffect.cpp.c` (0/9)

### DecalEffect
- [ ] `DecalEffect::ClipPolygonAgainstPlane` · 005dd100 · legions 0059ddf0
- [ ] `DecalEffect::DecalEffect` · 005de3e0 · legions 0059f170
- [ ] `DecalEffect::Think` · 005de430 · legions 0059f1c0
- [ ] `DecalEffect::CreateGeometry` · 005de500 · legions 0059f290
- [ ] `DecalEffect::~DecalEffect` · 005de7b0 · legions 0059f540
- [ ] `DecalEffect::ConstructDecal` · 005de7e0 · legions 0059f570
- [ ] `DecalEffect::ConstructLandscapeDecal` · 005dee50 · legions 0059fbe0
- [ ] `DecalEffect::ConstructDungeonDecal` · 005df1e0 · legions 0059ff70
- [ ] `DecalEffect::Init` · 005df350 · legions 005a00e0

## `portal/client/src/render/decal/decaleffectgroup.cpp.c` (0/5)

### DecalEffectGroup
- [ ] `DecalEffectGroup::Init` · 005db6c0
- [ ] `DecalEffectGroup::~DecalEffectGroup` · 005db730 · legions 0059c540
- [ ] `DecalEffectGroup::Think` · 005db7b0 · legions 0059c5c0
- [ ] `DecalEffectGroup::Render` · 005db850 · legions 0059c660
- [ ] `DecalEffectGroup::CreateDecalEffect` · 005db920 · legions 0059c730

## `portal/client/src/render/decal/decaleffectmanager.cpp.c` (0/6)

### DecalEffectManager
- [ ] `DecalEffectManager::Init` · 005c38c0 · legions 0058a9d0
- [ ] `DecalEffectManager::DecalEffectManager` · 005c3900 · legions 0058aa10
- [ ] `DecalEffectManager::~DecalEffectManager` · 005c3920 · legions 0058aa30
- [ ] `DecalEffectManager::Think` · 005c39b0 · legions 0058aac0
- [ ] `DecalEffectManager::Render` · 005c3a30 · legions 0058ab40
- [ ] `DecalEffectManager::CreateDecalEffect` · 005c3c10 · legions 0058ad20

## `portal/client/src/render/flare/lensflareeffect.cpp.c` (0/14)

### LensFlarePoint
- [ ] `LensFlarePoint::LensFlarePoint` · 005b6050 · legions 005809f0
- [ ] `LensFlarePoint::~LensFlarePoint` · 005b6070 · legions 00580a10

### LensFlareEffect
- [ ] `LensFlareEffect::LensFlareEffect` · 005b6090 · legions 00580a30
- [ ] `LensFlareEffect::~LensFlareEffect` · 005b60c0 · legions 00580a60
- [ ] `LensFlareEffect::Destroy` · 005b60f0 · legions 00580a90
- [ ] `LensFlareEffect::InitNew` · 005b6130 · legions 00580ad0
- [ ] `LensFlareEffect::GetSubDataIDs` · 005b6160 · legions 00580b00
- [ ] `LensFlareEffect::StreamPack` · 005b61f0 · legions 00580b90
- [ ] `LensFlareEffect::CopyInto` · 005b62f0 · legions 00580c90
- [ ] `LensFlareEffect::Draw` · 005b63d0 · legions 00580d70

### SmartArray<LensFlarePoint>
- [ ] `SmartArray<LensFlarePoint>::AddToEnd` · 005b6980 · legions 00581320

### LensFlareEffect
- [ ] `LensFlareEffect::AddPoint` · 005b69e0 · legions 00581380
- [-] `LensFlareEffect::FromFileNode` · 005b6a20 — text source format (admin tools)
- [-] `LensFlareEffect::ToFileNode` · 005b6d30 — text source format (admin tools)

## `portal/client/src/render/gamerender.cpp.c` (0/2)

### GameRender
- [ ] `GameRender::Init` · 005b84e0
- [ ] `GameRender::Cleanup` · 005b8740

## `portal/client/src/render/particle/particle.cpp.c` (0/41)

### ParticleWrapper
- [ ] `ParticleWrapper::ParticleWrapper` · 005dca80 · legions 0059d770

### ParticleFormatInfo
- [ ] `ParticleFormatInfo::ParticleFormatInfo` · 005dcaa0 · legions 0059d790

### ParticleWrapper
- [ ] `ParticleWrapper::GetScale` · 005dcc70 · legions 0059d960
- [ ] `ParticleWrapper::SetScale` · 005dcc80 · legions 0059d970
- [ ] `ParticleWrapper::GetTimeCreated` · 005dcc90 · legions 0059d980
- [ ] `ParticleWrapper::SetTimeCreated` · 005dcca0 · legions 0059d990
- [ ] `ParticleWrapper::GetStartOrigin` · 005dccb0 · legions 0059d9a0
- [ ] `ParticleWrapper::SetStartOrigin` · 005dccc0 · legions 0059d9b0
- [ ] `ParticleWrapper::GetOrigin` · 005dcce0 · legions 0059d9d0
- [ ] `ParticleWrapper::SetOrigin` · 005dccf0 · legions 0059d9e0
- [ ] `ParticleWrapper::GetWorldOrigin` · 005dcd10 · legions 0059da00
- [ ] `ParticleWrapper::SetWorldOrigin` · 005dcd20 · legions 0059da10
- [ ] `ParticleWrapper::GetPreviousOrigin` · 005dcd40 · legions 0059da30
- [ ] `ParticleWrapper::SetPreviousOrigin` · 005dcd50 · legions 0059da40
- [ ] `ParticleWrapper::GetVelocity` · 005dcd70 · legions 0059da60
- [ ] `ParticleWrapper::SetVelocity` · 005dcd80 · legions 0059da70
- [ ] `ParticleWrapper::GetStreak` · 005dcda0 · legions 0059da90
- [ ] `ParticleWrapper::SetStreak` · 005dcdb0 · legions 0059daa0
- [ ] `ParticleWrapper::GetStreakOrigin` · 005dcdc0 · legions 0059dab0
- [ ] `ParticleWrapper::SetStreakOrigin` · 005dcdd0 · legions 0059dac0
- [ ] `ParticleWrapper::GetPreviousStreakOrigin` · 005dcdf0 · legions 0059dae0
- [ ] `ParticleWrapper::SetPreviousStreakOrigin` · 005dce00 · legions 0059daf0
- [ ] `ParticleWrapper::GetRotation` · 005dce20 · legions 0059db10
- [ ] `ParticleWrapper::SetRotation` · 005dce30 · legions 0059db20
- [ ] `ParticleWrapper::GetWorldRotation` · 005dce50 · legions 0059db40
- [ ] `ParticleWrapper::SetWorldRotation` · 005dce60 · legions 0059db50
- [ ] `ParticleWrapper::GetStartRotation` · 005dce80 · legions 0059db70
- [ ] `ParticleWrapper::SetStartRotation` · 005dce90 · legions 0059db80
- [ ] `ParticleWrapper::GetStartWorldRotation` · 005dceb0 · legions 0059dba0
- [ ] `ParticleWrapper::SetStartWorldRotation` · 005dcec0 · legions 0059dbb0
- [ ] `ParticleWrapper::GetRotateVelocity` · 005dcee0 · legions 0059dbd0
- [ ] `ParticleWrapper::SetRotateVelocity` · 005dcef0 · legions 0059dbe0
- [ ] `ParticleWrapper::GetKeyframeFlags` · 005dcf10 · legions 0059dc00
- [ ] `ParticleWrapper::SetKeyframeFlags` · 005dcf20 · legions 0059dc10
- [ ] `ParticleWrapper::GetKeyframeInfo` · 005dcf30 · legions 0059dc20
- [ ] `ParticleWrapper::GetWaveformVector` · 005dcf40 · legions 0059dc30
- [ ] `ParticleWrapper::SetWaveformVector` · 005dcf50 · legions 0059dc40
- [ ] `ParticleWrapper::GetPointIndex` · 005dcf60 · legions 0059dc50
- [ ] `ParticleWrapper::SetPointIndex` · 005dcf70 · legions 0059dc60
- [ ] `ParticleWrapper::GetPointSpeed` · 005dcf90 · legions 0059dc80
- [ ] `ParticleWrapper::SetPointSpeed` · 005dcfa0 · legions 0059dc90

## `portal/client/src/render/particle/particleemitter.cpp.c` (0/40)

### ParticleEmitter
- [ ] `ParticleEmitter::SetLastDischargeTime` · 005d07a0 · legions 00593e90

### FillVerts
- [ ] `FillVerts::GetNullRotationMatrix` · 005d07b0
- [ ] `FillVerts::GetWorldRotationMatrix` · 005d07f0
- [ ] `FillVerts::GetRotationMatrix` · 005d0840
- [ ] `FillVerts::GetVertexPositions` · 005d0880 · legions 00593f70
- [ ] `FillVerts::GetStreakedVertexPositions` · 005d0940
- [ ] `FillVerts::RotateVertex` · 005d0f50

### ParticleBuffer
- [ ] `ParticleBuffer::Allocate` · 005d1000 · legions 005946f0

### ParticleEmitter
- [ ] `ParticleEmitter::ParticleEmitter` · 005d1060 · legions 00594750
- [ ] `ParticleEmitter::~ParticleEmitter` · 005d1150 · legions 00594840
- [ ] `ParticleEmitter::GetDesc` · 005d11b0 · legions 005948a0
- [ ] `ParticleEmitter::GetReadyToDie` · 005d11d0 · legions 005948c0
- [ ] `ParticleEmitter::DetermineDataFlags` · 005d1250 · legions 00594940
- [ ] `ParticleEmitter::GetParticleNumVerts` · 005d1540 · legions 00594c30
- [ ] `ParticleEmitter::CalculateInitialOrigin` · 005d1580 · legions 00594c70
- [ ] `ParticleEmitter::DoWaveform` · 005d1850 · legions 00594f40
- [ ] `ParticleEmitter::GetMass` · 005d1980 · legions 00595070
- [ ] `ParticleEmitter::DoPhysics` · 005d1a20 · legions 00595110
- [ ] `ParticleEmitter::DoRepulse` · 005d1cb0 · legions 005953a0
- [ ] `ParticleEmitter::DoPointFile` · 005d2150 · legions 00595840
- [ ] `ParticleEmitter::DoPointFileLerp` · 005d2290 · legions 00595980

### FillVerts
- [ ] `FillVerts::GetOrigin` · 005d2550 · legions 00595c40
- [ ] `FillVerts::GetBlendedOrigin` · 005d2570
- [ ] `FillVerts::GetNullStreakOrigin` · 005d2600
- [ ] `FillVerts::GetStreakOrigin` · 005d2620
- [ ] `FillVerts::GetBlendedStreakOrigin` · 005d2640
- [ ] `FillVerts::GetLighting` · 005d26d0

### ParticleEmitter
- [ ] `ParticleEmitter::FillVertexBuffer` · 005d28a0 · legions 00595f90
- [ ] `ParticleEmitter::SetPosition` · 005d3690 · legions 00596d80
- [ ] `ParticleEmitter::ApplyKeyframe` · 005d3850 · legions 00596f40
- [ ] `ParticleEmitter::ApplyKeyframes` · 005d3d10 · legions 00597400
- [ ] `ParticleEmitter::ParticleThink` · 005d3de0 · legions 005974d0
- [ ] `ParticleEmitter::Think` · 005d40a0 · legions 00597790
- [ ] `ParticleEmitter::DischargeParticle` · 005d42a0 · legions 00597990
- [ ] `ParticleEmitter::SetupKeyframes` · 005d4bb0 · legions 005982a0
- [ ] `ParticleEmitter::Initialize` · 005d4e30 · legions 00598520
- [ ] `ParticleEmitter::Reset` · 005d4ed0 · legions 005985c0
- [ ] `ParticleEmitter::EmitTimedParticles` · 005d4f00 · legions 005985f0
- [ ] `ParticleEmitter::EmitDistanceParticles` · 005d5280 · legions 00598970
- [ ] `ParticleEmitter::EmitParticles` · 005d58a0 · legions 00598f90

## `portal/client/src/render/particle/particlerepulsor.cpp.c` (0/4)

### ParticleRepulsor
- [ ] `ParticleRepulsor::ParticleRepulsor` · 005c3790 · legions 0058a8a0
- [ ] `ParticleRepulsor::GetIsReadyToDie` · 005c37f0 · legions 0058a900
- [ ] `ParticleRepulsor::GetIsActive` · 005c3820 · legions 0058a930
- [ ] `ParticleRepulsor::SetPosition` · 005c3870 · legions 0058a980

## `portal/client/src/render/particle/particlesystem.cpp.c` (0/22)

### ParticleSystem
- [ ] `ParticleSystem::AddRef` · 005c22a0 · legions 005894e0
- [ ] `ParticleSystem::SetCanBeShared` · 005c22b0 · legions 005894f0
- [ ] `ParticleSystem::GetForceDraw` · 005c22c0 · legions 00589500
- [ ] `ParticleSystem::Begin` · 005c22e0 · legions 00589520
- [ ] `ParticleSystem::GetExternalForces` · 005c2350 · legions 00589590

### Position
- [ ] `Position::Position` · 005c2520 · legions 00475a40

### ParticleSystem
- [ ] `ParticleSystem::CanBeShared` · 005c25b0 · legions 005897b0
- [ ] `ParticleSystem::GetReadyToDie` · 005c25f0 · legions 005897f0
- [ ] `ParticleSystem::Destroy` · 005c2680 · legions 00589880
- [ ] `ParticleSystem::Start` · 005c2700 · legions 00589900
- [ ] `ParticleSystem::Think` · 005c2770 · legions 00589970
- [ ] `ParticleSystem::GetLightingOrigin` · 005c28b0 · legions 00589a80

### ParticleSystem::ParticleRenderPos
- [ ] `ParticleSystem::ParticleRenderPos::operator=` · 005c2910 · legions 00589ae0

### ParticleSystem
- [ ] `ParticleSystem::ParticleSystem` · 005c2a40 · legions 00589c10
- [ ] `ParticleSystem::~ParticleSystem` · 005c2ad0 · legions 00589ca0
- [ ] `ParticleSystem::Release` · 005c2b30 · legions 00589d00
- [ ] `ParticleSystem::FillVertexBuffer` · 005c2bf0 · legions 00589dc0
- [ ] `ParticleSystem::RenderAtPosition` · 005c2e80 · legions 0058a050
- [ ] `ParticleSystem::Render` · 005c3250 · legions 0058a420

### SmartArray<ParticleSystem::ParticleRenderPos>
- [ ] `SmartArray<ParticleSystem::ParticleRenderPos>::AddToEnd` · 005c3450 · legions 0058a560

### ParticleSystem
- [ ] `ParticleSystem::Create` · 005c34e0 · legions 0058a5f0
- [ ] `ParticleSystem::Queue` · 005c3600 · legions 0058a710

## `portal/client/src/render/particle/psmanager.cpp.c` (0/15)

### PSManager
- [ ] `PSManager::Startup` · 005c0540 · legions 005877e0

### PSManager::AutonomousSystemInfo
- [ ] `PSManager::AutonomousSystemInfo::AutonomousSystemInfo` · 005c0670

### PSManager
- [ ] `PSManager::DestroyRepulsor` · 005c0700 · legions 005879a0
- [ ] `PSManager::CreateParticleSystem` · 005c0760 · legions 00587a00
- [ ] `PSManager::UnregisterSystem` · 005c0850 · legions 00587ae0

### PSManager::AutonomousSystemInfo
- [ ] `PSManager::AutonomousSystemInfo::operator=` · 005c0890 · legions 00587b20

### SmartArray<PSManager::AutonomousSystemInfo>
- [ ] `SmartArray<PSManager::AutonomousSystemInfo>::operator=` · 005c09c0 · legions 00587c50

### PSManager
- [ ] `PSManager::UnregisterAutonomousSystem` · 005c0ae0 · legions 00587d70

### SmartArray<PSManager::AutonomousSystemInfo>
- [ ] `SmartArray<PSManager::AutonomousSystemInfo>::AddToEnd` · 005c0b70 · legions 00587e00

### PSManager
- [ ] `PSManager::Shutdown` · 005c0c00 · legions 00587e90
- [ ] `PSManager::CreateRepulsor` · 005c0d20 · legions 00587fb0
- [ ] `PSManager::RenderParticleSystems` · 005c0db0 · legions 00588040
- [ ] `PSManager::RegisterSystem` · 005c0e60 · legions 005880f0
- [ ] `PSManager::RegisterAutonomousSystem` · 005c0ec0 · legions 00588150
- [ ] `PSManager::CreateAutonomousParticleSystem` · 005c0f80 · legions 00588210

## `portal/client/src/render/streak/streakeffect.cpp.c` (0/10)

### StreakEffect
- [ ] `StreakEffect::Activate` · 005c13d0 · legions 00588640
- [ ] `StreakEffect::Deactivate` · 005c13f0 · legions 00588660
- [ ] `StreakEffect::Render` · 005c1410 · legions 00588680

### SmartArray<StreakSegment>
- [ ] `SmartArray<StreakSegment>::RemoveOrderedByIndex` · 005c1560 · legions 005887c0

### StreakEffect
- [ ] `StreakEffect::StreakEffect` · 005c15e0 · legions 00588840
- [ ] `StreakEffect::~StreakEffect` · 005c1850 · legions 00588a90
- [ ] `StreakEffect::CreateGeometry` · 005c1900 · legions 00588b40
- [ ] `StreakEffect::SetPosition` · 005c1ed0 · legions 00589110

### SmartArray<StreakSegment>
- [ ] `SmartArray<StreakSegment>::AddToEnd` · 005c1f60 · legions 005891a0

### StreakEffect
- [ ] `StreakEffect::Think` · 005c1f90 · legions 005891d0

## `portal/client/src/render/streak/streakeffectmanager.cpp.c` (0/6)

### StreakEffectManager
- [ ] `StreakEffectManager::~StreakEffectManager` · 005c1040 · legions 005882b0
- [ ] `StreakEffectManager::DestroyStreakEffect` · 005c10d0 · legions 00588340
- [ ] `StreakEffectManager::UpdateStreakEffects` · 005c1150 · legions 005883c0
- [ ] `StreakEffectManager::RenderStreakEffects` · 005c11a0 · legions 00588410
- [ ] `StreakEffectManager::CreateStreakEffect` · 005c1240 · legions 005884b0
- [ ] `StreakEffectManager::StreakEffectManager` · 005c12d0 · legions 00588540

## `portal/client/src/scenerender/frustum.cpp.c` (0/7)

### Frustum
- [ ] `Frustum::Generate` · 00572cd0 · legions 00542cd0
- [ ] `Frustum::CheckRelationship` · 00572f40 · legions 00542f40

### CellFrustum
- [ ] `CellFrustum::operator=` · 005730b0 · legions 005430b0

### Frustum
- [ ] `Frustum::Frustum` · 00573110 · legions 00543110
- [ ] `Frustum::Generate` · 00573150 · legions 00543150

### CellFrustum
- [ ] `CellFrustum::CellFrustum` · 00573390 · legions 00543390

### Frustum
- [ ] `Frustum::CheckPolygon` · 005733e0 · legions 005433e0

## `portal/client/src/scenerender/scenecamera.cpp.c` (4/11)

### SceneCamera
- [ ] `SceneCamera::~SceneCamera` · 00572820 · legions 0053a560
- [ ] `SceneCamera::ApplyProperty` · 00572840
- [ ] `SceneCamera::BuildOptionalProperties` · 005728f0
- [x] `SceneCamera::SetFOV` · 00572aa0 · legions 0053a7e0
- [ ] `SceneCamera::SetVerticalScale` · 00572ab0 · legions 0053a7f0
- [x] `SceneCamera::slerp` · 00572ac0 · legions 0053a800
- [ ] `SceneCamera::SceneCamera` · 00572b00 · legions 0053a840
- [ ] `SceneCamera::ApplyPropertyCollection` · 00572bc0
- [x] `SceneCamera::SetPreviousPosition` · 00572c70 · legions 0053a9b0
- [ ] `SceneCamera::SetRenderPosition` · 00572c90 · legions 0053a9d0
- [x] `SceneCamera::SetCurrentPosition` · 00572cb0 · legions 0053a9f0

## `portal/client/src/scenerender/scenetool.cpp.c` (5/100)

### SceneTool
- [ ] `SceneTool::UsingSceneMotionBlur` · 0056a5d0 · legions 0053aa10

### MaterialLayer
- [ ] `MaterialLayer::SetDiffuse` · 0056a600 · legions 0053aa40

### SceneTool
- [ ] `SceneTool::StartupDebugHUD` · 0056a630 · legions 0053aa70
- [ ] `SceneTool::ShowDebugHUD` · 0056a7f0
- [ ] `SceneTool::SetGammaBrightness` · 0056a810 · legions 0053ac30
- [ ] `SceneTool::InitFromGlobalPreferences` · 0056a830 · legions 0053ac50
- [ ] `SceneTool::InitTextureTargets` · 0056a910 · legions 0053ad30
- [ ] `SceneTool::SetWorldCellID` · 0056aac0 · legions 0053aee0

### (free functions)
- [ ] `SwapWorldMatrixForwardAndUp` · 0056ab00 · legions 0053af20
- [ ] `SwapViewMatrixForwardAndUp` · 0056ab90 · legions 0053afb0

### SceneTool
- [ ] `SceneTool::SetWorldMatrix` · 0056ac20 · legions 0053b040
- [ ] `SceneTool::SetWorldTransform` · 0056ac70 · legions 0053b090
- [x] `SceneTool::SetMatrixStack` · 0056ad20 · legions 0053b140 — ac2-gpu ObjectInstance.pose (part matrices, bone storage buffer)
- [ ] `SceneTool::TransformVector` · 0056ad40 · legions 0053b160
- [ ] `SceneTool::TransformPixelToClipPoint` · 0056af60 · legions 0053b380
- [ ] `SceneTool::TransformClipPointToPixel` · 0056afb0 · legions 0053b3d0
- [ ] `SceneTool::ViewportToClip` · 0056b010 · legions 0053b430
- [ ] `SceneTool::ClipToViewport` · 0056b080 · legions 0053b4a0

### LightSorterPredicate
- [ ] `LightSorterPredicate::operator()` · 0056b100 · legions 0053b520

### SceneTool
- [x] `SceneTool::SetAmbientLight` · 0056b150 · legions 0053b570 — ac2-render object::pass / texture_matrix, ac2-gpu object
- [x] `SceneTool::SetOutdoorAmbient` · 0056b170 · legions 0053b590 — ac2-render object::pass / texture_matrix, ac2-gpu object
- [ ] `SceneTool::SetFogProperties` · 0056b1a0 · legions 0053b5c0
- [x] `SceneTool::IlluminateMeshVertices` · 0056b1c0 · legions 0053b5e0
- [ ] `SceneTool::PrepareGenericRenderTarget0` · 0056b1e0 · legions 0053b600
- [ ] `SceneTool::PrepareGenericRenderTarget1` · 0056b250 · legions 0053b670
- [ ] `SceneTool::PrepareGenericRenderTarget2` · 0056b2c0 · legions 0053b6e0
- [ ] `SceneTool::PrepareGenericDepthTarget` · 0056b330 · legions 0053b750
- [ ] `SceneTool::GenerateGlobalShadowVolume` · 0056b390 · legions 0053b7b0
- [ ] `SceneTool::StartScreenshotCapture` · 0056b6b0 · legions 0053ba90
- [ ] `SceneTool::FinishScreenshotCapture` · 0056b7a0 · legions 0053bb80
- [ ] `SceneTool::GetNumSubScreenshots` · 0056b7c0 · legions 0053bba0
- [ ] `SceneTool::FinishSubScreenshot` · 0056b7d0 · legions 0053bbb0
- [ ] `SceneTool::StartReflectionMap` · 0056ba00 · legions 0053bde0
- [ ] `SceneTool::FinishReflectionMap` · 0056ba60 · legions 0053be40
- [ ] `SceneTool::StartWaterBumpMap` · 0056baa0 · legions 0053be80
- [ ] `SceneTool::FinishWaterBumpMap` · 0056bb20 · legions 0053bf00
- [ ] `SceneTool::IdentityMatrices` · 0056bb50 · legions 0053bf30
- [ ] `SceneTool::StoreMatrices` · 0056bc90 · legions 0053c070
- [ ] `SceneTool::RestoreMatrices` · 0056bcd0 · legions 0053c0b0
- [ ] `SceneTool::SetBackgroundColor` · 0056bd90 · legions 0053c170
- [ ] `SceneTool::StartFrame` · 0056bdd0 · legions 0053c1b0
- [ ] `SceneTool::CheckObjectSelection` · 0056be60 · legions 0053c240
- [ ] `SceneTool::ShouldSkipRenderNow` · 0056bf90 · legions 0053c370
- [ ] `SceneTool::SetSelectionID` · 0056bfb0 · legions 0053c390
- [ ] `SceneTool::GetDistortionMap` · 0056bfe0 · legions 0053c3c0
- [ ] `SceneTool::GetFrameBufferTexture` · 0056c270 · legions 0053c640
- [ ] `SceneTool::PurgeOldGraphicsResources` · 0056c290 · legions 0053c660

### BBox
- [ ] `BBox::MakeValid` · 0056c570 · legions 0053c950

### SceneTool
- [ ] `SceneTool::SetupView` · 0056c5d0 · legions 0053c9b0
- [ ] `SceneTool::SetupPerspectiveProjection` · 0056c730 · legions 0053cb10
- [ ] `SceneTool::SetupPerpspectiveFromFrustum` · 0056c880 · legions 0053cc60
- [ ] `SceneTool::SetupOrthographicProjection` · 0056c980 · legions 0053cd60
- [ ] `SceneTool::SetupOrthographicProjection` · 0056ca70 · legions 0053ce50
- [ ] `SceneTool::InverseRotate` · 0056cb90 · legions 0053cf70
- [ ] `SceneTool::InverseTransform` · 0056cd40 · legions 0053d120
- [ ] `SceneTool::GetLight` · 0056cee0 · legions 0053d2c0
- [ ] `SceneTool::UnlinkAllLights` · 0056cf20 · legions 0053d300
- [ ] `SceneTool::DrawLine` · 0056d000 · legions 0053d3e0
- [ ] `SceneTool::DrawTexturedTriangle` · 0056d0b0 · legions 0053d490
- [ ] `SceneTool::DrawTriangle` · 0056d1e0 · legions 0053d5c0
- [ ] `SceneTool::DrawTexturedPolygon` · 0056d240 · legions 0053d620
- [ ] `SceneTool::BuildCylinderMesh` · 0056d2b0 · legions 0053d690
- [ ] `SceneTool::BuildSphereMesh` · 0056dc60 · legions 0053e040
- [ ] `SceneTool::StartSelection` · 0056e450 · legions 0053e830
- [ ] `SceneTool::DrawScreenQuadUsingMaterial` · 0056e5f0 · legions 0053e9d0
- [ ] `SceneTool::Think` · 0056e860 · legions 0053ec40
- [ ] `SceneTool::GetPixelScanResult` · 0056e880 · legions 0053ec60
- [ ] `SceneTool::SetupCamera` · 0056ea20 · legions 0053ee00
- [ ] `SceneTool::GetPointLightShadowVector` · 0056ed50 · legions 0053f130
- [ ] `SceneTool::DisableSunlight` · 0056eef0 · legions 0053f2d0
- [~] `SceneTool::SetSunlight` · 0056ef40 · legions 0053f320 — object::ObjectLights; static/volumetric shadow strengths not used yet
- [ ] `SceneTool::DrawLine` · 0056f230 · legions 0053f610
- [~] `SceneTool::DrawScreenQuad` · 0056f260 · legions 0053f640 — untextured colour quad (ac2-gpu ScreenQuad) only
- [ ] `SceneTool::SwitchTextureTargets` · 0056f2e0 · legions 0053f6c0
- [ ] `SceneTool::Render` · 0056f440 · legions 0053f820

### SmartArray<ObjectSelection>
- [ ] `SmartArray<ObjectSelection>::AddToEnd` · 0056f4f0 · legions 0053f930
- [ ] `SmartArray<ObjectSelection>::InsertOrderedByIndex` · 0056f560 · legions 0053f9a0

### SceneTool
- [ ] `SceneTool::UpdateDebugHUD` · 0056f770 · legions 0053fbb0
- [ ] `SceneTool::RefreshGraphicsResources` · 0056fb70
- [ ] `SceneTool::Shutdown` · 0056fc00 · legions 00540040
- [ ] `SceneTool::SetupCamera` · 0056fe60 · legions 005402a0
- [ ] `SceneTool::StartShadowMap` · 0056fea0 · legions 005402e0
- [ ] `SceneTool::FinishShadowMap` · 00570300 · legions 00540740
- [ ] `SceneTool::RenderGlobalVolumetricShadows` · 005703e0 · legions 00540820
- [ ] `SceneTool::FinishSelection` · 005706d0 · legions 00540b10
- [ ] `SceneTool::SelectObject` · 00570730 · legions 00540b70

### SmartArray<PixelScanRequest>
- [ ] `SmartArray<PixelScanRequest>::AddToEnd` · 005708c0 · legions 00540d00

### MaterialLayer
- [ ] `MaterialLayer::InsertStage` · 00570970 · legions 00540e20

### RenderMaterial
- [ ] `RenderMaterial::InsertLayer` · 00570990 · legions 00540e40

### SceneTool
- [ ] `SceneTool::CreateDefaultMaterial` · 005709b0 · legions 00540e60
- [ ] `SceneTool::Startup` · 00570f00 · legions 005413b0
- [ ] `SceneTool::LinkLight` · 00571630 · legions 00541ae0
- [ ] `SceneTool::AddDynamicLights` · 005716c0 · legions 00541b70
- [ ] `SceneTool::RequestPixelScan` · 00571860 · legions 00541d10
- [ ] `SceneTool::ScanRequestedPixels` · 005718c0 · legions 00541d70
- [ ] `SceneTool::EndFrame` · 00571a40 · legions 00541ef0
- [x] `SceneTool::SortLights` · 00571da0 · legions 00542250 — ac2-render object::cell / point_lights
- [~] `SceneTool::CacheStaticLights` · 00571ec0 · legions 00542370 — static point lights: weight sort, six cap, range test, indoor/outdoor; sunlight-type and dynamic lights not yet
- [~] `SceneTool::ApplyLights` · 00572110 · legions 005425c0 — outdoor sun only; dynamic/point lights and shadow direction not yet
- [ ] `SceneTool::GetLightsForPosition` · 005726b0 · legions 00542b60

## `portal/client/src/smartbox/worldrender/cellrender.cpp.c` (0/2)

### CellRender
- [ ] `CellRender::CellRender` · 00583480 · legions 00552bf0
- [ ] `CellRender::AddCellSounds` · 005834d0 · legions 00552c40

## `portal/client/src/smartbox/worldrender/frillrender.cpp.c` (0/24)

### FrillRender
- [ ] `FrillRender::OnContextFinished` · 00580990

### FrillEntry
- [ ] `FrillEntry::operator=` · 005809e0 · legions 00550290

### FrillRender
- [ ] `FrillRender::DestroyFrillMesh` · 00580c30 · legions 00550430
- [ ] `FrillRender::ClearFrillMesh` · 00580cb0 · legions 005504b0
- [ ] `FrillRender::InitFrillMesh` · 005810b0 · legions 005508b0
- [ ] `FrillRender::RenderFrillMesh` · 00581180 · legions 00550980

### FrillListType
- [ ] `FrillListType::~FrillListType` · 00581800 · legions 00551000

### HashIterator<unsigned_long,BaseList<unsigned_long,11>_>
- [ ] `HashIterator<unsigned_long,BaseList<unsigned_long,11>_>::next` · 00581840 · legions 00536680

### SmartArray<RGBAColor>
- [ ] `SmartArray<RGBAColor>::AddToEnd` · 00581880 · legions 00551040

### FrillRender
- [ ] `FrillRender::GetSubDataIDs` · 005818e0 · legions 005510a0
- [ ] `FrillRender::ClearAllRenderLists` · 00581990 · legions 00551150

### FrillRenderType
- [~] `FrillRenderType::AddPos` · 00581a00 · legions 005511c0 — frills selected by radius around the camera cell; radius is the constructor default (script sets it)

### SmartArray<FrillEntry>
- [ ] `SmartArray<FrillEntry>::AddToEnd` · 00581be0 · legions 005513a0

### FrillRender
- [ ] `FrillRender::Clear` · 00581d00 · legions 005514c0
- [ ] `FrillRender::RemoveOldFrillObjects` · 00581e90 · legions 00551650

### FrillListType
- [ ] `FrillListType::Add` · 00582010 · legions 005517d0

### FrillRender
- [ ] `FrillRender::~FrillRender` · 00582170 · legions 00551930
- [ ] `FrillRender::EnableFrillObjects` · 00582220 · legions 005519d0
- [~] `FrillRender::AddNewFrillObjects` · 005823e0 · legions 00551b90 — frills selected by radius around the camera cell; radius is the constructor default (script sets it)
- [ ] `FrillRender::FrillRender` · 00582670 · legions 00551e20
- [ ] `FrillRender::UpdateFrillPosition` · 00582760 · legions 00551f00
- [ ] `FrillRender::Render` · 00582890 · legions 00552030
- [~] `FrillRender::RefreshFrillMesh` · 005828e0 · legions 00552050 — ac2-render frill::select_frills + per-instance frill normal; shadow colour white (no static shadow map), drawn as instances not one merged mesh
- [~] `FrillRender::UpdateRenderList` · 005832a0 · legions 00552a10 — frills selected by radius around the camera cell; radius is the constructor default (script sets it)

## `portal/client/src/smartbox/worldrender/lscapelighting.cpp.c` (1/19)

### RGBAColor
- [ ] `RGBAColor::Normalize` · 0058f080 · legions 0055e6f0
- [ ] `RGBAColor::IsValid` · 0058f130 · legions 0055e7a0

### LScapeLighting
- [x] `LScapeLighting::SetDayLighting` · 0058f1d0 · legions 0055e840
- [ ] `LScapeLighting::SetDayFog` · 0058f210
- [ ] `LScapeLighting::CornerInterp` · 0058f230 · legions 0055e8a0
- [ ] `LScapeLighting::Combine` · 0058f890 · legions 0055ef00
- [ ] `LScapeLighting::Combine` · 0058f910 · legions 0055ef80
- [ ] `LScapeLighting::CalcDayLight` · 0058f9a0 · legions 0055f010
- [ ] `LScapeLighting::CalcCombineFog` · 0058fac0 · legions 0055f130
- [ ] `LScapeLighting::CalcWaterLight` · 0058fc10 · legions 0055f280

### DBObjGrabber<CSoundDesc>
- [ ] `DBObjGrabber<CSoundDesc>::~DBObjGrabber<CSoundDesc>` · 00590150 · legions 00478460

### LScapeLighting
- [~] `LScapeLighting::UpdateLighting` · 005901f0 · legions 0055f840 — day light and fog; no region fog blocks (CalcCombineFog) or water light
- [ ] `LScapeLighting::GetFogCorners` · 00590670 · legions 0055fcc0
- [ ] `LScapeLighting::GetWaterInfo` · 00590840 · legions 0055fe90

### SmartArray<WaterLightInfo>
- [ ] `SmartArray<WaterLightInfo>::~SmartArray<WaterLightInfo>` · 00590900 · legions 0055ff50

### LScapeLighting
- [ ] `LScapeLighting::LScapeLighting` · 00590940 · legions 0055ff90
- [ ] `LScapeLighting::~LScapeLighting` · 00590b10 · legions 00560160
- [ ] `LScapeLighting::UpdateInfo` · 00590b60 · legions 005601b0
- [ ] `LScapeLighting::ChangeCell` · 00590bc0 · legions 00560210

## `portal/client/src/smartbox/worldrender/lscapematerialinfo.cpp.c` (10/14)

### RenderTextureLocal
- [ ] `RenderTextureLocal::GetGlobalOrLocal` · 00590c10 · legions 00560260

### LScapeMaterialInfo
- [ ] `LScapeMaterialInfo::~LScapeMaterialInfo` · 00590c60 · legions 005602b0
- [x] `LScapeMaterialInfo::CornerMatch` · 00590c80 · legions 005602d0
- [x] `LScapeMaterialInfo::GetCombineMaterials` · 00590cb0 · legions 00560300
- [x] `LScapeMaterialInfo::GetAlphaTexture` · 00590d80 · legions 005603d0
- [x] `LScapeMaterialInfo::CalcUVInfo` · 00590de0 · legions 00560410
- [x] `LScapeMaterialInfo::Match` · 00590ea0 · legions 005604d0
- [x] `LScapeMaterialInfo::CalcRotUVs` · 00591020 · legions 00560650
- [ ] `LScapeMaterialInfo::LScapeMaterialInfo` · 005910d0 · legions 00560700
- [x] `LScapeMaterialInfo::SetUVs` · 005911e0 · legions 00560810
- [x] `LScapeMaterialInfo::Apply` · 00591270 · legions 005608a0
- [~] `LScapeMaterialInfo::CombineMaterials` · 005912d0 · legions 00560900 — base and alpha stages; no shadow-map or cloud-shadow stages
- [x] `LScapeMaterialInfo::MultiCombine` · 005919d0 · legions 00561000
- [x] `LScapeMaterialInfo::Create` · 00591a40 · legions 00561070

## `portal/client/src/smartbox/worldrender/lscaperender.cpp.c` (2/39)

### LScapeLighting
- [x] `LScapeLighting::GetAmbientLight` · 005835c0 · legions 00552d30

### LScapeRender
- [ ] `LScapeRender::RefreshGraphicsResources` · 00583630 · legions 00552da0
- [ ] `LScapeRender::ReinitializeTerrain` · 00583650 · legions 00552dc0
- [ ] `LScapeRender::ResetWorldState` · 005836b0 · legions 00552e20
- [ ] `LScapeRender::IsCameraInWater` · 005836d0 · legions 00552e40
- [ ] `LScapeRender::DrawRain` · 00583700 · legions 00552e70
- [ ] `LScapeRender::DrawTerrain` · 00583850
- [ ] `LScapeRender::DrawWater` · 00583860
- [ ] `LScapeRender::SetDayLighting` · 00583870 · legions 00552fe0
- [ ] `LScapeRender::SetDayFog` · 00583880
- [x] `LScapeRender::SetDefaultLighting` · 00583890 · legions 00553000
- [ ] `LScapeRender::ReloadLandBlocks` · 00583920 · legions 00553090
- [ ] `LScapeRender::SetVertexValue` · 00583950 · legions 005530c0
- [ ] `LScapeRender::Refresh` · 005839b0 · legions 00553120
- [ ] `LScapeRender::DirtyLighting` · 005839e0 · legions 00553150
- [ ] `LScapeRender::UpdateLightingInfo` · 005839f0 · legions 00553160
- [ ] `LScapeRender::FlagRefreshDrawLists` · 00583a00 · legions 00553170
- [ ] `LScapeRender::CalcBlockDrawFrame` · 00583a40 · legions 005531b0

### RainSegment
- [ ] `RainSegment::operator=` · 00583aa0 · legions 00553210

### LScapeRender
- [ ] `LScapeRender::LScapeRender` · 00583b00 · legions 00553270

### CPhysicsObj
- [ ] `CPhysicsObj::get_acceleration` · 00583c10 · legions 00553380

### LScapeRender
- [ ] `LScapeRender::~LScapeRender` · 00583c20 · legions 00553390
- [ ] `LScapeRender::NotifyChangeRegion` · 00583d00 · legions 00553470
- [ ] `LScapeRender::ReleaseCells` · 00583d20 · legions 00553490
- [ ] `LScapeRender::AddCellSounds` · 00583d40 · legions 005534b0
- [ ] `LScapeRender::CullLandBlocks` · 00583e70 · legions 005535e0
- [ ] `LScapeRender::FindReflectionPlane` · 00583f40 · legions 00553680
- [ ] `LScapeRender::GatherVisibleOutsideObjects` · 00584040 · legions 00553780
- [ ] `LScapeRender::GetSunLightDirectionBits` · 00584090 · legions 005537d0
- [ ] `LScapeRender::UpdateLighting` · 005841c0 · legions 00553900
- [ ] `LScapeRender::CalculateShadowColor` · 00584290 · legions 005539d0
- [ ] `LScapeRender::GetTerrainGeometry` · 005843b0 · legions 00553af0
- [ ] `LScapeRender::RefreshShadowMap` · 00584540 · legions 00553c80
- [ ] `LScapeRender::Init` · 005847b0 · legions 00553ee0
- [ ] `LScapeRender::CalcBlockDrawOrderInternal` · 00584950 · legions 00554080
- [ ] `LScapeRender::CalcBlockDrawOrder` · 00584b10 · legions 00554240
- [ ] `LScapeRender::SetViewerPosition` · 00584c80 · legions 005543b0
- [ ] `LScapeRender::FillRainVertices` · 00584d70 · legions 00554490
- [ ] `LScapeRender::UpdateRain` · 005856f0 · legions 00554de0

## `portal/client/src/smartbox/worldrender/lscapesky.cpp.c` (3/9)

### LScapeSky
- [x] `LScapeSky::SetSkyFog` · 005857b0 · legions 00554ea0 — ac2-world region::sky_objects, ac2-render sky
- [x] `LScapeSky::CalcFrame` · 005857f0 · legions 00554ee0 — ac2-world region::sky_objects, ac2-render sky
- [ ] `LScapeSky::Clear` · 00585870 · legions 00554f60
- [x] `LScapeSky::SetViewerPosition` · 005858b0 · legions 00554fa0 — ac2-world region::sky_objects, ac2-render sky
- [~] `LScapeSky::Draw` · 00585980 · legions 00555070 — sky objects drawn as tinted object instances before/after the world with the fog_sky quad; lens flares and fixed_pos/height_mod not yet
- [ ] `LScapeSky::LScapeSky` · 00585b30 · legions 00555220
- [ ] `LScapeSky::~LScapeSky` · 00585b50 · legions 00555240
- [~] `LScapeSky::CreateDeletePhysicsObjects` · 00585ba0 · legions 00555290 — sky objects drawn as tinted object instances before/after the world with the fog_sky quad; lens flares and fixed_pos/height_mod not yet
- [~] `LScapeSky::UpdateTimeOfDay` · 00585cf0 · legions 005553e0 — sky objects drawn as tinted object instances before/after the world with the fog_sky quad; lens flares and fixed_pos/height_mod not yet

## `portal/client/src/smartbox/worldrender/lscapeterrainmesh.cpp.c` (5/48)

### LM_UVTranslate
- [ ] `LM_UVTranslate::SetVTranslate` · 005880a0 · legions 005577e0

### LScapeTerrainMesh
- [ ] `LScapeTerrainMesh::CreateLocalMemBumpMap` · 005880f0 · legions 00557830
- [ ] `LScapeTerrainMesh::SetSelfShadowBit` · 00588300 · legions 00557a30
- [x] `LScapeTerrainMesh::IlluminateVertices` · 00588320 · legions 00557a50

### LScapeVertex
- [ ] `LScapeVertex::operator=` · 00588370 · legions 0059f0d0

### LScapeTerrainMesh
- [ ] `LScapeTerrainMesh::CalculatePointsOfIntersection` · 00588410 · legions 00557aa0

### ShorePointType
- [ ] `ShorePointType::operator=` · 005884d0 · legions 00557b60

### LandDefs
- [ ] `LandDefs::blockcoord_to_blockid` · 00588530 · legions 00557bc0

### CVertexArray
- [ ] `CVertexArray::CVertexArray` · 00588590 · legions 00557c20
- [ ] `CVertexArray::~CVertexArray` · 005885e0 · legions 00557c70

### LScapeTerrainMesh
- [~] `LScapeTerrainMesh::DrawTerrain` · 00588610 — near mesh then far chunks 2.. (far.rs, far_terrain.rs); no per-chunk frustum cull, no N-patches
- [~] `LScapeTerrainMesh::DrawWaterMesh` · 00588840 · legions 00557ee0 — near water mesh with its material; far water chunks, bump map, reflection and shore passes not yet
- [ ] `LScapeTerrainMesh::ApplyList` · 00588a30 · legions 005580d0
- [ ] `LScapeTerrainMesh::InqNearBlockIndex` · 00588aa0 · legions 00558140

### (free functions)
- [ ] `G_CalculateNormal` · 00588ae0 · legions 00558180

### LScapeTerrainMesh
- [ ] `LScapeTerrainMesh::FindShorePointIndexByOrigin` · 00588b70 · legions 00558210
- [ ] `LScapeTerrainMesh::DrawShore` · 00588be0 · legions 00558280

### DBObjGrabber<RenderTexture>
- [ ] `DBObjGrabber<RenderTexture>::DBObjGrabber<RenderTexture>` · 00588e10 · legions 005584b0

### LScapeFarChunk
- [ ] `LScapeFarChunk::LScapeFarChunk` · 005890f0 · legions 00558840

### LScapeTerrainMesh
- [x] `LScapeTerrainMesh::BuildFarTerrainMesh` · 00589130 · legions 00558880 — indices with RollDice diagonals and LOD crack fill, vertices, far water quads (terrain/far.rs)
- [ ] `LScapeTerrainMesh::SetVertex` · 00589ec0 · legions 00559610
- [ ] `LScapeTerrainMesh::GetPixelGeometry` · 0058a000 · legions 00559750
- [ ] `LScapeTerrainMesh::CleanupShore` · 0058a1f0 · legions 00559940

### SmartArray<ShoreStripType>
- [ ] `SmartArray<ShoreStripType>::~SmartArray<ShoreStripType>` · 0058a270 · legions 005599f0

### SmartArray<short>
- [ ] `SmartArray<short>::operator=` · 0058a400 · legions 00559af0

### SmartArray<LScapeFarBlock>
- [ ] `SmartArray<LScapeFarBlock>::operator=` · 0058a460 · legions 00559b50

### LScapeTerrainMesh
- [ ] `LScapeTerrainMesh::GetShadowColor` · 0058a4d0 · legions 00559bc0
- [ ] `LScapeTerrainMesh::SetVertexValue` · 0058a870 · legions 00559f60

### SmartArray<LayerStage_*>
- [ ] `SmartArray<LayerStage_*>::InsertOrderedByIndex` · 0058aab0 · legions 00540d30

### LScapeFarChunk
- [ ] `LScapeFarChunk::operator=` · 0058ac60 · legions 0055a2e0

### MaterialLayer
- [ ] `MaterialLayer::InsertModifier` · 0058ad10 · legions 0055a390

### LScapeTerrainMesh
- [ ] `LScapeTerrainMesh::DestroyLScapeMaterialInfo` · 0058ad30 · legions 0055a3b0
- [~] `LScapeTerrainMesh::DrawWater` · 0058ad90 — near water mesh with its material; far water chunks, bump map, reflection and shore passes not yet
- [ ] `LScapeTerrainMesh::ReOrderNearVertices` · 0058b460 · legions 0055aad0
- [x] `LScapeTerrainMesh::BuildNearWaterMesh` · 0058b600 · legions 0055ac70 — ac2-render object::load_water_model
- [ ] `LScapeTerrainMesh::Destroy` · 0058b8a0 · legions 0055af10
- [~] `LScapeTerrainMesh::BuildNearTerrainMesh` · 0058b9e0 · legions 0055b050 — per-block near mesh; no cloud-shadow UVs or shared near vertex array
- [ ] `LScapeTerrainMesh::~LScapeTerrainMesh` · 0058bc10 · legions 0055b280
- [x] `LScapeTerrainMesh::InitFarMesh` · 0058bce0 · legions 0055b350 — chunks, LOD steps, block edges, far material (no cloud-shadow stage: LandscapeCloudShadows off)
- [~] `LScapeTerrainMesh::UpdateNearTerrain` · 0058c8c0 · legions 0055bf30 — per-block near mesh; no cloud-shadow UVs or shared near vertex array
- [x] `LScapeTerrainMesh::UpdateNearWater` · 0058d700 · legions 0055cd70 — ac2-render object::load_water_model
- [ ] `LScapeTerrainMesh::GenerateShore` · 0058dcb0 · legions 0055d320
- [ ] `LScapeTerrainMesh::Init` · 0058e940 · legions 0055dfb0
- [ ] `LScapeTerrainMesh::LScapeTerrainMesh` · 0058eb20 · legions 0055e190
- [ ] `LScapeTerrainMesh::RefreshWorldBuilderTerrain` · 0058ec60 · legions 0055e2d0
- [ ] `LScapeTerrainMesh::Update` · 0058ee40 · legions 0055e4b0
- [ ] `LScapeTerrainMesh::Refresh` · 0058f030 · legions 0055e6a0
- [ ] `LScapeTerrainMesh::Refresh` · 0058f070 · legions 0055e6e0

## `portal/client/src/smartbox/worldrender/portalspace.cpp.c` (0/14)

### CDataMesh
- [ ] `CDataMesh::GetByEnum` · 00585e60 · legions 00555550

### PortalSpace::Effect
- [ ] `PortalSpace::Effect::Effect` · 00585e90 · legions 00555580
- [ ] `PortalSpace::Effect::MakeSafePortalIndex` · 00585f40 · legions 00555630
- [ ] `PortalSpace::Effect::GeneratePortals` · 00585f90 · legions 00555680
- [ ] `PortalSpace::Effect::StartNow` · 00586220 · legions 00555910
- [ ] `PortalSpace::Effect::Destroy` · 00586650 · legions 00555d40
- [ ] `PortalSpace::Effect::Start` · 00586720
- [ ] `PortalSpace::Effect::UpdateGeometry` · 00586740 · legions 00555e30
- [ ] `PortalSpace::Effect::StopNow` · 00586eb0 · legions 005565a0
- [ ] `PortalSpace::Effect::Think` · 00586f50 · legions 00556640
- [ ] `PortalSpace::Effect::Render` · 00587360 · legions 00556a50
- [ ] `PortalSpace::Effect::~Effect` · 00587da0 · legions 00557490
- [ ] `PortalSpace::Effect::Initialize` · 00587db0 · legions 005574a0
- [ ] `PortalSpace::Effect::Stop` · 00588050

## `portal/client/src/smartbox/worldrender/worldrender.cpp.c` (0/36)

### GameTime
- [ ] `GameTime::GetTimeTilTomorrow` · 0057c720 · legions 0054bff0
- [ ] `GameTime::GetNeedLightingRefresh` · 0057c740 · legions 0054c010
- [ ] `GameTime::NotifyLightingRefreshed` · 0057c760 · legions 0054c030

### PhysicsGlobals
- [ ] `PhysicsGlobals::SetWind` · 0057c780 · legions 0054c050

### FogInfo
- [ ] `FogInfo::FogInfo` · 0057c7d0 · legions 0054c0a0

### LightInfo
- [ ] `LightInfo::LightInfo` · 0057c820 · legions 0054c0f0

### LScapeLighting
- [ ] `LScapeLighting::GetShadowColor` · 0057c8b0 · legions 0054c180

### WorldRender
- [ ] `WorldRender::Init` · 0057c8e0 · legions 0054c1b0
- [ ] `WorldRender::EnterPortalSpace` · 0057ca90
- [ ] `WorldRender::ExitPortalSpace` · 0057cab0
- [ ] `WorldRender::InPortalSpace` · 0057cad0 · legions 0054c3a0
- [ ] `WorldRender::ForceShadowRefresh` · 0057caf0 · legions 0054c3c0
- [ ] `WorldRender::UpdateAmbientSounds` · 0057cb00 · legions 0054c3d0
- [ ] `WorldRender::InitAmbientSounds` · 0057cb40
- [~] `WorldRender::SetupEnvironment` · 0057cb50 · legions 0054c420 — outdoor sun, ambient and fog; indoor and sky environments not yet
- [ ] `WorldRender::UpdateRainSounds` · 0057cf80 · legions 0054c850

### SceneCamera
- [ ] `SceneCamera::SceneCamera` · 0057d190 · legions 0054ca60

### WorldRender
- [ ] `WorldRender::DrawBrushes` · 0057d290 · legions 0054cb60
- [ ] `WorldRender::CreateDiscShadows` · 0057d3d0 · legions 0054cca0
- [ ] `WorldRender::StartSelection` · 0057d680 · legions 0054cf50

### DBObjGrabber<CDayDesc>
- [ ] `DBObjGrabber<CDayDesc>::DBObjGrabber<CDayDesc>` · 0057d6d0 · legions 0054cfa0

### WorldRender
- [ ] `WorldRender::WorldRender` · 0057d870 · legions 0054d140
- [ ] `WorldRender::SetWantNoticeOnNextFrameDrawn` · 0057da00 · legions 0054d2d0
- [ ] `WorldRender::GetWantNoticeOnNextFrameDrawn` · 0057da10 · legions 0054d2e0
- [ ] `WorldRender::~WorldRender` · 0057da20 · legions 0054d2f0
- [ ] `WorldRender::ResetWorldState` · 0057db20 · legions 0054d3f0
- [ ] `WorldRender::DrawWorldObjects` · 0057dba0 · legions 0054d470
- [ ] `WorldRender::UpdateTimeOfDay` · 0057dcc0 · legions 0054d590
- [~] `WorldRender::Update` · 0057e1b0 · legions 0054da80 — ac2-client world_manager_update: WorldManager::Update, UpdateRenderInterpolation, UpdateAnimation; no time of day, rain, ambient sound or scene tools
- [ ] `WorldRender::RenderReflectionMap` · 0057e270 · legions 0054db40
- [ ] `WorldRender::UpdateRenderTargets` · 0057e750 · legions 0054e020
- [ ] `WorldRender::GatherIndoorObjects` · 0057e830 · legions 0054e100
- [ ] `WorldRender::GatherOutdoorObjects` · 0057ea20 · legions 0054e2f0
- [ ] `WorldRender::CreateLandscapePointLightDecals` · 0057eb10 · legions 0054e3e0
- [ ] `WorldRender::RenderSkeletonEffects` · 0057ef60 · legions 0054e830
- [ ] `WorldRender::Render` · 0057f960 · legions 0054f230
