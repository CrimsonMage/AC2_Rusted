//! The world map: the land drawn from its own terrain, finer as you zoom, the game's map notes
//! (towns and other places) and the AC2Hero guide's places, as an egui widget (`WorldMap`) that
//! the standalone tool and the world viewer show.

mod canvas;
mod map;
mod notes;
mod panel;
mod tiles;

pub use canvas::{colour, Player};
pub use map::{open_world, player_at, world_metres, MapAction, WorldMap};
