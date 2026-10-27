//! Placed entities' ethereal bits over their template chain (`EntityDesc::GetTotalProperties`),
//! on the dataset's block 0x21020000; skipped without `AC2_DAT_DIR`.

use super::*;

#[test]
fn placed_entities_merge_their_template_chain_ethereal_bits() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ResourceStore::new(Arc::clone(&dataset));
    assert!(ac2_db::install_ac2_property_types(&store).expect("MasterProperty decodes"));
    let block = 0x2102_0000u32;
    let did = DataId::new((block & 0xFFFF_0000) | 0xFFFE);
    let Some(record) = store
        .load_cached_packed::<Ac2LandBlockEntities>(TypedResourceRef::new(
            DatFileKind::Cell,
            did,
            0,
        ))
        .expect("the landblock's entities decode")
    else {
        return;
    };
    let (mut placed, mut own_bits, mut template_bits, mut false_valued) = (0, 0, 0, 0);
    for entity in &record.entities {
        let Some(data) = entity.data else {
            continue;
        };
        if data.raw() & 0xFF00_0000 != ENTITY_DESC_PREFIX {
            continue;
        }
        placed += 1;
        let own = ac2_db::ac2_ethereal_mask(&entity_own_groups(entity));
        let total = entity_ethereal_mask(&store, entity);
        if own != 0 {
            own_bits += 1;
        }
        if total != own {
            template_bits += 1;
        }
        false_valued += entity_own_groups(entity)
            .iter()
            .flat_map(|group| &group.properties)
            .filter(|p| {
                p.word() == Some(0)
                    && ac2_db::ac2_property_desc(p.name)
                        .is_some_and(|d| d.group == 0x4000_0003 && d.value_type == 0x4000_0001)
            })
            .count();
    }
    eprintln!(
        "block 0x{block:08X}: {placed} placed template entities, {own_bits} with their own \
         ethereal bits, {template_bits} whose merged bits differ from their own, \
         {false_valued} false-valued ethereal bool properties of their own"
    );
    assert!(placed > 0, "the block places template entities");
}
