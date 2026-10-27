//! The help window's model on hand-written KSML (no shipped page text): wrapping, styles, tab
//! rows, frames, page and frame loads, and scrolling.

use crate::help_view::*;
use crate::ksml::{parse_ksml_text, KsmlDocument, KsmlStyle};

/// Six pixels a character, eight more for bold: enough to exercise wrapping.
fn width(text: &str, style: KsmlStyle) -> i32 {
    let per = if style.bold { 7 } else { 6 };
    text.chars().count() as i32 * per
}

fn metrics() -> HelpMetrics<'static> {
    HelpMetrics {
        width: &width,
        line_height: 10,
    }
}

fn page(body: &str) -> KsmlDocument {
    parse_ksml_text(&format!("<ksml><head></head><body>{body}</body></ksml>")).unwrap()
}

fn texts(layout: &HelpLayout) -> Vec<String> {
    layout
        .blocks
        .iter()
        .filter_map(|(_, _, block)| match block {
            HelpBlock::Line(segments) => {
                Some(segments.iter().map(|s| s.text.as_str()).collect::<String>())
            }
            _ => None,
        })
        .collect()
}

#[test]
fn text_wraps_at_spaces_within_the_width() {
    let doc = page("the quick brown fox jumps over the lazy dog");
    // 60 pixels is ten characters: "the quick " is ten with its space, trailing spaces trimmed.
    let layout = layout_blocks(&doc.nodes, 60, &metrics());
    assert_eq!(
        texts(&layout),
        vec!["the quick", "brown fox", "jumps over", "the lazy", "dog"]
    );
    assert!(layout.blocks.iter().all(|(_, h, _)| *h == 10));
    assert_eq!(layout.height, 50);
}

#[test]
fn a_word_wider_than_the_line_sits_on_its_own_line() {
    let doc = page("supercalifragilistic ok");
    let layout = layout_blocks(&doc.nodes, 30, &metrics());
    assert_eq!(texts(&layout), vec!["supercalifragilistic", "ok"]);
}

#[test]
fn line_breaks_end_lines_and_styles_stay_on_their_words() {
    let doc = page("plain <b>bold</b> <i>it</i><br/>next");
    let layout = layout_blocks(&doc.nodes, 400, &metrics());
    assert_eq!(texts(&layout), vec!["plain bold it", "next"]);
    let HelpBlock::Line(first) = &layout.blocks[0].2 else {
        panic!("first block is a line");
    };
    let bold: Vec<&str> = first
        .iter()
        .filter(|s| s.style.bold)
        .map(|s| s.text.as_str())
        .collect();
    assert_eq!(bold, vec!["bold"]);
    assert!(first.iter().any(|s| s.style.italic && s.text == "it"));
}

#[test]
fn links_carry_their_href_and_target() {
    let doc = page(r#"see <a href="content/x.ksml" target="main">this</a>"#);
    let layout = layout_blocks(&doc.nodes, 400, &metrics());
    let HelpBlock::Line(segments) = &layout.blocks[0].2 else {
        panic!("a line");
    };
    let link = segments
        .iter()
        .find_map(|s| s.link.clone())
        .expect("a linked segment");
    assert_eq!(link.href.as_deref(), Some("content/x.ksml"));
    assert_eq!(link.target.as_deref(), Some("main"));
}

#[test]
fn consecutive_tabs_form_one_row_and_block_text_follows() {
    let doc = page(
        r#"<radiobutton name="t" id="a" href="a.ksml" target="main" checked="true">A</radiobutton><radiobutton name="t" id="b" href="b.ksml" target="main">B</radiobutton>after"#,
    );
    let layout = layout_blocks(&doc.nodes, 400, &metrics());
    let HelpBlock::Tabs(row) = &layout.blocks[0].2 else {
        panic!("tab row first");
    };
    assert_eq!(row.len(), 2);
    assert!(row[0].checked);
    assert_eq!(row[1].href.as_deref(), Some("b.ksml"));
    assert_eq!(texts(&layout), vec!["after"]);
}

#[test]
fn frames_take_their_height_and_stack_in_order() {
    let doc = page(r#"top<iframe name="main" src="a.ksml" height="40"/>bottom"#);
    let layout = layout_blocks(&doc.nodes, 400, &metrics());
    let tops: Vec<(i32, i32)> = layout.blocks.iter().map(|(y, h, _)| (*y, *h)).collect();
    assert_eq!(tops, vec![(0, 10), (10, 40), (50, 10)]);
    assert_eq!(layout.height, 60);
}

#[test]
fn opening_then_loading_fills_the_page_and_a_stale_load_is_dropped() {
    let mut view = HelpView::new();
    let HelpRequest::Load { ticket: first, .. } = view.open("content/a.ksml") else {
        panic!("open loads");
    };
    let HelpRequest::Load { ticket: second, .. } = view.open("content/b.ksml") else {
        panic!("open loads");
    };
    assert!(!view.loaded(first, None, page("old")));
    assert!(view.document().is_none());
    assert!(view.loaded(second, None, page("new")));
    assert_eq!(view.page(), Some("content/b.ksml"));
    assert!(view.document().is_some());
}

#[test]
fn a_link_into_a_frame_loads_that_frame_and_the_page_stays() {
    let mut view = HelpView::new();
    let HelpRequest::Load { ticket, .. } = view.open("content/main.ksml") else {
        panic!("open loads");
    };
    assert!(view.loaded(ticket, None, page(r#"<iframe name="pane" height="20"/>"#)));
    let frames = page(r#"<iframe name="pane" src="x"/>"#).nodes;
    let crate::ksml::KsmlNode::Frame(frame) = &frames[0] else {
        panic!("a frame node");
    };
    let loads = view.embed_frames(std::slice::from_ref(frame));
    let [HelpRequest::Load {
        ticket: frame_ticket,
        page: src,
        target,
    }] = loads.as_slice()
    else {
        panic!("one frame load");
    };
    assert_eq!(src, "x");
    assert_eq!(target.as_deref(), Some("pane"));
    // The main page has not moved while the frame loads.
    assert_eq!(view.page(), Some("content/main.ksml"));
    assert!(view.loaded(*frame_ticket, Some("pane"), page("inside")));
    assert_eq!(view.frames()[0].page.as_deref(), Some("x"));
    assert!(view.frames()[0].document.is_some());
}

#[test]
fn an_earlier_frame_load_still_lands_after_a_later_frame_is_embedded() {
    let mut view = HelpView::new();
    let HelpRequest::Load { ticket, .. } = view.open("main") else {
        panic!("open loads");
    };
    assert!(view.loaded(ticket, None, page("")));
    let frames = [
        crate::ksml::KsmlFrame {
            name: Some("one".into()),
            id: None,
            src: Some("a".into()),
            width: None,
            height: None,
        },
        crate::ksml::KsmlFrame {
            name: Some("two".into()),
            id: None,
            src: Some("b".into()),
            width: None,
            height: None,
        },
    ];
    let loads = view.embed_frames(&frames);
    let tickets: Vec<u64> = loads
        .iter()
        .map(|r| match r {
            HelpRequest::Load { ticket, .. } => *ticket,
            HelpRequest::Close => panic!("no close"),
        })
        .collect();
    assert!(view.loaded(tickets[0], Some("one"), page("")));
    assert!(view.loaded(tickets[1], Some("two"), page("")));
}

#[test]
fn pressing_a_link_without_href_asks_for_nothing_and_a_link_loads_the_main_page() {
    let mut view = HelpView::new();
    assert_eq!(
        view.press_link(&HelpLink {
            href: None,
            target: None
        }),
        None
    );
    let request = view.press_link(&HelpLink {
        href: Some("content/c.ksml".into()),
        target: None,
    });
    assert!(matches!(
        request,
        Some(HelpRequest::Load { target: None, ref page, .. }) if page == "content/c.ksml"
    ));
}

#[test]
fn scrolling_clamps_to_the_page_and_wheel_notches_move_three_lines() {
    let mut view = HelpView::new();
    view.scroll_by(-50, 200, 100);
    assert_eq!(view.scroll(), 0);
    view.scroll_by(500, 200, 100);
    assert_eq!(view.scroll(), 100);
    // A positive notch is a wheel turn up: three 10-pixel lines back per notch.
    view.wheel(1, 10, 200, 100);
    assert_eq!(view.scroll(), 70);
    view.wheel(-1, 10, 200, 100);
    assert_eq!(view.scroll(), 100);
    view.scroll_by(-100, 200, 100);
    view.wheel(-1, 10, 200, 100);
    assert_eq!(view.scroll(), 30);
    view.wheel(1, 10, 200, 100);
    assert_eq!(view.scroll(), 0);
}

#[test]
fn a_short_page_does_not_scroll_and_close_clears_the_window() {
    let mut view = HelpView::new();
    view.scroll_by(40, 50, 100);
    assert_eq!(view.scroll(), 0);
    let HelpRequest::Load { ticket, .. } = view.open("content/d.ksml") else {
        panic!("open loads");
    };
    assert!(view.loaded(ticket, None, page("d")));
    assert_eq!(view.close(), HelpRequest::Close);
    assert!(view.document().is_none());
    assert!(view.page().is_none());
}
