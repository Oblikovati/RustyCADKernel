// Copyright (c) 1993-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.
// Rust translation of SectionPointToParameters, CollectInterferencePoints and
// ProcessSortedPoints (IntCurveSurface_InterUtils.pxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The start points of the exact refinement, read from the section points of the
//! polygon-polyhedron interference, and their refinement.
use super::SortedStartPoints;
use super::analytic::compute_append_point;
use crate::int_curve_surface::IntersectionPoint;
use crate::int_curve_surface::tools::{InterCurveTool, InterSurfaceTool};
use crate::int_imp::{IntImpIntCS, IntImpZerCSParFunc};
use crate::intf::{Interference, PIType, SectionPoint};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkmath::gp::{Point3, Vector3};
use rustycad_tkmath::math::FunctionSetRoot;

/// The polyhedron calls of `SectionPointToParameters` (the `PolyhedronType`).
pub trait SectionPolyhedron {
    /// `Triangle(Index, P1, P2, P3)`.
    fn triangle(&self, index: i32) -> Result<[i32; 3], Failure>;
    /// `Point(Index)`.
    fn point(&self, index: i32) -> Result<Point3, Failure>;
    /// `Parameters(Index, U, V)`.
    fn parameters(&self, index: i32) -> Result<(f64, f64), Failure>;
}

/// The polygon call of `SectionPointToParameters` (the `PolygonType`).
pub trait SectionPolygon {
    /// `ApproxParamOnCurve(Index, ParamOnLine)`.
    fn approx_param_on_curve(&self, index: i32, param_on_line: f64) -> Result<f64, Failure>;
}

/// `SectionPointToParameters`: approximate (u, v, w) of a section point. On the
/// polyhedron: the vertex parameters, the edge parameters interpolated, or the
/// barycentric parameters in the facet (the closest point of the longest edge for a
/// degenerate facet); (0, 0) for an external point. On the polygon: the curve
/// parameter of its segment and position;
/// `let (u, v, w) = section_point_to_parameters(&sp, &polyhedron, &polygon)?;`.
pub fn section_point_to_parameters(
    sp: &SectionPoint,
    polyhedron: &impl SectionPolyhedron,
    polygon: &impl SectionPolygon,
) -> Result<(f64, f64, f64), Failure> {
    let (typ, adr1, adr2, param) = sp.info_second();
    let (u1, v1): (f64, f64) = match typ {
        PIType::Vertex => polyhedron.parameters(adr1)?,
        PIType::Edge => {
            let (mut u1, mut v1) = polyhedron.parameters(adr1)?;
            let (u, v) = polyhedron.parameters(adr2)?;
            u1 += param * (u - u1);
            v1 += param * (v - v1);
            (u1, v1)
        }
        PIType::Face => face_parameters(sp.pnt(), polyhedron, adr1)?,
        PIType::External => (0.0, 0.0),
    };
    let (_, seg_index, param) = sp.info_first_address();
    let w: f64 = polygon.approx_param_on_curve(seg_index, param)?;
    Ok((u1, v1, w))
}

/// The barycentric parameters of `p` in facet `adr1`.
fn face_parameters(
    p: Point3,
    polyhedron: &impl SectionPolyhedron,
    adr1: i32,
) -> Result<(f64, f64), Failure> {
    let [pt1, pt2, pt3] = polyhedron.triangle(adr1)?;
    let (pa, pb, pc) = (
        polyhedron.point(pt1)?,
        polyhedron.point(pt2)?,
        polyhedron.point(pt3)?,
    );
    let uv = [
        polyhedron.parameters(pt1)?,
        polyhedron.parameters(pt2)?,
        polyhedron.parameters(pt3)?,
    ];
    let normale: Vector3 = Vector3::between(pa, pb).crossed(Vector3::between(pa, pc));
    let mut cc: f64 = Vector3::between(pa, pb)
        .crossed(Vector3::between(pa, p))
        .dot(normale);
    let mut ca: f64 = Vector3::between(pb, pc)
        .crossed(Vector3::between(pb, p))
        .dot(normale);
    let mut cb: f64 = Vector3::between(pc, pa)
        .crossed(Vector3::between(pc, p))
        .dot(normale);
    let cabc: f64 = ca + cb + cc;
    if cabc.abs() > precision::computational() {
        ca /= cabc;
        cb /= cabc;
        cc /= cabc;
        let [(ua, va), (ub, vb), (uc, vc)] = uv;
        return Ok((ca * ua + cb * ub + cc * uc, ca * va + cb * vb + cc * vc));
    }
    Ok(longest_edge_parameters(p, [pa, pb, pc], uv))
}

/// The parameters of the projection of `p` on the longest edge of a degenerate
/// facet (clamped to the edge), or of its first vertex when every edge is null.
fn longest_edge_parameters(
    p: Point3,
    [pa, pb, pc]: [Point3; 3],
    uv: [(f64, f64); 3],
) -> (f64, f64) {
    let (ab, bc, ca) = (
        Vector3::between(pa, pb),
        Vector3::between(pb, pc),
        Vector3::between(pc, pa),
    );
    let (sq_ab, sq_bc, sq_ca) = (
        ab.square_magnitude(),
        bc.square_magnitude(),
        ca.square_magnitude(),
    );
    let sq_conf: f64 = precision::square_confusion();
    let [(ua, va), (ub, vb), (uc, vc)] = uv;
    let along =
        |from: Point3, edge: Vector3, sq: f64, (u0, v0): (f64, f64), (u1, v1): (f64, f64)| {
            let t: f64 = std_min(1.0, std_max(0.0, Vector3::between(from, p).dot(edge) / sq));
            (u0 + t * (u1 - u0), v0 + t * (v1 - v0))
        };
    if sq_ab >= sq_bc && sq_ab >= sq_ca && sq_ab > sq_conf {
        along(pa, ab, sq_ab, (ua, va), (ub, vb))
    } else if sq_bc >= sq_ca && sq_bc > sq_conf {
        along(pb, bc, sq_bc, (ub, vb), (uc, vc))
    } else if sq_ca > sq_conf {
        along(pc, ca, sq_ca, (uc, vc), (ua, va))
    } else {
        (ua, va)
    }
}

/// `CollectInterferencePoints`: the parameters of every section point, then of
/// every point of every tangent zone, in order; `collect_interference_points(&i, &ph, &pg)?`.
pub fn collect_interference_points(
    interference: &Interference,
    polyhedron: &impl SectionPolyhedron,
    polygon: &impl SectionPolygon,
) -> Result<SortedStartPoints, Failure> {
    let mut points = SortedStartPoints::default();
    for i in 1..=interference.nb_section_points() {
        let (u, v, w) =
            section_point_to_parameters(interference.pnt_value(i)?, polyhedron, polygon)?;
        points.append(u, v, w);
    }
    for i in 1..=interference.nb_tangent_zones() {
        let zone = interference.zone_value(i)?;
        for j in 1..=zone.number_of_points() {
            let (u, v, w) = section_point_to_parameters(zone.get_point(j)?, polyhedron, polygon)?;
            points.append(u, v, w);
        }
    }
    Ok(points)
}

/// The exact intersector of the IntCurveSurface instances.
type ExactInter<S, C> = IntImpIntCS<IntImpZerCSParFunc<S, C>>;

/// `ProcessSortedPoints`: refines each start point that differs from the previous
/// one by more than `10 * PConfusion` (the first always) within the patch
/// `[u0, u1] x [v0, v1]` and the curve range `[winf, wsup]`, and keeps the refined
/// points that [`compute_append_point`] accepts;
/// `process_sorted_points(&mut exact, &mut rsnld, &pts, [u0, u1, v0, v1, winf, wsup], &c, &s)?`.
pub fn process_sorted_points<C: InterCurveTool, S: InterSurfaceTool>(
    exact: &mut ExactInter<S, C>,
    rsnld: &mut FunctionSetRoot,
    points: &SortedStartPoints,
    bounds: [f64; 6],
    curve: &C::Curve,
    surface: &S::Surface,
) -> Result<Vec<IntersectionPoint>, Failure> {
    let ptol: f64 = 10.0 * precision::p_confusion();
    let (mut su, mut sv, mut sw) = (0.0, 0.0, 0.0);
    let mut result: Vec<IntersectionPoint> = Vec::new();
    for i in 0..points.tab_u.len() {
        let (u, v, w) = (points.tab_u[i], points.tab_v[i], points.tab_w[i]);
        if i == 0 {
            su = u - 1.0;
        }
        if (u - su).abs() > ptol || (v - sv).abs() > ptol || (w - sw).abs() > ptol {
            exact.perform([u, v, w], rsnld, bounds)?;
            if exact.is_done() && !exact.is_empty()? {
                let w: f64 = exact.parameter_on_curve()?;
                let (u, v) = exact.parameter_on_surface()?;
                result.extend(compute_append_point::<C, S>(curve, w, surface, u, v)?);
            }
        }
        (su, sv, sw) = (u, v, w);
    }
    Ok(result)
}
