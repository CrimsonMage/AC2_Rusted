//! DirectMusic styles (`RIFF 'DMST'`): parts of chord-relative notes in variations, and patterns
//! that pick parts for a groove-level range, plus the style's bands.

use super::band::{read_band, Band};
use super::riff::{Chunk, Guid, Reader};
use super::segment::TimeSignature;

/// `DMUS_IO_STYLENOTE`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StyleNote {
    /// `mtGridStart`: in grids from the part's start.
    pub grid_start: i32,
    /// `dwVariation`: the variations (bits) that play it.
    pub variation: u32,
    pub duration: i32,
    pub time_offset: i16,
    /// `wMusicValue`: octave, chord tone, scale offset and accidental (or a MIDI note).
    pub music_value: u16,
    pub velocity: u8,
    pub time_range: u8,
    pub duration_range: u8,
    pub velocity_range: u8,
    pub inversion_id: u8,
    /// `bPlayModeFlags`: `DMUS_PLAYMODE_*`, where 16 (NONE) takes the part's mode and 0 is FIXED.
    pub play_mode: u8,
    pub note_flags: u8,
}

/// `DMUS_IO_STYLECURVE`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StyleCurve {
    pub grid_start: i32,
    pub variation: u32,
    pub duration: i32,
    pub reset_duration: i32,
    pub time_offset: i16,
    pub start_value: i16,
    pub end_value: i16,
    pub reset_value: i16,
    pub kind: u8,
    pub shape: u8,
    pub cc: u8,
    pub flags: u8,
}

/// A part (`DMUS_IO_STYLEPART` and its notes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Part {
    pub time_sig: TimeSignature,
    /// `dwVariationChoices`: for each variation, the chords it may play over.
    pub variation_choices: [u32; 32],
    pub guid: Guid,
    pub measures: u16,
    pub play_mode: u8,
    pub invert_upper: u8,
    pub invert_lower: u8,
    pub flags: u32,
    pub notes: Vec<StyleNote>,
    pub curves: Vec<StyleCurve>,
}

/// `DMUS_IO_PARTREF`: a pattern's use of a part on a performance channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PartRef {
    pub part: Guid,
    pub logical_part: u16,
    pub variation_lock: u8,
    pub subchord_level: u8,
    pub priority: u8,
    /// `DMUS_VARIATIONT_*`: how the next variation is chosen.
    pub random_variation: u8,
    pub pchannel: u32,
}

/// A pattern (`DMUS_IO_PATTERN`, rhythm and part references).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pattern {
    pub name: Option<String>,
    pub time_sig: TimeSignature,
    pub groove_bottom: u8,
    pub groove_top: u8,
    /// `wEmbellishment`: 0 a groove, else intro / fill / break / end / motif bits.
    pub embellishment: u16,
    pub measures: u16,
    pub dest_groove_bottom: u8,
    pub dest_groove_top: u8,
    pub flags: u32,
    /// `rhtm`: a chord rhythm per measure.
    pub rhythm: Vec<u32>,
    pub parts: Vec<PartRef>,
    pub band: Option<Band>,
}

/// A style.
#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    pub guid: Option<Guid>,
    pub name: Option<String>,
    pub time_sig: TimeSignature,
    pub tempo: f64,
    pub parts: Vec<Part>,
    pub patterns: Vec<Pattern>,
    pub bands: Vec<Band>,
}

impl Style {
    pub fn part(&self, guid: &Guid) -> Option<&Part> {
        self.parts.iter().find(|p| &p.guid == guid)
    }
}

fn time_sig(r: &mut Reader<'_>) -> TimeSignature {
    TimeSignature {
        beats_per_measure: r.u8(),
        beat: r.u8(),
        grids_per_beat: r.u16(),
    }
}

fn name(list: &Chunk<'_>) -> Option<String> {
    list.list(b"UNFO")
        .and_then(|u| u.child(b"UNAM"))
        .map(|c| c.text())
}

fn read_part(list: &Chunk<'_>) -> Option<Part> {
    let mut h = list.child(b"prth")?.reader();
    let time_sig = time_sig(&mut h);
    let mut variation_choices = [0u32; 32];
    for choice in &mut variation_choices {
        *choice = h.u32();
    }
    let guid = h.guid();
    let measures = h.u16();
    let play_mode = h.u8();
    let invert_upper = h.u8();
    let invert_lower = h.u8();
    h.skip(3);
    let flags = h.u32();
    let notes = list.child(b"note").map_or_else(Vec::new, |c| {
        c.reader().sized_items(|r| StyleNote {
            grid_start: r.i32(),
            variation: r.u32(),
            duration: r.i32(),
            time_offset: r.i16(),
            music_value: r.u16(),
            velocity: r.u8(),
            time_range: r.u8(),
            duration_range: r.u8(),
            velocity_range: r.u8(),
            inversion_id: r.u8(),
            play_mode: r.u8(),
            note_flags: r.u8(),
        })
    });
    let curves = list.child(b"crve").map_or_else(Vec::new, |c| {
        c.reader().sized_items(|r| StyleCurve {
            grid_start: r.i32(),
            variation: r.u32(),
            duration: r.i32(),
            reset_duration: r.i32(),
            time_offset: r.i16(),
            start_value: r.i16(),
            end_value: r.i16(),
            reset_value: r.i16(),
            kind: r.u8(),
            shape: r.u8(),
            cc: r.u8(),
            flags: r.u8(),
        })
    });
    Some(Part {
        time_sig,
        variation_choices,
        guid,
        measures,
        play_mode,
        invert_upper,
        invert_lower,
        flags,
        notes,
        curves,
    })
}

fn read_pattern(list: &Chunk<'_>) -> Option<Pattern> {
    let mut h = list.child(b"ptnh")?.reader();
    let time_sig = time_sig(&mut h);
    let groove_bottom = h.u8();
    let groove_top = h.u8();
    let embellishment = h.u16();
    let measures = h.u16();
    let dest_groove_bottom = h.u8();
    let dest_groove_top = h.u8();
    let flags = h.u32();
    let rhythm = list.child(b"rhtm").map_or_else(Vec::new, |c| {
        let mut r = c.reader();
        (0..c.data.len() / 4).map(|_| r.u32()).collect()
    });
    let parts = list
        .children()
        .filter(|c| c.is_list(b"pref"))
        .filter_map(|c| {
            let mut r = c.child(b"prfc")?.reader();
            let part = r.guid();
            let logical_part = r.u16();
            let variation_lock = r.u8();
            let subchord_level = r.u8();
            let priority = r.u8();
            let random_variation = r.u8();
            r.skip(2);
            Some(PartRef {
                part,
                logical_part,
                variation_lock,
                subchord_level,
                priority,
                random_variation,
                pchannel: r.u32(),
            })
        })
        .collect();
    Some(Pattern {
        name: name(list),
        time_sig,
        groove_bottom,
        groove_top,
        embellishment,
        measures,
        dest_groove_bottom,
        dest_groove_top,
        flags,
        rhythm,
        parts,
        band: list.list(b"DMBD").map(|b| read_band(&b)),
    })
}

/// Reads a style from its `RIFF 'DMST'` chunk.
pub fn read_style(top: &Chunk<'_>) -> Style {
    let mut h = top.child(b"styh").map_or(Reader::new(&[]), |c| c.reader());
    let time_sig = time_sig(&mut h);
    let tempo = h.f64();
    let mut bands: Vec<Band> = top
        .children()
        .filter(|c| c.is_list(b"DMBD"))
        .map(|c| read_band(&c))
        .collect();
    if let Some(lbnd) = top.list(b"lbnd") {
        bands.extend(
            lbnd.children()
                .filter(|c| c.is_list(b"DMBD"))
                .map(|c| read_band(&c)),
        );
    }
    Style {
        guid: top.child(b"guid").map(|c| c.reader().guid()),
        name: name(top),
        time_sig,
        tempo,
        parts: top
            .children()
            .filter(|c| c.is_list(b"part"))
            .filter_map(|c| read_part(&c))
            .collect(),
        patterns: top
            .children()
            .filter(|c| c.is_list(b"pttn"))
            .filter_map(|c| read_pattern(&c))
            .collect(),
        bands,
    }
}
