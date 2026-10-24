//! `CLandBlock::GenerateSceneObjectList` with `ProcScene`, `ProcGroup`,
//! `CheckGroupPlacement`, `SceneGroup::{GetScale,UpdateXYLoc,FinishPlaceAndCheck}`,
//! `CheckRadius` and `ObjectCollides` (Legions 7160).

use super::clearance::Clearance;
use super::format::{Ac2SceneEntity, Ac2SceneGroup};
use super::geometry::{
    circle_in_triangle, cross, downhill_heading, heading_of, normalize, plane_z, q_from_axes,
    q_heading, qmul, qrot, Ac2SceneTerrain, BLOCK_LENGTH, EPSILON, IDENTITY, POLYGON, Q, SIDE, V3,
};
use super::hash::{check_freq, prand2};
use super::Ac2SceneLookup;

const SCENE_SEED: u32 = 0x0009_23B1;
const ENTITY_SEED_STEP: u32 = 0x2D41_BF19;
const FRILL_SEED: u32 = 0x0380_4309;

/// A generated placement before conversion to the public single-precision shape.
#[derive(Debug, Clone, Copy)]
pub(super) struct RawPlacement {
    pub(super) object: u32,
    pub(super) origin: V3,
    pub(super) rotation: Q,
    pub(super) scale: V3,
    pub(super) frill: bool,
}

#[derive(Clone, Copy)]
struct Pose {
    origin: V3,
    rotation: Q,
}

pub(super) struct Generator<'a, L: Ac2SceneLookup> {
    terrain: &'a Ac2SceneTerrain,
    lookup: &'a L,
    water: Option<f64>,
    clearances: Vec<Clearance>,
    pub(super) placements: Vec<RawPlacement>,
}

impl<'a, L: Ac2SceneLookup> Generator<'a, L> {
    /// `clearances` seeds the clearance list with the block's `CLandBlockInfo` entities.
    pub(super) fn new(
        terrain: &'a Ac2SceneTerrain,
        lookup: &'a L,
        water: Option<f64>,
        clearances: Vec<Clearance>,
    ) -> Self {
        Self {
            terrain,
            lookup,
            water,
            clearances,
            placements: Vec::new(),
        }
    }

    /// `GenerateSceneObjectList`: one scene per landscape vertex whose cell info selects one.
    pub(super) fn generate(&mut self, scene_at: impl Fn(usize, usize) -> u32) {
        let spacing = f64::from((POLYGON - EPSILON) as f32);
        for ix in 0..SIDE {
            for iy in 0..SIDE {
                let scene = scene_at(ix, iy);
                if scene == 0 {
                    continue;
                }
                // `GenerateSceneObjectList` stores the vertex offset into a float `Position`.
                let pose = Pose {
                    origin: store3([
                        ix as f64 * spacing,
                        iy as f64 * spacing,
                        self.terrain.height(ix, iy),
                    ]),
                    rotation: IDENTITY,
                };
                let x = self.terrain.block_x * 16 + ix as u32;
                let y = self.terrain.block_y * 16 + iy as u32;
                self.proc_scene(x, y, scene, pose, [1.0; 3], SCENE_SEED, false, None);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn proc_scene(
        &mut self,
        x: u32,
        y: u32,
        scene: u32,
        pose: Pose,
        scale: V3,
        seed: u32,
        frill: bool,
        mut count: Option<&mut u32>,
    ) -> bool {
        let Some(file) = self.lookup.scene_file(scene) else {
            return true;
        };
        let seed = scene.wrapping_mul(seed).wrapping_mul(SCENE_SEED);
        // `GetCorrectDegrade(0)`: the nearest degrade.
        if let Some(objects) = file.degrades.first() {
            let start = (!frill).then(|| self.push_group_clearance(pose, 0.0, false));
            for (index, entity) in objects.iter().enumerate() {
                let child_seed = seed.wrapping_add((index as u32).wrapping_mul(ENTITY_SEED_STEP));
                match entity {
                    Ac2SceneEntity::Group(group) => {
                        if !self.proc_group(x, y, group, pose, scale, child_seed, frill) {
                            return false;
                        }
                    }
                    Ac2SceneEntity::Scene(nested) => {
                        if !self.proc_scene(x, y, *nested, pose, scale, child_seed, frill, None) {
                            return false;
                        }
                    }
                    Ac2SceneEntity::Object(object) => {
                        self.emit(*object, pose, scale, frill);
                        if let Some(count) = count.as_deref_mut() {
                            *count += 1;
                        }
                    }
                }
            }
            if let Some(start) = start {
                self.clearances[start].group_done = true;
            }
        }
        if let Some(frill_group) = &file.frill {
            self.proc_group(
                x,
                y,
                frill_group,
                pose,
                scale,
                seed.wrapping_add(FRILL_SEED),
                true,
            );
        }
        true
    }

    #[allow(clippy::too_many_arguments)]
    fn proc_group(
        &mut self,
        x: u32,
        y: u32,
        g: &Ac2SceneGroup,
        base: Pose,
        parent_scale: V3,
        seed: u32,
        frill: bool,
    ) -> bool {
        if !check_freq(g.freq, x, y, seed) {
            return true;
        }
        // `SceneGroup::GetScale` writes the float `Vector new_scale`.
        let Some(mut scale) = get_scale(g, x, y, parent_scale, seed).map(store3) else {
            return false;
        };
        let mut pose = base;
        if !self.place(x, y, g, &mut pose, parent_scale, &mut scale, seed) {
            return true;
        }
        let start =
            (!frill).then(|| self.push_group_clearance(pose, clearance_radius(g, scale), g.use_3d));
        let mut count = 0_u32;
        for (index, entity) in g.subs.iter().enumerate() {
            let child_seed = seed.wrapping_add((index as u32).wrapping_mul(ENTITY_SEED_STEP));
            match entity {
                Ac2SceneEntity::Group(group) => {
                    if !self.proc_group(x, y, group, pose, scale, child_seed, frill) {
                        return false;
                    }
                }
                Ac2SceneEntity::Scene(nested) => {
                    if !self.proc_scene(
                        x,
                        y,
                        *nested,
                        pose,
                        scale,
                        child_seed,
                        frill,
                        Some(&mut count),
                    ) {
                        return false;
                    }
                }
                Ac2SceneEntity::Object(object) => {
                    self.emit(*object, pose, scale, frill);
                    count += 1;
                }
            }
            if g.max_objects != 0 && count >= g.max_objects {
                break;
            }
        }
        if let Some(start) = start {
            self.clearances[start].group_done = true;
        }
        true
    }

    /// `CLandBlock::StartSceneGroup`: an open group clearance, closed by `ObjectClearance::SetEnd`.
    fn push_group_clearance(&mut self, pose: Pose, radius: f64, is_3d: bool) -> usize {
        self.clearances
            .push(Clearance::group(pose.origin, pose.rotation, radius, is_3d));
        self.clearances.len() - 1
    }

    fn emit(&mut self, object: u32, pose: Pose, scale: V3, frill: bool) {
        self.placements.push(RawPlacement {
            object,
            origin: pose.origin,
            rotation: pose.rotation,
            scale,
            frill,
        });
        // `CLandBlock::ProcEntity`: scene objects clear their origin; frills go to `ProcFrill`.
        if !frill {
            self.clearances.push(Clearance::entity(
                pose.origin,
                pose.rotation,
                (Vec::new(), Vec::new()),
            ));
        }
    }

    /// `CheckGroupPlacement` + `SceneGroup::FinishPlaceAndCheck` for `PLACE_RANDOM`.
    #[allow(clippy::too_many_arguments)]
    fn place(
        &self,
        x: u32,
        y: u32,
        g: &Ac2SceneGroup,
        pose: &mut Pose,
        parent_scale: V3,
        scale: &mut V3,
        n: u32,
    ) -> bool {
        // `UpdateXYLoc`.
        let loc_scale = if g.scale_loc_sub {
            parent_scale
        } else {
            [1.0; 3]
        };
        let rx = prand2(x, y, n.wrapping_add(0x28D0_642D));
        let ry = prand2(x, y, n.wrapping_add(0x062A_DE4F));
        let rz = prand2(x, y, n.wrapping_add(0x0071_D36D));
        let (mn, mx) = (g.min_loc, g.max_loc);
        let mut loc = [
            ((mx[0] - mn[0]) * rx + mn[0]) * loc_scale[0],
            ((mx[1] - mn[1]) * ry + mn[1]) * loc_scale[1],
            0.0,
        ];
        if g.height_relative == 9 {
            loc[2] = ((mx[2] - mn[2]) * rz + mn[2]) * loc_scale[2];
        }
        let offset = if g.rot_sub_loc {
            qrot(pose.rotation, loc)
        } else {
            loc
        };
        // `UpdateXYLoc` stores the moved origin back into the float frame.
        pose.origin = store3([
            pose.origin[0] + offset[0],
            pose.origin[1] + offset[1],
            pose.origin[2] + offset[2],
        ]);
        let radius = clearance_radius(g, *scale);
        let xy_loc = pose.origin;
        let [x0, y0, _] = xy_loc;
        if !(radius <= x0
            && radius <= y0
            && x0 < BLOCK_LENGTH - radius
            && y0 < BLOCK_LENGTH - radius)
        {
            return false;
        }
        if !g.no_legal_check && self.terrain.on_road(x0, y0) {
            return false;
        }
        let plane = self.terrain.plane(x0, y0);
        let normal = plane.0;
        if !(normal[2] <= g.min_slope && normal[2] >= g.max_slope) {
            return false;
        }
        // `UpdateFrame`.
        let rr = rx;
        let mut rotation = (g.max_rot - g.min_rot) * rr + g.min_rot;
        if !g.orient && !g.align {
            pose.rotation = if g.rot_sub_orient {
                qmul(pose.rotation, q_heading(rotation))
            } else {
                q_heading(rotation)
            };
        } else {
            if g.orient {
                rotation = downhill_heading(normal) + rotation + 180.0;
            }
            pose.rotation = if g.align {
                let forward = if normal[2].abs() > EPSILON {
                    normalize([0.0, normal[2], -normal[1]])
                } else {
                    [0.0, 0.0, 1.0]
                };
                qmul(
                    q_from_axes(cross(forward, normal), forward, normal),
                    q_heading(rotation),
                )
            } else {
                q_heading(rotation)
            };
        }
        // `Frame::operator=` stores the new rotation as floats.
        pose.rotation = pose.rotation.map(store);
        if g.height_relative != 9 {
            let base = match g.height_relative {
                0 => plane_z(plane, pose.origin[0], pose.origin[1]).unwrap_or(pose.origin[2]),
                1 => match self.water {
                    Some(water) => water,
                    None => return false,
                },
                _ => 0.0,
            };
            let lift = ((mx[2] - mn[2]) * rr + mn[2])
                * if g.scale_loc_sub {
                    parent_scale[2]
                } else {
                    1.0
                };
            // `UpdateZLoc` then `FinishPlaceAndCheck` store `loc` back into the float frame.
            pose.origin[2] = store(base + lift);
        }
        match g.scale_type {
            // `SceneGroup::ScaleByWaterDepth`: depth below the water over `scale_depth`, into z for
            // the height types and every axis for `DEPTH_SCALE_SIZE`; dry land rejects the group.
            5..=7 => {
                let Some(water) = self.water else {
                    return false;
                };
                let land = plane_z(plane, pose.origin[0], pose.origin[1]).unwrap_or(pose.origin[2]);
                let depth = water - land;
                if !(depth > EPSILON) {
                    return false;
                }
                let factor = depth / g.scale_depth;
                if g.scale_type == 7 {
                    *scale = [store(factor); 3];
                } else {
                    scale[2] = store(factor);
                }
            }
            // `SceneGroup::ScaleByLandSlope`.
            8 | 9 => {
                let (nx, ny) = (normal[0].abs(), normal[1].abs());
                if !(nx < 0.9998 && ny < 0.9998) {
                    return false;
                }
                scale[0] = store(1.0 / (1.0 - nx * nx).sqrt());
                scale[1] = store(1.0 / (1.0 - ny * ny).sqrt());
                if g.scale_type == 9 {
                    scale[2] = store((scale[0] + scale[1]) * 0.5);
                }
            }
            _ => {}
        }
        if !self.final_height(g, pose.origin, plane) {
            return false;
        }
        if g.check_slope_radius > 0.0 && !self.check_radius(g, pose, *scale) {
            return false;
        }
        g.no_legal_check || !self.collides(xy_loc, radius, g.use_3d)
    }

    /// `SceneGroup::CheckFinalHeight`: the above-dry / below-ground / water bands.
    fn final_height(&self, g: &Ac2SceneGroup, origin: V3, plane: (V3, f64)) -> bool {
        let land = plane_z(plane, origin[0], origin[1]).unwrap_or(origin[2]);
        let depth = self.water.map_or(0.0, |water| water - land);
        if depth <= EPSILON {
            let above = origin[2] - land;
            if above < -EPSILON {
                return band(
                    g.below_ground_type,
                    g.min_below_ground,
                    g.max_below_ground,
                    -above,
                );
            }
            return band(g.above_dry_type, g.min_above_dry, g.max_above_dry, above);
        }
        if g.water_depth_type <= 2
            && !band(
                g.water_depth_type,
                g.min_water_depth,
                g.max_water_depth,
                depth,
            )
        {
            return false;
        }
        let above = origin[2] - self.water.unwrap_or(0.0);
        if above < -EPSILON {
            return band(
                g.below_water_type,
                g.min_below_water,
                g.max_below_water,
                -above,
            );
        }
        band(
            g.above_water_type,
            g.min_above_water,
            g.max_above_water,
            above,
        )
    }

    /// `CLandBlock::CheckRadius`: every terrain triangle within the radius must have an
    /// acceptable slope and, for aligned groups, a downhill heading near the object's heading.
    fn check_radius(&self, g: &Ac2SceneGroup, pose: &Pose, scale: V3) -> bool {
        let r = if g.scale_check_radius {
            scale[0].max(scale[1])
        } else {
            1.0
        } * g.check_slope_radius;
        let [x, y, _] = pose.origin;
        let inside = |v: f64| (0.0..BLOCK_LENGTH).contains(&v);
        if !(inside(x - r) && inside(y - r) && inside(x + r) && inside(y + r)) {
            return false;
        }
        let (mut low, mut high) = (0.0, 360.0);
        if g.align && g.align_max_heading < 180.0 {
            let heading = heading_of(pose.rotation);
            low = heading - g.align_max_heading;
            if low < 0.0 {
                low += 360.0;
            }
            high = heading + g.align_max_heading;
        }
        for px in 0..16 {
            for py in 0..16 {
                let (cx, cy) = (px as f64 * POLYGON + 5.0, py as f64 * POLYGON + 5.0);
                if (cx - x).abs() > 5.0 + r || (cy - y).abs() > 5.0 + r {
                    continue;
                }
                for (normal, corners) in self.terrain.polygon_triangles(px, py) {
                    let ok = g.max_slope_in_radius - EPSILON < normal[2]
                        && normal[2] < g.min_slope_in_radius + EPSILON;
                    if ok && !g.align {
                        continue;
                    }
                    if !circle_in_triangle(&corners, x, y, r) {
                        continue;
                    }
                    if !ok {
                        return false;
                    }
                    let heading = downhill_heading(normal);
                    if high <= low {
                        if low <= heading {
                            continue;
                        }
                    } else if heading < low {
                        return false;
                    }
                    if high < heading {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// `CLandBlock::ObjectCollides`.
    fn collides(&self, loc: V3, radius: f64, is_3d: bool) -> bool {
        self.clearances.iter().any(|c| c.blocks(loc, radius, is_3d))
    }
}

/// `SceneGroup` height band test: type 0 rejects, 1 needs `min <= v`, 2 needs
/// `min <= v <= max`, any other type accepts.
fn band(kind: u32, min: f64, max: f64, value: f64) -> bool {
    match kind {
        0 => false,
        2 if max < value => false,
        1 | 2 => min <= value,
        _ => true,
    }
}

/// `SceneGroup::GetScale`.
fn get_scale(g: &Ac2SceneGroup, x: u32, y: u32, parent: V3, n: u32) -> Option<V3> {
    let sc = if g.scale_size_sub { parent } else { [1.0; 3] };
    let (mn, mx) = (g.min_scale, g.max_scale);
    let same = (0..3).all(|i| (mn[i] - mx[i]).abs() < EPSILON);
    if same || matches!(g.scale_type, 7 | 8) {
        return Some([sc[0] * mn[0], sc[1] * mn[1], sc[2] * mn[2]]);
    }
    let lerp = |i: usize, r: f64| (mx[i] - mn[i]) * r + mn[i];
    let ra = prand2(x, y, n.wrapping_add(0x0051_5E71));
    let rb = prand2(x, y, n.wrapping_add(0x0DE8_76C7));
    let rc = prand2(x, y, n.wrapping_add(0x17E0_784F));
    let [s0, s1, s2] = sc;
    Some(match g.scale_type {
        0 => [lerp(0, ra) * s0, lerp(1, rb) * s1, lerp(2, rc) * s2],
        1 => [lerp(0, ra) * s0, lerp(1, ra) * s1, lerp(2, rb) * s2],
        2 => [lerp(0, ra) * s0, lerp(1, rb) * s1, lerp(2, ra) * s2],
        3 => [lerp(0, ra) * s0, lerp(1, rb) * s1, lerp(2, rb) * s2],
        4 => [lerp(0, ra) * s0, lerp(1, ra) * s1, lerp(2, ra) * s2],
        5 => [lerp(0, ra) * s0, lerp(1, rb) * s1, s2],
        6 => [lerp(0, ra) * s0, lerp(1, ra) * s1, s2],
        _ => return None,
    })
}

/// `SceneGroup::GetClearanceRadius`.
fn clearance_radius(g: &Ac2SceneGroup, scale: V3) -> f64 {
    if !g.scale_clearance {
        return g.clearance_radius;
    }
    let factor = if g.use_3d {
        scale[0].max(scale[1]).max(scale[2])
    } else {
        scale[0].max(scale[1])
    };
    factor * g.clearance_radius
}

/// A value retail stores into a `float`: x87 intermediates are 53-bit, memory is f32.
fn store(v: f64) -> f64 {
    v as f32 as f64
}

fn store3(v: V3) -> V3 {
    v.map(store)
}
