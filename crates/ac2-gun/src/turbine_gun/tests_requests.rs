//! Tests of the `Request*` wrappers (`0c_client.md` section 3.5).

use super::tests::*;
use crate::interfaces::{Context, FlagOperation, FlagWord, SupportCallType, TagBlob};
use crate::test_support::Call;
use crate::zt::ZT_E_INTERNAL;
use crate::Hresult;

fn context_of(call: &Call) -> Context {
    match call {
        Call::FriendAdd(_, c)
        | Call::FriendDelete(_, c)
        | Call::IgnoredAdd(_, c)
        | Call::IgnoredDelete(_, c)
        | Call::GetFriendsList(c)
        | Call::GetIgnoredList(c) => *c,
        Call::Flags { context, .. }
        | Call::SetLocation { context, .. }
        | Call::SendToRoomEx { context, .. }
        | Call::SendToUserEx { context, .. }
        | Call::CallCreate { context, .. } => *context,
        other => panic!("no context in {other:?}"),
    }
}

#[test]
fn name_requests_convert_the_name_to_global_and_pass_the_context() {
    // RequestFriendAdd/Delete, RequestIgnoredAdd/Delete: LocalToGlobalName where retail calls it
    let rig = Rig::connected().with_world();
    let c = rig.probe("c");
    assert!(rig.gun.request_friend_add("Birch", Some(c.clone())));
    assert!(rig
        .gun
        .request_friend_delete("Birch@Other", Some(c.clone())));
    assert!(rig.gun.request_ignored_add("Oak", Some(c.clone())));
    assert!(rig.gun.request_ignored_delete("Oak", Some(c.clone())));
    let calls = rig.handle.calls();
    assert!(matches!(&calls[0], Call::FriendAdd(n, _) if n == "Birch@Legions"));
    assert!(matches!(&calls[1], Call::FriendDelete(n, _) if n == "Birch@Other"));
    assert!(matches!(&calls[2], Call::IgnoredAdd(n, _) if n == "Oak@Legions"));
    assert!(matches!(&calls[3], Call::IgnoredDelete(n, _) if n == "Oak@Legions"));
    // each request holds its own reference, under a distinct nonzero id
    let ids: Vec<u32> = calls.iter().map(|c| context_of(c).0).collect();
    assert!(ids.iter().all(|i| *i != 0));
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), 4);
    assert_eq!(rig.gun.pending_context_count(), 4);
}

#[test]
fn requests_are_refused_unless_connected_with_a_context() {
    let idle = Rig::new();
    let c = idle.probe("c");
    assert!(!idle.gun.request_friend_add("a", Some(c.clone())));
    assert!(!idle.gun.request_get_friends_list(Some(c.clone())));
    assert!(!idle.gun.request_set_location(1, "x", c.clone()));
    assert!(!idle.gun.request_call_create(4, "t", "d", Some(c.clone())));
    assert!(idle.handle.calls().is_empty());
    assert_eq!(idle.gun.pending_context_count(), 0);

    let rig = Rig::connected();
    assert!(!rig.gun.request_friend_add("a", None));
    assert!(!rig.gun.request_friend_delete("a", None));
    assert!(!rig.gun.request_ignored_add("a", None));
    assert!(!rig.gun.request_ignored_delete("a", None));
    assert!(!rig.gun.request_get_friends_list(None));
    assert!(!rig.gun.request_get_ignored_list(None));
    assert!(!rig
        .gun
        .request_flags(FlagWord::State, FlagOperation::Set, 1, None));
    assert!(!rig.gun.request_send_to_room_ex(1, "m", None, None));
    assert!(!rig.gun.request_send_to_user_ex("u", "m", None, None));
    assert!(!rig.gun.request_call_create(4, "t", "d", None));
    assert!(rig.handle.calls().is_empty());
    assert_eq!(rig.gun.pending_context_count(), 0);
}

#[test]
fn a_failing_hr_is_false_and_leaves_the_addref() {
    let rig = Rig::connected();
    rig.handle.script("friend_add", ZT_E_INTERNAL);
    assert!(!rig.gun.request_friend_add("a", Some(rig.probe("c"))));
    assert_eq!(rig.gun.pending_context_count(), 1);
    // E_PENDING is negative too: PrintCmdHRESULT(hr >= 0)
    rig.handle.script("get_friends_list", Hresult::E_PENDING);
    assert!(!rig.gun.request_get_friends_list(Some(rig.probe("c"))));
    // a positive hr is a success
    rig.handle.script("get_ignored_list", Hresult(1));
    assert!(rig.gun.request_get_ignored_list(Some(rig.probe("c"))));
}

#[test]
fn list_requests_pass_only_the_context() {
    let rig = Rig::connected();
    assert!(rig.gun.request_get_friends_list(Some(rig.probe("c"))));
    assert!(rig.gun.request_get_ignored_list(Some(rig.probe("c"))));
    let calls = rig.handle.calls();
    assert!(matches!(calls[0], Call::GetFriendsList(_)));
    assert!(matches!(calls[1], Call::GetIgnoredList(_)));
}

#[test]
fn request_flags_swaps_which_and_op_for_the_interface() {
    // RequestFlags(which, op, bits, ctx) -> IGunFriends::Flags(op, which, bits, ctx)
    let rig = Rig::connected();
    assert!(rig.gun.request_flags(
        FlagWord::State,
        FlagOperation::Set,
        0x18,
        Some(rig.probe("c"))
    ));
    let Call::Flags {
        op, which, bits, ..
    } = &rig.handle.calls()[0]
    else {
        panic!()
    };
    assert_eq!(
        (*op, *which, *bits),
        (FlagOperation::Set, FlagWord::State, 0x18)
    );
}

#[test]
fn set_location_does_not_convert_the_location() {
    let rig = Rig::connected().with_world();
    assert!(rig.gun.request_set_location(8, "Hall", rig.probe("c")));
    let Call::SetLocation { mode, location, .. } = &rig.handle.calls()[0] else {
        panic!()
    };
    assert_eq!((*mode, location.as_str()), (8, "Hall"));
}

#[test]
fn send_requests_use_a_zero_blob_for_none_and_convert_only_the_user() {
    let rig = Rig::connected().with_world();
    assert!(rig
        .gun
        .request_send_to_room_ex(0x77, "Birch hi", None, Some(rig.probe("c"))));
    let blob = TagBlob {
        cb_size: 2,
        data: Some(vec![5, 6]),
    };
    assert!(rig
        .gun
        .request_send_to_user_ex("Birch", "psst", Some(&blob), Some(rig.probe("c"))));
    assert!(rig
        .gun
        .request_send_to_user_ex("Oak", "no blob", None, Some(rig.probe("c"))));
    let calls = rig.handle.calls();
    assert!(matches!(
        &calls[0],
        Call::SendToRoomEx { room_id: 0x77, message, blob, .. } if message == "Birch hi" && blob.is_empty()
    ));
    assert!(matches!(
        &calls[1],
        Call::SendToUserEx { user, message, blob, .. }
            if user == "Birch@Legions" && message == "psst" && blob == &[5, 6]
    ));
    assert!(matches!(
        &calls[2],
        Call::SendToUserEx { user, blob, .. } if user == "Oak@Legions" && blob.is_empty()
    ));
}

#[test]
fn call_create_passes_the_lcid_an_empty_blob_and_a_global_target() {
    // RequestCallCreate (admin 0x00607100 / Legions 0x005C7FA0)
    let rig = Rig::connected().with_world();
    assert!(rig
        .gun
        .request_call_create(0x8005, "Birch", "stuck", Some(rig.probe("c"))));
    assert!(rig.gun.request_call_create(4, "", "", Some(rig.probe("c"))));
    let calls = rig.handle.calls();
    let Call::CallCreate {
        call_type,
        target,
        description,
        blob,
        locale,
        ..
    } = &calls[0]
    else {
        panic!()
    };
    assert_eq!(*call_type, SupportCallType(0x8005));
    assert_eq!(target.as_deref(), Some("Birch@Legions"));
    assert_eq!(description.as_deref(), Some("stuck"));
    assert!(blob.is_empty());
    assert_eq!(*locale, LCID);
    let Call::CallCreate {
        target,
        description,
        ..
    } = &calls[1]
    else {
        panic!()
    };
    // retail passes NULL for an empty target and description
    assert!(target.is_none() && description.is_none());
}
