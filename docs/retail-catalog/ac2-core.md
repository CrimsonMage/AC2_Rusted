# ac2-core: Core runtime: ids, strings, timers, errors, serialization helpers

589 retail functions: 16 done, 0 partial, 169 to do, 404 skipped.
Legend and workflow: [README](README.md).

## `core/src/debug/crashcleaners.cpp.c` (0/3)

### CrashCleaners
- [-] `CrashCleaners::CallCrashCleaners` · 00598500 · legions 00562ab0 — crash reporting and debug tooling

### CrashCleaner
- [-] `CrashCleaner::~CrashCleaner` · 00598540 · legions 00562af0 — crash reporting and debug tooling
- [-] `CrashCleaner::CrashCleaner` · 005985c0 · legions 00562b70 — crash reporting and debug tooling

## `core/src/debug/dbghelpwrapper/dbghelpwrapper.cpp.c` (0/14)

### DbgHelpWrapper
- [-] `DbgHelpWrapper::DbgHelpVersionIsSufficient` · 00598da0 — crash reporting and debug tooling
- [-] `DbgHelpWrapper::FreeDbgHelpFunctions` · 00598eb0 · legions 00566560 — crash reporting and debug tooling
- [-] `DbgHelpWrapper::StackWalk` · 00598f10 — crash reporting and debug tooling
- [-] `DbgHelpWrapper::SymFunctionTableAccess` · 00598f50 — crash reporting and debug tooling
- [-] `DbgHelpWrapper::SymSetOptions` · 00598f60 — crash reporting and debug tooling
- [-] `DbgHelpWrapper::SymGetOptions` · 00598f80 — crash reporting and debug tooling
- [-] `DbgHelpWrapper::SymInitialize` · 00598f90 — crash reporting and debug tooling
- [-] `DbgHelpWrapper::SymCleanup` · 00598fb0 — crash reporting and debug tooling
- [-] `DbgHelpWrapper::SymUnloadModule` · 00598fd0 — crash reporting and debug tooling
- [-] `DbgHelpWrapper::SymLoadModule` · 00598ff0 — crash reporting and debug tooling
- [-] `DbgHelpWrapper::SymGetModuleInfo` · 00599020 — crash reporting and debug tooling
- [-] `DbgHelpWrapper::SymGetSymFromAddr` · 00599040 — crash reporting and debug tooling
- [-] `DbgHelpWrapper::SymGetLineFromAddr` · 00599070 — crash reporting and debug tooling
- [-] `DbgHelpWrapper::LoadDbgHelpFunctions` · 00599080 — crash reporting and debug tooling

## `core/src/debug/debug.cpp.c` (0/40)

### GameSettings
- [-] `GameSettings::GetAssertionsEmailAddress` · 00592630 · legions 00561c60 — crash reporting and debug tooling

### PortalLogFile
- [-] `PortalLogFile::ChangeLogFile` · 00592640 · legions 00561cf0 — crash reporting and debug tooling
- [-] `PortalLogFile::GetLogFileName` · 00592720 · legions 00561dd0 — crash reporting and debug tooling
- [-] `PortalLogFile::PrintToFile` · 00592730 · legions 00561de0 — crash reporting and debug tooling
- [-] `PortalLogFile::OutputToScreen` · 00592760 — crash reporting and debug tooling

### Turbine::Debug
- [-] `Turbine::Debug::SetDebugFlags` · 005927b0 · legions 00561e10 — crash reporting and debug tooling
- [-] `Turbine::Debug::GetDebugFlags` · 005927e0 · legions 00561e40 — crash reporting and debug tooling
- [-] `Turbine::Debug::IsDebugFlagSet` · 005927f0 · legions 00561e50 — crash reporting and debug tooling

### (free functions)
- [-] `AttachDebugger` · 00592810 — crash reporting and debug tooling

### Turbine::Debug
- [-] `Turbine::Debug::EnterDebugger` · 00592970 — crash reporting and debug tooling

### (free functions)
- [-] `ExceptionFilterThread` · 005929a0 — crash reporting and debug tooling
- [-] `RunInBackupStack` · 005929b0 — crash reporting and debug tooling

### Turbine::Debug
- [-] `Turbine::Debug::Debug` · 00592a20 · legions 00561e90 — crash reporting and debug tooling
- [-] `Turbine::Debug::~Debug` · 00592a40 · legions 00561eb0 — crash reporting and debug tooling
- [-] `Turbine::Debug::OutputStringToUserWithoutUI` · 00592a90 · legions 00561f10 — crash reporting and debug tooling
- [-] `Turbine::Debug::OutputStringToFileInternal` · 00592ab0 — crash reporting and debug tooling

### FrillRender
- [-] `FrillRender::Refresh` · 00592af0 · legions 00561f70 — crash reporting and debug tooling

### AssertMsgFmtStruct
- [-] `AssertMsgFmtStruct::AssertMsgFmtStruct` · 00592b00 — crash reporting and debug tooling

### Turbine::Debug
- [-] `Turbine::Debug::SetFloatingPointException` · 00592b30 — crash reporting and debug tooling
- [-] `Turbine::Debug::LogCriticalMessage` · 00592b70 · legions 00561f80 — crash reporting and debug tooling

### SuperGenericDlg
- [-] `SuperGenericDlg::SuperGenericDlg` · 00592c10 · legions 00562020 — crash reporting and debug tooling

### NormalDialogTextObject
- [-] `NormalDialogTextObject::NormalDialogTextObject` · 00592c90 — crash reporting and debug tooling
- [-] `NormalDialogTextObject::~NormalDialogTextObject` · 00592cd0 — crash reporting and debug tooling

### DbgReportObject
- [-] `DbgReportObject::ResetCachedReport` · 00592d50 · legions 005620f0 — crash reporting and debug tooling

### SuperGenericDlg
- [-] `SuperGenericDlg::~SuperGenericDlg` · 00592da0 · legions 00562140 — crash reporting and debug tooling

### Turbine::Debug
- [-] `Turbine::Debug::Init` · 00592e10 · legions 005621b0 — crash reporting and debug tooling

### (free functions)
- [-] `MessageBoxThread` · 00592eb0 · legions 00562230 — crash reporting and debug tooling

### Turbine::Debug
- [-] `Turbine::Debug::GUI_Dlg` · 00592fc0 — crash reporting and debug tooling
- [-] `Turbine::Debug::OutputString` · 005930a0 · legions 005623c0 — crash reporting and debug tooling
- [-] `Turbine::Debug::OutputStringArg` · 005932b0 · legions 005625d0 — crash reporting and debug tooling
- [-] `Turbine::Debug::printf` · 005933b0 · legions 005626d0 — crash reporting and debug tooling
- [-] `Turbine::Debug::OutputStringToUserInternal` · 005933d0 — crash reporting and debug tooling
- [-] `Turbine::Debug::Abort` · 00593510 · legions 00561e70 — crash reporting and debug tooling
- [-] `Turbine::Debug::OutputString` · 00593550 — crash reporting and debug tooling
- [-] `Turbine::Debug::ReportException` · 00593600 — crash reporting and debug tooling
- [-] `Turbine::Debug::AssertFail` · 00593810 — crash reporting and debug tooling
- [-] `Turbine::Debug::AssertFailMsg` · 005938c0 — crash reporting and debug tooling
- [-] `Turbine::Debug::TurbineExceptionFilter` · 00593920 — crash reporting and debug tooling
- [-] `Turbine::Debug::InitCommon` · 00593a70 — crash reporting and debug tooling
- [-] `Turbine::Debug::InitDebug` · 00593c70 — crash reporting and debug tooling

## `core/src/debug/dialogboxgateway.cpp.c` (0/6)

### DialogBoxGateways
- [-] `DialogBoxGateways::ReportThread_PrepareForDialogBox` · 00593d20 · legions 00562830 — crash reporting and debug tooling
- [-] `DialogBoxGateways::ReportThread_DoneWithDialogBox` · 00593d90 · legions 005628a0 — crash reporting and debug tooling
- [-] `DialogBoxGateways::DialogThread_PrepareForDialogBox` · 00593df0 · legions 00562900 — crash reporting and debug tooling
- [-] `DialogBoxGateways::DialogThread_DoneWithDialogBox` · 00593e60 · legions 00562970 — crash reporting and debug tooling

### DialogBoxGateway
- [-] `DialogBoxGateway::~DialogBoxGateway` · 00593ec0 · legions 005629d0 — crash reporting and debug tooling
- [-] `DialogBoxGateway::DialogBoxGateway` · 00593f40 · legions 00562a50 — crash reporting and debug tooling

## `core/src/debug/dwexceptionhandler.cpp.c` (0/9)

### TurbineDwExceptionFilter::TurnOnDwExceptionHandler
- [-] `TurbineDwExceptionFilter::TurnOnDwExceptionHandler::AddDllToDumpCapture` · 00734a30 · legions 006afed0 — crash reporting and debug tooling
- [-] `TurbineDwExceptionFilter::TurnOnDwExceptionHandler::SetRegistryKeysForWatson` · 00734a90 · legions 006aff30 — crash reporting and debug tooling

### TurbineDwExceptionFilter
- [-] `TurbineDwExceptionFilter::AddDllToDumpCapture` · 00734b50 · legions 006afff0 — crash reporting and debug tooling

### TurbineDwExceptionFilter::TurnOnDwExceptionHandler
- [-] `TurbineDwExceptionFilter::TurnOnDwExceptionHandler::~TurnOnDwExceptionHandler` · 00734b60 · legions 006b0000 — crash reporting and debug tooling
- [-] `TurbineDwExceptionFilter::TurnOnDwExceptionHandler::BuildListOfFilesToUpload` · 00734bc0 · legions 006b0060 — crash reporting and debug tooling
- [-] `TurbineDwExceptionFilter::TurnOnDwExceptionHandler::DwExceptionFilter` · 00734c30 · legions 006b00d0 — crash reporting and debug tooling

### TurbineDwExceptionFilter::WatsonCrashReportHelper
- [-] `TurbineDwExceptionFilter::WatsonCrashReportHelper::~WatsonCrashReportHelper` · 00735140 · legions 006b05e0 — crash reporting and debug tooling

### TurbineDwExceptionFilter
- [-] `TurbineDwExceptionFilter::InstallWatsonExceptionHandler` · 007351c0 · legions 006b0660 — crash reporting and debug tooling

### TurbineDwExceptionFilter::WatsonCrashReportHelper
- [-] `TurbineDwExceptionFilter::WatsonCrashReportHelper::WatsonCrashReportHelper` · 00735210 · legions 006b0780 — crash reporting and debug tooling

## `core/src/debug/extendedmapi.cpp.c` (0/6)

### ExtendedMAPI
- [-] `ExtendedMAPI::~ExtendedMAPI` · 00598620 · legions 006b07f0 — crash reporting and debug tooling
- [-] `ExtendedMAPI::LoadMapiExFunctions` · 00598640 · legions 006b0810 — crash reporting and debug tooling
- [-] `ExtendedMAPI::SetOutGoingMessageProps` · 00598740 · legions 006b0910 — crash reporting and debug tooling
- [-] `ExtendedMAPI::GetDefMsgStore` · 005987c0 · legions 006b0990 — crash reporting and debug tooling
- [-] `ExtendedMAPI::ReallySendMessage` · 00598890 · legions 006b0a60 — crash reporting and debug tooling
- [-] `ExtendedMAPI::SendMessageA` · 00598bc0 · legions 006b0d90 — crash reporting and debug tooling

## `core/src/debug/reportdlg.cpp.c` (0/33)

### `anonymous_namespace'
- [-] ``anonymous_namespace'::WriteToClipboard` · 00593fa0 · legions 00562be2 — crash reporting and debug tooling

### `anonymous-namespace'::AutoDialog
- [-] ``anonymous-namespace'::AutoDialog::OnCommand` · 00594020 · legions 00562c50 — crash reporting and debug tooling
- [-] ``anonymous-namespace'::AutoDialog::OnActivate` · 00594060 · legions 00562c90 — crash reporting and debug tooling
- [-] ``anonymous-namespace'::AutoDialog::DlgProc` · 00594090 · legions 00562cc0 — crash reporting and debug tooling
- [-] ``anonymous-namespace'::AutoDialog::OnInternalSetup` · 00594140 · legions 00562d70 — crash reporting and debug tooling
- [-] ``anonymous-namespace'::AutoDialog::Create` · 005941a0 · legions 00562dd0 — crash reporting and debug tooling
- [-] ``anonymous-namespace'::AutoDialog::GetCreationStyles` · 00594260 · legions 00562e90 — crash reporting and debug tooling
- [-] ``anonymous-namespace'::AutoDialog::Destroy` · 00594270 · legions 00562ea0 — crash reporting and debug tooling
- [-] ``anonymous-namespace'::AutoDialog::CenterWindow` · 00594290 · legions 00562ec0 — crash reporting and debug tooling
- [-] ``anonymous-namespace'::AutoDialog::GetNonClientArea` · 005943a0 · legions 00562fd0 — crash reporting and debug tooling
- [-] ``anonymous-namespace'::AutoDialog::ForceFont` · 00594420 · legions 00563050 — crash reporting and debug tooling

### `anonymous-namespace'::ButtonyAutoDialog
- [-] ``anonymous-namespace'::ButtonyAutoDialog::GetButtonInfo` · 00594470 · legions 005630a0 — crash reporting and debug tooling
- [-] ``anonymous-namespace'::ButtonyAutoDialog::PositionButtons` · 00594620 · legions 00563250 — crash reporting and debug tooling
- [-] ``anonymous-namespace'::ButtonyAutoDialog::GetTextBoxSize` · 005946d0 · legions 00563300 — crash reporting and debug tooling

### `anonymous-namespace'::AdvancedAssertionDialog
- [-] ``anonymous-namespace'::AdvancedAssertionDialog::OnActivate` · 00594780 — crash reporting and debug tooling
- [-] ``anonymous-namespace'::AdvancedAssertionDialog::GetCreationStyles` · 005947d0 — crash reporting and debug tooling

### `anonymous-namespace'::SemiGenericDialog
- [-] ``anonymous-namespace'::SemiGenericDialog::OnTimer` · 005947e0 — crash reporting and debug tooling

### ReportDlg
- [-] `ReportDlg::ReportDlg` · 00594820 · legions 00563450 — crash reporting and debug tooling
- [-] `ReportDlg::~ReportDlg` · 00594830 · legions 00563460 — crash reporting and debug tooling

### SuperGenericDlg
- [-] `SuperGenericDlg::Create` · 00594840 · legions 00563470 — crash reporting and debug tooling

### `anonymous_namespace'
- [-] ``anonymous_namespace'::IsRegBoolTrue` · 00594890 · legions 005634c0 — crash reporting and debug tooling

### `anonymous-namespace'::EMailDialog
- [-] ``anonymous-namespace'::EMailDialog::OnInit` · 00594970 · legions 00563600 — crash reporting and debug tooling

### `anonymous-namespace'::BasicAssertionDialog
- [-] ``anonymous-namespace'::BasicAssertionDialog::OnCommand` · 00594c70 — crash reporting and debug tooling

### `anonymous_namespace'
- [-] ``anonymous_namespace'::GetExecutableName` · 00594da0 · legions 00563b90 — crash reporting and debug tooling
- [-] ``anonymous_namespace'::SendEmail` · 00594e10 — crash reporting and debug tooling

### `anonymous-namespace'::EMailDialog
- [-] ``anonymous-namespace'::EMailDialog::OnCommand` · 005950f0 · legions 00563c00 — crash reporting and debug tooling

### `anonymous-namespace'::AdvancedAssertionDialog
- [-] ``anonymous-namespace'::AdvancedAssertionDialog::OnCommand` · 00595420 — crash reporting and debug tooling

### `anonymous-namespace'::SemiGenericDialog
- [-] ``anonymous-namespace'::SemiGenericDialog::OnCommand` · 00595620 — crash reporting and debug tooling

### ReportDlg
- [-] `ReportDlg::Create` · 00595780 · legions 00564130 — crash reporting and debug tooling

### `anonymous_namespace'
- [-] ``anonymous_namespace'::BloatCarriageReturns` · 00595bc0 · legions 00564570 — crash reporting and debug tooling

### `anonymous-namespace'::AdvancedAssertionDialog
- [-] ``anonymous-namespace'::AdvancedAssertionDialog::OnInit` · 00595c70 — crash reporting and debug tooling

### `anonymous-namespace'::BasicAssertionDialog
- [-] ``anonymous-namespace'::BasicAssertionDialog::OnInit` · 005961e0 — crash reporting and debug tooling

### `anonymous-namespace'::SemiGenericDialog
- [-] ``anonymous-namespace'::SemiGenericDialog::OnInit` · 00596580 — crash reporting and debug tooling

## `core/src/debug/reportgen.cpp.c` (0/21)

### CSymbolEngine
- [-] `CSymbolEngine::SymGetLineFromAddr` · 005968f0 — crash reporting and debug tooling

### (free functions)
- [-] `GetExceptionString` · 00596970 · legions 005652a0 — crash reporting and debug tooling

### CppDbgReportGenPage
- [-] `CppDbgReportGenPage::ShouldReport` · 00596bf0 — crash reporting and debug tooling

### (free functions)
- [-] `ConvertAddress` · 00596c10 — crash reporting and debug tooling

### DbgReportGenerator
- [-] `DbgReportGenerator::RemovePage` · 00596e90 · legions 00565590 — crash reporting and debug tooling

### DbgReportGenPage
- [-] `DbgReportGenPage::~DbgReportGenPage` · 00596ee0 · legions 008c2700 — crash reporting and debug tooling

### DbgReportObject
- [-] `DbgReportObject::DbgReportObject` · 00596f90 · legions 005656c0 — crash reporting and debug tooling
- [-] `DbgReportObject::~DbgReportObject` · 00597000 · legions 00565730 — crash reporting and debug tooling

### DbgReportGenerator
- [-] `DbgReportGenerator::AddPage` · 00597170 · legions 005658a0 — crash reporting and debug tooling

### DbgReportGenPage
- [-] `DbgReportGenPage::DbgReportGenPage` · 005971d0 · legions 00565900 — crash reporting and debug tooling

### VersionDbgReportGenPage
- [-] `VersionDbgReportGenPage::GetReportName` · 005971f0 · legions 00565920 — crash reporting and debug tooling

### CppDbgReportGenPage
- [-] `CppDbgReportGenPage::GetReportName` · 00597240 · legions 00565940 — crash reporting and debug tooling

### DbgReportGenerator
- [-] `DbgReportGenerator::GenReportSummary` · 00597330 · legions 00565a30 — crash reporting and debug tooling

### VersionDbgReportGenPage
- [-] `VersionDbgReportGenPage::GenReport` · 005973e0 — crash reporting and debug tooling

### (free functions)
- [-] `CleanTheHellUp` · 00597730 — crash reporting and debug tooling

### DbgReportObject
- [-] `DbgReportObject::GetSummary` · 005977c0 · legions 00565fd0 — crash reporting and debug tooling

### (free functions)
- [-] `GetModBase` · 00597910 — crash reporting and debug tooling

### DbgReportGenerator
- [-] `DbgReportGenerator::GenReport` · 00597c60 · legions 00566220 — crash reporting and debug tooling

### (free functions)
- [-] `DoStackTrace` · 00597f00 — crash reporting and debug tooling

### DbgReportObject
- [-] `DbgReportObject::GetReport` · 00598200 · legions 005664c0 — crash reporting and debug tooling

### CppDbgReportGenPage
- [-] `CppDbgReportGenPage::GenReport` · 005982a0 · legions 00565e30 — crash reporting and debug tooling

## `core/src/falloc/falloc.cpp.c` (0/3)

### FallocPool
- [ ] `FallocPool::FallocPool` · 00662fc0 · legions 00611150
- [ ] `FallocPool::~FallocPool` · 00663000 · legions 00611190
- [ ] `FallocPool::AllocateNewPool` · 00663040 · legions 006111d0

## `core/src/newthread/portaldumperr.cpp.c` (0/1)

### PortalThreadErrorDump
- [ ] `PortalThreadErrorDump::DumpLastError` · 0063c8e0 · legions 005f1410

## `core/src/newthread/portalevent_win32.cpp.c` (0/4)

### PortalEvent
- [ ] `PortalEvent::PortalEvent` · 0063c870 · legions 005f13a0
- [ ] `PortalEvent::~PortalEvent` · 0063c890 · legions 005f13c0
- [ ] `PortalEvent::Signal` · 0063c8b0 · legions 005f13e0
- [ ] `PortalEvent::WaitForSignal` · 0063c8c0 · legions 005f13f0

## `core/src/newthread/portalthread_win32.cpp.c` (0/5)

### PortalThread
- [ ] `PortalThread::~PortalThread` · 0063c700 · legions 005f1250
- [ ] `PortalThread::Resume` · 0063c720 · legions 005f1270
- [ ] `PortalThread::WaitFor` · 0063c760
- [ ] `PortalThread::Join` · 0063c790 · legions 005f12e0
- [ ] `PortalThread::PortalThread` · 0063c7f0 · legions 005f1320

## `core/src/newthread/sharedcritsec_win32.cpp.c` (0/4)

### SharedCriticalSection
- [ ] `SharedCriticalSection::SharedCriticalSection` · 0063c650 · legions 005f11a0
- [ ] `SharedCriticalSection::~SharedCriticalSection` · 0063c670 · legions 005f11c0
- [ ] `SharedCriticalSection::EnterCriticalSection` · 0063c690 · legions 005f11e0
- [ ] `SharedCriticalSection::LeaveCriticalSection` · 0063c6b0 · legions 005f1200

## `core/src/portable/pfile.cpp.c` (0/1)

### (free functions)
- [ ] `GetEnvironmentPath` · 00737e60 · legions 006b3a20

## `core/src/pstring/pstringdecls.cpp.c` (0/1)

### (free functions)
- [ ] `ConvertWPStringToCIWPstring` · 0064b6c0 · legions 005fa9f0

## `core/src/serialize/smartbuffer.cpp.c` (0/12)

### AC2CoreSDKCopy::GrowBuffer
- [ ] `AC2CoreSDKCopy::GrowBuffer::FreeBuffer` · 00735e50 · legions 006b1af0

### AC2CoreSDKCopy::SmartBuffer
- [ ] `AC2CoreSDKCopy::SmartBuffer::GetBuffer` · 00735e90 · legions 006b1b30
- [ ] `AC2CoreSDKCopy::SmartBuffer::SmartBuffer` · 00735ed0 · legions 006b1b70
- [ ] `AC2CoreSDKCopy::SmartBuffer::SmartBuffer` · 00735f40 · legions 006b1be0

### `AC2CoreSDKCopy::GrowBuffer::GrowExact'::__l2::FreeListLockAllocator
- [ ] ``AC2CoreSDKCopy::GrowBuffer::GrowExact'::__l2::FreeListLockAllocator::~FreeListLockAllocator` · 00735f70 · legions 006b1c10

### AC2CoreSDKCopy::SmartBuffer
- [ ] `AC2CoreSDKCopy::SmartBuffer::~SmartBuffer` · 00735fc0 · legions 006b1c60
- [ ] `AC2CoreSDKCopy::SmartBuffer::operator=` · 00736000 · legions 006b1ca0
- [ ] `AC2CoreSDKCopy::SmartBuffer::MakeWindow` · 00736070 · legions 006b1d10

### AC2CoreSDKCopy::GrowBuffer
- [ ] `AC2CoreSDKCopy::GrowBuffer::GrowExact` · 00736100 · legions 006b1da0
- [ ] `AC2CoreSDKCopy::GrowBuffer::Grow` · 007361c0 · legions 006b1e60

### AC2CoreSDKCopy::SmartBuffer
- [ ] `AC2CoreSDKCopy::SmartBuffer::Grow` · 00736200 · legions 006b1ea0

### `AC2CoreSDKCopy::GrowBuffer::GrowExact'::__l2
- [ ] ``AC2CoreSDKCopy::GrowBuffer::GrowExact'::__l2::s_FreeListHelper` · 00a43c30

## `core/src/timer/timer.cpp.c` (3/11)

### (free functions)
- [ ] `get_elapsed_time_internal` · 0064b0c0 · legions 005fa3f0

### Timer
- [ ] `Timer::get_real_time` · 0064b310 · legions 005fa640

### CSpinLock<512,0>
- [ ] `CSpinLock<512,0>::Enter` · 0064b320 · legions 005fa650

### Win32TimeWrapper
- [ ] `Win32TimeWrapper::init` · 0064b370 · legions 005fa6a0
- [ ] `Win32TimeWrapper::get_elapsed_time` · 0064b460 · legions 005fa790

### Timer
- [x] `Timer::Init` · 0064b540 · legions 005fa870
- [ ] `Timer::asctime` · 0064b5a0 · legions 005fa8d0
- [ ] `Timer::strftime` · 0064b5d0 · legions 005fa900
- [ ] `Timer::get_elapsed_time` · 0064b610 · legions 00455160
- [x] `Timer::set_time` · 0064b620 · legions 005fa950
- [x] `Timer::update_time` · 0064b660 · legions 005fa990

## `core/src/utils/strtoint64.cpp.c` (0/6)

### (free functions)
- [ ] `strtoxint64` · 0065f720 · legions 0060e030
- [ ] `strtoint64` · 0065fa00 · legions 0060e310
- [ ] `strtouint64` · 0065fa20 · legions 0060e330
- [ ] `wcstoxint64` · 0065fa40 · legions 0060e350
- [ ] `wcstoint64` · 0065fcf0 · legions 0060e600
- [ ] `wcstouint64` · 0065fd10 · legions 0060e620

## `portal/client/src/bench/bench.cpp.c` (0/30)

### (free functions)
- [-] `benchModeToggle` · 00479b20 — benchmarks (admin tools)
- [-] `benchBinResetAll` · 00479b30 — benchmarks (admin tools)
- [-] `benchBinDestroy` · 00479b80 — benchmarks (admin tools)
- [-] `benchBinInBin` · 00479bd0 — benchmarks (admin tools)
- [-] `benchBinStart` · 00479bf0 — benchmarks (admin tools)
- [-] `benchBinFinish` · 00479c50 — benchmarks (admin tools)
- [-] `benchGraphDestroy` · 00479ce0 — benchmarks (admin tools)
- [-] `benchGraphReset` · 00479d50 — benchmarks (admin tools)
- [-] `benchGraphResetAll` · 00479d90 — benchmarks (admin tools)
- [-] `benchBinStat` · 00479dd0 — benchmarks (admin tools)
- [-] `benchDisplayBegin` · 00479e60 — benchmarks (admin tools)
- [-] `benchDisplayEnd` · 00479f50 — benchmarks (admin tools)

### benchBin
- [-] `benchBin::Init` · 00479fa0 — benchmarks (admin tools)

### (free functions)
- [-] `benchBinCreate` · 0047a000 — benchmarks (admin tools)
- [-] `benchGraphCreate` · 0047a050 — benchmarks (admin tools)

### benchStat
- [-] `benchStat::Cleanup` · 0047a340 — benchmarks (admin tools)
- [-] `benchStat::Update` · 0047a390 — benchmarks (admin tools)

### PerfMonCounterInfo::PerfMonCounterNameHelp
- [-] `PerfMonCounterInfo::PerfMonCounterNameHelp::operator=` · 0047a730 · legions 004ba620 — benchmarks (admin tools)

### (free functions)
- [-] `benchEnd` · 0047a7b0 — benchmarks (admin tools)
- [-] `benchGraphDraw` · 0047a880 — benchmarks (admin tools)
- [-] `benchGraphFrame` · 0047af00 — benchmarks (admin tools)

### benchStat
- [-] `benchStat::Render` · 0047af90 — benchmarks (admin tools)

### (free functions)
- [-] `benchUpdate` · 0047b250 — benchmarks (admin tools)

### List<PerfMonCounterInfo::PerfMonCounterNameHelp>
- [-] `List<PerfMonCounterInfo::PerfMonCounterNameHelp>::~List<PerfMonCounterInfo::PerfMonCounterNameHelp>` · 0047b440 · legions 004bdb80 — benchmarks (admin tools)

### PerfMonCounterInfo
- [-] `PerfMonCounterInfo::~PerfMonCounterInfo` · 0047b4b0 · legions 004bdbf0 — benchmarks (admin tools)

### PerfMonCounter<unsigned_long>
- [-] `PerfMonCounter<unsigned_long>::IncCounter` · 0047b4e0 — benchmarks (admin tools)
- [-] `PerfMonCounter<unsigned_long>::DecCounter` · 0047b4f0 — benchmarks (admin tools)
- [-] `PerfMonCounter<unsigned_long>::GetValueAsInt64` · 0047b500 — benchmarks (admin tools)

### benchStat
- [-] `benchStat::Init` · 0047b510 — benchmarks (admin tools)

### (free functions)
- [-] `benchBegin` · 0047b780 — benchmarks (admin tools)

## `portal/client/src/bench/benchquery.cpp.c` (0/2)

### (free functions)
- [-] `p5TimeCalibrateReset` · 0047b810 — benchmarks (admin tools)
- [-] `p5TimeCalibrate` · 0047b840 — benchmarks (admin tools)

## `portal/client/src/cerror/clienterror.cpp.c` (0/1)

### ClientError
- [ ] `ClientError::Init` · 00492ba0 · legions 0048b550

## `portal/client/src/platformstring/platformstring.cpp.c` (0/4)

### PlatformString::StringAccessor
- [ ] `PlatformString::StringAccessor::GetString` · 00569a90 · legions 00539a20

### PlatformString
- [ ] `PlatformString::GetString` · 00569be0 · legions 00539b70
- [ ] `PlatformString::DisplayFinalString` · 00569d40 · legions 00539cd0
- [ ] `PlatformString::DisplayString` · 00569f20 · legions 00539eb0

## `portal/shared/src/error/baseerror/baseerror.cpp.c` (0/2)

### BaseError
- [ ] `BaseError::Init` · 00614e70 · legions 005d2ba0
- [ ] `BaseError::ReportGenericErrorInternal` · 00614ee0 · legions 005d2c10

## `portal/shared/src/error/dataerror/dataerror.cpp.c` (0/15)

### DataError
- [ ] `DataError::ReportGenericErrorInternal` · 00614f40 · legions 005d2c70

### _STL::less<PStringBase<char>_>
- [ ] `_STL::less<PStringBase<char>_>::operator()` · 00614fb0 · legions 005d2ce0

### DataError
- [ ] `DataError::GetFileName` · 00615130 · legions 005d2e60
- [ ] `DataError::ReportDataErrorInternal` · 00615290 · legions 005d2fc0
- [ ] `DataError::ReportMissingDataInternal` · 006152e0 · legions 005d3010
- [ ] `DataError::ReportMissingDataEnumInternal` · 00615340 · legions 005d3070
- [ ] `DataError::~DataError` · 00615920 · legions 005d3650
- [ ] `DataError::ResetDataFileInternal` · 00615980 · legions 005d36b0
- [ ] `DataError::ResetReportedErrorsInternal` · 00615a10 · legions 005d3740

### _STL::map<PStringBase<char>,PStringBase<char>,_STL::less<PStringBase<char>_>,_STL::allocator<_STL::pair<PStringBase<char>_const_,PStringBase<char>_>_>_>
- [ ] `_STL::map<PStringBase<char>,PStringBase<char>,_STL::less<PStringBase<char>_>,_STL::allocator<_STL::pair<PStringBase<char>_const_,PStringBase<char>_>_>_>::operator[]` · 00615a50 · legions 005d3780

### DataError
- [ ] `DataError::DataError` · 00615b30 · legions 005d3860
- [ ] `DataError::ReportErrorFromFileInternal` · 00615ba0 · legions 005d38d0
- [ ] `DataError::ReportDataErrorInternal` · 00615c10 · legions 005d3940
- [ ] `DataError::ReportMissingFileInternal` · 00615c90 · legions 005d39c0
- [ ] `DataError::Init` · 00615d40 · legions 005d3a70

## `portal/shared/src/error/ierror.cpp.c` (0/25)

### (free functions)
- [ ] `DisplayErrorDialogThread` · 00613fb0

### LongNIHash<NIList<NoticeHandler_*>_>
- [ ] `LongNIHash<NIList<NoticeHandler_*>_>::LongNIHash<NIList<NoticeHandler_*>_>` · 00613fc0 · legions 0048b760
- [ ] `LongNIHash<NIList<NoticeHandler_*>_>::flush` · 00614020 · legions 0048b820

### IError
- [ ] `IError::PrintToFile` · 00614070 · legions 005d2030

### LongNIHash<IError>
- [ ] `LongNIHash<IError>::remove` · 006140f0 · legions 005d20b0

### IError::StackEmailDialogTextObject
- [ ] `IError::StackEmailDialogTextObject::StackEmailDialogTextObject` · 00614160 · legions 005d2120

### IError
- [ ] `IError::IError` · 00614220 · legions 005d21e0
- [ ] `IError::Cleanup` · 00614250 · legions 005d2210
- [ ] `IError::GetCurrError` · 00614290 · legions 005d2250
- [ ] `IError::ReportGenericError` · 00614300 · legions 005d22c0
- [ ] `IError::~IError` · 00614320 · legions 005d22e0
- [ ] `IError::SetCurrentErrorHandler` · 006143a0 · legions 005d2360
- [ ] `IError::SetLogFile` · 00614470 · legions 005d2430
- [ ] `IError::ReportDataErrorInternal` · 00614580 · legions 005d2540
- [ ] `IError::ReportMissingFileInternal` · 006145d0 · legions 005d2590
- [ ] `IError::ReportMissingDataInternal` · 00614620 · legions 005d25e0
- [ ] `IError::ReportMissingDataEnumInternal` · 00614670 · legions 005d2630
- [ ] `IError::ReportGenericErrorArg` · 006146c0
- [ ] `IError::ReportDataErrorFrom` · 00614730
- [ ] `IError::ReportDataErrorFrom` · 006147b0
- [ ] `IError::PopUpWindow` · 00614830 · legions 005d26f0

### IError::StackEmailDialogTextObject
- [ ] `IError::StackEmailDialogTextObject::GetText` · 00614c30

### IError
- [ ] `IError::ReportMissingFileFrom` · 00614ce0
- [ ] `IError::ReportMissingDataFrom` · 00614d60
- [ ] `IError::ReportMissingDataEnumFrom` · 00614de0

## `portal/shared/src/filenode/filenodename.cpp.c` (0/42)

### IFileNodeName
- [-] `IFileNodeName::GetNameType` · 00665420 · legions 00612d60 — text source files (admin tools)

### FileNodeName_UInt32
- [-] `FileNodeName_UInt32::Pack` · 00665440 · legions 00612d80 — text source files (admin tools)
- [-] `FileNodeName_UInt32::UnPack` · 00665460 · legions 00612da0 — text source files (admin tools)

### FileNodeName_UInt64
- [-] `FileNodeName_UInt64::Pack` · 00665480 · legions 00612dc0 — text source files (admin tools)
- [-] `FileNodeName_UInt64::UnPack` · 006654a0 · legions 00612de0 — text source files (admin tools)
- [-] `FileNodeName_UInt64::operator==` · 006654c0 · legions 00612e00 — text source files (admin tools)

### FileNodeName_Double
- [-] `FileNodeName_Double::Pack` · 00665500 · legions 00612e40 — text source files (admin tools)
- [-] `FileNodeName_Double::operator==` · 00665520 · legions 00612e60 — text source files (admin tools)

### FileNodeName_Float
- [-] `FileNodeName_Float::Pack` · 00665560 · legions 00612ea0 — text source files (admin tools)
- [-] `FileNodeName_Float::operator==` · 00665580 · legions 00612ec0 — text source files (admin tools)

### FileNodeName_InstanceID
- [-] `FileNodeName_InstanceID::Pack` · 006655c0 · legions 00612f00 — text source files (admin tools)
- [-] `FileNodeName_InstanceID::operator==` · 006655e0 · legions 00612f20 — text source files (admin tools)

### FileNodeName_UInt32
- [-] `FileNodeName_UInt32::operator==` · 00665640 · legions 00612f80 — text source files (admin tools)

### FileNodeName_Binary
- [-] `FileNodeName_Binary::FileNodeName_Binary` · 00665680 · legions 00612fc0 — text source files (admin tools)
- [-] `FileNodeName_Binary::GetPackSize` · 006656e0 · legions 00613020 — text source files (admin tools)
- [-] `FileNodeName_Binary::Pack` · 00665700 · legions 00613040 — text source files (admin tools)
- [-] `FileNodeName_Binary::UnPack` · 00665750 · legions 00613090 — text source files (admin tools)
- [-] `FileNodeName_Binary::operator==` · 006657a0 · legions 006130e0 — text source files (admin tools)

### FileNodeName_PString
- [-] `FileNodeName_PString::operator==` · 00665820 · legions 00613160 — text source files (admin tools)

### PSUtils
- [-] `PSUtils::is_int32` · 00665860 · legions 006131a0 — text source files (admin tools)
- [-] `PSUtils::is_uint64` · 006658d0 · legions 00613210 — text source files (admin tools)
- [-] `PSUtils::is_int64` · 00665940 · legions 00613280 — text source files (admin tools)
- [-] `PSUtils::is_double` · 006659b0 · legions 006132f0 — text source files (admin tools)

### FileNodeName_UInt32
- [-] `FileNodeName_UInt32::FromPString` · 00665a20 · legions 00613360 — text source files (admin tools)

### FileNodeName_Int32
- [-] `FileNodeName_Int32::FromPString` · 00665a80 · legions 006133c0 — text source files (admin tools)

### FileNodeName_UInt64
- [-] `FileNodeName_UInt64::FromPString` · 00665ae0 · legions 00613540 — text source files (admin tools)

### FileNodeName_Int64
- [-] `FileNodeName_Int64::FromPString` · 00665b40 · legions 00613420 — text source files (admin tools)

### FileNodeName_Double
- [-] `FileNodeName_Double::FromPString` · 00665ba0 · legions 00613480 — text source files (admin tools)

### FileNodeName_Float
- [-] `FileNodeName_Float::FromPString` · 00665c00 · legions 006134e0 — text source files (admin tools)

### FileNodeName_Bool
- [-] `FileNodeName_Bool::FromPString` · 00665c60 · legions 006135a0 — text source files (admin tools)
- [-] `FileNodeName_Bool::ToPString` · 00665cc0 · legions 00613600 — text source files (admin tools)

### FileNodeName_Binary
- [-] `FileNodeName_Binary::ToPString` · 00665cf0 · legions 00613630 — text source files (admin tools)

### FileNodeName_UInt32
- [-] `FileNodeName_UInt32::ToPString` · 00665d80 · legions 006136c0 — text source files (admin tools)

### FileNodeName_Int32
- [-] `FileNodeName_Int32::ToPString` · 00665df0 · legions 00613730 — text source files (admin tools)

### FileNodeName_UInt64
- [-] `FileNodeName_UInt64::ToPString` · 00665e60 · legions 006137a0 — text source files (admin tools)

### FileNodeName_Int64
- [-] `FileNodeName_Int64::ToPString` · 00665ed0 · legions 00613810 — text source files (admin tools)

### FileNodeName_Double
- [-] `FileNodeName_Double::ToPString` · 00665f40 · legions 00613880 — text source files (admin tools)

### FileNodeName_Float
- [-] `FileNodeName_Float::ToPString` · 00665fb0 · legions 006138f0 — text source files (admin tools)

### FileNodeName_CellID
- [-] `FileNodeName_CellID::ToPString` · 00666020 — text source files (admin tools)

### FileNodeName_InstanceID
- [-] `FileNodeName_InstanceID::ToPString` · 00666090 · legions 00613960 — text source files (admin tools)

### FileNodeName_Hex
- [-] `FileNodeName_Hex::ToPString` · 00666100 · legions 006139d0 — text source files (admin tools)

### FileNodeName_Enum
- [-] `FileNodeName_Enum::ToPString` · 00666170 — text source files (admin tools)

## `portal/shared/src/filenode/pfilenode.cpp.c` (0/69)

### LM_Diffuse
- [-] `LM_Diffuse::GetType` · 006630b0 · legions 00611240 — text source files (admin tools)

### FileNodeName_Double
- [-] `FileNodeName_Double::Clone` · 006630c0 · legions 00611250 — text source files (admin tools)

### FileNodeName_Binary
- [-] `FileNodeName_Binary::GetType` · 00663110 · legions 006112a0 — text source files (admin tools)
- [-] `FileNodeName_Binary::Clone` · 00663120 · legions 006112b0 — text source files (admin tools)

### PFileNode
- [-] `PFileNode::SetLineNumber` · 00663180 · legions 00611310 — text source files (admin tools)
- [-] `PFileNode::SetColumnNumber` · 00663190 · legions 00611320 — text source files (admin tools)
- [-] `PFileNode::GetPackSize` · 006631a0 · legions 00611330 — text source files (admin tools)

### PFileNodeIter
- [-] `PFileNodeIter::Reset` · 006631b0 · legions 00611340 — text source files (admin tools)
- [-] `PFileNodeIter::Curr` · 006631c0 · legions 00611350 — text source files (admin tools)

### PFileNodeIter_NonConst
- [-] `PFileNodeIter_NonConst::Curr` · 006631e0 — text source files (admin tools)

### FileNodeName_Enum
- [-] `FileNodeName_Enum::GetType` · 00663280 — text source files (admin tools)
- [-] `FileNodeName_Enum::Clone` · 00663290 — text source files (admin tools)

### PFileNode
- [-] `PFileNode::GetNameAsDouble` · 006632c0 — text source files (admin tools)
- [-] `PFileNode::GetNameAsEnum` · 00663390 — text source files (admin tools)
- [-] `PFileNode::GetFileInfo` · 00663450 · legions 006113b0 — text source files (admin tools)

### DataBuffer
- [-] `DataBuffer::create_buffer` · 00663470 · legions 006113d0 — text source files (admin tools)

### PFileNode
- [-] `PFileNode::SetNameAsBinary` · 006634b0 · legions 00611410 — text source files (admin tools)
- [-] `PFileNode::MakeSubNodeAsBinary` · 00663530 — text source files (admin tools)
- [-] `PFileNode::GetSubNode` · 00663560 · legions 00611490 — text source files (admin tools)
- [-] `PFileNode::GetLeaf` · 00663580 · legions 006114b0 — text source files (admin tools)
- [-] `PFileNode::RemoveSubNode` · 006635a0 · legions 006114d0 — text source files (admin tools)
- [-] `PFileNode::EndParse` · 006635f0 — text source files (admin tools)
- [-] `PFileNode::SetCompatabilityVersion` · 00663630 — text source files (admin tools)
- [-] `PFileNode::pack_size` · 00663660 · legions 00611540 — text source files (admin tools)
- [-] `PFileNode::PackInternal` · 006636d0 · legions 006115b0 — text source files (admin tools)

### PFileNodeIter
- [-] `PFileNodeIter::Next` · 00663760 · legions 00611660 — text source files (admin tools)
- [-] `PFileNodeIter::~PFileNodeIter` · 00663790 · legions 00611640 — text source files (admin tools)

### PFileNodeIter_NonConst
- [-] `PFileNodeIter_NonConst::Next` · 006637b0 — text source files (admin tools)

### PFileNode
- [-] `PFileNode::FallocDump` · 00663880 — text source files (admin tools)
- [-] `PFileNode::GetSubName` · 006638d0 · legions 00611780 — text source files (admin tools)
- [-] `PFileNode::ConvertName` · 00663940 — text source files (admin tools)
- [-] `PFileNode::SetNameAsWPString` · 00663e20 · legions 006117f0 — text source files (admin tools)
- [-] `PFileNode::GetNameAsWPString` · 00663e80 · legions 00611850 — text source files (admin tools)
- [-] `PFileNode::GetLeafAsWPString` · 00663f60 · legions 00611930 — text source files (admin tools)
- [-] `PFileNode::GetSubNameAsWPString` · 00664060 · legions 00611a30 — text source files (admin tools)
- [-] `PFileNode::IsKeyword` · 00664150 · legions 00611b20 — text source files (admin tools)
- [-] `PFileNode::IsKeywordW` · 006641c0 · legions 00611b90 — text source files (admin tools)

### (free functions)
- [-] `PFileNodeSorter` · 00664210 — text source files (admin tools)

### PFileNode
- [-] `PFileNode::FindSubNode` · 006642e0 · legions 00611cb0 — text source files (admin tools)
- [-] `PFileNode::ClearComments` · 006643a0 · legions 00611d70 — text source files (admin tools)
- [-] `PFileNode::ReportErrorInternal` · 00664420 · legions 00611df0 — text source files (admin tools)
- [-] `PFileNode::ReportWarningInternal` · 006644b0 · legions 00611e50 — text source files (admin tools)
- [-] `PFileNode::ReportErrorWSL` · 00664540 · legions 00611eb0 — text source files (admin tools)
- [-] `PFileNode::ReportWarningWSL` · 00664590 · legions 00611f00 — text source files (admin tools)
- [-] `PFileNode::Pack` · 006645e0 · legions 00611f50 — text source files (admin tools)

### PFileNodeIter
- [-] `PFileNodeIter::FindInternal` · 00664610 · legions 00611fb0 — text source files (admin tools)
- [-] `PFileNodeIter::PFileNodeIter` · 00664700 · legions 00611f80 — text source files (admin tools)

### IntrusiveSmartPointer<FileNodeFileInfo>
- [-] `IntrusiveSmartPointer<FileNodeFileInfo>::create` · 00664730 · legions 006120a0 — text source files (admin tools)

### PFileNode
- [-] `PFileNode::PFileNode` · 006647b0 · legions 006121f0 — text source files (admin tools)
- [-] `PFileNode::Destroy` · 00664810 · legions 00612250 — text source files (admin tools)
- [-] `PFileNode::operator=` · 00664940 · legions 00612380 — text source files (admin tools)
- [-] `PFileNode::GetSourceFileName` · 00664a40 · legions 00612480 — text source files (admin tools)
- [-] `PFileNode::GetBinaryName` · 00664aa0 · legions 006124e0 — text source files (admin tools)
- [-] `PFileNode::AdoptSubNode` · 00664c20 · legions 00612660 — text source files (admin tools)
- [-] `PFileNode::AddAndRefSubNode` · 00664c70 · legions 006126b0 — text source files (admin tools)
- [-] `PFileNode::InitFileInfo` · 00664ca0 · legions 006126e0 — text source files (admin tools)
- [-] `PFileNode::BeginParse` · 00664d00 · legions 00612740 — text source files (admin tools)
- [-] `PFileNode::ReportError` · 00664db0 · legions 006127f0 — text source files (admin tools)
- [-] `PFileNode::ReportWarning` · 00664e00 · legions 00612840 — text source files (admin tools)
- [-] `PFileNode::ReportMissingFile` · 00664e50 — text source files (admin tools)
- [-] `PFileNode::UnPackInternal` · 00664e80 · legions 00612890 — text source files (admin tools)

### PFileNodeIter
- [-] `PFileNodeIter::Find` · 00664f50 · legions 00612960 — text source files (admin tools)
- [-] `PFileNodeIter::FindNext` · 00664fa0 · legions 006129b0 — text source files (admin tools)

### PFileNode
- [-] `PFileNode::~PFileNode` · 00664fe0 · legions 006129f0 — text source files (admin tools)
- [-] `PFileNode::PFileNode` · 00665060 · legions 00612a70 — text source files (admin tools)
- [-] `PFileNode::MakeSubNode` · 006650d0 · legions 00612ae0 — text source files (admin tools)
- [-] `PFileNode::AddComment` · 00665110 · legions 00612b20 — text source files (admin tools)
- [-] `PFileNode::UnPack` · 00665180 · legions 00612b90 — text source files (admin tools)
- [-] `PFileNode::SortSubNodes` · 00665400 · legions 00612d40 — text source files (admin tools)

## `portal/shared/src/perfmon/install/perfmoninstaller.cpp.c` (0/8)

### PerfMonInstaller
- [-] `PerfMonInstaller::InstallRegistryKeys` · 006f43c0 — performance monitor (admin tools)
- [-] `PerfMonInstaller::ProvideRPCServerName` · 006f4a30 — performance monitor (admin tools)
- [-] `PerfMonInstaller::AddCounterTypesToRegistry` · 006f4c10 — performance monitor (admin tools)

### List<PerfMonCounterInfo::PerfMonCounterNameHelp>
- [-] `List<PerfMonCounterInfo::PerfMonCounterNameHelp>::operator=` · 006f4d80 — performance monitor (admin tools)

### PerfMonInstaller
- [-] `PerfMonInstaller::PerfMonInstaller` · 006f4e30 — performance monitor (admin tools)
- [-] `PerfMonInstaller::FillInHeaderAndIniFiles` · 006f4fc0 — performance monitor (admin tools)
- [-] `PerfMonInstaller::SynchCounterOrderWithRegistry` · 006f5410 — performance monitor (admin tools)
- [-] `PerfMonInstaller::InstallCounters` · 006f5b50 — performance monitor (admin tools)

## `portal/shared/src/perfmon/perfmoncounterinfo.cpp.c` (0/6)

### PerfMonCounterInfo
- [-] `PerfMonCounterInfo::LookupExistingCounterByName` · 006f24d0 · legions 00674180 — performance monitor (admin tools)

### ListNode<PerfMonCounterInfo::PerfMonCounterNameHelp>
- [-] `ListNode<PerfMonCounterInfo::PerfMonCounterNameHelp>::ListNode<PerfMonCounterInfo::PerfMonCounterNameHelp>` · 006f24f0 · legions 006741a0 — performance monitor (admin tools)

### PerfMonCounterInfo
- [-] `PerfMonCounterInfo::GetFirstName` · 006f2540 · legions 006741f0 — performance monitor (admin tools)
- [-] `PerfMonCounterInfo::LookupByLanguage` · 006f2590 — performance monitor (admin tools)
- [-] `PerfMonCounterInfo::AddLanguageNameAndHelp` · 006f2820 · legions 006744d0 — performance monitor (admin tools)
- [-] `PerfMonCounterInfo::PerfMonCounterInfo` · 006f28d0 · legions 00674580 — performance monitor (admin tools)

## `portal/shared/src/perfmon/perfmoncountermanager.cpp.c` (0/13)

### PerfMonCounterManager
- [-] `PerfMonCounterManager::CrashCleanup` · 006f1940 · legions 00673c70 — performance monitor (admin tools)
- [-] `PerfMonCounterManager::StartRpcServer` · 006f1950 — performance monitor (admin tools)

### (free functions)
- [-] `GetNumberPerfCounters` · 006f19c0 · legions 00673c80 — performance monitor (admin tools)

### PerfMonCounterManager
- [-] `PerfMonCounterManager::LookupExistingCounterByName` · 006f1a80 · legions 00673ca0 — performance monitor (admin tools)
- [-] `PerfMonCounterManager::AddCounter` · 006f1b40 · legions 00673d60 — performance monitor (admin tools)
- [-] `PerfMonCounterManager::ShutdownAndResetInternal` · 006f1c10 · legions 00673e30 — performance monitor (admin tools)
- [-] `PerfMonCounterManager::FillInCounterTypes` · 006f1c90 · legions 00673eb0 — performance monitor (admin tools)
- [-] `PerfMonCounterManager::FillInCounterValues` · 006f1cf0 · legions 00673f10 — performance monitor (admin tools)

### (free functions)
- [-] `GetPerfCounterDescriptions` · 006f1d60 · legions 00673f80 — performance monitor (admin tools)
- [-] `GetPerfCounterValues` · 006f1e10 · legions 00674030 — performance monitor (admin tools)

### PerfMonCounterManager
- [-] `PerfMonCounterManager::ShutdownAndReset` · 006f1ec0 · legions 006740e0 — performance monitor (admin tools)
- [-] `PerfMonCounterManager::InstallAndProvideCountersInternal` · 006f1f60 — performance monitor (admin tools)
- [-] `PerfMonCounterManager::InstallAndProvideCounters` · 006f24a0 — performance monitor (admin tools)

## `portal/shared/src/perfmon/perfmondescriptor.cpp.c` (0/8)

### MissileInfo
- [-] `MissileInfo::GetMissileParameters` · 006f3760 — performance monitor (admin tools)

### FXManager
- [-] `FXManager::GetFXTable` · 006f3770 · legions 00646bd0 — performance monitor (admin tools)

### PerfMonDescriptor
- [-] `PerfMonDescriptor::GetPackSize` · 006f3780 — performance monitor (admin tools)
- [-] `PerfMonDescriptor::~PerfMonDescriptor` · 006f37f0 · legions 00675310 — performance monitor (admin tools)
- [-] `PerfMonDescriptor::Set` · 006f3840 · legions 006753f0 — performance monitor (admin tools)
- [-] `PerfMonDescriptor::PerfMonDescriptor` · 006f38d0 · legions 00675480 — performance monitor (admin tools)
- [-] `PerfMonDescriptor::Pack` · 006f3930 — performance monitor (admin tools)
- [-] `PerfMonDescriptor::UnPack` · 006f3990 — performance monitor (admin tools)

## `portal/shared/src/perfmon/perfmonrelay.cpp.c` (0/12)

### PerfMonRelay
- [-] `PerfMonRelay::Shutdown` · 006f2a60 — performance monitor (admin tools)
- [-] `PerfMonRelay::AddCounter` · 006f2b30 — performance monitor (admin tools)
- [-] `PerfMonRelay::IncrementCounter` · 006f2c00 · legions 006747f0 — performance monitor (admin tools)
- [-] `PerfMonRelay::DecrementCounter` · 006f2c40 · legions 00674830 — performance monitor (admin tools)
- [-] `PerfMonRelay::AdjustCounter64` · 006f2c80 · legions 00674870 — performance monitor (admin tools)
- [-] `PerfMonRelay::AdjustCounter32` · 006f2cd0 · legions 006748c0 — performance monitor (admin tools)
- [-] `PerfMonRelay::AdjustCounter16` · 006f2d20 · legions 00674910 — performance monitor (admin tools)
- [-] `PerfMonRelay::AdjustCounter8` · 006f2d80 · legions 00674970 — performance monitor (admin tools)
- [-] `PerfMonRelay::StartTimerCounter` · 006f2de0 · legions 006749d0 — performance monitor (admin tools)
- [-] `PerfMonRelay::StopTimerCounter` · 006f2e50 · legions 00674a40 — performance monitor (admin tools)
- [-] `PerfMonRelay::MeasureLock` · 006f2ec0 · legions 00674ab0 — performance monitor (admin tools)
- [-] `PerfMonRelay::Initialize` · 006f3030 — performance monitor (admin tools)

## `portal/shared/src/perfmon/perfmonvariant.cpp.c` (0/11)

### CWDistributor
- [-] `CWDistributor::GetProcessState` · 006f3a00 · legions 006755b0 — performance monitor (admin tools)

### PerfMonVariant
- [-] `PerfMonVariant::Convert` · 006f3a10 · legions 006755c0 — performance monitor (admin tools)
- [-] `PerfMonVariant::Convert` · 006f3aa0 · legions 00675650 — performance monitor (admin tools)
- [-] `PerfMonVariant::Lock` · 006f3b90 · legions 00675740 — performance monitor (admin tools)
- [-] `PerfMonVariant::Unlock` · 006f3ba0 · legions 00675750 — performance monitor (admin tools)

### CWPhysicsObject
- [-] `CWPhysicsObject::GetPhysObj` · 006f3bb0 · legions 00675760 — performance monitor (admin tools)

### PerfMonVariant
- [-] `PerfMonVariant::IncCounter` · 006f3bc0 — performance monitor (admin tools)
- [-] `PerfMonVariant::DecCounter` · 006f3c10 — performance monitor (admin tools)
- [-] `PerfMonVariant::AdjustCounter` · 006f3c60 · legions 00675770 — performance monitor (admin tools)
- [-] `PerfMonVariant::SetCounter` · 006f3d50 · legions 00675860 — performance monitor (admin tools)
- [-] `PerfMonVariant::PerfMonVariant` · 006f3de0 — performance monitor (admin tools)

## `portal/shared/src/perfmon/performer.cpp.c` (0/9)

### Performer
- [-] `Performer::ListCounters` · 006f30c0 — performance monitor (admin tools)
- [-] `Performer::StreamPack` · 006f30d0 · legions 00674c30 — performance monitor (admin tools)
- [-] `Performer::LoadFromFile` · 006f3180 — performance monitor (admin tools)
- [-] `Performer::Performer` · 006f32e0 · legions 00674e30 — performance monitor (admin tools)
- [-] `Performer::GetDBOType` · 006f3310 — performance monitor (admin tools)
- [-] `Performer::Destroy` · 006f3320 — performance monitor (admin tools)

### PSmartArray<PerfMonDescriptor>
- [-] `PSmartArray<PerfMonDescriptor>::StreamPack` · 006f3330 · legions 00674e80 — performance monitor (admin tools)

### Performer
- [-] `Performer::Allocate` · 006f33d0 — performance monitor (admin tools)
- [-] `Performer::LoadFromEMP` · 006f3440 — performance monitor (admin tools)

## `portal/shared/src/perfmon/rpc/perfmonrpc_s.c.c` (0/6)

### (free functions)
- [-] `turbineperfmonrpcserverapi_GetNumberPerfCounters` · 006f3e90 · legions 006758f0 — performance monitor (admin tools)
- [-] `FUN_006f3ffc` · 006f3ffc · legions 00675a5c — performance monitor (admin tools)
- [-] `turbineperfmonrpcserverapi_GetPerfCounterDescriptions` · 006f4000 · legions 00675a60 — performance monitor (admin tools)
- [-] `FUN_006f41c9` · 006f41c9 · legions 00675c29 — performance monitor (admin tools)
- [-] `turbineperfmonrpcserverapi_GetPerfCounterValues` · 006f41e0 · legions 00675c40 — performance monitor (admin tools)
- [-] `FUN_006f43a9` · 006f43a9 · legions 00675e09 — performance monitor (admin tools)

## `portal/shared/src/pparser/newparser.cpp.c` (0/18)

### InputStream
- [-] `InputStream::Next` · 0066aa40 · legions 00615470 — text source files (admin tools)
- [-] `InputStream::SetFile` · 0066aae0 · legions 00615510 — text source files (admin tools)
- [-] `InputStream::InputStream` · 0066ac30 · legions 00615660 — text source files (admin tools)
- [-] `InputStream::~InputStream` · 0066ac60 · legions 00615690 — text source files (admin tools)
- [-] `InputStream::SetString` · 0066acb0 · legions 006156e0 — text source files (admin tools)

### NewParser
- [-] `NewParser::Init` · 0066ad40 · legions 00615770 — text source files (admin tools)
- [-] `NewParser::Cleanup` · 0066add0 · legions 00615800 — text source files (admin tools)
- [-] `NewParser::Error` · 0066ae90 — text source files (admin tools)
- [-] `NewParser::Warning` · 0066af00 — text source files (admin tools)
- [-] `NewParser::ParseComment` · 0066af60 · legions 006158c0 — text source files (admin tools)
- [-] `NewParser::ParseNodeName` · 0066b090 · legions 006159f0 — text source files (admin tools)
- [-] `NewParser::ParseNode` · 0066b360 · legions 00615ca0 — text source files (admin tools)
- [-] `NewParser::FindIncludeFiles` · 0066b5b0 · legions 00615ee0 — text source files (admin tools)
- [-] `NewParser::ParseNodes` · 0066b7a0 · legions 006160d0 — text source files (admin tools)
- [-] `NewParser::Parse` · 0066b930 · legions 00616260 — text source files (admin tools)
- [-] `NewParser::ParseFile` · 0066ba30 · legions 00616360 — text source files (admin tools)
- [-] `NewParser::ParseString` · 0066ba80 · legions 006163b0 — text source files (admin tools)
- [-] `NewParser::PostProcessNode` · 0066bac0 · legions 006163f0 — text source files (admin tools)

## `portal/shared/src/pparser/pfileparser.cpp.c` (0/23)

### PFileParser
- [-] `PFileParser::LoadFromFile` · 00669b30 · legions 00614570 — text source files (admin tools)
- [-] `PFileParser::LoadFromString` · 00669b60 — text source files (admin tools)
- [-] `PFileParser::FormatSubNodes` · 00669b70 · legions 006145a0 — text source files (admin tools)
- [-] `PFileParser::ReleaseFileLock` · 00669bf0 · legions 00614620 — text source files (admin tools)

### EffectUINode
- [-] `EffectUINode::SetDescription` · 00669c30 · legions 00614660 — text source files (admin tools)

### OutputStream
- [-] `OutputStream::SetString` · 00669c80 · legions 006146b0 — text source files (admin tools)

### PFileParser
- [-] `PFileParser::LoadFromStringW` · 00669ce0 · legions 00614710 — text source files (admin tools)
- [-] `PFileParser::CheckWrapTree` · 00669d30 · legions 00614760 — text source files (admin tools)
- [-] `PFileParser::ShouldSeparate` · 00669e00 · legions 00614830 — text source files (admin tools)
- [-] `PFileParser::MaybeIndent` · 00669e80 · legions 006148b0 — text source files (admin tools)
- [-] `PFileParser::PutQuotedString` · 00669ef0 · legions 00614920 — text source files (admin tools)
- [-] `PFileParser::NewLine` · 0066a1e0 · legions 00614c10 — text source files (admin tools)
- [-] `PFileParser::InsertSeparator` · 0066a220 · legions 00614c50 — text source files (admin tools)
- [-] `PFileParser::InsertComments` · 0066a290 · legions 00614cc0 — text source files (admin tools)
- [-] `PFileParser::FormatNode` · 0066a450 · legions 00614e80 — text source files (admin tools)

### NewParser
- [-] `NewParser::NewParser` · 0066a5d0 · legions 00615000 — text source files (admin tools)
- [-] `NewParser::~NewParser` · 0066a630 · legions 00615060 — text source files (admin tools)

### PFileParser
- [-] `PFileParser::PFileParser` · 0066a690 · legions 006150c0 — text source files (admin tools)
- [-] `PFileParser::SaveToFile` · 0066a6e0 · legions 00615110 — text source files (admin tools)
- [-] `PFileParser::SaveToLockedFile` · 0066a800 · legions 00615230 — text source files (admin tools)
- [-] `PFileParser::SaveToString` · 0066a8c0 · legions 006152f0 — text source files (admin tools)
- [-] `PFileParser::~PFileParser` · 0066a920 · legions 00615350 — text source files (admin tools)
- [-] `PFileParser::SaveToStringW` · 0066a960 · legions 00615390 — text source files (admin tools)

## `portal/shared/src/random/random.cpp.c` (3/5)

### Random
- [x] `Random::rand` · 0060d2d0 · legions 005ce0a0 — ac2_core::Random
- [x] `Random::RollDice` · 0060d3b0 · legions 005ce180 — ac2_core::Random
- [x] `Random::RollDice` · 0060d3f0 · legions 005ce1c0 — ac2_core::Random
- [ ] `Random::RollGaussianDice` · 0060d450 · legions 005ce220
- [ ] `Random::RollPercentile` · 0060d520 · legions 005ce2f0

## `portal/shared/src/reg_edit/reg_edit.cpp.c` (0/8)

### RegEdit
- [-] `RegEdit::CreateKey` · 00662ba0 — registry editing (admin tools)
- [-] `RegEdit::SetStrValue` · 00662c30 — registry editing (admin tools)
- [-] `RegEdit::QueryStrValue` · 00662cc0 · legions 00610f50 — registry editing (admin tools)
- [-] `RegEdit::SetIntValue` · 00662d40 — registry editing (admin tools)
- [-] `RegEdit::QueryIntValue` · 00662dc0 · legions 00610fd0 — registry editing (admin tools)
- [-] `RegEdit::SetBinValue` · 00662e50 — registry editing (admin tools)
- [-] `RegEdit::QueryBinValue` · 00662ed0 · legions 00611060 — registry editing (admin tools)
- [-] `RegEdit::DeleteValue` · 00662f50 · legions 006110e0 — registry editing (admin tools)

## `portal/shared/src/setproject/projectsettings.cpp.c` (0/7)

### ProjectSettings
- [-] `ProjectSettings::Set` · 0066c070 · legions 00616970 — project setup (admin tools)
- [-] `ProjectSettings::QueryRegistryForCurrentProject` · 0066c0d0 · legions 006169d0 — project setup (admin tools)
- [-] `ProjectSettings::QueryIniForProject` · 0066c120 · legions 00616a20 — project setup (admin tools)
- [-] `ProjectSettings::InitSettingFromIni` · 0066c170 · legions 00616a70 — project setup (admin tools)
- [-] `ProjectSettings::QueryRegistryForCurrentIni` · 0066c250 · legions 00616b50 — project setup (admin tools)
- [-] `ProjectSettings::InitFromIni` · 0066c300 · legions 00616c00 — project setup (admin tools)
- [-] `ProjectSettings::InitFromRegistry` · 0066c360 · legions 00616c40 — project setup (admin tools)

## `portal/shared/src/stringtoken/stringtoken.cpp.c` (2/4)

### (free functions)
- [x] `Tokenize` · 0065a4a0 · legions 00608e10 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [ ] `Tokenize` · 0065a510 · legions 00608e80
- [ ] `Tokenize` · 0065a580 · legions 00608ef0
- [x] `Tokenize` · 0065a5a0 · legions 00608f10

## `portal/shared/src/timestamper/tsrecv.cpp.c` (5/7)

### TSRecv
- [x] `TSRecv::TSRecv` · 0065b1e0 · legions 00609af0
- [x] `TSRecv::GetNextReadyEntry` · 0065b210 · legions 00609b20
- [ ] `TSRecv::~TSRecv` · 0065b2d0 · legions 00609be0
- [ ] `TSRecv::AddEntryLatest` · 0065b340 · legions 00609c50
- [x] `TSRecv::AddEntry` · 0065b3c0 · legions 00609cd0
- [x] `TSRecv::AddEntryBlocking` · 0065b480 · legions 00609d90
- [x] `TSRecv::AddAndCheck` · 0065b5b0 · legions 00609ec0

## `portal/shared/src/utils/argparser.cpp.c` (0/13)

### ArgumentParser
- [ ] `ArgumentParser::AppendOutputText` · 0065b5e0 · legions 00609ef0

### (free functions)
- [ ] `SortCommandLineArgs` · 0065b610

### CommandLineArg
- [ ] `CommandLineArg::operator=` · 0065b6c0 · legions 00609fd0
- [ ] `CommandLineArg::CommandLineArg` · 0065b760 · legions 0060a070

### ArgumentParser
- [ ] `ArgumentParser::EvaluateCommandLineArg` · 0065b7a0 · legions 0060a0b0
- [ ] `ArgumentParser::IsCommandLineArgument` · 0065b9b0 · legions 0060a2c0
- [ ] `ArgumentParser::DisplayUsage` · 0065bce0 · legions 0060a610
- [ ] `ArgumentParser::AppendAndWordWrap` · 0065bf50 · legions 0060a880

### (free functions)
- [ ] `ConvertCommandLineToArgList` · 0065c0e0 · legions 0060aa10

### ArgumentParser
- [ ] `ArgumentParser::ParseCommandLine` · 0065c1a0 · legions 0060aad0

### ArgumentParser::CommandLineArgList
- [ ] `ArgumentParser::CommandLineArgList::add` · 0065c520 · legions 0060ae50
- [ ] `ArgumentParser::CommandLineArgList::~CommandLineArgList` · 0065c640 · legions 0060af70

### ArgumentParser
- [ ] `ArgumentParser::ParseArgs` · 0065c6c0 · legions 0060aff0

## `portal/shared/src/utils/checksum.cpp.c` (1/1)

### PortalChecksum
- [x] `PortalChecksum::CalcChecksum32` · 0065f3d0 · legions 0060dce0

## `portal/shared/src/utils/emailutils.cpp.c` (0/2)

### EmailUtils
- [ ] `EmailUtils::send_email` · 0065f440 · legions 0060dd50
- [ ] `EmailUtils::send_emailW` · 0065f5e0 · legions 0060def0

## `portal/shared/src/utils/mmaputil.cpp.c` (0/3)

### MMapUtil
- [ ] `MMapUtil::MMapUtil` · 00737f60 · legions 006b3b20
- [ ] `MMapUtil::~MMapUtil` · 00737f90 · legions 006b3b50
- [ ] `MMapUtil::MMap` · 00738010 · legions 006b3bd0

## `portal/shared/src/utils/psutils.cpp.c` (2/40)

### (free functions)
- [x] `search_skipescapes` · 0065ccc0 · legions 0060b5f0 — ac2-db string_meta_text / string_state (CRT C locale classes)

### PSUtils
- [ ] `PSUtils::check_access` · 0065cd10 · legions 0060b640
- [x] `PSUtils::wcschr_skipescapes` · 0065cd30 · legions 0060b660 — ac2-db string_meta_text / string_state (CRT C locale classes)

### PStringBaseIter_Common<char>
- [ ] `PStringBaseIter_Common<char>::FindChar` · 0065cdf0 · legions 0060b720

### (free functions)
- [ ] `wildcard_match_iter` · 0065ce90 · legions 0060b7c0

### PSUtils
- [ ] `PSUtils::get_binary_buffer_from_string` · 0065d010 · legions 0060b940

### PStringBaseIter_Common<char>
- [ ] `PStringBaseIter_Common<char>::FindChar` · 0065d150 · legions 0060ba80
- [ ] `PStringBaseIter_Common<char>::GetSubString` · 0065d2b0 · legions 0060bbe0
- [ ] `PStringBaseIter_Common<char>::GetSubString` · 0065d330 · legions 0060bc60

### PSUtils
- [ ] `PSUtils::explode` · 0065d420 · legions 0060bd50
- [ ] `PSUtils::get_process_name` · 0065d560 · legions 0060be90
- [ ] `PSUtils::change_extension` · 0065d5a0 · legions 0060bed0
- [ ] `PSUtils::remove_extension` · 0065d690 · legions 0060bfc0
- [ ] `PSUtils::get_directory` · 0065d6f0 · legions 0060c020
- [ ] `PSUtils::get_extension` · 0065d800 · legions 0060c130
- [ ] `PSUtils::GetEnvironmentVariableA` · 0065d890 · legions 0060c1c0
- [ ] `PSUtils::GetEnvironmentPath` · 0065d900 · legions 0060c230
- [ ] `PSUtils::stringify_buffer` · 0065d9f0 · legions 0060c320
- [ ] `PSUtils::widify` · 0065da70 · legions 0060c3a0
- [ ] `PSUtils::widify` · 0065dae0 · legions 0060c410
- [ ] `PSUtils::strip_directory` · 0065e0b0 · legions 0060c9e0
- [ ] `PSUtils::change_directory` · 0065e180 · legions 0060cab0
- [ ] `PSUtils::GetEnvironmentPath` · 0065e350 · legions 0060cc80
- [ ] `PSUtils::linewrap_string` · 0065e4b0 · legions 0060cde0
- [ ] `PSUtils::MakeCFriendlySymbol` · 0065e5a0 · legions 0060ced0
- [ ] `PSUtils::find_files` · 0065e750 · legions 0060d080
- [ ] `PSUtils::narrowify` · 0065ea40 · legions 0060d370
- [ ] `PSUtils::narrowify` · 0065eab0 · legions 0060d3e0
- [ ] `PSUtils::make_log_string` · 0065eb10 · legions 0060d440
- [ ] `PSUtils::remove_extensionW` · 0065eba0 · legions 0060d4d0
- [ ] `PSUtils::strip_directoryW` · 0065ec50 · legions 0060d580
- [ ] `PSUtils::get_cwd` · 0065ed00 · legions 0060d630
- [ ] `PSUtils::get_cwdW` · 0065edc0 · legions 0060d6f0
- [ ] `PSUtils::get_cwd` · 0065ee80 · legions 0060d7b0
- [ ] `PSUtils::cleanup_filename` · 0065ef20 · legions 0060d850
- [ ] `PSUtils::cleanup_directory_name` · 0065efc0 · legions 0060d8f0
- [ ] `PSUtils::cleanup_directory_nameW` · 0065f020 · legions 0060d950
- [ ] `PSUtils::unwrap_string` · 0065f0d0 · legions 0060da00
- [ ] `PSUtils::full_path` · 0065f120 · legions 0060da50
- [ ] `PSUtils::create_directory` · 0065f180 · legions 0060dab0

## `portal/shared/src/utils/scratchpad.cpp.c` (0/3)

### (free functions)
- [ ] `GetScratchBuffer` · 0065cbf0 · legions 0060b520
- [ ] `ReleaseScratchBuffer` · 0065cc70 · legions 0060b5a0
- [ ] `GetScratchBuffer` · 0065cca0 · legions 0060b5d0

## `portal/shared/src/version/resource.cpp.c` (0/5)

### Resource
- [ ] `Resource::LoadData` · 0056a2b0 · legions 0053a240
- [ ] `Resource::Init` · 0056a330 · legions 0053a2c0
- [ ] `Resource::Cleanup` · 0056a4c0 · legions 0053a450
- [ ] `Resource::GetComments` · 0056a4e0 · legions 0053a470
- [ ] `Resource::GetVersion` · 0056a550 · legions 0053a4e0

## `portal/shared/src/version/version.cpp.c` (0/7)

### Version
- [ ] `Version::StringFromVersion` · 00569fe0 · legions 00539f70
- [ ] `Version::FileVersionStringFromVersion` · 0056a030 · legions 00539fc0
- [ ] `Version::Cleanup` · 0056a080 · legions 0053a010

### PerfMonCounter<unsigned___int64>
- [ ] `PerfMonCounter<unsigned___int64>::IncCounter` · 0056a0f0
- [ ] `PerfMonCounter<unsigned___int64>::DecCounter` · 0056a100
- [ ] `PerfMonCounter<unsigned___int64>::GetValueAsInt64` · 0056a110

### Version
- [ ] `Version::Init` · 0056a120 · legions 0053a0b0
