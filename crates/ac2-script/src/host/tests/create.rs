//! `ScriptHost::create_weenie` (`CWDistributor::Construct`) on a cache-only host: an entity's
//! WState the cache does not hold yet is a wait, not a fresh instance.

use super::*;

/// `CWDistributor::Construct` reads the entity's WState through `DBObj::Get`, which blocks until
/// the record is there; only an absent record falls back to a fresh instance. On a cache-only
/// host the read is a request instead: `create_weenie` answers false with the WState among the
/// data requests (no process is made), the loader reads it, and the same call then succeeds from
/// that WState.
#[test]
fn create_weenie_waits_for_a_cache_only_wstate_then_retries() {
    let Some(dataset) = ac2_dat::test_support::env_dataset() else {
        return;
    };
    let store = ResourceStore::new(dataset.clone());
    let wlib = load_wlib(&store, DataId::new(LEGIONS_WLIB_DID))
        .ok()
        .flatten()
        .expect("the Legions script library");
    let program = Program::from_stream(&wlib.stream);
    let pid = program.package_id("CPlayer").expect("CPlayer");
    let data = DatScriptData::new(dataset, store);
    data.preload_tables();
    data.set_cache_only(true);
    let mut host = ScriptHost::new(Arc::new(program), Box::new(data.clone()));

    let player = 0x5000_0001;
    // The entity DID 0x4700110F is a 0x40000049 object: its WState is 0x71000000 | 0x00110F.
    assert!(!host.create_weenie(player, pid, 0x4700_110F));
    assert_eq!(
        host.take_data_requests(),
        vec![DataRequest::WState(0x7100_110F)]
    );
    data.load(DataRequest::WState(0x7100_110F));
    assert!(host.create_weenie(player, pid, 0x4700_110F));
    assert!(host.take_data_requests().is_empty());
}
