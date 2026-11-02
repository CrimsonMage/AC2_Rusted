//! Tests of `TurbineGunUtils`, `TurbineUserStatus` and `CTTicketResult`.

use crate::ct::CtTicketState;
use crate::interfaces::TagBlob;
use crate::test_support::{TestGunHandle, TestLoader};
use crate::turbine_gun_utils::*;
use crate::turbine_user_status::TurbineUserStatus;
use crate::zt::*;
use crate::Hresult;

#[test]
fn mode_text_follows_the_bits_and_the_cap() {
    // GetModeText (admin 0x006075D0 / Legions 0x005C8470)
    assert_eq!(get_mode_text(0, 0xff), "Logged Out ");
    assert_eq!(get_mode_text(1, 0xff), "Logged In ");
    assert_eq!(
        get_mode_text(0x1f, 0xff),
        "Logged In Away From Keyboard Do Not Disturb In Game In Chat "
    );
    assert_eq!(get_mode_text(0x18, 0xff), "In Game In Chat ");
    // each word needs cap >= len + 2 (mode 0: 0xd), and the cap shrinks by len + 1
    assert_eq!(get_mode_text(0, 0xc), "");
    assert_eq!(get_mode_text(0, 0xd), "Logged Out ");
    assert_eq!(get_mode_text(3, 0x1a), "Logged In ");
    assert_eq!(get_mode_text(3, 0x20), "Logged In Away From Keyboard ");
}

#[test]
fn room_mode_text() {
    // GetRoomModeText (admin 0x00607690 / Legions 0x005C8530)
    assert_eq!(get_room_mode_text(1, 0x50), "STANDARD");
    assert_eq!(get_room_mode_text(2, 0x50), "USER");
    assert_eq!(get_room_mode_text(4, 0x50), "PASSWORD");
    assert_eq!(get_room_mode_text(8, 0x50), "PRIVATE");
    assert_eq!(get_room_mode_text(0x10, 0x50), "SECRET");
    assert_eq!(get_room_mode_text(3, 0x50), "Unknown Mode ");
    assert_eq!(get_room_mode_text(1, 9), "");
    assert_eq!(get_room_mode_text(1, 10), "STANDARD");
    assert_eq!(get_room_mode_text(2, 5), "");
    assert_eq!(get_room_mode_text(0x10, 8), "SECRET");
    assert_eq!(get_room_mode_text(0, 0xe), "");
}

#[test]
fn string_from_hr_scans_the_112_entry_table() {
    // StringFromHR (admin 0x006078B0 / Legions 0x005C8750)
    assert_eq!(crate::hr_table::GUN_ERRORS.len(), 112);
    assert_eq!(string_from_hr(Hresult(0)), "S_OK");
    assert_eq!(string_from_hr(Hresult::E_PENDING), "E_PENDING");
    assert_eq!(string_from_hr(ZT_E_INTERNAL), "ZT_E_INTERNAL");
    assert_eq!(string_from_hr(Hresult(0x8000_4005)), "E_FAIL");
    assert_eq!(string_from_hr(Hresult(0x102)), "WAIT_TIMEOUT");
    assert_eq!(string_from_hr(Hresult(0x8007_2AF9)), "HOST NOT FOUND");
    assert_eq!(string_from_hr(ZT_E_INVALID_OWNERID), "ZT_E_INVALID_OWNERID");
    // not in the table: FormatMessageA's text, not available
    assert_eq!(string_from_hr(Hresult(0x1234_5678)), "");
}

#[test]
fn print_cmd_hresult_is_the_success_test() {
    // PrintCmdHRESULT (admin 0x00607930 / Legions 0x005C87D0): hr >= 0
    assert!(print_cmd_hresult("x", Hresult(0)));
    assert!(print_cmd_hresult("x", Hresult(1)));
    assert!(print_cmd_hresult("x", Hresult(0x4112_001B)));
    assert!(!print_cmd_hresult("x", Hresult::E_PENDING));
    assert!(!print_cmd_hresult("x", ZT_E_INTERNAL));
}

#[test]
fn gun_create_instance_steps() {
    // GunCreateInstance (admin 0x00607780 / Legions 0x005C8620)
    let handle = TestGunHandle::new();
    let mut loader = TestLoader::new(handle.gun());
    let mut module = loader
        .load_library("GunDll.dll")
        .unwrap_or_else(|_| panic!());
    let clsid = crate::turbine_gun::CLSID_GUN;
    let (hr, object) = gun_create_instance(
        module.as_mut(),
        &clsid,
        &crate::turbine_gun::IID_IGUN_CONNECTION,
    );
    assert_eq!(hr, Hresult::S_OK);
    assert!(object.is_some());
    // a missing export returns GetLastError(), a positive number
    let mut loader = TestLoader::new(handle.gun());
    loader.export_missing = Some(127);
    let mut module = loader.load_library("x").unwrap_or_else(|_| panic!());
    let (hr, object) = gun_create_instance(module.as_mut(), &clsid, &clsid);
    assert_eq!(hr, Hresult(127));
    assert!(hr.succeeded());
    assert!(object.is_none());
}

#[test]
fn world_statics_follow_the_current_gun() {
    // GetWorldName / GetWorldSuffix / GlobalToLocalName (statics over S_pTurbineGun)
    let gun = crate::TurbineGun::new(0x409);
    assert_eq!(get_world_name(), "");
    assert_eq!(global_to_local_name("Birch@Legions"), "Birch@Legions");
    gun.set_world_name("Legions");
    assert_eq!(get_world_name(), "Legions");
    assert_eq!(get_world_suffix(), "@Legions");
    assert_eq!(global_to_local_name("Birch@Legions"), "Birch");
    assert_eq!(global_to_local_name("Birch@Other"), "Birch@Other");
}

#[test]
fn turbine_user_status_members() {
    // TurbineUserStatus (admin 0x006047D0 ...)
    let mut s = TurbineUserStatus::new();
    assert_eq!(
        (s.inq_name(), s.inq_location(), s.inq_mode()),
        (None, None, 0)
    );
    s.name = "Birch".into();
    s.set_location("Hall");
    s.mode = 9;
    assert_eq!(s.inq_name().as_deref(), Some("Birch"));
    assert_eq!(s.inq_location().as_deref(), Some("Hall"));
    assert_eq!(s.inq_mode(), 9);
    let copy = s.clone();
    s.clear();
    assert_eq!(s, TurbineUserStatus::default());
    assert_eq!(copy.mode, 9);
}

#[test]
fn turbine_user_status_stream_pack_in_retails_layout_and_size_accounting() {
    use crate::turbine_user_status::StreamType;
    // StreamPack (admin 0x00604880): name, location (u16 length, munged characters, padded to
    // 4), then the mode as 4 bytes.
    let mut status = TurbineUserStatus {
        name: "Birch".into(),
        location: String::new(),
        mode: 0x11,
    };
    // 2 + 5*2 = 12 (aligned), 2 -> 4, then 4
    let mut size = 100u32;
    assert!(status.stream_pack(StreamType::GetPackSize, &mut [], &mut 0, &mut size));
    assert_eq!(size, 100 - 20);
    let mut out = vec![0xAAu8; 20];
    let (mut pos, mut size) = (0usize, 20u32);
    assert!(status.stream_pack(StreamType::Pack, &mut out, &mut pos, &mut size));
    assert_eq!((pos, size), (16 + 4, 0));
    assert_eq!(&out[..2], &[5, 0]);
    // the padding after an empty string is not written; the length is
    assert_eq!(&out[12..14], &[0, 0]);
    assert_eq!(&out[16..], &[0x11, 0, 0, 0]);
    let mut back = TurbineUserStatus::new();
    let (mut pos, mut size) = (0usize, 20u32);
    assert!(back.stream_pack(StreamType::UnPack, &mut out, &mut pos, &mut size));
    assert_eq!(back, status);
    assert_eq!((pos, size), (20, 0));
}

#[test]
fn stream_pack_always_returns_true_and_pack_refuses_what_does_not_fit() {
    use crate::turbine_user_status::StreamType;
    let mut status = TurbineUserStatus {
        name: "Birch".into(),
        location: "Hall".into(),
        mode: 7,
    };
    // Pack with size 12: the name (12) fits, the location (2 + 8 -> 12) does not (nothing
    // written, size still drops), the mode (size is then 0 - 12 wrapped > 3) is written.
    let mut out = vec![0u8; 64];
    let (mut pos, mut size) = (0usize, 12u32);
    assert!(status.stream_pack(StreamType::Pack, &mut out, &mut pos, &mut size));
    assert_eq!(pos, 12 + 4);
    assert_eq!(size, 12u32.wrapping_sub(12 + 12 + 4));
    assert_eq!(&out[12..16], &[7, 0, 0, 0]);

    // UnPack with 4 bytes left: the name (length 0) clears and the cursor aligns to 4; the
    // location (size < 2) and the mode (size < 4) fail, and StreamPack still returns true with
    // those members as they were.
    let zeros = vec![0u8; 8];
    let mut target = TurbineUserStatus {
        name: "old".into(),
        location: "old".into(),
        mode: 99,
    };
    let (mut pos, mut size) = (0usize, 4u32);
    assert!(target.stream_pack(StreamType::UnPack, &mut zeros.clone(), &mut pos, &mut size));
    assert_eq!(target.name, "");
    assert_eq!((target.location.as_str(), target.mode), ("old", 99));
    assert_eq!((pos, size), (4, 0));
}

#[test]
fn ticket_state_adopt_and_copy() {
    // AdoptTicket (admin 0x006074B0), CopyTicket (admin 0x00607500)
    let state = CtTicketState::new();
    assert!(state.get_ticket_blob().is_none());
    assert!(state.adopt_ticket(TagBlob {
        cb_size: 2,
        data: Some(vec![1, 2])
    }));
    assert_eq!(state.get_ticket_blob().unwrap().bytes(), [1, 2]);
    let src = TagBlob {
        cb_size: 3,
        data: Some(vec![7, 8, 9]),
    };
    assert!(state.copy_ticket(&src));
    let copy = state.get_ticket_blob().unwrap();
    assert_eq!((copy.cb_size, copy.bytes()), (3, &[7u8, 8, 9][..]));
}
