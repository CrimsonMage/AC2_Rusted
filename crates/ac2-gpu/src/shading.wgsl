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

// The procedural cloud deck the sky draws (`sky.wgsl`) and the landscape's cloud shadows sample
// (`terrain.wgsl`): height above the eye, the noise's feature size and drift (metres, m/s).
const CLOUD_HEIGHT: f32 = 1600.0;
const CLOUD_SCALE: f32 = 2400.0;
const CLOUD_WIND: vec2<f32> = vec2<f32>(9.0, 3.5);
const CLOUD_FADE: f32 = 16000.0;

fn hash(p: vec2<f32>) -> f32 {
    let q = fract(p * vec2(0.1031, 0.1030));
    let r = q + dot(q, q.yx + 33.33);
    return fract((r.x + r.y) * r.x);
}

fn value_noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    return mix(
        mix(hash(i), hash(i + vec2(1.0, 0.0)), u.x),
        mix(hash(i + vec2(0.0, 1.0)), hash(i + vec2(1.0, 1.0)), u.x),
        u.y,
    );
}

fn fbm(p: vec2<f32>) -> f32 {
    var sum = 0.0;
    var amplitude = 0.5;
    var q = p;
    for (var i = 0; i < 6; i = i + 1) {
        sum += amplitude * value_noise(q);
        q = q * 2.03 + vec2(17.1, 9.7);
        amplitude *= 0.5;
    }
    return sum;
}

// The cloud deck's coverage at a point of the deck (0 clear sky, 1 solid cloud).
fn cloud_density(at: vec2<f32>, cover: f32) -> f32 {
    let n = fbm(at);
    let threshold = 1.0 - 0.45 - 0.45 * cover;
    return smoothstep(threshold, threshold + 0.22, n);
}

// How much of the sun reaches world point `p` through the cloud deck (1 clear, down to 0.35
// under thick cloud): the deck sampled where the ray to the sun crosses it. `clouds` is the sky's
// x cover, y drift seconds; `sun` points to the sun; `eye_z` places the deck. Retail's terrain
// cloud-shadow stage (`LScapeMaterialInfo::CombineMaterials`, texcoord 6) samples a region
// texture through coordinates nothing writes; this follows the drawn clouds instead.
fn cloud_shadow(p: vec3<f32>, sun: vec3<f32>, eye_z: f32, clouds: vec4<f32>) -> f32 {
    let cover = clamp(clouds.x, 0.0, 1.0);
    if cover <= 0.0 || sun.z <= 0.05 {
        return 1.0;
    }
    let deck = eye_z + CLOUD_HEIGHT;
    let at_deck = p.xy + sun.xy / sun.z * (deck - p.z);
    let at = at_deck / CLOUD_SCALE + CLOUD_WIND * clouds.y / CLOUD_SCALE;
    let density = cloud_density(at, cover);
    return 1.0 - 0.65 * density * smoothstep(0.05, 0.25, sun.z);
}
