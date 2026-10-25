//! `gPosition` and `gHeading` (`gPosition.cpp`, `gHeading.cpp`) over `Position`
//! (`geometry/Position.cpp`), `LandDefs` and `Heading` as ac2-physics ports them.
//!
//! x87 arithmetic is f64 here with f32 at retail's stores (53-bit precision control).

use ac2_physics::landdefs::{
    adjust_to_outside, get_block_offset, gid_to_lcoord, lcoord_to_gid, LAND_WIDTH, SQUARE_LENGTH,
};
use ac2_physics::{Frame, Heading, Position as PhysPosition, Quaternion, Vector};

use super::format::msvc_fixed;
use super::{narrow, word, Call, INT_RESULT};
use crate::native::{ntype, NativeObject, Position, StringInfo};
use crate::vm::{tag, Context, EFunFn, EFuns, InterpStatus, Vm};

/// `F_EPSILON` / the 0.0002 the geometry code compares with.
const EPSILON: f64 = 0.0002;
/// `LandDefs::cellid_mask`, `first_envcell_id`, `blockid_mask | block_cell_id`.
const CELLID_MASK: u32 = 0xFFFF;
const FIRST_ENVCELL_ID: u32 = 0x100;
/// `5 pi / 2`, the heading offset `Position::heading` subtracts from.
const FIVE_HALVES_PI: f64 = 7.853_981_633_974_483;

pub(super) fn register(efuns: &mut EFuns) {
    for (id, name) in super::efun_table() {
        let f = match name.split_once("::CallEFun_") {
            Some(("gPosition", m)) => position_method(m),
            Some(("gHeading", m)) => heading_method(m),
            _ => None,
        };
        if let Some(f) = f {
            efuns.register(id, f);
        }
    }
}

fn pos(vm: &Vm, r: u32) -> Option<Position> {
    match vm.native(r)? {
        NativeObject::Position(p) => Some(*p),
        _ => None,
    }
}

fn vec3(vm: &Vm, r: u32) -> Option<[f32; 3]> {
    match vm.native(r)? {
        NativeObject::Vector(v) => Some(*v),
        _ => None,
    }
}

fn phys(p: &Position) -> PhysPosition {
    let [w, x, y, z] = p.rotation;
    let [ox, oy, oz] = p.origin;
    PhysPosition::new(
        p.cell,
        Frame::new(Vector::new(ox, oy, oz), Quaternion { w, x, y, z }),
    )
}

fn arr(v: Vector) -> [f32; 3] {
    [v.x, v.y, v.z]
}

fn ld(v: f32) -> f64 {
    f64::from(v)
}

/// Runs `f` on `this` as a Position.
fn edit(vm: &mut Vm, ctx: &mut Context, f: impl FnOnce(&mut Position, &Vm)) -> InterpStatus {
    let this = Call::new(ctx).this();
    let Some(mut p) = pos(vm, this) else {
        return vm.bad_this(this);
    };
    f(&mut p, vm);
    if let Some(NativeObject::Position(slot)) = vm.native_mut(this) {
        *slot = p;
    }
    InterpStatus::Success
}

/// An int-tagged result in slot `n` from `this` (and the Position in word 2 when `pair`).
fn query(
    vm: &mut Vm,
    ctx: &mut Context,
    n: usize,
    pair: bool,
    f: fn(&Position, Option<&Position>, &Context) -> u32,
) -> InterpStatus {
    let call = Call::new(ctx);
    let (this, other) = (call.this(), call.word(2));
    let Some(a) = pos(vm, this) else {
        return vm.bad_this(this);
    };
    let b = if pair {
        match pos(vm, other) {
            Some(b) => Some(b),
            None => return vm.bad_this(other),
        }
    } else {
        None
    };
    let v = f(&a, b.as_ref(), ctx);
    Call::new(ctx).ret(n, v, INT_RESULT);
    InterpStatus::Success
}

fn new_native(
    vm: &mut Vm,
    ctx: &mut Context,
    n: usize,
    t: u32,
    native: NativeObject,
) -> InterpStatus {
    let r = vm.refs.create_native(t, native);
    Call::new(ctx).ret(n, r, tag::REF);
    InterpStatus::Success
}

/// `Position::pitch`: the angle above the horizontal toward `p`, degrees.
fn pitch(a: &Position, b: &Position) -> f32 {
    let o = phys(a).offset_to(&phys(b));
    let len = (ld(o.z) * ld(o.z) + ld(o.y) * ld(o.y) + ld(o.x) * ld(o.x)).sqrt() as f32;
    if EPSILON <= ld(len) {
        let z = ((1.0 / ld(len)) * ld(o.z)) as f32;
        (ld(z).asin() * 57.295_779_513_082_32) as f32
    } else {
        0.0
    }
}

/// `Position::determine_quadrant(height, p)`: where `p`'s origin is in this frame.
fn quadrant(a: &Position, height: f32, b: &Position) -> u32 {
    let hit = phys(a).local_to_local(&phys(b), Vector::ZERO);
    let mut q = if 0.0 <= hit.x { 0x10 } else { 0x08 };
    q |= if 0.0 <= hit.y { 0x20 } else { 0x40 };
    let h = ld(height);
    q | if ld(hit.z) < h * ld(0.333_333_34_f32) {
        0x04
    } else if ld(hit.z) < h * ld(0.666_666_7_f32) {
        0x02
    } else {
        0x01
    }
}

/// `Position::cylinder_distance(r1, h1, p1, r2, h2, p2)`.
fn cylinder_distance(r1: f32, h1: f32, p1: &Position, r2: f32, h2: f32, p2: &Position) -> f32 {
    let o = phys(p1).offset_to(&phys(p2));
    let mut flat = ((ld(o.x) * ld(o.x) + ld(o.y) * ld(o.y)).sqrt() - (ld(r1) + ld(r2))) as f32;
    if flat < 0.0 {
        flat = 0.0;
    }
    let (z1, z2) = (p1.origin[2], p2.origin[2]);
    let (top, other) = if z1 <= z2 {
        ((ld(z1) + ld(h1)) as f32, z2)
    } else {
        ((ld(z2) + ld(h2)) as f32, z1)
    };
    let mut up = (ld(other) - ld(top)) as f32;
    if up < 0.0 {
        up = 0.0;
    }
    (ld(flat) * ld(flat) + ld(up) * ld(up)).sqrt() as f32
}

/// `LandDefs::inbound_valid_cellid` and `AFrame::IsValid`.
fn is_valid(p: &Position) -> bool {
    let local = p.cell & CELLID_MASK;
    let id_ok = (1..=64).contains(&local) || (0x100..=0xFFFD).contains(&local) || local == 0xFFFF;
    let block = p.cell >> 16;
    let x = ((block & 0xFF00) >> 8) << 3;
    let y = (block & 0xFF) << 3;
    let inbound = id_ok && (x as i32) < LAND_WIDTH && (y as i32) < LAND_WIDTH;
    let finite = p.origin.iter().chain(&p.rotation).all(|v| !v.is_nan());
    let [w, qx, qy, qz] = p.rotation.map(ld);
    let norm = (w * w + qx * qx + qy * qy + qz * qz) as f32;
    inbound && finite && !norm.is_nan() && (ld(norm) - 1.0).abs() < EPSILON
}

/// `AFrame::operator==`: every origin and rotation component within 0.0002.
fn frames_equal(a: &Position, b: &Position) -> bool {
    a.origin
        .iter()
        .chain(&a.rotation)
        .zip(b.origin.iter().chain(&b.rotation))
        .all(|(x, y)| (ld(*x) - ld(*y)).abs() < EPSILON)
}

/// `LandDefs::GetCellDistanceSquared`: `land_length * square_length` off the map.
fn cell_distance_squared(c1: u32, c2: u32) -> f32 {
    match (gid_to_lcoord(c1), gid_to_lcoord(c2)) {
        (Some((x1, y1)), Some((x2, y2))) => {
            let d = ((x1 - x2) * (x1 - x2) + (y1 - y2) * (y1 - y2)) as f32;
            (ld(d) * ld(SQUARE_LENGTH) * ld(SQUARE_LENGTH)) as f32
        }
        _ => (f64::from(LAND_WIDTH) * ld(SQUARE_LENGTH)) as f32,
    }
}

/// `Position::FromCellCoords` / `FromGameMapCoords`: the land cell and the middle of it.
fn place_in_cell(p: &mut Position, cell: u32) -> bool {
    p.cell = cell;
    if cell == 0 {
        return false;
    }
    let local = (cell & CELLID_MASK) - 1;
    let half = ld(SQUARE_LENGTH) * 0.5;
    p.origin[0] = (f64::from(local >> 3) * ld(SQUARE_LENGTH) + half) as f32;
    p.origin[1] = (f64::from(local & 7) * ld(SQUARE_LENGTH) + half) as f32;
    true
}

/// `LandDefs::gid_to_game_map_coords`: percent of the half map from its middle (x, y).
fn game_map_coords(cell: u32) -> Option<(f32, f32)> {
    ac2_physics::landdefs::gid_to_game_map_coords(cell)
}

/// `LandDefs::game_map_coords_to_gid`.
fn game_map_coords_to_gid(x: f32, y: f32) -> u32 {
    let w = f64::from(LAND_WIDTH);
    let to_cell = |v: f32| (ld(v) * 0.01 * w * 0.5 + w * 0.5 + 0.5).floor() as i64 as i32;
    lcoord_to_gid(to_cell(x), to_cell(y))
}

/// `swscanf`'s `%f` / `%d` over a word stream (leading whitespace skipped).
fn scan_number(s: &[u8], at: &mut usize, float: bool) -> Option<f64> {
    while s.get(*at).is_some_and(|c| c.is_ascii_whitespace()) {
        *at += 1;
    }
    let start = *at;
    let mut end = *at;
    if matches!(s.get(end), Some(b'+' | b'-')) {
        end += 1;
    }
    let mut seen = false;
    while let Some(c) = s.get(end) {
        if c.is_ascii_digit() || (float && (*c == b'.' || *c == b'e' || *c == b'E')) {
            seen |= c.is_ascii_digit();
            end += 1;
        } else {
            break;
        }
    }
    if !seen {
        return None;
    }
    *at = end;
    std::str::from_utf8(&s[start..end]).ok()?.parse().ok()
}

fn expect(s: &[u8], at: &mut usize, lit: &str) -> bool {
    for c in lit.bytes() {
        if c == b' ' {
            while s.get(*at).is_some_and(|c| c.is_ascii_whitespace()) {
                *at += 1;
            }
        } else if s.get(*at) == Some(&c) {
            *at += 1;
        } else {
            return false;
        }
    }
    true
}

/// `Position::FromWPString`: `0x%08X [%f %f %f] %f %f %f %f` with a heading (5 fields) or a
/// quaternion (8).
fn from_wp_string(p: &mut Position, text: &[u8]) -> bool {
    let mut at = 0;
    if !expect(text, &mut at, " 0x") {
        return false;
    }
    let hex_end = text[at..]
        .iter()
        .position(|c| !c.is_ascii_hexdigit())
        .map_or(text.len(), |n| at + n);
    let Some(cell) = std::str::from_utf8(&text[at..hex_end])
        .ok()
        .and_then(|h| u32::from_str_radix(h, 16).ok())
    else {
        return false;
    };
    p.cell = cell;
    at = hex_end;
    let mut fields = 1;
    if expect(text, &mut at, " [") {
        for i in 0..3 {
            match scan_number(text, &mut at, true) {
                Some(v) => {
                    p.origin[i] = v as f32;
                    fields += 1;
                }
                None => return false,
            }
        }
    }
    if fields < 4 || !expect(text, &mut at, "]") {
        return false;
    }
    let mut rest = Vec::new();
    while rest.len() < 4 {
        match scan_number(text, &mut at, true) {
            Some(v) => rest.push(v as f32),
            None => break,
        }
    }
    match rest.len() {
        1 => {
            let mut frame = phys(p).frame;
            frame.set_heading(Heading::from_degrees(rest[0]));
            let q = frame.q;
            p.rotation = [q.w, q.x, q.y, q.z];
            true
        }
        4 => {
            let frame = Frame::new(
                Vector::ZERO,
                Quaternion {
                    w: rest[0],
                    x: rest[1],
                    y: rest[2],
                    z: rest[3],
                },
            );
            p.rotation = [frame.q.w, frame.q.x, frame.q.y, frame.q.z];
            true
        }
        _ => false,
    }
}

/// `Position::FromRealMapCoords`: `12.3N 45.6E` (or with a comma; a missing `E` may be
/// implied when the text ends with one).
fn from_real_map_coords(p: &mut Position, text: &[u8]) -> bool {
    let mut at = 0;
    let Some(y) = scan_number(text, &mut at, true) else {
        return false;
    };
    let Some(&ns) = text.get(at) else {
        return false;
    };
    at += 1;
    let _ = expect(text, &mut at, ",");
    let Some(x) = scan_number(text, &mut at, true) else {
        return false;
    };
    let ew = text.get(at).copied().unwrap_or(b'E').to_ascii_uppercase();
    let (mut x, mut y) = (x as f32, y as f32);
    match ew {
        b'W' => x = -x,
        b'E' => {}
        _ => return false,
    }
    match ns.to_ascii_uppercase() {
        b'S' => y = -y,
        b'N' => {}
        _ => return false,
    }
    place_in_cell(p, game_map_coords_to_gid(x, y))
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

/// `Position::ToRealMapCoords`: `%.1f%s, %.1f%s` (north/south first; no letter at 0).
fn to_real_map_coords(p: &Position) -> String {
    let Some((x, y)) = game_map_coords(p.cell) else {
        return "<error>".into();
    };
    let ns = if y < 0.0 {
        "S"
    } else if y > 0.0 {
        "N"
    } else {
        ""
    };
    let ew = if x < 0.0 {
        "W"
    } else if x > 0.0 {
        "E"
    } else {
        ""
    };
    format!(
        "{}{ns}, {}{ew}",
        msvc_fixed(ld(y.abs()), 1),
        msvc_fixed(ld(x.abs()), 1)
    )
}

/// `Position::ToRealMapCoords_Info`: a StringInfo in table enum 0x40000007, "RealMapCoords"
/// with NUM1/DIR1 (north/south) and NUM2/DIR2 (east/west), or "Inside".
fn to_real_map_coords_info(vm: &Vm, p: &Position) -> StringInfo {
    use crate::native::{StringVar, StringVarValue};
    let table = vm.data().enum_did(0x4000_0007, 0x4000_0004).unwrap_or(0);
    let mut info = StringInfo {
        table_id: if vm.data().db_type(table) == Some(0x4000_0026) {
            table
        } else {
            0
        },
        ..StringInfo::default()
    };
    let t = |s: &str| ac2_net::tokenize(s.as_bytes());
    match game_map_coords(p.cell) {
        None => info.string_id = t("Inside"),
        Some((x, y)) => {
            info.string_id = t("RealMapCoords");
            let float = |id, v: f32| StringVar {
                id: t(id),
                kind: 4,
                value: StringVarValue::Float {
                    value: ld(v.abs()),
                    precision: 1,
                },
            };
            let word = |id, s: &str| StringVar {
                id: t(id),
                kind: 9,
                value: StringVarValue::StringInfo(Box::new(StringInfo {
                    string_id: t(s),
                    table_id: info.table_id,
                    ..StringInfo::default()
                })),
            };
            info.variables = vec![
                float("NUM1", y),
                word("DIR1", if 0.0 <= y { "North" } else { "South" }),
                float("NUM2", x),
                word("DIR2", if 0.0 <= x { "East" } else { "West" }),
            ];
        }
    }
    info
}

fn string_arg(vm: &Vm, ctx: &Context, n: usize) -> Vec<u8> {
    vm.wstring(word(ctx, n)).map(narrow).unwrap_or_default()
}

fn from_text(vm: &mut Vm, ctx: &mut Context, f: fn(&mut Position, &[u8]) -> bool) -> InterpStatus {
    let text = string_arg(vm, ctx, 2);
    let mut ok = false;
    let status = edit(vm, ctx, |p, _| ok = f(p, &text));
    Call::new(ctx).ret(3, u32::from(ok), INT_RESULT);
    status
}

fn position_method(m: &str) -> Option<EFunFn> {
    Some(match m {
        "Set" => |vm, c| {
            let Some(o) = pos(vm, word(c, 2)) else {
                return vm.bad_this(word(c, 2));
            };
            edit(vm, c, |p, _| *p = o)
        },
        "SetCellID" | "set_cell_id" => |vm, c| {
            let cell = word(c, 2);
            edit(vm, c, |p, _| p.cell = cell)
        },
        "set_height" => |vm, c| {
            let z = f32::from_bits(word(c, 2));
            edit(vm, c, |p, _| p.origin[2] = z)
        },
        "set_origin" => |vm, c| {
            let Some(v) = vec3(vm, word(c, 2)) else {
                return vm.bad_this(word(c, 2));
            };
            edit(vm, c, |p, _| p.origin = v)
        },
        "add_global_offset" => |vm, c| {
            let Some(v) = vec3(vm, word(c, 2)) else {
                return vm.bad_this(word(c, 2));
            };
            edit(vm, c, |p, _| {
                for (axis, offset) in p.origin.iter_mut().zip(v) {
                    *axis += offset;
                }
            })
        },
        "add_local_offset" => |vm, c| {
            let Some(v) = vec3(vm, word(c, 2)) else {
                return vm.bad_this(word(c, 2));
            };
            edit(vm, c, |p, _| {
                let mut q = phys(p);
                q.add_local_offset(Vector::new(v[0], v[1], v[2]));
                p.origin = arr(q.frame.origin);
            })
        },
        "adjust_to_outside" => |vm, c| {
            edit(vm, c, |p, _| {
                let mut origin = Vector::new(p.origin[0], p.origin[1], p.origin[2]);
                adjust_to_outside(&mut p.cell, &mut origin);
                p.origin = arr(origin);
            })
        },
        "FromCellCoords" => |vm, c| {
            let (x, y) = (word(c, 2) as i32, word(c, 3) as i32);
            let mut ok = false;
            let status = edit(vm, c, |p, _| ok = place_in_cell(p, lcoord_to_gid(x, y)));
            Call::new(c).ret(4, u32::from(ok), INT_RESULT);
            status
        },
        "FromCellCoordsString" => |vm, c| {
            from_text(vm, c, |p, s| {
                let mut at = 0;
                match (
                    scan_number(s, &mut at, false),
                    scan_number(s, &mut at, false),
                ) {
                    (Some(x), Some(y)) => place_in_cell(p, lcoord_to_gid(x as i32, y as i32)),
                    _ => false,
                }
            })
        },
        "FromRealMapCoords" => |vm, c| from_text(vm, c, from_real_map_coords),
        "FromWPString" => |vm, c| from_text(vm, c, from_wp_string),
        "GetBlockID" => |vm, c| query(vm, c, 2, false, |p, _, _| p.cell & 0xFFFF_0000 | 0xFFFF),
        "GetCellID" => |vm, c| query(vm, c, 2, false, |p, _, _| p.cell),
        "GetCellX" => |vm, c| {
            query(vm, c, 2, false, |p, _, _| {
                gid_to_lcoord(p.cell).map_or(u32::MAX, |(x, _)| x as u32)
            })
        },
        "GetCellY" => |vm, c| {
            query(vm, c, 2, false, |p, _, _| {
                gid_to_lcoord(p.cell).map_or(u32::MAX, |(_, y)| y as u32)
            })
        },
        "GetCellDistance" | "GetCellDistanceSquared" => |vm, c| {
            let (c1, c2) = (word(c, 2), word(c, 3));
            let squared =
                super::efun_name(vm.current_efun()).is_some_and(|n| n.ends_with("Squared"));
            let d2 = cell_distance_squared(c1, c2);
            let v = if squared { d2 } else { ld(d2).sqrt() as f32 };
            Call::new(c).ret(4, v.to_bits(), INT_RESULT);
            InterpStatus::Success
        },
        "IsValid" => |vm, c| query(vm, c, 2, false, |p, _, _| u32::from(is_valid(p))),
        "NotInWorld" => |vm, c| query(vm, c, 2, false, |p, _, _| u32::from(p.cell == 0)),
        "outside" => |vm, c| {
            query(vm, c, 2, false, |p, _, _| {
                u32::from(p.cell & CELLID_MASK < FIRST_ENVCELL_ID)
            })
        },
        "get_height" => |vm, c| query(vm, c, 2, false, |p, _, _| p.origin[2].to_bits()),
        "in_same_block" => |vm, c| {
            query(vm, c, 3, true, |a, b, _| {
                let (x, y) = (a.cell, b.map_or(0, |b| b.cell));
                u32::from(match (x, y) {
                    (0, 0) => true,
                    (0, _) | (_, 0) => false,
                    _ => x >> 16 == y >> 16,
                })
            })
        },
        "equals" => |vm, c| {
            query(vm, c, 3, true, |a, b, _| {
                u32::from(b.is_some_and(|b| a.cell == b.cell && frames_equal(a, b)))
            })
        },
        "distance" => |vm, c| {
            query(vm, c, 3, true, |a, b, _| {
                (phys(a).distance(&phys(b.unwrap())) as f32).to_bits()
            })
        },
        "distance_squared" => |vm, c| {
            query(vm, c, 3, true, |a, b, _| {
                (phys(a).distance_squared(&phys(b.unwrap())) as f32).to_bits()
            })
        },
        "xy_distance" | "xy_distance_squared" => |vm, c| {
            let squared =
                super::efun_name(vm.current_efun()).is_some_and(|n| n.ends_with("squared"));
            let call = Call::new(c);
            let (Some(a), Some(b)) = (pos(vm, call.this()), pos(vm, call.word(2))) else {
                return vm.bad_this(Call::new(c).this());
            };
            let o = phys(&a).offset_to(&phys(&b));
            let d2 = ld(o.x) * ld(o.x) + ld(o.y) * ld(o.y);
            let v = if squared { d2 as f32 } else { d2.sqrt() as f32 };
            Call::new(c).ret(3, v.to_bits(), INT_RESULT);
            InterpStatus::Success
        },
        "pitch" => |vm, c| query(vm, c, 3, true, |a, b, _| pitch(a, b.unwrap()).to_bits()),
        "determine_quadrant" => |vm, c| {
            let call = Call::new(c);
            let (this, height, other) = (call.this(), f32::from_bits(call.word(2)), call.word(3));
            let (Some(a), Some(b)) = (pos(vm, this), pos(vm, other)) else {
                return vm.bad_this(this);
            };
            Call::new(c).ret(4, quadrant(&a, height, &b), INT_RESULT);
            InterpStatus::Success
        },
        "cylinder_distance" => |vm, c| {
            let f = |n| f32::from_bits(word(c, n));
            let (r1, h1, r2, h2) = (f(2), f(3), f(5), f(6));
            let (Some(p1), Some(p2)) = (pos(vm, word(c, 4)), pos(vm, word(c, 7))) else {
                return vm.bad_this(word(c, 4));
            };
            let v = cylinder_distance(r1, h1, &p1, r2, h2, &p2);
            Call::new(c).ret(8, v.to_bits(), INT_RESULT);
            InterpStatus::Success
        },
        "heading" | "GetLocalHeading" => |vm, c| {
            let local =
                super::efun_name(vm.current_efun()).is_some_and(|n| n.ends_with("GetLocalHeading"));
            let call = Call::new(c);
            let (Some(a), Some(b)) = (pos(vm, call.this()), pos(vm, call.word(2))) else {
                return vm.bad_this(Call::new(c).this());
            };
            let h = if local {
                local_heading(&a, &b)
            } else {
                phys(&a).heading_to(&phys(&b))
            };
            new_native(vm, c, 3, ntype::HEADING, NativeObject::Heading(h))
        },
        "get_origin" => |vm, c| {
            let this = Call::new(c).this();
            let Some(p) = pos(vm, this) else {
                return vm.bad_this(this);
            };
            new_native(vm, c, 2, ntype::VECTOR, NativeObject::Vector(p.origin))
        },
        "get_offset" => |vm, c| {
            let call = Call::new(c);
            let (Some(a), Some(b)) = (pos(vm, call.this()), pos(vm, call.word(2))) else {
                return vm.bad_this(Call::new(c).this());
            };
            let o = arr(phys(&a).offset_to(&phys(&b)));
            new_native(vm, c, 3, ntype::VECTOR, NativeObject::Vector(o))
        },
        "get_block_offset" => |vm, c| {
            let this = Call::new(c).this();
            let Some(p) = pos(vm, this) else {
                return vm.bad_this(this);
            };
            let o = arr(get_block_offset(p.cell, word(c, 2)));
            new_native(vm, c, 3, ntype::VECTOR, NativeObject::Vector(o))
        },
        "GetNullPosition" => |vm, c| {
            let p = NativeObject::new_empty(ntype::POSITION);
            new_native(vm, c, 2, ntype::POSITION, p)
        },
        "ToWPString" => |vm, c| {
            let this = Call::new(c).this();
            let Some(p) = pos(vm, this) else {
                return vm.bad_this(this);
            };
            let f = |v: f32| msvc_fixed(ld(v), 6);
            let text = format!(
                "0x{:08X} [{} {} {}] {} {} {} {}",
                p.cell,
                f(p.origin[0]),
                f(p.origin[1]),
                f(p.origin[2]),
                f(p.rotation[0]),
                f(p.rotation[1]),
                f(p.rotation[2]),
                f(p.rotation[3])
            );
            let r = vm.new_wstring(wide(&text));
            Call::new(c).ret(2, r, tag::REF);
            InterpStatus::Success
        },
        "ToCellCoordsString" => |vm, c| {
            let this = Call::new(c).this();
            let Some(p) = pos(vm, this) else {
                return vm.bad_this(this);
            };
            let text = gid_to_lcoord(p.cell).map_or("<error>".into(), |(x, y)| format!("{x} {y}"));
            let r = vm.new_wstring(wide(&text));
            Call::new(c).ret(2, r, tag::REF);
            InterpStatus::Success
        },
        "ToRealMapCoords" => |vm, c| {
            let this = Call::new(c).this();
            let Some(p) = pos(vm, this) else {
                return vm.bad_this(this);
            };
            let r = vm.new_wstring(wide(&to_real_map_coords(&p)));
            Call::new(c).ret(2, r, tag::REF);
            InterpStatus::Success
        },
        "ToRealMapCoords_Info" => |vm, c| {
            let this = Call::new(c).this();
            let Some(p) = pos(vm, this) else {
                return vm.bad_this(this);
            };
            let info = to_real_map_coords_info(vm, &p);
            new_native(vm, c, 2, ntype::STRING_INFO, NativeObject::StringInfo(info))
        },
        _ => return None,
    })
}

/// `Position::GetLocalHeading(p)`: the heading toward `p` in this frame's own axes.
fn local_heading(a: &Position, b: &Position) -> Heading {
    let block = get_block_offset(a.cell, b.cell);
    let g = Vector::new(
        (ld(block.x) + ld(b.origin[0])) as f32,
        (ld(block.y) + ld(b.origin[1])) as f32,
        (ld(block.z) + ld(b.origin[2])) as f32,
    );
    let v = phys(a).frame.global_to_local(g);
    let len = (ld(v.x) * ld(v.x) + ld(v.y) * ld(v.y)).sqrt();
    if len < EPSILON {
        return Heading::from_degrees(0.0);
    }
    let inv = 1.0 / len;
    let nx = (ld(v.x) * inv) as f32;
    Heading::from_radians((FIVE_HALVES_PI - (inv * ld(v.y)).atan2(ld(nx))) as f32)
}

fn heading_of(vm: &Vm, r: u32) -> Option<Heading> {
    match vm.native(r)? {
        NativeObject::Heading(h) => Some(*h),
        _ => None,
    }
}

fn heading_method(m: &str) -> Option<EFunFn> {
    Some(match m {
        "set_degrees" => |vm, c| {
            let this = Call::new(c).this();
            let d = f32::from_bits(word(c, 2));
            match vm.native_mut(this) {
                Some(NativeObject::Heading(h)) => {
                    *h = Heading::from_degrees(d);
                    InterpStatus::Success
                }
                _ => vm.bad_this(this),
            }
        },
        "get_degrees" | "get_degrees_0_centered" => |vm, c| {
            let this = Call::new(c).this();
            let centered =
                super::efun_name(vm.current_efun()).is_some_and(|n| n.ends_with("centered"));
            let Some(h) = heading_of(vm, this) else {
                return vm.bad_this(this);
            };
            let v = if centered {
                h.degrees_0_centered() as f32
            } else {
                h.degrees()
            };
            Call::new(c).ret(2, v.to_bits(), INT_RESULT);
            InterpStatus::Success
        },
        "is_in_range" => |vm, c| {
            let call = Call::new(c);
            let (this, center, range) = (call.this(), call.word(2), call.word(3));
            let (Some(h), Some(center), Some(range)) = (
                heading_of(vm, this),
                heading_of(vm, center),
                heading_of(vm, range),
            ) else {
                return vm.bad_this(this);
            };
            Call::new(c).ret(4, u32::from(h.is_in_range(center, range)), INT_RESULT);
            InterpStatus::Success
        },
        _ => return None,
    })
}
