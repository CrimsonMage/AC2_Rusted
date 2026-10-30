// Sun shadow receivers (`sun_shadow.rs`). The including shader declares `sun_shadow` (a
// `SunShadow` uniform), `sun_shadow_map` (texture_depth_2d_array) and `sun_shadow_sampler`
// (sampler_comparison).

struct SunShadow {
    // Eye-relative points to each cascade's clip space (`Cascade::clip_rel`).
    cascades: array<mat4x4<f32>, 4>,
    // Far distance of each cascade from the eye.
    splits: vec4<f32>,
    // Metres per texel of each cascade.
    texels: vec4<f32>,
    // rgb: the shadow strength; w: 1 / map size.
    strength: vec4<f32>,
    // xyz: the eye the points are taken off; w: the cascade count (0: off).
    eye: vec4<f32>,
};

// Points here are relative to the eye (`sun_shadow.eye`), so a body tens of kilometres out
// keeps its millimetres. How far the cascades cover point `p`: 1 near the eye, falling to 0
// over the last cascade's far end (where the landscape map takes over again).
fn sun_cover(p: vec3<f32>) -> f32 {
    let count = u32(sun_shadow.eye.w);
    if count == 0u {
        return 0.0;
    }
    let last = sun_shadow.splits[count - 1u];
    return 1.0 - smoothstep(last * (1.0 - SUN_SHADOW_FADE), last, length(p));
}

// How much sun reaches eye-relative point `p` with world normal `n`: 1 lit, 0 in full shadow. The
// nearest cascade holding `p`, offset along its normal by a texel and a half (no acne on slopes),
// 3 x 3 bilinear comparison taps (a 4 x 4 texel soft edge), fading to lit with `sun_cover`.
fn sun_lit(p: vec3<f32>, n: vec3<f32>) -> f32 {
    let count = u32(sun_shadow.eye.w);
    if count == 0u {
        return 1.0;
    }
    let distance = length(p);
    if distance >= sun_shadow.splits[count - 1u] {
        return 1.0;
    }
    var index = 0u;
    for (var i = 0u; i < count; i = i + 1u) {
        if distance >= sun_shadow.splits[i] {
            index = i + 1u;
        }
    }
    index = min(index, count - 1u);
    let offset = p + normalize(n) * sun_shadow.texels[index] * 1.5;
    let clip = sun_shadow.cascades[index] * vec4(offset, 1.0);
    let uv = clip.xy * vec2(0.5, -0.5) + 0.5;
    let depth = clip.z;
    if any(uv < vec2(0.0)) || any(uv > vec2(1.0)) || depth > 1.0 {
        return 1.0;
    }
    let step = sun_shadow.strength.w;
    var lit = 0.0;
    for (var dx = -1; dx <= 1; dx = dx + 1) {
        for (var dy = -1; dy <= 1; dy = dy + 1) {
            let at = uv + vec2(f32(dx), f32(dy)) * step;
            lit = lit + textureSampleCompareLevel(sun_shadow_map, sun_shadow_sampler, at, i32(index), depth);
        }
    }
    return mix(lit / 9.0, 1.0, 1.0 - sun_cover(p));
}

// Retail's darkening of a shadowed colour (`RenderGlobalVolumetricShadows`' quad): one minus the
// shadow strength in full shadow, all of it in the sun.
fn sun_shadow_keep(lit: f32) -> vec3<f32> {
    return mix(vec3(1.0) - sun_shadow.strength.rgb, vec3(1.0), lit);
}
