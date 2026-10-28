//! The per-vertex words of a landblock (`CLandBlockData::m_height` and `m_cellinfo`): one height
//! byte and one packed word per vertex, the word holding the terrain, scene, encounter, colour,
//! road and replace fields whose widths `LandDefs::set_bitlengths` sets.

/// A field of a vertex's cell-info word, in packing order from the low bit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellInfoField {
    Terrain,
    Scene,
    Encounter,
    Color,
    Road,
    Replace,
}

/// The widths of the six fields; each field starts where the one before it ends
/// (`LandDefs::terrain_offset`, `scene_offset`, ..., `road_offset`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellInfoLayout {
    bits: [u32; 6],
}

impl CellInfoLayout {
    /// `LandDefs::set_bitlengths`: refused (None) when the fields together exceed the 32 bits of
    /// the word.
    pub fn new(bits: [u32; 6]) -> Option<Self> {
        (bits.iter().sum::<u32>() <= 32).then_some(Self { bits })
    }

    /// The bit a field starts at.
    pub fn offset(self, field: CellInfoField) -> u32 {
        self.bits[..field as usize].iter().sum()
    }

    /// A field's width in bits.
    pub fn bit_length(self, field: CellInfoField) -> u32 {
        self.bits[field as usize]
    }

    fn mask(self, field: CellInfoField) -> u32 {
        let bits = self.bit_length(field);
        if bits >= 32 {
            u32::MAX
        } else {
            (1 << bits) - 1
        }
    }

    fn fits(self, field: CellInfoField) -> bool {
        self.offset(field) + self.bit_length(field) <= 32
    }

    /// `CLandBlockData::GetScene`, `GetRoad`, ...: the field of a word, 0 for a field that runs
    /// past the word.
    pub fn get(self, field: CellInfoField, word: u32) -> u32 {
        if !self.fits(field) {
            return 0;
        }
        (word & (self.mask(field) << self.offset(field))) >> self.offset(field)
    }

    /// `CLandBlockData::SetTerrain`, `SetScene`, `SetEncounter`, `SetRoad`: the word with the
    /// field replaced; unchanged when the field runs past the word or the value does not fit it.
    pub fn set(self, field: CellInfoField, word: u32, value: u32) -> u32 {
        let mask = self.mask(field);
        if !self.fits(field) || value > mask {
            return word;
        }
        let offset = self.offset(field);
        (word & !(mask << offset)) | (value << offset)
    }
}

/// `CLandBlockData`'s vertex grid: `side * side` height bytes and cell-info words, x-major
/// (`side_vertex_count * ix + iy`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LandBlockCellInfo {
    side: usize,
    layout: CellInfoLayout,
    heights: Vec<u8>,
    cell_info: Vec<u32>,
}

impl LandBlockCellInfo {
    pub fn new(side: usize, layout: CellInfoLayout) -> Self {
        Self {
            side,
            layout,
            heights: vec![0; side * side],
            cell_info: vec![0; side * side],
        }
    }

    fn index(&self, ix: usize, iy: usize) -> usize {
        self.side * ix + iy
    }

    /// `CLandBlockData::SetHeight`.
    pub fn set_height(&mut self, ix: usize, iy: usize, height_bits: u8) {
        let index = self.index(ix, iy);
        self.heights[index] = height_bits;
    }

    /// The height byte at a vertex.
    pub fn height(&self, ix: usize, iy: usize) -> u8 {
        self.heights[self.index(ix, iy)]
    }

    fn set_field(&mut self, field: CellInfoField, ix: usize, iy: usize, value: u32) {
        let index = self.index(ix, iy);
        self.cell_info[index] = self.layout.set(field, self.cell_info[index], value);
    }

    fn get_field(&self, field: CellInfoField, ix: usize, iy: usize) -> u32 {
        self.layout.get(field, self.cell_info[self.index(ix, iy)])
    }

    /// `CLandBlockData::SetTerrain`.
    pub fn set_terrain(&mut self, ix: usize, iy: usize, terrain_type: u32) {
        self.set_field(CellInfoField::Terrain, ix, iy, terrain_type);
    }

    /// `CLandBlockData::SetScene`.
    pub fn set_scene(&mut self, ix: usize, iy: usize, scene_type: u32) {
        self.set_field(CellInfoField::Scene, ix, iy, scene_type);
    }

    /// `CLandBlockData::SetEncounter`.
    pub fn set_encounter(&mut self, ix: usize, iy: usize, encounter_type: u32) {
        self.set_field(CellInfoField::Encounter, ix, iy, encounter_type);
    }

    /// `CLandBlockData::SetRoad`.
    pub fn set_road(&mut self, ix: usize, iy: usize, road_type: u32) {
        self.set_field(CellInfoField::Road, ix, iy, road_type);
    }

    /// `CLandBlockData::GetScene`.
    pub fn scene(&self, ix: usize, iy: usize) -> u32 {
        self.get_field(CellInfoField::Scene, ix, iy)
    }

    /// The terrain type of a vertex.
    pub fn terrain(&self, ix: usize, iy: usize) -> u32 {
        self.get_field(CellInfoField::Terrain, ix, iy)
    }

    /// The road type of a vertex (`CLandBlockData::GetRoad`).
    pub fn road(&self, ix: usize, iy: usize) -> u32 {
        self.get_field(CellInfoField::Road, ix, iy)
    }

    /// The encounter type of a vertex.
    pub fn encounter(&self, ix: usize, iy: usize) -> u32 {
        self.get_field(CellInfoField::Encounter, ix, iy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layout() -> CellInfoLayout {
        // terrain 5, scene 4, encounter 4, colour 8, road 4, replace 7: the AC2 widths.
        CellInfoLayout::new([5, 4, 4, 8, 4, 7]).unwrap()
    }

    #[test]
    fn fields_pack_low_to_high_and_never_disturb_each_other() {
        let layout = layout();
        assert_eq!(layout.offset(CellInfoField::Terrain), 0);
        assert_eq!(layout.offset(CellInfoField::Scene), 5);
        assert_eq!(layout.offset(CellInfoField::Road), 21);
        let mut grid = LandBlockCellInfo::new(3, layout);
        grid.set_terrain(1, 2, 0x13);
        grid.set_scene(1, 2, 0xA);
        grid.set_road(1, 2, 3);
        grid.set_encounter(1, 2, 7);
        assert_eq!(grid.terrain(1, 2), 0x13);
        assert_eq!(grid.scene(1, 2), 0xA);
        assert_eq!(grid.road(1, 2), 3);
        assert_eq!(grid.encounter(1, 2), 7);
        // Overwriting one field keeps the others, and the neighbours are untouched.
        grid.set_scene(1, 2, 1);
        assert_eq!((grid.terrain(1, 2), grid.scene(1, 2)), (0x13, 1));
        assert_eq!(grid.terrain(2, 1), 0);
        grid.set_height(2, 0, 200);
        assert_eq!(grid.height(2, 0), 200);
    }

    #[test]
    fn a_value_wider_than_its_field_is_refused() {
        let mut grid = LandBlockCellInfo::new(2, layout());
        grid.set_scene(0, 0, 5);
        grid.set_scene(0, 0, 16);
        assert_eq!(grid.scene(0, 0), 5);
    }

    #[test]
    fn widths_over_the_word_are_refused_and_a_field_past_it_reads_zero() {
        assert!(CellInfoLayout::new([10, 10, 10, 10, 0, 0]).is_none());
        assert!(CellInfoLayout::new([16, 16, 1, 0, 0, 0]).is_none());
        // A road field that starts inside the word but ends past it reads as no road.
        let layout = CellInfoLayout::new([16, 8, 0, 0, 8, 0]).unwrap();
        assert_eq!(layout.get(CellInfoField::Road, u32::MAX), 0xFF);
        let wide = CellInfoLayout {
            bits: [16, 8, 0, 0, 16, 0],
        };
        assert_eq!(wide.get(CellInfoField::Road, u32::MAX), 0);
        assert_eq!(wide.set(CellInfoField::Road, 5, 1), 5);
    }
}
