//! The baked guide: every page of the mirror as styled lines of text, links, images and map
//! coordinates, the quests' stated facts (level, rewards), the item tables and the images the
//! pages show, with a compact little-endian encoding (`encode` at build time, `decode` at
//! start-up). Decoding borrows the images from the baked bytes instead of copying them.

use std::borrow::Cow;

/// What a page is, from where it sits in the mirror.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Quest,
    Npc,
    Monster,
    Guide,
}

impl Kind {
    pub const ALL: [Kind; 4] = [Kind::Quest, Kind::Npc, Kind::Monster, Kind::Guide];

    pub fn label(self) -> &'static str {
        match self {
            Kind::Quest => "Quests",
            Kind::Npc => "NPCs",
            Kind::Monster => "Monsters",
            Kind::Guide => "Guides",
        }
    }

    fn from_code(code: u8) -> Option<Self> {
        Kind::ALL.get(usize::from(code)).copied()
    }
}

/// A point on the game map as players write it: percent of the half map north and east of the
/// middle (`12.3N 45.6E`, `Position::FromRealMapCoords`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MapCoord {
    pub north: f32,
    pub east: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Span {
    Text {
        text: String,
        bold: bool,
    },
    /// A link to another page of the guide (an index into `Guide::pages`).
    Page {
        text: String,
        page: u32,
    },
    /// A link the guide does not hold (a page the mirror missed, another site).
    Missing {
        text: String,
        href: String,
    },
    Coord {
        text: String,
        at: MapCoord,
    },
    /// An image the page shows (an index into `Guide::images`).
    Image {
        image: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    Body,
    Heading,
    Bullet,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub style: Style,
    pub spans: Vec<Span>,
}

/// One entry of a quest's `Reward:` block, as the site wrote it, and the item table entry it
/// names when one matches.
#[derive(Debug, Clone, PartialEq)]
pub struct Reward {
    pub text: String,
    pub item: Option<u32>,
}

/// What a quest page states in its header fields; empty strings where it says nothing.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct QuestFacts {
    /// The level in `Requirements: Level 45+`.
    pub level: Option<u32>,
    pub requirements: String,
    pub rewards: Vec<Reward>,
    pub time_limit: String,
    pub reset: String,
    pub start: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    /// The page's path in the mirror, `/`-separated.
    pub path: String,
    pub kind: Kind,
    pub title: String,
    /// The continent the page belongs to, where the mirror says (`osteth_…` quests, an NPC's
    /// `Continent:`); empty otherwise.
    pub area: String,
    pub quest: Option<QuestFacts>,
    pub lines: Vec<Line>,
}

impl Page {
    /// Every coordinate on the page, in reading order.
    pub fn coords(&self) -> impl Iterator<Item = (&str, MapCoord)> {
        self.lines
            .iter()
            .flat_map(|l| &l.spans)
            .filter_map(|s| match s {
                Span::Coord { text, at } => Some((text.as_str(), *at)),
                _ => None,
            })
    }

    pub fn level(&self) -> Option<u32> {
        self.quest.as_ref().and_then(|q| q.level)
    }
}

/// A row of one of the site's item tables (`items/armor.html`: icon, name, related quests).
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    pub name: String,
    /// The table's page (its title is the category).
    pub page: u32,
    pub icon: Option<u32>,
    pub note: String,
}

/// An image file from the mirror, as stored there (JPEG, PNG or GIF): borrowed from the binary
/// for the baked guide, owned when read from a mirror at run time.
#[derive(Debug, Clone, PartialEq)]
pub struct Image {
    pub path: String,
    pub bytes: Cow<'static, [u8]>,
}

impl Image {
    /// The file's real format from its first bytes (the site names many BMPs `.jpg`).
    pub fn format(bytes: &[u8]) -> Option<&'static str> {
        match bytes {
            [0xFF, 0xD8, 0xFF, ..] => Some("jpg"),
            [0x89, b'P', b'N', b'G', ..] => Some("png"),
            [b'G', b'I', b'F', ..] => Some("gif"),
            [b'B', b'M', ..] => Some("bmp"),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Guide {
    pub pages: Vec<Page>,
    pub items: Vec<Item>,
    pub images: Vec<Image>,
}

const MAGIC: &[u8; 4] = b"AHG2";

pub fn encode(guide: &Guide) -> Vec<u8> {
    let mut w = Writer(Vec::new());
    w.0.extend_from_slice(MAGIC);
    w.u32(guide.pages.len() as u32);
    for page in &guide.pages {
        w.str(&page.path);
        w.0.push(page.kind as u8);
        w.str(&page.title);
        w.str(&page.area);
        match &page.quest {
            None => w.0.push(0),
            Some(q) => {
                w.0.push(1);
                w.opt(q.level);
                w.str(&q.requirements);
                w.u32(q.rewards.len() as u32);
                for r in &q.rewards {
                    w.str(&r.text);
                    w.opt(r.item);
                }
                w.str(&q.time_limit);
                w.str(&q.reset);
                w.str(&q.start);
            }
        }
        w.u32(page.lines.len() as u32);
        for line in &page.lines {
            w.0.push(line.style as u8);
            w.u32(line.spans.len() as u32);
            for span in &line.spans {
                match span {
                    Span::Text { text, bold } => {
                        w.0.push(u8::from(*bold));
                        w.str(text);
                    }
                    Span::Page { text, page } => {
                        w.0.push(2);
                        w.str(text);
                        w.u32(*page);
                    }
                    Span::Missing { text, href } => {
                        w.0.push(3);
                        w.str(text);
                        w.str(href);
                    }
                    Span::Coord { text, at } => {
                        w.0.push(4);
                        w.str(text);
                        w.u32(at.north.to_bits());
                        w.u32(at.east.to_bits());
                    }
                    Span::Image { image } => {
                        w.0.push(5);
                        w.u32(*image);
                    }
                }
            }
        }
    }
    w.u32(guide.items.len() as u32);
    for item in &guide.items {
        w.str(&item.name);
        w.u32(item.page);
        w.opt(item.icon);
        w.str(&item.note);
    }
    w.u32(guide.images.len() as u32);
    for image in &guide.images {
        w.str(&image.path);
        w.u32(image.bytes.len() as u32);
        w.0.extend_from_slice(&image.bytes);
    }
    w.0
}

/// `None` when the bytes are not an encoded guide. The images borrow from `bytes`.
pub fn decode(bytes: &'static [u8]) -> Option<Guide> {
    let mut r = Reader { bytes, at: 0 };
    if r.take(4)? != MAGIC {
        return None;
    }
    let mut pages = Vec::with_capacity(r.u32()? as usize);
    for _ in 0..pages.capacity() {
        let path = r.str()?;
        let kind = Kind::from_code(r.u8()?)?;
        let title = r.str()?;
        let area = r.str()?;
        let quest = match r.u8()? {
            0 => None,
            _ => {
                let level = r.opt()?;
                let requirements = r.str()?;
                let mut rewards = Vec::with_capacity(r.u32()? as usize);
                for _ in 0..rewards.capacity() {
                    rewards.push(Reward {
                        text: r.str()?,
                        item: r.opt()?,
                    });
                }
                Some(QuestFacts {
                    level,
                    requirements,
                    rewards,
                    time_limit: r.str()?,
                    reset: r.str()?,
                    start: r.str()?,
                })
            }
        };
        let mut lines = Vec::with_capacity(r.u32()? as usize);
        for _ in 0..lines.capacity() {
            let style = match r.u8()? {
                0 => Style::Body,
                1 => Style::Heading,
                2 => Style::Bullet,
                _ => return None,
            };
            let mut spans = Vec::with_capacity(r.u32()? as usize);
            for _ in 0..spans.capacity() {
                spans.push(match r.u8()? {
                    tag @ (0 | 1) => Span::Text {
                        text: r.str()?,
                        bold: tag == 1,
                    },
                    2 => Span::Page {
                        text: r.str()?,
                        page: r.u32()?,
                    },
                    3 => Span::Missing {
                        text: r.str()?,
                        href: r.str()?,
                    },
                    4 => Span::Coord {
                        text: r.str()?,
                        at: MapCoord {
                            north: f32::from_bits(r.u32()?),
                            east: f32::from_bits(r.u32()?),
                        },
                    },
                    5 => Span::Image { image: r.u32()? },
                    _ => return None,
                });
            }
            lines.push(Line { style, spans });
        }
        pages.push(Page {
            path,
            kind,
            title,
            area,
            quest,
            lines,
        });
    }
    let mut items = Vec::with_capacity(r.u32()? as usize);
    for _ in 0..items.capacity() {
        items.push(Item {
            name: r.str()?,
            page: r.u32()?,
            icon: r.opt()?,
            note: r.str()?,
        });
    }
    let mut images = Vec::with_capacity(r.u32()? as usize);
    for _ in 0..images.capacity() {
        let path = r.str()?;
        let n = r.u32()? as usize;
        images.push(Image {
            path,
            bytes: Cow::Borrowed(r.take(n)?),
        });
    }
    (r.at == bytes.len()).then_some(Guide {
        pages,
        items,
        images,
    })
}

struct Writer(Vec<u8>);

impl Writer {
    fn u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }

    /// `u32::MAX` stands for none.
    fn opt(&mut self, v: Option<u32>) {
        self.u32(v.unwrap_or(u32::MAX));
    }

    fn str(&mut self, s: &str) {
        self.u32(s.len() as u32);
        self.0.extend_from_slice(s.as_bytes());
    }
}

struct Reader {
    bytes: &'static [u8],
    at: usize,
}

impl Reader {
    fn take(&mut self, n: usize) -> Option<&'static [u8]> {
        let out = self.bytes.get(self.at..self.at.checked_add(n)?)?;
        self.at += n;
        Some(out)
    }

    fn u8(&mut self) -> Option<u8> {
        Some(self.take(1)?[0])
    }

    fn u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }

    fn opt(&mut self) -> Option<Option<u32>> {
        let v = self.u32()?;
        Some((v != u32::MAX).then_some(v))
    }

    fn str(&mut self) -> Option<String> {
        let n = self.u32()? as usize;
        String::from_utf8(self.take(n)?.to_vec()).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        let guide = Guide {
            pages: vec![Page {
                path: "quest/a.html".into(),
                kind: Kind::Quest,
                title: "A".into(),
                area: "Knorr".into(),
                quest: Some(QuestFacts {
                    level: Some(45),
                    requirements: "Level 45+".into(),
                    rewards: vec![
                        Reward {
                            text: "4,000 XP".into(),
                            item: None,
                        },
                        Reward {
                            text: "Drudge Bane Sword".into(),
                            item: Some(0),
                        },
                    ],
                    time_limit: "2 Hours".into(),
                    reset: String::new(),
                    start: "Bo (1.0N 2.0E)".into(),
                }),
                lines: vec![Line {
                    style: Style::Bullet,
                    spans: vec![
                        Span::Text {
                            text: "go ".into(),
                            bold: true,
                        },
                        Span::Page {
                            text: "b".into(),
                            page: 0,
                        },
                        Span::Missing {
                            text: "map".into(),
                            href: "maps/x.png".into(),
                        },
                        Span::Coord {
                            text: "1.5N 2.0W".into(),
                            at: MapCoord {
                                north: 1.5,
                                east: -2.0,
                            },
                        },
                        Span::Image { image: 0 },
                    ],
                }],
            }],
            items: vec![Item {
                name: "Drudge Bane Sword".into(),
                page: 0,
                icon: Some(0),
                note: "Rashan's Revenge".into(),
            }],
            images: vec![Image {
                path: "items/x.jpg".into(),
                bytes: Cow::Owned(vec![1, 2, 3]),
            }],
        };
        let bytes: &'static [u8] = Box::leak(encode(&guide).into_boxed_slice());
        assert_eq!(decode(bytes), Some(guide));
        assert_eq!(decode(&bytes[..bytes.len() - 1]), None);
    }
}
