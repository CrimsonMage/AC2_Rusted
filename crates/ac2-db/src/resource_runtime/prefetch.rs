use crate::{
    ResourceBodyDecode, ResourceDecodeScope, ResourceKey, ResourceResult, ResourceStore,
    TypedResourceRef,
};

/// Executes a worker-owned domain body warmup without exposing raw resource bytes to that
/// domain. The reference retains the resource type identity used by the body decoder.
pub(super) fn prefetch_body<T>(store: &ResourceStore, key: ResourceKey) -> ResourceResult<bool>
where
    T: ResourceBodyDecode,
{
    let reference =
        TypedResourceRef::from_resource_type(key.file_kind(), key.data_id(), T::RESOURCE_TYPE);
    Ok(store.load_cached_body::<T>(reference)?.is_some())
}

/// Executes a worker-owned body warmup into a caller-owned active decode scope. This must not
/// consult or populate the session-wide typed cache.
pub(super) fn prefetch_scoped_body<T>(
    store: &ResourceStore,
    scope: &ResourceDecodeScope,
    key: ResourceKey,
) -> ResourceResult<bool>
where
    T: ResourceBodyDecode,
{
    let reference =
        TypedResourceRef::from_resource_type(key.file_kind(), key.data_id(), T::RESOURCE_TYPE);
    Ok(store.load_scoped_body::<T>(scope, reference)?.is_some())
}
