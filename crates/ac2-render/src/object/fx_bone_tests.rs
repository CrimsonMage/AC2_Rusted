//! Real-data check of the FX bone attachments: a bone node is resolved when its name is among the
//! posed part groups of its visual (`CPartGroup::FindBoneType` searches the child groups too,
//! root first). Runs only when `AC2_DAT_DIR` points at an AC2 DAT directory.
//!
//! The check is per visual, so a bone that lives on a wearer's Setup (equipment visuals with one
//! group) cannot resolve here; the printed share includes those.

use std::sync::Arc;

use ac2_assets::{resolve_ac2_visual_fx_particles, rest_named_bones};
use ac2_core::DataId;
use ac2_dat::DatFileKind;
use ac2_db::ResourceStore;

use super::{load_object_model, RenderSettings};

#[test]
fn fx_bone_nodes_find_their_bone_in_the_dataset_part_groups() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ResourceStore::new(Arc::clone(&dataset));
    assert!(ac2_db::install_ac2_property_types(&store).expect("MasterProperty decodes"));
    let settings = RenderSettings::default();
    let (mut bones, mut in_root, mut found) = (0usize, 0usize, 0usize);
    // `ac2_assets::rest_named_bones` must list the same bones, in the same order and at the same
    // place, as the posed skeleton's `named_bones` at rest.
    let (mut compared, mut mismatched) = (0usize, 0usize);
    for did in dataset.ids_in_range(DatFileKind::Portal, 0x1F00_0000, 0x1F00_0000 | 0x00FF_FFFF) {
        let visual = DataId::new(did);
        let Some(table) = resolve_ac2_visual_fx_particles(&store, visual)
            .unwrap_or_else(|error| panic!("VisualDesc {did:08X}: {error}"))
        else {
            continue;
        };
        let names = table.bone_names();
        if names.is_empty() {
            continue;
        }
        bones += names.len();
        let posed = load_object_model(&store, visual, &settings)
            .unwrap_or_else(|error| panic!("VisualDesc {did:08X} model: {error}"))
            .and_then(|model| model.pose(None));
        let Some(posed) = posed else {
            continue;
        };
        let assets = rest_named_bones(&store, visual)
            .unwrap_or_else(|error| panic!("VisualDesc {did:08X} named bones: {error}"));
        compared += 1;
        let same_names = assets.len() == posed.named_bones.len()
            && assets
                .iter()
                .zip(&posed.named_bones)
                .all(|((a, ta), (b, tb))| a == b && (ta.offset - tb.offset).length() < 1e-3);
        mismatched += usize::from(!same_names);
        // The root group's bones come first in the search order.
        let root_len = posed.bones.len().min(posed.named_bones.len());
        for name in &names {
            if posed.named_bones[..root_len].iter().any(|(n, _)| n == name) {
                in_root += 1;
            }
            if posed.named_bones.iter().any(|(n, _)| n == name) {
                found += 1;
            }
        }
    }
    eprintln!(
        "FX bone nodes: {bones}; found by name in the part groups {found} ({:.2}%); found in the \
         root Setup alone {in_root} ({:.2}%)",
        100.0 * found as f64 / bones.max(1) as f64,
        100.0 * in_root as f64 / bones.max(1) as f64,
    );
    eprintln!("rest named bones: {compared} visuals compared, {mismatched} differing");
    assert_eq!(
        mismatched, 0,
        "the assets port lists the render skeleton's bones"
    );
    assert!(bones > 0, "the dataset has bone FX nodes");
    assert!(
        found > in_root,
        "the child part groups resolve bones the root Setup lacks"
    );
    assert!(
        found as f64 >= 0.80 * bones as f64,
        "{found} of {bones} FX bone nodes resolve"
    );
}
