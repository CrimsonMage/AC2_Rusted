// The world's post chain in linear light, before the tone map (`post.rs`): a dual-filter bloom
// (a 13-tap downsample chain from half size, then 3x3 tent upsamples adding each level back) and
// the eye adaptation, which averages log luminance over the smallest bloom level and eases the
// exposure towards its key. Every pass is one full-screen triangle into its own target.

@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var linear_clamp: sampler;
// The upsample's coarser level (the down level is `source`).
@group(0) @binding(2) var coarser: texture_2d<f32>;

struct Adapt {
    // x: seconds since the last adaptation, y: 1 to jump to the target, z: log2 of the key,
    // w: bloom strength.
    step: vec4<f32>,
    // x, y: the exposure's EV limits; z: seconds to adapt towards a brighter scene, w: towards a
    // darker one; both are e-folding times.
    limits: vec4<f32>,
    // x: vignette strength, y: one over the bloom's level count, z: the share of the metered
    // difference the exposure follows.
    look: vec4<f32>,
};
@group(1) @binding(0) var<uniform> adapt: Adapt;
@group(1) @binding(1) var previous: texture_2d<f32>;

@vertex
fn vs(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let uv = vec2(f32((index << 1u) & 2u), f32(index & 2u));
    return vec4(uv * 2.0 - 1.0, 0.0, 1.0);
}

fn luma(c: vec3<f32>) -> f32 {
    return dot(c, vec3(0.2126, 0.7152, 0.0722));
}

fn tap(uv: vec2<f32>, texel: vec2<f32>, x: f32, y: f32) -> vec3<f32> {
    return textureSampleLevel(source, linear_clamp, uv + vec2(x, y) * texel, 0.0).rgb;
}

// Jimenez's 13-tap downsample: five overlapping 2x2 boxes, the centre one weighted 0.5.
fn downsample(position: vec4<f32>, karis: bool) -> vec3<f32> {
    let size = vec2<f32>(textureDimensions(source));
    let texel = 1.0 / size;
    let out_size = size * 0.5;
    let uv = position.xy / out_size;
    let a = tap(uv, texel, -2.0, 2.0);
    let b = tap(uv, texel, 0.0, 2.0);
    let c = tap(uv, texel, 2.0, 2.0);
    let d = tap(uv, texel, -2.0, 0.0);
    let e = tap(uv, texel, 0.0, 0.0);
    let f = tap(uv, texel, 2.0, 0.0);
    let g = tap(uv, texel, -2.0, -2.0);
    let h = tap(uv, texel, 0.0, -2.0);
    let i = tap(uv, texel, 2.0, -2.0);
    let j = tap(uv, texel, -1.0, 1.0);
    let k = tap(uv, texel, 1.0, 1.0);
    let l = tap(uv, texel, -1.0, -1.0);
    let m = tap(uv, texel, 1.0, -1.0);
    let boxes = array<vec3<f32>, 5>(
        (j + k + l + m) * 0.25,
        (a + b + d + e) * 0.25,
        (b + c + e + f) * 0.25,
        (d + e + g + h) * 0.25,
        (e + f + h + i) * 0.25,
    );
    let weights = array<f32, 5>(0.5, 0.125, 0.125, 0.125, 0.125);
    var sum = vec3(0.0);
    var total = 0.0;
    for (var n = 0; n < 5; n++) {
        // The first level weighs each box by 1 / (1 + luma) (Karis), so a lone bright texel (the
        // sun's disc, a spark) blooms as a glow instead of a flickering square.
        let w = select(weights[n], weights[n] / (1.0 + luma(boxes[n])), karis);
        sum += boxes[n] * w;
        total += w;
    }
    return max(sum / total, vec3(0.0));
}

@fragment
fn fs_down_first(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    // Scenes are unclamped; a texel past this is the sun or a light source, not area light.
    return vec4(min(downsample(position, true), vec3(256.0)), 1.0);
}

@fragment
fn fs_down(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    return vec4(downsample(position, false), 1.0);
}

// This level plus the 3x3 tent upsample of the coarser level's sum.
@fragment
fn fs_up(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let size = vec2<f32>(textureDimensions(source));
    let uv = position.xy / size;
    let texel = 1.0 / vec2<f32>(textureDimensions(coarser));
    var tent = vec3(0.0);
    for (var y = -1; y <= 1; y++) {
        for (var x = -1; x <= 1; x++) {
            let w = f32((2 - abs(x)) * (2 - abs(y)));
            tent += w * textureSampleLevel(coarser, linear_clamp, uv + vec2(f32(x), f32(y)) * texel, 0.0).rgb;
        }
    }
    let here = textureSampleLevel(source, linear_clamp, uv, 0.0).rgb;
    return vec4(here + tent / 16.0, 1.0);
}

// The adapted exposure as one texel: r the EV the tone map applies, g the bloom strength, b the
// vignette strength (`tonemap.wgsl`).
@fragment
fn fs_adapt() -> @location(0) vec4<f32> {
    let size = textureDimensions(source);
    var sum = 0.0;
    var total = 0.0;
    for (var y = 0u; y < size.y; y++) {
        for (var x = 0u; x < size.x; x++) {
            let uv = (vec2(f32(x), f32(y)) + 0.5) / vec2<f32>(size);
            // Centre-weighted: what the player looks at sets the exposure more than the edges.
            let r = length((uv - 0.5) * vec2(1.6, 2.0));
            let w = 1.0 - 0.75 * smoothstep(0.2, 1.0, r);
            let l = clamp(luma(textureLoad(source, vec2(x, y), 0).rgb), 1.0 / 1024.0, 1024.0);
            sum += w * log2(l);
            total += w;
        }
    }
    let metered = adapt.step.z - sum / max(total, 1e-4);
    let target_ev = clamp(metered * adapt.look.z, adapt.limits.x, adapt.limits.y);
    let last = textureLoad(previous, vec2(0, 0), 0).r;
    // Brighter scene (lower EV) adapts at `limits.z`, darker at `limits.w`.
    let time = select(adapt.limits.w, adapt.limits.z, target_ev < last);
    let blend = 1.0 - exp(-adapt.step.x / max(time, 1e-3));
    let ev = select(mix(last, target_ev, blend), target_ev, adapt.step.y > 0.5);
    return vec4(ev, adapt.step.w, adapt.look.x, adapt.look.y);
}
