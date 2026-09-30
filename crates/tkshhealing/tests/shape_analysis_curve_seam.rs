//! Seam direction precedence, tie decisions and unsupported-curve return value.
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve, Geom2dLine};
use rustycad_tkmath::gp::{Axis2d, Axis22d, Direction2, Point2};
use rustycad_tkshhealing::shape_analysis::ShapeAnalysisCurve;
fn seam_at(y: f64, direction: Direction2) -> Handle<dyn Geom2dCurve> {
    Handle::new(Geom2dLine::new(Axis2d::new(Point2::new(0.0, y), direction))).cast()
}
#[test]
fn positive_x_prefers_lower_y_and_ties_choose_second() {
    let first = seam_at(1.0, Direction2::X);
    let high = seam_at(2.0, Direction2::Y);
    assert_eq!(
        ShapeAnalysisCurve
            .select_forward_seam(&first, &high)
            .unwrap(),
        1
    );
    assert_eq!(
        ShapeAnalysisCurve
            .select_forward_seam(&first, &first)
            .unwrap(),
        2
    );
}
#[test]
fn oblique_direction_uses_x_before_y() {
    let first = seam_at(1.0, Direction2::new(-1.0, 1.0).unwrap());
    let high = seam_at(2.0, Direction2::Y);
    assert_eq!(
        ShapeAnalysisCurve
            .select_forward_seam(&first, &high)
            .unwrap(),
        2
    );
}
#[test]
fn unsupported_curve_or_null_returns_zero() {
    let circle: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dCircle::new(Axis22d::default(), 1.0).unwrap()).cast();
    let line = seam_at(0.0, Direction2::X);
    assert_eq!(
        ShapeAnalysisCurve
            .select_forward_seam(&circle, &line)
            .unwrap(),
        0
    );
    assert_eq!(
        ShapeAnalysisCurve
            .select_forward_seam(&line, &circle)
            .unwrap(),
        0
    );
    assert_eq!(
        ShapeAnalysisCurve
            .select_forward_seam(&Handle::default(), &line)
            .unwrap(),
        0
    );
}
