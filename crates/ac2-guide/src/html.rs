//! Reads the AC2Hero mirror into a `Guide`: each page's content between the site template's
//! `<!-- header2 end-->` and `<!-- footer2 start-->` markers, as lines of text, links, images
//! and the map coordinates `coord::find` picks out of the text; the quests' header fields and the
//! item tables come from `facts`.

use std::collections::HashMap;
use std::path::Path;

use crate::coord;
use crate::facts::{self, ItemRow};
use crate::guide::{Guide, Image, Item, Kind, Line, Page, Span, Style};

const CONTENT_START: &str = "<!-- header2 end-->";
const CONTENT_END: &str = "<!-- footer2 start-->";

/// A page as read, before the guide joins them: its images and item rows by mirror path.
struct Parsed {
    page: Page,
    images: Vec<String>,
    items: Vec<ItemRow>,
}

/// Every content page under `root` (the mirror's `ac2hero.e5i5o.com` folder), sorted by kind and
/// title, with links between them resolved, the images they show loaded, and quest rewards
/// matched to the item tables.
pub fn read_mirror(root: &Path) -> std::io::Result<Guide> {
    let mut files = Vec::new();
    collect(root, root, &mut files)?;
    files.sort();
    let mut parsed = Vec::new();
    for rel in files {
        let bytes = std::fs::read(root.join(&rel))?;
        if let Some(p) = parse(&rel, &decode_text(&bytes)) {
            parsed.push(p);
        }
    }
    parsed.sort_by(|a, b| {
        let (a, b) = (&a.page, &b.page);
        (a.kind as u8, a.title.to_lowercase(), &a.path).cmp(&(
            b.kind as u8,
            b.title.to_lowercase(),
            &b.path,
        ))
    });

    let mut images: Vec<Image> = Vec::new();
    let mut image_index: HashMap<String, Option<u32>> = HashMap::new();
    let mut load = |path: &str| -> Option<u32> {
        *image_index.entry(path.to_string()).or_insert_with(|| {
            let bytes = std::fs::read(root.join(path))
                .or_else(|_| std::fs::read(root.join(percent_decode(path))))
                .ok()
                .filter(|b| Image::format(b).is_some())?;
            images.push(Image {
                path: path.to_string(),
                bytes: bytes.into(),
            });
            Some(images.len() as u32 - 1)
        })
    };
    let mut items = Vec::new();
    let mut pages = Vec::with_capacity(parsed.len());
    for (index, p) in parsed.into_iter().enumerate() {
        let global: Vec<Option<u32>> = p.images.iter().map(|path| load(path)).collect();
        let mut page = p.page;
        for line in &mut page.lines {
            line.spans.retain_mut(|span| match span {
                Span::Image { image } => match global[*image as usize] {
                    Some(g) => {
                        *image = g;
                        true
                    }
                    None => false,
                },
                _ => true,
            });
        }
        page.lines.retain(|l| !l.spans.is_empty());
        for row in p.items {
            items.push(Item {
                name: row.name,
                page: index as u32,
                icon: row.icon.as_deref().and_then(&mut load),
                note: row.note,
            });
        }
        pages.push(page);
    }
    match_rewards(&mut pages, &items);
    let index: HashMap<String, u32> = pages
        .iter()
        .enumerate()
        .map(|(i, p)| (p.path.clone(), i as u32))
        .collect();
    for span in pages
        .iter_mut()
        .flat_map(|p| &mut p.lines)
        .flat_map(|l| &mut l.spans)
    {
        if let Span::Missing { text, href } = span {
            let page = index
                .get(href.as_str())
                .or_else(|| index.get(&format!("{href}.html")))
                .or_else(|| index.get(&percent_decode(href)));
            if let Some(&page) = page {
                *span = Span::Page {
                    text: std::mem::take(text),
                    page,
                };
            }
        }
    }
    Ok(Guide {
        pages,
        items,
        images,
    })
}

/// Points each quest reward at the item table entry it names: the same name (letters and digits,
/// any case), else the longest item name the reward contains.
fn match_rewards(pages: &mut [Page], items: &[Item]) {
    let keys: Vec<String> = items.iter().map(|i| facts::key(&i.name)).collect();
    let exact: HashMap<&str, u32> = keys
        .iter()
        .enumerate()
        .rev()
        .map(|(i, k)| (k.as_str(), i as u32))
        .collect();
    for reward in pages
        .iter_mut()
        .filter_map(|p| p.quest.as_mut())
        .flat_map(|q| &mut q.rewards)
    {
        let k = facts::key(&reward.text);
        reward.item = exact.get(k.as_str()).copied().or_else(|| {
            keys.iter()
                .enumerate()
                .filter(|(_, ik)| ik.len() >= 8 && k.contains(ik.as_str()))
                .max_by_key(|(_, ik)| ik.len())
                .map(|(i, _)| i as u32)
        });
    }
}

fn collect(root: &Path, dir: &Path, out: &mut Vec<String>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            collect(root, &path, out)?;
        } else if path.extension().is_some_and(|e| e == "html") {
            if let Ok(rel) = path.strip_prefix(root) {
                let parts: Vec<_> = rel
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy())
                    .collect();
                out.push(parts.join("/"));
            }
        }
    }
    Ok(())
}

/// UTF-8 when it is valid, else Windows-1252 (the site's older pages).
fn decode_text(bytes: &[u8]) -> String {
    match std::str::from_utf8(bytes) {
        Ok(s) => s.replace('\u{feff}', ""),
        Err(_) => bytes.iter().map(|&b| cp1252(b)).collect(),
    }
}

fn cp1252(b: u8) -> char {
    const HIGH: [char; 32] = [
        '€', '\u{81}', '‚', 'ƒ', '„', '…', '†', '‡', 'ˆ', '‰', 'Š', '‹', 'Œ', '\u{8d}', 'Ž',
        '\u{8f}', '\u{90}', '‘', '’', '“', '”', '•', '–', '—', '˜', '™', 'š', '›', 'œ', '\u{9d}',
        'ž', 'Ÿ',
    ];
    match b {
        0x80..=0x9F => HIGH[usize::from(b - 0x80)],
        _ => char::from(b),
    }
}

/// One page, or `None` when it has no content block or nothing in it. Its `Span::Image`s index
/// the page's own image list (`read_mirror` makes them the guide's).
pub fn parse_page(rel: &str, html: &str) -> Option<Page> {
    parse(rel, html).map(|p| p.page)
}

fn parse(rel: &str, html: &str) -> Option<Parsed> {
    let start = html.find(CONTENT_START)? + CONTENT_START.len();
    let end = start + html[start..].find(CONTENT_END)?;
    let file = rel.rsplit('/').next().unwrap_or(rel);
    let kind = if rel.starts_with("quest/") {
        Kind::Quest
    } else if file.starts_with("npcsearch") {
        Kind::Npc
    } else if file.starts_with("mobsearch") {
        Kind::Monster
    } else {
        Kind::Guide
    };
    let dir = rel.rsplit_once('/').map_or("", |(d, _)| d);
    let content = &html[start..end];
    let (mut lines, images) = Lines::new(dir).run(content);
    if kind != Kind::Quest && kind != Kind::Guide {
        lines.retain(|l| {
            let t = line_text(l);
            !(t == "Results"
                || t.starts_with("You searched for:")
                || (t.starts_with("AC2hero") && t.ends_with("Database")))
        });
    }
    if lines.is_empty() {
        return None;
    }
    let query = file.split_once("?search=").map(|(_, q)| {
        let q = q.strip_suffix(".html").unwrap_or(q).replace('+', " ");
        percent_decode(&q).trim().to_string()
    });
    let title = query
        .filter(|q| !q.is_empty())
        .or_else(|| page_title(html))
        .unwrap_or_else(|| file.trim_end_matches(".html").to_string());
    let area = match kind {
        Kind::Quest => file
            .split_once('_')
            .map(|(a, _)| capitalise(a))
            .unwrap_or_default(),
        _ => lines
            .iter()
            .find_map(|l| {
                line_text(l)
                    .strip_prefix("Continent:")
                    .map(|a| a.trim().to_string())
            })
            .unwrap_or_default(),
    };
    let quest = (kind == Kind::Quest).then(|| facts::quest_facts(content));
    let items = if rel.starts_with("items/") && !rel[6..].contains('/') {
        facts::item_rows(content, dir)
    } else {
        Vec::new()
    };
    Some(Parsed {
        page: Page {
            path: rel.to_string(),
            kind,
            title,
            area,
            quest,
            lines,
        },
        images,
        items,
    })
}

fn page_title(html: &str) -> Option<String> {
    let at = html.find("id=\"page-title\"")?;
    let open = at + html[at..].find('>')? + 1;
    let close = open + html[open..].find("</")?;
    let t = decode_entities(&html[open..close]);
    let t = t.split_whitespace().collect::<Vec<_>>().join(" ");
    (!t.is_empty()).then_some(t)
}

fn capitalise(s: &str) -> String {
    let mut c = s.chars();
    c.next()
        .map(|f| f.to_uppercase().chain(c).collect())
        .unwrap_or_default()
}

pub fn line_text(line: &Line) -> String {
    line.spans
        .iter()
        .map(|s| match s {
            Span::Text { text, .. }
            | Span::Page { text, .. }
            | Span::Missing { text, .. }
            | Span::Coord { text, .. } => text.as_str(),
            Span::Image { .. } => "",
        })
        .collect()
}

/// `%XX` escapes decoded; malformed ones kept.
fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        let hex = (b[i] == b'%')
            .then(|| b.get(i + 1..i + 3))
            .flatten()
            .and_then(|h| u8::from_str_radix(std::str::from_utf8(h).ok()?, 16).ok());
        match hex {
            Some(v) => {
                out.push(v);
                i += 3;
            }
            None => {
                out.push(b[i]);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub(crate) fn decode_entities(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        rest = &rest[amp..];
        let semi = rest.bytes().take(10).position(|b| b == b';');
        let decoded = semi.and_then(|semi| {
            let name = &rest[1..semi];
            let c = match name {
                "nbsp" => ' ',
                "amp" => '&',
                "lt" => '<',
                "gt" => '>',
                "quot" => '"',
                "apos" => '\'',
                "rsquo" => '’',
                "lsquo" => '‘',
                "rdquo" => '”',
                "ldquo" => '“',
                "ndash" => '–',
                "mdash" => '—',
                "hellip" => '…',
                "eacute" => 'é',
                "copy" => '©',
                "reg" => '®',
                "trade" => '™',
                "deg" => '°',
                _ => {
                    let n = name.strip_prefix('#')?;
                    let v = match n.strip_prefix(['x', 'X']) {
                        Some(h) => u32::from_str_radix(h, 16).ok()?,
                        None => n.parse().ok()?,
                    };
                    char::from_u32(v)?
                }
            };
            Some((c, semi + 1))
        });
        match decoded {
            Some((c, len)) => {
                out.push(c);
                rest = &rest[len..];
            }
            None => {
                out.push('&');
                rest = &rest[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// Joins a link to the page's folder: the mirror path it names, fragment and `..` resolved.
pub(crate) fn resolve(dir: &str, href: &str) -> String {
    let href = href.split('#').next().unwrap_or("");
    let mut parts: Vec<&str> = if href.starts_with('/') {
        Vec::new()
    } else {
        dir.split('/').filter(|p| !p.is_empty()).collect()
    };
    for part in href.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            p => parts.push(p),
        }
    }
    parts.join("/")
}

/// The content block's tag soup turned into lines.
struct Lines<'a> {
    dir: &'a str,
    lines: Vec<Line>,
    spans: Vec<Span>,
    style: Style,
    bold: u32,
    skip: u32,
    /// The open link's target and text.
    link: Option<(String, String)>,
    space: bool,
    /// The images the page shows, by mirror path; `Span::Image` indexes this.
    images: Vec<String>,
}

impl<'a> Lines<'a> {
    fn new(dir: &'a str) -> Self {
        Self {
            dir,
            lines: Vec::new(),
            spans: Vec::new(),
            style: Style::Body,
            bold: 0,
            skip: 0,
            link: None,
            space: false,
            images: Vec::new(),
        }
    }

    fn run(mut self, html: &str) -> (Vec<Line>, Vec<String>) {
        let mut rest = html;
        while !rest.is_empty() {
            if let Some(comment) = rest.strip_prefix("<!--") {
                rest = comment.find("-->").map_or("", |e| &comment[e + 3..]);
            } else if rest.starts_with('<')
                && rest[1..].starts_with(|c: char| c.is_ascii_alphabetic() || c == '/')
            {
                let end = rest.find('>').unwrap_or(rest.len() - 1);
                self.tag(&rest[1..end]);
                rest = &rest[end + 1..];
            } else {
                let first = rest.chars().next().map_or(1, char::len_utf8);
                let end = rest[first..].find('<').map_or(rest.len(), |e| e + first);
                if self.skip == 0 {
                    self.text(&decode_entities(&rest[..end]));
                }
                rest = &rest[end..];
            }
        }
        self.flush();
        (self.lines, self.images)
    }

    fn tag(&mut self, inner: &str) {
        let closing = inner.starts_with('/');
        let inner = inner.trim_start_matches('/');
        let name_end = inner
            .find(|c: char| c.is_ascii_whitespace() || c == '/')
            .unwrap_or(inner.len());
        let name = inner[..name_end].to_ascii_lowercase();
        match name.as_str() {
            "script" | "style" | "form" | "select" | "textarea" => {
                self.skip = if closing {
                    self.skip.saturating_sub(1)
                } else {
                    self.skip + 1
                };
            }
            _ if self.skip > 0 => {}
            "strong" | "b" => {
                self.bold = if closing {
                    self.bold.saturating_sub(1)
                } else {
                    self.bold + 1
                }
            }
            "a" if closing => self.close_link(),
            "a" => {
                self.close_link();
                if let Some(href) = attr(inner, "href") {
                    if self.space && !self.spans.is_empty() {
                        self.space = false;
                        self.push_text(" ");
                    }
                    self.link = Some((href, String::new()));
                }
            }
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                self.flush();
                self.style = if closing { Style::Body } else { Style::Heading };
            }
            "li" => {
                self.flush();
                self.style = if closing { Style::Body } else { Style::Bullet };
            }
            "img" => {
                if let Some(src) = attr(inner, "src") {
                    let path = resolve(self.dir, &src);
                    if !(path.starts_with("images/")
                        || path.starts_with("js/")
                        || path.contains("://"))
                    {
                        let image = match self.images.iter().position(|p| *p == path) {
                            Some(i) => i,
                            None => {
                                self.images.push(path);
                                self.images.len() - 1
                            }
                        } as u32;
                        if self.space && !self.spans.is_empty() {
                            self.push_text(" ");
                        }
                        self.space = false;
                        self.spans.push(Span::Image { image });
                    }
                }
            }
            "td" | "th" if !closing && !self.spans.is_empty() => {
                self.space = false;
                self.push_text(" | ");
            }
            "p" | "div" | "br" | "ul" | "ol" | "table" | "tr" | "center" | "hr" | "blockquote"
            | "dl" | "dt" | "dd" => {
                self.flush();
                if name != "br" {
                    self.style = Style::Body;
                }
            }
            _ => {}
        }
    }

    fn text(&mut self, text: &str) {
        let mut out = String::new();
        for c in text.chars() {
            if c.is_whitespace() {
                self.space = true;
            } else {
                let started = !self.spans.is_empty()
                    || self.link.as_ref().is_some_and(|(_, t)| !t.is_empty())
                    || !out.is_empty();
                if self.space && started {
                    out.push(' ');
                }
                self.space = false;
                out.push(c);
            }
        }
        match &mut self.link {
            Some((_, t)) => t.push_str(&out),
            None => self.push_text(&out),
        }
    }

    fn push_text(&mut self, s: &str) {
        if s.is_empty() {
            return;
        }
        let bold = self.bold > 0;
        if let Some(Span::Text { text, bold: b }) = self.spans.last_mut() {
            if *b == bold {
                text.push_str(s);
                return;
            }
        }
        self.spans.push(Span::Text {
            text: s.to_string(),
            bold,
        });
    }

    fn close_link(&mut self) {
        let Some((href, text)) = self.link.take() else {
            return;
        };
        let text = text.trim().to_string();
        if text.is_empty() {
            return;
        }
        if href.is_empty() || href.starts_with('#') || href.starts_with("javascript:") {
            self.push_text(&text);
        } else if href.contains("://") || href.starts_with("mailto:") {
            self.spans.push(Span::Missing { text, href });
        } else {
            let href = resolve(self.dir, &href);
            self.spans.push(Span::Missing { text, href });
        }
    }

    /// Ends the line: its text spans split around coordinates, kept if it has any text.
    fn flush(&mut self) {
        self.close_link();
        self.space = false;
        let mut spans = Vec::new();
        for span in self.spans.drain(..) {
            match span {
                Span::Text { text, bold } => {
                    let mut at = 0;
                    for (range, c) in coord::find(&text) {
                        if range.start > at {
                            spans.push(Span::Text {
                                text: text[at..range.start].to_string(),
                                bold,
                            });
                        }
                        spans.push(Span::Coord {
                            text: text[range.clone()].to_string(),
                            at: c,
                        });
                        at = range.end;
                    }
                    if at < text.len() {
                        spans.push(Span::Text {
                            text: text[at..].to_string(),
                            bold,
                        });
                    }
                }
                other => spans.push(other),
            }
        }
        if let Some(Span::Text { text, .. }) = spans.last_mut() {
            let trimmed = text.trim_end().len();
            text.truncate(trimmed);
            if text.is_empty() {
                spans.pop();
            }
        }
        let line = Line {
            style: self.style,
            spans,
        };
        let image = line.spans.iter().any(|s| matches!(s, Span::Image { .. }));
        if image || !line_text(&line).trim().trim_matches('|').trim().is_empty() {
            self.lines.push(line);
        }
    }
}

/// An attribute's value: quoted with `"` or `'`, or bare.
fn attr(inner: &str, name: &str) -> Option<String> {
    let lower = inner.to_ascii_lowercase();
    let mut from = 0;
    while let Some(at) = lower[from..].find(name) {
        let at = from + at;
        from = at + name.len();
        let before_ok = at == 0 || lower.as_bytes()[at - 1].is_ascii_whitespace();
        let rest = lower[from..].trim_start();
        if !before_ok || !rest.starts_with('=') {
            continue;
        }
        let value_at = inner.len() - rest.len() + 1;
        let value = inner[value_at..].trim_start();
        let v = match value.chars().next()? {
            q @ ('"' | '\'') => value[1..].split(q).next()?,
            _ => value.split(|c: char| c.is_ascii_whitespace()).next()?,
        };
        return Some(decode_entities(v));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guide::MapCoord;

    fn page(rel: &str, body: &str) -> Page {
        let html = format!("<html><h2 id=\"page-title\">The &amp; Title</h2>{CONTENT_START}{body}{CONTENT_END}</html>");
        parse_page(rel, &html).expect("content")
    }

    #[test]
    fn quest_lines() {
        let p = page(
            "quest/knorr_x.html",
            "<h3><span>Quest Name: X</span></h3><p><strong>Start:</strong> see\n<a href=\"../search/npc.html?search=Bo+B\">Bo B</a> at 51.8S, 39.0E</p><ul><li>one&nbsp;two</li></ul><script>no</script>",
        );
        assert_eq!(p.kind, Kind::Quest);
        assert_eq!(p.title, "The & Title");
        assert_eq!(p.area, "Knorr");
        assert_eq!(p.lines.len(), 3);
        assert_eq!(p.lines[0].style, Style::Heading);
        assert_eq!(
            p.lines[1].spans,
            [
                Span::Text {
                    text: "Start:".into(),
                    bold: true
                },
                Span::Text {
                    text: " see ".into(),
                    bold: false
                },
                Span::Missing {
                    text: "Bo B".into(),
                    href: "search/npc.html?search=Bo+B".into()
                },
                Span::Text {
                    text: " at ".into(),
                    bold: false
                },
                Span::Coord {
                    text: "51.8S, 39.0E".into(),
                    at: MapCoord {
                        north: -51.8,
                        east: 39.0
                    }
                },
            ]
        );
        assert_eq!(p.lines[2].style, Style::Bullet);
        assert_eq!(line_text(&p.lines[2]), "one two");
    }

    #[test]
    fn npc_page() {
        let p = page(
            "search/npcsearchb297.html?search=Sia+Jorlorn.html",
            "<center>AC2hero NPC Database</center><center><form>Search: <input></form></center><h2>Results</h2><p><b>You searched for: </b>Sia Jorlorn<p></p><table><tr><td><span>Sia Jorlorn</span><br><strong>Coords:</strong> (51.8S, 39.0E) <p></p><strong>Continent:</strong> Knorr<br></td><td><a href=\"#\"></a></td></tr></table>",
        );
        assert_eq!(p.kind, Kind::Npc);
        assert_eq!(p.title, "Sia Jorlorn");
        assert_eq!(p.area, "Knorr");
        let texts: Vec<_> = p.lines.iter().map(line_text).collect();
        assert_eq!(
            texts,
            ["Sia Jorlorn", "Coords: (51.8S, 39.0E)", "Continent: Knorr"]
        );
    }

    #[test]
    fn resolves_links() {
        assert_eq!(resolve("quest", "../maps/a.png#top"), "maps/a.png");
        assert_eq!(resolve("", "quest/b.html"), "quest/b.html");
        assert_eq!(attr("a class=x HREF='y z'", "href").as_deref(), Some("y z"));
    }
}
