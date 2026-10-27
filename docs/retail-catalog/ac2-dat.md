# ac2-dat: DAT files and the database layer

507 retail functions: 42 done, 7 partial, 67 to do, 391 skipped.
Legend and workflow: [README](README.md).

## `portal/shared/src/database/asynccache.cpp.c` (1/48)

### AsyncCache::CAsyncRequest
- [-] `AsyncCache::CAsyncRequest::AdoptDBObj` · 006d0900 · legions 0065a650 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)
- [-] `AsyncCache::CAsyncRequest::ReleaseDBObj` · 006d0920 · legions 0065a670 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)

### AsyncCache
- [-] `AsyncCache::EnqueueAsyncRequest` · 006d0940 · legions 0065a690 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)
- [-] `AsyncCache::OnRequestFinished` · 006d0a00 · legions 0065a750 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)
- [-] `AsyncCache::OnPurgeRequestFinished` · 006d0a30 · legions 0065a780 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)
- [-] `AsyncCache::AddObjToDBOCache` · 006d0a40 · legions 0065a790 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)

### DBCache
- [ ] `DBCache::GetIfInMemory` · 006d0a60 · legions 0065a7b0 — needs port: sub-object resolution, load dependent DIDs on first fetch (AGENTS item 1); verify ac2-db dependency plan and cite

### AsyncCache
- [-] `AsyncCache::GetFreeObj` · 006d0a90 · legions 0065a7e0 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)
- [-] `AsyncCache::IsOnDisk` · 006d0ac0 · legions 0065a810 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)
- [-] `AsyncCache::LoadData` · 006d0b00 · legions 0065a850 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)
- [-] `AsyncCache::PurgeData` · 006d0b40 · legions 0065a890 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)
- [-] `AsyncCache::SaveData` · 006d0b80 · legions 0065a8d0 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)

### CPhysicsObj
- [x] `CPhysicsObj::RecvEvt_SetMode` · 006d0bc0 · legions 0065a910 — ac2-client motion.rs forwards SessionEvent::ObjectAction to the PhysicsBody entry point (players and other objects)

### CObjCell
- [-] `CObjCell::GetCID` · 006d0bd0 · legions 0065a920 — trivial accessor or setter; the value is a field of the Rust struct (cell id is a field of the Rust cell struct)

### AsyncCache
- [-] `AsyncCache::BlockingGetFromDisk` · 006d0be0 · legions 0065a930 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)

### NIList<AsyncCache::CCallbackHandler_*>
- [-] `NIList<AsyncCache::CCallbackHandler_*>::RemoveFromHead` · 006d0c50 · legions 00679d20 — STL/container template internals (Rust std collections)

### AsyncCache::CAsyncRequest
- [-] `AsyncCache::CAsyncRequest::~CAsyncRequest` · 006d0c80 · legions 0065a9a0 — C++ destructor or operator (Rust Drop / value semantics)
- [-] `AsyncCache::CAsyncRequest::RemoveAllCallbacks` · 006d0cb0 · legions 0065a9d0 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)

### AsyncCache::CAsyncGetRequest
- [-] `AsyncCache::CAsyncGetRequest::ReleaseDBObj` · 006d0d00 · legions 0065aa20 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)
- [-] `AsyncCache::CAsyncGetRequest::~CAsyncGetRequest` · 006d0d50 · legions 0065aa70 — C++ destructor or operator (Rust Drop / value semantics)
- [ ] `AsyncCache::CAsyncGetRequest::bAllDependanciesDone` · 006d0e20 · legions 0065ab40 — needs port: sub-object resolution, load dependent DIDs on first fetch (AGENTS item 1); verify ac2-db dependency plan and cite

### AsyncCache
- [ ] `AsyncCache::OnAsyncGetFinished` · 006d0e30 · legions 0065ab50 — needs port: sub-object resolution, load dependent DIDs on first fetch (AGENTS item 1); verify ac2-db dependency plan and cite
- [-] `AsyncCache::BlockingLoadInto` · 006d0ec0 · legions 0065abe0 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)

### AsyncCache::CAsyncGetRequest
- [-] `AsyncCache::CAsyncGetRequest::CAsyncGetRequest` · 006d0f40 · legions 0065ac60 — constructor (Rust struct init)

### AsyncCache::CAsyncRequest
- [-] `AsyncCache::CAsyncRequest::RemoveCallback` · 006d0fb0 · legions 0065acd0 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)

### AsyncCache::CCallbackHandler
- [-] `AsyncCache::CCallbackHandler::OnContextReleased` · 006d1020 · legions 0065ad40 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)
- [-] `AsyncCache::CCallbackHandler::~CCallbackHandler` · 006d1090 · legions 0065adb0 — C++ destructor or operator (Rust Drop / value semantics)

### AsyncCache
- [~] `AsyncCache::UseTime` · 006d1140 · legions 0065ae60 — ac2-db typed cache trims on the inserting worker; the finished-request pump is the client's time-budgeted loader drain (LOAD_BUDGET)
- [-] `AsyncCache::AreOnDisk` · 006d11d0 · legions 0065aef0 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)
- [-] `AsyncCache::FlushPendingRequests` · 006d1470 · legions 0065b140 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)

### AsyncCache::CAsyncRequest
- [-] `AsyncCache::CAsyncRequest::AddCallback` · 006d15e0 · legions 0065b2b0 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)

### AsyncCache
- [ ] `AsyncCache::FormGetRequest` · 006d1630 · legions 0065b300 — needs port: sub-object resolution, load dependent DIDs on first fetch (AGENTS item 1); verify ac2-db dependency plan and cite
- [-] `AsyncCache::OnSaveRequestFinished` · 006d16d0 · legions 0065b3a0 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)
- [-] `AsyncCache::UnhashPendingGet` · 006d1770 · legions 0065b440 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)
- [-] `AsyncCache::ReleaseContext` · 006d1830 · legions 0065b500 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)
- [-] `AsyncCache::NotifyCallback` · 006d18b0 · legions 0065b580 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)
- [-] `AsyncCache::BlockingGet` · 006d19d0 · legions 0065b6a0 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)
- [ ] `AsyncCache::OnAsyncGetFromOtherSourcesFailed` · 006d1ab0 · legions 0065b780 — needs port: cite-only; failed get from other sources completes with an error; ac2-db typed cache failure events likely cover it, verify and cite (ac2-net opcode.rs:44 cites the name)

### AsyncCache::CCallbackHandler
- [-] `AsyncCache::CCallbackHandler::AddTopLevelRequest` · 006d1af0 · legions 0065b7c0 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)

### AsyncCache
- [ ] `AsyncCache::AddToAsyncGet` · 006d1b50 · legions 0065b820 — needs port: sub-object resolution, load dependent DIDs on first fetch (AGENTS item 1); verify ac2-db dependency plan and cite
- [-] `AsyncCache::AllocateCallback` · 006d1d20 · legions 0065ba40 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)
- [ ] `AsyncCache::AsyncGet` · 006d1de0 · legions 0065bb00 — needs port: sub-object resolution, load dependent DIDs on first fetch (AGENTS item 1); verify ac2-db dependency plan and cite
- [ ] `AsyncCache::AsyncGet` · 006d1e90 · legions 0065bbb0 — needs port: sub-object resolution, load dependent DIDs on first fetch (AGENTS item 1); verify ac2-db dependency plan and cite
- [-] `AsyncCache::AsyncPurge` · 006d2030 · legions 0065bd50 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)
- [-] `AsyncCache::AsyncPurge` · 006d2120 · legions 0065be40 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)
- [-] `AsyncCache::AsyncSave` · 006d22b0 · legions 0065bfd0 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)
- [-] `AsyncCache::HashAndEnqueue` · 006d23a0 · legions 0065c0c0 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)
- [-] `AsyncCache::OnGetRequestFinished` · 006d2410 · legions 0065c130 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)

## `portal/shared/src/database/clidatevents.cpp.c` (9/12)

### CCliDatEvent
- [x] `CCliDatEvent::CCliDatEvent` · 006da360 · legions 006637a0

### CCliDatDataEvent
- [-] `CCliDatDataEvent::CreateCachePack` · 006da3d0 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)

### CCliDatInterrogationResponseEvent
- [x] `CCliDatInterrogationResponseEvent::CreateData` · 006da480

### CCliDatBeginDDDEvent
- [x] `CCliDatBeginDDDEvent::GetMissingIterations` · 006da500 · legions 00663940

### CCliDatRequestEvent
- [x] `CCliDatRequestEvent::CreateForSend` · 006da550 · legions 00663990

### CAllIterationList::PTaggedIterationList
- [x] `CAllIterationList::PTaggedIterationList::StreamPack` · 006da5b0 · legions 006639f0

### CCliDatInterrogationResponseEvent
- [x] `CCliDatInterrogationResponseEvent::CreateForSend` · 006da600

### CCliDatEndDDDEvent
- [x] `CCliDatEndDDDEvent::CreateForSend` · 006da680 · legions 00663ac0

### (free functions)
- [ ] `StreamIDs` · 006da6d0 · legions 00663b10 — needs port: cite-only; QualifiedDataID list stream for CliDatDataEvent; ac2-net protocol/dat_event.rs reads id sets, verify and cite

### CMissingIteration
- [x] `CMissingIteration::StreamPack` · 006da7b0

### SmartArray<CAllIterationList::PTaggedIterationList>
- [-] `SmartArray<CAllIterationList::PTaggedIterationList>::AddToEnd` · 006da840 · legions 00663c80 — STL/container template internals (Rust std collections)

### CAllIterationList
- [x] `CAllIterationList::AddIterationList` · 006da8b0

## `portal/shared/src/database/datfiledata.cpp.c` (0/8)

### DatFileData
- [-] `DatFileData::GetDBOType` · 006d9930 — DBObj class factory (Allocate/GetDBOType); Rust decodes into typed structs
- [-] `DatFileData::Pack` · 006d9940 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)

### IntMap
- [-] `IntMap::Pack` · 006d9960 · legions 00662ce0 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)

### DatFileData
- [-] `DatFileData::DatFileData` · 006d99c0 · legions 00662d40 — constructor (Rust struct init)
- [-] `DatFileData::Allocate` · 006d9a20 — DBObj class factory (Allocate/GetDBOType); Rust decodes into typed structs
- [ ] `DatFileData::UnPack` · 006d9a30 — needs port: DatFileData payload decode (wraps IntMap::UnPack of the groove map); verify whether any shipped DAT record uses it and cite

### IntMap
- [ ] `IntMap::UnPack` · 006d9a50 · legions 00662dd0 — needs port: IntMap payload decode (int key to GrooveInfo map) for DatFileData; ac2-dat archive/cursor/collections.rs does generic collections, verify and cite

### GrooveInfo
- [-] `GrooveInfo::GrooveInfo` · 006d9b30 · legions 00662f60 — C++ constructor/destructor (Rust struct init or Drop)

## `portal/shared/src/database/dbcache.cpp.c` (6/30)

### DBCache
- [-] `DBCache::ResetCache` · 006cfa10 · legions 00659790 — retail DB cache internals (freelist, pool, idle eviction, reload); AGENTS says do not reproduce
- [-] `DBCache::SetMasterMapDID` · 006cfa20 · legions 006597a0 — trivial accessor or setter; the value is a field of the Rust struct (master map DID is a setter; ac2-script/src/data/db_types.rs holds the DID ranges)
- [ ] `DBCache::Get` · 006cfa30 · legions 006597b0 — needs port (check): only cited from ac2-audio; cite in the ac2-db typed load path or port
- [x] `DBCache::GetDIDFromEnumStatic` · 006cfa60 · legions 006597e0 — enum/group to DID lookup; cited in ac2-script/src/glue/enums.rs:39
- [x] `DBCache::GetDIDFromEnum` · 006cfa80 · legions 00659800 — ac2-script data/glue::enums
- [ ] `DBCache::GetFromEnum` · 006cfb60 · legions 006598b0 — needs port: cite-only; enum and group to DID (non-static); ac2-script/src/data/enum_map.rs and DBObj::GetByEnum cover it, verify and cite
- [ ] `DBCache::GetCurrentRegion` · 006cfbb0 · legions 00659900 — needs port: cite-only; current region for DID lookups, check against ac2-world region state
- [x] `DBCache::IsRunTime` · 006cfbd0 · legions 00659920 — run-time client cache flag; cited in ac2-script/src/glue/enums.rs:28
- [x] `DBCache::IsClient` · 006cfbf0 · legions 00659940 — client flag; cited with IsRunTime in ac2-script/src/glue/enums.rs:28
- [x] `DBCache::IsServer` · 006cfc10 · legions 00659960 — server flag; cited with IsRunTime in ac2-script/src/glue/enums.rs:28
- [-] `DBCache::SetRegion` · 006cfc30 · legions 00659980 — trivial accessor or setter; the value is a field of the Rust struct
- [-] `DBCache::ReloadObject` · 006cfc40 · legions 00659990 — retail DB cache internals (freelist, pool, idle eviction, reload); AGENTS says do not reproduce
- [ ] `DBCache::AddToDataGraph` · 006cfcf0 · legions 00659a40 — needs port: verify whether AddToDataGraph matters outside tools (AGENTS open question); record finding in AGENTS
- [~] `DBCache::SetLocalLanguage` · 006cfdc0 · legions 00659b10 — the local DAT choice at mount (ac2_dat local_dat_for_language); no live switch, and the id path is not ported
- [x] `DBCache::GetLocalLanguage` · 006cfde0 — local DAT language id; cited in ac2-weenie/src/session.rs:64
- [ ] `DBCache::Init` · 006cfec0 · legions 00659c20 — needs port: cite-only; cache init (DAT open per language, master map, budgets); behaviour spread over ac2-dat dataset.rs open and ac2-db typed cache
- [~] `DBCache::UseTime` · 006cfff0 · legions 00659d40 — ac2-db typed cache trims on the inserting worker; the finished-request pump is the client's time-budgeted loader drain (LOAD_BUDGET)
- [ ] `DBCache::UnloadCellData` · 006d0050 · legions 00659da0 — needs port: verify cell data unload on landblock leave is covered by ac2-world unload (AGENTS: free on last drop); cell eviction behaviour unverified
- [-] `DBCache::AskForLastWords` · 006d00e0 · legions 00659e30 — shutdown write-back of cached objects (their last-words save to DAT); client writes nothing (AGENTS)
- [-] `DBCache::FlushFreeObjects` · 006d0140 · legions 00659ee0 — retail DB cache internals (freelist, pool, idle eviction, reload); AGENTS says do not reproduce
- [-] `DBCache::GetDBOCache` · 006d01f0 · legions 00659f40 — retail DB cache internals (freelist, pool, idle eviction, reload); AGENTS says do not reproduce
- [-] `DBCache::DestroyObjectCaches` · 006d0230 · legions 00659f80 — retail DB cache internals (freelist, pool, idle eviction, reload); AGENTS says do not reproduce
- [-] `DBCache::ReloadCache` · 006d02b0 · legions 0065a000 — retail DB cache internals (freelist, pool, idle eviction, reload); AGENTS says do not reproduce

### AsyncCache
- [-] `AsyncCache::~AsyncCache` · 006d0440 · legions 0065a0f0 — C++ destructor or operator (Rust Drop / value semantics)

### DBCache
- [-] `DBCache::~DBCache` · 006d04d0 · legions 0065a180 — C++ destructor or operator (Rust Drop / value semantics)
- [-] `DBCache::KeepFreeObjects` · 006d0500 · legions 0065a1b0 — retail DB cache internals (freelist, pool, idle eviction, reload); AGENTS says do not reproduce
- [ ] `DBCache::GetIfInMemory` · 006d05a0 · legions 0065a250 — needs port: sub-object resolution, load dependent DIDs on first fetch (AGENTS item 1); verify ac2-db dependency plan and cite
- [-] `DBCache::GetDBOCache` · 006d05e0 · legions 0065a290 — retail DB cache internals (freelist, pool, idle eviction, reload); AGENTS says do not reproduce

### AsyncCache
- [-] `AsyncCache::AsyncCache` · 006d0820 · legions 0065a570 — constructor (Rust struct init)

### DBCache
- [-] `DBCache::DBCache` · 006d08a0 · legions 0065a5f0 — constructor (Rust struct init)

## `portal/shared/src/database/dbcategories.cpp.c` (0/14)

### DBCategories
- [ ] `DBCategories::FillDataGraph` · 006d8bc0 — needs port: verify whether AddToDataGraph matters outside tools (AGENTS open question); record finding in AGENTS

### ParseHelper<EmpGenParse>::ParseHandler
- [-] `ParseHelper<EmpGenParse>::ParseHandler::ParseHandler` · 006d8c30 — constructor (Rust struct init)

### DBCategories
- [-] `DBCategories::SaveToFile` · 006d8c60 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `DBCategories::Destroy` · 006d8fd0 — DBObj class factory (Allocate/GetDBOType); Rust decodes into typed structs

### HashTable<CaseInsensitiveStringBase<PStringBase<char>_>,unsigned_long>
- [-] `HashTable<CaseInsensitiveStringBase<PStringBase<char>_>,unsigned_long>::operator=` · 006d8fe0 · legions 006625e0 — C++ destructor or operator (Rust Drop / value semantics)

### DBCategories
- [-] `DBCategories::ParseEMPsNode` · 006d9130 — admin-only tooling (text source, preprocess, EMP or format authoring)

### PHashTable<CaseInsensitiveStringBase<PStringBase<char>_>,unsigned_long>
- [ ] `PHashTable<CaseInsensitiveStringBase<PStringBase<char>_>,unsigned_long>::StreamPack` · 006d9480 · legions 00662740 — needs port: cite-only; PHashTable StreamPack decode (string-keyed hash table in DAT payloads); ac2-script/src/data/enum_map.rs decodes the tables inline under EnumIDMap::StreamPack, verify and cite

### IntrusiveHashData<CaseInsensitiveStringBase<PStringBase<char>_>,HashTableData<CaseInsensitiveStringBase<PStringBase<char>_>,unsigned_long>_*>
- [-] `IntrusiveHashData<CaseInsensitiveStringBase<PStringBase<char>_>,HashTableData<CaseInsensitiveStringBase<PStringBase<char>_>,unsigned_long>_*>::IntrusiveHashData<CaseInsensitiveStringBase<PStringBase<char>_>,HashTableData<CaseInsensitiveStringBase<PStringBase<char>_>,unsigned_long>_*>` · 006d96e0 · legions 00662ab0 — constructor (Rust struct init)

### DBCategories
- [-] `DBCategories::DBCategories` · 006d9760 · legions 00662bb0 — C++ constructor/destructor (Rust struct init or Drop)
- [-] `DBCategories::GetDBOType` · 006d97a0 — DBObj class factory (Allocate/GetDBOType); Rust decodes into typed structs
- [-] `DBCategories::~DBCategories` · 006d97b0 · legions 00662c00 — C++ constructor/destructor (Rust struct init or Drop)
- [-] `DBCategories::CopyInto` · 006d9810 — DBObj class factory (Allocate/GetDBOType); Rust decodes into typed structs
- [-] `DBCategories::FromFileNode` · 006d9830 — text source format (admin tools)
- [-] `DBCategories::Allocate` · 006d9900 — DBObj class factory (Allocate/GetDBOType); Rust decodes into typed structs

## `portal/shared/src/database/dbobj.cpp.c` (3/21)

### CWBookKeeper
- [-] `CWBookKeeper::SetBookie` · 006d7690 · legions 006613b0 — trivial accessor or setter; the value is a field of the Rust struct (book keeper pointer wiring of the weenie)

### DBObj
- [ ] `DBObj::AddToDataGraph` · 006d76a0 · legions 006613c0 — needs port: verify whether AddToDataGraph matters outside tools (AGENTS open question); record finding in AGENTS
- [-] `DBObj::AddRef` · 006d76b0 · legions 006613d0 — retail DB cache internals (freelist, pool, idle eviction, reload); AGENTS says do not reproduce
- [-] `DBObj::Release` · 006d76e0 — retail DB cache internals (freelist, pool, idle eviction, reload); AGENTS says do not reproduce
- [-] `DBObj::ReloadFromDisk` · 006d7710 · legions 00661430 — retail DB cache internals (freelist, pool, idle eviction, reload); AGENTS says do not reproduce
- [-] `DBObj::SaveToDisk` · 006d7740 · legions 00661460 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `DBObj::GetPackSize` · 006d7780 · legions 006614a0 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `DBObj::Pack_Type` · 006d77b0 · legions 006614d0 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [~] `DBObj::UnPack_Type` · 006d7830 · legions 00661550 — unpack only, in ac2-net protocol::qualities / visual (Legions CBaseQualities::UnPack instruction-identical)
- [x] `DBObj::Get` · 006d7890 · legions 006615b0 — DID get of a typed DBObj; cited in ac2-script/src/vm/mod.rs and ac2-world/src/region/detail_map.rs
- [-] `DBObj::GetDBOCache` · 006d78b0 · legions 006615d0 — DBObj class factory (Allocate/GetDBOType); Rust decodes into typed structs

### FileArray
- [x] `FileArray::GetDIDFromEnum` · 006d78c0 · legions 00616cd0 — ac2-script data/glue::enums

### DBObj
- [-] `DBObj::Copy` · 006d78d0 · legions 006615e0 — retail DB cache internals (freelist, pool, idle eviction, reload); AGENTS says do not reproduce
- [-] `DBObj::Put` · 006d7910 · legions 00661620 — retail DB cache internals (freelist, pool, idle eviction, reload); AGENTS says do not reproduce
- [-] `DBObj::Remove` · 006d7940 · legions 00661650 — retail DB cache internals (freelist, pool, idle eviction, reload); AGENTS says do not reproduce
- [x] `DBObj::GetByEnum` · 006d7970 · legions 00661680 — enum/group get; cited in ac2-db/src/string_state.rs:26 and ac2-assets/src/ac2_appearance.rs:355
- [-] `DBObj::DBObj` · 006d79a0 · legions 006616b0 — C++ constructor/destructor (Rust struct init or Drop)
- [-] `DBObj::~DBObj` · 006d7a10 · legions 00661720 — C++ constructor/destructor (Rust struct init or Drop)
- [-] `DBObj::SaveToFile` · 006d7a60 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `DBObj::LoadFromFile` · 006d7b20 — text source loader, admin build only
- [ ] `DBObj::FillDataGraph` · 006d7bf0 — needs port: verify whether AddToDataGraph matters outside tools (AGENTS open question); record finding in AGENTS

## `portal/shared/src/database/dbocache.cpp.c` (0/21)

### DBOCache
- [-] `DBOCache::UseTime` · 006d9b40 · legions 00662f70 — ac2-db typed cache: free on last drop with a free-byte budget replaces DBOCache's free pool and 30 s idle aging (AGENTS.md exception)
- [-] `DBOCache::AddObj` · 006d9b80 — retail DB cache internals (freelist, pool, idle eviction, reload); AGENTS says do not reproduce
- [-] `DBOCache::FreeObject` · 006d9ba0 — retail DB cache internals (freelist, pool, idle eviction, reload); AGENTS says do not reproduce
- [-] `DBOCache::ReloadObject` · 006d9be0 — retail DB cache internals (freelist, pool, idle eviction, reload); AGENTS says do not reproduce
- [-] `DBOCache::FreelistAdd` · 006d9c00 — retail DB cache internals (freelist, pool, idle eviction, reload); AGENTS says do not reproduce
- [-] `DBOCache::FreelistRemove` · 006d9c70 — retail DB cache internals (freelist, pool, idle eviction, reload); AGENTS says do not reproduce
- [-] `DBOCache::FreelistRemoveOldest` · 006d9cc0 — retail DB cache internals (freelist, pool, idle eviction, reload); AGENTS says do not reproduce
- [-] `DBOCache::GetFreeObj` · 006d9ce0 · legions 00663120 — retail DB cache internals (freelist, pool, idle eviction, reload); AGENTS says do not reproduce

### SetPositionStruct
- [ ] `SetPositionStruct::SetFlags` · 006d9d20 · legions 00663160 — needs port: cite-only; SetPositionStruct flag set behind efun gSetPositionStruct::CallEFun_SetFlags (ac2-script glue efun_names.txt); verify the bit op and cite

### DBOCache
- [-] `DBOCache::FlushFreeObjects` · 006d9d30 · legions 00663170 — retail DB cache internals (freelist, pool, idle eviction, reload); AGENTS says do not reproduce
- [-] `DBOCache::KeepFreeObjects` · 006d9d90 · legions 006631d0 — retail DB cache internals (freelist, pool, idle eviction, reload); AGENTS says do not reproduce
- [-] `DBOCache::ReloadCache` · 006d9dc0 · legions 00663200 — retail DB cache internals (freelist, pool, idle eviction, reload); AGENTS says do not reproduce
- [-] `DBOCache::RemoveObj_Internal` · 006d9e70 — retail DB cache internals (freelist, pool, idle eviction, reload); AGENTS says do not reproduce
- [-] `DBOCache::DestroyObj` · 006d9eb0 — retail DB cache internals (freelist, pool, idle eviction, reload); AGENTS says do not reproduce
- [-] `DBOCache::RemoveObj` · 006d9f00 · legions 00663340 — retail DB cache internals (freelist, pool, idle eviction, reload); AGENTS says do not reproduce
- [ ] `DBOCache::GetIfInMemory` · 006d9f70 · legions 006633b0 — needs port: sub-object resolution, load dependent DIDs on first fetch (AGENTS item 1); verify ac2-db dependency plan and cite
- [-] `DBOCache::GetIfUsing` · 006da000 · legions 00663440 — retail DB cache internals (freelist, pool, idle eviction, reload); AGENTS says do not reproduce
- [-] `DBOCache::Release` · 006da060 — retail DB cache internals (freelist, pool, idle eviction, reload); AGENTS says do not reproduce
- [-] `DBOCache::~DBOCache` · 006da170 · legions 006635b0 — C++ destructor or operator (Rust Drop / value semantics)
- [-] `DBOCache::DBOCache` · 006da240 · legions 00663680 — constructor (Rust struct init)
- [-] `DBOCache::AddObj_Internal` · 006da2e0 — retail DB cache internals (freelist, pool, idle eviction, reload); AGENTS says do not reproduce

## `portal/shared/src/database/enuidmap.cpp.c` (4/15)

### EnumIDMap
- [x] `EnumIDMap::StreamPack` · 006d80b0 · legions 00661c30 — ac2-script data/glue::enums
- [x] `EnumIDMap::NameToEnum` · 006d8150 · legions 00661cd0 — ac2-script data/glue::enums
- [ ] `EnumIDMap::GetSubDataIDs` · 006d8280 · legions 00661e00 — needs port: sub-object resolution, an enum map's dependent data DIDs (AGENTS item 1 family); verify and cite

### SmartArray<PEnum>
- [-] `SmartArray<PEnum>::operator=` · 006d8380 · legions 00661f00 — STL/container template internals (Rust std collections)

### EnumIDMap
- [x] `EnumIDMap::EnumToDID` · 006d83f0 · legions 00661f70 — ac2-script data/glue::enums
- [x] `EnumIDMap::GetName` · 006d8450 · legions 00661fd0 — ac2-script data/glue::enums
- [-] `EnumIDMap::Destroy` · 006d8490 · legions 00662010 — DBObj class factory (Allocate/GetDBOType); Rust decodes into typed structs
- [ ] `EnumIDMap::GetAllEnumVals` · 006d84c0 · legions 00662040 — needs port: cite-only; enumerate enum values and names; ac2-script/src/data/enum_map.rs has GetName and EnumToDID, verify the list form and cite
- [-] `EnumIDMap::EnumIDMap` · 006d85f0 · legions 00662170 — constructor (Rust struct init)
- [-] `EnumIDMap::GetDBOType` · 006d86b0 — DBObj class factory (Allocate/GetDBOType); Rust decodes into typed structs
- [-] `EnumIDMap::~EnumIDMap` · 006d86c0 · legions 00662240 — C++ destructor or operator (Rust Drop / value semantics)
- [ ] `EnumIDMap::GetAllEnumNames` · 006d8800 · legions 00662380 — needs port: cite-only; enumerate enum values and names; ac2-script/src/data/enum_map.rs has GetName and EnumToDID, verify the list form and cite
- [-] `EnumIDMap::AddEmp` · 006d8920 — admin-only tooling (text source, preprocess, EMP or format authoring)
- [-] `EnumIDMap::Allocate` · 006d8b20 — DBObj class factory (Allocate/GetDBOType); Rust decodes into typed structs
- [-] `EnumIDMap::LoadFromFile` · 006d8b50 — text source loader, admin build only

## `portal/shared/src/database/masterdbmap.cpp.c` (0/4)

### MasterDBMap
- [ ] `MasterDBMap::DivineType_Internal` · 006d2530 · legions 0065c250 — needs port: cite-only; DB type of a DID by registered ranges; ac2-script/src/data/db_types.rs DivineType does it, cite the _Internal

### DBTypeDef
- [-] `DBTypeDef::DBTypeDef` · 006d2b70 · legions 0065c890 — constructor (Rust struct init)

### MasterDBMap
- [ ] `MasterDBMap::DivineType_Internal` · 006d2c10 · legions 0065c930 — needs port: cite-only; DB type of a DID by registered ranges; ac2-script/src/data/db_types.rs DivineType does it, cite the _Internal
- [ ] `MasterDBMap::InitDBTypeDef_Internal` · 006d4640 · legions 0065e360 — needs port: cite-only; registers the DBTypeDef DID ranges; ac2-script/src/data/db_types.rs holds the table, verify and cite

## `portal/shared/src/database/masterdbmapmain.cpp.c` (1/19)

### MasterDBMap
- [ ] `MasterDBMap::Init` · 006d6f80 · legions 00660ca0 — needs port: cite-only; master map init (type defs from the DAT); verify against ac2-script/src/data/db_types.rs and cite
- [-] `MasterDBMap::Cleanup` · 006d6fa0 · legions 00660cc0 — teardown of the master map (Rust statics/Drop)
- [ ] `MasterDBMap::DivineCellType` · 006d6fc0 · legions 00660ce0 — needs port: cell DB type classification by DID; not located in Rust, verify against ac2-world cell DIDs
- [~] `MasterDBMap::DivineType` · 006d7020 · legions 00660d40 — ac2-script: FileArray has no data in Legions (no READONLY_DIDName.dat); DivineType by DID range
- [~] `MasterDBMap::DivineType` · 006d7040 · legions 00660d60 — ac2-script: FileArray has no data in Legions (no READONLY_DIDName.dat); DivineType by DID range
- [-] `MasterDBMap::SetGameDataDir` · 006d70a0 · legions 00660dc0 — trivial accessor or setter; the value is a field of the Rust struct (data directory setup; ac2-dat paths.rs resolves the shipped DAT)
- [-] `MasterDBMap::SetEngineDataDir` · 006d70e0 · legions 00660e00 — trivial accessor or setter; the value is a field of the Rust struct (data directory setup; ac2-dat paths.rs resolves the shipped DAT)
- [ ] `MasterDBMap::GetDBTypeDef` · 006d71e0 · legions 00660f00 — needs port: cite-only; DBTypeDef lookup by type; ac2-script/src/data/db_types.rs, verify and cite
- [-] `MasterDBMap::HasFile` · 006d7260 · legions 00660f80 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [x] `MasterDBMap::GetBaseDID` · 006d7280 · legions 00660fa0 — base DID of a DB type; cited in ac2-script/src/wstate.rs:19
- [ ] `MasterDBMap::GetPackVer` · 006d7290 — needs port: verify whether DAT pack version gates the reader; no version check found in ac2-dat
- [-] `MasterDBMap::GetGameDataDir` · 006d72b0 · legions 00660fd0 — trivial accessor or setter; the value is a field of the Rust struct (data directory setup; ac2-dat paths.rs resolves the shipped DAT)
- [-] `MasterDBMap::GetEngineDataDir` · 006d72f0 · legions 00661010 — trivial accessor or setter; the value is a field of the Rust struct (data directory setup; ac2-dat paths.rs resolves the shipped DAT)
- [ ] `MasterDBMap::IsPortalType` · 006d7330 · legions 00661050 — needs port: cite-only; DB type class predicates; ac2-db/src/db_type.rs classifies DIDs, verify each predicate and cite
- [ ] `MasterDBMap::IsCellType` · 006d7350 · legions 00661070 — needs port: cite-only; DB type class predicates; ac2-db/src/db_type.rs classifies DIDs, verify each predicate and cite
- [ ] `MasterDBMap::IsLocalType` · 006d7370 · legions 00661090 — needs port: cite-only; DB type class predicates; ac2-db/src/db_type.rs classifies DIDs, verify each predicate and cite
- [ ] `MasterDBMap::IsCategorized` · 006d7390 · legions 006610b0 — needs port: cite-only; DB type class predicates; ac2-db/src/db_type.rs classifies DIDs, verify each predicate and cite
- [-] `MasterDBMap::GetFreelistDef` · 006d73b0 · legions 006610d0 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `MasterDBMap::VerifyDirectory` · 006d73d0 · legions 006610f0 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)

## `portal/shared/src/database/qualifieddataid.cpp.c` (4/6)

### QualifiedDataID
- [-] `QualifiedDataID::QualifiedDataID` · 006d7ce0 · legions 00661860 — constructor (Rust struct init)

### QualifiedDataIDArray
- [-] `QualifiedDataIDArray::operator[]` · 006d7d10 · legions 00661890 — C++ destructor or operator (Rust Drop / value semantics)
- [x] `QualifiedDataIDArray::AddToEnd` · 006d7f30 · legions 00661ab0 — DID set add of a non-zero DID; cited in ac2-script/src/glue/enums.rs:280
- [x] `QualifiedDataIDArray::AddToEnd` · 006d7f60 · legions 00661ae0 — DID set add of a non-zero DID; cited in ac2-script/src/glue/enums.rs:280
- [x] `QualifiedDataIDArray::AddToEnd` · 006d7fe0 · legions 00661b60 — ac2-script glue::enums::add_data_id
- [x] `QualifiedDataIDArray::AddToEnd` · 006d8040 · legions 00661bc0 — DID set add of a non-zero DID; cited in ac2-script/src/glue/enums.rs:280

## `portal/shared/src/database/threadedcache.cpp.c` (0/16)

### DBCache
- [-] `DBCache::SetShutdown` · 006cf490 · legions 006590b0 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)

### ThreadedCache
- [-] `ThreadedCache::WorkerExecuteRequest` · 006cf4b0 · legions 006590d0 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)
- [-] `ThreadedCache::SetShutdown` · 006cf4f0 · legions 00659110 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)
- [-] `ThreadedCache::StopThread` · 006cf520 · legions 00659140 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)
- [-] `ThreadedCache::WakeForTheReaper` · 006cf540 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)
- [-] `ThreadedCache::WorkerExecuteGetRequest` · 006cf550 · legions 00659170 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)
- [-] `ThreadedCache::WorkerExecuteSaveRequest` · 006cf580 · legions 006591a0 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)
- [-] `ThreadedCache::WorkerExecutePurgeRequest` · 006cf5d0 · legions 006591f0 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)

### LFQueue<AsyncCache::CCallbackHandler_*>
- [-] `LFQueue<AsyncCache::CCallbackHandler_*>::Consume` · 006cf600 · legions 00659220 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)

### ThreadedCache
- [ ] `ThreadedCache::UseTime` · 006cf6d0 · legions 006592f0 — needs port: main-thread drain of finished async requests (AGENTS item 2); verify the scene pump covers it and cite
- [-] `ThreadedCache::ThreadedCache` · 006cf750 · legions 006594d0 — constructor (Rust struct init)

### DIDName_Dat
- [-] `DIDName_Dat::RequestMappings` · 006cf830 · legions 006595b0 — DID name table; READONLY_DIDName.dat is not shipped (ac2-script/src/data/mod.rs:17)

### ThreadedCache
- [-] `ThreadedCache::Startup` · 006cf850 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)
- [-] `ThreadedCache::PutOnWorkerThreadJobQueue` · 006cf8d0 · legions 00659650 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)
- [-] `ThreadedCache::AsyncGetInternal` · 006cf900 · legions 00659680 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)
- [-] `ThreadedCache::EnqueueAsyncRequest` · 006cf9d0 · legions 00659750 — retail async request machinery; ac2-db typed cache (in-flight get registry, Arc sharing, worker lanes) replaces it (AGENTS)

## `portal/shared/src/datasrc/datasrc.cpp.c` (0/4)

### DataSource
- [-] `DataSource::DataSource` · 00736510 · legions 006b2080 — C++ constructor/destructor (Rust struct init or Drop)
- [-] `DataSource::Open` · 00736530 · legions 006b20a0 — Win32 file handle or DataSource abstraction; Rust std::fs in ac2-dat
- [-] `DataSource::Close` · 00736550 · legions 006b20c0 — Win32 file handle or DataSource abstraction; Rust std::fs in ac2-dat
- [-] `DataSource::~DataSource` · 00736560 · legions 006b20d0 — C++ constructor/destructor (Rust struct init or Drop)

## `portal/shared/src/datasrc/filesrc.cpp.c` (0/4)

### FileSource
- [-] `FileSource::FileSource` · 00736260 · legions 006b1f00 — C++ constructor/destructor (Rust struct init or Drop)
- [-] `FileSource::OpenFile` · 00736280 · legions 006b1f20 — Win32 file handle or DataSource abstraction; Rust std::fs in ac2-dat
- [-] `FileSource::Close` · 00736310 · legions 006b1fb0 — Win32 file handle or DataSource abstraction; Rust std::fs in ac2-dat
- [-] `FileSource::~FileSource` · 00736360 · legions 006b2000 — C++ constructor/destructor (Rust struct init or Drop)

## `portal/shared/src/datasrc/outfilesrc.cpp.c` (0/3)

### OutFileSource
- [-] `OutFileSource::OutFileSource` · 007363e0 — C++ constructor/destructor (Rust struct init or Drop)
- [-] `OutFileSource::~OutFileSource` · 00736400 — C++ constructor/destructor (Rust struct init or Drop)
- [-] `OutFileSource::OpenFile` · 00736460 — Win32 file handle or DataSource abstraction; Rust std::fs in ac2-dat

## `portal/shared/src/datcompat/datidstamp.cpp.c` (0/12)

### DatIDStamp
- [-] `DatIDStamp::operator=` · 0060cf60 · legions 006b2200 — STL/container template internals (Rust std collections)
- [-] `DatIDStamp::Clear` · 0060cf90 · legions 005cdd60 — trivial accessor or setter; the value is a field of the Rust struct
- [-] `DatIDStamp::GenerateNewMajorVersion` · 0060cfb0 · legions 005cdd80 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `DatIDStamp::GenerateNewMinorVersion` · 0060cfd0 · legions 005cdda0 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `DatIDStamp::DatIDStamp` · 0060cfe0 · legions 005cddb0 — C++ constructor/destructor (Rust struct init or Drop)
- [-] `DatIDStamp::DatIDStamp` · 0060d000 · legions 005cddd0 — C++ constructor/destructor (Rust struct init or Drop)
- [-] `DatIDStamp::FromString` · 0060d040 · legions 005cde10 — admin-only tooling (text source, preprocess, EMP or format authoring)
- [-] `DatIDStamp::ToString` · 0060d0a0 · legions 005cde70 — text form of the stamp for tools/logs
- [-] `DatIDStamp::pack_size` · 0060d150 · legions 005cdf20 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `DatIDStamp::GetPackSize` · 0060d1c0 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `DatIDStamp::Pack` · 0060d1d0 · legions 005cdfa0 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [ ] `DatIDStamp::UnPack` · 0060d230 — needs port: cite-only; DAT id stamp read from the header; entry.rs DatIdStamp and btree.rs parse the stamp, verify and cite

## `portal/shared/src/diskcon/blkalloc.cpp.c` (1/14)

### CLBlockAllocator
- [-] `CLBlockAllocator::CLBlockAllocator` · 0064e6e0 · legions 005fda10 — C++ constructor/destructor (Rust struct init or Drop)
- [-] `CLBlockAllocator::~CLBlockAllocator` · 0064e700 · legions 005fda30 — C++ constructor/destructor (Rust struct init or Drop)
- [-] `CLBlockAllocator::Close_Data_File` · 0064e710 · legions 005fda40 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `CLBlockAllocator::SaveFileInfo` · 0064e730 · legions 005fda60 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `CLBlockAllocator::Store_Data` · 0064e750 · legions 005fda80 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `CLBlockAllocator::StoreDataRollback` · 0064e870 · legions 005fdba0 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [x] `CLBlockAllocator::Load_Data` · 0064e930 · legions 005fdc60 — block-chain record read; cited in ac2-dat/src/file_btree_runtime.rs:372,400
- [-] `CLBlockAllocator::DeleteBlocks` · 0064e9e0 · legions 005fdd10 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `CLBlockAllocator::ExpandFile` · 0064eac0 · legions 005fddf0 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `CLBlockAllocator::SaveTransaction` · 0064ebd0 · legions 005fdf00 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `CLBlockAllocator::ClearTransaction` · 0064ebf0 · legions 005fdf20 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `CLBlockAllocator::CreateDataFile` · 0064ec40 · legions 005fdf70 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `CLBlockAllocator::OpenDataFile` · 0064edd0 · legions 005fe100 — opens the DAT for writing; the reader opens read-only (ac2-dat file_btree_runtime.rs open)
- [-] `CLBlockAllocator::Update` · 0064ee50 · legions 005fe180 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)

## `portal/shared/src/diskcon/btree.cpp.c` (5/44)

### BTEntry
- [-] `BTEntry::`default_constructor_closure'` · 0064c6f0 — C++ constructor/destructor (Rust struct init or Drop)

### BTree
- [ ] `BTree::HasEntry` · 0064c700 · legions 005fba30 — needs port: cite-only; B-tree lookup (DID presence, node find, min/max); file_btree_runtime.rs find_in_node and DatDataset::contains do it, verify each and cite
- [-] `BTree::ExtractEntryShift` · 0064c780 · legions 005fbab0 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `BTree::RemoveLeast` · 0064c820 · legions 005fbb50 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `BTree::AddMost` · 0064c8c0 · legions 005fbbf0 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `BTree::AddLeast` · 0064c900 · legions 005fbc30 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `BTree::Restamp_Entry` · 0064c970 · legions 005fbca0 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)

### BTMemNode
- [-] `BTMemNode::BTMemNode` · 0064c990 · legions 005fbcc0 — C++ constructor/destructor (Rust struct init or Drop)

### BTree
- [-] `BTree::BTree` · 0064c9d0 · legions 005fbd00 — C++ constructor/destructor (Rust struct init or Drop)
- [-] `BTree::DestroyTree` · 0064ca10 · legions 005fc030 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `BTree::CreateTree` · 0064ca60 · legions 005fbd40 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [x] `BTree::LoadTree` · 0064cbb0 · legions 005fbe90 — root read at open; ac2-dat/src/file_btree_runtime.rs open (cited in tests/file_btree_profile.rs:19)
- [-] `BTree::UpdateNode` · 0064ccf0 · legions 005fbfd0 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `BTree::SaveDataToFile` · 0064cd50 · legions 005fc080 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [ ] `BTree::FindNode` · 0064ce30 · legions 005fc160 — needs port: cite-only; B-tree lookup (DID presence, node find, min/max); file_btree_runtime.rs find_in_node and DatDataset::contains do it, verify each and cite
- [-] `BTree::AllocateEmptyNode` · 0064cfc0 · legions 005fc2f0 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `BTree::SplitNodeExec` · 0064d030 · legions 005fc360 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `BTree::InsertEntryExec` · 0064d230 · legions 005fc560 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `BTree::UpdateObjectExec` · 0064d370 · legions 005fc6a0 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `BTree::MergeNodesExec` · 0064d450 · legions 005fc780 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `BTree::MergeNodesTrans` · 0064d6a0 · legions 005fc9d0 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [ ] `BTree::FindMin` · 0064d730 · legions 005fca60 — needs port: cite-only; B-tree lookup (DID presence, node find, min/max); file_btree_runtime.rs find_in_node and DatDataset::contains do it, verify each and cite
- [ ] `BTree::FindMax` · 0064d760 · legions 005fca90 — needs port: cite-only; B-tree lookup (DID presence, node find, min/max); file_btree_runtime.rs find_in_node and DatDataset::contains do it, verify each and cite
- [-] `BTree::DeleteLeafExec` · 0064d7a0 · legions 005fcad0 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `BTree::DeleteLeafTrans` · 0064d850 · legions 005fcb80 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `BTree::DeleteInternalExec` · 0064d8d0 · legions 005fcc00 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `BTree::RotateEntryExec` · 0064d9e0 · legions 005fcd10 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `BTree::RecoverTransaction` · 0064db80 · legions 005fceb0 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [ ] `BTree::Search` · 0064dc60 · legions 005fcf90 — needs port: cite-only; B-tree lookup (DID presence, node find, min/max); file_btree_runtime.rs find_in_node and DatDataset::contains do it, verify each and cite
- [-] `BTree::SplitNodeTrans` · 0064dce0 · legions 005fd010 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `BTree::InsertEntryTrans` · 0064dd70 · legions 005fd0a0 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `BTree::Update_Data_Trans` · 0064de00 · legions 005fd130 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `BTree::DescendToAdd` · 0064dea0 · legions 005fd1d0 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `BTree::DeleteInternalTrans` · 0064dfc0 · legions 005fd2f0 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `BTree::RotateEntryTrans` · 0064e040 · legions 005fd370 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [ ] `BTree::Lookup` · 0064e0d0 · legions 005fd400 — needs port: cite-only; B-tree lookup (DID presence, node find, min/max); file_btree_runtime.rs find_in_node and DatDataset::contains do it, verify each and cite
- [-] `BTree::DescendToDelete` · 0064e100 · legions 005fd430 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [x] `BTree::Build_Id_List` · 0064e280 · legions 005fd5b0 — ac2-dat FileBtreeRuntime::collect_entries_in_range: pruned walk through the pinned root + 100-node cache; emits ascending DIDs (retail appends a node's entry before its left child)
- [x] `BTree::Build_Id_List` · 0064e380 · legions 005fd6b0 — ac2-dat FileBtreeRuntime::collect_entries_in_range: pruned walk through the pinned root + 100-node cache; emits ascending DIDs (retail appends a node's entry before its left child)
- [-] `BTree::Delete` · 0064e470 · legions 005fd7a0 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `BTree::Remove` · 0064e620 · legions 005fd950 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [x] `BTree::Get_Id_List` · 0064e640 · legions 005fd970 — ac2-dat FileBtreeRuntime::collect_entries_in_range: pruned walk through the pinned root + 100-node cache; emits ascending DIDs (retail appends a node's entry before its left child)
- [x] `BTree::Get_Id_List_Range` · 0064e660 · legions 005fd990 — ac2-dat FileBtreeRuntime::collect_entries_in_range: pruned walk through the pinned root + 100-node cache; emits ascending DIDs (retail appends a node's entry before its left child)
- [-] `BTree::Try_Delete_Oldest` · 0064e6a0 · legions 005fd9d0 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)

## `portal/shared/src/diskcon/diskcon.cpp.c` (0/19)

### DiskFileInfo_t
- [-] `DiskFileInfo_t::ClearData` · 0064b700 · legions 005faa30 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)

### DiskConBase
- [ ] `DiskConBase::GetDatIDStamp` · 0064b740 · legions 005faa70 — needs port: cite-only; DAT metadata read (stamp, size, presence) covered by ac2-dat DatFileMeta/DatEntryMeta and DatDataset::contains, verify and cite

### DiskController
- [-] `DiskController::Close` · 0064b780 · legions 005faab0 — close of the DAT handle (Rust Drop)
- [-] `DiskController::CheckRoom` · 0064b7d0 · legions 005fab00 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [ ] `DiskController::GetDatestamp` · 0064b830 · legions 005fab60 — needs port: cite-only; DAT metadata read (stamp, size, presence) covered by ac2-dat DatFileMeta/DatEntryMeta and DatDataset::contains, verify and cite
- [ ] `DiskController::GetDataSize` · 0064b860 · legions 005fab90 — needs port: cite-only; DAT metadata read (stamp, size, presence) covered by ac2-dat DatFileMeta/DatEntryMeta and DatDataset::contains, verify and cite
- [ ] `DiskController::LoadData` · 0064b890 · legions 005fabc0 — needs port: cite-only; record read wrapper of LoadDataEx (ac2-dat file.rs read_record is cited under LoadDataEx); verify and cite
- [ ] `DiskController::IsMember` · 0064b940 · legions 005fac70 — needs port: cite-only; DAT metadata read (stamp, size, presence) covered by ac2-dat DatFileMeta/DatEntryMeta and DatDataset::contains, verify and cite
- [-] `DiskController::DeleteData` · 0064b980 · legions 005facb0 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `DiskController::SetMasterMapDID` · 0064b9c0 · legions 005facf0 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `DiskController::SetDatIDStamp` · 0064ba00 · legions 005fad30 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)

### DiskFileInfo_t
- [-] `DiskFileInfo_t::DiskFileInfo_t` · 0064ba30 · legions 005fad60 — C++ constructor/destructor (Rust struct init or Drop)

### DiskController
- [-] `DiskController::SaveData` · 0064ba80 · legions 005fadb0 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `DiskController::DeleteDataByMask` · 0064bb10 · legions 005fae40 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [~] `DiskController::LoadIterationList` · 0064bc00 · legions 005faf30 — history read via ac2-dat read_packed(0xFFFF0001) + ac2-net read_int_set in ac2-weenie::dat_iterations
- [-] `DiskController::SaveIterationList` · 0064bc80 · legions 005fafb0 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `DiskController::DiskController` · 0064bd10 · legions 005fb040 — C++ constructor/destructor (Rust struct init or Drop)
- [-] `DiskController::~DiskController` · 0064bd70 · legions 005fb0a0 — C++ constructor/destructor (Rust struct init or Drop)
- [-] `DiskController::InitFile` · 0064be10 · legions 005fb140 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)

## `portal/shared/src/diskcon/diskdev.cpp.c` (1/8)

### DiskDev
- [-] `DiskDev::DiskDev` · 00650480 · legions 005ff750 — C++ constructor/destructor (Rust struct init or Drop)

### DiskFileInfo_t
- [-] `DiskFileInfo_t::operator=` · 00650490 · legions 005ff760 — STL/container template internals (Rust std collections)

### DiskDev
- [-] `DiskDev::Close` · 00650510 · legions 005ff7e0 — Win32 file handle or DataSource abstraction; Rust std::fs in ac2-dat

### DiskHeaderBlock_t
- [-] `DiskHeaderBlock_t::DiskHeaderBlock_t` · 00650540 · legions 005ff810 — C++ constructor/destructor (Rust struct init or Drop)

### DiskDev
- [-] `DiskDev::~DiskDev` · 006505c0 · legions 005ff890 — C++ constructor/destructor (Rust struct init or Drop)
- [x] `DiskDev::SyncRead` · 006505e0 · legions 005ff8b0 — positioned reads; ac2-dat/src/file.rs:14 and file_btree_runtime.rs
- [-] `DiskDev::SyncWrite` · 00650640 · legions 005ff910 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `DiskDev::Open_File` · 006506a0 · legions 005ff970 — Win32 file handle or DataSource abstraction; Rust std::fs in ac2-dat

## `portal/shared/src/diskcon/iterationlist.cpp.c` (2/5)

### (free functions)
- [-] `CompareInts` · 0064c060 — qsort comparator for an int array; Rust sorts with std

### CMostlyConsecutiveIntSet
- [-] `CMostlyConsecutiveIntSet::CMostlyConsecutiveIntSet` · 0064c090 · legions 005fb3c0 — C++ constructor/destructor (Rust struct init or Drop)
- [x] `CMostlyConsecutiveIntSet::Sort` · 0064c0e0 · legions 005fb410
- [x] `CMostlyConsecutiveIntSet::StreamPack` · 0064c170
- [-] `CMostlyConsecutiveIntSet::Add` · 0064c370 · legions 005fb6a0 — int-set insert used by DAT writers; the reader's id sets are parsed by ac2-net protocol/dat_event.rs

## `portal/shared/src/diskcon/lookfile.cpp.c` (0/3)

### LookFile
- [-] `LookFile::GetFoundPath` · 0064c3b0 · legions 005fb6e0 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `LookFile::IsFileHere` · 0064c3d0 · legions 005fb700 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `LookFile::LookForFile` · 0064c4a0 · legions 005fb7d0 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)

## `portal/shared/src/diskcon/lrulist.cpp.c` (0/21)

### DLListBase
- [-] `DLListBase::Remove` · 0064efe0 · legions 00430630 — STL/container template internals (Rust std collections)

### LRUB_Mem_t
- [-] `LRUB_Mem_t::LRUB_Mem_t` · 0064f030 · legions 005fe310 — C++ constructor/destructor (Rust struct init or Drop)
- [-] `LRUB_Mem_t::Make_Buffer` · 0064f0b0 · legions 005fe390 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `LRUB_Mem_t::Remove_Below` · 0064f110 · legions 005fe3f0 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `LRUB_Mem_t::Remove_Above` · 0064f150 · legions 005fe430 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)

### LRU_List
- [-] `LRU_List::Create_List` · 0064f190 · legions 005fe470 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)

### LRUB_Mem_t
- [-] `LRUB_Mem_t::LRUB_Mem_t` · 0064f1d0 · legions 005fe4b0 — C++ constructor/destructor (Rust struct init or Drop)

### LRU_List
- [-] `LRU_List::Load_List` · 0064f210 · legions 005fe4f0 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `LRU_List::Add_Exec` · 0064f360 · legions 005fe640 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `LRU_List::Expand_Exec` · 0064f3f0 · legions 005fe6d0 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `LRU_List::Delete_Exec` · 0064f500 · legions 005fe7e0 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `LRU_List::Expand_Trans` · 0064f650 · legions 005fe960 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `LRU_List::Delete_Trans` · 0064f760 · legions 005fea70 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `LRU_List::LRU_List` · 0064f970 · legions 005fec40 — C++ constructor/destructor (Rust struct init or Drop)
- [-] `LRU_List::Mark_Used` · 0064fa60 · legions 005fed30 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `LRU_List::Mark_Deleted` · 0064fb70 · legions 005fee40 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `LRU_List::Write_Added` · 0064fc00 · legions 005feed0 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `LRU_List::Purge_Duplicates` · 0064fc60 · legions 005fef30 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `LRU_List::Get_Oldest` · 0064fee0 · legions 005ff1b0 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `LRU_List::~LRU_List` · 006500f0 · legions 005ff3c0 — C++ constructor/destructor (Rust struct init or Drop)
- [-] `LRU_List::Write_All_Out` · 00650340 · legions 005ff610 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)

## `portal/shared/src/filedb/dbfile2idtable.cpp.c` (0/6)

### DBFile2IDTable
- [-] `DBFile2IDTable::GetPackSize` · 0066d4d0 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)

### CaseInsensitiveStringBase<PStringBase<char>_>
- [-] `CaseInsensitiveStringBase<PStringBase<char>_>::case_insensitive_hash` · 0066d4f0 · legions 00617cf0 — STL/container template internals (Rust std collections)

### TFileEntry
- [-] `TFileEntry::GetPath` · 0066d540 · legions 00617d40 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)

### ShortcutInfo
- [-] `ShortcutInfo::GetDataStr` · 0066d560 · legions 00617d60 — admin-only tooling (text source, preprocess, EMP or format authoring)

### DBFile2IDTable
- [-] `DBFile2IDTable::Pack` · 0066d690 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [ ] `DBFile2IDTable::UnPack` · 0066d910 — needs port: cite-only; file-name to DID table decode that FileArray::GetDID uses; verify the decoder and cite

## `portal/shared/src/filedb/didnamedat.cpp.c` (0/17)

### DIDName_Dat
- [-] `DIDName_Dat::RequestClearAll` · 00670d70 · legions 0061b440 — DID name table; READONLY_DIDName.dat is not shipped (ac2-script/src/data/mod.rs:17)
- [-] `DIDName_Dat::SetTypeData` · 00670d80 · legions 0061b450 — DID name table; READONLY_DIDName.dat is not shipped (ac2-script/src/data/mod.rs:17)
- [-] `DIDName_Dat::OnDone` · 00670d90 · legions 0061b460 — DID name table; READONLY_DIDName.dat is not shipped (ac2-script/src/data/mod.rs:17)
- [-] `DIDName_Dat::WriteToDIDVersionLog` · 00670db0 · legions 0061b480 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `DIDName_Dat::Cleanup` · 00670e10 · legions 0061b4e0 — DID name table; READONLY_DIDName.dat is not shipped (ac2-script/src/data/mod.rs:17)
- [-] `DIDName_Dat::InitFromFilename` · 00670e40 · legions 0061b510 — DID name table; READONLY_DIDName.dat is not shipped (ac2-script/src/data/mod.rs:17)
- [-] `DIDName_Dat::TestForExistance` · 00670ef0 · legions 0061b5c0 — DID name table; READONLY_DIDName.dat is not shipped (ac2-script/src/data/mod.rs:17)
- [-] `DIDName_Dat::RequestAllTypeData` · 00670f30 · legions 0061b600 — DID name table; READONLY_DIDName.dat is not shipped (ac2-script/src/data/mod.rs:17)
- [-] `DIDName_Dat::RequestTypeNumber` · 00671010 · legions 0061b6e0 — DID name table; READONLY_DIDName.dat is not shipped (ac2-script/src/data/mod.rs:17)
- [-] `DIDName_Dat::RequestMapping` · 00671100 · legions 0061b7d0 — DID name table; READONLY_DIDName.dat is not shipped (ac2-script/src/data/mod.rs:17)

### DBOCache
- [-] `DBOCache::SetCollection` · 00671190 · legions 0061b860 — retail DB cache internals (freelist, pool, idle eviction, reload); AGENTS says do not reproduce

### DIDName_Dat
- [-] `DIDName_Dat::DIDName_Dat` · 006711a0 · legions 0061b870 — C++ constructor/destructor (Rust struct init or Drop)
- [-] `DIDName_Dat::~DIDName_Dat` · 00671210 · legions 0061b8e0 — C++ constructor/destructor (Rust struct init or Drop)
- [-] `DIDName_Dat::Read` · 00671280 · legions 0061b950 — DID name table; READONLY_DIDName.dat is not shipped (ac2-script/src/data/mod.rs:17)
- [-] `DIDName_Dat::Save` · 006713c0 · legions 0061ba90 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `DIDName_Dat::RequestAllMappings` · 00671a80 · legions 0061c150 — DID name table; READONLY_DIDName.dat is not shipped (ac2-script/src/data/mod.rs:17)
- [-] `DIDName_Dat::GetDescriptionString` · 00671c70 · legions 0061c340 — DID name table; READONLY_DIDName.dat is not shipped (ac2-script/src/data/mod.rs:17)

## `portal/shared/src/filedb/file2idtable.cpp.c` (0/33)

### TFile2IDTable
- [-] `TFile2IDTable::GetFileWriteTimeByDID` · 0066da60 · legions 00618150 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `TFile2IDTable::GetFileWriteTimeByEntry` · 0066dab0 · legions 006181a0 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `TFile2IDTable::GetHighestDID` · 0066dad0 · legions 006181c0 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)

### TFileEntry
- [-] `TFileEntry::SetPath` · 0066dbb0 · legions 006182a0 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `TFileEntry::~TFileEntry` · 0066dc10 · legions 00618300 — C++ constructor/destructor (Rust struct init or Drop)
- [-] `TFileEntry::TFileEntry` · 0066dc80 · legions 00618370 — C++ constructor/destructor (Rust struct init or Drop)

### TFile2IDTable
- [-] `TFile2IDTable::GetFileNameByDID` · 0066dd10 · legions 00618400 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `TFile2IDTable::GetFullPathByDID` · 0066dda0 · legions 00618490 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `TFile2IDTable::GetFileWriteTimeByFileName` · 0066de30 · legions 00618520 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `TFile2IDTable::GetDBTypeString` · 0066ded0 · legions 006185c0 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `TFile2IDTable::ReformatDirectoryName` · 0066df20 · legions 00618610 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `TFile2IDTable::RegisterDBType` · 0066dfc0 · legions 00618720 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `TFile2IDTable::SanitizeFileName` · 0066e210 · legions 00618970 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `TFile2IDTable::GetDIDByFileName` · 0066e2a0 · legions 00618a00 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `TFile2IDTable::GetFullPathByFileName` · 0066e3a0 · legions 00618b00 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `TFile2IDTable::GetFullPathByEntry` · 0066e4e0 · legions 00618c40 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `TFile2IDTable::GetDBTypes` · 0066e5d0 · legions 00618d30 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `TFile2IDTable::GetDIDs` · 0066e660 · legions 00618dc0 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `TFile2IDTable::AssignSoonToExistFilePath` · 0066e720 · legions 00618e80 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `TFile2IDTable::IsDBTypeRegistered` · 0066e890 · legions 00618ff0 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `TFile2IDTable::IsDBDirSearched` · 0066e8d0 · legions 00619030 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `TFile2IDTable::FindEntryByDID` · 0066e910 · legions 00619070 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)

### TDBTypeEntry
- [-] `TDBTypeEntry::~TDBTypeEntry` · 0066ea80 · legions 00619140 — C++ constructor/destructor (Rust struct init or Drop)

### TFile2IDTable
- [-] `TFile2IDTable::GetFilePaths` · 0066eb10 · legions 006191d0 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `TFile2IDTable::Clear` · 0066ef40 · legions 00619560 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)

### TDBTypeEntry
- [-] `TDBTypeEntry::TDBTypeEntry` · 0066f140 · legions 00619850 — C++ constructor/destructor (Rust struct init or Drop)

### TFile2IDTable
- [-] `TFile2IDTable::TFile2IDTable` · 0066f1a0 · legions 006198b0 — C++ constructor/destructor (Rust struct init or Drop)
- [-] `TFile2IDTable::~TFile2IDTable` · 0066f230 · legions 00619940 — C++ constructor/destructor (Rust struct init or Drop)
- [-] `TFile2IDTable::AssignDIDToFileName` · 0066f2b0 · legions 006199c0 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `TFile2IDTable::AssignNonAuthoritativePathToFileName` · 0066f4b0 · legions 00619bc0 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `TFile2IDTable::GetDBTypeEntry` · 0066f6f0 · legions 00619e00 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `TFile2IDTable::FindEntryByName` · 0066f810 · legions 00619f20 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `TFile2IDTable::AssignPathToFileNameInternal` · 0066f9a0 · legions 0061a0b0 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)

## `portal/shared/src/filedb/filearray.cpp.c` (5/25)

### FileArray
- [-] `FileArray::IsActive` · 0066c410 · legions 00616cc0 — trivial accessor or setter; the value is a field of the Rust struct
- [-] `FileArray::SetUsesDirs` · 0066c420 · legions 00616ce0 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `FileArray::SetDIDAllocOk` · 0066c440 · legions 00616d00 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)

### FileArrayFiller
- [-] `FileArrayFiller::ProcessMapping` · 0066c450 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)

### FileArray
- [ ] `FileArray::ReadIDs` · 0066c4d0 · legions 00616d90 — needs port: cite-only; reads the id tables (DBFile2IDTable) from the DAT; same gap as DBFile2IDTable::UnPack
- [-] `FileArray::WriteMasterIDs` · 0066c570 · legions 00616e30 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `FileArray::Cleanup` · 0066c5d0 · legions 00616e90 — C++ constructor/destructor (Rust struct init or Drop)
- [-] `FileArray::VerifyDBOTypes` · 0066c630 · legions 00616ef0 — admin-only tooling (text source, preprocess, EMP or format authoring)
- [-] `FileArray::GenerateDescription` · 0066c820 · legions 006170e0 — admin-only tooling (text source, preprocess, EMP or format authoring)
- [ ] `FileArray::RegisterDBType` · 0066c8c0 · legions 00617180 — needs port: cite-only; DB type registration for FileArray (DBTypeDef ranges); ac2-script/src/data/db_types.rs, verify and cite
- [-] `FileArray::GetAbsoluteFilePath` · 0066ca40 · legions 00617300 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `FileArray::GetAbsoluteFilePathForDummies` · 0066cb40 · legions 00617400 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [x] `FileArray::GetFileName` · 0066cb80 · legions 00617440 — DID to data file name; cited in ac2-script/src/vm/mod.rs:162
- [ ] `FileArray::GetFileNameForDummies` · 0066cc70 · legions 00617530 — needs port: cite-only; ANSI DID to file name; W variant cited in ac2-script glue enums.rs:260, verify and cite
- [x] `FileArray::GetFileNameForDummiesW` · 0066ccb0 · legions 00617570 — DID to wide file name; cited in ac2-script/src/glue/enums.rs:260
- [-] `FileArray::GetAbsoluteFilePath` · 0066cd20 · legions 006175e0 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)

### FileArrayFiller
- [-] `FileArrayFiller::ProcessTypeData` · 0066ce40 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)

### FileArray
- [x] `FileArray::GetDID` · 0066ce60 · legions 00617720 — name/enum to DID; cited in ac2-script/src/vm/mod.rs:147
- [x] `FileArray::GetDIDW` · 0066d050 · legions 00617910 — wide name to DID; cited in ac2-script/src/glue/enums.rs:250
- [ ] `FileArray::GetDIDForDummies` · 0066d0a0 · legions 00617960 — needs port: cite-only; ANSI name to DID with type; the W variant is cited in ac2-script glue enums.rs:239, verify and cite
- [x] `FileArray::GetDIDForDummiesW` · 0066d150 · legions 00617a10 — wide name to DID with type; cited in ac2-script/src/glue/enums.rs:239
- [ ] `FileArray::GetPhysicsObjectDID` · 0066d1a0 — needs port: physics object DID lookup by name/type; not located in Rust, verify against ac2-physics
- [-] `FileArray::GetFilePathForDummies` · 0066d260 · legions 00617a60 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `FileArray::AddIDTableStorage` · 0066d280 · legions 00617a80 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [ ] `FileArray::Init` · 0066d2e0 · legions 00617ae0 — needs port: cite-only; FileArray init loads the master and DAT id tables (DBFile2IDTable); verify against ac2-script data name lookup and cite

## `portal/shared/src/filedb/filefinder.cpp.c` (0/17)

### TFileFinder
- [-] `TFileFinder::TFileFinder` · 0066fca0 · legions 0061a370 — C++ constructor/destructor (Rust struct init or Drop)
- [-] `TFileFinder::~TFileFinder` · 0066fd10 · legions 0061a3e0 — C++ constructor/destructor (Rust struct init or Drop)
- [-] `TFileFinder::SetUsesDirs` · 0066fd20 · legions 0061a3f0 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `TFileFinder::RecurseInto` · 0066fd30 · legions 0061a400 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `TFileFinder::SearchForFileRecursive` · 0066ffa0 · legions 0061a670 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `TFileFinder::PowerOnHashes` · 00670390 · legions 0061aa60 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `TFileFinder::SearchDBTypeDirs` · 006703e0 · legions 0061aab0 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `TFileFinder::SearchForFile` · 006704e0 · legions 0061abb0 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `TFileFinder::FileExists` · 006705c0 · legions 0061ac90 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `TFileFinder::AssignDIDToFileName` · 00670910 · legions 0061afe0 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `TFileFinder::AssignNonAuthoritativePathToFileName` · 006709a0 · legions 0061b070 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `TFileFinder::AssignSoonToExistFilePath` · 00670a60 · legions 0061b130 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `TFileFinder::AssignPathToFileNameInternal` · 00670af0 · legions 0061b1c0 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `TFileFinder::GetFullPathByFileName` · 00670bb0 · legions 0061b280 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `TFileFinder::GetFileWriteTimeByFileName` · 00670c40 · legions 0061b310 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `TFileFinder::GetFullPathByEntry` · 00670cd0 · legions 0061b3a0 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `TFileFinder::GetFileWriteTimeByEntry` · 00670d20 · legions 0061b3f0 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)

## `portal/shared/src/filedb/filepathcache.cpp.c` (0/4)

### PHashTable<PStringBase<char>,PStringBase<char>_>
- [-] `PHashTable<PStringBase<char>,PStringBase<char>_>::PHashTable<PStringBase<char>,PStringBase<char>_>` · 00672020 · legions 0061c5e0 — constructor (Rust struct init)
- [ ] `PHashTable<PStringBase<char>,PStringBase<char>_>::StreamPack` · 00672060 · legions 0061c620 — needs port: cite-only; PHashTable StreamPack decode (string-keyed hash table in DAT payloads); ac2-script/src/data/enum_map.rs decodes the tables inline under EnumIDMap::StreamPack, verify and cite

### FilePathCache
- [-] `FilePathCache::InitFileFinder` · 00672360 · legions 0061c920 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)
- [-] `FilePathCache::SaveFileFinder` · 00672670 · legions 0061cc30 — loose-file/directory registry; the client reads only the shipped DATs (AGENTS)

## `portal/shared/src/fileformat/fileformat.cpp.c` (0/45)

### FileTemplate
- [-] `FileTemplate::FreeMatchData` · 006661e0 — C++ constructor/destructor (Rust struct init or Drop)

### FileFormatGrabber
- [-] `FileFormatGrabber::FileFormatGrabber` · 00666250 — C++ constructor/destructor (Rust struct init or Drop)
- [-] `FileFormatGrabber::GetFileFormat` · 006662e0 · legions 00613a90 — admin-only tooling (text source, preprocess, EMP or format authoring)

### PFileNode
- [ ] `PFileNode::SetNameAsInt32` · 006662f0 · legions 00613aa0 — needs port: cite-only; PFileNode name setters behind efun gPFileNode::CallEFun_SetNameAsInt32 (ac2-script glue); verify whether WSL scripts reach them
- [ ] `PFileNode::SetName` · 00666340 · legions 00613af0 — needs port: cite-only; PFileNode name setter behind WSL efuns (ac2-script glue efun_names.txt); verify

### FileFormat
- [-] `FileFormat::IsBinary` · 00666390 · legions 00613b40 — admin-only tooling (text source, preprocess, EMP or format authoring)

### PFileNode
- [ ] `PFileNode::GetNameAsCellID` · 00666480 — needs port: cite-only; PFileNode name getter as cell id; verify against WSL efuns
- [ ] `PFileNode::GetNameAsHex` · 00666540 — needs port: cite-only; PFileNode name getter as hex; verify against WSL efuns

### TemplateInstance
- [-] `TemplateInstance::TemplateInstance` · 00666600 — C++ constructor/destructor (Rust struct init or Drop)
- [-] `TemplateInstance::TemplateInstance` · 00666630 — C++ constructor/destructor (Rust struct init or Drop)

### PStringBase<char>
- [-] `PStringBase<char>::~PStringBase<char>` · 006666a0 · legions 005f0620 — C++ constructor/destructor (Rust struct init or Drop)

### FileTemplate
- [-] `FileTemplate::NodeMatchesTemplate` · 006666c0 — text-template matching for the pack path (write side); reader needs only UnPackBinary, see FileFormat::LoadBinary
- [-] `FileTemplate::BinarySize_Internal` · 00666800 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `FileTemplate::BinarySize_SubNodes` · 00666960 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `FileTemplate::HandleRepeatNode` · 00666a20 — text-template matching for the pack path (write side)
- [-] `FileTemplate::SubNodesMatchTemplates` · 00666c70 — text-template matching for the pack path (write side)

### (free functions)
- [ ] `is_keyword` · 00666dc0 — needs port: cite-only; keyword check behind gPFileNode::CallEFun_IsKeywordW (ac2-script glue)

### FileTemplate
- [-] `FileTemplate::MakeFileNode` · 00667010 — text-template node building for the pack path (write side)

### FileFormat
- [-] `FileFormat::ToFileNode` · 006672a0 — text source format (admin tools)

### FileTemplate
- [-] `FileTemplate::VerifyAndConvert` · 00667500 — text-template verification for the pack path (write side)
- [-] `FileTemplate::BinarySize` · 00667580 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `FileTemplate::PackBinary_Internal` · 00667620 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `FileTemplate::PackBinary_SubNodes` · 006678d0 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [ ] `FileTemplate::UnPackBinary_Internal` · 006679c0 · legions 00613cd0 — needs port: FileTemplate binary node decode (UnPackBinary); needed with FileFormat::LoadBinary if shipped DATs hold FileFormat records
- [ ] `FileTemplate::UnPackBinary_SubNodes` · 00667c40 · legions 00613f50 — needs port: FileTemplate binary sub-node decode (UnPackBinary); needed with FileFormat::LoadBinary
- [-] `FileTemplate::PackBinary` · 00667d10 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)

### FileFormat
- [-] `FileFormat::SaveBinary` · 00667d70 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [ ] `FileFormat::LoadBinary` · 00667ea0 · legions 00614020 — needs port: FileFormat binary load (DB_TYPE_FILEFORMAT records, FileTemplate::UnPackBinary); verify whether shipped DATs hold FileFormat records, decode if so
- [-] `FileFormat::SaveBinary` · 00668420 — DAT write, transaction or patch path; the client only reads the shipped DATs (AGENTS)
- [-] `FileFormat::LoadFile` · 00668450 · legions 00614220 — admin-only tooling (text source, preprocess, EMP or format authoring)

### _STL::map<unsigned_long,FileTemplate_*,_STL::greater<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,FileTemplate_*>_>_>
- [-] `_STL::map<unsigned_long,FileTemplate_*,_STL::greater<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,FileTemplate_*>_>_>::operator[]` · 00668a70 — C++ destructor or operator (Rust Drop / value semantics)

### _STL::map<PStringBase<char>,unsigned_long,_STL::less<PStringBase<char>_>,_STL::allocator<_STL::pair<PStringBase<char>_const_,unsigned_long>_>_>
- [-] `_STL::map<PStringBase<char>,unsigned_long,_STL::less<PStringBase<char>_>,_STL::allocator<_STL::pair<PStringBase<char>_const_,unsigned_long>_>_>::operator[]` · 00668b30 — C++ destructor or operator (Rust Drop / value semantics)

### FileTemplate
- [-] `FileTemplate::FileTemplate` · 00668bc0 — C++ constructor/destructor (Rust struct init or Drop)
- [-] `FileTemplate::Destroy` · 00668c20 · legions 00614290 — C++ constructor/destructor (Rust struct init or Drop)
- [-] `FileTemplate::MakeNodeTemplate` · 00668cd0 — admin-only tooling (text source, preprocess, EMP or format authoring)

### FileFormat
- [-] `FileFormat::FileFormat` · 00668d80 · legions 00614340 — C++ constructor/destructor (Rust struct init or Drop)
- [-] `FileFormat::GetDBOType` · 00668de0 — DBObj class factory (Allocate/GetDBOType); Rust decodes into typed structs
- [-] `FileFormat::Allocate` · 00668df0 — DBObj class factory (Allocate/GetDBOType); Rust decodes into typed structs

### FileTemplate
- [-] `FileTemplate::~FileTemplate` · 00668e00 · legions 006143c0 — C++ constructor/destructor (Rust struct init or Drop)
- [-] `FileTemplate::ParseNodeTemplate_Internal` · 00668e90 — admin-only tooling (text source, preprocess, EMP or format authoring)
- [-] `FileTemplate::ParseDefinitions` · 00669490 — admin-only tooling (text source, preprocess, EMP or format authoring)
- [-] `FileTemplate::ParseNodeTemplate` · 00669670 — admin-only tooling (text source, preprocess, EMP or format authoring)

### FileFormat
- [-] `FileFormat::FromFileNode` · 00669800 — text source format (admin tools)
- [-] `FileFormat::Destroy` · 00669a10 · legions 00614450 — C++ constructor/destructor (Rust struct init or Drop)
- [-] `FileFormat::~FileFormat` · 00669ab0 · legions 006144f0 — C++ constructor/destructor (Rust struct init or Drop)

## `portal/shared/src/pheader/pheader.cpp.c` (0/9)

### PreprocHeader
- [-] `PreprocHeader::PreprocHeader` · 00678990 · legions 00622fd0 — C++ constructor/destructor (Rust struct init or Drop)
- [-] `PreprocHeader::~PreprocHeader` · 006789c0 · legions 00623000 — C++ constructor/destructor (Rust struct init or Drop)
- [-] `PreprocHeader::SetHeaderData` · 00678a00 · legions 00623040 — admin-only tooling (text source, preprocess, EMP or format authoring)
- [-] `PreprocHeader::Init` · 00678a60 · legions 006230a0 — admin-only tooling (text source, preprocess, EMP or format authoring)
- [-] `PreprocHeader::GetHeaderData` · 00678b70 — admin-only tooling (text source, preprocess, EMP or format authoring)
- [-] `PreprocHeader::SaveHeaderInternal` · 00678bc0 · legions 006231b0 — admin-only tooling (text source, preprocess, EMP or format authoring)
- [-] `PreprocHeader::SaveHeader` · 00678cf0 · legions 006232e0 — admin-only tooling (text source, preprocess, EMP or format authoring)
- [-] `PreprocHeader::SaveHeaderAsComment` · 00678d20 · legions 00623310 — admin-only tooling (text source, preprocess, EMP or format authoring)
- [-] `PreprocHeader::GetHeader` · 00678dc0 — admin-only tooling (text source, preprocess, EMP or format authoring)
