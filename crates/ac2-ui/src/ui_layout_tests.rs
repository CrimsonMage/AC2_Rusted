//! `/saveui` and `/loadui`: the file's text on synthetic strings (0930's nine lines, the style,
//! window and setting lines, escaping, garbage), and the safe file name. The real HUD is in
//! `ui_layout_hud_tests`.

use crate::ui_layout::*;
use crate::ui_layout_text::{UiLayoutFile, UiLayoutWindow};
use crate::HudStyle;

fn line(tag: &'static str, x: i32, y: i32, width: i32, height: i32) -> UiLayoutLine {
    UiLayoutLine {
        tag,
        x,
        y,
        width,
        height,
    }
}

#[test]
fn a_line_is_written_in_retails_format() {
    assert_eq!(
        ui_layout_to_text(&[
            line("<SOCIA>", 10, -4, 300, 200),
            line("<CHAT4>", 0, 1, 2, 3)
        ]),
        "<SOCIA> X:10 Y: -4 W: 300 H: 200\n<CHAT4> X:0 Y: 1 W: 2 H: 3\n"
    );
}

#[test]
fn text_round_trips_and_the_windows_keep_their_order() {
    let lines: Vec<UiLayoutLine> = UI_LAYOUT_WINDOWS
        .iter()
        .enumerate()
        .map(|(i, (tag, _))| line(tag, i as i32 * 7, -(i as i32), 100 + i as i32, 50))
        .collect();
    assert_eq!(ui_layout_parse(&ui_layout_to_text(&lines)), lines);
    let tags: Vec<&str> = UI_LAYOUT_WINDOWS.iter().map(|w| w.0).collect();
    assert_eq!(
        tags,
        [
            "<SOCIA>", "<QUEST>", "<SKILL>", "<NAVIG>", "<EXAMI>", "<CHAT1>", "<CHAT2>", "<CHAT3>",
            "<CHAT4>"
        ]
    );
}

#[test]
fn unknown_tags_and_short_lines_are_skipped() {
    let text = "<SOCIAL> X:1 Y: 2 W: 3 H: 4\n\
                <SOCIA> X:1 Y: 2 W: 3\n\
                <QUEST> X:a Y: 2 W: 3 H: 4\n\
                garbage\n\
                \n\
                <SKILL> X:5 Y: 6 W: 7 H: 8\n\
                <SKILL> X:5 Y: 6 W: 7 H: 8 trailing\n\
                <NAVIG> X:-5 Y:6 W:7 H:8\r\n";
    assert_eq!(
        ui_layout_parse(text),
        [
            line("<SKILL>", 5, 6, 7, 8),
            line("<SKILL>", 5, 6, 7, 8),
            line("<NAVIG>", -5, 6, 7, 8)
        ]
    );
}

#[test]
fn the_file_name_cannot_leave_the_folder() {
    assert_eq!(ui_layout_file_name("UI-Default"), "UI-Default.txt");
    assert_eq!(
        ui_layout_file_name("../../etc/passwd"),
        "___.._etc_passwd.txt"
    );
    assert_eq!(ui_layout_file_name(".."), "__.txt");
    assert_eq!(ui_layout_file_name("a\\b:c"), "a_b_c.txt");
    assert_eq!(ui_layout_file_name(""), "_.txt");
    let dir = std::path::Path::new("/x/ui");
    assert_eq!(ui_layout_file(dir, "/abs"), dir.join("_abs.txt"));
}

fn window(layout: u32, element: u32, x: i32) -> UiLayoutWindow {
    UiLayoutWindow {
        layout,
        element,
        x,
        y: -x,
        width: 50 + x,
        height: 20,
    }
}

fn full_file(style: HudStyle) -> UiLayoutFile {
    UiLayoutFile {
        nine: vec![line("<SOCIA>", 1, 2, 3, 4), line("<CHAT2>", -5, 6, 7, 8)],
        style: Some(style),
        windows: vec![window(0x2200_0001, 0xA05C_6B95, 10), window(1, 2, -3)],
        settings: vec![
            ("vitals.layout".into(), "stacked".into()),
            ("chat.1.name".into(), "Trade = gold \\ n".into()),
            ("chat.2.name".into(), "two\nlines\r and  spaces ".into()),
            ("offset.radar".into(), "10,-20".into()),
            ("chat.3.name".into(), String::new()),
        ],
    }
}

#[test]
fn the_extended_lines_are_written_after_the_nine_in_a_fixed_form() {
    let text = UiLayoutFile {
        nine: vec![line("<SOCIA>", 10, -4, 300, 200)],
        style: Some(HudStyle::Modern),
        windows: vec![window(0x2200_0001, 0xA05C_6B95, 7)],
        settings: vec![("chat.1.name".into(), "a=b\nc".into())],
    }
    .to_text();
    assert_eq!(
        text,
        "<SOCIA> X:10 Y: -4 W: 300 H: 200\n\
         <STYLE> modern\n\
         <WIN> 22000001:a05c6b95 X:7 Y: -7 W: 57 H: 20\n\
         <SET> chat.1.name=a=b\\nc\n"
    );
    assert!(text.contains("<STYLE> modern"));
}

#[test]
fn both_styles_round_trip_with_escaping() {
    for style in [HudStyle::Modern, HudStyle::Classic] {
        let file = full_file(style);
        assert_eq!(UiLayoutFile::parse(&file.to_text()), file, "{style:?}");
    }
}

#[test]
fn zero_nine_thirty_skips_every_extended_line() {
    // 0930 keeps only a line that gives five values for one of its nine tags.
    let text = full_file(HudStyle::Modern).to_text();
    assert_eq!(
        ui_layout_parse(&text),
        [line("<SOCIA>", 1, 2, 3, 4), line("<CHAT2>", -5, 6, 7, 8)]
    );
    for l in text.lines().skip(2) {
        let words: Vec<&str> = l.split_whitespace().collect();
        let sscanf_ok = words.len() >= 6 && words[1].starts_with("X:");
        assert!(!sscanf_ok, "{l}");
    }
}

#[test]
fn a_bare_zero_nine_thirty_file_has_no_extras() {
    let file = UiLayoutFile::parse("<SOCIA> X:1 Y: 2 W: 3 H: 4\n<QUEST> X:5 Y: 6 W: 7 H: 8\n");
    assert_eq!(file.nine.len(), 2);
    assert_eq!(file.style, None);
    assert!(!file.has_extras());
}

#[test]
fn garbage_and_malformed_extended_lines_are_skipped() {
    let text = "<STYLE> sideways\n\
                <STYLE>\n\
                <WIN> nothex:2 X:1 Y: 2 W: 3 H: 4\n\
                <WIN> 1:2 X:1 Y: 2 W: 3\n\
                <WIN> 12 X:1 Y: 2 W: 3 H: 4\n\
                <WIN> 1:2 X:1 Y: 2 W: 3 H: 4\n\
                <SET> no equals sign\n\
                <SET> =empty key\n\
                <SET> a=1\n\
                <SET> a=2\n\
                <SET>\n\
                \x01\x02 garbage\n\
                <STYLE> classic\n";
    let file = UiLayoutFile::parse(text);
    assert_eq!(file.style, Some(HudStyle::Classic));
    assert_eq!(file.windows, [window_at(1, 2)]);
    assert_eq!(file.settings, [("a".to_string(), "2".to_string())]);
    assert!(file.nine.is_empty());
}

fn window_at(layout: u32, element: u32) -> UiLayoutWindow {
    UiLayoutWindow {
        layout,
        element,
        x: 1,
        y: 2,
        width: 3,
        height: 4,
    }
}

#[test]
fn a_preset_settings_replace_keeps_the_saved_layouts() {
    let mut layout = crate::hud_layout::HudLayout::default();
    layout.set("vitals.layout", "stacked");
    layout.set("offset.strip", "1,2");
    layout.save_preset("Raid", &[]);
    layout.set("extra", "x");
    let before = layout.settings().len();
    assert!(before >= 3);
    layout.replace_settings(&[
        ("offset.radar".into(), "5,6".into()),
        ("preset.Evil.saved".into(), "true".into()),
        ("bad=key".into(), "v".into()),
        (String::new(), "v".into()),
    ]);
    assert_eq!(layout.settings(), [("offset.radar", "5,6")]);
    assert_eq!(layout.presets(), ["Raid"]);
}
