use super::*;

const KNIFE: Knife = Knife {
    id: 0x8000_0001,
    valid_targets: 8,
};

fn corpse(id: u64, distance: f32) -> Corpse {
    Corpse {
        id,
        target_type: 8,
        distance,
        level: Some(10),
        killed_by: KilledBy::Player,
    }
}

fn view<'a>(now: f64, knives: &'a [Knife], corpses: &'a [Corpse]) -> ButcherView<'a> {
    ButcherView {
        now,
        ready: true,
        in_combat: false,
        player_level: Some(12),
        knives,
        corpses,
    }
}

fn butcher() -> AutoButcher {
    AutoButcher::with_settings(ButcherSettings {
        enabled: true,
        ..ButcherSettings::default()
    })
}

#[test]
fn the_knife_is_used_then_the_corpse_clicked_once_the_pointer_is_armed() {
    let mut tool = butcher();
    let corpses = [corpse(1, 4.0), corpse(2, 2.0)];
    // The nearest corpse first: the knife's use.
    assert_eq!(
        tool.tick(&view(0.0, &[KNIFE], &corpses), &[]),
        vec![ButcherAction::UseItem(KNIFE.id)]
    );
    // Nothing more until the pointer is armed, then the click on that corpse.
    assert!(tool.tick(&view(0.5, &[KNIFE], &corpses), &[]).is_empty());
    assert_eq!(
        tool.tick(
            &view(0.6, &[KNIFE], &corpses),
            &[ButcherEvent::TargetingPointer(true)]
        ),
        vec![ButcherAction::UseOn(2)]
    );
    // The items arrive; after the cooldown the other corpse, never the same one again.
    assert!(tool
        .tick(&view(1.0, &[KNIFE], &corpses), &[ButcherEvent::PackGained])
        .is_empty());
    assert!(tool.tick(&view(1.5, &[KNIFE], &corpses), &[]).is_empty());
    assert_eq!(
        tool.tick(&view(3.1, &[KNIFE], &corpses), &[]),
        vec![ButcherAction::UseItem(KNIFE.id)]
    );
    tool.tick(
        &view(3.2, &[KNIFE], &corpses),
        &[ButcherEvent::TargetingPointer(true)],
    );
    tool.tick(&view(3.5, &[KNIFE], &corpses), &[ButcherEvent::PackGained]);
    assert!(tool.tick(&view(9.0, &[KNIFE], &corpses), &[]).is_empty());
}

#[test]
fn eligibility_follows_the_target_bits_reach_and_level() {
    let tool = butcher();
    let level = Some(12);
    assert!(tool.eligible(&corpse(1, 6.0), &KNIFE, level));
    // `Usage_TargetType` 4 (a living creature's default) shares no bit with 8.
    assert!(!tool.eligible(
        &Corpse {
            target_type: 4,
            ..corpse(1, 1.0)
        },
        &KNIFE,
        level
    ));
    assert!(!tool.eligible(&corpse(1, 6.1), &KNIFE, level));
    // Eight levels below is refused; seven is not.
    assert!(!tool.eligible(
        &Corpse {
            level: Some(4),
            ..corpse(1, 1.0)
        },
        &KNIFE,
        level
    ));
    assert!(tool.eligible(
        &Corpse {
            level: Some(5),
            ..corpse(1, 1.0)
        },
        &KNIFE,
        level
    ));
    // Unknown levels do not hold it.
    assert!(tool.eligible(
        &Corpse {
            level: None,
            ..corpse(1, 1.0)
        },
        &KNIFE,
        level
    ));
}

#[test]
fn combat_cooldown_no_knife_and_off_hold_it() {
    let corpses = [corpse(1, 1.0)];
    let mut tool = butcher();
    let mut fighting = view(0.0, &[KNIFE], &corpses);
    fighting.in_combat = true;
    assert!(tool.tick(&fighting, &[]).is_empty());
    assert!(tool.tick(&view(0.0, &[], &corpses), &[]).is_empty());
    tool.settings.enabled = false;
    assert!(tool.tick(&view(0.0, &[KNIFE], &corpses), &[]).is_empty());
}

#[test]
fn a_refusal_or_silence_switches_it_off() {
    let corpses = [corpse(1, 1.0), corpse(2, 1.0)];
    let mut tool = butcher();
    tool.tick(&view(0.0, &[KNIFE], &corpses), &[]);
    tool.tick(
        &view(0.1, &[KNIFE], &corpses),
        &[ButcherEvent::TargetingPointer(true)],
    );
    let out = tool.tick(
        &view(0.2, &[KNIFE], &corpses),
        &[ButcherEvent::UsageRefused],
    );
    assert!(matches!(&out[..], [ButcherAction::Stopped(_)]));
    assert!(!tool.settings.enabled && !tool.busy());
    // No pointer within ARM_SECONDS.
    let mut tool = butcher();
    tool.tick(&view(0.0, &[KNIFE], &corpses), &[]);
    let out = tool.tick(&view(butcher::ARM_SECONDS + 0.1, &[KNIFE], &corpses), &[]);
    assert!(matches!(&out[..], [ButcherAction::Stopped(_)]));
    // Nothing reaching the pack.
    let mut tool = butcher();
    tool.tick(&view(0.0, &[KNIFE], &corpses), &[]);
    tool.tick(
        &view(0.1, &[KNIFE], &corpses),
        &[ButcherEvent::TargetingPointer(true)],
    );
    let out = tool.tick(
        &view(0.2 + butcher::RESULT_SECONDS, &[KNIFE], &corpses),
        &[],
    );
    assert!(matches!(&out[..], [ButcherAction::Stopped(_)]));
}

const STANCE: u32 = 0xA8;

fn rebuff() -> AutoRebuff {
    AutoRebuff::with_settings(RebuffSettings {
        enabled: true,
        entries: vec![RebuffEntry {
            effect: "Defensive Stance".into(),
            skill: STANCE,
            enabled: true,
        }],
        ..RebuffSettings::default()
    })
}

fn rview<'a>(now: f64, effects: &'a [ActiveEffect], skills: &'a [SkillState]) -> RebuffView<'a> {
    RebuffView {
        now,
        ready: true,
        in_combat: false,
        vigor: 1.0,
        busy: false,
        effects,
        skills,
    }
}

const ON_BAR: [SkillState; 1] = [SkillState {
    skill: STANCE,
    slot: Some(3),
    recharge: 0.0,
}];

fn stance(ends: f64) -> ActiveEffect {
    ActiveEffect {
        name: "Defensive Stance".into(),
        ends: Some(ends),
    }
}

#[test]
fn a_missing_or_expiring_effect_is_recast_from_its_slot() {
    let mut tool = rebuff();
    let cast = Some(UseShortcut {
        slot: 3,
        skill: STANCE,
    });
    assert_eq!(tool.tick(&rview(0.0, &[], &ON_BAR)), cast);
    // The cast renewed it: nothing until the lead time.
    let effects = [stance(120.0)];
    assert_eq!(tool.tick(&rview(1.0, &effects, &ON_BAR)), None);
    assert_eq!(tool.tick(&rview(89.0, &effects, &ON_BAR)), None);
    assert_eq!(tool.tick(&rview(90.0, &effects, &ON_BAR)), cast);
    // An effect without a duration is never recast.
    let permanent = [ActiveEffect {
        name: "Defensive Stance".into(),
        ends: None,
    }];
    let mut tool = rebuff();
    assert_eq!(tool.tick(&rview(0.0, &permanent, &ON_BAR)), None);
}

#[test]
fn the_conditions_hold_the_cast() {
    let off_bar = [SkillState {
        slot: None,
        ..ON_BAR[0]
    }];
    let recharging = [SkillState {
        recharge: 2.0,
        ..ON_BAR[0]
    }];
    for (skills, mutate) in [
        (&off_bar[..], None),
        (&recharging[..], None),
        (&ON_BAR[..], Some(0)),
        (&ON_BAR[..], Some(1)),
        (&ON_BAR[..], Some(2)),
    ] {
        let mut tool = rebuff();
        let mut v = rview(0.0, &[], skills);
        match mutate {
            Some(0) => v.in_combat = true,
            Some(1) => v.busy = true,
            Some(2) => v.vigor = 0.1,
            _ => {}
        }
        assert_eq!(tool.tick(&v), None, "{mutate:?}");
    }
    let mut tool = rebuff();
    tool.settings.entries[0].enabled = false;
    assert_eq!(tool.tick(&rview(0.0, &[], &ON_BAR)), None);
}

#[test]
fn a_cast_that_does_not_take_backs_off_and_casts_are_spaced() {
    let mut tool = rebuff();
    assert!(tool.tick(&rview(0.0, &[], &ON_BAR)).is_some());
    // Still missing: no second press while the first is unconfirmed, then a minute's rest.
    assert_eq!(tool.tick(&rview(3.0, &[], &ON_BAR)), None);
    assert_eq!(tool.tick(&rview(6.0, &[], &ON_BAR)), None);
    assert_eq!(tool.tick(&rview(60.0, &[], &ON_BAR)), None);
    assert!(tool.tick(&rview(66.5, &[], &ON_BAR)).is_some());
    // Two entries are cast at least MIN_GAP_SECONDS apart.
    let mut tool = rebuff();
    tool.settings.entries.push(RebuffEntry {
        effect: "Haste".into(),
        skill: 0xB0,
        enabled: true,
    });
    let skills = [
        ON_BAR[0],
        SkillState {
            skill: 0xB0,
            slot: Some(4),
            recharge: 0.0,
        },
    ];
    assert!(tool.tick(&rview(0.0, &[], &skills)).is_some());
    assert_eq!(tool.tick(&rview(1.0, &[], &skills)), None);
    assert_eq!(
        tool.tick(&rview(2.0, &[], &skills)).map(|u| u.skill),
        Some(0xB0)
    );
}

#[test]
fn settings_round_trip_and_the_window_changes_apply() {
    let mut settings = Settings::default();
    assert!(settings.apply(&Change::Toggle(Flag::Butcher)));
    assert!(settings.apply(&Change::Step(Value::MaxDistance, false)));
    assert!(settings.apply(&Change::Step(Value::Lead, true)));
    assert!(settings.apply(&Change::Maintain {
        effect: "Defensive Stance".into(),
        skill: STANCE,
    }));
    assert!(!settings.apply(&Change::Maintain {
        effect: "Defensive Stance".into(),
        skill: 1,
    }));
    assert!(settings.apply(&Change::SetSkill {
        index: 0,
        skill: 0xB0
    }));
    assert!(settings.apply(&Change::ToggleEntry(0)));
    assert_eq!(settings.butcher.max_distance, 5.0);
    assert_eq!(settings.rebuff.lead, 35.0);
    let decoded = Settings::decode(&settings.encode());
    assert_eq!(decoded, settings);
    assert!(settings.apply(&Change::Forget(0)));
    assert!(settings.rebuff.entries.is_empty());
    // Out of range and unknown lines are clamped or skipped.
    let odd = Settings::decode(
        b"butcher.max_distance=900\nbogus=1\nrebuff.entry=1|zz|x\nrebuff.min_vigor=2\n",
    );
    assert_eq!(odd.butcher.max_distance, settings::MAX_DISTANCE);
    assert!(odd.rebuff.entries.is_empty());
    assert_eq!(odd.rebuff.min_vigor, 1.0);
    assert_eq!(
        settings::file_name(0x2130_0000_0000_0dfd),
        "2130000000000dfd.txt"
    );
}

#[test]
fn only_the_players_kills_are_butchered_and_an_unknown_corpse_is_examined_first() {
    let mut tool = butcher();
    let unknown = Corpse {
        killed_by: KilledBy::Unknown,
        ..corpse(1, 2.0)
    };
    let theirs = Corpse {
        killed_by: KilledBy::Other,
        ..corpse(2, 1.0)
    };
    // The nearer corpse is someone else's: passed over; the other is asked about once.
    assert_eq!(
        tool.tick(&view(0.0, &[KNIFE], &[unknown, theirs]), &[]),
        vec![ButcherAction::Examine(1)]
    );
    assert!(tool
        .tick(&view(1.0, &[KNIFE], &[unknown, theirs]), &[])
        .is_empty());
    // Its profile names the player: the knife is used on it.
    let mine = Corpse {
        killed_by: KilledBy::Player,
        ..unknown
    };
    assert_eq!(
        tool.tick(&view(1.5, &[KNIFE], &[mine, theirs]), &[]),
        vec![ButcherAction::UseItem(KNIFE.id)]
    );
}

#[test]
fn a_corpse_whose_profile_names_no_killer_is_passed_over() {
    let mut tool = butcher();
    let unknown = [Corpse {
        killed_by: KilledBy::Unknown,
        ..corpse(1, 2.0)
    }];
    assert_eq!(
        tool.tick(&view(0.0, &[KNIFE], &unknown), &[]),
        vec![ButcherAction::Examine(1)]
    );
    assert!(tool
        .tick(&view(butcher::RESULT_SECONDS + 0.5, &[KNIFE], &unknown), &[])
        .is_empty());
    assert!(tool
        .tick(&view(butcher::RESULT_SECONDS + 10.0, &[KNIFE], &unknown), &[])
        .is_empty());
    // With the option off, any corpse is used without asking.
    let mut any = AutoButcher::with_settings(ButcherSettings {
        enabled: true,
        only_my_kills: false,
        ..ButcherSettings::default()
    });
    let theirs = [Corpse {
        killed_by: KilledBy::Other,
        ..corpse(2, 2.0)
    }];
    assert_eq!(
        any.tick(&view(0.0, &[KNIFE], &theirs), &[]),
        vec![ButcherAction::UseItem(KNIFE.id)]
    );
}
