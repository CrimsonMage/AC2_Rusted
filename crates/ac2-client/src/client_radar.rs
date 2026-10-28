//! The client's side of the radar (`gmRadarUI`, `gmNavigationUI`): the session's notices and
//! objects and the scripts' answers go to `ac2_ui::RadarState`, which decides the radar's looks,
//! groove and map notes. The client supplies the ranges (`gRadarControl::CallIFun_InqRadarRanges`),
//! the looks (`gmCWInterface::InqRadarLook`) and the positions.

use ac2_physics::Position;
use ac2_script::host::radar;
use ac2_script::vm::InterpStatus;
use ac2_ui::{groove_of, MapMarker};
use ac2_weenie::{GameplayNotice, SessionEvent};

use crate::client::Client;
use ac2_audio::is_outside;

impl Client {
    /// The radar's part of the frame's events: the notices (looks, map notes), the objects
    /// that left, then the looks the scripts can now answer. Returns what the scripts produced
    /// meanwhile, for the frame's events.
    pub(crate) fn radar_use_time(&mut self, events: &[SessionEvent]) -> Vec<SessionEvent> {
        // `gmRadarUI::Init` reads the ranges as the radar comes up.
        self.radar_ranges();
        for event in events {
            match event {
                SessionEvent::Notice(notice) => self.radar_notice(notice),
                SessionEvent::ObjectRemoved(id) => self.radar.remove(*id),
                SessionEvent::LeftWorld | SessionEvent::Disconnected(_) => {
                    self.radar.init();
                    self.radar.forget_markers();
                }
                _ => {}
            }
        }
        self.radar_looks()
    }

    fn radar_notice(&mut self, notice: &GameplayNotice) {
        match notice {
            // `gmRadarUI::RecvNotice_ChangeRadarLook`: the player's look means `Init`.
            GameplayNotice::ChangeRadarLook(id) if Some(*id) == self.session.player() => {
                self.radar.init();
            }
            GameplayNotice::ChangeRadarLook(id) => self.radar.remove(*id),
            GameplayNotice::ReloadOptions => self.radar.init(),
            GameplayNotice::AddMarker {
                kind,
                id,
                cell,
                image,
                message,
            } if self.session.player().is_some() => self.radar.add_marker(MapMarker {
                kind: *kind,
                id: *id,
                cell: *cell,
                image: *image,
                message: message.clone(),
            }),
            GameplayNotice::RemoveMarker { kind, id } => self.radar.remove_marker(*kind, *id),
            GameplayNotice::ClearMarkers(kind) => self.radar.clear_markers(*kind),
            _ => {}
        }
    }

    /// `gRadarControl::CallIFun_InqRadarRanges`, read once after each `Init`; without a script
    /// VM the ranges are 0.
    fn radar_ranges(&mut self) -> (f32, f32) {
        if let Some(ranges) = self.radar.ranges() {
            return ranges;
        }
        let Some(scripts) = self.scripts.as_mut() else {
            return (0.0, 0.0);
        };
        let ranges = radar::inq_radar_ranges(scripts.host.vm_mut(), 0.0, 0.0).unwrap_or((0.0, 0.0));
        self.radar.set_ranges(ranges);
        ranges
    }

    /// `gmRadarUI::AddObject` for each object whose process runs and that `AddObject` has not
    /// run for yet. A look that waits for data is asked again on a later frame.
    fn radar_looks(&mut self) -> Vec<SessionEvent> {
        let mut produced = Vec::new();
        let Some(scripts) = self.scripts.as_mut() else {
            return produced;
        };
        let pending: Vec<(u64, i8)> = self
            .session
            .objects()
            .iter()
            .filter(|o| !self.radar.knows(o.id))
            .filter(|o| scripts.host.instance(o.id).is_some())
            .map(|o| (o.id, groove_of(&o.qualities)))
            .collect();
        for (id, groove) in pending {
            let Some(scripts) = self.scripts.as_mut() else {
                break;
            };
            match scripts.host.inq_radar_look(id) {
                Ok((outcome, look)) => {
                    if outcome.status != InterpStatus::Interrupted {
                        self.radar.add(id, groove, look);
                    }
                    self.script_outcome(id, None, Ok(outcome), &mut produced);
                }
                Err(error) => {
                    self.radar.add(id, groove, None);
                    if let Some(scripts) = self.scripts.as_mut() {
                        scripts
                            .problems
                            .push(format!("{id:#018x} InqRadarLook: {error:?}"));
                    }
                }
            }
        }
        produced
    }

    /// The object's position: the player's, or the remote's.
    fn object_position(&self, id: u64) -> Option<Position> {
        match self.player.as_ref() {
            Some(player) if player.id == id => Some(player.position()),
            _ => self.remotes.get(id).and_then(|r| r.position()),
        }
    }

    /// `gmRadarUI::UpdateGroove`: once due, the groove levels of the objects within range go to
    /// the music when the total changes (`SoundManager::SetGrooveLevel`).
    pub(crate) fn groove_use_time(&mut self, now: f64) {
        if !self.radar.groove_due(now) {
            return;
        }
        let Some(player) = self.player.as_ref().map(|p| p.position()) else {
            return;
        };
        let ranges = self.radar_ranges();
        let members: Vec<(i8, Option<Position>)> = self
            .radar
            .known_grooves()
            .map(|(id, groove)| (groove, self.object_position(id)))
            .collect();
        let outside = is_outside(player.cell_id);
        if let Some(level) = self
            .radar
            .update_groove(now, &player, ranges, outside, &members)
        {
            if let Some(audio) = self.audio.as_mut() {
                audio.manager.set_groove_level(i32::from(level));
            }
        }
    }

    /// The radar for [`crate::ClientFrame::radar`].
    pub(crate) fn radar_frame(&self) -> crate::frame::RadarFrame {
        let player = self.player.as_ref().map(|p| p.position());
        let selection = self.selection_frame().map(|s| s.id);
        let outside = player.as_ref().is_some_and(|p| is_outside(p.cell_id));
        self.radar.frame(player.as_ref(), outside, selection, |id| {
            self.object_position(id)
        })
    }
}
