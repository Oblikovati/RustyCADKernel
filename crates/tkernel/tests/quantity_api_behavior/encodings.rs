use rustycad_tkernel::{
    ncollection::HashKey,
    quantity::{Color, ColorSpace, NamedColor},
    standard::{dump::DumpStream, hash_bytes},
};

fn text(value: &rustycad_tkernel::tcollection::AsciiString) -> &str {
    std::str::from_utf8(value.as_bytes()).expect("hex is ASCII")
}

#[test]
fn hex_round_trip_uses_srgb() {
    let color = Color::from_hex("#808080").expect("valid hex");
    assert!((color.red() - 0.2158605).abs() < 1e-6);
    assert_eq!(text(&Color::to_hex(color, true)), "#808080");
    assert_eq!(
        text(&Color::to_hex(Color::named(NamedColor::MATRABLUE), false)),
        "0000BF"
    );
    assert!(Color::from_hex("#FF0000FF").is_none(), "alpha is refused");
    assert!(Color::from_hex("FF0").is_none(), "short form needs '#'");
}

#[test]
fn argb_packs_linear_channels_without_gamma() {
    let gray = Color::new(-1.0, 0.5, 0.0, ColorSpace::Hls).expect("gray");
    // Linear 0.214041 * 255 + 0.5 truncates to 55 = 0x37 in every channel.
    assert_eq!(Color::to_argb(gray), 0x373737);
    let red = Color::from_argb(0x7fff_0000);
    assert_eq!(red.rgb(), [1.0, 0.0, 0.0]);
    let from_negative = Color::from_argb(-1);
    assert_eq!(from_negative.rgb(), [1.0, 1.0, 1.0]);
}

#[test]
fn hls_helpers_round_through_float() {
    assert_eq!(Color::hls_rgb(0.0, 1.0, 1.0), Ok([1.0, 0.0, 0.0]));
    assert_eq!(Color::rgb_hls(1.0, 0.0, 0.0), [0.0, 1.0, 1.0]);
    assert_eq!(Color::hls_rgb(-1.0, 0.25, 0.0), Ok([0.25; 3]));
    assert!(Color::hls_rgb(420.0, 0.5, 0.5).is_err());
    // A NaN hue reaches the native "Color out" throw on x86-64 (INT_MIN hue index).
    assert!(Color::hls_to_srgb([f32::NAN, 0.5, 0.5]).is_err());
    assert_eq!(
        Color::hls_to_linear_rgb([0.0, 1.0, 1.0]),
        Ok([1.0, 0.0, 0.0])
    );
    // The float gamma encoding of 1.0 lands one ulp below 1.0.
    assert_eq!(
        Color::linear_rgb_to_hls([0.0, 0.0, 1.0]),
        [240.0, 0.99999994, 1.0]
    );
}

#[test]
fn gamma_vector_and_approx22_conversions() {
    assert_eq!(Color::linear_to_srgb_vec3([0.0, 1.0, 0.5])[0], 0.0);
    assert_eq!(
        Color::srgb_to_linear_vec3([0.5; 3]),
        [Color::srgb_to_linear(0.5); 3]
    );
    assert_eq!(
        Color::linear_to_srgb_vec3_f32([0.5; 3]),
        [Color::linear_to_srgb_f32(0.5); 3]
    );
    assert_eq!(
        Color::srgb_to_linear_vec3_f32([0.5; 3]),
        [Color::srgb_to_linear_f32(0.5); 3]
    );
    // Native naming quirk: the "linear to sRGB" approximation raises to 2.2.
    assert_eq!(Color::linear_to_srgb_approx22(0.5), 0.5_f32.powf(2.2));
    assert_eq!(Color::srgb_to_linear_approx22(0.5), 0.5_f32.powf(1.0 / 2.2));
    assert_eq!(Color::linear_to_srgb_approx22_vec3([1.0; 3]), [1.0; 3]);
    assert_eq!(Color::srgb_to_linear_approx22_vec3([0.0; 3]), [0.0; 3]);
}

#[test]
fn hash_uses_truncated_bytes() {
    let color = Color::new(1.0, 0.5, 0.0, ColorSpace::Rgb).expect("valid");
    assert_eq!(color.occt_hash(), hash_bytes(&[255, 127, 0]));
}

#[test]
fn json_dump_and_init_keep_the_native_cursor_quirk() {
    let mut stream = DumpStream::default();
    Color::named(NamedColor::MATRABLUE)
        .dump_json(&mut stream)
        .expect("in-memory");
    assert_eq!(stream.text(), "\"RGB\": [0, 0, 0.522522]");
    let mut color = Color::named(NamedColor::WHITE);
    let mut position = 1;
    assert_eq!(
        color.init_from_json("\"RGB\": [1, 0.5, 0]", &mut position),
        Ok(true)
    );
    assert_eq!((color.rgb(), position), ([1.0, 0.5, 0.0], 1));
    assert_eq!(
        color.init_from_json("\"rgb\": [1, 0, 0]", &mut position),
        Ok(false)
    );
    let failure = color
        .init_from_json("\"RGB\": [2, 0, 0]", &mut position)
        .unwrap_err();
    assert_eq!(failure.message(), "Color out");
}
