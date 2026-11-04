//! The automatic tools over the client (`ac2_macro::automation`, a modern addition): each frame
//! the client's own state (the pack, the world objects' qualities and places, the effect list, the
//! shortcut bar, the session's notices) goes to the tools, and what they ask for is done the way
//! the player's gesture does it, at the start of the next frame: a use is
//! [`Command::Select`] then [`Command::UseSelection`] (the inventory's double-click and the
//! targeting click), a cast is [`Client::use_shortcut`] (the bar's press). The one request of the
//! tools' own is the auto-butcher's quiet examination of a corpse whose killer is not known yet
//! (`OnlineSession::examine_quietly`, the examination panel's object request, as the name plates'
//! hit chance asks it). Nothing else is sent.
//! The settings window and the per-character file are the HUD's (`hud_automation`).

use std::collections::BTreeMap;

use ac2_macro::automation::{
    ActiveEffect, AutoButcher, AutoRebuff, ButcherAction, ButcherEvent, ButcherView, Corpse,
    KilledBy, Knife, RebuffView, Settings, SkillState,
};
use ac2_net::protocol::ShortcutInfo;
use ac2_physics::landdefs::get_block_offset;
use ac2_weenie::{GameplayNotice, Killer, SessionEvent};

use crate::client::Client;
use crate::gameplay::Command;

/// `Usage_TargetType` and `Usage_ValidTargetTypes` (the player's script's usage test).
pub const USAGE_TARGET_TYPE: u32 = 721;
pub const USAGE_VALID_TARGET_TYPES: u32 = 722;
/// The level quality Pyrealium's butchery reads (`object.stat(0x12e)`).
pub const LEVEL: u32 = 0x12e;
/// The shortcut type of a skill (`ShortcutInfo` 0x40000001, `_data_enum`).
const SKILL_SHORTCUT: u32 = 0x4000_0001;
/// The tools look at the world this often (seconds).
const TICK_SECONDS: f64 = 0.25;
/// A special attack with no end is taken as finished after this long.
const ACTION_SECONDS: f64 = 10.0;

/// One gesture the tools asked for, done next frame.
#[derive(Debug, Clone)]
enum Gesture {
    Commands(Vec<Command>),
    Shortcut(u32),
}

/// The tools and what they saw last.
#[derive(Default)]
pub(crate) struct AutomationHost {
    butcher: AutoButcher,
    rebuff: AutoRebuff,
    pending: Vec<Gesture>,
    events: Vec<ButcherEvent>,
    attacking: bool,
    /// When the current special attack started (`RequestSpecialAttack` / `StartSpecialAttack`).
    action_since: Option<f64>,
    /// The pack's items and their quantities at the last look.
    pack: BTreeMap<u64, i32>,
    last_tick: f64,
    /// Lines for the chat (a tool that switched itself off).
    lines: Vec<String>,
}

impl AutomationHost {
    fn settings(&self) -> Settings {
        Settings {
            butcher: self.butcher.settings.clone(),
            rebuff: self.rebuff.settings.clone(),
        }
    }
}

impl Client {
    /// Both tools' settings as they stand (a tool that stopped itself is off here).
    pub fn automation_settings(&self) -> Settings {
        self.automation.settings()
    }

    /// The settings the player chose (the window, the saved file).
    pub fn set_automation_settings(&mut self, settings: Settings) {
        self.automation.butcher.settings = settings.butcher;
        self.automation.rebuff.settings = settings.rebuff;
    }

    /// The tools' chat lines since the last call.
    pub fn take_automation_lines(&mut self) -> Vec<String> {
        std::mem::take(&mut self.automation.lines)
    }

    /// The gestures asked for last frame, carried out as the player's own.
    pub(crate) fn automation_gestures(&mut self) -> Vec<SessionEvent> {
        let mut events = Vec::new();
        for gesture in std::mem::take(&mut self.automation.pending) {
            match gesture {
                Gesture::Commands(commands) => events.extend(self.commands(&commands)),
                Gesture::Shortcut(slot) => {
                    if let Err(error) = self.use_shortcut(slot) {
                        eprintln!("ac2: auto-rebuff: shortcut {slot}: {error}");
                    }
                }
            }
        }
        events
    }

    /// One frame of the tools: the frame's notices, then (a few times a second) a look at the
    /// world and at most one gesture from each.
    pub(crate) fn automation_frame(&mut self, events: &[SessionEvent]) {
        let now = self.session.cur_time();
        for event in events {
            let SessionEvent::Notice(notice) = event else {
                continue;
            };
            match notice {
                GameplayNotice::UpdateAttackState(on) => self.automation.attacking = *on,
                GameplayNotice::SetTargetingPointer(on) => {
                    self.automation
                        .events
                        .push(ButcherEvent::TargetingPointer(*on));
                }
                GameplayNotice::UsageError { .. } => {
                    self.automation.events.push(ButcherEvent::UsageRefused)
                }
                GameplayNotice::RequestSpecialAttack { .. }
                | GameplayNotice::StartSpecialAttack(_) => {
                    self.automation.action_since = Some(now);
                }
                GameplayNotice::EndSpecialAttack(_) => self.automation.action_since = None,
                _ => {}
            }
        }
        let butcher_on = self.automation.butcher.settings.enabled || self.automation.butcher.busy();
        let rebuff_on = self.automation.rebuff.settings.enabled;
        if !butcher_on && !rebuff_on {
            self.automation.events.clear();
            return;
        }
        if now - self.automation.last_tick < TICK_SECONDS && self.automation.events.is_empty() {
            return;
        }
        self.automation.last_tick = now;
        let player = self.player.as_ref().map(|p| p.id);
        let ready = player.is_some()
            && self.player.as_ref().is_some_and(|p| p.in_world)
            && !self.progress_frame().dead
            // A teleport under way (its destination not in yet).
            && !self.portal_frame().teleporting;
        if let Some(id) = player {
            if self.pack_gained(id) {
                self.automation.events.push(ButcherEvent::PackGained);
            }
        }
        let busy = self
            .automation
            .action_since
            .is_some_and(|since| now - since < ACTION_SECONDS);
        let in_combat = self.automation.attacking;
        if butcher_on {
            let knives = self.knives();
            let corpses = self.corpses();
            let player_level = player
                .and_then(|p| self.quality(p, LEVEL))
                .map(|l| l as u32);
            let view = ButcherView {
                now,
                ready,
                in_combat,
                player_level,
                knives: &knives,
                corpses: &corpses,
            };
            let events = std::mem::take(&mut self.automation.events);
            for action in self.automation.butcher.tick(&view, &events) {
                match action {
                    ButcherAction::UseItem(id) | ButcherAction::UseOn(id) => {
                        self.automation.pending.push(Gesture::Commands(vec![
                            Command::Select(Some(id)),
                            Command::UseSelection,
                        ]));
                    }
                    ButcherAction::Examine(id) => {
                        self.session.examine_quietly(id);
                    }
                    ButcherAction::Stopped(line) => self.automation.lines.push(line),
                }
            }
        }
        self.automation.events.clear();
        if rebuff_on && !self.automation.butcher.busy() {
            let effects = self.active_effects();
            let skills = self.bar_skills();
            let vigor = self
                .session
                .player_script()
                .vitals()
                .vigor
                .filter(|v| v.max > 0)
                // Before the server names it the vigor is not known; the server judges the cost.
                .map_or(1.0, |v| v.current as f32 / v.max as f32);
            let view = RebuffView {
                now,
                ready,
                in_combat,
                vigor,
                busy,
                effects: &effects,
                skills: &skills,
            };
            if let Some(cast) = self.automation.rebuff.tick(&view) {
                self.automation.pending.push(Gesture::Shortcut(cast.slot));
            }
        }
    }

    fn quality(&self, object: u64, key: u32) -> Option<i32> {
        self.session.objects().get(object)?.qualities.inq_int(key)
    }

    /// Whether the pack holds an item it did not, or more of one, since the last look.
    fn pack_gained(&mut self, player: u64) -> bool {
        let objects = self.session.objects();
        let now: BTreeMap<u64, i32> = self
            .session
            .inventory()
            .contents(player)
            .map(|c| {
                c.items
                    .iter()
                    .map(|id| {
                        let quantity = objects
                            .get(*id)
                            .map_or(1, |o| ac2_weenie::ItemStats::of(&o.qualities).quantity);
                        (*id, quantity)
                    })
                    .collect()
            })
            .unwrap_or_default();
        let gained = now
            .iter()
            .any(|(id, q)| self.automation.pack.get(id).is_none_or(|before| q > before));
        let first = self.automation.pack.is_empty();
        self.automation.pack = now;
        gained && !first
    }

    /// The pack's items that are used on a target (`Usage_ValidTargetTypes` set).
    fn knives(&self) -> Vec<Knife> {
        let Some(player) = self.player.as_ref().map(|p| p.id) else {
            return Vec::new();
        };
        let inventory = self.session.inventory();
        let Some(contents) = inventory.contents(player) else {
            return Vec::new();
        };
        contents
            .items
            .iter()
            .filter_map(|id| {
                let valid = self.quality(*id, USAGE_VALID_TARGET_TYPES)?;
                (valid != 0).then_some(Knife {
                    id: *id,
                    valid_targets: valid as u32,
                })
            })
            .collect()
    }

    /// The world objects that name a usage target type, with their distance from the player.
    fn corpses(&self) -> Vec<Corpse> {
        let Some(body) = self.player.as_ref().and_then(|p| p.body.as_ref()) else {
            return Vec::new();
        };
        let (cell, origin, _) =
            ac2_physics::led_drawn_place(&body.physics, self.render_interpolation);
        // The player's name as the character set lists it (`gmCWInterface::InqName`).
        let player_name = self
            .player
            .as_ref()
            .and_then(|p| self.session.character_name(p.id));
        self.object_frames()
            .into_iter()
            .filter(|o| o.in_world && o.container.is_none() && o.holder.is_none())
            .filter_map(|o| {
                let target_type = self.quality(o.id, USAGE_TARGET_TYPE)? as u32;
                let block = get_block_offset(cell, o.cell);
                let offset = [
                    f64::from(block.x) + f64::from(o.origin[0]) - f64::from(origin[0]),
                    f64::from(block.y) + f64::from(o.origin[1]) - f64::from(origin[1]),
                    f64::from(block.z) + f64::from(o.origin[2]) - f64::from(origin[2]),
                ];
                let distance = offset.iter().map(|v| v * v).sum::<f64>().sqrt() as f32;
                Some(Corpse {
                    id: o.id,
                    target_type,
                    distance,
                    level: self.quality(o.id, LEVEL).map(|l| l as u32),
                    killed_by: match self.session.corpse_killer(o.id) {
                        None => KilledBy::Unknown,
                        Some(Killer::Named(name))
                            if player_name.as_deref() == Some(name.as_str()) =>
                        {
                            KilledBy::Player
                        }
                        Some(_) => KilledBy::Other,
                    },
                })
            })
            .collect()
    }

    /// The status bar strip's effects (`InqEffectList`), each with its end.
    fn active_effects(&mut self) -> Vec<ActiveEffect> {
        self.effect_list()
            .unwrap_or_default()
            .into_iter()
            .map(|n| ActiveEffect {
                ends: (n.duration > 0.0 && n.time_cast >= 0.0).then_some(n.time_cast + n.duration),
                name: n.name,
            })
            .collect()
    }

    /// The skills on the shortcut bar, their slot and how long until each is ready
    /// (`InqSkillTimeLeft`).
    fn bar_skills(&mut self) -> Vec<SkillState> {
        let wanted: Vec<u32> = self
            .automation
            .rebuff
            .settings
            .entries
            .iter()
            .map(|e| e.skill)
            .collect();
        let shortcuts = self.progress_frame().shortcuts;
        let mut skills = Vec::new();
        for skill in wanted {
            let slot = shortcuts.slots.iter().position(|s| {
                matches!(s, ShortcutInfo::Enum { kind, value } if *kind == SKILL_SHORTCUT && *value == skill)
            });
            let recharge = self
                .skill_time_left(skill)
                .ok()
                .filter(|t| t.ok)
                .map_or(0.0, |t| t.left.max(0.0));
            skills.push(SkillState {
                skill,
                slot: slot.map(|s| s as u32),
                recharge,
            });
        }
        skills
    }
}
