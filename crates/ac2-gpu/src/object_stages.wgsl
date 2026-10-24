// `shade` for layers without an authored pixel shader: the D3D8 fixed-function texture cascade
// `RenderDeviceD3D::SetupState` configures, then the specular add (`D3DRS_SPECULARENABLE`).

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

fn shade(input: VsOut, texels: array<vec4<f32>, 4>) -> vec4<f32> {
    let diffuse = input.color0;
    let specular = input.color1;
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
    if has(FLAG_SPECULAR_ENABLE) {
        current = vec4(saturate(current.rgb + specular.rgb), current.a);
    }
    return current;
}
