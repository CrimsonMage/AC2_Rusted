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
//! The bandwidth graph (`gmNetStatusUI::DrawBandwidthGraph`) is painted over the composed status
//! bar surface ([`NetStatusUi::draw_bandwidth_graph`]) when the latency-graph option is on
//! (`gmNetStatusUI::RecvNotice_ReloadOptions`, [`NetStatusUi::set_graph_enabled`]). Retail blits it
//! through a 32 x 32 render surface with `SurfaceWindow::Blit` flags `0x14`, which are not modelled:
//! this writes the bars straight into the root's surface.

use ac2_net::LinkAverages;

use crate::compose::UiSurface;
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

/// The graph's largest size in pixels (`DrawBandwidthGraph`'s `0x20` caps).
const GRAPH_MAX: i32 = 0x20;

/// The attribute names of the graph's colours: `r_back`, ..., `a_recv` (`token_*`).
const BACK: [&str; 4] = ["r_back", "g_back", "b_back", "a_back"];
const SENT: [&str; 4] = ["r_sent", "g_sent", "b_sent", "a_sent"];
const RECV: [&str; 4] = ["r_recv", "g_recv", "b_recv", "a_recv"];

/// `gmNetStatusUI`'s state.
#[derive(Debug, Clone)]
pub struct NetStatusUi {
    field: Option<usize>,
    /// `m_bGraphEnabled`: the latency-graph option (`gmCWInterface::GetLatencyIndicatorGraph`).
    graph_enabled: bool,
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
            graph_enabled: false,
            good: float("good"),
            bad: float("bad"),
            ugly: float("ugly"),
            history: Vec::new(),
            offset: 0,
        }
    }

    /// The field, the element the tooltip and state go on (`m_field`).
    pub fn field(&self) -> Option<usize> {
        self.field
    }

    /// `gmNetStatusUI::RecvNotice_ReloadOptions`: whether the graph is drawn.
    pub fn set_graph_enabled(&mut self, enabled: bool) {
        self.graph_enabled = enabled;
    }

    pub fn graph_enabled(&self) -> bool {
        self.graph_enabled
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

/// The field's absolute origin in the surface its root composes to, and that root: the
/// composition draws each child at its parent's origin plus its own box (`UIRegion::DrawHere`).
fn origin_in_root(tree: &UiTree, index: usize) -> (usize, (i32, i32)) {
    let (mut x, mut y) = (0, 0);
    let mut at = index;
    loop {
        let Some(element) = tree.get(at) else {
            return (at, (x, y));
        };
        x += element.rect().x0;
        y += element.rect().y0;
        match element.parent_index() {
            Some(parent) => at = parent,
            None => return (at, (x, y)),
        }
    }
}

/// A `float` colour attribute set to a byte, as the surface stores it.
fn byte(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn colour(tree: &UiTree, field: usize, names: [&str; 4]) -> [f32; 4] {
    let float = |name: &str| match tree.get(field).and_then(|e| e.attribute(id(name))) {
        Some(AttributeValue::Float(v)) => *v,
        _ => 0.0,
    };
    names.map(float)
}

/// `GraphTool`: the colours and the bar height limit of one graph.
struct Graph<'a> {
    surface: &'a mut UiSurface,
    /// The graph's top-left corner in the surface.
    x0: i32,
    y0: i32,
    height: i32,
    sent: [f32; 4],
    recv: [f32; 4],
    back: [f32; 4],
    /// `m_HeightCap`: the tallest bar, in log2 steps.
    cap: u32,
}

impl Graph<'_> {
    fn put(&mut self, x: i32, y: i32, rgba: [f32; 4]) {
        let (sw, sh) = (self.surface.width as i32, self.surface.height as i32);
        let (x, y) = (self.x0 + x, self.y0 + y);
        if x < 0 || y < 0 || x >= sw || y >= sh {
            return;
        }
        let at = (y * sw + x) as usize * 4;
        for (dst, src) in self.surface.rgba[at..at + 4].iter_mut().zip(rgba) {
            *dst = byte(src);
        }
    }

    /// `GraphTool::BlendColors`: `frac * bar + (1 - frac) * back`, in `float`.
    fn blend(&self, bar: [f32; 4], frac: f32) -> [f32; 4] {
        std::array::from_fn(|i| frac * bar[i] + (1.0 - frac) * self.back[i])
    }

    /// `GraphTool::DrawBar`: a column of `bytes / 8` pixels takes its height from the log2 of
    /// that, up from the bottom for received and down from the top for sent, with the
    /// fraction above the last whole step blended in.
    fn bar(&mut self, x: i32, bytes: u32, up: bool) {
        let units = bytes >> 3;
        if units == 0 {
            return;
        }
        let steps = (31 - units.leading_zeros()).min(self.cap) as i32;
        let (bar, rows) = if up {
            (self.recv, (self.height - 1 - steps)..=(self.height - 1))
        } else {
            (self.sent, 0..=steps)
        };
        for y in rows {
            self.put(x, y, bar);
        }
        if steps as u32 != self.cap {
            let whole = 1u32 << steps;
            let frac = (units - whole) as f32 / whole as f32;
            let blended = self.blend(bar, frac);
            let y = if up {
                self.height - 2 - steps
            } else {
                steps + 1
            };
            self.put(x, y, blended);
        }
    }
}

impl NetStatusUi {
    /// `gmNetStatusUI::DrawBandwidthGraph`: the history's columns, oldest on the left, in the
    /// field's box less its `border_pixels` (at most 32 by 32). Drawn into the surface whose root
    /// holds the field, as `ListenToMessage` does when the status bar draws. Nothing is drawn with
    /// the option off, no field, or no history.
    pub fn draw_bandwidth_graph(&self, tree: &UiTree, surfaces: &mut [UiSurface]) {
        let Some(field) = self.field.filter(|_| self.graph_enabled) else {
            return;
        };
        let Some(element) = tree.get(field) else {
            return;
        };
        let (root, (fx, fy)) = origin_in_root(tree, field);
        let Some(surface) = surfaces.iter_mut().find(|s| s.root == root) else {
            return;
        };
        let border = match element.attribute(id("border_pixels")) {
            Some(AttributeValue::Int(v)) => *v,
            _ => 0,
        };
        let (width, height) = (element.rect().width(), element.rect().height());
        if border * 2 >= width || border * 2 >= height {
            return;
        }
        let width = (width - border * 2).min(GRAPH_MAX);
        let height = (height - border * 2).min(GRAPH_MAX);
        // `GraphTool::GraphTool`: half the graph's height, less one.
        let half = (height + 1) / 2;
        let mut graph = Graph {
            surface,
            x0: fx + border,
            y0: fy + border,
            height,
            sent: colour(tree, field, SENT),
            recv: colour(tree, field, RECV),
            back: colour(tree, field, BACK),
            cap: if half > 0 { (half - 1) as u32 } else { 0 },
        };
        for y in 0..height {
            for x in 0..width {
                let back = graph.back;
                graph.put(x, y, back);
            }
        }
        let n = self.history.len();
        if n == 0 {
            return;
        }
        for x in 0..width {
            let column = self.history[(self.offset + x as usize) % n];
            graph.bar(x, column.bytes_sent, false);
            graph.bar(x, column.bytes_received, true);
        }
    }
}

#[cfg(test)]
mod tests {
    use ac2_net::{LinkAverages, LinkSnapshot};

    use super::{Graph, LinkSample};
    use crate::compose::UiSurface;

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

    fn blank_surface() -> UiSurface {
        UiSurface {
            root: 0,
            x: 0,
            y: 0,
            width: 1,
            height: 32,
            rotation: 0,
            shown: true,
            opacity: 1.0,
            rgba: vec![0; 32 * 4],
        }
    }

    fn pixel(surface: &UiSurface, y: usize) -> Vec<u8> {
        surface.rgba[y * 4..y * 4 + 4].to_vec()
    }

    const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
    const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];

    /// `GraphTool::DrawBar` on a 32 pixel graph capped at 15 steps: received bars grow up from the
    /// bottom, and the fraction above the last whole step is blended with the back colour.
    #[test]
    fn a_received_bar_grows_up_with_its_blended_step() {
        let mut surface = blank_surface();
        let mut graph = Graph {
            surface: &mut surface,
            x0: 0,
            y0: 0,
            height: 32,
            sent: GREEN,
            recv: RED,
            back: [0.0; 4],
            cap: 15,
        };
        // 12288 bytes is 1536 units: 10 whole steps (1024), then a half step (512 of 1024).
        graph.bar(0, 8 * 1536, true);
        assert_eq!(pixel(&surface, 31), vec![255, 0, 0, 255]);
        assert_eq!(pixel(&surface, 21), vec![255, 0, 0, 255]);
        assert_eq!(pixel(&surface, 20), vec![128, 0, 0, 128]);
        assert_eq!(pixel(&surface, 19), vec![0, 0, 0, 0]);
    }

    /// Sent bars grow down from the top; a count past the cap fills to the cap and has no
    /// fraction above it.
    #[test]
    fn a_sent_bar_grows_down_and_stops_at_the_cap() {
        let mut surface = blank_surface();
        let mut graph = Graph {
            surface: &mut surface,
            x0: 0,
            y0: 0,
            height: 32,
            sent: GREEN,
            recv: RED,
            back: [0.0; 4],
            cap: 15,
        };
        graph.bar(0, 8 * 1536, false);
        assert_eq!(pixel(&surface, 0), vec![0, 255, 0, 255]);
        assert_eq!(pixel(&surface, 10), vec![0, 255, 0, 255]);
        assert_eq!(pixel(&surface, 11), vec![0, 128, 0, 128]);
        assert_eq!(pixel(&surface, 12), vec![0, 0, 0, 0]);

        let mut surface = blank_surface();
        let mut graph = Graph {
            surface: &mut surface,
            x0: 0,
            y0: 0,
            height: 32,
            sent: GREEN,
            recv: RED,
            back: [0.0; 4],
            cap: 15,
        };
        graph.bar(0, u32::MAX, false);
        assert_eq!(pixel(&surface, 15), vec![0, 255, 0, 255]);
        assert_eq!(pixel(&surface, 16), vec![0, 0, 0, 0]);
    }
}
