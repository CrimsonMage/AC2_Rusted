# ac2-gpu: The render device

95 retail functions: 0 done, 0 partial, 95 to do, 0 skipped.
Legend and workflow: [README](README.md).

## `portal/client/src/render/core/d3d/renderd3d.cpp.c` (0/11)

### RenderD3D
- [ ] `RenderD3D::IncludeFileInWatsonUpload` · 005d97b0
- [ ] `RenderD3D::GetFragmentCompiler` · 005d98f0 · legions 0059af90
- [ ] `RenderD3D::SelectBestAdapter` · 005d9910 · legions 0059afb0
- [ ] `RenderD3D::Begin` · 005d9a30 · legions 0059b0d0
- [ ] `RenderD3D::End` · 005d9a60 · legions 0059b100
- [ ] `RenderD3D::Shutdown` · 005d9b00 · legions 0059b1a0
- [ ] `RenderD3D::ResetVertexStreams` · 005d9b40 · legions 0059b1e0
- [ ] `RenderD3D::CreateRenderDevice` · 005d9cd0 · legions 0059b2b0
- [ ] `RenderD3D::ReferenceVertexStream` · 005d9d00 · legions 0059b2e0
- [ ] `RenderD3D::DetectDisplayModes` · 005d9dc0 · legions 0059b3a0
- [ ] `RenderD3D::Startup` · 005d9e70 · legions 0059b450

## `portal/client/src/render/core/d3d/renderdeviced3d.cpp.c` (0/84)

### RenderIndexBuffer
- [ ] `RenderIndexBuffer::~RenderIndexBuffer` · 005c49d0 · legions 0058ba80

### (free functions)
- [ ] `CreateScreenCompatibleBitmapFromRGBA` · 005c4a20 · legions 0058bad0
- [ ] `CreateMonoBitmapFromRGBA` · 005c4c00 · legions 0058bcb0
- [ ] `CreateIconFromRGBA` · 005c4d60 · legions 0058be10

### RenderDeviceD3D
- [ ] `RenderDeviceD3D::SetStageTexture` · 005c4e00
- [ ] `RenderDeviceD3D::SetStageColorSource` · 005c4e90 · legions 0058beb0
- [ ] `RenderDeviceD3D::SetStageAlphaSource` · 005c4f20 · legions 0058bf40
- [ ] `RenderDeviceD3D::SetStageFilterMode` · 005c4fb0 · legions 0058bfd0
- [ ] `RenderDeviceD3D::SetMultiSampleAntialias` · 005c5080 · legions 0058c0a0
- [ ] `RenderDeviceD3D::Begin` · 005c50d0 · legions 0058c0f0
- [ ] `RenderDeviceD3D::End` · 005c5170 · legions 0058c190
- [ ] `RenderDeviceD3D::BeginScene` · 005c51a0 · legions 0058c1c0
- [ ] `RenderDeviceD3D::EndScene` · 005c51d0
- [ ] `RenderDeviceD3D::SetCursorProperties` · 005c5250
- [ ] `RenderDeviceD3D::ShowCursor` · 005c5450
- [ ] `RenderDeviceD3D::SetWorldTransform` · 005c5470 · legions 0058c480
- [ ] `RenderDeviceD3D::SetViewTransform` · 005c54a0 · legions 0058c4b0
- [ ] `RenderDeviceD3D::SetProjectionTransform` · 005c54d0 · legions 0058c4e0
- [ ] `RenderDeviceD3D::SetRenderTarget` · 005c5500 · legions 0058c510
- [ ] `RenderDeviceD3D::CreateDepthStencilSurface` · 005c55a0 · legions 0058c5b0
- [ ] `RenderDeviceD3D::SetViewport` · 005c5650
- [ ] `RenderDeviceD3D::SelectPixelFormat` · 005c56d0 · legions 0058c6e0
- [ ] `RenderDeviceD3D::ChoosePixelFormats` · 005c57f0 · legions 0058c800
- [ ] `RenderDeviceD3D::CheckDisplayModes` · 005c5a50 · legions 0058ca60
- [ ] `RenderDeviceD3D::Shutdown` · 005c5b10 · legions 0058cb20
- [ ] `RenderDeviceD3D::DetectDeviceCaps` · 005c5b50 · legions 0058cb60
- [ ] `RenderDeviceD3D::SetDefaultD3DStates` · 005c5d40 · legions 0058cd70
- [ ] `RenderDeviceD3D::SetGamma` · 005c60a0 · legions 0058d0d0
- [ ] `RenderDeviceD3D::CreateSurface` · 005c6190
- [ ] `RenderDeviceD3D::CreateTexture` · 005c61b0
- [ ] `RenderDeviceD3D::CreateIndexBuffer` · 005c61d0
- [ ] `RenderDeviceD3D::Clear` · 005c6210
- [ ] `RenderDeviceD3D::Flip` · 005c6350
- [ ] `RenderDeviceD3D::IsResetPossible` · 005c63e0 · legions 0058d3e0
- [ ] `RenderDeviceD3D::SetAmbient` · 005c6410
- [ ] `RenderDeviceD3D::CreateVertexShaderFromSource` · 005c6440 · legions 0058d440
- [ ] `RenderDeviceD3D::CreateVertexShader` · 005c64e0
- [ ] `RenderDeviceD3D::DeleteVertexShader` · 005c6700
- [ ] `RenderDeviceD3D::CreatePixelShader` · 005c6720
- [ ] `RenderDeviceD3D::DeletePixelShader` · 005c67f0
- [ ] `RenderDeviceD3D::SetupStaticVertexShaderConstants` · 005c6810 · legions 0058d810
- [ ] `RenderDeviceD3D::SetFogProperties` · 005c68e0
- [ ] `RenderDeviceD3D::SetAlphaFogProperties` · 005c6a10
- [ ] `RenderDeviceD3D::SetStencilMode` · 005c6a30
- [ ] `RenderDeviceD3D::SetZBias` · 005c6a60
- [ ] `RenderDeviceD3D::SetPatchSegments` · 005c6a90
- [ ] `RenderDeviceD3D::SetStencilFailOp` · 005c6ae0 · legions 0058da30
- [ ] `RenderDeviceD3D::SetStencilZFailOp` · 005c6b10 · legions 0058da60
- [ ] `RenderDeviceD3D::SetStencilPassOp` · 005c6b40 · legions 0058dae0
- [ ] `RenderDeviceD3D::SetStencilTest` · 005c6b70 · legions 0058db10
- [ ] `RenderDeviceD3D::SetStencilRef` · 005c6ba0 · legions 0058db40
- [ ] `RenderDeviceD3D::SetStencilTestMask` · 005c6bd0 · legions 0058db70
- [ ] `RenderDeviceD3D::SetStencilWriteMask` · 005c6c00 · legions 0058dba0
- [ ] `RenderDeviceD3D::SetColorWriteMode` · 005c6c30 · legions 0058dc30
- [ ] `RenderDeviceD3D::SetFillMode` · 005c6c70 · legions 0058dc70
- [ ] `RenderDeviceD3D::SetVertexShader` · 005c6ca0 · legions 0058dca0
- [ ] `RenderDeviceD3D::SetVertexShaderConstant` · 005c6cd0
- [ ] `RenderDeviceD3D::SetPixelShader` · 005c6d40 · legions 0058dd40
- [ ] `RenderDeviceD3D::SetPixelShaderConstant` · 005c6d70
- [ ] `RenderDeviceD3D::SetVertexStream` · 005c6de0 · legions 0058dde0
- [ ] `RenderDeviceD3D::GenerateSurfaceFromFrontBuffer` · 005c6e30 · legions 0058de30

### RenderLight
- [ ] `RenderLight::operator=` · 005c6ea0 · legions 0058df10

### RenderVertexBuffer
- [ ] `RenderVertexBuffer::RenderIndexedPrimitives` · 005c6f60 · legions 0058dfd0

### RenderVertexBufferD3D
- [ ] `RenderVertexBufferD3D::RenderVertexBufferD3D` · 005c6fc0 · legions 0058e030
- [ ] `RenderVertexBufferD3D::~RenderVertexBufferD3D` · 005c7050 · legions 0058e0c0

### RenderDeviceD3D
- [ ] `RenderDeviceD3D::SetSourceBlend` · 005c70b0 · legions 0058e120
- [ ] `RenderDeviceD3D::SetDestBlend` · 005c7120 · legions 0058e190
- [ ] `RenderDeviceD3D::SetDepthWrite` · 005c7190 · legions 0058e200
- [ ] `RenderDeviceD3D::SetDepthTest` · 005c7200 · legions 0058e270
- [ ] `RenderDeviceD3D::SetDepthFog` · 005c7270 · legions 0058e2e0
- [ ] `RenderDeviceD3D::CreateVertexBuffer` · 005c72e0
- [ ] `RenderDeviceD3D::ClearLights` · 005c7300
- [ ] `RenderDeviceD3D::SetupVertexStates` · 005c7320 · legions 0058e390
- [ ] `RenderDeviceD3D::SetupVertexShaderConstants` · 005c7740 · legions 0058e7b0
- [ ] `RenderDeviceD3D::SetupState` · 005c88f0
- [ ] `RenderDeviceD3D::SetDefaultDeviceStates` · 005c91c0 · legions 00590190
- [ ] `RenderDeviceD3D::DetectDisplayFormats` · 005c9820 · legions 005907f0
- [ ] `RenderDeviceD3D::SetupDisplayInfo` · 005c9980 · legions 00590950
- [ ] `RenderDeviceD3D::SelectBufferFormats` · 005c9b00 · legions 00590ad0
- [ ] `RenderDeviceD3D::CopyRects` · 005c9f30 · legions 00590f00
- [ ] `RenderDeviceD3D::AddLight` · 005ca000 · legions 00590fd0
- [ ] `RenderDeviceD3D::SetupPresentation` · 005ca060 · legions 00591030
- [ ] `RenderDeviceD3D::Startup` · 005ca280 · legions 00591250
- [ ] `RenderDeviceD3D::ResetDevice` · 005ca6f0
