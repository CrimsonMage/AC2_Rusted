use super::*;

#[test]
fn hresult_success_is_the_sign_bit() {
    assert!(Hresult::S_OK.succeeded());
    assert!(zt::ZT_S_ALREADY_EXISTS.succeeded());
    assert!(!Hresult::E_PENDING.succeeded());
    assert!(!zt::ZT_E_NOT_CONNECTED.succeeded());
}

/// `TurbineGun::Read` routes on `id & 0xf00`; `TurbineGun::RcvResult` on `id & 0xff000`.
#[test]
fn ids_split_into_kind_and_family() {
    assert_eq!(message_id::CONNECT_EVENT.kind(), MessageKind::Event);
    assert_eq!(message_id::CONNECT_EVENT.family(), 0x1000);
    assert_eq!(
        message_id::GET_FRIENDS_LIST_RESULT.kind(),
        MessageKind::Result
    );
    assert_eq!(message_id::GET_FRIENDS_LIST_RESULT.family(), 0x4000);
    assert_eq!(MessageId(0x1_0201).family(), 0x1_0000);
    assert_eq!(MessageId(0x0f101).kind(), MessageKind::Request);
}

/// The ids `decompiled-pdb/gundll/STUB_SPEC.md` and the client spec name.
#[test]
fn the_client_ids_are_the_pdbs() {
    for (id, value) in [
        (message_id::CONNECT_EVENT, 0x1300),
        (message_id::DISCONNECT_EVENT, 0x1301),
        (message_id::CONNECT_FAILED_EVENT, 0x1302),
        (message_id::GET_FRIENDS_LIST_RESULT, 0x4200),
        (message_id::FRIEND_START_EVENT, 0x430c),
        (message_id::FRIEND_STATUS_EVENT, 0x430d),
    ] {
        assert_eq!(id, MessageId(value));
    }
}

#[test]
fn the_tables_are_whole_and_unique() {
    assert_eq!(zt::ALL.len(), 103);
    let mut codes: Vec<u32> = zt::ALL.iter().map(|(hr, _)| hr.0).collect();
    codes.sort_unstable();
    codes.dedup();
    assert_eq!(codes.len(), zt::ALL.len(), "one name per ZT code");
    assert_eq!(message_id::ALL.len(), 79);
    assert!(message_id::ALL
        .iter()
        .any(|(id, name)| *id == message_id::CONNECT_EVENT && *name == "c_msgConnectEvent"));
}

/// Every PDB id is distinct except the two invalid ids, both 0 in the enum.
#[test]
fn message_ids_are_unique_but_the_invalid_pair() {
    let mut values: Vec<u32> = message_id::ALL.iter().map(|(id, _)| id.0).collect();
    values.sort_unstable();
    let before = values.len();
    values.dedup();
    assert_eq!(before - values.len(), 1);
    assert_eq!(message_id::INVALID_EVENT, message_id::INVALID_MESSAGE);
    assert_eq!(message_id::INVALID_EVENT, MessageId(0));
}

/// The PDB enums' values (`INDEX/manifests/01_enum_catalog.json`).
#[test]
fn the_enums_carry_the_pdbs_values() {
    use interfaces::{ConnectStatus as C, FlagOperation as O, FlagWord as W};
    assert_eq!(
        [
            C::Created,
            C::Connecting,
            C::Connected,
            C::Disconnected,
            C::Failed
        ]
        .map(|c| c as u32),
        [0, 1, 2, 3, 4]
    );
    assert_eq!([O::Get, O::Set, O::Clear].map(|o| o as u32), [0, 1, 2]);
    assert_eq!([W::Control, W::State].map(|w| w as u32), [0, 1]);
    assert_eq!(SupportCallType::LAG.0, 4);
    assert_eq!(SupportCallType::OTHER_COC.0, 57351);
}
