use rustycad_tkernel::{
    ErrorKind,
    ncollection::{HashKey, Vec4},
    quantity::{Color, ColorRgba, Date, NamedColor, Period},
    standard::{dump::DumpStream, hash_bytes},
};

#[test]
fn rgba_constructors_and_accessors() {
    let red = Color::named(NamedColor::RED);
    assert_eq!(ColorRgba::from_color(red).alpha(), 1.0);
    let rgba = ColorRgba::from_vec4(Vec4::new(1.0, 0.0, 0.0, 7.0)).expect("rgb in range");
    assert_eq!(rgba.to_vec4().data(), [1.0, 0.0, 0.0, 7.0]);
    assert!(ColorRgba::from_vec4(Vec4::new(1.5, 0.0, 0.0, 1.0)).is_err());
    let mut changed = rgba;
    changed.set_rgb(Color::named(NamedColor::BLUE));
    assert_eq!(
        (changed.rgb().rgb(), changed.alpha()),
        ([0.0, 0.0, 1.0], 7.0)
    );
}

#[test]
fn rgba_names_and_hex() {
    let named = ColorRgba::from_name("quantity_noc_red").expect("known name");
    assert_eq!((named.rgb().rgb(), named.alpha()), ([1.0, 0.0, 0.0], 1.0));
    assert!(ColorRgba::from_name("nope").is_none());
    let parsed = ColorRgba::from_hex("#FF000080", false).expect("valid");
    assert_eq!(parsed.alpha(), 128.0 / 255.0);
    assert!(ColorRgba::from_hex("#FF000080", true).is_none());
    let text = ColorRgba::to_hex(parsed, true);
    assert_eq!(text.as_bytes(), b"#FF000080");
    // Alpha is not clamped: 2.0 * 255 + 0.5 prints as the three digits of 510.
    let bright = ColorRgba::from_rgb(Color::named(NamedColor::BLACK), 2.0);
    assert_eq!(ColorRgba::to_hex(bright, false).as_bytes(), b"0000001FE");
}

#[test]
fn rgba_vector_gamma_keeps_alpha() {
    let encoded = ColorRgba::linear_to_srgb_vec4([0.5, 0.0, 1.0, -3.0]);
    assert_eq!(encoded[3], -3.0);
    assert_eq!(encoded[0], Color::linear_to_srgb_f32(0.5));
    let decoded = ColorRgba::srgb_to_linear_vec4([0.5, 0.0, 1.0, 0.25]);
    assert_eq!(
        (decoded[0], decoded[3]),
        (Color::srgb_to_linear_f32(0.5), 0.25)
    );
}

#[test]
fn rgba_hash_and_json() {
    let rgba = ColorRgba::new(1.0, 0.5, 0.0, 0.5).expect("valid");
    assert_eq!(rgba.occt_hash(), hash_bytes(&[50, 255, 127, 0]));
    let mut stream = DumpStream::default();
    rgba.dump_json(&mut stream).expect("in-memory");
    assert_eq!(stream.text(), "\"RGBA\": [1, 0.5, 0, 0.5]");
    let mut parsed = ColorRgba::default();
    let mut position = 1;
    assert_eq!(
        parsed.init_from_json(stream.text(), &mut position),
        Ok(true)
    );
    assert_eq!(
        (parsed.to_vec4().data(), position),
        ([1.0, 0.5, 0.0, 0.5], 1)
    );
}

#[test]
fn date_and_period_operators_match_the_named_methods() {
    let first = Period::new(5, 6).expect("valid");
    let second = Period::new(5, 5).expect("valid");
    assert!(first > second && second < first && first != second);
    assert_eq!((first - second).expect("difference").values(), (0, 1));
    assert_eq!((first + second).expect("sum").values(), (10, 11));
    let epoch = Date::default();
    let later = (epoch + first).expect("later date");
    assert!(later > epoch && epoch < later);
    let error = (epoch - first).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::DateDefinition);
    assert_eq!(error.exception_type(), "Quantity_DateDefinitionError");
    let invalid = Period::new(-1, 0).unwrap_err();
    assert_eq!(invalid.exception_type(), "Quantity_PeriodDefinitionError");
}
