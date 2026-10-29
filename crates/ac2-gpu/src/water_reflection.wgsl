// The water's reflection (`water_reflection.rs`): the world's pixels on the frame's reflection
// plane (`LScapeRender::FindReflectionPlane`) march the mirrored view ray through the world's depth
// and take the colour it meets, or the sky's where it leaves the world; a Fresnel weight blends it
// over the water.

struct Water {
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
    inv_proj: mat4x4<f32>,
    inv_view: mat4x4<f32>,
    // The water plane in view space: unit normal and distance.
    plane: vec4<f32>,
    // Client seconds, the `WaterReflections` level, the near plane.
    params: vec4<f32>,
    // The horizon's colour (the fog's, linear) where nothing is met.
    horizon: vec4<f32>,
    // The water block window: its first block's world x, y, the block edge, the window's width
    // in blocks.
    blocks: vec4<f32>,
}

@group(0) @binding(0) var<uniform> water: Water;
@group(0) @binding(1) var scene_color: texture_2d<f32>;
@group(0) @binding(2) var scene_depth: texture_depth_2d;
@group(0) @binding(3) var linear_sampler: sampler;
// 1 for each block of the window whose water lies on the plane.
@group(0) @binding(4) var water_blocks: texture_2d<f32>;

// Whether the block under view-space point `p` holds the plane's water.
fn on_water(p: vec3<f32>) -> bool {
    let world = (water.inv_view * vec4(p, 1.0)).xy;
    let cell = floor((world - water.blocks.xy) / water.blocks.z);
    if any(cell < vec2(0.0)) || any(cell >= vec2(water.blocks.w)) {
        return false;
    }
    return textureLoad(water_blocks, vec2<i32>(cell), 0).r > 0.5;
}

@vertex
fn vs(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let uv = vec2(f32((index << 1u) & 2u), f32(index & 2u));
    return vec4(uv * 2.0 - 1.0, 0.0, 1.0);
}

fn view_at(uv: vec2<f32>, depth: f32) -> vec3<f32> {
    let point = water.inv_proj * vec4(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, depth, 1.0);
    return point.xyz / point.w;
}

fn screen_of(p: vec3<f32>) -> vec3<f32> {
    let clip = water.proj * vec4(p, 1.0);
    let ndc = clip.xyz / clip.w;
    return vec3(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5, ndc.z);
}

// `WaterReflections` 2: the surface ripples (retail's bump-mapped reflection, a 0.04 offset).
// The ripples fade with distance, where they would only alias.
fn rippled(n: vec3<f32>, p: vec3<f32>) -> vec3<f32> {
    let world = (water.inv_view * vec4(p, 1.0)).xy;
    let t = water.params.x;
    let slope = vec2(
        cos(world.x * 0.9 + t * 1.3) + 0.6 * cos(world.x * 0.37 - world.y * 0.53 + t * 0.9),
        sin(world.y * 1.1 - t * 1.1) + 0.6 * sin(world.y * 0.41 + world.x * 0.29 + t * 0.7),
    ) * (0.02 / (1.0 + length(p) / 60.0));
    let offset = (water.view * vec4(slope, 0.0, 0.0)).xyz;
    return normalize(n + offset);
}

@fragment
fn fs_march(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let size = vec2<f32>(textureDimensions(scene_depth));
    let uv = position.xy / size;
    // Where the pixel's view ray meets the plane (the camera is above it).
    let ray = normalize(view_at(uv, 1.0));
    let down = dot(water.plane.xyz, ray);
    if down >= -1e-4 {
        return vec4(0.0);
    }
    let along = -water.plane.w / down;
    let p = ray * along;
    // Water shows where nothing stands in front of the surface (what is under it shows through
    // the water) over a block whose water is on the plane.
    let depth = textureLoad(scene_depth, vec2<i32>(position.xy), 0);
    if depth < 1.0 && length(view_at(uv, depth)) < along * 0.998 - 0.05 {
        return vec4(0.0);
    }
    if !on_water(p) {
        return vec4(0.0);
    }
    var n = water.plane.xyz;
    if water.params.y > 1.5 {
        n = rippled(n, p);
    }
    let v = normalize(p);
    let r = reflect(v, n);
    var color = water.horizon.rgb;
    var sky_uv = vec2(-1.0);
    var hit = false;
    var hit_uv = vec2(0.0);
    var before = 0.0;
    var t = 0.3;
    for (var i = 0; i < 48; i++) {
        let q = p + r * t;
        if q.z > -water.params.z {
            break;
        }
        let s = screen_of(q);
        if any(s.xy < vec2(0.0)) || any(s.xy > vec2(1.0)) {
            break;
        }
        let met = textureLoad(scene_depth, vec2<i32>(s.xy * size), 0);
        if met >= 1.0 {
            // The sky shows there: what the ray leaves the world towards.
            sky_uv = s.xy;
        } else {
            let behind = q.z - view_at(s.xy, met).z;
            if behind < 0.0 && -behind < max(0.5, t * 0.08) {
                // Close in on the surface between the last step and this one.
                var lo = before;
                var hi = t;
                for (var k = 0; k < 6; k++) {
                    let mid = (lo + hi) * 0.5;
                    let m = screen_of(p + r * mid);
                    let at = textureLoad(scene_depth, vec2<i32>(m.xy * size), 0);
                    if (p + r * mid).z - view_at(m.xy, at).z < 0.0 {
                        hi = mid;
                    } else {
                        lo = mid;
                    }
                }
                hit_uv = screen_of(p + r * hi).xy;
                hit = true;
                break;
            }
        }
        before = t;
        t = t * 1.18 + 0.2;
        if t > 400.0 {
            break;
        }
    }
    if hit {
        let edge = min(min(hit_uv.x, 1.0 - hit_uv.x), min(hit_uv.y, 1.0 - hit_uv.y));
        let met = textureSampleLevel(scene_color, linear_sampler, hit_uv, 0.0).rgb;
        color = mix(color, met, smoothstep(0.0, 0.06, edge));
    } else if sky_uv.x >= 0.0 {
        color = textureSampleLevel(scene_color, linear_sampler, sky_uv, 0.0).rgb;
    } else {
        // Where the mirrored direction reaches the screen at infinity, kept on the screen: the
        // world's colour along the horizon, without seams where rays leave the sides.
        let far_point = water.proj * vec4(r, 0.0);
        if far_point.w > 1e-4 {
            let ndc = far_point.xy / far_point.w;
            let at = clamp(vec2(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5), vec2(0.0), vec2(1.0));
            color = textureSampleLevel(scene_color, linear_sampler, at, 0.0).rgb;
        }
    }
    // Schlick's Fresnel for water (normal incidence 0.02).
    let facing = clamp(dot(-v, n), 0.0, 1.0);
    let fresnel = 0.02 + 0.98 * pow(1.0 - facing, 5.0);
    return vec4(color, fresnel);
}
