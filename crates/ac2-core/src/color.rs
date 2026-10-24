/// Minimal byte-reader seam used to decode shared packed scalar values without coupling
/// foundation to a particular archive implementation.
pub trait ColorArgbByteReader {
    type Error;

    fn read_u8(&mut self) -> Result<u8, Self::Error>;
}

/// Generic packed BGRA value shared by authored light and property records.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ColorArgb {
    pub blue: u8,
    pub green: u8,
    pub red: u8,
    pub alpha: u8,
}

impl ColorArgb {
    pub fn decode<R: ColorArgbByteReader>(reader: &mut R) -> Result<Self, R::Error> {
        Ok(Self {
            blue: reader.read_u8()?,
            green: reader.read_u8()?,
            red: reader.read_u8()?,
            alpha: reader.read_u8()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{ColorArgb, ColorArgbByteReader};

    struct Reader<'a> {
        bytes: &'a [u8],
        position: usize,
    }

    impl ColorArgbByteReader for Reader<'_> {
        type Error = &'static str;

        fn read_u8(&mut self) -> Result<u8, Self::Error> {
            let byte = self
                .bytes
                .get(self.position)
                .copied()
                .ok_or("unexpected eof")?;
            self.position += 1;
            Ok(byte)
        }
    }

    #[test]
    fn decodes_retail_bgra_order() {
        let mut reader = Reader {
            bytes: &[0x10, 0x20, 0x30, 0x40],
            position: 0,
        };

        assert_eq!(
            ColorArgb::decode(&mut reader).unwrap(),
            ColorArgb {
                blue: 0x10,
                green: 0x20,
                red: 0x30,
                alpha: 0x40,
            }
        );
    }
}
