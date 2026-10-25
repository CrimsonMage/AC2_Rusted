//! DirectMusic bands (`RIFF 'DMBD'`) and object references (`LIST 'DMRF'`): a band assigns an
//! instrument (a patch from a referenced DLS collection, or General MIDI) with volume, pan and
//! transposition to each performance channel.

use super::riff::{Chunk, Guid, Reader};

/// `DMUS_IO_INST_*` flags of a band instrument.
pub mod inst {
    pub const PATCH: u32 = 0x1;
    pub const BANKSELECT: u32 = 0x2;
    pub const ASSIGN_PATCH: u32 = 0x8;
    pub const NOTERANGES: u32 = 0x10;
    pub const PAN: u32 = 0x20;
    pub const VOLUME: u32 = 0x40;
    pub const TRANSPOSE: u32 = 0x80;
    pub const GM: u32 = 0x100;
    pub const GS: u32 = 0x200;
    pub const XG: u32 = 0x400;
    pub const CHANNEL_PRIORITY: u32 = 0x800;
    pub const USE_DEFAULT_GM_SET: u32 = 0x1000;
    pub const PITCHBENDRANGE: u32 = 0x2000;
}

/// A reference to another DirectMusic object (`DMUS_IO_REFERENCE` and its descriptor chunks).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Reference {
    pub class: Guid,
    pub valid: u32,
    pub guid: Option<Guid>,
    pub name: Option<String>,
    pub file: Option<String>,
}

/// Reads a `LIST 'DMRF'`.
pub fn read_reference(list: &Chunk<'_>) -> Reference {
    let mut out = Reference::default();
    if let Some(h) = list.child(b"refh") {
        let mut r = h.reader();
        out.class = r.guid();
        out.valid = r.u32();
    }
    out.guid = list.child(b"guid").map(|c| c.reader().guid());
    out.name = list.child(b"name").map(|c| c.text());
    out.file = list.child(b"file").map(|c| c.text());
    out
}

/// `DMUS_IO_INSTRUMENT` with its collection reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BandInstrument {
    pub patch: u32,
    pub assign_patch: u32,
    pub note_ranges: [u32; 4],
    pub pchannel: u32,
    pub flags: u32,
    pub pan: u8,
    pub volume: u8,
    pub transpose: i16,
    pub channel_priority: u32,
    pub pitch_bend_range: i16,
    /// The DLS collection (none: the default General MIDI set).
    pub collection: Option<Reference>,
}

impl BandInstrument {
    /// The patch it plays (`ASSIGN_PATCH` replaces the patch).
    pub fn effective_patch(&self) -> u32 {
        if self.flags & inst::ASSIGN_PATCH != 0 {
            self.assign_patch
        } else {
            self.patch
        }
    }
}

/// A band (`DMBD`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Band {
    pub guid: Option<Guid>,
    pub name: Option<String>,
    pub instruments: Vec<BandInstrument>,
}

fn instrument(list: &Chunk<'_>) -> Option<BandInstrument> {
    let mut r: Reader<'_> = list.child(b"bins")?.reader();
    let patch = r.u32();
    let assign_patch = r.u32();
    let note_ranges = [r.u32(), r.u32(), r.u32(), r.u32()];
    Some(BandInstrument {
        patch,
        assign_patch,
        note_ranges,
        pchannel: r.u32(),
        flags: r.u32(),
        pan: r.u8(),
        volume: r.u8(),
        transpose: r.i16(),
        channel_priority: r.u32(),
        pitch_bend_range: r.i16(),
        collection: list.list(b"DMRF").map(|c| read_reference(&c)),
    })
}

/// Reads a band from its `RIFF 'DMBD'` chunk.
pub fn read_band(chunk: &Chunk<'_>) -> Band {
    Band {
        guid: chunk.child(b"guid").map(|c| c.reader().guid()),
        name: chunk
            .list(b"UNFO")
            .and_then(|u| u.child(b"UNAM"))
            .map(|c| c.text()),
        instruments: chunk.list(b"lbil").map_or_else(Vec::new, |list| {
            list.children()
                .filter(|c| c.is_list(b"lbin"))
                .filter_map(|c| instrument(&c))
                .collect()
        }),
    }
}
