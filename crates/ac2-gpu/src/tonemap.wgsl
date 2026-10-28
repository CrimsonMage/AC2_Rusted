// The scene's HDR colour (linear light, `Rgba16Float`) to the display: the Khronos PBR Neutral
// tone map, then sRGB encode. It passes colours below about 0.8 through nearly unchanged (a
// slight toe offset only), so the DAT's authored colours keep their hue and saturation, and rolls
// brighter light off towards white without per-channel hue shifts. One pass after the world
// (and after each UI viewport's scene, limited to its rectangle); the texel under each output
// pixel is read 1:1, so the HDR target matches the output's size. `tonemap.rs` mirrors the curve
// for its tests.
//
// A UI viewport's scene composites over the UI surfaces behind it (`UIScene::Render` clears
// only the depth): `fs_composite` covers the pixels the scene drew, fully where it wrote depth
// and by its blended alpha (accumulated with `max`) where it did not.

@group(0) @binding(0) var scene: texture_2d<f32>;
@group(0) @binding(1) var scene_depth: texture_depth_2d;
// The world's post chain (`post.rs`): its bloom, and one texel with the adapted EV (r), the bloom
// strength (g), the vignette strength (b) and one over the bloom's level count (a). Neutral (all
// zero) for the UI viewports' scenes and offline renders.
@group(1) @binding(0) var bloom: texture_2d<f32>;
@group(1) @binding(1) var bloom_sampler: sampler;
@group(1) @binding(2) var exposure: texture_2d<f32>;

const EXPOSURE: f32 = 1.0;
const START_COMPRESSION: f32 = 0.76;
const DESATURATION: f32 = 0.15;

fn linear_to_srgb(c: vec3<f32>) -> vec3<f32> {
    let low = c * 12.92;
    let high = 1.055 * pow(max(c, vec3(0.0)), vec3(1.0 / 2.4)) - 0.055;
    return select(high, low, c <= vec3(0.0031308));
}

// Khronos PBR Neutral (glTF sample viewer).
fn pbr_neutral(input: vec3<f32>) -> vec3<f32> {
    var color = input;
    let x = min(color.r, min(color.g, color.b));
    let offset = select(0.04, x - 6.25 * x * x, x < 0.08);
    color -= offset;
    let peak = max(color.r, max(color.g, color.b));
    if peak < START_COMPRESSION {
        return color;
    }
    let d = 1.0 - START_COMPRESSION;
    let new_peak = 1.0 - d * d / (peak + d - START_COMPRESSION);
    color *= new_peak / peak;
    let g = 1.0 - 1.0 / (DESATURATION * (peak - new_peak) + 1.0);
    return mix(color, vec3(new_peak), g);
}

@vertex
fn vs(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let uv = vec2(f32((index << 1u) & 2u), f32(index & 2u));
    return vec4(uv * 2.0 - 1.0, 0.0, 1.0);
}

fn display(linear: vec3<f32>) -> vec3<f32> {
    let c = max(linear, vec3(0.0)) * EXPOSURE;
    return clamp(linear_to_srgb(pbr_neutral(c)), vec3(0.0), vec3(1.0));
}

@fragment
fn fs(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let texel = textureLoad(scene, vec2<i32>(position.xy), 0);
    let size = vec2<f32>(textureDimensions(scene));
    let uv = position.xy / size;
    let post = textureLoad(exposure, vec2(0, 0), 0);
    // The bloom's sum over its levels, as their mean, mixed into the scene in linear light.
    let glow = textureSampleLevel(bloom, bloom_sampler, uv, 0.0).rgb * post.a;
    var c = mix(max(texel.rgb, vec3(0.0)), glow, post.g) * exp2(post.r);
    // The vignette: 0 at the centre, 1 in the corners whatever the aspect.
    let aspect = vec2(size.x / size.y, 1.0);
    let edge = length((uv - 0.5) * aspect) / length(0.5 * aspect);
    c *= 1.0 - post.b * smoothstep(0.35, 1.0, edge);
    return vec4(display(c), 1.0);
}

@fragment
fn fs_composite(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let at = vec2<i32>(position.xy);
    let texel = textureLoad(scene, at, 0);
    let drawn = textureLoad(scene_depth, at, 0) < 1.0;
    let coverage = select(clamp(texel.a, 0.0, 1.0), 1.0, drawn);
    return vec4(display(texel.rgb), coverage);
}
