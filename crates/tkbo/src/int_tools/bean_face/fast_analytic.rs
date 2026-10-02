//! Private fast classification for BeanFace analytic curve/surface pairs.
use crate::int_tools::Range;
use rustycad_tkmath::gp::{Axis1, Line3, Point3, Vector3};

#[derive(Clone, Copy)]
pub(super) struct AnalyticCurve {
    pub kind: i32,
    pub axis: Axis1,
    pub radius: f64,
}

#[derive(Clone, Copy)]
pub(super) struct AnalyticSurface {
    pub kind: i32,
    pub axis: Axis1,
    pub radius: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct FastAnalyticResult {
    pub successful: bool,
    pub coincident_range: Option<Range>,
}

const ANGULAR_TOLERANCE: f64 = 1e-12;

pub(super) fn classify(
    curve: AnalyticCurve,
    surface: AnalyticSurface,
    first: f64,
    last: f64,
    criteria: f64,
) -> FastAnalyticResult {
    if unsupported_curve(curve.kind) {
        return result(false, None);
    }
    match surface.kind {
        0 => plane_pair(curve, surface, first, last, criteria),
        1 => cylinder_pair(curve, surface, first, last, criteria),
        2 => sphere_pair(curve, surface, criteria),
        _ => result(false, None),
    }
}

fn unsupported_curve(kind: i32) -> bool {
    matches!(kind, 5..=8)
}

fn plane_pair(
    curve: AnalyticCurve,
    surface: AnalyticSurface,
    first: f64,
    last: f64,
    criteria: f64,
) -> FastAnalyticResult {
    if !matches!(curve.kind, 1..=4) {
        return result(false, None);
    }
    if curve.axis.direction().angle(&surface.axis.direction()) > ANGULAR_TOLERANCE {
        return result(false, None);
    }
    let distance = point_plane_distance(curve.axis.location(), surface.axis);
    coincident_or_no_intersection(distance < criteria, true, first, last)
}

fn cylinder_pair(
    curve: AnalyticCurve,
    surface: AnalyticSurface,
    first: f64,
    last: f64,
    criteria: f64,
) -> FastAnalyticResult {
    match curve.kind {
        0 => cylinder_line(curve, surface, first, last, criteria),
        1 => cylinder_circle(curve, surface, first, last, criteria),
        _ => result(false, None),
    }
}

fn cylinder_line(
    curve: AnalyticCurve,
    surface: AnalyticSurface,
    first: f64,
    last: f64,
    criteria: f64,
) -> FastAnalyticResult {
    let line = Line3::from_axis(curve.axis);
    if !line
        .direction()
        .is_parallel(surface.axis.direction(), ANGULAR_TOLERANCE)
    {
        return result(false, None);
    }
    let distance = (line.distance(surface.axis.location()) - surface.radius).abs();
    coincident_or_no_intersection(distance < criteria, true, first, last)
}

fn cylinder_circle(
    curve: AnalyticCurve,
    surface: AnalyticSurface,
    first: f64,
    last: f64,
    criteria: f64,
) -> FastAnalyticResult {
    if curve.axis.direction().angle(&surface.axis.direction()) > ANGULAR_TOLERANCE {
        return result(false, None);
    }
    let axis = Line3::from_axis(surface.axis);
    let distance = axis.distance(curve.axis.location());
    let radii_difference = (curve.radius - surface.radius).abs();
    let is_coincident = distance + radii_difference < criteria;
    let has_intersection = if is_coincident {
        false
    } else {
        distance - (curve.radius + surface.radius) < criteria
            && radii_difference - distance < criteria
    };
    coincident_or_no_intersection(is_coincident, !has_intersection, first, last)
}

fn sphere_pair(
    curve: AnalyticCurve,
    surface: AnalyticSurface,
    criteria: f64,
) -> FastAnalyticResult {
    if curve.kind != 0 {
        return result(false, None);
    }
    let distance = Line3::from_axis(curve.axis).distance(surface.axis.location()) - surface.radius;
    result(
        distance.partial_cmp(&criteria) != Some(std::cmp::Ordering::Less),
        None,
    )
}

fn point_plane_distance(point: Point3, plane: Axis1) -> f64 {
    Vector3::between(plane.location(), point)
        .dot(Vector3::from(plane.direction()))
        .abs()
}

fn coincident_or_no_intersection(
    coincident: bool,
    no_intersection: bool,
    first: f64,
    last: f64,
) -> FastAnalyticResult {
    let range = coincident.then(|| Range::new(first, last));
    result(coincident || no_intersection, range)
}

fn result(successful: bool, coincident_range: Option<Range>) -> FastAnalyticResult {
    FastAnalyticResult {
        successful,
        coincident_range,
    }
}
