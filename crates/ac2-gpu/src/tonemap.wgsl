// The scene's HDR colour (linear light, `Rgba16Float`) to the display: exposure and contrast
// about middle grey, the ACES filmic curve, sRGB encode, then saturation about luma in display
// space. One pass after the world (and after each UI viewport's scene, limited to its
// rectangle); the texel under each output pixel is read 1:1, so the HDR target matches the
// output's size. `tonemap.rs` mirrors these curves for its tests.
//
// A UI viewport's scene composites over the UI surfaces behind it (`UIScene::Render` clears
// only the depth): `fs_composite` covers the pixels the scene drew, fully where it wrote depth
// and by its blended alpha (accumulated with `max`) where it did not.

@group(0) @binding(0) var scene: texture_2d<f32>;
@group(0) @binding(1) var scene_depth: texture_depth_2d;

const EXPOSURE: f32 = 1.0;
const CONTRAST: f32 = 1.05;
const SATURATION: f32 = 1.05;
const PIVOT: f32 = 0.18;

fn linear_to_srgb(c: vec3<f32>) -> vec3<f32> {
    let low = c * 12.92;
    let high = 1.055 * pow(max(c, vec3(0.0)), vec3(1.0 / 2.4)) - 0.055;
    return select(high, low, c <= vec3(0.0031308));
}

// Narkowicz's ACES fit (the approximation Unreal's default view transform uses).
fn aces_filmic(x: vec3<f32>) -> vec3<f32> {
    let num = x * (2.51 * x + 0.03);
    let den = x * (2.43 * x + 0.59) + 0.14;
    return clamp(num / den, vec3(0.0), vec3(1.0));
}

@vertex
fn vs(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let uv = vec2(f32((index << 1u) & 2u), f32(index & 2u));
    return vec4(uv * 2.0 - 1.0, 0.0, 1.0);
}

fn display(linear: vec3<f32>) -> vec3<f32> {
    let c = max(linear, vec3(0.0)) * EXPOSURE;
    let contrasted = PIVOT * pow(c / PIVOT, vec3(CONTRAST));
    let encoded = linear_to_srgb(aces_filmic(contrasted));
    let luma = dot(encoded, vec3(0.2126, 0.7152, 0.0722));
    return clamp(mix(vec3(luma), encoded, SATURATION), vec3(0.0), vec3(1.0));
}

@fragment
fn fs(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let texel = textureLoad(scene, vec2<i32>(position.xy), 0);
    return vec4(display(texel.rgb), 1.0);
}

@fragment
fn fs_composite(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let at = vec2<i32>(position.xy);
    let texel = textureLoad(scene, at, 0);
    let drawn = textureLoad(scene_depth, at, 0) < 1.0;
    let coverage = select(clamp(texel.a, 0.0, 1.0), 1.0, drawn);
    return vec4(display(texel.rgb), coverage);
}
