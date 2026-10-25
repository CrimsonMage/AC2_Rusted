//! `gmNavigationUI`'s macro map mapping: landscape cells to pixels of the map field, back again,
//! the map field's fit into its parent (`SizeMap`, macro branch) and a marker's centring
//! (`CenterMarker`). Admin 7148 addresses; Legions 7160 has the same instructions.
//!
//! The map frame is the macro view's window on the land: `origin` (`MapX`, `MapY`, from the
//! layout's `MacroXOffset` and `MacroYOffset`) and `size` (`MapW`, `MapH`: the land's
//! `LAND_CELLS` times the layout's `MacroScale`, truncated). In the macro view the scale constant
//! is 238 (`ConvertLandscapeToPixel` at 0x431760: `fmul 0xa53504`); the other mode uses 250.
//! Retail keeps the intermediate values in the x87's 53-bit precision and rounds to `float` only
//! where it stores one, which is what the `f32` casts below mark.

/// `LandDefs::land_width` and `land_length`: 255 blocks of `1 << lblock_shift` (8) land cells.
pub const LAND_CELLS: i32 = 255 * 8;

/// `ConvertLandscapeToPixel`'s constant in the macro view (`0xa53504`, 238.0f).
pub const MACRO_SCALE: f32 = 238.0;
/// The constant of the other mode (`0xa53500`, 250.0f).
pub const DETAIL_SCALE: f32 = 250.0;
/// `ConvertPixelToLandscape`'s reciprocal in the macro view (`0xa5350c`, 1/238 as a float).
const MACRO_RECIPROCAL: f32 = 1.0 / 238.0;
/// `ConvertPixelToLandscape`'s reciprocal of the other mode (`0xa53508`, 1/250 as a float).
const DETAIL_RECIPROCAL: f32 = 1.0 / 250.0;

/// The map's window on the land (`gmNavigationUI`'s `MapX`, `MapY`, `MapW`, `MapH` and the
/// macro flag `m_fMacroMap`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MapFrame {
    pub origin: (i32, i32),
    pub size: (i32, i32),
    pub macro_map: bool,
}

/// `InitMacroMap`'s `MapW`/`MapH`: `(LAND_CELLS << lblock_shift)` times the scale, truncated
/// (`fild` of the land size, `fmul` by the `float` scale, `_ftol2`).
pub fn macro_extent(scale: f32) -> i32 {
    (f64::from(LAND_CELLS) * f64::from(scale)) as i32
}

/// Rounds an x87 result to the `float` it is stored as.
fn stored(value: f64) -> f32 {
    value as f32
}

impl MapFrame {
    fn scale(&self) -> f32 {
        if self.macro_map {
            MACRO_SCALE
        } else {
            DETAIL_SCALE
        }
    }

    fn reciprocal(&self) -> f32 {
        if self.macro_map {
            MACRO_RECIPROCAL
        } else {
            DETAIL_RECIPROCAL
        }
    }

    /// `gmNavigationUI::ConvertLandscapeToPixel`: the pixel of land cell `land` in a map field
    /// of `field` (width, height), and whether it lies inside that field. The pixel is written
    /// even when it lies outside (retail stores both before its bounds test), so callers that
    /// centre a marker off the map get the same numbers.
    ///
    /// `pY = ((lY - MapY) * C / MapH)` stored as a float, then `pY = C - pY`; `pX` likewise
    /// over `MapW`. Inside: `0 <= pY < field height` and `0 <= pX < field width`.
    pub fn landscape_to_pixel(&self, field: (i32, i32), land: (i32, i32)) -> ([f32; 2], bool) {
        let c = f64::from(self.scale());
        let dy = f64::from((land.1 - self.origin.1) as f32);
        let dx = f64::from((land.0 - self.origin.0) as f32);
        let y0 = stored(dy * c / f64::from(self.size.1));
        let y = stored(c - f64::from(y0));
        let x = stored(dx * c / f64::from(self.size.0));
        let inside = 0.0 <= f64::from(y)
            && f64::from(y) < f64::from(field.1)
            && 0.0 <= f64::from(x)
            && f64::from(x) < f64::from(field.0);
        ([x, y], inside)
    }

    /// `gmNavigationUI::ConvertPixelToLandscape`: the land cell under pixel `pixel` of the map
    /// field, when it is inside the map (`0 <= lY < MapH`, `0 <= lX < MapW`).
    pub fn pixel_to_landscape(&self, pixel: (i32, i32)) -> Option<(i32, i32)> {
        let k = f64::from(self.reciprocal());
        let ly = (((self.scale() as i32 - pixel.1) * self.size.1) as f64 * k) as i32;
        let lx = ((self.size.0 * pixel.0) as f64 * k) as i32;
        ((0..self.size.1).contains(&ly) && (0..self.size.0).contains(&lx))
            .then_some((lx + self.origin.0, ly + self.origin.1))
    }
}

/// `CenterMarker`'s offset: half the element's width and height, each truncated
/// (`fmul 0.5`, `_ftol2`). Retail subtracts these from the marker's pixel.
pub fn center_offset(size: (i32, i32)) -> (i32, i32) {
    (
        (f64::from(size.0) * 0.5) as i32,
        (f64::from(size.1) * 0.5) as i32,
    )
}

/// Where `SizeMap`'s macro branch puts the map field: its position in the parent and its size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapPlacement {
    pub origin: (i32, i32),
    pub size: (i32, i32),
}

/// `gmNavigationUI::SizeMap`, the macro branch (`MapField`'s image `image` (width, height) fitted
/// into the parent (width, height)). The scale is the parent's width over the image's width when
/// the image is wider than tall, else the parent's height over its height. The field is centred
/// across the leftover width and height; a wider image is pinned to the left, a taller one to
/// the top. Returns `None` for an image without a size (`InqImageSize` fails).
pub fn size_macro_map(parent: (i32, i32), image: (u32, u32)) -> Option<MapPlacement> {
    let (iw, ih) = (image.0 as i32, image.1 as i32);
    if iw <= 0 || ih <= 0 {
        return None;
    }
    let (pw, ph) = parent;
    let scale = if ih < iw {
        f64::from(pw) / f64::from(iw)
    } else {
        f64::from(ph) / f64::from(ih)
    };
    let new_w = stored(f64::from(iw) * scale);
    let new_h = stored(f64::from(ih) * scale);
    // `newX` stays in the x87 stack (not rounded); `newY` is stored, or zero when the image is
    // narrower than tall.
    let x = if iw > ih {
        0.0
    } else {
        (f64::from(pw) - f64::from(new_w)) * 0.5
    };
    let y = if iw < ih {
        0.0
    } else {
        f64::from(stored((f64::from(ph) - f64::from(new_h)) * 0.5))
    };
    Some(MapPlacement {
        origin: (x as i32, y as i32),
        size: (new_w as i32, new_h as i32),
    })
}
