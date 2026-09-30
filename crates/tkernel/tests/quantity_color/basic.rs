#[test]
fn Quantity_ColorTest__BasicConstruction() {
    components(Color::default(), [1.0, 1.0, 0.0], 0.001);
    components(rgb(0.5, 0.6, 0.7), [0.5, 0.6, 0.7], 0.001);
    components(Color::named(NamedColor::RED), [1.0, 0.0, 0.0], 0.001);
}
#[test]
fn Quantity_ColorTest__ConstexprGetters() {
    let color = rgb(0.3, 0.5, 0.7);
    near(color.red(), 0.3, 0.001);
    near(color.green(), 0.5, 0.001);
    near(color.blue(), 0.7, 0.001);
}
#[test]
fn Quantity_ColorTest__EqualityComparison() {
    let first = rgb(0.5, 0.6, 0.7);
    let second = rgb(0.5, 0.6, 0.7);
    let third = rgb(0.5, 0.6, 0.8);
    assert!(first.is_equal(second));
    assert_eq!(first, second);
    assert!(!first.is_different(second));
    assert!(!(first != second));
    assert!(!first.is_equal(third));
    assert!(!(first == third));
    assert!(first.is_different(third));
    assert_ne!(first, third);
}
#[test]
fn Quantity_ColorTest__DistanceCalculation() {
    let first = rgb(0.0, 0.0, 0.0);
    let second = rgb(0.3, 0.4, 0.0);
    near(first.distance(second), 0.5, 0.001);
    near(first.square_distance(second), 0.25, 0.001);
}
#[test]
fn Quantity_ColorTest__NamedColors() {
    components(Color::named(NamedColor::RED), [1.0, 0.0, 0.0], 0.001);
    let green = Color::named(NamedColor::GREEN);
    near(green.red(), 0.0, 0.001);
    assert!(green.green() > 0.5);
    near(green.blue(), 0.0, 0.001);
    components(Color::named(NamedColor::BLUE), [0.0, 0.0, 1.0], 0.001);
}
#[test]
fn Quantity_ColorTest__SetValues() {
    let mut color = Color::default();
    color.set_values(0.2, 0.4, 0.6, ColorSpace::Rgb).unwrap();
    components(color, [0.2, 0.4, 0.6], 0.001);
    color.set_named(NamedColor::YELLOW);
    components(color, [1.0, 1.0, 0.0], 0.001);
}
#[test]
fn Quantity_ColorTest__HLS_Extraction() {
    let red = Color::named(NamedColor::RED);
    assert!(red.hue().abs() < 5.0 || (red.hue() - 360.0).abs() < 5.0);
    near(red.light(), 1.0, 0.01);
    near(red.saturation(), 1.0, 0.01);
}
#[test]
fn Quantity_ColorTest__EpsilonThreadSafety() {
    let original = Color::epsilon();
    Color::set_epsilon(0.0002);
    near(Color::epsilon(), 0.0002, 0.001);
    Color::set_epsilon(original);
    near(Color::epsilon(), original, 0.001);
}
#[test]
fn Quantity_ColorTest__ColorNameString() {
    assert_eq!(NamedColor::RED.name(), "RED");
    assert_eq!(NamedColor::BLUE.name(), "BLUE");
}
#[test]
fn Quantity_ColorTest__EdgeCases() {
    components(rgb(0.0, 0.0, 0.0), [0.0, 0.0, 0.0], 0.001);
    components(rgb(1.0, 1.0, 1.0), [1.0, 1.0, 1.0], 0.001);
    assert!(rgb(0.5, 0.5, 0.5).is_equal(rgb(0.50001, 0.50001, 0.50001)));
}
