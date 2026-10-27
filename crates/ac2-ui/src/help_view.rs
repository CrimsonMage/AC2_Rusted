//! The help window's model (`AC2HelpPlugin.dll`'s KSML viewer, opened by `KeyStone::OpenHelp`):
//! a page's [`KsmlDocument`] laid out into blocks of wrapped, styled text, radio tabs and frames,
//! the page and frame loads it asks for, and the scroll position.
//!
//! Retail rendered these pages in the plugin's own window (Windows only). Here they are drawn by
//! the client on every platform, so the model is kept apart from the drawing: [`HelpView::open`]
//! and [`HelpView::press_link`] return the [`HelpRequest`]s a loader fills off the frame thread
//! ([`HelpView::loaded`]); [`layout_blocks`] wraps a page to a width with the caller's measure.
//! Not drawn yet: controls (`button`, `listbox`, `editbox`, `treeview`) and the window chrome.

use crate::ksml::{KsmlDocument, KsmlFrame, KsmlNode, KsmlStyle, KsmlTab};
use crate::wheel::WheelAccumulator;

/// The scroll step of one wheel notch, in line heights.
const LINES_PER_NOTCH: i32 = 3;

/// A link's destination: the page it loads and the frame it loads into (`None` for the page).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelpLink {
    pub href: Option<String>,
    pub target: Option<String>,
}

/// A word or a span of words with one style, as laid out on a line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelpSegment {
    pub text: String,
    pub style: KsmlStyle,
    /// The link the text is in, if any.
    pub link: Option<HelpLink>,
}

/// One block of a laid-out page, from the top down.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HelpBlock {
    /// A line of wrapped text.
    Line(Vec<HelpSegment>),
    /// A row of radio tabs (consecutive `radiobutton` elements).
    Tabs(Vec<KsmlTab>),
    /// An `iframe`: a region of `height` pixels showing its own page.
    Frame(KsmlFrame),
}

/// The laid-out page: each block with its top edge and height, and the total height.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HelpLayout {
    pub blocks: Vec<(i32, i32, HelpBlock)>,
    pub height: i32,
}

/// The text metrics a layout uses: the width of a run of text in a style, and the line height.
pub struct HelpMetrics<'a> {
    pub width: &'a dyn Fn(&str, KsmlStyle) -> i32,
    pub line_height: i32,
}

/// A page load the caller makes off the frame thread, answered with [`HelpView::loaded`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HelpRequest {
    /// Load `page` into the main page (`target` is `None`) or the frame named `target`.
    Load {
        ticket: u64,
        page: String,
        target: Option<String>,
    },
    /// The window closes (`gmHelp`'s close button).
    Close,
}

/// A frame's embedded page.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HelpFrame {
    pub name: String,
    pub page: Option<String>,
    pub document: Option<KsmlDocument>,
    /// The ticket of the frame's pending load, if any.
    pending: Option<u64>,
}

/// The help window's state over one page.
#[derive(Debug, Clone, Default)]
pub struct HelpView {
    page: Option<String>,
    document: Option<KsmlDocument>,
    /// The newest ticket handed out; a load answers with the ticket it was given.
    ticket: u64,
    /// The pending load for the main page, if any.
    pending: Option<u64>,
    frames: Vec<HelpFrame>,
    /// The scroll position in pixels from the top of the page.
    scroll: i32,
    wheel: WheelAccumulator,
}

impl HelpView {
    /// The window's state before any page: nothing loaded, no frames.
    pub fn new() -> Self {
        Self::default()
    }

    /// `KeyStone::OpenHelp` on a page: the request that loads it.
    pub fn open(&mut self, page: &str) -> HelpRequest {
        self.ticket += 1;
        self.pending = Some(self.ticket);
        self.page = Some(page.to_string());
        self.document = None;
        self.frames.clear();
        self.scroll = 0;
        HelpRequest::Load {
            ticket: self.ticket,
            page: page.to_string(),
            target: None,
        }
    }

    /// A link or tab pressed: the page it names loads into its target frame (or the main page).
    /// A link with no `href` asks for nothing.
    pub fn press_link(&mut self, link: &HelpLink) -> Option<HelpRequest> {
        let page = link.href.as_ref()?;
        match link.target.as_deref() {
            None | Some("") => {
                self.ticket += 1;
                self.pending = Some(self.ticket);
                Some(HelpRequest::Load {
                    ticket: self.ticket,
                    page: page.clone(),
                    target: None,
                })
            }
            Some(target) => {
                self.ticket += 1;
                let ticket = self.ticket;
                let frame = self.frame_mut(target);
                frame.page = Some(page.clone());
                frame.document = None;
                frame.pending = Some(ticket);
                Some(HelpRequest::Load {
                    ticket,
                    page: page.clone(),
                    target: Some(target.to_string()),
                })
            }
        }
    }

    /// The load for `ticket` finished. Returns `false` (and drops the page) when a newer request
    /// has replaced it, so a slow page never overwrites the one on screen.
    pub fn loaded(&mut self, ticket: u64, target: Option<&str>, document: KsmlDocument) -> bool {
        match target {
            None => {
                if self.pending != Some(ticket) {
                    return false;
                }
                self.pending = None;
                self.document = Some(document);
                self.scroll = 0;
                true
            }
            Some(name) => {
                let Some(frame) = self.frames.iter_mut().find(|f| f.name == name) else {
                    return false;
                };
                if frame.pending != Some(ticket) {
                    return false;
                }
                frame.pending = None;
                frame.document = Some(document);
                true
            }
        }
    }

    /// The page on screen, if it has loaded.
    pub fn document(&self) -> Option<&KsmlDocument> {
        self.document.as_ref()
    }

    /// The name of the page the window shows (its `open` argument).
    pub fn page(&self) -> Option<&str> {
        self.page.as_deref()
    }

    /// The frames the page embeds, by name.
    pub fn frames(&self) -> &[HelpFrame] {
        &self.frames
    }

    fn frame_mut(&mut self, name: &str) -> &mut HelpFrame {
        if let Some(index) = self.frames.iter().position(|f| f.name == name) {
            return &mut self.frames[index];
        }
        self.frames.push(HelpFrame {
            name: name.to_string(),
            page: None,
            document: None,
            pending: None,
        });
        self.frames.last_mut().expect("a frame was just pushed")
    }

    /// The iframes of a page, once it has laid out: each named frame gets its `src` loaded, unless
    /// it already shows that page. Returns the loads the caller starts off the frame thread.
    pub fn embed_frames(&mut self, frames: &[KsmlFrame]) -> Vec<HelpRequest> {
        let mut requests = Vec::new();
        for frame in frames {
            let Some(name) = frame.name.as_ref() else {
                continue;
            };
            let Some(src) = frame.src.as_ref() else {
                continue;
            };
            let already = self
                .frames
                .iter()
                .any(|f| &f.name == name && f.page.as_ref() == Some(src));
            if already {
                continue;
            }
            self.ticket += 1;
            let ticket = self.ticket;
            let slot = self.frame_mut(name);
            slot.page = Some(src.clone());
            slot.document = None;
            slot.pending = Some(ticket);
            requests.push(HelpRequest::Load {
                ticket,
                page: src.clone(),
                target: Some(name.clone()),
            });
        }
        requests
    }

    /// The scroll position in pixels.
    pub fn scroll(&self) -> i32 {
        self.scroll
    }

    /// Scrolls by `pixels` (positive down), clamped to the page for a view `view_height` tall
    /// and `content_height` tall.
    pub fn scroll_by(&mut self, pixels: i32, content_height: i32, view_height: i32) {
        let max = (content_height - view_height).max(0);
        self.scroll = (self.scroll + pixels).clamp(0, max);
    }

    /// A wheel turn over the page. Line deltas are notches; pixel deltas accumulate into whole
    /// notches (the shared [`WheelAccumulator`]). Each notch scrolls [`LINES_PER_NOTCH`] lines.
    pub fn wheel(&mut self, notches: i32, line_height: i32, content_height: i32, view_height: i32) {
        self.scroll_by(
            -notches * LINES_PER_NOTCH * line_height,
            content_height,
            view_height,
        );
    }

    /// The accumulator for pixel wheel deltas.
    pub fn wheel_accumulator(&mut self) -> &mut WheelAccumulator {
        &mut self.wheel
    }

    /// Closes the window: the state is cleared and the close request returned.
    pub fn close(&mut self) -> HelpRequest {
        *self = Self::default();
        HelpRequest::Close
    }
}

/// Lays `nodes` out into blocks no wider than `width`, wrapping at spaces. A word wider than
/// the line is kept whole on its own line. Tabs group into one row; frames take their height.
pub fn layout_blocks(nodes: &[KsmlNode], width: i32, metrics: &HelpMetrics<'_>) -> HelpLayout {
    let mut flow = Flow {
        width,
        metrics,
        line: Vec::new(),
        line_width: 0,
        blocks: Vec::new(),
    };
    flow.walk_all(nodes, KsmlStyle::default(), None);
    flow.flush();
    let mut y = 0;
    let mut blocks = Vec::with_capacity(flow.blocks.len());
    for block in flow.blocks {
        let height = match &block {
            HelpBlock::Line(_) | HelpBlock::Tabs(_) => metrics.line_height,
            HelpBlock::Frame(frame) => frame.height.unwrap_or(0) as i32,
        };
        blocks.push((y, height, block));
        y += height;
    }
    HelpLayout { blocks, height: y }
}

struct Flow<'a, 'm> {
    width: i32,
    metrics: &'a HelpMetrics<'m>,
    line: Vec<HelpSegment>,
    line_width: i32,
    blocks: Vec<HelpBlock>,
}

impl Flow<'_, '_> {
    fn walk_all(&mut self, nodes: &[KsmlNode], style: KsmlStyle, link: Option<&HelpLink>) {
        for node in nodes {
            self.walk(node, style, link);
        }
    }

    fn walk(&mut self, node: &KsmlNode, style: KsmlStyle, link: Option<&HelpLink>) {
        match node {
            KsmlNode::Run(run) => {
                let style = KsmlStyle {
                    bold: style.bold || run.style.bold,
                    italic: style.italic || run.style.italic,
                    underline: style.underline || run.style.underline,
                };
                for word in run.text.split_inclusive(' ') {
                    self.word(word, style, link);
                }
            }
            KsmlNode::LineBreak => self.flush(),
            KsmlNode::Link(a) => {
                let target = HelpLink {
                    href: a.href.clone(),
                    target: a.target.clone(),
                };
                self.walk_all(&a.children, style, Some(&target));
            }
            KsmlNode::Tab(tab) => {
                self.flush();
                match self.blocks.last_mut() {
                    Some(HelpBlock::Tabs(row)) => row.push(tab.clone()),
                    _ => self.blocks.push(HelpBlock::Tabs(vec![tab.clone()])),
                }
            }
            KsmlNode::Frame(frame) => {
                self.flush();
                self.blocks.push(HelpBlock::Frame(frame.clone()));
            }
            KsmlNode::Control(control) => {
                self.walk_all(&control.children, style, link);
            }
        }
    }

    fn word(&mut self, word: &str, style: KsmlStyle, link: Option<&HelpLink>) {
        if word.is_empty() {
            return;
        }
        // The fit test counts the word's visible text and the gap before it, not the space that
        // ends it, so a line may end exactly at the width.
        let core = word.trim_end_matches(' ');
        let gap = if self.line.is_empty() {
            0
        } else {
            (self.metrics.width)(" ", style)
        };
        let core_width = if core.is_empty() {
            0
        } else {
            (self.metrics.width)(core, style)
        };
        if self.line_width + gap + core_width > self.width && !self.line.is_empty() {
            self.flush();
        }
        let gap = if self.line.is_empty() {
            0
        } else {
            (self.metrics.width)(" ", style)
        };
        self.line_width += gap + core_width;
        match self.line.last_mut() {
            Some(last) if last.style == style && last.link.as_ref() == link => {
                last.text.push_str(word);
            }
            _ => self.line.push(HelpSegment {
                text: word.to_string(),
                style,
                link: link.cloned(),
            }),
        }
    }

    fn flush(&mut self) {
        if self.line.is_empty() {
            return;
        }
        // A line never ends in the space that wrapped it onto the next.
        if let Some(last) = self.line.last_mut() {
            while last.text.ends_with(' ') {
                last.text.pop();
            }
        }
        self.line.retain(|s| !s.text.is_empty());
        if !self.line.is_empty() {
            self.blocks
                .push(HelpBlock::Line(std::mem::take(&mut self.line)));
        }
        self.line.clear();
        self.line_width = 0;
    }
}
