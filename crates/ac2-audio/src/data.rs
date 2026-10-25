//! The DAT sound records: `SoundInfo` (`0x2A`, a wave and how it plays), `MusicInfo` (`0x2C`, a
//! DirectMusic segment and how it plays) and `DBWave` (`0x0C`, PCM wave data). They are read and
//! decoded on a loader thread ([`DatSounds`]); the frame thread only sees the finished
//! [`LoadedSound`].

use std::sync::Arc;

use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::{ArchiveCursor, DatFileKind};
use ac2_db::{
    ResourceError, ResourcePackedDecode, ResourceResult, ResourceStore, ResourceTypeId,
    TypedResourceRef,
};

/// `SoundInfo::GetDBOType`.
pub const DB_TYPE_SOUND_INFO: u32 = 0x4000_002C;
/// `MusicInfo::GetDBOType`.
pub const DB_TYPE_MUSIC_INFO: u32 = 0x4000_002D;
/// `DBWave::GetDBOType`.
pub const DB_TYPE_WAVE: u32 = 0x4000_0018;

/// `SoundInfo` / `MusicInfo` `m_bitField` bits (`SoundInfo::ParseFlags`).
pub mod flags {
    /// `loop`.
    pub const LOOP: u32 = 0x1;
    /// `exclusive`.
    pub const EXCLUSIVE: u32 = 0x2;
    /// `3D`.
    pub const THREE_D: u32 = 0x4;
    /// `freq`.
    pub const FREQUENCY: u32 = 0x8;
    /// `novirt`.
    pub const NO_VIRTUAL: u32 = 0x10;
    /// `light`.
    pub const LIGHT: u32 = 0x20;
}

/// `MasterDBMap::DivineType` for the sound ranges `SoundManager::PlayInternal` accepts.
pub fn sound_db_type(did: u32) -> Option<u32> {
    match did >> 24 {
        0x2A if did != 0x2AFF_FFFF => Some(DB_TYPE_SOUND_INFO),
        0x2C if did != 0x2CFF_FFFF => Some(DB_TYPE_MUSIC_INFO),
        _ => None,
    }
}

fn malformed(
    qdid: QualifiedDataId,
    type_name: &'static str,
    context: &'static str,
) -> ResourceError {
    ResourceError::MalformedTypedObject {
        qdid,
        type_name,
        context,
    }
}

/// `DBObj::UnPack_Type`: the embedded DID must be the record's.
fn unpack_header(
    cursor: &mut ArchiveCursor<'_>,
    qdid: QualifiedDataId,
    type_name: &'static str,
) -> ResourceResult<()> {
    if cursor.read_u32()? != qdid.data_id().raw() {
        return Err(malformed(
            qdid,
            type_name,
            "embedded DID did not match DAT identity",
        ));
    }
    Ok(())
}

/// `SoundInfo` (`SoundInfo::StreamPack`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SoundInfo {
    pub did: u32,
    /// `m_bitField` ([`flags`]).
    pub bit_field: u32,
    /// `m_randFreqTop` (`-1` when unset).
    pub rand_freq_top: f32,
    /// `m_randFreqBot` (`-1` when unset).
    pub rand_freq_bot: f32,
    /// `m_priority`.
    pub priority: f32,
    /// `m_minDistance`.
    pub min_distance: f32,
    /// `m_objID`: the `DBWave`.
    pub wave: u32,
}

impl SoundInfo {
    /// `MusicInfo::TestBitMask` (retail calls it on both records).
    pub fn test(&self, mask: u32) -> bool {
        self.bit_field & mask != 0
    }
}

impl ResourcePackedDecode for SoundInfo {
    const TYPE_NAME: &'static str = "AC2 SoundInfo";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(qdid: QualifiedDataId, _version: u16, bytes: &[u8]) -> ResourceResult<Self> {
        let mut cursor = ArchiveCursor::new(bytes);
        unpack_header(&mut cursor, qdid, Self::TYPE_NAME)?;
        Ok(Self {
            did: qdid.data_id().raw(),
            bit_field: cursor.read_u32()?,
            rand_freq_top: cursor.read_f32()?,
            rand_freq_bot: cursor.read_f32()?,
            priority: cursor.read_f32()?,
            min_distance: cursor.read_f32()?,
            wave: cursor.read_u32()?,
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}

/// `MusicInfo` (`MusicInfo::StreamPack`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MusicInfo {
    pub did: u32,
    /// `m_bitField`.
    pub bit_field: u32,
    /// `m_priority`.
    pub priority: f32,
    /// `m_minDistance`.
    pub min_distance: f32,
    /// `m_type`.
    pub kind: u32,
    /// `m_objID`: the `DBMusicObj` segment.
    pub segment: u32,
}

impl MusicInfo {
    /// `MusicInfo::TestBitMask`.
    pub fn test(&self, mask: u32) -> bool {
        self.bit_field & mask != 0
    }
}

impl ResourcePackedDecode for MusicInfo {
    const TYPE_NAME: &'static str = "AC2 MusicInfo";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(qdid: QualifiedDataId, _version: u16, bytes: &[u8]) -> ResourceResult<Self> {
        let mut cursor = ArchiveCursor::new(bytes);
        unpack_header(&mut cursor, qdid, Self::TYPE_NAME)?;
        Ok(Self {
            did: qdid.data_id().raw(),
            bit_field: cursor.read_u32()?,
            priority: cursor.read_f32()?,
            min_distance: cursor.read_f32()?,
            kind: cursor.read_u32()?,
            segment: cursor.read_u32()?,
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}

/// Wave samples ready to mix: interleaved, `-1..1`.
#[derive(Debug, Clone, PartialEq)]
pub struct Pcm {
    /// `nSamplesPerSec`.
    pub rate: u32,
    /// `nChannels`.
    pub channels: u16,
    pub samples: Arc<[f32]>,
}

impl Pcm {
    /// Sample frames (one sample per channel).
    pub fn frames(&self) -> usize {
        self.samples.len() / usize::from(self.channels.max(1))
    }
}

/// `DBWave`: a `WAVEFORMATEX` and its data (`DBWave::UnPack` builds a RIFF file from them for
/// DirectMusic; the samples are converted here instead, on the loader thread).
#[derive(Debug, Clone, PartialEq)]
pub struct Wave {
    /// `wFormatTag` (1, PCM, in every Legions wave).
    pub format_tag: u16,
    /// `wBitsPerSample`.
    pub bits: u16,
    /// `m_nDataSize`.
    pub data_size: u32,
    pub pcm: Arc<Pcm>,
}

impl Wave {
    /// `DBWave::GetTime`: the data's length in seconds.
    pub fn time(&self) -> f32 {
        let bytes = u64::from((self.bits >> 3).max(1));
        let channels = u64::from(self.pcm.channels.max(1));
        let frames = u64::from(self.data_size) / bytes / channels;
        (frames as f64 / f64::from(self.pcm.rate.max(1))) as f32
    }
}

impl ResourcePackedDecode for Wave {
    const TYPE_NAME: &'static str = "AC2 DBWave";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    /// `DBWave::UnPack`: the header, `m_nFormatSize` (the `WAVEFORMATEX` with its `cbSize`),
    /// `m_nDataSize`, the format, then the data.
    fn decode_packed(qdid: QualifiedDataId, _version: u16, bytes: &[u8]) -> ResourceResult<Self> {
        let bad = |context| malformed(qdid, Self::TYPE_NAME, context);
        let mut cursor = ArchiveCursor::new(bytes);
        unpack_header(&mut cursor, qdid, Self::TYPE_NAME)?;
        let format_size = cursor.read_u32()? as usize;
        let data_size = cursor.read_u32()?;
        if format_size < 16 {
            return Err(bad("format shorter than PCMWAVEFORMAT"));
        }
        let format = cursor.read_bytes(format_size)?;
        let word = |at: usize| u16::from_le_bytes([format[at], format[at + 1]]);
        let format_tag = word(0);
        let channels = word(2);
        let rate = u32::from_le_bytes([format[4], format[5], format[6], format[7]]);
        let bits = word(14);
        let data = cursor.read_bytes(data_size as usize)?;
        if format_tag != 1 || !(bits == 8 || bits == 16) || channels == 0 || rate == 0 {
            return Err(bad("wave is not 8- or 16-bit PCM"));
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
        Ok(Self {
            format_tag,
            bits,
            data_size,
            pcm: Arc::new(Pcm {
                rate,
                channels,
                samples,
            }),
        })
    }

    fn cache_retained_bytes(&self) -> usize {
        self.pcm.samples.len() * std::mem::size_of::<f32>()
    }
}

/// What a sound DID loads to: retail's `ISoundInstance` source (`DirectXSoundInstance` with its
/// `SoundInfo` and the segment made from its wave, or its `MusicInfo`).
#[derive(Debug, Clone, PartialEq)]
pub enum LoadedSound {
    /// A `SoundInfo` with its wave's samples and `DBWave::GetTime`.
    Sound {
        info: SoundInfo,
        pcm: Arc<Pcm>,
        length: f32,
    },
    /// A `MusicInfo`; its DirectMusic segment is not played yet.
    Music(MusicInfo),
}

/// Where the sound manager's loads come from. Called on the loader thread only.
pub trait SoundSource: Send + Sync {
    /// The sound `did` (a `SoundInfo` or `MusicInfo`), or `None` when the DATs lack it.
    fn load(&self, did: u32) -> Option<LoadedSound>;
}

/// The sounds in the portal DAT.
pub struct DatSounds {
    store: Arc<ResourceStore>,
}

impl DatSounds {
    pub fn new(store: Arc<ResourceStore>) -> Self {
        Self { store }
    }

    fn get<T: ResourcePackedDecode, R>(&self, did: u32, read: impl FnOnce(&T) -> R) -> Option<R> {
        let reference = TypedResourceRef::new(DatFileKind::Portal, DataId::new(did), 0);
        match self.store.load_cached_packed::<T>(reference) {
            Ok(Some(value)) => Some(read(&value)),
            Ok(None) => None,
            Err(error) => {
                log::warn!("sound {did:08X}: {error}");
                None
            }
        }
    }
}

impl SoundSource for DatSounds {
    /// `DBCache::Get` of the `SoundInfo` / `MusicInfo`, then (`DirectXSoundInstance::Initialize`
    /// / `InitDXPSDFromSoundInfo`) its `DBWave`.
    fn load(&self, did: u32) -> Option<LoadedSound> {
        match sound_db_type(did)? {
            DB_TYPE_SOUND_INFO => {
                let info = self.get(did, |info: &SoundInfo| *info)?;
                let (pcm, length) = self.get(info.wave, |wave: &Wave| {
                    (Arc::clone(&wave.pcm), wave.time())
                })?;
                Some(LoadedSound::Sound { info, pcm, length })
            }
            _ => Some(LoadedSound::Music(self.get(did, |m: &MusicInfo| *m)?)),
        }
    }
}
