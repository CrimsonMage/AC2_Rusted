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

/// The creatures and where they stand.
pub struct Ac1Showcase {
    creatures: Vec<Ac1Creature>,
    /// Each creature's idle state, made at its first frame.
    players: Vec<Option<IdlePlayer>>,
    /// `--ac1-heartbeat`: the heartbeat interval in place of the weenies'.
    pub heartbeat: Option<f64>,
    models: Arc<Ac1Models>,
    /// The ring's centre: the cell and block-relative origin the player first stood at.
    anchor: Option<(u32, [f32; 3])>,
}

impl Ac1Showcase {
    /// Builds every weenie in `weenies` from the AC1 DATs in `dat_dir`; a weenie that does not
    /// build is reported and left out.
    pub fn load(dat_dir: &Path, weenies: &Path) -> Result<Self, String> {
        let dats = Ac1Dats::open(dat_dir).map_err(|e| format!("AC1 DATs: {e:?}"))?;
        let mut paths: Vec<_> = std::fs::read_dir(weenies)
            .map_err(|e| format!("{}: {e}", weenies.display()))?
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| path.extension().is_some_and(|ext| ext == "sql"))
            .collect();
        paths.sort();
        let mut creatures = Vec::new();
        let mut models = Ac1Models::default();
        for path in paths {
            let weenie = match AceWeenie::load(&path) {
                Ok(weenie) => weenie,
                Err(error) => {
                    eprintln!("world-viewer: {error}");
                    continue;
                }
            };
            let visual =
                DataId::new(ac1_assets::creature::AC1_VISUAL_BASE + creatures.len() as u32);
            match dats.creature(&weenie, visual) {
                Ok(creature) => {
                    models.insert(&creature);
                    creatures.push(creature);
                }
                Err(error) => eprintln!("world-viewer: {}: {error}", weenie.name),
            }
        }
        println!(
            "ac1: {} creatures ({})",
            creatures.len(),
            creatures
                .iter()
                .map(|c| c.weenie.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
        Ok(Self {
            players: vec![None; creatures.len()],
            heartbeat: None,
            creatures,
            models: Arc::new(models),
            anchor: None,
        })
    }

    /// Centres the ring on `origin` (block-relative) in `cell` instead of the player's place.
    pub fn set_anchor(&mut self, cell: u32, origin: [f32; 3]) {
        self.anchor = Some((cell, origin));
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
        self.creatures
            .iter()
            .zip(self.players.iter_mut())
            .enumerate()
            .map(|(index, (creature, player))| {
                let angle = index as f32 / count as f32 * 360.0;
                let (sin, cos) = angle.to_radians().sin_cos();
                let x = centre[0] + radius * sin;
                let y = centre[1] + radius * cos;
                let z = ground(cell, [x, y, centre[2]]).unwrap_or(centre[2]);
                // Heading is clockwise from north; face the ring's centre.
                let heading = (angle + 180.0) % 360.0;
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
