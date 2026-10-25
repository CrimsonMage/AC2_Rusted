//! `IIDUtils`: what an object's 64-bit instance id says about it. The top byte of the high
//! word marks its kind (`IIDUtils::GetType`): 0x80 dynamic (server-made objects: creatures,
//! items), 0x40 static (landblock objects, the block in bits 8-31), 0x20 player; 0x11 UI,
//! 0x13 temporary, 0x12 the camera's two ids.

/// `IDType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdType {
    Unknown,
    Dynamic,
    Static,
    Player,
    Ui,
    Camera,
    Temporary,
}

/// `IIDUtils::GetType`: the first matching bit of the type byte, then the special kinds.
pub fn id_type(id: u64) -> IdType {
    let kind = (id >> 56) as u32;
    if kind & 0x80 != 0 {
        IdType::Dynamic
    } else if kind & 0x40 != 0 {
        IdType::Static
    } else if kind & 0x20 != 0 {
        IdType::Player
    } else if kind == 0x11 {
        IdType::Ui
    } else if id as u32 & 0x03FC_0000 == 0x0048_0000 {
        IdType::Camera
    } else if kind == 0x13 {
        IdType::Temporary
    } else {
        IdType::Unknown
    }
}

/// `IIDUtils::VerifyPlayer`.
pub fn is_player(id: u64) -> bool {
    id_type(id) == IdType::Player
}

/// `IIDUtils::VerifyDynamic`.
pub fn is_dynamic(id: u64) -> bool {
    id_type(id) == IdType::Dynamic
}

/// `IIDUtils::VerifyStatic`: an id of no known kind counts as static.
pub fn is_static(id: u64) -> bool {
    matches!(id_type(id), IdType::Static | IdType::Unknown)
}

/// `IIDUtils::ReadBlock`: a static id's landblock (`0xXXYYFFFF`).
pub fn read_block(id: u64) -> u32 {
    ((id >> 32) as u32 >> 8) << 16 | 0xFFFF
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_type_byte_names_the_kind() {
        assert_eq!(id_type(0x2000_0000_0000_0001), IdType::Player);
        assert_eq!(id_type(0x8130_0000_0000_0001), IdType::Dynamic);
        assert_eq!(id_type(0x7000_0000_0000_0042), IdType::Static);
        assert_eq!(id_type(0x5000_0000_0000_0001), IdType::Static);
        assert_eq!(id_type(0x1100_0000_0000_0001), IdType::Ui);
        assert_eq!(id_type(0x1300_0000_0000_0001), IdType::Temporary);
        assert_eq!(id_type(0x1200_0000_0048_0FFF), IdType::Camera);
        assert_eq!(id_type(1), IdType::Unknown);
        assert!(is_static(1) && !is_player(1));
        let s = 0x40A9_B412_0000_0007;
        assert!(is_static(s));
        assert_eq!(read_block(s), 0xA9B4_FFFF);
    }
}
