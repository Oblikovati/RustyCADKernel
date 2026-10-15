//! Translation of OCCT's GTests/BRepFont_PlanarRegion_Test.cxx at 3d097a0.
use rustycad_tkmath::gp::Xy;
use rustycad_tkservice::font::FontGlyphOutlineSegment as Segment;
use rustycad_tkv3d::brep_font::{
    BRepFontCurveId, BRepFontPlanarRegionCurve, BRepFontPlanarRegionUse, BRepFontVertexId,
};

fn assert_near(expected: f64, actual: f64, tolerance: f64) {
    assert!(
        (expected - actual).abs() <= tolerance,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn use_separates_range_from_orientation() {
    let use_ = BRepFontPlanarRegionUse::new(
        BRepFontCurveId::new(7),
        BRepFontVertexId::new(3),
        BRepFontVertexId::new(2),
        0.25,
        0.75,
        true,
    );
    assert_eq!(7, use_.curve_index().value());
    assert_eq!(3, use_.first_vertex().value());
    assert_eq!(2, use_.last_vertex().value());
    assert_eq!(0.25, use_.first_parameter());
    assert_eq!(0.75, use_.last_parameter());
    assert!(use_.is_reversed());
}

#[test]
fn quadratic_bounds_use_interior_extremum() {
    let curve = BRepFontPlanarRegionCurve::new(&Segment::quadratic(
        Xy::new(0.0, 0.0),
        Xy::new(1.0, 2.0),
        Xy::new(2.0, 0.0),
    ));
    let [min_x, min_y, max_x, max_y] = curve.bounds().get().unwrap();
    assert_near(0.0, min_x, 1.0e-12);
    assert_near(0.0, min_y, 1.0e-12);
    assert_near(2.0, max_x, 1.0e-12);
    assert_near(1.0, max_y, 1.0e-12);
}

#[test]
fn cubic_bounds_handle_constant_coordinate() {
    let curve = BRepFontPlanarRegionCurve::new(&Segment::cubic(
        Xy::new(0.0, 2.0),
        Xy::new(4.0, 2.0),
        Xy::new(4.0, 2.0),
        Xy::new(0.0, 2.0),
    ));
    let [min_x, min_y, max_x, max_y] = curve.bounds().get().unwrap();
    assert_near(0.0, min_x, 1.0e-12);
    assert_near(2.0, min_y, 1.0e-12);
    assert_near(3.0, max_x, 1.0e-12);
    assert_near(2.0, max_y, 1.0e-12);
}

#[test]
fn bounds_remain_conservative_when_coefficients_overflow() {
    let limit = f64::MAX;
    let curve = BRepFontPlanarRegionCurve::new(&Segment::quadratic(
        Xy::new(limit, 0.0),
        Xy::new(-limit, 1.0),
        Xy::new(limit, 0.0),
    ));
    let [min_x, min_y, max_x, max_y] = curve.bounds().get().unwrap();
    assert_eq!(-limit, min_x);
    assert_eq!(0.0, min_y);
    assert_eq!(limit, max_x);
    assert_eq!(1.0, max_y);
}
