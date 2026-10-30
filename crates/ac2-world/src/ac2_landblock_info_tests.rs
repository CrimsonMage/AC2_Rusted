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

/// `CPhysicsObj::makeObjectInternal`: a template whose `0x4000006B` property is false makes no
/// object, so the placed entities that instance it resolve to no visual; the rest still do.
#[test]
fn a_template_with_a_false_no_object_property_places_nothing() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ResourceStore::new(Arc::clone(&dataset));
    assert!(ac2_db::install_ac2_property_types(&store).expect("MasterProperty decodes"));
    let mut seen = (0, 0);
    for block in [0x2102_0000u32, 0x2002_0000, 0x2202_0000] {
        let did = DataId::new((block & 0xFFFF_0000) | 0xFFFE);
        let Some(record) = store
            .load_cached_packed::<Ac2LandBlockEntities>(TypedResourceRef::new(
                DatFileKind::Cell,
                did,
                0,
            ))
            .expect("the landblock's entities decode")
        else {
            continue;
        };
        for entity in &record.entities {
            let Some(data) = entity
                .data
                .filter(|d| d.raw() & 0xFF00_0000 == ENTITY_DESC_PREFIX)
            else {
                continue;
            };
            let Some(template) = store
                .load_cached_packed::<Ac2EntityDesc>(TypedResourceRef::new(
                    DatFileKind::Portal,
                    data,
                    0,
                ))
                .expect("the template decodes")
            else {
                continue;
            };
            let resolved = resolve_entity_visual(&store, entity).is_some();
            if template_makes_no_object(&template) {
                assert!(!resolved, "{data:?} makes no object");
                seen.0 += 1;
            } else {
                seen.1 += 1;
            }
        }
    }
    eprintln!("templates making no object: {}, others: {}", seen.0, seen.1);
    assert!(seen.1 > 0, "the blocks place templates");
}
