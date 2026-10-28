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
    CombatMessage, DeathMessage, InstanceIdWithStamp, Opcode, QualityUpdate, QualityValue,
    ScriptCall, ScriptMessage, SelectionInfo, SpecialAttackRequest,
};

use crate::corpse::CORPSE;
use crate::objects::object_event_bytes;
use crate::radar::radar_objects;
use crate::{Spawn, State, CREATURE_ONE, CREATURE_TWO, PLAYER};

pub use crate::character::MELEE_AUTO_ATTACK;

/// `WM_Combat` 0x16 (start attacking: the target, a long int) and 0x14 (stop).
const ATTACK: u32 = 0x8204_0008;
const STOP_ATTACKING: u32 = 0x8204_0010;
/// `WM_Combat` 0x25 (the script's `gmCWInterface::StopAttacking(target)`, a long int): the stop
/// for one target. The double treats it as the stop when the attack is on that target.
const STOP_ATTACKING_TARGET: u32 = 0x8204_0025;
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
/// The lifestone the player respawns at, from where the player enters (its offset).
const LIFESTONE_OFFSET: [f32; 3] = [-8.0, 8.0, 0.0];
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
/// The `dead` mode (`gmmodes.emp`) and the `death` behaviour (`gmbehaviors.emp`), and the mode a
/// body returns to on a respawn (`MODE_DEFAULT`). The monster and human behaviour tables both map
/// `death` to the `death` animation key (0x8000002E) and the `dead` mode's idle to the `dead` key
/// (0x8000002C).
pub const MODE_DEAD: u32 = 0x4100_0005;
pub const BEHAVIOR_DEATH: u32 = 0x4100_001F;
pub const MODE_PEACE: u32 = 0x4000_0001;
/// The fighting modes (`gmmodes.emp`): `combat` (the unarmed player's) and `combat_monster` (a
/// creature's). `Melee_Auto1` has combos only in the combat modes, never in `peace`.
pub const MODE_COMBAT: u32 = 0x4100_0009;
pub const MODE_COMBAT_MONSTER: u32 = 0x4100_0012;
/// `BehaviorParams` bits: the behaviour (0x1) and the mode it enters first (0x2).
const BEHAVIOR_AND_MODE: u32 = 0x3;
/// A position stamp past the creatures' starting one (1): a leave-world the object takes.
const LEAVE_STAMP: u32 = 2;
/// The swings a blow's attacker plays (`gmbehaviors.emp`): the player's `Melee_Auto1` (the
/// human table's combat combo) and a creature's `Melee_Normal1` (the monster table's: a drudge's
/// animation map has no `Melee_Auto` animation, so its tracker refuses that one).
pub const BEHAVIOR_MELEE_AUTO: u32 = 0x4100_0089;
pub const BEHAVIOR_MELEE_NORMAL: u32 = 0x4100_0077;
/// The attacker's swing story, as servers send one (a retained capture): the attacker, its
/// behaviour and the basic-attack bit (`0x2 | 0x4 | 0x20`), the behaviour with propagate and
/// send-event (`0x1 | 0x400 | 0x200`), no hooks.
const SWING_STORY_BITS: u32 = 0x26;
const SWING_BEHAVIOR_BITS: u32 = 0x601;
/// Seconds a dead creature lies in its death before it leaves the world for its corpse.
pub const DEATH_SECONDS: f64 = 4.0;

/// The fight's state: each creature's (current, max) health while it lives, the player's, and
/// the player's attack (its target and the time of the next exchange).
#[derive(Debug, Clone, Default)]
pub struct Fight {
    pub creatures: BTreeMap<u64, (u32, u32)>,
    pub player: (u32, u32),
    pub attack: Option<(u64, f64)>,
    /// The creatures that died, in order.
    pub dead: Vec<u64>,
    /// Whether the player is dead (its health is zero until the respawn request).
    pub player_dead: bool,
    /// The player's respawns so far (each teleport's stamps are newer than the last).
    pub respawns: u16,
    /// Dead creatures still lying in their death, with the time they leave the world.
    pub dying: Vec<(u64, f64)>,
}

/// The lifestone's position: the player's entry plus [`LIFESTONE_OFFSET`].
pub fn lifestone(spawn: &Spawn) -> [f32; 3] {
    [
        spawn.origin[0] + LIFESTONE_OFFSET[0],
        spawn.origin[1] + LIFESTONE_OFFSET[1],
        spawn.origin[2] + LIFESTONE_OFFSET[2],
    ]
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
        if DeathMessage::is_resurrect_request(call) {
            if self.fight.player_dead {
                self.respawn();
            }
            return;
        }
        // A dead player makes no attack.
        if self.fight.player_dead {
            return;
        }
        if let Some(request) = SpecialAttackRequest::from_call(call) {
            // A self-buff needs no target: its cost comes off, its effect goes on the player,
            // and the bar's special attack ends (the double's choice, as the hit's below).
            if self.effects.skill_effects.contains_key(&request.maneuver) {
                if self.charge_vigor(request.maneuver) {
                    self.apply_skill_effect(request.maneuver);
                    self.send_combat(&CombatMessage::EndSpecialAttack(request.attack));
                }
                return;
            }
            if self.is_live_creature(request.target) && self.charge_vigor(request.maneuver) {
                self.send_combat(&CombatMessage::UpdateAttackState(true));
                self.enter_combat(request.target);
                self.special_hit(request.target, request.maneuver, request.attack);
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
                self.enter_combat(*target);
            }
            (STOP_ATTACKING, []) => self.stop_attack(),
            (STOP_ATTACKING_TARGET, [ac2_net::protocol::PackValue::LongInt(target)])
                if self
                    .fight
                    .attack
                    .is_some_and(|(attacked, _)| attacked == *target) =>
            {
                self.stop_attack()
            }
            _ => {}
        }
    }

    /// The fight's clock: when the attack's interval has run, the player blows and the creature
    /// hits back, and the attack goes on (or ends with the creature's death).
    pub fn combat_tick(&mut self) {
        self.regenerate();
        self.effects_tick();
        self.dying_tick();
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

    /// A creature strikes the player unprovoked: one blow of its melee attack (the player's
    /// attack is not started).
    pub fn creature_strikes(&mut self, creature: u64) {
        if self.is_live_creature(creature) && !self.fight.player_dead {
            self.hit(creature, PLAYER, MELEE_AUTO_ATTACK, HIT, CREATURE_DAMAGE);
        }
    }

    /// Queues a combat answer (a test can send one directly).
    pub fn send_combat(&mut self, message: &CombatMessage) {
        self.send_script_call(message.function(), &message.arguments());
    }

    fn stop_attack(&mut self) {
        if let Some((target, _)) = self.fight.attack.take() {
            self.send_combat(&CombatMessage::UpdateAttackState(false));
            self.send_combat_mode(PLAYER, MODE_PEACE);
            if self.is_live_creature(target) {
                self.send_combat_mode(target, MODE_PEACE);
            }
        }
    }

    /// The fight's stances: the player and its target go into their combat modes (`DoMode`), the
    /// ones their swings play in.
    fn enter_combat(&mut self, target: u64) {
        self.send_combat_mode(PLAYER, MODE_COMBAT);
        self.send_combat_mode(target, MODE_COMBAT_MONSTER);
    }

    /// `ECM_Physics::RecvEvt_DoMode` on `object`.
    fn send_combat_mode(&mut self, object: u64, mode: u32) {
        let bytes = object_event_bytes(Opcode::DO_MODE, object, instance_of(object), &[mode]);
        self.send_event(&bytes);
    }

    fn is_live_creature(&self, id: u64) -> bool {
        self.fight.creatures.contains_key(&id)
    }

    /// One blow from `attacker` on `target`: the physics story, then the target's health.
    fn hit(&mut self, attacker: u64, target: u64, skill: u32, result: u32, damage: u32) {
        self.blow(attacker, target, skill, None, result, damage);
    }

    /// [`State::hit`] for a special attack: its damage story carries the client's attack id as
    /// the client attack context, which the client's `StoryManager::CleanupStory` hands back to
    /// the script (`gmCWInterface::SpecialAttackEnded`).
    fn special_hit(&mut self, target: u64, skill: u32, attack: u32) {
        self.blow(PLAYER, target, skill, Some(attack), HIT, PLAYER_DAMAGE);
    }

    fn blow(
        &mut self,
        attacker: u64,
        target: u64,
        skill: u32,
        context: Option<u32>,
        result: u32,
        damage: u32,
    ) {
        self.send_swing(attacker);
        self.send_story(attacker, target, skill, context, result, damage);
        if target == PLAYER {
            let (health, max) = self.fight.player;
            self.fight.player = (health.saturating_sub(damage), max);
            self.send_player_vitals();
            if self.fight.player.0 == 0 {
                self.die_player();
            }
        } else if let Some((health, _)) = self.fight.creatures.get_mut(&target) {
            *health = health.saturating_sub(damage);
            if *health == 0 {
                self.kill(target);
            } else {
                self.send_creature_health(target);
            }
        }
    }

    /// The attacker's swing: a story with its melee behaviour and no hooks, which the
    /// client's `StoryManager::DoStory` plays on the attacker.
    fn send_swing(&mut self, attacker: u64) {
        let args = [
            SWING_STORY_BITS,
            attacker as u32,
            (attacker >> 32) as u32,
            SWING_BEHAVIOR_BITS,
            if attacker == PLAYER {
                BEHAVIOR_MELEE_AUTO
            } else {
                BEHAVIOR_MELEE_NORMAL
            },
            1,
            0,
            0,
        ];
        let bytes = object_event_bytes(Opcode::DO_STORY, attacker, instance_of(attacker), &args);
        self.send_event(&bytes);
    }

    /// The creatures whose death has played out leave the world, and the corpse takes the first
    /// one's place.
    fn dying_tick(&mut self) {
        let elapsed = self.elapsed;
        let (due, still): (Vec<_>, Vec<_>) =
            self.fight.dying.iter().partition(|(_, at)| *at <= elapsed);
        self.fight.dying = still;
        for (creature, _) in due {
            self.leave_world(creature);
        }
    }

    /// `ECM_Physics::RecvEvt_DoStory`: the attacker's story with one damage hook on `target`, and
    /// the client attack context (0x8) when there is one.
    fn send_story(
        &mut self,
        attacker: u64,
        target: u64,
        skill: u32,
        context: Option<u32>,
        result: u32,
        damage: u32,
    ) {
        let mut args = vec![STORY_BITS | if context.is_some() { 0x8 } else { 0 }];
        args.extend([attacker as u32, (attacker >> 32) as u32]);
        args.extend(context);
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

    /// A creature at zero health: its Dead bool and its death behaviour (into the dead mode), then,
    /// [`DEATH_SECONDS`] later, it leaves the world (`LEAVE_WORLD`) and its corpse lies where it
    /// died (`corpse.rs`: the skin and the item in it). The corpse's id is one id: a creature
    /// killed after the first corpse leaves none.
    fn kill(&mut self, creature: u64) {
        self.fight.creatures.remove(&creature);
        self.fight.dead.push(creature);
        self.send_quality(&QualityUpdate {
            object: Some(InstanceIdWithStamp {
                id: creature,
                instance: instance_of(creature),
                other: 0,
            }),
            quality: 0x117,
            value: QualityValue::Bool(1),
        });
        self.send_death_behavior(creature);
        self.fight
            .dying
            .push((creature, self.elapsed + DEATH_SECONDS));
    }

    /// A dead creature's leave-world, and its corpse in its place (the first death's).
    fn leave_world(&mut self, creature: u64) {
        let bytes = object_event_bytes(
            Opcode::LEAVE_WORLD,
            creature,
            instance_of(creature),
            &[LEAVE_STAMP],
        );
        self.send_event(&bytes);
        let place = radar_objects(&self.spawn)
            .into_iter()
            .find(|(object, _)| object.id == creature)
            .map(|(object, _)| (object.cell, object.origin));
        let corpse_placed = self.script_world.contents.contains_key(&CORPSE);
        if let (Some((cell, origin)), false) = (place, corpse_placed) {
            self.spawn_corpse_at(cell, origin);
        }
    }

    /// The player at zero health: the Dead bool (quality 0x117) goes up, the death flag goes up
    /// (`WM_Death` 0x33), and the attack ends.
    fn die_player(&mut self) {
        self.fight.player_dead = true;
        self.stop_attack();
        self.send_private(0x117, QualityValue::Bool(1));
        self.send_death_behavior(PLAYER);
        self.send_death(true);
        self.apply_vitae_penalty();
    }

    /// The respawn request (`WM_Death` 0x36): the player is teleported to the lifestone with full
    /// health, and the death flag comes down.
    fn respawn(&mut self) {
        let n = self.fight.respawns;
        self.fight.respawns += 1;
        self.fight.player_dead = false;
        self.fight.player = (PLAYER_HEALTH, PLAYER_HEALTH_MAX);
        self.vitals.vigor.0 = self.shown_max(self.vitals.vigor.1);
        let origin = lifestone(&self.spawn);
        let cell = self.spawn.cell;
        // The teleport's stamps: the spawn's position 4 and teleport 5, each teleport newer.
        self.teleport_player(cell, origin, 5 + 2 * n, 6 + 2 * n);
        self.send_player_vitals();
        self.send_player_vigor();
        self.send_private(0x117, QualityValue::Bool(0));
        self.send_event(&object_event_bytes(
            Opcode::SET_MODE,
            PLAYER,
            instance_of(PLAYER),
            &[MODE_PEACE],
        ));
        self.send_death(false);
    }

    /// The death presentation of `object`: `RecvEvt_DoBehavior` with the `death` behaviour and the
    /// `dead` mode it enters first. The client plays the table's `death` animation, then rests in
    /// the `dead` mode. The server's own choice of event is not traced (the client reacts to
    /// physics events only: nothing in the executable or the wlib reads the Dead quality of a
    /// creature); this is the one the behaviour tables are built for.
    fn send_death_behavior(&mut self, object: u64) {
        self.send_event(&object_event_bytes(
            Opcode::DO_BEHAVIOR,
            object,
            instance_of(object),
            &[BEHAVIOR_AND_MODE, BEHAVIOR_DEATH, MODE_DEAD],
        ));
    }

    /// `WM_Death` 0x33: the death flag.
    fn send_death(&mut self, dead: bool) {
        let message = DeathMessage::PlayerDeath(dead);
        self.send_script_call(message.function(), &message.arguments());
    }
}
