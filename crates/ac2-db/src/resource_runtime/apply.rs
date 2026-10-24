use crate::{ResourceDddApplyOutcome, ResourceDddInvalidation};

use super::ResourceRuntime;

impl ResourceRuntime {
    /// Invalidates all retained typed representations for a delivered resource.
    pub fn apply_ddd_invalidation(
        &self,
        invalidation: ResourceDddInvalidation,
    ) -> ResourceDddApplyOutcome {
        let key = invalidation.key();
        let invalidated_typed_entries = self
            .store
            .invalidate_cached_resource(key.file_kind(), key.data_id());
        ResourceDddApplyOutcome::new(invalidated_typed_entries, key)
    }
}
