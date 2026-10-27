//! `gmNetStatusUI`: the status bar's net indicator (`NetStatusField`). Each link heartbeat
//! (`gmNetStatusUI::OnHeartbeat`, fed by `CLinkStatusAverages`) sets its tooltip (the latency,
//! the packet loss and the bytes per second in each direction, `ID_Indicator_TT_NetStatus` and
//! `ID_Indicator_TT_NetStatusBW`), pushes one column of bytes sent and received into the graph
//! history, and moves the field to the state for the time since the last data arrived
//! (`stateless`, `good`, `bad` or `ugly`). The thresholds are the field's `good`, `bad` and `ugly`
//! float attributes.
//!
//! The session emits each world heartbeat (`SessionEvent::LinkHeartbeat`, the `ac2_net`
//! averages). The status bar host turns it into a [`LinkSample`] with
//! [`LinkSample::from_link`] and calls [`NetStatusUi::on_heartbeat`]; no client host exists yet.
//! Not modelled yet: the bandwidth graph's drawing (`gmNetStatusUI::DrawBandwidthGraph` blits the
//! history into a render surface) and the latency-graph option (`RecvNotice_ReloadOptions`).

use ac2_net::LinkAverages;

use crate::element::UiTree;
use crate::fellow_info::FellowStrings;
use crate::layout::AttributeValue;
use crate::skill_tree::SkillVar;
use crate::token::tokenize;

fn id(name: &str) -> u32 {
    tokenize(name)
}

/// `ID_Indicator_TT_NetStatus` and `ID_Indicator_TT_NetStatusBW`, with the variables
/// `latency`, `loss`, `BytesIn` and `BytesOut`.
const TOOLTIP: &str = "ID_Indicator_TT_NetStatus";
const TOOLTIP_BANDWIDTH: &str = "ID_Indicator_TT_NetStatusBW";

/// What `gmNetStatusUI::OnHeartbeat` reads from `CLinkStatusAverages`.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct LinkSample {
    /// The mean round trip, in seconds (`m_RoundTripDelays`; 0 with no samples).
    pub round_trip: f64,
    /// `GetAveragePacketLoss`, as a fraction.
    pub packet_loss: f64,
    /// Bytes received over the last snapshot, per second.
    pub bytes_in_per_second: f32,
    /// Bytes sent over the last snapshot, whole bytes per second (`(long)` in retail).
    pub bytes_out_per_second: f32,
    /// `CLinkStatusSnapshot::TimeSinceLastGotData`, in seconds.
    pub since_last_data: f32,
}

impl LinkSample {
    /// `gmNetStatusUI::OnHeartbeat`'s reads: the rates are the last snapshot's counters over its
    /// duration, in `float` (`1.0 / SnapshotDuration`, then the products).
    pub fn from_link(links: &LinkAverages) -> Self {
        let snap = links.snapshot;
        let per_second = 1.0f32 / snap.duration;
        Self {
            round_trip: links.mean_round_trip(),
            packet_loss: links.packet_loss(),
            bytes_in_per_second: snap.bytes_received as f32 * per_second,
            bytes_out_per_second: (snap.bytes_sent as f32 * per_second).trunc(),
            since_last_data: snap.since_last_data,
        }
    }
}

/// `gmNetStatusUI::GraphHistoryColumn`: one heartbeat's bytes sent and received.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct GraphColumn {
    pub bytes_sent: u32,
    pub bytes_received: u32,
}

/// `gmNetStatusUI`'s state.
#[derive(Debug, Clone)]
pub struct NetStatusUi {
    field: Option<usize>,
    /// `m_good`, `m_bad`, `m_ugly`: the times since the last data at which the field moves on.
    good: f32,
    bad: f32,
    ugly: f32,
    /// `m_GraphHistory` and `m_GraphHistoryOffset`: a ring of columns, one per graph pixel.
    history: Vec<GraphColumn>,
    offset: usize,
}

impl NetStatusUi {
    /// `gmNetStatusUI::gmNetStatusUI`: the field (an `IUIManager` lookup of `NetStatusField`)
    /// and its three thresholds (`GetAttribute_Float`, 0 where the layout sets none).
    pub fn create(tree: &UiTree) -> Self {
        let field = tree
            .roots()
            .iter()
            .find_map(|&r| tree.child_recursive(r, id("NetStatusField")));
        let float = |name: &str| {
            field
                .and_then(|f| tree.get(f))
                .and_then(|e| e.attribute(id(name)))
                .and_then(|value| match value {
                    AttributeValue::Float(v) => Some(*v),
                    _ => None,
                })
                .unwrap_or(0.0)
        };
        Self {
            field,
            good: float("good"),
            bad: float("bad"),
            ugly: float("ugly"),
            history: Vec::new(),
            offset: 0,
        }
    }

    /// The field's state.
    pub fn state(&self, tree: &UiTree) -> Option<u32> {
        Some(tree.get(self.field?)?.state())
    }

    /// The graph's columns, oldest at `offset` (the next one written).
    pub fn history(&self) -> (&[GraphColumn], usize) {
        (&self.history, self.offset)
    }

    /// `gmNetStatusUI::OnHeartbeat`: the tooltip, one graph column, then the state.
    pub fn on_heartbeat(
        &mut self,
        tree: &mut UiTree,
        strings: &dyn FellowStrings,
        sample: LinkSample,
    ) {
        let Some(field) = self.field else {
            return;
        };
        let float = |value: f64| SkillVar::Float {
            value,
            precision: 0,
        };
        let loss = (sample.packet_loss * 100.0) as f32;
        let latency = strings.render(
            id(TOOLTIP),
            &[
                (id("latency"), float(sample.round_trip * 1000.0)),
                (id("loss"), float(f64::from(loss))),
            ],
        );
        let bandwidth = strings.render(
            id(TOOLTIP_BANDWIDTH),
            &[
                (id("BytesIn"), float(f64::from(sample.bytes_in_per_second))),
                (
                    id("BytesOut"),
                    float(f64::from(sample.bytes_out_per_second)),
                ),
            ],
        );
        tree.set_tooltip(field, Some(format!("{latency}\n{bandwidth}")));

        self.push_column(
            tree,
            GraphColumn {
                bytes_sent: sample.bytes_out_per_second as u32,
                bytes_received: sample.bytes_in_per_second as u32,
            },
        );

        let state = if sample.since_last_data < self.good {
            id("stateless")
        } else if sample.since_last_data < self.bad {
            id("good")
        } else if sample.since_last_data < self.ugly {
            id("bad")
        } else {
            id("ugly")
        };
        tree.set_state(field, state);
    }

    /// The graph is as wide as the field less its `border_pixels` on each side; the ring is
    /// resized to that and the column goes in at the offset, which then advances.
    fn push_column(&mut self, tree: &UiTree, column: GraphColumn) {
        let Some(field) = self.field else {
            return;
        };
        let Some(element) = tree.get(field) else {
            return;
        };
        let border = match element.attribute(id("border_pixels")) {
            Some(AttributeValue::Int(v)) => *v,
            _ => 0,
        };
        let width = element.rect().width() - border * 2;
        if width <= 0 {
            return;
        }
        let width = width as usize;
        if self.history.len() != width {
            self.history.resize(width, GraphColumn::default());
            self.offset = 0;
        }
        self.history[self.offset] = column;
        self.offset = (self.offset + 1) % width;
    }
}

#[cfg(test)]
mod tests {
    use ac2_net::{LinkAverages, LinkSnapshot};

    use super::LinkSample;

    /// `gmNetStatusUI::OnHeartbeat`'s reads over a heartbeat: the rates are the snapshot's bytes
    /// over its duration (the first sample counts one second only in the averages, not here).
    #[test]
    fn a_heartbeat_reads_the_last_snapshot_as_rates() {
        let mut links = LinkAverages::default();
        links.add_snapshot(
            &LinkSnapshot {
                duration: 2.0,
                bytes_received: 1000,
                bytes_sent: 501,
                since_last_data: 0.25,
                ..Default::default()
            },
            10.0,
        );
        let sample = LinkSample::from_link(&links);
        assert_eq!(sample.bytes_in_per_second, 500.0);
        // (long)(501 / 2.0) truncates to 250.
        assert_eq!(sample.bytes_out_per_second, 250.0);
        assert_eq!(sample.since_last_data, 0.25);
        assert_eq!(sample.round_trip, 0.0);
        assert_eq!(sample.packet_loss, 0.0);
    }

    #[test]
    fn the_round_trip_is_the_mean_of_the_world_pings() {
        let mut links = LinkAverages::default();
        links.on_ping_response(0.25);
        links.on_ping_response(0.75);
        let sample = LinkSample::from_link(&links);
        assert!((sample.round_trip - 0.5).abs() < 1e-6);
    }
}
