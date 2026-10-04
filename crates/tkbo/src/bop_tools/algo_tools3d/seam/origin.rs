use super::*;
use crate::int_tools::Tools;
use rustycad_tkernel::precision;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::geom2d_api::ProjectPointOnCurve;
use rustycad_tkmath::gp::Point2;
/// Reconstruct a split seam from the original seam pair; `split_seam_from_origin(&original,&split,&face)?`.
pub fn split_seam_from_origin(original: &Edge, split: &Edge, face: &Face) -> Result<bool, Failure> {
    if !BRepTool::is_closed_on_face(original, face)? {
        return Ok(false);
    }
    if BRepTool::is_closed_on_face(split, face)? {
        return Ok(true);
    }
    let mut split = split.clone();
    split.set_orientation(Orientation::Forward);
    let mut face = face.clone();
    face.set_orientation(Orientation::Forward);
    let curve = BRepTool::curve_on_face(&split, &face)?;
    if curve.pcurve.is_null() {
        return Ok(false);
    }
    let range = (curve.first, curve.last);
    let parameter = Tools::intermediate_point(range.0, range.1);
    let (point, tangent) = curve.pcurve.d1(parameter)?;
    let Some((other, origin_tangent, first)) = project_to_origin(original, &face, point)? else {
        return Ok(false);
    };
    let translation = Vector2::new(other.x() - point.x(), other.y() - point.y());
    install_pair(
        &split,
        &face,
        &curve.pcurve,
        range,
        translation,
        first == (tangent.dot(origin_tangent) > 0.),
    )?;
    Ok(true)
}
fn project_to_origin(
    original: &Edge,
    face: &Face,
    point: Point2,
) -> Result<Option<(Point2, Vector2, bool)>, Failure> {
    let mut forward = original.clone();
    forward.set_orientation(Orientation::Forward);
    let first = BRepTool::curve_on_face(&forward, face)?;
    forward.set_orientation(Orientation::Reversed);
    let last = BRepTool::curve_on_face(&forward, face)?;
    let first_projection =
        ProjectPointOnCurve::with_range(point, &first.pcurve, last.first, last.last)?;
    let last_projection =
        ProjectPointOnCurve::with_range(point, &last.pcurve, last.first, last.last)?;
    let d1 = projection_distance(&first_projection)?;
    let d2 = projection_distance(&last_projection)?;
    if d1 > precision::p_confusion() && d2 > precision::p_confusion() {
        return Ok(None);
    }
    if d1 < d2 {
        return paired_point(
            &last.pcurve,
            &first.pcurve,
            first_projection.lower_distance_parameter()?,
            true,
        )
        .map(Some);
    }
    paired_point(
        &first.pcurve,
        &last.pcurve,
        last_projection.lower_distance_parameter()?,
        false,
    )
    .map(Some)
}
fn projection_distance(projection: &ProjectPointOnCurve) -> Result<f64, Failure> {
    if projection.nb_points() == 0 {
        return Ok(f64::MAX);
    }
    projection.lower_distance()
}
fn paired_point(
    opposite: &Handle<dyn Geom2dCurve>,
    chosen: &Handle<dyn Geom2dCurve>,
    parameter: f64,
    first: bool,
) -> Result<(Point2, Vector2, bool), Failure> {
    Ok((opposite.value(parameter)?, chosen.d1(parameter)?.1, first))
}
