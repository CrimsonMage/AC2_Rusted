//! `gmExaminationUI`: the examination panel. While shown it holds the player's examination
//! request open (`Show`: `StartExamination` / `StopExamination`) and shows a pending row until
//! the server's profile arrives, or an empty row without a request (`Update`). A profile is laid
//! out into one row of the panel's list box (`RecvNotice_UpdateExaminationProfile`,
//! `ProcessNode`): text nodes as elements of their appearance template (else
//! `ExaminationTextTemplate`) placed left to right and wrapped to the row's width with the tab
//! indent (`DoBreak`), images and icons, breaks, tabs of 16 pixels, and countdowns; the row is as
//! tall as its content.
//!
//! The client turns the wire profile into [`ExaminationItem`]s, rendering each value's text the
//! way `MakeString` builds it, and expands effect nodes first (`ProcessEffectIDNode`, in the
//! client, which asks the player's script).
//!
//! A countdown is a `TimeDisplay` (`Init` with the seconds left; `ListenToMessage` on the global
//! loop rewrites the words, see [`crate::journal_time`]); the panel passes it the global loop each
//! frame through [`ExaminationUi::global_loop`]. The countdown's thresholds are a day and a minute
//! (1 and 60, as `TimeDisplay::Init` is called); the month threshold is 1 as the journal sets it.
//!
//! The Legions filter tabs are [`crate::examination_tabs`]: the panel keeps the request's third
//! word as its filter (`+0xB8`), sends it with `StartExamination`, and the tabs' presses and the
//! selection keys change it.
//!
//! Not modelled yet: an image's own size before it loads (the template's size is used), and text
//! that arrives later (`download_done`).

use std::sync::Arc;

use crate::element::UiTree;
use crate::examination_tabs::{ExaminationTabs, DEFAULT_FILTER, NEXT_TAB_KEY, PREV_TAB_KEY};
use crate::journal_time::QuestTimeDisplay;
use crate::layout::LayoutDesc;
use crate::mouse::{UiMessage, GLOBAL_KEY_PRESS};
use crate::token::{tokenize, MSG_MOUSE_PRESS};
use crate::ui_string::UiStrings;

/// One laid-out item of a profile (`ExaminationDataNode` by type).
#[derive(Debug, Clone, PartialEq)]
pub enum ExaminationItem {
    /// Types 1-4 and 0xA: `MakeString`'s text, in the node's appearance.
    Text {
        appearance: u32,
        text: String,
    },
    /// Type 5: an image DID; type 9: an icon description's layers.
    Image(u32),
    Icon(Vec<crate::IconLayer>),
    /// Type 6.
    Break,
    /// Type 7: the tab count.
    Tabs(u32),
    /// Type 8: seconds left.
    Countdown {
        appearance: u32,
        seconds: f64,
    },
}

/// What the panel asks of the client.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExaminationCall {
    /// `gmCWInterface::StartExamination` with the panel's filter word (`+0xB8`).
    Start(u32),
    /// `gmCWInterface::StopExamination`.
    Stop,
}

fn id(name: &str) -> u32 {
    tokenize(name)
}

/// `gmExaminationUI`'s state. `timers` are the countdown rows' `TimeDisplay`s (`m_timeArray`).
#[derive(Debug, Clone)]
pub struct ExaminationUi {
    layout: Arc<LayoutDesc>,
    field: Option<usize>,
    list: Option<usize>,
    /// `_currentRequest` (its type; 0 for none).
    request: u32,
    /// `_currentRequest`'s third word (`+0x28`): what `Update` shows the tabs for. The panel's
    /// own filter (`+0xB8`) follows it and then the tabs the player chooses.
    request_filter: u32,
    shown: bool,
    cur: (i32, i32),
    next_y: i32,
    tab_level: i32,
    row: Option<usize>,
    timers: Vec<QuestTimeDisplay>,
    calls: Vec<ExaminationCall>,
    tabs: ExaminationTabs,
}

/// `TimeDisplay::Init`'s thresholds for a countdown row: months (1), days (1), minutes (60).
const COUNTDOWN_THRESHOLDS: (i64, i64, i64) = (1, 1, 60);

/// The clock a profile is laid out at: the strings the countdowns render with, and `now`.
#[derive(Clone, Copy)]
struct Clock<'a> {
    strings: &'a dyn UiStrings,
    now: f64,
}

impl ExaminationUi {
    /// `gmExaminationUI::gmExaminationUI`: the field and its list box; hidden.
    pub fn create(tree: &mut UiTree, layout: &Arc<LayoutDesc>) -> Self {
        let field = tree
            .roots()
            .iter()
            .find_map(|&r| tree.child_recursive(r, id("ExaminationField")));
        let tabs = ExaminationTabs::new(tree, field);
        let mut ui = Self {
            layout: Arc::clone(layout),
            field,
            list: field.and_then(|f| tree.child_recursive(f, id("ExaminationListBox"))),
            request: 0,
            request_filter: DEFAULT_FILTER,
            shown: false,
            cur: (0, 0),
            next_y: 0,
            tab_level: 0,
            row: None,
            timers: Vec::new(),
            calls: Vec::new(),
            tabs,
        };
        // Legions 0x00431C40: the tabs start in the construction filter's state.
        let filter = ui.tabs.filter();
        ui.tabs.set_states(tree, filter, filter);
        ui.update(tree, 0, filter);
        ui
    }

    pub fn take_calls(&mut self) -> Vec<ExaminationCall> {
        std::mem::take(&mut self.calls)
    }

    /// The panel's list box (its rows).
    pub fn list(&self) -> Option<usize> {
        self.list
    }

    /// `gmExaminationUI::Show`, as the toolbar shows or hides its field.
    pub fn shown_changed(&mut self, tree: &mut UiTree, shown: bool) {
        if shown == self.shown {
            return;
        }
        self.shown = shown;
        if shown {
            self.update(tree, self.request, self.request_filter);
            self.calls.push(ExaminationCall::Start(self.tabs.filter()));
        } else {
            self.calls.push(ExaminationCall::Stop);
            // `Update` of a cleared `ExaminationRequest`: its filter is the default word.
            self.update(tree, 0, DEFAULT_FILTER);
        }
    }

    /// The panel follows its field's visibility (the toolbar's buttons and keys show it).
    pub fn follow_field(&mut self, tree: &mut UiTree) {
        let shown = self
            .field
            .and_then(|f| tree.get(f))
            .is_some_and(|e| e.visible);
        self.shown_changed(tree, shown);
    }

    /// `RecvNotice_ExaminationRequestChanged` (Legions 0x00431B00): the request's type, and its
    /// third word becomes the panel's filter.
    pub fn request_changed(&mut self, tree: &mut UiTree, kind: u32, filter: u32) {
        self.request = kind;
        self.request_filter = filter;
        self.tabs.set_filter(filter);
        if self.shown {
            self.update(tree, kind, filter);
        }
    }

    /// The panel's filter word (`+0xB8`).
    pub fn filter(&self) -> u32 {
        self.tabs.filter()
    }

    /// `gmExaminationUI::CleanupArray` and the list's flush (`Update`): the rows and the
    /// countdowns go, and the layout starts again.
    fn start_rows(&mut self, tree: &mut UiTree) {
        if let Some(list) = self.list {
            tree.list_flush(list);
        }
        self.timers.clear();
        self.cur = (0, 0);
        self.next_y = 0;
        self.tab_level = 0;
        self.row = None;
    }

    /// The countdowns' global loop (`TimeDisplay::ListenToMessage`): each rewrites its words
    /// when its next change is due.
    pub fn global_loop(&mut self, tree: &mut UiTree, strings: &dyn UiStrings, now: f64) {
        for timer in &mut self.timers {
            timer.global_loop(tree, strings, now);
        }
    }

    fn template(&self, tree: &mut UiTree, element: u32) -> Option<usize> {
        let desc = self.layout.element(element)?.clone();
        tree.create_element(Arc::clone(&self.layout), desc, false)
    }

    /// `gmExaminationUI::Update`: the empty row, or the pending row while a request waits; the
    /// tabs show the request's `filter` (the active ones, the rest disabled).
    fn update(&mut self, tree: &mut UiTree, kind: u32, filter: u32) {
        self.start_rows(tree);
        self.tabs.set_states(tree, filter, filter);
        let Some(list) = self.list else {
            return;
        };
        let row = if kind == 0 {
            "ExaminationEmptyRow"
        } else {
            "ExaminationPendingRow"
        };
        if let Some(row) = self.template(tree, id(row)) {
            tree.list_add_item(list, row);
        }
    }

    /// `gmExaminationUI::DoBreak`: the next line, at least 16 pixels down, indented by the tabs
    /// when `auto_indent`.
    fn do_break(&mut self, auto_indent: bool) {
        if self.next_y - self.cur.1 < 16 {
            self.next_y = self.cur.1 + 16;
        }
        self.cur.1 = self.next_y;
        self.cur.0 = 0;
        if auto_indent {
            self.cur.0 = self.tab_level << 4;
        } else {
            self.tab_level = 0;
        }
    }

    fn row_width(&self, tree: &UiTree) -> i32 {
        self.row
            .and_then(|r| tree.get(r))
            .map_or(0, |e| e.rect.width())
    }

    /// `RecvNotice_UpdateExaminationProfile`: the profile's items laid out at `now`. `filter` is
    /// the profile's request word and `available` the word packed after its nodes: they set the
    /// tabs (Legions 0x00431200).
    pub fn update_profile(
        &mut self,
        tree: &mut UiTree,
        strings: &dyn UiStrings,
        now: f64,
        items: &[ExaminationItem],
        (filter, available): (u32, u32),
    ) {
        self.start_rows(tree);
        self.tabs.set_states(tree, filter, available);
        let clock = Clock { strings, now };
        for item in items {
            self.process_node(tree, clock, item);
        }
        let (Some(list), Some(row)) = (self.list, self.row.take()) else {
            return;
        };
        let rect = tree.get(row).map(|e| e.rect).unwrap_or_default();
        if rect.height() < self.next_y {
            tree.resize_to(row, rect.width(), self.next_y);
        }
        tree.list_add_item(list, row);
    }

    /// The row being laid out (`ExaminationRowTemplate`, as wide as the list).
    fn current_row(&mut self, tree: &mut UiTree) -> Option<usize> {
        if self.row.is_none() {
            let row = self.template(tree, id("ExaminationRowTemplate"))?;
            let width = self
                .list
                .and_then(|l| tree.get(l))
                .map_or(0, |e| e.rect.width());
            let height = tree.get(row).map_or(0, |e| e.rect.height());
            tree.resize_to(row, width, height);
            self.row = Some(row);
        }
        self.row
    }

    /// The text element for an appearance (its template, else `ExaminationTextTemplate`).
    fn text_element(&self, tree: &mut UiTree, appearance: u32) -> Option<usize> {
        self.template(tree, appearance)
            .or_else(|| self.template(tree, id("ExaminationTextTemplate")))
    }

    fn place(&mut self, tree: &mut UiTree, row: usize, element: usize, size: (i32, i32)) {
        tree.set_parent(element, Some(row));
        tree.move_to(element, self.cur.0, self.cur.1);
        tree.resize_to(element, size.0, size.1);
    }

    fn grow_next_y(&mut self, tree: &UiTree, element: usize) {
        let height = tree.get(element).map_or(0, |e| e.rect.height());
        self.next_y = self.next_y.max(self.cur.1 + height);
    }

    /// `gmExaminationUI::ProcessNode`.
    fn process_node(&mut self, tree: &mut UiTree, clock: Clock<'_>, item: &ExaminationItem) {
        let Some(row) = self.current_row(tree) else {
            return;
        };
        match item {
            ExaminationItem::Text { appearance, text } => {
                let Some(element) = self.text_element(tree, *appearance) else {
                    return;
                };
                tree.set_text(element, text);
                let (mut width, _) = tree.text_size_with_margins(element, text).unwrap_or((0, 0));
                let row_width = self.row_width(tree);
                if row_width <= width + self.cur.0 && self.tab_level * 16 < self.cur.0 {
                    self.do_break(true);
                }
                width = width.min(row_width - self.cur.0);
                let height = tree.get(element).map_or(0, |e| e.rect.height());
                self.place(tree, row, element, (width, height));
                self.cur.0 += width;
                // `RecalculateGlyphList`, then as tall as the laid-out text.
                let height = tree.text_paper_height(element).unwrap_or(height);
                tree.resize_to(element, width, height);
                self.grow_next_y(tree, element);
            }
            ExaminationItem::Image(_) | ExaminationItem::Icon(_) => {
                if self.row_width(tree) <= self.cur.0 {
                    self.do_break(true);
                    self.cur.0 += 16;
                    self.tab_level += 1;
                }
                let Some(element) = self.template(tree, id("ExaminationImageTemplate")) else {
                    return;
                };
                // Type 5: the image in its stateless state (`AddImageToState`, draw mode 0); type
                // 9: the icon description's layers (`SetIconDesc`).
                match item {
                    ExaminationItem::Image(file) => tree.set_image(element, *file, 0),
                    ExaminationItem::Icon(layers) => tree.set_icon_desc(element, layers),
                    _ => {}
                }
                let rect = tree.get(element).map(|e| e.rect).unwrap_or_default();
                let width = rect.width().min(self.row_width(tree) - self.cur.0);
                self.place(tree, row, element, (width, rect.height()));
                self.cur.0 += width;
                self.grow_next_y(tree, element);
            }
            ExaminationItem::Break => self.do_break(false),
            ExaminationItem::Tabs(count) => {
                let count = (*count as i32).max(1);
                self.cur.0 += count * 16;
                self.tab_level += count;
            }
            ExaminationItem::Countdown {
                appearance,
                seconds,
            } => {
                let Some(element) = self.text_element(tree, *appearance) else {
                    return;
                };
                let (mut width, _) = tree
                    .text_size_with_margins(element, "(XXX XXX XX)")
                    .unwrap_or((0, 0));
                if self.row_width(tree) - self.cur.0 < width {
                    self.do_break(true);
                    width = width.min(self.row_width(tree) - self.cur.0);
                }
                self.place(tree, row, element, (width, 16));
                // `TimeDisplay::Init` on the element: the seconds left from now.
                let timer = QuestTimeDisplay::init(
                    tree,
                    clock.strings,
                    element,
                    clock.now + seconds,
                    COUNTDOWN_THRESHOLDS,
                    clock.now,
                );
                self.timers.push(timer);
                self.grow_next_y(tree, element);
            }
        }
    }

    /// `gmExaminationUI::ListenToMessage`: the field's close button hides it (the toolbar's
    /// panel children handle that), so the shown state is followed; a press on a tab that is not
    /// disabled chooses its filter (a Ctrl press toggles it), and the selection keys step
    /// through the tabs (Legions 0x00431990 and 0x00431900). Either asks the player to start the
    /// examination again with the new word.
    pub fn message(&mut self, tree: &mut UiTree, message: &UiMessage, ctrl: bool) {
        self.follow_field(tree);
        if !self.shown {
            return;
        }
        let chosen = if message.element.is_none() && message.message == GLOBAL_KEY_PRESS {
            match message.data as u32 {
                NEXT_TAB_KEY => self.tabs.step(tree, true, ctrl),
                PREV_TAB_KEY => self.tabs.step(tree, false, ctrl),
                _ => None,
            }
        } else if message.message == MSG_MOUSE_PRESS {
            self.tabs.press(tree, message.element_id, ctrl)
        } else {
            None
        };
        if let Some(filter) = chosen {
            self.calls.push(ExaminationCall::Start(filter));
        }
    }
}
