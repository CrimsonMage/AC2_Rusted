# ac2-ui: UI framework, input maps, IME, game panels, maps

2247 retail functions: 1244 done, 341 partial, 168 to do, 494 skipped.
Legend and workflow: [README](README.md).

## `ac2/client/src/ui/gameplay/gm3dtextui.cpp.c` (29/53)

### BBox
- [x] `BBox::AdjustBBox` · 00426720 · legions 00426510 — ac2-physics objects.rs: static-object collision dispatch, containment radius, shadow box

### Box2D
- [x] `Box2D::Box2D` · 004267a0 · legions 00426590 — ac2-ui name_plates.rs + ac2-client (Command::SetShowObjectNames, IFun 0xB400CE)
- [-] `Box2D::Box2D` · 004267d0 · legions 005a35f0 — ctor/dtor/operator= of plain data; Rust struct construction and Drop replace it
- [x] `Box2D::Collide` · 00426800 · legions 004265c0 — ac2-ui name_plates.rs + ac2-client (Command::SetShowObjectNames, IFun 0xB400CE)

### gm3DTextUI
- [x] `gm3DTextUI::ListenToMessage` · 004268a0 · legions 00426660 — ac2-ui name_plates.rs + ac2-client (Command::SetShowObjectNames, IFun 0xB400CE)
- [ ] `gm3DTextUI::RecvNotice_ItemUpdate` · 00426920 — Legions has it: vtable slot 90 of gm3DTextUI at 0x004266e0 (vtable audit); was skip: no Legions 7160 counterpart (no legions_address in the index
- [x] `gm3DTextUI::ShowReticle` · 00426930 · legions 004266f0 — ac2-ui reticle.rs + ac2-gpu ui.rs: the targeting reticle
- [~] `gm3DTextUI::CheckName` · 00426970 · legions 00426730 — ac2-ui reticle.rs; ac2-client SelectionFrame::name_info (display_name(quantity) = GetName(quantity)) rendered in view/overlays.rs. Not ported: the off-screen indicator; name plates still use the plain name
- [x] `gm3DTextUI::DoStuffMeter` · 00426ae0 · legions 004268a0 — ac2-ui reticle.rs
- [x] `gm3DTextUI::MakeNewData` · 00426b70 · legions 00426930 — ac2-ui name_plates.rs + ac2-client (Command::SetShowObjectNames, IFun 0xB400CE)

### BBox
- [-] `BBox::BBox` · 00426d00 · legions 00426ac0 — ctor/dtor/operator= of plain data; Rust struct construction and Drop replace it
- [-] `BBox::Pack` · 00426d40 — plain-data stream writer (BBox::Pack); client reads, never writes, this record

### Position
- [-] `Position::Position` · 00426d50 · legions 00426b10 — ctor/dtor/operator= of plain data; Rust struct construction and Drop replace it

### gm3DTextUI
- [~] `gm3DTextUI::InitReticle` · 00426d90 · legions 00426b50 — ac2-ui reticle.rs: the meters (DoStuffMeter, selection info) and the name are ported; not ported: the off-screen indicator (hidden), the radar-look colour of the selection name (CheckName's InqRadarLook font colour, needs the selection look passed from hud_state) and the name's quantity (GetName(quantity))
- [x] `gm3DTextUI::RecvNotice_UpdateSelectionInfo` · 00426ee0 · legions 00426ca0 — ac2-ui reticle.rs
- [x] `gm3DTextUI::PlaceHere` · 00426f60 · legions 00426d20 — ac2-ui name_plates.rs + ac2-client (Command::SetShowObjectNames, IFun 0xB400CE)

### _STL::_Deque_iterator<BBox,_STL::_Nonconst_traits<BBox>_>
- [-] `_STL::_Deque_iterator<BBox,_STL::_Nonconst_traits<BBox>_>::operator-` · 00427090 · legions 00426eb0 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them
- [-] `_STL::_Deque_iterator<BBox,_STL::_Nonconst_traits<BBox>_>::operator+` · 004270f0 · legions 00426e50 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### SmartArray<Box2D>
- [-] `SmartArray<Box2D>::AddToEnd` · 00427460 · legions 00427220 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### _STL::deque<BBox,_STL::allocator<BBox>_>
- [-] `_STL::deque<BBox,_STL::allocator<BBox>_>::~deque<BBox,_STL::allocator<BBox>_>` · 004280e0 · legions 00427ea0 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### gm3DTextUI::NameTextInfo
- [x] `gm3DTextUI::NameTextInfo::NameTextInfo` · 00428e00 · legions 00428bc0 — ac2-ui name_plates.rs + ac2-client (Command::SetShowObjectNames, IFun 0xB400CE)

### gm3DTextUI::DamageTextInfo
- [x] `gm3DTextUI::DamageTextInfo::DamageTextInfo` · 00428e30 — ac2-ui damage_text.rs Damage entry (Vec; element, object, start, duration, boxes)

### gm3DTextUI
- [x] `gm3DTextUI::AverageBoundingBox` · 00428e60 · legions 00428c20 — ac2-ui reticle.rs + ac2-gpu ui.rs: the targeting reticle
- [x] `gm3DTextUI::ConvertObjectToWindow` · 004291a0 · legions 00428f60 — ac2-ui reticle.rs + ac2-gpu ui.rs: the targeting reticle
- [~] `gm3DTextUI::UpdateReticleInternal` · 00429320 · legions 004290e0 — ac2-ui reticle.rs: the meters (DoStuffMeter, selection info) and the name are ported; not ported: the off-screen indicator (hidden), the radar-look colour of the selection name (CheckName's InqRadarLook font colour, needs the selection look passed from hud_state) and the name's quantity (GetName(quantity))
- [x] `gm3DTextUI::CalcScreenPosition` · 00429ab0 · legions 00429870 — ac2-ui name_plates.rs + ac2-client (Command::SetShowObjectNames, IFun 0xB400CE)

### _STL::deque<BBox,_STL::allocator<BBox>_>
- [-] `_STL::deque<BBox,_STL::allocator<BBox>_>::operator=` · 00429bd0 · legions 00429990 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### gm3DTextUI
- [x] `gm3DTextUI::UpdateNames` · 00429e20 · legions 00429be0 — ac2-ui name_plates.rs + ac2-client (Command::SetShowObjectNames, IFun 0xB400CE)
- [x] `gm3DTextUI::UpdateReticle` · 0042a1d0 · legions 00429f90 — ac2-ui reticle.rs + ac2-gpu ui.rs: the targeting reticle

### gm3DTextUI::DamageTextInfo
- [x] `gm3DTextUI::DamageTextInfo::operator=` · 0042a290 · legions 0042a050 — ac2-ui damage_text.rs Damage entry (Vec; element, object, start, duration, boxes)

### SmartArray<gm3DTextUI::DamageTextInfo>
- [x] `SmartArray<gm3DTextUI::DamageTextInfo>::RemoveOrderedByIndex` · 0042a4f0 · legions 0042a2b0 — ac2-ui damage_text.rs and name_plates.rs: the arrays are Vecs kept in order (remove keeps the order, add appends)

### gm3DTextUI
- [x] `gm3DTextUI::UpdateDamage` · 0042a560 · legions 0042a320 — ac2-ui damage_text.rs update: progress, rise by height*FloatyTextTravel*progress (selection offset), fade 1-progress, CalcScreenPosition, placement via placement.rs place_box
- [x] `gm3DTextUI::ClearNames` · 0042a980 · legions 0042a740 — ac2-ui name_plates.rs + ac2-client (Command::SetShowObjectNames, IFun 0xB400CE)
- [x] `gm3DTextUI::RemoveName` · 0042aa40 · legions 0042a800 — ac2-ui name_plates.rs + ac2-client (Command::SetShowObjectNames, IFun 0xB400CE)
- [~] `gm3DTextUI::RecvNotice_BeingDeleted` · 0042ab30 · legions 0042a8f0 — ac2-ui name_plates.rs sync: plates follow the radar looks each frame; the name is the object's name, not GetName(quantity)

### SmartArray<gm3DTextUI::DamageTextInfo>
- [x] `SmartArray<gm3DTextUI::DamageTextInfo>::AddToEnd` · 0042ac30 · legions 0042a9f0 — ac2-ui damage_text.rs and name_plates.rs: the arrays are Vecs kept in order (remove keeps the order, add appends)

### SmartArray<gm3DTextUI::NameTextInfo>
- [x] `SmartArray<gm3DTextUI::NameTextInfo>::AddToEnd` · 0042aca0 · legions 0042aa60 — ac2-ui damage_text.rs and name_plates.rs: the arrays are Vecs kept in order (remove keeps the order, add appends)

### gm3DTextUI
- [x] `gm3DTextUI::CleanUp` · 0042ad00 · legions 0042aac0 — ac2-ui DamageTexts::clear (destroys every damage text); not yet called on a world reset
- [~] `gm3DTextUI::Update` · 0042adf0 · legions 0042abb0 — ac2-ui hud_state.rs update_reticle, update_damage, update_names (per frame from ac2-client view.rs). Order differs: retail UpdateReticle, UpdateDamage, UpdateNames; view.rs runs names before damage (no shared state, not changed). Off-screen indicator, radar-look colour and quantity not ported
- [~] `gm3DTextUI::AddName` · 0042ae40 · legions 0042ac00 — ac2-ui name_plates.rs sync: plates follow the radar looks each frame; the name is the object's name, not GetName(quantity)
- [~] `gm3DTextUI::RecvNotice_CreateObject` · 0042b0c0 — ac2-ui name_plates.rs sync: plates follow the radar looks each frame; the name is the object's name, not GetName(quantity)
- [x] `gm3DTextUI::RecvNotice_Do3DDamage` · 0042b150 — ac2-ui damage_text.rs (DamageTextTemplate, blob colour and scale, object and start time); ac2-client hud_player routes CM_Physics::SendNotice_Do3DDamage. Legions keeps UpdateDamage and the array, and its WLib calls efun 0x8D0001; handler body from admin (no Legions match)
- [ ] `gm3DTextUI::RecvNotice_PrecludeFromTop` · 0042b340 — Legions has it: gm3DTextUI notice vtable slot 141 at 0x0042b160 (or slot 142 at 0x0042b100; both match in size, pick by reading). Was skip
- [~] `gm3DTextUI::RecvNotice_PrecludeFromBottom` · 0042b3a0 — ac2-ui toolbar.rs show_bar via ac2-client show_toolbar sets reticle.precluded_bottom; the update runs on the next frame, not inside the notice
- [-] `gm3DTextUI::~gm3DTextUI` · 0042b400 · legions 0042b1c0
- [~] `gm3DTextUI::Init` · 0042b510 · legions 0042b2d0 — ac2-ui hud.rs Reticle/NamePlates/DamageTexts::create and ac2-client client_radar.rs (the radar ranges, squared in Init); the object notice registration (0x446, 0x454, 0x443, 0x1040c) is replaced by the per-frame sync in name_plates.rs
- [x] `gm3DTextUI::InitNames` · 0042b610 · legions 0042b3d0 — ac2-ui name_plates.rs + ac2-client (Command::SetShowObjectNames, IFun 0xB400CE)
- [~] `gm3DTextUI::RecvNotice_ChangeRadarLook` · 0042b6a0 — ac2-ui name_plates.rs sync: plates follow the radar looks each frame; the name is the object's name, not GetName(quantity)
- [~] `gm3DTextUI::RecvNotice_SelectionChanged` · 0042b730 — ac2-ui reticle.rs selection_changed and selection_info via hud_state.rs update_reticle; the radar-look colour of the name and the quantity suffix (CheckName) and the off-screen indicator are not ported
- [x] `gm3DTextUI::ToggleNames` · 0042b7e0 · legions 0042b5a0 — ac2-ui name_plates.rs + ac2-client (Command::SetShowObjectNames, IFun 0xB400CE)
- [~] `gm3DTextUI::gm3DTextUI` · 0042b840 · legions 0042b600 — ac2-ui hud.rs Reticle/NamePlates/DamageTexts::create and ac2-client client_radar.rs (the radar ranges, squared in Init); the object notice registration (0x446, 0x454, 0x443, 0x1040c) is replaced by the per-frame sync in name_plates.rs
- [x] `gm3DTextUI::ListenToMessage` · 0042ba40 · legions 0042b800 — ac2-ui name_plates.rs + ac2-client (Command::SetShowObjectNames, IFun 0xB400CE)
- [x] `gm3DTextUI::RecvNotice_ReloadOptions` · 0042ba70 — ac2-ui name_plates.rs + ac2-client (Command::SetShowObjectNames, IFun 0xB400CE)

## `ac2/client/src/ui/gameplay/gmbookui.cpp.c` (10/19)

### gmBookUI
- [x] `gmBookUI::CreateManagedElements` · 0041eaa0 · legions 0041e8c0 — ac2-ui book.rs BookUi::create: the Book template and its children, made with the HUD and hidden (retail makes them on each show)
- [x] `gmBookUI::Show` · 0041eb50 · legions 0041e970 — ac2-ui book.rs BookUi::show; book_pass runs in Hud::frame on the use notice (the client's hud_book.rs is gone)
- [x] `gmBookUI::SynchSliderBar` · 0041ebc0 · legions 0041e9e0 — ac2-ui book.rs BookUi::sync_slider (BookSliderBar x position)
- [x] `gmBookUI::PreviousPage` · 0041ec50 · legions 0041ea70 — ac2-ui book.rs BookUi::previous_page / next_page; ac2-ui book_tests.rs a_long_book_pages_and_the_page_turns_follow_the_book
- [x] `gmBookUI::NextPage` · 0041ec70 · legions 0041ea90 — ac2-ui book.rs BookUi::previous_page / next_page; ac2-ui book_tests.rs a_long_book_pages_and_the_page_turns_follow_the_book
- [~] `gmBookUI::SetPageFromSlider` · 0041ec90 · legions 0041eab0 — ac2-ui book.rs BookUi::set_page_from_slider: rounds half up as ftol2 does; a page 0 is clamped to 1 (retail can set 0)
- [x] `gmBookUI::SetMode` · 0041ed20 · legions 0041eb40 — ac2-ui book.rs BookUi::set_mode: BookClientArea, the text and header states; BookBottom shown with the controls
- [x] `gmBookUI::Reset` · 0041ed90 · legions 0041ebb0 — ac2-ui book.rs BookUi::reset (page 1 of 1, texts and foreground image cleared, Dialog state)

### PStringBase<unsigned_short>
- [-] `PStringBase<unsigned_short>::operator[]` · 0041edf0 · legions 0041ec10 — PStringBase iterator and operator[]: the Rust port keeps Strings and indexes them by char

### gmBookUI
- [~] `gmBookUI::gmBookUI` · 0041ee70 · legions 0041ec90 — ac2-ui book.rs BookUi::create; the notice registration is the frame's event list (ac2-client hud_book.rs), not a handler list
- [-] `gmBookUI::~gmBookUI` · 0041ef90 · legions 0041edb0 — Rust drop: the tree owns the book's elements, and the frame's events need no listener removal
- [~] `gmBookUI::UpdateBook` · 0041f010 · legions 0041ee30 — ac2-ui book.rs BookUi::update_book: page label and slider; the view offset is the box height per page (the decompile's field difference plus one)

### PStringBaseIter<unsigned_short>
- [-] `PStringBaseIter<unsigned_short>::GetStrLen` · 0041f0d0 · legions 0041eeb0 — PStringBase iterator and operator[]: the Rust port keeps Strings and indexes them by char

### gmBookUI
- [x] `gmBookUI::AddNewlines` · 0041f480 · legions 0041f260 — ac2-ui book.rs paginate: the padding newlines (AddNewlines as repeated newlines)
- [x] `gmBookUI::TruncateText` · 0041f510 · legions 0041f2f0 — ac2-ui book.rs truncate_to_width: the text plus L" " (a space, Legions 0x9cbaf0, not an ellipsis) is measured; wider than a positive limit, characters come off until it fits and the text keeps the space
- [~] `gmBookUI::ParseText` · 0041f7c0 · legions 0041f5a0 — ac2-ui book.rs paginate and segments: <BR> pages, <H>..</H> headers, pages padded to lines_per_page; a tail after the last <BR> is paginated (retail's flush does not); PStringBase substring end unresolved
- [~] `gmBookUI::RecvNotice_UseBook` · 0041ff30 · legions 0041fd30 — ac2-ui book.rs book_pass runs the use notices and messages over the frame; ac2-client hud_book.rs maps help requests to KeyStone::OpenHelp, copy dropped (no clipboard device in the client)
- [x] `gmBookUI::SendBookToClipboard` · 00420010 · legions 0041fe10 — ac2-ui book.rs book_pass sends BookUi::clipboard_text to the tree's clipboard (UiTree::send_string_to_clipboard, the OS clipboard in the client); test book_tests.rs
- [~] `gmBookUI::ListenToMessage` · 00420320 · legions 00420120 — ac2-ui book.rs book_pass runs the use notices and messages over the frame; ac2-client hud_book.rs maps help requests to KeyStone::OpenHelp, copy dropped (no clipboard device in the client)

## `ac2/client/src/ui/gameplay/gmcameraui.cpp.c` (4/7)

### gmCameraUI
- [-] `gmCameraUI::~gmCameraUI` · 0042baa0 · legions 0042b860 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own
- [x] `gmCameraUI::ListenToMessage` · 0042bb00 · legions 0042b8c0 — ac2-ui mouse.rs / button.rs: UI hit tests, presses and buttons
- [~] `gmCameraUI::SendKey` · 0042bb50 — ac2-ui camera_ui.rs CameraUi::action (the four arrow UI actions re-sent as turn and move actions with the action as its key, device 1; Legions has SendKey inlined at the end of gmCameraUI::ListenToMessage 0x42BAA0). Not ported: the guard that drops the key when either IUIManager current-object query (vtable +0xb4, +0xb8) is true
- [x] `gmCameraUI::UpdateCamera` · 0042bbb0 · legions 0042b970 — ac2-ui camera_ui.rs (no UI element hit test: the wheel always reaches the world)
- [-] `gmCameraUI::gmCameraUI` · 0042bc20 · legions 0042b9e0 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own
- [x] `gmCameraUI::ListenToMessage` · 0042bce0 · legions 0042baa0 — ac2-ui camera_ui.rs (no UI element hit test: the wheel always reaches the world)
- [x] `gmCameraUI::RecvNotice_UpdateCanMouseLook` · 0042bf50 — ac2-ui camera_ui.rs CameraUi::recv_notice_update_can_mouse_look (m_canMouseLook is a count: +1 for 0, -1 for non-zero; UpdateCamera follows) via controls.rs Controls::update_can_mouse_look; senders: DialogFactory (MakeDialog 0, DialogDone and a queued CloseDialog 1; dialog.rs take_mouse_look_notices) and the story slot (gameplay_story.rs MouseLook); ac2-client bin/ac2/main.rs passes both on. The Legions handler is an undecompiled vtable slot (ctor registers notice 0x455)

## `ac2/client/src/ui/gameplay/gmexternalcontainerui.cpp.c` (7/8)

### gmExternalContainerUI
- [-] `gmExternalContainerUI::~gmExternalContainerUI` · 00420400 · legions 00420200 — teardown: the HUD tree is dropped whole
- [x] `gmExternalContainerUI::ListenToMessage` · 00420460 · legions 00420290 — ac2-ui external_container.rs
- [x] `gmExternalContainerUI::RecvNotice_CreateObject` · 00420520 — ac2-ui: the panel hears every object's notices (no per-object registration)
- [x] `gmExternalContainerUI::RecvNotice_BeingDeleted` · 00420570 — ac2-ui external_container.rs
- [x] `gmExternalContainerUI::PositionWindow` · 004205e0 · legions 00420410 — ac2-ui external_container.rs
- [x] `gmExternalContainerUI::RecvNotice_ContainerStateUpdate` · 00420860 · legions 00420690 — ac2-ui external_container.rs
- [x] `gmExternalContainerUI::Init` · 004208d0 · legions 00420700 — ac2-ui: the panel hears every object's notices (no per-object registration)
- [x] `gmExternalContainerUI::gmExternalContainerUI` · 00420980 · legions 004207b0 — ac2-ui external_container.rs

## `ac2/client/src/ui/gameplay/gmgameplayui.cpp.c` (25/58)

### gmGamePlayUI
- [~] `gmGamePlayUI::Show` · 00407910 · legions 00407620 — ac2-client hud_view.rs set_gameplay_shown: status bar, radar and toolbar bar. Legions UIFramework::Show re-applies each child's own m_shown (disassembly 0x005BE390), it does not pass the new flag down; cascade unchanged
- [x] `gmGamePlayUI::SetErrorMsg` · 00407940 · legions 00407650 — ac2-client view.rs ClientView::set_error_msg -> chat_display(ac2_ui::ERROR_TEXT_TYPE); no gameplay caller in retail (only the ECM_Admin notice), so no test reaches it
- [x] `gmGamePlayUI::RecvNotice_EndCharacterSession` · 00407980 — ac2-ui game_dialogs.rs logout_key (the Leave question's yes)
- [x] `gmGamePlayUI::RecvNotice_Logoff` · 00407990 — ac2-ui HudRequest::Quit: UploadGameOptions, then the window closes (ac2-client hud_request.rs, main.rs quitting)
- [x] `gmGamePlayUI::RecvNotice_NextFrameDrawn` · 004079b0 — OnlineSession::scene_ready (SceneRenderingComplete) on the frame drawn after each PlayerReadyToEnterWorld
- [ ] `gmGamePlayUI::RecvNotice_BeingDeleted` · 004079c0 — Legions has it: gmGamePlayUI notice vtable slot 10 at 0x004076d0 (vtable audit)
- [x] `gmGamePlayUI::MakePleaseWaitDialog` · 004079e0 · legions 004076f0 — ac2-ui game_dialogs.rs end_session raises it (ID_Wait_PleaseWait, wait type); ac2-client hud_game_dialogs.rs game_dialog_logout runs it on the logout key; logout_hud_7160 and the game_dialogs unit tests
- [x] `gmGamePlayUI::MakeResurrectDialog` · 00407a40 · legions 00407750 — ac2-ui death.rs: MakeResurrectDialog's prompt (ID_ResurrectionPrompt with the resurrecter's name, 0x4100000a) or the death dialog (0x41000018), closing the dialog up first (DeathPanel::offer / death dialog); tested in death tests

### PlayingSoundContext
- [x] `PlayingSoundContext::Stop` · 00407b80 · legions 004078b0 — ac2-audio/ac2-client: 3D emitters and FX sounds

### gmGamePlayUI
- [x] `gmGamePlayUI::RecvNotice_PlayerTeleportStart` · 00407bf0 — ac2-ui gameplay_story.rs GameplayStory::player_teleport_start / portal_space (the gate: Exit iff weenie and client ready, else Enter; the teleport start clears _clientReady); ac2-client client_teleport.rs portal_call; gameplay_story_tests (the teleport and leave tests), portal_7160 (the teleport places and exits)
- [x] `gmGamePlayUI::RecvNotice_PortalSpace` · 00407c30 — ac2-ui gameplay_story.rs GameplayStory::player_teleport_start / portal_space (the gate: Exit iff weenie and client ready, else Enter; the teleport start clears _clientReady); ac2-client client_teleport.rs portal_call; gameplay_story_tests (the teleport and leave tests), portal_7160 (the teleport places and exits)
- [~] `gmGamePlayUI::RecvNotice_UpdateDeathState` · 00407c80 — ac2-ui death.rs (DeathPanel::notice: the death dialog up or closed); ac2-client hud_death.rs; not reproduced: the vtable call at +0x20 (the gameplay UI's show toggle, target not named in the export)
- [x] `gmGamePlayUI::RecvNotice_ResurrectionRequestedByOther` · 00407cf0 — ac2-ui death.rs (DeathPanel::recv_resurrection_requested, the prompt); ac2-client hud_death.rs renders the notice; death_panel_7160 (offer layout)
- [x] `gmGamePlayUI::RecvNotice_CloseConfirmationDialog` · 00407d00 · legions 00407a30 — ac2-ui death.rs (DeathPanel::recv_close_confirmation: yes respawns, no clears and makes the death dialog); ac2-client hud_game_dialogs.rs sends Command::ClearResurrectionRequest (IFun 0xB400F7) on no; death_panel_7160, combat_7160 (declined IFun)
- [x] `gmGamePlayUI::RecvNotice_CloseResurrectDialog` · 00407d70 — ac2-ui death.rs (DeathPanel::take_resurrect: the death dialog's button asks the respawn); death_panel_7160
- [x] `gmGamePlayUI::ChildKeyPress` · 00407f70 · legions 00407ca0 — ac2-ui gameplay_keys.rs (child_for_key: keys 0x41000008 toolbar, 0x41000009 statusbar, 0x4100000B radar); ac2-client hud_view.rs gameplay_keys / set_child_shown; gameplay_keys_7160 (each_child_key_toggles_its_child)
- [~] `gmGamePlayUI::Update` · 00408020 · legions 00407ce0 — per-frame trigger: WinMain's frame loop calls slot +0x3c on the app (Legions 0x4016E7), and Update is slot +0x3c of gmGamePlayUI's primary vtable 0x9C7550 (verified). Not ported: the app gate at vtable +0x18 is unresolved, so GameplayDialogs::update's queued character-selection mode and _endSession stay unfired; the leave path is bin/ac2 back_to_character_selection
- [x] `gmGamePlayUI::RecvNotice_DisplayMessage` · 00408080 — retail handler registered by gmGamePlayUI::SetupChildren (Init at login); boxes raised in the world wait in HudView::display_backlog until the HUD dialogs exist (tests/message_box_hud_7160.rs)
- [~] `gmGamePlayUI::SetupChildren` · 004080c0 · legions 00407d80 — ac2-ui: no transitions, timed media, mouse, listeners or dirty rects; A8R8G8B8 blits only; HUD roots from SetupChildren's radar/selection/statusbar/toolbar; the keypress registrations (GameplayChildInfo: radar 0x4100000B, statusbar 0x41000009, toolbar 0x41000008, the rest 0) are in ac2-ui gameplay_keys.rs
- [~] `gmGamePlayUI::DoPleaseWaitCancel` · 004084a0 · legions 00408160 — ac2-ui game_dialogs.rs recv_close_wait_dialog clears the wait context; the app's QueueUIMode(characterselection) is ac2-client bin/ac2 back_to_character_selection after CharExitGame (pregame resume_at_character_selection), not from the wait close: no HUD path calls recv_close_wait_dialog yet
- [~] `gmGamePlayUI::Init` · 00408500 · legions 004081c0 — ac2-client hud_game_dialogs.rs game_dialog_notices (LoginFinished -> GameplayDialogs::init closes the please-wait dialog); ac2-ui game_dialogs.rs init. SetupChildren's HUD is built at entry and the notice registrations are implicit in the event flow
- [~] `gmGamePlayUI::RecvNotice_CloseWaitDialog` · 00408640 — ac2-ui game_dialogs.rs recv_close_wait_dialog clears the wait context; the app's QueueUIMode(characterselection) is ac2-client bin/ac2 back_to_character_selection after CharExitGame (pregame resume_at_character_selection), not from the wait close: no HUD path calls recv_close_wait_dialog yet

### HashTable<unsigned_long,int>
- [-] `HashTable<unsigned_long,int>::~HashTable<unsigned_long,int>` · 00408660 · legions 00408320 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### gmGamePlayUI
- [x] `gmGamePlayUI::RecvNotice_LoginFinished` · 00408840 · legions 00408580 — ac2-client hud_game_dialogs.rs game_dialog_notices (Init's wait close); client_teleport.rs login_finished (PortalSpace(0)); game_dialogs.rs init tests
- [x] `gmGamePlayUI::RecvNotice_CloseMessageDialog` · 00408860 — ac2-ui game_dialogs.rs (GameplayDialogs: message contexts, display_message, close_message); ac2-client hud_game_dialogs.rs wires the DisplayMessage notice and the OK answer; game_dialogs_7160 (message box text and OK)
- [~] `gmGamePlayUI::CleanupChildren` · 00408890 · legions 004085d0 — ac2-client: the HUD's children are dropped with the view when the character leaves (bin/ac2 back_to_character_selection); the per-child destroy and the list flush are not run

### PHashTable<unsigned_long,UILocationData>
- [-] `PHashTable<unsigned_long,UILocationData>::~PHashTable<unsigned_long,UILocationData>` · 004088f0 · legions 00408630 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### PList<gmSceneInfo>
- [-] `PList<gmSceneInfo>::PList<gmSceneInfo>` · 00408a20 · legions 00408760 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them
- [-] `PList<gmSceneInfo>::Pack` · 00408a60 · legions 004087a0 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them
- [-] `PList<gmSceneInfo>::UnPack` · 00408ab0 · legions 00408804 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### gmGamePlayUI
- [~] `gmGamePlayUI::gmGamePlayUI` · 00408c60 · legions 004089a0 — ac2-ui gameplay_story.rs GameplayStory::new (_weenieReady, _clientReady, _playingPortalScene = 0); the constructor's SmartBox::EnterPortalSpace(1) when both are not ready is not modelled (the login's fade is the PlayerReady and LoginFinished gate calls)

### UIFramework
- [ ] `UIFramework::Shown` · 00408e60 · legions 00408ba0 — needs port (check): UIFramework::Shown behaviour; implemented area ac2-ui hud.rs but uncited

### gmGamePlayUI
- [-] `gmGamePlayUI::Create` · 00408e70 — admin 7148 factory (new gmGamePlayUI); Legions 7160 has no counterpart; the HUD is constructed directly by ac2-client [vtable-checked: not a slot of any located Legions vtable]
- [~] `gmGamePlayUI::EndSession` · 00408e90 · legions 00408bd0 — ac2-ui gameplay_story.rs (story slot, portal flag, teardown, PlayScenesDone); scene list from script natives and the HUD drawing pending
- [~] `gmGamePlayUI::RecvNotice_FinishPlaySequence` · 00408f50 · legions 00408c90 — ac2-ui gameplay_story.rs (story slot, portal flag, teardown, PlayScenesDone); scene list from script natives and the HUD drawing pending
- [~] `gmGamePlayUI::RecvNotice_PlaySequence` · 00408fa0 · legions 00408ce0 — ac2-ui gameplay_story.rs (story slot, portal flag, teardown, PlayScenesDone); scene list from script natives and the HUD drawing pending
- [~] `gmGamePlayUI::CheckPortalScene` · 00409000 · legions 00408d40 — ac2-ui gameplay_story.rs (story slot, portal flag, teardown, PlayScenesDone); scene list from script natives and the HUD drawing pending

### PList<gmSceneInfo>
- [-] `PList<gmSceneInfo>::PList<gmSceneInfo>` · 004090e0 · legions 00408e20 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### gmGamePlayUI
- [~] `gmGamePlayUI::~gmGamePlayUI` · 00409110 · legions 00408e50 — ac2-ui game_dialogs.rs GameplayDialogs::destroy (EndSession(0) quiet, wait and message dialogs closed); the view drop closes the rest; CleanupChildren and the global notice unregistering are not run
- [-] `gmGamePlayUI::Register` · 00409250 · legions 00408f90 — UIFlow::RegisterFrameworkClass sets the mode's factory; the Rust HUD is built by ac2-client's view when the world enters, so there is no factory table
- [x] `gmGamePlayUI::RecvNotice_PlayerReadyToEnterWorld` · 00409270 — tail call is the portal gate (SmartBox::EnterPortalSpace / ExitPortalSpace with bInstantly 0: Exit once weenie and client are ready, else Enter), ported as GameplayStory::player_ready_to_enter_world -> Client::player_ready_to_enter_world (client_teleport.rs)
- [~] `gmGamePlayUI::RecvNotice_PortalScene` · 00409330 — ac2-ui gameplay_story.rs (story slot, portal flag, teardown, PlayScenesDone); scene list from script natives and the HUD drawing pending

### PHashTable<unsigned_long,UILocationData>
- [x] `PHashTable<unsigned_long,UILocationData>::StreamPack` · 004093a0 — ac2-net protocol::native pack+unpack (Legions layouts; WState graphs round-trip byte-exact); ac2-script still decodes its own copy until it switches

### PHashTable<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>
- [-] `PHashTable<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>::StreamPack` · 004095e0 · legions 00409320 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### IntrusiveHashData<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>_*>
- [-] `IntrusiveHashData<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>_*>::IntrusiveHashData<PStringBase<unsigned_short>,HashTableData<PStringBase<unsigned_short>,PStringBase<unsigned_short>_>_*>` · 004098b0 · legions 004095f0 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### (free functions)
- [ ] `STREAMPACK` · 004098e0 · legions 00409650 — no call site in the Legions wlib (scan: host::tests::efun_coverage); lower priority

### PStringBase<unsigned_short>
- [x] `PStringBase<unsigned_short>::GetPackSize` · 00409960 · legions 004096d0
- [x] `PStringBase<unsigned_short>::Pack` · 00409990 · legions 00409700
- [x] `PStringBase<unsigned_short>::UnPack` · 00409be0 · legions 00409950 — ac2-core pstring.rs: the DAT string descramble (ac2-ui, ac2-db use it)

### PHashTable<unsigned_long,float>
- [x] `PHashTable<unsigned_long,float>::StreamPack` · 00409e00

### PHashTable<unsigned_long,PHashTable<unsigned_long,UILocationData>_>
- [-] `PHashTable<unsigned_long,PHashTable<unsigned_long,UILocationData>_>::~PHashTable<unsigned_long,PHashTable<unsigned_long,UILocationData>_>` · 0040a190 · legions 0040a020 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them
- [x] `PHashTable<unsigned_long,PHashTable<unsigned_long,UILocationData>_>::StreamPack` · 0040a1f0 · legions 0040a080 — ac2-net protocol::native pack+unpack (Legions layouts; WState graphs round-trip byte-exact); ac2-script still decodes its own copy until it switches

### PHashTable<unsigned_long,UILocationData>
- [-] `PHashTable<unsigned_long,UILocationData>::PHashTable<unsigned_long,UILocationData>` · 0040a440 · legions 0040a2c0 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### UISaveLocations
- [-] `UISaveLocations::~UISaveLocations` · 0040a540 · legions 0040a3c0 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own

### GameplayOptionsProfile
- [ ] `GameplayOptionsProfile::~GameplayOptionsProfile` · 0040a590 · legions 0040a410 — needs port: GameplayOptionsProfile::~GameplayOptionsProfile lifecycle (teardown/creation)

### gmGamePlayUI
- [x] `gmGamePlayUI::RecvNotice_SaveLocationsChanged` · 0040a790 — ac2-client client_locations.rs: ReloadOptions and the refresh apply the profile's locations (hud_player.rs); a moved window sends SaveWindowLocations: IFun 0xB4007B then 0xB400DB (WM_Player 0x32); AddLocation bucket order in ac2-script native/options.rs; tests window_locations_7160.rs
- [~] `gmGamePlayUI::ListenToMessage` · 0040a810 · legions 0040a6d0 — ac2-ui game_dialogs.rs logout_key (0x4100000e); ac2-client hud_view.rs gameplay_keys (child keys, 0x4100000C via gameplay_show_key, 0x4100000D via take_use_key -> Command::UseSelection); 0xffff0005 and 0xffff0007 notices not here
- [x] `gmGamePlayUI::RecvNotice_ReloadOptions` · 0040a960 — ac2-client client_locations.rs: ReloadOptions and the refresh apply the profile's locations (hud_player.rs); a moved window sends SaveWindowLocations: IFun 0xB4007B then 0xB400DB (WM_Player 0x32); AddLocation bucket order in ac2-script native/options.rs; tests window_locations_7160.rs

## `ac2/client/src/ui/gameplay/gmingredientui.cpp.c` (25/39)

### gmCObjectMaint
- [-] `gmCObjectMaint::GetWeenieObject` · 00415870 · legions 00415660 — gmCObjectMaint::GetWeenieObject: the session's object table serves it (client hud_ingredient.rs)

### gmIngredientUI
- [~] `gmIngredientUI::DestroyManagedElements` · 00415890 · legions 00415680 — ac2-ui ingredient.rs close: the slots' objects leave and the list flushes; the panel's root is kept (the 15 s dormancy destroy is not modelled)
- [x] `gmIngredientUI::UpdateCraftButton` · 00415900 · legions 004156f0 — ac2-ui ingredient.rs update_craft_button: pending only ghosts; mounted / spinner out of range / difficulty past the cutoff ghost with the ID_Craft_CraftButton_Disabled caption on CraftButtonText and the ID_Craft_TT_Mounted / InvalidSpinner / TooHard tooltip; otherwise a ghosted button gets ID_Craft_CraftButton back and no tooltip

### (free functions)
- [x] `gmIngredientUI_DDCB` · 00415b70 — ac2-ui drag.rs DragCheck::IngredientDrop on the ingredient list (CatchDroppedItem asks it of the container): only an item (ShortcutType 0x40000002) from a container (FromID set); a refusal goes to the drag's owner as a drop outside

### gmIngredientUI
- [x] `gmIngredientUI::UpdateCost` · 00415be0 · legions 004159d0 — ac2-ui ingredient.rs update_cost: ID_Craft_Cost with the cost as value, on CostField (ingredient_ui_7160 asserts the text)
- [x] `gmIngredientUI::Scale_` · 00415c50 · legions 00415a40 — ac2-ui ingredient.rs scale: Scale_ in f64, clamped to its range, mapped onto the scaled range
- [x] `gmIngredientUI::CreateManagedElements` · 00415cd0 · legions 00415ac0 — ac2-ui ingredient.rs create_managed: root RealIngredientField (Legions DAT_00a46a44), RecipeLabel, RecipeMiscText, CostField, IngredientListBox with the drop check, spinner field and buttons, AutoPopulateButton, CraftButton and CraftButtonText
- [~] `gmIngredientUI::Show` · 00415e60 · legions 00415c50 — ac2-ui ingredient.rs open/close: shown and hidden with the root; the timed dormancy is not modelled
- [x] `gmIngredientUI::RecvNotice_UpdateMountState` · 00415ee0 · legions 00415cd0 — ac2-ui ingredient.rs mount_state: mounted, the craft button ghosts
- [x] `gmIngredientUI::AutoSizeIngredientUI` · 00415f10 · legions 00415d00 — ac2-ui ingredient.rs auto_size: min_width, min_height and max_height of the panel, plus one template row (box y1-y0+1) for each ingredient past three, capped at max_height; ingredient_ui_7160 bounds the height
- [x] `gmIngredientUI::SetPending` · 00415fc0 · legions 00415db0 — ac2-ui ingredient.rs set_pending: the slots show pending (dragable off), the craft button ghosts
- [~] `gmIngredientUI::UpdateApparentDifficulty` · 004160b0 · legions 00415ea0 — ac2-ui ingredient.rs update_difficulty: the name colour from Scale_ of the difficulty against the skill, 0..5, or 6 past the cutoff. The colour is not drawn (the tree has no name colour)
- [x] `gmIngredientUI::RecvNotice_UpdateCraftSkill` · 00416280 · legions 00416070 — ac2-ui ingredient.rs craft_skill_updated: the level changes, the craft button and the difficulty follow
- [-] `gmIngredientUI::~gmIngredientUI` · 004164b0 · legions 004162b0 — C++ destructor: the Rust drop and the tree own the panel's state

### HashTable<unsigned_long,InstanceID>
- [-] `HashTable<unsigned_long,InstanceID>::operator=` · 00416640 · legions 004a16f0 — HashTable assignment: the Rust maps are copied by value

### PList<InstanceID>
- [-] `PList<InstanceID>::Pack` · 00416740 · legions 004164e0 — PList pack: no save of the slot list (the panel is rebuilt on open)
- [-] `PList<InstanceID>::UnPack` · 00416790 — PList unpack: no load of the slot list (the panel is rebuilt on open)

### gmIngredientUI
- [~] `gmIngredientUI::MultiOpAddIngredientSlot` · 00416900 · legions 004166c0 — ac2-ui ingredient_slots.rs add_slot: the template at its ordinal's place in the list, with its description and the slot's mask bit (the list is sorted by ordinal)
- [x] `gmIngredientUI::HandleItemRemoval` · 00416ad0 · legions 00416890 — ac2-ui ingredient_slots.rs handle_item_removal: the object leaves its slot; the icon clears and stops being draggable
- [x] `gmIngredientUI::CheckItem` · 00416be0 · legions 004169a0 — ac2-ui ingredient_slots.rs check_item: DoIHaveItem and CheckIngredient at the spinner, else the object leaves its slot
- [x] `gmIngredientUI::RecvNotice_BeingDeleted` · 00416d20 — ac2-ui ingredient.rs object_removed: a deleted object leaves its slot (client hud_ingredient.rs ObjectRemoved)
- [x] `gmIngredientUI::RecvNotice_UpdateInventory` · 00416da0 — ac2-ui ingredient_slots.rs item_changed: an object in a slot is checked again (client hud_ingredient.rs)
- [x] `gmIngredientUI::RecvNotice_UpdateContainer` · 00416df0 · legions 00416bb0 — ac2-ui ingredient_slots.rs item_changed on the container notice's item
- [x] `gmIngredientUI::RecvNotice_ItemUpdate` · 00416e40 · legions 00416c00 — ac2-ui ingredient_slots.rs item_changed on ItemUpdate
- [x] `gmIngredientUI::Init` · 00416ea0 · legions 00416c60 — ac2-ui ingredient.rs open: name in RecipeLabel, the charges line ID_Craft_Charges in RecipeMiscText (hidden without charges), spinner range and value, cost, skill level, slots from InqRecipeIngredients, craft button, difficulty. The spinner starts at last_spinner (the Init decompile overlays two adaptors)
- [x] `gmIngredientUI::MultiOpRemoveIngredientSlot` · 00417290 · legions 00417050 — ac2-ui ingredient_slots.rs remove_slot: the slot's object leaves, the list item and the mask bit go
- [x] `gmIngredientUI::UpdateIngredientSlot` · 004173f0 · legions 004171b0 — ac2-ui ingredient_slots.rs update_slot: the description and the object checked again at the spinner
- [~] `gmIngredientUI::HandleListBoxDrop` · 00417530 · legions 004172f0 — ac2-ui ingredient_slots.rs message: a drop on the list (MSG_STOP_DRAG_ITEM, the list item under the pointer) leaves the object's old slot, the slot goes pending and the add goes out; gmIngredientUI_DDCB now refuses what is not an item from a container (drag_tests). Not tested through the HUD with a real pack drag
- [x] `gmIngredientUI::UpdateIngredients` · 004176c0 · legions 00417480 — ac2-ui ingredient_slots.rs update_ingredients: slots added, kept ones refreshed, the ones no longer asked for removed (the mask as retail)
- [~] `gmIngredientUI::gmIngredientUI` · 00417800 · legions 004175c0 — ac2-ui ingredient.rs IngredientUi::new: the layout kept; the root is made when the panel opens. Not: the notice and message registrations (per object and listeners in Rust, no table)
- [x] `gmIngredientUI::RecvNotice_OpenIngredientPanel` · 00417ac0 · legions 00417880 — ac2-ui ingredient.rs open_notice: the recipe's panel opens unless it is shown and its cost is dynamic (client hud_ingredient.rs)
- [x] `gmIngredientUI::UpdateRecipeFromSpinnerChange` · 00417b00 · legions 004178c0 — ac2-ui ingredient.rs spinner_changed and step_spinner: the spinner text and buttons, the dynamic cost and difficulty, the craft button, the slots (tested: craft spinner and execute)
- [x] `gmIngredientUI::AutoPopulateIngredients` · 00417d20 · legions 00417ae0 — ac2-ui ingredient_slots.rs auto_populate: each inventory object outside the slots goes to the first free slot the recipe auto-populates and takes; the requests go out (tested through the auto-populate button)
- [x] `gmIngredientUI::DoClose` · 00418060 · legions 00417e20 — ac2-ui ingredient.rs close: the slots' objects leave, the list flushes, the panel hides
- [~] `gmIngredientUI::ListenToMessage` · 00418150 · legions 00417f10 — ac2-ui ingredient.rs tick: typed spinner text waits 2.0 s after its last message and is then taken (wcstol base 0, clamped to the range, UpdateRecipeFromSpinnerChange). Not ported: the 15 s dormancy that destroys the panel (DestroyManagedElements)
- [x] `gmIngredientUI::ListenToMessage` · 00418210 · legions 00417fd0 — ac2-ui ingredient_slots.rs message (spinner buttons, typed SpinnerValueField: deactivated takes the text at once, other messages start the 2 s wait, IngredientCloseButton closes unless pending, auto-populate, CraftButton, a drop on the list, an object dragged out of its slot); ignored while the panel is not shown
- [~] `gmIngredientUI::RecvNotice_AddIngredient_Done` · 004185d0 — ac2-ui ingredient_slots.rs add_done: a successful add puts the object in its slot (icon from the object's item display, draggable); a failure frees the slot; a new spinner value follows. Not: the object's notice registration and the icon desc (the item display's icon only)
- [x] `gmIngredientUI::RecvNotice_ExecuteCraft_Done` · 00418780 — ac2-ui ingredient.rs execute_done: the slots go out of pending; no error and no charges closes the panel; charges show ID_Craft_Charges (a named string id) with the value
- [ ] `gmIngredientUI::RecvNotice_CloseConfirmationDialog` · 00418880 — Legions has it: gmIngredientUI notice vtable slot 124 at 0x00418640 (admin slot 123; Legions inserted one notice slot before it; same size and call count)

## `ac2/client/src/ui/gameplay/gmminifellowui.cpp.c` (6/6)

### gmMiniFellowUI
- [x] `gmMiniFellowUI::~gmMiniFellowUI` · 00421c50 · legions 00421ac0 — Rust drop and the HUD tree own the panel and its rows; no notice handlers to unregister
- [x] `gmMiniFellowUI::LocationApplied` · 00421ca0 · legions 0043eb50 — ac2-ui mini_fellow.rs, vitals.rs (tooltips per the numbers' toggle and the hero status); window location save and apply are the generic tree mechanism (mini_fellow_tests the_mini_panel_saves_its_location...)
- [x] `gmMiniFellowUI::ListenToMessage` · 00421cc0 · legions 00421b10 — ac2-ui mini_fellow.rs, vitals.rs (tooltips per the numbers' toggle and the hero status); window location save and apply are the generic tree mechanism (mini_fellow_tests the_mini_panel_saves_its_location...)
- [x] `gmMiniFellowUI::RecvNotice_UpdateFellowship` · 00421d10 · legions 00421b60 — ac2-ui mini_fellow.rs update_fellowship: panel state fellowship, old list goes, new FellowRows over FellowMiniPanelListBox (mini template, Legions 7160 followed; tested)
- [x] `gmMiniFellowUI::RecvNotice_ClearFellowship` · 00421da0 · legions 00421bf0 — ac2-ui mini_fellow.rs clear_fellowship: rows removed, then Show(0) on the mini (disassembly of admin 0x421da0 ends in vt+0x20 with shown=0; tested)
- [x] `gmMiniFellowUI::gmMiniFellowUI` · 00421dd0 · legions 00421c20 — ac2-ui mini_fellow.rs MiniFellowUi::create: FellowMiniPanel root from the gameplay layout, hidden; built in hud.rs

## `ac2/client/src/ui/gameplay/gmoverviewmapui.cpp.c` (1/6)

### gmOverviewMapUI
- [~] `gmOverviewMapUI::CreateManagedElements` · 0043e150 · legions 0043e9a0 — ac2-ui overview_map.rs: label is Panel_Label_OverviewMap, client area OverviewMapClientArea, close OverviewMapCloseButton (admin Tokenize strings); the help button is a generic help button (hud_help.rs ID_Help_OverviewMap). The field is hidden with the HUD rather than made on show and destroyed on hide
- [-] `gmOverviewMapUI::~gmOverviewMapUI` · 0043e1c0 · legions 0043ea10 — C++ destructors and scalar deleting destructors: the Rust drop and the tree own these elements
- [~] `gmOverviewMapUI::Show` · 0043e230 · legions 0043ea80 — ac2-ui overview_map.rs: label is Panel_Label_OverviewMap, client area OverviewMapClientArea, close OverviewMapCloseButton (admin Tokenize strings); the help button is a generic help button (hud_help.rs ID_Help_OverviewMap). The field is hidden with the HUD rather than made on show and destroyed on hide
- [x] `gmOverviewMapUI::ShowMap` · 0043e290 · legions 0043eae0 — ac2-ui overview_map.rs: shows the overview and sets the label and client area to the map's state (the map buttons that call it are not identified in the layout)
- [~] `gmOverviewMapUI::ListenToMessage` · 0043e2c0 · legions 0043eb10 — ac2-ui overview_map.rs map_for_button/message: the four zoom buttons (Osteth, Omishan, Arramora, LinvakMassif) call show_map with their map state; the close button hides. Client routes UiMessage to it (ac2-client hud_player_message.rs). Not: KnorrMap (its button id is not bound: Legions binds DAT_00a4b640 as both the coordinates text and the Knorr-style message), the help page (KeyStone::OpenHelp)
- [~] `gmOverviewMapUI::gmOverviewMapUI` · 0043e300 · legions 0043eb70 — ac2-ui overview_map.rs: label is Panel_Label_OverviewMap, client area OverviewMapClientArea, close OverviewMapCloseButton (admin Tokenize strings); the help button is a generic help button (hud_help.rs ID_Help_OverviewMap). The field is hidden with the HUD rather than made on show and destroyed on hide

## `ac2/client/src/ui/gameplay/gmpetui.cpp.c` (11/18)

### gmPetUI
- [~] `gmPetUI::Show` · 004188d0 · legions 00418690 — ac2-ui pets.rs show: the framework's shown state; the option-driven show (GetShowPetUI) is now set from the options reload (frame_passes.rs reload_pass)
- [x] `gmPetUI::GetNextMode` · 00418920 · legions 004186e0 — ac2-ui pets.rs next_mode / previous_mode: the ModeTable of admin 7148's static initializer (_E554); the table is in the admin tree only, not the Legions decompile
- [x] `gmPetUI::GetPreviousMode` · 00418950 · legions 00418710 — ac2-ui pets.rs next_mode / previous_mode: the ModeTable of admin 7148's static initializer (_E554); the table is in the admin tree only, not the Legions decompile

### _STL::_Node_Alloc_Lock<1,0>
- [-] `_STL::_Node_Alloc_Lock<1,0>::~_Node_Alloc_Lock<1,0>` · 004189c0 · legions 00418780 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### gmPetUI
- [x] `gmPetUI::UpdateState` · 00418cb0 · legions 00418a70 — ac2-ui pets.rs update_state: the PetPanel state (pets / nopets), the four buttons ghosted, the root drawn only with a pet listed and the UI on; tests in pets_tests.rs
- [x] `gmPetUI::RecvNotice_ReloadGameplayOptions` · 00418e20 — Legions has it (undecompiled vtable slot 0x418c00: UpdateState). ac2-ui frame_passes.rs reload_pass calls PetUi::set_show_pet_ui from the options profile (PROFILE_HIDE_PET_UI) on every options reload
- [x] `gmPetUI::RecvNotice_RemovePet` · 00419130 — Legions 0x418F70 (undecompiled slot). ac2-ui pets.rs PetUi::remove_pet: the pet leaves the table and its row the list; UpdateState runs either way
- [x] `gmPetUI::RecvNotice_SelectionChanged` · 004191d0 — Legions 0x419010 (undecompiled slot, matches admin). ac2-ui pets.rs PetUi::selection_changed: old row unselected (and its Name stops being editable when the selection moves), new listed pet's row selected; via pet_notice on GameplayNotice::SelectionChanged
- [-] `gmPetUI::~gmPetUI` · 00419430 · legions 00419270 — C++ destructor: the Rust drop and the HUD's tree own the panel's elements
- [x] `gmPetUI::Refresh` · 00419580 · legions 004193c0 — ac2-ui pets.rs tick: the five-second pass, each row's wanted mode sent (PetAction::Mode -> Command::PetMode), the unsafe pet back to safe
- [x] `gmPetUI::RecvNotice_PetIsUnderAttack` · 00419730 — ac2-ui pets.rs pet_under_attack: the Name state unsafe and the attack time; HUD routes PetIsUnderAttack (ac2-client hud_pets.rs)
- [x] `gmPetUI::ListenToMessage` · 00419810 · legions 00419650 — ac2-ui pets.rs tick: the refresh message's five-second gate (the HUD calls it each frame, ac2-client hud_player.rs)
- [~] `gmPetUI::RequestSetPetMode` · 00419840 · legions 00419680 — ac2-ui pets.rs step_mode: the row's wanted mode and its text; the row's server mode is not changed (UpdatePetMode is a no-op in Legions)
- [~] `gmPetUI::RecvNotice_AddPet` · 004199d0 — Legions 0x419810 (undecompiled slot). ac2-ui pets.rs PetUi::add_pet: row, mode, name through UpdatePetName, Icon image (AddImageToState icon, 0), UpdateState. A pet already listed is skipped (retail replaces its table entry and leaves the old row in the list)
- [x] `gmPetUI::RecvNotice_UpdatePetMode` · 00419b70 · legions 004199b0 — Legions 0x4199B0 (named in the Legions decompile; the notice is a live slot, not a stub). ac2-ui pets.rs PetUi::update_pet_mode
- [~] `gmPetUI::ListenToMessage` · 00419c20 · legions 00419a60 — ac2-ui pets.rs message: the buttons, the arrows, a press on a row selecting its pet, and the right press (0x40000005) making the row's Name editable, selected and active; its deactivated asks RequestSetPetName when the text differs (ignoring case) and shows the stored name again (PetAction::Rename -> Command::PetRename). Not ported: the set_show_state / location_changed branches of the panel itself
- [~] `gmPetUI::gmPetUI` · 0041a040 · legions 00419e80 — ac2-ui pets.rs create: the PetPanel root, its list and UpdateState; the notice registration is the HUD's dispatch (ac2-client hud_pets.rs)
- [x] `gmPetUI::RecvNotice_UpdatePetName` · 0041a3a0 — Legions 0x41A1E0 (undecompiled slot). ac2-ui pets.rs PetUi::update_pet_name: the first (...) (open not first, a close after it) removed from the server's name (PStringBase::replace with L""), stored as PetInfo::name and set on the row's Name

## `ac2/client/src/ui/gameplay/gmradarui.cpp.c` (28/35)

### Position
- [ ] `Position::get_heading` · 00424fe0 · legions 00424dd0 — needs port: Position::get_heading behaviour

### RadarInfo
- [x] `RadarInfo::RadarInfo` · 00425000 — ac2-client client_radar.rs + ac2-script host/radar.rs (InqRadarLook), ac2-weenie notice.rs (ChangeRadarLook(obj))

### gmRadarUI
- [x] `gmRadarUI::Show` · 00425040 · legions 00424e30 — ac2-ui radar.rs show and location_applied (the radar field and heading ring); ac2-client hud_player.rs applies it after the saved locations; radar_tests the_radar_shows_once_its_location_is_applied
- [x] `gmRadarUI::LocationApplied` · 00425090 · legions 00424e80 — ac2-ui radar.rs show and location_applied (the radar field and heading ring); ac2-client hud_player.rs applies it after the saved locations; radar_tests the_radar_shows_once_its_location_is_applied
- [x] `gmRadarUI::RecvNotice_SelectionChanged` · 004250b0 — ac2-ui radar.rs
- [x] `gmRadarUI::DrawPoint` · 004250d0 · legions 00424ec0 — ac2-ui radar.rs
- [x] `gmRadarUI::DrawL` · 00425110 · legions 00424f00 — ac2-ui radar.rs
- [x] `gmRadarUI::DrawEdges` · 00425180 · legions 00424f70 — ac2-ui radar.rs
- [x] `gmRadarUI::DrawCorners` · 00425240 · legions 00425030 — ac2-ui radar.rs
- [x] `gmRadarUI::DrawT` · 00425310 · legions 00425100 — ac2-ui radar.rs
- [x] `gmRadarUI::DrawFace` · 00425380 · legions 00425170 — ac2-ui radar.rs
- [x] `gmRadarUI::DrawSelected` · 00425420 · legions 00425210 — ac2-ui radar.rs
- [x] `gmRadarUI::SetCoordButtonToolTip` · 00425500 · legions 004252f0 — ac2-ui radar.rs + ac2-client (Command::SetShowRadarCoords, IFun 0xB4007D; show_radar_coords from the options profile)

### SmartArray<RadarInfo>
- [x] `SmartArray<RadarInfo>::RemoveUnOrdered` · 00425560 · legions 00425350 — ac2-client client_radar.rs + ac2-script host/radar.rs (InqRadarLook), ac2-weenie notice.rs (ChangeRadarLook(obj))

### SmartArray<GrooveInfo>
- [x] `SmartArray<GrooveInfo>::RemoveUnOrdered` · 004255b0 · legions 004253a0 — ac2-client client_radar.rs + ac2-script host/radar.rs (InqRadarLook), ac2-weenie notice.rs (ChangeRadarLook(obj))

### gmRadarUI
- [-] `gmRadarUI::~gmRadarUI` · 004256b0 · legions 004254a0 — teardown: the HUD tree is dropped whole
- [x] `gmRadarUI::UpdateGroove` · 00425740 · legions 00425530 — ac2-audio/ac2-client: ambient environments, groove
- [x] `gmRadarUI::RemoveObject` · 00425850 · legions 00425640
- [x] `gmRadarUI::RecvNotice_BeingDeleted` · 004258d0 · legions 004256c0 — ac2-client client_radar.rs + ac2-script host/radar.rs (InqRadarLook), ac2-weenie notice.rs (ChangeRadarLook(obj))
- [x] `gmRadarUI::DrawCross` · 00425900 · legions 004256f0 — ac2-ui radar.rs
- [x] `gmRadarUI::DrawX` · 00425950 · legions 00425740 — ac2-ui radar.rs
- [x] `gmRadarUI::DrawHollow` · 004259a0 · legions 00425790 — ac2-ui radar.rs
- [~] `gmRadarUI::ListenToMessage` · 004259f0 · legions 004257e0 — ac2-client hud_view.rs: the radar field drawn alone each frame (ForceDraw); UpdateGroove in client_radar.rs
- [x] `gmRadarUI::DrawFill` · 00425a30 · legions 00425820 — ac2-ui radar.rs

### SmartArray<RadarInfo>
- [x] `SmartArray<RadarInfo>::AddToEnd` · 00425a90 · legions 00425880 — ac2-client client_radar.rs + ac2-script host/radar.rs (InqRadarLook), ac2-weenie notice.rs (ChangeRadarLook(obj))

### gmRadarUI
- [x] `gmRadarUI::DrawBlip` · 00425af0 · legions 004258e0 — ac2-ui radar.rs
- [~] `gmRadarUI::Update` · 00425c00 · legions 004259f0 — ac2-ui radar.rs + ac2-client: heading ring, blips (scale float, ftol, square clip), tooltip and last-under-mouse, CoordinatesText; SaveLocationsChanged on location_changed not yet
- [x] `gmRadarUI::ListenToMessage` · 00426060 · legions 00425e50 — ac2-ui radar.rs + ac2-client (Command::SetShowRadarCoords, IFun 0xB4007D; show_radar_coords from the options profile)
- [~] `gmRadarUI::AddObject` · 00426190 · legions 00425f80 — ac2-audio/ac2-client ambient: four-copy contin spread unused by Legions; interior visibility from the player's stab list; groove objects read every update
- [x] `gmRadarUI::RecvNotice_CreateObject` · 00426270 — ac2-client client_radar.rs + ac2-script host/radar.rs (InqRadarLook), ac2-weenie notice.rs (ChangeRadarLook(obj))
- [~] `gmRadarUI::Init` · 004262a0 · legions 00426090 — ac2-client client_radar.rs + ac2-script radar.rs: looks asked on each object's script (the templates' override; selectable objects, the player's radar mask), ranges from RadarControl
- [x] `gmRadarUI::RecvNotice_ChangeRadarLook` · 004263a0 — ac2-client client_radar.rs + ac2-script host/radar.rs (InqRadarLook), ac2-weenie notice.rs (ChangeRadarLook(obj))
- [x] `gmRadarUI::ReloadOptions` · 00426400 — ac2-ui radar.rs + ac2-client (Command::SetShowRadarCoords, IFun 0xB4007D; show_radar_coords from the options profile)
- [~] `gmRadarUI::gmRadarUI` · 00426450 · legions 00426240 — ac2-ui hud.rs + ac2-gpu ui.rs + ac2-client hud_view.rs: HUD roots made and drawn; no framework behaviour beyond hiding the toolbar's dismount/consignment buttons
- [x] `gmRadarUI::RecvNotice_ReloadOptions` · 00426710 — ac2-client client_radar.rs + ac2-script host/radar.rs (InqRadarLook), ac2-weenie notice.rs (ChangeRadarLook(obj))

## `ac2/client/src/ui/gameplay/gmsecuretradeui.cpp.c` (24/27)

### gmSecureTradeUI
- [x] `gmSecureTradeUI::Show` · 00420a90 · legions 004208c0 — ac2-ui secure_trade.rs: SecureTradeUi (Show, CloseTrade, FinishListBoxDrop, UpdateButton, HandleItemRemoval, the DDCB as the list's item check in message())
- [x] `gmSecureTradeUI::CloseTrade` · 00420b10 · legions 00420940 — ac2-ui secure_trade.rs: SecureTradeUi (Show, CloseTrade, FinishListBoxDrop, UpdateButton, HandleItemRemoval, the DDCB as the list's item check in message())
- [x] `gmSecureTradeUI::FinishListBoxDrop` · 00420b50 · legions 00420980 — ac2-ui secure_trade.rs: SecureTradeUi (Show, CloseTrade, FinishListBoxDrop, UpdateButton, HandleItemRemoval, the DDCB as the list's item check in message())
- [x] `gmSecureTradeUI::UpdateButton` · 00420ba0 · legions 004209d0 — ac2-ui secure_trade.rs: SecureTradeUi (Show, CloseTrade, FinishListBoxDrop, UpdateButton, HandleItemRemoval, the DDCB as the list's item check in message())
- [x] `gmSecureTradeUI::HandleItemRemoval` · 00420c20 · legions 00420a50 — ac2-ui secure_trade.rs: SecureTradeUi (Show, CloseTrade, FinishListBoxDrop, UpdateButton, HandleItemRemoval, the DDCB as the list's item check in message())

### (free functions)
- [x] `gmSecureTradeUI_DDCB` · 00420c70 — ac2-ui secure_trade.rs: SecureTradeUi (Show, CloseTrade, FinishListBoxDrop, UpdateButton, HandleItemRemoval, the DDCB as the list's item check in message())

### gmSecureTradeUI
- [x] `gmSecureTradeUI::ListenToMessage` · 00420ca0 · legions 00420ad0 — ac2-ui secure_trade.rs: SecureTradeUi::message (buttons, list drops, TradeItemTemplate drag out) and tick (the global loop)
- [x] `gmSecureTradeUI::RecvNotice_StackUIDone` · 00420cd0 — ac2-ui secure_trade.rs: recv_* notice methods (StackUIDone, BeAcceptanceCleared, BeAccepted, BeDeclined, BeOpened, DoAcceptTrade, HandleListBoxDrop, CloseConfirmationDialog, MakeAcceptTradeDialog, BeRegistered); client ui_inventory::secure_trade_notice
- [x] `gmSecureTradeUI::RecvNotice_Trade_BeAcceptanceCleared` · 00420d10 · legions 00420b40 — ac2-ui secure_trade.rs: recv_* notice methods (StackUIDone, BeAcceptanceCleared, BeAccepted, BeDeclined, BeOpened, DoAcceptTrade, HandleListBoxDrop, CloseConfirmationDialog, MakeAcceptTradeDialog, BeRegistered); client ui_inventory::secure_trade_notice
- [x] `gmSecureTradeUI::RecvNotice_Trade_BeAccepted` · 00420d50 · legions 00420b80 — ac2-ui secure_trade.rs: recv_* notice methods (StackUIDone, BeAcceptanceCleared, BeAccepted, BeDeclined, BeOpened, DoAcceptTrade, HandleListBoxDrop, CloseConfirmationDialog, MakeAcceptTradeDialog, BeRegistered); client ui_inventory::secure_trade_notice
- [x] `gmSecureTradeUI::RecvNotice_Trade_BeDeclined` · 00420dd0 · legions 00420c00 — ac2-ui secure_trade.rs: recv_* notice methods (StackUIDone, BeAcceptanceCleared, BeAccepted, BeDeclined, BeOpened, DoAcceptTrade, HandleListBoxDrop, CloseConfirmationDialog, MakeAcceptTradeDialog, BeRegistered); client ui_inventory::secure_trade_notice
- [x] `gmSecureTradeUI::RecvNotice_Trade_BeOpened` · 00420e50 · legions 00420c80 — ac2-ui secure_trade.rs: recv_* notice methods (StackUIDone, BeAcceptanceCleared, BeAccepted, BeDeclined, BeOpened, DoAcceptTrade, HandleListBoxDrop, CloseConfirmationDialog, MakeAcceptTradeDialog, BeRegistered); client ui_inventory::secure_trade_notice
- [x] `gmSecureTradeUI::DoAcceptTrade` · 00420f00 · legions 00420d30 — ac2-ui secure_trade.rs: recv_* notice methods (StackUIDone, BeAcceptanceCleared, BeAccepted, BeDeclined, BeOpened, DoAcceptTrade, HandleListBoxDrop, CloseConfirmationDialog, MakeAcceptTradeDialog, BeRegistered); client ui_inventory::secure_trade_notice
- [x] `gmSecureTradeUI::HandleListBoxDrop` · 00420f60 · legions 00420d90 — ac2-ui secure_trade.rs: recv_* notice methods (StackUIDone, BeAcceptanceCleared, BeAccepted, BeDeclined, BeOpened, DoAcceptTrade, HandleListBoxDrop, CloseConfirmationDialog, MakeAcceptTradeDialog, BeRegistered); client ui_inventory::secure_trade_notice
- [x] `gmSecureTradeUI::ListenToMessage` · 00421100 · legions 00420f30 — ac2-ui secure_trade.rs: SecureTradeUi::message (buttons, list drops, TradeItemTemplate drag out) and tick (the global loop)
- [x] `gmSecureTradeUI::RecvNotice_CloseConfirmationDialog` · 00421230 — ac2-ui secure_trade.rs: recv_* notice methods (StackUIDone, BeAcceptanceCleared, BeAccepted, BeDeclined, BeOpened, DoAcceptTrade, HandleListBoxDrop, CloseConfirmationDialog, MakeAcceptTradeDialog, BeRegistered); client ui_inventory::secure_trade_notice
- [x] `gmSecureTradeUI::MakeAcceptTradeDialog` · 00421250 · legions 00421080 — ac2-ui secure_trade.rs: recv_* notice methods (StackUIDone, BeAcceptanceCleared, BeAccepted, BeDeclined, BeOpened, DoAcceptTrade, HandleListBoxDrop, CloseConfirmationDialog, MakeAcceptTradeDialog, BeRegistered); client ui_inventory::secure_trade_notice
- [x] `gmSecureTradeUI::RecvNotice_Trade_BeRegistered` · 004212d0 — ac2-ui secure_trade.rs: recv_* notice methods (StackUIDone, BeAcceptanceCleared, BeAccepted, BeDeclined, BeOpened, DoAcceptTrade, HandleListBoxDrop, CloseConfirmationDialog, MakeAcceptTradeDialog, BeRegistered); client ui_inventory::secure_trade_notice
- [~] `gmSecureTradeUI::AddItem` · 00421380 · legions 004211b0 — ac2-ui secure_trade.rs add_item: the list, the draggable test (src == player) and the no-shortcut icon are as retail; the SetQuantity count (retail passes the item's high dword) is not modelled
- [x] `gmSecureTradeUI::RecvNotice_Trade_BeOffered` · 00421550 — ac2-ui secure_trade.rs: recv_trade_be_offered / remove_item / recv_trade_be_revoked / reset_side / List::remove (Vec position) / recv_trade_be_closed (delay from the field) / recv_trade_be_reset
- [x] `gmSecureTradeUI::RemoveItem` · 00421590 · legions 004213c0 — ac2-ui secure_trade.rs: recv_trade_be_offered / remove_item / recv_trade_be_revoked / reset_side / List::remove (Vec position) / recv_trade_be_closed (delay from the field) / recv_trade_be_reset
- [x] `gmSecureTradeUI::RecvNotice_Trade_BeRevoked` · 00421690 — ac2-ui secure_trade.rs: recv_trade_be_offered / remove_item / recv_trade_be_revoked / reset_side / List::remove (Vec position) / recv_trade_be_closed (delay from the field) / recv_trade_be_reset
- [x] `gmSecureTradeUI::ResetInternal` · 004216b0 · legions 00421520 — ac2-ui secure_trade.rs: recv_trade_be_offered / remove_item / recv_trade_be_revoked / reset_side / List::remove (Vec position) / recv_trade_be_closed (delay from the field) / recv_trade_be_reset
- [~] `gmSecureTradeUI::gmSecureTradeUI` · 004217a0 · legions 00421610 — ac2-ui SecureTradeUi::create (hidden, no field); the notice registrations are the client's (hud_player), the player is set_player
- [-] `gmSecureTradeUI::~gmSecureTradeUI` · 004219d0 · legions 00421840 — C++ destructors: the Rust state drops
- [x] `gmSecureTradeUI::RecvNotice_Trade_BeClosed` · 00421a90 — ac2-ui secure_trade.rs: recv_trade_be_offered / remove_item / recv_trade_be_revoked / reset_side / List::remove (Vec position) / recv_trade_be_closed (delay from the field) / recv_trade_be_reset
- [x] `gmSecureTradeUI::RecvNotice_Trade_BeReset` · 00421bf0 — ac2-ui secure_trade.rs: recv_trade_be_offered / remove_item / recv_trade_be_revoked / reset_side / List::remove (Vec position) / recv_trade_be_closed (delay from the field) / recv_trade_be_reset

## `ac2/client/src/ui/gameplay/gmselectionui.cpp.c` (12/24)

### LandDefs
- [ ] `LandDefs::in_bounds` · 004233a0 · legions 00423170 — needs port: LandDefs::in_bounds behaviour
- [x] `LandDefs::blockid_to_lcoord` · 004233d0 · legions 004231a0
- [ ] `LandDefs::in_same_block` · 00423450 · legions 00423220 — needs port: LandDefs::in_same_block behaviour
- [x] `LandDefs::get_block_offset` · 00423490 · legions 00423260

### Frame
- [x] `Frame::globaltolocalvec` · 00423550 · legions 00423320
- [ ] `Frame::localtoglobal` · 004235b0 · legions 00423380 — needs port: Frame::localtoglobal behaviour
- [x] `Frame::globaltolocal` · 00423610 · legions 004233e0 — ac2-physics bsp/tree.rs, position.rs: physics-BSP collision in mesh space (asm float order; Legions identical)

### Position
- [x] `Position::localtolocal` · 00423690 · legions 00423460 — ac2-physics bsp/tree.rs, position.rs: physics-BSP collision in mesh space (asm float order; Legions identical)

### gmSelectionUI
- [-] `gmSelectionUI::RegisterObjectMessages` · 00423750 · legions 00423520 — notice handlers are registered per object by the session in Rust (no per-object handler table)
- [-] `gmSelectionUI::RecvNotice_CreateObject` · 004237a0 — notice handlers are registered per object by the session in Rust (no per-object handler table)
- [~] `gmSelectionUI::RecvNotice_E_LastAttacker` · 004237d0 · legions 004235a0 — ac2-ui selection_notices.rs last_attacker_notice; ac2-client target_select.rs: the auto-target select with nothing selected. Producer ECM_Physics::SendNotice_E_LastAttacker (from StoryManager::DoStoryHook) not ported, so no notice arrives yet; LookAtSelection not modelled. Legions identical
- [x] `gmSelectionUI::RecvNotice_SetTargetingPointer` · 00423820 — ac2-ui selection_notices.rs set_targeting_pointer: usage cursor swapped in, saved cursor restored (real-data test); ac2-client target_select.rs routes the notice
- [x] `gmSelectionUI::Farther` · 004238f0 · legions 004236c0 — ac2-ui target_cycle.rs farther(): the nearer-by-distance-then-id order
- [x] `gmSelectionUI::TargetTypeVsKey` · 00423970 · legions 00423740 — ac2-ui target_cycle.rs type_vs_key(): the type bits against the key ranges
- [x] `gmSelectionUI::Start3DSelection` · 00423a50 · legions 00423820 — ac2-render selection.rs + ac2-ui selection_ui.rs: clicking the world selects
- [x] `gmSelectionUI::ListenToMessage` · 00423b20 · legions 004238f0 — ac2-render selection.rs + ac2-ui selection_ui.rs: clicking the world selects
- [~] `gmSelectionUI::Finish3DSelection` · 00423bb0 · legions 00423980 — ac2-render selection.rs pick_list (drawn-mesh triangles, the potential list nearest first); ac2-ui selection_ui.rs finish: the first entry's 0x270 selectability refused unless the player's 0x14 has the 0x3f bits, the clicks wait; ac2-client target_select.rs pick_targets. Not: CWPhysicsObject::LookAtSelection (physics look-at and ECM_Physics::SendEvt_CLookAt), the double click's own-item check (0x105)
- [-] `gmSelectionUI::Init` · 00423d20 · legions 00423af0 — notice handlers are registered per object by the session in Rust (no per-object handler table)
- [~] `gmSelectionUI::Select` · 00423da0 — ac2-ui target_cycle.rs select_target (the cycling keys) and selection_notices.rs direct_key: reselect 0x41000018, last selection 0x41000028, last attacker 0x4100002A, clear 0x4000001E with the ICIDM text-mode check. Not: CanAutoAttackTarget approximated as creature and not a fellow; candidates are the radar blips; LookAtSelection not modelled. Legions FUN_00423b70 has the same key branches
- [x] `gmSelectionUI::RecvNotice_Select` · 00424500 — ac2-client target_select.rs + hud_player: the notice's key picks through select_target
- [~] `gmSelectionUI::gmSelectionUI` · 00424510 · legions 004242e0 — ac2-ui hud.rs + ac2-gpu ui.rs + ac2-client hud_view.rs: HUD roots made and drawn; no framework behaviour beyond hiding the toolbar's dismount/consignment buttons
- [x] `gmSelectionUI::ListenToMessage` · 00424610 · legions 004243e0 — ac2-ui target_cycle.rs select_key_of/is_select_key; ac2-client hud_player routes the global key press
- [~] `gmSelectionUI::RecvNotice_UpdateSelectability` · 00424670 — ac2-ui selection_notices.rs (selectability, being_deleted); ac2-client target_select.rs: clear the selection, auto-target next foe, and StopAttacking now runs gmCWInterface::StopAttacking (IFun 0xB40104 on the VM; combat_7160 stop_attacking_runs_on_the_vm_for_the_target). Not: CWPhysicsObject::LookAtSelection; UnRegisterAllNoticeHandlers is per-object in Rust
- [~] `gmSelectionUI::RecvNotice_BeingDeleted` · 00424720 — ac2-ui selection_notices.rs (selectability, being_deleted); ac2-client target_select.rs: clear the selection, auto-target next foe, and StopAttacking now runs gmCWInterface::StopAttacking (IFun 0xB40104 on the VM; combat_7160 stop_attacking_runs_on_the_vm_for_the_target). Not: CWPhysicsObject::LookAtSelection; UnRegisterAllNoticeHandlers is per-object in Rust

## `ac2/client/src/ui/gameplay/gmshopcatalogui.cpp.c` (9/11)

### gmShopCatalogUI
- [x] `gmShopCatalogUI::DestroyManagedElements` · 0041cc80 · legions 0041caa0 — ac2-ui catalog.rs (CatalogUi: the Catalog root, the StoreElement rows, select and enter); ac2-client hud_shop.rs catalog_notice; tests ac2-ui catalog_tests.rs and ac2-client tests/catalog_panel_7160.rs
- [x] `gmShopCatalogUI::Show` · 0041ccb0 · legions 0041cad0 — ac2-ui catalog.rs (CatalogUi: the Catalog root, the StoreElement rows, select and enter); ac2-client hud_shop.rs catalog_notice; tests ac2-ui catalog_tests.rs and ac2-client tests/catalog_panel_7160.rs
- [x] `gmShopCatalogUI::Select` · 0041cd30 · legions 0041cb50 — ac2-ui catalog.rs (CatalogUi: the Catalog root, the StoreElement rows, select and enter); ac2-client hud_shop.rs catalog_notice; tests ac2-ui catalog_tests.rs and ac2-client tests/catalog_panel_7160.rs
- [x] `gmShopCatalogUI::ListenToMessage` · 0041cda0 · legions 0041cbc0 — ac2-ui catalog.rs (CatalogUi: the Catalog root, the StoreElement rows, select and enter); ac2-client hud_shop.rs catalog_notice; tests ac2-ui catalog_tests.rs and ac2-client tests/catalog_panel_7160.rs
- [x] `gmShopCatalogUI::EnterStore` · 0041cde0 · legions 0041cc00 — ac2-ui catalog.rs (CatalogUi: the Catalog root, the StoreElement rows, select and enter); ac2-client hud_shop.rs catalog_notice; tests ac2-ui catalog_tests.rs and ac2-client tests/catalog_panel_7160.rs
- [x] `gmShopCatalogUI::gmShopCatalogUI` · 0041ce30 · legions 0041cc50 — ac2-ui catalog.rs (CatalogUi: the Catalog root, the StoreElement rows, select and enter); ac2-client hud_shop.rs catalog_notice; tests ac2-ui catalog_tests.rs and ac2-client tests/catalog_panel_7160.rs
- [~] `gmShopCatalogUI::~gmShopCatalogUI` · 0041cf10 · legions 0041cd30 — catalog.rs message: close, enter, consignment and row presses; CatalogHelpButton opens ID_Help_ShopCatalog (hud_help.rs; help_buttons_tests)
- [x] `gmShopCatalogUI::RecvNotice_LeaveCatalog` · 0041cf80 — ac2-ui catalog.rs (CatalogUi: the Catalog root, the StoreElement rows, select and enter); ac2-client hud_shop.rs catalog_notice; tests ac2-ui catalog_tests.rs and ac2-client tests/catalog_panel_7160.rs
- [~] `gmShopCatalogUI::ListenToMessage` · 0041cff0 · legions 0041ce10 — catalog.rs message: close, enter, consignment and row presses; CatalogHelpButton opens ID_Help_ShopCatalog (hud_help.rs; help_buttons_tests)
- [x] `gmShopCatalogUI::PostShop` · 0041d0f0 · legions 0041cf10 — ac2-ui catalog.rs (CatalogUi: the Catalog root, the StoreElement rows, select and enter); ac2-client hud_shop.rs catalog_notice; tests ac2-ui catalog_tests.rs and ac2-client tests/catalog_panel_7160.rs
- [x] `gmShopCatalogUI::RecvNotice_ViewCatalog` · 0041d230 — ac2-ui catalog.rs (CatalogUi: the Catalog root, the StoreElement rows, select and enter); ac2-client hud_shop.rs catalog_notice; tests ac2-ui catalog_tests.rs and ac2-client tests/catalog_panel_7160.rs

## `ac2/client/src/ui/gameplay/gmshopconsignmentui.cpp.c` (15/28)

### UIShopSale
- [-] `UIShopSale::~UIShopSale` · 0041b090 · legions 0041aef0 — C++ destructor of the sale record: ConsignmentUi's rows own their data in Rust

### gmShopConsignmentUI
- [~] `gmShopConsignmentUI::UpdatePortraitPosition` · 0041b0b0 · legions 004424c0 — 3D portrait of the player (UIElement_Viewport, UIScenePhysObjEntity, camera): not modelled in ac2-ui
- [x] `gmShopConsignmentUI::RecvNotice_UpdateMountState` · 0041b240 — ac2-ui consignment.rs ConsignmentUi::recv_update_mount_state keeps the flag (retail reads it only to redraw the player portrait in RecvNotice_LocalVisualDescUpdate, which is not modelled); routed by shop_notices.rs consignment_notice

### gmShopUI
- [~] `gmShopUI::RecvNotice_LeaveStore` · 0041b250 — ac2-ui shop.rs recv_leave_store: closes via show(false); the vtable call in the decompile is untraced

### gmShopConsignmentUI
- [~] `gmShopConsignmentUI::SyncPortrait` · 0041b270 · legions 0041b0d0 — 3D portrait of the player (UIElement_Viewport, UIScenePhysObjEntity, camera): not modelled in ac2-ui

### (free functions)
- [-] `gmShopConsignmentUI_DDCB` · 0041b320 — scalar deleting destructor and MSVC unwind cleanup: Rust owns the value

### gmShopConsignmentUI
- [x] `gmShopConsignmentUI::RecvNotice_StackUIDone` · 0041b390 — ac2-ui consignment.rs (ConsignmentUi); tests consignment_tests.rs
- [x] `gmShopConsignmentUI::RecvNotice_CloseConfirmationDialog` · 0041b550 · legions 0041b3b0 — ac2-ui consignment.rs (ConsignmentUi); tests consignment_tests.rs
- [~] `gmShopConsignmentUI::CanStackWith` · 0041b5d0 · legions 0041b430 — ac2-ui consignment.rs can_stack_with: the item definition id comes from ConsignmentSource::item_did, which the client does not yet supply (no stacking restock in the game)
- [~] `gmShopConsignmentUI::InitPortrait` · 0041b6a0 · legions 0041b500 — 3D portrait of the player (UIElement_Viewport, UIScenePhysObjEntity, camera): not modelled in ac2-ui
- [~] `gmShopConsignmentUI::UpdatePortrait` · 0041b760 · legions 0041b5c0 — 3D portrait of the player (UIElement_Viewport, UIScenePhysObjEntity, camera): not modelled in ac2-ui
- [~] `gmShopConsignmentUI::RecvNotice_LocalVisualDescUpdate` · 0041b7c0 — the 3D portrait of the player (UIElement_Viewport, UIScenePhysObjEntity, camera) is not modelled in ac2-ui; the mount flag it reads is kept (recv_update_mount_state)
- [x] `gmShopConsignmentUI::UpdatePosition` · 0041b7e0 · legions 0041b640 — ac2-ui consignment.rs (ConsignmentUi); tests consignment_tests.rs
- [x] `gmShopConsignmentUI::CreateManagedElements` · 0041b8b0 · legions 0041b710 — ac2-ui consignment.rs (ConsignmentUi); tests consignment_tests.rs
- [x] `gmShopConsignmentUI::Show` · 0041b950 · legions 0041b7b0 — ac2-ui consignment.rs (ConsignmentUi); tests consignment_tests.rs
- [x] `gmShopConsignmentUI::Select` · 0041ba10 · legions 0041b870 — ac2-ui consignment.rs (ConsignmentUi); tests consignment_tests.rs
- [x] `gmShopConsignmentUI::DestroyManagedElements` · 0041bc00 · legions 0041ba20 — ac2-ui consignment.rs (ConsignmentUi); tests consignment_tests.rs
- [~] `gmShopConsignmentUI::RecvNotice_HandleDragFrom` · 0041bc90 — not modelled: the mount state, the drag-from-list notice and the server selection notice
- [x] `gmShopConsignmentUI::RecvNotice_SelectionChanged` · 0041bd20 — ac2-ui consignment.rs ConsignmentUi::recv_selection_changed: while shown, a selection other than the selected sale's item clears the sale's selection (Select(null)); routed by shop_notices.rs consignment_notice
- [x] `gmShopConsignmentUI::ListenToMessage` · 0041bd90 · legions 0041bbb0 — ac2-ui consignment.rs (ConsignmentUi); tests consignment_tests.rs
- [x] `gmShopConsignmentUI::HandleListBoxDrop` · 0041bde0 · legions 0041bc00 — ac2-ui consignment.rs (ConsignmentUi); tests consignment_tests.rs
- [~] `gmShopConsignmentUI::~gmShopConsignmentUI` · 0041bff0 · legions 0041be10 — no Drop and no notice registration in ac2-ui; the HUD routes the notices and the panel is created hidden
- [x] `gmShopConsignmentUI::PostSale` · 0041c0f0 · legions 0041bf10 — ac2-ui consignment.rs (ConsignmentUi); tests consignment_tests.rs
- [~] `gmShopConsignmentUI::ListenToMessage` · 0041c480 · legions 0041c2a0 — consignment.rs message: close (ConsignmentCloseButton), new (NewConsignmentButton), collect (CollectConsignmentButton), paging (ConsignmentNext/PrevButton), selection, examination and the drop are modelled, under the layout's retail token names; the help page (ConsignmentHelpButton, KeyStone::OpenHelp) opens ID_Help_ShopConsignment (hud_help.rs)
- [x] `gmShopConsignmentUI::UpdateSales` · 0041c820 · legions 0041c640 — ac2-ui consignment.rs (ConsignmentUi); tests consignment_tests.rs
- [~] `gmShopConsignmentUI::gmShopConsignmentUI` · 0041c980 · legions 0041c7a0 — no Drop and no notice registration in ac2-ui; the HUD routes the notices and the panel is created hidden
- [x] `gmShopConsignmentUI::RecvNotice_UpdateConsignment` · 0041cb60 — ac2-ui consignment.rs (ConsignmentUi); tests consignment_tests.rs
- [x] `gmShopConsignmentUI::RecvNotice_ViewConsignment` · 0041cbd0 · legions 0041c9f0 — ac2-ui consignment.rs (ConsignmentUi); tests consignment_tests.rs

## `ac2/client/src/ui/gameplay/gmshopnewsaleui.cpp.c` (10/14)

### gmShopNewSaleUI
- [x] `gmShopNewSaleUI::DestroyManagedElements` · 0041a600 · legions 0041a440 — ac2-ui newsale.rs (NewSaleUi); tests newsale_tests.rs
- [x] `gmShopNewSaleUI::Update` · 0041a640 · legions 0041a4a0 — ac2-ui newsale.rs (NewSaleUi); tests newsale_tests.rs
- [x] `gmShopNewSaleUI::ListenToMessage` · 0041a760 · legions 0041a5c0 — ac2-ui newsale.rs (NewSaleUi); tests newsale_tests.rs
- [x] `gmShopNewSaleUI::RecvNotice_CloseConfirmationDialog` · 0041a7b0 · legions 0041a610 — ac2-ui newsale.rs (NewSaleUi); tests newsale_tests.rs
- [x] `gmShopNewSaleUI::HandleDragDrop` · 0041a830 · legions 0041a690 — ac2-ui newsale.rs (NewSaleUi); tests newsale_tests.rs

### (free functions)
- [-] `gmShopNewSaleUI_DDCB` · 0041a950 — scalar deleting destructor and MSVC unwind cleanup: Rust owns the value

### gmShopNewSaleUI
- [~] `gmShopNewSaleUI::~gmShopNewSaleUI` · 0041a9c0 · legions 0041a820 — ac2-ui newsale.rs: no Drop; the destructor's stack and confirmation closes are the panel's Show(false), its notice unregistration is the HUD's routing
- [x] `gmShopNewSaleUI::CreateManagedElements` · 0041aa70 · legions 0041a8d0 — ac2-ui newsale.rs (NewSaleUi); tests newsale_tests.rs
- [x] `gmShopNewSaleUI::RecvNotice_StackUIDone` · 0041ab00 · legions 0041a960 — ac2-ui newsale.rs (NewSaleUi); tests newsale_tests.rs
- [x] `gmShopNewSaleUI::RecvNotice_InitNewSale` · 0041ab40 — ac2-ui newsale.rs (NewSaleUi); tests newsale_tests.rs
- [x] `gmShopNewSaleUI::Show` · 0041ab60 · legions 0041a9c0 — ac2-ui newsale.rs (NewSaleUi); tests newsale_tests.rs
- [~] `gmShopNewSaleUI::gmShopNewSaleUI` · 0041ac00 · legions 0041aa60 — ac2-ui newsale.rs NewSaleUi::create: hidden with no managed elements; the notice registrations are the HUD's routing
- [x] `gmShopNewSaleUI::AddSale` · 0041ad40 · legions 0041aba0 — ac2-ui newsale.rs (NewSaleUi); tests newsale_tests.rs
- [~] `gmShopNewSaleUI::ListenToMessage` · 0041af80 · legions 0041ade0 — newsale.rs message: close is NewSaleCloseButton (retail token, was ShopCloseButton); NewSaleHelpButton opens ID_Help_ShopConsignment (hud_help.rs); the sale slot's mouse-over tooltip is not modelled

## `ac2/client/src/ui/gameplay/gmshopui.cpp.c` (17/22)

### gmShopUI
- [x] `gmShopUI::CreateManagedElements` · 0041d3d0 · legions 0041d1f0 — ac2-ui shop.rs create_managed: the Shop root and its page buttons (Legions 7160)
- [~] `gmShopUI::UpdatePosition` · 0041d4a0 · legions 0041d2c0 — ac2-ui shop_rows.rs update_position: page ghosting; the untraced one-argument branch is read as enabled without a pending page
- [x] `gmShopUI::Show` · 0041d590 · legions 0041d3b0 — ac2-ui shop.rs show: stack and confirmation closed, root hidden for the dormant wait
- [x] `gmShopUI::RecvNotice_ConfirmPurchase` · 0041d630 — ac2-ui shop.rs recv_confirm_purchase: singular or plural question on DIALOG_CONFIRMATION
- [x] `gmShopUI::RecvNotice_CloseConfirmationDialog` · 0041d700 · legions 0041d520 — ac2-ui shop.rs recv_close_confirmation_dialog: PurchaseConfirmed(answer) for the open context
- [x] `gmShopUI::Select` · 0041d740 · legions 0041d560 — ac2-ui shop.rs select: selected state, purchase ghost, Select(item)
- [x] `gmShopUI::FinishDragFromShop` · 0041d800 · legions 0041d620 — ac2-ui shop.rs finish_drag_from_shop: Purchase(item, quantity, slot)
- [x] `gmShopUI::RecvNotice_StackUIDone` · 0041d850 · legions 0041d670 — ac2-ui shop.rs recv_stack_ui_done: the stack's amount buys
- [x] `gmShopUI::RecvNotice_UpdatePosition` · 0041d880 — ac2-ui shop.rs recv_update_position: the page's position and size while shown
- [x] `gmShopUI::RecvNotice_SelectionChanged` · 0041d8b0 — ac2-ui shop.rs recv_selection_changed: clears a selection elsewhere
- [x] `gmShopUI::DestroyManagedElements` · 0041d960 · legions 0041d780 — ac2-ui shop.rs destroy_managed: root and sales removed
- [x] `gmShopUI::RecvNotice_UpdateSale` · 0041d9b0 — ac2-ui shop_rows.rs recv_update_sale: stock text, sold out is unavailable
- [x] `gmShopUI::ListenToMessage` · 0041dac0 · legions 0041d8e0 — ac2-ui shop.rs tick: dormant destroy after 120 s, sort text page request, page buttons released
- [x] `gmShopUI::PurchaseItem` · 0041dc40 · legions 0041da60 — ac2-ui shop.rs purchase: refusals to chat, stack for more than one, else one purchase
- [~] `gmShopUI::~gmShopUI` · 0041de50 · legions 0041dc70 — no Drop: RequestLeaveStore is not sent when the panel is destroyed
- [~] `gmShopUI::RecvNotice_HandleDragFrom` · 0041df30 — ac2-ui shop.rs recv_handle_drag_from: purchase into the slot; the drag notice has no GameplayNotice yet
- [x] `gmShopUI::PostSale` · 0041df90 · legions 0041ddb0 — ac2-ui shop_rows.rs post_sale: icon, descriptions, cost, stock; drag context on the icon
- [~] `gmShopUI::ListenToMessage` · 0041e2c0 · legions 0041e0e0 — ac2-ui shop.rs message: close, sort, pages, purchase, select and examine; ShopHelpButton opens ID_Help_Shop (hud_help.rs)
- [x] `gmShopUI::UpdateSales` · 0041e540 · legions 0041e360 — ac2-ui shop_rows.rs update_sales: page position, list rebuilt in sale order, selection kept
- [x] `gmShopUI::gmShopUI` · 0041e6a0 · legions 0041e4c0 — ac2-ui shop.rs create: hidden, drag context; notices are routed by the frames
- [x] `gmShopUI::RecvNotice_UpdateStore` · 0041e8d0 — ac2-ui shop.rs recv_update_store: rebuilds the list while shown
- [~] `gmShopUI::RecvNotice_ViewStore` · 0041e940 — ac2-ui shop.rs recv_view_store: name, description, portrait, sort text, sales; the first vtable call is untraced

## `ac2/client/src/ui/gameplay/gmstackui.cpp.c` (12/13)

### gmStackUI
- [x] `gmStackUI::Show` · 004247f0 · legions 004245e0 — ac2-ui stack.rs
- [x] `gmStackUI::PositionWindow` · 00424810 · legions 00424600 — ac2-ui stack.rs
- [x] `gmStackUI::Close` · 004248f0 · legions 004246e0 — ac2-ui stack.rs
- [x] `gmStackUI::gmStackUI` · 004249b0 · legions 004247a0 — ac2-ui stack.rs
- [-] `gmStackUI::~gmStackUI` · 00424b00 · legions 004248f0 — teardown: the HUD tree is dropped whole
- [x] `gmStackUI::RecvNotice_CloseStackUI` · 00424b50 — ac2-ui stack.rs
- [x] `gmStackUI::RecvNotice_ContainerStateUpdate` · 00424b70 — ac2-ui stack.rs
- [x] `gmStackUI::RecvNotice_BeingDeleted` · 00424ba0 — ac2-ui stack.rs

### PSUtils
- [x] `PSUtils::is_uint32` · 00424be0 · legions 004249d0 — ac2-ui stack.rs

### gmStackUI
- [x] `gmStackUI::HandleOkButton` · 00424c70 · legions 00424a60 — ac2-ui stack.rs
- [x] `gmStackUI::SetAmount` · 00424da0 · legions 00424b90 — ac2-ui stack.rs
- [x] `gmStackUI::ListenToMessage` · 00424e20 · legions 00424c10 — ac2-ui stack.rs
- [x] `gmStackUI::RecvNotice_InitStackUI` · 00424ed0 · legions 00424cc0 — ac2-ui stack.rs

## `ac2/client/src/ui/gameplay/gmstoryui.cpp.c` (2/11)

### gmKeyframe
- [x] `gmKeyframe::GetSubtitle` · 0042bf80 · legions 0042bd40 — ac2-ui story.rs carries the keyframe's StringInfo; ac2-client hud_story.rs renders it on the string tables (table subtitles too)

### gmStoryUI
- [x] `gmStoryUI::Finish` · 0042c030 · legions 0042bdf0 — ac2-client client_audio.rs stop_story_sounds: the music and voice fade out over 2 s (PlayingSoundContext::Stop); the ~gmStoryUI teardown reaches it
- [~] `gmStoryUI::~gmStoryUI` · 0042c1e0 · legions 0042bfa0 — ac2-ui story_view.rs draws the BackgroundField and StoryField1/2 (image cross-fade, subtitle, Frame states; story_view_tests.rs on the real layout); ac2-client hud_story.rs drives it and the skip key (0xffff0003, 0x4000001E). Not made: the story music and voice (SoundManager), the background fade's token_play (the prefetch and fade are immediate), the teardown's StopAudio
- [~] `gmStoryUI::Play` · 0042c2d0 · legions 0042c090 — story music and voice play (ac2-client client_audio.rs play_story_sounds, Type_ForceAllExclusive, Channel_Main, Looping_ForceOn for music); subtitles are the keyframe's StringInfo rendered on the HUD (hud_story.rs); the background's token_play fade is still immediate
- [~] `gmStoryUI::CheckStart` · 0042c5a0 · legions 0042c360 — CheckStart waits for the fade (0042c6b0); background not sized to the viewport yet (rect 0 in the layout)

### QualifiedDataIDArray
- [-] `QualifiedDataIDArray::~QualifiedDataIDArray` · 0042c660 · legions 0042c420 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs

### gmStoryUI
- [~] `gmStoryUI::ListenToMessage` · 0042c6b0 · legions 0042c470 — fade done = the background's media message (layout MD_Data_Message, 3.5 s); the HUD waits for it (ac2-ui story_view.rs is_fade_done, ac2-client hud_story.rs). Test: story_view_tests.rs
- [~] `gmStoryUI::ListenToMessage` · 0042c6f0 · legions 0042c4b0 — story music and voice play (ac2-client client_audio.rs play_story_sounds, Type_ForceAllExclusive, Channel_Main, Looping_ForceOn for music); the background's token_play fade is still immediate
- [~] `gmStoryUI::OnContextFinished` · 0042c780 — ac2-ui story_view.rs draws the BackgroundField and StoryField1/2 (image cross-fade, subtitle, Frame states; story_view_tests.rs on the real layout); ac2-client hud_story.rs drives it and the skip key (0xffff0003, 0x4000001E). Not made: the story music and voice (SoundManager), the background fade's token_play (the prefetch and fade are immediate), the teardown's StopAudio

### QualifiedDataIDArray
- [-] `QualifiedDataIDArray::QualifiedDataIDArray` · 0042c7a0 · legions 0042c560 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs

### gmStoryUI
- [~] `gmStoryUI::gmStoryUI` · 0042c7f0 · legions 0042c5b0 — begin event (StoryEvent::Begin) starts the background's fade; HUD panels hidden while a story plays and restored after (hud_story.rs), a modern addition: the decompile shows no Show(0) at story start

## `ac2/client/src/ui/gameplay/helper/chatwindow.cpp.c` (16/32)

### ChatWindow
- [x] `ChatWindow::IsActiveRoot` · 0044afd0 · legions 0044bc30 — ac2-ui chat.rs / chat_window.rs
- [x] `ChatWindow::IsActive` · 0044b000 · legions 0044bc60 — ac2-ui chat.rs / chat_window.rs
- [x] `ChatWindow::HandleActivation` · 0044b030 · legions 0044bcd0 — ac2-ui chat.rs / chat_window.rs
- [~] `ChatWindow::Show` · 0044b090 · legions 0044bd30 — ac2-ui chat: opacity, scroll lock and the spew box are ported; save loc and selection are still open (remaining gaps in the chat's notes)
- [~] `ChatWindow::LocationApplied` · 0044b120 · legions 0044bdc0 — window positions are applied by the client's window-position code (AGENTS exception: saved per character, save_loc to the server); the chat has no separate LocationApplied
- [x] `ChatWindow::SetZLevel` · 0044b140 · legions 0044bde0 — ac2-ui chat.rs / chat_window.rs

### gmChatUI
- [-] `gmChatUI::RecvNotice_CreateObject` · 0044b150 — registers the chat notices on each new weenie; the client routes chat text itself (hud_view chat_display), so no per-weenie handler is needed

### ChatWindow::ChatInfo
- [x] `ChatWindow::ChatInfo::ChatInfo` · 0044b1a0 · legions 0044bdf0 — ac2-ui chat.rs / chat_window.rs

### ChatWindow
- [x] `ChatWindow::SetOpacity` · 0044b1b0 · legions 0044be20 — ac2-client: Command::SetChatOption / SetWindowOpacity (IFuns 0xB400D3/D4/DD/83, DF; SetWindowOpacity then UploadUISettings), UploadChatOptions (IFun 0xB400D5, WM_Player 0x2E asserted in chat_options_7160), InqGameplayOptions = GetGameplayOptions IFun 0xB400DE (scripts_queries gameplay_options); ac2-ui chat_window.rs opacity slider, apply_opacity, reload (window_locations_tests.rs, window_locations_7160.rs a_window_opacity...)
- [~] `ChatWindow::SetActiveWindow` · 0044b1f0 · legions 0044be60 — ac2-ui chat: opacity, scroll lock and the spew box are ported; save loc and selection are still open (remaining gaps in the chat's notes)
- [~] `ChatWindow::ListenToMessage` · 0044b220 · legions 0044c570 — ac2-ui chat.rs: log scroll bar lock (ChatLogScrollbar widget_press locks, widget_release unlocks; AllTextToLog skips the scroll); remaining: save loc (save_loc window), selection, Legions routes the lock on the root element

### PStringBaseIter_Common<unsigned_short>
- [-] `PStringBaseIter_Common<unsigned_short>::FindChar` · 0044b5b0 · legions 0044c090 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### ChatWindow
- [~] `ChatWindow::ProcessCommand` · 0044b650 · legions 0044da90 — ac2-ui chat: opacity, scroll lock and the spew box are ported; save loc and selection are still open (remaining gaps in the chat's notes)

### PStringBaseIter_Common<unsigned_short>
- [-] `PStringBaseIter_Common<unsigned_short>::FindChar` · 0044b730 · legions 0044c130 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### ChatWindow
- [x] `ChatWindow::HandleEnterKey` · 0044b810 · legions 0044db60 — ac2-ui chat.rs / chat_window.rs

### SmartArray<ChatWindow::ChatInfo>
- [-] `SmartArray<ChatWindow::ChatInfo>::AddToEnd` · 0044b880 · legions 0044c210 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### ChatWindow
- [x] `ChatWindow::RecvNotice_DisplayStringInfo` · 0044b8e0 — ac2-ui chat.rs / chat_window.rs
- [x] `ChatWindow::GetChatMessageInfo` · 0044ba70 · legions 0044c790 — ac2-ui chat.rs / chat_window.rs
- [~] `ChatWindow::DoAssistedTell` · 0044bad0 · legions 0044c7f0 — ac2-ui chat: opacity, scroll lock and the spew box are ported; save loc and selection are still open (remaining gaps in the chat's notes)

### PSUtils
- [ ] `PSUtils::FindAllWords` · 0044bc80 · legions 0044c990 — needs port (check): PSUtils::FindAllWords split in ac2-script string_utils.rs, cite is on the separator constant

### ChatWindow
- [-] `ChatWindow::~ChatWindow` · 0044beb0 · legions 0044cbc0 — destructor: Rust drop and UiTree roots replace the teardown
- [x] `ChatWindow::HandleCommandOrAliasKey` · 0044bf80 · legions 0044cc90 — ac2-ui chat.rs / chat_window.rs
- [x] `ChatWindow::AllTextToLog` · 0044c020 · legions 0044cd30 — ac2-ui chat_window.rs: pending lines write to the log, the log scrolls to its end unless the scroll bar holds the lock (tested on the real layout)
- [x] `ChatWindow::DisplayCmdHistory` · 0044c110 · legions 0044ce30 — ac2-ui chat + ac2-client chat_text
- [x] `ChatWindow::DisplayRetellHistory` · 0044c1f0 · legions 0044cf10 — ac2-ui chat + ac2-client chat_text
- [x] `ChatWindow::DisplayReplyHistory` · 0044c2a0 · legions 0044cfc0 — ac2-ui chat + ac2-client chat_text
- [x] `ChatWindow::HandleReplyKey` · 0044c400 · legions 0044d120 — ac2-ui chat.rs / chat_window.rs
- [~] `ChatWindow::ListenToMessage` · 0044c5b0 · legions 0044d2d0 — ac2-ui chat.rs: log scroll bar lock (ChatLogScrollbar widget_press locks, widget_release unlocks; AllTextToLog skips the scroll); remaining: save loc (save_loc window), selection, Legions routes the lock on the root element
- [~] `ChatWindow::KeyPress` · 0044c5f0 · legions 0044d310 — ac2-ui chat: opacity, scroll lock and the spew box are ported; save loc and selection are still open (remaining gaps in the chat's notes)
- [~] `ChatWindow::ChatWindow` · 0044c7d0 · legions 0044d700 — ac2-ui chat: opacity, scroll lock and the spew box are ported; save loc and selection are still open (remaining gaps in the chat's notes)
- [x] `ChatWindow::RecvNotice_ReloadOptions` · 0044cb10 — ac2-client: Command::SetChatOption / SetWindowOpacity (IFuns 0xB400D3/D4/DD/83, DF; SetWindowOpacity then UploadUISettings), UploadChatOptions (IFun 0xB400D5, WM_Player 0x2E asserted in chat_options_7160), InqGameplayOptions = GetGameplayOptions IFun 0xB400DE (scripts_queries gameplay_options); ac2-ui chat_window.rs opacity slider, apply_opacity, reload (window_locations_tests.rs, window_locations_7160.rs a_window_opacity...)
- [~] `ChatWindow::RecvNotice_ReloadChatSettings` · 0044cbc0 — ac2-ui chat: opacity, scroll lock and the spew box are ported; save loc and selection are still open (remaining gaps in the chat's notes)

## `ac2/client/src/ui/gameplay/helper/containerdisplay.cpp.c` (17/22)

### ContainerDisplay
- [x] `ContainerDisplay::FinishListBoxDrop` · 0042d910 · legions 0042d6e0 — ac2-ui item_display.rs

### (free functions)
- [x] `ContainerDisplay_DDCB` · 0042da30 — ac2-ui drag.rs DragCheck::ContainerDrop on each container list: the coin field or an item

### ContainerDisplay
- [~] `ContainerDisplay::Show` · 0042da90 · legions 0042d860 — ac2-ui external_container.rs ContainerDisplay show: the container field's visibility; backpack.rs show_pack for the pages
- [ ] `ContainerDisplay::RequestContext` · 0042dab0 · legions 0042d880 — needs port (check): ContainerDisplay::RequestContext counter; shop.rs cite is on a constant

### UIElement_ListBox
- [x] `UIElement_ListBox::WhatNum` · 0042dac0 · legions 0042d890 — ac2-ui list_box.rs

### ContainerDisplay
- [x] `ContainerDisplay::RecvNotice_StackUIDone` · 0042daf0 · legions 0042d8c0 — ac2-ui money_bag.rs / item_display.rs / backpack.rs; ac2-client ui_inventory.rs; ac2-script host/inventory_globals.rs (coin max) and the player's IFUN 0xB400D7 (bypass); Legions 7160 matches admin here (same IFUNs 0xA60001 and 0xB400D7)
- [x] `ContainerDisplay::GetNewIndex` · 0042db40 · legions 0042d910 — ac2-ui item_display.rs
- [ ] `ContainerDisplay::HandleDragFrom` · 0042dbc0 · legions 0042d990 — needs port (check): ContainerDisplay::HandleDragFrom behaviour; implemented area ac2-ui item_display.rs / shop.rs but uncited
- [x] `ContainerDisplay::HandleListBoxDrop` · 0042dc20 · legions 0042d9f0 — ac2-ui item_display.rs
- [x] `ContainerDisplay::HandleDragFromMoneyBag` · 0042de90 · legions 0042dc60 — ac2-ui money_bag.rs / item_display.rs / backpack.rs; ac2-client ui_inventory.rs; ac2-script host/inventory_globals.rs (coin max) and the player's IFUN 0xB400D7 (bypass); Legions 7160 matches admin here (same IFUNs 0xA60001 and 0xB400D7)
- [x] `ContainerDisplay::ListenToMessage` · 0042dfb0 · legions 0042de40 — ac2-ui item_display.rs
- [x] `ContainerDisplay::AddItem` · 0042e070 · legions 0042df00 — ac2-ui item_display.rs / backpack.rs
- [x] `ContainerDisplay::FindItem` · 0042e150 · legions 0042dfe0 — ac2-ui item_display.rs / backpack.rs
- [x] `ContainerDisplay::RemoveItem` · 0042e1e0 · legions 0042e070 — ac2-ui item_display.rs / backpack.rs
- [x] `ContainerDisplay::BeginRequest` · 0042e2a0 · legions 0042e130 — ac2-ui item_display.rs / backpack.rs
- [x] `ContainerDisplay::RequestFailed` · 0042e2d0 · legions 0042e160 — ac2-ui item_display.rs / backpack.rs
- [x] `ContainerDisplay::Flush` · 0042e300 · legions 0042e190 — ac2-ui item_display.rs / backpack.rs
- [-] `ContainerDisplay::ContainerDisplay` · 0042e3a0 · legions 0042e230 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own
- [x] `ContainerDisplay::Cleanup` · 0042e470 · legions 0042e300 — ac2-ui item_display.rs
- [x] `ContainerDisplay::GetContents` · 0042e560 · legions 0042e3f0 — ac2-ui item_display.rs / backpack.rs
- [~] `ContainerDisplay::Init` · 0042e650 · legions 0042e4e0 — icon descriptions show their first layer; Token_ShortcutData/FromID/Equipped attributes and drag-and-drop not modelled; GetContentsInfo answered from the frame until the client asks the VM
- [x] `ContainerDisplay::RecvNotice_UpdateContainer` · 0042e7b0 — ac2-ui item_display.rs / backpack.rs

## `ac2/client/src/ui/gameplay/helper/fellowinfo.cpp.c` (14/15)

### FellowInfo
- [-] `FellowInfo::~FellowInfo` · 00439ea0 · legions 0043a6f0
- [x] `FellowInfo::UpdateCellID` · 00439f30 · legions 0043a780 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs
- [x] `FellowInfo::UpdateLevel` · 00439f40 · legions 0043a790 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs
- [x] `FellowInfo::UpdatePercentage` · 00439fa0 · legions 0043a7f0 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs
- [x] `FellowInfo::ToggleTextState` · 0043a030 · legions 0043a880 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs
- [x] `FellowInfo::SetLeader` · 0043a060 · legions 0043a8b0 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs
- [x] `FellowInfo::ShowSelectionHighlight` · 0043a090 · legions 0043a8e0 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs
- [x] `FellowInfo::RecvNotice_UpdateFellowHealth` · 0043a0d0 · legions 0043a920 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs
- [x] `FellowInfo::RecvNotice_UpdateFellowMaxHealth` · 0043a110 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs
- [x] `FellowInfo::RecvNotice_UpdateFellowVigor` · 0043a150 · legions 0043a9a0 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs
- [x] `FellowInfo::RecvNotice_UpdateFellowMaxVigor` · 0043a190 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs
- [x] `FellowInfo::RecvNotice_UpdateFellowLevel` · 0043a1d0 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs
- [x] `FellowInfo::RecvNotice_BeingDeleted` · 0043a1f0 · legions 0043aa40 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs
- [x] `FellowInfo::FellowInfo` · 0043a200 · legions 0043aa50 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs
- [x] `FellowInfo::UpdateInfo` · 0043a4d0 · legions 0043ad20 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs

## `ac2/client/src/ui/gameplay/helper/fellowshiphelper.cpp.c` (13/24)

### FellowshipHelper
- [x] `FellowshipHelper::SelectFellowByIdx` · 0042f0f0 · legions 0042eff0 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs

### ItemDisplay
- [ ] `ItemDisplay::RecvNotice_BeingDeleted` · 0042f180 — Legions has it: vtable slot 10 of ItemDisplay at 0x0042eaf0 (vtable audit); was skip: no Legions 7160 counterpart (no legions_address in the index

### FellowshipHelper
- [x] `FellowshipHelper::ListenToMessage` · 0042f1a0 · legions 0042f080 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs

### HashIterator<InstanceID,float>
- [-] `HashIterator<InstanceID,float>::operator++` · 0042f2c0 · legions 0042f1a0 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### FellowshipHelper
- [x] `FellowshipHelper::ToggleTextState` · 0042f300 · legions 0042f220 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs

### PHashTable<InstanceID,float>
- [-] `PHashTable<InstanceID,float>::~PHashTable<InstanceID,float>` · 0042f4a0 · legions 0042f360 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### FellowshipHelper
- [x] `FellowshipHelper::ListenToMessage` · 0042f5f0 · legions 0042f4b0 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs
- [x] `FellowshipHelper::SetLeader` · 0042f6e0 · legions 0042f5a0 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs
- [x] `FellowshipHelper::RecvNotice_UpdateLeader` · 0042f7a0 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs
- [x] `FellowshipHelper::RecvNotice_SelectionChanged` · 0042f7c0 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs

### PHashTable<InstanceID,float>
- [-] `PHashTable<InstanceID,float>::StreamPack` · 0042f8e0 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### (free functions)
- [ ] `STREAMPACK` · 0042fb10 · legions 0042f9d0 — no call site in the Legions wlib (scan: host::tests::efun_coverage); lower priority

### FellowshipHelper
- [~] `FellowshipHelper::UpdateXP` · 0042fca0 · legions 0042fa60 — GetFellowXPHash needs the host to return the script-filled LAHash
- [x] `FellowshipHelper::RecvNotice_UpdateFellowLevel` · 0042fdc0 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs
- [~] `FellowshipHelper::RemoveFellow` · 0042fdd0 · legions 0042fb90 — map markers (SendNotice_AddMarker/RemoveMarker/ClearMarkers) not raised
- [~] `FellowshipHelper::RemoveAllFellows` · 0042feb0 · legions 0042fc70 — map markers (SendNotice_AddMarker/RemoveMarker/ClearMarkers) not raised
- [~] `FellowshipHelper::RecvNotice_UpdateFellowCellID` · 0042ff90 — map markers (SendNotice_AddMarker/RemoveMarker/ClearMarkers) not raised
- [x] `FellowshipHelper::RecvNotice_ClearFellow` · 00430060 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs
- [ ] `FellowshipHelper::RecvNotice_ReloadGameplayOptions` · 00430080 — needs port (check): FellowshipHelper::RecvNotice_ReloadGameplayOptions behaviour; implemented area ac2-ui fellowship.rs but uncited
- [x] `FellowshipHelper::~FellowshipHelper` · 00430180 · legions 0042ff40 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs
- [~] `FellowshipHelper::AddFellow` · 00430210 · legions 0042ffd0 — map markers (SendNotice_AddMarker/RemoveMarker/ClearMarkers) not raised
- [x] `FellowshipHelper::AddAllFellows` · 00430390 · legions 00430150 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs
- [x] `FellowshipHelper::RecvNotice_UpdateFellow` · 00430430 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs
- [x] `FellowshipHelper::FellowshipHelper` · 004304a0 · legions 00430260 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs

## `ac2/client/src/ui/gameplay/helper/itemdisplay.cpp.c` (8/15)

### ItemDisplay
- [x] `ItemDisplay::ItemDisplay` · 0042e900 · legions 0042e7e0 — ac2-ui item_display.rs / backpack.rs
- [-] `ItemDisplay::~ItemDisplay` · 0042e9d0 · legions 0042e8b0 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own
- [x] `ItemDisplay::ListenToMessage` · 0042eb30 · legions 0042ea10 — ac2-ui item_display.rs / backpack.rs
- [x] `ItemDisplay::RecvNotice_SelectionChanged` · 0042ebb0 · legions 0042ea90 — ac2-ui item_display.rs / backpack.rs
- [ ] `ItemDisplay::RecvNotice_SetTargetingPointer` · 0042ec10 — Legions has it: ItemDisplay notice vtable slot 146 at 0x0042eb10 (admin slot 145, shifted by one; same size and calls)
- [~] `ItemDisplay::Update` · 0042ece0 · legions 0042ebe0 — icon descriptions show their first layer; Token_ShortcutData/FromID/Equipped attributes and drag-and-drop not modelled; GetContentsInfo answered from the frame until the client asks the VM
- [x] `ItemDisplay::SetItem` · 0042eed0 · legions 0042edd0 — ac2-ui item_display.rs / backpack.rs
- [x] `ItemDisplay::SetPending` · 0042ef20 · legions 0042ee20 — ac2-ui item_display.rs / backpack.rs
- [x] `ItemDisplay::SetDraggable` · 0042ef30 · legions 0042ee30 — ac2-ui item_display.rs / backpack.rs
- [ ] `ItemDisplay::SetQuantity` · 0042ef40 · legions 0042ee40 — needs port (check): ItemDisplay::SetQuantity behaviour; implemented area ac2-ui item_display.rs but uncited
- [x] `ItemDisplay::SetShortcutType` · 0042ef60 · legions 0042ee60 — ItemDisplay::SetShortcutType done: item_display.rs set_shortcut_type cites it
- [x] `ItemDisplay::RecvNotice_ItemUpdate` · 0042ef90 — ac2-ui item_display.rs / backpack.rs
- [ ] `ItemDisplay::RecvNotice_LocalVisualDescUpdate` · 0042efc0 — Legions has it: vtable slot 14 of ItemDisplay at 0x0042eec0 (vtable audit); was skip: no Legions 7160 counterpart (no legions_address in the index
- [ ] `ItemDisplay::RecvNotice_CreateObject` · 0042eff0 — Legions has it: vtable slot 11 of ItemDisplay at 0x0042eef0 (vtable audit); was skip: no Legions 7160 counterpart (no legions_address in the index
- [~] `ItemDisplay::Init` · 0042f020 · legions 0042ef20 — icon descriptions show their first layer; Token_ShortcutData/FromID/Equipped attributes and drag-and-drop not modelled; GetContentsInfo answered from the frame until the client asks the VM

## `ac2/client/src/ui/gameplay/helper/optionbase.cpp.c` (6/7)

### OptionBase
- [x] `OptionBase::OptionBase` · 0044f7b0 · legions 004508f0 — character.rs and the option widgets' states: the element from its template, its label and tooltip (Describe); tested on the real layout
- [-] `OptionBase::~OptionBase` · 0044f7e0 · legions 00450920 — C++ destructor and scalar deleter: the element is owned by the tree and dropped with it
- [x] `OptionBase::Create` · 0044f810 · legions 00450950 — character.rs and the option widgets' states: the element from its template, its label and tooltip (Describe); tested on the real layout
- [x] `OptionBase::Retrieve` · 0044f850 · legions 00450990 — character.rs and the option widgets' states: the element from its template, its label and tooltip (Describe); tested on the real layout
- [x] `OptionBase::Describe` · 0044f870 · legions 004509b0 — character.rs and the option widgets' states: the element from its template, its label and tooltip (Describe); tested on the real layout
- [x] `OptionBase::Describe` · 0044f890 · legions 004509d0 — character.rs and the option widgets' states: the element from its template, its label and tooltip (Describe); tested on the real layout
- [x] `OptionBase::Identify` · 0044f940 · legions 00450a80 — character.rs and the option widgets' states: the element from its template, its label and tooltip (Describe); tested on the real layout

## `ac2/client/src/ui/gameplay/helper/optionchoice.cpp.c` (11/15)

### OptionChoice
- [x] `OptionChoice::OptionChoice` · 0044df20 · legions 0044f0f0 — ac2-ui character_options.rs (fill_choices, select_choice, selected_choice, sync_option) and character.rs / chat_options.rs drop-downs: UIElement_Menu items keyed by choice number (tested on the real layout in character_tests.rs, chat_options_tests.rs)
- [-] `OptionChoice::Cast` · 0044dfa0 · legions 0044f170 — C++ downcast (Cast) and scalar deleter: the page keeps typed entries
- [x] `OptionChoice::AddChoice` · 0044dfe0 · legions 0044f1b0 — ac2-ui character_options.rs (fill_choices, select_choice, selected_choice, sync_option) and character.rs / chat_options.rs drop-downs: UIElement_Menu items keyed by choice number (tested on the real layout in character_tests.rs, chat_options_tests.rs)
- [~] `OptionChoice::AssignChoice` · 0044e060 · legions 0044f230 — ac2-ui character_options.rs: the menu's new selection is read into the page's value (ListenToMessage's selection); PreferenceBank::Change and Merge's preference copy are not modelled, the page's Apply sends the setter instead
- [x] `OptionChoice::SetCurrent` · 0044e090 · legions 0044f260 — ac2-ui character_options.rs (fill_choices, select_choice, selected_choice, sync_option) and character.rs / chat_options.rs drop-downs: UIElement_Menu items keyed by choice number (tested on the real layout in character_tests.rs, chat_options_tests.rs)
- [x] `OptionChoice::SetDefault` · 0044e140 · legions 0044f310 — character_options.rs / character.rs: the choice's default, current, changed, apply, recall, default and refresh (tested)
- [x] `OptionChoice::GetCurrent` · 0044e180 · legions 0044f350 — character_options.rs / character.rs: the choice's default, current, changed, apply, recall, default and refresh (tested)
- [x] `OptionChoice::Changed` · 0044e1c0 · legions 0044f390 — character_options.rs / character.rs: the choice's default, current, changed, apply, recall, default and refresh (tested)
- [x] `OptionChoice::Apply` · 0044e210 · legions 0044f3e0 — character_options.rs / character.rs: the choice's default, current, changed, apply, recall, default and refresh (tested)
- [x] `OptionChoice::Recall` · 0044e260 · legions 0044f430 — character_options.rs / character.rs: the choice's default, current, changed, apply, recall, default and refresh (tested)
- [x] `OptionChoice::Default` · 0044e280 · legions 0044f450 — character_options.rs / character.rs: the choice's default, current, changed, apply, recall, default and refresh (tested)
- [~] `OptionChoice::ListenToMessage` · 0044e2a0 · legions 0044f470 — ac2-ui character_options.rs: the menu's new selection is read into the page's value (ListenToMessage's selection); PreferenceBank::Change and Merge's preference copy are not modelled, the page's Apply sends the setter instead
- [x] `OptionChoice::Refresh` · 0044e3a0 · legions 0044f570 — character_options.rs / character.rs: the choice's default, current, changed, apply, recall, default and refresh (tested)
- [x] `OptionChoice::Refresh` · 0044e3c0 · legions 0044f590 — character_options.rs / character.rs: the choice's default, current, changed, apply, recall, default and refresh (tested)
- [~] `OptionChoice::Merge` · 0044e590 · legions 0044f6d0 — ac2-ui character_options.rs: the menu's new selection is read into the page's value (ListenToMessage's selection); PreferenceBank::Change and Merge's preference copy are not modelled, the page's Apply sends the setter instead

## `ac2/client/src/ui/gameplay/helper/optionexpander.cpp.c` (9/16)

### OptionExpander
- [~] `OptionExpander::CountOptions` · 0044eb80 · legions 0044fcc0 — the count and the refresh over the children are the page's; the list box's Refit sizes are not modelled
- [~] `OptionExpander::Refit` · 0044eb90 · legions 0044fcd0 — the count and the refresh over the children are the page's; the list box's Refit sizes are not modelled
- [~] `OptionExpander::Refresh` · 0044ec30 · legions 0044fd70 — the count and the refresh over the children are the page's; the list box's Refit sizes are not modelled
- [~] `OptionExpander::Refresh` · 0044ec80 · legions 0044fdc0 — the count and the refresh over the children are the page's; the list box's Refit sizes are not modelled
- [-] `OptionExpander::GetOption` · 0044ed10 · legions 0044fe50 — index access (the page keeps children as entries), the destructor, Clear (the tree drops the rows) and the scalar deleter
- [x] `OptionExpander::Construct` · 0044ed30 · legions 0044fe70 — character.rs: the expander's header toggles its option rows (Construct / Dismantle into TreeList), its changed / apply / recall / default over its children (tested)
- [x] `OptionExpander::Dismantle` · 0044edb0 · legions 0044fef0 — character.rs: the expander's header toggles its option rows (Construct / Dismantle into TreeList), its changed / apply / recall / default over its children (tested)
- [x] `OptionExpander::Changed` · 0044ee10 · legions 0044ff50 — character.rs: the expander's header toggles its option rows (Construct / Dismantle into TreeList), its changed / apply / recall / default over its children (tested)
- [x] `OptionExpander::Apply` · 0044ee50 · legions 0044ff90 — character.rs: the expander's header toggles its option rows (Construct / Dismantle into TreeList), its changed / apply / recall / default over its children (tested)
- [x] `OptionExpander::Recall` · 0044ee80 · legions 0044ffc0 — character.rs: the expander's header toggles its option rows (Construct / Dismantle into TreeList), its changed / apply / recall / default over its children (tested)
- [x] `OptionExpander::Default` · 0044eeb0 · legions 0044fff0 — character.rs: the expander's header toggles its option rows (Construct / Dismantle into TreeList), its changed / apply / recall / default over its children (tested)
- [x] `OptionExpander::ListenToMessage` · 0044eee0 · legions 00450020 — character.rs: the expander's header toggles its option rows (Construct / Dismantle into TreeList), its changed / apply / recall / default over its children (tested)
- [x] `OptionExpander::OptionExpander` · 0044ef30 · legions 00450070 — character.rs: the expander's header toggles its option rows (Construct / Dismantle into TreeList), its changed / apply / recall / default over its children (tested)
- [-] `OptionExpander::~OptionExpander` · 0044efc0 · legions 00450100 — index access (the page keeps children as entries), the destructor, Clear (the tree drops the rows) and the scalar deleter
- [-] `OptionExpander::Clear` · 0044f030 · legions 00450170 — index access (the page keeps children as entries), the destructor, Clear (the tree drops the rows) and the scalar deleter
- [x] `OptionExpander::AddOption` · 0044f0b0 · legions 004501f0 — character.rs: the expander's header toggles its option rows (Construct / Dismantle into TreeList), its changed / apply / recall / default over its children (tested)

## `ac2/client/src/ui/gameplay/helper/optionflag.cpp.c` (9/13)

### OptionFlag
- [~] `OptionFlag::OptionFlag` · 0044e6d0 · legions 0044f810 — the checkbox, its value and the messages are ported; PreferenceBank::Change / ConnectSource (the bank's handlers) are not modelled
- [-] `OptionFlag::Cast` · 0044e760 · legions 0044f8a0 — C++ downcast (Cast) and scalar deleter: the page keeps typed entries; no runtime cast needed
- [~] `OptionFlag::Merge` · 0044e7a0 · legions 0044f8e0 — the checkbox, its value and the messages are ported; PreferenceBank::Change / ConnectSource (the bank's handlers) are not modelled
- [x] `OptionFlag::SetCurrent` · 0044e800 · legions 0044f940 — character_options.rs / character.rs: the checkbox's value, default, changed, apply, recall, default and refresh over the option's current and saved values (tested)
- [x] `OptionFlag::SetDefault` · 0044e850 · legions 0044f990 — character_options.rs / character.rs: the checkbox's value, default, changed, apply, recall, default and refresh over the option's current and saved values (tested)
- [x] `OptionFlag::GetCurrent` · 0044e8b0 · legions 0044f9f0 — character_options.rs / character.rs: the checkbox's value, default, changed, apply, recall, default and refresh over the option's current and saved values (tested)
- [x] `OptionFlag::Changed` · 0044e8e0 · legions 0044fa20 — character_options.rs / character.rs: the checkbox's value, default, changed, apply, recall, default and refresh over the option's current and saved values (tested)
- [x] `OptionFlag::Apply` · 0044e920 · legions 0044fa60 — character_options.rs / character.rs: the checkbox's value, default, changed, apply, recall, default and refresh over the option's current and saved values (tested)
- [x] `OptionFlag::Recall` · 0044e960 · legions 0044faa0 — character_options.rs / character.rs: the checkbox's value, default, changed, apply, recall, default and refresh over the option's current and saved values (tested)
- [x] `OptionFlag::Default` · 0044e9b0 · legions 0044faf0 — character_options.rs / character.rs: the checkbox's value, default, changed, apply, recall, default and refresh over the option's current and saved values (tested)
- [~] `OptionFlag::ListenToMessage` · 0044ea00 · legions 0044fb40 — the checkbox, its value and the messages are ported; PreferenceBank::Change / ConnectSource (the bank's handlers) are not modelled
- [x] `OptionFlag::Refresh` · 0044eae0 · legions 0044fc20 — character_options.rs / character.rs: the checkbox's value, default, changed, apply, recall, default and refresh over the option's current and saved values (tested)
- [x] `OptionFlag::Refresh` · 0044eb30 · legions 0044fc70 — character_options.rs / character.rs: the checkbox's value, default, changed, apply, recall, default and refresh over the option's current and saved values (tested)

## `ac2/client/src/ui/gameplay/helper/optionvalue.cpp.c` (10/14)

### OptionValue
- [~] `OptionValue::OptionValue` · 0044f100 · legions 00450240 — the slider and its arrows (a tenth of the range) and the position message are ported; the preference bank's handlers and the float preference are not modelled
- [-] `OptionValue::Cast` · 0044f1f0 · legions 00450330 — C++ downcast (Cast) and scalar deleter: the page keeps typed entries
- [~] `OptionValue::Merge` · 0044f230 · legions 00450370 — the slider and its arrows (a tenth of the range) and the position message are ported; the preference bank's handlers and the float preference are not modelled
- [x] `OptionValue::SetRange` · 0044f2a0 · legions 004503e0 — character_options.rs / character.rs: the slider's range, value (clamped), default, changed (the 1e-4 squared test), apply, recall, default and refresh (tested)
- [x] `OptionValue::SetCurrent` · 0044f2f0 · legions 00450430 — character_options.rs / character.rs: the slider's range, value (clamped), default, changed (the 1e-4 squared test), apply, recall, default and refresh (tested)
- [x] `OptionValue::SetDefault` · 0044f3d0 · legions 00450510 — character_options.rs / character.rs: the slider's range, value (clamped), default, changed (the 1e-4 squared test), apply, recall, default and refresh (tested)
- [x] `OptionValue::GetCurrent` · 0044f460 · legions 004505a0 — character_options.rs / character.rs: the slider's range, value (clamped), default, changed (the 1e-4 squared test), apply, recall, default and refresh (tested)
- [x] `OptionValue::Changed` · 0044f4d0 · legions 00450610 — character_options.rs / character.rs: the slider's range, value (clamped), default, changed (the 1e-4 squared test), apply, recall, default and refresh (tested)
- [x] `OptionValue::Apply` · 0044f510 · legions 00450650 — character_options.rs / character.rs: the slider's range, value (clamped), default, changed (the 1e-4 squared test), apply, recall, default and refresh (tested)
- [x] `OptionValue::Recall` · 0044f540 · legions 00450680 — character_options.rs / character.rs: the slider's range, value (clamped), default, changed (the 1e-4 squared test), apply, recall, default and refresh (tested)
- [x] `OptionValue::Default` · 0044f570 · legions 004506b0 — character_options.rs / character.rs: the slider's range, value (clamped), default, changed (the 1e-4 squared test), apply, recall, default and refresh (tested)
- [~] `OptionValue::ListenToMessage` · 0044f5a0 · legions 004506e0 — the slider and its arrows (a tenth of the range) and the position message are ported; the preference bank's handlers and the float preference are not modelled
- [x] `OptionValue::Refresh` · 0044f750 · legions 00450890 — character_options.rs / character.rs: the slider's range, value (clamped), default, changed (the 1e-4 squared test), apply, recall, default and refresh (tested)
- [x] `OptionValue::Refresh` · 0044f780 · legions 004508c0 — character_options.rs / character.rs: the slider's range, value (clamped), default, changed (the 1e-4 squared test), apply, recall, default and refresh (tested)

## `ac2/client/src/ui/gameplay/helper/timedisplay.cpp.c` (3/4)

### TimeDisplay
- [x] `TimeDisplay::TimeDisplay` · 0043dd20 · legions 0043e570 — ac2-ui journal_time.rs QuestTimeDisplay (init, global_loop)
- [~] `TimeDisplay::ComputeTimeLeft` · 0043ddb0 · legions 0043e600 — ac2-ui journal_time.rs time_left: the words and thresholds; minute and second fields two digits wide (sprintf format not recovered)
- [x] `TimeDisplay::ListenToMessage` · 0043e080 · legions 0043e8d0 — ac2-ui journal_time.rs QuestTimeDisplay (init, global_loop)
- [x] `TimeDisplay::Init` · 0043e0d0 · legions 0043e920 — ac2-ui journal_time.rs QuestTimeDisplay (init, global_loop)

## `ac2/client/src/ui/gameplay/helper/vitaldisplay.cpp.c` (7/7)

### VitalDisplay
- [x] `VitalDisplay::VitalDisplay` · 0044dc80 · legions 0044ee50 — ac2-ui meter.rs
- [x] `VitalDisplay::Update` · 0044dcd0 · legions 0044eea0 — ac2-ui meter.rs
- [x] `VitalDisplay::UpdateCurrentVal` · 0044de30 · legions 0044f000 — ac2-ui meter.rs
- [x] `VitalDisplay::UpdateMaxVal` · 0044de50 · legions 0044f020 — ac2-ui meter.rs
- [x] `VitalDisplay::ToggleTextState` · 0044de70 · legions 0044f040 — ac2-ui meter.rs
- [x] `VitalDisplay::SetTextState` · 0044dea0 · legions 0044f070 — ac2-ui meter.rs
- [x] `VitalDisplay::SetMeterState` · 0044ded0 · legions 0044f0a0 — ac2-ui meter.rs

## `ac2/client/src/ui/gameplay/statusbar/gmeffectsui.cpp.c` (10/10)

### gmEffectsUI
- [x] `gmEffectsUI::SelectElement` · 00438a20 · legions 00439270 — ac2-ui effects.rs
- [x] `gmEffectsUI::ListenToMessage` · 00438b80 — ac2-ui effects.rs
- [x] `gmEffectsUI::RecvNotice_PulseMissed` · 00438c30 — ac2-ui effects.rs

### PList<EffectUINode>
- [x] `PList<EffectUINode>::Pack` · 00438ef0 · legions 00439740 — ac2-net native/gm.rs pack_effect_list / read_effect_list: count then EffectUINode::StreamPack; no retail caller in the index, so no crate packs it yet
- [x] `PList<EffectUINode>::GetPackSize` · 00438f40 — ac2-net native/gm.rs pack_effect_list / read_effect_list: count then EffectUINode::StreamPack; no retail caller in the index, so no crate packs it yet
- [x] `PList<EffectUINode>::UnPack` · 00438f80 — ac2-net native/gm.rs pack_effect_list / read_effect_list: count then EffectUINode::StreamPack; no retail caller in the index, so no crate packs it yet

### gmEffectsUI
- [x] `gmEffectsUI::Init` · 004391c0 · legions 00439a10 — ac2-ui effects.rs init: insertion by expiry, removal from the end, selection, pulsed/active meter; EffectUINode selection without a zero guard
- [x] `gmEffectsUI::RecvNotice_ExaminationRequestChanged` · 004396b0 — ac2-ui effects.rs
- [x] `gmEffectsUI::RecvNotice_UpdateEffect` · 00439700 — ac2-ui effects.rs
- [x] `gmEffectsUI::gmEffectsUI` · 00439710 · legions 00439f60 — ac2-ui effects.rs

## `ac2/client/src/ui/gameplay/statusbar/gmstatusbarui.cpp.c` (3/3)

### gmStatusbarUI
- [x] `gmStatusbarUI::Show` · 004231b0 · legions 00422f80 — ac2-ui statusbar.rs show_statusbar (field shows or hides; top preclude 0); hud.rs makes vitals, effects, indicators in SetupChildren order
- [x] `gmStatusbarUI::SetupChildren` · 00423210 · legions 00422fe0 — ac2-ui statusbar.rs show_statusbar (field shows or hides; top preclude 0); hud.rs makes vitals, effects, indicators in SetupChildren order
- [x] `gmStatusbarUI::gmStatusbarUI` · 004232f0 · legions 004230c0 — ac2-ui hud.rs + statusbar.rs: root, SetupChildren, and Show(1) once the children exist; ac2-gpu ui.rs + ac2-client hud_view.rs draw it

## `ac2/client/src/ui/gameplay/statusbar/gmvitalsui.cpp.c` (5/6)

### gmVitalsUI
- [-] `gmVitalsUI::~gmVitalsUI` · 004397d0 · legions 0043a020
- [x] `gmVitalsUI::RecvNotice_UpdateVital` · 004398c0 — ac2-ui vitals.rs
- [x] `gmVitalsUI::SetTooltip` · 00439930 · legions 0043a180 — ac2-ui mini_fellow.rs, vitals.rs (tooltips per the numbers' toggle and the hero status); window location save and apply are the generic tree mechanism (mini_fellow_tests the_mini_panel_saves_its_location...)
- [x] `gmVitalsUI::gmVitalsUI` · 004399a0 · legions 0043a1f0 — ac2-ui vitals.rs
- [x] `gmVitalsUI::ListenToMessage` · 00439d30 · legions 0043a580 — ac2-ui mini_fellow.rs, vitals.rs (tooltips per the numbers' toggle and the hero status); window location save and apply are the generic tree mechanism (mini_fellow_tests the_mini_panel_saves_its_location...)
- [x] `gmVitalsUI::RecvNotice_UpdateHeroStatus` · 00439e00 · legions 0043a650 — ac2-ui mini_fellow.rs, vitals.rs (tooltips per the numbers' toggle and the hero status); window location save and apply are the generic tree mechanism (mini_fellow_tests the_mini_panel_saves_its_location...)

## `ac2/client/src/ui/gameplay/statusbar/indicators/gmindicatorsui.cpp.c` (2/4)

### gmIndicatorsUI
- [~] `gmIndicatorsUI::Show` · 00438850 · legions 004390a0 — ac2-ui indicators.rs: no root element of its own, so only the shown flag is kept; the children show notification (vtable 0x20) is not modelled
- [x] `gmIndicatorsUI::SetupChildren` · 00438860 · legions 004390b0 — ac2-ui indicators.rs: net, PK and vitae children made in retail order (net_status.rs, pk_status.rs); the child list and the children show notification are not kept
- [~] `gmIndicatorsUI::CleanupChildren` · 00438910 · legions 00439160 — ac2-ui: the indicators children are fields of GameplayHud and drop with its tree; no per-child teardown call
- [x] `gmIndicatorsUI::gmIndicatorsUI` · 004389a0 · legions 004391f0 — ac2-ui indicators.rs: net, PK and vitae children made in retail order (net_status.rs, pk_status.rs); the child list and the children show notification are not kept

## `ac2/client/src/ui/gameplay/statusbar/indicators/gmnetstatusui.cpp.c` (9/11)

### GraphTool
- [x] `GraphTool::GraphTool` · 0044d1a0 · legions 0044e370 — ac2-ui net_status.rs draw_bandwidth_graph (GraphTool bars, unit-tested); ac2-client hud_view/compose.rs paints it on each status bar compose; the latency-graph option from hud_player reload
- [x] `GraphTool::BlendColors` · 0044d220 · legions 0044e3f0 — ac2-ui net_status.rs draw_bandwidth_graph (GraphTool bars, unit-tested); ac2-client hud_view/compose.rs paints it on each status bar compose; the latency-graph option from hud_player reload

### gmNetStatusUI
- [x] `gmNetStatusUI::RecvNotice_ReloadGameplayOptions` · 0044d2a0 · legions 0044e470 — ac2-ui net_status.rs draw_bandwidth_graph (GraphTool bars, unit-tested); ac2-client hud_view/compose.rs paints it on each status bar compose; the latency-graph option from hud_player reload
- [x] `gmNetStatusUI::RecvNotice_ReloadOptions` · 0044d2c0 · legions 0044e490 — ac2-ui net_status.rs draw_bandwidth_graph (GraphTool bars, unit-tested); ac2-client hud_view/compose.rs paints it on each status bar compose; the latency-graph option from hud_player reload

### GraphTool
- [x] `GraphTool::DrawBar` · 0044d2e0 · legions 0044e4b0 — ac2-ui net_status.rs draw_bandwidth_graph (GraphTool bars, unit-tested); ac2-client hud_view/compose.rs paints it on each status bar compose; the latency-graph option from hud_player reload

### gmNetStatusUI
- [~] `gmNetStatusUI::DrawBandwidthGraph` · 0044d3d0 · legions 0044e5a0 — ac2-ui net_status.rs draw_bandwidth_graph paints the kept history ring (GraphTool bars, unit-tested, DAT test on the real HUD); painted by ac2-client hud_view/compose.rs on each status bar compose. Not modelled: SurfaceWindow::Blit flags 0x14 (retail blits a 32 x 32 surface; the port writes the pixels directly) and m_bGraphBright (no writer in the Legions tree)

### SmartArray<gmNetStatusUI::GraphHistoryColumn>
- [x] `SmartArray<gmNetStatusUI::GraphHistoryColumn>::AddNElements` · 0044d720 · legions 0044e8f0 — ac2-ui net_status.rs: ring resized to the graph width in push_column (AddNElements); destructor is Rust drop

### gmNetStatusUI
- [x] `gmNetStatusUI::~gmNetStatusUI` · 0044d750 · legions 0044e920 — ac2-ui net_status.rs: ring resized to the graph width in push_column (AddNElements); destructor is Rust drop
- [x] `gmNetStatusUI::OnHeartbeat` · 0044d800 · legions 0044e9d0 — ac2-client hud_player/hud_status: SessionEvent::LinkHeartbeat -> NetStatusUi::on_heartbeat (ac2-ui net_status.rs)
- [x] `gmNetStatusUI::ListenToMessage` · 0044da90 · legions 0044ec60 — ac2-ui net_status.rs draw_bandwidth_graph (GraphTool bars, unit-tested); ac2-client hud_view/compose.rs paints it on each status bar compose; the latency-graph option from hud_player reload
- [~] `gmNetStatusUI::gmNetStatusUI` · 0044dab0 · legions 0044ec80 — ac2-ui net_status.rs create; client routes LinkHeartbeat (hud_player). Not modelled: CWeenieObject notice registrations (0x46d, 0x417) and the CLinkStatusPlugin registration

## `ac2/client/src/ui/gameplay/statusbar/indicators/gmpkstatusui.cpp.c` (1/2)

### gmPKStatusUI
- [~] `gmPKStatusUI::gmPKStatusUI` · 0044d040 · legions 0044e210 — ac2-ui pk_status.rs PkStatusUi::create: PKStatusField lookup. Not modelled: the two CWeenieObject notice registrations (0x460 is the faction notice, RecvNotice_UpdateFactionStatus; 0x1040c not traced)
- [x] `gmPKStatusUI::RecvNotice_UpdateFactionStatus` · 0044d0c0 — PK field fed: Legions 0x0044E290 is gmPKStatusUI::RecvNotice_UpdateFactionStatus (the index missed its match; 1 peace, 2 control, 3 factional, 4 ffa, else stateless), raised by CM_Character::SendNotice_UpdateFactionStatus; routed through ac2-client hud_status::pk_notice

## `ac2/client/src/ui/gameplay/statusbar/indicators/gmvitaestatusui.cpp.c` (4/4)

### gmVitaeStatusUI
- [x] `gmVitaeStatusUI::gmVitaeStatusUI` · 0044cd80 · legions 0044df50 — ac2-ui vitae.rs (update_vitae, examination_request_changed, message); wired in ac2-client hud_status.rs; ctor notice handlers 0x45d, 0x461, 0x1040c
- [x] `gmVitaeStatusUI::RecvNotice_ExaminationRequestChanged` · 0044ce40 · legions 0044e010 — ac2-ui vitae.rs (update_vitae, examination_request_changed, message); wired in ac2-client hud_status.rs; ctor notice handlers 0x45d, 0x461, 0x1040c
- [x] `gmVitaeStatusUI::RecvNotice_UpdateVitae` · 0044ce90 — ac2-ui vitae.rs (update_vitae, examination_request_changed, message); wired in ac2-client hud_status.rs; ctor notice handlers 0x45d, 0x461, 0x1040c
- [x] `gmVitaeStatusUI::ListenToMessage` · 0044cfc0 — ac2-ui vitae.rs (update_vitae, examination_request_changed, message); wired in ac2-client hud_status.rs; ctor notice handlers 0x45d, 0x461, 0x1040c

## `ac2/client/src/ui/gameplay/toolbar/gmchatui.cpp.c` (8/14)

### gmChatUI
- [x] `gmChatUI::SetLastFocus` · 004351c0 · legions 00435990 — ac2-ui chat.rs / chat_window.rs
- [-] `gmChatUI::~gmChatUI` · 004351d0 · legions 004359f0 — destructor: Rust drop and UiTree roots replace the teardown
- [x] `gmChatUI::DetermineTargetWindow` · 00435280 · legions 00435aa0 — ac2-ui chat.rs / chat_window.rs
- [x] `gmChatUI::UpdateZLevels` · 00435320 · legions 00435b40 — ac2-ui chat.rs / chat_window.rs
- [x] `gmChatUI::SetActiveWindow` · 00435370 · legions 00435b90 — ac2-ui chat.rs / chat_window.rs
- [x] `gmChatUI::RecvNotice_DoAssistedTell` · 004353c0 — ac2-ui chat.rs / chat_window.rs
- [x] `gmChatUI::DispatchKeypressToWindow` · 00435550 · legions 00435d70 — ac2-ui chat.rs / chat_window.rs

### SmartArray<StringInfo>
- [-] `SmartArray<StringInfo>::AddToEnd` · 004355d0 · legions 00435e70 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### gmChatUI
- [~] `gmChatUI::Init` · 00435620 · legions 00435ec0 — ac2-ui chat.rs/chat_spew.rs: spew list made from the layout (0x4100000f) and global loop Update done; Show's Update on first show not modelled (chat always shown); object notice registration (0x419/0x1040c) is routed by the client
- [x] `gmChatUI::RecvNotice_DisplayStringInfo` · 004357d0 — ac2-ui chat.rs display: the error bit (type & 1) queues the spew box line; admin 7148 body (Legions handler unmatched in the index)
- [~] `gmChatUI::gmChatUI` · 004357f0 · legions 00436080 — ac2-ui chat_spew.rs: the spew list is found by id over the tree (GetElement); it sits in the status bar root through a reference child (holder 0x10059476), mouse-invisible, at the reference rect (314,0); Show's Update on first show not modelled (chat always shown); object notice registration (0x419/0x1040c) is routed by the client
- [x] `gmChatUI::Update` · 004358d0 · legions 00436160 — ac2-ui chat_spew.rs: expired lines leave, pending lines show with decay_time from SpewBoxMessage (6 s in the DAT); tested on the real chat layout (chat_tests)
- [~] `gmChatUI::Show` · 00435b80 · legions 00436410 — ac2-ui chat.rs/chat_spew.rs: spew list made from the layout (0x4100000f) and global loop Update done; Show's Update on first show not modelled (chat always shown); object notice registration (0x419/0x1040c) is routed by the client
- [~] `gmChatUI::ListenToMessage` · 00435bc0 · legions 00436450 — ac2-ui chat.rs/chat_spew.rs: spew list made from the layout (0x4100000f) and global loop Update done; Show's Update on first show not modelled (chat always shown); object notice registration (0x419/0x1040c) is routed by the client

## `ac2/client/src/ui/gameplay/toolbar/gmexaminationui.cpp.c` (10/14)

### gmExaminationUI
- [x] `gmExaminationUI::RecvNotice_OpenExaminationPanel` · 00430810 — ac2-ui toolbar.rs

### gmMiniFellowUI
- [~] `gmMiniFellowUI::RecvNotice_BeingDeleted` · 00430820 — ac2-ui: no per-object notice registration in Rust; the mini panel lives with the HUD and is rebuilt with it

### gmExaminationUI
- [x] `gmExaminationUI::MakeString` · 00430840 · legions 00430690 — ac2-ui examination.rs + ac2-client ui_sources
- [x] `gmExaminationUI::DoBreak` · 004309a0 · legions 004307f0 — ac2-ui examination.rs + ac2-client ui_sources
- [x] `gmExaminationUI::CleanupArray` · 00430a00 · legions 00430850 — ac2-ui examination.rs start_rows: the list flushed, the countdown timers and the row cursor cleared (Update and the destructor use it the same way)
- [~] `gmExaminationUI::ProcessEffectIDNode` · 00430a50 · legions 004308e0 — ac2-client ui_examination.rs: each effect node asks the script (IFun 0xB40123 with context, order, id and an empty profile) and is replaced by the nodes it fills; argument order is from the glue, not checked against a live script. Icon layers and filter tabs not modelled
- [-] `gmExaminationUI::~gmExaminationUI` · 00430ad0 · legions 004317b0 — C++ destructor: the Rust drop and the tree own the rows, timers and notice handlers
- [x] `gmExaminationUI::Update` · 00430b60 · legions 004312c0 — ac2-ui examination.rs + ac2-client ui_sources
- [x] `gmExaminationUI::Show` · 00430c00 · legions 00431880 — ac2-ui examination.rs shown_changed/follow_field: Update then StartExamination when shown, StopExamination then the empty row when hidden (the filter tabs are not part of Show)
- [x] `gmExaminationUI::ListenToMessage` · 00430c80 — ac2-ui examination.rs message + examination_tabs.rs: tab presses (Ctrl toggles bits) and SELECTION_NEXT/PREV_EXAM_TAB_KEY choose the filter and restart the examination with it (Legions 0x00431990, 0x00431900); wired in hud_message.rs; tests examination_tests.rs
- [x] `gmExaminationUI::RecvNotice_ExaminationRequestChanged` · 00430d80 — ac2-ui examination.rs + ac2-client ui_sources
- [~] `gmExaminationUI::ProcessNode` · 00430db0 · legions 00430ae0 — countdowns tick (QuestTimeDisplay, global loop); effect nodes expanded by the client. Icon descriptions show their first layer only (SetIconDesc layers not drawn)
- [x] `gmExaminationUI::gmExaminationUI` · 004312e0 — ac2-ui examination.rs create: field, list, timers, hidden start, the four filter tabs (GeneralTab 0x203, DetailsTab 0x18C, EffectsTab 0x10, UsageTab 0x60) in the construction state (Legions 0x00431C40); ExaminationProfile now carries the trailing available-tabs word (ac2-net)
- [x] `gmExaminationUI::RecvNotice_UpdateExaminationProfile` · 00431410 — ac2-ui examination.rs + ac2-client ui_sources

## `ac2/client/src/ui/gameplay/toolbar/gmnavigationui.cpp.c` (27/37)

### LandDefs
- [x] `LandDefs::inbound_valid_cellid` · 004314c0 · legions 00431dc0 — ac2-script glue::position (ported beside the gPosition glue)

### gmNavigationUI
- [x] `gmNavigationUI::AutoShowScrollButtons` · 00431550 · legions 00431e50 — ac2-ui navigation_detail.rs (window, corners, tiles, set_lock, pan, switch); navigation.rs (SizeMap detail branch via tile_position); tests navigation_detail_tests.rs and ac2-client tests/navigation_detail_7160.rs
- [x] `gmNavigationUI::UpdateMacroMap` · 004315f0 · legions 00431ef0 — ac2-world region/game_time.rs (GameClock::date_time: YEAR, DAY, YEARSPEC, SEASON, TIME ids), ac2-client ui_sources::game_date_time rendered for the frame, ac2-ui navigation.rs (DateText and TimeText in the macro view, CoordinatesText); tests game_time_real_tests.rs, navigation_tests, navigation_detail_7160.rs
- [x] `gmNavigationUI::UpdateLockPositionButtonToolTip` · 004316d0 · legions 00431fd0 — Legions 7160 layout has no LockPositionButton (the element is absent, so the function does nothing there); the lock state and its tooltip id (navigation_detail lock_tooltip_id) are kept for the layout that has it
- [x] `gmNavigationUI::ConvertLandscapeToPixel` · 00431760 · legions 00432060 — ac2-ui navigation_map.rs and navigation.rs: the macro view (238 constant, MapX/MapY/MapW/MapH from the layout); the detail view's 250 branch is in the same formula but no detail maps are read yet
- [x] `gmNavigationUI::ConvertPixelToLandscape` · 00431860 · legions 00432160 — ac2-ui navigation_map.rs and navigation.rs: the macro view (238 constant, MapX/MapY/MapW/MapH from the layout); the detail view's 250 branch is in the same formula but no detail maps are read yet
- [x] `gmNavigationUI::CenterMarker` · 00431950 · legions 00432250 — ac2-ui navigation_map.rs and navigation.rs: the macro view (238 constant, MapX/MapY/MapW/MapH from the layout); the detail view's 250 branch is in the same formula but no detail maps are read yet
- [x] `gmNavigationUI::SizeMap` · 004319b0 · legions 004322b0 — ac2-ui navigation_detail.rs (window, corners, tiles, set_lock, pan, switch); navigation.rs (SizeMap detail branch via tile_position); tests navigation_detail_tests.rs and ac2-client tests/navigation_detail_7160.rs
- [x] `gmNavigationUI::ReadMarkerInfo` · 00431bd0 · legions 004324d0 — ac2-ui navigation.rs read_marker_info: the hovered note (tooltip owner) gives NoteText its tooltip text each frame; the tooltip is ID_Navigation_Marker with message and position (ac2-client ui_sources marker_tooltip)

### LandDefs
- [x] `LandDefs::lcoord_to_gid` · 00431e20 · legions 004326b0
- [x] `LandDefs::gid_to_lcoord` · 00431e90 · legions 00432720

### Position
- [-] `Position::Position` · 00431f60 · legions 004327f0 — ctor/dtor/operator= of plain data; Rust struct construction and Drop replace it

### gmNavigationUI
- [x] `gmNavigationUI::Update` · 00431fa0 · legions 00432830 — ac2-world region/game_time.rs (GameClock::date_time: YEAR, DAY, YEARSPEC, SEASON, TIME ids), ac2-client ui_sources::game_date_time rendered for the frame, ac2-ui navigation.rs (DateText and TimeText in the macro view, CoordinatesText); tests game_time_real_tests.rs, navigation_tests, navigation_detail_7160.rs
- [x] `gmNavigationUI::PlaceMarker` · 00432090 · legions 00432920 — ac2-ui navigation_map.rs and navigation.rs: the macro view (238 constant, MapX/MapY/MapW/MapH from the layout); the detail view's 250 branch is in the same formula but no detail maps are read yet
- [x] `gmNavigationUI::Show` · 00432330 · legions 00432bc0 — ac2-ui navigation.rs show (visibility); update and the scroll buttons run on the next frame while shown

### SmartArray<CMapNoteRow>
- [-] `SmartArray<CMapNoteRow>::operator=` · 00432460 · legions 00432cf0 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### PSmartArray<CMapNoteRow>
- [-] `PSmartArray<CMapNoteRow>::StreamPack` · 00432510 · legions 00432da0 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### gmNavigationUI
- [x] `gmNavigationUI::PlaceMarkers` · 004325b0 · legions 00432e40 — ac2-ui navigation_map.rs and navigation.rs: the macro view (238 constant, MapX/MapY/MapW/MapH from the layout); the detail view's 250 branch is in the same formula but no detail maps are read yet
- [x] `gmNavigationUI::FlushMarkers` · 00432640 · legions 00432ed0 — ac2-ui navigation_map.rs and navigation.rs: the macro view (238 constant, MapX/MapY/MapW/MapH from the layout); the detail view's 250 branch is in the same formula but no detail maps are read yet
- [~] `gmNavigationUI::FlushAllMarkers` · 004326a0 · legions 00432f30 — ac2-ui navigation.rs flush_marker_type: the static type flush (SetupElementsForMapMode) deletes only its notes; one note list is kept (retail's per-type hash of lists is not modelled) and the all-types flush is only the reconcile against the frame
- [-] `gmNavigationUI::~gmNavigationUI` · 004327f0 · legions 00433080 — C++ destructors and scalar deleting destructors: the Rust drop and the tree own these elements
- [x] `gmNavigationUI::SetupElementsForMapMode` · 004328b0 · legions 00433140 — ac2-ui navigation.rs setup_elements_for_map_mode: flushes the Static_MarkerType notes first (flush_marker_type, the other types kept), map field and tiles ClearTooltip, the four overview buttons visible in the macro view, map and detail visibility, parent state, marker parent, scroll buttons; test navigation_tests a_flush_deletes_only_its_marker_type and the_overview_buttons_show_their_map_and_show_in_the_macro_view
- [x] `gmNavigationUI::InitMacroMap` · 004329c0 · legions 00433250 — ac2-world region/game_time.rs (GameClock::date_time: YEAR, DAY, YEARSPEC, SEASON, TIME ids), ac2-client ui_sources::game_date_time rendered for the frame, ac2-ui navigation.rs (DateText and TimeText in the macro view, CoordinatesText); tests game_time_real_tests.rs, navigation_tests, navigation_detail_7160.rs
- [x] `gmNavigationUI::RecvNotice_ClearMarkers` · 00432b30 — ac2-client client_radar.rs (add_map_note: type 3 never shown, a note within sqrt(6) cells of its type ignored, ten per type, same object replaces) and navigation.rs reconcile; Legions 7160 has no separate notice handler (admin rule)
- [x] `gmNavigationUI::AddMarker` · 00432b90 · legions 00433420 — ac2-client marker_tooltip renders ID_Navigation_Marker (message, map coords of the cell) into the note tooltip (ac2-ui navigation.rs reconcile_notes sets it); PlaceMarker via the notes
- [~] `gmNavigationUI::FlushMarkerByIID` · 00432e40 · legions 004336d0 — ac2-ui navigation.rs reconcile_notes: the frame list replaces the server notes, so a removed marker is gone next frame; the per-IID list removal is not separate
- [x] `gmNavigationUI::RecvNotice_AddMarker` · 00432ef0 — ac2-client client_radar.rs (add_map_note: type 3 never shown, a note within sqrt(6) cells of its type ignored, ten per type, same object replaces) and navigation.rs reconcile; Legions 7160 has no separate notice handler (admin rule)
- [x] `gmNavigationUI::RecvNotice_RemoveMarker` · 004330e0 — ac2-client client_radar.rs (add_map_note: type 3 never shown, a note within sqrt(6) cells of its type ignored, ten per type, same object replaces) and navigation.rs reconcile; Legions 7160 has no separate notice handler (admin rule)
- [~] `gmNavigationUI::AddMapNotes` · 00433100 · legions 00433990 — ac2-ui navigation_detail.rs tile_notes: the tiles' notes at their cells and icons, shown only in the detail view (refresh skips them in macro, as the static flush does). Tooltip text is still empty: the Legions AddMapNotes body is truncated in the decompile, and the admin body's tooltip template (the tooltip's StringInfo id at local_7c) is not determinable; the note's string (MapNoteRow.string_id) is loaded but not shown
- [x] `gmNavigationUI::InitDetailMapFromCellCoords` · 004335f0 · legions 00433e50 — ac2-ui navigation_detail.rs (window, corners, tiles, set_lock, pan, switch); navigation.rs (SizeMap detail branch via tile_position); tests navigation_detail_tests.rs and ac2-client tests/navigation_detail_7160.rs
- [x] `gmNavigationUI::ZoomToCharPos` · 00433810 · legions 00434070 — ac2-ui navigation_detail.rs (window, corners, tiles, set_lock, pan, switch); navigation.rs (SizeMap detail branch via tile_position); tests navigation_detail_tests.rs and ac2-client tests/navigation_detail_7160.rs
- [x] `gmNavigationUI::OnPlayerBlockChanged` · 00433830 · legions 00434090 — ac2-ui navigation_detail.rs (window, corners, tiles, set_lock, pan, switch); navigation.rs (SizeMap detail branch via tile_position); tests navigation_detail_tests.rs and ac2-client tests/navigation_detail_7160.rs
- [~] `gmNavigationUI::gmNavigationUI` · 00433880 · legions 004340e0 — ac2-ui navigation.rs: macro view only; the outdoor flag and the global loop's Update; detail-map init, lock and buttons not ported
- [~] `gmNavigationUI::ListenToMessage` · 00433cf0 · legions 00434570 — ac2-ui overview_map.rs map_for_button/message: the four zoom buttons (Osteth, Omishan, Arramora, LinvakMassif) call show_map with their map state; the close button hides. Client routes UiMessage to it (ac2-client hud_player_message.rs). Not: KnorrMap (its button id is not bound: Legions binds DAT_00a4b640 as both the coordinates text and the Knorr-style message), the help page (KeyStone::OpenHelp)
- [x] `gmNavigationUI::InitDetailMapFromLastClick` · 00434160 · legions 00434930 — ac2-ui navigation_detail.rs (window, corners, tiles, set_lock, pan, switch); navigation.rs (SizeMap detail branch via tile_position); tests navigation_detail_tests.rs and ac2-client tests/navigation_detail_7160.rs
- [x] `gmNavigationUI::UpdatePlayerPos` · 004341b0 · legions 00434980 — ac2-ui navigation_detail.rs (window, corners, tiles, set_lock, pan, switch); navigation.rs (SizeMap detail branch via tile_position); tests navigation_detail_tests.rs and ac2-client tests/navigation_detail_7160.rs
- [~] `gmNavigationUI::ListenToMessage` · 00434280 · legions 00434a50 — ac2-ui overview_map.rs map_for_button/message: the four zoom buttons (Osteth, Omishan, Arramora, LinvakMassif) call show_map with their map state; the close button hides. Client routes UiMessage to it (ac2-client hud_player_message.rs). Not: KnorrMap (its button id is not bound: Legions binds DAT_00a4b640 as both the coordinates text and the Knorr-style message), the help page (KeyStone::OpenHelp)

## `ac2/client/src/ui/gameplay/toolbar/gmshortcutui.cpp.c` (30/46)

### gmShortcutUI
- [x] `gmShortcutUI::Show` · 00435cd0 · legions 00436520 — ac2-ui shortcut.rs show: the bar's field shows or hides; ac2-client show_toolbar calls it from the toolbar's Show
- [x] `gmShortcutUI::CheckAttackButtons` · 00435ce0 · legions 00436530 — ac2-ui shortcut.rs
- [x] `gmShortcutUI::RecvNotice_UpdateAttackState` · 00435d30 — ac2-ui shortcut.rs
- [x] `gmShortcutUI::RequestAttackStateChange` · 00435d80 · legions 004365d0 — ac2-client Command::SetAttacking: ShouldAutoTarget's Select(0x41000024) notice, then the IFun on the VM (stand-in: OnlineSession::request_attack_state -> WM_Combat 0x16 with the selection / 0x14)

### (free functions)
- [x] `gmShortcutUI_DDCB` · 00435e10 · legions 00436660 — ac2-ui drag.rs DragCheck::ShortcutDrop on the bar's list: a drag carrying a shortcut type, over a slot below 100
- [x] `gmShortcutUI_DOCB` · 00435e70 — ac2-ui drag.rs DragCheck::ShortcutOut on the bar's list (UIElement::DragItem asks it of the owner): a slot leaves the bar only below index 100 and, with the bar locked (GetShortcutLocked, kept by the options page's setter and the profile at reload), only while shift is held; tested through the HUD (shortcut_drag_7160)

### SmartArray<TimeLeftInfo>
- [-] `SmartArray<TimeLeftInfo>::RemoveOrderedByIndex` · 00435ec0 · legions 00436710 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### gmShortcutUI
- [x] `gmShortcutUI::RemoveShortcut` · 00435fc0 · legions 00436810 — ac2-ui shortcut.rs
- [ ] `gmShortcutUI::RecvNotice_RemoveRecipe` · 00436060 — Legions has it: gmShortcutUI notice vtable slot 101 at 0x004368b0 (admin slot 100, shifted by one; same size and calls)
- [x] `gmShortcutUI::CheckRecipe` · 004360e0 · legions 00436930 — ac2-ui shortcut.rs
- [x] `gmShortcutUI::CheckTimeLeftArray` · 00436120 · legions 00436970 — ac2-ui shortcut_states.rs time_left_position: the entry for (skill, slot)
- [x] `gmShortcutUI::UpdateTimeLeft` · 00436180 · legions 004369d0 — ac2-ui shortcut_states.rs update_time_left: the meter fills by 1 - (end - now)/total clamped; the entry goes when full or its slot no longer holds the skill; ac2-client HudView::update runs it per frame
- [x] `gmShortcutUI::RecvNotice_ToggleSpecialAttack` · 00436330 — ac2-ui shortcut_states.rs toggle/request/start/end special attack: the maneuver's slots pending, active, toggled or stateless and dragable; ac2-client hud_player routes the notices

### ShortcutInfo
- [x] `ShortcutInfo::SetDataStr` · 004363e0 · legions 00436c30 — ac2-script glue::shortcut / native::shortcut

### gmShortcutUI
- [~] `gmShortcutUI::CheckItem` · 00436440 · legions 00436c90 — ac2-ui shortcut.rs: pending items (check_item) keep a pending state, cleared on reload only: no object-creation notice (RecvNotice_CreateObject, FIXMESetupItemElement icon and tooltip for an item not yet in the client); no drag/drop on the bar edge, rows, special attacks' drag state
- [x] `gmShortcutUI::FromElement` · 004364f0 · legions 00436d40 — ac2-ui drag.rs; shortcut.rs drop

### SmartArray<TimeLeftInfo>
- [-] `SmartArray<TimeLeftInfo>::AddToEnd` · 00436660 · legions 00436eb0 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### gmShortcutUI
- [x] `gmShortcutUI::CheckSkill` · 004367a0 · legions 00436ff0 — ac2-ui shortcut.rs check_skill_slot, check_all_items/skills, check_skill; ac2-client hud_player.rs ShortcutCheck from the UpdateInventory/UpdateContainer/UpdateSkill/UpdateResetTime notices; ac2-ui shortcut_checks_tests.rs
- [x] `gmShortcutUI::UpdateSetNumber` · 00436940 · legions 00437190 — ac2-ui shortcut.rs check_skill_slot, check_all_items/skills, check_skill; ac2-client hud_player.rs ShortcutCheck from the UpdateInventory/UpdateContainer/UpdateSkill/UpdateResetTime notices; ac2-ui shortcut_checks_tests.rs

### PSmartArray<ShortcutInfo>
- [-] `PSmartArray<ShortcutInfo>::StreamPack` · 00436a00 · legions 00437250 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### gmShortcutUI
- [x] `gmShortcutUI::CheckShortcut` · 00436bc0 · legions 00437410 — ac2-ui shortcut.rs
- [x] `gmShortcutUI::CheckAllItems` · 00436c20 · legions 00437470 — ac2-ui shortcut.rs
- [~] `gmShortcutUI::CheckItem` · 00436ca0 · legions 004374f0 — ac2-ui shortcut.rs: pending items (check_item) keep a pending state, cleared on reload only: no object-creation notice (RecvNotice_CreateObject, FIXMESetupItemElement icon and tooltip for an item not yet in the client); no drag/drop on the bar edge, rows, special attacks' drag state
- [x] `gmShortcutUI::CheckAllSkills` · 00436da0 · legions 004375f0 — ac2-ui shortcut.rs
- [x] `gmShortcutUI::CheckSkill` · 00436e20 · legions 00437670 — ac2-ui shortcut.rs check_skill_slot, check_all_items/skills, check_skill; ac2-client hud_player.rs ShortcutCheck from the UpdateInventory/UpdateContainer/UpdateSkill/UpdateResetTime notices; ac2-ui shortcut_checks_tests.rs
- [x] `gmShortcutUI::RecvNotice_UpdateInventory` · 00436ea0 — ac2-ui shortcut.rs check_skill_slot, check_all_items/skills, check_skill; ac2-client hud_player.rs ShortcutCheck from the UpdateInventory/UpdateContainer/UpdateSkill/UpdateResetTime notices; ac2-ui shortcut_checks_tests.rs
- [x] `gmShortcutUI::RecvNotice_UpdateContainer` · 00436f00 — ac2-ui shortcut.rs check_skill_slot, check_all_items/skills, check_skill; ac2-client hud_player.rs ShortcutCheck from the UpdateInventory/UpdateContainer/UpdateSkill/UpdateResetTime notices; ac2-ui shortcut_checks_tests.rs
- [x] `gmShortcutUI::RecvNotice_UpdateSkill` · 00436f40 — ac2-ui shortcut.rs check_skill_slot, check_all_items/skills, check_skill; ac2-client hud_player.rs ShortcutCheck from the UpdateInventory/UpdateContainer/UpdateSkill/UpdateResetTime notices; ac2-ui shortcut_checks_tests.rs
- [x] `gmShortcutUI::RecvNotice_UpdateResetTime` · 00436f60 — ac2-ui shortcut.rs check_skill_slot, check_all_items/skills, check_skill; ac2-client hud_player.rs ShortcutCheck from the UpdateInventory/UpdateContainer/UpdateSkill/UpdateResetTime notices; ac2-ui shortcut_checks_tests.rs
- [ ] `gmShortcutUI::RecvNotice_BeingDeleted` · 00436f70 — Legions has it: vtable slot 10 of gmShortcutUI at 0x004377c0 (vtable audit); was skip: no Legions 7160 counterpart (no legions_address in the index
- [~] `gmShortcutUI::FIXMESetupItemElement` · 00436fe0 · legions 00437830 — ac2-ui shortcut.rs make_element sets the slot icon from the shortcut info and check_item keeps the pending state. Not ported: the tooltip (the object's name with quantity, or ID_Item_TT_WaitingForServer while the weenie is unknown), SetIconDesc layers (first layer only) and the _pendingCreate append for a not-yet-created weenie (Legions 0x437830)
- [-] `gmShortcutUI::~gmShortcutUI` · 004371c0 · legions 00437a10 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own
- [-] `gmShortcutUI::RecvNotice_CreateObject` · 00437290 — notice handlers are registered per object by the session in Rust (no per-object handler table)
- [~] `gmShortcutUI::MakeElement` · 00437320 · legions 00437b70 — ac2-ui shortcut.rs: pending items (check_item) keep a pending state, cleared on reload only: no object-creation notice (RecvNotice_CreateObject, FIXMESetupItemElement icon and tooltip for an item not yet in the client); no drag/drop on the bar edge, rows, special attacks' drag state
- [x] `gmShortcutUI::SetShortcut` · 00437620 · legions 00437e70 — ac2-ui shortcut.rs set_shortcut: requested sets the slot (gmCWInterface::SetShortcut, WM_Player call 0x82120018 from the script) and the list item; tested through the HUD with skills, items and slot to slot drags (shortcut_drag_7160)
- [x] `gmShortcutUI::RecvNotice_RequestSpecialAttack` · 004376c0 — ac2-ui shortcut_states.rs toggle/request/start/end special attack: the maneuver's slots pending, active, toggled or stateless and dragable; ac2-client hud_player routes the notices
- [x] `gmShortcutUI::RecvNotice_StartSpecialAttack` · 00437770 — ac2-ui shortcut_states.rs toggle/request/start/end special attack: the maneuver's slots pending, active, toggled or stateless and dragable; ac2-client hud_player routes the notices
- [x] `gmShortcutUI::RecvNotice_EndSpecialAttack` · 00437820 — ac2-ui shortcut_states.rs toggle/request/start/end special attack: the maneuver's slots pending, active, toggled or stateless and dragable; ac2-client hud_player routes the notices
- [x] `gmShortcutUI::ListenToMessage` · 004379a0 · legions 004381f0 — ac2-ui shortcut.rs message: the double click on a skill, the use/examine presses and stop_drag_item (RemoveShortcut of the source slot, then SetShortcut of the slot under the pointer; a drop outside removes); tested through the HUD (shortcut_drag_7160)
- [x] `gmShortcutUI::UpdatePadding` · 00437d90 · legions 004385e0 — ac2-ui shortcut.rs
- [~] `gmShortcutUI::RefreshShortcutHeight` · 00437e50 · legions 004386a0 — ac2-ui shortcut.rs: pending items (check_item) keep a pending state, cleared on reload only: no object-creation notice (RecvNotice_CreateObject, FIXMESetupItemElement icon and tooltip for an item not yet in the client); no drag/drop on the bar edge, rows, special attacks' drag state
- [~] `gmShortcutUI::gmShortcutUI` · 00437fb0 · legions 00438800 — ac2-ui shortcut.rs: pending items (check_item) keep a pending state, cleared on reload only: no object-creation notice (RecvNotice_CreateObject, FIXMESetupItemElement icon and tooltip for an item not yet in the client); no drag/drop on the bar edge, rows, special attacks' drag state
- [~] `gmShortcutUI::ListenToMessage` · 00438270 · legions 00438ac0 — ac2-ui shortcut.rs: pending items (check_item) keep a pending state, cleared on reload only: no object-creation notice (RecvNotice_CreateObject, FIXMESetupItemElement icon and tooltip for an item not yet in the client); no drag/drop on the bar edge, rows, special attacks' drag state
- [~] `gmShortcutUI::Init` · 004386c0 · legions 00438f10 — ac2-ui shortcut.rs: pending items (check_item) keep a pending state, cleared on reload only: no object-creation notice (RecvNotice_CreateObject, FIXMESetupItemElement icon and tooltip for an item not yet in the client); no drag/drop on the bar edge, rows, special attacks' drag state
- [x] `gmShortcutUI::RecvNotice_ReloadShortcuts` · 004387f0 — ac2-ui shortcut.rs
- [~] `gmShortcutUI::RecvNotice_ReloadOptions` · 00438830 · legions 00439080 — ac2-ui shortcut.rs: pending items (check_item) keep a pending state, cleared on reload only: no object-creation notice (RecvNotice_CreateObject, FIXMESetupItemElement icon and tooltip for an item not yet in the client); no drag/drop on the bar edge, rows, special attacks' drag state

## `ac2/client/src/ui/gameplay/toolbar/gmtoolbarui.cpp.c` (5/16)

### PlayingSoundContext
- [-] `PlayingSoundContext::PlayingSoundContext` · 00421ec0 · legions 006b07e0 — Win32 / DirectInput / D3D plumbing (device poll, VK mapping, focus, hardware cursor, draw list, sound device) replaced by winit/wgpu/rodio

### gmToolbarUI
- [-] `gmToolbarUI::ShowKeyStone` · 00421ed0 · legions 00421d10 — KeyStone help and the plug-in manager are external windows (MSXML4); not opened, so the missing-XML4 message is not sent
- [-] `gmToolbarUI::ShowPlugin` · 00421f60 · legions 00421da0 — KeyStone help and the plug-in manager are external windows (MSXML4); not opened, so the missing-XML4 message is not sent
- [x] `gmToolbarUI::RecvNotice_ShowSaleReminder` · 00421fe0 — ac2-ui toolbar.rs sale_reminder: the consignments button in its state; the sale sound is not played
- [x] `gmToolbarUI::RecvNotice_UpdateMountState` · 00422080 · legions 00421ec0 — ac2-ui toolbar.rs mount_state: the dismount button in its dismount state; ac2-client hud_player routes UpdateMountState
- [~] `gmToolbarUI::Show` · 004220d0 · legions 00421f10 — ac2-ui toolbar.rs show_bar: the toolbar field shows and the bottom preclude is its precludeheight (0 hidden), applied at arrival (hud_view show_toolbar). Not: the keystone help state (IsHelpOpen remembered and reopened; ShowKeyStone toggles the help window, hud_help.rs toggle_keystone) and the shortcut bar Show is a separate entry
- [~] `gmToolbarUI::ManageChildren` · 00422160 · legions 00421fa0 — ac2-ui toolbar.rs: fields shown/hidden by side; ManageChildren tooltips ID_Toolbar_TT_<child> (Hide while shown, Show while hidden) via refresh_tooltips. Not: the child panel contents (each child's framework)
- [x] `gmToolbarUI::ChildKeyPress` · 00422230 · legions 00422070 — ac2-ui toolbar.rs
- [x] `gmToolbarUI::ListenToMessage` · 00422270 · legions 004220b0 — ConsignmentsButton release (Legions 0x00422140 element DAT_00a48d50) calls gmCWInterface::DisplaySaleReminders: ac2-ui toolbar.rs is_sale_reminder_press, routed in ac2-client hud_player_message.rs to Command::DisplaySaleReminders (IFun 0xB4013D); toolbar_tests the_consignments_button_release_is_the_sale_reminder_press
- [x] `gmToolbarUI::ListenToMessage` · 00422300 · legions 00422140 — ConsignmentsButton release (Legions 0x00422140 element DAT_00a48d50) calls gmCWInterface::DisplaySaleReminders: ac2-ui toolbar.rs is_sale_reminder_press, routed in ac2-client hud_player_message.rs to Command::DisplaySaleReminders (IFun 0xB4013D); toolbar_tests the_consignments_button_release_is_the_sale_reminder_press

### List<unsigned_long>
- [-] `List<unsigned_long>::operator=` · 00422360 · legions 0040d980 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### ListNode<ToolbarChildInfo>
- [-] `ListNode<ToolbarChildInfo>::ListNode<ToolbarChildInfo>` · 00422400 · legions 004221d0 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### gmToolbarUI
- [~] `gmToolbarUI::SetupChildren` · 00422550 · legions 00422320 — ac2-ui toolbar.rs: fields shown/hidden by side; ManageChildren tooltips ID_Toolbar_TT_<child> (Hide while shown, Show while hidden) via refresh_tooltips. Not: the child panel contents (each child's framework)
- [~] `gmToolbarUI::CleanupChildren` · 00422f00 · legions 00422cd0 — ac2-ui toolbar.rs cleanup_children: the child fields hide; the child frameworks are not made so none is deleted
- [~] `gmToolbarUI::gmToolbarUI` · 00422f80 · legions 00422d50 — ac2-ui hud.rs + ac2-gpu ui.rs + ac2-client hud_view.rs: HUD roots made and drawn; no framework behaviour beyond hiding the toolbar's dismount/consignment buttons
- [-] `gmToolbarUI::~gmToolbarUI` · 00423120 · legions 00422ef0 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own

## `ac2/client/src/ui/gameplay/toolbar/inventory/gm3ditemsui.cpp.c` (12/14)

### gm3DItemsUI
- [x] `gm3DItemsUI::FinishItemDrop` · 00440240 · legions 00440aa0 — ac2-ui items_3d.rs; client hud_items_3d.rs and view.rs (the target search is ClientView::pick at the pointer); tested in items_3d_tests.rs and through the HUD (world_drop_7160): FinishItemDrop is gmCWInterface::MoveItem(item, player, target, fromSlot, 0, quantity)
- [x] `gm3DItemsUI::StartFind3DTarget` · 004402c0 · legions 00440b20 — ac2-ui items_3d.rs; client hud_items_3d.rs and view.rs (the target search is ClientView::pick at the pointer); tested in items_3d_tests.rs and through the HUD (world_drop_7160): StartFind3DTarget asks the pick at the pointer (one search at a time)
- [~] `gm3DItemsUI::Start3DDrag` · 00440320 · legions 00440b80 — ac2-ui items_3d.rs; client hud_items_3d.rs and view.rs (the target search is ClientView::pick at the pointer); tested in items_3d_tests.rs and through the HUD (world_drop_7160): Start3DDrag puts the selection's image (first icon layer) on the field's ItemIcon, dragable, ShortcutType 0x40000002 and the item as ShortcutData, and starts the drag; the icon description's layers are not composed. A drag of a world item into the pack is not traced (HandleListBoxDrop for an item with no FromID)

### (free functions)
- [x] `gm3DItems_DDCB` · 00440490 — ac2-ui drag.rs DragCheck::WorldDrop on ItemSelectionField: the coin field, or an item from a container (FromID set); tested in drag_tests

### gm3DItemsUI
- [~] `gm3DItemsUI::gm3DItemsUI` · 00440520 · legions 00440d80 — ac2-ui items_3d.rs; client hud_items_3d.rs and view.rs (the target search is ClientView::pick at the pointer); tested in items_3d_tests.rs and through the HUD (world_drop_7160): DDCB on ItemSelectionField and its ItemIcon found; the global loop and message registrations are the HUD's routing, and the notice handler 0x45a is not registered
- [x] `gm3DItemsUI::~gm3DItemsUI` · 00440650 · legions 00440eb0 — ac2-ui items_3d.rs Items3dUi::contexts lists the two stack dialogs to close (CloseStackUI item, money); the panel lives for the whole HUD and the stack dialog is dropped with it, so no separate close is needed
- [x] `gm3DItemsUI::RecvNotice_StackUIDone` · 004406b0 · legions 00440f10 — ac2-ui items_3d.rs; client hud_items_3d.rs and view.rs (the target search is ClientView::pick at the pointer); tested in items_3d_tests.rs and through the HUD (world_drop_7160): the dialog's answer finishes the item drop or leaves the money bag for the target
- [x] `gm3DItemsUI::StartItemDrop` · 00440700 · legions 00440f60 — ac2-ui items_3d.rs; client hud_items_3d.rs and view.rs (the target search is ClientView::pick at the pointer); tested in items_3d_tests.rs and through the HUD (world_drop_7160): an unworn item whose FromID is the player starts the search
- [x] `gm3DItemsUI::StartMoneyDrop` · 004407f0 · legions 00441050 — ac2-ui items_3d.rs; client hud_items_3d.rs and view.rs (the target search is ClientView::pick at the pointer); tested in items_3d_tests.rs and through the HUD (world_drop_7160)
- [x] `gm3DItemsUI::FinishFind3DTarget` · 00440810 · legions 00441070 — ac2-ui items_3d.rs; client hud_items_3d.rs and view.rs (the target search is ClientView::pick at the pointer); tested in items_3d_tests.rs and through the HUD (world_drop_7160): the player itself is no target
- [x] `gm3DItemsUI::ListenToMessage` · 004408e0 · legions 00441140 — ac2-ui items_3d.rs; client hud_items_3d.rs and view.rs (the target search is ClientView::pick at the pointer); tested in items_3d_tests.rs and through the HUD (world_drop_7160): a drop on ItemSelectionField (the coin field: money, else StartItemDrop), a press nothing took (Start3DDrag)
- [x] `gm3DItemsUI::MidItemDrop` · 00440960 · legions 004411c0 — ac2-ui items_3d.rs; client hud_items_3d.rs and view.rs (the target search is ClientView::pick at the pointer); tested in items_3d_tests.rs and through the HUD (world_drop_7160): a stack of more than one asks how many first (gmStackUI), the default and the most min(quantity, max stack)
- [x] `gm3DItemsUI::MidMoneyDrop` · 00440a60 · legions 004412c0 — ac2-ui items_3d.rs; client hud_items_3d.rs and view.rs (the target search is ClientView::pick at the pointer); tested in items_3d_tests.rs and through the HUD (world_drop_7160): the money up to the coin stack's maximum
- [x] `gm3DItemsUI::ListenToMessage` · 00440b40 · legions 004413a0 — ac2-ui items_3d.rs; client hud_items_3d.rs and view.rs (the target search is ClientView::pick at the pointer); tested in items_3d_tests.rs and through the HUD (world_drop_7160): the pick answer arrives with the frame's frameworks (the global loop's)

## `ac2/client/src/ui/gameplay/toolbar/inventory/gmbackpackui.cpp.c` (5/7)

### gmBackpackUI
- [x] `gmBackpackUI::FinishItemDrop` · 004412f0 · legions 00441b50 — ac2-ui backpack.rs finish_item_drop: MoveItem inside the page's container unless the slot is unchanged; reached by a drop on a page tab (tab_drop) with the stack dialog for a stack
- [x] `gmBackpackUI::FinishMoneyBagDrop` · 00441390 · legions 00441bf0 — ac2-ui money_bag.rs / item_display.rs / backpack.rs; ac2-client ui_inventory.rs; ac2-script host/inventory_globals.rs (coin max) and the player's IFUN 0xB400D7 (bypass); Legions 7160 matches admin here (same IFUNs 0xA60001 and 0xB400D7)
- [x] `gmBackpackUI::RecvNotice_StackUIDone` · 00441410 — ac2-ui backpack.rs stack_ui_done: the money bag and item drop answers
- [x] `gmBackpackUI::ShowPack` · 00441480 · legions 00441d70 — ac2-ui item_display.rs / backpack.rs
- [~] `gmBackpackUI::ListenToMessage` · 004414e0 · legions 00441de0 — ac2-ui backpack.rs tab_drop: a worn item unequips to the page's end, the coin field asks how much, another item on a tab not shown moves to the page's end (stack dialog first); tested through the HUD (item_drag_7160). Not: icon descriptions' layers, GetContentsInfo from the VM
- [-] `gmBackpackUI::~gmBackpackUI` · 00441860 · legions 00442160 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own
- [x] `gmBackpackUI::gmBackpackUI` · 00441950 · legions 00442250 — ac2-ui item_display.rs / backpack.rs

## `ac2/client/src/ui/gameplay/toolbar/inventory/gminventoryui.cpp.c` (4/5)

### gmInventoryUI
- [x] `gmInventoryUI::Show` · 00434500 · legions 00434cd0 — toolbar child toggle and close (ac2-ui toolbar.rs; view_7160 and paper_doll_7160 show the Skill and Inventory panels); children made with the HUD (hud.rs). The help button's KeyStone::OpenHelp opens the help window (hud_help.rs; gmToolbarUI::ShowKeyStone toggles it)
- [x] `gmInventoryUI::ListenToMessage` · 00434530 · legions 00434d00 — toolbar child toggle and close (ac2-ui toolbar.rs; view_7160 and paper_doll_7160 show the Skill and Inventory panels); children made with the HUD (hud.rs). The help button's KeyStone::OpenHelp opens the help window (hud_help.rs; gmToolbarUI::ShowKeyStone toggles it)
- [x] `gmInventoryUI::SetupChildren` · 00434590 · legions 00434d60 — toolbar child toggle and close (ac2-ui toolbar.rs; view_7160 and paper_doll_7160 show the Skill and Inventory panels); children made with the HUD (hud.rs). The help button's KeyStone::OpenHelp opens the help window (hud_help.rs; gmToolbarUI::ShowKeyStone toggles it)

### gmStatusbarUI
- [~] `gmStatusbarUI::CleanupChildren` · 00434670 · legions 00434e40 — ac2-ui: the status bar children are fields of GameplayHud and drop with its tree; no per-child teardown call

### gmInventoryUI
- [x] `gmInventoryUI::gmInventoryUI` · 00434700 · legions 00434ed0 — toolbar child toggle and close (ac2-ui toolbar.rs; view_7160 and paper_doll_7160 show the Skill and Inventory panels); children made with the HUD (hud.rs). The help button's KeyStone::OpenHelp opens the help window (hud_help.rs; gmToolbarUI::ShowKeyStone toggles it)

## `ac2/client/src/ui/gameplay/toolbar/inventory/gmmoneybagui.cpp.c` (8/11)

### gmMoneyBagUI
- [x] `gmMoneyBagUI::RecvNotice_UpdateMoney` · 00440b70 — ac2-ui money_bag.rs / item_display.rs / backpack.rs; ac2-client ui_inventory.rs; ac2-script host/inventory_globals.rs (coin max) and the player's IFUN 0xB400D7 (bypass); Legions 7160 matches admin here (same IFUNs 0xA60001 and 0xB400D7)
- [x] `gmMoneyBagUI::CloseDialog` · 00440be0 · legions 00441440 — ac2-ui money_bag.rs / item_display.rs / backpack.rs; ac2-client ui_inventory.rs; ac2-script host/inventory_globals.rs (coin max) and the player's IFUN 0xB400D7 (bypass); Legions 7160 matches admin here (same IFUNs 0xA60001 and 0xB400D7)
- [x] `gmMoneyBagUI::IsConfirmationDialogAppropriate` · 00440c00 · legions 00441460 — ac2-ui money_bag.rs / item_display.rs / backpack.rs; ac2-client ui_inventory.rs; ac2-script host/inventory_globals.rs (coin max) and the player's IFUN 0xB400D7 (bypass); Legions 7160 matches admin here (same IFUNs 0xA60001 and 0xB400D7)
- [x] `gmMoneyBagUI::MakeConfirmationDialog` · 00440c30 · legions 00441490 — ac2-ui money_bag.rs / item_display.rs / backpack.rs; ac2-client ui_inventory.rs; ac2-script host/inventory_globals.rs (coin max) and the player's IFUN 0xB400D7 (bypass); Legions 7160 matches admin here (same IFUNs 0xA60001 and 0xB400D7)
- [x] `gmMoneyBagUI::DoDragTo` · 00440cd0 · legions 00441530 — ac2-ui money_bag.rs / item_display.rs / backpack.rs; ac2-client ui_inventory.rs; ac2-script host/inventory_globals.rs (coin max) and the player's IFUN 0xB400D7 (bypass); Legions 7160 matches admin here (same IFUNs 0xA60001 and 0xB400D7)

### (free functions)
- [~] `gmMoneyBagUI_DDCB` · 00440d50 — money bag: the coin/melt paths ported; not modelled: DDCB's IsOnInventory test (gmMoneyBagUI_DDCB), the bag's ctor listeners and the not_dragable branch (m_curAmount is only 0 in retail), the backpack page's item drop (FinishItemDrop) and its ListenToMessage item branch

### gmMoneyBagUI
- [~] `gmMoneyBagUI::gmMoneyBagUI` · 00440df0 · legions 00441650 — money bag: the coin/melt paths ported; not modelled: DDCB's IsOnInventory test (gmMoneyBagUI_DDCB), the bag's ctor listeners and the not_dragable branch (m_curAmount is only 0 in retail), the backpack page's item drop (FinishItemDrop) and its ListenToMessage item branch
- [x] `gmMoneyBagUI::RecvNotice_CloseConfirmationDialog` · 00440f70 — ac2-ui money_bag.rs / item_display.rs / backpack.rs; ac2-client ui_inventory.rs; ac2-script host/inventory_globals.rs (coin max) and the player's IFUN 0xB400D7 (bypass); Legions 7160 matches admin here (same IFUNs 0xA60001 and 0xB400D7)
- [x] `gmMoneyBagUI::RecvNotice_StackUIDone` · 00440f90 — ac2-ui money_bag.rs / item_display.rs / backpack.rs; ac2-client ui_inventory.rs; ac2-script host/inventory_globals.rs (coin max) and the player's IFUN 0xB400D7 (bypass); Legions 7160 matches admin here (same IFUNs 0xA60001 and 0xB400D7)
- [x] `gmMoneyBagUI::HandleMoneyBagDrop` · 00440fe0 · legions 00441840 — ac2-ui money_bag.rs / item_display.rs / backpack.rs; ac2-client ui_inventory.rs; ac2-script host/inventory_globals.rs (coin max) and the player's IFUN 0xB400D7 (bypass); Legions 7160 matches admin here (same IFUNs 0xA60001 and 0xB400D7)
- [~] `gmMoneyBagUI::ListenToMessage` · 00441280 · legions 00441ae0 — money bag: the coin/melt paths ported; not modelled: DDCB's IsOnInventory test (gmMoneyBagUI_DDCB), the bag's ctor listeners and the not_dragable branch (m_curAmount is only 0 in retail), the backpack page's item drop (FinishItemDrop) and its ListenToMessage item branch

## `ac2/client/src/ui/gameplay/toolbar/inventory/gmpaperdollui.cpp.c` (22/31)

### gmMiniFellowUI
- [x] `gmMiniFellowUI::Show` · 00441b40 · legions 00420260 — ac2-ui mini_fellow.rs MiniFellowUi::show: the shown state, the panel follows it (tested: mini_fellow_tests)

### gmPaperDollUI
- [~] `gmPaperDollUI::RecvNotice_OpenIngredientPanel` · 00441b70 · legions 00442490 — hud_ingredient.rs OpenIngredientPanel path into ac2-ui ingredient.rs
- [~] `gmPaperDollUI::UpdatePaperDollPosition` · 00441ba0 · legions 0041af10 — Legions 7160 (followed): race 1, 2, 8, 16 x0.9, race 4 x0.75; z = -0.875 x height; camera scale 0.25 x height (admin: -0.5, full height). The physobj call before it (vtbl 0x2F4 (1, 0)) is not modelled
- [x] `gmPaperDollUI::MeshTypeToInvLoc` · 00441d20 · legions 00442670 — step 3: the doll's pick, drag and drops (see paper_doll.rs, paper_doll_scene.rs)
- [x] `gmPaperDollUI::HandlePaperDollClick` · 00441db0 · legions 00442700 — step 3: the doll's pick, drag and drops (see paper_doll.rs, paper_doll_scene.rs)

### (free functions)
- [x] `gmPaperDollUI_DDCB` · 00441e50 — ac2-ui drag.rs DragCheck::ItemDrop on each paper doll slot (InitSlots): only an item

### gmPaperDollUI
- [ ] `gmPaperDollUI::RecvNotice_UpdateMountState` · 00441e90 — Legions has it: vtable slot 45 of gmPaperDollUI at 0x004427e0 (vtable audit); was skip: admin-only mount flag (sets the paper doll token count); no 
- [x] `gmPaperDollUI::SetDurabilityTooltip` · 00441ea0 · legions 004427f0 — step 4: the durability meter (1 - current/max, its tooltip) shows for an item that has durability, clears on remove and on an update without it
- [x] `gmPaperDollUI::InitPaperDoll` · 00441f10 · legions 00442860 — step 3: the doll's pick, drag and drops (see paper_doll.rs, paper_doll_scene.rs)
- [~] `gmPaperDollUI::UpdatePaperDoll` · 00441fd0 · legions 00442920 — the avatar follows the frame appearance (remade on change); Legions also calls the physobj with mode 0x38d1b717 (vtbl 0x110) and 0xe0000000/0x3f1a36e2 (vtbl 0x114), and the portrait placement (gmShopConsignmentUI::UpdatePortraitPosition): not modelled
- [x] `gmPaperDollUI::UpdateDurabilityMeter` · 00442030 · legions 00442980 — step 4: the durability meter (1 - current/max, its tooltip) shows for an item that has durability, clears on remove and on an update without it
- [x] `gmPaperDollUI::RecvNotice_LocalVisualDescUpdate` · 004420e0 — step 3: the doll's pick, drag and drops (see paper_doll.rs, paper_doll_scene.rs)
- [x] `gmPaperDollUI::TokenToInvLoc` · 00442100 · legions 00442a50 — step 2: slot lookup by element (the slots are the containers the layout names); DDCB is is_item_copy
- [x] `gmPaperDollUI::HandlePaperDollDrop` · 00442180 · legions 00442ad0 — ac2-ui paper_doll.rs drop: an item of the player's containers equips at the slot's (or the preferred) location; one from another container only moves into the player's pack and returns (admin and Legions alike, the earlier MoveAndEquip was wrong); tested through the HUD (item_drag_7160)
- [x] `gmPaperDollUI::InqItemInvLoc` · 00442300 · legions 00442c50 — ac2-ui paper_doll.rs inq_item_inv_loc: where an item is worn; item_changed (RecvNotice_ItemUpdate) uses it
- [x] `gmPaperDollUI::FinishPaperDollClick` · 00442380 · legions 00442cd0 — step 3: the doll's pick, drag and drops (see paper_doll.rs, paper_doll_scene.rs)
- [x] `gmPaperDollUI::HandlePaperDollDrag` · 004424b0 · legions 00442e00 — step 3: the doll's pick, drag and drops (see paper_doll.rs, paper_doll_scene.rs)
- [-] `gmPaperDollUI::~gmPaperDollUI` · 00442580 · legions 00442ed0 — C++ destructors: the Rust state drops
- [x] `gmPaperDollUI::ListenToMessage` · 00442870 · legions 004431c0 — step 3: the doll's pick, drag and drops (see paper_doll.rs, paper_doll_scene.rs)
- [x] `gmPaperDollUI::ListenToMessage` · 00442890 · legions 004431e0 — step 3: the doll's pick, drag and drops (see paper_doll.rs, paper_doll_scene.rs)
- [ ] `gmPaperDollUI::RecvNotice_CreateObject` · 00442900 — Legions has it: vtable slot 11 of gmPaperDollUI at 0x00443250 (vtable audit); was skip: no Legions 7160 counterpart; the durability meter refreshes 
- [x] `gmPaperDollUI::RecvNotice_ItemUpdate` · 004429e0 — step 4: the durability meter (1 - current/max, its tooltip) shows for an item that has durability, clears on remove and on an update without it
- [x] `gmPaperDollUI::AddItem` · 00442af0 · legions 00443440 — step 4: the durability meter (1 - current/max, its tooltip) shows for an item that has durability, clears on remove and on an update without it
- [x] `gmPaperDollUI::RemoveItem` · 00442c70 · legions 004435c0 — step 4: the durability meter (1 - current/max, its tooltip) shows for an item that has durability, clears on remove and on an update without it
- [x] `gmPaperDollUI::BeginRequest` · 00442dc0 · legions 00443710 — step 1: request begun or failed marks the worn item pending
- [x] `gmPaperDollUI::RequestFailed` · 00442e10 · legions 00443760 — step 1: request begun or failed marks the worn item pending
- [x] `gmPaperDollUI::InitSlots` · 00442e60 · legions 004437b0 — step 4: the durability meter (1 - current/max, its tooltip) shows for an item that has durability, clears on remove and on an update without it

### PHashTable<unsigned_long,InstanceID>
- [~] `PHashTable<unsigned_long,InstanceID>::StreamPack` · 004431d0 — unpack only, in ac2-net protocol::qualities / visual (Legions CBaseQualities::UnPack instruction-identical)

### gmPaperDollUI
- [~] `gmPaperDollUI::GetInventory` · 00443400 · legions 00443d50 — frame's equipment list; a list the worn items are not in takes them off (double answers an unequip with 0x40000001 alone; retail only adds)
- [~] `gmPaperDollUI::gmPaperDollUI` · 004434d0 · legions 00443e20 — viewport root, slots, the avatar and the worn list; notices come from the frame, not registered handlers
- [x] `gmPaperDollUI::RecvNotice_UpdateInventory` · 00443670 — step 1: request begun or failed marks the worn item pending

## `ac2/client/src/ui/gameplay/toolbar/option/gmassistanceui.cpp.c` (5/6)

### gmAssistanceUI
- [x] `gmAssistanceUI::Reset` · 00443720 · legions 00444070 — ac2-ui assistance.rs: Reset, the page's ctor, the message box close (close_message via assistance_outcomes), and the menu/send/AssistanceButton messages; assistance_tests.rs
- [x] `gmAssistanceUI::RecvNotice_CloseMessageDialog` · 004437c0 — ac2-ui assistance.rs: Reset, the page's ctor, the message box close (close_message via assistance_outcomes), and the menu/send/AssistanceButton messages; assistance_tests.rs
- [x] `gmAssistanceUI::InitCallTypeMenu` · 004437e0 · legions 00444130 — ac2-ui assistance.rs CALL_TYPES: InitCallTypeMenu's nine InsertTextItem calls in Legions 0x00444130's order, each with its GUNCallType (traced from the disassembly; each string global named through its static initializer)
- [x] `gmAssistanceUI::gmAssistanceUI` · 00443990 · legions 004442e0 — ac2-ui assistance.rs: Reset, the page's ctor, the message box close (close_message via assistance_outcomes), and the menu/send/AssistanceButton messages; assistance_tests.rs
- [~] `gmAssistanceUI::HandleSendButton` · 00443b00 · legions 00444450 — ac2-ui assistance.rs: nine GUN call types with GUNCallType items and the checks/box order; world name empty (client does not keep it), the eight non-default types' order is inferred, and the GUN answer is not wired (send_answered, queued = reset)
- [x] `gmAssistanceUI::ListenToMessage` · 00443cc0 · legions 00444610 — ac2-ui assistance.rs: Reset, the page's ctor, the message box close (close_message via assistance_outcomes), and the menu/send/AssistanceButton messages; assistance_tests.rs

## `ac2/client/src/ui/gameplay/toolbar/option/gmcharacterui.cpp.c` (21/22)

### gmCharacterUI
- [x] `gmCharacterUI::ConfirmRecall` · 00444e00 · legions 00445940 — character_options.rs (the options' Setup tree, the setters' values) and character.rs: the page's refresh, apply, recall, default and confirmations; ported, tested on the real layout (character_tests.rs)
- [x] `gmCharacterUI::ConfirmDefault` · 00444e60 · legions 004459a0 — character_options.rs (the options' Setup tree, the setters' values) and character.rs: the page's refresh, apply, recall, default and confirmations; ported, tested on the real layout (character_tests.rs)
- [x] `gmCharacterUI::ClearConfirmations` · 00444ec0 · legions 00445a00 — character_options.rs (the options' Setup tree, the setters' values) and character.rs: the page's refresh, apply, recall, default and confirmations; ported, tested on the real layout (character_tests.rs)
- [x] `gmCharacterUI::RefreshFlag` · 00444f20 · legions 00445a60 — character_options.rs (the options' Setup tree, the setters' values) and character.rs: the page's refresh, apply, recall, default and confirmations; ported, tested on the real layout (character_tests.rs)
- [x] `gmCharacterUI::ApplyFlag` · 004451b0 · legions 00445cf0 — character_options.rs (the options' Setup tree, the setters' values) and character.rs: the page's refresh, apply, recall, default and confirmations; ported, tested on the real layout (character_tests.rs)
- [x] `gmCharacterUI::ApplyValue` · 00445450 · legions 00445f90 — character_options.rs (the options' Setup tree, the setters' values) and character.rs: the page's refresh, apply, recall, default and confirmations; ported, tested on the real layout (character_tests.rs)
- [x] `gmCharacterUI::ApplyChoice` · 004454a0 · legions 00445fe0 — character_options.rs (the options' Setup tree, the setters' values) and character.rs: the page's refresh, apply, recall, default and confirmations; ported, tested on the real layout (character_tests.rs)
- [-] `gmCharacterUI::~gmCharacterUI` · 004454f0 · legions 00446030 — C++ destructor and scalar deleter: the page's widgets are owned by the tree and dropped with it
- [x] `gmCharacterUI::Refresh` · 004455d0 · legions 00446110 — character_options.rs (the options' Setup tree, the setters' values) and character.rs: the page's refresh, apply, recall, default and confirmations; ported, tested on the real layout (character_tests.rs)
- [x] `gmCharacterUI::Changed` · 004456d0 · legions 00446210 — character_options.rs (the options' Setup tree, the setters' values) and character.rs: the page's refresh, apply, recall, default and confirmations; ported, tested on the real layout (character_tests.rs)
- [x] `gmCharacterUI::RecvNotice_ReloadOptions` · 00445710 — character_options.rs (the options' Setup tree, the setters' values) and character.rs: the page's refresh, apply, recall, default and confirmations; ported, tested on the real layout (character_tests.rs)
- [x] `gmCharacterUI::RecvNotice_SetShortcutHeight` · 00445720 — character_options.rs (the options' Setup tree, the setters' values) and character.rs: the page's refresh, apply, recall, default and confirmations; ported, tested on the real layout (character_tests.rs)
- [x] `gmCharacterUI::BuildList` · 004457b0 · legions 004462f0 — character_options.rs (the options' Setup tree, the setters' values) and character.rs: the page's refresh, apply, recall, default and confirmations; ported, tested on the real layout (character_tests.rs)
- [x] `gmCharacterUI::UpdateButtons` · 00445800 · legions 00446340 — character_options.rs (the options' Setup tree, the setters' values) and character.rs: the page's refresh, apply, recall, default and confirmations; ported, tested on the real layout (character_tests.rs)
- [x] `gmCharacterUI::Apply` · 004458f0 · legions 00446430 — character_options.rs (the options' Setup tree, the setters' values) and character.rs: the page's refresh, apply, recall, default and confirmations; ported, tested on the real layout (character_tests.rs)
- [x] `gmCharacterUI::Recall` · 004459e0 · legions 00446520 — character_options.rs (the options' Setup tree, the setters' values) and character.rs: the page's refresh, apply, recall, default and confirmations; ported, tested on the real layout (character_tests.rs)
- [x] `gmCharacterUI::Default` · 00445a10 · legions 00446550 — character_options.rs (the options' Setup tree, the setters' values) and character.rs: the page's refresh, apply, recall, default and confirmations; ported, tested on the real layout (character_tests.rs)
- [x] `gmCharacterUI::RecvNotice_CloseConfirmationDialog` · 00445a50 — character_options.rs (the options' Setup tree, the setters' values) and character.rs: the page's refresh, apply, recall, default and confirmations; ported, tested on the real layout (character_tests.rs)
- [x] `gmCharacterUI::ListenToMessage` · 00445ac0 · legions 00446600 — character_options.rs (the options' Setup tree, the setters' values) and character.rs: the page's refresh, apply, recall, default and confirmations; ported, tested on the real layout (character_tests.rs)
- [x] `gmCharacterUI::AddOption` · 00445c20 · legions 00446760 — character_options.rs (the options' Setup tree, the setters' values) and character.rs: the page's refresh, apply, recall, default and confirmations; ported, tested on the real layout (character_tests.rs)
- [x] `gmCharacterUI::Setup` · 00445c80 · legions 004467c0 — character_options.rs (the options' Setup tree, the setters' values) and character.rs: the page's refresh, apply, recall, default and confirmations; ported, tested on the real layout (character_tests.rs)
- [x] `gmCharacterUI::gmCharacterUI` · 00446610 · legions 00447150 — character_options.rs (the options' Setup tree, the setters' values) and character.rs: the page's refresh, apply, recall, default and confirmations; ported, tested on the real layout (character_tests.rs)

## `ac2/client/src/ui/gameplay/toolbar/option/gmchatoptionsui.cpp.c` (17/18)

### gmChatOptionsUI
- [x] `gmChatOptionsUI::ConfirmApply` · 00443d20 · legions 00444670 — chat_options.rs confirm_apply / confirm_recall / confirm_default ask their confirmation (ID_Option_ApplyText, RecallText, DefaultText); answers via dialog_outcomes; chat_options_tests.rs on the real layout
- [x] `gmChatOptionsUI::ConfirmRecall` · 00443d80 · legions 004446d0 — chat_options.rs confirm_apply / confirm_recall / confirm_default ask their confirmation (ID_Option_ApplyText, RecallText, DefaultText); answers via dialog_outcomes; chat_options_tests.rs on the real layout
- [x] `gmChatOptionsUI::ConfirmDefault` · 00443de0 · legions 00444730 — chat_options.rs confirm_apply / confirm_recall / confirm_default ask their confirmation (ID_Option_ApplyText, RecallText, DefaultText); answers via dialog_outcomes; chat_options_tests.rs on the real layout
- [x] `gmChatOptionsUI::CreateChatFontSizeChoice` · 00443e40 · legions 00444790 — ac2-ui character_options.rs (fill_choices, select_choice, selected_choice, sync_option) and character.rs / chat_options.rs drop-downs: UIElement_Menu items keyed by choice number (tested on the real layout in character_tests.rs, chat_options_tests.rs)
- [x] `gmChatOptionsUI::CreateChatFontColorChoice` · 00443ee0 · legions 00444830 — ac2-ui character_options.rs (fill_choices, select_choice, selected_choice, sync_option) and character.rs / chat_options.rs drop-downs: UIElement_Menu items keyed by choice number (tested on the real layout in character_tests.rs, chat_options_tests.rs)
- [x] `gmChatOptionsUI::PopulateExpander` · 00443f60 — chat_options_setup.rs specs: Legions 7160's order (font size, colour, the four window flags, popup; the entry line has the first two; the window section has each window's text type). Admin 7148 has no window flags; Legions followed
- [-] `gmChatOptionsUI::~gmChatOptionsUI` · 00444100 · legions 00444b40 — C++ destructor and scalar deleter: the page's widgets are owned by the tree and dropped with it
- [x] `gmChatOptionsUI::Changed` · 004442a0 · legions 00444ce0 — chat_options.rs changed / update_buttons: Apply and Recall ghost while nothing differs; Default never ghosts (retail)
- [x] `gmChatOptionsUI::UpdateButtons` · 00444340 · legions 00444d80 — chat_options.rs changed / update_buttons: Apply and Recall ghost while nothing differs; Default never ghosts (retail)
- [x] `gmChatOptionsUI::Recall` · 00444450 · legions 00444e90 — chat_options.rs recall: every option back to its saved value, no requests (chat_options_tests.rs, after the confirmation)
- [x] `gmChatOptionsUI::ListenToMessage` · 004444f0 · legions 00444f30 — chat_options.rs message: the ChatTab opened and closed, checkbox highlights, expander headers, and the buttons' confirmations while the tab is open (routed from hud_player.rs)
- [x] `gmChatOptionsUI::Setup` · 00444640 · legions 00445080 — chat_options.rs setup: one expander per visible row of chat_options_table::ROWS (ID_Option_ChatAdminMessages hidden), in table order, from Legions' initialiser (0x70CBD0, 0x70CE80)
- [x] `gmChatOptionsUI::Apply` · 00444770 · legions 004451b0 — chat_options.rs apply: each option's setter in table order, each window's filter (the page's bits over the kept 0x84 of the last read), the reload notice, the upload. Retail sends every option, not only the changed ones
- [x] `gmChatOptionsUI::Default` · 004449b0 · legions 00445470 — chat_options.rs default: every option to its default, then apply (retail sends every setter; the font size default is the size as the choice index, the first choice, as retail's SetDefault reads it)
- [x] `gmChatOptionsUI::RecvNotice_CloseConfirmationDialog` · 00444a50 — chat_options.rs recv_close_confirmation_dialog: a yes runs the question's action (apply, recall or default); the open questions close. Legions 7160 has no matched function of this name; the admin flow is followed
- [x] `gmChatOptionsUI::Refresh` · 00444af0 — chat_options.rs refresh: each control reads ChatSettings (font size by its size, colour, window flags, popup, window text types) and is saved; run at login and on CM_Character::SendNotice_ReloadOptions (hud_player.rs). Legions' Refresh is FUN_004455b0 (unnamed)
- [x] `gmChatOptionsUI::RecvNotice_ReloadOptions` · 00444cb0 — hud_player.rs refreshes the chat page on CM_Character::SendNotice_ReloadOptions (the page's refresh); Legions 7160 has no matched handler of this name
- [x] `gmChatOptionsUI::gmChatOptionsUI` · 00444cc0 · legions 00445800 — ChatOptionsUi::create over the ChatPage (hud.rs): the expanders, the refresh at login (hud_player.rs). Legions registers the same notices; the tooltips are not set

## `ac2/client/src/ui/gameplay/toolbar/option/gmclientui.cpp.c` (11/18)

### gmClientUI
- [x] `gmClientUI::ConfirmRecall` · 00446710 · legions 00447250 — ac2-ui client_options_page.rs (ClientOptionsUi: tab, widgets, Apply/Recall/Default presses, confirmations); ac2-client hud_options.rs run (options_page_answer, Recall, Default); tests client_page_7160.rs default_asks_first_and_a_yes_resets_the_page
- [x] `gmClientUI::ConfirmDefault` · 00446770 · legions 004472b0 — ac2-ui client_options_page.rs (ClientOptionsUi: tab, widgets, Apply/Recall/Default presses, confirmations); ac2-client hud_options.rs run (options_page_answer, Recall, Default); tests client_page_7160.rs default_asks_first_and_a_yes_resets_the_page
- [x] `gmClientUI::ClearConfirmations` · 004467d0 · legions 00447310 — ac2-ui client_page.rs ClientPage::clear_confirmations; client_page tests
- [x] `gmClientUI::VerifyRenderOptions` · 00446830 · legions 00447370 — ac2-ui client_page.rs verify_render_options: a timed dialog (type 0x40000005) once while open, text ID_Option_RenderText (DAT_00a4f870 = Tokenize string of the admin initializer). The DVD UI string table has no such entry (client_page_7160 test), so the port asks its own question
- [ ] `gmClientUI::MonitorChange` · 00446890 — Legions has it: gmClientUI IPreferenceMonitor vtable slot 1 at 0x004473d0 (vtable audit)
- [x] `gmClientUI::Changed` · 004468d0 · legions 00447410 — ac2-ui client_options_page.rs update_buttons (Apply and Recall ghost while the bank holds no change, over ClientPage::buttons); ac2-client client_options.rs options_changed
- [x] `gmClientUI::LookupExpander` · 00446910 · legions 00447450 — ac2-ui client_page.rs ClientPage::lookup_expander over SECTION_ORDER; client_page tests
- [x] `gmClientUI::Dismantle` · 004469d0 · legions 00447510 — ac2-ui client_page.rs ClientPage::dismantle (expanders gone, questions closed); no monitors to detach in the port
- [x] `gmClientUI::UpdateButtons` · 00446a70 · legions 004475b0 — ac2-ui client_options_page.rs update_buttons (Apply and Recall ghost while the bank holds no change, over ClientPage::buttons); ac2-client client_options.rs options_changed
- [~] `gmClientUI::~gmClientUI` · 00446b40 · legions 00447680 — ac2-ui client_options_page.rs: an expander per section in Construct order, rows from the bank's preferences labelled ID_<section>_<key>; section headers are the section names as text (their string IDs are not identified); the FX and Misc sections register no preferences yet
- [~] `gmClientUI::Apply` · 00446bf0 · legions 00447730 — ac2-client hud_options.rs run: options_apply / options_default, then live sound (ac2-audio prefs), camera (CameraManager::apply_prefs), scene (view set_scene_options) and the window for a verified FullScreen; D3D-only options (texture, anisotropic and trilinear filters, shadows, water) are stored, not applied
- [x] `gmClientUI::Recall` · 00446ce0 · legions 00447820 — ac2-ui client_options_page.rs (ClientOptionsUi: tab, widgets, Apply/Recall/Default presses, confirmations); ac2-client hud_options.rs run (options_page_answer, Recall, Default); tests client_page_7160.rs default_asks_first_and_a_yes_resets_the_page
- [~] `gmClientUI::Default` · 00446d10 · legions 00447850 — ac2-client hud_options.rs run: options_apply / options_default, then live sound (ac2-audio prefs), camera (CameraManager::apply_prefs), scene (view set_scene_options) and the window for a verified FullScreen; D3D-only options (texture, anisotropic and trilinear filters, shadows, water) are stored, not applied
- [ ] `gmClientUI::MonitorReload` · 00446d50 — Legions has it: gmClientUI IPreferenceMonitor vtable slot 0 at 0x00447890 (vtable audit)
- [x] `gmClientUI::RecvNotice_CloseConfirmationDialog` · 00446e80 — ac2-ui client_options_page.rs (ClientOptionsUi: tab, widgets, Apply/Recall/Default presses, confirmations); ac2-client hud_options.rs run (options_page_answer, Recall, Default); tests client_page_7160.rs default_asks_first_and_a_yes_resets_the_page
- [x] `gmClientUI::ListenToMessage` · 00446f40 · legions 00447a80 — ac2-ui client_options_page.rs (ClientOptionsUi: tab, widgets, Apply/Recall/Default presses, confirmations); ac2-client hud_options.rs run (options_page_answer, Recall, Default); tests client_page_7160.rs default_asks_first_and_a_yes_resets_the_page
- [~] `gmClientUI::Construct` · 004470c0 · legions 00447c00 — ac2-ui client_options_page.rs: an expander per section in Construct order, rows from the bank's preferences labelled ID_<section>_<key>; section headers are the section names as text (their string IDs are not identified); the FX and Misc sections register no preferences yet
- [~] `gmClientUI::gmClientUI` · 00447450 · legions 00447f90 — ac2-ui client_options_page.rs: an expander per section in Construct order, rows from the bank's preferences labelled ID_<section>_<key>; section headers are the section names as text (their string IDs are not identified); the FX and Misc sections register no preferences yet

## `ac2/client/src/ui/gameplay/toolbar/option/gmoptionui.cpp.c` (10/11)

### gmOptionUI
- [x] `gmOptionUI::gmOptionUI` · 004347e0 · legions 00434fb0 — ac2-ui option_panel.rs OptionPanel::create: the field and the three buttons, hidden; the pages are the HUD's
- [x] `gmOptionUI::UpdateButtons` · 00434a20 · legions 004351f0 — ac2-ui option_panel.rs update_buttons: Leave, Quit and Assistance ghosted while a question is up
- [x] `gmOptionUI::MakeConfirmDialog` · 00434aa0 · legions 00435270 — ac2-ui option_panel.rs: ID_Option_ConfirmText confirmation; tests there
- [x] `gmOptionUI::DoConfirm` · 00434af0 · legions 004352c0 — ac2-ui option_panel.rs OptionAction::Confirm, run by hud_tick.rs: yes applies the character, chat option and client pages, no recalls them
- [x] `gmOptionUI::MakeQuitDialog` · 00434b40 · legions 00435310 — ac2-ui option_panel.rs: ID_Option_QuitText on the QuitButton; yes closes the client after UploadGameOptions (HudRequest::Quit, ac2-client main.rs)
- [x] `gmOptionUI::MakeLogoutDialog` · 00434ba0 · legions 00435370 — ac2-ui option_panel.rs: ID_Option_LogoutText on the LeaveButton; yes is EndCharacterSession (GameplayDialogs::logout_key)
- [x] `gmOptionUI::CloseDialog` · 00434c00 · legions 004353d0 — ac2-ui option_panel.rs (CommandTab messages and Show close the questions)
- [-] `gmOptionUI::~gmOptionUI` · 00434c30 · legions 00435400 — C++ destructor: the page's widgets and the OptionsWindow model drop with the owner
- [x] `gmOptionUI::Show` · 00434d10 · legions 004354e0 — ac2-ui option_panel.rs closing/close, run by hud_tick.rs: hiding the panel with changes asks the confirmation
- [x] `gmOptionUI::ListenToMessage` · 00434dd0 · legions 004355a0 — ac2-ui option_panel.rs message: LeaveButton, QuitButton, AssistanceButton (opens AssistanceTab), CommandTab, DropDownMenu open_up/open_down; close, help and option buttons are the toolbar's
- [x] `gmOptionUI::RecvNotice_CloseConfirmationDialog` · 00434f70 — ac2-ui option_panel.rs dialog_outcomes (Legions 0x00435740)

## `ac2/client/src/ui/gameplay/toolbar/quest/gmjournalquestsui.cpp.c` (12/14)

### gmJournalQuestsUI
- [x] `gmJournalQuestsUI::RemoveQuestElement` · 0043e3a0 · legions 0043ec10 — ac2-ui journal_quests.rs drop_row
- [x] `gmJournalQuestsUI::DoQuestTime` · 0043e430 · legions 0043eca0 — ac2-ui journal_quests.rs Rows::do_quest_time (countdown in journal_time.rs)

### gmJournalQuestsUI::QuestDescriptor
- [x] `gmJournalQuestsUI::QuestDescriptor::QuestDescriptor` · 0043e520 · legions 0043ed90 — ac2-ui journal_quests.rs QuestDescriptor (Clone, new)

### UIElement_ListBox
- [x] `UIElement_ListBox::GetRow` · 0043e580 · legions 0043edf0 — ac2-ui list_box.rs list_what_num (the row index the extra text goes after)

### gmJournalQuestsUI
- [x] `gmJournalQuestsUI::AddQuestElement` · 0043e5c0 · legions 0043ee30 — ac2-ui journal_quests.rs Rows::add
- [x] `gmJournalQuestsUI::UpdateQuestElement` · 0043e720 · legions 0043ef90 — ac2-ui journal_quests.rs Rows::update
- [x] `gmJournalQuestsUI::Update` · 0043e9d0 · legions 0043f240 — ac2-ui journal_quests.rs update_pages / lay_out, run from global_loop
- [x] `gmJournalQuestsUI::FindQuestDescriptor` · 0043eae0 · legions 0043f350 — ac2-ui journal_quests.rs find_mut
- [x] `gmJournalQuestsUI::ListenToMessage` · 0043ebd0 · legions 0043f440 — ac2-ui journal_quests.rs message (QuestTemplate click) and global_loop (m_update); client hud_player.rs
- [x] `gmJournalQuestsUI::ListenToMessage` · 0043ecb0 · legions 0043f520 — ac2-ui journal_quests.rs message (QuestTemplate click) and global_loop (m_update); client hud_player.rs
- [x] `gmJournalQuestsUI::Show` · 0043ecd0 · legions 0043f540 — ac2-ui journal_quests.rs show (the updated quest closes when the panel closes); reached from quests_panel.rs QuestsUi::show
- [~] `gmJournalQuestsUI::gmJournalQuestsUI` · 0043f000 · legions 0043f8b0 — ac2-ui journal_quests.rs JournalQuestsUi::create (lists found under the pages) and drop; the notice and message registrations are the client dispatch (ac2-client ui_quests.rs, hud_player.rs); elements are left to the tree
- [~] `gmJournalQuestsUI::~gmJournalQuestsUI` · 0043f130 · legions 0043f9e0 — ac2-ui journal_quests.rs JournalQuestsUi::create (lists found under the pages) and drop; the notice and message registrations are the client dispatch (ac2-client ui_quests.rs, hud_player.rs); elements are left to the tree
- [x] `gmJournalQuestsUI::RecvNotice_UpdateQuests` · 0043f1e0 — ac2-ui journal_quests.rs recv_notice_update_quests (all codes); quests_panel.rs recv_notice makes the tabs active; client ui_quests.rs quest_notice fetches the list for code 2

## `ac2/client/src/ui/gameplay/toolbar/quest/gmquestsui.cpp.c` (3/4)

### gmQuestsUI
- [x] `gmQuestsUI::Show` · 004342d0 · legions 00434aa0 — ac2-ui quests_panel.rs QuestsUi::show and sync (journal show, QuestButton unchanged when open); the toolbar shows the field (toolbar.rs)
- [x] `gmQuestsUI::ListenToMessage` · 00434330 · legions 00434b00 — ac2-ui toolbar.rs
- [x] `gmQuestsUI::SetActivePanel` · 00434390 · legions 00434b60 — ac2-ui quests_panel.rs QuestsUi::set_active_panel (tab found by name, panel.rs start_use_active opens it and closes the other tab)
- [~] `gmQuestsUI::gmQuestsUI` · 004343c0 · legions 00434b90 — ac2-ui quests_panel.rs QuestsUi::create (journal made; the field is not modelled, the registrations are the toolbar's); QuestHelpButton opens ID_Help_Quest (hud_help.rs)

## `ac2/client/src/ui/gameplay/toolbar/quest/gmstoryquestsui.cpp.c` (9/11)

### gmSceneInfo
- [x] `gmSceneInfo::GetName` · 0043f590 · legions 0043fe40 — ac2-ui story_quests.rs (StoryQuestsUi: RecvNotice, ListenToMessage, the act and scene rows); act list natives in ac2-net act.rs and ac2-script glue/act_info.rs

### PList<gmActInfo>
- [x] `PList<gmActInfo>::PList<gmActInfo>` · 0043f680 · legions 0043ff30 — ac2-ui story_quests.rs (StoryQuestsUi: RecvNotice, ListenToMessage, the act and scene rows); act list natives in ac2-net act.rs and ac2-script glue/act_info.rs
- [x] `PList<gmActInfo>::Pack` · 0043f6c0 · legions 0062f690 — ac2-ui story_quests.rs (StoryQuestsUi: RecvNotice, ListenToMessage, the act and scene rows); act list natives in ac2-net act.rs and ac2-script glue/act_info.rs
- [x] `PList<gmActInfo>::UnPack` · 0043f710 · legions 0043ff84 — ac2-ui story_quests.rs (StoryQuestsUi: RecvNotice, ListenToMessage, the act and scene rows); act list natives in ac2-net act.rs and ac2-script glue/act_info.rs

### gmStoryQuestsUI
- [x] `gmStoryQuestsUI::~gmStoryQuestsUI` · 0043f8b0 · legions 00440110 — ac2-ui story_quests.rs (StoryQuestsUi: RecvNotice, ListenToMessage, the act and scene rows); act list natives in ac2-net act.rs and ac2-script glue/act_info.rs
- [x] `gmStoryQuestsUI::RecvNotice_UpdateStoryQuests` · 0043f9a0 · legions 00440200 — ac2-ui story_quests.rs (StoryQuestsUi: RecvNotice, ListenToMessage, the act and scene rows); act list natives in ac2-net act.rs and ac2-script glue/act_info.rs
- [~] `gmStoryQuestsUI::ReloadActs` · 0043fa30 · legions 00440290 — ac2-ui story_quests.rs reload/show: every act listed (the act record has no physics-object word, see module doc); Show clears the updated act when hidden, from the panel's own shown state
- [~] `gmStoryQuestsUI::Show` · 0043fd60 · legions 004405c0 — ac2-ui story_quests.rs reload/show: every act listed (the act record has no physics-object word, see module doc); Show clears the updated act when hidden, from the panel's own shown state
- [x] `gmStoryQuestsUI::ListenToMessage` · 0043fea0 · legions 00440700 — ac2-ui story_quests.rs (StoryQuestsUi: RecvNotice, ListenToMessage, the act and scene rows); act list natives in ac2-net act.rs and ac2-script glue/act_info.rs
- [x] `gmStoryQuestsUI::ListenToMessage` · 004400d0 · legions 00440930 — ac2-ui story_quests.rs (StoryQuestsUi: RecvNotice, ListenToMessage, the act and scene rows); act list natives in ac2-net act.rs and ac2-script glue/act_info.rs
- [x] `gmStoryQuestsUI::gmStoryQuestsUI` · 00440110 · legions 00440970 — ac2-ui quests_panel.rs (the Story page's act list under StoryPage, StoryQuestsUi::create); the Quest panel holds it

## `ac2/client/src/ui/gameplay/toolbar/skill/gmcraftui.cpp.c` (18/22)

### gmCraftUI
- [x] `gmCraftUI::SelectElement` · 004494b0 · legions 0044a110 — ac2-ui craft.rs select_element: the previous row returns to its default state, the new one shows selected
- [x] `gmCraftUI::SelectCraftSkillCategory` · 004494f0 · legions 0044a150 — ac2-ui craft.rs select_category: the category's examination (SetSelection cleared, data_enum); tested through the category row (craft_ui_7160)
- [x] `gmCraftUI::SelectRecipe` · 00449550 · legions 0044a1b0 — ac2-ui craft.rs select_recipe: the recipe's examination (data_id), the selected recipe kept even at 0; tested through the recipe button
- [x] `gmCraftUI::GetRecipeIcon` · 004495b0 · legions 0044a210 — ac2-ui craft_pages.rs recipe_icon: the recipe's icon, else the selected skill's; tested with the recipe row
- [x] `gmCraftUI::MakeLevelText` · 00449610 — ac2-ui craft_pages.rs level_text: Legions 0x44A270 (unnamed in the decompile): ID_Craft_SkillLevelNormal with level, or Buffed/Debuffed with the difference when the state argument is set; the state-token output is unused by the caller
- [x] `gmCraftUI::RefreshCraftSkillDisplay` · 00449700 · legions 0044a360 — ac2-ui craft_pages.rs refresh_skill_display: name, icon, selected field, level text (MakeLevelText buffed state), raise ghosting and raise text on SelectedLevel when the page has RaiseCraftButton; craft_ui_7160 asserts the level text
- [x] `gmCraftUI::MakeCraftSkill` · 004499e0 · legions 0044a640 — ac2-ui craft_pages.rs make_skill: DID on row and back button, name state enabled/disabled, ellipsis cut with tooltip (ID_Ellipsis), Level text ID_Craft_Rating with the level and state buff/debuff/stateless, selected row. String and element names from the admin Tokenize initializers matched by position (tools in the worker scratchpad)
- [x] `gmCraftUI::MakeCraftSkillCategory` · 00449bb0 · legions 0044a810 — ac2-ui craft_pages.rs make_category: DID, forward/back button, name with ellipsis cut and tooltip, selected row
- [x] `gmCraftUI::MakeRecipe` · 00449ce0 · legions 0044a940 — ac2-ui craft_pages.rs make_recipe: icon, button, name state (disabled while m_timeUseAgain, a double, is above 0), ellipsis cut and tooltip, selected row
- [x] `gmCraftUI::ClearSelectionInfo` · 00449eb0 · legions 0044ab10 — ac2-ui craft.rs clear_selection_info: the row maps and selected element cleared
- [x] `gmCraftUI::UpdateCategories` · 0044a040 · legions 0044aca0 — ac2-ui craft_pages.rs update_categories: one row per category, sorted by category id (CraftSkillCategoryAdaptor::operator< compares m_category, legions 005cdb70); craft_ui_7160
- [-] `gmCraftUI::~gmCraftUI` · 0044a1a0 · legions 0044ae00 — the handler registrations are per object in Rust (no per-object table); the panel's state is dropped with the HUD
- [ ] `gmCraftUI::RecvNotice_ExaminationRequestChanged` · 0044a2d0 — Legions has it: gmCraftUI notice vtable slot 69 at 0x0044af30 (vtable audit)
- [x] `gmCraftUI::SelectCraftSkill` · 0044a3b0 · legions 0044b010 — ac2-ui craft.rs select_craft_skill: recipes and categories of the skill, the selection cleared, the examination; tested through the skill row (craft_ui_7160)
- [x] `gmCraftUI::RefreshCraftSkillPage` · 0044a440 · legions 0044b0a0 — ac2-ui craft_pages.rs refresh_skill_page / refresh_category_page / refresh_recipe_page over make_skill, make_category, make_recipe and refresh_skill_display (raise text ID_Craft_CantRaise / ID_Craft_AvailableXPRaise)
- [x] `gmCraftUI::RefreshCraftSkillCategoryPage` · 0044a550 · legions 0044b1b0 — ac2-ui craft_pages.rs refresh_skill_page / refresh_category_page / refresh_recipe_page over make_skill, make_category, make_recipe and refresh_skill_display (raise text ID_Craft_CantRaise / ID_Craft_AvailableXPRaise)
- [x] `gmCraftUI::RefreshRecipePage` · 0044a6e0 · legions 0044b340 — ac2-ui craft_pages.rs refresh_skill_page / refresh_category_page / refresh_recipe_page over make_skill, make_category, make_recipe and refresh_skill_display (raise text ID_Craft_CantRaise / ID_Craft_AvailableXPRaise)
- [x] `gmCraftUI::RefreshCraftPanel` · 0044a8e0 · legions 0044b540 — ac2-ui craft.rs refresh_panel: the recipes of the selected skill, then the view's page (SkillPage, CategoryPage, RecipePage)
- [~] `gmCraftUI::gmCraftUI` · 0044a960 · legions 0044b5c0 — ac2-ui craft.rs CraftUi::create: the page and its lists looked up per refresh (GetChildRecursive). Not: the drag field and the notice handlers (per object in Rust)
- [x] `gmCraftUI::SetState` · 0044ab30 · legions 0044b790 — ac2-ui craft.rs set_state: the page takes the view, then the panel refreshes
- [x] `gmCraftUI::RecvNotice_UpdateCraftPanel` · 0044ab60 — ac2-ui craft.rs CraftUi::frame_pass refreshes the panel on the notice; the frame pass runs in ac2-ui frame_passes.rs Hud::frame (HudRequest::Craft); ac2-client hud_craft.rs is the CraftSource adapter over the player's queries
- [~] `gmCraftUI::ListenToMessage` · 0044ab70 · legions 0044b7d0 — ac2-ui craft.rs message: forward, back, the headers, the rows (press and double click), recipe button, raise button, craft tab opened. Not: the recipe's icon drag (drag field, start drag), the raise button is not under the page in the layout tree yet

## `ac2/client/src/ui/gameplay/toolbar/skill/gmskilltreeui.cpp.c` (21/26)

### gmSkillTreeUI
- [x] `gmSkillTreeUI::MakeUntrainCancelDialog` · 00447550 · legions 00448090 — ac2-ui skill_tree.rs: confirmation dialogs, tooltips, truncation, Show closes dialogs on a shown-state change
- [x] `gmSkillTreeUI::MakeTrainDialog` · 004475b0 · legions 004480f0 — ac2-ui skill_tree.rs: confirmation dialogs, tooltips, truncation, Show closes dialogs on a shown-state change
- [x] `gmSkillTreeUI::MakeLevelText` · 00447610 · legions 00448150 — ac2-ui skill_tree.rs
- [x] `gmSkillTreeUI::SelectNode` · 00447700 · legions 00448240 — ac2-ui skill_tree.rs
- [x] `gmSkillTreeUI::Show` · 00447760 · legions 004482a0 — ac2-ui skill_tree.rs: confirmation dialogs, tooltips, truncation, Show closes dialogs on a shown-state change
- [x] `gmSkillTreeUI::RecvNotice_CloseConfirmationDialog` · 004477c0 · legions 00448300 — ac2-ui skill_tree.rs: confirmation dialogs, tooltips, truncation, Show closes dialogs on a shown-state change
- [x] `gmSkillTreeUI::SetStringAndTruncate` · 00447890 · legions 004483d0 — ac2-ui skill_tree.rs: confirmation dialogs, tooltips, truncation, Show closes dialogs on a shown-state change

### HashIterator<unsigned_long,BaseList<unsigned_long,11>_>
- [-] `HashIterator<unsigned_long,BaseList<unsigned_long,11>_>::operator++` · 004478f0 · legions 0062aa70 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### gmSkillTreeUI
- [x] `gmSkillTreeUI::UpdateSelectedSkillDisplay` · 00447930 · legions 00448450 — ac2-ui skill_tree.rs: confirmation dialogs, tooltips, truncation, Show closes dialogs on a shown-state change
- [x] `gmSkillTreeUI::RefreshSkillNode` · 00447d50 · legions 00448870 — ac2-ui skill_tree.rs: confirmation dialogs, tooltips, truncation, Show closes dialogs on a shown-state change
- [x] `gmSkillTreeUI::FastRefreshSkillNode` · 00448040 · legions 00448bd0 — ac2-ui skill_tree.rs
- [x] `gmSkillTreeUI::HandleElement` · 00448110 · legions 00448ca0 — ac2-ui skill_tree.rs
- [x] `gmSkillTreeUI::RecvNotice_UpdateSkillPanel` · 00448250 — ac2-ui skill_tree.rs
- [x] `gmSkillTreeUI::SelectSkill` · 00448310 · legions 00448eb0 — ac2-ui skill_tree.rs
- [x] `gmSkillTreeUI::MakeUntrainDialog` · 00448430 · legions 00448fd0 — ac2-ui skill_tree.rs: confirmation dialogs, tooltips, truncation, Show closes dialogs on a shown-state change
- [x] `gmSkillTreeUI::ClearSelectedInfo` · 004484c0 · legions 00449060 — ac2-ui skill_tree.rs
- [x] `gmSkillTreeUI::ListenToMessage` · 00448630 · legions 00449230 — ac2-ui skill_tree.rs: confirmation dialogs, tooltips, truncation, Show closes dialogs on a shown-state change
- [x] `gmSkillTreeUI::InitElements` · 00448810 · legions 004496b0 — ac2-ui skill_tree.rs

### SkillUINode
- [ ] `SkillUINode::~SkillUINode` · 00448860 · legions 00449140 — needs port: SkillUINode::~SkillUINode lifecycle (teardown/creation)

### gmSkillTreeUI
- [x] `gmSkillTreeUI::RecvNotice_UpdateSkill` · 004488c0 — ac2-ui skill_tree.rs
- [x] `gmSkillTreeUI::AddNode` · 004489c0 · legions 00449550 — ac2-ui skill_tree.rs
- [x] `gmSkillTreeUI::LoadPanel` · 00448c70 — ac2-ui skill_tree.rs: confirmation dialogs, tooltips, truncation, Show closes dialogs on a shown-state change
- [x] `gmSkillTreeUI::RecvNotice_ExaminationRequestChanged` · 00448e50 — ac2-ui skill_tree.rs

### PHashTable<unsigned_long,SkillUINode>
- [-] `PHashTable<unsigned_long,SkillUINode>::StreamPack` · 00448eb0 · legions 00449ae0 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### gmSkillTreeUI
- [~] `gmSkillTreeUI::gmSkillTreeUI` · 00449110 · legions 00449d40 — ac2-ui skill_tree.rs: no tooltips, dialogs as requests, no truncation
- [-] `gmSkillTreeUI::~gmSkillTreeUI` · 00449370 · legions 00449fc0 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own

## `ac2/client/src/ui/gameplay/toolbar/skill/gmskillui.cpp.c` (3/3)

### gmSkillUI
- [x] `gmSkillUI::Show` · 00435000 · legions 004357d0 — toolbar child toggle and close (ac2-ui toolbar.rs; view_7160 and paper_doll_7160 show the Skill and Inventory panels); children made with the HUD (hud.rs). The help button's KeyStone::OpenHelp opens the help window (hud_help.rs; gmToolbarUI::ShowKeyStone toggles it)
- [x] `gmSkillUI::ListenToMessage` · 00435030 · legions 00435800 — toolbar child toggle and close (ac2-ui toolbar.rs; view_7160 and paper_doll_7160 show the Skill and Inventory panels); children made with the HUD (hud.rs). The help button's KeyStone::OpenHelp opens the help window (hud_help.rs; gmToolbarUI::ShowKeyStone toggles it)
- [x] `gmSkillUI::gmSkillUI` · 00435090 · legions 00435860 — toolbar child toggle and close (ac2-ui toolbar.rs; view_7160 and paper_doll_7160 show the Skill and Inventory panels); children made with the HUD (hud.rs). The help button's KeyStone::OpenHelp opens the help window (hud_help.rs; gmToolbarUI::ShowKeyStone toggles it)

## `ac2/client/src/ui/gameplay/toolbar/social/gmallegianceui.cpp.c` (15/20)

### AllegianceDataAdaptor
- [-] `AllegianceDataAdaptor::~AllegianceDataAdaptor` · 0043c7a0 · legions 0043cff0 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own

### gmAllegianceUI
- [x] `gmAllegianceUI::MakeCurrent` · 0043c7c0 · legions 0043d010 — ac2-ui allegiance.rs; ac2-client ui_social.rs
- [x] `gmAllegianceUI::MakeVassal` · 0043ca20 · legions 0043d270 — ac2-ui allegiance.rs; ac2-client ui_social.rs
- [x] `gmAllegianceUI::SendRequest` · 0043cbc0 · legions 0043d410 — ac2-ui allegiance.rs; ac2-client ui_social.rs
- [x] `gmAllegianceUI::UpdateButtons` · 0043cce0 · legions 0043d530 — ac2-ui allegiance.rs; ac2-client ui_social.rs
- [x] `gmAllegianceUI::FocusTree` · 0043ce50 · legions 0043d6a0 — ac2-ui allegiance.rs; ac2-client ui_social.rs

### HashSet<InstanceID>
- [-] `HashSet<InstanceID>::operator=` · 0043ce80 · legions 0043d6d0 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### gmAllegianceUI
- [x] `gmAllegianceUI::RecvNotice_SelectionChanged` · 0043cfe0 — ac2-ui allegiance.rs; ac2-client ui_social.rs
- [~] `gmAllegianceUI::UpdateShowState` · 0043d000 · legions 0043d850 — follows the tab and field each frame instead of their opened/closed/set_show_state messages
- [x] `gmAllegianceUI::MakeAcceptSwearDialog` · 0043d080 · legions 0043d8d0 — ac2-ui allegiance.rs; ac2-client ui_social.rs
- [x] `gmAllegianceUI::MakeBreakDialog` · 0043d100 · legions 0043d950 — ac2-ui allegiance.rs; ac2-client ui_social.rs
- [x] `gmAllegianceUI::DoBreak` · 0043d180 · legions 0043d9d0 — ac2-ui allegiance.rs; ac2-client ui_social.rs
- [-] `gmAllegianceUI::~gmAllegianceUI` · 0043d290 · legions 0043dae0
- [~] `gmAllegianceUI::ListenToMessage` · 0043d350 · legions 0043dba0 — follows the tab and field each frame instead of their opened/closed/set_show_state messages
- [x] `gmAllegianceUI::RecvNotice_CloseConfirmationDialog` · 0043d440 · legions 0043dc90 — ac2-ui allegiance.rs; ac2-client ui_social.rs
- [x] `gmAllegianceUI::RecvNotice_RequestVassal` · 0043d4a0 — ac2-ui allegiance.rs; ac2-client ui_social.rs
- [x] `gmAllegianceUI::RecvNotice_Patron_SwearDone` · 0043d4b0 — ac2-ui allegiance.rs; ac2-client ui_social.rs
- [x] `gmAllegianceUI::gmAllegianceUI` · 0043d4e0 · legions 0043dd30 — ac2-ui allegiance.rs; ac2-client ui_social.rs
- [x] `gmAllegianceUI::Update` · 0043d790 · legions 0043dfe0 — ac2-ui allegiance.rs; ac2-client ui_social.rs
- [x] `gmAllegianceUI::RecvNotice_UpdateAllegianceProfile` · 0043dcb0 — ac2-ui allegiance.rs; ac2-client ui_social.rs

## `ac2/client/src/ui/gameplay/toolbar/social/gmfellowshipui.cpp.c` (16/17)

### gmFellowshipUI
- [~] `gmFellowshipUI::InitLootingMenu` · 0043bde0 · legions 0043c630 — ac2-ui fellowship.rs: LootingMethodMenu filled with the five ID_Fellowship_LootMethod names (table 0x41000018), method 1 selected; item tooltips (ID_Fellowship_TT_LootMethod*) not set
- [x] `gmFellowshipUI::HandleRecruitButton` · 0043bea0 · legions 0043c6f0 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs
- [x] `gmFellowshipUI::HandleDismissButton` · 0043bef0 · legions 0043c740 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs
- [x] `gmFellowshipUI::SetLeaderState` · 0043bf40 · legions 0043c790 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs
- [x] `gmFellowshipUI::MakeRecruitDialog` · 0043bfd0 · legions 0043c820 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs
- [x] `gmFellowshipUI::RecvNotice_CloseConfirmationDialog` · 0043c050 · legions 0043c8a0 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs
- [x] `gmFellowshipUI::RecvNotice_RequestRecruitment` · 0043c0a0 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs
- [x] `gmFellowshipUI::RecvNotice_ExpireRecruitment` · 0043c0c0 · legions 0043c910 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs
- [x] `gmFellowshipUI::RecvNotice_UpdateLeader` · 0043c0e0 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs
- [x] `gmFellowshipUI::~gmFellowshipUI` · 0043c130 · legions 0043c980 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs
- [x] `gmFellowshipUI::HandleCreateButton` · 0043c1d0 · legions 0043ca20 — ac2-ui fellowship.rs: Create sends the entered name, the XP toggle and the looting menu's selected index (client test social_7160 asserts WM_Fellowship bytes for looting 3)
- [x] `gmFellowshipUI::SetFellowshipState` · 0043c280 · legions 0043cad0 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs
- [x] `gmFellowshipUI::RecvNotice_UpdateFellowship` · 0043c3c0 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs
- [x] `gmFellowshipUI::ListenToMessage` · 0043c4e0 · legions 0043cd30 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs
- [x] `gmFellowshipUI::SetNoFellowshipState` · 0043c570 · legions 0043cdc0 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs
- [x] `gmFellowshipUI::RecvNotice_ClearFellowship` · 0043c5c0 · legions 0043ce10 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs
- [x] `gmFellowshipUI::gmFellowshipUI` · 0043c630 · legions 0043ce80 — ac2-ui fellowship.rs / fellow_info.rs; ac2-client ui_social.rs

## `ac2/client/src/ui/gameplay/toolbar/social/gmfriendsui.cpp.c` (11/20)

### gmFriendsUI
- [~] `gmFriendsUI::ListenToMessage` · 0043a570 · legions 0043adc0 — ac2-ui friends_panel.rs message: add, remove and status buttons; the buttons follow the selection (set_target). Not modelled: the status drop-down's LocalStatus setting, the IsMoving reset of the menu and the GUN away/dnd text
- [x] `gmFriendsUI::RecvNotice_GUNResult_AddFriend` · 0043a5c0 — ac2-ui friends_panel.rs (FriendsPanel: BuildUIList rows, recv_* , remove request); ac2-client hud_friends.rs and friends.rs (GUN list/status feed); tests friends_page_7160, friends_script_7160
- [x] `gmFriendsUI::RecvNotice_LocalStatus` · 0043a5e0 · legions 0043ae30 — ac2-ui friends_panel.rs + ac2-client hud_friends.rs: the status drop-down follows the server's local status (RecvNotice_LocalStatus, broadcast as SetSelectedIndex does)

### PStringBase<unsigned_short>
- [-] `PStringBase<unsigned_short>::operator==` · 0043a620 · legions 0043ae70 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them
- [-] `PStringBase<unsigned_short>::operator!=` · 0043a650 · legions 0043aea0 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### TextPredicate
- [-] `TextPredicate::TextPredicate` · 0043a680 · legions 0043aed0 — ctor/dtor/operator= of plain data; Rust struct construction and Drop replace it
- [-] `TextPredicate::TextPredicate` · 0043a6f0 · legions 0043af40 — ctor/dtor/operator= of plain data; Rust struct construction and Drop replace it

### gmFriendsUI
- [~] `gmFriendsUI::HandleAddButton` · 0043a710 · legions 0043af60 — ac2-ui friends_panel.rs: the add button sends the selected other player's name (hud_friends.rs friend_target; HandleAddButton's IsPlayer test and GetName(0)), the remove button and the status drop-down are wired and filled. Not modelled: the GUN away/dnd status text, the drop-down's IsMoving reset on the global loop (0xFFFF0005) and the server status setting the drop-down (LocalStatus)
- [x] `gmFriendsUI::HandleRemoveButton` · 0043a7a0 · legions 0043aff0 — ac2-ui friends_panel.rs (FriendsPanel: BuildUIList rows, recv_* , remove request); ac2-client hud_friends.rs and friends.rs (GUN list/status feed); tests friends_page_7160, friends_script_7160
- [x] `gmFriendsUI::HandleStatusChange` · 0043a820 · legions 0043b070 — ac2-ui friends_panel.rs: FriendsPanel::message on StatusDropDownMenu's new selection -> status_request; ac2-client friends_command sends the GUN flags and the emote
- [x] `gmFriendsUI::UpdateButtonStates` · 0043abe0 · legions 0043b430 — ac2-ui friends_panel.rs (set_target, update_add: add enabled for another player's name with no list row selected and not a friend; remove for a selected friend) and ac2-client hud_friends.rs friend_target (the selection's IsPlayer and not the local player, weenie type 0x14 & 0x1F); tested on the real social layout in friends_panel_tests.rs
- [~] `gmFriendsUI::ListenToMessage` · 0043ae30 · legions 0043b680 — ac2-ui friends_panel.rs message: add, remove and status buttons; the buttons follow the selection (set_target). Not modelled: the status drop-down's LocalStatus setting, the IsMoving reset of the menu and the GUN away/dnd text
- [x] `gmFriendsUI::RecvNotice_SelectionChanged` · 0043ae90 — Legions has the handler (an undecompiled vtable slot); ac2-ui friends_panel.rs set_target follows the game selection every frame (hud_player pick_pass), which gives the add and remove buttons the same state as the notice did

### PList<TurbineUserStatus>
- [-] `PList<TurbineUserStatus>::UnPack` · 0043b020 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### gmFriendsUI
- [x] `gmFriendsUI::BuildUIList` · 0043b190 · legions 0043b9e0 — ac2-ui friends_panel.rs + ac2-client hud_friends.rs: rows split on mode (m_Mode != 0 online, admin's RecvNotice_GUNResult_FriendStatus/GetFriendsList), AFK then DND text appended and token_awayfromkeyboard/token_donotdisturb states (BuildUIList), LocalStatus selects the drop-down (broadcast)
- [x] `gmFriendsUI::RecvNotice_GUNResult_RemoveFriend` · 0043b4a0 — ac2-ui friends_panel.rs (FriendsPanel: BuildUIList rows, recv_* , remove request); ac2-client hud_friends.rs and friends.rs (GUN list/status feed); tests friends_page_7160, friends_script_7160
- [~] `gmFriendsUI::gmFriendsUI` · 0043b6f0 · legions 0043bf40 — ac2-ui friends_panel.rs: the add button sends the selected other player's name (hud_friends.rs friend_target; HandleAddButton's IsPlayer test and GetName(0)), the remove button and the status drop-down are wired and filled. Not modelled: the GUN away/dnd status text, the drop-down's IsMoving reset on the global loop (0xFFFF0005) and the server status setting the drop-down (LocalStatus)
- [x] `gmFriendsUI::~gmFriendsUI` · 0043b8d0 · legions 0043c120 — ac2-ui friends_panel.rs (FriendsPanel: BuildUIList rows, recv_* , remove request); ac2-client hud_friends.rs and friends.rs (GUN list/status feed); tests friends_page_7160, friends_script_7160
- [x] `gmFriendsUI::RecvNotice_GUNResult_GetFriendsList` · 0043ba00 — ac2-ui friends_panel.rs + ac2-client hud_friends.rs: rows split on mode (m_Mode != 0 online, admin's RecvNotice_GUNResult_FriendStatus/GetFriendsList), AFK then DND text appended and token_awayfromkeyboard/token_donotdisturb states (BuildUIList), LocalStatus selects the drop-down (broadcast)
- [x] `gmFriendsUI::RecvNotice_GUNResult_FriendStatus` · 0043bb90 — ac2-ui friends_panel.rs + ac2-client hud_friends.rs: rows split on mode (m_Mode != 0 online, admin's RecvNotice_GUNResult_FriendStatus/GetFriendsList), AFK then DND text appended and token_awayfromkeyboard/token_donotdisturb states (BuildUIList), LocalStatus selects the drop-down (broadcast)

## `ac2/client/src/ui/gameplay/toolbar/social/gmsocialui.cpp.c` (0/3)

### gmSocialUI
- [~] `gmSocialUI::Show` · 00430620 · legions 004303e0 — ac2-ui social_page.rs: SocialField shown/hidden, SocialButton toggle, SocialCloseButton; SocialHelpButton returns SocialAction::OpenHelp, opened as ID_Help_Social (hud_player_message.rs); UIFramework child-framework propagation not modelled; social_page_tests.rs
- [~] `gmSocialUI::ListenToMessage` · 00430650 · legions 00430410 — ac2-ui social_page.rs: SocialField shown/hidden, SocialButton toggle, SocialCloseButton; SocialHelpButton returns SocialAction::OpenHelp, opened as ID_Help_Social (hud_player_message.rs); UIFramework child-framework propagation not modelled; social_page_tests.rs
- [~] `gmSocialUI::gmSocialUI` · 004306b0 · legions 00430470 — ac2-ui social_page.rs: SocialField shown/hidden, SocialButton toggle, SocialCloseButton; SocialHelpButton returns SocialAction::OpenHelp, opened as ID_Help_Social (hud_player_message.rs); UIFramework child-framework propagation not modelled; social_page_tests.rs

## `ac2/client/src/ui/gmuiflow.cpp.c` (10/37)

### gmUIFlow
- [x] `gmUIFlow::~gmUIFlow` · 00405d80 · legions 00405a60 — ac2-ui pregame/flow.rs, pregame/ui.rs

### UIFlow
- [ ] `UIFlow::GetSmartBox` · 00405df0 · legions 00405ae0 — needs port (check): UIFlow::GetSmartBox behaviour; implemented area ac2-ui hud.rs but uncited
- [x] `UIFlow::GetUIMainFramework` · 00405e00 — ac2-ui pregame/flow.rs, pregame/ui.rs

### gmUIFlow
- [~] `gmUIFlow::ListenToMessage` · 00405e10 · legions 00405af0 — screenshot key 0x41000017 on release: crates/ac2-client/src/bin/ac2/screenshot.rs (the other debug keys are admin-only, skip)

### PLineOffsetList
- [-] `PLineOffsetList::~PLineOffsetList` · 004060b0 · legions 00405dc0 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### POriginalSourceFileInfo
- [-] `POriginalSourceFileInfo::~POriginalSourceFileInfo` · 00406100 · legions 00405e10 — authoring/export/debug-info tooling (checkpoint export, file tree, source line info); no runtime behaviour

### FrameDebugInfo
- [-] `FrameDebugInfo::~FrameDebugInfo` · 00406150 · legions 00405e60 — authoring/export/debug-info tooling (checkpoint export, file tree, source line info); no runtime behaviour

### ImportData
- [-] `ImportData::~ImportData` · 004061b0 · legions 00405ec0 — authoring/export/debug-info tooling (checkpoint export, file tree, source line info); no runtime behaviour

### gmUIFlow
- [x] `gmUIFlow::RecvNotice_CharacterError` · 00406220 — ac2-ui pregame/flow.rs, pregame/ui.rs
- [x] `gmUIFlow::RecvNotice_ServerDied` · 00406410 — ac2-ui pregame/flow.rs, pregame/ui.rs
- [-] `gmUIFlow::RecvNotice_GunConnectionLost` · 004064a0 — GUN chat servers are not used (Turbine chat)
- [x] `gmUIFlow::RecvNotice_CharacterError` · 00406af0 — ac2-ui pregame/flow.rs, pregame/ui.rs
- [x] `gmUIFlow::RecvNotice_ServerDied` · 00406b00 — ac2-ui pregame/flow.rs, pregame/ui.rs
- [x] `gmUIFlow::gmUIFlow` · 00406b10 — ac2-ui pregame/ui.rs PregameUi::new (every framework, the intro queued); gameplay is the client's view

### ImportData
- [~] `ImportData::StreamPack` · 00406c60 · legions 00406970 — ac2-script: unpack direction only

### PStringBase<char>
- [x] `PStringBase<char>::UnPack` · 00406cb0 · legions 004069c0 — ac2-core pstring.rs: the DAT string descramble (ac2-ui, ac2-db use it)

### DualHash<PStringBase<char>,CheckpointExportData>
- [-] `DualHash<PStringBase<char>,CheckpointExportData>::clear` · 00406d90 · legions 00406aa0 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them
- [-] `DualHash<PStringBase<char>,CheckpointExportData>::destroy` · 00406ea0 · legions 00406bb0 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them
- [-] `DualHash<PStringBase<char>,CheckpointExportData>::~DualHash<PStringBase<char>,CheckpointExportData>` · 00406ef0 · legions 00406c00 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them
- [x] `DualHash<PStringBase<char>,CheckpointExportData>::UnPack` · 00406f50 — ac2-script reference/native unpack

### CheckpointExportData
- [-] `CheckpointExportData::Pack` · 00407010 · legions 00406d20 — authoring/export/debug-info tooling (checkpoint export, file tree, source line info); no runtime behaviour
- [x] `CheckpointExportData::UnPack` · 00407050 · legions 00406d60 — ac2-script reference/native unpack

### DualHash<PStringBase<char>,CheckpointExportData>
- [-] `DualHash<PStringBase<char>,CheckpointExportData>::add_internal` · 004070b0 · legions 00406dc0 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### DualHashData<PStringBase<char>,CheckpointExportData>
- [-] `DualHashData<PStringBase<char>,CheckpointExportData>::DualHashData<PStringBase<char>,CheckpointExportData>` · 004071a0 · legions 00406eb0 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### DualHash<PStringBase<char>,CheckpointExportData>
- [-] `DualHash<PStringBase<char>,CheckpointExportData>::GetPackSize` · 004071e0 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them
- [-] `DualHash<PStringBase<char>,CheckpointExportData>::pack_size` · 004071f0 · legions 00406f00 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### DualHashIter<PStringBase<char>,CheckpointExportData>
- [-] `DualHashIter<PStringBase<char>,CheckpointExportData>::DualHashIter<PStringBase<char>,CheckpointExportData>` · 004072d0 · legions 00406fe0 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them
- [-] `DualHashIter<PStringBase<char>,CheckpointExportData>::operator=` · 00407300 · legions 00407010 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them
- [-] `DualHashIter<PStringBase<char>,CheckpointExportData>::DualHashIter<PStringBase<char>,CheckpointExportData>` · 00407350 · legions 00407060 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them
- [-] `DualHashIter<PStringBase<char>,CheckpointExportData>::DualHashIter<PStringBase<char>,CheckpointExportData>` · 004073b0 · legions 004070c0 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them
- [-] `DualHashIter<PStringBase<char>,CheckpointExportData>::operator==` · 004073f0 · legions 00407100 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them
- [-] `DualHashIter<PStringBase<char>,CheckpointExportData>::operator++` · 00407460 · legions 00407170 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### DualHash<PStringBase<char>,CheckpointExportData>
- [-] `DualHash<PStringBase<char>,CheckpointExportData>::Pack` · 004074a0 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### ListIterator<DualHashData<PStringBase<char>,CheckpointExportData>*>
- [-] `ListIterator<DualHashData<PStringBase<char>,CheckpointExportData>*>::operator=` · 004075f0 · legions 00407300 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### ExportPackageArgs
- [-] `ExportPackageArgs::~ExportPackageArgs` · 004076d0 · legions 004073e0 — authoring/export/debug-info tooling (checkpoint export, file tree, source line info); no runtime behaviour

### ExportData
- [-] `ExportData::~ExportData` · 00407720 · legions 00407430 — authoring/export/debug-info tooling (checkpoint export, file tree, source line info); no runtime behaviour
- [~] `ExportData::StreamPack` · 00407770 · legions 00407480 — ac2-script: unpack direction only

## `ac2/client/src/ui/pregame/generation/gmavatarmodificationui.cpp.c` (19/27)

### AFrame
- [-] `AFrame::AFrame` · 0040a9e0 · legions 0040a8a0 — ctor/dtor/operator= of plain data; Rust struct construction and Drop replace it

### Frame
- [-] `Frame::Frame` · 0040aa00 · legions 0040a8c0 — ctor/dtor/operator= of plain data; Rust struct construction and Drop replace it
- [-] `Frame::operator=` · 0040aa30 · legions 0040a8f0 — plain data copy operator (Frame::operator=); Rust struct copy

### UIRegion
- [ ] `UIRegion::IsVisible` · 0040aaa0 · legions 0040a960 — needs port (check): UIRegion::IsVisible; element.rs visible() exists but the cite sits on rect()

### gmAvatarModificationUI
- [x] `gmAvatarModificationUI::GetCamera` · 0040aad0 · legions 0040a990 — pregame 3D scenes: ac2-assets ac2_ui_scene.rs, ac2-client pregame_scene/, ac2-ui pregame/scene.rs
- [x] `gmAvatarModificationUI::MakeCurrentMask` · 0040ab10 · legions 0040a9d0 — ac2-ui pregame/avatar_modification.rs
- [x] `gmAvatarModificationUI::MakePleaseWaitDialog` · 0040ab70 · legions 0040aa30 — ac2-ui pregame/avatar_modification.rs

### Frame
- [-] `Frame::Frame` · 0040ac80 · legions 0040ab40 — ctor/dtor/operator= of plain data; Rust struct construction and Drop replace it

### UISceneEntity
- [x] `UISceneEntity::GetFrame` · 0040acb0 · legions 0040ab70 — pregame 3D scenes: ac2-assets ac2_ui_scene.rs, ac2-client pregame_scene/, ac2-ui pregame/scene.rs

### gmAvatarModificationUI
- [x] `gmAvatarModificationUI::UpdateCamera` · 0040ace0 · legions 0040aba0 — pregame 3D scenes: ac2-assets ac2_ui_scene.rs, ac2-client pregame_scene/, ac2-ui pregame/scene.rs
- [x] `gmAvatarModificationUI::InitCamera` · 0040b0b0 · legions 0040af70 — pregame 3D scenes: ac2-assets ac2_ui_scene.rs, ac2-client pregame_scene/, ac2-ui pregame/scene.rs

### PStringBase<char>
- [-] `PStringBase<char>::operator+` · 0040b300 · legions 0040b200 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### gmAvatarModificationUI
- [x] `gmAvatarModificationUI::Init` · 0040b420 · legions 0040b320 — ac2-ui pregame/avatar_modification.rs (sliders, parts, randomize, zoom/rotation camera state, tooltips, naming dialog, music) with ac2-client pregame_scene/modification.rs driving the camera; captions are the script StringInfo labels in their own string tables (UiStrings preloads every string table)
- [x] `gmAvatarModificationUI::UpdateByMask` · 0040b640 · legions 0040b540 — ac2-ui pregame/avatar_modification.rs (sliders, parts, randomize, zoom/rotation camera state, tooltips, naming dialog, music) with ac2-client pregame_scene/modification.rs driving the camera; captions are the script StringInfo labels in their own string tables (UiStrings preloads every string table)
- [x] `gmAvatarModificationUI::Temp` · 0040b6e0 · legions 0040b5e0 — ac2-ui pregame/avatar_modification.rs
- [x] `gmAvatarModificationUI::InitUIFromData` · 0040b760 · legions 0040b660 — ac2-ui pregame/avatar_modification.rs
- [x] `gmAvatarModificationUI::InitElements` · 0040b8c0 · legions 0040b7c0 — ac2-ui avatar_modification.rs init_elements: the slider, right and left arrow tooltips by ID_AvModify_TT_* string ids

### PList<unsigned_long>
- [-] `PList<unsigned_long>::GetPackSize` · 0040bb40 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### PList<float>
- [-] `PList<float>::UnPack` · 0040bb60 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### gmAvatarModificationUI
- [x] `gmAvatarModificationUI::ListenToMessage` · 0040bc90 · legions 0040bc60 — ac2-ui pregame/avatar_modification.rs (sliders, parts, randomize, zoom/rotation camera state, tooltips, naming dialog, music) with ac2-client pregame_scene/modification.rs driving the camera; captions are the script StringInfo labels in their own string tables (UiStrings preloads every string table)
- [x] `gmAvatarModificationUI::InitUITables` · 0040c0d0 — ac2-ui pregame/avatar_modification.rs
- [x] `gmAvatarModificationUI::ClearDataTable` · 0040c7e0 · legions 0040c800 — ac2-ui pregame/avatar_modification.rs
- [x] `gmAvatarModificationUI::GetDataTable` · 0040c880 · legions 0040c8a0 — ac2-ui pregame/avatar_modification.rs
- [x] `gmAvatarModificationUI::gmAvatarModificationUI` · 0040cc00 — ac2-ui pregame/avatar_modification.rs (sliders, parts, randomize, zoom/rotation camera state, tooltips, naming dialog, music) with ac2-client pregame_scene/modification.rs driving the camera; captions are the script StringInfo labels in their own string tables (UiStrings preloads every string table)
- [x] `gmAvatarModificationUI::~gmAvatarModificationUI` · 0040cff0 · legions 0040d010 — ac2-ui pregame/avatar_modification.rs
- [x] `gmAvatarModificationUI::Create` · 0040d290 — ac2-ui pregame/avatar_modification.rs
- [x] `gmAvatarModificationUI::Register` · 0040d2d0 · legions 0040d2f0 — ac2-ui pregame/avatar_modification.rs

## `ac2/client/src/ui/pregame/generation/gmavatarselectionui.cpp.c` (9/25)

### _InstanceID
- [-] `_InstanceID::operator==` · 0040d2f0 · legions 0040d310 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them
- [-] `_InstanceID::operator!=` · 0040d310 · legions 0040d330 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### ObjectSelection
- [-] `ObjectSelection::ObjectSelection` · 0040d330 · legions 0040d350 — ctor/dtor/operator= of plain data; Rust struct construction and Drop replace it
- [-] `ObjectSelection::operator=` · 0040d360 · legions 0040d380 — ctor/dtor/operator= of plain data; Rust struct construction and Drop replace it

### gmAvatarSelectionUI
- [x] `gmAvatarSelectionUI::MakePleaseWaitDialog` · 0040d390 · legions 0040d3b0 — ac2-ui pregame/avatar_selection.rs

### SmartArray<UISceneEntity_*>
- [-] `SmartArray<UISceneEntity_*>::AddToEnd` · 0040d690 · legions 0040ff80 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### PList<unsigned_long>
- [-] `PList<unsigned_long>::Pack` · 0040d9d0 · legions 0040bad0 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them
- [-] `PList<unsigned_long>::UnPack` · 0040da30 · legions 0040da32 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### BehaviorParams
- [-] `BehaviorParams::~BehaviorParams` · 0040daf0 · legions 0040dae0 — ctor/dtor/operator= of plain data; Rust struct construction and Drop replace it

### gmAvatarSelectionUI
- [~] `gmAvatarSelectionUI::MouseOffObject` · 0040db40 · legions 0040db30 — avatar_selection.rs mouse_off_object / mouse_on_object: the viewport tooltip ID_AvSelect_TT_3DAvatar and RaceDescField state; retail check of the object current mode (0x41000006 off, 0x41000007 on) is not made in the UI
- [~] `gmAvatarSelectionUI::MouseOnObject` · 0040dbd0 · legions 0040dbc0 — avatar_selection.rs mouse_off_object / mouse_on_object: the viewport tooltip ID_AvSelect_TT_3DAvatar and RaceDescField state; retail check of the object current mode (0x41000006 off, 0x41000007 on) is not made in the UI

### BaseList<unsigned_long,11>
- [-] `BaseList<unsigned_long,11>::BaseList<unsigned_long,11>` · 0040dca0 · legions 0040dc90 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### gmAvatarSelectionUI
- [x] `gmAvatarSelectionUI::ListenToMessage` · 0040de00 · legions 0040dde0 — ac2-ui pregame/avatar_selection.rs (pick, selected-animation protocol, primSong music) with ac2-client pregame_scene/ (line-up, mouse-over mode, selected behavior 0x41000005)
- [x] `gmAvatarSelectionUI::ListenToMessage` · 0040dfe0 · legions 0040dfc0 — ac2-ui pregame/avatar_selection.rs (pick, selected-animation protocol, primSong music) with ac2-client pregame_scene/ (line-up, mouse-over mode, selected behavior 0x41000005)

### BaseList<unsigned_long,11>
- [-] `BaseList<unsigned_long,11>::BaseList<unsigned_long,11>` · 0040e180 · legions 0040e160 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### PHashTable<unsigned_long,gmRaceSexInfo>
- [-] `PHashTable<unsigned_long,gmRaceSexInfo>::StreamPack` · 0040e200 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### PHashTable<unsigned_long,unsigned_long>
- [-] `PHashTable<unsigned_long,unsigned_long>::PHashTable<unsigned_long,unsigned_long>` · 0040e3f0 · legions 0040e3f0 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### gmAvatarSelectionUI
- [~] `gmAvatarSelectionUI::InitEntities` · 0040e510 · legions 0040e510 — avatar_selection.rs avatars + ac2-client pregame_scene/content.rs figures (makeObject, applied appearance, held objects, land location, line-up mode); the child-entity bookkeeping (_childArray) is the scene
- [x] `gmAvatarSelectionUI::InitGenders` · 0040eaa0 · legions 0040ea70 — ac2-ui pregame/avatar_selection.rs
- [x] `gmAvatarSelectionUI::InitRaces` · 0040ebf0 · legions 0040ebc0 — ac2-ui pregame/avatar_selection.rs

### PHashTable<unsigned_long,BaseList<unsigned_long,11>_>
- [-] `PHashTable<unsigned_long,BaseList<unsigned_long,11>_>::StreamPack` · 0040ecc0 · legions 0040ec90 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### gmAvatarSelectionUI
- [x] `gmAvatarSelectionUI::gmAvatarSelectionUI` · 0040f020 — ac2-ui pregame/avatar_selection.rs (pick, selected-animation protocol, primSong music) with ac2-client pregame_scene/ (line-up, mouse-over mode, selected behavior 0x41000005)
- [x] `gmAvatarSelectionUI::~gmAvatarSelectionUI` · 0040f270 · legions 0040f240 — ac2-ui pregame/avatar_selection.rs
- [x] `gmAvatarSelectionUI::Create` · 0040f500 — ac2-ui pregame/avatar_selection.rs
- [x] `gmAvatarSelectionUI::Register` · 0040f540 · legions 0040f510 — ac2-ui pregame/avatar_selection.rs

## `ac2/client/src/ui/pregame/generation/gmgenerationconfirmationui.cpp.c` (5/6)

### (free functions)
- [~] `NameInputFilter` · 0042cb60 — pregame/confirmation.rs is_name_character: English letters, ' - and space; other languages' letters not modelled

### gmGenerationConfirmationUI
- [x] `gmGenerationConfirmationUI::gmGenerationConfirmationUI` · 0042cb90 · legions 0042c950 — ac2-ui pregame/confirmation.rs (Legions: expansion/drudge entitlement checks, nonadminclient buttons)
- [x] `gmGenerationConfirmationUI::GhostButtons` · 0042cc90 · legions 0042ca50 — ac2-ui pregame/confirmation.rs (Legions: expansion/drudge entitlement checks, nonadminclient buttons)
- [x] `gmGenerationConfirmationUI::SetErrorMsg` · 0042cda0 · legions 0042cb60 — ac2-ui pregame/confirmation.rs (Legions: expansion/drudge entitlement checks, nonadminclient buttons)
- [x] `gmGenerationConfirmationUI::ListenToMessage` · 0042cde0 · legions 0042cba0 — ac2-ui pregame/confirmation.rs (Legions: expansion/drudge entitlement checks, nonadminclient buttons)
- [x] `gmGenerationConfirmationUI::RecvNotice_CharGenVerification` · 0042d080 — ac2-ui pregame/confirmation.rs (Legions: expansion/drudge entitlement checks, nonadminclient buttons)

## `ac2/client/src/ui/pregame/gmcharacterselectionui.cpp.c` (20/23)

### CharacterIdentity
- [x] `CharacterIdentity::Invalid` · 0040f560 · legions 0040f530 — ac2-ui pregame/character_selection.rs, pregame/arms.rs

### gmCharacterSelectionUI
- [x] `gmCharacterSelectionUI::GhostButtons` · 0040f590 · legions 0040f560 — ac2-ui pregame/character_selection.rs, pregame/arms.rs
- [x] `gmCharacterSelectionUI::Start3DSelection` · 0040f6c0 · legions 0040f690 — pregame 3D scenes: ac2-assets ac2_ui_scene.rs, ac2-client pregame_scene/, ac2-ui pregame/scene.rs
- [x] `gmCharacterSelectionUI::DoDelete` · 0040f720 · legions 0040f6f0 — ac2-ui pregame/character_selection.rs, pregame/arms.rs
- [x] `gmCharacterSelectionUI::MakePleaseWaitDialog` · 0040f810 · legions 0040f7e0 — ac2-ui pregame/character_selection.rs, pregame/arms.rs

### UIElement_ListBox
- [x] `UIElement_ListBox::GetSelectedIndex` · 0040f880 · legions 0040f850 — ac2-ui list_box_select.rs

### gmCharacterSelectionUI
- [x] `gmCharacterSelectionUI::MoveArm` · 0040f8c0 · legions 0040f890 — ac2-ui pregame/character_selection.rs, pregame/arms.rs
- [x] `gmCharacterSelectionUI::UnParentandDeleteArms` · 0040fa40 · legions 0040fa10 — ac2-ui pregame/character_selection.rs, pregame/arms.rs
- [x] `gmCharacterSelectionUI::RecvNotice_CloseConfirmationDialog` · 0040fae0 — ac2-ui pregame/character_selection.rs, pregame/arms.rs
- [x] `gmCharacterSelectionUI::SetErrorMsg` · 0040fba0 · legions 0040fb70 — ac2-ui pregame/character_selection.rs, pregame/arms.rs
- [x] `gmCharacterSelectionUI::MoveArms` · 0040fbe0 · legions 0040fbb0 — ac2-ui pregame/character_selection.rs, pregame/arms.rs
- [x] `gmCharacterSelectionUI::RecvNotice_CharGenVerification` · 0040fca0 — ac2-ui pregame/character_selection.rs, pregame/arms.rs
- [x] `gmCharacterSelectionUI::MakeDeleteDialog` · 0040ff20 · legions 0040fef0 — ac2-ui pregame/character_selection.rs, pregame/arms.rs
- [~] `gmCharacterSelectionUI::ListenToMessage` · 0040ffb0 · legions 0040ffd0 — character_selection.rs: all buttons, list, scene presses; the name's ellipsis (TruncateText on download_done) not modelled
- [x] `gmCharacterSelectionUI::InitArms` · 00410510 · legions 00410530 — ac2-ui pregame/character_selection.rs, pregame/arms.rs
- [x] `gmCharacterSelectionUI::~gmCharacterSelectionUI` · 00410750 · legions 00410770 — ac2-ui pregame/character_selection.rs, pregame/arms.rs
- [x] `gmCharacterSelectionUI::AddPhysObjToArm` · 004108e0 · legions 004108e0 — pregame 3D scenes: ac2-assets ac2_ui_scene.rs, ac2-client pregame_scene/, ac2-ui pregame/scene.rs
- [x] `gmCharacterSelectionUI::Finish3DSelection` · 00410a10 · legions 00410a10 — pregame 3D scenes: ac2-assets ac2_ui_scene.rs, ac2-client pregame_scene/, ac2-ui pregame/scene.rs
- [~] `gmCharacterSelectionUI::ListenToMessage` · 00410ad0 · legions 00410ad0 — character_selection.rs: all buttons, list, scene presses; the name's ellipsis (TruncateText on download_done) not modelled
- [x] `gmCharacterSelectionUI::Update` · 00410c00 · legions 00410c00 — ac2-ui pregame/character_selection.rs, pregame/arms.rs
- [~] `gmCharacterSelectionUI::gmCharacterSelectionUI` · 00410f00 — character_selection.rs: roots, handlers, Update; the background's music is requested when the screen opens (PregameRequest::Music, Channel_Main, the previous screen's fading over 2 s); the 3D scene itself is the client's
- [x] `gmCharacterSelectionUI::Create` · 00411220 — ac2-ui pregame/character_selection.rs, pregame/arms.rs
- [x] `gmCharacterSelectionUI::Register` · 00411240 · legions 00411220 — ac2-ui pregame/character_selection.rs, pregame/arms.rs

## `ac2/client/src/ui/pregame/gmcreditsui.cpp.c` (9/15)

### gmCreditsUI
- [x] `gmCreditsUI::MakePleaseWaitDialog` · 00412900 · legions 00412780 — ac2-ui pregame/{intro,data_patch,credits,keyboard}.rs
- [-] `gmCreditsUI::~gmCreditsUI` · 00412950 · legions 004127d0 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own
- [x] `gmCreditsUI::GetX` · 004129c0 · legions 00412840 — ac2-ui pregame/{intro,data_patch,credits,keyboard}.rs
- [x] `gmCreditsUI::GetY` · 004129f0 · legions 00412870 — ac2-ui pregame/{intro,data_patch,credits,keyboard}.rs
- [x] `gmCreditsUI::UpdatePicture` · 00412a20 · legions 004128a0 — ac2-ui pregame/{intro,data_patch,credits,keyboard}.rs
- [x] `gmCreditsUI::Close` · 00412b10 · legions 00412990 — ac2-ui pregame/{intro,data_patch,credits,keyboard}.rs
- [x] `gmCreditsUI::GetSoundID` · 00412b90 · legions 00412a10 — ac2-ui pregame/credits.rs sound_id (ID_Sound<n> on the PictureField, restarting from 0)
- [~] `gmCreditsUI::UpdateSound` · 00412c50 · legions 00412ad0 — ac2-ui pregame/credits.rs update_sound + ac2-client play_credit_sound/credit_sound_busy: the next sound plays once the last has finished; the sound context is kept by the panel, not in the element retail reads
- [x] `gmCreditsUI::UpdateText` · 00412d00 · legions 00412b80 — ac2-ui pregame/{intro,data_patch,credits,keyboard}.rs
- [x] `gmCreditsUI::ListenToMessage` · 00412e30 · legions 00412cb0 — ac2-ui pregame/{intro,data_patch,credits,keyboard}.rs
- [x] `gmCreditsUI::Update` · 00412e60 · legions 00412ce0 — ac2-ui pregame/{intro,data_patch,credits,keyboard}.rs
- [~] `gmCreditsUI::Initialize` · 00412e80 · legions 00412d00 — pregame screens: no intro/credits music, alt credits fields, data patch progress or admin start char; controls list and editing not modelled
- [~] `gmCreditsUI::gmCreditsUI` · 00413360 — pregame screens: no intro/credits music, alt credits fields, data patch progress or admin start char; controls list and editing not modelled
- [-] `gmCreditsUI::Create` · 004134b0 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own
- [-] `gmCreditsUI::Register` · 004134d0 · legions 00413350 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own

## `ac2/client/src/ui/pregame/gmdatapatchui.cpp.c` (5/19)

### gmDataPatchUI
- [x] `gmDataPatchUI::ListenToMessage` · 00411260 · legions 00411240 — ac2-ui pregame/{intro,data_patch,credits,keyboard}.rs
- [-] `gmDataPatchUI::Refresh` · 004112a0 · legions 00411280 — no DAT patching in this client (DDD patch events never come)
- [-] `gmDataPatchUI::OnHeartbeat` · 004113b0 — no DAT patching in this client (DDD patch events never come)

### CPluginManager
- [ ] `CPluginManager::RemovePluginPrototype` · 00411500 · legions 004114e0 — needs port: CPluginManager::RemovePluginPrototype behaviour

### gmClient
- [ ] `gmClient::SetStartChar` · 00411550 · legions 00411530 — needs port (check): gmClient::SetStartChar behaviour; implemented area ac2-client but uncited
- [ ] `gmClient::SetCreateChar` · 004115a0 · legions 00411580 — needs port (check): gmClient::SetCreateChar behaviour; implemented area ac2-client but uncited

### gmDataPatchUI
- [x] `gmDataPatchUI::SetErrorMsg` · 004115f0 · legions 004115d0 — ac2-ui pregame/{intro,data_patch,credits,keyboard}.rs
- [~] `gmDataPatchUI::OnConnectFailed` · 00411630 · legions 00411610 — the error text selection by NetError (ID_DataPatch_ConnectFailed variants, GUN world name, crypto error) is not ported: ac2-ui pregame/data_patch.rs only sets ID_DataPatch_ConnectingGUN / Connected; login failures are shown by the login screen instead
- [x] `gmDataPatchUI::Connect` · 00411890 · legions 00411870 — ac2-ui pregame/{intro,data_patch,credits,keyboard}.rs

### (free functions)
- [-] `operator+` · 00411950 · legions 00411930 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### gmDataPatchUI
- [-] `gmDataPatchUI::~gmDataPatchUI` · 004119e0 · legions 004119c0 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own
- [~] `gmDataPatchUI::Update` · 00411ac0 · legions 00411aa0 — ac2-ui PregameUi::character_set (first set -> character selection) and bin/ac2 --character (start character enters at once); patching not ported
- [x] `gmDataPatchUI::RecvNotice_CharGenVerification` · 00411f10 — ac2-ui pregame/data_patch.rs char_gen_verification (enter at once, else character selection with no text)
- [-] `gmDataPatchUI::OnDDDEvent` · 00411fd0 — no DAT patching in this client (DDD patch events never come)
- [x] `gmDataPatchUI::gmDataPatchUI` · 004121d0 · legions 00412050 — ac2-ui pregame/{intro,data_patch,credits,keyboard}.rs
- [-] `gmDataPatchUI::Create` · 00412330 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own

### CPluginManager
- [ ] `CPluginManager::AddPluginPrototype` · 00412350 · legions 004121d0 — needs port: CPluginManager::AddPluginPrototype behaviour

### gmDataPatchUI
- [-] `gmDataPatchUI::Register` · 004123b0 · legions 00412230 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own
- [~] `gmDataPatchUI::OnNetStatusChange` · 004123d0 — pregame screens: no intro/credits music, alt credits fields, data patch progress or admin start char; controls list and editing not modelled

## `ac2/client/src/ui/pregame/gmdisconnectedui.cpp.c` (5/5)

### gmDisconnectedUI
- [x] `gmDisconnectedUI::gmDisconnectedUI` · 004125c0 · legions 00412440 — ac2-ui pregame/disconnected.rs
- [x] `gmDisconnectedUI::Create` · 00412630 — ac2-ui pregame/disconnected.rs
- [x] `gmDisconnectedUI::ListenToMessage` · 00412650 · legions 004124d0 — ac2-ui pregame/disconnected.rs
- [x] `gmDisconnectedUI::Register` · 004126b0 · legions 00412530 — ac2-ui pregame/disconnected.rs
- [x] `gmDisconnectedUI::SetErrorMsg` · 004126d0 · legions 00412550 — ac2-ui pregame/disconnected.rs

## `ac2/client/src/ui/pregame/gmintroui.cpp.c` (4/6)

### gmIntroUI
- [x] `gmIntroUI::gmIntroUI` · 00412710 — ac2-ui pregame/intro.rs: IntroField's music is requested when the screen opens (PregameRequest::Music)
- [-] `gmIntroUI::Create` · 00412830 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own
- [-] `gmIntroUI::Register` · 00412850 · legions 004126d0 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own
- [x] `gmIntroUI::Close` · 00412870 · legions 004126f0 — ac2-ui pregame/{intro,data_patch,credits,keyboard}.rs
- [x] `gmIntroUI::ListenToMessage` · 004128c0 · legions 00412740 — ac2-ui pregame/{intro,data_patch,credits,keyboard}.rs
- [x] `gmIntroUI::ListenToMessage` · 004128e0 · legions 00412760 — ac2-ui pregame/{intro,data_patch,credits,keyboard}.rs

## `ac2/client/src/ui/pregame/gmkeyboardui.cpp.c` (10/19)

### gmKeyboardUI
- [x] `gmKeyboardUI::ConfirmationBoxesOpen` · 004134f0 · legions 00413370 — ac2-ui pregame/{intro,data_patch,credits,keyboard}.rs
- [x] `gmKeyboardUI::UpdateButtons` · 00413530 · legions 004133b0 — ac2-ui pregame/{intro,data_patch,credits,keyboard}.rs
- [x] `gmKeyboardUI::DoClearAll` · 004135c0 · legions 00413440 — ac2-ui pregame/keyboard.rs KeyboardUi::dialog_outcome: the clear-all question's yes clears every slot (keyboard_list clear_all)
- [x] `gmKeyboardUI::CloseAllDialogs` · 004136a0 · legions 00413520 — ac2-ui pregame/keyboard.rs KeyboardUi::destroy closes the clear-all, already-mapped, restore, undo, exit and wait questions
- [x] `gmKeyboardUI::~gmKeyboardUI` · 00413720 · legions 004135a0 — ac2-ui pregame/keyboard.rs KeyboardUi::destroy: questions closed, root removed (rows go with it); notice handlers are per object in Rust
- [~] `gmKeyboardUI::InqKeyStringInfo` · 004137d0 · legions 00413650 — Key text is ac2-ui pregame/keyboard.rs binding_text, rendered from ID_Keyboard_key / ID_Keyboard_key_meta (keys key, key2) in the UI string table. Key names are key_names.rs: DirectInput's GetObjectInfo tszName stand-in, an unverified US table; no name gives no text; the retail names are not copied

### IntrusiveSmartPointer<PFileNode>
- [-] `IntrusiveSmartPointer<PFileNode>::~IntrusiveSmartPointer<PFileNode>` · 00413930 · legions 004137b0 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them
- [-] `IntrusiveSmartPointer<PFileNode>::create` · 00413960 · legions 004137e0 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### gmKeyboardUI
- [x] `gmKeyboardUI::Close` · 004139d0 · legions 00413850 — ac2-ui pregame/{intro,data_patch,credits,keyboard}.rs
- [x] `gmKeyboardUI::FinishSet` · 00413a70 · legions 004138f0 — ac2-ui pregame/keyboard_list.rs finish_set: the selection takes the key, the occupied slot clears; keyboard.rs rebuilds the texts
- [~] `gmKeyboardUI::StartSet` · 00413d10 · legions 00413b50 — ac2-ui pregame/keyboard_list.rs start_set: the already-mapped conflict on another slot, a free key is set. Not: the second pass over actions without a row (ICIDM 0x54)
- [~] `gmKeyboardUI::InitializeKeyboardList` · 00414140 · legions 00413f80 — ac2-ui pregame/keyboard_list.rs from_maps and keyboard.rs rebuild_rows: a row per action (action_box_item header, key_box_item slots); the key names from pregame/key_names.rs (authored DIK names). Not: overMapped flags; the map order assumed (main, shift, control)
- [~] `gmKeyboardUI::SaveKeyboardList` · 00414510 · legions 00414350 — ac2-ui pregame/keyboard.rs save: the three maps go to Main/Shift/Control.keymap in the user config dir (keymap_file::save_set) and to the input manager. Names: the DAT maps' binding names; an action no map names (5 of 115 described) is saved as its enum value 0x........, not its EnumMapper 0x4000001d name (not decoded)
- [x] `gmKeyboardUI::RecvNotice_CloseConfirmationDialog` · 004149f0 — ac2-ui pregame/keyboard.rs dialog_outcome: already-mapped (yes finishes), clear all, restore (keyboard_maps presets, sent as PregameRequest::KeyMaps), undo (rebuilt from the active maps), exit (close). The questions' wording is authored here
- [~] `gmKeyboardUI::gmKeyboardUI` · 00414bb0 · legions 004149f0 — ac2-ui pregame/keyboard.rs KeyboardUi::create and keyboard_maps.rs ActiveKeyMaps: the field, the buttons, the list. Not: the GlobalEventHandler notice 0x436 (no notice table), CKeyMap::LoadSemantics (names are authored)
- [x] `gmKeyboardUI::Create` · 00414e50 — ac2-ui pregame/keyboard.rs KeyboardUi::create: the field, the buttons, the list from the input manager's maps (keyboard_maps list_of), UpdateButtons; the row templates action_box_item and key_box_item resolved from the layout
- [~] `gmKeyboardUI::ListenToMessage` · 00414e70 · legions 00414c60 — ac2-ui pregame/keyboard.rs message and dialog_outcome: the list selection (header to first key), clear, save, undo, quit, clear all and the restore presets, each question gated while one is up. Not: the ~mouse wait flag, the restore question's ICIDM bindings (the presets are the library's)
- [~] `gmKeyboardUI::KeyHitHandler` · 00415140 · legions 00414f30 — ac2-ui pregame/keyboard.rs key_hit with keyboard_list key_hit: the cancel key, dialog and mouse gates, then StartSet; the key presses come from the input manager (InputManager::take_key_presses). Not: the ICIDM reserved-key check (none reserved), the name-length check
- [x] `gmKeyboardUI::Register` · 004151f0 · legions 00414fe0 — ac2-ui pregame/ui.rs use_new_mode: MODE_KEYBOARD makes the framework (the registration is the match on the mode)

## `ac2/client/src/ui/util/dialog/confirmationdialog.cpp.c` (3/3)

### ConfirmationDialog
- [x] `ConfirmationDialog::ListenToMessage` · 0042d7f0 · legions 0042d5c0 — ac2-ui dialog.rs
- [x] `ConfirmationDialog::SetText` · 0042d830 · legions 0042d600 — ac2-ui dialog.rs
- [x] `ConfirmationDialog::ConfirmationDialog` · 0042d850 · legions 0042d620 — ac2-ui dialog.rs

## `ac2/client/src/ui/util/dialog/dialogfactory.cpp.c` (3/4)

### DialogFactory
- [~] `DialogFactory::CreateDialogInternal` · 00415210 · legions 00415000 — dialog.rs: confirmation, wait, message; resurrect and timed dialogs not made
- [x] `DialogFactory::MakeDialog` · 004155b0 · legions 004153a0 — ac2-ui dialog.rs
- [x] `DialogFactory::DialogDone` · 00415740 · legions 00415530 — ac2-ui dialog.rs
- [x] `DialogFactory::CloseDialog` · 00415810 · legions 00415600 — ac2-ui dialog.rs

## `ac2/client/src/ui/util/dialog/messagedialog.cpp.c` (3/3)

### MessageDialog
- [x] `MessageDialog::ListenToMessage` · 0042d300 · legions 0042d0d0 — ac2-ui dialog.rs
- [x] `MessageDialog::SetText` · 0042d330 · legions 0042d100 — ac2-ui dialog.rs
- [x] `MessageDialog::MessageDialog` · 0042d350 · legions 0042d120 — ac2-ui dialog.rs

## `ac2/client/src/ui/util/dialog/resurrectdialog.cpp.c` (0/3)

### ResurrectDialog
- [ ] `ResurrectDialog::ListenToMessage` · 0042d3f0 · legions 0042d1c0 — needs port (check): ResurrectDialog::ListenToMessage behaviour; implemented area ac2-ui dialog.rs but uncited
- [ ] `ResurrectDialog::SetText` · 0042d420 · legions 0042d1f0 — needs port (check): ResurrectDialog::SetText behaviour; implemented area ac2-ui dialog.rs but uncited
- [-] `ResurrectDialog::ResurrectDialog` · 0042d440 · legions 0042d210 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own

## `ac2/client/src/ui/util/dialog/timeddialog.cpp.c` (0/5)

### TimedDialog
- [ ] `TimedDialog::ListenToMessage` · 0042d590 · legions 0042d360 — needs port (check): TimedDialog::ListenToMessage; hud_options.rs cite is a module doc
- [ ] `TimedDialog::ListenToMessage` · 0042d5d0 · legions 0042d3a0 — needs port (check): TimedDialog::ListenToMessage; hud_options.rs cite is a module doc
- [ ] `TimedDialog::SetDelay` · 0042d670 · legions 0042d440 — needs port (check): TimedDialog::SetDelay; dialog.rs TIMED_DELAY constant is cited, not the function
- [-] `TimedDialog::TimedDialog` · 0042d6e0 · legions 0042d4b0 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own
- [ ] `TimedDialog::SetText` · 0042d7b0 · legions 0042d580 — needs port (check): TimedDialog::SetText behaviour; implemented area ac2-ui dialog.rs but uncited

## `ac2/client/src/ui/util/dialog/waitdialog.cpp.c` (2/2)

### WaitDialog
- [x] `WaitDialog::SetText` · 0042d4e0 · legions 0042d2b0 — ac2-ui dialog.rs
- [x] `WaitDialog::WaitDialog` · 0042d500 · legions 0042d2d0 — ac2-ui dialog.rs

## `portal/client/src/cimelib/cime.cpp.c` (0/23)

### MultibyteProcessor
- [-] `MultibyteProcessor::MultibyteProcessor` · 004bd3b0 · legions 004b16d0 — Win32 IME plumbing (TSF/IMM32 language bar, caret, hot keys, composition drawing, ANSI codepage conversion). winit delivers the OS IME's committed text and the OS draws its own candidates; Rust strings are UTF-8 so no multibyte processor is needed
- [-] `MultibyteProcessor::UpdateInternals` · 004bd3c0 · legions 004b16e0 — Win32 IME plumbing (TSF/IMM32 language bar, caret, hot keys, composition drawing, ANSI codepage conversion). winit delivers the OS IME's committed text and the OS draws its own candidates; Rust strings are UTF-8 so no multibyte processor is needed
- [-] `MultibyteProcessor::GetUnicodeChar` · 004bd420 · legions 004b1740 — Win32 IME plumbing (TSF/IMM32 language bar, caret, hot keys, composition drawing, ANSI codepage conversion). winit delivers the OS IME's committed text and the OS draws its own candidates; Rust strings are UTF-8 so no multibyte processor is needed

### CIme
- [-] `CIme::DrawRect` · 004bd4a0 — Win32 IME plumbing (TSF/IMM32 language bar, caret, hot keys, composition drawing, ANSI codepage conversion). winit delivers the OS IME's committed text and the OS draws its own candidates; Rust strings are UTF-8 so no multibyte processor is needed
- [-] `CIme::SurfaceToClip` · 004bd4f0 · legions 004b1810 — Win32 IME plumbing (TSF/IMM32 language bar, caret, hot keys, composition drawing, ANSI codepage conversion). winit delivers the OS IME's committed text and the OS draws its own candidates; Rust strings are UTF-8 so no multibyte processor is needed
- [-] `CIme::DrawTriangle` · 004bd5a0 · legions 004b18c0 — Win32 IME plumbing (TSF/IMM32 language bar, caret, hot keys, composition drawing, ANSI codepage conversion). winit delivers the OS IME's committed text and the OS draws its own candidates; Rust strings are UTF-8 so no multibyte processor is needed
- [-] `CIme::DrawFans` · 004bd6a0 — Win32 IME plumbing (TSF/IMM32 language bar, caret, hot keys, composition drawing, ANSI codepage conversion). winit delivers the OS IME's committed text and the OS draws its own candidates; Rust strings are UTF-8 so no multibyte processor is needed

### CImeFontWrapper
- [-] `CImeFontWrapper::SetPosition` · 004bd740 · legions 004b1a60 — Win32 IME plumbing (TSF/IMM32 language bar, caret, hot keys, composition drawing, ANSI codepage conversion). winit delivers the OS IME's committed text and the OS draws its own candidates; Rust strings are UTF-8 so no multibyte processor is needed
- [-] `CImeFontWrapper::SetFont` · 004bd760 · legions 004b1a80 — Win32 IME plumbing (TSF/IMM32 language bar, caret, hot keys, composition drawing, ANSI codepage conversion). winit delivers the OS IME's committed text and the OS draws its own candidates; Rust strings are UTF-8 so no multibyte processor is needed

### CIme
- [~] `CIme::Deactivate` · 004bd790 · legions 004b1ab0 — ac2-ui input.rs typed_text + ac2-client bin/ac2/main.rs: winit Ime::Commit types into text elements (UTF-16 units); set_ime_allowed follows text mode (sync_ime). Not: the preedit composition is not drawn in the entry line; no candidate list (the OS draws it)
- [-] `CIme::Render` · 004bd810 · legions 004b1b30 — Win32 IME plumbing (TSF/IMM32 language bar, caret, hot keys, composition drawing, ANSI codepage conversion). winit delivers the OS IME's committed text and the OS draws its own candidates; Rust strings are UTF-8 so no multibyte processor is needed
- [-] `CIme::IsImeEnabled` · 004bd860 · legions 004b1b80 — Win32 IME plumbing (TSF/IMM32 language bar, caret, hot keys, composition drawing, ANSI codepage conversion). winit delivers the OS IME's committed text and the OS draws its own candidates; Rust strings are UTF-8 so no multibyte processor is needed
- [-] `CIme::IgnoreHotKey` · 004bd870 · legions 004b1b90 — Win32 IME plumbing (TSF/IMM32 language bar, caret, hot keys, composition drawing, ANSI codepage conversion). winit delivers the OS IME's committed text and the OS draws its own candidates; Rust strings are UTF-8 so no multibyte processor is needed
- [~] `CIme::ProcessMessage` · 004bd880 — ac2-ui input.rs typed_text + ac2-client bin/ac2/main.rs: winit Ime::Commit types into text elements (UTF-16 units); set_ime_allowed follows text mode (sync_ime). Not: the preedit composition is not drawn in the entry line; no candidate list (the OS draws it)
- [-] `CIme::SetCaretPosition` · 004bd890 · legions 004b1bb0 — Win32 IME plumbing (TSF/IMM32 language bar, caret, hot keys, composition drawing, ANSI codepage conversion). winit delivers the OS IME's committed text and the OS draws its own candidates; Rust strings are UTF-8 so no multibyte processor is needed
- [-] `CIme::ToggleLanguageBar` · 004bd8e0 — Win32 IME plumbing (TSF/IMM32 language bar, caret, hot keys, composition drawing, ANSI codepage conversion). winit delivers the OS IME's committed text and the OS draws its own candidates; Rust strings are UTF-8 so no multibyte processor is needed
- [-] `CIme::IsSendingKeyMessage` · 004bd8f0 · legions 004b1c10 — Win32 IME plumbing (TSF/IMM32 language bar, caret, hot keys, composition drawing, ANSI codepage conversion). winit delivers the OS IME's committed text and the OS draws its own candidates; Rust strings are UTF-8 so no multibyte processor is needed
- [~] `CIme::Activate` · 004bd900 · legions 004b1c20 — ac2-ui input.rs typed_text + ac2-client bin/ac2/main.rs: winit Ime::Commit types into text elements (UTF-16 units); set_ime_allowed follows text mode (sync_ime). Not: the preedit composition is not drawn in the entry line; no candidate list (the OS draws it)
- [-] `CIme::SetAppearanceToDefault` · 004bdb70 · legions 004b1e90 — Win32 IME plumbing (TSF/IMM32 language bar, caret, hot keys, composition drawing, ANSI codepage conversion). winit delivers the OS IME's committed text and the OS draws its own candidates; Rust strings are UTF-8 so no multibyte processor is needed
- [-] `CIme::Init` · 004bdc70 · legions 004b1f90 — Win32 IME plumbing (TSF/IMM32 language bar, caret, hot keys, composition drawing, ANSI codepage conversion). winit delivers the OS IME's committed text and the OS draws its own candidates; Rust strings are UTF-8 so no multibyte processor is needed

### MultibyteProcessor
- [-] `MultibyteProcessor::GetUnicodeString` · 004bdd00 · legions 004b2020 — Win32 IME plumbing (TSF/IMM32 language bar, caret, hot keys, composition drawing, ANSI codepage conversion). winit delivers the OS IME's committed text and the OS draws its own candidates; Rust strings are UTF-8 so no multibyte processor is needed

### CImeFontWrapper
- [-] `CImeFontWrapper::GetTextExtent` · 004bddb0 · legions 004b20d0 — Win32 IME plumbing (TSF/IMM32 language bar, caret, hot keys, composition drawing, ANSI codepage conversion). winit delivers the OS IME's committed text and the OS draws its own candidates; Rust strings are UTF-8 so no multibyte processor is needed
- [-] `CImeFontWrapper::DrawTextA` · 004bde10 · legions 004b2130 — Win32 IME plumbing (TSF/IMM32 language bar, caret, hot keys, composition drawing, ANSI codepage conversion). winit delivers the OS IME's committed text and the OS draws its own candidates; Rust strings are UTF-8 so no multibyte processor is needed

## `portal/client/src/cinputmap/cinputmanager.cpp.c` (5/16)

### CInputManager
- [~] `CInputManager::CInputManager` · 00599540 · legions 005665e0 — ac2-ui input.rs InputManager::new and controls.rs Controls::load_with (preproc map from the DAT). Not: the DirectInput device list and the acquire/keystone state

### NIListIter<NoticeHandler_*>
- [-] `NIListIter<NoticeHandler_*>::RemoveCur` · 00599570 · legions 004c0420 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### ICIDM
- [ ] `ICIDM::PushKeyMap` · 005995e0 · legions 00566610 — needs port: key map action table (CKeyMap / ICIDM key map stack); keymap_file.rs
- [~] `ICIDM::GetNameFromKey` · 00599610 · legions 00566640 — ac2-ui key_names.rs key_name: authored names, the hex form for an unnamed semantic

### CInputManager
- [-] `CInputManager::~CInputManager` · 00599630 · legions 00566660 — Win32 / DirectInput / D3D plumbing (device poll, VK mapping, focus, hardware cursor, draw list, sound device) replaced by winit/wgpu/rodio
- [-] `CInputManager::AddHandlerToList` · 00599930 · legions 00566960 — handler registry lists: ac2-ui Controls dispatches to its owners directly (no notice-handler list)
- [-] `CInputManager::RemoveHandlerFromList` · 00599980 · legions 005669b0 — handler registry lists: ac2-ui Controls dispatches to its owners directly (no notice-handler list)
- [-] `CInputManager::RegisterInputHandler` · 005999d0 · legions 00566a00 — handler registry lists: ac2-ui Controls dispatches to its owners directly (no notice-handler list)
- [-] `CInputManager::UnregisterInputHandler` · 00599ae0 · legions 00566b10 — handler registry lists: ac2-ui Controls dispatches to its owners directly (no notice-handler list)
- [x] `CInputManager::CallCharacterHandler` · 00599b50 · legions 00566b80 — ac2-ui input.rs / text_edit.rs: typing into text elements
- [x] `CInputManager::CallActionHandler` · 00599b90 · legions 00566bc0 — ac2-ui input.rs
- [x] `CInputManager::CallMouseLookHandler` · 00599c00 · legions 00566c30 — ac2-ui input.rs
- [x] `CInputManager::CallKeyHitHandler` · 00599c40 · legions 00566c70 — ac2-ui controls.rs take_key_presses -> gmKeyboardUI::KeyHitHandler (InputManager::take_key_presses)
- [-] `CInputManager::CallFocusSwitchHandler` · 00599c80 · legions 00566cb0 — Win32 / DirectInput / D3D plumbing (device poll, VK mapping, focus, hardware cursor, draw list, sound device) replaced by winit/wgpu/rodio
- [~] `CInputManager::SetMouseLookMode` · 00599ce0 · legions 00566d10 — ac2-ui input.rs: keys, buttons, wheel, axes, cursor; no text/keystone mode, no device acquisition
- [x] `CInputManager::SetTextMode` · 00599d40 · legions 00566d70 — ac2-ui input.rs / text_edit.rs: typing into text elements

## `portal/client/src/cinputmap/ckeymap.cpp.c` (7/23)

### MapGUID
- [x] `MapGUID::StreamPack` · 00599d60 · legions 00566d90 — ac2-ui maps.rs

### HashIterator<unsigned_long,unsigned_long>
- [-] `HashIterator<unsigned_long,unsigned_long>::HashIterator<unsigned_long,unsigned_long>` · 00599e10 · legions 004b66a0 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### PSUtils
- [x] `PSUtils::is_uint32` · 00599e30 · legions 00566e40 — ac2-ui stack.rs

### (free functions)
- [-] `compute_hash` · 00599ea0 · legions 00566eb0 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### ActionMapping
- [-] `ActionMapping::operator=` · 00599f10 · legions 00566f20 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs

### CKeyMap
- [~] `CKeyMap::GetNameBySemantic` · 0059a000 · legions 00566fc0 — ac2-ui key_names.rs (key_name, semantic_from_name): authored DIK names, not the retail LoadSemantics strings (not copied); UpdateVersion's table upgrade is not ported (keymap_file.rs reads the names)

### ActionMapping
- [x] `ActionMapping::StreamPack` · 0059a1c0 · legions 00567180 — ac2-ui maps.rs

### CKeyMap
- [ ] `CKeyMap::AddAction` · 0059a210 · legions 005671d0 — needs port: key map action table (CKeyMap / ICIDM key map stack); keymap_file.rs
- [x] `CKeyMap::StreamPack` · 0059a2b0 · legions 00567270 — ac2-ui maps.rs
- [~] `CKeyMap::GetSemanticByName` · 0059a320 · legions 005672e0 — ac2-ui key_names.rs (key_name, semantic_from_name): authored DIK names, not the retail LoadSemantics strings (not copied); UpdateVersion's table upgrade is not ported (keymap_file.rs reads the names)
- [x] `CKeyMap::UpdateVersion` · 0059a360 · legions 00567320 — ac2-ui keymap_file.rs update_version + MAIN_KEY_UPDATES (Main table row 0x9d73ac); read_key_map_upgraded rewrites the file (keymap_file_tests)
- [-] `CKeyMap::Destroy` · 0059a680 — DBObj lifecycle / copy plumbing (key map object); Rust keymap struct owns it
- [-] `CKeyMap::LoadSemantics` · 0059a690 · legions 00567650 — CKeyMap::LoadSemantics: the retail key-name strings are game data; this port authors its own names (ac2-ui key_names.rs)

### PList<TurbineUserStatus>
- [-] `PList<TurbineUserStatus>::Pack` · 0059d4a0 · legions 0056a460 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### PList<ActionMapping>
- [-] `PList<ActionMapping>::UnPack` · 0059d4f0 · legions 0056a4c4 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### CKeyMap
- [-] `CKeyMap::CKeyMap` · 0059d680 · legions 0056a640 — DBObj lifecycle / copy plumbing (key map object); Rust keymap struct owns it
- [-] `CKeyMap::GetDBOType` · 0059d710 — DBObj lifecycle / copy plumbing (key map object); Rust keymap struct owns it
- [-] `CKeyMap::~CKeyMap` · 0059d720 · legions 0056a6e0 — DBObj lifecycle / copy plumbing (key map object); Rust keymap struct owns it
- [x] `CKeyMap::FromFileNode` · 0059d820 · legions 0056a7e0 — ac2-ui keymap_file.rs key_map_tree / key_map_from_tree: the .keymap text tree (PFileParser name [ children ]) written by SaveKeyboardList and read by PushKeyMap, the VersionHeader and GUID layout; round-trip tests and real-data round trip
- [x] `CKeyMap::ToFileNode` · 0059de20 · legions 0056ade0 — ac2-ui keymap_file.rs key_map_tree / key_map_from_tree: the .keymap text tree (PFileParser name [ children ]) written by SaveKeyboardList and read by PushKeyMap, the VersionHeader and GUID layout; round-trip tests and real-data round trip
- [-] `CKeyMap::CopyInto` · 0059e140 · legions 0056b100 — DBObj lifecycle / copy plumbing (key map object); Rust keymap struct owns it
- [-] `CKeyMap::Allocate` · 0059e210 — DBObj lifecycle / copy plumbing (key map object); Rust keymap struct owns it
- [-] `CKeyMap::FromFileNode` · 0059e240 — text source format (admin tools)

## `portal/client/src/cinputmap/directx/cinputmanager_win32.cpp.c` (11/35)

### CInputManager_WIN32
- [x] `CInputManager_WIN32::IsMetaKey` · 0059e260 · legions 0056b200 — ac2-ui input.rs
- [x] `CInputManager_WIN32::MetaModeFromKey` · 0059e320 · legions 0056b2c0 — ac2-ui input.rs
- [-] `CInputManager_WIN32::PollDevice` · 0059e400 · legions 0056b3a0 — Win32 / DirectInput / D3D plumbing (device poll, VK mapping, focus, hardware cursor, draw list, sound device) replaced by winit/wgpu/rodio
- [x] `CInputManager_WIN32::IsUIAction` · 0059e470 · legions 0056b410 — ac2-ui input.rs / text_edit.rs: typing into text elements
- [x] `CInputManager_WIN32::UpdateCharacter` · 0059e500 · legions 0056b4a0 — ac2-ui input.rs / text_edit.rs: typing into text elements
- [-] `CInputManager_WIN32::OurMapVirtualKey` · 0059e520 · legions 0056b4c0 — Win32 / DirectInput / D3D plumbing (device poll, VK mapping, focus, hardware cursor, draw list, sound device) replaced by winit/wgpu/rodio

### _STL::_Rb_tree_iterator<_STL::pair<ActionWithModifiers_const_,ActionMapValue>,_STL::_Nonconst_traits<_STL::pair<ActionWithModifiers_const_,ActionMapValue>_>_>
- [-] `_STL::_Rb_tree_iterator<_STL::pair<ActionWithModifiers_const_,ActionMapValue>,_STL::_Nonconst_traits<_STL::pair<ActionWithModifiers_const_,ActionMapValue>_>_>::operator++` · 0059e680 · legions 004c7b10 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### CInputManager_WIN32
- [-] `CInputManager_WIN32::AcquireAll` · 0059e6c0 · legions 0056b590 — DirectInput device management (AcquireAll, UnacquireAll, ReleaseDevices, SwitchMouseMode, Cleanup, AddDevice, AddToEnd): winit delivers window events (ac2-client bin/ac2/main.rs)
- [-] `CInputManager_WIN32::UnacquireAll` · 0059e700 · legions 0056b5d0 — DirectInput device management (AcquireAll, UnacquireAll, ReleaseDevices, SwitchMouseMode, Cleanup, AddDevice, AddToEnd): winit delivers window events (ac2-client bin/ac2/main.rs)
- [-] `CInputManager_WIN32::ReleaseDevices` · 0059e730 · legions 0056b600 — DirectInput device management (AcquireAll, UnacquireAll, ReleaseDevices, SwitchMouseMode, Cleanup, AddDevice, AddToEnd): winit delivers window events (ac2-client bin/ac2/main.rs)
- [-] `CInputManager_WIN32::SwitchMouseMode` · 0059e770 · legions 0056b640 — DirectInput device management (AcquireAll, UnacquireAll, ReleaseDevices, SwitchMouseMode, Cleanup, AddDevice, AddToEnd): winit delivers window events (ac2-client bin/ac2/main.rs)
- [ ] `CInputManager_WIN32::SetFocus` · 0059e850 · legions 0056b720 — needs port (check): key action lookup in the input manager; Win32 focus plumbing is replaced by winit
- [~] `CInputManager_WIN32::GetNameFromKey` · 0059e970 · legions 0056b890 — ac2-ui pregame/key_names.rs key_name: stands in for DirectInput's GetObjectInfo(DIK).tszName on macOS/Linux, an unverified US table of scan-code names; a key with no name gives no text (the Key 0x hex form is kept only for keymap files); the retail names are not copied

### SmartArray<CInputManager_WIN32::InputDevice>
- [-] `SmartArray<CInputManager_WIN32::InputDevice>::AddToEnd` · 0059eb80 · legions 0056ba40 — DirectInput device management (AcquireAll, UnacquireAll, ReleaseDevices, SwitchMouseMode, Cleanup, AddDevice, AddToEnd): winit delivers window events (ac2-client bin/ac2/main.rs)

### CInputManager_WIN32
- [-] `CInputManager_WIN32::Cleanup` · 0059ebe0 · legions 0056baa0 — DirectInput device management (AcquireAll, UnacquireAll, ReleaseDevices, SwitchMouseMode, Cleanup, AddDevice, AddToEnd): winit delivers window events (ac2-client bin/ac2/main.rs)
- [-] `CInputManager_WIN32::AddDevice` · 0059ec20 · legions 0056bae0 — DirectInput device management (AcquireAll, UnacquireAll, ReleaseDevices, SwitchMouseMode, Cleanup, AddDevice, AddToEnd): winit delivers window events (ac2-client bin/ac2/main.rs)
- [-] `CInputManager_WIN32::EnumSuitableDevicesCB` · 0059ee30 — Win32 / DirectInput / D3D plumbing (device poll, VK mapping, focus, hardware cursor, draw list, sound device) replaced by winit/wgpu/rodio

### HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>
- [-] `HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>::operator=` · 0059ef60 · legions 0056bed0 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### CInputManager_WIN32
- [~] `CInputManager_WIN32::Init` · 0059f210 · legions 0056bfc0 — ac2-ui input.rs InputManager::new and controls.rs Controls::load_with (preproc map from the DAT). Not: the DirectInput device list and the acquire/keystone state
- [x] `CInputManager_WIN32::GetInputActionFromKey` · 0059f5d0 · legions 0056c380 — ac2-ui input.rs
- [x] `CInputManager_WIN32::ReleaseKey` · 0059f690 · legions 0056c440 — ac2-ui input.rs
- [~] `CInputManager_WIN32::SwitchTextMode` · 0059f750 · legions 0056c500 — ac2-ui text_edit.rs: no selection, clipboard, IME, page up/down, cursor flashing or view scrolling
- [x] `CInputManager_WIN32::SetKey` · 0059f980 · legions 0056c730 — ac2-ui input.rs
- [~] `CInputManager_WIN32::UpdateVirtualKey` · 0059fa30 · legions 0056c7e0 — ac2-ui text_edit.rs: no selection, clipboard, IME, page up/down, cursor flashing or view scrolling

### _STL::deque<HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>,_STL::allocator<HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>_>_>
- [-] `_STL::deque<HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>,_STL::allocator<HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>_>_>::~deque<HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>,_STL::allocator<HashTable<CInputManager_WIN32::QualifiedKey,unsigned_long>_>_>` · 0059fbf0 · legions 00645850 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### CInputManager_WIN32
- [x] `CInputManager_WIN32::PopKeyMap` · 0059ff00 · legions 0056cc00 — ac2-ui input.rs
- [ ] `CInputManager_WIN32::GetKeyFromInputAction` · 0059ff60 · legions 0056cc60 — needs port (check): key action lookup in the input manager; Win32 focus plumbing is replaced by winit
- [-] `CInputManager_WIN32::~CInputManager_WIN32` · 005a01f0 · legions 0056cef0 — Win32 / DirectInput / D3D plumbing (device poll, VK mapping, focus, hardware cursor, draw list, sound device) replaced by winit/wgpu/rodio
- [-] `CInputManager_WIN32::CInputManager_WIN32` · 005a03a0 · legions 0056d0a0 — Win32 / DirectInput / D3D plumbing (device poll, VK mapping, focus, hardware cursor, draw list, sound device) replaced by winit/wgpu/rodio
- [x] `CInputManager_WIN32::CheckDoubleClick` · 005a04a0 · legions 0056d1a0 — ac2-ui input.rs
- [~] `CInputManager_WIN32::ProcessDeviceData` · 005a05e0 · legions 0056d2e0 — ac2-ui input.rs: keys, buttons, wheel, axes, cursor; no text/keystone mode, no device acquisition
- [x] `CInputManager_WIN32::PushKeyMap_Internal` · 005a09b0 · legions 0056d6b0 — ac2-ui input.rs
- [~] `CInputManager_WIN32::GetInput` · 005a0ac0 · legions 0056d7c0 — ac2-ui input.rs: keys, buttons, wheel, axes, cursor; no text/keystone mode, no device acquisition
- [~] `CInputManager_WIN32::PushKeyMap` · 005a0da0 · legions 0056daa0 — ac2-ui controls.rs load_with + keymap_file read_key_map_upgraded: upgrade and rewrite done; retail's Destroy/ToFileNode/SaveToFile step is the same write_atomic
- [x] `CInputManager_WIN32::PushKeyMap` · 005a0f80 — ac2-ui input.rs

## `portal/client/src/cregion/cdetailmap/cdetailmapdesc.cpp.c` (3/22)

### MyRect
- [-] `MyRect::MyRect` · 0051c390 — ctor/dtor/operator= of plain data; Rust struct construction and Drop replace it
- [ ] `MyRect::Include` · 0051c3b0 — needs port: MyRect::Include behaviour
- [ ] `MyRect::Include` · 0051c400 — needs port: MyRect::Include behaviour

### CDetailMapDesc
- [x] `CDetailMapDesc::StreamPack` · 0051c4e0 · legions 00509c80 — ac2-world region/detail_map.rs: DBObj header, _blockMapDID, PSmartArray<CDetailMapRow>
- [ ] `CDetailMapDesc::GetSubDataIDs` · 0051c560 · legions 00509d00 — needs port: sub-object DIDs for DBOCache resolution (GetSubDataIDs); ac2-dat has none yet (AGENTS requires it)
- [-] `CDetailMapDesc::CDetailMapDesc` · 0051c6b0 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [-] `CDetailMapDesc::GetDBOType` · 0051c6e0 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs

### SmartArray<MyRect>
- [-] `SmartArray<MyRect>::operator=` · 0051c780 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### CDetailMapDesc
- [-] `CDetailMapDesc::Allocate` · 0051c7f0 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [-] `CDetailMapDesc::ToFileNode` · 0051c860 — text source format (admin tools)
- [-] `CDetailMapDesc::LoadMap` · 0051c970 — text source loader, admin build only

### CDetailMapRow
- [-] `CDetailMapRow::operator=` · 0051cab0 · legions 00509e00 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs

### MapInfo
- [-] `MapInfo::MapInfo` · 0051cb10 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [ ] `MapInfo::InqPString` · 0051cbe0 — needs port: MapInfo::InqPString behaviour

### CDetailMapDesc
- [x] `CDetailMapDesc::InqRowFromValue` · 0051cc70 · legions 00509e60 — ac2-world region/detail_map.rs (DetailMap::row_from_value / row_from_block, MapNoteRow) and ac2_region.rs; real-data tests in region/detail_map_tests.rs
- [x] `CDetailMapDesc::InqRowFromXY` · 0051ce30 · legions 00509fd0 — ac2-world region/detail_map.rs (DetailMap::row_from_value / row_from_block, MapNoteRow) and ac2_region.rs; real-data tests in region/detail_map_tests.rs

### PSmartArray<CDetailMapRow>
- [-] `PSmartArray<CDetailMapRow>::StreamPack` · 0051cef0 · legions 0050a040 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### SmartArray<CDetailMapRow>
- [-] `SmartArray<CDetailMapRow>::AddToEnd` · 0051d0b0 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### CDetailMapDesc
- [-] `CDetailMapDesc::CheckDupAdd` · 0051d100 — text source format (admin tools)
- [-] `CDetailMapDesc::LoadRow` · 0051d160 — text source loader, admin build only
- [-] `CDetailMapDesc::CheckAll` · 0051d1f0 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [-] `CDetailMapDesc::FromFileNode` · 0051d570 — text source format (admin tools)

## `portal/client/src/cregion/cdetailmap/cdetailmaprow.cpp.c` (1/7)

### CDetailMapRow
- [-] `CDetailMapRow::CheckXY` · 0051b8c0 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [-] `CDetailMapRow::Destroy` · 0051b900 · legions 00509a10 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [x] `CDetailMapRow::StreamPack` · 0051b930 · legions 00509a40 — ac2-world region/detail_map.rs (DetailMap::row_from_value / row_from_block, MapNoteRow) and ac2_region.rs; real-data tests in region/detail_map_tests.rs
- [ ] `CDetailMapRow::GetSubDataIDs` · 0051b9e0 · legions 00509af0 — needs port: sub-object DIDs for DBOCache resolution (GetSubDataIDs); ac2-dat has none yet (AGENTS requires it)
- [-] `CDetailMapRow::CDetailMapRow` · 0051ba20 · legions 00509b30 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [-] `CDetailMapRow::~CDetailMapRow` · 0051ba50 · legions 00509b60 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [-] `CDetailMapRow::FromFileNode` · 0051baa0 — text source format (admin tools)

## `portal/client/src/cregion/cmapnote/cmapnotedesc.cpp.c` (0/10)

### CDetailMapDesc
- [-] `CDetailMapDesc::Destroy` · 00528e60 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs

### CMapNoteDesc
- [ ] `CMapNoteDesc::GetSubDataIDs` · 00528e70 · legions 0050fcd0 — needs port: sub-object DIDs for DBOCache resolution (GetSubDataIDs); ac2-dat has none yet (AGENTS requires it)
- [-] `CMapNoteDesc::CMapNoteDesc` · 00528ec0 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [-] `CMapNoteDesc::GetDBOType` · 00528ef0 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [-] `CMapNoteDesc::Allocate` · 00528f00 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [-] `CMapNoteDesc::ToFileNode` · 00528f70 — text source format (admin tools)
- [-] `CMapNoteDesc::CheckStrings` · 00529080 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs

### SmartArray<CMapNoteRow>
- [-] `SmartArray<CMapNoteRow>::AddToEnd` · 005291b0 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### CMapNoteDesc
- [-] `CMapNoteDesc::LoadRow` · 00529210 — text source loader, admin build only
- [-] `CMapNoteDesc::FromFileNode` · 00529280 — text source format (admin tools)

## `portal/client/src/cregion/cmapnote/cmapnoterow.cpp.c` (1/4)

### CMapNoteRow
- [-] `CMapNoteRow::CMapNoteRow` · 0051bf70 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [-] `CMapNoteRow::~CMapNoteRow` · 0051bf90 · legions 00509bf0 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [x] `CMapNoteRow::StreamPack` · 0051bfb0 · legions 00509c10 — ac2-world region/detail_map.rs (DetailMap::row_from_value / row_from_block, MapNoteRow) and ac2_region.rs; real-data tests in region/detail_map_tests.rs
- [-] `CMapNoteRow::FromFileNode` · 0051c020 — text source format (admin tools)

## `portal/client/src/render/ui/renderui.cpp.c` (0/17)

### RenderUI
- [-] `RenderUI::CreateUISurfaceIndexBuffer` · 005c3d00 · legions 0058ae10 — Win32 / DirectInput / D3D plumbing (device poll, VK mapping, focus, hardware cursor, draw list, sound device) replaced by winit/wgpu/rodio
- [ ] `RenderUI::SetCursorProperties` · 005c3d80 · legions 0058ae90 — needs port (check): RenderUI::SetCursorProperties behaviour; implemented area ac2-ui media.rs but uncited
- [ ] `RenderUI::SetDefaultCursor` · 005c3f20 · legions 0058b030 — needs port (check): RenderUI::SetDefaultCursor behaviour; implemented area ac2-ui media.rs but uncited
- [-] `RenderUI::ShowCursor` · 005c3f80 · legions 0058b090 — Win32 / DirectInput / D3D plumbing (device poll, VK mapping, focus, hardware cursor, draw list, sound device) replaced by winit/wgpu/rodio
- [-] `RenderUI::LockList` · 005c3fc0 · legions 0058b0d0 — Win32 / DirectInput / D3D plumbing (device poll, VK mapping, focus, hardware cursor, draw list, sound device) replaced by winit/wgpu/rodio
- [-] `RenderUI::InitCursor` · 005c4000 · legions 0058b110 — Win32 / DirectInput / D3D plumbing (device poll, VK mapping, focus, hardware cursor, draw list, sound device) replaced by winit/wgpu/rodio
- [-] `RenderUI::SetCursorPosition` · 005c4160 · legions 0058b270 — Win32 / DirectInput / D3D plumbing (device poll, VK mapping, focus, hardware cursor, draw list, sound device) replaced by winit/wgpu/rodio
- [-] `RenderUI::Startup` · 005c42f0 · legions 0058b400 — Win32 / DirectInput / D3D plumbing (device poll, VK mapping, focus, hardware cursor, draw list, sound device) replaced by winit/wgpu/rodio
- [ ] `RenderUI::RenderObjects` · 005c4340 · legions 0058b430 — needs port (check): RenderUI::RenderObjects behaviour; implemented area ac2-ui media.rs but uncited
- [-] `RenderUI::GetLastObject` · 005c4460 · legions 0058b520 — Win32 / DirectInput / D3D plumbing (device poll, VK mapping, focus, hardware cursor, draw list, sound device) replaced by winit/wgpu/rodio
- [-] `RenderUI::GetPreviousObject` · 005c4480 · legions 0058b540 — Win32 / DirectInput / D3D plumbing (device poll, VK mapping, focus, hardware cursor, draw list, sound device) replaced by winit/wgpu/rodio
- [-] `RenderUI::LinkObject` · 005c4510 · legions 0058b560 — Win32 / DirectInput / D3D plumbing (device poll, VK mapping, focus, hardware cursor, draw list, sound device) replaced by winit/wgpu/rodio
- [-] `RenderUI::Shutdown` · 005c45d0 · legions 0058b660 — Win32 / DirectInput / D3D plumbing (device poll, VK mapping, focus, hardware cursor, draw list, sound device) replaced by winit/wgpu/rodio
- [-] `RenderUI::UnlinkObject` · 005c4660 · legions 0058b6e0 — Win32 / DirectInput / D3D plumbing (device poll, VK mapping, focus, hardware cursor, draw list, sound device) replaced by winit/wgpu/rodio
- [-] `RenderUI::CleanupObjectList` · 005c46e0 · legions 0058b760 — Win32 / DirectInput / D3D plumbing (device poll, VK mapping, focus, hardware cursor, draw list, sound device) replaced by winit/wgpu/rodio
- [ ] `RenderUI::SetObjectDepth` · 005c4750 · legions 0058b7d0 — needs port (check): RenderUI::SetObjectDepth behaviour; implemented area ac2-ui media.rs but uncited
- [-] `RenderUI::UnlockList` · 005c4780 · legions 0058b800 — Win32 / DirectInput / D3D plumbing (device poll, VK mapping, focus, hardware cursor, draw list, sound device) replaced by winit/wgpu/rodio

## `portal/client/src/render/ui/uinosurfaceobject.cpp.c` (0/9)

### UINoSurfaceObject
- [-] `UINoSurfaceObject::UINoSurfaceObject` · 00735650 · legions 006b1350 — D3D surface / vertex / matrix / material plumbing replaced by wgpu; the drawn result is ac2-gpu

### PHYSICSOBJREP
- [ ] `PHYSICSOBJREP::GetBehaviorTableID` · 00735680 · legions 006b1260 — needs port: PHYSICSOBJREP::GetBehaviorTableID behaviour

### UINoSurfaceObject
- [-] `UINoSurfaceObject::GetObjectYPos` · 00735690 · legions 006b1270 — UI render-surface plumbing is ac2-gpu ui.rs (UiOverlay layers placed by set_placements, resized with the window, drawn after the tone map) with ac2-client compose.rs; the D3D surface/material/ray-test object has no counterpart type

### CPhysicsObj
- [ ] `CPhysicsObj::get_cell_id` · 007356a0 · legions 006b1280 — needs port: CPhysicsObj::get_cell_id behaviour

### UINoSurfaceObject
- [-] `UINoSurfaceObject::GetObjectHeight` · 007356b0 · legions 006b0fd0 — UI render-surface plumbing is ac2-gpu ui.rs (UiOverlay layers placed by set_placements, resized with the window, drawn after the tone map) with ac2-client compose.rs; the D3D surface/material/ray-test object has no counterpart type

### Vector
- [-] `Vector::Vector` · 007356c0 · legions 006b0fe0 — ctor/dtor/operator= of plain data; Rust struct construction and Drop replace it

### UINoSurfaceObject
- [-] `UINoSurfaceObject::SetScreenPosition` · 007356d0 · legions 006b1380 — UI render-surface plumbing is ac2-gpu ui.rs (UiOverlay layers placed by set_placements, resized with the window, drawn after the tone map) with ac2-client compose.rs; the D3D surface/material/ray-test object has no counterpart type
- [-] `UINoSurfaceObject::SetScreenPosition` · 007356f0 · legions 006b13a0 — UI render-surface plumbing is ac2-gpu ui.rs (UiOverlay layers placed by set_placements, resized with the window, drawn after the tone map) with ac2-client compose.rs; the D3D surface/material/ray-test object has no counterpart type
- [-] `UINoSurfaceObject::ResizeSurface` · 00735710 · legions 006b13c0 — D3D surface / vertex / matrix / material plumbing replaced by wgpu; the drawn result is ac2-gpu

## `portal/client/src/render/ui/uiobject.cpp.c` (0/2)

### UIObject
- [-] `UIObject::UIObject` · 005d9370 · legions 0059b780 — D3D surface / vertex / matrix / material plumbing replaced by wgpu; the drawn result is ac2-gpu
- [-] `UIObject::~UIObject` · 005d93c0 · legions 0059b7d0 — D3D surface / vertex / matrix / material plumbing replaced by wgpu; the drawn result is ac2-gpu

## `portal/client/src/render/ui/uirectangleobject.cpp.c` (0/4)

### UIRectangleObject
- [-] `UIRectangleObject::UIRectangleObject` · 00735270 · legions 006b0f70 — D3D surface / vertex / matrix / material plumbing replaced by wgpu; the drawn result is ac2-gpu

### UINoSurfaceObject
- [-] `UINoSurfaceObject::RayTest` · 007352d0 · legions 006b0ff0 — UI render-surface plumbing is ac2-gpu ui.rs (UiOverlay layers placed by set_placements, resized with the window, drawn after the tone map) with ac2-client compose.rs; the D3D surface/material/ray-test object has no counterpart type
- [-] `UINoSurfaceObject::GetScreenLocalPosition` · 00735340 · legions 006b13e0 — UI render-surface plumbing is ac2-gpu ui.rs (UiOverlay layers placed by set_placements, resized with the window, drawn after the tone map) with ac2-client compose.rs; the D3D surface/material/ray-test object has no counterpart type

### UIRectangleObject
- [-] `UIRectangleObject::Render` · 00735390 · legions 006b1060 — UI render-surface plumbing is ac2-gpu ui.rs (UiOverlay layers placed by set_placements, resized with the window, drawn after the tone map) with ac2-client compose.rs; the D3D surface/material/ray-test object has no counterpart type

## `portal/client/src/render/ui/uiscene.cpp.c` (5/23)

### (free functions)
- [ ] `STREAMPACK_PACKOBJ` · 005acbc0 · legions 00578b20 — needs port: STREAMPACK_PACKOBJ behaviour

### UIScene
- [x] `UIScene::GetEntity` · 005acd10 · legions 00578c70 — pregame 3D scenes: ac2-assets ac2_ui_scene.rs, ac2-client pregame_scene/, ac2-ui pregame/scene.rs
- [-] `UIScene::RemoveEntity` · 005acd50 · legions 00578cb0 — Scene bookkeeping (id pools, entity lists, teardown): UI scenes are owned values (scene_viewport.rs / paper_doll_scene.rs) freed on drop, with no manager-wide id table
- [-] `UIScene::DeleteEntity` · 005acdd0 · legions 00578d30 — Scene bookkeeping (id pools, entity lists, teardown): UI scenes are owned values (scene_viewport.rs / paper_doll_scene.rs) freed on drop, with no manager-wide id table
- [-] `UIScene::DeleteAllEntities` · 005ace50 · legions 00578db0 — Scene bookkeeping (id pools, entity lists, teardown): UI scenes are owned values (scene_viewport.rs / paper_doll_scene.rs) freed on drop, with no manager-wide id table
- [x] `UIScene::StartSelection` · 005acea0 · legions 00578e00 — pregame 3D scenes: ac2-assets ac2_ui_scene.rs, ac2-client pregame_scene/, ac2-ui pregame/scene.rs
- [~] `UIScene::Think` · 005acee0 · legions 00578e40 — pregame 3D scenes: no orthographic cameras, volumetric shadows or coronas; no tooltips
- [-] `UIScene::Destroy` · 005acf10 · legions 00578e70 — Scene bookkeeping (id pools, entity lists, teardown): UI scenes are owned values (scene_viewport.rs / paper_doll_scene.rs) freed on drop, with no manager-wide id table
- [-] `UIScene::Cleanup` · 005acf50 — Scene bookkeeping (id pools, entity lists, teardown): UI scenes are owned values (scene_viewport.rs / paper_doll_scene.rs) freed on drop, with no manager-wide id table
- [x] `UIScene::StreamPack` · 005acf60 · legions 00578ec0 — pregame 3D scenes: ac2-assets ac2_ui_scene.rs, ac2-client pregame_scene/, ac2-ui pregame/scene.rs
- [ ] `UIScene::GetSubDataIDs` · 005ad010 · legions 00578f70 — needs port: sub-object DIDs for DBOCache resolution (GetSubDataIDs); ac2-dat has none yet (AGENTS requires it)

### PSmartArray<EntityDesc>
- [-] `PSmartArray<EntityDesc>::StreamPack` · 005ad050 · legions 00578fb0 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### UIScene
- [-] `UIScene::UIScene` · 005ad100 · legions 00579060 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [-] `UIScene::GetDBOType` · 005ad1c0 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [-] `UIScene::~UIScene` · 005ad1d0 · legions 00579130 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs

### SmartArray<EntityDesc>
- [-] `SmartArray<EntityDesc>::AddToEnd` · 005ad270 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### UIScene
- [-] `UIScene::Allocate` · 005ad2c0 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [x] `UIScene::AddEntity` · 005ad2f0 · legions 00579200 — pregame 3D scenes: ac2-assets ac2_ui_scene.rs, ac2-client pregame_scene/, ac2-ui pregame/scene.rs
- [x] `UIScene::AddEntity` · 005ad330 · legions 00579240 — pregame 3D scenes: ac2-assets ac2_ui_scene.rs, ac2-client pregame_scene/, ac2-ui pregame/scene.rs
- [-] `UIScene::Init` · 005ad410 · legions 00579320 — Scene bookkeeping (id pools, entity lists, teardown): UI scenes are owned values (scene_viewport.rs / paper_doll_scene.rs) freed on drop, with no manager-wide id table
- [-] `UIScene::FromFileNode` · 005ad450 — text source format (admin tools)
- [-] `UIScene::ToFileNode` · 005ad820 — text source format (admin tools)
- [~] `UIScene::Render` · 005ad8e0 — pregame 3D scenes: no orthographic cameras, volumetric shadows or coronas; no tooltips

## `portal/client/src/render/ui/uiscenecameraentity.cpp.c` (1/3)

### UISceneCameraEntity
- [-] `UISceneCameraEntity::UISceneCameraEntity` · 005d8d30 · legions 0059a460 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own

### UIScenePhysObjEntity
- [ ] `UIScenePhysObjEntity::GetPhysObj` · 005d8d80 · legions 0059a4b0 — needs port (check): UIScenePhysObjEntity::GetPhysObj behaviour; implemented area ac2-ui scene_viewport.rs but uncited

### UISceneCameraEntity
- [x] `UISceneCameraEntity::SetFrame` · 005d8d90 · legions 0059a500 — pregame 3D scenes: ac2-assets ac2_ui_scene.rs, ac2-client pregame_scene/, ac2-ui pregame/scene.rs

## `portal/client/src/render/ui/uisceneentity.cpp.c` (4/6)

### UISceneEntity
- [~] `UISceneEntity::UISceneEntity` · 005d8ff0 · legions 0059a720 — pregame/arms.rs: frames and slerp; no instance id/scene
- [-] `UISceneEntity::~UISceneEntity` · 005d90a0 · legions 0059a7d0 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own
- [x] `UISceneEntity::Think` · 005d9120 · legions 0059a850 — ac2-ui pregame/arms.rs SceneEntity
- [x] `UISceneEntity::SLERPTo` · 005d9210 · legions 0059a940 — ac2-ui pregame/arms.rs SceneEntity
- [x] `UISceneEntity::InitFromDesc` · 005d92e0 · legions 0059aa10 — pregame 3D scenes: ac2-assets ac2_ui_scene.rs, ac2-client pregame_scene/, ac2-ui pregame/scene.rs
- [x] `UISceneEntity::SetID` · 005d9330 · legions 0059aa60 — pregame 3D scenes: ac2-assets ac2_ui_scene.rs, ac2-client pregame_scene/, ac2-ui pregame/scene.rs

## `portal/client/src/render/ui/uiscenelightentity.cpp.c` (2/3)

### UISceneLightEntity
- [-] `UISceneLightEntity::UISceneLightEntity` · 005d8ed0 · legions 0059a640 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own
- [x] `UISceneLightEntity::InitFromDesc` · 005d8f10 · legions 0059a4c0 — pregame 3D scenes: ac2-assets ac2_ui_scene.rs, ac2-client pregame_scene/, ac2-ui pregame/scene.rs
- [x] `UISceneLightEntity::SetFrame` · 005d8f50 · legions 0059a680 — pregame 3D scenes: ac2-assets ac2_ui_scene.rs, ac2-client pregame_scene/, ac2-ui pregame/scene.rs

## `portal/client/src/render/ui/uiscenemanager.cpp.c` (0/7)

### UISceneManager
- [~] `UISceneManager::UseTime` · 005ae180 · legions 00579c20 — pregame 3D scenes: no orthographic cameras, volumetric shadows or coronas; no tooltips
- [-] `UISceneManager::ReleaseUIScene` · 005ae1b0 · legions 00579c50 — Scene bookkeeping (id pools, entity lists, teardown): UI scenes are owned values (scene_viewport.rs / paper_doll_scene.rs) freed on drop, with no manager-wide id table
- [-] `UISceneManager::Cleanup` · 005ae240 · legions 00579ce0 — Scene bookkeeping (id pools, entity lists, teardown): UI scenes are owned values (scene_viewport.rs / paper_doll_scene.rs) freed on drop, with no manager-wide id table
- [-] `UISceneManager::FreeID` · 005ae290 · legions 00579d30 — Scene bookkeeping (id pools, entity lists, teardown): UI scenes are owned values (scene_viewport.rs / paper_doll_scene.rs) freed on drop, with no manager-wide id table
- [-] `UISceneManager::Init` · 005ae2c0 · legions 00579d60 — Scene bookkeeping (id pools, entity lists, teardown): UI scenes are owned values (scene_viewport.rs / paper_doll_scene.rs) freed on drop, with no manager-wide id table
- [~] `UISceneManager::GetUIScene` · 005ae370 · legions 00579e10 — pregame 3D scenes: no orthographic cameras, volumetric shadows or coronas; no tooltips
- [-] `UISceneManager::GetFreeID` · 005ae420 · legions 00579ec0 — Scene bookkeeping (id pools, entity lists, teardown): UI scenes are owned values (scene_viewport.rs / paper_doll_scene.rs) freed on drop, with no manager-wide id table

## `portal/client/src/render/ui/uiscenephysobjentity.cpp.c` (6/13)

### UIScenePhysObjEntity
- [x] `UIScenePhysObjEntity::InitFromDID` · 005adcd0 · legions 00579770 — pregame 3D scenes: ac2-assets ac2_ui_scene.rs, ac2-client pregame_scene/, ac2-ui pregame/scene.rs
- [ ] `UIScenePhysObjEntity::InitFromVDesc` · 005add20 · legions 005797c0 — needs port (check): UIScenePhysObjEntity::InitFromVDesc; paper_doll_scene.rs module doc
- [x] `UIScenePhysObjEntity::InitFromDesc` · 005add60 · legions 00579800 — pregame 3D scenes: ac2-assets ac2_ui_scene.rs, ac2-client pregame_scene/, ac2-ui pregame/scene.rs
- [x] `UIScenePhysObjEntity::Think` · 005addb0 · legions 00579850 — pregame 3D scenes: ac2-assets ac2_ui_scene.rs, ac2-client pregame_scene/, ac2-ui pregame/scene.rs
- [ ] `UIScenePhysObjEntity::SetInstanceID` · 005ade20 · legions 005798c0 — needs port (check): UIScenePhysObjEntity::SetInstanceID behaviour; implemented area ac2-ui scene_viewport.rs but uncited
- [ ] `UIScenePhysObjEntity::ApplyVisualDesc` · 005ade60 · legions 00579900 — needs port (check): UIScenePhysObjEntity::ApplyVisualDesc behaviour; implemented area ac2-ui scene_viewport.rs but uncited
- [ ] `UIScenePhysObjEntity::ApplyAdditiveVisualDesc` · 005ade80 · legions 00579920 — needs port (check): UIScenePhysObjEntity::ApplyAdditiveVisualDesc behaviour; implemented area ac2-ui scene_viewport.rs but uncited
- [-] `UIScenePhysObjEntity::UIScenePhysObjEntity` · 005adec0 · legions 00579960 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own

### UISceneEntity
- [x] `UISceneEntity::SetFrame` · 005adf30 · legions 005799d0 — pregame 3D scenes: ac2-assets ac2_ui_scene.rs, ac2-client pregame_scene/, ac2-ui pregame/scene.rs

### UIScenePhysObjEntity
- [ ] `UIScenePhysObjEntity::Animate` · 005adf40 · legions 005799e0 — needs port (check): UIScenePhysObjEntity::Animate behaviour; implemented area ac2-ui scene_viewport.rs but uncited
- [-] `UIScenePhysObjEntity::~UIScenePhysObjEntity` · 005adf50 · legions 005799f0 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own
- [x] `UIScenePhysObjEntity::Render` · 005ae030 · legions 00579ad0 — pregame 3D scenes: ac2-assets ac2_ui_scene.rs, ac2-client pregame_scene/, ac2-ui pregame/scene.rs
- [x] `UIScenePhysObjEntity::AddChild` · 005ae130 · legions 00579bd0 — pregame 3D scenes: ac2-assets ac2_ui_scene.rs, ac2-client pregame_scene/, ac2-ui pregame/scene.rs

## `portal/client/src/render/ui/uisurface.cpp.c` (0/9)

### UISurface
- [-] `UISurface::GetBestWidthHeight` · 005b0ec0 · legions 0057c940 — D3D surface / vertex / matrix / material plumbing replaced by wgpu; the drawn result is ac2-gpu

### RenderSurface
- [-] `RenderSurface::SelectDirtyRectList` · 005b0f20 · legions 0057c9d0 — D3D surface / vertex / matrix / material plumbing replaced by wgpu; the drawn result is ac2-gpu

### UISurface
- [-] `UISurface::UISurface` · 005b0f40 · legions 0057c9f0 — D3D surface / vertex / matrix / material plumbing replaced by wgpu; the drawn result is ac2-gpu
- [-] `UISurface::SetupVertexBuffer` · 005b0f90 · legions 0057ca40 — D3D surface / vertex / matrix / material plumbing replaced by wgpu; the drawn result is ac2-gpu
- [-] `UISurface::DestroySurface` · 005b10a0 · legions 0057cb50 — D3D surface / vertex / matrix / material plumbing replaced by wgpu; the drawn result is ac2-gpu
- [-] `UISurface::CreateSurface` · 005b1140 · legions 0057cbf0 — D3D surface / vertex / matrix / material plumbing replaced by wgpu; the drawn result is ac2-gpu
- [-] `UISurface::PrepareSurface` · 005b1270 · legions 0057cd20 — UI render-surface plumbing is ac2-gpu ui.rs (UiOverlay layers placed by set_placements, resized with the window, drawn after the tone map) with ac2-client compose.rs; the D3D surface/material/ray-test object has no counterpart type
- [-] `UISurface::Resize` · 005b1320 · legions 0057cdd0 — UI render-surface plumbing is ac2-gpu ui.rs (UiOverlay layers placed by set_placements, resized with the window, drawn after the tone map) with ac2-client compose.rs; the D3D surface/material/ray-test object has no counterpart type
- [-] `UISurface::RefreshHardware` · 005b13b0 · legions 0057ce60 — D3D surface / vertex / matrix / material plumbing replaced by wgpu; the drawn result is ac2-gpu

## `portal/client/src/render/ui/uisurfaceobject.cpp.c` (2/28)

### UISurfaceObject
- [-] `UISurfaceObject::UISurfaceObject` · 005b23b0 · legions 00582870 — D3D surface / vertex / matrix / material plumbing replaced by wgpu; the drawn result is ac2-gpu
- [-] `UISurfaceObject::GetSurface` · 005b2500 · legions 005829c0 — UI render-surface plumbing is ac2-gpu ui.rs (UiOverlay layers placed by set_placements, resized with the window, drawn after the tone map) with ac2-client compose.rs; the D3D surface/material/ray-test object has no counterpart type
- [-] `UISurfaceObject::GetMaterialOpacity` · 005b2510 · legions 005829d0 — UI render-surface plumbing is ac2-gpu ui.rs (UiOverlay layers placed by set_placements, resized with the window, drawn after the tone map) with ac2-client compose.rs; the D3D surface/material/ray-test object has no counterpart type
- [-] `UISurfaceObject::GetObjectXPos` · 005b2520 · legions 005829e0 — UI render-surface plumbing is ac2-gpu ui.rs (UiOverlay layers placed by set_placements, resized with the window, drawn after the tone map) with ac2-client compose.rs; the D3D surface/material/ray-test object has no counterpart type
- [-] `UISurfaceObject::GetObjectYPos` · 005b2530 · legions 005829f0 — UI render-surface plumbing is ac2-gpu ui.rs (UiOverlay layers placed by set_placements, resized with the window, drawn after the tone map) with ac2-client compose.rs; the D3D surface/material/ray-test object has no counterpart type
- [-] `UISurfaceObject::GetObjectHeight` · 005b2540 · legions 00582a00 — UI render-surface plumbing is ac2-gpu ui.rs (UiOverlay layers placed by set_placements, resized with the window, drawn after the tone map) with ac2-client compose.rs; the D3D surface/material/ray-test object has no counterpart type
- [-] `UISurfaceObject::GetObjectViewHeight` · 005b2550 · legions 00582a10 — UI render-surface plumbing is ac2-gpu ui.rs (UiOverlay layers placed by set_placements, resized with the window, drawn after the tone map) with ac2-client compose.rs; the D3D surface/material/ray-test object has no counterpart type
- [-] `UISurfaceObject::GetRotateVals` · 005b2560 · legions 00582a20 — UI render-surface plumbing is ac2-gpu ui.rs (UiOverlay layers placed by set_placements, resized with the window, drawn after the tone map) with ac2-client compose.rs; the D3D surface/material/ray-test object has no counterpart type
- [-] `UISurfaceObject::SetTransform` · 005b2590 · legions 00582a50 — D3D surface / vertex / matrix / material plumbing replaced by wgpu; the drawn result is ac2-gpu
- [-] `UISurfaceObject::SetSurface` · 005b25c0 · legions 00582a80 — UI render-surface plumbing is ac2-gpu ui.rs (UiOverlay layers placed by set_placements, resized with the window, drawn after the tone map) with ac2-client compose.rs; the D3D surface/material/ray-test object has no counterpart type
- [-] `UISurfaceObject::RayTest` · 005b2610 · legions 00582ad0 — UI render-surface plumbing is ac2-gpu ui.rs (UiOverlay layers placed by set_placements, resized with the window, drawn after the tone map) with ac2-client compose.rs; the D3D surface/material/ray-test object has no counterpart type
- [-] `UISurfaceObject::GetScreenLocalPosition` · 005b2630 · legions 00582af0 — UI render-surface plumbing is ac2-gpu ui.rs (UiOverlay layers placed by set_placements, resized with the window, drawn after the tone map) with ac2-client compose.rs; the D3D surface/material/ray-test object has no counterpart type
- [-] `UISurfaceObject::ObjectToSurface` · 005b2680 · legions 00582b40 — UI render-surface plumbing is ac2-gpu ui.rs (UiOverlay layers placed by set_placements, resized with the window, drawn after the tone map) with ac2-client compose.rs; the D3D surface/material/ray-test object has no counterpart type
- [-] `UISurfaceObject::ClipToSurface` · 005b26f0 · legions 00582bb0 — UI render-surface plumbing is ac2-gpu ui.rs (UiOverlay layers placed by set_placements, resized with the window, drawn after the tone map) with ac2-client compose.rs; the D3D surface/material/ray-test object has no counterpart type
- [-] `UISurfaceObject::BuildClipMatrix` · 005b2790 · legions 00582c50 — D3D surface / vertex / matrix / material plumbing replaced by wgpu; the drawn result is ac2-gpu
- [x] `UISurfaceObject::BuildRotateMatrix` · 005b2860 · legions 00582d20 — ac2-ui reticle.rs + ac2-gpu ui.rs: the targeting reticle
- [-] `UISurfaceObject::BuildScreenMatrix` · 005b2930 · legions 00582df0 — D3D surface / vertex / matrix / material plumbing replaced by wgpu; the drawn result is ac2-gpu
- [-] `UISurfaceObject::SetScreenPosition` · 005b29f0 · legions 00582eb0 — UI render-surface plumbing is ac2-gpu ui.rs (UiOverlay layers placed by set_placements, resized with the window, drawn after the tone map) with ac2-client compose.rs; the D3D surface/material/ray-test object has no counterpart type
- [-] `UISurfaceObject::ResizeSurface` · 005b2a70 · legions 00582f30 — UI render-surface plumbing is ac2-gpu ui.rs (UiOverlay layers placed by set_placements, resized with the window, drawn after the tone map) with ac2-client compose.rs; the D3D surface/material/ray-test object has no counterpart type
- [-] `UISurfaceObject::SetScreenPosition` · 005b2ab0 · legions 00582f70 — UI render-surface plumbing is ac2-gpu ui.rs (UiOverlay layers placed by set_placements, resized with the window, drawn after the tone map) with ac2-client compose.rs; the D3D surface/material/ray-test object has no counterpart type
- [-] `UISurfaceObject::SetMaterialColor` · 005b2b60 · legions 00583020 — UI render-surface plumbing is ac2-gpu ui.rs (UiOverlay layers placed by set_placements, resized with the window, drawn after the tone map) with ac2-client compose.rs; the D3D surface/material/ray-test object has no counterpart type
- [-] `UISurfaceObject::SetMaterialOpacity` · 005b2bc0 · legions 00583080 — UI render-surface plumbing is ac2-gpu ui.rs (UiOverlay layers placed by set_placements, resized with the window, drawn after the tone map) with ac2-client compose.rs; the D3D surface/material/ray-test object has no counterpart type
- [-] `UISurfaceObject::SetMaterialFiltering` · 005b2c30 · legions 005830f0 — D3D surface / vertex / matrix / material plumbing replaced by wgpu; the drawn result is ac2-gpu
- [~] `UISurfaceObject::Render` · 005b2c80 · legions 00583140 — ac2-ui hud.rs + ac2-gpu ui.rs + ac2-client hud_view.rs: HUD roots made and drawn; no framework behaviour beyond hiding the toolbar's dismount/consignment buttons
- [-] `UISurfaceObject::GetLocalPosition` · 005b2dc0 · legions 00583280 — UI render-surface plumbing is ac2-gpu ui.rs (UiOverlay layers placed by set_placements, resized with the window, drawn after the tone map) with ac2-client compose.rs; the D3D surface/material/ray-test object has no counterpart type
- [x] `UISurfaceObject::SetRotateVals` · 005b2e10 · legions 005832d0 — ac2-ui reticle.rs + ac2-gpu ui.rs: the targeting reticle
- [~] `UISurfaceObject::GenerateMaterial` · 005b2ea0 · legions 00583360 — ac2-ui hud.rs + ac2-gpu ui.rs + ac2-client hud_view.rs: HUD roots made and drawn; no framework behaviour beyond hiding the toolbar's dismount/consignment buttons
- [-] `UISurfaceObject::SetMaterial` · 005b3000 · legions 005834c0 — D3D surface / vertex / matrix / material plumbing replaced by wgpu; the drawn result is ac2-gpu

## `portal/client/src/render/ui/uiviewportobject.cpp.c` (2/3)

### UIViewportObject
- [-] `UIViewportObject::UIViewportObject` · 00735550 · legions 006b1220 — D3D surface / vertex / matrix / material plumbing replaced by wgpu; the drawn result is ac2-gpu
- [x] `UIViewportObject::SetSceneID` · 00735590 · legions 006b1290 — pregame 3D scenes: ac2-assets ac2_ui_scene.rs, ac2-client pregame_scene/, ac2-ui pregame/scene.rs
- [x] `UIViewportObject::Render` · 007355a0 · legions 006b12a0 — pregame 3D scenes: ac2-assets ac2_ui_scene.rs, ac2-client pregame_scene/, ac2-ui pregame/scene.rs

## `portal/client/src/uielement/desc/attributedesc.cpp.c` (1/6)

### AttributeDesc
- [ ] `AttributeDesc::GetSubDataIDs` · 005e10f0 · legions 005a1e80 — needs port: sub-object DIDs for DBOCache resolution (GetSubDataIDs); ac2-dat has none yet (AGENTS requires it)
- [-] `AttributeDesc::AttributeDesc` · 005e1110 · legions 005a1ea0 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [-] `AttributeDesc::~AttributeDesc` · 005e1130 · legions 005a1ec0 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [-] `AttributeDesc::GetPackSize` · 005e1210 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [-] `AttributeDesc::Pack` · 005e1280 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [x] `AttributeDesc::UnPack` · 005e1390 — ac2-ui layout.rs / layout_media.rs: LayoutDesc decode; all 14 Legions layouts decode to the last byte

## `portal/client/src/uielement/desc/elementdesc.cpp.c` (2/8)

### ElementDesc
- [x] `ElementDesc::GetReferentType` · 005dfaf0 · legions 005a0880 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [-] `ElementDesc::GetPackSize` · 005dfb50 · legions 005a08e0 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [-] `ElementDesc::Pack` · 005dfbd0 · legions 005a0960 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [~] `ElementDesc::GetSubDataIDs` · 005dfd30 · legions 005a0ac0 — subIDArray and each element's sub ids decode (layout.rs sub_ids); in every local layout the sub ids are the layout's own string table (0x25xxxxxx) and add no element (real_data_tests every_layout_sub_id_is_a_string_table_and_adds_no_element). The DBOCache dependent-load that consumes them is not ported.
- [-] `ElementDesc::~ElementDesc` · 005dfde0 · legions 005a0b70 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [x] `ElementDesc::UnPack` · 005dff30 · legions 005a0cc0 — ac2-ui layout.rs / layout_media.rs: LayoutDesc decode; all 14 Legions layouts decode to the last byte
- [-] `ElementDesc::Copy` · 005e0110 · legions 005a0ea0 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [-] `ElementDesc::ElementDesc` · 005e0310 · legions 005a10a0 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs

## `portal/client/src/uielement/desc/layoutdesc.cpp.c` (3/10)

### LayoutDesc
- [-] `LayoutDesc::GetPackSize` · 005df410 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [-] `LayoutDesc::Pack` · 005df4a0 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [~] `LayoutDesc::GetSubDataIDs` · 005df590 — subIDArray and each element's sub ids decode (layout.rs sub_ids); in every local layout the sub ids are the layout's own string table (0x25xxxxxx) and add no element (real_data_tests every_layout_sub_id_is_a_string_table_and_adds_no_element). The DBOCache dependent-load that consumes them is not ported.
- [-] `LayoutDesc::~LayoutDesc` · 005df660 · legions 005a03f0 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [-] `LayoutDesc::GetDBOType` · 005df770 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [x] `LayoutDesc::GetRootElementDesc` · 005df7a0 · legions 005a0530 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [x] `LayoutDesc::DereferenceElementDesc` · 005df7e0 · legions 005a0570 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [x] `LayoutDesc::UnPack` · 005df8a0 — ac2-ui layout.rs / layout_media.rs: LayoutDesc decode; all 14 Legions layouts decode to the last byte
- [-] `LayoutDesc::LayoutDesc` · 005dfa80 · legions 005a0810 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [-] `LayoutDesc::Allocate` · 005dfae0 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs

## `portal/client/src/uielement/desc/media/md_data_alpha.cpp.c` (2/4)

### MD_Data_Alpha
- [-] `MD_Data_Alpha::MD_Data_Alpha` · 005e2190 · legions 005a2f80 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [x] `MD_Data_Alpha::Update` · 005e21b0 — ac2-ui media.rs / transition.rs
- [-] `MD_Data_Alpha::Copy` · 005e21d0 · legions 005a2ff0 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [x] `MD_Data_Alpha::StreamPack` · 005e2230 · legions 005a2770 — ac2-ui layout.rs / layout_media.rs: LayoutDesc decode; all 14 Legions layouts decode to the last byte

## `portal/client/src/uielement/desc/media/md_data_anim.cpp.c` (3/6)

### MD_Data_Anim
- [x] `MD_Data_Anim::Reset` · 005e22a0 — ac2-ui media.rs / transition.rs
- [ ] `MD_Data_Anim::GetSubDataIDs` · 005e22c0 — needs port: sub-object DIDs for DBOCache resolution (GetSubDataIDs); ac2-dat has none yet (AGENTS requires it)
- [x] `MD_Data_Anim::Update` · 005e2300 — ac2-ui media.rs / transition.rs
- [x] `MD_Data_Anim::StreamPack` · 005e23b0 · legions 005a3140 — ac2-ui layout.rs / layout_media.rs: LayoutDesc decode; all 14 Legions layouts decode to the last byte
- [-] `MD_Data_Anim::MD_Data_Anim` · 005e2440 · legions 005a31d0 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [-] `MD_Data_Anim::Copy` · 005e24d0 · legions 005a3260 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs

## `portal/client/src/uielement/desc/media/md_data_avi.cpp.c` (1/6)

### MD_Data_AVI
- [-] `MD_Data_AVI::Reset` · 005e19c0 · legions 005a27e0 — no layout on the DVD has an AVI item
- [-] `MD_Data_AVI::Update` · 005e19f0 · legions 005a2810 — no layout on the DVD has an AVI item
- [-] `MD_Data_AVI::MD_Data_AVI` · 005e1ab0 · legions 005a28d0 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [-] `MD_Data_AVI::~MD_Data_AVI` · 005e1af0 · legions 005a2910 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [-] `MD_Data_AVI::Copy` · 005e1b40 · legions 005a2960 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [x] `MD_Data_AVI::StreamPack` · 005e1bd0 · legions 005a29f0 — ac2-ui layout.rs / layout_media.rs: LayoutDesc decode; all 14 Legions layouts decode to the last byte

## `portal/client/src/uielement/desc/media/md_data_cursor.cpp.c` (1/3)

### MD_Data_Cursor
- [-] `MD_Data_Cursor::MD_Data_Cursor` · 005e1930 · legions 005a26c0 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [x] `MD_Data_Cursor::Update` · 005e1950 — ac2-ui media.rs / transition.rs
- [-] `MD_Data_Cursor::Copy` · 005e1980 · legions 005a2710 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs

## `portal/client/src/uielement/desc/media/md_data_image.cpp.c` (2/5)

### MD_Data_Image
- [-] `MD_Data_Image::MD_Data_Image` · 005e1f60 · legions 005a2d80 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs

### MD_Data_Alpha
- [ ] `MD_Data_Alpha::GetSubDataIDs` · 005e1f90 — needs port: sub-object DIDs for DBOCache resolution (GetSubDataIDs); ac2-dat has none yet (AGENTS requires it)

### MD_Data_Image
- [x] `MD_Data_Image::Update` · 005e1fc0 — ac2-ui media.rs / transition.rs
- [-] `MD_Data_Image::Copy` · 005e1fe0 · legions 005a2dd0 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [x] `MD_Data_Image::StreamPack` · 005e2020 · legions 005a2e10 — ac2-ui layout.rs / layout_media.rs: LayoutDesc decode; all 14 Legions layouts decode to the last byte

## `portal/client/src/uielement/desc/media/md_data_jump.cpp.c` (2/4)

### MD_Data_Jump
- [-] `MD_Data_Jump::MD_Data_Jump` · 005e1da0 · legions 005a2c20 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [x] `MD_Data_Jump::Update` · 005e1dc0 — ac2-ui media.rs / transition.rs
- [-] `MD_Data_Jump::Copy` · 005e1e10 · legions 005a2c90 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [x] `MD_Data_Jump::StreamPack` · 005e1e40 · legions 005a2bc0 — ac2-ui layout.rs / layout_media.rs: LayoutDesc decode; all 14 Legions layouts decode to the last byte

## `portal/client/src/uielement/desc/media/md_data_message.cpp.c` (1/3)

### MD_Data_Message
- [-] `MD_Data_Message::MD_Data_Message` · 005e1d00 · legions 005a2b20 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [x] `MD_Data_Message::Update` · 005e1d20 — ac2-ui media.rs / transition.rs
- [-] `MD_Data_Message::Copy` · 005e1d70 · legions 005a2b90 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs

## `portal/client/src/uielement/desc/media/md_data_pause.cpp.c` (2/4)

### MD_Data_Pause
- [-] `MD_Data_Pause::MD_Data_Pause` · 005e20a0 · legions 005a2e90 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [x] `MD_Data_Pause::Reset` · 005e20d0 — ac2-ui media.rs / transition.rs
- [x] `MD_Data_Pause::Update` · 005e20f0 — ac2-ui media.rs / transition.rs
- [-] `MD_Data_Pause::Copy` · 005e2160 · legions 005a2f50 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs

## `portal/client/src/uielement/desc/media/md_data_sound.cpp.c` (1/3)

### MD_Data_Sound
- [-] `MD_Data_Sound::MD_Data_Sound` · 005e1ea0 · legions 005a2cc0 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [x] `MD_Data_Sound::Update` · 005e1ec0 — ac2-ui media queue -> ac2-audio SoundManager::Play
- [-] `MD_Data_Sound::Copy` · 005e1f30 · legions 005a2d50 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs

## `portal/client/src/uielement/desc/media/md_data_state.cpp.c` (2/4)

### MD_Data_State
- [-] `MD_Data_State::MD_Data_State` · 005e1c40 · legions 005a2a60 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [x] `MD_Data_State::Update` · 005e1c60 — ac2-ui media.rs / transition.rs
- [-] `MD_Data_State::Copy` · 005e1c80 · legions 005a2aa0 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs

### MD_Data_Sound
- [x] `MD_Data_Sound::StreamPack` · 005e1cb0 · legions 005a2ad0 — ac2-ui layout.rs / layout_media.rs: LayoutDesc decode; all 14 Legions layouts decode to the last byte

## `portal/client/src/uielement/desc/mediadesc.cpp.c` (2/4)

### MediaDesc
- [x] `MediaDesc::CreateMediaType` · 005e1700 · legions 005a2490 — ac2-ui layout.rs / layout_media.rs: LayoutDesc decode; all 14 Legions layouts decode to the last byte
- [-] `MediaDesc::MediaDesc` · 005e1830 · legions 005a25c0 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [x] `MediaDesc::StreamPack` · 005e1850 · legions 005a25e0 — ac2-ui layout.rs / layout_media.rs: LayoutDesc decode; all 14 Legions layouts decode to the last byte
- [-] `MediaDesc::~MediaDesc` · 005e1880 · legions 005a2610 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs

## `portal/client/src/uielement/desc/statedesc.cpp.c` (1/7)

### StateDesc
- [-] `StateDesc::GetPackSize` · 005e0380 · legions 005a1110 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [-] `StateDesc::Pack` · 005e04a0 · legions 005a1230 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [ ] `StateDesc::GetSubDataIDs` · 005e0690 · legions 005a1420 — needs port: sub-object DIDs for DBOCache resolution (GetSubDataIDs); ac2-dat has none yet (AGENTS requires it)
- [-] `StateDesc::~StateDesc` · 005e0750 · legions 005a14e0 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [x] `StateDesc::UnPack` · 005e0950 · legions 005a16e0 — ac2-ui layout.rs / layout_media.rs: LayoutDesc decode; all 14 Legions layouts decode to the last byte
- [-] `StateDesc::Copy` · 005e0cd0 · legions 005a1a60 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [-] `StateDesc::StateDesc` · 005e1060 · legions 005a1df0 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs

## `portal/client/src/uielement/desc/transitiondesc.cpp.c` (1/2)

### TransitionDesc
- [-] `TransitionDesc::TransitionDesc` · 005e1490 · legions 005a2220 — DBObj record plumbing (ctor/dtor/copy/alloc/pack writer); the client only reads the record in layout.rs / layout_media.rs
- [x] `TransitionDesc::StreamPack` · 005e14b0 · legions 005a2240 — ac2-ui layout.rs / layout_media.rs: LayoutDesc decode; all 14 Legions layouts decode to the last byte

## `portal/client/src/uielement/file/filetree.cpp.c` (0/1)

### FileTree
- [-] `FileTree::~FileTree` · 005fd250 · legions 005bdd80 — authoring/export/debug-info tooling (checkpoint export, file tree, source line info); no runtime behaviour

## `portal/client/src/uielement/framework/gameplay/gameplayui.cpp.c` (0/2)

### GamePlayUI
- [-] `GamePlayUI::GamePlayUI` · 005fd9c0 · legions 005be4f0 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own
- [-] `GamePlayUI::~GamePlayUI` · 005fda00 · legions 005be530 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own

## `portal/client/src/uielement/framework/uichildframework.cpp.c` (1/6)

### UIChildFramework
- [-] `UIChildFramework::UIChildFramework` · 005fe110 · legions 005bec10 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own
- [-] `UIChildFramework::~UIChildFramework` · 005fe140 · legions 005bec40 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own
- [x] `UIChildFramework::CreateAndAddRootElement` · 005fe170 · legions 005bec70 — ac2-ui hud.rs
- [-] `UIChildFramework::RemoveRootElement` · 005fe190 · legions 005bec90 — child-framework plumbing (root list, parent forwarding); ac2-ui panels own their roots as UiTree indices and Hud calls them directly
- [-] `UIChildFramework::GetUIFlow` · 005fe1a0 · legions 005beca0 — child-framework plumbing (root list, parent forwarding); ac2-ui panels own their roots as UiTree indices and Hud calls them directly
- [-] `UIChildFramework::Shown` · 005fe1b0 · legions 005becb0 — child-framework plumbing (root list, parent forwarding); ac2-ui panels own their roots as UiTree indices and Hud calls them directly

## `portal/client/src/uielement/framework/uiflow.cpp.c` (4/8)

### UIFlow
- [ ] `UIFlow::GetUIFlowPtr` · 005fd2c0 · legions 005bddf0 — needs port (check): UIFlow::GetUIFlowPtr behaviour; implemented area ac2-ui hud.rs but uncited
- [ ] `UIFlow::Update` · 005fd2d0 — needs port (check): UIFlow::Update behaviour; implemented area ac2-ui hud.rs but uncited
- [~] `UIFlow::UIFlow` · 005fd320 · legions 005bde50 — pregame/ui.rs: character selection and disconnected made; intro, data patch (the client connects), keyboard, credits not ported
- [-] `UIFlow::~UIFlow` · 005fd410 · legions 005bdf40 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own
- [x] `UIFlow::QueueUIMode` · 005fd4c0 · legions 005bdff0 — ac2-ui pregame/flow.rs, pregame/ui.rs
- [x] `UIFlow::RegisterFrameworkClass` · 005fd5d0 · legions 005be100 — ac2-ui pregame/flow.rs, pregame/ui.rs
- [x] `UIFlow::UseNewMode` · 005fd620 · legions 005be150 — ac2-ui pregame/flow.rs, pregame/ui.rs
- [x] `UIFlow::ListenToMessage` · 005fd720 · legions 005be250 — ac2-ui pregame/flow.rs, pregame/ui.rs

## `portal/client/src/uielement/framework/uiframework.cpp.c` (0/9)

### UIFramework
- [-] `UIFramework::GetUIPersistantData` · 005fd7d0 · legions 005be300 — child-framework list bookkeeping (a SmartArray of UIChildFramework with swap-remove, Show/LocationApplied fan-out); ac2-ui Hud owns its panels as typed fields and calls them directly, so no framework object or child list exists. CreateElement is UiTree::create_element/create_root
- [-] `UIFramework::CreateElement` · 005fd7f0 · legions 005be320 — child-framework list bookkeeping (a SmartArray of UIChildFramework with swap-remove, Show/LocationApplied fan-out); ac2-ui Hud owns its panels as typed fields and calls them directly, so no framework object or child list exists. CreateElement is UiTree::create_element/create_root
- [-] `UIFramework::UIFramework` · 005fd810 · legions 005be340 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own
- [-] `UIFramework::~UIFramework` · 005fd840 · legions 005be370 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own
- [-] `UIFramework::Show` · 005fd860 · legions 005be390 — child-framework list bookkeeping (a SmartArray of UIChildFramework with swap-remove, Show/LocationApplied fan-out); ac2-ui Hud owns its panels as typed fields and calls them directly, so no framework object or child list exists. CreateElement is UiTree::create_element/create_root
- [-] `UIFramework::FindChild` · 005fd8a0 · legions 005be3d0 — child-framework list bookkeeping (a SmartArray of UIChildFramework with swap-remove, Show/LocationApplied fan-out); ac2-ui Hud owns its panels as typed fields and calls them directly, so no framework object or child list exists. CreateElement is UiTree::create_element/create_root
- [-] `UIFramework::LocationApplied` · 005fd8d0 · legions 005be400 — child-framework list bookkeeping (a SmartArray of UIChildFramework with swap-remove, Show/LocationApplied fan-out); ac2-ui Hud owns its panels as typed fields and calls them directly, so no framework object or child list exists. CreateElement is UiTree::create_element/create_root
- [-] `UIFramework::RemoveChild` · 005fd940 · legions 005be470 — child-framework list bookkeeping (a SmartArray of UIChildFramework with swap-remove, Show/LocationApplied fan-out); ac2-ui Hud owns its panels as typed fields and calls them directly, so no framework object or child list exists. CreateElement is UiTree::create_element/create_root
- [-] `UIFramework::AddChild` · 005fd970 · legions 005be4a0 — child-framework list bookkeeping (a SmartArray of UIChildFramework with swap-remove, Show/LocationApplied fan-out); ac2-ui Hud owns its panels as typed fields and calls them directly, so no framework object or child list exists. CreateElement is UiTree::create_element/create_root

## `portal/client/src/uielement/framework/uimainframework.cpp.c` (3/11)

### UIMainFramework
- [ ] `UIMainFramework::MakeNoSurface` · 005fda70 · legions 005be5a0 — needs port (check): UIMainFramework::MakeNoSurface behaviour; implemented area ac2-ui hud.rs but uncited
- [ ] `UIMainFramework::MakeSurfaceObject` · 005fdaa0 · legions 005be5d0 — needs port (check): UIMainFramework::MakeSurfaceObject behaviour; implemented area ac2-ui hud.rs but uncited
- [ ] `UIMainFramework::GetUIFlow` · 005fdba0 · legions 005be6d0 — needs port (check): UIMainFramework::GetUIFlow behaviour; implemented area ac2-ui hud.rs but uncited
- [x] `UIMainFramework::InqLocations` · 005fdbb0 · legions 005be6e0 — ac2-ui window_locations.rs (inq_locations / apply_locations, clip conversions, drag bar movement); tests window_locations_tests.rs on real HUD
- [x] `UIMainFramework::ApplyLocations` · 005fdc40 · legions 005be770 — ac2-ui window_locations.rs (inq_locations / apply_locations, clip conversions, drag bar movement); tests window_locations_tests.rs on real HUD
- [ ] `UIMainFramework::DeleteUISurfaceList` · 005fdd40 · legions 005be870 — needs port (check): UIMainFramework::DeleteUISurfaceList behaviour; implemented area ac2-ui hud.rs but uncited
- [x] `UIMainFramework::RemoveRootElement` · 005fddc0 · legions 005be8f0 — UIMainFramework::RemoveRootElement done: dialog.rs remove_root
- [-] `UIMainFramework::UIMainFramework` · 005fde50 · legions 005be980 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own
- [-] `UIMainFramework::~UIMainFramework` · 005fde90 · legions 005be9c0 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own
- [~] `UIMainFramework::CreateAndAddRootElementByDataID` · 005fded0 · legions 005bea00 — ac2-ui: no transitions, timed media, mouse, listeners or dirty rects; A8R8G8B8 blits only; HUD roots from SetupChildren's radar/selection/statusbar/toolbar
- [~] `UIMainFramework::CreateAndAddRootElement` · 005fe0b0 · legions 005bebe0 — ac2-ui: no transitions, timed media, mouse, listeners or dirty rects; A8R8G8B8 blits only; HUD roots from SetupChildren's radar/selection/statusbar/toolbar

## `portal/client/src/uielement/framework/uipersistantdata.cpp.c` (9/13)

### UIPersistantData
- [x] `UIPersistantData::SetCharSet` · 005fe220 — ac2-ui pregame/persistent.rs (plain data)
- [x] `UIPersistantData::InqCurrentCGR` · 005fe250 — ac2-ui pregame/persistent.rs (plain data)
- [x] `UIPersistantData::ClearGenerationResultTable` · 005fe2b0 · legions 005bedb0 — ac2-ui pregame/persistent.rs (plain data)
- [x] `UIPersistantData::~UIPersistantData` · 005fe380 · legions 005bee80 — ac2-ui pregame/persistent.rs (plain data)

### DirectXPlayingSound
- [-] `DirectXPlayingSound::GetIsInWorld` · 005fe400 — Win32 / DirectInput / D3D plumbing (device poll, VK mapping, focus, hardware cursor, draw list, sound device) replaced by winit/wgpu/rodio

### UIPersistantData
- [x] `UIPersistantData::InCharGen` · 005fe410 — ac2-ui pregame/persistent.rs (plain data)

### CameraManager
- [x] `CameraManager::GetFirstPersonState` · 005fe420 · legions 005bef00

### UIPersistantData
- [ ] `UIPersistantData::SetPlayingSoundContext` · 005fe430 — needs port (check): UIPersistantData::SetPlayingSoundContext behaviour; implemented area ac2-ui window_positions.rs but uncited

### CPhysicsObj
- [ ] `CPhysicsObj::GetScale` · 005fe440 — needs port: CPhysicsObj::GetScale behaviour

### UIPersistantData
- [x] `UIPersistantData::SetCharGenResult` · 005fe450 — ac2-ui pregame/persistent.rs (plain data)
- [x] `UIPersistantData::InqCharGenResult` · 005fe470 · legions 005bef50 — ac2-ui pregame/persistent.rs (plain data)
- [ ] `UIPersistantData::InqGenerationResultTable` · 005fe4c0 — needs port (check): UIPersistantData::InqGenerationResultTable behaviour; implemented area ac2-ui window_positions.rs but uncited
- [x] `UIPersistantData::UIPersistantData` · 005fe640 · legions 005bf120 — ac2-ui pregame/persistent.rs (plain data)

## `portal/client/src/uielement/prim/node.cpp.c` (7/7)

### UIElement_Node
- [x] `UIElement_Node::GetToken` · 00735760 · legions 006b1460 — ac2-ui tree_view.rs
- [x] `UIElement_Node::GetConnection` · 00735770 · legions 006b1470 — ac2-ui tree_view.rs
- [x] `UIElement_Node::GetChildLines` · 007358c0 · legions 006b1560 — ac2-ui tree_view.rs
- [x] `UIElement_Node::UIElement_Node` · 00735a60 · legions 006b1700 — ac2-ui tree_view.rs
- [x] `UIElement_Node::~UIElement_Node` · 00735a90 · legions 006b1730 — ac2-ui tree_view.rs
- [x] `UIElement_Node::AddParent` · 00735c00 · legions 006b18a0 — ac2-ui tree_view.rs
- [x] `UIElement_Node::AddChild` · 00735c30 · legions 006b18d0 — ac2-ui tree_view.rs

## `portal/client/src/uielement/prim/uiregion.cpp.c` (14/31)

### UIRegion
- [-] `UIRegion::SetObject` · 005e2540 · legions 005a32d0 — Render-surface plumbing: UIObject/UISurface ownership, alpha images, dirty marking and forced draws belong to the client's per-root UiPixels composition (ac2-client hud_view/compose.rs), which redraws a root when UiTree::take_changed reports a change
- [-] `UIRegion::SetAlphaImage` · 005e2580 · legions 005a3310 — Render-surface plumbing: UIObject/UISurface ownership, alpha images, dirty marking and forced draws belong to the client's per-root UiPixels composition (ac2-client hud_view/compose.rs), which redraws a root when UiTree::take_changed reports a change
- [-] `UIRegion::ClearAlphaImage` · 005e2590 · legions 005a3320 — Render-surface plumbing: UIObject/UISurface ownership, alpha images, dirty marking and forced draws belong to the client's per-root UiPixels composition (ac2-client hud_view/compose.rs), which redraws a root when UiTree::take_changed reports a change

### CMotionInterp
- [~] `CMotionInterp::SetAccelerationScale` · 005e25b0 · legions 005a3340 — ac2-physics motion_interp.rs set_velocity_scale / set_acceleration_scale (reached from the server and script events); the collision impact minimum CPhysicsObj::SetVelocityScale also sets is not modelled

### UIRegion
- [~] `UIRegion::NotifyMouseMove` · 005e25c0 · legions 005a3350 — ac2-ui mouse.rs / button.rs: no tooltips, cursors, drag and drop, moving/resizing, activation, hot clicks or listener registry (messages are taken by the caller)
- [-] `UIRegion::GetObjectA` · 005e26d0 · legions 005a3460 — Render-surface plumbing: UIObject/UISurface ownership, alpha images, dirty marking and forced draws belong to the client's per-root UiPixels composition (ac2-client hud_view/compose.rs), which redraws a root when UiTree::take_changed reports a change
- [x] `UIRegion::GetObjectX0` · 005e26f0 · legions 005a3480 — UIRegion::GetObjectX0 done: meter.rs object_origin (corner in surface)
- [x] `UIRegion::GetObjectY0` · 005e2720 · legions 005a34b0 — ac2-ui place.rs: the root's object origin (Element::object) that screen_box starts from
- [x] `UIRegion::GetScreenX0` · 005e2750 · legions 005a34e0 — ac2-ui place.rs: UiTree::screen_box (the parent's screen origin, or the root's object origin, plus the rect)
- [x] `UIRegion::GetScreenY0` · 005e2790 · legions 005a3520 — ac2-ui place.rs: UiTree::screen_box (the parent's screen origin, or the root's object origin, plus the rect)
- [x] `UIRegion::GetObjectBox` · 005e27d0 · legions 005a3560 — ac2-ui place.rs: UiTree::screen_box (the parent's screen origin, or the root's object origin, plus the rect)
- [-] `UIRegion::GetSurface` · 005e2860 · legions 005a3620 — Render-surface plumbing: UIObject/UISurface ownership, alpha images, dirty marking and forced draws belong to the client's per-root UiPixels composition (ac2-client hud_view/compose.rs), which redraws a root when UiTree::take_changed reports a change
- [-] `UIRegion::GetSurfaceObjAbsX` · 005e2890 · legions 005a3650 — Render-surface plumbing: UIObject/UISurface ownership, alpha images, dirty marking and forced draws belong to the client's per-root UiPixels composition (ac2-client hud_view/compose.rs), which redraws a root when UiTree::take_changed reports a change
- [-] `UIRegion::GetSurfaceObjAbsY` · 005e28c0 · legions 005a3680 — Render-surface plumbing: UIObject/UISurface ownership, alpha images, dirty marking and forced draws belong to the client's per-root UiPixels composition (ac2-client hud_view/compose.rs), which redraws a root when UiTree::take_changed reports a change
- [-] `UIRegion::MakeRootDirtyHere` · 005e28f0 · legions 005a36b0 — Render-surface plumbing: UIObject/UISurface ownership, alpha images, dirty marking and forced draws belong to the client's per-root UiPixels composition (ac2-client hud_view/compose.rs), which redraws a root when UiTree::take_changed reports a change
- [-] `UIRegion::ForceDraw` · 005e2930 · legions 005a36f0 — Render-surface plumbing: UIObject/UISurface ownership, alpha images, dirty marking and forced draws belong to the client's per-root UiPixels composition (ac2-client hud_view/compose.rs), which redraws a root when UiTree::take_changed reports a change
- [ ] `UIRegion::SetParent_Internal` · 005e2980 · legions 005a3740 — needs port (check): UIRegion::SetParent_Internal behaviour; implemented area ac2-ui element.rs but uncited
- [x] `UIRegion::GetClipBox` · 005e2a30 · legions 005a37f0 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [ ] `UIRegion::GetScreenClipBox` · 005e2b50 · legions 005a3910 — needs port (check): UIRegion::GetScreenClipBox behaviour; implemented area ac2-ui element.rs but uncited
- [x] `UIRegion::SetVisible` · 005e2c30 · legions 005a39f0 — UIRegion::SetVisible done: element.rs set_visible
- [-] `UIRegion::SetImage` · 005e2cc0 · legions 005a3a80 — Render-surface plumbing: UIObject/UISurface ownership, alpha images, dirty marking and forced draws belong to the client's per-root UiPixels composition (ac2-client hud_view/compose.rs), which redraws a root when UiTree::take_changed reports a change
- [x] `UIRegion::ClearImage` · 005e2d30 · legions 005a3af0 — UIRegion::ClearImage done: element.rs clear_image
- [x] `UIRegion::MoveTo` · 005e2da0 · legions 005a3b60 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [x] `UIRegion::ResizeTo` · 005e2e70 · legions 005a3c30 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [~] `UIRegion::DrawHere` · 005e2f50 · legions 005a3d10 — A8R8G8B8 only; colour key and format conversion not modelled
- [-] `UIRegion::~UIRegion` · 005e3140 · legions 005a3f00 — region construction and teardown: Element construction in UiTree::create_element and Drop
- [x] `UIRegion::MouseOver` · 005e31e0 · legions 005a3fa0 — ac2-ui element.rs: Element::mouse_over / mouse_over_top flags set by the mouse pass (mouse.rs)
- [x] `UIRegion::MouseOverTop` · 005e31f0 · legions 005a3fb0 — ac2-ui element.rs: Element::mouse_over / mouse_over_top flags set by the mouse pass (mouse.rs)
- [x] `UIRegion::NotifyMouseDown` · 005e3220 — ac2-ui mouse.rs / button.rs: UI hit tests, presses and buttons
- [x] `UIRegion::NotifyMouseUp` · 005e3310 · legions 005a40d0 — ac2-ui mouse.rs / button.rs: UI hit tests, presses and buttons
- [-] `UIRegion::UIRegion` · 005e33c0 · legions 005a4180 — region construction and teardown: Element construction in UiTree::create_element and Drop

## `portal/client/src/uielement/system/uielement.cpp.c` (68/104)

### UIRectangleObject
- [-] `UIRectangleObject::SetColor` · 005e8b30 · legions 005a9790 — UI render-surface plumbing is ac2-gpu ui.rs (UiOverlay layers placed by set_placements, resized with the window, drawn after the tone map) with ac2-client compose.rs; the D3D surface/material/ray-test object has no counterpart type

### UIElement
- [x] `UIElement::GetType` · 005e8b60 · legions 005a97c0 — ac2-ui element.rs: UiElement::kind (desc->type), parent_index, id (referenceDesc->elementID), UiTree::child (GetChild)

### UISurfaceObject
- [-] `UISurfaceObject::GetObjectWidth` · 005e8b70 · legions 005a97d0 — UI render-surface plumbing is ac2-gpu ui.rs (UiOverlay layers placed by set_placements, resized with the window, drawn after the tone map) with ac2-client compose.rs; the D3D surface/material/ray-test object has no counterpart type

### UIElement
- [x] `UIElement::GetParent` · 005e8b80 · legions 005a97e0 — ac2-ui element.rs: UiElement::kind (desc->type), parent_index, id (referenceDesc->elementID), UiTree::child (GetChild)
- [x] `UIElement::GetID` · 005e8b90 · legions 005a97f0 — ac2-ui element.rs: UiElement::kind (desc->type), parent_index, id (referenceDesc->elementID), UiTree::child (GetChild)
- [x] `UIElement::SetZLevel` · 005e8ba0 · legions 005a9800 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor

### UISurfaceObject
- [-] `UISurfaceObject::GetObjectViewWidth` · 005e8c20 · legions 005a9880 — UI render-surface plumbing is ac2-gpu ui.rs (UiOverlay layers placed by set_placements, resized with the window, drawn after the tone map) with ac2-client compose.rs; the D3D surface/material/ray-test object has no counterpart type

### UIElement
- [ ] `UIElement::Initialized` · 005e8c30 · legions 005a9890 — needs port (check): UIElement::Initialized behaviour; implemented area ac2-ui element.rs but uncited
- [x] `UIElement::GetRootElement` · 005e8c40 · legions 005a98a0 — UIElement::GetRootElement done: place.rs root_of
- [x] `UIElement::MouseMove` · 005e8c60 · legions 005a98c0 — ac2-ui mouse.rs / button.rs: UI hit tests, presses and buttons
- [x] `UIElement::MouseUp` · 005e8c90 · legions 005a98f0 — ac2-ui mouse.rs / button.rs: UI hit tests, presses and buttons
- [x] `UIElement::StartMovement` · 005e8ce0 · legions 005a9940 — ac2-ui window_locations.rs (inq_locations / apply_locations, clip conversions, drag bar movement); tests window_locations_tests.rs on real HUD
- [x] `UIElement::StopMovement` · 005e8d40 · legions 005a99a0 — ac2-ui window_locations.rs (inq_locations / apply_locations, clip conversions, drag bar movement); tests window_locations_tests.rs on real HUD
- [x] `UIElement::MouseMoveElement` · 005e8d60 · legions 005a99c0 — ac2-ui window_locations.rs (inq_locations / apply_locations, clip conversions, drag bar movement); tests window_locations_tests.rs on real HUD
- [x] `UIElement::GetChild` · 005e8e00 · legions 005a9a60 — ac2-ui meter.rs: UiTree::child (the first direct child with the element id)
- [x] `UIElement::GetChildRecursive` · 005e8e30 · legions 005a9a90 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [x] `UIElement::GetStringTableID` · 005e8e70 · legions 005a9ad0 — ac2-ui text.rs / text_attr.rs: UIElement_Text layout and drawing
- [x] `UIElement::SetMediaIndex` · 005e8e80 · legions 005a9ae0 — ac2-ui media.rs / transition.rs
- [x] `UIElement::AddChild` · 005e8e90 · legions 005a9af0 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [ ] `UIElement::SetAlphaImage` · 005e8f30 · legions 005a9b90 — needs port (check): UIElement::SetAlphaImage behaviour; implemented area ac2-ui element.rs but uncited
- [x] `UIElement::SetCursor` · 005e8f70 · legions 005a9bd0 — ac2-ui media.rs / transition.rs
- [ ] `UIElement::RestoreSavedCursor` · 005e8fd0 · legions 005a9c30 — needs port (check): cited from a caller or a neighbouring helper, not the generic element code; verify and cite, or port
- [ ] `UIElement::SaveCursor` · 005e9010 · legions 005a9c70 — needs port (check): cited from a caller or a neighbouring helper, not the generic element code; verify and cite, or port
- [x] `UIElement::MouseOver` · 005e9040 · legions 005a9ca0 — ac2-ui mouse.rs / button.rs: UI hit tests, presses and buttons
- [x] `UIElement::MouseOverTop` · 005e9060 · legions 005a9cc0 — ac2-ui media.rs / transition.rs
- [x] `UIElement::SetAttributeFromDesc` · 005e90b0 · legions 005a9d10 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [x] `UIElement::CopyElem` · 005e9180 · legions 005a9de0 — ac2-ui drag.rs; shortcut.rs drop
- [x] `UIElement::MatchGraphics` · 005e91b0 · legions 005a9e10 — ac2-ui drag.rs; shortcut.rs drop
- [x] `UIElement::DragAndDropComplete` · 005e9250 · legions 005a9eb0 — ac2-ui drag.rs; shortcut.rs drop
- [ ] `UIElement::SetDefaultLocation` · 005e92b0 · legions 005a9f10 — needs port (check): UIElement::SetDefaultLocation behaviour; implemented area ac2-ui element.rs but uncited
- [ ] `UIElement::InqImageSize` · 005e92c0 · legions 005a9f20 — needs port (check): UIElement::InqImageSize behaviour; implemented area ac2-ui element.rs but uncited
- [x] `UIElement::SetTooltip` · 005e9300 · legions 005a9f60 — ac2-ui tooltip.rs
- [x] `UIElement::GetParentCurrentPosition` · 005e9340 · legions 005a9fa0 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [ ] `UIElement::UpdateActivatable` · 005e93e0 · legions 005aa040 — needs port (check): UIElement::UpdateActivatable behaviour; implemented area ac2-ui element.rs but uncited
- [ ] `UIElement::UpdateActivateOrder` · 005e9420 · legions 005aa080 — needs port (check): UIElement::UpdateActivateOrder behaviour; implemented area ac2-ui element.rs but uncited
- [x] `UIElement::Activate` · 005e9470 · legions 005aa0d0 — ac2-ui text_edit.rs: m_visible & parent IsVisible
- [x] `UIElement::Deactivate` · 005e94c0 · legions 005aa120 — ac2-ui input.rs / text_edit.rs: typing into text elements

### UIRegion
- [x] `UIRegion::GetScreenX1` · 005e94f0 · legions 005aa150 — ac2-ui place.rs: UiTree::screen_box (the parent's screen origin, or the root's object origin, plus the rect)
- [x] `UIRegion::GetScreenY1` · 005e9510 · legions 005aa170 — ac2-ui place.rs: UiTree::screen_box (the parent's screen origin, or the root's object origin, plus the rect)

### UIElement
- [~] `UIElement::UIElement` · 005e9530 · legions 005aa190 — ac2-ui: no transitions, timed media, mouse, listeners or dirty rects; A8R8G8B8 blits only; HUD roots from SetupChildren's radar/selection/statusbar/toolbar
- [x] `UIElement::HasCursor` · 005e9710 · legions 005aa370 — ac2-ui media.rs / transition.rs
- [x] `UIElement::SetParent` · 005e9720 · legions 005aa380 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [~] `UIElement::BroadcastMessage` · 005e9790 · legions 005aa3f0 — ac2-ui mouse.rs / button.rs: no tooltips, cursors, drag and drop, moving/resizing, activation, hot clicks or listener registry (messages are taken by the caller)
- [x] `UIElement::UpdateMedia` · 005e97c0 · legions 005aa420 — ac2-ui media.rs / transition.rs
- [x] `UIElement::ClearTooltip` · 005e9810 · legions 005aa470 — ac2-ui tooltip.rs

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PStringBase<char>_*>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PStringBase<char>_*>_*>::~IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PStringBase<char>_*>_*>` · 005e9860 · legions 005aa4c0 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PStringBase<unsigned_short>_>_*>
- [-] `IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PStringBase<unsigned_short>_>_*>::~IntrusiveHashTable<unsigned_long,HashTableData<unsigned_long,PStringBase<unsigned_short>_>_*>` · 005e9890 · legions 005aa4f0 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### UIElement
- [~] `UIElement::StopTransition` · 005e9940 · legions 005aa5a0 — m_delOnTransStop (delete after the exit transition) not modelled
- [x] `UIElement::StopAllTransitions` · 005e99e0 · legions 005aa640 — ac2-ui media.rs / transition.rs
- [ ] `UIElement::SetIconDesc` · 005e9ab0 · legions 005aa710 — needs port (check): UIElement::SetIconDesc behaviour; implemented area ac2-ui element.rs but uncited
- [~] `UIElement::UpdateTransitions` · 005e9b80 · legions 005aa7e0 — fades only (no rotate); sounds queued for an audio sink; cursor image not drawn by the client yet
- [x] `UIElement::GetAttribute_ID` · 005e9f00 · legions 005aaad0 — UIElement::GetAttribute_ID done: story_quests.rs attribute_id
- [x] `UIElement::GetAttribute_InstanceID` · 005e9f60 · legions 005aab30 — ac2-ui element.rs: Element::attributes (typed AttributeValue map: Id/Int/Float/Bool/Ascii/StringId/DataId/InstanceId) read by UiElement::attribute and the int/bool/id helpers in place.rs
- [x] `UIElement::GetAttribute_StringID` · 005e9fa0 · legions 005aab70 — ac2-ui element.rs: Element::attributes (typed AttributeValue map: Id/Int/Float/Bool/Ascii/StringId/DataId/InstanceId) read by UiElement::attribute and the int/bool/id helpers in place.rs
- [x] `UIElement::GetAttribute_Int` · 005ea000 · legions 005aabd0 — ac2-ui element.rs: Element::attributes (typed AttributeValue map: Id/Int/Float/Bool/Ascii/StringId/DataId/InstanceId) read by UiElement::attribute and the int/bool/id helpers in place.rs
- [x] `UIElement::GetAttribute_Float` · 005ea060 · legions 005aac30 — ac2-ui element.rs: Element::attributes (typed AttributeValue map: Id/Int/Float/Bool/Ascii/StringId/DataId/InstanceId) read by UiElement::attribute and the int/bool/id helpers in place.rs
- [x] `UIElement::GetAttribute_Bool` · 005ea0c0 · legions 005aac90 — ac2-ui element.rs: Element::attributes (typed AttributeValue map: Id/Int/Float/Bool/Ascii/StringId/DataId/InstanceId) read by UiElement::attribute and the int/bool/id helpers in place.rs
- [x] `UIElement::GetAttribute_DataID` · 005ea120 · legions 005aacf0 — ac2-ui element.rs: Element::attributes (typed AttributeValue map: Id/Int/Float/Bool/Ascii/StringId/DataId/InstanceId) read by UiElement::attribute and the int/bool/id helpers in place.rs
- [~] `UIElement::GetAttribute_Unicode` · 005ea180 · legions 005aad50 — Unicode attributes are not in AttributeValue: a text element keeps its string in the element (UiTree::set_text, text.rs), not in a generic attribute table
- [x] `UIElement::MouseDown` · 005ea220 · legions 005aadf0 — ac2-ui mouse.rs / button.rs: UI hit tests, presses and buttons
- [x] `UIElement::ResetMedia` · 005ea2c0 · legions 005aae90 — ac2-ui media.rs / transition.rs
- [x] `UIElement::SetImage` · 005ea430 · legions 005ab000 — ac2-ui compose.rs / element.rs set_image
- [x] `UIElement::AddAttributesFromDesc` · 005ea510 · legions 005ab0e0 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [x] `UIElement::MatchAttributes` · 005ea5c0 · legions 005ab190 — ac2-ui drag.rs; shortcut.rs drop
- [x] `UIElement::DragItem` · 005eab10 · legions 005ab6e0 — ac2-ui drag.rs drag_item: disallow_drag_out, then the owner's drag-out callback (DragCheck), the copy and start_drag_item
- [x] `UIElement::CatchDroppedItem` · 005eabd0 · legions 005ab7a0 — ac2-ui drag.rs catch_dropped_item: a container that takes drops asks its drag-drop callback (DragCheck; a refusal ends the catch), else stop_drag_item with the copy; other elements pass it to the parent
- [x] `UIElement::IsTooltipOn` · 005eac90 · legions 005ab860 — ac2-ui tooltip.rs
- [ ] `UIElement::IsActivatable` · 005eacb0 · legions 005ab880 — needs port (check): UIElement::IsActivatable behaviour; implemented area ac2-ui element.rs but uncited
- [ ] `UIElement::GetActivateOrder` · 005eacd0 · legions 005ab8a0 — needs port (check): UIElement::GetActivateOrder behaviour; implemented area ac2-ui element.rs but uncited
- [~] `UIElement::MoveTo` · 005eacf0 · legions 005ab8c0 — ac2-ui: no transitions, timed media, mouse, listeners or dirty rects; A8R8G8B8 blits only; HUD roots from SetupChildren's radar/selection/statusbar/toolbar
- [x] `UIElement::GetPositionFromState` · 005eae20 · legions 005ab9f0 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [~] `UIElement::SetVisible` · 005eafc0 · legions 005abb90 — ac2-ui: no transitions, timed media, mouse, listeners or dirty rects; A8R8G8B8 blits only; HUD roots from SetupChildren's radar/selection/statusbar/toolbar
- [x] `UIElement::StartTransition` · 005eb100 · legions 005abcd0 — ac2-ui media.rs / transition.rs
- [x] `UIElement::StartResizing` · 005eb1f0 · legions 005abdc0 — ac2-ui resize.rs: the grip's border_* attributes pick the edge (border_of, MouseDown), the drag moves the edges within min/max and the display (resized_box, MouseResizeElement), a resize_line outline is applied at the release (StopResizing); resize_tests.rs on the HUD strip and the chat strip layouts
- [x] `UIElement::StopResizing` · 005eb370 · legions 005abf40 — ac2-ui resize.rs: the grip's border_* attributes pick the edge (border_of, MouseDown), the drag moves the edges within min/max and the display (resized_box, MouseResizeElement), a resize_line outline is applied at the release (StopResizing); resize_tests.rs on the HUD strip and the chat strip layouts
- [x] `UIElement::MouseResizeElement` · 005eb430 · legions 005ac000 — ac2-ui resize.rs: the grip's border_* attributes pick the edge (border_of, MouseDown), the drag moves the edges within min/max and the display (resized_box, MouseResizeElement), a resize_line outline is applied at the release (StopResizing); resize_tests.rs on the HUD strip and the chat strip layouts
- [ ] `UIElement::AddImageToState` · 005eb9c0 · legions 005ac590 — needs port (check): UIElement::AddImageToState behaviour; implemented area ac2-ui element.rs but uncited
- [x] `UIElement::AddAttributesForState` · 005ebaf0 · legions 005ac6c0 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [x] `UIElement::InqLocation` · 005ebb80 · legions 005ac750 — ac2-ui window_locations.rs (inq_locations / apply_locations, clip conversions, drag bar movement); tests window_locations_tests.rs on real HUD
- [x] `UIElement::CheckLocation` · 005ebc80 · legions 005ac850 — ac2-ui window_locations.rs (inq_locations / apply_locations, clip conversions, drag bar movement); tests window_locations_tests.rs on real HUD
- [x] `UIElement::ApplyLocation` · 005ebcb0 · legions 005ac880 — ac2-ui window_locations.rs (inq_locations / apply_locations, clip conversions, drag bar movement); tests window_locations_tests.rs on real HUD
- [ ] `UIElement::DrawStart` · 005ebdd0 · legions 005ac9a0 — needs port (check): UIElement::DrawStart behaviour; implemented area ac2-ui element.rs but uncited
- [ ] `UIElement::DrawDone` · 005ebe10 · legions 005ac9e0 — needs port (check): UIElement::DrawDone behaviour; implemented area ac2-ui element.rs but uncited
- [x] `UIElement::GetPositionFromDesc` · 005ebe50 · legions 005aca20 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [-] `UIElement::~UIElement` · 005ec020 · legions 005acbf0 — element teardown: Drop of the arena slot (UiTree::delete)
- [x] `UIElement::SetAttribute_ID` · 005ec240 · legions 005ace10 — UIElement::SetAttribute_ID done: element.rs set_attribute_id
- [x] `UIElement::SetAttribute_InstanceID` · 005ec2e0 · legions 005aceb0 — UIElement::SetAttribute_InstanceID done: panel.rs set_instance_attribute
- [ ] `UIElement::SetAttribute_StringID` · 005ec380 · legions 005acf50 — needs port (check): UIElement::SetAttribute_StringID behaviour; implemented area ac2-ui element.rs but uncited
- [x] `UIElement::SetAttribute_Int` · 005ec420 · legions 005acff0 — UIElement::SetAttribute_Int done: element.rs set_int
- [x] `UIElement::SetAttribute_Float` · 005ec4d0 · legions 005ad0a0 — UIElement::SetAttribute_Float done: meter.rs set_float
- [~] `UIElement::SetAttribute_Bool` · 005ec570 · legions 005ad140 — ac2-ui: no transitions, timed media, mouse, listeners or dirty rects; A8R8G8B8 blits only; HUD roots from SetupChildren's radar/selection/statusbar/toolbar
- [ ] `UIElement::SetAttribute_DataID` · 005ec660 · legions 005ad230 — needs port (check): UIElement::SetAttribute_DataID behaviour; implemented area ac2-ui element.rs but uncited
- [~] `UIElement::SetAttribute_ASCII` · 005ec700 · legions 005ad2d0 — Unicode attributes are not in AttributeValue: a text element keeps its string in the element (UiTree::set_text, text.rs), not in a generic attribute table
- [~] `UIElement::SetAttribute_Unicode` · 005ec7a0 · legions 005ad370 — Unicode attributes are not in AttributeValue: a text element keeps its string in the element (UiTree::set_text, text.rs), not in a generic attribute table
- [x] `UIElement::GetOriginalPosition` · 005ec870 · legions 005ad440 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [x] `UIElement::GetParentOriginalPosition` · 005ec8a0 · legions 005ad470 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [x] `UIElement::ListenToMessage` · 005ec9d0 · legions 005ad5a0 — ac2-ui media.rs / transition.rs
- [x] `UIElement::UpdateSizeAndPosition` · 005eca20 · legions 005ad5f0 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [~] `UIElement::Update_Internal` · 005ecd60 · legions 005ad930 — ac2-ui: no transitions, timed media, mouse, listeners or dirty rects; A8R8G8B8 blits only; HUD roots from SetupChildren's radar/selection/statusbar/toolbar
- [~] `UIElement::ResizeTo` · 005ed150 · legions 005add20 — ac2-ui: no transitions, timed media, mouse, listeners or dirty rects; A8R8G8B8 blits only; HUD roots from SetupChildren's radar/selection/statusbar/toolbar
- [x] `UIElement::SwitchDescription` · 005ed2c0 · legions 005ade90 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [ ] `UIElement::Update` · 005ed330 · legions 005adf00 — needs port (check): UIElement::Update behaviour; implemented area ac2-ui element.rs but uncited
- [x] `UIElement::UpdateChild` · 005ed340 · legions 005adf10 — ac2-ui element.rs / place.rs / compose.rs: UI element tree and software compositor
- [~] `UIElement::SetState` · 005ed450 · legions 005ae020 — m_delOnTransStop (delete after the exit transition) not modelled

## `portal/client/src/uielement/system/uielementmanager.cpp.c` (12/69)

### Box2D
- [ ] `Box2D::Includes` · 005e3490 · legions 005a4250 — needs port: Box2D::Includes behaviour

### UIElementManager
- [x] `UIElementManager::CreateRootElement` · 005e3520 — ac2-ui element.rs: UiTree::create_root
- [x] `UIElementManager::RemoveRootElement` · 005e3580 · legions 005a4320 — ac2-ui element.rs: UiTree::delete (removes a root and its children)
- [~] `UIElementManager::SetCursor` · 005e35f0 — fades only (no rotate); sounds queued for an audio sink; cursor image not drawn by the client yet
- [x] `UIElementManager::UseDefaultCursor` · 005e3690 · legions 005a4420 — ac2-ui media.rs / transition.rs
- [ ] `UIElementManager::KeyReleaseEvent` · 005e36b0 — needs port (check): UIElementManager::KeyReleaseEvent behaviour; implemented area ac2-ui element.rs / mouse.rs but uncited
- [x] `UIElementManager::ClearTooltip` · 005e3740 · legions 005a4570 — ac2-ui tooltip.rs
- [x] `UIElementManager::CheckDragandDrop` · 005e37c0 · legions 005a45f0 — ac2-ui drag.rs; shortcut.rs drop
- [x] `UIElementManager::ClearDragandDrop` · 005e3820 · legions 005a4650 — ac2-ui drag.rs; shortcut.rs drop
- [~] `UIElementManager::ActivationAlert` · 005e38b0 · legions 005a46e0 — ac2-ui text_edit.rs: no selection, clipboard, IME, page up/down, cursor flashing or view scrolling

### SmartArray<CPartGroup_*>
- [-] `SmartArray<CPartGroup_*>::RemoveOrdered` · 005e3910 · legions 005a4740 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### UIElementManager
- [x] `UIElementManager::AddRootElement` · 005e3960 — ac2-ui element.rs: UiTree::create_root
- [ ] `UIElementManager::ChangingElementVisibility` · 005e3aa0 · legions 005a48d0 — needs port (check): UIElementManager::ChangingElementVisibility behaviour; implemented area ac2-ui element.rs / mouse.rs but uncited
- [~] `UIElementManager::MouseMoveEvent` · 005e3af0 · legions 005a4920 — ac2-ui mouse.rs / button.rs: no tooltips, cursors, drag and drop, moving/resizing, activation, hot clicks or listener registry (messages are taken by the caller)
- [~] `UIElementManager::UpdateMouse` · 005e3b70 · legions 005a49a0 — ac2-ui mouse.rs / button.rs: no tooltips, cursors, drag and drop, moving/resizing, activation, hot clicks or listener registry (messages are taken by the caller)
- [~] `UIElementManager::MouseDownEvent` · 005e3cc0 — ac2-ui text_edit.rs: no selection, clipboard, IME, page up/down, cursor flashing or view scrolling
- [-] `UIElementManager::CleanDeleteQueue` · 005e3de0 · legions 005a4c10 — Retail defers element deletion and listener removal to queues cleaned at UseTime so a handler can delete safely mid-message; UiTree deletes by arena index (UiTree::delete) and broadcasts into a queue the caller takes, so there is nothing to defer
- [~] `UIElementManager::StartTooltip` · 005e3e90 · legions 005a4cc0 — no right-button check; 32x32 cursor size assumed
- [x] `UIElementManager::StartDragandDrop` · 005e42e0 — ac2-ui drag.rs; shortcut.rs drop
- [x] `UIElementManager::StopDragandDrop` · 005e46d0 · legions 005a5500 — ac2-ui drag.rs; shortcut.rs drop
- [-] `UIElementManager::UnregisterActivatable` · 005e4720 · legions 005a5550 — Tab traversal is admin-only: UIElementManager::ActivateNext has no Legions 7160 counterpart (its only caller is admin's KeyPressEvent on action 0x40000017) and the activatable list has no other reader [vtable-checked: not a slot of any located Legions vtable]
- [-] `UIElementManager::ActivateNext` · 005e4740 — Tab traversal is admin-only: UIElementManager::ActivateNext has no Legions 7160 counterpart (its only caller is admin's KeyPressEvent on action 0x40000017) and the activatable list has no other reader [vtable-checked: not a slot of any located Legions vtable]
- [-] `UIElementManager::DrawDirtyRegions` · 005e4970 — Per-render-surface region bookkeeping (dirty rects, region info per UIObject): the client composes each root surface whole when UiTree::take_changed reports a change (compose.rs)
- [-] `UIElementManager::GetObjectRegionInfo` · 005e4b10 — Per-render-surface region bookkeeping (dirty rects, region info per UIObject): the client composes each root surface whole when UiTree::take_changed reports a change (compose.rs)
- [~] `UIElementManager::MouseUpEvent` · 005e4b60 — ac2-ui mouse.rs / button.rs: no tooltips, cursors, drag and drop, moving/resizing, activation, hot clicks or listener registry (messages are taken by the caller)
- [~] `UIElementManager::KeyPressEvent` · 005e4c70 · legions 005a4440 — ac2-ui mouse.rs / button.rs: no tooltips, cursors, drag and drop, moving/resizing, activation, hot clicks or listener registry (messages are taken by the caller)
- [x] `UIElementManager::ResetTooltip` · 005e4d20 · legions 005a5bb0 — ac2-ui tooltip.rs
- [-] `UIElementManager::AddtoListCleanQue` · 005e4d30 · legions 005a5bc0 — Retail defers element deletion and listener removal to queues cleaned at UseTime so a handler can delete safely mid-message; UiTree deletes by arena index (UiTree::delete) and broadcasts into a queue the caller takes, so there is nothing to defer
- [ ] `UIElementManager::HandleApply` · 005e4d70 · legions 005a5c00 — needs port (check): UIElementManager::HandleApply behaviour; implemented area ac2-ui element.rs / mouse.rs but uncited
- [~] `UIElementManager::StartInput` · 005e4e50 — ac2-ui mouse.rs / button.rs: no tooltips, cursors, drag and drop, moving/resizing, activation, hot clicks or listener registry (messages are taken by the caller)
- [~] `UIElementManager::StopInput` · 005e4fc0 — ac2-ui mouse.rs / button.rs: no tooltips, cursors, drag and drop, moving/resizing, activation, hot clicks or listener registry (messages are taken by the caller)
- [-] `UIElementManager::RegisterActivatable` · 005e50d0 · legions 005a5df0 — Tab traversal is admin-only: UIElementManager::ActivateNext has no Legions 7160 counterpart (its only caller is admin's KeyPressEvent on action 0x40000017) and the activatable list has no other reader [vtable-checked: not a slot of any located Legions vtable]

### Box2D
- [ ] `Box2D::Subtract` · 005e5150 · legions 005a5e70 — needs port: Box2D::Subtract behaviour

### UIElementManager
- [ ] `UIElementManager::MakeObjectTopmost` · 005e5260 — needs port (check): UIElementManager::MakeObjectTopmost behaviour; implemented area ac2-ui element.rs / mouse.rs but uncited

### UIObjectRegionInfo
- [ ] `UIObjectRegionInfo::AddDirtyRect` · 005e53a0 · legions 005a60c0 — needs port: UIObjectRegionInfo::AddDirtyRect behaviour

### UIElementManager
- [-] `UIElementManager::MakeRootDirtyHere` · 005e5510 — Per-render-surface region bookkeeping (dirty rects, region info per UIObject): the client composes each root surface whole when UiTree::take_changed reports a change (compose.rs)
- [-] `UIElementManager::ReleaseLayoutDesc` · 005e5830 · legions 005a6580 — Manager construction, teardown and tables: UiTree::new, Arc<LayoutDesc> (released on drop) and element kinds dispatched by match replace Init, Cleanup, ReleaseLayoutDesc, RegisterElementClass and the ctor/dtor
- [ ] `UIElementManager::RefreshEvent` · 005e5870 · legions 005a65c0 — needs port (check): UIElementManager::RefreshEvent; client_locations.rs cite is a module doc
- [-] `UIElementManager::AddElementToDeleteQueue` · 005e5950 · legions 005a66a0 — Retail defers element deletion and listener removal to queues cleaned at UseTime so a handler can delete safely mid-message; UiTree deletes by arena index (UiTree::delete) and broadcasts into a queue the caller takes, so there is nothing to defer
- [~] `UIElementManager::CreateRootElementByDataID` · 005e59b0 — ac2-ui: no transitions, timed media, mouse, listeners or dirty rects; A8R8G8B8 blits only; HUD roots from SetupChildren's radar/selection/statusbar/toolbar
- [~] `UIElementManager::CreateElement` · 005e5a80 — ac2-ui: no transitions, timed media, mouse, listeners or dirty rects; A8R8G8B8 blits only; HUD roots from SetupChildren's radar/selection/statusbar/toolbar
- [-] `UIElementManager::RemoveAndDeleteRootElement` · 005e5bf0 · legions 005a6930 — Retail defers element deletion and listener removal to queues cleaned at UseTime so a handler can delete safely mid-message; UiTree deletes by arena index (UiTree::delete) and broadcasts into a queue the caller takes, so there is nothing to defer
- [~] `UIElementManager::UnregisterForMessage` · 005e5c30 · legions 005a6970 — Messages are UiMessage values queued by UiTree::broadcast (mouse.rs) and heard by the panels' frame_pass/message_pass over the tree's message list; no per-element listener registry, so a listener cannot register for one element, object or global id
- [-] `UIElementManager::CleanElementListCleanQue` · 005e5cb0 · legions 005a69f0 — Retail defers element deletion and listener removal to queues cleaned at UseTime so a handler can delete safely mid-message; UiTree deletes by arena index (UiTree::delete) and broadcasts into a queue the caller takes, so there is nothing to defer
- [~] `UIElementManager::RegisterForObjectMessage` · 005e5ea0 — Messages are UiMessage values queued by UiTree::broadcast (mouse.rs) and heard by the panels' frame_pass/message_pass over the tree's message list; no per-element listener registry, so a listener cannot register for one element, object or global id
- [~] `UIElementManager::UnregisterForObjectMessage` · 005e5fd0 · legions 005a6d10 — Messages are UiMessage values queued by UiTree::broadcast (mouse.rs) and heard by the panels' frame_pass/message_pass over the tree's message list; no per-element listener registry, so a listener cannot register for one element, object or global id
- [-] `UIElementManager::DeleteListenerFromElementCleanQue` · 005e6050 · legions 005a6d90 — Retail defers element deletion and listener removal to queues cleaned at UseTime so a handler can delete safely mid-message; UiTree deletes by arena index (UiTree::delete) and broadcasts into a queue the caller takes, so there is nothing to defer
- [-] `UIElementManager::DeleteListenerFromCleanQue` · 005e6360 · legions 005a70a0 — Retail defers element deletion and listener removal to queues cleaned at UseTime so a handler can delete safely mid-message; UiTree deletes by arena index (UiTree::delete) and broadcasts into a queue the caller takes, so there is nothing to defer
- [-] `UIElementManager::CleanObjectListCleanQue` · 005e65d0 · legions 005a7310 — Retail defers element deletion and listener removal to queues cleaned at UseTime so a handler can delete safely mid-message; UiTree deletes by arena index (UiTree::delete) and broadcasts into a queue the caller takes, so there is nothing to defer
- [~] `UIElementManager::RegisterForGlobalMessage` · 005e6720 · legions 005a7460 — Messages are UiMessage values queued by UiTree::broadcast (mouse.rs) and heard by the panels' frame_pass/message_pass over the tree's message list; no per-element listener registry, so a listener cannot register for one element, object or global id
- [~] `UIElementManager::UnregisterForGlobalMessage` · 005e6800 · legions 005a7540 — Messages are UiMessage values queued by UiTree::broadcast (mouse.rs) and heard by the panels' frame_pass/message_pass over the tree's message list; no per-element listener registry, so a listener cannot register for one element, object or global id
- [-] `UIElementManager::CleanGlobalListCleanQue` · 005e6870 · legions 005a75b0 — Retail defers element deletion and listener removal to queues cleaned at UseTime so a handler can delete safely mid-message; UiTree deletes by arena index (UiTree::delete) and broadcasts into a queue the caller takes, so there is nothing to defer
- [~] `UIElementManager::GetElement` · 005e69f0 · legions 005a7730 — UiTree::child_recursive looks an id up under a root; there is no manager-wide id table for the first match across roots
- [x] `UIElementManager::GetUniqueMatchElement` · 005e6a40 · legions 005a7780 — ac2-ui element.rs: UiTree::child_recursive(root, id) finds an element by id under its root
- [-] `UIElementManager::DeletingElement` · 005e6ab0 · legions 005a77f0 — Retail defers element deletion and listener removal to queues cleaned at UseTime so a handler can delete safely mid-message; UiTree deletes by arena index (UiTree::delete) and broadcasts into a queue the caller takes, so there is nothing to defer
- [-] `UIElementManager::RegisterElementClass` · 005e6bb0 — Manager construction, teardown and tables: UiTree::new, Arc<LayoutDesc> (released on drop) and element kinds dispatched by match replace Init, Cleanup, ReleaseLayoutDesc, RegisterElementClass and the ctor/dtor

### (free functions)
- [ ] `RefreshEvent_g` · 005e6be0 — needs port: RefreshEvent_g behaviour

### UIElementManager
- [-] `UIElementManager::DeleteUIObjectRegionInfo` · 005e6c00 — Per-render-surface region bookkeeping (dirty rects, region info per UIObject): the client composes each root surface whole when UiTree::take_changed reports a change (compose.rs)
- [-] `UIElementManager::Init` · 005e6f70 — Manager construction, teardown and tables: UiTree::new, Arc<LayoutDesc> (released on drop) and element kinds dispatched by match replace Init, Cleanup, ReleaseLayoutDesc, RegisterElementClass and the ctor/dtor
- [~] `UIElementManager::BroadcastMessage` · 005e7130 — Messages are UiMessage values queued by UiTree::broadcast (mouse.rs) and heard by the panels' frame_pass/message_pass over the tree's message list; no per-element listener registry, so a listener cannot register for one element, object or global id
- [~] `UIElementManager::BroadcastGlobalMessage` · 005e7280 — Messages are UiMessage values queued by UiTree::broadcast (mouse.rs) and heard by the panels' frame_pass/message_pass over the tree's message list; no per-element listener registry, so a listener cannot register for one element, object or global id
- [-] `UIElementManager::UIElementManager` · 005e7370 · legions 005a8000 — Manager construction, teardown and tables: UiTree::new, Arc<LayoutDesc> (released on drop) and element kinds dispatched by match replace Init, Cleanup, ReleaseLayoutDesc, RegisterElementClass and the ctor/dtor

### MoveToManager
- [ ] `MoveToManager::ReachedPersistentTarget` · 005e7590 · legions 005a8230 — needs port (check): MoveToManager::ReachedPersistentTarget behaviour; implemented area ac2-ui navigation.rs but uncited
- [x] `MoveToManager::IsMovingTo` · 005e75a0 — ac2-physics moveto (slice 3)

### UIElementManager
- [~] `UIElementManager::UseTime` · 005e75b0 · legions 005a8240 — tooltip owner/timing and default cursor ported; drag, resize and object regions not
- [-] `UIElementManager::Cleanup` · 005e7780 · legions 005a83e0 — Manager construction, teardown and tables: UiTree::new, Arc<LayoutDesc> (released on drop) and element kinds dispatched by match replace Init, Cleanup, ReleaseLayoutDesc, RegisterElementClass and the ctor/dtor
- [~] `UIElementManager::RegisterForMessage` · 005e7b30 — Messages are UiMessage values queued by UiTree::broadcast (mouse.rs) and heard by the panels' frame_pass/message_pass over the tree's message list; no per-element listener registry, so a listener cannot register for one element, object or global id
- [-] `UIElementManager::~UIElementManager` · 005e7cd0 · legions 005a8930 — Manager construction, teardown and tables: UiTree::new, Arc<LayoutDesc> (released on drop) and element kinds dispatched by match replace Init, Cleanup, ReleaseLayoutDesc, RegisterElementClass and the ctor/dtor
- [-] `UIElementManager::CreateUIObjectRegionInfo` · 005e7e70 · legions 005a8ad0 — Per-render-surface region bookkeeping (dirty rects, region info per UIObject): the client composes each root surface whole when UiTree::take_changed reports a change (compose.rs)

## `portal/client/src/uielement/system/uilistener.cpp.c` (0/12)

### UIListener
- [-] `UIListener::GetListenerElementNumberRegistered` · 005e7f80 · legions 005a8be0 — per-listener registration counts and tables: UiTree::broadcast queues UiMessage values and the panels hear them from the tree's message list, so there is no listener object (see UIElementManager::RegisterForMessage)
- [-] `UIListener::GetListenerNumberRegistered` · 005e7fd0 · legions 005a8c30 — per-listener registration counts and tables: UiTree::broadcast queues UiMessage values and the panels hear them from the tree's message list, so there is no listener object (see UIElementManager::RegisterForMessage)
- [-] `UIListener::UnregisterForAllGlobalMessages` · 005e8010 · legions 005a8c70 — per-listener registration counts and tables: UiTree::broadcast queues UiMessage values and the panels hear them from the tree's message list, so there is no listener object (see UIElementManager::RegisterForMessage)
- [-] `UIListener::IncrementMessage` · 005e8110 · legions 005a8d70 — per-listener registration counts and tables: UiTree::broadcast queues UiMessage values and the panels hear them from the tree's message list, so there is no listener object (see UIElementManager::RegisterForMessage)
- [-] `UIListener::DecrementMessage` · 005e8180 · legions 005a8de0 — per-listener registration counts and tables: UiTree::broadcast queues UiMessage values and the panels hear them from the tree's message list, so there is no listener object (see UIElementManager::RegisterForMessage)
- [-] `UIListener::UnregisterForAllMessages` · 005e81f0 · legions 005a8e50 — per-listener registration counts and tables: UiTree::broadcast queues UiMessage values and the panels hear them from the tree's message list, so there is no listener object (see UIElementManager::RegisterForMessage)
- [-] `UIListener::DecrementElementMessage` · 005e8340 · legions 005a8fa0 — per-listener registration counts and tables: UiTree::broadcast queues UiMessage values and the panels hear them from the tree's message list, so there is no listener object (see UIElementManager::RegisterForMessage)
- [-] `UIListener::UIListener` · 005e8510 · legions 005a9170 — per-listener registration counts and tables: UiTree::broadcast queues UiMessage values and the panels hear them from the tree's message list, so there is no listener object (see UIElementManager::RegisterForMessage)
- [-] `UIListener::~UIListener` · 005e8560 · legions 005a91c0 — per-listener registration counts and tables: UiTree::broadcast queues UiMessage values and the panels hear them from the tree's message list, so there is no listener object (see UIElementManager::RegisterForMessage)
- [-] `UIListener::IncrementElementMessage` · 005e89b0 · legions 005a9610 — per-listener registration counts and tables: UiTree::broadcast queues UiMessage values and the panels hear them from the tree's message list, so there is no listener object (see UIElementManager::RegisterForMessage)
- [-] `UIListener::RegisterForMessage` · 005e8a60 · legions 005a96c0 — per-listener registration counts and tables: UiTree::broadcast queues UiMessage values and the panels hear them from the tree's message list, so there is no listener object (see UIElementManager::RegisterForMessage)
- [-] `UIListener::RegisterForGlobalMessage` · 005e8ae0 · legions 005a9740 — per-listener registration counts and tables: UiTree::broadcast queues UiMessage values and the panels hear them from the tree's message list, so there is no listener object (see UIElementManager::RegisterForMessage)

## `portal/client/src/uielement/types/text/glyph.cpp.c` (3/7)

### Glyph
- [ ] `Glyph::Glyph` · 005fcd50 · legions 005bd880 — needs port (check): Glyph::Glyph overload (newline width); text.rs new() cites it, overload ambiguous
- [x] `Glyph::Glyph` · 005fcd70 · legions 005bd8a0 — ac2-ui text.rs / text_attr.rs: UIElement_Text layout and drawing
- [-] `Glyph::~Glyph` · 005fcdf0 · legions 005bd920 — ctor/dtor/operator= of plain data; Rust struct construction and Drop replace it
- [-] `Glyph::operator=` · 005fce10 · legions 005bd940 — ctor/dtor/operator= of plain data; Rust struct construction and Drop replace it
- [x] `Glyph::IsWhiteSpace` · 005fce80 · legions 005bd9b0 — ac2-ui text.rs / text_attr.rs: UIElement_Text layout and drawing
- [x] `Glyph::IsNewLine` · 005fceb0 · legions 005bd9e0 — ac2-ui text.rs / text_attr.rs: UIElement_Text layout and drawing
- [ ] `Glyph::Glyph` · 005fcec0 · legions 005bd9f0 — needs port (check): Glyph::Glyph overload (newline width); text.rs new() cites it, overload ambiguous

## `portal/client/src/uielement/types/text/glyphlist.cpp.c` (18/24)

### LinebreakUtils
- [x] `LinebreakUtils::IsWhiteSpace` · 005fbe40 · legions 005bc970 — ac2-ui text.rs / text_attr.rs: UIElement_Text layout and drawing
- [x] `LinebreakUtils::IsNonBeginningChar` · 005fbe60 · legions 005bc990 — ac2-ui text.rs / text_attr.rs: UIElement_Text layout and drawing
- [x] `LinebreakUtils::IsNonEndingChar` · 005fbed0 · legions 005bca00 — ac2-ui text.rs / text_attr.rs: UIElement_Text layout and drawing
- [x] `LinebreakUtils::IsEastAsianChar` · 005fbef0 · legions 005bca20 — ac2-ui text.rs / text_attr.rs: UIElement_Text layout and drawing
- [x] `LinebreakUtils::CanBreakLineAt` · 005fbf30 · legions 005bca60 — ac2-ui text.rs / text_attr.rs: UIElement_Text layout and drawing

### GlyphList
- [x] `GlyphList::FindCurrentLine` · 005fbfb0 · legions 005bcae0 — ac2-ui glyph_find.rs: TextState::find_current_line / find_line_from_y (past the bottom = last line, checked against the machine code) / find_next_line / find_prev_line / find_next_word / find_prev_word / find_pixels_from_pos / find_pos_from_line_and_pixels; GlyphLine heights and widths are GlyphLine::height / width
- [x] `GlyphList::FindLineFromY` · 005fc040 · legions 005bcb70 — ac2-ui glyph_find.rs: TextState::find_current_line / find_line_from_y (past the bottom = last line, checked against the machine code) / find_next_line / find_prev_line / find_next_word / find_prev_word / find_pixels_from_pos / find_pos_from_line_and_pixels; GlyphLine heights and widths are GlyphLine::height / width
- [x] `GlyphList::FindNextLine` · 005fc0d0 · legions 005bcc00 — ac2-ui glyph_find.rs: TextState::find_current_line / find_line_from_y (past the bottom = last line, checked against the machine code) / find_next_line / find_prev_line / find_next_word / find_prev_word / find_pixels_from_pos / find_pos_from_line_and_pixels; GlyphLine heights and widths are GlyphLine::height / width
- [x] `GlyphList::FindPrevLine` · 005fc130 · legions 005bcc60 — ac2-ui glyph_find.rs: TextState::find_current_line / find_line_from_y (past the bottom = last line, checked against the machine code) / find_next_line / find_prev_line / find_next_word / find_prev_word / find_pixels_from_pos / find_pos_from_line_and_pixels; GlyphLine heights and widths are GlyphLine::height / width
- [x] `GlyphList::GetGlyphLineHeight` · 005fc190 · legions 005bccc0 — ac2-ui glyph_find.rs: TextState::find_current_line / find_line_from_y (past the bottom = last line, checked against the machine code) / find_next_line / find_prev_line / find_next_word / find_prev_word / find_pixels_from_pos / find_pos_from_line_and_pixels; GlyphLine heights and widths are GlyphLine::height / width
- [x] `GlyphList::GetGlyphLineWidth` · 005fc1d0 · legions 005bcd00 — ac2-ui glyph_find.rs: TextState::find_current_line / find_line_from_y (past the bottom = last line, checked against the machine code) / find_next_line / find_prev_line / find_next_word / find_prev_word / find_pixels_from_pos / find_pos_from_line_and_pixels; GlyphLine heights and widths are GlyphLine::height / width
- [x] `GlyphList::FindNextWord` · 005fc240 · legions 005bcd70 — ac2-ui glyph_find.rs: TextState::find_current_line / find_line_from_y (past the bottom = last line, checked against the machine code) / find_next_line / find_prev_line / find_next_word / find_prev_word / find_pixels_from_pos / find_pos_from_line_and_pixels; GlyphLine heights and widths are GlyphLine::height / width
- [x] `GlyphList::FindPrevWord` · 005fc2b0 · legions 005bcde0 — ac2-ui glyph_find.rs: TextState::find_current_line / find_line_from_y (past the bottom = last line, checked against the machine code) / find_next_line / find_prev_line / find_next_word / find_prev_word / find_pixels_from_pos / find_pos_from_line_and_pixels; GlyphLine heights and widths are GlyphLine::height / width
- [x] `GlyphList::FindPixelsFromPos` · 005fc320 · legions 005bce50 — ac2-ui glyph_find.rs: TextState::find_current_line / find_line_from_y (past the bottom = last line, checked against the machine code) / find_next_line / find_prev_line / find_next_word / find_prev_word / find_pixels_from_pos / find_pos_from_line_and_pixels; GlyphLine heights and widths are GlyphLine::height / width
- [x] `GlyphList::FindPosFromLineAndPixels` · 005fc3a0 · legions 005bced0 — ac2-ui glyph_find.rs: TextState::find_current_line / find_line_from_y (past the bottom = last line, checked against the machine code) / find_next_line / find_prev_line / find_next_word / find_prev_word / find_pixels_from_pos / find_pos_from_line_and_pixels; GlyphLine heights and widths are GlyphLine::height / width
- [ ] `GlyphList::FindXYFromPosition` · 005fc5f0 · legions 005bd120 — needs port (check): GlyphList::FindXYFromPosition behaviour; implemented area ac2-ui text.rs but uncited
- [x] `GlyphList::Flush` · 005fc6d0 · legions 005bd200 — ac2-ui text.rs: TextState::clear
- [x] `GlyphList::Delete` · 005fc710 · legions 005bd240 — ac2-ui text.rs: TextState::delete_range (both ends clamped, nothing removed unless a glyph lies between, a cursor after the start moves back by the removed count)
- [~] `GlyphList::AddText` · 005fc7e0 · legions 005bd310 — ac2-ui text.rs: no editing, cursor, selection, scrolling, fallback fonts, string variables or server downloads; A8R8G8B8 surfaces
- [-] `GlyphList::GlyphList` · 005fc930 · legions 005bd460 — ctor/dtor/operator= of plain data; Rust struct construction and Drop replace it
- [-] `GlyphList::~GlyphList` · 005fc960 · legions 005bd490 — ctor/dtor/operator= of plain data; Rust struct construction and Drop replace it
- [ ] `GlyphList::InqText` · 005fc9f0 · legions 005bd520 — needs port (check): GlyphList::InqText behaviour; implemented area ac2-ui text.rs but uncited
- [x] `GlyphList::Recalculate` · 005fcaf0 · legions 005bd620 — ac2-ui text.rs / text_attr.rs: UIElement_Text layout and drawing
- [ ] `GlyphList::InqText` · 005fcd20 · legions 005bd850 — needs port (check): GlyphList::InqText behaviour; implemented area ac2-ui text.rs but uncited

## `portal/client/src/uielement/types/text/stringdownload.cpp.c` (0/4)

### StringDownload
- [ ] `StringDownload::OnContextFinished` · 005fcef0 — needs port (check): StringDownload::OnContextFinished behaviour; implemented area ac2-ui text_data.rs but uncited
- [ ] `StringDownload::Init` · 005fcf50 · legions 005bda80 — needs port (check): StringDownload::Init behaviour; implemented area ac2-ui text_data.rs but uncited
- [ ] `StringDownload::OnTopLevelRequestFinished` · 005fd150 — needs port (check): StringDownload::OnTopLevelRequestFinished behaviour; implemented area ac2-ui text_data.rs but uncited
- [-] `StringDownload::StringDownload` · 005fd210 · legions 005bdd40 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own

## `portal/client/src/uielement/types/text/uielement_text.cpp.c` (47/92)

### (free functions)
- [x] `NumberInputFilter` · 005ed7c0 — ac2-ui stack.rs

### UIElement_Text
- [x] `UIElement_Text::GetScrollbar` · 005ed7d0 · legions 005ae3a0 — named_scrollbar (scrollbar.rs): GetElement without unique_msg finds a scroll bar in any root, GetUniqueMatchElement with it in the owner's root; text_scroll.rs, list_scroll.rs and tree_scroll.rs bind through it. Real data: scroll_coverage_tests.rs
- [x] `UIElement_Text::HandleMouseWheel` · 005ed840 · legions 005ae410 — wheel: the element under the pointer hands the turn to its vertical bar (text_scroll.rs text_wheel, tree_scroll.rs tree_wheel, list_scroll.rs list_wheel); the bar steps its stop, or the lists, texts and trees it drives one line (scrollbar.rs scrollbar_mouse_wheel). Real-data: scroll_coverage_tests.rs over every layout; synthetic: list_scroll_tests.rs
- [x] `UIElement_Text::CalcJustification` · 005ed8b0 · legions 005ae480 — ac2-ui text.rs / text_attr.rs: UIElement_Text layout and drawing
- [ ] `UIElement_Text::IMETurnOn` · 005ed950 · legions 005ae540 — needs port (check): UIElement_Text::IMETurnOn behaviour; implemented area ac2-ui text.rs / text_attr.rs / text_scroll.rs but uncited
- [~] `UIElement_Text::Activate` · 005ed9a0 · legions 005ae590 — ac2-ui text_edit.rs: no selection, clipboard, IME, page up/down, cursor flashing or view scrolling
- [~] `UIElement_Text::Deactivate` · 005eda10 · legions 005ae600 — ac2-ui text_edit.rs: no selection, clipboard, IME, page up/down, cursor flashing or view scrolling
- [ ] `UIElement_Text::CleanupStringDownloads` · 005eda80 · legions 005ae670 — needs port (check): UIElement_Text::CleanupStringDownloads behaviour; implemented area ac2-ui text.rs / text_attr.rs / text_scroll.rs but uncited
- [x] `UIElement_Text::SetAtrb_Editable` · 005edaf0 · legions 005ae6e0 — ac2-ui input.rs / text_edit.rs: typing into text elements
- [x] `UIElement_Text::DetermineMarginValues` · 005edb30 · legions 005ae720 — ac2-ui text.rs / text_attr.rs: UIElement_Text layout and drawing
- [ ] `UIElement_Text::SetAtrb_OneLine` · 005edbf0 · legions 005ae7e0 — needs port (check): UIElement_Text::SetAtrb_OneLine behaviour; implemented area ac2-ui text.rs / text_attr.rs / text_scroll.rs but uncited
- [~] `UIElement_Text::GetPercentageV` · 005edc40 · legions 005ae830 — vertical share of the scrollable range paper - (y1 - y0 + 1) (text_scroll.rs / tree_scroll.rs); horizontal share not modelled
- [ ] `UIElement_Text::GetPercentageH` · 005edcc0 · legions 005ae8b0 — needs port (check): UIElement_Text::GetPercentageH behaviour; implemented area ac2-ui text.rs / text_attr.rs / text_scroll.rs but uncited
- [x] `UIElement_Text::ClampPosition` · 005edd40 · legions 005ae930 — ac2-ui text_edit.rs: UiTree::set_cursor clamps the position to 0..=len
- [x] `UIElement_Text::MoveUpDown` · 005edd80 · legions 005ae970 — ac2-ui glyph_find.rs: TextState::move_begin_end_line (the penultimate and last lines end at the end of the text: FindNextLine overwrites the line the last-line test reads), move_up_down (pixel column kept across lines, justification offsets, stays on the first/last line), position_at
- [x] `UIElement_Text::MoveBeginEndLine` · 005edf30 · legions 005aeb20 — ac2-ui glyph_find.rs: TextState::move_begin_end_line (the penultimate and last lines end at the end of the text: FindNextLine overwrites the line the last-line test reads), move_up_down (pixel column kept across lines, justification offsets, stays on the first/last line), position_at
- [x] `UIElement_Text::SetSelecting` · 005edfd0 · legions 005aebc0 — ac2-ui text_edit.rs, text_clip.rs: selection, drag, Ctrl+A/C/X/V through the tree's clipboard (text_clipboard.rs; ac2-client clipboard.rs over arboard); password fields refuse copy and cut; tests text_clip_tests.rs, real_data_tests.rs chat entry
- [x] `UIElement_Text::SetSelectionStart` · 005ee010 · legions 005aec00 — ac2-ui text_edit.rs, text_clip.rs: selection, drag, Ctrl+A/C/X/V through the tree's clipboard (text_clipboard.rs; ac2-client clipboard.rs over arboard); password fields refuse copy and cut; tests text_clip_tests.rs, real_data_tests.rs chat entry
- [x] `UIElement_Text::SetSelectionEnd` · 005ee070 · legions 005aec60 — ac2-ui text_edit.rs, text_clip.rs: selection, drag, Ctrl+A/C/X/V through the tree's clipboard (text_clipboard.rs; ac2-client clipboard.rs over arboard); password fields refuse copy and cut; tests text_clip_tests.rs, real_data_tests.rs chat entry
- [x] `UIElement_Text::GetSelection` · 005ee0d0 · legions 005aecc0 — ac2-ui text_edit.rs, text_clip.rs: selection, drag, Ctrl+A/C/X/V through the tree's clipboard (text_clipboard.rs; ac2-client clipboard.rs over arboard); password fields refuse copy and cut; tests text_clip_tests.rs, real_data_tests.rs chat entry
- [x] `UIElement_Text::GetTotalLines` · 005ee130 · legions 005aed20 — ac2-ui text_attr.rs: UiTree::text_line_count

### GlyphLine
- [-] `GlyphLine::GlyphLine` · 005ee140 — ctor/dtor/operator= of plain data; Rust struct construction and Drop replace it

### UIElement_Text
- [-] `UIElement_Text::~UIElement_Text` · 005ee160 · legions 005aed50 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own
- [x] `UIElement_Text::ClearAllText` · 005ee340 · legions 005aef30 — ac2-ui text.rs / text_attr.rs: UIElement_Text layout and drawing
- [x] `UIElement_Text::SelectAll` · 005ee3a0 · legions 005aef90 — ac2-ui text_edit.rs, text_clip.rs: selection, drag, Ctrl+A/C/X/V through the tree's clipboard (text_clipboard.rs; ac2-client clipboard.rs over arboard); password fields refuse copy and cut; tests text_clip_tests.rs, real_data_tests.rs chat entry
- [x] `UIElement_Text::Deselect` · 005ee400 · legions 005aeff0 — ac2-ui text_edit.rs, text_clip.rs: selection, drag, Ctrl+A/C/X/V through the tree's clipboard (text_clipboard.rs; ac2-client clipboard.rs over arboard); password fields refuse copy and cut; tests text_clip_tests.rs, real_data_tests.rs chat entry
- [x] `UIElement_Text::GetText` · 005ee420 · legions 005af010 — UIElement_Text::GetText done: element.rs text
- [x] `UIElement_Text::GetSelectedText` · 005ee490 — ac2-ui text_edit.rs, text_clip.rs: selection, drag, Ctrl+A/C/X/V through the tree's clipboard (text_clipboard.rs; ac2-client clipboard.rs over arboard); password fields refuse copy and cut; tests text_clip_tests.rs, real_data_tests.rs chat entry
- [~] `UIElement_Text::UpdateScrollbar` · 005ee550 · legions 005af140 — vertical bar only (text_scroll.rs): position x (paper - (y1 - y0 + 1)) as admin 7148 0x005F0790 and 0x005EDC40 read; horizontal bar, widget size (thumb keeps its own), page messages from the bar's track not modelled. The bar is found across the UI (named_scrollbar)
- [x] `UIElement_Text::ResizeTo` · 005ee6a0 · legions 005af290 — ac2-ui text.rs / text_attr.rs: UIElement_Text layout and drawing
- [ ] `UIElement_Text::DetermineFont` · 005ee6f0 · legions 005af2e0 — needs port (check): UIElement_Text::DetermineFont behaviour; implemented area ac2-ui text.rs / text_attr.rs / text_scroll.rs but uncited
- [~] `UIElement_Text::SetCursorPosition` · 005ee850 · legions 005af3e0 — ac2-ui text_edit.rs / text_clip.rs / text.rs: drag and Shift selection, clipboard keys; not modelled: wheel on text (HandleMouseWheel), page up/down, IME, fallback fonts; clipboard is arboard, not CF_UNICODETEXT (ac2-client clipboard.rs)
- [~] `UIElement_Text::UIElement_Text` · 005eea40 · legions 005af5d0 — ac2-ui text.rs: no editing, cursor, selection, scrolling, fallback fonts, string variables or server downloads; A8R8G8B8 surfaces
- [~] `UIElement_Text::SetAttribute_Int` · 005eec20 · legions 005af780 — ac2-ui text.rs: no editing, cursor, selection, scrolling, fallback fonts, string variables or server downloads; A8R8G8B8 surfaces
- [-] `UIElement_Text::Create` · 005eed10 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own
- [~] `UIElement_Text::InqSizewMargins` · 005eed40 · legions 005af8a0 — ac2-ui reticle.rs: no off-screen indicator, meters, radar-look colour or quantity; name plates and damage text not ported
- [~] `UIElement_Text::UpdateScrollbars` · 005eef10 · legions 005afa70 — vertical bar only (text_scroll.rs): position x (paper - (y1 - y0 + 1)) as admin 7148 0x005F0790 and 0x005EDC40 read; horizontal bar, widget size (thumb keeps its own), page messages from the bar's track not modelled. The bar is found across the UI (named_scrollbar)

### SmartArray<GlyphLine>
- [-] `SmartArray<GlyphLine>::operator=` · 005eef60 · legions 005afac0 — container/STL internals (template Pack/StreamPack/iterator/operator); Rust Vec, HashMap and String replace them

### UIElement_Text
- [ ] `UIElement_Text::SetAttribute_ID` · 005eefd0 · legions 005afb30 — needs port (check): UIElement_Text::SetAttribute_ID behaviour; implemented area ac2-ui text.rs / text_attr.rs / text_scroll.rs but uncited
- [-] `UIElement_Text::Register` · 005ef100 · legions 005afc60 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own
- [ ] `UIElement_Text::Initialized` · 005ef120 · legions 005afc80 — needs port (check): UIElement_Text::Initialized behaviour; implemented area ac2-ui text.rs / text_attr.rs / text_scroll.rs but uncited
- [x] `UIElement_Text::Copy` · 005ef170 — ac2-ui text_clip.rs copy_selection: copies the selection to the OS clipboard via text_clipboard.rs; refused for password fields (no retail attribute, modern flag)
- [~] `UIElement_Text::KeepInBounds` · 005ef1c0 · legions 005afd20 — ac2-ui text.rs: no editing, cursor, selection, scrolling, fallback fonts, string variables or server downloads; A8R8G8B8 surfaces
- [~] `UIElement_Text::PostBlit` · 005ef2f0 · legions 005afe50 — ac2-ui text.rs: no editing, cursor, selection, scrolling, fallback fonts, string variables or server downloads; A8R8G8B8 surfaces
- [ ] `UIElement_Text::SetAtrb_CenterVertically` · 005ef8d0 · legions 005b0430 — needs port (check): UIElement_Text::SetAtrb_CenterVertically behaviour; implemented area ac2-ui text.rs / text_attr.rs / text_scroll.rs but uncited
- [~] `UIElement_Text::SetFontNum` · 005ef940 · legions 005b04a0 — ac2-ui text.rs: no editing, cursor, selection, scrolling, fallback fonts, string variables or server downloads; A8R8G8B8 surfaces
- [x] `UIElement_Text::SetFontNumAndColor` · 005efc10 · legions 005b0750 — ac2-ui text.rs / text_attr.rs: UIElement_Text layout and drawing
- [~] `UIElement_Text::RecalculateGlyphList` · 005eff20 · legions 005b0a50 — ac2-ui text.rs: no editing, cursor, selection, scrolling, fallback fonts, string variables or server downloads; A8R8G8B8 surfaces
- [x] `UIElement_Text::SetViewY` · 005f0080 · legions 005b0bb0 — UIElement_Text::SetViewY done: text_attr.rs set_text_view_y
- [x] `UIElement_Text::DrawStart` · 005f00a0 · legions 005b0bd0 — ac2-ui text.rs / text_attr.rs: UIElement_Text layout and drawing
- [x] `UIElement_Text::DeterminePositionFromXY` · 005f00b0 · legions 005b0be0 — ac2-ui glyph_find.rs: TextState::move_begin_end_line (the penultimate and last lines end at the end of the text: FindNextLine overwrites the line the last-line test reads), move_up_down (pixel column kept across lines, justification offsets, stays on the first/last line), position_at
- [~] `UIElement_Text::SetCursorPosition` · 005f01d0 · legions 005b0d00 — ac2-ui text_edit.rs / text_clip.rs / text.rs: drag and Shift selection, clipboard keys; not modelled: wheel on text (HandleMouseWheel), page up/down, IME, fallback fonts; clipboard is arboard, not CF_UNICODETEXT (ac2-client clipboard.rs)
- [~] `UIElement_Text::MouseDown` · 005f0200 · legions 005b0d30 — ac2-ui text_edit.rs / text_clip.rs / text.rs: drag and Shift selection, clipboard keys; not modelled: wheel on text (HandleMouseWheel), page up/down, IME, fallback fonts; clipboard is arboard, not CF_UNICODETEXT (ac2-client clipboard.rs)
- [x] `UIElement_Text::MouseUp` · 005f02e0 · legions 005b0e10 — ac2-ui text_edit.rs, text_clip.rs: selection, drag, Ctrl+A/C/X/V through the tree's clipboard (text_clipboard.rs; ac2-client clipboard.rs over arboard); password fields refuse copy and cut; tests text_clip_tests.rs, real_data_tests.rs chat entry
- [x] `UIElement_Text::Global_MouseMove` · 005f0370 · legions 005b0ea0 — ac2-ui text_clip.rs: retail moves the cursor; drag auto-scroll past the top or bottom of multi-line text is modern (text_drag_auto_scroll on the global loop)
- [x] `UIElement_Text::DetermineAndSetNewLine` · 005f03e0 · legions 005b0f10 — UIElement_Text::DetermineAndSetNewLine done: text_scroll.rs text_line_step
- [x] `UIElement_Text::DetermineAndSetNewPage` · 005f04f0 · legions 005b1020 — ac2-ui text_attr.rs: UiTree::text_new_page (view moves an element extent, kept in bounds by the layout, bars updated) and page_text; MoveCursor PAGEUP/PAGEDOWN in text_edit.rs
- [ ] `UIElement_Text::ScrollToPosition` · 005f0590 · legions 005b10c0 — needs port (check): UIElement_Text::ScrollToPosition behaviour; implemented area ac2-ui text.rs / text_attr.rs / text_scroll.rs but uncited
- [x] `UIElement_Text::ScrollHorizontalView` · 005f05f0 · legions 005b1120 — UIElement_Text::ScrollHorizontalView done: text_edit.rs scroll_to_cursor
- [x] `UIElement_Text::DeleteSection` · 005f06a0 · legions 005b11d0 — ac2-ui text.rs: TextState::delete_range (both ends clamped, nothing removed unless a glyph lies between, a cursor after the start moves back by the removed count)
- [x] `UIElement_Text::VerticalPageDecrement` · 005f0730 · legions 005b1260 — ac2-ui text_attr.rs: UiTree::text_new_page (view moves an element extent, kept in bounds by the layout, bars updated) and page_text; MoveCursor PAGEUP/PAGEDOWN in text_edit.rs
- [x] `UIElement_Text::VerticalPageIncrement` · 005f0740 · legions 005b1270 — ac2-ui text_attr.rs: UiTree::text_new_page (view moves an element extent, kept in bounds by the layout, bars updated) and page_text; MoveCursor PAGEUP/PAGEDOWN in text_edit.rs
- [x] `UIElement_Text::ScrollToHome` · 005f0750 · legions 005b1280 — UIElement_Text::ScrollToHome done: chat_window.rs scroll_log
- [x] `UIElement_Text::ScrollToEnd` · 005f0770 · legions 005b12a0 — ac2-ui chat.rs / chat_window.rs
- [~] `UIElement_Text::HandleScrollBarMessage` · 005f0790 · legions 005b12c0 — vertical bar only (text_scroll.rs): position x (paper - (y1 - y0 + 1)) as admin 7148 0x005F0790 and 0x005EDC40 read; horizontal bar, widget size (thumb keeps its own), page messages from the bar's track not modelled. The bar is found across the UI (named_scrollbar)
- [x] `UIElement_Text::MoveCursor` · 005f0940 · legions 005b1470 — ac2-ui text_edit.rs: UiTree::move_cursor now covers page up/down (virtual keys 0x21/0x22) beside home/end, line, word, left/right, up/down; not modelled: IME
- [ ] `UIElement_Text::MoveCursorToEnd` · 005f0ad0 · legions 005b1600 — needs port (check): UIElement_Text::MoveCursorToEnd behaviour; implemented area ac2-ui text.rs / text_attr.rs / text_scroll.rs but uncited
- [x] `UIElement_Text::DeleteChar` · 005f0b10 · legions 005b1640 — ac2-ui text_edit.rs, text_clip.rs: selection, drag, Ctrl+A/C/X/V through the tree's clipboard (text_clipboard.rs; ac2-client clipboard.rs over arboard); password fields refuse copy and cut; tests text_clip_tests.rs, real_data_tests.rs chat entry
- [x] `UIElement_Text::DeleteSelection` · 005f0b30 · legions 005b1660 — ac2-ui text_edit.rs, text_clip.rs: selection, drag, Ctrl+A/C/X/V through the tree's clipboard (text_clipboard.rs; ac2-client clipboard.rs over arboard); password fields refuse copy and cut; tests text_clip_tests.rs, real_data_tests.rs chat entry
- [~] `UIElement_Text::AddText_Internal` · 005f0ba0 · legions 005b16d0 — ac2-ui text_edit.rs / text_clip.rs / text.rs: drag and Shift selection, clipboard keys; not modelled: wheel on text (HandleMouseWheel), page up/down, IME, fallback fonts; clipboard is arboard, not CF_UNICODETEXT (ac2-client clipboard.rs)
- [x] `UIElement_Text::Cut` · 005f0cf0 — ac2-ui text_edit.rs, text_clip.rs: selection, drag, Ctrl+A/C/X/V through the tree's clipboard (text_clipboard.rs; ac2-client clipboard.rs over arboard); password fields refuse copy and cut; tests text_clip_tests.rs, real_data_tests.rs chat entry
- [x] `UIElement_Text::ActionHandler` · 005f0d50 — ac2-ui input.rs / text_edit.rs: typing into text elements
- [~] `UIElement_Text::CheckStringDownloads` · 005f0e90 · legions 005b19c0 — ac2-ui text.rs: no editing, cursor, selection, scrolling, fallback fonts, string variables or server downloads; A8R8G8B8 surfaces
- [x] `UIElement_Text::Paste` · 005f0f70 — ac2-ui text_edit.rs, text_clip.rs: selection, drag, Ctrl+A/C/X/V through the tree's clipboard (text_clipboard.rs; ac2-client clipboard.rs over arboard); password fields refuse copy and cut; tests text_clip_tests.rs, real_data_tests.rs chat entry
- [~] `UIElement_Text::CharacterHandler` · 005f0ff0 — ac2-ui text_edit.rs / text_clip.rs / text.rs: drag and Shift selection, clipboard keys; not modelled: wheel on text (HandleMouseWheel), page up/down, IME, fallback fonts; clipboard is arboard, not CF_UNICODETEXT (ac2-client clipboard.rs)
- [ ] `UIElement_Text::Global_Loop` · 005f12a0 · legions 005b1dd0 — needs port (check): UIElement_Text::Global_Loop behaviour; implemented area ac2-ui text.rs / text_attr.rs / text_scroll.rs but uncited
- [x] `UIElement_Text::AddStringInfowFont` · 005f1350 · legions 005b1e80 — ac2-ui chat.rs / chat_window.rs
- [x] `UIElement_Text::AddStringInfowFont` · 005f13e0 · legions 005b1f10 — ac2-ui chat.rs / chat_window.rs
- [ ] `UIElement_Text::ListenToMessage` · 005f1470 · legions 005b1fa0 — needs port (check): UIElement_Text::ListenToMessage; menu.rs cite is a menu selection broadcast
- [ ] `UIElement_Text::AddStringInfo` · 005f14b0 · legions 005b1fe0 — needs port (check): UIElement_Text::AddStringInfo behaviour; implemented area ac2-ui text.rs / text_attr.rs / text_scroll.rs but uncited
- [ ] `UIElement_Text::SetStringInfo` · 005f14c0 · legions 005b1ff0 — needs port (check): UIElement_Text::SetStringInfo overload; chat_window.rs prefill_entry cites two overloads
- [ ] `UIElement_Text::SetStringInfo` · 005f14e0 · legions 005b2010 — needs port (check): UIElement_Text::SetStringInfo overload; chat_window.rs prefill_entry cites two overloads
- [x] `UIElement_Text::AddText` · 005f1510 · legions 005b2040 — UIElement_Text::AddText done: text_attr.rs append_text
- [x] `UIElement_Text::TruncateText` · 005f1570 · legions 005b20a0 — UIElement_Text::TruncateText done: skill_tree.rs truncate_text
- [~] `UIElement_Text::SetText` · 005f16e0 · legions 005b2210 — ac2-ui reticle.rs: no off-screen indicator, meters, radar-look colour or quantity; name plates and damage text not ported
- [~] `UIElement_Text::SetText` · 005f1750 · legions 005b2280 — ac2-ui reticle.rs: no off-screen indicator, meters, radar-look colour or quantity; name plates and damage text not ported
- [~] `UIElement_Text::SetString` · 005f17c0 · legions 005b22f0 — ac2-ui text.rs: no editing, cursor, selection, scrolling, fallback fonts, string variables or server downloads; A8R8G8B8 surfaces
- [ ] `UIElement_Text::MatchAttributes` · 005f1810 · legions 005b2340 — needs port (check): UIElement_Text::MatchAttributes behaviour; implemented area ac2-ui text.rs / text_attr.rs / text_scroll.rs but uncited
- [~] `UIElement_Text::SetAttribute_Bool` · 005f18c0 · legions 005b23f0 — ac2-ui text.rs: no editing, cursor, selection, scrolling, fallback fonts, string variables or server downloads; A8R8G8B8 surfaces
- [x] `UIElement_Text::SetAttribute_StringID` · 005f1a10 · legions 005b2540 — ac2-ui text.rs / text_attr.rs: UIElement_Text layout and drawing
- [x] `UIElement_Text::SetAttribute_DataID` · 005f1a50 · legions 005b2580 — ac2-ui text.rs / text_attr.rs: UIElement_Text layout and drawing
- [ ] `UIElement_Text::ListenToMessage` · 005f1aa0 · legions 005b25d0 — needs port (check): UIElement_Text::ListenToMessage; menu.rs cite is a menu selection broadcast

## `portal/client/src/uielement/types/uielement_button.cpp.c` (11/16)

### UIElement_Button
- [~] `UIElement_Button::UIElement_Button` · 005fb4e0 · legions 005bc010 — ac2-ui mouse.rs / button.rs: no tooltips, cursors, drag and drop, moving/resizing, activation, hot clicks or listener registry (messages are taken by the caller)
- [-] `UIElement_Button::Create` · 005fb540 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own
- [x] `UIElement_Button::Global_Loop` · 005fb570 · legions 005bc0a0 — ac2-ui button.rs button_global_loop / button_mouse_down / button_mouse_up: the press clicks at once on a hot_button, then every hot_click_interval after hot_click_first_interval (defaults 0.3 / 0.1 s from the ctor) off the HUD frame clock (UiTree::global_loop); the release stops it. Test: scrollbar_tests a_held_hot_button_repeats_its_click
- [x] `UIElement_Button::ListenToMessage` · 005fb5f0 · legions 005bc120 — ac2-ui button.rs button_global_loop / button_mouse_down / button_mouse_up: the press clicks at once on a hot_button, then every hot_click_interval after hot_click_first_interval (defaults 0.3 / 0.1 s from the ctor) off the HUD frame clock (UiTree::global_loop); the release stops it. Test: scrollbar_tests a_held_hot_button_repeats_its_click
- [x] `UIElement_Button::UpdateState` · 005fb620 · legions 005bc150 — ac2-ui mouse.rs / button.rs: UI hit tests, presses and buttons
- [x] `UIElement_Button::SetState` · 005fb710 · legions 005bc240 — ac2-ui mouse.rs / button.rs: UI hit tests, presses and buttons
- [x] `UIElement_Button::IsActivatable` · 005fb770 · legions 005bc2a0 — ac2-ui button.rs is_activatable (ghosted is never activatable, else the activatable attribute); text_edit.rs activate/deactivate refresh a button's state
- [x] `UIElement_Button::Activate` · 005fb7a0 · legions 005bc2d0 — ac2-ui button.rs is_activatable (ghosted is never activatable, else the activatable attribute); text_edit.rs activate/deactivate refresh a button's state
- [x] `UIElement_Button::Deactivate` · 005fb7c0 · legions 005bc2f0 — ac2-ui button.rs is_activatable (ghosted is never activatable, else the activatable attribute); text_edit.rs activate/deactivate refresh a button's state
- [x] `UIElement_Button::StartUseActive` · 005fb7e0 · legions 005bc310 — ac2-ui button.rs button_global_loop / button_mouse_down / button_mouse_up: the press clicks at once on a hot_button, then every hot_click_interval after hot_click_first_interval (defaults 0.3 / 0.1 s from the ctor) off the HUD frame clock (UiTree::global_loop); the release stops it. Test: scrollbar_tests a_held_hot_button_repeats_its_click
- [x] `UIElement_Button::EndUseActive` · 005fb870 · legions 005bc3a0 — ac2-ui button.rs button_global_loop / button_mouse_down / button_mouse_up: the press clicks at once on a hot_button, then every hot_click_interval after hot_click_first_interval (defaults 0.3 / 0.1 s from the ctor) off the HUD frame clock (UiTree::global_loop); the release stops it. Test: scrollbar_tests a_held_hot_button_repeats_its_click
- [~] `UIElement_Button::SetAttribute_Bool` · 005fb940 · legions 005bc470 — ac2-ui mouse.rs / button.rs: no tooltips, cursors, drag and drop, moving/resizing, activation, hot clicks or listener registry (messages are taken by the caller)
- [-] `UIElement_Button::Register` · 005fb9b0 · legions 005bc4e0 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own
- [~] `UIElement_Button::MouseDown` · 005fb9d0 · legions 005bc500 — ac2-ui mouse.rs / button.rs: no tooltips, cursors, drag and drop, moving/resizing, activation, hot clicks or listener registry (messages are taken by the caller)
- [x] `UIElement_Button::MouseUp` · 005fbb10 · legions 005bc640 — ac2-ui mouse.rs / button.rs: UI hit tests, presses and buttons
- [x] `UIElement_Button::MouseOverTop` · 005fbc50 · legions 005bc780 — ac2-ui mouse.rs / button.rs: UI hit tests, presses and buttons

## `portal/client/src/uielement/types/uielement_dragbar.cpp.c` (2/8)

### UIElement_Dragbar
- [x] `UIElement_Dragbar::MouseDown` · 005f8b80 · legions 005b96b0 — ac2-ui window_locations.rs (inq_locations / apply_locations, clip conversions, drag bar movement); tests window_locations_tests.rs on real HUD
- [x] `UIElement_Dragbar::MouseUp` · 005f8c30 · legions 005b9760 — UIElement_Dragbar::MouseUp done: window_locations.rs dragbar_mouse_up
- [ ] `UIElement_Dragbar::UpdateLayout` · 005f8c90 · legions 005b97c0 — needs port (check): UIElement_Dragbar::UpdateLayout behaviour; implemented area ac2-ui window_locations.rs but uncited
- [ ] `UIElement_Dragbar::SetState` · 005f8d60 · legions 005b9890 — needs port (check): UIElement_Dragbar::SetState behaviour; implemented area ac2-ui window_locations.rs but uncited
- [ ] `UIElement_Dragbar::SetAttribute_Bool` · 005f8d80 · legions 005b98b0 — needs port (check): UIElement_Dragbar::SetAttribute_Bool behaviour; implemented area ac2-ui window_locations.rs but uncited
- [-] `UIElement_Dragbar::UIElement_Dragbar` · 005f8dc0 · legions 005b98f0 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own
- [-] `UIElement_Dragbar::Create` · 005f8e20 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own
- [-] `UIElement_Dragbar::Register` · 005f8e50 · legions 005b9980 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own

## `portal/client/src/uielement/types/uielement_field.cpp.c` (1/4)

### UIElement_Field
- [-] `UIElement_Field::Create` · 005fbc70 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own
- [ ] `UIElement_Field::MouseOverTop` · 005fbcb0 · legions 005bc7e0 — needs port (check): UIElement_Field::MouseOverTop behaviour; implemented area ac2-ui text_edit.rs but uncited
- [x] `UIElement_Field::CatchDroppedItem` · 005fbdb0 · legions 005bc8e0 — ac2-ui drag.rs; shortcut.rs drop
- [-] `UIElement_Field::Register` · 005fbe20 · legions 005bc950 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own

## `portal/client/src/uielement/types/uielement_listbox.cpp.c` (50/82)

### UIElement_ListBox
- [x] `UIElement_ListBox::Home` · 005f1b90 · legions 005b26c0 — ac2-ui list_box.rs
- [x] `UIElement_ListBox::End` · 005f1bc0 · legions 005b26f0 — ac2-ui list_box.rs
- [x] `UIElement_ListBox::ScrollToRow` · 005f1bf0 · legions 005b2720 — ac2-ui list_box.rs
- [x] `UIElement_ListBox::ScrollToCol` · 005f1c20 · legions 005b2750 — ac2-ui list_goal.rs: UiTree::list_scroll_to_col / list_left_col / list_col / list_percentage
- [x] `UIElement_ListBox::GetLeftCol` · 005f1c50 · legions 005b2780 — ac2-ui list_goal.rs: UiTree::list_scroll_to_col / list_left_col / list_col / list_percentage
- [x] `UIElement_ListBox::GetTopRow` · 005f1c70 · legions 005b27a0 — ac2-ui list_box.rs
- [x] `UIElement_ListBox::KeepViewInBounds` · 005f1c90 · legions 005b27c0 — ac2-ui list_box.rs
- [-] `UIElement_ListBox::HandleButtonGhosting` · 005f1d40 · legions 005b2870 — ctor/dtor/Create/Register are the framework's element creation (UiTree::create_element) and Drop; scroll-bar listeners are the lists_bound_to lookup (list_scroll.rs), not message registrations
- [x] `UIElement_ListBox::GetScrollBar` · 005f1db0 · legions 005b28e0 — named_scrollbar (scrollbar.rs): GetElement without unique_msg finds a scroll bar in any root, GetUniqueMatchElement with it in the owner's root; text_scroll.rs, list_scroll.rs and tree_scroll.rs bind through it. Real data: scroll_coverage_tests.rs
- [-] `UIElement_ListBox::UnregisterForScrollbar` · 005f1e50 · legions 005b2980 — ctor/dtor/Create/Register are the framework's element creation (UiTree::create_element) and Drop; scroll-bar listeners are the lists_bound_to lookup (list_scroll.rs), not message registrations
- [-] `UIElement_ListBox::TestBitMask` · 005f1f60 · legions 005b2a90 — m_bitField is typed fields in ListBoxState (multiop, from_scrollbar) and attributes; no mask test needed
- [~] `UIElement_ListBox::ScrollToShow` · 005f1f80 · legions 005b2ab0 — list_box_select.rs: left-press selection; wheel/scroll bar and smooth (goal) scrolling not modelled
- [~] `UIElement_ListBox::SetAttribute_Bool` · 005f1fc0 · legions 005b2af0 — ac2-ui list_box.rs: no smooth scrolling, controls or selection
- [x] `UIElement_ListBox::DetermineAndSetNewGoal` · 005f2180 · legions 005b2cb0 — ac2-ui list_goal.rs / list_msize.rs: the max/min clamp of the goal is inlined in list_line_step, list_page_step and list_msize_step; a smooth list jumps to the goal
- [x] `UIElement_ListBox::UpdateLayout` · 005f21f0 · legions 005b2d20 — ac2-ui list_box.rs
- [ ] `UIElement_ListBox::StartAnimation` · 005f23f0 · legions 005b2f20 — needs port (check): UIElement_ListBox::StartAnimation behaviour; implemented area ac2-ui list_box.rs / list_scroll.rs but uncited
- [ ] `UIElement_ListBox::StopAnimation` · 005f2500 · legions 005b3030 — needs port (check): UIElement_ListBox::StopAnimation behaviour; implemented area ac2-ui list_box.rs / list_scroll.rs but uncited
- [x] `UIElement_ListBox::GetItem` · 005f2540 · legions 005b3070 — ac2-ui list_box.rs
- [x] `UIElement_ListBox::SetSelectedItem` · 005f2560 · legions 005b3090 — ac2-ui list_box_select.rs
- [x] `UIElement_ListBox::SetSelectedIndex` · 005f2650 · legions 005b3180 — ac2-ui list_box_select.rs
- [x] `UIElement_ListBox::GetItemIndexAtPoint` · 005f26b0 · legions 005b31e0 — ac2-ui list_box.rs
- [~] `UIElement_ListBox::InsertItem` · 005f27c0 · legions 005b32f0 — ac2-ui list_box.rs: no smooth scrolling, controls or selection
- [x] `UIElement_ListBox::MultiOpAddItem` · 005f2820 · legions 005b3350 — ac2-ui list_goal.rs: list_multi_op_add_item / list_multi_op_insert_item / list_multi_op_remove_item (the 0x200000 multi-op bit is ListBoxState::multiop)
- [x] `UIElement_ListBox::MultiOpInsertItem` · 005f2840 · legions 005b3370 — ac2-ui list_goal.rs: list_multi_op_add_item / list_multi_op_insert_item / list_multi_op_remove_item (the 0x200000 multi-op bit is ListBoxState::multiop)
- [x] `UIElement_ListBox::MultiOpInsertItem` · 005f2850 · legions 005b3380 — ac2-ui list_goal.rs: list_multi_op_add_item / list_multi_op_insert_item / list_multi_op_remove_item (the 0x200000 multi-op bit is ListBoxState::multiop)
- [x] `UIElement_ListBox::GetCol` · 005f2870 · legions 005b33a0 — ac2-ui list_goal.rs: UiTree::list_scroll_to_col / list_left_col / list_col / list_percentage
- [x] `UIElement_ListBox::GetRow` · 005f28d0 · legions 005b3400 — ac2-ui list_box.rs list_what_num (the row index the extra text goes after)
- [~] `UIElement_ListBox::ScrollToShow` · 005f2930 · legions 005b3460 — list_box_select.rs: left-press selection; wheel/scroll bar and smooth (goal) scrolling not modelled
- [ ] `UIElement_ListBox::StartDragSelect` · 005f2a50 · legions 005b3580 — needs port (check): UIElement_ListBox::StartDragSelect behaviour; implemented area ac2-ui list_box.rs / list_scroll.rs but uncited
- [ ] `UIElement_ListBox::EndDragSelect` · 005f2ac0 · legions 005b35f0 — needs port (check): UIElement_ListBox::EndDragSelect behaviour; implemented area ac2-ui list_box.rs / list_scroll.rs but uncited
- [ ] `UIElement_ListBox::StartRollover` · 005f2b10 · legions 005b3640 — needs port (check): UIElement_ListBox::StartRollover behaviour; implemented area ac2-ui list_box.rs / list_scroll.rs but uncited
- [ ] `UIElement_ListBox::EndRollover` · 005f2b60 · legions 005b3690 — needs port (check): UIElement_ListBox::EndRollover behaviour; implemented area ac2-ui list_box.rs / list_scroll.rs but uncited
- [x] `UIElement_ListBox::ConstrainViewValue` · 005f2bc0 · legions 005b36f0 — ac2-ui list_box.rs
- [x] `UIElement_ListBox::GetPercentageV` · 005f2c30 · legions 005b3760 — ac2-ui list_goal.rs: UiTree::list_scroll_to_col / list_left_col / list_col / list_percentage
- [x] `UIElement_ListBox::GetPercentageH` · 005f2cc0 · legions 005b37f0 — ac2-ui list_goal.rs: UiTree::list_scroll_to_col / list_left_col / list_col / list_percentage
- [x] `UIElement_ListBox::HandleMouseWheel` · 005f2d50 · legions 005b3880 — wheel: the element under the pointer hands the turn to its vertical bar (text_scroll.rs text_wheel, tree_scroll.rs tree_wheel, list_scroll.rs list_wheel); the bar steps its stop, or the lists, texts and trees it drives one line (scrollbar.rs scrollbar_mouse_wheel). Real-data: scroll_coverage_tests.rs over every layout; synthetic: list_scroll_tests.rs
- [~] `UIElement_ListBox::HandleScrollBarMessage` · 005f2d90 · legions 005b38c0 — list_scroll.rs: view = position x (paper - (y1 - y0 + 1)) and share = view / (paper - (y1 - y0 + 1)) (admin 0x005F2D90, 0x005F2C30); smooth goal (goal_x_view, StartAnimation) and paging not modelled
- [ ] `UIElement_ListBox::HandleScrollbarWidgetRelease` · 005f2f60 · legions 005b3a90 — needs port (check): UIElement_ListBox::HandleScrollbarWidgetRelease behaviour; implemented area ac2-ui list_box.rs / list_scroll.rs but uncited
- [-] `UIElement_ListBox::~UIElement_ListBox` · 005f3070 · legions 005b3ba0 — ctor/dtor/Create/Register are the framework's element creation (UiTree::create_element) and Drop; scroll-bar listeners are the lists_bound_to lookup (list_scroll.rs), not message registrations
- [ ] `UIElement_ListBox::MouseOverTop` · 005f3190 · legions 005b3cc0 — needs port (check): UIElement_ListBox::MouseOverTop behaviour; implemented area ac2-ui list_box.rs / list_scroll.rs but uncited
- [ ] `UIElement_ListBox::ListenToMessage` · 005f31f0 · legions 005b3d20 — needs port (check): UIElement_ListBox::ListenToMessage behaviour; implemented area ac2-ui list_box.rs / list_scroll.rs but uncited
- [ ] `UIElement_ListBox::Global_Loop` · 005f3320 · legions 005b3e50 — needs port (check): UIElement_ListBox::Global_Loop behaviour; implemented area ac2-ui list_box.rs / list_scroll.rs but uncited
- [x] `UIElement_ListBox::StartActive` · 005f3430 · legions 005b3f60 — UIElement_ListBox::StartActive done: list_scroll.rs list_step_selection
- [x] `UIElement_ListBox::VerticalPageDecrement` · 005f34d0 · legions 005b4000 — ac2-ui list_goal.rs: list_line_step (line steps clamp to 0 and paper - height) and list_page_step (a page less one item); a smooth list jumps to the goal (no animation clock)
- [x] `UIElement_ListBox::VerticalPageIncrement` · 005f3550 · legions 005b4080 — ac2-ui list_goal.rs: list_line_step (line steps clamp to 0 and paper - height) and list_page_step (a page less one item); a smooth list jumps to the goal (no animation clock)
- [x] `UIElement_ListBox::HorizontalPageDecrement` · 005f35e0 · legions 005b4110 — ac2-ui list_goal.rs: list_line_step (line steps clamp to 0 and paper - height) and list_page_step (a page less one item); a smooth list jumps to the goal (no animation clock)
- [x] `UIElement_ListBox::HorizontalPageIncrement` · 005f3660 · legions 005b4190 — ac2-ui list_goal.rs: list_line_step (line steps clamp to 0 and paper - height) and list_page_step (a page less one item); a smooth list jumps to the goal (no animation clock)
- [x] `UIElement_ListBox::VerticalLineDecrement` · 005f36f0 · legions 005b4220 — ac2-ui list_goal.rs: list_line_step (line steps clamp to 0 and paper - height) and list_page_step (a page less one item); a smooth list jumps to the goal (no animation clock)
- [x] `UIElement_ListBox::VerticalLineIncrement` · 005f3760 · legions 005b4290 — ac2-ui list_goal.rs: list_line_step (line steps clamp to 0 and paper - height) and list_page_step (a page less one item); a smooth list jumps to the goal (no animation clock)
- [x] `UIElement_ListBox::HorizontalLineDecrement` · 005f37e0 · legions 005b4310 — ac2-ui list_goal.rs: list_line_step (line steps clamp to 0 and paper - height) and list_page_step (a page less one item); a smooth list jumps to the goal (no animation clock)
- [x] `UIElement_ListBox::HorizontalLineIncrement` · 005f3850 · legions 005b4380 — ac2-ui list_goal.rs: list_line_step (line steps clamp to 0 and paper - height) and list_page_step (a page less one item); a smooth list jumps to the goal (no animation clock)
- [x] `UIElement_ListBox::UpdateControls` · 005f38d0 · legions 005b4400 — UIElement_ListBox::UpdateControls done: list_scroll.rs list_update_controls
- [x] `UIElement_ListBox::UpdateAndDraw` · 005f3af0 · legions 005b4620 — ac2-ui list_goal.rs: UiTree::list_update_and_draw (layout, KeepViewInBounds, UpdateControls, redraw)
- [ ] `UIElement_ListBox::ResizeTo` · 005f3b20 · legions 005b4650 — needs port (check): UIElement_ListBox::ResizeTo behaviour; implemented area ac2-ui list_box.rs / list_scroll.rs but uncited
- [x] `UIElement_ListBox::SetSelectedItemByID` · 005f3b60 · legions 005b4690 — ac2-ui list_goal.rs: UiTree::list_set_selected_item_by_id
- [x] `UIElement_ListBox::GetItemUnderMouse` · 005f3c20 · legions 005b4750 — ac2-ui list_box.rs
- [x] `UIElement_ListBox::RemoveItem` · 005f3c90 · legions 005b47c0 — ac2-ui list_box.rs
- [x] `UIElement_ListBox::MultiOpRemoveItem` · 005f3d60 · legions 005b4890 — ac2-ui list_goal.rs: list_multi_op_add_item / list_multi_op_insert_item / list_multi_op_remove_item (the 0x200000 multi-op bit is ListBoxState::multiop)
- [x] `UIElement_ListBox::MultiOpComplete` · 005f3d70 · legions 005b48a0 — ac2-ui list_box.rs
- [x] `UIElement_ListBox::Flush` · 005f3db0 · legions 005b48e0 — ac2-ui list_box.rs
- [ ] `UIElement_ListBox::Rollover` · 005f3e70 · legions 005b49a0 — needs port (check): UIElement_ListBox::Rollover behaviour; implemented area ac2-ui list_box.rs / list_scroll.rs but uncited
- [x] `UIElement_ListBox::ViewChanged` · 005f3f30 · legions 005b4a60 — ac2-ui list_box.rs
- [x] `UIElement_ListBox::HandleSelection` · 005f3fd0 · legions 005b4b00 — ac2-ui list_box_select.rs
- [ ] `UIElement_ListBox::SetState` · 005f4030 · legions 005b4b60 — needs port (check): UIElement_ListBox::SetState behaviour; implemented area ac2-ui list_box.rs / list_scroll.rs but uncited
- [~] `UIElement_ListBox::SetAttribute_Int` · 005f4170 · legions 005b4ca0 — ac2-ui list_box.rs: no smooth scrolling, controls or selection
- [ ] `UIElement_ListBox::SetAttribute_ID` · 005f4270 · legions 005b4da0 — needs port (check): UIElement_ListBox::SetAttribute_ID behaviour; implemented area ac2-ui list_box.rs / list_scroll.rs but uncited
- [~] `UIElement_ListBox::MouseDown` · 005f4420 · legions 005b4f50 — left press selects the item (list_box_select.rs); the wheel goes to the scroll bar (list_scroll.rs); drag selection and rollover not modelled
- [ ] `UIElement_ListBox::Global_MouseMove` · 005f44c0 · legions 005b4ff0 — needs port (check): UIElement_ListBox::Global_MouseMove behaviour; implemented area ac2-ui list_box.rs / list_scroll.rs but uncited
- [x] `UIElement_ListBox::StartUpActive` · 005f4500 · legions 005b5030 — ac2-ui list_scroll.rs: UiTree::list_step_active (arrow keys step the selection)
- [x] `UIElement_ListBox::StartDownActive` · 005f4540 · legions 005b5070 — ac2-ui list_scroll.rs: UiTree::list_step_active (arrow keys step the selection)
- [x] `UIElement_ListBox::StartLeftActive` · 005f4570 · legions 005b50a0 — ac2-ui list_scroll.rs: UiTree::list_step_active (arrow keys step the selection)
- [x] `UIElement_ListBox::StartRightActive` · 005f45b0 · legions 005b50e0 — ac2-ui list_scroll.rs: UiTree::list_step_active (arrow keys step the selection)
- [x] `UIElement_ListBox::AddItem` · 005f45e0 · legions 005b5110 — ac2-ui list_box.rs
- [~] `UIElement_ListBox::InsertItem` · 005f4680 · legions 005b51b0 — ac2-ui list_box.rs: no smooth scrolling, controls or selection
- [x] `UIElement_ListBox::DeleteItem` · 005f46f0 · legions 005b5220 — ac2-ui list_box.rs
- [x] `UIElement_ListBox::MultiOpRemoveItem` · 005f4750 · legions 005b5280 — ac2-ui list_goal.rs: list_multi_op_add_item / list_multi_op_insert_item / list_multi_op_remove_item (the 0x200000 multi-op bit is ListBoxState::multiop)
- [~] `UIElement_ListBox::UIElement_ListBox` · 005f47a0 · legions 005b52d0 — ac2-ui list_box.rs: no smooth scrolling, controls or selection
- [-] `UIElement_ListBox::Create` · 005f4a80 — ctor/dtor/Create/Register are the framework's element creation (UiTree::create_element) and Drop; scroll-bar listeners are the lists_bound_to lookup (list_scroll.rs), not message registrations
- [ ] `UIElement_ListBox::ListenToMessage` · 005f4ab0 · legions 005b55e0 — needs port (check): UIElement_ListBox::ListenToMessage behaviour; implemented area ac2-ui list_box.rs / list_scroll.rs but uncited
- [ ] `UIElement_ListBox::ListenToMessage` · 005f4b00 · legions 005b5630 — needs port (check): UIElement_ListBox::ListenToMessage behaviour; implemented area ac2-ui list_box.rs / list_scroll.rs but uncited
- [x] `UIElement_ListBox::SetItem` · 005f4b90 · legions 005b56c0 — ac2-ui list_box.rs
- [-] `UIElement_ListBox::Register` · 005f4c30 · legions 005b5760 — ctor/dtor/Create/Register are the framework's element creation (UiTree::create_element) and Drop; scroll-bar listeners are the lists_bound_to lookup (list_scroll.rs), not message registrations

## `portal/client/src/uielement/types/uielement_listboxmsize.cpp.c` (16/20)

### UIElement_ListBoxMSize
- [x] `UIElement_ListBoxMSize::ConstrainViewValue` · 005f8e70 · legions 005b99a0 — ac2-ui list_msize.rs: UiTree::list_msize_constrain (no snapping, within 0 and paper - extent)
- [x] `UIElement_ListBoxMSize::InqItemIndex` · 005f8ed0 · legions 005b9a00 — ac2-ui list_msize.rs: item_index / item_offset (InqItemIndex, InqItemSize over the running sizes)
- [x] `UIElement_ListBoxMSize::InqItemSize` · 005f8f20 · legions 005b9a50 — ac2-ui list_msize.rs: item_index / item_offset (InqItemIndex, InqItemSize over the running sizes)
- [~] `UIElement_ListBoxMSize::ScrollToShow` · 005f8f60 · legions 005b9a90 — list_msize.rs: not ported: the base list's item size that ScrollToShow reads (the sized list never sets it); a selection is shown by the base list_scroll_to_show
- [x] `UIElement_ListBoxMSize::ScrollToRow` · 005f90d0 · legions 005b9c00 — ac2-ui list_msize.rs: list_msize_scroll_to (immediate; smooth scrolling is not modelled)
- [x] `UIElement_ListBoxMSize::ScrollToCol` · 005f9110 · legions 005b9c40 — ac2-ui list_msize.rs: list_msize_scroll_to (immediate; smooth scrolling is not modelled)
- [x] `UIElement_ListBoxMSize::VerticalPageDecrement` · 005f91a0 · legions 005b9cd0 — ac2-ui list_msize.rs: list_msize_step (line/page by the cell at the view edge, DetermineAndSetNewGoal clamps)
- [x] `UIElement_ListBoxMSize::VerticalPageIncrement` · 005f9200 · legions 005b9d30 — ac2-ui list_msize.rs: list_msize_step (line/page by the cell at the view edge, DetermineAndSetNewGoal clamps)
- [x] `UIElement_ListBoxMSize::HorizontalPageDecrement` · 005f9270 · legions 005b9da0 — ac2-ui list_msize.rs: list_msize_step (line/page by the cell at the view edge, DetermineAndSetNewGoal clamps)
- [x] `UIElement_ListBoxMSize::HorizontalPageIncrement` · 005f92d0 · legions 005b9e00 — ac2-ui list_msize.rs: list_msize_step (line/page by the cell at the view edge, DetermineAndSetNewGoal clamps)
- [x] `UIElement_ListBoxMSize::VerticalLineDecrement` · 005f9340 · legions 005b9e70 — ac2-ui list_msize.rs: list_msize_step (line/page by the cell at the view edge, DetermineAndSetNewGoal clamps)
- [x] `UIElement_ListBoxMSize::VerticalLineIncrement` · 005f93a0 · legions 005b9ed0 — ac2-ui list_msize.rs: list_msize_step (line/page by the cell at the view edge, DetermineAndSetNewGoal clamps)
- [x] `UIElement_ListBoxMSize::HorizontalLineDecrement` · 005f9410 · legions 005b9f40 — ac2-ui list_msize.rs: list_msize_step (line/page by the cell at the view edge, DetermineAndSetNewGoal clamps)
- [x] `UIElement_ListBoxMSize::HorizontalLineIncrement` · 005f9470 · legions 005b9fa0 — ac2-ui list_msize.rs: list_msize_step (line/page by the cell at the view edge, DetermineAndSetNewGoal clamps)
- [x] `UIElement_ListBoxMSize::GetItemIndexAtPoint` · 005f94e0 · legions 005ba010 — ac2-ui list_msize.rs: list_msize_index_at
- [-] `UIElement_ListBoxMSize::UIElement_ListBoxMSize` · 005f95f0 · legions 005ba120 — ctor/Create/Register are the framework's element creation (UiTree::create_element, the listboxmsize kind in list_box_select.rs)
- [-] `UIElement_ListBoxMSize::Create` · 005f9640 — ctor/Create/Register are the framework's element creation (UiTree::create_element, the listboxmsize kind in list_box_select.rs)
- [x] `UIElement_ListBoxMSize::CalculateItemWidthsHeights` · 005f9670 · legions 005ba1a0 — ac2-ui list_msize.rs: list_msize_layout (column widths and row heights, running positions, paper)
- [-] `UIElement_ListBoxMSize::Register` · 005f98a0 · legions 005ba3d0 — ctor/Create/Register are the framework's element creation (UiTree::create_element, the listboxmsize kind in list_box_select.rs)
- [x] `UIElement_ListBoxMSize::UpdateLayout` · 005f98c0 · legions 005ba3f0 — ac2-ui list_msize.rs: list_msize_layout (column widths and row heights, running positions, paper)

## `portal/client/src/uielement/types/uielement_menu.cpp.c` (21/22)

### UIElement_Menu
- [x] `UIElement_Menu::Create` · 005f5420 — ac2-ui menu.rs: UiTree::menu_* (open, close, items, selection, state, display, teardown); hooks in element.rs, list_box_select.rs, mouse.rs
- [x] `UIElement_Menu::Flush` · 005f5470 · legions 005b5fa0 — ac2-ui menu.rs: UiTree::menu_* (open, close, items, selection, state, display, teardown); hooks in element.rs, list_box_select.rs, mouse.rs
- [x] `UIElement_Menu::GetItem` · 005f5480 · legions 005b5fb0 — ac2-ui menu.rs: UiTree::menu_* (open, close, items, selection, state, display, teardown); hooks in element.rs, list_box_select.rs, mouse.rs
- [x] `UIElement_Menu::GetSelectedItem` · 005f54a0 · legions 005b5fd0 — ac2-ui menu.rs: UiTree::menu_* (open, close, items, selection, state, display, teardown); hooks in element.rs, list_box_select.rs, mouse.rs
- [x] `UIElement_Menu::SetSelectedIndex` · 005f54c0 · legions 005b5ff0 — ac2-ui menu.rs: UiTree::menu_* (open, close, items, selection, state, display, teardown); hooks in element.rs, list_box_select.rs, mouse.rs
- [x] `UIElement_Menu::SetSelectedItem` · 005f54e0 · legions 005b6010 — ac2-ui menu.rs: UiTree::menu_* (open, close, items, selection, state, display, teardown); hooks in element.rs, list_box_select.rs, mouse.rs
- [x] `UIElement_Menu::Register` · 005f5500 · legions 005b6030 — ac2-ui menu.rs: UiTree::menu_* (open, close, items, selection, state, display, teardown); hooks in element.rs, list_box_select.rs, mouse.rs
- [x] `UIElement_Menu::InsertTextItem` · 005f5520 · legions 005b6050 — ac2-ui menu.rs: UiTree::menu_* (open, close, items, selection, state, display, teardown); hooks in element.rs, list_box_select.rs, mouse.rs
- [x] `UIElement_Menu::GetNumItems` · 005f55e0 · legions 005b6110 — ac2-ui menu.rs: UiTree::menu_* (open, close, items, selection, state, display, teardown); hooks in element.rs, list_box_select.rs, mouse.rs
- [x] `UIElement_Menu::GetSelectedIndex` · 005f5600 · legions 005b6130 — ac2-ui menu.rs: UiTree::menu_* (open, close, items, selection, state, display, teardown); hooks in element.rs, list_box_select.rs, mouse.rs
- [x] `UIElement_Menu::Initialized` · 005f5640 · legions 005b6170 — ac2-ui menu.rs: UiTree::menu_* (open, close, items, selection, state, display, teardown); hooks in element.rs, list_box_select.rs, mouse.rs
- [x] `UIElement_Menu::NewSelection` · 005f5760 · legions 005b6290 — ac2-ui menu.rs: UiTree::menu_* (open, close, items, selection, state, display, teardown); hooks in element.rs, list_box_select.rs, mouse.rs
- [x] `UIElement_Menu::UpdateState` · 005f5810 · legions 005b6340 — ac2-ui menu.rs: UiTree::menu_* (open, close, items, selection, state, display, teardown); hooks in element.rs, list_box_select.rs, mouse.rs
- [x] `UIElement_Menu::MouseOverTop` · 005f58a0 · legions 005b63d0 — ac2-ui menu.rs: UiTree::menu_* (open, close, items, selection, state, display, teardown); hooks in element.rs, list_box_select.rs, mouse.rs
- [x] `UIElement_Menu::ListenToMessage` · 005f58c0 · legions 005b63f0 — ac2-ui menu.rs: UiTree::menu_* (open, close, items, selection, state, display, teardown); hooks in element.rs, list_box_select.rs, mouse.rs
- [x] `UIElement_Menu::Open` · 005f58e0 · legions 005b6410 — ac2-ui menu.rs: UiTree::menu_* (open, close, items, selection, state, display, teardown); hooks in element.rs, list_box_select.rs, mouse.rs
- [x] `UIElement_Menu::Close` · 005f5ad0 · legions 005b6600 — ac2-ui menu.rs: UiTree::menu_close
- [x] `UIElement_Menu::~UIElement_Menu` · 005f5b40 · legions 005b6670 — ac2-ui menu.rs: UiTree::menu_* (open, close, items, selection, state, display, teardown); hooks in element.rs, list_box_select.rs, mouse.rs
- [x] `UIElement_Menu::SetVisible` · 005f5bd0 · legions 005b6700 — ac2-ui menu.rs: UiTree::menu_* (open, close, items, selection, state, display, teardown); hooks in element.rs, list_box_select.rs, mouse.rs
- [x] `UIElement_Menu::SetAttribute_Bool` · 005f5c10 · legions 005b6740 — ac2-ui menu.rs: UiTree::menu_* (open, close, items, selection, state, display, teardown); hooks in element.rs, list_box_select.rs, mouse.rs
- [~] `UIElement_Menu::MouseDown` · 005f5c60 · legions 005b6790 — ac2-ui menu.rs: UiTree::menu_mouse_down (left press toggles); m_moving and m_resizing are not modelled (no move or resize)
- [x] `UIElement_Menu::ListenToMessage` · 005f5cf0 · legions 005b6820 — ac2-ui menu.rs: UiTree::menu_* (open, close, items, selection, state, display, teardown); hooks in element.rs, list_box_select.rs, mouse.rs

## `portal/client/src/uielement/types/uielement_meter.cpp.c` (13/14)

### UIElement_Meter
- [x] `UIElement_Meter::UpdateLayout` · 005f4c50 · legions 005b5780 — ac2-ui meter.rs
- [x] `UIElement_Meter::SetParent` · 005f4eb0 · legions 005b59e0 — ac2-ui meter.rs
- [x] `UIElement_Meter::SetState` · 005f4ed0 · legions 005b5a00 — ac2-ui meter.rs
- [x] `UIElement_Meter::MoveTo` · 005f4ef0 · legions 005b5a20 — ac2-ui meter.rs
- [x] `UIElement_Meter::ResizeTo` · 005f4f10 · legions 005b5a40 — ac2-ui meter.rs
- [~] `UIElement_Meter::CalculateNextStateChange` · 005f4f30 · legions 005b5a60 — ties between equal percentages follow insertion order, not hash bucket order
- [x] `UIElement_Meter::Global_Loop` · 005f4fe0 · legions 005b5b10 — ac2-ui meter.rs
- [x] `UIElement_Meter::ListenToMessage` · 005f5110 · legions 005b5c40 — ac2-ui meter.rs
- [x] `UIElement_Meter::StartAnimation` · 005f5140 · legions 005b5c70 — ac2-ui meter.rs
- [x] `UIElement_Meter::SetStateAtPercentage` · 005f5270 · legions 005b5da0 — ac2-ui meter.rs
- [x] `UIElement_Meter::SetAttribute_Float` · 005f52a0 · legions 005b5dd0 — ac2-ui meter.rs
- [x] `UIElement_Meter::UIElement_Meter` · 005f5320 · legions 005b5e50 — ac2-ui meter.rs
- [x] `UIElement_Meter::Create` · 005f53d0 — ac2-ui meter.rs
- [x] `UIElement_Meter::Register` · 005f5400 · legions 005b5f30 — ac2-ui meter.rs

## `portal/client/src/uielement/types/uielement_panel.cpp.c` (2/6)

### UIElement_Panel
- [-] `UIElement_Panel::~UIElement_Panel` · 005f89a0 · legions 005b94d0 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own
- [-] `UIElement_Panel::Create` · 005f8a30 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own
- [ ] `UIElement_Panel::SetAttribute_Bool` · 005f8a60 · legions 005b9590 — needs port (check): UIElement_Panel::SetAttribute_Bool behaviour; implemented area ac2-ui panel.rs but uncited
- [x] `UIElement_Panel::Initialized` · 005f8a70 · legions 005b95a0 — ac2-ui panel.rs
- [x] `UIElement_Panel::ListenToMessage` · 005f8ae0 · legions 005b9610 — ac2-ui panel.rs
- [-] `UIElement_Panel::Register` · 005f8b60 · legions 005b9690 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own

## `portal/client/src/uielement/types/uielement_panelpage.cpp.c` (5/9)

### UIElement_PanelPage
- [-] `UIElement_PanelPage::Create` · 005f8660 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own
- [x] `UIElement_PanelPage::Initialized` · 005f86a0 · legions 005b91d0 — ac2-ui panel.rs
- [x] `UIElement_PanelPage::SetState` · 005f8760 · legions 005b9290 — ac2-ui panel.rs
- [x] `UIElement_PanelPage::MouseDown` · 005f87e0 · legions 005b9310 — ac2-ui panel.rs
- [x] `UIElement_PanelPage::MouseOverTop` · 005f8830 · legions 005b9360 — ac2-ui panel.rs
- [ ] `UIElement_PanelPage::Activate` · 005f8890 · legions 005b93c0 — needs port (check): UIElement_PanelPage::Activate behaviour; implemented area ac2-ui panel.rs but uncited
- [ ] `UIElement_PanelPage::Deactivate` · 005f88d0 · legions 005b9400 — needs port (check): UIElement_PanelPage::Deactivate behaviour; implemented area ac2-ui panel.rs but uncited
- [x] `UIElement_PanelPage::StartUseActive` · 005f8910 · legions 005b9440 — ac2-ui panel.rs
- [-] `UIElement_PanelPage::Register` · 005f8980 · legions 005b94b0 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own

## `portal/client/src/uielement/types/uielement_resizebar.cpp.c` (2/8)

### UIElement_Resizebar
- [x] `UIElement_Resizebar::MouseDown` · 005f8050 · legions 005b8b80 — ac2-ui resize.rs: the grip's border_* attributes pick the edge (border_of, MouseDown), the drag moves the edges within min/max and the display (resized_box, MouseResizeElement), a resize_line outline is applied at the release (StopResizing); resize_tests.rs on the HUD strip and the chat strip layouts
- [x] `UIElement_Resizebar::MouseUp` · 005f81d0 · legions 005b8d00 — ac2-ui resize.rs: the grip's border_* attributes pick the edge (border_of, MouseDown), the drag moves the edges within min/max and the display (resized_box, MouseResizeElement), a resize_line outline is applied at the release (StopResizing); resize_tests.rs on the HUD strip and the chat strip layouts
- [~] `UIElement_Resizebar::UpdateLayout` · 005f8230 · legions 005b8d60 — ac2-ui resize.rs: the grip reads its border and disabled attributes at the press; the grip's own shown or hidden state when disabled (UpdateLayout, SetState) is not modelled
- [~] `UIElement_Resizebar::SetState` · 005f8300 · legions 005b8e30 — ac2-ui resize.rs: the grip reads its border and disabled attributes at the press; the grip's own shown or hidden state when disabled (UpdateLayout, SetState) is not modelled
- [~] `UIElement_Resizebar::SetAttribute_Bool` · 005f8320 · legions 005b8e50 — ac2-ui resize.rs: the grip reads its border and disabled attributes at the press; the grip's own shown or hidden state when disabled (UpdateLayout, SetState) is not modelled
- [~] `UIElement_Resizebar::UIElement_Resizebar` · 005f8360 · legions 005b8e90 — ac2-ui resize.rs: the grip reads its border and disabled attributes at the press; the grip's own shown or hidden state when disabled (UpdateLayout, SetState) is not modelled
- [~] `UIElement_Resizebar::Create` · 005f83c0 — ac2-ui resize.rs: the grip reads its border and disabled attributes at the press; the grip's own shown or hidden state when disabled (UpdateLayout, SetState) is not modelled
- [~] `UIElement_Resizebar::Register` · 005f83f0 · legions 005b8f20 — ac2-ui resize.rs: the grip reads its border and disabled attributes at the press; the grip's own shown or hidden state when disabled (UpdateLayout, SetState) is not modelled

## `portal/client/src/uielement/types/uielement_scrollbar.cpp.c` (11/28)

### UIElement_Scrollbar
- [ ] `UIElement_Scrollbar::SetAttribute_ID` · 005f9a20 · legions 005ba550 — needs port (check): UIElement_Scrollbar::SetAttribute_ID behaviour; implemented area ac2-ui scrollbar.rs but uncited
- [x] `UIElement_Scrollbar::BroadcastMessageOnRoot` · 005f9a80 · legions 005ba5b0 — ac2-ui scrollbar.rs
- [ ] `UIElement_Scrollbar::HandleHideButton` · 005f9aa0 · legions 005ba5d0 — needs port (check): UIElement_Scrollbar::HandleHideButton behaviour; implemented area ac2-ui scrollbar.rs but uncited
- [ ] `UIElement_Scrollbar::HandleFillChild` · 005f9ae0 · legions 005ba610 — needs port (check): UIElement_Scrollbar::HandleFillChild behaviour; implemented area ac2-ui scrollbar.rs but uncited
- [~] `UIElement_Scrollbar::UpdateLayout` · 005f9b30 · legions 005ba660 — scrollbar.rs: widget placement, drag, touched position, stops, step buttons; proportional widgets, fills, paging, smooth movement, wheel and disabling not modelled
- [ ] `UIElement_Scrollbar::StartAnimation` · 005fa050 · legions 005bab80 — needs port (check): UIElement_Scrollbar::StartAnimation behaviour; implemented area ac2-ui scrollbar.rs but uncited
- [ ] `UIElement_Scrollbar::StopAnimation` · 005fa170 · legions 005baca0 — needs port (check): UIElement_Scrollbar::StopAnimation behaviour; implemented area ac2-ui scrollbar.rs but uncited
- [x] `UIElement_Scrollbar::ResizeTo` · 005fa1b0 · legions 005bace0 — ac2-ui scrollbar.rs
- [x] `UIElement_Scrollbar::RegisterForMouseMove` · 005fa1d0 · legions 005bad00 — ac2-ui scrollbar.rs
- [x] `UIElement_Scrollbar::HandleMoveTo` · 005fa200 · legions 005bad30 — ac2-ui scrollbar.rs
- [x] `UIElement_Scrollbar::HandleMouseMove` · 005fa330 · legions 005bae60 — ac2-ui scrollbar.rs
- [x] `UIElement_Scrollbar::HandleMoveSteps` · 005fa3a0 · legions 005baed0 — ac2-ui scrollbar.rs
- [-] `UIElement_Scrollbar::~UIElement_Scrollbar` · 005fa4a0 · legions 005bafd0 — element creation and teardown are UiTree::create_element and Drop; no listener registration (see UIElementManager::RegisterForMessage)
- [x] `UIElement_Scrollbar::SetState` · 005fa5b0 · legions 005bb0e0 — ac2-ui scrollbar.rs
- [ ] `UIElement_Scrollbar::SetAttribute_Bool` · 005fa5d0 · legions 005bb100 — needs port (check): UIElement_Scrollbar::SetAttribute_Bool behaviour; implemented area ac2-ui scrollbar.rs but uncited
- [~] `UIElement_Scrollbar::SetAttribute_Float` · 005fa780 · legions 005bb2b0 — scrollbar.rs: widget placement, drag, touched position, stops, step buttons; proportional widgets, fills, paging, smooth movement, wheel and disabling not modelled
- [ ] `UIElement_Scrollbar::SetAttribute_Int` · 005fa820 · legions 005bb350 — needs port (check): UIElement_Scrollbar::SetAttribute_Int behaviour; implemented area ac2-ui scrollbar.rs but uncited
- [x] `UIElement_Scrollbar::MouseUp` · 005fa860 · legions 005bb390 — ac2-ui scrollbar.rs
- [~] `UIElement_Scrollbar::Global_Loop` · 005fa910 · legions 005bb440 — ac2-ui scrollbar.rs scrollbar_global_loop: a held track press pages again every page_interval (0.1 s) after page_first_interval (0.3 s), while the pointer has not reached the widget and the bar can move that way (test list_scroll_tests a_track_press_held_pages_again). Not: the smooth-movement animation (StartAnimation/StopAnimation: nothing in the ported code sets goal_x/y_position yet); retail arms the repeat time only for a vertical press, a horizontal-only press does not repeat here (retail would fire every frame from time 0)
- [x] `UIElement_Scrollbar::Global_MouseMove` · 005fabe0 · legions 005bb710 — ac2-ui scrollbar.rs
- [~] `UIElement_Scrollbar::ListenToMessage` · 005facd0 · legions 005bb800 — scrollbar.rs: widget placement, drag, touched position, stops, step buttons; proportional widgets, fills, paging, smooth movement, wheel and disabling not modelled
- [x] `UIElement_Scrollbar::HandleMouseWheel` · 005fad20 · legions 005bb850 — wheel: the element under the pointer hands the turn to its vertical bar (text_scroll.rs text_wheel, tree_scroll.rs tree_wheel, list_scroll.rs list_wheel); the bar steps its stop, or the lists, texts and trees it drives one line (scrollbar.rs scrollbar_mouse_wheel). Real-data: scroll_coverage_tests.rs over every layout; synthetic: list_scroll_tests.rs
- [x] `UIElement_Scrollbar::HandleButtonHotclick` · 005fadd0 · legions 005bb900 — ac2-ui scrollbar.rs
- [~] `UIElement_Scrollbar::UIElement_Scrollbar` · 005fae40 · legions 005bb970 — scrollbar.rs: widget placement, drag, touched position, stops, step buttons; proportional widgets, fills, paging, smooth movement, wheel and disabling not modelled
- [-] `UIElement_Scrollbar::Create` · 005fb0a0 — element creation and teardown are UiTree::create_element and Drop; no listener registration (see UIElementManager::RegisterForMessage)
- [~] `UIElement_Scrollbar::MouseDown` · 005fb0d0 · legions 005bbc00 — scrollbar.rs: wheel, widget press/drag, touched position, a track press pages once and arms the held repeat (Global_Loop paging, vertical presses). Not modelled: the horizontal-only repeat
- [~] `UIElement_Scrollbar::ListenToMessage` · 005fb400 · legions 005bbf30 — scrollbar.rs: widget placement, drag, touched position, stops, step buttons; proportional widgets, fills, paging, smooth movement, wheel and disabling not modelled
- [-] `UIElement_Scrollbar::Register` · 005fb4c0 · legions 005bbff0 — element creation and teardown are UiTree::create_element and Drop; no listener registration (see UIElementManager::RegisterForMessage)

## `portal/client/src/uielement/types/uielement_tree.cpp.c` (26/39)

### UIElement_Tree
- [x] `UIElement_Tree::CalculatePaperDimen` · 005f5d90 · legions 005b68c0 — ac2-ui tree_view.rs
- [x] `UIElement_Tree::MultiOpStart` · 005f5e20 · legions 005b6950 — ac2-ui tree_view.rs
- [ ] `UIElement_Tree::DrawMeBotToTop` · 005f5e30 · legions 005b6960 — needs port (check): UIElement_Tree::DrawMeBotToTop behaviour; implemented area ac2-ui tree_view.rs / tree_scroll.rs but uncited
- [x] `UIElement_Tree::GetPercentageV` · 005f5e50 · legions 005b6980 — ac2-ui tree_scroll.rs: UiTree::tree_percentage (vertical and horizontal share of paper - extent)
- [x] `UIElement_Tree::GetPercentageH` · 005f5ee0 · legions 005b6a10 — ac2-ui tree_scroll.rs: UiTree::tree_percentage (vertical and horizontal share of paper - extent)
- [x] `UIElement_Tree::VerticalPageDecrement` · 005f5f70 · legions 005b6aa0 — ac2-ui tree_scroll.rs: UiTree::tree_goal_step (node steps and page = extent - node, within 0 and paper - extent; the vertical page steps are reached by the bar's page messages, scrollbar_page); no horizontal bar, and a smooth tree jumps to the goal
- [x] `UIElement_Tree::VerticalPageIncrement` · 005f5fc0 · legions 005b6af0 — ac2-ui tree_scroll.rs: UiTree::tree_goal_step (node steps and page = extent - node, within 0 and paper - extent; the vertical page steps are reached by the bar's page messages, scrollbar_page); no horizontal bar, and a smooth tree jumps to the goal
- [x] `UIElement_Tree::HorizontalPageDecrement` · 005f6020 · legions 005b6b50 — ac2-ui tree_scroll.rs: UiTree::tree_goal_step (node steps and page = extent - node, within 0 and paper - extent; the vertical page steps are reached by the bar's page messages, scrollbar_page); no horizontal bar, and a smooth tree jumps to the goal
- [x] `UIElement_Tree::HorizontalPageIncrement` · 005f6070 · legions 005b6ba0 — ac2-ui tree_scroll.rs: UiTree::tree_goal_step (node steps and page = extent - node, within 0 and paper - extent; the vertical page steps are reached by the bar's page messages, scrollbar_page); no horizontal bar, and a smooth tree jumps to the goal
- [x] `UIElement_Tree::VerticalLineDecrement` · 005f60c0 · legions 005b6bf0 — ac2-ui tree_scroll.rs: UiTree::tree_goal_step (node steps and page = extent - node, within 0 and paper - extent; the vertical page steps are reached by the bar's page messages, scrollbar_page); no horizontal bar, and a smooth tree jumps to the goal
- [x] `UIElement_Tree::VerticalLineIncrement` · 005f6100 · legions 005b6c30 — ac2-ui tree_scroll.rs: UiTree::tree_goal_step (node steps and page = extent - node, within 0 and paper - extent; the vertical page steps are reached by the bar's page messages, scrollbar_page); no horizontal bar, and a smooth tree jumps to the goal
- [x] `UIElement_Tree::HorizontalLineDecrement` · 005f6150 · legions 005b6c80 — ac2-ui tree_scroll.rs: UiTree::tree_goal_step (node steps and page = extent - node, within 0 and paper - extent; the vertical page steps are reached by the bar's page messages, scrollbar_page); no horizontal bar, and a smooth tree jumps to the goal
- [x] `UIElement_Tree::HorizontalLineIncrement` · 005f6190 · legions 005b6cc0 — ac2-ui tree_scroll.rs: UiTree::tree_goal_step (node steps and page = extent - node, within 0 and paper - extent; the vertical page steps are reached by the bar's page messages, scrollbar_page); no horizontal bar, and a smooth tree jumps to the goal
- [x] `UIElement_Tree::End` · 005f61e0 · legions 005b6d10 — ac2-ui tree_view.rs
- [x] `UIElement_Tree::DrawLine` · 005f6270 · legions 005b6da0 — ac2-ui tree_view.rs
- [x] `UIElement_Tree::GetNodeInGrid` · 005f6410 · legions 005b6f40 — ac2-ui tree_view.rs
- [x] `UIElement_Tree::StaticLayoutUpdate` · 005f6450 · legions 005b6f80 — ac2-ui tree_view.rs
- [x] `UIElement_Tree::GridLayoutUpdate` · 005f6500 · legions 005b7030 — ac2-ui tree_view.rs
- [~] `UIElement_Tree::UpdateControls` · 005f65f0 · legions 005b7120 — tree_scroll.rs: a wheel turn goes to the vertical bar (MouseDown), the bar's new_position sets y_view, UpdateControls sets the bar; horizontal bar and smooth goal_y_view not modelled
- [~] `UIElement_Tree::ListenToMessage` · 005f6a50 · legions 005b7580 — tree_scroll.rs: a wheel turn goes to the vertical bar (MouseDown), the bar's new_position sets y_view, UpdateControls sets the bar; horizontal bar and smooth goal_y_view not modelled
- [x] `UIElement_Tree::DrawDone` · 005f6d10 · legions 005b7840 — ac2-ui tree_view.rs
- [x] `UIElement_Tree::GetItem` · 005f6f50 · legions 005b7a80 — ac2-ui tree_view.rs
- [ ] `UIElement_Tree::SetAttribute_ID` · 005f6f70 · legions 005b7aa0 — needs port (check): UIElement_Tree::SetAttribute_ID behaviour; implemented area ac2-ui tree_view.rs / tree_scroll.rs but uncited
- [x] `UIElement_Tree::AddGridNode` · 005f70d0 · legions 005b7c00 — ac2-ui tree_view.rs
- [x] `UIElement_Tree::IsOccupied` · 005f71f0 · legions 005b7d20 — ac2-ui tree_view.rs
- [x] `UIElement_Tree::UpdateLayout` · 005f7220 · legions 005b7d50 — ac2-ui tree_view.rs
- [ ] `UIElement_Tree::SetSelectedItem` · 005f73e0 · legions 005b7f10 — needs port (check): UIElement_Tree::SetSelectedItem behaviour; implemented area ac2-ui tree_view.rs / tree_scroll.rs but uncited
- [x] `UIElement_Tree::SetItemsParent` · 005f74f0 · legions 005b8020 — ac2-ui tree_view.rs
- [x] `UIElement_Tree::MultiOpComplete` · 005f7590 · legions 005b80c0 — ac2-ui tree_view.rs
- [x] `UIElement_Tree::AddItem` · 005f75c0 · legions 005b80f0 — ac2-ui tree_view.rs
- [x] `UIElement_Tree::FlushTree` · 005f76e0 · legions 005b8210 — ac2-ui tree_view.rs
- [-] `UIElement_Tree::~UIElement_Tree` · 005f7830 · legions 005b8360 — element creation and teardown: UiTree::create_element and Drop
- [~] `UIElement_Tree::SetAttribute_Bool` · 005f7b40 · legions 005b8670 — ac2-ui tree_view.rs: view via tree_end/flush only
- [~] `UIElement_Tree::SetAttribute_Int` · 005f7b90 · legions 005b86c0 — ac2-ui tree_view.rs: view via tree_end/flush only
- [~] `UIElement_Tree::MouseDown` · 005f7c50 · legions 005b8780 — tree_scroll.rs: a wheel turn goes to the vertical bar (MouseDown), the bar's new_position sets y_view, UpdateControls sets the bar; horizontal bar and smooth goal_y_view not modelled
- [ ] `UIElement_Tree::SetState` · 005f7dd0 · legions 005b8900 — needs port (check): UIElement_Tree::SetState behaviour; implemented area ac2-ui tree_view.rs / tree_scroll.rs but uncited
- [-] `UIElement_Tree::UIElement_Tree` · 005f7f30 · legions 005b8a60 — element creation and teardown: UiTree::create_element and Drop
- [-] `UIElement_Tree::Create` · 005f8000 — element creation and teardown: UiTree::create_element and Drop
- [-] `UIElement_Tree::Register` · 005f8030 · legions 005b8b60 — element creation and teardown: UiTree::create_element and Drop

## `portal/client/src/uielement/types/uielement_viewport.cpp.c` (7/10)

### UIElement_Viewport
- [x] `UIElement_Viewport::GetUISceneID` · 005f8410 · legions 005b8f40 — pregame 3D scenes: ac2-assets ac2_ui_scene.rs, ac2-client pregame_scene/, ac2-ui pregame/scene.rs
- [-] `UIElement_Viewport::Create` · 005f8420 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own
- [x] `UIElement_Viewport::SetCameraID` · 005f8480 · legions 005b8fb0 — pregame 3D scenes: ac2-assets ac2_ui_scene.rs, ac2-client pregame_scene/, ac2-ui pregame/scene.rs
- [x] `UIElement_Viewport::GetCameraID` · 005f84d0 · legions 005b9000 — pregame 3D scenes: ac2-assets ac2_ui_scene.rs, ac2-client pregame_scene/, ac2-ui pregame/scene.rs
- [x] `UIElement_Viewport::GetUIScene` · 005f84f0 · legions 005b9020 — pregame 3D scenes: ac2-assets ac2_ui_scene.rs, ac2-client pregame_scene/, ac2-ui pregame/scene.rs
- [ ] `UIElement_Viewport::ReleaseUIScene` · 005f8510 · legions 005b9040 — needs port (check): UIElement_Viewport::ReleaseUIScene behaviour; implemented area ac2-ui scene_viewport.rs but uncited
- [-] `UIElement_Viewport::Register` · 005f8560 · legions 005b9090 — Element construction, creation and teardown: UiTree::create_element builds the element from its layout description (kind dispatch by match), the arena slot is freed by UiTree::delete / Drop, and class registration (Register/Create) has no table to fill; no behaviour of its own
- [x] `UIElement_Viewport::SetAttribute_DataID` · 005f8580 · legions 005b90b0 — pregame 3D scenes: ac2-assets ac2_ui_scene.rs, ac2-client pregame_scene/, ac2-ui pregame/scene.rs
- [x] `UIElement_Viewport::SetAttribute_ASCII` · 005f85e0 · legions 005b9110 — pregame 3D scenes: ac2-assets ac2_ui_scene.rs, ac2-client pregame_scene/, ac2-ui pregame/scene.rs
- [x] `UIElement_Viewport::SetObject` · 005f8620 · legions 005b9150 — pregame 3D scenes: ac2-assets ac2_ui_scene.rs, ac2-client pregame_scene/, ac2-ui pregame/scene.rs
