//! The structured parts of the site's pages, read from their HTML: a quest's header fields
//! (`<strong>Requirements:</strong> Level 45+`, `Reward:`, `Time Limit:`, …) and the rows of the
//! item tables (`items/armor.html`: icon, name, related quests).

use crate::guide::QuestFacts;
use crate::html::{decode_entities, resolve};

/// A quest page's header fields from its content HTML. Rewards are the `Reward:` block split
/// where the site separated them (runs of `&nbsp;`, line and paragraph breaks).
pub fn quest_facts(html: &str) -> QuestFacts {
    let requirements = field(html, "Requirements")
        .map(|v| plain(&v))
        .unwrap_or_default();
    let level = requirements
        .to_ascii_lowercase()
        .split("level")
        .nth(1)
        .and_then(|rest| {
            let digits: String = rest
                .trim_start()
                .chars()
                .take_while(char::is_ascii_digit)
                .collect();
            digits.parse().ok()
        });
    let rewards = field(html, "Reward")
        .map(|v| split_entries(&v))
        .unwrap_or_default()
        .into_iter()
        .map(|text| crate::guide::Reward { text, item: None })
        .collect();
    QuestFacts {
        level,
        requirements,
        rewards,
        time_limit: field(html, "Time Limit")
            .map(|v| plain(&v))
            .unwrap_or_default(),
        reset: field(html, "Reset Timer")
            .map(|v| plain(&v))
            .unwrap_or_default(),
        start: field(html, "Start Location")
            .map(|v| plain(&v))
            .unwrap_or_default(),
    }
}

/// One item table row: name, icon path in the mirror, the rest of the row as text.
pub struct ItemRow {
    pub name: String,
    pub icon: Option<String>,
    pub note: String,
}

/// The rows of every table on an item page that start with an icon cell.
pub fn item_rows(html: &str, dir: &str) -> Vec<ItemRow> {
    let lower = html.to_ascii_lowercase();
    let mut rows = Vec::new();
    for (at, _) in lower.match_indices("<tr") {
        let end = lower[at + 3..]
            .find("<tr")
            .map_or(html.len(), |e| at + 3 + e);
        let row = &html[at..end];
        let row_lower = &lower[at..end];
        let cells: Vec<&str> = row_lower
            .match_indices("<td")
            .map(|(c, _)| {
                let close = row_lower[c..].find("</td").map_or(row.len(), |e| c + e);
                &row[c..close]
            })
            .collect();
        if cells.len() < 2 {
            continue;
        }
        let icon = src(cells[0]).map(|s| resolve(dir, &s));
        let name = plain(cells[1]);
        if icon.is_none() || name.is_empty() {
            continue;
        }
        let note = cells[2..]
            .iter()
            .map(|c| plain(c))
            .filter(|t| !t.is_empty())
            .collect::<Vec<_>>()
            .join(" · ");
        rows.push(ItemRow { name, icon, note });
    }
    rows
}

/// Lower-case letters and digits only, for matching a reward to an item name.
pub fn key(s: &str) -> String {
    s.chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

/// The raw HTML after `<strong>label` (and its `</strong>` and colon) up to the next `<strong`.
fn field(html: &str, label: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    let needle = format!("<strong>{}", label.to_ascii_lowercase());
    let at = lower.find(&needle)? + needle.len();
    let close = at + lower[at..].find("</strong>")? + "</strong>".len();
    let mut value = &html[close..];
    value = value.trim_start().strip_prefix(':').unwrap_or(value);
    let start = html.len() - value.len();
    let end = lower[start..]
        .find("<strong")
        .map_or(html.len(), |e| start + e);
    Some(html[start..end].to_string())
}

/// The entries of a field: split at `&nbsp;` runs and breaks, tags stripped.
fn split_entries(raw: &str) -> Vec<String> {
    let mut s = raw.replace("\r", "");
    for sep in ["<br>", "<br/>", "<br />", "<p>", "</p>", "<li>", "</li>"] {
        s = s
            .replace(sep, "\u{1}")
            .replace(&sep.to_ascii_uppercase(), "\u{1}");
    }
    while s.contains("&nbsp;&nbsp;") {
        s = s.replace("&nbsp;&nbsp;", "\u{1}");
    }
    s.split('\u{1}')
        .map(plain)
        .filter(|t| !t.is_empty())
        .collect()
}

/// Text with tags removed, entities decoded and whitespace collapsed.
fn plain(raw: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for c in raw.chars() {
        match c {
            '<' => in_tag = true,
            '>' if in_tag => {
                in_tag = false;
                out.push(' ');
            }
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    let text = decode_entities(&out);
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn src(cell: &str) -> Option<String> {
    let lower = cell.to_ascii_lowercase();
    let at = lower.find("src=")? + 4;
    let rest = &cell[at..];
    let v = match rest.chars().next()? {
        q @ ('"' | '\'') => rest[1..].split(q).next()?,
        _ => rest
            .split(|c: char| c.is_ascii_whitespace() || c == '>')
            .next()?,
    };
    Some(v.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quest_header() {
        let html = "<h3><span>Quest Name: X</span></h3>\
            <p><strong>Requirements:</strong> Level 45+</p>\
            <p><strong>Reward:</strong></p>\
            <p>&nbsp;&nbsp;&nbsp;&nbsp;50,000 XP &nbsp;&nbsp;&nbsp;10,000 QP &nbsp;&nbsp;Boots of the Ancient Knights\n\
            &nbsp;&nbsp;Gloves of the Ancient Knights</p>\
            <p><strong>Time Limit:</strong> 2 Days</p>\
            <p><strong>Reset Timer:</strong> 7 Days Reset</p>\
            <p><strong>Start Location</strong>: <a href=\"x\">Tima Drokh</a> (14.3N, 3.3E)</p>\
            <p><strong>Walkthrough:</strong></p>";
        let f = quest_facts(html);
        assert_eq!(f.level, Some(45));
        assert_eq!(f.requirements, "Level 45+");
        let rewards: Vec<_> = f.rewards.iter().map(|r| r.text.as_str()).collect();
        assert_eq!(
            rewards,
            [
                "50,000 XP",
                "10,000 QP",
                "Boots of the Ancient Knights",
                "Gloves of the Ancient Knights"
            ]
        );
        assert_eq!(f.time_limit, "2 Days");
        assert_eq!(f.reset, "7 Days Reset");
        assert_eq!(f.start, "Tima Drokh (14.3N, 3.3E)");
    }

    #[test]
    fn item_table() {
        let html = "<table><tr><td>Icon</td><td>Item Name</td><td>Related Quests</td></tr>\
            <tr><td><img src=\"armor/boots.jpg\"></td><td>Boots of the Lost Heroes</td><td><a href=\"../quest/x.html\">Shades</a></td></tr></table>";
        let rows = item_rows(html, "items");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].name, "Boots of the Lost Heroes");
        assert_eq!(rows[0].icon.as_deref(), Some("items/armor/boots.jpg"));
        assert_eq!(rows[0].note, "Shades");
        assert_eq!(key("Drudge Bane-Sword!"), "drudgebanesword");
    }
}
