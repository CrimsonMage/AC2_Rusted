//! DirectMusic segments (`RIFF 'DMSG'`): a header (length, repeats, loop points, default
//! boundary) and tracks — tempo, time signature, sequence (MIDI events), chords, command
//! (groove), style, band, segment triggers, markers and script calls.

use super::band::{read_band, read_reference, Band, Reference};
use super::riff::{Chunk, Guid, Reader};

/// Track classes (`CLSID_DirectMusic*Track`).
pub mod class {
    use super::Guid;
    const DM: [u8; 8] = [0x87, 0x04, 0x00, 0x60, 0x08, 0x93, 0xB1, 0xBD];
    pub const TEMPO: Guid = Guid::from_parts(0xD2AC_2885, 0xB39B, 0x11D1, DM);
    pub const SEQUENCE: Guid = Guid::from_parts(0xD2AC_2886, 0xB39B, 0x11D1, DM);
    pub const TIME_SIG: Guid = Guid::from_parts(0xD2AC_2888, 0xB39B, 0x11D1, DM);
    pub const CHORD: Guid = Guid::from_parts(0xD2AC_288B, 0xB39B, 0x11D1, DM);
    pub const COMMAND: Guid = Guid::from_parts(0xD2AC_288C, 0xB39B, 0x11D1, DM);
    pub const STYLE: Guid = Guid::from_parts(0xD2AC_288D, 0xB39B, 0x11D1, DM);
    pub const BAND: Guid = Guid::from_parts(0xD2AC_2894, 0xB39B, 0x11D1, DM);
    /// The object classes references name.
    pub const SEGMENT_OBJECT: Guid = Guid::from_parts(0xD2AC_2882, 0xB39B, 0x11D1, DM);
    pub const STYLE_OBJECT: Guid = Guid::from_parts(0xD2AC_288A, 0xB39B, 0x11D1, DM);
}

/// `DMUS_IO_TIMESIGNATURE_ITEM` (time and signature).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeSignature {
    pub beats_per_measure: u8,
    /// The note value of a beat (4 a quarter, 8 an eighth).
    pub beat: u8,
    pub grids_per_beat: u16,
}

impl Default for TimeSignature {
    fn default() -> Self {
        Self {
            beats_per_measure: 4,
            beat: 4,
            grids_per_beat: 4,
        }
    }
}

/// `DMUS_IO_SEQ_ITEM`: a MIDI channel message with a duration for notes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeqItem {
    pub time: i32,
    pub duration: i32,
    pub pchannel: u32,
    pub offset: i16,
    pub status: u8,
    pub byte1: u8,
    pub byte2: u8,
}

/// `DMUS_IO_CURVE_ITEM`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurveItem {
    pub start: i32,
    pub duration: i32,
    pub reset_duration: i32,
    pub pchannel: u32,
    pub offset: i16,
    pub start_value: i16,
    pub end_value: i16,
    pub reset_value: i16,
    /// `DMUS_CURVET_*`: 3 pitch bend, 4 control change, 5 mono aftertouch, 6 poly aftertouch.
    pub kind: u8,
    /// `DMUS_CURVES_*`: 0 linear, 1 instant, 2 exponential, 3 logarithmic, 4 sine.
    pub shape: u8,
    pub cc: u8,
    pub flags: u8,
}

/// `DMUS_IO_SUBCHORD`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SubChord {
    pub chord_pattern: u32,
    pub scale_pattern: u32,
    pub inversion_points: u32,
    pub levels: u32,
    pub chord_root: u8,
    pub scale_root: u8,
}

/// `DMUS_IO_CHORD` with its subchords.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chord {
    pub name: String,
    pub time: i32,
    pub measure: u16,
    pub beat: u8,
    pub flags: u8,
    pub subchords: Vec<SubChord>,
}

/// `DMUS_IO_COMMAND`: groove level and embellishment changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Command {
    pub time: i32,
    pub measure: u16,
    pub beat: u8,
    pub command: u8,
    pub groove_level: u8,
    pub groove_range: u8,
    pub repeat_mode: u8,
}

/// A segment trigger (`DMUS_IO_SEGMENT_ITEM_HEADER` and what it plays).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SegmentTrigger {
    pub time: i32,
    pub physical_time: i32,
    /// `DMUS_SEGF_*` play flags.
    pub play_flags: u32,
    pub flags: u32,
    pub segment: Option<Reference>,
    /// A motif of the style at the time (`snam`).
    pub motif: Option<String>,
}

/// A script routine call (`DMUS_IO_SCRIPTTRACK_EVENTHEADER`, the script and routine).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptCall {
    pub flags: u32,
    pub time: i32,
    pub physical_time: i32,
    pub script: Option<Reference>,
    pub routine: String,
}

/// One track's data.
#[derive(Debug, Clone, PartialEq)]
pub enum TrackData {
    Tempo(Vec<(i32, f64)>),
    TimeSig(Vec<(i32, TimeSignature)>),
    Sequence {
        events: Vec<SeqItem>,
        curves: Vec<CurveItem>,
    },
    Chords {
        /// `crdh`: the key root in the top byte, the scale pattern below.
        scale: u32,
        chords: Vec<Chord>,
    },
    Commands(Vec<Command>),
    /// `(time, style reference)`.
    Styles(Vec<(i32, Reference)>),
    /// `(logical time, band)`.
    Bands(Vec<(i32, Band)>),
    Triggers(Vec<SegmentTrigger>),
    Markers(Vec<i32>),
    Scripts(Vec<ScriptCall>),
    Other,
}

/// A track (`DMUS_IO_TRACK_HEADER` and extras).
#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    pub class: Guid,
    pub position: u32,
    pub group: u32,
    pub config: u32,
    pub priority: u32,
    pub data: TrackData,
}

/// A segment (`DMUS_IO_SEGMENT_HEADER` and its tracks).
#[derive(Debug, Clone, PartialEq)]
pub struct Segment {
    pub guid: Option<Guid>,
    pub name: Option<String>,
    pub repeats: u32,
    /// `mtLength`, music time (768 per quarter note).
    pub length: i32,
    pub play_start: i32,
    pub loop_start: i32,
    pub loop_end: i32,
    /// `dwResolution`: the default `DMUS_SEGF_*` boundary.
    pub resolution: u32,
    pub flags: u32,
    pub tracks: Vec<Track>,
}

fn time_items(chunk: &Chunk<'_>) -> Vec<i32> {
    chunk.reader().sized_items(|r| r.i32())
}

fn track_data(class: Guid, track: &Chunk<'_>) -> TrackData {
    let child = |id: &[u8; 4]| track.child(id);
    let list = |form: &[u8; 4]| track.list(form);
    if let Some(tetr) = child(b"tetr") {
        return TrackData::Tempo(tetr.reader().sized_items(|r| {
            let time = r.i32();
            r.skip(4);
            (time, r.f64())
        }));
    }
    if let Some(tims) = list(b"TIMS").and_then(|l| l.child(b"tims")) {
        return TrackData::TimeSig(tims.reader().sized_items(|r| {
            let time = r.i32();
            let sig = TimeSignature {
                beats_per_measure: r.u8(),
                beat: r.u8(),
                grids_per_beat: r.u16(),
            };
            (time, sig)
        }));
    }
    if let Some(seqt) = child(b"seqt") {
        let mut events = Vec::new();
        let mut curves = Vec::new();
        for sub in super::riff::Chunks::new(seqt.data) {
            if &sub.id == b"evtl" {
                events = sub.reader().sized_items(|r| SeqItem {
                    time: r.i32(),
                    duration: r.i32(),
                    pchannel: r.u32(),
                    offset: r.i16(),
                    status: r.u8(),
                    byte1: r.u8(),
                    byte2: r.u8(),
                });
            } else if &sub.id == b"curl" {
                curves = sub.reader().sized_items(|r| CurveItem {
                    start: r.i32(),
                    duration: r.i32(),
                    reset_duration: r.i32(),
                    pchannel: r.u32(),
                    offset: r.i16(),
                    start_value: r.i16(),
                    end_value: r.i16(),
                    reset_value: r.i16(),
                    kind: r.u8(),
                    shape: r.u8(),
                    cc: r.u8(),
                    flags: r.u8(),
                });
            }
        }
        return TrackData::Sequence { events, curves };
    }
    if let Some(cord) = list(b"cord") {
        let scale = cord.child(b"crdh").map_or(0, |c| c.reader().u32());
        let chords = cord
            .children()
            .filter(|c| &c.id == b"crdb")
            .map(|c| read_chord(&mut c.reader()))
            .collect();
        return TrackData::Chords { scale, chords };
    }
    if let Some(cmnd) = child(b"cmnd") {
        return TrackData::Commands(cmnd.reader().sized_items(|r| Command {
            time: r.i32(),
            measure: r.u16(),
            beat: r.u8(),
            command: r.u8(),
            groove_level: r.u8(),
            groove_range: r.u8(),
            repeat_mode: r.u8(),
        }));
    }
    if let Some(sttr) = list(b"sttr") {
        let styles = sttr
            .children()
            .filter(|c| c.is_list(b"strf"))
            .filter_map(|c| {
                let time = c.child(b"stmp")?.reader().i32();
                Some((time, read_reference(&c.list(b"DMRF")?)))
            })
            .collect();
        return TrackData::Styles(styles);
    }
    // The band track is a `RIFF 'DMBT'` whose `lbdl` list holds the band items.
    if let Some(lbdl) = list(b"lbdl").or_else(|| list(b"DMBT").and_then(|b| b.list(b"lbdl"))) {
        let bands = lbdl
            .children()
            .filter(|c| c.is_list(b"lbnd"))
            .filter_map(|c| {
                let time = c
                    .child(b"bd2h")
                    .or_else(|| c.child(b"bdih"))
                    .map_or(0, |h| h.reader().i32());
                Some((time, read_band(&c.list(b"DMBD")?)))
            })
            .collect();
        return TrackData::Bands(bands);
    }
    if let Some(segt) = list(b"segt") {
        let triggers = segt.list(b"lsgl").map_or_else(Vec::new, |l| {
            l.children()
                .filter(|c| c.is_list(b"lseg"))
                .map(|c| {
                    let mut h = c.child(b"sgih").map_or(Reader::new(&[]), |h| h.reader());
                    SegmentTrigger {
                        time: h.i32(),
                        physical_time: h.i32(),
                        play_flags: h.u32(),
                        flags: h.u32(),
                        segment: c.list(b"DMRF").map(|r| read_reference(&r)),
                        motif: c.child(b"snam").map(|n| n.text()),
                    }
                })
                .collect()
        });
        return TrackData::Triggers(triggers);
    }
    if let Some(mark) = list(b"MARK") {
        return TrackData::Markers(
            mark.child(b"play")
                .map_or_else(Vec::new, |p| time_items(&p)),
        );
    }
    if let Some(scrt) = list(b"scrt") {
        let calls = scrt.list(b"scrl").map_or_else(Vec::new, |l| {
            l.children()
                .filter(|c| c.is_list(b"scre"))
                .map(|c| {
                    let mut h = c.child(b"scrh").map_or(Reader::new(&[]), |h| h.reader());
                    ScriptCall {
                        flags: h.u32(),
                        time: h.i32(),
                        physical_time: h.i32(),
                        script: c.list(b"DMRF").map(|r| read_reference(&r)),
                        routine: c.child(b"scrn").map(|n| n.text()).unwrap_or_default(),
                    }
                })
                .collect()
        });
        return TrackData::Scripts(calls);
    }
    let _ = class;
    TrackData::Other
}

/// `DMUS_IO_CHORD` (size-prefixed) then its subchords (count, size, items).
pub(crate) fn read_chord(r: &mut Reader<'_>) -> Chord {
    let size = r.u32() as usize;
    let start = r.position();
    let name_bytes = r.take(32);
    let name: Vec<u16> = name_bytes
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .take_while(|&u| u != 0)
        .collect();
    let time = r.i32();
    let measure = r.u16();
    let beat = r.u8();
    let flags = r.u8();
    r.seek(start + size);
    let count = r.u32() as usize;
    let item = r.u32() as usize;
    let subchords = (0..count.min(16))
        .map(|_| {
            let mut s = Reader::new(r.take(item));
            SubChord {
                chord_pattern: s.u32(),
                scale_pattern: s.u32(),
                inversion_points: s.u32(),
                levels: s.u32(),
                chord_root: s.u8(),
                scale_root: s.u8(),
            }
        })
        .collect();
    Chord {
        name: String::from_utf16_lossy(&name),
        time,
        measure,
        beat,
        flags,
        subchords,
    }
}

/// Reads a segment from its `RIFF 'DMSG'` chunk.
/// The segment's band: the first of its band tracks (`DMUS_IO_BAND_ITEM`, by logical time). The
/// performance and the synthesizer's channel bindings both use this one.
pub fn first_band(segment: &Segment) -> Option<&Band> {
    segment.tracks.iter().find_map(|track| match &track.data {
        TrackData::Bands(bands) => bands.first().map(|(_, band)| band),
        _ => None,
    })
}

pub fn read_segment(top: &Chunk<'_>) -> Segment {
    let mut h = top.child(b"segh").map_or(Reader::new(&[]), |c| c.reader());
    let repeats = h.u32();
    let length = h.i32();
    let play_start = h.i32();
    let loop_start = h.i32();
    let loop_end = h.i32();
    let resolution = h.u32();
    let _rt_length = h.f64();
    let flags = h.u32();
    let tracks = top.list(b"trkl").map_or_else(Vec::new, |list| {
        list.children()
            .filter(|c| c.is_list(b"DMTK"))
            .map(|t| {
                let mut th = t.child(b"trkh").map_or(Reader::new(&[]), |c| c.reader());
                let class = th.guid();
                let position = th.u32();
                let group = th.u32();
                let mut tx = t.child(b"trkx").map_or(Reader::new(&[]), |c| c.reader());
                Track {
                    class,
                    position,
                    group,
                    config: tx.u32(),
                    priority: tx.u32(),
                    data: track_data(class, &t),
                }
            })
            .collect()
    });
    Segment {
        guid: top.child(b"guid").map(|c| c.reader().guid()),
        name: top
            .list(b"UNFO")
            .and_then(|u| u.child(b"UNAM"))
            .map(|c| c.text()),
        repeats,
        length,
        play_start,
        loop_start,
        loop_end,
        resolution,
        flags,
        tracks,
    }
}
