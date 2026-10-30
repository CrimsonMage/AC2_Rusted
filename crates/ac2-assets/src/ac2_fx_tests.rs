use std::sync::Arc;

use ac2_db::{ResourceStore, TypedResourceRef};

use super::*;

/// Every `DBFXTable` in portal.dat decodes, and the nodes that name a camera effect
/// (`m_cameraFX_id`) name `CameraFX` records (DIDs `0x55xxxxxx`).
#[test]
fn every_fx_table_decodes_and_its_camera_effects_are_camera_fx_records() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ResourceStore::new(Arc::clone(&dataset));
    assert!(ac2_db::install_ac2_property_types(&store).expect("MasterProperty decodes"));
    let ids: Vec<u32> = dataset
        .ids_in_range(ac2_dat::DatFileKind::Portal, 0x1E00_0000, 0x1EFF_FFFE)
        .collect();
    assert!(!ids.is_empty());
    let mut camera = Vec::new();
    for did in ids {
        let table = store
            .load_cached_packed::<Ac2FxTable>(TypedResourceRef::new(
                ac2_dat::DatFileKind::Portal,
                DataId::new(did),
                0,
            ))
            .unwrap_or_else(|e| panic!("{did:08X}: {e:?}"))
            .unwrap_or_else(|| panic!("{did:08X} missing"));
        for (fx, nodes) in table.entries.iter() {
            for node in nodes.iter().filter(|n| n.object.camera_fx != 0) {
                camera.push((did, *fx, node.object.camera_fx));
            }
        }
    }
    eprintln!("{} camera FX nodes: {camera:08X?}", camera.len());
    assert!(!camera.is_empty(), "some FX play a camera effect");
    for (table, fx, id) in camera {
        assert_eq!(id >> 24, 0x55, "{table:08X} fx {fx:08X}");
    }
}

/// Which FX nodes of portal.dat carry a slider effect or a repulsor (`m_slider`, `m_has_repulsor`).
#[test]
fn slider_and_repulsor_nodes_in_the_fx_tables() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ResourceStore::new(Arc::clone(&dataset));
    assert!(ac2_db::install_ac2_property_types(&store).expect("MasterProperty decodes"));
    let (mut sliders, mut repulsors) = (Vec::new(), Vec::new());
    let ids: Vec<u32> = dataset
        .ids_in_range(ac2_dat::DatFileKind::Portal, 0x1E00_0000, 0x1EFF_FFFE)
        .collect();
    for did in ids {
        let table = store
            .load_cached_packed::<Ac2FxTable>(TypedResourceRef::new(
                ac2_dat::DatFileKind::Portal,
                DataId::new(did),
                0,
            ))
            .unwrap()
            .unwrap();
        for (fx, nodes) in table.entries.iter() {
            for node in nodes.iter() {
                if node.object.slider.id != 0 {
                    sliders.push((did, *fx, node.object.slider));
                }
                if let Some(r) = node.object.repulsor {
                    repulsors.push((did, *fx, r));
                }
            }
        }
    }
    eprintln!(
        "sliders {}: {:08X?}",
        sliders.len(),
        &sliders[..sliders.len().min(8)]
    );
    eprintln!(
        "repulsors {}: {:?}",
        repulsors.len(),
        &repulsors[..repulsors.len().min(4)]
    );
}

/// Which FX of portal.dat key terrain nodes (`FXData::m_terrainData`) and channel keys
/// (`FXNode::m_channel_keys`).
#[test]
fn terrain_and_channel_keyed_nodes_in_the_fx_tables() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ResourceStore::new(Arc::clone(&dataset));
    assert!(ac2_db::install_ac2_property_types(&store).expect("MasterProperty decodes"));
    let ids: Vec<u32> = dataset
        .ids_in_range(ac2_dat::DatFileKind::Portal, 0x1E00_0000, 0x1EFF_FFFE)
        .collect();
    let mut terrains = std::collections::BTreeMap::<u32, usize>::new();
    let (mut channel_nodes, mut keyed) = (0, 0);
    for did in ids {
        let table = store
            .load_cached_packed::<Ac2FxTable>(TypedResourceRef::new(
                ac2_dat::DatFileKind::Portal,
                DataId::new(did),
                0,
            ))
            .unwrap()
            .unwrap();
        for (_, sets) in table.terrains.iter() {
            for set in sets.iter() {
                *terrains.entry(set.terrain).or_default() += 1;
            }
        }
        for (_, nodes) in table.entries.iter() {
            for node in nodes.iter() {
                channel_nodes += usize::from(node.object.gate.has_channels);
                keyed += usize::from(node.object.gate.channels.iter().any(Option::is_some));
            }
        }
    }
    eprintln!("terrain sets by terrain {terrains:?}; {channel_nodes} channel nodes, {keyed} on the preference keys");
    assert!(!terrains.is_empty(), "some FX key terrain nodes");
    assert!(keyed > 0, "some FX nodes key the gore or blood channels");
}

/// `FXManager::ChannelMissmatch`: only a key both the node and the table hold, with different
/// values, mismatches; a node without keys, or a table without values, never does.
#[test]
fn channel_missmatch_compares_only_the_keys_both_sides_hold() {
    let foot_particles = CHANNEL_KEYS[3];
    let gore = CHANNEL_KEYS[0];
    let mut gate = Ac2FxNodeGate {
        has_channels: true,
        ..Ac2FxNodeGate::default()
    };
    gate.channels[3] = Some(1);
    let mut prefs = Ac2FxChannelPrefs::default();
    assert!(
        !gate.channel_missmatch(&prefs),
        "an empty table never mismatches"
    );
    prefs.set(foot_particles, 1);
    assert!(!gate.channel_missmatch(&prefs), "the same value plays");
    prefs.set(foot_particles, 0);
    assert!(
        gate.channel_missmatch(&prefs),
        "foot step particles off stops a node that wants them on"
    );
    prefs.set(gore, 3);
    prefs.set(foot_particles, 1);
    assert!(
        !gate.channel_missmatch(&prefs),
        "a key the node lacks is not compared"
    );
    prefs.set(0x4000_0009, 9);
    assert_eq!(prefs.values[2], None, "only the four channel keys are kept");
    let plain = Ac2FxNodeGate::default();
    prefs.set(foot_particles, 0);
    assert!(
        !plain.channel_missmatch(&prefs),
        "a node with no channel keys plays"
    );
}

/// `FXData::GetData(terrain)` of each `FXData` in an id's list: the terrain's node where the data
/// keys one, else its default node, so a data without the key keeps its default.
#[test]
fn terrain_sets_take_the_keyed_node_or_the_default_per_data() {
    let node = |tag: u32| Ac2FxParticleNode {
        particle_descs: Arc::from([DataId::new(tag)]),
        fire_and_forget: false,
        light: None,
        holding_location: 0,
        bone_name: 0,
        bone_offset: Vec3::ZERO,
        remove_rotation: false,
        water_plane: false,
        object: Ac2FxObject::default(),
    };
    let sound = |tag: u32| Ac2FxSoundNode {
        sounds: Arc::from([tag]),
        ramp_time: 0.0,
        fire_and_forget: false,
        player_only: false,
        day_time: true,
        night_time: true,
        has_channels: false,
        channels: [None; 4],
    };
    let first = FxDataParts {
        default: (Some(node(1)), sound(1)),
        terrains: vec![(7, (Some(node(2)), sound(2)))],
    };
    // The second data's default is empty, and it keys terrain 8 only.
    let second = FxDataParts {
        default: (None, sound(3)),
        terrains: vec![(8, (Some(node(4)), sound(4)))],
    };
    let (particles, sounds, terrains) = FxDataParts::segment(vec![first, second]);
    assert_eq!(particles.len(), 1);
    assert_eq!(sounds.len(), 2);
    let on = |terrain: u32| {
        terrains
            .iter()
            .find(|set| set.terrain == terrain)
            .expect("keyed")
    };
    let ids = |set: &Ac2FxTerrainSet| -> Vec<u32> {
        set.particles
            .iter()
            .map(|n| n.particle_descs[0].raw())
            .collect()
    };
    let sound_ids =
        |set: &Ac2FxTerrainSet| -> Vec<u32> { set.sounds.iter().map(|n| n.sounds[0]).collect() };
    assert_eq!(ids(on(7)), vec![2]);
    assert_eq!(sound_ids(on(7)), vec![2, 3]);
    assert_eq!(ids(on(8)), vec![1, 4]);
    assert_eq!(sound_ids(on(8)), vec![1, 4]);
}

/// The FX lights keep their range and diffuse intensity as the waveforms the DAT stores
/// (`LightSource::m_wRange`, `m_wDiffuseIntensity`): the waveform at time zero is the light's base
/// value when it does not move, and some lights animate.
#[test]
fn fx_lights_keep_their_animated_range_and_intensity_waveforms() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ResourceStore::new(Arc::clone(&dataset));
    assert!(ac2_db::install_ac2_property_types(&store).expect("MasterProperty decodes"));
    let ids: Vec<u32> = dataset
        .ids_in_range(ac2_dat::DatFileKind::Portal, 0x1E00_0000, 0x1EFF_FFFE)
        .collect();
    let (mut lights, mut range_waves, mut intensity_waves) = (0, 0, 0);
    for did in ids {
        let table = store
            .load_cached_packed::<Ac2FxTable>(TypedResourceRef::new(
                ac2_dat::DatFileKind::Portal,
                DataId::new(did),
                0,
            ))
            .unwrap()
            .unwrap();
        for light in table
            .entries
            .iter()
            .flat_map(|(_, nodes)| nodes.iter())
            .filter_map(|node| node.light.as_ref())
        {
            lights += 1;
            let source = &light.source;
            range_waves += usize::from(source.range_wave.kind != ac2_math::waveform::kind::NONE);
            intensity_waves +=
                usize::from(source.intensity_wave.kind != ac2_math::waveform::kind::NONE);
            // The base value both ways: the stored word and the waveform's own base.
            assert_eq!(source.range_wave.base, source.range, "{did:08X}");
            assert_eq!(source.intensity_wave.base, source.intensity, "{did:08X}");
        }
    }
    eprintln!("{lights} FX lights, {range_waves} with a range waveform, {intensity_waves} with an intensity waveform");
    assert!(lights > 0);
}
