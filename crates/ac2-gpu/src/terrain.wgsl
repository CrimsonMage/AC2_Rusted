// Near landscape. One pass computes what `LScapeMaterialInfo::CombineMaterials` draws in up to
// three: layer 0 opaque (ONE/ZERO), each later layer SRCALPHA/INVSRCALPHA through its alpha mask,
// every layer modulated by the lit vertex diffuse, then depth fog.

struct Frame {
    view_proj: mat4x4<f32>,
    eye: vec4<f32>,
    sun_dir: vec4<f32>,
    sun: vec4<f32>,
    ambient: vec4<f32>,
    fog_color: vec4<f32>,
    // start, end, enabled
    fog: vec4<f32>,
};

struct Batch {
    // UV scale per layer: vertex spacing / tiling (`CalcUVInfo`).
    scale: vec4<f32>,
    // Mask rotation (`rot_types`, 1..4) of layers 1 and 2, then the layer count.
    rot: vec4<u32>,
};

@group(0) @binding(0) var<uniform> frame: Frame;
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
    @location(2) color: vec4<f32>,
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
    // `LScapeTerrainMesh::IlluminateVertices`: ambient plus the sun's diffuse term, saturated
    // into the vertex colour like D3D's diffuse register.
    let lambert = max(dot(normalize(input.normal), frame.sun_dir.xyz), 0.0);
    let light = frame.ambient.rgb + frame.sun.rgb * lambert;
    out.color = vec4(clamp(input.color.rgb * light, vec3(0.0), vec3(1.0)), 1.0);
    out.grid = input.grid;
    out.mask1_uv = mask_uv(batch.rot.x, input.corner);
    out.mask2_uv = mask_uv(batch.rot.y, input.corner);
    out.distance = distance(input.position, frame.eye.xyz);
    return out;
}

@fragment
fn fs(input: VsOut) -> @location(0) vec4<f32> {
    var color = textureSample(base0, wrap_sampler, input.grid * batch.scale.x).rgb;
    let layer1 = textureSample(base1, wrap_sampler, input.grid * batch.scale.y).rgb;
    let layer2 = textureSample(base2, wrap_sampler, input.grid * batch.scale.z).rgb;
    let alpha1 = textureSample(mask1, clamp_sampler, input.mask1_uv).a;
    let alpha2 = textureSample(mask2, clamp_sampler, input.mask2_uv).a;
    color = mix(color, layer1, select(0.0, alpha1, batch.rot.z > 1u));
    color = mix(color, layer2, select(0.0, alpha2, batch.rot.z > 2u));
    color = color * input.color.rgb;
    let fog_span = max(frame.fog.y - frame.fog.x, 1e-3);
    let fog = clamp((input.distance - frame.fog.x) / fog_span, 0.0, 1.0) * frame.fog.z;
    return vec4(mix(color, frame.fog_color.rgb, fog), 1.0);
}
