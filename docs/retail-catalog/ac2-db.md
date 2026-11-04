# ac2-db: Database objects, properties, enum maps, string tables, client cache

633 retail functions: 271 done, 0 partial, 0 to do, 362 skipped.
Legend and workflow: [README](README.md).

## `ac2/client/src/ccache/gmclcacheinit.cpp.c` (2/5)

### gmCLCache
- [x] `gmCLCache::Init_Internal` · 00401940 · legions 00401870 — a forward to CLCache::Init_Internal (done): ac2-dat DatDataset::open and ac2-weenie ddd.rs slot order, which now names it
- [-] `gmCLCache::gmCLCache` · 00401950 · legions 00401880 — skip: constructor/destructor of plain data; Rust struct init and Drop

### PortalThread
- [-] `PortalThread::SetShouldExit` · 00401970 · legions 004018a0 — thread shutdown flag; worker threads stop on drop (ac2-db worker_lanes.rs)

### Turbine::Client
- [-] `Turbine::Client::CreateChatInterfaceProvider` · 00401990 · legions 00401b60 — not needed:  [body is "return NULL" (gmclcacheinit.cpp.c:1889)]

### gmCLCache
- [x] `gmCLCache::Init` · 004019e0 · legions 00401900 — audit: covered: internal.dat (slot 4) is a dev file not opened [ac2-dat/src/dataset.rs:20 DatDataset::open (portal, cell, local, highres); ac2-weenie/src/ddd.rs:3-30 slot order and initialized check]

## `ac2/shared/src/database/gmmasterdbmap.cpp.c` (3/4)

### gmMasterDBMap
- [x] `gmMasterDBMap::DivineType_Internal` · 006018c0 — ac2-script data/db_types.rs RANGES/db_type: every range of the hard-coded gmMasterDBMap::DivineType_Internal(did) table, now with its own DIDs 0x0E000003 (FILE2ID_TABLE), 0x0EBADA55 (TABOO_TABLE), 0x0E000004 (PLACES_TABLE) and 0xFFFF0000..0xFFFFFFFE (DATFILEDATA); InitDBTypeDef_Internal and the filename overload add nothing a DID lookup needs. Test the_gm_map_types_its_single_dids_and_the_iteration_lists

### DBTypeDef
- [-] `DBTypeDef::DBTypeDef` · 006018d0 · legions 005c27b0 — skip: constructor/destructor of plain data; Rust struct init and Drop

### gmMasterDBMap
- [x] `gmMasterDBMap::InitDBTypeDef_Internal` · 00601940 — ac2-script data/db_types.rs RANGES/db_type: every range of the hard-coded gmMasterDBMap::DivineType_Internal(did) table, now with its own DIDs 0x0E000003 (FILE2ID_TABLE), 0x0EBADA55 (TABOO_TABLE), 0x0E000004 (PLACES_TABLE) and 0xFFFF0000..0xFFFFFFFE (DATFILEDATA); InitDBTypeDef_Internal and the filename overload add nothing a DID lookup needs. Test the_gm_map_types_its_single_dids_and_the_iteration_lists
- [x] `gmMasterDBMap::DivineType_Internal` · 00601a50 — ac2-script data/db_types.rs RANGES/db_type: every range of the hard-coded gmMasterDBMap::DivineType_Internal(did) table, now with its own DIDs 0x0E000003 (FILE2ID_TABLE), 0x0EBADA55 (TABOO_TABLE), 0x0E000004 (PLACES_TABLE) and 0xFFFF0000..0xFFFFFFFE (DATFILEDATA); InitDBTypeDef_Internal and the filename overload add nothing a DID lookup needs. Test the_gm_map_types_its_single_dids_and_the_iteration_lists

## `ac2/shared/src/languagespecific/gmcharnames.cpp.c` (2/2)

### gmCharNameCharacterValidator
- [x] `gmCharNameCharacterValidator::IsCharacterValid` · 0060cb90 · legions 005cd990 — ac2-db char_names.rs (LanguageLetters from Legions 0x00A3B7D0; language set from the local DAT's name in mapper 0x4000002A, g_validator->SetLanguage at 0x004B43E2); ac2-ui confirmation.rs is_name_character = NameInputFilter; tests each_language_takes_ascii_and_its_own_letters, only_korean_takes_the_jamo..., the_language_names_map_to_the_name_validators_ids
- [x] `gmCharNameCharacterValidator::IsCharacterValidForTransitions` · 0060cc50 · legions 005cda50 — ac2-db char_names.rs (LanguageLetters from Legions 0x00A3B7D0; language set from the local DAT's name in mapper 0x4000002A, g_validator->SetLanguage at 0x004B43E2); ac2-ui confirmation.rs is_name_character = NameInputFilter; tests each_language_takes_ascii_and_its_own_letters, only_korean_takes_the_jamo..., the_language_names_map_to_the_name_validators_ids

## `portal/client/src/clcache/clcachemain.cpp.c` (21/132)

### EntityDesc
- [-] `EntityDesc::Allocator` · 004be6e0 · legions 004b2a20 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### EntityGroupDesc
- [-] `EntityGroupDesc::Allocator` · 004be700 · legions 004b2a40 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### PathMap
- [-] `PathMap::Allocator` · 004be720 · legions 004b2a60 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### CCellMesh
- [-] `CCellMesh::Allocator` · 004be740 · legions 004b2a80 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### SceneFile
- [-] `SceneFile::Allocator` · 004be760 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### AnimMap
- [-] `AnimMap::Allocator` · 004be780 · legions 004b2ac0 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### CSetup
- [-] `CSetup::Allocator` · 004be7a0 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### DBAnimator
- [-] `DBAnimator::Allocator` · 004be7d0 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### CDataMesh
- [-] `CDataMesh::Allocator` · 004be7f0 · legions 004b2b30 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### DBFile2IDTable
- [-] `DBFile2IDTable::Allocator` · 004be820 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)
- [-] `DBFile2IDTable::GetDBOType` · 004be860 — skip: retail DBO type tag; Rust typed enum in ac2-db db_type.rs
- [-] `DBFile2IDTable::~DBFile2IDTable` · 004be890 · legions 004b2bd0 — skip: constructor/destructor of plain data; Rust struct init and Drop

### TabooTable
- [-] `TabooTable::Allocator` · 004be8b0 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### EnumMapper
- [-] `EnumMapper::Allocator` · 004be8d0 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### CRegionDesc
- [-] `CRegionDesc::Allocator` · 004be8f0 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### CSoundDesc
- [-] `CSoundDesc::Allocator` · 004be910 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### CSceneDesc
- [-] `CSceneDesc::Allocator` · 004be930 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### CTerrainDesc
- [-] `CTerrainDesc::Allocator` · 004be950 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### CSurfaceDesc
- [-] `CSurfaceDesc::Allocator` · 004be970 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### CEncounterDesc
- [-] `CEncounterDesc::Allocator` · 004be990 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### CSkyDesc
- [-] `CSkyDesc::Allocator` · 004be9b0 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### CWaterDesc
- [-] `CWaterDesc::Allocator` · 004be9d0 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### CFogDesc
- [-] `CFogDesc::Allocator` · 004be9f0 · legions 004b2d30 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### PropertyDesc
- [-] `PropertyDesc::Allocator` · 004bea10 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### BlockMap
- [-] `BlockMap::Allocator` · 004bea30 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### CDayDesc
- [-] `CDayDesc::Allocator` · 004bea50 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### CKeyMap
- [-] `CKeyMap::Allocator` · 004bea70 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### DBFXTable
- [-] `DBFXTable::Allocator` · 004bea90 · legions 004b2dd0 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### AppearanceTable
- [-] `AppearanceTable::Allocator` · 004beab0 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### VisualDesc
- [-] `VisualDesc::Allocator` · 004bead0 · legions 004b2e10 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### LayoutDesc
- [-] `LayoutDesc::Allocator` · 004beaf0 · legions 004b2e30 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### UIScene
- [-] `UIScene::Allocator` · 004beb10 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### DBMusicObj
- [-] `DBMusicObj::Allocator` · 004beb30 · legions 004b2e70 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### StringTable
- [-] `StringTable::Allocator` · 004beb50 · legions 004b2e90 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### FileFormat
- [-] `FileFormat::Allocator` · 004beb70 · legions 004b2eb0 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### EnumIDMap
- [-] `EnumIDMap::Allocator` · 004beb90 · legions 004b2ed0 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### SoundInfo
- [-] `SoundInfo::Allocator` · 004bebb0 · legions 004b2ef0 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### MaterialModifier
- [-] `MaterialModifier::Allocator` · 004bebd0 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### RenderMaterial
- [-] `RenderMaterial::Allocator` · 004bebf0 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### MusicInfo
- [-] `MusicInfo::Allocator` · 004bec10 · legions 004b2f50 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### CharTemplate
- [-] `CharTemplate::Allocator` · 004bec30 · legions 004b2f70 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### MaterialInstance
- [-] `MaterialInstance::Allocator` · 004bec50 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### MotionInterpDesc
- [-] `MotionInterpDesc::Allocator` · 004bec70 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### MasterProperty
- [-] `MasterProperty::Allocator` · 004bec90 · legions 004b2fd0 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### LandscapeDefs
- [-] `LandscapeDefs::Allocator` · 004becb0 · legions 004b3010 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### PhysicsMaterial
- [-] `PhysicsMaterial::Allocator` · 004becd0 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### ObstacleDesc
- [-] `ObstacleDesc::Allocator` · 004becf0 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### PSDesc
- [-] `PSDesc::Allocator` · 004bed10 · legions 004b3050 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### RenderTextureLocal
- [-] `RenderTextureLocal::GetDBOType` · 004bed30 — skip: retail DBO type tag; Rust typed enum in ac2-db db_type.rs

### RenderSurfaceLocal
- [-] `RenderSurfaceLocal::GetDBOType` · 004bed40 — skip: retail DBO type tag; Rust typed enum in ac2-db db_type.rs

### Font
- [-] `Font::~Font` · 004bed50 · legions 004b3090 — skip: constructor/destructor of plain data; Rust struct init and Drop
- [-] `Font::Allocator` · 004bed70 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)
- [-] `Font::GetDBOType` · 004beda0 — skip: retail DBO type tag; Rust typed enum in ac2-db db_type.rs

### FontLocal
- [-] `FontLocal::GetDBOType` · 004bede0 — skip: retail DBO type tag; Rust typed enum in ac2-db db_type.rs

### QualityFilter
- [-] `QualityFilter::Allocator` · 004bedf0 · legions 004b3130 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### BehaviorTable
- [-] `BehaviorTable::Allocator` · 004bee10 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### ActionMap
- [-] `ActionMap::Allocator` · 004bee30 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### Performer
- [-] `Performer::Allocator` · 004bee50 · legions 004b3190 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### ValidModes
- [-] `ValidModes::Allocator` · 004bee70 · legions 004b31b0 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### CameraFX
- [x] `CameraFX::Allocator` · 004bee90 — ac2-smartbox camera_fx.rs (CameraFx decode, CameraFxManager tracks, sums, clamps, distortion); all 7 portal records decode

### WLib
- [-] `WLib::Allocator` · 004beeb0 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### StringState
- [-] `StringState::Allocator` · 004beed0 · legions 004b3210 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### CDetailMapDesc
- [-] `CDetailMapDesc::Allocator` · 004beef0 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### CMapNoteDesc
- [-] `CMapNoteDesc::Allocator` · 004bef10 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### ConversationTree
- [-] `ConversationTree::Allocator` · 004bef30 · legions 004b3270 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### DBCategories
- [-] `DBCategories::Allocator` · 004bef50 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### WState
- [-] `WState::Allocator` · 004bef70 · legions 004b32b0 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### CBaseQualities
- [-] `CBaseQualities::Allocator` · 004bef90 · legions 004b32f0 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### DatFileData
- [-] `DatFileData::Allocator` · 004befb0 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### CLCache
- [-] `CLCache::ResetCache` · 004befd0 · legions 004b3310 — replaced by design:  [AGENTS.md DB object cache exception: free on last drop, no retention pool to reset]
- [x] `CLCache::WorkerExecuteRequest` · 004befe0 · legions 004b3320 — ac2-db loader_pool.rs (named there): a get runs ThreadedCache::WorkerExecuteRequest on the loader workers; operation 3 is the BeginDDD download (skipped)
- [x] `CLCache::OnRequestFinished` · 004bf000 · legions 004b3340 — audit: covered: AGENTS.md item 2 [ac2-client/src/client.rs:557 db_cache(): time-budgeted drain of the loader channel (LOAD_BUDGET, ac2-client/src/bin/ac2/main.rs:64); ac2-db/src/loader_pool.rs:102 try_next]

### Cache_Pack_t
- [-] `Cache_Pack_t::~Cache_Pack_t` · 004bf500 · legions 004b38d0 — skip: constructor/destructor of plain data; Rust struct init and Drop

### RenderTextureLocal
- [-] `RenderTextureLocal::Allocator` · 004bf570 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### RenderSurfaceLocal
- [-] `RenderSurfaceLocal::Allocator` · 004bf5c0 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### FontLocal
- [-] `FontLocal::Allocator` · 004bf610 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

### CThreadsafeDiskController
- [x] `CThreadsafeDiskController::LoadData` · 004bf670 · legions 004b3a40 — audit: covered:  [ac2-dat/src/file.rs:83 read_record (locked B-tree read)]
- [-] `CThreadsafeDiskController::SaveData` · 004bf6b0 · legions 004b3a80 — skip: DAT writing/saving/packing (tools only)
- [-] `CThreadsafeDiskController::DeleteData` · 004bf6f0 · legions 004b3ac0 — skip: DAT writing/saving/packing (tools only)
- [x] `CThreadsafeDiskController::IsMember` · 004bf730 · legions 004b3b00 — audit: covered:  [ac2-dat/src/dataset.rs:72 contains; file_btree_runtime.rs:97]
- [-] `CThreadsafeDiskController::DeleteDataByMask` · 004bf770 · legions 004b3b40 — skip: DAT writing/saving/packing (tools only)

### CLCache
- [-] `CLCache::AsyncGetFromOtherSources` · 004bf7b0 · legions 004b3b80 — the data download (DDD) path: no supported server sends it. Pyrealium sends only 0x1003B (end) before the character set and never 0x1003A (begin), 0x10035 (data) or an answer to a missing-DID CCliDatRequestEvent (pyrealium-server crates/pyrealium/src/server.rs); the listeners (gmClient/Turbine::Client/gmDataPatchUI::OnDDDEvent) are already skipped. Recorded in AGENTS.md (DB cache rule)
- [-] `CLCache::WorkerExecuteBeginDDDRequest` · 004bf800 · legions 004b3bd0 — the data download (DDD) path: no supported server sends it. Pyrealium sends only 0x1003B (end) before the character set and never 0x1003A (begin), 0x10035 (data) or an answer to a missing-DID CCliDatRequestEvent (pyrealium-server crates/pyrealium/src/server.rs); the listeners (gmClient/Turbine::Client/gmDataPatchUI::OnDDDEvent) are already skipped. Recorded in AGENTS.md (DB cache rule)

### PSmartArray<LOAD_UVPAIR>
- [-] `PSmartArray<LOAD_UVPAIR>::~PSmartArray<LOAD_UVPAIR>` · 004bfa10 · legions 0063bba0 — skip: constructor/destructor of plain data; Rust struct init and Drop

### DiskConInitInfo
- [-] `DiskConInitInfo::DiskConInitInfo` · 004bfa40 · legions 004b3de0 — skip: constructor/destructor of plain data; Rust struct init and Drop

### LookFile
- [-] `LookFile::~LookFile` · 004bfab0 · legions 004b3e50 — skip: constructor/destructor of plain data; Rust struct init and Drop

### FxScript
- [-] `FxScript::GetDBOType` · 004bfb10 — editor text source / cache allocator (admin tools or Rust value type)

### DiskConInitInfo
- [-] `DiskConInitInfo::~DiskConInitInfo` · 004bfb20 · legions 004b3ec0 — skip: constructor/destructor of plain data; Rust struct init and Drop

### CThreadsafeDiskController
- [-] `CThreadsafeDiskController::CThreadsafeDiskController` · 004bfb60 · legions 004b3f00 — skip: constructor/destructor of plain data; Rust struct init and Drop

### DiskConBase
- [x] `DiskConBase::GetDatFileID` · 004bfb90 · legions 004b3f30

### DiskController
- [-] `DiskController::HowmuchFreeSpace` · 004bfbb0 · legions 004b3f50 — skip: DAT writing/saving/packing (tools only)
- [x] `DiskController::GetMasterMapDID` · 004bfbc0 · legions 004b3f60 — a field getter: ac2-dat btree.rs DatHeader master_map_did (named there), read by ac2-script data MASTER_MAP_DID
- [x] `DiskController::GetDatIDStamp` · 004bfbd0 · legions 004b3f70 — audit: covered:  [ac2-dat/src/btree.rs:92; ac2-weenie/src/ddd.rs dat_file_id]
- [x] `DiskController::GetFilename` · 004bfbf0 · legions 004b3f90 — a field getter: ac2-dat paths.rs DatOpenOptions::path_for (named there)

### CLCache
- [-] `CLCache::NotifyDDDEvent` · 004bfc40 · legions 004b3fe0 — the data download (DDD) path: no supported server sends it. Pyrealium sends only 0x1003B (end) before the character set and never 0x1003A (begin), 0x10035 (data) or an answer to a missing-DID CCliDatRequestEvent (pyrealium-server crates/pyrealium/src/server.rs); the listeners (gmClient/Turbine::Client/gmDataPatchUI::OnDDDEvent) are already skipped. Recorded in AGENTS.md (DB cache rule)

### (free functions)
- [-] `operator+` · 004bfc80 · legions 004b40c0 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)

### FxScript
- [-] `FxScript::Allocator` · 004bfcd0 — editor text source / cache allocator (admin tools or Rust value type)
- [-] `FxScript::Allocate` · 004bfd30 — editor text source / cache allocator (admin tools or Rust value type)

### CThreadsafeDiskController
- [x] `CThreadsafeDiskController::InitFile` · 004bfd70 · legions 004b41b0 — audit: covered:  [ac2-dat/src/dataset.rs:20 open; file_btree_runtime.rs:47]

### ThreadedCache
- [-] `ThreadedCache::~ThreadedCache` · 004bfe10 · legions 004b4250 — skip: constructor/destructor of plain data; Rust struct init and Drop

### CLCache
- [x] `CLCache::OnServerInterrogation` · 004bfeb0 — audit: covered: internal.dat slot not opened; language id configured, not looked up via EnumMapper 0x4000002A [ac2-weenie/src/session.rs:559, ac2-weenie/src/ddd.rs:1; ac2-net/src/protocol/dat_event.rs:137]
- [-] `CLCache::OnBeginDDD` · 004c0060 · legions 004b44a0 — the data download (DDD) path: no supported server sends it. Pyrealium sends only 0x1003B (end) before the character set and never 0x1003A (begin), 0x10035 (data) or an answer to a missing-DID CCliDatRequestEvent (pyrealium-server crates/pyrealium/src/server.rs); the listeners (gmClient/Turbine::Client/gmDataPatchUI::OnDDDEvent) are already skipped. Recorded in AGENTS.md (DB cache rule)
- [x] `CLCache::SetLanguageInternal` · 004c00f0 · legions 004b4530 — audit: covered: no live reload [ac2-dat/src/paths.rs:13; ac2-client/src/bin/ac2/main.rs:763]
- [-] `CLCache::OnEndDDD` · 004c0610 · legions 004b4aa0 — the data download (DDD) path: no supported server sends it. Pyrealium sends only 0x1003B (end) before the character set and never 0x1003A (begin), 0x10035 (data) or an answer to a missing-DID CCliDatRequestEvent (pyrealium-server crates/pyrealium/src/server.rs); the listeners (gmClient/Turbine::Client/gmDataPatchUI::OnDDDEvent) are already skipped. Recorded in AGENTS.md (DB cache rule)
- [-] `CLCache::SetRegion` · 004c0690 · legions 004b4b20 — opens cell_<rid>.dat only when the region differs from the current one; every release ships one region (0x0F000000) and cell_1.dat, and Pyrealium CreatePlayer always names region 1 (bootstrap.rs REGION), so it never acts

### PSmartArray<Trigger>
- [x] `PSmartArray<Trigger>::StreamPack` · 004c0a40 · legions 004b4ed0 — audit: covered:  [ac2-anim/src/dbanimator/asset.rs:24,48 Ac2AnimTrigger/Ac2ImpulseTrigger decoded with the animator]

### Trigger
- [-] `Trigger::Trigger` · 004c0bc0 · legions 004b5050 — skip: constructor/destructor of plain data; Rust struct init and Drop

### PSmartArray<ImpulseTrigger>
- [x] `PSmartArray<ImpulseTrigger>::StreamPack` · 004c0bf0 · legions 004b5080 — audit: covered:  [ac2-anim/src/dbanimator/asset.rs:24,48 Ac2AnimTrigger/Ac2ImpulseTrigger decoded with the animator]

### ImpulseTrigger
- [-] `ImpulseTrigger::ImpulseTrigger` · 004c0d70 — skip: constructor/destructor of plain data; Rust struct init and Drop

### CLCache
- [-] `CLCache::~CLCache` · 004c0da0 · legions 004b5230 — skip: constructor/destructor of plain data; Rust struct init and Drop
- [-] `CLCache::UseTime` · 004c0f10 — CLCache::UseTime is the idle eviction of the retail DB cache (AGENTS: no DBOCache eviction)
- [-] `CLCache::RemovePendingDownload` · 004c10d0 · legions 004b5560 — the data download (DDD) path: no supported server sends it. Pyrealium sends only 0x1003B (end) before the character set and never 0x1003A (begin), 0x10035 (data) or an answer to a missing-DID CCliDatRequestEvent (pyrealium-server crates/pyrealium/src/server.rs); the listeners (gmClient/Turbine::Client/gmDataPatchUI::OnDDDEvent) are already skipped. Recorded in AGENTS.md (DB cache rule)
- [-] `CLCache::GetDiskController` · 004c1280 · legions 004b5710 — accessor for the DiskController; Rust holds the controller directly
- [x] `CLCache::IsDatFileLoaded` · 004c13a0 · legions 004b5800 — audit: covered: internal.dat (slot 4) is a dev file not opened [ac2-dat/src/dataset.rs:20 DatDataset::open (portal, cell, local, highres); ac2-weenie/src/ddd.rs:3-30 slot order and initialized check]

### TerrainList
- [-] `TerrainList::~TerrainList` · 004c1450 · legions 004b58b0 — skip: constructor/destructor of plain data; Rust struct init and Drop

### PSmartArray<CAllIterationList::PTaggedIterationList>
- [x] `PSmartArray<CAllIterationList::PTaggedIterationList>::StreamPack` · 004c14b0 · legions 004b5910 — ac2-net protocol/dat_event.rs interrogation_response: the count of DAT files, then each u64 id and its CMostlyConsecutiveIntSet (named there)

### CAllIterationList::PTaggedIterationList
- [-] `CAllIterationList::PTaggedIterationList::PTaggedIterationList` · 004c1630 · legions 004b5a90 — skip: constructor/destructor of plain data; Rust struct init and Drop
- [-] `CAllIterationList::PTaggedIterationList::operator=` · 004c1650 · legions 004b5ab0 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)

### SmartArray<unsigned_long>
- [-] `SmartArray<unsigned_long>::operator=` · 004c1680 · legions 0053f8d0 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)

### PSmartArray<CMissingIteration>
- [x] `PSmartArray<CMissingIteration>::StreamPack` · 004c16e0 · legions 004b5ae0 — audit: covered: decoded but not applied (see CLCache::OnBeginDDD) [ac2-net/src/protocol/dat_event.rs:73 MissingIteration decode]

### CMissingIteration
- [-] `CMissingIteration::CMissingIteration` · 004c1860 — skip: constructor/destructor of plain data; Rust struct init and Drop
- [-] `CMissingIteration::operator=` · 004c1880 · legions 004b5c80 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)

### CLCache
- [x] `CLCache::Init` · 004c18e0 · legions 004b5ce0 — audit: covered: internal.dat (slot 4) is a dev file not opened [ac2-dat/src/dataset.rs:20 DatDataset::open (portal, cell, local, highres); ac2-weenie/src/ddd.rs:3-30 slot order and initialized check]
- [-] `CLCache::OnSaveRequestFinished` · 004c1fa0 · legions 004b6330 — skip: DAT writing/saving/packing (tools only) (cache save completion)

### PHashTable<unsigned_long,MotionValues>
- [x] `PHashTable<unsigned_long,MotionValues>::StreamPack` · 004c2130 — audit: covered:  [ac2-world/src/ac2_motion_interp.rs:90,153,335,374 MotionValues/TerrainList stream order]

### IntrusiveHashIterator<unsigned_long,HashTableData<unsigned_long,unsigned_long>*>
- [-] `IntrusiveHashIterator<unsigned_long,HashTableData<unsigned_long,unsigned_long>*>::operator=` · 004c2310 · legions 00504950 — skip: STL/container internal (iterator, tree, comparator); Rust collections

### CLCache
- [x] `CLCache::Init_Internal` · 004c2490 · legions 004b6860 — audit: covered: internal.dat (slot 4) is a dev file not opened [ac2-dat/src/dataset.rs:20 DatDataset::open (portal, cell, local, highres); ac2-weenie/src/ddd.rs:3-30 slot order and initialized check]
- [-] `CLCache::CLCache` · 004c41d0 · legions 004b85a0 — skip: constructor/destructor of plain data; Rust struct init and Drop
- [-] `CLCache::OnBeginDDDRequestFinished` · 004c42a0 · legions 004b8670 — the data download (DDD) path: no supported server sends it. Pyrealium sends only 0x1003B (end) before the character set and never 0x1003A (begin), 0x10035 (data) or an answer to a missing-DID CCliDatRequestEvent (pyrealium-server crates/pyrealium/src/server.rs); the listeners (gmClient/Turbine::Client/gmDataPatchUI::OnDDDEvent) are already skipped. Recorded in AGENTS.md (DB cache rule)

### PHashTable<unsigned_long,TerrainList>
- [x] `PHashTable<unsigned_long,TerrainList>::StreamPack` · 004c45d0 · legions 004b89a0 — audit: covered:  [ac2-world/src/ac2_motion_interp.rs:90,153,335,374 MotionValues/TerrainList stream order]

### PHashTable<unsigned_long,MotionValues>
- [-] `PHashTable<unsigned_long,MotionValues>::PHashTable<unsigned_long,MotionValues>` · 004c47f0 · legions 004b8bc0 — skip: constructor/destructor of plain data; Rust struct init and Drop

### MSList
- [-] `MSList::~MSList` · 004c48f0 · legions 004b8cc0 — skip: constructor/destructor of plain data; Rust struct init and Drop

## `portal/client/src/clcache/clocache.cpp.c` (0/1)

### CLOCache
- [-] `CLOCache::CLOCache` · 004c4a10 · legions 004b8de0 — skip: constructor/destructor of plain data; Rust struct init and Drop

## `portal/shared/src/enummap/enummapper.cpp.c` (21/47)

### EnumMapper
- [x] `EnumMapper::Get` · 00660750 · legions 0060f060 — audit: covered:  [ac2-script/src/data/mod.rs:479 enum_mapper(did); data/enum_map.rs:21]
- [-] `EnumMapper::GetSubDataIDs` · 00660770 — not needed:  [data-graph prefetch only; m_pDataGraph never set]
- [x] `EnumMapper::GetSubObjects` · 00660790 — audit: covered:  [ac2-script/src/data/enum_map.rs:10 lookups fall through to the base mapper; glue/enums.rs:72 chain]
- [-] `EnumMapper::ReleaseSubObjects` · 006607f0 · legions 0060f0c0 — skip: release of a shared ref or buffer; Arc/Drop replaces it

### EnumMapper::iterator
- [-] `EnumMapper::iterator::operator!=` · 00660810 · legions 0060f0e0 — skip: STL/container internal (iterator, tree, comparator); Rust collections

### _STL::_Rb_tree_iterator<_STL::pair<unsigned_long_const_,PStringBase<char>_>,_STL::_Const_traits<_STL::pair<unsigned_long_const_,PStringBase<char>_>_>_>
- [-] `_STL::_Rb_tree_iterator<_STL::pair<unsigned_long_const_,PStringBase<char>_>,_STL::_Const_traits<_STL::pair<unsigned_long_const_,PStringBase<char>_>_>_>::operator++` · 00660860 · legions 0060f130 — skip: STL/container internal (iterator, tree, comparator); Rust collections

### CaseInsensitivePStringCompare
- [-] `CaseInsensitivePStringCompare::operator()` · 006608b0 · legions 0060f1d0 — skip: STL/container internal (iterator, tree, comparator); Rust collections

### EnumMapper
- [x] `EnumMapper::GetNumMappings` · 006608e0 · legions 0060f200 — ac2-script data/glue::enums

### EnumMapper::iterator
- [-] `EnumMapper::iterator::get_enum` · 00660900 · legions 0060f220 — skip: STL/container internal (iterator, tree, comparator); Rust collections

### EnumMapper
- [x] `EnumMapper::GetNumMappings` · 00660a30 · legions 0060f350 — ac2-script data/glue::enums
- [-] `EnumMapper::end` · 00660ae0 · legions 0060f400 — skip: STL/container internal (iterator, tree, comparator); Rust collections

### EnumMapper::iterator
- [-] `EnumMapper::iterator::iterator` · 00660b10 · legions 0060f430 — skip: STL/container internal (iterator, tree, comparator); Rust collections

### EnumMapper
- [x] `EnumMapper::GetString` · 00660b90 · legions 0060f4b0 — ac2-script data/glue::enums
- [x] `EnumMapper::GetString` · 00660bd0 · legions 0060f4f0 — ac2-script data/glue::enums
- [x] `EnumMapper::GetString` · 00660c50 · legions 0060f570 — ac2-script data/glue::enums
- [x] `EnumMapper::InqString` · 00660d10 · legions 0060f630 — ac2-script data/glue::enums
- [x] `EnumMapper::GetStringW` · 00660dd0 · legions 0060f6f0 — ac2-script data/glue::enums
- [-] `EnumMapper::GetStringByDID` · 00660e40 · legions 0060f760 — not needed:  [only callers BasePropertyDesc::InqPropertyEnum (EnumPropertyValue From/ToFileNode, GetValueAsString: tool/debug) and MasterProperty::LoadFromFile/PEnumMapper::AddEnum (tools, no Legions address)]
- [x] `EnumMapper::GetLogStringW` · 00660f10 · legions 0060f800 — ac2-script data/glue::enums
- [-] `EnumMapper::begin` · 00660f70 · legions 0060f860 — skip: STL/container internal (iterator, tree, comparator); Rust collections

### EnumMapper::iterator
- [-] `EnumMapper::iterator::operator++` · 00660fa0 · legions 0060f890 — skip: STL/container internal (iterator, tree, comparator); Rust collections
- [-] `EnumMapper::iterator::operator++` · 00661000 · legions 0060f8f0 — skip: STL/container internal (iterator, tree, comparator); Rust collections

### EnumMapper
- [x] `EnumMapper::GetEnum` · 00661020 · legions 0060f910 — ac2-script data/glue::enums
- [x] `EnumMapper::GetEnum` · 00661060 · legions 0060f950 — ac2-script data/glue::enums
- [x] `EnumMapper::GetEnum` · 006610b0 · legions 0060f9a0 — ac2-script data/glue::enums
- [x] `EnumMapper::InqEnum` · 00661170 · legions 0060fa60 — ac2-script data/glue::enums
- [x] `EnumMapper::GetEnumByDID` · 006611c0 · legions 0060fab0 — ac2-script data/glue::enums
- [x] `EnumMapper::GetEnumByDIDW` · 00661290 · legions 0060fb50 — ac2-script data/glue::enums

### _STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,PStringBase<char>_>,_STL::_Select1st<_STL::pair<unsigned_long_const_,PStringBase<char>_>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,PStringBase<char>_>_>_>
- [-] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,PStringBase<char>_>,_STL::_Select1st<_STL::pair<unsigned_long_const_,PStringBase<char>_>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,PStringBase<char>_>_>_>::operator=` · 00661950 — skip: STL/container internal (iterator, tree, comparator); Rust collections

### _STL::_Rb_tree<PStringBase<char>,_STL::pair<PStringBase<char>_const_,unsigned_long>,_STL::_Select1st<_STL::pair<PStringBase<char>_const_,unsigned_long>_>,CaseInsensitivePStringCompare,_STL::allocator<_STL::pair<PStringBase<char>_const_,unsigned_long>_>_>
- [-] `_STL::_Rb_tree<PStringBase<char>,_STL::pair<PStringBase<char>_const_,unsigned_long>,_STL::_Select1st<_STL::pair<PStringBase<char>_const_,unsigned_long>_>,CaseInsensitivePStringCompare,_STL::allocator<_STL::pair<PStringBase<char>_const_,unsigned_long>_>_>::operator=` · 00661bf0 — skip: STL/container internal (iterator, tree, comparator); Rust collections

### EnumMapper
- [x] `EnumMapper::GetAllEnumVals` · 00661f30 · legions 00610450 — ac2-script data/glue::enums
- [x] `EnumMapper::GetAllEnumVals` · 00662040 · legions 00610560 — ac2-script data/glue::enums

### (free functions)
- [-] `STREAMPACK_STL` · 00662120 · legions 00610640 — skip: STL/container internal (iterator, tree, comparator); Rust collections

### EnumMapper
- [-] `EnumMapper::Destroy` · 00662330 · legions 00610850 — skip: constructor/destructor of plain data; Rust struct init and Drop
- [-] `EnumMapper::LoadFromFile` · 006623a0 — text source loader, admin build only
- [x] `EnumMapper::GetAllEnumNames` · 00662490 · legions 006108c0 — ac2-script data/glue::enums
- [x] `EnumMapper::GetAllEnumNames` · 00662590 · legions 006109c0 — ac2-script data/glue::enums

### _STL::map<unsigned_long,PStringBase<char>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,PStringBase<char>_>_>_>
- [-] `_STL::map<unsigned_long,PStringBase<char>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,PStringBase<char>_>_>_>::operator[]` · 00662670 · legions 00610aa0 — skip: STL/container internal (iterator, tree, comparator); Rust collections

### _STL::map<PStringBase<char>,unsigned_long,CaseInsensitivePStringCompare,_STL::allocator<_STL::pair<PStringBase<char>_const_,unsigned_long>_>_>
- [-] `_STL::map<PStringBase<char>,unsigned_long,CaseInsensitivePStringCompare,_STL::allocator<_STL::pair<PStringBase<char>_const_,unsigned_long>_>_>::operator[]` · 00662740 · legions 00610b70 — skip: STL/container internal (iterator, tree, comparator); Rust collections

### EnumMapper
- [-] `EnumMapper::EnumMapper` · 006627d0 · legions 00610c00 — skip: constructor/destructor of plain data; Rust struct init and Drop
- [-] `EnumMapper::GetDBOType` · 00662850 — skip: retail DBO type tag; Rust typed enum in ac2-db db_type.rs
- [-] `EnumMapper::EnumMapper` · 00662860 · legions 00610c90 — skip: constructor/destructor of plain data; Rust struct init and Drop
- [-] `EnumMapper::~EnumMapper` · 006628e0 · legions 00610d10 — skip: constructor/destructor of plain data; Rust struct init and Drop
- [x] `EnumMapper::StreamPack` · 00662970 · legions 00610da0 — ac2-script data/glue::enums
- [-] `EnumMapper::AddMapping` · 00662a30 · legions 00610e60 — not needed:  [only callers BasePropertyDesc::InqPropertyEnum (EnumPropertyValue From/ToFileNode, GetValueAsString: tool/debug) and MasterProperty::LoadFromFile/PEnumMapper::AddEnum (tools, no Legions address)]
- [-] `EnumMapper::Allocate` · 00662af0 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)
- [-] `EnumMapper::AddEnumMapper` · 00662b20 — not needed:  [only callers BasePropertyDesc::InqPropertyEnum (EnumPropertyValue From/ToFileNode, GetValueAsString: tool/debug) and MasterProperty::LoadFromFile/PEnumMapper::AddEnum (tools, no Legions address)]

## `portal/shared/src/enummap/penummapper.cpp.c` (0/16)

### PEnumMapper
- [-] `PEnumMapper::SetStartingValue` · 007381d0 · legions 006b3d90 — not needed:  [only callers are EmpGen/EmpGenParse and PEnumMapper ctor (enum-map generator tool, catalog [-])]

### _STL::less<PEnum>
- [-] `_STL::less<PEnum>::operator()` · 007381e0 · legions 006b3da0 — skip: STL/container internal (iterator, tree, comparator); Rust collections

### PEnum
- [-] `PEnum::Destroy` · 00738230 · legions 006b3df0 — skip: constructor/destructor of plain data; Rust struct init and Drop
- [-] `PEnum::operator=` · 00738300 · legions 006b3ec0 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)
- [-] `PEnum::Initialize` · 00738470 · legions 006b4030 — not needed:  [only callers are EmpGen/EmpGenParse and PEnumMapper ctor (enum-map generator tool, catalog [-])]

### PEnumMapper
- [-] `PEnumMapper::FindEnum` · 007385a0 · legions 006b4160 — not needed:  [only callers are EmpGen/EmpGenParse and PEnumMapper ctor (enum-map generator tool, catalog [-])]

### PEnum
- [-] `PEnum::PEnum` · 00738730 · legions 006b42f0 — skip: constructor/destructor of plain data; Rust struct init and Drop

### PEnumMapper
- [-] `PEnumMapper::SetInheritanceDepth` · 007387a0 · legions 006b4360 — not needed:  [only callers are EmpGen/EmpGenParse and PEnumMapper ctor (enum-map generator tool, catalog [-])]
- [-] `PEnumMapper::HaveMapping` · 00738800 · legions 006b43c0 — not needed:  [only callers are EmpGen/EmpGenParse and PEnumMapper ctor (enum-map generator tool, catalog [-])]
- [-] `PEnumMapper::NextValidID` · 007389b0 · legions 006b4570 — not needed:  [only callers are EmpGen/EmpGenParse and PEnumMapper ctor (enum-map generator tool, catalog [-])]
- [-] `PEnumMapper::~PEnumMapper` · 00738ee0 · legions 006b4aa0 — skip: constructor/destructor of plain data; Rust struct init and Drop
- [-] `PEnumMapper::Initialize` · 00738f60 · legions 006b4b20 — not needed:  [only callers are EmpGen/EmpGenParse and PEnumMapper ctor (enum-map generator tool, catalog [-])]
- [-] `PEnumMapper::PEnumMapper` · 00739020 · legions 006b4be0 — skip: constructor/destructor of plain data; Rust struct init and Drop
- [-] `PEnumMapper::AddEnum` · 00739080 · legions 006b4c40 — not needed:  [only callers are EmpGen/EmpGenParse and PEnumMapper ctor (enum-map generator tool, catalog [-])]
- [-] `PEnumMapper::AddEnum` · 00739320 — not needed:  [only callers are EmpGen/EmpGenParse and PEnumMapper ctor (enum-map generator tool, catalog [-])]
- [-] `PEnumMapper::SortEnums` · 00739400 · legions 006b4f40 — not needed:  [only callers are EmpGen/EmpGenParse and PEnumMapper ctor (enum-map generator tool, catalog [-])]

## `portal/shared/src/languagespecific/charnames.cpp.c` (0/2)

### CharNameCharacterValidator
- [-] `CharNameCharacterValidator::CharNameCharacterValidator` · 006bc580 · legions 00651520 — skip: constructor/destructor of plain data; Rust struct init and Drop
- [-] `CharNameCharacterValidator::~CharNameCharacterValidator` · 006bc5a0 · legions 00651540 — skip: constructor/destructor of plain data; Rust struct init and Drop

## `portal/shared/src/languagespecific/charutils.cpp.c` (1/1)

### CharUtils
- [x] `CharUtils::IsVowel` · 006bc600 · legions 006515a0 — ac2-db string_meta_text / string_state (CRT C locale classes)

## `portal/shared/src/property/baseproperty.cpp.c` (11/22)

### BaseProperty
- [x] `BaseProperty::IsReportedToWeenie` · 006b1f50 · legions 0064b1e0 — audit: covered:  [ac2-db/src/property.rs:214,220; ac2-physics/src/collide_property.rs:63,68]
- [x] `BaseProperty::IsInquirable` · 006b1f60 · legions 0064b1f0 — audit: covered:  [ac2-db/src/property.rs:214,220; ac2-physics/src/collide_property.rs:63,68]
- [-] `BaseProperty::IsDefaultValue` · 006b1f70 · legions 0064b200 — not needed: ac2-physics/src/physics_obj_property.rs:7 comment claiming a gap is wrong; fix the comment [BasePropertyDesc::StreamPack (basepropertydesc.cpp.c:162) does not stream m_defaultValue; only FromFileNode (tool) sets it, so IsDefaultValue is always 0 in the client and "keep every property" ma
- [x] `BaseProperty::InitToDefaultValue` · 006b1fa0 · legions 0064b230 — audit: covered: default value never loaded from the DAT (m_defaultValue null in client) [ac2-db/src/base_property.rs:485]

### DiskConBase
- [-] `DiskConBase::IsExpandable` · 006b2000 · legions 0064b290 — skip: DAT writing/saving/packing (tools only) (DiskConBase::IsExpandable is the DAT growth check)

### BaseProperty
- [x] `BaseProperty::SetPropertyType` · 006b2010 · legions 0064b2a0 — ac2-db base_property.rs: typed BasePropertyValue Inq/Set by type, same-type copy, Compare semantics (Legions Compare rows are the vtable form)
- [-] `BaseProperty::GetSubDataIDs` · 006b2060 · legions 0064b2f0 — not needed:  [data-graph prefetch / StringDownload (tool) only; catalog policy rows (CharacterSetup::GetSubDataIDs etc.) are [-]; m_pDataGraph never set]
- [-] `BaseProperty::BaseProperty` · 006b20f0 · legions 0064b380 — skip: constructor/destructor of plain data; Rust struct init and Drop
- [-] `BaseProperty::~BaseProperty` · 006b2110 · legions 0064b3a0 — skip: constructor/destructor of plain data; Rust struct init and Drop
- [-] `BaseProperty::operator=` · 006b2130 · legions 0064b3c0 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)
- [x] `BaseProperty::SetPropertyName` · 006b2180 · legions 0064b410 — ac2-db property names: MasterProperty name table (the m_emapper pairs, descrambled) read by ac2_property_name / ac2_property_name_enum; BaseProperty::SetPropertyName[W] (group and type from the desc, then a fresh value); tests property_tests.rs, tests/master_property_names.rs
- [x] `BaseProperty::InqPropertyBit` · 006b21e0 · legions 0064b470 — audit: covered:  [ac2-db/src/property.rs:210 ac2_property_desc (data word = bit); :235 ethereal_mask_with; ac2-physics/src/body_profile.rs:36]
- [x] `BaseProperty::Compare` · 006b2210 · legions 0064b4a0 — ac2-db base_property.rs: typed BasePropertyValue Inq/Set by type, same-type copy, Compare semantics (Legions Compare rows are the vtable form)
- [x] `BaseProperty::StreamPack` · 006b2250 — audit: covered: pack direction unused by the client [ac2-net/src/protocol/property.rs:3-8 and ac2-db/src/property.rs:509 read_ac2_base_property_array (unpack)]
- [-] `BaseProperty::BaseProperty` · 006b22c0 · legions 0064b550 — skip: constructor/destructor of plain data; Rust struct init and Drop
- [x] `BaseProperty::SetPropertyNameW` · 006b2310 · legions 0064b5a0 — ac2-db property names: MasterProperty name table (the m_emapper pairs, descrambled) read by ac2_property_name / ac2_property_name_enum; BaseProperty::SetPropertyName[W] (group and type from the desc, then a fresh value); tests property_tests.rs, tests/master_property_names.rs
- [x] `BaseProperty::InqPropertyNameW` · 006b2380 · legions 0064b610 — ac2-db property names: MasterProperty name table (the m_emapper pairs, descrambled) read by ac2_property_name / ac2_property_name_enum; BaseProperty::SetPropertyName[W] (group and type from the desc, then a fresh value); tests property_tests.rs, tests/master_property_names.rs
- [x] `BaseProperty::InqPropertyTypeW` · 006b2440 · legions 0064b6d0 — ac2-db property names: MasterProperty name table (the m_emapper pairs, descrambled) read by ac2_property_name / ac2_property_name_enum; BaseProperty::SetPropertyName[W] (group and type from the desc, then a fresh value); tests property_tests.rs, tests/master_property_names.rs
- [-] `BaseProperty::GetOutputString` · 006b2510 · legions 0064b7a0 — not needed: admin property viewer text [GetValueAsString is only reached through GetOutputString <- CollideProperty::GetPropertyOutputStringW <- efun with no Legions wlib call site (catalog [-] row)]
- [-] `BaseProperty::ValueFromFileNode` · 006b2650 — skip: admin/debug text or tool code (no client use in Legions); ValueFromFileNode parses text source files [vtable-checked: not a slot of any located Legions vtable]
- [-] `BaseProperty::ToFileNode` · 006b26b0 — text source format (admin tools)
- [-] `BaseProperty::FromFileNode` · 006b2790 — text source format (admin tools)

## `portal/shared/src/property/desc/basepropertydesc.cpp.c` (1/10)

### BasePropertyDesc
- [-] `BasePropertyDesc::BasePropertyDesc` · 006b6e70 · legions 0064ed50 — skip: constructor/destructor of plain data; Rust struct init and Drop
- [-] `BasePropertyDesc::~BasePropertyDesc` · 006b6ea0 · legions 0064ed80 — skip: constructor/destructor of plain data; Rust struct init and Drop
- [-] `BasePropertyDesc::operator=` · 006b6ee0 · legions 0064edc0 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)
- [-] `BasePropertyDesc::InqPropertyEnum` · 006b6f40 · legions 0064ee20 — not needed:  [callers EnumPropertyValue::FromFileNode/ToFileNode/GetValueAsString: tool/debug]
- [-] `BasePropertyDesc::InqPropertyEnum` · 006b6f70 — not needed:  [callers EnumPropertyValue::FromFileNode/ToFileNode/GetValueAsString: tool/debug]
- [-] `BasePropertyDesc::InitProperty` · 006b6fa0 · legions 0064ee50 — not needed:  [only from MasterProperty::InqOptionalProperties <- CPhysicsObj::BuildOptionalProperties (editor tooling, catalog [-]); m_defaultValue is null in the client]
- [-] `BasePropertyDesc::GetSubDataIDs` · 006b6fc0 · legions 0064ee70 — not needed:  [data-graph prefetch / StringDownload (tool) only; catalog policy rows (CharacterSetup::GetSubDataIDs etc.) are [-]; m_pDataGraph never set]
- [-] `BasePropertyDesc::BasePropertyDesc` · 006b7030 · legions 0064eee0 — skip: constructor/destructor of plain data; Rust struct init and Drop
- [x] `BasePropertyDesc::StreamPack` · 006b7070 · legions 0064ef20 — audit: covered:  [ac2-db/src/property.rs:97 Ac2PropertyTypes::decode_packed (type, group, data, report, required per desc)]
- [-] `BasePropertyDesc::FromFileNode` · 006b7100 — text source format (admin tools)

## `portal/shared/src/property/masterproperty.cpp.c` (52/130)

### BoolPropertyValue
- [-] `BoolPropertyValue::ToFileNode` · 006b3d50 — text source format (admin tools)
- [x] `BoolPropertyValue::InqBool` · 006b3d80 — ac2-db base_property.rs: typed BasePropertyValue Inq/Set by type, same-type copy, Compare semantics (Legions Compare rows are the vtable form)
- [x] `BoolPropertyValue::Set` · 006b3da0 — ac2-db base_property.rs: typed BasePropertyValue Inq/Set by type, same-type copy, Compare semantics (Legions Compare rows are the vtable form)
- [-] `BoolPropertyValue::Compare` · 006b3dc0 · legions 0064cc80 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)
- [-] `BoolPropertyValue::Copy` · 006b3e00 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)

### ColorPropertyValue
- [-] `ColorPropertyValue::ColorPropertyValue` · 006b3e30 — skip: constructor/destructor of plain data; Rust struct init and Drop
- [x] `ColorPropertyValue::StreamPack` · 006b3e60 — audit: covered: pack direction unused by the client [ac2-net/src/protocol/property.rs:3-8 and ac2-db/src/property.rs:509 read_ac2_base_property_array (unpack)]
- [-] `ColorPropertyValue::FromFileNode` · 006b3e70 — text source format (admin tools)
- [-] `ColorPropertyValue::ToFileNode` · 006b3e90 — text source format (admin tools)
- [x] `ColorPropertyValue::InqColor` · 006b3eb0 — ac2-db base_property.rs: typed BasePropertyValue Inq/Set by type, same-type copy, Compare semantics (Legions Compare rows are the vtable form)
- [x] `ColorPropertyValue::SetColor` · 006b3ee0 — ac2-db base_property.rs: typed BasePropertyValue Inq/Set by type, same-type copy, Compare semantics (Legions Compare rows are the vtable form)
- [x] `ColorPropertyValue::Set` · 006b3f10 — ac2-db base_property.rs: typed BasePropertyValue Inq/Set by type, same-type copy, Compare semantics (Legions Compare rows are the vtable form)
- [-] `ColorPropertyValue::Compare` · 006b3f30 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)
- [-] `ColorPropertyValue::Copy` · 006b3f70 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)

### DataFilePropertyValue
- [x] `DataFilePropertyValue::Set` · 006b3fd0 — ac2-db base_property.rs: typed BasePropertyValue Inq/Set by type, same-type copy, Compare semantics (Legions Compare rows are the vtable form)
- [-] `DataFilePropertyValue::Compare` · 006b3ff0 · legions 0064c8e0 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)
- [-] `DataFilePropertyValue::Copy` · 006b4030 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)

### EnumPropertyValue
- [x] `EnumPropertyValue::Set` · 006b4060 — ac2-db base_property.rs: typed BasePropertyValue Inq/Set by type, same-type copy, Compare semantics (Legions Compare rows are the vtable form)
- [-] `EnumPropertyValue::Compare` · 006b4080 · legions 0064cad0 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)
- [-] `EnumPropertyValue::Copy` · 006b40c0 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)

### FloatPropertyValue
- [x] `FloatPropertyValue::Set` · 006b40f0 — ac2-db base_property.rs: typed BasePropertyValue Inq/Set by type, same-type copy, Compare semantics (Legions Compare rows are the vtable form)
- [-] `FloatPropertyValue::Compare` · 006b4110 · legions 0064cbf0 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)
- [-] `FloatPropertyValue::Copy` · 006b4150 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)
- [-] `FloatPropertyValue::ToFileNode` · 006b4180 — text source format (admin tools)

### IntegerPropertyValue
- [x] `IntegerPropertyValue::Set` · 006b41c0 — ac2-db base_property.rs: typed BasePropertyValue Inq/Set by type, same-type copy, Compare semantics (Legions Compare rows are the vtable form)
- [-] `IntegerPropertyValue::Compare` · 006b41e0 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)
- [-] `IntegerPropertyValue::Copy` · 006b4220 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)
- [-] `IntegerPropertyValue::ToFileNode` · 006b4250 — text source format (admin tools)

### LongIntegerPropertyValue
- [x] `LongIntegerPropertyValue::InqLongInteger` · 006b4280 — ac2-db base_property.rs: typed BasePropertyValue Inq/Set by type, same-type copy, Compare semantics (Legions Compare rows are the vtable form)
- [x] `LongIntegerPropertyValue::SetLongInteger` · 006b42a0 — ac2-db base_property.rs: typed BasePropertyValue Inq/Set by type, same-type copy, Compare semantics (Legions Compare rows are the vtable form)
- [x] `LongIntegerPropertyValue::Set` · 006b42c0 — ac2-db base_property.rs: typed BasePropertyValue Inq/Set by type, same-type copy, Compare semantics (Legions Compare rows are the vtable form)
- [-] `LongIntegerPropertyValue::Compare` · 006b42e0 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)
- [-] `LongIntegerPropertyValue::Copy` · 006b4330 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)
- [-] `LongIntegerPropertyValue::ToFileNode` · 006b4370 — text source format (admin tools)

### PFileNode
- [-] `PFileNode::SetNameAsInt64` · 006b43b0 — skip: admin/debug text or tool code (no client use in Legions); PFileNode::SetNameAsInt64 is the text file node [vtable-checked: not a slot of any located Legions vtable]

### PackageIDPropertyValue
- [-] `PackageIDPropertyValue::HasValidData` · 006b4400 — never called: the Legions vtable slot is +0x20 (PackageIDPropertyValue vtable 0x009F0A90, slot 8 = 0x0064CDE0); no property code calls +0x20, and the 371 call/jmp [reg+0x20] sites in the Legions exe resolve to other classes (StringInfo, WeenieDesc); the authoring FromFileNode/ToFileNode beside them are absent from Legions

### BehaviorTracker
- [x] `BehaviorTracker::SetAnimationObj` · 006b4410 · legions 0064ce10 — audit: covered:  [ac2-script/src/glue/property_collection.rs:266]

### PackageIDPropertyValue
- [x] `PackageIDPropertyValue::Set` · 006b4420 — ac2-db base_property.rs: typed BasePropertyValue Inq/Set by type, same-type copy, Compare semantics (Legions Compare rows are the vtable form)
- [-] `PackageIDPropertyValue::Compare` · 006b4440 · legions 0064ce40 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)
- [-] `PackageIDPropertyValue::Copy` · 006b4480 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)

### VectorPropertyValue
- [-] `VectorPropertyValue::VectorPropertyValue` · 006b44b0 — skip: constructor/destructor of plain data; Rust struct init and Drop
- [-] `VectorPropertyValue::FromFileNode` · 006b44e0 — text source format (admin tools)
- [-] `VectorPropertyValue::ToFileNode` · 006b4500 — text source format (admin tools)
- [x] `VectorPropertyValue::InqVector` · 006b4520 — ac2-db base_property.rs: typed BasePropertyValue Inq/Set by type, same-type copy, Compare semantics (Legions Compare rows are the vtable form)
- [x] `VectorPropertyValue::SetVector` · 006b4540 — ac2-db base_property.rs: typed BasePropertyValue Inq/Set by type, same-type copy, Compare semantics (Legions Compare rows are the vtable form)
- [x] `VectorPropertyValue::Set` · 006b4560 — ac2-db base_property.rs: typed BasePropertyValue Inq/Set by type, same-type copy, Compare semantics (Legions Compare rows are the vtable form)
- [-] `VectorPropertyValue::Compare` · 006b4580 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)
- [-] `VectorPropertyValue::Copy` · 006b4600 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)
- [x] `VectorPropertyValue::StreamPack` · 006b4650 — audit: covered: pack direction unused by the client [ac2-net/src/protocol/property.rs:3-8 and ac2-db/src/property.rs:509 read_ac2_base_property_array (unpack)]

### WaveformPropertyValue
- [-] `WaveformPropertyValue::WaveformPropertyValue` · 006b46b0 · legions 0064d070 — skip: constructor/destructor of plain data; Rust struct init and Drop

### DataFilePropertyValue
- [-] `DataFilePropertyValue::HasValidData` · 006b46d0 — never called: the Legions vtable slot is +0x20 (PackageIDPropertyValue vtable 0x009F0A90, slot 8 = 0x0064CDE0); no property code calls +0x20, and the 371 call/jmp [reg+0x20] sites in the Legions exe resolve to other classes (StringInfo, WeenieDesc); the authoring FromFileNode/ToFileNode beside them are absent from Legions

### WaveformPropertyValue
- [-] `WaveformPropertyValue::FromFileNode` · 006b46e0 — text source format (admin tools)
- [-] `WaveformPropertyValue::ToFileNode` · 006b4710 — text source format (admin tools)
- [x] `WaveformPropertyValue::InqWaveform` · 006b4730 — ac2-db base_property.rs: typed BasePropertyValue Inq/Set by type, same-type copy, Compare semantics (Legions Compare rows are the vtable form)
- [x] `WaveformPropertyValue::SetWaveform` · 006b4750 — ac2-db base_property.rs: typed BasePropertyValue Inq/Set by type, same-type copy, Compare semantics (Legions Compare rows are the vtable form)
- [x] `WaveformPropertyValue::Set` · 006b47a0 — ac2-db base_property.rs: typed BasePropertyValue Inq/Set by type, same-type copy, Compare semantics (Legions Compare rows are the vtable form)
- [-] `WaveformPropertyValue::GetValueAsString` · 006b47c0 — not needed: admin property viewer text [GetValueAsString is only reached through GetOutputString <- CollideProperty::GetPropertyOutputStringW <- efun with no Legions wlib call site (catalog [-] row)]
- [-] `WaveformPropertyValue::Compare` · 006b47e0 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)

### Waveform
- [-] `Waveform::operator==` · 006b4830 · legions 0064d1a0 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)

### WaveformPropertyValue
- [-] `WaveformPropertyValue::Copy` · 006b4950 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)
- [x] `WaveformPropertyValue::StreamPack` · 006b49d0 — audit: covered: pack direction unused by the client [ac2-net/src/protocol/property.rs:3-8 and ac2-db/src/property.rs:509 read_ac2_base_property_array (unpack)]

### PositionPropertyValue
- [-] `PositionPropertyValue::PositionPropertyValue` · 006b4a20 — skip: constructor/destructor of plain data; Rust struct init and Drop
- [-] `PositionPropertyValue::HasValidData` · 006b4a60 — never called: the Legions vtable slot is +0x20 (PackageIDPropertyValue vtable 0x009F0A90, slot 8 = 0x0064CDE0); no property code calls +0x20, and the 371 call/jmp [reg+0x20] sites in the Legions exe resolve to other classes (StringInfo, WeenieDesc); the authoring FromFileNode/ToFileNode beside them are absent from Legions
- [-] `PositionPropertyValue::FromFileNode` · 006b4a70 — text source format (admin tools)
- [-] `PositionPropertyValue::ToFileNode` · 006b4a90 — text source format (admin tools)
- [x] `PositionPropertyValue::Set` · 006b4ab0 — ac2-db base_property.rs: typed BasePropertyValue Inq/Set by type, same-type copy, Compare semantics (Legions Compare rows are the vtable form)
- [-] `PositionPropertyValue::Compare` · 006b4ad0 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)
- [x] `PositionPropertyValue::StreamPack` · 006b4b80 — audit: covered: pack direction unused by the client [ac2-net/src/protocol/property.rs:3-8 and ac2-db/src/property.rs:509 read_ac2_base_property_array (unpack)]

### MasterProperty
- [x] `MasterProperty::InqPropertyNameEnum` · 006b4be0 · legions 0064d510 — ac2-db property names: MasterProperty name table (the m_emapper pairs, descrambled) read by ac2_property_name / ac2_property_name_enum; BaseProperty::SetPropertyName[W] (group and type from the desc, then a fresh value); tests property_tests.rs, tests/master_property_names.rs
- [x] `MasterProperty::InqPropertyNameString` · 006b4c50 · legions 0064d560 — ac2-db property names: MasterProperty name table (the m_emapper pairs, descrambled) read by ac2_property_name / ac2_property_name_enum; BaseProperty::SetPropertyName[W] (group and type from the desc, then a fresh value); tests property_tests.rs, tests/master_property_names.rs
- [x] `MasterProperty::StreamPack` · 006b4cc0 · legions 0064d5b0 — audit: covered:  [ac2-db/src/property.rs:97 Ac2PropertyTypes::decode_packed (type, group, data, report, required per desc)]

### BoolPropertyValue
- [-] `BoolPropertyValue::FromFileNode` · 006b4d80 — text source format (admin tools)

### FloatPropertyValue
- [-] `FloatPropertyValue::FromFileNode` · 006b4db0 — text source format (admin tools)

### IntegerPropertyValue
- [-] `IntegerPropertyValue::FromFileNode` · 006b4de0 — text source format (admin tools)

### LongIntegerPropertyValue
- [-] `LongIntegerPropertyValue::FromFileNode` · 006b4e10 — text source format (admin tools)

### StringPropertyValue
- [-] `StringPropertyValue::StringPropertyValue` · 006b4e40 · legions 0064d670 — skip: constructor/destructor of plain data; Rust struct init and Drop
- [x] `StringPropertyValue::Set` · 006b4e70 — ac2-db base_property.rs: typed BasePropertyValue Inq/Set by type, same-type copy, Compare semantics (Legions Compare rows are the vtable form)
- [-] `StringPropertyValue::HasValidData` · 006b4e90 — never called: the Legions vtable slot is +0x20 (PackageIDPropertyValue vtable 0x009F0A90, slot 8 = 0x0064CDE0); no property code calls +0x20, and the 371 call/jmp [reg+0x20] sites in the Legions exe resolve to other classes (StringInfo, WeenieDesc); the authoring FromFileNode/ToFileNode beside them are absent from Legions
- [-] `StringPropertyValue::Copy` · 006b4eb0 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)
- [-] `StringPropertyValue::FromFileNode` · 006b4f00 — text source format (admin tools)
- [x] `StringPropertyValue::InqString` · 006b4f30 — ac2-db base_property.rs: typed BasePropertyValue Inq/Set by type, same-type copy, Compare semantics (Legions Compare rows are the vtable form)
- [x] `StringPropertyValue::SetString` · 006b4f50 — ac2-db base_property.rs: typed BasePropertyValue Inq/Set by type, same-type copy, Compare semantics (Legions Compare rows are the vtable form)
- [-] `StringPropertyValue::GetValueAsString` · 006b4f70 — not needed: admin property viewer text [GetValueAsString is only reached through GetOutputString <- CollideProperty::GetPropertyOutputStringW <- efun with no Legions wlib call site (catalog [-] row)]
- [-] `StringPropertyValue::Compare` · 006b4f90 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)

### StringInfoPropertyValue
- [-] `StringInfoPropertyValue::StringInfoPropertyValue` · 006b5050 · legions 0064d850 — skip: constructor/destructor of plain data; Rust struct init and Drop
- [-] `StringInfoPropertyValue::FromFileNode` · 006b5080 — text source format (admin tools)
- [-] `StringInfoPropertyValue::ToFileNode` · 006b50b0 — text source format (admin tools)
- [x] `StringInfoPropertyValue::InqStringInfo` · 006b50e0 — ac2-db BasePropertyValue::StringInfo (StringInfoValue: token + RenderInfo); inq/set_string_info, set_from, is_equal (string_info_equal); tests base_property_tests.rs
- [x] `StringInfoPropertyValue::SetStringInfo` · 006b5100 — ac2-db BasePropertyValue::StringInfo (StringInfoValue: token + RenderInfo); inq/set_string_info, set_from, is_equal (string_info_equal); tests base_property_tests.rs
- [x] `StringInfoPropertyValue::Set` · 006b5120 — ac2-db BasePropertyValue::StringInfo (StringInfoValue: token + RenderInfo); inq/set_string_info, set_from, is_equal (string_info_equal); tests base_property_tests.rs
- [-] `StringInfoPropertyValue::HasValidData` · 006b5140 — never called: the Legions vtable slot is +0x20 (PackageIDPropertyValue vtable 0x009F0A90, slot 8 = 0x0064CDE0); no property code calls +0x20, and the 371 call/jmp [reg+0x20] sites in the Legions exe resolve to other classes (StringInfo, WeenieDesc); the authoring FromFileNode/ToFileNode beside them are absent from Legions
- [x] `StringInfoPropertyValue::StreamPack` · 006b5160 — audit: covered: pack direction unused by the client [ac2-net/src/protocol/property.rs:3-8 and ac2-db/src/property.rs:509 read_ac2_base_property_array (unpack)]

### (free functions)
- [x] `STREAMPACK_PACKOBJ` · 006b5190 · legions 00578ab0 — audit: covered:  [generic PackObj member stream thunk; owners decode in order (ac2-assets/src/ac2_mesh.rs:6, UIScene/LightSourceFX decoders)]

### StringInfoPropertyValue
- [x] `StringInfoPropertyValue::Copy` · 006b5200 — ac2-db BasePropertyValue::StringInfo (StringInfoValue: token + RenderInfo); inq/set_string_info, set_from, is_equal (string_info_equal); tests base_property_tests.rs
- [x] `StringInfoPropertyValue::InqStringInfo` · 006b5260 · legions 0064d990 — audit: covered:  [ac2-db/src/base_property.rs:281,374 (1-argument form; the 2-argument token-out form has no efun)]
- [x] `StringInfoPropertyValue::SetStringInfo` · 006b5290 — audit: covered:  [ac2-db/src/base_property.rs:281,374 (1-argument form; the 2-argument token-out form has no efun)]
- [x] `StringInfoPropertyValue::Compare` · 006b52c0 — ac2-db BasePropertyValue::StringInfo (StringInfoValue: token + RenderInfo); inq/set_string_info, set_from, is_equal (string_info_equal); tests base_property_tests.rs

### PositionPropertyValue
- [-] `PositionPropertyValue::Copy` · 006b5410 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)
- [x] `PositionPropertyValue::InqPosition` · 006b5480 — ac2-db base_property.rs: typed BasePropertyValue Inq/Set by type, same-type copy, Compare semantics (Legions Compare rows are the vtable form)
- [x] `PositionPropertyValue::SetPosition` · 006b54a0 — ac2-db base_property.rs: typed BasePropertyValue Inq/Set by type, same-type copy, Compare semantics (Legions Compare rows are the vtable form)

### MasterProperty
- [x] `MasterProperty::AllocatePropertyValue` · 006b54c0 · legions 0064dbf0 — audit: covered:  [ac2-db/src/base_property.rs:167]

### BoolPropertyValue
- [-] `BoolPropertyValue::GetValueAsString` · 006b5680 — not needed: admin property viewer text [GetValueAsString is only reached through GetOutputString <- CollideProperty::GetPropertyOutputStringW <- efun with no Legions wlib call site (catalog [-] row)]

### ColorPropertyValue
- [-] `ColorPropertyValue::GetValueAsString` · 006b56b0 — not needed: admin property viewer text [GetValueAsString is only reached through GetOutputString <- CollideProperty::GetPropertyOutputStringW <- efun with no Legions wlib call site (catalog [-] row)]

### FloatPropertyValue
- [-] `FloatPropertyValue::GetValueAsString` · 006b56f0 — not needed: admin property viewer text [GetValueAsString is only reached through GetOutputString <- CollideProperty::GetPropertyOutputStringW <- efun with no Legions wlib call site (catalog [-] row)]

### IntegerPropertyValue
- [-] `IntegerPropertyValue::GetValueAsString` · 006b5720 — not needed: admin property viewer text [GetValueAsString is only reached through GetOutputString <- CollideProperty::GetPropertyOutputStringW <- efun with no Legions wlib call site (catalog [-] row)]

### LongIntegerPropertyValue
- [-] `LongIntegerPropertyValue::GetValueAsString` · 006b5740 — not needed: admin property viewer text [GetValueAsString is only reached through GetOutputString <- CollideProperty::GetPropertyOutputStringW <- efun with no Legions wlib call site (catalog [-] row)]

### StringPropertyValue
- [-] `StringPropertyValue::ToFileNode` · 006b5770 — text source format (admin tools)

### StringInfoPropertyValue
- [-] `StringInfoPropertyValue::GetValueAsString` · 006b57b0 — not needed: admin property viewer text [GetValueAsString is only reached through GetOutputString <- CollideProperty::GetPropertyOutputStringW <- efun with no Legions wlib call site (catalog [-] row)]

### VectorPropertyValue
- [-] `VectorPropertyValue::GetValueAsString` · 006b5880 — not needed: admin property viewer text [GetValueAsString is only reached through GetOutputString <- CollideProperty::GetPropertyOutputStringW <- efun with no Legions wlib call site (catalog [-] row)]

### StringPropertyValue
- [x] `StringPropertyValue::StreamPack` · 006b5950 — audit: covered: pack direction unused by the client [ac2-net/src/protocol/property.rs:3-8 and ac2-db/src/property.rs:509 read_ac2_base_property_array (unpack)]

### (free functions)
- [x] `STREAMPACK_PACKOBJ` · 006b5980 — audit: covered:  [generic PackObj member stream thunk; owners decode in order (ac2-assets/src/ac2_mesh.rs:6, UIScene/LightSourceFX decoders)]

### MasterProperty
- [-] `MasterProperty::InqOptionalPropertiesInternal` · 006b5a30 · legions 0064e0f0 — not needed:  [only from MasterProperty::InqOptionalProperties <- CPhysicsObj::BuildOptionalProperties (editor tooling, catalog [-]); m_defaultValue is null in the client]
- [-] `MasterProperty::GetSubDataIDs` · 006b5af0 — not needed:  [data-graph prefetch / StringDownload (tool) only; catalog policy rows (CharacterSetup::GetSubDataIDs etc.) are [-]; m_pDataGraph never set]

### PositionPropertyValue
- [-] `PositionPropertyValue::GetValueAsString` · 006b5b60 — not needed: admin property viewer text [GetValueAsString is only reached through GetOutputString <- CollideProperty::GetPropertyOutputStringW <- efun with no Legions wlib call site (catalog [-] row)]

### MasterProperty
- [x] `MasterProperty::GetPropertyDesc` · 006b5bd0 · legions 0064e2d0 — audit: covered:  [ac2-db/src/property.rs:80,210]
- [x] `MasterProperty::InqPropertyBit` · 006b5c40 — audit: covered:  [ac2-db/src/property.rs:210 ac2_property_desc (data word = bit); :235 ethereal_mask_with; ac2-physics/src/body_profile.rs:36]
- [-] `MasterProperty::InqOptionalProperties` · 006b5c70 · legions 0064e340 — not needed:  [only from MasterProperty::InqOptionalProperties <- CPhysicsObj::BuildOptionalProperties (editor tooling, catalog [-]); m_defaultValue is null in the client]
- [-] `MasterProperty::Destroy` · 006b5ce0 · legions 0064e500 — skip: constructor/destructor of plain data; Rust struct init and Drop

### HashTable<unsigned_long,BasePropertyDesc>
- [-] `HashTable<unsigned_long,BasePropertyDesc>::operator=` · 006b5d00 · legions 0064e520 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)

### MasterProperty
- [x] `MasterProperty::InqPropertyType` · 006b5f40 — ac2_property_type in crates/ac2-db/src/property.rs cites MasterProperty::InqPropertyType
- [x] `MasterProperty::InqPropertyGroup` · 006b5fd0 — audit: covered:  [ac2-db/src/property_collection.rs:121]
- [x] `MasterProperty::IsReportedToWeenie` · 006b6060 — audit: covered:  [ac2-db/src/property.rs:214,220]
- [x] `MasterProperty::IsInquirable` · 006b60f0 — audit: covered:  [ac2-db/src/property.rs:214,220]
- [-] `MasterProperty::~MasterProperty` · 006b61f0 · legions 0064e860 — skip: constructor/destructor of plain data; Rust struct init and Drop
- [-] `MasterProperty::GetDBOType` · 006b6270 — skip: retail DBO type tag; Rust typed enum in ac2-db db_type.rs

### PAutoGrowHashTable<unsigned_long,BasePropertyDesc>
- [x] `PAutoGrowHashTable<unsigned_long,BasePropertyDesc>::StreamPack` · 006b6280 — audit: covered:  [ac2-db/src/property.rs:97 Ac2PropertyTypes::decode_packed (type, group, data, report, required per desc)]

### MasterProperty
- [-] `MasterProperty::CopyInto` · 006b6480 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)
- [-] `MasterProperty::LoadFromFile` · 006b64a0 — text source loader, admin build only
- [-] `MasterProperty::MasterProperty` · 006b6670 · legions 0064eb10 — skip: constructor/destructor of plain data; Rust struct init and Drop
- [-] `MasterProperty::Allocate` · 006b66d0 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

## `portal/shared/src/property/propertycollection.cpp.c` (18/26)

### PropertyCollection
- [-] `PropertyCollection::Destroy` · 006b31b0 · legions 0064c070 — skip: constructor/destructor of plain data; Rust struct init and Drop
- [x] `PropertyCollection::SetOptionalToRequired` · 006b31f0 · legions 0064c0b0 — ac2-db property_collection.rs (PropertyCollection/PropertyGroup over BaseProperty; group lookup by the MasterProperty group of the name, as GetProperty does)
- [-] `PropertyCollection::GetSubDataIDs` · 006b3230 · legions 0064c0f0 — not needed:  [data-graph prefetch / StringDownload (tool) only; catalog policy rows (CharacterSetup::GetSubDataIDs etc.) are [-]; m_pDataGraph never set]
- [x] `PropertyCollection::GetGroup` · 006b3270 · legions 0064c130 — ac2-db property_collection.rs (PropertyCollection/PropertyGroup over BaseProperty; group lookup by the MasterProperty group of the name, as GetProperty does)
- [x] `PropertyCollection::GetProperty` · 006b32b0 · legions 0064c170 — ac2-db property_collection.rs (PropertyCollection/PropertyGroup over BaseProperty; group lookup by the MasterProperty group of the name, as GetProperty does)
- [x] `PropertyCollection::GetOptionalProperty` · 006b3320 · legions 0064c1e0 — ac2-db property_collection.rs (PropertyCollection/PropertyGroup over BaseProperty; group lookup by the MasterProperty group of the name, as GetProperty does)
- [x] `PropertyCollection::InqGroup` · 006b3390 · legions 0064c250 — ac2-db property_collection.rs (PropertyCollection/PropertyGroup over BaseProperty; group lookup by the MasterProperty group of the name, as GetProperty does)
- [x] `PropertyCollection::InqProperty` · 006b33e0 · legions 0064c2a0 — ac2-db property_collection.rs (PropertyCollection/PropertyGroup over BaseProperty; group lookup by the MasterProperty group of the name, as GetProperty does)
- [x] `PropertyCollection::InqOptionalProperty` · 006b3410 · legions 0064c2d0 — ac2-db property_collection.rs (PropertyCollection/PropertyGroup over BaseProperty; group lookup by the MasterProperty group of the name, as GetProperty does)
- [-] `PropertyCollection::PropertyCollection` · 006b3440 · legions 0064c300 — skip: constructor/destructor of plain data; Rust struct init and Drop
- [-] `PropertyCollection::~PropertyCollection` · 006b3460 · legions 0064c320 — skip: constructor/destructor of plain data; Rust struct init and Drop
- [x] `PropertyCollection::InqGroupAt` · 006b34e0 · legions 0064c3a0 — ac2-db property_collection.rs (PropertyCollection/PropertyGroup over BaseProperty; group lookup by the MasterProperty group of the name, as GetProperty does)
- [x] `PropertyCollection::ContainsGroup` · 006b3510 · legions 0064c3d0 — ac2-db property_collection.rs (PropertyCollection/PropertyGroup over BaseProperty; group lookup by the MasterProperty group of the name, as GetProperty does)
- [x] `PropertyCollection::ContainsProperty` · 006b3560 · legions 0064c420 — ac2-db property_collection.rs (PropertyCollection/PropertyGroup over BaseProperty; group lookup by the MasterProperty group of the name, as GetProperty does)
- [x] `PropertyCollection::InqPropertyW` · 006b3580 · legions 0064c440 — ac2-db property_collection.rs (PropertyCollection/PropertyGroup over BaseProperty; group lookup by the MasterProperty group of the name, as GetProperty does)
- [x] `PropertyCollection::InqOptionalPropertyW` · 006b3600 · legions 0064c4c0 — ac2-db property_collection.rs (PropertyCollection/PropertyGroup over BaseProperty; group lookup by the MasterProperty group of the name, as GetProperty does)

### SmartArray<PropertyGroup>
- [-] `SmartArray<PropertyGroup>::AddToEnd` · 006b3680 · legions 0064c540 — skip: STL/container internal (iterator, tree, comparator); Rust collections (SmartArray append; Rust Vec push)

### PropertyCollection
- [x] `PropertyCollection::AddGroup` · 006b36d0 · legions 0064c590 — ac2-db property_collection.rs (PropertyCollection/PropertyGroup over BaseProperty; group lookup by the MasterProperty group of the name, as GetProperty does)
- [x] `PropertyCollection::UpdateGroup` · 006b3700 · legions 0064c5c0 — ac2-db property_collection.rs (PropertyCollection/PropertyGroup over BaseProperty; group lookup by the MasterProperty group of the name, as GetProperty does)
- [x] `PropertyCollection::UpdateCollection` · 006b3770 · legions 0064c630 — ac2-db property_collection.rs (PropertyCollection/PropertyGroup over BaseProperty; group lookup by the MasterProperty group of the name, as GetProperty does)
- [-] `PropertyCollection::ToFileNode` · 006b37c0 — text source format (admin tools)
- [-] `PropertyCollection::FromFileNode` · 006b3910 — text source format (admin tools)
- [-] `PropertyCollection::operator=` · 006b3b10 · legions 0064c680 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)
- [x] `PropertyCollection::AddProperty` · 006b3b50 · legions 0064c6c0 — ac2-db property_collection.rs (PropertyCollection/PropertyGroup over BaseProperty; group lookup by the MasterProperty group of the name, as GetProperty does)
- [x] `PropertyCollection::AddOptionalProperty` · 006b3c20 · legions 0064c790 — ac2-db property_collection.rs (PropertyCollection/PropertyGroup over BaseProperty; group lookup by the MasterProperty group of the name, as GetProperty does)
- [x] `PropertyCollection::UpdateProperty` · 006b3cf0 · legions 0064c860 — ac2-db property_collection.rs (PropertyCollection/PropertyGroup over BaseProperty; group lookup by the MasterProperty group of the name, as GetProperty does)

## `portal/shared/src/property/propertygroup.cpp.c` (16/25)

### PropertyGroup
- [-] `PropertyGroup::InqPropertyGroupName` · 006b2890 — not needed:  [only caller PropertyCollection::ToFileNode (tool); the W efun is ac2-script/src/glue/property_collection.rs:19]
- [-] `PropertyGroup::Destroy` · 006b28b0 · legions 0064b8e0 — skip: constructor/destructor of plain data; Rust struct init and Drop
- [-] `PropertyGroup::GetSubDataIDs` · 006b2920 · legions 0064b950 — not needed:  [data-graph prefetch / StringDownload (tool) only; catalog policy rows (CharacterSetup::GetSubDataIDs etc.) are [-]; m_pDataGraph never set]
- [x] `PropertyGroup::StreamPack` · 006b2960 · legions 0064b990 — audit: covered: pack direction unused by the client [ac2-net/src/protocol/property.rs:3-8 and ac2-db/src/property.rs:509 read_ac2_base_property_array (unpack)]
- [-] `PropertyGroup::ToFileNode` · 006b29b0 — text source format (admin tools)
- [x] `PropertyGroup::RemoveProperty` · 006b2a00 · legions 0064b9e0 — ac2-db property_collection.rs (PropertyCollection/PropertyGroup over BaseProperty; group lookup by the MasterProperty group of the name, as GetProperty does)
- [x] `PropertyGroup::GetProperty` · 006b2a80 · legions 0064ba60 — ac2-db property_collection.rs (PropertyCollection/PropertyGroup over BaseProperty; group lookup by the MasterProperty group of the name, as GetProperty does)
- [x] `PropertyGroup::GetOptionalProperty` · 006b2ad0 · legions 0064bab0 — ac2-db property_collection.rs (PropertyCollection/PropertyGroup over BaseProperty; group lookup by the MasterProperty group of the name, as GetProperty does)
- [x] `PropertyGroup::InqProperty` · 006b2b20 · legions 0064bb00 — ac2-db property_collection.rs (PropertyCollection/PropertyGroup over BaseProperty; group lookup by the MasterProperty group of the name, as GetProperty does)
- [x] `PropertyGroup::InqOptionalProperty` · 006b2b80 · legions 0064bb60 — ac2-db property_collection.rs (PropertyCollection/PropertyGroup over BaseProperty; group lookup by the MasterProperty group of the name, as GetProperty does)
- [-] `PropertyGroup::PropertyGroup` · 006b2be0 · legions 0064bbc0 — skip: constructor/destructor of plain data; Rust struct init and Drop
- [-] `PropertyGroup::~PropertyGroup` · 006b2c10 · legions 0064bbf0 — skip: constructor/destructor of plain data; Rust struct init and Drop
- [x] `PropertyGroup::InqPropertyAt` · 006b2c80 · legions 0064bc60 — ac2-db property_collection.rs (PropertyCollection/PropertyGroup over BaseProperty; group lookup by the MasterProperty group of the name, as GetProperty does)
- [x] `PropertyGroup::InqOptionalPropertyAt` · 006b2cb0 · legions 0064bc90 — ac2-db property_collection.rs (PropertyCollection/PropertyGroup over BaseProperty; group lookup by the MasterProperty group of the name, as GetProperty does)
- [x] `PropertyGroup::InqPropertyGroupNameW` · 006b2ce0 · legions 0064bcc0 — ac2-db property_collection.rs (PropertyCollection/PropertyGroup over BaseProperty; group lookup by the MasterProperty group of the name, as GetProperty does)
- [x] `PropertyGroup::ContainsProperty` · 006b2db0 · legions 0064bd90 — ac2-db property_collection.rs (PropertyCollection/PropertyGroup over BaseProperty; group lookup by the MasterProperty group of the name, as GetProperty does)
- [x] `PropertyGroup::ContainsOptionalProperty` · 006b2dd0 · legions 0064bdb0 — ac2-db property_collection.rs (PropertyCollection/PropertyGroup over BaseProperty; group lookup by the MasterProperty group of the name, as GetProperty does)

### SmartArray<BaseProperty>
- [-] `SmartArray<BaseProperty>::AddToEnd` · 006b2df0 · legions 0064bdd0 — skip: STL/container internal (iterator, tree, comparator); Rust collections (SmartArray append; Rust Vec push)

### PropertyGroup
- [x] `PropertyGroup::SetOptionalToRequired` · 006b2e40 · legions 0064be20 — ac2-db property_collection.rs (PropertyCollection/PropertyGroup over BaseProperty; group lookup by the MasterProperty group of the name, as GetProperty does)
- [x] `PropertyGroup::AddProperty` · 006b2e80 · legions 0064be60 — ac2-db property_collection.rs (PropertyCollection/PropertyGroup over BaseProperty; group lookup by the MasterProperty group of the name, as GetProperty does)
- [x] `PropertyGroup::AddOptionalProperty` · 006b2eb0 · legions 0064be90 — ac2-db property_collection.rs (PropertyCollection/PropertyGroup over BaseProperty; group lookup by the MasterProperty group of the name, as GetProperty does)
- [x] `PropertyGroup::UpdateProperty` · 006b2ee0 · legions 0064bec0 — ac2-db property_collection.rs (PropertyCollection/PropertyGroup over BaseProperty; group lookup by the MasterProperty group of the name, as GetProperty does)
- [x] `PropertyGroup::UpdateGroup` · 006b2ff0 · legions 0064bfd0 — ac2-db property_collection.rs (PropertyCollection/PropertyGroup over BaseProperty; group lookup by the MasterProperty group of the name, as GetProperty does)
- [-] `PropertyGroup::FromFileNode` · 006b3030 — text source format (admin tools)
- [-] `PropertyGroup::operator=` · 006b3150 · legions 0064c010 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)

## `portal/shared/src/property/value/datafilepropertyvalue.cpp.c` (0/3)

### DataFilePropertyValue
- [-] `DataFilePropertyValue::FromFileNode` · 006b66e0 — text source format (admin tools)
- [-] `DataFilePropertyValue::ToFileNode` · 006b6810 — text source format (admin tools)
- [-] `DataFilePropertyValue::GetValueAsString` · 006b6900 — not needed: admin property viewer text [GetValueAsString is only reached through GetOutputString <- CollideProperty::GetPropertyOutputStringW <- efun with no Legions wlib call site (catalog [-] row)]

## `portal/shared/src/property/value/enumpropertyvalue.cpp.c` (0/3)

### EnumPropertyValue
- [-] `EnumPropertyValue::GetValueAsString` · 006b69f0 — not needed: admin property viewer text [GetValueAsString is only reached through GetOutputString <- CollideProperty::GetPropertyOutputStringW <- efun with no Legions wlib call site (catalog [-] row)]
- [-] `EnumPropertyValue::FromFileNode` · 006b6a10 — text source format (admin tools)
- [-] `EnumPropertyValue::ToFileNode` · 006b6b00 — text source format (admin tools)

## `portal/shared/src/property/value/packageidpropertyvalue.cpp.c` (0/3)

### PackageIDPropertyValue
- [-] `PackageIDPropertyValue::FromFileNode` · 006b6bf0 — text source format (admin tools)
- [-] `PackageIDPropertyValue::ToFileNode` · 006b6ce0 — text source format (admin tools)
- [-] `PackageIDPropertyValue::GetValueAsString` · 006b6db0 — not needed: admin property viewer text [GetValueAsString is only reached through GetOutputString <- CollideProperty::GetPropertyOutputStringW <- efun with no Legions wlib call site (catalog [-] row)]

## `portal/shared/src/stringtable/metalanguage/metalanguage.cpp.c` (7/8)

### StringTableMetaLanguage
- [x] `StringTableMetaLanguage::GetMatchScore` · 00657600 · legions 00605f50 — ac2-db string_meta_render (render_string; Legions code identical)
- [x] `StringTableMetaLanguage::SetError` · 006576c0 · legions 00606010 — ac2-db string_meta_render (render_string; Legions code identical)
- [x] `StringTableMetaLanguage::SetError` · 00657720 — ac2-db string_meta_render (render_string; Legions code identical)
- [x] `StringTableMetaLanguage::ChooseChoice` · 00657780 · legions 006060d0 — ac2-db string_meta_render (render_string; Legions code identical)

### SmartArray<PStringBase<unsigned_short>_>
- [-] `SmartArray<PStringBase<unsigned_short>_>::operator=` · 006577d0 · legions 00606120 — Rust ownership (Vec/drop)

### StringTableMetaLanguage
- [x] `StringTableMetaLanguage::TakeValidLetters` · 00657860 · legions 006061b0 — ac2-db string_meta_render (render_string; Legions code identical)
- [x] `StringTableMetaLanguage::RenderNodes` · 00657960 · legions 006062b0 — ac2-db string_meta_render (render_string; Legions code identical)
- [x] `StringTableMetaLanguage::RenderString` · 00658660 · legions 00606fb0 — ac2-db string_meta_render (render_string; Legions code identical)

## `portal/shared/src/stringtable/metalanguage/metalanguage_rules.cpp.c` (13/15)

### StringTableMetaLanguage
- [x] `StringTableMetaLanguage::GetRoot` · 00658b00 · legions 00607450 — ac2-db string_meta_parse (grammar checked against the bison tables, identical in Legions)
- [x] `StringTableMetaLanguage::IsInChoiceBlock` · 00658b10 · legions 00607460 — ac2-db string_meta_parse (grammar checked against the bison tables, identical in Legions)
- [x] `StringTableMetaLanguage::AddNode` · 00658b30 · legions 00607480 — ac2-db string_meta_parse (grammar checked against the bison tables, identical in Legions)
- [x] `StringTableMetaLanguage::r_Flags` · 00658ba0 — ac2-db string_meta_parse (grammar checked against the bison tables, identical in Legions)
- [x] `StringTableMetaLanguage::CheckForErrorsInChoiceBlock` · 00658bf0 — ac2-db string_meta_parse (grammar checked against the bison tables, identical in Legions)

### StringTableMetaLanguage::Node
- [-] `StringTableMetaLanguage::Node::Reset` · 00658c70 · legions 006075c0 — Rust ownership (Vec/drop)

### StringTableMetaLanguage
- [x] `StringTableMetaLanguage::r_ChoiceBlockEnd` · 00658cd0 — ac2-db string_meta_parse (grammar checked against the bison tables, identical in Legions)

### StringTableMetaLanguage::Node
- [x] `StringTableMetaLanguage::Node::Node` · 00658cf0 · legions 00607640 — ac2-db string_meta_parse (grammar checked against the bison tables, identical in Legions)
- [-] `StringTableMetaLanguage::Node::~Node` · 00658d50 · legions 006076a0 — Rust ownership (Vec/drop)

### StringTableMetaLanguage
- [x] `StringTableMetaLanguage::InitRules` · 00658db0 · legions 00607700 — ac2-db string_meta_parse (grammar checked against the bison tables, identical in Legions)
- [x] `StringTableMetaLanguage::CleanRules` · 00658df0 · legions 00607740 — ac2-db string_meta_parse (grammar checked against the bison tables, identical in Legions)
- [x] `StringTableMetaLanguage::r_Text` · 00658e40 · legions 00607790 — ac2-db string_meta_parse (grammar checked against the bison tables, identical in Legions)
- [x] `StringTableMetaLanguage::r_Var` · 00658ea0 — ac2-db string_meta_parse; Legions adds the + flag for non-negative numbers
- [x] `StringTableMetaLanguage::r_ChoiceBlockBegin` · 006592b0 — ac2-db string_meta_parse (grammar checked against the bison tables, identical in Legions)
- [x] `StringTableMetaLanguage::r_OutsideInfoBlock` · 006592f0 · legions 00607c60 — ac2-db string_meta_parse (grammar checked against the bison tables, identical in Legions)

## `portal/shared/src/stringtable/metalanguage/metalanguage_tokenizer.cpp.c` (7/7)

### (free functions)
- [x] `IsSpecialChar` · 00659920 · legions 00608290 — ac2-db string_meta_lex

### StringTableMetaLanguage
- [x] `StringTableMetaLanguage::GetCurrentStreamPosition` · 00659960 · legions 006082d0 — ac2-db string_meta_lex

### (free functions)
- [x] `IsNumberLabel` · 00659970 · legions 006082e0 — ac2-db string_meta_lex
- [x] `stringmeta_error` · 00659c10 · legions 00608580 — ac2-db string_meta_lex
- [x] `IsFlagsBlock` · 00659c80 · legions 006085f0 — ac2-db string_meta_lex
- [x] `stringmeta_lex` · 0065a300 · legions 00608c70 — ac2-db string_meta_lex

### StringTableMetaLanguage
- [x] `StringTableMetaLanguage::LoadTokenizer` · 0065a310 · legions 00608c80 — ac2-db string_meta_lex

## `portal/shared/src/stringtable/metalanguage/metalanguage_utils.cpp.c` (14/14)

### StringTableMetaLanguage
- [x] `StringTableMetaLanguage::IsEscapeChar` · 00656b20 · legions 00605470 — ac2-db string_meta_text / string_state (CRT C locale classes)
- [x] `StringTableMetaLanguage::IsNumberSingular` · 00656b50 — ac2-db string_meta_text / string_state (CRT C locale classes)
- [x] `StringTableMetaLanguage::GetMaleMetaLetters` · 00656b80 — ac2-db string_meta_text / string_state (CRT C locale classes)
- [x] `StringTableMetaLanguage::GetFemaleMetaLetters` · 00656bb0 — ac2-db string_meta_text / string_state (CRT C locale classes)
- [x] `StringTableMetaLanguage::GetTreasureMetaLetters` · 00656be0 · legions 00605530 — ac2-db string_meta_text / string_state (CRT C locale classes)

### PStringBase<unsigned_short>
- [x] `PStringBase<unsigned_short>::SetAtIndex` · 00656c40 · legions 00605590 — ac2-db string_meta_text / string_state (CRT C locale classes)

### PSUtils
- [x] `PSUtils::TrimExcessSpaces` · 00656cb0 · legions 00605600 — ac2-script glue/taboo.rs trim_excess_spaces

### StringTableMetaLanguage
- [x] `StringTableMetaLanguage::TrimExcessSpaces` · 00656ea0 · legions 006057f0 — ac2-db string_meta_text / string_state (CRT C locale classes)
- [x] `StringTableMetaLanguage::CapitalizeFirstLetter` · 00656ee0 · legions 00605830 — ac2-db string_meta_text / string_state (CRT C locale classes)
- [x] `StringTableMetaLanguage::RetrieveMetaLetters` · 00656fd0 · legions 00605920 — ac2-db string_meta_text / string_state (CRT C locale classes)
- [x] `StringTableMetaLanguage::StripMetaLetters` · 006570f0 · legions 00605a40 — ac2-db string_meta_text / string_state (CRT C locale classes)
- [x] `StringTableMetaLanguage::EscapeString` · 006571c0 · legions 00605b10 — ac2-db string_meta_text / string_state (CRT C locale classes)
- [x] `StringTableMetaLanguage::UnescapeString` · 006572d0 · legions 00605c20 — ac2-db string_meta_text / string_state (CRT C locale classes)
- [x] `StringTableMetaLanguage::ApplyMetaTagsToName` · 006573f0 · legions 00605d40 — ac2-db string_meta_text / string_state (CRT C locale classes)

## `portal/shared/src/stringtable/stringinfo.cpp.c` (39/56)

### StringInfoData
- [-] `StringInfoData::operator==` · 00650880 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)
- [-] `StringInfoData::operator!=` · 006508b0 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)

### StringInfo
- [x] `StringInfo::SetTableID` · 006508d0 · legions 005ffba0 — ac2-script glue::string_text (tables via ScriptData::string_table)
- [x] `StringInfo::ApplyMetasToPlayerName` · 00650900 · legions 005ffbd0 — ac2-db string_info over RenderInfo (StringTables trait); ac2-script glue::string_text
- [-] `StringInfo::ReportError` · 00650920 — logging only
- [x] `StringInfo::SetTableEnum` · 00650a50 · legions 005ffc10 — ac2-script glue::string_text (tables via ScriptData::string_table)
- [x] `StringInfo::SetStringIDandTableEnum` · 00650a90 · legions 005ffc50 — ac2-script glue::string_info (lookups via ScriptData)
- [x] `StringInfo::SetStringIDandTableID` · 00650ad0 · legions 005ffc90 — ac2-script glue::string_info (lookups via ScriptData)
- [x] `StringInfo::operator%` · 00650b40 · legions 005ffd00 — audit: covered:  [ac2-script/src/glue/hashes.rs:115]
- [x] `StringInfo::SetTableName` · 00650b90 · legions 005ffd50 — ac2-script glue::string_info (lookups via ScriptData)
- [x] `StringInfo::SetStringIDandTableName` · 00650c50 · legions 005ffdf0 — ac2-script glue::string_info (lookups via ScriptData)
- [x] `StringInfo::GetString` · 00650c70 · legions 005ffe10 — ac2-db string_info over RenderInfo (StringTables trait); ac2-script glue::string_text
- [x] `StringInfo::GetStringWithMetaTags` · 00650ce0 · legions 005ffe80 — ac2-db string_info over RenderInfo (StringTables trait); ac2-script glue::string_text
- [x] `StringInfo::InqString` · 00650d00 · legions 005ffea0 — ac2-db string_info over RenderInfo (StringTables trait); ac2-script glue::string_text
- [-] `StringInfo::InqStringTableFile` · 00650da0 — skip: admin/debug text or tool code (no client use in Legions) (StringInfo::InqStringTableFile, table file name) [vtable-checked: not a slot of any located Legions vtable]
- [x] `StringInfo::SetLiteralValue` · 00650e30 · legions 005fffd0 — ac2-script glue::string_info (lookups via ScriptData)
- [x] `StringInfo::GetLiteralValue` · 00650ef0 · legions 00600090 — audit: covered:  [ac2-weenie/src/gun/chat_extra.rs:12]
- [x] `StringInfo::SetVarInTable` · 00651000 · legions 006001a0 — ac2-db string_info over RenderInfo (StringTables trait); ac2-script glue::string_text
- [x] `StringInfo::GetLogString` · 00651220 · legions 00600410 — ac2-db string_info over RenderInfo (StringTables trait); ac2-script glue::string_text
- [-] `StringInfo::FromFileNode` · 00651390 · legions 00600580 — text source format (admin tools)
- [-] `StringInfo::FromFileNode` · 006515c0 · legions 006007b0 — text source format (admin tools)
- [-] `StringInfo::ToFileNode` · 00651620 — text source format (admin tools)
- [x] `StringInfo::IsValid` · 00651730 · legions 00600840 — ac2-script glue::string_text (tables via ScriptData::string_table)
- [x] `StringInfo::GetSubDataIDs` · 00651860 · legions 00600950 — audit: covered:  [ac2-script/src/glue/sub_data_ids.rs:19]
- [-] `StringInfo::GetSubPrivateIDs` · 006518f0 · legions 006009e0 — not needed:  [data-graph prefetch / StringDownload (tool) only; catalog policy rows (CharacterSetup::GetSubDataIDs etc.) are [-]; m_pDataGraph never set]
- [-] `StringInfo::GetPackSize` · 006519d0 · legions 00600aa0 — skip: DAT writing/saving/packing (tools only) (StringInfo::GetPackSize)
- [x] `StringInfo::Pack` · 00651b20 · legions 00600bf0
- [x] `StringInfo::InqStringInternal` · 00651dd0 · legions 00600ea0 — ac2-db string_info over RenderInfo (StringTables trait); ac2-script glue::string_text
- [-] `StringInfo::ClearVars` · 00651fb0 · legions 00601080 — skip: STL/container internal (iterator, tree, comparator); Rust collections (variable list clear; Rust builds a fresh RenderInfo)
- [x] `StringInfo::Reset` · 00652070 · legions 00601140 — ac2-script glue::string_info (lookups via ScriptData)
- [x] `StringInfo::operator==` · 00652080 · legions 00601150 — ac2-script glue::string_text (tables via ScriptData::string_table)
- [x] `StringInfo::IsReady` · 00652140 · legions 00601210 — audit: covered:  [ac2-ui/src/ui_text.rs:267; ac2-ui/src/item_display.rs:45]
- [-] `StringInfo::~StringInfo` · 00652150 · legions 00601220 — skip: constructor/destructor of plain data; Rust struct init and Drop
- [x] `StringInfo::AddVar` · 006521d0 · legions 006012a0 — ac2-script glue::string_info (lookups via ScriptData)
- [x] `StringInfo::AddVariable_Int` · 006522b0 · legions 00601380 — audit: covered:  [ac2-db/src/string_info.rs:42-100 RenderVar variants (Int, FormattedInt, UInt, ..., Float, FormattedFloat, Info, String); ac2-ui/src/pregame/strings.rs:134; ac2-weenie/src/chat.rs:47]
- [x] `StringInfo::AddVariable_FormattedInt` · 00652330 · legions 00601400 — audit: covered:  [ac2-db/src/string_info.rs:42-100 RenderVar variants (Int, FormattedInt, UInt, ..., Float, FormattedFloat, Info, String); ac2-ui/src/pregame/strings.rs:134; ac2-weenie/src/chat.rs:47]
- [x] `StringInfo::AddVariable_UInt` · 006523c0 · legions 00601490 — audit: covered:  [ac2-db/src/string_info.rs:42-100 RenderVar variants (Int, FormattedInt, UInt, ..., Float, FormattedFloat, Info, String); ac2-ui/src/pregame/strings.rs:134; ac2-weenie/src/chat.rs:47]
- [x] `StringInfo::AddVariable_FormattedUInt` · 00652440 · legions 00601510 — audit: covered:  [ac2-db/src/string_info.rs:42-100 RenderVar variants (Int, FormattedInt, UInt, ..., Float, FormattedFloat, Info, String); ac2-ui/src/pregame/strings.rs:134; ac2-weenie/src/chat.rs:47]
- [x] `StringInfo::AddVariable_LInt` · 006524d0 · legions 006015a0 — audit: covered:  [ac2-db/src/string_info.rs:42-100 RenderVar variants (Int, FormattedInt, UInt, ..., Float, FormattedFloat, Info, String); ac2-ui/src/pregame/strings.rs:134; ac2-weenie/src/chat.rs:47]
- [x] `StringInfo::AddVariable_FormattedLInt` · 00652550 · legions 00601620 — audit: covered:  [ac2-db/src/string_info.rs:42-100 RenderVar variants (Int, FormattedInt, UInt, ..., Float, FormattedFloat, Info, String); ac2-ui/src/pregame/strings.rs:134; ac2-weenie/src/chat.rs:47]
- [x] `StringInfo::AddVariable_ULInt` · 006525e0 · legions 006016b0 — audit: covered:  [ac2-db/src/string_info.rs:42-100 RenderVar variants (Int, FormattedInt, UInt, ..., Float, FormattedFloat, Info, String); ac2-ui/src/pregame/strings.rs:134; ac2-weenie/src/chat.rs:47]
- [x] `StringInfo::AddVariable_FormattedULInt` · 00652660 · legions 00601730 — audit: covered:  [ac2-db/src/string_info.rs:42-100 RenderVar variants (Int, FormattedInt, UInt, ..., Float, FormattedFloat, Info, String); ac2-ui/src/pregame/strings.rs:134; ac2-weenie/src/chat.rs:47]
- [x] `StringInfo::AddVariable_Float` · 006526f0 · legions 006017c0 — audit: covered:  [ac2-db/src/string_info.rs:42-100 RenderVar variants (Int, FormattedInt, UInt, ..., Float, FormattedFloat, Info, String); ac2-ui/src/pregame/strings.rs:134; ac2-weenie/src/chat.rs:47]
- [x] `StringInfo::AddVariable_FormattedFloat` · 00652780 · legions 00601850 — audit: covered:  [ac2-db/src/string_info.rs:42-100 RenderVar variants (Int, FormattedInt, UInt, ..., Float, FormattedFloat, Info, String); ac2-ui/src/pregame/strings.rs:134; ac2-weenie/src/chat.rs:47]
- [-] `StringInfo::CopyVariables` · 00652820 · legions 006018f0 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq) (CopyVariables)
- [-] `StringInfo::StringInfo` · 00652930 · legions 00601a00 — skip: constructor/destructor of plain data; Rust struct init and Drop
- [-] `StringInfo::operator=` · 00652980 · legions 00601a50 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)

### SID_StringInfo
- [-] `SID_StringInfo::SID_StringInfo` · 006529f0 · legions 00601ac0 — skip: constructor/destructor of plain data; Rust struct init and Drop

### StringInfo
- [-] `StringInfo::StringInfo` · 00652a80 · legions 00601b50 — skip: constructor/destructor of plain data; Rust struct init and Drop
- [x] `StringInfo::AddVariable_StringID` · 00652ac0 · legions 00601b90 — crates/ac2-db/src/string_info.rs RenderVar::string_id cites StringInfo::AddVariable_StringID
- [x] `StringInfo::AddVariable_StringIDandTableEnum` · 00652b50 · legions 00601c20 — crates/ac2-db/src/string_info.rs RenderVar::string_id cites AddVariable_StringIDandTableEnum (table DID arg)
- [x] `StringInfo::AddVariable_StringInfo` · 00652bf0 · legions 00601cc0 — audit: covered:  [ac2-db/src/string_info.rs:42-100 RenderVar variants (Int, FormattedInt, UInt, ..., Float, FormattedFloat, Info, String); ac2-ui/src/pregame/strings.rs:134; ac2-weenie/src/chat.rs:47]
- [x] `StringInfo::AddVariable_String` · 00652c70 · legions 00601d40 — crates/ac2-db/src/string_info.rs RenderVar::String cites AddVariable_String(text), the wide overload
- [-] `StringInfo::AsyncGetString` · 00652d10 · legions 00601de0 — replaced by design:  [text is loaded on workers by the string-table loaders (ac2-db/src/string_table.rs:188); AGENTS.md DB object cache exception]
- [x] `StringInfo::UnPack` · 00652db0 · legions 00601e80 — ac2-script reference/native unpack
- [x] `StringInfo::AddVariable_String` · 00653190 · legions 00602260 — audit: covered:  [ac2-db/src/string_info.rs:42-100 RenderVar variants (Int, FormattedInt, UInt, ..., Float, FormattedFloat, Info, String); ac2-ui/src/pregame/strings.rs:134; ac2-weenie/src/chat.rs:47]

## `portal/shared/src/stringtable/stringinfodata.cpp.c` (0/18)

### SID_Int
- [-] `SID_Int::DuplicateYourself` · 00656180 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)
- [-] `SID_Int::operator==` · 006561c0 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)

### SID_FormattedInt
- [-] `SID_FormattedInt::DuplicateYourself` · 00656200 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)

### SID_UInt
- [-] `SID_UInt::DuplicateYourself` · 00656240 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)

### SID_FormattedUInt
- [-] `SID_FormattedUInt::DuplicateYourself` · 00656280 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)

### SID_FormattedInt
- [-] `SID_FormattedInt::operator==` · 006562c0 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)

### SID_Float
- [-] `SID_Float::DuplicateYourself` · 00656310 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)
- [-] `SID_Float::operator==` · 00656350 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)

### SID_FormattedFloat
- [-] `SID_FormattedFloat::DuplicateYourself` · 00656390 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)
- [-] `SID_FormattedFloat::operator==` · 006563e0 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)

### SID_StringInfo
- [-] `SID_StringInfo::DuplicateYourself` · 00656430 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)
- [-] `SID_StringInfo::operator==` · 00656490 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)

### SID_LInt
- [-] `SID_LInt::DuplicateYourself` · 006564c0 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)
- [-] `SID_LInt::operator==` · 00656510 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)

### SID_FormattedLInt
- [-] `SID_FormattedLInt::DuplicateYourself` · 00656550 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)

### SID_ULInt
- [-] `SID_ULInt::DuplicateYourself` · 006565a0 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)

### SID_FormattedULInt
- [-] `SID_FormattedULInt::DuplicateYourself` · 006565f0 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)

### SID_FormattedLInt
- [-] `SID_FormattedLInt::operator==` · 00656640 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)

## `portal/shared/src/stringtable/stringstate.cpp.c` (23/30)

### StringStateInfo
- [x] `StringStateInfo::GetNumDecDigits` · 00653200 · legions 006022d0 — ac2-db string_state (StringState decode, 0x58000000 via enum 0x40000001)
- [x] `StringStateInfo::GetLeadingZero` · 00653220 · legions 006022f0 — ac2-db string_state (StringState decode, 0x58000000 via enum 0x40000001)
- [x] `StringStateInfo::GetBase` · 00653240 · legions 00602310 — ac2-db string_state (StringState decode, 0x58000000 via enum 0x40000001)
- [x] `StringStateInfo::GetTopGoreLevel` · 00653260 · legions 00602330 — audit: covered:  [ac2-smartbox/src/fx_prefs.rs:10; ac2-client/src/client_prefs.rs:109; ac2-db/src/string_state.rs settings[]]
- [x] `StringStateInfo::GetTopBloodLevel` · 006532a0 · legions 00602370 — audit: covered:  [ac2-smartbox/src/fx_prefs.rs:10; ac2-client/src/client_prefs.rs:109; ac2-db/src/string_state.rs settings[]]
- [x] `StringStateInfo::GetGroupingSize` · 006532e0 · legions 006023b0 — ac2-db string_state (StringState decode, 0x58000000 via enum 0x40000001)
- [x] `StringStateInfo::IsNumberSingular` · 00653300 — ac2-db string_state (StringState decode, 0x58000000 via enum 0x40000001)
- [-] `StringStateInfo::InqIMEAppearance` · 00653370 · legions 00602440 — not needed:  [sole caller CIme::SetAppearanceToDefault is [-] (IME)]
- [x] `StringStateInfo::Startup` · 00653430 · legions 00602500 — ac2-db string_state (StringState decode, 0x58000000 via enum 0x40000001)
- [x] `StringStateInfo::SetTopGoreLevelOverride` · 00653480 · legions 00602550 — ac2-db string_state.rs TopLevelOverrides (set_max_gore / set_green_blood, top_levels = GetTopGoreLevel/GetTopBloodLevel); ac2 cmdline -maxgore / -greenblood -> ClientPrefs::load_with; tests the_command_line_overrides_the_top_gore_and_blood_levels, the_command_line_gore_and_blood_overrides_become_the_fx_defaults
- [x] `StringStateInfo::SetTopBloodColorOverride` · 00653490 · legions 00602560 — ac2-db string_state.rs TopLevelOverrides (set_max_gore / set_green_blood, top_levels = GetTopGoreLevel/GetTopBloodLevel); ac2 cmdline -maxgore / -greenblood -> ClientPrefs::load_with; tests the_command_line_overrides_the_top_gore_and_blood_levels, the_command_line_gore_and_blood_overrides_become_the_fx_defaults

### StringState
- [x] `StringState::GetPackSize` · 006534a0 · legions 00602570 — ac2-db string_state (StringState decode, 0x58000000 via enum 0x40000001)
- [-] `StringState::Pack` · 006534d0 · legions 006025a0 — not needed:  [pack direction (recursive self call); the client only reads StringState (ac2-db/src/string_state.rs:105)]
- [x] `StringState::UnPack` · 00653510 · legions 006025e0 — ac2-db string_state (StringState decode, 0x58000000 via enum 0x40000001)

### StringStateInfo
- [-] `StringStateInfo::Shutdown` · 00653530 · legions 00602600 — Rust ownership

### StringState
- [x] `StringState::StringState` · 00653570 · legions 00602640 — ac2-db string_state (StringState decode, 0x58000000 via enum 0x40000001)
- [x] `StringState::GetDBOType` · 00653650 — ac2-db string_state (StringState decode, 0x58000000 via enum 0x40000001)
- [-] `StringState::GetPackSize` · 00653670 — inline copy of 006534a0
- [-] `StringState::Pack` · 00653680 — not needed:  [pack direction (recursive self call); the client only reads StringState (ac2-db/src/string_state.rs:105)]
- [x] `StringState::UnPack` · 00653690 — ac2-db string_state (StringState decode, 0x58000000 via enum 0x40000001)
- [-] `StringState::~StringState` · 006536a0 · legions 00602770 — Rust ownership

### StringStateInfo
- [x] `StringStateInfo::GetNumerals` · 006537d0 · legions 006028a0 — ac2-db string_state (StringState decode, 0x58000000 via enum 0x40000001)
- [x] `StringStateInfo::GetGroupingSeperator` · 00653820 · legions 006028f0 — ac2-db string_state (StringState decode, 0x58000000 via enum 0x40000001)
- [x] `StringStateInfo::GetDecimalSeperator` · 00653870 · legions 00602940 — ac2-db string_state (StringState decode, 0x58000000 via enum 0x40000001)
- [x] `StringStateInfo::GetNegNumFormat` · 006538c0 · legions 00602990 — ac2-db string_state (StringState decode, 0x58000000 via enum 0x40000001)
- [x] `StringStateInfo::GetMaleMetaLetters` · 00653910 — ac2-db string_state (StringState decode, 0x58000000 via enum 0x40000001)
- [x] `StringStateInfo::GetFemaleMetaLetters` · 00653960 — ac2-db string_state (StringState decode, 0x58000000 via enum 0x40000001)
- [x] `StringStateInfo::GetTreasureMetaLetters` · 006539c0 · legions 00602a90 — ac2-db string_state (StringState decode, 0x58000000 via enum 0x40000001)

### StringState
- [-] `StringState::Allocate` · 00653a90 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)
- [x] `StringState::StreamPack` · 00653ac0 · legions 00602b90 — ac2-db string_state (StringState decode, 0x58000000 via enum 0x40000001)

## `portal/shared/src/stringtable/stringtable.cpp.c` (10/28)

### StringTableString
- [-] `StringTableString::Destroy` · 00653d60 · legions 00602e30 — skip: constructor/destructor of plain data; Rust struct init and Drop

### StringTable
- [x] `StringTable::FloatToString` · 00653e00 · legions 00602ed0 — ac2-db string_number (from the x87 assembly; Legions identical)
- [-] `StringTable::GetPackSize` · 00654260 — skip: DAT writing/saving/packing (tools only) (StringTable::GetPackSize)
- [-] `StringTable::Pack` · 006542f0 — skip: DAT writing/saving/packing (tools only) (StringTable::Pack)

### StringTableString
- [-] `StringTableString::StringTableString` · 00654400 · legions 006034d0 — skip: constructor/destructor of plain data; Rust struct init and Drop
- [-] `StringTableString::~StringTableString` · 00654470 · legions 00603540 — skip: constructor/destructor of plain data; Rust struct init and Drop

### StringTable
- [-] `StringTable::GetSubDataIDs` · 006544d0 — not needed:  [data-graph prefetch / StringDownload (tool) only; catalog policy rows (CharacterSetup::GetSubDataIDs etc.) are [-]; m_pDataGraph never set]
- [-] `StringTable::Destroy` · 00654540 · legions 00603610 — skip: constructor/destructor of plain data; Rust struct init and Drop

### StringTableString
- [-] `StringTableString::IsVarNameTableWorthPacking` · 00654640 · legions 00603710 — skip: DAT writing/saving/packing (tools only) (IsVarNameTableWorthPacking)

### StringTable
- [-] `StringTable::~StringTable` · 00654870 · legions 00603940 — skip: constructor/destructor of plain data; Rust struct init and Drop
- [-] `StringTable::GetDBOType` · 00654900 — skip: retail DBO type tag; Rust typed enum in ac2-db db_type.rs
- [x] `StringTable::IntToString` · 00654910 · legions 006039e0 — ac2-db string_number (from the x87 assembly; Legions identical)
- [x] `StringTable::UIntToString` · 00654940 · legions 00603a10 — ac2-db string_number (from the x87 assembly; Legions identical)
- [x] `StringTable::LIntToString` · 00654980 · legions 00603a50 — ac2-db string_number (from the x87 assembly; Legions identical)
- [x] `StringTable::ULIntToString` · 006549b0 · legions 00603a80 — ac2-db string_number (from the x87 assembly; Legions identical)

### StringTableString
- [-] `StringTableString::GetPackSize` · 00654a30 — skip: DAT writing/saving/packing (tools only) (StringTableString::GetPackSize)
- [-] `StringTableString::Pack` · 00654b00 — skip: DAT writing/saving/packing (tools only) (StringTableString::Pack)
- [x] `StringTableString::UnPack` · 00654cb0 — ac2-db string_table.rs: all 10,598 Legions tables decode

### StringTable
- [x] `StringTable::GetStringTableString` · 00654f10 · legions 00603fe0 — ac2-db string_table.rs: all 10,598 Legions tables decode

### StringTableString
- [-] `StringTableString::CopyInto` · 00654f50 · legions 00604020 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)

### StringTable
- [x] `StringTable::GetString` · 00655170 — ac2-db string_info (plain_entry / render_entry; Legions vtable +0x5C/+0x58)
- [-] `StringTable::GetRawString` · 00655260 — admin 7148 only, not in Legions 7160 [vtable-checked: not a slot of any located Legions vtable]
- [x] `StringTable::GetString` · 00655420 — ac2-db string_info (plain_entry / render_entry; Legions vtable +0x5C/+0x58)
- [-] `StringTable::SaveToFile` · 006557b0 — admin 7148 only, not in Legions 7160 [vtable-checked: Legions StringTable vtable has no such slot]
- [-] `StringTable::StringTable` · 00655d70 · legions 006046c0 — skip: constructor/destructor of plain data; Rust struct init and Drop
- [x] `StringTable::UnPack` · 00655de0 — ac2-db string_table.rs: all 10,598 Legions tables decode
- [-] `StringTable::CopyInto` · 00655f90 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)
- [-] `StringTable::Allocate` · 00656170 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

## `portal/shared/src/stringtable/stringvartable.cpp.c` (8/9)

### StringVarTable
- [x] `StringVarTable::StringVarTable` · 00656690 · legions 00604fe0 — ac2-db string_info (StringVarTable map)
- [x] `StringVarTable::SetVariable` · 00656700 · legions 00605050 — ac2-db string_info (StringVarTable map)
- [x] `StringVarTable::SetVariableFromInt` · 006567d0 · legions 00605120 — ac2-db string_info (StringVarTable map)
- [x] `StringVarTable::SetVariableFromUInt` · 00656840 · legions 00605190 — ac2-db string_info (StringVarTable map)
- [x] `StringVarTable::SetVariableFromLInt` · 006568b0 · legions 00605200 — ac2-db string_info (StringVarTable map)
- [x] `StringVarTable::SetVariableFromULInt` · 00656930 · legions 00605280 — ac2-db string_info (StringVarTable map)
- [x] `StringVarTable::SetVariableFromFloat` · 006569b0 · legions 00605300 — ac2-db string_info (StringVarTable map)
- [x] `StringVarTable::GetVariable` · 00656a30 · legions 00605380 — ac2-db string_info (StringVarTable map)
- [-] `StringVarTable::~StringVarTable` · 00656af0 · legions 00605440 — Rust ownership

## `portal/shared/src/taboo/tabootable.cpp.c` (1/13)

### TabooTable
- [-] `TabooTable::CreateCheckString` · 0065a5c0 · legions 00608f30 — only reached from the efuns gTabooTableAdaptor::CheckCharactersW / CheckCensorsW, which the 7230 wlib never calls (efun_coverage AC2_DUMP_REFERENCED on dataset-7230: of the adaptor only FormatStringForAudienceW, bound); no taboo table record is in the 7160/7230 DATs
- [-] `TabooTable::StringMatchesFilter` · 0065a650 · legions 00608fc0 — only reached from the efuns gTabooTableAdaptor::CheckCharactersW / CheckCensorsW, which the 7230 wlib never calls (efun_coverage AC2_DUMP_REFERENCED on dataset-7230: of the adaptor only FormatStringForAudienceW, bound); no taboo table record is in the 7160/7230 DATs
- [-] `TabooTable::Pack` · 0065a6f0 — skip: DAT writing/saving/packing (tools only) (TabooTable::Pack)
- [-] `TabooTable::UnPack` · 0065a750 · legions 006090c0 — only reached from the efuns gTabooTableAdaptor::CheckCharactersW / CheckCensorsW, which the 7230 wlib never calls (efun_coverage AC2_DUMP_REFERENCED on dataset-7230: of the adaptor only FormatStringForAudienceW, bound); no taboo table record is in the 7160/7230 DATs
- [-] `TabooTable::CheckCharacters` · 0065a790 · legions 00609100 — only reached from the efuns gTabooTableAdaptor::CheckCharactersW / CheckCensorsW, which the 7230 wlib never calls (efun_coverage AC2_DUMP_REFERENCED on dataset-7230: of the adaptor only FormatStringForAudienceW, bound); no taboo table record is in the 7160/7230 DATs
- [x] `TabooTable::FormatStringForAudience` · 0065a920 · legions 00609290 — ac2-script glue/taboo.rs: trim, TrimExcessSpaces, title-case; 0 without the TabooTable DBObj

### PList<PStringBase<unsigned_short>_>
- [-] `PList<PStringBase<unsigned_short>_>::operator=` · 0065aab0 · legions 00609420 — skip: copy/assign/compare operator; Rust derive(Clone, PartialEq)

### TabooTable
- [-] `TabooTable::CheckCensors` · 0065aaf0 · legions 00609460 — only reached from the efuns gTabooTableAdaptor::CheckCharactersW / CheckCensorsW, which the 7230 wlib never calls (efun_coverage AC2_DUMP_REFERENCED on dataset-7230: of the adaptor only FormatStringForAudienceW, bound); no taboo table record is in the 7160/7230 DATs

### PHashTable<unsigned_long,PList<PStringBase<unsigned_short>_>_>
- [-] `PHashTable<unsigned_long,PList<PStringBase<unsigned_short>_>_>::StreamPack` · 0065ad60 · legions 006096d0 — only reached from the efuns gTabooTableAdaptor::CheckCharactersW / CheckCensorsW, which the 7230 wlib never calls (efun_coverage AC2_DUMP_REFERENCED on dataset-7230: of the adaptor only FormatStringForAudienceW, bound); no taboo table record is in the 7160/7230 DATs
- [-] `PHashTable<unsigned_long,PList<PStringBase<unsigned_short>_>_>::~PHashTable<unsigned_long,PList<PStringBase<unsigned_short>_>_>` · 0065af50 · legions 006098c0 — skip: constructor/destructor of plain data; Rust struct init and Drop

### TabooTable
- [-] `TabooTable::GetDBOType` · 0065b000 — skip: retail DBO type tag; Rust typed enum in ac2-db db_type.rs
- [-] `TabooTable::TabooTable` · 0065b010 — skip: constructor/destructor of plain data; Rust struct init and Drop
- [-] `TabooTable::Allocate` · 0065b050 — skip: object pool allocator; Rust frees on last drop (AGENTS: no DBOCache pooling)

## `portal/shared/src/taboo/tabootableadaptor.cpp.c` (1/3)

### TabooTableAdaptor
- [-] `TabooTableAdaptor::CheckCharactersW` · 0065b090 · legions 00609a00 — only reached from the efuns gTabooTableAdaptor::CheckCharactersW / CheckCensorsW, which the 7230 wlib never calls (efun_coverage AC2_DUMP_REFERENCED on dataset-7230: of the adaptor only FormatStringForAudienceW, bound); no taboo table record is in the 7160/7230 DATs
- [-] `TabooTableAdaptor::CheckCensorsW` · 0065b100 · legions 00609a50 — only reached from the efuns gTabooTableAdaptor::CheckCharactersW / CheckCensorsW, which the 7230 wlib never calls (efun_coverage AC2_DUMP_REFERENCED on dataset-7230: of the adaptor only FormatStringForAudienceW, bound); no taboo table record is in the 7160/7230 DATs
- [x] `TabooTableAdaptor::FormatStringForAudienceW` · 0065b170 · legions 00609aa0 — ac2-script glue/taboo.rs: trim, TrimExcessSpaces, title-case; 0 without the TabooTable DBObj
