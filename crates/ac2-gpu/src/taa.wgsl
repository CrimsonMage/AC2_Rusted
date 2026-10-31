// Temporal anti-aliasing resolve (`taa.rs`): the jittered scene colour and the history,
// reprojected from the depth buffer, blended into the new history. One full-screen triangle, so
// it runs on the GL fallback as well. `taa.rs` copies the result back over the scene colour.
//
// Reprojection is camera-only: the current frame's unjittered eye-relative inverse
// view-projection puts the pixel's depth back in space, the eye's step (previous minus current,
// taken in f64 on the CPU) moves it into the previous frame's eye-relative space, and the
// previous unjittered view-projection puts it on the previous screen. A pixel at the far depth
// (the sky) moves as a direction (rotation only). A pixel an object drew carries the object's own
// screen-space velocity (`velocity`, written by `object.wgsl` from the previous placement), which
// replaces that reprojection; where the object had no previous placement it holds a sentinel and
// the camera's motion stands.

struct Params {
    inv_view_proj: mat4x4<f32>,
    prev_view_proj: mat4x4<f32>,
    // xyz: the previous eye minus the current eye, in metres.
    eye_delta: vec4<f32>,
    // x: 1 when the history holds the previous frame, y: the blend towards the current frame
    // at rest, z: the blend at speed, w: the screen motion in pixels that reaches z.
    blend: vec4<f32>,
}

@group(0) @binding(0) var current: texture_2d<f32>;
@group(0) @binding(1) var scene_depth: texture_depth_2d;
@group(0) @binding(2) var history: texture_2d<f32>;
@group(0) @binding(3) var history_sampler: sampler;
@group(0) @binding(4) var<uniform> params: Params;
// The previous minus the current screen position of the object at each pixel, in uv units;
// past `NO_VELOCITY_ABOVE` where no object drew (`velocity.rs`).
@group(0) @binding(5) var velocity: texture_2d<f32>;

const NO_VELOCITY_ABOVE: f32 = 100.0;

@vertex
fn vs(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let uv = vec2(f32((index << 1u) & 2u), f32(index & 2u));
    return vec4(uv * 2.0 - 1.0, 0.0, 1.0);
}

fn luma(c: vec3<f32>) -> f32 {
    return dot(c, vec3(0.25, 0.5, 0.25));
}

// Karis' luminance weight: a bright HDR pixel counts for little in the average, so one lit
// texel does not flicker the whole neighbourhood.
fn weigh(c: vec3<f32>) -> vec3<f32> {
    let v = max(c, vec3(0.0));
    return v / (1.0 + luma(v));
}

fn unweigh(c: vec3<f32>) -> vec3<f32> {
    return c / max(1.0 - luma(c), 1.0e-3);
}

fn to_ycocg(c: vec3<f32>) -> vec3<f32> {
    return vec3(
        0.25 * c.r + 0.5 * c.g + 0.25 * c.b,
        0.5 * c.r - 0.5 * c.b,
        -0.25 * c.r + 0.5 * c.g - 0.25 * c.b,
    );
}

fn from_ycocg(c: vec3<f32>) -> vec3<f32> {
    return vec3(c.x + c.y - c.z, c.x + c.z, c.x - c.y - c.z);
}

fn history_tap(uv: vec2<f32>) -> vec3<f32> {
    return weigh(textureSampleLevel(history, history_sampler, uv, 0.0).rgb);
}

// The history at `uv` through a Catmull-Rom filter in five bilinear taps.
fn history_catmull_rom(uv: vec2<f32>, size: vec2<f32>) -> vec3<f32> {
    let position = uv * size;
    let centre = floor(position - 0.5) + 0.5;
    let f = position - centre;
    let w0 = f * (-0.5 + f * (1.0 - 0.5 * f));
    let w1 = 1.0 + f * f * (-2.5 + 1.5 * f);
    let w2 = f * (0.5 + f * (2.0 - 1.5 * f));
    let w3 = f * f * (-0.5 + 0.5 * f);
    let w12 = w1 + w2;
    let uv0 = (centre - 1.0) / size;
    let uv3 = (centre + 2.0) / size;
    let uv12 = (centre + w2 / w12) / size;
    let sum = history_tap(vec2(uv12.x, uv0.y)) * (w12.x * w0.y)
        + history_tap(vec2(uv0.x, uv12.y)) * (w0.x * w12.y)
        + history_tap(uv12) * (w12.x * w12.y)
        + history_tap(vec2(uv3.x, uv12.y)) * (w3.x * w12.y)
        + history_tap(vec2(uv12.x, uv3.y)) * (w12.x * w3.y);
    let total = w12.x * w0.y + w0.x * w12.y + w12.x * w12.y + w3.x * w12.y + w12.x * w3.y;
    return max(sum / total, vec3(0.0));
}

fn clip_to_box(low: vec3<f32>, high: vec3<f32>, q: vec3<f32>) -> vec3<f32> {
    let centre = 0.5 * (low + high);
    let extent = 0.5 * (high - low) + 1.0e-5;
    let v = q - centre;
    let a = abs(v / extent);
    let m = max(a.x, max(a.y, a.z));
    return select(q, centre + v / m, m > 1.0);
}

// Where the pixel `at` was on the previous frame's screen (uv), or `valid` false when it has no
// place there. `depth` is the pixel's own (the nearest of its neighbours, see `fs`).
struct Motion {
    prev_uv: vec2<f32>,
    valid: bool,
}

fn motion(at: vec2<i32>, size: vec2<f32>, depth: f32) -> Motion {
    let uv = (vec2<f32>(at) + 0.5) / size;
    let ndc = vec2(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0);
    var clip: vec4<f32>;
    if depth >= 1.0 {
        // The sky: a direction, rotated by the previous view alone.
        let near = params.inv_view_proj * vec4(ndc, 0.0, 1.0);
        clip = params.prev_view_proj * vec4(near.xyz / near.w, 0.0);
    } else {
        let h = params.inv_view_proj * vec4(ndc, depth, 1.0);
        let relative = h.xyz / h.w - params.eye_delta.xyz;
        clip = params.prev_view_proj * vec4(relative, 1.0);
    }
    if clip.w <= 1.0e-6 {
        return Motion(uv, false);
    }
    let prev_ndc = clip.xy / clip.w;
    return Motion(vec2(prev_ndc.x * 0.5 + 0.5, 0.5 - prev_ndc.y * 0.5), true);
}

@fragment
fn fs(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let at = vec2<i32>(position.xy);
    let dims = vec2<i32>(textureDimensions(current));
    let size = vec2<f32>(dims);
    let top = dims - vec2(1, 1);
    let raw = textureLoad(current, at, 0);
    if params.blend.x < 0.5 {
        return raw;
    }
    // The 3 x 3 neighbourhood of the current frame in YCoCg, and its nearest depth.
    var sum = vec3(0.0);
    var squares = vec3(0.0);
    var low = vec3(1.0e9);
    var high = vec3(-1.0e9);
    var nearest = 2.0;
    var nearest_at = at;
    for (var y = -1; y <= 1; y++) {
        for (var x = -1; x <= 1; x++) {
            let tap = clamp(at + vec2(x, y), vec2(0, 0), top);
            let c = to_ycocg(weigh(textureLoad(current, tap, 0).rgb));
            sum += c;
            squares += c * c;
            low = min(low, c);
            high = max(high, c);
            let d = textureLoad(scene_depth, tap, 0);
            if d < nearest {
                nearest = d;
                nearest_at = tap;
            }
        }
    }
    let mean = sum / 9.0;
    let sigma = sqrt(max(squares / 9.0 - mean * mean, vec3(0.0)));
    let box_low = max(mean - 1.25 * sigma, low);
    let box_high = min(mean + 1.25 * sigma, high);

    // The nearest neighbour's motion moves the pixel, so a foreground edge keeps its own: the
    // object's when it drew there, else the camera's.
    let uv = (vec2<f32>(at) + 0.5) / size;
    let object = textureLoad(velocity, nearest_at, 0).xy;
    var step: vec2<f32>;
    if object.x < NO_VELOCITY_ABOVE {
        step = object;
    } else {
        let m = motion(nearest_at, size, nearest);
        if !m.valid {
            return raw;
        }
        step = m.prev_uv - (vec2<f32>(nearest_at) + 0.5) / size;
    }
    let prev_uv = uv + step;
    if any(prev_uv < vec2(0.0)) || any(prev_uv > vec2(1.0)) {
        return raw;
    }
    let past = to_ycocg(history_catmull_rom(prev_uv, size));
    let clipped = clip_to_box(box_low, box_high, past);
    let speed = length(step * size);
    let alpha = mix(params.blend.y, params.blend.z, clamp(speed / params.blend.w, 0.0, 1.0));
    let blended = mix(clipped, to_ycocg(weigh(raw.rgb)), alpha);
    return vec4(unweigh(max(from_ycocg(blended), vec3(0.0))), raw.a);
}
