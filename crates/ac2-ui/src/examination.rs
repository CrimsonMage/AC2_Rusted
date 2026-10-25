//! `gmExaminationUI`: the examination panel. While shown it holds the player's examination
//! request open (`Show`: `StartExamination` / `StopExamination`) and shows a pending row until
//! the server's profile arrives, or an empty row without a request (`Update`). A profile is laid
//! out into one row of the panel's list box (`RecvNotice_UpdateExaminationProfile`,
//! `ProcessNode`): text nodes as elements of their appearance template (else
//! `ExaminationTextTemplate`) placed left to right and wrapped to the row's width with the tab
//! indent (`DoBreak`), images and icons, breaks, tabs of 16 pixels, and countdowns; the row is as
//! tall as its content.
//!
//! The client turns the wire profile into [`ExaminationNode`]s, rendering each value's text the
//! way `MakeString` builds it, and expands effect nodes (`ProcessEffectIDNode`).
//!
//! Not modelled yet: the countdowns' ticking text (`TimeDisplay`, shown empty at its width), icon
//! descriptions drawn as layered icons (`SetIconDesc`: the first layer is shown), an image's own
//! size before it loads (the template's size is used), and text that arrives later
//! (`download_done`).

use std::sync::Arc;

use crate::element::UiTree;
use crate::layout::LayoutDesc;
use crate::mouse::UiMessage;
use crate::token::tokenize;

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
    Icon(Vec<u32>),
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
    /// `gmCWInterface::StartExamination`.
    Start,
    /// `gmCWInterface::StopExamination`.
    Stop,
}

fn id(name: &str) -> u32 {
    tokenize(name)
}

/// `gmExaminationUI`'s state.
#[derive(Debug, Clone)]
pub struct ExaminationUi {
    layout: Arc<LayoutDesc>,
    field: Option<usize>,
    list: Option<usize>,
    /// `_currentRequest` (its type; 0 for none).
    request: u32,
    shown: bool,
    cur: (i32, i32),
    next_y: i32,
    tab_level: i32,
    row: Option<usize>,
    calls: Vec<ExaminationCall>,
}

impl ExaminationUi {
    /// `gmExaminationUI::gmExaminationUI`: the field and its list box; hidden.
    pub fn create(tree: &mut UiTree, layout: &Arc<LayoutDesc>) -> Self {
        let field = tree
            .roots()
            .iter()
            .find_map(|&r| tree.child_recursive(r, id("ExaminationField")));
        let mut ui = Self {
            layout: Arc::clone(layout),
            field,
            list: field.and_then(|f| tree.child_recursive(f, id("ExaminationListBox"))),
            request: 0,
            shown: false,
            cur: (0, 0),
            next_y: 0,
            tab_level: 0,
            row: None,
            calls: Vec::new(),
        };
        ui.update(tree, 0);
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
            self.update(tree, self.request);
            self.calls.push(ExaminationCall::Start);
        } else {
            self.calls.push(ExaminationCall::Stop);
            self.update(tree, 0);
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

    /// `RecvNotice_ExaminationRequestChanged` (the request's type).
    pub fn request_changed(&mut self, tree: &mut UiTree, kind: u32) {
        self.request = kind;
        if self.shown {
            self.update(tree, kind);
        }
    }

    fn start_rows(&mut self, tree: &mut UiTree) {
        if let Some(list) = self.list {
            tree.list_flush(list);
        }
        self.cur = (0, 0);
        self.next_y = 0;
        self.tab_level = 0;
        self.row = None;
    }

    fn template(&self, tree: &mut UiTree, element: u32) -> Option<usize> {
        let desc = self.layout.element(element)?.clone();
        tree.create_element(Arc::clone(&self.layout), desc, false)
    }

    /// `gmExaminationUI::Update`: the empty row, or the pending row while a request waits.
    fn update(&mut self, tree: &mut UiTree, kind: u32) {
        self.start_rows(tree);
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

    /// `RecvNotice_UpdateExaminationProfile`.
    pub fn update_profile(&mut self, tree: &mut UiTree, items: &[ExaminationItem]) {
        self.start_rows(tree);
        for item in items {
            self.process_node(tree, item);
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
    fn process_node(&mut self, tree: &mut UiTree, item: &ExaminationItem) {
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
                let file = match item {
                    ExaminationItem::Image(file) => Some(*file),
                    ExaminationItem::Icon(layers) => layers.first().copied(),
                    _ => None,
                };
                if let Some(file) = file {
                    tree.set_image(element, file, 0);
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
            ExaminationItem::Countdown { appearance, .. } => {
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
                self.grow_next_y(tree, element);
            }
        }
    }

    /// `gmExaminationUI::ListenToMessage`: the field's close button hides it (the toolbar's
    /// panel children handle that), so only the shown state is followed here.
    pub fn message(&mut self, tree: &mut UiTree, _message: &UiMessage) {
        self.follow_field(tree);
    }
}
