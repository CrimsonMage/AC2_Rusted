//! `--ac1-weapons DIR`: Asheron's Call 1 weapons in the AC2 player's right hand. Every ACE weenie
//! patch file in DIR (melee weapons) is built from the AC1 DATs like the creatures, and the chosen
//! one hangs from the player's holding location 1, as the server's `RecvEvt_Parent` hangs a wielded
//! item (`CPhysicsObj::set_parent`, placed by `CPhysicsObj::UpdateChild`), with the AC1 Setup's
//! placement frame (`SetPlacementFrame`; `RightHandCombat` 1 by default, `--ac1-weapon-placement`).
//! PageUp and PageDown step through the weapons, back to the session's own AC2 sword.

use std::collections::HashMap;
use std::sync::Arc;

use ac1_assets::Ac1Creature;
use ac2_client::{Holder, ObjectFrame, PlayerFrame, WorldData};
use ac2_physics::{find_holding_point, HoldPoint};
use ac2_weenie::Appearance;

/// The weapon's object id.
const WEAPON_ID: u64 = 0x5000;
/// `RIGHT_HAND_HOLD`: the holding location a wielded weapon hangs from (AC1 and AC2 alike).
pub const RIGHT_HAND: u32 = 1;
/// AC1 `Placement.RightHandCombat`.
const RIGHT_HAND_COMBAT: u32 = 1;

/// The weapons and which one is in hand.
pub struct Ac1Weapons {
    pub weapons: Vec<Ac1Creature>,
    /// The weapon in hand; `None` leaves the session's AC2 sword there.
    pub current: Option<usize>,
    /// `--ac1-weapon-placement`: the AC1 placement frame the weapon takes in the hand.
    pub placement: u32,
    world: Option<ac2_client::DatWorld>,
    /// The right-hand holding point of each player visual seen.
    points: HashMap<u32, Option<HoldPoint>>,
}

impl Ac1Weapons {
    pub fn new(weapons: Vec<Ac1Creature>, store: Arc<ac2_db::ResourceStore>) -> Self {
        let world = match ac2_client::DatWorld::new(store) {
            Ok(world) => Some(world),
            Err(error) => {
                eprintln!("world-viewer: no holding locations: {error}");
                None
            }
        };
        Self {
            current: (!weapons.is_empty()).then_some(0),
            weapons,
            placement: RIGHT_HAND_COMBAT,
            world,
            points: HashMap::new(),
        }
    }

    /// Steps the weapon in hand by `step`, through "none" (the session's sword) at the end.
    pub fn cycle(&mut self, step: i32) {
        let slots = self.weapons.len() as i32 + 1;
        let at = self.current.map_or(slots - 1, |i| i as i32);
        let next = (at + step).rem_euclid(slots);
        self.current = (next < slots - 1).then_some(next as usize);
        println!(
            "ac1 weapon: {}",
            self.current
                .map_or("none (the AC2 sword)", |i| self.weapons[i]
                    .weenie
                    .name
                    .as_str())
        );
    }

    /// `player`'s right-hand holding point (`CPartArray::GetHoldingLocation`), from its body.
    fn point(&mut self, visual: u32) -> Option<HoldPoint> {
        let world = self.world.as_ref()?;
        *self.points.entry(visual).or_insert_with(|| {
            let body = world.body(Some(visual)).ok()?;
            find_holding_point(&body.holding, RIGHT_HAND)
        })
    }

    /// Puts the chosen weapon in `frame`'s player's right hand, in place of what the session
    /// hangs there.
    pub fn hold(&mut self, frame: &mut ac2_client::ClientFrame) {
        let Some(weapon) = self.held(frame.player.as_ref()) else {
            return;
        };
        let player = weapon.holder.map(|h| h.id);
        frame.objects.retain(|o| {
            o.holder
                .is_none_or(|h| Some(h.id) != player || h.location != RIGHT_HAND)
        });
        frame.objects.push(weapon);
    }

    /// The weapon in `player`'s right hand this frame, if one is chosen and the hand is found.
    pub fn held(&mut self, player: Option<&PlayerFrame>) -> Option<ObjectFrame> {
        let player = player?;
        let current = self.current?;
        let point = self.point(player.visual?)?;
        let weapon = &self.weapons[current];
        let visual = weapon.visual.raw();
        Some(ObjectFrame {
            id: WEAPON_ID,
            visual: Some(visual),
            setup: None,
            name: Some(weapon.weenie.name.as_str().into()),
            stack_name: None,
            cell: player.cell,
            origin: player.origin,
            rotation: player.rotation,
            heading_degrees: None,
            in_world: false,
            lost_cell: None,
            active: true,
            velocity: [0.0; 3],
            contact: false,
            mode: None,
            animated_mode: None,
            motion: [0.0; 3],
            holder: Some(Holder {
                id: player.id,
                location: RIGHT_HAND,
                orientation: Some(self.placement),
                point: Some(point),
                // AC1's hand axes into AC2's (`ac1_assets::creature::AC1_HAND_TO_AC2`).
                offset: ac2_math::Frame::new(
                    ac2_math::Vec3::ZERO,
                    ac1_assets::creature::AC1_HAND_TO_AC2,
                ),
            }),
            fx_follow: None,
            container: None,
            wielder: Some(player.id),
            appearance: Arc::new(Appearance::default()),
            pose: Vec::new(),
            anim_scale: [1.0; 4],
            body_scale: [weapon.weenie.scale; 3],
            adaptive_degrade: false,
        })
    }
}
