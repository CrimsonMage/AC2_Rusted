// The one shading of objects (`object.wgsl`'s fragment). Lit program 1 objects
// leave their light out of the vertex stage: the texture cascade runs on the albedo alone, and the
// fragment lights it per pixel with a Lambert diffuse, GGX specular (Smith visibility, Schlick
// Fresnel, dielectric F0 0.04, metallic 0) and a sky-over-ground ambient. Roughness comes from
// the material's retail specular power. Pixel-shader layers (programs 2 and 4) keep their
// authored maths and are only linearised for the output. Fog and tone mapping follow in linear.

// Scales the sun and point light colours. Retail multiplies albedo by the light directly (no
// 1 / pi), so this is 1 and `frame.shading` exposure does the rest.
const MODERN_SUN_SCALE: f32 = 1.0;
const MODERN_AMBIENT_SCALE: f32 = 1.0;
// The ground bounce under the sky colour, a warm earth tone.
const MODERN_GROUND_TINT: vec3<f32> = vec3<f32>(0.55, 0.5, 0.42);
const MODERN_ROUGHNESS_MIN: f32 = 0.45;
const MODERN_ROUGHNESS_MAX: f32 = 0.8;
const MODERN_F0: f32 = 0.04;
// The backdrop dome's detail (its texel above the local average) as haze over the atmosphere.
const BACKDROP_DETAIL: f32 = 2.5;
// Point lights in an additive sky layer (texels far above their local average: the stars) shine
// this much brighter in the HDR scene, so they read through the night's glow band over them.
const STAR_GAIN: f32 = 24.0;

// The lit light of one pixel: diffuse irradiance (multiplies the albedo) and specular radiance.
struct ModernLight {
    diffuse: vec3<f32>,
    specular: vec3<f32>,
};

// Program 1 takes its albedo (the material diffuse) in colour 0 and no vertex light or specular.
fn modern_lit_albedo(program: u32, vertex_color: vec4<f32>) -> vec4<f32> {
    return select(vec4(1.0), draw.diffuse, has(FLAG_MATERIAL_DIFFUSE));
}

// Roughness from the retail specular power (a Blinn lobe of power p is about GGX roughness
// sqrt(2 / (p + 2)); materials with no power sit at the rough end).
fn modern_roughness() -> f32 {
    let power = max(bitcast<f32>(draw.program.w), 0.0);
    return clamp(sqrt(2.0 / (power + 2.0)), MODERN_ROUGHNESS_MIN, MODERN_ROUGHNESS_MAX);
}

// Lambert and GGX terms of one light: (1 - F) N.L for diffuse, D V F N.L for specular.
fn modern_direct(n: vec3<f32>, v: vec3<f32>, l: vec3<f32>, alpha: f32) -> ModernLight {
    let n_dot_l = dot(n, l);
    var term: ModernLight;
    term.diffuse = vec3(0.0);
    term.specular = vec3(0.0);
    if n_dot_l <= 0.0 {
        return term;
    }
    let h = normalize(l + v);
    let n_dot_h = max(dot(n, h), 0.0);
    let v_dot_h = max(dot(v, h), 0.0);
    let n_dot_v = max(dot(n, v), 1e-4);
    let f = fresnel_schlick(v_dot_h, vec3(MODERN_F0));
    term.diffuse = (vec3(1.0) - f) * n_dot_l;
    term.specular = ggx_distribution(n_dot_h, alpha) * smith_visibility(n_dot_v, n_dot_l, alpha)
        * f * n_dot_l;
    return term;
}

// Every light on a pixel: the sky over the ground (`frame.ambient` times the material ambient,
// the region's sky colour from above and its ground bounce from below), the sun, and the
// device's point lights, each with its colour decoded from sRGB.
fn modern_light(world: vec3<f32>, n: vec3<f32>) -> ModernLight {
    let v = normalize(frame.eye.xyz - world);
    let alpha = modern_roughness() * modern_roughness();
    // Only materials retail lights with a highlight (the specular flag) get one, tinted by their
    // specular colour; the rest stay matte, as the DAT authored them.
    let tint = select(vec3(0.0), light_linear(draw.specular.rgb), has(FLAG_SPECULAR));
    // Light colours are authored in display space like the surfaces, so they decode too; retail
    // clamps them to 1 there, and so does the decode.
    let sky = light_linear(min(frame.ambient.rgb * draw.ambient.rgb, vec3(1.0)));
    var light: ModernLight;
    light.diffuse = mix(sky * MODERN_GROUND_TINT, sky, n.z * 0.5 + 0.5) * MODERN_AMBIENT_SCALE;
    light.specular = vec3(0.0);
    if sun_on() {
        let l = normalize(frame.sun_inv_dir.xyz);
        let term = modern_direct(n, v, l, alpha);
        let e = light_linear(frame.sun_diffuse.rgb) * MODERN_SUN_SCALE;
        let es = light_linear(frame.sun_specular.rgb) * MODERN_SUN_SCALE;
        light.diffuse = light.diffuse + e * term.diffuse;
        light.specular = light.specular + es * tint * term.specular;
    }
    for (var i = 0u; i < point_count(); i = i + 1u) {
        let to_light = draw.points[i * 4u].xyz - world;
        let distance_sq = dot(to_light, to_light);
        let attenuation = point_attenuation(i, distance_sq);
        let l = to_light * inverseSqrt(max(distance_sq, 1e-30));
        let term = modern_direct(n, v, l, alpha);
        // Retail sums the attenuated light colour in display space; decode that sum.
        let e = light_linear(draw.points[i * 4u + 1u].rgb * attenuation);
        let es = light_linear(draw.points[i * 4u + 2u].rgb * attenuation);
        light.diffuse = light.diffuse + e * term.diffuse;
        light.specular = light.specular + es * tint * term.specular;
    }
    return light;
}

// The modern fragment. The cascade runs on the authored gamma values, exactly as retail does:
// its ops (ADDSIGNED, SUBTRACT, DOTPRODUCT3 ...) assume the 0.5 neutral point of encoded texels,
// so decoding the inputs first garbles them. The cascade's output is then decoded to linear light
// and lit there (lit programs multiply their linear albedo by the linear light); fog and the
// alpha test follow; the scene's tone map encodes the result.
fn shade_modern(input: VsOut, texels: array<vec4<f32>, 4>, coarse: vec4<f32>) -> vec4<f32> {
    if has(FLAG_SKY_BACKDROP) {
        // The dome's retail cascade saturates to white by day; the atmosphere (`sky.wgsl`) is the
        // sky's colour instead, and the dome adds the wisps of its texture in its authored tint
        // (`texture_factor`: the time of day's colour and opacity).
        let luma = vec3(0.2126, 0.7152, 0.0722);
        let detail = max(dot(texels[0].rgb, luma) - dot(coarse.rgb, luma), 0.0);
        let tint = draw.texture_factor;
        return vec4(srgb_to_linear(tint.rgb) * detail * BACKDROP_DETAIL, tint.a);
    }
    let program = draw.program.x;
    let pixel_program = program == 2u || program == 4u;
    let shaded = shade(input, texels);
    var radiance = srgb_to_linear(clamp(shaded.rgb, vec3(0.0), vec3(1.0)));
    if !pixel_program && program == 1u {
        let light = modern_light(input.world, normalize(input.normal));
        // Retail clamps the specular colour to [0, 1] (`out.color1`); the modern highlights keep
        // the tone curve's roll-off above that.
        radiance = radiance * light.diffuse + clamp(light.specular, vec3(0.0), vec3(1.0));
    }
    var color = radiance;
    if has(FLAG_SKY_EMISSIVE) {
        let luma = vec3(0.2126, 0.7152, 0.0722);
        let sparkle = smoothstep(0.04, 0.35, dot(texels[0].rgb, luma) - 2.0 * dot(coarse.rgb, luma));
        color *= mix(1.0, STAR_GAIN, sparkle);
    }
    if has(FLAG_FOG) {
        color = mix(fog_linear(frame.fog_color.rgb), color, 1.0 - input.haze);
    }
    let reference = draw.program.z;
    if reference < 256u && !(round(clamp(shaded.a, 0.0, 1.0) * 255.0) > f32(reference)) {
        discard;
    }
    // Linear light into the HDR scene; the scene's tone map follows.
    return vec4(max(color, vec3(0.0)), shaded.a);
}
