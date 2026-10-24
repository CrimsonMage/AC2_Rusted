// Static objects. The vertex stage is the program `fragment.dll` links for a layer (fragment list
// from its compiler, fragment bodies from its vs.1.1 bytecode); the fragment stage is the D3D8
// fixed-function texture cascade `RenderDeviceD3D::SetupState` configures, then specular add, fog
// and the GREATER alpha test. Lighting dot products are taken in world space: the retail view
// transform is rigid, so they equal its view-space ones.

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
    // x: program (0 unlit, 1 lit, 3 lit vertex diffuse), y: flag bits, z: alpha reference
    // (256 = no alpha test), w: specular power as bits.
    program: vec4<u32>,
    // Per stage: color op, arg1, arg2, alpha op.
    color_ops: array<vec4<u32>, 4>,
    // Per stage: alpha arg1, arg2, texture coordinate source, has texture matrix.
    alpha_ops: array<vec4<u32>, 4>,
    // Per stage: texture matrix rows 0 and 1.
    matrices: array<vec4<f32>, 8>,
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

fn has(bit: u32) -> bool {
    return (draw.program.y & bit) != 0u;
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

// `f_CopyVertexTex*`, `f_CopyViewNormalReflectionToTex*`, `f_CopyClipPosToTex*`,
// `f_CopyViewPosToTex*`, then `f_ApplyTextureMatrixToTex*`.
fn stage_coord(input: VsIn, stage: u32, reflection: vec3<f32>, clip: vec4<f32>, view_pos: vec3<f32>) -> vec3<f32> {
    let source = draw.alpha_ops[stage].z;
    var t = vec4(0.0, 0.0, 0.0, 1.0);
    if source < 8u {
        t = vec4(vertex_uv(input, source), 0.0, 1.0);
    } else if source == 0x100u {
        t = vec4(reflection * 0.5 + 0.5, 1.0);
    } else if source == 0x300u {
        t = vec4(clip.xyz / clip.z, 1.0);
    } else if source == 0x400u {
        t = vec4(view_pos.xy / view_pos.z, 0.0, 1.0);
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

@vertex
fn vs(input: VsIn) -> VsOut {
    var out: VsOut;
    let world = draw.model * vec4(input.position, 1.0);
    out.clip = frame.view_proj * world;
    let view_pos = (frame.view * world).xyz;
    // `f_TransformModelNormalToViewNormal`: the model matrix itself, then normalise.
    let normal = normalize((draw.model * vec4(input.normal, 0.0)).xyz);
    // `f_CalculateViewVertexToEyeVector`, `f_CalculateViewNormalReflectionVector` in view space.
    let view_normal = normalize((frame.view * vec4(normal, 0.0)).xyz);
    let to_eye = -normalize(view_pos);
    let reflection = normalize(view_normal * -2.0 * dot(-to_eye, view_normal) - to_eye);
    // `f_CalculateLinearFog`: (end - |view position|) / (end - start), clamped at output.
    let distance = length(view_pos);
    let fog = clamp((frame.fog.y - distance) * frame.fog.z, 0.0, 1.0);

    let program = draw.program.x;
    let power = bitcast<f32>(draw.program.w);
    let sun_on = frame.sun_inv_dir.w > 0.0 && !has(FLAG_IGNORE_DIRECTIONAL);
    let l = frame.sun_inv_dir.xyz;
    let h = normalize(l + frame.to_viewer.xyz);
    let intensity = lit(dot(normal, l), dot(normal, h), power);
    // `c_AmbientColor`: (device ambient + light ambient) * material ambient, capped at 1.
    let ambient = min(frame.ambient.rgb * draw.ambient.rgb, vec3(1.0));

    var color0 = vec4(1.0);
    var color1 = vec4(0.0);
    if program == 1u {
        color0 = vec4(0.0, 0.0, 0.0, 1.0);
        if sun_on {
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
        // Vertex diffuse with dynamic lighting: colour 1 carries the light colour, no ambient.
        color0 = select(input.color, vec4(0.0, 0.0, 0.0, 1.0), has(FLAG_MODULATE));
        color1 = vec4(frame.sun_diffuse.rgb, 1.0);
        if sun_on {
            color0 = vec4(color0.rgb + intensity.x * frame.sun_diffuse.rgb, color0.a);
        }
        if has(FLAG_SPECULAR) {
            color1 = vec4(select(0.0, intensity.y, sun_on) * frame.sun_specular.rgb * draw.specular.rgb, 0.0);
        }
        if has(FLAG_MODULATE) {
            color0 = color0 * input.color;
        }
    } else {
        if has(FLAG_DEPTH_FOG) && has(FLAG_DIFFUSE_FOG_AMOUNT) {
            color0.a = color0.a * fog;
        }
        if has(FLAG_MATERIAL_DIFFUSE) {
            color0 = color0 * draw.diffuse;
        }
        if has(FLAG_VERTEX_DIFFUSE) {
            color0 = color0 * input.color;
        }
    }
    out.color0 = clamp(color0, vec4(0.0), vec4(1.0));
    out.color1 = clamp(color1, vec4(0.0), vec4(1.0));
    out.t0 = stage_coord(input, 0u, reflection, out.clip, view_pos);
    out.t1 = stage_coord(input, 1u, reflection, out.clip, view_pos);
    out.t2 = stage_coord(input, 2u, reflection, out.clip, view_pos);
    out.t3 = stage_coord(input, 3u, reflection, out.clip, view_pos);
    out.fog = fog;
    return out;
}

// `D3DTA_*` with the COMPLEMENT (0x10) and ALPHAREPLICATE (0x20) modifiers.
fn argument(arg: u32, current: vec4<f32>, diffuse: vec4<f32>, specular: vec4<f32>, texel: vec4<f32>) -> vec4<f32> {
    var v = current;
    switch arg & 0xFu {
        case 0u: { v = diffuse; }
        case 1u: { v = current; }
        case 2u: { v = texel; }
        case 3u: { v = draw.texture_factor; }
        case 4u: { v = specular; }
        default: { v = vec4(0.0); }
    }
    if (arg & 0x10u) != 0u {
        v = 1.0 - v;
    }
    if (arg & 0x20u) != 0u {
        v = vec4(v.a);
    }
    return v;
}

// `D3DTEXTUREOP` (`TextureOp`) on one channel group; alphas for the BLEND*ALPHA ops.
fn combine(op: u32, a1: vec4<f32>, a2: vec4<f32>, current: vec4<f32>, diffuse: vec4<f32>, texel: vec4<f32>) -> vec4<f32> {
    switch op {
        case 2u: { return a1; }
        case 3u: { return a2; }
        case 4u: { return a1 * a2; }
        case 5u: { return a1 * a2 * 2.0; }
        case 6u: { return a1 * a2 * 4.0; }
        case 7u: { return a1 + a2; }
        case 8u: { return a1 + a2 - 0.5; }
        case 9u: { return (a1 + a2 - 0.5) * 2.0; }
        case 10u: { return a1 - a2; }
        case 11u: { return a1 + a2 - a1 * a2; }
        case 12u: { return mix(a2, a1, diffuse.a); }
        case 13u: { return mix(a2, a1, texel.a); }
        case 14u: { return mix(a2, a1, draw.texture_factor.a); }
        case 15u: { return a1 + a2 * (1.0 - texel.a); }
        case 16u: { return mix(a2, a1, current.a); }
        case 18u: { return vec4(a1.rgb + a1.a * a2.rgb, a1.a); }
        case 19u: { return vec4(a1.rgb * a2.rgb + a1.a, a1.a); }
        case 20u: { return vec4((1.0 - a1.a) * a2.rgb + a1.rgb, a1.a); }
        case 21u: { return vec4((1.0 - a1.rgb) * a2.rgb + a1.a, a1.a); }
        case 24u: {
            return vec4(saturate(dot((a1.rgb - 0.5) * 2.0, (a2.rgb - 0.5) * 2.0)));
        }
        default: { return a1; }
    }
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
    let diffuse = input.color0;
    let specular = input.color1;
    // Sample every stage up front: texture sampling needs uniform control flow.
    let texels = array<vec4<f32>, 4>(
        sample_stage(0u, input),
        sample_stage(1u, input),
        sample_stage(2u, input),
        sample_stage(3u, input),
    );
    var current = diffuse;
    for (var stage = 0u; stage < 4u; stage = stage + 1u) {
        let ops = draw.color_ops[stage];
        let alpha = draw.alpha_ops[stage];
        if ops.x <= 1u {
            break;
        }
        let texel = texels[stage];
        let c = combine(ops.x, argument(ops.y, current, diffuse, specular, texel),
            argument(ops.z, current, diffuse, specular, texel), current, diffuse, texel);
        var a = current.a;
        if ops.w > 1u {
            a = combine(ops.w, argument(alpha.x, current, diffuse, specular, texel),
                argument(alpha.y, current, diffuse, specular, texel), current, diffuse, texel).a;
        }
        current = clamp(vec4(c.rgb, a), vec4(0.0), vec4(1.0));
    }
    var color = current.rgb;
    if has(FLAG_SPECULAR_ENABLE) {
        color = saturate(color + specular.rgb);
    }
    if has(FLAG_FOG) {
        color = mix(frame.fog_color.rgb, color, input.fog);
    }
    let reference = draw.program.z;
    if reference < 256u && !(round(current.a * 255.0) > f32(reference)) {
        discard;
    }
    return vec4(color, current.a);
}
