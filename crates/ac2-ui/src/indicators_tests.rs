//! `gmIndicatorsUI`'s children (net, PK, vitae) on the real gameplay layout (`AC2_DAT_DIR`).

use std::cell::RefCell;
use std::sync::Arc;

use ac2_db::ResourceStore;

use crate::fellow_info::FellowStrings;
use crate::net_status::LinkSample;
use crate::skill_tree::SkillVar;
use crate::token::tokenize;
use crate::{build_gameplay_hud, GameplayHud};

/// Strings as `id[vars]`, rendered variables in order.
#[derive(Default)]
struct Strings {
    rendered: RefCell<Vec<String>>,
}

impl FellowStrings for Strings {
    fn render(&self, string_id: u32, vars: &[(u32, SkillVar)]) -> String {
        let vars: Vec<String> = vars
            .iter()
            .map(|(_, v)| match v {
                SkillVar::Float { value, .. } => format!("{value:.1}"),
                other => format!("{other:?}"),
            })
            .collect();
        let text = format!("{string_id:08x}[{}]", vars.join(","));
        self.rendered.borrow_mut().push(text.clone());
        text
    }
}

fn hud() -> Option<GameplayHud> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    let store = Arc::new(ResourceStore::new(dataset));
    Some(build_gameplay_hud(&store, 1024, 768).unwrap())
}

#[test]
fn the_pk_field_shows_the_faction_status() {
    let Some(mut hud) = hud() else {
        return;
    };
    let indicators = &mut hud.indicators;
    let tree = &mut hud.tree;
    assert!(indicators.pk.state(tree).is_some(), "PKStatusField");
    // `gmPKStatusUI::RecvNotice_UpdateFactionStatus`: 1 peace, 2 control, 3 factional, 4 ffa.
    for (status, state) in [
        (1, "peace"),
        (2, "control"),
        (3, "factional"),
        (4, "ffa"),
        (0, "stateless"),
        (9, "stateless"),
    ] {
        indicators.pk.update_faction_status(tree, status);
        assert_eq!(
            indicators.pk.state(tree),
            Some(tokenize(state)),
            "status {status}"
        );
    }
}

#[test]
fn the_net_field_takes_the_heartbeat_tooltip_and_state() {
    let Some(mut hud) = hud() else {
        return;
    };
    let indicators = &mut hud.indicators;
    let tree = &mut hud.tree;
    assert!(indicators.net.state(tree).is_some(), "NetStatusField");
    let strings = Strings::default();
    let sample = LinkSample {
        round_trip: 0.25,
        packet_loss: 0.5,
        bytes_in_per_second: 1200.0,
        bytes_out_per_second: 300.0,
        since_last_data: 0.0,
    };
    indicators.net.on_heartbeat(tree, &strings, sample);

    // The tooltip carries the latency and the loss, then the bandwidth.
    let field = tree
        .roots()
        .iter()
        .find_map(|&r| tree.child_recursive(r, tokenize("NetStatusField")))
        .unwrap();
    let tip = tree.tooltip_text(field).unwrap();
    let net = format!("{:08x}[250.0,50.0]", tokenize("ID_Indicator_TT_NetStatus"));
    let bw = format!(
        "{:08x}[1200.0,300.0]",
        tokenize("ID_Indicator_TT_NetStatusBW")
    );
    assert_eq!(tip, format!("{net}\n{bw}"));

    // One column per graph pixel: the first written is the sample's bytes, then the offset moves.
    let (history, offset) = indicators.net.history();
    assert!(!history.is_empty());
    assert_eq!(history[0].bytes_received, 1200);
    assert_eq!(history[0].bytes_sent, 300);
    assert_eq!(offset, 1 % history.len());

    // The layout's thresholds are good 0, bad 10 and ugly 30 seconds: fresh data is `good`,
    // 20 seconds without is `bad`, and beyond 30 seconds `ugly`.
    assert_eq!(indicators.net.state(tree), Some(tokenize("good")));
    let since = |seconds: f32| LinkSample {
        since_last_data: seconds,
        ..sample
    };
    indicators.net.on_heartbeat(tree, &strings, since(20.0));
    assert_eq!(indicators.net.state(tree), Some(tokenize("bad")));
    indicators.net.on_heartbeat(tree, &strings, since(1.0e9));
    assert_eq!(indicators.net.state(tree), Some(tokenize("ugly")));
}

#[test]
fn the_indicators_hold_the_net_pk_and_vitae_children() {
    let Some(mut hud) = hud() else {
        return;
    };
    let GameplayHud {
        tree, indicators, ..
    } = &mut hud;
    indicators
        .vitae
        .update_vitae(tree, &Strings::default(), 100.0, 0.0, 0.0);
    assert_eq!(
        indicators.vitae.state(tree),
        Some(tokenize("stateless")),
        "full vitae"
    );
}
