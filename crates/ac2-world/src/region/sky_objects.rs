//! `CDayDesc::GetSky`: where each of a day's `SkyObject`s sits at a time of day. Objects outside
//! their time window draw nothing unless the current `SkyTimeOfDay` replaces their graphic; the
//! replacement also offsets heading and pitch, and its colour is interpolated (HSB) towards the
//! next key's replacement.

use std::sync::Arc;

use ac2_core::DataId;

use super::day_desc::{after_time, before_time, interp_color, time_ratio};
use super::SkyTimeOfDay;
use crate::color::{Rgba, WHITE};

/// One sprite of a lens flare (`LensFlarePoint`): where it sits on the axis from the light
/// through the screen centre (-1 on the light, 1 mirrored across the centre), its
/// `RenderTexture`, and its size as a fraction of the screen's width.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LensFlarePoint {
    pub axis_pos: f32,
    pub texture: Option<DataId>,
    pub size: f32,
}

/// `LensFlareEffect`: the flare a sky object draws over the world (`LScapeSky::Draw(1)`), fading
/// in and out at `fade_speed` per second as the light is seen or hidden.
#[derive(Debug, Clone, PartialEq)]
pub struct LensFlare {
    pub fade_speed: f32,
    pub points: Arc<[LensFlarePoint]>,
}

/// `SkyObject` (`SkyObject::StreamPack` order).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SkyObject {
    pub(crate) name: u32,
    pub(crate) default_gfx: u32,
    pub(crate) begin_time: f32,
    pub(crate) end_time: f32,
    pub(crate) begin_pitch: f32,
    pub(crate) end_pitch: f32,
    pub(crate) begin_heading: f32,
    pub(crate) end_heading: f32,
    pub(crate) height_mod: f32,
    pub(crate) after: bool,
    pub(crate) fixed_pos: bool,
    pub(crate) water_effect: u32,
    pub(crate) fixed_x: f32,
    pub(crate) fixed_y: f32,
    pub(crate) west_to_east: bool,
    /// Its `LensFlareEffect`, when it has points.
    pub(crate) lens_flare: Option<LensFlare>,
}

/// `SkyObjectReplace` of one `SkyTimeOfDay`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct SkyObjectReplace {
    pub(crate) name: u32,
    pub(crate) replace_gfx: u32,
    pub(crate) pitch_offset: f32,
    pub(crate) heading_offset: f32,
    pub(crate) color: Rgba,
}

/// `CelestialPosition`: one sky object as `LScapeSky` places and tints it.
#[derive(Debug, Clone, PartialEq)]
pub struct CelestialPosition {
    /// The `CSetup`/visual to draw; `None` when the object is not up.
    pub gfx: Option<DataId>,
    /// Degrees.
    pub heading: f32,
    /// Degrees.
    pub pitch: f32,
    /// RGBA tint (`SetDiffuse` colour and opacity).
    pub color: [f32; 4],
    /// Drawn after the world (`LScapeSky::Draw(1, …)`) rather than before it.
    pub after: bool,
    /// 0: always; otherwise drawn only when the viewer's in-water state equals it.
    pub water: u32,
    pub height_mod: f32,
    pub fixed_pos: bool,
    pub fixed_x: i32,
    pub fixed_y: i32,
    /// The flare it draws after the world (`m_bDrawLensFlareEffect`: its object's effect has
    /// points).
    pub lens_flare: Option<LensFlare>,
}

/// `CDayDesc::GetSky` at `time` (fraction of the day).
pub(crate) fn get_sky(
    objects: &[SkyObject],
    keys: &[SkyTimeOfDay],
    time: f32,
) -> Vec<CelestialPosition> {
    objects
        .iter()
        .map(|object| {
            let (begin, end) = (object.begin_time, object.end_time);
            let up = begin == end
                || (begin <= time && time < end)
                || (end < begin && (time < end || begin < time));
            let mut position = CelestialPosition {
                gfx: None,
                heading: 0.0,
                pitch: 0.0,
                color: WHITE,
                after: false,
                water: 0,
                height_mod: 0.0,
                fixed_pos: false,
                fixed_x: 0,
                fixed_y: 0,
                lens_flare: object.lens_flare.clone(),
            };
            if up {
                position.gfx = gfx(object.default_gfx);
                position.after = object.after;
                position.water = object.water_effect;
                position.fixed_pos = object.fixed_pos;
                position.fixed_x = object.fixed_x as i32;
                position.fixed_y = object.fixed_y as i32;
                position.height_mod = object.height_mod;
                if begin == end {
                    position.heading = object.begin_heading;
                    position.pitch = object.begin_pitch;
                } else {
                    let ratio = time_ratio(begin, end, time);
                    position.heading = sweep(
                        object.begin_heading,
                        object.end_heading,
                        ratio,
                        object.west_to_east,
                    );
                    position.pitch = sweep(
                        object.begin_pitch,
                        object.end_pitch,
                        ratio,
                        object.west_to_east,
                    );
                }
            }
            if let Some(before) = before_time(keys, time) {
                if let Some(replace) = find_replacement(before, object.name) {
                    if replace.replace_gfx != 0 {
                        position.gfx = gfx(replace.replace_gfx);
                    }
                    position.heading += replace.heading_offset;
                    position.pitch += replace.pitch_offset;
                    let next = after_time(keys, time)
                        .and_then(|after| Some((after, find_replacement(after, object.name)?)));
                    if let Some((after, next)) = next {
                        let ratio = time_ratio(before.begin, after.begin, time);
                        let rgb = interp_color(replace.color, next.color, ratio);
                        let a = (next.color[3] - replace.color[3]) * ratio + replace.color[3];
                        position.color = [rgb[0], rgb[1], rgb[2], a];
                    }
                }
            }
            position
        })
        .collect()
}

/// Heading/pitch sweep: east-to-west objects go down through 360 when the end is past the start
/// the other way; west-to-east ones the opposite way. Negative results wrap up by 360.
fn sweep(begin: f32, end: f32, ratio: f32, west_to_east: bool) -> f32 {
    let (mut begin, mut end) = (begin, end);
    if !west_to_east {
        if end < begin {
            begin -= 360.0;
        }
    } else if begin < end {
        end -= 360.0;
    }
    let angle = (end - begin) * ratio + begin;
    if angle < 0.0 {
        angle + 360.0
    } else {
        angle
    }
}

fn find_replacement(key: &SkyTimeOfDay, name: u32) -> Option<&SkyObjectReplace> {
    key.replacements.iter().find(|replace| replace.name == name)
}

fn gfx(did: u32) -> Option<DataId> {
    (did != 0).then(|| DataId::new(did))
}

#[cfg(test)]
mod tests {
    use super::{get_sky, sweep, SkyObject};

    fn sun() -> SkyObject {
        SkyObject {
            name: 1,
            default_gfx: 0x0400_0001,
            begin_time: 0.25,
            end_time: 0.75,
            begin_pitch: 0.0,
            end_pitch: 180.0,
            begin_heading: 90.0,
            end_heading: 90.0,
            height_mod: 0.0,
            after: false,
            fixed_pos: false,
            water_effect: 0,
            fixed_x: 0.0,
            fixed_y: 0.0,
            west_to_east: false,
            lens_flare: None,
        }
    }

    #[test]
    fn objects_sweep_through_their_window_and_set_outside_it() {
        let noon = get_sky(&[sun()], &[], 0.5);
        assert_eq!(noon[0].gfx.map(|d| d.raw()), Some(0x0400_0001));
        assert!((noon[0].pitch - 90.0).abs() < 1e-4);
        assert_eq!(get_sky(&[sun()], &[], 0.9)[0].gfx, None);
    }

    #[test]
    fn sweeps_wrap_by_direction() {
        assert_eq!(sweep(350.0, 10.0, 0.5, false), 0.0);
        assert_eq!(sweep(10.0, 350.0, 0.5, true), 0.0);
        assert_eq!(sweep(10.0, 350.0, 0.5, false), 180.0);
    }
}
