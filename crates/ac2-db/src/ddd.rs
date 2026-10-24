//! Generic resource-cache invalidation vocabulary for data delivery systems.

use crate::ResourceKey;

/// A typed resource identity whose cached representations must be invalidated.
///
/// Protocol and product-specific delivery systems construct this value at their
/// own boundary; resources only applies the generic cache operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceDddInvalidation {
    key: ResourceKey,
}

impl ResourceDddInvalidation {
    pub const fn new(key: ResourceKey) -> Self {
        Self { key }
    }

    pub const fn key(self) -> ResourceKey {
        self.key
    }
}

/// Result of applying a generic resource-cache invalidation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResourceDddApplyOutcome {
    invalidated_typed_entries: usize,
    pub invalidated: Vec<ResourceKey>,
}

impl ResourceDddApplyOutcome {
    pub(crate) fn new(invalidated_typed_entries: usize, key: ResourceKey) -> Self {
        Self {
            invalidated_typed_entries,
            invalidated: vec![key],
        }
    }

    pub fn invalidated_typed_entries(&self) -> usize {
        self.invalidated_typed_entries
    }
}
