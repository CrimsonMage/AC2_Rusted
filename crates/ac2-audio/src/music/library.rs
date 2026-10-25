//! The music library: the `DBMusicObj` records by GUID (`IDirectMusicLoader` resolving
//! references), and loading what a `MusicInfo` plays (`DirectXMusicLoader::LoadFromMusicInfo`):
//! a DirectMusic segment with every object it reaches, or an `EncodedWave` as a wave.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, OnceLock};

use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::DatFileKind;
use ac2_db::{
    ResourceError, ResourcePackedDecode, ResourceResult, ResourceStore, ResourceTypeId,
    TypedResourceRef,
};

use super::band::{Band, Reference};
use super::riff::{riff, Guid, Reader};
use super::segment::{Segment, TrackData};
use super::style::Style;
use super::{read_music_record, MusicObject};
use crate::data::{MusicInfo, Pcm};

const MUSIC_FIRST: u32 = 0x2400_0000;
const MUSIC_LAST: u32 = 0x24FF_FFFE;
const ENCODED_WAVE: u32 = 0x61;

/// A decoded `DBMusicObj`.
#[derive(Debug, Clone, PartialEq)]
pub struct MusicRecord(pub MusicObject);

impl ResourcePackedDecode for MusicRecord {
    const TYPE_NAME: &'static str = "AC2 DBMusicObj";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(qdid: QualifiedDataId, _version: u16, bytes: &[u8]) -> ResourceResult<Self> {
        read_music_record(qdid.data_id().raw(), bytes)
            .map(MusicRecord)
            .map_err(|e| ResourceError::MalformedTypedObject {
                qdid,
                type_name: Self::TYPE_NAME,
                context: e.0,
            })
    }

    fn cache_retained_bytes(&self) -> usize {
        match &self.0 {
            MusicObject::Collection(c) => c
                .waves
                .iter()
                .flatten()
                .map(|w| w.pcm.samples.len() * 4)
                .sum(),
            _ => 4096,
        }
    }
}

/// A `DBMusicObj`'s GUID only, read without decoding the rest (the library's index).
#[derive(Debug, Clone, Copy, PartialEq)]
struct MusicGuid(Option<Guid>);

impl ResourcePackedDecode for MusicGuid {
    const TYPE_NAME: &'static str = "AC2 DBMusicObj GUID";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    fn decode_packed(_qdid: QualifiedDataId, _version: u16, bytes: &[u8]) -> ResourceResult<Self> {
        let guid = bytes
            .get(8..)
            .and_then(|file| riff(file).ok())
            .and_then(|top| {
                top.child(b"guid")
                    .or_else(|| top.child(b"dlid"))
                    .map(|c| c.reader().guid())
            });
        Ok(MusicGuid(guid))
    }

    fn cache_retained_bytes(&self) -> usize {
        24
    }
}

/// `EncodedWave` (`0x61`, in the local DAT): a compressed WAVE file.
#[derive(Debug, Clone, PartialEq)]
struct EncodedWaveRecord(Arc<Pcm>);

impl ResourcePackedDecode for EncodedWaveRecord {
    const TYPE_NAME: &'static str = "AC2 EncodedWave";
    const RESOURCE_TYPE: ResourceTypeId = ResourceTypeId::Unknown;

    /// `EncodedWave::UnPack`: the DID, the file's length, the file (decoded here).
    fn decode_packed(qdid: QualifiedDataId, _version: u16, bytes: &[u8]) -> ResourceResult<Self> {
        let bad = |context| ResourceError::MalformedTypedObject {
            qdid,
            type_name: Self::TYPE_NAME,
            context,
        };
        let mut r = Reader::new(bytes);
        if r.u32() != qdid.data_id().raw() {
            return Err(bad("embedded DID did not match DAT identity"));
        }
        let length = r.u32() as usize;
        let file = bytes
            .get(8..8 + length)
            .ok_or(bad("wave longer than the record"))?;
        super::encoded::decode(file)
            .map(|pcm| EncodedWaveRecord(Arc::new(pcm)))
            .map_err(|e| bad(e.0))
    }

    fn cache_retained_bytes(&self) -> usize {
        self.0.samples.len() * 4
    }
}

/// A segment with every object it reaches, by GUID.
#[derive(Debug, Clone, PartialEq)]
pub struct LoadedMusic {
    pub segment: MusicObject,
    pub objects: HashMap<Guid, MusicObject>,
    /// References no record answers.
    pub missing: usize,
}

impl LoadedMusic {
    /// What a reference names (its GUID, else nothing).
    pub fn resolve(&self, reference: &Reference) -> Option<&MusicObject> {
        self.objects.get(reference.guid.as_ref()?)
    }

    /// The style `segment` references first (its styles track), when the closure holds it.
    pub fn style_of(&self, segment: &Segment) -> Option<Arc<Style>> {
        let reference = segment.tracks.iter().find_map(|track| match &track.data {
            TrackData::Styles(styles) => styles.first().map(|(_, reference)| reference),
            _ => None,
        })?;
        match self.resolve(reference)? {
            MusicObject::Style(style) => Some(Arc::clone(style)),
            _ => None,
        }
    }
}

/// What a `MusicInfo` plays.
#[derive(Debug, Clone, PartialEq)]
pub enum MusicSource {
    DirectMusic(Arc<LoadedMusic>),
    Wave(Arc<Pcm>),
}

/// The `DBMusicObj` records of the portal DAT.
pub struct MusicLibrary {
    store: Arc<ResourceStore>,
    index: OnceLock<HashMap<Guid, u32>>,
}

impl MusicLibrary {
    pub fn new(store: Arc<ResourceStore>) -> Self {
        Self {
            store,
            index: OnceLock::new(),
        }
    }

    fn reference(kind: DatFileKind, did: u32) -> TypedResourceRef {
        TypedResourceRef::new(kind, DataId::new(did), 0)
    }

    /// The record `did`.
    pub fn record(&self, did: u32) -> Option<MusicObject> {
        let record = self
            .store
            .load_cached_packed::<MusicRecord>(Self::reference(DatFileKind::Portal, did))
            .ok()??;
        Some(record.0.clone())
    }

    /// Every record's GUID (`IDirectMusicLoader::ScanDirectory`), built once.
    fn index(&self) -> &HashMap<Guid, u32> {
        self.index.get_or_init(|| {
            let ids: Vec<u32> = self
                .store
                .dataset()
                .ids_in_range(DatFileKind::Portal, MUSIC_FIRST, MUSIC_LAST)
                .collect();
            ids.into_iter()
                .filter_map(|did| {
                    let guid = self
                        .store
                        .load_cached_packed::<MusicGuid>(Self::reference(DatFileKind::Portal, did))
                        .ok()??;
                    Some((guid.0?, did))
                })
                .collect()
        })
    }

    /// The object a GUID names.
    pub fn by_guid(&self, guid: &Guid) -> Option<MusicObject> {
        self.record(*self.index().get(guid)?)
    }

    /// `DirectXMusicLoader::LoadFromMusicInfo`: a `DBMusicObj` segment with its closure, or an
    /// `EncodedWave` decoded to samples.
    pub fn load_music(&self, info: &MusicInfo) -> Option<MusicSource> {
        if info.segment >> 24 == ENCODED_WAVE {
            let wave = self
                .store
                .load_cached_packed::<EncodedWaveRecord>(Self::reference(
                    DatFileKind::Local,
                    info.segment,
                ))
                .ok()??;
            return Some(MusicSource::Wave(Arc::clone(&wave.0)));
        }
        let segment = self.record(info.segment)?;
        Some(MusicSource::DirectMusic(Arc::new(self.closure(segment))))
    }

    /// Everything `root` reaches through references and embedded objects.
    pub fn closure(&self, root: MusicObject) -> LoadedMusic {
        let mut objects = HashMap::new();
        let mut missing = 0;
        let mut seen = HashSet::new();
        let mut todo: Vec<MusicObject> = vec![root.clone()];
        while let Some(object) = todo.pop() {
            if let Some(guid) = object.guid() {
                if !seen.insert(guid) {
                    continue;
                }
                objects.insert(guid, object.clone());
            }
            for item in children(&object) {
                match item {
                    Child::Object(o) => todo.push(o),
                    Child::Reference(r) => match r.guid.and_then(|g| {
                        if seen.contains(&g) {
                            None
                        } else {
                            Some(self.by_guid(&g))
                        }
                    }) {
                        Some(Some(o)) => todo.push(o),
                        Some(None) => missing += 1,
                        None => {}
                    },
                }
            }
        }
        LoadedMusic {
            segment: root,
            objects,
            missing,
        }
    }
}

enum Child {
    Object(MusicObject),
    Reference(Reference),
}

fn band_children(band: &Band, out: &mut Vec<Child>) {
    for instrument in &band.instruments {
        if let Some(r) = &instrument.collection {
            out.push(Child::Reference(r.clone()));
        }
    }
}

/// The objects and references an object holds.
fn children(object: &MusicObject) -> Vec<Child> {
    let mut out = Vec::new();
    match object {
        MusicObject::Segment(segment) => {
            for track in &segment.tracks {
                match &track.data {
                    TrackData::Styles(styles) => {
                        out.extend(styles.iter().map(|(_, r)| Child::Reference(r.clone())))
                    }
                    TrackData::Bands(bands) => {
                        for (_, band) in bands {
                            band_children(band, &mut out);
                        }
                    }
                    TrackData::Triggers(triggers) => out.extend(
                        triggers
                            .iter()
                            .filter_map(|t| t.segment.clone())
                            .map(Child::Reference),
                    ),
                    TrackData::Scripts(calls) => out.extend(
                        calls
                            .iter()
                            .filter_map(|c| c.script.clone())
                            .map(Child::Reference),
                    ),
                    _ => {}
                }
            }
        }
        MusicObject::Style(style) => {
            for band in style
                .bands
                .iter()
                .chain(style.patterns.iter().filter_map(|p| p.band.as_ref()))
            {
                band_children(band, &mut out);
            }
        }
        MusicObject::Band(band) => band_children(band, &mut out),
        MusicObject::Script(script) => {
            if let Some(container) = &script.container {
                container_children(container, &mut out);
            }
        }
        MusicObject::Container(container) => container_children(container, &mut out),
        MusicObject::Collection(_) | MusicObject::Other(_) | MusicObject::Wave(_) => {}
    }
    out
}

fn container_children(container: &super::container::Container, out: &mut Vec<Child>) {
    for item in &container.objects {
        if let Some(o) = &item.object {
            out.push(Child::Object(o.clone()));
        }
        if let Some(r) = &item.reference {
            out.push(Child::Reference(r.clone()));
        }
    }
}
