//! The quest guide (`ac2-guide`): the AC2Hero site, baked in at build time, with its map
//! coordinates made targets the player can steer to.
//!
//! - `html` reads the mirror (at build time, or at run time for a quick look at an edit);
//!   `facts` reads the quests' header fields and the item tables.
//! - `atlas` turns the guide's coordinates into map markers.
//! - `guide` is the baked form; `coord` finds coordinates in text; `nav` steers to them.
//! - `browse` lists a tab's pages for a search and names pages for sharing in chat; `picture`
//!   turns the site's pictures into pixels.

pub mod atlas;
pub mod browse;
pub mod coord;
pub mod facts;
pub mod guide;
pub mod html;
pub mod nav;
pub mod picture;

pub use guide::{Guide, Image, Item, Kind, Line, MapCoord, Page, QuestFacts, Reward, Span, Style};

static BAKED: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/ac2hero.bin"));

/// The guide baked in at build time (empty when `AC2HERO_DIR` was unset). Each call decodes a
/// new copy of the text (a few MB; the images borrow from the binary), so a caller keeps one while
/// it is in use and drops it when done.
pub fn baked() -> Guide {
    guide::decode(BAKED).expect("the baked guide decodes")
}
