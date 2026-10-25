//! DLS collections (Downloadable Sounds level 1 and 2, `RIFF 'DLS '`): instruments of key- and
//! velocity-ranged regions, each a wave from the collection's pool with its sample loops
//! (`wsmp`), tuning and articulation (`art1` / `art2` connection blocks).

use std::sync::Arc;

use super::riff::{riff, Chunk, Guid, MusicFormatError, MusicResult, Reader};
use crate::data::Pcm;

/// `F_INSTRUMENT_DRUMS`.
pub const DRUMS: u32 = 0x8000_0000;

/// One `CONNECTION` block: source, control, destination, transform and scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Connection {
    pub source: u16,
    pub control: u16,
    pub destination: u16,
    pub transform: u16,
    pub scale: i32,
}

/// An articulation (`lart` with `art1`, or `lar2` with `art2`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Articulation {
    pub connections: Vec<Connection>,
}

/// A `WLOOP`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SampleLoop {
    /// `WLOOP_TYPE_FORWARD` (0) or `WLOOP_TYPE_RELEASE` (1, DLS2).
    pub kind: u32,
    pub start: u32,
    pub length: u32,
}

/// `WSMPL`: tuning, gain and loops.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WaveSample {
    pub unity_note: u16,
    /// Cents, signed 16-bit (`sFineTune`).
    pub fine_tune: i16,
    /// Gain in 1/655360 dB.
    pub attenuation: i32,
    pub options: u32,
    pub loops: Vec<SampleLoop>,
}

/// `WAVELINK`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WaveLink {
    pub options: u16,
    pub phase_group: u16,
    pub channel: u32,
    pub table_index: u32,
}

/// A region (`rgn ` / `rgn2`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Region {
    pub keys: (u16, u16),
    pub velocities: (u16, u16),
    pub options: u16,
    pub key_group: u16,
    pub sample: Option<WaveSample>,
    pub link: WaveLink,
    pub articulation: Option<Articulation>,
}

impl Region {
    /// Whether `key` at `velocity` plays it (a zero velocity range means any velocity).
    pub fn covers(&self, key: u8, velocity: u8) -> bool {
        let (k, v) = (u16::from(key), u16::from(velocity));
        let velocity_any = self.velocities == (0, 0);
        (self.keys.0..=self.keys.1).contains(&k)
            && (velocity_any || (self.velocities.0..=self.velocities.1).contains(&v))
    }
}

/// An instrument (`ins `): its patch, regions and default articulation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instrument {
    /// `MIDILOCALE::ulBank`: MSB in bits 8-14, LSB in 0-6, [`DRUMS`] for a drum kit.
    pub bank: u32,
    pub program: u32,
    pub regions: Vec<Region>,
    pub articulation: Option<Articulation>,
}

impl Instrument {
    /// The DirectMusic patch this instrument answers (`MSB << 16 | LSB << 8 | program`, drum
    /// flag kept).
    pub fn patch(&self) -> u32 {
        let msb = (self.bank >> 8) & 0x7F;
        let lsb = self.bank & 0x7F;
        (self.bank & DRUMS) | (msb << 16) | (lsb << 8) | (self.program & 0x7F)
    }
}

/// A pool wave: its samples and its own `wsmp`.
#[derive(Debug, Clone, PartialEq)]
pub struct DlsWave {
    pub pcm: Arc<Pcm>,
    pub sample: Option<WaveSample>,
}

/// A DLS collection.
#[derive(Debug, Clone, PartialEq)]
pub struct Collection {
    /// `dlid`.
    pub guid: Guid,
    pub instruments: Vec<Instrument>,
    /// The pool, in `ptbl` cue order (a region's `ulTableIndex` indexes it).
    pub waves: Vec<Option<Arc<DlsWave>>>,
}

impl Collection {
    /// The instrument for a DirectMusic patch.
    pub fn instrument(&self, patch: u32) -> Option<&Instrument> {
        self.instruments.iter().find(|i| i.patch() == patch)
    }
}

fn wave_sample(chunk: &Chunk<'_>) -> WaveSample {
    let mut r = chunk.reader();
    let size = r.u32() as usize;
    let unity_note = r.u16();
    let fine_tune = r.i16();
    let attenuation = r.i32();
    let options = r.u32();
    let count = r.u32() as usize;
    r.seek(size);
    let loops = (0..count.min(16))
        .map(|_| {
            let mut l = Reader::new(r.take(16));
            let _size = l.u32();
            SampleLoop {
                kind: l.u32(),
                start: l.u32(),
                length: l.u32(),
            }
        })
        .collect();
    WaveSample {
        unity_note,
        fine_tune,
        attenuation,
        options,
        loops,
    }
}

fn articulation(chunk: &Chunk<'_>) -> Articulation {
    let mut connections = Vec::new();
    for art in chunk
        .children()
        .filter(|c| &c.id == b"art1" || &c.id == b"art2")
    {
        let mut r = art.reader();
        let size = r.u32() as usize;
        let count = r.u32() as usize;
        r.seek(size);
        for _ in 0..count.min(256) {
            connections.push(Connection {
                source: r.u16(),
                control: r.u16(),
                destination: r.u16(),
                transform: r.u16(),
                scale: r.i32(),
            });
        }
    }
    Articulation { connections }
}

fn articulation_of(chunk: &Chunk<'_>) -> Option<Articulation> {
    chunk
        .list(b"lar2")
        .or_else(|| chunk.list(b"lart"))
        .map(|list| articulation(&list))
}

fn region(chunk: &Chunk<'_>) -> Region {
    let mut h = chunk
        .child(b"rgnh")
        .map(|c| c.reader())
        .unwrap_or(Reader::new(&[]));
    let keys = (h.u16(), h.u16());
    let velocities = (h.u16(), h.u16());
    let options = h.u16();
    let key_group = h.u16();
    let link = chunk.child(b"wlnk").map_or_else(WaveLink::default, |c| {
        let mut r = c.reader();
        WaveLink {
            options: r.u16(),
            phase_group: r.u16(),
            channel: r.u32(),
            table_index: r.u32(),
        }
    });
    Region {
        keys,
        velocities,
        options,
        key_group,
        sample: chunk.child(b"wsmp").map(|c| wave_sample(&c)),
        link,
        articulation: articulation_of(chunk),
    }
}

/// A `WAVEFORMATEX` PCM wave's samples as `-1..1` floats.
pub(crate) fn pcm_from(format: &[u8], data: &[u8]) -> MusicResult<Pcm> {
    let mut f = Reader::new(format);
    let tag = f.u16();
    let channels = f.u16();
    let rate = f.u32();
    let _avg = f.u32();
    let _align = f.u16();
    let bits = f.u16();
    if tag != 1 || channels == 0 || rate == 0 || !(bits == 8 || bits == 16) {
        return Err(MusicFormatError("a DLS wave is not 8- or 16-bit PCM"));
    }
    let samples: Arc<[f32]> = if bits == 8 {
        data.iter()
            .map(|&b| (f32::from(b) - 128.0) / 128.0)
            .collect()
    } else {
        data.chunks_exact(2)
            .map(|s| f32::from(i16::from_le_bytes([s[0], s[1]])) / 32768.0)
            .collect()
    };
    Ok(Pcm {
        rate,
        channels,
        samples,
    })
}

/// Reads a DLS collection.
pub fn read_collection(data: &[u8]) -> MusicResult<Collection> {
    let top = riff(data)?;
    if !top.is_list(b"DLS ") {
        return Err(MusicFormatError("not a DLS collection"));
    }
    read_collection_chunk(&top)
}

/// Reads a DLS collection from its `RIFF 'DLS '` chunk.
pub fn read_collection_chunk(top: &Chunk<'_>) -> MusicResult<Collection> {
    let guid = top.child(b"dlid").map_or(Guid::NULL, |c| c.reader().guid());
    let mut instruments = Vec::new();
    if let Some(lins) = top.list(b"lins") {
        for ins in lins.children().filter(|c| c.is_list(b"ins ")) {
            let mut h = ins
                .child(b"insh")
                .map(|c| c.reader())
                .unwrap_or(Reader::new(&[]));
            let _regions = h.u32();
            let bank = h.u32();
            let program = h.u32();
            let regions = ins.list(b"lrgn").map_or_else(Vec::new, |list| {
                list.children()
                    .filter(|c| c.is_list(b"rgn ") || c.is_list(b"rgn2"))
                    .map(|c| region(&c))
                    .collect()
            });
            instruments.push(Instrument {
                bank,
                program,
                regions,
                articulation: articulation_of(&ins),
            });
        }
    }
    // The pool table: each cue an offset from the `wvpl` list's data to a `wave` list.
    let cues: Vec<u32> = top.child(b"ptbl").map_or_else(Vec::new, |c| {
        let mut r = c.reader();
        let size = r.u32() as usize;
        let count = r.u32() as usize;
        r.seek(size);
        (0..count.min(4096)).map(|_| r.u32()).collect()
    });
    let mut waves = Vec::with_capacity(cues.len());
    if let Some(pool) = top.list(b"wvpl") {
        for cue in cues {
            // A cue points at the `LIST` header inside the pool's data.
            let at = cue as usize;
            let wave = pool
                .data
                .get(at..)
                .and_then(|rest| super::riff::Chunks::new(rest).next())
                .filter(|c| c.is_list(b"wave"))
                .and_then(|w| {
                    let format = w.child(b"fmt ")?;
                    let data = w.child(b"data")?;
                    let pcm = pcm_from(format.data, data.data).ok()?;
                    Some(Arc::new(DlsWave {
                        pcm: Arc::new(pcm),
                        sample: w.child(b"wsmp").map(|c| wave_sample(&c)),
                    }))
                });
            waves.push(wave);
        }
    }
    Ok(Collection {
        guid,
        instruments,
        waves,
    })
}
