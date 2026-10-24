// Static objects. The vertex stage is the program `fragment.dll` links for a layer (fragment list
// from its compiler, fragment bodies from its vs.1.1 bytecode). The fragment stage samples the
// four stages and calls `shade`, which is either the D3D8 fixed-function texture cascade
// (`object_stages.wgsl`) or the layer's authored ps.1.1 shader translated to WGSL; fog and the
// GREATER alpha test follow. Lighting dot products are taken in world space: the retail view
// transform is rigid, so they equal its view-space ones, and tangent-space vectors are the same.

struct Frame {
    view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    eye: vec4<f32>,
    // Unit vector from the scene towards the viewer (the view-space (0, 0, -1) of the half vector).
    to_viewer: vec4<f32>,
    // `c_DirLight_ViewInvDirection`: -direction; w = 1 when the sun is on the device.
    sun_inv_dir: vec4<f32>,
    sun_diffuse: vec4<f32>,
    sun_specular: vec4<f32>,
    // `RenderDeviceD3D::SetAmbient` colour.
    ambient: vec4<f32>,
    fog_color: vec4<f32>,
    // start, end, 1 / (end - start)
    fog: vec4<f32>,
};

struct Draw {
    model: mat4x4<f32>,
    ambient: vec4<f32>,
    diffuse: vec4<f32>,
    specular: vec4<f32>,
    texture_factor: vec4<f32>,
    // x: program (0 unlit, 1 lit, 2 pixel lit, 3 lit vertex diffuse, 4 pixel lit multiple),
    // y: flag bits, z: alpha reference (256 = no alpha test), w: specular power as bits.
    program: vec4<u32>,
    // Per stage: color op, arg1, arg2, alpha op.
    color_ops: array<vec4<u32>, 4>,
    // Per stage: alpha arg1, arg2, texture coordinate source, has texture matrix.
    alpha_ops: array<vec4<u32>, 4>,
    // Per stage: texture matrix rows 0 and 1.
    matrices: array<vec4<f32>, 8>,
    // A frill's world normal for every vertex when w = 1 (`FrillRender::RefreshFrillMesh`).
    frill: vec4<f32>,
};

const FLAG_MATERIAL_DIFFUSE: u32 = 1u;
const FLAG_VERTEX_DIFFUSE: u32 = 2u;
const FLAG_MODULATE: u32 = 4u;
const FLAG_SPECULAR: u32 = 8u;
const FLAG_FOG: u32 = 16u;
const FLAG_DIFFUSE_FOG_AMOUNT: u32 = 32u;
const FLAG_ALPHA_FOG: u32 = 64u;
const FLAG_IGNORE_DIRECTIONAL: u32 = 128u;
const FLAG_SPECULAR_ENABLE: u32 = 256u;
const FLAG_DEPTH_FOG: u32 = 512u;
const FLAG_PIXEL_SPECULAR: u32 = 1024u;
const FLAG_UNMODULATED_COLOR1: u32 = 2048u;
const FLAG_DYNAMIC: u32 = 4096u;

@group(0) @binding(0) var<uniform> frame: Frame;
@group(1) @binding(0) var<uniform> draw: Draw;
@group(2) @binding(0) var tex0: texture_2d<f32>;
@group(2) @binding(1) var tex1: texture_2d<f32>;
@group(2) @binding(2) var tex2: texture_2d<f32>;
@group(2) @binding(3) var tex3: texture_2d<f32>;
@group(2) @binding(4) var samp0: sampler;
@group(2) @binding(5) var samp1: sampler;
@group(2) @binding(6) var samp2: sampler;
@group(2) @binding(7) var samp3: sampler;

struct VsIn {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
    @location(3) uv0: vec2<f32>,
    @location(4) uv1: vec2<f32>,
    @location(5) uv2: vec2<f32>,
    @location(6) uv3: vec2<f32>,
    @location(7) tangent_s: vec3<f32>,
    @location(8) tangent_t: vec3<f32>,
};

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) color0: vec4<f32>,
    @location(1) color1: vec4<f32>,
    @location(2) t0: vec3<f32>,
    @location(3) t1: vec3<f32>,
    @location(4) t2: vec3<f32>,
    @location(5) t3: vec3<f32>,
    @location(6) fog: f32,
};

/// Per-vertex vectors the fragments share, in world space.
struct Basis {
    normal: vec3<f32>,
    tangent_s: vec3<f32>,
    tangent_t: vec3<f32>,
    // `r_ViewVertexToEye`.
    to_eye: vec3<f32>,
    // `r_BumpLight_ViewInvDirection`: the sun's inverse direction, or towards the zero-filled
    // point light the (0, 1) program falls back to, which sits at the eye.
    bump_light: vec3<f32>,
};

fn has(bit: u32) -> bool {
    return (draw.program.y & bit) != 0u;
}

fn sun_on() -> bool {
    return frame.sun_inv_dir.w > 0.0 && !has(FLAG_IGNORE_DIRECTIONAL);
}

fn vertex_uv(input: VsIn, uv_set: u32) -> vec2<f32> {
    switch uv_set {
        case 0u: { return input.uv0; }
        case 1u: { return input.uv1; }
        case 2u: { return input.uv2; }
        case 3u: { return input.uv3; }
        default: { return vec2(0.0); }
    }
}

// A vector in the vertex's tangent space (`f_TransformBump*ToTex*`), normalised.
fn to_tangent_space(basis: Basis, v: vec3<f32>) -> vec3<f32> {
    let t = vec3(dot(basis.tangent_s, v), dot(basis.tangent_t, v), dot(basis.normal, v));
    return t * inverseSqrt(max(dot(t, t), 1e-30));
}

// `f_CopyVertexTex*`, `f_CopyViewNormalReflectionToTex*`, `f_CopyAnisotropicTexCoordToTex*`,
// `f_CopyClipPosToTex*`, `f_CopyViewPosToTex*`, `f_NormalizeTex*` of the point-to-light vector,
// `f_CopyTexLightHalfVectorToTex*`, then `f_ApplyTextureMatrixToTex*`.
fn stage_coord(input: VsIn, stage: u32, basis: Basis, reflection: vec3<f32>, clip: vec4<f32>, view_pos: vec3<f32>) -> vec3<f32> {
    let source = draw.alpha_ops[stage].z;
    var t = vec4(0.0, 0.0, 0.0, 1.0);
    if source < 8u {
        t = vec4(vertex_uv(input, source), 0.0, 1.0);
    } else if source == 0x100u {
        t = vec4(reflection * 0.5 + 0.5, 1.0);
    } else if source == 0x200u {
        let half_vector = normalize(basis.bump_light + basis.to_eye);
        t = vec4(dot(basis.bump_light, basis.normal), dot(half_vector, basis.normal), 1.0, 1.0);
    } else if source == 0x300u {
        t = vec4(clip.xyz / clip.z, 1.0);
    } else if source == 0x400u {
        t = vec4(view_pos.xy / view_pos.z, 0.0, 1.0);
    } else if source == 0x500u {
        // No point lights reach the device yet: the zero vector normalises to 0 (D3D's
        // 0 * inf = 0) and encodes as 0.5.
        t = vec4(0.5, 0.5, 0.5, 1.0);
    } else if source == 0x600u {
        let half_vector = to_tangent_space(basis, normalize(basis.to_eye + basis.bump_light));
        t = vec4(half_vector * 0.5 + 0.5, 1.0);
    }
    if draw.alpha_ops[stage].w != 0u {
        t = vec4(dot(t, draw.matrices[stage * 2u]), dot(t, draw.matrices[stage * 2u + 1u]), 0.0, 1.0);
    }
    return t.xyz;
}

// D3D `lit`: (1, max(n.l, 0), n.l > 0 && n.h > 0 ? n.h^power : 0, 1).
fn lit(n_dot_l: f32, n_dot_h: f32, power: f32) -> vec2<f32> {
    var specular = 0.0;
    if n_dot_l > 0.0 && n_dot_h > 0.0 {
        specular = pow(n_dot_h, clamp(power, -127.9961, 127.9961));
    }
    return vec2(max(n_dot_l, 0.0), specular);
}

fn has_stage_source(source: u32) -> bool {
    for (var stage = 0u; stage < 4u; stage = stage + 1u) {
        if draw.alpha_ops[stage].z == source {
            return true;
        }
    }
    return false;
}

@vertex
fn vs(input: VsIn) -> VsOut {
    var out: VsOut;
    let world = draw.model * vec4(input.position, 1.0);
    out.clip = frame.view_proj * world;
    let view_pos = (frame.view * world).xyz;
    var basis: Basis;
    // `f_TransformModel*ToView*`: the model matrix itself, then normalise.
    basis.normal = normalize((draw.model * vec4(input.normal, 0.0)).xyz);
    var vertex_color = input.color;
    if draw.frill.w > 0.5 {
        // Merged frill vertices: the frill's up axis, and the landscape shadow colour (white).
        basis.normal = normalize(draw.frill.xyz);
        vertex_color = vec4(1.0);
    }
    basis.tangent_s = normalize((draw.model * vec4(input.tangent_s, 0.0)).xyz);
    basis.tangent_t = normalize((draw.model * vec4(input.tangent_t, 0.0)).xyz);
    basis.to_eye = normalize(frame.eye.xyz - world.xyz);
    basis.bump_light = select(basis.to_eye, frame.sun_inv_dir.xyz, sun_on());
    // `f_CalculateViewNormalReflectionVector` in view space.
    let view_normal = normalize((frame.view * vec4(basis.normal, 0.0)).xyz);
    let view_to_eye = -normalize(view_pos);
    let reflection = normalize(view_normal * -2.0 * dot(-view_to_eye, view_normal) - view_to_eye);
    // `f_CalculateLinearFog`: (end - |view position|) / (end - start), clamped at output.
    let distance = length(view_pos);
    let fog = clamp((frame.fog.y - distance) * frame.fog.z, 0.0, 1.0);

    let program = draw.program.x;
    let power = bitcast<f32>(draw.program.w);
    let l = frame.sun_inv_dir.xyz;
    // `c_DirLight_ViewHalfVector`.
    let h = normalize(l + frame.to_viewer.xyz);
    let intensity = lit(dot(basis.normal, l), dot(basis.normal, h), power);
    // `c_AmbientColor`: (device ambient + light ambient) * material ambient, capped at 1.
    let ambient = min(frame.ambient.rgb * draw.ambient.rgb, vec3(1.0));

    var color0 = vec4(1.0);
    var color1 = vec4(0.0);
    if program == 1u {
        color0 = vec4(0.0, 0.0, 0.0, 1.0);
        if sun_on() {
            color0 = vec4(color0.rgb + intensity.x * frame.sun_diffuse.rgb, color0.a);
            if has(FLAG_SPECULAR) {
                color1 = vec4(intensity.y * frame.sun_specular.rgb, 0.0);
            }
        }
        if has(FLAG_MATERIAL_DIFFUSE) {
            color0 = color0 * draw.diffuse;
        }
        color0 = vec4(color0.rgb + ambient, color0.a);
        if has(FLAG_SPECULAR) {
            color1 = vec4(color1.rgb * draw.specular.rgb, color1.a);
        }
    } else if program == 3u {
        // Vertex diffuse with dynamic lighting: colour 1 carries the light colour; ambient only
        // when a stage takes the point-to-light vector.
        color0 = select(vertex_color, vec4(0.0, 0.0, 0.0, 1.0), has(FLAG_MODULATE));
        color1 = vec4(frame.sun_diffuse.rgb, 1.0);
        if sun_on() {
            color0 = vec4(color0.rgb + intensity.x * frame.sun_diffuse.rgb, color0.a);
        }
        if has(FLAG_SPECULAR) {
            color1 = vec4(select(0.0, intensity.y, sun_on()) * frame.sun_specular.rgb * draw.specular.rgb, 0.0);
        }
        if has_stage_source(0x500u) {
            color0 = vec4(color0.rgb + ambient, color0.a);
        }
        if has(FLAG_MODULATE) {
            color0 = color0 * vertex_color;
        }
    } else if program == 2u || program == 4u {
        // Per-pixel lighting: colour 0 is the tangent-space light direction with the light's
        // attenuation (1 for the sun) in alpha (`f_SetColor0ToTexLightDirection`,
        // `f_SetColor0AlphaToBumpLightAttenuation`).
        color0 = vec4(to_tangent_space(basis, basis.bump_light) * 0.5 + 0.5, 1.0);
        let specular_alpha = select(0.0, intensity.y * frame.sun_specular.r, sun_on());
        if program == 4u {
            // The first light is lit per pixel; its specular goes to colour 1 alpha unless the
            // pixel shader does specular (then alpha is attenuation * 0.75, `c0.w`).
            color1 = select(vec4(0.0, 0.0, 0.0, specular_alpha), vec4(0.0, 0.0, 0.0, 0.75), has(FLAG_PIXEL_SPECULAR));
            if !has(FLAG_UNMODULATED_COLOR1) {
                // `f_OutputPreModulatedColor1_Color1`.
                color1 = vec4(color1.rgb * draw.diffuse.rgb + ambient, color1.a);
            }
        } else if has(FLAG_SPECULAR) && !has(FLAG_PIXEL_SPECULAR) {
            color1 = vec4(select(0.0, intensity.y, sun_on()) * frame.sun_specular.rgb * draw.specular.rgb, 0.0);
        } else if !has(FLAG_PIXEL_SPECULAR) {
            color1 = vec4(frame.sun_diffuse.rgb, 1.0);
            if has(FLAG_MATERIAL_DIFFUSE) {
                color1 = color1 * draw.diffuse;
            }
        }
    } else {
        if has(FLAG_DEPTH_FOG) && has(FLAG_DIFFUSE_FOG_AMOUNT) {
            color0.a = color0.a * fog;
        }
        if has(FLAG_MATERIAL_DIFFUSE) {
            color0 = color0 * draw.diffuse;
        }
        if has(FLAG_VERTEX_DIFFUSE) {
            color0 = color0 * vertex_color;
        }
    }
    out.color0 = clamp(color0, vec4(0.0), vec4(1.0));
    out.color1 = clamp(color1, vec4(0.0), vec4(1.0));
    out.t0 = stage_coord(input, 0u, basis, reflection, out.clip, view_pos);
    out.t1 = stage_coord(input, 1u, basis, reflection, out.clip, view_pos);
    out.t2 = stage_coord(input, 2u, basis, reflection, out.clip, view_pos);
    out.t3 = stage_coord(input, 3u, basis, reflection, out.clip, view_pos);
    out.fog = fog;
    return out;
}

fn sample_stage(stage: u32, input: VsOut) -> vec4<f32> {
    switch stage {
        case 0u: { return textureSample(tex0, samp0, input.t0.xy); }
        case 1u: { return textureSample(tex1, samp1, input.t1.xy); }
        case 2u: { return textureSample(tex2, samp2, input.t2.xy); }
        default: { return textureSample(tex3, samp3, input.t3.xy); }
    }
}

@fragment
fn fs(input: VsOut) -> @location(0) vec4<f32> {
    // Sample every stage up front: texture sampling needs uniform control flow.
    let texels = array<vec4<f32>, 4>(
        sample_stage(0u, input),
        sample_stage(1u, input),
        sample_stage(2u, input),
        sample_stage(3u, input),
    );
    let shaded = shade(input, texels);
    var color = shaded.rgb;
    if has(FLAG_FOG) {
        color = mix(frame.fog_color.rgb, color, input.fog);
    }
    let reference = draw.program.z;
    if reference < 256u && !(round(clamp(shaded.a, 0.0, 1.0) * 255.0) > f32(reference)) {
        discard;
    }
    return vec4(color, shaded.a);
}
