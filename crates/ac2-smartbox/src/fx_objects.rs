//! What an object's FX does to the world beyond sound and particles: the objects it creates
//! (`FXManager::DoCreateObjectFX`), its colour (`FXManager::DoColorFX`), the hiding of its target
//! (`FXManager::DoTargetAndCloningFXs`), and the fade-out and destruction of those objects
//! (`CObjectMaint::FadeAndDestroyObject`). `FXManager::StopFX` and the expiry of an FX in
//! `FXManager::UseTime` share one teardown: the FX's objects fade and are destroyed, its targets
//! are unhidden, and its colour is restored when the FX asked for it.
//!
//! The client creates the objects itself (`CObjectMaint::CreateTemporaryObject`), so they are
//! [`PendingObject`]s here until the client adds them to its remote objects each animation step
//! (`Client::apply_fx_objects`, in `client_fx_objects`). None of this needs a sound device.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use ac2_assets::{Ac2FxObject, Ac2FxObjectAttach, Ac2VisualFxParticles};

/// The temporary ids the client gives the objects its FX create (`IIDUtils` type 0x13, numbered
/// from the top of the instance range so they cannot meet the server's own).
const FX_OBJECT_BASE: u64 = 0x13FF_FE00_0000_0000;
/// `FXManager::StopFX`'s colour restore: each channel ramps back to 1.0 over 0.1 seconds
/// (`PHYSICSOBJREP`'s property calls with 1.0 and 0.1).
const RESTORE_SECONDS: f32 = 0.1;
/// `CObjectMaint::FadeAndDestroyObject(id, 0.5, 3.0)`: the fade, and the destruction time from
/// the fade's start.
const FADE_SECONDS: f64 = 0.5;
const DESTROY_SECONDS: f64 = 3.0;
const WHITE: [f32; 4] = [1.0; 4];
/// The slider `FXManager::DoCreateObjectFX` moves an object's alpha on (`CPhysicsObj::DoSlider`).
pub const ALPHA_SLIDER: u32 = 0x4000_001a;

/// The FX nodes that create or hide objects for `fx` (`FXManager::DoTargettedFX`'s node list).
pub fn fx_nodes(table: &Ac2VisualFxParticles, fx: u32) -> Vec<Ac2FxObject> {
    table
        .get(fx)
        .map(|list| list.iter().map(|a| a.object).collect())
        .unwrap_or_default()
}

/// The FX state the object part keeps per emitter and FX id (`FXState`'s object fields).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FxRecord {
    /// `m_bStartupFX`: a startup FX stops only when forced, so `StopFX` leaves it running.
    pub startup: bool,
    /// `m_object_ids`: the objects the FX created, faded and destroyed when it stops.
    pub objects: Vec<u64>,
    /// `m_objects_to_unhide`: the targets the FX hid.
    pub unhide: Vec<u64>,
    /// `m_restore_color`: the FX's colour lasts `color_lifetime`, then restores.
    pub restore_colour: bool,
    /// `m_end_time`: when the FX ends by itself (`None` for never).
    pub end: Option<f64>,
}

/// A colour channel set in motion (`PHYSICSOBJREP::SetProperty` with a ramp): linear from
/// `from` to `to` over `seconds` from `start`.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Ramp {
    from: [f32; 4],
    to: [f32; 4],
    start: f64,
    seconds: f32,
}

impl Ramp {
    fn at(&self, now: f64) -> [f32; 4] {
        let t = if self.seconds <= 0.0 {
            1.0
        } else {
            ((now - self.start) / f64::from(self.seconds)).clamp(0.0, 1.0) as f32
        };
        std::array::from_fn(|i| self.from[i] + (self.to[i] - self.from[i]) * t)
    }

    fn finished(&self, now: f64) -> bool {
        now >= self.start + f64::from(self.seconds)
    }
}

/// `CObjectMaint`'s pending deletion of a faded object: the fade's start and the time it goes.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Fade {
    start: f64,
    destroy_at: f64,
}

/// An object an FX asked for, to be created by the client at its emitter, where its node's
/// attachment puts it (`Ac2FxObjectAttach`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PendingObject {
    pub id: u64,
    pub emitter: u64,
    pub visual: u32,
    pub attach: Ac2FxObjectAttach,
    /// The node's `m_ramp_time`, which sets how long the object takes to fade in.
    pub ramp_time: f32,
}

/// The two alpha moves `FXManager::DoCreateObjectFX` makes on a created object, as
/// `(pos, time)` for `CPhysicsObj::DoSlider(0, ALPHA_SLIDER, pos, time)`: the object starts
/// clear at once, then fades to opaque over `ramp_time`, or over 0.25 s when that is zero.
pub fn create_alpha_moves(ramp_time: f32) -> [(f32, f32); 2] {
    let fade_in = if ramp_time == 0.0 { 0.25 } else { ramp_time };
    [(0.0, 0.0), (1.0, fade_in)]
}

/// What the renderer tints and hides: each object's colour times its fade, and the hidden ones.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FxLooks {
    /// Objects an FX hid (`DoTargetAndCloningFXs`'s `m_hide_target`).
    pub hidden: BTreeSet<u64>,
    /// Objects whose colour or fade differs from white: r, g, b, a.
    pub tints: BTreeMap<u64, [f32; 4]>,
}

/// The FX-made objects, colours and hides of every emitter.
#[derive(Debug, Default)]
pub struct FxObjects {
    /// The objects whose startup FX have run (`CPhysicsObj::InitObjectEnd`), once per object.
    startup_objects: HashSet<u64>,
    records: HashMap<u64, HashMap<u32, FxRecord>>,
    colours: HashMap<u64, Ramp>,
    hidden: BTreeSet<u64>,
    fades: BTreeMap<u64, Fade>,
    pending: Vec<PendingObject>,
    next: u64,
    now: f64,
}

impl FxObjects {
    /// The FX's record on `emitter`, when it is running (`FXManager::m_current_fx`).
    pub fn record(&self, emitter: u64, fx: u32) -> Option<&FxRecord> {
        self.records.get(&emitter)?.get(&fx)
    }

    /// `FXManager::DoTargettedFX` for a new FX, its object part: each node's colour
    /// (`DoColorFX`), the object it creates (`DoCreateObjectFX`, which hands the object to the
    /// target), and the target's hide and destruction (`DoTargetAndCloningFXs`). A node that
    /// creates an object without a behaviour keeps it on the FX; a behaviour is not run here, so
    /// its object is kept as the retail fallback keeps a failed one (`DoCreateObjectFX`).
    /// Returns without a record when no node has an object fact, or when the FX is running
    /// already (`DoTargettedFX` only updates a running FX's scalar).
    pub fn start(&mut self, emitter: u64, fx: u32, nodes: &[Ac2FxObject], now: f64) {
        if !nodes.iter().any(Ac2FxObject::is_active) || self.record(emitter, fx).is_some() {
            return;
        }
        let mut record = FxRecord::default();
        let mut end = None;
        for node in nodes {
            if let Some(colour) = node.color {
                self.set_colour(emitter, colour, node.ramp_time, now);
                if node.color_lifetime != 0.0 {
                    record.restore_colour = true;
                }
            }
            let mut target = None;
            if node.vdesc != 0 {
                let id = self.new_object(emitter, node);
                record.objects.push(id);
                target = Some(id);
                // `DoTargettedFX`: a non-negative lifetime ends the FX; a negative one never does.
                end = (node.lifetime > 0.0).then(|| now + f64::from(node.lifetime));
            }
            if let Some(target) = target {
                if node.hide_target {
                    self.hidden.insert(target);
                    record.unhide.push(target);
                }
                if node.destroy_target {
                    self.fade_and_destroy(target, now);
                }
            }
        }
        if record.restore_colour {
            // `m_color_lifetime` overrides the object's lifetime for a colour that restores.
            if let Some(lifetime) = nodes
                .iter()
                .find(|node| node.color.is_some() && node.color_lifetime > 0.0)
            {
                end = Some(now + f64::from(lifetime.color_lifetime));
            }
        }
        record.end = end;
        self.records.entry(emitter).or_default().insert(fx, record);
    }

    /// `FXManager::DoStartupFX`'s object part, run when the object is created: each node that
    /// creates an object does so (`DoCreateObjectFX`), and the FX is recorded as a startup one,
    /// which `StopFX` leaves running. Colour and hides are not part of the startup branch.
    pub fn start_startup(&mut self, emitter: u64, fx: u32, nodes: &[Ac2FxObject]) {
        if self.record(emitter, fx).is_some() {
            return;
        }
        let mut record = FxRecord {
            startup: true,
            ..FxRecord::default()
        };
        for node in nodes.iter().filter(|node| node.vdesc != 0) {
            record.objects.push(self.new_object(emitter, node));
        }
        self.records.entry(emitter).or_default().insert(fx, record);
    }

    /// `FXManager::StopFX`'s object part: a running FX (not a startup one) tears down; the
    /// colour of every node that restores is ramped back to white.
    pub fn stop(&mut self, emitter: u64, fx: u32, nodes: &[Ac2FxObject], now: f64) {
        if self
            .record(emitter, fx)
            .is_some_and(|record| record.startup)
        {
            return;
        }
        if let Some(record) = self.take_record(emitter, fx) {
            self.teardown(emitter, &record, now);
        }
        if nodes.iter().any(|node| node.restore) {
            self.set_colour(emitter, WHITE, RESTORE_SECONDS, now);
        }
    }

    /// `FXManager::UseTime`'s expired FX: each one whose end has come tears down, as `stop`
    /// does (the restore of its colour included).
    pub fn expire(&mut self, now: f64) {
        let expired: Vec<(u64, u32)> = self
            .records
            .iter()
            .flat_map(|(emitter, fxs)| {
                fxs.iter()
                    .filter(|(_, record)| record.end.is_some_and(|end| end <= now))
                    .map(|(fx, _)| (*emitter, *fx))
            })
            .collect();
        for (emitter, fx) in expired {
            if let Some(record) = self.take_record(emitter, fx) {
                self.teardown(emitter, &record, now);
            }
        }
    }

    /// Marks `object` as having run its startup FX; `false` when it already had.
    pub fn mark_startup(&mut self, object: u64) -> bool {
        self.startup_objects.insert(object)
    }

    /// Forgets the startup marks of objects `keep` rejects, so they run again when they return.
    pub fn retain_startup(&mut self, keep: impl Fn(u64) -> bool) {
        self.startup_objects.retain(|id| keep(*id));
    }

    /// The emitters that have FX records, in id order.
    pub fn emitters(&self) -> Vec<u64> {
        let mut ids: Vec<u64> = self.records.keys().copied().collect();
        ids.sort_unstable();
        ids
    }

    /// The emitter left the world: its FX stop (`DestroyAllTransientFX`).
    pub fn remove_emitter(&mut self, emitter: u64, now: f64) {
        let Some(fxs) = self.records.remove(&emitter) else {
            return;
        };
        for record in fxs.values() {
            self.teardown(emitter, record, now);
        }
        self.colours.remove(&emitter);
    }

    /// `FXManager::StopFX`'s and `UseTime`'s shared teardown: the FX's objects fade and go, its
    /// targets come unhidden, and a colour that restores ramps back to white.
    fn teardown(&mut self, emitter: u64, record: &FxRecord, now: f64) {
        for &id in &record.objects {
            self.fade_and_destroy(id, now);
        }
        for id in &record.unhide {
            self.hidden.remove(id);
        }
        if record.restore_colour {
            self.set_colour(emitter, WHITE, RESTORE_SECONDS, now);
        }
    }

    fn take_record(&mut self, emitter: u64, fx: u32) -> Option<FxRecord> {
        let fxs = self.records.get_mut(&emitter)?;
        let record = fxs.remove(&fx);
        if fxs.is_empty() {
            self.records.remove(&emitter);
        }
        record
    }

    fn new_object(&mut self, emitter: u64, node: &Ac2FxObject) -> u64 {
        self.next += 1;
        let id = FX_OBJECT_BASE | self.next;
        self.pending.push(PendingObject {
            id,
            emitter,
            visual: node.vdesc,
            attach: node.attach,
            ramp_time: node.ramp_time,
        });
        id
    }

    /// `FXManager::DestroyObject`: an object with a temporary id (`IIDUtils::VerifyTemporary`,
    /// `temporary`: the caller's `id_type` answer) fades and is destroyed; any other is left
    /// alone. Returns whether it was.
    pub fn destroy_object(&mut self, id: u64, temporary: bool, now: f64) -> bool {
        if temporary {
            self.fade_and_destroy(id, now);
        }
        temporary
    }

    /// `FXManager::GetCurrentFX`: the FX ids running on `emitter` that are not startup FX
    /// (`m_bStartupFX` clear), in id order (`map<unsigned long, ...>`). Only FX with object
    /// facts have a record here; the scalar and target of each are not kept.
    pub fn current_fx(&self, emitter: u64) -> Vec<u32> {
        let mut ids: Vec<u32> = self
            .records
            .get(&emitter)
            .into_iter()
            .flatten()
            .filter(|(_, record)| !record.startup)
            .map(|(&fx, _)| fx)
            .collect();
        ids.sort_unstable();
        ids
    }

    /// `CObjectMaint::FadeAndDestroyObject(id, 0.5, 3.0)`.
    pub fn fade_and_destroy(&mut self, id: u64, now: f64) {
        self.fades.insert(
            id,
            Fade {
                start: now,
                destroy_at: now + DESTROY_SECONDS,
            },
        );
    }

    /// `DoColorFX`: the emitter's colour ramps to `colour` over `seconds`.
    pub fn set_colour(&mut self, emitter: u64, colour: [f32; 4], seconds: f32, now: f64) {
        let from = self.colour(emitter, now).unwrap_or(WHITE);
        self.colours.insert(
            emitter,
            Ramp {
                from,
                to: colour,
                start: now,
                seconds,
            },
        );
    }

    /// The emitter's colour now, `None` while it is white.
    pub fn colour(&self, emitter: u64, now: f64) -> Option<[f32; 4]> {
        self.colours
            .get(&emitter)
            .filter(|ramp| !(ramp.to == WHITE && ramp.finished(now)))
            .map(|ramp| ramp.at(now))
    }

    /// Takes the objects FX asked for since the last call.
    pub fn take_pending(&mut self) -> Vec<PendingObject> {
        std::mem::take(&mut self.pending)
    }

    /// The per-frame pass: records the time, drops the colours that have finished at white,
    /// and returns the faded objects whose destruction has come (the client removes them).
    pub fn step(&mut self, now: f64) -> Vec<u64> {
        self.now = now;
        self.colours
            .retain(|_, ramp| ramp.to != WHITE || !ramp.finished(now));
        let gone: Vec<u64> = self
            .fades
            .iter()
            .filter(|(_, fade)| now >= fade.destroy_at)
            .map(|(id, _)| *id)
            .collect();
        for id in &gone {
            self.fades.remove(id);
            self.hidden.remove(id);
        }
        gone
    }

    /// What the renderer draws differently this frame, at the last step's time.
    pub fn looks(&self) -> FxLooks {
        let mut tints = BTreeMap::new();
        for &emitter in self.colours.keys() {
            if let Some(colour) = self.colour(emitter, self.now) {
                tints.insert(emitter, colour);
            }
        }
        for (&id, fade) in &self.fades {
            let t = ((self.now - fade.start) / FADE_SECONDS).clamp(0.0, 1.0) as f32;
            let alpha = 1.0 - t;
            let mut tint = tints.get(&id).copied().unwrap_or(WHITE);
            tint[3] *= alpha;
            tints.insert(id, tint);
        }
        FxLooks {
            hidden: self.hidden.clone(),
            tints,
        }
    }
}

/// `PHYSICSOBJREP::Draw`: the drawn colour is the object's colour (its FX colour and fade) times
/// its animation colour scale (the colour and opacity sliders). Computed in f64, rounded once.
pub fn drawn_tint(colour: [f32; 4], anim: [f32; 4]) -> [f32; 4] {
    std::array::from_fn(|i| (f64::from(colour[i]) * f64::from(anim[i])) as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vdesc_node(lifetime: f32) -> Ac2FxObject {
        Ac2FxObject {
            vdesc: 0x1F00_0001,
            lifetime,
            ..Ac2FxObject::default()
        }
    }

    /// `DoCreateObjectFX` hands the object to the FX; a stop fades it out and destroys it three
    /// seconds on, and the FX's record goes.
    #[test]
    fn a_stopped_fx_fades_and_destroys_the_objects_it_created() {
        let mut fx = FxObjects::default();
        fx.start(7, 9, &[vdesc_node(0.0)], 10.0);
        let pending = fx.take_pending();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].emitter, 7);
        let id = pending[0].id;
        fx.stop(7, 9, &[], 11.0);
        assert!(fx.record(7, 9).is_none());
        assert!(fx.step(11.25).is_empty(), "the fade runs first");
        assert!((fx.looks().tints[&id][3] - 0.5).abs() < 1e-6, "half faded");
        assert!(fx.step(13.9).is_empty());
        assert_eq!(
            fx.step(14.0),
            vec![id],
            "destroyed three seconds after the fade began"
        );
    }

    /// A startup FX is not stopped by `StopFX` (`m_bStartupFX` without the force flag).
    /// `DoCreateObjectFX`'s alpha moves: an instant clear, then a fade over the ramp (0.25 s for
    /// a zero ramp).
    #[test]
    fn a_created_object_fades_in_over_its_ramp_time() {
        assert_eq!(create_alpha_moves(0.0), [(0.0, 0.0), (1.0, 0.25)]);
        assert_eq!(create_alpha_moves(0.5), [(0.0, 0.0), (1.0, 0.5)]);
        let mut fx = FxObjects::default();
        let node = Ac2FxObject {
            ramp_time: 2.0,
            ..vdesc_node(0.0)
        };
        fx.start(7, 9, &[node], 0.0);
        let pending = fx.take_pending();
        assert_eq!(pending[0].ramp_time, 2.0);
    }

    #[test]
    fn a_startup_fx_keeps_running_through_stop() {
        let mut fx = FxObjects::default();
        fx.start_startup(7, 9, &[vdesc_node(0.0)]);
        fx.stop(7, 9, &[], 1.0);
        assert!(fx.record(7, 9).is_some_and(|record| record.startup));
        assert!(fx.step(10.0).is_empty());
    }

    /// `m_hide_target`: the created object is hidden until the FX stops, then it shows (its
    /// fade keeps it out of the frame only once it is destroyed).
    #[test]
    fn a_hidden_target_shows_again_when_the_fx_stops() {
        let mut fx = FxObjects::default();
        let node = Ac2FxObject {
            hide_target: true,
            ..vdesc_node(0.0)
        };
        fx.start(7, 9, &[node], 0.0);
        let id = fx.take_pending()[0].id;
        assert!(fx.looks().hidden.contains(&id));
        fx.stop(7, 9, &[], 1.0);
        assert!(!fx.looks().hidden.contains(&id));
    }

    /// `DoColorFX` ramps the emitter's colour, and `StopFX` restores a node that asks for it.
    #[test]
    fn a_colour_ramps_and_restores_on_stop() {
        let mut fx = FxObjects::default();
        let node = Ac2FxObject {
            color: Some([1.0, 0.0, 0.0, 1.0]),
            ramp_time: 1.0,
            restore: true,
            ..Ac2FxObject::default()
        };
        fx.start(7, 9, &[node], 0.0);
        assert_eq!(fx.colour(7, 0.5), Some([1.0, 0.5, 0.5, 1.0]));
        fx.stop(7, 9, &[node], 2.0);
        assert_eq!(fx.colour(7, 2.05), Some([1.0, 0.5, 0.5, 1.0]));
        assert_eq!(fx.colour(7, 2.2), None, "back to white");
    }

    /// An object whose alpha slider is at 0 draws clear; white animation leaves the FX colour.
    #[test]
    fn the_animation_scale_multiplies_the_fx_colour() {
        assert_eq!(
            drawn_tint([1.0, 0.5, 0.25, 1.0], [1.0; 4]),
            [1.0, 0.5, 0.25, 1.0]
        );
        assert_eq!(
            drawn_tint([1.0; 4], [0.5, 1.0, 1.0, 0.0]),
            [0.5, 1.0, 1.0, 0.0]
        );
        assert_eq!(drawn_tint([1.0; 4], [1.0, 1.0, 1.0, 0.25])[3], 0.25);
    }

    /// `FXManager::GetCurrentFX` lists the running FX that are not startup FX, and
    /// `FXManager::DestroyObject` fades only temporary ids.
    #[test]
    fn current_fx_skips_startup_fx_and_destroy_object_wants_a_temporary_id() {
        let mut fx = FxObjects::default();
        let node = vdesc_node(10.0);
        fx.start(1, 5, &[node], 0.0);
        fx.start(1, 3, &[node], 0.0);
        fx.start_startup(1, 4, &[node]);
        assert_eq!(fx.current_fx(1), vec![3, 5]);
        assert!(fx.current_fx(2).is_empty());
        assert!(!fx.destroy_object(0x8000_0000_0000_0001, false, 1.0));
        assert!(fx.fades.is_empty());
        assert!(fx.destroy_object(FX_OBJECT_BASE | 1, true, 1.0));
        assert!(fx.fades.contains_key(&(FX_OBJECT_BASE | 1)));
    }
}
