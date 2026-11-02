//! Entering the world already dead against a running AC2e server (`AC2E_LIVE`, see
//! `tests/live/mod.rs`): the character dies, logs off and enters again.
//!
//! Run with `AC2E_LIVE_TEST=ac2e_live_death_login scripts/ac2e-live-test.sh`.

#![cfg(feature = "window")]

mod live;

use ac2_ui::tokenize;
use ac2_weenie::{
    ActiveAction, GameplayNotice, SessionEvent, ACTION_CLASS_MOVEMENT, ACTION_FORWARD,
};
use live::LiveRig;

fn notices(rig: &LiveRig) -> impl Iterator<Item = &GameplayNotice> {
    rig.seen.iter().filter_map(|e| match e {
        SessionEvent::Notice(n) => Some(n),
        _ => None,
    })
}

fn health(rig: &LiveRig) -> Option<(u64, u64)> {
    rig.client
        .frame()
        .player_vitals
        .health
        .map(|v| (v.current, v.max))
}

/// The resurrect dialog's button centre while the dialog shows.
fn dialog(rig: &LiveRig) -> Option<(i32, i32)> {
    let tree = rig.view.hud_tree()?;
    let root = *tree.roots().iter().find(|&&r| {
        tree.get(r)
            .is_some_and(|e| e.id() == tokenize("ResurrectField"))
    })?;
    let button = tree.child_recursive(root, tokenize("ResurrectButton"))?;
    let b = tree.screen_box(button);
    tree.get(root)
        .is_some_and(|e| e.visible())
        .then_some(((b.x0 + b.x1) / 2, (b.y0 + b.y1) / 2))
}

fn forward(rig: &mut LiveRig, seconds: f64) {
    rig.held = vec![ActiveAction {
        class: ACTION_CLASS_MOVEMENT,
        enum_value: ACTION_FORWARD,
        map_extent: 1.0,
        extent: 1.0,
    }];
    rig.run(seconds);
    rig.held.clear();
    rig.run(0.3);
}

/// What the health meter shows: (meter box, state, text).
fn meter(rig: &LiveRig) -> String {
    let Some(tree) = rig.view.hud_tree() else {
        return "no tree".into();
    };
    let Some(m) = tree
        .roots()
        .iter()
        .find_map(|&r| tree.child_recursive(r, tokenize("HealthMeter")))
    else {
        return "no meter".into();
    };
    let t = tree
        .roots()
        .iter()
        .find_map(|&r| tree.child_recursive(r, tokenize("HealthText")));
    format!(
        "meter box {:?} state {:#x} visible {} text {:?}",
        tree.screen_box(m),
        tree.get(m).map_or(0, |e| e.state()),
        tree.get(m).is_some_and(|e| e.visible()),
        t.and_then(|t| tree.get(t).map(|e| e.text()))
    )
}

/// Whether the status bar (the vitals panel) shows (`gmGamePlayUI::Show`).
fn statusbar_shown(rig: &LiveRig) -> Option<bool> {
    let tree = rig.view.hud_tree()?;
    let field = tree.roots().iter().find_map(|&r| {
        if tree
            .get(r)
            .is_some_and(|e| e.id() == tokenize("StatusbarField"))
        {
            Some(r)
        } else {
            tree.child_recursive(r, tokenize("StatusbarField"))
        }
    })?;
    tree.get(field).map(|e| e.visible())
}

/// A character saved dead enters the world dead: `CDeathSystem` 0x3's `UpdateDeathState` (the
/// server's `WM_Death` 0x33) raises the resurrect dialog (`gmGamePlayUI::RecvNotice_UpdateDeathState`
/// -> `MakeResurrectDialog`) and hides the gameplay panels (`gmGamePlayUI::Show(0)`, the vitals
/// with them) even though the login's notice comes before the HUD of the new session exists; the
/// death behaviour locks the movement keys (`PlayerMovementModule::LockActions`); the dialog's
/// button asks the respawn and the player lives and walks again.
#[test]
fn entering_the_world_dead() {
    let Some(mut rig) = LiveRig::new("Deadlogin") else {
        return;
    };
    rig.wait("no objects", |r| {
        r.client.session().objects().iter().count() > 5
    });
    let dead = rig.until(240.0, |r| {
        notices(r).any(|n| matches!(n, GameplayNotice::UpdateDeathState(true)))
    });
    assert!(dead, "the player never died; health {:?}", health(&rig));
    assert!(rig.until(5.0, |r| dialog(r).is_some()), "no dialog dying");
    // Dying in the session: the keys do nothing and the panels are hidden.
    let at = rig.origin();
    forward(&mut rig, 1.5);
    assert!(
        planar(at, rig.origin()) < 0.5,
        "walked dead in the session: {at:?} -> {:?}",
        rig.origin()
    );
    assert_eq!(statusbar_shown(&rig), Some(false), "the bar shows dying");

    rig.relog();
    assert!(
        rig.until(15.0, |r| dialog(r).is_some()),
        "no resurrect dialog on entering dead"
    );
    assert!(rig.client.frame().progress.dead);
    assert_eq!(health(&rig).map(|(now, _)| now), Some(0));
    assert!(
        rig.until(5.0, |r| statusbar_shown(r) == Some(false)),
        "the vitals show on entering dead: {}",
        meter(&rig)
    );
    let at = rig.origin();
    rig.seen.clear();
    forward(&mut rig, 1.5);
    assert!(
        planar(at, rig.origin()) < 0.5,
        "walked dead after the relog: {at:?} -> {:?}",
        rig.origin()
    );
    // Resurrect from the dialog.
    let button = dialog(&rig).unwrap();
    rig.click(button);
    rig.wait("the player never came back", |r| {
        notices(r).any(|n| matches!(n, GameplayNotice::UpdateDeathState(false)))
    });
    assert!(
        rig.until(10.0, |r| health(r).is_some_and(|(now, _)| now > 0)),
        "alive again but health {:?}",
        health(&rig)
    );
    assert!(rig.until(5.0, |r| dialog(r).is_none()), "the dialog stayed");
    assert!(
        rig.until(5.0, |r| statusbar_shown(r) == Some(true)),
        "the vitals stayed hidden"
    );
    assert!(
        rig.until(30.0, |r| {
            let f = r.client.frame();
            f.in_world && !f.portal.teleporting
        }),
        "the respawn's move never ended"
    );
    rig.run(3.0);
    let at = rig.origin();
    forward(&mut rig, 1.5);
    eprintln!("AFTER RESPAWN {at:?} -> {:?}", rig.origin());
    assert!(
        planar(at, rig.origin()) > 1.0,
        "cannot walk after the respawn: {at:?} -> {:?}",
        rig.origin()
    );
}

fn planar(a: [f32; 3], b: [f32; 3]) -> f32 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}
