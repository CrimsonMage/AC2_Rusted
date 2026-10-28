// Screen-space ambient occlusion from the world's depth alone (`ao.rs`), at half size: a
// normal-oriented hemisphere of samples around each pixel's view-space point, each counted when
// the scene's surface at its screen position lies in front of it within the radius, then a
// depth-aware 4x4 blur. The tone map darkens the scene by it (`tonemap.wgsl`). The sky (depth 1)
// takes no occlusion.

struct Ao {
    // x: tan(fov_y / 2) * aspect, y: tan(fov_y / 2), z: near, w: far.
    projection: vec4<f32>,
    // x: radius in metres, y: depth bias in metres, z: samples, w: 1 when the camera is known.
    sampling: vec4<f32>,
    // x: how much of the occlusion darkens the scene.
    look: vec4<f32>,
};

@group(0) @binding(0) var depth: texture_depth_2d;
@group(0) @binding(1) var<uniform> ao: Ao;
// The blur's input: the raw occlusion.
@group(0) @binding(2) var raw: texture_2d<f32>;

@vertex
fn vs(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let uv = vec2(f32((index << 1u) & 2u), f32(index & 2u));
    return vec4(uv * 2.0 - 1.0, 0.0, 1.0);
}

// Distance along the view axis of a depth-buffer value (`perspective`: 0 near, 1 far).
fn view_z(d: f32) -> f32 {
    let near = ao.projection.z;
    let far = ao.projection.w;
    return near * far / (far - d * (far - near));
}

fn depth_at(px: vec2<i32>) -> f32 {
    let size = vec2<i32>(textureDimensions(depth));
    return textureLoad(depth, clamp(px, vec2(0), size - 1), 0);
}

// The view-space point (x right, y up, z forward) under full-size pixel `px`.
fn view_point(px: vec2<i32>) -> vec3<f32> {
    let size = vec2<f32>(textureDimensions(depth));
    let z = view_z(depth_at(px));
    let ndc = (vec2<f32>(px) + 0.5) / size * 2.0 - 1.0;
    return vec3(ndc.x * ao.projection.x * z, -ndc.y * ao.projection.y * z, z);
}

// Interleaved gradient noise (Jimenez): a per-pixel rotation that the blur averages out.
fn noise(px: vec2<f32>) -> f32 {
    return fract(52.982919 * fract(dot(px, vec2(0.06711056, 0.00583715))));
}

@fragment
fn fs_ao(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let px = vec2<i32>(position.xy) * 2;
    let d = depth_at(px);
    if ao.sampling.w < 0.5 || d >= 1.0 {
        return vec4(1.0);
    }
    let p = view_point(px);
    // The normal from the nearer neighbour on each axis, so silhouettes do not smear.
    let right = view_point(px + vec2(2, 0)) - p;
    let left = p - view_point(px - vec2(2, 0));
    let down = view_point(px + vec2(0, 2)) - p;
    let up = p - view_point(px - vec2(0, 2));
    let dx = select(left, right, abs(right.z) < abs(left.z));
    let dy = select(up, down, abs(down.z) < abs(up.z));
    var n = normalize(cross(dy, dx));
    if dot(n, p) > 0.0 {
        n = -n;
    }
    // A tangent frame around the normal, turned by the pixel's noise.
    let angle = noise(position.xy) * 6.2831853;
    let helper = select(vec3(0.0, 1.0, 0.0), vec3(1.0, 0.0, 0.0), abs(n.y) > 0.9);
    let t0 = normalize(cross(helper, n));
    let b0 = cross(n, t0);
    let t = t0 * cos(angle) + b0 * sin(angle);
    let b = cross(n, t);
    let radius = ao.sampling.x;
    let bias = ao.sampling.y;
    let count = u32(ao.sampling.z);
    let size = vec2<f32>(textureDimensions(depth));
    var occluded = 0.0;
    for (var i = 0u; i < count; i++) {
        // A golden-angle spiral over the hemisphere, denser towards the point.
        let f = (f32(i) + 0.5) / f32(count);
        let phi = f32(i) * 2.3999632;
        let cos_theta = sqrt(1.0 - f);
        let sin_theta = sqrt(f);
        let dir = t * (cos(phi) * sin_theta) + b * (sin(phi) * sin_theta) + n * cos_theta;
        let s = p + dir * radius * mix(0.1, 1.0, f * f);
        let ndc = vec2(s.x / (s.z * ao.projection.x), -s.y / (s.z * ao.projection.y));
        let spx = vec2<i32>((ndc * 0.5 + 0.5) * size);
        let surface = view_z(depth_at(spx));
        let in_range = smoothstep(0.0, 1.0, radius / max(abs(p.z - surface), 1e-4));
        occluded += select(0.0, in_range, surface < s.z - bias);
    }
    return vec4(vec3(1.0 - ao.look.x * occluded / f32(count)), 1.0);
}

// A 4x4 average of the raw occlusion around this half-size pixel, each tap weighted by how close
// its depth is to the centre's, so occlusion does not bleed across silhouettes.
@fragment
fn fs_blur(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let here = vec2<i32>(position.xy);
    let centre = view_z(depth_at(here * 2));
    let raw_size = vec2<i32>(textureDimensions(raw));
    var sum = 0.0;
    var total = 0.0;
    for (var y = -2; y < 2; y++) {
        for (var x = -2; x < 2; x++) {
            let at = clamp(here + vec2(x, y), vec2(0), raw_size - 1);
            let z = view_z(depth_at(at * 2));
            let w = 1.0 / (1e-3 + abs(z - centre) / max(centre, 1e-3) * 50.0);
            sum += textureLoad(raw, at, 0).r * w;
            total += w;
        }
    }
    return vec4(vec3(sum / total), 1.0);
}
