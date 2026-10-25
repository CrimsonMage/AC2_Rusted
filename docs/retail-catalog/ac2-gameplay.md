# ac2-gameplay: Characters, qualities, quests, fellowships, crafting, trade

499 retail functions: 114 done, 10 partial, 366 to do, 9 skipped.
Legend and workflow: [README](README.md).

## `ac2/shared/src/allegiance/allegiancedataadaptor.cpp.c` (2/4)

### AllegianceDataAdaptor
- [x] `AllegianceDataAdaptor::AllegianceDataAdaptor` · 0060c6f0 · legions 005cd440 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)
- [ ] `AllegianceDataAdaptor::operator=` · 0060c740 · legions 005cd490
- [x] `AllegianceDataAdaptor::AllegianceDataAdaptor` · 0060c7a0 · legions 005cd4f0 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)
- [ ] `AllegianceDataAdaptor::GetSpeciesGenderString` · 0060c7d0 · legions 005cd520

## `ac2/shared/src/allegiance/allegianceprofileadaptor.cpp.c` (2/5)

### AllegianceProfileAdaptor
- [ ] `AllegianceProfileAdaptor::InqRealMonarch` · 0060c8f0 · legions 005cd6f0

### List<AllegianceDataAdaptor>
- [ ] `List<AllegianceDataAdaptor>::operator=` · 0060c950 · legions 005cd750

### AllegianceProfileAdaptor
- [ ] `AllegianceProfileAdaptor::operator=` · 0060c990 · legions 005cd790
- [x] `AllegianceProfileAdaptor::AllegianceProfileAdaptor` · 0060ca40 · legions 005cd840 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)
- [x] `AllegianceProfileAdaptor::AllegianceProfileAdaptor` · 0060caf0 · legions 005cd8f0 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)

## `ac2/shared/src/character/gameplayoptionsprofile.cpp.c` (35/57)

### PFileNode
- [ ] `PFileNode::MakeSubNodeAsBool` · 005fe6a0 · legions 005bf180

### GameplayOptionsProfile
- [x] `GameplayOptionsProfile::SetShortcutHeight` · 005fe700 · legions 005bf1e0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `GameplayOptionsProfile::GetShortcutHeight` · 005fe710 · legions 005bf1f0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `GameplayOptionsProfile::SetUISaveLocations` · 005fe720 · legions 005bf200 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `GameplayOptionsProfile::InqUISaveLocations` · 005fe730 · legions 005bf210 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `GameplayOptionsProfile::TestBitMask` · 005fe750 · legions 005bf230 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `GameplayOptionsProfile::SetBitMask` · 005fe760 · legions 005bf240 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `GameplayOptionsProfile::SetShortcut` · 005fe7a0 · legions 005bf280 — ac2-weenie Progress stand-in: GameplayNotice::ReloadOptions / UpdateSkill / UpdateSkillPanel, set_shortcut (CPlayer 0x87)

### SmartArray<ShortcutInfo>
- [ ] `SmartArray<ShortcutInfo>::operator=` · 005fe7d0 · legions 005bf2b0

### PFileNode
- [ ] `PFileNode::MakeSubNode` · 005fe840

### GameplayOptionsProfile
- [x] `GameplayOptionsProfile::SetShortcutArray` · 005fe860 · legions 005bf320 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `GameplayOptionsProfile::InqShortcutArray` · 005fe890 · legions 005bf350 — ac2-script glue::options / examination (Legions layout and efun switch)
- [ ] `GameplayOptionsProfile::ShortcutsFromFileNode` · 005fe8c0 · legions 005bf380
- [ ] `GameplayOptionsProfile::DamageTextToFileNode` · 005fe920 · legions 005bf3e0
- [ ] `GameplayOptionsProfile::DamageTextFromFileNode` · 005fe9e0 · legions 005bf4a0
- [ ] `GameplayOptionsProfile::UISaveLocationsToFileNode` · 005fec10 · legions 005bf6d0
- [ ] `GameplayOptionsProfile::UISaveLocationsFromFileNode` · 005fec30 · legions 005bf6f0
- [ ] `GameplayOptionsProfile::FilterSettingsToFileNode` · 005fec90 · legions 005bf750
- [ ] `GameplayOptionsProfile::FilterSettingsFromFileNode` · 005fecb0 · legions 005bf770
- [ ] `GameplayOptionsProfile::AutoTargetToFileNode` · 005fed10 · legions 005bf7d0
- [ ] `GameplayOptionsProfile::AutoTargetFromFileNode` · 005fed30 · legions 005bf7f0
- [ ] `GameplayOptionsProfile::ConfirmationSettingsFromFileNode` · 005fed90 · legions 005bf850
- [x] `GameplayOptionsProfile::GetPackBitfield` · 005fedf0 · legions 005bf8b0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `GameplayOptionsProfile::InqShortcut` · 005feec0 · legions 005bf990 — ac2-script glue::options / examination (Legions layout and efun switch)
- [ ] `GameplayOptionsProfile::AliasesToFileNode` · 005feef0 · legions 005bf9c0
- [x] `GameplayOptionsProfile::BuildAliasList` · 005ff090 · legions 005bfbc0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `GameplayOptionsProfile::InitShortcutArray` · 005ff140 · legions 005bfc70 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `GameplayOptionsProfile::GetFilter` · 005ff1e0 · legions 005bfd10 — ac2-ui chat.rs / chat_window.rs
- [x] `GameplayOptionsProfile::GetChatFontSize` · 005ff220 · legions 005bfda0 — ac2-ui chat.rs / chat_window.rs
- [x] `GameplayOptionsProfile::GetChatFontColor` · 005ff270 — ac2-ui chat.rs / chat_window.rs
- [x] `GameplayOptionsProfile::GetChatPopupFlag` · 005ff2c0 · legions 005bfdf0 — ac2-ui chat.rs / chat_window.rs
- [~] `GameplayOptionsProfile::InqWindowOpacity` · 005ff310 · legions 005bfe90 — ac2-script native::options (ShortcutInfo keeps its packed datum only; unversioned profiles upgrade before packing; inner UI tables copied as is)
- [-] `GameplayOptionsProfile::ToFileNode` · 005ff330 · legions 005bfeb0 — text source format (admin tools)
- [ ] `GameplayOptionsProfile::Export` · 005ff400 · legions 005bff80
- [x] `GameplayOptionsProfile::InqAlias` · 005ff630 · legions 005c01b0 — ac2-weenie shell::expand_aliases over the options profile's m_aliasTable (ChatState::login_options)
- [x] `GameplayOptionsProfile::FlushAliasTable` · 005ff6e0 · legions 005c0260 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `GameplayOptionsProfile::RemoveAlias` · 005ff6f0 · legions 005c0270 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `GameplayOptionsProfile::RemoveFilter` · 005ff710 · legions 005c0290 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `GameplayOptionsProfile::SetFilter` · 005ff770 · legions 005c02f0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `GameplayOptionsProfile::SetChatFontSize` · 005ff790 · legions 005c04b0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `GameplayOptionsProfile::SetChatFontColor` · 005ff7b0 · legions 005c04d0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `GameplayOptionsProfile::FlushChatFontOptions` · 005ff7d0 · legions 005c04f0 — ac2-ui chat.rs / chat_window.rs
- [x] `GameplayOptionsProfile::SetChatPopupFlag` · 005ffa20 · legions 005c07e0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `GameplayOptionsProfile::SetWindowOpacity` · 005ffa40 · legions 005c0870 — ac2-script glue::options / examination (Legions layout and efun switch)

### HashTable<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>
- [ ] `HashTable<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>::operator=` · 005ffa60 · legions 005c0890

### GameplayOptionsProfile
- [x] `GameplayOptionsProfile::Clear` · 005ffc60 · legions 005c0a90 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `GameplayOptionsProfile::SetAlias` · 005ffce0 · legions 005c0af0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `GameplayOptionsProfile::SetAliasTable` · 005ffd00 · legions 005c0b10 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `GameplayOptionsProfile::InqAliasTable` · 005ffd10 · legions 005c0b20 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `GameplayOptionsProfile::AddFilter` · 005ffd30 · legions 005c0b40 — ac2-script glue::options / examination (Legions layout and efun switch)
- [ ] `GameplayOptionsProfile::AliasesFromFileNode` · 005ffd90 · legions 005c0ba0
- [x] `GameplayOptionsProfile::StreamPack` · 006000b0 — ac2-net protocol::native pack+unpack (Legions layouts; WState graphs round-trip byte-exact); ac2-script still decodes its own copy until it switches
- [ ] `GameplayOptionsProfile::operator=` · 006004f0 · legions 005c1360
- [-] `GameplayOptionsProfile::FromFileNode` · 006005c0 · legions 005c1440 — text source format (admin tools)
- [ ] `GameplayOptionsProfile::Import` · 00600870 · legions 005c16f0
- [x] `GameplayOptionsProfile::GameplayOptionsProfile` · 006009e0 · legions 005c1860 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `GameplayOptionsProfile::GameplayOptionsProfile` · 00600b40 · legions 005c19e0 — ac2-script glue::options / examination (Legions layout and efun switch)

## `ac2/shared/src/character/gmchargenresult.cpp.c` (20/26)

### gmCharGenResult
- [x] `gmCharGenResult::InqValidPhysTypeValues` · 00600cb0 · legions 005c1b70 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `gmCharGenResult::SetClassType` · 00600cf0 · legions 005c1bb0 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `gmCharGenResult::GetPhysTypeLabel` · 00600d00 · legions 005c1bc0 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [ ] `gmCharGenResult::InitDefaults` · 00600d60
- [x] `gmCharGenResult::Randomize` · 00600d90 · legions 005c1c20 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `gmCharGenResult::GetPhysObj` · 00600dc0 · legions 005c1c50 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [ ] `gmCharGenResult::InqAppliedAppearanceVDesc` · 00600df0 · legions 005c1c80
- [x] `gmCharGenResult::SetAdmin` · 00600e20 · legions 005c1cb0 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `gmCharGenResult::IsAdmin` · 00600e40 · legions 005c1cd0 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `gmCharGenResult::FormatName` · 00600e70 · legions 005c1d00 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `gmCharGenResult::SanityCheck` · 00600f10 · legions 005c1da0 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [ ] `gmCharGenResult::GetPackSize` · 00600f50 · legions 005c1de0
- [x] `gmCharGenResult::Pack` · 00600f90 · legions 005c1e20 — ac2-net CharGenResult::write/read
- [x] `gmCharGenResult::Clear` · 00601020 · legions 005c1eb0 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `gmCharGenResult::GetPhysTypeValue` · 00601090 · legions 005c1f20 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `gmCharGenResult::UnPack` · 006010d0 · legions 005c1f60 — ac2-net CharGenResult::write/read
- [x] `gmCharGenResult::operator=` · 00601170 · legions 005c2000 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `gmCharGenResult::SetPhysTypeValue` · 006011f0 · legions 005c2080 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `gmCharGenResult::GetStartingPosition` · 00601210 · legions 005c20a0 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [ ] `gmCharGenResult::~gmCharGenResult` · 00601310 · legions 005c21a0
- [x] `gmCharGenResult::GetStartArea` · 00601390 · legions 005c2220 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)

### CMotionInterp
- [ ] `CMotionInterp::SetVelocityScale` · 006013a0 · legions 005c2230

### gmCharGenResult
- [ ] `gmCharGenResult::SetStartingPosition` · 006013b0 · legions 005c2240
- [x] `gmCharGenResult::gmCharGenResult` · 006013d0 · legions 005c22b0 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `gmCharGenResult::gmCharGenResult` · 00601470 · legions 005c2350 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `gmCharGenResult::gmCharGenResult` · 00601500 · legions 005c23e0 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)

## `ac2/shared/src/character/gmracesexinfo.cpp.c` (6/8)

### gmRaceSexInfo
- [x] `gmRaceSexInfo::StreamPack` · 006015b0 · legions 005c2490 — ac2-net protocol::native pack+unpack (Legions layouts; WState graphs round-trip byte-exact); ac2-script still decodes its own copy until it switches
- [x] `gmRaceSexInfo::GetSubDataIDs` · 00601610 · legions 005c24f0 — ac2-script glue::places / enums / chargen (PlacesTable data::places)
- [x] `gmRaceSexInfo::Verify` · 006016a0 · legions 005c2580 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `gmRaceSexInfo::PutObjInLoc` · 00601720 · legions 005c2600 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [ ] `gmRaceSexInfo::~gmRaceSexInfo` · 00601740 · legions 005c2620
- [ ] `gmRaceSexInfo::operator=` · 006017b0 · legions 005c2690
- [x] `gmRaceSexInfo::gmRaceSexInfo` · 00601810 · legions 005c26f0 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)
- [x] `gmRaceSexInfo::gmRaceSexInfo` · 00601860 · legions 005c2740 — ac2-script host::chargen / glue::chargen (CharacterGenSystem IFuns, gmCharGenResult, gmRaceSexInfo)

## `ac2/shared/src/craft/craftskilladaptor.cpp.c` (0/3)

### CraftSkillAdaptor
- [ ] `CraftSkillAdaptor::CraftSkillAdaptor` · 0060c550 · legions 005cd2a0
- [ ] `CraftSkillAdaptor::operator=` · 0060c590 · legions 005cd2e0
- [ ] `CraftSkillAdaptor::CraftSkillAdaptor` · 0060c5f0 · legions 005cd340

## `ac2/shared/src/craft/craftskillcategoryadaptor.cpp.c` (0/6)

### CraftSkillCategoryAdaptor
- [ ] `CraftSkillCategoryAdaptor::CraftSkillCategoryAdaptor` · 0060c620 · legions 005cd370
- [ ] `CraftSkillCategoryAdaptor::CraftSkillCategoryAdaptor` · 0060c640 · legions 005cd390
- [ ] `CraftSkillCategoryAdaptor::~CraftSkillCategoryAdaptor` · 0060c670 · legions 005cd3c0
- [ ] `CraftSkillCategoryAdaptor::operator=` · 0060c680 · legions 005cd3d0
- [ ] `CraftSkillCategoryAdaptor::operator==` · 0060c6b0 · legions 005cd400
- [ ] `CraftSkillCategoryAdaptor::CraftSkillCategoryAdaptor` · 0060c6c0 · legions 005cd410

## `ac2/shared/src/craft/ingredientadaptor.cpp.c` (0/4)

### IngredientAdaptor
- [ ] `IngredientAdaptor::IngredientAdaptor` · 0060c2b0 · legions 005cd000
- [ ] `IngredientAdaptor::~IngredientAdaptor` · 0060c2d0 · legions 005cd020
- [ ] `IngredientAdaptor::operator=` · 0060c2e0 · legions 005cd030
- [ ] `IngredientAdaptor::IngredientAdaptor` · 0060c310 · legions 005cd060

## `ac2/shared/src/craft/recipeadaptor.cpp.c` (0/5)

### RecipeAdaptor
- [ ] `RecipeAdaptor::RecipeAdaptor` · 0060c350 · legions 005cd0a0

### CraftSkillAdaptor
- [ ] `CraftSkillAdaptor::~CraftSkillAdaptor` · 0060c3e0 · legions 005cd130

### RecipeAdaptor
- [ ] `RecipeAdaptor::operator=` · 0060c400 · legions 005cd150
- [ ] `RecipeAdaptor::SetDifficultyMinusSkillCutoff` · 0060c500 · legions 005cd250
- [ ] `RecipeAdaptor::RecipeAdaptor` · 0060c520 · legions 005cd270

## `ac2/shared/src/damagetext/damagetextblob.cpp.c` (0/5)

### DamageTextBlob
- [ ] `DamageTextBlob::operator=` · 00609850 · legions 005ca560
- [ ] `DamageTextBlob::Clear` · 006098d0 · legions 005ca5e0
- [ ] `DamageTextBlob::DamageTextBlob` · 00609950 · legions 005ca660
- [ ] `DamageTextBlob::DamageTextBlob` · 006099b0 · legions 005ca6c0
- [ ] `DamageTextBlob::StreamPack` · 006099e0 · legions 005ca6f0

## `ac2/shared/src/effectdisplay/effectuinode.cpp.c` (0/10)

### EffectUINode
- [ ] `EffectUINode::SetEnabled` · 00607ce0 · legions 005c8b80
- [ ] `EffectUINode::SetDisabled` · 00607cf0 · legions 005c8b90
- [ ] `EffectUINode::SetToggled` · 00607d00 · legions 005c8ba0
- [ ] `EffectUINode::SetPulsed` · 00607d10 · legions 005c8bb0
- [ ] `EffectUINode::IsPulsed` · 00607d20 · legions 005c8bc0
- [ ] `EffectUINode::operator=` · 00607d30 · legions 005c8bd0
- [ ] `EffectUINode::Clear` · 00607de0 · legions 005c8c80
- [ ] `EffectUINode::EffectUINode` · 00607e90 · legions 005c8d30
- [ ] `EffectUINode::EffectUINode` · 00607f00 · legions 005c8da0
- [ ] `EffectUINode::StreamPack` · 00607f50 · legions 005c8df0

## `ac2/shared/src/examination/examinationprofile.cpp.c` (2/35)

### ExaminationDataNode
- [ ] `ExaminationDataNode::operator==` · 006081f0 · legions 005c9310
- [ ] `ExaminationDataNode::operator=` · 00608270 · legions 005c9390
- [ ] `ExaminationDataNode::GetIntValue` · 006082d0 · legions 005c93f0
- [ ] `ExaminationDataNode::GetLongIntValue` · 006082f0 · legions 005c9410
- [ ] `ExaminationDataNode::GetFloatValue` · 00608310 · legions 005c9430
- [ ] `ExaminationDataNode::GetBoolValue` · 00608330 · legions 005c9450
- [ ] `ExaminationDataNode::GetImageValue` · 00608340 · legions 005c9460
- [ ] `ExaminationDataNode::GetCountdownValue` · 00608350 · legions 005c9470
- [ ] `ExaminationDataNode::GetIconDescValue` · 00608370 · legions 005c9490

### ExaminationProfile
- [x] `ExaminationProfile::StreamPack` · 00608380 · legions 005c94a0 — ac2-net protocol::native (native objects in script-call PackBuffers)

### ExaminationDataNode
- [ ] `ExaminationDataNode::ExaminationDataNode` · 006083d0 · legions 005c9500
- [ ] `ExaminationDataNode::~ExaminationDataNode` · 00608430 · legions 005c9560
- [x] `ExaminationDataNode::StreamPack` · 00608490 · legions 005c95c0 — ac2-net protocol::native (native objects in script-call PackBuffers)

### ExaminationProfile
- [ ] `ExaminationProfile::operator==` · 00608640 · legions 005c9770
- [ ] `ExaminationProfile::AddBool` · 00608800
- [ ] `ExaminationProfile::AddCountdown` · 006088a0
- [ ] `ExaminationProfile::AddFloat` · 00608950
- [ ] `ExaminationProfile::AddInt` · 006089f0
- [ ] `ExaminationProfile::AddLongInt` · 00608a90
- [ ] `ExaminationProfile::AddStringInfo` · 00608b40
- [ ] `ExaminationProfile::AddImage` · 00608bf0
- [ ] `ExaminationProfile::AddIconDesc` · 00608c80
- [ ] `ExaminationProfile::AddEffectID` · 00608d20
- [ ] `ExaminationProfile::AddTab` · 00608dc0
- [ ] `ExaminationProfile::AddTabs` · 00608e50
- [ ] `ExaminationProfile::AddBreak` · 00608ee0
- [ ] `ExaminationProfile::AddSimpleStringInfo` · 006090e0 · legions 005c9fc0

### PList<ExaminationDataNode>
- [ ] `PList<ExaminationDataNode>::Pack` · 00609140 · legions 005ca030
- [ ] `PList<ExaminationDataNode>::GetPackSize` · 00609190
- [ ] `PList<ExaminationDataNode>::UnPack` · 006091d0 · legions 005ca0d4

### ExaminationProfile
- [ ] `ExaminationProfile::ExaminationProfile` · 00609360 · legions 005ca250
- [ ] `ExaminationProfile::~ExaminationProfile` · 006093b0 · legions 005ca2a0

### PList<ExaminationDataNode>
- [ ] `PList<ExaminationDataNode>::operator=` · 00609400 · legions 005ca300

### ExaminationProfile
- [ ] `ExaminationProfile::operator=` · 006094a0 · legions 005ca360
- [ ] `ExaminationProfile::ExaminationProfile` · 006094d0 · legions 005ca3a0

## `ac2/shared/src/examination/examinationrequest.cpp.c` (1/6)

### ExaminationRequest
- [ ] `ExaminationRequest::Clear` · 00609530 · legions 005c9090
- [ ] `ExaminationRequest::operator==` · 00609550 · legions 005c90c0
- [ ] `ExaminationRequest::operator=` · 006095b0 · legions 005c9130
- [ ] `ExaminationRequest::ExaminationRequest` · 006095f0 · legions 005c9180
- [ ] `ExaminationRequest::ExaminationRequest` · 00609620 · legions 005c91c0
- [x] `ExaminationRequest::StreamPack` · 00609660 — ac2-net ExaminationRequest: Legions 0x005C9210 layout (+3 words, type 0x4000000D)

## `ac2/shared/src/examination/selectioninfo.cpp.c` (4/4)

### SelectionInfo
- [x] `SelectionInfo::operator==` · 00609700 · legions 005ca410 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `SelectionInfo::SelectionInfo` · 00609760 · legions 005ca470 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `SelectionInfo::SelectionInfo` · 00609780 · legions 005ca490 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `SelectionInfo::StreamPack` · 006097c0 · legions 005ca4d0 — ac2-net protocol::native (native objects in script-call PackBuffers)

## `ac2/shared/src/fellowship/fellowadaptor.cpp.c` (2/2)

### FellowAdaptor
- [x] `FellowAdaptor::FellowAdaptor` · 0060c230 · legions 005ccf80 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)
- [x] `FellowAdaptor::FellowAdaptor` · 0060c260 · legions 005ccfb0 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)

## `ac2/shared/src/fellowship/fellowshipadaptor.cpp.c` (3/5)

### HashTable<InstanceID,FellowAdaptor>
- [ ] `HashTable<InstanceID,FellowAdaptor>::operator=` · 0060bee0 · legions 005ccc30

### FellowshipAdaptor
- [ ] `FellowshipAdaptor::operator=` · 0060c080 · legions 005ccdd0
- [x] `FellowshipAdaptor::AddFellow` · 0060c0f0 · legions 005cce40 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)
- [x] `FellowshipAdaptor::FellowshipAdaptor` · 0060c1a0 · legions 005ccef0 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)
- [x] `FellowshipAdaptor::FellowshipAdaptor` · 0060c1f0 · legions 005ccf40 — ac2-script glue::social / native::social (adaptors as ac2-weenie types, retail hash order)

## `ac2/shared/src/quest/gmactinfo.cpp.c` (0/12)

### gmActInfo
- [ ] `gmActInfo::GetActName` · 0060b900 · legions 005cc650
- [ ] `gmActInfo::StreamPack` · 0060b920 · legions 005cc670
- [ ] `gmActInfo::AddScene` · 0060b990 · legions 005cc6e0
- [ ] `gmActInfo::gmActInfo` · 0060b9e0 · legions 005cc730
- [ ] `gmActInfo::~gmActInfo` · 0060ba40 · legions 005cc790

### PList<gmSceneInfo>
- [ ] `PList<gmSceneInfo>::operator=` · 0060ba80 · legions 005cc7d0

### gmActInfo
- [ ] `gmActInfo::DumpScenesW` · 0060bac0 · legions 005cc810
- [ ] `gmActInfo::Init` · 0060bc10 · legions 005cc960
- [ ] `gmActInfo::GetOrderedSceneList` · 0060bcb0 · legions 005cca00
- [ ] `gmActInfo::DumpW` · 0060bce0 · legions 005cca30
- [ ] `gmActInfo::operator=` · 0060be40 · legions 005ccb90
- [ ] `gmActInfo::gmActInfo` · 0060be90 · legions 005ccbe0

## `ac2/shared/src/quest/gmkeyframe.cpp.c` (1/23)

### gmKeyframe
- [ ] `gmKeyframe::GetSubDataIDs` · 0060a350 · legions 005cb060
- [ ] `gmKeyframe::HasVideoTrack` · 0060a3a0 · legions 005cb0b0
- [ ] `gmKeyframe::HasAudioTrack` · 0060a3b0 · legions 005cb0c0
- [ ] `gmKeyframe::HasVoiceTrack` · 0060a3c0 · legions 005cb0d0
- [ ] `gmKeyframe::HasSubtitle` · 0060a3d0 · legions 005cb0e0
- [ ] `gmKeyframe::HasDurationDefined` · 0060a3e0 · legions 005cb0f0
- [ ] `gmKeyframe::SetVideoTrack` · 0060a400 · legions 005cb110
- [ ] `gmKeyframe::SetAudioTrack` · 0060a410 · legions 005cb120
- [ ] `gmKeyframe::SetVoiceTrack` · 0060a420 · legions 005cb130
- [ ] `gmKeyframe::SetSubtitle` · 0060a430 · legions 005cb140
- [ ] `gmKeyframe::operator=` · 0060a450 · legions 005cb160
- [ ] `gmKeyframe::SetVideoTrackByName` · 0060a4e0 · legions 005cb1f0
- [ ] `gmKeyframe::SetAudioTrackByName` · 0060a530 · legions 005cb240
- [ ] `gmKeyframe::SetVoiceTrackByName` · 0060a580 · legions 005cb290
- [ ] `gmKeyframe::SetMovieTrack` · 0060a5d0 · legions 005cb2e0
- [ ] `gmKeyframe::gmKeyframe` · 0060a620 · legions 005cb330
- [ ] `gmKeyframe::Clear` · 0060a660 · legions 005cb370
- [ ] `gmKeyframe::ClearForHackers` · 0060a6a0 · legions 005cb3b0
- [ ] `gmKeyframe::gmKeyframe` · 0060a6d0 · legions 005cb3e0
- [ ] `gmKeyframe::~gmKeyframe` · 0060a730 · legions 005cb440
- [ ] `gmKeyframe::Dump` · 0060a7a0 · legions 005cb4b0
- [ ] `gmKeyframe::Verify` · 0060a900 · legions 005cb610
- [x] `gmKeyframe::StreamPack` · 0060aa40 · legions 005cb750 — ac2-net protocol::native pack+unpack (Legions layouts; WState graphs round-trip byte-exact); ac2-script still decodes its own copy until it switches

## `ac2/shared/src/quest/gmquestinfo.cpp.c` (5/15)

### gmQuestInfo
- [ ] `gmQuestInfo::~gmQuestInfo` · 0060aae0 · legions 005cb7f0
- [x] `gmQuestInfo::Init` · 0060ab10 · legions 005cb820 — ac2-script glue::quests / ac2-net native::quest
- [x] `gmQuestInfo::Clear` · 0060abd0 · legions 005cb8e0 — ac2-script glue::quests / ac2-net native::quest
- [x] `gmQuestInfo::GetStatus` · 0060ac50 · legions 005cb960 — ac2-script glue::quests / ac2-net native::quest
- [ ] `gmQuestInfo::MaxedOut` · 0060ac60 · legions 005cb970
- [ ] `gmQuestInfo::GetSecondsRemaining` · 0060ac80 · legions 005cb990
- [ ] `gmQuestInfo::InqExtraText` · 0060aca0 · legions 005cb9b0
- [ ] `gmQuestInfo::PlaysFXOnUpdate` · 0060ada0
- [ ] `gmQuestInfo::gmQuestInfo` · 0060adb0 · legions 005cbac0
- [ ] `gmQuestInfo::gmQuestInfo` · 0060ae20 · legions 005cbb30
- [ ] `gmQuestInfo::operator=` · 0060ae60 · legions 005cbb70
- [x] `gmQuestInfo::Update` · 0060aef0 · legions 005cbc00 — ac2-script glue::quests / ac2-net native::quest
- [x] `gmQuestInfo::StreamPack` · 0060af00 · legions 005cbc10 — ac2-script glue::quests / ac2-net native::quest
- [ ] `gmQuestInfo::gmQuestInfo` · 0060aff0 · legions 005cbd00
- [ ] `gmQuestInfo::DumpW` · 0060b030 · legions 005cbd40

## `ac2/shared/src/quest/gmquestinfolist.cpp.c` (5/12)

### gmQuestInfoList
- [x] `gmQuestInfoList::Find` · 0060b240 · legions 005cbf50 — ac2-script glue::quests / ac2-net native::quest
- [x] `gmQuestInfoList::UpdateQuestInfo` · 0060b3d0 · legions 005cc120 — ac2-script glue::quests / ac2-net native::quest
- [x] `gmQuestInfoList::Erase` · 0060b4b0 · legions 005cc200 — ac2-script glue::quests / ac2-net native::quest

### PList<gmQuestInfo>
- [x] `PList<gmQuestInfo>::Pack` · 0060b500 · legions 005cc250 — ac2-script glue::quests / ac2-net native::quest
- [ ] `PList<gmQuestInfo>::GetPackSize` · 0060b550
- [x] `PList<gmQuestInfo>::UnPack` · 0060b590 · legions 005cc2fa — ac2-script glue::quests / ac2-net native::quest

### gmQuestInfoList
- [ ] `gmQuestInfoList::gmQuestInfoList` · 0060b6d0
- [ ] `gmQuestInfoList::~gmQuestInfoList` · 0060b710 · legions 005cc460

### PList<gmQuestInfo>
- [ ] `PList<gmQuestInfo>::operator=` · 0060b750 · legions 005cc4a0

### gmQuestInfoList
- [ ] `gmQuestInfoList::DumpW` · 0060b790 · legions 005cc4e0
- [ ] `gmQuestInfoList::operator=` · 0060b8a0 · legions 005cc5f0
- [ ] `gmQuestInfoList::gmQuestInfoList` · 0060b8c0 · legions 005cc610

## `ac2/shared/src/quest/gmsceneinfo.cpp.c` (1/18)

### gmSceneInfo
- [ ] `gmSceneInfo::SetIDByName` · 00609a50
- [x] `gmSceneInfo::StreamPack` · 00609a80 · legions 005ca790 — ac2-net protocol::native pack+unpack (Legions layouts; WState graphs round-trip byte-exact); ac2-script still decodes its own copy until it switches
- [ ] `gmSceneInfo::Begin` · 00609b70 · legions 005ca880
- [ ] `gmSceneInfo::End` · 00609b90 · legions 005ca8a0
- [ ] `gmSceneInfo::ClearForHackers` · 00609bb0 · legions 005ca8c0
- [ ] `gmSceneInfo::GetSubDataIDs` · 00609bd0 · legions 005ca8e0
- [ ] `gmSceneInfo::AddKeyframe` · 00609c00 · legions 005ca910
- [ ] `gmSceneInfo::Clear` · 00609ca0 · legions 005ca9b0

### PList<gmSceneInfo>
- [ ] `PList<gmSceneInfo>::GetPackSize` · 00609ce0

### PList<gmKeyframe>
- [ ] `PList<gmKeyframe>::UnPack` · 00609d20 · legions 005caa44

### gmSceneInfo
- [ ] `gmSceneInfo::gmSceneInfo` · 00609e50 · legions 005cab60
- [ ] `gmSceneInfo::~gmSceneInfo` · 00609eb0 · legions 005cabc0

### PList<gmKeyframe>
- [ ] `PList<gmKeyframe>::operator=` · 00609f10 · legions 005cac20

### gmSceneInfo
- [ ] `gmSceneInfo::Dump` · 00609f50 · legions 005cac60
- [ ] `gmSceneInfo::Verify` · 0060a170 · legions 005cae80
- [ ] `gmSceneInfo::operator=` · 0060a230 · legions 005caf40
- [ ] `gmSceneInfo::Set` · 0060a280 · legions 005caf90
- [ ] `gmSceneInfo::gmSceneInfo` · 0060a2d0 · legions 005cafe0

## `ac2/shared/src/shop/uishop.cpp.c` (0/5)

### UIShop
- [ ] `UIShop::InqSale` · 0060cdb0 · legions 005cdbb0

### List<UIShopSale>
- [ ] `List<UIShopSale>::operator=` · 0060cdf0 · legions 005cdbf0

### UIShop
- [ ] `UIShop::UIShop` · 0060ce30 · legions 005cdc30
- [ ] `UIShop::operator=` · 0060ce90 · legions 005cdc90
- [ ] `UIShop::UIShop` · 0060cf10 · legions 005cdd10

## `ac2/shared/src/shop/uishopsale.cpp.c` (0/4)

### UIShopSale
- [ ] `UIShopSale::UIShopSale` · 0060cc80 · legions 005cda80
- [ ] `UIShopSale::operator=` · 0060cce0 · legions 005cdae0

### CraftSkillCategoryAdaptor
- [ ] `CraftSkillCategoryAdaptor::operator<` · 0060cd70 · legions 005cdb70

### UIShopSale
- [ ] `UIShopSale::UIShopSale` · 0060cd80 · legions 005cdb80

## `ac2/shared/src/shortcut/shortcutinfo.cpp.c` (4/6)

### ShortcutInfo
- [x] `ShortcutInfo::Clear` · 00607fd0 · legions 005c8e70 — ac2-script glue::options / examination (Legions layout and efun switch)
- [~] `ShortcutInfo::operator=` · 00608040 · legions 005c8ee0 — ac2-script native::options (ShortcutInfo keeps its packed datum only; unversioned profiles upgrade before packing; inner UI tables copied as is)
- [x] `ShortcutInfo::ShortcutInfo` · 006080b0 · legions 005c8f50 — ac2-script glue::options / examination (Legions layout and efun switch)
- [ ] `ShortcutInfo::~ShortcutInfo` · 006080e0 · legions 005c8f80
- [x] `ShortcutInfo::ShortcutInfo` · 00608120 · legions 005c8fc0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [x] `ShortcutInfo::StreamPack` · 00608150 · legions 005c8ff0 — ac2-net protocol::native pack+unpack (Legions layouts; WState graphs round-trip byte-exact); ac2-script still decodes its own copy until it switches

## `ac2/shared/src/skilldisplay/skilluinode.cpp.c` (4/7)

### SkillUINode
- [x] `SkillUINode::StreamPack` · 006079d0 · legions 005c8870 — ac2-net protocol::native pack+unpack (Legions layouts; WState graphs round-trip byte-exact); ac2-script still decodes its own copy until it switches
- [x] `SkillUINode::Clear` · 00607a90 · legions 005c8930 — ac2-script glue::skill_node over ac2-net's SkillUiNode
- [x] `SkillUINode::AddParent` · 00607ae0 · legions 005c8980 — ac2-script glue::skill_node over ac2-net's SkillUiNode
- [x] `SkillUINode::AddDependency` · 00607b10 · legions 005c89b0 — ac2-script glue::skill_node over ac2-net's SkillUiNode
- [ ] `SkillUINode::SkillUINode` · 00607b40 · legions 005c89e0
- [ ] `SkillUINode::operator=` · 00607bd0 · legions 005c8a70
- [ ] `SkillUINode::SkillUINode` · 00607c60 · legions 005c8b00

## `portal/shared/src/conversation/conversationaction.cpp.c` (0/6)

### ConversationAction
- [ ] `ConversationAction::ConversationAction` · 006bb610 · legions 00650f40
- [ ] `ConversationAction::ConversationAction` · 006bb640 · legions 00650f70
- [ ] `ConversationAction::~ConversationAction` · 006bb670 · legions 00650fa0
- [-] `ConversationAction::FromFileNode` · 006bb6f0 — text source format (admin tools)
- [-] `ConversationAction::ToFileNode` · 006bb8a0 — text source format (admin tools)
- [ ] `ConversationAction::StreamPack` · 006bb930 · legions 00651040

## `portal/shared/src/conversation/conversationdecision.cpp.c` (0/10)

### ConversationDecision
- [ ] `ConversationDecision::WantsResponse` · 006bb980
- [ ] `ConversationDecision::ConversationDecision` · 006bb990 · legions 00651090
- [ ] `ConversationDecision::ConversationDecision` · 006bb9d0 · legions 006510d0
- [ ] `ConversationDecision::operator=` · 006bba20 · legions 00675360
- [ ] `ConversationDecision::Destroy` · 006bbab0 · legions 00651120
- [ ] `ConversationDecision::~ConversationDecision` · 006bbb40 · legions 006511b0
- [ ] `ConversationDecision::FromFileNode_Decision` · 006bbba0
- [ ] `ConversationDecision::FromFileNode_Response` · 006bbf50
- [-] `ConversationDecision::ToFileNode` · 006bc0d0 — text source format (admin tools)
- [ ] `ConversationDecision::StreamPack` · 006bc290 · legions 00651230

## `portal/shared/src/conversation/conversationnode.cpp.c` (0/6)

### ConversationNode
- [ ] `ConversationNode::Destroy` · 006bafb0
- [-] `ConversationNode::FromFileNode` · 006bb000 — text source format (admin tools)
- [ ] `ConversationNode::ConversationNode` · 006bb340 · legions 00650e10
- [ ] `ConversationNode::~ConversationNode` · 006bb3a0 · legions 00650e70
- [-] `ConversationNode::ToFileNode` · 006bb410 — text source format (admin tools)
- [ ] `ConversationNode::StreamPack` · 006bb5b0 · legions 00650ee0

## `portal/shared/src/conversation/conversationtree.cpp.c` (0/18)

### ConversationTree
- [-] `ConversationTree::ToFileNode` · 006b9af0 — text source format (admin tools)

### PList<ConversationAction>
- [ ] `PList<ConversationAction>::UnPack` · 006b9d40 · legions 0064fe54

### PList<ConversationDecision>
- [ ] `PList<ConversationDecision>::Pack` · 006b9e10 · legions 0064ff10
- [ ] `PList<ConversationDecision>::GetPackSize` · 006b9e60
- [ ] `PList<ConversationDecision>::UnPack` · 006b9ea0 · legions 0064ffb4

### ConversationNode
- [ ] `ConversationNode::operator=` · 006ba160 · legions 00650260
- [ ] `ConversationNode::ConversationNode` · 006ba200 · legions 00650300

### ConversationTree
- [ ] `ConversationTree::Destroy` · 006ba290
- [ ] `ConversationTree::ConversationTree` · 006ba2d0 · legions 006503d0
- [ ] `ConversationTree::GetDBOType` · 006ba320
- [ ] `ConversationTree::~ConversationTree` · 006ba330 · legions 00650430
- [ ] `ConversationTree::Allocate` · 006ba400

### _STL::_Rb_tree<PStringBase<unsigned_short>,_STL::pair<PStringBase<unsigned_short>_const_,ConversationNode>,_STL::_Select1st<_STL::pair<PStringBase<unsigned_short>_const_,ConversationNode>_>,_STL::less<PStringBase<unsigned_short>_>,_STL::allocator<_STL::pair<PStringBase<unsigned_short>_const_,ConversationNode>_>_>
- [ ] `_STL::_Rb_tree<PStringBase<unsigned_short>,_STL::pair<PStringBase<unsigned_short>_const_,ConversationNode>,_STL::_Select1st<_STL::pair<PStringBase<unsigned_short>_const_,ConversationNode>_>,_STL::less<PStringBase<unsigned_short>_>,_STL::allocator<_STL::pair<PStringBase<unsigned_short>_const_,ConversationNode>_>_>::operator=` · 006ba700 · legions 00650800

### (free functions)
- [ ] `STREAMPACK_STL` · 006baa40 · legions 00650b40

### ConversationTree
- [ ] `ConversationTree::StreamPack` · 006bacb0
- [ ] `ConversationTree::CopyInto` · 006bacf0

### _STL::map<PStringBase<unsigned_short>,ConversationNode,_STL::less<PStringBase<unsigned_short>_>,_STL::allocator<_STL::pair<PStringBase<unsigned_short>_const_,ConversationNode>_>_>
- [ ] `_STL::map<PStringBase<unsigned_short>,ConversationNode,_STL::less<PStringBase<unsigned_short>_>,_STL::allocator<_STL::pair<PStringBase<unsigned_short>_const_,ConversationNode>_>_>::operator[]` · 006bad10

### ConversationTree
- [-] `ConversationTree::FromFileNode` · 006badd0 — text source format (admin tools)

## `portal/shared/src/gameplay/character/characteridentity.cpp.c` (1/4)

### Preference
- [ ] `Preference::SetHelpText` · 006db020 · legions 00664490

### CharacterIdentity
- [ ] `CharacterIdentity::CharacterIdentity` · 006db030 · legions 006644a0
- [ ] `CharacterIdentity::operator=` · 006db080 · legions 006644f0
- [x] `CharacterIdentity::StreamPack` · 006db0f0 · legions 00664560

## `portal/shared/src/gameplay/character/characterset.cpp.c` (2/14)

### CharacterSet
- [ ] `CharacterSet::WhichSlot` · 006db150 · legions 006645c0
- [ ] `CharacterSet::GetIdentity` · 006db1a0 · legions 00664610
- [ ] `CharacterSet::GetName` · 006db1d0 · legions 00664640
- [ ] `CharacterSet::GetID` · 006db210 · legions 00664680
- [ ] `CharacterSet::SetGreyedOutFor` · 006db270 · legions 006646e0
- [ ] `CharacterSet::GetGreyedOutFor` · 006db2b0 · legions 00664720
- [ ] `CharacterSet::CharacterSet` · 006db3e0 · legions 00664850
- [ ] `CharacterSet::IncludesIdentity` · 006db430 · legions 006648b0

### PSmartArray<CharacterIdentity>
- [x] `PSmartArray<CharacterIdentity>::StreamPack` · 006db450 · legions 006648d0

### SmartArray<CharacterIdentity>
- [ ] `SmartArray<CharacterIdentity>::AddToEnd` · 006db520 · legions 006649a0
- [ ] `SmartArray<CharacterIdentity>::InsertOrderedByIndex` · 006db570 · legions 006649f0

### CharacterSet
- [ ] `CharacterSet::operator=` · 006db620 · legions 00664aa0
- [ ] `CharacterSet::AddIdentity` · 006db7a0 · legions 00664c30
- [x] `CharacterSet::StreamPack` · 006db7f0

## `portal/shared/src/gameplay/qualities/basequalities.cpp.c` (0/66)

### PackObj
- [ ] `PackObj::VERIFY_ADDR` · 00516990 · legions 005049c0

### CBaseQualities
- [ ] `CBaseQualities::CBaseQualities` · 005169b0 · legions 005049e0
- [ ] `CBaseQualities::GetDBOType` · 00516a00
- [ ] `CBaseQualities::CBaseQualities` · 00516a10 · legions 00504a40
- [ ] `CBaseQualities::InitDefaults` · 00516a50 · legions 00504a80
- [ ] `CBaseQualities::IsMobile` · 00516aa0 · legions 00504ad0
- [ ] `CBaseQualities::Allocate` · 00516ac0
- [ ] `CBaseQualities::Clear` · 00516ad0 · legions 00504b00
- [ ] `CBaseQualities::~CBaseQualities` · 00516cf0 · legions 00504d20
- [ ] `CBaseQualities::GetPackHeader` · 00516d10 · legions 00504d40
- [ ] `CBaseQualities::GetPackSize` · 00516df0 · legions 00504e20
- [ ] `CBaseQualities::Pack` · 00516f80
- [ ] `CBaseQualities::GetIntTableIterator` · 00517170 · legions 00505230
- [ ] `CBaseQualities::GetLongIntTableIterator` · 005171c0 · legions 00505280
- [ ] `CBaseQualities::GetBoolTableIterator` · 00517210 · legions 005052d0
- [ ] `CBaseQualities::GetFloatTableIterator` · 00517260 · legions 00505320
- [ ] `CBaseQualities::GetTSTableIterator` · 005172b0 · legions 00505370
- [ ] `CBaseQualities::GetStringTableIterator` · 00517300 · legions 005053c0
- [ ] `CBaseQualities::GetDataIDTableIterator` · 00517350 · legions 00505410
- [ ] `CBaseQualities::GetInstanceIDTableIterator` · 005173a0 · legions 00505460
- [ ] `CBaseQualities::GetPositionTableIterator` · 005173f0 · legions 005054b0
- [ ] `CBaseQualities::GetStringInfoTableIterator` · 00517440 · legions 00505500
- [ ] `CBaseQualities::GetIntTableEnd` · 00517490 · legions 00505550
- [ ] `CBaseQualities::GetLongIntTableEnd` · 005174c0 · legions 00505580
- [ ] `CBaseQualities::GetBoolTableEnd` · 005174f0 · legions 005055b0
- [ ] `CBaseQualities::GetFloatTableEnd` · 00517520 · legions 005055e0
- [ ] `CBaseQualities::GetTSTableEnd` · 00517550 · legions 00505610
- [ ] `CBaseQualities::GetStringTableEnd` · 00517580 · legions 00505640
- [ ] `CBaseQualities::GetDataIDTableEnd` · 005175b0 · legions 00505670
- [ ] `CBaseQualities::GetInstanceIDTableEnd` · 005175e0 · legions 005056a0
- [ ] `CBaseQualities::GetPositionTableEnd` · 00517610 · legions 005056d0
- [ ] `CBaseQualities::GetStringInfoTableEnd` · 00517640 · legions 00505700
- [ ] `CBaseQualities::GetSubDataIDs` · 00517670

### HashTable<unsigned_long,long>
- [ ] `HashTable<unsigned_long,long>::~HashTable<unsigned_long,long>` · 00517790 · legions 00505850

### HashTable<unsigned_long,__int64>
- [ ] `HashTable<unsigned_long,__int64>::~HashTable<unsigned_long,__int64>` · 005177d0 · legions 00505980

### HashTable<unsigned_long,double>
- [ ] `HashTable<unsigned_long,double>::~HashTable<unsigned_long,double>` · 00517810 · legions 005059c0

### HashTable<unsigned_long,Position>
- [ ] `HashTable<unsigned_long,Position>::~HashTable<unsigned_long,Position>` · 00517850 · legions 00505a00
- [ ] `HashTable<unsigned_long,Position>::operator=` · 00517890 · legions 00505a40

### HashTable<unsigned_long,StringInfo>
- [ ] `HashTable<unsigned_long,StringInfo>::~HashTable<unsigned_long,StringInfo>` · 005179c0 · legions 00505b70

### PHashTable<unsigned_long,long>
- [ ] `PHashTable<unsigned_long,long>::PHashTable<unsigned_long,long>` · 00517f10 · legions 005060c0
- [ ] `PHashTable<unsigned_long,long>::PHashTable<unsigned_long,long>` · 00517f90 · legions 00506140
- [~] `PHashTable<unsigned_long,long>::StreamPack` · 00517fd0 — unpack only, in ac2-net protocol::qualities / visual (Legions CBaseQualities::UnPack instruction-identical)

### PHashTable<unsigned_long,__int64>
- [ ] `PHashTable<unsigned_long,__int64>::PHashTable<unsigned_long,__int64>` · 00518190 · legions 00506340
- [ ] `PHashTable<unsigned_long,__int64>::PHashTable<unsigned_long,__int64>` · 00518210 · legions 005063c0
- [~] `PHashTable<unsigned_long,__int64>::StreamPack` · 00518250 — unpack only, in ac2-net protocol::qualities / visual (Legions CBaseQualities::UnPack instruction-identical)

### PHashTable<unsigned_long,int>
- [ ] `PHashTable<unsigned_long,int>::PHashTable<unsigned_long,int>` · 00518440 · legions 005065f0
- [ ] `PHashTable<unsigned_long,int>::PHashTable<unsigned_long,int>` · 00518470 · legions 00506620

### PHashTable<unsigned_long,float>
- [ ] `PHashTable<unsigned_long,float>::PHashTable<unsigned_long,float>` · 005184b0 · legions 00506660

### PHashTable<unsigned_long,double>
- [ ] `PHashTable<unsigned_long,double>::PHashTable<unsigned_long,double>` · 005184e0 · legions 00506690
- [ ] `PHashTable<unsigned_long,double>::PHashTable<unsigned_long,double>` · 00518560 · legions 00506710
- [~] `PHashTable<unsigned_long,double>::StreamPack` · 005185a0 — unpack only, in ac2-net protocol::qualities / visual (Legions CBaseQualities::UnPack instruction-identical)

### PHashTable<unsigned_long,PStringBase<char>_>
- [ ] `PHashTable<unsigned_long,PStringBase<char>_>::PHashTable<unsigned_long,PStringBase<char>_>` · 00518760 · legions 00506910
- [ ] `PHashTable<unsigned_long,PStringBase<char>_>::PHashTable<unsigned_long,PStringBase<char>_>` · 005187a0 · legions 00506950
- [~] `PHashTable<unsigned_long,PStringBase<char>_>::StreamPack` · 005187e0 — unpack only, in ac2-net protocol::qualities / visual (Legions CBaseQualities::UnPack instruction-identical)

### PHashTable<unsigned_long,unsigned_long>
- [ ] `PHashTable<unsigned_long,unsigned_long>::PHashTable<unsigned_long,unsigned_long>` · 005189d0 · legions 00506b80

### PHashTable<unsigned_long,InstanceID>
- [ ] `PHashTable<unsigned_long,InstanceID>::PHashTable<unsigned_long,InstanceID>` · 00518a00 · legions 00506bb0
- [ ] `PHashTable<unsigned_long,InstanceID>::PHashTable<unsigned_long,InstanceID>` · 00518a30 · legions 00506be0

### PHashTable<unsigned_long,Position>
- [ ] `PHashTable<unsigned_long,Position>::PHashTable<unsigned_long,Position>` · 00518a70 · legions 00506c20
- [ ] `PHashTable<unsigned_long,Position>::PHashTable<unsigned_long,Position>` · 00518af0 · legions 00506ca0
- [~] `PHashTable<unsigned_long,Position>::StreamPack` · 00518b30 — unpack only, in ac2-net protocol::qualities / visual (Legions CBaseQualities::UnPack instruction-identical)

### PHashTable<unsigned_long,StringInfo>
- [ ] `PHashTable<unsigned_long,StringInfo>::PHashTable<unsigned_long,StringInfo>` · 00518d00 · legions 00506eb0
- [ ] `PHashTable<unsigned_long,StringInfo>::PHashTable<unsigned_long,StringInfo>` · 00518d80 · legions 00506f30
- [~] `PHashTable<unsigned_long,StringInfo>::StreamPack` · 00518dc0 — unpack only, in ac2-net protocol::qualities / visual (Legions CBaseQualities::UnPack instruction-identical)

### CBaseQualities
- [ ] `CBaseQualities::CopyTables` · 00518fd0 · legions 00507180
- [~] `CBaseQualities::UnPack` · 005192f0 · legions 005074a0 — unpack only, in ac2-net protocol::qualities / visual (Legions CBaseQualities::UnPack instruction-identical)
- [ ] `CBaseQualities::operator=` · 00519840 · legions 005079f0

## `portal/shared/src/gameplay/qualities/basequalitiesstats.cpp.c` (11/69)

### CBaseQualities
- [ ] `CBaseQualities::InqWeenieDescription` · 00519880
- [ ] `CBaseQualities::AllocateWeenieDesc` · 005198a0 · legions 00507a50
- [ ] `CBaseQualities::CopyWeenieDesc` · 005198d0 · legions 00507a80
- [ ] `CBaseQualities::TruncateInt` · 00519930 · legions 00507ae0
- [ ] `CBaseQualities::TruncateLongInt` · 00519970 · legions 00507b20
- [ ] `CBaseQualities::TruncateFloat` · 00519a00 · legions 00507bb0
- [x] `CBaseQualities::IsInWeenieDesc` · 00519a60 · legions 00507c10
- [ ] `CBaseQualities::SetWeenieDesc` · 00519ac0 · legions 00507c70
- [ ] `CBaseQualities::DumpWeenieDesc` · 00519af0 · legions 00507ca0
- [ ] `CBaseQualities::RemoveInt` · 00519ba0 · legions 00507d50
- [ ] `CBaseQualities::InqInt` · 00519c10 · legions 00507dc0
- [ ] `CBaseQualities::RemoveLongInt` · 00519c70 · legions 00507e20
- [ ] `CBaseQualities::InqLongInt` · 00519ce0 · legions 00507e90
- [ ] `CBaseQualities::RemoveBool` · 00519d50 · legions 00507f00
- [ ] `CBaseQualities::InqBool` · 00519dc0 · legions 00507f70
- [ ] `CBaseQualities::RemoveFloat` · 00519e20 · legions 00507fd0
- [ ] `CBaseQualities::InqFloat` · 00519e90 · legions 00508040
- [ ] `CBaseQualities::RemoveTimestamp` · 00519ef0 · legions 005080a0
- [ ] `CBaseQualities::InqTimestamp` · 00519f70 · legions 00508120
- [ ] `CBaseQualities::RemoveDataID` · 00519fe0 · legions 00508190
- [ ] `CBaseQualities::InqDataID` · 0051a050 · legions 00508200
- [ ] `CBaseQualities::RemoveInstanceID` · 0051a0b0 · legions 00508260
- [ ] `CBaseQualities::InqInstanceID` · 0051a130 · legions 005082e0
- [ ] `CBaseQualities::InqString` · 0051a1a0 · legions 00508350
- [ ] `CBaseQualities::InqStringW` · 0051a210 · legions 005083c0
- [ ] `CBaseQualities::InqPosition` · 0051a2d0 · legions 00508480
- [ ] `CBaseQualities::RemoveStringInfo` · 0051a340 · legions 005084f0
- [ ] `CBaseQualities::InqStringInfo` · 0051a3c0 · legions 00508570
- [ ] `CBaseQualities::RemovePackageID` · 0051a430 · legions 005085e0
- [ ] `CBaseQualities::InqPackageID` · 0051a4a0 · legions 00508650
- [ ] `CBaseQualities::RemoveInt` · 0051a650 · legions 005087a0
- [ ] `CBaseQualities::RemoveLongInt` · 0051a670 · legions 005087c0
- [ ] `CBaseQualities::RemoveBool` · 0051a690 · legions 005087e0
- [ ] `CBaseQualities::RemoveFloat` · 0051a6b0 · legions 00508800
- [ ] `CBaseQualities::RemoveTimestamp` · 0051a6d0 · legions 00508820
- [ ] `CBaseQualities::RemoveDataID` · 0051a6f0 · legions 00508840
- [ ] `CBaseQualities::RemoveInstanceID` · 0051a710 · legions 00508860
- [ ] `CBaseQualities::RemoveString` · 0051a730 · legions 00508880
- [ ] `CBaseQualities::RemovePosition` · 0051a7c0 · legions 00508910
- [ ] `CBaseQualities::RemoveStringInfo` · 0051a8b0 · legions 00508a00
- [ ] `CBaseQualities::RemovePackageID` · 0051a8f0 · legions 00508a40
- [ ] `CBaseQualities::AllocateIntTable` · 0051a910 · legions 00508a60
- [ ] `CBaseQualities::AllocateLongIntTable` · 0051a970 · legions 00508ac0
- [ ] `CBaseQualities::AllocateBoolTable` · 0051a9d0 · legions 00508b20
- [ ] `CBaseQualities::AllocateFloatTable` · 0051aa30 · legions 00508b80
- [ ] `CBaseQualities::AllocateTSTable` · 0051aa90 · legions 00508be0
- [ ] `CBaseQualities::AllocateStringTable` · 0051aaf0 · legions 00508c40
- [ ] `CBaseQualities::AllocateDataIDTable` · 0051ab50 · legions 00508ca0
- [ ] `CBaseQualities::AllocateInstanceIDTable` · 0051abb0 · legions 00508d00
- [ ] `CBaseQualities::AllocatePositionTable` · 0051ac10 · legions 00508d60
- [ ] `CBaseQualities::AllocateStringInfoTable` · 0051ac70 · legions 00508dc0
- [ ] `CBaseQualities::AllocatePackageIDTable` · 0051acd0 · legions 00508e20
- [x] `CBaseQualities::SetInt` · 0051ad30 · legions 00508e80
- [ ] `CBaseQualities::AdjInt` · 0051adb0 · legions 00508f00
- [ ] `CBaseQualities::AdjAndTruncateInt` · 0051ae70 · legions 00508fc0
- [x] `CBaseQualities::SetLongInt` · 0051af80 · legions 005090d0
- [ ] `CBaseQualities::AdjLongInt` · 0051b000 · legions 00509150
- [ ] `CBaseQualities::AdjAndTruncateLongInt` · 0051b110 · legions 00509260
- [x] `CBaseQualities::SetBool` · 0051b270 · legions 005093c0
- [x] `CBaseQualities::SetFloat` · 0051b2f0 · legions 00509440
- [ ] `CBaseQualities::AdjFloat` · 0051b370 · legions 005094c0
- [ ] `CBaseQualities::AdjAndTruncateFloat` · 0051b430 · legions 00509580
- [x] `CBaseQualities::SetTimestamp` · 0051b540 · legions 00509690
- [x] `CBaseQualities::SetDataID` · 0051b5c0 · legions 00509710
- [x] `CBaseQualities::SetInstanceID` · 0051b640 · legions 00509790
- [x] `CBaseQualities::SetString` · 0051b6c0 · legions 00509810
- [x] `CBaseQualities::SetPosition` · 0051b740 · legions 00509890
- [x] `CBaseQualities::SetStringInfo` · 0051b7c0 · legions 00509910
- [ ] `CBaseQualities::SetPackageID` · 0051b840 · legions 00509990

## `portal/shared/src/gameplay/qualities/physobjprofile.cpp.c` (0/1)

### PhysObjProfile
- [ ] `PhysObjProfile::PhysObjProfile` · 006dba20 · legions 00664ed0

## `portal/shared/src/gameplay/qualities/qualityfilter.cpp.c` (0/7)

### QualityFilter
- [ ] `QualityFilter::GetPackSize` · 006db860
- [ ] `QualityFilter::Pack` · 006db890 · legions 00664d40
- [ ] `QualityFilter::UnPack` · 006db8d0
- [ ] `QualityFilter::~QualityFilter` · 006db940 · legions 00664df0
- [ ] `QualityFilter::GetDBOType` · 006db9a0
- [ ] `QualityFilter::QualityFilter` · 006db9d0 · legions 00664e80
- [ ] `QualityFilter::Allocate` · 006dba10

## `portal/shared/src/gameplay/uilocationdata.cpp.c` (0/3)

### UILocationData
- [ ] `UILocationData::operator=` · 006da910 · legions 00663d50
- [ ] `UILocationData::UILocationData` · 006da940 · legions 00663d80
- [ ] `UILocationData::UILocationData` · 006da970 · legions 00663db0

## `portal/shared/src/gameplay/uisavelocations.cpp.c` (3/8)

### HashTable<unsigned_long,UILocationData>
- [ ] `HashTable<unsigned_long,UILocationData>::operator=` · 006daa70 · legions 00663ee0

### UISaveLocations
- [ ] `UISaveLocations::InqUILocationData` · 006dab60 · legions 00663fd0
- [x] `UISaveLocations::Clear` · 006dad20 · legions 00664190 — ac2-script glue::options / examination (Legions layout and efun switch)
- [ ] `UISaveLocations::AddLocation` · 006dad30 · legions 006641a0

### HashTable<unsigned_long,PHashTable<unsigned_long,UILocationData>_>
- [ ] `HashTable<unsigned_long,PHashTable<unsigned_long,UILocationData>_>::operator=` · 006dae20 · legions 00664290

### UISaveLocations
- [x] `UISaveLocations::UISaveLocations` · 006daf40 · legions 006643b0 — ac2-script glue::options / examination (Legions layout and efun switch)
- [~] `UISaveLocations::operator=` · 006daf80 · legions 006643f0 — ac2-script native::options (ShortcutInfo keeps its packed datum only; unversioned profiles upgrade before packing; inner UI tables copied as is)
- [x] `UISaveLocations::UISaveLocations` · 006dafd0 · legions 00664440 — ac2-script glue::options / examination (Legions layout and efun switch)
