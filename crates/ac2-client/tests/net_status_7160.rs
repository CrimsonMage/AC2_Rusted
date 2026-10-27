//! The net indicator on the HUD against the Legions server double (`AC2_DAT_DIR`, a GPU
//! adapter): each world heartbeat (`SessionEvent::LinkHeartbeat`, every two seconds of the
//! client's clock) sets the indicator's tooltip and its state (`gmNetStatusUI::OnHeartbeat`).

#![cfg(feature = "window")]

mod hud;

use ac2_ui::tokenize;
use ac2_weenie::SessionEvent;
use hud::Rig;

#[test]
fn a_heartbeat_sets_the_net_indicator_tooltip_and_state() {
    let Some(mut rig) = Rig::new() else {
        return;
    };
    assert!(
        rig.view.net_indicator().is_none(),
        "no heartbeat has come yet"
    );
    let mut heartbeats = 0;
    // Two seconds of client time is 128 frames at `hud::STEP`; allow for the first heartbeat
    // to start its interval late.
    for _ in 0..600 {
        heartbeats += rig
            .frame(&[])
            .iter()
            .filter(|e| matches!(e, SessionEvent::LinkHeartbeat(_)))
            .count();
        if heartbeats > 0 {
            break;
        }
    }
    assert!(heartbeats > 0, "the session sent no link heartbeat");
    // The tooltip is `ID_Indicator_TT_NetStatus` and `ID_Indicator_TT_NetStatusBW`: the latency
    // and loss line, then the bandwidth line.
    let (tooltip, state) = rig
        .view
        .net_indicator()
        .expect("the heartbeat sets the indicator");
    assert_eq!(tooltip.matches('\n').count(), 1, "{tooltip:?}");
    // The layout's `good` threshold is 0 s, so `stateless` (data newer than `good`) never
    // applies; the data is 15 ms old, which is `good` (`bad` is 10 s, `ugly` 30 s).
    assert_eq!(state, tokenize("good"));
}
