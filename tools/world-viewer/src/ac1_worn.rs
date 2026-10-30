//! `--ac1-armor DIR`: Asheron's Call 1 armour worn by the AC2 player. Every ACE weenie patch file
//! in DIR (worn items) is built, for each AC2 player body's AC1 heritage body, into the parts its
//! ClothingBase swaps in (`Ac1Dats::worn_pieces`: AC1's `CPartArray::SetPart`). AC2 has no AC1
//! parts to swap, so each piece hangs from the AC2 bone its part is paired with
//! (`ac1_assets::retarget`) as a held child (`CPhysicsObj::UpdateChild`), turned so that it stands
//! as the AC1 part stood, AC1's joints (`Ac1Dats::limb_joints`) on AC2's, and stretched to span
//! the AC2 bones.
//!
//! As AC2's own armour does (`MeshReplacementAtom`: the body's meshes under the armour give way
//! to the armour's), the AC2 body's triangles under the armour are dropped: the player is drawn
//! under a visual of its own whose model is the AC2 body without the triangles whose vertices
//! are all weighted most to bones of covered AC1 parts (`Retarget::bone_parts`: the neck goes
//! with the head, a clavicle with its upper arm; a tail is no part's). The AC2 head, neck and
//! hands stay AC2's, except on the races in `WHOLE_AC1_BODY`, drawn as the whole AC1 body.

use std::collections::{BTreeSet, HashMap};
use std::path::Path;
use std::sync::Arc;

use ac1_assets::{Ac1Dats, Ac1Models, AceWeenie};
use ac2_client::{ClientFrame, Holder, ObjectFrame};
use ac2_core::DataId;
use ac2_db::ResourceStore;
use ac2_math::Vec3;
use ac2_physics::HoldPoint;
use ac2_render::object::{ObjectModel, RenderSettings};
use ac2_weenie::Appearance;

use crate::ac1_motion::{ac1_body, Ac1Motion};

/// The pieces' visual DIDs (`AC1_VISUAL_BASE` onwards), and the dressed bodies'.
const PIECE_VISUALS: u32 = 0xA00;
const BODY_VISUALS: u32 = 0xB80;
/// The pieces' object ids.
const FIRST_PIECE_ID: u64 = 0x5100;
/// The holding locations the pieces hang from: past every Setup's own.
const FIRST_LOCATION: u32 = 0x100;
/// The AC2 races drawn as their whole AC1 heritage body (armour and bare parts) in place of
/// the AC2 body: the Lugian (species 2), whose massive AC2 build the AC1 pieces cannot be fitted
/// to, so it wears AC1's Lugian as AC1 drew it, at the AC2 Lugian's size.
const WHOLE_AC1_BODY: [u32; 1] = [0x2];
/// The AC1 player body's drawn parts (`HUMAN`'s 0 abdomen to 16 head).
const AC1_PLAYER_PARTS: usize = 17;

/// A built piece: the AC1 body part it covers and the visual it is drawn under.
#[derive(Debug, Clone, Copy)]
struct Piece {
    part: usize,
    visual: u32,
}

/// One AC2 player body dressed: the visual its uncovered AC2 triangles are drawn under and the
/// pieces over them.
#[derive(Debug, Clone)]
struct Dressed {
    body: u32,
    pieces: Vec<Piece>,
}

/// The worn items' pieces for each AC2 player body.
pub struct Ac1Worn {
    /// By the AC2 player visual.
    dressed: HashMap<u32, Dressed>,
}

impl Ac1Worn {
    /// Builds every weenie in `dir` for each AC2 player body `players` lists (visual, race,
    /// sex), adding the pieces and the dressed bodies to `models`.
    pub fn load(
        dats: &Ac1Dats,
        dir: &Path,
        players: &[(DataId, u32, u32)],
        store: &ResourceStore,
        motion: &mut Ac1Motion,
        models: &mut Ac1Models,
    ) -> Result<Self, String> {
        let mut paths: Vec<_> = std::fs::read_dir(dir)
            .map_err(|e| format!("{}: {e}", dir.display()))?
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| path.extension().is_some_and(|ext| ext == "sql"))
            .collect();
        paths.sort();
        let items: Vec<AceWeenie> = paths
            .iter()
            .filter_map(|path| {
                AceWeenie::load(path)
                    .map_err(|error| eprintln!("world-viewer: {error}"))
                    .ok()
            })
            .collect();
        let base = ac1_assets::creature::AC1_VISUAL_BASE;
        let mut next_piece = base + PIECE_VISUALS;
        let mut next_body = base + BODY_VISUALS;
        let mut dressed = HashMap::new();
        for &(visual, race, sex) in players {
            let Some((wearer, _)) = ac1_body(race, sex) else {
                continue;
            };
            let mut pieces = Vec::new();
            for item in &items {
                match dats.worn_pieces(item, wearer, next_piece) {
                    Ok(built) => add(built, &mut pieces, &mut next_piece, models),
                    Err(error) => {
                        eprintln!("world-viewer: {} on {wearer:#010x}: {error}", item.name)
                    }
                }
            }
            if pieces.is_empty() {
                continue;
            }
            let mut covered: BTreeSet<usize> = pieces.iter().map(|p| p.part).collect();
            if WHOLE_AC1_BODY.contains(&race) {
                // The rest of the AC1 body is drawn bare, and the AC2 body under all of it goes.
                let bare: Vec<usize> = (0..AC1_PLAYER_PARTS)
                    .filter(|p| !covered.contains(p))
                    .collect();
                match dats.body_pieces(wearer, &bare, next_piece) {
                    Ok(built) => add(built, &mut pieces, &mut next_piece, models),
                    Err(error) => eprintln!("world-viewer: bare parts of {wearer:#010x}: {error}"),
                }
                covered.extend(0..AC1_PLAYER_PARTS);
            }
            let Some(retarget) = motion.retarget(visual.raw(), race, sex) else {
                continue;
            };
            let owners = retarget.bone_parts();
            let Some(model) =
                ac2_render::object::load_object_model(store, visual, &RenderSettings::default())
                    .ok()
                    .flatten()
            else {
                eprintln!(
                    "world-viewer: no AC2 body for visual {:#010x}",
                    visual.raw()
                );
                continue;
            };
            let body = undressed_body(
                store,
                visual,
                model,
                &owners,
                &covered,
                DataId::new(next_body),
                models,
            );
            next_body += 1;
            dressed.insert(visual.raw(), Dressed { body, pieces });
        }
        if !items.is_empty() {
            println!(
                "ac1: {} worn items ({}) on {} bodies, {} pieces",
                items.len(),
                items
                    .iter()
                    .map(|w| w.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
                dressed.len(),
                next_piece - base - PIECE_VISUALS
            );
        }
        if next_piece > base + BODY_VISUALS || next_body > base + 0xC00 {
            return Err("too many AC1 armour pieces or bodies for their visual ranges".into());
        }
        Ok(Self { dressed })
    }

    /// Dresses the player (AC2 race `race`, sex `sex`): hangs its pieces from its bones and
    /// draws it under its dressed body's visual.
    pub fn wear(&self, frame: &mut ClientFrame, motion: &mut Ac1Motion, race: u32, sex: u32) {
        let Some(player) = frame.player.as_ref() else {
            return;
        };
        let Some(visual) = player.visual else {
            return;
        };
        let Some(dressed) = self.dressed.get(&visual) else {
            return;
        };
        let Some(retarget) = motion.retarget(visual, race, sex) else {
            return;
        };
        let mut worn = Vec::new();
        for (index, piece) in dressed.pieces.iter().enumerate() {
            let Some((bone, mut transform)) = retarget.part_on_bone(piece.part) else {
                continue;
            };
            // The stretch is the piece's own scale; the bone point only places and turns it.
            let scale = std::mem::replace(&mut transform.scale, Vec3::ONE);
            worn.push(ObjectFrame {
                id: FIRST_PIECE_ID + index as u64,
                visual: Some(piece.visual),
                setup: None,
                name: None,
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
                    location: FIRST_LOCATION + index as u32,
                    orientation: None,
                    point: Some(HoldPoint {
                        group: 0,
                        group_key: None,
                        bone,
                        transform,
                    }),
                    offset: ac2_math::Frame::IDENTITY,
                }),
                fx_follow: None,
                container: None,
                wielder: Some(player.id),
                appearance: Arc::new(Appearance::default()),
                pose: Vec::new(),
                anim_scale: [1.0; 4],
                body_scale: scale.to_array(),
                adaptive_degrade: false,
            });
        }
        frame.objects.extend(worn);
        if let Some(player) = frame.player.as_mut() {
            player.visual = Some(dressed.body);
        }
    }
}

/// Adds built pieces to `pieces` and their models to `models`, moving `next` past their visuals.
fn add(
    built: Vec<ac1_assets::Ac1WornPiece>,
    pieces: &mut Vec<Piece>,
    next: &mut u32,
    models: &mut Ac1Models,
) {
    *next += built.len() as u32;
    for piece in built {
        pieces.push(Piece {
            part: piece.part,
            visual: piece.visual.raw(),
        });
        models.insert_model(piece.visual, piece.model, piece.images);
    }
}

/// AC2 player visual `visual`'s model (`model`) without the root group's triangles under the
/// armour: those whose three vertices are each weighted most to a bone whose AC1 part
/// (`owners`) is in `covered`. Added to `models` under `body` at the visual's scale; its DID.
fn undressed_body(
    store: &ResourceStore,
    visual: DataId,
    mut model: ObjectModel,
    owners: &[Option<usize>],
    covered: &BTreeSet<usize>,
    body: DataId,
    models: &mut Ac1Models,
) -> u32 {
    for part in model.parts.iter_mut().filter(|part| part.slot.0 == 0) {
        let Some(skin) = part.skin.clone() else {
            continue;
        };
        let under: Vec<bool> = skin
            .iter()
            .map(|influences| {
                influences
                    .iter()
                    .max_by(|a, b| a.1.total_cmp(&b.1))
                    .and_then(|(bone, _)| owners.get(*bone as usize).copied().flatten())
                    .is_some_and(|owner| covered.contains(&owner))
            })
            .collect();
        for batch in &mut part.batches {
            let mut kept: Vec<u32> = batch
                .indices
                .chunks_exact(3)
                .filter(|tri| !tri.iter().all(|&i| under.get(i as usize) == Some(&true)))
                .flatten()
                .copied()
                .collect();
            // The degrade ranges count batches, so an emptied batch stays, as one degenerate
            // triangle (an index buffer cannot be empty).
            if kept.is_empty() {
                kept = vec![0; 3];
            }
            batch.indices = Arc::from(kept);
        }
    }
    model.visual = body;
    let scale = ac2_render::object::total_visual_desc(store, visual, None)
        .ok()
        .flatten()
        .map_or(ac2_math::Vec3::ONE, |total| total.scale);
    models.insert_scaled(body, model, scale);
    body.raw()
}
