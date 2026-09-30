//! Native forward seam selection, including X-before-Y direction precedence.
use super::ShapeAnalysisCurve;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBoundedCurve, Geom2dCurve, Geom2dLine};
use rustycad_tkmath::gp::{Direction2, Point2, Vector2, resolution};

fn seam_line(curve: &Handle<dyn Geom2dCurve>) -> Result<Option<(Point2, Direction2)>, Failure> {
    if let Some(line) = curve.cast::<Geom2dLine>().get() {
        return Ok(Some((line.location(), line.direction())));
    }
    let bounded = curve.cast::<dyn Geom2dBoundedCurve>();
    let Some(bounded) = bounded.get() else {
        return Ok(None);
    };
    let start = bounded.start_point()?;
    let end = bounded.end_point()?;
    let direction = Vector2::between(start, end);
    if direction.square_magnitude() < resolution() {
        return Ok(None);
    }
    Ok(Some((start, Direction2::from_vector(direction)?)))
}
fn forward_index(first: Point2, second: Point2, direction: Direction2) -> i32 {
    if direction.x() > 0.0 {
        return if first.y() < second.y() { 1 } else { 2 };
    }
    if direction.x() < 0.0 {
        return if first.y() < second.y() { 2 } else { 1 };
    }
    if direction.y() > 0.0 {
        return if first.x() > second.x() { 1 } else { 2 };
    }
    if direction.y() < 0.0 {
        return if first.x() > second.x() { 2 } else { 1 };
    }
    0
}
impl ShapeAnalysisCurve {
    /// Choose native forward seam index 0/1/2; `ShapeAnalysisCurve.select_forward_seam(&first,&second)?`.
    pub fn select_forward_seam(
        &self,
        first: &Handle<dyn Geom2dCurve>,
        second: &Handle<dyn Geom2dCurve>,
    ) -> Result<i32, Failure> {
        let Some((first, direction)) = seam_line(first)? else {
            return Ok(0);
        };
        let Some((second, _)) = seam_line(second)? else {
            return Ok(0);
        };
        Ok(forward_index(first, second, direction))
    }
}
