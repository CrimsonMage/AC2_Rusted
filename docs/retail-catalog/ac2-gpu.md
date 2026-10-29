# ac2-gpu: The render device

95 retail functions: 10 done, 3 partial, 6 to do, 76 skipped.
Legend and workflow: [README](README.md).

## `portal/client/src/render/core/d3d/renderd3d.cpp.c` (0/11)

### RenderD3D
- [-] `RenderD3D::IncludeFileInWatsonUpload` · 005d97b0 — Win32 crash-report or GDI bitmap/icon conversion; Windows plumbing
- [-] `RenderD3D::GetFragmentCompiler` · 005d98f0 · legions 0059af90 — texture-stage binding (wgpu bind groups), default render-state baseline (each pipeline sets its own), or fragment.dll compiler loader (object.wgsl is the port)
- [-] `RenderD3D::SelectBestAdapter` · 005d9910 · legions 0059afb0 — D3D8 device lifecycle, adapter, presentation or caps plumbing; wgpu surface, passes and HdrTarget replace it
- [-] `RenderD3D::Begin` · 005d9a30 · legions 0059b0d0 — D3D8 device lifecycle, adapter, presentation or caps plumbing; wgpu surface, passes and HdrTarget replace it
- [-] `RenderD3D::End` · 005d9a60 · legions 0059b100 — D3D8 device lifecycle, adapter, presentation or caps plumbing; wgpu surface, passes and HdrTarget replace it
- [-] `RenderD3D::Shutdown` · 005d9b00 · legions 0059b1a0 — D3D8 device lifecycle, adapter, presentation or caps plumbing; wgpu surface, passes and HdrTarget replace it
- [-] `RenderD3D::ResetVertexStreams` · 005d9b40 · legions 0059b1e0 — D3D8 resource and shader-object lifetime or binding; wgpu buffers, textures and pipelines replace it
- [-] `RenderD3D::CreateRenderDevice` · 005d9cd0 · legions 0059b2b0 — D3D8 device lifecycle, adapter, presentation or caps plumbing; wgpu surface, passes and HdrTarget replace it
- [-] `RenderD3D::ReferenceVertexStream` · 005d9d00 · legions 0059b2e0 — D3D8 resource and shader-object lifetime or binding; wgpu buffers, textures and pipelines replace it
- [-] `RenderD3D::DetectDisplayModes` · 005d9dc0 · legions 0059b3a0 — D3D8 device lifecycle, adapter, presentation or caps plumbing; wgpu surface, passes and HdrTarget replace it
- [-] `RenderD3D::Startup` · 005d9e70 · legions 0059b450 — D3D8 device lifecycle, adapter, presentation or caps plumbing; wgpu surface, passes and HdrTarget replace it

## `portal/client/src/render/core/d3d/renderdeviced3d.cpp.c` (10/84)

### RenderIndexBuffer
- [-] `RenderIndexBuffer::~RenderIndexBuffer` · 005c49d0 · legions 0058ba80 — D3D8 resource and shader-object lifetime or binding; wgpu buffers, textures and pipelines replace it

### (free functions)
- [-] `CreateScreenCompatibleBitmapFromRGBA` · 005c4a20 · legions 0058bad0 — Win32 crash-report or GDI bitmap/icon conversion; Windows plumbing
- [-] `CreateMonoBitmapFromRGBA` · 005c4c00 · legions 0058bcb0 — Win32 crash-report or GDI bitmap/icon conversion; Windows plumbing
- [-] `CreateIconFromRGBA` · 005c4d60 · legions 0058be10 — Win32 crash-report or GDI bitmap/icon conversion; Windows plumbing

### RenderDeviceD3D
- [-] `RenderDeviceD3D::SetStageTexture` · 005c4e00 — texture-stage binding (wgpu bind groups), default render-state baseline (each pipeline sets its own), or fragment.dll compiler loader (object.wgsl is the port)
- [x] `RenderDeviceD3D::SetStageColorSource` · 005c4e90 · legions 0058beb0 — ac2-render object::pass / texture_matrix, ac2-gpu object
- [x] `RenderDeviceD3D::SetStageAlphaSource` · 005c4f20 · legions 0058bf40 — ac2-render object::pass / texture_matrix, ac2-gpu object
- [~] `RenderDeviceD3D::SetStageFilterMode` · 005c4fb0 · legions 0058bfd0 — point/linear per stage; anisotropic preference not modelled, mip filter linear
- [ ] `RenderDeviceD3D::SetMultiSampleAntialias` · 005c5080 · legions 0058c0a0 — needs port (check): edge antialias state toggled for UI surfaces (UISurfaceObject::Render); crates draw UI with no edge AA; confirm the visible effect before porting
- [-] `RenderDeviceD3D::Begin` · 005c50d0 · legions 0058c0f0 — D3D8 device lifecycle, adapter, presentation or caps plumbing; wgpu surface, passes and HdrTarget replace it
- [-] `RenderDeviceD3D::End` · 005c5170 · legions 0058c190 — D3D8 device lifecycle, adapter, presentation or caps plumbing; wgpu surface, passes and HdrTarget replace it
- [-] `RenderDeviceD3D::BeginScene` · 005c51a0 · legions 0058c1c0 — D3D8 device lifecycle, adapter, presentation or caps plumbing; wgpu surface, passes and HdrTarget replace it
- [-] `RenderDeviceD3D::EndScene` · 005c51d0 — D3D8 device lifecycle, adapter, presentation or caps plumbing; wgpu surface, passes and HdrTarget replace it
- [x] `RenderDeviceD3D::SetCursorProperties` · 005c5250 — ac2-client bin/ac2/cursor.rs GameCursor: the HUD's pointer cursor (UiTree::pointer_cursor, its image loaded with the UI images) as a winit custom cursor with its hot spot, made once per image and hot spot; the pregame screens keep the system cursor
- [ ] `RenderDeviceD3D::ShowCursor` · 005c5450 — needs port: game cursor show/hide from renderui.cpp.c; winit set_cursor_visible only follows mouse look (ac2-client bin/ac2/main.rs)
- [-] `RenderDeviceD3D::SetWorldTransform` · 005c5470 · legions 0058c480 — D3D8 device lifecycle, adapter, presentation or caps plumbing; wgpu surface, passes and HdrTarget replace it
- [-] `RenderDeviceD3D::SetViewTransform` · 005c54a0 · legions 0058c4b0 — D3D8 device lifecycle, adapter, presentation or caps plumbing; wgpu surface, passes and HdrTarget replace it
- [-] `RenderDeviceD3D::SetProjectionTransform` · 005c54d0 · legions 0058c4e0 — D3D8 device lifecycle, adapter, presentation or caps plumbing; wgpu surface, passes and HdrTarget replace it
- [-] `RenderDeviceD3D::SetRenderTarget` · 005c5500 · legions 0058c510 — D3D8 device lifecycle, adapter, presentation or caps plumbing; wgpu surface, passes and HdrTarget replace it
- [-] `RenderDeviceD3D::CreateDepthStencilSurface` · 005c55a0 · legions 0058c5b0 — D3D8 device lifecycle, adapter, presentation or caps plumbing; wgpu surface, passes and HdrTarget replace it
- [-] `RenderDeviceD3D::SetViewport` · 005c5650 — D3D8 device lifecycle, adapter, presentation or caps plumbing; wgpu surface, passes and HdrTarget replace it
- [-] `RenderDeviceD3D::SelectPixelFormat` · 005c56d0 · legions 0058c6e0 — D3D8 device lifecycle, adapter, presentation or caps plumbing; wgpu surface, passes and HdrTarget replace it
- [-] `RenderDeviceD3D::ChoosePixelFormats` · 005c57f0 · legions 0058c800 — D3D8 device lifecycle, adapter, presentation or caps plumbing; wgpu surface, passes and HdrTarget replace it
- [-] `RenderDeviceD3D::CheckDisplayModes` · 005c5a50 · legions 0058ca60 — D3D8 device lifecycle, adapter, presentation or caps plumbing; wgpu surface, passes and HdrTarget replace it
- [-] `RenderDeviceD3D::Shutdown` · 005c5b10 · legions 0058cb20 — D3D8 device lifecycle, adapter, presentation or caps plumbing; wgpu surface, passes and HdrTarget replace it
- [-] `RenderDeviceD3D::DetectDeviceCaps` · 005c5b50 · legions 0058cb60 — D3D8 device lifecycle, adapter, presentation or caps plumbing; wgpu surface, passes and HdrTarget replace it
- [-] `RenderDeviceD3D::SetDefaultD3DStates` · 005c5d40 · legions 0058cd70 — texture-stage binding (wgpu bind groups), default render-state baseline (each pipeline sets its own), or fragment.dll compiler loader (object.wgsl is the port)
- [x] `RenderDeviceD3D::SetGamma` · 005c60a0 · legions 0058d0d0 — ac2-gpu ToneMap::set_brightness (the ramp's 1 + 2b scale on the world's display levels)
- [-] `RenderDeviceD3D::CreateSurface` · 005c6190 — D3D8 resource and shader-object lifetime or binding; wgpu buffers, textures and pipelines replace it
- [-] `RenderDeviceD3D::CreateTexture` · 005c61b0 — D3D8 resource and shader-object lifetime or binding; wgpu buffers, textures and pipelines replace it
- [-] `RenderDeviceD3D::CreateIndexBuffer` · 005c61d0 — D3D8 resource and shader-object lifetime or binding; wgpu buffers, textures and pipelines replace it
- [-] `RenderDeviceD3D::Clear` · 005c6210 — D3D8 device lifecycle, adapter, presentation or caps plumbing; wgpu surface, passes and HdrTarget replace it
- [-] `RenderDeviceD3D::Flip` · 005c6350 — D3D8 device lifecycle, adapter, presentation or caps plumbing; wgpu surface, passes and HdrTarget replace it
- [-] `RenderDeviceD3D::IsResetPossible` · 005c63e0 · legions 0058d3e0 — D3D8 device lifecycle, adapter, presentation or caps plumbing; wgpu surface, passes and HdrTarget replace it
- [x] `RenderDeviceD3D::SetAmbient` · 005c6410 — ac2-render object::pass / texture_matrix, ac2-gpu object
- [-] `RenderDeviceD3D::CreateVertexShaderFromSource` · 005c6440 · legions 0058d440 — D3D8 resource and shader-object lifetime or binding; wgpu buffers, textures and pipelines replace it
- [-] `RenderDeviceD3D::CreateVertexShader` · 005c64e0 — D3D8 resource and shader-object lifetime or binding; wgpu buffers, textures and pipelines replace it
- [-] `RenderDeviceD3D::DeleteVertexShader` · 005c6700 — D3D8 resource and shader-object lifetime or binding; wgpu buffers, textures and pipelines replace it
- [-] `RenderDeviceD3D::CreatePixelShader` · 005c6720 — D3D8 resource and shader-object lifetime or binding; wgpu buffers, textures and pipelines replace it
- [-] `RenderDeviceD3D::DeletePixelShader` · 005c67f0 — D3D8 resource and shader-object lifetime or binding; wgpu buffers, textures and pipelines replace it
- [ ] `RenderDeviceD3D::SetupStaticVertexShaderConstants` · 005c6810 · legions 0058d810 — needs port (check): static vertex-shader constants c0..c2 = (1, 0, 0.5, 0.75) loaded for every vertex program; confirm object.wgsl carries the same values
- [ ] `RenderDeviceD3D::SetFogProperties` · 005c68e0 — needs port: fog colour and near/far range per scene (SceneTool::SetFogProperties callers: worldrender, portalspace, lscapesky, uiscene). Fog exists in object.wgsl and terrain.wgsl, but nothing cites this function; verify the FOGMODE_NONE defaults (colour 0x7f7f7f, range 9000-10000) and cite before marking done
- [-] `RenderDeviceD3D::SetAlphaFogProperties` · 005c6a10 — D3D8 render-state setter with no caller in the admin source tree; wgpu pipelines carry the state
- [-] `RenderDeviceD3D::SetStencilMode` · 005c6a30 — D3D8 render-state setter with no caller in the admin source tree; wgpu pipelines carry the state
- [-] `RenderDeviceD3D::SetZBias` · 005c6a60 — D3D8 render-state setter with no caller in the admin source tree; wgpu pipelines carry the state
- [-] `RenderDeviceD3D::SetPatchSegments` · 005c6a90 — D3D8 render-state setter with no caller in the admin source tree; wgpu pipelines carry the state
- [-] `RenderDeviceD3D::SetStencilFailOp` · 005c6ae0 · legions 0058da30 — D3D8 render-state setter with no caller in the admin source tree; wgpu pipelines carry the state
- [-] `RenderDeviceD3D::SetStencilZFailOp` · 005c6b10 · legions 0058da60 — D3D8 render-state setter with no caller in the admin source tree; wgpu pipelines carry the state
- [-] `RenderDeviceD3D::SetStencilPassOp` · 005c6b40 · legions 0058dae0 — D3D8 render-state setter with no caller in the admin source tree; wgpu pipelines carry the state
- [-] `RenderDeviceD3D::SetStencilTest` · 005c6b70 · legions 0058db10 — D3D8 render-state setter with no caller in the admin source tree; wgpu pipelines carry the state
- [-] `RenderDeviceD3D::SetStencilRef` · 005c6ba0 · legions 0058db40 — D3D8 render-state setter with no caller in the admin source tree; wgpu pipelines carry the state
- [-] `RenderDeviceD3D::SetStencilTestMask` · 005c6bd0 · legions 0058db70 — D3D8 render-state setter with no caller in the admin source tree; wgpu pipelines carry the state
- [-] `RenderDeviceD3D::SetStencilWriteMask` · 005c6c00 · legions 0058dba0 — D3D8 render-state setter with no caller in the admin source tree; wgpu pipelines carry the state
- [-] `RenderDeviceD3D::SetColorWriteMode` · 005c6c30 · legions 0058dc30 — D3D8 render-state setter with no caller in the admin source tree; wgpu pipelines carry the state
- [-] `RenderDeviceD3D::SetFillMode` · 005c6c70 · legions 0058dc70 — D3D8 render-state setter with no caller in the admin source tree; wgpu pipelines carry the state
- [-] `RenderDeviceD3D::SetVertexShader` · 005c6ca0 · legions 0058dca0 — D3D8 resource and shader-object lifetime or binding; wgpu buffers, textures and pipelines replace it
- [-] `RenderDeviceD3D::SetVertexShaderConstant` · 005c6cd0 — D3D8 resource and shader-object lifetime or binding; wgpu buffers, textures and pipelines replace it
- [-] `RenderDeviceD3D::SetPixelShader` · 005c6d40 · legions 0058dd40 — D3D8 resource and shader-object lifetime or binding; wgpu buffers, textures and pipelines replace it
- [-] `RenderDeviceD3D::SetPixelShaderConstant` · 005c6d70 — D3D8 resource and shader-object lifetime or binding; wgpu buffers, textures and pipelines replace it
- [-] `RenderDeviceD3D::SetVertexStream` · 005c6de0 · legions 0058dde0 — D3D8 resource and shader-object lifetime or binding; wgpu buffers, textures and pipelines replace it
- [ ] `RenderDeviceD3D::GenerateSurfaceFromFrontBuffer` · 005c6e30 · legions 0058de30 — needs port (verify): front-buffer capture behind the screenshot key; ac2-client bin/ac2/screenshot.rs does the capture but nothing cites this function; confirm it matches Device::SaveScreenshot before marking

### RenderLight
- [-] `RenderLight::operator=` · 005c6ea0 · legions 0058df10 — light-list storage, empty admin stub, or surface copy; lights.rs selects lights and the shadow map renders in place (shadow_map.rs)

### RenderVertexBuffer
- [-] `RenderVertexBuffer::RenderIndexedPrimitives` · 005c6f60 · legions 0058dfd0 — light-list storage, empty admin stub, or surface copy; lights.rs selects lights and the shadow map renders in place (shadow_map.rs)

### RenderVertexBufferD3D
- [-] `RenderVertexBufferD3D::RenderVertexBufferD3D` · 005c6fc0 · legions 0058e030 — D3D8 resource and shader-object lifetime or binding; wgpu buffers, textures and pipelines replace it
- [-] `RenderVertexBufferD3D::~RenderVertexBufferD3D` · 005c7050 · legions 0058e0c0 — D3D8 resource and shader-object lifetime or binding; wgpu buffers, textures and pipelines replace it

### RenderDeviceD3D
- [x] `RenderDeviceD3D::SetSourceBlend` · 005c70b0 · legions 0058e120 — ac2-render object::pass / texture_matrix, ac2-gpu object
- [x] `RenderDeviceD3D::SetDestBlend` · 005c7120 · legions 0058e190 — ac2-render object::pass / texture_matrix, ac2-gpu object
- [x] `RenderDeviceD3D::SetDepthWrite` · 005c7190 · legions 0058e200 — ac2-render object::pass / texture_matrix, ac2-gpu object
- [x] `RenderDeviceD3D::SetDepthTest` · 005c7200 · legions 0058e270 — ac2-render object::pass / texture_matrix, ac2-gpu object
- [x] `RenderDeviceD3D::SetDepthFog` · 005c7270 · legions 0058e2e0 — ac2-render object::pass / texture_matrix, ac2-gpu object
- [-] `RenderDeviceD3D::CreateVertexBuffer` · 005c72e0 — D3D8 resource and shader-object lifetime or binding; wgpu buffers, textures and pipelines replace it
- [-] `RenderDeviceD3D::ClearLights` · 005c7300 — light-list storage, empty admin stub, or surface copy; lights.rs selects lights and the shadow map renders in place (shadow_map.rs)
- [ ] `RenderDeviceD3D::SetupVertexStates` · 005c7320 · legions 0058e390 — needs port (check): sets the D3D material (diffuse, ambient, specular from the vertex colour) and lighting state; confirm the object.wgsl material and vertex-colour path before marking
- [~] `RenderDeviceD3D::SetupVertexShaderConstants` · 005c7740 · legions 0058e7b0 — sun, up to four point lights, c_AmbientColor and pixel constants c0-c5
- [~] `RenderDeviceD3D::SetupState` · 005c88f0 — object::pass::layer_pass; special textures (frame buffer, distortion map) unbound, TEXTUREFACTOR taken as material diffuse while lit
- [-] `RenderDeviceD3D::SetDefaultDeviceStates` · 005c91c0 · legions 00590190 — texture-stage binding (wgpu bind groups), default render-state baseline (each pipeline sets its own), or fragment.dll compiler loader (object.wgsl is the port)
- [-] `RenderDeviceD3D::DetectDisplayFormats` · 005c9820 · legions 005907f0 — D3D8 device lifecycle, adapter, presentation or caps plumbing; wgpu surface, passes and HdrTarget replace it
- [-] `RenderDeviceD3D::SetupDisplayInfo` · 005c9980 · legions 00590950 — D3D8 device lifecycle, adapter, presentation or caps plumbing; wgpu surface, passes and HdrTarget replace it
- [-] `RenderDeviceD3D::SelectBufferFormats` · 005c9b00 · legions 00590ad0 — D3D8 device lifecycle, adapter, presentation or caps plumbing; wgpu surface, passes and HdrTarget replace it
- [-] `RenderDeviceD3D::CopyRects` · 005c9f30 · legions 00590f00 — light-list storage, empty admin stub, or surface copy; lights.rs selects lights and the shadow map renders in place (shadow_map.rs)
- [-] `RenderDeviceD3D::AddLight` · 005ca000 · legions 00590fd0 — light-list storage, empty admin stub, or surface copy; lights.rs selects lights and the shadow map renders in place (shadow_map.rs)
- [-] `RenderDeviceD3D::SetupPresentation` · 005ca060 · legions 00591030 — D3D8 device lifecycle, adapter, presentation or caps plumbing; wgpu surface, passes and HdrTarget replace it
- [-] `RenderDeviceD3D::Startup` · 005ca280 · legions 00591250 — D3D8 device lifecycle, adapter, presentation or caps plumbing; wgpu surface, passes and HdrTarget replace it
- [-] `RenderDeviceD3D::ResetDevice` · 005ca6f0 — D3D8 device lifecycle, adapter, presentation or caps plumbing; wgpu surface, passes and HdrTarget replace it
