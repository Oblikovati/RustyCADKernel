//! Bisector regressions: the bisector of two points (the full comparison with native
//! OCCT is the tp_bisec_* probe corpus).
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCartesianPoint, Geom2dCurve, Geom2dPoint};
use rustycad_tkmath::gp::{Point2, Vector2};
use rustycad_tktopalgo::bisector::Bisec;

fn point(x: f64, y: f64) -> Handle<dyn Geom2dPoint> {
    Handle::new(Geom2dCartesianPoint::new(x, y)).cast()
}

#[test]
fn bisector_of_two_points_is_equidistant() {
    let (a, b) = (Point2::new(0.0, 0.0), Point2::new(2.0, 0.0));
    let mut bisec = Bisec::new();
    bisec
        .perform_points(
            &point(0.0, 0.0),
            &point(2.0, 0.0),
            Point2::new(1.0, 0.0),
            Vector2::new(-1.0, 0.0),
            Vector2::new(1.0, 0.0),
            1.0,
            1.0e-7,
            false,
        )
        .unwrap();
    let curve = bisec.value();
    let first = curve.first_parameter();
    for step in 0..4 {
        let p = curve.value(first + f64::from(step)).unwrap();
        assert!((p.distance(a) - p.distance(b)).abs() < 1.0e-12, "{p:?}");
    }
}
