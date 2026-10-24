//! Real-data checks of AC2 cell surfaces and dungeon appearance atoms. They run only when
//! `AC2_DAT_DIR` points at an AC2 DAT directory.

use std::collections::BTreeSet;
use std::sync::Arc;

use ac2_dat::{DatDataset, DatFileKind, DatOpenOptions};
use ac2_core::DataId;
use ac2_db::ResourceStore;

use crate::{
    ac2_appearance_material_atoms, ac2_dungeon_appearance_table, decode_ac2_cell_surfaces,
    load_ac2_texture_asset, TextureAsset, TexturePayload,
};

fn ac2_dataset() -> Option<(Arc<DatDataset>, ResourceStore)> {
    let root = std::env::var_os("AC2_DAT_DIR")?;
    let dataset = DatDataset::open(DatOpenOptions::new(root)).expect("AC2 DATs mount");
    let store = ResourceStore::new(Arc::clone(&dataset));
    assert!(ac2_db::install_ac2_property_types(&store).expect("MasterProperty decodes"));
    Some((dataset, store))
}

fn level_sizes(asset: &TextureAsset) -> Vec<(u32, u32)> {
    match asset.payload() {
        TexturePayload::Rgba(levels) => levels.levels.iter().map(|l| (l.width, l.height)).collect(),
        TexturePayload::BlockCompressed(levels) => {
            levels.levels.iter().map(|l| (l.width, l.height)).collect()
        }
    }
}

#[test]
fn every_ac2_cell_mesh_draws_mipmapped_merged_surfaces() {
    let Some((dataset, store)) = ac2_dataset() else {
        return;
    };
    let meshes = dataset
        .ids_in_range(DatFileKind::Portal, 0x0100_0000, 0x01FF_FFFF)
        .collect::<Vec<_>>();
    let (mut surfaces, mut merged, mut doubled) = (0, 0, 0);
    let mut textures = BTreeSet::new();
    for did in &meshes {
        let cell = decode_ac2_cell_surfaces(&store, DataId::new(*did), &[])
            .unwrap_or_else(|error| panic!("cell mesh {did:08X}: {error}"))
            .expect("listed record exists");
        for surface in &cell.surfaces {
            surfaces += 1;
            merged += usize::from(surface.detail_texture.is_some());
            assert!(
                [1.0, 2.0].contains(&surface.diffuse_scale),
                "{did:08X} diffuse modulation"
            );
            doubled += usize::from(surface.diffuse_scale == 2.0);
            textures.insert(surface.base_texture);
            textures.extend(surface.detail_texture);
            if surface.detail_texture.is_some() {
                assert!(surface.detail_uv_scale.iter().all(|scale| *scale > 0.0));
            }
        }
    }
    assert!(surfaces > 1_000, "{surfaces} cell surfaces");
    eprintln!("{surfaces} cell surfaces, {merged} merge a detail texture, {doubled} MODULATE2X");
    assert!(
        merged * 2 > surfaces,
        "{merged} of {surfaces} cell surfaces merge a detail texture"
    );
    assert!(
        doubled * 2 > surfaces,
        "{doubled} of {surfaces} cell surfaces MODULATE2X the diffuse"
    );
    for chain in textures {
        let asset = load_ac2_texture_asset(&store, chain)
            .expect("texture chain decodes")
            .expect("texture chain exists");
        let sizes = level_sizes(&asset);
        assert!(
            sizes.len() > 1,
            "{chain:?} uploads a full mip chain, not one level"
        );
        for pair in sizes.windows(2) {
            assert_eq!(pair[1], ((pair[0].0 / 2).max(1), (pair[0].1 / 2).max(1)));
        }
    }
}

#[test]
fn dungeon_appearance_atoms_retexture_cell_materials() {
    let Some((_, store)) = ac2_dataset() else {
        return;
    };
    let table = ac2_dungeon_appearance_table(&store)
        .expect("enum maps decode")
        .expect("dungeon appearance table is mapped");
    assert_eq!(table, DataId::new(0x2000_0053));
    let atoms =
        ac2_appearance_material_atoms(&store, table, 0x4100_000F, 0.0).expect("table decodes");
    assert_eq!(atoms.len(), 19);
    assert!(atoms.iter().all(|atom| atom.modifier.is_some()));
    // Cell mesh 0x0100003F's materials (types 0x81000030 and kin) take the keyed modifiers.
    let plain = decode_ac2_cell_surfaces(&store, DataId::new(0x0100_003F), &[])
        .unwrap()
        .unwrap();
    let themed = decode_ac2_cell_surfaces(&store, DataId::new(0x0100_003F), &atoms)
        .unwrap()
        .unwrap();
    assert_eq!(plain.surfaces.len(), themed.surfaces.len());
    assert!(plain
        .surfaces
        .iter()
        .zip(&themed.surfaces)
        .any(|(plain, themed)| {
            (plain.base_texture, plain.detail_texture)
                != (themed.base_texture, themed.detail_texture)
        }));
}

/// Every texture chain the AC2 paths upload keeps its authored mip chain with the high-resolution
/// DAT mounted, as the viewer mounts it.
#[test]
fn every_ac2_texture_chain_keeps_its_mips_with_highres_mounted() {
    let Some(root) = std::env::var_os("AC2_DAT_DIR") else {
        return;
    };
    let mut options = DatOpenOptions::new(root);
    options.high_res_enabled = options.path_for(DatFileKind::HighRes).is_file();
    let dataset = DatDataset::open(options).expect("AC2 DATs mount");
    let store = ResourceStore::new(Arc::clone(&dataset));
    let chains = dataset
        .ids_in_range(DatFileKind::Portal, 0x4000_0000, 0x400F_FFFF)
        .collect::<Vec<_>>();
    let (mut single, mut checked) = (Vec::new(), 0);
    for did in &chains {
        let Some(asset) = load_ac2_texture_asset(&store, DataId::new(*did)).expect("chain decodes")
        else {
            continue;
        };
        checked += 1;
        let sizes = level_sizes(&asset);
        if sizes.len() == 1 && sizes[0].0.max(sizes[0].1) > 1 {
            single.push((*did, sizes[0]));
        }
    }
    eprintln!(
        "{checked} chains, {} single-level: {:08X?}",
        single.len(),
        &single[..single.len().min(12)]
    );
    assert!(
        single.is_empty(),
        "{} AC2 textures upload one level",
        single.len()
    );
}
