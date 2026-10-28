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

mod base_property;
#[cfg(test)]
mod base_property_tests;
mod body_decode;
mod db_type;
mod ddd;
mod decode;
mod decode_failure;
mod decode_flight;
mod deferred_work;
mod error;
mod key;
mod loader_pool;
mod ownership_stats;
mod packed_decode;
mod prefetch_sequence;
mod preparation_executor;
mod preparation_queue;
mod property;
mod property_collection;
#[cfg(test)]
mod property_collection_tests;
#[cfg(test)]
mod property_tests;
mod resource_runtime;
mod store;
mod string_info;
#[cfg(test)]
mod string_info_tests;
mod string_meta_lex;
mod string_meta_parse;
mod string_meta_render;
mod string_meta_text;
mod string_number;
mod string_state;
mod string_table;
mod string_tables;
#[cfg(test)]
mod test_support;
mod thread_executor;
mod typed_cache;
mod typed_ref;
mod worker_lanes;

pub use base_property::{
    BaseProperty, BasePropertyValue, PropertyPosition, RgbaColor, StringInfoValue,
};
pub use body_decode::ResourceBodyDecodeWith;
pub use body_decode::{ResourceBodyDecode, ResourceDbObjBody, ResourceDbObjDecodeContext};
pub use db_type::ResourceTypeId;
pub use property_collection::{PropertyCollection, PropertyGroup};

pub use ddd::{ResourceDddApplyOutcome, ResourceDddInvalidation};
pub use deferred_work::spawn_deferred_on;
pub use deferred_work::{spawn_deferred, DeferredResult};
pub use error::{ResourceError, ResourceResult};
pub use key::ResourceKey;
pub use loader_pool::{LoaderPool, WORLD_LOADER_THREADS};
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
    ac2_ethereal_mask, ac2_property_desc, ac2_property_inquirable, ac2_property_name,
    ac2_property_name_enum, ac2_property_names, ac2_property_reported_to_weenie, ac2_property_type,
    ethereal_mask_with, install_ac2_property_types, read_ac2_base_property_array,
    read_ac2_property_collection, read_ac2_property_groups, skip_ac2_property_collection,
    skip_property_collection, update_ac2_property_collection, Ac2PropertyDesc, Ac2PropertyGroup,
    Ac2PropertyTypes, Ac2PropertyValue, REPORT_IGNORED, REPORT_INQUIRABLE, REPORT_REPORTABLE,
};
pub use resource_runtime::ResourceRuntime;
pub use resource_runtime::{
    ResourcePriority, ResourceRequestContext, ResourceRequestOutcome, ResourceRequestProgress,
    ResourceRuntimeConfig, ResourceRuntimeEvent,
};
pub use store::ResourceResidencyLease;
pub use store::ResourceStore;
pub use string_info::{
    get_log_string, get_string, get_string_with_meta_tags, inq_string, inq_string_internal,
    render_with_table, GetStringError, GetStringFailure, RenderInfo, RenderVar, StringStateRef,
    StringTableRef, StringTables, StringVarTable,
};
pub use string_meta_parse::{RenderErr, RenderErrorCode};
pub use string_meta_render::{get_match_score, render_string};
pub use string_meta_text::{
    apply_meta_tags_to_name, capitalize_first_letter, escape_string, is_escape_char,
    retrieve_meta_letters, strip_meta_letters, trim_excess_spaces, unescape_string,
};
pub use string_state::{
    female_meta_letters, is_number_singular, load_string_state, male_meta_letters,
    treasure_meta_letters, StringState, STRING_STATE_ENUM, STRING_STATE_ENUM_GROUP,
    STRING_STATE_FIRST, STRING_STATE_LAST,
};
pub use string_table::{
    load_string_table, read_wide_pstring, StringTable, StringTableString, STRING_TABLE_FIRST,
    STRING_TABLE_LAST,
};
pub use string_tables::{PreloadedStringTables, StoreStringTables};
pub use thread_executor::ThreadPreparationExecutor;
pub use typed_cache::{
    CachedResource, ResourceCacheStats, ResourceDecodeScope, DEFAULT_FREE_CACHE_BUDGET_BYTES,
};
pub use typed_cache::{
    ResourceCacheFamilyMaintenanceReport, ResourceCacheMaintenanceOutcome,
    ResourceCacheMaintenanceReport, ResourceCacheObservation,
};
pub use typed_ref::TypedResourceRef;
pub use worker_lanes::{
    frame_map, frame_map_on, frame_map_with_threshold, frame_worker_count, global_worker_lanes,
    install_frame_workers, install_global_worker_lanes, WorkerLaneWaiter, WorkerLanes,
    FRAME_PARALLEL_MIN_ITEMS, MAX_FRAME_WORKERS,
};
