# ac2-script: WSL interpreter and native glue

2468 retail functions: 1014 done, 71 partial, 1305 to do, 78 skipped.
Legend and workflow: [README](README.md).

## `ac2/shared/src/interp/glue/gcharactergensystem.cpp.c` (0/25)

### Position
- [ ] `Position::operator=` · 0046ad40 · legions 0046b420

### PHashTable<unsigned_long,unsigned_long>
- [ ] `PHashTable<unsigned_long,unsigned_long>::~PHashTable<unsigned_long,unsigned_long>` · 0046adc0 · legions 0046b440

### gCharacterGenSystem
- [ ] `gCharacterGenSystem::ARHashToRaceSexInfoHash` · 0046ae10 · legions 0046b570
- [ ] `gCharacterGenSystem::RaceSexInfoHashToARHash` · 0046af20 · legions 0046b680
- [ ] `gCharacterGenSystem::RaceSexListHashToARHash` · 0046b090 · legions 0046b7f0

### ARHash
- [ ] `ARHash::ARHash` · 0046b250 · legions 0046b9b0
- [ ] `ARHash::GetReferences` · 0046b290

### BaseHash<unsigned_long,unsigned_long,18>
- [ ] `BaseHash<unsigned_long,unsigned_long,18>::AddDataAsReferences` · 0046b2b0

### ARHash
- [ ] `ARHash::UpdateReferences` · 0046b310

### BaseHash<unsigned_long,unsigned_long,18>
- [ ] `BaseHash<unsigned_long,unsigned_long,18>::UpdateData` · 0046b330

### gCharacterGenSystem
- [ ] `gCharacterGenSystem::ARHashToRaceSexListHash` · 0046b3f0 · legions 0046bb50
- [ ] `gCharacterGenSystem::CallIFun_FormatNameString` · 0046b480 · legions 0046bbe0
- [ ] `gCharacterGenSystem::CallIFun_GetPhysObj` · 0046b6d0 · legions 0046be30
- [ ] `gCharacterGenSystem::CallIFun_InqAdminAvatarEntityDID` · 0046b800 · legions 0046bf60
- [ ] `gCharacterGenSystem::CallIFun_InqAppliedAppearanceVDesc` · 0046b960 · legions 0046c0c0
- [ ] `gCharacterGenSystem::CallIFun_InqPhysTypeLabel` · 0046bb90 · legions 0046c2f0
- [ ] `gCharacterGenSystem::CallIFun_InqPlayerAvatarEntityDID` · 0046bdb0 · legions 0046c510
- [ ] `gCharacterGenSystem::CallIFun_InqRaceSexData` · 0046bf10 · legions 0046c670
- [ ] `gCharacterGenSystem::CallIFun_InqRaceSexInfoData` · 0046c100 · legions 0046c860
- [ ] `gCharacterGenSystem::CallIFun_InqStartingPosition` · 0046c2f0 · legions 0046ca50
- [ ] `gCharacterGenSystem::CallIFun_InqValidPhysTypeValues` · 0046c500 · legions 0046cc60
- [ ] `gCharacterGenSystem::CallIFun_InqValidStartingAreas` · 0046c750 · legions 0046ceb0
- [ ] `gCharacterGenSystem::CallIFun_IsValid` · 0046c980 · legions 0046d0e0
- [ ] `gCharacterGenSystem::CallIFun_RandomizeCGR` · 0046cac0 · legions 0046d220
- [ ] `gCharacterGenSystem::CallIFun_SetCGRDefaults` · 0046cca0 · legions 0046d400

## `ac2/shared/src/interp/glue/gcraftskilladaptor.cpp.c` (0/16)

### gCraftSkillAdaptor
- [ ] `gCraftSkillAdaptor::CallEFun_SetCostToRaise` · 0046e320
- [ ] `gCraftSkillAdaptor::CallEFun_SetDID` · 0046e380
- [ ] `gCraftSkillAdaptor::CallEFun_SetDescription` · 0046e3e0
- [ ] `gCraftSkillAdaptor::CallEFun_SetIconDID` · 0046e440
- [ ] `gCraftSkillAdaptor::CallEFun_SetLevel` · 0046e4a0
- [ ] `gCraftSkillAdaptor::CallEFun_SetName` · 0046e500
- [ ] `gCraftSkillAdaptor::CallEFun_SetRawLevel` · 0046e560
- [ ] `gCraftSkillAdaptor::CallEFun_SetUsageXPMod` · 0046e5c0
- [ ] `gCraftSkillAdaptor::CallEFun_GetCostToRaise` · 0046e620
- [ ] `gCraftSkillAdaptor::CallEFun_GetDID` · 0046e690
- [ ] `gCraftSkillAdaptor::CallEFun_GetIconDID` · 0046e700
- [ ] `gCraftSkillAdaptor::CallEFun_GetLevel` · 0046e770
- [ ] `gCraftSkillAdaptor::CallEFun_GetRawLevel` · 0046e7e0
- [ ] `gCraftSkillAdaptor::CallEFun_GetUsageXPMod` · 0046e850
- [ ] `gCraftSkillAdaptor::CallEFun_GetDescription` · 0046e8c0
- [ ] `gCraftSkillAdaptor::CallEFun_GetName` · 0046e970

## `ac2/shared/src/interp/glue/gdamagetextblob.cpp.c` (0/9)

### gDamageTextBlob
- [ ] `gDamageTextBlob::CallEFun_SetDuration` · 00478bb0
- [ ] `gDamageTextBlob::CallEFun_SetFontColor` · 00478c10
- [ ] `gDamageTextBlob::CallEFun_SetScale` · 00478c80

### gmCharGenResult
- [ ] `gmCharGenResult::SetName` · 00478ce0 · legions 005c2260

### gDamageTextBlob
- [ ] `gDamageTextBlob::CallEFun_GetDuration` · 00478d30
- [ ] `gDamageTextBlob::CallEFun_GetScale` · 00478da0
- [ ] `gDamageTextBlob::CallEFun_SetDisplayString` · 00478e10
- [ ] `gDamageTextBlob::CallEFun_GetDisplayString` · 00478e70
- [ ] `gDamageTextBlob::CallEFun_GetFontColor` · 00478f30

## `ac2/shared/src/interp/glue/geffectuinode.cpp.c` (0/11)

### gEffectUINode
- [ ] `gEffectUINode::CallEFun_Set` · 004787a0
- [ ] `gEffectUINode::CallEFun_SetDisabled` · 00478800
- [ ] `gEffectUINode::CallEFun_SetDurationInfo` · 00478860
- [ ] `gEffectUINode::CallEFun_SetEID` · 004788c0
- [ ] `gEffectUINode::CallEFun_SetEnabled` · 00478920
- [ ] `gEffectUINode::CallEFun_SetIcon` · 00478980
- [ ] `gEffectUINode::CallEFun_SetPulsed` · 004789e0
- [ ] `gEffectUINode::CallEFun_SetToggled` · 00478a40

### EffectUINode
- [ ] `EffectUINode::SetName` · 00478aa0 · legions 00474b30

### gEffectUINode
- [ ] `gEffectUINode::CallEFun_SetDescription` · 00478af0
- [ ] `gEffectUINode::CallEFun_SetName` · 00478b50

## `ac2/shared/src/interp/glue/gexaminationprofile.cpp.c` (0/17)

### gExaminationProfile
- [ ] `gExaminationProfile::CallEFun_AddBool` · 00478050
- [ ] `gExaminationProfile::CallEFun_AddBreak` · 004780b0
- [ ] `gExaminationProfile::CallEFun_AddCountdown` · 00478110
- [ ] `gExaminationProfile::CallEFun_AddEffectID` · 00478180
- [ ] `gExaminationProfile::CallEFun_AddFloat` · 004781e0
- [ ] `gExaminationProfile::CallEFun_AddIconDesc` · 00478250
- [ ] `gExaminationProfile::CallEFun_AddImage` · 004782c0
- [ ] `gExaminationProfile::CallEFun_AddInt` · 00478320
- [ ] `gExaminationProfile::CallEFun_AddLongInt` · 00478380
- [ ] `gExaminationProfile::CallEFun_AddSimpleStringInfo` · 004783f0
- [ ] `gExaminationProfile::CallEFun_AddStringInfo` · 00478460
- [ ] `gExaminationProfile::CallEFun_AddTab` · 004784d0
- [ ] `gExaminationProfile::CallEFun_AddTabs` · 00478530
- [ ] `gExaminationProfile::CallEFun_SetExaminationRequest` · 00478590
- [ ] `gExaminationProfile::CallEFun_eq` · 004785f0
- [ ] `gExaminationProfile::CallEFun_ne` · 00478670
- [ ] `gExaminationProfile::CallEFun_GetExaminationRequest` · 004786f0

## `ac2/shared/src/interp/glue/gexaminationrequest.cpp.c` (0/12)

### gExaminationRequest
- [ ] `gExaminationRequest::CallEFun_SetAdmin` · 00477b40
- [ ] `gExaminationRequest::CallEFun_SetDataEnum` · 00477ba0
- [ ] `gExaminationRequest::CallEFun_SetDataID` · 00477c00
- [ ] `gExaminationRequest::CallEFun_SetDataIID` · 00477c60
- [ ] `gExaminationRequest::CallEFun_SetType` · 00477cc0
- [ ] `gExaminationRequest::CallEFun_GetDataEnum` · 00477d20
- [ ] `gExaminationRequest::CallEFun_GetDataID` · 00477d90
- [ ] `gExaminationRequest::CallEFun_GetDataIID` · 00477e00
- [ ] `gExaminationRequest::CallEFun_GetType` · 00477e70
- [ ] `gExaminationRequest::CallEFun_IsAdmin` · 00477ee0
- [ ] `gExaminationRequest::CallEFun_eq` · 00477f50
- [ ] `gExaminationRequest::CallEFun_ne` · 00477fd0

## `ac2/shared/src/interp/glue/ggameplayoptionsprofile.cpp.c` (1/83)

### GameplayOptionsProfile
- [ ] `GameplayOptionsProfile::SetShowDamageTextOther` · 00474770 · legions 00471870

### ReferenceIDWrapper
- [x] `ReferenceIDWrapper::StreamPack` · 00474780 · legions 00471880 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

### gGameplayOptionsProfile
- [ ] `gGameplayOptionsProfile::CallEFun_AddRadarMask` · 004747f0
- [ ] `gGameplayOptionsProfile::CallEFun_FlushAliasTable` · 00474850
- [ ] `gGameplayOptionsProfile::CallEFun_RemoveRadarMask` · 004748b0
- [ ] `gGameplayOptionsProfile::CallEFun_SetAcceptAllegianceRequest` · 00474910
- [ ] `gGameplayOptionsProfile::CallEFun_SetAcceptFellowshipRequest` · 00474970
- [ ] `gGameplayOptionsProfile::CallEFun_SetAcceptTradeRequest` · 004749d0
- [ ] `gGameplayOptionsProfile::CallEFun_SetAutoTarget` · 00474a30
- [ ] `gGameplayOptionsProfile::CallEFun_SetBitfield` · 00474a90
- [ ] `gGameplayOptionsProfile::CallEFun_SetBypassGoldMeltingConfirmationDlg` · 00474af0
- [ ] `gGameplayOptionsProfile::CallEFun_SetDamageTextRangeOther` · 00474b50
- [ ] `gGameplayOptionsProfile::CallEFun_SetLatencyIndicatorGraph` · 00474bb0
- [ ] `gGameplayOptionsProfile::CallEFun_SetRadarMask` · 00474c10
- [ ] `gGameplayOptionsProfile::CallEFun_SetShortcutHeight` · 00474c70
- [ ] `gGameplayOptionsProfile::CallEFun_SetShortcutInstaScroll` · 00474cd0
- [ ] `gGameplayOptionsProfile::CallEFun_SetShortcutLocked` · 00474d30
- [ ] `gGameplayOptionsProfile::CallEFun_SetShowDamageText` · 00474d90
- [ ] `gGameplayOptionsProfile::CallEFun_SetShowDamageTextOther` · 00474df0
- [ ] `gGameplayOptionsProfile::CallEFun_SetShowFellowsOnMap` · 00474e50
- [ ] `gGameplayOptionsProfile::CallEFun_SetShowObjectNames` · 00474ec0
- [ ] `gGameplayOptionsProfile::CallEFun_SetShowPKDamage` · 00474f20
- [ ] `gGameplayOptionsProfile::CallEFun_SetShowPetUI` · 00474f80
- [ ] `gGameplayOptionsProfile::CallEFun_SetShowRadarCoords` · 00474ff0
- [ ] `gGameplayOptionsProfile::CallEFun_SetShowShortcutNumbers` · 00475050
- [ ] `gGameplayOptionsProfile::CallEFun_SetUISaveLocations` · 004750b0
- [ ] `gGameplayOptionsProfile::CallEFun_SetWhichShortcutSet` · 00475110
- [ ] `gGameplayOptionsProfile::CallEFun_AddFilter` · 00475220
- [ ] `gGameplayOptionsProfile::CallEFun_Export` · 004752a0
- [ ] `gGameplayOptionsProfile::CallEFun_GetAcceptAllegianceRequest` · 00475320
- [ ] `gGameplayOptionsProfile::CallEFun_GetAcceptFellowshipRequest` · 00475390
- [ ] `gGameplayOptionsProfile::CallEFun_GetAcceptTradeRequest` · 00475410
- [ ] `gGameplayOptionsProfile::CallEFun_GetAutoTarget` · 00475490
- [ ] `gGameplayOptionsProfile::CallEFun_GetBitfield` · 00475500
- [ ] `gGameplayOptionsProfile::CallEFun_GetBypassGoldMeltingConfirmationDlg` · 00475570
- [ ] `gGameplayOptionsProfile::CallEFun_GetChatFontColor` · 004755f0
- [ ] `gGameplayOptionsProfile::CallEFun_GetChatFontSize` · 00475670
- [ ] `gGameplayOptionsProfile::CallEFun_GetChatPopupFlag` · 004756f0
- [ ] `gGameplayOptionsProfile::CallEFun_GetDamageTextRangeOther` · 00475770
- [ ] `gGameplayOptionsProfile::CallEFun_GetFilter` · 004757e0
- [ ] `gGameplayOptionsProfile::CallEFun_GetLatencyIndicatorGraph` · 00475860
- [ ] `gGameplayOptionsProfile::CallEFun_GetRadarMask` · 004758e0
- [ ] `gGameplayOptionsProfile::CallEFun_GetShortcutHeight` · 00475950
- [ ] `gGameplayOptionsProfile::CallEFun_GetShortcutInstaScroll` · 004759c0
- [ ] `gGameplayOptionsProfile::CallEFun_GetShortcutLocked` · 00475a40
- [ ] `gGameplayOptionsProfile::CallEFun_GetShowDamageText` · 00475ac0
- [ ] `gGameplayOptionsProfile::CallEFun_GetShowDamageTextOther` · 00475b30
- [ ] `gGameplayOptionsProfile::CallEFun_GetShowFellowsOnMap` · 00475ba0
- [ ] `gGameplayOptionsProfile::CallEFun_GetShowObjectNames` · 00475c20
- [ ] `gGameplayOptionsProfile::CallEFun_GetShowPKDamage` · 00475ca0
- [ ] `gGameplayOptionsProfile::CallEFun_GetShowPetUI` · 00475d20
- [ ] `gGameplayOptionsProfile::CallEFun_GetShowRadarCoords` · 00475da0
- [ ] `gGameplayOptionsProfile::CallEFun_GetShowShortcutNumbers` · 00475e10
- [ ] `gGameplayOptionsProfile::CallEFun_GetWhichShortcutSet` · 00475e80
- [ ] `gGameplayOptionsProfile::CallEFun_Import` · 00475ef0
- [ ] `gGameplayOptionsProfile::CallEFun_InitShortcutArray` · 00475f70
- [ ] `gGameplayOptionsProfile::CallEFun_RemoveAlias` · 00475ff0
- [ ] `gGameplayOptionsProfile::CallEFun_RemoveFilter` · 00476070
- [ ] `gGameplayOptionsProfile::CallEFun_SetAlias` · 004760f0
- [ ] `gGameplayOptionsProfile::CallEFun_SetChatFontColor` · 00476180
- [ ] `gGameplayOptionsProfile::CallEFun_SetChatFontSize` · 00476210
- [ ] `gGameplayOptionsProfile::CallEFun_SetChatPopupFlag` · 004762a0
- [ ] `gGameplayOptionsProfile::CallEFun_SetFilter` · 00476330
- [ ] `gGameplayOptionsProfile::CallEFun_SetShortcut` · 004763b0
- [ ] `gGameplayOptionsProfile::CallEFun_SetWindowOpacity` · 00476430

### ggmCWInterface
- [ ] `ggmCWInterface::RArrayToShortcutInfoArray` · 004766c0 · legions 00472db0

### gGameplayOptionsProfile
- [ ] `gGameplayOptionsProfile::CallEFun_SetShortcutArray` · 00476920
- [ ] `gGameplayOptionsProfile::AAHashToChatPopupFlags` · 00476a00 · legions 004730d0
- [ ] `gGameplayOptionsProfile::NRHashToStrStrHash` · 00476af0 · legions 004731c0
- [ ] `gGameplayOptionsProfile::ChatPopupFlagsToAAHash` · 00476b80 · legions 00473250
- [ ] `gGameplayOptionsProfile::CallEFun_BuildAliasList` · 00476c70 · legions 00473340
- [ ] `gGameplayOptionsProfile::StrStrHashToNRHash` · 00476d30 · legions 004733d0
- [ ] `gGameplayOptionsProfile::CallEFun_SetAliasTable` · 00476e40
- [ ] `gGameplayOptionsProfile::CallEFun_SetWindowOpacities` · 00476f20
- [ ] `gGameplayOptionsProfile::CallEFun_SetChatOptions` · 00477000
- [ ] `gGameplayOptionsProfile::CallEFun_GetChatOptions` · 00477280
- [ ] `gGameplayOptionsProfile::CallEFun_GetUISaveLocations` · 004775b0
- [ ] `gGameplayOptionsProfile::CallEFun_GetWindowOpacities` · 00477630
- [ ] `gGameplayOptionsProfile::CallEFun_InqAlias` · 00477730 · legions 00473ea0
- [ ] `gGameplayOptionsProfile::CallEFun_InqAliasTable` · 004777d0
- [ ] `gGameplayOptionsProfile::CallEFun_InqShortcut` · 004778f0 · legions 00474010
- [ ] `gGameplayOptionsProfile::CallEFun_InqShortcutArray` · 00477990
- [ ] `gGameplayOptionsProfile::CallEFun_InqUISaveLocations` · 00477aa0

## `ac2/shared/src/interp/glue/ggmactinfo.cpp.c` (0/4)

### ggmActInfo
- [ ] `ggmActInfo::CallEFun_Init` · 00474530
- [ ] `ggmActInfo::CallEFun_AddScene` · 004745c0
- [ ] `ggmActInfo::CallEFun_GetActID` · 00474640
- [ ] `ggmActInfo::CallEFun_DumpW` · 004746b0

## `ac2/shared/src/interp/glue/ggmactinfolist.cpp.c` (0/2)

### ggmActInfoList
- [ ] `ggmActInfoList::CallEFun_pop_front` · 004743f0
- [ ] `ggmActInfoList::CallEFun_insert_sort` · 004744b0

## `ac2/shared/src/interp/glue/ggmchargenresult.cpp.c` (0/21)

### ggmCharGenResult
- [ ] `ggmCharGenResult::CallEFun_Clear` · 00473990
- [ ] `ggmCharGenResult::CallEFun_SetAdmin` · 004739f0
- [ ] `ggmCharGenResult::CallEFun_SetName` · 00473a50
- [ ] `ggmCharGenResult::CallEFun_SetRace` · 00473ab0
- [ ] `ggmCharGenResult::CallEFun_SetSex` · 00473b10
- [ ] `ggmCharGenResult::CallEFun_SetStartArea` · 00473b70
- [ ] `ggmCharGenResult::CallEFun_GetClassType` · 00473bd0
- [ ] `ggmCharGenResult::CallEFun_GetImplement` · 00473c40
- [ ] `ggmCharGenResult::CallEFun_GetPhysTypeValue` · 00473cb0
- [ ] `ggmCharGenResult::CallEFun_GetRace` · 00473d30
- [ ] `ggmCharGenResult::CallEFun_GetSex` · 00473da0
- [ ] `ggmCharGenResult::CallEFun_GetStartArea` · 00473e10
- [ ] `ggmCharGenResult::CallEFun_IsAdmin` · 00473e80
- [ ] `ggmCharGenResult::CallEFun_SanityCheck` · 00473ef0
- [ ] `ggmCharGenResult::CallEFun_SetClassType` · 00473f60
- [ ] `ggmCharGenResult::CallEFun_SetImplement` · 00473fe0
- [ ] `ggmCharGenResult::CallEFun_SetPhysTypeValue` · 00474060
- [ ] `ggmCharGenResult::CallEFun_SetStartingPosition` · 004740e0
- [ ] `ggmCharGenResult::CallEFun_VerifyCharGenResult` · 00474180
- [ ] `ggmCharGenResult::CallEFun_GetName` · 004741f0
- [ ] `ggmCharGenResult::CallEFun_GetStartingPosition` · 004742c0

## `ac2/shared/src/interp/glue/ggmkeyframe.cpp.c` (0/21)

### ggmKeyframe
- [ ] `ggmKeyframe::CallEFun_Clear` · 00473030
- [ ] `ggmKeyframe::CallEFun_SetAudioTrack` · 00473090
- [ ] `ggmKeyframe::CallEFun_SetAudioTrackByName` · 004730f0
- [ ] `ggmKeyframe::CallEFun_SetDuration` · 00473150
- [ ] `ggmKeyframe::CallEFun_SetFrameNum` · 004731b0
- [ ] `ggmKeyframe::CallEFun_SetMovieTrack` · 00473210
- [ ] `ggmKeyframe::CallEFun_SetSceneID` · 00473270
- [ ] `ggmKeyframe::CallEFun_SetSubtitle` · 004732d0
- [ ] `ggmKeyframe::CallEFun_SetVideoTrack` · 00473330
- [ ] `ggmKeyframe::CallEFun_SetVideoTrackByName` · 00473390
- [ ] `ggmKeyframe::CallEFun_SetVoiceTrack` · 004733f0
- [ ] `ggmKeyframe::CallEFun_SetVoiceTrackByName` · 00473450
- [ ] `ggmKeyframe::CallEFun_GetAudioTrack` · 004734b0
- [ ] `ggmKeyframe::CallEFun_GetDuration` · 00473520
- [ ] `ggmKeyframe::CallEFun_GetFrameNum` · 00473590
- [ ] `ggmKeyframe::CallEFun_GetSceneID` · 00473600
- [ ] `ggmKeyframe::CallEFun_GetVideoTrack` · 00473670
- [ ] `ggmKeyframe::CallEFun_GetVoiceTrack` · 004736e0
- [ ] `ggmKeyframe::CallEFun_Dump` · 00473750
- [ ] `ggmKeyframe::CallEFun_GetMovieTrack` · 00473810
- [ ] `ggmKeyframe::CallEFun_GetSubtitle` · 004738e0

## `ac2/shared/src/interp/glue/ggmquestinfo.cpp.c` (0/8)

### ggmQuestInfo
- [ ] `ggmQuestInfo::CallEFun_Init` · 00472b50
- [ ] `ggmQuestInfo::CallEFun_GetQuestID` · 00472c10
- [ ] `ggmQuestInfo::CallEFun_GetStatus` · 00472c80
- [ ] `ggmQuestInfo::CallEFun_PlaysFXOnUpdate` · 00472cf0
- [ ] `ggmQuestInfo::CallEFun_DumpW` · 00472d60
- [ ] `ggmQuestInfo::CallEFun_GetCurrentJournalEntry` · 00472e20
- [ ] `ggmQuestInfo::CallEFun_GetQuestDescription` · 00472ed0
- [ ] `ggmQuestInfo::CallEFun_GetQuestName` · 00472f80

## `ac2/shared/src/interp/glue/ggmquestinfolist.cpp.c` (0/7)

### ggmQuestInfoList
- [ ] `ggmQuestInfoList::CallEFun_Erase` · 004726d0
- [ ] `ggmQuestInfoList::CallEFun_num_elements` · 00472750
- [ ] `ggmQuestInfoList::CallEFun_push_tail` · 00472820
- [ ] `ggmQuestInfoList::CallEFun_DumpW` · 004728a0
- [ ] `ggmQuestInfoList::CallEFun_Find` · 00472960 · legions 00470640
- [ ] `ggmQuestInfoList::CallEFun_UpdateQuestInfo` · 00472a00 · legions 004706c0
- [ ] `ggmQuestInfoList::CallEFun_pop_head` · 00472ab0

## `ac2/shared/src/interp/glue/ggmracesexinfo.cpp.c` (0/9)

### ggmRaceSexInfo
- [ ] `ggmRaceSexInfo::CallEFun_SetMusic` · 00472260
- [ ] `ggmRaceSexInfo::CallEFun_SetPhysObj` · 004722c0
- [ ] `ggmRaceSexInfo::CallEFun_SetPileAppearance` · 00472320
- [ ] `ggmRaceSexInfo::CallEFun_GetMusic` · 00472380
- [ ] `ggmRaceSexInfo::CallEFun_GetPhysObj` · 004723f0
- [ ] `ggmRaceSexInfo::CallEFun_GetPileAppearance` · 00472460
- [ ] `ggmRaceSexInfo::CallEFun_PutObjInLoc` · 004724d0
- [ ] `ggmRaceSexInfo::CallEFun_Verify` · 00472560
- [ ] `ggmRaceSexInfo::CallEFun_GetSubDataIDs` · 004725d0 · legions 004703b0

## `ac2/shared/src/interp/glue/ggmsceneinfo.cpp.c` (0/20)

### ggmSceneInfo
- [ ] `ggmSceneInfo::CallEFun_Clear` · 00471950
- [ ] `ggmSceneInfo::CallEFun_ClearForHackers` · 004719b0
- [ ] `ggmSceneInfo::CallEFun_Set` · 00471a10
- [ ] `ggmSceneInfo::CallEFun_SetActID` · 00471a70
- [ ] `ggmSceneInfo::CallEFun_SetHidden` · 00471ad0
- [ ] `ggmSceneInfo::CallEFun_SetID` · 00471b30
- [ ] `ggmSceneInfo::CallEFun_SetIDByName` · 00471b90
- [ ] `ggmSceneInfo::CallEFun_SetName` · 00471bf0
- [ ] `ggmSceneInfo::CallEFun_SetPlayable` · 00471c50
- [ ] `ggmSceneInfo::CallEFun_SetSceneNum` · 00471cb0
- [ ] `ggmSceneInfo::CallEFun_AddKeyframe` · 00471d10
- [ ] `ggmSceneInfo::CallEFun_GetActID` · 00471da0
- [ ] `ggmSceneInfo::CallEFun_GetID` · 00471e10
- [ ] `ggmSceneInfo::CallEFun_GetSceneNum` · 00471e80
- [ ] `ggmSceneInfo::CallEFun_IsHidden` · 00471ef0
- [ ] `ggmSceneInfo::CallEFun_IsPlayable` · 00471f60
- [ ] `ggmSceneInfo::CallEFun_Dump` · 00471fd0
- [ ] `ggmSceneInfo::CallEFun_GetName` · 00472090
- [ ] `ggmSceneInfo::CallEFun_GetSubDataIDs` · 00472140
- [ ] `ggmSceneInfo::CallEFun_Verify` · 004721c0 · legions 00470280

## `ac2/shared/src/interp/glue/ggmsceneinfolist.cpp.c` (0/4)

### ggmSceneInfoList
- [ ] `ggmSceneInfoList::CallEFun_num_elements` · 00471720
- [ ] `ggmSceneInfoList::CallEFun_pop_front` · 00471790
- [ ] `ggmSceneInfoList::CallEFun_push_tail` · 00471810
- [ ] `ggmSceneInfoList::CallEFun_insert_sort` · 004718d0

## `ac2/shared/src/interp/glue/gingredientadaptor.cpp.c` (0/8)

### gIngredientAdaptor
- [ ] `gIngredientAdaptor::CallEFun_SetAutoPopulate` · 00471310
- [ ] `gIngredientAdaptor::CallEFun_SetDescription` · 00471370
- [ ] `gIngredientAdaptor::CallEFun_SetID` · 004713d0
- [ ] `gIngredientAdaptor::CallEFun_SetRecipeDID` · 00471430
- [ ] `gIngredientAdaptor::CallEFun_GetAutoPopulate` · 00471490
- [ ] `gIngredientAdaptor::CallEFun_GetID` · 00471500
- [ ] `gIngredientAdaptor::CallEFun_GetRecipeDID` · 00471570
- [ ] `gIngredientAdaptor::CallEFun_GetDescription` · 004715e0

## `ac2/shared/src/interp/glue/ginventoryglobals.cpp.c` (0/1)

### gInventoryGlobals
- [ ] `gInventoryGlobals::CallIFun_GetCoinMaxStackSize` · 0046d050 · legions 0046d7b0

## `ac2/shared/src/interp/glue/gradarcontrol.cpp.c` (0/1)

### gRadarControl
- [ ] `gRadarControl::CallIFun_InqRadarRanges` · 0046ce60 · legions 0046d5c0

## `ac2/shared/src/interp/glue/grecipeadaptor.cpp.c` (0/34)

### gRecipeAdaptor
- [ ] `gRecipeAdaptor::CallEFun_SetCharges` · 004704f0
- [ ] `gRecipeAdaptor::CallEFun_SetCost` · 00470550
- [ ] `gRecipeAdaptor::CallEFun_SetCraftSkillCategory` · 004705b0
- [ ] `gRecipeAdaptor::CallEFun_SetCraftSkillDID` · 00470610
- [ ] `gRecipeAdaptor::CallEFun_SetDID` · 00470670
- [ ] `gRecipeAdaptor::CallEFun_SetDescription` · 004706d0
- [ ] `gRecipeAdaptor::CallEFun_SetDifficulty` · 00470730
- [ ] `gRecipeAdaptor::CallEFun_SetDifficultyMinusSkillCutoff` · 00470790
- [ ] `gRecipeAdaptor::CallEFun_SetDynamicCost` · 004707f0
- [ ] `gRecipeAdaptor::CallEFun_SetDynamicDifficulty` · 00470850
- [ ] `gRecipeAdaptor::CallEFun_SetHasAutoPopulatingIngredients` · 004708b0
- [ ] `gRecipeAdaptor::CallEFun_SetHasNameColoringTable` · 00470910
- [ ] `gRecipeAdaptor::CallEFun_SetIconDID` · 00470970
- [ ] `gRecipeAdaptor::CallEFun_SetLastSpinnerValue` · 004709d0
- [ ] `gRecipeAdaptor::CallEFun_SetMaxSpinnerValue` · 00470a30
- [ ] `gRecipeAdaptor::CallEFun_SetMaximumNumberOfIngredients` · 00470a90
- [ ] `gRecipeAdaptor::CallEFun_SetMinSpinnerValue` · 00470af0
- [ ] `gRecipeAdaptor::CallEFun_SetName` · 00470b50
- [ ] `gRecipeAdaptor::CallEFun_SetNameColor` · 00470bb0
- [ ] `gRecipeAdaptor::CallEFun_SetReadOnlySpinner` · 00470c10
- [ ] `gRecipeAdaptor::CallEFun_SetShown` · 00470c70
- [ ] `gRecipeAdaptor::CallEFun_SetTimeUseAgain` · 00470cd0
- [ ] `gRecipeAdaptor::CallEFun_GetCharges` · 00470d30
- [ ] `gRecipeAdaptor::CallEFun_GetCost` · 00470da0
- [ ] `gRecipeAdaptor::CallEFun_GetCraftSkillCategory` · 00470e10
- [ ] `gRecipeAdaptor::CallEFun_GetCraftSkillDID` · 00470e80
- [ ] `gRecipeAdaptor::CallEFun_GetDID` · 00470ef0
- [ ] `gRecipeAdaptor::CallEFun_GetDynamicCost` · 00470f60
- [ ] `gRecipeAdaptor::CallEFun_GetIconDID` · 00470fd0
- [ ] `gRecipeAdaptor::CallEFun_GetShown` · 00471040
- [ ] `gRecipeAdaptor::CallEFun_GetTimeUseAgain` · 004710b0
- [ ] `gRecipeAdaptor::CallEFun_HasReadOnlySpinner` · 00471130
- [ ] `gRecipeAdaptor::CallEFun_GetDescription` · 004711b0
- [ ] `gRecipeAdaptor::CallEFun_GetName` · 00471260

## `ac2/shared/src/interp/glue/gselectioninfo.cpp.c` (0/16)

### gSelectionInfo
- [ ] `gSelectionInfo::CallEFun_SetCurHealth` · 0046fe40
- [ ] `gSelectionInfo::CallEFun_SetCurVigor` · 0046fea0
- [ ] `gSelectionInfo::CallEFun_SetMaxHealth` · 0046ff00
- [ ] `gSelectionInfo::CallEFun_SetMaxVigor` · 0046ff60
- [ ] `gSelectionInfo::CallEFun_SetPKDamage` · 0046ffc0
- [ ] `gSelectionInfo::CallEFun_SetPKVigorloss` · 00470020
- [ ] `gSelectionInfo::CallEFun_SetType` · 00470080
- [ ] `gSelectionInfo::CallEFun_GetCurHealth` · 004700e0
- [ ] `gSelectionInfo::CallEFun_GetCurVigor` · 00470150
- [ ] `gSelectionInfo::CallEFun_GetMaxHealth` · 004701c0
- [ ] `gSelectionInfo::CallEFun_GetMaxVigor` · 00470230
- [ ] `gSelectionInfo::CallEFun_GetPKDamage` · 004702a0
- [ ] `gSelectionInfo::CallEFun_GetPKVigorloss` · 00470310
- [ ] `gSelectionInfo::CallEFun_GetType` · 00470380
- [ ] `gSelectionInfo::CallEFun_eq` · 004703f0
- [ ] `gSelectionInfo::CallEFun_ne` · 00470470

## `ac2/shared/src/interp/glue/gshortcutinfo.cpp.c` (0/10)

### gShortcutInfo
- [ ] `gShortcutInfo::CallEFun_SetDataEnum` · 0046f9b0
- [ ] `gShortcutInfo::CallEFun_SetDataID` · 0046fa10
- [ ] `gShortcutInfo::CallEFun_SetDataIID` · 0046fa70
- [ ] `gShortcutInfo::CallEFun_SetType` · 0046fad0
- [ ] `gShortcutInfo::CallEFun_GetDataEnum` · 0046fb30
- [ ] `gShortcutInfo::CallEFun_GetDataID` · 0046fba0
- [ ] `gShortcutInfo::CallEFun_GetDataIID` · 0046fc10
- [ ] `gShortcutInfo::CallEFun_GetType` · 0046fc80
- [ ] `gShortcutInfo::CallEFun_SetDataStr` · 0046fcf0
- [ ] `gShortcutInfo::CallEFun_GetDataStr` · 0046fd70

## `ac2/shared/src/interp/glue/gskillpanelmanager.cpp.c` (0/5)

### gSkillPanelManager
- [ ] `gSkillPanelManager::SkillUINodeHashToARHash` · 0046d140 · legions 0046d8a0
- [ ] `gSkillPanelManager::ARHashToSkillUINodeHash` · 0046d270 · legions 0046d9d0
- [ ] `gSkillPanelManager::CallIFun_InqIcon` · 0046d380 · legions 0046dae0
- [ ] `gSkillPanelManager::CallIFun_InqName` · 0046d510 · legions 0046dc70
- [ ] `gSkillPanelManager::CallIFun_InqNodeHash` · 0046d6f0 · legions 0046de50

## `ac2/shared/src/interp/glue/gskilluinode.cpp.c` (0/24)

### gSkillUINode
- [ ] `gSkillUINode::CallEFun_AddParent` · 0046ed90
- [ ] `gSkillUINode::CallEFun_Clear` · 0046edf0
- [ ] `gSkillUINode::CallEFun_Set` · 0046ee50
- [ ] `gSkillUINode::CallEFun_SetCoordinates` · 0046eeb0
- [ ] `gSkillUINode::CallEFun_SetDescription` · 0046ef20
- [ ] `gSkillUINode::CallEFun_SetEnum` · 0046ef80
- [ ] `gSkillUINode::CallEFun_SetIcon` · 0046efe0
- [ ] `gSkillUINode::CallEFun_SetName` · 0046f040
- [ ] `gSkillUINode::CallEFun_SetPassive` · 0046f0a0
- [ ] `gSkillUINode::CallEFun_SetType` · 0046f100
- [ ] `gSkillUINode::CallEFun_SetUntrainable` · 0046f160
- [ ] `gSkillUINode::CallEFun_AddDependency` · 0046f1c0
- [ ] `gSkillUINode::CallEFun_GetEnum` · 0046f230
- [ ] `gSkillUINode::CallEFun_GetIcon` · 0046f2a0
- [ ] `gSkillUINode::CallEFun_GetType` · 0046f310
- [ ] `gSkillUINode::CallEFun_GetX` · 0046f380
- [ ] `gSkillUINode::CallEFun_GetY` · 0046f3f0
- [ ] `gSkillUINode::CallEFun_IsPassive` · 0046f460
- [ ] `gSkillUINode::CallEFun_IsUntrainable` · 0046f4d0
- [ ] `gSkillUINode::CallEFun_GetDependencyList` · 0046f540
- [ ] `gSkillUINode::CallEFun_GetDescription` · 0046f660
- [ ] `gSkillUINode::CallEFun_GetName` · 0046f710
- [ ] `gSkillUINode::CallEFun_GetParentList` · 0046f7c0
- [ ] `gSkillUINode::CallEFun_InqCoordinates` · 0046f8e0

## `ac2/shared/src/interp/glue/gturbinegunutils.cpp.c` (0/2)

### gTurbineGunUtils
- [ ] `gTurbineGunUtils::CallEFun_GetWorldName` · 0046ea20
- [ ] `gTurbineGunUtils::CallEFun_GetWorldSuffix` · 0046eaa0

## `ac2/shared/src/interp/glue/gturbineuserstatus.cpp.c` (0/4)

### ReferenceTable
- [ ] `ReferenceTable::GetReferenceTagPtr` · 0046eb20 · legions 0046e9c0

### gTurbineUserStatus
- [ ] `gTurbineUserStatus::CallEFun_InqLocation` · 0046ebd0 · legions 0046ea70
- [ ] `gTurbineUserStatus::CallEFun_InqMode` · 0046ec60 · legions 0046eae0
- [ ] `gTurbineUserStatus::CallEFun_InqName` · 0046ed00

## `ac2/shared/src/interp/glue/guishop.cpp.c` (0/8)

### gUIShop
- [ ] `gUIShop::CallEFun_SetDesc` · 0046de60
- [ ] `gUIShop::CallEFun_SetID` · 0046dec0
- [ ] `gUIShop::CallEFun_SetName` · 0046df20
- [ ] `gUIShop::CallEFun_SetPortrait` · 0046df80
- [ ] `gUIShop::CallEFun_SetPosition` · 0046dfe0
- [ ] `gUIShop::CallEFun_SetSorter` · 0046e040
- [ ] `gUIShop::CallEFun_SetStoreSize` · 0046e0a0
- [ ] `gUIShop::CallEFun_AddSale` · 0046e280

## `ac2/shared/src/interp/glue/guishopsale.cpp.c` (0/13)

### gUIShopSale
- [ ] `gUIShopSale::CallEFun_SetCostDesc` · 0046d900
- [ ] `gUIShopSale::CallEFun_SetCostIcon` · 0046d980
- [ ] `gUIShopSale::CallEFun_SetCostQuantity` · 0046d9e0
- [ ] `gUIShopSale::CallEFun_SetExpired` · 0046da40
- [ ] `gUIShopSale::CallEFun_SetItemDID` · 0046daa0
- [ ] `gUIShopSale::CallEFun_SetItemDesc` · 0046db00
- [ ] `gUIShopSale::CallEFun_SetItemIID` · 0046db90
- [ ] `gUIShopSale::CallEFun_SetItemIconDesc` · 0046dbf0
- [ ] `gUIShopSale::CallEFun_SetItemIconID` · 0046dc80
- [ ] `gUIShopSale::CallEFun_SetOrdinal` · 0046dce0
- [ ] `gUIShopSale::CallEFun_SetRestricted` · 0046dd40
- [ ] `gUIShopSale::CallEFun_SetSold` · 0046dda0
- [ ] `gUIShopSale::CallEFun_SetStockQuantity` · 0046de00

## `portal/shared/src/interp/breakpointinfo.cpp.c` (0/1)

### InterpreterBreakpointHelper
- [ ] `InterpreterBreakpointHelper::SetTemporaryBreakpoint` · 0073c990

## `portal/shared/src/interp/bytestream.cpp.c` (14/87)

### ByteStream
- [ ] `ByteStream::Pack` · 006e4cd0 · legions 0066b440
- [ ] `ByteStream::PackSection` · 006e4d40 · legions 0066b4b0
- [x] `ByteStream::LoadOpcodeSection` · 006e4dc0 — ac2-script WLib/WState/ByteStream decode
- [x] `ByteStream::LoadImportTableSection` · 006e4e50 — ac2-script WLib/WState/ByteStream decode
- [x] `ByteStream::LoadStringLitTableSection` · 006e4ee0 — ac2-script WLib/WState/ByteStream decode
- [x] `ByteStream::LoadExportTableSection` · 006e4f70 — ac2-script WLib/WState/ByteStream decode
- [x] `ByteStream::LoadVTableSection` · 006e5000 — ac2-script WLib/WState/ByteStream decode
- [x] `ByteStream::LoadValidEventTableSection` · 006e5090 — ac2-script WLib/WState/ByteStream decode

### CObjCell
- [ ] `CObjCell::UpdateCellMaterial` · 006e5130 · legions 0066b600

### OpcodeStream
- [~] `OpcodeStream::StreamPack` · 006e5140 — ac2-script: unpack direction only

### ByteStream
- [ ] `ByteStream::DiscardAllDebuggingInformation` · 006e5220 · legions 0066b6f0
- [ ] `ByteStream::PackSectionList` · 006e5310 · legions 0066b7e0
- [ ] `ByteStream::InqExportPackage` · 006e5450
- [-] `ByteStream::LoadLineNumDebugSection` · 006e54e0 — text source loader, admin build only
- [-] `ByteStream::LoadFrameDebugInfoSection` · 006e5690 — text source loader, admin build only
- [ ] `ByteStream::ValidationFailure` · 006e5760
- [x] `ByteStream::IsDerivedFrom` · 006e5790 · legions 0066ba70 — ac2-script glue

### VersionTable
- [ ] `VersionTable::GetVersion` · 006e5820
- [~] `VersionTable::StreamPack` · 006e58e0 — ac2-script: unpack direction only

### ByteStream
- [x] `ByteStream::IsDerivedFrom` · 006e5960 · legions 0066bc20 — ac2-script glue
- [x] `ByteStream::LoadVersionInfoSection` · 006e5aa0 · legions 0066bd30 — ac2-script WLib/WState/ByteStream decode
- [x] `ByteStream::IsDerivedFrom` · 006e5c40 · legions 0066be60 — ac2-script glue

### PSmartArray<PackageInfo>
- [ ] `PSmartArray<PackageInfo>::StreamPack` · 006e5ce0 · legions 0066bea0

### PackageInfo
- [ ] `PackageInfo::PackageInfo` · 006e5e70

### AnimAssetWeightKey
- [ ] `AnimAssetWeightKey::StreamPack` · 006e5e80 · legions 006b22d0

### StringLitTable
- [~] `StringLitTable::StreamPack` · 006e5ec0 · legions 0066c040 — ac2-script: unpack direction only

### ByteStream
- [x] `ByteStream::IsDerivedFrom` · 006e5f00 — ac2-script glue

### DualHash<PStringBase<char>,CheckpointExportData>
- [ ] `DualHash<PStringBase<char>,CheckpointExportData>::find_right` · 006e5fa0 · legions 0066c0c0

### PSmartArray<VersionTable::ver>
- [ ] `PSmartArray<VersionTable::ver>::StreamPack` · 006e6030 · legions 0066c150

### VersionTable::ver
- [ ] `VersionTable::ver::ver` · 006e61f0 · legions 0066c310

### StringLitTable
- [ ] `StringLitTable::~StringLitTable` · 006e6230 · legions 0066c350

### ByteStream
- [ ] `ByteStream::LoadFunctionLocDebugSection` · 006e62d0 · legions 0066c3f0

### DualHash<PStringBase<char>,CheckpointExportData>
- [ ] `DualHash<PStringBase<char>,CheckpointExportData>::contains_right` · 006e6410 · legions 0066c4c0

### PSmartArray<PSmartArray<unsigned_long>_>
- [ ] `PSmartArray<PSmartArray<unsigned_long>_>::StreamPack` · 006e6480 · legions 0066c530

### PSmartArray<PSmartArray<VTableID>_>
- [ ] `PSmartArray<PSmartArray<VTableID>_>::StreamPack` · 006e6620 · legions 0066c6d0

### SmartArray<VTableID>
- [ ] `SmartArray<VTableID>::operator=` · 006e6820 · legions 0066c890

### VTableID
- [ ] `VTableID::`default_constructor_closure'` · 006e6930

### PSmartArray<VTableID>
- [ ] `PSmartArray<VTableID>::StreamPack` · 006e6940

### (free functions)
- [ ] `STREAMPACK` · 006e69e0 · legions 0066d840

### PSmartArray<POriginalSourceFileInfo>
- [ ] `PSmartArray<POriginalSourceFileInfo>::StreamPack` · 006e6a60 · legions 0066ca50

### POriginalSourceFileInfo
- [ ] `POriginalSourceFileInfo::POriginalSourceFileInfo` · 006e6c20 · legions 0066cc10

### ByteStream
- [x] `ByteStream::ReadByteStream` · 006e6c60 — ac2-script WLib/WState/ByteStream decode
- [x] `ByteStream::UnPack` · 006e6f40 — ac2-script WLib/WState/ByteStream decode
- [ ] `ByteStream::IsCheckpointedMember` · 006e6fb0 · legions 0066ce00
- [ ] `ByteStream::LoadSourceFileDebugSection` · 006e7090 · legions 0066ce80
- [ ] `ByteStream::Open` · 006e7380
- [x] `ByteStream::LoadSection` · 006e74f0 — ac2-script WLib/WState/ByteStream decode

### PSmartArray<FrameDebugInfo>
- [ ] `PSmartArray<FrameDebugInfo>::StreamPack` · 006e7750 · legions 0066d2e0

### FrameDebugInfo
- [ ] `FrameDebugInfo::FrameDebugInfo` · 006e78d0 · legions 0066d460
- [ ] `FrameDebugInfo::operator=` · 006e7910 · legions 0066d4a0

### POriginalSourceFileInfo
- [ ] `POriginalSourceFileInfo::StreamPack` · 006e7990 · legions 0066d520

### ByteStream
- [ ] `ByteStream::Clear` · 006e79d0 · legions 0066d560

### PHashTable<FunctionID,int>
- [~] `PHashTable<FunctionID,int>::StreamPack` · 006e7ad0 — ac2-script: unpack direction only

### PSmartArray<ImportData>
- [ ] `PSmartArray<ImportData>::StreamPack` · 006e7cb0 · legions 0066d8c0

### ImportData
- [ ] `ImportData::ImportData` · 006e7e30 · legions 0066da40

### PSmartArray<ImportSymbolInfo>
- [ ] `PSmartArray<ImportSymbolInfo>::StreamPack` · 006e7e80 · legions 0066da90

### ImportSymbolInfo
- [ ] `ImportSymbolInfo::ImportSymbolInfo` · 006e8000 · legions 0066dc10
- [ ] `ImportSymbolInfo::operator=` · 006e8040 · legions 0066dc50

### ImportData
- [ ] `ImportData::operator=` · 006e80b0 · legions 0066dcc0

### SmartArray<ImportSymbolInfo>
- [ ] `SmartArray<ImportSymbolInfo>::operator=` · 006e8140 · legions 0066dd50

### ImportSymbolInfo
- [~] `ImportSymbolInfo::StreamPack` · 006e81f0 · legions 0066de00 — ac2-script: unpack direction only

### PSmartArray<PLineOffsetList>
- [ ] `PSmartArray<PLineOffsetList>::StreamPack` · 006e8240 · legions 0066de50

### PLineOffsetList
- [ ] `PLineOffsetList::PLineOffsetList` · 006e8400

### SmartArray<PLineOffsetInfo>
- [ ] `SmartArray<PLineOffsetInfo>::operator=` · 006e8430 · legions 0066e040

### PLineOffsetInfo
- [ ] `PLineOffsetInfo::PLineOffsetInfo` · 006e8590

### PLineOffsetList
- [ ] `PLineOffsetList::StreamPack` · 006e85a0 · legions 0066e1b0

### PSmartArray<PLineOffsetInfo>
- [ ] `PSmartArray<PLineOffsetInfo>::StreamPack` · 006e85e0 · legions 0066e1f0

### PHashTable<PStringBase<char>,unsigned_long>
- [ ] `PHashTable<PStringBase<char>,unsigned_long>::StreamPack` · 006e8690 · legions 0066e2a0

### VTableSection
- [ ] `VTableSection::VTableSection` · 006e8950 · legions 0066e560
- [~] `VTableSection::StreamPack` · 006e89d0 · legions 0066e5e0 — ac2-script: unpack direction only
- [ ] `VTableSection::~VTableSection` · 006e8a30 · legions 0066e640

### ByteStream
- [ ] `ByteStream::ByteStream` · 006e8b00 · legions 0066e710
- [ ] `ByteStream::~ByteStream` · 006e8c80 · legions 0066e890

### PSmartArray<ExportData>
- [ ] `PSmartArray<ExportData>::StreamPack` · 006e8e90 · legions 0066eaa0

### ExportData
- [ ] `ExportData::ExportData` · 006e9010 · legions 0066ec20

### PSmartArray<ExportFunctionData>
- [ ] `PSmartArray<ExportFunctionData>::StreamPack` · 006e9040 · legions 0066ec50

### ExportFunctionData
- [ ] `ExportFunctionData::ExportFunctionData` · 006e91c0 · legions 0066edd0
- [ ] `ExportFunctionData::operator=` · 006e9210 · legions 0066ee20
- [~] `ExportFunctionData::StreamPack` · 006e92a0 · legions 0066eeb0 — ac2-script: unpack direction only

### ExportPackageArgs
- [ ] `ExportPackageArgs::ExportPackageArgs` · 006e9310 · legions 0066ef20

### DualHash<PStringBase<char>,CheckpointExportData>
- [ ] `DualHash<PStringBase<char>,CheckpointExportData>::init` · 006e9380 · legions 0066ef90

### ExportPackageArgs
- [~] `ExportPackageArgs::StreamPack` · 006e9440 · legions 0066f050 — ac2-script: unpack direction only

### ExportData
- [ ] `ExportData::operator=` · 006e94f0 · legions 0066f100

### SmartArray<ExportFunctionData>
- [ ] `SmartArray<ExportFunctionData>::operator=` · 006e9530 · legions 0066f140

### ExportPackageArgs
- [ ] `ExportPackageArgs::operator=` · 006e95a0 · legions 0066f1b0

### DualHash<PStringBase<char>,CheckpointExportData>
- [ ] `DualHash<PStringBase<char>,CheckpointExportData>::operator=` · 006e9650 · legions 0066f260
- [ ] `DualHash<PStringBase<char>,CheckpointExportData>::inc_num_elements` · 006e97e0 · legions 0066f3f0

## `portal/shared/src/interp/centraldebuginfo.cpp.c` (0/35)

### CentralDebugInfo
- [ ] `CentralDebugInfo::FindNearestSourceLineIdx` · 006e9910
- [ ] `CentralDebugInfo::FindNearestFunctionLocationIdx` · 006e9950

### DualHash<PStringBase<char>,unsigned_long>
- [ ] `DualHash<PStringBase<char>,unsigned_long>::init` · 006e9a60 · legions 0066f5e0

### DualHashIter<PStringBase<char>,CheckpointExportData>
- [ ] `DualHashIter<PStringBase<char>,CheckpointExportData>::start_bucket` · 006e9b20 · legions 0066f6a0

### CentralDebugInfo
- [ ] `CentralDebugInfo::FindFunctionLocation` · 006e9c30

### DualHash<PStringBase<char>,unsigned_long>
- [ ] `DualHash<PStringBase<char>,unsigned_long>::find_right` · 006e9cb0 · legions 0066f7b0

### OriginalSourceFileInfo
- [ ] `OriginalSourceFileInfo::operator=` · 006e9d40

### DualHash<PStringBase<char>,unsigned_long>
- [ ] `DualHash<PStringBase<char>,unsigned_long>::find_left` · 006e9f00 · legions 0066f9b0
- [ ] `DualHash<PStringBase<char>,unsigned_long>::add_internal` · 006e9ff0 · legions 0066faa0
- [ ] `DualHash<PStringBase<char>,unsigned_long>::find_left` · 006ea090 · legions 0066fb40
- [ ] `DualHash<PStringBase<char>,unsigned_long>::find_right` · 006ea120
- [ ] `DualHash<PStringBase<char>,unsigned_long>::UnPack` · 006ea1e0 · legions 0066fbd0
- [ ] `DualHash<PStringBase<char>,unsigned_long>::contains_left` · 006ea3c0 · legions 0066fdb0
- [ ] `DualHash<PStringBase<char>,unsigned_long>::contains_right` · 006ea430 · legions 0066fe20

### CentralDebugInfo
- [ ] `CentralDebugInfo::GetFilename` · 006ea580

### DualHash<PStringBase<char>,unsigned_long>
- [ ] `DualHash<PStringBase<char>,unsigned_long>::GetPackSize` · 006ea650 · legions 0066ffb0
- [ ] `DualHash<PStringBase<char>,unsigned_long>::pack_size` · 006ea660 · legions 0066ffc0
- [ ] `DualHash<PStringBase<char>,unsigned_long>::Pack` · 006ea7e0 · legions 00670140
- [ ] `DualHash<PStringBase<char>,unsigned_long>::clear` · 006ea9f0 · legions 00670350

### CentralDebugInfo
- [ ] `CentralDebugInfo::FindSourceLineLocation` · 006eac60
- [ ] `CentralDebugInfo::GetClosestSourceLineAtPos` · 006eae90
- [ ] `CentralDebugInfo::GetFunctionAtPos` · 006eaf90
- [ ] `CentralDebugInfo::GetFunctionContainingPos` · 006eb010
- [-] `CentralDebugInfo::LoadReadonlyWLinkMap` · 006eb060 — text source loader, admin build only

### DualHash<PStringBase<char>,unsigned_long>
- [ ] `DualHash<PStringBase<char>,unsigned_long>::destroy` · 006eb3f0 · legions 00670950
- [ ] `DualHash<PStringBase<char>,unsigned_long>::~DualHash<PStringBase<char>,unsigned_long>` · 006eb510 · legions 00670a70
- [ ] `DualHash<PStringBase<char>,unsigned_long>::inc_num_elements` · 006eb590 · legions 00670af0
- [ ] `DualHash<PStringBase<char>,unsigned_long>::operator=` · 006eb610 · legions 00670b70

### CentralDebugInfo
- [ ] `CentralDebugInfo::CentralDebugInfo` · 006eb920 · legions 00670e80
- [ ] `CentralDebugInfo::~CentralDebugInfo` · 006eba50 · legions 00670f80
- [ ] `CentralDebugInfo::PrepareForUse` · 006ebbc0
- [ ] `CentralDebugInfo::FindFilenameIdx` · 006ebdf0
- [ ] `CentralDebugInfo::GetSourceFile` · 006ebe90
- [ ] `CentralDebugInfo::GetLinesOfSourceFile` · 006ec280
- [ ] `CentralDebugInfo::GetLineOfSourceFile` · 006ec400

## `portal/shared/src/interp/contextstate.cpp.c` (0/1)

### ContextState
- [ ] `ContextState::QueryReferenceTree` · 006de060

## `portal/shared/src/interp/debugdata.cpp.c` (0/7)

### ConversationAction
- [ ] `ConversationAction::operator=` · 006ec580 · legions 00671450

### PFunctionLocationInfo
- [ ] `PFunctionLocationInfo::PFunctionLocationInfo` · 006ec5d0 · legions 006714a0

### PFunctionLocationInfoTable
- [ ] `PFunctionLocationInfoTable::GetPackSize` · 006ec790
- [ ] `PFunctionLocationInfoTable::Pack` · 006ec7c0

### SmartArray<PFunctionLocationInfo>
- [ ] `SmartArray<PFunctionLocationInfo>::AddToEnd` · 006ec8a0 · legions 00671730

### PFunctionLocationInfo
- [ ] `PFunctionLocationInfo::StreamPack` · 006ec930 · legions 006717c0

### PFunctionLocationInfoTable
- [ ] `PFunctionLocationInfoTable::UnPack` · 006ec970

## `portal/shared/src/interp/debugger/debug_cmds.cpp.c` (0/50)

### (free functions)
- [-] `PrintText` · 006edef0 · legions 00672c10 — script debugger (admin tools)
- [-] `PrintMessage` · 006edf10 — script debugger (admin tools)

### WslDebugger
- [-] `WslDebugger::ExecDebugCmd` · 006edf30 — script debugger (admin tools)
- [-] `WslDebugger::MakeSureConsoleWindowIsAvailable` · 006edfb0 · legions 00672c80 — script debugger (admin tools)
- [-] `WslDebugger::yyinput` · 006ee010 · legions 00672ce0 — script debugger (admin tools)
- [-] `WslDebugger::r_Once` · 006ee090 · legions 00672d30 — script debugger (admin tools)
- [-] `WslDebugger::r_SpecifyProcess` · 006ee0a0 · legions 00672d40 — script debugger (admin tools)
- [-] `WslDebugger::r_SpecifyProcessAndContext` · 006ee0b0 · legions 00672d50 — script debugger (admin tools)
- [-] `WslDebugger::r_EvalIntVal` · 006ee0d0 — script debugger (admin tools)
- [-] `WslDebugger::r_Step` · 006ee0f0 — script debugger (admin tools)
- [-] `WslDebugger::r_SetTraceLevel` · 006ee140 — script debugger (admin tools)
- [-] `WslDebugger::r_SetWslDebugLevel` · 006ee160 — script debugger (admin tools)
- [-] `WslDebugger::r_CppBreak` · 006ee180 — script debugger (admin tools)
- [-] `WslDebugger::r_Resume` · 006ee1b0 — script debugger (admin tools)
- [-] `WslDebugger::r_Quit` · 006ee1d0 — script debugger (admin tools)
- [-] `WslDebugger::r_DisplayHelp` · 006ee1f0 — script debugger (admin tools)
- [-] `WslDebugger::l_GetCurProcessID` · 006ee280 — script debugger (admin tools)
- [-] `WslDebugger::l_GetCurContextID` · 006ee2d0 — script debugger (admin tools)
- [-] `WslDebugger::l_GetCurContextPtr` · 006ee310 · legions 00672da0 — script debugger (admin tools)

### FrameMemberDebugInfo
- [-] `FrameMemberDebugInfo::~FrameMemberDebugInfo` · 006ee340 — script debugger (admin tools)

### WslDebugger
- [-] `WslDebugger::r_FunctionCatalog` · 006ee390 — script debugger (admin tools)

### MemberValue
- [-] `MemberValue::MemberValue` · 006ee410 — script debugger (admin tools)

### WslDebugger
- [-] `WslDebugger::GetLocationString` · 006ee4d0 — script debugger (admin tools)
- [-] `WslDebugger::r_ListSource` · 006ee5e0 — script debugger (admin tools)
- [-] `WslDebugger::l_GetCurFile` · 006ee730 · legions 00672e40 — script debugger (admin tools)

### WslDbgReportGenPage
- [-] `WslDbgReportGenPage::GetReportName` · 006ee750 · legions 00672e60 — script debugger (admin tools)

### (free functions)
- [-] `GetVarChunk` · 006ee770 — script debugger (admin tools)

### MemberValue
- [-] `MemberValue::GetVarString` · 006ee850 — script debugger (admin tools)

### WslDebugger
- [-] `WslDebugger::GetWslStackTrace` · 006eed00 · legions 00672dd0 — script debugger (admin tools)
- [-] `WslDebugger::GetProcAndContextDescription` · 006ef050 — script debugger (admin tools)
- [-] `WslDebugger::r_SourceCatalog` · 006ef1a0 — script debugger (admin tools)
- [-] `WslDebugger::r_SourceList` · 006ef300 — script debugger (admin tools)
- [-] `WslDebugger::r_BreakpointCatalog` · 006ef4d0 — script debugger (admin tools)
- [-] `WslDebugger::r_BreakpointRemoveAll` · 006ef5e0 — script debugger (admin tools)
- [-] `WslDebugger::r_StackList` · 006ef600 — script debugger (admin tools)

### WslDbgReportGenPage
- [-] `WslDbgReportGenPage::GenReport` · 006ef690 · legions 00673110 — script debugger (admin tools)

### WslDebugger
- [-] `WslDebugger::RunConsoleDebugger` · 006ef7d0 · legions 00672e80 — script debugger (admin tools)
- [-] `WslDebugger::r_BreakpointRemoveAtFileLine` · 006ef9a0 — script debugger (admin tools)
- [-] `WslDebugger::r_BreakpointRemoveAtFunction` · 006efaa0 — script debugger (admin tools)
- [-] `WslDebugger::r_BreakpointRemoveAtOpcode` · 006efbb0 — script debugger (admin tools)

### InterpreterBreakpointHelper
- [-] `InterpreterBreakpointHelper::AddBreakpoint` · 006efc70 — script debugger (admin tools)

### WslDebugger
- [-] `WslDebugger::r_BreakpointSetAtFileLine` · 006efcd0 — script debugger (admin tools)
- [-] `WslDebugger::r_BreakpointSetAtFunction` · 006efe20 — script debugger (admin tools)
- [-] `WslDebugger::r_BreakpointSetAtOpcode` · 006eff70 — script debugger (admin tools)

### MemberValue
- [-] `MemberValue::Initialize` · 006f0010 — script debugger (admin tools)
- [-] `MemberValue::InitializeByName` · 006f0160 — script debugger (admin tools)

### (free functions)
- [-] `DumpAllFrameVars` · 006f0230 — script debugger (admin tools)
- [-] `FindAndPrintVar` · 006f05c0 — script debugger (admin tools)

### WslDebugger
- [-] `WslDebugger::r_EvalVar` · 006f0970 — script debugger (admin tools)
- [-] `WslDebugger::r_FrameList` · 006f0c50 — script debugger (admin tools)

## `portal/shared/src/interp/debugger/debug_lexer.cpp.c` (0/7)

### (free functions)
- [-] `yy_get_previous_state` · 0073d120 · legions 006b8f40 — script debugger (admin tools)
- [-] `yy_try_NUL_trans` · 0073d1f0 · legions 006b9010 — script debugger (admin tools)
- [-] `yy_flush_buffer` · 0073d290 · legions 006b90b0 — script debugger (admin tools)
- [-] `yy_create_buffer` · 0073d2f0 · legions 006b9110 — script debugger (admin tools)
- [-] `yyrestart` · 0073d390 · legions 006b91b0 — script debugger (admin tools)
- [-] `yy_get_next_buffer` · 0073d400 · legions 006b9220 — script debugger (admin tools)
- [-] `yylex` · 0073d5a0 · legions 006b93c0 — script debugger (admin tools)

## `portal/shared/src/interp/debugger/debuggingefuns.cpp.c` (0/13)

### (free functions)
- [-] `PrintInt` · 006f10f0 · legions 006733c0 — script debugger (admin tools)
- [-] `PrintLongInt` · 006f1110 · legions 006733e0 — script debugger (admin tools)
- [-] `PrintFloat` · 006f1130 · legions 00673400 — script debugger (admin tools)
- [-] `PrintLongFloat` · 006f1150 · legions 00673420 — script debugger (admin tools)
- [-] `MapIIDtoProcID` · 006f1170 — script debugger (admin tools)
- [-] `MapProcIDtoIID` · 006f11a0 — script debugger (admin tools)
- [-] `PrintString` · 006f1210 · legions 00673490 — script debugger (admin tools)
- [-] `GenerateFileAndLine` · 006f1280 · legions 00673500 — script debugger (admin tools)
- [-] `WSLDebugger` · 006f12f0 · legions 00673620 — script debugger (admin tools)

### HashSet<PStringBase<char>_>
- [-] `HashSet<PStringBase<char>_>::add` · 006f13a0 · legions 006736d0 — script debugger (admin tools)

### (free functions)
- [-] `WSLAssert` · 006f1760 · legions 00673a90 — script debugger (admin tools)
- [-] `WSLAssertMsg` · 006f1820 · legions 00673b50 — script debugger (admin tools)
- [-] `WSLAssertFailure` · 006f1910 · legions 00673c40 — script debugger (admin tools)

## `portal/shared/src/interp/debugger/framedebuginfo.cpp.c` (0/3)

### FrameMemberDebugInfo
- [-] `FrameMemberDebugInfo::GetPackSize` · 006f0e40 — script debugger (admin tools)
- [-] `FrameMemberDebugInfo::Pack` · 006f0eb0 — script debugger (admin tools)
- [-] `FrameMemberDebugInfo::UnPack` · 006f0f50 — script debugger (admin tools)

## `portal/shared/src/interp/glue/gaahash.cpp.c` (20/20)

### gAAHash
- [x] `gAAHash::CallEFun_empty` · 00567350 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gAAHash::CallEFun_num_buckets` · 005673c0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gAAHash::CallEFun_num_elements` · 00567430 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gAAHash::CallEFun_too_big` · 005674a0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gAAHash::CallEFun_too_small` · 00567520 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gAAHash::CallEFun_flush` · 005675a0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gAAHash::CallEFun_contains` · 00567600 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gAAHash::CallEFun_erase` · 00567690 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gAAHash::CallEFun_add` · 00567730 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gAAHash::CallEFun_set` · 005677c0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

### BaseHash<unsigned_long,unsigned_long,2>
- [x] `BaseHash<unsigned_long,unsigned_long,2>::get_begin` · 00567850 · legions 00537a90 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `BaseHash<unsigned_long,unsigned_long,2>::get_end` · 005678d0 · legions 00537b10 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

### gAAHash
- [x] `gAAHash::CallEFun_get_begin` · 00567930 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gAAHash::CallEFun_get_end` · 005679b0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gAAHash::CallEFun_grow` · 00567a30 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gAAHash::CallEFun_resize` · 00567aa0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gAAHash::CallEFun_shrink` · 00567b50 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gAAHash::CallEFun_optimize_size` · 00567bc0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gAAHash::CallEFun_find` · 00567c40 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gAAHash::CallEFun_remove` · 00567cf0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

## `portal/shared/src/interp/glue/gaahashiter.cpp.c` (6/6)

### gAAHashIter
- [x] `gAAHashIter::CallEFun_data` · 005670a0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gAAHashIter::CallEFun_key` · 00567110 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gAAHashIter::CallEFun_equals` · 00567180 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gAAHashIter::CallEFun_next` · 00567210 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gAAHashIter::CallEFun_notdone` · 00567270 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gAAHashIter::CallEFun_remove` · 005672e0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

## `portal/shared/src/interp/glue/gaamultihash.cpp.c` (13/13)

### gAAMultiHash
- [x] `gAAMultiHash::CallEFun_size` · 00566a20 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gAAMultiHash::CallEFun_count` · 00566a90 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

### BaseMultiHash<unsigned_long,unsigned_long,4>
- [x] `BaseMultiHash<unsigned_long,unsigned_long,4>::find` · 00566b30 · legions 00537490 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `BaseMultiHash<unsigned_long,unsigned_long,4>::find_last` · 00566bb0 · legions 00537510 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `BaseMultiHash<unsigned_long,unsigned_long,4>::get_begin` · 00566c10 · legions 00537570 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

### gAAMultiHash
- [x] `gAAMultiHash::CallEFun_clear` · 00566c70 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gAAMultiHash::CallEFun_erase` · 00566cd0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gAAMultiHash::CallEFun_find` · 00566d50 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gAAMultiHash::CallEFun_find_last` · 00566dd0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gAAMultiHash::CallEFun_get_begin` · 00566e50 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gAAMultiHash::CallEFun_get_end` · 00566ed0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gAAMultiHash::CallEFun_remove` · 00566f90 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gAAMultiHash::CallEFun_add` · 00567010 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

## `portal/shared/src/interp/glue/gaamultihashiter.cpp.c` (4/4)

### gAAMultiHashIter
- [x] `gAAMultiHashIter::CallEFun_data` · 00566850 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gAAMultiHashIter::CallEFun_equals` · 005668c0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gAAMultiHashIter::CallEFun_key` · 00566940 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gAAMultiHashIter::CallEFun_next` · 005669b0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

## `portal/shared/src/interp/glue/gaamultimap.cpp.c` (19/19)

### gAAMultiMap
- [x] `gAAMultiMap::CallEFun_empty` · 0054f510 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gAAMultiMap::CallEFun_count` · 0054f5e0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

### BaseMultiMap<unsigned_long,unsigned_long,13970>
- [x] `BaseMultiMap<unsigned_long,unsigned_long,13970>::find` · 0054f7d0 · legions 0052bf80 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `BaseMultiMap<unsigned_long,unsigned_long,13970>::find_last` · 0054f840 · legions 0052bff0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `BaseMultiMap<unsigned_long,unsigned_long,13970>::get_begin` · 0054f8a0 · legions 0052c050 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `BaseMultiMap<unsigned_long,unsigned_long,13970>::get_end` · 0054f8f0 · legions 0052c0a0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `BaseMultiMap<unsigned_long,unsigned_long,13970>::lower_bound` · 0054f940 · legions 0052c0f0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `BaseMultiMap<unsigned_long,unsigned_long,13970>::upper_bound` · 0054f9b0 · legions 0052c160 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

### gAAMultiMap
- [x] `gAAMultiMap::CallEFun_clear` · 0054fa20 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gAAMultiMap::CallEFun_erase` · 0054faa0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gAAMultiMap::CallEFun_find` · 0054fb30 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gAAMultiMap::CallEFun_find_last` · 0054fbb0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gAAMultiMap::CallEFun_get_begin` · 0054fc30 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gAAMultiMap::CallEFun_get_end` · 0054fcb0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gAAMultiMap::CallEFun_lower_bound` · 0054fd30 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gAAMultiMap::CallEFun_upper_bound` · 0054fdb0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

### BaseMultiMap<unsigned_long,unsigned_long,13970>
- [x] `BaseMultiMap<unsigned_long,unsigned_long,13970>::insert` · 0054fe30 · legions 0052c4c0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

### gAAMultiMap
- [x] `gAAMultiMap::CallEFun_insert` · 0054fe80 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gAAMultiMap::CallEFun_remove` · 0054fee0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

## `portal/shared/src/interp/glue/gaamultimapiter.cpp.c` (5/5)

### gAAMultiMapIter
- [x] `gAAMultiMapIter::CallEFun_data` · 0054f2a0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gAAMultiMapIter::CallEFun_equals` · 0054f310 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gAAMultiMapIter::CallEFun_key` · 0054f3a0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gAAMultiMapIter::CallEFun_next` · 0054f410 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gAAMultiMapIter::CallEFun_prev` · 0054f4b0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

## `portal/shared/src/interp/glue/gaarray.cpp.c` (12/12)

### gAArray
- [x] `gAArray::CallEFun_flush` · 00566190 — ac2-script glue::lists
- [x] `gAArray::CallEFun_SetNElements` · 005661f0 — ac2-script glue::lists
- [x] `gAArray::CallEFun_contains` · 00566280 — ac2-script glue::lists
- [x] `gAArray::CallEFun_empty` · 00566310 — ac2-script glue::lists
- [x] `gAArray::CallEFun_get` · 00566380 — ac2-script glue::lists
- [x] `gAArray::CallEFun_num_elements` · 00566400 — ac2-script glue::lists
- [x] `gAArray::CallEFun_set` · 00566470 — ac2-script glue::lists
- [x] `gAArray::CallEFun_erase` · 00566500 — ac2-script glue::lists
- [x] `gAArray::CallEFun_push` · 005665a0 — ac2-script glue::lists
- [x] `gAArray::CallEFun_push_unique` · 00566650 — ac2-script glue::lists
- [x] `gAArray::CallEFun_pop` · 005666d0 — ac2-script glue::lists
- [x] `gAArray::CallEFun_query` · 00566790 — ac2-script glue::lists

## `portal/shared/src/interp/glue/gahashset.cpp.c` (17/20)

### gAHashSet
- [x] `gAHashSet::CallEFun_empty` · 00565870 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gAHashSet::CallEFun_num_buckets` · 005658e0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gAHashSet::CallEFun_num_elements` · 00565950 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gAHashSet::CallEFun_too_big` · 005659c0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gAHashSet::CallEFun_too_small` · 00565a40 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gAHashSet::CallEFun_flush` · 00565ac0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gAHashSet::CallEFun_contains` · 00565b20 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gAHashSet::CallEFun_erase` · 00565bb0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

### HashSet<unsigned_long>
- [ ] `HashSet<unsigned_long>::add` · 00565c50 · legions 00536d30

### gAHashSet
- [x] `gAHashSet::CallEFun_add` · 00565cc0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

### HashSet<unsigned_long>
- [ ] `HashSet<unsigned_long>::grow` · 00565d40 · legions 00536e00
- [ ] `HashSet<unsigned_long>::shrink` · 00565d90 · legions 00536e50

### BaseHashSet<unsigned_long,7>
- [x] `BaseHashSet<unsigned_long,7>::get_begin` · 00565de0 · legions 00536f00 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `BaseHashSet<unsigned_long,7>::get_end` · 00565e60 · legions 00536f80 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

### gAHashSet
- [x] `gAHashSet::CallEFun_get_begin` · 00565ec0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gAHashSet::CallEFun_get_end` · 00565f40 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gAHashSet::CallEFun_grow` · 00565fc0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gAHashSet::CallEFun_resize` · 00566020 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gAHashSet::CallEFun_shrink` · 005660b0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gAHashSet::CallEFun_optimize_size` · 00566110 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

## `portal/shared/src/interp/glue/gahashsetiter.cpp.c` (5/6)

### gAHashSetIter
- [x] `gAHashSetIter::CallEFun_key` · 005655d0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

### IntrusiveHashIterator<unsigned_long,HashSetData<unsigned_long>_*>
- [ ] `IntrusiveHashIterator<unsigned_long,HashSetData<unsigned_long>_*>::remove` · 00565640

### gAHashSetIter
- [x] `gAHashSetIter::CallEFun_equals` · 005656a0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gAHashSetIter::CallEFun_notdone` · 00565730 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gAHashSetIter::CallEFun_next` · 005657a0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gAHashSetIter::CallEFun_remove` · 00565800 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

## `portal/shared/src/interp/glue/galhash.cpp.c` (20/20)

### gALHash
- [x] `gALHash::CallEFun_empty` · 005649a0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gALHash::CallEFun_num_buckets` · 00564a10 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gALHash::CallEFun_num_elements` · 00564a80 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gALHash::CallEFun_too_big` · 00564af0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gALHash::CallEFun_too_small` · 00564b70 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gALHash::CallEFun_flush` · 00564bf0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gALHash::CallEFun_contains` · 00564ca0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gALHash::CallEFun_erase` · 00564d30 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gALHash::CallEFun_add` · 00564e60 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gALHash::CallEFun_set` · 00564f00 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

### BaseHash<unsigned_long,unsigned___int64,9>
- [x] `BaseHash<unsigned_long,unsigned___int64,9>::get_begin` · 00565050 · legions 00536890 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `BaseHash<unsigned_long,unsigned___int64,9>::get_end` · 005650d0 · legions 00536910 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

### gALHash
- [x] `gALHash::CallEFun_get_begin` · 00565130 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gALHash::CallEFun_get_end` · 005651b0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gALHash::CallEFun_grow` · 00565230 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gALHash::CallEFun_resize` · 005652a0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gALHash::CallEFun_shrink` · 00565350 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gALHash::CallEFun_optimize_size` · 005653c0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gALHash::CallEFun_find` · 00565440 · legions 00536b90 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gALHash::CallEFun_remove` · 005654f0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

## `portal/shared/src/interp/glue/galhashiter.cpp.c` (6/6)

### gALHashIter
- [x] `gALHashIter::CallEFun_data` · 005646f0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gALHashIter::CallEFun_key` · 00564760 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gALHashIter::CallEFun_equals` · 005647d0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gALHashIter::CallEFun_notdone` · 00564860 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gALHashIter::CallEFun_next` · 005648d0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gALHashIter::CallEFun_remove` · 00564930 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

## `portal/shared/src/interp/glue/galist.cpp.c` (49/52)

### gAList
- [x] `gAList::CallEFun_empty` · 00562880 — ac2-script glue::lists
- [x] `gAList::CallEFun_num_elements` · 005628f0 — ac2-script glue::lists
- [x] `gAList::CallEFun_push_back` · 00562960 — ac2-script glue::lists
- [x] `gAList::CallEFun_push_front` · 005629e0 — ac2-script glue::lists

### BaseList<unsigned_long,11>
- [x] `BaseList<unsigned_long,11>::insert_sort_float` · 00562a60 · legions 00535ce0 — ac2-script glue::lists
- [x] `BaseList<unsigned_long,11>::insert_sort_int` · 00562b00 · legions 00535d80 — ac2-script glue::lists

### gAList
- [x] `gAList::CallEFun_get_position` · 00562b90 — ac2-script glue::lists
- [x] `gAList::CallEFun_insert_back` · 00562c20 — ac2-script glue::lists
- [x] `gAList::CallEFun_insert_front` · 00562ca0 — ac2-script glue::lists
- [x] `gAList::CallEFun_insert_head` · 00562d20 — ac2-script glue::lists
- [x] `gAList::CallEFun_insert_sort_float` · 00562da0 — ac2-script glue::lists
- [x] `gAList::CallEFun_insert_sort_int` · 00562e20 — ac2-script glue::lists
- [x] `gAList::CallEFun_insert_tail` · 00562ea0 — ac2-script glue::lists
- [x] `gAList::CallEFun_push` · 00562f20 — ac2-script glue::lists
- [x] `gAList::CallEFun_push_head` · 00562fa0 — ac2-script glue::lists
- [x] `gAList::CallEFun_push_tail` · 00563020 — ac2-script glue::lists
- [x] `gAList::CallEFun_remove` · 005630a0 — ac2-script glue::lists
- [x] `gAList::CallEFun_contains` · 00563140 — ac2-script glue::lists
- [x] `gAList::CallEFun_erase_back` · 005631d0 — ac2-script glue::lists
- [x] `gAList::CallEFun_erase_front` · 00563280 — ac2-script glue::lists
- [x] `gAList::CallEFun_erase_head` · 00563330 — ac2-script glue::lists
- [x] `gAList::CallEFun_erase_tail` · 005633e0 — ac2-script glue::lists
- [x] `gAList::CallEFun_flush` · 00563490 — ac2-script glue::lists
- [x] `gAList::CallEFun_insert` · 005634f0 — ac2-script glue::lists
- [x] `gAList::CallEFun_push_unique_back` · 00563570 — ac2-script glue::lists
- [x] `gAList::CallEFun_push_unique_front` · 00563610 — ac2-script glue::lists
- [x] `gAList::CallEFun_insert_unique_back` · 005636b0 — ac2-script glue::lists
- [x] `gAList::CallEFun_insert_unique_front` · 00563750 — ac2-script glue::lists
- [x] `gAList::CallEFun_insert_unique_head` · 005637f0 — ac2-script glue::lists
- [x] `gAList::CallEFun_insert_unique_tail` · 00563890 — ac2-script glue::lists
- [x] `gAList::CallEFun_push_unique_head` · 00563930 — ac2-script glue::lists
- [x] `gAList::CallEFun_push_unique_tail` · 005639d0 — ac2-script glue::lists

### BaseList<unsigned_long,11>
- [~] `BaseList<unsigned_long,11>::get_begin` · 00563a70 · legions 00536090 — ac2-script glue::lists: iterators hold an index, not a node
- [~] `BaseList<unsigned_long,11>::get_reverse` · 00563ac0 · legions 005360e0 — ac2-script glue::lists: iterators hold an index, not a node
- [~] `BaseList<unsigned_long,11>::get_end` · 00563b10 · legions 00536130 — ac2-script glue::lists: iterators hold an index, not a node

### gAList
- [x] `gAList::CallEFun_get_begin` · 00563b60 — ac2-script glue::lists
- [x] `gAList::CallEFun_get_end` · 00563be0 — ac2-script glue::lists
- [x] `gAList::CallEFun_get_reverse` · 00563c60 — ac2-script glue::lists
- [x] `gAList::CallEFun_clone` · 00563ce0 — ac2-script glue::lists
- [x] `gAList::CallEFun_inq_data` · 00563d80 — ac2-script glue::lists
- [x] `gAList::CallEFun_peek_back` · 00563e30 — ac2-script glue::lists
- [x] `gAList::CallEFun_peek_front` · 00563ee0 — ac2-script glue::lists
- [x] `gAList::CallEFun_peek_head` · 00563f90 — ac2-script glue::lists
- [x] `gAList::CallEFun_peek_tail` · 00564040 — ac2-script glue::lists
- [x] `gAList::CallEFun_pop_back` · 005640f0 — ac2-script glue::lists
- [x] `gAList::CallEFun_pop_front` · 00564190 — ac2-script glue::lists
- [x] `gAList::CallEFun_pop_head` · 00564270 — ac2-script glue::lists
- [x] `gAList::CallEFun_pop_tail` · 00564350 — ac2-script glue::lists
- [x] `gAList::CallEFun_remove_back` · 005643f0 — ac2-script glue::lists
- [x] `gAList::CallEFun_remove_front` · 00564490 — ac2-script glue::lists
- [x] `gAList::CallEFun_remove_head` · 00564570 — ac2-script glue::lists
- [x] `gAList::CallEFun_remove_tail` · 00564650 — ac2-script glue::lists

## `portal/shared/src/interp/glue/galistiter.cpp.c` (0/7)

### gAListIter
- [~] `gAListIter::CallEFun_next` · 00562520 — ac2-script glue::lists: iterators hold an index, not a node
- [~] `gAListIter::CallEFun_prev` · 00562580 — ac2-script glue::lists: iterators hold an index, not a node
- [~] `gAListIter::CallEFun_data` · 005625e0 — ac2-script glue::lists: iterators hold an index, not a node
- [~] `gAListIter::CallEFun_equals` · 00562650 — ac2-script glue::lists: iterators hold an index, not a node
- [~] `gAListIter::CallEFun_notdone` · 005626d0 — ac2-script glue::lists: iterators hold an index, not a node
- [~] `gAListIter::CallEFun_insert` · 00562740 — ac2-script glue::lists: iterators hold an index, not a node
- [~] `gAListIter::CallEFun_remove` · 005627f0 — ac2-script glue::lists: iterators hold an index, not a node

## `portal/shared/src/interp/glue/galmultihash.cpp.c` (13/13)

### gALMultiHash
- [x] `gALMultiHash::CallEFun_size` · 00561d30 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gALMultiHash::CallEFun_count` · 00561e50 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

### BaseMultiHash<unsigned_long,unsigned___int64,13>
- [x] `BaseMultiHash<unsigned_long,unsigned___int64,13>::find` · 00561fb0 · legions 00535690 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `BaseMultiHash<unsigned_long,unsigned___int64,13>::find_last` · 00562030 · legions 00535710 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `BaseMultiHash<unsigned_long,unsigned___int64,13>::get_begin` · 00562090 · legions 00535770 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

### gALMultiHash
- [x] `gALMultiHash::CallEFun_clear` · 005620f0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gALMultiHash::CallEFun_erase` · 00562150 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gALMultiHash::CallEFun_find` · 005621d0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gALMultiHash::CallEFun_find_last` · 00562250 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gALMultiHash::CallEFun_get_begin` · 005622d0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gALMultiHash::CallEFun_get_end` · 00562350 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gALMultiHash::CallEFun_remove` · 00562410 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gALMultiHash::CallEFun_add` · 00562490 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

## `portal/shared/src/interp/glue/galmultihashiter.cpp.c` (4/4)

### gALMultiHashIter
- [x] `gALMultiHashIter::CallEFun_data` · 00561b60 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gALMultiHashIter::CallEFun_equals` · 00561bd0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gALMultiHashIter::CallEFun_key` · 00561c50 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gALMultiHashIter::CallEFun_next` · 00561cc0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

## `portal/shared/src/interp/glue/galmultimap.cpp.c` (19/19)

### gALMultiMap
- [x] `gALMultiMap::CallEFun_empty` · 0054e8b0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gALMultiMap::CallEFun_count` · 0054e920 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

### BaseMultiMap<unsigned_long,unsigned___int64,13972>
- [x] `BaseMultiMap<unsigned_long,unsigned___int64,13972>::find` · 0054eb10 · legions 0052b6a0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `BaseMultiMap<unsigned_long,unsigned___int64,13972>::find_last` · 0054eb80 · legions 0052b710 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `BaseMultiMap<unsigned_long,unsigned___int64,13972>::get_begin` · 0054ebe0 · legions 0052b770 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `BaseMultiMap<unsigned_long,unsigned___int64,13972>::get_end` · 0054ec30 · legions 0052b7c0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `BaseMultiMap<unsigned_long,unsigned___int64,13972>::lower_bound` · 0054ec80 · legions 0052b810 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `BaseMultiMap<unsigned_long,unsigned___int64,13972>::upper_bound` · 0054ecf0 · legions 0052b880 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

### gALMultiMap
- [x] `gALMultiMap::CallEFun_clear` · 0054ed60 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gALMultiMap::CallEFun_erase` · 0054ede0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gALMultiMap::CallEFun_find` · 0054ee70 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gALMultiMap::CallEFun_find_last` · 0054eef0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gALMultiMap::CallEFun_get_begin` · 0054ef70 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gALMultiMap::CallEFun_get_end` · 0054eff0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gALMultiMap::CallEFun_lower_bound` · 0054f070 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gALMultiMap::CallEFun_upper_bound` · 0054f0f0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

### BaseMultiMap<unsigned_long,unsigned___int64,13972>
- [x] `BaseMultiMap<unsigned_long,unsigned___int64,13972>::insert` · 0054f170 · legions 0052bbe0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

### gALMultiMap
- [x] `gALMultiMap::CallEFun_insert` · 0054f1d0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gALMultiMap::CallEFun_remove` · 0054f230 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

## `portal/shared/src/interp/glue/galmultimapiter.cpp.c` (5/5)

### gALMultiMapIter
- [x] `gALMultiMapIter::CallEFun_data` · 0054e640 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gALMultiMapIter::CallEFun_equals` · 0054e6b0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gALMultiMapIter::CallEFun_key` · 0054e740 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gALMultiMapIter::CallEFun_next` · 0054e7b0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gALMultiMapIter::CallEFun_prev` · 0054e850 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

## `portal/shared/src/interp/glue/gappearancetableadaptor.cpp.c` (0/5)

### gAppearanceTableAdaptor
- [ ] `gAppearanceTableAdaptor::CallEFun_HasAppearanceKey` · 0054c920

### gCharacterGenSystem
- [ ] `gCharacterGenSystem::ValueListToAList` · 0054c960 · legions 0046b490
- [ ] `gCharacterGenSystem::AListToValueList` · 0054c9d0 · legions 0046b500

### gAppearanceTableAdaptor
- [ ] `gAppearanceTableAdaptor::CallEFun_InqAppearanceKeyModList` · 0054ca40
- [ ] `gAppearanceTableAdaptor::CallEFun_InqTsysAppearanceKeyModCount` · 0054caf0

## `portal/shared/src/interp/glue/gappinfohash.cpp.c` (0/1)

### gAppInfoHash
- [ ] `gAppInfoHash::CallEFun_num_elements` · 0054cb60

## `portal/shared/src/interp/glue/gappliedappearancekey.cpp.c` (0/5)

### gAppliedAppearanceKey
- [ ] `gAppliedAppearanceKey::CallEFun_SetAppKey` · 0054c700
- [ ] `gAppliedAppearanceKey::CallEFun_SetMod` · 0054c760
- [ ] `gAppliedAppearanceKey::CallEFun_FromFileNode` · 0054c7c0
- [ ] `gAppliedAppearanceKey::CallEFun_GetAppKey` · 0054c840
- [ ] `gAppliedAppearanceKey::CallEFun_GetMod` · 0054c8b0

## `portal/shared/src/interp/glue/garhash.cpp.c` (20/20)

### gARHash
- [x] `gARHash::CallEFun_empty` · 00561050 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gARHash::CallEFun_num_buckets` · 005610c0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gARHash::CallEFun_num_elements` · 00561130 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gARHash::CallEFun_too_big` · 005611a0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gARHash::CallEFun_too_small` · 00561220 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gARHash::CallEFun_flush` · 005612a0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gARHash::CallEFun_contains` · 00561300 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gARHash::CallEFun_erase` · 00561390 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gARHash::CallEFun_add` · 00561430 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gARHash::CallEFun_set` · 005614c0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

### BaseHash<unsigned_long,unsigned_long,18>
- [x] `BaseHash<unsigned_long,unsigned_long,18>::get_begin` · 005615f0 · legions 00534f80 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `BaseHash<unsigned_long,unsigned_long,18>::get_end` · 00561670 · legions 00535000 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

### gARHash
- [x] `gARHash::CallEFun_get_begin` · 005616d0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gARHash::CallEFun_get_end` · 00561750 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gARHash::CallEFun_grow` · 005617d0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gARHash::CallEFun_resize` · 00561840 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gARHash::CallEFun_shrink` · 005618f0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gARHash::CallEFun_optimize_size` · 00561960 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gARHash::CallEFun_find` · 005619e0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gARHash::CallEFun_remove` · 00561a90 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

## `portal/shared/src/interp/glue/garhashiter.cpp.c` (6/6)

### gARHashIter
- [x] `gARHashIter::CallEFun_data` · 00560da0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gARHashIter::CallEFun_key` · 00560e10 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gARHashIter::CallEFun_equals` · 00560e80 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gARHashIter::CallEFun_next` · 00560f10 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gARHashIter::CallEFun_notdone` · 00560f70 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gARHashIter::CallEFun_remove` · 00560fe0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

## `portal/shared/src/interp/glue/garmultihash.cpp.c` (14/14)

### gARMultiHash
- [x] `gARMultiHash::CallEFun_empty` · 005604c0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gARMultiHash::CallEFun_size` · 00560530 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gARMultiHash::CallEFun_count` · 00560650 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

### BaseMultiHash<unsigned_long,unsigned_long,20>
- [x] `BaseMultiHash<unsigned_long,unsigned_long,20>::find` · 00560830 · legions 00534a80 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `BaseMultiHash<unsigned_long,unsigned_long,20>::find_last` · 005608b0 · legions 00534b00 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `BaseMultiHash<unsigned_long,unsigned_long,20>::get_begin` · 00560910 · legions 00534b60 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

### gARMultiHash
- [x] `gARMultiHash::CallEFun_clear` · 00560970 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gARMultiHash::CallEFun_erase` · 005609d0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gARMultiHash::CallEFun_find` · 00560a50 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gARMultiHash::CallEFun_find_last` · 00560ad0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gARMultiHash::CallEFun_get_begin` · 00560b50 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gARMultiHash::CallEFun_get_end` · 00560bd0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gARMultiHash::CallEFun_remove` · 00560c90 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gARMultiHash::CallEFun_add` · 00560d10 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

## `portal/shared/src/interp/glue/garmultihashiter.cpp.c` (4/4)

### gARMultiHashIter
- [x] `gARMultiHashIter::CallEFun_data` · 005602f0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gARMultiHashIter::CallEFun_equals` · 00560360 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gARMultiHashIter::CallEFun_key` · 005603e0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gARMultiHashIter::CallEFun_next` · 00560450 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

## `portal/shared/src/interp/glue/gbaseproperty.cpp.c` (0/33)

### BaseProperty
- [ ] `BaseProperty::InqStringW` · 0054b360 · legions 005292d0
- [ ] `BaseProperty::SetStringW` · 0054b430 · legions 005293a0

### gBaseProperty
- [ ] `gBaseProperty::CallEFun_GetGroupName` · 0054b480
- [ ] `gBaseProperty::CallEFun_GetPropertyName` · 0054b4f0
- [ ] `gBaseProperty::CallEFun_GetPropertyType` · 0054b560
- [ ] `gBaseProperty::CallEFun_IsDefaultValue` · 0054b5d0
- [ ] `gBaseProperty::CallEFun_IsReportedToWeenie` · 0054b640
- [ ] `gBaseProperty::CallEFun_SetBool` · 0054b6b0
- [ ] `gBaseProperty::CallEFun_SetDataID` · 0054b740
- [ ] `gBaseProperty::CallEFun_SetEnum` · 0054b7d0
- [ ] `gBaseProperty::CallEFun_SetFloat` · 0054b860
- [ ] `gBaseProperty::CallEFun_SetInteger` · 0054b8f0
- [ ] `gBaseProperty::CallEFun_SetLongInteger` · 0054b980
- [ ] `gBaseProperty::CallEFun_SetPackageID` · 0054ba10
- [ ] `gBaseProperty::CallEFun_SetPosition` · 0054baa0
- [ ] `gBaseProperty::CallEFun_SetPropertyName` · 0054bb30
- [ ] `gBaseProperty::CallEFun_SetPropertyNameW` · 0054bbb0
- [ ] `gBaseProperty::CallEFun_SetStringInfo` · 0054bc30
- [ ] `gBaseProperty::CallEFun_SetStringW` · 0054bcc0
- [ ] `gBaseProperty::CallEFun_SetVector` · 0054bd40
- [ ] `gBaseProperty::CallEFun_InqBool` · 0054bdd0
- [ ] `gBaseProperty::CallEFun_InqDataID` · 0054be90
- [ ] `gBaseProperty::CallEFun_InqEnum` · 0054bf50
- [ ] `gBaseProperty::CallEFun_InqFloat` · 0054c010
- [ ] `gBaseProperty::CallEFun_InqInteger` · 0054c0d0
- [ ] `gBaseProperty::CallEFun_InqLongInteger` · 0054c190
- [ ] `gBaseProperty::CallEFun_InqPackageID` · 0054c250
- [ ] `gBaseProperty::CallEFun_InqPosition` · 0054c310
- [ ] `gBaseProperty::CallEFun_InqPropertyNameW` · 0054c3c0 · legions 00529f30
- [ ] `gBaseProperty::CallEFun_InqPropertyTypeW` · 0054c460
- [ ] `gBaseProperty::CallEFun_InqStringInfo` · 0054c500
- [ ] `gBaseProperty::CallEFun_InqStringW` · 0054c5b0 · legions 0052a0a0
- [ ] `gBaseProperty::CallEFun_InqVector` · 0054c650

## `portal/shared/src/interp/glue/gbehaviorparams.cpp.c` (26/27)

### gBehaviorParams
- [x] `gBehaviorParams::CallEFun_SendEarlyCallBack` · 0054a800 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gBehaviorParams::CallEFun_SetBehaviorID` · 0054a860 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gBehaviorParams::CallEFun_SetCameraBehavior` · 0054a8c0 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gBehaviorParams::CallEFun_SetClonedAppearanceID` · 0054a920 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gBehaviorParams::CallEFun_SetFXScript` · 0054a980 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gBehaviorParams::CallEFun_SetFXToStartAndStop` · 0054a9e0 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gBehaviorParams::CallEFun_SetFadeChildren` · 0054aa40 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gBehaviorParams::CallEFun_SetHoldCamera` · 0054aaa0 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gBehaviorParams::CallEFun_SetLeaveIdleAlone` · 0054ab00 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gBehaviorParams::CallEFun_SetLockActions` · 0054ab60 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gBehaviorParams::CallEFun_SetModeID` · 0054abc0 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gBehaviorParams::CallEFun_SetMoveToCancels` · 0054ac20 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gBehaviorParams::CallEFun_SetNewCameraOwnerID` · 0054ac80 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gBehaviorParams::CallEFun_SetNumberOfHoldCycles` · 0054ace0 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gBehaviorParams::CallEFun_SetParentedCamera` · 0054ad40 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gBehaviorParams::CallEFun_SetPropagate` · 0054ada0 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gBehaviorParams::CallEFun_SetRestoreCamera` · 0054ae00 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gBehaviorParams::CallEFun_SetSendEvent` · 0054ae60 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gBehaviorParams::CallEFun_SetTargetID` · 0054aec0 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gBehaviorParams::CallEFun_SetTargetedCamera` · 0054af20 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gBehaviorParams::CallEFun_SetTargetedImpulse` · 0054af80 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gBehaviorParams::CallEFun_SetTimeScale` · 0054b000 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gBehaviorParams::CallEFun_SetVDescIDToClone` · 0054b060 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gBehaviorParams::CallEFun_SetWeenieEmoteID` · 0054b0c0 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gBehaviorParams::CallEFun_GetBehaviorID` · 0054b120 — ac2-script glue (wlog, random, behavior, enums: gDBCache)

### gGameplayOptionsProfile
- [~] `gGameplayOptionsProfile::AAHashToUIElementOpacities` · 0054b190 · legions 00529120 — ac2-script glue: f32 spill points unchecked against x87 / copy order of PHashTable operator= unverified

### gBehaviorParams
- [x] `gBehaviorParams::CallEFun_SetClonedAppKeyHash` · 0054b280 — ac2-script glue (wlog, random, behavior, enums: gDBCache)

## `portal/shared/src/interp/glue/gcaseinsensitivewpstring.cpp.c` (0/33)

### gCaseInsensitiveWPString
- [ ] `gCaseInsensitiveWPString::CallEFun_clear` · 005500e0
- [ ] `gCaseInsensitiveWPString::CallEFun_cmp` · 00550170
- [ ] `gCaseInsensitiveWPString::CallEFun_empty` · 00550220
- [ ] `gCaseInsensitiveWPString::CallEFun_eq` · 00550290
- [ ] `gCaseInsensitiveWPString::CallEFun_find_substring` · 00550320
- [ ] `gCaseInsensitiveWPString::CallEFun_ge` · 005503c0
- [ ] `gCaseInsensitiveWPString::CallEFun_gt` · 00550450
- [ ] `gCaseInsensitiveWPString::CallEFun_hash` · 005504e0
- [ ] `gCaseInsensitiveWPString::CallEFun_le` · 00550560
- [ ] `gCaseInsensitiveWPString::CallEFun_length` · 005505f0
- [ ] `gCaseInsensitiveWPString::CallEFun_lt` · 00550660
- [ ] `gCaseInsensitiveWPString::CallEFun_ne` · 005506f0
- [ ] `gCaseInsensitiveWPString::CallEFun_set` · 00550780
- [ ] `gCaseInsensitiveWPString::CallEFun_to_float` · 00550810
- [ ] `gCaseInsensitiveWPString::CallEFun_to_int32` · 005508a0
- [ ] `gCaseInsensitiveWPString::CallEFun_to_int64` · 00550930
- [ ] `gCaseInsensitiveWPString::CallEFun_to_uint32` · 005509c0
- [ ] `gCaseInsensitiveWPString::CallEFun_to_uint64` · 00550a50
- [ ] `gCaseInsensitiveWPString::CallEFun_decrypt` · 00550ae0
- [ ] `gCaseInsensitiveWPString::CallEFun_encrypt` · 00550b40
- [ ] `gCaseInsensitiveWPString::CallEFun_tolower` · 00550ba0
- [ ] `gCaseInsensitiveWPString::CallEFun_toupper` · 00550c10
- [ ] `gCaseInsensitiveWPString::CallEFun_append_float` · 00550df0
- [ ] `gCaseInsensitiveWPString::CallEFun_append_int32` · 00550e50
- [ ] `gCaseInsensitiveWPString::CallEFun_append_int32_hex` · 00550eb0
- [ ] `gCaseInsensitiveWPString::CallEFun_append_int64` · 00550f10
- [ ] `gCaseInsensitiveWPString::CallEFun_append_string` · 00550f70
- [ ] `gCaseInsensitiveWPString::CallEFun_append_uint32` · 00550ff0
- [ ] `gCaseInsensitiveWPString::CallEFun_append_uint32_hex` · 00551050
- [ ] `gCaseInsensitiveWPString::CallEFun_trim` · 005510b0
- [ ] `gCaseInsensitiveWPString::CallEFun_append_bool` · 00551130
- [ ] `gCaseInsensitiveWPString::CallEFun_substring` · 00551190
- [ ] `gCaseInsensitiveWPString::CallEFun_replace` · 00551260

## `portal/shared/src/interp/glue/gcdbform.cpp.c` (0/14)

### CDBForm
- [ ] `CDBForm::SetNameW` · 0054a200 · legions 00528910

### gCDBForm
- [ ] `gCDBForm::CallEFun_SetNameW` · 0054a270
- [ ] `gCDBForm::CallEFun_AddTable` · 0054a2d0

### CDBForm
- [ ] `CDBForm::AddPropertyW` · 0054a380 · legions 00528a50
- [ ] `CDBForm::AddProperty` · 0054a3e0 · legions 00528ab0
- [ ] `CDBForm::AddProperty` · 0054a450 · legions 00528b20
- [ ] `CDBForm::AddProperty` · 0054a4c0 · legions 00528b90
- [ ] `CDBForm::AddProperty` · 0054a530 · legions 00528c00

### gCDBForm
- [ ] `gCDBForm::CallEFun_AddFloatProperty` · 0054a5a0
- [ ] `gCDBForm::CallEFun_AddIntProperty` · 0054a600
- [ ] `gCDBForm::CallEFun_AddLongIntProperty` · 0054a660
- [ ] `gCDBForm::CallEFun_AddPropertyW` · 0054a6c0
- [ ] `gCDBForm::CallEFun_AddStringProperty` · 0054a730
- [ ] `gCDBForm::CallEFun_AddTimetypeProperty` · 0054a7a0

## `portal/shared/src/interp/glue/gcollideproperty.cpp.c` (0/6)

### gCollideProperty
- [ ] `gCollideProperty::CallEFun_GetID` · 00549f00
- [ ] `gCollideProperty::CallEFun_IsBegin` · 00549f70
- [ ] `gCollideProperty::CallEFun_IsChanged` · 00549fe0
- [ ] `gCollideProperty::CallEFun_IsEnd` · 0054a050
- [ ] `gCollideProperty::CallEFun_GetProperty` · 0054a0c0
- [ ] `gCollideProperty::CallEFun_GetPropertyOutputStringW` · 0054a170 · legions 005288a0

## `portal/shared/src/interp/glue/gcollisionparameters.cpp.c` (0/25)

### gCollisionParameters
- [ ] `gCollisionParameters::CallEFun_SetMinReportedImpactXY` · 005491f0
- [ ] `gCollisionParameters::CallEFun_SetMinReportedImpactZ` · 00549250
- [ ] `gCollisionParameters::CallEFun_SetMovementHighDWord` · 005492b0
- [ ] `gCollisionParameters::CallEFun_SetMovementLowDWord` · 00549320
- [ ] `gCollisionParameters::CallEFun_SetPhysicsMovementType` · 00549390
- [ ] `gCollisionParameters::CallEFun_SetPhysicsPlacementType` · 00549400
- [ ] `gCollisionParameters::CallEFun_SetPhysicsType` · 00549470
- [ ] `gCollisionParameters::CallEFun_SetPhysicsTypeHighDWord` · 005494e0
- [ ] `gCollisionParameters::CallEFun_SetPhysicsTypeLowDWord` · 00549550
- [ ] `gCollisionParameters::CallEFun_SetPlacementHighDWord` · 005495c0
- [ ] `gCollisionParameters::CallEFun_SetPlacementLowDWord` · 00549630
- [ ] `gCollisionParameters::CallEFun_SetWeenieMovementType` · 005496a0
- [ ] `gCollisionParameters::CallEFun_SetWeeniePlacementType` · 00549710
- [ ] `gCollisionParameters::CallEFun_AddToEtherealList` · 00549780
- [ ] `gCollisionParameters::CallEFun_RemovePhysicsType` · 00549800
- [ ] `gCollisionParameters::CallEFun_UpdateMovementEthereal` · 00549880
- [ ] `gCollisionParameters::CallEFun_UpdateMovementSolid` · 00549900
- [ ] `gCollisionParameters::CallEFun_UpdatePhysicsType` · 00549980
- [ ] `gCollisionParameters::CallEFun_UpdatePlacementEthereal` · 00549a00
- [ ] `gCollisionParameters::CallEFun_UpdatePlacementSolid` · 00549a80
- [ ] `gCollisionParameters::CallEFun_GetPhysicsMovementType` · 00549b00
- [ ] `gCollisionParameters::CallEFun_GetPhysicsPlacementType` · 00549bc0
- [ ] `gCollisionParameters::CallEFun_GetPhysicsType` · 00549c80
- [ ] `gCollisionParameters::CallEFun_GetWeenieMovementType` · 00549d40
- [ ] `gCollisionParameters::CallEFun_GetWeeniePlacementType` · 00549e20

## `portal/shared/src/interp/glue/gcollisionprofile.cpp.c` (0/15)

### gCollisionProfile
- [ ] `gCollisionProfile::CallEFun_GetNumEnvProperties` · 005489a0
- [ ] `gCollisionProfile::CallEFun_GetNumObjectProperties` · 00548a10
- [ ] `gCollisionProfile::CallEFun_GetNumObjects` · 00548a80
- [ ] `gCollisionProfile::CallEFun_GetNumSurfaceProperties` · 00548af0
- [ ] `gCollisionProfile::CallEFun_GetNumWaterProperties` · 00548b60
- [ ] `gCollisionProfile::CallEFun_IsEndCollision` · 00548bd0
- [ ] `gCollisionProfile::CallEFun_IsEndContact` · 00548c40
- [ ] `gCollisionProfile::CallEFun_FindObject` · 00548cb0
- [ ] `gCollisionProfile::CallEFun_GetEnvProperty` · 00548d60
- [ ] `gCollisionProfile::CallEFun_GetObject` · 00548e00 · legions 00527aa0
- [ ] `gCollisionProfile::CallEFun_GetObjectProperty` · 00548ea0
- [ ] `gCollisionProfile::CallEFun_GetSurfaceProperty` · 00548f40
- [ ] `gCollisionProfile::CallEFun_GetWaterProperty` · 00548fe0
- [ ] `gCollisionProfile::CallEFun_HasCollided` · 00549080 · legions 00527ca0
- [ ] `gCollisionProfile::CallEFun_HasContact` · 00549150 · legions 00527d50

## `portal/shared/src/interp/glue/gdbcache.cpp.c` (4/4)

### gDBCache
- [x] `gDBCache::CallEFun_GetDIDFromEnumStatic` · 00536af0 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gDBCache::CallEFun_IsClient` · 00536b30 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gDBCache::CallEFun_IsRunTime` · 00536b60 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gDBCache::CallEFun_IsServer` · 00536b90 — ac2-script glue (wlog, random, behavior, enums: gDBCache)

## `portal/shared/src/interp/glue/gdebug.cpp.c` (0/11)

### gdebug
- [ ] `gdebug::CallEFun_PrintFloat` · 005600a0
- [ ] `gdebug::CallEFun_PrintInt` · 005600c0
- [ ] `gdebug::CallEFun_PrintLongFloat` · 005600e0
- [ ] `gdebug::CallEFun_PrintLongInt` · 00560100
- [ ] `gdebug::CallEFun_PrintString` · 00560120
- [ ] `gdebug::CallEFun_WSLAssert` · 00560150 · legions 00534580
- [ ] `gdebug::CallEFun_WSLAssertFailure` · 00560190 · legions 005345c0
- [ ] `gdebug::CallEFun_WSLAssertMsg` · 005601e0 · legions 00534610
- [ ] `gdebug::CallEFun_WSLDebugger` · 00560230 · legions 00534660
- [ ] `gdebug::CallEFun_MapIIDtoProcID` · 00560270
- [ ] `gdebug::CallEFun_MapProcIDtoIID` · 005602b0

## `portal/shared/src/interp/glue/gdetectionparameters.cpp.c` (0/13)

### gDetectionParameters
- [ ] `gDetectionParameters::CallEFun_SetBasePosition` · 00548390
- [ ] `gDetectionParameters::CallEFun_SetDetectOffset` · 005483f0
- [ ] `gDetectionParameters::CallEFun_SetDetectionObjectType` · 00548490
- [ ] `gDetectionParameters::CallEFun_SetLOSBlockingMasks` · 005484f0
- [ ] `gDetectionParameters::CallEFun_SetShapeAttackCone` · 00548560
- [ ] `gDetectionParameters::CallEFun_SetShapeBox` · 005485f0
- [ ] `gDetectionParameters::CallEFun_SetShapeCylinder` · 00548670
- [ ] `gDetectionParameters::CallEFun_SetShapeSphere` · 005486e0
- [ ] `gDetectionParameters::CallEFun_SetShapeSphereCone` · 00548740
- [ ] `gDetectionParameters::CallEFun_SetShapeSphereWithOffset` · 005487c0
- [ ] `gDetectionParameters::CallEFun_UseDetectingObjectForBasePosition` · 00548830
- [ ] `gDetectionParameters::CallEFun_AddExcludeObject` · 005488a0
- [ ] `gDetectionParameters::CallEFun_AddTarget` · 00548920

## `portal/shared/src/interp/glue/gdetectionprofile.cpp.c` (0/9)

### gDetectionProfile
- [ ] `gDetectionProfile::CallEFun_SetFloatApp` · 00547ee0
- [ ] `gDetectionProfile::CallEFun_SetIntApp` · 00547f40
- [ ] `gDetectionProfile::CallEFun_GetCoverage` · 00547fa0
- [ ] `gDetectionProfile::CallEFun_GetEntityDID` · 00548010
- [ ] `gDetectionProfile::CallEFun_GetFloatApp` · 00548080
- [ ] `gDetectionProfile::CallEFun_GetID` · 00548100
- [ ] `gDetectionProfile::CallEFun_GetIntApp` · 00548170
- [ ] `gDetectionProfile::CallEFun_GetDetectionReportType` · 005481f0
- [ ] `gDetectionProfile::CallEFun_GetPosition` · 005482a0

## `portal/shared/src/interp/glue/gdetectionreporttype.cpp.c` (0/4)

### gDetectionReportType
- [ ] `gDetectionReportType::CallEFun_SetHighDWord` · 00547d10
- [ ] `gDetectionReportType::CallEFun_SetLowDWord` · 00547d80
- [ ] `gDetectionReportType::CallEFun_GetHighDWord` · 00547df0
- [ ] `gDetectionReportType::CallEFun_GetLowDWord` · 00547e70

## `portal/shared/src/interp/glue/gefuntestsystem.cpp.c` (0/2)

### EFunTestSystem
- [ ] `EFunTestSystem::Run` · 00560000 · legions 00534430

### gEFunTestSystem
- [ ] `gEFunTestSystem::CallEFun_Run` · 00560070

## `portal/shared/src/interp/glue/gemailutils.cpp.c` (0/1)

### gEmailUtils
- [ ] `gEmailUtils::CallEFun_send_emailW` · 00547c90

## `portal/shared/src/interp/glue/gentitydesc.cpp.c` (1/28)

### gEntityDesc
- [ ] `gEntityDesc::CallEFun_GetDataID` · 00546c10
- [x] `gEntityDesc::CallEFun_GetDataIDFromEntityNameW` · 00546c80 — ac2-script glue::enums (FileArray::GetDID: no name data on the DVD)
- [ ] `gEntityDesc::CallEFun_GetRootPackageID` · 00546cc0
- [ ] `gEntityDesc::CallEFun_GetRuntimeID` · 00546d30
- [ ] `gEntityDesc::CallEFun_GetType` · 00546da0
- [ ] `gEntityDesc::CallEFun_GetVersion` · 00546e10
- [ ] `gEntityDesc::CallEFun_GetWBID` · 00546e80
- [ ] `gEntityDesc::CallEFun_IsInside` · 00546f00
- [ ] `gEntityDesc::CallEFun_SetProperties` · 00546f70
- [ ] `gEntityDesc::CallEFun_GetCommentsW` · 00546ff0
- [ ] `gEntityDesc::CallEFun_GetEntityNameFromDataIDW` · 005470b0 · legions 00526470
- [ ] `gEntityDesc::CallEFun_GetOffset` · 00547130
- [ ] `gEntityDesc::CallEFun_GetProperties` · 00547210
- [ ] `gEntityDesc::CallEFun_GetScale` · 005472f0
- [ ] `gEntityDesc::CallEFun_GetWBNameW` · 005473b0
- [ ] `gEntityDesc::CallEFun_InqBool` · 00547480
- [ ] `gEntityDesc::CallEFun_InqDataFilePathW` · 00547530 · legions 00526830
- [ ] `gEntityDesc::CallEFun_InqDataFileW` · 005475d0 · legions 005268a0
- [ ] `gEntityDesc::CallEFun_InqDataID` · 00547670
- [ ] `gEntityDesc::CallEFun_InqEntityDescFromDataID` · 00547720
- [ ] `gEntityDesc::CallEFun_InqEntityDescFromNameW` · 00547780
- [ ] `gEntityDesc::CallEFun_InqEnum` · 005477f0
- [ ] `gEntityDesc::CallEFun_InqFloat` · 005478a0
- [ ] `gEntityDesc::CallEFun_InqInteger` · 00547950
- [ ] `gEntityDesc::CallEFun_InqPackageID` · 00547a00
- [ ] `gEntityDesc::CallEFun_InqStringInfo` · 00547ab0
- [ ] `gEntityDesc::CallEFun_InqStringW` · 00547b50 · legions 00526d30
- [ ] `gEntityDesc::CallEFun_InqVector` · 00547bf0

## `portal/shared/src/interp/glue/gentitylinkdesc.cpp.c` (0/7)

### gEntityLinkDesc
- [ ] `gEntityLinkDesc::CallEFun_GetInputID` · 005366d0
- [ ] `gEntityLinkDesc::CallEFun_GetLinkType` · 00536740
- [ ] `gEntityLinkDesc::CallEFun_GetOppositeID` · 005367b0
- [ ] `gEntityLinkDesc::CallEFun_GetOutputID` · 00536840
- [ ] `gEntityLinkDesc::CallEFun_GetProperties` · 005368b0
- [ ] `gEntityLinkDesc::CallEFun_GetWBName_ForWsl` · 00536990
- [ ] `gEntityLinkDesc::CallEFun_InqStringPropertyW` · 00536a50 · legions 0051a7e0

## `portal/shared/src/interp/glue/genummapper.cpp.c` (8/8)

### gEnumMapper
- [x] `gEnumMapper::CallEFun_GetNumMappings` · 00546820 — ac2-script data/glue::enums
- [x] `gEnumMapper::CallEFun_GetAllEnumVals` · 00546850 — ac2-script data/glue::enums
- [x] `gEnumMapper::CallEFun_GetLogStringW` · 00546900 · legions 00525d50 — ac2-script data/glue::enums
- [x] `gEnumMapper::CallEFun_GetStringW` · 00546980 · legions 00525dd0 — ac2-script data/glue::enums
- [x] `gEnumMapper::CallEFun_GetAllEnumNames` · 00546a00 — ac2-script data/glue::enums
- [x] `gEnumMapper::CallEFun_GetEnumByDIDW` · 00546ab0 — ac2-script data/glue::enums
- [x] `gEnumMapper::CallEFun_InqEnum` · 00546b30 — ac2-script data/glue::enums
- [x] `gEnumMapper::CallEFun_InqString` · 00546bb0 — ac2-script data/glue::enums

## `portal/shared/src/interp/glue/getherealphysicstype.cpp.c` (0/7)

### gEtherealPhysicsType
- [ ] `gEtherealPhysicsType::CallEFun_SetEtherealToWaterLayer` · 00546520
- [ ] `gEtherealPhysicsType::CallEFun_SetSolidOnlyToEnvironment` · 00546580
- [ ] `gEtherealPhysicsType::CallEFun_SetToEthereal` · 005465e0
- [ ] `gEtherealPhysicsType::CallEFun_SetToSolid` · 00546640
- [ ] `gEtherealPhysicsType::CallEFun_SetProperty` · 005466a0
- [ ] `gEtherealPhysicsType::CallEFun_SetPropertyEthereal` · 00546720
- [ ] `gEtherealPhysicsType::CallEFun_SetPropertySolid` · 005467a0

## `portal/shared/src/interp/glue/getherealweenietype.cpp.c` (0/4)

### gEtherealWeenieType
- [ ] `gEtherealWeenieType::CallEFun_SetToEthereal` · 00546360
- [ ] `gEtherealWeenieType::CallEFun_SetToSolid` · 005463c0
- [ ] `gEtherealWeenieType::CallEFun_AddToEtherealList` · 00546420
- [ ] `gEtherealWeenieType::CallEFun_AddToSolidList` · 005464a0

## `portal/shared/src/interp/glue/gfile.cpp.c` (0/15)

### WSLFilesCleaner
- [ ] `WSLFilesCleaner::~WSLFilesCleaner` · 005455b0 · legions 00524c10

### SmartArray<PStringBase<unsigned_short>_>
- [ ] `SmartArray<PStringBase<unsigned_short>_>::AddToEnd` · 00545670 · legions 00524c70

### (free functions)
- [ ] `WSLCloseFile` · 005456f0 · legions 00524cf0

### gFile
- [ ] `gFile::CallEFun_WSLCloseFile` · 00545790
- [ ] `gFile::RArrayToWPStringArray` · 005457e0 · legions 00524de0

### (free functions)
- [ ] `WSLOpenFile` · 00545910 · legions 00524f10
- [ ] `WSLReadFromFile` · 00545ab0 · legions 005250b0

### gFile
- [ ] `gFile::CallEFun_WSLOpenFile` · 00545cb0
- [ ] `gFile::WPStringArrayToRArray` · 00545d10 · legions 00525310

### (free functions)
- [ ] `WriteToFile` · 00545db0 · legions 005253b0
- [ ] `WSLWriteToFileTimestamped` · 00545ec0 · legions 005254c0
- [ ] `WSLWriteToFile` · 00546070 · legions 00525670

### gFile
- [ ] `gFile::CallEFun_WSLWriteToFile` · 005461c0
- [ ] `gFile::CallEFun_WSLWriteToFileTimestamped` · 00546230
- [ ] `gFile::CallEFun_WSLReadFromFile` · 005462a0

## `portal/shared/src/interp/glue/gfilearray.cpp.c` (1/4)

### gFileArray
- [~] `gFileArray::CallEFun_GetDIDForDummiesW` · 00545420 — ac2-script: FileArray has no data in Legions (no READONLY_DIDName.dat); DivineType by DID range
- [x] `gFileArray::CallEFun_GetDIDFromEnum` · 00545460 — ac2-script data/glue::enums
- [~] `gFileArray::CallEFun_GetDIDW` · 005454a0 — ac2-script: FileArray has no data in Legions (no READONLY_DIDName.dat); DivineType by DID range
- [~] `gFileArray::CallEFun_GetFileNameForDummiesW` · 005454f0 · legions 00524b00 — ac2-script: FileArray has no data in Legions (no READONLY_DIDName.dat); DivineType by DID range

## `portal/shared/src/interp/glue/gframe.cpp.c` (0/1)

### gFrame
- [ ] `gFrame::CallEFun_SetOrigin` · 00543cf0

## `portal/shared/src/interp/glue/gfxandbehaviorinfo.cpp.c` (0/5)

### gFXAndBehaviorInfo
- [ ] `gFXAndBehaviorInfo::CallEFun_SetAttackerFX` · 005439f0
- [ ] `gFXAndBehaviorInfo::CallEFun_SetTargetBehavior` · 00543a50
- [ ] `gFXAndBehaviorInfo::CallEFun_SetTargetFX` · 00543ab0
- [ ] `gFXAndBehaviorInfo::CallEFun_SetWeaponBehavior` · 00543b10
- [ ] `gFXAndBehaviorInfo::CallEFun_SetWeaponFX` · 00543b70

## `portal/shared/src/interp/glue/ggametime.cpp.c` (0/10)

### gGameTime
- [ ] `gGameTime::CallEFun_AlterTimeOfDayImmediate` · 00543630
- [ ] `gGameTime::CallEFun_GetDayInYear` · 00543670
- [ ] `gGameTime::CallEFun_GetDayLength` · 005436e0
- [ ] `gGameTime::CallEFun_GetTimeOfDay` · 00543750
- [ ] `gGameTime::CallEFun_GetYear` · 005437c0
- [ ] `gGameTime::CallEFun_GetYearLength` · 00543830
- [ ] `gGameTime::CallEFun_InqCurrentGameDateTimeString` · 005438a0
- [ ] `gGameTime::CallEFun_InqGameDateTimeString` · 005438f0
- [ ] `gGameTime::CallEFun_IsNight` · 00543960
- [ ] `gGameTime::CallEFun_TemporalDilation` · 005439b0

## `portal/shared/src/interp/glue/gheading.cpp.c` (0/5)

### gHeading
- [ ] `gHeading::CallEFun_set_degrees` · 005433d0
- [ ] `gHeading::CallEFun_get_degrees` · 00543430
- [ ] `gHeading::CallEFun_get_degrees_0_centered` · 005434a0
- [ ] `gHeading::CallEFun_is_in_range` · 00543510
- [ ] `gHeading::CallEFun_ToWPString` · 005435b0 · legions 005233a0

## `portal/shared/src/interp/glue/gicondesc.cpp.c` (0/2)

### gIconDesc
- [ ] `gIconDesc::CallEFun_Empty` · 005432f0
- [ ] `gIconDesc::CallEFun_IsValid` · 00543360

## `portal/shared/src/interp/glue/gifuntestsystem.cpp.c` (0/2)

### gIFunTestSystem
- [ ] `gIFunTestSystem::CallIFun_GetFoo` · 00567e60 · legions 00537df0
- [ ] `gIFunTestSystem::CallIFun_SetFoo` · 00567f50 · legions 00537ee0

## `portal/shared/src/interp/glue/gifunvirtualtestsystem.cpp.c` (0/1)

### gIFunVirtualTestSystem
- [ ] `gIFunVirtualTestSystem::CallIFun_VirtualIFun` · 00567dd0 · legions 00537d60

## `portal/shared/src/interp/glue/gipreproc.cpp.c` (0/2)

### _STL::_Rb_tree<QualifiedDataID,QualifiedDataID,_STL::_Identity<QualifiedDataID>,_STL::less<QualifiedDataID>,_STL::allocator<QualifiedDataID>_>
- [ ] `_STL::_Rb_tree<QualifiedDataID,QualifiedDataID,_STL::_Identity<QualifiedDataID>,_STL::less<QualifiedDataID>,_STL::allocator<QualifiedDataID>_>::operator=` · 007337b0 · legions 006afc60

### gIPreproc
- [ ] `gIPreproc::CallIFun_GetSubDataIDs` · 00733870 · legions 006afd20

## `portal/shared/src/interp/glue/glahash.cpp.c` (20/20)

### gLAHash
- [x] `gLAHash::CallEFun_empty` · 0055f4f0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLAHash::CallEFun_num_buckets` · 0055f560 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLAHash::CallEFun_num_elements` · 0055f5d0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLAHash::CallEFun_too_big` · 0055f640 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLAHash::CallEFun_too_small` · 0055f6c0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLAHash::CallEFun_flush` · 0055f740 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLAHash::CallEFun_contains` · 0055f7a0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLAHash::CallEFun_erase` · 0055f840 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLAHash::CallEFun_add` · 0055f8f0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLAHash::CallEFun_set` · 0055f990 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

### BaseHash<unsigned___int64,unsigned_long,51>
- [x] `BaseHash<unsigned___int64,unsigned_long,51>::get_begin` · 0055fa30 · legions 00534080 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `BaseHash<unsigned___int64,unsigned_long,51>::get_end` · 0055fab0 · legions 00534100 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

### gLAHash
- [x] `gLAHash::CallEFun_get_begin` · 0055fb10 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLAHash::CallEFun_get_end` · 0055fb90 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLAHash::CallEFun_grow` · 0055fc10 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLAHash::CallEFun_resize` · 0055fc80 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLAHash::CallEFun_shrink` · 0055fd30 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLAHash::CallEFun_optimize_size` · 0055fda0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLAHash::CallEFun_find` · 0055fe20 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLAHash::CallEFun_remove` · 0055ff10 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

## `portal/shared/src/interp/glue/glahashiter.cpp.c` (6/6)

### gLAHashIter
- [x] `gLAHashIter::CallEFun_data` · 0055f240 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLAHashIter::CallEFun_key` · 0055f2b0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLAHashIter::CallEFun_equals` · 0055f320 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLAHashIter::CallEFun_notdone` · 0055f3b0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLAHashIter::CallEFun_next` · 0055f420 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLAHashIter::CallEFun_remove` · 0055f480 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

## `portal/shared/src/interp/glue/glahashset.cpp.c` (17/20)

### gLAHashSet
- [x] `gLAHashSet::CallEFun_empty` · 00551530 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLAHashSet::CallEFun_num_buckets` · 005515a0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLAHashSet::CallEFun_num_elements` · 00551610 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLAHashSet::CallEFun_too_big` · 00551680 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLAHashSet::CallEFun_too_small` · 00551700 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLAHashSet::CallEFun_flush` · 00551780 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLAHashSet::CallEFun_contains` · 005517e0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLAHashSet::CallEFun_erase` · 00551880 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

### HashSet<unsigned___int64>
- [ ] `HashSet<unsigned___int64>::add` · 00551930 · legions 0052ccf0

### gLAHashSet
- [x] `gLAHashSet::CallEFun_add` · 005519a0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

### HashSet<unsigned___int64>
- [ ] `HashSet<unsigned___int64>::grow` · 00551a30 · legions 0052cdd0
- [ ] `HashSet<unsigned___int64>::shrink` · 00551a80 · legions 0052ce20

### BaseHashSet<unsigned___int64,11788>
- [x] `BaseHashSet<unsigned___int64,11788>::get_begin` · 00551b30 · legions 0052ce70 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `BaseHashSet<unsigned___int64,11788>::get_end` · 00551bb0 · legions 0052cef0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

### gLAHashSet
- [x] `gLAHashSet::CallEFun_get_begin` · 00551c10 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLAHashSet::CallEFun_get_end` · 00551c90 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLAHashSet::CallEFun_grow` · 00551d10 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLAHashSet::CallEFun_resize` · 00551d70 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLAHashSet::CallEFun_shrink` · 00551e00 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLAHashSet::CallEFun_optimize_size` · 00551e60 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

## `portal/shared/src/interp/glue/glahashsetiter.cpp.c` (5/5)

### gLAHashSetIter
- [x] `gLAHashSetIter::CallEFun_equals` · 005512f0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLAHashSetIter::CallEFun_key` · 00551380 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLAHashSetIter::CallEFun_notdone` · 005513f0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLAHashSetIter::CallEFun_next` · 00551460 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLAHashSetIter::CallEFun_remove` · 005514c0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

## `portal/shared/src/interp/glue/glamultihash.cpp.c` (13/13)

### gLAMultiHash
- [x] `gLAMultiHash::CallEFun_size` · 0055ebb0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLAMultiHash::CallEFun_count` · 0055ec20 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

### BaseMultiHash<unsigned___int64,unsigned_long,53>
- [x] `BaseMultiHash<unsigned___int64,unsigned_long,53>::find` · 0055ecb0 · legions 00533a70 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `BaseMultiHash<unsigned___int64,unsigned_long,53>::find_last` · 0055ed40 · legions 00533b00 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `BaseMultiHash<unsigned___int64,unsigned_long,53>::get_begin` · 0055eda0 · legions 00533b60 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

### gLAMultiHash
- [x] `gLAMultiHash::CallEFun_clear` · 0055ee00 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLAMultiHash::CallEFun_erase` · 0055ee60 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLAMultiHash::CallEFun_find` · 0055eee0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLAMultiHash::CallEFun_find_last` · 0055ef60 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLAMultiHash::CallEFun_get_begin` · 0055efe0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLAMultiHash::CallEFun_get_end` · 0055f060 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLAMultiHash::CallEFun_remove` · 0055f120 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLAMultiHash::CallEFun_add` · 0055f1b0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

## `portal/shared/src/interp/glue/glamultihashiter.cpp.c` (4/4)

### gLAMultiHashIter
- [x] `gLAMultiHashIter::CallEFun_data` · 0055e9e0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLAMultiHashIter::CallEFun_equals` · 0055ea50 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLAMultiHashIter::CallEFun_key` · 0055ead0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLAMultiHashIter::CallEFun_next` · 0055eb40 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

## `portal/shared/src/interp/glue/glamultimap.cpp.c` (18/18)

### gLAMultiMap
- [x] `gLAMultiMap::CallEFun_empty` · 0054dbf0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLAMultiMap::CallEFun_count` · 0054dcf0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

### BaseMultiMap<unsigned___int64,unsigned_long,13974>
- [x] `BaseMultiMap<unsigned___int64,unsigned_long,13974>::find` · 0054def0 · legions 0052af90 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `BaseMultiMap<unsigned___int64,unsigned_long,13974>::find_last` · 0054df50 · legions 0052aff0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `BaseMultiMap<unsigned___int64,unsigned_long,13974>::get_begin` · 0054dfb0 · legions 0052b050 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `BaseMultiMap<unsigned___int64,unsigned_long,13974>::get_end` · 0054e000 · legions 0052b0a0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `BaseMultiMap<unsigned___int64,unsigned_long,13974>::lower_bound` · 0054e050 · legions 0052b0f0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `BaseMultiMap<unsigned___int64,unsigned_long,13974>::upper_bound` · 0054e0b0 · legions 0052b150 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

### gLAMultiMap
- [x] `gLAMultiMap::CallEFun_clear` · 0054e130 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLAMultiMap::CallEFun_erase` · 0054e1b0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLAMultiMap::CallEFun_find` · 0054e240 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLAMultiMap::CallEFun_find_last` · 0054e2c0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLAMultiMap::CallEFun_get_begin` · 0054e340 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLAMultiMap::CallEFun_get_end` · 0054e3c0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLAMultiMap::CallEFun_lower_bound` · 0054e440 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLAMultiMap::CallEFun_upper_bound` · 0054e4c0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLAMultiMap::CallEFun_insert` · 0054e540 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLAMultiMap::CallEFun_remove` · 0054e5c0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

## `portal/shared/src/interp/glue/glamultimapiter.cpp.c` (5/6)

### _STL::_Rb_tree_iterator<_STL::pair<ActionWithModifiers_const_,ActionMapValue>,_STL::_Nonconst_traits<_STL::pair<ActionWithModifiers_const_,ActionMapValue>_>_>
- [ ] `_STL::_Rb_tree_iterator<_STL::pair<ActionWithModifiers_const_,ActionMapValue>,_STL::_Nonconst_traits<_STL::pair<ActionWithModifiers_const_,ActionMapValue>_>_>::operator--` · 0054d8e0 · legions 0060f180

### gLAMultiMapIter
- [x] `gLAMultiMapIter::CallEFun_data` · 0054d930 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLAMultiMapIter::CallEFun_equals` · 0054d9a0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLAMultiMapIter::CallEFun_key` · 0054da30 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLAMultiMapIter::CallEFun_next` · 0054daa0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLAMultiMapIter::CallEFun_prev` · 0054db40 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

## `portal/shared/src/interp/glue/glanddefs.cpp.c` (0/3)

### gLandDefs
- [ ] `gLandDefs::CallEFun_get_land_length` · 00543220
- [ ] `gLandDefs::CallEFun_get_land_width` · 00543250
- [ ] `gLandDefs::CallEFun_InqLandblockIIDFromStaticObjectIID` · 00543280

## `portal/shared/src/interp/glue/glarray.cpp.c` (12/13)

### gLArray
- [x] `gLArray::CallEFun_contains` · 0055e270 — ac2-script glue::lists
- [x] `gLArray::CallEFun_empty` · 0055e300 — ac2-script glue::lists
- [x] `gLArray::CallEFun_flush` · 0055e370 — ac2-script glue::lists
- [x] `gLArray::CallEFun_num_elements` · 0055e3d0 — ac2-script glue::lists
- [x] `gLArray::CallEFun_set` · 0055e440 — ac2-script glue::lists
- [x] `gLArray::CallEFun_SetNElements` · 0055e4e0 — ac2-script glue::lists
- [x] `gLArray::CallEFun_erase` · 0055e570 — ac2-script glue::lists
- [x] `gLArray::CallEFun_get` · 0055e610 — ac2-script glue::lists

### SmartArray<CLandBlock::DynamicObjectInfo>
- [ ] `SmartArray<CLandBlock::DynamicObjectInfo>::AddToEnd` · 0055e690 · legions 00533650

### gLArray
- [x] `gLArray::CallEFun_push` · 0055e6e0 — ac2-script glue::lists
- [x] `gLArray::CallEFun_push_unique` · 0055e770 — ac2-script glue::lists
- [x] `gLArray::CallEFun_pop` · 0055e840 — ac2-script glue::lists
- [x] `gLArray::CallEFun_query` · 0055e910 — ac2-script glue::lists

## `portal/shared/src/interp/glue/glinkrequirements.cpp.c` (0/10)

### gLinkRequirements
- [ ] `gLinkRequirements::CallEFun_SetInputLink` · 00542e10
- [ ] `gLinkRequirements::CallEFun_SetLinkType` · 00542e70
- [ ] `gLinkRequirements::CallEFun_SetNumLinks` · 00542ed0
- [ ] `gLinkRequirements::CallEFun_SetNumReqLinks` · 00542f30
- [ ] `gLinkRequirements::CallEFun_SetRequired` · 00542f90
- [ ] `gLinkRequirements::CallEFun_GetLinkType` · 00542ff0
- [ ] `gLinkRequirements::CallEFun_GetNumLinks` · 00543060
- [ ] `gLinkRequirements::CallEFun_GetNumReqLinks` · 005430d0
- [ ] `gLinkRequirements::CallEFun_IsInputLink` · 00543140
- [ ] `gLinkRequirements::CallEFun_IsRequired` · 005431b0

## `portal/shared/src/interp/glue/gllhash.cpp.c` (20/20)

### gLLHash
- [x] `gLLHash::CallEFun_empty` · 0055d690 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLLHash::CallEFun_num_buckets` · 0055d700 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLLHash::CallEFun_num_elements` · 0055d770 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLLHash::CallEFun_too_big` · 0055d7e0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLLHash::CallEFun_too_small` · 0055d860 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLLHash::CallEFun_flush` · 0055d8e0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLLHash::CallEFun_contains` · 0055d940 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLLHash::CallEFun_erase` · 0055d9e0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLLHash::CallEFun_add` · 0055db00 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLLHash::CallEFun_set` · 0055dba0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

### BaseHash<unsigned___int64,unsigned___int64,58>
- [x] `BaseHash<unsigned___int64,unsigned___int64,58>::get_begin` · 0055dc40 · legions 00533010 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `BaseHash<unsigned___int64,unsigned___int64,58>::get_end` · 0055dcc0 · legions 00533090 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

### gLLHash
- [x] `gLLHash::CallEFun_get_begin` · 0055dd20 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLLHash::CallEFun_get_end` · 0055dda0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLLHash::CallEFun_grow` · 0055de20 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLLHash::CallEFun_resize` · 0055de90 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLLHash::CallEFun_shrink` · 0055df40 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLLHash::CallEFun_optimize_size` · 0055dfb0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLLHash::CallEFun_find` · 0055e030 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLLHash::CallEFun_remove` · 0055e130 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

## `portal/shared/src/interp/glue/gllhashiter.cpp.c` (6/7)

### gLLHashIter
- [x] `gLLHashIter::CallEFun_data` · 0055d380 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLLHashIter::CallEFun_key` · 0055d3f0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

### IntrusiveHashIterator<unsigned___int64,HashSetData<unsigned___int64>_*>
- [ ] `IntrusiveHashIterator<unsigned___int64,HashSetData<unsigned___int64>_*>::remove` · 0055d460 · legions 00531370

### gLLHashIter
- [x] `gLLHashIter::CallEFun_equals` · 0055d4c0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLLHashIter::CallEFun_notdone` · 0055d550 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLLHashIter::CallEFun_next` · 0055d5c0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLLHashIter::CallEFun_remove` · 0055d620 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

## `portal/shared/src/interp/glue/gllist.cpp.c` (46/51)

### gLList
- [x] `gLList::CallEFun_empty` · 0055b450 — ac2-script glue::lists
- [x] `gLList::CallEFun_get_position` · 0055b4c0 — ac2-script glue::lists
- [x] `gLList::CallEFun_num_elements` · 0055b560 — ac2-script glue::lists
- [x] `gLList::CallEFun_push_back` · 0055b5d0 — ac2-script glue::lists
- [x] `gLList::CallEFun_push_front` · 0055b660 — ac2-script glue::lists

### BaseList<unsigned___int64,60>
- [ ] `BaseList<unsigned___int64,60>::insert_sort_ulint` · 0055b810 · legions 00532260

### gLList
- [x] `gLList::CallEFun_insert_back` · 0055b860 — ac2-script glue::lists
- [x] `gLList::CallEFun_insert_front` · 0055b8f0 — ac2-script glue::lists
- [x] `gLList::CallEFun_insert_head` · 0055b980 — ac2-script glue::lists
- [x] `gLList::CallEFun_insert_sort_ulint` · 0055ba10 — ac2-script glue::lists
- [x] `gLList::CallEFun_insert_tail` · 0055baa0 — ac2-script glue::lists
- [x] `gLList::CallEFun_push` · 0055bb30 — ac2-script glue::lists
- [x] `gLList::CallEFun_push_head` · 0055bbc0 — ac2-script glue::lists
- [x] `gLList::CallEFun_push_tail` · 0055bc50 — ac2-script glue::lists
- [x] `gLList::CallEFun_remove` · 0055bce0 — ac2-script glue::lists

### BaseList<unsigned___int64,60>
- [ ] `BaseList<unsigned___int64,60>::inq_data` · 0055be10 · legions 00532510

### gLList
- [x] `gLList::CallEFun_contains` · 0055be60 — ac2-script glue::lists
- [x] `gLList::CallEFun_erase_back` · 0055bf00 — ac2-script glue::lists
- [x] `gLList::CallEFun_erase_front` · 0055bfb0 — ac2-script glue::lists
- [x] `gLList::CallEFun_erase_head` · 0055c060 — ac2-script glue::lists
- [x] `gLList::CallEFun_erase_tail` · 0055c110 — ac2-script glue::lists
- [x] `gLList::CallEFun_flush` · 0055c1c0 — ac2-script glue::lists
- [x] `gLList::CallEFun_insert` · 0055c220 — ac2-script glue::lists
- [x] `gLList::CallEFun_push_unique_back` · 0055c340 — ac2-script glue::lists
- [x] `gLList::CallEFun_push_unique_front` · 0055c3d0 — ac2-script glue::lists
- [x] `gLList::CallEFun_insert_unique_back` · 0055c460 — ac2-script glue::lists
- [x] `gLList::CallEFun_insert_unique_front` · 0055c4f0 — ac2-script glue::lists
- [x] `gLList::CallEFun_insert_unique_head` · 0055c580 — ac2-script glue::lists
- [x] `gLList::CallEFun_insert_unique_tail` · 0055c610 — ac2-script glue::lists
- [x] `gLList::CallEFun_push_unique_head` · 0055c6a0 — ac2-script glue::lists
- [x] `gLList::CallEFun_push_unique_tail` · 0055c730 — ac2-script glue::lists

### BaseList<unsigned___int64,60>
- [ ] `BaseList<unsigned___int64,60>::get_begin` · 0055c7c0 · legions 005328f0
- [ ] `BaseList<unsigned___int64,60>::get_reverse` · 0055c810 · legions 00532940
- [ ] `BaseList<unsigned___int64,60>::get_end` · 0055c860 · legions 00532990

### gLList
- [x] `gLList::CallEFun_get_begin` · 0055c8b0 — ac2-script glue::lists
- [x] `gLList::CallEFun_get_end` · 0055c930 — ac2-script glue::lists
- [x] `gLList::CallEFun_get_reverse` · 0055c9b0 — ac2-script glue::lists
- [x] `gLList::CallEFun_clone` · 0055ca30 · legions 00532ad0 — ac2-script glue::lists
- [x] `gLList::CallEFun_inq_data` · 0055cad0 · legions 00532b40 — ac2-script glue::lists
- [x] `gLList::CallEFun_peek_back` · 0055cb80 — ac2-script glue::lists
- [x] `gLList::CallEFun_peek_front` · 0055cc40 — ac2-script glue::lists
- [x] `gLList::CallEFun_peek_head` · 0055cd00 — ac2-script glue::lists
- [x] `gLList::CallEFun_peek_tail` · 0055cdc0 — ac2-script glue::lists
- [x] `gLList::CallEFun_pop_back` · 0055ce80 — ac2-script glue::lists
- [x] `gLList::CallEFun_pop_front` · 0055cf20 — ac2-script glue::lists
- [x] `gLList::CallEFun_pop_head` · 0055cfc0 — ac2-script glue::lists
- [x] `gLList::CallEFun_pop_tail` · 0055d060 — ac2-script glue::lists
- [x] `gLList::CallEFun_remove_back` · 0055d100 — ac2-script glue::lists
- [x] `gLList::CallEFun_remove_front` · 0055d1a0 — ac2-script glue::lists
- [x] `gLList::CallEFun_remove_head` · 0055d240 — ac2-script glue::lists
- [x] `gLList::CallEFun_remove_tail` · 0055d2e0 — ac2-script glue::lists

## `portal/shared/src/interp/glue/gllistiter.cpp.c` (0/7)

### gLListIter
- [~] `gLListIter::CallEFun_next` · 0055b0b0 — ac2-script glue::lists: iterators hold an index, not a node
- [~] `gLListIter::CallEFun_prev` · 0055b110 — ac2-script glue::lists: iterators hold an index, not a node
- [~] `gLListIter::CallEFun_data` · 0055b170 — ac2-script glue::lists: iterators hold an index, not a node
- [~] `gLListIter::CallEFun_equals` · 0055b1e0 — ac2-script glue::lists: iterators hold an index, not a node
- [~] `gLListIter::CallEFun_notdone` · 0055b260 — ac2-script glue::lists: iterators hold an index, not a node
- [~] `gLListIter::CallEFun_insert` · 0055b2d0 — ac2-script glue::lists: iterators hold an index, not a node
- [~] `gLListIter::CallEFun_remove` · 0055b390 — ac2-script glue::lists: iterators hold an index, not a node

## `portal/shared/src/interp/glue/gllmultihash.cpp.c` (14/14)

### gLLMultiHash
- [x] `gLLMultiHash::CallEFun_size` · 0055a8d0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLLMultiHash::CallEFun_count` · 0055aa60 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

### BaseMultiHash<unsigned___int64,unsigned___int64,62>
- [x] `BaseMultiHash<unsigned___int64,unsigned___int64,62>::find` · 0055aaf0 · legions 00531ac0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `BaseMultiHash<unsigned___int64,unsigned___int64,62>::find_last` · 0055ab80 · legions 00531b50 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `BaseMultiHash<unsigned___int64,unsigned___int64,62>::get_begin` · 0055abe0 · legions 00531bb0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

### gLLMultiHash
- [x] `gLLMultiHash::CallEFun_clear` · 0055ac40 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLLMultiHash::CallEFun_erase` · 0055aca0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLLMultiHash::CallEFun_find` · 0055ad20 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLLMultiHash::CallEFun_find_last` · 0055ada0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLLMultiHash::CallEFun_get_begin` · 0055ae20 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLLMultiHash::CallEFun_get_end` · 0055aea0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLLMultiHash::CallEFun_remove` · 0055af60 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

### BaseMultiHash<unsigned___int64,unsigned___int64,62>
- [x] `BaseMultiHash<unsigned___int64,unsigned___int64,62>::add` · 0055aff0 · legions 00531db0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

### gLLMultiHash
- [x] `gLLMultiHash::CallEFun_add` · 0055b040 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

## `portal/shared/src/interp/glue/gllmultihashiter.cpp.c` (4/4)

### gLLMultiHashIter
- [x] `gLLMultiHashIter::CallEFun_data` · 0055a700 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLLMultiHashIter::CallEFun_equals` · 0055a770 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLLMultiHashIter::CallEFun_key` · 0055a7f0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLLMultiHashIter::CallEFun_next` · 0055a860 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

## `portal/shared/src/interp/glue/gllmultimap.cpp.c` (18/18)

### gLLMultiMap
- [x] `gLLMultiMap::CallEFun_empty` · 0054cf10 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLLMultiMap::CallEFun_count` · 0054cf80 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

### BaseMultiMap<unsigned___int64,unsigned___int64,13976>
- [x] `BaseMultiMap<unsigned___int64,unsigned___int64,13976>::find` · 0054d180 · legions 0052a680 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `BaseMultiMap<unsigned___int64,unsigned___int64,13976>::find_last` · 0054d1e0 · legions 0052a6e0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `BaseMultiMap<unsigned___int64,unsigned___int64,13976>::get_begin` · 0054d240 · legions 0052a740 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `BaseMultiMap<unsigned___int64,unsigned___int64,13976>::get_end` · 0054d290 · legions 0052a790 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `BaseMultiMap<unsigned___int64,unsigned___int64,13976>::lower_bound` · 0054d2e0 · legions 0052a7e0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `BaseMultiMap<unsigned___int64,unsigned___int64,13976>::upper_bound` · 0054d340 · legions 0052a840 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

### gLLMultiMap
- [x] `gLLMultiMap::CallEFun_clear` · 0054d3c0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLLMultiMap::CallEFun_erase` · 0054d440 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLLMultiMap::CallEFun_find` · 0054d4d0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLLMultiMap::CallEFun_find_last` · 0054d550 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLLMultiMap::CallEFun_get_begin` · 0054d5d0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLLMultiMap::CallEFun_get_end` · 0054d650 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLLMultiMap::CallEFun_lower_bound` · 0054d6d0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLLMultiMap::CallEFun_upper_bound` · 0054d750 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLLMultiMap::CallEFun_insert` · 0054d7d0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLLMultiMap::CallEFun_remove` · 0054d860 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

## `portal/shared/src/interp/glue/gllmultimapiter.cpp.c` (5/5)

### gLLMultiMapIter
- [x] `gLLMultiMapIter::CallEFun_data` · 0054cc40 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLLMultiMapIter::CallEFun_equals` · 0054ccb0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLLMultiMapIter::CallEFun_key` · 0054cd40 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLLMultiMapIter::CallEFun_next` · 0054cdb0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLLMultiMapIter::CallEFun_prev` · 0054ce50 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

## `portal/shared/src/interp/glue/glosparameters.cpp.c` (2/18)

### LOSParameters
- [x] `LOSParameters::SetFromPositionAndOffset` · 00542530 · legions 00522ad0 — ac2-physics los (Terrain::los_check)
- [x] `LOSParameters::SetFromBeginAndEndPosition` · 005425b0 · legions 00522b50 — ac2-physics los (Terrain::los_check)

### gLOSParameters
- [ ] `gLOSParameters::CallEFun_AddToEtherealList` · 00542680
- [ ] `gLOSParameters::CallEFun_SetBlockedByWater` · 00542700
- [ ] `gLOSParameters::CallEFun_SetCheckEnvOnly` · 00542760
- [ ] `gLOSParameters::CallEFun_SetCheckPortalsOnly` · 005427c0
- [ ] `gLOSParameters::CallEFun_SetCheckUnconnectedPortalsOnly` · 00542820
- [ ] `gLOSParameters::CallEFun_SetExactDrawingCheck` · 00542880
- [ ] `gLOSParameters::CallEFun_SetFromBeginAndEndPosition` · 005428e0
- [ ] `gLOSParameters::CallEFun_SetFromPositionAndOffset` · 00542950
- [ ] `gLOSParameters::CallEFun_SetLOSParams` · 005429e0
- [ ] `gLOSParameters::CallEFun_SetPhysicsCheck` · 00542a50
- [ ] `gLOSParameters::CallEFun_SetRay` · 00542ab0
- [ ] `gLOSParameters::CallEFun_GetFinalCellID` · 00542b20
- [ ] `gLOSParameters::CallEFun_HitUnloadedBlock` · 00542b90
- [ ] `gLOSParameters::CallEFun_RayUnblocked` · 00542c10
- [ ] `gLOSParameters::CallEFun_GetEndPoint` · 00542c90
- [ ] `gLOSParameters::CallEFun_GetFinalRay` · 00542d30

## `portal/shared/src/interp/glue/glrhash.cpp.c` (20/20)

### gLRHash
- [x] `gLRHash::CallEFun_empty` · 00559a80 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLRHash::CallEFun_num_buckets` · 00559af0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLRHash::CallEFun_num_elements` · 00559b60 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLRHash::CallEFun_too_big` · 00559bd0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLRHash::CallEFun_too_small` · 00559c50 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLRHash::CallEFun_flush` · 00559d30 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLRHash::CallEFun_contains` · 00559d90 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLRHash::CallEFun_erase` · 00559e30 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLRHash::CallEFun_add` · 00559f70 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLRHash::CallEFun_set` · 0055a010 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

### BaseHash<unsigned___int64,unsigned_long,65>
- [x] `BaseHash<unsigned___int64,unsigned_long,65>::get_begin` · 0055a150 · legions 00531600 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `BaseHash<unsigned___int64,unsigned_long,65>::get_end` · 0055a1d0 · legions 00531680 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

### gLRHash
- [x] `gLRHash::CallEFun_get_begin` · 0055a230 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLRHash::CallEFun_get_end` · 0055a2b0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLRHash::CallEFun_grow` · 0055a330 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLRHash::CallEFun_resize` · 0055a3a0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLRHash::CallEFun_shrink` · 0055a450 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLRHash::CallEFun_optimize_size` · 0055a4c0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLRHash::CallEFun_find` · 0055a540 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLRHash::CallEFun_remove` · 0055a620 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

## `portal/shared/src/interp/glue/glrhashiter.cpp.c` (6/6)

### gLRHashIter
- [x] `gLRHashIter::CallEFun_data` · 005597d0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLRHashIter::CallEFun_key` · 00559840 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLRHashIter::CallEFun_equals` · 005598b0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLRHashIter::CallEFun_notdone` · 00559940 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLRHashIter::CallEFun_next` · 005599b0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gLRHashIter::CallEFun_remove` · 00559a10 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

## `portal/shared/src/interp/glue/glrmultihash.cpp.c` (13/13)

### gLRMultiHash
- [x] `gLRMultiHash::CallEFun_size` · 00558fe0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLRMultiHash::CallEFun_count` · 00559050 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

### BaseMultiHash<unsigned___int64,unsigned_long,67>
- [x] `BaseMultiHash<unsigned___int64,unsigned_long,67>::find` · 00559240 · legions 00530fc0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `BaseMultiHash<unsigned___int64,unsigned_long,67>::find_last` · 005592d0 · legions 00531050 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `BaseMultiHash<unsigned___int64,unsigned_long,67>::get_begin` · 00559330 · legions 005310b0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

### gLRMultiHash
- [x] `gLRMultiHash::CallEFun_clear` · 00559390 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLRMultiHash::CallEFun_erase` · 005593f0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLRMultiHash::CallEFun_find` · 00559470 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLRMultiHash::CallEFun_find_last` · 005594f0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLRMultiHash::CallEFun_get_begin` · 00559570 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLRMultiHash::CallEFun_get_end` · 005595f0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLRMultiHash::CallEFun_remove` · 005596b0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLRMultiHash::CallEFun_add` · 00559740 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

## `portal/shared/src/interp/glue/glrmultihashiter.cpp.c` (4/4)

### gLRMultiHashIter
- [x] `gLRMultiHashIter::CallEFun_data` · 00558e10 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLRMultiHashIter::CallEFun_equals` · 00558e80 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLRMultiHashIter::CallEFun_key` · 00558f00 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLRMultiHashIter::CallEFun_next` · 00558f70 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

## `portal/shared/src/interp/glue/gmath.cpp.c` (16/28)

### gMath
- [~] `gMath::CallEFun_MapToUnitInterval` · 00558690 — ac2-script glue::math: Rust f64 transcendentals / D3DXVec3Normalize, not the x87 instruction results
- [x] `gMath::CallEFun___max` · 00558700 — ac2-script glue::math / glue::missile
- [x] `gMath::CallEFun___min` · 00558750 — ac2-script glue::math / glue::missile
- [x] `gMath::CallEFun_abs` · 005587a0 — ac2-script glue::math / glue::missile
- [~] `gMath::CallEFun_acos` · 005587d0 — ac2-script glue::math: Rust f64 transcendentals / D3DXVec3Normalize, not the x87 instruction results
- [~] `gMath::CallEFun_asin` · 00558820 — ac2-script glue::math: Rust f64 transcendentals / D3DXVec3Normalize, not the x87 instruction results
- [~] `gMath::CallEFun_atan` · 00558870 — ac2-script glue::math: Rust f64 transcendentals / D3DXVec3Normalize, not the x87 instruction results
- [x] `gMath::CallEFun_ceil` · 005588a0 — ac2-script glue::math / glue::missile
- [~] `gMath::CallEFun_cos` · 005588f0 — ac2-script glue::math: Rust f64 transcendentals / D3DXVec3Normalize, not the x87 instruction results
- [~] `gMath::CallEFun_exp` · 00558920 — ac2-script glue::math: Rust f64 transcendentals / D3DXVec3Normalize, not the x87 instruction results
- [x] `gMath::CallEFun_fabs` · 00558960 — ac2-script glue::math / glue::missile
- [x] `gMath::CallEFun_fake_abs64` · 00558990 · legions 00530920 — ac2-script glue::math / glue::missile
- [x] `gMath::CallEFun_floatMax` · 005589d0 — ac2-script glue::math / glue::missile
- [x] `gMath::CallEFun_floatMin` · 00558a10 — ac2-script glue::math / glue::missile
- [x] `gMath::CallEFun_floor` · 00558a50 — ac2-script glue::math / glue::missile
- [~] `gMath::CallEFun_fmod` · 00558aa0 — ac2-script glue::math: Rust f64 transcendentals / D3DXVec3Normalize, not the x87 instruction results
- [x] `gMath::CallEFun_intMax` · 00558b00 — ac2-script glue::math / glue::missile
- [x] `gMath::CallEFun_intMin` · 00558b40 — ac2-script glue::math / glue::missile
- [x] `gMath::CallEFun_lintMax` · 00558b80 — ac2-script glue::math / glue::missile
- [x] `gMath::CallEFun_lintMin` · 00558bd0 — ac2-script glue::math / glue::missile
- [~] `gMath::CallEFun_log` · 00558c20 — ac2-script glue::math: Rust f64 transcendentals / D3DXVec3Normalize, not the x87 instruction results
- [~] `gMath::CallEFun_log10` · 00558c50 — ac2-script glue::math: Rust f64 transcendentals / D3DXVec3Normalize, not the x87 instruction results
- [~] `gMath::CallEFun_pow` · 00558c80 — ac2-script glue::math: Rust f64 transcendentals / D3DXVec3Normalize, not the x87 instruction results
- [~] `gMath::CallEFun_sin` · 00558ce0 — ac2-script glue::math: Rust f64 transcendentals / D3DXVec3Normalize, not the x87 instruction results
- [x] `gMath::CallEFun_sqrt` · 00558d10 — ac2-script glue::math / glue::missile
- [~] `gMath::CallEFun_tan` · 00558d40 — ac2-script glue::math: Rust f64 transcendentals / D3DXVec3Normalize, not the x87 instruction results
- [x] `gMath::CallEFun_ulintMax` · 00558d70 — ac2-script glue::math / glue::missile
- [x] `gMath::CallEFun_ulintMin` · 00558dc0 — ac2-script glue::math / glue::missile

## `portal/shared/src/interp/glue/gmissileinfo.cpp.c` (0/7)

### gMissileInfo
- [ ] `gMissileInfo::CallEFun_SetClone` · 00542290
- [ ] `gMissileInfo::CallEFun_SetCollisionParameters` · 005422f0
- [ ] `gMissileInfo::CallEFun_SetEntityDID` · 00542350
- [ ] `gMissileInfo::CallEFun_SetHoldingLocation` · 005423b0
- [ ] `gMissileInfo::CallEFun_SetMissileParameters` · 00542410
- [ ] `gMissileInfo::CallEFun_SetTargetingParameters` · 00542470
- [ ] `gMissileInfo::CallEFun_SetThrown` · 005424d0

## `portal/shared/src/interp/glue/gmissileparameters.cpp.c` (23/23)

### gMissileParameters
- [x] `gMissileParameters::CallEFun_SetAffectedByGravity` · 00541990 — ac2-script glue::math / glue::missile
- [x] `gMissileParameters::CallEFun_SetAlignToPath` · 005419f0 — ac2-script glue::math / glue::missile
- [x] `gMissileParameters::CallEFun_SetCanLand` · 00541a50 — ac2-script glue::math / glue::missile
- [x] `gMissileParameters::CallEFun_SetConserveFuel` · 00541ab0 — ac2-script glue::math / glue::missile
- [x] `gMissileParameters::CallEFun_SetDetonatesOnMiss` · 00541b20 — ac2-script glue::math / glue::missile
- [x] `gMissileParameters::CallEFun_SetDetonationProximity` · 00541b80 — ac2-script glue::math / glue::missile
- [x] `gMissileParameters::CallEFun_SetDragFactor` · 00541bf0 — ac2-script glue::math / glue::missile
- [x] `gMissileParameters::CallEFun_SetFollowGroundType` · 00541c50 — ac2-script glue::math / glue::missile
- [x] `gMissileParameters::CallEFun_SetHasNoLifespan` · 00541cb0 — ac2-script glue::math / glue::missile
- [x] `gMissileParameters::CallEFun_SetMaintainDistance` · 00541d10 — ac2-script glue::math / glue::missile
- [x] `gMissileParameters::CallEFun_SetMaxAcceleration` · 00541d70 — ac2-script glue::math / glue::missile
- [x] `gMissileParameters::CallEFun_SetMaxAccelerationUniform` · 00541df0 — ac2-script glue::math / glue::missile
- [x] `gMissileParameters::CallEFun_SetMissileAsHoming` · 00541e50 — ac2-script glue::math / glue::missile
- [x] `gMissileParameters::CallEFun_SetMissileReaction` · 00541eb0 — ac2-script glue::math / glue::missile
- [x] `gMissileParameters::CallEFun_SetMissileSticky` · 00541f20 — ac2-script glue::math / glue::missile
- [x] `gMissileParameters::CallEFun_SetSpeedLimits` · 00541f80 — ac2-script glue::math / glue::missile
- [x] `gMissileParameters::CallEFun_SetSpinRate` · 00541ff0 — ac2-script glue::math / glue::missile
- [x] `gMissileParameters::CallEFun_SetToNoAlign` · 00542050 — ac2-script glue::math / glue::missile
- [x] `gMissileParameters::CallEFun_SetTotalFuel` · 005420b0 — ac2-script glue::math / glue::missile
- [x] `gMissileParameters::CallEFun_SetTotalLifespan` · 00542110 — ac2-script glue::math / glue::missile
- [x] `gMissileParameters::CallEFun_SetTurnMethod` · 00542170 — ac2-script glue::math / glue::missile
- [x] `gMissileParameters::CallEFun_SetUseNoFuel` · 005421d0 — ac2-script glue::math / glue::missile
- [x] `gMissileParameters::CallEFun_UseEnvironmentalMaximumVelocities` · 00542230 — ac2-script glue::math / glue::missile

## `portal/shared/src/interp/glue/gmissiletargetingparameters.cpp.c` (0/12)

### gMissileTargetingParameters
- [ ] `gMissileTargetingParameters::CallEFun_ClientUseParentForBasePosition` · 005414d0
- [ ] `gMissileTargetingParameters::CallEFun_SetAlwaysStartTowardTarget` · 00541530
- [ ] `gMissileTargetingParameters::CallEFun_SetBasePosition` · 00541590
- [ ] `gMissileTargetingParameters::CallEFun_SetInitialPitch` · 005415f0
- [ ] `gMissileTargetingParameters::CallEFun_SetInitialSpeed` · 00541650
- [ ] `gMissileTargetingParameters::CallEFun_SetMaxInitialTargetRange` · 005416b0
- [ ] `gMissileTargetingParameters::CallEFun_SetMissileDetonationChance` · 00541710
- [ ] `gMissileTargetingParameters::CallEFun_SetNoFireIfTooFar` · 00541770
- [ ] `gMissileTargetingParameters::CallEFun_SetStartTowardTarget` · 005417d0
- [ ] `gMissileTargetingParameters::CallEFun_SetTargetID` · 00541830
- [ ] `gMissileTargetingParameters::CallEFun_SetTargetPos` · 005418b0
- [ ] `gMissileTargetingParameters::CallEFun_UseSourceObjectForBasePosition` · 00541910

## `portal/shared/src/interp/glue/gmotionvalues.cpp.c` (0/22)

### gMotionValues
- [ ] `gMotionValues::CallEFun_GetBackwardAcl` · 00540b30
- [ ] `gMotionValues::CallEFun_GetBackwardSB` · 00540ba0
- [ ] `gMotionValues::CallEFun_GetDrag` · 00540c10
- [ ] `gMotionValues::CallEFun_GetForwardAcl` · 00540c80
- [ ] `gMotionValues::CallEFun_GetForwardSB` · 00540cf0
- [ ] `gMotionValues::CallEFun_GetGravity` · 00540d60
- [ ] `gMotionValues::CallEFun_GetHoverHeight` · 00540dd0
- [ ] `gMotionValues::CallEFun_GetJumpImpulse` · 00540e40
- [ ] `gMotionValues::CallEFun_GetLift` · 00540eb0
- [ ] `gMotionValues::CallEFun_GetMaxBackwardVel` · 00540f20
- [ ] `gMotionValues::CallEFun_GetMaxForwardVel` · 00540f90
- [ ] `gMotionValues::CallEFun_GetMaxStrafeVel` · 00541000
- [ ] `gMotionValues::CallEFun_GetMaxVertVel` · 00541070
- [ ] `gMotionValues::CallEFun_GetRunningTurnVel` · 005410e0
- [ ] `gMotionValues::CallEFun_GetSinkOffset` · 00541150
- [ ] `gMotionValues::CallEFun_GetStillTurnVel` · 005411c0
- [ ] `gMotionValues::CallEFun_GetStrafeAcl` · 00541230
- [ ] `gMotionValues::CallEFun_GetStrafeSB` · 005412a0
- [ ] `gMotionValues::CallEFun_GetTerminalVel` · 00541310
- [ ] `gMotionValues::CallEFun_GetTurnAcl` · 00541380
- [ ] `gMotionValues::CallEFun_GetTurnSB` · 005413f0
- [ ] `gMotionValues::CallEFun_GetVertAcl` · 00541460

## `portal/shared/src/interp/glue/gmovementparameters.cpp.c` (0/21)

### gMovementParameters
- [ ] `gMovementParameters::CallEFun_set_callback_when_in_range` · 005402a0
- [ ] `gMovementParameters::CallEFun_set_context_id` · 00540300
- [ ] `gMovementParameters::CallEFun_set_desired_distance` · 00540360
- [ ] `gMovementParameters::CallEFun_set_desired_velocity` · 005403c0
- [ ] `gMovementParameters::CallEFun_set_fail_distance` · 00540420
- [ ] `gMovementParameters::CallEFun_set_max_success_distance` · 00540480
- [ ] `gMovementParameters::CallEFun_set_min_success_distance` · 005404e0
- [ ] `gMovementParameters::CallEFun_set_move_away_flag` · 00540540
- [ ] `gMovementParameters::CallEFun_set_move_towards_flag` · 005405a0
- [ ] `gMovementParameters::CallEFun_set_object_flag` · 00540600
- [ ] `gMovementParameters::CallEFun_set_offset_vector` · 00540660
- [ ] `gMovementParameters::CallEFun_set_persistent_fail_exits_flag` · 005406c0
- [ ] `gMovementParameters::CallEFun_set_persistent_flag` · 00540720
- [ ] `gMovementParameters::CallEFun_set_target_heading` · 00540780
- [ ] `gMovementParameters::CallEFun_set_target_object_id` · 005407f0
- [ ] `gMovementParameters::CallEFun_set_target_position` · 00540850
- [ ] `gMovementParameters::CallEFun_set_turn_flag` · 005408b0
- [ ] `gMovementParameters::CallEFun_set_use_cylinders` · 00540910
- [ ] `gMovementParameters::CallEFun_get_context_id` · 00540970
- [ ] `gMovementParameters::CallEFun_set_position_list` · 005409e0
- [ ] `gMovementParameters::CallEFun_pop_position_list` · 00540aa0

## `portal/shared/src/interp/glue/gmovementreturn.cpp.c` (0/8)

### gMovementReturn
- [ ] `gMovementReturn::CallEFun_GetError` · 0053fe10
- [ ] `gMovementReturn::CallEFun_GetObjectHeight` · 0053fe80
- [ ] `gMovementReturn::CallEFun_GetObjectID` · 0053fef0
- [ ] `gMovementReturn::CallEFun_GetObjectRadius` · 0053ff80
- [ ] `gMovementReturn::CallEFun_GetPersistentFlag` · 0053fff0
- [ ] `gMovementReturn::CallEFun_GetStatus` · 00540060
- [ ] `gMovementReturn::CallEFun_GetMovementParameters` · 005400d0
- [ ] `gMovementReturn::CallEFun_GetPosition` · 005401a0

## `portal/shared/src/interp/glue/gnahash.cpp.c` (20/20)

### gNAHash
- [x] `gNAHash::CallEFun_empty` · 00557b10 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gNAHash::CallEFun_num_buckets` · 00557b80 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gNAHash::CallEFun_num_elements` · 00557bf0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gNAHash::CallEFun_too_big` · 00557c60 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gNAHash::CallEFun_too_small` · 00557ce0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gNAHash::CallEFun_contains` · 00557d60 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gNAHash::CallEFun_erase` · 00557e10 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gNAHash::CallEFun_flush` · 00557ed0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gNAHash::CallEFun_add` · 00557f30 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gNAHash::CallEFun_set` · 00557fe0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

### BaseHash<ReferenceIDWrapper,unsigned_long,76>
- [x] `BaseHash<ReferenceIDWrapper,unsigned_long,76>::get_begin` · 00558090 · legions 005302e0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `BaseHash<ReferenceIDWrapper,unsigned_long,76>::get_end` · 00558110 · legions 00530360 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

### gNAHash
- [x] `gNAHash::CallEFun_get_begin` · 00558170 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gNAHash::CallEFun_get_end` · 005581f0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gNAHash::CallEFun_grow` · 00558270 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gNAHash::CallEFun_resize` · 005582e0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gNAHash::CallEFun_shrink` · 00558390 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gNAHash::CallEFun_optimize_size` · 00558400 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gNAHash::CallEFun_find` · 00558480 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gNAHash::CallEFun_remove` · 00558580 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

## `portal/shared/src/interp/glue/gnahashiter.cpp.c` (5/5)

### gNAHashIter
- [x] `gNAHashIter::CallEFun_data` · 005578d0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gNAHashIter::CallEFun_key` · 00557940 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gNAHashIter::CallEFun_equals` · 005579b0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gNAHashIter::CallEFun_notdone` · 00557a40 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gNAHashIter::CallEFun_next` · 00557ab0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

## `portal/shared/src/interp/glue/gnrhash.cpp.c` (20/20)

### gNRHash
- [x] `gNRHash::CallEFun_empty` · 00556b90 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gNRHash::CallEFun_num_buckets` · 00556c00 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gNRHash::CallEFun_num_elements` · 00556c70 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gNRHash::CallEFun_too_big` · 00556ce0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gNRHash::CallEFun_too_small` · 00556d60 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gNRHash::CallEFun_contains` · 00556e40 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gNRHash::CallEFun_erase` · 00556ef0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gNRHash::CallEFun_flush` · 00556fb0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gNRHash::CallEFun_add` · 005570a0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gNRHash::CallEFun_set` · 00557150 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

### BaseHash<ReferenceIDWrapper,unsigned_long,78>
- [x] `BaseHash<ReferenceIDWrapper,unsigned_long,78>::get_begin` · 00557300 · legions 0052fca0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `BaseHash<ReferenceIDWrapper,unsigned_long,78>::get_end` · 00557380 · legions 0052fd20 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

### gNRHash
- [x] `gNRHash::CallEFun_get_begin` · 005573e0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gNRHash::CallEFun_get_end` · 00557460 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gNRHash::CallEFun_grow` · 005574e0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gNRHash::CallEFun_resize` · 00557550 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gNRHash::CallEFun_shrink` · 00557600 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gNRHash::CallEFun_optimize_size` · 00557670 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gNRHash::CallEFun_find` · 005576f0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gNRHash::CallEFun_remove` · 005577e0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

## `portal/shared/src/interp/glue/gnrhashiter.cpp.c` (5/6)

### gNRHashIter
- [x] `gNRHashIter::CallEFun_data` · 00556910 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gNRHashIter::CallEFun_key` · 00556980 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gNRHashIter::CallEFun_equals` · 005569f0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)
- [x] `gNRHashIter::CallEFun_notdone` · 00556a80 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

### HashIterator<ReferenceIDWrapper,unsigned_long>
- [ ] `HashIterator<ReferenceIDWrapper,unsigned_long>::next` · 00556af0 · legions 0052f8e0

### gNRHashIter
- [x] `gNRHashIter::CallEFun_next` · 00556b30 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

## `portal/shared/src/interp/glue/gobjcollisionprofile.cpp.c` (0/4)

### gObjCollisionProfile
- [ ] `gObjCollisionProfile::CallEFun_GetID` · 0053fc50
- [ ] `gObjCollisionProfile::CallEFun_IsBeginCollision` · 0053fcc0
- [ ] `gObjCollisionProfile::CallEFun_IsEndCollision` · 0053fd30
- [ ] `gObjCollisionProfile::CallEFun_IsEthereal` · 0053fda0

## `portal/shared/src/interp/glue/gpackbuffer.cpp.c` (10/10)

### gPackBuffer
- [x] `gPackBuffer::CallEFun_PackFloat` · 0053f690 — ac2-script host (ScriptHost dispatch, PackBuffer, SendServerEvt, notices)
- [x] `gPackBuffer::CallEFun_PackInt` · 0053f710 — ac2-script host (ScriptHost dispatch, PackBuffer, SendServerEvt, notices)
- [x] `gPackBuffer::CallEFun_PackLFloat` · 0053f790 — ac2-script host (ScriptHost dispatch, PackBuffer, SendServerEvt, notices)
- [x] `gPackBuffer::CallEFun_PackLInt` · 0053f830 — ac2-script host (ScriptHost dispatch, PackBuffer, SendServerEvt, notices)
- [x] `gPackBuffer::CallEFun_PackPackage` · 0053f8b0 — ac2-script host (ScriptHost dispatch, PackBuffer, SendServerEvt, notices)
- [x] `gPackBuffer::CallEFun_UnPackFloat` · 0053f910 · legions 00520d70 — ac2-script host (ScriptHost dispatch, PackBuffer, SendServerEvt, notices)
- [x] `gPackBuffer::CallEFun_UnPackInt` · 0053f9b0 · legions 00520df0 — ac2-script host (ScriptHost dispatch, PackBuffer, SendServerEvt, notices)
- [x] `gPackBuffer::CallEFun_UnPackLFloat` · 0053fa50 · legions 00520e70 — ac2-script host (ScriptHost dispatch, PackBuffer, SendServerEvt, notices)
- [x] `gPackBuffer::CallEFun_UnPackLInt` · 0053faf0 · legions 00520ef0 — ac2-script host (ScriptHost dispatch, PackBuffer, SendServerEvt, notices)
- [x] `gPackBuffer::CallEFun_UnPackPackage` · 0053fba0 · legions 00520f70 — ac2-script host (ScriptHost dispatch, PackBuffer, SendServerEvt, notices)

## `portal/shared/src/interp/glue/gpath.cpp.c` (0/12)

### gPath
- [ ] `gPath::CallEFun_set_dist` · 0053efd0
- [ ] `gPath::CallEFun_set_ratio` · 0053f030
- [ ] `gPath::CallEFun_set_time` · 0053f090
- [ ] `gPath::CallEFun_get_dist` · 0053f0f0
- [ ] `gPath::CallEFun_get_ratio` · 0053f160
- [ ] `gPath::CallEFun_get_time` · 0053f1d0
- [ ] `gPath::CallEFun_length` · 0053f250
- [ ] `gPath::CallEFun_unsolved` · 0053f2c0

### gMovementParameters
- [ ] `gMovementParameters::RListToPositionList` · 0053f340 · legions 00521800

### gPath
- [ ] `gPath::CallEFun_ToStringW` · 0053f410
- [ ] `gPath::PositionListToRList` · 0053f4d0 · legions 00520a00
- [ ] `gPath::CallEFun_TranslateToPositionList` · 0053f590

## `portal/shared/src/interp/glue/gpathfinder.cpp.c` (0/14)

### gPathFinder
- [ ] `gPathFinder::CallEFun_AddRequiredProperties` · 0053e9b0
- [ ] `gPathFinder::CallEFun_AddSolutionTypes` · 0053ea10
- [ ] `gPathFinder::CallEFun_AddTerminateSolutionTypes` · 0053ea70
- [ ] `gPathFinder::CallEFun_HasRequiredProperties` · 0053ead0
- [ ] `gPathFinder::CallEFun_SetRequiredProperties` · 0053eb10
- [ ] `gPathFinder::CallEFun_SetSolutionTypes` · 0053eb70
- [ ] `gPathFinder::CallEFun_SetTerminateSolutionTypes` · 0053ebd0
- [ ] `gPathFinder::CallEFun_HasSolutionType` · 0053ec30
- [ ] `gPathFinder::CallEFun_HasTerminateSolutionType` · 0053ecb0
- [ ] `gPathFinder::CallEFun_SetEnd` · 0053ed30
- [ ] `gPathFinder::CallEFun_SetEndPosition` · 0053edb0
- [ ] `gPathFinder::CallEFun_SetStart` · 0053ee30
- [ ] `gPathFinder::CallEFun_SetStartPosition` · 0053eeb0
- [ ] `gPathFinder::CallEFun_InqSolution` · 0053ef30

## `portal/shared/src/interp/glue/gpathfinder_normal.cpp.c` (0/10)

### gPathFinder_Normal
- [ ] `gPathFinder_Normal::CallEFun_SetHeight` · 0053e5e0
- [ ] `gPathFinder_Normal::CallEFun_SetMaxDistance` · 0053e640
- [ ] `gPathFinder_Normal::CallEFun_SetMinDistance` · 0053e6a0
- [ ] `gPathFinder_Normal::CallEFun_SetMoveAway` · 0053e700
- [ ] `gPathFinder_Normal::CallEFun_SetRandomComponent` · 0053e760
- [ ] `gPathFinder_Normal::CallEFun_SetSameBlockConstraint` · 0053e7c0
- [ ] `gPathFinder_Normal::CallEFun_SetSlopeConstraint` · 0053e830
- [ ] `gPathFinder_Normal::CallEFun_SetSolidConstraint` · 0053e890
- [ ] `gPathFinder_Normal::CallEFun_SetWaterConstraint` · 0053e8f0
- [ ] `gPathFinder_Normal::CallEFun_SetWidth` · 0053e950

## `portal/shared/src/interp/glue/gpathnodedata.cpp.c` (0/3)

### gPathNodeData
- [ ] `gPathNodeData::CallEFun_GetSlopeConstraint` · 0053e490
- [ ] `gPathNodeData::CallEFun_IsSolid` · 0053e500
- [ ] `gPathNodeData::CallEFun_IsWater` · 0053e570

## `portal/shared/src/interp/glue/gpathsystem.cpp.c` (0/13)

### PathSystem
- [ ] `PathSystem::QueryPosition_Internal` · 0053de90 · legions 0051fb50

### gPathSystem
- [ ] `gPathSystem::CallEFun_GetPathMapID` · 0053df10
- [ ] `gPathSystem::CallEFun_GetPathNodeID` · 0053df50

### PathSystem
- [ ] `PathSystem::DumpPathMapW` · 0053dfb0 · legions 0051fc70
- [ ] `PathSystem::DumpSearchStatsW` · 0053e070 · legions 0051fd30
- [ ] `PathSystem::DumpCacheW` · 0053e130 · legions 0051fdf0

### gPathSystem
- [ ] `gPathSystem::CallEFun_DumpCacheW` · 0053e220
- [ ] `gPathSystem::CallEFun_DumpPathMapW` · 0053e260
- [ ] `gPathSystem::CallEFun_DumpSearchStatsW` · 0053e2c0
- [ ] `gPathSystem::CallEFun_FindPath` · 0053e300
- [ ] `gPathSystem::CallEFun_InqPathNodeDataByNodeID` · 0053e360
- [ ] `gPathSystem::CallEFun_InqPathNodeDataByPosition` · 0053e3c0
- [ ] `gPathSystem::CallEFun_QueryPosition` · 0053e430

## `portal/shared/src/interp/glue/gperfmon.cpp.c` (0/9)

### gPerfMon
- [ ] `gPerfMon::CallEFun_AdjustCounter16` · 0053dd50
- [ ] `gPerfMon::CallEFun_AdjustCounter32` · 0053dd70
- [ ] `gPerfMon::CallEFun_AdjustCounter64` · 0053dd90
- [ ] `gPerfMon::CallEFun_AdjustCounter8` · 0053ddc0
- [ ] `gPerfMon::CallEFun_DecrementCounter` · 0053dde0
- [ ] `gPerfMon::CallEFun_IncrementCounter` · 0053de00
- [ ] `gPerfMon::CallEFun_StartTimerCounter` · 0053de20
- [ ] `gPerfMon::CallEFun_StopTimerCounter` · 0053de40
- [ ] `gPerfMon::CallEFun_MeasureLock` · 0053de60

## `portal/shared/src/interp/glue/gpfilenode.cpp.c` (0/39)

### FileNodeName_UInt64
- [ ] `FileNodeName_UInt64::GetType` · 00543d60 · legions 005238d0
- [ ] `FileNodeName_UInt64::Clone` · 00543d70 · legions 005238e0

### FileNodeName_Int64
- [ ] `FileNodeName_Int64::Clone` · 00543da0 · legions 00523910

### PFileNode
- [ ] `PFileNode::SetNameAsBool` · 00543dd0 · legions 00523940

### gPFileNode
- [ ] `gPFileNode::CallEFun_AddAndRefSubNode` · 00543e20
- [ ] `gPFileNode::CallEFun_ReportErrorWSL` · 00543e80
- [ ] `gPFileNode::CallEFun_ReportWarningWSL` · 00543ee0
- [ ] `gPFileNode::CallEFun_SetNameAsBool` · 00543f40
- [ ] `gPFileNode::CallEFun_SetNameAsFloat` · 00543fe0
- [ ] `gPFileNode::CallEFun_SetNameAsInt32` · 00544080
- [ ] `gPFileNode::CallEFun_SetNameAsUInt32` · 00544120
- [ ] `gPFileNode::CallEFun_SetNameAsWPString` · 005441c0

### PFileNode
- [ ] `PFileNode::GetLeafAsInt32` · 00544220 · legions 00523c50
- [ ] `PFileNode::GetNameAsInt64` · 00544240
- [ ] `PFileNode::GetNameAsUInt64` · 00544310

### gPFileNode
- [ ] `gPFileNode::CallEFun_GetNumSubNodes` · 005443e0
- [ ] `gPFileNode::CallEFun_IsKeywordW` · 00544450
- [ ] `gPFileNode::CallEFun_GetSourceFileNameW` · 005444d0
- [ ] `gPFileNode::CallEFun_GetSubNode` · 00544590
- [ ] `gPFileNode::CallEFun_MakeSubNode` · 00544630
- [ ] `gPFileNode::CallEFun_GetLeafAsBool` · 005446c0
- [ ] `gPFileNode::CallEFun_GetLeafAsFloat` · 00544780
- [ ] `gPFileNode::CallEFun_GetLeafAsInt32` · 00544840
- [ ] `gPFileNode::CallEFun_GetLeafAsInt64` · 00544900
- [ ] `gPFileNode::CallEFun_GetLeafAsUInt32` · 005449c0
- [ ] `gPFileNode::CallEFun_GetLeafAsUInt64` · 00544a80
- [ ] `gPFileNode::CallEFun_GetLeafAsWPString` · 00544b40 · legions 00524350
- [ ] `gPFileNode::CallEFun_GetNameAsBool` · 00544be0
- [ ] `gPFileNode::CallEFun_GetNameAsFloat` · 00544c80
- [ ] `gPFileNode::CallEFun_GetNameAsInt32` · 00544d20
- [ ] `gPFileNode::CallEFun_GetNameAsInt64` · 00544dc0
- [ ] `gPFileNode::CallEFun_GetNameAsUInt32` · 00544e60
- [ ] `gPFileNode::CallEFun_GetNameAsUInt64` · 00544f00
- [ ] `gPFileNode::CallEFun_GetNameAsWPString` · 00544fa0 · legions 005246c0
- [ ] `gPFileNode::CallEFun_GetSubNameAsBool` · 00545040 · legions 00524730
- [ ] `gPFileNode::CallEFun_GetSubNameAsFloat` · 00545110 · legions 005247d0
- [ ] `gPFileNode::CallEFun_GetSubNameAsInt32` · 005451e0 · legions 00524870
- [ ] `gPFileNode::CallEFun_GetSubNameAsUInt32` · 005452b0 · legions 00524910
- [ ] `gPFileNode::CallEFun_GetSubNameAsWPString` · 00545380 · legions 005249b0

## `portal/shared/src/interp/glue/gpfileparser.cpp.c` (0/2)

### gPFileParser
- [ ] `gPFileParser::CallEFun_LoadFromStringW` · 00543bd0
- [ ] `gPFileParser::CallEFun_SaveToStringW` · 00543c50 · legions 00523800

## `portal/shared/src/interp/glue/gphysicsstory.cpp.c` (0/7)

### gPhysicsStory
- [ ] `gPhysicsStory::CallEFun_AddHook` · 0053daa0
- [ ] `gPhysicsStory::CallEFun_SetAttackerBehavior` · 0053db00
- [ ] `gPhysicsStory::CallEFun_SetAttackerID` · 0053db60
- [ ] `gPhysicsStory::CallEFun_SetBasicAttack` · 0053dbc0
- [ ] `gPhysicsStory::CallEFun_SetClientAttackContext` · 0053dc20
- [ ] `gPhysicsStory::CallEFun_SetSkillID` · 0053dc80
- [ ] `gPhysicsStory::CallEFun_GetOverallAttackResult` · 0053dce0

## `portal/shared/src/interp/glue/gplacestable.cpp.c` (0/6)

### gPlacesTable
- [ ] `gPlacesTable::NRHashToPlacesHash` · 0053d6a0 · legions 0051f480
- [ ] `gPlacesTable::CallEFun_WslDump` · 0053d730
- [ ] `gPlacesTable::PlacesHashToNRHash` · 0053d7b0 · legions 0051f590
- [ ] `gPlacesTable::CallEFun_InqPlace` · 0053d8f0
- [ ] `gPlacesTable::CallEFun_InqPlaceStr` · 0053d960
- [ ] `gPlacesTable::CallEFun_InqPlacesHash` · 0053d9d0

## `portal/shared/src/interp/glue/gplane.cpp.c` (0/4)

### gPlane
- [ ] `gPlane::CallEFun_InitPlane` · 0053d3f0
- [ ] `gPlane::CallEFun_distance_to_point` · 0053d480
- [ ] `gPlane::CallEFun_GetStringW` · 0053d510 · legions 0051f330
- [ ] `gPlane::CallEFun_snap_to_plane` · 0053d590 · legions 0051f390

## `portal/shared/src/interp/glue/gposition.cpp.c` (0/41)

### gPosition
- [ ] `gPosition::CallEFun_Set` · 0053be40
- [ ] `gPosition::CallEFun_SetCellID` · 0053beb0
- [ ] `gPosition::CallEFun_add_global_offset` · 0053bf10
- [ ] `gPosition::CallEFun_add_local_offset` · 0053bf90
- [ ] `gPosition::CallEFun_adjust_to_outside` · 0053bff0
- [ ] `gPosition::CallEFun_set_cell_id` · 0053c050
- [ ] `gPosition::CallEFun_set_height` · 0053c0b0
- [ ] `gPosition::CallEFun_set_origin` · 0053c110
- [ ] `gPosition::CallEFun_FromCellCoords` · 0053c180
- [ ] `gPosition::CallEFun_FromCellCoordsString` · 0053c200
- [ ] `gPosition::CallEFun_FromRealMapCoords` · 0053c280
- [ ] `gPosition::CallEFun_FromWPString` · 0053c300
- [ ] `gPosition::CallEFun_GetBlockID` · 0053c380
- [ ] `gPosition::CallEFun_GetCellDistance` · 0053c400
- [ ] `gPosition::CallEFun_GetCellDistanceSquared` · 0053c440
- [ ] `gPosition::CallEFun_GetCellID` · 0053c480
- [ ] `gPosition::CallEFun_GetCellX` · 0053c4f0
- [ ] `gPosition::CallEFun_GetCellY` · 0053c560
- [ ] `gPosition::CallEFun_IsValid` · 0053c5d0
- [ ] `gPosition::CallEFun_NotInWorld` · 0053c640
- [ ] `gPosition::CallEFun_cylinder_distance` · 0053c6b0
- [ ] `gPosition::CallEFun_determine_quadrant` · 0053c770
- [ ] `gPosition::CallEFun_distance` · 0053c800
- [ ] `gPosition::CallEFun_distance_squared` · 0053c880
- [ ] `gPosition::CallEFun_equals` · 0053c900
- [ ] `gPosition::CallEFun_get_height` · 0053c9a0
- [ ] `gPosition::CallEFun_in_same_block` · 0053ca10
- [ ] `gPosition::CallEFun_outside` · 0053cac0
- [ ] `gPosition::CallEFun_pitch` · 0053cb40
- [ ] `gPosition::CallEFun_xy_distance` · 0053cbc0
- [ ] `gPosition::CallEFun_xy_distance_squared` · 0053cc40
- [ ] `gPosition::CallEFun_GetLocalHeading` · 0053ccc0
- [ ] `gPosition::CallEFun_GetNullPosition` · 0053cd80
- [ ] `gPosition::CallEFun_ToCellCoordsString` · 0053ce30
- [ ] `gPosition::CallEFun_ToRealMapCoords` · 0053cef0
- [ ] `gPosition::CallEFun_ToRealMapCoords_Info` · 0053cfb0
- [ ] `gPosition::CallEFun_ToWPString` · 0053d060
- [ ] `gPosition::CallEFun_get_block_offset` · 0053d120
- [ ] `gPosition::CallEFun_get_offset` · 0053d1d0
- [ ] `gPosition::CallEFun_get_origin` · 0053d280
- [ ] `gPosition::CallEFun_heading` · 0053d330

## `portal/shared/src/interp/glue/gpropertycollection.cpp.c` (0/12)

### gPropertyCollection
- [ ] `gPropertyCollection::CallEFun_AddOptionalProperty` · 0053b790
- [ ] `gPropertyCollection::CallEFun_AddProperty` · 0053b810
- [ ] `gPropertyCollection::CallEFun_ContainsGroup` · 0053b890
- [ ] `gPropertyCollection::CallEFun_ContainsProperty` · 0053b910
- [ ] `gPropertyCollection::CallEFun_GetNumGroups` · 0053b990
- [ ] `gPropertyCollection::CallEFun_UpdateProperty` · 0053ba00
- [ ] `gPropertyCollection::CallEFun_InqGroup` · 0053ba80 · legions 0051e060
- [ ] `gPropertyCollection::CallEFun_InqGroupAt` · 0053bb20
- [ ] `gPropertyCollection::CallEFun_InqOptionalProperty` · 0053bbc0
- [ ] `gPropertyCollection::CallEFun_InqOptionalPropertyW` · 0053bc60
- [ ] `gPropertyCollection::CallEFun_InqProperty` · 0053bd00
- [ ] `gPropertyCollection::CallEFun_InqPropertyW` · 0053bda0

## `portal/shared/src/interp/glue/gpropertygroup.cpp.c` (0/14)

### gPropertyGroup
- [ ] `gPropertyGroup::CallEFun_AddOptionalProperty` · 0053b010
- [ ] `gPropertyGroup::CallEFun_AddProperty` · 0053b090
- [ ] `gPropertyGroup::CallEFun_ContainsOptionalProperty` · 0053b110
- [ ] `gPropertyGroup::CallEFun_ContainsProperty` · 0053b190
- [ ] `gPropertyGroup::CallEFun_GetNumOptionalProperties` · 0053b210
- [ ] `gPropertyGroup::CallEFun_GetNumProperties` · 0053b280
- [ ] `gPropertyGroup::CallEFun_RemoveProperty` · 0053b2f0
- [ ] `gPropertyGroup::CallEFun_SetGroupName` · 0053b370
- [ ] `gPropertyGroup::CallEFun_UpdateProperty` · 0053b3f0
- [ ] `gPropertyGroup::CallEFun_InqOptionalProperty` · 0053b470
- [ ] `gPropertyGroup::CallEFun_InqOptionalPropertyAt` · 0053b510
- [ ] `gPropertyGroup::CallEFun_InqProperty` · 0053b5b0
- [ ] `gPropertyGroup::CallEFun_InqPropertyAt` · 0053b650
- [ ] `gPropertyGroup::CallEFun_InqPropertyGroupNameW` · 0053b6f0

## `portal/shared/src/interp/glue/gpsutils.cpp.c` (0/13)

### PSUtils
- [ ] `PSUtils::is_double` · 0053ab40 · legions 0051d4b0
- [ ] `PSUtils::is_int32` · 0053abb0 · legions 0051d520
- [ ] `PSUtils::is_int64` · 0053ac20 · legions 0051d590
- [ ] `PSUtils::is_uint64` · 0053ac90 · legions 0051d600

### gPSUtils
- [ ] `gPSUtils::CallEFun_is_double` · 0053ad00
- [ ] `gPSUtils::CallEFun_is_int32` · 0053ad50
- [ ] `gPSUtils::CallEFun_is_int64` · 0053ada0
- [ ] `gPSUtils::CallEFun_is_uint32` · 0053adf0
- [ ] `gPSUtils::CallEFun_is_uint64` · 0053ae40
- [ ] `gPSUtils::CallEFun_cleanup_directory_nameW` · 0053ae90
- [ ] `gPSUtils::CallEFun_get_cwdW` · 0053aef0
- [ ] `gPSUtils::CallEFun_remove_extensionW` · 0053af50
- [ ] `gPSUtils::CallEFun_strip_directoryW` · 0053afb0

## `portal/shared/src/interp/glue/gqualifieddataidarray.cpp.c` (0/1)

### gQualifiedDataIDArray
- [ ] `gQualifiedDataIDArray::CallEFun_WslAddToEnd` · 0053aac0

## `portal/shared/src/interp/glue/grandom.cpp.c` (6/7)

### (free functions)
- [~] `RollSkewedNumberEx` · 005566d0 · legions 0052f650 — ac2-script glue: f32 spill points unchecked against x87 / copy order of PHashTable operator= unverified

### gRandom
- [x] `gRandom::CallEFun_RollDice` · 00556740 · legions 0052f6c0 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gRandom::CallEFun_RollDiceInt` · 00556790 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gRandom::CallEFun_RollGaussianDice` · 005567d0 · legions 0052f750 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gRandom::CallEFun_RollPercentile` · 00556820 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gRandom::CallEFun_RollSkewedNumber` · 00556850 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gRandom::CallEFun_RollSkewedNumberEx` · 005568b0 — ac2-script glue (wlog, random, behavior, enums: gDBCache)

## `portal/shared/src/interp/glue/grandomselectiontable_int.cpp.c` (12/13)

### RandomSelectionTable<unsigned_long>
- [x] `RandomSelectionTable<unsigned_long>::InitializeWeights_Internal` · 0053a480 · legions 0051cf10 — ac2-script glue (wlog, random, behavior, enums: gDBCache)

### gRandomSelectionTable_Int
- [x] `gRandomSelectionTable_Int::CallEFun_Done` · 0053a4e0 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gRandomSelectionTable_Int::CallEFun_Clear` · 0053a550 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gRandomSelectionTable_Int::CallEFun_InitializeWeights` · 0053a5b0 — ac2-script glue (wlog, random, behavior, enums: gDBCache)

### SmartArray<RandomSelectionTable<unsigned_long>::ElementStruct>
- [ ] `SmartArray<RandomSelectionTable<unsigned_long>::ElementStruct>::AddToEnd` · 0053a610 · legions 0051d040

### RandomSelectionTable<unsigned_long>
- [x] `RandomSelectionTable<unsigned_long>::AddEntry_Internal` · 0053a670 · legions 0051d0a0 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `RandomSelectionTable<unsigned_long>::MoveToUsedList` · 0053a6c0 · legions 0051d0f0 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `RandomSelectionTable<unsigned_long>::StartOver` · 0053a7a0 · legions 0051d1d0 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `RandomSelectionTable<unsigned_long>::Choose` · 0053a840 · legions 0051d270 — ac2-script glue (wlog, random, behavior, enums: gDBCache)

### gRandomSelectionTable_Int
- [x] `gRandomSelectionTable_Int::CallEFun_AddEntry` · 0053a8c0 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gRandomSelectionTable_Int::CallEFun_FastAddEntry` · 0053a940 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gRandomSelectionTable_Int::CallEFun_StartOver` · 0053a9c0 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gRandomSelectionTable_Int::CallEFun_Choose` · 0053aa20 · legions 0051d3e0 — ac2-script glue (wlog, random, behavior, enums: gDBCache)

## `portal/shared/src/interp/glue/grarray.cpp.c` (10/11)

### gRArray
- [x] `gRArray::CallEFun_flush` · 00556110 — ac2-script glue::lists

### RArray
- [ ] `RArray::SetNElements` · 00556170 · legions 0052f2d0

### gRArray
- [x] `gRArray::CallEFun_SetNElements` · 005561c0 — ac2-script glue::lists
- [x] `gRArray::CallEFun_empty` · 00556230 — ac2-script glue::lists
- [x] `gRArray::CallEFun_get` · 005562a0 — ac2-script glue::lists
- [x] `gRArray::CallEFun_num_elements` · 00556320 — ac2-script glue::lists
- [x] `gRArray::CallEFun_set` · 00556390 — ac2-script glue::lists
- [x] `gRArray::CallEFun_erase` · 00556420 — ac2-script glue::lists
- [x] `gRArray::CallEFun_push` · 005564c0 — ac2-script glue::lists
- [x] `gRArray::CallEFun_pop` · 00556570 — ac2-script glue::lists
- [x] `gRArray::CallEFun_query` · 00556620 — ac2-script glue::lists

## `portal/shared/src/interp/glue/gray.cpp.c` (0/8)

### gRay
- [ ] `gRay::CallEFun_InitFromOffset` · 00539fe0
- [ ] `gRay::CallEFun_InitLineSegment` · 0053a050
- [ ] `gRay::CallEFun_normalize` · 0053a0f0
- [ ] `gRay::CallEFun_GetLength` · 0053a150
- [ ] `gRay::CallEFun_IsFinite` · 0053a1c0
- [ ] `gRay::CallEFun_GetDirection` · 0053a250
- [ ] `gRay::CallEFun_GetEndPoint` · 0053a300
- [ ] `gRay::CallEFun_GetStartPoint` · 0053a3d0

## `portal/shared/src/interp/glue/greservedids.cpp.c` (0/6)

### gReservedIDs
- [ ] `gReservedIDs::CallEFun_IsCameraID` · 00539e60
- [ ] `gReservedIDs::CallEFun_IsDynamicID` · 00539ea0
- [ ] `gReservedIDs::CallEFun_IsPlayerID` · 00539ee0
- [ ] `gReservedIDs::CallEFun_IsStaticID` · 00539f20
- [ ] `gReservedIDs::CallEFun_IsTemporaryID` · 00539f60
- [ ] `gReservedIDs::CallEFun_IsUIID` · 00539fa0

## `portal/shared/src/interp/glue/grgbacolor.cpp.c` (0/5)

### gRGBAColor
- [ ] `gRGBAColor::CallEFun_Attenuate` · 00539c50
- [ ] `gRGBAColor::CallEFun_Set` · 00539cc0
- [ ] `gRGBAColor::CallEFun_SetAlpha` · 00539d30
- [ ] `gRGBAColor::CallEFun_SetColor32` · 00539d90
- [ ] `gRGBAColor::CallEFun_GetColor32` · 00539df0

## `portal/shared/src/interp/glue/grlist.cpp.c` (45/48)

### gRList
- [x] `gRList::CallEFun_empty` · 00554420 — ac2-script glue::lists
- [x] `gRList::CallEFun_num_elements` · 00554490 — ac2-script glue::lists
- [x] `gRList::CallEFun_push_back` · 00554500 — ac2-script glue::lists
- [x] `gRList::CallEFun_push_front` · 00554580 — ac2-script glue::lists
- [x] `gRList::CallEFun_insert_back` · 005546c0 — ac2-script glue::lists
- [x] `gRList::CallEFun_insert_front` · 00554740 — ac2-script glue::lists
- [x] `gRList::CallEFun_insert_head` · 005547c0 — ac2-script glue::lists
- [x] `gRList::CallEFun_insert_tail` · 00554840 — ac2-script glue::lists
- [x] `gRList::CallEFun_push` · 005548c0 — ac2-script glue::lists
- [x] `gRList::CallEFun_push_head` · 00554940 — ac2-script glue::lists
- [x] `gRList::CallEFun_push_tail` · 005549c0 — ac2-script glue::lists
- [x] `gRList::CallEFun_remove` · 00554a40 — ac2-script glue::lists

### BaseList<unsigned_long,11>
- [x] `BaseList<unsigned_long,11>::inq_data` · 00554b30 · legions 00535ff0 — ac2-script glue::lists

### gRList
- [x] `gRList::CallEFun_contains` · 00554b70 — ac2-script glue::lists
- [x] `gRList::CallEFun_erase_back` · 00554c00 — ac2-script glue::lists
- [x] `gRList::CallEFun_erase_front` · 00554cb0 — ac2-script glue::lists
- [x] `gRList::CallEFun_erase_head` · 00554d60 — ac2-script glue::lists
- [x] `gRList::CallEFun_erase_tail` · 00554e10 — ac2-script glue::lists
- [x] `gRList::CallEFun_flush` · 00554ec0 — ac2-script glue::lists
- [x] `gRList::CallEFun_insert` · 00554f20 — ac2-script glue::lists
- [x] `gRList::CallEFun_push_unique_back` · 00554fa0 — ac2-script glue::lists
- [x] `gRList::CallEFun_push_unique_front` · 00555040 — ac2-script glue::lists
- [x] `gRList::CallEFun_insert_unique_back` · 005550e0 — ac2-script glue::lists
- [x] `gRList::CallEFun_insert_unique_front` · 00555180 — ac2-script glue::lists
- [x] `gRList::CallEFun_insert_unique_head` · 00555220 — ac2-script glue::lists
- [x] `gRList::CallEFun_insert_unique_tail` · 005552c0 — ac2-script glue::lists
- [x] `gRList::CallEFun_push_unique_head` · 00555360 — ac2-script glue::lists
- [x] `gRList::CallEFun_push_unique_tail` · 00555400 — ac2-script glue::lists

### BaseList<unsigned_long,102>
- [ ] `BaseList<unsigned_long,102>::get_begin` · 005554a0 · legions 0052ee40
- [ ] `BaseList<unsigned_long,102>::get_reverse` · 005554f0 · legions 0052ee90
- [ ] `BaseList<unsigned_long,102>::get_end` · 00555540 · legions 0052eee0

### gRList
- [x] `gRList::CallEFun_get_begin` · 00555590 — ac2-script glue::lists
- [x] `gRList::CallEFun_get_end` · 00555610 — ac2-script glue::lists
- [x] `gRList::CallEFun_get_reverse` · 00555690 — ac2-script glue::lists
- [x] `gRList::CallEFun_clone` · 00555710 — ac2-script glue::lists
- [x] `gRList::CallEFun_inq_data` · 005557b0 — ac2-script glue::lists
- [x] `gRList::CallEFun_peek_back` · 00555850 — ac2-script glue::lists
- [x] `gRList::CallEFun_peek_front` · 00555900 — ac2-script glue::lists
- [x] `gRList::CallEFun_peek_head` · 005559b0 — ac2-script glue::lists
- [x] `gRList::CallEFun_peek_tail` · 00555a60 — ac2-script glue::lists
- [x] `gRList::CallEFun_pop_back` · 00555b10 — ac2-script glue::lists
- [x] `gRList::CallEFun_pop_front` · 00555bb0 — ac2-script glue::lists
- [x] `gRList::CallEFun_pop_head` · 00555c90 — ac2-script glue::lists
- [x] `gRList::CallEFun_pop_tail` · 00555d70 — ac2-script glue::lists
- [x] `gRList::CallEFun_remove_back` · 00555e10 — ac2-script glue::lists
- [x] `gRList::CallEFun_remove_front` · 00555eb0 — ac2-script glue::lists
- [x] `gRList::CallEFun_remove_head` · 00555f90 — ac2-script glue::lists
- [x] `gRList::CallEFun_remove_tail` · 00556070 — ac2-script glue::lists

## `portal/shared/src/interp/glue/grlistiter.cpp.c` (0/7)

### gRListIter
- [~] `gRListIter::CallEFun_next` · 005540c0 — ac2-script glue::lists: iterators hold an index, not a node
- [~] `gRListIter::CallEFun_prev` · 00554120 — ac2-script glue::lists: iterators hold an index, not a node
- [~] `gRListIter::CallEFun_data` · 00554180 — ac2-script glue::lists: iterators hold an index, not a node
- [~] `gRListIter::CallEFun_equals` · 005541f0 — ac2-script glue::lists: iterators hold an index, not a node
- [~] `gRListIter::CallEFun_notdone` · 00554270 — ac2-script glue::lists: iterators hold an index, not a node
- [~] `gRListIter::CallEFun_insert` · 005542e0 — ac2-script glue::lists: iterators hold an index, not a node
- [~] `gRListIter::CallEFun_remove` · 00554390 — ac2-script glue::lists: iterators hold an index, not a node

## `portal/shared/src/interp/glue/gsetpositionstruct.cpp.c` (0/10)

### gSetPositionStruct
- [ ] `gSetPositionStruct::CallEFun_AddFlags` · 00539840
- [ ] `gSetPositionStruct::CallEFun_RemoveFlags` · 005398a0
- [ ] `gSetPositionStruct::CallEFun_SetFlags` · 00539900
- [ ] `gSetPositionStruct::CallEFun_SetLine` · 00539960
- [ ] `gSetPositionStruct::CallEFun_SetLineByPos` · 005399c0
- [ ] `gSetPositionStruct::CallEFun_SetNumTries` · 00539a30
- [ ] `gSetPositionStruct::CallEFun_SetPosition` · 00539a90
- [ ] `gSetPositionStruct::CallEFun_SetScatterRadius` · 00539af0
- [ ] `gSetPositionStruct::CallEFun_GetFlags` · 00539b60
- [ ] `gSetPositionStruct::CallEFun_IsFlagSet` · 00539bd0

## `portal/shared/src/interp/glue/gsphere.cpp.c` (0/3)

### gSphere
- [ ] `gSphere::CallEFun_Intersects` · 005396a0
- [ ] `gSphere::CallEFun_get_radius` · 00539720
- [ ] `gSphere::CallEFun_get_center` · 00539790

## `portal/shared/src/interp/glue/gstoryhookdata.cpp.c` (0/12)

### gStoryHookData
- [ ] `gStoryHookData::CallEFun_SetAnimHookNumber` · 00539220
- [ ] `gStoryHookData::CallEFun_SetAttackHookNumber` · 00539280
- [ ] `gStoryHookData::CallEFun_SetAttackResult` · 005392e0
- [ ] `gStoryHookData::CallEFun_SetAttackerHealthChange` · 00539340
- [ ] `gStoryHookData::CallEFun_SetAttackerPKDamageChange` · 005393a0
- [ ] `gStoryHookData::CallEFun_SetTargetHealthChange` · 00539400
- [ ] `gStoryHookData::CallEFun_SetTargetID` · 00539460
- [ ] `gStoryHookData::CallEFun_SetTargetPKDamageChange` · 005394c0
- [ ] `gStoryHookData::CallEFun_SetTypeToCreateMissiles` · 00539520
- [ ] `gStoryHookData::CallEFun_SetTypeToDamage` · 00539580
- [ ] `gStoryHookData::CallEFun_SetTypeToFireMissile` · 005395e0
- [ ] `gStoryHookData::CallEFun_SetWeaponID` · 00539640

## `portal/shared/src/interp/glue/gstringinfo.cpp.c` (29/38)

### gStringInfo
- [x] `gStringInfo::CallEFun_AddVariable_Float` · 00538010 — ac2-script glue::string_info (lookups via ScriptData)
- [x] `gStringInfo::CallEFun_AddVariable_FormattedFloat` · 00538080 — ac2-script glue::string_info (lookups via ScriptData)
- [x] `gStringInfo::CallEFun_AddVariable_FormattedInt` · 005380f0 — ac2-script glue::string_info (lookups via ScriptData)
- [ ] `gStringInfo::CallEFun_AddVariable_FormattedLInt` · 00538160
- [x] `gStringInfo::CallEFun_AddVariable_FormattedUInt` · 005381d0 — ac2-script glue::string_info (lookups via ScriptData)
- [ ] `gStringInfo::CallEFun_AddVariable_FormattedULInt` · 00538240
- [x] `gStringInfo::CallEFun_AddVariable_Int` · 005382b0 — ac2-script glue::string_info (lookups via ScriptData)
- [ ] `gStringInfo::CallEFun_AddVariable_LInt` · 00538310
- [x] `gStringInfo::CallEFun_AddVariable_StringIDandTableEnum` · 00538370 — ac2-script glue::string_info (lookups via ScriptData)
- [x] `gStringInfo::CallEFun_AddVariable_StringInfo` · 005383d0 — ac2-script glue::string_info (lookups via ScriptData)
- [x] `gStringInfo::CallEFun_AddVariable_UInt` · 00538440 — ac2-script glue::string_info (lookups via ScriptData)
- [ ] `gStringInfo::CallEFun_AddVariable_ULInt` · 005384a0
- [ ] `gStringInfo::CallEFun_ApplyMetasToPlayerName` · 00538500
- [x] `gStringInfo::CallEFun_Reset` · 00538560 — ac2-script glue::string_info (lookups via ScriptData)
- [x] `gStringInfo::CallEFun_SetLiteralValue` · 005385c0 — ac2-script glue::string_info (lookups via ScriptData)
- [x] `gStringInfo::CallEFun_SetStringID` · 00538620 — ac2-script glue::string_info (lookups via ScriptData)
- [x] `gStringInfo::CallEFun_SetStringIDandTableEnum` · 00538680 — ac2-script glue::string_info (lookups via ScriptData)
- [x] `gStringInfo::CallEFun_SetStringIDandTableID` · 005386e0 — ac2-script glue::string_info (lookups via ScriptData)
- [x] `gStringInfo::CallEFun_SetStringIDandTableName` · 00538740 — ac2-script glue::string_info (lookups via ScriptData)
- [x] `gStringInfo::CallEFun_SetTableEnum` · 005387b0 — ac2-script glue::string_text (tables via ScriptData::string_table)
- [x] `gStringInfo::CallEFun_SetTableName` · 00538810 — ac2-script glue::string_info (lookups via ScriptData)
- [x] `gStringInfo::CallEFun_AddVariable_String` · 00538870 — ac2-script glue::string_info (lookups via ScriptData)
- [x] `gStringInfo::CallEFun_GetStringID` · 005388f0 — ac2-script glue::string_info (lookups via ScriptData)
- [x] `gStringInfo::CallEFun_GetTableID` · 00538960 — ac2-script glue::string_info (lookups via ScriptData)
- [x] `gStringInfo::CallEFun_HasStringID` · 005389d0 — ac2-script glue::string_info (lookups via ScriptData)
- [x] `gStringInfo::CallEFun_HasStringIDandTableID` · 00538a40 — ac2-script glue::string_info (lookups via ScriptData)
- [x] `gStringInfo::CallEFun_HasTableID` · 00538ad0 — ac2-script glue::string_info (lookups via ScriptData)
- [x] `gStringInfo::CallEFun_IsLiteralValue` · 00538b40 — ac2-script glue::string_info (lookups via ScriptData)
- [x] `gStringInfo::CallEFun_IsValid` · 00538bb0 — ac2-script glue::string_text (tables via ScriptData::string_table)
- [x] `gStringInfo::CallEFun_eq` · 00538c20 — ac2-script glue::string_text (tables via ScriptData::string_table)
- [x] `gStringInfo::CallEFun_ne` · 00538ca0 — ac2-script glue::string_text (tables via ScriptData::string_table)
- [~] `gStringInfo::CallEFun_GetLogString` · 00538d20 — ac2-script glue::string_text; table rendering is the ScriptData::render_string hook (plain strings only by default)
- [x] `gStringInfo::CallEFun_GetString` · 00538de0 — ac2-script glue::string_text (tables via ScriptData::string_table)
- [ ] `gStringInfo::CallEFun_GetStringCaseInsensitive` · 00538ea0
- [x] `gStringInfo::CallEFun_GetStringWithMetaTags` · 00538fa0 · legions 0051c0d0 — ac2-script glue::string_text (tables via ScriptData::string_table)
- [ ] `gStringInfo::CallEFun_FromFileNode` · 00539060 · legions 0051c160
- [ ] `gStringInfo::CallEFun_GetSubDataIDs` · 00539100 · legions 0051c1d0
- [x] `gStringInfo::CallEFun_InqString` · 00539180 — ac2-script glue::string_text (tables via ScriptData::string_table)

## `portal/shared/src/interp/glue/gstringutils.cpp.c` (0/3)

### gStringUtils
- [ ] `gStringUtils::CallEFun_ConvertWPStringToCIWPstring` · 00551ee0

### (free functions)
- [ ] `IsWordInString` · 00551fa0 · legions 0052d1d0

### gStringUtils
- [ ] `gStringUtils::CallEFun_IsWordInString` · 005520c0

## `portal/shared/src/interp/glue/gtabootableadaptor.cpp.c` (0/3)

### gTabooTableAdaptor
- [ ] `gTabooTableAdaptor::CallEFun_CheckCharactersW` · 00537ed0
- [ ] `gTabooTableAdaptor::CallEFun_CheckCensorsW` · 00537f20
- [ ] `gTabooTableAdaptor::CallEFun_FormatStringForAudienceW` · 00537fa0

## `portal/shared/src/interp/glue/gtimer.cpp.c` (0/3)

### gTimer
- [ ] `gTimer::CallEFun_compute_time` · 00554020
- [ ] `gTimer::CallEFun_get_real_time` · 00554050
- [ ] `gTimer::CallEFun_get_time` · 00554080

## `portal/shared/src/interp/glue/gtokenize.cpp.c` (0/1)

### gTokenize
- [ ] `gTokenize::CallEFun_Tokenize` · 00537e90

## `portal/shared/src/interp/glue/gtypeinfo.cpp.c` (0/3)

### gTypeInfo
- [ ] `gTypeInfo::CallEFun_get_type` · 00553f30
- [ ] `gTypeInfo::CallEFun_new_type` · 00553f80
- [ ] `gTypeInfo::CallEFun_new_validated_type` · 00553fd0

## `portal/shared/src/interp/glue/guisavelocations.cpp.c` (0/1)

### gUISaveLocations
- [ ] `gUISaveLocations::CallEFun_Clear` · 00537e30

## `portal/shared/src/interp/glue/gvector.cpp.c` (14/15)

### gVector
- [x] `gVector::CallEFun_Add` · 005376c0 — ac2-script glue::math / glue::missile
- [x] `gVector::CallEFun_Copy` · 00537730 — ac2-script glue::math / glue::missile
- [x] `gVector::CallEFun_MagnitudeSquared` · 005377a0 — ac2-script glue::math / glue::missile
- [~] `gVector::CallEFun_Normalize` · 005377e0 — ac2-script glue::math: Rust f64 transcendentals / D3DXVec3Normalize, not the x87 instruction results
- [x] `gVector::CallEFun_Scale` · 00537840 — ac2-script glue::math / glue::missile
- [x] `gVector::CallEFun_Set` · 005378b0 — ac2-script glue::math / glue::missile
- [x] `gVector::CallEFun_Subtract` · 00537920 — ac2-script glue::math / glue::missile
- [x] `gVector::CallEFun_Dot` · 00537990 — ac2-script glue::math / glue::missile
- [x] `gVector::CallEFun_Equals` · 00537a20 — ac2-script glue::math / glue::missile
- [x] `gVector::CallEFun_GetX` · 00537af0 — ac2-script glue::math / glue::missile
- [x] `gVector::CallEFun_GetY` · 00537b60 — ac2-script glue::math / glue::missile
- [x] `gVector::CallEFun_GetZ` · 00537bd0 — ac2-script glue::math / glue::missile
- [x] `gVector::CallEFun_Magnitude` · 00537c40 — ac2-script glue::math / glue::missile
- [x] `gVector::CallEFun_Cross` · 00537cd0 — ac2-script glue::math / glue::missile
- [x] `gVector::CallEFun_GetStringW` · 00537db0 — ac2-script glue::math / glue::missile

## `portal/shared/src/interp/glue/gvisualdesc.cpp.c` (0/13)

### gVisualDesc
- [ ] `gVisualDesc::CallEFun_Clear` · 00537020
- [ ] `gVisualDesc::CallEFun_SetScale` · 00537080
- [ ] `gVisualDesc::CallEFun_AddAppInfoHashToPartGroup` · 005370f0
- [ ] `gVisualDesc::CallEFun_AddAppearanceKey` · 00537170
- [ ] `gVisualDesc::CallEFun_ApplyAdditiveVDesc` · 005371f0
- [ ] `gVisualDesc::CallEFun_ApplySubtractiveVDesc` · 00537270
- [ ] `gVisualDesc::CallEFun_Empty` · 005372f0
- [ ] `gVisualDesc::CallEFun_RemoveAppearanceKey` · 00537360
- [ ] `gVisualDesc::CallEFun_ReplaceVisualDesc` · 005373f0
- [ ] `gVisualDesc::CallEFun_GetScale` · 00537470
- [ ] `gVisualDesc::CallEFun_GetTotalIconDesc` · 00537520 · legions 0051b090
- [ ] `gVisualDesc::CallEFun_GetVisualDescFromDB` · 005375c0
- [ ] `gVisualDesc::CallEFun_InqAppInfoHashForPartGroup` · 00537620 · legions 0051b160

## `portal/shared/src/interp/glue/gvm.cpp.c` (12/24)

### gVM
- [~] `gVM::CallEFun_GarbageCollect` · 00553840 — ac2-script glue: no collector/debugger
- [~] `gVM::CallEFun_SetActiveGC` · 00553850 — ac2-script glue: no collector/debugger
- [~] `gVM::CallEFun_SetAggressiveGCDeletion` · 00553870 — ac2-script glue: no collector/debugger
- [x] `gVM::CallEFun_CloneReference` · 00553890 · legions 0052e1c0 — ac2-script vm::pack (dependent-reference walk, CloneReference)
- [~] `gVM::CallEFun_DeleteAllUnUsedReferences` · 005538d0 — ac2-script glue: no collector/debugger
- [x] `gVM::CallEFun_GetContextID` · 00553900 — ac2-script glue
- [x] `gVM::CallEFun_GetDataID` · 00553930 — ac2-script glue
- [x] `gVM::CallEFun_GetPackageID` · 00553990 — ac2-script glue
- [x] `gVM::CallEFun_GetPackageReference` · 005539d0 · legions 0052e300 — ac2-script glue
- [x] `gVM::CallEFun_GetSingletonPackageByID` · 00553a10 · legions 0052e340 — ac2-script glue
- [x] `gVM::CallEFun_GetSingletonPackageW` · 00553a50 — ac2-script glue
- [x] `gVM::CallEFun_IsDerivedFrom` · 00553aa0 — ac2-script glue
- [x] `gVM::CallEFun_IsInstanceOf` · 00553ae0 — ac2-script glue
- [x] `gVM::CallEFun_WSL_EFun_GetInstanceReference` · 00553b20 · legions 0052e450 — ac2-script glue
- [x] `gVM::CallEFun_GetPackageNameByPkgIDW` · 00553b50 · legions 0052e480 — ac2-script glue
- [~] `gVM::CallEFun_GetStackTrace` · 00553bd0 · legions 0052e500 — ac2-script glue: no collector/debugger
- [x] `gVM::CallEFun_GetPackageIDByRef` · 00553c50 — ac2-script glue
- [ ] `gVM::CallEFun_QueryPerfW` · 00553c90
- [ ] `gVM::CallEFun_QueryProcessTableW` · 00553cf0
- [ ] `gVM::CallEFun_QueryProcessW` · 00553d50
- [ ] `gVM::CallEFun_QueryProcessesW` · 00553db0
- [ ] `gVM::CallEFun_QueryReferenceTreeForProcessW` · 00553e10
- [ ] `gVM::CallEFun_QueryReferenceTreeW` · 00553e70
- [ ] `gVM::CallEFun_QueryReferencesW` · 00553ed0

## `portal/shared/src/interp/glue/gvmdata.cpp.c` (0/4)

### gVMData
- [ ] `gVMData::CallEFun_AddData` · 00536e50
- [ ] `gVMData::CallEFun_GetData` · 00536eb0
- [ ] `gVMData::CallEFun_HasData` · 00536f30
- [ ] `gVMData::CallEFun_RemoveData` · 00536fa0

## `portal/shared/src/interp/glue/gwccinternal.cpp.c` (10/10)

### gwccinternal
- [x] `gwccinternal::CallEFun_ToHex` · 005532b0 · legions 0052dbe0 — ac2-script glue
- [x] `gwccinternal::CallEFun_ToIID` · 00553330 · legions 0052dc60 — ac2-script glue
- [x] `gwccinternal::CallEFun_ToLHex` · 005533b0 · legions 0052dce0 — ac2-script glue
- [x] `gwccinternal::CallEFun__WCC_strcat_float` · 00553430 · legions 0052dd60 — ac2-script glue
- [x] `gwccinternal::CallEFun__WCC_strcat_int` · 005534c0 · legions 0052ddf0 — ac2-script glue
- [x] `gwccinternal::CallEFun__WCC_strcat_lfloat` · 00553550 · legions 0052de80 — ac2-script glue
- [x] `gwccinternal::CallEFun__WCC_strcat_lint` · 005535f0 · legions 0052df20 — ac2-script glue
- [x] `gwccinternal::CallEFun__WCC_strcat_str` · 00553680 — ac2-script glue
- [x] `gwccinternal::CallEFun__WCC_strcat_uint` · 00553720 · legions 0052e050 — ac2-script glue
- [x] `gwccinternal::CallEFun__WCC_strcat_ulint` · 005537b0 · legions 0052e0e0 — ac2-script glue

## `portal/shared/src/interp/glue/gwlogsystem.cpp.c` (12/12)

### gWLogSystem
- [x] `gWLogSystem::CallEFun_AddFileLog` · 00536c30 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gWLogSystem::CallEFun_AddPrintLog` · 00536c50 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gWLogSystem::CallEFun_RemoveFileLog` · 00536c70 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gWLogSystem::CallEFun_RemovePrintLog` · 00536c90 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gWLogSystem::CallEFun_SetFileLog` · 00536cb0 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gWLogSystem::CallEFun_SetPrintLog` · 00536cd0 — ac2-script glue (wlog, random, behavior, enums: gDBCache)

### (free functions)
- [x] `DLogMsg` · 00536cf0 · legions 0051aa60 — ac2-script glue (wlog, random, behavior, enums: gDBCache)

### gWLogSystem
- [x] `gWLogSystem::CallEFun_GetFileLog` · 00536d50 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gWLogSystem::CallEFun_GetPrintLog` · 00536d80 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gWLogSystem::CallEFun_IsFileLogging` · 00536db0 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gWLogSystem::CallEFun_IsPrintLogging` · 00536de0 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gWLogSystem::CallEFun_DLogMsg` · 00536e10 — ac2-script glue (wlog, random, behavior, enums: gDBCache)

## `portal/shared/src/interp/glue/gwpstring.cpp.c` (34/34)

### gwpstring
- [x] `gwpstring::CallEFun_clear` · 00552130 — ac2-script glue
- [x] `gwpstring::CallEFun_cmp` · 005521c0 — ac2-script glue
- [x] `gwpstring::CallEFun_empty` · 00552270 — ac2-script glue
- [x] `gwpstring::CallEFun_eq` · 005522e0 — ac2-script glue
- [x] `gwpstring::CallEFun_find_substring` · 00552370 — ac2-script glue
- [x] `gwpstring::CallEFun_ge` · 00552410 — ac2-script glue
- [x] `gwpstring::CallEFun_gt` · 005524a0 — ac2-script glue
- [x] `gwpstring::CallEFun_hash` · 00552530 — ac2-script glue
- [x] `gwpstring::CallEFun_le` · 005525b0 — ac2-script glue
- [x] `gwpstring::CallEFun_length` · 00552640 — ac2-script glue
- [x] `gwpstring::CallEFun_lt` · 005526b0 — ac2-script glue
- [x] `gwpstring::CallEFun_ne` · 00552740 — ac2-script glue
- [x] `gwpstring::CallEFun_set` · 005527d0 — ac2-script glue
- [x] `gwpstring::CallEFun_to_float` · 00552860 — ac2-script glue
- [x] `gwpstring::CallEFun_to_int32` · 005528f0 — ac2-script glue
- [x] `gwpstring::CallEFun_to_int64` · 00552980 — ac2-script glue
- [x] `gwpstring::CallEFun_to_uint32` · 00552a10 — ac2-script glue
- [x] `gwpstring::CallEFun_to_uint64` · 00552aa0 — ac2-script glue
- [x] `gwpstring::CallEFun_decrypt` · 00552b30 — ac2-script glue
- [x] `gwpstring::CallEFun_encrypt` · 00552b90 — ac2-script glue
- [x] `gwpstring::CallEFun_tolower` · 00552bf0 — ac2-script glue
- [x] `gwpstring::CallEFun_toupper` · 00552c60 — ac2-script glue
- [x] `gwpstring::CallEFun_append_double` · 00552d50 — ac2-script glue
- [x] `gwpstring::CallEFun_append_float` · 00552db0 — ac2-script glue
- [x] `gwpstring::CallEFun_append_int32` · 00552e10 — ac2-script glue
- [x] `gwpstring::CallEFun_append_int32_hex` · 00552e70 — ac2-script glue
- [x] `gwpstring::CallEFun_append_int64` · 00552ed0 — ac2-script glue
- [x] `gwpstring::CallEFun_append_string` · 00552f30 — ac2-script glue
- [x] `gwpstring::CallEFun_append_uint32` · 00552fb0 — ac2-script glue
- [x] `gwpstring::CallEFun_append_uint32_hex` · 00553010 — ac2-script glue
- [x] `gwpstring::CallEFun_trim` · 00553070 — ac2-script glue
- [x] `gwpstring::CallEFun_append_bool` · 005530f0 — ac2-script glue
- [x] `gwpstring::CallEFun_substring` · 00553150 — ac2-script glue
- [x] `gwpstring::CallEFun_replace` · 00553220 — ac2-script glue

## `portal/shared/src/interp/glue/gwstate.cpp.c` (0/2)

### gWState
- [ ] `gWState::CallEFun_SaveStateByType` · 00536bc0
- [ ] `gWState::CallEFun_GetStateByType` · 00536bf0 · legions 0051a960

## `portal/shared/src/interp/glue/nativefactory.cpp.c` (0/2)

### NativeFactory
- [ ] `NativeFactory::NativeFactory` · 0054cbd0 · legions 0052a380
- [ ] `NativeFactory::~NativeFactory` · 0054cbf0 · legions 0052a3a0

## `portal/shared/src/interp/heapobject.cpp.c` (2/6)

### HeapObject
- [ ] `HeapObject::Clear` · 006dba60 · legions 00664f10
- [ ] `HeapObject::GetPackSize` · 006dbab0 · legions 00664f60
- [x] `HeapObject::Pack` · 006dbb00 · legions 00664fb0 — ac2-net protocol::pack
- [x] `HeapObject::UnPack` · 006dbbd0 · legions 00665080 — ac2-script reference/native unpack
- [ ] `HeapObject::UpdatePackageReferences` · 006dbd50 · legions 00665200
- [ ] `HeapObject::UpdateReferences` · 006dbdd0 · legions 00665280

## `portal/shared/src/interp/interp_binop.cpp.c` (71/71)

### ContextState
- [x] `ContextState::PushDFloat` · 006e2280 · legions 006689f0 — ac2-script vm::arith / Context

### Interp
- [x] `Interp::HandleBinOp_O_ADD` · 006e22d0 — ac2-script vm::arith
- [x] `Interp::HandleBinOp_O_SUB` · 006e2310 — ac2-script vm::arith
- [x] `Interp::HandleBinOp_O_MULT` · 006e2350 — ac2-script vm::arith
- [x] `Interp::HandleBinOp_O_DIV` · 006e2390 — ac2-script vm::arith
- [x] `Interp::HandleBinOp_O_MOD` · 006e23d0 — ac2-script vm::arith
- [x] `Interp::HandleBinOp_O_L_OR` · 006e2410 — ac2-script vm::arith
- [x] `Interp::HandleBinOp_O_L_AND` · 006e2450 — ac2-script vm::arith
- [x] `Interp::HandleBinOp_O_L_EQ` · 006e2490 — ac2-script vm::arith
- [x] `Interp::HandleBinOp_O_L_NEQ` · 006e24d0 — ac2-script vm::arith
- [x] `Interp::HandleBinOp_O_L_SLT` · 006e2510 — ac2-script vm::arith
- [x] `Interp::HandleBinOp_O_L_SGT` · 006e2550 — ac2-script vm::arith
- [x] `Interp::HandleBinOp_O_L_SLTE` · 006e2590 — ac2-script vm::arith
- [x] `Interp::HandleBinOp_O_L_SGTE` · 006e25d0 — ac2-script vm::arith
- [x] `Interp::HandleBinOp_O_L_ULT` · 006e2610 — ac2-script vm::arith
- [x] `Interp::HandleBinOp_O_L_UGT` · 006e2650 — ac2-script vm::arith
- [x] `Interp::HandleBinOp_O_L_ULTE` · 006e2690 — ac2-script vm::arith
- [x] `Interp::HandleBinOp_O_L_UGTE` · 006e26d0 — ac2-script vm::arith
- [x] `Interp::HandleBinOp_O_B_OR` · 006e2710 — ac2-script vm::arith
- [x] `Interp::HandleBinOp_O_B_XOR` · 006e2750 — ac2-script vm::arith
- [x] `Interp::HandleBinOp_O_B_AND` · 006e2790 — ac2-script vm::arith
- [x] `Interp::HandleBinOp_O_B_LSHIFT` · 006e27d0 — ac2-script vm::arith
- [x] `Interp::HandleBinOp_O_B_RSHIFT` · 006e2810 — ac2-script vm::arith
- [x] `Interp::HandleBinOp_O_B_URSHIFT` · 006e2850 — ac2-script vm::arith
- [x] `Interp::HandleBinOp_Long_O_ADD` · 006e2890 — ac2-script vm::arith
- [x] `Interp::HandleBinOp_Long_O_SUB` · 006e28f0 — ac2-script vm::arith
- [x] `Interp::HandleBinOp_Long_O_MULT` · 006e2950 — ac2-script vm::arith
- [x] `Interp::HandleBinOp_Long_O_DIV` · 006e29b0 — ac2-script vm::arith
- [x] `Interp::HandleBinOp_Long_O_MOD` · 006e2a10 — ac2-script vm::arith
- [x] `Interp::HandleBinOp_Long_O_B_OR` · 006e2a70 — ac2-script vm::arith
- [x] `Interp::HandleBinOp_Long_O_B_XOR` · 006e2ad0 — ac2-script vm::arith
- [x] `Interp::HandleBinOp_Long_O_B_AND` · 006e2b30 — ac2-script vm::arith
- [x] `Interp::HandleBinOp_Long_O_B_LSHIFT` · 006e2b90 — ac2-script vm::arith
- [x] `Interp::HandleBinOp_Long_O_B_RSHIFT` · 006e2c00 — ac2-script vm::arith
- [x] `Interp::HandleBinOp_Long_O_B_URSHIFT` · 006e2c80 — ac2-script vm::arith
- [x] `Interp::HandleBinLogicalOp_Long_O_L_OR` · 006e2cf0 — ac2-script vm::arith
- [x] `Interp::HandleBinLogicalOp_Long_O_L_AND` · 006e2d50 — ac2-script vm::arith
- [x] `Interp::HandleBinLogicalOp_Long_O_L_EQ` · 006e2db0 — ac2-script vm::arith
- [x] `Interp::HandleBinLogicalOp_Long_O_L_NEQ` · 006e2e10 — ac2-script vm::arith
- [x] `Interp::HandleBinLogicalOp_Long_O_L_SLT` · 006e2e70 — ac2-script vm::arith
- [x] `Interp::HandleBinLogicalOp_Long_O_L_SGT` · 006e2ed0 — ac2-script vm::arith
- [x] `Interp::HandleBinLogicalOp_Long_O_L_SLTE` · 006e2f30 — ac2-script vm::arith
- [x] `Interp::HandleBinLogicalOp_Long_O_L_SGTE` · 006e2f90 — ac2-script vm::arith
- [x] `Interp::HandleBinLogicalOp_Long_O_L_ULT` · 006e2ff0 — ac2-script vm::arith
- [x] `Interp::HandleBinLogicalOp_Long_O_L_UGT` · 006e3050 — ac2-script vm::arith
- [x] `Interp::HandleBinLogicalOp_Long_O_L_ULTE` · 006e30b0 — ac2-script vm::arith
- [x] `Interp::HandleBinLogicalOp_Long_O_L_UGTE` · 006e3110 — ac2-script vm::arith
- [x] `Interp::HandleFBinOp_O_FADD` · 006e3170 — ac2-script vm::arith
- [x] `Interp::HandleFBinOp_O_FSUB` · 006e31c0 — ac2-script vm::arith
- [x] `Interp::HandleFBinOp_O_FMULT` · 006e3210 — ac2-script vm::arith
- [x] `Interp::HandleFBinOp_O_FDIV` · 006e3260 — ac2-script vm::arith
- [x] `Interp::HandleFBinLogicalOp_O_L_FOR` · 006e32b0 — ac2-script vm::arith
- [x] `Interp::HandleFBinLogicalOp_O_L_FAND` · 006e3330 — ac2-script vm::arith
- [x] `Interp::HandleFBinLogicalOp_O_L_FEQ` · 006e33b0 — ac2-script vm::arith
- [x] `Interp::HandleFBinLogicalOp_O_L_FNEQ` · 006e3420 — ac2-script vm::arith
- [x] `Interp::HandleFBinLogicalOp_O_L_FLT` · 006e3490 — ac2-script vm::arith
- [x] `Interp::HandleFBinLogicalOp_O_L_FGT` · 006e3500 — ac2-script vm::arith
- [x] `Interp::HandleFBinLogicalOp_O_L_FLTE` · 006e3570 — ac2-script vm::arith
- [x] `Interp::HandleFBinLogicalOp_O_L_FGTE` · 006e35e0 — ac2-script vm::arith
- [x] `Interp::HandleFBinOp_Long_O_FADD` · 006e3650 — ac2-script vm::arith
- [x] `Interp::HandleFBinOp_Long_O_FSUB` · 006e36d0 — ac2-script vm::arith
- [x] `Interp::HandleFBinOp_Long_O_FMULT` · 006e3750 — ac2-script vm::arith
- [x] `Interp::HandleFBinOp_Long_O_FDIV` · 006e37d0 — ac2-script vm::arith
- [x] `Interp::HandleFBinLogicalOp_Long_O_L_FOR` · 006e3850 — ac2-script vm::arith
- [x] `Interp::HandleFBinLogicalOp_Long_O_L_FAND` · 006e38d0 — ac2-script vm::arith
- [x] `Interp::HandleFBinLogicalOp_Long_O_L_FEQ` · 006e3950 — ac2-script vm::arith
- [x] `Interp::HandleFBinLogicalOp_Long_O_L_FNEQ` · 006e39c0 — ac2-script vm::arith
- [x] `Interp::HandleFBinLogicalOp_Long_O_L_FLT` · 006e3a30 — ac2-script vm::arith
- [x] `Interp::HandleFBinLogicalOp_Long_O_L_FGT` · 006e3aa0 — ac2-script vm::arith
- [x] `Interp::HandleFBinLogicalOp_Long_O_L_FLTE` · 006e3b10 — ac2-script vm::arith
- [x] `Interp::HandleFBinLogicalOp_Long_O_L_FGTE` · 006e3b80 — ac2-script vm::arith

## `portal/shared/src/interp/interp_core.cpp.c` (28/31)

### FileNodeName_Int32
- [ ] `FileNodeName_Int32::GetType` · 006e3bf0 · legions 0066a360

### Interp
- [x] `Interp::HandlePop` · 006e3c00 — ac2-script vm::exec
- [x] `Interp::HandlePopN` · 006e3c10 · legions 0066a380 — ac2-script vm::exec
- [x] `Interp::HandlePopFrame` · 006e3c30 — ac2-script vm::exec
- [x] `Interp::HandleCallStatic` · 006e3c70 — ac2-script vm::exec
- [ ] `Interp::HandleCallEFun` · 006e3cc0
- [x] `Interp::HandleRJmp` · 006e3d40 — ac2-script vm::exec

### FileNodeName_Bool
- [ ] `FileNodeName_Bool::GetPackSize` · 006e3d50 · legions 0066a4c0

### Interp
- [x] `Interp::HandlePush` · 006e3d60 — ac2-script vm::exec
- [x] `Interp::HandlePush_Long` · 006e3d90 · legions 0066a500 — ac2-script vm::exec
- [x] `Interp::HandlePushV` · 006e3e00 — ac2-script vm::exec
- [x] `Interp::HandlePushV_Long` · 006e3e30 — ac2-script vm::exec
- [x] `Interp::HandlePushFrame` · 006e3e80 — ac2-script vm::exec
- [x] `Interp::HandleLoad` · 006e3ed0 · legions 0066a640 — ac2-script vm::exec
- [x] `Interp::HandleLoad_Long` · 006e3f10 — ac2-script vm::exec
- [x] `Interp::HandleLoadThis` · 006e3f80 — ac2-script vm::exec
- [x] `Interp::HandleNewString` · 006e3fb0 — ac2-script vm::exec
- [x] `Interp::HandleStore` · 006e3ff0 · legions 0066a760 — ac2-script vm::exec
- [x] `Interp::HandleStore_Long` · 006e4040 — ac2-script vm::exec
- [x] `Interp::HandleIf` · 006e4090 — ac2-script vm::exec
- [x] `Interp::HandleSwitch` · 006e40b0 · legions 0066a820 — ac2-script vm::exec
- [x] `Interp::HandleSAddr` · 006e4120 — ac2-script vm::exec
- [x] `Interp::HandleRAddr` · 006e4190 — ac2-script vm::exec
- [x] `Interp::HandleNewNative` · 006e41f0 — ac2-script vm::exec
- [x] `Interp::HandleCall` · 006e4240 · legions 0066a9b0 — ac2-script vm::exec

### ReferenceTable
- [x] `ReferenceTable::GetReferenceTagAndPtr` · 006e42f0 · legions 0066aa60 — ac2-script vm::exec

### Interp
- [x] `Interp::HandleRLoad` · 006e43b0 · legions 0066ab20 — ac2-script vm::exec
- [x] `Interp::HandleRLoad_Long` · 006e4410 — ac2-script vm::exec
- [x] `Interp::HandleRStore` · 006e4490 — ac2-script vm::exec
- [x] `Interp::HandleRStore_Long` · 006e44f0 — ac2-script vm::exec
- [x] `Interp::HandleNew` · 006e4560 — ac2-script vm::exec

## `portal/shared/src/interp/interp_exec.cpp.c` (1/4)

### Interp
- [~] `Interp::AddLongHandler` · 006de120 — ac2-script vm: no GC/reference reuse, no VMMon timers, efun ids resolved per call
- [~] `Interp::SetupHandlers` · 006de1e0 — ac2-script vm: no GC/reference reuse, no VMMon timers, efun ids resolved per call
- [~] `Interp::Interp` · 006e08d0 · legions 00667890 — ac2-script: handler table is a match in exec.rs
- [x] `Interp::Exec` · 006e08f0 — ac2-script vm::exec / Program::walk

## `portal/shared/src/interp/interp_optimized.cpp.c` (8/8)

### Interp
- [x] `Interp::HandleOptimized_Store_PopN` · 006e20c0 — ac2-script vm::exec / Program::walk
- [x] `Interp::HandleOptimized_This_CallEFun` · 006e20e0 — ac2-script vm::exec / Program::walk
- [x] `Interp::HandleOptimized_Load_Call` · 006e2100 — ac2-script vm::exec / Program::walk
- [x] `Interp::HandleOptimized_Load_CallEFun` · 006e2120 — ac2-script vm::exec / Program::walk
- [x] `Interp::HandleOptimized_RLoad_Call` · 006e2140 — ac2-script vm::exec / Program::walk
- [x] `Interp::HandleOptimized_RLoad_CallEFun` · 006e2160 — ac2-script vm::exec / Program::walk
- [x] `Interp::HandleOptimized_This_Call` · 006e2180 — ac2-script vm::exec / Program::walk
- [x] `Interp::HandleOptimized_This_RLoad` · 006e2220 · legions 00668990 — ac2-script vm::exec / Program::walk

## `portal/shared/src/interp/interp_unaryop.cpp.c` (13/13)

### ContextState
- [x] `ContextState::PushTimetype` · 006e45a0 · legions 0066ad10 — ac2-script vm::arith / Context

### Interp
- [x] `Interp::HandleUnaryOp_O_NEGATE` · 006e45f0 — ac2-script vm::arith / Context
- [x] `Interp::HandleUnaryOp_O_L_NOT` · 006e4620 — ac2-script vm::arith / Context
- [x] `Interp::HandleUnaryOp_O_B_NEGATE` · 006e4650 — ac2-script vm::arith / Context
- [x] `Interp::HandleUnaryOp_O_INC` · 006e4680 — ac2-script vm::arith / Context
- [x] `Interp::HandleUnaryOp_O_DEC` · 006e46b0 — ac2-script vm::arith / Context
- [x] `Interp::HandleUnaryOp_O_I2F` · 006e46e0 — ac2-script vm::arith / Context
- [x] `Interp::HandleUnaryOp_O_U2F` · 006e4710 — ac2-script vm::arith / Context
- [x] `Interp::HandleUnaryOp_O_I2I` · 006e4750 — ac2-script vm::arith / Context
- [x] `Interp::HandleUnaryOp_O_U2U` · 006e47a0 — ac2-script vm::arith / Context
- [x] `Interp::HandleUnaryOp_Long` · 006e47f0 — ac2-script vm::arith / Context
- [x] `Interp::HandleFUnaryOp` · 006e49c0 — ac2-script vm::arith / Context
- [x] `Interp::HandleFUnaryOp_Long` · 006e4b30 — ac2-script vm::arith / Context

## `portal/shared/src/interp/opcodestream.cpp.c` (1/1)

### OpcodeStream
- [x] `OpcodeStream::WalkStream` · 006ec490 · legions 00671360 — ac2-script vm::exec / Program::walk

## `portal/shared/src/interp/packagestate.cpp.c` (1/5)

### PackageState
- [ ] `PackageState::PackageState` · 006e1630 · legions 00668470
- [ ] `PackageState::Pack` · 006e1650 · legions 00668490
- [ ] `PackageState::Clear` · 006e16a0 · legions 006684e0
- [ ] `PackageState::~PackageState` · 006e16d0 · legions 00668510
- [x] `PackageState::UnPack` · 006e1700 · legions 00668540 — ac2-script WLib/WState/ByteStream decode

## `portal/shared/src/interp/packing/packbuffer.cpp.c` (14/20)

### PackBuffer
- [ ] `PackBuffer::~PackBuffer` · 006ed4f0 · legions 00672300
- [ ] `PackBuffer::operator=` · 006ed500 · legions 00672310

### PackageState
- [ ] `PackageState::GetPackSize` · 006ed580 · legions 00672390

### PackBuffer
- [x] `PackBuffer::UnPackInt` · 006ed590 · legions 006723a0 — ac2-script host (ScriptHost dispatch, PackBuffer, SendServerEvt, notices)
- [x] `PackBuffer::UnPackLInt` · 006ed630 · legions 00672440 — ac2-script host (ScriptHost dispatch, PackBuffer, SendServerEvt, notices)
- [x] `PackBuffer::UnPackFloat` · 006ed700 · legions 00672510 — ac2-script host (ScriptHost dispatch, PackBuffer, SendServerEvt, notices)
- [x] `PackBuffer::UnPackLFloat` · 006ed7b0 · legions 006725c0 — ac2-script host (ScriptHost dispatch, PackBuffer, SendServerEvt, notices)
- [x] `PackBuffer::UnPackPackage` · 006ed870 · legions 00672680 — ac2-script host (ScriptHost dispatch, PackBuffer, SendServerEvt, notices)
- [ ] `PackBuffer::Grow` · 006ed920 · legions 00672730
- [ ] `PackBuffer::PackBuffer` · 006ed9a0 · legions 006727b0
- [x] `PackBuffer::Pack` · 006ed9c0 · legions 006727d0
- [x] `PackBuffer::UnPack` · 006eda20 · legions 00672830
- [ ] `PackBuffer::PackRaw` · 006edaa0
- [x] `PackBuffer::PackPackage` · 006edb40 — ac2-script host (ScriptHost dispatch, PackBuffer, SendServerEvt, notices)
- [x] `PackBuffer::UnPackBuffer_Int` · 006edc00 · legions 00672920 — ac2-script host (ScriptHost dispatch, PackBuffer, SendServerEvt, notices)
- [x] `PackBuffer::UnPackBuffer_LongInt` · 006edc50 · legions 00672970 — ac2-script host (ScriptHost dispatch, PackBuffer, SendServerEvt, notices)
- [x] `PackBuffer::UnPackBuffer_Float` · 006edcc0 · legions 006729e0 — ac2-script host (ScriptHost dispatch, PackBuffer, SendServerEvt, notices)
- [x] `PackBuffer::UnPackBuffer_LongFloat` · 006edd10 · legions 00672a30 — ac2-script host (ScriptHost dispatch, PackBuffer, SendServerEvt, notices)
- [x] `PackBuffer::UnPackBuffer_Package` · 006edd90 · legions 00672ab0 — ac2-script host (ScriptHost dispatch, PackBuffer, SendServerEvt, notices)
- [x] `PackBuffer::UnPackBuffer` · 006edde0 · legions 00672b00 — ac2-script host (ScriptHost dispatch, PackBuffer, SendServerEvt, notices)

## `portal/shared/src/interp/packing/vmdata.cpp.c` (0/13)

### VMData
- [ ] `VMData::VMData` · 006ece90 · legions 00671ca0
- [ ] `VMData::GetReferenceData` · 006ecea0 · legions 00671cb0
- [ ] `VMData::GetPackSize` · 006eceb0
- [ ] `VMData::Pack` · 006ecf30
- [ ] `VMData::GetData` · 006ecff0 · legions 00671e00
- [ ] `VMData::HasData` · 006ed020 · legions 00671e30
- [ ] `VMData::DecrementRefCount_` · 006ed060 · legions 00671e70
- [ ] `VMData::Clear_` · 006ed0d0 · legions 00671ee0
- [ ] `VMData::RemoveData` · 006ed150 · legions 00671f60
- [ ] `VMData::operator=` · 006ed1b0 · legions 00671fc0
- [ ] `VMData::AddData` · 006ed2e0 · legions 006720f0
- [ ] `VMData::VMData` · 006ed400 · legions 00672210
- [ ] `VMData::UnPack` · 006ed420

## `portal/shared/src/interp/perf/vmmon.cpp.c` (0/9)

### VMMon
- [ ] `VMMon::iInitClockDefaults` · 004bc8d0
- [ ] `VMMon::iBeginExtra` · 004bc970
- [ ] `VMMon::iEndExtra` · 004bc9b0
- [ ] `VMMon::~VMMon` · 004bcb60
- [ ] `VMMon::iQuery` · 004bcb80

### SmartArray<PStringBase<char>_>
- [ ] `SmartArray<PStringBase<char>_>::AddToEnd` · 004bccf0 · legions 004e60e0

### VMMon
- [ ] `VMMon::iLog` · 004bcd70
- [ ] `VMMon::iInitMapper` · 004bce20
- [ ] `VMMon::VMMon` · 004bd040

## `portal/shared/src/interp/processstate.cpp.c` (0/14)

### ProcessState
- [ ] `ProcessState::GetPackageID` · 006dd7a0 · legions 00666b90
- [ ] `ProcessState::ClearContextTable` · 006dd7c0 · legions 00666bb0
- [ ] `ProcessState::QueryContexts` · 006dd860
- [ ] `ProcessState::QueryContextReferenceTree` · 006dd9a0
- [ ] `ProcessState::~ProcessState` · 006dda60 · legions 00666ca0
- [ ] `ProcessState::GetContextState` · 006ddab0 · legions 00666cf0
- [ ] `ProcessState::DeleteContext` · 006ddaf0 · legions 00666d30
- [ ] `ProcessState::QueryReferenceTree` · 006ddb60
- [ ] `ProcessState::CreateContext` · 006ddba0 · legions 00666da0
- [ ] `ProcessState::ProcessState` · 006ddcf0 · legions 00666ea0

### ReferenceTable
- [~] `ReferenceTable::AllocatePackage` · 006ddd50 · legions 00666f00 — ac2-script vm: no GC/reference reuse, no VMMon timers, efun ids resolved per call

### ProcessState
- [ ] `ProcessState::ProcessState` · 006dddf0 · legions 00666fa0

### ReferenceTable
- [~] `ReferenceTable::AllocatePackage` · 006dde60 · legions 00667010 — ac2-script vm: no GC/reference reuse, no VMMon timers, efun ids resolved per call

### ProcessState
- [ ] `ProcessState::ProcessState` · 006ddfe0 · legions 00667190

## `portal/shared/src/interp/processtable.cpp.c` (0/8)

### ProcessTable
- [ ] `ProcessTable::~ProcessTable` · 006e17f0
- [ ] `ProcessTable::UnRegisterProcess` · 006e1830 · legions 00668660
- [ ] `ProcessTable::GetProcess` · 006e1860 · legions 00668640
- [ ] `ProcessTable::QueryProcess` · 006e1880 · legions 00668690
- [ ] `ProcessTable::ProcessTable` · 006e18b0 · legions 006686c0
- [ ] `ProcessTable::RegisterProcess` · 006e18d0 · legions 006686e0
- [ ] `ProcessTable::QueryProcessTable` · 006e1920 · legions 00668730
- [ ] `ProcessTable::QueryProcesses` · 006e1960 · legions 00668770

## `portal/shared/src/interp/referenceentry.cpp.c` (2/4)

### ReferenceEntry
- [ ] `ReferenceEntry::GetPackSize` · 006e0ac0 · legions 00667990
- [ ] `ReferenceEntry::GetResidentSizeEstimate` · 006e0b10
- [x] `ReferenceEntry::Pack` · 006e0b40 · legions 006679e0 — ac2-net protocol::pack
- [x] `ReferenceEntry::UnPack` · 006e0c20 · legions 00667ac0 — ac2-script reference/native unpack

## `portal/shared/src/interp/referencetable.cpp.c` (0/3)

### ReferenceTable
- [ ] `ReferenceTable::QueryReferences` · 006e1a20
- [ ] `ReferenceTable::QueryReferenceTree_Recursive` · 006e1cd0
- [ ] `ReferenceTable::QueryReferenceTree` · 006e2040

## `portal/shared/src/interp/reservedreftable.cpp.c` (0/4)

### ReservedRefTable
- [ ] `ReservedRefTable::StartMarking` · 006e09e0 · legions 006678b0

### Box2D
- [ ] `Box2D::Box2D` · 006e09f0 · legions 006678c0

### ReservedRefTable
- [ ] `ReservedRefTable::StopMarking` · 006e0a00 · legions 006678d0
- [ ] `ReservedRefTable::AddReference` · 006e0a50 · legions 00667920

## `portal/shared/src/interp/runtimebytestream.cpp.c` (0/3)

### RuntimeByteStream
- [~] `RuntimeByteStream::ValidateVersion` · 006ecb60 — ac2-script ByteStream::validate_versions: wcc check; InterpVersionCheck env path not ported

### ReferenceTable
- [~] `ReferenceTable::CreateStringLiteral` · 006ecd30 · legions 00671b70 — ac2-script vm: no GC/reference reuse, no VMMon timers, efun ids resolved per call

### RuntimeByteStream
- [~] `RuntimeByteStream::LoadStringLitTableSection` · 006ecdb0 — ac2-script vm: no GC/reference reuse, no VMMon timers, efun ids resolved per call

## `portal/shared/src/interp/typeinfo.cpp.c` (0/3)

### (free functions)
- [ ] `new_validated_type` · 006e1490
- [ ] `get_type` · 006e14c0 · legions 00668300
- [ ] `new_type` · 006e15a0 · legions 006683e0

## `portal/shared/src/interp/vm.cpp.c` (12/66)

### VM
- [ ] `VM::CreateProcessA` · 006dbe00 · legions 006652b0
- [ ] `VM::CreateProcessA` · 006dbe30 · legions 006652e0
- [ ] `VM::CopyNativeObject` · 006dbe60 · legions 00665310
- [ ] `VM::IsDerivedFrom` · 006dbe80 · legions 00665330
- [ ] `VM::IsDerivedFrom` · 006dbea0 · legions 00665350
- [ ] `VM::ValidateVersion` · 006dbec0 · legions 00665370
- [ ] `VM::GetReferences` · 006dbef0 · legions 006653a0
- [ ] `VM::UpdateNativeReferences` · 006dbf10 · legions 006653c0
- [ ] `VM::DeleteNativeObject` · 006dbf30 · legions 006653e0
- [ ] `VM::GetNativePackSize` · 006dbf50 · legions 00665400
- [ ] `VM::PackNativeObject` · 006dbf70 · legions 00665420
- [ ] `VM::UnPackNativeObject` · 006dbf90 · legions 00665440
- [ ] `VM::GetContextID` · 006dbfb0 · legions 00665460
- [ ] `VM::SetActiveGC` · 006dbfc0 · legions 00665470
- [ ] `VM::SetAggressiveGCDeletion` · 006dbfd0 · legions 00665480
- [ ] `VM::WSL_EFun_GetInstanceReference` · 006dbfe0 · legions 00665490

### ReferenceTable
- [ ] `ReferenceTable::GetReferencePackSize` · 006dbff0 · legions 006654a0
- [~] `ReferenceTable::PackReferenceArray` · 006dc040 · legions 006654f0 — ac2-net protocol::pack: packed reference graph both ways; no VM reference table (no WState lookup, no library slot check, no dependent-reference walk)
- [x] `ReferenceTable::GetDependentReferences_Relative` · 006dc0b0 · legions 00665560 — ac2-script vm::pack (dependent-reference walk, CloneReference)

### VM
- [ ] `VM::DeleteProcess` · 006dc0e0 · legions 00665590
- [~] `VM::HashNativeObject` · 006dc110 · legions 006655c0 — ac2-script glue::hashes NativeKeys: StringInfo and wpstring; CaseInsensitiveWPString (0x353F) not yet
- [~] `VM::IsNativeEqual` · 006dc140 · legions 006655f0 — ac2-script glue::hashes NativeKeys: StringInfo and wpstring; CaseInsensitiveWPString (0x353F) not yet
- [x] `VM::GetStringLiteral` · 006dc170 · legions 00665620 — ac2-script vm (VM/ReferenceTable/Resolver)
- [x] `VM::GetHeapObject` · 006dc190 · legions 00665640 — ac2-script vm (VM/ReferenceTable/Resolver)
- [ ] `VM::IsReferenceValid` · 006dc1a0

### InterpreterBreakpointHelper::StepBreakpointData
- [-] `InterpreterBreakpointHelper::StepBreakpointData::~StepBreakpointData` · 006dc210 · legions 00665690 — script debugger

### ReferenceTable
- [ ] `ReferenceTable::DeleteAllReferences` · 006dc270 · legions 006656f0

### VM
- [ ] `VM::GetStackTrace` · 006dc2d0 · legions 00665750
- [ ] `VM::QueryProcessTableW` · 006dc390 · legions 00665810
- [ ] `VM::QueryProcessesW` · 006dc450 · legions 006658d0
- [ ] `VM::QueryProcessW` · 006dc520 · legions 006659a0
- [ ] `VM::QueryReferencesW` · 006dc5f0 · legions 00665a70
- [ ] `VM::QueryReferenceTreeW` · 006dc6b0 · legions 00665b30
- [ ] `VM::QueryReferenceTreeForProcessW` · 006dc780 · legions 00665c00
- [ ] `VM::DeleteAllUnUsedReferences` · 006dc850 · legions 00665cd0

### ReferenceTable
- [x] `ReferenceTable::GetDependentReferences_Native` · 006dc860 · legions 00665ce0 — ac2-script vm::pack (dependent-reference walk, CloneReference)
- [x] `ReferenceTable::GetDependentReferences_Package` · 006dc910 · legions 00665d90 — ac2-script vm::pack (dependent-reference walk, CloneReference)

### VM
- [ ] `VM::QueryPerf` · 006dc9e0
- [ ] `VM::QueryPerfW` · 006dca70

### ReferenceTable
- [ ] `ReferenceTable::~ReferenceTable` · 006dcb80 · legions 00665f80
- [x] `ReferenceTable::GetDependentReferences_HeapObject` · 006dcbe0 · legions 00665fe0 — ac2-script vm::pack (dependent-reference walk, CloneReference)

### VM
- [ ] `VM::GetPackageNameByPkgID` · 006dcc00 · legions 00666000
- [ ] `VM::GetPackageNameByPkgIDW` · 006dcc50 · legions 00666050
- [ ] `VM::GarbageCollect` · 006dcc70 · legions 00666070
- [ ] `VM::~VM` · 006dcc80 · legions 00666080
- [~] `VM::GetPackageID` · 006dcce0 · legions 006660e0 — ac2-script wstate DID formulas / ByteStream::package_id
- [ ] `VM::GetPackageID` · 006dcd20 · legions 00666120
- [ ] `VM::GetSingletonPackage` · 006dce20 · legions 00666220
- [ ] `VM::GetSingletonPackageW` · 006dce90 · legions 00666290
- [~] `VM::GetSingletonPackageByID` · 006dcf50 · legions 00666350 — ac2-script wstate DID formulas / ByteStream::package_id
- [x] `VM::GetWStateReference` · 006dcf90 · legions 00666390 — ac2-script vm (VM/ReferenceTable/Resolver)
- [x] `VM::GetPackageReference` · 006dcfb0 · legions 006663b0 — ac2-script glue

### InterpreterBreakpointHelper
- [-] `InterpreterBreakpointHelper::InterpreterBreakpointHelper` · 006dd040 · legions 00666440 — script debugger

### ReferenceTable
- [ ] `ReferenceTable::ReferenceTable` · 006dd0a0 · legions 006664a0
- [x] `ReferenceTable::GetDependentReferences` · 006dd160 · legions 00666560 — ac2-script vm::pack (dependent-reference walk, CloneReference)

### VM
- [ ] `VM::VM` · 006dd220 · legions 00666620
- [ ] `VM::CreateSingletonProcess` · 006dd2d0 · legions 006666d0

### ReferenceTable
- [ ] `ReferenceTable::GetReferencePackSize` · 006dd310 · legions 00666710
- [~] `ReferenceTable::PackReference` · 006dd3f0 · legions 006667f0 — ac2-script vm::pack: builds the PackedReference structure; the byte stream (native StreamPack) is the wire codec's

### VM
- [ ] `VM::GetReferencePackSize` · 006dd4e0 · legions 006668e0
- [~] `VM::PackReference` · 006dd500 · legions 00666900 — ac2-script vm::pack: builds the PackedReference structure; the byte stream (native StreamPack) is the wire codec's
- [x] `VM::UnPackReference` · 006dd520 · legions 00666920 — ac2-script vm (VM/ReferenceTable/Resolver)
- [ ] `VM::Preproc_CleanWState` · 006dd540 · legions 00666940
- [ ] `VM::GetPackageName` · 006dd630 · legions 00666a20
- [x] `VM::IsInstanceOf` · 006dd690 · legions 00666a80 — ac2-script glue
- [x] `VM::CloneReference` · 006dd6c0 · legions 00666ab0 — ac2-script vm::pack (dependent-reference walk, CloneReference)

## `portal/shared/src/interp/vmlogger.cpp.c` (0/2)

### VMLogger
- [ ] `VMLogger::OpenLog` · 004bd0f0
- [ ] `VMLogger::Log` · 004bd2d0

## `portal/shared/src/interp/wccefuns.cpp.c` (4/4)

### (free functions)
- [x] `ToIID` · 006e0d80 · legions 00667bf0 — ac2-script glue
- [x] `ToSHex` · 006e13b0 · legions 00668220 — ac2-script glue
- [x] `ToLHex` · 006e1410 · legions 00668280 — ac2-script glue
- [x] `ToHex` · 006e1470 · legions 006682e0 — ac2-script glue

## `portal/shared/src/interp/wlink/wlibdb.cpp.c` (0/4)

### WLinkMap
- [ ] `WLinkMap::GetPackageName` · 006186a0 · legions 005d5080

### WLibDB
- [ ] `WLibDB::I_GetPackageName` · 00618700

### WLinkMap
- [ ] `WLinkMap::GetPackageID` · 00618780

### WLibDB
- [ ] `WLibDB::I_GetPackageID` · 006187f0
