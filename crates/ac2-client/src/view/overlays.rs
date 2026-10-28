//! The world-space overlays the HUD draws over the scene (`gm3DTextUI`: the targeting reticle,
//! name plates and floating damage text; `gmExternalContainerUI`'s window). The client projects
//! each drawn object's model box through the camera ([`ac2_render::Projector`]) into an
//! [`ac2_ui::OverlayBox`]; which overlays show, and where, is the HUD's
//! ([`ac2_ui::Hud::update_overlays`]). The radar's view of the frame is the HUD's too.

use ac2_render::Camera;
use ac2_ui::OverlayBox;

use super::{drawn_objects, ClientView};
use crate::frame::ClientFrame;

impl ClientView {
    /// The overlays' boxes for this frame: each drawn object among the named ones, the floating
    /// damage texts' objects, the selection and the external container, with the planar distance
    /// from the player.
    pub(super) fn overlay_boxes(
        &self,
        frame: &ClientFrame,
        camera: &Camera,
        aspect: f32,
    ) -> Vec<OverlayBox> {
        let mut ids = self.hud.named();
        ids.extend(self.hud.damage_objects());
        ids.extend(frame.selection.as_ref().map(|s| s.id));
        ids.extend(self.hud.external_container_position());
        ids.sort_unstable();
        ids.dedup();
        let view_projection = camera.view_projection(aspect);
        let drawn = drawn_objects(frame);
        let player = frame
            .player
            .as_ref()
            .map(|p| self.world_point(p.cell, p.origin));
        ids.iter()
            .filter_map(|&id| {
                let d = drawn.iter().find(|d| d.id == id)?;
                let (key, world, _) = self.placed(d, self.drawn_frame(d))?;
                let mesh = self.scene.object_pick(key)?;
                let at = self.world_point(d.cell, d.origin);
                Some(OverlayBox {
                    id,
                    min: mesh.min.to_array(),
                    max: mesh.max.to_array(),
                    projector: ac2_render::Projector::new(view_projection, world),
                    planar_from_player: player.map(|p| (at - p).truncate().length()),
                })
            })
            .collect()
    }

    /// `gm3DTextUI::Update`'s reticle for this frame, over the selection's box.
    pub(super) fn update_reticle(&mut self, frame: &ClientFrame, boxes: &[OverlayBox]) {
        let selection = frame.selection.as_ref();
        let selected = selection.and_then(|s| boxes.iter().find(|b| b.id == s.id));
        // `CheckName`: `GetName(quantity)`, rendered from the string tables when they are there.
        let name = selection.and_then(|s| {
            s.name_info
                .as_ref()
                .and_then(|info| self.hud.render_info(info))
                .or_else(|| s.name.as_deref().map(str::to_owned))
        });
        let outcome = self.hud.reticle(
            selection.map(|s| (s.id, name.as_deref())),
            selection.and_then(|s| s.info.as_ref()),
            // `CheckName`: the selection's radar-look colour.
            selection.and_then(|s| {
                frame
                    .radar
                    .blips
                    .iter()
                    .find(|b| b.id == s.id)
                    .map(|b| b.color)
            }),
            selected,
        );
        self.deselect |= outcome.deselect;
    }

    /// `gmExternalContainerUI::PositionWindow` for a container that just opened.
    pub(super) fn position_external_container(&mut self, boxes: &[OverlayBox]) {
        self.hud.position_external_container(boxes);
    }

    /// `gm3DTextUI::UpdateNames` and `UpdateDamage` for this frame, over the boxes.
    pub(super) fn update_overlays(&mut self, frame: &ClientFrame, boxes: &[OverlayBox]) {
        let selection = frame.selection.as_ref().map_or(0, |s| s.id);
        self.hud
            .update_overlays(boxes, &frame.radar.blips, selection, frame.radar.range);
    }
}

/// `gmRadarUI::Update`'s view of the frame: each object's name, the selection and the radar.
pub(super) fn radar_view(frame: &ClientFrame) -> ac2_ui::RadarView {
    let objects: Vec<(u64, Option<&str>)> = frame
        .objects
        .iter()
        .map(|o| (o.id, o.name.as_deref()))
        .collect();
    let selection = frame.selection.as_ref().map_or(0, |s| s.id);
    ac2_ui::radar_view(&frame.radar, selection, &objects)
}
