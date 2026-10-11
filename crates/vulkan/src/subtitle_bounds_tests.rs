//! Pinned native formatter bounds replay uses embedded font bytes, without filesystem lookup.
use super::*;
use rustycad_tkernel::{ncollection::String8, standard::Handle, tcollection::AsciiString};
use rustycad_tkservice::graphic3d::*;
const SUBTITLE_TEXTS: [&str; 8] = [
    "AV", "AV   ", " A V ", "A\tV", "AV\nA ", "A\n\nV", "", " \t ",
];
const HORIZONTAL: [Graphic3dHorizontalTextAlignment; 3] = [
    Graphic3dHorizontalTextAlignment::Left,
    Graphic3dHorizontalTextAlignment::Center,
    Graphic3dHorizontalTextAlignment::Right,
];
const VERTICAL: [Graphic3dVerticalTextAlignment; 4] = [
    Graphic3dVerticalTextAlignment::Bottom,
    Graphic3dVerticalTextAlignment::Center,
    Graphic3dVerticalTextAlignment::Top,
    Graphic3dVerticalTextAlignment::Topfirstline,
];
struct EmbeddedSubtitleFont;
impl EmbeddedSubtitleFont {
    fn at_size(size: u32, resolution: u32) -> FontFTFont {
        let font = FontFTFont::new(Handle::default());
        let params = FontFTFontParams {
            point_size: size,
            resolution,
            ..Default::default()
        };
        assert!(font.init_from(
            &FontFontMgr::embed_fallback_font(),
            &AsciiString::from("embedded Subtitle fixture"),
            &params,
            0
        ));
        font
    }
}
fn replay_subtitle_bounds(row: &str) {
    let fields: Vec<f32> = row.split_whitespace().map(|s| s.parse().unwrap()).collect();
    let font = EmbeddedSubtitleFont::at_size(fields[3] as u32, fields[4] as u32);
    let formatter = FontTextFormatter::default();
    formatter.setup_alignment(HORIZONTAL[fields[1] as usize], VERTICAL[fields[2] as usize]);
    formatter
        .append(
            &String8::from_unicode_units(SUBTITLE_TEXTS[fields[0] as usize].as_bytes(), -1),
            &font,
        )
        .unwrap();
    formatter.format().unwrap();
    let rectangle = checked_subtitle_rectangle(&formatter, &fields, row);
    check_layout_formatter_bounds(&font, &formatter, fields[0] as usize, rectangle);
}
fn checked_subtitle_rectangle(
    formatter: &FontTextFormatter,
    fields: &[f32],
    row: &str,
) -> FontRect {
    let rectangle = native_formatter_rectangle(formatter);
    let actual = [
        rectangle.left,
        rectangle.right,
        rectangle.top,
        rectangle.bottom,
    ];
    assert_eq!(actual, fields[5..9], "{row}");
    rectangle
}
fn check_layout_formatter_bounds(
    font: &FontFTFont,
    formatter: &FontTextFormatter,
    text: usize,
    rectangle: FontRect,
) {
    let layout = build_glyph_layout(font, formatter).unwrap();
    if text >= 6 {
        assert!(layout.is_none());
        return;
    }
    let layout = layout.unwrap();
    assert_eq!(layout.formatter_bounds, rectangle);
    assert!(!layout.positions.is_empty());
}
#[test]
fn native_font_bounds_cover_whitespace_tabs_lines_alignment_size_and_resolution() {
    let fixture = include_str!("../tests/fixtures/subtitle_text/native-bounds.txt");
    for row in fixture.lines() {
        replay_subtitle_bounds(row);
    }
    assert_eq!(fixture.lines().count(), 384);
}
