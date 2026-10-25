# ac2-db: Database objects, properties, enum maps, string tables, client cache

633 retail functions: 46 done, 13 partial, 535 to do, 39 skipped.
Legend and workflow: [README](README.md).

## `ac2/client/src/ccache/gmclcacheinit.cpp.c` (0/5)

### gmCLCache
- [ ] `gmCLCache::Init_Internal` · 00401940 · legions 00401870
- [ ] `gmCLCache::gmCLCache` · 00401950 · legions 00401880

### PortalThread
- [ ] `PortalThread::SetShouldExit` · 00401970 · legions 004018a0

### Turbine::Client
- [ ] `Turbine::Client::CreateChatInterfaceProvider` · 00401990 · legions 00401b60

### gmCLCache
- [ ] `gmCLCache::Init` · 004019e0 · legions 00401900

## `ac2/shared/src/database/gmmasterdbmap.cpp.c` (0/4)

### gmMasterDBMap
- [ ] `gmMasterDBMap::DivineType_Internal` · 006018c0

### DBTypeDef
- [ ] `DBTypeDef::DBTypeDef` · 006018d0 · legions 005c27b0

### gmMasterDBMap
- [ ] `gmMasterDBMap::InitDBTypeDef_Internal` · 00601940
- [ ] `gmMasterDBMap::DivineType_Internal` · 00601a50

## `ac2/shared/src/languagespecific/gmcharnames.cpp.c` (0/2)

### gmCharNameCharacterValidator
- [ ] `gmCharNameCharacterValidator::IsCharacterValid` · 0060cb90 · legions 005cd990
- [ ] `gmCharNameCharacterValidator::IsCharacterValidForTransitions` · 0060cc50 · legions 005cda50

## `portal/client/src/clcache/clcachemain.cpp.c` (1/132)

### EntityDesc
- [ ] `EntityDesc::Allocator` · 004be6e0 · legions 004b2a20

### EntityGroupDesc
- [ ] `EntityGroupDesc::Allocator` · 004be700 · legions 004b2a40

### PathMap
- [ ] `PathMap::Allocator` · 004be720 · legions 004b2a60

### CCellMesh
- [ ] `CCellMesh::Allocator` · 004be740 · legions 004b2a80

### SceneFile
- [ ] `SceneFile::Allocator` · 004be760

### AnimMap
- [ ] `AnimMap::Allocator` · 004be780 · legions 004b2ac0

### CSetup
- [ ] `CSetup::Allocator` · 004be7a0

### DBAnimator
- [ ] `DBAnimator::Allocator` · 004be7d0

### CDataMesh
- [ ] `CDataMesh::Allocator` · 004be7f0 · legions 004b2b30

### DBFile2IDTable
- [ ] `DBFile2IDTable::Allocator` · 004be820
- [ ] `DBFile2IDTable::GetDBOType` · 004be860
- [ ] `DBFile2IDTable::~DBFile2IDTable` · 004be890 · legions 004b2bd0

### TabooTable
- [ ] `TabooTable::Allocator` · 004be8b0

### EnumMapper
- [ ] `EnumMapper::Allocator` · 004be8d0

### CRegionDesc
- [ ] `CRegionDesc::Allocator` · 004be8f0

### CSoundDesc
- [ ] `CSoundDesc::Allocator` · 004be910

### CSceneDesc
- [ ] `CSceneDesc::Allocator` · 004be930

### CTerrainDesc
- [ ] `CTerrainDesc::Allocator` · 004be950

### CSurfaceDesc
- [ ] `CSurfaceDesc::Allocator` · 004be970

### CEncounterDesc
- [ ] `CEncounterDesc::Allocator` · 004be990

### CSkyDesc
- [ ] `CSkyDesc::Allocator` · 004be9b0

### CWaterDesc
- [ ] `CWaterDesc::Allocator` · 004be9d0

### CFogDesc
- [ ] `CFogDesc::Allocator` · 004be9f0 · legions 004b2d30

### PropertyDesc
- [ ] `PropertyDesc::Allocator` · 004bea10

### BlockMap
- [ ] `BlockMap::Allocator` · 004bea30

### CDayDesc
- [ ] `CDayDesc::Allocator` · 004bea50

### CKeyMap
- [ ] `CKeyMap::Allocator` · 004bea70

### DBFXTable
- [ ] `DBFXTable::Allocator` · 004bea90 · legions 004b2dd0

### AppearanceTable
- [ ] `AppearanceTable::Allocator` · 004beab0

### VisualDesc
- [ ] `VisualDesc::Allocator` · 004bead0 · legions 004b2e10

### LayoutDesc
- [ ] `LayoutDesc::Allocator` · 004beaf0 · legions 004b2e30

### UIScene
- [ ] `UIScene::Allocator` · 004beb10

### DBMusicObj
- [ ] `DBMusicObj::Allocator` · 004beb30 · legions 004b2e70

### StringTable
- [ ] `StringTable::Allocator` · 004beb50 · legions 004b2e90

### FileFormat
- [ ] `FileFormat::Allocator` · 004beb70 · legions 004b2eb0

### EnumIDMap
- [ ] `EnumIDMap::Allocator` · 004beb90 · legions 004b2ed0

### SoundInfo
- [ ] `SoundInfo::Allocator` · 004bebb0 · legions 004b2ef0

### MaterialModifier
- [ ] `MaterialModifier::Allocator` · 004bebd0

### RenderMaterial
- [ ] `RenderMaterial::Allocator` · 004bebf0

### MusicInfo
- [ ] `MusicInfo::Allocator` · 004bec10 · legions 004b2f50

### CharTemplate
- [ ] `CharTemplate::Allocator` · 004bec30 · legions 004b2f70

### MaterialInstance
- [ ] `MaterialInstance::Allocator` · 004bec50

### MotionInterpDesc
- [ ] `MotionInterpDesc::Allocator` · 004bec70

### MasterProperty
- [ ] `MasterProperty::Allocator` · 004bec90 · legions 004b2fd0

### LandscapeDefs
- [ ] `LandscapeDefs::Allocator` · 004becb0 · legions 004b3010

### PhysicsMaterial
- [ ] `PhysicsMaterial::Allocator` · 004becd0

### ObstacleDesc
- [ ] `ObstacleDesc::Allocator` · 004becf0

### PSDesc
- [ ] `PSDesc::Allocator` · 004bed10 · legions 004b3050

### RenderTextureLocal
- [ ] `RenderTextureLocal::GetDBOType` · 004bed30

### RenderSurfaceLocal
- [ ] `RenderSurfaceLocal::GetDBOType` · 004bed40

### Font
- [ ] `Font::~Font` · 004bed50 · legions 004b3090
- [ ] `Font::Allocator` · 004bed70
- [ ] `Font::GetDBOType` · 004beda0

### FontLocal
- [ ] `FontLocal::GetDBOType` · 004bede0

### QualityFilter
- [ ] `QualityFilter::Allocator` · 004bedf0 · legions 004b3130

### BehaviorTable
- [ ] `BehaviorTable::Allocator` · 004bee10

### ActionMap
- [ ] `ActionMap::Allocator` · 004bee30

### Performer
- [ ] `Performer::Allocator` · 004bee50 · legions 004b3190

### ValidModes
- [ ] `ValidModes::Allocator` · 004bee70 · legions 004b31b0

### CameraFX
- [ ] `CameraFX::Allocator` · 004bee90

### WLib
- [ ] `WLib::Allocator` · 004beeb0

### StringState
- [ ] `StringState::Allocator` · 004beed0 · legions 004b3210

### CDetailMapDesc
- [ ] `CDetailMapDesc::Allocator` · 004beef0

### CMapNoteDesc
- [ ] `CMapNoteDesc::Allocator` · 004bef10

### ConversationTree
- [ ] `ConversationTree::Allocator` · 004bef30 · legions 004b3270

### DBCategories
- [ ] `DBCategories::Allocator` · 004bef50

### WState
- [ ] `WState::Allocator` · 004bef70 · legions 004b32b0

### CBaseQualities
- [ ] `CBaseQualities::Allocator` · 004bef90 · legions 004b32f0

### DatFileData
- [ ] `DatFileData::Allocator` · 004befb0

### CLCache
- [ ] `CLCache::ResetCache` · 004befd0 · legions 004b3310
- [ ] `CLCache::WorkerExecuteRequest` · 004befe0 · legions 004b3320
- [ ] `CLCache::OnRequestFinished` · 004bf000 · legions 004b3340

### Cache_Pack_t
- [ ] `Cache_Pack_t::~Cache_Pack_t` · 004bf500 · legions 004b38d0

### RenderTextureLocal
- [ ] `RenderTextureLocal::Allocator` · 004bf570

### RenderSurfaceLocal
- [ ] `RenderSurfaceLocal::Allocator` · 004bf5c0

### FontLocal
- [ ] `FontLocal::Allocator` · 004bf610

### CThreadsafeDiskController
- [ ] `CThreadsafeDiskController::LoadData` · 004bf670 · legions 004b3a40
- [ ] `CThreadsafeDiskController::SaveData` · 004bf6b0 · legions 004b3a80
- [ ] `CThreadsafeDiskController::DeleteData` · 004bf6f0 · legions 004b3ac0
- [ ] `CThreadsafeDiskController::IsMember` · 004bf730 · legions 004b3b00
- [ ] `CThreadsafeDiskController::DeleteDataByMask` · 004bf770 · legions 004b3b40

### CLCache
- [ ] `CLCache::AsyncGetFromOtherSources` · 004bf7b0 · legions 004b3b80
- [ ] `CLCache::WorkerExecuteBeginDDDRequest` · 004bf800 · legions 004b3bd0

### PSmartArray<LOAD_UVPAIR>
- [ ] `PSmartArray<LOAD_UVPAIR>::~PSmartArray<LOAD_UVPAIR>` · 004bfa10 · legions 0063bba0

### DiskConInitInfo
- [ ] `DiskConInitInfo::DiskConInitInfo` · 004bfa40 · legions 004b3de0

### LookFile
- [ ] `LookFile::~LookFile` · 004bfab0 · legions 004b3e50

### FxScript
- [ ] `FxScript::GetDBOType` · 004bfb10

### DiskConInitInfo
- [ ] `DiskConInitInfo::~DiskConInitInfo` · 004bfb20 · legions 004b3ec0

### CThreadsafeDiskController
- [ ] `CThreadsafeDiskController::CThreadsafeDiskController` · 004bfb60 · legions 004b3f00

### DiskConBase
- [x] `DiskConBase::GetDatFileID` · 004bfb90 · legions 004b3f30

### DiskController
- [ ] `DiskController::HowmuchFreeSpace` · 004bfbb0 · legions 004b3f50
- [ ] `DiskController::GetMasterMapDID` · 004bfbc0 · legions 004b3f60
- [ ] `DiskController::GetDatIDStamp` · 004bfbd0 · legions 004b3f70
- [ ] `DiskController::GetFilename` · 004bfbf0 · legions 004b3f90

### CLCache
- [ ] `CLCache::NotifyDDDEvent` · 004bfc40 · legions 004b3fe0

### (free functions)
- [ ] `operator+` · 004bfc80 · legions 004b40c0

### FxScript
- [ ] `FxScript::Allocator` · 004bfcd0
- [ ] `FxScript::Allocate` · 004bfd30

### CThreadsafeDiskController
- [ ] `CThreadsafeDiskController::InitFile` · 004bfd70 · legions 004b41b0

### ThreadedCache
- [ ] `ThreadedCache::~ThreadedCache` · 004bfe10 · legions 004b4250

### CLCache
- [~] `CLCache::OnServerInterrogation` · 004bfeb0 — ac2-weenie dat_iterations + session answer; internal.dat (slot 4) not opened; language id is configuration (EnumMapper 0x4000002A lookup not ported)
- [ ] `CLCache::OnBeginDDD` · 004c0060 · legions 004b44a0
- [ ] `CLCache::SetLanguageInternal` · 004c00f0 · legions 004b4530
- [ ] `CLCache::OnEndDDD` · 004c0610 · legions 004b4aa0
- [ ] `CLCache::SetRegion` · 004c0690 · legions 004b4b20

### PSmartArray<Trigger>
- [ ] `PSmartArray<Trigger>::StreamPack` · 004c0a40 · legions 004b4ed0

### Trigger
- [ ] `Trigger::Trigger` · 004c0bc0 · legions 004b5050

### PSmartArray<ImpulseTrigger>
- [ ] `PSmartArray<ImpulseTrigger>::StreamPack` · 004c0bf0 · legions 004b5080

### ImpulseTrigger
- [ ] `ImpulseTrigger::ImpulseTrigger` · 004c0d70

### CLCache
- [ ] `CLCache::~CLCache` · 004c0da0 · legions 004b5230
- [ ] `CLCache::UseTime` · 004c0f10
- [ ] `CLCache::RemovePendingDownload` · 004c10d0 · legions 004b5560
- [ ] `CLCache::GetDiskController` · 004c1280 · legions 004b5710
- [ ] `CLCache::IsDatFileLoaded` · 004c13a0 · legions 004b5800

### TerrainList
- [ ] `TerrainList::~TerrainList` · 004c1450 · legions 004b58b0

### PSmartArray<CAllIterationList::PTaggedIterationList>
- [ ] `PSmartArray<CAllIterationList::PTaggedIterationList>::StreamPack` · 004c14b0 · legions 004b5910

### CAllIterationList::PTaggedIterationList
- [ ] `CAllIterationList::PTaggedIterationList::PTaggedIterationList` · 004c1630 · legions 004b5a90
- [ ] `CAllIterationList::PTaggedIterationList::operator=` · 004c1650 · legions 004b5ab0

### SmartArray<unsigned_long>
- [ ] `SmartArray<unsigned_long>::operator=` · 004c1680 · legions 0053f8d0

### PSmartArray<CMissingIteration>
- [ ] `PSmartArray<CMissingIteration>::StreamPack` · 004c16e0 · legions 004b5ae0

### CMissingIteration
- [ ] `CMissingIteration::CMissingIteration` · 004c1860
- [ ] `CMissingIteration::operator=` · 004c1880 · legions 004b5c80

### CLCache
- [ ] `CLCache::Init` · 004c18e0 · legions 004b5ce0
- [ ] `CLCache::OnSaveRequestFinished` · 004c1fa0 · legions 004b6330

### PHashTable<unsigned_long,MotionValues>
- [ ] `PHashTable<unsigned_long,MotionValues>::StreamPack` · 004c2130

### IntrusiveHashIterator<unsigned_long,HashTableData<unsigned_long,unsigned_long>*>
- [ ] `IntrusiveHashIterator<unsigned_long,HashTableData<unsigned_long,unsigned_long>*>::operator=` · 004c2310 · legions 00504950

### CLCache
- [ ] `CLCache::Init_Internal` · 004c2490 · legions 004b6860
- [ ] `CLCache::CLCache` · 004c41d0 · legions 004b85a0
- [ ] `CLCache::OnBeginDDDRequestFinished` · 004c42a0 · legions 004b8670

### PHashTable<unsigned_long,TerrainList>
- [ ] `PHashTable<unsigned_long,TerrainList>::StreamPack` · 004c45d0 · legions 004b89a0

### PHashTable<unsigned_long,MotionValues>
- [ ] `PHashTable<unsigned_long,MotionValues>::PHashTable<unsigned_long,MotionValues>` · 004c47f0 · legions 004b8bc0

### MSList
- [ ] `MSList::~MSList` · 004c48f0 · legions 004b8cc0

## `portal/client/src/clcache/clocache.cpp.c` (0/1)

### CLOCache
- [ ] `CLOCache::CLOCache` · 004c4a10 · legions 004b8de0

## `portal/shared/src/enummap/enummapper.cpp.c` (19/47)

### EnumMapper
- [ ] `EnumMapper::Get` · 00660750 · legions 0060f060
- [ ] `EnumMapper::GetSubDataIDs` · 00660770
- [ ] `EnumMapper::GetSubObjects` · 00660790
- [ ] `EnumMapper::ReleaseSubObjects` · 006607f0 · legions 0060f0c0

### EnumMapper::iterator
- [ ] `EnumMapper::iterator::operator!=` · 00660810 · legions 0060f0e0

### _STL::_Rb_tree_iterator<_STL::pair<unsigned_long_const_,PStringBase<char>_>,_STL::_Const_traits<_STL::pair<unsigned_long_const_,PStringBase<char>_>_>_>
- [ ] `_STL::_Rb_tree_iterator<_STL::pair<unsigned_long_const_,PStringBase<char>_>,_STL::_Const_traits<_STL::pair<unsigned_long_const_,PStringBase<char>_>_>_>::operator++` · 00660860 · legions 0060f130

### CaseInsensitivePStringCompare
- [ ] `CaseInsensitivePStringCompare::operator()` · 006608b0 · legions 0060f1d0

### EnumMapper
- [x] `EnumMapper::GetNumMappings` · 006608e0 · legions 0060f200 — ac2-script data/glue::enums

### EnumMapper::iterator
- [ ] `EnumMapper::iterator::get_enum` · 00660900 · legions 0060f220

### EnumMapper
- [x] `EnumMapper::GetNumMappings` · 00660a30 · legions 0060f350 — ac2-script data/glue::enums
- [ ] `EnumMapper::end` · 00660ae0 · legions 0060f400

### EnumMapper::iterator
- [ ] `EnumMapper::iterator::iterator` · 00660b10 · legions 0060f430

### EnumMapper
- [x] `EnumMapper::GetString` · 00660b90 · legions 0060f4b0 — ac2-script data/glue::enums
- [x] `EnumMapper::GetString` · 00660bd0 · legions 0060f4f0 — ac2-script data/glue::enums
- [x] `EnumMapper::GetString` · 00660c50 · legions 0060f570 — ac2-script data/glue::enums
- [x] `EnumMapper::InqString` · 00660d10 · legions 0060f630 — ac2-script data/glue::enums
- [x] `EnumMapper::GetStringW` · 00660dd0 · legions 0060f6f0 — ac2-script data/glue::enums
- [ ] `EnumMapper::GetStringByDID` · 00660e40 · legions 0060f760
- [x] `EnumMapper::GetLogStringW` · 00660f10 · legions 0060f800 — ac2-script data/glue::enums
- [ ] `EnumMapper::begin` · 00660f70 · legions 0060f860

### EnumMapper::iterator
- [ ] `EnumMapper::iterator::operator++` · 00660fa0 · legions 0060f890
- [ ] `EnumMapper::iterator::operator++` · 00661000 · legions 0060f8f0

### EnumMapper
- [x] `EnumMapper::GetEnum` · 00661020 · legions 0060f910 — ac2-script data/glue::enums
- [x] `EnumMapper::GetEnum` · 00661060 · legions 0060f950 — ac2-script data/glue::enums
- [x] `EnumMapper::GetEnum` · 006610b0 · legions 0060f9a0 — ac2-script data/glue::enums
- [x] `EnumMapper::InqEnum` · 00661170 · legions 0060fa60 — ac2-script data/glue::enums
- [x] `EnumMapper::GetEnumByDID` · 006611c0 · legions 0060fab0 — ac2-script data/glue::enums
- [x] `EnumMapper::GetEnumByDIDW` · 00661290 · legions 0060fb50 — ac2-script data/glue::enums

### _STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,PStringBase<char>_>,_STL::_Select1st<_STL::pair<unsigned_long_const_,PStringBase<char>_>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,PStringBase<char>_>_>_>
- [ ] `_STL::_Rb_tree<unsigned_long,_STL::pair<unsigned_long_const_,PStringBase<char>_>,_STL::_Select1st<_STL::pair<unsigned_long_const_,PStringBase<char>_>_>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,PStringBase<char>_>_>_>::operator=` · 00661950

### _STL::_Rb_tree<PStringBase<char>,_STL::pair<PStringBase<char>_const_,unsigned_long>,_STL::_Select1st<_STL::pair<PStringBase<char>_const_,unsigned_long>_>,CaseInsensitivePStringCompare,_STL::allocator<_STL::pair<PStringBase<char>_const_,unsigned_long>_>_>
- [ ] `_STL::_Rb_tree<PStringBase<char>,_STL::pair<PStringBase<char>_const_,unsigned_long>,_STL::_Select1st<_STL::pair<PStringBase<char>_const_,unsigned_long>_>,CaseInsensitivePStringCompare,_STL::allocator<_STL::pair<PStringBase<char>_const_,unsigned_long>_>_>::operator=` · 00661bf0

### EnumMapper
- [x] `EnumMapper::GetAllEnumVals` · 00661f30 · legions 00610450 — ac2-script data/glue::enums
- [x] `EnumMapper::GetAllEnumVals` · 00662040 · legions 00610560 — ac2-script data/glue::enums

### (free functions)
- [ ] `STREAMPACK_STL` · 00662120 · legions 00610640

### EnumMapper
- [ ] `EnumMapper::Destroy` · 00662330 · legions 00610850
- [-] `EnumMapper::LoadFromFile` · 006623a0 — text source loader, admin build only
- [x] `EnumMapper::GetAllEnumNames` · 00662490 · legions 006108c0 — ac2-script data/glue::enums
- [x] `EnumMapper::GetAllEnumNames` · 00662590 · legions 006109c0 — ac2-script data/glue::enums

### _STL::map<unsigned_long,PStringBase<char>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,PStringBase<char>_>_>_>
- [ ] `_STL::map<unsigned_long,PStringBase<char>,_STL::less<unsigned_long>,_STL::allocator<_STL::pair<unsigned_long_const_,PStringBase<char>_>_>_>::operator[]` · 00662670 · legions 00610aa0

### _STL::map<PStringBase<char>,unsigned_long,CaseInsensitivePStringCompare,_STL::allocator<_STL::pair<PStringBase<char>_const_,unsigned_long>_>_>
- [ ] `_STL::map<PStringBase<char>,unsigned_long,CaseInsensitivePStringCompare,_STL::allocator<_STL::pair<PStringBase<char>_const_,unsigned_long>_>_>::operator[]` · 00662740 · legions 00610b70

### EnumMapper
- [ ] `EnumMapper::EnumMapper` · 006627d0 · legions 00610c00
- [ ] `EnumMapper::GetDBOType` · 00662850
- [ ] `EnumMapper::EnumMapper` · 00662860 · legions 00610c90
- [ ] `EnumMapper::~EnumMapper` · 006628e0 · legions 00610d10
- [x] `EnumMapper::StreamPack` · 00662970 · legions 00610da0 — ac2-script data/glue::enums
- [ ] `EnumMapper::AddMapping` · 00662a30 · legions 00610e60
- [ ] `EnumMapper::Allocate` · 00662af0
- [ ] `EnumMapper::AddEnumMapper` · 00662b20

## `portal/shared/src/enummap/penummapper.cpp.c` (0/16)

### PEnumMapper
- [ ] `PEnumMapper::SetStartingValue` · 007381d0 · legions 006b3d90

### _STL::less<PEnum>
- [ ] `_STL::less<PEnum>::operator()` · 007381e0 · legions 006b3da0

### PEnum
- [ ] `PEnum::Destroy` · 00738230 · legions 006b3df0
- [ ] `PEnum::operator=` · 00738300 · legions 006b3ec0
- [ ] `PEnum::Initialize` · 00738470 · legions 006b4030

### PEnumMapper
- [ ] `PEnumMapper::FindEnum` · 007385a0 · legions 006b4160

### PEnum
- [ ] `PEnum::PEnum` · 00738730 · legions 006b42f0

### PEnumMapper
- [ ] `PEnumMapper::SetInheritanceDepth` · 007387a0 · legions 006b4360
- [ ] `PEnumMapper::HaveMapping` · 00738800 · legions 006b43c0
- [ ] `PEnumMapper::NextValidID` · 007389b0 · legions 006b4570
- [ ] `PEnumMapper::~PEnumMapper` · 00738ee0 · legions 006b4aa0
- [ ] `PEnumMapper::Initialize` · 00738f60 · legions 006b4b20
- [ ] `PEnumMapper::PEnumMapper` · 00739020 · legions 006b4be0
- [ ] `PEnumMapper::AddEnum` · 00739080 · legions 006b4c40
- [ ] `PEnumMapper::AddEnum` · 00739320
- [ ] `PEnumMapper::SortEnums` · 00739400 · legions 006b4f40

## `portal/shared/src/languagespecific/charnames.cpp.c` (0/2)

### CharNameCharacterValidator
- [ ] `CharNameCharacterValidator::CharNameCharacterValidator` · 006bc580 · legions 00651520
- [ ] `CharNameCharacterValidator::~CharNameCharacterValidator` · 006bc5a0 · legions 00651540

## `portal/shared/src/languagespecific/charutils.cpp.c` (0/1)

### CharUtils
- [ ] `CharUtils::IsVowel` · 006bc600 · legions 006515a0

## `portal/shared/src/property/baseproperty.cpp.c` (0/22)

### BaseProperty
- [ ] `BaseProperty::IsReportedToWeenie` · 006b1f50 · legions 0064b1e0
- [ ] `BaseProperty::IsInquirable` · 006b1f60 · legions 0064b1f0
- [ ] `BaseProperty::IsDefaultValue` · 006b1f70 · legions 0064b200
- [ ] `BaseProperty::InitToDefaultValue` · 006b1fa0 · legions 0064b230

### DiskConBase
- [ ] `DiskConBase::IsExpandable` · 006b2000 · legions 0064b290

### BaseProperty
- [ ] `BaseProperty::SetPropertyType` · 006b2010 · legions 0064b2a0
- [ ] `BaseProperty::GetSubDataIDs` · 006b2060 · legions 0064b2f0
- [ ] `BaseProperty::BaseProperty` · 006b20f0 · legions 0064b380
- [ ] `BaseProperty::~BaseProperty` · 006b2110 · legions 0064b3a0
- [ ] `BaseProperty::operator=` · 006b2130 · legions 0064b3c0
- [~] `BaseProperty::SetPropertyName` · 006b2180 · legions 0064b410 — value type via PropertyTypes (MasterProperty::InqPropertyType); group lookup and defaults not modelled
- [ ] `BaseProperty::InqPropertyBit` · 006b21e0 · legions 0064b470
- [ ] `BaseProperty::Compare` · 006b2210 · legions 0064b4a0
- [~] `BaseProperty::StreamPack` · 006b2250 — unpack only, in ac2-net protocol::property / fx (types from a PropertyTypes resolver; Legions BaseProperty::StreamPack 0x0064B4E0 instruction-identical)
- [ ] `BaseProperty::BaseProperty` · 006b22c0 · legions 0064b550
- [ ] `BaseProperty::SetPropertyNameW` · 006b2310 · legions 0064b5a0
- [ ] `BaseProperty::InqPropertyNameW` · 006b2380 · legions 0064b610
- [ ] `BaseProperty::InqPropertyTypeW` · 006b2440 · legions 0064b6d0
- [ ] `BaseProperty::GetOutputString` · 006b2510 · legions 0064b7a0
- [ ] `BaseProperty::ValueFromFileNode` · 006b2650
- [-] `BaseProperty::ToFileNode` · 006b26b0 — text source format (admin tools)
- [-] `BaseProperty::FromFileNode` · 006b2790 — text source format (admin tools)

## `portal/shared/src/property/desc/basepropertydesc.cpp.c` (0/10)

### BasePropertyDesc
- [ ] `BasePropertyDesc::BasePropertyDesc` · 006b6e70 · legions 0064ed50
- [ ] `BasePropertyDesc::~BasePropertyDesc` · 006b6ea0 · legions 0064ed80
- [ ] `BasePropertyDesc::operator=` · 006b6ee0 · legions 0064edc0
- [ ] `BasePropertyDesc::InqPropertyEnum` · 006b6f40 · legions 0064ee20
- [ ] `BasePropertyDesc::InqPropertyEnum` · 006b6f70
- [ ] `BasePropertyDesc::InitProperty` · 006b6fa0 · legions 0064ee50
- [ ] `BasePropertyDesc::GetSubDataIDs` · 006b6fc0 · legions 0064ee70
- [ ] `BasePropertyDesc::BasePropertyDesc` · 006b7030 · legions 0064eee0
- [ ] `BasePropertyDesc::StreamPack` · 006b7070 · legions 0064ef20
- [-] `BasePropertyDesc::FromFileNode` · 006b7100 — text source format (admin tools)

## `portal/shared/src/property/masterproperty.cpp.c` (0/130)

### BoolPropertyValue
- [-] `BoolPropertyValue::ToFileNode` · 006b3d50 — text source format (admin tools)
- [ ] `BoolPropertyValue::InqBool` · 006b3d80
- [ ] `BoolPropertyValue::Set` · 006b3da0
- [ ] `BoolPropertyValue::Compare` · 006b3dc0 · legions 0064cc80
- [ ] `BoolPropertyValue::Copy` · 006b3e00

### ColorPropertyValue
- [ ] `ColorPropertyValue::ColorPropertyValue` · 006b3e30
- [~] `ColorPropertyValue::StreamPack` · 006b3e60 — unpack only, in ac2-net protocol::property / fx (types from a PropertyTypes resolver; Legions BaseProperty::StreamPack 0x0064B4E0 instruction-identical)
- [-] `ColorPropertyValue::FromFileNode` · 006b3e70 — text source format (admin tools)
- [-] `ColorPropertyValue::ToFileNode` · 006b3e90 — text source format (admin tools)
- [ ] `ColorPropertyValue::InqColor` · 006b3eb0
- [ ] `ColorPropertyValue::SetColor` · 006b3ee0
- [ ] `ColorPropertyValue::Set` · 006b3f10
- [ ] `ColorPropertyValue::Compare` · 006b3f30
- [ ] `ColorPropertyValue::Copy` · 006b3f70

### DataFilePropertyValue
- [ ] `DataFilePropertyValue::Set` · 006b3fd0
- [ ] `DataFilePropertyValue::Compare` · 006b3ff0 · legions 0064c8e0
- [ ] `DataFilePropertyValue::Copy` · 006b4030

### EnumPropertyValue
- [ ] `EnumPropertyValue::Set` · 006b4060
- [ ] `EnumPropertyValue::Compare` · 006b4080 · legions 0064cad0
- [ ] `EnumPropertyValue::Copy` · 006b40c0

### FloatPropertyValue
- [ ] `FloatPropertyValue::Set` · 006b40f0
- [ ] `FloatPropertyValue::Compare` · 006b4110 · legions 0064cbf0
- [ ] `FloatPropertyValue::Copy` · 006b4150
- [-] `FloatPropertyValue::ToFileNode` · 006b4180 — text source format (admin tools)

### IntegerPropertyValue
- [ ] `IntegerPropertyValue::Set` · 006b41c0
- [ ] `IntegerPropertyValue::Compare` · 006b41e0
- [ ] `IntegerPropertyValue::Copy` · 006b4220
- [-] `IntegerPropertyValue::ToFileNode` · 006b4250 — text source format (admin tools)

### LongIntegerPropertyValue
- [ ] `LongIntegerPropertyValue::InqLongInteger` · 006b4280
- [ ] `LongIntegerPropertyValue::SetLongInteger` · 006b42a0
- [ ] `LongIntegerPropertyValue::Set` · 006b42c0
- [ ] `LongIntegerPropertyValue::Compare` · 006b42e0
- [ ] `LongIntegerPropertyValue::Copy` · 006b4330
- [-] `LongIntegerPropertyValue::ToFileNode` · 006b4370 — text source format (admin tools)

### PFileNode
- [ ] `PFileNode::SetNameAsInt64` · 006b43b0

### PackageIDPropertyValue
- [ ] `PackageIDPropertyValue::HasValidData` · 006b4400

### BehaviorTracker
- [ ] `BehaviorTracker::SetAnimationObj` · 006b4410 · legions 0064ce10

### PackageIDPropertyValue
- [ ] `PackageIDPropertyValue::Set` · 006b4420
- [ ] `PackageIDPropertyValue::Compare` · 006b4440 · legions 0064ce40
- [ ] `PackageIDPropertyValue::Copy` · 006b4480

### VectorPropertyValue
- [ ] `VectorPropertyValue::VectorPropertyValue` · 006b44b0
- [-] `VectorPropertyValue::FromFileNode` · 006b44e0 — text source format (admin tools)
- [-] `VectorPropertyValue::ToFileNode` · 006b4500 — text source format (admin tools)
- [ ] `VectorPropertyValue::InqVector` · 006b4520
- [ ] `VectorPropertyValue::SetVector` · 006b4540
- [ ] `VectorPropertyValue::Set` · 006b4560
- [ ] `VectorPropertyValue::Compare` · 006b4580
- [ ] `VectorPropertyValue::Copy` · 006b4600
- [~] `VectorPropertyValue::StreamPack` · 006b4650 — unpack only, in ac2-net protocol::property / fx (types from a PropertyTypes resolver; Legions BaseProperty::StreamPack 0x0064B4E0 instruction-identical)

### WaveformPropertyValue
- [ ] `WaveformPropertyValue::WaveformPropertyValue` · 006b46b0 · legions 0064d070

### DataFilePropertyValue
- [ ] `DataFilePropertyValue::HasValidData` · 006b46d0

### WaveformPropertyValue
- [-] `WaveformPropertyValue::FromFileNode` · 006b46e0 — text source format (admin tools)
- [-] `WaveformPropertyValue::ToFileNode` · 006b4710 — text source format (admin tools)
- [ ] `WaveformPropertyValue::InqWaveform` · 006b4730
- [ ] `WaveformPropertyValue::SetWaveform` · 006b4750
- [ ] `WaveformPropertyValue::Set` · 006b47a0
- [ ] `WaveformPropertyValue::GetValueAsString` · 006b47c0
- [ ] `WaveformPropertyValue::Compare` · 006b47e0

### Waveform
- [ ] `Waveform::operator==` · 006b4830 · legions 0064d1a0

### WaveformPropertyValue
- [ ] `WaveformPropertyValue::Copy` · 006b4950
- [~] `WaveformPropertyValue::StreamPack` · 006b49d0 — unpack only, in ac2-net protocol::property / fx (types from a PropertyTypes resolver; Legions BaseProperty::StreamPack 0x0064B4E0 instruction-identical)

### PositionPropertyValue
- [ ] `PositionPropertyValue::PositionPropertyValue` · 006b4a20
- [ ] `PositionPropertyValue::HasValidData` · 006b4a60
- [-] `PositionPropertyValue::FromFileNode` · 006b4a70 — text source format (admin tools)
- [-] `PositionPropertyValue::ToFileNode` · 006b4a90 — text source format (admin tools)
- [ ] `PositionPropertyValue::Set` · 006b4ab0
- [ ] `PositionPropertyValue::Compare` · 006b4ad0
- [~] `PositionPropertyValue::StreamPack` · 006b4b80 — unpack only, in ac2-net protocol::property / fx (types from a PropertyTypes resolver; Legions BaseProperty::StreamPack 0x0064B4E0 instruction-identical)

### MasterProperty
- [ ] `MasterProperty::InqPropertyNameEnum` · 006b4be0 · legions 0064d510
- [ ] `MasterProperty::InqPropertyNameString` · 006b4c50 · legions 0064d560
- [ ] `MasterProperty::StreamPack` · 006b4cc0 · legions 0064d5b0

### BoolPropertyValue
- [-] `BoolPropertyValue::FromFileNode` · 006b4d80 — text source format (admin tools)

### FloatPropertyValue
- [-] `FloatPropertyValue::FromFileNode` · 006b4db0 — text source format (admin tools)

### IntegerPropertyValue
- [-] `IntegerPropertyValue::FromFileNode` · 006b4de0 — text source format (admin tools)

### LongIntegerPropertyValue
- [-] `LongIntegerPropertyValue::FromFileNode` · 006b4e10 — text source format (admin tools)

### StringPropertyValue
- [ ] `StringPropertyValue::StringPropertyValue` · 006b4e40 · legions 0064d670
- [ ] `StringPropertyValue::Set` · 006b4e70
- [ ] `StringPropertyValue::HasValidData` · 006b4e90
- [ ] `StringPropertyValue::Copy` · 006b4eb0
- [-] `StringPropertyValue::FromFileNode` · 006b4f00 — text source format (admin tools)
- [ ] `StringPropertyValue::InqString` · 006b4f30
- [ ] `StringPropertyValue::SetString` · 006b4f50
- [ ] `StringPropertyValue::GetValueAsString` · 006b4f70
- [ ] `StringPropertyValue::Compare` · 006b4f90

### StringInfoPropertyValue
- [ ] `StringInfoPropertyValue::StringInfoPropertyValue` · 006b5050 · legions 0064d850
- [-] `StringInfoPropertyValue::FromFileNode` · 006b5080 — text source format (admin tools)
- [-] `StringInfoPropertyValue::ToFileNode` · 006b50b0 — text source format (admin tools)
- [ ] `StringInfoPropertyValue::InqStringInfo` · 006b50e0
- [ ] `StringInfoPropertyValue::SetStringInfo` · 006b5100
- [ ] `StringInfoPropertyValue::Set` · 006b5120
- [ ] `StringInfoPropertyValue::HasValidData` · 006b5140
- [~] `StringInfoPropertyValue::StreamPack` · 006b5160 — unpack only, in ac2-net protocol::property / fx (types from a PropertyTypes resolver; Legions BaseProperty::StreamPack 0x0064B4E0 instruction-identical)

### (free functions)
- [ ] `STREAMPACK_PACKOBJ` · 006b5190 · legions 00578ab0

### StringInfoPropertyValue
- [ ] `StringInfoPropertyValue::Copy` · 006b5200
- [ ] `StringInfoPropertyValue::InqStringInfo` · 006b5260 · legions 0064d990
- [ ] `StringInfoPropertyValue::SetStringInfo` · 006b5290
- [ ] `StringInfoPropertyValue::Compare` · 006b52c0

### PositionPropertyValue
- [ ] `PositionPropertyValue::Copy` · 006b5410
- [ ] `PositionPropertyValue::InqPosition` · 006b5480
- [ ] `PositionPropertyValue::SetPosition` · 006b54a0

### MasterProperty
- [~] `MasterProperty::AllocatePropertyValue` · 006b54c0 · legions 0064dbf0 — unpack only, in ac2-net protocol::property / fx (types from a PropertyTypes resolver; Legions BaseProperty::StreamPack 0x0064B4E0 instruction-identical)

### BoolPropertyValue
- [ ] `BoolPropertyValue::GetValueAsString` · 006b5680

### ColorPropertyValue
- [ ] `ColorPropertyValue::GetValueAsString` · 006b56b0

### FloatPropertyValue
- [ ] `FloatPropertyValue::GetValueAsString` · 006b56f0

### IntegerPropertyValue
- [ ] `IntegerPropertyValue::GetValueAsString` · 006b5720

### LongIntegerPropertyValue
- [ ] `LongIntegerPropertyValue::GetValueAsString` · 006b5740

### StringPropertyValue
- [-] `StringPropertyValue::ToFileNode` · 006b5770 — text source format (admin tools)

### StringInfoPropertyValue
- [ ] `StringInfoPropertyValue::GetValueAsString` · 006b57b0

### VectorPropertyValue
- [ ] `VectorPropertyValue::GetValueAsString` · 006b5880

### StringPropertyValue
- [~] `StringPropertyValue::StreamPack` · 006b5950 — unpack only, in ac2-net protocol::property / fx (types from a PropertyTypes resolver; Legions BaseProperty::StreamPack 0x0064B4E0 instruction-identical)

### (free functions)
- [ ] `STREAMPACK_PACKOBJ` · 006b5980

### MasterProperty
- [ ] `MasterProperty::InqOptionalPropertiesInternal` · 006b5a30 · legions 0064e0f0
- [ ] `MasterProperty::GetSubDataIDs` · 006b5af0

### PositionPropertyValue
- [ ] `PositionPropertyValue::GetValueAsString` · 006b5b60

### MasterProperty
- [ ] `MasterProperty::GetPropertyDesc` · 006b5bd0 · legions 0064e2d0
- [ ] `MasterProperty::InqPropertyBit` · 006b5c40
- [ ] `MasterProperty::InqOptionalProperties` · 006b5c70 · legions 0064e340
- [ ] `MasterProperty::Destroy` · 006b5ce0 · legions 0064e500

### HashTable<unsigned_long,BasePropertyDesc>
- [ ] `HashTable<unsigned_long,BasePropertyDesc>::operator=` · 006b5d00 · legions 0064e520

### MasterProperty
- [ ] `MasterProperty::InqPropertyType` · 006b5f40
- [ ] `MasterProperty::InqPropertyGroup` · 006b5fd0
- [ ] `MasterProperty::IsReportedToWeenie` · 006b6060
- [ ] `MasterProperty::IsInquirable` · 006b60f0
- [ ] `MasterProperty::~MasterProperty` · 006b61f0 · legions 0064e860
- [ ] `MasterProperty::GetDBOType` · 006b6270

### PAutoGrowHashTable<unsigned_long,BasePropertyDesc>
- [ ] `PAutoGrowHashTable<unsigned_long,BasePropertyDesc>::StreamPack` · 006b6280

### MasterProperty
- [ ] `MasterProperty::CopyInto` · 006b6480
- [-] `MasterProperty::LoadFromFile` · 006b64a0 — text source loader, admin build only
- [ ] `MasterProperty::MasterProperty` · 006b6670 · legions 0064eb10
- [ ] `MasterProperty::Allocate` · 006b66d0

## `portal/shared/src/property/propertycollection.cpp.c` (2/26)

### PropertyCollection
- [ ] `PropertyCollection::Destroy` · 006b31b0 · legions 0064c070
- [ ] `PropertyCollection::SetOptionalToRequired` · 006b31f0 · legions 0064c0b0
- [ ] `PropertyCollection::GetSubDataIDs` · 006b3230 · legions 0064c0f0
- [ ] `PropertyCollection::GetGroup` · 006b3270 · legions 0064c130
- [ ] `PropertyCollection::GetProperty` · 006b32b0 · legions 0064c170
- [ ] `PropertyCollection::GetOptionalProperty` · 006b3320 · legions 0064c1e0
- [ ] `PropertyCollection::InqGroup` · 006b3390 · legions 0064c250
- [ ] `PropertyCollection::InqProperty` · 006b33e0 · legions 0064c2a0
- [ ] `PropertyCollection::InqOptionalProperty` · 006b3410 · legions 0064c2d0
- [ ] `PropertyCollection::PropertyCollection` · 006b3440 · legions 0064c300
- [ ] `PropertyCollection::~PropertyCollection` · 006b3460 · legions 0064c320
- [ ] `PropertyCollection::InqGroupAt` · 006b34e0 · legions 0064c3a0
- [ ] `PropertyCollection::ContainsGroup` · 006b3510 · legions 0064c3d0
- [ ] `PropertyCollection::ContainsProperty` · 006b3560 · legions 0064c420
- [ ] `PropertyCollection::InqPropertyW` · 006b3580 · legions 0064c440
- [ ] `PropertyCollection::InqOptionalPropertyW` · 006b3600 · legions 0064c4c0

### SmartArray<PropertyGroup>
- [ ] `SmartArray<PropertyGroup>::AddToEnd` · 006b3680 · legions 0064c540

### PropertyCollection
- [ ] `PropertyCollection::AddGroup` · 006b36d0 · legions 0064c590
- [x] `PropertyCollection::UpdateGroup` · 006b3700 · legions 0064c5c0
- [x] `PropertyCollection::UpdateCollection` · 006b3770 · legions 0064c630
- [-] `PropertyCollection::ToFileNode` · 006b37c0 — text source format (admin tools)
- [-] `PropertyCollection::FromFileNode` · 006b3910 — text source format (admin tools)
- [ ] `PropertyCollection::operator=` · 006b3b10 · legions 0064c680
- [ ] `PropertyCollection::AddProperty` · 006b3b50 · legions 0064c6c0
- [ ] `PropertyCollection::AddOptionalProperty` · 006b3c20 · legions 0064c790
- [ ] `PropertyCollection::UpdateProperty` · 006b3cf0 · legions 0064c860

## `portal/shared/src/property/propertygroup.cpp.c` (2/25)

### PropertyGroup
- [ ] `PropertyGroup::InqPropertyGroupName` · 006b2890
- [ ] `PropertyGroup::Destroy` · 006b28b0 · legions 0064b8e0
- [ ] `PropertyGroup::GetSubDataIDs` · 006b2920 · legions 0064b950
- [~] `PropertyGroup::StreamPack` · 006b2960 · legions 0064b990 — unpack only, in ac2-net protocol::property / fx (types from a PropertyTypes resolver; Legions BaseProperty::StreamPack 0x0064B4E0 instruction-identical)
- [-] `PropertyGroup::ToFileNode` · 006b29b0 — text source format (admin tools)
- [ ] `PropertyGroup::RemoveProperty` · 006b2a00 · legions 0064b9e0
- [ ] `PropertyGroup::GetProperty` · 006b2a80 · legions 0064ba60
- [ ] `PropertyGroup::GetOptionalProperty` · 006b2ad0 · legions 0064bab0
- [ ] `PropertyGroup::InqProperty` · 006b2b20 · legions 0064bb00
- [ ] `PropertyGroup::InqOptionalProperty` · 006b2b80 · legions 0064bb60
- [ ] `PropertyGroup::PropertyGroup` · 006b2be0 · legions 0064bbc0
- [ ] `PropertyGroup::~PropertyGroup` · 006b2c10 · legions 0064bbf0
- [ ] `PropertyGroup::InqPropertyAt` · 006b2c80 · legions 0064bc60
- [ ] `PropertyGroup::InqOptionalPropertyAt` · 006b2cb0 · legions 0064bc90
- [ ] `PropertyGroup::InqPropertyGroupNameW` · 006b2ce0 · legions 0064bcc0
- [ ] `PropertyGroup::ContainsProperty` · 006b2db0 · legions 0064bd90
- [ ] `PropertyGroup::ContainsOptionalProperty` · 006b2dd0 · legions 0064bdb0

### SmartArray<BaseProperty>
- [ ] `SmartArray<BaseProperty>::AddToEnd` · 006b2df0 · legions 0064bdd0

### PropertyGroup
- [ ] `PropertyGroup::SetOptionalToRequired` · 006b2e40 · legions 0064be20
- [ ] `PropertyGroup::AddProperty` · 006b2e80 · legions 0064be60
- [ ] `PropertyGroup::AddOptionalProperty` · 006b2eb0 · legions 0064be90
- [x] `PropertyGroup::UpdateProperty` · 006b2ee0 · legions 0064bec0
- [x] `PropertyGroup::UpdateGroup` · 006b2ff0 · legions 0064bfd0
- [-] `PropertyGroup::FromFileNode` · 006b3030 — text source format (admin tools)
- [ ] `PropertyGroup::operator=` · 006b3150 · legions 0064c010

## `portal/shared/src/property/value/datafilepropertyvalue.cpp.c` (0/3)

### DataFilePropertyValue
- [-] `DataFilePropertyValue::FromFileNode` · 006b66e0 — text source format (admin tools)
- [-] `DataFilePropertyValue::ToFileNode` · 006b6810 — text source format (admin tools)
- [ ] `DataFilePropertyValue::GetValueAsString` · 006b6900

## `portal/shared/src/property/value/enumpropertyvalue.cpp.c` (0/3)

### EnumPropertyValue
- [ ] `EnumPropertyValue::GetValueAsString` · 006b69f0
- [-] `EnumPropertyValue::FromFileNode` · 006b6a10 — text source format (admin tools)
- [-] `EnumPropertyValue::ToFileNode` · 006b6b00 — text source format (admin tools)

## `portal/shared/src/property/value/packageidpropertyvalue.cpp.c` (0/3)

### PackageIDPropertyValue
- [-] `PackageIDPropertyValue::FromFileNode` · 006b6bf0 — text source format (admin tools)
- [-] `PackageIDPropertyValue::ToFileNode` · 006b6ce0 — text source format (admin tools)
- [ ] `PackageIDPropertyValue::GetValueAsString` · 006b6db0

## `portal/shared/src/stringtable/metalanguage/metalanguage.cpp.c` (0/8)

### StringTableMetaLanguage
- [ ] `StringTableMetaLanguage::GetMatchScore` · 00657600 · legions 00605f50
- [ ] `StringTableMetaLanguage::SetError` · 006576c0 · legions 00606010
- [ ] `StringTableMetaLanguage::SetError` · 00657720
- [ ] `StringTableMetaLanguage::ChooseChoice` · 00657780 · legions 006060d0

### SmartArray<PStringBase<unsigned_short>_>
- [ ] `SmartArray<PStringBase<unsigned_short>_>::operator=` · 006577d0 · legions 00606120

### StringTableMetaLanguage
- [ ] `StringTableMetaLanguage::TakeValidLetters` · 00657860 · legions 006061b0
- [ ] `StringTableMetaLanguage::RenderNodes` · 00657960 · legions 006062b0
- [ ] `StringTableMetaLanguage::RenderString` · 00658660 · legions 00606fb0

## `portal/shared/src/stringtable/metalanguage/metalanguage_rules.cpp.c` (0/15)

### StringTableMetaLanguage
- [ ] `StringTableMetaLanguage::GetRoot` · 00658b00 · legions 00607450
- [ ] `StringTableMetaLanguage::IsInChoiceBlock` · 00658b10 · legions 00607460
- [ ] `StringTableMetaLanguage::AddNode` · 00658b30 · legions 00607480
- [ ] `StringTableMetaLanguage::r_Flags` · 00658ba0
- [ ] `StringTableMetaLanguage::CheckForErrorsInChoiceBlock` · 00658bf0

### StringTableMetaLanguage::Node
- [ ] `StringTableMetaLanguage::Node::Reset` · 00658c70 · legions 006075c0

### StringTableMetaLanguage
- [ ] `StringTableMetaLanguage::r_ChoiceBlockEnd` · 00658cd0

### StringTableMetaLanguage::Node
- [ ] `StringTableMetaLanguage::Node::Node` · 00658cf0 · legions 00607640
- [ ] `StringTableMetaLanguage::Node::~Node` · 00658d50 · legions 006076a0

### StringTableMetaLanguage
- [ ] `StringTableMetaLanguage::InitRules` · 00658db0 · legions 00607700
- [ ] `StringTableMetaLanguage::CleanRules` · 00658df0 · legions 00607740
- [ ] `StringTableMetaLanguage::r_Text` · 00658e40 · legions 00607790
- [ ] `StringTableMetaLanguage::r_Var` · 00658ea0
- [ ] `StringTableMetaLanguage::r_ChoiceBlockBegin` · 006592b0
- [ ] `StringTableMetaLanguage::r_OutsideInfoBlock` · 006592f0 · legions 00607c60

## `portal/shared/src/stringtable/metalanguage/metalanguage_tokenizer.cpp.c` (0/7)

### (free functions)
- [ ] `IsSpecialChar` · 00659920 · legions 00608290

### StringTableMetaLanguage
- [ ] `StringTableMetaLanguage::GetCurrentStreamPosition` · 00659960 · legions 006082d0

### (free functions)
- [ ] `IsNumberLabel` · 00659970 · legions 006082e0
- [ ] `stringmeta_error` · 00659c10 · legions 00608580
- [ ] `IsFlagsBlock` · 00659c80 · legions 006085f0
- [ ] `stringmeta_lex` · 0065a300 · legions 00608c70

### StringTableMetaLanguage
- [ ] `StringTableMetaLanguage::LoadTokenizer` · 0065a310 · legions 00608c80

## `portal/shared/src/stringtable/metalanguage/metalanguage_utils.cpp.c` (3/14)

### StringTableMetaLanguage
- [ ] `StringTableMetaLanguage::IsEscapeChar` · 00656b20 · legions 00605470
- [ ] `StringTableMetaLanguage::IsNumberSingular` · 00656b50
- [ ] `StringTableMetaLanguage::GetMaleMetaLetters` · 00656b80
- [ ] `StringTableMetaLanguage::GetFemaleMetaLetters` · 00656bb0
- [ ] `StringTableMetaLanguage::GetTreasureMetaLetters` · 00656be0 · legions 00605530

### PStringBase<unsigned_short>
- [ ] `PStringBase<unsigned_short>::SetAtIndex` · 00656c40 · legions 00605590

### PSUtils
- [ ] `PSUtils::TrimExcessSpaces` · 00656cb0 · legions 00605600

### StringTableMetaLanguage
- [ ] `StringTableMetaLanguage::TrimExcessSpaces` · 00656ea0 · legions 006057f0
- [ ] `StringTableMetaLanguage::CapitalizeFirstLetter` · 00656ee0 · legions 00605830
- [ ] `StringTableMetaLanguage::RetrieveMetaLetters` · 00656fd0 · legions 00605920
- [x] `StringTableMetaLanguage::StripMetaLetters` · 006570f0 · legions 00605a40 — ac2-script glue::string_text (tables via ScriptData::string_table)
- [x] `StringTableMetaLanguage::EscapeString` · 006571c0 · legions 00605b10 — ac2-script glue
- [x] `StringTableMetaLanguage::UnescapeString` · 006572d0 · legions 00605c20 — ac2-script glue::string_text (tables via ScriptData::string_table)
- [ ] `StringTableMetaLanguage::ApplyMetaTagsToName` · 006573f0 · legions 00605d40

## `portal/shared/src/stringtable/stringinfo.cpp.c` (16/56)

### StringInfoData
- [ ] `StringInfoData::operator==` · 00650880
- [ ] `StringInfoData::operator!=` · 006508b0

### StringInfo
- [x] `StringInfo::SetTableID` · 006508d0 · legions 005ffba0 — ac2-script glue::string_text (tables via ScriptData::string_table)
- [ ] `StringInfo::ApplyMetasToPlayerName` · 00650900 · legions 005ffbd0
- [ ] `StringInfo::ReportError` · 00650920
- [x] `StringInfo::SetTableEnum` · 00650a50 · legions 005ffc10 — ac2-script glue::string_text (tables via ScriptData::string_table)
- [x] `StringInfo::SetStringIDandTableEnum` · 00650a90 · legions 005ffc50 — ac2-script glue::string_info (lookups via ScriptData)
- [x] `StringInfo::SetStringIDandTableID` · 00650ad0 · legions 005ffc90 — ac2-script glue::string_info (lookups via ScriptData)
- [ ] `StringInfo::operator%` · 00650b40 · legions 005ffd00
- [x] `StringInfo::SetTableName` · 00650b90 · legions 005ffd50 — ac2-script glue::string_info (lookups via ScriptData)
- [x] `StringInfo::SetStringIDandTableName` · 00650c50 · legions 005ffdf0 — ac2-script glue::string_info (lookups via ScriptData)
- [x] `StringInfo::GetString` · 00650c70 · legions 005ffe10 — ac2-script glue::string_text (tables via ScriptData::string_table)
- [x] `StringInfo::GetStringWithMetaTags` · 00650ce0 · legions 005ffe80 — ac2-script glue::string_text (tables via ScriptData::string_table)
- [x] `StringInfo::InqString` · 00650d00 · legions 005ffea0 — ac2-script glue::string_text (tables via ScriptData::string_table)
- [ ] `StringInfo::InqStringTableFile` · 00650da0
- [x] `StringInfo::SetLiteralValue` · 00650e30 · legions 005fffd0 — ac2-script glue::string_info (lookups via ScriptData)
- [ ] `StringInfo::GetLiteralValue` · 00650ef0 · legions 00600090
- [ ] `StringInfo::SetVarInTable` · 00651000 · legions 006001a0
- [~] `StringInfo::GetLogString` · 00651220 · legions 00600410 — ac2-script glue::string_text; table rendering is the ScriptData::render_string hook (plain strings only by default)
- [-] `StringInfo::FromFileNode` · 00651390 · legions 00600580 — text source format (admin tools)
- [-] `StringInfo::FromFileNode` · 006515c0 · legions 006007b0 — text source format (admin tools)
- [-] `StringInfo::ToFileNode` · 00651620 — text source format (admin tools)
- [x] `StringInfo::IsValid` · 00651730 · legions 00600840 — ac2-script glue::string_text (tables via ScriptData::string_table)
- [ ] `StringInfo::GetSubDataIDs` · 00651860 · legions 00600950
- [ ] `StringInfo::GetSubPrivateIDs` · 006518f0 · legions 006009e0
- [ ] `StringInfo::GetPackSize` · 006519d0 · legions 00600aa0
- [x] `StringInfo::Pack` · 00651b20 · legions 00600bf0
- [~] `StringInfo::InqStringInternal` · 00651dd0 · legions 00600ea0 — ac2-script glue::string_text; table rendering is the ScriptData::render_string hook (plain strings only by default)
- [ ] `StringInfo::ClearVars` · 00651fb0 · legions 00601080
- [x] `StringInfo::Reset` · 00652070 · legions 00601140 — ac2-script glue::string_info (lookups via ScriptData)
- [x] `StringInfo::operator==` · 00652080 · legions 00601150 — ac2-script glue::string_text (tables via ScriptData::string_table)
- [ ] `StringInfo::IsReady` · 00652140 · legions 00601210
- [ ] `StringInfo::~StringInfo` · 00652150 · legions 00601220
- [x] `StringInfo::AddVar` · 006521d0 · legions 006012a0 — ac2-script glue::string_info (lookups via ScriptData)
- [ ] `StringInfo::AddVariable_Int` · 006522b0 · legions 00601380
- [ ] `StringInfo::AddVariable_FormattedInt` · 00652330 · legions 00601400
- [ ] `StringInfo::AddVariable_UInt` · 006523c0 · legions 00601490
- [ ] `StringInfo::AddVariable_FormattedUInt` · 00652440 · legions 00601510
- [ ] `StringInfo::AddVariable_LInt` · 006524d0 · legions 006015a0
- [ ] `StringInfo::AddVariable_FormattedLInt` · 00652550 · legions 00601620
- [ ] `StringInfo::AddVariable_ULInt` · 006525e0 · legions 006016b0
- [ ] `StringInfo::AddVariable_FormattedULInt` · 00652660 · legions 00601730
- [ ] `StringInfo::AddVariable_Float` · 006526f0 · legions 006017c0
- [ ] `StringInfo::AddVariable_FormattedFloat` · 00652780 · legions 00601850
- [ ] `StringInfo::CopyVariables` · 00652820 · legions 006018f0
- [ ] `StringInfo::StringInfo` · 00652930 · legions 00601a00
- [ ] `StringInfo::operator=` · 00652980 · legions 00601a50

### SID_StringInfo
- [ ] `SID_StringInfo::SID_StringInfo` · 006529f0 · legions 00601ac0

### StringInfo
- [ ] `StringInfo::StringInfo` · 00652a80 · legions 00601b50
- [ ] `StringInfo::AddVariable_StringID` · 00652ac0 · legions 00601b90
- [ ] `StringInfo::AddVariable_StringIDandTableEnum` · 00652b50 · legions 00601c20
- [ ] `StringInfo::AddVariable_StringInfo` · 00652bf0 · legions 00601cc0
- [ ] `StringInfo::AddVariable_String` · 00652c70 · legions 00601d40
- [ ] `StringInfo::AsyncGetString` · 00652d10 · legions 00601de0
- [x] `StringInfo::UnPack` · 00652db0 · legions 00601e80 — ac2-script reference/native unpack
- [ ] `StringInfo::AddVariable_String` · 00653190 · legions 00602260

## `portal/shared/src/stringtable/stringinfodata.cpp.c` (0/18)

### SID_Int
- [ ] `SID_Int::DuplicateYourself` · 00656180
- [ ] `SID_Int::operator==` · 006561c0

### SID_FormattedInt
- [ ] `SID_FormattedInt::DuplicateYourself` · 00656200

### SID_UInt
- [ ] `SID_UInt::DuplicateYourself` · 00656240

### SID_FormattedUInt
- [ ] `SID_FormattedUInt::DuplicateYourself` · 00656280

### SID_FormattedInt
- [ ] `SID_FormattedInt::operator==` · 006562c0

### SID_Float
- [ ] `SID_Float::DuplicateYourself` · 00656310
- [ ] `SID_Float::operator==` · 00656350

### SID_FormattedFloat
- [ ] `SID_FormattedFloat::DuplicateYourself` · 00656390
- [ ] `SID_FormattedFloat::operator==` · 006563e0

### SID_StringInfo
- [ ] `SID_StringInfo::DuplicateYourself` · 00656430
- [ ] `SID_StringInfo::operator==` · 00656490

### SID_LInt
- [ ] `SID_LInt::DuplicateYourself` · 006564c0
- [ ] `SID_LInt::operator==` · 00656510

### SID_FormattedLInt
- [ ] `SID_FormattedLInt::DuplicateYourself` · 00656550

### SID_ULInt
- [ ] `SID_ULInt::DuplicateYourself` · 006565a0

### SID_FormattedULInt
- [ ] `SID_FormattedULInt::DuplicateYourself` · 006565f0

### SID_FormattedLInt
- [ ] `SID_FormattedLInt::operator==` · 00656640

## `portal/shared/src/stringtable/stringstate.cpp.c` (0/30)

### StringStateInfo
- [ ] `StringStateInfo::GetNumDecDigits` · 00653200 · legions 006022d0
- [ ] `StringStateInfo::GetLeadingZero` · 00653220 · legions 006022f0
- [ ] `StringStateInfo::GetBase` · 00653240 · legions 00602310
- [ ] `StringStateInfo::GetTopGoreLevel` · 00653260 · legions 00602330
- [ ] `StringStateInfo::GetTopBloodLevel` · 006532a0 · legions 00602370
- [ ] `StringStateInfo::GetGroupingSize` · 006532e0 · legions 006023b0
- [ ] `StringStateInfo::IsNumberSingular` · 00653300
- [ ] `StringStateInfo::InqIMEAppearance` · 00653370 · legions 00602440
- [ ] `StringStateInfo::Startup` · 00653430 · legions 00602500
- [ ] `StringStateInfo::SetTopGoreLevelOverride` · 00653480 · legions 00602550
- [ ] `StringStateInfo::SetTopBloodColorOverride` · 00653490 · legions 00602560

### StringState
- [ ] `StringState::GetPackSize` · 006534a0 · legions 00602570
- [ ] `StringState::Pack` · 006534d0 · legions 006025a0
- [ ] `StringState::UnPack` · 00653510 · legions 006025e0

### StringStateInfo
- [ ] `StringStateInfo::Shutdown` · 00653530 · legions 00602600

### StringState
- [ ] `StringState::StringState` · 00653570 · legions 00602640
- [ ] `StringState::GetDBOType` · 00653650
- [-] `StringState::GetPackSize` · 00653670 — inline copy of 006534a0
- [ ] `StringState::Pack` · 00653680
- [ ] `StringState::UnPack` · 00653690
- [ ] `StringState::~StringState` · 006536a0 · legions 00602770

### StringStateInfo
- [ ] `StringStateInfo::GetNumerals` · 006537d0 · legions 006028a0
- [ ] `StringStateInfo::GetGroupingSeperator` · 00653820 · legions 006028f0
- [ ] `StringStateInfo::GetDecimalSeperator` · 00653870 · legions 00602940
- [ ] `StringStateInfo::GetNegNumFormat` · 006538c0 · legions 00602990
- [ ] `StringStateInfo::GetMaleMetaLetters` · 00653910
- [ ] `StringStateInfo::GetFemaleMetaLetters` · 00653960
- [ ] `StringStateInfo::GetTreasureMetaLetters` · 006539c0 · legions 00602a90

### StringState
- [ ] `StringState::Allocate` · 00653a90
- [ ] `StringState::StreamPack` · 00653ac0 · legions 00602b90

## `portal/shared/src/stringtable/stringtable.cpp.c` (3/28)

### StringTableString
- [ ] `StringTableString::Destroy` · 00653d60 · legions 00602e30

### StringTable
- [ ] `StringTable::FloatToString` · 00653e00 · legions 00602ed0
- [ ] `StringTable::GetPackSize` · 00654260
- [ ] `StringTable::Pack` · 006542f0

### StringTableString
- [ ] `StringTableString::StringTableString` · 00654400 · legions 006034d0
- [ ] `StringTableString::~StringTableString` · 00654470 · legions 00603540

### StringTable
- [ ] `StringTable::GetSubDataIDs` · 006544d0
- [ ] `StringTable::Destroy` · 00654540 · legions 00603610

### StringTableString
- [ ] `StringTableString::IsVarNameTableWorthPacking` · 00654640 · legions 00603710

### StringTable
- [ ] `StringTable::~StringTable` · 00654870 · legions 00603940
- [ ] `StringTable::GetDBOType` · 00654900
- [ ] `StringTable::IntToString` · 00654910 · legions 006039e0
- [ ] `StringTable::UIntToString` · 00654940 · legions 00603a10
- [ ] `StringTable::LIntToString` · 00654980 · legions 00603a50
- [ ] `StringTable::ULIntToString` · 006549b0 · legions 00603a80

### StringTableString
- [ ] `StringTableString::GetPackSize` · 00654a30
- [ ] `StringTableString::Pack` · 00654b00
- [x] `StringTableString::UnPack` · 00654cb0 — ac2-db string_table.rs: all 10,598 Legions tables decode

### StringTable
- [x] `StringTable::GetStringTableString` · 00654f10 · legions 00603fe0 — ac2-db string_table.rs: all 10,598 Legions tables decode

### StringTableString
- [ ] `StringTableString::CopyInto` · 00654f50 · legions 00604020

### StringTable
- [ ] `StringTable::GetString` · 00655170
- [ ] `StringTable::GetRawString` · 00655260
- [ ] `StringTable::GetString` · 00655420
- [ ] `StringTable::SaveToFile` · 006557b0
- [ ] `StringTable::StringTable` · 00655d70 · legions 006046c0
- [x] `StringTable::UnPack` · 00655de0 — ac2-db string_table.rs: all 10,598 Legions tables decode
- [ ] `StringTable::CopyInto` · 00655f90
- [ ] `StringTable::Allocate` · 00656170

## `portal/shared/src/stringtable/stringvartable.cpp.c` (0/9)

### StringVarTable
- [ ] `StringVarTable::StringVarTable` · 00656690 · legions 00604fe0
- [ ] `StringVarTable::SetVariable` · 00656700 · legions 00605050
- [ ] `StringVarTable::SetVariableFromInt` · 006567d0 · legions 00605120
- [ ] `StringVarTable::SetVariableFromUInt` · 00656840 · legions 00605190
- [ ] `StringVarTable::SetVariableFromLInt` · 006568b0 · legions 00605200
- [ ] `StringVarTable::SetVariableFromULInt` · 00656930 · legions 00605280
- [ ] `StringVarTable::SetVariableFromFloat` · 006569b0 · legions 00605300
- [ ] `StringVarTable::GetVariable` · 00656a30 · legions 00605380
- [ ] `StringVarTable::~StringVarTable` · 00656af0 · legions 00605440

## `portal/shared/src/taboo/tabootable.cpp.c` (0/13)

### TabooTable
- [ ] `TabooTable::CreateCheckString` · 0065a5c0 · legions 00608f30
- [ ] `TabooTable::StringMatchesFilter` · 0065a650 · legions 00608fc0
- [ ] `TabooTable::Pack` · 0065a6f0
- [ ] `TabooTable::UnPack` · 0065a750 · legions 006090c0
- [ ] `TabooTable::CheckCharacters` · 0065a790 · legions 00609100
- [ ] `TabooTable::FormatStringForAudience` · 0065a920 · legions 00609290

### PList<PStringBase<unsigned_short>_>
- [ ] `PList<PStringBase<unsigned_short>_>::operator=` · 0065aab0 · legions 00609420

### TabooTable
- [ ] `TabooTable::CheckCensors` · 0065aaf0 · legions 00609460

### PHashTable<unsigned_long,PList<PStringBase<unsigned_short>_>_>
- [ ] `PHashTable<unsigned_long,PList<PStringBase<unsigned_short>_>_>::StreamPack` · 0065ad60 · legions 006096d0
- [ ] `PHashTable<unsigned_long,PList<PStringBase<unsigned_short>_>_>::~PHashTable<unsigned_long,PList<PStringBase<unsigned_short>_>_>` · 0065af50 · legions 006098c0

### TabooTable
- [ ] `TabooTable::GetDBOType` · 0065b000
- [ ] `TabooTable::TabooTable` · 0065b010
- [ ] `TabooTable::Allocate` · 0065b050

## `portal/shared/src/taboo/tabootableadaptor.cpp.c` (0/3)

### TabooTableAdaptor
- [ ] `TabooTableAdaptor::CheckCharactersW` · 0065b090 · legions 00609a00
- [ ] `TabooTableAdaptor::CheckCensorsW` · 0065b100 · legions 00609a50
- [ ] `TabooTableAdaptor::FormatStringForAudienceW` · 0065b170 · legions 00609aa0
