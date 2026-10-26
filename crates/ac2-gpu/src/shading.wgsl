// Shading helpers shared by the object and terrain shaders (`shading.rs` mirrors them). Colour
// inputs are sRGB-encoded in the data: they are decoded, lit and fogged in linear light, and the
// HDR scene is encoded once by `tonemap.wgsl`.

const MODERN_PI: f32 = 3.14159265;

fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let low = c / 12.92;
    let high = pow((c + 0.055) / 1.055, vec3(2.4));
    return select(high, low, c <= vec3(0.04045));
}

fn linear_to_srgb(c: vec3<f32>) -> vec3<f32> {
    let low = c * 12.92;
    let high = 1.055 * pow(max(c, vec3(0.0)), vec3(1.0 / 2.4)) - 0.055;
    return select(high, low, c <= vec3(0.0031308));
}

// An authored light colour in linear light: clamped to [0, 1] in display space first, as retail's
// lighting is.
fn light_linear(light: vec3<f32>) -> vec3<f32> {
    return srgb_to_linear(clamp(light, vec3(0.0), vec3(1.0)));
}

// The modern sky gradient in linear light: the region's fog colour (the horizon) rising to a deeper
// zenith. The sky pass and the DAT sky objects (`FLAG_SKY`) both take their colour from here, so
// clouds and the sun sprites sit on the same sky. `dir` is the unit view direction, Z up.
const SKY_GRADIENT_POWER: f32 = 0.6;
const SKY_ZENITH_TINT: vec3<f32> = vec3<f32>(0.12, 0.35, 0.95);

fn sky_color(dir: vec3<f32>, fog_color: vec3<f32>) -> vec3<f32> {
    let horizon = fog_linear(fog_color);
    let up = clamp(dir.z, 0.0, 1.0);
    return mix(horizon, horizon * SKY_ZENITH_TINT, pow(up, SKY_GRADIENT_POWER));
}

// The region's fog colour in linear light. The back buffer clamps it to [0, 1] as retail does, so
// an HDR fog colour does not blow the haze out.
fn fog_linear(fog_color: vec3<f32>) -> vec3<f32> {
    return srgb_to_linear(clamp(fog_color, vec3(0.0), vec3(1.0)));
}

// Modern fog: the fogged fraction (0 clear, 1 fully fogged) of a point `distance` metres from the
// eye and `height` metres above the world's zero, over the region's linear fog `range` (start,
// end). Retail's distance ramp, thinned exponentially with altitude, so high ground and the sky
// above the horizon stay clear. Only terrain and objects fog; the sky never does.
const MODERN_FOG_HEIGHT_FALLOFF: f32 = 0.01;

fn modern_fog(distance: f32, height: f32, range: vec2<f32>) -> f32 {
    let span = max(range.y - range.x, 1e-3);
    let ramp = clamp((distance - range.x) / span, 0.0, 1.0);
    return ramp * exp(-MODERN_FOG_HEIGHT_FALLOFF * max(height, 0.0));
}

// GGX normal distribution (Trowbridge-Reitz); `a` is roughness squared.
fn ggx_distribution(n_dot_h: f32, a: f32) -> f32 {
    let a2 = a * a;
    let d = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;
    return a2 / (MODERN_PI * d * d);
}

// Smith visibility term V = G / (4 N.L N.V), height-correlated.
fn smith_visibility(n_dot_v: f32, n_dot_l: f32, a: f32) -> f32 {
    let a2 = a * a;
    let gv = n_dot_l * sqrt(n_dot_v * n_dot_v * (1.0 - a2) + a2);
    let gl = n_dot_v * sqrt(n_dot_l * n_dot_l * (1.0 - a2) + a2);
    return 0.5 / max(gv + gl, 1e-5);
}

// Schlick Fresnel.
fn fresnel_schlick(v_dot_h: f32, f0: vec3<f32>) -> vec3<f32> {
    let f = pow(1.0 - clamp(v_dot_h, 0.0, 1.0), 5.0);
    return f0 + (vec3(1.0) - f0) * f;
}
