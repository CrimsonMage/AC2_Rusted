// Near landscape. One pass computes what `LScapeMaterialInfo::CombineMaterials` draws in up to
// three: every layer's stage 0 is MODULATE2X(texture, diffuse); layer 0 blends ONE/ZERO, later
// layers SRCALPHA/INVSRCALPHA with alpha from their mask (stage 1 SELECTARG1 texture alpha).
// Fixed-function linear range fog (`FOGVERTEXMODE` LINEAR, `RANGEFOGENABLE`) applies per layer,
// which equals fogging the blend. With landscape shadows on, the blend is modulated by the static
// shadow map (texcoord 7, `SceneTool::StartShadowMap`) and the region's cloud-shadow texture
// (texcoord 6, scrolled by an `LM_UVTranslate`): stages 1 and 2 of a single layer, or a
// DESTCOLOR/ZERO pass after several, both before the fog.

struct Frame {
    view_proj: mat4x4<f32>,
    eye: vec4<f32>,
    // `m_finalLightDir`: light travel direction times direct brightness.
    light_dir: vec4<f32>,
    dir_color: vec4<f32>,
    ambient: vec4<f32>,
    fog_color: vec4<f32>,
    // start, end
    fog: vec4<f32>,
    // The cloud-shadow texture: xy its scroll offset, z its scale per metre, w 1 when loaded.
    clouds: vec4<f32>,
    // World to static shadow map coordinates (the texcoord 7 `StartShadowMap` writes).
    shadow: mat4x4<f32>,
};

struct Batch {
    // UV scale per layer: vertex spacing / tiling (`CalcUVInfo`).
    scale: vec4<f32>,
    // Mask rotation (`rot_types`, 1..4) of layers 1 and 2, then the layer count.
    rot: vec4<u32>,
};

@group(0) @binding(0) var<uniform> frame: Frame;
@group(0) @binding(1) var cloud_texture: texture_2d<f32>;
@group(0) @binding(2) var shadow_map: texture_2d<f32>;
@group(0) @binding(3) var frame_wrap: sampler;
@group(0) @binding(4) var frame_clamp: sampler;
@group(1) @binding(0) var<uniform> batch: Batch;
@group(1) @binding(1) var base0: texture_2d<f32>;
@group(1) @binding(2) var base1: texture_2d<f32>;
@group(1) @binding(3) var base2: texture_2d<f32>;
@group(1) @binding(4) var mask1: texture_2d<f32>;
@group(1) @binding(5) var mask2: texture_2d<f32>;
@group(1) @binding(6) var wrap_sampler: sampler;
@group(1) @binding(7) var clamp_sampler: sampler;

struct VsIn {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    // `SurfaceInfo::vertex_color`, the retail specular slot.
    @location(2) surface_color: vec4<f32>,
    @location(3) grid: vec2<f32>,
    @location(4) corner: u32,
};

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) grid: vec2<f32>,
    @location(2) mask1_uv: vec2<f32>,
    @location(3) mask2_uv: vec2<f32>,
    @location(4) distance: f32,
    // The modern fogged fraction (`modern_fog`).
    @location(5) haze: f32,
    @location(6) normal: vec3<f32>,
    // `SurfaceInfo::vertex_color`: the surface tint (rgb) and how much the sun reaches it (a).
    @location(7) surface: vec4<f32>,
    @location(8) world: vec3<f32>,
};

// `LScapeMaterialInfo::CalcRotUVs` by rotation (ROT_0..ROT_270) and slot (SW, SE, NW, NE).
const MASK_UV = array<vec2<f32>, 16>(
    vec2(0.0, 1.0), vec2(1.0, 1.0), vec2(0.0, 0.0), vec2(1.0, 0.0),
    vec2(1.0, 1.0), vec2(1.0, 0.0), vec2(0.0, 1.0), vec2(0.0, 0.0),
    vec2(1.0, 0.0), vec2(0.0, 0.0), vec2(1.0, 1.0), vec2(0.0, 1.0),
    vec2(0.0, 0.0), vec2(0.0, 1.0), vec2(1.0, 0.0), vec2(1.0, 1.0),
);

fn mask_uv(rot: u32, corner: u32) -> vec2<f32> {
    var table = MASK_UV;
    return table[(max(rot, 1u) - 1u) * 4u + (corner & 3u)];
}

@vertex
fn vs(input: VsIn) -> VsOut {
    var out: VsOut;
    out.clip = frame.view_proj * vec4(input.position, 1.0);
    // The light is evaluated per pixel (`fs`); the vertex passes the normal and surface colour.
    out.color = vec4(1.0);
    out.normal = input.normal;
    out.surface = input.surface_color;
    out.world = input.position;
    out.grid = input.grid;
    out.mask1_uv = mask_uv(batch.rot.x, input.corner);
    out.mask2_uv = mask_uv(batch.rot.y, input.corner);
    out.distance = distance(input.position, frame.eye.xyz);
    out.haze = modern_fog(out.distance, input.position.z, frame.fog.xy);
    return out;
}

// The layers as the D3D8 stages combined them, in display space (MODULATE2X saturating per
// layer, the masks blending the 8-bit results), fogged there, then decoded once to linear light
// for the HDR scene; the tone map keeps colours below its shoulder as they are.
// Retail's `RenderVertexBuffer::IlluminateVertices` light, saturate((max(-N.L c.a, 0) dir +
// ambient) c.rgb), evaluated per pixel on the interpolated normal instead of per vertex. The
// ambient is a sky-over-ground hemisphere as the objects' (`object_modern.wgsl`): full from
// above, the warm ground bounce from below.
const TERRAIN_GROUND_TINT: vec3<f32> = vec3<f32>(0.55, 0.5, 0.42);

fn terrain_light(normal: vec3<f32>, c: vec4<f32>) -> vec3<f32> {
    let n = normalize(normal);
    let direct = max(-dot(n, frame.light_dir.xyz) * c.a, 0.0);
    let ambient = mix(frame.ambient.rgb * TERRAIN_GROUND_TINT, frame.ambient.rgb, n.z * 0.5 + 0.5);
    return saturate((direct * frame.dir_color.rgb + ambient) * c.rgb);
}

@fragment
fn fs(input: VsOut) -> @location(0) vec4<f32> {
    let diffuse = terrain_light(input.normal, input.surface);
    let layer0 = saturate(2.0 * textureSample(base0, wrap_sampler, input.grid * batch.scale.x).rgb * diffuse);
    let layer1 = saturate(2.0 * textureSample(base1, wrap_sampler, input.grid * batch.scale.y).rgb * diffuse);
    let layer2 = saturate(2.0 * textureSample(base2, wrap_sampler, input.grid * batch.scale.z).rgb * diffuse);
    let alpha1 = textureSample(mask1, clamp_sampler, input.mask1_uv).a;
    let alpha2 = textureSample(mask2, clamp_sampler, input.mask2_uv).a;
    var color = mix(layer0, layer1, select(0.0, alpha1, batch.rot.z > 1u));
    color = mix(color, layer2, select(0.0, alpha2, batch.rot.z > 2u));
    // D3DTOP_MODULATE by the shadow map (clamped) and the cloud shadows (wrapped).
    let shadow_uv = (frame.shadow * vec4(input.world, 1.0)).xy;
    color *= textureSample(shadow_map, frame_clamp, shadow_uv).rgb;
    let cloud_uv = input.world.xy * frame.clouds.z + frame.clouds.xy;
    color *= textureSample(cloud_texture, frame_wrap, cloud_uv).rgb;
    let fogged = mix(color, saturate(frame.fog_color.rgb), input.haze);
    return vec4(srgb_to_linear(fogged), 1.0);
}
