use std::collections::BTreeMap;

/// Retained decoded-resource owner observed by a memory capture.
///
/// This describes where a decoded value is retained, not which consumer originally requested it.
/// Ephemeral loads deliberately have no entry because the store does not retain them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceOwnershipLane {
    GenericTypedCache,
    DecodeScope,
}

/// One raw retail resource-type bucket within a retained owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceOwnershipTypeStats {
    resource_type_id: u32,
    entry_count: usize,
    retained_bytes: usize,
}

impl ResourceOwnershipTypeStats {
    pub const fn resource_type_id(self) -> u32 {
        self.resource_type_id
    }

    pub const fn entry_count(self) -> usize {
        self.entry_count
    }

    pub const fn retained_bytes(self) -> usize {
        self.retained_bytes
    }
}

/// Read-only retained-byte observation grouped by the resource owner and retail type id.
///
/// The byte charge is supplied by the destination decoder. It includes allocations directly
/// owned by that decoded value, but not another `Arc` owner's allocation, transient packed DAT
/// bytes, or graphics-driver memory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceOwnershipStats {
    lane: ResourceOwnershipLane,
    entry_count: usize,
    retained_bytes: usize,
    by_resource_type: Vec<ResourceOwnershipTypeStats>,
}

impl ResourceOwnershipStats {
    pub const fn lane(&self) -> ResourceOwnershipLane {
        self.lane
    }

    pub const fn entry_count(&self) -> usize {
        self.entry_count
    }

    pub const fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }

    pub fn by_resource_type(&self) -> &[ResourceOwnershipTypeStats] {
        &self.by_resource_type
    }

    pub(crate) fn from_entries(
        lane: ResourceOwnershipLane,
        entries: impl IntoIterator<Item = (u32, usize)>,
    ) -> Self {
        let mut grouped = BTreeMap::<u32, (usize, usize)>::new();
        for (resource_type_id, retained_bytes) in entries {
            let entry = grouped.entry(resource_type_id).or_default();
            entry.0 = entry.0.saturating_add(1);
            entry.1 = entry.1.saturating_add(retained_bytes);
        }
        let by_resource_type = grouped
            .into_iter()
            .map(
                |(resource_type_id, (entry_count, retained_bytes))| ResourceOwnershipTypeStats {
                    resource_type_id,
                    entry_count,
                    retained_bytes,
                },
            )
            .collect::<Vec<_>>();
        let entry_count = by_resource_type.iter().map(|entry| entry.entry_count).sum();
        let retained_bytes = by_resource_type
            .iter()
            .map(|entry| entry.retained_bytes)
            .sum();
        Self {
            lane,
            entry_count,
            retained_bytes,
            by_resource_type,
        }
    }
}

/// One typed key retained by both the generic cache and a caller-owned decode scope.
///
/// The two owner charges are intentionally reported separately. A shared `Arc` does not imply a
/// second decoded allocation, while a distinct value is only a candidate for future policy work;
/// this observation never changes either owner's lifetime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceScopeOverlapTypeStats {
    resource_type_id: u32,
    entry_count: usize,
    shared_value_entry_count: usize,
    distinct_value_entry_count: usize,
    generic_retained_bytes: usize,
    scoped_retained_bytes: usize,
    distinct_scoped_retained_bytes: usize,
}

impl ResourceScopeOverlapTypeStats {
    pub const fn resource_type_id(self) -> u32 {
        self.resource_type_id
    }

    pub const fn entry_count(self) -> usize {
        self.entry_count
    }

    pub const fn shared_value_entry_count(self) -> usize {
        self.shared_value_entry_count
    }

    pub const fn distinct_value_entry_count(self) -> usize {
        self.distinct_value_entry_count
    }

    pub const fn generic_retained_bytes(self) -> usize {
        self.generic_retained_bytes
    }

    pub const fn scoped_retained_bytes(self) -> usize {
        self.scoped_retained_bytes
    }

    /// Logical scoped charge for the entries whose values do not share the generic cache `Arc`.
    pub const fn distinct_scoped_retained_bytes(self) -> usize {
        self.distinct_scoped_retained_bytes
    }
}

/// Non-blocking diagnostic comparison of one decode scope with generic typed-cache residency.
///
/// This is deliberately an ownership overlap report, not a physical-memory total: source values
/// may share an `Arc`, and decoder charges can include metadata beyond that allocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceScopeOverlapStats {
    entry_count: usize,
    shared_value_entry_count: usize,
    distinct_value_entry_count: usize,
    generic_retained_bytes: usize,
    scoped_retained_bytes: usize,
    distinct_scoped_retained_bytes: usize,
    by_resource_type: Vec<ResourceScopeOverlapTypeStats>,
}

impl ResourceScopeOverlapStats {
    pub const fn entry_count(&self) -> usize {
        self.entry_count
    }

    pub const fn shared_value_entry_count(&self) -> usize {
        self.shared_value_entry_count
    }

    pub const fn distinct_value_entry_count(&self) -> usize {
        self.distinct_value_entry_count
    }

    pub const fn generic_retained_bytes(&self) -> usize {
        self.generic_retained_bytes
    }

    pub const fn scoped_retained_bytes(&self) -> usize {
        self.scoped_retained_bytes
    }

    pub const fn distinct_scoped_retained_bytes(&self) -> usize {
        self.distinct_scoped_retained_bytes
    }

    pub fn by_resource_type(&self) -> &[ResourceScopeOverlapTypeStats] {
        &self.by_resource_type
    }

    pub(crate) fn from_entries(
        entries: impl IntoIterator<Item = (u32, usize, usize, bool)>,
    ) -> Self {
        let mut grouped = BTreeMap::<u32, ResourceScopeOverlapTypeStats>::new();
        for (resource_type_id, generic_bytes, scoped_bytes, shared_value) in entries {
            let entry = grouped
                .entry(resource_type_id)
                .or_insert(ResourceScopeOverlapTypeStats {
                    resource_type_id,
                    entry_count: 0,
                    shared_value_entry_count: 0,
                    distinct_value_entry_count: 0,
                    generic_retained_bytes: 0,
                    scoped_retained_bytes: 0,
                    distinct_scoped_retained_bytes: 0,
                });
            entry.entry_count = entry.entry_count.saturating_add(1);
            entry.shared_value_entry_count = entry
                .shared_value_entry_count
                .saturating_add(usize::from(shared_value));
            entry.distinct_value_entry_count = entry
                .distinct_value_entry_count
                .saturating_add(usize::from(!shared_value));
            entry.generic_retained_bytes =
                entry.generic_retained_bytes.saturating_add(generic_bytes);
            entry.scoped_retained_bytes = entry.scoped_retained_bytes.saturating_add(scoped_bytes);
            entry.distinct_scoped_retained_bytes = entry
                .distinct_scoped_retained_bytes
                .saturating_add((!shared_value).then_some(scoped_bytes).unwrap_or_default());
        }
        let by_resource_type = grouped.into_values().collect::<Vec<_>>();
        Self {
            entry_count: by_resource_type.iter().map(|entry| entry.entry_count).sum(),
            shared_value_entry_count: by_resource_type
                .iter()
                .map(|entry| entry.shared_value_entry_count)
                .sum(),
            distinct_value_entry_count: by_resource_type
                .iter()
                .map(|entry| entry.distinct_value_entry_count)
                .sum(),
            generic_retained_bytes: by_resource_type
                .iter()
                .map(|entry| entry.generic_retained_bytes)
                .sum(),
            scoped_retained_bytes: by_resource_type
                .iter()
                .map(|entry| entry.scoped_retained_bytes)
                .sum(),
            distinct_scoped_retained_bytes: by_resource_type
                .iter()
                .map(|entry| entry.distinct_scoped_retained_bytes)
                .sum(),
            by_resource_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ResourceOwnershipLane, ResourceOwnershipStats, ResourceScopeOverlapStats};

    #[test]
    fn groups_retained_entries_by_type_in_stable_order() {
        let stats = ResourceOwnershipStats::from_entries(
            ResourceOwnershipLane::DecodeScope,
            [(7, 16), (6, 32), (7, 8)],
        );

        assert_eq!(stats.lane(), ResourceOwnershipLane::DecodeScope);
        assert_eq!(stats.entry_count(), 3);
        assert_eq!(stats.retained_bytes(), 56);
        assert_eq!(stats.by_resource_type().len(), 2);
        assert_eq!(stats.by_resource_type()[0].resource_type_id(), 6);
        assert_eq!(stats.by_resource_type()[0].entry_count(), 1);
        assert_eq!(stats.by_resource_type()[1].resource_type_id(), 7);
        assert_eq!(stats.by_resource_type()[1].retained_bytes(), 24);
    }

    #[test]
    fn overlap_stats_keep_shared_and_distinct_owner_charges_separate() {
        let stats = ResourceScopeOverlapStats::from_entries([
            (7, 16, 16, true),
            (7, 8, 12, false),
            (6, 4, 4, true),
        ]);

        assert_eq!(stats.entry_count(), 3);
        assert_eq!(stats.shared_value_entry_count(), 2);
        assert_eq!(stats.distinct_value_entry_count(), 1);
        assert_eq!(stats.generic_retained_bytes(), 28);
        assert_eq!(stats.scoped_retained_bytes(), 32);
        assert_eq!(stats.distinct_scoped_retained_bytes(), 12);
        assert_eq!(stats.by_resource_type()[0].resource_type_id(), 6);
        assert_eq!(stats.by_resource_type()[1].distinct_value_entry_count(), 1);
    }
}
