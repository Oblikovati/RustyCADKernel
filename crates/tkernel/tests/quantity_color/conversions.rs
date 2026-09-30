#[test]
fn Quantity_ColorTest__RGB_to_HLS_Conversion() {
    let hls = Color::srgb_to_hls(Color::named(NamedColor::RED).rgb());
    near(hls[0].into(), 0.0, 1.0);
    near(hls[1].into(), 1.0, 0.001);
    near(hls[2].into(), 1.0, 0.001);
    let gray = Color::srgb_to_hls(rgb(0.5, 0.5, 0.5).rgb());
    near(gray[1].into(), 0.5, 0.001);
    near(gray[2].into(), 0.0, 0.001);
}
#[test]
fn Quantity_ColorTest__LinearRGB_to_Lab_Conversion() {
    let lab = Color::linear_rgb_to_lab(rgb(1.0, 1.0, 1.0).rgb());
    near(lab[0].into(), 100.0, 1.0);
    near(lab[1].into(), 0.0, 5.0);
    near(lab[2].into(), 0.0, 5.0);
    let black = Color::linear_rgb_to_lab(rgb(0.0, 0.0, 0.0).rgb());
    near(black[0].into(), 0.0, 1.0);
}
#[test]
fn Quantity_ColorTest__Lab_to_Lch_Conversion() {
    let lch = Color::lab_to_lch([50.0, 25.0, 25.0]);
    near(lch[0].into(), 50.0, 0.001);
    near(lch[1].into(), 35.36, 0.1);
    near(lch[2].into(), 45.0, 1.0);
}
#[test]
fn Quantity_ColorTest__Lch_to_Lab_RoundTrip() {
    let original = [50.0, 25.0, 25.0];
    let result = Color::lch_to_lab(Color::lab_to_lch(original));
    for (a, b) in original.into_iter().zip(result) {
        near(a.into(), b.into(), 0.01);
    }
}
#[test]
fn Quantity_ColorTest__Lab_to_RGB_RoundTrip() {
    let original = rgb(0.5, 0.6, 0.7);
    let result = Color::lab_to_linear_rgb(Color::linear_rgb_to_lab(original.rgb()));
    for (a, b) in original.rgb().into_iter().zip(result) {
        near(a.into(), b.into(), 0.01);
    }
}
#[test]
fn Quantity_ColorTest__DeltaE2000_Calculation() {
    let first = rgb(0.5, 0.6, 0.7);
    let second = rgb(0.5, 0.6, 0.7);
    near(first.delta_e2000(second), 0.0, 0.01);
    assert!(first.delta_e2000(rgb(0.3, 0.4, 0.5)) > 0.0);
}
fn check_conversion_samples(samples: &[[f64; 6]], space: ColorSpace) {
    for &[r, g, b, c1, c2, c3] in samples {
        let actual = rgb(r, g, b).values(space);
        for (a, b) in actual.into_iter().zip([c1, c2, c3]) {
            assert!((a - b).abs() <= 1e-4, "{a} != {b}");
        }
    }
}
#[test]
fn Quantity_ColorTest__Draw_RGBToLabReferenceSamples() {
    check_conversion_samples(&DRAW_RGBTOLABREFERENCESAMPLES, ColorSpace::Lab);
}
#[test]
fn Quantity_ColorTest__Draw_RGBToLchReferenceSamples() {
    check_conversion_samples(&DRAW_RGBTOLCHREFERENCESAMPLES, ColorSpace::Lch);
}
#[test]
fn Quantity_ColorTest__Draw_ColorSpaceRoundTripStability() {
    for index in 1..1000 {
        let expected = [
            (f64::from(index) * 0.6180339887498949) % 1.0,
            (f64::from(index) * 0.4142135623730950) % 1.0,
            (f64::from(index) * 0.7320508075688773) % 1.0,
        ];
        let original = rgb(expected[0], expected[1], expected[2]);
        for space in [ColorSpace::Lab, ColorSpace::Lch] {
            let [a, b, c] = original.values(space);
            let mut result = Color::default();
            result.set_values(a, b, c, space).unwrap();
            components(result, expected, 1e-4);
        }
    }
}
fn check_difference_samples(samples: &[[f64; 8]]) {
    for &[l1, a1, b1, l2, a2, b2, expected, tolerance] in samples {
        let first = Color::new(l1, a1, b1, ColorSpace::Lab).unwrap();
        let second = Color::new(l2, a2, b2, ColorSpace::Lab).unwrap();
        let actual = first.delta_e2000(second);
        assert!(
            (actual - expected).abs() <= tolerance,
            "{actual} != {expected}, tolerance {tolerance}"
        );
    }
}
#[test]
fn Quantity_ColorTest__Draw_CIEDE2000ReferenceSamples() {
    check_difference_samples(&DRAW_CIEDE2000REFERENCESAMPLES);
}
#[test]
fn Quantity_ColorTest__Draw_CIEDE2000SharmaReferenceSamples() {
    check_difference_samples(&DRAW_CIEDE2000SHARMAREFERENCESAMPLES);
}
