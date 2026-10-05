//! Signed integer cwiseAbs native fixtures exclude minimum-value overflow domains.
use rustycad_tkernel::ncollection::{Vec2, Vec3, Vec4};
use std::fmt::Display;

fn row<T: Display, const N: usize>(name: &str, values: [T; N]) -> String {
    let components = values.map(|value| value.to_string()).join(",");
    format!("{name}={components}")
}

fn narrow_integer_rows() -> [String; 6] {
    [
        row("vec2_i8", Vec2::new(-7_i8, 9).cwise_abs().data()),
        row("vec3_i8", Vec3::new(-1_i8, 2, -3).cwise_abs().data()),
        row("vec4_i8", Vec4::new(-4_i8, 5, -6, 7).cwise_abs().data()),
        row("vec2_i16", Vec2::new(-123_i16, 456).cwise_abs().data()),
        row(
            "vec3_i16",
            Vec3::new(-123_i16, 456, -789).cwise_abs().data(),
        ),
        row(
            "vec4_i16",
            Vec4::new(-111_i16, 222, -333, 444).cwise_abs().data(),
        ),
    ]
}

fn wide_integer_rows() -> [String; 6] {
    [
        row(
            "vec2_i64",
            Vec2::new(-1234567890123_i64, 4).cwise_abs().data(),
        ),
        row("vec3_i64", Vec3::new(-5_i64, 6, -7).cwise_abs().data()),
        row("vec4_i64", Vec4::new(-8_i64, 9, -10, 11).cwise_abs().data()),
        row("vec2_long", Vec2::new(-12_isize, 13).cwise_abs().data()),
        row(
            "vec3_long",
            Vec3::new(-14_isize, 15, -16).cwise_abs().data(),
        ),
        row(
            "vec4_long",
            Vec4::new(-17_isize, 18, -19, 20).cwise_abs().data(),
        ),
    ]
}

fn narrow_minimum_rows() -> [String; 2] {
    [
        row("vec2_i8_min", Vec2::new(i8::MIN, 1).cwise_abs().data()),
        row("vec2_i16_min", Vec2::new(i16::MIN, 1).cwise_abs().data()),
    ]
}

#[test]
fn signed_integer_cwise_abs_matches_pinned_native_values() {
    let actual = narrow_integer_rows()
        .into_iter()
        .chain(wide_integer_rows())
        .chain(narrow_minimum_rows())
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    assert_eq!(
        actual,
        include_str!("fixtures/ncollection_vector_signed_abs.expected")
    );
}
