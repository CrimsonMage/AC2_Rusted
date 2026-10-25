//! The music records against real data (`AC2_DAT_DIR`): invariants, no stored outputs.

use std::collections::HashMap;
use std::sync::Arc;

use ac2_core::DataId;
use ac2_dat::DatFileKind;
use ac2_db::{ResourceStore, TypedResourceRef};

use super::library::MusicRecord;
use super::segment::TrackData;
use super::{MusicLibrary, MusicObject, MusicSource};
use crate::data::MusicInfo;

fn store() -> Option<Arc<ResourceStore>> {
    let dataset = ac2_dat::test_support::env_dataset()?;
    Some(Arc::new(ResourceStore::new(dataset)))
}

#[test]
fn every_music_record_reads() {
    let Some(store) = store() else {
        return;
    };
    let ids: Vec<u32> = store
        .dataset()
        .ids_in_range(DatFileKind::Portal, 0x2400_0000, 0x24FF_FFFE)
        .collect();
    assert_eq!(ids.len(), 1111);
    let mut kinds: HashMap<&str, usize> = HashMap::new();
    let mut notes = 0usize;
    for did in ids {
        let record = store
            .load_cached_packed::<MusicRecord>(TypedResourceRef::new(
                DatFileKind::Portal,
                DataId::new(did),
                0,
            ))
            .unwrap_or_else(|e| panic!("{did:08X}: {e}"))
            .expect("listed record");
        let kind = match &record.0 {
            MusicObject::Segment(s) => {
                assert!(s.length >= 0, "{did:08X}");
                for track in &s.tracks {
                    if let TrackData::Tempo(t) = &track.data {
                        assert!(t.iter().all(|(_, bpm)| *bpm > 0.0 && *bpm < 1000.0));
                    }
                }
                "segment"
            }
            MusicObject::Style(s) => {
                notes += s.parts.iter().map(|p| p.notes.len()).sum::<usize>();
                assert!(!s.patterns.is_empty(), "{did:08X} has patterns");
                "style"
            }
            MusicObject::Band(_) => "band",
            MusicObject::Collection(c) => {
                assert!(!c.instruments.is_empty());
                for i in &c.instruments {
                    for r in &i.regions {
                        let index = r.link.table_index as usize;
                        assert!(
                            c.waves.get(index).is_some_and(|w| w.is_some()),
                            "{did:08X}: region wave {index}"
                        );
                    }
                }
                "dls"
            }
            MusicObject::Script(s) => {
                assert_eq!(s.language, "AudioVBScript");
                assert!(s.container.is_some());
                "script"
            }
            MusicObject::Container(_) => "container",
            MusicObject::Other(_) => "other",
        };
        *kinds.entry(kind).or_default() += 1;
    }
    eprintln!("{kinds:?}, {notes} style notes");
    assert_eq!(kinds["segment"], 630);
    assert_eq!(kinds["style"], 396);
    assert_eq!(kinds["dls"], 61);
    assert_eq!(kinds["script"], 14);
}

#[test]
fn every_music_info_loads_its_music() {
    let Some(store) = store() else {
        return;
    };
    let library = MusicLibrary::new(Arc::clone(&store));
    let ids: Vec<u32> = store
        .dataset()
        .ids_in_range(DatFileKind::Portal, 0x2C00_0000, 0x2CFF_FFFE)
        .collect();
    let (mut segments, mut waves, mut missing) = (0, 0, 0);
    for did in ids {
        let info = store
            .load_cached_packed::<MusicInfo>(TypedResourceRef::new(
                DatFileKind::Portal,
                DataId::new(did),
                0,
            ))
            .unwrap()
            .unwrap();
        if info.segment >> 24 == 0x61 {
            // Decoding every MP3 is slow; one is enough here.
            if waves > 0 {
                continue;
            }
        }
        match library.load_music(&info) {
            Some(MusicSource::DirectMusic(music)) => {
                segments += 1;
                missing += music.missing;
                assert!(matches!(music.segment, MusicObject::Segment(_)));
            }
            Some(MusicSource::Wave(pcm)) => {
                waves += 1;
                assert!(pcm.rate >= 8000 && pcm.frames() > 44_100, "{did:08X}");
            }
            None => panic!("{did:08X}: no music"),
        }
    }
    eprintln!("{segments} segments ({missing} unresolved references), {waves} waves");
    assert_eq!(segments, 530);
    assert_eq!(waves, 1);
}
