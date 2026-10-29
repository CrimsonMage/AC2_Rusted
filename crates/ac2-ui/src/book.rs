//! `gmBookUI`: the book panel (`Book` in the gameplay layout, `0x41000009`). A notice from the
//! player's script (`CM_Usage::SendNotice_UseBook`, the book's `UseBook` usage) shows the book's
//! title and text: the text is split into pages (`<BR>` ends a page, `<H>...</H>` is the page's
//! header), each page padded to the client area's `lines_per_page` lines, and the page number
//! reads `current / last`. The page turns, the slider, the close and copy buttons and the help
//! button are the book's `ListenToMessage`.
//!
//! The panel is kept hidden in the HUD and shown by the notice (`gmBookUI::Show`); retail creates
//! and destroys its elements on each show, here they stay in the tree.
//!
//! Retail never ghosts the `BookSliderLeft`/`Right`/`Copy` buttons: `SetMode` only shows or hides
//! the controls. The help page is returned as [`BookRequest::OpenHelp`] for the caller to open in
//! the help window; the copy button's text goes to the OS clipboard.

use std::sync::Arc;

use ac2_weenie::{GameplayNotice, SessionEvent};

use crate::element::UiTree;
use crate::layout::LayoutDesc;
use crate::mouse::UiMessage;
use crate::scrollbar::MSG_NEW_POSITION;
use crate::skill_tree::SkillVar;
use crate::token::{tokenize, MSG_BUTTON_RELEASE_ON, MSG_MOUSE_PRESS_TOP};
use crate::ui_string::{UiString, UiStrings};

/// The book's `ID_Book_PageNumberText` string (the page number's text in the UI table).
const PAGE_NUMBER_STRING: &str = "ID_Book_PageNumberText";
/// The variable the current page fills (`CurrentPageToken`).
const CURRENT_PAGE: &str = "CURRENT";
/// The variable the last page fills (`LastPageToken`).
const LAST_PAGE: &str = "LAST";
/// `gmBookUI::ParseText`: the last page is capped here.
const MAX_PAGES: u32 = 1000;
/// `gmBookUI::TruncateText`'s limit is the header field's width less this.
const HEADER_MARGIN: i32 = 19;
/// `gmBookUI::Reset`'s state (`mode_default`).
const MODE_DEFAULT: &str = "Dialog";
/// `gmBookUI::RecvNotice_UseBook`: the foreground image's draw mode (`AddImageToState(.., 2, ..)`).
const FOREGROUND_DRAW_MODE: u32 = 2;

/// What the book asks of the client when its buttons are pressed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BookRequest {
    /// `gmBookUI::ListenToMessage` (`BookHelpButton`): `KeyStone::OpenHelp(ID_Help_Book)`.
    OpenHelp,
    /// `gmBookUI::SendBookToClipboard`: the book's text for the clipboard.
    Copy(String),
}

/// `gmBookUI` on the frame: the use notices in `events` show the book
/// (`RecvNotice_UseBook`), then the book's `messages` run [`BookUi::message`]. Returns the
/// requests in message order, except the copy button's: `gmBookUI::SendBookToClipboard` hands
/// the book's text to the clipboard here (`Device::SendStringToClipboard`).
pub fn book_pass(
    book: &mut BookUi,
    tree: &mut UiTree,
    strings: &dyn UiStrings,
    events: &[SessionEvent],
    messages: &[UiMessage],
) -> Vec<BookRequest> {
    for event in events {
        if let SessionEvent::Notice(GameplayNotice::UseBook {
            show_controls,
            mode,
            title,
            text,
            image,
        }) = event
        {
            book.recv_use_book(tree, strings, *show_controls, *mode, title, text, *image);
        }
    }
    let mut requests = Vec::new();
    for message in messages {
        match book.message(tree, strings, message) {
            Some(BookRequest::Copy(text)) => {
                tree.send_string_to_clipboard(&text);
            }
            Some(request) => requests.push(request),
            None => {}
        }
    }
    requests
}

/// `gmBookUI`'s state over its elements.
#[derive(Debug, Clone)]
pub struct BookUi {
    /// `m_pBookField`: the `Book` element.
    field: usize,
    /// `m_pTitle`: `Panel_Title_Book`.
    title: Option<usize>,
    /// `m_pClientAreaText`: `BookClientAreaText`, the page's text.
    text: Option<usize>,
    /// `m_pClientAreaTextHeader`: `BookClientAreaTextHeader`, the page headers.
    header: Option<usize>,
    /// `m_pSlider`: `BookSliderBar`.
    slider: Option<usize>,
    /// `m_pPageNumberDisplay`: `PageNumberText`.
    page_number: Option<usize>,
    /// `BookForeground`: the page image, clicked to turn the page.
    foreground: Option<usize>,
    /// `BookClientArea`: the state `SetMode` sets on the page.
    client_area: Option<usize>,
    /// `BookBottom`: the page controls (`ID_BookControls`).
    controls: Option<usize>,
    /// `m_currentPage` (from 1).
    current: u32,
    /// `m_lastPage`.
    last: u32,
    /// `m_linesPerPage`: the client area's `lines_per_page` attribute.
    lines_per_page: i32,
    /// `m_showControls`: the page controls and the slider are live.
    show_controls: bool,
    /// The framework's shown state (`UIFramework::Show`).
    shown: bool,
}

impl BookUi {
    /// `gmBookUI::CreateManagedElements`: the `Book` element from the gameplay layout, its
    /// children found, hidden until a notice shows it. `None` when the layout has no `Book`.
    pub fn create(tree: &mut UiTree, gameplay: &Arc<LayoutDesc>) -> Option<Self> {
        let field = tree.create_nested_element(gameplay, tokenize("Book"))?;
        Self::from_field(tree, field)
    }

    /// Another `Book` made as its own HUD root under the client's id `id`, so the window moves and
    /// keeps its place like any other window (the help window, the Tracker, the guide and the map
    /// layers reuse the book's frame and text area this way). A copy of the layout's top-level
    /// `Book`; each copy's place is kept locally under its own id, and none uploads a location to
    /// the server (`save_loc` cleared: the book's own location is the retail one, and
    /// `UISaveLocations` lists a window once). `None` when the layout has no `Book`.
    pub fn create_window(tree: &mut UiTree, gameplay: &Arc<LayoutDesc>, id: &str) -> Option<Self> {
        let mut desc = (**gameplay.element(tokenize("Book"))?).clone();
        desc.id = tokenize(id);
        let field = tree.create_root_from(Arc::clone(gameplay), Arc::new(desc))?;
        tree.set_bool(field, crate::token::SAVE_LOC, false);
        Self::from_field(tree, field)
    }

    fn from_field(tree: &mut UiTree, field: usize) -> Option<Self> {
        tree.set_visible(field, false);
        let find = |tree: &UiTree, name: &str| tree.child_recursive(field, tokenize(name));
        let text = find(tree, "BookClientAreaText");
        let lines_per_page = text
            .and_then(|t| tree.get(t))
            .and_then(|e| e.attribute(tokenize("lines_per_page")))
            .and_then(|v| match v {
                crate::layout::AttributeValue::Int(n) => Some(*n),
                _ => None,
            })
            .unwrap_or(0);
        Some(Self {
            field,
            title: find(tree, "Panel_Title_Book"),
            text,
            header: find(tree, "BookClientAreaTextHeader"),
            slider: find(tree, "BookSliderBar"),
            page_number: find(tree, "PageNumberText"),
            foreground: find(tree, "BookForeground"),
            client_area: find(tree, "BookClientArea"),
            controls: find(tree, "BookBottom"),
            current: 1,
            last: 1,
            lines_per_page,
            show_controls: false,
            shown: false,
        })
    }

    /// The `Book` element.
    pub fn field(&self) -> usize {
        self.field
    }

    /// Whether the book is shown (`UIFramework::IsShown`).
    pub fn is_shown(&self) -> bool {
        self.shown
    }

    /// The current page, from 1 (`m_currentPage`).
    pub fn current_page(&self) -> u32 {
        self.current
    }

    /// The last page (`m_lastPage`).
    pub fn last_page(&self) -> u32 {
        self.last
    }

    /// `gmBookUI::Show`: the framework's shown state; the element follows it.
    pub fn show(&mut self, tree: &mut UiTree, shown: bool) {
        self.shown = shown;
        tree.set_visible(self.field, shown);
    }

    /// `gmBookUI::Reset`: page 1 of 1, the texts and the foreground image cleared, the default
    /// state set.
    pub fn reset(&mut self, tree: &mut UiTree) {
        self.current = 1;
        self.last = 1;
        for element in [self.title, self.text, self.header].into_iter().flatten() {
            tree.clear_text(element);
        }
        if let Some(foreground) = self.foreground {
            tree.clear_image(foreground);
        }
        self.set_mode(tree, tokenize(MODE_DEFAULT));
    }

    /// `gmBookUI::SetMode`: the page's elements take the state `mode`; the controls show when
    /// the book has them.
    pub fn set_mode(&mut self, tree: &mut UiTree, mode: u32) {
        for element in [self.client_area, self.text, self.header]
            .into_iter()
            .flatten()
        {
            tree.set_state(element, mode);
        }
        if let Some(controls) = self.controls {
            tree.set_visible(controls, self.show_controls);
        }
    }

    /// `gmBookUI::RecvNotice_UseBook`: the book is shown with `title` and the pages of `text`
    /// (`gmBookUI::ParseText`), the state `mode`, the page controls when `show_controls`, and
    /// the page image `image` (a DID; 0 for none) on the foreground.
    pub fn recv_use_book(
        &mut self,
        tree: &mut UiTree,
        strings: &dyn UiStrings,
        show_controls: bool,
        mode: u32,
        title: &str,
        text: &str,
        image: u32,
    ) {
        self.show(tree, true);
        self.reset(tree);
        self.show_controls = show_controls;
        if let Some(title_element) = self.title {
            tree.set_text(title_element, title);
        }
        self.set_mode(tree, mode);
        if !text.is_empty() {
            self.paginate(tree, text);
        }
        if let Some(foreground) = self.foreground {
            tree.clear_image(foreground);
            if image != 0 {
                tree.set_image(foreground, image, FOREGROUND_DRAW_MODE);
                tree.set_visible(foreground, true);
            }
        }
        self.update_book(tree, strings);
    }

    /// `gmBookUI::UpdateBook`: the page number and slider when the controls show, and the text
    /// and header scrolled to the current page (`SetViewY`, one page's height per page).
    pub fn update_book(&mut self, tree: &mut UiTree, strings: &dyn UiStrings) {
        if self.show_controls {
            if let Some(number) = self.page_number {
                let label = UiString::ui(
                    tokenize(PAGE_NUMBER_STRING),
                    vec![
                        (
                            tokenize(CURRENT_PAGE),
                            SkillVar::Int(i64::from(self.current)),
                        ),
                        (tokenize(LAST_PAGE), SkillVar::Int(i64::from(self.last))),
                    ],
                );
                tree.set_text(number, &strings.render_string(&label));
            }
            self.sync_slider(tree);
        }
        for element in [self.text, self.header].into_iter().flatten() {
            let height = tree.get(element).map_or(0, |e| e.rect().height());
            let y = height * (self.current as i32 - 1);
            tree.set_text_view_y(element, y);
        }
    }

    /// `gmBookUI::SynchSliderBar`: the slider at `(current - 1) / (last - 1)`, 0 for one page.
    pub fn sync_slider(&mut self, tree: &mut UiTree) {
        let Some(slider) = self.slider else {
            return;
        };
        let position = if self.last > 1 {
            (self.current as f32 - 1.0) / (self.last as f32 - 1.0)
        } else {
            0.0
        };
        tree.scrollbar_set_position(slider, position);
    }

    /// `gmBookUI::PreviousPage`: back one page if there is one.
    pub fn previous_page(&mut self) -> bool {
        if self.current > 1 {
            self.current -= 1;
            true
        } else {
            false
        }
    }

    /// `gmBookUI::NextPage`: forward one page if there is one.
    pub fn next_page(&mut self) -> bool {
        if self.current < self.last {
            self.current += 1;
            true
        } else {
            false
        }
    }

    /// `gmBookUI::SetPageFromSlider`: `s = (last - 1) * position` (a float), and the page is
    /// `s + 2` when its fraction is over a half, else `s + 1`, truncated (`_ftol2` of the x87 sum):
    /// the 1-based page nearest the slider, a half rounding down.
    pub fn set_page_from_slider(&mut self, tree: &UiTree) {
        let Some(slider) = self.slider else {
            return;
        };
        let position = tree.scrollbar_position(slider);
        let scaled = f64::from((self.last.wrapping_sub(1) as f32) * position);
        let step = if scaled - scaled.floor() > 0.5 {
            2.0
        } else {
            1.0
        };
        self.current = (scaled + step) as u32;
    }

    /// `gmBookUI::ParseText`: the book's text laid out into pages. Returns whether it laid
    /// anything out (`lines_per_page` is positive). Sets `last` to the page count.
    fn paginate(&mut self, tree: &mut UiTree, book: &str) -> bool {
        let (Some(text), Some(header)) = (self.text, self.header) else {
            return false;
        };
        tree.clear_text(text);
        tree.clear_text(header);
        self.last = 1;
        if self.lines_per_page <= 0 {
            return false;
        }
        let lines = self.lines_per_page as usize;
        // `(textField right - left) - 19`: the box's width without its one-pixel inclusive edge.
        let header_limit = tree.get(header).map_or(0, |e| e.rect().width() - 1) - HEADER_MARGIN;
        let mut pages: u32 = 0;
        for (heading, body) in segments(book) {
            // `header != "\n"`: a page with a header is set off by a blank line.
            if heading.is_some() {
                tree.append_text(text, "\n");
            }
            tree.append_text(text, &body);
            let total = tree.text_line_count(text);
            let remainder = total % lines;
            let filler = if remainder != 0 { lines - remainder } else { 0 };
            tree.append_text(text, &"\n".repeat(filler));
            let heading_line = heading.map_or_else(|| "\n".to_owned(), |h| format!("{h}\n"));
            let shown = truncate_to_width(tree, header, &heading_line, header_limit);
            tree.append_text(header, &shown);
            let now = ((filler + total) / lines) as u32;
            let these = now.saturating_sub(pages);
            if these > 1 {
                tree.append_text(header, &"\n".repeat(these as usize - 1));
            }
            pages += these;
        }
        tree.text_line_count(text);
        self.last = pages.clamp(1, MAX_PAGES);
        true
    }

    /// `gmBookUI::ListenToMessage`: the close button hides the book (also when the book is not
    /// shown); while shown, the foreground turns the page (the last page closes it), the help
    /// button asks for the help page, and with the controls the slider and its buttons turn the
    /// page and the copy button copies the text. Returns the request, if any.
    pub fn message(
        &mut self,
        tree: &mut UiTree,
        strings: &dyn UiStrings,
        message: &UiMessage,
    ) -> Option<BookRequest> {
        // The book and the help window both carry these element names: only this book's own
        // elements answer its messages.
        if !self.owns(tree, message.element) {
            return None;
        }
        let id = message.element_id;
        if id == tokenize("BookCloseButton") && message.message == MSG_BUTTON_RELEASE_ON {
            self.show(tree, false);
            return None;
        }
        if !self.shown {
            return None;
        }
        if id == tokenize("BookForeground") && message.message == MSG_MOUSE_PRESS_TOP {
            if self.next_page() {
                self.update_book(tree, strings);
            } else {
                self.show(tree, false);
            }
            return None;
        }
        if id == tokenize("BookHelpButton") && message.message == MSG_BUTTON_RELEASE_ON {
            return Some(BookRequest::OpenHelp);
        }
        if !self.show_controls {
            return None;
        }
        if id == tokenize("BookSliderLeft") && message.message == MSG_BUTTON_RELEASE_ON {
            self.previous_page();
            self.update_book(tree, strings);
        } else if id == tokenize("BookSliderRight") && message.message == MSG_BUTTON_RELEASE_ON {
            self.next_page();
            self.update_book(tree, strings);
        } else if message.element.is_some()
            && message.element == self.slider
            && message.message == MSG_NEW_POSITION
        {
            self.set_page_from_slider(tree);
            self.update_book(tree, strings);
        } else if id == tokenize("BookCopyButton") && message.message == MSG_BUTTON_RELEASE_ON {
            return Some(BookRequest::Copy(self.clipboard_text(tree)));
        }
        None
    }

    /// Whether `element` is this book's field or one of its children. A message with no element
    /// is global (its element id is 0), so no book owns it and no name in it matches.
    fn owns(&self, tree: &UiTree, element: Option<usize>) -> bool {
        let Some(element) = element else {
            return true;
        };
        let mut current = Some(element);
        while let Some(index) = current {
            if index == self.field {
                return true;
            }
            current = tree.get(index).and_then(|e| e.parent_index());
        }
        false
    }

    /// `gmBookUI::SendBookToClipboard`: the title, a blank line (`L"\n\n"`, Legions 0x9cbb28) and the
    /// page text, with the markup
    /// as the clipboard shows it (`<H>` dropped, `</H>` a newline, `<BR>` three newlines).
    fn clipboard_text(&self, tree: &UiTree) -> String {
        let element_text = |index: Option<usize>| {
            index
                .and_then(|i| tree.get(i))
                .and_then(|e| e.text())
                .unwrap_or_default()
        };
        let title = element_text(self.title);
        let body = element_text(self.text);
        let mut book = if title.is_empty() {
            String::new()
        } else {
            format!("{title}\n\n")
        };
        book.push_str(&body);
        if let Some(tail) = book.find("<T>") {
            book = book[tail + 3..].to_owned();
        }
        book.replace("<H>", "")
            .replace("</H>", "\n")
            .replace("<BR>", "\n\n\n")
    }
}

/// The book's pages as `(header, body)`: each `<BR>`-ended piece, split at its `<H>...</H>`
/// header (the header's text, the rest of the piece after the `</H>` and its newline is the
/// body). A piece without a closed header is all body. Each body is trimmed at the end and
/// gets a newline, as `ParseText` appends one.
pub(crate) fn segments(book: &str) -> Vec<(Option<String>, String)> {
    let mut pieces: Vec<&str> = book.split("<BR>").collect();
    if pieces.last().is_some_and(|tail| tail.is_empty()) {
        pieces.pop();
    }
    pieces
        .into_iter()
        .map(|piece| {
            let (heading, rest) = match (piece.find("<H>"), piece.find("</H>")) {
                (Some(open), Some(close)) if close > open => {
                    let heading = piece[open + 3..close].trim_end().to_owned();
                    let mut after = close + 4;
                    if piece[after..].starts_with('\n') {
                        after += 1;
                    }
                    (Some(heading), &piece[after..])
                }
                _ => (None, piece),
            };
            (heading, format!("{}\n", rest.trim_end()))
        })
        .collect()
}

/// `gmBookUI::TruncateText`: the text plus a space (`L" "`, Legions 0x9cbaf0: no ellipsis) is
/// measured; when it is wider than `limit` (a positive limit), characters come off the end until
/// the text plus the space fits, and the text keeps that space. A text that fits is unchanged.
fn truncate_to_width(tree: &UiTree, field: usize, text: &str, limit: i32) -> String {
    let width_of = |shown: &str| {
        tree.text_size_with_margins(field, &format!("{shown} "))
            .map_or(0, |(w, _)| w)
    };
    if limit <= 0 || width_of(text) <= limit {
        return text.to_owned();
    }
    let mut kept: Vec<char> = text.chars().collect();
    loop {
        kept.pop();
        let shown: String = kept.iter().collect();
        if width_of(&shown) <= limit || kept.is_empty() {
            return format!("{shown} ");
        }
    }
}
