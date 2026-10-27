# ac2-script: WSL interpreter and native glue

2468 retail functions: 1431 done, 110 partial, 602 to do, 325 skipped.
Legend and workflow: [README](README.md).

## `ac2/shared/src/interp/glue/gcharactergensystem.cpp.c` (14/25)

### Position
- [-] `Position::operator=` · 0046ad40 · legions 0046b420 — C++ assignment operator of plain data (Rust copy/move)

### PHashTable<unsigned_long,unsigned_long>
- [-] `PHashTable<unsigned_long,unsigned_long>::~PHashTable<unsigned_long,unsigned_long>` · 0046adc0 · legions 0046b440 — C++ destructor (Rust drop replaces it)

### gCharacterGenSystem
- [~] `gCharacterGenSystem::ARHashToRaceSexInfoHash` · 0046ae10 · legions 0046b570 — ac2-script host::chargen: container conversions done inline (AList/ARHash reads)
- [~] `gCharacterGenSystem::RaceSexInfoHashToARHash` · 0046af20 · legions 0046b680 — ac2-script host::chargen: container conversions done inline (AList/ARHash reads)
- [~] `gCharacterGenSystem::RaceSexListHashToARHash` · 0046b090 · legions 0046b7f0 — ac2-script host::chargen: container conversions done inline (AList/ARHash reads)

### ARHash
- [-] `ARHash::ARHash` · 0046b250 · legions 0046b9b0 — C++ constructor of plain data or a container (Rust struct init)
- [-] `ARHash::GetReferences` · 0046b290 — container internals (Rust collections replace them)

### BaseHash<unsigned_long,unsigned_long,18>
- [-] `BaseHash<unsigned_long,unsigned_long,18>::AddDataAsReferences` · 0046b2b0 — container internals (Rust collections replace them)

### ARHash
- [-] `ARHash::UpdateReferences` · 0046b310 — container internals (Rust collections replace them)

### BaseHash<unsigned_long,unsigned_long,18>
- [-] `BaseHash<unsigned_long,unsigned_long,18>::UpdateData` · 0046b330 — container internals (Rust collections replace them)

### gCharacterGenSystem
- [~] `gCharacterGenSystem::ARHashToRaceSexListHash` · 0046b3f0 · legions 0046bb50 — ac2-script host::chargen: container conversions done inline (AList/ARHash reads)
- [x] `gCharacterGenSystem::CallIFun_FormatNameString` · 0046b480 · legions 0046bbe0 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `gCharacterGenSystem::CallIFun_GetPhysObj` · 0046b6d0 · legions 0046be30 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `gCharacterGenSystem::CallIFun_InqAdminAvatarEntityDID` · 0046b800 · legions 0046bf60 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `gCharacterGenSystem::CallIFun_InqAppliedAppearanceVDesc` · 0046b960 · legions 0046c0c0 — ac2-script host/chargen.rs applied_appearance over the CharacterGenSystem script
- [x] `gCharacterGenSystem::CallIFun_InqPhysTypeLabel` · 0046bb90 · legions 0046c2f0 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `gCharacterGenSystem::CallIFun_InqPlayerAvatarEntityDID` · 0046bdb0 · legions 0046c510 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `gCharacterGenSystem::CallIFun_InqRaceSexData` · 0046bf10 · legions 0046c670 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `gCharacterGenSystem::CallIFun_InqRaceSexInfoData` · 0046c100 · legions 0046c860 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `gCharacterGenSystem::CallIFun_InqStartingPosition` · 0046c2f0 · legions 0046ca50 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `gCharacterGenSystem::CallIFun_InqValidPhysTypeValues` · 0046c500 · legions 0046cc60 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `gCharacterGenSystem::CallIFun_InqValidStartingAreas` · 0046c750 · legions 0046ceb0 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `gCharacterGenSystem::CallIFun_IsValid` · 0046c980 · legions 0046d0e0 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `gCharacterGenSystem::CallIFun_RandomizeCGR` · 0046cac0 · legions 0046d220 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `gCharacterGenSystem::CallIFun_SetCGRDefaults` · 0046cca0 · legions 0046d400 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)

## `ac2/shared/src/interp/glue/gcraftskilladaptor.cpp.c` (16/16)

### gCraftSkillAdaptor
- [x] `gCraftSkillAdaptor::CallEFun_SetCostToRaise` · 0046e320 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gCraftSkillAdaptor::CallEFun_SetDID` · 0046e380 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gCraftSkillAdaptor::CallEFun_SetDescription` · 0046e3e0 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gCraftSkillAdaptor::CallEFun_SetIconDID` · 0046e440 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gCraftSkillAdaptor::CallEFun_SetLevel` · 0046e4a0 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gCraftSkillAdaptor::CallEFun_SetName` · 0046e500 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gCraftSkillAdaptor::CallEFun_SetRawLevel` · 0046e560 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gCraftSkillAdaptor::CallEFun_SetUsageXPMod` · 0046e5c0 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gCraftSkillAdaptor::CallEFun_GetCostToRaise` · 0046e620 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gCraftSkillAdaptor::CallEFun_GetDID` · 0046e690 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gCraftSkillAdaptor::CallEFun_GetIconDID` · 0046e700 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gCraftSkillAdaptor::CallEFun_GetLevel` · 0046e770 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gCraftSkillAdaptor::CallEFun_GetRawLevel` · 0046e7e0 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gCraftSkillAdaptor::CallEFun_GetUsageXPMod` · 0046e850 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gCraftSkillAdaptor::CallEFun_GetDescription` · 0046e8c0 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gCraftSkillAdaptor::CallEFun_GetName` · 0046e970 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)

## `ac2/shared/src/interp/glue/gdamagetextblob.cpp.c` (9/9)

### gDamageTextBlob
- [x] `gDamageTextBlob::CallEFun_SetDuration` · 00478bb0 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gDamageTextBlob::CallEFun_SetFontColor` · 00478c10 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gDamageTextBlob::CallEFun_SetScale` · 00478c80 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)

### gmCharGenResult
- [x] `gmCharGenResult::SetName` · 00478ce0 · legions 005c2260 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)

### gDamageTextBlob
- [x] `gDamageTextBlob::CallEFun_GetDuration` · 00478d30 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gDamageTextBlob::CallEFun_GetScale` · 00478da0 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gDamageTextBlob::CallEFun_SetDisplayString` · 00478e10 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gDamageTextBlob::CallEFun_GetDisplayString` · 00478e70 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gDamageTextBlob::CallEFun_GetFontColor` · 00478f30 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)

## `ac2/shared/src/interp/glue/geffectuinode.cpp.c` (11/11)

### gEffectUINode
- [x] `gEffectUINode::CallEFun_Set` · 004787a0 — ac2-script glue/effect_node.rs: efuns 0x98 SetEID/SetName/SetIcon/SetDescription/SetEnabled/SetDisabled/Set/SetDurationInfo/SetToggled/SetPulsed over ac2-net EffectUiNode
- [x] `gEffectUINode::CallEFun_SetDisabled` · 00478800 — ac2-script glue/effect_node.rs: efuns 0x98 SetEID/SetName/SetIcon/SetDescription/SetEnabled/SetDisabled/Set/SetDurationInfo/SetToggled/SetPulsed over ac2-net EffectUiNode
- [x] `gEffectUINode::CallEFun_SetDurationInfo` · 00478860 — ac2-script glue/effect_node.rs: efuns 0x98 SetEID/SetName/SetIcon/SetDescription/SetEnabled/SetDisabled/Set/SetDurationInfo/SetToggled/SetPulsed over ac2-net EffectUiNode
- [x] `gEffectUINode::CallEFun_SetEID` · 004788c0 — ac2-script glue/effect_node.rs: efuns 0x98 SetEID/SetName/SetIcon/SetDescription/SetEnabled/SetDisabled/Set/SetDurationInfo/SetToggled/SetPulsed over ac2-net EffectUiNode
- [x] `gEffectUINode::CallEFun_SetEnabled` · 00478920 — ac2-script glue/effect_node.rs: efuns 0x98 SetEID/SetName/SetIcon/SetDescription/SetEnabled/SetDisabled/Set/SetDurationInfo/SetToggled/SetPulsed over ac2-net EffectUiNode
- [x] `gEffectUINode::CallEFun_SetIcon` · 00478980 — ac2-script glue/effect_node.rs: efuns 0x98 SetEID/SetName/SetIcon/SetDescription/SetEnabled/SetDisabled/Set/SetDurationInfo/SetToggled/SetPulsed over ac2-net EffectUiNode
- [x] `gEffectUINode::CallEFun_SetPulsed` · 004789e0 — ac2-script glue/effect_node.rs: efuns 0x98 SetEID/SetName/SetIcon/SetDescription/SetEnabled/SetDisabled/Set/SetDurationInfo/SetToggled/SetPulsed over ac2-net EffectUiNode
- [x] `gEffectUINode::CallEFun_SetToggled` · 00478a40 — ac2-script glue/effect_node.rs: efuns 0x98 SetEID/SetName/SetIcon/SetDescription/SetEnabled/SetDisabled/Set/SetDurationInfo/SetToggled/SetPulsed over ac2-net EffectUiNode

### EffectUINode
- [x] `EffectUINode::SetName` · 00478aa0 · legions 00474b30 — reproduced by the gEffectUINode glue (ac2-script glue/effect_node.rs); body from admin 7148 and Legions layout

### gEffectUINode
- [x] `gEffectUINode::CallEFun_SetDescription` · 00478af0 — ac2-script glue/effect_node.rs: efuns 0x98 SetEID/SetName/SetIcon/SetDescription/SetEnabled/SetDisabled/Set/SetDurationInfo/SetToggled/SetPulsed over ac2-net EffectUiNode
- [x] `gEffectUINode::CallEFun_SetName` · 00478b50 — ac2-script glue/effect_node.rs: efuns 0x98 SetEID/SetName/SetIcon/SetDescription/SetEnabled/SetDisabled/Set/SetDurationInfo/SetToggled/SetPulsed over ac2-net EffectUiNode

## `ac2/shared/src/interp/glue/gexaminationprofile.cpp.c` (17/17)

### gExaminationProfile
- [x] `gExaminationProfile::CallEFun_AddBool` · 00478050 — ac2-net ExaminationProfile Add* (raw valDataID words, sorted insert); ac2-script glue examination.rs binds gExaminationProfile 0x990001-0x990019
- [x] `gExaminationProfile::CallEFun_AddBreak` · 004780b0 — ac2-net ExaminationProfile Add* (raw valDataID words, sorted insert); ac2-script glue examination.rs binds gExaminationProfile 0x990001-0x990019
- [x] `gExaminationProfile::CallEFun_AddCountdown` · 00478110 — ac2-net ExaminationProfile Add* (raw valDataID words, sorted insert); ac2-script glue examination.rs binds gExaminationProfile 0x990001-0x990019
- [x] `gExaminationProfile::CallEFun_AddEffectID` · 00478180 — ac2-net ExaminationProfile Add* (raw valDataID words, sorted insert); ac2-script glue examination.rs binds gExaminationProfile 0x990001-0x990019
- [x] `gExaminationProfile::CallEFun_AddFloat` · 004781e0 — ac2-net ExaminationProfile Add* (raw valDataID words, sorted insert); ac2-script glue examination.rs binds gExaminationProfile 0x990001-0x990019
- [x] `gExaminationProfile::CallEFun_AddIconDesc` · 00478250 — ac2-net ExaminationProfile Add* (raw valDataID words, sorted insert); ac2-script glue examination.rs binds gExaminationProfile 0x990001-0x990019
- [x] `gExaminationProfile::CallEFun_AddImage` · 004782c0 — ac2-net ExaminationProfile Add* (raw valDataID words, sorted insert); ac2-script glue examination.rs binds gExaminationProfile 0x990001-0x990019
- [x] `gExaminationProfile::CallEFun_AddInt` · 00478320 — ac2-net ExaminationProfile Add* (raw valDataID words, sorted insert); ac2-script glue examination.rs binds gExaminationProfile 0x990001-0x990019
- [x] `gExaminationProfile::CallEFun_AddLongInt` · 00478380 — ac2-net ExaminationProfile Add* (raw valDataID words, sorted insert); ac2-script glue examination.rs binds gExaminationProfile 0x990001-0x990019
- [x] `gExaminationProfile::CallEFun_AddSimpleStringInfo` · 004783f0 — ac2-net ExaminationProfile Add* (raw valDataID words, sorted insert); ac2-script glue examination.rs binds gExaminationProfile 0x990001-0x990019
- [x] `gExaminationProfile::CallEFun_AddStringInfo` · 00478460 — ac2-net ExaminationProfile Add* (raw valDataID words, sorted insert); ac2-script glue examination.rs binds gExaminationProfile 0x990001-0x990019
- [x] `gExaminationProfile::CallEFun_AddTab` · 004784d0 — ac2-net ExaminationProfile Add* (raw valDataID words, sorted insert); ac2-script glue examination.rs binds gExaminationProfile 0x990001-0x990019
- [x] `gExaminationProfile::CallEFun_AddTabs` · 00478530 — ac2-net ExaminationProfile Add* (raw valDataID words, sorted insert); ac2-script glue examination.rs binds gExaminationProfile 0x990001-0x990019
- [x] `gExaminationProfile::CallEFun_SetExaminationRequest` · 00478590 — ac2-net ExaminationProfile Add* (raw valDataID words, sorted insert); ac2-script glue examination.rs binds gExaminationProfile 0x990001-0x990019
- [x] `gExaminationProfile::CallEFun_eq` · 004785f0 — ac2-net ExaminationProfile Add* (raw valDataID words, sorted insert); ac2-script glue examination.rs binds gExaminationProfile 0x990001-0x990019
- [x] `gExaminationProfile::CallEFun_ne` · 00478670 — ac2-net ExaminationProfile Add* (raw valDataID words, sorted insert); ac2-script glue examination.rs binds gExaminationProfile 0x990001-0x990019
- [x] `gExaminationProfile::CallEFun_GetExaminationRequest` · 004786f0 — ac2-net ExaminationProfile Add* (raw valDataID words, sorted insert); ac2-script glue examination.rs binds gExaminationProfile 0x990001-0x990019

## `ac2/shared/src/interp/glue/gexaminationrequest.cpp.c` (12/12)

### gExaminationRequest
- [x] `gExaminationRequest::CallEFun_SetAdmin` · 00477b40 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gExaminationRequest::CallEFun_SetDataEnum` · 00477ba0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gExaminationRequest::CallEFun_SetDataID` · 00477c00 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gExaminationRequest::CallEFun_SetDataIID` · 00477c60 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gExaminationRequest::CallEFun_SetType` · 00477cc0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gExaminationRequest::CallEFun_GetDataEnum` · 00477d20 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gExaminationRequest::CallEFun_GetDataID` · 00477d90 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gExaminationRequest::CallEFun_GetDataIID` · 00477e00 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gExaminationRequest::CallEFun_GetType` · 00477e70 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gExaminationRequest::CallEFun_IsAdmin` · 00477ee0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gExaminationRequest::CallEFun_eq` · 00477f50 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gExaminationRequest::CallEFun_ne` · 00477fd0 — ac2-script glue::options / examination (Legions layout and efun switch)

## `ac2/shared/src/interp/glue/ggameplayoptionsprofile.cpp.c` (80/83)

### GameplayOptionsProfile
- [ ] `GameplayOptionsProfile::SetShowDamageTextOther` · 00474770 · legions 00471870 — needs port: options profile show-damage-text-other setter

### ReferenceIDWrapper
- [x] `ReferenceIDWrapper::StreamPack` · 00474780 · legions 00471880 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

### gGameplayOptionsProfile
- [x] `gGameplayOptionsProfile::CallEFun_AddRadarMask` · 004747f0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_FlushAliasTable` · 00474850 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_RemoveRadarMask` · 004748b0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_SetAcceptAllegianceRequest` · 00474910 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_SetAcceptFellowshipRequest` · 00474970 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_SetAcceptTradeRequest` · 004749d0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_SetAutoTarget` · 00474a30 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_SetBitfield` · 00474a90 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_SetBypassGoldMeltingConfirmationDlg` · 00474af0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_SetDamageTextRangeOther` · 00474b50 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_SetLatencyIndicatorGraph` · 00474bb0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_SetRadarMask` · 00474c10 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_SetShortcutHeight` · 00474c70 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_SetShortcutInstaScroll` · 00474cd0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_SetShortcutLocked` · 00474d30 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_SetShowDamageText` · 00474d90 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_SetShowDamageTextOther` · 00474df0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_SetShowFellowsOnMap` · 00474e50 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_SetShowObjectNames` · 00474ec0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_SetShowPKDamage` · 00474f20 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_SetShowPetUI` · 00474f80 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_SetShowRadarCoords` · 00474ff0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_SetShowShortcutNumbers` · 00475050 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_SetUISaveLocations` · 004750b0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_SetWhichShortcutSet` · 00475110 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_AddFilter` · 00475220 — ac2-script glue::options / examination (Legions layout and efun switch)
- [ ] `gGameplayOptionsProfile::CallEFun_Export` · 004752a0 — needs port: gameplay options profile Export/Import
- [x] `gGameplayOptionsProfile::CallEFun_GetAcceptAllegianceRequest` · 00475320 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_GetAcceptFellowshipRequest` · 00475390 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_GetAcceptTradeRequest` · 00475410 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_GetAutoTarget` · 00475490 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_GetBitfield` · 00475500 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_GetBypassGoldMeltingConfirmationDlg` · 00475570 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_GetChatFontColor` · 004755f0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_GetChatFontSize` · 00475670 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_GetChatPopupFlag` · 004756f0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_GetDamageTextRangeOther` · 00475770 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_GetFilter` · 004757e0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_GetLatencyIndicatorGraph` · 00475860 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_GetRadarMask` · 004758e0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_GetShortcutHeight` · 00475950 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_GetShortcutInstaScroll` · 004759c0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_GetShortcutLocked` · 00475a40 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_GetShowDamageText` · 00475ac0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_GetShowDamageTextOther` · 00475b30 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_GetShowFellowsOnMap` · 00475ba0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_GetShowObjectNames` · 00475c20 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_GetShowPKDamage` · 00475ca0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_GetShowPetUI` · 00475d20 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_GetShowRadarCoords` · 00475da0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_GetShowShortcutNumbers` · 00475e10 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_GetWhichShortcutSet` · 00475e80 — ac2-script glue::options / examination (Legions layout and efun switch)
- [ ] `gGameplayOptionsProfile::CallEFun_Import` · 00475ef0 — needs port: gameplay options profile Export/Import
- [x] `gGameplayOptionsProfile::CallEFun_InitShortcutArray` · 00475f70 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_RemoveAlias` · 00475ff0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_RemoveFilter` · 00476070 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_SetAlias` · 004760f0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_SetChatFontColor` · 00476180 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_SetChatFontSize` · 00476210 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_SetChatPopupFlag` · 004762a0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_SetFilter` · 00476330 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_SetShortcut` · 004763b0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_SetWindowOpacity` · 00476430 — ac2-script glue::options / examination (Legions layout and efun switch)

### ggmCWInterface
- [x] `ggmCWInterface::RArrayToShortcutInfoArray` · 004766c0 · legions 00472db0 — ac2-script glue::options / examination (Legions layout and efun switch)

### gGameplayOptionsProfile
- [x] `gGameplayOptionsProfile::CallEFun_SetShortcutArray` · 00476920 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::AAHashToChatPopupFlags` · 00476a00 · legions 004730d0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::NRHashToStrStrHash` · 00476af0 · legions 004731c0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::ChatPopupFlagsToAAHash` · 00476b80 · legions 00473250 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_BuildAliasList` · 00476c70 · legions 00473340 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::StrStrHashToNRHash` · 00476d30 · legions 004733d0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_SetAliasTable` · 00476e40 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_SetWindowOpacities` · 00476f20 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_SetChatOptions` · 00477000 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_GetChatOptions` · 00477280 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_GetUISaveLocations` · 004775b0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_GetWindowOpacities` · 00477630 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_InqAlias` · 00477730 · legions 00473ea0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_InqAliasTable` · 004777d0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_InqShortcut` · 004778f0 · legions 00474010 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_InqShortcutArray` · 00477990 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gGameplayOptionsProfile::CallEFun_InqUISaveLocations` · 00477aa0 — ac2-script glue::options / examination (Legions layout and efun switch)

## `ac2/shared/src/interp/glue/ggmactinfo.cpp.c` (3/4)

### ggmActInfo
- [x] `ggmActInfo::CallEFun_Init` · 00474530 — ac2-script glue/act_info.rs (ggmActInfo, ggmActInfoList efuns); ac2-net native/act.rs (gmActInfo StreamPack); client ui_quests.rs inq_act_info_list (InqActInfoList IFun 0xB4008F)
- [x] `ggmActInfo::CallEFun_AddScene` · 004745c0 — ac2-script glue/act_info.rs (ggmActInfo, ggmActInfoList efuns); ac2-net native/act.rs (gmActInfo StreamPack); client ui_quests.rs inq_act_info_list (InqActInfoList IFun 0xB4008F)
- [x] `ggmActInfo::CallEFun_GetActID` · 00474640 — ac2-script glue/act_info.rs (ggmActInfo, ggmActInfoList efuns); ac2-net native/act.rs (gmActInfo StreamPack); client ui_quests.rs inq_act_info_list (InqActInfoList IFun 0xB4008F)
- [-] `ggmActInfo::CallEFun_DumpW` · 004746b0 — debug dump (DumpW, DumpScenesW); not bound, as the other dump efuns

## `ac2/shared/src/interp/glue/ggmactinfolist.cpp.c` (2/2)

### ggmActInfoList
- [x] `ggmActInfoList::CallEFun_pop_front` · 004743f0 — ac2-script glue/act_info.rs (ggmActInfo, ggmActInfoList efuns); ac2-net native/act.rs (gmActInfo StreamPack); client ui_quests.rs inq_act_info_list (InqActInfoList IFun 0xB4008F)
- [x] `ggmActInfoList::CallEFun_insert_sort` · 004744b0 — ac2-script glue/act_info.rs (ggmActInfo, ggmActInfoList efuns); ac2-net native/act.rs (gmActInfo StreamPack); client ui_quests.rs inq_act_info_list (InqActInfoList IFun 0xB4008F)

## `ac2/shared/src/interp/glue/ggmchargenresult.cpp.c` (21/21)

### ggmCharGenResult
- [x] `ggmCharGenResult::CallEFun_Clear` · 00473990 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `ggmCharGenResult::CallEFun_SetAdmin` · 004739f0 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `ggmCharGenResult::CallEFun_SetName` · 00473a50 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `ggmCharGenResult::CallEFun_SetRace` · 00473ab0 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `ggmCharGenResult::CallEFun_SetSex` · 00473b10 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `ggmCharGenResult::CallEFun_SetStartArea` · 00473b70 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `ggmCharGenResult::CallEFun_GetClassType` · 00473bd0 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `ggmCharGenResult::CallEFun_GetImplement` · 00473c40 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `ggmCharGenResult::CallEFun_GetPhysTypeValue` · 00473cb0 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `ggmCharGenResult::CallEFun_GetRace` · 00473d30 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `ggmCharGenResult::CallEFun_GetSex` · 00473da0 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `ggmCharGenResult::CallEFun_GetStartArea` · 00473e10 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `ggmCharGenResult::CallEFun_IsAdmin` · 00473e80 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `ggmCharGenResult::CallEFun_SanityCheck` · 00473ef0 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `ggmCharGenResult::CallEFun_SetClassType` · 00473f60 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `ggmCharGenResult::CallEFun_SetImplement` · 00473fe0 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `ggmCharGenResult::CallEFun_SetPhysTypeValue` · 00474060 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `ggmCharGenResult::CallEFun_SetStartingPosition` · 004740e0 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `ggmCharGenResult::CallEFun_VerifyCharGenResult` · 00474180 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `ggmCharGenResult::CallEFun_GetName` · 004741f0 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `ggmCharGenResult::CallEFun_GetStartingPosition` · 004742c0 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)

## `ac2/shared/src/interp/glue/ggmkeyframe.cpp.c` (15/21)

### ggmKeyframe
- [x] `ggmKeyframe::CallEFun_Clear` · 00473030 — ac2-script glue/story.rs (ggmKeyframe, ggmSceneInfo, ggmSceneInfoList efuns)
- [x] `ggmKeyframe::CallEFun_SetAudioTrack` · 00473090 — ac2-script glue/story.rs (ggmKeyframe, ggmSceneInfo, ggmSceneInfoList efuns)
- [ ] `ggmKeyframe::CallEFun_SetAudioTrackByName` · 004730f0 — needs port: keyframe efuns (audio/video/voice/movie track setters and GetMovieTrack)
- [x] `ggmKeyframe::CallEFun_SetDuration` · 00473150 — ac2-script glue/story.rs (ggmKeyframe, ggmSceneInfo, ggmSceneInfoList efuns)
- [x] `ggmKeyframe::CallEFun_SetFrameNum` · 004731b0 — ac2-script glue/story.rs (ggmKeyframe, ggmSceneInfo, ggmSceneInfoList efuns)
- [ ] `ggmKeyframe::CallEFun_SetMovieTrack` · 00473210 — needs port: keyframe efuns (audio/video/voice/movie track setters and GetMovieTrack)
- [x] `ggmKeyframe::CallEFun_SetSceneID` · 00473270 — ac2-script glue/story.rs (ggmKeyframe, ggmSceneInfo, ggmSceneInfoList efuns)
- [x] `ggmKeyframe::CallEFun_SetSubtitle` · 004732d0 — ac2-script glue/story.rs (ggmKeyframe, ggmSceneInfo, ggmSceneInfoList efuns)
- [x] `ggmKeyframe::CallEFun_SetVideoTrack` · 00473330 — ac2-script glue/story.rs (ggmKeyframe, ggmSceneInfo, ggmSceneInfoList efuns)
- [ ] `ggmKeyframe::CallEFun_SetVideoTrackByName` · 00473390 — needs port: keyframe efuns (audio/video/voice/movie track setters and GetMovieTrack)
- [x] `ggmKeyframe::CallEFun_SetVoiceTrack` · 004733f0 — ac2-script glue/story.rs (ggmKeyframe, ggmSceneInfo, ggmSceneInfoList efuns)
- [ ] `ggmKeyframe::CallEFun_SetVoiceTrackByName` · 00473450 — needs port: keyframe efuns (audio/video/voice/movie track setters and GetMovieTrack)
- [x] `ggmKeyframe::CallEFun_GetAudioTrack` · 004734b0 — ac2-script glue/story.rs (ggmKeyframe, ggmSceneInfo, ggmSceneInfoList efuns)
- [x] `ggmKeyframe::CallEFun_GetDuration` · 00473520 — ac2-script glue/story.rs (ggmKeyframe, ggmSceneInfo, ggmSceneInfoList efuns)
- [x] `ggmKeyframe::CallEFun_GetFrameNum` · 00473590 — ac2-script glue/story.rs (ggmKeyframe, ggmSceneInfo, ggmSceneInfoList efuns)
- [x] `ggmKeyframe::CallEFun_GetSceneID` · 00473600 — ac2-script glue/story.rs (ggmKeyframe, ggmSceneInfo, ggmSceneInfoList efuns)
- [x] `ggmKeyframe::CallEFun_GetVideoTrack` · 00473670 — ac2-script glue/story.rs (ggmKeyframe, ggmSceneInfo, ggmSceneInfoList efuns)
- [x] `ggmKeyframe::CallEFun_GetVoiceTrack` · 004736e0 — ac2-script glue/story.rs (ggmKeyframe, ggmSceneInfo, ggmSceneInfoList efuns)
- [-] `ggmKeyframe::CallEFun_Dump` · 00473750 — WSL debug dump
- [ ] `ggmKeyframe::CallEFun_GetMovieTrack` · 00473810 — needs port: keyframe efuns (audio/video/voice/movie track setters and GetMovieTrack)
- [x] `ggmKeyframe::CallEFun_GetSubtitle` · 004738e0 — ac2-script glue/story.rs (ggmKeyframe, ggmSceneInfo, ggmSceneInfoList efuns)

## `ac2/shared/src/interp/glue/ggmquestinfo.cpp.c` (7/8)

### ggmQuestInfo
- [x] `ggmQuestInfo::CallEFun_Init` · 00472b50 — ac2-script glue::quests / ac2-net native::quest
- [x] `ggmQuestInfo::CallEFun_GetQuestID` · 00472c10 — ac2-script glue::quests / ac2-net native::quest
- [x] `ggmQuestInfo::CallEFun_GetStatus` · 00472c80 — ac2-script glue::quests / ac2-net native::quest
- [x] `ggmQuestInfo::CallEFun_PlaysFXOnUpdate` · 00472cf0 — ac2-script glue::quests / ac2-net native::quest
- [-] `ggmQuestInfo::CallEFun_DumpW` · 00472d60 — ac2-script: debug text dump, not bound
- [x] `ggmQuestInfo::CallEFun_GetCurrentJournalEntry` · 00472e20 — ac2-script glue::quests / ac2-net native::quest
- [x] `ggmQuestInfo::CallEFun_GetQuestDescription` · 00472ed0 — ac2-script glue::quests / ac2-net native::quest
- [x] `ggmQuestInfo::CallEFun_GetQuestName` · 00472f80 — ac2-script glue::quests / ac2-net native::quest

## `ac2/shared/src/interp/glue/ggmquestinfolist.cpp.c` (6/7)

### ggmQuestInfoList
- [x] `ggmQuestInfoList::CallEFun_Erase` · 004726d0 — ac2-script glue::quests / ac2-net native::quest
- [x] `ggmQuestInfoList::CallEFun_num_elements` · 00472750 — ac2-script glue::quests / ac2-net native::quest
- [x] `ggmQuestInfoList::CallEFun_push_tail` · 00472820 — ac2-script glue::quests / ac2-net native::quest
- [-] `ggmQuestInfoList::CallEFun_DumpW` · 004728a0 — ac2-script: debug text dump, not bound
- [x] `ggmQuestInfoList::CallEFun_Find` · 00472960 · legions 00470640 — ac2-script glue::quests / ac2-net native::quest
- [x] `ggmQuestInfoList::CallEFun_UpdateQuestInfo` · 00472a00 · legions 004706c0 — ac2-script glue::quests / ac2-net native::quest
- [x] `ggmQuestInfoList::CallEFun_pop_head` · 00472ab0 — ac2-script glue::quests / ac2-net native::quest

## `ac2/shared/src/interp/glue/ggmracesexinfo.cpp.c` (9/9)

### ggmRaceSexInfo
- [x] `ggmRaceSexInfo::CallEFun_SetMusic` · 00472260 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `ggmRaceSexInfo::CallEFun_SetPhysObj` · 004722c0 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `ggmRaceSexInfo::CallEFun_SetPileAppearance` · 00472320 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `ggmRaceSexInfo::CallEFun_GetMusic` · 00472380 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `ggmRaceSexInfo::CallEFun_GetPhysObj` · 004723f0 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `ggmRaceSexInfo::CallEFun_GetPileAppearance` · 00472460 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `ggmRaceSexInfo::CallEFun_PutObjInLoc` · 004724d0 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `ggmRaceSexInfo::CallEFun_Verify` · 00472560 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `ggmRaceSexInfo::CallEFun_GetSubDataIDs` · 004725d0 · legions 004703b0 — ac2-script glue::places / enums / chargen (PlacesTable data::places)

## `ac2/shared/src/interp/glue/ggmsceneinfo.cpp.c` (17/20)

### ggmSceneInfo
- [x] `ggmSceneInfo::CallEFun_Clear` · 00471950 — ac2-script glue/story.rs (ggmKeyframe, ggmSceneInfo, ggmSceneInfoList efuns)
- [x] `ggmSceneInfo::CallEFun_ClearForHackers` · 004719b0 — ac2-script glue/story.rs (ggmKeyframe, ggmSceneInfo, ggmSceneInfoList efuns)
- [x] `ggmSceneInfo::CallEFun_Set` · 00471a10 — ac2-script glue/story.rs (ggmKeyframe, ggmSceneInfo, ggmSceneInfoList efuns)
- [x] `ggmSceneInfo::CallEFun_SetActID` · 00471a70 — ac2-script glue/story.rs (ggmKeyframe, ggmSceneInfo, ggmSceneInfoList efuns)
- [x] `ggmSceneInfo::CallEFun_SetHidden` · 00471ad0 — ac2-script glue/story.rs (ggmKeyframe, ggmSceneInfo, ggmSceneInfoList efuns)
- [x] `ggmSceneInfo::CallEFun_SetID` · 00471b30 — ac2-script glue/story.rs (ggmKeyframe, ggmSceneInfo, ggmSceneInfoList efuns)
- [x] `ggmSceneInfo::CallEFun_SetIDByName` · 00471b90 — ac2-script glue/story.rs (ggmKeyframe, ggmSceneInfo, ggmSceneInfoList efuns)
- [x] `ggmSceneInfo::CallEFun_SetName` · 00471bf0 — ac2-script glue/story.rs (ggmKeyframe, ggmSceneInfo, ggmSceneInfoList efuns)
- [x] `ggmSceneInfo::CallEFun_SetPlayable` · 00471c50 — ac2-script glue/story.rs (ggmKeyframe, ggmSceneInfo, ggmSceneInfoList efuns)
- [x] `ggmSceneInfo::CallEFun_SetSceneNum` · 00471cb0 — ac2-script glue/story.rs (ggmKeyframe, ggmSceneInfo, ggmSceneInfoList efuns)
- [x] `ggmSceneInfo::CallEFun_AddKeyframe` · 00471d10 — ac2-script glue/story.rs (ggmKeyframe, ggmSceneInfo, ggmSceneInfoList efuns)
- [x] `ggmSceneInfo::CallEFun_GetActID` · 00471da0 — ac2-script glue/story.rs (ggmKeyframe, ggmSceneInfo, ggmSceneInfoList efuns)
- [x] `ggmSceneInfo::CallEFun_GetID` · 00471e10 — ac2-script glue/story.rs (ggmKeyframe, ggmSceneInfo, ggmSceneInfoList efuns)
- [x] `ggmSceneInfo::CallEFun_GetSceneNum` · 00471e80 — ac2-script glue/story.rs (ggmKeyframe, ggmSceneInfo, ggmSceneInfoList efuns)
- [x] `ggmSceneInfo::CallEFun_IsHidden` · 00471ef0 — ac2-script glue/story.rs (ggmKeyframe, ggmSceneInfo, ggmSceneInfoList efuns)
- [x] `ggmSceneInfo::CallEFun_IsPlayable` · 00471f60 — ac2-script glue/story.rs (ggmKeyframe, ggmSceneInfo, ggmSceneInfoList efuns)
- [-] `ggmSceneInfo::CallEFun_Dump` · 00471fd0 — WSL debug dump
- [x] `ggmSceneInfo::CallEFun_GetName` · 00472090 — ac2-script glue/story.rs (ggmKeyframe, ggmSceneInfo, ggmSceneInfoList efuns)
- [ ] `ggmSceneInfo::CallEFun_GetSubDataIDs` · 00472140 — needs port: scene info sub-DID list (GetSubDataIDs) and Verify
- [ ] `ggmSceneInfo::CallEFun_Verify` · 004721c0 · legions 00470280 — needs port: scene info sub-DID list (GetSubDataIDs) and Verify

## `ac2/shared/src/interp/glue/ggmsceneinfolist.cpp.c` (4/4)

### ggmSceneInfoList
- [x] `ggmSceneInfoList::CallEFun_num_elements` · 00471720 — ac2-script glue/story.rs (ggmKeyframe, ggmSceneInfo, ggmSceneInfoList efuns)
- [x] `ggmSceneInfoList::CallEFun_pop_front` · 00471790 — ac2-script glue/story.rs (ggmKeyframe, ggmSceneInfo, ggmSceneInfoList efuns)
- [x] `ggmSceneInfoList::CallEFun_push_tail` · 00471810 — ac2-script glue/story.rs (ggmKeyframe, ggmSceneInfo, ggmSceneInfoList efuns)
- [x] `ggmSceneInfoList::CallEFun_insert_sort` · 004718d0 — ac2-script glue/story.rs (ggmKeyframe, ggmSceneInfo, ggmSceneInfoList efuns)

## `ac2/shared/src/interp/glue/gingredientadaptor.cpp.c` (1/8)

### gIngredientAdaptor
- [ ] `gIngredientAdaptor::CallEFun_SetAutoPopulate` · 00471310 — needs port: crafting ingredient adaptor (GetID, GetDescription, SetAutoPopulate)
- [~] `gIngredientAdaptor::CallEFun_SetDescription` · 00471370 — net: bound or decoded, not yet exercised by a test (queries answer empty lists for the double's player; UpdateCost/UpdateCraftPanel/UpdateCraftSkill forwarders not sent)
- [x] `gIngredientAdaptor::CallEFun_SetID` · 004713d0 — net: craft_script_7160 (the VM runs the IFun or the forwarder; the packet or notice is asserted)
- [~] `gIngredientAdaptor::CallEFun_SetRecipeDID` · 00471430 — net: bound or decoded, not yet exercised by a test (queries answer empty lists for the double's player; UpdateCost/UpdateCraftPanel/UpdateCraftSkill forwarders not sent)
- [~] `gIngredientAdaptor::CallEFun_GetAutoPopulate` · 00471490 — net: bound or decoded, not yet exercised by a test (queries answer empty lists for the double's player; UpdateCost/UpdateCraftPanel/UpdateCraftSkill forwarders not sent)
- [ ] `gIngredientAdaptor::CallEFun_GetID` · 00471500 — needs port: crafting ingredient adaptor (GetID, GetDescription, SetAutoPopulate)
- [~] `gIngredientAdaptor::CallEFun_GetRecipeDID` · 00471570 — net: bound or decoded, not yet exercised by a test (queries answer empty lists for the double's player; UpdateCost/UpdateCraftPanel/UpdateCraftSkill forwarders not sent)
- [ ] `gIngredientAdaptor::CallEFun_GetDescription` · 004715e0 — needs port: crafting ingredient adaptor (GetID, GetDescription, SetAutoPopulate)

## `ac2/shared/src/interp/glue/ginventoryglobals.cpp.c` (1/1)

### gInventoryGlobals
- [x] `gInventoryGlobals::CallIFun_GetCoinMaxStackSize` · 0046d050 · legions 0046d7b0 — ac2-ui money_bag.rs / item_display.rs / backpack.rs; ac2-client ui_inventory.rs; ac2-script host/inventory_globals.rs (coin max) and the player's IFUN 0xB400D7 (bypass); Legions 7160 matches admin here (same IFUNs 0xA60001 and 0xB400D7)

## `ac2/shared/src/interp/glue/gradarcontrol.cpp.c` (1/1)

### gRadarControl
- [x] `gRadarControl::CallIFun_InqRadarRanges` · 0046ce60 · legions 0046d5c0 — ac2-audio/ac2-client: ambient environments, groove

## `ac2/shared/src/interp/glue/grecipeadaptor.cpp.c` (11/34)

### gRecipeAdaptor
- [~] `gRecipeAdaptor::CallEFun_SetCharges` · 004704f0 — net: bound in glue/craft_natives.rs; the query fills them, values not individually checked
- [x] `gRecipeAdaptor::CallEFun_SetCost` · 00470550 — net: client call over the script VM (craft_script_7160); DAT-backed answers and the request bytes tested
- [~] `gRecipeAdaptor::CallEFun_SetCraftSkillCategory` · 004705b0 — net: bound in glue/craft_natives.rs; the query fills them, values not individually checked
- [x] `gRecipeAdaptor::CallEFun_SetCraftSkillDID` · 00470610 — net: client call over the script VM (craft_script_7160); DAT-backed answers and the request bytes tested
- [x] `gRecipeAdaptor::CallEFun_SetDID` · 00470670 — net: client call over the script VM (craft_script_7160); DAT-backed answers and the request bytes tested
- [~] `gRecipeAdaptor::CallEFun_SetDescription` · 004706d0 — net: bound in glue/craft_natives.rs; the query fills them, values not individually checked
- [~] `gRecipeAdaptor::CallEFun_SetDifficulty` · 00470730 — net: bound in glue/craft_natives.rs; the query fills them, values not individually checked
- [~] `gRecipeAdaptor::CallEFun_SetDifficultyMinusSkillCutoff` · 00470790 — net: bound in glue/craft_natives.rs; the query fills them, values not individually checked
- [~] `gRecipeAdaptor::CallEFun_SetDynamicCost` · 004707f0 — net: bound in glue/craft_natives.rs; the query fills them, values not individually checked
- [~] `gRecipeAdaptor::CallEFun_SetDynamicDifficulty` · 00470850 — net: bound in glue/craft_natives.rs; the query fills them, values not individually checked
- [~] `gRecipeAdaptor::CallEFun_SetHasAutoPopulatingIngredients` · 004708b0 — net: bound in glue/craft_natives.rs; the query fills them, values not individually checked
- [~] `gRecipeAdaptor::CallEFun_SetHasNameColoringTable` · 00470910 — net: bound in glue/craft_natives.rs; the query fills them, values not individually checked
- [~] `gRecipeAdaptor::CallEFun_SetIconDID` · 00470970 — net: bound in glue/craft_natives.rs; the query fills them, values not individually checked
- [~] `gRecipeAdaptor::CallEFun_SetLastSpinnerValue` · 004709d0 — net: bound in glue/craft_natives.rs; the query fills them, values not individually checked
- [~] `gRecipeAdaptor::CallEFun_SetMaxSpinnerValue` · 00470a30 — net: bound in glue/craft_natives.rs; the query fills them, values not individually checked
- [~] `gRecipeAdaptor::CallEFun_SetMaximumNumberOfIngredients` · 00470a90 — net: bound in glue/craft_natives.rs; the query fills them, values not individually checked
- [~] `gRecipeAdaptor::CallEFun_SetMinSpinnerValue` · 00470af0 — net: bound in glue/craft_natives.rs; the query fills them, values not individually checked
- [~] `gRecipeAdaptor::CallEFun_SetName` · 00470b50 — net: bound in glue/craft_natives.rs; the query fills them, values not individually checked
- [~] `gRecipeAdaptor::CallEFun_SetNameColor` · 00470bb0 — net: bound in glue/craft_natives.rs; the query fills them, values not individually checked
- [~] `gRecipeAdaptor::CallEFun_SetReadOnlySpinner` · 00470c10 — net: bound in glue/craft_natives.rs; the query fills them, values not individually checked
- [~] `gRecipeAdaptor::CallEFun_SetShown` · 00470c70 — net: bound in glue/craft_natives.rs; the query fills them, values not individually checked
- [~] `gRecipeAdaptor::CallEFun_SetTimeUseAgain` · 00470cd0 — net: bound in glue/craft_natives.rs; the query fills them, values not individually checked
- [x] `gRecipeAdaptor::CallEFun_GetCharges` · 00470d30 — ac2-script glue::craft_natives (unit test recipe_getters_return_the_fields_the_setters_write; the value is the setter's field)
- [ ] `gRecipeAdaptor::CallEFun_GetCost` · 00470da0 — needs port: crafting recipe adaptor (GetCost, HasReadOnlySpinner)
- [x] `gRecipeAdaptor::CallEFun_GetCraftSkillCategory` · 00470e10 — ac2-script glue::craft_natives (unit test recipe_getters_return_the_fields_the_setters_write; the value is the setter's field)
- [x] `gRecipeAdaptor::CallEFun_GetCraftSkillDID` · 00470e80 — ac2-script glue::craft_natives (unit test recipe_getters_return_the_fields_the_setters_write; the value is the setter's field)
- [x] `gRecipeAdaptor::CallEFun_GetDID` · 00470ef0 — net: client call over the script VM (craft_script_7160); DAT-backed answers and the request bytes tested
- [x] `gRecipeAdaptor::CallEFun_GetDynamicCost` · 00470f60 — ac2-script glue::craft_natives (unit test recipe_getters_return_the_fields_the_setters_write; the value is the setter's field)
- [x] `gRecipeAdaptor::CallEFun_GetIconDID` · 00470fd0 — ac2-script glue::craft_natives (unit test recipe_getters_return_the_fields_the_setters_write; the value is the setter's field)
- [x] `gRecipeAdaptor::CallEFun_GetShown` · 00471040 — ac2-script glue::craft_natives (unit test recipe_getters_return_the_fields_the_setters_write; the value is the setter's field)
- [x] `gRecipeAdaptor::CallEFun_GetTimeUseAgain` · 004710b0 — ac2-script glue::craft_natives (unit test recipe_getters_return_the_fields_the_setters_write; the value is the setter's field)
- [ ] `gRecipeAdaptor::CallEFun_HasReadOnlySpinner` · 00471130 — needs port: crafting recipe adaptor (GetCost, HasReadOnlySpinner)
- [~] `gRecipeAdaptor::CallEFun_GetDescription` · 004711b0 — net: bound or read generically (ifun.rs read_filled / text_out); no test reads it back
- [~] `gRecipeAdaptor::CallEFun_GetName` · 00471260 — net: bound or read generically (ifun.rs read_filled / text_out); no test reads it back

## `ac2/shared/src/interp/glue/gselectioninfo.cpp.c` (15/16)

### gSelectionInfo
- [x] `gSelectionInfo::CallEFun_SetCurHealth` · 0046fe40 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gSelectionInfo::CallEFun_SetCurVigor` · 0046fea0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [ ] `gSelectionInfo::CallEFun_SetMaxHealth` · 0046ff00 — needs port (check): implemented as glue/examination.rs but uncited
- [x] `gSelectionInfo::CallEFun_SetMaxVigor` · 0046ff60 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gSelectionInfo::CallEFun_SetPKDamage` · 0046ffc0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gSelectionInfo::CallEFun_SetPKVigorloss` · 00470020 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gSelectionInfo::CallEFun_SetType` · 00470080 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gSelectionInfo::CallEFun_GetCurHealth` · 004700e0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gSelectionInfo::CallEFun_GetCurVigor` · 00470150 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gSelectionInfo::CallEFun_GetMaxHealth` · 004701c0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gSelectionInfo::CallEFun_GetMaxVigor` · 00470230 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gSelectionInfo::CallEFun_GetPKDamage` · 004702a0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gSelectionInfo::CallEFun_GetPKVigorloss` · 00470310 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gSelectionInfo::CallEFun_GetType` · 00470380 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gSelectionInfo::CallEFun_eq` · 004703f0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `gSelectionInfo::CallEFun_ne` · 00470470 — ac2-script glue::options / examination (Legions layout and efun switch)

## `ac2/shared/src/interp/glue/gshortcutinfo.cpp.c` (10/10)

### gShortcutInfo
- [x] `gShortcutInfo::CallEFun_SetDataEnum` · 0046f9b0 — ac2-script glue::shortcut / native::shortcut
- [x] `gShortcutInfo::CallEFun_SetDataID` · 0046fa10 — ac2-script glue::shortcut / native::shortcut
- [x] `gShortcutInfo::CallEFun_SetDataIID` · 0046fa70 — ac2-script glue::shortcut / native::shortcut
- [x] `gShortcutInfo::CallEFun_SetType` · 0046fad0 — ac2-script glue::shortcut / native::shortcut
- [x] `gShortcutInfo::CallEFun_GetDataEnum` · 0046fb30 — ac2-script glue::shortcut / native::shortcut
- [x] `gShortcutInfo::CallEFun_GetDataID` · 0046fba0 — ac2-script glue::shortcut / native::shortcut
- [x] `gShortcutInfo::CallEFun_GetDataIID` · 0046fc10 — ac2-script glue::shortcut / native::shortcut
- [x] `gShortcutInfo::CallEFun_GetType` · 0046fc80 — ac2-script glue::shortcut / native::shortcut
- [x] `gShortcutInfo::CallEFun_SetDataStr` · 0046fcf0 — ac2-script glue::shortcut / native::shortcut
- [x] `gShortcutInfo::CallEFun_GetDataStr` · 0046fd70 — ac2-script glue::shortcut / native::shortcut

## `ac2/shared/src/interp/glue/gskillpanelmanager.cpp.c` (4/5)

### gSkillPanelManager
- [ ] `gSkillPanelManager::SkillUINodeHashToARHash` · 0046d140 · legions 0046d8a0 — needs port: skill panel node hash conversion
- [x] `gSkillPanelManager::ARHashToSkillUINodeHash` · 0046d270 · legions 0046d9d0 — ac2-script host::skill_panel (ScriptHost::skill_panel)
- [x] `gSkillPanelManager::CallIFun_InqIcon` · 0046d380 · legions 0046dae0 — ac2-script host::skill_panel (ScriptHost::skill_panel)
- [x] `gSkillPanelManager::CallIFun_InqName` · 0046d510 · legions 0046dc70 — ac2-script host::skill_panel (ScriptHost::skill_panel)
- [x] `gSkillPanelManager::CallIFun_InqNodeHash` · 0046d6f0 · legions 0046de50 — ac2-script host::skill_panel (ScriptHost::skill_panel)

## `ac2/shared/src/interp/glue/gskilluinode.cpp.c` (24/24)

### gSkillUINode
- [x] `gSkillUINode::CallEFun_AddParent` · 0046ed90 — ac2-script glue::skill_node over ac2-net's SkillUiNode
- [x] `gSkillUINode::CallEFun_Clear` · 0046edf0 — ac2-script glue::skill_node over ac2-net's SkillUiNode
- [x] `gSkillUINode::CallEFun_Set` · 0046ee50 — ac2-script glue::skill_node over ac2-net's SkillUiNode
- [x] `gSkillUINode::CallEFun_SetCoordinates` · 0046eeb0 — ac2-script glue::skill_node over ac2-net's SkillUiNode
- [x] `gSkillUINode::CallEFun_SetDescription` · 0046ef20 — ac2-script glue::skill_node over ac2-net's SkillUiNode
- [x] `gSkillUINode::CallEFun_SetEnum` · 0046ef80 — ac2-script glue::skill_node over ac2-net's SkillUiNode
- [x] `gSkillUINode::CallEFun_SetIcon` · 0046efe0 — ac2-script glue::skill_node over ac2-net's SkillUiNode
- [x] `gSkillUINode::CallEFun_SetName` · 0046f040 — ac2-script glue::skill_node over ac2-net's SkillUiNode
- [x] `gSkillUINode::CallEFun_SetPassive` · 0046f0a0 — ac2-script glue::skill_node over ac2-net's SkillUiNode
- [x] `gSkillUINode::CallEFun_SetType` · 0046f100 — ac2-script glue::skill_node over ac2-net's SkillUiNode
- [x] `gSkillUINode::CallEFun_SetUntrainable` · 0046f160 — ac2-script glue::skill_node over ac2-net's SkillUiNode
- [x] `gSkillUINode::CallEFun_AddDependency` · 0046f1c0 — ac2-script glue::skill_node over ac2-net's SkillUiNode
- [x] `gSkillUINode::CallEFun_GetEnum` · 0046f230 — ac2-script glue::skill_node over ac2-net's SkillUiNode
- [x] `gSkillUINode::CallEFun_GetIcon` · 0046f2a0 — ac2-script glue::skill_node over ac2-net's SkillUiNode
- [x] `gSkillUINode::CallEFun_GetType` · 0046f310 — ac2-script glue::skill_node over ac2-net's SkillUiNode
- [x] `gSkillUINode::CallEFun_GetX` · 0046f380 — ac2-script glue::skill_node over ac2-net's SkillUiNode
- [x] `gSkillUINode::CallEFun_GetY` · 0046f3f0 — ac2-script glue::skill_node over ac2-net's SkillUiNode
- [x] `gSkillUINode::CallEFun_IsPassive` · 0046f460 — ac2-script glue::skill_node over ac2-net's SkillUiNode
- [x] `gSkillUINode::CallEFun_IsUntrainable` · 0046f4d0 — ac2-script glue::skill_node over ac2-net's SkillUiNode
- [x] `gSkillUINode::CallEFun_GetDependencyList` · 0046f540 — ac2-script glue::skill_node over ac2-net's SkillUiNode
- [x] `gSkillUINode::CallEFun_GetDescription` · 0046f660 — ac2-script glue::skill_node over ac2-net's SkillUiNode
- [x] `gSkillUINode::CallEFun_GetName` · 0046f710 — ac2-script glue::skill_node over ac2-net's SkillUiNode
- [x] `gSkillUINode::CallEFun_GetParentList` · 0046f7c0 — ac2-script glue::skill_node over ac2-net's SkillUiNode
- [x] `gSkillUINode::CallEFun_InqCoordinates` · 0046f8e0 — ac2-script glue::skill_node over ac2-net's SkillUiNode

## `ac2/shared/src/interp/glue/gturbinegunutils.cpp.c` (1/2)

### gTurbineGunUtils
- [x] `gTurbineGunUtils::CallEFun_GetWorldName` · 0046ea20 — ac2-script glue/turbine_gun.rs: returns the client's world name (Vm::world_name, from the WORLD_NAME notice); test in glue/turbine_gun.rs
- [ ] `gTurbineGunUtils::CallEFun_GetWorldSuffix` · 0046eaa0 — needs port: gTurbineGunUtils::GetWorldSuffix (world name suffix; GetWorldName is in glue/turbine_gun.rs)

## `ac2/shared/src/interp/glue/gturbineuserstatus.cpp.c` (3/4)

### ReferenceTable
- [ ] `ReferenceTable::GetReferenceTagPtr` · 0046eb20 · legions 0046e9c0 — needs port: reference table tag lookup (GetReferenceTagPtr)

### gTurbineUserStatus
- [x] `gTurbineUserStatus::CallEFun_InqLocation` · 0046ebd0 · legions 0046ea70 — net: friends_script_7160 / ac2-ui friends model (the status event runs the script; the native's efuns read name, location, mode; the GUN code converts through the admin table, 23 rows, default 0xc741)
- [x] `gTurbineUserStatus::CallEFun_InqMode` · 0046ec60 · legions 0046eae0 — net: friends_script_7160 / ac2-ui friends model (the status event runs the script; the native's efuns read name, location, mode; the GUN code converts through the admin table, 23 rows, default 0xc741)
- [x] `gTurbineUserStatus::CallEFun_InqName` · 0046ed00 — net: friends_script_7160 / ac2-ui friends model (the status event runs the script; the native's efuns read name, location, mode; the GUN code converts through the admin table, 23 rows, default 0xc741)

## `ac2/shared/src/interp/glue/guishop.cpp.c` (8/8)

### gUIShop
- [x] `gUIShop::CallEFun_SetDesc` · 0046de60 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gUIShop::CallEFun_SetID` · 0046dec0 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gUIShop::CallEFun_SetName` · 0046df20 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gUIShop::CallEFun_SetPortrait` · 0046df80 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gUIShop::CallEFun_SetPosition` · 0046dfe0 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gUIShop::CallEFun_SetSorter` · 0046e040 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gUIShop::CallEFun_SetStoreSize` · 0046e0a0 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gUIShop::CallEFun_AddSale` · 0046e280 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)

## `ac2/shared/src/interp/glue/guishopsale.cpp.c` (12/13)

### gUIShopSale
- [x] `gUIShopSale::CallEFun_SetCostDesc` · 0046d900 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gUIShopSale::CallEFun_SetCostIcon` · 0046d980 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gUIShopSale::CallEFun_SetCostQuantity` · 0046d9e0 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gUIShopSale::CallEFun_SetExpired` · 0046da40 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gUIShopSale::CallEFun_SetItemDID` · 0046daa0 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gUIShopSale::CallEFun_SetItemDesc` · 0046db00 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gUIShopSale::CallEFun_SetItemIID` · 0046db90 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [ ] `gUIShopSale::CallEFun_SetItemIconDesc` · 0046dbf0 — needs port: shop sale icon description (SetItemIconDesc)
- [x] `gUIShopSale::CallEFun_SetItemIconID` · 0046dc80 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gUIShopSale::CallEFun_SetOrdinal` · 0046dce0 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gUIShopSale::CallEFun_SetRestricted` · 0046dd40 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gUIShopSale::CallEFun_SetSold` · 0046dda0 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gUIShopSale::CallEFun_SetStockQuantity` · 0046de00 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)

## `portal/shared/src/interp/breakpointinfo.cpp.c` (0/1)

### InterpreterBreakpointHelper
- [-] `InterpreterBreakpointHelper::SetTemporaryBreakpoint` · 0073c990 — debug info (debugger only)

## `portal/shared/src/interp/bytestream.cpp.c` (16/87)

### ByteStream
- [-] `ByteStream::Pack` · 006e4cd0 · legions 0066b440 — compiled-wlib writer (client only reads wlib)
- [-] `ByteStream::PackSection` · 006e4d40 · legions 0066b4b0 — compiled-wlib writer (client only reads wlib)
- [x] `ByteStream::LoadOpcodeSection` · 006e4dc0 — ac2-script WLib/WState/ByteStream decode
- [x] `ByteStream::LoadImportTableSection` · 006e4e50 — ac2-script WLib/WState/ByteStream decode
- [x] `ByteStream::LoadStringLitTableSection` · 006e4ee0 — ac2-script WLib/WState/ByteStream decode
- [x] `ByteStream::LoadExportTableSection` · 006e4f70 — ac2-script WLib/WState/ByteStream decode
- [x] `ByteStream::LoadVTableSection` · 006e5000 — ac2-script WLib/WState/ByteStream decode
- [x] `ByteStream::LoadValidEventTableSection` · 006e5090 — ac2-script WLib/WState/ByteStream decode

### CObjCell
- [ ] `CObjCell::UpdateCellMaterial` · 006e5130 · legions 0066b600 — needs port: cell material update (CObjCell::UpdateCellMaterial)

### OpcodeStream
- [~] `OpcodeStream::StreamPack` · 006e5140 — ac2-script: unpack direction only

### ByteStream
- [-] `ByteStream::DiscardAllDebuggingInformation` · 006e5220 · legions 0066b6f0 — wlib debug-info section (debugger only)
- [-] `ByteStream::PackSectionList` · 006e5310 · legions 0066b7e0 — compiled-wlib writer (client only reads wlib)
- [-] `ByteStream::InqExportPackage` · 006e5450 — admin wlib tooling (no Legions match)
- [-] `ByteStream::LoadLineNumDebugSection` · 006e54e0 — text source loader, admin build only
- [-] `ByteStream::LoadFrameDebugInfoSection` · 006e5690 — text source loader, admin build only
- [-] `ByteStream::ValidationFailure` · 006e5760 — admin wlib tooling (no Legions match)
- [x] `ByteStream::IsDerivedFrom` · 006e5790 · legions 0066ba70 — ac2-script glue

### VersionTable
- [-] `VersionTable::GetVersion` · 006e5820 — admin wlib version query (no Legions match)
- [~] `VersionTable::StreamPack` · 006e58e0 — ac2-script: unpack direction only

### ByteStream
- [x] `ByteStream::IsDerivedFrom` · 006e5960 · legions 0066bc20 — ac2-script glue
- [x] `ByteStream::LoadVersionInfoSection` · 006e5aa0 · legions 0066bd30 — ac2-script WLib/WState/ByteStream decode
- [x] `ByteStream::IsDerivedFrom` · 006e5c40 · legions 0066be60 — ac2-script glue

### PSmartArray<PackageInfo>
- [ ] `PSmartArray<PackageInfo>::StreamPack` · 006e5ce0 · legions 0066bea0 — needs port: wlib loader table (package, vtable or nested array) in bytestream

### PackageInfo
- [-] `PackageInfo::PackageInfo` · 006e5e70 — C++ constructor of plain data or a container (Rust struct init)

### AnimAssetWeightKey
- [ ] `AnimAssetWeightKey::StreamPack` · 006e5e80 · legions 006b22d0 — needs port: animation asset weight key load (StreamPack)

### StringLitTable
- [~] `StringLitTable::StreamPack` · 006e5ec0 · legions 0066c040 — ac2-script: unpack direction only

### ByteStream
- [x] `ByteStream::IsDerivedFrom` · 006e5f00 — ac2-script glue

### DualHash<PStringBase<char>,CheckpointExportData>
- [-] `DualHash<PStringBase<char>,CheckpointExportData>::find_right` · 006e5fa0 · legions 0066c0c0 — container internals (Rust collections replace them)

### PSmartArray<VersionTable::ver>
- [ ] `PSmartArray<VersionTable::ver>::StreamPack` · 006e6030 · legions 0066c150 — needs port: wlib loader table (package, vtable or nested array) in bytestream

### VersionTable::ver
- [-] `VersionTable::ver::ver` · 006e61f0 · legions 0066c310 — C++ constructor of plain data or a container (Rust struct init)

### StringLitTable
- [-] `StringLitTable::~StringLitTable` · 006e6230 · legions 0066c350 — C++ destructor (Rust drop replaces it)

### ByteStream
- [-] `ByteStream::LoadFunctionLocDebugSection` · 006e62d0 · legions 0066c3f0 — wlib debug-info section (debugger only)

### DualHash<PStringBase<char>,CheckpointExportData>
- [-] `DualHash<PStringBase<char>,CheckpointExportData>::contains_right` · 006e6410 · legions 0066c4c0 — container internals (Rust collections replace them)

### PSmartArray<PSmartArray<unsigned_long>_>
- [ ] `PSmartArray<PSmartArray<unsigned_long>_>::StreamPack` · 006e6480 · legions 0066c530 — needs port: wlib loader table (package, vtable or nested array) in bytestream

### PSmartArray<PSmartArray<VTableID>_>
- [ ] `PSmartArray<PSmartArray<VTableID>_>::StreamPack` · 006e6620 · legions 0066c6d0 — needs port: wlib loader table (package, vtable or nested array) in bytestream

### SmartArray<VTableID>
- [-] `SmartArray<VTableID>::operator=` · 006e6820 · legions 0066c890 — C++ assignment operator of plain data (Rust copy/move)

### VTableID
- [-] `VTableID::`default_constructor_closure'` · 006e6930 — MSVC compiler stub (default_constructor_closure)

### PSmartArray<VTableID>
- [ ] `PSmartArray<VTableID>::StreamPack` · 006e6940 — needs port: wlib loader table (package, vtable or nested array) in bytestream

### (free functions)
- [ ] `STREAMPACK` · 006e69e0 · legions 0066d840 — needs port: STREAMPACK loader helper (wlib arrays)

### PSmartArray<POriginalSourceFileInfo>
- [-] `PSmartArray<POriginalSourceFileInfo>::StreamPack` · 006e6a60 · legions 0066ca50 — debug-info section table (debugger only)

### POriginalSourceFileInfo
- [-] `POriginalSourceFileInfo::POriginalSourceFileInfo` · 006e6c20 · legions 0066cc10 — C++ constructor of plain data or a container (Rust struct init)

### ByteStream
- [x] `ByteStream::ReadByteStream` · 006e6c60 — ac2-script WLib/WState/ByteStream decode
- [x] `ByteStream::UnPack` · 006e6f40 — ac2-script WLib/WState/ByteStream decode
- [-] `ByteStream::IsCheckpointedMember` · 006e6fb0 · legions 0066ce00 — checkpoint member test (debugger/checkpoint)
- [-] `ByteStream::LoadSourceFileDebugSection` · 006e7090 · legions 0066ce80 — wlib debug-info section (debugger only)
- [-] `ByteStream::Open` · 006e7380 — admin wlib tooling (no Legions match)
- [x] `ByteStream::LoadSection` · 006e74f0 — ac2-script WLib/WState/ByteStream decode

### PSmartArray<FrameDebugInfo>
- [-] `PSmartArray<FrameDebugInfo>::StreamPack` · 006e7750 · legions 0066d2e0 — debug-info section table (debugger only)

### FrameDebugInfo
- [-] `FrameDebugInfo::FrameDebugInfo` · 006e78d0 · legions 0066d460 — C++ constructor of plain data or a container (Rust struct init)
- [-] `FrameDebugInfo::operator=` · 006e7910 · legions 0066d4a0 — C++ assignment operator of plain data (Rust copy/move)

### POriginalSourceFileInfo
- [-] `POriginalSourceFileInfo::StreamPack` · 006e7990 · legions 0066d520 — debug info (debugger only)

### ByteStream
- [-] `ByteStream::Clear` · 006e79d0 · legions 0066d560 — wlib reset helper (Rust Vec owns the stream)

### PHashTable<FunctionID,int>
- [~] `PHashTable<FunctionID,int>::StreamPack` · 006e7ad0 — ac2-script: unpack direction only

### PSmartArray<ImportData>
- [x] `PSmartArray<ImportData>::StreamPack` · 006e7cb0 · legions 0066d8c0 — bytestream/tables.rs fn imports (PSmartArray<ImportData>::StreamPack)

### ImportData
- [-] `ImportData::ImportData` · 006e7e30 · legions 0066da40 — C++ constructor of plain data or a container (Rust struct init)

### PSmartArray<ImportSymbolInfo>
- [ ] `PSmartArray<ImportSymbolInfo>::StreamPack` · 006e7e80 · legions 0066da90 — needs port: wlib loader table (package, vtable or nested array) in bytestream

### ImportSymbolInfo
- [-] `ImportSymbolInfo::ImportSymbolInfo` · 006e8000 · legions 0066dc10 — C++ constructor of plain data or a container (Rust struct init)
- [-] `ImportSymbolInfo::operator=` · 006e8040 · legions 0066dc50 — C++ assignment operator of plain data (Rust copy/move)

### ImportData
- [-] `ImportData::operator=` · 006e80b0 · legions 0066dcc0 — C++ assignment operator of plain data (Rust copy/move)

### SmartArray<ImportSymbolInfo>
- [-] `SmartArray<ImportSymbolInfo>::operator=` · 006e8140 · legions 0066dd50 — C++ assignment operator of plain data (Rust copy/move)

### ImportSymbolInfo
- [~] `ImportSymbolInfo::StreamPack` · 006e81f0 · legions 0066de00 — ac2-script: unpack direction only

### PSmartArray<PLineOffsetList>
- [-] `PSmartArray<PLineOffsetList>::StreamPack` · 006e8240 · legions 0066de50 — debug-info section table (debugger only)

### PLineOffsetList
- [-] `PLineOffsetList::PLineOffsetList` · 006e8400 — C++ constructor of plain data or a container (Rust struct init)

### SmartArray<PLineOffsetInfo>
- [-] `SmartArray<PLineOffsetInfo>::operator=` · 006e8430 · legions 0066e040 — C++ assignment operator of plain data (Rust copy/move)

### PLineOffsetInfo
- [-] `PLineOffsetInfo::PLineOffsetInfo` · 006e8590 — C++ constructor of plain data or a container (Rust struct init)

### PLineOffsetList
- [-] `PLineOffsetList::StreamPack` · 006e85a0 · legions 0066e1b0 — debug info (debugger only)

### PSmartArray<PLineOffsetInfo>
- [-] `PSmartArray<PLineOffsetInfo>::StreamPack` · 006e85e0 · legions 0066e1f0 — debug-info section table (debugger only)

### PHashTable<PStringBase<char>,unsigned_long>
- [ ] `PHashTable<PStringBase<char>,unsigned_long>::StreamPack` · 006e8690 · legions 0066e2a0 — needs port: wlib loader table (package, vtable or nested array) in bytestream

### VTableSection
- [-] `VTableSection::VTableSection` · 006e8950 · legions 0066e560 — C++ constructor of plain data or a container (Rust struct init)
- [~] `VTableSection::StreamPack` · 006e89d0 · legions 0066e5e0 — ac2-script: unpack direction only
- [-] `VTableSection::~VTableSection` · 006e8a30 · legions 0066e640 — C++ destructor (Rust drop replaces it)

### ByteStream
- [-] `ByteStream::ByteStream` · 006e8b00 · legions 0066e710 — C++ constructor of plain data or a container (Rust struct init)
- [-] `ByteStream::~ByteStream` · 006e8c80 · legions 0066e890 — C++ destructor (Rust drop replaces it)

### PSmartArray<ExportData>
- [x] `PSmartArray<ExportData>::StreamPack` · 006e8e90 · legions 0066eaa0 — bytestream/tables.rs fn exports (PSmartArray<ExportData>::StreamPack)

### ExportData
- [-] `ExportData::ExportData` · 006e9010 · legions 0066ec20 — C++ constructor of plain data or a container (Rust struct init)

### PSmartArray<ExportFunctionData>
- [ ] `PSmartArray<ExportFunctionData>::StreamPack` · 006e9040 · legions 0066ec50 — needs port: wlib loader table (package, vtable or nested array) in bytestream

### ExportFunctionData
- [-] `ExportFunctionData::ExportFunctionData` · 006e91c0 · legions 0066edd0 — C++ constructor of plain data or a container (Rust struct init)
- [-] `ExportFunctionData::operator=` · 006e9210 · legions 0066ee20 — C++ assignment operator of plain data (Rust copy/move)
- [~] `ExportFunctionData::StreamPack` · 006e92a0 · legions 0066eeb0 — ac2-script: unpack direction only

### ExportPackageArgs
- [-] `ExportPackageArgs::ExportPackageArgs` · 006e9310 · legions 0066ef20 — C++ constructor of plain data or a container (Rust struct init)

### DualHash<PStringBase<char>,CheckpointExportData>
- [-] `DualHash<PStringBase<char>,CheckpointExportData>::init` · 006e9380 · legions 0066ef90 — container internals (Rust collections replace them)

### ExportPackageArgs
- [~] `ExportPackageArgs::StreamPack` · 006e9440 · legions 0066f050 — ac2-script: unpack direction only

### ExportData
- [-] `ExportData::operator=` · 006e94f0 · legions 0066f100 — C++ assignment operator of plain data (Rust copy/move)

### SmartArray<ExportFunctionData>
- [-] `SmartArray<ExportFunctionData>::operator=` · 006e9530 · legions 0066f140 — C++ assignment operator of plain data (Rust copy/move)

### ExportPackageArgs
- [-] `ExportPackageArgs::operator=` · 006e95a0 · legions 0066f1b0 — C++ assignment operator of plain data (Rust copy/move)

### DualHash<PStringBase<char>,CheckpointExportData>
- [-] `DualHash<PStringBase<char>,CheckpointExportData>::operator=` · 006e9650 · legions 0066f260 — C++ assignment operator of plain data (Rust copy/move)
- [-] `DualHash<PStringBase<char>,CheckpointExportData>::inc_num_elements` · 006e97e0 · legions 0066f3f0 — container internals (Rust collections replace them)

## `portal/shared/src/interp/centraldebuginfo.cpp.c` (0/35)

### CentralDebugInfo
- [-] `CentralDebugInfo::FindNearestSourceLineIdx` · 006e9910 — debug info (debugger only)
- [-] `CentralDebugInfo::FindNearestFunctionLocationIdx` · 006e9950 — debug info (debugger only)

### DualHash<PStringBase<char>,unsigned_long>
- [-] `DualHash<PStringBase<char>,unsigned_long>::init` · 006e9a60 · legions 0066f5e0 — container internals (Rust collections replace them)

### DualHashIter<PStringBase<char>,CheckpointExportData>
- [-] `DualHashIter<PStringBase<char>,CheckpointExportData>::start_bucket` · 006e9b20 · legions 0066f6a0 — container internals (Rust collections replace them)

### CentralDebugInfo
- [-] `CentralDebugInfo::FindFunctionLocation` · 006e9c30 — debug info (debugger only)

### DualHash<PStringBase<char>,unsigned_long>
- [-] `DualHash<PStringBase<char>,unsigned_long>::find_right` · 006e9cb0 · legions 0066f7b0 — container internals (Rust collections replace them)

### OriginalSourceFileInfo
- [-] `OriginalSourceFileInfo::operator=` · 006e9d40 — C++ assignment operator of plain data (Rust copy/move)

### DualHash<PStringBase<char>,unsigned_long>
- [-] `DualHash<PStringBase<char>,unsigned_long>::find_left` · 006e9f00 · legions 0066f9b0 — container internals (Rust collections replace them)
- [-] `DualHash<PStringBase<char>,unsigned_long>::add_internal` · 006e9ff0 · legions 0066faa0 — container internals (Rust collections replace them)
- [-] `DualHash<PStringBase<char>,unsigned_long>::find_left` · 006ea090 · legions 0066fb40 — container internals (Rust collections replace them)
- [-] `DualHash<PStringBase<char>,unsigned_long>::find_right` · 006ea120 — container internals (Rust collections replace them)
- [ ] `DualHash<PStringBase<char>,unsigned_long>::UnPack` · 006ea1e0 · legions 0066fbd0 — needs port: wlib loader table (package, vtable or nested array) in bytestream
- [-] `DualHash<PStringBase<char>,unsigned_long>::contains_left` · 006ea3c0 · legions 0066fdb0 — container internals (Rust collections replace them)
- [-] `DualHash<PStringBase<char>,unsigned_long>::contains_right` · 006ea430 · legions 0066fe20 — container internals (Rust collections replace them)

### CentralDebugInfo
- [-] `CentralDebugInfo::GetFilename` · 006ea580 — debug info (debugger only)

### DualHash<PStringBase<char>,unsigned_long>
- [-] `DualHash<PStringBase<char>,unsigned_long>::GetPackSize` · 006ea650 · legions 0066ffb0 — compiled-wlib writer / pack size (client only reads wlib)
- [-] `DualHash<PStringBase<char>,unsigned_long>::pack_size` · 006ea660 · legions 0066ffc0 — compiled-wlib writer / pack size (client only reads wlib)
- [-] `DualHash<PStringBase<char>,unsigned_long>::Pack` · 006ea7e0 · legions 00670140 — compiled-wlib writer / pack size (client only reads wlib)
- [-] `DualHash<PStringBase<char>,unsigned_long>::clear` · 006ea9f0 · legions 00670350 — container internals (Rust collections replace them)

### CentralDebugInfo
- [-] `CentralDebugInfo::FindSourceLineLocation` · 006eac60 — debug info (debugger only)
- [-] `CentralDebugInfo::GetClosestSourceLineAtPos` · 006eae90 — debug info (debugger only)
- [-] `CentralDebugInfo::GetFunctionAtPos` · 006eaf90 — debug info (debugger only)
- [-] `CentralDebugInfo::GetFunctionContainingPos` · 006eb010 — debug info (debugger only)
- [-] `CentralDebugInfo::LoadReadonlyWLinkMap` · 006eb060 — text source loader, admin build only

### DualHash<PStringBase<char>,unsigned_long>
- [-] `DualHash<PStringBase<char>,unsigned_long>::destroy` · 006eb3f0 · legions 00670950 — container internals (Rust collections replace them)
- [-] `DualHash<PStringBase<char>,unsigned_long>::~DualHash<PStringBase<char>,unsigned_long>` · 006eb510 · legions 00670a70 — C++ destructor (Rust drop replaces it)
- [-] `DualHash<PStringBase<char>,unsigned_long>::inc_num_elements` · 006eb590 · legions 00670af0 — container internals (Rust collections replace them)
- [-] `DualHash<PStringBase<char>,unsigned_long>::operator=` · 006eb610 · legions 00670b70 — C++ assignment operator of plain data (Rust copy/move)

### CentralDebugInfo
- [-] `CentralDebugInfo::CentralDebugInfo` · 006eb920 · legions 00670e80 — C++ constructor of plain data or a container (Rust struct init)
- [-] `CentralDebugInfo::~CentralDebugInfo` · 006eba50 · legions 00670f80 — C++ destructor (Rust drop replaces it)
- [-] `CentralDebugInfo::PrepareForUse` · 006ebbc0 — debug info (debugger only)
- [-] `CentralDebugInfo::FindFilenameIdx` · 006ebdf0 — debug info (debugger only)
- [-] `CentralDebugInfo::GetSourceFile` · 006ebe90 — debug info (debugger only)
- [-] `CentralDebugInfo::GetLinesOfSourceFile` · 006ec280 — debug info (debugger only)
- [-] `CentralDebugInfo::GetLineOfSourceFile` · 006ec400 — debug info (debugger only)

## `portal/shared/src/interp/contextstate.cpp.c` (0/1)

### ContextState
- [-] `ContextState::QueryReferenceTree` · 006de060 — WSL debugger query or dump (process, reference and perf tables)

## `portal/shared/src/interp/debugdata.cpp.c` (0/7)

### ConversationAction
- [-] `ConversationAction::operator=` · 006ec580 · legions 00671450 — C++ assignment operator of plain data (Rust copy/move)

### PFunctionLocationInfo
- [-] `PFunctionLocationInfo::PFunctionLocationInfo` · 006ec5d0 · legions 006714a0 — C++ constructor of plain data or a container (Rust struct init)

### PFunctionLocationInfoTable
- [-] `PFunctionLocationInfoTable::GetPackSize` · 006ec790 — debug info (debugger only)
- [-] `PFunctionLocationInfoTable::Pack` · 006ec7c0 — debug info (debugger only)

### SmartArray<PFunctionLocationInfo>
- [-] `SmartArray<PFunctionLocationInfo>::AddToEnd` · 006ec8a0 · legions 00671730 — container internals (Rust collections replace them)

### PFunctionLocationInfo
- [-] `PFunctionLocationInfo::StreamPack` · 006ec930 · legions 006717c0 — debug info (debugger only)

### PFunctionLocationInfoTable
- [-] `PFunctionLocationInfoTable::UnPack` · 006ec970 — debug info (debugger only)

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
- [-] `HashSet<unsigned_long>::add` · 00565c50 · legions 00536d30 — container internals (Rust collections replace them)

### gAHashSet
- [x] `gAHashSet::CallEFun_add` · 00565cc0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

### HashSet<unsigned_long>
- [-] `HashSet<unsigned_long>::grow` · 00565d40 · legions 00536e00 — container internals (Rust collections replace them)
- [-] `HashSet<unsigned_long>::shrink` · 00565d90 · legions 00536e50 — container internals (Rust collections replace them)

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
- [-] `IntrusiveHashIterator<unsigned_long,HashSetData<unsigned_long>_*>::remove` · 00565640 — container internals (Rust collections replace them)

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
- [ ] `gAppearanceTableAdaptor::CallEFun_HasAppearanceKey` · 0054c920 — needs port: appearance table adaptor (HasAppearanceKey, key mod lists, tsys mod count)

### gCharacterGenSystem
- [~] `gCharacterGenSystem::ValueListToAList` · 0054c960 · legions 0046b490 — ac2-script host::chargen: container conversions done inline (AList/ARHash reads)
- [~] `gCharacterGenSystem::AListToValueList` · 0054c9d0 · legions 0046b500 — ac2-script host::chargen: container conversions done inline (AList/ARHash reads)

### gAppearanceTableAdaptor
- [ ] `gAppearanceTableAdaptor::CallEFun_InqAppearanceKeyModList` · 0054ca40 — needs port: appearance table adaptor (HasAppearanceKey, key mod lists, tsys mod count)
- [ ] `gAppearanceTableAdaptor::CallEFun_InqTsysAppearanceKeyModCount` · 0054caf0 — needs port: appearance table adaptor (HasAppearanceKey, key mod lists, tsys mod count)

## `portal/shared/src/interp/glue/gappinfohash.cpp.c` (0/1)

### gAppInfoHash
- [ ] `gAppInfoHash::CallEFun_num_elements` · 0054cb60 — needs port: gAppInfoHash::num_elements (appearance info hash count)

## `portal/shared/src/interp/glue/gappliedappearancekey.cpp.c` (4/5)

### gAppliedAppearanceKey
- [x] `gAppliedAppearanceKey::CallEFun_SetAppKey` · 0054c700 — ac2-script glue/visual_desc.rs (gAppliedAppearanceKey over native 0x10)
- [x] `gAppliedAppearanceKey::CallEFun_SetMod` · 0054c760 — ac2-script glue/visual_desc.rs
- [~] `gAppliedAppearanceKey::CallEFun_FromFileNode` · 0054c7c0 — FromFileNode reads a PFileNode (gPFileNode, unbound: no PFileNode native in the port; ac2-dat PFileNode rows unchecked)
- [x] `gAppliedAppearanceKey::CallEFun_GetAppKey` · 0054c840 — ac2-script glue/visual_desc.rs
- [x] `gAppliedAppearanceKey::CallEFun_GetMod` · 0054c8b0 — ac2-script glue/visual_desc.rs

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
- [ ] `BaseProperty::InqStringW` · 0054b360 · legions 005292d0 — needs port: BaseProperty InqStringW (weenie property string)
- [ ] `BaseProperty::SetStringW` · 0054b430 · legions 005293a0 — needs port: BaseProperty SetStringW (weenie property string)

### gBaseProperty
- [ ] `gBaseProperty::CallEFun_GetGroupName` · 0054b480 — needs port: weenie property get/set efuns (gBaseProperty Inq*/Set*, names, package ids, positions)
- [ ] `gBaseProperty::CallEFun_GetPropertyName` · 0054b4f0 — needs port: weenie property get/set efuns (gBaseProperty Inq*/Set*, names, package ids, positions)
- [ ] `gBaseProperty::CallEFun_GetPropertyType` · 0054b560 — needs port: weenie property get/set efuns (gBaseProperty Inq*/Set*, names, package ids, positions)
- [ ] `gBaseProperty::CallEFun_IsDefaultValue` · 0054b5d0 — needs port: weenie property get/set efuns (gBaseProperty Inq*/Set*, names, package ids, positions)
- [ ] `gBaseProperty::CallEFun_IsReportedToWeenie` · 0054b640 — needs port: weenie property get/set efuns (gBaseProperty Inq*/Set*, names, package ids, positions)
- [ ] `gBaseProperty::CallEFun_SetBool` · 0054b6b0 — needs port: weenie property get/set efuns (gBaseProperty Inq*/Set*, names, package ids, positions)
- [ ] `gBaseProperty::CallEFun_SetDataID` · 0054b740 — needs port: weenie property get/set efuns (gBaseProperty Inq*/Set*, names, package ids, positions)
- [ ] `gBaseProperty::CallEFun_SetEnum` · 0054b7d0 — needs port: weenie property get/set efuns (gBaseProperty Inq*/Set*, names, package ids, positions)
- [ ] `gBaseProperty::CallEFun_SetFloat` · 0054b860 — needs port: weenie property get/set efuns (gBaseProperty Inq*/Set*, names, package ids, positions)
- [ ] `gBaseProperty::CallEFun_SetInteger` · 0054b8f0 — needs port: weenie property get/set efuns (gBaseProperty Inq*/Set*, names, package ids, positions)
- [ ] `gBaseProperty::CallEFun_SetLongInteger` · 0054b980 — needs port: weenie property get/set efuns (gBaseProperty Inq*/Set*, names, package ids, positions)
- [ ] `gBaseProperty::CallEFun_SetPackageID` · 0054ba10 — needs port: weenie property get/set efuns (gBaseProperty Inq*/Set*, names, package ids, positions)
- [ ] `gBaseProperty::CallEFun_SetPosition` · 0054baa0 — needs port: weenie property get/set efuns (gBaseProperty Inq*/Set*, names, package ids, positions)
- [ ] `gBaseProperty::CallEFun_SetPropertyName` · 0054bb30 — needs port: weenie property get/set efuns (gBaseProperty Inq*/Set*, names, package ids, positions)
- [ ] `gBaseProperty::CallEFun_SetPropertyNameW` · 0054bbb0 — needs port: weenie property get/set efuns (gBaseProperty Inq*/Set*, names, package ids, positions)
- [ ] `gBaseProperty::CallEFun_SetStringInfo` · 0054bc30 — needs port: weenie property get/set efuns (gBaseProperty Inq*/Set*, names, package ids, positions)
- [ ] `gBaseProperty::CallEFun_SetStringW` · 0054bcc0 — needs port: weenie property get/set efuns (gBaseProperty Inq*/Set*, names, package ids, positions)
- [ ] `gBaseProperty::CallEFun_SetVector` · 0054bd40 — needs port: weenie property get/set efuns (gBaseProperty Inq*/Set*, names, package ids, positions)
- [ ] `gBaseProperty::CallEFun_InqBool` · 0054bdd0 — needs port: weenie property get/set efuns (gBaseProperty Inq*/Set*, names, package ids, positions)
- [ ] `gBaseProperty::CallEFun_InqDataID` · 0054be90 — needs port: weenie property get/set efuns (gBaseProperty Inq*/Set*, names, package ids, positions)
- [ ] `gBaseProperty::CallEFun_InqEnum` · 0054bf50 — needs port: weenie property get/set efuns (gBaseProperty Inq*/Set*, names, package ids, positions)
- [ ] `gBaseProperty::CallEFun_InqFloat` · 0054c010 — needs port: weenie property get/set efuns (gBaseProperty Inq*/Set*, names, package ids, positions)
- [ ] `gBaseProperty::CallEFun_InqInteger` · 0054c0d0 — needs port: weenie property get/set efuns (gBaseProperty Inq*/Set*, names, package ids, positions)
- [ ] `gBaseProperty::CallEFun_InqLongInteger` · 0054c190 — needs port: weenie property get/set efuns (gBaseProperty Inq*/Set*, names, package ids, positions)
- [ ] `gBaseProperty::CallEFun_InqPackageID` · 0054c250 — needs port: weenie property get/set efuns (gBaseProperty Inq*/Set*, names, package ids, positions)
- [ ] `gBaseProperty::CallEFun_InqPosition` · 0054c310 — needs port: weenie property get/set efuns (gBaseProperty Inq*/Set*, names, package ids, positions)
- [ ] `gBaseProperty::CallEFun_InqPropertyNameW` · 0054c3c0 · legions 00529f30 — needs port: weenie property get/set efuns (gBaseProperty Inq*/Set*, names, package ids, positions)
- [ ] `gBaseProperty::CallEFun_InqPropertyTypeW` · 0054c460 — needs port: weenie property get/set efuns (gBaseProperty Inq*/Set*, names, package ids, positions)
- [ ] `gBaseProperty::CallEFun_InqStringInfo` · 0054c500 — needs port: weenie property get/set efuns (gBaseProperty Inq*/Set*, names, package ids, positions)
- [ ] `gBaseProperty::CallEFun_InqStringW` · 0054c5b0 · legions 0052a0a0 — needs port: weenie property get/set efuns (gBaseProperty Inq*/Set*, names, package ids, positions)
- [ ] `gBaseProperty::CallEFun_InqVector` · 0054c650 — needs port: weenie property get/set efuns (gBaseProperty Inq*/Set*, names, package ids, positions)

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
- [ ] `gCaseInsensitiveWPString::CallEFun_clear` · 005500e0 — needs port: case-insensitive wide string efuns (compare, find, case map, encrypt, append)
- [ ] `gCaseInsensitiveWPString::CallEFun_cmp` · 00550170 — needs port: case-insensitive wide string efuns (compare, find, case map, encrypt, append)
- [ ] `gCaseInsensitiveWPString::CallEFun_empty` · 00550220 — needs port: case-insensitive wide string efuns (compare, find, case map, encrypt, append)
- [ ] `gCaseInsensitiveWPString::CallEFun_eq` · 00550290 — needs port: case-insensitive wide string efuns (compare, find, case map, encrypt, append)
- [ ] `gCaseInsensitiveWPString::CallEFun_find_substring` · 00550320 — needs port: case-insensitive wide string efuns (compare, find, case map, encrypt, append)
- [ ] `gCaseInsensitiveWPString::CallEFun_ge` · 005503c0 — needs port: case-insensitive wide string efuns (compare, find, case map, encrypt, append)
- [ ] `gCaseInsensitiveWPString::CallEFun_gt` · 00550450 — needs port: case-insensitive wide string efuns (compare, find, case map, encrypt, append)
- [ ] `gCaseInsensitiveWPString::CallEFun_hash` · 005504e0 — needs port: case-insensitive wide string efuns (compare, find, case map, encrypt, append)
- [ ] `gCaseInsensitiveWPString::CallEFun_le` · 00550560 — needs port: case-insensitive wide string efuns (compare, find, case map, encrypt, append)
- [ ] `gCaseInsensitiveWPString::CallEFun_length` · 005505f0 — needs port: case-insensitive wide string efuns (compare, find, case map, encrypt, append)
- [ ] `gCaseInsensitiveWPString::CallEFun_lt` · 00550660 — needs port: case-insensitive wide string efuns (compare, find, case map, encrypt, append)
- [ ] `gCaseInsensitiveWPString::CallEFun_ne` · 005506f0 — needs port: case-insensitive wide string efuns (compare, find, case map, encrypt, append)
- [ ] `gCaseInsensitiveWPString::CallEFun_set` · 00550780 — needs port: case-insensitive wide string efuns (compare, find, case map, encrypt, append)
- [ ] `gCaseInsensitiveWPString::CallEFun_to_float` · 00550810 — needs port: case-insensitive wide string efuns (compare, find, case map, encrypt, append)
- [ ] `gCaseInsensitiveWPString::CallEFun_to_int32` · 005508a0 — needs port: case-insensitive wide string efuns (compare, find, case map, encrypt, append)
- [ ] `gCaseInsensitiveWPString::CallEFun_to_int64` · 00550930 — needs port: case-insensitive wide string efuns (compare, find, case map, encrypt, append)
- [ ] `gCaseInsensitiveWPString::CallEFun_to_uint32` · 005509c0 — needs port: case-insensitive wide string efuns (compare, find, case map, encrypt, append)
- [ ] `gCaseInsensitiveWPString::CallEFun_to_uint64` · 00550a50 — needs port: case-insensitive wide string efuns (compare, find, case map, encrypt, append)
- [ ] `gCaseInsensitiveWPString::CallEFun_decrypt` · 00550ae0 — needs port: case-insensitive wide string efuns (compare, find, case map, encrypt, append)
- [ ] `gCaseInsensitiveWPString::CallEFun_encrypt` · 00550b40 — needs port: case-insensitive wide string efuns (compare, find, case map, encrypt, append)
- [ ] `gCaseInsensitiveWPString::CallEFun_tolower` · 00550ba0 — needs port: case-insensitive wide string efuns (compare, find, case map, encrypt, append)
- [ ] `gCaseInsensitiveWPString::CallEFun_toupper` · 00550c10 — needs port: case-insensitive wide string efuns (compare, find, case map, encrypt, append)
- [ ] `gCaseInsensitiveWPString::CallEFun_append_float` · 00550df0 — needs port: case-insensitive wide string efuns (compare, find, case map, encrypt, append)
- [ ] `gCaseInsensitiveWPString::CallEFun_append_int32` · 00550e50 — needs port: case-insensitive wide string efuns (compare, find, case map, encrypt, append)
- [ ] `gCaseInsensitiveWPString::CallEFun_append_int32_hex` · 00550eb0 — needs port: case-insensitive wide string efuns (compare, find, case map, encrypt, append)
- [ ] `gCaseInsensitiveWPString::CallEFun_append_int64` · 00550f10 — needs port: case-insensitive wide string efuns (compare, find, case map, encrypt, append)
- [ ] `gCaseInsensitiveWPString::CallEFun_append_string` · 00550f70 — needs port: case-insensitive wide string efuns (compare, find, case map, encrypt, append)
- [ ] `gCaseInsensitiveWPString::CallEFun_append_uint32` · 00550ff0 — needs port: case-insensitive wide string efuns (compare, find, case map, encrypt, append)
- [ ] `gCaseInsensitiveWPString::CallEFun_append_uint32_hex` · 00551050 — needs port: case-insensitive wide string efuns (compare, find, case map, encrypt, append)
- [ ] `gCaseInsensitiveWPString::CallEFun_trim` · 005510b0 — needs port: case-insensitive wide string efuns (compare, find, case map, encrypt, append)
- [ ] `gCaseInsensitiveWPString::CallEFun_append_bool` · 00551130 — needs port: case-insensitive wide string efuns (compare, find, case map, encrypt, append)
- [ ] `gCaseInsensitiveWPString::CallEFun_substring` · 00551190 — needs port: case-insensitive wide string efuns (compare, find, case map, encrypt, append)
- [ ] `gCaseInsensitiveWPString::CallEFun_replace` · 00551260 — needs port: case-insensitive wide string efuns (compare, find, case map, encrypt, append)

## `portal/shared/src/interp/glue/gcdbform.cpp.c` (0/14)

### CDBForm
- [ ] `CDBForm::SetNameW` · 0054a200 · legions 00528910 — needs port: CDBForm SetNameW (DB form builder)

### gCDBForm
- [ ] `gCDBForm::CallEFun_SetNameW` · 0054a270 — needs port: DB form builder efuns (SetNameW, AddProperty*, AddTable)
- [ ] `gCDBForm::CallEFun_AddTable` · 0054a2d0 — needs port: DB form builder efuns (SetNameW, AddProperty*, AddTable)

### CDBForm
- [ ] `CDBForm::AddPropertyW` · 0054a380 · legions 00528a50 — needs port: CDBForm AddPropertyW (DB form builder)
- [ ] `CDBForm::AddProperty` · 0054a3e0 · legions 00528ab0 — needs port: CDBForm AddProperty (DB form builder)
- [ ] `CDBForm::AddProperty` · 0054a450 · legions 00528b20 — needs port: CDBForm AddProperty (DB form builder)
- [ ] `CDBForm::AddProperty` · 0054a4c0 · legions 00528b90 — needs port: CDBForm AddProperty (DB form builder)
- [ ] `CDBForm::AddProperty` · 0054a530 · legions 00528c00 — needs port: CDBForm AddProperty (DB form builder)

### gCDBForm
- [ ] `gCDBForm::CallEFun_AddFloatProperty` · 0054a5a0 — needs port: DB form builder efuns (SetNameW, AddProperty*, AddTable)
- [ ] `gCDBForm::CallEFun_AddIntProperty` · 0054a600 — needs port: DB form builder efuns (SetNameW, AddProperty*, AddTable)
- [ ] `gCDBForm::CallEFun_AddLongIntProperty` · 0054a660 — needs port: DB form builder efuns (SetNameW, AddProperty*, AddTable)
- [ ] `gCDBForm::CallEFun_AddPropertyW` · 0054a6c0 — needs port: DB form builder efuns (SetNameW, AddProperty*, AddTable)
- [ ] `gCDBForm::CallEFun_AddStringProperty` · 0054a730 — needs port: DB form builder efuns (SetNameW, AddProperty*, AddTable)
- [ ] `gCDBForm::CallEFun_AddTimetypeProperty` · 0054a7a0 — needs port: DB form builder efuns (SetNameW, AddProperty*, AddTable)

## `portal/shared/src/interp/glue/gcollideproperty.cpp.c` (0/6)

### gCollideProperty
- [ ] `gCollideProperty::CallEFun_GetID` · 00549f00 — needs port: collide property efuns (GetID, IsBegin/IsEnd, GetProperty)
- [ ] `gCollideProperty::CallEFun_IsBegin` · 00549f70 — needs port: collide property efuns (GetID, IsBegin/IsEnd, GetProperty)
- [ ] `gCollideProperty::CallEFun_IsChanged` · 00549fe0 — needs port: collide property efuns (GetID, IsBegin/IsEnd, GetProperty)
- [ ] `gCollideProperty::CallEFun_IsEnd` · 0054a050 — needs port: collide property efuns (GetID, IsBegin/IsEnd, GetProperty)
- [ ] `gCollideProperty::CallEFun_GetProperty` · 0054a0c0 — needs port: collide property efuns (GetID, IsBegin/IsEnd, GetProperty)
- [ ] `gCollideProperty::CallEFun_GetPropertyOutputStringW` · 0054a170 · legions 005288a0 — needs port: collide property efuns (GetID, IsBegin/IsEnd, GetProperty)

## `portal/shared/src/interp/glue/gcollisionparameters.cpp.c` (0/25)

### gCollisionParameters
- [ ] `gCollisionParameters::CallEFun_SetMinReportedImpactXY` · 005491f0 — needs port: collision parameter setters/getters (physics and placement types, ethereal list)
- [ ] `gCollisionParameters::CallEFun_SetMinReportedImpactZ` · 00549250 — needs port: collision parameter setters/getters (physics and placement types, ethereal list)
- [ ] `gCollisionParameters::CallEFun_SetMovementHighDWord` · 005492b0 — needs port: collision parameter setters/getters (physics and placement types, ethereal list)
- [ ] `gCollisionParameters::CallEFun_SetMovementLowDWord` · 00549320 — needs port: collision parameter setters/getters (physics and placement types, ethereal list)
- [ ] `gCollisionParameters::CallEFun_SetPhysicsMovementType` · 00549390 — needs port: collision parameter setters/getters (physics and placement types, ethereal list)
- [ ] `gCollisionParameters::CallEFun_SetPhysicsPlacementType` · 00549400 — needs port: collision parameter setters/getters (physics and placement types, ethereal list)
- [ ] `gCollisionParameters::CallEFun_SetPhysicsType` · 00549470 — needs port: collision parameter setters/getters (physics and placement types, ethereal list)
- [ ] `gCollisionParameters::CallEFun_SetPhysicsTypeHighDWord` · 005494e0 — needs port: collision parameter setters/getters (physics and placement types, ethereal list)
- [ ] `gCollisionParameters::CallEFun_SetPhysicsTypeLowDWord` · 00549550 — needs port: collision parameter setters/getters (physics and placement types, ethereal list)
- [ ] `gCollisionParameters::CallEFun_SetPlacementHighDWord` · 005495c0 — needs port: collision parameter setters/getters (physics and placement types, ethereal list)
- [ ] `gCollisionParameters::CallEFun_SetPlacementLowDWord` · 00549630 — needs port: collision parameter setters/getters (physics and placement types, ethereal list)
- [ ] `gCollisionParameters::CallEFun_SetWeenieMovementType` · 005496a0 — needs port: collision parameter setters/getters (physics and placement types, ethereal list)
- [ ] `gCollisionParameters::CallEFun_SetWeeniePlacementType` · 00549710 — needs port: collision parameter setters/getters (physics and placement types, ethereal list)
- [ ] `gCollisionParameters::CallEFun_AddToEtherealList` · 00549780 — needs port: collision parameter setters/getters (physics and placement types, ethereal list)
- [ ] `gCollisionParameters::CallEFun_RemovePhysicsType` · 00549800 — needs port: collision parameter setters/getters (physics and placement types, ethereal list)
- [ ] `gCollisionParameters::CallEFun_UpdateMovementEthereal` · 00549880 — needs port: collision parameter setters/getters (physics and placement types, ethereal list)
- [ ] `gCollisionParameters::CallEFun_UpdateMovementSolid` · 00549900 — needs port: collision parameter setters/getters (physics and placement types, ethereal list)
- [ ] `gCollisionParameters::CallEFun_UpdatePhysicsType` · 00549980 — needs port: collision parameter setters/getters (physics and placement types, ethereal list)
- [ ] `gCollisionParameters::CallEFun_UpdatePlacementEthereal` · 00549a00 — needs port: collision parameter setters/getters (physics and placement types, ethereal list)
- [ ] `gCollisionParameters::CallEFun_UpdatePlacementSolid` · 00549a80 — needs port: collision parameter setters/getters (physics and placement types, ethereal list)
- [ ] `gCollisionParameters::CallEFun_GetPhysicsMovementType` · 00549b00 — needs port: collision parameter setters/getters (physics and placement types, ethereal list)
- [ ] `gCollisionParameters::CallEFun_GetPhysicsPlacementType` · 00549bc0 — needs port: collision parameter setters/getters (physics and placement types, ethereal list)
- [ ] `gCollisionParameters::CallEFun_GetPhysicsType` · 00549c80 — needs port: collision parameter setters/getters (physics and placement types, ethereal list)
- [ ] `gCollisionParameters::CallEFun_GetWeenieMovementType` · 00549d40 — needs port: collision parameter setters/getters (physics and placement types, ethereal list)
- [ ] `gCollisionParameters::CallEFun_GetWeeniePlacementType` · 00549e20 — needs port: collision parameter setters/getters (physics and placement types, ethereal list)

## `portal/shared/src/interp/glue/gcollisionprofile.cpp.c` (0/15)

### gCollisionProfile
- [ ] `gCollisionProfile::CallEFun_GetNumEnvProperties` · 005489a0 — needs port: collision profile inquiries (object, environment, surface and water properties, contacts)
- [ ] `gCollisionProfile::CallEFun_GetNumObjectProperties` · 00548a10 — needs port: collision profile inquiries (object, environment, surface and water properties, contacts)
- [ ] `gCollisionProfile::CallEFun_GetNumObjects` · 00548a80 — needs port: collision profile inquiries (object, environment, surface and water properties, contacts)
- [ ] `gCollisionProfile::CallEFun_GetNumSurfaceProperties` · 00548af0 — needs port: collision profile inquiries (object, environment, surface and water properties, contacts)
- [ ] `gCollisionProfile::CallEFun_GetNumWaterProperties` · 00548b60 — needs port: collision profile inquiries (object, environment, surface and water properties, contacts)
- [ ] `gCollisionProfile::CallEFun_IsEndCollision` · 00548bd0 — needs port: collision profile inquiries (object, environment, surface and water properties, contacts)
- [ ] `gCollisionProfile::CallEFun_IsEndContact` · 00548c40 — needs port: collision profile inquiries (object, environment, surface and water properties, contacts)
- [ ] `gCollisionProfile::CallEFun_FindObject` · 00548cb0 — needs port: collision profile inquiries (object, environment, surface and water properties, contacts)
- [ ] `gCollisionProfile::CallEFun_GetEnvProperty` · 00548d60 — needs port: collision profile inquiries (object, environment, surface and water properties, contacts)
- [ ] `gCollisionProfile::CallEFun_GetObject` · 00548e00 · legions 00527aa0 — needs port: collision profile inquiries (object, environment, surface and water properties, contacts)
- [ ] `gCollisionProfile::CallEFun_GetObjectProperty` · 00548ea0 — needs port: collision profile inquiries (object, environment, surface and water properties, contacts)
- [ ] `gCollisionProfile::CallEFun_GetSurfaceProperty` · 00548f40 — needs port: collision profile inquiries (object, environment, surface and water properties, contacts)
- [ ] `gCollisionProfile::CallEFun_GetWaterProperty` · 00548fe0 — needs port: collision profile inquiries (object, environment, surface and water properties, contacts)
- [ ] `gCollisionProfile::CallEFun_HasCollided` · 00549080 · legions 00527ca0 — needs port: collision profile inquiries (object, environment, surface and water properties, contacts)
- [ ] `gCollisionProfile::CallEFun_HasContact` · 00549150 · legions 00527d50 — needs port: collision profile inquiries (object, environment, surface and water properties, contacts)

## `portal/shared/src/interp/glue/gdbcache.cpp.c` (4/4)

### gDBCache
- [x] `gDBCache::CallEFun_GetDIDFromEnumStatic` · 00536af0 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gDBCache::CallEFun_IsClient` · 00536b30 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gDBCache::CallEFun_IsRunTime` · 00536b60 — ac2-script glue (wlog, random, behavior, enums: gDBCache)
- [x] `gDBCache::CallEFun_IsServer` · 00536b90 — ac2-script glue (wlog, random, behavior, enums: gDBCache)

## `portal/shared/src/interp/glue/gdebug.cpp.c` (0/11)

### gdebug
- [-] `gdebug::CallEFun_PrintFloat` · 005600a0 — debugger/debug output efun (admin only)
- [-] `gdebug::CallEFun_PrintInt` · 005600c0 — debugger/debug output efun (admin only)
- [-] `gdebug::CallEFun_PrintLongFloat` · 005600e0 — debugger/debug output efun (admin only)
- [-] `gdebug::CallEFun_PrintLongInt` · 00560100 — debugger/debug output efun (admin only)
- [-] `gdebug::CallEFun_PrintString` · 00560120 — debugger/debug output efun (admin only)
- [ ] `gdebug::CallEFun_WSLAssert` · 00560150 · legions 00534580 — needs port: gdebug WSL assert efuns (assert failure path)
- [ ] `gdebug::CallEFun_WSLAssertFailure` · 00560190 · legions 005345c0 — needs port: gdebug WSL assert efuns (assert failure path)
- [ ] `gdebug::CallEFun_WSLAssertMsg` · 005601e0 · legions 00534610 — needs port: gdebug WSL assert efuns (assert failure path)
- [-] `gdebug::CallEFun_WSLDebugger` · 00560230 · legions 00534660 — debugger/debug output efun (admin only)
- [-] `gdebug::CallEFun_MapIIDtoProcID` · 00560270 — debugger/debug output efun (admin only)
- [-] `gdebug::CallEFun_MapProcIDtoIID` · 005602b0 — debugger/debug output efun (admin only)

## `portal/shared/src/interp/glue/gdetectionparameters.cpp.c` (0/13)

### gDetectionParameters
- [ ] `gDetectionParameters::CallEFun_SetBasePosition` · 00548390 — needs port: detection parameter setters (shape, base position, targets, LOS masks)
- [ ] `gDetectionParameters::CallEFun_SetDetectOffset` · 005483f0 — needs port: detection parameter setters (shape, base position, targets, LOS masks)
- [ ] `gDetectionParameters::CallEFun_SetDetectionObjectType` · 00548490 — needs port: detection parameter setters (shape, base position, targets, LOS masks)
- [ ] `gDetectionParameters::CallEFun_SetLOSBlockingMasks` · 005484f0 — needs port: detection parameter setters (shape, base position, targets, LOS masks)
- [ ] `gDetectionParameters::CallEFun_SetShapeAttackCone` · 00548560 — needs port: detection parameter setters (shape, base position, targets, LOS masks)
- [ ] `gDetectionParameters::CallEFun_SetShapeBox` · 005485f0 — needs port: detection parameter setters (shape, base position, targets, LOS masks)
- [ ] `gDetectionParameters::CallEFun_SetShapeCylinder` · 00548670 — needs port: detection parameter setters (shape, base position, targets, LOS masks)
- [ ] `gDetectionParameters::CallEFun_SetShapeSphere` · 005486e0 — needs port: detection parameter setters (shape, base position, targets, LOS masks)
- [ ] `gDetectionParameters::CallEFun_SetShapeSphereCone` · 00548740 — needs port: detection parameter setters (shape, base position, targets, LOS masks)
- [ ] `gDetectionParameters::CallEFun_SetShapeSphereWithOffset` · 005487c0 — needs port: detection parameter setters (shape, base position, targets, LOS masks)
- [ ] `gDetectionParameters::CallEFun_UseDetectingObjectForBasePosition` · 00548830 — needs port: detection parameter setters (shape, base position, targets, LOS masks)
- [ ] `gDetectionParameters::CallEFun_AddExcludeObject` · 005488a0 — needs port: detection parameter setters (shape, base position, targets, LOS masks)
- [ ] `gDetectionParameters::CallEFun_AddTarget` · 00548920 — needs port: detection parameter setters (shape, base position, targets, LOS masks)

## `portal/shared/src/interp/glue/gdetectionprofile.cpp.c` (0/9)

### gDetectionProfile
- [ ] `gDetectionProfile::CallEFun_SetFloatApp` · 00547ee0 — needs port: detection profile getters (coverage, float/int appearance, report type, position)
- [ ] `gDetectionProfile::CallEFun_SetIntApp` · 00547f40 — needs port: detection profile getters (coverage, float/int appearance, report type, position)
- [ ] `gDetectionProfile::CallEFun_GetCoverage` · 00547fa0 — needs port: detection profile getters (coverage, float/int appearance, report type, position)
- [ ] `gDetectionProfile::CallEFun_GetEntityDID` · 00548010 — needs port: detection profile getters (coverage, float/int appearance, report type, position)
- [ ] `gDetectionProfile::CallEFun_GetFloatApp` · 00548080 — needs port: detection profile getters (coverage, float/int appearance, report type, position)
- [ ] `gDetectionProfile::CallEFun_GetID` · 00548100 — needs port: detection profile getters (coverage, float/int appearance, report type, position)
- [ ] `gDetectionProfile::CallEFun_GetIntApp` · 00548170 — needs port: detection profile getters (coverage, float/int appearance, report type, position)
- [ ] `gDetectionProfile::CallEFun_GetDetectionReportType` · 005481f0 — needs port: detection profile getters (coverage, float/int appearance, report type, position)
- [ ] `gDetectionProfile::CallEFun_GetPosition` · 005482a0 — needs port: detection profile getters (coverage, float/int appearance, report type, position)

## `portal/shared/src/interp/glue/gdetectionreporttype.cpp.c` (0/4)

### gDetectionReportType
- [ ] `gDetectionReportType::CallEFun_SetHighDWord` · 00547d10 — needs port: detection report type dword get/set
- [ ] `gDetectionReportType::CallEFun_SetLowDWord` · 00547d80 — needs port: detection report type dword get/set
- [ ] `gDetectionReportType::CallEFun_GetHighDWord` · 00547df0 — needs port: detection report type dword get/set
- [ ] `gDetectionReportType::CallEFun_GetLowDWord` · 00547e70 — needs port: detection report type dword get/set

## `portal/shared/src/interp/glue/gefuntestsystem.cpp.c` (0/2)

### EFunTestSystem
- [-] `EFunTestSystem::Run` · 00560000 · legions 00534430 — test system (admin test harness)

### gEFunTestSystem
- [-] `gEFunTestSystem::CallEFun_Run` · 00560070 — admin only: test system efun (admin test harness)

## `portal/shared/src/interp/glue/gemailutils.cpp.c` (0/1)

### gEmailUtils
- [-] `gEmailUtils::CallEFun_send_emailW` · 00547c90 — admin only: admin email tool (no Legions match)

## `portal/shared/src/interp/glue/gentitydesc.cpp.c` (4/28)

### gEntityDesc
- [ ] `gEntityDesc::CallEFun_GetDataID` · 00546c10 — needs port: entity description getters and inquiries (glue/entity_desc.rs covers only 4 of the 23)
- [x] `gEntityDesc::CallEFun_GetDataIDFromEntityNameW` · 00546c80 — ac2-script glue::enums (FileArray::GetDID: no name data on the DVD)
- [ ] `gEntityDesc::CallEFun_GetRootPackageID` · 00546cc0 — needs port: entity description getters and inquiries (glue/entity_desc.rs covers only 4 of the 23)
- [ ] `gEntityDesc::CallEFun_GetRuntimeID` · 00546d30 — needs port: entity description getters and inquiries (glue/entity_desc.rs covers only 4 of the 23)
- [ ] `gEntityDesc::CallEFun_GetType` · 00546da0 — needs port: entity description getters and inquiries (glue/entity_desc.rs covers only 4 of the 23)
- [ ] `gEntityDesc::CallEFun_GetVersion` · 00546e10 — needs port: entity description getters and inquiries (glue/entity_desc.rs covers only 4 of the 23)
- [ ] `gEntityDesc::CallEFun_GetWBID` · 00546e80 — needs port: entity description getters and inquiries (glue/entity_desc.rs covers only 4 of the 23)
- [ ] `gEntityDesc::CallEFun_IsInside` · 00546f00 — needs port: entity description getters and inquiries (glue/entity_desc.rs covers only 4 of the 23)
- [ ] `gEntityDesc::CallEFun_SetProperties` · 00546f70 — needs port: entity description getters and inquiries (glue/entity_desc.rs covers only 4 of the 23)
- [ ] `gEntityDesc::CallEFun_GetCommentsW` · 00546ff0 — needs port: entity description getters and inquiries (glue/entity_desc.rs covers only 4 of the 23)
- [ ] `gEntityDesc::CallEFun_GetEntityNameFromDataIDW` · 005470b0 · legions 00526470 — needs port: entity description getters and inquiries (glue/entity_desc.rs covers only 4 of the 23)
- [ ] `gEntityDesc::CallEFun_GetOffset` · 00547130 — needs port: entity description getters and inquiries (glue/entity_desc.rs covers only 4 of the 23)
- [ ] `gEntityDesc::CallEFun_GetProperties` · 00547210 — needs port: entity description getters and inquiries (glue/entity_desc.rs covers only 4 of the 23)
- [ ] `gEntityDesc::CallEFun_GetScale` · 005472f0 — needs port: entity description getters and inquiries (glue/entity_desc.rs covers only 4 of the 23)
- [ ] `gEntityDesc::CallEFun_GetWBNameW` · 005473b0 — needs port: entity description getters and inquiries (glue/entity_desc.rs covers only 4 of the 23)
- [ ] `gEntityDesc::CallEFun_InqBool` · 00547480 — needs port: entity description getters and inquiries (glue/entity_desc.rs covers only 4 of the 23)
- [ ] `gEntityDesc::CallEFun_InqDataFilePathW` · 00547530 · legions 00526830 — needs port: entity description getters and inquiries (glue/entity_desc.rs covers only 4 of the 23)
- [ ] `gEntityDesc::CallEFun_InqDataFileW` · 005475d0 · legions 005268a0 — needs port: entity description getters and inquiries (glue/entity_desc.rs covers only 4 of the 23)
- [x] `gEntityDesc::CallEFun_InqDataID` · 00547670 — ac2-script glue/entity_desc.rs: the entity link (NativeObject::EntityDesc) reads the record's properties through ScriptData::entity_properties (worker load, DataRequest::EntityDesc; DB type 0x40000049); InqDataID writes a DataFile property's DID (other types 0). Names and icons of the sale come through it
- [x] `gEntityDesc::CallEFun_InqEntityDescFromDataID` · 00547720 — ac2-script glue/entity_desc.rs: the entity link (NativeObject::EntityDesc) reads the record's properties through ScriptData::entity_properties (worker load, DataRequest::EntityDesc; DB type 0x40000049); InqDataID writes a DataFile property's DID (other types 0). Names and icons of the sale come through it
- [ ] `gEntityDesc::CallEFun_InqEntityDescFromNameW` · 00547780 — needs port: entity description getters and inquiries (glue/entity_desc.rs covers only 4 of the 23)
- [ ] `gEntityDesc::CallEFun_InqEnum` · 005477f0 — needs port: entity description getters and inquiries (glue/entity_desc.rs covers only 4 of the 23)
- [ ] `gEntityDesc::CallEFun_InqFloat` · 005478a0 — needs port: entity description getters and inquiries (glue/entity_desc.rs covers only 4 of the 23)
- [x] `gEntityDesc::CallEFun_InqInteger` · 00547950 — out args read through the SADDR local (out_native); LongInteger (0x4000000D) answers 0 and writes nothing, as EntityDesc::InqInteger's base virtual does
- [ ] `gEntityDesc::CallEFun_InqPackageID` · 00547a00 — needs port: entity description getters and inquiries (glue/entity_desc.rs covers only 4 of the 23)
- [~] `gEntityDesc::CallEFun_InqStringInfo` · 00547ab0 — ac2-script glue/entity_desc.rs: the DB type check (DB_TYPE_ENTITYDESC 0x40000049) answers InqEntityDescFromDataID; the entity's properties are not read (InqStringInfo, InqDataID report missing), so names and icons the sale build takes from them stay empty
- [ ] `gEntityDesc::CallEFun_InqStringW` · 00547b50 · legions 00526d30 — needs port: entity description getters and inquiries (glue/entity_desc.rs covers only 4 of the 23)
- [ ] `gEntityDesc::CallEFun_InqVector` · 00547bf0 — needs port: entity description getters and inquiries (glue/entity_desc.rs covers only 4 of the 23)

## `portal/shared/src/interp/glue/gentitylinkdesc.cpp.c` (0/7)

### gEntityLinkDesc
- [ ] `gEntityLinkDesc::CallEFun_GetInputID` · 005366d0 — needs port: entity link description getters (input/output id, link type, properties)
- [ ] `gEntityLinkDesc::CallEFun_GetLinkType` · 00536740 — needs port: entity link description getters (input/output id, link type, properties)
- [ ] `gEntityLinkDesc::CallEFun_GetOppositeID` · 005367b0 — needs port: entity link description getters (input/output id, link type, properties)
- [ ] `gEntityLinkDesc::CallEFun_GetOutputID` · 00536840 — needs port: entity link description getters (input/output id, link type, properties)
- [ ] `gEntityLinkDesc::CallEFun_GetProperties` · 005368b0 — needs port: entity link description getters (input/output id, link type, properties)
- [ ] `gEntityLinkDesc::CallEFun_GetWBName_ForWsl` · 00536990 — needs port: entity link description getters (input/output id, link type, properties)
- [ ] `gEntityLinkDesc::CallEFun_InqStringPropertyW` · 00536a50 · legions 0051a7e0 — needs port: entity link description getters (input/output id, link type, properties)

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
- [ ] `gEtherealPhysicsType::CallEFun_SetEtherealToWaterLayer` · 00546520 — needs port: ethereal physics type setters (to ethereal/solid, property setters)
- [ ] `gEtherealPhysicsType::CallEFun_SetSolidOnlyToEnvironment` · 00546580 — needs port: ethereal physics type setters (to ethereal/solid, property setters)
- [ ] `gEtherealPhysicsType::CallEFun_SetToEthereal` · 005465e0 — needs port: ethereal physics type setters (to ethereal/solid, property setters)
- [ ] `gEtherealPhysicsType::CallEFun_SetToSolid` · 00546640 — needs port: ethereal physics type setters (to ethereal/solid, property setters)
- [ ] `gEtherealPhysicsType::CallEFun_SetProperty` · 005466a0 — needs port: ethereal physics type setters (to ethereal/solid, property setters)
- [ ] `gEtherealPhysicsType::CallEFun_SetPropertyEthereal` · 00546720 — needs port: ethereal physics type setters (to ethereal/solid, property setters)
- [ ] `gEtherealPhysicsType::CallEFun_SetPropertySolid` · 005467a0 — needs port: ethereal physics type setters (to ethereal/solid, property setters)

## `portal/shared/src/interp/glue/getherealweenietype.cpp.c` (0/4)

### gEtherealWeenieType
- [ ] `gEtherealWeenieType::CallEFun_SetToEthereal` · 00546360 — needs port: ethereal weenie type setters and list adds
- [ ] `gEtherealWeenieType::CallEFun_SetToSolid` · 005463c0 — needs port: ethereal weenie type setters and list adds
- [ ] `gEtherealWeenieType::CallEFun_AddToEtherealList` · 00546420 — needs port: ethereal weenie type setters and list adds
- [ ] `gEtherealWeenieType::CallEFun_AddToSolidList` · 005464a0 — needs port: ethereal weenie type setters and list adds

## `portal/shared/src/interp/glue/gfile.cpp.c` (0/15)

### WSLFilesCleaner
- [-] `WSLFilesCleaner::~WSLFilesCleaner` · 005455b0 · legions 00524c10 — C++ destructor (Rust drop replaces it)

### SmartArray<PStringBase<unsigned_short>_>
- [-] `SmartArray<PStringBase<unsigned_short>_>::AddToEnd` · 00545670 · legions 00524c70 — container internals (Rust collections replace them)

### (free functions)
- [ ] `WSLCloseFile` · 005456f0 · legions 00524cf0 — needs port: script file I/O (WSLCloseFile)

### gFile
- [ ] `gFile::CallEFun_WSLCloseFile` · 00545790 — needs port: script file I/O efuns (WSL open/read/write/close, timestamped write)
- [ ] `gFile::RArrayToWPStringArray` · 005457e0 · legions 00524de0 — needs port: script file I/O helper (RArrayToWPStringArray)

### (free functions)
- [ ] `WSLOpenFile` · 00545910 · legions 00524f10 — needs port: script file I/O (WSLOpenFile)
- [ ] `WSLReadFromFile` · 00545ab0 · legions 005250b0 — needs port: script file I/O (WSLReadFromFile)

### gFile
- [ ] `gFile::CallEFun_WSLOpenFile` · 00545cb0 — needs port: script file I/O efuns (WSL open/read/write/close, timestamped write)
- [ ] `gFile::WPStringArrayToRArray` · 00545d10 · legions 00525310 — needs port: script file I/O helper (WPStringArrayToRArray)

### (free functions)
- [ ] `WriteToFile` · 00545db0 · legions 005253b0 — needs port: script file I/O (WriteToFile)
- [ ] `WSLWriteToFileTimestamped` · 00545ec0 · legions 005254c0 — needs port: script file I/O (WSLWriteToFileTimestamped)
- [ ] `WSLWriteToFile` · 00546070 · legions 00525670 — needs port: script file I/O (WSLWriteToFile)

### gFile
- [ ] `gFile::CallEFun_WSLWriteToFile` · 005461c0 — needs port: script file I/O efuns (WSL open/read/write/close, timestamped write)
- [ ] `gFile::CallEFun_WSLWriteToFileTimestamped` · 00546230 — needs port: script file I/O efuns (WSL open/read/write/close, timestamped write)
- [ ] `gFile::CallEFun_WSLReadFromFile` · 005462a0 — needs port: script file I/O efuns (WSL open/read/write/close, timestamped write)

## `portal/shared/src/interp/glue/gfilearray.cpp.c` (1/4)

### gFileArray
- [~] `gFileArray::CallEFun_GetDIDForDummiesW` · 00545420 — ac2-script: FileArray has no data in Legions (no READONLY_DIDName.dat); DivineType by DID range
- [x] `gFileArray::CallEFun_GetDIDFromEnum` · 00545460 — ac2-script data/glue::enums
- [~] `gFileArray::CallEFun_GetDIDW` · 005454a0 — ac2-script: FileArray has no data in Legions (no READONLY_DIDName.dat); DivineType by DID range
- [~] `gFileArray::CallEFun_GetFileNameForDummiesW` · 005454f0 · legions 00524b00 — ac2-script: FileArray has no data in Legions (no READONLY_DIDName.dat); DivineType by DID range

## `portal/shared/src/interp/glue/gframe.cpp.c` (0/1)

### gFrame
- [ ] `gFrame::CallEFun_SetOrigin` · 00543cf0 — needs port: frame origin setter

## `portal/shared/src/interp/glue/gfxandbehaviorinfo.cpp.c` (0/5)

### gFXAndBehaviorInfo
- [ ] `gFXAndBehaviorInfo::CallEFun_SetAttackerFX` · 005439f0 — needs port: FX and behaviour info setters (attacker/target/weapon FX and behaviour)
- [ ] `gFXAndBehaviorInfo::CallEFun_SetTargetBehavior` · 00543a50 — needs port: FX and behaviour info setters (attacker/target/weapon FX and behaviour)
- [ ] `gFXAndBehaviorInfo::CallEFun_SetTargetFX` · 00543ab0 — needs port: FX and behaviour info setters (attacker/target/weapon FX and behaviour)
- [ ] `gFXAndBehaviorInfo::CallEFun_SetWeaponBehavior` · 00543b10 — needs port: FX and behaviour info setters (attacker/target/weapon FX and behaviour)
- [ ] `gFXAndBehaviorInfo::CallEFun_SetWeaponFX` · 00543b70 — needs port: FX and behaviour info setters (attacker/target/weapon FX and behaviour)

## `portal/shared/src/interp/glue/ggametime.cpp.c` (0/10)

### gGameTime
- [ ] `gGameTime::CallEFun_AlterTimeOfDayImmediate` · 00543630 — needs port: game time efuns (time of day, day/year length, date strings, dilation)
- [ ] `gGameTime::CallEFun_GetDayInYear` · 00543670 — needs port: game time efuns (time of day, day/year length, date strings, dilation)
- [ ] `gGameTime::CallEFun_GetDayLength` · 005436e0 — needs port: game time efuns (time of day, day/year length, date strings, dilation)
- [ ] `gGameTime::CallEFun_GetTimeOfDay` · 00543750 — needs port: game time efuns (time of day, day/year length, date strings, dilation)
- [ ] `gGameTime::CallEFun_GetYear` · 005437c0 — needs port: game time efuns (time of day, day/year length, date strings, dilation)
- [ ] `gGameTime::CallEFun_GetYearLength` · 00543830 — needs port: game time efuns (time of day, day/year length, date strings, dilation)
- [ ] `gGameTime::CallEFun_InqCurrentGameDateTimeString` · 005438a0 — needs port: game time efuns (time of day, day/year length, date strings, dilation)
- [ ] `gGameTime::CallEFun_InqGameDateTimeString` · 005438f0 — needs port: game time efuns (time of day, day/year length, date strings, dilation)
- [ ] `gGameTime::CallEFun_IsNight` · 00543960 — needs port: game time efuns (time of day, day/year length, date strings, dilation)
- [ ] `gGameTime::CallEFun_TemporalDilation` · 005439b0 — needs port: game time efuns (time of day, day/year length, date strings, dilation)

## `portal/shared/src/interp/glue/gheading.cpp.c` (4/5)

### gHeading
- [x] `gHeading::CallEFun_set_degrees` · 005433d0 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [x] `gHeading::CallEFun_get_degrees` · 00543430 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [x] `gHeading::CallEFun_get_degrees_0_centered` · 005434a0 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [x] `gHeading::CallEFun_is_in_range` · 00543510 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [ ] `gHeading::CallEFun_ToWPString` · 005435b0 · legions 005233a0 — needs port: heading efuns (set/get degrees, range check, ToWPString; position.rs dispatches only gPosition)

## `portal/shared/src/interp/glue/gicondesc.cpp.c` (0/2)

### gIconDesc
- [ ] `gIconDesc::CallEFun_Empty` · 005432f0 — needs port: icon description Empty/IsValid
- [ ] `gIconDesc::CallEFun_IsValid` · 00543360 — needs port: icon description Empty/IsValid

## `portal/shared/src/interp/glue/gifuntestsystem.cpp.c` (0/2)

### gIFunTestSystem
- [-] `gIFunTestSystem::CallIFun_GetFoo` · 00567e60 · legions 00537df0 — admin only: test system IFun (admin test harness)
- [-] `gIFunTestSystem::CallIFun_SetFoo` · 00567f50 · legions 00537ee0 — admin only: test system IFun (admin test harness)

## `portal/shared/src/interp/glue/gifunvirtualtestsystem.cpp.c` (0/1)

### gIFunVirtualTestSystem
- [-] `gIFunVirtualTestSystem::CallIFun_VirtualIFun` · 00567dd0 · legions 00537d60 — admin only: test system IFun (admin test harness)

## `portal/shared/src/interp/glue/gipreproc.cpp.c` (0/2)

### _STL::_Rb_tree<QualifiedDataID,QualifiedDataID,_STL::_Identity<QualifiedDataID>,_STL::less<QualifiedDataID>,_STL::allocator<QualifiedDataID>_>
- [-] `_STL::_Rb_tree<QualifiedDataID,QualifiedDataID,_STL::_Identity<QualifiedDataID>,_STL::less<QualifiedDataID>,_STL::allocator<QualifiedDataID>_>::operator=` · 007337b0 · legions 006afc60 — C++ assignment operator of plain data (Rust copy/move)

### gIPreproc
- [-] `gIPreproc::CallIFun_GetSubDataIDs` · 00733870 · legions 006afd20 — admin only: script preprocessor (compiler), the client runs compiled wlib

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
- [-] `HashSet<unsigned___int64>::add` · 00551930 · legions 0052ccf0 — container internals (Rust collections replace them)

### gLAHashSet
- [x] `gLAHashSet::CallEFun_add` · 005519a0 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

### HashSet<unsigned___int64>
- [-] `HashSet<unsigned___int64>::grow` · 00551a30 · legions 0052cdd0 — container internals (Rust collections replace them)
- [-] `HashSet<unsigned___int64>::shrink` · 00551a80 · legions 0052ce20 — container internals (Rust collections replace them)

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
- [-] `_STL::_Rb_tree_iterator<_STL::pair<ActionWithModifiers_const_,ActionMapValue>,_STL::_Nonconst_traits<_STL::pair<ActionWithModifiers_const_,ActionMapValue>_>_>::operator--` · 0054d8e0 · legions 0060f180 — MSVC STL internals (Rust collections replace them)

### gLAMultiMapIter
- [x] `gLAMultiMapIter::CallEFun_data` · 0054d930 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLAMultiMapIter::CallEFun_equals` · 0054d9a0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLAMultiMapIter::CallEFun_key` · 0054da30 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLAMultiMapIter::CallEFun_next` · 0054daa0 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)
- [x] `gLAMultiMapIter::CallEFun_prev` · 0054db40 — ac2-script glue::multi over native::multi (STLport hash_multimap / multimap model)

## `portal/shared/src/interp/glue/glanddefs.cpp.c` (0/3)

### gLandDefs
- [ ] `gLandDefs::CallEFun_get_land_length` · 00543220 — needs port: landblock size and static-object lookup efuns
- [ ] `gLandDefs::CallEFun_get_land_width` · 00543250 — needs port: landblock size and static-object lookup efuns
- [ ] `gLandDefs::CallEFun_InqLandblockIIDFromStaticObjectIID` · 00543280 — needs port: landblock size and static-object lookup efuns

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
- [-] `SmartArray<CLandBlock::DynamicObjectInfo>::AddToEnd` · 0055e690 · legions 00533650 — container internals (Rust collections replace them)

### gLArray
- [x] `gLArray::CallEFun_push` · 0055e6e0 — ac2-script glue::lists
- [x] `gLArray::CallEFun_push_unique` · 0055e770 — ac2-script glue::lists
- [x] `gLArray::CallEFun_pop` · 0055e840 — ac2-script glue::lists
- [x] `gLArray::CallEFun_query` · 0055e910 — ac2-script glue::lists

## `portal/shared/src/interp/glue/glinkrequirements.cpp.c` (0/10)

### gLinkRequirements
- [ ] `gLinkRequirements::CallEFun_SetInputLink` · 00542e10 — needs port: link requirement setters and getters
- [ ] `gLinkRequirements::CallEFun_SetLinkType` · 00542e70 — needs port: link requirement setters and getters
- [ ] `gLinkRequirements::CallEFun_SetNumLinks` · 00542ed0 — needs port: link requirement setters and getters
- [ ] `gLinkRequirements::CallEFun_SetNumReqLinks` · 00542f30 — needs port: link requirement setters and getters
- [ ] `gLinkRequirements::CallEFun_SetRequired` · 00542f90 — needs port: link requirement setters and getters
- [ ] `gLinkRequirements::CallEFun_GetLinkType` · 00542ff0 — needs port: link requirement setters and getters
- [ ] `gLinkRequirements::CallEFun_GetNumLinks` · 00543060 — needs port: link requirement setters and getters
- [ ] `gLinkRequirements::CallEFun_GetNumReqLinks` · 005430d0 — needs port: link requirement setters and getters
- [ ] `gLinkRequirements::CallEFun_IsInputLink` · 00543140 — needs port: link requirement setters and getters
- [ ] `gLinkRequirements::CallEFun_IsRequired` · 005431b0 — needs port: link requirement setters and getters

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
- [-] `IntrusiveHashIterator<unsigned___int64,HashSetData<unsigned___int64>_*>::remove` · 0055d460 · legions 00531370 — container internals (Rust collections replace them)

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
- [-] `BaseList<unsigned___int64,60>::insert_sort_ulint` · 0055b810 · legions 00532260 — container internals (Rust collections replace them)

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
- [-] `BaseList<unsigned___int64,60>::inq_data` · 0055be10 · legions 00532510 — container internals (Rust collections replace them)

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
- [-] `BaseList<unsigned___int64,60>::get_begin` · 0055c7c0 · legions 005328f0 — container internals (Rust collections replace them)
- [-] `BaseList<unsigned___int64,60>::get_reverse` · 0055c810 · legions 00532940 — container internals (Rust collections replace them)
- [-] `BaseList<unsigned___int64,60>::get_end` · 0055c860 · legions 00532990 — container internals (Rust collections replace them)

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
- [ ] `gLOSParameters::CallEFun_AddToEtherealList` · 00542680 — needs port: line-of-sight parameter setters and results (ray, portals, exact draw check)
- [ ] `gLOSParameters::CallEFun_SetBlockedByWater` · 00542700 — needs port: line-of-sight parameter setters and results (ray, portals, exact draw check)
- [ ] `gLOSParameters::CallEFun_SetCheckEnvOnly` · 00542760 — needs port: line-of-sight parameter setters and results (ray, portals, exact draw check)
- [ ] `gLOSParameters::CallEFun_SetCheckPortalsOnly` · 005427c0 — needs port: line-of-sight parameter setters and results (ray, portals, exact draw check)
- [ ] `gLOSParameters::CallEFun_SetCheckUnconnectedPortalsOnly` · 00542820 — needs port: line-of-sight parameter setters and results (ray, portals, exact draw check)
- [ ] `gLOSParameters::CallEFun_SetExactDrawingCheck` · 00542880 — needs port: line-of-sight parameter setters and results (ray, portals, exact draw check)
- [ ] `gLOSParameters::CallEFun_SetFromBeginAndEndPosition` · 005428e0 — needs port: line-of-sight parameter setters and results (ray, portals, exact draw check)
- [ ] `gLOSParameters::CallEFun_SetFromPositionAndOffset` · 00542950 — needs port: line-of-sight parameter setters and results (ray, portals, exact draw check)
- [ ] `gLOSParameters::CallEFun_SetLOSParams` · 005429e0 — needs port: line-of-sight parameter setters and results (ray, portals, exact draw check)
- [ ] `gLOSParameters::CallEFun_SetPhysicsCheck` · 00542a50 — needs port: line-of-sight parameter setters and results (ray, portals, exact draw check)
- [ ] `gLOSParameters::CallEFun_SetRay` · 00542ab0 — needs port: line-of-sight parameter setters and results (ray, portals, exact draw check)
- [ ] `gLOSParameters::CallEFun_GetFinalCellID` · 00542b20 — needs port: line-of-sight parameter setters and results (ray, portals, exact draw check)
- [ ] `gLOSParameters::CallEFun_HitUnloadedBlock` · 00542b90 — needs port: line-of-sight parameter setters and results (ray, portals, exact draw check)
- [ ] `gLOSParameters::CallEFun_RayUnblocked` · 00542c10 — needs port: line-of-sight parameter setters and results (ray, portals, exact draw check)
- [ ] `gLOSParameters::CallEFun_GetEndPoint` · 00542c90 — needs port: line-of-sight parameter setters and results (ray, portals, exact draw check)
- [ ] `gLOSParameters::CallEFun_GetFinalRay` · 00542d30 — needs port: line-of-sight parameter setters and results (ray, portals, exact draw check)

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
- [ ] `gMissileInfo::CallEFun_SetClone` · 00542290 — needs port: missile info setters (clone, collision, targeting, holding location)
- [ ] `gMissileInfo::CallEFun_SetCollisionParameters` · 005422f0 — needs port: missile info setters (clone, collision, targeting, holding location)
- [ ] `gMissileInfo::CallEFun_SetEntityDID` · 00542350 — needs port: missile info setters (clone, collision, targeting, holding location)
- [ ] `gMissileInfo::CallEFun_SetHoldingLocation` · 005423b0 — needs port: missile info setters (clone, collision, targeting, holding location)
- [ ] `gMissileInfo::CallEFun_SetMissileParameters` · 00542410 — needs port: missile info setters (clone, collision, targeting, holding location)
- [ ] `gMissileInfo::CallEFun_SetTargetingParameters` · 00542470 — needs port: missile info setters (clone, collision, targeting, holding location)
- [ ] `gMissileInfo::CallEFun_SetThrown` · 005424d0 — needs port: missile info setters (clone, collision, targeting, holding location)

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
- [ ] `gMissileTargetingParameters::CallEFun_ClientUseParentForBasePosition` · 005414d0 — needs port: missile targeting parameter setters (speed, pitch, target, start toward target)
- [ ] `gMissileTargetingParameters::CallEFun_SetAlwaysStartTowardTarget` · 00541530 — needs port: missile targeting parameter setters (speed, pitch, target, start toward target)
- [ ] `gMissileTargetingParameters::CallEFun_SetBasePosition` · 00541590 — needs port: missile targeting parameter setters (speed, pitch, target, start toward target)
- [ ] `gMissileTargetingParameters::CallEFun_SetInitialPitch` · 005415f0 — needs port: missile targeting parameter setters (speed, pitch, target, start toward target)
- [ ] `gMissileTargetingParameters::CallEFun_SetInitialSpeed` · 00541650 — needs port: missile targeting parameter setters (speed, pitch, target, start toward target)
- [ ] `gMissileTargetingParameters::CallEFun_SetMaxInitialTargetRange` · 005416b0 — needs port: missile targeting parameter setters (speed, pitch, target, start toward target)
- [ ] `gMissileTargetingParameters::CallEFun_SetMissileDetonationChance` · 00541710 — needs port: missile targeting parameter setters (speed, pitch, target, start toward target)
- [ ] `gMissileTargetingParameters::CallEFun_SetNoFireIfTooFar` · 00541770 — needs port: missile targeting parameter setters (speed, pitch, target, start toward target)
- [ ] `gMissileTargetingParameters::CallEFun_SetStartTowardTarget` · 005417d0 — needs port: missile targeting parameter setters (speed, pitch, target, start toward target)
- [ ] `gMissileTargetingParameters::CallEFun_SetTargetID` · 00541830 — needs port: missile targeting parameter setters (speed, pitch, target, start toward target)
- [ ] `gMissileTargetingParameters::CallEFun_SetTargetPos` · 005418b0 — needs port: missile targeting parameter setters (speed, pitch, target, start toward target)
- [ ] `gMissileTargetingParameters::CallEFun_UseSourceObjectForBasePosition` · 00541910 — needs port: missile targeting parameter setters (speed, pitch, target, start toward target)

## `portal/shared/src/interp/glue/gmotionvalues.cpp.c` (0/22)

### gMotionValues
- [ ] `gMotionValues::CallEFun_GetBackwardAcl` · 00540b30 — needs port: motion value getters (acceleration, velocity, turn, gravity, sinking)
- [ ] `gMotionValues::CallEFun_GetBackwardSB` · 00540ba0 — needs port: motion value getters (acceleration, velocity, turn, gravity, sinking)
- [ ] `gMotionValues::CallEFun_GetDrag` · 00540c10 — needs port: motion value getters (acceleration, velocity, turn, gravity, sinking)
- [ ] `gMotionValues::CallEFun_GetForwardAcl` · 00540c80 — needs port: motion value getters (acceleration, velocity, turn, gravity, sinking)
- [ ] `gMotionValues::CallEFun_GetForwardSB` · 00540cf0 — needs port: motion value getters (acceleration, velocity, turn, gravity, sinking)
- [ ] `gMotionValues::CallEFun_GetGravity` · 00540d60 — needs port: motion value getters (acceleration, velocity, turn, gravity, sinking)
- [ ] `gMotionValues::CallEFun_GetHoverHeight` · 00540dd0 — needs port: motion value getters (acceleration, velocity, turn, gravity, sinking)
- [ ] `gMotionValues::CallEFun_GetJumpImpulse` · 00540e40 — needs port: motion value getters (acceleration, velocity, turn, gravity, sinking)
- [ ] `gMotionValues::CallEFun_GetLift` · 00540eb0 — needs port: motion value getters (acceleration, velocity, turn, gravity, sinking)
- [ ] `gMotionValues::CallEFun_GetMaxBackwardVel` · 00540f20 — needs port: motion value getters (acceleration, velocity, turn, gravity, sinking)
- [ ] `gMotionValues::CallEFun_GetMaxForwardVel` · 00540f90 — needs port: motion value getters (acceleration, velocity, turn, gravity, sinking)
- [ ] `gMotionValues::CallEFun_GetMaxStrafeVel` · 00541000 — needs port: motion value getters (acceleration, velocity, turn, gravity, sinking)
- [ ] `gMotionValues::CallEFun_GetMaxVertVel` · 00541070 — needs port: motion value getters (acceleration, velocity, turn, gravity, sinking)
- [ ] `gMotionValues::CallEFun_GetRunningTurnVel` · 005410e0 — needs port: motion value getters (acceleration, velocity, turn, gravity, sinking)
- [ ] `gMotionValues::CallEFun_GetSinkOffset` · 00541150 — needs port: motion value getters (acceleration, velocity, turn, gravity, sinking)
- [ ] `gMotionValues::CallEFun_GetStillTurnVel` · 005411c0 — needs port: motion value getters (acceleration, velocity, turn, gravity, sinking)
- [ ] `gMotionValues::CallEFun_GetStrafeAcl` · 00541230 — needs port: motion value getters (acceleration, velocity, turn, gravity, sinking)
- [ ] `gMotionValues::CallEFun_GetStrafeSB` · 005412a0 — needs port: motion value getters (acceleration, velocity, turn, gravity, sinking)
- [ ] `gMotionValues::CallEFun_GetTerminalVel` · 00541310 — needs port: motion value getters (acceleration, velocity, turn, gravity, sinking)
- [ ] `gMotionValues::CallEFun_GetTurnAcl` · 00541380 — needs port: motion value getters (acceleration, velocity, turn, gravity, sinking)
- [ ] `gMotionValues::CallEFun_GetTurnSB` · 005413f0 — needs port: motion value getters (acceleration, velocity, turn, gravity, sinking)
- [ ] `gMotionValues::CallEFun_GetVertAcl` · 00541460 — needs port: motion value getters (acceleration, velocity, turn, gravity, sinking)

## `portal/shared/src/interp/glue/gmovementparameters.cpp.c` (0/21)

### gMovementParameters
- [ ] `gMovementParameters::CallEFun_set_callback_when_in_range` · 005402a0 — needs port: movement parameter setters (target, distances, flags, position list)
- [ ] `gMovementParameters::CallEFun_set_context_id` · 00540300 — needs port: movement parameter setters (target, distances, flags, position list)
- [ ] `gMovementParameters::CallEFun_set_desired_distance` · 00540360 — needs port: movement parameter setters (target, distances, flags, position list)
- [ ] `gMovementParameters::CallEFun_set_desired_velocity` · 005403c0 — needs port: movement parameter setters (target, distances, flags, position list)
- [ ] `gMovementParameters::CallEFun_set_fail_distance` · 00540420 — needs port: movement parameter setters (target, distances, flags, position list)
- [ ] `gMovementParameters::CallEFun_set_max_success_distance` · 00540480 — needs port: movement parameter setters (target, distances, flags, position list)
- [ ] `gMovementParameters::CallEFun_set_min_success_distance` · 005404e0 — needs port: movement parameter setters (target, distances, flags, position list)
- [ ] `gMovementParameters::CallEFun_set_move_away_flag` · 00540540 — needs port: movement parameter setters (target, distances, flags, position list)
- [ ] `gMovementParameters::CallEFun_set_move_towards_flag` · 005405a0 — needs port: movement parameter setters (target, distances, flags, position list)
- [ ] `gMovementParameters::CallEFun_set_object_flag` · 00540600 — needs port: movement parameter setters (target, distances, flags, position list)
- [ ] `gMovementParameters::CallEFun_set_offset_vector` · 00540660 — needs port: movement parameter setters (target, distances, flags, position list)
- [ ] `gMovementParameters::CallEFun_set_persistent_fail_exits_flag` · 005406c0 — needs port: movement parameter setters (target, distances, flags, position list)
- [ ] `gMovementParameters::CallEFun_set_persistent_flag` · 00540720 — needs port: movement parameter setters (target, distances, flags, position list)
- [ ] `gMovementParameters::CallEFun_set_target_heading` · 00540780 — needs port: movement parameter setters (target, distances, flags, position list)
- [ ] `gMovementParameters::CallEFun_set_target_object_id` · 005407f0 — needs port: movement parameter setters (target, distances, flags, position list)
- [ ] `gMovementParameters::CallEFun_set_target_position` · 00540850 — needs port: movement parameter setters (target, distances, flags, position list)
- [ ] `gMovementParameters::CallEFun_set_turn_flag` · 005408b0 — needs port: movement parameter setters (target, distances, flags, position list)
- [ ] `gMovementParameters::CallEFun_set_use_cylinders` · 00540910 — needs port: movement parameter setters (target, distances, flags, position list)
- [ ] `gMovementParameters::CallEFun_get_context_id` · 00540970 — needs port: movement parameter setters (target, distances, flags, position list)
- [ ] `gMovementParameters::CallEFun_set_position_list` · 005409e0 — needs port: movement parameter setters (target, distances, flags, position list)
- [ ] `gMovementParameters::CallEFun_pop_position_list` · 00540aa0 — needs port: movement parameter setters (target, distances, flags, position list)

## `portal/shared/src/interp/glue/gmovementreturn.cpp.c` (0/8)

### gMovementReturn
- [ ] `gMovementReturn::CallEFun_GetError` · 0053fe10 — needs port: movement return getters (status, error, object id, position)
- [ ] `gMovementReturn::CallEFun_GetObjectHeight` · 0053fe80 — needs port: movement return getters (status, error, object id, position)
- [ ] `gMovementReturn::CallEFun_GetObjectID` · 0053fef0 — needs port: movement return getters (status, error, object id, position)
- [ ] `gMovementReturn::CallEFun_GetObjectRadius` · 0053ff80 — needs port: movement return getters (status, error, object id, position)
- [ ] `gMovementReturn::CallEFun_GetPersistentFlag` · 0053fff0 — needs port: movement return getters (status, error, object id, position)
- [ ] `gMovementReturn::CallEFun_GetStatus` · 00540060 — needs port: movement return getters (status, error, object id, position)
- [ ] `gMovementReturn::CallEFun_GetMovementParameters` · 005400d0 — needs port: movement return getters (status, error, object id, position)
- [ ] `gMovementReturn::CallEFun_GetPosition` · 005401a0 — needs port: movement return getters (status, error, object id, position)

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
- [-] `HashIterator<ReferenceIDWrapper,unsigned_long>::next` · 00556af0 · legions 0052f8e0 — container internals (Rust collections replace them)

### gNRHashIter
- [x] `gNRHashIter::CallEFun_next` · 00556b30 — ac2-script glue::hashes over native::ScriptHash (IntrusiveHashTable bucket model, g_bucketSizes)

## `portal/shared/src/interp/glue/gobjcollisionprofile.cpp.c` (0/4)

### gObjCollisionProfile
- [ ] `gObjCollisionProfile::CallEFun_GetID` · 0053fc50 — needs port: object collision profile inquiries (ID, begin/end collision, ethereal)
- [ ] `gObjCollisionProfile::CallEFun_IsBeginCollision` · 0053fcc0 — needs port: object collision profile inquiries (ID, begin/end collision, ethereal)
- [ ] `gObjCollisionProfile::CallEFun_IsEndCollision` · 0053fd30 — needs port: object collision profile inquiries (ID, begin/end collision, ethereal)
- [ ] `gObjCollisionProfile::CallEFun_IsEthereal` · 0053fda0 — needs port: object collision profile inquiries (ID, begin/end collision, ethereal)

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
- [ ] `gPath::CallEFun_set_dist` · 0053efd0 — needs port: path efuns (dist/ratio/time get/set, length, unsolved, ToString, position list)
- [ ] `gPath::CallEFun_set_ratio` · 0053f030 — needs port: path efuns (dist/ratio/time get/set, length, unsolved, ToString, position list)
- [ ] `gPath::CallEFun_set_time` · 0053f090 — needs port: path efuns (dist/ratio/time get/set, length, unsolved, ToString, position list)
- [ ] `gPath::CallEFun_get_dist` · 0053f0f0 — needs port: path efuns (dist/ratio/time get/set, length, unsolved, ToString, position list)
- [ ] `gPath::CallEFun_get_ratio` · 0053f160 — needs port: path efuns (dist/ratio/time get/set, length, unsolved, ToString, position list)
- [ ] `gPath::CallEFun_get_time` · 0053f1d0 — needs port: path efuns (dist/ratio/time get/set, length, unsolved, ToString, position list)
- [ ] `gPath::CallEFun_length` · 0053f250 — needs port: path efuns (dist/ratio/time get/set, length, unsolved, ToString, position list)
- [ ] `gPath::CallEFun_unsolved` · 0053f2c0 — needs port: path efuns (dist/ratio/time get/set, length, unsolved, ToString, position list)

### gMovementParameters
- [ ] `gMovementParameters::RListToPositionList` · 0053f340 · legions 00521800 — needs port: path position list conversion (RListToPositionList)

### gPath
- [ ] `gPath::CallEFun_ToStringW` · 0053f410 — needs port: path efuns (dist/ratio/time get/set, length, unsolved, ToString, position list)
- [ ] `gPath::PositionListToRList` · 0053f4d0 · legions 00520a00 — needs port: path position list conversion (PositionListToRList)
- [ ] `gPath::CallEFun_TranslateToPositionList` · 0053f590 — needs port: path efuns (dist/ratio/time get/set, length, unsolved, ToString, position list)

## `portal/shared/src/interp/glue/gpathfinder.cpp.c` (0/14)

### gPathFinder
- [ ] `gPathFinder::CallEFun_AddRequiredProperties` · 0053e9b0 — needs port: path finder request setters (start, end, solution and property types) and InqSolution
- [ ] `gPathFinder::CallEFun_AddSolutionTypes` · 0053ea10 — needs port: path finder request setters (start, end, solution and property types) and InqSolution
- [ ] `gPathFinder::CallEFun_AddTerminateSolutionTypes` · 0053ea70 — needs port: path finder request setters (start, end, solution and property types) and InqSolution
- [ ] `gPathFinder::CallEFun_HasRequiredProperties` · 0053ead0 — needs port: path finder request setters (start, end, solution and property types) and InqSolution
- [ ] `gPathFinder::CallEFun_SetRequiredProperties` · 0053eb10 — needs port: path finder request setters (start, end, solution and property types) and InqSolution
- [ ] `gPathFinder::CallEFun_SetSolutionTypes` · 0053eb70 — needs port: path finder request setters (start, end, solution and property types) and InqSolution
- [ ] `gPathFinder::CallEFun_SetTerminateSolutionTypes` · 0053ebd0 — needs port: path finder request setters (start, end, solution and property types) and InqSolution
- [ ] `gPathFinder::CallEFun_HasSolutionType` · 0053ec30 — needs port: path finder request setters (start, end, solution and property types) and InqSolution
- [ ] `gPathFinder::CallEFun_HasTerminateSolutionType` · 0053ecb0 — needs port: path finder request setters (start, end, solution and property types) and InqSolution
- [ ] `gPathFinder::CallEFun_SetEnd` · 0053ed30 — needs port: path finder request setters (start, end, solution and property types) and InqSolution
- [ ] `gPathFinder::CallEFun_SetEndPosition` · 0053edb0 — needs port: path finder request setters (start, end, solution and property types) and InqSolution
- [ ] `gPathFinder::CallEFun_SetStart` · 0053ee30 — needs port: path finder request setters (start, end, solution and property types) and InqSolution
- [ ] `gPathFinder::CallEFun_SetStartPosition` · 0053eeb0 — needs port: path finder request setters (start, end, solution and property types) and InqSolution
- [ ] `gPathFinder::CallEFun_InqSolution` · 0053ef30 — needs port: path finder request setters (start, end, solution and property types) and InqSolution

## `portal/shared/src/interp/glue/gpathfinder_normal.cpp.c` (0/10)

### gPathFinder_Normal
- [ ] `gPathFinder_Normal::CallEFun_SetHeight` · 0053e5e0 — needs port: normal path finder constraint setters (height, distance, slope, solid, water, width)
- [ ] `gPathFinder_Normal::CallEFun_SetMaxDistance` · 0053e640 — needs port: normal path finder constraint setters (height, distance, slope, solid, water, width)
- [ ] `gPathFinder_Normal::CallEFun_SetMinDistance` · 0053e6a0 — needs port: normal path finder constraint setters (height, distance, slope, solid, water, width)
- [ ] `gPathFinder_Normal::CallEFun_SetMoveAway` · 0053e700 — needs port: normal path finder constraint setters (height, distance, slope, solid, water, width)
- [ ] `gPathFinder_Normal::CallEFun_SetRandomComponent` · 0053e760 — needs port: normal path finder constraint setters (height, distance, slope, solid, water, width)
- [ ] `gPathFinder_Normal::CallEFun_SetSameBlockConstraint` · 0053e7c0 — needs port: normal path finder constraint setters (height, distance, slope, solid, water, width)
- [ ] `gPathFinder_Normal::CallEFun_SetSlopeConstraint` · 0053e830 — needs port: normal path finder constraint setters (height, distance, slope, solid, water, width)
- [ ] `gPathFinder_Normal::CallEFun_SetSolidConstraint` · 0053e890 — needs port: normal path finder constraint setters (height, distance, slope, solid, water, width)
- [ ] `gPathFinder_Normal::CallEFun_SetWaterConstraint` · 0053e8f0 — needs port: normal path finder constraint setters (height, distance, slope, solid, water, width)
- [ ] `gPathFinder_Normal::CallEFun_SetWidth` · 0053e950 — needs port: normal path finder constraint setters (height, distance, slope, solid, water, width)

## `portal/shared/src/interp/glue/gpathnodedata.cpp.c` (0/3)

### gPathNodeData
- [ ] `gPathNodeData::CallEFun_GetSlopeConstraint` · 0053e490 — needs port: path node data inquiries (slope constraint, solid, water)
- [ ] `gPathNodeData::CallEFun_IsSolid` · 0053e500 — needs port: path node data inquiries (slope constraint, solid, water)
- [ ] `gPathNodeData::CallEFun_IsWater` · 0053e570 — needs port: path node data inquiries (slope constraint, solid, water)

## `portal/shared/src/interp/glue/gpathsystem.cpp.c` (0/13)

### PathSystem
- [ ] `PathSystem::QueryPosition_Internal` · 0053de90 · legions 0051fb50 — needs port: path system position query (QueryPosition_Internal)

### gPathSystem
- [ ] `gPathSystem::CallEFun_GetPathMapID` · 0053df10 — needs port: path system efuns (FindPath, QueryPosition, node ids and path node data lookups)
- [ ] `gPathSystem::CallEFun_GetPathNodeID` · 0053df50 — needs port: path system efuns (FindPath, QueryPosition, node ids and path node data lookups)

### PathSystem
- [-] `PathSystem::DumpPathMapW` · 0053dfb0 · legions 0051fc70 — WSL debugger query or dump (process, reference and perf tables)
- [-] `PathSystem::DumpSearchStatsW` · 0053e070 · legions 0051fd30 — WSL debugger query or dump (process, reference and perf tables)
- [-] `PathSystem::DumpCacheW` · 0053e130 · legions 0051fdf0 — WSL debugger query or dump (process, reference and perf tables)

### gPathSystem
- [-] `gPathSystem::CallEFun_DumpCacheW` · 0053e220 — admin dump of path map and caches
- [-] `gPathSystem::CallEFun_DumpPathMapW` · 0053e260 — admin dump of path map and caches
- [-] `gPathSystem::CallEFun_DumpSearchStatsW` · 0053e2c0 — admin dump of path map and caches
- [ ] `gPathSystem::CallEFun_FindPath` · 0053e300 — needs port: path system efuns (FindPath, QueryPosition, node ids and path node data lookups)
- [ ] `gPathSystem::CallEFun_InqPathNodeDataByNodeID` · 0053e360 — needs port: path system efuns (FindPath, QueryPosition, node ids and path node data lookups)
- [ ] `gPathSystem::CallEFun_InqPathNodeDataByPosition` · 0053e3c0 — needs port: path system efuns (FindPath, QueryPosition, node ids and path node data lookups)
- [ ] `gPathSystem::CallEFun_QueryPosition` · 0053e430 — needs port: path system efuns (FindPath, QueryPosition, node ids and path node data lookups)

## `portal/shared/src/interp/glue/gperfmon.cpp.c` (0/9)

### gPerfMon
- [-] `gPerfMon::CallEFun_AdjustCounter16` · 0053dd50 — admin only: profiler counters (admin tool)
- [-] `gPerfMon::CallEFun_AdjustCounter32` · 0053dd70 — admin only: profiler counters (admin tool)
- [-] `gPerfMon::CallEFun_AdjustCounter64` · 0053dd90 — admin only: profiler counters (admin tool)
- [-] `gPerfMon::CallEFun_AdjustCounter8` · 0053ddc0 — admin only: profiler counters (admin tool)
- [-] `gPerfMon::CallEFun_DecrementCounter` · 0053dde0 — admin only: profiler counters (admin tool)
- [-] `gPerfMon::CallEFun_IncrementCounter` · 0053de00 — admin only: profiler counters (admin tool)
- [-] `gPerfMon::CallEFun_StartTimerCounter` · 0053de20 — admin only: profiler counters (admin tool)
- [-] `gPerfMon::CallEFun_StopTimerCounter` · 0053de40 — admin only: profiler counters (admin tool)
- [-] `gPerfMon::CallEFun_MeasureLock` · 0053de60 — admin only: profiler counters (admin tool)

## `portal/shared/src/interp/glue/gpfilenode.cpp.c` (0/39)

### FileNodeName_UInt64
- [ ] `FileNodeName_UInt64::GetType` · 00543d60 · legions 005238d0 — needs port: PFile node GetType (PFile tree names and leaves)
- [ ] `FileNodeName_UInt64::Clone` · 00543d70 · legions 005238e0 — needs port: PFile node Clone (PFile tree names and leaves)

### FileNodeName_Int64
- [ ] `FileNodeName_Int64::Clone` · 00543da0 · legions 00523910 — needs port: PFile node Clone (PFile tree names and leaves)

### PFileNode
- [ ] `PFileNode::SetNameAsBool` · 00543dd0 · legions 00523940 — needs port: PFile node SetNameAsBool (PFile tree names and leaves)

### gPFileNode
- [ ] `gPFileNode::CallEFun_AddAndRefSubNode` · 00543e20 — needs port: PFile tree node efuns (name and leaf get/set, sub nodes, keyword test)
- [ ] `gPFileNode::CallEFun_ReportErrorWSL` · 00543e80 — needs port: PFile tree node efuns (name and leaf get/set, sub nodes, keyword test)
- [ ] `gPFileNode::CallEFun_ReportWarningWSL` · 00543ee0 — needs port: PFile tree node efuns (name and leaf get/set, sub nodes, keyword test)
- [ ] `gPFileNode::CallEFun_SetNameAsBool` · 00543f40 — needs port: PFile tree node efuns (name and leaf get/set, sub nodes, keyword test)
- [ ] `gPFileNode::CallEFun_SetNameAsFloat` · 00543fe0 — needs port: PFile tree node efuns (name and leaf get/set, sub nodes, keyword test)
- [ ] `gPFileNode::CallEFun_SetNameAsInt32` · 00544080 — needs port: PFile tree node efuns (name and leaf get/set, sub nodes, keyword test)
- [ ] `gPFileNode::CallEFun_SetNameAsUInt32` · 00544120 — needs port: PFile tree node efuns (name and leaf get/set, sub nodes, keyword test)
- [ ] `gPFileNode::CallEFun_SetNameAsWPString` · 005441c0 — needs port: PFile tree node efuns (name and leaf get/set, sub nodes, keyword test)

### PFileNode
- [ ] `PFileNode::GetLeafAsInt32` · 00544220 · legions 00523c50 — needs port: PFile node GetLeafAsInt32 (PFile tree names and leaves)
- [ ] `PFileNode::GetNameAsInt64` · 00544240 — needs port: PFile node GetNameAsInt64 (PFile tree names and leaves)
- [ ] `PFileNode::GetNameAsUInt64` · 00544310 — needs port: PFile node GetNameAsUInt64 (PFile tree names and leaves)

### gPFileNode
- [ ] `gPFileNode::CallEFun_GetNumSubNodes` · 005443e0 — needs port: PFile tree node efuns (name and leaf get/set, sub nodes, keyword test)
- [ ] `gPFileNode::CallEFun_IsKeywordW` · 00544450 — needs port: PFile tree node efuns (name and leaf get/set, sub nodes, keyword test)
- [-] `gPFileNode::CallEFun_GetSourceFileNameW` · 005444d0 — debugger source file name
- [ ] `gPFileNode::CallEFun_GetSubNode` · 00544590 — needs port: PFile tree node efuns (name and leaf get/set, sub nodes, keyword test)
- [ ] `gPFileNode::CallEFun_MakeSubNode` · 00544630 — needs port: PFile tree node efuns (name and leaf get/set, sub nodes, keyword test)
- [ ] `gPFileNode::CallEFun_GetLeafAsBool` · 005446c0 — needs port: PFile tree node efuns (name and leaf get/set, sub nodes, keyword test)
- [ ] `gPFileNode::CallEFun_GetLeafAsFloat` · 00544780 — needs port: PFile tree node efuns (name and leaf get/set, sub nodes, keyword test)
- [ ] `gPFileNode::CallEFun_GetLeafAsInt32` · 00544840 — needs port: PFile tree node efuns (name and leaf get/set, sub nodes, keyword test)
- [ ] `gPFileNode::CallEFun_GetLeafAsInt64` · 00544900 — needs port: PFile tree node efuns (name and leaf get/set, sub nodes, keyword test)
- [ ] `gPFileNode::CallEFun_GetLeafAsUInt32` · 005449c0 — needs port: PFile tree node efuns (name and leaf get/set, sub nodes, keyword test)
- [ ] `gPFileNode::CallEFun_GetLeafAsUInt64` · 00544a80 — needs port: PFile tree node efuns (name and leaf get/set, sub nodes, keyword test)
- [ ] `gPFileNode::CallEFun_GetLeafAsWPString` · 00544b40 · legions 00524350 — needs port: PFile tree node efuns (name and leaf get/set, sub nodes, keyword test)
- [ ] `gPFileNode::CallEFun_GetNameAsBool` · 00544be0 — needs port: PFile tree node efuns (name and leaf get/set, sub nodes, keyword test)
- [ ] `gPFileNode::CallEFun_GetNameAsFloat` · 00544c80 — needs port: PFile tree node efuns (name and leaf get/set, sub nodes, keyword test)
- [ ] `gPFileNode::CallEFun_GetNameAsInt32` · 00544d20 — needs port: PFile tree node efuns (name and leaf get/set, sub nodes, keyword test)
- [ ] `gPFileNode::CallEFun_GetNameAsInt64` · 00544dc0 — needs port: PFile tree node efuns (name and leaf get/set, sub nodes, keyword test)
- [ ] `gPFileNode::CallEFun_GetNameAsUInt32` · 00544e60 — needs port: PFile tree node efuns (name and leaf get/set, sub nodes, keyword test)
- [ ] `gPFileNode::CallEFun_GetNameAsUInt64` · 00544f00 — needs port: PFile tree node efuns (name and leaf get/set, sub nodes, keyword test)
- [ ] `gPFileNode::CallEFun_GetNameAsWPString` · 00544fa0 · legions 005246c0 — needs port: PFile tree node efuns (name and leaf get/set, sub nodes, keyword test)
- [ ] `gPFileNode::CallEFun_GetSubNameAsBool` · 00545040 · legions 00524730 — needs port: PFile tree node efuns (name and leaf get/set, sub nodes, keyword test)
- [ ] `gPFileNode::CallEFun_GetSubNameAsFloat` · 00545110 · legions 005247d0 — needs port: PFile tree node efuns (name and leaf get/set, sub nodes, keyword test)
- [ ] `gPFileNode::CallEFun_GetSubNameAsInt32` · 005451e0 · legions 00524870 — needs port: PFile tree node efuns (name and leaf get/set, sub nodes, keyword test)
- [ ] `gPFileNode::CallEFun_GetSubNameAsUInt32` · 005452b0 · legions 00524910 — needs port: PFile tree node efuns (name and leaf get/set, sub nodes, keyword test)
- [ ] `gPFileNode::CallEFun_GetSubNameAsWPString` · 00545380 · legions 005249b0 — needs port: PFile tree node efuns (name and leaf get/set, sub nodes, keyword test)

## `portal/shared/src/interp/glue/gpfileparser.cpp.c` (0/2)

### gPFileParser
- [ ] `gPFileParser::CallEFun_LoadFromStringW` · 00543bd0 — needs port: PFile parser efuns (load and save string)
- [ ] `gPFileParser::CallEFun_SaveToStringW` · 00543c50 · legions 00523800 — needs port: PFile parser efuns (load and save string)

## `portal/shared/src/interp/glue/gphysicsstory.cpp.c` (0/7)

### gPhysicsStory
- [ ] `gPhysicsStory::CallEFun_AddHook` · 0053daa0 — needs port: physics story hook efuns (AddHook, attacker/skill/basic attack setters, attack result)
- [ ] `gPhysicsStory::CallEFun_SetAttackerBehavior` · 0053db00 — needs port: physics story hook efuns (AddHook, attacker/skill/basic attack setters, attack result)
- [ ] `gPhysicsStory::CallEFun_SetAttackerID` · 0053db60 — needs port: physics story hook efuns (AddHook, attacker/skill/basic attack setters, attack result)
- [ ] `gPhysicsStory::CallEFun_SetBasicAttack` · 0053dbc0 — needs port: physics story hook efuns (AddHook, attacker/skill/basic attack setters, attack result)
- [ ] `gPhysicsStory::CallEFun_SetClientAttackContext` · 0053dc20 — needs port: physics story hook efuns (AddHook, attacker/skill/basic attack setters, attack result)
- [ ] `gPhysicsStory::CallEFun_SetSkillID` · 0053dc80 — needs port: physics story hook efuns (AddHook, attacker/skill/basic attack setters, attack result)
- [ ] `gPhysicsStory::CallEFun_GetOverallAttackResult` · 0053dce0 — needs port: physics story hook efuns (AddHook, attacker/skill/basic attack setters, attack result)

## `portal/shared/src/interp/glue/gplacestable.cpp.c` (3/6)

### gPlacesTable
- [ ] `gPlacesTable::NRHashToPlacesHash` · 0053d6a0 · legions 0051f480 — needs port: places hash conversion (NRHashToPlacesHash)
- [-] `gPlacesTable::CallEFun_WslDump` · 0053d730 — WSL debug dump
- [x] `gPlacesTable::PlacesHashToNRHash` · 0053d7b0 · legions 0051f590 — ac2-script glue::places / enums / chargen (PlacesTable data::places)
- [x] `gPlacesTable::CallEFun_InqPlace` · 0053d8f0 — ac2-script glue::places / enums / chargen (PlacesTable data::places)
- [ ] `gPlacesTable::CallEFun_InqPlaceStr` · 0053d960 — needs port: places table efuns (InqPlace, InqPlaceStr, InqPlacesHash)
- [x] `gPlacesTable::CallEFun_InqPlacesHash` · 0053d9d0 — ac2-script glue::places / enums / chargen (PlacesTable data::places)

## `portal/shared/src/interp/glue/gplane.cpp.c` (0/4)

### gPlane
- [ ] `gPlane::CallEFun_InitPlane` · 0053d3f0 — needs port: plane efuns (InitPlane, distance to point, snap to plane, GetString)
- [ ] `gPlane::CallEFun_distance_to_point` · 0053d480 — needs port: plane efuns (InitPlane, distance to point, snap to plane, GetString)
- [ ] `gPlane::CallEFun_GetStringW` · 0053d510 · legions 0051f330 — needs port: plane efuns (InitPlane, distance to point, snap to plane, GetString)
- [ ] `gPlane::CallEFun_snap_to_plane` · 0053d590 · legions 0051f390 — needs port: plane efuns (InitPlane, distance to point, snap to plane, GetString)

## `portal/shared/src/interp/glue/gposition.cpp.c` (38/41)

### gPosition
- [x] `gPosition::CallEFun_Set` · 0053be40 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [x] `gPosition::CallEFun_SetCellID` · 0053beb0 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [x] `gPosition::CallEFun_add_global_offset` · 0053bf10 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [x] `gPosition::CallEFun_add_local_offset` · 0053bf90 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [x] `gPosition::CallEFun_adjust_to_outside` · 0053bff0 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [x] `gPosition::CallEFun_set_cell_id` · 0053c050 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [x] `gPosition::CallEFun_set_height` · 0053c0b0 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [x] `gPosition::CallEFun_set_origin` · 0053c110 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [x] `gPosition::CallEFun_FromCellCoords` · 0053c180 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [x] `gPosition::CallEFun_FromCellCoordsString` · 0053c200 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [~] `gPosition::CallEFun_FromRealMapCoords` · 0053c280 — ac2-script glue::position: swscanf parsing approximated / StringInfo variable layout inferred
- [~] `gPosition::CallEFun_FromWPString` · 0053c300 — ac2-script glue::position: swscanf parsing approximated / StringInfo variable layout inferred
- [x] `gPosition::CallEFun_GetBlockID` · 0053c380 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [x] `gPosition::CallEFun_GetCellDistance` · 0053c400 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [x] `gPosition::CallEFun_GetCellDistanceSquared` · 0053c440 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [x] `gPosition::CallEFun_GetCellID` · 0053c480 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [x] `gPosition::CallEFun_GetCellX` · 0053c4f0 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [x] `gPosition::CallEFun_GetCellY` · 0053c560 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [x] `gPosition::CallEFun_IsValid` · 0053c5d0 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [x] `gPosition::CallEFun_NotInWorld` · 0053c640 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [x] `gPosition::CallEFun_cylinder_distance` · 0053c6b0 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [x] `gPosition::CallEFun_determine_quadrant` · 0053c770 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [x] `gPosition::CallEFun_distance` · 0053c800 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [x] `gPosition::CallEFun_distance_squared` · 0053c880 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [x] `gPosition::CallEFun_equals` · 0053c900 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [x] `gPosition::CallEFun_get_height` · 0053c9a0 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [x] `gPosition::CallEFun_in_same_block` · 0053ca10 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [x] `gPosition::CallEFun_outside` · 0053cac0 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [x] `gPosition::CallEFun_pitch` · 0053cb40 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [x] `gPosition::CallEFun_xy_distance` · 0053cbc0 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [x] `gPosition::CallEFun_xy_distance_squared` · 0053cc40 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [x] `gPosition::CallEFun_GetLocalHeading` · 0053ccc0 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [x] `gPosition::CallEFun_GetNullPosition` · 0053cd80 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [x] `gPosition::CallEFun_ToCellCoordsString` · 0053ce30 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [x] `gPosition::CallEFun_ToRealMapCoords` · 0053cef0 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [~] `gPosition::CallEFun_ToRealMapCoords_Info` · 0053cfb0 — ac2-script glue::position: swscanf parsing approximated / StringInfo variable layout inferred
- [x] `gPosition::CallEFun_ToWPString` · 0053d060 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [x] `gPosition::CallEFun_get_block_offset` · 0053d120 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [x] `gPosition::CallEFun_get_offset` · 0053d1d0 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [x] `gPosition::CallEFun_get_origin` · 0053d280 — ac2-script glue::position over ac2-physics landdefs/Position/Heading
- [x] `gPosition::CallEFun_heading` · 0053d330 — ac2-script glue::position over ac2-physics landdefs/Position/Heading

## `portal/shared/src/interp/glue/gpropertycollection.cpp.c` (0/12)

### gPropertyCollection
- [ ] `gPropertyCollection::CallEFun_AddOptionalProperty` · 0053b790 — needs port: property collection efuns (AddProperty, group and property inquiries)
- [ ] `gPropertyCollection::CallEFun_AddProperty` · 0053b810 — needs port: property collection efuns (AddProperty, group and property inquiries)
- [ ] `gPropertyCollection::CallEFun_ContainsGroup` · 0053b890 — needs port: property collection efuns (AddProperty, group and property inquiries)
- [ ] `gPropertyCollection::CallEFun_ContainsProperty` · 0053b910 — needs port: property collection efuns (AddProperty, group and property inquiries)
- [ ] `gPropertyCollection::CallEFun_GetNumGroups` · 0053b990 — needs port: property collection efuns (AddProperty, group and property inquiries)
- [ ] `gPropertyCollection::CallEFun_UpdateProperty` · 0053ba00 — needs port: property collection efuns (AddProperty, group and property inquiries)
- [ ] `gPropertyCollection::CallEFun_InqGroup` · 0053ba80 · legions 0051e060 — needs port: property collection efuns (AddProperty, group and property inquiries)
- [ ] `gPropertyCollection::CallEFun_InqGroupAt` · 0053bb20 — needs port: property collection efuns (AddProperty, group and property inquiries)
- [ ] `gPropertyCollection::CallEFun_InqOptionalProperty` · 0053bbc0 — needs port: property collection efuns (AddProperty, group and property inquiries)
- [ ] `gPropertyCollection::CallEFun_InqOptionalPropertyW` · 0053bc60 — needs port: property collection efuns (AddProperty, group and property inquiries)
- [ ] `gPropertyCollection::CallEFun_InqProperty` · 0053bd00 — needs port: property collection efuns (AddProperty, group and property inquiries)
- [ ] `gPropertyCollection::CallEFun_InqPropertyW` · 0053bda0 — needs port: property collection efuns (AddProperty, group and property inquiries)

## `portal/shared/src/interp/glue/gpropertygroup.cpp.c` (0/14)

### gPropertyGroup
- [ ] `gPropertyGroup::CallEFun_AddOptionalProperty` · 0053b010 — needs port: property group efuns (AddProperty, group name, property inquiries)
- [ ] `gPropertyGroup::CallEFun_AddProperty` · 0053b090 — needs port: property group efuns (AddProperty, group name, property inquiries)
- [ ] `gPropertyGroup::CallEFun_ContainsOptionalProperty` · 0053b110 — needs port: property group efuns (AddProperty, group name, property inquiries)
- [ ] `gPropertyGroup::CallEFun_ContainsProperty` · 0053b190 — needs port: property group efuns (AddProperty, group name, property inquiries)
- [ ] `gPropertyGroup::CallEFun_GetNumOptionalProperties` · 0053b210 — needs port: property group efuns (AddProperty, group name, property inquiries)
- [ ] `gPropertyGroup::CallEFun_GetNumProperties` · 0053b280 — needs port: property group efuns (AddProperty, group name, property inquiries)
- [ ] `gPropertyGroup::CallEFun_RemoveProperty` · 0053b2f0 — needs port: property group efuns (AddProperty, group name, property inquiries)
- [ ] `gPropertyGroup::CallEFun_SetGroupName` · 0053b370 — needs port: property group efuns (AddProperty, group name, property inquiries)
- [ ] `gPropertyGroup::CallEFun_UpdateProperty` · 0053b3f0 — needs port: property group efuns (AddProperty, group name, property inquiries)
- [ ] `gPropertyGroup::CallEFun_InqOptionalProperty` · 0053b470 — needs port: property group efuns (AddProperty, group name, property inquiries)
- [ ] `gPropertyGroup::CallEFun_InqOptionalPropertyAt` · 0053b510 — needs port: property group efuns (AddProperty, group name, property inquiries)
- [ ] `gPropertyGroup::CallEFun_InqProperty` · 0053b5b0 — needs port: property group efuns (AddProperty, group name, property inquiries)
- [ ] `gPropertyGroup::CallEFun_InqPropertyAt` · 0053b650 — needs port: property group efuns (AddProperty, group name, property inquiries)
- [ ] `gPropertyGroup::CallEFun_InqPropertyGroupNameW` · 0053b6f0 — needs port: property group efuns (AddProperty, group name, property inquiries)

## `portal/shared/src/interp/glue/gpsutils.cpp.c` (2/13)

### PSUtils
- [x] `PSUtils::is_double` · 0053ab40 · legions 0051d4b0 — ac2-prefs/src/cnum.rs fn is_double (PSUtils::is_double)
- [x] `PSUtils::is_int32` · 0053abb0 · legions 0051d520 — ac2-prefs/src/cnum.rs fn is_int32 (PSUtils::is_int32)
- [ ] `PSUtils::is_int64` · 0053ac20 · legions 0051d590 — needs port: PSUtils::is_int64 (64-bit number parse)
- [ ] `PSUtils::is_uint64` · 0053ac90 · legions 0051d600 — needs port: PSUtils::is_uint64 (64-bit number parse)

### gPSUtils
- [ ] `gPSUtils::CallEFun_is_double` · 0053ad00 — needs port: PSUtils efuns (number checks, cwd, directory name and extension helpers)
- [ ] `gPSUtils::CallEFun_is_int32` · 0053ad50 — needs port: PSUtils efuns (number checks, cwd, directory name and extension helpers)
- [ ] `gPSUtils::CallEFun_is_int64` · 0053ada0 — needs port: PSUtils efuns (number checks, cwd, directory name and extension helpers)
- [ ] `gPSUtils::CallEFun_is_uint32` · 0053adf0 — needs port: PSUtils efuns (number checks, cwd, directory name and extension helpers)
- [ ] `gPSUtils::CallEFun_is_uint64` · 0053ae40 — needs port: PSUtils efuns (number checks, cwd, directory name and extension helpers)
- [ ] `gPSUtils::CallEFun_cleanup_directory_nameW` · 0053ae90 — needs port: PSUtils efuns (number checks, cwd, directory name and extension helpers)
- [ ] `gPSUtils::CallEFun_get_cwdW` · 0053aef0 — needs port: PSUtils efuns (number checks, cwd, directory name and extension helpers)
- [ ] `gPSUtils::CallEFun_remove_extensionW` · 0053af50 — needs port: PSUtils efuns (number checks, cwd, directory name and extension helpers)
- [ ] `gPSUtils::CallEFun_strip_directoryW` · 0053afb0 — needs port: PSUtils efuns (number checks, cwd, directory name and extension helpers)

## `portal/shared/src/interp/glue/gqualifieddataidarray.cpp.c` (1/1)

### gQualifiedDataIDArray
- [x] `gQualifiedDataIDArray::CallEFun_WslAddToEnd` · 0053aac0 — ac2-script glue::places / enums / chargen (PlacesTable data::places)

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
- [-] `SmartArray<RandomSelectionTable<unsigned_long>::ElementStruct>::AddToEnd` · 0053a610 · legions 0051d040 — container internals (Rust collections replace them)

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
- [-] `RArray::SetNElements` · 00556170 · legions 0052f2d0 — container internals (Rust collections replace them)

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
- [ ] `gRay::CallEFun_InitFromOffset` · 00539fe0 — needs port: ray efuns (setters and getters)
- [ ] `gRay::CallEFun_InitLineSegment` · 0053a050 — needs port: ray efuns (setters and getters)
- [ ] `gRay::CallEFun_normalize` · 0053a0f0 — needs port: ray efuns (setters and getters)
- [ ] `gRay::CallEFun_GetLength` · 0053a150 — needs port: ray efuns (setters and getters)
- [ ] `gRay::CallEFun_IsFinite` · 0053a1c0 — needs port: ray efuns (setters and getters)
- [ ] `gRay::CallEFun_GetDirection` · 0053a250 — needs port: ray efuns (setters and getters)
- [ ] `gRay::CallEFun_GetEndPoint` · 0053a300 — needs port: ray efuns (setters and getters)
- [ ] `gRay::CallEFun_GetStartPoint` · 0053a3d0 — needs port: ray efuns (setters and getters)

## `portal/shared/src/interp/glue/greservedids.cpp.c` (3/6)

### gReservedIDs
- [ ] `gReservedIDs::CallEFun_IsCameraID` · 00539e60 — needs port: reserved-id checks IsCameraID, IsTemporaryID, IsUIID (IIDUtils::VerifyCamera/Temporary/UI)
- [x] `gReservedIDs::CallEFun_IsDynamicID` · 00539ea0 — ac2-script: gReservedIDs over ac2-net iid
- [x] `gReservedIDs::CallEFun_IsPlayerID` · 00539ee0 — ac2-script: gReservedIDs over ac2-net iid
- [x] `gReservedIDs::CallEFun_IsStaticID` · 00539f20 — ac2-script: gReservedIDs over ac2-net iid
- [ ] `gReservedIDs::CallEFun_IsTemporaryID` · 00539f60 — needs port: reserved-id checks IsCameraID, IsTemporaryID, IsUIID (IIDUtils::VerifyCamera/Temporary/UI)
- [ ] `gReservedIDs::CallEFun_IsUIID` · 00539fa0 — needs port: reserved-id checks IsCameraID, IsTemporaryID, IsUIID (IIDUtils::VerifyCamera/Temporary/UI)

## `portal/shared/src/interp/glue/grgbacolor.cpp.c` (5/5)

### gRGBAColor
- [x] `gRGBAColor::CallEFun_Attenuate` · 00539c50 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gRGBAColor::CallEFun_Set` · 00539cc0 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gRGBAColor::CallEFun_SetAlpha` · 00539d30 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gRGBAColor::CallEFun_SetColor32` · 00539d90 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)
- [x] `gRGBAColor::CallEFun_GetColor32` · 00539df0 — ac2-script glue::ui_natives (ac2-weenie Shop/ShopSale/CraftSkill/DamageText)

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
- [-] `BaseList<unsigned_long,102>::get_begin` · 005554a0 · legions 0052ee40 — container internals (Rust collections replace them)
- [-] `BaseList<unsigned_long,102>::get_reverse` · 005554f0 · legions 0052ee90 — container internals (Rust collections replace them)
- [-] `BaseList<unsigned_long,102>::get_end` · 00555540 · legions 0052eee0 — container internals (Rust collections replace them)

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
- [ ] `gSetPositionStruct::CallEFun_AddFlags` · 00539840 — needs port: set-position struct setters and flags (scatter radius, tries, line, flags)
- [ ] `gSetPositionStruct::CallEFun_RemoveFlags` · 005398a0 — needs port: set-position struct setters and flags (scatter radius, tries, line, flags)
- [ ] `gSetPositionStruct::CallEFun_SetFlags` · 00539900 — needs port: set-position struct setters and flags (scatter radius, tries, line, flags)
- [ ] `gSetPositionStruct::CallEFun_SetLine` · 00539960 — needs port: set-position struct setters and flags (scatter radius, tries, line, flags)
- [ ] `gSetPositionStruct::CallEFun_SetLineByPos` · 005399c0 — needs port: set-position struct setters and flags (scatter radius, tries, line, flags)
- [ ] `gSetPositionStruct::CallEFun_SetNumTries` · 00539a30 — needs port: set-position struct setters and flags (scatter radius, tries, line, flags)
- [ ] `gSetPositionStruct::CallEFun_SetPosition` · 00539a90 — needs port: set-position struct setters and flags (scatter radius, tries, line, flags)
- [ ] `gSetPositionStruct::CallEFun_SetScatterRadius` · 00539af0 — needs port: set-position struct setters and flags (scatter radius, tries, line, flags)
- [ ] `gSetPositionStruct::CallEFun_GetFlags` · 00539b60 — needs port: set-position struct setters and flags (scatter radius, tries, line, flags)
- [ ] `gSetPositionStruct::CallEFun_IsFlagSet` · 00539bd0 — needs port: set-position struct setters and flags (scatter radius, tries, line, flags)

## `portal/shared/src/interp/glue/gsphere.cpp.c` (0/3)

### gSphere
- [ ] `gSphere::CallEFun_Intersects` · 005396a0 — needs port: sphere efuns (Intersects, get radius and center)
- [ ] `gSphere::CallEFun_get_radius` · 00539720 — needs port: sphere efuns (Intersects, get radius and center)
- [ ] `gSphere::CallEFun_get_center` · 00539790 — needs port: sphere efuns (Intersects, get radius and center)

## `portal/shared/src/interp/glue/gstoryhookdata.cpp.c` (0/12)

### gStoryHookData
- [ ] `gStoryHookData::CallEFun_SetAnimHookNumber` · 00539220 — needs port: story hook data setters (attack, health, PK, missiles, weapon, target)
- [ ] `gStoryHookData::CallEFun_SetAttackHookNumber` · 00539280 — needs port: story hook data setters (attack, health, PK, missiles, weapon, target)
- [ ] `gStoryHookData::CallEFun_SetAttackResult` · 005392e0 — needs port: story hook data setters (attack, health, PK, missiles, weapon, target)
- [ ] `gStoryHookData::CallEFun_SetAttackerHealthChange` · 00539340 — needs port: story hook data setters (attack, health, PK, missiles, weapon, target)
- [ ] `gStoryHookData::CallEFun_SetAttackerPKDamageChange` · 005393a0 — needs port: story hook data setters (attack, health, PK, missiles, weapon, target)
- [ ] `gStoryHookData::CallEFun_SetTargetHealthChange` · 00539400 — needs port: story hook data setters (attack, health, PK, missiles, weapon, target)
- [ ] `gStoryHookData::CallEFun_SetTargetID` · 00539460 — needs port: story hook data setters (attack, health, PK, missiles, weapon, target)
- [ ] `gStoryHookData::CallEFun_SetTargetPKDamageChange` · 005394c0 — needs port: story hook data setters (attack, health, PK, missiles, weapon, target)
- [ ] `gStoryHookData::CallEFun_SetTypeToCreateMissiles` · 00539520 — needs port: story hook data setters (attack, health, PK, missiles, weapon, target)
- [ ] `gStoryHookData::CallEFun_SetTypeToDamage` · 00539580 — needs port: story hook data setters (attack, health, PK, missiles, weapon, target)
- [ ] `gStoryHookData::CallEFun_SetTypeToFireMissile` · 005395e0 — needs port: story hook data setters (attack, health, PK, missiles, weapon, target)
- [ ] `gStoryHookData::CallEFun_SetWeaponID` · 00539640 — needs port: story hook data setters (attack, health, PK, missiles, weapon, target)

## `portal/shared/src/interp/glue/gstringinfo.cpp.c` (29/38)

### gStringInfo
- [x] `gStringInfo::CallEFun_AddVariable_Float` · 00538010 — ac2-script glue::string_info (lookups via ScriptData)
- [x] `gStringInfo::CallEFun_AddVariable_FormattedFloat` · 00538080 — ac2-script glue::string_info (lookups via ScriptData)
- [x] `gStringInfo::CallEFun_AddVariable_FormattedInt` · 005380f0 — ac2-script glue::string_info (lookups via ScriptData)
- [ ] `gStringInfo::CallEFun_AddVariable_FormattedLInt` · 00538160 — needs port (check): implemented as glue/string_info.rs but uncited
- [x] `gStringInfo::CallEFun_AddVariable_FormattedUInt` · 005381d0 — ac2-script glue::string_info (lookups via ScriptData)
- [ ] `gStringInfo::CallEFun_AddVariable_FormattedULInt` · 00538240 — needs port (check): implemented as glue/string_info.rs but uncited
- [x] `gStringInfo::CallEFun_AddVariable_Int` · 005382b0 — ac2-script glue::string_info (lookups via ScriptData)
- [ ] `gStringInfo::CallEFun_AddVariable_LInt` · 00538310 — needs port (check): implemented as glue/string_info.rs but uncited
- [x] `gStringInfo::CallEFun_AddVariable_StringIDandTableEnum` · 00538370 — ac2-script glue::string_info (lookups via ScriptData)
- [x] `gStringInfo::CallEFun_AddVariable_StringInfo` · 005383d0 — ac2-script glue::string_info (lookups via ScriptData)
- [x] `gStringInfo::CallEFun_AddVariable_UInt` · 00538440 — ac2-script glue::string_info (lookups via ScriptData)
- [ ] `gStringInfo::CallEFun_AddVariable_ULInt` · 005384a0 — needs port (check): implemented as glue/string_info.rs but uncited
- [ ] `gStringInfo::CallEFun_ApplyMetasToPlayerName` · 00538500 — needs port: StringInfo efuns (GetStringCaseInsensitive, FromFileNode, GetSubDataIDs, ApplyMetasToPlayerName)
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
- [ ] `gStringInfo::CallEFun_GetStringCaseInsensitive` · 00538ea0 — needs port: StringInfo efuns (GetStringCaseInsensitive, FromFileNode, GetSubDataIDs, ApplyMetasToPlayerName)
- [x] `gStringInfo::CallEFun_GetStringWithMetaTags` · 00538fa0 · legions 0051c0d0 — ac2-script glue::string_text (tables via ScriptData::string_table)
- [ ] `gStringInfo::CallEFun_FromFileNode` · 00539060 · legions 0051c160 — needs port: StringInfo efuns (GetStringCaseInsensitive, FromFileNode, GetSubDataIDs, ApplyMetasToPlayerName)
- [ ] `gStringInfo::CallEFun_GetSubDataIDs` · 00539100 · legions 0051c1d0 — needs port: StringInfo efuns (GetStringCaseInsensitive, FromFileNode, GetSubDataIDs, ApplyMetasToPlayerName)
- [x] `gStringInfo::CallEFun_InqString` · 00539180 — ac2-script glue::string_text (tables via ScriptData::string_table)

## `portal/shared/src/interp/glue/gstringutils.cpp.c` (2/3)

### gStringUtils
- [ ] `gStringUtils::CallEFun_ConvertWPStringToCIWPstring` · 00551ee0 — needs port: gStringUtils::ConvertWPStringToCIWPstring

### (free functions)
- [x] `IsWordInString` · 00551fa0 · legions 0052d1d0 — ac2-client friends.rs do_emote: the status's emote is the IFun with the emote StringInfo (table 0x4100001C); gStringUtils IsWordInString efun in ac2-script glue/string_utils.rs (friends_script_7160)

### gStringUtils
- [x] `gStringUtils::CallEFun_IsWordInString` · 005520c0 — ac2-client friends.rs do_emote: the status's emote is the IFun with the emote StringInfo (table 0x4100001C); gStringUtils IsWordInString efun in ac2-script glue/string_utils.rs (friends_script_7160)

## `portal/shared/src/interp/glue/gtabootableadaptor.cpp.c` (0/3)

### gTabooTableAdaptor
- [ ] `gTabooTableAdaptor::CallEFun_CheckCharactersW` · 00537ed0 — needs port: taboo table efuns (CheckCharacters, CheckCensors, FormatStringForAudience)
- [ ] `gTabooTableAdaptor::CallEFun_CheckCensorsW` · 00537f20 — needs port: taboo table efuns (CheckCharacters, CheckCensors, FormatStringForAudience)
- [ ] `gTabooTableAdaptor::CallEFun_FormatStringForAudienceW` · 00537fa0 — needs port: taboo table efuns (CheckCharacters, CheckCensors, FormatStringForAudience)

## `portal/shared/src/interp/glue/gtimer.cpp.c` (0/3)

### gTimer
- [ ] `gTimer::CallEFun_compute_time` · 00554020 — needs port: timer efuns (compute_time, get_time, get_real_time)
- [ ] `gTimer::CallEFun_get_real_time` · 00554050 — needs port: timer efuns (compute_time, get_time, get_real_time)
- [ ] `gTimer::CallEFun_get_time` · 00554080 — needs port: timer efuns (compute_time, get_time, get_real_time)

## `portal/shared/src/interp/glue/gtokenize.cpp.c` (1/1)

### gTokenize
- [x] `gTokenize::CallEFun_Tokenize` · 00537e90 — ac2-script: gTokenize (Tokenize(wchar_t*))

## `portal/shared/src/interp/glue/gtypeinfo.cpp.c` (0/3)

### gTypeInfo
- [ ] `gTypeInfo::CallEFun_get_type` · 00553f30 — needs port: type info efuns (get_type, new_type, new_validated_type)
- [ ] `gTypeInfo::CallEFun_new_type` · 00553f80 — needs port: type info efuns (get_type, new_type, new_validated_type)
- [ ] `gTypeInfo::CallEFun_new_validated_type` · 00553fd0 — needs port: type info efuns (get_type, new_type, new_validated_type)

## `portal/shared/src/interp/glue/guisavelocations.cpp.c` (0/1)

### gUISaveLocations
- [ ] `gUISaveLocations::CallEFun_Clear` · 00537e30 — needs port: UI save locations Clear

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

## `portal/shared/src/interp/glue/gvisualdesc.cpp.c` (12/13)

### gVisualDesc
- [x] `gVisualDesc::CallEFun_Clear` · 00537020 — ac2-script glue/visual_desc.rs (efun test glue/visual_desc_tests.rs)
- [x] `gVisualDesc::CallEFun_SetScale` · 00537080 — ac2-script glue/visual_desc.rs (efun test glue/visual_desc_tests.rs)
- [x] `gVisualDesc::CallEFun_AddAppInfoHashToPartGroup` · 005370f0 — ac2-script glue/visual_desc.rs (efun test glue/visual_desc_tests.rs)
- [x] `gVisualDesc::CallEFun_AddAppearanceKey` · 00537170 — ac2-script glue/visual_desc.rs (efun test glue/visual_desc_tests.rs)
- [x] `gVisualDesc::CallEFun_ApplyAdditiveVDesc` · 005371f0 — ac2-script glue/visual_desc.rs (efun test glue/visual_desc_tests.rs)
- [x] `gVisualDesc::CallEFun_ApplySubtractiveVDesc` · 00537270 — ac2-script glue/visual_desc.rs (efun test glue/visual_desc_tests.rs)
- [x] `gVisualDesc::CallEFun_Empty` · 005372f0 — ac2-script glue/visual_desc.rs (efun test glue/visual_desc_tests.rs)
- [x] `gVisualDesc::CallEFun_RemoveAppearanceKey` · 00537360 — ac2-script glue/visual_desc.rs (efun test glue/visual_desc_tests.rs)
- [x] `gVisualDesc::CallEFun_ReplaceVisualDesc` · 005373f0 — ac2-script glue/visual_desc.rs (efun test glue/visual_desc_tests.rs)
- [x] `gVisualDesc::CallEFun_GetScale` · 00537470 — ac2-script glue/visual_desc.rs (efun test glue/visual_desc_tests.rs)
- [~] `gVisualDesc::CallEFun_GetTotalIconDesc` · 00537520 · legions 0051b090 — ac2-script glue/visual_desc.rs: the parent-and-own icon layers; atom icon modifiers not modelled
- [x] `gVisualDesc::CallEFun_GetVisualDescFromDB` · 005375c0 — out args read through the SADDR local (out_native); LongInteger (0x4000000D) answers 0 and writes nothing, as EntityDesc::InqInteger's base virtual does
- [x] `gVisualDesc::CallEFun_InqAppInfoHashForPartGroup` · 00537620 · legions 0051b160 — out args read through the SADDR local (out_native); LongInteger (0x4000000D) answers 0 and writes nothing, as EntityDesc::InqInteger's base virtual does

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
- [-] `gVM::CallEFun_QueryPerfW` · 00553c90 — WSL debugger query (process, reference and perf tables)
- [-] `gVM::CallEFun_QueryProcessTableW` · 00553cf0 — WSL debugger query (process, reference and perf tables)
- [-] `gVM::CallEFun_QueryProcessW` · 00553d50 — WSL debugger query (process, reference and perf tables)
- [-] `gVM::CallEFun_QueryProcessesW` · 00553db0 — WSL debugger query (process, reference and perf tables)
- [-] `gVM::CallEFun_QueryReferenceTreeForProcessW` · 00553e10 — WSL debugger query (process, reference and perf tables)
- [-] `gVM::CallEFun_QueryReferenceTreeW` · 00553e70 — WSL debugger query (process, reference and perf tables)
- [-] `gVM::CallEFun_QueryReferencesW` · 00553ed0 — WSL debugger query (process, reference and perf tables)

## `portal/shared/src/interp/glue/gvmdata.cpp.c` (0/4)

### gVMData
- [ ] `gVMData::CallEFun_AddData` · 00536e50 — needs port: VM data attach efuns (AddData, GetData, HasData, RemoveData)
- [ ] `gVMData::CallEFun_GetData` · 00536eb0 — needs port: VM data attach efuns (AddData, GetData, HasData, RemoveData)
- [ ] `gVMData::CallEFun_HasData` · 00536f30 — needs port: VM data attach efuns (AddData, GetData, HasData, RemoveData)
- [ ] `gVMData::CallEFun_RemoveData` · 00536fa0 — needs port: VM data attach efuns (AddData, GetData, HasData, RemoveData)

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
- [ ] `gWState::CallEFun_SaveStateByType` · 00536bc0 — needs port: WState save and load efuns (SaveStateByType, GetStateByType)
- [ ] `gWState::CallEFun_GetStateByType` · 00536bf0 · legions 0051a960 — needs port: WState save and load efuns (SaveStateByType, GetStateByType)

## `portal/shared/src/interp/glue/nativefactory.cpp.c` (0/2)

### NativeFactory
- [-] `NativeFactory::NativeFactory` · 0054cbd0 · legions 0052a380 — C++ constructor of plain data or a container (Rust struct init)
- [-] `NativeFactory::~NativeFactory` · 0054cbf0 · legions 0052a3a0 — C++ destructor (Rust drop replaces it)

## `portal/shared/src/interp/heapobject.cpp.c` (3/6)

### HeapObject
- [-] `HeapObject::Clear` · 006dba60 · legions 00664f10 — VM heap clear and pack size (Rust memory and writer)
- [-] `HeapObject::GetPackSize` · 006dbab0 · legions 00664f60 — VM heap clear and pack size (Rust memory and writer)
- [x] `HeapObject::Pack` · 006dbb00 · legions 00664fb0 — ac2-net protocol::pack
- [x] `HeapObject::UnPack` · 006dbbd0 · legions 00665080 — ac2-script reference/native unpack
- [ ] `HeapObject::UpdatePackageReferences` · 006dbd50 · legions 00665200 — needs port: wlib package reference binding (HeapObject)
- [x] `HeapObject::UpdateReferences` · 006dbdd0 · legions 00665280 — vm/refs.rs fn update_references (HeapObject::UpdateReferences)

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

## `portal/shared/src/interp/interp_core.cpp.c` (29/31)

### FileNodeName_Int32
- [ ] `FileNodeName_Int32::GetType` · 006e3bf0 · legions 0066a360 — needs port: PFile node GetType (PFile tree names and leaves)

### Interp
- [x] `Interp::HandlePop` · 006e3c00 — ac2-script vm::exec
- [x] `Interp::HandlePopN` · 006e3c10 · legions 0066a380 — ac2-script vm::exec
- [x] `Interp::HandlePopFrame` · 006e3c30 — ac2-script vm::exec
- [x] `Interp::HandleCallStatic` · 006e3c70 — ac2-script vm::exec
- [x] `Interp::HandleCallEFun` · 006e3cc0 — ac2-script vm::efun (frame kept on Interrupted; m_efun runs again on resume)
- [x] `Interp::HandleRJmp` · 006e3d40 — ac2-script vm::exec

### FileNodeName_Bool
- [-] `FileNodeName_Bool::GetPackSize` · 006e3d50 · legions 0066a4c0 — PFile node pack size (writer)

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
- [-] `PackageState::PackageState` · 006e1630 · legions 00668470 — C++ constructor of plain data or a container (Rust struct init)
- [ ] `PackageState::Pack` · 006e1650 · legions 00668490 — needs port: WState save (Pack)
- [ ] `PackageState::Clear` · 006e16a0 · legions 006684e0 — needs port: WState save (Clear)
- [-] `PackageState::~PackageState` · 006e16d0 · legions 00668510 — C++ destructor (Rust drop replaces it)
- [x] `PackageState::UnPack` · 006e1700 · legions 00668540 — ac2-script WLib/WState/ByteStream decode

## `portal/shared/src/interp/packing/packbuffer.cpp.c` (14/20)

### PackBuffer
- [-] `PackBuffer::~PackBuffer` · 006ed4f0 · legions 00672300 — C++ destructor (Rust drop replaces it)
- [-] `PackBuffer::operator=` · 006ed500 · legions 00672310 — C++ assignment operator of plain data (Rust copy/move)

### PackageState
- [-] `PackageState::GetPackSize` · 006ed580 · legions 00672390 — WState pack size (writer)

### PackBuffer
- [x] `PackBuffer::UnPackInt` · 006ed590 · legions 006723a0 — ac2-script host (ScriptHost dispatch, PackBuffer, SendServerEvt, notices)
- [x] `PackBuffer::UnPackLInt` · 006ed630 · legions 00672440 — ac2-script host (ScriptHost dispatch, PackBuffer, SendServerEvt, notices)
- [x] `PackBuffer::UnPackFloat` · 006ed700 · legions 00672510 — ac2-script host (ScriptHost dispatch, PackBuffer, SendServerEvt, notices)
- [x] `PackBuffer::UnPackLFloat` · 006ed7b0 · legions 006725c0 — ac2-script host (ScriptHost dispatch, PackBuffer, SendServerEvt, notices)
- [x] `PackBuffer::UnPackPackage` · 006ed870 · legions 00672680 — ac2-script host (ScriptHost dispatch, PackBuffer, SendServerEvt, notices)
- [-] `PackBuffer::Grow` · 006ed920 · legions 00672730 — pack buffer growth (Rust Vec grows itself)
- [-] `PackBuffer::PackBuffer` · 006ed9a0 · legions 006727b0 — C++ constructor of plain data or a container (Rust struct init)
- [x] `PackBuffer::Pack` · 006ed9c0 · legions 006727d0
- [x] `PackBuffer::UnPack` · 006eda20 · legions 00672830
- [-] `PackBuffer::PackRaw` · 006edaa0 — pack buffer growth (Rust Vec grows itself)
- [x] `PackBuffer::PackPackage` · 006edb40 — ac2-script host (ScriptHost dispatch, PackBuffer, SendServerEvt, notices)
- [x] `PackBuffer::UnPackBuffer_Int` · 006edc00 · legions 00672920 — ac2-script host (ScriptHost dispatch, PackBuffer, SendServerEvt, notices)
- [x] `PackBuffer::UnPackBuffer_LongInt` · 006edc50 · legions 00672970 — ac2-script host (ScriptHost dispatch, PackBuffer, SendServerEvt, notices)
- [x] `PackBuffer::UnPackBuffer_Float` · 006edcc0 · legions 006729e0 — ac2-script host (ScriptHost dispatch, PackBuffer, SendServerEvt, notices)
- [x] `PackBuffer::UnPackBuffer_LongFloat` · 006edd10 · legions 00672a30 — ac2-script host (ScriptHost dispatch, PackBuffer, SendServerEvt, notices)
- [x] `PackBuffer::UnPackBuffer_Package` · 006edd90 · legions 00672ab0 — ac2-script host (ScriptHost dispatch, PackBuffer, SendServerEvt, notices)
- [x] `PackBuffer::UnPackBuffer` · 006edde0 · legions 00672b00 — ac2-script host (ScriptHost dispatch, PackBuffer, SendServerEvt, notices)

## `portal/shared/src/interp/packing/vmdata.cpp.c` (0/13)

### VMData
- [-] `VMData::VMData` · 006ece90 · legions 00671ca0 — C++ constructor of plain data or a container (Rust struct init)
- [ ] `VMData::GetReferenceData` · 006ecea0 · legions 00671cb0 — needs port: VM data attach (GetReferenceData)
- [-] `VMData::GetPackSize` · 006eceb0 — VM data refcount or checkpoint pack (Rust memory replaces it)
- [-] `VMData::Pack` · 006ecf30 — VM data refcount or checkpoint pack (Rust memory replaces it)
- [ ] `VMData::GetData` · 006ecff0 · legions 00671e00 — needs port: VM data attach (GetData)
- [ ] `VMData::HasData` · 006ed020 · legions 00671e30 — needs port: VM data attach (HasData)
- [-] `VMData::DecrementRefCount_` · 006ed060 · legions 00671e70 — VM data refcount or checkpoint pack (Rust memory replaces it)
- [-] `VMData::Clear_` · 006ed0d0 · legions 00671ee0 — VM data refcount or checkpoint pack (Rust memory replaces it)
- [ ] `VMData::RemoveData` · 006ed150 · legions 00671f60 — needs port: VM data attach (RemoveData)
- [-] `VMData::operator=` · 006ed1b0 · legions 00671fc0 — C++ assignment operator of plain data (Rust copy/move)
- [ ] `VMData::AddData` · 006ed2e0 · legions 006720f0 — needs port: VM data attach (AddData)
- [-] `VMData::VMData` · 006ed400 · legions 00672210 — C++ constructor of plain data or a container (Rust struct init)
- [-] `VMData::UnPack` · 006ed420 — VM data refcount or checkpoint pack (Rust memory replaces it)

## `portal/shared/src/interp/perf/vmmon.cpp.c` (0/9)

### VMMon
- [-] `VMMon::iInitClockDefaults` · 004bc8d0 — admin VM monitor and logger
- [-] `VMMon::iBeginExtra` · 004bc970 — admin VM monitor and logger
- [-] `VMMon::iEndExtra` · 004bc9b0 — admin VM monitor and logger
- [-] `VMMon::~VMMon` · 004bcb60 — C++ destructor (Rust drop replaces it)
- [-] `VMMon::iQuery` · 004bcb80 — admin VM monitor and logger

### SmartArray<PStringBase<char>_>
- [-] `SmartArray<PStringBase<char>_>::AddToEnd` · 004bccf0 · legions 004e60e0 — container internals (Rust collections replace them)

### VMMon
- [-] `VMMon::iLog` · 004bcd70 — admin VM monitor and logger
- [-] `VMMon::iInitMapper` · 004bce20 — admin VM monitor and logger
- [-] `VMMon::VMMon` · 004bd040 — C++ constructor of plain data or a container (Rust struct init)

## `portal/shared/src/interp/processstate.cpp.c` (0/14)

### ProcessState
- [ ] `ProcessState::GetPackageID` · 006dd7a0 · legions 00666b90 — needs port: VM process context GetPackageID (ProcessState)
- [ ] `ProcessState::ClearContextTable` · 006dd7c0 · legions 00666bb0 — needs port: VM process context ClearContextTable (ProcessState)
- [-] `ProcessState::QueryContexts` · 006dd860 — WSL debugger query or dump (process, reference and perf tables)
- [-] `ProcessState::QueryContextReferenceTree` · 006dd9a0 — WSL debugger query or dump (process, reference and perf tables)
- [-] `ProcessState::~ProcessState` · 006dda60 · legions 00666ca0 — C++ destructor (Rust drop replaces it)
- [ ] `ProcessState::GetContextState` · 006ddab0 · legions 00666cf0 — needs port: VM process context GetContextState (ProcessState)
- [ ] `ProcessState::DeleteContext` · 006ddaf0 · legions 00666d30 — needs port: VM process context DeleteContext (ProcessState)
- [-] `ProcessState::QueryReferenceTree` · 006ddb60 — WSL debugger query or dump (process, reference and perf tables)
- [ ] `ProcessState::CreateContext` · 006ddba0 · legions 00666da0 — needs port: VM process context CreateContext (ProcessState)
- [-] `ProcessState::ProcessState` · 006ddcf0 · legions 00666ea0 — C++ constructor of plain data or a container (Rust struct init)

### ReferenceTable
- [~] `ReferenceTable::AllocatePackage` · 006ddd50 · legions 00666f00 — ac2-script vm: no GC/reference reuse, no VMMon timers, efun ids resolved per call

### ProcessState
- [-] `ProcessState::ProcessState` · 006dddf0 · legions 00666fa0 — C++ constructor of plain data or a container (Rust struct init)

### ReferenceTable
- [~] `ReferenceTable::AllocatePackage` · 006dde60 · legions 00667010 — ac2-script vm: no GC/reference reuse, no VMMon timers, efun ids resolved per call

### ProcessState
- [-] `ProcessState::ProcessState` · 006ddfe0 · legions 00667190 — C++ constructor of plain data or a container (Rust struct init)

## `portal/shared/src/interp/processtable.cpp.c` (0/8)

### ProcessTable
- [-] `ProcessTable::~ProcessTable` · 006e17f0 — C++ destructor (Rust drop replaces it)
- [ ] `ProcessTable::UnRegisterProcess` · 006e1830 · legions 00668660 — needs port: VM process table UnRegisterProcess
- [ ] `ProcessTable::GetProcess` · 006e1860 · legions 00668640 — needs port: VM process table GetProcess
- [-] `ProcessTable::QueryProcess` · 006e1880 · legions 00668690 — WSL debugger query or dump (process, reference and perf tables)
- [-] `ProcessTable::ProcessTable` · 006e18b0 · legions 006686c0 — C++ constructor of plain data or a container (Rust struct init)
- [ ] `ProcessTable::RegisterProcess` · 006e18d0 · legions 006686e0 — needs port: VM process table RegisterProcess
- [-] `ProcessTable::QueryProcessTable` · 006e1920 · legions 00668730 — WSL debugger query or dump (process, reference and perf tables)
- [-] `ProcessTable::QueryProcesses` · 006e1960 · legions 00668770 — WSL debugger query or dump (process, reference and perf tables)

## `portal/shared/src/interp/referenceentry.cpp.c` (2/4)

### ReferenceEntry
- [-] `ReferenceEntry::GetPackSize` · 006e0ac0 · legions 00667990 — checkpoint size estimate (Rust memory replaces it)
- [-] `ReferenceEntry::GetResidentSizeEstimate` · 006e0b10 — checkpoint size estimate (Rust memory replaces it)
- [x] `ReferenceEntry::Pack` · 006e0b40 · legions 006679e0 — ac2-net protocol::pack
- [x] `ReferenceEntry::UnPack` · 006e0c20 · legions 00667ac0 — ac2-script reference/native unpack

## `portal/shared/src/interp/referencetable.cpp.c` (0/3)

### ReferenceTable
- [-] `ReferenceTable::QueryReferences` · 006e1a20 — WSL debugger query or dump (process, reference and perf tables)
- [-] `ReferenceTable::QueryReferenceTree_Recursive` · 006e1cd0 — WSL debugger query or dump (process, reference and perf tables)
- [-] `ReferenceTable::QueryReferenceTree` · 006e2040 — WSL debugger query or dump (process, reference and perf tables)

## `portal/shared/src/interp/reservedreftable.cpp.c` (1/4)

### ReservedRefTable
- [-] `ReservedRefTable::StartMarking` · 006e09e0 · legions 006678b0 — container internals (Rust collections replace them)

### Box2D
- [-] `Box2D::Box2D` · 006e09f0 · legions 006678c0 — C++ constructor of plain data or a container (Rust struct init)

### ReservedRefTable
- [-] `ReservedRefTable::StopMarking` · 006e0a00 · legions 006678d0 — container internals (Rust collections replace them)
- [x] `ReservedRefTable::AddReference` · 006e0a50 · legions 00667920 — ac2-script vm::refs (reserved bit on every new entry)

## `portal/shared/src/interp/runtimebytestream.cpp.c` (0/3)

### RuntimeByteStream
- [~] `RuntimeByteStream::ValidateVersion` · 006ecb60 — ac2-script ByteStream::validate_versions: wcc check; InterpVersionCheck env path not ported

### ReferenceTable
- [~] `ReferenceTable::CreateStringLiteral` · 006ecd30 · legions 00671b70 — ac2-script vm: no GC/reference reuse, no VMMon timers, efun ids resolved per call

### RuntimeByteStream
- [~] `RuntimeByteStream::LoadStringLitTableSection` · 006ecdb0 — ac2-script vm: no GC/reference reuse, no VMMon timers, efun ids resolved per call

## `portal/shared/src/interp/typeinfo.cpp.c` (0/3)

### (free functions)
- [ ] `new_validated_type` · 006e1490 — needs port: VM value type helper (new_validated_type)
- [ ] `get_type` · 006e14c0 · legions 00668300 — needs port: VM value type helper (get_type)
- [ ] `new_type` · 006e15a0 · legions 006683e0 — needs port: VM value type helper (new_type)

## `portal/shared/src/interp/vm.cpp.c` (13/66)

### VM
- [ ] `VM::CreateProcessA` · 006dbe00 · legions 006652b0 — needs port: VM CreateProcessA (process creation)
- [ ] `VM::CreateProcessA` · 006dbe30 · legions 006652e0 — needs port: VM CreateProcessA (process creation)
- [ ] `VM::CopyNativeObject` · 006dbe60 · legions 00665310 — needs port: VM::CopyNativeObject (native object copy)
- [ ] `VM::IsDerivedFrom` · 006dbe80 · legions 00665330 — needs port (check): implemented as glue/vm_glue.rs is_derived_from but uncited
- [ ] `VM::IsDerivedFrom` · 006dbea0 · legions 00665350 — needs port (check): implemented as glue/vm_glue.rs is_derived_from but uncited
- [ ] `VM::ValidateVersion` · 006dbec0 · legions 00665370 — needs port: VM::ValidateVersion (wlib version check)
- [-] `VM::GetReferences` · 006dbef0 · legions 006653a0 — VM garbage collector and allocator internals (Rust memory replaces them)
- [-] `VM::UpdateNativeReferences` · 006dbf10 · legions 006653c0 — VM garbage collector and allocator internals (Rust memory replaces them)
- [-] `VM::DeleteNativeObject` · 006dbf30 · legions 006653e0 — VM garbage collector and allocator internals (Rust memory replaces them)
- [-] `VM::GetNativePackSize` · 006dbf50 · legions 00665400 — VM garbage collector and allocator internals (Rust memory replaces them)
- [~] `VM::PackNativeObject` · 006dbf70 · legions 00665420 — ac2-net native read/write; coverage of every native object type unverified
- [~] `VM::UnPackNativeObject` · 006dbf90 · legions 00665440 — ac2-net native read/write; coverage of every native object type unverified
- [ ] `VM::GetContextID` · 006dbfb0 · legions 00665460 — needs port (check): implemented as glue/vm_glue.rs get_context_id but uncited
- [-] `VM::SetActiveGC` · 006dbfc0 · legions 00665470 — VM garbage collector and allocator internals (Rust memory replaces them)
- [-] `VM::SetAggressiveGCDeletion` · 006dbfd0 · legions 00665480 — VM garbage collector and allocator internals (Rust memory replaces them)
- [ ] `VM::WSL_EFun_GetInstanceReference` · 006dbfe0 · legions 00665490 — needs port (check): implemented as glue/vm_glue.rs get_instance_reference but uncited

### ReferenceTable
- [-] `ReferenceTable::GetReferencePackSize` · 006dbff0 · legions 006654a0 — VM garbage collector internals (Rust memory replaces them)
- [~] `ReferenceTable::PackReferenceArray` · 006dc040 · legions 006654f0 — ac2-net protocol::pack: packed reference graph both ways; no VM reference table (no WState lookup, no library slot check, no dependent-reference walk)
- [x] `ReferenceTable::GetDependentReferences_Relative` · 006dc0b0 · legions 00665560 — ac2-script vm::pack (dependent-reference walk, CloneReference)

### VM
- [ ] `VM::DeleteProcess` · 006dc0e0 · legions 00665590 — needs port (check): implemented as host/mod.rs remove_weenie (cites VM::DeleteProcess as caller)
- [~] `VM::HashNativeObject` · 006dc110 · legions 006655c0 — ac2-script glue::hashes NativeKeys: StringInfo and wpstring; CaseInsensitiveWPString (0x353F) not yet
- [~] `VM::IsNativeEqual` · 006dc140 · legions 006655f0 — ac2-script glue::hashes NativeKeys: StringInfo and wpstring; CaseInsensitiveWPString (0x353F) not yet
- [x] `VM::GetStringLiteral` · 006dc170 · legions 00665620 — ac2-script vm (VM/ReferenceTable/Resolver)
- [x] `VM::GetHeapObject` · 006dc190 · legions 00665640 — ac2-script vm (VM/ReferenceTable/Resolver)
- [-] `VM::IsReferenceValid` · 006dc1a0 — VM garbage collector and allocator internals (Rust memory replaces them)

### InterpreterBreakpointHelper::StepBreakpointData
- [-] `InterpreterBreakpointHelper::StepBreakpointData::~StepBreakpointData` · 006dc210 · legions 00665690 — script debugger

### ReferenceTable
- [-] `ReferenceTable::DeleteAllReferences` · 006dc270 · legions 006656f0 — VM garbage collector internals (Rust memory replaces them)

### VM
- [-] `VM::GetStackTrace` · 006dc2d0 · legions 00665750 — WSL debugger query or dump (process, reference and perf tables)
- [-] `VM::QueryProcessTableW` · 006dc390 · legions 00665810 — WSL debugger query or dump (process, reference and perf tables)
- [-] `VM::QueryProcessesW` · 006dc450 · legions 006658d0 — WSL debugger query or dump (process, reference and perf tables)
- [-] `VM::QueryProcessW` · 006dc520 · legions 006659a0 — WSL debugger query or dump (process, reference and perf tables)
- [-] `VM::QueryReferencesW` · 006dc5f0 · legions 00665a70 — WSL debugger query or dump (process, reference and perf tables)
- [-] `VM::QueryReferenceTreeW` · 006dc6b0 · legions 00665b30 — WSL debugger query or dump (process, reference and perf tables)
- [-] `VM::QueryReferenceTreeForProcessW` · 006dc780 · legions 00665c00 — WSL debugger query or dump (process, reference and perf tables)
- [-] `VM::DeleteAllUnUsedReferences` · 006dc850 · legions 00665cd0 — VM garbage collector and allocator internals (Rust memory replaces them)

### ReferenceTable
- [x] `ReferenceTable::GetDependentReferences_Native` · 006dc860 · legions 00665ce0 — ac2-script vm::pack (dependent-reference walk, CloneReference)
- [x] `ReferenceTable::GetDependentReferences_Package` · 006dc910 · legions 00665d90 — ac2-script vm::pack (dependent-reference walk, CloneReference)

### VM
- [-] `VM::QueryPerf` · 006dc9e0 — WSL debugger query or dump (process, reference and perf tables)
- [-] `VM::QueryPerfW` · 006dca70 — WSL debugger query or dump (process, reference and perf tables)

### ReferenceTable
- [-] `ReferenceTable::~ReferenceTable` · 006dcb80 · legions 00665f80 — C++ destructor (Rust drop replaces it)
- [x] `ReferenceTable::GetDependentReferences_HeapObject` · 006dcbe0 · legions 00665fe0 — ac2-script vm::pack (dependent-reference walk, CloneReference)

### VM
- [ ] `VM::GetPackageNameByPkgID` · 006dcc00 · legions 00666000 — needs port (check): implemented as glue/vm_glue.rs but uncited
- [ ] `VM::GetPackageNameByPkgIDW` · 006dcc50 · legions 00666050 — needs port (check): implemented as glue/vm_glue.rs but uncited
- [-] `VM::GarbageCollect` · 006dcc70 · legions 00666070 — VM garbage collector and allocator internals (Rust memory replaces them)
- [-] `VM::~VM` · 006dcc80 · legions 00666080 — C++ destructor (Rust drop replaces it)
- [~] `VM::GetPackageID` · 006dcce0 · legions 006660e0 — ac2-script wstate DID formulas / ByteStream::package_id
- [x] `VM::GetPackageID` · 006dcd20 · legions 00666120 — vm/program.rs fn package_id (VM::GetPackageID)
- [ ] `VM::GetSingletonPackage` · 006dce20 · legions 00666220 — needs port (check): implemented as vm/mod.rs singleton_package but uncited
- [ ] `VM::GetSingletonPackageW` · 006dce90 · legions 00666290 — needs port: VM::GetSingletonPackageW
- [~] `VM::GetSingletonPackageByID` · 006dcf50 · legions 00666350 — ac2-script wstate DID formulas / ByteStream::package_id
- [x] `VM::GetWStateReference` · 006dcf90 · legions 00666390 — ac2-script vm (VM/ReferenceTable/Resolver)
- [x] `VM::GetPackageReference` · 006dcfb0 · legions 006663b0 — ac2-script glue

### InterpreterBreakpointHelper
- [-] `InterpreterBreakpointHelper::InterpreterBreakpointHelper` · 006dd040 · legions 00666440 — script debugger

### ReferenceTable
- [-] `ReferenceTable::ReferenceTable` · 006dd0a0 · legions 006664a0 — C++ constructor of plain data or a container (Rust struct init)
- [x] `ReferenceTable::GetDependentReferences` · 006dd160 · legions 00666560 — ac2-script vm::pack (dependent-reference walk, CloneReference)

### VM
- [ ] `VM::VM` · 006dd220 · legions 00666620 — needs port (check): implemented as vm/mod.rs VM::new (string literal table load) but uncited
- [ ] `VM::CreateSingletonProcess` · 006dd2d0 · legions 006666d0 — needs port: VM CreateSingletonProcess (process creation)

### ReferenceTable
- [-] `ReferenceTable::GetReferencePackSize` · 006dd310 · legions 00666710 — VM garbage collector internals (Rust memory replaces them)
- [~] `ReferenceTable::PackReference` · 006dd3f0 · legions 006667f0 — ac2-script vm::pack: builds the PackedReference structure; the byte stream (native StreamPack) is the wire codec's

### VM
- [-] `VM::GetReferencePackSize` · 006dd4e0 · legions 006668e0 — VM garbage collector and allocator internals (Rust memory replaces them)
- [~] `VM::PackReference` · 006dd500 · legions 00666900 — ac2-script vm::pack: builds the PackedReference structure; the byte stream (native StreamPack) is the wire codec's
- [x] `VM::UnPackReference` · 006dd520 · legions 00666920 — ac2-script vm (VM/ReferenceTable/Resolver)
- [ ] `VM::Preproc_CleanWState` · 006dd540 · legions 00666940 — needs port: VM preprocessor WState cleanup
- [ ] `VM::GetPackageName` · 006dd630 · legions 00666a20 — needs port: VM::GetPackageName (package name by id)
- [x] `VM::IsInstanceOf` · 006dd690 · legions 00666a80 — ac2-script glue
- [x] `VM::CloneReference` · 006dd6c0 · legions 00666ab0 — ac2-script vm::pack (dependent-reference walk, CloneReference)

## `portal/shared/src/interp/vmlogger.cpp.c` (0/2)

### VMLogger
- [-] `VMLogger::OpenLog` · 004bd0f0 — admin VM monitor and logger
- [-] `VMLogger::Log` · 004bd2d0 — admin VM monitor and logger

## `portal/shared/src/interp/wccefuns.cpp.c` (4/4)

### (free functions)
- [x] `ToIID` · 006e0d80 · legions 00667bf0 — ac2-script glue
- [x] `ToSHex` · 006e13b0 · legions 00668220 — ac2-script glue
- [x] `ToLHex` · 006e1410 · legions 00668280 — ac2-script glue
- [x] `ToHex` · 006e1470 · legions 006682e0 — ac2-script glue

## `portal/shared/src/interp/wlink/wlibdb.cpp.c` (0/4)

### WLinkMap
- [ ] `WLinkMap::GetPackageName` · 006186a0 · legions 005d5080 — needs port: wlib link map package name lookup

### WLibDB
- [-] `WLibDB::I_GetPackageName` · 00618700 — admin wlib database (no Legions match)

### WLinkMap
- [-] `WLinkMap::GetPackageID` · 00618780 — admin wlib link map (no Legions match)

### WLibDB
- [-] `WLibDB::I_GetPackageID` · 006187f0 — admin wlib database (no Legions match)
