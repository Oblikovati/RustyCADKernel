//! Native triangle/link intersection precedence and degeneracy thresholds, OCCT3d097a0.
use super::isoline_triangle_projection::project_isoline_link;
use super::{StdPrsPntOnIso, StdPrsSegOnIso};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle, SharedState},
};
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkmath::gp::{Point2, Point3};
pub(super) struct IsolineTriangle {
    pub xyz: [Point3; 3],
    pub uv: [Point2; 3],
}
pub(super) fn triangle_isoline_segment(
    surface: &SharedState<Handle<dyn GeomSurface>>,
    is_u: bool,
    parameter: f64,
    triangle: &IsolineTriangle,
) -> Result<Option<StdPrsSegOnIso>, Failure> {
    let mut points = empty_isoline_segment();
    let (mut link, mut count) = (0, 0);
    while link < 3 && count < 2 {
        let (point, advance) = crossed_isoline_link(surface, is_u, parameter, triangle, link)?;
        if let Some(point) = point {
            points[count] = point;
            count += 1;
        }
        link += advance;
    }
    Ok(finish_isoline_segment(points, count))
}
fn finish_isoline_segment(mut points: [StdPrsPntOnIso; 2], count: usize) -> Option<StdPrsSegOnIso> {
    if count != 2 || (points[1].param - points[0].param).abs() <= precision::p_confusion() {
        return None;
    }
    if points[1].param < points[0].param {
        points.swap(0, 1);
    }
    Some(StdPrsSegOnIso { pnts: points })
}
fn crossed_isoline_link(
    surface: &SharedState<Handle<dyn GeomSurface>>,
    is_u: bool,
    parameter: f64,
    triangle: &IsolineTriangle,
    link: usize,
) -> Result<(Option<StdPrsPntOnIso>, usize), Failure> {
    let next = (link + 1) % 3;
    let distances = isoline_link_distances(triangle, link, is_u, parameter);
    if distances[0].abs() < precision::p_confusion() {
        return Ok((Some(isoline_node(triangle, link, is_u)), 1));
    }
    if distances[1].abs() < precision::p_confusion() {
        return Ok((Some(isoline_node(triangle, next, is_u)), 2));
    }
    if distances[0] * distances[1] > 0. {
        return Ok((None, 1));
    }
    project_crossed_isoline_link(surface, is_u, triangle, link, distances)
}
fn project_crossed_isoline_link(
    surface: &SharedState<Handle<dyn GeomSurface>>,
    is_u: bool,
    triangle: &IsolineTriangle,
    link: usize,
    distances: [f64; 2],
) -> Result<(Option<StdPrsPntOnIso>, usize), Failure> {
    let next = (link + 1) % 3;
    if triangle.xyz[link].square_distance(&triangle.xyz[next]) < precision::p_confusion() {
        return Ok((Some(isoline_node(triangle, link, is_u)), 1));
    }
    let alpha = distances[0].abs() / (distances[0].abs() + distances[1].abs());
    Ok((
        Some(project_isoline_link(surface, is_u, triangle, link, alpha)?),
        1,
    ))
}
fn signed_isoline_distance(is_u: bool, parameter: f64, point: Point2) -> f64 {
    // Native isoU points along +V; isoV points along +U, so their signed sides differ.
    if is_u {
        parameter - point.x()
    } else {
        point.y() - parameter
    }
}
fn isoline_node(triangle: &IsolineTriangle, index: usize, is_u: bool) -> StdPrsPntOnIso {
    let param = if is_u {
        triangle.uv[index].y()
    } else {
        triangle.uv[index].x()
    };
    StdPrsPntOnIso {
        pnt: triangle.xyz[index],
        param,
    }
}

fn empty_isoline_segment() -> [StdPrsPntOnIso; 2] {
    // Unused slots remain private and are never emitted as fabricated geometry.
    [StdPrsPntOnIso {
        pnt: Point3::default(),
        param: 0.,
    }; 2]
}
fn isoline_link_distances(
    triangle: &IsolineTriangle,
    link: usize,
    is_u: bool,
    parameter: f64,
) -> [f64; 2] {
    let next = (link + 1) % 3;
    [
        signed_isoline_distance(is_u, parameter, triangle.uv[link]),
        signed_isoline_distance(is_u, parameter, triangle.uv[next]),
    ]
}
