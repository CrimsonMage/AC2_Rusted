//! The `MasterProperty` name table of the local DAT (`AC2_DAT_DIR`): every name resolves both
//! ways, and the names are the ones the client shows.

use ac2_db::{
    ac2_property_name, ac2_property_name_enum, install_ac2_property_types, ResourceStore,
};

#[test]
fn every_property_name_resolves_both_ways() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ResourceStore::new(dataset);
    assert!(install_ac2_property_types(&store).expect("MasterProperty decodes"));
    let names = ac2_db::ac2_property_names().expect("installed table");
    assert!(names.len() > 100, "{} names", names.len());
    for (id, name) in &names {
        assert_eq!(ac2_property_name(*id), Some(name.as_slice()));
        assert_eq!(ac2_property_name_enum(name), Some(*id));
    }
    for (id, name) in names.iter().take(8) {
        eprintln!("property {id:08x} = {}", String::from_utf8_lossy(name));
    }
}
