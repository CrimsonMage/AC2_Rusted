// Ground-truth ambient occlusion from the world's depth alone (`ao.rs`), at half size, after
// Jimenez et al., "Practical Realtime Strategies for Accurate Indirect Occlusion" (2016): per
// pixel a few slices through the view vector, a horizon search each way along each slice's
// screen direction, and the cosine-weighted visibility integral with the normal projected into
// the slice. Normals come from the depth. Then a depth-aware 4x4 blur and, while the temporal
// anti-aliasing runs, a reprojected accumulation with the previous frames. The tone map darkens
// the scene by the result (`tonemap.wgsl`). The sky (depth 1) takes no occlusion.

struct Ao {
    // x: tan(fov_y / 2) * aspect, y: tan(fov_y / 2), z: near, w: far.
    projection: vec4<f32>,
    // x: radius in metres, y: how much of the occlusion darkens the scene, z: slices per pixel,
    // w: horizon steps per side.
    sampling: vec4<f32>,
    // x: the frame counter (rotates the noise), y: 1 when the history is read, z: the relative
    // depth difference that rejects it, w: the blend towards the current frame.
    state: vec4<f32>,
    // The current frame's unjittered eye-relative inverse view-projection, the previous
    // frame's view-projection and the previous eye minus the current one (`taa.rs`).
    inv_view_proj: mat4x4<f32>,
    prev_view_proj: mat4x4<f32>,
    eye_delta: vec4<f32>,
};

const PI: f32 = 3.14159265;
// Occluders count fully up to this share of the radius from the point, fading out to the radius.
const FALLOFF_START: f32 = 0.6;
// The widest horizon search in full-size pixels, so a close surface does not fetch far away.
const MAX_RADIUS_PIXELS: f32 = 96.0;
// The multi-bounce approximation's albedo: there is no albedo buffer, so a mid grey.
const ALBEDO: f32 = 0.5;

@group(0) @binding(0) var depth: texture_depth_2d;
@group(0) @binding(1) var<uniform> ao: Ao;
// The blur's and the accumulation's input.
@group(0) @binding(2) var source: texture_2d<f32>;
// The previous frame's occlusion and the view-space depth it was made at.
@group(0) @binding(3) var history: texture_2d<f32>;
@group(0) @binding(4) var history_depth: texture_2d<f32>;

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

// Interleaved gradient noise (Jimenez): a per-pixel value the blur and the accumulation average.
fn noise(px: vec2<f32>) -> f32 {
    return fract(52.982919 * fract(dot(px, vec2(0.06711056, 0.00583715))));
}

// The cosine of the horizon angle a slice starts from on one side: the tangent plane, `side`
// (+1 along the slice direction, -1 against) of the view vector, for the projected normal at
// signed angle `n` from it.
fn tangent_cos(n: f32, side: f32) -> f32 {
    return cos(n + side * (0.5 * PI));
}

// One slice's visibility: the arc between the horizons `h1` (negative) and `h2` (positive),
// measured from the view vector, weighted by the cosine to the projected normal at angle `n`.
fn slice_arc(n: f32, h1: f32, h2: f32) -> f32 {
    let c = cos(n);
    let s = sin(n);
    return 0.25 * (-cos(2.0 * h1 - n) + c + 2.0 * h1 * s)
        + 0.25 * (-cos(2.0 * h2 - n) + c + 2.0 * h2 * s);
}

// Fakes the light that bounces between nearby surfaces (Jimenez 2016): brightens the occlusion of
// a surface of `albedo`, so concave corners are not as black as one bounce-free visibility says.
fn multi_bounce(visibility: f32, albedo: f32) -> f32 {
    let a = 2.0404 * albedo - 0.3324;
    let b = -4.7951 * albedo + 0.6417;
    let c = 2.7552 * albedo + 0.6903;
    return max(visibility, ((visibility * a + b) * visibility + c) * visibility);
}

@fragment
fn fs_ao(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let px = vec2<i32>(position.xy) * 2;
    let d = depth_at(px);
    if d >= 1.0 {
        return vec4(1.0);
    }
    let p = view_point(px);
    let view = normalize(-p);
    // The normal from the nearer neighbour on each axis, so silhouettes do not smear.
    let right = view_point(px + vec2(2, 0)) - p;
    let left = p - view_point(px - vec2(2, 0));
    let down = view_point(px + vec2(0, 2)) - p;
    let up = p - view_point(px - vec2(0, 2));
    let dx = select(left, right, abs(right.z) < abs(left.z));
    let dy = select(up, down, abs(down.z) < abs(up.z));
    var normal = normalize(cross(dy, dx));
    if dot(normal, view) < 0.0 {
        normal = -normal;
    }
    let radius = ao.sampling.x;
    let slices = i32(ao.sampling.z);
    let steps = i32(ao.sampling.w);
    let size = vec2<f32>(textureDimensions(depth));
    let focal = 0.5 * size.y / ao.projection.y;
    let radius_pixels = min(radius * focal / p.z, MAX_RADIUS_PIXELS);
    if radius_pixels < 2.0 {
        return vec4(1.0);
    }
    // Both noises rotate with the frame, so the accumulation integrates them.
    let frame = ao.state.x % 64.0;
    let slice_noise = fract(noise(position.xy) + 0.6180340 * frame);
    let step_noise = fract(noise(position.xy + vec2(37.0, 17.0)) + 0.7548777 * frame);
    var visibility = 0.0;
    for (var s = 0; s < slices; s++) {
        let phi = (f32(s) + slice_noise) / f32(slices) * PI;
        // The slice's direction on the screen plane (y up) and as screen pixels (y down).
        let direction = vec3(cos(phi), sin(phi), 0.0);
        let pixels = vec2(direction.x, -direction.y);
        let axis = normalize(cross(direction, view));
        let ortho = direction - dot(direction, view) * view;
        let projected = normal - axis * dot(normal, axis);
        let length_n = length(projected);
        if length_n < 1e-4 {
            continue;
        }
        let sign_n = select(-1.0, 1.0, dot(ortho, projected) >= 0.0);
        let n = sign_n * acos(clamp(dot(projected, view) / length_n, -1.0, 1.0));
        // The horizons start at the tangent plane; occluders only raise them.
        var cos1 = tangent_cos(n, -1.0);
        var cos2 = tangent_cos(n, 1.0);
        let low1 = cos1;
        let low2 = cos2;
        for (var i = 0; i < steps; i++) {
            let f = (f32(i) + step_noise) / f32(steps);
            let offset = pixels * max(radius_pixels * f * f, 1.5);
            for (var side = 0; side < 2; side++) {
                let sign_s = select(-1.0, 1.0, side == 1);
                let point = view_point(px + vec2<i32>(round(offset * sign_s)));
                let delta = point - p;
                let dist = max(length(delta), 1e-4);
                let weight = clamp((radius - dist) / (radius * (1.0 - FALLOFF_START)), 0.0, 1.0);
                let low = select(low1, low2, side == 1);
                let sample_cos = mix(low, dot(delta, view) / dist, weight);
                if side == 1 {
                    cos2 = max(cos2, sample_cos);
                } else {
                    cos1 = max(cos1, sample_cos);
                }
            }
        }
        let h1 = -acos(clamp(cos1, -1.0, 1.0));
        let h2 = acos(clamp(cos2, -1.0, 1.0));
        visibility += length_n * slice_arc(n, h1, h2);
    }
    let bounced = multi_bounce(clamp(visibility / f32(slices), 0.0, 1.0), ALBEDO);
    return vec4(vec3(1.0 - ao.sampling.y * (1.0 - bounced)), 1.0);
}

// A 4x4 average of the raw occlusion around this half-size pixel, each tap weighted by how close
// its depth is to the centre's, so occlusion does not bleed across silhouettes.
@fragment
fn fs_blur(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let here = vec2<i32>(position.xy);
    let centre = view_z(depth_at(here * 2));
    let raw_size = vec2<i32>(textureDimensions(source));
    var sum = 0.0;
    var total = 0.0;
    for (var y = -2; y < 2; y++) {
        for (var x = -2; x < 2; x++) {
            let at = clamp(here + vec2(x, y), vec2(0), raw_size - 1);
            let z = view_z(depth_at(at * 2));
            let w = 1.0 / (1e-3 + abs(z - centre) / max(centre, 1e-3) * 50.0);
            sum += textureLoad(source, at, 0).r * w;
            total += w;
        }
    }
    return vec4(vec3(sum / total), 1.0);
}

struct Accumulated {
    @location(0) occlusion: vec4<f32>,
    // The view-space depth this frame's occlusion was made at: the next frame's rejection test.
    @location(1) view_depth: vec4<f32>,
};

// The blurred occlusion blended with the previous frames', reprojected with the camera's motion
// (`taa.rs`). A previous pixel whose depth differs from where this one was is rejected, and the
// history is held within the current neighbourhood's range, so a moving object does not ghost.
@fragment
fn fs_accumulate(@builtin(position) position: vec4<f32>) -> Accumulated {
    let here = vec2<i32>(position.xy);
    let px = here * 2;
    let d = depth_at(px);
    if d >= 1.0 {
        return Accumulated(vec4(1.0), vec4(ao.projection.w));
    }
    let z = view_z(d);
    let current = textureLoad(source, here, 0).r;
    var result = current;
    if ao.state.y > 0.5 {
        let size = vec2<f32>(textureDimensions(depth));
        let uv = (vec2<f32>(px) + 0.5) / size;
        let world = ao.inv_view_proj * vec4(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, d, 1.0);
        let relative = world.xyz / world.w - ao.eye_delta.xyz;
        let clip = ao.prev_view_proj * vec4(relative, 1.0);
        if clip.w > 1e-4 {
            let previous = vec2(clip.x / clip.w * 0.5 + 0.5, 0.5 - clip.y / clip.w * 0.5);
            if all(previous >= vec2(0.0)) && all(previous < vec2(1.0)) {
                let half_size = vec2<f32>(textureDimensions(history_depth));
                let at = vec2<i32>(previous * half_size);
                let past_z = textureLoad(history_depth, at, 0).r;
                if abs(past_z - clip.w) < ao.state.z * clip.w {
                    let low_high = neighbourhood(here);
                    let past = clamp(
                        textureLoad(history, at, 0).r,
                        low_high.x - 0.04,
                        low_high.y + 0.04,
                    );
                    result = mix(past, current, ao.state.w);
                }
            }
        }
    }
    return Accumulated(vec4(vec3(result), 1.0), vec4(z));
}

// The range of the 3 x 3 blurred occlusion around `here`: x the lowest, y the highest.
fn neighbourhood(here: vec2<i32>) -> vec2<f32> {
    let top = vec2<i32>(textureDimensions(source)) - 1;
    var low = 1.0;
    var high = 0.0;
    for (var y = -1; y <= 1; y++) {
        for (var x = -1; x <= 1; x++) {
            let v = textureLoad(source, clamp(here + vec2(x, y), vec2(0), top), 0).r;
            low = min(low, v);
            high = max(high, v);
        }
    }
    return vec2(low, high);
}
