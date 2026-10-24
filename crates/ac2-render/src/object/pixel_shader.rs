//! Authored `ps.1.1` pixel shaders. `MaterialLayer::BuildPixelShader` hands a layer's
//! `m_pPSSource` (assembly text) to the device, which assembles and binds it in place of the
//! fixed-function texture stages. This parses that text into a small instruction list a backend
//! can translate.

/// A `ps.1.1` register.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PsRegister {
    /// `r0`, `r1`; `r0` is the output.
    Temp(u8),
    /// `t0`..`t3`: the texel after `tex`, or the clamped coordinate after `texcoord`.
    Texture(u8),
    /// `v0`, `v1`: the vertex program's colours, clamped to `[0, 1]`.
    Color(u8),
    /// `c0`..`c7`.
    Constant(u8),
}

/// Source modifiers: `_bias` (x - 0.5), `_bx2` (2x - 1), `1-` (complement).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PsSourceModifier {
    None,
    Bias,
    SignedScale,
    Complement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PsSource {
    pub register: PsRegister,
    pub negate: bool,
    pub modifier: PsSourceModifier,
    /// `.r`/`.g`/`.b`/`.a` replicate swizzle (channel index), or `None` for `.rgba`.
    pub replicate: Option<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PsOp {
    Tex,
    Texcoord,
    Mov,
    Add,
    Sub,
    Mul,
    Mad,
    Lrp,
    Dp3,
    Dp4,
    Cnd,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PsInstruction {
    pub op: PsOp,
    /// `+`: issued with the previous instruction (they read the registers as they were before
    /// the pair).
    pub coissue: bool,
    pub dest: PsRegister,
    /// Write mask, bit 0 red .. bit 3 alpha.
    pub mask: u8,
    /// Result shift: `_x2` 1, `_x4` 2, `_d2` -1.
    pub shift: i8,
    pub saturate: bool,
    pub sources: Vec<PsSource>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PixelProgram {
    /// `def cN, x, y, z, w`.
    pub constants: Vec<(u8, [f32; 4])>,
    pub instructions: Vec<PsInstruction>,
}

/// Parses `ps.1.1` assembly text.
pub fn parse_pixel_shader(source: &[u8]) -> Result<PixelProgram, String> {
    let text = String::from_utf8_lossy(source);
    let mut lines = text
        .lines()
        .map(|line| {
            let line = line.split(';').next().unwrap_or("");
            line.split("//").next().unwrap_or("").trim()
        })
        .filter(|line| !line.is_empty() && *line != "\0");
    match lines.next() {
        Some(version) if version.trim_end_matches('\0') == "ps.1.1" => {}
        other => return Err(format!("unsupported pixel shader version {other:?}")),
    }
    let mut program = PixelProgram {
        constants: Vec::new(),
        instructions: Vec::new(),
    };
    for line in lines {
        let line = line.trim_end_matches('\0').trim();
        if line.is_empty() {
            continue;
        }
        let (coissue, line) = match line.strip_prefix('+') {
            Some(rest) => (true, rest.trim()),
            None => (false, line),
        };
        let (mnemonic, operands) = line
            .split_once(char::is_whitespace)
            .ok_or_else(|| format!("no operands: {line}"))?;
        let operands: Vec<&str> = operands.split(',').map(str::trim).collect();
        let mut parts = mnemonic.split('_');
        let op = parts.next().unwrap_or("");
        if op == "def" {
            let register = register(operands[0])?;
            let PsRegister::Constant(index) = register else {
                return Err(format!("def of {}", operands[0]));
            };
            let mut value = [0.0; 4];
            for (slot, text) in value.iter_mut().zip(&operands[1..]) {
                *slot = text.parse().map_err(|_| format!("bad def value {text}"))?;
            }
            program.constants.push((index, value));
            continue;
        }
        let op = match op {
            "tex" => PsOp::Tex,
            "texcoord" => PsOp::Texcoord,
            "mov" => PsOp::Mov,
            "add" => PsOp::Add,
            "sub" => PsOp::Sub,
            "mul" => PsOp::Mul,
            "mad" => PsOp::Mad,
            "lrp" => PsOp::Lrp,
            "dp3" => PsOp::Dp3,
            "dp4" => PsOp::Dp4,
            "cnd" => PsOp::Cnd,
            other => return Err(format!("unsupported instruction {other}")),
        };
        let (mut shift, mut saturate) = (0, false);
        for modifier in parts {
            match modifier {
                "x2" => shift = 1,
                "x4" => shift = 2,
                "d2" => shift = -1,
                "sat" => saturate = true,
                other => return Err(format!("unsupported modifier _{other}")),
            }
        }
        let (dest_name, mask) = match operands[0].split_once('.') {
            Some((name, swizzle)) => (name, write_mask(swizzle)?),
            None => (operands[0], 0xF),
        };
        let sources = operands[1..]
            .iter()
            .map(|text| parse_source(text))
            .collect::<Result<Vec<_>, _>>()?;
        program.instructions.push(PsInstruction {
            op,
            coissue,
            dest: register(dest_name)?,
            mask,
            shift,
            saturate,
            sources,
        });
    }
    Ok(program)
}

fn register(text: &str) -> Result<PsRegister, String> {
    let (kind, index) = text.split_at(1);
    let index: u8 = index.parse().map_err(|_| format!("bad register {text}"))?;
    Ok(match kind {
        "r" => PsRegister::Temp(index),
        "t" => PsRegister::Texture(index),
        "v" => PsRegister::Color(index),
        "c" => PsRegister::Constant(index),
        _ => return Err(format!("bad register {text}")),
    })
}

fn write_mask(swizzle: &str) -> Result<u8, String> {
    let mut mask = 0;
    for channel in swizzle.chars() {
        mask |= 1 << channel_index(channel).ok_or_else(|| format!("bad mask .{swizzle}"))?;
    }
    Ok(mask)
}

fn channel_index(channel: char) -> Option<u8> {
    match channel {
        'r' | 'x' => Some(0),
        'g' | 'y' => Some(1),
        'b' | 'z' => Some(2),
        'a' | 'w' => Some(3),
        _ => None,
    }
}

fn parse_source(text: &str) -> Result<PsSource, String> {
    let (negate, text) = match text.strip_prefix('-') {
        Some(rest) => (true, rest.trim()),
        None => (false, text),
    };
    let (complement, text) = match text.strip_prefix("1-") {
        Some(rest) => (true, rest.trim()),
        None => (false, text),
    };
    let (text, replicate) = match text.split_once('.') {
        Some((name, swizzle)) => {
            let mut chars = swizzle.chars();
            let first = chars.next().and_then(channel_index);
            match (first, chars.all(|c| channel_index(c) == first)) {
                (Some(channel), true) => (name, Some(channel)),
                _ => return Err(format!("unsupported swizzle .{swizzle}")),
            }
        }
        None => (text, None),
    };
    let (name, modifier) = if let Some(name) = text.strip_suffix("_bias") {
        (name, PsSourceModifier::Bias)
    } else if let Some(name) = text.strip_suffix("_bx2") {
        (name, PsSourceModifier::SignedScale)
    } else if complement {
        (text, PsSourceModifier::Complement)
    } else {
        (text, PsSourceModifier::None)
    };
    Ok(PsSource {
        register: register(name)?,
        negate,
        modifier,
        replicate,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_modifiers_masks_and_coissue() {
        let source =
            b"   ps.1.1  \n def c6, 0.75, 0.75, 0.75, 0.75 ; approx\n tex t0 ; normal map\n\
            dp3_sat r0.rgb, t0_bx2, v0_bx2\n mad_x4_sat r1.rgb, r1, r1, -c6\n\
            mul_x2 r0.rgb, r0, t1\n +mov r0.a, c4.a\n add t1.rgb, t1, t2_bias\n\0\0";
        let program = parse_pixel_shader(source).unwrap();
        assert_eq!(program.constants, vec![(6, [0.75; 4])]);
        assert_eq!(program.instructions.len(), 6);
        let dp3 = &program.instructions[1];
        assert_eq!((dp3.op, dp3.mask, dp3.saturate), (PsOp::Dp3, 0b0111, true));
        assert_eq!(dp3.sources[0].modifier, PsSourceModifier::SignedScale);
        let mad = &program.instructions[2];
        assert_eq!((mad.shift, mad.sources[2].negate), (2, true));
        let mov = &program.instructions[4];
        assert!(mov.coissue);
        assert_eq!((mov.mask, mov.sources[0].replicate), (0b1000, Some(3)));
        assert_eq!(
            program.instructions[5].sources[1].modifier,
            PsSourceModifier::Bias
        );
    }

    #[test]
    fn rejects_other_versions() {
        assert!(parse_pixel_shader(b"ps.1.4\nmov r0, v0").is_err());
    }
}
