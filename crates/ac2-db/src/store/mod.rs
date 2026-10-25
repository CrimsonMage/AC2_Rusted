use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use ac2_core::DataId;
#[cfg(test)]
use ac2_dat::DatEntryMeta;
use ac2_dat::{DatDataset, DatFileKind, DatReadAttribution};

use crate::decode::{decode_resource_body, decode_resource_packed_body, DecodeContext};
use crate::decode_failure::DecodeFailureDiagnostics;
use crate::decode_flight::{DecodeFlightJoin, DecodeFlights, FlightValue};
use crate::error::ResourceResult;
use crate::typed_cache::{
    CacheValue, CachedResource, ResourceCacheConfig, ResourceCacheMaintenanceOutcome,
    ResourceCacheObservation, ResourceCachePin, ResourceCacheStats, ResourceDecodeScope,
    TypedCacheKey, TypedResourceCache,
};
use crate::{
    ResourceBodyDecode, ResourceBodyDecodeWith, ResourceDbObjBody, ResourceDbObjDecodeContext,
    ResourceError, ResourceKey, ResourceOwnershipStats, ResourcePackedDecode,
    ResourceScopeOverlapStats, ResourceTypeId, TypedResourceRef,
};

mod access;
mod decode_input;
mod texture_detail;

use access::decompress_record_bytes;
use decode_input::ResourceDecodeInput;

#[derive(Debug, Clone)]
pub struct ResourceStore {
    dataset: Arc<DatDataset>,
    typed_cache: TypedResourceCache,
    decode_flights: Arc<DecodeFlights>,
    decode_failures: Arc<DecodeFailureDiagnostics>,
    high_detail_textures: Arc<AtomicBool>,
}

struct BodyLoad<'a, Decode, Charge> {
    scope: Option<&'a ResourceDecodeScope>,
    reference: TypedResourceRef,
    type_name: &'static str,
    expected_type: ResourceTypeId,
    decode: Decode,
    retained_bytes: Charge,
    retain_in_typed_cache: bool,
}

enum LoadedResource<T> {
    Cached(CachedResource<T>),
    Uncached(Arc<T>),
}

/// Generic typed-cache residency claim for already-decoded resource references.
///
/// Domain owners decide which closure is active; resources merely retains/releases the matching
/// cache keys. References absent from the cache are ignored so this never decodes or schedules.
#[derive(Debug)]
pub struct ResourceResidencyLease {
    pins: Vec<ResourceCachePin>,
}

impl ResourceResidencyLease {
    pub fn resource_count(&self) -> usize {
        self.pins.len()
    }
    pub fn is_empty(&self) -> bool {
        self.pins.is_empty()
    }
}

impl ResourceStore {
    pub fn new(dataset: Arc<DatDataset>) -> Self {
        Self::with_cache_config(dataset, ResourceCacheConfig::default())
    }

    /// The DATs the store reads.
    pub fn dataset(&self) -> &Arc<DatDataset> {
        &self.dataset
    }

    pub(crate) fn with_cache_config(
        dataset: Arc<DatDataset>,
        cache_config: ResourceCacheConfig,
    ) -> Self {
        Self {
            dataset,
            typed_cache: TypedResourceCache::new(cache_config),
            decode_flights: Arc::new(DecodeFlights::default()),
            decode_failures: Arc::new(DecodeFailureDiagnostics::default()),
            high_detail_textures: Arc::new(AtomicBool::new(false)),
        }
    }

    #[cfg(test)]
    pub(crate) fn cache_config(&self) -> ResourceCacheConfig {
        self.typed_cache.config()
    }

    pub fn cache_stats(&self) -> ResourceCacheStats {
        self.typed_cache.stats()
    }

    /// Returns per-type retained-byte observation for the session-wide typed cache.
    ///
    /// Scoped and ephemeral values are intentionally excluded; inspect their
    /// [`ResourceDecodeScope::ownership_stats`] snapshots separately.
    pub fn ownership_stats(&self) -> ResourceOwnershipStats {
        self.typed_cache.ownership_stats()
    }

    /// Creates one caller-owned decoded-resource lifetime. Values loaded through
    /// `load_scoped_body` are retained only by this handle and its clones.
    pub fn begin_decode_scope(&self) -> ResourceDecodeScope {
        ResourceDecodeScope::default()
    }

    /// Retains only references already present in the generic typed cache.
    pub fn retain_cached_resources(
        &self,
        references: &[TypedResourceRef],
    ) -> ResourceResidencyLease {
        let cache = self.typed_cache_handle();
        let pins = references
            .iter()
            .flat_map(|reference| cache.retain_resource(reference.resource_key()))
            .collect();
        ResourceResidencyLease { pins }
    }

    /// Returns an already-resident typed value without decoding or refreshing its cache entry.
    ///
    /// The cache's concrete Rust value type remains part of its identity, so this is suitable for
    /// both DBObj bodies and standalone packed values. It performs no storage or metadata access.
    pub fn peek_cached<T>(&self, reference: TypedResourceRef) -> Option<CachedResource<T>>
    where
        T: Send + Sync + 'static,
    {
        self.typed_cache.peek(reference.resource_key())
    }

    /// Returns an already-resident value only when the generic typed cache can be observed
    /// without waiting for a decode or eviction mutation.
    ///
    /// This is the frame-budgeted counterpart to [`Self::peek_cached`]. A contended cache is
    /// treated as unavailable so active world work never turns a resource hand-off into a
    /// blocking cache wait.
    pub fn try_peek_cached<T>(&self, reference: TypedResourceRef) -> Option<CachedResource<T>>
    where
        T: Send + Sync + 'static,
    {
        self.typed_cache.try_peek(reference.resource_key())
    }

    /// Distinguishes a definite cache miss from temporary cache mutation without waiting.
    pub fn try_observe_cached<T>(&self, reference: TypedResourceRef) -> ResourceCacheObservation<T>
    where
        T: Send + Sync + 'static,
    {
        self.typed_cache.try_observe(reference.resource_key())
    }

    /// Returns typed-cache observation only when it does not contend with decode/cache mutation.
    pub fn try_cache_stats(&self) -> Option<ResourceCacheStats> {
        self.typed_cache.try_stats()
    }

    /// Returns per-type generic-cache observation only when it can avoid cache contention.
    pub fn try_ownership_stats(&self) -> Option<ResourceOwnershipStats> {
        self.typed_cache.try_ownership_stats()
    }

    /// Returns a non-blocking logical ownership comparison for one active decode scope.
    ///
    /// A shared value is represented once by its `Arc`; a distinct scoped value is only a
    /// measured candidate for future cache-policy work, never an automatic deduplication order.
    pub fn try_scope_overlap_stats(
        &self,
        scope: &ResourceDecodeScope,
    ) -> Option<ResourceScopeOverlapStats> {
        self.typed_cache.try_scope_overlap_stats(scope)
    }

    /// Returns whether the mounted dataset has a record for a known typed reference.
    ///
    /// This is a deliberately narrow presence query for source-selection and residency policy.
    /// It neither materializes packed bytes nor exposes DAT metadata, and it does not imply that
    /// the record's DBObj header or body is valid.
    pub fn has_typed_resource(&self, reference: TypedResourceRef) -> bool {
        reference.resource_type_id().is_some()
            && self
                .dataset
                .contains(reference.file_kind(), reference.data_id().raw())
    }

    /// Reads and validates the DBObj header, then returns the first little-endian body word.
    ///
    /// Some retail presence decisions need a scalar body discriminator (for example the
    /// `LandBlock::has_objects` flag) but must not allocate a domain's full decoded object just
    /// to obtain it.  This preserves resource-owned DAT access and header validation while
    /// intentionally keeping the public result to one scalar.  Callers remain responsible for
    /// assigning semantic meaning to that word.
    pub fn probe_dbobj_body_u32_le(
        &self,
        reference: TypedResourceRef,
    ) -> ResourceResult<Option<u32>> {
        let Some(expected_type) = reference.resource_type_id() else {
            return Err(ResourceError::MalformedTypedObject {
                qdid: reference.qualified_data_id(),
                type_name: "DBObj body scalar probe",
                context: "qualified data id did not name a known resource type",
            });
        };
        let key = reference.resource_key();
        let Some(input) = self.read_record_for_decode(
            key.file_kind(),
            key.data_id(),
            DatReadAttribution::EphemeralWorldTooling,
        )?
        else {
            return Ok(None);
        };
        let (_, body) = decode_resource_body(
            key.qualified_data_id(),
            "DBObj body scalar probe",
            expected_type,
            input.entry().version,
            input.as_slice(),
        )?;
        let Some(bytes) = body.bytes().get(..std::mem::size_of::<u32>()) else {
            return Err(ResourceError::MalformedTypedObject {
                qdid: key.qualified_data_id(),
                type_name: "DBObj body scalar probe",
                context: "missing first DBObj body word",
            });
        };
        Ok(Some(u32::from_le_bytes(
            bytes
                .try_into()
                .expect("u32-sized body prefix must convert"),
        )))
    }

    /// Reapplies typed-object free-entry retention without waiting for an in-progress decode or
    /// cache mutation. A skipped pass is reported as `Busy`, allowing the client-owned
    /// preparation lane to preserve active-world responsiveness.
    pub fn try_maintain_typed_cache(&self) -> ResourceCacheMaintenanceOutcome {
        self.typed_cache.try_maintain()
    }

    #[cfg(test)]
    pub(crate) fn entry(&self, key: ResourceKey) -> Option<DatEntryMeta> {
        self.dataset.entry(key.file_kind(), key.data_id().raw())
    }

    pub(crate) fn read_record_for_decode(
        &self,
        file_kind: DatFileKind,
        did: DataId,
        attribution: DatReadAttribution,
    ) -> ResourceResult<Option<ResourceDecodeInput>> {
        // DAT resolves metadata and reads one decode-local packed record together, so decode
        // never repeats the DID lookup.
        let Some(record) = self
            .dataset
            .read_record(file_kind, did.raw(), attribution)?
        else {
            return Ok(None);
        };
        let entry = record.meta;
        if entry.is_stored_compressed() {
            return decompress_record_bytes(file_kind, did, record.packed_bytes)
                .map(|bytes| bytes.map(|bytes| ResourceDecodeInput::decompressed(entry, bytes)));
        }
        Ok(Some(ResourceDecodeInput::packed(
            entry,
            record.packed_bytes,
        )))
    }

    /// Loads a domain-owned immutable DBObj body through resource-owned metadata/header policy.
    ///
    /// This is the primary DBObj extension seam. It does not expose DAT bytes, entry metadata,
    /// or header-layout decisions.
    pub fn load_cached_body<T>(
        &self,
        reference: TypedResourceRef,
    ) -> ResourceResult<Option<CachedResource<T>>>
    where
        T: ResourceBodyDecode + Send + Sync + 'static,
    {
        self.load_body_inner(BodyLoad {
            scope: None,
            reference,
            type_name: T::TYPE_NAME,
            expected_type: T::RESOURCE_TYPE,
            decode: T::decode_body,
            retained_bytes: T::cache_retained_bytes,
            retain_in_typed_cache: true,
        })
        .map(|value| value.map(expect_cached_resource))
    }

    /// Loads one canonical Portal-DAT DBObj body by its decoder-declared resource type.
    ///
    /// Portal routing is resource lookup policy. Higher domain owners supply only the authored
    /// DID and never repeat `DatFileKind`/typed-reference construction at composition roots.
    pub fn load_portal_cached_body<T>(
        &self,
        did: DataId,
    ) -> ResourceResult<Option<CachedResource<T>>>
    where
        T: ResourceBodyDecode + Send + Sync + 'static,
    {
        self.load_cached_body(TypedResourceRef::from_resource_type(
            DatFileKind::Portal,
            did,
            T::RESOURCE_TYPE,
        ))
    }

    /// Loads a validated immutable DBObj body without creating a generic typed-cache owner.
    ///
    /// This is for a domain that already owns the decoded value for its active lifetime (the
    /// static-world scope is the current consumer).  It deliberately bypasses both generic-cache
    /// lookup and insertion so an inactive scope cannot be kept alive by a session cache.
    /// Callers must provide their own active-scope sharing; this is not a replacement cache.
    pub fn load_ephemeral_body<T>(
        &self,
        reference: TypedResourceRef,
    ) -> ResourceResult<Option<Arc<T>>>
    where
        T: ResourceBodyDecode + Send + Sync + 'static,
    {
        self.load_body_inner(BodyLoad {
            scope: None,
            reference,
            type_name: T::TYPE_NAME,
            expected_type: T::RESOURCE_TYPE,
            decode: T::decode_body,
            retained_bytes: T::cache_retained_bytes,
            retain_in_typed_cache: false,
        })
        .map(|value| value.map(expect_uncached_resource))
    }

    /// Loads a DBObj body into a caller-owned active decode scope.
    ///
    /// The scope supplies sharing only while its owner remains active; it neither consults nor
    /// inserts into the session-wide typed cache. Concurrent body loads still join the same
    /// transient typed flight to avoid duplicate DAT read/decode work.
    pub fn load_scoped_body<T>(
        &self,
        scope: &ResourceDecodeScope,
        reference: TypedResourceRef,
    ) -> ResourceResult<Option<Arc<T>>>
    where
        T: ResourceBodyDecode + Send + Sync + 'static,
    {
        self.load_body_inner(BodyLoad {
            scope: Some(scope),
            reference,
            type_name: T::TYPE_NAME,
            expected_type: T::RESOURCE_TYPE,
            decode: T::decode_body,
            retained_bytes: T::cache_retained_bytes,
            retain_in_typed_cache: false,
        })
        .map(|value| value.map(expect_uncached_resource))
    }

    /// Loads a validated DBObj body through a destination-owned decoder with stable companion
    /// schema state.
    ///
    /// Resources owns mounted-record access, decompression, DBObj header validation, typed cache
    /// identity, and retention. The decoder receives no store or raw-storage capability, so a
    /// destination may supply already-resolved schema data without pulling its policy downward.
    pub fn load_cached_body_with<T>(
        &self,
        reference: TypedResourceRef,
        state: &T::DecodeState,
    ) -> ResourceResult<Option<CachedResource<T>>>
    where
        T: ResourceBodyDecodeWith,
    {
        self.load_body_inner(BodyLoad {
            scope: None,
            reference,
            type_name: T::TYPE_NAME,
            expected_type: T::RESOURCE_TYPE,
            decode: |context: ResourceDbObjDecodeContext, body: ResourceDbObjBody<'_>| {
                T::decode_body_with(state, context, body)
            },
            retained_bytes: T::cache_retained_bytes,
            retain_in_typed_cache: true,
        })
        .map(|value| value.map(expect_cached_resource))
    }

    fn load_body_inner<T, Decode, Charge>(
        &self,
        load: BodyLoad<'_, Decode, Charge>,
    ) -> ResourceResult<Option<LoadedResource<T>>>
    where
        T: Send + Sync + 'static,
        Decode: Fn(ResourceDbObjDecodeContext, ResourceDbObjBody<'_>) -> ResourceResult<T>,
        Charge: Fn(&T) -> usize,
    {
        let BodyLoad {
            scope,
            reference,
            type_name,
            expected_type,
            decode,
            retained_bytes,
            retain_in_typed_cache,
        } = load;
        DecodeContext::new(reference.qualified_data_id(), type_name)
            .ensure_resource_type(expected_type)?;
        let key = reference.resource_key();
        let typed_key = TypedCacheKey::of::<T>(key);
        loop {
            if let Some(scope) = scope {
                if let Some(cached) = scope.get::<T>(key) {
                    return Ok(Some(LoadedResource::Uncached(cached)));
                }
            } else if retain_in_typed_cache {
                if let Some(cached) = self.typed_cache.get::<T>(key) {
                    return Ok(Some(LoadedResource::Cached(cached)));
                }
            }

            match self.decode_flights.join(typed_key, type_name) {
                DecodeFlightJoin::Completed(Some(flight_value)) => {
                    let Some(decoded) = downcast_flight_value::<T>(&flight_value.value) else {
                        continue;
                    };
                    let decoded = self.retain_loaded_body(
                        scope,
                        key,
                        decoded,
                        flight_value.retained_bytes,
                        retain_in_typed_cache,
                    );
                    return Ok(Some(decoded));
                }
                DecodeFlightJoin::Completed(None) => return Ok(None),
                DecodeFlightJoin::Retry => continue,
                DecodeFlightJoin::Loader(flight) => {
                    let decoded = self.decode_body_from_record(
                        key,
                        type_name,
                        expected_type,
                        &decode,
                        &retained_bytes,
                        decode_attribution(scope, retain_in_typed_cache),
                    );
                    match decoded {
                        Ok(Some((decoded, retained_bytes))) => {
                            let decoded = self.retain_loaded_body(
                                scope,
                                key,
                                decoded,
                                retained_bytes,
                                retain_in_typed_cache,
                            );
                            let flight_value = match &decoded {
                                LoadedResource::Cached(value) => FlightValue {
                                    value: value.value_arc(),
                                    retained_bytes,
                                    _cache_pin: Some(value.pin()),
                                },
                                LoadedResource::Uncached(value) => FlightValue {
                                    value: value.clone(),
                                    retained_bytes,
                                    _cache_pin: None,
                                },
                            };
                            self.decode_flights
                                .complete(typed_key, &flight, Some(flight_value));
                            return Ok(Some(decoded));
                        }
                        Ok(None) => {
                            self.decode_flights.complete(typed_key, &flight, None);
                            return Ok(None);
                        }
                        Err(error) => {
                            self.decode_failures.record(key, type_name, &error);
                            self.decode_flights.fail(typed_key, &flight);
                            return Err(error);
                        }
                    }
                }
            }
        }
    }

    fn decode_body_from_record<T, Decode, Charge>(
        &self,
        key: ResourceKey,
        type_name: &'static str,
        expected_type: ResourceTypeId,
        decode: &Decode,
        retained_bytes: &Charge,
        attribution: DatReadAttribution,
    ) -> ResourceResult<Option<(Arc<T>, usize)>>
    where
        T: Send + Sync + 'static,
        Decode: Fn(ResourceDbObjDecodeContext, ResourceDbObjBody<'_>) -> ResourceResult<T>,
        Charge: Fn(&T) -> usize,
    {
        let Some(input) =
            self.read_record_for_decode(key.file_kind(), key.data_id(), attribution)?
        else {
            return Ok(None);
        };
        let (context, body) = decode_resource_body(
            key.qualified_data_id(),
            type_name,
            expected_type,
            input.entry().version,
            input.as_slice(),
        )?;
        let decoded = decode(context, body)?;
        let retained_bytes = retained_bytes(&decoded);
        drop(input);
        Ok(Some((Arc::new(decoded), retained_bytes)))
    }

    fn retain_loaded_body<T>(
        &self,
        scope: Option<&ResourceDecodeScope>,
        key: ResourceKey,
        decoded: Arc<T>,
        retained_bytes: usize,
        retain_in_typed_cache: bool,
    ) -> LoadedResource<T>
    where
        T: Send + Sync + 'static,
    {
        if let Some(scope) = scope {
            LoadedResource::Uncached(scope.insert(key, decoded, retained_bytes))
        } else if retain_in_typed_cache {
            LoadedResource::Cached(self.typed_cache.insert_with_retained_bytes(
                key,
                decoded,
                retained_bytes,
            ))
        } else {
            LoadedResource::Uncached(decoded)
        }
    }

    /// Loads a standalone packed value through a destination-owned decoder.
    ///
    /// This intentionally does not expose a DBObj header contract. The destination receives only
    /// the validated typed identity, mounted record version, and decompressed packed bytes.
    pub fn load_cached_packed<T>(
        &self,
        reference: TypedResourceRef,
    ) -> ResourceResult<Option<CachedResource<T>>>
    where
        T: ResourcePackedDecode,
    {
        DecodeContext::new(reference.qualified_data_id(), T::TYPE_NAME)
            .ensure_resource_type(T::RESOURCE_TYPE)?;
        let key = reference.resource_key();
        if let Some(cached) = self.typed_cache.get::<T>(key) {
            return Ok(Some(cached));
        }
        let Some(input) = self.read_record_for_decode(
            key.file_kind(),
            key.data_id(),
            DatReadAttribution::GenericResource,
        )?
        else {
            return Ok(None);
        };
        let (qdid, pack_version, bytes) = decode_resource_packed_body(
            key.qualified_data_id(),
            T::TYPE_NAME,
            T::RESOURCE_TYPE,
            input.entry().version,
            input.as_slice(),
        )?;
        let decoded = T::decode_packed(qdid, pack_version, bytes)?;
        let retained_bytes = decoded.cache_retained_bytes();
        drop(input);
        Ok(Some(self.typed_cache.insert_with_retained_bytes(
            key,
            Arc::new(decoded),
            retained_bytes,
        )))
    }

    pub(crate) fn typed_cache_handle(&self) -> TypedResourceCache {
        self.typed_cache.clone()
    }

    #[cfg(test)]
    pub(crate) fn evict_cached_typed<T>(&self, key: ResourceKey) -> bool
    where
        T: Send + Sync + 'static,
    {
        self.typed_cache.evict::<T>(key)
    }

    pub(crate) fn clear_typed_cache(&self) {
        self.typed_cache.clear();
    }

    pub(crate) fn invalidate_cached_resource(&self, file_kind: DatFileKind, did: DataId) -> usize {
        self.typed_cache.invalidate_resource(file_kind, did)
    }
}

fn decode_attribution(
    scope: Option<&ResourceDecodeScope>,
    retain_in_typed_cache: bool,
) -> DatReadAttribution {
    if scope.is_some() {
        DatReadAttribution::ActiveWorldScoped
    } else if retain_in_typed_cache {
        DatReadAttribution::GenericResource
    } else {
        DatReadAttribution::EphemeralWorldTooling
    }
}

fn downcast_flight_value<T>(value: &CacheValue) -> Option<Arc<T>>
where
    T: Send + Sync + 'static,
{
    Arc::downcast::<T>(Arc::clone(value)).ok()
}

fn expect_cached_resource<T>(value: LoadedResource<T>) -> CachedResource<T> {
    match value {
        LoadedResource::Cached(value) => value,
        LoadedResource::Uncached(_) => unreachable!("cached load returned an uncached value"),
    }
}

fn expect_uncached_resource<T>(value: LoadedResource<T>) -> Arc<T> {
    match value {
        LoadedResource::Uncached(value) => value,
        LoadedResource::Cached(_) => unreachable!("uncached load returned a cached value"),
    }
}

#[cfg(test)]
mod tests;
