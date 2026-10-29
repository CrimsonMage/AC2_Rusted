//! Browsing the guide: the window's tabs, the list each tab shows for a search, the quests that
//! reward each item, and the stable names pages are shared by in chat.
//!
//! A shared page travels as plain text, `Title [wiki:slug]`, so any client shows it; the slug is
//! the page's mirror path without `.html`, which stays the same from one build to the next.

use crate::guide::{Guide, Kind, Page};

/// A tab of the guide window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Pages(Kind),
    /// The item tables: what quests give.
    Rewards,
}

impl Tab {
    pub const ALL: [Tab; 5] = [
        Tab::Pages(Kind::Quest),
        Tab::Pages(Kind::Npc),
        Tab::Pages(Kind::Monster),
        Tab::Pages(Kind::Guide),
        Tab::Rewards,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Tab::Pages(kind) => kind.label(),
            Tab::Rewards => "Rewards",
        }
    }

    /// What the tab's entries are called in its count (`435 quests`).
    pub fn noun(self) -> &'static str {
        match self {
            Tab::Pages(Kind::Quest) => "quests",
            Tab::Pages(Kind::Npc) => "NPCs",
            Tab::Pages(Kind::Monster) => "monsters",
            Tab::Pages(Kind::Guide) => "guides",
            Tab::Rewards => "items",
        }
    }
}

/// One line of a tab's list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    Heading(String),
    /// A page, by index into `Guide::pages`.
    Page(u32),
    /// An item, by index into `Guide::items`.
    Item(u32),
}

/// Whether every word of `query` is in `text`, ignoring case.
fn matches(words: &[String], text: &str) -> bool {
    let text = text.to_lowercase();
    words.iter().all(|w| text.contains(w.as_str()))
}

/// The list `tab` shows for `query` (words, any case, all present):
///
/// - Quests: matched on the title or a reward, sorted by level then title, under level bands.
/// - NPCs, monsters and guides: matched on the title, by title, under their continent.
/// - Rewards: the item tables' rows matched on name, note or table, under each table's title.
pub fn entries(guide: &Guide, tab: Tab, query: &str) -> Vec<Entry> {
    let words: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
    let mut out = Vec::new();
    match tab {
        Tab::Rewards => {
            let mut items: Vec<u32> = (0..guide.items.len() as u32)
                .filter(|&i| {
                    let item = &guide.items[i as usize];
                    let table = guide.pages.get(item.page as usize).map_or("", |p| &p.title);
                    matches(&words, &format!("{} {} {table}", item.name, item.note))
                })
                .collect();
            items.sort_by_key(|&i| {
                let item = &guide.items[i as usize];
                (item.page, item.name.to_lowercase())
            });
            let mut table = None;
            for i in items {
                let page = guide.items[i as usize].page;
                if table != Some(page) {
                    table = Some(page);
                    let title = guide.pages.get(page as usize).map_or("", |p| &p.title);
                    out.push(Entry::Heading(title.to_owned()));
                }
                out.push(Entry::Item(i));
            }
        }
        Tab::Pages(Kind::Quest) => {
            let mut pages: Vec<u32> = pages_of(guide, Kind::Quest)
                .filter(|(_, p)| {
                    matches(&words, &p.title)
                        || p.quest
                            .iter()
                            .flat_map(|q| &q.rewards)
                            .any(|r| matches(&words, &r.text))
                })
                .map(|(i, _)| i)
                .collect();
            pages.sort_by_key(|&i| {
                let p = &guide.pages[i as usize];
                (p.level().unwrap_or(u32::MAX), p.title.to_lowercase())
            });
            let mut band = None;
            for i in pages {
                let b = guide.pages[i as usize].level().map(|l| l / 10);
                if band != Some(b) {
                    band = Some(b);
                    out.push(Entry::Heading(match b {
                        Some(0) => "Levels 1 to 9".into(),
                        Some(b) => format!("Levels {} to {}", b * 10, b * 10 + 9),
                        None => "Level not stated".into(),
                    }));
                }
                out.push(Entry::Page(i));
            }
        }
        Tab::Pages(kind) => {
            let mut pages: Vec<u32> = pages_of(guide, kind)
                .filter(|(_, p)| matches(&words, &p.title))
                .map(|(i, _)| i)
                .collect();
            // Pages with no continent after the rest.
            pages.sort_by_key(|&i| {
                let p = &guide.pages[i as usize];
                (p.area.is_empty(), p.area.clone(), p.title.to_lowercase())
            });
            let mut area = None;
            for i in pages {
                let a = &guide.pages[i as usize].area;
                if area != Some(a) {
                    area = Some(a);
                    out.push(Entry::Heading(if a.is_empty() {
                        "Elsewhere".to_owned()
                    } else {
                        a.clone()
                    }));
                }
                out.push(Entry::Page(i));
            }
        }
    }
    out
}

fn pages_of(guide: &Guide, kind: Kind) -> impl Iterator<Item = (u32, &Page)> {
    guide
        .pages
        .iter()
        .enumerate()
        .filter(move |(_, p)| p.kind == kind)
        .map(|(i, p)| (i as u32, p))
}

/// For each item (by index), the quests whose rewards name it, each once, in page order.
pub fn item_quests(guide: &Guide) -> Vec<Vec<u32>> {
    let mut out = vec![Vec::new(); guide.items.len()];
    for (i, page) in guide.pages.iter().enumerate() {
        for item in page
            .quest
            .iter()
            .flat_map(|q| &q.rewards)
            .filter_map(|r| r.item)
        {
            if let Some(quests) = out.get_mut(item as usize) {
                if quests.last() != Some(&(i as u32)) {
                    quests.push(i as u32);
                }
            }
        }
    }
    out
}

/// The opening of a shared page's token in chat.
pub const TOKEN_OPEN: &str = "[wiki:";

/// The page's stable name: its mirror path without `.html`.
pub fn slug(page: &Page) -> &str {
    page.path.strip_suffix(".html").unwrap_or(&page.path)
}

/// The page named `slug`.
pub fn page_of_slug(guide: &Guide, slug: &str) -> Option<u32> {
    guide
        .pages
        .iter()
        .position(|p| self::slug(p) == slug)
        .map(|i| i as u32)
}

/// The chat text that shares `page`: its title, then its token.
pub fn share_text(page: &Page) -> String {
    format!("{} {TOKEN_OPEN}{}]", page.title, slug(page))
}

/// The slug of the token in `line` that covers the character at `column`, if any.
pub fn token_at(line: &str, column: usize) -> Option<&str> {
    let mut from = 0;
    while let Some(found) = line[from..].find(TOKEN_OPEN) {
        let start = from + found;
        let body = start + TOKEN_OPEN.len();
        let close = body + line[body..].find(']')?;
        let first = line[..start].chars().count();
        let last = first + line[start..=close].chars().count();
        let slug = &line[body..close];
        if (first..last).contains(&column)
            && !slug.is_empty()
            && !slug.contains(char::is_whitespace)
        {
            return Some(slug);
        }
        from = close + 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guide::{QuestFacts, Reward};

    fn page(path: &str, kind: Kind, title: &str, level: Option<u32>, area: &str) -> Page {
        Page {
            path: path.to_owned(),
            kind,
            title: title.to_owned(),
            area: area.to_owned(),
            quest: (kind == Kind::Quest).then(|| QuestFacts {
                level,
                rewards: vec![Reward {
                    text: format!("{title} Boots"),
                    item: None,
                }],
                ..Default::default()
            }),
            lines: Vec::new(),
        }
    }

    fn guide() -> Guide {
        Guide {
            pages: vec![
                page("quests/b.html", Kind::Quest, "Beta", Some(25), ""),
                page("quests/a.html", Kind::Quest, "Alpha", Some(21), ""),
                page("quests/c.html", Kind::Quest, "Gamma", None, ""),
                page("npcs/x.html", Kind::Npc, "Xan", None, "Dereth"),
                page("npcs/y.html", Kind::Npc, "Yor", None, ""),
            ],
            ..Default::default()
        }
    }

    #[test]
    fn quests_are_grouped_by_level_band_and_found_by_reward() {
        let g = guide();
        assert_eq!(
            entries(&g, Tab::Pages(Kind::Quest), ""),
            vec![
                Entry::Heading("Levels 20 to 29".into()),
                Entry::Page(1),
                Entry::Page(0),
                Entry::Heading("Level not stated".into()),
                Entry::Page(2),
            ]
        );
        assert_eq!(
            entries(&g, Tab::Pages(Kind::Quest), "gamma boots"),
            vec![Entry::Heading("Level not stated".into()), Entry::Page(2)]
        );
        assert_eq!(
            entries(&g, Tab::Pages(Kind::Npc), ""),
            vec![
                Entry::Heading("Dereth".into()),
                Entry::Page(3),
                Entry::Heading("Elsewhere".into()),
                Entry::Page(4),
            ]
        );
    }

    #[test]
    fn a_shared_page_round_trips_through_its_chat_token() {
        let g = guide();
        let text = share_text(&g.pages[1]);
        assert_eq!(text, "Alpha [wiki:quests/a]");
        let line = format!("Bob says, \"{text}\"");
        let at = line.find("[wiki").unwrap() + 3;
        let slug = token_at(&line, at).unwrap();
        assert_eq!(page_of_slug(&g, slug), Some(1));
        assert_eq!(token_at(&line, 2), None);
        assert_eq!(token_at("[wiki:]", 2), None);
        assert_eq!(token_at("é [wiki:quests/b] x", 3), Some("quests/b"));
    }
}
