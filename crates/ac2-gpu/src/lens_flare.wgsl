// `LensFlareEffect::Draw`: each light's flare sprites over the world (the sky lights' and the
// interior lights' coronas), and the scan of the world's depth at the light that fades them in and
// out (`SceneTool::RequestPixelScan`, read back the next frame). Joined after `shading.wgsl`.

struct Flares {
    // Per slot: the light's NDC position, the depth it is seen at (1 for a light at infinity, -1
    // when it is out of the view), its fade speed per second.
    lights: array<vec4<f32>, 64>,
    // Per slot: the opacity a flare new to the slot starts from (-1 to go on from the last scan),
    // 1 when the light is at a finite distance (its depth counts in the falloff), 1 when the slot
    // is in use.
    slots: array<vec4<f32>, 64>,
    // Per sprite: its axis position, its size, its slot.
    points: array<vec4<f32>, 128>,
    // Seconds since the last scan, the output's width / height.
    frame: vec4<f32>,
}

@group(0) @binding(0) var<uniform> flares: Flares;
// The opacity of each slot (red, texel x = slot) as the last scan left it.
@group(0) @binding(1) var opacity: texture_2d<f32>;

// --- The scan: one texel per slot. ---

@group(0) @binding(2) var depth: texture_depth_2d;

@vertex
fn vs_scan(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let uv = vec2(f32((index << 1u) & 2u), f32(index & 2u));
    return vec4(uv * 2.0 - 1.0, 0.0, 1.0);
}

// How much of a 5 x 5 patch, 2 pixels apart, around the light shows nothing nearer than the light
// (for a light at infinity, the cleared far depth: the sky draws no depth): retail's one-pixel
// test, softened at the edges of what hides the light.
fn seen_fraction(ndc: vec2<f32>, light_depth: f32) -> f32 {
    let size = vec2<f32>(textureDimensions(depth));
    let center = vec2((ndc.x * 0.5 + 0.5) * size.x, (0.5 - ndc.y * 0.5) * size.y);
    var seen = 0.0;
    for (var y = -2; y <= 2; y++) {
        for (var x = -2; x <= 2; x++) {
            let at = center + vec2(f32(x), f32(y)) * 2.0;
            if all(at >= vec2(0.0)) && all(at < size) {
                if textureLoad(depth, vec2<i32>(at), 0) >= light_depth {
                    seen += 1.0;
                }
            }
        }
    }
    return seen / 25.0;
}

@fragment
fn fs_scan(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let index = u32(position.x);
    let slot = flares.slots[index];
    if slot.z == 0.0 {
        return vec4(0.0);
    }
    let light = flares.lights[index];
    var last = textureLoad(opacity, vec2(i32(index), 0), 0).r;
    // `LensFlareEffect::InitNew`: a new flare starts from its own opacity.
    if slot.x >= 0.0 {
        last = slot.x;
    }
    // A light outside the view requests no scan, which reads as hidden.
    var goal = 0.0;
    if light.z >= 0.0 {
        goal = seen_fraction(light.xy, light.z);
    }
    // `m_fOpacity`: up or down by the elapsed time times the fade speed, within 0..1.
    let stride = flares.frame.x * light.w;
    let next = clamp(last + clamp(goal - last, -stride, stride), 0.0, 1.0);
    return vec4(next, 0.0, 0.0, 1.0);
}

// --- The sprites. ---

@group(1) @binding(0) var sprite: texture_2d<f32>;
@group(1) @binding(1) var sprite_sampler: sampler;

struct SpriteOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) alpha: f32,
}

@vertex
fn vs_sprite(
    @builtin(vertex_index) corner: u32,
    @builtin(instance_index) point: u32,
) -> SpriteOut {
    let p = flares.points[point];
    let index = u32(p.z);
    let light = flares.lights[index];
    let fade = textureLoad(opacity, vec2(i32(index), 0), 0).r;
    // White at `fAlpha`: brightest with the light at the screen's centre (and, for a light at a
    // finite distance, near the camera: its depth counts), never under a fifth. Retail's square
    // rose again for a light past the screen's corners while it faded out; it stays at the fifth
    // here.
    let reach = vec3(light.xy, select(0.0, max(light.z, 0.0), flares.slots[index].y > 0.0));
    let centred = max(1.0 - length(reach) * 0.70422536, 0.0);
    let falloff = max(centred * centred, 0.2);
    // The sprite's centre on the axis through the screen centre, its half size square in pixels
    // (`m_fPixelAspect`).
    let center = -p.x * light.xy;
    let extent = vec2(p.y * 0.5, flares.frame.y * p.y * 0.5);
    let side = vec2(f32(corner & 1u), f32((corner >> 1u) & 1u));
    var out: SpriteOut;
    out.position = vec4(center + (side * 2.0 - 1.0) * extent, 0.0, 1.0);
    out.uv = side;
    out.alpha = clamp(falloff * fade, 0.0, 1.0);
    return out;
}

@fragment
fn fs_sprite(in: SpriteOut) -> @location(0) vec4<f32> {
    let texel = textureSample(sprite, sprite_sampler, in.uv);
    // Blended SRCALPHA, ONE: added over the scene.
    return vec4(srgb_to_linear(texel.rgb), texel.a * in.alpha);
}
