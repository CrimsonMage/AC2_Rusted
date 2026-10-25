# ac2-audio: Sound

367 retail functions: 123 done, 32 partial, 131 to do, 81 skipped.
Legend and workflow: [README](README.md).

## `portal/client/src/cregion/csound/cambientsoundinfo.cpp.c` (1/13)

### CAmbientSoundInfo
- [ ] `CAmbientSoundInfo::Destroy` · 0052a5f0 · legions 00510150
- [ ] `CAmbientSoundInfo::GetSubDataIDs` · 0052a660 · legions 005101c0
- [x] `CAmbientSoundInfo::StreamPack` · 0052a6d0 · legions 00510230 — ac2-audio/ac2-client: ambient environments, groove
- [ ] `CAmbientSoundInfo::CAmbientSoundInfo` · 0052a900 · legions 00510460
- [ ] `CAmbientSoundInfo::~CAmbientSoundInfo` · 0052a940 · legions 005104a0

### SmartArray<CContinSoundInfo>
- [ ] `SmartArray<CContinSoundInfo>::operator=` · 0052a9b0 · legions 00510510

### PSmartArray<CContinSoundInfo>
- [ ] `PSmartArray<CContinSoundInfo>::StreamPack` · 0052aa20 · legions 00510580

### SmartArray<CIntermitSoundInfo>
- [ ] `SmartArray<CIntermitSoundInfo>::operator=` · 0052aac0 · legions 00510620

### PSmartArray<CIntermitSoundInfo>
- [ ] `PSmartArray<CIntermitSoundInfo>::StreamPack` · 0052ab30 · legions 00510690

### SmartArray<CContinSoundInfo>
- [ ] `SmartArray<CContinSoundInfo>::AddToEnd` · 0052abd0

### SmartArray<CIntermitSoundInfo>
- [ ] `SmartArray<CIntermitSoundInfo>::AddToEnd` · 0052ac20

### CAmbientSoundInfo
- [ ] `CAmbientSoundInfo::operator=` · 0052ac70 · legions 00510730
- [-] `CAmbientSoundInfo::FromFileNode` · 0052acd0 — text source format (admin tools)

## `portal/client/src/cregion/csound/ccontinsoundinfo.cpp.c` (3/6)

### PFileNode
- [ ] `PFileNode::MakeSubNodeAsFloat` · 00529500 · legions 0050fdd0

### CContinSoundInfo
- [ ] `CContinSoundInfo::operator=` · 00529560 · legions 0050fe30
- [x] `CContinSoundInfo::CContinSoundInfo` · 005295a0 — ac2-audio/ac2-client: ambient environments, groove
- [x] `CContinSoundInfo::CContinSoundInfo` · 005295d0 · legions 0050fea0 — ac2-audio/ac2-client: ambient environments, groove
- [x] `CContinSoundInfo::StreamPack` · 00529610 · legions 0050fee0 — ac2-audio/ac2-client: ambient environments, groove
- [-] `CContinSoundInfo::FromFileNode` · 005296c0 — text source format (admin tools)

## `portal/client/src/cregion/csound/cintermitsoundinfo.cpp.c` (3/5)

### CIntermitSoundInfo
- [ ] `CIntermitSoundInfo::operator=` · 00529d10 · legions 0050ff90
- [x] `CIntermitSoundInfo::CIntermitSoundInfo` · 00529d60 — ac2-audio/ac2-client: ambient environments, groove
- [x] `CIntermitSoundInfo::CIntermitSoundInfo` · 00529d90 · legions 00510010 — ac2-audio/ac2-client: ambient environments, groove
- [x] `CIntermitSoundInfo::StreamPack` · 00529de0 · legions 00510060 — ac2-audio/ac2-client: ambient environments, groove
- [-] `CIntermitSoundInfo::FromFileNode` · 00529ed0 — text source format (admin tools)

## `portal/client/src/cregion/csound/csounddesc.cpp.c` (1/11)

### CSoundDesc
- [ ] `CSoundDesc::Destroy` · 00523550 · legions 0050d560
- [ ] `CSoundDesc::GetSubDataIDs` · 00523590 · legions 0050d5a0
- [x] `CSoundDesc::InqSoundInfo` · 005235d0 · legions 0050d5e0 — ac2-audio/ac2-client: ambient environments, groove
- [ ] `CSoundDesc::CSoundDesc` · 00523700 · legions 0050d710
- [ ] `CSoundDesc::GetDBOType` · 00523730
- [ ] `CSoundDesc::~CSoundDesc` · 00523740 · legions 0050d750

### PSmartArray<CAmbientSoundInfo>
- [ ] `PSmartArray<CAmbientSoundInfo>::StreamPack` · 005237c0 · legions 0050d7d0

### CSoundDesc
- [ ] `CSoundDesc::Allocate` · 00523860
- [-] `CSoundDesc::CheckDupAdd` · 00523890 — text source format (admin tools)
- [-] `CSoundDesc::LoadAmbSound` · 00523910 — text source loader, admin build only
- [-] `CSoundDesc::FromFileNode` · 005239a0 — text source format (admin tools)

## `portal/client/src/sound/ambient/ambientsound.cpp.c` (8/11)

### AmbientSound
- [x] `AmbientSound::Init` · 0072aef0 — ac2-audio/ac2-client: ambient environments, groove
- [x] `AmbientSound::AmbientSound` · 0072af00 · legions 006a91c0 — ac2-audio/ac2-client: ambient environments, groove
- [ ] `AmbientSound::~AmbientSound` · 0072b000 · legions 006a92c0
- [~] `AmbientSound::Clear` · 0072b030 · legions 006a92f0 — ac2-audio/ac2-client ambient: four-copy contin spread unused by Legions; interior visibility from the player's stab list; groove objects read every update
- [x] `AmbientSound::UseTime` · 0072b330 · legions 006a95f0 — ac2-audio/ac2-client: ambient environments, groove
- [x] `AmbientSound::ChangeCellStart` · 0072b4b0 · legions 006a9710 — ac2-audio/ac2-client: ambient environments, groove
- [x] `AmbientSound::FindSoundDesc` · 0072b500 · legions 006a9760 — ac2-audio/ac2-client: ambient environments, groove
- [x] `AmbientSound::ChangeCellEnd` · 0072b540 · legions 006a97a0 — ac2-audio ambient.rs change_cell_end: the dominant environment's hand-off (SoundManager->GetReferences, 0x739420) is a folded 'return 1' stub in admin and Legions alike (mov eax,1; ret 4): retail applies no EnvInfo reverb; ac2-audio records the environment only.
- [x] `AmbientSound::DoSoundUpdate` · 0072b6c0 · legions 006a98d0 — ac2-audio/ac2-client: ambient environments, groove

### DBObjGrabber<CSoundDesc>
- [ ] `DBObjGrabber<CSoundDesc>::DBObjGrabber<CSoundDesc>` · 0072b740 · legions 006a9950

### AmbientSound
- [x] `AmbientSound::AddCellSound` · 0072b780 · legions 006a9990 — ac2-audio/ac2-client: ambient environments, groove

## `portal/client/src/sound/ambient/ambientsounddesc.cpp.c` (9/11)

### AmbientSoundDesc
- [x] `AmbientSoundDesc::TimedOut` · 0072ba00 · legions 006a9b30 — ac2-audio/ac2-client: ambient environments, groove
- [ ] `AmbientSoundDesc::Destroy` · 0072ba30 · legions 006a9b60
- [x] `AmbientSoundDesc::UseTime` · 0072bae0 · legions 006a9c10 — ac2-audio/ac2-client: ambient environments, groove
- [x] `AmbientSoundDesc::Reset` · 0072bb80 · legions 006a9cb0 — ac2-audio/ac2-client: ambient environments, groove
- [x] `AmbientSoundDesc::AddSound` · 0072bbe0 · legions 006a9d10 — ac2-audio/ac2-client: ambient environments, groove
- [x] `AmbientSoundDesc::SetAsDominant` · 0072bd10 · legions 006a9e40 — ac2-audio/ac2-client: ambient environments, groove
- [x] `AmbientSoundDesc::NoLongerDominant` · 0072bd40 · legions 006a9e70 — ac2-audio/ac2-client: ambient environments, groove
- [ ] `AmbientSoundDesc::~AmbientSoundDesc` · 0072bd70 · legions 006a9ea0
- [x] `AmbientSoundDesc::CalcWeights` · 0072bd90 · legions 006a9ec0 — ac2-audio/ac2-client: ambient environments, groove
- [x] `AmbientSoundDesc::AddIntermitSound` · 0072be50 · legions 006a9f80 — ac2-audio/ac2-client: ambient environments, groove
- [x] `AmbientSoundDesc::AmbientSoundDesc` · 0072bf00 · legions 006aa030 — ac2-audio/ac2-client: ambient environments, groove

## `portal/client/src/sound/ambient/ambsoundobject.cpp.c` (6/8)

### AmbSoundObject
- [ ] `AmbSoundObject::~AmbSoundObject` · 0072ca30 · legions 006aab60
- [ ] `AmbSoundObject::CalcHeight` · 0072ca60 · legions 0056fff0
- [x] `AmbSoundObject::AddSoundStart` · 0072ca70 · legions 006aab90 — ac2-audio/ac2-client: ambient environments, groove
- [x] `AmbSoundObject::AddSoundEnd` · 0072caa0 · legions 006aabc0 — ac2-audio/ac2-client: ambient environments, groove
- [x] `AmbSoundObject::GetIntermitLoc` · 0072cae0 · legions 006aac00 — ac2-audio/ac2-client: ambient environments, groove
- [x] `AmbSoundObject::AmbSoundObject` · 0072cbc0 · legions 006aace0 — ac2-audio/ac2-client: ambient environments, groove
- [x] `AmbSoundObject::GetDir` · 0072cc40 · legions 006aad60 — ac2-audio/ac2-client: ambient environments, groove
- [x] `AmbSoundObject::AddSound` · 0072ccf0 · legions 006aae10 — ac2-audio/ac2-client: ambient environments, groove

## `portal/client/src/sound/ambient/continmusicobject.cpp.c` (4/4)

### ContinMusicObject
- [x] `ContinMusicObject::ContinMusicObject` · 0072d440 · legions 006ab530 — ac2-audio/ac2-client: ambient environments, groove

### ContinSoundObject
- [x] `ContinSoundObject::StopAllSounds` · 0072d460 · legions 006ab550 — ac2-audio/ac2-client: ambient environments, groove

### ContinMusicObject
- [x] `ContinMusicObject::PlaySoundA` · 0072d480 · legions 006ab570 — ac2-audio/ac2-client: ambient environments, groove
- [x] `ContinMusicObject::Is3D` · 0072d4f0 · legions 006ab5e0 — ac2-audio/ac2-client: ambient environments, groove

## `portal/client/src/sound/ambient/continobject.cpp.c` (4/8)

### ContinObject
- [x] `ContinObject::ContinObject` · 0072d180 · legions 006ab280 — ac2-audio/ac2-client: ambient environments, groove
- [ ] `ContinObject::GetID` · 0072d1d0 · legions 006ab2d0
- [ ] `ContinObject::GetSoundPosition` · 0072d1f0 · legions 006ab2f0
- [ ] `ContinObject::get_global_velocity` · 0072d200 · legions 006ab300
- [ ] `ContinObject::~ContinObject` · 0072d220 · legions 006ab320
- [x] `ContinObject::Init` · 0072d230 · legions 006ab330 — ac2-audio/ac2-client: ambient environments, groove
- [x] `ContinObject::ShouldPlay` · 0072d250 · legions 006ab350 — ac2-audio/ac2-client: ambient environments, groove
- [x] `ContinObject::UpdatePos` · 0072d2a0 · legions 006ab3a0 — ac2-audio/ac2-client: ambient environments, groove

## `portal/client/src/sound/ambient/continsoundmanager.cpp.c` (6/12)

### ContinSoundManager
- [x] `ContinSoundManager::SetVolume` · 0072c030 · legions 006aa160 — ac2-audio/ac2-client: ambient environments, groove
- [ ] `ContinSoundManager::CalcHeight` · 0072c040
- [ ] `ContinSoundManager::UpdatePos` · 0072c070 · legions 006aa1a0
- [x] `ContinSoundManager::StopAllSounds` · 0072c0b0 · legions 006aa1e0 — ac2-audio/ac2-client: ambient environments, groove
- [x] `ContinSoundManager::PlaySounds` · 0072c100 · legions 006aa230 — ac2-audio/ac2-client: ambient environments, groove
- [ ] `ContinSoundManager::CalcAngleSum` · 0072c180 · legions 006aa2b0
- [x] `ContinSoundManager::NowDominant` · 0072c200 · legions 006aa330 — ac2-audio/ac2-client: ambient environments, groove
- [x] `ContinSoundManager::NoLongerDominant` · 0072c220 · legions 006aa350 — ac2-audio/ac2-client: ambient environments, groove
- [ ] `ContinSoundManager::Is3D` · 0072c240
- [ ] `ContinSoundManager::~ContinSoundManager` · 0072c260 · legions 006aa390
- [~] `ContinSoundManager::ReCalcContinSounds` · 0072c300 · legions 006aa430 — ac2-audio ambient.rs/ambient_objects.rs: a dominant-only music re-requested on each weighing; the queue holds one playing copy and one waiting copy (replaced by the newest, as QueuePrimaryInternal aborts it). ac2-client tests/ambient_music_7160 keeps one copy sounding and at most one waiting. The 4-copy spread (0x10) is still unused by Legions sounds.
- [x] `ContinSoundManager::ContinSoundManager` · 0072c5b0 · legions 006aa6e0 — ac2-audio/ac2-client: ambient environments, groove

## `portal/client/src/sound/ambient/continsoundobject.cpp.c` (4/4)

### ContinSoundObject
- [x] `ContinSoundObject::ContinSoundObject` · 0072d2f0 · legions 006ab3f0 — ac2-audio/ac2-client: ambient environments, groove
- [x] `ContinSoundObject::PlaySoundA` · 0072d310 · legions 006ab410 — ac2-audio/ac2-client: ambient environments, groove
- [x] `ContinSoundObject::Is3D` · 0072d380 · legions 006ab480 — ac2-audio/ac2-client: ambient environments, groove
- [x] `ContinSoundObject::StopSound` · 0072d3e0 · legions 006ab4d0 — ac2-audio/ac2-client: ambient environments, groove

## `portal/client/src/sound/ambient/intermitmusicobject.cpp.c` (3/4)

### IntermitMusicObject
- [ ] `IntermitMusicObject::IntermitMusicObject` · 0072d000 · legions 006ab110
- [x] `IntermitMusicObject::Is3D` · 0072d030 — ac2-audio/ac2-client: ambient environments, groove
- [x] `IntermitMusicObject::Play_Internal` · 0072d090 · legions 006ab190 — ac2-audio/ac2-client: ambient environments, groove
- [x] `IntermitMusicObject::IsPlaying_Internal` · 0072d150 · legions 006ab250 — ac2-audio/ac2-client: ambient environments, groove

## `portal/client/src/sound/ambient/intermitobject.cpp.c` (2/7)

### IntermitObject
- [ ] `IntermitObject::~IntermitObject` · 0072c6d0 · legions 006aa800
- [ ] `IntermitObject::GetSoundPosition` · 0072c710 · legions 006aa840
- [ ] `IntermitObject::get_global_velocity` · 0072c720 · legions 006aa850
- [~] `IntermitObject::CalcHeight` · 0072c750 — ac2-audio/ac2-client ambient: four-copy contin spread unused by Legions; interior visibility from the player's stab list; groove objects read every update
- [x] `IntermitObject::IntermitObject` · 0072c790 · legions 006aa8c0 — ac2-audio/ac2-client: ambient environments, groove
- [x] `IntermitObject::PlayCheck` · 0072c890 · legions 006aa9c0 — ac2-audio/ac2-client: ambient environments, groove
- [~] `IntermitObject::Play` · 0072c970 · legions 006aaaa0 — ac2-audio/ac2-client ambient: four-copy contin spread unused by Legions; interior visibility from the player's stab list; groove objects read every update

## `portal/client/src/sound/ambient/intermitsoundobject.cpp.c` (3/5)

### IntermitSoundObject
- [ ] `IntermitSoundObject::IntermitSoundObject` · 0072ce50 · legions 006aaf70
- [x] `IntermitSoundObject::Is3D` · 0072ce80 — ac2-audio/ac2-client: ambient environments, groove
- [x] `IntermitSoundObject::Play_Internal` · 0072cee0 · legions 006aaff0 — ac2-audio/ac2-client: ambient environments, groove
- [x] `IntermitSoundObject::IsPlaying_Internal` · 0072cf90 · legions 006ab0a0 — ac2-audio/ac2-client: ambient environments, groove
- [ ] `IntermitSoundObject::SetTempo_Internal` · 0072cfd0 · legions 006ab0e0

## `portal/client/src/sound/ambient/sounddirection.cpp.c` (6/6)

### SoundDirection
- [x] `SoundDirection::SoundDirection` · 0072d590 · legions 006ab670 — ac2-audio/ac2-client: ambient environments, groove
- [x] `SoundDirection::Reset` · 0072d5b0 · legions 006ab690 — ac2-audio/ac2-client: ambient environments, groove
- [x] `SoundDirection::AddToWeight` · 0072d5d0 · legions 006ab6b0 — ac2-audio/ac2-client: ambient environments, groove
- [x] `SoundDirection::AddToDistance` · 0072d5e0 · legions 006ab6c0 — ac2-audio/ac2-client: ambient environments, groove
- [x] `SoundDirection::CalcWeights` · 0072d620 · legions 006ab700 — ac2-audio/ac2-client: ambient environments, groove
- [x] `SoundDirection::ChooseSoundDistance` · 0072d630 · legions 006ab710 — ac2-audio/ac2-client: ambient environments, groove

## `portal/client/src/sound/dbmusicobj.cpp.c` (0/5)

### DBMusicObj
- [ ] `DBMusicObj::GetSubDataIDs` · 005a30e0
- [ ] `DBMusicObj::StreamPack` · 005a3110
- [ ] `DBMusicObj::DBMusicObj` · 005a31f0 · legions 0056fd70
- [ ] `DBMusicObj::GetDBOType` · 005a3220
- [ ] `DBMusicObj::Allocate` · 005a3230

## `portal/client/src/sound/dbwave.cpp.c` (4/12)

### std::strstream
- [ ] `std::strstream::`vbase_destructor'` · 005a3580 · legions 00570080

### DBWave
- [ ] `DBWave::~DBWave` · 005a35a0 · legions 005700a0
- [x] `DBWave::GetDBOType` · 005a35b0 — ac2-audio: SoundManager / DirectXPlayingSound / sound records
- [ ] `DBWave::Pack` · 005a35c0
- [x] `DBWave::GetTime` · 005a3680 · legions 00570180 — ac2-audio: SoundManager / DirectXPlayingSound / sound records
- [ ] `DBWave::ZeroInit` · 005a36d0 · legions 005701d0
- [ ] `DBWave::ReleaseAll` · 005a36f0 · legions 005701f0
- [ ] `DBWave::Allocator` · 005a3720
- [ ] `DBWave::CompressAudio` · 005a3740
- [x] `DBWave::DBWave` · 005a3a50 · legions 00570240 — ac2-audio: SoundManager / DirectXPlayingSound / sound records
- [x] `DBWave::UnPack` · 005a3ab0 — ac2-audio: SoundManager / DirectXPlayingSound / sound records
- [-] `DBWave::LoadFromFile` · 005a3e50 — text source loader, admin build only

## `portal/client/src/sound/directx/directxmusicloader.cpp.c` (0/32)

### DirectXMusicLoader
- [ ] `DirectXMusicLoader::Initialize` · 005a71e0 · legions 00573870
- [ ] `DirectXMusicLoader::AddRef` · 005a7260 · legions 005738f0
- [ ] `DirectXMusicLoader::QueryInterface` · 005a7280 · legions 00573910
- [ ] `DirectXMusicLoader::LoadFromStream` · 005a72e0 · legions 00573970

### DBObjGrabber<DBMusicObj>
- [ ] `DBObjGrabber<DBMusicObj>::DBObjGrabber<DBMusicObj>` · 005a7380 · legions 00573a00

### MusicByteStream
- [ ] `MusicByteStream::MusicByteStream` · 005a74d0 · legions 00573b50
- [ ] `MusicByteStream::QueryInterface` · 005a7520 · legions 00573ba0
- [ ] `MusicByteStream::AddRef` · 005a75d0 · legions 00573c50
- [ ] `MusicByteStream::Read` · 005a75f0 · legions 00573c70
- [ ] `MusicByteStream::Revert` · 005a7670 · legions 00573cf0
- [ ] `MusicByteStream::CopyTo` · 005a7680 · legions 00573d00
- [ ] `MusicByteStream::GetLoader` · 005a7690
- [ ] `MusicByteStream::QueryInterface` · 005a76c0
- [-] `MusicByteStream::AddRef` · 005a76d0 — inline copy of 005a75d0
- [ ] `MusicByteStream::Release` · 005a76e0 · legions 00573d60
- [ ] `MusicByteStream::~MusicByteStream` · 005a7720 · legions 00573da0
- [ ] `MusicByteStream::Seek` · 005a7750 · legions 00573dd0
- [-] `MusicByteStream::Release` · 005a77e0 — inline copy of 005a76e0
- [ ] `MusicByteStream::Clone` · 005a77f0 · legions 00573e70

### DirectXMusicLoader
- [ ] `DirectXMusicLoader::LoadFromObject` · 005a7850 · legions 00573ed0
- [ ] `DirectXMusicLoader::LoadFromObject` · 005a7a30 · legions 00574080
- [~] `DirectXMusicLoader::LoadFromObject` · 005a7ad0 · legions 00574100 — ac2-audio music/mod.rs (MusicObject::Wave), music_thread/playback.rs (Playback::wave), manager.rs: an EncodedWave is a DirectMusic segment with a wave track; it plays as a queue segment (primary by Track_ForcePrimary, stop at the MusicInfo's stop boundary on the default 120 bpm 4/4 clock, looping until stopped). Start timing flags not modelled; the file is decoded as RIFF WAVE (encoded.rs).
- [-] `DirectXMusicLoader::LoadFromSoundInfo` · 005a7b70 — text source loader, admin build only
- [~] `DirectXMusicLoader::LoadFromMusicInfo` · 005a7c20 · legions 005741e0 — ac2-audio playing.rs from_wave: an EncodedWave MusicInfo is a queue segment (QueuedWave), its length -1 (InitDXPSDFromMusicInfo: no length cap); terms as InitDXPSDFromMusicInfo; the wave voice starts on the queue's SegmentStart and ends on SegmentEnd (on_segment_started / on_segment_ended).
- [ ] `DirectXMusicLoader::~DirectXMusicLoader` · 005a7e90 · legions 00574420

### MusicByteStream
- [ ] `MusicByteStream::Write` · 005a7f60 · legions 005744f0
- [ ] `MusicByteStream::Commit` · 005a7f70 · legions 00574500

### DirectXMusicLoader
- [ ] `DirectXMusicLoader::LoadObjectFromFile` · 005a7f80 · legions 00574510
- [ ] `DirectXMusicLoader::SetObject` · 005a7f90 · legions 00574520
- [ ] `DirectXMusicLoader::GetObjectA` · 005a80e0 · legions 00574670
- [ ] `DirectXMusicLoader::Release` · 005a84b0 · legions 00574a10
- [ ] `DirectXMusicLoader::DirectXMusicLoader` · 005a8580 · legions 00574ae0

## `portal/client/src/sound/directx/directxplayingsound.cpp.c` (21/32)

### DirectXPlayingSound
- [x] `DirectXPlayingSound::OnSegmentStarted` · 005a6010 · legions 00572750 — ac2-audio: sets m_bHasStarted, which the admin decompile never reads: no effect ported (the start callback is a no-op)
- [~] `DirectXPlayingSound::Update3DInfo` · 005a6030 · legions 00572770 — ac2-client FX sounds: sound branch only (no particles/appearance/channel prefs, day assumed); DirectSound 3D pan law and doppler approximated
- [x] `DirectXPlayingSound::GetIsPlaying` · 005a6170 — ac2-audio: SoundManager / DirectXPlayingSound / sound records
- [ ] `DirectXPlayingSound::GetIsActuallyPlaying` · 005a61a0 · legions 005728e0
- [x] `DirectXPlayingSound::GetReadyToDie` · 005a61c0 — ac2-audio: SoundManager / DirectXPlayingSound / sound records
- [ ] `DirectXPlayingSound::GetVolume` · 005a61d0
- [~] `DirectXPlayingSound::SetFrequencyInternal` · 005a61e0 · legions 00572920 — ac2-audio: no listener/3D positioning or emitter muting, no MusicInfo (DirectMusic) playback, prefs not linked to PreferenceBank, cpal device instead of DirectSound

### CMotionInterp
- [ ] `CMotionInterp::GetVelocityScale` · 005a6260

### DirectXPlayingSound
- [x] `DirectXPlayingSound::GetIsLooping` · 005a6270 — ac2-audio: SoundManager / DirectXPlayingSound / sound records
- [x] `DirectXPlayingSound::ReleaseAudioPathAndBuffers` · 005a6280 · legions 005729c0 — ac2-audio: SoundManager / DirectXPlayingSound / sound records
- [x] `DirectXPlayingSound::DirectXPlayingSound` · 005a6320 · legions 00572a20 — ac2-audio: SoundManager / DirectXPlayingSound / sound records
- [x] `DirectXPlayingSound::GetIsSelfExclusive` · 005a6400 — ac2-audio: SoundManager / DirectXPlayingSound / sound records
- [x] `DirectXPlayingSound::GetIsAllExclusive` · 005a6420 — ac2-audio: SoundManager / DirectXPlayingSound / sound records
- [x] `DirectXPlayingSound::GetIsBudgeted` · 005a6430 — ac2-audio: SoundManager / DirectXPlayingSound / sound records
- [ ] `DirectXPlayingSound::GetEmitter` · 005a6450
- [ ] `DirectXPlayingSound::GetIsPrimaryTrack` · 005a6460
- [ ] `DirectXPlayingSound::~DirectXPlayingSound` · 005a6470 · legions 00572b80
- [x] `DirectXPlayingSound::DXInitialize` · 005a64d0 · legions 00572be0 — ac2-audio: SoundManager / DirectXPlayingSound / sound records
- [x] `DirectXPlayingSound::OnSegmentEnded` · 005a6600 · legions 00572d10 — ac2-audio: SoundManager / DirectXPlayingSound / sound records
- [x] `DirectXPlayingSound::UpdatePosition` · 005a6640 · legions 00572d50 — ac2-audio/ac2-client: 3D emitters and FX sounds
- [x] `DirectXPlayingSound::SetVolumeInternal` · 005a6790 · legions 00572ea0 — ac2-audio: SoundManager / DirectXPlayingSound / sound records
- [ ] `DirectXPlayingSound::SetFrequency` · 005a6920
- [ ] `DirectXPlayingSound::GetAudioPathAndBuffers` · 005a69b0 · legions 00573110
- [~] `DirectXPlayingSound::PlayInternal` · 005a6b50 · legions 005731e0 — ac2-audio: no listener/3D positioning or emitter muting, no MusicInfo (DirectMusic) playback, prefs not linked to PreferenceBank, cpal device instead of DirectSound
- [x] `DirectXPlayingSound::StopInternal` · 005a6d50 · legions 005733e0 — ac2-audio manager.rs stop_context: a fade keeps the segment in the queue (the queue stop waits for the fade's end, as this function returns after the ramp); verified by music_wave_tests::a_faded_primary_wave_plays_out_silently_and_the_next_takes_over.
- [x] `DirectXPlayingSound::SetVolume` · 005a6ee0 — ac2-audio/ac2-client: 3D emitters and FX sounds
- [x] `DirectXPlayingSound::RecalculateVolume` · 005a6f00 · legions 00573590 — ac2-audio: SoundManager / DirectXPlayingSound / sound records
- [x] `DirectXPlayingSound::UseTime` · 005a6f20 · legions 005735b0 — ac2-audio manager.rs flush_sounds: a queue segment the queue still holds when its sound goes ready-to-die (the fade end) is stopped with Boundary::End, as Stop(handle, 0) (plays out the current pass, drops a queued one); PlayingSound::use_time length cap is -1 for MusicInfo sounds (no cap).
- [x] `DirectXPlayingSound::Play` · 005a70a0 — ac2-audio: SoundManager / DirectXPlayingSound / sound records
- [x] `DirectXPlayingSound::Stop` · 005a70b0 — ac2-audio: SoundManager / DirectXPlayingSound / sound records
- [x] `DirectXPlayingSound::Enable` · 005a70e0 · legions 00573770 — ac2-audio: SoundManager / DirectXPlayingSound / sound records
- [x] `DirectXPlayingSound::OnDeviceStateChanged` · 005a7180 · legions 00573810 — ac2-audio: SoundManager / DirectXPlayingSound / sound records

## `portal/client/src/sound/directx/directxsounddevice.cpp.c` (5/34)

### DirectXSoundDevice
- [ ] `DirectXSoundDevice::SetSoundFeatures` · 005a3e70 · legions 00570640
- [x] `DirectXSoundDevice::OnSegmentStart` · 005a3eb0 — ac2-audio: music queue events reach the playing sounds (OnSegmentEnded), then each sound's UseTime; DirectXSoundDevice::OnSegmentStart/UseTime
- [x] `DirectXSoundDevice::OnSegmentEnded` · 005a3ee0 — ac2-audio: music queue events reach the playing sounds (OnSegmentEnded), then each sound's UseTime; DirectXSoundDevice::OnSegmentStart/UseTime
- [~] `DirectXSoundDevice::SetGrooveLevel` · 005a3f10 · legions 005706e0 — ac2-audio: the level is passed to MusicThread::set_groove_level as one byte (GUID_PerfMasterGrooveLevel); the device enable gate is not modelled

### DirectXSoundInstance
- [~] `DirectXSoundInstance::Initialize` · 005a3f70 · legions 00570740 — ac2-audio: no listener/3D positioning or emitter muting, no MusicInfo (DirectMusic) playback, prefs not linked to PreferenceBank, cpal device instead of DirectSound
- [~] `DirectXSoundInstance::Initialize` · 005a3ff0 · legions 005707c0 — ac2-audio: no listener/3D positioning or emitter muting, no MusicInfo (DirectMusic) playback, prefs not linked to PreferenceBank, cpal device instead of DirectSound
- [x] `DirectXSoundInstance::InitDXPSDFromMusicInfo` · 005a4070 · legions 00570840 — ac2-audio/ac2-client: ambient environments, groove
- [ ] `DirectXSoundInstance::GetDataID` · 005a4270 · legions 00570a40

### DirectXSoundDevice
- [~] `DirectXSoundDevice::SetChannelVolume` · 005a4290 · legions 00570a60 — ac2-audio: SoundManager / DirectXPlayingSound / sound records
- [~] `DirectXSoundDevice::UpdateListener` · 005a4440 · legions 00570c10 — ac2-audio: no listener/3D positioning or emitter muting, no MusicInfo (DirectMusic) playback, prefs not linked to PreferenceBank, cpal device instead of DirectSound
- [x] `DirectXSoundDevice::UseTime` · 005a45c0 · legions 00570d90 — ac2-audio: music queue events reach the playing sounds (OnSegmentEnded), then each sound's UseTime; DirectXSoundDevice::OnSegmentStart/UseTime
- [ ] `DirectXSoundDevice::RemovePlayingSound` · 005a4600 · legions 00570dd0
- [ ] `DirectXSoundDevice::RemoveSoundInstance` · 005a4650 · legions 00570e20

### DirectXSoundInstance
- [ ] `DirectXSoundInstance::~DirectXSoundInstance` · 005a46a0 · legions 00570e70
- [ ] `DirectXSoundInstance::OnDeviceStateChanged` · 005a4750 · legions 00570f20

### DirectXSoundDevice
- [ ] `DirectXSoundDevice::DebugPrintPlayingSounds` · 005a4810 · legions 00570fe0

### DirectXSoundInstance
- [x] `DirectXSoundInstance::InitDXPSDFromSoundInfo` · 005a4990 · legions 00571160 — ac2-audio: SoundManager / DirectXPlayingSound / sound records
- [~] `DirectXSoundInstance::CreatePlayingSound` · 005a4ae0 · legions 00571290 — ac2-audio: no listener/3D positioning or emitter muting, no MusicInfo (DirectMusic) playback, prefs not linked to PreferenceBank, cpal device instead of DirectSound

### DirectXSoundDevice
- [ ] `DirectXSoundDevice::AddPlayingSound` · 005a4ba0 · legions 00571350

### DirectXSoundInstance
- [ ] `DirectXSoundInstance::DirectXSoundInstance` · 005a4bf0 · legions 005713a0
- [ ] `DirectXSoundInstance::DownloadToAudioPath` · 005a4c60 · legions 00571410

### DirectXSoundDevice
- [ ] `DirectXSoundDevice::CreateInstance` · 005a4cb0 · legions 00571460
- [ ] `DirectXSoundDevice::CreateInstance` · 005a4d10 · legions 005714c0
- [ ] `DirectXSoundDevice::DestroyDevice` · 005a4e10 · legions 005715c0
- [~] `DirectXSoundDevice::CreateDevice` · 005a5160 · legions 00571910 — ac2-audio: no listener/3D positioning or emitter muting, no MusicInfo (DirectMusic) playback, prefs not linked to PreferenceBank, cpal device instead of DirectSound
- [~] `DirectXSoundDevice::Enable` · 005a52a0 · legions 00571a50 — ac2-audio: no listener/3D positioning or emitter muting, no MusicInfo (DirectMusic) playback, prefs not linked to PreferenceBank, cpal device instead of DirectSound
- [ ] `DirectXSoundDevice::Shutdown` · 005a53c0 · legions 00571b70
- [ ] `DirectXSoundDevice::ReserveAudioPath` · 005a5430 · legions 00571be0
- [~] `DirectXSoundDevice::Initialize` · 005a5500 · legions 00571cb0 — ac2-audio: no listener/3D positioning or emitter muting, no MusicInfo (DirectMusic) playback, prefs not linked to PreferenceBank, cpal device instead of DirectSound
- [ ] `DirectXSoundDevice::CreateAudioPaths` · 005a55a0 · legions 00571d50
- [ ] `DirectXSoundDevice::FreeAudioPath` · 005a5860 · legions 00571fa0
- [ ] `DirectXSoundDevice::DirectXSoundDevice` · 005a58d0 · legions 00572010
- [ ] `DirectXSoundDevice::OnDDDComplete` · 005a59f0 · legions 00572130

### SoundDeviceFactory
- [~] `SoundDeviceFactory::CreateDevice` · 005a5a80 · legions 005721c0 — ac2-audio: no listener/3D positioning or emitter muting, no MusicInfo (DirectMusic) playback, prefs not linked to PreferenceBank, cpal device instead of DirectSound

## `portal/client/src/sound/directx/dxmusicmessagethread.cpp.c` (8/16)

### DXMusicMessageThread::find_if_SegmentState
- [-] `DXMusicMessageThread::find_if_SegmentState::find_if_SegmentState` · 005a85b0 · legions 00574b10 — COM reference counting and C++ deleting destructor: Arc and Option<SegmentInfo> ownership in music_thread

### DXMusicMessageThread
- [-] `DXMusicMessageThread::ReleaseSegmentInfo` · 005a85d0 · legions 00574b30 — COM reference counting and C++ deleting destructor: Arc and Option<SegmentInfo> ownership in music_thread
- [~] `DXMusicMessageThread::PlayInternal` · 005a8600 · legions 00574b60 — ac2-audio music_thread: PlayInternal = PlaySegmentEx(segment, pAfter); the timing flags and audio path are not modelled (a secondary starts at once)
- [~] `DXMusicMessageThread::SetHandler` · 005a87d0 · legions 00574d30 — ac2-audio music_thread: no DirectMusic performance object; attached at MusicThread::new, detached at shutdown; the handler is passed to use_time
- [x] `DXMusicMessageThread::GetAssociatedSoundHandle` · 005a8950 · legions 00574eb0 — ac2-audio music_thread: DXMusicMessageThread queue (primary queue, handles, notifications, UseTime dispatch); see music_thread/mod.rs
- [x] `DXMusicMessageThread::IsPlaying` · 005a8a10 · legions 00574f70 — ac2-audio music_thread: DXMusicMessageThread queue (primary queue, handles, notifications, UseTime dispatch); see music_thread/mod.rs
- [x] `DXMusicMessageThread::AssociateSoundHandle` · 005a8ac0 · legions 00574ff0 — ac2-audio music_thread: DXMusicMessageThread queue (primary queue, handles, notifications, UseTime dispatch); see music_thread/mod.rs
- [x] `DXMusicMessageThread::PlayQueuedPrimarySegment` · 005a8b70 · legions 00575100 — ac2-audio music_thread: DXMusicMessageThread queue (primary queue, handles, notifications, UseTime dispatch); see music_thread/mod.rs
- [~] `DXMusicMessageThread::OnMusicStopping` · 005a8cb0 · legions 00575240 — ac2-audio music_thread/mod.rs on_music_stopping: retail's queue (next_info, replace, abort) as in retail. Port addition: a primary a script started (Segment.play with AtFinish, DMUS_SEGF_QUEUE) waits in the performance's own queue ahead of next_info (scripted_primaries); DirectMusic keeps it there, not in the AC2 decompile.
- [x] `DXMusicMessageThread::~DXMusicMessageThread` · 005a8dd0 · legions 00575410 — ac2-audio music_thread: DXMusicMessageThread queue (primary queue, handles, notifications, UseTime dispatch); see music_thread/mod.rs
- [x] `DXMusicMessageThread::UseTime` · 005a8ef0 · legions 00575530 — ac2-audio music_thread: DXMusicMessageThread queue (primary queue, handles, notifications, UseTime dispatch); see music_thread/mod.rs
- [x] `DXMusicMessageThread::DXMusicMessageThread` · 005a90c0 · legions 00575700 — ac2-audio music_thread: DXMusicMessageThread queue (primary queue, handles, notifications, UseTime dispatch); see music_thread/mod.rs
- [~] `DXMusicMessageThread::Startup` · 005a9170 — ac2-audio music_thread: Startup's notification loop runs inside MusicThread::use_time (no thread, no 100 ms wait; the WAIT_TIMEOUT watchdog is kept)
- [x] `DXMusicMessageThread::QueuePrimaryInternal` · 005a93b0 · legions 005759f0 — ac2-audio music_thread: DXMusicMessageThread queue (primary queue, handles, notifications, UseTime dispatch); see music_thread/mod.rs
- [~] `DXMusicMessageThread::Stop` · 005a9540 · legions 00575b80 — ac2-audio music_thread/mod.rs stop(): the current primary's stop only clears its loop (m_bLoopPrimary = 0; dwStopFlags unread; Legions 00575b80 the same), so its pass plays out to its end (playback.rs end_pass_after_stop; a looping wave ends at the end of its pass). A forced-immediate stop ends the voice at once: DirectXPlayingSound::StopInternal (005a6d50) calls Stop(handle, 0) then OnSegmentEnded. A secondary segment stops at once: StopEx's boundary flags are not modelled (gap).
- [~] `DXMusicMessageThread::Play` · 005a96f0 · legions 00575d30 — ac2-audio music_thread: Stop = StopEx at once (stop boundary flags not modelled); Play's audio path check and DMUS start flags not modelled

## `portal/client/src/sound/directx/envinfo_win32.cpp.c` (0/61)

### PFileNode
- [ ] `PFileNode::MakeSubNodeAsInt32` · 005a9780 · legions 00575dc0

### EnvInfo_WIN32
- [-] `EnvInfo_WIN32::Init` · 005a97e0 · legions 00575e20 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::CopyInto` · 005a9850 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::SetProperty` · 005a9950 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::InvalidateProperty` · 005a9cc0 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::GetProperty` · 005a9ce0 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::SetRoom` · 005a9e70 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::SetRoomHF` · 005a9e90 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::SetRoomLF` · 005a9eb0 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::SetRoomRolloffFactor` · 005a9ed0 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::SetDecayTime` · 005a9ef0 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::SetDecayHFRatio` · 005a9f10 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::SetReflections` · 005a9f30 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::SetReflectionsDelay` · 005a9f50 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::SetReflectionsPan` · 005a9f70 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::SetReverb` · 005a9f90 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::SetReverbDelay` · 005a9fb0 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::SetReverbPan` · 005a9fd0 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::SetEchoTime` · 005a9ff0 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::SetEchoDepth` · 005aa010 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::SetModulationTime` · 005aa030 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::SetModulationDepth` · 005aa050 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::SetEnvironment` · 005aa070 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::SetEnvironmentSize` · 005aa090 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::SetEnvironmentDiffusion` · 005aa0b0 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::SetAirAbsorptionHF` · 005aa0d0 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::SetHFReference` · 005aa0f0 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::SetLFReference` · 005aa110 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::SetFlags` · 005aa130 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::GetRoom` · 005aa150 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::GetRoomHF` · 005aa160 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::GetRoomLF` · 005aa170 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::GetRoomRolloffFactor` · 005aa180 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::GetDecayTime` · 005aa190 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::GetDecayHFRatio` · 005aa1a0 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::GetDecayLFRatio` · 005aa1b0 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::GetReflections` · 005aa1c0 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::GetReflectionsDelay` · 005aa1d0 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::GetReflectionsPan` · 005aa1e0 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::GetReverb` · 005aa210 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::GetReverbDelay` · 005aa220 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::GetReverbPan` · 005aa230 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::GetEchoTime` · 005aa260 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::GetEchoDepth` · 005aa270 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::GetModulationTime` · 005aa280 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::GetModulationDepth` · 005aa290 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::GetEnvironment` · 005aa2a0 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::GetEnvironmentSize` · 005aa2b0 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::GetEnvironmentDiffusion` · 005aa2c0 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::GetAirAbsorptionHF` · 005aa2d0 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::GetHFReference` · 005aa2e0 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::GetLFReference` · 005aa2f0 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::GetFlags` · 005aa300 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::EnvInfo_WIN32` · 005aa310 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::Allocate` · 005aa340 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.

### EnvInfo
- [-] `EnvInfo::GetDBOType` · 005aa350 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.

### EnvInfo_WIN32
- [-] `EnvInfo_WIN32::GetBitMask` · 005aa360 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::SetBitMask` · 005aa370 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::StreamPack` · 005aa3b0 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::FromFileNode` · 005aa720 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo_WIN32::ToFileNode` · 005aae80 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.

## `portal/client/src/sound/directx/soundfactory_win32.cpp.c` (0/2)

### SoundFactory_WIN32
- [ ] `SoundFactory_WIN32::MakeEncodedWave` · 005a5ac0
- [-] `SoundFactory_WIN32::MakeEnvInfo` · 005a5ae0 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.

## `portal/client/src/sound/directx/wavefile.cpp.c` (0/5)

### std::strstreambuf
- [ ] `std::strstreambuf::str` · 005a5b30 · legions 00572270

### WaveFile
- [ ] `WaveFile::WaveFile` · 005a5b50 · legions 00572290

### WaveFileStreamUtilities
- [ ] `WaveFileStreamUtilities::WriteWaveHeaderChunk` · 005a5b70 · legions 005722b0
- [ ] `WaveFileStreamUtilities::ReadChunk` · 005a5c40 · legions 00572380

### WaveFile
- [ ] `WaveFile::Load` · 005a5db0

## `portal/client/src/sound/encodedwave.cpp.c` (0/6)

### EncodedWave
- [ ] `EncodedWave::Pack` · 005a32b0
- [ ] `EncodedWave::Allocator` · 005a3330
- [ ] `EncodedWave::EncodedWave` · 005a3340
- [ ] `EncodedWave::GetDBOType` · 005a3370
- [ ] `EncodedWave::UnPack` · 005a3380
- [-] `EncodedWave::LoadFromFile` · 005a3420 — text source loader, admin build only

## `portal/client/src/sound/envinfo.cpp.c` (0/3)

### EnvInfo
- [-] `EnvInfo::EnvInfo` · 005a3500 · legions 00570000 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo::~EnvInfo` · 005a3530 · legions 00570030 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.
- [-] `EnvInfo::Allocator` · 005a3540 — EnvInfo (EAX listener properties, PFileNode-loaded) is never applied in retail: no caller of the property accessors in admin 7148 or Legions 7160 (the only sound-side hand-off is ChangeCellEnd's 'return 1' stub); DXSoundDevice::UpdateListener sets DS3D listener data only. Nothing to port.

## `portal/client/src/sound/soundmanager.cpp.c` (15/25)

### SoundManager
- [~] `SoundManager::Initialize` · 005a1000 · legions 0056dcd0 — ac2-audio: no listener/3D positioning or emitter muting, no MusicInfo (DirectMusic) playback, prefs not linked to PreferenceBank, cpal device instead of DirectSound
- [ ] `SoundManager::OnDDDComplete` · 005a10d0

### PlaySoundData
- [x] `PlaySoundData::PlaySoundData` · 005a1160 · legions 0056de30 — ac2-audio: SoundManager / DirectXPlayingSound / sound records

### SoundManager
- [ ] `SoundManager::DebugPrintPlayingSounds` · 005a1190
- [x] `SoundManager::SetGrooveLevel` · 005a11a0 · legions 0056de60 — ac2-audio: SetGrooveLevel/TweakGrooveLevel forward level+offset to MusicThread::set_groove_level (StylePerformance groove) and the sink
- [x] `SoundManager::TweakGrooveLevel` · 005a11d0 — ac2-audio: SetGrooveLevel/TweakGrooveLevel forward level+offset to MusicThread::set_groove_level (StylePerformance groove) and the sink
- [~] `SoundManager::InitPrefs` · 005a1210 · legions 0056de90 — ac2-audio: no listener/3D positioning or emitter muting, no MusicInfo (DirectMusic) playback, prefs not linked to PreferenceBank, cpal device instead of DirectSound
- [x] `SoundManager::AllocateSounds` · 005a1720 · legions 0056e3e0 — ac2-audio: SoundManager / DirectXPlayingSound / sound records
- [x] `SoundManager::CheckSoundBudget` · 005a1830 · legions 0056e4d0 — ac2-audio: SoundManager / DirectXPlayingSound / sound records
- [x] `SoundManager::StopSounds` · 005a1a70 · legions 0056e690 — ac2-audio: SoundManager / DirectXPlayingSound / sound records
- [x] `SoundManager::StopSoundsForEmitter` · 005a1ba0 · legions 0056e7c0 — ac2-audio: SoundManager / DirectXPlayingSound / sound records
- [x] `SoundManager::HasSoundFinished` · 005a1ca0 — ac2-audio: SoundManager / DirectXPlayingSound / sound records
- [x] `SoundManager::StopSoundRequest` · 005a1d10 — ac2-audio: SoundManager / DirectXPlayingSound / sound records
- [~] `SoundManager::HandleChangeInternal` · 005a1d90 · legions 0056e9b0 — ac2-audio: no listener/3D positioning or emitter muting, no MusicInfo (DirectMusic) playback, prefs not linked to PreferenceBank, cpal device instead of DirectSound
- [~] `SoundManager::HandleApply` · 005a1fe0 — ac2-audio: no listener/3D positioning or emitter muting, no MusicInfo (DirectMusic) playback, prefs not linked to PreferenceBank, cpal device instead of DirectSound
- [~] `SoundManager::HandleChange` · 005a2120 — ac2-audio: no listener/3D positioning or emitter muting, no MusicInfo (DirectMusic) playback, prefs not linked to PreferenceBank, cpal device instead of DirectSound
- [ ] `SoundManager::GetPlayingSoundFromHandle` · 005a2230
- [x] `SoundManager::SoundManager` · 005a2680 · legions 0056f370 — ac2-audio: SoundManager / DirectXPlayingSound / sound records
- [x] `SoundManager::FlushSounds` · 005a27d0 · legions 0056f4c0 — ac2-audio: SoundManager / DirectXPlayingSound / sound records
- [x] `SoundManager::AddSoundInstance` · 005a28d0 · legions 0056f5c0 — ac2-audio: SoundManager / DirectXPlayingSound / sound records
- [x] `SoundManager::Shutdown` · 005a29f0 · legions 0056f6e0 — ac2-audio: SoundManager / DirectXPlayingSound / sound records
- [~] `SoundManager::UseTime` · 005a2b00 · legions 0056f7f0 — ac2-audio: the music queue runs before the voices in UseTime; a music voice's PlayInternal starts its segment in the queue; after the voices, the queue's notes go to the mixer's synthesizers (MusicClock + Note); no listener/3D or emitter muting
- [~] `SoundManager::PlayInternal` · 005a2c50 · legions 0056f8f0 — ac2-audio: the music queue runs before the voices in UseTime; a music voice's PlayInternal starts its segment in the queue; no listener/3D or emitter muting
- [x] `SoundManager::OnContextFinished` · 005a3050 — ac2-audio: SoundManager / DirectXPlayingSound / sound records
- [x] `SoundManager::Play` · 005a30c0 · legions 0056fc40 — ac2-audio: SoundManager / DirectXPlayingSound / sound records

## `portal/shared/src/sound/musicinfo.cpp.c` (4/10)

### MusicInfo
- [x] `MusicInfo::MusicInfo` · 006b8a70 · legions 0064fa30 — ac2-audio: SoundManager / DirectXPlayingSound / sound records
- [x] `MusicInfo::GetDBOType` · 006b8aa0 — ac2-audio: SoundManager / DirectXPlayingSound / sound records
- [ ] `MusicInfo::GetSubDataIDs` · 006b8ab0
- [ ] `MusicInfo::CopyInto` · 006b8ad0
- [x] `MusicInfo::TestBitMask` · 006b8b00 · legions 0064fac0 — ac2-audio: SoundManager / DirectXPlayingSound / sound records
- [ ] `MusicInfo::Allocate` · 006b8b10
- [x] `MusicInfo::StreamPack` · 006b8b20 · legions 0064fae0 — ac2-audio: SoundManager / DirectXPlayingSound / sound records
- [-] `MusicInfo::ToFileNode` · 006b8c00 — text source format (admin tools)
- [ ] `MusicInfo::ParseFlags` · 006b8fb0
- [-] `MusicInfo::FromFileNode` · 006b9300 — text source format (admin tools)

## `portal/shared/src/sound/soundinfo.cpp.c` (3/9)

### SoundInfo
- [x] `SoundInfo::SoundInfo` · 006b8090 · legions 0064f890 — ac2-audio: SoundManager / DirectXPlayingSound / sound records
- [x] `SoundInfo::GetDBOType` · 006b80d0 — ac2-audio: SoundManager / DirectXPlayingSound / sound records
- [ ] `SoundInfo::GetSubDataIDs` · 006b80e0
- [ ] `SoundInfo::CopyInto` · 006b8100
- [ ] `SoundInfo::Allocate` · 006b8130
- [x] `SoundInfo::StreamPack` · 006b8140 · legions 0064f940 — ac2-audio: SoundManager / DirectXPlayingSound / sound records
- [-] `SoundInfo::ToFileNode` · 006b8230 — text source format (admin tools)
- [ ] `SoundInfo::ParseFlags` · 006b84d0
- [-] `SoundInfo::FromFileNode` · 006b8670 — text source format (admin tools)
