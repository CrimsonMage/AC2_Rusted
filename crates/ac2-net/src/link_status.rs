//! `CLinkStatusAverages` and `CLinkStatusSnapshot` (`portal/client/src/clinet/linkstatus.cpp`):
//! the link statistics the client keeps for the net indicator. The client counts packets and
//! bytes into the current snapshot (`ClientNet::AddToPerfCounter`); each world heartbeat
//! (`ClientNet::ProcessConnection`, every two seconds) closes that snapshot into the averages
//! and hands them to the link-status plugins (`ClientNet::NotifyPluginsOfHeartbeat`).

/// `CLinkStatusSnapshot`: one heartbeat interval's counters, and the receiver's state at its end.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct LinkSnapshot {
    /// `RoundTripDelay`: the world receiver's latest echo round trip, in seconds.
    pub round_trip: f32,
    /// `nPktsSent` (`PacketsSentCounter`).
    pub packets_sent: u16,
    /// `nPktsRetransmitted` (`RetransmitsSentCounter`).
    pub packets_retransmitted: u16,
    /// `nPktsReceived` (the optional-header, data and mixed packet counters).
    pub packets_received: u16,
    /// `nPktsNAKed` (`AcksReceivedCounter`). Legions 7160 never calls that counter (nor
    /// `NaksReceivedCounter`), so this stays 0 there.
    pub packets_naked: u16,
    /// `nBytesSent` (`BytesSentCounter`).
    pub bytes_sent: u32,
    /// `nBytesReceived` (`BytesReceivedCounter`).
    pub bytes_received: u32,
    /// `TimeSinceLastGotData`: seconds since the receiver last got data.
    pub since_last_data: f32,
    /// `SnapshotDuration`: seconds since the previous heartbeat.
    pub duration: f32,
}

/// `CLinkStatusAverages::CAverager<T, N>`: the last `N` samples and their running total, kept in
/// double precision (`m_CurTotal`). Once full, a new sample replaces the oldest.
#[derive(Debug, Clone, PartialEq)]
pub struct Averager<T, const N: usize> {
    samples: [T; N],
    count: usize,
    first: usize,
    total: f64,
}

impl<T, const N: usize> Default for Averager<T, N>
where
    T: Copy + Default,
{
    fn default() -> Self {
        Self {
            samples: [T::default(); N],
            count: 0,
            first: 0,
            total: 0.0,
        }
    }
}

impl<T, const N: usize> Averager<T, N>
where
    T: Copy + Into<f64>,
{
    /// `CAverager<T, N>::AddSample`.
    pub fn add(&mut self, sample: T) {
        if self.count == N {
            self.total -= self.samples[self.first].into();
            self.samples[self.first] = sample;
            self.first = (self.first + 1) % N;
        } else {
            self.samples[self.count] = sample;
            self.count += 1;
        }
        self.total += sample.into();
    }

    /// The running total of the samples held.
    pub fn total(&self) -> f64 {
        self.total
    }
}

/// `CLinkStatusAverages::CAverager<float,4>`: round trips. Its total is kept as the retail code
/// keeps it: each step rounds the running total to `float` before adding a `float` sample.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RoundTripAverager {
    samples: [f32; 4],
    count: usize,
    first: usize,
    total: f64,
}

impl RoundTripAverager {
    /// `CLinkStatusAverages::CAverager<float,4>::AddSample`.
    pub fn add(&mut self, sample: f32) {
        if self.count != 4 {
            self.samples[self.count] = sample;
            self.count += 1;
            self.total = f64::from(sample) + f64::from(self.total as f32);
            return;
        }
        self.total = f64::from(self.total as f32) - f64::from(self.samples[self.first]);
        self.samples[self.first] = sample;
        self.first = (self.first + 1) & 3;
        self.total = f64::from(sample) + f64::from(self.total as f32);
    }

    /// The number of samples held.
    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

/// `CLinkStatusAverages`: the averages over the last heartbeats (`m_RoundTripDelays`,
/// `m_nPktsSent` and the rest, `m_TimeDiffs`) and the last snapshot (`m_Snapshot`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LinkAverages {
    round_trips: RoundTripAverager,
    packets_sent: Averager<u16, 40>,
    packets_retransmitted: Averager<u16, 40>,
    packets_received: Averager<u16, 40>,
    packets_naked: Averager<u16, 40>,
    bytes_sent: Averager<u32, 2>,
    bytes_received: Averager<u32, 2>,
    time_diffs: Averager<f64, 2>,
    /// `m_LocalTimeOfSnapshot`: the local time of the last heartbeat (0 before the first).
    local_time_of_snapshot: f64,
    /// `m_Snapshot`: the last heartbeat's snapshot.
    pub snapshot: LinkSnapshot,
}

impl LinkAverages {
    /// `CLinkStatusAverages::AddSnapshot`: copy the snapshot, then average it. A snapshot with
    /// no positive duration (or the first one) counts as one second.
    pub fn add_snapshot(&mut self, snap: &LinkSnapshot, local_time: f64) {
        let sample_duration = if self.local_time_of_snapshot == 0.0 || snap.duration <= 0.0 {
            1.0
        } else {
            f64::from(snap.duration)
        };
        self.snapshot = *snap;
        self.local_time_of_snapshot = local_time;
        self.packets_sent.add(snap.packets_sent);
        self.packets_retransmitted.add(snap.packets_retransmitted);
        self.packets_received.add(snap.packets_received);
        self.packets_naked.add(snap.packets_naked);
        self.bytes_sent.add(snap.bytes_sent);
        self.bytes_received.add(snap.bytes_received);
        self.time_diffs.add(sample_duration);
    }

    /// `CLinkStatusAverages::OnPingResponse`.
    pub fn on_ping_response(&mut self, round_trip: f32) {
        self.round_trips.add(round_trip);
    }

    /// The mean of the round trips held, in seconds (0 with none; `gmNetStatusUI::OnHeartbeat`).
    pub fn mean_round_trip(&self) -> f64 {
        if self.round_trips.is_empty() {
            return 0.0;
        }
        let total = self.round_trips.total;
        total / self.round_trips.len() as f64
    }

    /// `CLinkStatusAverages::GetAveragePacketLoss`: (NAKed plus retransmitted) over (received
    /// plus sent), doubled as retail returns it.
    pub fn packet_loss(&self) -> f64 {
        let denominator = self.packets_received.total() + self.packets_sent.total();
        if denominator <= 0.0 {
            return 0.0;
        }
        let ratio = (self.packets_naked.total() + self.packets_retransmitted.total()) / denominator;
        ratio + ratio
    }

    /// `CLinkStatusAverages::GetAverageBytesPerSecondReceived`.
    pub fn bytes_received_per_second(&self) -> f64 {
        let seconds = self.time_diffs.total();
        if seconds <= 0.0 {
            return 0.0;
        }
        self.bytes_received.total() / seconds
    }

    /// `m_LocalTimeOfSnapshot`.
    pub fn local_time_of_snapshot(&self) -> f64 {
        self.local_time_of_snapshot
    }
}

#[cfg(test)]
mod tests {
    use super::{LinkAverages, LinkSnapshot, RoundTripAverager};

    fn snap(duration: f32, received: u32, sent: u32) -> LinkSnapshot {
        LinkSnapshot {
            duration,
            bytes_received: received,
            bytes_sent: sent,
            ..Default::default()
        }
    }

    #[test]
    fn round_trips_average_the_last_four_pings() {
        let mut rtt = RoundTripAverager::default();
        for v in [0.1f32, 0.2, 0.3, 0.4] {
            rtt.add(v);
        }
        assert_eq!(rtt.len(), 4);
        let mut links = LinkAverages::default();
        assert_eq!(links.mean_round_trip(), 0.0);
        for v in [0.1f32, 0.2, 0.3, 0.4, 0.5] {
            links.on_ping_response(v);
        }
        // The window holds 0.2..=0.5: the mean is 0.35 (to float rounding of the total).
        assert!((links.mean_round_trip() - 0.35).abs() < 1e-6);
    }

    #[test]
    fn the_first_snapshot_counts_as_one_second_and_later_ones_by_duration() {
        let mut links = LinkAverages::default();
        links.add_snapshot(&snap(2.0, 1000, 500), 10.0);
        // First heartbeat: the time sample is 1 second, so 1000 bytes is 1000 per second.
        assert_eq!(links.bytes_received_per_second(), 1000.0);
        links.add_snapshot(&snap(2.0, 3000, 0), 12.0);
        // (1000 + 3000) bytes over (1 + 2) seconds.
        assert!((links.bytes_received_per_second() - 4000.0 / 3.0).abs() < 1e-9);
        assert_eq!(links.snapshot.bytes_received, 3000);
        assert_eq!(links.local_time_of_snapshot(), 12.0);
    }

    #[test]
    fn a_non_positive_duration_counts_as_one_second() {
        let mut links = LinkAverages::default();
        links.add_snapshot(&snap(2.0, 100, 0), 5.0);
        links.add_snapshot(&snap(0.0, 100, 0), 6.0);
        assert_eq!(links.bytes_received_per_second(), 100.0);
    }

    #[test]
    fn packet_loss_is_twice_the_naked_and_retransmitted_share() {
        let mut links = LinkAverages::default();
        assert_eq!(links.packet_loss(), 0.0);
        let mut s = snap(2.0, 0, 0);
        s.packets_received = 30;
        s.packets_sent = 70;
        s.packets_retransmitted = 5;
        s.packets_naked = 0;
        links.add_snapshot(&s, 1.0);
        // 5 / 100, doubled.
        assert!((links.packet_loss() - 0.1).abs() < 1e-12);
    }
}
