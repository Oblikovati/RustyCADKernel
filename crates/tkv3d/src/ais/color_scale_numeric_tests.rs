//! Rust-only oracle replay for discrete lookup, automatic formatting and CIE uniform palettes.
use super::*;
use rustycad_tkernel::{
    quantity::{Color, ColorSpace, NamedColor},
    tcollection::ExtendedString,
};
use rustycad_tkservice::aspect::AspectTypeOfColorScaleData;
const NUMERIC: &str = include_str!("../../tests/fixtures/color-scale-native-numeric.txt");
fn rows(kind: &str) -> impl Iterator<Item = Vec<&str>> + '_ {
    NUMERIC
        .lines()
        .filter(move |line| line.split_whitespace().next() == Some(kind))
        .map(|line| line.split_whitespace().skip(1).collect())
}
fn close_colors(color: Color, expected: &[&str]) {
    let actual = color.values(ColorSpace::Rgb);
    for (a, b) in actual.into_iter().zip(expected) {
        let b = b.parse::<f64>().unwrap();
        assert!(
            (a - b).abs() <= 2e-6,
            "color={actual:?}; expected={expected:?}"
        );
    }
}
#[test]
fn native_discrete_color_lookup_preserves_failed_static_output() {
    for row in rows("find") {
        let mut color = Color::named(NamedColor::GREEN);
        let found = AisColorScale::find_color_default(
            row[1].parse().unwrap(),
            0.0,
            1.0,
            row[0].parse().unwrap(),
            &mut color,
        )
        .unwrap();
        assert_eq!(found, row[2] == "1");
        close_colors(color, &row[3..]);
    }
}
#[test]
fn native_cie_uniform_palette_generation() {
    for row in rows("uniform") {
        let count = row[0].parse().unwrap();
        let from = row[1].parse::<f64>().unwrap();
        let colors = AisColorScale::make_uniform_colors(count, 50.0, from, from + 300.0).unwrap();
        assert_eq!(colors.len(), row[2].parse::<usize>().unwrap());
        for (color, expected) in colors.into_iter().zip(row[3..].chunks(3)) {
            close_colors(color, expected);
        }
    }
}
#[test]
fn native_linear_logarithmic_and_middle_labels() {
    let scale = AisColorScale::new().unwrap();
    scale.set_number_of_intervals(4);
    for row in rows("label") {
        scale.set_range(
            if row[0] == "1" { 1.0 } else { -2.0 },
            if row[0] == "1" { 1000.0 } else { 9.0 },
        );
        scale.set_logarithmic(row[0] == "1");
        scale.set_label_at_border(row[1] == "1");
        let label = scale.get_label(row[2].parse().unwrap()).unwrap();
        assert_eq!(
            String::from_utf16_lossy(label.as_utf16()),
            row[3].trim_matches('"')
        );
    }
}
#[test]
fn native_custom_label_index_bug_retains_errors() {
    let scale = AisColorScale::new().unwrap();
    scale.set_label_type(AspectTypeOfColorScaleData::User);
    scale.set_label(&ExtendedString::from_utf8("last"), 3);
    for row in rows("custom-label") {
        let label = scale.get_label(row[0].parse().unwrap());
        if row[1] == "error" {
            assert!(label.is_err());
            continue;
        }
        assert_eq!(
            String::from_utf16_lossy(label.unwrap().as_utf16()),
            row[2].trim_matches('"')
        );
    }
}
#[test]
fn configured_hue_changes_interval_colors_but_not_native_automatic_find_color() {
    let scale = AisColorScale::new().unwrap();
    scale.set_hue_range(60.0, 300.0);
    let mut actual = Color::default();
    scale.find_color(0.0, &mut actual).unwrap();
    let mut default = Color::default();
    AisColorScale::find_color_default(0.0, 0.0, 1.0, 10, &mut default).unwrap();
    assert_eq!(actual, default);
    assert_ne!(actual, scale.get_interval_color(1).unwrap());
    assert!(!scale.find_color(-1.0, &mut actual).unwrap());
    assert_eq!(actual, Color::default());
}
#[test]
fn scale_uniform_setter_and_user_lookup_use_actual_generated_colors() {
    let scale = AisColorScale::new().unwrap();
    scale.set_number_of_intervals(7);
    scale.set_uniform_colors(50.0, -30.0, 270.0).unwrap();
    assert_eq!(scale.get_color_type(), AspectTypeOfColorScaleData::User);
    let expected = AisColorScale::make_uniform_colors(7, 50.0, -30.0, 270.0).unwrap();
    assert_eq!(&*scale.colors(), &expected);
    let mut color = Color::default();
    scale.find_color(1.0, &mut color).unwrap();
    assert_eq!(color, *expected.last().unwrap());
    assert_eq!(scale.get_interval_color(99).unwrap(), Color::default());
}
#[test]
fn native_full_circle_reverse_and_multiturn_uniform_palettes() {
    for row in rows("uniform-extra") {
        let count = row[0].parse().unwrap();
        let span = row[1].parse::<f64>().unwrap();
        let colors = AisColorScale::make_uniform_colors(count, 65.0, -25.0, -25.0 + span).unwrap();
        assert_eq!(colors.len(), row[2].parse::<usize>().unwrap());
        for (color, expected) in colors.into_iter().zip(row[3..].chunks(3)) {
            close_colors(color, expected);
        }
    }
}
#[test]
fn native_static_lookup_interpolates_all_configured_hls_channels() {
    for row in rows("find-hls") {
        let mut color = Color::default();
        let found = AisColorScale::find_color_hls(
            row[0].parse().unwrap(),
            0.0,
            1.0,
            4,
            [120.0, 0.7, 0.6],
            [-30.0, 0.9, 0.8],
            &mut color,
        )
        .unwrap();
        assert_eq!(found, row[1] == "1");
        close_colors(color, &row[2..]);
    }
}
