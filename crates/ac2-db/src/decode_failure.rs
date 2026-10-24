use std::collections::HashMap;
use std::sync::Mutex;

use crate::{ResourceError, ResourceKey};

const DECODE_FAILURE_TARGET: &str = "ac2_db::decode_failure";
const MAX_UNIQUE_FAILURES: usize = 1_024;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct DecodeFailureSignature {
    resource: ResourceKey,
    decoder: &'static str,
    error_kind: &'static str,
    error: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DecodeFailureObservation {
    First,
    Repeat(u64),
    SuppressedRepeat,
    Overflow(u64),
    SuppressedOverflow,
}

#[derive(Debug, Default)]
struct DecodeFailureState {
    occurrences: HashMap<DecodeFailureSignature, u64>,
    overflowed_unique: u64,
}

impl DecodeFailureState {
    fn observe(&mut self, signature: DecodeFailureSignature) -> DecodeFailureObservation {
        if let Some(occurrences) = self.occurrences.get_mut(&signature) {
            *occurrences = occurrences.saturating_add(1);
            return if occurrences.is_power_of_two() {
                DecodeFailureObservation::Repeat(*occurrences)
            } else {
                DecodeFailureObservation::SuppressedRepeat
            };
        }
        if self.occurrences.len() >= MAX_UNIQUE_FAILURES {
            self.overflowed_unique = self.overflowed_unique.saturating_add(1);
            return if self.overflowed_unique.is_power_of_two() {
                DecodeFailureObservation::Overflow(self.overflowed_unique)
            } else {
                DecodeFailureObservation::SuppressedOverflow
            };
        }
        self.occurrences.insert(signature, 1);
        DecodeFailureObservation::First
    }
}

/// Opt-in, bounded detail for typed loader errors. The normal decode-flight counter remains the
/// cheap aggregate; this state is not touched unless the client explicitly enables its target.
#[derive(Debug, Default)]
pub(crate) struct DecodeFailureDiagnostics {
    state: Mutex<DecodeFailureState>,
}

impl DecodeFailureDiagnostics {
    pub(crate) fn record(
        &self,
        resource: ResourceKey,
        decoder: &'static str,
        error: &ResourceError,
    ) {
        if !log::log_enabled!(target: DECODE_FAILURE_TARGET, log::Level::Debug) {
            return;
        }
        let signature = DecodeFailureSignature {
            resource,
            decoder,
            error_kind: error.diagnostic_kind(),
            error: error.to_string(),
        };
        let observation = self
            .state
            .lock()
            .expect("decode failure diagnostics poisoned")
            .observe(signature.clone());
        match observation {
            DecodeFailureObservation::First => {
                log::debug!(target: DECODE_FAILURE_TARGET,
                    "decode_failure: file={:?} did=0x{:08X} db_type=0x{:08X} decoder={} error_kind={} error={:?}",
                    resource.file_kind(),
                    resource.data_id().raw(),
                    resource.type_id(),
                    decoder,
                    signature.error_kind,
                    signature.error,
                );
            }
            DecodeFailureObservation::Repeat(occurrences) => {
                log::debug!(target: DECODE_FAILURE_TARGET,
                    "decode_failure_repeat: file={:?} did=0x{:08X} db_type=0x{:08X} decoder={} error_kind={} occurrences={}",
                    resource.file_kind(),
                    resource.data_id().raw(),
                    resource.type_id(),
                    decoder,
                    signature.error_kind,
                    occurrences,
                );
            }
            DecodeFailureObservation::Overflow(overflowed_unique) => {
                log::debug!(target: DECODE_FAILURE_TARGET,
                    "decode_failure_overflow: retained_unique={} overflowed_unique={}",
                    MAX_UNIQUE_FAILURES,
                    overflowed_unique,
                );
            }
            DecodeFailureObservation::SuppressedRepeat
            | DecodeFailureObservation::SuppressedOverflow => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use ac2_core::{DataId, QualifiedDataId};
    use ac2_dat::DatFileKind;

    use super::{
        DecodeFailureObservation, DecodeFailureSignature, DecodeFailureState, MAX_UNIQUE_FAILURES,
    };
    use crate::ResourceKey;

    fn signature(did: u32) -> DecodeFailureSignature {
        DecodeFailureSignature {
            resource: ResourceKey::new(
                DatFileKind::Portal,
                QualifiedDataId::new(DataId::new(did), 7),
            ),
            decoder: "Example",
            error_kind: "invalid_data",
            error: "bad body".to_string(),
        }
    }

    #[test]
    fn reports_first_and_power_of_two_repeats() {
        let mut state = DecodeFailureState::default();
        assert_eq!(state.observe(signature(1)), DecodeFailureObservation::First);
        assert_eq!(
            state.observe(signature(1)),
            DecodeFailureObservation::Repeat(2)
        );
        assert_eq!(
            state.observe(signature(1)),
            DecodeFailureObservation::SuppressedRepeat
        );
        assert_eq!(
            state.observe(signature(1)),
            DecodeFailureObservation::Repeat(4)
        );
    }

    #[test]
    fn bounds_unique_failure_memory_and_reports_overflow_powers() {
        let mut state = DecodeFailureState::default();
        for did in 0..MAX_UNIQUE_FAILURES as u32 {
            assert_eq!(
                state.observe(signature(did)),
                DecodeFailureObservation::First
            );
        }
        assert_eq!(
            state.observe(signature(MAX_UNIQUE_FAILURES as u32)),
            DecodeFailureObservation::Overflow(1)
        );
        assert_eq!(
            state.observe(signature(MAX_UNIQUE_FAILURES as u32 + 1)),
            DecodeFailureObservation::Overflow(2)
        );
        assert_eq!(
            state.observe(signature(MAX_UNIQUE_FAILURES as u32 + 2)),
            DecodeFailureObservation::SuppressedOverflow
        );
    }
}
