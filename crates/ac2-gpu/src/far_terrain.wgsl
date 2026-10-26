// Far landscape (`LScapeTerrainMesh::InitFarMesh`'s material): one layer, vertex diffuse, ONE/ZERO,
// LESSEQUAL with depth write, stage 0 ADDSIGNED2X(texture, diffuse) with the far texture, alpha
// SELECTARG2 diffuse. `IlluminateVertices` lights the diffuse from the far colour exactly as it
// lights the near mesh; fog is the near landscape's linear range fog.

struct Frame {
    view_proj: mat4x4<f32>,
    eye: vec4<f32>,
    light_dir: vec4<f32>,
    dir_color: vec4<f32>,
    ambient: vec4<f32>,
    fog_color: vec4<f32>,
    fog: vec4<f32>,
    // The terrain frame's shading vec4: x 1 for the modern path.
    shading: vec4<f32>,
};

@group(0) @binding(0) var<uniform> frame: Frame;
@group(1) @binding(0) var far_texture: texture_2d<f32>;
@group(1) @binding(1) var wrap_sampler: sampler;

struct VsIn {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    // `SurfaceInfo::far_vertex_color`, the retail specular slot.
    @location(2) far_color: vec4<f32>,
    @location(3) uv: vec2<f32>,
};

struct VsOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) distance: f32,
    @location(3) haze: f32,
};

@vertex
fn vs(input: VsIn) -> VsOut {
    var out: VsOut;
    out.clip = frame.view_proj * vec4(input.position, 1.0);
    // `RenderVertexBuffer::IlluminateVertices`, as in terrain.wgsl.
    let c = input.far_color;
    let direct = max(-dot(input.normal, frame.light_dir.xyz) * c.a, 0.0);
    let light = (direct * frame.dir_color.rgb + frame.ambient.rgb) * c.rgb;
    out.color = vec4(clamp(light, vec3(0.0), vec3(1.0)), 1.0);
    out.uv = input.uv;
    out.distance = distance(input.position, frame.eye.xyz);
    out.haze = modern_fog(out.distance, input.position.z, frame.fog.xy);
    return out;
}

// `D3DTOP_ADDSIGNED2X` ((arg1 + arg2 - 0.5) * 2, saturated) runs on the authored gamma values as
// retail does (its 0.5 neutral point is an encoded value); the result is fogged there too, as the
// near terrain is, then decoded once for the HDR scene.
@fragment
fn fs(input: VsOut) -> @location(0) vec4<f32> {
    let texel = textureSample(far_texture, wrap_sampler, input.uv).rgb;
    let combined = saturate((texel + input.color.rgb - 0.5) * 2.0);
    return vec4(srgb_to_linear(mix(combined, saturate(frame.fog_color.rgb), input.haze)), 1.0);
}
