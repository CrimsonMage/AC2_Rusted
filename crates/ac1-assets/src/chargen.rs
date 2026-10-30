//! `CharGen` (`0x0E000002`): what a new AC1 character of each heritage and sex is made of. A
//! player's body is its Setup dressed by the server's `ObjDesc` built from these choices
//! (ACE's `WorldObject.AddBaseModelData`): the hairstyle's head (part 16) and its texture
//! changes, the eye, nose and mouth strips' texture changes on the head, and the sex's base
//! palette with the skin (colours 0 to 191), hair (192 to 255) and eye (256 to 319) palette
//! sets over it. Without them the Setup's own head is a faceless placeholder.

use crate::dats::Ac1Dats;
use crate::formats::{Ac1Error, Ac1Result, Reader};
use crate::weenie::AceWeenie;

/// The `CharGen` record.
pub const CHAR_GEN: u32 = 0x0E00_0002;
/// The shade the skin, hair and eye palette sets are taken at: the middle of the sliders.
const DEFAULT_SHADE: f64 = 0.5;

/// An `ObjDesc` as `CharGen` stores it (`ObjDesc::UnPack`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CgObjDesc {
    /// (part, old SurfaceTexture, new SurfaceTexture).
    pub textures: Vec<(u32, u32, u32)>,
    /// (part, GfxObj).
    pub parts: Vec<(u32, u32)>,
}

/// One sex of one heritage (`SexCG`), as far as the body's look needs.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CgSex {
    pub setup: u32,
    pub motion_table: u32,
    pub base_palette: u32,
    pub skin_palette_set: u32,
    pub base: CgObjDesc,
    pub hair_colors: Vec<u32>,
    /// (bald, its `ObjDesc`).
    pub hair_styles: Vec<(bool, CgObjDesc)>,
    pub eye_colors: Vec<u32>,
    /// (with hair, bald).
    pub eye_strips: Vec<(CgObjDesc, CgObjDesc)>,
    pub nose_strips: Vec<CgObjDesc>,
    pub mouth_strips: Vec<CgObjDesc>,
}

impl CgSex {
    /// The body as a weenie the creature builder dresses (`Ac1Dats::body_pieces`): the first
    /// hairstyle, eyes, nose and mouth, and the first hair and eye colours, every palette set at
    /// the middle shade.
    pub fn default_look(&self) -> AceWeenie {
        let mut look = AceWeenie {
            setup: self.setup,
            motion_table: self.motion_table,
            palette_base: self.base_palette,
            shade: DEFAULT_SHADE,
            has_shade: true,
            scale: 1.0,
            ..Default::default()
        };
        let mut add = |desc: &CgObjDesc| {
            look.anim_parts.extend(desc.parts.iter().copied());
            look.texture_maps.extend(desc.textures.iter().copied());
        };
        add(&self.base);
        let bald = self.hair_styles.first().is_some_and(|(bald, _)| *bald);
        if let Some((_, hair)) = self.hair_styles.first() {
            add(hair);
        }
        if let Some((eyes, eyes_bald)) = self.eye_strips.first() {
            add(if bald { eyes_bald } else { eyes });
        }
        for strip in [self.nose_strips.first(), self.mouth_strips.first()]
            .into_iter()
            .flatten()
        {
            add(strip);
        }
        // `weenie_properties_palette` ranges are in units of eight colours.
        if self.skin_palette_set != 0 {
            look.palettes.push((self.skin_palette_set, 0x0, 0x18));
        }
        if let Some(&hair) = self.hair_colors.first() {
            look.palettes.push((hair, 0x18, 0x8));
        }
        if let Some(&eyes) = self.eye_colors.first() {
            look.palettes.push((eyes, 0x20, 0x8));
        }
        look
    }
}

impl Ac1Dats {
    /// The `CharGen` sex whose Setup is `setup`, if any heritage has one.
    pub fn char_gen_sex(&self, setup: u32) -> Ac1Result<Option<CgSex>> {
        let sexes = self.decode(CHAR_GEN, decode_char_gen)?;
        Ok(sexes.into_iter().find(|sex| sex.setup == setup))
    }
}

/// `CharGen::UnPack`, keeping each heritage's sexes.
pub fn decode_char_gen(did: u32, bytes: &[u8]) -> Ac1Result<Vec<CgSex>> {
    let mut r = Reader::new(did, bytes);
    r.header()?;
    r.skip(4, "missing CharGen field")?;
    for _ in 0..smart_count(&mut r)? {
        net_string(&mut r)?;
        // Each starting place: a cell and a frame.
        let places = smart_count(&mut r)?;
        r.skip(places * 32, "missing starting places")?;
    }
    r.skip(1, "missing CharGen flag")?;
    let mut sexes = Vec::new();
    for _ in 0..smart_count(&mut r)? {
        r.u32("missing heritage key")?;
        heritage(&mut r, &mut sexes)?;
    }
    Ok(sexes)
}

/// `HeritageGroupCG::UnPack`, adding its sexes to `sexes`.
fn heritage(r: &mut Reader<'_>, sexes: &mut Vec<CgSex>) -> Ac1Result<()> {
    net_string(r)?;
    // Icon, Setup, environment Setup, attribute and skill credits.
    r.skip(20, "missing heritage fields")?;
    for _ in 0..2 {
        let areas = smart_count(r)?;
        r.skip(areas * 4, "missing start areas")?;
    }
    let skills = smart_count(r)?;
    r.skip(skills * 12, "missing heritage skills")?;
    for _ in 0..smart_count(r)? {
        net_string(r)?;
        // Icon, title, six attributes.
        r.skip(32, "missing template fields")?;
        for _ in 0..2 {
            let listed = smart_count(r)?;
            r.skip(listed * 4, "missing template skills")?;
        }
    }
    r.skip(1, "missing heritage flag")?;
    for _ in 0..smart_count(r)? {
        r.u32("missing sex key")?;
        sexes.push(sex(r)?);
    }
    Ok(())
}

/// `SexCG::UnPack`.
fn sex(r: &mut Reader<'_>) -> Ac1Result<CgSex> {
    net_string(r)?;
    r.u32("missing sex scale")?;
    let setup = r.u32("missing sex Setup")?;
    r.u32("missing sex sound table")?;
    r.u32("missing sex icon")?;
    let base_palette = r.u32("missing sex palette")?;
    let skin_palette_set = r.u32("missing sex skin palettes")?;
    r.u32("missing sex physics table")?;
    let motion_table = r.u32("missing sex motion table")?;
    r.u32("missing sex combat table")?;
    let base = obj_desc(r)?;
    let hair_colors = u32_list(r)?;
    let mut hair_styles = Vec::new();
    for _ in 0..smart_count(r)? {
        r.u32("missing hairstyle icon")?;
        let bald = r.u8("missing hairstyle bald")? == 1;
        r.u32("missing hairstyle Setup")?;
        hair_styles.push((bald, obj_desc(r)?));
    }
    let eye_colors = u32_list(r)?;
    let mut eye_strips = Vec::new();
    for _ in 0..smart_count(r)? {
        r.skip(8, "missing eye strip icons")?;
        eye_strips.push((obj_desc(r)?, obj_desc(r)?));
    }
    let mut face_strips = || -> Ac1Result<Vec<CgObjDesc>> {
        (0..smart_count(r)?)
            .map(|_| {
                r.u32("missing face strip icon")?;
                obj_desc(r)
            })
            .collect()
    };
    let nose_strips = face_strips()?;
    let mouth_strips = face_strips()?;
    // Headgear, shirts, pants, footwear: a name, a ClothingTable and a weenie each.
    for _ in 0..4 {
        for _ in 0..smart_count(r)? {
            net_string(r)?;
            r.skip(8, "missing gear fields")?;
        }
    }
    u32_list(r)?;
    Ok(CgSex {
        setup,
        motion_table,
        base_palette,
        skin_palette_set,
        base,
        hair_colors,
        hair_styles,
        eye_colors,
        eye_strips,
        nose_strips,
        mouth_strips,
    })
}

/// `ObjDesc::UnPack`: aligned, a count byte each for sub-palettes, texture changes and part
/// changes, the palette when there are sub-palettes, then each list; aligned again.
fn obj_desc(r: &mut Reader<'_>) -> Ac1Result<CgObjDesc> {
    r.align4("missing ObjDesc")?;
    r.u8("missing ObjDesc marker")?;
    let palettes = r.u8("missing ObjDesc palette count")?;
    let textures = r.u8("missing ObjDesc texture count")?;
    let parts = r.u8("missing ObjDesc part count")?;
    if palettes > 0 {
        packed_did(r, 0x0400_0000)?;
    }
    for _ in 0..palettes {
        packed_did(r, 0x0400_0000)?;
        r.skip(2, "missing sub-palette range")?;
    }
    let mut desc = CgObjDesc::default();
    for _ in 0..textures {
        let part = u32::from(r.u8("missing texture change part")?);
        let old = packed_did(r, 0x0500_0000)?;
        let new = packed_did(r, 0x0500_0000)?;
        desc.textures.push((part, old, new));
    }
    for _ in 0..parts {
        let part = u32::from(r.u8("missing part change part")?);
        desc.parts.push((part, packed_did(r, 0x0100_0000)?));
    }
    r.align4("missing ObjDesc end")?;
    Ok(desc)
}

/// A `DataID` of a known type: 16 bits, or 30 when the first has its top bit set.
fn packed_did(r: &mut Reader<'_>, kind: u32) -> Ac1Result<u32> {
    let first = u32::from(r.u16("missing packed DID")?);
    if first & 0x8000 == 0 {
        return Ok(kind + first);
    }
    let low = u32::from(r.u16("missing packed DID low half")?);
    Ok(kind + (((first & 0x3FFF) << 16) | low))
}

/// A `SmartArray` count: one, two or four bytes by its first byte's top bits.
fn smart_count(r: &mut Reader<'_>) -> Ac1Result<usize> {
    let b0 = u32::from(r.u8("missing count")?);
    let count = if b0 & 0x80 == 0 {
        b0
    } else {
        let b1 = u32::from(r.u8("missing count")?);
        if b0 & 0x40 == 0 {
            ((b0 & 0x7F) << 8) | b1
        } else {
            let low = u32::from(r.u16("missing count")?);
            ((((b0 & 0x3F) << 8) | b1) << 16) | low
        }
    };
    if count > 1 << 16 {
        return Err(Ac1Error {
            did: CHAR_GEN,
            context: "CharGen count out of range",
        });
    }
    Ok(count as usize)
}

/// A `SmartArray` of `u32`s.
fn u32_list(r: &mut Reader<'_>) -> Ac1Result<Vec<u32>> {
    (0..smart_count(r)?)
        .map(|_| r.u32("missing list entry"))
        .collect()
}

/// A string as `BinaryWriter` writes it: a 7-bit varint length, then its bytes.
fn net_string(r: &mut Reader<'_>) -> Ac1Result<()> {
    let mut len = 0usize;
    for shift in (0..35).step_by(7) {
        let b = r.u8("missing string length")?;
        len |= usize::from(b & 0x7F) << shift;
        if b & 0x80 == 0 {
            break;
        }
    }
    r.bytes(len, "missing string").map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_packed_did_takes_two_or_four_bytes() {
        let short = [0x34, 0x12];
        let mut r = Reader::new(0, &short);
        assert_eq!(packed_did(&mut r, 0x0500_0000).unwrap(), 0x0500_1234);
        // 0x8001 0x2345: the top bit marks the long form, 0x3FFF of the first half is high.
        let long = [0x01, 0x80, 0x45, 0x23];
        let mut r = Reader::new(0, &long);
        assert_eq!(packed_did(&mut r, 0x0100_0000).unwrap(), 0x0101_2345);
    }

    #[test]
    fn a_smart_count_takes_one_two_or_four_bytes() {
        let mut r = Reader::new(0, &[0x05]);
        assert_eq!(smart_count(&mut r).unwrap(), 5);
        let mut r = Reader::new(0, &[0x81, 0x02]);
        assert_eq!(smart_count(&mut r).unwrap(), 0x102);
    }

    /// The AC1 Lugian male's `CharGen` sex gives its head and face (data test).
    #[test]
    fn the_lugian_male_has_a_head_and_face() {
        let Some(dats) = Ac1Dats::from_env() else {
            return;
        };
        let dats = dats.expect("AC1 DATs mount");
        let sex = dats
            .char_gen_sex(0x0200_1A18)
            .expect("CharGen decodes")
            .expect("a heritage has the Lugian male");
        let look = sex.default_look();
        assert!(
            look.anim_parts.iter().any(|(part, _)| *part == 16),
            "{look:?}"
        );
        assert!(
            look.texture_maps.iter().any(|(part, _, _)| *part == 16),
            "{look:?}"
        );
        assert_ne!(look.palette_base, 0);
        assert!(!look.palettes.is_empty());
    }
}
