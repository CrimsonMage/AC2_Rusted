//! The DirectMusic content of the `DBMusicObj` records (`0x24`): segments, styles, bands, DLS
//! collections, scripts and containers, read into plain data, and the library that resolves their
//! references to one another. Everything here runs on the loader thread.

pub mod band;
pub mod clock;
pub mod container;
pub mod curves;
pub mod dls;
pub mod encoded;
pub mod harmony;
pub mod library;
pub mod music_value;
pub mod performance;
pub mod riff;
pub mod segment;
pub mod style;
pub mod style_performance;
pub mod style_play;

use std::sync::Arc;

use band::Band;
use container::{Container, Script};
use dls::Collection;
use riff::{riff, Chunk, Guid, MusicFormatError, MusicResult};
use segment::Segment;
use style::Style;

pub use library::{MusicLibrary, MusicSource};

/// A wave played as a segment's only track (`DirectXMusicLoader::LoadFromObject` on an
/// `EncodedWave`: the file loads as a DirectMusic segment with a wave track). `seconds` is the
/// wave's time (`DBWave::GetTime`); `looping` is the segment's repeat, which runs until stopped.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WaveSegment {
    pub seconds: f64,
    pub looping: bool,
}

/// One DirectMusic object.
#[derive(Debug, Clone, PartialEq)]
pub enum MusicObject {
    Segment(Arc<Segment>),
    /// A segment whose track is a wave (see [`WaveSegment`]).
    Wave(WaveSegment),
    Style(Arc<Style>),
    Band(Arc<Band>),
    Collection(Arc<Collection>),
    Script(Arc<Script>),
    Container(Arc<Container>),
    /// An audiopath configuration or chordmap (not interpreted).
    Other([u8; 4]),
}

impl MusicObject {
    /// Its GUID (the DLS `dlid`), if it has one.
    pub fn guid(&self) -> Option<Guid> {
        match self {
            MusicObject::Segment(s) => s.guid,
            MusicObject::Wave(_) => None,
            MusicObject::Style(s) => s.guid,
            MusicObject::Band(b) => b.guid,
            MusicObject::Collection(c) => Some(c.guid),
            MusicObject::Script(s) => s.guid,
            MusicObject::Container(c) => c.guid,
            MusicObject::Other(_) => None,
        }
    }
}

/// Reads an object from its top `RIFF` chunk.
pub fn read_object_chunk(top: &Chunk<'_>) -> MusicResult<MusicObject> {
    let form = top.form.ok_or(MusicFormatError("not a RIFF form"))?;
    Ok(match &form {
        b"DMSG" => MusicObject::Segment(Arc::new(segment::read_segment(top))),
        b"DMST" => MusicObject::Style(Arc::new(style::read_style(top))),
        b"DMBD" => MusicObject::Band(Arc::new(band::read_band(top))),
        b"DLS " => MusicObject::Collection(Arc::new(dls::read_collection_chunk(top)?)),
        b"DMSC" => MusicObject::Script(Arc::new(container::read_script(top))),
        b"DMCN" => MusicObject::Container(Arc::new(container::read_container(top))),
        other => MusicObject::Other(*other),
    })
}

/// Reads a `DBMusicObj` record (`DBMusicObj::StreamPack`: the DID, the file's length, the
/// file).
pub fn read_music_record(did: u32, bytes: &[u8]) -> MusicResult<MusicObject> {
    let mut r = riff::Reader::new(bytes);
    if r.u32() != did {
        return Err(MusicFormatError("embedded DID did not match DAT identity"));
    }
    let length = r.u32() as usize;
    let file = bytes
        .get(8..8 + length)
        .ok_or(MusicFormatError("music file longer than the record"))?;
    read_object_chunk(&riff(file)?)
}

#[cfg(test)]
mod curve_dat_tests;
#[cfg(test)]
mod harmony_tests;
#[cfg(test)]
mod music_value_tests;
#[cfg(test)]
pub(crate) mod style_tests;
#[cfg(test)]
mod tests;
