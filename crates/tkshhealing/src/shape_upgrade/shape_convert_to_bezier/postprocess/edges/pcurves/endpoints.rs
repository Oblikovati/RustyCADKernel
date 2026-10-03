use rustycad_tkbrep::topods::Edge;
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle},
};
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCurve};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::Point2;

pub(super) fn join_bezier_ends(
    edge: &Edge,
    next: &Edge,
    curve: &Handle<Geom2dBezierCurve>,
    next_curve: &Handle<Geom2dBezierCurve>,
) -> Result<(), Failure> {
    let (first, last) = endpoint_parameters(curve);
    let (next_first, next_last) = endpoint_parameters(next_curve);
    let start = curve.value(first)?;
    let end = curve.value(last)?;
    let next_start = next_curve.value(next_first)?;
    let next_end = next_curve.value(next_last)?;
    join_for_orientations(
        edge,
        next,
        curve,
        next_curve,
        [start, end, next_start, next_end],
    )
}

fn endpoint_parameters(curve: &Geom2dBezierCurve) -> (f64, f64) {
    (curve.first_parameter(), curve.last_parameter())
}

fn join_for_orientations(
    edge: &Edge,
    next: &Edge,
    curve: &Geom2dBezierCurve,
    next_curve: &Geom2dBezierCurve,
    points: [Point2; 4],
) -> Result<(), Failure> {
    let [start, end, next_start, next_end] = points;
    let (current_index, next_index, left, right) = match (edge.orientation(), next.orientation()) {
        (Orientation::Forward, Orientation::Forward) => (curve.nb_poles(), 1, end, next_start),
        (Orientation::Forward, Orientation::Reversed) => {
            (curve.nb_poles(), next_curve.nb_poles(), end, next_end)
        }
        (Orientation::Reversed, Orientation::Forward) => (1, 1, start, next_start),
        (Orientation::Reversed, Orientation::Reversed) => {
            (1, next_curve.nb_poles(), start, next_end)
        }
        _ => return Ok(()),
    };
    join_pair(curve, current_index, next_curve, next_index, left, right)
}

fn join_pair(
    current: &Geom2dBezierCurve,
    current_index: i32,
    next: &Geom2dBezierCurve,
    next_index: i32,
    left: Point2,
    right: Point2,
) -> Result<(), Failure> {
    if left.distance(right) <= precision::p_confusion() {
        return Ok(());
    }
    let middle = Point2::new((left.x() + right.x()) / 2., (left.y() + right.y()) / 2.);
    current.set_pole(current_index, middle)?;
    next.set_pole(next_index, middle)
}
