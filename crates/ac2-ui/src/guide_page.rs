//! A guide page laid out for the guide window's text area: rows of coloured runs, each run maybe a
//! link (another page, an item, a map coordinate), the pictures placed between rows, and the item
//! icons at the start of reward rows. Laying out needs only a width measure and the line height,
//! so it runs (and is tested) without the DATs.

use ac2_guide::{Guide, MapCoord, Span, Style};

/// What a press on a run does.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Action {
    Page(u32),
    Item(u32),
    Coord(MapCoord),
}

/// A run's colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Tone {
    Body,
    Heading,
    /// A field's name (`Requires`, `Starts`).
    Label,
    Link,
    Coord,
    /// A link the guide does not hold.
    Missing,
}

impl Tone {
    /// The run's colour: the text's own for the body, the gold of the book's titles for headings
    /// and coordinates, a pale blue for links.
    pub(crate) fn rgb(self) -> Option<[f32; 3]> {
        match self {
            Tone::Body => None,
            Tone::Heading => Some([0.93, 0.80, 0.45]),
            Tone::Label => Some([0.62, 0.58, 0.50]),
            Tone::Link => Some([0.56, 0.71, 1.0]),
            Tone::Coord => Some([0.96, 0.84, 0.40]),
            Tone::Missing => Some([0.55, 0.55, 0.55]),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Run {
    pub text: String,
    pub tone: Tone,
    pub action: Option<Action>,
}

/// One line of the text area.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct Row {
    pub runs: Vec<Run>,
}

impl Row {
    #[cfg(test)]
    pub(crate) fn text(&self) -> String {
        self.runs.iter().map(|r| r.text.as_str()).collect()
    }
}

/// A picture between rows: the guide's image, the first row it covers, and its place and size
/// in pixels from the text's left edge and that row's top.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Picture {
    pub image: u32,
    pub row: usize,
    pub x: i32,
    pub width: i32,
    pub height: i32,
}

/// A laid-out page: its rows, and the pictures and icons over them.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct Laid {
    pub rows: Vec<Row>,
    pub pictures: Vec<Picture>,
}

/// What laying out measures with.
pub(crate) struct Metrics<'a> {
    /// A string's width in pixels.
    pub width: &'a dyn Fn(&str) -> i32,
    pub line: i32,
    /// The text's usable width.
    pub wrap: i32,
    /// An image's size in pixels, from its header.
    pub image_size: &'a dyn Fn(u32) -> Option<(u32, u32)>,
}

/// The tallest a picture is drawn, in pixels.
const PICTURE_MAX_HEIGHT: i32 = 240;

/// Builds rows, wrapping runs at spaces to the metrics' width.
struct Writer<'a, 'm> {
    m: &'a Metrics<'m>,
    laid: Laid,
    row: Row,
    used: i32,
    /// Spaces a wrapped line starts with (a bullet's text lines up after its bullet).
    hang: String,
    /// The line was begun by a wrap and has only its hang so far: the space it broke at is
    /// dropped.
    wrapped: bool,
}

impl<'a, 'm> Writer<'a, 'm> {
    fn new(m: &'a Metrics<'m>) -> Self {
        Self {
            m,
            laid: Laid::default(),
            row: Row::default(),
            used: 0,
            hang: String::new(),
            wrapped: false,
        }
    }

    fn push_piece(&mut self, text: &str, tone: Tone, action: Option<Action>) {
        if let Some(last) = self.row.runs.last_mut() {
            if last.tone == tone && last.action == action {
                last.text.push_str(text);
                self.used += (self.m.width)(text);
                return;
            }
        }
        self.row.runs.push(Run {
            text: text.to_owned(),
            tone,
            action,
        });
        self.used += (self.m.width)(text);
    }

    /// Writes `text`, wrapping before a word that would pass the width. A link wraps whole
    /// (unless it is wider than a line).
    fn write(&mut self, text: &str, tone: Tone, action: Option<Action>) {
        if action.is_some() {
            let width = (self.m.width)(text);
            if self.used > 0 && self.used + width > self.m.wrap && width <= self.m.wrap {
                self.break_line();
            }
            if width <= self.m.wrap {
                self.push_piece(text, tone, action);
                return;
            }
        }
        for word in split_keep_spaces(text) {
            let width = (self.m.width)(word);
            if self.used > 0 && self.used + width > self.m.wrap && !word.trim().is_empty() {
                self.break_line();
            }
            // A line does not start with the space it broke at, and a space that would pass the
            // width is left out (the text element would wrap it onto a line of its own).
            let space = word.trim().is_empty();
            if space && (self.wrapped || self.used + width > self.m.wrap) {
                continue;
            }
            self.wrapped = false;
            self.push_piece(word, tone, action);
        }
    }

    fn break_line(&mut self) {
        self.laid.rows.push(std::mem::take(&mut self.row));
        self.used = 0;
        self.wrapped = true;
        if !self.hang.is_empty() {
            let hang = self.hang.clone();
            self.push_piece(&hang, Tone::Body, None);
        }
    }

    /// Ends the line being written, if it has anything.
    fn end_line(&mut self) {
        if !self.row.runs.is_empty() {
            self.laid.rows.push(std::mem::take(&mut self.row));
        }
        self.used = 0;
        self.hang.clear();
        self.wrapped = false;
    }

    fn blank(&mut self) {
        self.end_line();
        if self.laid.rows.last().is_some_and(|r| !r.runs.is_empty()) {
            self.laid.rows.push(Row::default());
        }
    }

    /// A picture on rows of its own, scaled to fit the width and [`PICTURE_MAX_HEIGHT`].
    fn picture(&mut self, image: u32) {
        let Some((w, h)) = (self.m.image_size)(image) else {
            return;
        };
        let (w, h) = (w.max(1) as f32, h.max(1) as f32);
        let scale = (self.m.wrap as f32 / w)
            .min(PICTURE_MAX_HEIGHT as f32 / h)
            .min(1.0);
        let (width, height) = ((w * scale) as i32, (h * scale) as i32);
        if width < 2 || height < 2 {
            return;
        }
        self.end_line();
        let row = self.laid.rows.len();
        self.laid.pictures.push(Picture {
            image,
            row,
            x: 0,
            width,
            height,
        });
        let rows = (height + self.m.line - 1) / self.m.line.max(1);
        for _ in 0..rows {
            self.laid.rows.push(Row::default());
        }
    }

    /// An icon at the start of the line being written, with the text after it.
    fn icon(&mut self, image: u32) {
        let size = self.m.line;
        let row = self.laid.rows.len();
        self.laid.pictures.push(Picture {
            image,
            row,
            x: self.used,
            width: size,
            height: size,
        });
        let space = (self.m.width)(" ").max(1);
        let pad = " ".repeat(((size + 4 + space - 1) / space) as usize);
        self.push_piece(&pad, Tone::Body, None);
    }
}

/// `text` split into words and the runs of spaces between them.
fn split_keep_spaces(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut space = None;
    for (i, c) in text.char_indices() {
        let is_space = c == ' ';
        if space.is_some_and(|s| s != is_space) {
            out.push(&text[start..i]);
            start = i;
        }
        space = Some(is_space);
    }
    if start < text.len() {
        out.push(&text[start..]);
    }
    out
}

/// The quest facts' rows: `Requires`, `Starts`, `Time limit`, `Resets`, their values lined up
/// after the longest name.
fn facts(w: &mut Writer, guide: &Guide, page: &ac2_guide::Page) {
    let Some(q) = &page.quest else {
        return;
    };
    let fields = [
        ("Requires", q.requirements.as_str()),
        ("Starts", q.start.as_str()),
        ("Time limit", q.time_limit.as_str()),
        ("Resets", q.reset.as_str()),
    ];
    let column = fields
        .iter()
        .filter(|(_, v)| !v.is_empty())
        .map(|(k, _)| (w.m.width)(k))
        .max()
        .unwrap_or(0)
        + (w.m.width)("    ");
    let space = (w.m.width)(" ").max(1);
    // In a narrow pane the names go above their values.
    let aligned = column * 3 <= w.m.wrap;
    for (name, value) in fields {
        if value.is_empty() {
            continue;
        }
        w.write(name, Tone::Label, None);
        if aligned {
            let pad = ((column - (w.m.width)(name)) / space).max(1) as usize;
            w.push_piece(&" ".repeat(pad), Tone::Body, None);
            w.hang = " ".repeat(((column + space - 1) / space) as usize);
        } else {
            w.end_line();
            w.write("  ", Tone::Body, None);
            w.hang = "  ".to_owned();
        }
        write_with_coords(w, value, Tone::Body);
        w.end_line();
    }
    if !q.rewards.is_empty() {
        w.blank();
        w.write("Rewards", Tone::Heading, None);
        w.end_line();
        for reward in &q.rewards {
            let item = reward
                .item
                .and_then(|i| guide.items.get(i as usize).map(|it| (i, it)));
            match item {
                Some((index, it)) => {
                    if let Some(icon) = it.icon {
                        w.icon(icon);
                    } else {
                        w.write("  ", Tone::Body, None);
                    }
                    w.write(&reward.text, Tone::Link, Some(Action::Item(index)));
                }
                None => {
                    w.write("  ", Tone::Body, None);
                    w.write(&reward.text, Tone::Body, None);
                }
            }
            w.end_line();
        }
    }
    w.blank();
}

/// Text in `tone` with the map coordinates in it made coordinate runs.
fn write_with_coords(w: &mut Writer, text: &str, tone: Tone) {
    let mut last = 0;
    for (range, at) in ac2_guide::coord::find(text) {
        if range.start > last {
            w.write(&text[last..range.start], tone, None);
        }
        w.write(&text[range.clone()], Tone::Coord, Some(Action::Coord(at)));
        last = range.end;
    }
    if last < text.len() {
        w.write(&text[last..], tone, None);
    }
}

/// The page `index` laid out: its title, the quest's facts and rewards, then its lines.
pub(crate) fn lay_out_page(guide: &Guide, index: u32, m: &Metrics) -> Laid {
    let mut w = Writer::new(m);
    let Some(page) = guide.pages.get(index as usize) else {
        return w.laid;
    };
    w.write(&page.title, Tone::Heading, None);
    w.end_line();
    let mut subtitle = page.kind.label().trim_end_matches('s').to_owned();
    if !page.area.is_empty() {
        subtitle = format!("{subtitle}, {}", page.area);
    }
    w.write(&subtitle, Tone::Label, None);
    w.blank();
    facts(&mut w, guide, page);
    let skipped = header_lines(page);
    for (i, line) in page.lines.iter().enumerate() {
        if skipped.contains(&i) {
            continue;
        }
        match line.style {
            Style::Heading => {
                w.blank();
                for span in &line.spans {
                    span_text(&mut w, span, Tone::Heading);
                }
                w.end_line();
            }
            Style::Bullet => {
                w.write("\u{2022} ", Tone::Coord, None);
                w.hang = " ".repeat(
                    (((w.m.width)("\u{2022} ") + (w.m.width)(" ") - 1) / (w.m.width)(" ").max(1))
                        as usize,
                );
                for span in &line.spans {
                    span_text(&mut w, span, Tone::Body);
                }
                w.end_line();
            }
            Style::Body => {
                for span in &line.spans {
                    span_text(&mut w, span, Tone::Body);
                }
                w.end_line();
            }
        }
    }
    w.end_line();
    w.laid
}

/// The site's header fields as a quest page's text writes them; the facts above show them.
const HEADER_LABELS: [&str; 6] = [
    "quest name",
    "requirements",
    "reward",
    "time limit",
    "reset timer",
    "start location",
];

/// The lines of a quest page's header block (from its first field to its last, the rewards
/// listed between them), which the facts and rewards already show; empty for other pages.
fn header_lines(page: &ac2_guide::Page) -> std::ops::Range<usize> {
    if page.quest.is_none() {
        return 0..0;
    }
    let labelled = |line: &ac2_guide::Line| {
        let text: String = line
            .spans
            .iter()
            .filter_map(|s| match s {
                Span::Text { text, .. } | Span::Page { text, .. } | Span::Coord { text, .. } => {
                    Some(text.as_str())
                }
                _ => None,
            })
            .collect();
        let text = text.trim_start().to_ascii_lowercase();
        HEADER_LABELS.iter().any(|label| text.starts_with(label))
    };
    // The block is near the top; a label far below it is the walkthrough's own.
    let near = page.lines.len().min(40);
    let first = page.lines[..near].iter().position(labelled);
    let last = page.lines[..near].iter().rposition(labelled);
    match (first, last) {
        (Some(first), Some(last)) => first..last + 1,
        _ => 0..0,
    }
}

fn span_text(w: &mut Writer, span: &Span, tone: Tone) {
    match span {
        Span::Text { text, .. } => write_with_coords(w, text, tone),
        Span::Page { text, page } => w.write(text, Tone::Link, Some(Action::Page(*page))),
        Span::Missing { text, .. } => w.write(text, Tone::Missing, None),
        Span::Coord { text, at } => w.write(text, Tone::Coord, Some(Action::Coord(*at))),
        Span::Image { image } => w.picture(*image),
    }
}

/// The item `index` laid out: its icon and name, its table and note, and the quests that give it.
pub(crate) fn lay_out_item(guide: &Guide, index: u32, quests: &[u32], m: &Metrics) -> Laid {
    let mut w = Writer::new(m);
    let Some(item) = guide.items.get(index as usize) else {
        return w.laid;
    };
    if let Some(icon) = item.icon {
        w.icon(icon);
    }
    w.write(&item.name, Tone::Heading, None);
    w.end_line();
    if let Some(table) = guide.pages.get(item.page as usize) {
        w.write(&table.title, Tone::Link, Some(Action::Page(item.page)));
        w.end_line();
    }
    w.blank();
    if !item.note.is_empty() {
        write_with_coords(&mut w, &item.note, Tone::Body);
        w.end_line();
        w.blank();
    }
    w.write(
        if quests.is_empty() {
            "No quest in the guide gives it."
        } else {
            "Given by"
        },
        Tone::Heading,
        None,
    );
    w.end_line();
    for &quest in quests {
        let Some(page) = guide.pages.get(quest as usize) else {
            continue;
        };
        let level = page.level().map_or(String::new(), |l| format!("{l:>3}  "));
        w.write(&level, Tone::Label, None);
        w.write(&page.title, Tone::Link, Some(Action::Page(quest)));
        w.end_line();
    }
    w.laid
}

/// The run of `row` under `x` pixels from the text's left edge.
pub(crate) fn run_at<'a>(row: &'a Row, x: i32, width: &dyn Fn(&str) -> i32) -> Option<&'a Run> {
    let mut left = 0;
    for run in &row.runs {
        let w = width(&run.text);
        if x >= left && x < left + w {
            return Some(run);
        }
        left += w;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use ac2_guide::{Item, Kind, Line, Page, QuestFacts, Reward};

    fn metrics<'a>(
        width: &'a dyn Fn(&str) -> i32,
        image_size: &'a dyn Fn(u32) -> Option<(u32, u32)>,
    ) -> Metrics<'a> {
        Metrics {
            width,
            line: 10,
            wrap: 160,
            image_size,
        }
    }

    fn guide() -> Guide {
        Guide {
            pages: vec![Page {
                path: "q.html".into(),
                kind: Kind::Quest,
                title: "Knights".into(),
                area: String::new(),
                quest: Some(QuestFacts {
                    level: Some(20),
                    requirements: "Level 20+".into(),
                    start: "Tima, 14.3N, 3.3E".into(),
                    rewards: vec![
                        Reward {
                            text: "Boots".into(),
                            item: Some(0),
                        },
                        Reward {
                            text: "50,000 XP".into(),
                            item: None,
                        },
                    ],
                    ..Default::default()
                }),
                lines: vec![
                    Line {
                        style: Style::Body,
                        spans: vec![Span::Text {
                            text: "one two three four five six".into(),
                            bold: false,
                        }],
                    },
                    Line {
                        style: Style::Body,
                        spans: vec![Span::Image { image: 1 }],
                    },
                ],
            }],
            items: vec![Item {
                name: "Boots".into(),
                page: 0,
                icon: Some(0),
                note: String::new(),
            }],
            images: Vec::new(),
        }
    }

    #[test]
    fn a_quest_page_wraps_links_its_rewards_and_places_its_pictures() {
        let width = |s: &str| s.chars().count() as i32 * 10;
        let size = |image: u32| (image == 1).then_some((400, 200)).or(Some((32, 32)));
        let m = metrics(&width, &size);
        let laid = lay_out_page(&guide(), 0, &m);
        let texts: Vec<String> = laid.rows.iter().map(Row::text).collect();
        assert_eq!(texts[0], "Knights");
        // Every row fits the width.
        assert!(
            laid.rows.iter().all(|r| width(&r.text()) <= 160),
            "{texts:?}"
        );
        // The start's coordinate is a coordinate run.
        let coord = laid
            .rows
            .iter()
            .flat_map(|r| &r.runs)
            .find(|r| r.tone == Tone::Coord)
            .expect("a coordinate run");
        assert!(matches!(coord.action, Some(Action::Coord(_))));
        // The reward with an item links it and has its icon; the other is plain.
        let boots = laid
            .rows
            .iter()
            .position(|r| r.runs.iter().any(|run| run.action == Some(Action::Item(0))))
            .unwrap();
        assert!(laid
            .pictures
            .iter()
            .any(|p| p.row == boots && p.width == 10));
        assert!(texts.iter().any(|t| t.trim() == "50,000 XP"));
        // The picture is scaled to the width, on rows of its own.
        let picture = laid.pictures.iter().find(|p| p.image == 1).unwrap();
        assert_eq!((picture.width, picture.height), (160, 80));
        assert!(laid.rows.len() >= picture.row + 8);
        assert!(laid.rows[picture.row..picture.row + 8]
            .iter()
            .all(|r| r.runs.is_empty()));
        // A press finds the run under it.
        let row = &laid.rows[boots];
        let x = width(&row.runs[0].text) + 5;
        assert_eq!(
            run_at(row, x, &width).unwrap().action,
            Some(Action::Item(0))
        );
    }

    #[test]
    fn an_item_lists_the_quests_that_give_it() {
        let width = |s: &str| s.chars().count() as i32 * 10;
        let size = |_: u32| Some((32, 32));
        let m = metrics(&width, &size);
        let laid = lay_out_item(&guide(), 0, &[0], &m);
        assert!(laid.rows.iter().any(|r| r
            .runs
            .iter()
            .any(|run| run.action == Some(Action::Page(0)) && run.text == "Knights")));
    }
}
