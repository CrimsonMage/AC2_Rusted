//! Process-wide landblock lattice: the client's `LandDefs` globals.
//!
//! The world is addressed as 255 x 255 landblocks of 8 x 8 land cells. Asheron's Call 2
//! (`LandscapeDefs` 0x36000000) uses 20 m cells and 160 m blocks with 2 terrain vertices per cell
//! side (17 x 17 per block).
//!
//! `LandDefs::init_misc` installs these values once from data at startup; the composition root
//! does the same here with [`set_land_lattice`] before any world geometry is produced.

use std::sync::atomic::{AtomicU32, Ordering};

/// Landblock lattice parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LandLattice {
    cell_size: f32,
    vertices_per_cell: u32,
}

impl LandLattice {
    /// Asheron's Call 2: 20 m cells, two terrain polygons per cell side.
    pub const AC2: Self = Self {
        cell_size: 20.0,
        vertices_per_cell: 2,
    };

    pub const fn new(cell_size: f32, vertices_per_cell: u32) -> Self {
        Self {
            cell_size,
            vertices_per_cell,
        }
    }

    pub const fn cell_size(self) -> f32 {
        self.cell_size
    }

    pub const fn vertices_per_cell(self) -> u32 {
        self.vertices_per_cell
    }
}

static CELL_SIZE_BITS: AtomicU32 = AtomicU32::new(0x41A0_0000); // 20.0_f32
static VERTICES_PER_CELL: AtomicU32 = AtomicU32::new(2);

/// Installs the lattice for this process. Call once from the composition root before building
/// world geometry; changing it while landblocks are resident would mix two scales.
pub fn set_land_lattice(lattice: LandLattice) {
    CELL_SIZE_BITS.store(lattice.cell_size.to_bits(), Ordering::Release);
    VERTICES_PER_CELL.store(lattice.vertices_per_cell.max(1), Ordering::Release);
}

pub fn land_lattice() -> LandLattice {
    LandLattice {
        cell_size: land_cell_size(),
        vertices_per_cell: VERTICES_PER_CELL.load(Ordering::Acquire),
    }
}

/// Metric side length of one land cell (`LandDefs::square_length`).
pub fn land_cell_size() -> f32 {
    f32::from_bits(CELL_SIZE_BITS.load(Ordering::Acquire))
}

/// Metric side length of one landblock (`LandDefs::block_length`).
pub fn land_block_size() -> f32 {
    land_cell_size() * crate::LAND_BLOCK_CELLS_PER_SIDE as f32
}

#[cfg(test)]
mod tests {
    use super::LandLattice;

    #[test]
    fn default_matches_the_ac2_land_defs() {
        assert_eq!(f32::from_bits(0x41A0_0000), 20.0);
        assert_eq!(super::land_lattice(), LandLattice::AC2);
        assert_eq!(super::land_block_size(), 160.0);
    }
}
