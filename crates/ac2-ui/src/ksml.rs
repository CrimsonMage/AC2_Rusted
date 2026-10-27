//! KSML help pages (`AC2HelpPlugin.dll` renders them from `Content/*.ksml`): the markup behind
//! the help window that `KeyStone::OpenHelp` opens.
//!
//! A page is an XML document rooted at `ksml`: a `head` of `meta` name/content pairs and a
//! `body`. Shipped pages are UTF-16 with a byte order mark, or plain ASCII or UTF-8 without one;
//! [`decode_ksml`] reads all of them. The body becomes a [`KsmlDocument`] of styled runs
//! (`text`, `b`, `i`, `u`), line breaks (`br`, and the block elements `h1`..`h4`, `ul`, `ol`,
//! `li`, `table`, `tr`, `p`, `div`), links (`a`), radio-button tabs (`radiobutton`) and frames
//! (`iframe`). Every other element (`button`, `listbox`, `editbox`, `treeview`, `title`, ...) is
//! kept as a [`KsmlNode::Control`] with its attributes and children, so a renderer decides what
//! to draw.

use crate::ksml_xml::{parse_xml, XmlElement, XmlError, XmlNode};

/// A parsed page: its `meta` pairs, the body's `background` and the body's nodes.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct KsmlDocument {
    /// The `head`'s `meta` elements as (`name`, `content`), in order; missing attributes read
    /// as empty.
    pub meta: Vec<(String, String)>,
    /// The body's `background` attribute, the image behind the page.
    pub background: Option<String>,
    pub nodes: Vec<KsmlNode>,
}

/// One node of the body, in reading order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KsmlNode {
    Run(KsmlRun),
    /// A line break (`br`) or the break between blocks.
    LineBreak,
    Link(KsmlLink),
    Tab(KsmlTab),
    Frame(KsmlFrame),
    Control(KsmlControl),
}

/// The text styles KSML applies: `b`, `i`, `u`, and the bold of a heading.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct KsmlStyle {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

/// Text with one style. Whitespace is collapsed to single spaces, and a run never starts or
/// ends a line with a space.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KsmlRun {
    pub text: String,
    pub style: KsmlStyle,
}

/// An `a` element: `href` is the page it opens (absent when the element has none), `target` the
/// frame it opens in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KsmlLink {
    pub href: Option<String>,
    pub target: Option<String>,
    pub children: Vec<KsmlNode>,
}

/// A `radiobutton` element: one tab of a tab row. Tabs that share a `name` are one group, and
/// `href` loads into the frame named by `target`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KsmlTab {
    pub name: Option<String>,
    pub id: Option<String>,
    pub text: String,
    pub href: Option<String>,
    pub target: Option<String>,
    pub checked: bool,
}

/// An `iframe` element: a region that shows another page (`src`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KsmlFrame {
    pub name: Option<String>,
    pub id: Option<String>,
    pub src: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

/// Any other element, with its attributes and children.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KsmlControl {
    pub element: String,
    pub id: Option<String>,
    pub attributes: Vec<(String, String)>,
    pub children: Vec<KsmlNode>,
}

/// Why a page did not load.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KsmlError {
    /// The bytes are not the encoding their byte order mark (or the lack of one) declares.
    Decode(String),
    Xml(XmlError),
    /// The root element is not `ksml`.
    NotKsml(String),
}

impl std::fmt::Display for KsmlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Decode(reason) => write!(f, "KSML decode: {reason}"),
            Self::Xml(error) => write!(f, "KSML markup: {error}"),
            Self::NotKsml(root) => write!(f, "KSML root element is <{root}>"),
        }
    }
}

impl std::error::Error for KsmlError {}

impl From<XmlError> for KsmlError {
    fn from(error: XmlError) -> Self {
        Self::Xml(error)
    }
}

/// Decodes a page's bytes to text: a UTF-16 byte order mark (little or big endian) selects
/// UTF-16, a UTF-8 mark is dropped, and bytes with no mark are UTF-8.
pub fn decode_ksml(bytes: &[u8]) -> Result<String, KsmlError> {
    match bytes {
        [0xFF, 0xFE, rest @ ..] => decode_utf16(rest, u16::from_le_bytes),
        [0xFE, 0xFF, rest @ ..] => decode_utf16(rest, u16::from_be_bytes),
        [0xEF, 0xBB, 0xBF, rest @ ..] => utf8(rest),
        _ => utf8(bytes),
    }
}

fn utf8(bytes: &[u8]) -> Result<String, KsmlError> {
    String::from_utf8(bytes.to_vec()).map_err(|e| KsmlError::Decode(e.to_string()))
}

fn decode_utf16(bytes: &[u8], unit: fn([u8; 2]) -> u16) -> Result<String, KsmlError> {
    if !bytes.len().is_multiple_of(2) {
        return Err(KsmlError::Decode("odd byte count for UTF-16".into()));
    }
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|pair| unit([pair[0], pair[1]]))
        .collect();
    String::from_utf16(&units).map_err(|e| KsmlError::Decode(e.to_string()))
}

/// Decodes and parses one page.
pub fn parse_ksml(bytes: &[u8]) -> Result<KsmlDocument, KsmlError> {
    parse_ksml_text(&decode_ksml(bytes)?)
}

/// Parses one page already decoded to text.
pub fn parse_ksml_text(text: &str) -> Result<KsmlDocument, KsmlError> {
    let root = parse_xml(text)?;
    if root.name != "ksml" {
        return Err(KsmlError::NotKsml(root.name));
    }
    let mut doc = KsmlDocument::default();
    for section in root.elements() {
        match section.name.as_str() {
            "head" => {
                for meta in section.elements().filter(|e| e.name == "meta") {
                    doc.meta.push((
                        meta.attr("name").unwrap_or_default().to_string(),
                        meta.attr("content").unwrap_or_default().to_string(),
                    ));
                }
            }
            "body" => {
                doc.background = section.attr("background").map(str::to_string);
                doc.nodes = nodes_of(&section.children, KsmlStyle::default());
                trim_trailing_space(&mut doc.nodes);
            }
            _ => {}
        }
    }
    Ok(doc)
}

/// The nodes of a container's children, as a new list (a link's or a control's contents).
fn nodes_of(children: &[XmlNode], style: KsmlStyle) -> Vec<KsmlNode> {
    let mut out = Vec::new();
    push_children(&mut out, children, style);
    out
}

/// Appends the children to `out` itself, so inline elements see the nodes before them (a space
/// after `</b>` is kept, a space at the start of a line is not).
fn push_children(out: &mut Vec<KsmlNode>, children: &[XmlNode], style: KsmlStyle) {
    for child in children {
        match child {
            XmlNode::Text(text) => push_text(out, text, style),
            XmlNode::Element(element) => push_element(out, element, style),
        }
    }
}

fn push_element(out: &mut Vec<KsmlNode>, element: &XmlElement, style: KsmlStyle) {
    let styled = |bold, italic, underline| KsmlStyle {
        bold: style.bold || bold,
        italic: style.italic || italic,
        underline: style.underline || underline,
    };
    match element.name.as_str() {
        "text" => match element.attr("text") {
            Some(text) => push_text(out, text, style),
            None => push_children(out, &element.children, style),
        },
        "br" => push_break(out, true),
        "b" | "strong" => push_children(out, &element.children, styled(true, false, false)),
        "i" | "em" => push_children(out, &element.children, styled(false, true, false)),
        "u" => push_children(out, &element.children, styled(false, false, true)),
        "h1" | "h2" | "h3" | "h4" => {
            push_block(out, &element.children, styled(true, false, false));
        }
        "li" | "ul" | "ol" | "table" | "tr" | "p" | "div" => {
            push_block(out, &element.children, style);
        }
        "font" | "span" | "td" | "th" => push_children(out, &element.children, style),
        "meta" | "head" => {}
        "a" => out.push(KsmlNode::Link(KsmlLink {
            href: element.attr("href").map(str::to_string),
            target: element.attr("target").map(str::to_string),
            children: nodes_of(&element.children, style),
        })),
        "radiobutton" => out.push(KsmlNode::Tab(KsmlTab {
            name: element.attr("name").map(str::to_string),
            id: element.attr("id").map(str::to_string),
            text: element.attr("text").unwrap_or_default().to_string(),
            href: element.attr("href").map(str::to_string),
            target: element.attr("target").map(str::to_string),
            checked: element.attr("checked") == Some("true"),
        })),
        "iframe" => out.push(KsmlNode::Frame(KsmlFrame {
            name: element.attr("name").map(str::to_string),
            id: element.attr("id").map(str::to_string),
            src: element.attr("src").map(str::to_string),
            width: element.attr("width").and_then(|v| v.parse().ok()),
            height: element.attr("height").and_then(|v| v.parse().ok()),
        })),
        _ => out.push(KsmlNode::Control(KsmlControl {
            element: element.name.clone(),
            id: element.attr("id").map(str::to_string),
            attributes: element.attributes.clone(),
            children: nodes_of(&element.children, style),
        })),
    }
}

/// A block element: a break before and after it, its children between.
fn push_block(out: &mut Vec<KsmlNode>, children: &[XmlNode], style: KsmlStyle) {
    push_break(out, false);
    push_children(out, children, style);
    push_break(out, false);
}

/// Appends collapsed text. Whitespace runs become one space; a space is dropped where the line
/// starts, and text of the same style joins the run before it.
fn push_text(out: &mut Vec<KsmlNode>, raw: &str, style: KsmlStyle) {
    let mut text = String::with_capacity(raw.len());
    let mut in_space = false;
    for c in raw.chars() {
        if matches!(c, ' ' | '\t' | '\r' | '\n') {
            if !in_space {
                text.push(' ');
            }
            in_space = true;
        } else {
            text.push(c);
            in_space = false;
        }
    }
    let at_line_start = match out.last() {
        None | Some(KsmlNode::LineBreak) => true,
        Some(KsmlNode::Run(run)) => run.text.ends_with(' '),
        Some(_) => false,
    };
    if at_line_start {
        text = text.trim_start_matches(' ').to_string();
    }
    if text.is_empty() {
        return;
    }
    if let Some(KsmlNode::Run(run)) = out.last_mut() {
        if run.style == style {
            run.text.push_str(&text);
            return;
        }
    }
    out.push(KsmlNode::Run(KsmlRun { text, style }));
}

/// A line break. A forced one (`br`) always lands; a block boundary is skipped at the start of
/// the body or where a break already stands. Spaces before the break are dropped.
fn push_break(out: &mut Vec<KsmlNode>, forced: bool) {
    if !forced && matches!(out.last(), None | Some(KsmlNode::LineBreak)) {
        return;
    }
    if let Some(KsmlNode::Run(run)) = out.last_mut() {
        let kept = run.text.trim_end_matches(' ').len();
        run.text.truncate(kept);
        if run.text.is_empty() {
            out.pop();
        }
    }
    out.push(KsmlNode::LineBreak);
}

/// Drops the spaces that end the body's last run.
fn trim_trailing_space(nodes: &mut Vec<KsmlNode>) {
    if let Some(KsmlNode::Run(run)) = nodes.last_mut() {
        let kept = run.text.trim_end_matches(' ').len();
        run.text.truncate(kept);
        if run.text.is_empty() {
            nodes.pop();
        }
    }
}
