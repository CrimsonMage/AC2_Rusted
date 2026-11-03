//! Reads the AC2 Vault mirror (the merged `ac2vault.ign.com` tree, pages saved under their old
//! URLs with `?` written `@`) into a `Guide` of the same shape as the AC2Hero one, so the guide
//! window shows it unchanged:
//!
//! - Quests: the quest database (`dir=quests/db&content=view&id=N`), facts from its table.
//! - Places: the atlas (`dir=cartography/atlas&content=view&id=N`), by landmass and type.
//! - Monsters: the bestiary (`dir=beasts&content=detail&section=creature&id=N`), by level band.
//! - Guides: the player guides, key map, glyph vaults, monthly event guides, spell and race
//!   lists, and one table page per trophy type; site pages (PC tuning, forums, screenshots, site
//!   news, interviews) are left out.
//! - Items: the trophy database, each under its type's table page.
//!
//! A record saved more than once (`index.php`/`index.html`, double-escaped queries, several
//! crawls) is read once, from its largest copy. Every page's content sits between the IGN
//! template's ad block and its forum or network box.

use std::collections::HashMap;
use std::path::Path;

use crate::facts::{self, ItemRow};
use crate::guide::{Guide, Kind, Line, Page, QuestFacts, Reward, Span, Style};
use crate::html::{self, Parsed};

/// The guide sections taken, by the URL's `dir`: their heading in the Guides tab, and whether
/// the section's own index page (no `content`) is taken too.
const GUIDE_DIRS: [(&str, &str, bool); 5] = [
    ("guides", "Guides", false),
    ("guides/keymap", "Keys and commands", false),
    ("vault", "Glyph vaults", true),
    ("spells", "Spells and uniques", false),
    ("races", "Races and skills", true),
];

/// Billy Mountain's spell and uniques pages all call themselves "SPELLS"; their names, from
/// the list's own menu (spells: weapon, armor, shield; uniques: weapon, armor, shield, then one
/// page per weapon family).
const SPELL_TITLES: [(&str, &str); 21] = [
    ("index", "Spells and uniques"),
    ("ac2s-w", "Spells: weapon"),
    ("ac2s-a", "Spells: armor"),
    ("ac2s-s", "Spells: shield"),
    ("ac2r-w", "Uniques: weapon"),
    ("ac2r-a", "Uniques: armor"),
    ("ac2r-s", "Uniques: shield"),
    ("rare-jug", "Uniques: martial melee"),
    ("rare-stf", "Uniques: magic staves"),
    ("rare-swd", "Uniques: bladestaves"),
    ("rare-rok", "Uniques: boulders"),
    ("rare-bow", "Uniques: bows"),
    ("rare-ces", "Uniques: cesta"),
    ("rare-drm", "Uniques: drums"),
    ("rare-fla", "Uniques: flails"),
    ("rare-ham", "Uniques: hammers"),
    ("rare-han", "Uniques: handblades"),
    ("rare-hiv", "Uniques: hives"),
    ("rare-spr", "Uniques: spears"),
    ("rare-via", "Uniques: vials"),
    ("rare-wre", "Uniques: wrenches"),
];

/// Pages of the taken sections that are not about the game.
const LEFT_OUT: [&str; 1] = ["guides/optimize"];

/// What a Vault URL names.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Record {
    Quest(u32),
    Place(u32),
    Creature(u32),
    Trophy(u32),
    /// A guide page: its section (`dir`) and `content` (empty for the section's own page).
    Guide(String, String),
}

/// The record a page file or link names, if it is one the guide takes.
fn record(href: &str) -> Option<Record> {
    let href = href.trim();
    let href = ["http://ac2vault.ign.com/", "https://ac2vault.ign.com/"]
        .iter()
        .find_map(|host| strip_prefix_ignore_case(href, host))
        .unwrap_or(href);
    let href = href.trim_start_matches("./").trim_start_matches('/');
    let href = href.split('#').next().unwrap_or("");
    let split = href.find(['?', '@'])?;
    let (file, query) = (&href[..split], &href[split + 1..]);
    if !(file.eq_ignore_ascii_case("index.html") || file.eq_ignore_ascii_case("index.php")) {
        return None;
    }
    let mut fields: HashMap<String, String> = HashMap::new();
    for pair in query.replace("&amp;", "&").split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            fields
                .entry(k.to_ascii_lowercase())
                .or_insert_with(|| unescape(v));
        }
    }
    let get = |k: &str| fields.get(k).map_or("", String::as_str);
    let dir = get("dir").trim_matches('/').to_ascii_lowercase();
    let content = get("content").trim_matches('/').to_string();
    let id = || get("id").parse::<u32>().ok();
    match (dir.as_str(), content.as_str()) {
        ("quests/db", "view") => id().map(Record::Quest),
        ("cartography/atlas", "view") => id().map(Record::Place),
        ("beasts", "detail") => match get("section") {
            "creature" => id().map(Record::Creature),
            "trophy" => id().map(Record::Trophy),
            _ => None,
        },
        (d, c) if d.starts_with("eventguide/") && !d[11..].contains('/') => {
            Some(Record::Guide(dir.clone(), c.to_ascii_lowercase()))
        }
        (d, c) => {
            let &(_, _, index) = GUIDE_DIRS.iter().find(|(gd, _, _)| *gd == d)?;
            let c = c.to_ascii_lowercase();
            if (c.is_empty() && !index) || LEFT_OUT.contains(&format!("{d}/{c}").as_str()) {
                return None;
            }
            Some(Record::Guide(dir.clone(), c))
        }
    }
}

fn strip_prefix_ignore_case<'a>(s: &'a str, prefix: &str) -> Option<&'a str> {
    let head = s.get(..prefix.len())?;
    head.eq_ignore_ascii_case(prefix)
        .then(|| &s[prefix.len()..])
}

/// A query value with its escapes undone, however many times the site escaped it (`%252F`).
fn unescape(v: &str) -> String {
    let mut v = v.replace('+', " ");
    for _ in 0..3 {
        let next = html::percent_decode(&v);
        if next == v {
            break;
        }
        v = next;
    }
    v
}

/// The guide from the mirror at `root` (the merged `ac2vault.ign.com` tree).
pub fn read_vault(root: &Path) -> std::io::Result<Guide> {
    // The largest saved copy of each record.
    let mut best: HashMap<Record, (u64, String)> = HashMap::new();
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(rec) = record(&name) else { continue };
        let len = entry.metadata()?.len();
        let slot = best.entry(rec).or_insert((0, String::new()));
        if len > slot.0 || (len == slot.0 && name < slot.1) {
            *slot = (len, name);
        }
    }
    let mut records: Vec<(Record, String)> = best.into_iter().map(|(r, (_, f))| (r, f)).collect();
    records.sort();

    let mut parsed = Vec::new();
    let mut trophies: Vec<Trophy> = Vec::new();
    for (rec, file) in records {
        let text = html::decode_text(&std::fs::read(root.join(&file))?);
        let Some(content) = content(&text) else {
            continue;
        };
        let content = local_links(content);
        if let Record::Trophy(_) = rec {
            trophies.extend(trophy(&file, &content));
        } else if let Some(p) = page(&rec, &file, &content) {
            parsed.push(p);
        }
    }
    parsed.extend(trophy_tables(trophies));

    let read_image = |path: &str| {
        let decoded = html::percent_decode(path);
        let mut tries = vec![path.to_string(), decoded.clone()];
        // `vnmedia.ign.com/ac2vault.ign.com/x` was the site's own `x` on the media host.
        for p in [path, decoded.as_str()] {
            if let Some(rest) = p.strip_prefix("_ext/vnmedia.ign.com/ac2vault.ign.com/") {
                tries.push(rest.to_string());
            }
        }
        tries.iter().find_map(|p| std::fs::read(root.join(p)).ok())
    };
    let page_key = |path: &str| path.to_string();
    let href_keys = |href: &str| record(href).map(|r| key(&r)).into_iter().collect();
    Ok(html::assemble(parsed, &read_image, &page_key, &href_keys))
}

/// A record's stable name in the guide (its `Page::path`, the slug shared in chat).
fn key(rec: &Record) -> String {
    match rec {
        Record::Quest(id) => format!("vault/quest/{id}"),
        Record::Place(id) => format!("vault/place/{id}"),
        Record::Creature(id) => format!("vault/creature/{id}"),
        // A trophy's page is its type's table (`trophy_tables` names them `vault/trophy/<id>`
        // too, so links to a trophy open its table).
        Record::Trophy(id) => format!("vault/trophy/{id}"),
        Record::Guide(dir, c) if c.is_empty() => format!("vault/{dir}"),
        Record::Guide(dir, c) => format!("vault/{dir}/{c}"),
    }
}

/// The page's content block: after the template's ad (`</noscript>` past the search form) up to
/// its forum feed or network box.
fn content(html: &str) -> Option<&str> {
    let search = html.find("GOOGLE SEARCH END")?;
    let start = html[search..]
        .find("</noscript>")
        .map_or(search, |e| search + e + "</noscript>".len());
    let end = [
        "<!--NO FORUM CONTENT",
        "<!-------Start Network",
        "<!-- network connections",
    ]
    .iter()
    .filter_map(|m| html[start..].find(m))
    .min()
    .map_or(html.len(), |e| start + e);
    Some(&html[start..end])
}

/// Links and pictures on the site's own hosts made mirror paths: `http://ac2vault.ign.com/x` is
/// `x`, other hosts are under `_ext/<host>/`.
fn local_links(content: &str) -> String {
    let mut s = content.to_string();
    for (from, to) in [
        ("http://ac2vault.ign.com/", ""),
        ("http://vnmedia.ign.com/", "_ext/vnmedia.ign.com/"),
        ("http://media.ign.com/", "_ext/media.ign.com/"),
    ] {
        s = s.replace(from, to);
        s = s.replace(&from.to_ascii_uppercase(), to);
    }
    // The site often wrote `&nbsp` without its semicolon.
    let s = s.replace("&nbsp;", "&nbsp").replace("&nbsp", "&nbsp;");
    accent_headings(&s)
}

/// The site's `<span class="accent">` section titles as headings.
fn accent_headings(s: &str) -> String {
    let lower = s.to_ascii_lowercase();
    let mut out = String::with_capacity(s.len());
    let mut at = 0;
    while let Some(found) = ["<span class=\"accent\">", "<span class=accent>"]
        .iter()
        .filter_map(|m| lower[at..].find(m).map(|i| (at + i, m.len())))
        .min()
    {
        let (open, len) = found;
        let Some(close) = lower[open + len..].find("</span>").map(|i| open + len + i) else {
            break;
        };
        out.push_str(&s[at..open]);
        out.push_str("<h4>");
        out.push_str(&s[open + len..close]);
        out.push_str("</h4>");
        at = close + "</span>".len();
    }
    out.push_str(&s[at..]);
    out
}

/// One page of the guide from a record's content.
fn page(rec: &Record, file: &str, content: &str) -> Option<Parsed> {
    let (kind, quest) = match rec {
        Record::Quest(_) => (Kind::Quest, Some(quest_facts(content))),
        Record::Place(_) => (Kind::Npc, None),
        Record::Creature(_) => (Kind::Monster, None),
        Record::Guide(..) => (Kind::Guide, None),
        Record::Trophy(_) => return None,
    };
    let (mut lines, images) = html::content_lines("", content);
    lines.retain(|l| !site_line(l));
    if lines.is_empty() {
        return None;
    }
    let title = match rec {
        Record::Place(_) => field_line(&lines, "Name"),
        Record::Creature(_) => first_bold(&lines),
        Record::Guide(dir, c) if dir == "spells" => SPELL_TITLES
            .iter()
            .find(|(k, _)| k == c)
            .map(|(_, t)| t.to_string()),
        Record::Guide(_, c) if c == "newb" => Some("A Newbie's Guide to Dereth".into()),
        _ => None,
    }
    .or_else(|| accent(content))
    .or_else(|| html_title(file, content))?;
    // The window shows the title; the page's own copies of it at the top go (a picture beside
    // one stays).
    while let Some(first) = lines.first_mut() {
        let text = html::line_text(first);
        if !text.trim_matches(cell_edge).eq_ignore_ascii_case(&title) {
            break;
        }
        first.spans.retain(|s| matches!(s, Span::Image { .. }));
        if !first.spans.is_empty() {
            break;
        }
        lines.remove(0);
    }
    if let Record::Quest(_) = rec {
        // The facts block above the text shows the table's rows from Requirements to Reward;
        // Misc Notes stays.
        let starts = |l: &Line, label: &str| {
            html::line_text(l)
                .trim_start_matches(cell_edge)
                .starts_with(label)
        };
        if let Some(first) = lines.iter().position(|l| starts(l, "Requirements")) {
            let end = lines[first..]
                .iter()
                .position(|l| starts(l, "Misc Notes") || starts(l, "Description"))
                .map_or(first + 1, |e| first + e);
            lines.drain(first..end);
        }
    }
    // The table cells' bars: none at a line's end, none between a label and its value.
    for line in &mut lines {
        if let Some(Span::Text { text, .. }) = line.spans.last_mut() {
            let trimmed = text.trim_end_matches([' ', '|']).len();
            text.truncate(trimmed);
        }
        {
            // `Misc Notes: | None`: the cell bar between a label and its value goes.
            let mut after_label = false;
            for span in &mut line.spans {
                if let Span::Text { text, .. } = span {
                    *text = text.replace(": | ", ": ");
                    if after_label {
                        if let Some(rest) = text.strip_prefix(" | ") {
                            *text = format!(" {rest}");
                        }
                    }
                    after_label = text.trim_end().ends_with(':');
                }
            }
        }
    }
    lines.retain(|l| {
        !html::line_text(l).trim().is_empty()
            || l.spans.iter().any(|s| matches!(s, Span::Image { .. }))
    });
    group_experience(&mut lines);
    let field = |label: &str| field_line(&lines, label);
    let area = match rec {
        Record::Place(_) => {
            let land = field("Landmass").unwrap_or_default();
            let kind = field("Type").unwrap_or_default();
            [land, kind]
                .into_iter()
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join(", ")
        }
        Record::Creature(_) => match field("Level").and_then(|l| l.parse::<u32>().ok()) {
            Some(0) | None => String::new(),
            Some(l) if l < 10 => "Levels 1 to 9".into(),
            Some(l) => format!("Levels {} to {}", l / 10 * 10, l / 10 * 10 + 9),
        },
        Record::Guide(dir, _) => GUIDE_DIRS
            .iter()
            .find(|(d, _, _)| d == dir)
            .map_or("Event guides", |(_, h, _)| h)
            .to_string(),
        _ => String::new(),
    };
    Some(Parsed {
        page: Page {
            path: key(rec),
            kind,
            title,
            area,
            quest,
            lines,
        },
        images,
        items: Vec::new(),
    })
}

/// The site's navigation and print links, which mean nothing in the client.
fn site_line(line: &Line) -> bool {
    let t = html::line_text(line);
    let t = t.trim().trim_matches('|').trim();
    let lower = t.to_ascii_lowercase();
    [
        "return to index",
        "quest index",
        "back to the location database",
        "print this",
        "(click thumbnail",
        "click here to",
        "comments (submit a comment)",
        "no comments found",
        "please email any corrections",
    ]
    .iter()
    .any(|p| lower.starts_with(p))
        || lower.ends_with("forum link")
        || lower.contains("(click here")
        || lower.contains("print this quest")
        || lower == "information"
}

/// Spaces, colons and the ` | ` the lines put between table cells.
fn cell_edge(c: char) -> bool {
    c.is_whitespace() || c == '|' || c == ':'
}

/// The value of a `Label: | value` (or `Label |  | value`) table row.
fn field_line(lines: &[Line], label: &str) -> Option<String> {
    lines.iter().find_map(|l| {
        let t = html::line_text(l);
        let rest = t.trim_start_matches(cell_edge).strip_prefix(label)?;
        if !rest.starts_with(cell_edge) {
            return None;
        }
        let v = rest.trim_matches(cell_edge).to_string();
        (!v.is_empty()).then_some(v)
    })
}

/// The first bold run's text (a bestiary page's creature name).
fn first_bold(lines: &[Line]) -> Option<String> {
    lines.iter().flat_map(|l| &l.spans).find_map(|s| match s {
        Span::Text { text, bold: true } if !text.trim().is_empty() => Some(text.trim().to_string()),
        _ => None,
    })
}

/// The text of the first `class="accent"` span, the site's page heading (an `<h4>` once
/// `local_links` has made it one).
fn accent(content: &str) -> Option<String> {
    let lower = content.to_ascii_lowercase();
    let mut from = 0;
    while let Some(at) = ["accent", "<h4>"]
        .iter()
        .filter_map(|m| lower[from..].find(m))
        .min()
    {
        let at = from + at;
        from = at + 4;
        let open = at + lower[at..].find('>')? + 1;
        let close = open
            + ["</span", "</h4"]
                .iter()
                .filter_map(|m| lower[open..].find(m))
                .min()?;
        let t = plain(&content[open..close]);
        let t = t.trim_end_matches(':').trim();
        if !t.is_empty() && t.len() < 120 {
            return Some(t.to_string());
        }
    }
    None
}

fn html_title(file: &str, _content: &str) -> Option<String> {
    record(file).map(|r| match r {
        Record::Guide(dir, c) if c.is_empty() => dir,
        Record::Guide(_, c) => c,
        other => key(&other),
    })
}

fn plain(raw: &str) -> String {
    facts::plain(raw)
}

/// Experience and quest points with thousands separators: `60000 XP` is `60,000 XP` wherever it
/// is written, and the bestiary's `Experience` row's value too.
fn group_experience(lines: &mut [Line]) {
    for line in lines {
        let experience_row = html::line_text(line)
            .trim_start_matches(cell_edge)
            .starts_with("Experience");
        for span in &mut line.spans {
            if let Span::Text { text, .. } = span {
                *text = if experience_row {
                    group_all(text)
                } else {
                    group_points(text)
                };
            }
        }
    }
}

/// Every run of four or more digits followed by `XP` or `QP` (any case, spaces between) grouped.
fn group_points(text: &str) -> String {
    group_runs(text, |rest| {
        let rest = rest.trim_start().to_ascii_lowercase();
        rest.starts_with("xp") || rest.starts_with("qp")
    })
}

/// Every whole number of four or more digits grouped.
fn group_all(text: &str) -> String {
    group_runs(text, |_| true)
}

/// Digit runs of four or more, not part of a decimal or an already grouped number, grouped by
/// thousands where `wanted` accepts the text after them.
fn group_runs(text: &str, wanted: impl Fn(&str) -> bool) -> String {
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len() + 8);
    let mut i = 0;
    while i < bytes.len() {
        if !bytes[i].is_ascii_digit() {
            let c = text[i..].chars().next().unwrap_or(' ');
            out.push(c);
            i += c.len_utf8();
            continue;
        }
        let start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        let run = &text[start..i];
        let before = start.checked_sub(1).map(|b| bytes[b]);
        let after = bytes.get(i).copied();
        let part_of_more = matches!(before, Some(b'.' | b',' | b'#'))
            || (matches!(after, Some(b'.' | b','))
                && bytes.get(i + 1).is_some_and(u8::is_ascii_digit));
        if run.len() >= 4 && !part_of_more && wanted(&text[i..]) {
            let first = run.len() % 3;
            for (k, ch) in run.chars().enumerate() {
                if k > 0 && (k + 3 - first).is_multiple_of(3) {
                    out.push(',');
                }
                out.push(ch);
            }
        } else {
            out.push_str(run);
        }
    }
    out
}

/// A quest's facts from the quest database's table (`<b>Requirements:&nbsp</b></td><td>…`) and
/// its `Starting the Quest:` paragraph.
fn quest_facts(content: &str) -> QuestFacts {
    let requirements = cell(content, "Requirements")
        .map(|v| plain(&v))
        .unwrap_or_default();
    let level = requirements
        .to_ascii_lowercase()
        .split("level")
        .nth(1)
        .and_then(|rest| {
            let digits: String = rest
                .trim_start_matches([':', ' '])
                .chars()
                .take_while(char::is_ascii_digit)
                .collect();
            digits.parse().ok()
        });
    // A reward's bulleted details belong to it, not new rewards.
    const DETAIL: char = '\u{2}';
    let mut rewards: Vec<Reward> = Vec::new();
    let entries = cell(content, "Reward")
        .map(|v| {
            let v = v.replace("<li>", "<li>\u{2}").replace("<LI>", "<LI>\u{2}");
            facts::split_entries(&v)
        })
        .unwrap_or_default();
    for entry in entries {
        let detail = entry.starts_with(DETAIL);
        let text = entry.replace(DETAIL, " ");
        let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
        match rewards.last_mut() {
            Some(last) if detail => {
                last.text.push(' ');
                last.text.push_str(&text);
            }
            _ if text.is_empty() || text.eq_ignore_ascii_case("none") => {}
            _ => rewards.push(Reward {
                text: group_points(&text),
                item: None,
            }),
        }
    }
    let start = after_accent(content, "Starting the Quest")
        .map(|v| plain(&v))
        .unwrap_or_default();
    QuestFacts {
        level,
        requirements,
        rewards,
        time_limit: cell(content, "Time Limit")
            .map(|v| plain(&v))
            .unwrap_or_default(),
        reset: cell(content, "Reset Timer")
            .map(|v| plain(&v))
            .unwrap_or_default(),
        start,
    }
}

/// The raw HTML of the cell after a `<b>label:</b>` cell, up to the row's end.
fn cell(content: &str, label: &str) -> Option<String> {
    let lower = content.to_ascii_lowercase();
    let at = lower.find(&format!("<b>{}:", label.to_ascii_lowercase()))?;
    let td = at + lower[at..].find("</td>")? + 5;
    let open = td + lower[td..].find("<td")?;
    let open = open + lower[open..].find('>')? + 1;
    let end = open + lower[open..].find("</tr>")?;
    let raw = &content[open..end];
    Some(
        raw.trim_end()
            .trim_end_matches("</td>")
            .trim_end_matches("</TD>")
            .to_string(),
    )
}

/// The HTML after `<span class="accent">heading:</span>` (or its `<h4>`) up to the next
/// paragraph or heading.
fn after_accent(content: &str, heading: &str) -> Option<String> {
    let lower = content.to_ascii_lowercase();
    let at = lower.find(&heading.to_ascii_lowercase())?;
    // The heading's close: the site's `</span>`, or `</h4>` once `local_links` made it one.
    let open = ["</span>", "</h4>"]
        .iter()
        .filter_map(|m| lower[at..].find(m).map(|i| at + i + m.len()))
        .min()?;
    let end = ["<p", "<h4", "<span class=\"accent\"", "<span class=accent"]
        .iter()
        .filter_map(|m| lower[open..].find(m))
        .min()
        .map_or(content.len(), |e| open + e);
    Some(content[open..end].to_string())
}

/// A trophy record as an item of its type's table.
struct Trophy {
    id: u32,
    kind: String,
    row: ItemRow,
}

fn trophy(file: &str, content: &str) -> Option<Trophy> {
    let Some(Record::Trophy(id)) = record(file) else {
        return None;
    };
    let (lines, images) = html::content_lines("", content);
    let lines: Vec<Line> = lines.into_iter().filter(|l| !site_line(l)).collect();
    let name = first_bold(&lines)?;
    let kind = field_line(&lines, "Type").unwrap_or_else(|| "Other".into());
    let list = |heading: &str| -> Vec<String> {
        let mut out = Vec::new();
        let mut on = false;
        for l in &lines {
            let t = html::line_text(l);
            if l.style == Style::Bullet && on {
                let t = t.trim();
                if !t.to_ascii_lowercase().starts_with("no associated") {
                    out.push(t.to_string());
                }
            } else {
                on = t.trim().eq_ignore_ascii_case(heading);
            }
        }
        out
    };
    let mut note = Vec::new();
    if let Some(v) = field_line(&lines, "Value").filter(|v| v != "0") {
        note.push(format!("Value {v}"));
    }
    let creatures = list("Associated Creatures");
    if !creatures.is_empty() {
        note.push(format!("Dropped by {}", creatures.join(", ")));
    }
    let quests = list("Associated Quests");
    if !quests.is_empty() {
        note.push(format!("Quests: {}", quests.join(", ")));
    }
    if let Some(n) = field_line(&lines, "Notes") {
        note.push(n);
    }
    Some(Trophy {
        id,
        kind,
        row: ItemRow {
            name,
            icon: images.into_iter().next(),
            note: note.join(" · "),
        },
    })
}

/// One guide page per trophy type, listing its trophies with their icons and notes; the items
/// are its rows. Each trophy's own key is kept so links to it open its table.
fn trophy_tables(trophies: Vec<Trophy>) -> Vec<Parsed> {
    let mut by_kind: Vec<(String, Vec<Trophy>)> = Vec::new();
    for t in trophies {
        match by_kind.iter_mut().find(|(k, _)| *k == t.kind) {
            Some((_, list)) => list.push(t),
            None => by_kind.push((t.kind.clone(), vec![t])),
        }
    }
    by_kind
        .into_iter()
        .map(|(kind, mut list)| {
            list.sort_by_key(|a| a.row.name.to_lowercase());
            let mut images = Vec::new();
            let mut lines = Vec::new();
            for t in &list {
                let mut spans = Vec::new();
                if let Some(icon) = &t.row.icon {
                    images.push(icon.clone());
                    spans.push(Span::Image {
                        image: images.len() as u32 - 1,
                    });
                    spans.push(Span::Text {
                        text: " ".into(),
                        bold: false,
                    });
                }
                spans.push(Span::Text {
                    text: t.row.name.clone(),
                    bold: true,
                });
                lines.push(Line {
                    style: Style::Body,
                    spans,
                });
                if !t.row.note.is_empty() {
                    lines.push(Line {
                        style: Style::Bullet,
                        spans: vec![Span::Text {
                            text: t.row.note.clone(),
                            bold: false,
                        }],
                    });
                }
            }
            let path = list.iter().map(|t| t.id).min().map_or_else(
                || format!("vault/trophies/{kind}"),
                |id| key(&Record::Trophy(id)),
            );
            Parsed {
                page: Page {
                    path,
                    kind: Kind::Guide,
                    title: kind,
                    area: "Trophies and items".into(),
                    quest: None,
                    lines,
                },
                images,
                items: list.into_iter().map(|t| t.row).collect(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_from_saved_names_and_links() {
        assert_eq!(
            record("index.html@dir=quests%2Fdb&content=view&id=17"),
            Some(Record::Quest(17))
        );
        assert_eq!(
            record("index.html@dir=cartography%252Fatlas&amp;content=view&amp;id=4"),
            Some(Record::Place(4))
        );
        assert_eq!(
            record(
                "http://ac2vault.ign.com/index.php?dir=beasts&content=detail&section=creature&id=2"
            ),
            Some(Record::Creature(2))
        );
        assert_eq!(
            record("index.php@dir=beasts&content=detail&section=trophy&id=1&associate=&type=1"),
            Some(Record::Trophy(1))
        );
        assert_eq!(
            record("index.html@dir=guides&content=spellbinding#1.1"),
            Some(Record::Guide("guides".into(), "spellbinding".into()))
        );
        assert_eq!(
            record("index.html@dir=eventguide%2Fdec2002&slinks=blank"),
            Some(Record::Guide("eventguide/dec2002".into(), String::new()))
        );
        assert_eq!(record("index.html@dir=guides&content=optimize"), None);
        assert_eq!(record("index.html@dir=guides"), None);
        assert_eq!(record("index.html@dir=forum"), None);
        assert_eq!(record("vault/ariakivault.html"), None);
    }

    #[test]
    fn quest_table_facts() {
        let html = "<table><tr><td><b>Requirements:&nbsp</b></td>\n<td class=x>Level: 31+</td></tr>\
            <tr><td><b>Time Limit:&nbsp</b></td>\n<td>\n1 Days and 0 Hours to complete the quest</td></tr>\
            <tr><td><b>Reset Timer:&nbsp</b></td>\n<td>You must wait 2 Day(s)</td></tr>\
            <tr><td><b>Reward:&nbsp</b></td>\n<td>\n60000 XP<br>\n3000 QP<br>\nDarkstone \nUpon use:<br>\
            <li>1st 3 hours: armor.\n<li>Then 10 armor.</td></td></tr>\
            <tr><td><b>Misc Notes:&nbsp</b></td><td>None</td></tr></table>\
            <span class=\"accent\">Starting the Quest:</span><br>\nGo to the Outpost, 31.2N 93.1E. <p>next";
        assert_eq!(
            quest_facts(&local_links(html)).start,
            "Go to the Outpost, 31.2N 93.1E."
        );
        let f = quest_facts(html);
        assert_eq!(f.level, Some(31));
        assert_eq!(f.requirements, "Level: 31+");
        assert_eq!(f.time_limit, "1 Days and 0 Hours to complete the quest");
        assert_eq!(f.reset, "You must wait 2 Day(s)");
        let rewards: Vec<_> = f.rewards.iter().map(|r| r.text.as_str()).collect();
        assert_eq!(
            rewards,
            [
                "60,000 XP",
                "3,000 QP",
                "Darkstone Upon use: 1st 3 hours: armor. Then 10 armor."
            ]
        );
        assert_eq!(f.start, "Go to the Outpost, 31.2N 93.1E.");
    }

    #[test]
    fn experience_is_grouped() {
        assert_eq!(group_points("60000 XP, 3000 QP"), "60,000 XP, 3,000 QP");
        assert_eq!(
            group_points("14000000 xp and 300 XP"),
            "14,000,000 xp and 300 XP"
        );
        assert_eq!(
            group_points("Feb 18 2004, 31.2N 93.1E"),
            "Feb 18 2004, 31.2N 93.1E"
        );
        assert_eq!(group_points("already 60,000 XP"), "already 60,000 XP");
        assert_eq!(
            group_all("Experience |  | 2299000"),
            "Experience |  | 2,299,000"
        );
        assert_eq!(group_all("115500"), "115,500");
    }

    #[test]
    fn content_between_ad_and_footer() {
        let html =
            "<nav>menu</nav><!-- GOOGLE SEARCH END --><script>ad</script><noscript>x</noscript>\
            <span class=\"accent\">Spellbinding Guide</span><p>Body</p>\
            <!--NO FORUM CONTENT FEED AVAILABLE--><div>network</div>";
        let c = content(html).expect("content");
        assert!(c.starts_with("<span"));
        assert!(c.ends_with("</p>"));
        assert_eq!(accent(c).as_deref(), Some("Spellbinding Guide"));
        assert_eq!(
            accent(&local_links(c)).as_deref(),
            Some("Spellbinding Guide")
        );
        assert_eq!(
            accent_headings("a<SPAN class=accent>Description:</SPAN><br>b"),
            "a<h4>Description:</h4><br>b"
        );
    }
}
