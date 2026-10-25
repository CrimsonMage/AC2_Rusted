use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use crate::typed_cache::{CacheValue, ResourceCachePin, TypedCacheKey};

const DECODE_FLIGHT_REPORT_INTERVAL: Duration = Duration::from_secs(1);
const DECODE_FLIGHT_SLOW_THRESHOLD: Duration = Duration::from_millis(16);

#[derive(Debug)]
struct DecodeFlightDiagnostics {
    started_at: Instant,
    loader_thread: Option<String>,
    type_name: &'static str,
    waiters: u32,
    last_stall_report_at: Option<Instant>,
}

impl DecodeFlightDiagnostics {
    fn new(type_name: &'static str) -> Self {
        let loader_thread = log::log_enabled!(
            target: "ac2_db::decode_flight",
            log::Level::Debug
        )
        .then(|| {
            std::thread::current()
                .name()
                .unwrap_or("unnamed")
                .to_owned()
        });
        Self {
            started_at: Instant::now(),
            loader_thread,
            type_name,
            waiters: 0,
            last_stall_report_at: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct DecodeFlightTotals {
    loaders: u64,
    failed_loaders: u64,
    loader_elapsed_total: Duration,
    loader_elapsed_max: Duration,
    waits: u64,
    slow_waits: u64,
    wait_elapsed_total: Duration,
    wait_elapsed_max: Duration,
}

impl DecodeFlightTotals {
    fn is_anomalous(self) -> bool {
        self.failed_loaders != 0
            || self.waits != 0
            || self.slow_waits != 0
            || self.loader_elapsed_max >= DECODE_FLIGHT_SLOW_THRESHOLD
    }
}

#[derive(Debug)]
struct DecodeFlightTelemetry {
    report_started_at: Instant,
    totals: DecodeFlightTotals,
}

impl Default for DecodeFlightTelemetry {
    fn default() -> Self {
        Self {
            report_started_at: Instant::now(),
            totals: DecodeFlightTotals::default(),
        }
    }
}

#[derive(Debug)]
enum FlightState {
    Loading,
    Completed(Option<FlightValue>),
    Failed,
}

#[derive(Debug, Clone)]
pub(crate) struct FlightValue {
    pub(crate) value: CacheValue,
    pub(crate) retained_bytes: usize,
    pub(crate) _cache_pin: Option<ResourceCachePin>,
}

#[derive(Debug)]
pub(crate) struct DecodeFlight {
    state: Mutex<FlightState>,
    ready: Condvar,
    diagnostics: Mutex<DecodeFlightDiagnostics>,
}

impl DecodeFlight {
    fn new(type_name: &'static str) -> Self {
        Self {
            state: Mutex::new(FlightState::Loading),
            ready: Condvar::new(),
            diagnostics: Mutex::new(DecodeFlightDiagnostics::new(type_name)),
        }
    }

    fn begin_wait(&self) {
        if !log::log_enabled!(
            target: "ac2_db::decode_flight",
            log::Level::Debug
        ) {
            return;
        }
        let mut diagnostics = self
            .diagnostics
            .lock()
            .expect("decode flight diagnostics poisoned");
        diagnostics.waiters = diagnostics.waiters.saturating_add(1);
    }

    fn finish_wait(&self) {
        if !log::log_enabled!(
            target: "ac2_db::decode_flight",
            log::Level::Debug
        ) {
            return;
        }
        let mut diagnostics = self
            .diagnostics
            .lock()
            .expect("decode flight diagnostics poisoned");
        diagnostics.waiters = diagnostics.waiters.saturating_sub(1);
    }

    fn elapsed(&self) -> Duration {
        self.diagnostics
            .lock()
            .expect("decode flight diagnostics poisoned")
            .started_at
            .elapsed()
    }

    fn log_stall_if_due(&self, key: TypedCacheKey) {
        if !log::log_enabled!(
            target: "ac2_db::decode_flight",
            log::Level::Debug
        ) {
            return;
        }
        let now = Instant::now();
        let report = {
            let mut diagnostics = self
                .diagnostics
                .lock()
                .expect("decode flight diagnostics poisoned");
            let elapsed = now.saturating_duration_since(diagnostics.started_at);
            if elapsed < DECODE_FLIGHT_REPORT_INTERVAL
                || diagnostics.last_stall_report_at.is_some_and(|last| {
                    now.saturating_duration_since(last) < DECODE_FLIGHT_REPORT_INTERVAL
                })
            {
                None
            } else {
                diagnostics.last_stall_report_at = Some(now);
                Some((
                    elapsed,
                    diagnostics.waiters,
                    diagnostics.type_name,
                    diagnostics.loader_thread.clone(),
                ))
            }
        };
        let Some((elapsed, waiters, type_name, loader_thread)) = report else {
            return;
        };
        let resource = key.resource_key();
        log::debug!(target: "ac2_db::decode_flight",
            "decode_flight_stalled: file={:?} did=0x{:08X} db_type=0x{:08X} decoder={} loader_thread={} age_ms={:.2} waiters={}",
            resource.file_kind(),
            resource.data_id().raw(),
            resource.type_id(),
            type_name,
            loader_thread.as_deref().unwrap_or("diagnostics-disabled-at-start"),
            elapsed.as_secs_f64() * 1_000.0,
            waiters,
        );
    }
}

#[derive(Debug)]
pub(crate) enum DecodeFlightJoin {
    Loader(Arc<DecodeFlight>),
    Completed(Option<FlightValue>),
    Retry,
}

/// A short-lived, keyed decode rendezvous. Completed values live only until all joined callers
/// wake; normal cache or caller scope ownership decides whether they remain resident afterward.
#[derive(Debug)]
pub(crate) struct DecodeFlights {
    flights: Mutex<HashMap<TypedCacheKey, Arc<DecodeFlight>>>,
    telemetry: Mutex<DecodeFlightTelemetry>,
}

impl Default for DecodeFlights {
    fn default() -> Self {
        Self {
            flights: Mutex::new(HashMap::new()),
            telemetry: Mutex::new(DecodeFlightTelemetry::default()),
        }
    }
}

impl DecodeFlights {
    pub(crate) fn join(&self, key: TypedCacheKey, type_name: &'static str) -> DecodeFlightJoin {
        let (flight, is_loader) = {
            let mut flights = self.flights.lock().expect("decode flights poisoned");
            match flights.get(&key) {
                Some(flight) => (Arc::clone(flight), false),
                None => {
                    let flight = Arc::new(DecodeFlight::new(type_name));
                    flights.insert(key, Arc::clone(&flight));
                    (flight, true)
                }
            }
        };
        if is_loader {
            return DecodeFlightJoin::Loader(flight);
        }

        let diagnostics_enabled = log::log_enabled!(
            target: "ac2_db::decode_flight",
            log::Level::Debug
        );
        let wait_started_at = diagnostics_enabled.then(Instant::now);
        if diagnostics_enabled {
            flight.begin_wait();
        }
        let mut state = flight.state.lock().expect("decode flight poisoned");
        while matches!(*state, FlightState::Loading) {
            if diagnostics_enabled {
                let (next_state, timeout) = flight
                    .ready
                    .wait_timeout(state, DECODE_FLIGHT_REPORT_INTERVAL)
                    .expect("decode flight poisoned");
                state = next_state;
                if timeout.timed_out() && matches!(*state, FlightState::Loading) {
                    flight.log_stall_if_due(key);
                }
            } else {
                state = flight.ready.wait(state).expect("decode flight poisoned");
            }
        }
        let result = match &*state {
            FlightState::Completed(value) => DecodeFlightJoin::Completed(value.clone()),
            FlightState::Failed => DecodeFlightJoin::Retry,
            FlightState::Loading => unreachable!("decode flight wait completed while loading"),
        };
        drop(state);
        if let Some(wait_started_at) = wait_started_at {
            flight.finish_wait();
            self.record_wait(wait_started_at.elapsed());
        }
        result
    }

    pub(crate) fn complete(
        &self,
        key: TypedCacheKey,
        flight: &Arc<DecodeFlight>,
        result: Option<FlightValue>,
    ) {
        self.record_loader_finished(flight.elapsed(), false);
        *flight.state.lock().expect("decode flight poisoned") = FlightState::Completed(result);
        flight.ready.notify_all();
        self.remove_if_current(key, flight);
    }

    pub(crate) fn fail(&self, key: TypedCacheKey, flight: &Arc<DecodeFlight>) {
        self.record_loader_finished(flight.elapsed(), true);
        *flight.state.lock().expect("decode flight poisoned") = FlightState::Failed;
        flight.ready.notify_all();
        self.remove_if_current(key, flight);
    }

    fn remove_if_current(&self, key: TypedCacheKey, flight: &Arc<DecodeFlight>) {
        let mut flights = self.flights.lock().expect("decode flights poisoned");
        if flights
            .get(&key)
            .is_some_and(|current| Arc::ptr_eq(current, flight))
        {
            flights.remove(&key);
        }
    }

    fn record_wait(&self, elapsed: Duration) {
        if !log::log_enabled!(
            target: "ac2_db::decode_flight",
            log::Level::Debug
        ) {
            return;
        }
        let report = {
            let mut telemetry = self
                .telemetry
                .lock()
                .expect("decode flight telemetry poisoned");
            telemetry.totals.waits = telemetry.totals.waits.saturating_add(1);
            telemetry.totals.slow_waits += u64::from(elapsed >= DECODE_FLIGHT_SLOW_THRESHOLD);
            telemetry.totals.wait_elapsed_total += elapsed;
            telemetry.totals.wait_elapsed_max = telemetry.totals.wait_elapsed_max.max(elapsed);
            take_telemetry_report_if_due(&mut telemetry)
        };
        log_telemetry_report(report);
    }

    fn record_loader_finished(&self, elapsed: Duration, failed: bool) {
        if !log::log_enabled!(
            target: "ac2_db::decode_flight",
            log::Level::Debug
        ) {
            return;
        }
        let report = {
            let mut telemetry = self
                .telemetry
                .lock()
                .expect("decode flight telemetry poisoned");
            telemetry.totals.loaders = telemetry.totals.loaders.saturating_add(1);
            telemetry.totals.failed_loaders += u64::from(failed);
            telemetry.totals.loader_elapsed_total += elapsed;
            telemetry.totals.loader_elapsed_max = telemetry.totals.loader_elapsed_max.max(elapsed);
            take_telemetry_report_if_due(&mut telemetry)
        };
        log_telemetry_report(report);
    }
}

fn take_telemetry_report_if_due(
    telemetry: &mut DecodeFlightTelemetry,
) -> Option<DecodeFlightTotals> {
    if telemetry.report_started_at.elapsed() < DECODE_FLIGHT_REPORT_INTERVAL {
        return None;
    }
    let totals = telemetry.totals;
    *telemetry = DecodeFlightTelemetry::default();
    (totals.loaders != 0 || totals.waits != 0).then_some(totals)
}

fn log_telemetry_report(totals: Option<DecodeFlightTotals>) {
    let Some(totals) = totals else {
        return;
    };
    if !totals.is_anomalous() {
        return;
    }
    let loader_average = if totals.loaders != 0 {
        totals.loader_elapsed_total / (totals.loaders.min(u64::from(u32::MAX)) as u32)
    } else {
        Default::default()
    };
    let wait_average = if totals.waits != 0 {
        totals.wait_elapsed_total / (totals.waits.min(u64::from(u32::MAX)) as u32)
    } else {
        Default::default()
    };
    log::debug!(target: "ac2_db::decode_flight",
        "decode_flight_timing: loaders={} failed_loaders={} loader_avg_ms={:.2} loader_max_ms={:.2} waits={} slow_waits={} wait_avg_ms={:.2} wait_max_ms={:.2}",
        totals.loaders,
        totals.failed_loaders,
        loader_average.as_secs_f64() * 1_000.0,
        totals.loader_elapsed_max.as_secs_f64() * 1_000.0,
        totals.waits,
        totals.slow_waits,
        wait_average.as_secs_f64() * 1_000.0,
        totals.wait_elapsed_max.as_secs_f64() * 1_000.0,
    );
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::DecodeFlightTotals;

    #[test]
    fn ordinary_fast_loader_volume_is_not_anomalous() {
        assert!(!DecodeFlightTotals {
            loaders: 10_000,
            loader_elapsed_max: Duration::from_millis(2),
            ..Default::default()
        }
        .is_anomalous());
        assert!(DecodeFlightTotals {
            loaders: 1,
            failed_loaders: 1,
            ..Default::default()
        }
        .is_anomalous());
    }
}
