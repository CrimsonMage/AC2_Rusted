# replaced: Third-party libraries, CRT and container templates replaced by Rust

2058 retail functions: 23 done, 0 partial, 0 to do, 2035 skipped.
Legend and workflow: [README](README.md).

## `_libraries/ImeUI.lib/ImeUi.obj.c` (0/36)

### (free functions)
- [-] `FUN_0071ba80` · 0071ba80 · legions 0069c3a0 — third-party library
- [-] `FUN_0071bb10` · 0071bb10 · legions 0069c430 — third-party library
- [-] `FUN_0071bb50` · 0071bb50 · legions 0069c470 — third-party library
- [-] `FUN_0071bec0` · 0071bec0 · legions 0069c7e0 — third-party library
- [-] `FUN_0071bf60` · 0071bf60 · legions 0069c880 — third-party library
- [-] `FUN_0071bfb0` · 0071bfb0 · legions 0069c8d0 — third-party library
- [-] `FUN_0071c010` · 0071c010 · legions 0069c930 — third-party library
- [-] `FUN_0071c030` · 0071c030 · legions 0069c950 — third-party library
- [-] `FUN_0071c0e0` · 0071c0e0 · legions 0069ca00 — third-party library
- [-] `FUN_0071c110` · 0071c110 · legions 0069ca30 — third-party library
- [-] `ImeUi_SetCaretPosition` · 0071c140 · legions 0069ca60 — third-party library
- [-] `ImeUi_SetCompStringAppearance` · 0071c160 · legions 0069ca80 — third-party library
- [-] `FUN_0071c1d0` · 0071c1d0 · legions 0069caf0 — third-party library
- [-] `ImeUi_IgnoreHotKey` · 0071c420 · legions 0069cd40 — third-party library
- [-] `ImeUi_FinalizeString` · 0071c4b0 · legions 0069cdd0 — third-party library
- [-] `FUN_0071c550` · 0071c550 · legions 0069ce70 — third-party library
- [-] `FUN_0071c5c0` · 0071c5c0 · legions 0069cee0 — third-party library
- [-] `ImeUi_SetSupportLevel` · 0071c600 · legions 0069cf20 — third-party library
- [-] `ImeUi_SetAppearance` · 0071c620 · legions 0069cf40 — third-party library
- [-] `ImeUi_SetScreenDimension` · 0071c680 · legions 0069cfa0 — third-party library
- [-] `FUN_0071c6a0` · 0071c6a0 · legions 0069cfc0 — third-party library
- [-] `FUN_0071c720` · 0071c720 · legions 0069d040 — third-party library
- [-] `FUN_0071cbc0` · 0071cbc0 · legions 0069d4e0 — third-party library
- [-] `ImeUi_ToggleLanguageBar` · 0071cce0 · legions 0069d600 — third-party library
- [-] `ImeUi_IsSendingKeyMessage` · 0071ce30 · legions 0069d750 — third-party library
- [-] `FUN_0071ce40` · 0071ce40 · legions 0069d760 — third-party library
- [-] `FUN_0071ceb0` · 0071ceb0 · legions 0069d7d0 — third-party library
- [-] `ImeUi_SetWindow` · 0071cfc0 · legions 0069d8e0 — third-party library
- [-] `ImeUi_RenderUI` · 0071cfd0 · legions 0069d8f0 — third-party library
- [-] `FUN_0071da60` · 0071da60 · legions 0069e380 — third-party library
- [-] `FUN_0071deb0` · 0071deb0 · legions 0069e7d0 — third-party library
- [-] `ImeUi_ProcessMessage` · 0071df90 · legions 0069e8b0 — third-party library
- [-] `ImeUi_GetState` · 0071e840 · legions 0069f160 — third-party library
- [-] `ImeUi_EnableIme` · 0071e860 · legions 0069f180 — third-party library
- [-] `ImeUi_Initialize` · 0071e8c0 · legions 0069f1e0 — third-party library
- [-] `ImeUi_SetState` · 0071eac0 · legions 0069f3e0 — third-party library

## `_libraries/MSVCRT.lib/MSVCR70.dll.c` (0/37)

### (free functions)
- [-] `vsnprintf` · 0071ec2a — third-party library
- [-] `purecall` · 0071ec30 · legions 0069f550 — third-party library
- [-] `operator_delete` · 0071ec36 — third-party library
- [-] `operator_delete[]` · 0071ec3c — third-party library
- [-] `free` · 0071ec42 — third-party library
- [-] `setlocale` · 0071ee68 — third-party library
- [-] `__CIpow` · 0071ef66 — Rust std replaces it
- [-] `floor` · 0071ef6c — third-party library
- [-] `snprintf` · 0071f06a — third-party library
- [-] `malloc` · 0071f070 — third-party library
- [-] `___CxxFrameHandler` · 0071f0a2 — Rust std replaces it
- [-] `sscanf` · 0071f0a8 — third-party library
- [-] `__CIasin` · 0071f0d0 — Rust std replaces it
- [-] `__CIacos` · 0071f0d6 — Rust std replaces it
- [-] `__CIfmod` · 0071f0dc — Rust std replaces it
- [-] `isdigit` · 0071f110 — third-party library
- [-] `isalpha` · 0071f116 — third-party library
- [-] `isspace` · 0071f160 — third-party library
- [-] `except_handler3` · 0071f332 — third-party library
- [-] `calloc` · 0071f338 · legions 006774c0 — third-party library
- [-] `finite` · 0071f33e — third-party library
- [-] `atoi` · 0071f484 — third-party library
- [-] `isalnum` · 0071f48a — third-party library
- [-] `strdup` · 0071f490 — third-party library
- [-] `longjmp` · 0071f496 · legions 006b9a00 — third-party library
- [-] `setjmp3` · 0071f49c — third-party library
- [-] `ftol` · 0071f4a2 — third-party library
- [-] `callnewh` · 0071f4a8 — third-party library
- [-] `__XcptFilter` · 0071f4ae — Rust std replaces it
- [-] `amsg_exit` · 0071f4b4 — third-party library
- [-] `initterm` · 0071f542 — third-party library
- [-] `___dllonexit` · 0071f5ac — Rust std replaces it
- [-] `terminate` · 0071f5b2 — third-party library
- [-] `controlfp` · 0071f5b8 — third-party library
- [-] `__CxxThrowException@8` · 0073ddfa — Rust std replaces it
- [-] `signal` · 0073de00 — third-party library

### type_info
- [-] `type_info::~type_info` · 0073de06 — third-party library

## `_libraries/MSVCRT.lib/ehprolog.obj.c` (0/1)

### (free functions)
- [-] `__EH_prolog` · 0071f344 · legions 0069fc44 — Rust std replaces it

## `_libraries/MSVCRT.lib/ehvecctr.obj.c` (0/2)

### (free functions)
- [-] ``eh_vector_constructor_iterator'` · 0071f421 · legions 0069fd21 — third-party library
- [-] `FUN_0071f46b` · 0071f46b · legions 0069fd6b — third-party library

## `_libraries/MSVCRT.lib/ehvecdtr.obj.c` (0/3)

### (free functions)
- [-] `__ArrayUnwind` · 0071f363 · legions 0069fc63 — Rust std replaces it
- [-] ``eh_vector_destructor_iterator'` · 0071f3c1 · legions 0069fcc1 — third-party library
- [-] `FUN_0071f409` · 0071f409 · legions 0069fd09 — third-party library

## `_libraries/MSVCRT.lib/ftol2.obj.c` (0/1)

### (free functions)
- [-] `_ftol2` · 0071ee70 · legions 0069f790 — Rust std replaces it

## `_libraries/MSVCRT.lib/initsect.obj.c` (0/2)

### (free functions)
- [-] `_RTC_Initialize` · 0071f4ba · legions 0069fdba — Rust std replaces it
- [-] `_RTC_Terminate` · 0071f4fe — Rust std replaces it

## `_libraries/MSVCRT.lib/sehprolg.obj.c` (0/2)

### (free functions)
- [-] `__SEH_prolog` · 0071f560 · legions 0069fe60 — Rust std replaces it
- [-] `__SEH_epilog` · 0071f59b · legions 0069fe9b — Rust std replaces it

## `_libraries/animsys.lib/AnimTrack.obj.c` (0/1)

### WeenieHook
- [-] `WeenieHook::WeenieHook` · 006898c0 — third-party library

## `_libraries/ccell.lib/CLandBLockPathGen.obj.c` (0/17)

### Ray
- [-] `Ray::GetCenterPoint` · 00490b80 · legions 00489860 — third-party library

### PathLink
- [-] `PathLink::~PathLink` · 00490bf0 · legions 004898d0 — third-party library

### Position
- [-] `Position::localtoglobal` · 00490c20 · legions 00489900 — third-party library

### Plane
- [-] `Plane::InitVerticalPlane` · 00490c80 · legions 00489960 — third-party library

### SmartArray<PathLink>
- [-] `SmartArray<PathLink>::Reset` · 00490df0 · legions 00489ad0 — Rust std replaces it

### PathLink
- [-] `PathLink::operator=` · 00490e70 · legions 00489b50 — third-party library

### SimplePolygon
- [-] `SimplePolygon::SimplePolygon` · 00490f10 — third-party library

### SmartArray<PathLink>
- [-] `SmartArray<PathLink>::~SmartArray<PathLink>` · 00490f30 · legions 00489c80 — third-party library

### SmartArray<unsigned_long>
- [-] `SmartArray<unsigned_long>::contains` · 00490f90 · legions 00489ce0 — Rust std replaces it

### SmartArray<PathLink>
- [-] `SmartArray<PathLink>::grow` · 00490fc0 · legions 00489d10 — Rust std replaces it

### SmartArray<Vector>
- [-] `SmartArray<Vector>::operator=` · 00491080 · legions 00489dd0 — third-party library

### SmartArray<Ray>
- [-] `SmartArray<Ray>::grow` · 004910e0 · legions 00489e30 — Rust std replaces it

### SimplePolygon
- [-] `SimplePolygon::operator=` · 004911f0 · legions 00489f40 — third-party library

### SmartArray<BBox>
- [-] `SmartArray<BBox>::AddToEnd` · 00491210 · legions 00489f60 — third-party library

### SmartArray<PathLink>
- [-] `SmartArray<PathLink>::AddToEnd` · 00491280 · legions 00489fd0 — third-party library

### SmartArray<Ray>
- [-] `SmartArray<Ray>::operator=` · 004912d0 · legions 0048a020 — third-party library

### PathNode
- [-] `PathNode::AddLink` · 00491360 — third-party library

## `_libraries/comsupp.lib/comraise.obj.c` (0/3)

### (free functions)
- [-] `_com_raise_error` · 0071f5c0 · legions 0069fec0 — Rust std replaces it

### _com_error
- [-] `_com_error::_com_error` · 0071f600 — third-party library
- [-] `_com_error::~_com_error` · 0071f640 · legions 0069ff40 — third-party library

## `_libraries/d3dx8.lib/cd3dxassembler.obj.c` (0/14)

### CD3DXAssembler
- [-] `CD3DXAssembler::CD3DXAssembler` · 0070126b · legions 0068101b — third-party library
- [-] `CD3DXAssembler::~CD3DXAssembler` · 00701314 · legions 006810c4 — third-party library
- [-] `CD3DXAssembler::Error` · 007013a3 · legions 00681153 — third-party library
- [-] `CD3DXAssembler::DecodeOpcode` · 007014d9 · legions 00681289 — third-party library
- [-] `CD3DXAssembler::DecodeMask` · 00701bf0 · legions 006819a0 — third-party library
- [-] `CD3DXAssembler::DecodeSwizzle` · 00701c99 · legions 00681a49 — third-party library
- [-] `CD3DXAssembler::DecodeRegister` · 00701d3a · legions 00681aea — third-party library
- [-] `CD3DXAssembler::SetConstant` · 007020db — third-party library
- [-] `CD3DXAssembler::UpdateDebugText` · 00702184 · legions 00681f34 — third-party library
- [-] `CD3DXAssembler::UpdateDebugFileLine` · 0070225a · legions 0068200a — third-party library
- [-] `CD3DXAssembler::Token` · 007023d1 · legions 00682181 — third-party library
- [-] `CD3DXAssembler::Production` · 007028e9 — third-party library

### (free functions)
- [-] `d3dxasm_parse` · 00703220 · legions 00682fd0 — third-party library

### CD3DXAssembler
- [-] `CD3DXAssembler::Assemble` · 007036d9 · legions 00683489 — third-party library

## `_libraries/d3dx8.lib/cd3dxblt.obj.c` (0/35)

### CD3DXBlt
- [-] `CD3DXBlt::CD3DXBlt` · 00703c2c · legions 006839dc — third-party library
- [-] `CD3DXBlt::~CD3DXBlt` · 00703c36 · legions 006839e6 — third-party library

### TF_Row
- [-] `TF_Row::TF_Row` · 00703c54 — third-party library
- [-] `TF_Row::~TF_Row` · 00703c61 — third-party library

### (free functions)
- [-] `TF_SetupTriangle` · 00703c6a · legions 00683a1a — third-party library

### CD3DXBlt
- [-] `CD3DXBlt::BltTriangle3D` · 00703f73 · legions 00683d23 — third-party library
- [-] `CD3DXBlt::BltSame_DXTn` · 007043e7 · legions 00684197 — third-party library

### (free functions)
- [-] `c_BoxFilter2D_A8R8G8B8` · 00704575 · legions 00684325 — third-party library

### CD3DXBlt
- [-] `CD3DXBlt::BltBox2D_A8R8G8B8` · 0070469b · legions 0068444b — third-party library

### (free functions)
- [-] `c_BoxFilter2D_X8R8G8B8` · 007046cc · legions 0068447c — third-party library
- [-] `mmx_BoxFilter2D_A8R8G8B8` · 007047b4 · legions 00684564 — third-party library
- [-] `init_BoxFilter2D_A8R8G8B8` · 007048ca · legions 0068467a — third-party library
- [-] `init_BoxFilter2D_X8R8G8B8` · 00704902 · legions 006846b2 — third-party library

### CD3DXBlt
- [-] `CD3DXBlt::BltBox2D_X8R8G8B8` · 00704931 · legions 006846e1 — third-party library
- [-] `CD3DXBlt::BltBox2D_R5G6B5` · 00704962 · legions 00684712 — third-party library
- [-] `CD3DXBlt::BltBox2D_X1R5G5B5` · 00704ab2 · legions 00684862 — third-party library
- [-] `CD3DXBlt::BltBox2D_A1R5G5B5` · 00704bd6 · legions 00684986 — third-party library
- [-] `CD3DXBlt::BltBox2D_A4R4G4B4` · 00704d25 · legions 00684ad5 — third-party library
- [-] `CD3DXBlt::BltBox2D_R3G3B2` · 00704e78 · legions 00684c28 — third-party library
- [-] `CD3DXBlt::BltBox2D_A8` · 00704fb7 · legions 00684d67 — third-party library
- [-] `CD3DXBlt::BltBox2D_A8R3G3B2` · 00705052 · legions 00684e02 — third-party library
- [-] `CD3DXBlt::BltBox2D_X4R4G4B4` · 0070519e · legions 00684f4e — third-party library
- [-] `CD3DXBlt::BltBox2D_A8L8` · 007052bf · legions 0068506f — third-party library
- [-] `CD3DXBlt::BltBox2D_A4L4` · 0070540b · legions 006851bb — third-party library
- [-] `CD3DXBlt::BltSame` · 0070554a · legions 006852fa — third-party library
- [-] `CD3DXBlt::BltCopy` · 00705682 · legions 00685432 — third-party library
- [-] `CD3DXBlt::BltNone` · 00705773 · legions 00685523 — third-party library
- [-] `CD3DXBlt::BltPoint` · 00705975 · legions 00685725 — third-party library
- [-] `CD3DXBlt::BltBox2D` · 00705b79 · legions 00685929 — third-party library
- [-] `CD3DXBlt::BltBox3D` · 00705f9c · legions 00685d4c — third-party library

### (free functions)
- [-] `LF_SetupLinear` · 0070642e · legions 006861de — third-party library

### CD3DXBlt
- [-] `CD3DXBlt::BltLinear2D` · 0070652e · legions 006862de — third-party library
- [-] `CD3DXBlt::BltLinear3D` · 00706914 · legions 006866c4 — third-party library
- [-] `CD3DXBlt::BltTriangle2D` · 00707045 · legions 00686df5 — third-party library
- [-] `CD3DXBlt::Blt` · 007073e7 · legions 00687197 — third-party library

## `_libraries/d3dx8.lib/cd3dxbuffer.obj.c` (0/8)

### CD3DXBuffer
- [-] `CD3DXBuffer::~CD3DXBuffer` · 0070d89a · legions 0068d64a — third-party library
- [-] `CD3DXBuffer::Init` · 0070d8af · legions 0068d65f — third-party library
- [-] `CD3DXBuffer::AddRef` · 0070d8d5 · legions 0068d685 — third-party library
- [-] `CD3DXBuffer::Release` · 0070d8e2 · legions 0068d692 — third-party library
- [-] `CD3DXBuffer::GetBufferPointer` · 0070d902 · legions 0068d6b2 — third-party library
- [-] `CD3DXBuffer::GetBufferSize` · 0070d90c · legions 0068d6bc — third-party library
- [-] `CD3DXBuffer::QueryInterface` · 0070d932 · legions 0068d6e2 — third-party library
- [-] `CD3DXBuffer::CD3DXBuffer` · 0070d979 · legions 0068d729 — third-party library

## `_libraries/d3dx8.lib/cd3dxcodec.obj.c` (0/119)

### CD3DXCodec
- [-] `CD3DXCodec::~CD3DXCodec` · 00707982 · legions 00687732 — third-party library
- [-] `CD3DXCodec::Decode` · 00707995 · legions 00687745 — third-party library
- [-] `CD3DXCodec::Convert` · 00707998 · legions 00687748 — third-party library

### CD3DXCodecYUV
- [-] `CD3DXCodecYUV::QuantizeColorKey` · 00707c8f — third-party library

### CD3DXCodecDXT
- [-] `CD3DXCodecDXT::~CD3DXCodecDXT` · 00707c90 · legions 00687a40 — third-party library
- [-] `CD3DXCodecDXT::Encode` · 00707d30 · legions 00687ae0 — third-party library

### (free functions)
- [-] `F2IBegin` · 0070812a · legions 00687eda — third-party library

### CD3DXCodec
- [-] `CD3DXCodec::CD3DXCodec` · 0070814d · legions 00687efd — third-party library
- [-] `CD3DXCodec::ConvertFrom` · 007083b5 · legions 00688165 — third-party library
- [-] `CD3DXCodec::QuantizeColorKey` · 00708430 · legions 006881e0 — third-party library
- [-] `CD3DXCodec::ColorKey` · 007084fc · legions 006882ac — third-party library

### CD3DXCodec_R8G8B8
- [-] `CD3DXCodec_R8G8B8::Encode` · 0070856c · legions 0068831c — third-party library

### CD3DXCodec_A8R8G8B8
- [-] `CD3DXCodec_A8R8G8B8::Encode` · 00708658 · legions 00688408 — third-party library

### CD3DXCodec_X8R8G8B8
- [-] `CD3DXCodec_X8R8G8B8::Encode` · 0070877b · legions 0068852b — third-party library

### CD3DXCodec_R5G6B5
- [-] `CD3DXCodec_R5G6B5::Encode` · 0070887c · legions 0068862c — third-party library

### CD3DXCodec_X1R5G5B5
- [-] `CD3DXCodec_X1R5G5B5::Encode` · 0070897d · legions 0068872d — third-party library

### CD3DXCodec_A1R5G5B5
- [-] `CD3DXCodec_A1R5G5B5::Encode` · 00708a7e · legions 0068882e — third-party library

### CD3DXCodec_A4R4G4B4
- [-] `CD3DXCodec_A4R4G4B4::Encode` · 00708b9b · legions 0068894b — third-party library

### CD3DXCodec_R3G3B2
- [-] `CD3DXCodec_R3G3B2::Encode` · 00708cbe · legions 00688a6e — third-party library

### CD3DXCodec_A8
- [-] `CD3DXCodec_A8::Encode` · 00708dbc · legions 00688b6c — third-party library

### CD3DXCodec_A8R3G3B2
- [-] `CD3DXCodec_A8R3G3B2::Encode` · 00708e65 · legions 00688c15 — third-party library

### CD3DXCodec_X4R4G4B4
- [-] `CD3DXCodec_X4R4G4B4::Encode` · 00708f88 · legions 00688d38 — third-party library

### CD3DXCodec_A2B10G10R10
- [-] `CD3DXCodec_A2B10G10R10::Encode` · 00709089 · legions 00688e39 — third-party library

### CD3DXCodec_G16R16
- [-] `CD3DXCodec_G16R16::Encode` · 007091ac · legions 00688f5c — third-party library

### CD3DXCodec_A8P8
- [-] `CD3DXCodec_A8P8::Encode` · 0070927c · legions 0068902c — third-party library

### CD3DXCodec_P8
- [-] `CD3DXCodec_P8::Encode` · 00709394 · legions 00689144 — third-party library

### CD3DXCodec_L8
- [-] `CD3DXCodec_L8::Encode` · 00709459 · legions 00689209 — third-party library

### CD3DXCodec_A8L8
- [-] `CD3DXCodec_A8L8::Encode` · 0070951e · legions 006892ce — third-party library

### CD3DXCodec_A4L4
- [-] `CD3DXCodec_A4L4::Encode` · 00709614 · legions 006893c4 — third-party library

### CD3DXCodec_V8U8
- [-] `CD3DXCodec_V8U8::Encode` · 007096ff · legions 006894af — third-party library

### CD3DXCodec_L6V5U5
- [-] `CD3DXCodec_L6V5U5::Encode` · 007097d4 · legions 00689584 — third-party library

### CD3DXCodec_X8L8V8U8
- [-] `CD3DXCodec_X8L8V8U8::Encode` · 007098d9 · legions 00689689 — third-party library

### CD3DXCodec_Q8W8V8U8
- [-] `CD3DXCodec_Q8W8V8U8::Encode` · 007099e4 · legions 00689794 — third-party library

### CD3DXCodec_V16U16
- [-] `CD3DXCodec_V16U16::Encode` · 00709b1e · legions 006898ce — third-party library

### CD3DXCodec_W11V11U10
- [-] `CD3DXCodec_W11V11U10::Encode` · 00709bf4 · legions 006899a4 — third-party library

### CD3DXCodec_A2W10V10U10
- [-] `CD3DXCodec_A2W10V10U10::Encode` · 00709cfe · legions 00689aae — third-party library

### CD3DXCodec_D16_LOCKABLE
- [-] `CD3DXCodec_D16_LOCKABLE::Encode` · 00709e34 · legions 00689be4 — third-party library

### CD3DXCodec_D3DX_A16L16
- [-] `CD3DXCodec_D3DX_A16L16::Encode` · 00709efb · legions 00689cab — third-party library

### CD3DXCodec_D3DX_R16G16B16
- [-] `CD3DXCodec_D3DX_R16G16B16::Encode` · 00709fe9 · legions 00689d99 — third-party library

### CD3DXCodec_D3DX_A16R16G16B16
- [-] `CD3DXCodec_D3DX_A16R16G16B16::Encode` · 0070a0d8 · legions 00689e88 — third-party library

### CD3DXCodec_R8G8B8
- [-] `CD3DXCodec_R8G8B8::Decode` · 0070a234 · legions 00689fe4 — third-party library

### CD3DXCodec_A8R8G8B8
- [-] `CD3DXCodec_A8R8G8B8::Decode` · 0070a2c0 · legions 0068a070 — third-party library

### CD3DXCodec_X8R8G8B8
- [-] `CD3DXCodec_X8R8G8B8::Decode` · 0070a366 · legions 0068a116 — third-party library

### CD3DXCodec_R5G6B5
- [-] `CD3DXCodec_R5G6B5::Decode` · 0070a402 · legions 0068a1b2 — third-party library

### CD3DXCodec_X1R5G5B5
- [-] `CD3DXCodec_X1R5G5B5::Decode` · 0070a4a7 · legions 0068a257 — third-party library

### CD3DXCodec_A1R5G5B5
- [-] `CD3DXCodec_A1R5G5B5::Decode` · 0070a549 · legions 0068a2f9 — third-party library

### CD3DXCodec_A4R4G4B4
- [-] `CD3DXCodec_A4R4G4B4::Decode` · 0070a5f9 · legions 0068a3a9 — third-party library

### CD3DXCodec_A2B10G10R10
- [-] `CD3DXCodec_A2B10G10R10::Decode` · 0070a6a8 · legions 0068a458 — third-party library

### CD3DXCodec_G16R16
- [-] `CD3DXCodec_G16R16::Decode` · 0070a77e · legions 0068a52e — third-party library

### CD3DXCodec_R3G3B2
- [-] `CD3DXCodec_R3G3B2::Decode` · 0070a810 · legions 0068a5c0 — third-party library

### CD3DXCodec_A8
- [-] `CD3DXCodec_A8::Decode` · 0070a8b2 · legions 0068a662 — third-party library

### CD3DXCodec_A8R3G3B2
- [-] `CD3DXCodec_A8R3G3B2::Decode` · 0070a924 · legions 0068a6d4 — third-party library

### CD3DXCodec_X4R4G4B4
- [-] `CD3DXCodec_X4R4G4B4::Decode` · 0070a9d6 · legions 0068a786 — third-party library

### CD3DXCodec_A8P8
- [-] `CD3DXCodec_A8P8::Decode` · 0070aa75 · legions 0068a825 — third-party library

### CD3DXCodec_P8
- [-] `CD3DXCodec_P8::Decode` · 0070aafc · legions 0068a8ac — third-party library

### CD3DXCodec_L8
- [-] `CD3DXCodec_L8::Decode` · 0070ab60 · legions 0068a910 — third-party library

### CD3DXCodec_A8L8
- [-] `CD3DXCodec_A8L8::Decode` · 0070abce · legions 0068a97e — third-party library

### CD3DXCodec_A4L4
- [-] `CD3DXCodec_A4L4::Decode` · 0070ac57 · legions 0068aa07 — third-party library

### CD3DXCodec_V8U8
- [-] `CD3DXCodec_V8U8::Decode` · 0070ace0 · legions 0068aa90 — third-party library

### CD3DXCodec_L6V5U5
- [-] `CD3DXCodec_L6V5U5::Decode` · 0070ad62 · legions 0068ab12 — third-party library

### CD3DXCodec_X8L8V8U8
- [-] `CD3DXCodec_X8L8V8U8::Decode` · 0070ae13 · legions 0068abc3 — third-party library

### CD3DXCodec_Q8W8V8U8
- [-] `CD3DXCodec_Q8W8V8U8::Decode` · 0070aea4 · legions 0068ac54 — third-party library

### CD3DXCodec_V16U16
- [-] `CD3DXCodec_V16U16::Decode` · 0070af3b · legions 0068aceb — third-party library

### CD3DXCodec_W11V11U10
- [-] `CD3DXCodec_W11V11U10::Decode` · 0070afbe · legions 0068ad6e — third-party library

### CD3DXCodec_A2W10V10U10
- [-] `CD3DXCodec_A2W10V10U10::Decode` · 0070b078 · legions 0068ae28 — third-party library

### CD3DXCodec_D16_LOCKABLE
- [-] `CD3DXCodec_D16_LOCKABLE::Decode` · 0070b14b · legions 0068aefb — third-party library

### CD3DXCodec_D3DX_A16L16
- [-] `CD3DXCodec_D3DX_A16L16::Decode` · 0070b1bb · legions 0068af6b — third-party library

### CD3DXCodec_D3DX_R16G16B16
- [-] `CD3DXCodec_D3DX_R16G16B16::Decode` · 0070b249 · legions 0068aff9 — third-party library

### CD3DXCodec_D3DX_A16R16G16B16
- [-] `CD3DXCodec_D3DX_A16R16G16B16::Decode` · 0070b2d5 · legions 0068b085 — third-party library

### CD3DXCodec_R8G8B8
- [-] `CD3DXCodec_R8G8B8::CD3DXCodec_R8G8B8` · 0070b3b5 — third-party library

### CD3DXCodecYUV
- [-] `CD3DXCodecYUV::CD3DXCodecYUV` · 0070b3d1 · legions 0068b181 — third-party library

### CD3DXCodec_A8R8G8B8
- [-] `CD3DXCodec_A8R8G8B8::CD3DXCodec_A8R8G8B8` · 0070b4c9 — third-party library

### CD3DXCodec_X8R8G8B8
- [-] `CD3DXCodec_X8R8G8B8::CD3DXCodec_X8R8G8B8` · 0070b4e5 — third-party library

### CD3DXCodec_R5G6B5
- [-] `CD3DXCodec_R5G6B5::CD3DXCodec_R5G6B5` · 0070b51d — third-party library

### CD3DXCodecYUV
- [-] `CD3DXCodecYUV::Commit` · 0070b539 · legions 0068b2e9 — third-party library

### CD3DXCodec_X1R5G5B5
- [-] `CD3DXCodec_X1R5G5B5::CD3DXCodec_X1R5G5B5` · 0070b725 — third-party library

### CD3DXCodec_A1R5G5B5
- [-] `CD3DXCodec_A1R5G5B5::CD3DXCodec_A1R5G5B5` · 0070b741 — third-party library

### CD3DXCodecYUV
- [-] `CD3DXCodecYUV::Fetch` · 0070b75d · legions 0068b50d — third-party library

### CD3DXCodec_A4R4G4B4
- [-] `CD3DXCodec_A4R4G4B4::CD3DXCodec_A4R4G4B4` · 0070ba2e — third-party library

### CD3DXCodec_R3G3B2
- [-] `CD3DXCodec_R3G3B2::CD3DXCodec_R3G3B2` · 0070ba4a — third-party library

### CD3DXCodec_A8
- [-] `CD3DXCodec_A8::CD3DXCodec_A8` · 0070ba66 — third-party library

### CD3DXCodec_A8R3G3B2
- [-] `CD3DXCodec_A8R3G3B2::CD3DXCodec_A8R3G3B2` · 0070ba82 — third-party library

### CD3DXCodec_X4R4G4B4
- [-] `CD3DXCodec_X4R4G4B4::CD3DXCodec_X4R4G4B4` · 0070ba9e — third-party library

### CD3DXCodecYUV
- [-] `CD3DXCodecYUV::Encode` · 0070baba · legions 0068b86a — third-party library

### CD3DXCodec_A2B10G10R10
- [-] `CD3DXCodec_A2B10G10R10::CD3DXCodec_A2B10G10R10` · 0070bb47 — third-party library

### CD3DXCodecYUV
- [-] `CD3DXCodecYUV::Decode` · 0070bb63 — third-party library

### CD3DXCodec_G16R16
- [-] `CD3DXCodec_G16R16::CD3DXCodec_G16R16` · 0070bbd2 — third-party library

### CD3DXCodecDXT
- [-] `CD3DXCodecDXT::CD3DXCodecDXT` · 0070bbee · legions 0068b99e — third-party library

### CD3DXCodec_A8P8
- [-] `CD3DXCodec_A8P8::CD3DXCodec_A8P8` · 0070bde0 · legions 0068bb90 — third-party library

### CD3DXCodec_P8
- [-] `CD3DXCodec_P8::CD3DXCodec_P8` · 0070bdfc · legions 0068bbac — third-party library

### CD3DXCodec_L8
- [-] `CD3DXCodec_L8::CD3DXCodec_L8` · 0070be18 · legions 0068bbc8 — third-party library

### CD3DXCodec_A8L8
- [-] `CD3DXCodec_A8L8::CD3DXCodec_A8L8` · 0070be34 · legions 0068bbe4 — third-party library

### CD3DXCodec_A4L4
- [-] `CD3DXCodec_A4L4::CD3DXCodec_A4L4` · 0070be6c · legions 0068bc1c — third-party library

### CD3DXCodecDXT
- [-] `CD3DXCodecDXT::QuantizeColorKey` · 0070be88 · legions 0068bc38 — third-party library

### CD3DXCodec_V8U8
- [-] `CD3DXCodec_V8U8::CD3DXCodec_V8U8` · 0070bf89 · legions 0068bd39 — third-party library

### CD3DXCodec_L6V5U5
- [-] `CD3DXCodec_L6V5U5::CD3DXCodec_L6V5U5` · 0070bfa5 · legions 0068bd55 — third-party library

### CD3DXCodec_X8L8V8U8
- [-] `CD3DXCodec_X8L8V8U8::CD3DXCodec_X8L8V8U8` · 0070bfc1 · legions 0068bd71 — third-party library

### CD3DXCodec_Q8W8V8U8
- [-] `CD3DXCodec_Q8W8V8U8::CD3DXCodec_Q8W8V8U8` · 0070bfdd · legions 0068bd8d — third-party library

### CD3DXCodec_V16U16
- [-] `CD3DXCodec_V16U16::CD3DXCodec_V16U16` · 0070bff9 · legions 0068bda9 — third-party library

### CD3DXCodec_W11V11U10
- [-] `CD3DXCodec_W11V11U10::CD3DXCodec_W11V11U10` · 0070c015 · legions 0068bdc5 — third-party library

### CD3DXCodec_A2W10V10U10
- [-] `CD3DXCodec_A2W10V10U10::CD3DXCodec_A2W10V10U10` · 0070c031 · legions 0068bde1 — third-party library

### CD3DXCodec_D16_LOCKABLE
- [-] `CD3DXCodec_D16_LOCKABLE::CD3DXCodec_D16_LOCKABLE` · 0070c04d · legions 0068be02 — third-party library

### CD3DXCodec_D3DX_L16
- [-] `CD3DXCodec_D3DX_L16::CD3DXCodec_D3DX_L16` · 0070c069 · legions 0068be1e — third-party library

### CD3DXCodecDXT
- [-] `CD3DXCodecDXT::Decode` · 0070c085 · legions 0068be3a — third-party library

### CD3DXCodec_D3DX_A16L16
- [-] `CD3DXCodec_D3DX_A16L16::CD3DXCodec_D3DX_A16L16` · 0070c32e · legions 0068c0e3 — third-party library

### CD3DXCodec_D3DX_R16G16B16
- [-] `CD3DXCodec_D3DX_R16G16B16::CD3DXCodec_D3DX_R16G16B16` · 0070c34a · legions 0068c0ff — third-party library

### CD3DXCodec_D3DX_A16R16G16B16
- [-] `CD3DXCodec_D3DX_A16R16G16B16::CD3DXCodec_D3DX_A16R16G16B16` · 0070c366 · legions 0068c11b — third-party library

### CD3DXCodec_R8G8B8
- [-] `CD3DXCodec_R8G8B8::~CD3DXCodec_R8G8B8` · 0070c382 · legions 0068bdfd — third-party library

### CD3DXCodec_DXT1
- [-] `CD3DXCodec_DXT1::CD3DXCodec_DXT1` · 0070c387 · legions 0068c137 — third-party library

### CD3DXCodec_DXT2
- [-] `CD3DXCodec_DXT2::CD3DXCodec_DXT2` · 0070c3bb · legions 0068c16b — third-party library

### CD3DXCodec_DXT3
- [-] `CD3DXCodec_DXT3::CD3DXCodec_DXT3` · 0070c3d3 · legions 0068c183 — third-party library

### CD3DXCodec_DXT1
- [-] `CD3DXCodec_DXT1::~CD3DXCodec_DXT1` · 0070c3eb · legions 0068c1cb — third-party library

### CD3DXCodec_DXT4
- [-] `CD3DXCodec_DXT4::CD3DXCodec_DXT4` · 0070c3f0 · legions 0068c19b — third-party library

### CD3DXCodec_DXT5
- [-] `CD3DXCodec_DXT5::CD3DXCodec_DXT5` · 0070c408 · legions 0068c1b3 — third-party library

### CD3DXCodecYUV
- [-] `CD3DXCodecYUV::~CD3DXCodecYUV` · 0070c420 · legions 0068c1d0 — third-party library

### CD3DXCodec_UYVY
- [-] `CD3DXCodec_UYVY::CD3DXCodec_UYVY` · 0070c469 · legions 0068c219 — third-party library

### CD3DXCodec_YUY2
- [-] `CD3DXCodec_YUY2::CD3DXCodec_YUY2` · 0070c481 · legions 0068c231 — third-party library

### CD3DXCodec
- [-] `CD3DXCodec::Create` · 0070c499 · legions 0068c249 — third-party library

### CD3DXCodec_UYVY
- [-] `CD3DXCodec_UYVY::~CD3DXCodec_UYVY` · 0070cc2d — third-party library

## `_libraries/d3dx8.lib/cd3dxlock.obj.c` (0/3)

### CD3DXLockSurface
- [-] `CD3DXLockSurface::Unlock` · 00707501 · legions 006872b1 — third-party library
- [-] `CD3DXLockSurface::~CD3DXLockSurface` · 0070756a · legions 0068731a — third-party library
- [-] `CD3DXLockSurface::Lock` · 0070756f · legions 0068731f — third-party library

## `_libraries/d3dx8.lib/cd3dxstack.obj.c` (0/9)

### CD3DXLockSurface
- [-] `CD3DXLockSurface::CD3DXLockSurface` · 0070d5f3 · legions 0068d3a3 — third-party library

### CD3DXDwStack
- [-] `CD3DXDwStack::~CD3DXDwStack` · 0070d603 · legions 0068d3b3 — third-party library
- [-] `CD3DXDwStack::Push` · 0070d611 · legions 0068d3c1 — third-party library

### CD3DXSzStack
- [-] `CD3DXSzStack::Pop` · 0070d67f · legions 0068d42f — third-party library

### CD3DXDwStack
- [-] `CD3DXDwStack::GetLastError` · 0070d6b3 · legions 0068d463 — third-party library

### CD3DXSzStack
- [-] `CD3DXSzStack::CD3DXSzStack` · 0070d6bb · legions 0068d46b — third-party library
- [-] `CD3DXSzStack::~CD3DXSzStack` · 0070d6ce · legions 0068d47e — third-party library
- [-] `CD3DXSzStack::Push` · 0070d6ff · legions 0068d4af — third-party library
- [-] `CD3DXSzStack::GenerateStringBuffer` · 0070d7b2 · legions 0068d562 — third-party library

## `_libraries/d3dx8.lib/cpudetect.obj.c` (0/8)

### (free functions)
- [-] `GetD3DRegValue` · 00700ee3 · legions 00680c93 — third-party library
- [-] `_asm_isMMX` · 00700f3f · legions 00680cef — Rust std replaces it
- [-] `isMMXprocessor` · 00700f64 · legions 00680d14 — third-party library
- [-] `isX3Dprocessor` · 00700ffd · legions 00680dad — third-party library
- [-] `IsIntelSSEProcessor` · 00701035 · legions 00680de5 — third-party library
- [-] `FUN_0070107b` · 0070107b · legions 00680e2b — third-party library
- [-] `D3DXIsProcessorFeaturePresent` · 007010da · legions 00680e8a — third-party library
- [-] `_D3DXCpuOptimizations@4` · 0070118e · legions 00680f3e — Rust std replaces it

## `_libraries/d3dx8.lib/d3dx8core.obj.c` (0/1)

### (free functions)
- [-] `_D3DXAssembleShader@24` · 00700bf7 · legions 006809a7 — Rust std replaces it

## `_libraries/d3dx8.lib/d3dx8dbg.obj.c` (0/1)

### (free functions)
- [-] `D3DXDebugMute` · 00703b9c · legions 0068394c — third-party library

## `_libraries/d3dx8.lib/d3dx8tex.obj.c` (0/3)

### (free functions)
- [-] `_D3DXLoadSurfaceFromMemory@40` · 00700c5b · legions 00680a0b — Rust std replaces it

### D3DXCOLOR
- [-] `D3DXCOLOR::D3DXCOLOR` · 00700d3a — third-party library

### (free functions)
- [-] `_D3DXLoadSurfaceFromSurface@32` · 00700d3d · legions 00680aed — Rust std replaces it

## `_libraries/d3dx8.lib/d3dxmath.obj.c` (1/125)

### (free functions)
- [-] `WithinEpsilon` · 006fd6c0 · legions 0067d470 — third-party library
- [-] `init_D3DXVec2Normalize` · 006fd6ed — third-party library
- [-] `init_D3DXVec2Hermite` · 006fd6fa · legions 0067d4aa — third-party library
- [-] `c_D3DXVec2Hermite` · 006fd724 · legions 0067d4d4 — third-party library
- [-] `init_D3DXVec2CatmullRom` · 006fd7b2 · legions 0067d562 — third-party library
- [-] `c_D3DXVec2CatmullRom` · 006fd7dc · legions 0067d58c — third-party library
- [-] `init_D3DXVec2BaryCentric` · 006fd88b · legions 0067d63b — third-party library
- [-] `c_D3DXVec2BaryCentric` · 006fd8ba · legions 0067d66a — third-party library
- [-] `init_D3DXVec2Transform` · 006fd8fd — third-party library
- [-] `init_D3DXVec2TransformCoord` · 006fd90a — third-party library
- [-] `init_D3DXVec2TransformNormal` · 006fd917 — third-party library
- [-] `init_D3DXVec3Normalize` · 006fd924 · legions 0067d6d4 — third-party library
- [-] `_D3DXVec3Normalize@8` · 006fd931 · legions 0067d6e1 — Rust std replaces it
- [-] `init_D3DXVec3Hermite` · 006fd937 — third-party library
- [-] `c_D3DXVec3Hermite` · 006fd961 · legions 0067d711 — third-party library
- [-] `init_D3DXVec3CatmullRom` · 006fda0f · legions 0067d7bf — third-party library
- [-] `c_D3DXVec3CatmullRom` · 006fda39 · legions 0067d7e9 — third-party library
- [-] `init_D3DXVec3BaryCentric` · 006fdb0a · legions 0067d8ba — third-party library
- [-] `c_D3DXVec3BaryCentric` · 006fdb39 · legions 0067d8e9 — third-party library
- [-] `init_D3DXVec3Transform` · 006fdb96 · legions 0067d9de — third-party library
- [-] `c_D3DXVec3Transform` · 006fdba3 — third-party library
- [-] `init_D3DXVec3TransformCoord` · 006fdc2e — third-party library
- [-] `_D3DXVec3TransformCoord@12` · 006fdc3b — Rust std replaces it
- [-] `init_D3DXVec3TransformNormal` · 006fdc41 — third-party library
- [-] `c_D3DXVec3TransformNormal` · 006fdc4e — third-party library
- [-] `init_D3DXVec3Project` · 006fdcb4 · legions 0067da64 — third-party library
- [-] `init_D3DXVec3Unproject` · 006fdcc5 — third-party library
- [-] `init_D3DXVec4Cross` · 006fdcd6 — third-party library
- [-] `c_D3DXVec4Cross` · 006fdce3 — third-party library
- [-] `init_D3DXVec4Normalize` · 006fddd3 — third-party library
- [-] `init_D3DXVec4Hermite` · 006fdde0 — third-party library
- [-] `c_D3DXVec4Hermite` · 006fde0a · legions 0067dbba — third-party library
- [-] `init_D3DXVec4CatmullRom` · 006fdedb · legions 0067dc8b — third-party library
- [-] `c_D3DXVec4CatmullRom` · 006fdf05 · legions 0067dcb5 — third-party library
- [-] `init_D3DXVec4BaryCentric` · 006fdff8 · legions 0067dda8 — third-party library
- [-] `c_D3DXVec4BaryCentric` · 006fe027 · legions 0067ddd7 — third-party library
- [-] `init_D3DXVec4Transform` · 006fe09e — third-party library
- [-] `c_D3DXVec4Transform` · 006fe0ab — third-party library
- [-] `init_D3DXMatrixfDeterminant` · 006fe14a — third-party library
- [-] `c_D3DXMatrixfDeterminant` · 006fe157 — third-party library
- [-] `init_D3DXMatrixMultiply` · 006fe221 · legions 0067dfd1 — third-party library
- [-] `_D3DXMatrixMultiply@12` · 006fe22e · legions 0067dfde — Rust std replaces it
- [-] `c_D3DXMatrixMultiply` · 006fe234 — third-party library
- [-] `init_D3DXMatrixTranspose` · 006fe46a · legions 0067e21a — third-party library
- [-] `_D3DXMatrixTranspose@8` · 006fe477 · legions 0067e227 — Rust std replaces it
- [-] `c_D3DXMatrixTranspose` · 006fe47d — third-party library
- [-] `init_D3DXMatrixMultiplyTranspose` · 006fe4ea · legions 0067e4dd — third-party library
- [-] `c_D3DXMatrixMultiplyTranspose` · 006fe4f7 — third-party library
- [-] `init_D3DXMatrixInverse` · 006fe72d — third-party library
- [-] `_D3DXMatrixInverse@12` · 006fe73a — Rust std replaces it
- [-] `c_D3DXMatrixInverse` · 006fe740 — third-party library
- [-] `init_D3DXMatrixScaling` · 006feacd · legions 0067e87d — third-party library
- [-] `c_D3DXMatrixScaling` · 006feafb — third-party library
- [-] `init_D3DXMatrixTranslation` · 006feb57 · legions 0067e907 — third-party library
- [-] `c_D3DXMatrixTranslation` · 006feb85 — third-party library
- [-] `init_D3DXMatrixRotationX` · 006febe1 · legions 0067e991 — third-party library
- [-] `c_D3DXMatrixRotationX` · 006febfd · legions 0067e9ad — third-party library
- [-] `init_D3DXMatrixRotationY` · 006fec77 · legions 0067ea27 — third-party library
- [-] `c_D3DXMatrixRotationY` · 006fec93 · legions 0067ea43 — third-party library
- [-] `init_D3DXMatrixRotationZ` · 006fed0e · legions 0067eabe — third-party library
- [-] `c_D3DXMatrixRotationZ` · 006fed2a · legions 0067eada — third-party library
- [-] `init_D3DXMatrixRotationAxis` · 006feda6 · legions 0067eb56 — third-party library
- [-] `c_D3DXMatrixRotationAxis` · 006fedc6 · legions 0067eb76 — third-party library
- [-] `init_D3DXMatrixRotationQuaternion` · 006feec8 · legions 0067ec78 — third-party library
- [-] `_D3DXMatrixRotationQuaternion@8` · 006feed5 · legions 0067ec85 — Rust std replaces it
- [-] `c_D3DXMatrixRotationQuaternion` · 006feedb — third-party library
- [-] `init_D3DXMatrixRotationYawPitchRoll` · 006fefbd · legions 0067ed6d — third-party library
- [-] `init_D3DXMatrixTransformation` · 006fefeb · legions 0067ed9b — third-party library
- [-] `init_D3DXMatrixAffineTransformation` · 006feffc · legions 0067edac — third-party library
- [-] `c_D3DXMatrixAffineTransformation` · 006ff023 · legions 0067edd3 — third-party library
- [-] `init_D3DXMatrixShadow` · 006ff109 — third-party library
- [-] `init_D3DXMatrixReflect` · 006ff116 — third-party library
- [-] `init_D3DXQuaternionToAxisAngle` · 006ff123 — third-party library
- [-] `c_D3DXQuaternionToAxisAngle` · 006ff130 — third-party library
- [-] `init_D3DXQuaternionRotationMatrix` · 006ff1cd — third-party library
- [-] `c_D3DXQuaternionRotationMatrix` · 006ff1da — third-party library
- [-] `init_D3DXQuaternionRotationAxis` · 006ff30e · legions 0067f0be — third-party library
- [-] `c_D3DXQuaternionRotationAxis` · 006ff32e · legions 0067f0de — third-party library
- [-] `init_D3DXQuaternionRotationYawPitchRoll` · 006ff38a · legions 0067f13a — third-party library
- [-] `_D3DXQuaternionRotationYawPitchRoll@16` · 006ff3b8 · legions 0067f168 — Rust std replaces it
- [-] `c_D3DXQuaternionRotationYawPitchRoll` · 006ff3be — third-party library
- [-] `init_D3DXQuaternionMultiply` · 006ff497 — third-party library
- [-] `_D3DXQuaternionMultiply@12` · 006ff4a4 — Rust std replaces it
- [-] `c_D3DXQuaternionMultiply` · 006ff4aa — third-party library
- [-] `init_D3DXQuaternionNormalize` · 006ff540 · legions 0067f247 — third-party library
- [-] `init_D3DXQuaternionInverse` · 006ff54d · legions 0067f3b6 — third-party library
- [-] `_D3DXQuaternionInverse@8` · 006ff55a — Rust std replaces it
- [-] `c_D3DXQuaternionInverse` · 006ff560 — third-party library
- [-] `_D3DXQuaternionLn@8` · 006ff614 · legions 0067f47d — Rust std replaces it
- [-] `c_D3DXQuaternionLn` · 006ff61a — third-party library
- [-] `_D3DXQuaternionExp@8` · 006ff698 · legions 0067f501 — Rust std replaces it
- [-] `c_D3DXQuaternionExp` · 006ff69e — third-party library
- [-] `init_D3DXQuaternionSlerp` · 006ff739 · legions 0067f5a2 — third-party library
- [-] `_D3DXQuaternionSlerp@16` · 006ff75d · legions 0067f5c6 — Rust std replaces it
- [x] `c_D3DXQuaternionSlerp` · 006ff763 — ac2-anim dbanimator / ac2-physics anim_pose (slice 3)
- [-] `init_D3DXQuaternionSquad` · 006ff846 · legions 0067f6af — third-party library
- [-] `c_D3DXQuaternionSquad` · 006ff870 · legions 0067f6d9 — third-party library
- [-] `init_D3DXQuaternionSquadSetup` · 006ff8c7 · legions 0067f730 — third-party library
- [-] `init_D3DXQuaternionBaryCentric` · 006ff8d8 — third-party library
- [-] `c_D3DXQuaternionBaryCentric` · 006ff907 · legions 0067f770 — third-party library
- [-] `init_D3DXPlaneNormalize` · 006ff992 · legions 0067f7fb — third-party library
- [-] `_D3DXPlaneNormalize@8` · 006ff99f · legions 0067f808 — Rust std replaces it
- [-] `c_D3DXPlaneNormalize` · 006ff9a5 — third-party library
- [-] `init_D3DXPlaneIntersectLine` · 006ffa49 — third-party library
- [-] `c_D3DXPlaneIntersectLine` · 006ffa56 — third-party library
- [-] `init_D3DXPlaneFromPointNormal` · 006ffaf8 — third-party library
- [-] `c_D3DXPlaneFromPointNormal` · 006ffb05 — third-party library
- [-] `init_D3DXPlaneFromPoints` · 006ffb3d — third-party library
- [-] `c_D3DXPlaneFromPoints` · 006ffb4a — third-party library
- [-] `init_D3DXPlaneTransform` · 006ffbec — third-party library
- [-] `c_D3DXPlaneTransform` · 006ffbf9 — third-party library
- [-] `c_D3DXVec2Normalize` · 006ffc98 · legions 0067fb01 — third-party library
- [-] `c_D3DXVec2Transform` · 006ffd25 · legions 0067fb8e — third-party library
- [-] `c_D3DXVec2TransformCoord` · 006ffd90 · legions 0067fbf9 — third-party library
- [-] `c_D3DXVec2TransformNormal` · 006ffe10 · legions 0067fc79 — third-party library
- [-] `c_D3DXVec3Normalize` · 006ffe4c · legions 0067fcb5 — third-party library
- [-] `c_D3DXVec3TransformCoord` · 006ffeec · legions 0067fd55 — third-party library
- [-] `c_D3DXVec3Project` · 006fffab — third-party library
- [-] `c_D3DXVec3Unproject` · 00700138 — third-party library
- [-] `c_D3DXQuaternionNormalize` · 007002de — third-party library
- [-] `c_D3DXMatrixRotationYawPitchRoll` · 00700397 · legions 00680147 — third-party library
- [-] `c_D3DXMatrixTransformation` · 007003d0 · legions 00680180 — third-party library
- [-] `c_D3DXMatrixShadow` · 00700684 · legions 00680434 — third-party library
- [-] `c_D3DXMatrixReflect` · 0070075e · legions 0068050e — third-party library
- [-] `c_D3DXQuaternionSquadSetup` · 0070081d — third-party library

## `_libraries/d3dx8.lib/d3dxmathsse.obj.c` (0/23)

### (free functions)
- [-] `sse_D3DXVec2Transform` · 0070f550 · legions 0068f300 — third-party library
- [-] `sse_D3DXVec2TransformCoord` · 0070f590 · legions 0068f340 — third-party library
- [-] `sse_D3DXVec2TransformNormal` · 0070f5f0 — third-party library
- [-] `sse2_D3DXVec3TransformCoord` · 0070f630 — third-party library
- [-] `sse2_D3DXMatrixMultiply` · 0070f730 — third-party library
- [-] `sse_D3DXVec3Normalize` · 0070f880 — third-party library
- [-] `sse_D3DXPlaneNormalize` · 0070f950 — third-party library
- [-] `sse2_D3DXMatrixInverse` · 0070fa30 — third-party library
- [-] `sse_D3DXVec4Cross` · 0070fcf0 — third-party library
- [-] `sse_D3DXVec4Normalize` · 0070fda0 — third-party library
- [-] `sse_D3DXVec3CatmullRom` · 0070fe40 — third-party library
- [-] `sse_D3DXVec2CatmullRom` · 0070ff00 — third-party library
- [-] `sse_D3DXVec4CatmullRom` · 0070ff90 — third-party library
- [-] `sse_D3DXVec3Hermite` · 00710040 — third-party library
- [-] `sse_D3DXVec4Hermite` · 00710100 — third-party library
- [-] `sse_D3DXVec2Hermite` · 007101a0 — third-party library
- [-] `sse_D3DXVec3BaryCentric` · 00710230 — third-party library
- [-] `sse_D3DXVec4BaryCentric` · 00710290 — third-party library
- [-] `sse_D3DXVec2BaryCentric` · 007102e0 — third-party library
- [-] `sse_D3DXPlaneIntersectLine` · 00710340 — third-party library
- [-] `sse_D3DXPlaneFromPoints` · 00710400 — third-party library
- [-] `sse_D3DXMatrixShadow` · 007104f0 — third-party library
- [-] `sse_D3DXMatrixReflect` · 00710680 — third-party library

## `_libraries/d3dx8.lib/d3dxmathsse2.obj.c` (0/13)

### (free functions)
- [-] `sse2_D3DXVec3TransformNormal` · 00710af0 — third-party library
- [-] `sse2_D3DXVec3Transform` · 00710bb0 — third-party library
- [-] `sse2_D3DXVec4Transform` · 00710c70 — third-party library
- [-] `sse2_D3DXVec3Normalize` · 00710d30 — third-party library
- [-] `sse2_D3DXPlaneNormalize` · 00710e00 — third-party library
- [-] `sse2_D3DXMatrixMultiplyTranspose` · 00710ee0 — third-party library
- [-] `sse2_D3DXVec4Cross` · 00711040 — third-party library
- [-] `sse2_D3DXMatrixRotationX` · 007110f0 — third-party library
- [-] `sse2_D3DXMatrixRotationY` · 00711170 — third-party library
- [-] `sse2_D3DXMatrixDeterminant` · 00711200 — third-party library
- [-] `sse2_D3DXPlaneFromPoints` · 007112d0 — third-party library
- [-] `sse2_D3DXMatrixShadow` · 007113c0 — third-party library
- [-] `sse2_D3DXMatrixReflect` · 00711550 — third-party library

## `_libraries/d3dx8.lib/d3dxmathx3d.obj.c` (0/2)

### (free functions)
- [-] `get_feature_flags` · 0070ce60 · legions 0068cc10 — third-party library
- [-] `x3d_D3DXInitFastTable` · 0070cf35 · legions 0068cce5 — third-party library

## `_libraries/d3dx8.lib/d3dxmathx86.obj.c` (0/9)

### (free functions)
- [-] `x86_D3DXVec2Transform` · 0070d15b — third-party library
- [-] `x86_D3DXVec2TransformNormal` · 0070d1c4 — third-party library
- [-] `x86_D3DXVec3Transform` · 0070d1f9 — third-party library
- [-] `x86_D3DXVec3TransformNormal` · 0070d284 — third-party library
- [-] `x86_D3DXVec4Transform` · 0070d2e3 — third-party library
- [-] `x86_D3DXVec3Normalize` · 0070d37c · legions 0068d12c — third-party library
- [-] `x86_D3DXMatrixMultiply` · 0070d446 · legions 0068d1f6 — third-party library
- [-] `x86_D3DXMatrixMultiplyTranspose` · 0070d529 · legions 0068d2d9 — third-party library
- [-] `x86_D3DXInitFastTable` · 0070d5b7 · legions 0068d367 — third-party library

## `_libraries/d3dx8.lib/d3dxquatsse.obj.c` (0/5)

### (free functions)
- [-] `sse_D3DXQuaternionMultiply` · 00710760 — third-party library
- [-] `sse_D3DXQuaternionNormalize` · 007107f0 — third-party library
- [-] `sse_D3DXQuaternionInverse` · 00710890 — third-party library
- [-] `sse_D3DXQuaternionRotationMatrix` · 00710900 — third-party library
- [-] `sse_D3DXQuaternionRotationYawPitchRoll` · 00710a20 — third-party library

## `_libraries/d3dx8.lib/d3dxquatsse2.obj.c` (0/2)

### (free functions)
- [-] `sse2_D3DXQuaternionRotationMatrix` · 00711630 — third-party library
- [-] `sse2_D3DXQuaternionRotationYawPitchRoll` · 00711750 — third-party library

## `_libraries/d3dx8.lib/d3dxtrigsse.obj.c` (0/1)

### (free functions)
- [-] `sse_SinCos4` · 00717fb0 · legions 00697d60 — third-party library

## `_libraries/d3dx8.lib/d3dxtrigsse2.obj.c` (0/2)

### (free functions)
- [-] `sse2_sincos` · 007181a0 · legions 00697f50 — third-party library
- [-] `sse2_SinCos4` · 00718290 · legions 00698040 — third-party library

## `_libraries/d3dx8.lib/dxtn.obj.c` (0/18)

### (free functions)
- [-] `Decode565` · 0070d991 · legions 0068d741 — third-party library
- [-] `DecodePremult` · 0070d9dd · legions 0068d78d — third-party library
- [-] `EncodePremult` · 0070da5b · legions 0068d80b — third-party library
- [-] `FUN_0070da8b` · 0070da8b · legions 0068d83b — third-party library
- [-] `Encode565` · 0070daae · legions 0068d85e — third-party library
- [-] `OptimizeAlpha` · 0070dba8 · legions 0068d958 — third-party library
- [-] `OptimizeRGB` · 0070dead · legions 0068dc5d — third-party library
- [-] `EncodeDXT1` · 0070e418 · legions 0068e1c8 — third-party library
- [-] `D3DXDecodeDXT1` · 0070ec1c · legions 0068e9cc — third-party library
- [-] `D3DXDecodeDXT3` · 0070ed5a · legions 0068eb0a — third-party library
- [-] `D3DXDecodeDXT5` · 0070eddd · legions 0068eb8d — third-party library
- [-] `D3DXEncodeDXT1` · 0070ef19 — third-party library
- [-] `D3DXEncodeDXT3` · 0070ef31 · legions 0068ece1 — third-party library
- [-] `D3DXEncodeDXT5` · 0070f045 · legions 0068edf5 — third-party library
- [-] `D3DXDecodeDXT2` · 0070f497 — third-party library
- [-] `D3DXDecodeDXT4` · 0070f4ba — third-party library
- [-] `D3DXEncodeDXT2` · 0070f4dd · legions 0068f28d — third-party library
- [-] `D3DXEncodeDXT4` · 0070f511 · legions 0068f2c1 — third-party library

## `_libraries/d3dx8.lib/math.obj.c` (0/5)

### (free functions)
- [-] `a_atan2` · 007184c0 · legions 00698280 — third-party library
- [-] `a_acos` · 007185c0 · legions 00698380 — third-party library
- [-] `a_log` · 007187c0 — third-party library
- [-] `a_cos` · 00718ba0 · legions 00698960 — third-party library
- [-] `a_sin` · 00718cc0 · legions 00698a80 — third-party library

## `_libraries/d3dx8.lib/ssefasttable.obj.c` (0/1)

### (free functions)
- [-] `sse_D3DXInitFastTable` · 0070cc40 · legions 0068c9f0 — third-party library

## `_libraries/d3dx8.lib/ssefasttable2.obj.c` (0/1)

### (free functions)
- [-] `sse2_D3DXInitFastTable` · 0070cd50 · legions 0068cb00 — third-party library

## `_libraries/d3dx8.lib/x3d_matx.obj.c` (0/22)

### (free functions)
- [-] `x3d_D3DXMatrixMultiply` · 00712d8a — third-party library
- [-] `x3d_D3DXMatrixfDeterminant` · 00712fbc — third-party library
- [-] `x3d_D3DXMatrixTranslation` · 0071308b — third-party library
- [-] `x3d_D3DXMatrixShadow` · 007130db — third-party library
- [-] `x3d_D3DXMatrixReflect` · 007131c6 — third-party library
- [-] `x3d_D3DXMatrixRotationX` · 007132a2 — third-party library
- [-] `x3d_D3DXMatrixRotationY` · 00713305 — third-party library
- [-] `x3d_D3DXMatrixRotationZ` · 00713363 — third-party library
- [-] `x3d_D3DXMatrixScaling` · 007133bc — third-party library
- [-] `x3d_D3DXMatrixTranspose` · 00713403 — third-party library
- [-] `x3d_D3DXMatrixIdentity` · 00713472 · legions 00693222 — third-party library
- [-] `x3d_D3DXMatrixRotationQuaternion` · 007134ab — third-party library
- [-] `x3d_D3DXMatrixRotationQuaternion_K7` · 0071357d — third-party library
- [-] `x3d_D3DXMatrixTransformation` · 00713632 · legions 006933e2 — third-party library
- [-] `x3d_D3DXMatrixRotationYawPitchRoll` · 007147ca · legions 0069457a — third-party library
- [-] `x3d_D3DXMatrixAffineTransformation` · 0071496f — third-party library
- [-] `x3d_D3DXMatrixRotationAxis` · 00714ea3 · legions 00694c53 — third-party library
- [-] `x3d_D3DXMatrixTransformation_K7` · 00714fbc · legions 00694d6c — third-party library
- [-] `x3d_D3DXMatrixInverse_K7` · 00716110 · legions 00695ec0 — third-party library
- [-] `x3d_D3DXMatrixMultiplyTranspose` · 007163f2 — third-party library
- [-] `x3d_D3DXMatrixInverse_inline` · 007167ea · legions 0069659a — third-party library
- [-] `x3d_D3DXMatrixInverse` · 00716b1e — third-party library

## `_libraries/d3dx8.lib/x3d_plane.obj.c` (0/5)

### (free functions)
- [-] `x3d_D3DXPlaneFromPointNormal` · 00717ce3 — third-party library
- [-] `x3d_D3DXPlaneFromPoints` · 00717d26 — third-party library
- [-] `x3d_D3DXPlaneIntersectLine` · 00717e0a — third-party library
- [-] `x3d_D3DXPlaneNormalize` · 00717ec8 · legions 00697c78 — third-party library
- [-] `x3d_D3DXPlaneTransform` · 00717f2c — third-party library

## `_libraries/d3dx8.lib/x3d_quat.obj.c` (0/14)

### (free functions)
- [-] `x3d_D3DXQuaternionMultiply` · 00711820 — third-party library
- [-] `x3d_D3DXQuaternionNormalize` · 007118b9 — third-party library
- [-] `x3d_D3DXQuaternionToAxisAngle` · 00711916 · legions 006916c6 — third-party library
- [-] `x3d_D3DXQuaternionRotationMatrix` · 007119c9 — third-party library
- [-] `x3d_D3DXQuaternionRotationMatrix_K7` · 00711be5 — third-party library
- [-] `x3d_D3DXQuaternionRotationYawPitchRoll` · 00711dfa — third-party library
- [-] `x3d_D3DXQuaternionRotationAxis` · 00711ebf · legions 00691c6f — third-party library
- [-] `x3d_D3DXQuaternionSlerp` · 00711f2b · legions 00691cdb — third-party library
- [-] `x3d_D3DXQuaternionBaryCentric` · 00712062 · legions 00691e12 — third-party library
- [-] `x3d_D3DXQuaternionSquad` · 007120ef · legions 00691e9f — third-party library
- [-] `x3d_D3DXQuaternionInverse` · 00712181 — third-party library
- [-] `x3d_D3DXQuaternionLn` · 007121e5 · legions 00691f95 — third-party library
- [-] `x3d_D3DXQuaternionExp` · 0071228c — third-party library
- [-] `x3d_D3DXQuaternionSquadSetup` · 00712331 · legions 006920e1 — third-party library

## `_libraries/d3dx8.lib/x3d_vec2.obj.c` (0/7)

### (free functions)
- [-] `x3d_D3DXVec2Transform` · 007175f8 — third-party library
- [-] `x3d_D3DXVec2TransformNormal` · 0071764d — third-party library
- [-] `x3d_D3DXVec2TransformCoord` · 00717686 — third-party library
- [-] `x3d_D3DXVec2Hermite` · 007176ea — third-party library
- [-] `x3d_D3DXVec2BaryCentric` · 00717782 — third-party library
- [-] `x3d_D3DXVec2Normalize` · 007177cb — third-party library
- [-] `x3d_D3DXVec2CatmullRom` · 0071780e — third-party library

## `_libraries/d3dx8.lib/x3d_vec3.obj.c` (0/12)

### (free functions)
- [-] `x3d_D3DXVec3Transform` · 00716b3a — third-party library
- [-] `x3d_D3DXVec3TransformNormal` · 00716bab — third-party library
- [-] `x3d_D3DXVec3Normalize` · 00716c12 · legions 006969c2 — third-party library
- [-] `x3d_D3DXVec3BaryCentric` · 00716c71 — third-party library
- [-] `x3d_D3DXVec3Hermite` · 00716ce2 — third-party library
- [-] `x3d_D3DXVec3CatmullRom` · 00716db6 — third-party library
- [-] `x3d_D3DXVec3TransformCoord_inline` · 00716ea0 · legions 00696c50 — third-party library
- [-] `x3d_D3DXMatrixMultiply_inline` · 00716f2b · legions 00696cdb — third-party library
- [-] `x3d_D3DXVec3TransformCoord` · 007170f2 · legions 00696ea7 — third-party library
- [-] `x3d_D3DXVec3Project` · 007170f7 — third-party library
- [-] `x3d_D3DXVec3Unproject` · 0071727f — third-party library
- [-] `x3d_D3DXVec3Unproject_K7` · 007174a5 · legions 00697255 — third-party library

## `_libraries/d3dx8.lib/x3d_vec4.obj.c` (0/6)

### (free functions)
- [-] `x3d_D3DXVec4Transform` · 007178b9 — third-party library
- [-] `x3d_D3DXVec4Normalize` · 0071793b — third-party library
- [-] `x3d_D3DXVec4Cross` · 0071799a — third-party library
- [-] `x3d_D3DXVec4BaryCentric` · 00717acc — third-party library
- [-] `x3d_D3DXVec4Hermite` · 00717b3d — third-party library
- [-] `x3d_D3DXVec4CatmullRom` · 00717c05 — third-party library

## `_libraries/dxerr8.lib/dxerr8.obj.c` (0/1)

### (free functions)
- [-] `_DXGetErrorString8A@4` · 007190b8 · legions 00698e78 — Rust std replaces it

## `_libraries/ijl15.lib/ijl15.dll.c` (0/4)

### (free functions)
- [-] `ijlFree` · 0071ba46 — third-party library
- [-] `ijlRead` · 0071ba4c — third-party library
- [-] `ijlInit` · 0071ba52 — third-party library
- [-] `ijlWrite` · 0071ba58 — third-party library

## `_libraries/imm32.lib/IMM32.dll.c` (0/17)

### (free functions)
- [-] `_ImmAssociateContext@8` · 0071ba5e — Rust std replaces it
- [-] `_ImmReleaseContext@8` · 0071ba64 — Rust std replaces it
- [-] `_ImmGetContext@4` · 0071ba6a — Rust std replaces it
- [-] `_ImmGetCompositionStringA@16` · 0071ba70 — Rust std replaces it
- [-] `_ImmGetCandidateListW@16` · 0073dd60 — Rust std replaces it
- [-] `_ImmGetCompositionStringW@16` · 0073dd66 — Rust std replaces it
- [-] `_ImmGetIMEFileNameA@12` · 0073dd6c — Rust std replaces it
- [-] `_ImmGetVirtualKey@4` · 0073dd72 — Rust std replaces it
- [-] `_ImmNotifyIME@16` · 0073dd78 — Rust std replaces it
- [-] `_ImmGetDefaultIMEWnd@4` · 0073dd7e — Rust std replaces it
- [-] `_ImmGetOpenStatus@4` · 0073dd84 — Rust std replaces it
- [-] `_ImmGetConversionStatus@12` · 0073dd8a — Rust std replaces it
- [-] `_ImmIsIME@4` · 0073dd90 — Rust std replaces it
- [-] `_ImmGetCandidateListA@16` · 0073dd96 — Rust std replaces it
- [-] `_ImmSetConversionStatus@12` · 0073dd9c — Rust std replaces it
- [-] `_ImmSetOpenStatus@8` · 0073dda2 — Rust std replaces it
- [-] `_ImmSimulateHotKey@8` · 0073dda8 — Rust std replaces it

## `_libraries/intel_icc/BlendVert.cpp.c` (0/16)

### (free functions)
- [-] `BlendVerticesSingle` · 00729340 · legions 006a7600 — third-party library
- [-] `BlendVertices` · 00729430 — third-party library
- [-] `BlendVertices` · 00729460 — third-party library
- [-] `BlendVertices` · 00729630 — third-party library
- [-] `BlendLitVerticesSingle` · 007297c0 · legions 006a7a80 — third-party library
- [-] `BlendLitVertices` · 00729970 — third-party library
- [-] `BlendLitVertices` · 007299a0 — third-party library
- [-] `BlendLitVertices` · 00729cf0 — third-party library
- [-] `BlendVerticesSingle_UpdateBounds` · 00729fc0 · legions 006a8280 — third-party library
- [-] `BlendVertices_UpdateBounds` · 0072a140 — third-party library
- [-] `BlendVertices_UpdateBounds` · 0072a170 — third-party library
- [-] `BlendVertices_UpdateBounds` · 0072a3a0 — third-party library
- [-] `BlendLitVerticesSingle_UpdateBounds` · 0072a590 · legions 006a8850 — third-party library
- [-] `BlendLitVertices_UpdateBounds` · 0072a7c0 — third-party library
- [-] `BlendLitVertices_UpdateBounds` · 0072a7f0 — third-party library
- [-] `BlendLitVertices_UpdateBounds` · 0072aba0 — third-party library

## `_libraries/interp_debugger_lexer.lib/debug_grammar.obj.c` (0/1)

### (free functions)
- [-] `yyparse` · 0073ca50 · legions 006b8870 — third-party library

## `_libraries/libirc.lib/cpu_disp.obj.c` (0/2)

### (free functions)
- [-] `FUN_0073dc00` · 0073dc00 · legions 006b9a20 — third-party library
- [-] `___intel_cpu_indicator_init` · 0073dc60 · legions 006b9a80 — Rust std replaces it

## `_libraries/metalanguage.lib/metalanguage_grammar.obj.c` (0/1)

### (free functions)
- [-] `stringmeta_parse` · 00659350 · legions 00607cc0 — third-party library

## `_libraries/msvc_crt/atonexit.c.c` (0/2)

### (free functions)
- [-] `_onexit` · 0071ee30 · legions 0069f750 — Rust std replaces it
- [-] `atexit` · 0071ee56 · legions 0069f776 — third-party library

## `_libraries/msvc_crt/crtexe.c.c` (0/1)

### (free functions)
- [-] `WinMainCRTStartup` · 0071ec48 · legions 0069f568 — third-party library

## `_libraries/msvc_crt/intel/chkstk.asm.c` (0/1)

### (free functions)
- [-] `_chkstk` · 0071ef80 · legions 0069f8a0 — Rust std replaces it

## `_libraries/msvc_crt/intel/fp8.c.c` (0/1)

### (free functions)
- [-] `_setdefaultprecision` · 0071f548 · legions 0069fe48 — Rust std replaces it

## `_libraries/msvc_crt/intel/lldiv.asm.c` (0/1)

### (free functions)
- [-] `_alldiv` · 0071efc0 · legions 0069f9b0 — Rust std replaces it

## `_libraries/msvc_crt/intel/llmul.asm.c` (0/1)

### (free functions)
- [-] `_allmul` · 0071f120 · legions 0069f970 — Rust std replaces it

## `_libraries/msvc_crt/intel/llrem.asm.c` (0/1)

### (free functions)
- [-] `_allrem` · 0071f280 · legions 0069fb80 — Rust std replaces it

## `_libraries/msvc_crt/intel/llshl.asm.c` (0/1)

### (free functions)
- [-] `_allshl` · 0071f0f0 · legions 0069f930 — Rust std replaces it

## `_libraries/msvc_crt/intel/llshr.asm.c` (0/1)

### (free functions)
- [-] `_allshr` · 0071f080 — Rust std replaces it

## `_libraries/msvc_crt/intel/ulldiv.asm.c` (0/1)

### (free functions)
- [-] `_aulldiv` · 0071f210 · legions 0069fb10 — Rust std replaces it

## `_libraries/msvc_crt/intel/ulldvrm.asm.c` (0/1)

### (free functions)
- [-] `_aulldvrm` · 0071f170 · legions 0069fa70 — Rust std replaces it

## `_libraries/msvc_crt/intel/ullrem.asm.c` (0/1)

### (free functions)
- [-] `_aullrem` · 0071eef0 · legions 0069f810 — Rust std replaces it

## `_libraries/msvc_crt/intel/ullshr.asm.c` (0/1)

### (free functions)
- [-] `_aullshr` · 0071f0b0 · legions 0069f8f0 — Rust std replaces it

## `_libraries/msvc_crt/merr.c.c` (0/1)

### (free functions)
- [-] `_setargv` · 0071f55a · legions 0069fe5a — Rust std replaces it

## `_libraries/msvc_crt/newaop.cpp.c` (0/1)

### (free functions)
- [-] `operator_new[]` · 0071ec00 · legions 0069f520 — third-party library

## `_libraries/msvc_crt/newop.cpp.c` (0/1)

### (free functions)
- [-] `operator_new` · 0071ec05 · legions 0069f525 — third-party library

## `_libraries/version.lib/VERSION.dll.c` (0/3)

### (free functions)
- [-] `_GetFileVersionInfoA@16` · 006f5ef0 — Rust std replaces it
- [-] `_GetFileVersionInfoSizeA@8` · 006f5ef6 — Rust std replaces it
- [-] `_VerQueryValueA@16` · 006f5efc — Rust std replaces it

## `_libraries/zlibstat.lib/adler32.obj.c` (0/1)

### (free functions)
- [-] `_adler32@12` · 006f7460 · legions 00677390 — Rust std replaces it

## `_libraries/zlibstat.lib/infblock.obj.c` (0/4)

### (free functions)
- [-] `inflate_blocks_reset` · 006f65f0 · legions 00676520 — third-party library
- [-] `inflate_blocks_new` · 006f6670 · legions 006765a0 — third-party library
- [-] `inflate_blocks` · 006f6720 · legions 00676650 — third-party library
- [-] `inflate_blocks_free` · 006f7410 · legions 00677340 — third-party library

## `_libraries/zlibstat.lib/infcodes.obj.c` (0/3)

### (free functions)
- [-] `inflate_codes_new` · 006f75c0 · legions 006774f0 — third-party library
- [-] `inflate_codes` · 006f7600 · legions 00677530 — third-party library
- [-] `inflate_codes_free` · 006f7db0 · legions 00677ce0 — third-party library

## `_libraries/zlibstat.lib/inffast.obj.c` (0/1)

### (free functions)
- [-] `inflate_fast` · 006f8620 · legions 00678550 — third-party library

## `_libraries/zlibstat.lib/inflate.obj.c` (0/5)

### (free functions)
- [-] `inflateReset` · 006f5fb0 · legions 00675ee0 — third-party library
- [-] `inflateEnd` · 006f6000 · legions 00675f30 — third-party library
- [-] `inflateInit2_` · 006f6050 · legions 00675f80 — third-party library
- [-] `inflateInit_` · 006f6170 · legions 006760a0 — third-party library
- [-] `inflate` · 006f6190 · legions 006760c0 — third-party library

## `_libraries/zlibstat.lib/inftrees.obj.c` (0/4)

### (free functions)
- [-] `inflate_trees_bits` · 006f7dd0 · legions 00677d00 — third-party library
- [-] `FUN_006f7e80` · 006f7e80 · legions 00677db0 — third-party library
- [-] `inflate_trees_dynamic` · 006f8330 · legions 00678260 — third-party library
- [-] `inflate_trees_fixed` · 006f84c0 · legions 006783f0 — third-party library

## `_libraries/zlibstat.lib/infutil.obj.c` (0/1)

### (free functions)
- [-] `inflate_flush` · 006f84f0 · legions 00678420 — third-party library

## `_libraries/zlibstat.lib/uncompr.obj.c` (0/1)

### (free functions)
- [-] `uncompress` · 006f5f10 · legions 00675e40 — third-party library

## `_libraries/zlibstat.lib/zutil.obj.c` (0/1)

### (free functions)
- [-] `zcalloc` · 006f7590 — third-party library

## `ac2/client/src/main/climain.cpp.c` (0/5)

### PStringBase<char>
- [-] `PStringBase<char>::PStringBase<char>` · 004010d0 · legions 00401080 — Rust std replaces it
- [-] `PStringBase<char>::allocate_ref_buffer` · 00401110 · legions 004010e0 — Rust std replaces it
- [-] `PStringBase<char>::vsprintf` · 00401190 · legions 00401160 — Rust std replaces it
- [-] `PStringBase<char>::PStringBase<char>` · 00401290 · legions 00401260 — Rust std replaces it
- [-] `PStringBase<char>::sprintf` · 004012f0 · legions 004012c0 — Rust std replaces it

## `ac2/client/src/main/gmclient.cpp.c` (3/28)

### PStringBase<char>
- [-] `PStringBase<char>::cmp` · 00401f20 · legions 00401ec0 — Rust std replaces it
- [x] `PStringBase<char>::crypto_nibble_munge` · 00401f90 · legions 00401f30 — ac2-core pstring.rs: the DAT string descramble (ac2-ui, ac2-db use it)
- [x] `PStringBase<char>::crypto_uint32_munge` · 00401fc0 · legions 00401f60 — ac2-core pstring.rs: the DAT string descramble (ac2-ui, ac2-db use it)
- [-] `PStringBase<char>::clear` · 00402300 · legions 004022a0 — Rust std replaces it
- [-] `PStringBase<char>::eq` · 00402340 · legions 004022e0 — Rust std replaces it

### PStringBase<unsigned_short>
- [-] `PStringBase<unsigned_short>::PStringBase<unsigned_short>` · 00402360 · legions 00418be0 — Rust std replaces it
- [-] `PStringBase<unsigned_short>::allocate_ref_buffer` · 004023d0 · legions 00402350 — Rust std replaces it

### PStringBase<char>
- [-] `PStringBase<char>::set` · 00402450 · legions 004023d0 — Rust std replaces it
- [-] `PStringBase<char>::break_reference` · 004024f0 · legions 00402470 — Rust std replaces it

### PStringBase<unsigned_short>
- [-] `PStringBase<unsigned_short>::vsprintf` · 00402550 · legions 004024d0 — Rust std replaces it
- [-] `PStringBase<unsigned_short>::PStringBase<unsigned_short>` · 00402e70 · legions 00402de0 — Rust std replaces it
- [-] `PStringBase<unsigned_short>::sprintf` · 00402ed0 · legions 00402e40 — Rust std replaces it

### PStringBase<char>
- [-] `PStringBase<char>::append_n_chars` · 00402fa0 · legions 00402f10 — Rust std replaces it
- [x] `PStringBase<char>::encrypt` · 00403030 · legions 00402fa0 — Rust std replaces it
- [-] `PStringBase<char>::to_wpstring` · 00403140 · legions 004030b0 — Rust std replaces it
- [-] `PStringBase<char>::trim` · 004041b0 · legions 00404100 — Rust std replaces it

### List<TurbineUserStatus>
- [-] `List<TurbineUserStatus>::flush` · 00405070 · legions 00404e10 — Rust std replaces it
- [-] `List<TurbineUserStatus>::pop_front` · 004050b0 · legions 00404e50 — Rust std replaces it

### List<gmSceneInfo>
- [-] `List<gmSceneInfo>::flush` · 00405140 · legions 00404f00 — Rust std replaces it
- [-] `List<gmSceneInfo>::pop_front` · 00405180 · legions 00404f40 — Rust std replaces it

### List<UIShopSale>
- [-] `List<UIShopSale>::pop_head` · 00405380 · legions 00405120 — Rust std replaces it
- [-] `List<UIShopSale>::pop_front` · 004053d0 · legions 00405170 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,unsigned_long>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,unsigned_long>_*>::resize_internal` · 004056f0 · legions 00409f10 — Rust std replaces it

### _STL
- [-] `_STL::lower_bound` · 004057b0 · legions 00405490 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,FXNode>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,FXNode>_*>::delete_contents` · 004057f0 · legions 004054d0 — Rust std replaces it

### HashTable<unsigned_long,FXNode>
- [-] `HashTable<unsigned_long,FXNode>::resize` · 00405ac0 · legions 004057a0 — Rust std replaces it

### List<UIShop>
- [-] `List<UIShop>::flush` · 00405b60 · legions 00405840 — Rust std replaces it
- [-] `List<UIShop>::pop_front` · 00405ba0 · legions 00405880 — Rust std replaces it

## `ac2/client/src/ui/gameplay/gm3dtextui.cpp.c` (0/33)

### _STL::_Deque_iterator_base<BBox>
- [-] `_STL::_Deque_iterator_base<BBox>::_M_advance` · 00426c80 · legions 00426a40 — Rust std replaces it

### SmartArray<Box2D>
- [-] `SmartArray<Box2D>::grow` · 00426fb0 · legions 00426d70 — Rust std replaces it

### _STL
- [-] `_STL::__destroy_aux` · 00427150 · legions 00426fb0 — Rust std replaces it
- [-] `_STL::__copy_backward` · 00427190 · legions 00426ff0 — Rust std replaces it
- [-] `_STL::__copy` · 00427260 · legions 004270c0 — Rust std replaces it
- [-] `_STL::uninitialized_copy` · 00427340 · legions 004271a0 — Rust std replaces it
- [-] `_STL::__uninitialized_copy` · 004273c0 · legions 00426f10 — Rust std replaces it
- [-] `_STL::__copy_aux` · 004274c0 · legions 00427300 — Rust std replaces it
- [-] `_STL::__copy_backward_aux` · 00427540 · legions 00427280 — Rust std replaces it
- [-] `_STL::__uninitialized_copy_copy` · 004275c0 · legions 00427380 — Rust std replaces it
- [-] `_STL::__uninitialized_copy_copy` · 004276b0 · legions 00427470 — Rust std replaces it
- [-] `_STL::_Destroy` · 004277a0 · legions 00427560 — Rust std replaces it

### _STL::_Deque_base<BBox,_STL::allocator<BBox>_>
- [-] `_STL::_Deque_base<BBox,_STL::allocator<BBox>_>::_M_create_nodes` · 004277f0 · legions 004275b0 — Rust std replaces it
- [-] `_STL::_Deque_base<BBox,_STL::allocator<BBox>_>::_M_destroy_nodes` · 00427850 · legions 00427610 — Rust std replaces it

### _STL::deque<BBox,_STL::allocator<BBox>_>
- [-] `_STL::deque<BBox,_STL::allocator<BBox>_>::_M_reallocate_map` · 004278a0 · legions 00427660 — Rust std replaces it

### _STL
- [-] `_STL::copy` · 00427a00 · legions 004277c0 — Rust std replaces it
- [-] `_STL::copy_backward` · 00427a80 · legions 00427840 — Rust std replaces it

### _STL::_Deque_base<BBox,_STL::allocator<BBox>_>
- [-] `_STL::_Deque_base<BBox,_STL::allocator<BBox>_>::_M_initialize_map` · 00427b00 · legions 004278c0 — Rust std replaces it

### _STL::deque<BBox,_STL::allocator<BBox>_>
- [-] `_STL::deque<BBox,_STL::allocator<BBox>_>::clear` · 00427bb0 · legions 00427970 — Rust std replaces it
- [-] `_STL::deque<BBox,_STL::allocator<BBox>_>::erase` · 00427cc0 · legions 00427a80 — Rust std replaces it
- [-] `_STL::deque<BBox,_STL::allocator<BBox>_>::_M_new_elements_at_front` · 00427fd0 · legions 00427d90 — Rust std replaces it
- [-] `_STL::deque<BBox,_STL::allocator<BBox>_>::_M_new_elements_at_back` · 00428050 · legions 00427e10 — Rust std replaces it
- [-] `_STL::deque<BBox,_STL::allocator<BBox>_>::_M_push_back_aux_v` · 00428190 · legions 00427f50 — Rust std replaces it
- [-] `_STL::deque<BBox,_STL::allocator<BBox>_>::_M_reserve_elements_at_back` · 00428260 · legions 00428020 — Rust std replaces it
- [-] `_STL::deque<BBox,_STL::allocator<BBox>_>::_M_insert_aux` · 004282b0 · legions 00428070 — Rust std replaces it
- [-] `_STL::deque<BBox,_STL::allocator<BBox>_>::deque<BBox,_STL::allocator<BBox>_>` · 00428a20 · legions 004287e0 — Rust std replaces it

### SmartArray<gm3DTextUI::DamageTextInfo>
- [-] `SmartArray<gm3DTextUI::DamageTextInfo>::Reset` · 00428b10 · legions 004288d0 — Rust std replaces it

### SmartArray<gm3DTextUI::NameTextInfo>
- [-] `SmartArray<gm3DTextUI::NameTextInfo>::Reset` · 00428b70 · legions 00428930 — Rust std replaces it

### _STL::deque<BBox,_STL::allocator<BBox>_>
- [-] `_STL::deque<BBox,_STL::allocator<BBox>_>::insert` · 00428bd0 · legions 00428990 — Rust std replaces it
- [-] `_STL::deque<BBox,_STL::allocator<BBox>_>::_M_insert_dispatch` · 00429130 · legions 00428ef0 — Rust std replaces it
- [-] `_STL::deque<BBox,_STL::allocator<BBox>_>::insert` · 004292b0 · legions 00429070 — Rust std replaces it

### SmartArray<gm3DTextUI::DamageTextInfo>
- [-] `SmartArray<gm3DTextUI::DamageTextInfo>::grow` · 0042a2d0 · legions 0042a090 — Rust std replaces it

### SmartArray<gm3DTextUI::NameTextInfo>
- [-] `SmartArray<gm3DTextUI::NameTextInfo>::grow` · 0042a3c0 · legions 0042a180 — Rust std replaces it

## `ac2/client/src/ui/gameplay/gmbookui.cpp.c` (0/7)

### PStringBaseIter_Common<unsigned_short>
- [-] `PStringBaseIter_Common<unsigned_short>::Next` · 0041ee10 · legions 0041ec30 — Rust std replaces it

### PStringBaseIter<unsigned_short>
- [-] `PStringBaseIter<unsigned_short>::IndexToConstPointer` · 0041f090 — Rust std replaces it
- [-] `PStringBaseIter<unsigned_short>::IndexToChar` · 0041f0b0 · legions 0044bf30 — Rust std replaces it

### PStringBase<unsigned_short>
- [-] `PStringBase<unsigned_short>::trim` · 0041f100 · legions 0041eee0 — Rust std replaces it

### PStringBaseIter<unsigned_short>
- [-] `PStringBaseIter<unsigned_short>::Split` · 0041f240 · legions 0044c310 — Rust std replaces it
- [-] `PStringBaseIter<unsigned_short>::Truncate` · 0041f3f0 · legions 0041f130 — Rust std replaces it

### List<PStringBase<unsigned_short>_>
- [-] `List<PStringBase<unsigned_short>_>::flush` · 0041f730 · legions 0041f510 — Rust std replaces it

## `ac2/client/src/ui/gameplay/gmgameplayui.cpp.c` (2/21)

### List<unsigned_long>
- [-] `List<unsigned_long>::empty` · 00407b00 · legions 00407830 — Rust std replaces it

### List<gmSceneInfo>
- [-] `List<gmSceneInfo>::push_back` · 00407ee0 · legions 00407c10 — Rust std replaces it

### List<unsigned_long>
- [-] `List<unsigned_long>::remove` · 00407fb0 · legions 005a5710 — Rust std replaces it

### List<GameplayChildInfo>
- [-] `List<GameplayChildInfo>::flush` · 00408800 · legions 0058b620 — Rust std replaces it

### List<gmSceneInfo>
- [-] `List<gmSceneInfo>::List<gmSceneInfo>` · 00408c10 · legions 00408950 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,unsigned_long>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,unsigned_long>_*>::get_bucket_size_pointer` · 004095a0 · legions 004092e0 — Rust std replaces it

### IntrusiveHashTable<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>_*>
- [-] `IntrusiveHashTable<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>_*>::add_for_unpacking` · 004097b0 · legions 004094f0 — Rust std replaces it

### PStringBase<unsigned_short>
- [-] `PStringBase<unsigned_short>::compute_hash` · 00409840 · legions 00409580 — Rust std replaces it

### HashTableData<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>
- [-] `HashTableData<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>::HashTableData<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>` · 00409890 · legions 004095d0 — Rust std replaces it

### PStringBase<unsigned_short>
- [x] `PStringBase<unsigned_short>::crypto_nibble_munge` · 00409a80 · legions 004097f0 — ac2-core pstring.rs: the DAT string descramble (ac2-ui, ac2-db use it)
- [x] `PStringBase<unsigned_short>::crypto_uint32_munge` · 00409ab0 · legions 00409820 — ac2-core pstring.rs: the DAT string descramble (ac2-ui, ac2-db use it)
- [-] `PStringBase<unsigned_short>::break_reference` · 00409b80 · legions 004098f0 — Rust std replaces it
- [-] `PStringBase<unsigned_short>::clear` · 00409cc0 · legions 00409a30 — Rust std replaces it

### HashTable<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>
- [-] `HashTable<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>::resize` · 00409d00 · legions 00409a70 — Rust std replaces it

### IntrusiveHashTable<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>_*>
- [-] `IntrusiveHashTable<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>_*>::resize_internal` · 00409d20 · legions 00409a90 — Rust std replaces it

### HashTable<unsigned_long,unsigned_long>
- [-] `HashTable<unsigned_long,unsigned_long>::resize` · 00409de0 · legions 00409b50 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,unsigned_long>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,unsigned_long>_*>::add_internal` · 00409fd0 · legions 004de1f0 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PHashTable<unsigned_long,UILocationData>_>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PHashTable<unsigned_long,UILocationData>_>_*>::delete_contents` · 0040a060 · legions 00409bd0 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,FXNode>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,FXNode>_*>::get_bucket_size_pointer` · 0040a3d0 · legions 00409620 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,unsigned_long>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,unsigned_long>_*>::init` · 0040a400 · legions 004b6700 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,UILocationData>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,UILocationData>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,UILocationData>_*>` · 0040a470 · legions 0040a2f0 — Rust std replaces it

## `ac2/client/src/ui/gameplay/gmingredientui.cpp.c` (0/8)

### PStringBase<unsigned_short>
- [-] `PStringBase<unsigned_short>::to_int32` · 00415ca0 · legions 00415a90 — Rust std replaces it
- [-] `PStringBase<unsigned_short>::append_n_chars` · 004161f0 · legions 00415fe0 — Rust std replaces it

### HashTable<unsigned_long,InstanceID>
- [-] `HashTable<unsigned_long,InstanceID>::erase` · 004162e0 · legions 005a6250 — Rust std replaces it

### PStringBase<unsigned_short>
- [-] `PStringBase<unsigned_short>::append_int32` · 00416360 · legions 00416160 — Rust std replaces it
- [-] `PStringBase<unsigned_short>::append_uint32` · 004163d0 · legions 004161d0 — Rust std replaces it

### List<IngredientAdaptor>
- [-] `List<IngredientAdaptor>::flush` · 00416440 · legions 00416240 — Rust std replaces it

### HashTable<InstanceID,unsigned_long>
- [-] `HashTable<InstanceID,unsigned_long>::set` · 004165b0 · legions 00416440 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,InstanceID>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,InstanceID>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,InstanceID>_*>` · 00416c90 · legions 00416a50 — Rust std replaces it

## `ac2/client/src/ui/gameplay/gmpetui.cpp.c` (0/18)

### _STL
- [-] `_STL::__stl_delete` · 004188b0 — Rust std replaces it

### _STL::_STLP_mutex_base
- [-] `_STL::_STLP_mutex_base::_M_release_lock` · 004188c0 · legions 00418680 — Rust std replaces it

### PStringBase<unsigned_short>
- [-] `PStringBase<unsigned_short>::find_substring` · 00418980 · legions 00418740 — Rust std replaces it

### _STL::__node_alloc<1,0>
- [-] `_STL::__node_alloc<1,0>::_S_chunk_alloc` · 004189d0 · legions 00418790 — Rust std replaces it

### _STL::_STLP_mutex_spin<0>
- [-] `_STL::_STLP_mutex_spin<0>::_M_do_lock` · 00418b40 · legions 00418900 — Rust std replaces it

### _STL::__node_alloc<1,0>
- [-] `_STL::__node_alloc<1,0>::_S_refill` · 00418c30 · legions 004189f0 — Rust std replaces it

### _STL::_STLP_mutex_base
- [-] `_STL::_STLP_mutex_base::_M_acquire_lock` · 00418ca0 · legions 00418a60 — Rust std replaces it

### HashTableData<InstanceID,PetInfo>
- [-] `HashTableData<InstanceID,PetInfo>::HashTableData<InstanceID,PetInfo>` · 00418e30 · legions 00418c70 — Rust std replaces it

### _STL::_Node_Alloc_Lock<1,0>
- [-] `_STL::_Node_Alloc_Lock<1,0>::_Node_Alloc_Lock<1,0>` · 00418e90 · legions 00418cd0 — Rust std replaces it

### IntrusiveHashTable<InstanceID,HashTableData<InstanceID,PetInfo>_*>
- [-] `IntrusiveHashTable<InstanceID,HashTableData<InstanceID,PetInfo>_*>::delete_contents` · 00418eb0 · legions 00418cf0 — Rust std replaces it

### PStringBase<unsigned_short>
- [-] `PStringBase<unsigned_short>::substring` · 00418f60 · legions 00418da0 — Rust std replaces it

### _STL::__node_alloc<1,0>
- [-] `_STL::__node_alloc<1,0>::_M_allocate` · 00419010 · legions 00418e50 — Rust std replaces it
- [-] `_STL::__node_alloc<1,0>::_M_deallocate` · 00419060 · legions 00418ea0 — Rust std replaces it

### HashTable<InstanceID,PetInfo>
- [-] `HashTable<InstanceID,PetInfo>::find` · 004190a0 · legions 00418ee0 — Rust std replaces it

### _STL::__node_alloc<1,0>
- [-] `_STL::__node_alloc<1,0>::deallocate` · 004190e0 · legions 00418f20 — Rust std replaces it

### HashTable<InstanceID,PetInfo>
- [-] `HashTable<InstanceID,PetInfo>::set` · 00419360 · legions 004191a0 — Rust std replaces it

### IntrusiveHashTable<InstanceID,HashTableData<InstanceID,PetInfo>_*>
- [-] `IntrusiveHashTable<InstanceID,HashTableData<InstanceID,PetInfo>_*>::IntrusiveHashTable<InstanceID,HashTableData<InstanceID,PetInfo>_*>` · 00419920 · legions 00419760 — Rust std replaces it

### PStringBase<unsigned_short>
- [-] `PStringBase<unsigned_short>::replace` · 0041a1e0 · legions 0041a020 — Rust std replaces it

## `ac2/client/src/ui/gameplay/gmradarui.cpp.c` (0/3)

### SmartArray<RadarInfo>
- [-] `SmartArray<RadarInfo>::grow` · 004255f0 · legions 004253e0 — Rust std replaces it
- [-] `SmartArray<RadarInfo>::add_unique` · 00425fe0 · legions 00425dd0 — Rust std replaces it

### SmartArray<GrooveInfo>
- [-] `SmartArray<GrooveInfo>::add_unique` · 00426020 · legions 00425e10 — Rust std replaces it

## `ac2/client/src/ui/gameplay/gmsecuretradeui.cpp.c` (0/1)

### List<ItemDisplay_*>
- [-] `List<ItemDisplay_*>::remove` · 00421510 · legions 00421340 — Rust std replaces it

## `ac2/client/src/ui/gameplay/gmshopconsignmentui.cpp.c` (0/3)

### HashTable<unsigned_long,InstanceID>
- [-] `HashTable<unsigned_long,InstanceID>::contains` · 0041bb20 · legions 004160d0 — Rust std replaces it

### List<UIShopSale>
- [-] `List<UIShopSale>::flush` · 0041bb60 · legions 0041b980 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,UIElement_*>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,UIElement_*>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,UIElement_*>_*>` · 0041c770 · legions 0041c590 — Rust std replaces it

## `ac2/client/src/ui/gameplay/gmshopui.cpp.c` (0/1)

### IntrusiveHashTable<InstanceID,HashTableData<InstanceID,UIElement_*>_*>
- [-] `IntrusiveHashTable<InstanceID,HashTableData<InstanceID,UIElement_*>_*>::IntrusiveHashTable<InstanceID,HashTableData<InstanceID,UIElement_*>_*>` · 0041e490 · legions 0041e2b0 — Rust std replaces it

## `ac2/client/src/ui/gameplay/gmstoryui.cpp.c` (0/2)

### SmartArray<gmKeyframe>
- [-] `SmartArray<gmKeyframe>::grow` · 0042c0b0 · legions 0042be70 — Rust std replaces it

### _STL::_Rb_tree<QualifiedDataID,QualifiedDataID,_STL::_Identity<QualifiedDataID>,_STL::less<QualifiedDataID>,_STL::allocator<QualifiedDataID>_>
- [-] `_STL::_Rb_tree<QualifiedDataID,QualifiedDataID,_STL::_Identity<QualifiedDataID>,_STL::less<QualifiedDataID>,_STL::allocator<QualifiedDataID>_>::_M_erase` · 0042c190 · legions 0042bf50 — Rust std replaces it

## `ac2/client/src/ui/gameplay/helper/chatwindow.cpp.c` (0/6)

### SmartArray<ChatWindow::ChatInfo>
- [-] `SmartArray<ChatWindow::ChatInfo>::Reset` · 0044b430 · legions 0044bed0 — Rust std replaces it
- [-] `SmartArray<ChatWindow::ChatInfo>::grow` · 0044b490 · legions 0044bf70 — Rust std replaces it

### HashTable<unsigned_long,ChatWindow::ChatMessageInfo>
- [-] `HashTable<unsigned_long,ChatWindow::ChatMessageInfo>::find` · 0044b930 — Rust std replaces it

### PStringBaseIter<unsigned_short>
- [-] `PStringBaseIter<unsigned_short>::Split` · 0044b990 — Rust std replaces it

### HashTable<unsigned_long,ChatWindow::ChatMessageInfo>
- [-] `HashTable<unsigned_long,ChatWindow::ChatMessageInfo>::set` · 0044bb90 · legions 0044c8b0 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,ChatWindow::ChatMessageInfo>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,ChatWindow::ChatMessageInfo>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,ChatWindow::ChatMessageInfo>_*>` · 0044c350 · legions 0044d070 — Rust std replaces it

## `ac2/client/src/ui/gameplay/helper/fellowshiphelper.cpp.c` (0/5)

### HashTable<InstanceID,ICPhysicsObj_*>
- [-] `HashTable<InstanceID,ICPhysicsObj_*>::add` · 0042f400 · legions 0040d790 — Rust std replaces it
- [-] `HashTable<InstanceID,ICPhysicsObj_*>::lookup` · 0042f460 · legions 0042f320 — Rust std replaces it

### IntrusiveHashTable<InstanceID,HashTableData<InstanceID,float>_*>
- [-] `IntrusiveHashTable<InstanceID,HashTableData<InstanceID,float>_*>::IntrusiveHashTable<InstanceID,HashTableData<InstanceID,float>_*>` · 0042f560 · legions 0042f420 — Rust std replaces it

### IntrusiveHashTable<InstanceID,HashTableData<InstanceID,FellowInfo_*>_*>
- [-] `IntrusiveHashTable<InstanceID,HashTableData<InstanceID,FellowInfo_*>_*>::IntrusiveHashTable<InstanceID,HashTableData<InstanceID,FellowInfo_*>_*>` · 0042f850 · legions 0042f710 — Rust std replaces it

### IntrusiveHashTable<InstanceID,HashTableData<InstanceID,float>_*>
- [-] `IntrusiveHashTable<InstanceID,HashTableData<InstanceID,float>_*>::resize_internal` · 0042fba0 · legions 0063fb20 — Rust std replaces it

## `ac2/client/src/ui/gameplay/helper/optionchoice.cpp.c` (0/2)

### IntrusiveHashTable<long,HashTableData<long,StringInfo>_*>
- [-] `IntrusiveHashTable<long,HashTableData<long,StringInfo>_*>::delete_contents` · 0044e420 · legions 00505170 — Rust std replaces it
- [-] `IntrusiveHashTable<long,HashTableData<long,StringInfo>_*>::IntrusiveHashTable<long,HashTableData<long,StringInfo>_*>` · 0044e500 · legions 0044f640 — Rust std replaces it

## `ac2/client/src/ui/gameplay/statusbar/gmeffectsui.cpp.c` (0/3)

### List<EffectUINode>
- [-] `List<EffectUINode>::pop_front` · 00438dc0 · legions 00439610 — Rust std replaces it
- [-] `List<EffectUINode>::flush` · 00438e60 · legions 004396b0 — Rust std replaces it
- [-] `List<EffectUINode>::push_back` · 004390e0 · legions 00439930 — Rust std replaces it

## `ac2/client/src/ui/gameplay/toolbar/gmchatui.cpp.c` (0/1)

### SmartArray<StringInfo>
- [-] `SmartArray<StringInfo>::grow` · 004353e0 · legions 00435c00 — Rust std replaces it

## `ac2/client/src/ui/gameplay/toolbar/gmnavigationui.cpp.c` (0/7)

### List<MarkerInfo>
- [-] `List<MarkerInfo>::push_back` · 00432190 · legions 00432a20 — Rust std replaces it

### SmartArray<CMapNoteRow>
- [-] `SmartArray<CMapNoteRow>::grow` · 00432230 · legions 00432ac0 — Rust std replaces it

### List<MarkerInfo>
- [-] `List<MarkerInfo>::pop_front` · 00432370 · legions 00432c00 — Rust std replaces it
- [-] `List<MarkerInfo>::remove` · 004323f0 · legions 00432c80 — Rust std replaces it
- [-] `List<MarkerInfo>::flush` · 004324d0 · legions 00432d60 — Rust std replaces it

### HashTable<unsigned_long,unsigned_long>
- [-] `HashTable<unsigned_long,unsigned_long>::find` · 00432720 · legions 00432fb0 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,List<MarkerInfo>_*>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,List<MarkerInfo>_*>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,List<MarkerInfo>_*>_*>` · 00433560 · legions 00433dc0 — Rust std replaces it

## `ac2/client/src/ui/gameplay/toolbar/gmshortcutui.cpp.c` (0/6)

### SmartArray<TimeLeftInfo>
- [-] `SmartArray<TimeLeftInfo>::grow` · 00435f10 · legions 00436760 — Rust std replaces it

### List<_STL::pair<InstanceID,UIElement_*>_>
- [-] `List<_STL::pair<InstanceID,UIElement_*>_>::push_back` · 004366c0 · legions 00436f10 — Rust std replaces it
- [-] `List<_STL::pair<InstanceID,UIElement_*>_>::remove` · 00436730 · legions 00436f80 — Rust std replaces it

### SmartArray<ShortcutInfo>
- [-] `SmartArray<ShortcutInfo>::grow` · 00436aa0 · legions 004372f0 — Rust std replaces it

### List<_STL::pair<InstanceID,UIElement_*>_>
- [-] `List<_STL::pair<InstanceID,UIElement_*>_>::flush` · 00436b80 · legions 004373d0 — Rust std replaces it

### HashTable<unsigned_long,unsigned_long>
- [-] `HashTable<unsigned_long,unsigned_long>::remove` · 00437160 · legions 004379b0 — Rust std replaces it

## `ac2/client/src/ui/gameplay/toolbar/gmtoolbarui.cpp.c` (0/3)

### List<ToolbarChildInfo>
- [-] `List<ToolbarChildInfo>::insert` · 00422450 · legions 00422220 — Rust std replaces it
- [-] `List<ToolbarChildInfo>::pop_front` · 004224d0 · legions 004222a0 — Rust std replaces it
- [-] `List<ToolbarChildInfo>::flush` · 00422e90 · legions 00422c60 — Rust std replaces it

## `ac2/client/src/ui/gameplay/toolbar/inventory/gmbackpackui.cpp.c` (0/1)

### _STL
- [-] `_STL::for_each` · 00441440 · legions 00441ca0 — Rust std replaces it

## `ac2/client/src/ui/gameplay/toolbar/inventory/gmpaperdollui.cpp.c` (0/1)

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,ItemDisplay_*>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,ItemDisplay_*>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,ItemDisplay_*>_*>` · 00443120 · legions 00443a70 — Rust std replaces it

## `ac2/client/src/ui/gameplay/toolbar/option/gmchatoptionsui.cpp.c` (0/1)

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,OptionExpander_*>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,OptionExpander_*>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,OptionExpander_*>_*>` · 004446e0 · legions 00445120 — Rust std replaces it

## `ac2/client/src/ui/gameplay/toolbar/quest/gmjournalquestsui.cpp.c` (0/9)

### List<gmJournalQuestsUI::QuestDescriptor>
- [-] `List<gmJournalQuestsUI::QuestDescriptor>::find` · 0043e880 · legions 0043f0f0 — Rust std replaces it
- [-] `List<gmJournalQuestsUI::QuestDescriptor>::push_back` · 0043e8b0 · legions 0043f120 — Rust std replaces it
- [-] `List<gmJournalQuestsUI::QuestDescriptor>::insert` · 0043e930 · legions 0043f1a0 — Rust std replaces it
- [-] `List<gmJournalQuestsUI::QuestDescriptor>::remove` · 0043eb40 · legions 0043f3b0 — Rust std replaces it
- [-] `List<gmJournalQuestsUI::QuestDescriptor>::remove` · 0043ed60 · legions 0043f5d0 — Rust std replaces it
- [-] `List<gmJournalQuestsUI::QuestDescriptor>::remove` · 0043edf0 · legions 0043f660 — Rust std replaces it
- [-] `List<gmJournalQuestsUI::QuestDescriptor>::insert_sort` · 0043ee70 · legions 0043f6e0 — Rust std replaces it
- [-] `List<gmJournalQuestsUI::QuestDescriptor>::flush` · 0043eec0 · legions 0043f770 — Rust std replaces it
- [-] `List<gmJournalQuestsUI::QuestDescriptor>::insert_unique_sort` · 0043efa0 · legions 0043f850 — Rust std replaces it

## `ac2/client/src/ui/gameplay/toolbar/quest/gmstoryquestsui.cpp.c` (0/4)

### List<gmActInfo>
- [-] `List<gmActInfo>::pop_front` · 0043f5b0 · legions 0043fe60 — Rust std replaces it
- [-] `List<gmActInfo>::flush` · 0043f610 · legions 0043fec0 — Rust std replaces it
- [-] `List<gmActInfo>::push_back` · 0043f7d0 · legions 00440030 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,int>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,int>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,int>_*>` · 0043fdf0 · legions 00440650 — Rust std replaces it

## `ac2/client/src/ui/gameplay/toolbar/skill/gmcraftui.cpp.c` (0/6)

### List<CraftSkillCategoryAdaptor>
- [-] `List<CraftSkillCategoryAdaptor>::push_back` · 004498e0 · legions 0044a540 — Rust std replaces it
- [-] `List<CraftSkillCategoryAdaptor>::insert` · 00449950 · legions 0044a5b0 — Rust std replaces it
- [-] `List<CraftSkillCategoryAdaptor>::insert_sort` · 00449e50 · legions 0044aab0 — Rust std replaces it
- [-] `List<CraftSkillCategoryAdaptor>::flush` · 00449ee0 · legions 0044ab40 — Rust std replaces it

### List<RecipeAdaptor>
- [-] `List<RecipeAdaptor>::flush` · 00449f50 · legions 0044abb0 — Rust std replaces it

### List<CraftSkillAdaptor>
- [-] `List<CraftSkillAdaptor>::flush` · 00449fd0 · legions 0044ac30 — Rust std replaces it

## `ac2/client/src/ui/gameplay/toolbar/skill/gmskilltreeui.cpp.c` (0/3)

### HashTable<unsigned_long,PList<float>_>
- [-] `HashTable<unsigned_long,PList<float>_>::find` · 004482d0 · legions 0062acc0 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,SkillUINode>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,SkillUINode>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,SkillUINode>_*>` · 004485a0 · legions 004491a0 — Rust std replaces it
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,SkillUINode>_*>::delete_contents` · 00448b20 · legions 00449700 — Rust std replaces it

## `ac2/client/src/ui/gameplay/toolbar/social/gmallegianceui.cpp.c` (0/3)

### IntrusiveHashTable<InstanceID,HashTableData<InstanceID,ICPhysicsObj_*>_*>
- [-] `IntrusiveHashTable<InstanceID,HashTableData<InstanceID,ICPhysicsObj_*>_*>::find_internal` · 0043cc50 · legions 0043d4a0 — Rust std replaces it

### HashTable<InstanceID,unsigned_long>
- [-] `HashTable<InstanceID,unsigned_long>::contains` · 0043ccb0 · legions 0043d500 — Rust std replaces it

### IntrusiveHashTable<InstanceID,HashSetData<InstanceID>_*>
- [-] `IntrusiveHashTable<InstanceID,HashSetData<InstanceID>_*>::IntrusiveHashTable<InstanceID,HashSetData<InstanceID>_*>` · 0043d200 · legions 0043da50 — Rust std replaces it

## `ac2/client/src/ui/gameplay/toolbar/social/gmfriendsui.cpp.c` (0/10)

### _STL
- [-] `_STL::__uninitialized_copy` · 0043a8e0 · legions 0043b130 — Rust std replaces it
- [-] `_STL::__uninitialized_fill_n` · 0043a930 · legions 0043b180 — Rust std replaces it
- [-] `_STL::__find_if` · 0043a980 · legions 0043b1d0 — Rust std replaces it
- [-] `_STL::__copy` · 0043aad0 · legions 0043b320 — Rust std replaces it
- [-] `_STL::find_if` · 0043ab50 · legions 0043b3a0 — Rust std replaces it
- [-] `_STL::__destroy_aux` · 0043aba0 · legions 0043b3f0 — Rust std replaces it

### _STL::vector<gmFriendsUI::FriendDescriptor,_STL::allocator<gmFriendsUI::FriendDescriptor>_>
- [-] `_STL::vector<gmFriendsUI::FriendDescriptor,_STL::allocator<gmFriendsUI::FriendDescriptor>_>::erase` · 0043add0 · legions 0043b620 — Rust std replaces it

### List<TurbineUserStatus>
- [-] `List<TurbineUserStatus>::push_back` · 0043b0e0 · legions 0043b930 — Rust std replaces it

### _STL::vector<gmFriendsUI::FriendDescriptor,_STL::allocator<gmFriendsUI::FriendDescriptor>_>
- [-] `_STL::vector<gmFriendsUI::FriendDescriptor,_STL::allocator<gmFriendsUI::FriendDescriptor>_>::_M_insert_overflow` · 0043b5d0 · legions 0043be20 — Rust std replaces it
- [-] `_STL::vector<gmFriendsUI::FriendDescriptor,_STL::allocator<gmFriendsUI::FriendDescriptor>_>::push_back` · 0043b990 · legions 0043c1e0 — Rust std replaces it

## `ac2/client/src/ui/gmuiflow.cpp.c` (0/1)

### PStringBase<char>
- [-] `PStringBase<char>::compute_hash` · 00407150 · legions 00406e60 — Rust std replaces it

## `ac2/client/src/ui/pregame/generation/gmavatarmodificationui.cpp.c` (0/11)

### _STL
- [-] `_STL::iterator_category` · 0040ac40 · legions 0040ab00 — Rust std replaces it
- [-] `_STL::__distance` · 0040ac50 · legions 0040ab10 — Rust std replaces it
- [-] `_STL::__advance` · 0040ac60 · legions 0040ab20 — Rust std replaces it

### PStringBase<char>
- [-] `PStringBase<char>::tolower` · 0040b2e0 · legions 0040b1e0 — Rust std replaces it

### List<float>
- [-] `List<float>::get_data` · 0040b3b0 · legions 0040b2b0 — Rust std replaces it

### _STL
- [-] `_STL::__lower_bound` · 0040b3e0 · legions 0040b2e0 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,unsigned_long>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,unsigned_long>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,unsigned_long>_*>` · 0040c530 · legions 0040c550 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PList<float>_>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PList<float>_>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PList<float>_>_*>` · 0040c5c0 · legions 0040c5e0 — Rust std replaces it
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PList<float>_>_*>::delete_contents` · 0040c650 · legions 0040c670 — Rust std replaces it

### HashTableData<unsigned_long,PList<float>_>
- [-] `HashTableData<unsigned_long,PList<float>_>::HashTableData<unsigned_long,PList<float>_>` · 0040c720 · legions 0040c740 — Rust std replaces it

### HashTable<unsigned_long,PList<float>_>
- [-] `HashTable<unsigned_long,PList<float>_>::add` · 0040c770 · legions 0040c790 — Rust std replaces it

## `ac2/client/src/ui/pregame/generation/gmavatarselectionui.cpp.c` (0/10)

### SmartArray<char_*>
- [-] `SmartArray<char_*>::grow` · 0040d3e0 · legions 0040d400 — Rust std replaces it

### IntrusiveHashTable<InstanceID,HashTableData<InstanceID,ICPhysicsObj_*>_*>
- [-] `IntrusiveHashTable<InstanceID,HashTableData<InstanceID,ICPhysicsObj_*>_*>::delete_contents` · 0040d580 · legions 004ef420 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,gmRaceSexInfo>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,gmRaceSexInfo>_*>::delete_contents` · 0040d600 · legions 0040d5a0 — Rust std replaces it

### PStringBase<char>
- [-] `PStringBase<char>::sprintf_append` · 0040d6e0 · legions 0040d630 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,BaseList<unsigned_long,11>_>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,BaseList<unsigned_long,11>_>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,BaseList<unsigned_long,11>_>_*>` · 0040d8b0 · legions 0040d860 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,gmRaceSexInfo>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,gmRaceSexInfo>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,gmRaceSexInfo>_*>` · 0040d940 · legions 0040d8f0 — Rust std replaces it

### IntrusiveHashTable<InstanceID,HashTableData<InstanceID,ICPhysicsObj_*>_*>
- [-] `IntrusiveHashTable<InstanceID,HashTableData<InstanceID,ICPhysicsObj_*>_*>::IntrusiveHashTable<InstanceID,HashTableData<InstanceID,ICPhysicsObj_*>_*>` · 0040dce0 · legions 0040dcc0 — Rust std replaces it

### IntrusiveHashTable<InstanceID,HashTableData<InstanceID,unsigned_long>_*>
- [-] `IntrusiveHashTable<InstanceID,HashTableData<InstanceID,unsigned_long>_*>::IntrusiveHashTable<InstanceID,HashTableData<InstanceID,unsigned_long>_*>` · 0040dd70 · legions 0040dd50 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,BaseList<unsigned_long,11>_>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,BaseList<unsigned_long,11>_>_*>::delete_contents` · 0040e430 · legions 0040e430 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,gmRaceSexInfo>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,gmRaceSexInfo>_*>::resize_internal` · 0040eec0 · legions 0040ee90 — Rust std replaces it

## `ac2/client/src/ui/pregame/gmcharacterselectionui.cpp.c` (0/1)

### PStringBase<unsigned_short>
- [-] `PStringBase<unsigned_short>::set` · 0040fb00 · legions 0040fad0 — Rust std replaces it

## `ac2/client/src/ui/pregame/gmdatapatchui.cpp.c` (0/2)

### PStringBase<unsigned_short>
- [-] `PStringBase<unsigned_short>::set` · 00411460 · legions 00411440 — Rust std replaces it
- [-] `PStringBase<unsigned_short>::eq` · 004114b0 · legions 00411490 — Rust std replaces it

## `ac2/client/src/ui/pregame/gmkeyboardui.cpp.c` (0/3)

### List<MappedKey>
- [-] `List<MappedKey>::get_data` · 00413bf0 · legions 00413a70 — Rust std replaces it
- [-] `List<MappedKey>::flush` · 00413c20 · legions 0066fe90 — Rust std replaces it

### HashTable<unsigned_long,PStringBase<char>_>
- [-] `HashTable<unsigned_long,PStringBase<char>_>::add` · 00413c60 · legions 00413aa0 — Rust std replaces it

## `ac2/client/src/ui/util/dialog/dialogfactory.cpp.c` (0/5)

### List<DialogInfo>
- [-] `List<DialogInfo>::push_back` · 00415300 · legions 004150f0 — Rust std replaces it
- [-] `List<DialogInfo>::insert` · 00415390 · legions 00415180 — Rust std replaces it
- [-] `List<DialogInfo>::pop_front` · 00415440 · legions 00415230 — Rust std replaces it
- [-] `List<DialogInfo>::remove` · 004154b0 · legions 004152a0 — Rust std replaces it
- [-] `List<DialogInfo>::flush` · 00415520 · legions 00415310 — Rust std replaces it

## `ac2/client/src/weenie/interp/glue/gallegianceprofileadaptor.cpp.c` (0/1)

### List<AllegianceDataAdaptor>
- [-] `List<AllegianceDataAdaptor>::push_back` · 0046a770 · legions 0046b070 — Rust std replaces it

## `ac2/client/src/weenie/interp/glue/gcm_friends.cpp.c` (0/1)

### List<TurbineUserStatus>
- [-] `List<TurbineUserStatus>::List<TurbineUserStatus>` · 00468100 · legions 00468db0 — Rust std replaces it

## `ac2/client/src/weenie/interp/glue/gcm_store.cpp.c` (0/2)

### List<UIShop>
- [-] `List<UIShop>::push_back` · 00467da0 · legions 00468a50 — Rust std replaces it
- [-] `List<UIShop>::List<UIShop>` · 00467e70 · legions 00468b20 — Rust std replaces it

## `ac2/client/src/weenie/interp/glue/ggmcwinterface.cpp.c` (0/12)

### PSmartArray<unsigned_long>
- [-] `PSmartArray<unsigned_long>::PSmartArray<unsigned_long>` · 00454630 — Rust std replaces it

### SmartArray<GrooveInfo>
- [-] `SmartArray<GrooveInfo>::grow` · 00454650 · legions 00662eb0 — Rust std replaces it

### List<CraftSkillAdaptor>
- [-] `List<CraftSkillAdaptor>::push_back` · 00454700 · legions 00455580 — Rust std replaces it

### List<IngredientAdaptor>
- [-] `List<IngredientAdaptor>::push_back` · 00454780 · legions 00455600 — Rust std replaces it

### List<RecipeAdaptor>
- [-] `List<RecipeAdaptor>::push_back` · 004547f0 · legions 00455670 — Rust std replaces it

### IntrusiveHashTable<unsigned___int64,HashTableData<unsigned___int64,unsigned_long>_*>
- [-] `IntrusiveHashTable<unsigned___int64,HashTableData<unsigned___int64,unsigned_long>_*>::find_internal` · 00454fb0 · legions 004b4070 — Rust std replaces it
- [-] `IntrusiveHashTable<unsigned___int64,HashTableData<unsigned___int64,unsigned_long>_*>::add` · 004556e0 · legions 004562f0 — Rust std replaces it

### HashTable<unsigned___int64,unsigned_long>
- [-] `HashTable<unsigned___int64,unsigned_long>::add` · 00455d50 · legions 00456920 — Rust std replaces it

### IntrusiveHashTable<unsigned___int64,HashTableData<unsigned___int64,unsigned_long>_*>
- [-] `IntrusiveHashTable<unsigned___int64,HashTableData<unsigned___int64,unsigned_long>_*>::IntrusiveHashTable<unsigned___int64,HashTableData<unsigned___int64,unsigned_long>_*>` · 00455e80 · legions 00456a50 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,unsigned___int64>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,unsigned___int64>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,unsigned___int64>_*>` · 00455f10 · legions 00456ae0 — Rust std replaces it

### List<gmActInfo>
- [-] `List<gmActInfo>::List<gmActInfo>` · 00456140 · legions 00456cf0 — Rust std replaces it

### IntrusiveHashTable<InstanceID,HashTableData<InstanceID,float>_*>
- [-] `IntrusiveHashTable<InstanceID,HashTableData<InstanceID,float>_*>::add_for_unpacking` · 00456eb0 · legions 006402b0 — Rust std replaces it

## `ac2/client/src/weenie/interp/glue/gmcnativefactory.cpp.c` (0/2)

### IntrusiveHashTable<InstanceID,HashTableData<InstanceID,FellowAdaptor>_*>
- [-] `IntrusiveHashTable<InstanceID,HashTableData<InstanceID,FellowAdaptor>_*>::delete_contents` · 004668a0 · legions 00467550 — Rust std replaces it

### List<AllegianceDataAdaptor>
- [-] `List<AllegianceDataAdaptor>::flush` · 00466ad0 · legions 00467780 — Rust std replaces it

## `ac2/shared/src/character/gameplayoptionsprofile.cpp.c` (3/3)

### HashTable<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>
- [x] `HashTable<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>::erase` · 005ff550 · legions 005c00d0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `HashTable<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>::find` · 005ff5c0 · legions 005c0140 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `HashTable<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>::set` · 005ffb90 · legions 005c09c0 — ac2-script glue::options / examination (Legions layout and efun switch)

## `ac2/shared/src/database/gmmasterdbmap.cpp.c` (0/1)

### PStringBase<char>
- [-] `PStringBase<char>::substring` · 006019a0 · legions 005c2880 — Rust std replaces it

## `ac2/shared/src/examination/examinationprofile.cpp.c` (0/3)

### List<ExaminationDataNode>
- [-] `List<ExaminationDataNode>::push_back` · 006086c0 · legions 005c9800 — Rust std replaces it
- [-] `List<ExaminationDataNode>::insert` · 00608750 · legions 005c9890 — Rust std replaces it
- [-] `List<ExaminationDataNode>::flush` · 00608f70 · legions 005c9e50 — Rust std replaces it

## `ac2/shared/src/fellowship/fellowshipadaptor.cpp.c` (0/2)

### HashTable<InstanceID,FellowAdaptor>
- [-] `HashTable<InstanceID,FellowAdaptor>::add` · 0060c010 · legions 005ccd60 — Rust std replaces it

### IntrusiveHashTable<InstanceID,HashTableData<InstanceID,FellowAdaptor>_*>
- [-] `IntrusiveHashTable<InstanceID,HashTableData<InstanceID,FellowAdaptor>_*>::IntrusiveHashTable<InstanceID,HashTableData<InstanceID,FellowAdaptor>_*>` · 0060c110 · legions 005cce60 — Rust std replaces it

## `ac2/shared/src/gun/turbinegun.cpp.c` (0/2)

### List<DualHashData<PStringBase<char>,CheckpointExportData>_*>
- [-] `List<DualHashData<PStringBase<char>,CheckpointExportData>_*>::flush` · 00605ec0 · legions 004214c0 — Rust std replaces it

### List<PStringBase<unsigned_short>_>
- [-] `List<PStringBase<unsigned_short>_>::List<PStringBase<unsigned_short>_>` · 00606d80 · legions 005c7c20 — Rust std replaces it

## `ac2/shared/src/interp/glue/gcharactergensystem.cpp.c` (0/3)

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PStringBase<char>_>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PStringBase<char>_>_*>::add` · 0046ad60 · legions 0056be10 — Rust std replaces it

### HashTableData<unsigned_long,BaseList<unsigned_long,11>_>
- [-] `HashTableData<unsigned_long,BaseList<unsigned_long,11>_>::HashTableData<unsigned_long,BaseList<unsigned_long,11>_>` · 0046b040 · legions 0046b7a0 — Rust std replaces it

### HashTable<unsigned_long,BaseList<unsigned_long,11>_>
- [-] `HashTable<unsigned_long,BaseList<unsigned_long,11>_>::add` · 0046b1e0 · legions 0046b940 — Rust std replaces it

## `ac2/shared/src/interp/glue/ggameplayoptionsprofile.cpp.c` (0/9)

### IntrusiveHashTable<ReferenceIDWrapper,HashTableData<ReferenceIDWrapper,unsigned_long>_*>
- [-] `IntrusiveHashTable<ReferenceIDWrapper,HashTableData<ReferenceIDWrapper,unsigned_long>_*>::remove` · 00475170 · legions 00471ec0 — Rust std replaces it
- [-] `IntrusiveHashTable<ReferenceIDWrapper,HashTableData<ReferenceIDWrapper,unsigned_long>_*>::find_internal` · 004751b0 · legions 00471f00 — Rust std replaces it

### IntrusiveHashTable<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>_*>
- [-] `IntrusiveHashTable<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>_*>::find_internal` · 004764c0 · legions 0049ed70 — Rust std replaces it

### IntrusiveHashTable<ReferenceIDWrapper,HashTableData<ReferenceIDWrapper,unsigned_long>_*>
- [-] `IntrusiveHashTable<ReferenceIDWrapper,HashTableData<ReferenceIDWrapper,unsigned_long>_*>::add` · 00476540 · legions 00472c30 — Rust std replaces it
- [-] `IntrusiveHashTable<ReferenceIDWrapper,HashTableData<ReferenceIDWrapper,unsigned_long>_*>::delete_contents` · 004765b0 · legions 00472ca0 — Rust std replaces it

### HashTable<ReferenceIDWrapper,unsigned_long>
- [-] `HashTable<ReferenceIDWrapper,unsigned_long>::add` · 00476640 · legions 00472d30 — Rust std replaces it

### HashTable<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>
- [-] `HashTable<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>::add` · 00476740 · legions 00472e30 — Rust std replaces it

### IntrusiveHashTable<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>_*>
- [-] `IntrusiveHashTable<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>_*>::IntrusiveHashTable<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>_*>` · 00476800 · legions 00472ef0 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,float>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,float>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,float>_*>` · 00476890 · legions 00472f80 — Rust std replaces it

## `ac2/shared/src/interp/glue/ggmactinfolist.cpp.c` (0/2)

### List<gmActInfo>
- [-] `List<gmActInfo>::insert` · 00474360 · legions 00471590 — Rust std replaces it
- [-] `List<gmActInfo>::insert_sort` · 00474470 · legions 00471680 — Rust std replaces it

## `ac2/shared/src/interp/glue/ggmquestinfolist.cpp.c` (0/2)

### List<gmQuestInfo>
- [-] `List<gmQuestInfo>::push_back` · 00472650 · legions 00470410 — Rust std replaces it
- [-] `List<gmQuestInfo>::pop_front` · 004727c0 · legions 004704f0 — Rust std replaces it

## `ac2/shared/src/interp/glue/ggmsceneinfolist.cpp.c` (0/2)

### List<gmSceneInfo>
- [-] `List<gmSceneInfo>::insert` · 00471690 · legions 0046fd70 — Rust std replaces it
- [-] `List<gmSceneInfo>::insert_sort` · 00471890 · legions 0046fec0 — Rust std replaces it

## `ac2/shared/src/interp/glue/guishop.cpp.c` (0/3)

### List<UIShopSale>
- [-] `List<UIShopSale>::push_back` · 0046e100 · legions 0046e450 — Rust std replaces it
- [-] `List<UIShopSale>::insert` · 0046e180 · legions 0046e4d0 — Rust std replaces it
- [-] `List<UIShopSale>::insert_sort` · 0046e220 · legions 0046e570 — Rust std replaces it

## `ac2/shared/src/quest/gmquestinfolist.cpp.c` (0/5)

### List<gmQuestInfo>
- [-] `List<gmQuestInfo>::insert` · 0060b1a0 · legions 005cbeb0 — Rust std replaces it
- [-] `List<gmQuestInfo>::remove` · 0060b2a0 · legions 005cbfb0 — Rust std replaces it
- [-] `List<gmQuestInfo>::insert_sort` · 0060b330 · legions 005cc040 — Rust std replaces it
- [-] `List<gmQuestInfo>::remove` · 0060b380 · legions 005cc0d0 — Rust std replaces it
- [-] `List<gmQuestInfo>::flush` · 0060b430 · legions 005cc180 — Rust std replaces it

## `ac2/shared/src/quest/gmsceneinfo.cpp.c` (0/2)

### List<gmKeyframe>
- [-] `List<gmKeyframe>::push_back` · 00609b00 · legions 005ca810 — Rust std replaces it
- [-] `List<gmKeyframe>::flush` · 00609c30 · legions 005ca940 — Rust std replaces it

## `ac2/shared/src/weenie/interface/gmwinterface.cpp.c` (0/8)

### _STL
- [-] `_STL::__uninitialized_copy` · 00452070 · legions 004b8e40 — Rust std replaces it
- [-] `_STL::__uninitialized_fill_n` · 004520b0 · legions 004531f0 — Rust std replaces it

### List<PStringBase<unsigned_short>_>
- [-] `List<PStringBase<unsigned_short>_>::find` · 00452510 · legions 00453650 — Rust std replaces it

### _STL
- [-] `_STL::__push_heap` · 00452b60 · legions 00453c70 — Rust std replaces it

### List<PStringBase<unsigned_short>_>
- [-] `List<PStringBase<unsigned_short>_>::pop_back` · 00452eb0 · legions 00453fc0 — Rust std replaces it

### PStringBaseIter<unsigned_short>
- [-] `PStringBaseIter<unsigned_short>::Replace` · 00452fb0 · legions 004540c0 — Rust std replaces it

### _STL::vector<GenericTriggerScheduler<ClientTrigger>::TriggerInfo,_STL::allocator<GenericTriggerScheduler<ClientTrigger>::TriggerInfo>_>
- [-] `_STL::vector<GenericTriggerScheduler<ClientTrigger>::TriggerInfo,_STL::allocator<GenericTriggerScheduler<ClientTrigger>::TriggerInfo>_>::_M_insert_overflow` · 00453c50 · legions 00454cd0 — Rust std replaces it

### _STL::priority_queue<GenericTriggerScheduler<ClientTrigger>::TriggerInfo,_STL::vector<GenericTriggerScheduler<ClientTrigger>::TriggerInfo,_STL::allocator<GenericTriggerScheduler<ClientTrigger>::TriggerInfo>_>,_STL::less<GenericTriggerScheduler<ClientTrigger>::TriggerInfo>_>
- [-] `_STL::priority_queue<GenericTriggerScheduler<ClientTrigger>::TriggerInfo,_STL::vector<GenericTriggerScheduler<ClientTrigger>::TriggerInfo,_STL::allocator<GenericTriggerScheduler<ClientTrigger>::TriggerInfo>_>,_STL::less<GenericTriggerScheduler<ClientTrigger>::TriggerInfo>_>::push` · 00453d50 · legions 00454dd0 — Rust std replaces it

## `core/src/debug/reportgen.cpp.c` (0/13)

### _STL
- [-] `_STL::__uninitialized_copy` · 00596e00 — Rust std replaces it
- [-] `_STL::__push_heap` · 00596e40 · legions 00565540 — Rust std replaces it
- [-] `_STL::__unguarded_insertion_sort_aux` · 00596ef0 · legions 005655f0 — Rust std replaces it
- [-] `_STL::__adjust_heap` · 00596f30 · legions 00565630 — Rust std replaces it
- [-] `_STL::__unguarded_partition` · 005970c0 · legions 005657f0 — Rust std replaces it
- [-] `_STL::__make_heap` · 00597110 · legions 00565840 — Rust std replaces it
- [-] `_STL::__insertion_sort` · 00597260 · legions 00565960 — Rust std replaces it
- [-] `_STL::sort_heap` · 005972e0 · legions 005659e0 — Rust std replaces it
- [-] `_STL::__final_insertion_sort` · 00597850 · legions 00566060 — Rust std replaces it
- [-] `_STL::__partial_sort` · 005978a0 · legions 005660b0 — Rust std replaces it

### _STL::vector<_tagSTACKFRAME,_STL::allocator<_tagSTACKFRAME>_>
- [-] `_STL::vector<_tagSTACKFRAME,_STL::allocator<_tagSTACKFRAME>_>::_M_insert_overflow` · 00597a20 — Rust std replaces it

### _STL
- [-] `_STL::__introsort_loop` · 00597b60 · legions 00566120 — Rust std replaces it
- [-] `_STL::sort` · 00597c10 · legions 005661d0 — Rust std replaces it

## `core/src/newthread/portalthread_win32.cpp.c` (0/1)

### PortalThread
- [-] `PortalThread::_portal_startup` · 0063c7b0 — Rust std replaces it

## `core/src/serialize/smartbuffer.cpp.c` (0/1)

### SmartArray<_STL::pair<ICPhysicsObj_*,BBox>_>
- [-] `SmartArray<_STL::pair<ICPhysicsObj_*,BBox>_>::SmartArray<_STL::pair<ICPhysicsObj_*,BBox>_>` · 00735e80 · legions 006b1b20 — Rust std replaces it

## `portal/client/src/action/interp/actioninterp.cpp.c` (0/6)

### _STL
- [-] `_STL::__uninitialized_copy` · 00479180 · legions 00474fc0 — Rust std replaces it
- [-] `_STL::__copy_ptrs` · 00479280 · legions 004750c0 — Rust std replaces it

### _STL::vector<ActionInfo,_STL::allocator<ActionInfo>_>
- [-] `_STL::vector<ActionInfo,_STL::allocator<ActionInfo>_>::erase` · 004793a0 · legions 004751e0 — Rust std replaces it
- [-] `_STL::vector<ActionInfo,_STL::allocator<ActionInfo>_>::vector<ActionInfo,_STL::allocator<ActionInfo>_>` · 004795b0 · legions 004753f0 — Rust std replaces it
- [-] `_STL::vector<ActionInfo,_STL::allocator<ActionInfo>_>::_M_insert_overflow` · 00479640 · legions 00475480 — Rust std replaces it
- [-] `_STL::vector<ActionInfo,_STL::allocator<ActionInfo>_>::push_back` · 00479960 · legions 004757a0 — Rust std replaces it

## `portal/client/src/action/map/actionmap.cpp.c` (0/12)

### _STL::_Rb_tree<ActionWithModifiers,_STL::pair<ActionWithModifiers_const_,ActionMapValue>,_STL::_Select1st<_STL::pair<ActionWithModifiers_const_,ActionMapValue>_>,_STL::less<ActionWithModifiers>,_STL::allocator<_STL::pair<ActionWithModifiers_const_,ActionMapValue>_>_>
- [-] `_STL::_Rb_tree<ActionWithModifiers,_STL::pair<ActionWithModifiers_const_,ActionMapValue>,_STL::_Select1st<_STL::pair<ActionWithModifiers_const_,ActionMapValue>_>,_STL::less<ActionWithModifiers>,_STL::allocator<_STL::pair<ActionWithModifiers_const_,ActionMapValue>_>_>::_M_find` · 0072d6d0 · legions 006ab750 — Rust std replaces it
- [-] `_STL::_Rb_tree<ActionWithModifiers,_STL::pair<ActionWithModifiers_const_,ActionMapValue>,_STL::_Select1st<_STL::pair<ActionWithModifiers_const_,ActionMapValue>_>,_STL::less<ActionWithModifiers>,_STL::allocator<_STL::pair<ActionWithModifiers_const_,ActionMapValue>_>_>::_M_lower_bound` · 0072d730 · legions 006ab7b0 — Rust std replaces it
- [-] `_STL::_Rb_tree<ActionWithModifiers,_STL::pair<ActionWithModifiers_const_,ActionMapValue>,_STL::_Select1st<_STL::pair<ActionWithModifiers_const_,ActionMapValue>_>,_STL::less<ActionWithModifiers>,_STL::allocator<_STL::pair<ActionWithModifiers_const_,ActionMapValue>_>_>::_M_erase` · 0072db90 · legions 006abc10 — Rust std replaces it

### _STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,DescripValue>,_STL::_Select1st<_STL::pair<unsigned_long_const_,DescripValue>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,DescripValue>_>_>
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,DescripValue>,_STL::_Select1st<_STL::pair<unsigned_long_const_,DescripValue>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,DescripValue>_>_>::_M_erase` · 0072dbe0 · legions 006abc60 — Rust std replaces it

### _STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned_long>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned_long>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>::insert_unique` · 0072dc30 · legions 006abcb0 — Rust std replaces it

### _STL::_Rb_tree<ActionWithModifiers,_STL::pair<ActionWithModifiers_const_,ActionMapValue>,_STL::_Select1st<_STL::pair<ActionWithModifiers_const_,ActionMapValue>_>,_STL::less<ActionWithModifiers>,_STL::allocator<_STL::pair<ActionWithModifiers_const_,ActionMapValue>_>_>
- [-] `_STL::_Rb_tree<ActionWithModifiers,_STL::pair<ActionWithModifiers_const_,ActionMapValue>,_STL::_Select1st<_STL::pair<ActionWithModifiers_const_,ActionMapValue>_>,_STL::less<ActionWithModifiers>,_STL::allocator<_STL::pair<ActionWithModifiers_const_,ActionMapValue>_>_>::_M_insert` · 0072dcf0 · legions 006abd70 — Rust std replaces it
- [-] `_STL::_Rb_tree<ActionWithModifiers,_STL::pair<ActionWithModifiers_const_,ActionMapValue>,_STL::_Select1st<_STL::pair<ActionWithModifiers_const_,ActionMapValue>_>,_STL::less<ActionWithModifiers>,_STL::allocator<_STL::pair<ActionWithModifiers_const_,ActionMapValue>_>_>::insert_unique` · 0072ddf0 · legions 006abe70 — Rust std replaces it

### _STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,DescripValue>,_STL::_Select1st<_STL::pair<unsigned_long_const_,DescripValue>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,DescripValue>_>_>
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,DescripValue>,_STL::_Select1st<_STL::pair<unsigned_long_const_,DescripValue>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,DescripValue>_>_>::_M_insert` · 0072ded0 · legions 006abf50 — Rust std replaces it
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,DescripValue>,_STL::_Select1st<_STL::pair<unsigned_long_const_,DescripValue>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,DescripValue>_>_>::insert_unique` · 0072dfa0 · legions 006ac020 — Rust std replaces it

### _STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned_long>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned_long>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>::insert_unique` · 0072e060 · legions 006ac0e0 — Rust std replaces it

### _STL::_Rb_tree<ActionWithModifiers,_STL::pair<ActionWithModifiers_const_,ActionMapValue>,_STL::_Select1st<_STL::pair<ActionWithModifiers_const_,ActionMapValue>_>,_STL::less<ActionWithModifiers>,_STL::allocator<_STL::pair<ActionWithModifiers_const_,ActionMapValue>_>_>
- [-] `_STL::_Rb_tree<ActionWithModifiers,_STL::pair<ActionWithModifiers_const_,ActionMapValue>,_STL::_Select1st<_STL::pair<ActionWithModifiers_const_,ActionMapValue>_>,_STL::less<ActionWithModifiers>,_STL::allocator<_STL::pair<ActionWithModifiers_const_,ActionMapValue>_>_>::insert_unique` · 0072e240 · legions 006ac2c0 — Rust std replaces it

### _STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,DescripValue>,_STL::_Select1st<_STL::pair<unsigned_long_const_,DescripValue>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,DescripValue>_>_>
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,DescripValue>,_STL::_Select1st<_STL::pair<unsigned_long_const_,DescripValue>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,DescripValue>_>_>::insert_unique` · 0072e4b0 · legions 006ac530 — Rust std replaces it

## `portal/client/src/bench/bench.cpp.c` (0/2)

### List<PerfMonCounterInfo::PerfMonCounterNameHelp>
- [-] `List<PerfMonCounterInfo::PerfMonCounterNameHelp>::pop_front` · 0047a7f0 · legions 004bbf40 — Rust std replaces it
- [-] `List<PerfMonCounterInfo::PerfMonCounterNameHelp>::pop_head` · 0047b1c0 · legions 004bcb10 — Rust std replaces it

## `portal/client/src/cinputmap/ckeymap.cpp.c` (0/5)

### IntrusiveHashTable<PStringBase<char>,HashTableData<PStringBase<char>,FrameDebugInfo>_*>
- [-] `IntrusiveHashTable<PStringBase<char>,HashTableData<PStringBase<char>,FrameDebugInfo>_*>::find_internal` · 00599f60 · legions 0060a3f0 — Rust std replaces it

### List<ActionMapping>
- [-] `List<ActionMapping>::push_back` · 0059a0b0 · legions 00567070 — Rust std replaces it
- [-] `List<ActionMapping>::pop_front` · 0059a140 · legions 00567100 — Rust std replaces it

### IntrusiveHashTable<PStringBase<char>,HashTableData<PStringBase<char>,unsigned_long>_*>
- [-] `IntrusiveHashTable<PStringBase<char>,HashTableData<PStringBase<char>,unsigned_long>_*>::IntrusiveHashTable<PStringBase<char>,HashTableData<PStringBase<char>,unsigned_long>_*>` · 0059a580 · legions 00567540 — Rust std replaces it

### List<ActionMapping>
- [-] `List<ActionMapping>::flush` · 0059a610 · legions 005675d0 — Rust std replaces it

## `portal/client/src/cinputmap/directx/cinputmanager_win32.cpp.c` (0/14)

### _STL::_Deque_iterator_base<HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>_>
- [-] `_STL::_Deque_iterator_base<HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>_>::_M_advance` · 0059e5f0 · legions 00645390 — Rust std replaces it

### IntrusiveHashTable<CInputManager_WIN32::QualifiedKey,HashTableData<CInputManager_WIN32::QualifiedKey,unsigned_long>_*>
- [-] `IntrusiveHashTable<CInputManager_WIN32::QualifiedKey,HashTableData<CInputManager_WIN32::QualifiedKey,unsigned_long>_*>::find_internal` · 0059e910 · legions 0056b830 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,unsigned_long>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,unsigned_long>_*>::remove` · 0059eb20 · legions 005f6630 — Rust std replaces it

### _STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned_long>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned_long>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>::equal_range` · 0059ef00 · legions 0056be70 — Rust std replaces it

### _STL::_Deque_base<HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>,_STL::allocator<HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>_>_>
- [-] `_STL::_Deque_base<HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>,_STL::allocator<HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>_>_>::_M_create_nodes` · 0059f050 · legions 00645500 — Rust std replaces it

### _STL::deque<HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>,_STL::allocator<HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>_>_>
- [-] `_STL::deque<HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>,_STL::allocator<HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>_>_>::_M_reallocate_map` · 0059f0b0 · legions 006455b0 — Rust std replaces it

### _STL::_Deque_base<HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>,_STL::allocator<HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>_>_>
- [-] `_STL::_Deque_base<HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>,_STL::allocator<HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>_>_>::_M_initialize_map` · 0059f8c0 · legions 0056c670 — Rust std replaces it

### _STL::deque<HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>,_STL::allocator<HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>_>_>
- [-] `_STL::deque<HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>,_STL::allocator<HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>_>_>::pop_front` · 0059fca0 · legions 0056c9a0 — Rust std replaces it

### IntrusiveHashTable<CInputManager_WIN32::QualifiedKey,HashTableData<CInputManager_WIN32::QualifiedKey,unsigned_long>_*>
- [-] `IntrusiveHashTable<CInputManager_WIN32::QualifiedKey,HashTableData<CInputManager_WIN32::QualifiedKey,unsigned_long>_*>::IntrusiveHashTable<CInputManager_WIN32::QualifiedKey,HashTableData<CInputManager_WIN32::QualifiedKey,unsigned_long>_*>` · 0059fcf0 · legions 0056c9f0 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,CInputManager_WIN32::KeyHitInfo>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,CInputManager_WIN32::KeyHitInfo>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,CInputManager_WIN32::KeyHitInfo>_*>` · 0059fd80 · legions 0056ca80 — Rust std replaces it

### _STL::_Rb_tree<unsigned_long,unsigned_long,_STL::_Identity<unsigned_long>,_STL::less<unsigned_long>,_STL::allocator<unsigned_long>_>
- [-] `_STL::_Rb_tree<unsigned_long,unsigned_long,_STL::_Identity<unsigned_long>,_STL::less<unsigned_long>,_STL::allocator<unsigned_long>_>::erase` · 0059fe10 · legions 0056cb10 — Rust std replaces it
- [-] `_STL::_Rb_tree<unsigned_long,unsigned_long,_STL::_Identity<unsigned_long>,_STL::less<unsigned_long>,_STL::allocator<unsigned_long>_>::erase` · 005a0190 · legions 0056ce90 — Rust std replaces it

### _STL::deque<HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>,_STL::allocator<HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>_>_>
- [-] `_STL::deque<HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>,_STL::allocator<HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>_>_>::_M_push_front_aux_v` · 005a02e0 · legions 0056cfe0 — Rust std replaces it
- [-] `_STL::deque<HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>,_STL::allocator<HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>_>_>::push_front` · 005a0960 · legions 0056d660 — Rust std replaces it

## `portal/client/src/clcache/clcachemain.cpp.c` (0/12)

### PStringBase<char>
- [-] `PStringBase<char>::append_uint32` · 004c05a0 · legions 004b4a30 — Rust std replaces it

### SmartArray<Trigger>
- [-] `SmartArray<Trigger>::grow` · 004c0ae0 · legions 004b4f70 — Rust std replaces it

### SmartArray<ImpulseTrigger>
- [-] `SmartArray<ImpulseTrigger>::grow` · 004c0c90 · legions 004b5120 — Rust std replaces it

### SmartArray<CAllIterationList::PTaggedIterationList>
- [-] `SmartArray<CAllIterationList::PTaggedIterationList>::grow` · 004c1550 · legions 004b59b0 — Rust std replaces it

### SmartArray<CMissingIteration>
- [-] `SmartArray<CMissingIteration>::grow` · 004c1780 · legions 004b5b80 — Rust std replaces it

### IntrusiveHashTable<unsigned___int64,HashTableData<unsigned___int64,DiskController_*>_*>
- [-] `IntrusiveHashTable<unsigned___int64,HashTableData<unsigned___int64,DiskController_*>_*>::IntrusiveHashTable<unsigned___int64,HashTableData<unsigned___int64,DiskController_*>_*>` · 004c2010 · legions 004b63a0 — Rust std replaces it

### IntrusiveHashTable<QualifiedDataID,HashTableData<QualifiedDataID,CMissingIteration_*>_*>
- [-] `IntrusiveHashTable<QualifiedDataID,HashTableData<QualifiedDataID,CMissingIteration_*>_*>::IntrusiveHashTable<QualifiedDataID,HashTableData<QualifiedDataID,CMissingIteration_*>_*>` · 004c20a0 · legions 004b6430 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,unsigned_long>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,unsigned_long>_*>::end` · 004c2330 · legions 004b66c0 — Rust std replaces it

### HashTable<unsigned_long,MotionValues>
- [-] `HashTable<unsigned_long,MotionValues>::resize` · 004c2350 · legions 004b66e0 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,TerrainList>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,TerrainList>_*>::delete_contents` · 004c2370 · legions 004b6740 — Rust std replaces it

### HashTableData<unsigned_long,TerrainList>
- [-] `HashTableData<unsigned_long,TerrainList>::HashTableData<unsigned_long,TerrainList>` · 004c47a0 · legions 004b8b70 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,MotionValues>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,MotionValues>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,MotionValues>_*>` · 004c4820 · legions 004b8bf0 — Rust std replaces it

## `portal/client/src/clienttriggers/clienttriggers.cpp.c` (0/3)

### _STL
- [-] `_STL::__copy` · 004c4a70 · legions 004b8e80 — Rust std replaces it
- [-] `_STL::__adjust_heap` · 004c4ac0 · legions 004b8ed0 — Rust std replaces it

### _STL::vector<GenericTriggerScheduler<ClientTrigger>::TriggerInfo,_STL::allocator<GenericTriggerScheduler<ClientTrigger>::TriggerInfo>_>
- [-] `_STL::vector<GenericTriggerScheduler<ClientTrigger>::TriggerInfo,_STL::allocator<GenericTriggerScheduler<ClientTrigger>::TriggerInfo>_>::_M_allocate_and_copy` · 004c4b80 · legions 004b8f90 — Rust std replaces it

## `portal/client/src/clinet/cflowqueue.cpp.c` (2/2)

### PQueueArray<unsigned_long,void_*>
- [x] `PQueueArray<unsigned_long,void_*>::_ExpandNodes` · 004cbf50 · legions 004c04c0 — pqueue.rs PQueueArray grows as needed
- [x] `PQueueArray<unsigned_long,void_*>::_Heapify` · 004cbfa0 · legions 004c0510 — Rust std replaces it

## `portal/client/src/clinet/clientnet.cpp.c` (0/4)

### PStringBase<char>
- [-] `PStringBase<char>::find_substring` · 004c5b20 · legions 004b9ee0 — Rust std replaces it
- [-] `PStringBase<char>::to_uint32` · 004c5b50 · legions 004b9f10 — Rust std replaces it

### SmartArray<ClientNet::CReferralQueueEntry>
- [-] `SmartArray<ClientNet::CReferralQueueEntry>::grow` · 004c6010 · legions 004ba3d0 — Rust std replaces it

### PStringBase<unsigned_short>
- [-] `PStringBase<unsigned_short>::to_spstring` · 004c8650 · legions 004bcba0 — Rust std replaces it

## `portal/client/src/cregion/cdetailmap/cdetailmapdesc.cpp.c` (0/6)

### SmartArray<MyRect>
- [-] `SmartArray<MyRect>::grow` · 0051c5f0 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,MapInfo>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,MapInfo>_*>::delete_contents` · 0051c6f0 — Rust std replaces it

### SmartArray<CDetailMapRow>
- [-] `SmartArray<CDetailMapRow>::grow` · 0051cd50 · legions 00509ef0 — Rust std replaces it

### HashTableData<unsigned_long,MapInfo>
- [-] `HashTableData<unsigned_long,MapInfo>::HashTableData<unsigned_long,MapInfo>` · 0051cea0 — Rust std replaces it

### HashTable<unsigned_long,MapInfo>
- [-] `HashTable<unsigned_long,MapInfo>::set` · 0051cf90 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,MapInfo>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,MapInfo>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,MapInfo>_*>` · 0051d020 — Rust std replaces it

## `portal/client/src/cregion/csound/cambientsoundinfo.cpp.c` (0/2)

### SmartArray<CContinSoundInfo>
- [-] `SmartArray<CContinSoundInfo>::grow` · 0052a740 · legions 005102a0 — Rust std replaces it

### SmartArray<CIntermitSoundInfo>
- [-] `SmartArray<CIntermitSoundInfo>::grow` · 0052a820 · legions 00510380 — Rust std replaces it

## `portal/client/src/cregion/csound/csounddesc.cpp.c` (0/1)

### SmartArray<CAmbientSoundInfo>
- [-] `SmartArray<CAmbientSoundInfo>::grow` · 00523620 · legions 0050d630 — Rust std replaces it

## `portal/client/src/device/win95/wdevice.cpp.c` (0/16)

### _STL
- [-] `_STL::__unguarded_linear_insert` · 00533ff0 · legions 00517f90 — Rust std replaces it
- [-] `_STL::__push_heap` · 00534040 · legions 00517fe0 — Rust std replaces it
- [-] `_STL::__copy_backward` · 005340c0 · legions 00518060 — Rust std replaces it
- [-] `_STL::__unguarded_insertion_sort_aux` · 00534940 · legions 005188e0 — Rust std replaces it
- [-] `_STL::__adjust_heap` · 00534990 · legions 00518930 — Rust std replaces it
- [-] `_STL::__unguarded_partition` · 00534ba0 · legions 00518b40 — Rust std replaces it
- [-] `_STL::__make_heap` · 00534c30 · legions 00518bd0 — Rust std replaces it

### List<PStringBase<unsigned_short>_>
- [-] `List<PStringBase<unsigned_short>_>::pop_front` · 00535000 · legions 0041f1c0 — Rust std replaces it

### _STL
- [-] `_STL::__linear_insert` · 00535310 · legions 00519210 — Rust std replaces it
- [-] `_STL::sort_heap` · 00535390 · legions 00519290 — Rust std replaces it

### List<PStringBase<char>_>
- [-] `List<PStringBase<char>_>::flush` · 00535430 · legions 00519330 — Rust std replaces it

### _STL
- [-] `_STL::__insertion_sort` · 00535490 · legions 00519390 — Rust std replaces it
- [-] `_STL::__partial_sort` · 005354f0 · legions 005193f0 — Rust std replaces it

### _STL::vector<unsigned_short_const_*,_STL::allocator<unsigned_short_const_*>_>
- [-] `_STL::vector<unsigned_short_const_*,_STL::allocator<unsigned_short_const_*>_>::_M_insert_overflow` · 005355e0 · legions 005194e0 — Rust std replaces it

### _STL
- [-] `_STL::__introsort_loop` · 005357f0 · legions 005196f0 — Rust std replaces it

### PStringBase<char>
- [-] `PStringBase<char>::replace` · 005358d0 · legions 005197d0 — Rust std replaces it

## `portal/client/src/events/ecm_login.cpp.c` (0/2)

### SmartArray<PStringBase<unsigned_short>_>
- [-] `SmartArray<PStringBase<unsigned_short>_>::grow` · 00493e80 · legions 0048c650 — Rust std replaces it
- [-] `SmartArray<PStringBase<unsigned_short>_>::Reset` · 00493f30 · legions 0048c700 — Rust std replaces it

## `portal/client/src/events/ecm_physics.cpp.c` (0/3)

### List<Position>
- [-] `List<Position>::pop_front` · 00497320 · legions 0048f830 — Rust std replaces it
- [-] `List<Position>::flush` · 00497380 · legions 0048f890 — Rust std replaces it
- [-] `List<Position>::push_back` · 00497640 · legions 0048fb50 — Rust std replaces it

## `portal/client/src/http/connectioncache.cpp.c` (0/11)

### _STL::__char_traits_base<unsigned_short,unsigned_short>
- [-] `_STL::__char_traits_base<unsigned_short,unsigned_short>::compare` · 006fb790 · legions 0067b590 — Rust std replaces it

### _STL
- [-] `_STL::for_each` · 006fb7e0 · legions 0067b5e0 — Rust std replaces it
- [-] `_STL::__find` · 006fb840 · legions 0067b640 — Rust std replaces it
- [-] `_STL::find` · 006fbbb0 · legions 0067b9b0 — Rust std replaces it

### _STL::_Deque_base<HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>,_STL::allocator<HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>_>_>
- [-] `_STL::_Deque_base<HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>,_STL::allocator<HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>_>_>::_M_destroy_nodes` · 006fbc10 · legions 00645560 — Rust std replaces it

### _STL::_Deque_base<ConnectionCache::Entry,_STL::allocator<ConnectionCache::Entry>_>
- [-] `_STL::_Deque_base<ConnectionCache::Entry,_STL::allocator<ConnectionCache::Entry>_>::_M_initialize_map` · 006fbc60 · legions 0067ba10 — Rust std replaces it

### _STL
- [-] `_STL::__destroy_aux` · 006fbea0 · legions 0067bc50 — Rust std replaces it
- [-] `_STL::__destroy_aux` · 006fbf00 · legions 0067bcb0 — Rust std replaces it

### _STL::deque<ConnectionCache::Entry,_STL::allocator<ConnectionCache::Entry>_>
- [-] `_STL::deque<ConnectionCache::Entry,_STL::allocator<ConnectionCache::Entry>_>::_M_push_front_aux_v` · 006fbf50 · legions 0067bd00 — Rust std replaces it
- [-] `_STL::deque<ConnectionCache::Entry,_STL::allocator<ConnectionCache::Entry>_>::_M_pop_back_aux` · 006fbfe0 · legions 0067bd90 — Rust std replaces it
- [-] `_STL::deque<ConnectionCache::Entry,_STL::allocator<ConnectionCache::Entry>_>::clear` · 006fc200 · legions 0067bfb0 — Rust std replaces it

## `portal/client/src/http/wininetasynchttpclient.cpp.c` (0/2)

### _STL::_String_base<unsigned_short,_STL::allocator<unsigned_short>_>
- [-] `_STL::_String_base<unsigned_short,_STL::allocator<unsigned_short>_>::_M_allocate_block` · 006fa4d0 · legions 0067a2d0 — Rust std replaces it

### _STL::basic_string<unsigned_short,_STL::char_traits<unsigned_short>,_STL::allocator<unsigned_short>_>
- [-] `_STL::basic_string<unsigned_short,_STL::char_traits<unsigned_short>,_STL::allocator<unsigned_short>_>::basic_string<unsigned_short,_STL::char_traits<unsigned_short>,_STL::allocator<unsigned_short>_>` · 006fa570 · legions 0067a370 — Rust std replaces it

## `portal/client/src/http/wininetimplementation.cpp.c` (0/1)

### _STL::basic_string<unsigned_short,_STL::char_traits<unsigned_short>,_STL::allocator<unsigned_short>_>
- [-] `_STL::basic_string<unsigned_short,_STL::char_traits<unsigned_short>,_STL::allocator<unsigned_short>_>::basic_string<unsigned_short,_STL::char_traits<unsigned_short>,_STL::allocator<unsigned_short>_>` · 006fb020 · legions 0067ae20 — Rust std replaces it

## `portal/client/src/objects/cobjmaint.cpp.c` (0/34)

### _STL::_Deque_iterator<NetBlob_*,_STL::_Nonconst_traits<NetBlob_*>_>
- [-] `_STL::_Deque_iterator<NetBlob_*,_STL::_Nonconst_traits<NetBlob_*>_>::_Deque_iterator<NetBlob_*,_STL::_Nonconst_traits<NetBlob_*>_>` · 004d5080 · legions 004c7a00 — Rust std replaces it

### _STL::_Rb_global<bool>
- [-] `_STL::_Rb_global<bool>::_Rebalance_for_erase` · 004d52d0 · legions 004c7c90 — Rust std replaces it

### _STL::_Rb_tree<InstanceID,_STL::pair<InstanceID_const_,ObjectEventQueue>,_STL::_Select1st<_STL::pair<InstanceID_const_,ObjectEventQueue>_>,_STL::less<InstanceID>,_STL::allocator<_STL::pair<InstanceID_const_,ObjectEventQueue>_>_>
- [-] `_STL::_Rb_tree<InstanceID,_STL::pair<InstanceID_const_,ObjectEventQueue>,_STL::_Select1st<_STL::pair<InstanceID_const_,ObjectEventQueue>_>,_STL::less<InstanceID>,_STL::allocator<_STL::pair<InstanceID_const_,ObjectEventQueue>_>_>::_M_find` · 004d5620 · legions 004c80d0 — Rust std replaces it

### _STL::_Rb_tree<InstanceID,InstanceID,_STL::_Identity<InstanceID>,_STL::less<InstanceID>,_STL::allocator<InstanceID>_>
- [-] `_STL::_Rb_tree<InstanceID,InstanceID,_STL::_Identity<InstanceID>,_STL::less<InstanceID>,_STL::allocator<InstanceID>_>::_M_upper_bound` · 004d58e0 · legions 004c8390 — Rust std replaces it

### _STL
- [-] `_STL::__copy` · 004d59e0 · legions 004c8490 — Rust std replaces it

### SmartArray<QualifiedDataID>
- [-] `SmartArray<QualifiedDataID>::add` · 004d5cd0 · legions 004b49e0 — Rust std replaces it

### _STL
- [-] `_STL::__copy_aux` · 004d5d20 · legions 004c87e0 — Rust std replaces it

### PStringBase<unsigned_short>
- [-] `PStringBase<unsigned_short>::sprintf_append` · 004d6030 · legions 004c8b50 — Rust std replaces it

### _STL
- [-] `_STL::__uninitialized_copy` · 004d60a0 · legions 004c8bc0 — Rust std replaces it

### _STL::_Deque_base<NetBlob_*,_STL::allocator<NetBlob_*>_>
- [-] `_STL::_Deque_base<NetBlob_*,_STL::allocator<NetBlob_*>_>::_M_create_nodes` · 004d6120 · legions 005750a0 — Rust std replaces it
- [-] `_STL::_Deque_base<NetBlob_*,_STL::allocator<NetBlob_*>_>::_M_destroy_nodes` · 004d6180 · legions 004c8c40 — Rust std replaces it
- [-] `_STL::_Deque_base<NetBlob_*,_STL::allocator<NetBlob_*>_>::_M_initialize_map` · 004d6870 · legions 004dcae0 — Rust std replaces it

### _STL::_Rb_tree<InstanceID,InstanceID,_STL::_Identity<InstanceID>,_STL::less<InstanceID>,_STL::allocator<InstanceID>_>
- [-] `_STL::_Rb_tree<InstanceID,InstanceID,_STL::_Identity<InstanceID>,_STL::less<InstanceID>,_STL::allocator<InstanceID>_>::_M_erase` · 004d6920 · legions 004c9390 — Rust std replaces it

### IntrusiveHashTable<InstanceID,PriorityHash<InstanceID,double>::PriorityHashData_*>
- [-] `IntrusiveHashTable<InstanceID,PriorityHash<InstanceID,double>::PriorityHashData_*>::IntrusiveHashTable<InstanceID,PriorityHash<InstanceID,double>::PriorityHashData_*>` · 004d6a70 · legions 004c94e0 — Rust std replaces it
- [-] `IntrusiveHashTable<InstanceID,PriorityHash<InstanceID,double>::PriorityHashData_*>::grow` · 004d6b00 · legions 004c9570 — Rust std replaces it

### _STL::_Rb_tree<InstanceID,InstanceID,_STL::_Identity<InstanceID>,_STL::less<InstanceID>,_STL::allocator<InstanceID>_>
- [-] `_STL::_Rb_tree<InstanceID,InstanceID,_STL::_Identity<InstanceID>,_STL::less<InstanceID>,_STL::allocator<InstanceID>_>::_M_insert` · 004d6b50 · legions 004c95c0 — Rust std replaces it

### SmartArray<BehaviorParams>
- [-] `SmartArray<BehaviorParams>::grow` · 004d6c20 · legions 004c9690 — Rust std replaces it

### _STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,SliderData>,_STL::_Select1st<_STL::pair<unsigned_long_const_,SliderData>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,SliderData>_>_>
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,SliderData>,_STL::_Select1st<_STL::pair<unsigned_long_const_,SliderData>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,SliderData>_>_>::_M_erase` · 004d6d00 · legions 004c9770 — Rust std replaces it

### _STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,FXScalarAndTarget>,_STL::_Select1st<_STL::pair<unsigned_long_const_,FXScalarAndTarget>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,FXScalarAndTarget>_>_>
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,FXScalarAndTarget>,_STL::_Select1st<_STL::pair<unsigned_long_const_,FXScalarAndTarget>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,FXScalarAndTarget>_>_>::_M_erase` · 004d6d60 · legions 004c97d0 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,CLostCell_*>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,CLostCell_*>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,CLostCell_*>_*>` · 004d6fd0 · legions 004c9a40 — Rust std replaces it

### IntrusiveHashTable<InstanceID,HashTableData<InstanceID,CBookKeep_*>_*>
- [-] `IntrusiveHashTable<InstanceID,HashTableData<InstanceID,CBookKeep_*>_*>::IntrusiveHashTable<InstanceID,HashTableData<InstanceID,CBookKeep_*>_*>` · 004d7060 · legions 004c9ad0 — Rust std replaces it

### IntrusiveHashTable<AsyncContext,HashTableData<AsyncContext,EntityCreationDesc_*>_*>
- [-] `IntrusiveHashTable<AsyncContext,HashTableData<AsyncContext,EntityCreationDesc_*>_*>::IntrusiveHashTable<AsyncContext,HashTableData<AsyncContext,EntityCreationDesc_*>_*>` · 004d70f0 · legions 004c9b60 — Rust std replaces it

### _STL::_Rb_tree<InstanceID,InstanceID,_STL::_Identity<InstanceID>,_STL::less<InstanceID>,_STL::allocator<InstanceID>_>
- [-] `_STL::_Rb_tree<InstanceID,InstanceID,_STL::_Identity<InstanceID>,_STL::less<InstanceID>,_STL::allocator<InstanceID>_>::erase` · 004d7180 · legions 004c9bf0 — Rust std replaces it
- [-] `_STL::_Rb_tree<InstanceID,InstanceID,_STL::_Identity<InstanceID>,_STL::less<InstanceID>,_STL::allocator<InstanceID>_>::insert_unique` · 004d7270 · legions 004c9ce0 — Rust std replaces it
- [-] `_STL::_Rb_tree<InstanceID,InstanceID,_STL::_Identity<InstanceID>,_STL::less<InstanceID>,_STL::allocator<InstanceID>_>::erase` · 004d78d0 · legions 004ca340 — Rust std replaces it

### _STL::deque<NetBlob_*,_STL::allocator<NetBlob_*>_>
- [-] `_STL::deque<NetBlob_*,_STL::allocator<NetBlob_*>_>::deque<NetBlob_*,_STL::allocator<NetBlob_*>_>` · 004d7930 · legions 004ca3a0 — Rust std replaces it

### _STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,SliderData>,_STL::_Select1st<_STL::pair<unsigned_long_const_,SliderData>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,SliderData>_>_>
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,SliderData>,_STL::_Select1st<_STL::pair<unsigned_long_const_,SliderData>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,SliderData>_>_>::_M_copy` · 004d7a00 · legions 004ca470 — Rust std replaces it

### _STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,FXScalarAndTarget>,_STL::_Select1st<_STL::pair<unsigned_long_const_,FXScalarAndTarget>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,FXScalarAndTarget>_>_>
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,FXScalarAndTarget>,_STL::_Select1st<_STL::pair<unsigned_long_const_,FXScalarAndTarget>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,FXScalarAndTarget>_>_>::_M_copy` · 004d7b00 · legions 004ca570 — Rust std replaces it

### _STL::_Rb_tree<InstanceID,_STL::pair<InstanceID_const_,ObjectEventQueue>,_STL::_Select1st<_STL::pair<InstanceID_const_,ObjectEventQueue>_>,_STL::less<InstanceID>,_STL::allocator<_STL::pair<InstanceID_const_,ObjectEventQueue>_>_>
- [-] `_STL::_Rb_tree<InstanceID,_STL::pair<InstanceID_const_,ObjectEventQueue>,_STL::_Select1st<_STL::pair<InstanceID_const_,ObjectEventQueue>_>,_STL::less<InstanceID>,_STL::allocator<_STL::pair<InstanceID_const_,ObjectEventQueue>_>_>::erase` · 004d7f00 · legions 004ca970 — Rust std replaces it
- [-] `_STL::_Rb_tree<InstanceID,_STL::pair<InstanceID_const_,ObjectEventQueue>,_STL::_Select1st<_STL::pair<InstanceID_const_,ObjectEventQueue>_>,_STL::less<InstanceID>,_STL::allocator<_STL::pair<InstanceID_const_,ObjectEventQueue>_>_>::_M_erase` · 004d7fe0 · legions 004caa50 — Rust std replaces it
- [-] `_STL::_Rb_tree<InstanceID,_STL::pair<InstanceID_const_,ObjectEventQueue>,_STL::_Select1st<_STL::pair<InstanceID_const_,ObjectEventQueue>_>,_STL::less<InstanceID>,_STL::allocator<_STL::pair<InstanceID_const_,ObjectEventQueue>_>_>::_M_create_node` · 004d89b0 · legions 004cb420 — Rust std replaces it
- [-] `_STL::_Rb_tree<InstanceID,_STL::pair<InstanceID_const_,ObjectEventQueue>,_STL::_Select1st<_STL::pair<InstanceID_const_,ObjectEventQueue>_>,_STL::less<InstanceID>,_STL::allocator<_STL::pair<InstanceID_const_,ObjectEventQueue>_>_>::_M_insert` · 004d8c30 · legions 004cb6a0 — Rust std replaces it
- [-] `_STL::_Rb_tree<InstanceID,_STL::pair<InstanceID_const_,ObjectEventQueue>,_STL::_Select1st<_STL::pair<InstanceID_const_,ObjectEventQueue>_>,_STL::less<InstanceID>,_STL::allocator<_STL::pair<InstanceID_const_,ObjectEventQueue>_>_>::insert_unique` · 004d8d10 · legions 004cb780 — Rust std replaces it
- [-] `_STL::_Rb_tree<InstanceID,_STL::pair<InstanceID_const_,ObjectEventQueue>,_STL::_Select1st<_STL::pair<InstanceID_const_,ObjectEventQueue>_>,_STL::less<InstanceID>,_STL::allocator<_STL::pair<InstanceID_const_,ObjectEventQueue>_>_>::insert_unique` · 004d9220 · legions 004cbc90 — Rust std replaces it

## `portal/client/src/objects/visualorderingqueue.cpp.c` (0/4)

### _STL
- [-] `_STL::__push_heap` · 004da360 · legions 004ccdc0 — Rust std replaces it
- [-] `_STL::__adjust_heap` · 004da3e0 · legions 004cce40 — Rust std replaces it

### _STL::priority_queue<StampedNetBlob,_STL::vector<StampedNetBlob,_STL::allocator<StampedNetBlob>_>,_STL::greater<StampedNetBlob>_>
- [-] `_STL::priority_queue<StampedNetBlob,_STL::vector<StampedNetBlob,_STL::allocator<StampedNetBlob>_>,_STL::greater<StampedNetBlob>_>::push` · 004da490 · legions 004ccef0 — Rust std replaces it
- [-] `_STL::priority_queue<StampedNetBlob,_STL::vector<StampedNetBlob,_STL::allocator<StampedNetBlob>_>,_STL::greater<StampedNetBlob>_>::pop` · 004da500 · legions 004ccf60 — Rust std replaces it

## `portal/client/src/render/core/d3d/renderd3d.cpp.c` (0/1)

### SmartArray<SpewBoxMessageInfo>
- [-] `SmartArray<SpewBoxMessageInfo>::grow` · 005d9be0 · legions 0057b370 — Rust std replaces it

## `portal/client/src/render/core/d3d/renderdeviced3d.cpp.c` (0/1)

### SmartArray<RenderLight>
- [-] `SmartArray<RenderLight>::grow` · 005c9760 · legions 00590730 — Rust std replaces it

## `portal/client/src/render/core/graphicsresource.cpp.c` (0/11)

### _STL
- [-] `_STL::__push_heap` · 005bc2d0 · legions 00585a20 — Rust std replaces it
- [-] `_STL::__unguarded_insertion_sort_aux` · 005bc480 · legions 00585bf0 — Rust std replaces it
- [-] `_STL::__adjust_heap` · 005bc4c0 · legions 00585c30 — Rust std replaces it
- [-] `_STL::__unguarded_partition` · 005bc540 · legions 00585cb0 — Rust std replaces it
- [-] `_STL::__make_heap` · 005bc590 · legions 00585d00 — Rust std replaces it
- [-] `_STL::sort_heap` · 005bc6d0 · legions 00585ed0 — Rust std replaces it
- [-] `_STL::__insertion_sort` · 005bc720 — Rust std replaces it
- [-] `_STL::__partial_sort` · 005bc7e0 · legions 00585f70 — Rust std replaces it
- [-] `_STL::__final_insertion_sort` · 005bc860 · legions 00585f20 — Rust std replaces it
- [-] `_STL::__introsort_loop` · 005bc8b0 · legions 00585ff0 — Rust std replaces it
- [-] `_STL::sort` · 005bc9b0 · legions 005860f0 — Rust std replaces it

## `portal/client/src/render/core/texture/rendertexture.cpp.c` (0/1)

### SmartArray<DBLevelInfo>
- [-] `SmartArray<DBLevelInfo>::grow` · 005bd040 · legions 005866e0 — Rust std replaces it

## `portal/client/src/render/decal/decaleffect.cpp.c` (0/1)

### SmartArray<DecalTriangle>
- [-] `SmartArray<DecalTriangle>::grow` · 005de6f0 · legions 0059f480 — Rust std replaces it

## `portal/client/src/render/material/materiallayer.cpp.c` (0/3)

### SmartArray<LScapeFarBlockEdge>
- [-] `SmartArray<LScapeFarBlockEdge>::grow` · 005cb900 · legions 005586d0 — Rust std replaces it

### SmartArray<MaterialLayer::ShaderInfo>
- [-] `SmartArray<MaterialLayer::ShaderInfo>::Reset` · 005cbc50 · legions 00592b80 — Rust std replaces it
- [-] `SmartArray<MaterialLayer::ShaderInfo>::grow` · 005d0190 · legions 00593880 — Rust std replaces it

## `portal/client/src/render/particle/particleemitter.cpp.c` (0/4)

### SmartArray<ParticleKeyframe>
- [-] `SmartArray<ParticleKeyframe>::grow` · 005d3380 · legions 00596a70 — Rust std replaces it

### _STL
- [-] `_STL::__find_if` · 005d3430 · legions 00596b20 — Rust std replaces it
- [-] `_STL::__find_if` · 005d34f0 · legions 00596be0 — Rust std replaces it
- [-] `_STL::__find_if` · 005d35b0 · legions 00596ca0 — Rust std replaces it

## `portal/client/src/render/particle/particlesystem.cpp.c` (0/2)

### SmartArray<ParticleSystem::ParticleRenderPos>
- [-] `SmartArray<ParticleSystem::ParticleRenderPos>::Reset` · 005c2560 · legions 00589760 — Rust std replaces it
- [-] `SmartArray<ParticleSystem::ParticleRenderPos>::grow` · 005c2960 · legions 00589b30 — Rust std replaces it

## `portal/client/src/render/particle/psmanager.cpp.c` (0/2)

### SmartArray<PSManager::AutonomousSystemInfo>
- [-] `SmartArray<PSManager::AutonomousSystemInfo>::Reset` · 005c06b0 · legions 00587950 — Rust std replaces it
- [-] `SmartArray<PSManager::AutonomousSystemInfo>::grow` · 005c08e0 · legions 00587b70 — Rust std replaces it

## `portal/client/src/render/streak/streakeffect.cpp.c` (0/1)

### SmartArray<StreakSegment>
- [-] `SmartArray<StreakSegment>::grow` · 005c1df0 · legions 00589030 — Rust std replaces it

## `portal/client/src/render/ui/renderui.cpp.c` (0/3)

### List<InstanceID>
- [-] `List<InstanceID>::push_back` · 005c4200 · legions 0058b310 — Rust std replaces it

### List<GameplayChildInfo>
- [-] `List<GameplayChildInfo>::insert` · 005c4260 · legions 0058b370 — Rust std replaces it

### List<unsigned___int64>
- [-] `List<unsigned___int64>::remove` · 005c44a0 · legions 00531ed0 — Rust std replaces it

## `portal/client/src/render/ui/uiscene.cpp.c` (0/1)

### SmartArray<EntityDesc>
- [-] `SmartArray<EntityDesc>::grow` · 005acc30 · legions 00578b90 — Rust std replaces it

## `portal/client/src/scenerender/scenetool.cpp.c` (0/15)

### SmartArray<ObjectSelection>
- [-] `SmartArray<ObjectSelection>::grow` · 0056c2f0 · legions 0053c6d0 — Rust std replaces it

### _STL
- [-] `_STL::__median` · 0056c3d0 · legions 0053c7b0 — Rust std replaces it
- [-] `_STL::__unguarded_linear_insert` · 0056c480 · legions 0053c860 — Rust std replaces it
- [-] `_STL::__push_heap` · 0056c4e0 · legions 0053c8c0 — Rust std replaces it

### SmartArray<PixelScanResult>
- [-] `SmartArray<PixelScanResult>::grow` · 0056e8c0 · legions 0053eca0 — Rust std replaces it

### _STL
- [-] `_STL::__adjust_heap` · 0056e980 · legions 0053ed60 — Rust std replaces it
- [-] `_STL::__unguarded_partition` · 0056f630 · legions 0053fa70 — Rust std replaces it
- [-] `_STL::__unguarded_insertion_sort` · 0056f6e0 · legions 0053fb20 — Rust std replaces it
- [-] `_STL::__make_heap` · 0056f710 · legions 0053fb50 — Rust std replaces it
- [-] `_STL::__linear_insert` · 005708f0 · legions 00540da0 — Rust std replaces it
- [-] `_STL::__insertion_sort` · 005719b0 · legions 00541e60 — Rust std replaces it
- [-] `_STL::sort_heap` · 005719f0 · legions 00541ea0 — Rust std replaces it
- [-] `_STL::__partial_sort` · 00571bf0 · legions 005420a0 — Rust std replaces it
- [-] `_STL::__introsort_loop` · 00571ca0 · legions 00542150 — Rust std replaces it
- [-] `_STL::sort` · 00571d30 · legions 005421e0 — Rust std replaces it

## `portal/client/src/smartbox/camera/camerafxmanager.cpp.c` (0/2)

### List<CameraFxTrack>
- [-] `List<CameraFxTrack>::insert` · 00578980 · legions 005482e0 — Rust std replaces it
- [-] `List<CameraFxTrack>::remove` · 00578cb0 · legions 00548610 — Rust std replaces it

## `portal/client/src/smartbox/camera/cameramanager.cpp.c` (0/5)

### _STL::_Deque_iterator_base<_STL::pair<long,long>_>
- [-] `_STL::_Deque_iterator_base<_STL::pair<long,long>_>::_M_advance` · 00576080 · legions 00545f40 — Rust std replaces it

### List<CameraFxTrack>
- [-] `List<CameraFxTrack>::pop_head` · 005770a0 · legions 00546f90 — Rust std replaces it
- [-] `List<CameraFxTrack>::flush` · 00577460 · legions 00547350 — Rust std replaces it

### _STL::_Deque_base<_STL::pair<long,long>,_STL::allocator<_STL::pair<long,long>_>_>
- [-] `_STL::_Deque_base<_STL::pair<long,long>,_STL::allocator<_STL::pair<long,long>_>_>::_M_initialize_map` · 005774c0 · legions 00547510 — Rust std replaces it

### _STL::deque<_STL::pair<long,long>,_STL::allocator<_STL::pair<long,long>_>_>
- [-] `_STL::deque<_STL::pair<long,long>,_STL::allocator<_STL::pair<long,long>_>_>::_M_push_front_aux_v` · 005777d0 · legions 00547770 — Rust std replaces it

## `portal/client/src/smartbox/worldmanager/lscapemanager.cpp.c` (1/3)

### List<ItemDisplay_*>
- [x] `List<ItemDisplay_*>::insert` · 0057a700 · legions 005f8dd0 — ac2-script glue::lists

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,CLandCell_*>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,CLandCell_*>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,CLandCell_*>_*>` · 0057b090 · legions 0054a970 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,CLandBlock_*>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,CLandBlock_*>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,CLandBlock_*>_*>` · 0057b120 · legions 0054aa00 — Rust std replaces it

## `portal/client/src/smartbox/worldrender/frillrender.cpp.c` (0/21)

### SmartArray<FrillRefType>
- [-] `SmartArray<FrillRefType>::grow` · 00580a20 · legions 005502d0 — Rust std replaces it

### SmartArray<PhysObjBrush>
- [-] `SmartArray<PhysObjBrush>::grow` · 00580ad0 · legions 0047b760 — Rust std replaces it

### _STL
- [-] `_STL::__unguarded_linear_insert` · 00580b80 · legions 00550380 — Rust std replaces it
- [-] `_STL::__push_heap` · 00580bc0 · legions 005503c0 — Rust std replaces it

### SmartArray<Matrix4>
- [-] `SmartArray<Matrix4>::grow` · 00580d80 · legions 00550580 — Rust std replaces it

### SmartArray<RGBAColor>
- [-] `SmartArray<RGBAColor>::grow` · 00580e30 · legions 00550630 — Rust std replaces it

### SmartArray<FrillEntry>
- [-] `SmartArray<FrillEntry>::Reset` · 00580f00 · legions 00550700 — Rust std replaces it

### _STL
- [-] `_STL::__unguarded_insertion_sort_aux` · 00580f50 · legions 00550750 — Rust std replaces it
- [-] `_STL::__adjust_heap` · 00580fc0 · legions 005507c0 — Rust std replaces it
- [-] `_STL::__copy_backward_ptrs` · 00581060 · legions 00550860 — Rust std replaces it

### SmartArray<FrillEntry>
- [-] `SmartArray<FrillEntry>::grow` · 00581600 · legions 00550e00 — Rust std replaces it

### _STL
- [-] `_STL::__unguarded_partition` · 005816e0 · legions 00550ee0 — Rust std replaces it
- [-] `_STL::__make_heap` · 00581750 · legions 00550f50 — Rust std replaces it
- [-] `_STL::sort_heap` · 00581c60 · legions 00551420 — Rust std replaces it
- [-] `_STL::__insertion_sort` · 00582020 · legions 005517e0 — Rust std replaces it
- [-] `_STL::__partial_sort` · 005820b0 · legions 00551870 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,FrillListType_*>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,FrillListType_*>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,FrillListType_*>_*>` · 00582240 · legions 005519f0 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,FrillRenderType_*>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,FrillRenderType_*>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,FrillRenderType_*>_*>` · 005822d0 · legions 00551a80 — Rust std replaces it

### _STL
- [-] `_STL::__final_insertion_sort` · 00582360 · legions 00551b10 — Rust std replaces it
- [-] `_STL::__introsort_loop` · 00582550 · legions 00551d00 — Rust std replaces it
- [-] `_STL::sort` · 00582700 · legions 00551ea0 — Rust std replaces it

## `portal/client/src/smartbox/worldrender/lscaperender.cpp.c` (0/1)

### SmartArray<RainSegment>
- [-] `SmartArray<RainSegment>::grow` · 00584460 · legions 00553ba0 — Rust std replaces it

## `portal/client/src/smartbox/worldrender/lscapeterrainmesh.cpp.c` (0/9)

### SmartArray<ShoreStripType>
- [-] `SmartArray<ShoreStripType>::Reset` · 00588e90 · legions 00558530 — Rust std replaces it

### SmartArray<short>
- [-] `SmartArray<short>::grow` · 00588ef0 · legions 00558590 — Rust std replaces it

### SmartArray<LScapeFarBlock>
- [-] `SmartArray<LScapeFarBlock>::grow` · 00588f80 · legions 00558620 — Rust std replaces it

### SmartArray<ShorePointType>
- [-] `SmartArray<ShorePointType>::grow` · 00589030 · legions 00558780 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,SmartArray<unsigned_short>_>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,SmartArray<unsigned_short>_>_*>::delete_contents` · 0058a370 · legions 006746c0 — Rust std replaces it

### SmartArray<ShoreStripType>
- [-] `SmartArray<ShoreStripType>::grow` · 0058ab20 · legions 0055a1a0 — Rust std replaces it

### SmartArray<LScapeFarChunk>
- [-] `SmartArray<LScapeFarChunk>::grow` · 0058b7d0 · legions 0055ae40 — Rust std replaces it

### HashTable<unsigned_long,SmartArray<unsigned_short>_>
- [-] `HashTable<unsigned_long,SmartArray<unsigned_short>_>::add` · 0058bb90 · legions 0055b200 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,SmartArray<unsigned_short>_>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,SmartArray<unsigned_short>_>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,SmartArray<unsigned_short>_>_*>` · 0058e8b0 · legions 0055df20 — Rust std replaces it

## `portal/client/src/smartbox/worldrender/worldrender.cpp.c` (0/13)

### _STL
- [-] `_STL::__unguarded_linear_insert` · 0057ced0 · legions 0054c7a0 — Rust std replaces it
- [-] `_STL::__push_heap` · 0057cf10 · legions 0054c7e0 — Rust std replaces it
- [-] `_STL::__unguarded_insertion_sort_aux` · 0057d710 · legions 0054cfe0 — Rust std replaces it
- [-] `_STL::__adjust_heap` · 0057d780 · legions 0054d050 — Rust std replaces it
- [-] `_STL::__copy_backward_ptrs` · 0057d820 · legions 0054d0f0 — Rust std replaces it
- [-] `_STL::__unguarded_partition` · 0057e070 · legions 0054d940 — Rust std replaces it
- [-] `_STL::__make_heap` · 0057e0e0 · legions 0054d9b0 — Rust std replaces it
- [-] `_STL::sort_heap` · 0057e980 · legions 0054e250 — Rust std replaces it
- [-] `_STL::__insertion_sort` · 0057f630 · legions 0054ef00 — Rust std replaces it
- [-] `_STL::__partial_sort` · 0057f6c0 · legions 0054ef90 — Rust std replaces it
- [-] `_STL::__final_insertion_sort` · 0057f780 · legions 0054f050 — Rust std replaces it
- [-] `_STL::__introsort_loop` · 0057f7e0 · legions 0054f0b0 — Rust std replaces it
- [-] `_STL::sort` · 0057f900 · legions 0054f1d0 — Rust std replaces it

## `portal/client/src/sound/directx/directxmusicloader.cpp.c` (0/7)

### IntrusiveHashTable<_GUID,HashTableData<_GUID,IStream_*>_*>
- [-] `IntrusiveHashTable<_GUID,HashTableData<_GUID,IStream_*>_*>::find_internal` · 005a7400 · legions 00573a80 — Rust std replaces it
- [-] `IntrusiveHashTable<_GUID,HashTableData<_GUID,IStream_*>_*>::delete_contents` · 005a7450 · legions 00573ad0 — Rust std replaces it

### HashTable<_GUID,IStream_*>
- [-] `HashTable<_GUID,IStream_*>::find` · 005a7d10 · legions 005742a0 — Rust std replaces it
- [-] `HashTable<_GUID,IStream_*>::contains` · 005a7d40 · legions 005742d0 — Rust std replaces it

### IntrusiveHashTable<_GUID,HashTableData<_GUID,IStream_*>_*>
- [-] `IntrusiveHashTable<_GUID,HashTableData<_GUID,IStream_*>_*>::add` · 005a7d70 · legions 00574300 — Rust std replaces it

### HashTable<_GUID,IStream_*>
- [-] `HashTable<_GUID,IStream_*>::add` · 005a7dd0 · legions 00574360 — Rust std replaces it

### IntrusiveHashTable<_GUID,HashTableData<_GUID,IStream_*>_*>
- [-] `IntrusiveHashTable<_GUID,HashTableData<_GUID,IStream_*>_*>::IntrusiveHashTable<_GUID,HashTableData<_GUID,IStream_*>_*>` · 005a84f0 · legions 00574a50 — Rust std replaces it

## `portal/client/src/sound/directx/directxsounddevice.cpp.c` (0/5)

### _STL::deque<IDirectMusicAudioPath_*,_STL::allocator<IDirectMusicAudioPath_*>_>
- [-] `_STL::deque<IDirectMusicAudioPath_*,_STL::allocator<IDirectMusicAudioPath_*>_>::size` · 005a4ab0 · legions 00571260 — Rust std replaces it
- [-] `_STL::deque<IDirectMusicAudioPath_*,_STL::allocator<IDirectMusicAudioPath_*>_>::clear` · 005a4d70 · legions 00571520 — Rust std replaces it
- [-] `_STL::deque<IDirectMusicAudioPath_*,_STL::allocator<IDirectMusicAudioPath_*>_>::pop_front` · 005a5040 · legions 005717f0 — Rust std replaces it

### _STL::deque<NetBlob_*,_STL::allocator<NetBlob_*>_>
- [-] `_STL::deque<NetBlob_*,_STL::allocator<NetBlob_*>_>::_M_push_back_aux_v` · 005a5090 · legions 00571840 — Rust std replaces it

### _STL::deque<IDirectMusicAudioPath_*,_STL::allocator<IDirectMusicAudioPath_*>_>
- [-] `_STL::deque<IDirectMusicAudioPath_*,_STL::allocator<IDirectMusicAudioPath_*>_>::_M_push_front_aux_v` · 005a5100 · legions 005718b0 — Rust std replaces it

## `portal/client/src/sound/directx/dxmusicmessagethread.cpp.c` (0/5)

### _STL
- [-] `_STL::__find_if` · 005a86b0 · legions 00574c10 — Rust std replaces it
- [-] `_STL::__find_if` · 005a8730 · legions 00574c90 — Rust std replaces it
- [-] `_STL::find_if` · 005a8910 · legions 00574e70 — Rust std replaces it
- [-] `_STL::__destroy_aux` · 005a8a90 · legions 00546a00 — Rust std replaces it

### _STL::deque<DXMusicMessageThread::MusicEvent,_STL::allocator<DXMusicMessageThread::MusicEvent>_>
- [-] `_STL::deque<DXMusicMessageThread::MusicEvent,_STL::allocator<DXMusicMessageThread::MusicEvent>_>::_M_push_back_aux_v` · 005a8d60 · legions 005753a0 — Rust std replaces it

## `portal/client/src/sound/soundmanager.cpp.c` (0/16)

### (free functions)
- [-] `_PlayingSoundSorter` · 005a10f0 — Rust std replaces it
- [-] `_SoundInstanceLRU` · 005a1130 — Rust std replaces it

### _STL
- [-] `_STL::__unguarded_linear_insert` · 005a1680 · legions 00611370 — Rust std replaces it
- [-] `_STL::__push_heap` · 005a16c0 · legions 0056e380 — Rust std replaces it
- [-] `_STL::__unguarded_insertion_sort_aux` · 005a1940 · legions 0056e630 — Rust std replaces it
- [-] `_STL::__unguarded_partition` · 005a19a0 · legions 00612120 — Rust std replaces it
- [-] `_STL::__make_heap` · 005a1a10 · legions 00612190 — Rust std replaces it

### HashTable<AsyncContext,SoundManager::SoundRequest>
- [-] `HashTable<AsyncContext,SoundManager::SoundRequest>::add` · 005a2260 · legions 0056eed0 — Rust std replaces it

### _STL
- [-] `_STL::__final_insertion_sort` · 005a22c0 · legions 0056ef30 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,SoundManager::SoundInstanceInfo_*>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,SoundManager::SoundInstanceInfo_*>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,SoundManager::SoundInstanceInfo_*>_*>` · 005a2310 · legions 0056f000 — Rust std replaces it

### IntrusiveHashTable<AsyncContext,HashTableData<AsyncContext,SoundManager::SoundRequest>_*>
- [-] `IntrusiveHashTable<AsyncContext,HashTableData<AsyncContext,SoundManager::SoundRequest>_*>::IntrusiveHashTable<AsyncContext,HashTableData<AsyncContext,SoundManager::SoundRequest>_*>` · 005a23a0 · legions 0056f090 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PlayingSound_*>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PlayingSound_*>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PlayingSound_*>_*>` · 005a2430 · legions 0056f120 — Rust std replaces it

### _STL
- [-] `_STL::__introsort_loop` · 005a24c0 · legions 0056f1b0 — Rust std replaces it
- [-] `_STL::__introsort_loop` · 005a2550 · legions 0056f240 — Rust std replaces it
- [-] `_STL::sort` · 005a25e0 · legions 0056f2d0 — Rust std replaces it
- [-] `_STL::sort` · 005a2630 · legions 0056f320 — Rust std replaces it

## `portal/client/src/uielement/desc/elementdesc.cpp.c` (0/1)

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,StateDesc_*>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,StateDesc_*>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,StateDesc_*>_*>` · 005e0280 · legions 005a1010 — Rust std replaces it

## `portal/client/src/uielement/desc/layoutdesc.cpp.c` (0/1)

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,ElementDesc_*>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,ElementDesc_*>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,ElementDesc_*>_*>` · 005df9f0 · legions 005a0780 — Rust std replaces it

## `portal/client/src/uielement/framework/uiflow.cpp.c` (0/1)

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,UIMainFramework_*_(__cdecl*)(void)>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,UIMainFramework_*_(__cdecl*)(void)>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,UIMainFramework_*_(__cdecl*)(void)>_*>` · 005fd740 · legions 005be270 — Rust std replaces it

## `portal/client/src/uielement/framework/uipersistantdata.cpp.c` (0/1)

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,CharGenResult_*>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,CharGenResult_*>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,CharGenResult_*>_*>` · 005fe590 · legions 005bf070 — Rust std replaces it

## `portal/client/src/uielement/prim/node.cpp.c` (0/1)

### List<DualHashData<PStringBase<char>,CheckpointExportData>_*>
- [-] `List<DualHashData<PStringBase<char>,CheckpointExportData>_*>::push_back` · 00735860 · legions 005a56b0 — Rust std replaces it

## `portal/client/src/uielement/system/uielement.cpp.c` (0/6)

### HashTable<unsigned_long,unsigned_long>
- [-] `HashTable<unsigned_long,unsigned_long>::set` · 005e9db0 · legions 0040ba30 — Rust std replaces it

### HashTable<unsigned_long,PStringBase<char>_>
- [-] `HashTable<unsigned_long,PStringBase<char>_>::set` · 005e9e40 · legions 005aaa10 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PStringBase<char>_*>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PStringBase<char>_*>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PStringBase<char>_*>_*>` · 005eaea0 · legions 005aba70 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PStringBase<unsigned_short>_>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PStringBase<unsigned_short>_>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PStringBase<unsigned_short>_>_*>` · 005eaf30 · legions 005abb00 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PStringBase<char>_*>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PStringBase<char>_*>_*>::delete_contents` · 005ebf00 · legions 005acad0 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PStringBase<unsigned_short>_>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PStringBase<unsigned_short>_>_*>::delete_contents` · 005ebfb0 · legions 005acb80 — Rust std replaces it

## `portal/client/src/uielement/system/uielementmanager.cpp.c` (0/7)

### SmartArray<unsigned_long>
- [-] `SmartArray<unsigned_long>::add_unique` · 005e5530 · legions 005a6280 — Rust std replaces it

### List<UIElement_*>
- [-] `List<UIElement_*>::push_unique_back` · 005e5740 · legions 005a6490 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,List<UIElement_*>_*>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,List<UIElement_*>_*>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,List<UIElement_*>_*>_*>` · 005e6ca0 · legions 005a79e0 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,LayoutDesc_*>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,LayoutDesc_*>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,LayoutDesc_*>_*>` · 005e6d30 · legions 005a7a70 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,UIElement_*_(__cdecl*)(ElementDesc_*)>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,UIElement_*_(__cdecl*)(ElementDesc_*)>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,UIElement_*_(__cdecl*)(ElementDesc_*)>_*>` · 005e6dc0 · legions 005a7b00 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,HashTable<unsigned_long,SmartArray<UIListener_*>_*>_*>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,HashTable<unsigned_long,SmartArray<UIListener_*>_*>_*>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,HashTable<unsigned_long,SmartArray<UIListener_*>_*>_*>_*>` · 005e6e50 · legions 005a7b90 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,SmartArray<UIListener_*>_*>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,SmartArray<UIListener_*>_*>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,SmartArray<UIListener_*>_*>_*>` · 005e6ee0 · legions 005a7c20 — Rust std replaces it

## `portal/client/src/uielement/system/uilistener.cpp.c` (0/5)

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,HashTable<unsigned_long,long>_>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,HashTable<unsigned_long,long>_>_*>::delete_contents` · 005e8060 · legions 005a8cc0 — Rust std replaces it

### HashTable<unsigned_long,HashTable<unsigned_long,long>_>
- [-] `HashTable<unsigned_long,HashTable<unsigned_long,long>_>::erase` · 005e8250 · legions 005a8eb0 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,HashTable<unsigned_long,long>_>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,HashTable<unsigned_long,long>_>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,HashTable<unsigned_long,long>_>_*>` · 005e82b0 · legions 005a8f10 — Rust std replaces it

### _STL::vector<StampedNetBlob,_STL::allocator<StampedNetBlob>_>
- [-] `_STL::vector<StampedNetBlob,_STL::allocator<StampedNetBlob>_>::_M_insert_overflow` · 005e83f0 · legions 005a9050 — Rust std replaces it

### HashTable<unsigned_long,HashTable<unsigned_long,long>_>
- [-] `HashTable<unsigned_long,HashTable<unsigned_long,long>_>::add` · 005e88e0 · legions 005a9540 — Rust std replaces it

## `portal/client/src/uielement/types/text/glyphlist.cpp.c` (0/4)

### List<Glyph>
- [-] `List<Glyph>::get_node` · 005fc210 · legions 005bcd40 — Rust std replaces it
- [-] `List<Glyph>::remove` · 005fc4b0 · legions 005bcfe0 — Rust std replaces it
- [-] `List<Glyph>::push` · 005fc520 · legions 005bd050 — Rust std replaces it
- [-] `List<Glyph>::flush` · 005fc660 · legions 005bd190 — Rust std replaces it

## `portal/client/src/uielement/types/text/uielement_text.cpp.c` (0/1)

### SmartArray<GlyphLine>
- [-] `SmartArray<GlyphLine>::grow` · 005ee950 · legions 005af4e0 — Rust std replaces it

## `portal/client/src/weenie/interp/glue/cnativefactory.cpp.c` (0/73)

### _STL
- [-] `_STL::fill` · 004a6000 · legions 0049d080 — Rust std replaces it
- [-] `_STL::uninitialized_fill_n` · 004a6f50 · legions 0049df90 — Rust std replaces it

### SmartArray<TimeOfDay>
- [-] `SmartArray<TimeOfDay>::grow` · 004a6f70 · legions 0049dfb0 — Rust std replaces it

### SmartArray<WeekDay>
- [-] `SmartArray<WeekDay>::grow` · 004a7060 · legions 0049e0a0 — Rust std replaces it

### SmartArray<Season>
- [-] `SmartArray<Season>::grow` · 004a7130 · legions 0049e170 — Rust std replaces it

### SmartArray<BaseProperty>
- [-] `SmartArray<BaseProperty>::grow` · 004a73a0 · legions 0049e320 — Rust std replaces it

### SmartArray<PropertyGroup>
- [-] `SmartArray<PropertyGroup>::grow` · 004a7480 · legions 0049e400 — Rust std replaces it

### SmartArray<RandomSelectionTable<unsigned_long>::ElementStruct>
- [-] `SmartArray<RandomSelectionTable<unsigned_long>::ElementStruct>::grow` · 004a7560 · legions 0049e4e0 — Rust std replaces it

### PStringBase<char>
- [-] `PStringBase<char>::PStringBase<char>` · 004a78b0 · legions 0049e890 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,AppearanceInfo>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,AppearanceInfo>_*>::delete_contents` · 004a7d00 · legions 0049ece0 — Rust std replaces it

### SmartArray<CollideProperty>
- [-] `SmartArray<CollideProperty>::grow` · 004a7e30 · legions 0049ee90 — Rust std replaces it

### _STL::_Hashtable_iterator<_STL::pair<unsigned___int64_const_,unsigned_long>,unsigned___int64,_STL::hash<unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned_long>_>,_STL::equal_to<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned_long>_>_>
- [-] `_STL::_Hashtable_iterator<_STL::pair<unsigned___int64_const_,unsigned_long>,unsigned___int64,_STL::hash<unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned_long>_>,_STL::equal_to<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned_long>_>_>::_M_skip_to_next` · 004a7f40 · legions 0049efa0 — Rust std replaces it

### PStringBase<unsigned_short>
- [-] `PStringBase<unsigned_short>::decrypt` · 004a7f80 · legions 0049efe0 — Rust std replaces it
- [-] `PStringBase<unsigned_short>::encrypt` · 004a7fc0 · legions 0049f020 — Rust std replaces it

### List<Ray>
- [-] `List<Ray>::push_back` · 004a82e0 · legions 0049f340 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PStringBase<char>_>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PStringBase<char>_>_*>::delete_contents` · 004a90e0 · legions 004a00e0 — Rust std replaces it

### HashTableData<PStringBase<unsigned_short>,Position>
- [-] `HashTableData<PStringBase<unsigned_short>,Position>::HashTableData<PStringBase<unsigned_short>,Position>` · 004a9190 · legions 004a0190 — Rust std replaces it

### _STL::_Hashtable_iterator<_STL::pair<unsigned_long_const_,unsigned_long>,unsigned_long,_STL::hash<unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned_long>_>,_STL::equal_to<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>
- [-] `_STL::_Hashtable_iterator<_STL::pair<unsigned_long_const_,unsigned_long>,unsigned_long,_STL::hash<unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned_long>_>,_STL::equal_to<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>::_M_skip_to_next` · 004a91f0 · legions 004a01f0 — Rust std replaces it

### SmartArray<ObjCollisionProfile>
- [-] `SmartArray<ObjCollisionProfile>::grow` · 004a92d0 · legions 004a02d0 — Rust std replaces it

### IntrusiveHashTable<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,Position>_*>
- [-] `IntrusiveHashTable<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,Position>_*>::delete_contents` · 004aa4f0 · legions 004a1560 — Rust std replaces it

### _STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned_long>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned_long>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>::_M_erase` · 004aa970 · legions 004a1b60 — Rust std replaces it

### _STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned___int64>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned___int64>_>_>
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned___int64>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned___int64>_>_>::_M_erase` · 004aaab0 · legions 004a1c80 — Rust std replaces it

### _STL::_Rb_tree<unsigned___int64,_STL::pair<unsigned___int64_const_,unsigned_long>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned_long>_>,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned_long>_>_>
- [-] `_STL::_Rb_tree<unsigned___int64,_STL::pair<unsigned___int64_const_,unsigned_long>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned_long>_>,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned_long>_>_>::_M_erase` · 004aacd0 · legions 004a1f50 — Rust std replaces it

### _STL::_Rb_tree<unsigned___int64,_STL::pair<unsigned___int64_const_,unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned___int64>_>,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned___int64>_>_>
- [-] `_STL::_Rb_tree<unsigned___int64,_STL::pair<unsigned___int64_const_,unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned___int64>_>,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned___int64>_>_>::_M_erase` · 004aad20 · legions 004a1fa0 — Rust std replaces it

### _STL::vector<void*,_STL::allocator<void*>_>
- [-] `_STL::vector<void*,_STL::allocator<void*>_>::_M_allocate_and_copy` · 004ab0a0 · legions 004a22c0 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,AppearanceInfo>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,AppearanceInfo>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,AppearanceInfo>_*>` · 004abb20 · legions 004a2c80 — Rust std replaces it

### IntrusiveHashTable<unsigned___int64,HashSetData<unsigned___int64>_*>
- [-] `IntrusiveHashTable<unsigned___int64,HashSetData<unsigned___int64>_*>::IntrusiveHashTable<unsigned___int64,HashSetData<unsigned___int64>_*>` · 004abbb0 · legions 004a2d10 — Rust std replaces it

### IntrusiveHashTable<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,Position>_*>
- [-] `IntrusiveHashTable<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,Position>_*>::IntrusiveHashTable<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,Position>_*>` · 004abd90 · legions 004a2ef0 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashSetData<unsigned_long>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashSetData<unsigned_long>_*>::IntrusiveHashTable<unsigned_long,HashSetData<unsigned_long>_*>` · 004abe20 · legions 004a2f80 — Rust std replaces it

### IntrusiveHashTable<unsigned___int64,HashTableData<unsigned___int64,unsigned___int64>_*>
- [-] `IntrusiveHashTable<unsigned___int64,HashTableData<unsigned___int64,unsigned___int64>_*>::IntrusiveHashTable<unsigned___int64,HashTableData<unsigned___int64,unsigned___int64>_*>` · 004abeb0 · legions 004a3010 — Rust std replaces it

### IntrusiveHashTable<ReferenceIDWrapper,HashTableData<ReferenceIDWrapper,unsigned_long>_*>
- [-] `IntrusiveHashTable<ReferenceIDWrapper,HashTableData<ReferenceIDWrapper,unsigned_long>_*>::IntrusiveHashTable<ReferenceIDWrapper,HashTableData<ReferenceIDWrapper,unsigned_long>_*>` · 004abf40 · legions 004a30a0 — Rust std replaces it

### _STL::vector<void_*,_STL::allocator<void_*>_>
- [-] `_STL::vector<void_*,_STL::allocator<void_*>_>::reserve` · 004abfd0 · legions 004a3130 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PStringBase<char>_>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PStringBase<char>_>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PStringBase<char>_>_*>` · 004ac3e0 · legions 004a35b0 — Rust std replaces it

### _STL::_Rb_tree<QualifiedDataID,QualifiedDataID,_STL::_Identity<QualifiedDataID>,_STL::less<QualifiedDataID>,_STL::allocator<QualifiedDataID>_>
- [-] `_STL::_Rb_tree<QualifiedDataID,QualifiedDataID,_STL::_Identity<QualifiedDataID>,_STL::less<QualifiedDataID>,_STL::allocator<QualifiedDataID>_>::_M_copy` · 004ac470 · legions 004a3640 — Rust std replaces it

### _STL::vector<void_*,_STL::allocator<void_*>_>
- [-] `_STL::vector<void_*,_STL::allocator<void_*>_>::_M_fill_insert` · 004ac550 · legions 004a3720 — Rust std replaces it

### _STL::hashtable<_STL::pair<unsigned_long_const_,unsigned_long>,unsigned_long,_STL::hash<unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned_long>_>,_STL::equal_to<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>
- [-] `_STL::hashtable<_STL::pair<unsigned_long_const_,unsigned_long>,unsigned_long,_STL::hash<unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned_long>_>,_STL::equal_to<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>::clear` · 004ac670 · legions 004a3840 — Rust std replaces it

### _STL::hashtable<_STL::pair<unsigned_long_const_,unsigned___int64>,unsigned_long,_STL::hash<unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned___int64>_>,_STL::equal_to<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned___int64>_>_>
- [-] `_STL::hashtable<_STL::pair<unsigned_long_const_,unsigned___int64>,unsigned_long,_STL::hash<unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned___int64>_>,_STL::equal_to<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned___int64>_>_>::clear` · 004ac6f0 · legions 004a38c0 — Rust std replaces it

### _STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned_long>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned_long>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>::_M_copy` · 004ac8b0 · legions 004a3a60 — Rust std replaces it

### _STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned___int64>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned___int64>_>_>
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned___int64>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned___int64>_>_>::_M_copy` · 004aca00 · legions 004a3b80 — Rust std replaces it

### _STL::_Rb_tree<unsigned___int64,_STL::pair<unsigned___int64_const_,unsigned_long>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned_long>_>,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned_long>_>_>
- [-] `_STL::_Rb_tree<unsigned___int64,_STL::pair<unsigned___int64_const_,unsigned_long>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned_long>_>,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned_long>_>_>::_M_copy` · 004acb60 · legions 004a3cb0 — Rust std replaces it

### _STL::_Rb_tree<unsigned___int64,_STL::pair<unsigned___int64_const_,unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned___int64>_>,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned___int64>_>_>
- [-] `_STL::_Rb_tree<unsigned___int64,_STL::pair<unsigned___int64_const_,unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned___int64>_>,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned___int64>_>_>::_M_copy` · 004acc50 · legions 004a3da0 — Rust std replaces it

### List<Ray>
- [-] `List<Ray>::List<Ray>` · 004acdc0 · legions 004a3f10 — Rust std replaces it

### IntrusiveHashTable<ReferenceIDWrapper,HashTableData<ReferenceIDWrapper,unsigned_long>_*>
- [-] `IntrusiveHashTable<ReferenceIDWrapper,HashTableData<ReferenceIDWrapper,unsigned_long>_*>::add_for_unpacking` · 004ae070 · legions 004a51c0 — Rust std replaces it
- [-] `IntrusiveHashTable<ReferenceIDWrapper,HashTableData<ReferenceIDWrapper,unsigned_long>_*>::resize_internal` · 004ae100 · legions 004a5250 — Rust std replaces it

### IntrusiveHashTable<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,Position>_*>
- [-] `IntrusiveHashTable<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,Position>_*>::add_for_unpacking` · 004ae480 · legions 004a55d0 — Rust std replaces it

### _STL::_Rb_tree<QualifiedDataID,QualifiedDataID,_STL::_Identity<QualifiedDataID>,_STL::less<QualifiedDataID>,_STL::allocator<QualifiedDataID>_>
- [-] `_STL::_Rb_tree<QualifiedDataID,QualifiedDataID,_STL::_Identity<QualifiedDataID>,_STL::less<QualifiedDataID>,_STL::allocator<QualifiedDataID>_>::_Rb_tree<QualifiedDataID,QualifiedDataID,_STL::_Identity<QualifiedDataID>,_STL::less<QualifiedDataID>,_STL::allocator<QualifiedDataID>_>` · 004ae850 · legions 004a5ad0 — Rust std replaces it

### _STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned_long>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned_long>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned_long>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>` · 004ae900 · legions 004a5b80 — Rust std replaces it

### _STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned___int64>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned___int64>_>_>
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned___int64>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned___int64>_>_>::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned___int64>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned___int64>_>_>` · 004ae9b0 · legions 004a5c30 — Rust std replaces it

### _STL::_Rb_tree<unsigned___int64,_STL::pair<unsigned___int64_const_,unsigned_long>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned_long>_>,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned_long>_>_>
- [-] `_STL::_Rb_tree<unsigned___int64,_STL::pair<unsigned___int64_const_,unsigned_long>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned_long>_>,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned_long>_>_>::_Rb_tree<unsigned___int64,_STL::pair<unsigned___int64_const_,unsigned_long>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned_long>_>,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned_long>_>_>` · 004aea60 · legions 004a5ce0 — Rust std replaces it

### _STL::_Rb_tree<unsigned___int64,_STL::pair<unsigned___int64_const_,unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned___int64>_>,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned___int64>_>_>
- [-] `_STL::_Rb_tree<unsigned___int64,_STL::pair<unsigned___int64_const_,unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned___int64>_>,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned___int64>_>_>::_Rb_tree<unsigned___int64,_STL::pair<unsigned___int64_const_,unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned___int64>_>,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned___int64>_>_>` · 004aeb10 · legions 004a5d90 — Rust std replaces it

### _STL::hashtable<_STL::pair<unsigned_long_const_,unsigned___int64>,unsigned_long,_STL::hash<unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned___int64>_>,_STL::equal_to<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned___int64>_>_>
- [-] `_STL::hashtable<_STL::pair<unsigned_long_const_,unsigned___int64>,unsigned_long,_STL::hash<unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned___int64>_>,_STL::equal_to<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned___int64>_>_>::_M_copy_from` · 004aebc0 · legions 004a5e40 — Rust std replaces it

### _STL::hashtable<_STL::pair<unsigned_long_const_,unsigned_long>,unsigned_long,_STL::hash<unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned_long>_>,_STL::equal_to<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>
- [-] `_STL::hashtable<_STL::pair<unsigned_long_const_,unsigned_long>,unsigned_long,_STL::hash<unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned_long>_>,_STL::equal_to<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>::_M_copy_from` · 004aed30 · legions 004a5fb0 — Rust std replaces it

### _STL::hashtable<_STL::pair<unsigned___int64_const_,unsigned_long>,unsigned___int64,_STL::hash<unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned_long>_>,_STL::equal_to<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned_long>_>_>
- [-] `_STL::hashtable<_STL::pair<unsigned___int64_const_,unsigned_long>,unsigned___int64,_STL::hash<unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned_long>_>,_STL::equal_to<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned_long>_>_>::_M_copy_from` · 004aee80 · legions 004a6100 — Rust std replaces it

### _STL::hashtable<_STL::pair<unsigned___int64_const_,unsigned___int64>,unsigned___int64,_STL::hash<unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned___int64>_>,_STL::equal_to<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned___int64>_>_>
- [-] `_STL::hashtable<_STL::pair<unsigned___int64_const_,unsigned___int64>,unsigned___int64,_STL::hash<unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned___int64>_>,_STL::equal_to<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned___int64>_>_>::_M_copy_from` · 004aeff0 · legions 004a6270 — Rust std replaces it

### _STL::hashtable<_STL::pair<unsigned_long_const_,unsigned___int64>,unsigned_long,_STL::hash<unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned___int64>_>,_STL::equal_to<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned___int64>_>_>
- [-] `_STL::hashtable<_STL::pair<unsigned_long_const_,unsigned___int64>,unsigned_long,_STL::hash<unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned___int64>_>,_STL::equal_to<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned___int64>_>_>::_M_initialize_buckets` · 004af950 · legions 004a6bd0 — Rust std replaces it

### _STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned_long>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned_long>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>::insert_equal` · 004afb80 · legions 004a6d70 — Rust std replaces it
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned_long>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>::_M_insert` · 004afbc0 · legions 004a6db0 — Rust std replaces it

### _STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned___int64>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned___int64>_>_>
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned___int64>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned___int64>_>_>::insert_equal` · 004afdb0 · legions 004a6fa0 — Rust std replaces it
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned___int64>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned___int64>_>_>::_M_insert` · 004afdf0 · legions 004a6fe0 — Rust std replaces it

### _STL::_Rb_tree<unsigned___int64,_STL::pair<unsigned___int64_const_,unsigned_long>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned_long>_>,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned_long>_>_>
- [-] `_STL::_Rb_tree<unsigned___int64,_STL::pair<unsigned___int64_const_,unsigned_long>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned_long>_>,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned_long>_>_>::insert_equal` · 004b0000 · legions 004a71f0 — Rust std replaces it
- [-] `_STL::_Rb_tree<unsigned___int64,_STL::pair<unsigned___int64_const_,unsigned_long>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned_long>_>,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned_long>_>_>::_M_insert` · 004b0050 · legions 004a7240 — Rust std replaces it

### _STL::_Rb_tree<unsigned___int64,_STL::pair<unsigned___int64_const_,unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned___int64>_>,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned___int64>_>_>
- [-] `_STL::_Rb_tree<unsigned___int64,_STL::pair<unsigned___int64_const_,unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned___int64>_>,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned___int64>_>_>::insert_equal` · 004b0270 · legions 004a7460 — Rust std replaces it
- [-] `_STL::_Rb_tree<unsigned___int64,_STL::pair<unsigned___int64_const_,unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned___int64>_>,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned___int64>_>_>::_M_insert` · 004b02c0 · legions 004a74b0 — Rust std replaces it

### _STL::hashtable<_STL::pair<unsigned_long_const_,unsigned___int64>,unsigned_long,_STL::hash<unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned___int64>_>,_STL::equal_to<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned___int64>_>_>
- [-] `_STL::hashtable<_STL::pair<unsigned_long_const_,unsigned___int64>,unsigned_long,_STL::hash<unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned___int64>_>,_STL::equal_to<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned___int64>_>_>::insert_equal_noresize` · 004b1010 · legions 004a8200 — Rust std replaces it
- [-] `_STL::hashtable<_STL::pair<unsigned_long_const_,unsigned___int64>,unsigned_long,_STL::hash<unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned___int64>_>,_STL::equal_to<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned___int64>_>_>::resize` · 004b10f0 · legions 004a82e0 — Rust std replaces it

### _STL::vector<void_*,_STL::allocator<void_*>_>
- [-] `_STL::vector<void_*,_STL::allocator<void_*>_>::swap` · 004b11f0 · legions 004a83e0 — Rust std replaces it

### _STL::_Vector_base<void_*,_STL::allocator<void_*>_>
- [-] `_STL::_Vector_base<void_*,_STL::allocator<void_*>_>::_Vector_base<void_*,_STL::allocator<void_*>_>` · 004b1220 · legions 004a8410 — Rust std replaces it

### _STL::hashtable<_STL::pair<unsigned_long_const_,unsigned_long>,unsigned_long,_STL::hash<unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned_long>_>,_STL::equal_to<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>
- [-] `_STL::hashtable<_STL::pair<unsigned_long_const_,unsigned_long>,unsigned_long,_STL::hash<unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned_long>_>,_STL::equal_to<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>::insert_equal_noresize` · 004b1350 · legions 004a8600 — Rust std replaces it
- [-] `_STL::hashtable<_STL::pair<unsigned_long_const_,unsigned_long>,unsigned_long,_STL::hash<unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned_long>_>,_STL::equal_to<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>::resize` · 004b1430 · legions 004a86e0 — Rust std replaces it

### _STL::hashtable<_STL::pair<unsigned___int64_const_,unsigned_long>,unsigned___int64,_STL::hash<unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned_long>_>,_STL::equal_to<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned_long>_>_>
- [-] `_STL::hashtable<_STL::pair<unsigned___int64_const_,unsigned_long>,unsigned___int64,_STL::hash<unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned_long>_>,_STL::equal_to<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned_long>_>_>::insert_equal_noresize` · 004b1610 · legions 004a88c0 — Rust std replaces it

### _STL::hashtable<_STL::pair<unsigned___int64_const_,unsigned___int64>,unsigned___int64,_STL::hash<unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned___int64>_>,_STL::equal_to<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned___int64>_>_>
- [-] `_STL::hashtable<_STL::pair<unsigned___int64_const_,unsigned___int64>,unsigned___int64,_STL::hash<unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned___int64>_>,_STL::equal_to<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned___int64>_>_>::insert_equal_noresize` · 004b18f0 · legions 004a8ba0 — Rust std replaces it

### _STL::hashtable<_STL::pair<unsigned___int64_const_,unsigned_long>,unsigned___int64,_STL::hash<unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned_long>_>,_STL::equal_to<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned_long>_>_>
- [-] `_STL::hashtable<_STL::pair<unsigned___int64_const_,unsigned_long>,unsigned___int64,_STL::hash<unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned_long>_>,_STL::equal_to<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned_long>_>_>::resize` · 004b19f0 · legions 004a8ca0 — Rust std replaces it

### _STL::hashtable<_STL::pair<unsigned_long_const_,unsigned___int64>,unsigned_long,_STL::hash<unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned___int64>_>,_STL::equal_to<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned___int64>_>_>
- [-] `_STL::hashtable<_STL::pair<unsigned_long_const_,unsigned___int64>,unsigned_long,_STL::hash<unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned___int64>_>,_STL::equal_to<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned___int64>_>_>::begin` · 004b2430 · legions 004a9850 — Rust std replaces it

## `portal/client/src/weenie/interp/glue/gcwphysicsobject.cpp.c` (0/3)

### IntrusiveHashTable<InstanceID,HashTableData<InstanceID,float>_*>
- [-] `IntrusiveHashTable<InstanceID,HashTableData<InstanceID,float>_*>::add_internal` · 004b8230 · legions 004ae1a0 — Rust std replaces it

### IntrusiveHashTable<InstanceID,HashTableData<InstanceID,DetectionProfile>_*>
- [-] `IntrusiveHashTable<InstanceID,HashTableData<InstanceID,DetectionProfile>_*>::delete_contents` · 004ba430 · legions 004af820 — Rust std replaces it

### HashTable<InstanceID,DetectionProfile>
- [-] `HashTable<InstanceID,DetectionProfile>::add` · 004ba4c0 · legions 004af8b0 — Rust std replaces it

## `portal/preproc/src/empgen/empgenparse.cpp.c` (0/2)

### SmartArray<PEnum>
- [-] `SmartArray<PEnum>::Reset` · 00676d50 · legions 00621340 — Rust std replaces it
- [-] `SmartArray<PEnum>::grow` · 00677cc0 · legions 006222b0 — Rust std replaces it

## `portal/shared/src/anim/charactersetup.cpp.c` (0/2)

### LongHash<LocationType>
- [-] `LongHash<LocationType>::LongHash<LocationType>` · 0060ea10 · legions 005ceeb0 — Rust std replaces it

### LongHash<PlacementFrame>
- [-] `LongHash<PlacementFrame>::destroy_contents` · 006123b0 · legions 005d0470 — Rust std replaces it

## `portal/shared/src/animsys/animinfo.cpp.c` (0/13)

### _STL
- [-] `_STL::__unguarded_linear_insert` · 007394d0 · legions 006b5010 — Rust std replaces it
- [-] `_STL::__push_heap` · 00739540 · legions 006b5080 — Rust std replaces it
- [-] `_STL::__copy_backward` · 007395e0 · legions 006b5120 — Rust std replaces it
- [-] `_STL::__unguarded_insertion_sort_aux` · 00739640 · legions 006b5220 — Rust std replaces it
- [-] `_STL::__adjust_heap` · 007396b0 · legions 006b5290 — Rust std replaces it
- [-] `_STL::__pop_heap` · 0073a840 · legions 006b6a40 — Rust std replaces it
- [-] `_STL::__make_heap` · 0073a8d0 · legions 006b6ad0 — Rust std replaces it
- [-] `_STL::__unguarded_partition` · 0073a960 · legions 006b6bb0 — Rust std replaces it
- [-] `_STL::__linear_insert` · 0073aa10 · legions 006b6c60 — Rust std replaces it
- [-] `_STL::sort_heap` · 0073aac0 · legions 006b6d10 — Rust std replaces it
- [-] `_STL::__insertion_sort` · 0073ada0 · legions 006b70b0 — Rust std replaces it
- [-] `_STL::__partial_sort` · 0073ae10 · legions 006b7120 — Rust std replaces it
- [-] `_STL::__introsort_loop` · 0073aed0 · legions 006b71e0 — Rust std replaces it

## `portal/shared/src/animsys/animmodules/amangle.cpp.c` (0/3)

### List<BoneAngleMod>
- [-] `List<BoneAngleMod>::push_back` · 00692d00 · legions 00632eb0 — Rust std replaces it
- [-] `List<BoneAngleMod>::insert` · 00692d70 · legions 00632f20 — Rust std replaces it
- [-] `List<BoneAngleMod>::flush` · 00692e10 · legions 00632fc0 — Rust std replaces it

## `portal/shared/src/animsys/animmodules/amswitchme.cpp.c` (0/1)

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,DBAnimator_*>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,DBAnimator_*>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,DBAnimator_*>_*>` · 006947d0 · legions 00634860 — Rust std replaces it

## `portal/shared/src/animsys/animmodules/animasset.cpp.c` (0/4)

### SmartArray<AnimAssetOffsetKey>
- [-] `SmartArray<AnimAssetOffsetKey>::grow` · 006989b0 · legions 00637620 — Rust std replaces it

### SmartArray<AnimAssetQuatKey>
- [-] `SmartArray<AnimAssetQuatKey>::grow` · 00698a90 · legions 00637700 — Rust std replaces it

### SmartArray<AnimAssetWeightKey>
- [-] `SmartArray<AnimAssetWeightKey>::grow` · 00698b70 · legions 006377e0 — Rust std replaces it

### SmartArray<AnimAssetScaleKey>
- [-] `SmartArray<AnimAssetScaleKey>::grow` · 006991f0 · legions 00637e60 — Rust std replaces it

## `portal/shared/src/animsys/animmodules/animcube.cpp.c` (0/4)

### PStringBase<char>
- [-] `PStringBase<char>::append_float` · 006970e0 — Rust std replaces it

### HashTable<unsigned_long,WeenieHook>
- [-] `HashTable<unsigned_long,WeenieHook>::add` · 006977e0 · legions 00636a70 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,WeenieHook>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,WeenieHook>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,WeenieHook>_*>` · 00697890 · legions 00636b20 — Rust std replaces it

### SmartArray<AnimCube::SliderAndTickList>
- [-] `SmartArray<AnimCube::SliderAndTickList>::grow` · 00697920 · legions 00636bb0 — Rust std replaces it

## `portal/shared/src/animsys/animmodules/animinterpolator.cpp.c` (0/2)

### List<Ray>
- [-] `List<Ray>::flush` · 00692160 · legions 004a3200 — Rust std replaces it

### List<Trigger>
- [-] `List<Trigger>::push_back` · 00692280 · legions 006323f0 — Rust std replaces it

## `portal/shared/src/animsys/behaviortable.cpp.c` (0/36)

### List<ModeStateKey>
- [-] `List<ModeStateKey>::insert` · 0067d290 · legions 00625670 — Rust std replaces it

### IntrusiveHashTable<ModeTransitionKey,HashTableData<ModeTransitionKey,float>_*>
- [-] `IntrusiveHashTable<ModeTransitionKey,HashTableData<ModeTransitionKey,float>_*>::find_internal` · 0067d480 · legions 006257c0 — Rust std replaces it

### IntrusiveHashTable<ComboKey,HashTableData<ComboKey,ComboInfo>_*>
- [-] `IntrusiveHashTable<ComboKey,HashTableData<ComboKey,ComboInfo>_*>::find_internal` · 0067d4d0 · legions 00625810 — Rust std replaces it

### List<ModeStateKey>
- [-] `List<ModeStateKey>::push_back` · 0067d520 · legions 00625860 — Rust std replaces it

### SmartArray<ParseHelper<EmpGenParse>::ParseHandler>
- [-] `SmartArray<ParseHelper<EmpGenParse>::ParseHandler>::grow` · 0067d590 · legions 00621270 — Rust std replaces it

### IntrusiveHashTable<ModeTransitionKey,HashTableData<ModeTransitionKey,float>_*>
- [-] `IntrusiveHashTable<ModeTransitionKey,HashTableData<ModeTransitionKey,float>_*>::delete_contents` · 0067daa0 · legions 006258d0 — Rust std replaces it

### IntrusiveHashTable<ComboKey,HashTableData<ComboKey,ComboInfo>_*>
- [-] `IntrusiveHashTable<ComboKey,HashTableData<ComboKey,ComboInfo>_*>::delete_contents` · 0067db30 · legions 00625960 — Rust std replaces it

### IntrusiveHashTable<ModeTransitionKey,HashTableData<ModeTransitionKey,TransitionInfo>_*>
- [-] `IntrusiveHashTable<ModeTransitionKey,HashTableData<ModeTransitionKey,TransitionInfo>_*>::delete_contents` · 0067dbc0 · legions 006259f0 — Rust std replaces it

### HashTable<ModeStateKey,PList<ModeStateKey>_>
- [-] `HashTable<ModeStateKey,PList<ModeStateKey>_>::find` · 0067dc50 — Rust std replaces it

### IntrusiveHashTable<ComboKey,HashTableData<ComboKey,ComboInfo>_*>
- [-] `IntrusiveHashTable<ComboKey,HashTableData<ComboKey,ComboInfo>_*>::add` · 0067dce0 — Rust std replaces it

### IntrusiveHashTable<ModeTransitionKey,HashTableData<ModeTransitionKey,float>_*>
- [-] `IntrusiveHashTable<ModeTransitionKey,HashTableData<ModeTransitionKey,float>_*>::add` · 0067dd40 — Rust std replaces it

### HashTable<ModeTransitionKey,float>
- [-] `HashTable<ModeTransitionKey,float>::add` · 0067e0b0 — Rust std replaces it

### HashTable<ComboKey,ComboInfo>
- [-] `HashTable<ComboKey,ComboInfo>::add` · 0067e260 — Rust std replaces it

### HashTable<ModeTransitionKey,TransitionInfo>
- [-] `HashTable<ModeTransitionKey,TransitionInfo>::add` · 0067e2f0 — Rust std replaces it

### HashTable<unsigned_long,BehaviorInfo>
- [-] `HashTable<unsigned_long,BehaviorInfo>::set` · 0067e490 — Rust std replaces it

### IntrusiveHashTable<ModeTransitionKey,HashTableData<ModeTransitionKey,float>_*>
- [-] `IntrusiveHashTable<ModeTransitionKey,HashTableData<ModeTransitionKey,float>_*>::IntrusiveHashTable<ModeTransitionKey,HashTableData<ModeTransitionKey,float>_*>` · 0067e670 · legions 006261f0 — Rust std replaces it

### IntrusiveHashTable<ComboKey,HashTableData<ComboKey,ComboInfo>_*>
- [-] `IntrusiveHashTable<ComboKey,HashTableData<ComboKey,ComboInfo>_*>::IntrusiveHashTable<ComboKey,HashTableData<ComboKey,ComboInfo>_*>` · 0067e700 · legions 00626280 — Rust std replaces it

### IntrusiveHashTable<ModeTransitionKey,HashTableData<ModeTransitionKey,TransitionInfo>_*>
- [-] `IntrusiveHashTable<ModeTransitionKey,HashTableData<ModeTransitionKey,TransitionInfo>_*>::IntrusiveHashTable<ModeTransitionKey,HashTableData<ModeTransitionKey,TransitionInfo>_*>` · 0067e790 · legions 00626310 — Rust std replaces it

### IntrusiveHashTable<ModeStateKey,HashTableData<ModeStateKey,PList<ModeStateKey>_>_*>
- [-] `IntrusiveHashTable<ModeStateKey,HashTableData<ModeStateKey,PList<ModeStateKey>_>_*>::IntrusiveHashTable<ModeStateKey,HashTableData<ModeStateKey,PList<ModeStateKey>_>_*>` · 0067e820 · legions 006263a0 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,BehaviorInfo>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,BehaviorInfo>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,BehaviorInfo>_*>` · 0067e8b0 · legions 00626430 — Rust std replaces it

### List<ModeStateKey>
- [-] `List<ModeStateKey>::flush` · 0067e940 · legions 006264c0 — Rust std replaces it

### IntrusiveHashTable<ModeTransitionKey,HashTableData<ModeTransitionKey,float>_*>
- [-] `IntrusiveHashTable<ModeTransitionKey,HashTableData<ModeTransitionKey,float>_*>::resize_internal` · 006820b0 · legions 00627080 — Rust std replaces it

### IntrusiveHashTable<ComboKey,HashTableData<ComboKey,ComboInfo>_*>
- [-] `IntrusiveHashTable<ComboKey,HashTableData<ComboKey,ComboInfo>_*>::resize_internal` · 006823c0 · legions 00626cb0 — Rust std replaces it
- [-] `IntrusiveHashTable<ComboKey,HashTableData<ComboKey,ComboInfo>_*>::add_for_unpacking` · 006824e0 · legions 00626dd0 — Rust std replaces it

### HashTableData<ComboKey,ComboInfo>
- [-] `HashTableData<ComboKey,ComboInfo>::HashTableData<ComboKey,ComboInfo>` · 00682550 · legions 00626e40 — Rust std replaces it

### HashTable<unsigned_long,gmRaceSexInfo>
- [-] `HashTable<unsigned_long,gmRaceSexInfo>::end` · 00682790 · legions 0040e3d0 — Rust std replaces it

### IntrusiveHashTable<ModeTransitionKey,HashTableData<ModeTransitionKey,float>_*>
- [-] `IntrusiveHashTable<ModeTransitionKey,HashTableData<ModeTransitionKey,float>_*>::add_for_unpacking` · 006827b0 · legions 00626a50 — Rust std replaces it

### HashTableData<ModeTransitionKey,TransitionInfo>
- [-] `HashTableData<ModeTransitionKey,TransitionInfo>::HashTableData<ModeTransitionKey,TransitionInfo>` · 00682820 · legions 006271a0 — Rust std replaces it

### List<ModeStateKey>
- [-] `List<ModeStateKey>::List<ModeStateKey>` · 00682a90 · legions 00627410 — Rust std replaces it

### IntrusiveHashTable<ModeStateKey,HashTableData<ModeStateKey,PList<ModeStateKey>_>_*>
- [-] `IntrusiveHashTable<ModeStateKey,HashTableData<ModeStateKey,PList<ModeStateKey>_>_*>::delete_contents` · 00682ae0 · legions 00627460 — Rust std replaces it

### List<PList<ModeStateKey>_>
- [-] `List<PList<ModeStateKey>_>::pop_back` · 00682b90 · legions 00627510 — Rust std replaces it
- [-] `List<PList<ModeStateKey>_>::insert` · 00682c00 · legions 00627580 — Rust std replaces it

### HashTableData<ModeStateKey,PList<ModeStateKey>_>
- [-] `HashTableData<ModeStateKey,PList<ModeStateKey>_>::HashTableData<ModeStateKey,PList<ModeStateKey>_>` · 00682cb0 — Rust std replaces it

### List<PList<ModeStateKey>_>
- [-] `List<PList<ModeStateKey>_>::push_back` · 00682d00 · legions 00627630 — Rust std replaces it
- [-] `List<PList<ModeStateKey>_>::flush` · 00682d90 · legions 006276c0 — Rust std replaces it

### HashTable<ModeStateKey,PList<ModeStateKey>_>
- [-] `HashTable<ModeStateKey,PList<ModeStateKey>_>::add` · 00683040 — Rust std replaces it

## `portal/shared/src/animsys/behaviortrack.cpp.c` (0/1)

### _STL::_Rb_tree<unsigned_long,unsigned_long,_STL::_Identity<unsigned_long>,_STL::less<unsigned_long>,_STL::allocator<unsigned_long>_>
- [-] `_STL::_Rb_tree<unsigned_long,unsigned_long,_STL::_Identity<unsigned_long>,_STL::less<unsigned_long>,_STL::allocator<unsigned_long>_>::insert_unique` · 0068f7a0 · legions 00630320 — Rust std replaces it

## `portal/shared/src/animsys/behaviortracker.cpp.c` (0/7)

### SmartArray<WeenieHook>
- [-] `SmartArray<WeenieHook>::grow` · 0068bf30 · legions 0062cf60 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,BehaviorTrack>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,BehaviorTrack>_*>::delete_contents` · 0068c030 · legions 0062d060 — Rust std replaces it

### HashTable<unsigned_long,BehaviorTrack>
- [-] `HashTable<unsigned_long,BehaviorTrack>::erase` · 0068c370 · legions 0062d3a0 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,BehaviorTrack>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,BehaviorTrack>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,BehaviorTrack>_*>` · 0068c5b0 · legions 0062d5e0 — Rust std replaces it

### _STL::_Rb_tree<unsigned_long,unsigned_long,_STL::_Identity<unsigned_long>,_STL::less<unsigned_long>,_STL::allocator<unsigned_long>_>
- [-] `_STL::_Rb_tree<unsigned_long,unsigned_long,_STL::_Identity<unsigned_long>,_STL::less<unsigned_long>,_STL::allocator<unsigned_long>_>::_M_copy` · 0068cef0 · legions 0062df10 — Rust std replaces it
- [-] `_STL::_Rb_tree<unsigned_long,unsigned_long,_STL::_Identity<unsigned_long>,_STL::less<unsigned_long>,_STL::allocator<unsigned_long>_>::_Rb_tree<unsigned_long,unsigned_long,_STL::_Identity<unsigned_long>,_STL::less<unsigned_long>,_STL::allocator<unsigned_long>_>` · 0068d1c0 · legions 0062e1e0 — Rust std replaces it

### HashTable<unsigned_long,BehaviorTrack>
- [-] `HashTable<unsigned_long,BehaviorTrack>::set` · 0068dae0 · legions 0062eb00 — Rust std replaces it

## `portal/shared/src/animsys/chartemplate.cpp.c` (0/7)

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,UILocationData>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,UILocationData>_*>::delete_contents` · 0067a960 · legions 00625a80 — Rust std replaces it

### HashTable<unsigned_long,Behavior>
- [-] `HashTable<unsigned_long,Behavior>::add` · 0067b070 — Rust std replaces it

### HashTable<unsigned_long,Slider>
- [-] `HashTable<unsigned_long,Slider>::add` · 0067b1e0 — Rust std replaces it

### HashTable<unsigned_long,Mode>
- [-] `HashTable<unsigned_long,Mode>::add` · 0067b350 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,Behavior>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,Behavior>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,Behavior>_*>` · 0067b4e0 · legions 006249c0 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,Slider>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,Slider>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,Slider>_*>` · 0067b570 · legions 00624a50 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,Mode>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,Mode>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,Mode>_*>` · 0067b600 · legions 00624ae0 — Rust std replaces it

## `portal/shared/src/animsys/fxscript.cpp.c` (0/29)

### SmartArray<Trigger>
- [-] `SmartArray<Trigger>::WhatNum` · 00685980 · legions 00628920 — Rust std replaces it

### _STL
- [-] `_STL::__unguarded_linear_insert` · 00685aa0 · legions 006289e0 — Rust std replaces it
- [-] `_STL::__unguarded_linear_insert` · 00685b30 · legions 00628a70 — Rust std replaces it
- [-] `_STL::__push_heap` · 00685bd0 · legions 00628b10 — Rust std replaces it
- [-] `_STL::__push_heap` · 00685c90 · legions 00628bd0 — Rust std replaces it
- [-] `_STL::__copy_backward` · 00685d50 · legions 00628c90 — Rust std replaces it
- [-] `_STL::__copy_backward` · 00685db0 · legions 00628cf0 — Rust std replaces it
- [-] `_STL::__unguarded_insertion_sort_aux` · 00685f20 · legions 00628e30 — Rust std replaces it
- [-] `_STL::__unguarded_insertion_sort_aux` · 00685fa0 · legions 00628eb0 — Rust std replaces it
- [-] `_STL::swap` · 00686020 · legions 00628f30 — Rust std replaces it
- [-] `_STL::swap` · 006860a0 · legions 00628fb0 — Rust std replaces it
- [-] `_STL::__adjust_heap` · 00686120 · legions 00629030 — Rust std replaces it
- [-] `_STL::__adjust_heap` · 00686240 · legions 00629150 — Rust std replaces it
- [-] `_STL::__pop_heap` · 00687540 · legions 00629370 — Rust std replaces it
- [-] `_STL::__pop_heap` · 006875e0 · legions 00629410 — Rust std replaces it
- [-] `_STL::__make_heap` · 00687680 · legions 006294b0 — Rust std replaces it
- [-] `_STL::__make_heap` · 00687710 · legions 00629540 — Rust std replaces it
- [-] `_STL::__unguarded_partition` · 00687860 · legions 00629690 — Rust std replaces it
- [-] `_STL::__unguarded_partition` · 006878d0 · legions 00629700 — Rust std replaces it
- [-] `_STL::__linear_insert` · 00687940 · legions 00629770 — Rust std replaces it
- [-] `_STL::__linear_insert` · 00687a00 · legions 00629830 — Rust std replaces it
- [-] `_STL::sort_heap` · 00687ac0 · legions 006298f0 — Rust std replaces it
- [-] `_STL::sort_heap` · 00687b50 · legions 00629980 — Rust std replaces it
- [-] `_STL::__insertion_sort` · 00687cd0 · legions 00629b00 — Rust std replaces it
- [-] `_STL::__insertion_sort` · 00687d50 · legions 00629b80 — Rust std replaces it
- [-] `_STL::__partial_sort` · 00687dd0 · legions 00629c00 — Rust std replaces it
- [-] `_STL::__partial_sort` · 00687e90 · legions 00629cc0 — Rust std replaces it
- [-] `_STL::__introsort_loop` · 00687f50 · legions 00629d80 — Rust std replaces it
- [-] `_STL::__introsort_loop` · 00688070 · legions 00629ea0 — Rust std replaces it

## `portal/shared/src/animsys/fxscripttracker.cpp.c` (0/3)

### List<FXTrack>
- [-] `List<FXTrack>::remove` · 0068e540 · legions 0062f4d0 — Rust std replaces it
- [-] `List<FXTrack>::flush` · 0068e5b0 · legions 0062f540 — Rust std replaces it
- [-] `List<FXTrack>::push_back` · 0068e860 · legions 0062f840 — Rust std replaces it

## `portal/shared/src/animsys/slidertracker.cpp.c` (0/4)

### HashTable<unsigned_long,SliderInfo>
- [-] `HashTable<unsigned_long,SliderInfo>::add` · 00689e30 · legions 0062ae70 — Rust std replaces it

### HashTable<unsigned_long,SliderInterpolator>
- [-] `HashTable<unsigned_long,SliderInterpolator>::add` · 00689ec0 · legions 0062af00 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,SliderInfo>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,SliderInfo>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,SliderInfo>_*>` · 0068a010 · legions 0062b050 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,SliderInterpolator>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,SliderInterpolator>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,SliderInterpolator>_*>` · 0068a0a0 · legions 0062b0e0 — Rust std replaces it

## `portal/shared/src/animsys/triggerupdate.cpp.c` (0/1)

### HashTable<unsigned_long,Trigger>
- [-] `HashTable<unsigned_long,Trigger>::add` · 00689440 · legions 0062a460 — Rust std replaces it

## `portal/shared/src/cell/envcell.cpp.c` (0/28)

### SmartArray<CCellPortal>
- [-] `SmartArray<CCellPortal>::Reset` · 004801f0 · legions 0047a190 — Rust std replaces it

### SmartArray<CellFrustum>
- [-] `SmartArray<CellFrustum>::Reset` · 00480250 · legions 00477f30 — Rust std replaces it

### _STL::_Rb_global<bool>
- [-] `_STL::_Rb_global<bool>::_Rotate_left` · 00480280 · legions 0047a1f0 — Rust std replaces it
- [-] `_STL::_Rb_global<bool>::_Rotate_right` · 004802d0 · legions 0047a240 — Rust std replaces it

### SmartArray<Vector>
- [-] `SmartArray<Vector>::grow` · 004817d0 · legions 0047b690 — Rust std replaces it

### SmartArray<CCellPortal>
- [-] `SmartArray<CCellPortal>::grow` · 004818a0 · legions 0047b810 — Rust std replaces it

### SmartArray<CellFrustum>
- [-] `SmartArray<CellFrustum>::grow` · 00481990 · legions 0047b900 — Rust std replaces it

### SmartArray<Frustum>
- [-] `SmartArray<Frustum>::grow` · 00481a80 · legions 0047b9f0 — Rust std replaces it

### _STL::_Rb_global<bool>
- [-] `_STL::_Rb_global<bool>::_Rebalance` · 00481b40 · legions 0047bab0 — Rust std replaces it

### List<IconModAtom>
- [-] `List<IconModAtom>::pop_front` · 00482b70 · legions 0047ca10 — Rust std replaces it

### List<MaterialModifierAtom>
- [-] `List<MaterialModifierAtom>::pop_front` · 00483310 · legions 0047d1b0 — Rust std replaces it

### List<PGDReplacementAtom>
- [-] `List<PGDReplacementAtom>::pop_front` · 00483390 · legions 0047d230 — Rust std replaces it

### List<IconModAtom>
- [-] `List<IconModAtom>::flush` · 00484620 · legions 0047de90 — Rust std replaces it

### _STL::_Rb_tree<unsigned_long,unsigned_long,_STL::_Identity<unsigned_long>,_STL::less<unsigned_long>,_STL::allocator<unsigned_long>_>
- [-] `_STL::_Rb_tree<unsigned_long,unsigned_long,_STL::_Identity<unsigned_long>,_STL::less<unsigned_long>,_STL::allocator<unsigned_long>_>::_M_erase` · 00484840 · legions 0047e0b0 — Rust std replaces it

### List<IconModAtom>
- [-] `List<IconModAtom>::push_back` · 004849d0 · legions 0047e290 — Rust std replaces it

### _STL::_Rb_tree<FXAtom,FXAtom,_STL::_Identity<FXAtom>,_STL::less<FXAtom>,_STL::allocator<FXAtom>_>
- [-] `_STL::_Rb_tree<FXAtom,FXAtom,_STL::_Identity<FXAtom>,_STL::less<FXAtom>,_STL::allocator<FXAtom>_>::_M_erase` · 00484a60 · legions 0047e340 — Rust std replaces it

### _STL::_Rb_tree<unsigned_long,unsigned_long,_STL::_Identity<unsigned_long>,_STL::less<unsigned_long>,_STL::allocator<unsigned_long>_>
- [-] `_STL::_Rb_tree<unsigned_long,unsigned_long,_STL::_Identity<unsigned_long>,_STL::less<unsigned_long>,_STL::allocator<unsigned_long>_>::_M_insert` · 00484b30 · legions 0047e410 — Rust std replaces it

### List<MaterialModifierAtom>
- [-] `List<MaterialModifierAtom>::flush` · 00484bf0 · legions 0047e4d0 — Rust std replaces it

### List<OpacityAtom>
- [-] `List<OpacityAtom>::flush` · 00484c50 · legions 0047e530 — Rust std replaces it

### List<MeshReplacementAtom>
- [-] `List<MeshReplacementAtom>::flush` · 00484d00 · legions 0047e5e0 — Rust std replaces it

### List<PGDReplacementAtom>
- [-] `List<PGDReplacementAtom>::flush` · 00484db0 · legions 0047e690 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,CEnvCell_*>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,CEnvCell_*>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,CEnvCell_*>_*>` · 00486020 · legions 0047f8c0 — Rust std replaces it

### _STL::_Rb_tree<unsigned_long,unsigned_long,_STL::_Identity<unsigned_long>,_STL::less<unsigned_long>,_STL::allocator<unsigned_long>_>
- [-] `_STL::_Rb_tree<unsigned_long,unsigned_long,_STL::_Identity<unsigned_long>,_STL::less<unsigned_long>,_STL::allocator<unsigned_long>_>::insert_unique` · 004860b0 · legions 0047f950 — Rust std replaces it

### List<MaterialModifierAtom>
- [-] `List<MaterialModifierAtom>::push_back` · 004862f0 · legions 0047fb90 — Rust std replaces it

### List<OpacityAtom>
- [-] `List<OpacityAtom>::push_back` · 004864e0 · legions 0047fd30 — Rust std replaces it

### List<MeshReplacementAtom>
- [-] `List<MeshReplacementAtom>::push_back` · 00486640 · legions 0047fed0 — Rust std replaces it

### List<PGDReplacementAtom>
- [-] `List<PGDReplacementAtom>::push_back` · 00486880 · legions 00480110 — Rust std replaces it

### _STL::set<unsigned_long,_STL::less<unsigned_long>,_STL::allocator<unsigned_long>_>
- [-] `_STL::set<unsigned_long,_STL::less<unsigned_long>,_STL::allocator<unsigned_long>_>::insert` · 00486eb0 · legions 00480710 — Rust std replaces it

## `portal/shared/src/cell/landblock.cpp.c` (0/3)

### SmartArray<AmbientSoundPoint>
- [-] `SmartArray<AmbientSoundPoint>::grow` · 00489be0 · legions 00483420 — Rust std replaces it

### SmartArray<CELLINFO>
- [-] `SmartArray<CELLINFO>::grow` · 00489cc0 · legions 004835d0 — Rust std replaces it

### SmartArray<InstanceID>
- [-] `SmartArray<InstanceID>::grow` · 0048aa60 · legions 00484310 — Rust std replaces it

## `portal/shared/src/cell/landblockdata.cpp.c` (0/2)

### SmartArray<unsigned_char>
- [-] `SmartArray<unsigned_char>::grow` · 0048f410 · legions 00488230 — Rust std replaces it

### SmartArray<SurfaceInfo>
- [-] `SmartArray<SurfaceInfo>::grow` · 0048f4a0 · legions 004882c0 — Rust std replaces it

## `portal/shared/src/cell/landblockinfo.cpp.c` (0/1)

### SmartArray<EntityLinkDesc>
- [-] `SmartArray<EntityLinkDesc>::grow` · 00491940 · legions 0048a640 — Rust std replaces it

## `portal/shared/src/cell/landblockquadtree.cpp.c` (0/3)

### _STL::pair<ICPhysicsObj_*,BBox>
- [-] `_STL::pair<ICPhysicsObj_*,BBox>::pair<ICPhysicsObj_*,BBox>` · 00491f30 · legions 0048a8e0 — Rust std replaces it

### SmartArray<_STL::pair<ICPhysicsObj_*,BBox>_>
- [-] `SmartArray<_STL::pair<ICPhysicsObj_*,BBox>_>::Reset` · 00492250 · legions 0048ac00 — Rust std replaces it
- [-] `SmartArray<_STL::pair<ICPhysicsObj_*,BBox>_>::grow` · 00492470 · legions 0048ae20 — Rust std replaces it

## `portal/shared/src/cell/landcell.cpp.c` (0/1)

### SmartArray<Plane>
- [-] `SmartArray<Plane>::grow` · 0047e440 · legions 00478480 — Rust std replaces it

## `portal/shared/src/cell/lightinfo.cpp.c` (0/6)

### SmartArray<CLightInfo::LightCellInfo>
- [-] `SmartArray<CLightInfo::LightCellInfo>::Reset` · 0048d600 · legions 00486f30 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PSmartArray<unsigned_long>_>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PSmartArray<unsigned_long>_>_*>::delete_contents` · 0048d680 · legions 00486fb0 — Rust std replaces it

### SmartArray<CLightInfo::LightCellInfo>
- [-] `SmartArray<CLightInfo::LightCellInfo>::grow` · 0048d980 · legions 00487180 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PSmartArray<unsigned_long>_>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PSmartArray<unsigned_long>_>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PSmartArray<unsigned_long>_>_*>` · 0048e080 · legions 00487880 — Rust std replaces it

### HashTable<unsigned_long,PSmartArray<unsigned_long>_>
- [-] `HashTable<unsigned_long,PSmartArray<unsigned_long>_>::add` · 0048e1d0 — Rust std replaces it

### HashTableData<unsigned_long,PSmartArray<unsigned_long>_>
- [-] `HashTableData<unsigned_long,PSmartArray<unsigned_long>_>::HashTableData<unsigned_long,PSmartArray<unsigned_long>_>` · 0048e7b0 · legions 00487bd0 — Rust std replaces it

## `portal/shared/src/cell/objcell.cpp.c` (0/10)

### SmartArray<ObjectClearance>
- [-] `SmartArray<ObjectClearance>::Reset` · 0047bf60 · legions 00476090 — Rust std replaces it

### SmartArray<BBox>
- [-] `SmartArray<BBox>::grow` · 0047c6b0 · legions 00476770 — Rust std replaces it

### SmartArray<Cylsphere>
- [-] `SmartArray<Cylsphere>::grow` · 0047c7c0 · legions 00476880 — Rust std replaces it

### IntrusiveHashTable<InstanceID,HashTableData<InstanceID,unsigned_long>_*>
- [-] `IntrusiveHashTable<InstanceID,HashTableData<InstanceID,unsigned_long>_*>::remove` · 0047cea0 · legions 004c8710 — Rust std replaces it

### SmartArray<ObjectClearance>
- [-] `SmartArray<ObjectClearance>::grow` · 0047cf00 · legions 00476f90 — Rust std replaces it

### HashTable<InstanceID,DetectionVoyeur>
- [-] `HashTable<InstanceID,DetectionVoyeur>::find` · 0047d0e0 · legions 00477170 — Rust std replaces it

### IntrusiveHashTable<InstanceID,HashTableData<InstanceID,ICPhysicsObj_*>_*>
- [-] `IntrusiveHashTable<InstanceID,HashTableData<InstanceID,ICPhysicsObj_*>_*>::add` · 0047d110 · legions 004c8af0 — Rust std replaces it

### HashTable<InstanceID,DetectionVoyeur>
- [-] `HashTable<InstanceID,DetectionVoyeur>::add` · 0047d470 · legions 004774a0 — Rust std replaces it

### IntrusiveHashTable<InstanceID,HashTableData<InstanceID,DetectionVoyeur>_*>
- [-] `IntrusiveHashTable<InstanceID,HashTableData<InstanceID,DetectionVoyeur>_*>::IntrusiveHashTable<InstanceID,HashTableData<InstanceID,DetectionVoyeur>_*>` · 0047d650 · legions 00477680 — Rust std replaces it
- [-] `IntrusiveHashTable<InstanceID,HashTableData<InstanceID,DetectionVoyeur>_*>::delete_contents` · 0047d720 · legions 00477750 — Rust std replaces it

## `portal/shared/src/conversation/conversationtree.cpp.c` (0/13)

### List<ConversationAction>
- [-] `List<ConversationAction>::push_back` · 006b99c0 · legions 0064fbe0 — Rust std replaces it

### List<ConversationDecision>
- [-] `List<ConversationDecision>::push_back` · 006b9a30 · legions 0064fc50 — Rust std replaces it

### _STL::_Rb_tree<PStringBase<unsigned_short>,_STL::pair<PStringBase<unsigned_short>_const_,ConversationNode>,_STL::_Select1st<_STL::pair<PStringBase<unsigned_short>_const_,ConversationNode>_>,_STL::less<PStringBase<unsigned_short>_>,_STL::allocator<_STL::pair<PStringBase<unsigned_short>_const_,ConversationNode>_>_>
- [-] `_STL::_Rb_tree<PStringBase<unsigned_short>,_STL::pair<PStringBase<unsigned_short>_const_,ConversationNode>,_STL::_Select1st<_STL::pair<PStringBase<unsigned_short>_const_,ConversationNode>_>,_STL::less<PStringBase<unsigned_short>_>,_STL::allocator<_STL::pair<PStringBase<unsigned_short>_const_,ConversationNode>_>_>::_M_lower_bound` · 006b9aa0 · legions 0060f230 — Rust std replaces it

### List<ConversationAction>
- [-] `List<ConversationAction>::flush` · 006b9c50 · legions 0064fcc0 — Rust std replaces it

### List<ConversationDecision>
- [-] `List<ConversationDecision>::flush` · 006b9cc0 · legions 0064fd30 — Rust std replaces it

### _STL::_Rb_tree<PStringBase<unsigned_short>,_STL::pair<PStringBase<unsigned_short>_const_,ConversationNode>,_STL::_Select1st<_STL::pair<PStringBase<unsigned_short>_const_,ConversationNode>_>,_STL::less<PStringBase<unsigned_short>_>,_STL::allocator<_STL::pair<PStringBase<unsigned_short>_const_,ConversationNode>_>_>
- [-] `_STL::_Rb_tree<PStringBase<unsigned_short>,_STL::pair<PStringBase<unsigned_short>_const_,ConversationNode>,_STL::_Select1st<_STL::pair<PStringBase<unsigned_short>_const_,ConversationNode>_>,_STL::less<PStringBase<unsigned_short>_>,_STL::allocator<_STL::pair<PStringBase<unsigned_short>_const_,ConversationNode>_>_>::_M_erase` · 006b9f60 · legions 00650060 — Rust std replaces it

### List<ConversationAction>
- [-] `List<ConversationAction>::List<ConversationAction>` · 006ba0c0 · legions 006501c0 — Rust std replaces it

### List<ConversationDecision>
- [-] `List<ConversationDecision>::List<ConversationDecision>` · 006ba110 · legions 00650210 — Rust std replaces it

### _STL::_Rb_tree<PStringBase<unsigned_short>,_STL::pair<PStringBase<unsigned_short>_const_,ConversationNode>,_STL::_Select1st<_STL::pair<PStringBase<unsigned_short>_const_,ConversationNode>_>,_STL::less<PStringBase<unsigned_short>_>,_STL::allocator<_STL::pair<PStringBase<unsigned_short>_const_,ConversationNode>_>_>
- [-] `_STL::_Rb_tree<PStringBase<unsigned_short>,_STL::pair<PStringBase<unsigned_short>_const_,ConversationNode>,_STL::_Select1st<_STL::pair<PStringBase<unsigned_short>_const_,ConversationNode>_>,_STL::less<PStringBase<unsigned_short>_>,_STL::allocator<_STL::pair<PStringBase<unsigned_short>_const_,ConversationNode>_>_>::_M_clone_node` · 006ba3b0 · legions 006504b0 — Rust std replaces it
- [-] `_STL::_Rb_tree<PStringBase<unsigned_short>,_STL::pair<PStringBase<unsigned_short>_const_,ConversationNode>,_STL::_Select1st<_STL::pair<PStringBase<unsigned_short>_const_,ConversationNode>_>,_STL::less<PStringBase<unsigned_short>_>,_STL::allocator<_STL::pair<PStringBase<unsigned_short>_const_,ConversationNode>_>_>::_M_insert` · 006ba430 · legions 00650530 — Rust std replaces it
- [-] `_STL::_Rb_tree<PStringBase<unsigned_short>,_STL::pair<PStringBase<unsigned_short>_const_,ConversationNode>,_STL::_Select1st<_STL::pair<PStringBase<unsigned_short>_const_,ConversationNode>_>,_STL::less<PStringBase<unsigned_short>_>,_STL::allocator<_STL::pair<PStringBase<unsigned_short>_const_,ConversationNode>_>_>::_M_copy` · 006ba530 · legions 00650630 — Rust std replaces it
- [-] `_STL::_Rb_tree<PStringBase<unsigned_short>,_STL::pair<PStringBase<unsigned_short>_const_,ConversationNode>,_STL::_Select1st<_STL::pair<PStringBase<unsigned_short>_const_,ConversationNode>_>,_STL::less<PStringBase<unsigned_short>_>,_STL::allocator<_STL::pair<PStringBase<unsigned_short>_const_,ConversationNode>_>_>::insert_unique` · 006ba620 · legions 00650720 — Rust std replaces it
- [-] `_STL::_Rb_tree<PStringBase<unsigned_short>,_STL::pair<PStringBase<unsigned_short>_const_,ConversationNode>,_STL::_Select1st<_STL::pair<PStringBase<unsigned_short>_const_,ConversationNode>_>,_STL::less<PStringBase<unsigned_short>_>,_STL::allocator<_STL::pair<PStringBase<unsigned_short>_const_,ConversationNode>_>_>::insert_unique` · 006ba7c0 · legions 006508c0 — Rust std replaces it

## `portal/shared/src/database/asynccache.cpp.c` (0/7)

### IntrusiveHashTable<QualifiedDataID,HashTableData<QualifiedDataID,CMissingIteration_*>_*>
- [-] `IntrusiveHashTable<QualifiedDataID,HashTableData<QualifiedDataID,CMissingIteration_*>_*>::find_internal` · 006d1220 · legions 004b4020 — Rust std replaces it
- [-] `IntrusiveHashTable<QualifiedDataID,HashTableData<QualifiedDataID,CMissingIteration_*>_*>::resize_internal` · 006d1270 · legions 0065af40 — Rust std replaces it

### IntrusiveHashTable<QualifiedDataID,HashTableData<QualifiedDataID,AsyncCache::CAsyncGetRequest_*>_*>
- [-] `IntrusiveHashTable<QualifiedDataID,HashTableData<QualifiedDataID,AsyncCache::CAsyncGetRequest_*>_*>::remove` · 006d13b0 · legions 0065b080 — Rust std replaces it
- [-] `IntrusiveHashTable<QualifiedDataID,HashTableData<QualifiedDataID,AsyncCache::CAsyncGetRequest_*>_*>::add` · 006d1410 · legions 0065b0e0 — Rust std replaces it

### HashTable<QualifiedDataID,AsyncCache::CAsyncGetRequest_*>
- [-] `HashTable<QualifiedDataID,AsyncCache::CAsyncGetRequest_*>::find` · 006d1540 · legions 0065b210 — Rust std replaces it
- [-] `HashTable<QualifiedDataID,AsyncCache::CAsyncGetRequest_*>::add` · 006d1580 · legions 0065b250 — Rust std replaces it
- [-] `HashTable<QualifiedDataID,AsyncCache::CAsyncGetRequest_*>::grow` · 006d1cd0 · legions 0065b9a0 — Rust std replaces it

## `portal/shared/src/database/datfiledata.cpp.c` (0/1)

### SmartArray<GrooveInfo>
- [-] `SmartArray<GrooveInfo>::add` · 006d9ae0 · legions 00662e60 — Rust std replaces it

## `portal/shared/src/database/dbcache.cpp.c` (0/3)

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,DBOCache_*>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,DBOCache_*>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,DBOCache_*>_*>` · 006d0630 · legions 0065a2e0 — Rust std replaces it

### IntrusiveHashTable<QualifiedDataID,HashTableData<QualifiedDataID,AsyncCache::CAsyncGetRequest_*>_*>
- [-] `IntrusiveHashTable<QualifiedDataID,HashTableData<QualifiedDataID,AsyncCache::CAsyncGetRequest_*>_*>::IntrusiveHashTable<QualifiedDataID,HashTableData<QualifiedDataID,AsyncCache::CAsyncGetRequest_*>_*>` · 006d06c0 · legions 0065a370 — Rust std replaces it

### IntrusiveHashTable<AsyncContext,HashTableData<AsyncContext,AsyncCache::CCallbackHandler_*>_*>
- [-] `IntrusiveHashTable<AsyncContext,HashTableData<AsyncContext,AsyncCache::CCallbackHandler_*>_*>::IntrusiveHashTable<AsyncContext,HashTableData<AsyncContext,AsyncCache::CCallbackHandler_*>_*>` · 006d0750 · legions 0065a400 — Rust std replaces it

## `portal/shared/src/database/dbcategories.cpp.c` (0/5)

### IntrusiveHashTable<CaseInsensitiveStringBase<PStringBase<char>_>,HashTableData<CaseInsensitiveStringBase<PStringBase<char>_>,TFileEntry_*>_*>
- [-] `IntrusiveHashTable<CaseInsensitiveStringBase<PStringBase<char>_>,HashTableData<CaseInsensitiveStringBase<PStringBase<char>_>,TFileEntry_*>_*>::find_internal` · 006d8de0 · legions 006186b0 — Rust std replaces it

### IntrusiveHashTable<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,FunctionID>_*>
- [-] `IntrusiveHashTable<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,FunctionID>_*>::delete_contents` · 006d8e50 · legions 00673570 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,BaseList<unsigned_long,11>_>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,BaseList<unsigned_long,11>_>_*>::nice_num_buckets` · 006d8f00 · legions 004ea4b0 — Rust std replaces it

### IntrusiveHashTable<CaseInsensitiveStringBase<PStringBase<char>_>,HashTableData<CaseInsensitiveStringBase<PStringBase<char>_>,unsigned_long>_*>
- [-] `IntrusiveHashTable<CaseInsensitiveStringBase<PStringBase<char>_>,HashTableData<CaseInsensitiveStringBase<PStringBase<char>_>,unsigned_long>_*>::IntrusiveHashTable<CaseInsensitiveStringBase<PStringBase<char>_>,HashTableData<CaseInsensitiveStringBase<PStringBase<char>_>,unsigned_long>_*>` · 006d8f40 · legions 00662540 — Rust std replaces it
- [-] `IntrusiveHashTable<CaseInsensitiveStringBase<PStringBase<char>_>,HashTableData<CaseInsensitiveStringBase<PStringBase<char>_>,unsigned_long>_*>::add_for_unpacking` · 006d9670 · legions 00662a40 — Rust std replaces it

## `portal/shared/src/database/dbocache.cpp.c` (0/1)

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,DBObj_*>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,DBObj_*>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,DBObj_*>_*>` · 006da0e0 · legions 00663520 — Rust std replaces it

## `portal/shared/src/database/masterdbmap.cpp.c` (0/1)

### HashTable<unsigned_long,DBTypeDef>
- [-] `HashTable<unsigned_long,DBTypeDef>::add` · 006d45a0 · legions 0065e2c0 — Rust std replaces it

## `portal/shared/src/database/masterdbmapmain.cpp.c` (0/2)

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,DBTypeDef>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,DBTypeDef>_*>::delete_contents` · 006d7120 · legions 00660e40 — Rust std replaces it
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,DBTypeDef>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,DBTypeDef>_*>` · 006d7600 · legions 00661320 — Rust std replaces it

## `portal/shared/src/database/qualifieddataid.cpp.c` (0/2)

### _STL::_Rb_tree<QualifiedDataID,QualifiedDataID,_STL::_Identity<QualifiedDataID>,_STL::less<QualifiedDataID>,_STL::allocator<QualifiedDataID>_>
- [-] `_STL::_Rb_tree<QualifiedDataID,QualifiedDataID,_STL::_Identity<QualifiedDataID>,_STL::less<QualifiedDataID>,_STL::allocator<QualifiedDataID>_>::_M_insert` · 006d7d90 · legions 00661910 — Rust std replaces it
- [-] `_STL::_Rb_tree<QualifiedDataID,QualifiedDataID,_STL::_Identity<QualifiedDataID>,_STL::less<QualifiedDataID>,_STL::allocator<QualifiedDataID>_>::insert_unique` · 006d7e60 · legions 006619e0 — Rust std replaces it

## `portal/shared/src/enummap/enummapper.cpp.c` (0/13)

### _STL::_Rb_tree<PStringBase<char>,_STL::pair<PStringBase<char>_const_,unsigned_long>,_STL::_Select1st<_STL::pair<PStringBase<char>_const_,unsigned_long>_>,CaseInsensitivePStringCompare,_STL::allocator<_STL::pair<PStringBase<char>_const_,unsigned_long>_>_>
- [-] `_STL::_Rb_tree<PStringBase<char>,_STL::pair<PStringBase<char>_const_,unsigned_long>,_STL::_Select1st<_STL::pair<PStringBase<char>_const_,unsigned_long>_>,CaseInsensitivePStringCompare,_STL::allocator<_STL::pair<PStringBase<char>_const_,unsigned_long>_>_>::_M_lower_bound` · 00660910 — Rust std replaces it

### _STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned_long>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned_long>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>::find` · 00660960 · legions 0060f280 — Rust std replaces it

### _STL::_Rb_tree<PStringBase<char>,_STL::pair<PStringBase<char>_const_,unsigned_long>,_STL::_Select1st<_STL::pair<PStringBase<char>_const_,unsigned_long>_>,CaseInsensitivePStringCompare,_STL::allocator<_STL::pair<PStringBase<char>_const_,unsigned_long>_>_>
- [-] `_STL::_Rb_tree<PStringBase<char>,_STL::pair<PStringBase<char>_const_,unsigned_long>,_STL::_Select1st<_STL::pair<PStringBase<char>_const_,unsigned_long>_>,CaseInsensitivePStringCompare,_STL::allocator<_STL::pair<PStringBase<char>_const_,unsigned_long>_>_>::_M_find` · 006609b0 · legions 0060f2d0 — Rust std replaces it

### _STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,PStringBase<char>_>,_STL::_Select1st<_STL::pair<unsigned_long_const_,PStringBase<char>_>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,PStringBase<char>_>_>_>
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,PStringBase<char>_>,_STL::_Select1st<_STL::pair<unsigned_long_const_,PStringBase<char>_>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,PStringBase<char>_>_>_>::_M_erase` · 006612e0 · legions 0060fba0 — Rust std replaces it

### _STL::_Rb_tree<PStringBase<char>,_STL::pair<PStringBase<char>_const_,unsigned_long>,_STL::_Select1st<_STL::pair<PStringBase<char>_const_,unsigned_long>_>,CaseInsensitivePStringCompare,_STL::allocator<_STL::pair<PStringBase<char>_const_,unsigned_long>_>_>
- [-] `_STL::_Rb_tree<PStringBase<char>,_STL::pair<PStringBase<char>_const_,unsigned_long>,_STL::_Select1st<_STL::pair<PStringBase<char>_const_,unsigned_long>_>,CaseInsensitivePStringCompare,_STL::allocator<_STL::pair<PStringBase<char>_const_,unsigned_long>_>_>::_M_erase` · 00661350 · legions 0060fc10 — Rust std replaces it

### _STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,PStringBase<char>_>,_STL::_Select1st<_STL::pair<unsigned_long_const_,PStringBase<char>_>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,PStringBase<char>_>_>_>
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,PStringBase<char>_>,_STL::_Select1st<_STL::pair<unsigned_long_const_,PStringBase<char>_>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,PStringBase<char>_>_>_>::_M_insert` · 006613c0 · legions 0060fc80 — Rust std replaces it
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,PStringBase<char>_>,_STL::_Select1st<_STL::pair<unsigned_long_const_,PStringBase<char>_>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,PStringBase<char>_>_>_>::_M_copy` · 006614a0 — Rust std replaces it
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,PStringBase<char>_>,_STL::_Select1st<_STL::pair<unsigned_long_const_,PStringBase<char>_>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,PStringBase<char>_>_>_>::insert_unique` · 006615b0 · legions 0060fd60 — Rust std replaces it

### _STL::_Rb_tree<PStringBase<char>,_STL::pair<PStringBase<char>_const_,unsigned_long>,_STL::_Select1st<_STL::pair<PStringBase<char>_const_,unsigned_long>_>,CaseInsensitivePStringCompare,_STL::allocator<_STL::pair<PStringBase<char>_const_,unsigned_long>_>_>
- [-] `_STL::_Rb_tree<PStringBase<char>,_STL::pair<PStringBase<char>_const_,unsigned_long>,_STL::_Select1st<_STL::pair<PStringBase<char>_const_,unsigned_long>_>,CaseInsensitivePStringCompare,_STL::allocator<_STL::pair<PStringBase<char>_const_,unsigned_long>_>_>::_M_insert` · 00661670 · legions 0060fe20 — Rust std replaces it
- [-] `_STL::_Rb_tree<PStringBase<char>,_STL::pair<PStringBase<char>_const_,unsigned_long>,_STL::_Select1st<_STL::pair<PStringBase<char>_const_,unsigned_long>_>,CaseInsensitivePStringCompare,_STL::allocator<_STL::pair<PStringBase<char>_const_,unsigned_long>_>_>::_M_copy` · 00661760 — Rust std replaces it
- [-] `_STL::_Rb_tree<PStringBase<char>,_STL::pair<PStringBase<char>_const_,unsigned_long>,_STL::_Select1st<_STL::pair<PStringBase<char>_const_,unsigned_long>_>,CaseInsensitivePStringCompare,_STL::allocator<_STL::pair<PStringBase<char>_const_,unsigned_long>_>_>::insert_unique` · 00661870 · legions 0060ff10 — Rust std replaces it

### _STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,PStringBase<char>_>,_STL::_Select1st<_STL::pair<unsigned_long_const_,PStringBase<char>_>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,PStringBase<char>_>_>_>
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,PStringBase<char>_>,_STL::_Select1st<_STL::pair<unsigned_long_const_,PStringBase<char>_>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,PStringBase<char>_>_>_>::insert_unique` · 00661a10 · legions 0060fff0 — Rust std replaces it

### _STL::_Rb_tree<PStringBase<char>,_STL::pair<PStringBase<char>_const_,unsigned_long>,_STL::_Select1st<_STL::pair<PStringBase<char>_const_,unsigned_long>_>,CaseInsensitivePStringCompare,_STL::allocator<_STL::pair<PStringBase<char>_const_,unsigned_long>_>_>
- [-] `_STL::_Rb_tree<PStringBase<char>,_STL::pair<PStringBase<char>_const_,unsigned_long>,_STL::_Select1st<_STL::pair<PStringBase<char>_const_,unsigned_long>_>,CaseInsensitivePStringCompare,_STL::allocator<_STL::pair<PStringBase<char>_const_,unsigned_long>_>_>::insert_unique` · 00661cb0 · legions 006101d0 — Rust std replaces it

## `portal/shared/src/enummap/penummapper.cpp.c` (0/15)

### _STL
- [-] `_STL::__median` · 00738260 · legions 006b3e20 — Rust std replaces it
- [-] `_STL::__unguarded_linear_insert` · 007385f0 · legions 006b41b0 — Rust std replaces it
- [-] `_STL::__push_heap` · 00738650 · legions 006b4210 — Rust std replaces it
- [-] `_STL::__copy_backward` · 007386e0 · legions 006b42a0 — Rust std replaces it
- [-] `_STL::__unguarded_insertion_sort_aux` · 00738860 · legions 006b4420 — Rust std replaces it
- [-] `_STL::__adjust_heap` · 007388f0 · legions 006b44b0 — Rust std replaces it
- [-] `_STL::__make_heap` · 00738a00 · legions 006b45c0 — Rust std replaces it
- [-] `_STL::__unguarded_partition` · 00738ad0 · legions 006b4690 — Rust std replaces it
- [-] `_STL::__linear_insert` · 00738be0 · legions 006b47a0 — Rust std replaces it
- [-] `_STL::sort_heap` · 00738c80 · legions 006b4840 — Rust std replaces it
- [-] `_STL::__insertion_sort` · 00738dc0 · legions 006b4980 — Rust std replaces it
- [-] `_STL::__partial_sort` · 00738e10 · legions 006b49d0 — Rust std replaces it
- [-] `_STL::__final_insertion_sort` · 00738fc0 · legions 006b4b80 — Rust std replaces it
- [-] `_STL::__introsort_loop` · 007391e0 · legions 006b4da0 — Rust std replaces it
- [-] `_STL::sort` · 007393a0 · legions 006b4ee0 — Rust std replaces it

## `portal/shared/src/error/dataerror/dataerror.cpp.c` (0/8)

### _STL::_Rb_tree<PStringBase<char>,_STL::pair<PStringBase<char>_const_,PStringBase<char>_>,_STL::_Select1st<_STL::pair<PStringBase<char>_const_,PStringBase<char>_>_>,_STL::less<PStringBase<char>_>,_STL::allocator<_STL::pair<PStringBase<char>_const_,PStringBase<char>_>_>_>
- [-] `_STL::_Rb_tree<PStringBase<char>,_STL::pair<PStringBase<char>_const_,PStringBase<char>_>,_STL::_Select1st<_STL::pair<PStringBase<char>_const_,PStringBase<char>_>_>,_STL::less<PStringBase<char>_>,_STL::allocator<_STL::pair<PStringBase<char>_const_,PStringBase<char>_>_>_>::_M_find` · 00615010 · legions 005d2d40 — Rust std replaces it
- [-] `_STL::_Rb_tree<PStringBase<char>,_STL::pair<PStringBase<char>_const_,PStringBase<char>_>,_STL::_Select1st<_STL::pair<PStringBase<char>_const_,PStringBase<char>_>_>,_STL::less<PStringBase<char>_>,_STL::allocator<_STL::pair<PStringBase<char>_const_,PStringBase<char>_>_>_>::_M_lower_bound` · 006150c0 · legions 005d2df0 — Rust std replaces it
- [-] `_STL::_Rb_tree<PStringBase<char>,_STL::pair<PStringBase<char>_const_,PStringBase<char>_>,_STL::_Select1st<_STL::pair<PStringBase<char>_const_,PStringBase<char>_>_>,_STL::less<PStringBase<char>_>,_STL::allocator<_STL::pair<PStringBase<char>_const_,PStringBase<char>_>_>_>::_M_create_node` · 006153e0 · legions 005d3110 — Rust std replaces it
- [-] `_STL::_Rb_tree<PStringBase<char>,_STL::pair<PStringBase<char>_const_,PStringBase<char>_>,_STL::_Select1st<_STL::pair<PStringBase<char>_const_,PStringBase<char>_>_>,_STL::less<PStringBase<char>_>,_STL::allocator<_STL::pair<PStringBase<char>_const_,PStringBase<char>_>_>_>::_M_erase` · 00615420 · legions 005d3150 — Rust std replaces it
- [-] `_STL::_Rb_tree<PStringBase<char>,_STL::pair<PStringBase<char>_const_,PStringBase<char>_>,_STL::_Select1st<_STL::pair<PStringBase<char>_const_,PStringBase<char>_>_>,_STL::less<PStringBase<char>_>,_STL::allocator<_STL::pair<PStringBase<char>_const_,PStringBase<char>_>_>_>::_M_insert` · 006154b0 · legions 005d31e0 — Rust std replaces it
- [-] `_STL::_Rb_tree<PStringBase<char>,_STL::pair<PStringBase<char>_const_,PStringBase<char>_>,_STL::_Select1st<_STL::pair<PStringBase<char>_const_,PStringBase<char>_>_>,_STL::less<PStringBase<char>_>,_STL::allocator<_STL::pair<PStringBase<char>_const_,PStringBase<char>_>_>_>::insert_unique` · 00615590 · legions 005d32c0 — Rust std replaces it
- [-] `_STL::_Rb_tree<PStringBase<char>,_STL::pair<PStringBase<char>_const_,PStringBase<char>_>,_STL::_Select1st<_STL::pair<PStringBase<char>_const_,PStringBase<char>_>_>,_STL::less<PStringBase<char>_>,_STL::allocator<_STL::pair<PStringBase<char>_const_,PStringBase<char>_>_>_>::erase` · 006156a0 · legions 005d33d0 — Rust std replaces it
- [-] `_STL::_Rb_tree<PStringBase<char>,_STL::pair<PStringBase<char>_const_,PStringBase<char>_>,_STL::_Select1st<_STL::pair<PStringBase<char>_const_,PStringBase<char>_>_>,_STL::less<PStringBase<char>_>,_STL::allocator<_STL::pair<PStringBase<char>_const_,PStringBase<char>_>_>_>::insert_unique` · 00615720 · legions 005d3450 — Rust std replaces it

## `portal/shared/src/filedb/dbfile2idtable.cpp.c` (0/1)

### IntrusiveHashTable<CaseInsensitiveStringBase<PStringBase<char>_>,HashTableData<CaseInsensitiveStringBase<PStringBase<char>_>,TFileEntry_*>_*>
- [-] `IntrusiveHashTable<CaseInsensitiveStringBase<PStringBase<char>_>,HashTableData<CaseInsensitiveStringBase<PStringBase<char>_>,TFileEntry_*>_*>::resize_internal` · 0066d580 · legions 00662930 — Rust std replaces it

## `portal/shared/src/filedb/file2idtable.cpp.c` (0/8)

### IntrusiveHashTable<CaseInsensitiveStringBase<PStringBase<char>_>,HashTableData<CaseInsensitiveStringBase<PStringBase<char>_>,TFileEntry_*>_*>
- [-] `IntrusiveHashTable<CaseInsensitiveStringBase<PStringBase<char>_>,HashTableData<CaseInsensitiveStringBase<PStringBase<char>_>,TFileEntry_*>_*>::add` · 0066e240 · legions 006189a0 — Rust std replaces it

### HashTable<CaseInsensitiveStringBase<PStringBase<char>_>,TFileEntry_*>
- [-] `HashTable<CaseInsensitiveStringBase<PStringBase<char>_>,TFileEntry_*>::add` · 0066ea00 · legions 006190c0 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,TFileEntry_*>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,TFileEntry_*>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,TFileEntry_*>_*>` · 0066eca0 · legions 00619360 — Rust std replaces it

### IntrusiveHashTable<CaseInsensitiveStringBase<PStringBase<char>_>,HashTableData<CaseInsensitiveStringBase<PStringBase<char>_>,TFileEntry_*>_*>
- [-] `IntrusiveHashTable<CaseInsensitiveStringBase<PStringBase<char>_>,HashTableData<CaseInsensitiveStringBase<PStringBase<char>_>,TFileEntry_*>_*>::IntrusiveHashTable<CaseInsensitiveStringBase<PStringBase<char>_>,HashTableData<CaseInsensitiveStringBase<PStringBase<char>_>,TFileEntry_*>_*>` · 0066ed30 · legions 006193f0 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,TDBTypeEntry_*>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,TDBTypeEntry_*>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,TDBTypeEntry_*>_*>` · 0066edc0 · legions 00619480 — Rust std replaces it

### HashTable<unsigned_long,unsigned___int64>
- [-] `HashTable<unsigned_long,unsigned___int64>::grow` · 0066ee50 · legions 0065b9f0 — Rust std replaces it

### HashTable<CaseInsensitiveStringBase<PStringBase<char>_>,TFileEntry_*>
- [-] `HashTable<CaseInsensitiveStringBase<PStringBase<char>_>,TFileEntry_*>::grow` · 0066eef0 · legions 00619510 — Rust std replaces it

### AutoGrowHashTable<unsigned_long,TFileEntry_*>
- [-] `AutoGrowHashTable<unsigned_long,TFileEntry_*>::add` · 0066f100 · legions 00619770 — Rust std replaces it

## `portal/shared/src/filedb/filepathcache.cpp.c` (0/5)

### IntrusiveHashTable<PStringBase<char>,HashTableData<PStringBase<char>,FrameDebugInfo>_*>
- [-] `IntrusiveHashTable<PStringBase<char>,HashTableData<PStringBase<char>,FrameDebugInfo>_*>::add_internal` · 00671d60 · legions 00566f70 — Rust std replaces it

### IntrusiveHashTable<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>_*>
- [-] `IntrusiveHashTable<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>_*>::delete_contents` · 00671db0 · legions 004084c0 — Rust std replaces it

### IntrusiveHashTable<PStringBase<char>,HashTableData<PStringBase<char>,PStringBase<char>_>_*>
- [-] `IntrusiveHashTable<PStringBase<char>,HashTableData<PStringBase<char>,PStringBase<char>_>_*>::IntrusiveHashTable<PStringBase<char>,HashTableData<PStringBase<char>,PStringBase<char>_>_*>` · 00671e70 · legions 0061c430 — Rust std replaces it

### HashTable<PStringBase<char>,PStringBase<char>_>
- [-] `HashTable<PStringBase<char>,PStringBase<char>_>::set` · 00671f00 · legions 0061c4c0 — Rust std replaces it

### IntrusiveHashTable<PStringBase<char>,HashTableData<PStringBase<char>,PStringBase<char>_>_*>
- [-] `IntrusiveHashTable<PStringBase<char>,HashTableData<PStringBase<char>,PStringBase<char>_>_*>::add_for_unpacking` · 00672260 · legions 0061c820 — Rust std replaces it

## `portal/shared/src/fileformat/fileformat.cpp.c` (0/14)

### _STL::_Rb_tree<PStringBase<char>,_STL::pair<PStringBase<char>_const_,unsigned_long>,_STL::_Select1st<_STL::pair<PStringBase<char>_const_,unsigned_long>_>,_STL::less<PStringBase<char>_>,_STL::allocator<_STL::pair<PStringBase<char>_const_,unsigned_long>_>_>
- [-] `_STL::_Rb_tree<PStringBase<char>,_STL::pair<PStringBase<char>_const_,unsigned_long>,_STL::_Select1st<_STL::pair<PStringBase<char>_const_,unsigned_long>_>,_STL::less<PStringBase<char>_>,_STL::allocator<_STL::pair<PStringBase<char>_const_,unsigned_long>_>_>::_M_upper_bound` · 00666410 — Rust std replaces it

### _STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,FileTemplate_*>,_STL::_Select1st<_STL::pair<unsigned_long_const_,FileTemplate_*>_>,_STL::greater<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,FileTemplate_*>_>_>
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,FileTemplate_*>,_STL::_Select1st<_STL::pair<unsigned_long_const_,FileTemplate_*>_>,_STL::greater<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,FileTemplate_*>_>_>::find` · 006669d0 · legions 00613c80 — Rust std replaces it
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,FileTemplate_*>,_STL::_Select1st<_STL::pair<unsigned_long_const_,FileTemplate_*>_>,_STL::greater<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,FileTemplate_*>_>_>::_M_erase` · 00667f70 · legions 006140f0 — Rust std replaces it

### _STL::_List_base<TemplateInstance,_STL::allocator<TemplateInstance>_>
- [-] `_STL::_List_base<TemplateInstance,_STL::allocator<TemplateInstance>_>::clear` · 00667fc0 · legions 00614140 — Rust std replaces it

### _STL::_Rb_tree<PStringBase<char>,_STL::pair<PStringBase<char>_const_,unsigned_long>,_STL::_Select1st<_STL::pair<PStringBase<char>_const_,unsigned_long>_>,_STL::less<PStringBase<char>_>,_STL::allocator<_STL::pair<PStringBase<char>_const_,unsigned_long>_>_>
- [-] `_STL::_Rb_tree<PStringBase<char>,_STL::pair<PStringBase<char>_const_,unsigned_long>,_STL::_Select1st<_STL::pair<PStringBase<char>_const_,unsigned_long>_>,_STL::less<PStringBase<char>_>,_STL::allocator<_STL::pair<PStringBase<char>_const_,unsigned_long>_>_>::_M_erase` · 00668030 · legions 006141b0 — Rust std replaces it

### _STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,FileTemplate_*>,_STL::_Select1st<_STL::pair<unsigned_long_const_,FileTemplate_*>_>,_STL::greater<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,FileTemplate_*>_>_>
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,FileTemplate_*>,_STL::_Select1st<_STL::pair<unsigned_long_const_,FileTemplate_*>_>,_STL::greater<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,FileTemplate_*>_>_>::_M_insert` · 006680a0 — Rust std replaces it
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,FileTemplate_*>,_STL::_Select1st<_STL::pair<unsigned_long_const_,FileTemplate_*>_>,_STL::greater<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,FileTemplate_*>_>_>::insert_unique` · 00668160 — Rust std replaces it

### _STL::_Rb_tree<PStringBase<char>,_STL::pair<PStringBase<char>_const_,unsigned_long>,_STL::_Select1st<_STL::pair<PStringBase<char>_const_,unsigned_long>_>,_STL::less<PStringBase<char>_>,_STL::allocator<_STL::pair<PStringBase<char>_const_,unsigned_long>_>_>
- [-] `_STL::_Rb_tree<PStringBase<char>,_STL::pair<PStringBase<char>_const_,unsigned_long>,_STL::_Select1st<_STL::pair<PStringBase<char>_const_,unsigned_long>_>,_STL::less<PStringBase<char>_>,_STL::allocator<_STL::pair<PStringBase<char>_const_,unsigned_long>_>_>::_M_insert` · 00668220 — Rust std replaces it
- [-] `_STL::_Rb_tree<PStringBase<char>,_STL::pair<PStringBase<char>_const_,unsigned_long>,_STL::_Select1st<_STL::pair<PStringBase<char>_const_,unsigned_long>_>,_STL::less<PStringBase<char>_>,_STL::allocator<_STL::pair<PStringBase<char>_const_,unsigned_long>_>_>::insert_unique` · 00668310 — Rust std replaces it
- [-] `_STL::_Rb_tree<PStringBase<char>,_STL::pair<PStringBase<char>_const_,unsigned_long>,_STL::_Select1st<_STL::pair<PStringBase<char>_const_,unsigned_long>_>,_STL::less<PStringBase<char>_>,_STL::allocator<_STL::pair<PStringBase<char>_const_,unsigned_long>_>_>::erase` · 006684c0 — Rust std replaces it

### _STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,FileTemplate_*>,_STL::_Select1st<_STL::pair<unsigned_long_const_,FileTemplate_*>_>,_STL::greater<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,FileTemplate_*>_>_>
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,FileTemplate_*>,_STL::_Select1st<_STL::pair<unsigned_long_const_,FileTemplate_*>_>,_STL::greater<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,FileTemplate_*>_>_>::insert_unique` · 006685d0 — Rust std replaces it

### _STL::_Rb_tree<PStringBase<char>,_STL::pair<PStringBase<char>_const_,unsigned_long>,_STL::_Select1st<_STL::pair<PStringBase<char>_const_,unsigned_long>_>,_STL::less<PStringBase<char>_>,_STL::allocator<_STL::pair<PStringBase<char>_const_,unsigned_long>_>_>
- [-] `_STL::_Rb_tree<PStringBase<char>,_STL::pair<PStringBase<char>_const_,unsigned_long>,_STL::_Select1st<_STL::pair<PStringBase<char>_const_,unsigned_long>_>,_STL::less<PStringBase<char>_>,_STL::allocator<_STL::pair<PStringBase<char>_const_,unsigned_long>_>_>::insert_unique` · 006687b0 — Rust std replaces it
- [-] `_STL::_Rb_tree<PStringBase<char>,_STL::pair<PStringBase<char>_const_,unsigned_long>,_STL::_Select1st<_STL::pair<PStringBase<char>_const_,unsigned_long>_>,_STL::less<PStringBase<char>_>,_STL::allocator<_STL::pair<PStringBase<char>_const_,unsigned_long>_>_>::erase` · 006689b0 — Rust std replaces it

### _STL::list<TemplateInstance,_STL::allocator<TemplateInstance>_>
- [-] `_STL::list<TemplateInstance,_STL::allocator<TemplateInstance>_>::push_back` · 00668af0 — Rust std replaces it

## `portal/shared/src/filenode/pfilenode.cpp.c` (0/7)

### _STL
- [-] `_STL::__median` · 00663200 · legions 0056e300 — Rust std replaces it
- [-] `_STL::__adjust_heap` · 006637e0 · legions 00611690 — Rust std replaces it
- [-] `_STL::__insertion_sort` · 006651b0 · legions 00612bc0 — Rust std replaces it
- [-] `_STL::sort_heap` · 00665220 · legions 0056ee50 — Rust std replaces it
- [-] `_STL::__partial_sort` · 006652a0 · legions 0056ef80 — Rust std replaces it
- [-] `_STL::__introsort_loop` · 00665320 · legions 00612c60 — Rust std replaces it
- [-] `_STL::sort` · 006653b0 · legions 00612cf0 — Rust std replaces it

## `portal/shared/src/gameplay/character/characterset.cpp.c` (0/1)

### SmartArray<CharacterIdentity>
- [-] `SmartArray<CharacterIdentity>::grow` · 006db300 · legions 00664770 — Rust std replaces it

## `portal/shared/src/gameplay/qualities/basequalities.cpp.c` (0/9)

### HashTableData<unsigned_long,Position>
- [-] `HashTableData<unsigned_long,Position>::HashTableData<unsigned_long,Position>` · 005170f0 · legions 00505120 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,long>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,long>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,long>_*>` · 00517b90 · legions 00505d40 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,__int64>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,__int64>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,__int64>_*>` · 00517c20 · legions 00505dd0 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,double>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,double>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,double>_*>` · 00517cb0 · legions 00505e60 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,Position>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,Position>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,Position>_*>` · 00517d40 · legions 00505ef0 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,StringInfo>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,StringInfo>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,StringInfo>_*>` · 00517dd0 · legions 00505f80 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,long>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,long>_*>::resize` · 00517e60 · legions 00506010 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,unsigned_long>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,unsigned_long>_*>::optimize_size` · 00517eb0 · legions 00506060 — Rust std replaces it

### HashTableData<unsigned_long,PStringBase<char>_>
- [-] `HashTableData<unsigned_long,PStringBase<char>_>::HashTableData<unsigned_long,PStringBase<char>_>` · 005189a0 · legions 00506b50 — Rust std replaces it

## `portal/shared/src/gameplay/qualities/basequalitiesstats.cpp.c` (0/4)

### HashTable<unsigned_long,PStringBase<char>_>
- [-] `HashTable<unsigned_long,PStringBase<char>_>::remove` · 00519b20 · legions 00507cd0 — Rust std replaces it

### HashTable<unsigned_long,unsigned_long>
- [-] `HashTable<unsigned_long,unsigned_long>::add` · 0051a500 · legions 004c9330 — Rust std replaces it

### HashTable<unsigned_long,unsigned___int64>
- [-] `HashTable<unsigned_long,unsigned___int64>::add` · 0051a560 · legions 005086b0 — Rust std replaces it

### HashTable<unsigned_long,Position>
- [-] `HashTable<unsigned_long,Position>::set` · 0051a5c0 · legions 00508710 — Rust std replaces it

## `portal/shared/src/gameplay/uisavelocations.cpp.c` (0/5)

### HashTable<unsigned_long,UILocationData>
- [-] `HashTable<unsigned_long,UILocationData>::find` · 006da9b0 · legions 00663e20 — Rust std replaces it
- [-] `HashTable<unsigned_long,UILocationData>::add` · 006daa00 · legions 00663e70 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PHashTable<unsigned_long,UILocationData>_>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PHashTable<unsigned_long,UILocationData>_>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PHashTable<unsigned_long,UILocationData>_>_*>` · 006dabb0 · legions 00664020 — Rust std replaces it

### HashTableData<unsigned_long,PHashTable<unsigned_long,UILocationData>_>
- [-] `HashTableData<unsigned_long,PHashTable<unsigned_long,UILocationData>_>::HashTableData<unsigned_long,PHashTable<unsigned_long,UILocationData>_>` · 006dac40 · legions 006640b0 — Rust std replaces it

### HashTable<unsigned_long,PHashTable<unsigned_long,UILocationData>_>
- [-] `HashTable<unsigned_long,PHashTable<unsigned_long,UILocationData>_>::set` · 006daca0 · legions 00664110 — Rust std replaces it

## `portal/shared/src/geometry/polygon.cpp.c` (0/1)

### SmartArray<CPolygon>
- [-] `SmartArray<CPolygon>::grow` · 0049a780 · legions 00492c70 — Rust std replaces it

## `portal/shared/src/geometry/simd.cpp.c` (0/1)

### (free functions)
- [-] `_Matrix4TransformVector4` · 0062b2d0 — Rust std replaces it

## `portal/shared/src/geometry/simplepolygon.cpp.c` (0/2)

### SmartArray<SimplePolygon>
- [-] `SmartArray<SimplePolygon>::Reset` · 00626c30 · legions 005e1b80 — Rust std replaces it
- [-] `SmartArray<SimplePolygon>::grow` · 00626f30 · legions 005e1e80 — Rust std replaces it

## `portal/shared/src/geometry/vertexarray.cpp.c` (0/2)

### SmartArray<LOAD_UVPAIR>
- [-] `SmartArray<LOAD_UVPAIR>::grow` · 00730730 — Rust std replaces it

### SmartArray<LOAD_WEIGHTS>
- [-] `SmartArray<LOAD_WEIGHTS>::grow` · 00730910 — Rust std replaces it

## `portal/shared/src/interp/bytestream.cpp.c` (0/17)

### SmartArray<PackageInfo>
- [-] `SmartArray<PackageInfo>::grow` · 006e5d80 · legions 0066bf40 — Rust std replaces it

### SmartArray<VersionTable::ver>
- [-] `SmartArray<VersionTable::ver>::grow` · 006e60d0 · legions 0066c1f0 — Rust std replaces it

### SmartArray<PSmartArray<unsigned_long>_>
- [-] `SmartArray<PSmartArray<unsigned_long>_>::grow` · 006e6520 · legions 0066c5d0 — Rust std replaces it

### SmartArray<PSmartArray<VTableID>_>
- [-] `SmartArray<PSmartArray<VTableID>_>::grow` · 006e66c0 · legions 0066c770 — Rust std replaces it

### SmartArray<gmKeyframe>
- [-] `SmartArray<gmKeyframe>::Reset` · 006e67c0 · legions 004b33a0 — Rust std replaces it

### PSmartArray<VTableID>
- [-] `PSmartArray<VTableID>::PSmartArray<VTableID>` · 006e6800 — Rust std replaces it

### SmartArray<VTableID>
- [-] `SmartArray<VTableID>::grow` · 006e6880 · legions 0066c8f0 — Rust std replaces it

### SmartArray<POriginalSourceFileInfo>
- [-] `SmartArray<POriginalSourceFileInfo>::grow` · 006e6b00 · legions 0066caf0 — Rust std replaces it

### IntrusiveHashTable<FunctionID,HashTableData<FunctionID,int>_*>
- [-] `IntrusiveHashTable<FunctionID,HashTableData<FunctionID,int>_*>::IntrusiveHashTable<FunctionID,HashTableData<FunctionID,int>_*>` · 006e72f0 · legions 0066d020 — Rust std replaces it

### SmartArray<FrameDebugInfo>
- [-] `SmartArray<FrameDebugInfo>::grow` · 006e77f0 · legions 0066d380 — Rust std replaces it

### SmartArray<ImportData>
- [-] `SmartArray<ImportData>::grow` · 006e7d50 · legions 0066d960 — Rust std replaces it

### SmartArray<ImportSymbolInfo>
- [-] `SmartArray<ImportSymbolInfo>::grow` · 006e7f20 · legions 0066db30 — Rust std replaces it

### SmartArray<PLineOffsetList>
- [-] `SmartArray<PLineOffsetList>::grow` · 006e82e0 · legions 0066def0 — Rust std replaces it

### SmartArray<PLineOffsetInfo>
- [-] `SmartArray<PLineOffsetInfo>::grow` · 006e84a0 · legions 0066e0b0 — Rust std replaces it

### IntrusiveHashTable<PStringBase<char>,HashTableData<PStringBase<char>,unsigned_long>_*>
- [-] `IntrusiveHashTable<PStringBase<char>,HashTableData<PStringBase<char>,unsigned_long>_*>::add_for_unpacking` · 006e8880 · legions 0066e490 — Rust std replaces it

### SmartArray<ExportData>
- [-] `SmartArray<ExportData>::grow` · 006e8f30 · legions 0066eb40 — Rust std replaces it

### SmartArray<ExportFunctionData>
- [-] `SmartArray<ExportFunctionData>::grow` · 006e90e0 · legions 0066ecf0 — Rust std replaces it

## `portal/shared/src/interp/centraldebuginfo.cpp.c` (0/16)

### _STL
- [-] `_STL::__push_heap` · 006e98c0 · legions 0066f4d0 — Rust std replaces it
- [-] `_STL::__unguarded_insertion_sort_aux` · 006e9b80 · legions 0066f700 — Rust std replaces it
- [-] `_STL::__adjust_heap` · 006e9bc0 · legions 0066f740 — Rust std replaces it
- [-] `_STL::__unguarded_partition` · 006e9d90 · legions 0066f840 — Rust std replaces it
- [-] `_STL::__make_heap` · 006e9df0 · legions 0066f8a0 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,OriginalSourceFileInfo>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,OriginalSourceFileInfo>_*>::delete_contents` · 006e9e50 · legions 0066f900 — Rust std replaces it

### IntrusiveHashTable<PStringBase<char>,HashTableData<PStringBase<char>,FrameDebugInfo>_*>
- [-] `IntrusiveHashTable<PStringBase<char>,HashTableData<PStringBase<char>,FrameDebugInfo>_*>::delete_contents` · 006ea2d0 · legions 0066fcc0 — Rust std replaces it

### _STL
- [-] `_STL::__insertion_sort` · 006ea4a0 · legions 0066fed0 — Rust std replaces it
- [-] `_STL::sort_heap` · 006ea530 · legions 0066ff60 — Rust std replaces it

### List<FunctionLocationInfo>
- [-] `List<FunctionLocationInfo>::flush` · 006eaaf0 · legions 00670450 — Rust std replaces it

### _STL
- [-] `_STL::__partial_sort` · 006eabf0 · legions 00670550 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,OriginalSourceFileInfo>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,OriginalSourceFileInfo>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,OriginalSourceFileInfo>_*>` · 006eb240 · legions 006707a0 — Rust std replaces it

### IntrusiveHashTable<PStringBase<char>,HashTableData<PStringBase<char>,FrameDebugInfo>_*>
- [-] `IntrusiveHashTable<PStringBase<char>,HashTableData<PStringBase<char>,FrameDebugInfo>_*>::IntrusiveHashTable<PStringBase<char>,HashTableData<PStringBase<char>,FrameDebugInfo>_*>` · 006eb2d0 · legions 00670830 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,FunctionLocationInfo_*>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,FunctionLocationInfo_*>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,FunctionLocationInfo_*>_*>` · 006eb360 · legions 006708c0 — Rust std replaces it

### _STL
- [-] `_STL::__introsort_loop` · 006eb7a0 · legions 00670d00 — Rust std replaces it
- [-] `_STL::__introsort_loop` · 006eb860 · legions 00670dc0 — Rust std replaces it

## `portal/shared/src/interp/debugdata.cpp.c` (0/1)

### SmartArray<PFunctionLocationInfo>
- [-] `SmartArray<PFunctionLocationInfo>::grow` · 006ec600 · legions 006714d0 — Rust std replaces it

## `portal/shared/src/interp/debugger/debug_cmds.cpp.c` (0/2)

### WslDebugger
- [-] `WslDebugger::__SetContextForStackTraceDump` · 006ee000 · legions 00672cd0 — Rust std replaces it

### HashTable<unsigned_long,BreakpointInfo>
- [-] `HashTable<unsigned_long,BreakpointInfo>::add` · 006efc00 — Rust std replaces it

## `portal/shared/src/interp/debugger/debuggingefuns.cpp.c` (0/2)

### IntrusiveHashTable<PStringBase<char>,HashSetData<PStringBase<char>_>_*>
- [-] `IntrusiveHashTable<PStringBase<char>,HashSetData<PStringBase<char>_>_*>::IntrusiveHashTable<PStringBase<char>,HashSetData<PStringBase<char>_>_*>` · 006f1470 · legions 006737a0 — Rust std replaces it

### (free functions)
- [-] `_DoWslAssertMessageBox` · 006f1500 · legions 00673830 — Rust std replaces it

## `portal/shared/src/interp/glue/gaamultimap.cpp.c` (0/3)

### _STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned_long>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned_long>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>::equal_range` · 0054f580 · legions 0052bd50 — Rust std replaces it
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned_long>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>::erase` · 0054f680 · legions 0052be30 — Rust std replaces it
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned_long>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>::erase` · 0054f770 · legions 0052bf20 — Rust std replaces it

## `portal/shared/src/interp/glue/galhash.cpp.c` (0/4)

### HashTable<unsigned_long,InstanceID>
- [-] `HashTable<unsigned_long,InstanceID>::find` · 00564c50 · legions 0056bdc0 — Rust std replaces it
- [-] `HashTable<unsigned_long,InstanceID>::set` · 00564dd0 · legions 004163b0 — Rust std replaces it

### HashTable<unsigned_long,unsigned___int64>
- [-] `HashTable<unsigned_long,unsigned___int64>::shrink` · 00564fa0 · legions 00536840 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashSetData<unsigned_long>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashSetData<unsigned_long>_*>::optimize_size` · 00564ff0 — Rust std replaces it

## `portal/shared/src/interp/glue/galmultihash.cpp.c` (0/2)

### _STL::hashtable<_STL::pair<unsigned_long_const_,unsigned___int64>,unsigned_long,_STL::hash<unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned___int64>_>,_STL::equal_to<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned___int64>_>_>
- [-] `_STL::hashtable<_STL::pair<unsigned_long_const_,unsigned___int64>,unsigned_long,_STL::hash<unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned___int64>_>,_STL::equal_to<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned___int64>_>_>::equal_range` · 00561da0 · legions 005354a0 — Rust std replaces it
- [-] `_STL::hashtable<_STL::pair<unsigned_long_const_,unsigned___int64>,unsigned_long,_STL::hash<unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned___int64>_>,_STL::equal_to<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned___int64>_>_>::erase` · 00561ef0 · legions 005355d0 — Rust std replaces it

## `portal/shared/src/interp/glue/galmultimap.cpp.c` (0/2)

### _STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned___int64>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned___int64>_>_>
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned___int64>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned___int64>_>_>::erase` · 0054e9c0 · legions 0052b550 — Rust std replaces it
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned___int64>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned___int64>_>_>::erase` · 0054eab0 · legions 0052b640 — Rust std replaces it

## `portal/shared/src/interp/glue/garhash.cpp.c` (0/2)

### HashTable<unsigned_long,unsigned_long>
- [-] `HashTable<unsigned_long,unsigned_long>::grow` · 00561550 · legions 00534ee0 — Rust std replaces it
- [-] `HashTable<unsigned_long,unsigned_long>::shrink` · 005615a0 · legions 00534f30 — Rust std replaces it

## `portal/shared/src/interp/glue/garmultihash.cpp.c` (0/3)

### _STL::hashtable<_STL::pair<unsigned_long_const_,unsigned_long>,unsigned_long,_STL::hash<unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned_long>_>,_STL::equal_to<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>
- [-] `_STL::hashtable<_STL::pair<unsigned_long_const_,unsigned_long>,unsigned_long,_STL::hash<unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned_long>_>,_STL::equal_to<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>::equal_range` · 005605a0 · legions 00534810 — Rust std replaces it
- [-] `_STL::hashtable<_STL::pair<unsigned_long_const_,unsigned_long>,unsigned_long,_STL::hash<unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned_long>_>,_STL::equal_to<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>::erase` · 005606f0 · legions 00534940 — Rust std replaces it
- [-] `_STL::hashtable<_STL::pair<unsigned_long_const_,unsigned_long>,unsigned_long,_STL::hash<unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned_long>_>,_STL::equal_to<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned_long>_>_>::erase` · 00560770 · legions 005349c0 — Rust std replaces it

## `portal/shared/src/interp/glue/gcaseinsensitivewpstring.cpp.c` (0/8)

### PStringBase<unsigned_short>
- [-] `PStringBase<unsigned_short>::ge` · 0054ff50 · legions 0052c590 — Rust std replaces it
- [-] `PStringBase<unsigned_short>::gt` · 0054ffa0 · legions 0052c5e0 — Rust std replaces it
- [-] `PStringBase<unsigned_short>::le` · 0054fff0 · legions 0052c630 — Rust std replaces it
- [-] `PStringBase<unsigned_short>::lt` · 00550040 · legions 0052c680 — Rust std replaces it
- [-] `PStringBase<unsigned_short>::ne` · 00550090 · legions 0052c6d0 — Rust std replaces it
- [-] `PStringBase<unsigned_short>::append_int32_hex` · 00550c80 · legions 0052c9d0 — Rust std replaces it
- [-] `PStringBase<unsigned_short>::append_int64` · 00550cf0 · legions 0052ca40 — Rust std replaces it
- [-] `PStringBase<unsigned_short>::append_float` · 00550d70 · legions 0052cac0 — Rust std replaces it

## `portal/shared/src/interp/glue/gfile.cpp.c` (0/2)

### IntrusiveHashTable<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,_iobuf_*>_*>
- [-] `IntrusiveHashTable<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,_iobuf_*>_*>::remove` · 00545610 · legions 005bfb60 — Rust std replaces it
- [-] `IntrusiveHashTable<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,_iobuf_*>_*>::IntrusiveHashTable<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,_iobuf_*>_*>` · 00545880 · legions 00524e80 — Rust std replaces it

## `portal/shared/src/interp/glue/glahashset.cpp.c` (0/1)

### IntrusiveHashTable<unsigned___int64,HashSetData<unsigned___int64>_*>
- [-] `IntrusiveHashTable<unsigned___int64,HashSetData<unsigned___int64>_*>::optimize_size` · 00551ad0 — Rust std replaces it

## `portal/shared/src/interp/glue/glamultimap.cpp.c` (0/5)

### _STL
- [-] `_STL::__distance` · 0054dba0 · legions 0056b7e0 — Rust std replaces it

### _STL::_Rb_tree<InstanceID,_STL::pair<InstanceID_const_,ObjectEventQueue>,_STL::_Select1st<_STL::pair<InstanceID_const_,ObjectEventQueue>_>,_STL::less<InstanceID>,_STL::allocator<_STL::pair<InstanceID_const_,ObjectEventQueue>_>_>
- [-] `_STL::_Rb_tree<InstanceID,_STL::pair<InstanceID_const_,ObjectEventQueue>,_STL::_Select1st<_STL::pair<InstanceID_const_,ObjectEventQueue>_>,_STL::less<InstanceID>,_STL::allocator<_STL::pair<InstanceID_const_,ObjectEventQueue>_>_>::_M_lower_bound` · 0054dc60 · legions 0052ad70 — Rust std replaces it

### _STL::_Rb_tree<unsigned___int64,_STL::pair<unsigned___int64_const_,unsigned_long>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned_long>_>,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned_long>_>_>
- [-] `_STL::_Rb_tree<unsigned___int64,_STL::pair<unsigned___int64_const_,unsigned_long>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned_long>_>,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned_long>_>_>::equal_range` · 0054dca0 · legions 0052a4e0 — Rust std replaces it
- [-] `_STL::_Rb_tree<unsigned___int64,_STL::pair<unsigned___int64_const_,unsigned_long>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned_long>_>,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned_long>_>_>::erase` · 0054dda0 · legions 0052ae40 — Rust std replaces it
- [-] `_STL::_Rb_tree<unsigned___int64,_STL::pair<unsigned___int64_const_,unsigned_long>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned_long>_>,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned_long>_>_>::erase` · 0054de90 · legions 0052af30 — Rust std replaces it

## `portal/shared/src/interp/glue/glarray.cpp.c` (0/1)

### SmartArray<unsigned___int64>
- [-] `SmartArray<unsigned___int64>::contains` · 0055e230 · legions 00533330 — Rust std replaces it

## `portal/shared/src/interp/glue/gllhash.cpp.c` (0/1)

### HashTable<unsigned___int64,unsigned___int64>
- [-] `HashTable<unsigned___int64,unsigned___int64>::add` · 0055da90 · legions 00532e40 — Rust std replaces it

## `portal/shared/src/interp/glue/gllist.cpp.c` (0/7)

### List<unsigned___int64>
- [-] `List<unsigned___int64>::get_node` · 0055b420 · legions 00532050 — Rust std replaces it
- [-] `List<unsigned___int64>::remove` · 0055b6f0 · legions 00532140 — Rust std replaces it
- [-] `List<unsigned___int64>::push` · 0055b740 · legions 00532190 — Rust std replaces it
- [-] `List<unsigned___int64>::pop_front` · 0055bd70 · legions 00532470 — Rust std replaces it
- [-] `List<unsigned___int64>::pop_back` · 0055bdc0 · legions 005324c0 — Rust std replaces it
- [-] `List<unsigned___int64>::push_unique_front` · 0055c2b0 · legions 00532780 — Rust std replaces it
- [-] `List<unsigned___int64>::push_unique_back` · 0055c300 · legions 005327d0 — Rust std replaces it

## `portal/shared/src/interp/glue/gllmultihash.cpp.c` (0/2)

### _STL::hashtable<_STL::pair<unsigned___int64_const_,unsigned___int64>,unsigned___int64,_STL::hash<unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned___int64>_>,_STL::equal_to<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned___int64>_>_>
- [-] `_STL::hashtable<_STL::pair<unsigned___int64_const_,unsigned___int64>,unsigned___int64,_STL::hash<unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned___int64>_>,_STL::equal_to<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned___int64>_>_>::count` · 0055a940 · legions 005319a0 — Rust std replaces it
- [-] `_STL::hashtable<_STL::pair<unsigned___int64_const_,unsigned___int64>,unsigned___int64,_STL::hash<unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned___int64>_>,_STL::equal_to<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned___int64>_>_>::equal_range` · 0055a990 · legions 005319f0 — Rust std replaces it

## `portal/shared/src/interp/glue/gllmultimap.cpp.c` (0/3)

### _STL::_Rb_tree<unsigned___int64,_STL::pair<unsigned___int64_const_,unsigned_long>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned_long>_>,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned_long>_>_>
- [-] `_STL::_Rb_tree<unsigned___int64,_STL::pair<unsigned___int64_const_,unsigned_long>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned_long>_>,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned_long>_>_>::_M_find` · 0054ceb0 · legions 0052ad10 — Rust std replaces it

### _STL::_Rb_tree<unsigned___int64,_STL::pair<unsigned___int64_const_,unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned___int64>_>,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned___int64>_>_>
- [-] `_STL::_Rb_tree<unsigned___int64,_STL::pair<unsigned___int64_const_,unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned___int64>_>,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned___int64>_>_>::erase` · 0054d030 · legions 0052a530 — Rust std replaces it
- [-] `_STL::_Rb_tree<unsigned___int64,_STL::pair<unsigned___int64_const_,unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned___int64>_>,_STL::less<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned___int64>_>_>::erase` · 0054d120 · legions 0052a620 — Rust std replaces it

## `portal/shared/src/interp/glue/glrhash.cpp.c` (0/4)

### IntrusiveHashTable<unsigned___int64,HashTableData<unsigned___int64,unsigned___int64>_*>
- [-] `IntrusiveHashTable<unsigned___int64,HashTableData<unsigned___int64,unsigned___int64>_*>::remove` · 00559cd0 · legions 004c3020 — Rust std replaces it

### HashTable<unsigned___int64,unsigned_long>
- [-] `HashTable<unsigned___int64,unsigned_long>::set` · 00559ee0 · legions 005313d0 — Rust std replaces it

### HashTable<unsigned___int64,unsigned___int64>
- [-] `HashTable<unsigned___int64,unsigned___int64>::grow` · 0055a0b0 · legions 00531560 — Rust std replaces it
- [-] `HashTable<unsigned___int64,unsigned___int64>::shrink` · 0055a100 · legions 005315b0 — Rust std replaces it

## `portal/shared/src/interp/glue/glrmultihash.cpp.c` (0/2)

### _STL::hashtable<_STL::pair<unsigned_long_const_,unsigned___int64>,unsigned_long,_STL::hash<unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned___int64>_>,_STL::equal_to<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned___int64>_>_>
- [-] `_STL::hashtable<_STL::pair<unsigned_long_const_,unsigned___int64>,unsigned_long,_STL::hash<unsigned_long>,_STL::_Select1st<_STL::pair<unsigned_long_const_,unsigned___int64>_>,_STL::equal_to<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,unsigned___int64>_>_>::erase` · 005590e0 · legions 00530e60 — Rust std replaces it

### _STL::hashtable<_STL::pair<unsigned___int64_const_,unsigned___int64>,unsigned___int64,_STL::hash<unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned___int64>_>,_STL::equal_to<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned___int64>_>_>
- [-] `_STL::hashtable<_STL::pair<unsigned___int64_const_,unsigned___int64>,unsigned___int64,_STL::hash<unsigned___int64>,_STL::_Select1st<_STL::pair<unsigned___int64_const_,unsigned___int64>_>,_STL::equal_to<unsigned___int64>,_STL::allocator<_STL::pair<unsigned___int64_const_,unsigned___int64>_>_>::erase` · 00559160 · legions 00530ee0 — Rust std replaces it

## `portal/shared/src/interp/glue/gnrhash.cpp.c` (0/5)

### IntrusiveHashTable<ReferenceIDWrapper,HashTableData<ReferenceIDWrapper,unsigned_long>_*>
- [-] `IntrusiveHashTable<ReferenceIDWrapper,HashTableData<ReferenceIDWrapper,unsigned_long>_*>::remove` · 00556de0 · legions 0052f920 — Rust std replaces it

### HashTable<ReferenceIDWrapper,unsigned_long>
- [-] `HashTable<ReferenceIDWrapper,unsigned_long>::set` · 00557010 · legions 0052fa10 — Rust std replaces it
- [-] `HashTable<ReferenceIDWrapper,unsigned_long>::grow` · 00557200 · legions 0052fba0 — Rust std replaces it
- [-] `HashTable<ReferenceIDWrapper,unsigned_long>::shrink` · 00557250 · legions 0052fbf0 — Rust std replaces it

### IntrusiveHashTable<ReferenceIDWrapper,HashTableData<ReferenceIDWrapper,unsigned_long>_*>
- [-] `IntrusiveHashTable<ReferenceIDWrapper,HashTableData<ReferenceIDWrapper,unsigned_long>_*>::optimize_size` · 005572a0 · legions 0052fc40 — Rust std replaces it

## `portal/shared/src/interp/glue/gplacestable.cpp.c` (0/1)

### HashTable<PStringBase<unsigned_short>,Position>
- [-] `HashTable<PStringBase<unsigned_short>,Position>::add` · 0053d610 · legions 0051f3f0 — Rust std replaces it

## `portal/shared/src/interp/glue/grlist.cpp.c` (2/2)

### List<ItemDisplay_*>
- [x] `List<ItemDisplay_*>::push` · 00554600 · legions 0042dd80 — ac2-script glue::lists

### List<unsigned_long>
- [x] `List<unsigned_long>::pop_back` · 00554ae0 · legions 0052eac0 — ac2-script glue::lists

## `portal/shared/src/interp/glue/gwpstring.cpp.c` (0/1)

### PStringBase<unsigned_short>
- [-] `PStringBase<unsigned_short>::append_double` · 00552cd0 · legions 0052d8a0 — Rust std replaces it

## `portal/shared/src/interp/perf/vmmon.cpp.c` (0/1)

### SmartArray<PStringBase<char>_>
- [-] `SmartArray<PStringBase<char>_>::grow` · 004bca90 · legions 004e5ca0 — Rust std replaces it

## `portal/shared/src/interp/processstate.cpp.c` (0/1)

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,ContextState_*>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,ContextState_*>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,ContextState_*>_*>` · 006ddc60 · legions 00666e10 — Rust std replaces it

## `portal/shared/src/interp/vm.cpp.c` (0/1)

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,BreakpointInfo>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,BreakpointInfo>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,BreakpointInfo>_*>` · 006dcd90 · legions 00666190 — Rust std replaces it

## `portal/shared/src/interp/wccefuns.cpp.c` (7/9)

### (free functions)
- [x] `_WCC_strcat_str` · 006e0d30 — ac2-script glue

### PStringBase<unsigned_short>
- [-] `PStringBase<unsigned_short>::append_uint64` · 006e0e00 · legions 00667c70 — Rust std replaces it
- [-] `PStringBase<unsigned_short>::append_uint64_hex` · 006e0e80 · legions 00667cf0 — Rust std replaces it

### (free functions)
- [x] `_WCC_strcat_int` · 006e0f00 · legions 00667d70 — ac2-script glue
- [x] `_WCC_strcat_uint` · 006e0fc0 · legions 00667e30 — ac2-script glue
- [x] `_WCC_strcat_float` · 006e1080 · legions 00667ef0 — ac2-script glue
- [x] `_WCC_strcat_lint` · 006e1140 · legions 00667fb0 — ac2-script glue
- [x] `_WCC_strcat_ulint` · 006e1210 · legions 00668080 — ac2-script glue
- [x] `_WCC_strcat_lfloat` · 006e12e0 · legions 00668150 — ac2-script glue

## `portal/shared/src/mesh/basemesh.cpp.c` (0/7)

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,CMorphTargetData>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,CMorphTargetData>_*>::delete_contents` · 004d1140 · legions 004c52f0 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,CVertexSetData>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,CVertexSetData>_*>::delete_contents` · 004d11d0 · legions 004c5380 — Rust std replaces it

### SmartArray<MeshMaterials::MaterialAndDataID>
- [-] `SmartArray<MeshMaterials::MaterialAndDataID>::grow` · 004d1300 · legions 004c54b0 — Rust std replaces it

### HashTable<unsigned_long,CMorphTargetData>
- [-] `HashTable<unsigned_long,CMorphTargetData>::add` · 004d1bf0 — Rust std replaces it

### HashTable<unsigned_long,CVertexSetData>
- [-] `HashTable<unsigned_long,CVertexSetData>::add` · 004d1c70 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,CMorphTargetData>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,CMorphTargetData>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,CMorphTargetData>_*>` · 004d1da0 · legions 004c5e00 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,CVertexSetData>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,CVertexSetData>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,CVertexSetData>_*>` · 004d1e30 · legions 004c5e90 — Rust std replaces it

## `portal/shared/src/mesh/cellmesh.cpp.c` (0/2)

### SmartArray<CPortalPoly>
- [-] `SmartArray<CPortalPoly>::Reset` · 004cef60 · legions 004c3530 — Rust std replaces it
- [-] `SmartArray<CPortalPoly>::grow` · 004cf250 · legions 004c3710 — Rust std replaces it

## `portal/shared/src/newnet/asyncstate.cpp.c` (0/4)

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,unsigned___int64>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,unsigned___int64>_*>::resize_internal` · 0063a820 · legions 004ec840 — Rust std replaces it

### IntrusiveHashTable<PStringBase<char>,HashTableData<PStringBase<char>,unsigned_long>_*>
- [-] `IntrusiveHashTable<PStringBase<char>,HashTableData<PStringBase<char>,unsigned_long>_*>::resize_internal` · 0063a9f0 · legions 005ef520 — Rust std replaces it

### IntrusiveHashTable<AsyncContext,HashTableData<AsyncContext,CAsyncStateMachine::CAsyncStateData_*>_*>
- [-] `IntrusiveHashTable<AsyncContext,HashTableData<AsyncContext,CAsyncStateMachine::CAsyncStateData_*>_*>::IntrusiveHashTable<AsyncContext,HashTableData<AsyncContext,CAsyncStateMachine::CAsyncStateData_*>_*>` · 0063b000 · legions 005efb80 — Rust std replaces it

### HashTable<PStringBase<char>,unsigned_long>
- [-] `HashTable<PStringBase<char>,unsigned_long>::grow` · 0063b170 · legions 005efcf0 — Rust std replaces it

## `portal/shared/src/newnet/packetcontroller.cpp.c` (2/2)

### PQueueArray<double,void_*>
- [x] `PQueueArray<double,void_*>::_ExpandNodes` · 004ca400 · legions 004be9c0 — pqueue.rs PQueueArray grows as needed
- [x] `PQueueArray<double,void_*>::_Heapify` · 004ca450 · legions 004bea10 — Rust std replaces it

## `portal/shared/src/newnet/sentpacket.cpp.c` (0/1)

### HashTable<unsigned___int64,unsigned___int64>
- [-] `HashTable<unsigned___int64,unsigned___int64>::set` · 004ceaf0 · legions 004c30c0 — Rust std replaces it

## `portal/shared/src/objdesc/appearance/appearancetable.cpp.c` (0/7)

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,VisualModTable>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,VisualModTable>_*>::delete_contents` · 00648e50 · legions 005f8e50 — Rust std replaces it

### List<float>
- [-] `List<float>::insert_sort` · 00648ee0 · legions 005f8ee0 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,VisualModTable>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,VisualModTable>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,VisualModTable>_*>` · 00649710 · legions 005f93a0 — Rust std replaces it

### List<AtomCollection>
- [-] `List<AtomCollection>::push_back` · 00649d20 · legions 005f97b0 — Rust std replaces it
- [-] `List<AtomCollection>::flush` · 00649da0 · legions 005f9830 — Rust std replaces it
- [-] `List<AtomCollection>::List<AtomCollection>` · 00649ff0 · legions 005f9a80 — Rust std replaces it

### HashTable<unsigned_long,VisualModTable>
- [-] `HashTable<unsigned_long,VisualModTable>::add` · 0064a180 — Rust std replaces it

## `portal/shared/src/objdesc/appearance/atomcollection.cpp.c` (0/13)

### List<MaterialModifierAtom>
- [-] `List<MaterialModifierAtom>::find` · 00646700 — Rust std replaces it

### List<OpacityAtom>
- [-] `List<OpacityAtom>::find` · 00646740 — Rust std replaces it

### List<MeshReplacementAtom>
- [-] `List<MeshReplacementAtom>::find` · 00646780 — Rust std replaces it

### List<PGDReplacementAtom>
- [-] `List<PGDReplacementAtom>::find` · 006467c0 — Rust std replaces it

### List<IconModAtom>
- [-] `List<IconModAtom>::find` · 00646800 — Rust std replaces it
- [-] `List<IconModAtom>::remove` · 00646900 · legions 005f7550 — Rust std replaces it
- [-] `List<IconModAtom>::remove` · 00646e20 · legions 005f76f0 — Rust std replaces it

### List<MaterialModifierAtom>
- [-] `List<MaterialModifierAtom>::remove` · 00646ea0 · legions 005f7770 — Rust std replaces it
- [-] `List<MaterialModifierAtom>::remove` · 00646f30 · legions 005f7800 — Rust std replaces it

### _STL::_Rb_tree<FXAtom,FXAtom,_STL::_Identity<FXAtom>,_STL::less<FXAtom>,_STL::allocator<FXAtom>_>
- [-] `_STL::_Rb_tree<FXAtom,FXAtom,_STL::_Identity<FXAtom>,_STL::less<FXAtom>,_STL::allocator<FXAtom>_>::_M_insert` · 00647910 · legions 005f8000 — Rust std replaces it
- [-] `_STL::_Rb_tree<FXAtom,FXAtom,_STL::_Identity<FXAtom>,_STL::less<FXAtom>,_STL::allocator<FXAtom>_>::_M_copy` · 006479d0 · legions 005f80c0 — Rust std replaces it
- [-] `_STL::_Rb_tree<FXAtom,FXAtom,_STL::_Identity<FXAtom>,_STL::less<FXAtom>,_STL::allocator<FXAtom>_>::insert_unique` · 00647c20 · legions 005f8310 — Rust std replaces it
- [-] `_STL::_Rb_tree<FXAtom,FXAtom,_STL::_Identity<FXAtom>,_STL::less<FXAtom>,_STL::allocator<FXAtom>_>::insert_unique` · 00647ce0 · legions 005f83d0 — Rust std replaces it

## `portal/shared/src/objdesc/appearance/partgroupdatadesc.cpp.c` (0/5)

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,BaseList<unsigned_long,11>_>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,BaseList<unsigned_long,11>_>_*>::get_bucket_size_pointer` · 00644a00 · legions 0063f720 — Rust std replaces it

### HashTable<unsigned_long,AppearanceInfo>
- [-] `HashTable<unsigned_long,AppearanceInfo>::add` · 00645430 · legions 005f69d0 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PSmartArray<FXData>_>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PSmartArray<FXData>_>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PSmartArray<FXData>_>_*>` · 006454f0 · legions 005f6a90 — Rust std replaces it

### HashTableData<unsigned_long,PSmartArray<FXData>_>
- [-] `HashTableData<unsigned_long,PSmartArray<FXData>_>::HashTableData<unsigned_long,PSmartArray<FXData>_>` · 006460d0 · legions 005f6ea0 — Rust std replaces it

### HashTable<unsigned_long,PSmartArray<FXData>_>
- [-] `HashTable<unsigned_long,PSmartArray<FXData>_>::add` · 006461e0 · legions 005f6fb0 — Rust std replaces it

## `portal/shared/src/objdesc/appearance/visualmodtable.cpp.c` (0/1)

### List<AtomCollection>
- [-] `List<AtomCollection>::insert` · 0064ad20 — Rust std replaces it

## `portal/shared/src/objdesc/icon/icondesc.cpp.c` (0/8)

### List<IconLayerDesc>
- [-] `List<IconLayerDesc>::find` · 0063fbb0 · legions 005f3ac0 — Rust std replaces it
- [-] `List<IconLayerDesc>::pop_front` · 0063fd00 · legions 005f3c10 — Rust std replaces it
- [-] `List<IconLayerDesc>::remove` · 0063fd80 — Rust std replaces it
- [-] `List<IconLayerDesc>::insert` · 0063fdf0 · legions 005f3c90 — Rust std replaces it
- [-] `List<IconLayerDesc>::push_back` · 0063ff00 · legions 005f3d40 — Rust std replaces it
- [-] `List<IconLayerDesc>::remove` · 0063ff90 — Rust std replaces it
- [-] `List<IconLayerDesc>::remove` · 00640030 — Rust std replaces it
- [-] `List<IconLayerDesc>::flush` · 006400a0 · legions 005f3dd0 — Rust std replaces it

## `portal/shared/src/objdesc/physicsdesc.cpp.c` (0/4)

### _STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,SliderData>,_STL::_Select1st<_STL::pair<unsigned_long_const_,SliderData>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,SliderData>_>_>
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,SliderData>,_STL::_Select1st<_STL::pair<unsigned_long_const_,SliderData>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,SliderData>_>_>::_M_insert` · 00640bc0 · legions 005f43c0 — Rust std replaces it
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,SliderData>,_STL::_Select1st<_STL::pair<unsigned_long_const_,SliderData>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,SliderData>_>_>::insert_unique` · 00640ca0 · legions 005f44a0 — Rust std replaces it
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,SliderData>,_STL::_Select1st<_STL::pair<unsigned_long_const_,SliderData>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,SliderData>_>_>::insert_unique` · 00640d60 · legions 005f4560 — Rust std replaces it

### SmartArray<BehaviorParams>
- [-] `SmartArray<BehaviorParams>::add` · 00641c20 · legions 005f5420 — Rust std replaces it

## `portal/shared/src/objdesc/visualdesc.cpp.c` (0/10)

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,unsigned_long>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,unsigned_long>_*>::begin` · 0063ca10 · legions 0056e5e0 — Rust std replaces it

### List<AppearanceInfo>
- [-] `List<AppearanceInfo>::insert` · 0063ca60 · legions 005f14b0 — Rust std replaces it

### HashTable<unsigned_long,BaseList<unsigned_long,11>_>
- [-] `HashTable<unsigned_long,BaseList<unsigned_long,11>_>::begin` · 0063cb20 · legions 005599c0 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PartGroupDataDesc>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PartGroupDataDesc>_*>::delete_contents` · 0063cba0 · legions 005f1570 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PSmartArray<FXData>_>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PSmartArray<FXData>_>_*>::delete_contents` · 0063cd90 · legions 005f1700 — Rust std replaces it

### HashTable<unsigned_long,PartGroupDataDesc>
- [-] `HashTable<unsigned_long,PartGroupDataDesc>::add` · 0063da10 · legions 005f1fe0 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,FXNode>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,FXNode>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,FXNode>_*>` · 0063dae0 · legions 005f20b0 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PartGroupDataDesc>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PartGroupDataDesc>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PartGroupDataDesc>_*>` · 0063de10 · legions 005f23e0 — Rust std replaces it

### HashTable<unsigned_long,PartGroupDataDesc>
- [-] `HashTable<unsigned_long,PartGroupDataDesc>::resize` · 0063e080 · legions 005f2650 — Rust std replaces it

### SmartArray<FXData>
- [-] `SmartArray<FXData>::grow` · 0063f570 · legions 005f34a0 — Rust std replaces it

## `portal/shared/src/path/bsp/pathbspleaf.cpp.c` (0/2)

### SmartArray<PathNodeEdge>
- [-] `SmartArray<PathNodeEdge>::Reset` · 00635960 · legions 005ecc40 — Rust std replaces it
- [-] `SmartArray<PathNodeEdge>::grow` · 00635a50 · legions 005ecd00 — Rust std replaces it

## `portal/shared/src/path/finder/pathfinder.cpp.c` (0/6)

### List<Ray>
- [-] `List<Ray>::insert` · 006315e0 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,Path>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,Path>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,Path>_*>` · 00631820 · legions 005e9160 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,Marker_*>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,Marker_*>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,Marker_*>_*>` · 006318d0 · legions 005e9210 — Rust std replaces it

### HashTable<unsigned_long,Path>
- [-] `HashTable<unsigned_long,Path>::find` · 00631a10 · legions 005e9350 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,Path>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,Path>_*>::delete_contents` · 00631a60 · legions 005e93a0 — Rust std replaces it

### HashTable<unsigned_long,Path>
- [-] `HashTable<unsigned_long,Path>::set` · 00631b90 — Rust std replaces it

## `portal/shared/src/path/obstacle/obstacle.cpp.c` (0/1)

### SmartArray<PathExternalLink>
- [-] `SmartArray<PathExternalLink>::grow` · 00636a30 · legions 005edce0 — Rust std replaces it

## `portal/shared/src/path/path.cpp.c` (0/1)

### SmartArray<Position>
- [-] `SmartArray<Position>::grow` · 0062b7e0 · legions 005e5700 — Rust std replaces it

## `portal/shared/src/path/pathmap.cpp.c` (0/3)

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PathNode_*>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PathNode_*>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PathNode_*>_*>` · 0062d700 · legions 005e6a60 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,SimplePolygon_*>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,SimplePolygon_*>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,SimplePolygon_*>_*>` · 0062d790 · legions 005e6af0 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PathBSPTree_*>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PathBSPTree_*>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PathBSPTree_*>_*>` · 0062d820 · legions 005e6b80 — Rust std replaces it

## `portal/shared/src/perfmon/perfmoncounterinfo.cpp.c` (0/2)

### List<PerfMonCounterInfo::PerfMonCounterNameHelp>
- [-] `List<PerfMonCounterInfo::PerfMonCounterNameHelp>::insert` · 006f26f0 · legions 006743a0 — Rust std replaces it
- [-] `List<PerfMonCounterInfo::PerfMonCounterNameHelp>::insert_sort` · 006f2770 · legions 00674420 — Rust std replaces it

## `portal/shared/src/perfmon/perfmoncountermanager.cpp.c` (0/1)

### SmartArray<CLandBlock::DynamicObjectInfo>
- [-] `SmartArray<CLandBlock::DynamicObjectInfo>::grow` · 006f19e0 · legions 00483530 — Rust std replaces it

## `portal/shared/src/perfmon/perfmonrelay.cpp.c` (0/2)

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PerfMonVariant_*>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PerfMonVariant_*>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PerfMonVariant_*>_*>` · 006f2f10 · legions 00674b00 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,SmartArray<PMProcess>_>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,SmartArray<PMProcess>_>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,SmartArray<PMProcess>_>_*>` · 006f2fa0 · legions 00674b90 — Rust std replaces it

## `portal/shared/src/perfmon/performer.cpp.c` (0/1)

### SmartArray<PerfMonDescriptor>
- [-] `SmartArray<PerfMonDescriptor>::grow` · 006f3200 · legions 00674d50 — Rust std replaces it

## `portal/shared/src/physobj/childlist.cpp.c` (0/1)

### SmartArray<CHILDINFO>
- [-] `SmartArray<CHILDINFO>::grow` · 004ed9a0 · legions 004e0310 — Rust std replaces it

## `portal/shared/src/physobj/collision/collideproperty.cpp.c` (0/1)

### SmartArray<unsigned_long>
- [-] `SmartArray<unsigned_long>::FastReset` · 0069fee0 · legions 0063cd50 — Rust std replaces it

## `portal/shared/src/physobj/collision/collisionmanager.cpp.c` (0/10)

### IntrusiveHashTable<InstanceID,HashTableData<InstanceID,unsigned_long>_*>
- [-] `IntrusiveHashTable<InstanceID,HashTableData<InstanceID,unsigned_long>_*>::remove` · 004f6c40 · legions 004c7f90 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,unsigned_long>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,unsigned_long>_*>::remove` · 004f6cc0 · legions 004e9430 — Rust std replaces it

### IntrusiveHashTable<InstanceID,HashTableData<InstanceID,ObjCollisionType>_*>
- [-] `IntrusiveHashTable<InstanceID,HashTableData<InstanceID,ObjCollisionType>_*>::delete_contents` · 004f72a0 · legions 004e9ad0 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,CollidePropertyList>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,CollidePropertyList>_*>::delete_contents` · 004f7330 · legions 004e9b60 — Rust std replaces it

### HashTable<InstanceID,ObjCollisionType>
- [-] `HashTable<InstanceID,ObjCollisionType>::add` · 004f80c0 · legions 004ea910 — Rust std replaces it

### IntrusiveHashTable<InstanceID,HashTableData<InstanceID,ObjCollisionType>_*>
- [-] `IntrusiveHashTable<InstanceID,HashTableData<InstanceID,ObjCollisionType>_*>::IntrusiveHashTable<InstanceID,HashTableData<InstanceID,ObjCollisionType>_*>` · 004f8270 · legions 004eaac0 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,CollidePropertyList>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,CollidePropertyList>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,CollidePropertyList>_*>` · 004f8300 · legions 004eab50 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PusherValues>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PusherValues>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PusherValues>_*>` · 004f8390 · legions 004eabe0 — Rust std replaces it

### HashTable<unsigned_long,CollidePropertyList>
- [-] `HashTable<unsigned_long,CollidePropertyList>::add` · 004f92f0 · legions 004ebb40 — Rust std replaces it

### IntrusiveHashTable<unsigned___int64,HashTableData<unsigned___int64,unsigned_long>_*>
- [-] `IntrusiveHashTable<unsigned___int64,HashTableData<unsigned___int64,unsigned_long>_*>::resize_internal` · 004f9c10 · legions 00457970 — Rust std replaces it

## `portal/shared/src/physobj/detect/detectionblocktype.cpp.c` (0/2)

### IntrusiveHashTable<InstanceID,HashTableData<InstanceID,DetectionObjRay>_*>
- [-] `IntrusiveHashTable<InstanceID,HashTableData<InstanceID,DetectionObjRay>_*>::IntrusiveHashTable<InstanceID,HashTableData<InstanceID,DetectionObjRay>_*>` · 006a2f30 · legions 0063fe10 — Rust std replaces it

### HashTable<InstanceID,DetectionObjRay>
- [-] `HashTable<InstanceID,DetectionObjRay>::add` · 006a3020 · legions 0063ff00 — Rust std replaces it

## `portal/shared/src/physobj/detect/detectionmanager.cpp.c` (0/13)

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,DetectionCylsphere>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,DetectionCylsphere>_*>::delete_contents` · 004fab10 · legions 004ed310 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,InstantDetectionState>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,InstantDetectionState>_*>::delete_contents` · 004faba0 · legions 004ed3a0 — Rust std replaces it

### HashTable<unsigned_long,DetectionCylsphere>
- [-] `HashTable<unsigned_long,DetectionCylsphere>::remove` · 004fb760 · legions 004edf60 — Rust std replaces it

### HashTable<InstanceID,unsigned_long>
- [-] `HashTable<InstanceID,unsigned_long>::erase` · 004fb7d0 · legions 004edfd0 — Rust std replaces it

### HashTable<unsigned_long,InstantDetectionState>
- [-] `HashTable<unsigned_long,InstantDetectionState>::add` · 004fc0b0 · legions 004ee8b0 — Rust std replaces it

### HashTable<InstanceID,DetectionInfo>
- [-] `HashTable<InstanceID,DetectionInfo>::add` · 004fc120 · legions 004ee920 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,DetectionCylsphere>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,DetectionCylsphere>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,DetectionCylsphere>_*>` · 004fc260 · legions 004eea60 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,InstantDetectionState>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,InstantDetectionState>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,InstantDetectionState>_*>` · 004fc2f0 · legions 004eeaf0 — Rust std replaces it

### IntrusiveHashTable<InstanceID,HashTableData<InstanceID,DetectionInfo>_*>
- [-] `IntrusiveHashTable<InstanceID,HashTableData<InstanceID,DetectionInfo>_*>::IntrusiveHashTable<InstanceID,HashTableData<InstanceID,DetectionInfo>_*>` · 004fc740 · legions 004eef40 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,unsigned_long>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,unsigned_long>_*>::add_for_unpacking` · 004fc9f0 · legions 0040a260 — Rust std replaces it

### SmartArray<DetectionBlockType>
- [-] `SmartArray<DetectionBlockType>::grow` · 004fd1c0 · legions 004ef9e0 — Rust std replaces it

### PSmartArray<Ray>
- [-] `PSmartArray<Ray>::PSmartArray<Ray>` · 004fd3e0 · legions 004efc00 — Rust std replaces it

### IntrusiveHashTable<InstanceID,HashTableData<InstanceID,DetectionObjRay>_*>
- [-] `IntrusiveHashTable<InstanceID,HashTableData<InstanceID,DetectionObjRay>_*>::delete_contents` · 004fd430 · legions 004efc50 — Rust std replaces it

## `portal/shared/src/physobj/detect/profilelisttype.cpp.c` (0/1)

### IntrusiveHashTable<InstanceID,HashTableData<InstanceID,DetectionProfile>_*>
- [-] `IntrusiveHashTable<InstanceID,HashTableData<InstanceID,DetectionProfile>_*>::IntrusiveHashTable<InstanceID,HashTableData<InstanceID,DetectionProfile>_*>` · 006a29e0 · legions 0063f7c0 — Rust std replaces it

## `portal/shared/src/physobj/fx/fxmanager.cpp.c` (0/11)

### SmartArray<AsyncContext>
- [-] `SmartArray<AsyncContext>::grow` · 00505d20 · legions 004c8020 — Rust std replaces it

### SmartArray<PlayingSoundContext>
- [-] `SmartArray<PlayingSoundContext>::add` · 00506590 · legions 004f47b0 — Rust std replaces it

### _STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,FXState>,_STL::_Select1st<_STL::pair<unsigned_long_const_,FXState>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,FXState>_>_>
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,FXState>,_STL::_Select1st<_STL::pair<unsigned_long_const_,FXState>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,FXState>_>_>::_M_erase` · 00506a70 · legions 004f4c90 — Rust std replaces it
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,FXState>,_STL::_Select1st<_STL::pair<unsigned_long_const_,FXState>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,FXState>_>_>::erase` · 00506cc0 · legions 004f4ee0 — Rust std replaces it
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,FXState>,_STL::_Select1st<_STL::pair<unsigned_long_const_,FXState>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,FXState>_>_>::erase` · 00506d30 · legions 004f4f50 — Rust std replaces it

### _STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,FXScalarAndTarget>,_STL::_Select1st<_STL::pair<unsigned_long_const_,FXScalarAndTarget>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,FXScalarAndTarget>_>_>
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,FXScalarAndTarget>,_STL::_Select1st<_STL::pair<unsigned_long_const_,FXScalarAndTarget>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,FXScalarAndTarget>_>_>::_M_insert` · 00506e60 · legions 004f5080 — Rust std replaces it
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,FXScalarAndTarget>,_STL::_Select1st<_STL::pair<unsigned_long_const_,FXScalarAndTarget>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,FXScalarAndTarget>_>_>::insert_unique` · 00506f50 · legions 004f5170 — Rust std replaces it
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,FXScalarAndTarget>,_STL::_Select1st<_STL::pair<unsigned_long_const_,FXScalarAndTarget>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,FXScalarAndTarget>_>_>::insert_unique` · 005072e0 · legions 004f5500 — Rust std replaces it

### _STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,FXState>,_STL::_Select1st<_STL::pair<unsigned_long_const_,FXState>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,FXState>_>_>
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,FXState>,_STL::_Select1st<_STL::pair<unsigned_long_const_,FXState>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,FXState>_>_>::_M_insert` · 005074c0 · legions 004f56e0 — Rust std replaces it
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,FXState>,_STL::_Select1st<_STL::pair<unsigned_long_const_,FXState>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,FXState>_>_>::insert_unique` · 00507590 · legions 004f57b0 — Rust std replaces it
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,FXState>,_STL::_Select1st<_STL::pair<unsigned_long_const_,FXState>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,FXState>_>_>::insert_unique` · 00507b30 · legions 004f5ca0 — Rust std replaces it

## `portal/shared/src/physobj/fx/psdesc.cpp.c` (0/3)

### SmartArray<PEKeyframeDesc>
- [-] `SmartArray<PEKeyframeDesc>::Reset` · 004fe6f0 · legions 004f0ec0 — Rust std replaces it
- [-] `SmartArray<PEKeyframeDesc>::grow` · 00500940 · legions 004f1d50 — Rust std replaces it

### SmartArray<PEmitterDesc>
- [-] `SmartArray<PEmitterDesc>::grow` · 00502b40 · legions 004f2580 — Rust std replaces it

## `portal/shared/src/physobj/minterp/interpolationmanager.cpp.c` (0/30)

### _STL
- [-] `_STL::__destroy_aux` · 0050cd70 · legions 004fade0 — Rust std replaces it
- [-] `_STL::__copy_backward` · 0050cdc0 · legions 004fae30 — Rust std replaces it
- [-] `_STL::__copy` · 0050ced0 · legions 004faf40 — Rust std replaces it

### _STL::deque<_STL::pair<InterpolationManager::QueuedPosition,float>,_STL::allocator<_STL::pair<InterpolationManager::QueuedPosition,float>_>_>
- [-] `_STL::deque<_STL::pair<InterpolationManager::QueuedPosition,float>,_STL::allocator<_STL::pair<InterpolationManager::QueuedPosition,float>_>_>::back` · 0050ddb0 · legions 004fbe20 — Rust std replaces it

### _STL
- [-] `_STL::fill` · 0050dde0 · legions 004fbe50 — Rust std replaces it
- [-] `_STL::__uninitialized_fill` · 0050de90 · legions 004fbf00 — Rust std replaces it
- [-] `_STL::__copy_backward_aux` · 0050def0 · legions 004fbf60 — Rust std replaces it
- [-] `_STL::__copy_aux` · 0050df70 · legions 004fbfe0 — Rust std replaces it
- [-] `_STL::__uninitialized_copy` · 0050dff0 · legions 004fc060 — Rust std replaces it

### _STL::deque<_STL::pair<InterpolationManager::QueuedPosition,float>,_STL::allocator<_STL::pair<InterpolationManager::QueuedPosition,float>_>_>
- [-] `_STL::deque<_STL::pair<InterpolationManager::QueuedPosition,float>,_STL::allocator<_STL::pair<InterpolationManager::QueuedPosition,float>_>_>::at` · 0050e470 · legions 004fc4e0 — Rust std replaces it

### _STL
- [-] `_STL::_Destroy` · 0050e4f0 · legions 004fc560 — Rust std replaces it
- [-] `_STL::uninitialized_fill` · 0050e540 · legions 004fc5b0 — Rust std replaces it
- [-] `_STL::copy_backward` · 0050e5a0 · legions 004fc610 — Rust std replaces it
- [-] `_STL::copy` · 0050e620 · legions 004fc690 — Rust std replaces it
- [-] `_STL::uninitialized_copy` · 0050e6a0 · legions 004fc710 — Rust std replaces it
- [-] `_STL::__uninitialized_copy_fill` · 0050e720 · legions 004fc790 — Rust std replaces it
- [-] `_STL::__uninitialized_fill_copy` · 0050e7f0 · legions 004fc860 — Rust std replaces it

### _STL::_Deque_base<_STL::pair<InterpolationManager::QueuedPosition,float>,_STL::allocator<_STL::pair<InterpolationManager::QueuedPosition,float>_>_>
- [-] `_STL::_Deque_base<_STL::pair<InterpolationManager::QueuedPosition,float>,_STL::allocator<_STL::pair<InterpolationManager::QueuedPosition,float>_>_>::_M_initialize_map` · 0050eab0 · legions 004fcb20 — Rust std replaces it

### _STL::deque<_STL::pair<InterpolationManager::QueuedPosition,float>,_STL::allocator<_STL::pair<InterpolationManager::QueuedPosition,float>_>_>
- [-] `_STL::deque<_STL::pair<InterpolationManager::QueuedPosition,float>,_STL::allocator<_STL::pair<InterpolationManager::QueuedPosition,float>_>_>::clear` · 0050eb50 · legions 004fcbc0 — Rust std replaces it
- [-] `_STL::deque<_STL::pair<InterpolationManager::QueuedPosition,float>,_STL::allocator<_STL::pair<InterpolationManager::QueuedPosition,float>_>_>::_M_new_elements_at_front` · 0050ec60 · legions 004fccd0 — Rust std replaces it
- [-] `_STL::deque<_STL::pair<InterpolationManager::QueuedPosition,float>,_STL::allocator<_STL::pair<InterpolationManager::QueuedPosition,float>_>_>::_M_new_elements_at_back` · 0050ece0 · legions 004fcd50 — Rust std replaces it
- [-] `_STL::deque<_STL::pair<InterpolationManager::QueuedPosition,float>,_STL::allocator<_STL::pair<InterpolationManager::QueuedPosition,float>_>_>::erase` · 0050ee10 · legions 004fce80 — Rust std replaces it
- [-] `_STL::deque<_STL::pair<InterpolationManager::QueuedPosition,float>,_STL::allocator<_STL::pair<InterpolationManager::QueuedPosition,float>_>_>::_M_push_back_aux_v` · 0050f0b0 · legions 004fd120 — Rust std replaces it
- [-] `_STL::deque<_STL::pair<InterpolationManager::QueuedPosition,float>,_STL::allocator<_STL::pair<InterpolationManager::QueuedPosition,float>_>_>::_M_reserve_elements_at_front` · 0050f150 · legions 004fd1c0 — Rust std replaces it
- [-] `_STL::deque<_STL::pair<InterpolationManager::QueuedPosition,float>,_STL::allocator<_STL::pair<InterpolationManager::QueuedPosition,float>_>_>::_M_reserve_elements_at_back` · 0050f1c0 · legions 004fd230 — Rust std replaces it
- [-] `_STL::deque<_STL::pair<InterpolationManager::QueuedPosition,float>,_STL::allocator<_STL::pair<InterpolationManager::QueuedPosition,float>_>_>::_M_insert_aux` · 0050f230 · legions 004fd2a0 — Rust std replaces it
- [-] `_STL::deque<_STL::pair<InterpolationManager::QueuedPosition,float>,_STL::allocator<_STL::pair<InterpolationManager::QueuedPosition,float>_>_>::push_back` · 005101f0 · legions 004fe260 — Rust std replaces it
- [-] `_STL::deque<_STL::pair<InterpolationManager::QueuedPosition,float>,_STL::allocator<_STL::pair<InterpolationManager::QueuedPosition,float>_>_>::_M_fill_insert` · 00510240 · legions 004fe2b0 — Rust std replaces it
- [-] `_STL::deque<_STL::pair<InterpolationManager::QueuedPosition,float>,_STL::allocator<_STL::pair<InterpolationManager::QueuedPosition,float>_>_>::resize` · 00510be0 · legions 004febf0 — Rust std replaces it
- [-] `_STL::deque<_STL::pair<InterpolationManager::QueuedPosition,float>,_STL::allocator<_STL::pair<InterpolationManager::QueuedPosition,float>_>_>::resize` · 00510c90 · legions 004feca0 — Rust std replaces it

## `portal/shared/src/physobj/minterp/motioninterpdesc.cpp.c` (0/9)

### IntrusiveHashTable<ModeStateKey,HashTableData<ModeStateKey,int>_*>
- [-] `IntrusiveHashTable<ModeStateKey,HashTableData<ModeStateKey,int>_*>::find_internal` · 0069dcc0 · legions 0063bbd0 — Rust std replaces it
- [-] `IntrusiveHashTable<ModeStateKey,HashTableData<ModeStateKey,int>_*>::delete_contents` · 0069de10 · legions 004e5df0 — Rust std replaces it

### HashTable<unsigned_long,MotionValues>
- [-] `HashTable<unsigned_long,MotionValues>::add` · 0069e020 — Rust std replaces it

### HashTable<ModeStateKey,int>
- [-] `HashTable<ModeStateKey,int>::set` · 0069e090 · legions 00625f20 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,TerrainList>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,TerrainList>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,TerrainList>_*>` · 0069e120 · legions 0063bda0 — Rust std replaces it

### IntrusiveHashTable<ModeStateKey,HashTableData<ModeStateKey,unsigned_long>_*>
- [-] `IntrusiveHashTable<ModeStateKey,HashTableData<ModeStateKey,unsigned_long>_*>::IntrusiveHashTable<ModeStateKey,HashTableData<ModeStateKey,unsigned_long>_*>` · 0069e1b0 · legions 0063be30 — Rust std replaces it

### IntrusiveHashTable<ModeStateKey,HashTableData<ModeStateKey,int>_*>
- [-] `IntrusiveHashTable<ModeStateKey,HashTableData<ModeStateKey,int>_*>::add_for_unpacking` · 0069e6e0 · legions 00627c00 — Rust std replaces it

### HashTable<unsigned_long,TerrainList>
- [-] `HashTable<unsigned_long,TerrainList>::add` · 0069ea00 — Rust std replaces it

### SmartArray<MSList>
- [-] `SmartArray<MSList>::grow` · 0069ef10 · legions 0063c5a0 — Rust std replaces it

## `portal/shared/src/physobj/partarray.cpp.c` (0/4)

### SmartArray<SkeletonPoint>
- [-] `SmartArray<SkeletonPoint>::grow` · 004ef660 · legions 004e1fa0 — Rust std replaces it

### SmartArray<PSystemInstance>
- [-] `SmartArray<PSystemInstance>::grow` · 004ef740 · legions 004e2080 — Rust std replaces it

### SmartArray<LightSourceInstance>
- [-] `SmartArray<LightSourceInstance>::grow` · 004ef7f0 · legions 004e2130 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,AnimationAndState>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,AnimationAndState>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,AnimationAndState>_*>` · 004f0870 · legions 004e31a0 — Rust std replaces it

## `portal/shared/src/physobj/path/progresschecker.cpp.c` (0/4)

### _STL::deque<ProgressData,_STL::allocator<ProgressData>_>
- [-] `_STL::deque<ProgressData,_STL::allocator<ProgressData>_>::size` · 006ac4c0 · legions 00645470 — Rust std replaces it

### _STL
- [-] `_STL::__destroy_aux` · 006ac510 · legions 006454c0 — Rust std replaces it

### _STL::deque<ProgressData,_STL::allocator<ProgressData>_>
- [-] `_STL::deque<ProgressData,_STL::allocator<ProgressData>_>::clear` · 006ac550 · legions 00645710 — Rust std replaces it
- [-] `_STL::deque<ProgressData,_STL::allocator<ProgressData>_>::_M_push_back_aux_v` · 006ac690 · legions 00645900 — Rust std replaces it

## `portal/shared/src/physobj/physicsobj.cpp.c` (0/6)

### SmartArray<CShadowObj>
- [-] `SmartArray<CShadowObj>::grow` · 004e52b0 · legions 004d7d00 — Rust std replaces it

### List<ItemDisplay_*>
- [-] `List<ItemDisplay_*>::get_node` · 004e5380 · legions 004d7dd0 — Rust std replaces it

### IntrusiveHashTable<InstanceID,HashTableData<InstanceID,DetectionObjRay>_*>
- [-] `IntrusiveHashTable<InstanceID,HashTableData<InstanceID,DetectionObjRay>_*>::find` · 004e8d60 · legions 004db740 — Rust std replaces it

### _STL::deque<NetBlob_*,_STL::allocator<NetBlob_*>_>
- [-] `_STL::deque<NetBlob_*,_STL::allocator<NetBlob_*>_>::_M_reallocate_map` · 004e9340 · legions 005473b0 — Rust std replaces it

### List<AppearanceInfo>
- [-] `List<AppearanceInfo>::flush` · 004e94a0 · legions 004dbd20 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,Trigger>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,Trigger>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,Trigger>_*>` · 004ea1d0 · legions 004dca50 — Rust std replaces it

## `portal/shared/src/physobj/physicsobjrep.cpp.c` (0/5)

### SmartArray<ModeStateKey>
- [-] `SmartArray<ModeStateKey>::grow` · 004f2b40 — Rust std replaces it

### IntrusiveHashTable<ModeStateKey,HashTableData<ModeStateKey,int>_*>
- [-] `IntrusiveHashTable<ModeStateKey,HashTableData<ModeStateKey,int>_*>::add` · 004f3590 · legions 004e5e80 — Rust std replaces it
- [-] `IntrusiveHashTable<ModeStateKey,HashTableData<ModeStateKey,int>_*>::IntrusiveHashTable<ModeStateKey,HashTableData<ModeStateKey,int>_*>` · 004f3970 · legions 004e61b0 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PHYSICSOBJREP_*>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PHYSICSOBJREP_*>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PHYSICSOBJREP_*>_*>` · 004f3bd0 · legions 004e6410 — Rust std replaces it

### IntrusiveHashTable<ModeStateKey,HashTableData<ModeStateKey,int>_*>
- [-] `IntrusiveHashTable<ModeStateKey,HashTableData<ModeStateKey,int>_*>::resize_internal` · 004f3e40 · legions 004e6680 — Rust std replaces it

## `portal/shared/src/physobj/story/physicsstory.cpp.c` (0/6)

### _STL::_Rb_tree<StoryHookData,StoryHookData,_STL::_Identity<StoryHookData>,_STL::less<StoryHookData>,_STL::allocator<StoryHookData>_>
- [-] `_STL::_Rb_tree<StoryHookData,StoryHookData,_STL::_Identity<StoryHookData>,_STL::less<StoryHookData>,_STL::allocator<StoryHookData>_>::_M_erase` · 006adbe0 · legions 00646e70 — Rust std replaces it
- [-] `_STL::_Rb_tree<StoryHookData,StoryHookData,_STL::_Identity<StoryHookData>,_STL::less<StoryHookData>,_STL::allocator<StoryHookData>_>::_M_insert` · 006adc40 · legions 00646ed0 — Rust std replaces it
- [-] `_STL::_Rb_tree<StoryHookData,StoryHookData,_STL::_Identity<StoryHookData>,_STL::less<StoryHookData>,_STL::allocator<StoryHookData>_>::_M_copy` · 006add00 · legions 00646f90 — Rust std replaces it
- [-] `_STL::_Rb_tree<StoryHookData,StoryHookData,_STL::_Identity<StoryHookData>,_STL::less<StoryHookData>,_STL::allocator<StoryHookData>_>::insert_equal` · 006adef0 · legions 00647180 — Rust std replaces it
- [-] `_STL::_Rb_tree<StoryHookData,StoryHookData,_STL::_Identity<StoryHookData>,_STL::less<StoryHookData>,_STL::allocator<StoryHookData>_>::insert_equal` · 006adf30 · legions 006471c0 — Rust std replaces it

### _STL::multiset<StoryHookData,_STL::less<StoryHookData>,_STL::allocator<StoryHookData>_>
- [-] `_STL::multiset<StoryHookData,_STL::less<StoryHookData>,_STL::allocator<StoryHookData>_>::insert` · 006ae0f0 · legions 00647380 — Rust std replaces it

## `portal/shared/src/physobj/target/targetmanager.cpp.c` (0/3)

### IntrusiveHashTable<InstanceID,HashTableData<InstanceID,TargetVoyeur>_*>
- [-] `IntrusiveHashTable<InstanceID,HashTableData<InstanceID,TargetVoyeur>_*>::delete_contents` · 00516150 · legions 00504160 — Rust std replaces it

### HashTable<InstanceID,TargetVoyeur>
- [-] `HashTable<InstanceID,TargetVoyeur>::add` · 00516420 · legions 00504430 — Rust std replaces it

### IntrusiveHashTable<InstanceID,HashTableData<InstanceID,TargetVoyeur>_*>
- [-] `IntrusiveHashTable<InstanceID,HashTableData<InstanceID,TargetVoyeur>_*>::IntrusiveHashTable<InstanceID,HashTableData<InstanceID,TargetVoyeur>_*>` · 005164f0 · legions 00504500 — Rust std replaces it

## `portal/shared/src/places/placestable.cpp.c` (0/16)

### _STL
- [-] `_STL::swap` · 00638f70 — Rust std replaces it
- [-] `_STL::__unguarded_linear_insert` · 00639010 — Rust std replaces it
- [-] `_STL::__push_heap` · 006390d0 — Rust std replaces it
- [-] `_STL::__copy_backward` · 006391b0 — Rust std replaces it
- [-] `_STL::__median` · 00639220 — Rust std replaces it
- [-] `_STL::__unguarded_insertion_sort_aux` · 006392c0 — Rust std replaces it
- [-] `_STL::__adjust_heap` · 00639310 — Rust std replaces it
- [-] `_STL::__unguarded_partition` · 00639410 — Rust std replaces it
- [-] `_STL::__pop_heap` · 006394c0 — Rust std replaces it
- [-] `_STL::__make_heap` · 00639550 — Rust std replaces it
- [-] `_STL::__linear_insert` · 00639740 — Rust std replaces it
- [-] `_STL::sort_heap` · 006397f0 — Rust std replaces it

### HashTable<PStringBase<unsigned_short>,Position>
- [-] `HashTable<PStringBase<unsigned_short>,Position>::set` · 00639a40 — Rust std replaces it

### _STL
- [-] `_STL::__insertion_sort` · 00639ad0 — Rust std replaces it
- [-] `_STL::__partial_sort` · 00639b20 — Rust std replaces it
- [-] `_STL::__introsort_loop` · 00639ed0 — Rust std replaces it

## `portal/shared/src/prefs/preferencebank.cpp.c` (0/1)

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,IPreferenceMonitor_*>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,IPreferenceMonitor_*>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,IPreferenceMonitor_*>_*>` · 006b0770 · legions 006499c0 — Rust std replaces it

## `portal/shared/src/prefs/preferenceenum.cpp.c` (0/3)

### HashTable<long,StringInfo>
- [-] `HashTable<long,StringInfo>::find` · 006b0ce0 · legions 00649f30 — Rust std replaces it

### HashTable<unsigned_long,StringInfo>
- [-] `HashTable<unsigned_long,StringInfo>::set` · 006b0e40 · legions 0064a090 — Rust std replaces it

### IntrusiveHashTable<long,HashTableData<long,unsigned_long>_*>
- [-] `IntrusiveHashTable<long,HashTableData<long,unsigned_long>_*>::IntrusiveHashTable<long,HashTableData<long,unsigned_long>_*>` · 006b1540 · legions 0064a790 — Rust std replaces it

## `portal/shared/src/prefs/preferences.cpp.c` (0/6)

### HashTable<unsigned_long,PStringBase<unsigned_short>_>
- [-] `HashTable<unsigned_long,PStringBase<unsigned_short>_>::find` · 006ae7d0 · legions 00647a60 — Rust std replaces it

### PStringBase<char>
- [-] `PStringBase<char>::append_int32` · 006ae850 · legions 00647ae0 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,HashTable<unsigned_long,PStringBase<char>_>_>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,HashTable<unsigned_long,PStringBase<char>_>_>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,HashTable<unsigned_long,PStringBase<char>_>_>_*>` · 006aea20 · legions 00647cb0 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,IPreferenceHandler_*>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,IPreferenceHandler_*>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,IPreferenceHandler_*>_*>` · 006aeab0 · legions 00647d40 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,HashTable<unsigned_long,PStringBase<char>_>_>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,HashTable<unsigned_long,PStringBase<char>_>_>_*>::delete_contents` · 006aeb40 · legions 00647dd0 — Rust std replaces it

### HashTable<unsigned_long,HashTable<unsigned_long,PStringBase<char>_>_>
- [-] `HashTable<unsigned_long,HashTable<unsigned_long,PStringBase<char>_>_>::add` · 006aed10 · legions 00647fa0 — Rust std replaces it

## `portal/shared/src/property/masterproperty.cpp.c` (0/3)

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,BasePropertyDesc>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,BasePropertyDesc>_*>::delete_contents` · 006b58c0 · legions 0064dfb0 — Rust std replaces it

### HashTable<unsigned_long,BasePropertyDesc>
- [-] `HashTable<unsigned_long,BasePropertyDesc>::add` · 006b5e40 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,BasePropertyDesc>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,BasePropertyDesc>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,BasePropertyDesc>_*>` · 006b5eb0 · legions 0064e660 — Rust std replaces it

## `portal/shared/src/region/encounter/encounterdesc.cpp.c` (0/3)

### SmartArray<EncounterExcept>
- [-] `SmartArray<EncounterExcept>::grow` · 00520590 · legions 0050b700 — Rust std replaces it

### SmartArray<EncounterElem>
- [-] `SmartArray<EncounterElem>::grow` · 00520a60 · legions 0050b930 — Rust std replaces it

### SmartArray<EncounterRow>
- [-] `SmartArray<EncounterRow>::grow` · 00520c10 · legions 0050bae0 — Rust std replaces it

## `portal/shared/src/region/properties/propertydesc.cpp.c` (0/1)

### SmartArray<BlockProperty>
- [-] `SmartArray<BlockProperty>::grow` · 006ced70 · legions 00658bf0 — Rust std replaces it

## `portal/shared/src/region/regiondesc.cpp.c` (0/1)

### SmartArray<DIDLookup>
- [-] `SmartArray<DIDLookup>::grow` · 0051d860 · legions 0050a210 — Rust std replaces it

## `portal/shared/src/region/sky/daydesc.cpp.c` (0/6)

### SmartArray<CelestialPosition>
- [-] `SmartArray<CelestialPosition>::Reset` · 00525a30 · legions 0050e9e0 — Rust std replaces it

### SmartArray<SkyTimeOfDay>
- [-] `SmartArray<SkyTimeOfDay>::grow` · 00526110 · legions 0050efe0 — Rust std replaces it

### SmartArray<LensFlarePoint>
- [-] `SmartArray<LensFlarePoint>::grow` · 005261f0 · legions 0050f0c0 — Rust std replaces it

### SmartArray<SkyObjectReplace>
- [-] `SmartArray<SkyObjectReplace>::grow` · 005264b0 · legions 0050f380 — Rust std replaces it

### SmartArray<SkyObject>
- [-] `SmartArray<SkyObject>::grow` · 00526ac0 · legions 0050f640 — Rust std replaces it

### SmartArray<CelestialPosition>
- [-] `SmartArray<CelestialPosition>::grow` · 00526ec0 · legions 0050f7c0 — Rust std replaces it

## `portal/shared/src/region/sky/skydesc.cpp.c` (0/1)

### SmartArray<SkyDay>
- [-] `SmartArray<SkyDay>::grow` · 00524510 · legions 0050dfb0 — Rust std replaces it

## `portal/shared/src/region/surface/surfacedesc.cpp.c` (0/5)

### SmartArray<LScapeVertexColorDesc>
- [-] `SmartArray<LScapeVertexColorDesc>::grow` · 00522810 · legions 0050ccb0 — Rust std replaces it

### SmartArray<LScapeMaterialDesc>
- [-] `SmartArray<LScapeMaterialDesc>::grow` · 005228f0 · legions 0050cd90 — Rust std replaces it

### SmartArray<LScapeAlphaDesc>
- [-] `SmartArray<LScapeAlphaDesc>::grow` · 005229e0 · legions 0050ce80 — Rust std replaces it

### SmartArray<MaterialGroup>
- [-] `SmartArray<MaterialGroup>::grow` · 00522db0 · legions 0050d250 — Rust std replaces it

### SmartArray<LScapeSurfaceType>
- [-] `SmartArray<LScapeSurfaceType>::grow` · 00522f40 · legions 0050d3e0 — Rust std replaces it

## `portal/shared/src/region/water/watertype.cpp.c` (0/2)

### SmartArray<WaterLightInfo>
- [-] `SmartArray<WaterLightInfo>::Reset` · 006cc0e0 · legions 00657a70 — Rust std replaces it
- [-] `SmartArray<WaterLightInfo>::grow` · 006cc390 · legions 00657d20 — Rust std replaces it

## `portal/shared/src/shell/shell.cpp.c` (0/2)

### List<PStringBase<unsigned_short>_>
- [-] `List<PStringBase<unsigned_short>_>::insert` · 00732f20 · legions 006af3c0 — Rust std replaces it
- [-] `List<PStringBase<unsigned_short>_>::remove` · 00733030 · legions 006af4d0 — Rust std replaces it

## `portal/shared/src/shell/shellcmdtable.cpp.c` (0/6)

### IntrusiveHashTable<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>_*>
- [-] `IntrusiveHashTable<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>_*>::add_internal` · 00612fc0 · legions 00524bc0 — Rust std replaces it

### HashTable<PStringBase<unsigned_short>,CmdHashData>
- [-] `HashTable<PStringBase<unsigned_short>,CmdHashData>::find` · 006132c0 · legions 005d1330 — Rust std replaces it
- [-] `HashTable<PStringBase<unsigned_short>,CmdHashData>::add` · 00613940 · legions 005d19b0 — Rust std replaces it

### HashTable<PStringBase<unsigned_short>,FunctionID>
- [-] `HashTable<PStringBase<unsigned_short>,FunctionID>::add` · 006139e0 · legions 005d1a50 — Rust std replaces it

### IntrusiveHashTable<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,CmdHashData>_*>
- [-] `IntrusiveHashTable<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,CmdHashData>_*>::IntrusiveHashTable<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,CmdHashData>_*>` · 00613b20 · legions 005d1b90 — Rust std replaces it

### IntrusiveHashTable<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,HelpHashData>_*>
- [-] `IntrusiveHashTable<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,HelpHashData>_*>::IntrusiveHashTable<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,HelpHashData>_*>` · 00613bb0 · legions 005d1c20 — Rust std replaces it

## `portal/shared/src/stringtable/metalanguage/metalanguage.cpp.c` (0/1)

### SmartArray<PStringBase<unsigned_short>_>
- [-] `SmartArray<PStringBase<unsigned_short>_>::WhatNum` · 006575a0 · legions 00605ef0 — Rust std replaces it

## `portal/shared/src/stringtable/metalanguage/metalanguage_tokenizer.cpp.c` (0/1)

### (free functions)
- [-] `_stringmeta_lex` · 00659d80 · legions 006086f0 — Rust std replaces it

## `portal/shared/src/stringtable/metalanguage/metalanguage_utils.cpp.c` (0/1)

### PStringBaseIter<unsigned_short>
- [-] `PStringBaseIter<unsigned_short>::Behead` · 00656df0 · legions 00605740 — Rust std replaces it

## `portal/shared/src/stringtable/stringinfo.cpp.c` (0/1)

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,StringInfoData_*>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,StringInfoData_*>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,StringInfoData_*>_*>` · 00652220 · legions 006012f0 — Rust std replaces it

## `portal/shared/src/stringtable/stringtable.cpp.c` (0/1)

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,StringTableString_*>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,StringTableString_*>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,StringTableString_*>_*>` · 006550e0 · legions 006041b0 — Rust std replaces it

## `portal/shared/src/taboo/tabootable.cpp.c` (0/2)

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PList<PStringBase<unsigned_short>_>_>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PList<PStringBase<unsigned_short>_>_>_*>::IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PList<PStringBase<unsigned_short>_>_>_*>` · 0065a890 · legions 00609200 — Rust std replaces it
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PList<PStringBase<unsigned_short>_>_>_*>::delete_contents` · 0065ac60 · legions 006095d0 — Rust std replaces it

## `portal/shared/src/utils/argparser.cpp.c` (0/8)

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,unsigned_long>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,unsigned_long>_*>::delete_contents` · 0065bae0 · legions 00430a00 — Rust std replaces it

### SmartArray<CommandLineArg>
- [-] `SmartArray<CommandLineArg>::grow` · 0065bb60 · legions 0060a490 — Rust std replaces it

### IntrusiveHashTable<char,HashTableData<char,long>_*>
- [-] `IntrusiveHashTable<char,HashTableData<char,long>_*>::add` · 0065bc80 · legions 0060a5b0 — Rust std replaces it

### HashTable<char,long>
- [-] `HashTable<char,long>::add` · 0065c260 · legions 0060ab90 — Rust std replaces it

### SmartArray<CommandLineArg>
- [-] `SmartArray<CommandLineArg>::add` · 0065c2c0 · legions 0060abf0 — Rust std replaces it

### HashTable<PStringBase<char>,unsigned_long>
- [-] `HashTable<PStringBase<char>,unsigned_long>::add` · 0065c310 · legions 0060ac40 — Rust std replaces it

### IntrusiveHashTable<PStringBase<char>,HashTableData<PStringBase<char>,long>_*>
- [-] `IntrusiveHashTable<PStringBase<char>,HashTableData<PStringBase<char>,long>_*>::IntrusiveHashTable<PStringBase<char>,HashTableData<PStringBase<char>,long>_*>` · 0065c400 · legions 0060ad30 — Rust std replaces it

### IntrusiveHashTable<char,HashTableData<char,long>_*>
- [-] `IntrusiveHashTable<char,HashTableData<char,long>_*>::IntrusiveHashTable<char,HashTableData<char,long>_*>` · 0065c490 · legions 0060adc0 — Rust std replaces it

## `portal/shared/src/utils/psutils.cpp.c` (0/10)

### PStringBaseIter_Const<char>
- [-] `PStringBaseIter_Const<char>::IndexToChar` · 0065cd50 · legions 0060b6a0 — Rust std replaces it
- [-] `PStringBaseIter_Const<char>::IndexToConstPointer` · 0065cd70 — Rust std replaces it

### PStringBaseIter_Common<char>
- [-] `PStringBaseIter_Common<char>::Next` · 0065cd90 · legions 0060b6c0 — Rust std replaces it

### List<PStringBase<unsigned_short>_>
- [-] `List<PStringBase<unsigned_short>_>::push_back` · 0065d230 · legions 0060bb60 — Rust std replaces it

### PStringBaseIter<char>
- [-] `PStringBaseIter<char>::Truncate` · 0065d390 · legions 0060bcc0 — Rust std replaces it

### PStringBaseIter_Const<char>
- [-] `PStringBaseIter_Const<char>::Split` · 0065db40 · legions 0060c470 — Rust std replaces it
- [-] `PStringBaseIter_Const<char>::Split` · 0065dcf0 · legions 0041f050 — Rust std replaces it

### PStringBaseIter<char>
- [-] `PStringBaseIter<char>::Behead` · 0065ddd0 · legions 0060c700 — Rust std replaces it
- [-] `PStringBaseIter<char>::Insert` · 0065de70 · legions 0060c7a0 — Rust std replaces it
- [-] `PStringBaseIter<char>::Replace` · 0065df70 · legions 0060c8a0 — Rust std replaces it

## `portal/shared/src/weenie/allocator/wallocator.cpp.c` (0/9)

### List<SourceLineInfo>
- [-] `List<SourceLineInfo>::push_back` · 0052d470 — Rust std replaces it

### List<FunctionLocationInfo>
- [-] `List<FunctionLocationInfo>::push_back` · 0052d520 — Rust std replaces it

### HashTable<unsigned_long,OriginalSourceFileInfo>
- [-] `HashTable<unsigned_long,OriginalSourceFileInfo>::add` · 0052d5f0 · legions 00511a70 — Rust std replaces it

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,OriginalSourceFileInfo>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,OriginalSourceFileInfo>_*>::find` · 0052d670 · legions 00511af0 — Rust std replaces it
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,OriginalSourceFileInfo>_*>::find_internal` · 0052d6b0 · legions 00511b30 — Rust std replaces it

### HashTable<PStringBase<char>,FrameDebugInfo>
- [-] `HashTable<PStringBase<char>,FrameDebugInfo>::add` · 0052d740 · legions 00511b90 — Rust std replaces it

### IntrusiveHashTable<PStringBase<char>,HashTableData<PStringBase<char>,FrameDebugInfo>_*>
- [-] `IntrusiveHashTable<PStringBase<char>,HashTableData<PStringBase<char>,FrameDebugInfo>_*>::find` · 0052d7c0 · legions 00511c10 — Rust std replaces it

### SmartArray<FrameMemberDebugInfo>
- [-] `SmartArray<FrameMemberDebugInfo>::grow` · 0052d990 · legions 00511e30 — Rust std replaces it

### HashTable<PStringBase<char>,unsigned_long>
- [-] `HashTable<PStringBase<char>,unsigned_long>::end` · 0052dd10 · legions 005121b0 — Rust std replaces it

## `portal/shared/src/weenie/interface/winterface.cpp.c` (0/2)

### IntrusiveHashTable<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,Position>_*>
- [-] `IntrusiveHashTable<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,Position>_*>::resize_internal` · 0052ec00 · legions 004a5670 — Rust std replaces it

### IntrusiveHashTable<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,FunctionID>_*>
- [-] `IntrusiveHashTable<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,FunctionID>_*>::IntrusiveHashTable<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,FunctionID>_*>` · 0052edc0 · legions 00512fd0 — Rust std replaces it

## `portal/shared/src/weenie/wphysobj/wphysicsobject.cpp.c` (0/4)

### CWPhysicsObject
- [-] `CWPhysicsObject::_SetReportCollisions` · 00530a10 · legions 005149b0 — Rust std replaces it
- [-] `CWPhysicsObject::_SetIgnoreCollisions` · 00530a30 · legions 005149d0 — Rust std replaces it
- [-] `CWPhysicsObject::_SetEdgeSlide` · 00530a50 · legions 005149f0 — Rust std replaces it
- [-] `CWPhysicsObject::_SetScale` · 00530a80 · legions 00514a20 — Rust std replaces it
