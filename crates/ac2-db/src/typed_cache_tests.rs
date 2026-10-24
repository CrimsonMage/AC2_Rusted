use std::sync::Arc;
use std::time::Duration;

use ac2_core::{DataId, QualifiedDataId};
use ac2_dat::DatFileKind;

use super::{
    CachedResource, ResourceCacheConfig, ResourceCacheMaintenanceOutcome, ResourceCacheObservation,
    TypedResourceCache,
};
use crate::ResourceKey;

#[derive(Debug)]
struct CacheValueA(u32);

#[derive(Debug)]
struct CacheValueB(u32);

fn typed_key(raw_did: u32, type_id: u32) -> ResourceKey {
    ResourceKey::new(
        DatFileKind::Portal,
        QualifiedDataId::new(DataId::new(raw_did), type_id),
    )
}

fn fixture_key(raw_did: u32) -> ResourceKey {
    typed_key(raw_did, crate::ResourceTypeId::FixtureA as u32)
}

/// A key in the landblock family, whose ideal free count (21) is below its maximum (144).
fn land_key(raw_did: u32) -> ResourceKey {
    typed_key(raw_did, crate::ResourceTypeId::LandBlock as u32)
}

fn maintain(cache: &TypedResourceCache) -> super::ResourceCacheMaintenanceReport {
    match cache.try_maintain() {
        ResourceCacheMaintenanceOutcome::Completed(report) => report,
        ResourceCacheMaintenanceOutcome::Busy => panic!("uncontended cache maintenance was busy"),
    }
}

#[test]
fn keeps_distinct_rust_types_for_one_resource_key() {
    let cache = TypedResourceCache::default();
    let shared_key = typed_key(0x2200_0101, crate::ResourceTypeId::FixtureA as u32);
    let value_a = cache.insert_with_retained_bytes(shared_key, Arc::new(CacheValueA(10)), 4);
    let value_b = cache.insert_with_retained_bytes(shared_key, Arc::new(CacheValueB(20)), 8);

    let loaded_a = cache.get::<CacheValueA>(shared_key).unwrap();
    let loaded_b = cache.get::<CacheValueB>(shared_key).unwrap();
    assert!(CachedResource::ptr_eq(&value_a, &loaded_a));
    assert!(CachedResource::ptr_eq(&value_b, &loaded_b));
    assert_eq!((loaded_a.0, loaded_b.0), (10, 20));
    assert_eq!(cache.stats().entry_count(), 2);
    assert_eq!(cache.stats().retained_bytes(), 12);
}

#[test]
fn only_the_final_clone_queues_a_release() {
    let cache = TypedResourceCache::default();
    let key = fixture_key(0x2200_0201);
    let first = cache.insert_with_retained_bytes(key, Arc::new(CacheValueA(1)), 1);
    let last = first.clone();

    drop(first);
    let report = maintain(&cache);
    assert_eq!(report.lifecycle_events(), 0);

    drop(last);
    let report = maintain(&cache);
    assert_eq!(report.lifecycle_events(), 1);
    assert_eq!(report.stale_events(), 0);
    assert_eq!(report.family_activity()[0].releases_accepted(), 1);
}

#[test]
fn stats_separate_active_free_and_explicitly_pinned_ownership() {
    let cache = TypedResourceCache::default();
    let active_key = fixture_key(0x2200_0202);
    let free_key = fixture_key(0x2200_0203);
    let pinned_key = fixture_key(0x2200_0204);
    let active = cache.insert_with_retained_bytes(active_key, Arc::new(CacheValueA(1)), 10);
    drop(cache.insert_with_retained_bytes(free_key, Arc::new(CacheValueA(2)), 20));
    let pinned = cache.insert_with_retained_bytes(pinned_key, Arc::new(CacheValueA(3)), 30);
    let pins = cache.retain_resource(pinned_key);
    drop(pinned);

    let stats = cache.stats();
    assert_eq!((stats.entry_count(), stats.retained_bytes()), (3, 60));
    assert_eq!(
        (stats.active_entry_count(), stats.active_retained_bytes()),
        (2, 40)
    );
    assert_eq!(
        (stats.free_entry_count(), stats.free_retained_bytes()),
        (1, 20)
    );
    assert_eq!(
        (stats.pinned_entry_count(), stats.pinned_retained_bytes()),
        (1, 30)
    );

    drop(active);
    drop(pins);
    let stats = cache.stats();
    assert_eq!(
        (stats.active_entry_count(), stats.active_retained_bytes()),
        (0, 0)
    );
    assert_eq!(
        (stats.free_entry_count(), stats.free_retained_bytes()),
        (3, 60)
    );
    assert_eq!(
        (stats.pinned_entry_count(), stats.pinned_retained_bytes()),
        (0, 0)
    );
}

#[test]
fn try_stats_never_waits_for_a_busy_family() {
    let cache = TypedResourceCache::default();
    let key = fixture_key(0x2200_0205);
    let _active = cache.insert_with_retained_bytes(key, Arc::new(CacheValueA(1)), 10);
    let family = cache.state.family(key.type_id()).unwrap();
    let _guard = family.write().unwrap();

    assert_eq!(cache.try_stats(), None);
}

#[test]
fn reacquiring_before_maintenance_makes_the_old_release_stale() {
    let cache = TypedResourceCache::default();
    let key = fixture_key(0x2200_0301);
    drop(cache.insert_with_retained_bytes(key, Arc::new(CacheValueA(1)), 1));

    let active = cache.get::<CacheValueA>(key).unwrap();
    let report = maintain(&cache);
    assert_eq!(report.lifecycle_events(), 1);
    assert_eq!(report.stale_events(), 1);
    assert_eq!(cache.stats().entry_count(), 1);

    drop(active);
    let report = maintain(&cache);
    assert_eq!(report.lifecycle_events(), 1);
    assert_eq!(report.stale_events(), 0);
}

#[test]
fn hot_reacquire_release_lane_coalesces_to_one_lifecycle_event() {
    let cache = TypedResourceCache::default();
    let key = fixture_key(0x2200_0302);
    drop(cache.insert_with_retained_bytes(key, Arc::new(CacheValueA(1)), 1));

    for _ in 0..10_000 {
        drop(cache.get::<CacheValueA>(key).unwrap());
    }

    let report = maintain(&cache);
    assert_eq!(report.lifecycle_events(), 1);
    assert_eq!(report.stale_events(), 0);
    assert_eq!(report.family_activity()[0].releases_accepted(), 1);
    assert_eq!(maintain(&cache).lifecycle_events(), 0);
}

#[test]
fn stale_detached_lifecycle_never_releases_its_replacement() {
    let cache = TypedResourceCache::default();
    let key = fixture_key(0x2200_0303);
    let old = cache.insert_with_retained_bytes(key, Arc::new(CacheValueA(1)), 1);
    assert_eq!(cache.invalidate_resource(key.file_kind(), key.data_id()), 1);
    drop(cache.insert_with_retained_bytes(key, Arc::new(CacheValueA(2)), 1));
    drop(old);

    let report = maintain(&cache);
    assert_eq!(report.lifecycle_events(), 2);
    assert_eq!(report.stale_events(), 1);
    assert_eq!(cache.peek::<CacheValueA>(key).unwrap().0, 2);
}

#[test]
fn retail_maximum_evicts_the_oldest_free_entry() {
    let cache = TypedResourceCache::default();
    for offset in 0..17 {
        drop(cache.insert_with_retained_bytes(
            fixture_key(0x2200_1000 + offset),
            Arc::new(CacheValueA(offset)),
            1,
        ));
    }

    let report = maintain(&cache);
    assert_eq!(report.lifecycle_events(), 17);
    assert_eq!(report.evicted_over_maximum(), 1);
    assert_eq!(cache.stats().entry_count(), 16);
    assert!(cache.get::<CacheValueA>(fixture_key(0x2200_1000)).is_none());
    assert!(cache.get::<CacheValueA>(fixture_key(0x2200_1010)).is_some());
}

#[test]
fn retail_use_time_ages_one_oldest_entry_above_ideal() {
    let cache =
        TypedResourceCache::new(ResourceCacheConfig::default().with_idle_grace(Duration::ZERO));
    for offset in 0..22 {
        drop(cache.insert_with_retained_bytes(
            land_key(0x2200_2000 + offset),
            Arc::new(CacheValueA(offset)),
            1,
        ));
    }

    let report = maintain(&cache);
    assert_eq!(report.evicted_idle(), 1);
    assert_eq!(cache.stats().entry_count(), 21);
    let report = maintain(&cache);
    assert_eq!(report.evicted_idle(), 0);
}

#[test]
fn aged_backlog_continues_without_waiting_and_stops_at_the_retail_floor() {
    let cache =
        TypedResourceCache::new(ResourceCacheConfig::default().with_idle_grace(Duration::ZERO));
    for offset in 0..24 {
        drop(cache.insert_with_retained_bytes(
            land_key(0x2200_2800 + offset),
            Arc::new(CacheValueA(offset)),
            1,
        ));
    }
    for expected_remaining in (21..24).rev() {
        let report = maintain(&cache);
        assert_eq!(report.evicted_idle(), 1);
        assert_eq!(cache.stats().entry_count(), expected_remaining);
        assert_eq!(report.more_work(), expected_remaining > 21);
    }
    assert_eq!(maintain(&cache).evicted_entries(), 0);
}

#[test]
fn young_free_backlog_waits_for_the_idle_grace() {
    let cache = TypedResourceCache::default();
    for offset in 0..6 {
        drop(cache.insert_with_retained_bytes(
            fixture_key(0x2200_2900 + offset),
            Arc::new(CacheValueA(offset)),
            1,
        ));
    }
    let report = maintain(&cache);
    assert_eq!(report.evicted_idle(), 0);
    assert!(!report.more_work());
}

#[test]
fn active_checkout_and_explicit_pin_are_not_evicted() {
    let cache =
        TypedResourceCache::new(ResourceCacheConfig::default().with_idle_grace(Duration::ZERO));
    let active_key = fixture_key(0x2200_3000);
    let pinned_key = fixture_key(0x2200_3001);
    let active = cache.insert_with_retained_bytes(active_key, Arc::new(CacheValueA(1)), 1);
    let pinned = cache.insert_with_retained_bytes(pinned_key, Arc::new(CacheValueA(2)), 1);
    let pin = cache.retain_resource(pinned_key);
    assert_eq!(pin.len(), 1);
    drop(pinned);
    for offset in 0..4 {
        drop(cache.insert_with_retained_bytes(
            fixture_key(0x2200_3010 + offset),
            Arc::new(CacheValueA(offset)),
            1,
        ));
    }

    maintain(&cache);
    assert_eq!(active.0, 1);
    assert!(cache.peek::<CacheValueA>(active_key).is_some());
    assert!(cache.peek::<CacheValueA>(pinned_key).is_some());
    drop(pin);
    drop(active);
}

#[test]
fn free_byte_pressure_reaps_to_the_low_water_across_bounded_passes() {
    const ENTRY_BYTES: usize = 8 * 1024 * 1024;
    let cache = TypedResourceCache::default();
    for offset in 0..5 {
        drop(cache.insert_with_retained_bytes(
            fixture_key(0x2200_3800 + offset),
            Arc::new(CacheValueA(offset)),
            ENTRY_BYTES,
        ));
    }

    let first = maintain(&cache);
    assert_eq!(first.free_retained_bytes_before(), 5 * ENTRY_BYTES);
    assert_eq!(first.evicted_pressure(), 1);
    assert_eq!(first.pressure_evicted_bytes(), ENTRY_BYTES);
    assert_eq!(first.free_retained_bytes_after(), 4 * ENTRY_BYTES);
    assert!(first.more_work());

    let second = maintain(&cache);
    assert_eq!(second.evicted_pressure(), 1);
    assert_eq!(second.free_retained_bytes_after(), 3 * ENTRY_BYTES);
    assert!(!second.more_work());
    assert_eq!(cache.stats().entry_count(), 3);
}

#[test]
fn free_byte_pressure_never_counts_active_or_pinned_values() {
    const ENTRY_BYTES: usize = 8 * 1024 * 1024;
    let cache = TypedResourceCache::default();
    let active_key = fixture_key(0x2200_3900);
    let pinned_key = fixture_key(0x2200_3901);
    let active =
        cache.insert_with_retained_bytes(active_key, Arc::new(CacheValueA(1)), ENTRY_BYTES);
    let pinned =
        cache.insert_with_retained_bytes(pinned_key, Arc::new(CacheValueA(2)), ENTRY_BYTES);
    let pins = cache.retain_resource(pinned_key);
    drop(pinned);
    for offset in 0..4 {
        drop(cache.insert_with_retained_bytes(
            fixture_key(0x2200_3910 + offset),
            Arc::new(CacheValueA(offset)),
            ENTRY_BYTES,
        ));
    }

    let report = maintain(&cache);
    assert_eq!(report.free_retained_bytes_before(), 4 * ENTRY_BYTES);
    assert_eq!(report.evicted_pressure(), 1);
    assert!(cache.peek::<CacheValueA>(active_key).is_some());
    assert!(cache.peek::<CacheValueA>(pinned_key).is_some());
    assert_eq!(active.0, 1);
    drop(pins);
    drop(active);
}

#[test]
fn nonblocking_observation_distinguishes_hit_and_miss() {
    let cache = TypedResourceCache::default();
    let key = fixture_key(0x2200_4000);
    let value = cache.insert_with_retained_bytes(key, Arc::new(CacheValueA(7)), 1);

    match cache.try_observe::<CacheValueA>(key) {
        ResourceCacheObservation::Hit(observed) => {
            assert!(CachedResource::ptr_eq(&value, &observed));
        }
        other => panic!("expected hit, got {other:?}"),
    }
    assert!(matches!(
        cache.try_observe::<CacheValueA>(fixture_key(0x2200_4001)),
        ResourceCacheObservation::Miss
    ));
}

#[test]
fn weak_resource_identity_does_not_hold_a_checkout_or_payload() {
    let cache = TypedResourceCache::default();
    let key = fixture_key(0x2200_0310);
    let value = cache.insert_with_retained_bytes(key, Arc::new(vec![3_u8; 1024]), 1024);
    let weak = value.downgrade();
    drop(value);
    assert_eq!(maintain(&cache).family_activity()[0].releases_accepted(), 1);
    drop(cache);
    assert!(weak.upgrade().is_none());
}
