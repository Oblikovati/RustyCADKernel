// ClassifyLin2d and INTER/COINC from IntTools_FaceFace.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Native ClassifyLin2d boundary order, including the duplicate-corner suppression.
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::{
    elclib,
    gp::{Line2, Point2},
};
struct ClipRectangle {
    corners: [Point2; 4],
    line: Line2,
    tolerance: f64,
}
pub(super) fn classify_line(
    bounds: [f64; 4],
    line: Line2,
    tolerance: f64,
) -> Result<Option<(f64, f64)>, Failure> {
    let rectangle = ClipRectangle::new(bounds, line, tolerance);
    let mut parameters = Vec::with_capacity(2);
    for index in 0..4 {
        rectangle.inspect_edge(index, &mut parameters)?;
        if parameters.len() == 2 {
            return Ok(ordered_interval(parameters[0], parameters[1], tolerance));
        }
    }
    Ok(None)
}
impl ClipRectangle {
    fn new([xmin, xmax, ymin, ymax]: [f64; 4], line: Line2, tolerance: f64) -> Self {
        Self {
            corners: [
                Point2::new(xmin, ymin),
                Point2::new(xmin, ymax),
                Point2::new(xmax, ymax),
                Point2::new(xmax, ymin),
            ],
            line,
            tolerance,
        }
    }
    fn inspect_edge(&self, index: usize, parameters: &mut Vec<f64>) -> Result<(), Failure> {
        let endpoints = (self.corners[index], self.corners[(index + 1) % 4]);
        let coefficients = self.line.coefficients();
        let distances = (
            signed_distance(coefficients, endpoints.0),
            signed_distance(coefficients, endpoints.1),
        );
        if index != 0 && !outside(distances.0, self.tolerance) {
            return Ok(());
        }
        intersect_boundary(
            parameters,
            self.line,
            coefficients,
            endpoints,
            distances,
            self.tolerance,
            index.is_multiple_of(2),
        )
    }
}
fn signed_distance((a, b, c): (f64, f64, f64), point: Point2) -> f64 {
    a * point.x() + b * point.y() + c
}
fn outside(value: f64, tolerance: f64) -> bool {
    value > tolerance || value < -tolerance
}
fn crossing((first, last): (f64, f64), tolerance: f64) -> bool {
    (first > tolerance && last < -tolerance)
        || (first < -tolerance && last > tolerance)
        || (coincident(first, tolerance) && outside(last, tolerance))
        || (coincident(last, tolerance) && outside(first, tolerance))
}
fn coincident(value: f64, tolerance: f64) -> bool {
    value <= tolerance && value >= -tolerance
}
fn ordered_interval(first: f64, last: f64, tolerance: f64) -> Option<(f64, f64)> {
    let separated = (first - last).abs() > tolerance;
    if !separated {
        return None;
    }
    Some((
        if last < first { last } else { first },
        if first < last { last } else { first },
    ))
}
fn intersect_boundary(
    parameters: &mut Vec<f64>,
    line: Line2,
    coefficients: (f64, f64, f64),
    endpoints: (Point2, Point2),
    distances: (f64, f64),
    tolerance: f64,
    vertical: bool,
) -> Result<(), Failure> {
    if crossing(distances, tolerance) {
        let point = intersection(coefficients, endpoints.0, vertical);
        parameters.push(elclib::parameter(&line, point)?);
    } else if coincident(distances.0, tolerance) && coincident(distances.1, tolerance) {
        parameters.clear();
        parameters.push(elclib::parameter(&line, endpoints.0)?);
        parameters.push(elclib::parameter(&line, endpoints.1)?);
    }
    Ok(())
}
fn intersection((a, b, c): (f64, f64, f64), first: Point2, vertical: bool) -> Point2 {
    if vertical {
        return Point2::new(first.x(), -(c + a * first.x()) / b);
    }
    Point2::new(-(c + b * first.y()) / a, first.y())
}
