//! The guide's places as map markers: where each quest starts (the coordinate in its
//! `Start Location:`), the other coordinates its walkthrough names, every NPC the database
//! places, and the points the guides list (dye plants, tarot spots, …).

use crate::coord;
use crate::guide::{Guide, Kind, MapCoord};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MarkerKind {
    QuestStart,
    QuestStep,
    Npc,
    GuidePoint,
}

impl MarkerKind {
    pub const ALL: [MarkerKind; 4] = [
        MarkerKind::QuestStart,
        MarkerKind::Npc,
        MarkerKind::QuestStep,
        MarkerKind::GuidePoint,
    ];

    pub fn label(self) -> &'static str {
        match self {
            MarkerKind::QuestStart => "Quest starts",
            MarkerKind::QuestStep => "Quest steps",
            MarkerKind::Npc => "NPCs",
            MarkerKind::GuidePoint => "Guide points",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Marker {
    pub at: MapCoord,
    pub kind: MarkerKind,
    /// The page it comes from.
    pub page: u32,
    /// The page's title (a quest's, an NPC's name, a guide's).
    pub title: String,
    /// The text around the coordinate on the page (the quest giver for a start).
    pub context: String,
    /// The quest's level, for quest markers that state one.
    pub level: Option<u32>,
}

pub fn markers(guide: &Guide) -> Vec<Marker> {
    let mut out = Vec::new();
    for (i, page) in guide.pages.iter().enumerate() {
        let page_index = i as u32;
        let push = |out: &mut Vec<Marker>, at, kind, context: String| {
            out.push(Marker {
                at,
                kind,
                page: page_index,
                title: page.title.clone(),
                context,
                level: page.level(),
            })
        };
        let start = page.quest.as_ref().and_then(|q| {
            coord::find(&q.start)
                .first()
                .map(|(_, c)| (*c, q.start.clone()))
        });
        if let Some((at, text)) = &start {
            push(&mut out, *at, MarkerKind::QuestStart, giver(text));
        }
        let kind = match page.kind {
            Kind::Quest => MarkerKind::QuestStep,
            Kind::Npc => MarkerKind::Npc,
            Kind::Guide => MarkerKind::GuidePoint,
            Kind::Monster => continue,
        };
        let mut seen: Vec<MapCoord> = start.iter().map(|(c, _)| *c).collect();
        for line in &page.lines {
            let text = crate::html::line_text(line);
            for span in &line.spans {
                if let crate::guide::Span::Coord { at, .. } = span {
                    if seen.contains(at) {
                        continue;
                    }
                    seen.push(*at);
                    let context = if kind == MarkerKind::Npc {
                        npc_context(page)
                    } else {
                        shorten(&text)
                    };
                    push(&mut out, *at, kind, context);
                }
            }
        }
    }
    out
}

/// The quest giver: the start text before its coordinate (`Tima Drokh (14.3N, 3.3E), near …`).
fn giver(start: &str) -> String {
    let cut = coord::find(start)
        .first()
        .map_or(start.len(), |(r, _)| r.start);
    let name = start[..cut].trim_end_matches(['(', ',', ' ']);
    let name = name.strip_suffix(" at").unwrap_or(name).trim();
    let name = if name.is_empty() { start } else { name };
    shorten(name)
}

/// An NPC's `Region:` and `Local:` lines.
fn npc_context(page: &crate::guide::Page) -> String {
    let lines: Vec<String> = page.lines.iter().map(crate::html::line_text).collect();
    let field = |label: &str| {
        lines
            .iter()
            .find_map(|l| l.strip_prefix(label).map(|v| v.trim().to_string()))
            .filter(|v| !v.is_empty())
    };
    [
        field("Local:"),
        field("Region:"),
        Some(page.area.clone()).filter(|a| !a.is_empty()),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(", ")
}

fn shorten(s: &str) -> String {
    const MAX: usize = 140;
    if s.chars().count() <= MAX {
        return s.to_string();
    }
    let mut out: String = s.chars().take(MAX).collect();
    out.push('…');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quest_giver() {
        assert_eq!(
            giver("Tima Drokh (14.3N, 3.3E), near Molwirth"),
            "Tima Drokh"
        );
        assert_eq!(giver("Speak to Bo at 1.0N 2.0E"), "Speak to Bo");
        assert_eq!(giver("Matt (1.0N 2.0E)"), "Matt");
    }
}
