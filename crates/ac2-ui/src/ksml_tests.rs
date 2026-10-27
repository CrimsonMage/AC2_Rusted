//! KSML parsing on hand-written snippets (no shipped page text).

use crate::ksml::*;

fn utf16le(text: &str) -> Vec<u8> {
    let mut out = vec![0xFF, 0xFE];
    for unit in text.encode_utf16() {
        out.extend_from_slice(&unit.to_le_bytes());
    }
    out
}

fn run(text: &str, style: KsmlStyle) -> KsmlNode {
    KsmlNode::Run(KsmlRun {
        text: text.to_string(),
        style,
    })
}

const PLAIN: KsmlStyle = KsmlStyle {
    bold: false,
    italic: false,
    underline: false,
};

#[test]
fn utf16_little_endian_with_bom_decodes() {
    let page = "<ksml></ksml>";
    assert_eq!(decode_ksml(&utf16le(page)).unwrap(), page);
}

#[test]
fn utf16_big_endian_with_bom_decodes() {
    let mut bytes = vec![0xFE, 0xFF];
    for unit in "<a>é</a>".encode_utf16() {
        bytes.extend_from_slice(&unit.to_be_bytes());
    }
    assert_eq!(decode_ksml(&bytes).unwrap(), "<a>é</a>");
}

#[test]
fn utf8_with_and_without_bom_decodes() {
    assert_eq!(decode_ksml(b"<ksml/>").unwrap(), "<ksml/>");
    assert_eq!(decode_ksml(b"\xEF\xBB\xBF<ksml/>").unwrap(), "<ksml/>");
}

#[test]
fn odd_utf16_length_and_invalid_utf8_fail_to_decode() {
    assert!(matches!(
        decode_ksml(&[0xFF, 0xFE, 0x3C]),
        Err(KsmlError::Decode(_))
    ));
    assert!(matches!(
        decode_ksml(&[0x3C, 0xFF, 0x3E]),
        Err(KsmlError::Decode(_))
    ));
}

#[test]
fn head_meta_and_body_background_are_read() {
    let doc = parse_ksml(&utf16le(
        r#"<?xml version="1.0"?>
<ksml>
  <head>
    <meta name="keywords" content="alpha, beta"/>
    <meta name="toc" content="Gamma"/>
  </head>
  <body background="Gallery\panel.tga"></body>
</ksml>"#,
    ))
    .unwrap();
    assert_eq!(
        doc.meta,
        vec![
            ("keywords".to_string(), "alpha, beta".to_string()),
            ("toc".to_string(), "Gamma".to_string()),
        ]
    );
    assert_eq!(doc.background.as_deref(), Some("Gallery\\panel.tga"));
    assert!(doc.nodes.is_empty());
}

#[test]
fn text_styles_break_and_collapse_whitespace() {
    let doc = parse_ksml_text(
        "<ksml><body>\n  <h2>Heading</h2>\n  plain   words <b>bold</b><i> it</i><u>under</u>\
         <br/><br/>after\n</body></ksml>",
    )
    .unwrap();
    let bold = KsmlStyle {
        bold: true,
        ..PLAIN
    };
    let italic = KsmlStyle {
        italic: true,
        ..PLAIN
    };
    let under = KsmlStyle {
        underline: true,
        ..PLAIN
    };
    assert_eq!(
        doc.nodes,
        vec![
            run("Heading", bold),
            KsmlNode::LineBreak,
            run("plain words ", PLAIN),
            run("bold", bold),
            run(" it", italic),
            run("under", under),
            KsmlNode::LineBreak,
            KsmlNode::LineBreak,
            run("after", PLAIN),
        ]
    );
}

#[test]
fn nested_styles_combine() {
    let doc = parse_ksml_text("<ksml><body><b><i>both</i></b></body></ksml>").unwrap();
    assert_eq!(
        doc.nodes,
        vec![run(
            "both",
            KsmlStyle {
                bold: true,
                italic: true,
                underline: false,
            }
        )]
    );
}

#[test]
fn entities_decode_in_text_and_attributes() {
    let doc = parse_ksml_text(
        r#"<ksml><body><a href="x.ksml?a=1&amp;b=2">&lt;go&gt; &#65;&#x42; &bogus</a></body></ksml>"#,
    )
    .unwrap();
    let KsmlNode::Link(link) = &doc.nodes[0] else {
        panic!("a link");
    };
    assert_eq!(link.href.as_deref(), Some("x.ksml?a=1&b=2"));
    assert_eq!(link.children, vec![run("<go> AB &bogus", PLAIN)]);
}

#[test]
fn links_keep_their_target() {
    let doc = parse_ksml_text(
        r#"<ksml><body>see <a href="topic.ksml" target="pane">the topic</a></body></ksml>"#,
    )
    .unwrap();
    assert_eq!(doc.nodes[0], run("see ", PLAIN));
    assert_eq!(
        doc.nodes[1],
        KsmlNode::Link(KsmlLink {
            href: Some("topic.ksml".into()),
            target: Some("pane".into()),
            children: vec![run("the topic", PLAIN)],
        })
    );
}

#[test]
fn radio_buttons_are_tabs_with_href_and_target() {
    let doc = parse_ksml_text(
        r#"<ksml><body>
  <radiobutton text="First" name="tabs" id="t1" left="25" href="content/one.ksml" target="pane" checked="true"/>
  <radiobutton text="Second" name="tabs" id="t2" href="two.ksml" target="pane"/>
</body></ksml>"#,
    )
    .unwrap();
    let tabs: Vec<&KsmlTab> = doc
        .nodes
        .iter()
        .filter_map(|n| match n {
            KsmlNode::Tab(tab) => Some(tab),
            _ => None,
        })
        .collect();
    assert_eq!(tabs.len(), 2);
    assert_eq!(tabs[0].text, "First");
    assert_eq!(tabs[0].name.as_deref(), Some("tabs"));
    assert_eq!(tabs[0].id.as_deref(), Some("t1"));
    assert_eq!(tabs[0].href.as_deref(), Some("content/one.ksml"));
    assert_eq!(tabs[0].target.as_deref(), Some("pane"));
    assert!(tabs[0].checked);
    assert!(!tabs[1].checked);
}

#[test]
fn iframe_is_a_frame_with_its_size_and_source() {
    let doc = parse_ksml_text(
        r#"<ksml><body><iframe width="230" height="280" src="index.ksml" name="pane" id="pane" left="11"/></body></ksml>"#,
    )
    .unwrap();
    assert_eq!(
        doc.nodes,
        vec![KsmlNode::Frame(KsmlFrame {
            name: Some("pane".into()),
            id: Some("pane".into()),
            src: Some("index.ksml".into()),
            width: Some(230),
            height: Some(280),
        })]
    );
}

#[test]
fn text_attribute_elements_and_other_controls_are_kept() {
    let doc = parse_ksml_text(
        r#"<ksml><body>
  <listbox id="list" height="10"><option value="one.ksml"><text text="   /one"/></option></listbox>
</body></ksml>"#,
    )
    .unwrap();
    let KsmlNode::Control(list) = &doc.nodes[0] else {
        panic!("a control");
    };
    assert_eq!(list.element, "listbox");
    assert_eq!(list.id.as_deref(), Some("list"));
    assert_eq!(list.attributes[1], ("height".into(), "10".into()));
    let KsmlNode::Control(option) = &list.children[0] else {
        panic!("an option");
    };
    assert_eq!(option.children, vec![run("/one", PLAIN)]);
}

#[test]
fn a_mismatched_end_tag_is_an_error() {
    let err = parse_ksml_text("<ksml><body><b>text</i></body></ksml>").unwrap_err();
    assert!(matches!(err, KsmlError::Xml(_)), "{err:?}");
}

#[test]
fn an_unclosed_element_is_an_error() {
    assert!(matches!(
        parse_ksml_text("<ksml><body>text"),
        Err(KsmlError::Xml(_))
    ));
}

#[test]
fn a_root_other_than_ksml_is_rejected() {
    assert_eq!(
        parse_ksml_text("<html><body/></html>"),
        Err(KsmlError::NotKsml("html".into()))
    );
}

#[test]
fn comments_and_doctype_are_skipped() {
    let doc = parse_ksml_text(
        "<?xml version=\"1.0\"?>\n<!DOCTYPE ksml>\n<ksml><!-- note --><body>kept</body></ksml>",
    )
    .unwrap();
    assert_eq!(doc.nodes, vec![run("kept", PLAIN)]);
}

#[test]
fn unquoted_attribute_values_are_an_error() {
    assert!(matches!(
        parse_ksml_text("<ksml><body><text text=abc/></body></ksml>"),
        Err(KsmlError::Xml(_))
    ));
}
