//! `--ac1-weapons DIR`: Asheron's Call 1 weapons in the AC2 player's right hand. Every ACE weenie
//! patch file in DIR (melee weapons) is built from the AC1 DATs like the creatures, and the chosen
//! one hangs from the player's holding location 1, as the server's `RecvEvt_Parent` hangs a wielded
//! item (`CPhysicsObj::set_parent`, placed by `CPhysicsObj::UpdateChild`), with the AC1 Setup's
//! placement frame (`SetPlacementFrame`; `RightHandCombat` 1 by default, `--ac1-weapon-placement`).
//! With the HUD's session (the window, or `--hud-panel`), the weapons are also items in the
//! player's pack: equipping one in the inventory's primary hand makes the session's server wield
//! it, and the hand draws its AC1 model (`place_wielded`). PageUp and PageDown put a weapon in
//! the hand directly, past "none", where the hand shows what the session wields.

use std::collections::HashMap;
use std::sync::Arc;

use ac1_assets::Ac1Creature;
use ac2_client::{Holder, ObjectFrame, PlayerFrame, WorldData};
use ac2_physics::{find_holding_point, HoldPoint};
use ac2_weenie::Appearance;
use legions_server_double::{ItemSpawn, ICON_SWORD, PLAYER, PRIMARY_HAND};

/// The weapon's object id.
const WEAPON_ID: u64 = 0x5000;
/// The first AC1 weapon item's id in the session's inventory.
const FIRST_ITEM: u64 = 0x8000_0000_0000_0300;
/// `CWeapon`'s package.
const WEAPON_PACKAGE: u32 = 0x1AAB;
/// The image DIDs the weapons' AC1 icons are served under (`Ac1Icons`).
const FIRST_ICON: u32 = 0xAC40_0000;

/// The weapons' AC1 icons for the HUD (`ac2_ui::ForeignUiImages`), by image DID.
pub struct Ac1Icons(HashMap<u32, ac2_ui::UiPixels>);

impl Ac1Icons {
    /// Each weapon's icon, decoded from the AC1 DATs; a weapon without one keeps the AC2
    /// sword's.
    pub fn load(dats: &ac1_assets::Ac1Dats, weapons: &[Ac1Creature]) -> Self {
        let mut icons = HashMap::new();
        for (index, weapon) in weapons.iter().enumerate() {
            if weapon.icon == 0 {
                continue;
            }
            match dats.icon_rgba(weapon.icon) {
                Ok(Some((width, height, rgba))) => {
                    icons.insert(
                        Ac1Weapons::icon_id(index),
                        ac2_ui::UiPixels {
                            width,
                            height,
                            rgba: Arc::from(rgba),
                        },
                    );
                }
                Ok(None) => {}
                Err(error) => eprintln!("world-viewer: {}: {error}", weapon.weenie.name),
            }
        }
        Self(icons)
    }

    pub fn contains(&self, file: u32) -> bool {
        self.0.contains_key(&file)
    }
}

impl ac2_ui::ForeignUiImages for Ac1Icons {
    fn image(&self, file: u32) -> Option<ac2_ui::UiPixels> {
        self.0.get(&file).cloned()
    }
}
/// `RIGHT_HAND_HOLD`: the holding location a wielded weapon hangs from (AC1 and AC2 alike).
pub const RIGHT_HAND: u32 = 1;
/// The held weapons' default scale over their weenies': AC1 weapons are made for a 1.82 m AC1
/// body and run long beside AC2's (the AC1 Long Sword is 1.05 m at its weenie scale, the AC2
/// starter sword 0.77 m); this brings the Long Sword to the AC2 sword's length.
pub const HELD_SCALE: f32 = 0.75;
/// AC1 `Placement.RightHandCombat`.
const RIGHT_HAND_COMBAT: u32 = 1;

/// The weapons and which one is in hand.
pub struct Ac1Weapons {
    pub weapons: Vec<Ac1Creature>,
    /// Which weapons have an AC1 icon served (`Ac1Icons`).
    has_icon: Vec<bool>,
    /// The weapon in hand; `None` leaves the session's AC2 sword there.
    pub current: Option<usize>,
    /// `--ac1-weapon-placement`: the AC1 placement frame the weapon takes in the hand.
    pub placement: u32,
    /// `--ac1-weapon-scale`: the held weapons' scale over their weenies' (`HELD_SCALE`).
    pub scale: f32,
    world: Option<ac2_client::DatWorld>,
    /// The right-hand holding point of each player visual seen.
    points: HashMap<u32, Option<HoldPoint>>,
}

impl Ac1Weapons {
    pub fn new(
        weapons: Vec<Ac1Creature>,
        icons: &Ac1Icons,
        store: Arc<ac2_db::ResourceStore>,
    ) -> Self {
        let has_icon = (0..weapons.len())
            .map(|index| icons.contains(Self::icon_id(index)))
            .collect();
        let world = match ac2_client::DatWorld::new(store) {
            Ok(world) => Some(world),
            Err(error) => {
                eprintln!("world-viewer: no holding locations: {error}");
                None
            }
        };
        Self {
            has_icon,
            current: (!weapons.is_empty()).then_some(0),
            weapons,
            placement: RIGHT_HAND_COMBAT,
            scale: HELD_SCALE,
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

    /// The weapons as inventory items for the HUD's session: each drawn by its AC1 model when
    /// wielded, equipped in the primary hand, with its AC1 icon (else the AC2 sword's).
    pub fn items(&self) -> Vec<ItemSpawn> {
        self.weapons
            .iter()
            .enumerate()
            .map(|(index, weapon)| ItemSpawn {
                id: Self::item_id(index),
                package: WEAPON_PACKAGE,
                name: Box::leak(weapon.weenie.name.clone().into_boxed_str()),
                plural: None,
                icon: if self.has_icon[index] {
                    Self::icon_id(index)
                } else {
                    ICON_SWORD
                },
                quantity: None,
                value: 50,
                container: PLAYER,
                durability: Some((50, 50)),
                stackable: false,
                max_stack: None,
                capacity: None,
                locations: Some((PRIMARY_HAND, PRIMARY_HAND)),
                entity: None,
                visual: Some(weapon.visual.raw()),
            })
            .collect()
    }

    /// The image DID weapon `index`'s AC1 icon is served under.
    pub fn icon_id(index: usize) -> u32 {
        FIRST_ICON + index as u32
    }

    /// The inventory item of weapon `index` (`items`).
    pub fn item_id(index: usize) -> u64 {
        FIRST_ITEM + index as u64
    }

    /// An AC1 weapon the session wields hangs in its AC1 placement frame, turned from AC1's hand
    /// axes into AC2's (the server's `RecvEvt_Parent` names the AC2 placement 0), at its weenie's
    /// `DefaultScale` (the scale ACE sends) times the held scale.
    pub fn place_wielded(&self, frame: &mut ac2_client::ClientFrame) {
        let first = self.weapons.first().map(|w| w.visual.raw());
        let last = self.weapons.last().map(|w| w.visual.raw());
        let (Some(first), Some(last)) = (first, last) else {
            return;
        };
        for object in &mut frame.objects {
            let Some(visual) = object.visual.filter(|v| (first..=last).contains(v)) else {
                continue;
            };
            let weapon = &self.weapons[(visual - first) as usize];
            object.body_scale = [weapon.weenie.scale * self.scale; 3];
            if let Some(holder) = object.holder.as_mut() {
                holder.orientation = Some(self.placement);
                holder.offset = ac2_math::Frame::new(
                    ac2_math::Vec3::ZERO,
                    ac1_assets::creature::AC1_HAND_TO_AC2,
                );
            }
        }
    }

    /// Puts the chosen weapon in `frame`'s player's right hand, in place of what the session
    /// hangs there.
    pub fn hold(&mut self, frame: &mut ac2_client::ClientFrame) {
        self.place_wielded(frame);
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
            body_scale: [weapon.weenie.scale * self.scale; 3],
            adaptive_degrade: false,
        })
    }
}
