use std::fmt;
use std::str::FromStr;

/// Retail-shaped Turbine GUID POD.
#[repr(C)]
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TurbineGuid {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseTurbineGuidError;

impl TurbineGuid {
    pub const NULL: Self = Self {
        data1: 0,
        data2: 0,
        data3: 0,
        data4: [0; 8],
    };

    pub const fn new(data1: u32, data2: u16, data3: u16, data4: [u8; 8]) -> Self {
        Self {
            data1,
            data2,
            data3,
            data4,
        }
    }

    pub const fn is_null(self) -> bool {
        self.data1 == 0
            && self.data2 == 0
            && self.data3 == 0
            && self.data4[0] == 0
            && self.data4[1] == 0
            && self.data4[2] == 0
            && self.data4[3] == 0
            && self.data4[4] == 0
            && self.data4[5] == 0
            && self.data4[6] == 0
            && self.data4[7] == 0
    }

    pub const fn as_bytes(self) -> [u8; 16] {
        let data1 = self.data1.to_le_bytes();
        let data2 = self.data2.to_le_bytes();
        let data3 = self.data3.to_le_bytes();
        [
            data1[0],
            data1[1],
            data1[2],
            data1[3],
            data2[0],
            data2[1],
            data3[0],
            data3[1],
            self.data4[0],
            self.data4[1],
            self.data4[2],
            self.data4[3],
            self.data4[4],
            self.data4[5],
            self.data4[6],
            self.data4[7],
        ]
    }
}

impl FromStr for TurbineGuid {
    type Err = ParseTurbineGuidError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        // Replaces Turbine_GUID::Convert canonical GUID parsing.
        if value.len() != 36 || !value.is_ascii() {
            return Err(ParseTurbineGuidError);
        }

        let bytes = value.as_bytes();
        if bytes[8] != b'-' || bytes[13] != b'-' || bytes[18] != b'-' || bytes[23] != b'-' {
            return Err(ParseTurbineGuidError);
        }

        let data1 = parse_hex_u32(&value[0..8])?;
        let data2 = parse_hex_u16(&value[9..13])?;
        let data3 = parse_hex_u16(&value[14..18])?;
        let data4 = [
            parse_hex_u8(&value[19..21])?,
            parse_hex_u8(&value[21..23])?,
            parse_hex_u8(&value[24..26])?,
            parse_hex_u8(&value[26..28])?,
            parse_hex_u8(&value[28..30])?,
            parse_hex_u8(&value[30..32])?,
            parse_hex_u8(&value[32..34])?,
            parse_hex_u8(&value[34..36])?,
        ];

        Ok(Self::new(data1, data2, data3, data4))
    }
}

fn parse_hex_u32(value: &str) -> Result<u32, ParseTurbineGuidError> {
    u32::from_str_radix(value, 16).map_err(|_| ParseTurbineGuidError)
}

fn parse_hex_u16(value: &str) -> Result<u16, ParseTurbineGuidError> {
    u16::from_str_radix(value, 16).map_err(|_| ParseTurbineGuidError)
}

fn parse_hex_u8(value: &str) -> Result<u8, ParseTurbineGuidError> {
    u8::from_str_radix(value, 16).map_err(|_| ParseTurbineGuidError)
}

impl fmt::Display for TurbineGuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
            self.data1,
            self.data2,
            self.data3,
            self.data4[0],
            self.data4[1],
            self.data4[2],
            self.data4[3],
            self.data4[4],
            self.data4[5],
            self.data4[6],
            self.data4[7],
        )
    }
}

impl fmt::Debug for TurbineGuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TurbineGuid({self})")
    }
}

#[cfg(test)]
mod tests {
    use super::{ParseTurbineGuidError, TurbineGuid};

    #[test]
    fn guid_formats_in_canonical_form() {
        let guid = TurbineGuid::new(
            0x83C0_809B,
            0xBC20,
            0x4CE8,
            [0x92, 0x27, 0xAA, 0x08, 0x06, 0x3F, 0x83, 0x05],
        );

        assert_eq!(guid.to_string(), "83C0809B-BC20-4CE8-9227-AA08063F8305");
        assert!(!guid.is_null());
        assert_eq!(guid.as_bytes().len(), 16);
    }

    #[test]
    fn null_guid_is_all_zeroes() {
        let guid = TurbineGuid::NULL;

        assert!(guid.is_null());
        assert_eq!(guid.as_bytes(), [0; 16]);
    }

    #[test]
    fn guid_parses_from_canonical_form() {
        let guid: TurbineGuid = "83C0809B-BC20-4CE8-9227-AA08063F8305".parse().unwrap();

        assert_eq!(guid.data1, 0x83C0_809B);
        assert_eq!(guid.data2, 0xBC20);
        assert_eq!(guid.data3, 0x4CE8);
        assert_eq!(guid.data4, [0x92, 0x27, 0xAA, 0x08, 0x06, 0x3F, 0x83, 0x05]);
        assert_eq!(guid.to_string(), "83C0809B-BC20-4CE8-9227-AA08063F8305");
    }

    #[test]
    fn guid_rejects_invalid_text() {
        assert_eq!(
            "83C0809B-BC20-4CE8-9227-AA08063F830X".parse::<TurbineGuid>(),
            Err(ParseTurbineGuidError)
        );
        assert!("83C0809B".parse::<TurbineGuid>().is_err());
    }
}
