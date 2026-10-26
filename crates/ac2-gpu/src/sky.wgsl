// The sky under the DAT sky objects, in linear light: single-scattering Rayleigh and Mie
// atmosphere lit by the region's sun, graded by the backdrop dome's authored tint, with a
// procedural cloud layer drifting overhead. Drawn first and depth-free, so the DAT sky objects
// (sun, glow band, night layers, the dome's wisps) and the world draw over it. A deliberate
// deviation (AGENTS.md: one modern shading pass); fog never reaches it, the horizon meets the
// region's fog colour instead.

struct SkyFrame {
    inv_view_proj: mat4x4<f32>,
    eye: vec4<f32>,
    // The region's fog colour (the horizon), encoded.
    fog: vec4<f32>,
    // Unit vector towards the sun; sun_color is its linear colour (0 when the sun is off).
    sun_dir: vec4<f32>,
    sun_color: vec4<f32>,
    // The backdrop dome's tint (`UpdateTimeOfDay`'s colour, linear) and opacity; a 0 opacity
    // leaves the atmosphere ungraded.
    grade: vec4<f32>,
    // x cloud cover (0 clear to 1 overcast), y client seconds for the drift.
    clouds: vec4<f32>,
};

@group(0) @binding(0) var<uniform> sky: SkyFrame;

// Earth's atmosphere in metres (AC2 world units are metres).
const PLANET_RADIUS: f32 = 6360000.0;
const ATMOSPHERE_RADIUS: f32 = 6420000.0;
const RAYLEIGH: vec3<f32> = vec3<f32>(5.8e-6, 13.5e-6, 33.1e-6);
const RAYLEIGH_HEIGHT: f32 = 8000.0;
const MIE: f32 = 21e-6;
const MIE_HEIGHT: f32 = 1200.0;
const MIE_G: f32 = 0.76;
const VIEW_SAMPLES: i32 = 12;
const LIGHT_SAMPLES: i32 = 4;
// The sun's radiance into the scattering integral (the scene's exposure is 1).
const SUN_INTENSITY: f32 = 18.0;
// How far the dome's tint moves the atmosphere's hue.
const GRADE_STRENGTH: f32 = 0.35;
// The cloud deck: height above the eye, the noise's feature size and drift speed (metres, m/s).
const CLOUD_HEIGHT: f32 = 1600.0;
const CLOUD_SCALE: f32 = 2400.0;
const CLOUD_WIND: vec2<f32> = vec2<f32>(9.0, 3.5);
const CLOUD_FADE: f32 = 16000.0;
const PI: f32 = 3.14159265;

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) ndc: vec2<f32>,
};

@vertex
fn vs(@builtin(vertex_index) index: u32) -> VsOut {
    let uv = vec2(f32((index << 1u) & 2u), f32(index & 2u));
    var out: VsOut;
    out.ndc = uv * 2.0 - 1.0;
    out.clip = vec4(out.ndc, 1.0, 1.0);
    return out;
}

// Distance along `dir` from `origin` to the sphere of `radius` about the planet's centre.
fn sphere_exit(origin: vec3<f32>, dir: vec3<f32>, radius: f32) -> f32 {
    let b = dot(origin, dir);
    let c = dot(origin, origin) - radius * radius;
    return -b + sqrt(max(b * b - c, 0.0));
}

fn rayleigh_phase(mu: f32) -> f32 {
    return 3.0 / (16.0 * PI) * (1.0 + mu * mu);
}

fn mie_phase(mu: f32, g: f32) -> f32 {
    let g2 = g * g;
    return 3.0 / (8.0 * PI) * ((1.0 - g2) * (1.0 + mu * mu))
        / ((2.0 + g2) * pow(1.0 + g2 - 2.0 * g * mu, 1.5));
}

// Single scattering along a view ray from the eye (Nishita): Rayleigh and Mie in-scatter of the
// sun, attenuated towards the sun and back to the eye.
fn atmosphere(dir: vec3<f32>, sun: vec3<f32>, sun_radiance: vec3<f32>) -> vec3<f32> {
    let origin = vec3(0.0, 0.0, PLANET_RADIUS + clamp(sky.eye.z, 1.0, 4000.0));
    let ray_length = sphere_exit(origin, dir, ATMOSPHERE_RADIUS);
    let step = ray_length / f32(VIEW_SAMPLES);
    var optical_r = 0.0;
    var optical_m = 0.0;
    var sum_r = vec3(0.0);
    var sum_m = vec3(0.0);
    for (var i = 0; i < VIEW_SAMPLES; i = i + 1) {
        let p = origin + dir * (step * (f32(i) + 0.5));
        let height = length(p) - PLANET_RADIUS;
        let density_r = exp(-height / RAYLEIGH_HEIGHT) * step;
        let density_m = exp(-height / MIE_HEIGHT) * step;
        optical_r += density_r;
        optical_m += density_m;
        let light_length = sphere_exit(p, sun, ATMOSPHERE_RADIUS);
        let light_step = light_length / f32(LIGHT_SAMPLES);
        var light_r = 0.0;
        var light_m = 0.0;
        var lit = true;
        for (var j = 0; j < LIGHT_SAMPLES; j = j + 1) {
            let q = p + sun * (light_step * (f32(j) + 0.5));
            let h = length(q) - PLANET_RADIUS;
            if h < 0.0 {
                lit = false;
                break;
            }
            light_r += exp(-h / RAYLEIGH_HEIGHT) * light_step;
            light_m += exp(-h / MIE_HEIGHT) * light_step;
        }
        if lit {
            let tau = RAYLEIGH * (optical_r + light_r) + MIE * 1.1 * (optical_m + light_m);
            let attenuation = exp(-tau);
            sum_r += density_r * attenuation;
            sum_m += density_m * attenuation;
        }
    }
    let mu = dot(dir, sun);
    return sun_radiance * (sum_r * RAYLEIGH * rayleigh_phase(mu) + sum_m * MIE * mie_phase(mu, MIE_G));
}

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

@fragment
fn fs(input: VsOut) -> @location(0) vec4<f32> {
    let far = sky.inv_view_proj * vec4(input.ndc, 1.0, 1.0);
    let view = normalize(far.xyz / far.w - sky.eye.xyz);
    // Below the horizon the world covers the sky; keep the horizon's colour there.
    let dir = normalize(vec3(view.xy, max(view.z, 0.0)));
    let sun = sky.sun_dir.xyz;
    // The region's directional light (the sun by day, the moon's dim light by night).
    let sun_radiance = SUN_INTENSITY * clamp(sky.sun_color.rgb, vec3(0.0), vec3(2.0));
    var color = atmosphere(dir, sun, sun_radiance);

    // The dome's authored tint grades the hue (its opacity says how much it shows).
    let grade = sky.grade.rgb / max(dot(sky.grade.rgb, vec3(0.2126, 0.7152, 0.0722)), 1e-3);
    color *= mix(vec3(1.0), clamp(grade, vec3(0.0), vec3(2.0)), GRADE_STRENGTH * sky.grade.a);

    // The horizon meets the region's fog, as the fogged terrain does.
    let horizon = srgb_to_linear(clamp(sky.fog.rgb, vec3(0.0), vec3(1.0)));
    let level = max(dot(color, vec3(0.2126, 0.7152, 0.0722)), 1e-3);
    color = mix(color, horizon * level * 1.4, exp(-dir.z * 9.0) * 0.55);

    // The cloud deck, lit by the sun through it and by the sky from above.
    let cover = clamp(sky.clouds.x, 0.0, 1.0);
    if view.z > 0.0 && cover > 0.0 {
        let distance = CLOUD_HEIGHT / max(view.z, 0.01);
        let at = (sky.eye.xy + view.xy * distance) / CLOUD_SCALE + CLOUD_WIND * sky.clouds.y / CLOUD_SCALE;
        let density = cloud_density(at, cover);
        let toward_sun = cloud_density(at + sun.xy * 0.06, cover);
        let shadow = exp(-2.2 * toward_sun);
        let zenith = atmosphere(vec3(0.0, 0.0, 1.0), sun, sun_radiance);
        let sunlight = sky.sun_color.rgb * max(sun.z, 0.0) * 1.6 + sky.sun_color.rgb * 0.15;
        let silver = mie_phase(dot(view, sun), 0.55) * 2.0;
        let lit = sunlight * (0.55 * shadow + silver * shadow) + zenith * 1.3 + horizon * 0.08;
        let fade = exp(-distance / CLOUD_FADE) * smoothstep(0.03, 0.3, view.z);
        color = mix(color, lit, density * fade);
    }
    return vec4(max(color, vec3(0.0)), 1.0);
}
