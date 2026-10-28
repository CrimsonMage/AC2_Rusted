//! The world overlays `gm3DTextUI` places over the scene (the targeting reticle, the name plates
//! and the floating damage texts), `gmExternalContainerUI`'s window and the radar's view. Which of
//! them show, and where, is decided here from plain boxes: each drawn object's model box, its
//! projection ([`ac2_render::Projector`], the scene math) and its planar distance from the player,
//! which the client projects and the radar's blips.

use ac2_net::protocol::SelectionInfo;
use ac2_render::Projector;

use crate::element::UiTree;
use crate::{
    DamagePlace, Hud, NamePlace, RadarBlip, RadarDot, RadarFrame, RadarView, ReticleOutcome,
    ReticleTarget,
};

/// A drawn object the overlays may place: its id, its model box (`min`, `max`, model space), its
/// projection, and its planar distance from the player (`Position::localtolocal`), when both have
/// a cell.
#[derive(Debug, Clone, Copy)]
pub struct OverlayBox {
    pub id: u64,
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub projector: Projector,
    pub planar_from_player: Option<f32>,
}

impl Hud {
    /// `gm3DTextUI::Update`'s reticle: the selection, and its box when the client drew it.
    pub fn update_reticle_over(
        &mut self,
        tree: &mut UiTree,
        selection: Option<(u64, Option<&str>)>,
        info: Option<&SelectionInfo>,
        color: Option<[f32; 4]>,
        selected: Option<&OverlayBox>,
    ) -> ReticleOutcome {
        let project = selected.map(|b| move |p: [f32; 3]| b.projector.project(p));
        let target = match (selection, selected, &project) {
            (Some((id, _)), Some(b), Some(project)) => Some(ReticleTarget {
                id,
                bounds: (b.min, b.max),
                project,
                planar_distance: b.planar_from_player,
            }),
            _ => None,
        };
        self.update_reticle(tree, selection, info, color, target)
    }

    /// `gm3DTextUI::UpdateNames` and then `UpdateDamage` over the boxes the client projected:
    /// the plates of the named objects that have a radar blip, and the damage texts of the
    /// objects they hang over. A box the client did not draw places nothing.
    pub fn update_overlays(
        &mut self,
        tree: &mut UiTree,
        boxes: &[OverlayBox],
        blips: &[RadarBlip],
        selection: u64,
        range: f32,
    ) {
        let named = self.named();
        if !named.is_empty() {
            let mut found = Vec::new();
            for id in named {
                let Some(blip) = blips.iter().find(|b| b.id == id) else {
                    continue;
                };
                let Some(b) = boxes.iter().find(|b| b.id == id) else {
                    continue;
                };
                // `Position::xy_distance_squared`: the blip's offset, squared as a float.
                let (x, y) = (f64::from(blip.offset[0]), f64::from(blip.offset[1]));
                found.push((*b, (x * x + y * y) as f32));
            }
            let projectors: Vec<_> = found
                .iter()
                .map(|(b, _)| {
                    let projector = b.projector;
                    move |p: [f32; 3]| projector.project(p)
                })
                .collect();
            let places: Vec<_> = found
                .iter()
                .zip(&projectors)
                .map(|((b, distance), project)| NamePlace {
                    id: b.id,
                    bounds: (b.min, b.max),
                    project,
                    xy_distance_sq: *distance,
                })
                .collect();
            self.update_names(tree, &places, selection, range);
        }
        let objects = self.damage_objects();
        if !objects.is_empty() {
            let found: Vec<&OverlayBox> = objects
                .iter()
                .filter_map(|id| boxes.iter().find(|b| b.id == *id))
                .collect();
            let projectors: Vec<_> = found
                .iter()
                .map(|b| {
                    let projector = b.projector;
                    move |p: [f32; 3]| projector.project(p)
                })
                .collect();
            let places: Vec<_> = found
                .iter()
                .zip(&projectors)
                .map(|(b, project)| DamagePlace {
                    id: b.id,
                    bounds: (b.min, b.max),
                    project,
                })
                .collect();
            self.update_damage(tree, &places, selection);
        }
    }

    /// `gmExternalContainerUI::PositionWindow` for the container that wants a place: its box's
    /// centre projected; centred when the container is not drawn or is behind the camera. Marks
    /// the HUD changed when the window moves.
    pub fn position_external_container_over(
        &mut self,
        tree: &mut UiTree,
        dirty: &mut bool,
        container: Option<u64>,
        boxes: &[OverlayBox],
    ) {
        let Some(id) = container else {
            return;
        };
        let clip = boxes.iter().find(|b| b.id == id).and_then(|b| {
            let centre = [0, 1, 2].map(|i| (b.min[i] + b.max[i]) * 0.5);
            b.projector.project_front(centre)
        });
        if let Some(panel) = self.external_container.as_mut() {
            panel.position_window_clip(tree, clip);
            *dirty = true;
        }
    }
}

/// `gmRadarUI::Update`'s view of the frame: the range, the heading and each blip, named by the
/// first object with its id (`""` for a missing or unnamed one), with the selection (0 for none).
pub fn radar_view(
    radar: &RadarFrame,
    selection: u64,
    objects: &[(u64, Option<&str>)],
) -> RadarView {
    let mut names: std::collections::HashMap<u64, &str> =
        std::collections::HashMap::with_capacity(objects.len());
    for &(id, name) in objects {
        names.entry(id).or_insert_with(|| name.unwrap_or(""));
    }
    let name = |id| {
        names
            .get(&id)
            .map_or_else(String::new, |n| (*n).to_string())
    };
    RadarView {
        range: radar.range,
        heading_degrees: radar.heading_degrees,
        dots: radar
            .blips
            .iter()
            .map(|b| RadarDot {
                id: b.id,
                color: b.color,
                shape: b.shape,
                offset: [b.offset[0], b.offset[1]],
                in_range: b.in_range,
                name: name(b.id),
            })
            .collect(),
        selection,
        coordinates: None,
    }
}
