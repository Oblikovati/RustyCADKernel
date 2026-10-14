//! Exact native enum-range values survive safe conversion and real stored aspects.
use super::*;
use crate::font::{FontFontAspect, FontFontMgr};
use crate::graphic3d::{Graphic3dAspectLine3d, Graphic3dAspectsLineMembers};
use rustycad_tkernel::quantity::Color;
#[test]
fn every_native_style_discriminant_round_trips_and_outside_range_is_rejected() {
    check_native_range(-4, 3, |v| {
        AspectInteriorStyle::from_native_value(v).map(|x| x as i32)
    });
    check_native_range(-8, 7, |v| {
        AspectTypeOfLine::from_native_value(v).map(|x| x as i32)
    });
    check_native_range(-16, 15, |v| {
        AspectTypeOfMarker::from_native_value(v).map(|x| x as i32)
    });
}
#[test]
fn native_text_hatch_and_font_discriminants_keep_their_full_ranges() {
    check_native_range(0, 7, |v| {
        AspectTypeOfDisplayText::from_native_value(v).map(|x| x as i32)
    });
    check_native_range(0, 15, |v| {
        AspectHatchStyle::from_native_value(v).map(|x| x as i32)
    });
    check_native_range(0, 1, |v| {
        AspectTypeOfStyleText::from_native_value(v).map(|x| x as i32)
    });
    check_native_range(-4, 3, |v| {
        FontFontAspect::from_native_value(v).map(|x| x as i32)
    });
}
fn check_native_range(lower: i32, upper: i32, convert: impl Fn(i32) -> Option<i32>) {
    assert_eq!(convert(lower - 1), None);
    for value in lower..=upper {
        assert_eq!(convert(value), Some(value));
    }
    assert_eq!(convert(upper + 1), None);
}
#[test]
fn unnamed_line_type_is_stored_while_native_pattern_falls_through_to_solid() {
    let kind = AspectTypeOfLine::from_native_value(-2).unwrap();
    let line = Graphic3dAspectLine3d::new(Color::default(), kind, 1.).unwrap();
    assert_eq!(line.line_kind() as i32, -2);
    assert_eq!(line.line_pattern(), 0xFFFF);
    assert_eq!(
        FontFontMgr::font_aspect_to_string(FontFontAspect::NativeMinus2),
        "invalid"
    );
}
