//! Authored `ps.1.1` shaders as WGSL `shade` functions. Registers follow D3D8 ps.1.1 on the
//! hardware AC2 targeted: colour inputs clamped to [0, 1], every result clamped to [-1, 1]
//! (`MaxPixelShaderValue`), `tex tN` reads the stage's texel and `texcoord tN` its clamped
//! coordinate. The constants are those `RenderDeviceD3D::SetupVertexShaderConstants` sets:
//! c0 ambient, c1 and c5 light times material diffuse, c2 material specular, c3 light colour,
//! c4 material diffuse; `def` overrides them.

use std::fmt::Write;

use ac2_render::object::{PixelProgram, PsOp, PsRegister, PsSource, PsSourceModifier};

/// The WGSL `shade` function for `program`.
pub fn shade_function(program: &PixelProgram) -> String {
    let mut out = String::new();
    out.push_str("fn shade(input: VsOut, texels: array<vec4<f32>, 4>) -> vec4<f32> {\n");
    out.push_str(
        "    let light = select(vec3(0.0), frame.sun_diffuse.rgb, frame.sun_inv_dir.w > 0.0 && has(FLAG_DYNAMIC));\n\
         \x20   let c0 = vec4(min(frame.ambient.rgb * draw.ambient.rgb, vec3(1.0)), 0.0);\n\
         \x20   let c1 = vec4(min(light * draw.diffuse.rgb, vec3(1.0)), min(draw.diffuse.a, 1.0));\n\
         \x20   let c2 = draw.specular;\n\
         \x20   let c3 = vec4(light, 1.0);\n\
         \x20   let c4 = draw.diffuse;\n\
         \x20   let c5 = c1;\n\
         \x20   let c6 = vec4(0.0);\n\
         \x20   let c7 = vec4(0.0);\n\
         \x20   let v0 = input.color0;\n\
         \x20   let v1 = input.color1;\n\
         \x20   var r0 = vec4(0.0);\n\
         \x20   var r1 = vec4(0.0);\n\
         \x20   var t0 = vec4(0.0);\n\
         \x20   var t1 = vec4(0.0);\n\
         \x20   var t2 = vec4(0.0);\n\
         \x20   var t3 = vec4(0.0);\n",
    );
    let constant = |index: u8| -> String {
        match program.constants.iter().rev().find(|(i, _)| *i == index) {
            Some((_, v)) => format!("vec4({:?}, {:?}, {:?}, {:?})", v[0], v[1], v[2], v[3]),
            None => format!("c{index}"),
        }
    };
    let mut index = 0;
    let instructions = &program.instructions;
    while index < instructions.len() {
        // An instruction and the ones co-issued with it read the registers as before the group.
        let mut end = index + 1;
        while end < instructions.len() && instructions[end].coissue {
            end += 1;
        }
        out.push_str("    {\n");
        for (slot, instruction) in instructions[index..end].iter().enumerate() {
            let value = match instruction.op {
                PsOp::Tex => {
                    let PsRegister::Texture(n) = instruction.dest else {
                        continue;
                    };
                    format!("texels[{n}]")
                }
                PsOp::Texcoord => {
                    let PsRegister::Texture(n) = instruction.dest else {
                        continue;
                    };
                    format!("vec4(clamp(input.t{n}, vec3(0.0), vec3(1.0)), 1.0)")
                }
                op => {
                    let s: Vec<String> = instruction
                        .sources
                        .iter()
                        .map(|source| source_expression(source, &constant))
                        .collect();
                    let arg = |i: usize| s.get(i).cloned().unwrap_or_else(|| "vec4(0.0)".into());
                    let raw = match op {
                        PsOp::Mov => arg(0),
                        PsOp::Add => format!("{} + {}", arg(0), arg(1)),
                        PsOp::Sub => format!("{} - {}", arg(0), arg(1)),
                        PsOp::Mul => format!("{} * {}", arg(0), arg(1)),
                        PsOp::Mad => format!("{} * {} + {}", arg(0), arg(1), arg(2)),
                        PsOp::Lrp => format!("mix({}, {}, {})", arg(2), arg(1), arg(0)),
                        PsOp::Dp3 => format!("vec4(dot(({}).xyz, ({}).xyz))", arg(0), arg(1)),
                        PsOp::Dp4 => format!("vec4(dot({}, {}))", arg(0), arg(1)),
                        PsOp::Cnd => {
                            format!("select({}, {}, ({}).a > 0.5)", arg(2), arg(1), arg(0))
                        }
                        PsOp::Tex | PsOp::Texcoord => unreachable!(),
                    };
                    let scale = match instruction.shift {
                        1 => " * 2.0",
                        2 => " * 4.0",
                        -1 => " * 0.5",
                        _ => "",
                    };
                    let mut value = format!("({raw}){scale}");
                    if instruction.saturate {
                        value = format!("saturate({value})");
                    }
                    format!("clamp({value}, vec4(-1.0), vec4(1.0))")
                }
            };
            let _ = writeln!(out, "        let p{slot} = {value};");
        }
        for (slot, instruction) in instructions[index..end].iter().enumerate() {
            let dest = register_name(instruction.dest);
            let m = instruction.mask;
            let mask = format!(
                "vec4({}, {}, {}, {})",
                m & 1 != 0,
                m & 2 != 0,
                m & 4 != 0,
                m & 8 != 0
            );
            let _ = writeln!(out, "        {dest} = select({dest}, p{slot}, {mask});");
        }
        out.push_str("    }\n");
        index = end;
    }
    out.push_str("    return r0;\n}\n");
    out
}

fn register_name(register: PsRegister) -> String {
    match register {
        PsRegister::Temp(n) => format!("r{n}"),
        PsRegister::Texture(n) => format!("t{n}"),
        PsRegister::Color(n) => format!("v{n}"),
        PsRegister::Constant(n) => format!("c{n}"),
    }
}

fn source_expression(source: &PsSource, constant: &dyn Fn(u8) -> String) -> String {
    let mut value = match source.register {
        PsRegister::Constant(n) => constant(n),
        other => register_name(other),
    };
    if let Some(channel) = source.replicate {
        value = format!(
            "vec4(({value}).{})",
            ["x", "y", "z", "w"][usize::from(channel)]
        );
    }
    value = match source.modifier {
        PsSourceModifier::None => value,
        PsSourceModifier::Bias => format!("({value} - 0.5)"),
        PsSourceModifier::SignedScale => format!("({value} * 2.0 - 1.0)"),
        PsSourceModifier::Complement => format!("(1.0 - {value})"),
    };
    if source.negate {
        value = format!("(-{value})");
    }
    value
}

#[cfg(test)]
mod tests {
    use super::shade_function;
    use ac2_render::object::parse_pixel_shader;

    #[test]
    fn coissued_pairs_read_registers_before_the_pair() {
        let program = parse_pixel_shader(
            b"ps.1.1\ndef c6, 0.75, 0.75, 0.75, 0.75\ntex t0\ndp3_sat r0.rgb, t0_bx2, v0_bx2\n\
              mul_x2 r0.rgb, r0, c6\n+mov r0.a, c4.a\n",
        )
        .unwrap();
        let wgsl = shade_function(&program);
        assert!(wgsl.contains("let p0 = texels[0];"));
        assert!(wgsl.contains("vec4(0.75, 0.75, 0.75, 0.75)"));
        // The pair computes both values before writing r0.
        let pair = wgsl.find("let p1 = clamp((vec4((c4).w))").unwrap();
        let write = wgsl
            .rfind("r0 = select(r0, p0, vec4(true, true, true, false));")
            .unwrap();
        assert!(pair < write);
        assert!(wgsl.contains("r0 = select(r0, p1, vec4(false, false, false, true));"));
    }
}
