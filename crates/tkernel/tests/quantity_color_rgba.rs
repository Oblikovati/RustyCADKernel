//! All legacy Quantity_ColorRGBA cases with original tolerances and inputs.
#![allow(non_snake_case)]
use rustycad_tkernel::quantity::{Color, ColorRgba, ColorSpace};
fn rgba(r: f32, g: f32, b: f32, a: f32) -> ColorRgba {
    ColorRgba::new(r, g, b, a).unwrap()
}
fn near(actual: f64, expected: f32, tolerance: f64) {
    assert!(
        (actual - f64::from(expected)).abs() < tolerance,
        "{actual} vs {expected}"
    );
}
fn rgb_near(color: &ColorRgba, expected: [f32; 3], tolerance: f64) {
    for (actual, expected) in color.rgb().rgb().into_iter().zip(expected) {
        near(actual.into(), expected, tolerance);
    }
}
#[test]
fn Quantity_ColorRGBATest__BasicConstruction() {
    let first = ColorRgba::default();
    rgb_near(&first, [1.0, 1.0, 0.0], 0.001);
    near(first.alpha().into(), 1.0, 0.001);
    let second = ColorRgba::from_rgb(Color::new(0.5, 0.6, 0.7, ColorSpace::Rgb).unwrap(), 0.8);
    rgb_near(&second, [0.5, 0.6, 0.7], 0.001);
    near(second.alpha().into(), 0.8, 0.001);
    let third = rgba(0.3, 0.4, 0.5, 0.6);
    rgb_near(&third, [0.3, 0.4, 0.5], 0.001);
    near(third.alpha().into(), 0.6, 0.001);
}
#[test]
fn Quantity_ColorRGBATest__ConstexprGetters() {
    near(rgba(0.2, 0.4, 0.6, 0.8).alpha().into(), 0.8, 0.001);
}
#[test]
fn Quantity_ColorRGBATest__SetValues() {
    let mut color = ColorRgba::default();
    color.set_values(0.1, 0.2, 0.3, 0.4).unwrap();
    rgb_near(&color, [0.1, 0.2, 0.3], 0.001);
    near(color.alpha().into(), 0.4, 0.001);
}
#[test]
fn Quantity_ColorRGBATest__HexColorParsing_RGB() {
    let mut color = ColorRgba::default();
    for (text, expected) in [
        ("#FF0000", [1.0, 0.0, 0.0]),
        ("00FF00", [0.0, 1.0, 0.0]),
        ("#0000FF", [0.0, 0.0, 1.0]),
    ] {
        assert!(color.set_hex(text, true));
        rgb_near(&color, expected, f64::from(0.01_f32));
    }
}
#[test]
fn Quantity_ColorRGBATest__HexColorParsing_RGBA() {
    let mut color = ColorRgba::default();
    assert!(color.set_hex("#FF000080", false));
    rgb_near(&color, [1.0, 0.0, 0.0], f64::from(0.01_f32));
    near(color.alpha().into(), 0.5, f64::from(0.02_f32));
}
#[test]
fn Quantity_ColorRGBATest__HexColorParsing_ShortRGB() {
    let mut color = ColorRgba::default();
    assert!(color.set_hex("#F00", true));
    rgb_near(&color, [1.0, 0.0, 0.0], f64::from(0.1_f32));
    assert!(color.set_hex("#FFF", true));
    rgb_near(&color, [1.0, 1.0, 1.0], f64::from(0.1_f32));
}
#[test]
fn Quantity_ColorRGBATest__HexColorParsing_ShortRGBA() {
    let mut color = ColorRgba::default();
    assert!(color.set_hex("#F008", false));
    rgb_near(&color, [1.0, 0.0, 0.0], f64::from(0.1_f32));
    near(color.alpha().into(), 0.5, f64::from(0.1_f32));
}
#[test]
fn Quantity_ColorRGBATest__HexColorParsing_Invalid() {
    let mut color = ColorRgba::default();
    for text in ["", "#FFFF", "#GGGGGG", "#FF0000FF"] {
        assert!(!color.set_hex(text, true));
    }
}
#[test]
fn Quantity_ColorRGBATest__HexColorParsing_MixedCase() {
    let mut color = ColorRgba::default();
    assert!(color.set_hex("#FfAa00", true));
    rgb_near(&color, [1.0, 0.402, 0.0], f64::from(0.01_f32));
}
#[test]
fn Quantity_ColorRGBATest__HexColorParsing_SpecificValues() {
    let mut color = ColorRgba::default();
    assert!(color.set_hex("#102030", true));
    rgb_near(&color, [0.00518, 0.01444, 0.02956], f64::from(0.0001_f32));
}
#[test]
fn Quantity_ColorRGBATest__EqualityComparison() {
    let first = rgba(0.5, 0.6, 0.7, 0.8);
    let second = rgba(0.5, 0.6, 0.7, 0.8);
    let third = rgba(0.5, 0.6, 0.7, 0.9);
    assert!(first.is_equal(second));
    assert!(!first.is_equal(third));
}
#[test]
fn Quantity_ColorRGBATest__RGBAccess() {
    let mut color = rgba(0.2, 0.4, 0.6, 0.8);
    rgb_near(&color, [0.2, 0.4, 0.6], 0.001);
    color
        .change_rgb()
        .set_values(
            f64::from(0.3_f32),
            f64::from(0.5_f32),
            f64::from(0.7_f32),
            ColorSpace::Rgb,
        )
        .unwrap();
    rgb_near(&color, [0.3, 0.5, 0.7], 0.001);
    near(color.alpha().into(), 0.8, 0.001);
}
#[test]
fn Quantity_ColorRGBATest__SetAlpha() {
    let mut color = rgba(0.5, 0.5, 0.5, 1.0);
    near(color.alpha().into(), 1.0, 0.001);
    color.set_alpha(0.5);
    near(color.alpha().into(), 0.5, 0.001);
    rgb_near(&color, [0.5, 0.5, 0.5], 0.001);
}
#[test]
fn Quantity_ColorRGBATest__EdgeCases() {
    near(rgba(0.0, 0.0, 0.0, 0.0).alpha().into(), 0.0, 0.001);
    let opaque = rgba(1.0, 1.0, 1.0, 1.0);
    near(opaque.alpha().into(), 1.0, 0.001);
    near(opaque.rgb().red(), 1.0, 0.001);
}
#[test]
fn Quantity_ColorRGBATest__ComponentOrder() {
    let mut color = ColorRgba::default();
    assert!(color.set_hex("#123456", true));
    rgb_near(&color, [0.00605, 0.03434, 0.09306], f64::from(0.0001_f32));
}
