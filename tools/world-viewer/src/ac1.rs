//! `--ace-weenies DIR`: Asheron's Call 1 creatures standing in the AC2 world. Every ACE weenie
//! patch file (`*.sql`) in DIR is built from the AC1 DATs (`--ac1-dat-dir`, or `AC1_DAT_DIR`) by
//! `ac1-assets`, its model handed to the view as a foreign model, and the creatures stand in a
//! ring around where the player first stands, facing in, each playing its motion table's idle
//! cycle and, every heartbeat, the idle actions its ACE HeartBeat emotes roll for (`IdlePlayer`;
//! `--ac1-heartbeat SECONDS` shortens the weenies' interval). They have no AI and never move.

use std::path::Path;
use std::sync::Arc;

use ac1_assets::{Ac1Creature, Ac1Dats, Ac1Models, AceWeenie, IdlePlayer};
use ac2_client::{ObjectFrame, PlayerFrame};
use ac2_core::DataId;
use ac2_physics::{Frame, Heading, Quaternion, Vector};
use ac2_weenie::Appearance;

/// The first creature's object id (the crowd starts at 0x100).
const FIRST_ID: u64 = 0x4000;
/// The ring's smallest radius and the arc each creature takes, in metres.
const MIN_RADIUS: f32 = 6.0;
const SPACING: f32 = 4.0;
/// The line-up's spacing in metres.
const LINEUP_SPACING: f32 = 3.0;
/// How far north of the anchor the line-up stands.
const LINEUP_AHEAD: f32 = 6.0;

/// The first visual DID of the creatures and of the weapons (`AC1_VISUAL_BASE` onwards).
const WEENIE_VISUALS: u32 = 0;
const WEAPON_VISUALS: u32 = 0x800;

/// Every ACE weenie patch file in `dir`, sorted, built from `dats` under visual DIDs from
/// `AC1_VISUAL_BASE + first`, their models added to `models`.
fn build_dir(
    dats: &Ac1Dats,
    dir: &Path,
    first: u32,
    models: &mut Ac1Models,
) -> Result<Vec<Ac1Creature>, String> {
    let mut paths: Vec<_> = std::fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().is_some_and(|ext| ext == "sql"))
        .collect();
    paths.sort();
    let mut built = Vec::new();
    for path in paths {
        let weenie = match AceWeenie::load(&path) {
            Ok(weenie) => weenie,
            Err(error) => {
                eprintln!("world-viewer: {error}");
                continue;
            }
        };
        let visual =
            DataId::new(ac1_assets::creature::AC1_VISUAL_BASE + first + built.len() as u32);
        match dats.creature(&weenie, visual) {
            Ok(creature) => {
                models.insert(&creature);
                built.push(creature);
            }
            Err(error) => eprintln!("world-viewer: {}: {error}", weenie.name),
        }
    }
    Ok(built)
}

/// The creatures and where they stand.
pub struct Ac1Showcase {
    creatures: Vec<Ac1Creature>,
    /// Each creature's idle state, made at its first frame.
    players: Vec<Option<IdlePlayer>>,
    /// `--ac1-heartbeat`: the heartbeat interval in place of the weenies'.
    pub heartbeat: Option<f64>,
    /// `--ac1-lineup`: a row west to east, `LINEUP_AHEAD` north of the anchor, every creature
    /// facing south towards it.
    pub lineup: bool,
    models: Arc<Ac1Models>,
    /// `--ac1-weapons`: the weapons for the player's hand.
    pub weapons: crate::ac1_held::Ac1Weapons,
    /// The creatures' and weapons' particles (their Setups' default scripts).
    pub fx: crate::ac1_fx::Ac1Fx,
    /// The player's AC1 motion (the panel's toggle).
    pub motion: crate::ac1_motion::Ac1Motion,
    /// `--ac1-armor`: the AC1 armour the player wears.
    pub worn: Option<crate::ac1_worn::Ac1Worn>,
    /// The ring's centre: the cell and block-relative origin the player first stood at.
    anchor: Option<(u32, [f32; 3])>,
}

impl Ac1Showcase {
    /// Builds every weenie in `weenies` (creatures) and `weapons` from the AC1 DATs in
    /// `dat_dir`; a weenie that does not build is reported and left out.
    pub fn load(
        dat_dir: &Path,
        weenies: Option<&Path>,
        weapons: Option<&Path>,
        armor: Option<&Path>,
        players: &[(DataId, u32, u32)],
        held_scale: Option<f32>,
        store: Arc<ac2_db::ResourceStore>,
    ) -> Result<Self, String> {
        let dats = Ac1Dats::open(dat_dir).map_err(|e| format!("AC1 DATs: {e:?}"))?;
        let mut models = Ac1Models::default();
        let creatures = match weenies {
            Some(dir) => build_dir(&dats, dir, WEENIE_VISUALS, &mut models)?,
            None => Vec::new(),
        };
        let weapons = match weapons {
            Some(dir) => build_dir(&dats, dir, WEAPON_VISUALS, &mut models)?,
            None => Vec::new(),
        };
        let mut motion = crate::ac1_motion::Ac1Motion::new(dats.clone(), Arc::clone(&store));
        let worn = armor
            .map(|dir| {
                crate::ac1_worn::Ac1Worn::load(
                    &dats,
                    dir,
                    players,
                    &store,
                    &mut motion,
                    &mut models,
                )
            })
            .transpose()?;
        // The weapons' AC1 icons, served to the HUD before it is built.
        let icons = Arc::new(crate::ac1_held::Ac1Icons::load(&dats, &weapons));
        ac2_ui::set_foreign_ui_images(Arc::clone(&icons) as Arc<dyn ac2_ui::ForeignUiImages>);
        let mut fx = crate::ac1_fx::Ac1Fx::default();
        for creature in &creatures {
            fx.add(
                &dats,
                &mut models,
                creature.visual.raw(),
                creature.script,
                1.0,
            );
            fx.add_lights(creature.visual.raw(), &creature.lights);
        }
        for weapon in &weapons {
            let scale = held_scale.unwrap_or(crate::ac1_held::HELD_SCALE);
            fx.add(
                &dats,
                &mut models,
                weapon.visual.raw(),
                weapon.script,
                scale,
            );
            fx.add_lights(weapon.visual.raw(), &weapon.lights);
        }
        for (kind, list) in [("creatures", &creatures), ("weapons", &weapons)] {
            if !list.is_empty() {
                println!(
                    "ac1: {} {kind} ({})",
                    list.len(),
                    list.iter()
                        .map(|c| c.weenie.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
        }
        Ok(Self {
            players: vec![None; creatures.len()],
            heartbeat: None,
            lineup: false,
            creatures,
            models: Arc::new(models),
            motion,
            weapons: crate::ac1_held::Ac1Weapons::new(weapons, &icons, store),
            worn,
            fx,
            anchor: None,
        })
    }

    /// Centres the ring on `origin` (block-relative) in `cell` instead of the player's place.
    pub fn set_anchor(&mut self, cell: u32, origin: [f32; 3]) {
        self.anchor = Some((cell, origin));
    }

    /// Dresses `frame`'s player (AC2 race `race`, sex `sex`) in the `--ac1-armor` pieces.
    pub fn wear(&mut self, frame: &mut ac2_client::ClientFrame, race: u32, sex: u32) {
        if let Some(worn) = &self.worn {
            worn.wear(frame, &mut self.motion, race, sex);
        }
    }

    /// What the view asks for the creatures' models.
    pub fn models(&self) -> Arc<Ac1Models> {
        Arc::clone(&self.models)
    }

    /// The creatures as world objects at client time `time`, around the player's first place
    /// (`None` until the player is in the world). `ground` gives the terrain under an origin.
    pub fn objects(
        &mut self,
        player: Option<&PlayerFrame>,
        time: f64,
        ground: impl Fn(u32, [f32; 3]) -> Option<f32>,
    ) -> Vec<ObjectFrame> {
        if self.anchor.is_none() {
            self.anchor = player.map(|p| (p.cell, p.origin));
        }
        let Some((cell, centre)) = self.anchor else {
            return Vec::new();
        };
        let count = self.creatures.len();
        let heartbeat = self.heartbeat;
        let radius = MIN_RADIUS.max(count as f32 * SPACING / std::f32::consts::TAU);
        let lineup = self.lineup;
        self.creatures
            .iter()
            .zip(self.players.iter_mut())
            .enumerate()
            .map(|(index, (creature, player))| {
                let (x, y, heading) = if lineup {
                    let offset = (index as f32 - (count as f32 - 1.0) / 2.0) * LINEUP_SPACING;
                    (centre[0] + offset, centre[1] + LINEUP_AHEAD, 180.0)
                } else {
                    let angle = index as f32 / count as f32 * 360.0;
                    let (sin, cos) = angle.to_radians().sin_cos();
                    // Heading is clockwise from north; face the ring's centre.
                    (
                        centre[0] + radius * sin,
                        centre[1] + radius * cos,
                        (angle + 180.0) % 360.0,
                    )
                };
                let z = ground(cell, [x, y, centre[2]]).unwrap_or(centre[2]);
                let mut frame = Frame::new(Vector::new(0.0, 0.0, 0.0), Quaternion::IDENTITY);
                frame.set_heading(Heading::from_degrees(heading));
                let q = frame.q;
                let pose = creature
                    .idle
                    .as_ref()
                    .map(|idle| {
                        player
                            .get_or_insert_with(|| IdlePlayer::new(idle, index as u64 + 1, time))
                            .pose(idle, time, heartbeat)
                            .to_vec()
                    })
                    .unwrap_or_default();
                let scale = creature.weenie.scale;
                ObjectFrame {
                    id: FIRST_ID + index as u64,
                    visual: Some(creature.visual.raw()),
                    setup: None,
                    name: Some(creature.weenie.name.as_str().into()),
                    stack_name: None,
                    cell,
                    origin: [x, y, z],
                    rotation: [q.w, q.x, q.y, q.z],
                    heading_degrees: Some(heading),
                    in_world: true,
                    lost_cell: None,
                    active: true,
                    velocity: [0.0; 3],
                    contact: true,
                    mode: None,
                    animated_mode: None,
                    motion: [0.0; 3],
                    holder: None,
                    fx_follow: None,
                    container: None,
                    wielder: None,
                    appearance: Arc::new(Appearance::default()),
                    pose,
                    anim_scale: [1.0; 4],
                    body_scale: [scale; 3],
                    adaptive_degrade: true,
                }
            })
            .collect()
    }
}
