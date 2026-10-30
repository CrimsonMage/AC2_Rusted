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

/// The FX nodes that create or hide objects for `fx` (`FXManager::DoTargettedFX`'s node list) of
/// an emitter on `terrain` (`FXManager::m_terrain`).
pub fn fx_nodes(table: &Ac2VisualFxParticles, fx: u32, terrain: u32) -> Vec<Ac2FxObject> {
    table
        .get_on(fx, terrain)
        .map(|list| list.iter().map(|a| a.object).collect())
        .unwrap_or_default()
}

/// The nodes `FXManager::DoTargettedFX` goes on to act on: the ones that play for this emitter
/// (`m_player_only` only on the player) at this time of day (`m_day_time`, `m_night_time`).
/// A node whose channel keys mismatch the FX channel preferences does not play
/// (`FXManager::ChannelMissmatch`).
pub fn gated_nodes(nodes: &[Ac2FxObject], is_player: bool, night: bool) -> Vec<Ac2FxObject> {
    nodes
        .iter()
        .filter(|node| node.gate.plays(is_player, night))
        .copied()
        .collect()
}

/// `FXManager::DoTargettedFX`'s camera part for an FX that starts on an emitter: each node that
/// plays (no channel mismatch; `m_player_only` on the player;
/// its day or night flag for the time of day) and names a `CameraFX` plays it from the emitter
/// (`DoCameraFX`) at the FX's current scalar, which a ramping node starts at 0.
pub fn camera_fx_starts(
    nodes: &[Ac2FxObject],
    scalar: f32,
    is_player: bool,
    night: bool,
) -> Vec<(u32, f32)> {
    nodes
        .iter()
        .filter(|node| node.camera_fx != 0)
        .filter(|node| node.gate.plays(is_player, night))
        .map(|node| {
            let current = if node.ramp_time >= 0.0002 {
                0.0
            } else {
                scalar
            };
            (node.camera_fx, current)
        })
        .collect()
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
    /// The appearances the FX's nodes apply (`m_apr_id`, `m_apr_key`'s key), which `StopFX`
    /// removes from the emitter.
    pub appearances: Vec<(u32, u32)>,
    /// `m_current_scalar` and `m_desired_scalar`: `UseTime` ramps the one to the other.
    pub current: f32,
    pub desired: f32,
    /// `m_target_id`: the object the FX was started on (0 is `INVALID_IID`).
    pub target: u64,
    /// The FX's nodes as `UseTime` walks them: each one's `m_ramp_time`, and its appearance.
    ramps: Vec<RampNode>,
}

/// One FX node as `FXManager::UseTime` reads it.
#[derive(Debug, Clone, Copy, PartialEq)]
struct RampNode {
    ramp_time: f32,
    /// `m_apr_id`, `m_apr_key`'s key and scalar (`apr_id` 0 for none).
    appearance: (u32, u32, f32),
    /// `m_target_fx_id`: the FX the target plays, which `StopFX` stops on it.
    target_fx: u32,
}

impl RampNode {
    fn new(node: &Ac2FxObject) -> Self {
        Self {
            ramp_time: node.ramp_time,
            appearance: (
                node.appearance,
                node.appearance_key.0,
                node.appearance_key.1,
            ),
            target_fx: node.target_fx_id,
        }
    }

    /// `FXNode::m_ramp_time < 0.0002`: the node takes a new scalar at once.
    fn is_instant(&self) -> bool {
        self.ramp_time < MIN_RAMP_TIME
    }
}

/// The shortest ramp `FXManager` ramps over; a shorter one jumps to the new scalar.
const MIN_RAMP_TIME: f32 = 0.0002;

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

/// The target an FX starts with (`FXManager::DoTargettedFX`'s `_target_id`, and
/// `PHYSICSOBJREP::GetVDescToClone`, `GetClonedAppearanceID` and `GetClonedAppKeyHash` of the
/// emitter): the object the FX acts on and the description a retargeting node clones.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FxTarget {
    /// 0 is `INVALID_IID`.
    pub id: u64,
    /// `m_vdescToClone`: the visual a retargeting node makes a copy of (0 for none).
    pub vdesc_to_clone: u32,
    /// `m_clonedAprID` and `m_clonedAppAprHash` (key, scalar): the appearance the clone takes.
    pub apr: u32,
    pub apr_keys: Vec<(u32, f32)>,
}

/// An FX's scalar and target (`FXScalarAndTarget`, what `FXManager::GetCurrentFX` lists).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FxScalarAndTarget {
    pub scalar: f32,
    pub target: u64,
}

/// `FXManager::DoTargetAndCloningFXs`'s calls on the target the client carries out, in order.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TargetCall {
    /// `m_target_fx_id`: `CPhysicsObj::DoFX(fx, state.m_current_scalar)` on the object.
    DoFx { object: u64, fx: u32, scalar: f32 },
    /// `FXManager::StopFX`: the node's target FX stops on the FX's target (`CPhysicsObj::StopFX`).
    StopFx { object: u64, fx: u32 },
    /// `m_parent` with a holding location: `CPhysicsObj::FXSetParent` hangs the object from the
    /// emitter's holding point (`attach` is the node's place on the emitter).
    Parent {
        object: u64,
        emitter: u64,
        attach: Ac2FxObjectAttach,
    },
    /// `m_unparent`: the object leaves its parent where it stands.
    Unparent { object: u64 },
}

/// A slider effect a node asked of its emitter (`FXManager::DoSliderFX`): the emitter's
/// `DoSlider(0, slider, scalar, ramp_time)`, and with `propagate` the same on each direct child.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SliderMove {
    pub emitter: u64,
    pub slider: u32,
    pub scalar: f32,
    pub ramp_time: f32,
    pub propagate: bool,
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
    /// Each emitter's effects description (`CPhysicsObj::m_effects_vdesc`): the appearance keys
    /// its FX applied.
    appearances: HashMap<u64, ac2_physics::Appearance>,
    sliders: Vec<SliderMove>,
    /// `DoCreateObjectFX`'s `PHYSICSOBJREP::SetFXTarget(created id)`: the emitters' new FX
    /// targets, and the target calls, for the client to carry out.
    new_targets: Vec<(u64, u64)>,
    target_calls: Vec<TargetCall>,
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
    pub fn start(
        &mut self,
        emitter: u64,
        fx: u32,
        nodes: &[Ac2FxObject],
        now: f64,
        scalar: f32,
        script_period: &dyn Fn(u32) -> Option<f32>,
        ids: &mut dyn FnMut() -> Option<u64>,
    ) {
        self.start_targeted(
            emitter,
            fx,
            nodes,
            now,
            scalar,
            &FxTarget::default(),
            script_period,
            ids,
        );
    }

    /// [`Self::start`] with the FX's target (`DoTargettedFX`'s `_target_id`): its running
    /// state keeps it (`FXState::m_target_id`), and each node acts on it
    /// (`FXManager::DoTargetAndCloningFXs`).
    pub fn start_targeted(
        &mut self,
        emitter: u64,
        fx: u32,
        nodes: &[Ac2FxObject],
        now: f64,
        scalar: f32,
        fx_target: &FxTarget,
        script_period: &dyn Fn(u32) -> Option<f32>,
        ids: &mut dyn FnMut() -> Option<u64>,
    ) {
        let ramps: Vec<RampNode> = nodes.iter().map(RampNode::new).collect();
        if let Some(record) = self
            .records
            .get_mut(&emitter)
            .and_then(|fxs| fxs.get_mut(&fx))
        {
            // A running FX takes the new scalar as its desired one; an instant node sets the
            // current one too, and each appearance follows the current one (`DoAppearanceFX`).
            record.desired = scalar;
            record.target = fx_target.id;
            for node in &ramps {
                if node.is_instant() {
                    record.current = scalar;
                }
                appearance_fx(&mut self.appearances, emitter, node, record.current);
            }
            return;
        }
        if !nodes.iter().any(Ac2FxObject::is_active) {
            return;
        }
        // A new FX: each node starts its current scalar over, at 0 when it ramps, so a ramping
        // appearance starts removed and comes in through `use_time`.
        let mut current = 0.0;
        for node in &ramps {
            current = if node.is_instant() { scalar } else { 0.0 };
            appearance_fx(&mut self.appearances, emitter, node, current);
        }
        let mut record = FxRecord {
            appearances: appearance_names(&ramps),
            current,
            desired: scalar,
            target: fx_target.id,
            ramps,
            ..FxRecord::default()
        };
        let mut end = None;
        for node in nodes {
            // `FXManager::DoSliderFX(m_prop_color, m_slider, m_intensity, m_ramp_time)`.
            if node.slider.id != 0 {
                self.sliders.push(SliderMove {
                    emitter,
                    slider: node.slider.id,
                    scalar: node.slider.intensity,
                    ramp_time: node.ramp_time,
                    propagate: node.slider.propagate,
                });
            }
            if let Some(colour) = node.color {
                self.set_colour(emitter, colour, node.ramp_time, now);
                if node.color_lifetime != 0.0 {
                    record.restore_colour = true;
                }
            }
            let mut target = fx_target.id;
            // `CreateTemporaryObject` fails without a temporary id: no object. The object
            // becomes the FX's target and the emitter's (`SetFXTarget`).
            if let Some(id) = (node.vdesc != 0).then(&mut *ids).flatten() {
                self.new_object(id, emitter, node);
                record.objects.push(id);
                self.new_targets.push((emitter, id));
                target = id;
            }
            // `DoTargettedFX`'s duration, the last node's: `CalcDuration` (a looping system never
            // ends, nor does a looping sound, else the FX ends at once), the object's
            // lifetime, a target FX or an appearance that is not removed (never), a removing
            // appearance key (0.1 s), the FX script's period, and a restoring colour's lifetime.
            let mut seconds = if node.particle_looping || node.sound_looping {
                -1.0
            } else {
                0.0
            };
            if (node.vdesc != 0 || node.retarget) && node.lifetime != 0.0 {
                seconds = node.lifetime;
            }
            if node.target_fx_id != 0 {
                seconds = -1.0;
            }
            if node.appearance != 0 {
                seconds = if node.appearance_key.1 < 0.0 {
                    0.1
                } else {
                    -1.0
                };
            }
            if node.fx_script != 0 {
                if let Some(period) = script_period(node.fx_script) {
                    seconds = period;
                }
            }
            if record.restore_colour && node.color_lifetime > 0.0 {
                seconds = node.color_lifetime;
            }
            end = (seconds >= 0.0).then(|| now + f64::from(seconds));
            if target != 0 || fx_target.vdesc_to_clone != 0 {
                self.target_and_cloning(
                    emitter,
                    &mut record,
                    target,
                    fx_target,
                    node,
                    now,
                    &mut *ids,
                );
            }
        }
        record.end = end;
        self.records.entry(emitter).or_default().insert(fx, record);
    }

    /// `FXManager::DoTargetAndCloningFXs` for one node: the target hides (and shows again when
    /// the FX stops); a retargeting node without `m_conditional` acts on a clone of the
    /// emitter's `m_vdescToClone` instead (made with the cloned appearance, kept on the FX,
    /// placed only by `m_parent` at the holding point); then the object takes the node's
    /// target FX, parents to the holding point, unparents, and is destroyed when asked.
    fn target_and_cloning(
        &mut self,
        emitter: u64,
        record: &mut FxRecord,
        target: u64,
        fx_target: &FxTarget,
        node: &Ac2FxObject,
        now: f64,
        ids: &mut dyn FnMut() -> Option<u64>,
    ) {
        if node.hide_target && target != 0 {
            self.hidden.insert(target);
            record.unhide.push(target);
        }
        let mut object = target;
        let mut cloned = false;
        if node.retarget && !node.conditional {
            let Some(id) = (fx_target.vdesc_to_clone != 0).then(ids).flatten() else {
                return;
            };
            // The clone is placed only by a parent node: otherwise it has no place.
            let parented = node.holding_location != 0 && node.parent;
            self.pending.push(PendingObject {
                id,
                emitter,
                visual: fx_target.vdesc_to_clone,
                attach: if parented {
                    node.attach
                } else {
                    Ac2FxObjectAttach::Unplaced
                },
                ramp_time: 0.0,
            });
            if fx_target.apr != 0 {
                for &(key, scalar) in &fx_target.apr_keys {
                    self.appearances
                        .entry(id)
                        .or_default()
                        .add_appearance_mod(fx_target.apr, (key, scalar));
                }
            }
            record.objects.push(id);
            self.new_targets.push((emitter, id));
            object = id;
            cloned = true;
        }
        if object == 0 {
            return;
        }
        if node.holding_location != 0 && node.parent && !cloned {
            self.target_calls.push(TargetCall::Parent {
                object,
                emitter,
                attach: node.attach,
            });
        }
        if node.target_fx_id != 0 {
            self.target_calls.push(TargetCall::DoFx {
                object,
                fx: node.target_fx_id,
                scalar: record.current,
            });
        }
        if node.unparent {
            self.target_calls.push(TargetCall::Unparent { object });
        }
        if node.destroy_target && crate::temporary_ids::is_temporary(object) {
            self.fade_and_destroy(object, now);
        }
    }

    /// `FXManager::DoStartupFX`'s object part, run when the object is created: each node that
    /// creates an object does so (`DoCreateObjectFX`), and the FX is recorded as a startup one,
    /// which `StopFX` leaves running. Colour and hides are not part of the startup branch.
    pub fn start_startup(
        &mut self,
        emitter: u64,
        fx: u32,
        nodes: &[Ac2FxObject],
        ids: &mut dyn FnMut() -> Option<u64>,
    ) {
        if self.record(emitter, fx).is_some() {
            return;
        }
        // `FXManager::DoStartupFX` applies the appearances at full scalar.
        let ramps: Vec<RampNode> = nodes.iter().map(RampNode::new).collect();
        for node in &ramps {
            appearance_fx(&mut self.appearances, emitter, node, 1.0);
        }
        let mut record = FxRecord {
            startup: true,
            appearances: appearance_names(&ramps),
            current: 1.0,
            desired: 1.0,
            ramps,
            ..FxRecord::default()
        };
        for node in nodes.iter().filter(|node| node.vdesc != 0) {
            let Some(id) = ids() else {
                continue;
            };
            self.new_object(id, emitter, node);
            record.objects.push(id);
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
        self.appearances.remove(&emitter);
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
        // `StopFX`: each node's target FX stops on the FX's target.
        if record.target != 0 {
            for node in record.ramps.iter().filter(|node| node.target_fx != 0) {
                self.target_calls.push(TargetCall::StopFx {
                    object: record.target,
                    fx: node.target_fx,
                });
            }
        }
        // `StopFX`: `PHYSICSOBJREP::RemoveAppearanceFX` for each node's appearance.
        for &(table, key) in &record.appearances {
            self.remove_appearance(emitter, table, key);
        }
    }

    /// `FXManager::UseTime`'s ramp: each running FX whose current scalar is not its desired one
    /// steps toward it once per node (`quantum / m_ramp_time`, or at once for an instant node),
    /// and each node's appearance is applied again at the new current scalar.
    pub fn use_time(&mut self, quantum: f32) {
        for (&emitter, fxs) in &mut self.records {
            for record in fxs.values_mut() {
                if record.current == record.desired {
                    continue;
                }
                for node in &record.ramps {
                    record.current = if node.is_instant() {
                        record.desired
                    } else {
                        let step = quantum / node.ramp_time;
                        if record.desired <= record.current {
                            (record.current - step).max(record.desired)
                        } else {
                            (record.current + step).min(record.desired)
                        }
                    };
                    appearance_fx(&mut self.appearances, emitter, node, record.current);
                }
            }
        }
    }

    fn remove_appearance(&mut self, emitter: u64, table: u32, key: u32) {
        remove_appearance(&mut self.appearances, emitter, table, key);
    }

    /// The emitter's effects description (`m_effects_vdesc`), while its FX apply any appearance:
    /// the object is drawn with it merged onto its local description (`ApplyAdditiveVDesc`).
    pub fn appearance_effects(&self, emitter: u64) -> Option<&ac2_physics::Appearance> {
        self.appearances.get(&emitter)
    }

    fn take_record(&mut self, emitter: u64, fx: u32) -> Option<FxRecord> {
        let fxs = self.records.get_mut(&emitter)?;
        let record = fxs.remove(&fx);
        if fxs.is_empty() {
            self.records.remove(&emitter);
        }
        record
    }

    /// `FXManager::DoCreateObjectFX`'s `CreateTemporaryObject`: the object `id` waits to be made
    /// at its emitter.
    fn new_object(&mut self, id: u64, emitter: u64, node: &Ac2FxObject) {
        self.pending.push(PendingObject {
            id,
            emitter,
            visual: node.vdesc,
            attach: node.attach,
            ramp_time: node.ramp_time,
        });
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

    /// `FXManager::GetCurrentFX`: the FX running on `emitter` that are not startup FX
    /// (`m_bStartupFX` clear) with each one's current scalar and target, in id order
    /// (`map<unsigned long, FXScalarAndTarget>`). Only FX with object facts have a record here.
    pub fn current_fx(&self, emitter: u64) -> Vec<(u32, FxScalarAndTarget)> {
        let mut list: Vec<(u32, FxScalarAndTarget)> = self
            .records
            .get(&emitter)
            .into_iter()
            .flatten()
            .filter(|(_, record)| !record.startup)
            .map(|(&fx, record)| {
                (
                    fx,
                    FxScalarAndTarget {
                        scalar: record.current,
                        target: record.target,
                    },
                )
            })
            .collect();
        list.sort_unstable_by_key(|(fx, _)| *fx);
        list
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

    /// Takes the slider effects FX asked for since the last call (`DoSliderFX`), in order.
    pub fn take_sliders(&mut self) -> Vec<SliderMove> {
        std::mem::take(&mut self.sliders)
    }

    /// Takes the emitters' new FX targets (`PHYSICSOBJREP::SetFXTarget`) since the last call.
    pub fn take_new_targets(&mut self) -> Vec<(u64, u64)> {
        std::mem::take(&mut self.new_targets)
    }

    /// Takes the calls on targets FX asked for since the last call, in order.
    pub fn take_target_calls(&mut self) -> Vec<TargetCall> {
        std::mem::take(&mut self.target_calls)
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
            self.appearances.remove(id);
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

/// The appearances the nodes name (`m_apr_id`, `m_apr_key`'s key), which `StopFX` removes.
fn appearance_names(nodes: &[RampNode]) -> Vec<(u32, u32)> {
    nodes
        .iter()
        .filter(|node| node.appearance.0 != 0)
        .map(|node| (node.appearance.0, node.appearance.1))
        .collect()
}

/// `FXManager::DoAppearanceFX` for one node: its appearance applied at the key's scalar times
/// `scalar` while that is positive (`CPhysicsObj::DoAppearanceFX` on the emitter's effects
/// description), removed otherwise.
fn appearance_fx(
    appearances: &mut HashMap<u64, ac2_physics::Appearance>,
    emitter: u64,
    node: &RampNode,
    scalar: f32,
) {
    let (table, key, key_scalar) = node.appearance;
    match crate::fx_manager::appearance_fx_step(table, key_scalar, scalar) {
        crate::fx_manager::AppearanceFxStep::Apply { scalar } => {
            appearances
                .entry(emitter)
                .or_default()
                .add_appearance_mod(table, (key, scalar));
        }
        crate::fx_manager::AppearanceFxStep::Remove => {
            remove_appearance(appearances, emitter, table, key);
        }
        crate::fx_manager::AppearanceFxStep::None => {}
    }
}

/// `CPhysicsObj::RemoveAppearanceFX`: the key leaves the emitter's effects description.
fn remove_appearance(
    appearances: &mut HashMap<u64, ac2_physics::Appearance>,
    emitter: u64,
    table: u32,
    key: u32,
) {
    if let Some(effects) = appearances.get_mut(&emitter) {
        effects.remove_appearance_mod(table, key);
        if effects.is_empty() {
            appearances.remove(&emitter);
        }
    }
}

#[cfg(test)]
#[path = "fx_objects_tests.rs"]
mod tests;
