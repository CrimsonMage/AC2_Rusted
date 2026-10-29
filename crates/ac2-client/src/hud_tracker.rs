//! The Tracker over the client (`ac2_macro::tracker`, a modern addition): the world objects of the
//! frame go to it, the Tracker window's requests run against it, and its effects are carried out.
//! A match is announced in chat, picking one selects it and points the floating arrow at it (as
//! does a coordinate pressed in chat), the radar brackets the matches, and the pattern list is
//! read and saved on a short-lived worker so the frame never touches a file.

use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::time::Duration;

use ac2_macro::tracker::{
    Effect, ObservedObject, PatternRead, PositionQuery, Presentation, Tracker, TrackerPosition,
    TrackerUpdate, CANONICAL_DIRECTORY, LEGACY_SOURCES, PATTERN_FILE,
};
use ac2_ui::tracker_ui::TrackerRequest;
use ac2_ui::HudRequest;

use crate::frame::ClientFrame;
use crate::gameplay::Command;
use crate::hud_arrow::{arrow_view, format_place, ArrowTarget};
use crate::hud_view::HudView;

/// The chat type Tracker lines go out as (`TEXT_TYPE_STANDARD`, shown in every window).
const TRACKER_TEXT_TYPE: u32 = ac2_ui::TEXT_TYPE_STANDARD;

/// The Tracker and the file work in flight for it.
#[derive(Default)]
pub(crate) struct TrackerHost {
    tracker: Tracker,
    /// The first read: the Tracker's own list, then the older one it would import.
    loading: Option<Receiver<(PatternRead, PatternRead)>>,
    /// A reload the window asked for.
    reloading: Option<Receiver<PatternRead>>,
    /// The objects' identity at the last reconcile.
    revision: u64,
    /// The latest presentation, shown when the window opens.
    shown: Option<Presentation>,
    window_was_open: bool,
    /// What the floating arrow points at.
    arrow: Option<ArrowTarget>,
    /// The matches announced in chat, newest last (name, object), so a press on a name in a
    /// Tracker line selects what it found; at most `DETECTIONS_KEPT`.
    detections: Vec<(Arc<str>, u64)>,
}

/// How many announced matches a press on a chat line can still select.
const DETECTIONS_KEPT: usize = 64;
/// The start of the chat line that announces a match, and the word before its coordinates.
const FOUND_PREFIX: &str = "[Tracker] Found ";
const FOUND_AT: &str = " at ";

/// A place target is reached within this many metres (the arrow then clears).
const ARRIVED_METRES: f64 = 5.0;

/// The Tracker's own file.
fn pattern_path() -> PathBuf {
    ac2_prefs::paths::config_dir()
        .join(CANONICAL_DIRECTORY)
        .join(PATTERN_FILE)
}

/// The older lists the Tracker imports when it has none: next to the client's config directory.
fn legacy_paths() -> Vec<PathBuf> {
    let base = ac2_prefs::paths::config_dir();
    let base = base.parent().map(PathBuf::from).unwrap_or(base);
    LEGACY_SOURCES
        .iter()
        .map(|pieces| {
            let mut path = base.clone();
            for piece in *pieces {
                path.push(piece);
            }
            path.join(PATTERN_FILE)
        })
        .collect()
}

fn read(path: &std::path::Path) -> PatternRead {
    PatternRead::from_result(match std::fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    })
}

/// The frame's world objects as the Tracker sees them: none is in a container (contained items
/// are not world objects).
fn observed(frame: &ClientFrame) -> impl Iterator<Item = ObservedObject<'_>> {
    frame.objects.iter().map(|object| ObservedObject {
        object: object.id,
        name: object.name.as_deref(),
        containment_known: true,
        container: None,
        cell: Some(object.cell),
    })
}

/// A cheap identity of the frame's object set: it changes when an object comes or goes.
fn object_revision(frame: &ClientFrame) -> u64 {
    frame
        .objects
        .iter()
        .fold(frame.objects.len() as u64, |acc, object| {
            acc.wrapping_mul(0x100_0000_01B3).wrapping_add(object.id)
        })
}

struct FramePositions<'a>(&'a ClientFrame);

impl PositionQuery for FramePositions<'_> {
    fn position(&self, object: u64) -> Option<TrackerPosition> {
        let found = self.0.objects.iter().find(|o| o.id == object)?;
        Some(TrackerPosition {
            cell: found.cell,
            origin: found.origin,
        })
    }
}

impl HudView {
    /// One frame of the Tracker: the list loads once the player is in the world, the window's
    /// requests run, the objects are matched, and the window shows the list while it is open.
    pub(crate) fn tracker_frame(
        &mut self,
        frame: &ClientFrame,
        requests: Vec<HudRequest>,
        commands: &mut Vec<Command>,
    ) {
        if !frame.in_world {
            return;
        }
        let mut tracker_requests = Vec::new();
        for request in requests {
            match request {
                HudRequest::PointAt { north, east } => {
                    self.tracker.arrow = Some(ArrowTarget::Place { north, east });
                }
                HudRequest::Tracker(request) => tracker_requests.push(request),
                HudRequest::TextPress(press) => self.tracker_line_press(frame, &press, commands),
                _ => {}
            }
        }
        let requests = tracker_requests;
        if !self.tracker.tracker.is_initialized() && self.tracker.loading.is_none() {
            let (send, receive) = channel();
            let spawned = std::thread::Builder::new()
                .name("ac2 tracker read".into())
                .spawn(move || {
                    let canonical = read(&pattern_path());
                    let legacy = legacy_paths()
                        .iter()
                        .map(|path| read(path))
                        .find(|r| !matches!(r, PatternRead::Missing))
                        .unwrap_or(PatternRead::Missing);
                    let _ = send.send((canonical, legacy));
                });
            if spawned.is_ok() {
                self.tracker.loading = Some(receive);
            }
        }
        let revision = object_revision(frame);
        if let Some(loading) = &self.tracker.loading {
            if let Ok((canonical, legacy)) = loading.try_recv() {
                self.tracker.loading = None;
                let update =
                    self.tracker
                        .tracker
                        .load_initial(canonical, legacy, revision, observed(frame));
                self.tracker.revision = revision;
                self.apply_tracker(update, commands);
            }
        }
        if !self.tracker.tracker.is_initialized() {
            return;
        }
        if let Some(reloading) = &self.tracker.reloading {
            if let Ok(source) = reloading.try_recv() {
                self.tracker.reloading = None;
                let label = pattern_path().display().to_string();
                let update =
                    self.tracker
                        .tracker
                        .reload_source(source, &label, revision, observed(frame));
                self.apply_tracker(update, commands);
            }
        }
        for request in requests {
            self.tracker_request(frame, request, commands);
        }
        if self.tracker.revision != revision {
            self.tracker.revision = revision;
            let update = self.tracker.tracker.reconcile(revision, observed(frame));
            self.apply_tracker(update, commands);
        }
        let released = self
            .tracker
            .tracker
            .observe_selection(frame.selection.as_ref().map(|s| s.id));
        // The Tracker let its pick go (the player selected something else, or it left).
        if matches!(self.tracker.arrow, Some(ArrowTarget::Object(_)))
            && (released || self.tracker.tracker.pointer_target().is_none())
        {
            self.tracker.arrow = None;
        }
        self.present_tracker(frame);
        self.update_arrow(frame);
        let tracked: Vec<u64> = self
            .tracker
            .tracker
            .view()
            .matches
            .iter()
            .map(|m| m.object)
            .collect();
        if let Some(radar) = self.panels.radar.as_mut() {
            radar.set_tracked(&tracked);
        }
    }

    /// A press on a chat line: on the name in a Tracker "Found" line, the newest match of that
    /// name is selected in the world (and the client turns to it), if it is still around.
    fn tracker_line_press(
        &mut self,
        frame: &ClientFrame,
        press: &ac2_ui::TextPress,
        commands: &mut Vec<Command>,
    ) {
        let Some(name) = found_name_at(&press.line, press.column) else {
            return;
        };
        let object = self
            .tracker
            .detections
            .iter()
            .rev()
            .find(|(n, id)| n.as_ref() == name && frame.objects.iter().any(|o| o.id == *id))
            .map(|(_, id)| *id);
        match object {
            Some(id) => {
                commands.push(Command::Select(Some(id)));
                commands.push(Command::LookAtSelection(Some(id)));
            }
            None => {
                let line = format!("[Tracker] {name} is no longer around.");
                self.chat_display(TRACKER_TEXT_TYPE, &line);
            }
        }
    }

    /// The floating arrow turned to its target, hidden without one; a place reached clears it.
    fn update_arrow(&mut self, frame: &ClientFrame) {
        let view = self.tracker.arrow.and_then(|target| {
            let (view, distance) = arrow_view(frame, target)?;
            match target {
                ArrowTarget::Place { north, east } if distance < ARRIVED_METRES => {
                    self.tracker.arrow = None;
                    let line = format!("[Tracker] Arrived at {}", format_place(north, east));
                    self.chat_display(TRACKER_TEXT_TYPE, &line);
                    None
                }
                _ => Some(view),
            }
        });
        if let (Some(arrow), Some((tree, _))) =
            (self.panels.guide_arrow.as_mut(), self.hud.as_mut())
        {
            arrow.update(tree, view.as_ref());
            if tree.take_changed() {
                self.dirty = true;
            }
        }
    }

    fn tracker_request(
        &mut self,
        frame: &ClientFrame,
        request: TrackerRequest,
        commands: &mut Vec<Command>,
    ) {
        let update = match request {
            TrackerRequest::Add { pattern, exact } => {
                self.tracker
                    .tracker
                    .add_pattern(&pattern, exact, observed(frame))
            }
            TrackerRequest::AddSelected => {
                let name = frame.selection.as_ref().and_then(|s| s.name.clone());
                self.tracker
                    .tracker
                    .add_selected(name.as_deref(), observed(frame))
            }
            TrackerRequest::Reload => {
                if self.tracker.reloading.is_none() {
                    let (send, receive) = channel();
                    if std::thread::Builder::new()
                        .name("ac2 tracker read".into())
                        .spawn(move || {
                            let _ = send.send(read(&pattern_path()));
                        })
                        .is_ok()
                    {
                        self.tracker.reloading = Some(receive);
                    }
                }
                TrackerUpdate::default()
            }
            TrackerRequest::RemovePattern(index) => self.tracker.tracker.remove_pattern(index),
            TrackerRequest::Select(object) => {
                if let Some(id) = self.tracker.tracker.select_match(object) {
                    commands.push(Command::Select(Some(id)));
                    self.tracker.arrow = Some(ArrowTarget::Object(id));
                }
                TrackerUpdate::default()
            }
            TrackerRequest::Dismiss(object) => {
                self.tracker.tracker.dismiss_match(object);
                TrackerUpdate::default()
            }
        };
        self.apply_tracker(update, commands);
    }

    /// The Tracker's effects: the file saved on a worker, the chat lines shown.
    fn apply_tracker(&mut self, update: TrackerUpdate, _commands: &mut [Command]) {
        for effect in update.effects {
            match effect {
                Effect::SavePatterns(bytes) => save_patterns(bytes),
                Effect::Detected { object, name, at } => {
                    let line = match at {
                        Some(at) => format!("{FOUND_PREFIX}{name}{FOUND_AT}{at}"),
                        None => format!("{FOUND_PREFIX}{name}"),
                    };
                    let kept = &mut self.tracker.detections;
                    kept.push((Arc::clone(&name), object));
                    if kept.len() > DETECTIONS_KEPT {
                        kept.remove(0);
                    }
                    self.chat_display(TRACKER_TEXT_TYPE, &line);
                }
                Effect::Notice(text) => self.chat_display(TRACKER_TEXT_TYPE, &text),
            }
        }
    }

    /// The list with distances, re-projected as the Tracker allows, shown while the window is
    /// open (and at once when it opens).
    fn present_tracker(&mut self, frame: &ClientFrame) {
        let player = frame.player.as_ref().map(|p| TrackerPosition {
            cell: p.cell,
            origin: p.origin,
        });
        let now = Duration::from_secs_f64(frame.server_time.max(0.0));
        if let Some(presentation) =
            self.tracker
                .tracker
                .project(now, player, &FramePositions(frame))
        {
            self.tracker.shown = Some(presentation);
            self.tracker.window_was_open = false;
        }
        let (Some(ui), Some((tree, _))) = (self.panels.tracker.as_mut(), self.hud.as_mut()) else {
            return;
        };
        let open = ui.is_shown();
        if open && !self.tracker.window_was_open {
            if let Some(presentation) = &self.tracker.shown {
                ui.present(tree, presentation);
                self.dirty = true;
            }
        }
        self.tracker.window_was_open = open;
    }
}

/// The name a column of a Tracker "Found" line lands on (`[Tracker] Found <name> at <where>`).
fn found_name_at(line: &str, column: usize) -> Option<&str> {
    let rest = line.strip_prefix(FOUND_PREFIX)?;
    let name = rest.rfind(FOUND_AT).map_or(rest, |at| &rest[..at]);
    let start = FOUND_PREFIX.chars().count();
    let end = start + name.chars().count();
    (start..end).contains(&column).then_some(name)
}

/// Writes the pattern list on a worker; a failure is reported on the console (the Tracker's own
/// notice needs the frame).
fn save_patterns(bytes: Arc<[u8]>) {
    let _ = std::thread::Builder::new()
        .name("ac2 tracker save".into())
        .spawn(move || {
            let path = pattern_path();
            let result = path
                .parent()
                .map_or(Ok(()), std::fs::create_dir_all)
                .and_then(|()| std::fs::write(&path, &bytes));
            if let Err(error) = result {
                eprintln!(
                    "ac2: the Tracker's pattern list {}: {error}",
                    path.display()
                );
            }
        });
}

#[cfg(test)]
mod tests {
    use super::found_name_at;

    #[test]
    fn a_press_on_the_name_of_a_found_line_names_it() {
        let line = "[Tracker] Found White Rabbit at 43.0N, 11.7E";
        let on = |word: &str| line.find(word).unwrap();
        assert_eq!(found_name_at(line, on("Rabbit")), Some("White Rabbit"));
        assert_eq!(found_name_at(line, on("43.0N")), None);
        assert_eq!(found_name_at(line, on("Found")), None);
        assert_eq!(
            found_name_at("[Tracker] Found Gold Ore", 20),
            Some("Gold Ore")
        );
        assert_eq!(found_name_at("Bob says hi", 3), None);
    }
}
