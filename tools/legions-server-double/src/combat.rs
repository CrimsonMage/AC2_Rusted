//! The fight against the double (`WM_Combat`). A creature has health (`0x102`, its max `0x107`); the
//! player's attack (`WM_Combat` 0x16, or a skill from the shortcut bar: `WM_Combat` 0x17) damages
//! it, and it hits back on the attack's timer. A hit is a physics story (`DO_STORY`, its damage hook
//! with the attack result and the health change, sent by the attacker's object), the target's
//! health quality, and, for a creature, the selection panel's `UpdateSelectionInfo`. At zero health
//! the creature leaves the world (`LEAVE_WORLD`). A corpse is not spawned here (see the TODO in
//! `kill`). Starts of special attacks are not answered (no forwarder reaches them, see
//! `ac2_net::protocol::CombatMessage`).

use std::collections::BTreeMap;

use ac2_net::protocol::{
    CombatMessage, InstanceIdWithStamp, Opcode, QualityUpdate, QualityValue, ScriptCall,
    ScriptMessage, SelectionInfo, SpecialAttackRequest,
};

use crate::objects::object_event_bytes;
use crate::{State, CREATURE_ONE, CREATURE_TWO, PLAYER};

pub use crate::character::MELEE_AUTO_ATTACK;

/// `WM_Combat` 0x16 (start attacking: the target, a long int) and 0x14 (stop).
const ATTACK: u32 = 0x8204_0008;
const STOP_ATTACKING: u32 = 0x8204_0010;
/// `Health` and its max, the qualities the vitals read (`CPlayer` 0x85's table 4).
pub const HEALTH: u32 = 0x102;
pub const HEALTH_MAX: u32 = 0x107;
/// The creature's starting health.
pub const CREATURE_HEALTH: u32 = 30;
/// The player's starting health and max.
pub const PLAYER_HEALTH: u32 = 40;
pub const PLAYER_HEALTH_MAX: u32 = 50;
/// What a hit does: the player's blow to a creature, the creature's blow back to the player.
pub const PLAYER_DAMAGE: u32 = 10;
pub const CREATURE_DAMAGE: u32 = 3;
/// Seconds between the exchanges of an attack.
pub const HIT_INTERVAL: f64 = 2.0;
/// The attack result of a hit: the `AttackResult` enum the damage text reads (`0x40000007` reads
/// "MISS" in the DAT, `0x40000008` a hit).
pub const HIT: u32 = 0x4000_0008;
/// The `DoStory` hook bits: the damage type (1), the target (0x100), the attack result (0x80), the
/// target's and the attacker's health change (0x800, 0x1000). The story's bits: the attacker
/// (0x2) and the skill (0x10).
const STORY_BITS: u32 = 0x2 | 0x10;
const HOOK_BITS: u32 = 1 | 0x100 | 0x80 | 0x800 | 0x1000;
/// The instance of an object the double makes: the player's is 9 (`create_object`), the radar
/// creatures' 1.
fn instance_of(id: u64) -> u16 {
    if id == PLAYER {
        9
    } else {
        1
    }
}
/// A position stamp past the creatures' starting one (1): a leave-world the object takes.
const LEAVE_STAMP: u32 = 2;

/// The fight's state: each creature's (current, max) health while it lives, the player's, and
/// the player's attack (its target and the time of the next exchange).
#[derive(Debug, Clone, Default)]
pub struct Fight {
    pub creatures: BTreeMap<u64, (u32, u32)>,
    pub player: (u32, u32),
    pub attack: Option<(u64, f64)>,
    /// The creatures that died, in order.
    pub dead: Vec<u64>,
}

impl State {
    /// The fight's start: the two radar creatures at full health, the player at full vitals. The
    /// creatures' selection panels and the player's vitals are told.
    pub fn start_fight(&mut self) {
        self.fight.creatures.clear();
        for creature in [CREATURE_ONE, CREATURE_TWO] {
            self.fight
                .creatures
                .insert(creature, (CREATURE_HEALTH, CREATURE_HEALTH));
            self.send_creature_health(creature);
        }
        self.fight.player = (PLAYER_HEALTH, PLAYER_HEALTH_MAX);
        self.send_player_vitals();
    }

    /// `WM_Combat` for the calls the client makes: a special attack's request (answered at once
    /// with the attack state going up, a hit, and the attack's end), the attack's start (the
    /// attack is kept and exchanged on the timer, see [`State::combat_tick`]) and its stop.
    pub(crate) fn answer_combat(&mut self, call: &ScriptCall) {
        if let Some(request) = SpecialAttackRequest::from_call(call) {
            if self.is_live_creature(request.target) {
                self.send_combat(&CombatMessage::UpdateAttackState(true));
                self.hit(PLAYER, request.target, request.maneuver, HIT, PLAYER_DAMAGE);
                self.send_combat(&CombatMessage::EndSpecialAttack(request.attack));
            }
            return;
        }
        match (call.function, &call.arguments[..]) {
            (ATTACK, [ac2_net::protocol::PackValue::LongInt(target)])
                if self.is_live_creature(*target) =>
            {
                self.fight.attack = Some((*target, self.elapsed + HIT_INTERVAL));
                self.send_combat(&CombatMessage::UpdateAttackState(true));
            }
            (STOP_ATTACKING, []) => self.stop_attack(),
            _ => {}
        }
    }

    /// The fight's clock: when the attack's interval has run, the player blows and the creature
    /// hits back, and the attack goes on (or ends with the creature's death).
    pub fn combat_tick(&mut self) {
        let Some((target, due)) = self.fight.attack else {
            return;
        };
        if self.elapsed < due {
            return;
        }
        if !self.is_live_creature(target) {
            self.stop_attack();
            return;
        }
        self.hit(PLAYER, target, MELEE_AUTO_ATTACK, HIT, PLAYER_DAMAGE);
        if self.is_live_creature(target) {
            self.hit(target, PLAYER, MELEE_AUTO_ATTACK, HIT, CREATURE_DAMAGE);
        }
        if self.is_live_creature(target) {
            self.fight.attack = Some((target, self.elapsed + HIT_INTERVAL));
        } else {
            self.stop_attack();
        }
    }

    /// Queues a combat answer (a test can send one directly).
    pub fn send_combat(&mut self, message: &CombatMessage) {
        self.send_script_call(message.function(), &message.arguments());
    }

    fn stop_attack(&mut self) {
        if self.fight.attack.take().is_some() {
            self.send_combat(&CombatMessage::UpdateAttackState(false));
        }
    }

    fn is_live_creature(&self, id: u64) -> bool {
        self.fight.creatures.contains_key(&id)
    }

    /// One blow from `attacker` on `target`: the physics story, then the target's health.
    fn hit(&mut self, attacker: u64, target: u64, skill: u32, result: u32, damage: u32) {
        self.send_story(attacker, target, skill, result, damage);
        if target == PLAYER {
            let (health, max) = self.fight.player;
            self.fight.player = (health.saturating_sub(damage), max);
            self.send_player_vitals();
        } else if let Some((health, _)) = self.fight.creatures.get_mut(&target) {
            *health = health.saturating_sub(damage);
            if *health == 0 {
                self.kill(target);
            } else {
                self.send_creature_health(target);
            }
        }
    }

    /// `ECM_Physics::RecvEvt_DoStory`: the attacker's story with one damage hook on `target`.
    fn send_story(&mut self, attacker: u64, target: u64, skill: u32, result: u32, damage: u32) {
        let mut args = vec![STORY_BITS];
        args.extend([attacker as u32, (attacker >> 32) as u32]);
        args.push(skill);
        args.push(1);
        args.push(HOOK_BITS);
        args.extend([target as u32, (target >> 32) as u32]);
        args.extend([result, damage, 0]);
        let bytes = object_event_bytes(Opcode::DO_STORY, attacker, instance_of(attacker), &args);
        self.send_event(&bytes);
    }

    /// The creature's health quality and its selection panel.
    fn send_creature_health(&mut self, creature: u64) {
        let Some(&(health, max)) = self.fight.creatures.get(&creature) else {
            return;
        };
        let object = Some(InstanceIdWithStamp {
            id: creature,
            instance: instance_of(creature),
            other: 0,
        });
        self.send_quality(&QualityUpdate {
            object,
            quality: HEALTH,
            value: QualityValue::Int(health as i32),
        });
        let info = SelectionInfo {
            kind: SelectionInfo::CREATURE,
            cur_health: health,
            pk_damage: 0,
            max_health: max,
            cur_vigor: 0,
            pk_vigor_loss: 0,
            max_vigor: 0,
        };
        self.send_script(&ScriptMessage::UpdateSelectionInfo { info, id: creature });
    }

    /// The player's health and max (private qualities: `CPlayer` 0x85 hears them as vitals).
    fn send_player_vitals(&mut self) {
        let (health, max) = self.fight.player;
        self.send_private(HEALTH, QualityValue::Int(health as i32));
        self.send_private(HEALTH_MAX, QualityValue::Int(max as i32));
    }

    /// A creature at zero health leaves the world (`LEAVE_WORLD`). TODO(looting): the corpse
    /// object goes here once the looting worker's corpse code lands; retail's death motion is not
    /// traced yet, so the creature leaves at once.
    fn kill(&mut self, creature: u64) {
        self.fight.creatures.remove(&creature);
        self.fight.dead.push(creature);
        let bytes = object_event_bytes(
            Opcode::LEAVE_WORLD,
            creature,
            instance_of(creature),
            &[LEAVE_STAMP],
        );
        self.send_event(&bytes);
    }
}
