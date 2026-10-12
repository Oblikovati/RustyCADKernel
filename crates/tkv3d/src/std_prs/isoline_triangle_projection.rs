//! Surface arc-length weights fit isoline crossings to triangulation rather than replacing mesh coordinates.
use super::{StdPrsPntOnIso, isoline_triangle_crossing::IsolineTriangle};
use rustycad_tkernel::{
    precision,
    standard::{Failure, Handle, SharedState},
};
use rustycad_tkg3d::{geom::GeomSurface, geom_adaptor::GeomAdaptorCurve};
use rustycad_tkgeombase::gcpnts::AbscissaPoint;
use rustycad_tkmath::gp::Point3;
pub(super) fn project_isoline_link(
    surface: &SharedState<Handle<dyn GeomSurface>>,
    is_u: bool,
    triangle: &IsolineTriangle,
    link: usize,
    alpha: f64,
) -> Result<StdPrsPntOnIso, Failure> {
    let next = (link + 1) % 3;
    let [cross_u, cross_v] = isoline_link_coordinates(triangle, link, alpha);
    let parameter = if is_u { cross_v } else { cross_u };
    let weight = if surface.read().is_null() {
        alpha
    } else {
        surface_link_weight(surface, is_u, triangle, link, [cross_u, cross_v], alpha)?
    };
    Ok(StdPrsPntOnIso {
        pnt: weighted_isoline_point(triangle.xyz[link], triangle.xyz[next], weight),
        param: parameter,
    })
}
fn surface_link_weight(
    surface: &SharedState<Handle<dyn GeomSurface>>,
    is_u: bool,
    triangle: &IsolineTriangle,
    link: usize,
    cross: [f64; 2],
    alpha: f64,
) -> Result<f64, Failure> {
    let next = (link + 1) % 3;
    let first = isoline_projection_curve(surface, is_u, triangle.uv[link])?;
    let second = isoline_projection_curve(surface, is_u, triangle.uv[next])?;
    let curve1 = GeomAdaptorCurve::with_curve(&first)?;
    let curve2 = GeomAdaptorCurve::with_curve(&second)?;
    let endpoints = isoline_arc_endpoints(triangle, link, is_u, cross);
    isoline_arc_weight(&curve1, &curve2, endpoints, alpha)
}
fn isoline_projection_curve(
    surface: &SharedState<Handle<dyn GeomSurface>>,
    is_u: bool,
    uv: rustycad_tkmath::gp::Point2,
) -> Result<Handle<dyn rustycad_tkg3d::geom::GeomCurve>, Failure> {
    let source = surface.snapshot();
    // Each native dereference is fresh: the first callback can replace the surface used by the second.
    if is_u {
        source.v_iso(uv.y())
    } else {
        source.u_iso(uv.x())
    }
}
fn weighted_isoline_point(first: Point3, last: Point3, weight: f64) -> Point3 {
    Point3::new(
        first.x() + (last.x() - first.x()) * weight,
        first.y() + (last.y() - first.y()) * weight,
        first.z() + (last.z() - first.z()) * weight,
    )
}

fn isoline_link_coordinates(triangle: &IsolineTriangle, link: usize, alpha: f64) -> [f64; 2] {
    let next = (link + 1) % 3;
    [
        triangle.uv[link].x() + alpha * (triangle.uv[next].x() - triangle.uv[link].x()),
        triangle.uv[link].y() + alpha * (triangle.uv[next].y() - triangle.uv[link].y()),
    ]
}
fn isoline_arc_endpoints(
    triangle: &IsolineTriangle,
    link: usize,
    is_u: bool,
    cross: [f64; 2],
) -> [f64; 3] {
    let next = (link + 1) % 3;
    if is_u {
        [triangle.uv[link].x(), triangle.uv[next].x(), cross[0]]
    } else {
        [triangle.uv[link].y(), triangle.uv[next].y(), cross[1]]
    }
}
fn isoline_arc_weight(
    first: &GeomAdaptorCurve,
    second: &GeomAdaptorCurve,
    endpoints: [f64; 3],
    alpha: f64,
) -> Result<f64, Failure> {
    let length1 =
        AbscissaPoint::length_between_with_tolerance(first, endpoints[0], endpoints[2], 1e-2)?;
    let length2 =
        AbscissaPoint::length_between_with_tolerance(second, endpoints[1], endpoints[2], 1e-2)?;
    if length1.abs() < precision::confusion() || length2.abs() < precision::confusion() {
        return Ok(alpha);
    }
    Ok(length1 / (length1 + length2))
}
