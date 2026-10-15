//! Closed-form checks of the curve evaluator: values, derivatives, Green areas and bounds.
use super::BRepFontPlanarRegionCurve as Curve;
use rustycad_tkmath::gp::Xy;
use rustycad_tkservice::font::FontGlyphOutlineSegment as Segment;

#[test]
fn quadratic_value_and_derivative_match_bernstein_form() {
    let curve = Curve::new(&Segment::quadratic(
        Xy::new(0.0, 0.0),
        Xy::new(1.0, 2.0),
        Xy::new(2.0, 0.0),
    ));
    assert_eq!(curve.value(0.5), Xy::new(1.0, 1.0));
    assert_eq!(curve.d1(0.5), Xy::new(2.0, 0.0));
    assert_eq!(curve.d1(0.0), Xy::new(2.0, 4.0));
}

#[test]
fn closed_square_signed_area_is_its_area() {
    let corners = [
        Xy::new(0.0, 0.0),
        Xy::new(2.0, 0.0),
        Xy::new(2.0, 3.0),
        Xy::new(0.0, 3.0),
    ];
    let area: f64 = (0..4)
        .map(|i| Curve::new(&Segment::line(corners[i], corners[(i + 1) % 4])).signed_area())
        .sum();
    assert_eq!(area, 6.0);
}

#[test]
fn parabola_area_against_chord_is_two_thirds_of_hull() {
    // Area between y = 2t(1-t)*2 arc and its chord: 2/3 of the control triangle (base 2, height 2).
    let arc = Curve::new(&Segment::quadratic(
        Xy::new(0.0, 0.0),
        Xy::new(1.0, 2.0),
        Xy::new(2.0, 0.0),
    ));
    let chord = Curve::new(&Segment::line(Xy::new(2.0, 0.0), Xy::new(0.0, 0.0)));
    assert!((arc.signed_area() + chord.signed_area() + 4.0 / 3.0).abs() < 1e-15);
}

#[test]
fn cubic_value_hits_endpoints_and_with_endpoints_keeps_controls() {
    let curve = Curve::new(&Segment::cubic(
        Xy::new(0.0, 0.0),
        Xy::new(1.0, 1.0),
        Xy::new(2.0, 1.0),
        Xy::new(3.0, 0.0),
    ));
    assert_eq!(curve.value(0.0), Xy::new(0.0, 0.0));
    assert_eq!(curve.value(1.0), Xy::new(3.0, 0.0));
    let moved = curve.with_endpoints(Xy::new(-1.0, 0.0), Xy::new(4.0, 0.0));
    assert_eq!(*moved.control1(), Xy::new(1.0, 1.0));
    assert_eq!(*moved.start(), Xy::new(-1.0, 0.0));
}

#[test]
fn line_bounds_have_no_gap() {
    let curve = Curve::new(&Segment::line(Xy::new(1.0, 2.0), Xy::new(-3.0, 5.0)));
    assert_eq!(curve.bounds().get().unwrap(), [-3.0, 2.0, 1.0, 5.0]);
}
