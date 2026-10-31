//! `--map 46.5N,0.3W`: a map coordinate as the viewer's block and position. The coordinate is
//! the form `Position::FromRealMapCoords` reads, parsed as the guide parses them
//! (`ac2_guide::coord`) and placed on its land cell by `LandDefs::game_map_coords_to_gid`
//! (`ac2_guide::nav::cell`), at the cell's middle as `Position::FromRealMapCoords` places it.

use ac2_physics::landdefs::SQUARE_LENGTH;

/// The block and the position within it a map coordinate names.
pub fn parse_map(text: &str) -> Result<((u8, u8), (f32, f32)), String> {
    let found = ac2_guide::coord::find(text.trim());
    let [(range, at)] = found.as_slice() else {
        return Err("--map: LATITUDE,LONGITUDE as 46.5N,0.3W".into());
    };
    if range.len() != text.trim().len() {
        return Err("--map: LATITUDE,LONGITUDE as 46.5N,0.3W".into());
    }
    let cell = ac2_guide::nav::cell(*at);
    if cell == 0 {
        return Err(format!("--map: {text} is off the map"));
    }
    // `Position::FromRealMapCoords`' `place_in_cell`: x from the cell's high local bits.
    let local = (cell & 0xFFFF) - 1;
    let middle = |n: u32| n as f32 * SQUARE_LENGTH + SQUARE_LENGTH * 0.5;
    Ok((
        ((cell >> 24) as u8, (cell >> 16) as u8),
        (middle(local >> 3), middle(local & 7)),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_map_coordinate_names_a_block_and_a_cell_middle() {
        let (block, (x, y)) = parse_map("46.5N,0.3W").unwrap();
        // 46.5 north and 0.3 west: just west of the map's middle column, north of its middle.
        assert_eq!(block.0, 0x7F);
        assert!(block.1 > 0x7F);
        assert_eq!(x % SQUARE_LENGTH, SQUARE_LENGTH * 0.5);
        assert!((0.0..160.0).contains(&y));
    }

    #[test]
    fn bad_coordinates_are_refused() {
        assert!(parse_map("nonsense").is_err());
        assert!(parse_map("46.5N").is_err());
        assert!(parse_map("146.5N,0.3W").is_err());
    }
}
