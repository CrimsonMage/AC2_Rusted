//! The GUN stage of the retail start (`gmClient::StartupGUN`): connect to the `-gunserver`,
//! present the ticket (`-gunticket`), then a friends round trip, over `ac2_net::gun`.

use std::net::SocketAddr;
use std::time::{Duration, Instant};

use ac2_net::gun::{flag_op, flag_word, GunConnection, GunEvent, TurbineGun};

use crate::{Failure, FailureKind};

#[derive(Debug, Clone)]
pub struct GunProbe {
    pub server: SocketAddr,
    /// The ticket bytes (`-gunticketoverride`); AC2e's developer ticket is the account name.
    pub ticket: String,
    /// The world's name (`TurbineGun::SetWorldName`).
    pub world: String,
    /// A friend to add, list, and remove again.
    pub friend: String,
    pub timeout: Duration,
}

fn wait(
    connection: &mut GunConnection,
    timeout: Duration,
    what: &str,
    mut pick: impl FnMut(&GunEvent) -> bool,
) -> Result<GunEvent, Failure> {
    let started = Instant::now();
    while started.elapsed() < timeout {
        let mut events = connection.pump().into_iter();
        while let Some(event) = events.next() {
            if pick(&event) {
                // Later events of the batch belong to the next wait.
                connection.stash_events(events);
                return Ok(event);
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    Err(Failure::new(
        FailureKind::Timeout,
        format!("GUN: no {what}"),
    ))
}

fn hr_ok(hr: u32, what: &str) -> Result<(), Failure> {
    if hr == 0 {
        Ok(())
    } else {
        Err(Failure::new(
            FailureKind::Refused,
            format!("GUN: {what} failed, hr=0x{hr:08x}"),
        ))
    }
}

/// Runs the GUN stage, calling `report` with each step. Leaves the friends list as it found it.
pub fn run_gun(probe: &GunProbe, report: &mut dyn FnMut(String)) -> Result<(), Failure> {
    let mut gun = TurbineGun::new();
    gun.set_world_name(&probe.world);
    gun.set_authentication_ticket(probe.ticket.as_bytes());
    let mut connection = GunConnection::connect(gun, probe.server);
    wait(&mut connection, probe.timeout, "connect event", |e| {
        matches!(e, GunEvent::Connected | GunEvent::ConnectFailed)
    })
    .and_then(|event| match event {
        GunEvent::Connected => Ok(()),
        _ => Err(Failure::new(FailureKind::Refused, "GUN: connect failed")),
    })?;
    report(format!("GUN connected to {}", probe.server));

    let mut snapshot = None;
    connection.gun.request_present_authentication_ticket();
    let result = wait(&mut connection, probe.timeout, "ticket result", |e| {
        if let GunEvent::FriendStart(list) = e {
            snapshot = Some(list.len());
        }
        matches!(e, GunEvent::TicketResult { .. })
    })?;
    let GunEvent::TicketResult { hr } = result else {
        unreachable!("picked above")
    };
    hr_ok(hr, "ticket")?;
    report("GUN ticket accepted (PresentTicketRes hr=0)".into());

    // A friends round trip: add, list, set presence, remove.
    let friend = probe.friend.clone();
    connection.gun.request_friend_add(&friend);
    let added = wait(&mut connection, probe.timeout, "FriendAddRes", |e| {
        matches!(e, GunEvent::FriendAdd { .. })
    })?;
    if let GunEvent::FriendAdd { hr, .. } = added {
        hr_ok(hr, "FriendAdd")?;
    }
    report(format!("GUN friend {friend:?} added"));
    connection.gun.request_get_friends_list(true);
    let GunEvent::FriendsList { hr, friends, .. } =
        wait(&mut connection, probe.timeout, "friends list", |e| {
            matches!(e, GunEvent::FriendsList { .. })
        })?
    else {
        unreachable!("picked above")
    };
    hr_ok(hr, "GetFriendsList")?;
    if !friends.iter().any(|f| f.name.eq_ignore_ascii_case(&friend)) {
        return Err(Failure::new(
            FailureKind::Protocol,
            format!("GUN: {friend:?} is not in the friends list {friends:?}"),
        ));
    }
    report(format!(
        "GUN friends list: {}",
        friends
            .iter()
            .map(|f| format!("{} (mode {})", f.name, f.mode))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    connection.gun.request_set_location(0, "probe");
    connection
        .gun
        .request_flags(flag_word::STATE, flag_op::SET, 2);
    wait(&mut connection, probe.timeout, "location result", |e| {
        matches!(e, GunEvent::Location { .. })
    })?;
    wait(&mut connection, probe.timeout, "flags result", |e| {
        matches!(e, GunEvent::Flags { .. })
    })?;
    report("GUN presence set (SetLoc, Flags)".into());
    connection.gun.request_friend_delete(&friend);
    wait(&mut connection, probe.timeout, "FriendDeleteRes", |e| {
        matches!(e, GunEvent::FriendDelete { .. })
    })?;
    report(format!(
        "GUN friend {friend:?} removed; {} listed at start",
        snapshot.unwrap_or(0)
    ));
    connection.close();
    Ok(())
}
