//! The world-space overlays the HUD draws over the scene: the targeting reticle, the external
//! container's window, name plates and floating damage text (`gm3DTextUI`), and the radar's
//! input from the frame.

use std::collections::HashMap;

use ac2_math::{Mat4, Vec3};
use ac2_render::Camera;

use super::{drawn_objects, ClientView};
use crate::frame::ClientFrame;

impl ClientView {
    /// `gm3DTextUI::Update`: the reticle around the selected object, from its model box, its
    /// placement and the camera.
    pub(super) fn update_reticle(&mut self, frame: &ClientFrame, camera: &Camera, aspect: f32) {
        let selection = frame.selection.as_ref();
        let drawn = drawn_objects(frame);
        let placed = selection
            .and_then(|s| drawn.iter().find(|d| d.id == s.id))
            .and_then(|d| {
                let (key, world, _) = self.placed(d, self.drawn_frame(d))?;
                let mesh = self.scene.object_pick(key)?;
                Some((
                    world,
                    mesh.min,
                    mesh.max,
                    self.world_point(d.cell, d.origin),
                ))
            });
        let player = frame
            .player
            .as_ref()
            .map(|p| self.world_point(p.cell, p.origin));
        let view_projection = camera.view_projection(aspect);
        let project = |world: Mat4| {
            move |p: [f32; 3]| {
                let clip = view_projection * world * Vec3::from_array(p).extend(1.0);
                let w = if clip.w.abs() < 1e-6 { 1e-6 } else { clip.w };
                ([clip.x / w, clip.y / w], clip.w > 0.0)
            }
        };
        let projector = placed.map(|(world, ..)| project(world));
        let target = match (selection, placed, &projector) {
            (Some(s), Some((_, min, max, at)), Some(projector)) => Some(ac2_ui::ReticleTarget {
                id: s.id,
                bounds: (min.to_array(), max.to_array()),
                project: projector,
                planar_distance: player.map(|p| (at - p).truncate().length()),
            }),
            _ => None,
        };
        let outcome = self.hud.reticle(
            selection.map(|s| (s.id, s.name.as_deref())),
            selection.and_then(|s| s.info.as_ref()),
            target,
        );
        self.deselect |= outcome.deselect;
    }

    /// `gmExternalContainerUI::PositionWindow` for a container that just opened: its model's
    /// box centre projected (`SceneTool::TransformVector`); centred when it is not drawn or is
    /// behind the camera.
    pub(super) fn position_external_container(
        &mut self,
        frame: &ClientFrame,
        camera: &Camera,
        aspect: f32,
    ) {
        let Some(id) = self.hud.external_container_position() else {
            return;
        };
        let view_projection = camera.view_projection(aspect);
        let clip = drawn_objects(frame)
            .iter()
            .find(|d| d.id == id)
            .and_then(|d| {
                let (key, world, _) = self.placed(d, self.drawn_frame(d))?;
                let mesh = self.scene.object_pick(key)?;
                let centre = (mesh.min + mesh.max) * 0.5;
                let clip = view_projection * world * centre.extend(1.0);
                (clip.w > 0.0).then(|| [clip.x / clip.w, clip.y / clip.w])
            });
        self.hud.position_external_container(clip);
    }

    /// `gm3DTextUI::UpdateNames`: each named object's model box and projection, and its ground
    /// distance from the player (its radar offset).
    pub(super) fn update_names(&mut self, frame: &ClientFrame, camera: &Camera, aspect: f32) {
        let named = self.hud.named();
        if named.is_empty() {
            return;
        }
        let view_projection = camera.view_projection(aspect);
        let drawn = drawn_objects(frame);
        let mut objects = Vec::new();
        for id in named {
            let Some(blip) = frame.radar.blips.iter().find(|b| b.id == id) else {
                continue;
            };
            let Some(d) = drawn.iter().find(|d| d.id == id) else {
                continue;
            };
            let Some((key, world, _)) = self.placed(d, self.drawn_frame(d)) else {
                continue;
            };
            let Some(mesh) = self.scene.object_pick(key) else {
                continue;
            };
            let (x, y) = (f64::from(blip.offset[0]), f64::from(blip.offset[1]));
            let distance = (x * x + y * y) as f32;
            objects.push((
                id,
                world,
                mesh.min.to_array(),
                mesh.max.to_array(),
                distance,
            ));
        }
        let projectors: Vec<_> = objects
            .iter()
            .map(|&(_, world, ..)| {
                move |p: [f32; 3]| {
                    let clip = view_projection * world * Vec3::from_array(p).extend(1.0);
                    let w = if clip.w.abs() < 1e-6 { 1e-6 } else { clip.w };
                    ([clip.x / w, clip.y / w], clip.w > 0.0)
                }
            })
            .collect();
        let places: Vec<_> = objects
            .iter()
            .zip(&projectors)
            .map(
                |(&(id, _, min, max, distance), project)| ac2_ui::NamePlace {
                    id,
                    bounds: (min, max),
                    project,
                    xy_distance_sq: distance,
                },
            )
            .collect();
        let selection = frame.selection.as_ref().map_or(0, |s| s.id);
        self.hud.update_names(&places, selection, frame.radar.range);
    }

    /// `gm3DTextUI::UpdateDamage`: each floating damage text's object box and projection.
    pub(super) fn update_damage(&mut self, frame: &ClientFrame, camera: &Camera, aspect: f32) {
        let objects = self.hud.damage_objects();
        if objects.is_empty() {
            return;
        }
        let view_projection = camera.view_projection(aspect);
        let drawn = drawn_objects(frame);
        let mut boxes = Vec::new();
        for id in objects {
            let Some(d) = drawn.iter().find(|d| d.id == id) else {
                continue;
            };
            let Some((key, world, _)) = self.placed(d, self.drawn_frame(d)) else {
                continue;
            };
            let Some(mesh) = self.scene.object_pick(key) else {
                continue;
            };
            boxes.push((id, world, mesh.min.to_array(), mesh.max.to_array()));
        }
        let projectors: Vec<_> = boxes
            .iter()
            .map(|&(_, world, ..)| {
                move |p: [f32; 3]| {
                    let clip = view_projection * world * Vec3::from_array(p).extend(1.0);
                    let w = if clip.w.abs() < 1e-6 { 1e-6 } else { clip.w };
                    ([clip.x / w, clip.y / w], clip.w > 0.0)
                }
            })
            .collect();
        let places: Vec<_> = boxes
            .iter()
            .zip(&projectors)
            .map(|(&(id, _, min, max), project)| ac2_ui::DamagePlace {
                id,
                bounds: (min, max),
                project,
            })
            .collect();
        let selection = frame.selection.as_ref().map_or(0, |s| s.id);
        self.hud.update_damage(&places, selection);
    }
}

/// object's name, and the selection.
pub(super) fn radar_view(frame: &ClientFrame) -> ac2_ui::RadarView {
    let radar = &frame.radar;
    // The first object with each id names its blips (`find` order); a missing or unnamed one
    // gives "".
    let mut names: HashMap<u64, &str> = HashMap::with_capacity(frame.objects.len());
    for object in &frame.objects {
        names
            .entry(object.id)
            .or_insert_with(|| object.name.as_deref().unwrap_or(""));
    }
    let name = |id| {
        names
            .get(&id)
            .map_or_else(String::new, |n| (*n).to_string())
    };
    ac2_ui::RadarView {
        range: radar.range,
        heading_degrees: radar.heading_degrees,
        dots: radar
            .blips
            .iter()
            .map(|b| ac2_ui::RadarDot {
                id: b.id,
                color: b.color,
                shape: b.shape,
                offset: [b.offset[0], b.offset[1]],
                in_range: b.in_range,
                name: name(b.id),
            })
            .collect(),
        selection: frame.selection.as_ref().map_or(0, |s| s.id),
        coordinates: None,
    }
}
