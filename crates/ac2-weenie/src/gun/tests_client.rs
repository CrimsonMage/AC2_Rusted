//! Tests of the gmClient half: `StartupGUN`'s sequence, `AuthenticateWithGUN`, `InitAuth`, the
//! notices and the destructor, against `ac2-gun`'s recording `TestGun`.

use ac2_gun::message::MessageBody;
use ac2_gun::message_id as id;
use ac2_gun::test_support::{Call, TestGunHandle, TestLoader};
use ac2_gun::{Context, Hresult};
use ac2_net::{Credential, Logon};

use super::client::{
    ConnectPoll, GunClient, GUN_ACCOUNT_BANNED, GUN_FAILED, REAUTH_TABLE, UI_TABLE,
};
use super::options::GunOptions;
use super::output::GunOutput;
use super::shared::shared;
use super::tests::{event, gun_options, result, rig, Rig};

const APP_GUID_DATA1: u32 = 0x2280_767d;

fn ticket_options() -> GunOptions {
    GunOptions {
        use_gun_ticket: true,
        ..gun_options("gun.example")
    }
}

fn poll(rig: &Rig) -> ConnectPoll {
    rig.client.connect(&|| 0x0001_2345, true)
}

#[test]
fn gun_is_off_without_a_gunserver() {
    let rig = rig(GunOptions::default(), "");
    assert!(rig.client.turbine_gun().is_none());
    assert_eq!(poll(&rig), ConnectPoll::Ready(Credential::Account));
    assert!(rig.handle.calls().is_empty());
}

#[test]
fn init_sets_the_override_ticket_with_its_terminating_nul() {
    let options = GunOptions {
        gun_ticket_override: "tick".into(),
        ..gun_options("gun.example")
    };
    let rig = rig(options, "");
    // cbSize = strlen + 1: the NUL goes along (SetAuthenticationTicket, Legions 0x005C6A80).
    assert_eq!(rig.handle.calls(), [Call::SetTicket(b"tick\0".to_vec())]);
}

#[test]
fn a_failed_gun_init_leaves_no_turbine_gun_and_connect_fails() {
    let handle = TestGunHandle::new();
    let mut loader = TestLoader::new(handle.gun());
    loader.fail_load = Some(2);
    let client = GunClient::new(
        gun_options("gun.example"),
        false,
        "",
        0x0409,
        Box::new(loader),
    );
    client.init();
    assert!(client.turbine_gun().is_none());
    // StartupGUN makes the TurbineGun again; a second failure is ID_DataPatch_GUNFailed.
    assert_eq!(client.connect(&|| 0, true), ConnectPoll::Failed(GUN_FAILED));
}

#[test]
fn startup_gun_connects_and_waits_without_a_timeout() {
    let rig = rig(gun_options("gun.example"), "");
    for _ in 0..1000 {
        assert_eq!(poll(&rig), ConnectPoll::Pending);
    }
    let calls = rig.handle.calls();
    // Init(app GUID, L"1.0", the user's LCID for locale 0, L"XXXX-YYYY-ZZZZ", low 16 bits of
    // the preference), then Connect(server, 2310).
    let Call::Init {
        app,
        game_version,
        locale,
        cd_key,
        port_local,
    } = &calls[0]
    else {
        panic!("{calls:?}");
    };
    assert_eq!(app.data1, APP_GUID_DATA1);
    assert_eq!(game_version, "1.0");
    assert_eq!(*locale, 0x0409);
    assert_eq!(cd_key, "XXXX-YYYY-ZZZZ");
    assert_eq!(*port_local, 0x2345);
    assert_eq!(
        calls[1],
        Call::Connect {
            server: "gun.example".into(),
            port: 2310
        }
    );
    // Each pending frame polled DispatchQueue (CheckForMessages).
    assert!(calls[2..].iter().all(|c| *c == Call::DispatchQueue));
    assert!(calls.len() > 100);
    assert!(rig.client.starting_up());
    rig.handle
        .queue_message(event(id::CONNECT_EVENT, MessageBody::None));
    assert_eq!(poll(&rig), ConnectPoll::Ready(Credential::Account));
    assert!(!rig.client.starting_up());
    // The chat and friends listeners were registered after the connect.
    assert!(rig.client.turbine_gun().unwrap().is_connected());
}

#[test]
fn a_ui_that_is_gone_fails_the_wait() {
    let rig = rig(gun_options("gun.example"), "");
    assert_eq!(poll(&rig), ConnectPoll::Pending);
    assert_eq!(
        rig.client.connect(&|| 0, false),
        ConnectPoll::Failed(GUN_FAILED)
    );
}

#[test]
fn the_port_preference_is_read_only_when_the_local_port_is_zero() {
    let rig = rig(gun_options("gun.example"), "");
    rig.client.connect(&|| 0x1_0007, true);
    let Call::Init { port_local, .. } = &rig.handle.calls()[0] else {
        panic!()
    };
    assert_eq!(*port_local, 7, "only the low 16 bits are used");
}

#[test]
fn a_connect_failure_event_fails_startup() {
    for event_id in [id::CONNECT_FAILED_EVENT, id::DISCONNECT_EVENT] {
        let rig = rig(gun_options("gun.example"), "");
        poll(&rig);
        rig.handle.queue_message(event(event_id, MessageBody::None));
        assert_eq!(poll(&rig), ConnectPoll::Failed(GUN_FAILED));
    }
}

/// `gmClient::Connect` returns `StartupGUN`'s error and never calls `Turbine::Client::Connect`:
/// asked again, the answer stays the failure and no credential is ever made.
#[test]
fn a_failed_startup_stays_failed() {
    let rig = rig(gun_options("gun.example"), "");
    poll(&rig);
    rig.handle
        .queue_message(event(id::CONNECT_FAILED_EVENT, MessageBody::None));
    assert_eq!(poll(&rig), ConnectPoll::Failed(GUN_FAILED));
    rig.handle.clear_calls();
    for _ in 0..10 {
        assert_eq!(poll(&rig), ConnectPoll::Failed(GUN_FAILED));
    }
    assert!(rig.handle.calls().is_empty(), "no new connect attempt");
}

#[test]
fn a_refused_connect_fails_startup() {
    let rig = self::rig(gun_options("gun.example"), "");
    rig.handle.script("connect", Hresult(0x8000_4005));
    assert_eq!(poll(&rig), ConnectPoll::Failed(GUN_FAILED));
}

fn ticket_rig() -> Rig {
    let rig = rig(ticket_options(), "");
    rig.handle
        .set_ticket(Some(vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE]));
    assert_eq!(poll(&rig), ConnectPoll::Pending);
    rig.handle
        .queue_message(event(id::CONNECT_EVENT, MessageBody::None));
    // The connect event ends the first wait; AuthenticateWithGUN(false) then waits for the answer.
    assert_eq!(poll(&rig), ConnectPoll::Pending);
    rig
}

fn present_context(rig: &Rig) -> Context {
    rig.handle
        .calls()
        .into_iter()
        .find_map(|c| match c {
            Call::PresentTicket { context, .. } => Some(context),
            _ => None,
        })
        .expect("PresentTicket was called")
}

#[test]
fn startup_gun_with_a_ticket_waits_for_the_answer() {
    let rig = ticket_rig();
    let calls = rig.handle.calls();
    // GetTicket twice (the first answers BUFFER_TOO_SMALL), then PresentTicket with the bytes.
    assert_eq!(calls.iter().filter(|c| **c == Call::GetTicket).count(), 2);
    assert!(calls.iter().any(|c| matches!(
        c,
        Call::PresentTicket { ticket, .. } if ticket == &[0xAA, 0xBB, 0xCC, 0xDD, 0xEE]
    )));
    for _ in 0..50 {
        assert_eq!(poll(&rig), ConnectPoll::Pending);
    }
    let ctx = present_context(&rig);
    rig.handle
        .queue_message(result(id::PRESENT_TICKET_RESULT, ctx, 0, MessageBody::None));
    // InitAuth: empty primer, -gunticket on, a handler: auth type 0x41000001 with the ticket.
    let ConnectPoll::Ready(credential) = poll(&rig) else {
        panic!()
    };
    assert_eq!(
        credential,
        Credential::GunTicket(vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE])
    );
    let logon = Logon {
        version: "7160".into(),
        account: "acct".into(),
        credential,
        connection_sequence: 0,
        crypto: None,
    };
    let (_, auth_type, _, extra) = Logon::decode_payload(&logon.payload()).unwrap();
    assert_eq!(auth_type, 0x4100_0001);
    assert_eq!(extra, vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE]);
}

#[test]
fn a_banned_account_and_other_failures_end_startup() {
    for (hr, expected) in [
        (0xC112_1005, GUN_ACCOUNT_BANNED),
        (0xC112_1006, GUN_FAILED),
        (0xC112_080C, GUN_FAILED),
    ] {
        let rig = ticket_rig();
        let ctx = present_context(&rig);
        rig.handle.queue_message(result(
            id::PRESENT_TICKET_RESULT,
            ctx,
            hr,
            MessageBody::None,
        ));
        assert_eq!(poll(&rig), ConnectPoll::Failed(expected), "{hr:#x}");
        assert!(
            rig.client.ticket_handler().is_none(),
            "the handler is released"
        );
    }
    assert_eq!(GUN_ACCOUNT_BANNED.string, "ID_DataPatch_GUNAccountBanned");
    assert_eq!(GUN_FAILED.string, "ID_DataPatch_GUNFailed");
    assert_eq!(GUN_FAILED.table, UI_TABLE);
}

#[test]
fn a_ticket_that_cannot_be_fetched_fails_startup() {
    let rig = rig(ticket_options(), "");
    // No ticket: GetTicket answers NOT_TICKETED.
    poll(&rig);
    rig.handle
        .queue_message(event(id::CONNECT_EVENT, MessageBody::None));
    assert_eq!(poll(&rig), ConnectPoll::Failed(GUN_FAILED));
    assert!(rig.client.ticket_handler().is_none());
}

#[test]
fn init_auth_picks_the_authentication_type() {
    // Empty primer, no -gunticket: Turbine::Client::InitAuth (auth type 1).
    let rig0 = rig(GunOptions::default(), "");
    assert_eq!(rig0.client.init_auth(), Some(Credential::Account));
    // A primer: auth type 0x40000002 with the packed primer, even with -gunticket.
    let rig1 = rig(GunOptions::default(), "secret");
    assert_eq!(
        rig1.client.init_auth(),
        Some(Credential::Password("secret".into()))
    );
    // -gunticket without a handler (before AuthenticateWithGUN): the base InitAuth.
    let rig2 = rig(ticket_options(), "");
    assert_eq!(rig2.client.init_auth(), Some(Credential::Account));
    // The primer wins over a ticket handler.
    let rig3 = rig(ticket_options(), "pw");
    assert_eq!(
        rig3.client.init_auth(),
        Some(Credential::Password("pw".into()))
    );
}

#[test]
fn reconnect_reauthenticates_unconditionally_and_reports_through_the_ui_sink() {
    // No -gunticket at all: a RECONNECTED notice still calls AuthenticateWithGUN(true).
    let rig = rig(gun_options("gun.example"), "");
    rig.handle.set_ticket(Some(vec![1, 2, 3]));
    poll(&rig);
    rig.handle
        .queue_message(event(id::CONNECT_EVENT, MessageBody::None));
    assert!(matches!(poll(&rig), ConnectPoll::Ready(_)));
    shared().set_player(true);
    rig.handle.clear_calls();
    // Disconnect, then the next connect event is a reconnect (m_fPersistentConnection is set).
    rig.handle
        .queue_message(event(id::DISCONNECT_EVENT, MessageBody::None));
    rig.handle
        .queue_message(event(id::CONNECT_EVENT, MessageBody::None));
    rig.client.use_time();
    let calls = rig.handle.calls();
    assert!(
        calls
            .iter()
            .any(|c| matches!(c, Call::PresentTicket { ticket, .. } if ticket == &[1, 2, 3])),
        "{calls:?}"
    );
    let outputs = shared().take_outputs();
    assert!(outputs.contains(&GunOutput::Notice(super::output::GunNotice::ConnectionLost)));
    // The answer: the reauth flag is set, so a success runs GUNHandleReauthentication.
    let ctx = present_context(&rig);
    rig.handle
        .queue_message(result(id::PRESENT_TICKET_RESULT, ctx, 0, MessageBody::None));
    rig.client.use_time();
    assert_eq!(shared().take_outputs(), [GunOutput::HandleReauthentication]);
    // Another authenticate notice is ignored (the flag was cleared).
    rig.client.recv_notice_gun_authenticated(true);
    assert!(shared().take_outputs().is_empty());
}

#[test]
fn a_reauthentication_that_cannot_start_is_printed() {
    let rig = rig(gun_options("gun.example"), "");
    poll(&rig);
    rig.handle
        .queue_message(event(id::CONNECT_EVENT, MessageBody::None));
    poll(&rig);
    shared().take_outputs();
    // No ticket available: GetAuthenticationTicket fails.
    rig.client.recv_notice_gun_reconnected();
    assert_eq!(shared().take_outputs(), [GunOutput::UiNetError(GUN_FAILED)]);
    // With the string on disk the error is ID_GunReAuthenticationFailed (table 0x4100000E).
    rig.client.set_string_on_disk(Box::new(|name, table| {
        name == "ID_GunReAuthenticationFailed" && table == REAUTH_TABLE
    }));
    rig.client.recv_notice_gun_reconnected();
    let out = shared().take_outputs();
    assert!(matches!(
        &out[..],
        [GunOutput::UiNetError(e)] if e.string == "ID_GunReAuthenticationFailed" && e.table == 0x4100_000E
    ));
}

#[test]
fn the_world_name_reaches_the_turbine_gun_once() {
    let rig = rig(gun_options("gun.example"), "");
    rig.client.recv_notice_world_name("Legions");
    rig.client.recv_notice_world_name("Other");
    let gun = rig.client.turbine_gun().unwrap();
    assert_eq!(gun.world_name(), "Legions");
    assert_eq!(gun.world_suffix(), "@Legions");
    assert_eq!(ac2_gun::turbine_gun_utils::get_world_name(), "Legions");
    assert_eq!(ac2_gun::turbine_gun_utils::get_world_suffix(), "@Legions");
}

#[test]
fn use_time_polls_every_frame_and_the_destructor_releases_the_ticket() {
    let rig = rig(ticket_options(), "");
    rig.handle.set_ticket(Some(vec![1]));
    poll(&rig);
    rig.handle
        .queue_message(event(id::CONNECT_EVENT, MessageBody::None));
    poll(&rig);
    let ctx = present_context(&rig);
    rig.handle
        .queue_message(result(id::PRESENT_TICKET_RESULT, ctx, 0, MessageBody::None));
    assert!(matches!(poll(&rig), ConnectPoll::Ready(_)));
    rig.handle.clear_calls();
    rig.client.use_time();
    rig.client.use_time();
    assert_eq!(
        rig.handle.calls(),
        [Call::DispatchQueue, Call::DispatchQueue]
    );
    rig.handle.clear_calls();
    let Rig { handle, client } = rig;
    drop(client);
    // ~gmClient: RequestFinishWithTicket (ReleaseTicket), then the TurbineGun's destructor
    // disconnects.
    let calls = handle.calls();
    assert!(matches!(calls[0], Call::ReleaseTicket(_)), "{calls:?}");
    assert_eq!(calls[1], Call::Disconnect);
    assert!(shared().turbine_gun().is_none());
}
