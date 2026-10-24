# ac2-dat: DAT files and the database layer

507 retail functions: 6 done, 3 partial, 493 to do, 5 skipped.
Legend and workflow: [README](README.md).

## `portal/shared/src/database/asynccache.cpp.c` (0/48)

### AsyncCache::CAsyncRequest
- [ ] `AsyncCache::CAsyncRequest::AdoptDBObj` · 006d0900 · legions 0065a650
- [ ] `AsyncCache::CAsyncRequest::ReleaseDBObj` · 006d0920 · legions 0065a670

### AsyncCache
- [ ] `AsyncCache::EnqueueAsyncRequest` · 006d0940 · legions 0065a690
- [ ] `AsyncCache::OnRequestFinished` · 006d0a00 · legions 0065a750
- [ ] `AsyncCache::OnPurgeRequestFinished` · 006d0a30 · legions 0065a780
- [ ] `AsyncCache::AddObjToDBOCache` · 006d0a40 · legions 0065a790

### DBCache
- [ ] `DBCache::GetIfInMemory` · 006d0a60 · legions 0065a7b0

### AsyncCache
- [ ] `AsyncCache::GetFreeObj` · 006d0a90 · legions 0065a7e0
- [ ] `AsyncCache::IsOnDisk` · 006d0ac0 · legions 0065a810
- [ ] `AsyncCache::LoadData` · 006d0b00 · legions 0065a850
- [ ] `AsyncCache::PurgeData` · 006d0b40 · legions 0065a890
- [ ] `AsyncCache::SaveData` · 006d0b80 · legions 0065a8d0

### CPhysicsObj
- [ ] `CPhysicsObj::RecvEvt_SetMode` · 006d0bc0 · legions 0065a910

### CObjCell
- [ ] `CObjCell::GetCID` · 006d0bd0 · legions 0065a920

### AsyncCache
- [ ] `AsyncCache::BlockingGetFromDisk` · 006d0be0 · legions 0065a930

### NIList<AsyncCache::CCallbackHandler_*>
- [ ] `NIList<AsyncCache::CCallbackHandler_*>::RemoveFromHead` · 006d0c50 · legions 00679d20

### AsyncCache::CAsyncRequest
- [ ] `AsyncCache::CAsyncRequest::~CAsyncRequest` · 006d0c80 · legions 0065a9a0
- [ ] `AsyncCache::CAsyncRequest::RemoveAllCallbacks` · 006d0cb0 · legions 0065a9d0

### AsyncCache::CAsyncGetRequest
- [ ] `AsyncCache::CAsyncGetRequest::ReleaseDBObj` · 006d0d00 · legions 0065aa20
- [ ] `AsyncCache::CAsyncGetRequest::~CAsyncGetRequest` · 006d0d50 · legions 0065aa70
- [ ] `AsyncCache::CAsyncGetRequest::bAllDependanciesDone` · 006d0e20 · legions 0065ab40

### AsyncCache
- [ ] `AsyncCache::OnAsyncGetFinished` · 006d0e30 · legions 0065ab50
- [ ] `AsyncCache::BlockingLoadInto` · 006d0ec0 · legions 0065abe0

### AsyncCache::CAsyncGetRequest
- [ ] `AsyncCache::CAsyncGetRequest::CAsyncGetRequest` · 006d0f40 · legions 0065ac60

### AsyncCache::CAsyncRequest
- [ ] `AsyncCache::CAsyncRequest::RemoveCallback` · 006d0fb0 · legions 0065acd0

### AsyncCache::CCallbackHandler
- [ ] `AsyncCache::CCallbackHandler::OnContextReleased` · 006d1020 · legions 0065ad40
- [ ] `AsyncCache::CCallbackHandler::~CCallbackHandler` · 006d1090 · legions 0065adb0

### AsyncCache
- [ ] `AsyncCache::UseTime` · 006d1140 · legions 0065ae60
- [ ] `AsyncCache::AreOnDisk` · 006d11d0 · legions 0065aef0
- [ ] `AsyncCache::FlushPendingRequests` · 006d1470 · legions 0065b140

### AsyncCache::CAsyncRequest
- [ ] `AsyncCache::CAsyncRequest::AddCallback` · 006d15e0 · legions 0065b2b0

### AsyncCache
- [ ] `AsyncCache::FormGetRequest` · 006d1630 · legions 0065b300
- [ ] `AsyncCache::OnSaveRequestFinished` · 006d16d0 · legions 0065b3a0
- [ ] `AsyncCache::UnhashPendingGet` · 006d1770 · legions 0065b440
- [ ] `AsyncCache::ReleaseContext` · 006d1830 · legions 0065b500
- [ ] `AsyncCache::NotifyCallback` · 006d18b0 · legions 0065b580
- [ ] `AsyncCache::BlockingGet` · 006d19d0 · legions 0065b6a0
- [ ] `AsyncCache::OnAsyncGetFromOtherSourcesFailed` · 006d1ab0 · legions 0065b780

### AsyncCache::CCallbackHandler
- [ ] `AsyncCache::CCallbackHandler::AddTopLevelRequest` · 006d1af0 · legions 0065b7c0

### AsyncCache
- [ ] `AsyncCache::AddToAsyncGet` · 006d1b50 · legions 0065b820
- [ ] `AsyncCache::AllocateCallback` · 006d1d20 · legions 0065ba40
- [ ] `AsyncCache::AsyncGet` · 006d1de0 · legions 0065bb00
- [ ] `AsyncCache::AsyncGet` · 006d1e90 · legions 0065bbb0
- [ ] `AsyncCache::AsyncPurge` · 006d2030 · legions 0065bd50
- [ ] `AsyncCache::AsyncPurge` · 006d2120 · legions 0065be40
- [ ] `AsyncCache::AsyncSave` · 006d22b0 · legions 0065bfd0
- [ ] `AsyncCache::HashAndEnqueue` · 006d23a0 · legions 0065c0c0
- [ ] `AsyncCache::OnGetRequestFinished` · 006d2410 · legions 0065c130

## `portal/shared/src/database/clidatevents.cpp.c` (5/12)

### CCliDatEvent
- [x] `CCliDatEvent::CCliDatEvent` · 006da360 · legions 006637a0

### CCliDatDataEvent
- [ ] `CCliDatDataEvent::CreateCachePack` · 006da3d0

### CCliDatInterrogationResponseEvent
- [~] `CCliDatInterrogationResponseEvent::CreateData` · 006da480 — layout only; the client sends no DAT iteration lists

### CCliDatBeginDDDEvent
- [x] `CCliDatBeginDDDEvent::GetMissingIterations` · 006da500 · legions 00663940

### CCliDatRequestEvent
- [x] `CCliDatRequestEvent::CreateForSend` · 006da550 · legions 00663990

### CAllIterationList::PTaggedIterationList
- [ ] `CAllIterationList::PTaggedIterationList::StreamPack` · 006da5b0 · legions 006639f0

### CCliDatInterrogationResponseEvent
- [~] `CCliDatInterrogationResponseEvent::CreateForSend` · 006da600 — layout only; the client sends no DAT iteration lists

### CCliDatEndDDDEvent
- [x] `CCliDatEndDDDEvent::CreateForSend` · 006da680 · legions 00663ac0

### (free functions)
- [ ] `StreamIDs` · 006da6d0 · legions 00663b10

### CMissingIteration
- [x] `CMissingIteration::StreamPack` · 006da7b0

### SmartArray<CAllIterationList::PTaggedIterationList>
- [ ] `SmartArray<CAllIterationList::PTaggedIterationList>::AddToEnd` · 006da840 · legions 00663c80

### CAllIterationList
- [ ] `CAllIterationList::AddIterationList` · 006da8b0

## `portal/shared/src/database/datfiledata.cpp.c` (0/8)

### DatFileData
- [ ] `DatFileData::GetDBOType` · 006d9930
- [ ] `DatFileData::Pack` · 006d9940

### IntMap
- [ ] `IntMap::Pack` · 006d9960 · legions 00662ce0

### DatFileData
- [ ] `DatFileData::DatFileData` · 006d99c0 · legions 00662d40
- [ ] `DatFileData::Allocate` · 006d9a20
- [ ] `DatFileData::UnPack` · 006d9a30

### IntMap
- [ ] `IntMap::UnPack` · 006d9a50 · legions 00662dd0

### GrooveInfo
- [ ] `GrooveInfo::GrooveInfo` · 006d9b30 · legions 00662f60

## `portal/shared/src/database/dbcache.cpp.c` (0/30)

### DBCache
- [ ] `DBCache::ResetCache` · 006cfa10 · legions 00659790
- [ ] `DBCache::SetMasterMapDID` · 006cfa20 · legions 006597a0
- [ ] `DBCache::Get` · 006cfa30 · legions 006597b0
- [ ] `DBCache::GetDIDFromEnumStatic` · 006cfa60 · legions 006597e0
- [ ] `DBCache::GetDIDFromEnum` · 006cfa80 · legions 00659800
- [ ] `DBCache::GetFromEnum` · 006cfb60 · legions 006598b0
- [ ] `DBCache::GetCurrentRegion` · 006cfbb0 · legions 00659900
- [ ] `DBCache::IsRunTime` · 006cfbd0 · legions 00659920
- [ ] `DBCache::IsClient` · 006cfbf0 · legions 00659940
- [ ] `DBCache::IsServer` · 006cfc10 · legions 00659960
- [ ] `DBCache::SetRegion` · 006cfc30 · legions 00659980
- [ ] `DBCache::ReloadObject` · 006cfc40 · legions 00659990
- [ ] `DBCache::AddToDataGraph` · 006cfcf0 · legions 00659a40
- [ ] `DBCache::SetLocalLanguage` · 006cfdc0 · legions 00659b10
- [ ] `DBCache::GetLocalLanguage` · 006cfde0
- [ ] `DBCache::Init` · 006cfec0 · legions 00659c20
- [ ] `DBCache::UseTime` · 006cfff0 · legions 00659d40
- [ ] `DBCache::UnloadCellData` · 006d0050 · legions 00659da0
- [ ] `DBCache::AskForLastWords` · 006d00e0 · legions 00659e30
- [ ] `DBCache::FlushFreeObjects` · 006d0140 · legions 00659ee0
- [ ] `DBCache::GetDBOCache` · 006d01f0 · legions 00659f40
- [ ] `DBCache::DestroyObjectCaches` · 006d0230 · legions 00659f80
- [ ] `DBCache::ReloadCache` · 006d02b0 · legions 0065a000

### AsyncCache
- [ ] `AsyncCache::~AsyncCache` · 006d0440 · legions 0065a0f0

### DBCache
- [ ] `DBCache::~DBCache` · 006d04d0 · legions 0065a180
- [ ] `DBCache::KeepFreeObjects` · 006d0500 · legions 0065a1b0
- [ ] `DBCache::GetIfInMemory` · 006d05a0 · legions 0065a250
- [ ] `DBCache::GetDBOCache` · 006d05e0 · legions 0065a290

### AsyncCache
- [ ] `AsyncCache::AsyncCache` · 006d0820 · legions 0065a570

### DBCache
- [ ] `DBCache::DBCache` · 006d08a0 · legions 0065a5f0

## `portal/shared/src/database/dbcategories.cpp.c` (0/14)

### DBCategories
- [ ] `DBCategories::FillDataGraph` · 006d8bc0

### ParseHelper<EmpGenParse>::ParseHandler
- [ ] `ParseHelper<EmpGenParse>::ParseHandler::ParseHandler` · 006d8c30

### DBCategories
- [ ] `DBCategories::SaveToFile` · 006d8c60
- [ ] `DBCategories::Destroy` · 006d8fd0

### HashTable<CaseInsensitiveStringBase<PStringBase<char>_>,unsigned_long>
- [ ] `HashTable<CaseInsensitiveStringBase<PStringBase<char>_>,unsigned_long>::operator=` · 006d8fe0 · legions 006625e0

### DBCategories
- [ ] `DBCategories::ParseEMPsNode` · 006d9130

### PHashTable<CaseInsensitiveStringBase<PStringBase<char>_>,unsigned_long>
- [ ] `PHashTable<CaseInsensitiveStringBase<PStringBase<char>_>,unsigned_long>::StreamPack` · 006d9480 · legions 00662740

### IntrusiveHashData<CaseInsensitiveStringBase<PStringBase<char>_>,HashTableData<CaseInsensitiveStringBase<PStringBase<char>_>,unsigned_long>_*>
- [ ] `IntrusiveHashData<CaseInsensitiveStringBase<PStringBase<char>_>,HashTableData<CaseInsensitiveStringBase<PStringBase<char>_>,unsigned_long>_*>::IntrusiveHashData<CaseInsensitiveStringBase<PStringBase<char>_>,HashTableData<CaseInsensitiveStringBase<PStringBase<char>_>,unsigned_long>_*>` · 006d96e0 · legions 00662ab0

### DBCategories
- [ ] `DBCategories::DBCategories` · 006d9760 · legions 00662bb0
- [ ] `DBCategories::GetDBOType` · 006d97a0
- [ ] `DBCategories::~DBCategories` · 006d97b0 · legions 00662c00
- [ ] `DBCategories::CopyInto` · 006d9810
- [-] `DBCategories::FromFileNode` · 006d9830 — text source format (admin tools)
- [ ] `DBCategories::Allocate` · 006d9900

## `portal/shared/src/database/dbobj.cpp.c` (0/21)

### CWBookKeeper
- [ ] `CWBookKeeper::SetBookie` · 006d7690 · legions 006613b0

### DBObj
- [ ] `DBObj::AddToDataGraph` · 006d76a0 · legions 006613c0
- [ ] `DBObj::AddRef` · 006d76b0 · legions 006613d0
- [ ] `DBObj::Release` · 006d76e0
- [ ] `DBObj::ReloadFromDisk` · 006d7710 · legions 00661430
- [ ] `DBObj::SaveToDisk` · 006d7740 · legions 00661460
- [ ] `DBObj::GetPackSize` · 006d7780 · legions 006614a0
- [ ] `DBObj::Pack_Type` · 006d77b0 · legions 006614d0
- [~] `DBObj::UnPack_Type` · 006d7830 · legions 00661550 — unpack only, in ac2-net protocol::qualities / visual (Legions CBaseQualities::UnPack instruction-identical)
- [ ] `DBObj::Get` · 006d7890 · legions 006615b0
- [ ] `DBObj::GetDBOCache` · 006d78b0 · legions 006615d0

### FileArray
- [ ] `FileArray::GetDIDFromEnum` · 006d78c0 · legions 00616cd0

### DBObj
- [ ] `DBObj::Copy` · 006d78d0 · legions 006615e0
- [ ] `DBObj::Put` · 006d7910 · legions 00661620
- [ ] `DBObj::Remove` · 006d7940 · legions 00661650
- [ ] `DBObj::GetByEnum` · 006d7970 · legions 00661680
- [ ] `DBObj::DBObj` · 006d79a0 · legions 006616b0
- [ ] `DBObj::~DBObj` · 006d7a10 · legions 00661720
- [ ] `DBObj::SaveToFile` · 006d7a60
- [-] `DBObj::LoadFromFile` · 006d7b20 — text source loader, admin build only
- [ ] `DBObj::FillDataGraph` · 006d7bf0

## `portal/shared/src/database/dbocache.cpp.c` (0/21)

### DBOCache
- [ ] `DBOCache::UseTime` · 006d9b40 · legions 00662f70
- [ ] `DBOCache::AddObj` · 006d9b80
- [ ] `DBOCache::FreeObject` · 006d9ba0
- [ ] `DBOCache::ReloadObject` · 006d9be0
- [ ] `DBOCache::FreelistAdd` · 006d9c00
- [ ] `DBOCache::FreelistRemove` · 006d9c70
- [ ] `DBOCache::FreelistRemoveOldest` · 006d9cc0
- [ ] `DBOCache::GetFreeObj` · 006d9ce0 · legions 00663120

### SetPositionStruct
- [ ] `SetPositionStruct::SetFlags` · 006d9d20 · legions 00663160

### DBOCache
- [ ] `DBOCache::FlushFreeObjects` · 006d9d30 · legions 00663170
- [ ] `DBOCache::KeepFreeObjects` · 006d9d90 · legions 006631d0
- [ ] `DBOCache::ReloadCache` · 006d9dc0 · legions 00663200
- [ ] `DBOCache::RemoveObj_Internal` · 006d9e70
- [ ] `DBOCache::DestroyObj` · 006d9eb0
- [ ] `DBOCache::RemoveObj` · 006d9f00 · legions 00663340
- [ ] `DBOCache::GetIfInMemory` · 006d9f70 · legions 006633b0
- [ ] `DBOCache::GetIfUsing` · 006da000 · legions 00663440
- [ ] `DBOCache::Release` · 006da060
- [ ] `DBOCache::~DBOCache` · 006da170 · legions 006635b0
- [ ] `DBOCache::DBOCache` · 006da240 · legions 00663680
- [ ] `DBOCache::AddObj_Internal` · 006da2e0

## `portal/shared/src/database/enuidmap.cpp.c` (0/15)

### EnumIDMap
- [ ] `EnumIDMap::StreamPack` · 006d80b0 · legions 00661c30
- [ ] `EnumIDMap::NameToEnum` · 006d8150 · legions 00661cd0
- [ ] `EnumIDMap::GetSubDataIDs` · 006d8280 · legions 00661e00

### SmartArray<PEnum>
- [ ] `SmartArray<PEnum>::operator=` · 006d8380 · legions 00661f00

### EnumIDMap
- [ ] `EnumIDMap::EnumToDID` · 006d83f0 · legions 00661f70
- [ ] `EnumIDMap::GetName` · 006d8450 · legions 00661fd0
- [ ] `EnumIDMap::Destroy` · 006d8490 · legions 00662010
- [ ] `EnumIDMap::GetAllEnumVals` · 006d84c0 · legions 00662040
- [ ] `EnumIDMap::EnumIDMap` · 006d85f0 · legions 00662170
- [ ] `EnumIDMap::GetDBOType` · 006d86b0
- [ ] `EnumIDMap::~EnumIDMap` · 006d86c0 · legions 00662240
- [ ] `EnumIDMap::GetAllEnumNames` · 006d8800 · legions 00662380
- [ ] `EnumIDMap::AddEmp` · 006d8920
- [ ] `EnumIDMap::Allocate` · 006d8b20
- [-] `EnumIDMap::LoadFromFile` · 006d8b50 — text source loader, admin build only

## `portal/shared/src/database/masterdbmap.cpp.c` (0/4)

### MasterDBMap
- [ ] `MasterDBMap::DivineType_Internal` · 006d2530 · legions 0065c250

### DBTypeDef
- [ ] `DBTypeDef::DBTypeDef` · 006d2b70 · legions 0065c890

### MasterDBMap
- [ ] `MasterDBMap::DivineType_Internal` · 006d2c10 · legions 0065c930
- [ ] `MasterDBMap::InitDBTypeDef_Internal` · 006d4640 · legions 0065e360

## `portal/shared/src/database/masterdbmapmain.cpp.c` (0/19)

### MasterDBMap
- [ ] `MasterDBMap::Init` · 006d6f80 · legions 00660ca0
- [ ] `MasterDBMap::Cleanup` · 006d6fa0 · legions 00660cc0
- [ ] `MasterDBMap::DivineCellType` · 006d6fc0 · legions 00660ce0
- [ ] `MasterDBMap::DivineType` · 006d7020 · legions 00660d40
- [ ] `MasterDBMap::DivineType` · 006d7040 · legions 00660d60
- [ ] `MasterDBMap::SetGameDataDir` · 006d70a0 · legions 00660dc0
- [ ] `MasterDBMap::SetEngineDataDir` · 006d70e0 · legions 00660e00
- [ ] `MasterDBMap::GetDBTypeDef` · 006d71e0 · legions 00660f00
- [ ] `MasterDBMap::HasFile` · 006d7260 · legions 00660f80
- [ ] `MasterDBMap::GetBaseDID` · 006d7280 · legions 00660fa0
- [ ] `MasterDBMap::GetPackVer` · 006d7290
- [ ] `MasterDBMap::GetGameDataDir` · 006d72b0 · legions 00660fd0
- [ ] `MasterDBMap::GetEngineDataDir` · 006d72f0 · legions 00661010
- [ ] `MasterDBMap::IsPortalType` · 006d7330 · legions 00661050
- [ ] `MasterDBMap::IsCellType` · 006d7350 · legions 00661070
- [ ] `MasterDBMap::IsLocalType` · 006d7370 · legions 00661090
- [ ] `MasterDBMap::IsCategorized` · 006d7390 · legions 006610b0
- [ ] `MasterDBMap::GetFreelistDef` · 006d73b0 · legions 006610d0
- [ ] `MasterDBMap::VerifyDirectory` · 006d73d0 · legions 006610f0

## `portal/shared/src/database/qualifieddataid.cpp.c` (0/6)

### QualifiedDataID
- [ ] `QualifiedDataID::QualifiedDataID` · 006d7ce0 · legions 00661860

### QualifiedDataIDArray
- [ ] `QualifiedDataIDArray::operator[]` · 006d7d10 · legions 00661890
- [ ] `QualifiedDataIDArray::AddToEnd` · 006d7f30 · legions 00661ab0
- [ ] `QualifiedDataIDArray::AddToEnd` · 006d7f60 · legions 00661ae0
- [ ] `QualifiedDataIDArray::AddToEnd` · 006d7fe0 · legions 00661b60
- [ ] `QualifiedDataIDArray::AddToEnd` · 006d8040 · legions 00661bc0

## `portal/shared/src/database/threadedcache.cpp.c` (0/16)

### DBCache
- [ ] `DBCache::SetShutdown` · 006cf490 · legions 006590b0

### ThreadedCache
- [ ] `ThreadedCache::WorkerExecuteRequest` · 006cf4b0 · legions 006590d0
- [ ] `ThreadedCache::SetShutdown` · 006cf4f0 · legions 00659110
- [ ] `ThreadedCache::StopThread` · 006cf520 · legions 00659140
- [ ] `ThreadedCache::WakeForTheReaper` · 006cf540
- [ ] `ThreadedCache::WorkerExecuteGetRequest` · 006cf550 · legions 00659170
- [ ] `ThreadedCache::WorkerExecuteSaveRequest` · 006cf580 · legions 006591a0
- [ ] `ThreadedCache::WorkerExecutePurgeRequest` · 006cf5d0 · legions 006591f0

### LFQueue<AsyncCache::CCallbackHandler_*>
- [ ] `LFQueue<AsyncCache::CCallbackHandler_*>::Consume` · 006cf600 · legions 00659220

### ThreadedCache
- [ ] `ThreadedCache::UseTime` · 006cf6d0 · legions 006592f0
- [ ] `ThreadedCache::ThreadedCache` · 006cf750 · legions 006594d0

### DIDName_Dat
- [ ] `DIDName_Dat::RequestMappings` · 006cf830 · legions 006595b0

### ThreadedCache
- [ ] `ThreadedCache::Startup` · 006cf850
- [ ] `ThreadedCache::PutOnWorkerThreadJobQueue` · 006cf8d0 · legions 00659650
- [ ] `ThreadedCache::AsyncGetInternal` · 006cf900 · legions 00659680
- [ ] `ThreadedCache::EnqueueAsyncRequest` · 006cf9d0 · legions 00659750

## `portal/shared/src/datasrc/datasrc.cpp.c` (0/4)

### DataSource
- [ ] `DataSource::DataSource` · 00736510 · legions 006b2080
- [ ] `DataSource::Open` · 00736530 · legions 006b20a0
- [ ] `DataSource::Close` · 00736550 · legions 006b20c0
- [ ] `DataSource::~DataSource` · 00736560 · legions 006b20d0

## `portal/shared/src/datasrc/filesrc.cpp.c` (0/4)

### FileSource
- [ ] `FileSource::FileSource` · 00736260 · legions 006b1f00
- [ ] `FileSource::OpenFile` · 00736280 · legions 006b1f20
- [ ] `FileSource::Close` · 00736310 · legions 006b1fb0
- [ ] `FileSource::~FileSource` · 00736360 · legions 006b2000

## `portal/shared/src/datasrc/outfilesrc.cpp.c` (0/3)

### OutFileSource
- [ ] `OutFileSource::OutFileSource` · 007363e0
- [ ] `OutFileSource::~OutFileSource` · 00736400
- [ ] `OutFileSource::OpenFile` · 00736460

## `portal/shared/src/datcompat/datidstamp.cpp.c` (0/12)

### DatIDStamp
- [ ] `DatIDStamp::operator=` · 0060cf60 · legions 006b2200
- [ ] `DatIDStamp::Clear` · 0060cf90 · legions 005cdd60
- [ ] `DatIDStamp::GenerateNewMajorVersion` · 0060cfb0 · legions 005cdd80
- [ ] `DatIDStamp::GenerateNewMinorVersion` · 0060cfd0 · legions 005cdda0
- [ ] `DatIDStamp::DatIDStamp` · 0060cfe0 · legions 005cddb0
- [ ] `DatIDStamp::DatIDStamp` · 0060d000 · legions 005cddd0
- [ ] `DatIDStamp::FromString` · 0060d040 · legions 005cde10
- [ ] `DatIDStamp::ToString` · 0060d0a0 · legions 005cde70
- [ ] `DatIDStamp::pack_size` · 0060d150 · legions 005cdf20
- [ ] `DatIDStamp::GetPackSize` · 0060d1c0
- [ ] `DatIDStamp::Pack` · 0060d1d0 · legions 005cdfa0
- [ ] `DatIDStamp::UnPack` · 0060d230

## `portal/shared/src/diskcon/blkalloc.cpp.c` (0/14)

### CLBlockAllocator
- [ ] `CLBlockAllocator::CLBlockAllocator` · 0064e6e0 · legions 005fda10
- [ ] `CLBlockAllocator::~CLBlockAllocator` · 0064e700 · legions 005fda30
- [ ] `CLBlockAllocator::Close_Data_File` · 0064e710 · legions 005fda40
- [ ] `CLBlockAllocator::SaveFileInfo` · 0064e730 · legions 005fda60
- [ ] `CLBlockAllocator::Store_Data` · 0064e750 · legions 005fda80
- [ ] `CLBlockAllocator::StoreDataRollback` · 0064e870 · legions 005fdba0
- [ ] `CLBlockAllocator::Load_Data` · 0064e930 · legions 005fdc60
- [ ] `CLBlockAllocator::DeleteBlocks` · 0064e9e0 · legions 005fdd10
- [ ] `CLBlockAllocator::ExpandFile` · 0064eac0 · legions 005fddf0
- [ ] `CLBlockAllocator::SaveTransaction` · 0064ebd0 · legions 005fdf00
- [ ] `CLBlockAllocator::ClearTransaction` · 0064ebf0 · legions 005fdf20
- [ ] `CLBlockAllocator::CreateDataFile` · 0064ec40 · legions 005fdf70
- [ ] `CLBlockAllocator::OpenDataFile` · 0064edd0 · legions 005fe100
- [ ] `CLBlockAllocator::Update` · 0064ee50 · legions 005fe180

## `portal/shared/src/diskcon/btree.cpp.c` (0/44)

### BTEntry
- [ ] `BTEntry::`default_constructor_closure'` · 0064c6f0

### BTree
- [ ] `BTree::HasEntry` · 0064c700 · legions 005fba30
- [ ] `BTree::ExtractEntryShift` · 0064c780 · legions 005fbab0
- [ ] `BTree::RemoveLeast` · 0064c820 · legions 005fbb50
- [ ] `BTree::AddMost` · 0064c8c0 · legions 005fbbf0
- [ ] `BTree::AddLeast` · 0064c900 · legions 005fbc30
- [ ] `BTree::Restamp_Entry` · 0064c970 · legions 005fbca0

### BTMemNode
- [ ] `BTMemNode::BTMemNode` · 0064c990 · legions 005fbcc0

### BTree
- [ ] `BTree::BTree` · 0064c9d0 · legions 005fbd00
- [ ] `BTree::DestroyTree` · 0064ca10 · legions 005fc030
- [ ] `BTree::CreateTree` · 0064ca60 · legions 005fbd40
- [ ] `BTree::LoadTree` · 0064cbb0 · legions 005fbe90
- [ ] `BTree::UpdateNode` · 0064ccf0 · legions 005fbfd0
- [ ] `BTree::SaveDataToFile` · 0064cd50 · legions 005fc080
- [ ] `BTree::FindNode` · 0064ce30 · legions 005fc160
- [ ] `BTree::AllocateEmptyNode` · 0064cfc0 · legions 005fc2f0
- [ ] `BTree::SplitNodeExec` · 0064d030 · legions 005fc360
- [ ] `BTree::InsertEntryExec` · 0064d230 · legions 005fc560
- [ ] `BTree::UpdateObjectExec` · 0064d370 · legions 005fc6a0
- [ ] `BTree::MergeNodesExec` · 0064d450 · legions 005fc780
- [ ] `BTree::MergeNodesTrans` · 0064d6a0 · legions 005fc9d0
- [ ] `BTree::FindMin` · 0064d730 · legions 005fca60
- [ ] `BTree::FindMax` · 0064d760 · legions 005fca90
- [ ] `BTree::DeleteLeafExec` · 0064d7a0 · legions 005fcad0
- [ ] `BTree::DeleteLeafTrans` · 0064d850 · legions 005fcb80
- [ ] `BTree::DeleteInternalExec` · 0064d8d0 · legions 005fcc00
- [ ] `BTree::RotateEntryExec` · 0064d9e0 · legions 005fcd10
- [ ] `BTree::RecoverTransaction` · 0064db80 · legions 005fceb0
- [ ] `BTree::Search` · 0064dc60 · legions 005fcf90
- [ ] `BTree::SplitNodeTrans` · 0064dce0 · legions 005fd010
- [ ] `BTree::InsertEntryTrans` · 0064dd70 · legions 005fd0a0
- [ ] `BTree::Update_Data_Trans` · 0064de00 · legions 005fd130
- [ ] `BTree::DescendToAdd` · 0064dea0 · legions 005fd1d0
- [ ] `BTree::DeleteInternalTrans` · 0064dfc0 · legions 005fd2f0
- [ ] `BTree::RotateEntryTrans` · 0064e040 · legions 005fd370
- [ ] `BTree::Lookup` · 0064e0d0 · legions 005fd400
- [ ] `BTree::DescendToDelete` · 0064e100 · legions 005fd430
- [ ] `BTree::Build_Id_List` · 0064e280 · legions 005fd5b0
- [ ] `BTree::Build_Id_List` · 0064e380 · legions 005fd6b0
- [ ] `BTree::Delete` · 0064e470 · legions 005fd7a0
- [ ] `BTree::Remove` · 0064e620 · legions 005fd950
- [ ] `BTree::Get_Id_List` · 0064e640 · legions 005fd970
- [ ] `BTree::Get_Id_List_Range` · 0064e660 · legions 005fd990
- [ ] `BTree::Try_Delete_Oldest` · 0064e6a0 · legions 005fd9d0

## `portal/shared/src/diskcon/diskcon.cpp.c` (0/19)

### DiskFileInfo_t
- [ ] `DiskFileInfo_t::ClearData` · 0064b700 · legions 005faa30

### DiskConBase
- [ ] `DiskConBase::GetDatIDStamp` · 0064b740 · legions 005faa70

### DiskController
- [ ] `DiskController::Close` · 0064b780 · legions 005faab0
- [ ] `DiskController::CheckRoom` · 0064b7d0 · legions 005fab00
- [ ] `DiskController::GetDatestamp` · 0064b830 · legions 005fab60
- [ ] `DiskController::GetDataSize` · 0064b860 · legions 005fab90
- [ ] `DiskController::LoadData` · 0064b890 · legions 005fabc0
- [ ] `DiskController::IsMember` · 0064b940 · legions 005fac70
- [ ] `DiskController::DeleteData` · 0064b980 · legions 005facb0
- [ ] `DiskController::SetMasterMapDID` · 0064b9c0 · legions 005facf0
- [ ] `DiskController::SetDatIDStamp` · 0064ba00 · legions 005fad30

### DiskFileInfo_t
- [ ] `DiskFileInfo_t::DiskFileInfo_t` · 0064ba30 · legions 005fad60

### DiskController
- [ ] `DiskController::SaveData` · 0064ba80 · legions 005fadb0
- [ ] `DiskController::DeleteDataByMask` · 0064bb10 · legions 005fae40
- [ ] `DiskController::LoadIterationList` · 0064bc00 · legions 005faf30
- [ ] `DiskController::SaveIterationList` · 0064bc80 · legions 005fafb0
- [ ] `DiskController::DiskController` · 0064bd10 · legions 005fb040
- [ ] `DiskController::~DiskController` · 0064bd70 · legions 005fb0a0
- [ ] `DiskController::InitFile` · 0064be10 · legions 005fb140

## `portal/shared/src/diskcon/diskdev.cpp.c` (0/8)

### DiskDev
- [ ] `DiskDev::DiskDev` · 00650480 · legions 005ff750

### DiskFileInfo_t
- [ ] `DiskFileInfo_t::operator=` · 00650490 · legions 005ff760

### DiskDev
- [ ] `DiskDev::Close` · 00650510 · legions 005ff7e0

### DiskHeaderBlock_t
- [ ] `DiskHeaderBlock_t::DiskHeaderBlock_t` · 00650540 · legions 005ff810

### DiskDev
- [ ] `DiskDev::~DiskDev` · 006505c0 · legions 005ff890
- [ ] `DiskDev::SyncRead` · 006505e0 · legions 005ff8b0
- [ ] `DiskDev::SyncWrite` · 00650640 · legions 005ff910
- [ ] `DiskDev::Open_File` · 006506a0 · legions 005ff970

## `portal/shared/src/diskcon/iterationlist.cpp.c` (1/5)

### (free functions)
- [ ] `CompareInts` · 0064c060

### CMostlyConsecutiveIntSet
- [ ] `CMostlyConsecutiveIntSet::CMostlyConsecutiveIntSet` · 0064c090 · legions 005fb3c0
- [ ] `CMostlyConsecutiveIntSet::Sort` · 0064c0e0 · legions 005fb410
- [x] `CMostlyConsecutiveIntSet::StreamPack` · 0064c170
- [ ] `CMostlyConsecutiveIntSet::Add` · 0064c370 · legions 005fb6a0

## `portal/shared/src/diskcon/lookfile.cpp.c` (0/3)

### LookFile
- [ ] `LookFile::GetFoundPath` · 0064c3b0 · legions 005fb6e0
- [ ] `LookFile::IsFileHere` · 0064c3d0 · legions 005fb700
- [ ] `LookFile::LookForFile` · 0064c4a0 · legions 005fb7d0

## `portal/shared/src/diskcon/lrulist.cpp.c` (0/21)

### DLListBase
- [ ] `DLListBase::Remove` · 0064efe0 · legions 00430630

### LRUB_Mem_t
- [ ] `LRUB_Mem_t::LRUB_Mem_t` · 0064f030 · legions 005fe310
- [ ] `LRUB_Mem_t::Make_Buffer` · 0064f0b0 · legions 005fe390
- [ ] `LRUB_Mem_t::Remove_Below` · 0064f110 · legions 005fe3f0
- [ ] `LRUB_Mem_t::Remove_Above` · 0064f150 · legions 005fe430

### LRU_List
- [ ] `LRU_List::Create_List` · 0064f190 · legions 005fe470

### LRUB_Mem_t
- [ ] `LRUB_Mem_t::LRUB_Mem_t` · 0064f1d0 · legions 005fe4b0

### LRU_List
- [ ] `LRU_List::Load_List` · 0064f210 · legions 005fe4f0
- [ ] `LRU_List::Add_Exec` · 0064f360 · legions 005fe640
- [ ] `LRU_List::Expand_Exec` · 0064f3f0 · legions 005fe6d0
- [ ] `LRU_List::Delete_Exec` · 0064f500 · legions 005fe7e0
- [ ] `LRU_List::Expand_Trans` · 0064f650 · legions 005fe960
- [ ] `LRU_List::Delete_Trans` · 0064f760 · legions 005fea70
- [ ] `LRU_List::LRU_List` · 0064f970 · legions 005fec40
- [ ] `LRU_List::Mark_Used` · 0064fa60 · legions 005fed30
- [ ] `LRU_List::Mark_Deleted` · 0064fb70 · legions 005fee40
- [ ] `LRU_List::Write_Added` · 0064fc00 · legions 005feed0
- [ ] `LRU_List::Purge_Duplicates` · 0064fc60 · legions 005fef30
- [ ] `LRU_List::Get_Oldest` · 0064fee0 · legions 005ff1b0
- [ ] `LRU_List::~LRU_List` · 006500f0 · legions 005ff3c0
- [ ] `LRU_List::Write_All_Out` · 00650340 · legions 005ff610

## `portal/shared/src/filedb/dbfile2idtable.cpp.c` (0/6)

### DBFile2IDTable
- [ ] `DBFile2IDTable::GetPackSize` · 0066d4d0

### CaseInsensitiveStringBase<PStringBase<char>_>
- [ ] `CaseInsensitiveStringBase<PStringBase<char>_>::case_insensitive_hash` · 0066d4f0 · legions 00617cf0

### TFileEntry
- [ ] `TFileEntry::GetPath` · 0066d540 · legions 00617d40

### ShortcutInfo
- [ ] `ShortcutInfo::GetDataStr` · 0066d560 · legions 00617d60

### DBFile2IDTable
- [ ] `DBFile2IDTable::Pack` · 0066d690
- [ ] `DBFile2IDTable::UnPack` · 0066d910

## `portal/shared/src/filedb/didnamedat.cpp.c` (0/17)

### DIDName_Dat
- [ ] `DIDName_Dat::RequestClearAll` · 00670d70 · legions 0061b440
- [ ] `DIDName_Dat::SetTypeData` · 00670d80 · legions 0061b450
- [ ] `DIDName_Dat::OnDone` · 00670d90 · legions 0061b460
- [ ] `DIDName_Dat::WriteToDIDVersionLog` · 00670db0 · legions 0061b480
- [ ] `DIDName_Dat::Cleanup` · 00670e10 · legions 0061b4e0
- [ ] `DIDName_Dat::InitFromFilename` · 00670e40 · legions 0061b510
- [ ] `DIDName_Dat::TestForExistance` · 00670ef0 · legions 0061b5c0
- [ ] `DIDName_Dat::RequestAllTypeData` · 00670f30 · legions 0061b600
- [ ] `DIDName_Dat::RequestTypeNumber` · 00671010 · legions 0061b6e0
- [ ] `DIDName_Dat::RequestMapping` · 00671100 · legions 0061b7d0

### DBOCache
- [ ] `DBOCache::SetCollection` · 00671190 · legions 0061b860

### DIDName_Dat
- [ ] `DIDName_Dat::DIDName_Dat` · 006711a0 · legions 0061b870
- [ ] `DIDName_Dat::~DIDName_Dat` · 00671210 · legions 0061b8e0
- [ ] `DIDName_Dat::Read` · 00671280 · legions 0061b950
- [ ] `DIDName_Dat::Save` · 006713c0 · legions 0061ba90
- [ ] `DIDName_Dat::RequestAllMappings` · 00671a80 · legions 0061c150
- [ ] `DIDName_Dat::GetDescriptionString` · 00671c70 · legions 0061c340

## `portal/shared/src/filedb/file2idtable.cpp.c` (0/33)

### TFile2IDTable
- [ ] `TFile2IDTable::GetFileWriteTimeByDID` · 0066da60 · legions 00618150
- [ ] `TFile2IDTable::GetFileWriteTimeByEntry` · 0066dab0 · legions 006181a0
- [ ] `TFile2IDTable::GetHighestDID` · 0066dad0 · legions 006181c0

### TFileEntry
- [ ] `TFileEntry::SetPath` · 0066dbb0 · legions 006182a0
- [ ] `TFileEntry::~TFileEntry` · 0066dc10 · legions 00618300
- [ ] `TFileEntry::TFileEntry` · 0066dc80 · legions 00618370

### TFile2IDTable
- [ ] `TFile2IDTable::GetFileNameByDID` · 0066dd10 · legions 00618400
- [ ] `TFile2IDTable::GetFullPathByDID` · 0066dda0 · legions 00618490
- [ ] `TFile2IDTable::GetFileWriteTimeByFileName` · 0066de30 · legions 00618520
- [ ] `TFile2IDTable::GetDBTypeString` · 0066ded0 · legions 006185c0
- [ ] `TFile2IDTable::ReformatDirectoryName` · 0066df20 · legions 00618610
- [ ] `TFile2IDTable::RegisterDBType` · 0066dfc0 · legions 00618720
- [ ] `TFile2IDTable::SanitizeFileName` · 0066e210 · legions 00618970
- [ ] `TFile2IDTable::GetDIDByFileName` · 0066e2a0 · legions 00618a00
- [ ] `TFile2IDTable::GetFullPathByFileName` · 0066e3a0 · legions 00618b00
- [ ] `TFile2IDTable::GetFullPathByEntry` · 0066e4e0 · legions 00618c40
- [ ] `TFile2IDTable::GetDBTypes` · 0066e5d0 · legions 00618d30
- [ ] `TFile2IDTable::GetDIDs` · 0066e660 · legions 00618dc0
- [ ] `TFile2IDTable::AssignSoonToExistFilePath` · 0066e720 · legions 00618e80
- [ ] `TFile2IDTable::IsDBTypeRegistered` · 0066e890 · legions 00618ff0
- [ ] `TFile2IDTable::IsDBDirSearched` · 0066e8d0 · legions 00619030
- [ ] `TFile2IDTable::FindEntryByDID` · 0066e910 · legions 00619070

### TDBTypeEntry
- [ ] `TDBTypeEntry::~TDBTypeEntry` · 0066ea80 · legions 00619140

### TFile2IDTable
- [ ] `TFile2IDTable::GetFilePaths` · 0066eb10 · legions 006191d0
- [ ] `TFile2IDTable::Clear` · 0066ef40 · legions 00619560

### TDBTypeEntry
- [ ] `TDBTypeEntry::TDBTypeEntry` · 0066f140 · legions 00619850

### TFile2IDTable
- [ ] `TFile2IDTable::TFile2IDTable` · 0066f1a0 · legions 006198b0
- [ ] `TFile2IDTable::~TFile2IDTable` · 0066f230 · legions 00619940
- [ ] `TFile2IDTable::AssignDIDToFileName` · 0066f2b0 · legions 006199c0
- [ ] `TFile2IDTable::AssignNonAuthoritativePathToFileName` · 0066f4b0 · legions 00619bc0
- [ ] `TFile2IDTable::GetDBTypeEntry` · 0066f6f0 · legions 00619e00
- [ ] `TFile2IDTable::FindEntryByName` · 0066f810 · legions 00619f20
- [ ] `TFile2IDTable::AssignPathToFileNameInternal` · 0066f9a0 · legions 0061a0b0

## `portal/shared/src/filedb/filearray.cpp.c` (0/25)

### FileArray
- [ ] `FileArray::IsActive` · 0066c410 · legions 00616cc0
- [ ] `FileArray::SetUsesDirs` · 0066c420 · legions 00616ce0
- [ ] `FileArray::SetDIDAllocOk` · 0066c440 · legions 00616d00

### FileArrayFiller
- [ ] `FileArrayFiller::ProcessMapping` · 0066c450

### FileArray
- [ ] `FileArray::ReadIDs` · 0066c4d0 · legions 00616d90
- [ ] `FileArray::WriteMasterIDs` · 0066c570 · legions 00616e30
- [ ] `FileArray::Cleanup` · 0066c5d0 · legions 00616e90
- [ ] `FileArray::VerifyDBOTypes` · 0066c630 · legions 00616ef0
- [ ] `FileArray::GenerateDescription` · 0066c820 · legions 006170e0
- [ ] `FileArray::RegisterDBType` · 0066c8c0 · legions 00617180
- [ ] `FileArray::GetAbsoluteFilePath` · 0066ca40 · legions 00617300
- [ ] `FileArray::GetAbsoluteFilePathForDummies` · 0066cb40 · legions 00617400
- [ ] `FileArray::GetFileName` · 0066cb80 · legions 00617440
- [ ] `FileArray::GetFileNameForDummies` · 0066cc70 · legions 00617530
- [ ] `FileArray::GetFileNameForDummiesW` · 0066ccb0 · legions 00617570
- [ ] `FileArray::GetAbsoluteFilePath` · 0066cd20 · legions 006175e0

### FileArrayFiller
- [ ] `FileArrayFiller::ProcessTypeData` · 0066ce40

### FileArray
- [ ] `FileArray::GetDID` · 0066ce60 · legions 00617720
- [ ] `FileArray::GetDIDW` · 0066d050 · legions 00617910
- [ ] `FileArray::GetDIDForDummies` · 0066d0a0 · legions 00617960
- [ ] `FileArray::GetDIDForDummiesW` · 0066d150 · legions 00617a10
- [ ] `FileArray::GetPhysicsObjectDID` · 0066d1a0
- [ ] `FileArray::GetFilePathForDummies` · 0066d260 · legions 00617a60
- [ ] `FileArray::AddIDTableStorage` · 0066d280 · legions 00617a80
- [ ] `FileArray::Init` · 0066d2e0 · legions 00617ae0

## `portal/shared/src/filedb/filefinder.cpp.c` (0/17)

### TFileFinder
- [ ] `TFileFinder::TFileFinder` · 0066fca0 · legions 0061a370
- [ ] `TFileFinder::~TFileFinder` · 0066fd10 · legions 0061a3e0
- [ ] `TFileFinder::SetUsesDirs` · 0066fd20 · legions 0061a3f0
- [ ] `TFileFinder::RecurseInto` · 0066fd30 · legions 0061a400
- [ ] `TFileFinder::SearchForFileRecursive` · 0066ffa0 · legions 0061a670
- [ ] `TFileFinder::PowerOnHashes` · 00670390 · legions 0061aa60
- [ ] `TFileFinder::SearchDBTypeDirs` · 006703e0 · legions 0061aab0
- [ ] `TFileFinder::SearchForFile` · 006704e0 · legions 0061abb0
- [ ] `TFileFinder::FileExists` · 006705c0 · legions 0061ac90
- [ ] `TFileFinder::AssignDIDToFileName` · 00670910 · legions 0061afe0
- [ ] `TFileFinder::AssignNonAuthoritativePathToFileName` · 006709a0 · legions 0061b070
- [ ] `TFileFinder::AssignSoonToExistFilePath` · 00670a60 · legions 0061b130
- [ ] `TFileFinder::AssignPathToFileNameInternal` · 00670af0 · legions 0061b1c0
- [ ] `TFileFinder::GetFullPathByFileName` · 00670bb0 · legions 0061b280
- [ ] `TFileFinder::GetFileWriteTimeByFileName` · 00670c40 · legions 0061b310
- [ ] `TFileFinder::GetFullPathByEntry` · 00670cd0 · legions 0061b3a0
- [ ] `TFileFinder::GetFileWriteTimeByEntry` · 00670d20 · legions 0061b3f0

## `portal/shared/src/filedb/filepathcache.cpp.c` (0/4)

### PHashTable<PStringBase<char>,PStringBase<char>_>
- [ ] `PHashTable<PStringBase<char>,PStringBase<char>_>::PHashTable<PStringBase<char>,PStringBase<char>_>` · 00672020 · legions 0061c5e0
- [ ] `PHashTable<PStringBase<char>,PStringBase<char>_>::StreamPack` · 00672060 · legions 0061c620

### FilePathCache
- [ ] `FilePathCache::InitFileFinder` · 00672360 · legions 0061c920
- [ ] `FilePathCache::SaveFileFinder` · 00672670 · legions 0061cc30

## `portal/shared/src/fileformat/fileformat.cpp.c` (0/45)

### FileTemplate
- [ ] `FileTemplate::FreeMatchData` · 006661e0

### FileFormatGrabber
- [ ] `FileFormatGrabber::FileFormatGrabber` · 00666250
- [ ] `FileFormatGrabber::GetFileFormat` · 006662e0 · legions 00613a90

### PFileNode
- [ ] `PFileNode::SetNameAsInt32` · 006662f0 · legions 00613aa0
- [ ] `PFileNode::SetName` · 00666340 · legions 00613af0

### FileFormat
- [ ] `FileFormat::IsBinary` · 00666390 · legions 00613b40

### PFileNode
- [ ] `PFileNode::GetNameAsCellID` · 00666480
- [ ] `PFileNode::GetNameAsHex` · 00666540

### TemplateInstance
- [ ] `TemplateInstance::TemplateInstance` · 00666600
- [ ] `TemplateInstance::TemplateInstance` · 00666630

### PStringBase<char>
- [ ] `PStringBase<char>::~PStringBase<char>` · 006666a0 · legions 005f0620

### FileTemplate
- [ ] `FileTemplate::NodeMatchesTemplate` · 006666c0
- [ ] `FileTemplate::BinarySize_Internal` · 00666800
- [ ] `FileTemplate::BinarySize_SubNodes` · 00666960
- [ ] `FileTemplate::HandleRepeatNode` · 00666a20
- [ ] `FileTemplate::SubNodesMatchTemplates` · 00666c70

### (free functions)
- [ ] `is_keyword` · 00666dc0

### FileTemplate
- [ ] `FileTemplate::MakeFileNode` · 00667010

### FileFormat
- [-] `FileFormat::ToFileNode` · 006672a0 — text source format (admin tools)

### FileTemplate
- [ ] `FileTemplate::VerifyAndConvert` · 00667500
- [ ] `FileTemplate::BinarySize` · 00667580
- [ ] `FileTemplate::PackBinary_Internal` · 00667620
- [ ] `FileTemplate::PackBinary_SubNodes` · 006678d0
- [ ] `FileTemplate::UnPackBinary_Internal` · 006679c0 · legions 00613cd0
- [ ] `FileTemplate::UnPackBinary_SubNodes` · 00667c40 · legions 00613f50
- [ ] `FileTemplate::PackBinary` · 00667d10

### FileFormat
- [ ] `FileFormat::SaveBinary` · 00667d70
- [ ] `FileFormat::LoadBinary` · 00667ea0 · legions 00614020
- [ ] `FileFormat::SaveBinary` · 00668420
- [ ] `FileFormat::LoadFile` · 00668450 · legions 00614220

### _STL::map<unsigned_long,FileTemplate_*,_STL::greater<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,FileTemplate_*>_>_>
- [ ] `_STL::map<unsigned_long,FileTemplate_*,_STL::greater<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,FileTemplate_*>_>_>::operator[]` · 00668a70

### _STL::map<PStringBase<char>,unsigned_long,_STL::less<PStringBase<char>_>,_STL::allocator<_STL::pair<PStringBase<char>_const_,unsigned_long>_>_>
- [ ] `_STL::map<PStringBase<char>,unsigned_long,_STL::less<PStringBase<char>_>,_STL::allocator<_STL::pair<PStringBase<char>_const_,unsigned_long>_>_>::operator[]` · 00668b30

### FileTemplate
- [ ] `FileTemplate::FileTemplate` · 00668bc0
- [ ] `FileTemplate::Destroy` · 00668c20 · legions 00614290
- [ ] `FileTemplate::MakeNodeTemplate` · 00668cd0

### FileFormat
- [ ] `FileFormat::FileFormat` · 00668d80 · legions 00614340
- [ ] `FileFormat::GetDBOType` · 00668de0
- [ ] `FileFormat::Allocate` · 00668df0

### FileTemplate
- [ ] `FileTemplate::~FileTemplate` · 00668e00 · legions 006143c0
- [ ] `FileTemplate::ParseNodeTemplate_Internal` · 00668e90
- [ ] `FileTemplate::ParseDefinitions` · 00669490
- [ ] `FileTemplate::ParseNodeTemplate` · 00669670

### FileFormat
- [-] `FileFormat::FromFileNode` · 00669800 — text source format (admin tools)
- [ ] `FileFormat::Destroy` · 00669a10 · legions 00614450
- [ ] `FileFormat::~FileFormat` · 00669ab0 · legions 006144f0

## `portal/shared/src/pheader/pheader.cpp.c` (0/9)

### PreprocHeader
- [ ] `PreprocHeader::PreprocHeader` · 00678990 · legions 00622fd0
- [ ] `PreprocHeader::~PreprocHeader` · 006789c0 · legions 00623000
- [ ] `PreprocHeader::SetHeaderData` · 00678a00 · legions 00623040
- [ ] `PreprocHeader::Init` · 00678a60 · legions 006230a0
- [ ] `PreprocHeader::GetHeaderData` · 00678b70
- [ ] `PreprocHeader::SaveHeaderInternal` · 00678bc0 · legions 006231b0
- [ ] `PreprocHeader::SaveHeader` · 00678cf0 · legions 006232e0
- [ ] `PreprocHeader::SaveHeaderAsComment` · 00678d20 · legions 00623310
- [ ] `PreprocHeader::GetHeader` · 00678dc0
