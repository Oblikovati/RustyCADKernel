//! Analytic BeanFace line/plane computation from OCCT 3d097a0.
use crate::int_tools::{Range, Tools};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkmath::{
    elclib, elslib,
    gp::{Line3, Plane3, Point3},
};

pub(super) fn intersect_line_plane(
    line: Line3,
    plane: Plane3,
    range: Range,
    bounds: [f64; 4],
    tolerances: [f64; 2],
) -> Result<Option<Range>, Failure> {
    intersect_with_criteria(
        line,
        plane,
        range,
        bounds,
        tolerances,
        tolerances[0] + tolerances[1],
    )
}

pub(super) fn intersect_with_criteria(
    line: Line3,
    plane: Plane3,
    range: Range,
    bounds: [f64; 4],
    tolerances: [f64; 2],
    criteria: f64,
) -> Result<Option<Range>, Failure> {
    let coefficients = plane.coefficients()?;
    let direction = line.direction();
    let slope = coefficients[0] * direction.x()
        + coefficients[1] * direction.y()
        + coefficients[2] * direction.z();
    let distance = signed_distance(coefficients, line.location());
    let parallel = slope.abs() < 1e-9;
    if contained(line, range, coefficients, criteria, parallel, distance) {
        return Ok(Some(range));
    }
    if parallel {
        return Ok(None);
    }
    intersection_range(line, plane, range, bounds, tolerances, distance, slope)
}
fn intersection_range(
    line: Line3,
    plane: Plane3,
    range: Range,
    bounds: [f64; 4],
    tolerances: [f64; 2],
    distance: f64,
    slope: f64,
) -> Result<Option<Range>, Failure> {
    let Some(parameter) = intersection_parameter(range, distance, slope) else {
        return Ok(None);
    };
    let origin = line.location();
    let intersection = Point3::new(
        origin.x() + parameter * line.direction().x(),
        origin.y() + parameter * line.direction().y(),
        origin.z() + parameter * line.direction().z(),
    );
    if !inside_bounds(plane, intersection, bounds) {
        return Ok(None);
    }
    let angle = intersection_angle(line, plane);
    let delta = Tools::compute_int_range(tolerances[0], tolerances[1], angle);
    Ok(Some(Range::new(
        std_max(range.first(), parameter - delta),
        std_min(range.last(), parameter + delta),
    )))
}
fn intersection_parameter(range: Range, distance: f64, slope: f64) -> Option<f64> {
    let parameter = -distance / slope;
    if parameter < range.first() || parameter > range.last() {
        return None;
    }
    Some(parameter)
}
fn intersection_angle(line: Line3, plane: Plane3) -> f64 {
    let normal = plane.position().direction();
    let angle = line.direction().angle(&normal);
    (std::f64::consts::FRAC_PI_2 - angle).abs()
}
fn signed_distance(coefficients: [f64; 4], point: Point3) -> f64 {
    coefficients[0] * point.x()
        + coefficients[1] * point.y()
        + coefficients[2] * point.z()
        + coefficients[3]
}
fn contained(
    line: Line3,
    range: Range,
    coefficients: [f64; 4],
    criteria: f64,
    parallel: bool,
    distance: f64,
) -> bool {
    if parallel {
        return distance.abs() < criteria;
    }
    let first = signed_distance(coefficients, elclib::value(range.first(), &line));
    let last = signed_distance(coefficients, elclib::value(range.last(), &line));
    first.abs() <= criteria && last.abs() <= criteria
}
fn inside_bounds(plane: Plane3, point: Point3, bounds: [f64; 4]) -> bool {
    let (u, v) = elslib::parameters(&plane, point);
    !(bounds[0] > u || u > bounds[1] || bounds[2] > v || v > bounds[3])
}
