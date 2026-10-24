#![forbid(unsafe_code)]

//! Generic typed DBObj validation, cache policy, and request scheduling above DAT storage.
//!
//! This crate owns qualified DID/type classification, validated DBObj header/body boundaries,
//! typed-reference identity, generic cache residency/eviction/invalidation, dependency-plan
//! deduplication, and deterministic request completion/cancellation events.
//!
//! Domain crates own their DBObj body schemas and standalone packed values through
//! [`ResourceBodyDecode`], [`ResourceBodyDecodeWith`], and [`ResourcePackedDecode`]. This crate
//! does not expose raw DAT records, metadata proxies, filename lookup, or semantic loaders.

mod body_decode;
mod db_type;
mod ddd;
mod decode;
mod decode_failure;
mod decode_flight;
mod deferred_work;
mod error;
mod key;
mod ownership_stats;
mod packed_decode;
mod prefetch_sequence;
mod preparation_executor;
mod preparation_queue;
mod property;
mod resource_runtime;
mod retail_cache_retention;
mod store;
#[cfg(test)]
mod test_support;
mod thread_executor;
mod typed_cache;
mod typed_ref;
mod worker_lanes;

pub use body_decode::ResourceBodyDecodeWith;
pub use body_decode::{ResourceBodyDecode, ResourceDbObjBody, ResourceDbObjDecodeContext};
pub use db_type::ResourceTypeId;

pub use ddd::{ResourceDddApplyOutcome, ResourceDddInvalidation};
pub use deferred_work::spawn_deferred_on;
pub use deferred_work::{spawn_deferred, DeferredResult};
pub use error::{ResourceError, ResourceResult};
pub use key::ResourceKey;
pub use ownership_stats::ResourceOwnershipStats;
pub use ownership_stats::{
    ResourceOwnershipLane, ResourceOwnershipTypeStats, ResourceScopeOverlapStats,
    ResourceScopeOverlapTypeStats,
};
pub use packed_decode::ResourcePackedDecode;
pub use prefetch_sequence::TypedPrefetchSequence;
pub use preparation_executor::NonblockingPreparationSubmitError;
pub use preparation_executor::{
    InlinePreparationExecutor, PreparationDomain, PreparationExecutor, PreparationJob,
    PreparationPriority, PreparationSubmitError,
};
pub use preparation_queue::{
    GetAdmission, InFlightGet, InFlightGetRegistry, OrderedSpillQueue, PreparationDependencyFanIn,
    PreparationError, PreparationReplySequence, PreparationRequestSequence, PreparationStep,
    PreparationTransport, SequencedPreparation, RETAIL_PREPARATION_FAST_QUEUE_CAPACITY,
};
pub use property::{
    install_ac2_property_types, read_ac2_property_collection, skip_ac2_property_collection,
    skip_property_collection, Ac2PropertyTypes, Ac2PropertyValue,
};
pub use resource_runtime::ResourceRuntime;
pub use resource_runtime::{
    ResourcePriority, ResourceRequestContext, ResourceRequestOutcome, ResourceRequestProgress,
    ResourceRuntimeConfig, ResourceRuntimeEvent,
};
pub use retail_cache_retention::{
    retail_cache_retention_for_type, RetailCacheRetention, RETAIL_CACHE_IDLE_GRACE_SECONDS,
};
pub use store::ResourceResidencyLease;
pub use store::ResourceStore;
pub use thread_executor::ThreadPreparationExecutor;
pub use typed_cache::{CachedResource, ResourceCacheStats, ResourceDecodeScope};
pub use typed_cache::{
    ResourceCacheFamilyMaintenanceReport, ResourceCacheMaintenanceOutcome,
    ResourceCacheMaintenanceReport, ResourceCacheObservation,
};
pub use typed_ref::TypedResourceRef;
pub use worker_lanes::{
    frame_map, frame_map_on, frame_map_with_threshold, global_worker_lanes,
    install_global_worker_lanes, WorkerLaneWaiter, WorkerLanes, FRAME_PARALLEL_MIN_ITEMS,
};
