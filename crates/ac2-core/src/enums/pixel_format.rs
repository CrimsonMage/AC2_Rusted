/// Pixel format of a decoded texture surface. Values follow Direct3D `D3DFORMAT`; block formats
/// use their FourCC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct PixelFormatID(pub u32);

impl PixelFormatID {
    pub const PFID_UNKNOWN: Self = Self(0);
    pub const PFID_A8R8G8B8: Self = Self(21);
    pub const PFID_P8: Self = Self(41);
    pub const PFID_INDEX16: Self = Self(101);
    pub const PFID_DXT1: Self = Self(u32::from_le_bytes(*b"DXT1"));
    pub const PFID_DXT2: Self = Self(u32::from_le_bytes(*b"DXT2"));
    pub const PFID_DXT3: Self = Self(u32::from_le_bytes(*b"DXT3"));
    pub const PFID_DXT4: Self = Self(u32::from_le_bytes(*b"DXT4"));
    pub const PFID_DXT5: Self = Self(u32::from_le_bytes(*b"DXT5"));
}
