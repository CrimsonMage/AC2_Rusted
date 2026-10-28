// `LensFlareEffect::Draw`: each sky light's flare sprites over the world, and the scan of the
// world's depth at the light that fades them in and out (`SceneTool::RequestPixelScan`, read back
// the next frame). Joined after `shading.wgsl`.

struct Flares {
    // Per flare: the light's NDC position, 1 when it is in the view, its fade speed per second.
    lights: array<vec4<f32>, 4>,
    // Per sprite: its axis position, its size, its flare's index.
    points: array<vec4<f32>, 32>,
    // Seconds since the last scan, the output's width / height, the flare count.
    frame: vec4<f32>,
}

@group(0) @binding(0) var<uniform> flares: Flares;
// The opacity of each flare (red, texel x = flare index) as the last scan left it.
@group(0) @binding(1) var opacity: texture_2d<f32>;

// --- The scan: one texel per flare. ---

@group(0) @binding(2) var depth: texture_depth_2d;

@vertex
fn vs_scan(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let uv = vec2(f32((index << 1u) & 2u), f32(index & 2u));
    return vec4(uv * 2.0 - 1.0, 0.0, 1.0);
}

// How much of a 5 x 5 patch, 2 pixels apart, around the light shows the cleared far depth (the sky
// draws no depth): retail's one-pixel test, softened at the edges of what hides the light.
fn sky_fraction(ndc: vec2<f32>) -> f32 {
    let size = vec2<f32>(textureDimensions(depth));
    let center = vec2((ndc.x * 0.5 + 0.5) * size.x, (0.5 - ndc.y * 0.5) * size.y);
    var seen = 0.0;
    for (var y = -2; y <= 2; y++) {
        for (var x = -2; x <= 2; x++) {
            let at = center + vec2(f32(x), f32(y)) * 2.0;
            if all(at >= vec2(0.0)) && all(at < size) {
                if textureLoad(depth, vec2<i32>(at), 0) >= 1.0 {
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
    if f32(index) >= flares.frame.z {
        return vec4(0.0);
    }
    let light = flares.lights[index];
    let last = textureLoad(opacity, vec2(i32(index), 0), 0).r;
    // A light outside the view requests no scan, which reads as hidden.
    var goal = 0.0;
    if light.z > 0.0 {
        goal = sky_fraction(light.xy);
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
    let light = flares.lights[index].xy;
    let fade = textureLoad(opacity, vec2(i32(index), 0), 0).r;
    // White at `fAlpha`: brightest with the light at the screen's centre, never under a fifth.
    // Retail's square rose again for a light past the screen's corners while it faded out; it
    // stays at the fifth here.
    let centred = max(1.0 - length(light) * 0.70422536, 0.0);
    let falloff = max(centred * centred, 0.2);
    // The sprite's centre on the axis through the screen centre, its half size square in pixels
    // (`m_fPixelAspect`).
    let center = -p.x * light;
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
