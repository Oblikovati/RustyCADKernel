// Created on: 1995-03-06
// Created by: Laurent PAINNOT
// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of the geometric functions of FoundationClasses/TKMath/Poly/Poly.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The static `Poly` functions that are not stream I/O.
use super::{Triangle, Triangulation};
use crate::gp::{self, Axis1, Direction3, Point2, Xy, Xyz};
use rustycad_tkernel::{
    ncollection::List,
    standard::{Failure, Handle},
};
/// `NCollection_List<occ::handle<Poly_Triangulation>>`; `ListOfTriangulation::default()`.
pub type ListOfTriangulation = List<Handle<Triangulation>>;
/// Joins meshes without sharing nodes and drops UV nodes; a null handle when there is no
/// node at all; `catenate(&list)?`.
pub fn catenate(triangulations: &ListOfTriangulation) -> Result<Handle<Triangulation>, Failure> {
    let live = || triangulations.iter().filter_map(Handle::get);
    let nb_nodes: i32 = live().map(Triangulation::nb_nodes).sum();
    let nb_triangles: i32 = live().map(Triangulation::nb_triangles).sum();
    if nb_nodes == 0 {
        return Ok(Handle::default());
    }
    let result = Triangulation::with_sizes(nb_nodes, nb_triangles, false, false)?;
    let (mut node_offset, mut triangle_offset) = (0, 0);
    for triangulation in live() {
        append_catenated(&result, triangulation, node_offset, triangle_offset)?;
        node_offset += triangulation.nb_nodes();
        triangle_offset += triangulation.nb_triangles();
    }
    Ok(Handle::new(result))
}
fn append_catenated(
    result: &Triangulation,
    source: &Triangulation,
    node_offset: i32,
    triangle_offset: i32,
) -> Result<(), Failure> {
    for index in 1..=source.nb_nodes() {
        result.set_node(index + node_offset, source.node(index)?)?;
    }
    for index in 1..=source.nb_triangles() {
        let (a, b, c) = source.triangle(index)?.get();
        let shifted = Triangle::new(a + node_offset, b + node_offset, c + node_offset);
        result.set_triangle(index + triangle_offset, shifted)?;
    }
    Ok(())
}
/// Computes smooth node normals, as `Poly::ComputeNormals`; `compute_normals(&mesh)?`.
pub fn compute_normals(triangulation: &Triangulation) -> Result<(), Failure> {
    triangulation.compute_normals()
}
// std::max(0., x) and std::min(1., x): the first argument wins ties and NaN.
fn clamp_unit(value: f64) -> f64 {
    let lower = if 0.0 < value { value } else { 0.0 };
    if lower < 1.0 { lower } else { 1.0 }
}
/// Parameters `(u, v)` of `point` on triangle `p1 p2 p3` with the squared distance to it
/// (zero inside); outside or degenerate triangles give the closest point;
/// `point_on_triangle(p1, p2, p3, p)` returns `(distance2, uv)`.
pub fn point_on_triangle(p1: Xy, p2: Xy, p3: Xy, point: Xy) -> (f64, Xy) {
    let dp = point - p1;
    let du = p2 - p1;
    let dv = p3 - p1;
    let det = du ^ dv;
    if det.abs() > gp::resolution() {
        return regular_triangle_parameters(p2, point, dp, du, dv, det);
    }
    degenerate_triangle_parameters(p1, point, dp, du, dv)
}
fn regular_triangle_parameters(p2: Xy, point: Xy, dp: Xy, du: Xy, dv: Xy, det: f64) -> (f64, Xy) {
    let resolution = gp::resolution();
    let u = (dp ^ dv) / det;
    let v = -(dp ^ du) / det;
    if u > -resolution && v > -resolution && 1. - u - v > -resolution {
        return (0.0, Xy::new(u, v));
    }
    // Distorted triangles need every side checked, whatever the signs of u and v.
    let mut best = (0.0, clamp_unit((dp * dv) / dv.square_modulus()));
    let mut distance = (best.1 * dv - dp).square_modulus();
    let side_u = clamp_unit((dp * du) / du.square_modulus());
    let candidate = (side_u * du - dp).square_modulus();
    if candidate < distance {
        (best, distance) = ((side_u, 0.0), candidate);
    }
    let duv = dv - du;
    let side_uv = clamp_unit(((dp - du) * duv) / duv.square_modulus());
    let candidate = (p2 + side_uv * duv - point).square_modulus();
    if candidate < distance {
        (best, distance) = ((1. - side_uv, side_uv), candidate);
    }
    (distance, Xy::new(best.0, best.1))
}
fn degenerate_triangle_parameters(p1: Xy, point: Xy, dp: Xy, du: Xy, dv: Xy) -> (f64, Xy) {
    let resolution = gp::resolution();
    let l2u = du.square_modulus();
    let l2v = dv.square_modulus();
    if l2u < resolution && l2v < resolution {
        return ((point - p1).square_modulus(), Xy::new(0., 0.));
    }
    if l2u < resolution {
        let v = (dp * dv) / l2v;
        return ((point - (p1 + v * dv)).square_modulus(), Xy::new(0., v));
    }
    if l2v < resolution {
        let u = (dp * du) / l2u;
        return ((point - (p1 + u * du)).square_modulus(), Xy::new(u, 0.));
    }
    // Collinear sides: keep the side whose clamped foot is closer, but report the
    // unclamped parameter, as the native code does.
    let d1 = (dp - clamp_unit((dp * du) / l2u) * du).square_modulus();
    let d2 = (dp - clamp_unit((dp * dv) / l2v) * dv).square_modulus();
    if d1 < d2 {
        (d1, Xy::new((dp * du) / l2u, 0.))
    } else {
        (d2, Xy::new(0., (dp * dv) / l2v))
    }
}
/// Closest (or farthest) triangle hit by a ray with parameter above 1e-15, as
/// `Poly::Intersect`. As natively, `triangle` follows every improvement while `distance`
/// is written only on success; `intersect(&mesh, axis, true, &mut hit, &mut distance)?`.
pub fn intersect(
    triangulation: &Triangulation,
    axis: Axis1,
    closest: bool,
    triangle: &mut Triangle,
    distance: &mut f64,
) -> Result<bool, Failure> {
    const CONFUSION: f64 = 1E-15;
    let mut best = if closest { f64::MAX } else { 0.0 };
    for index in 1..=triangulation.nb_triangles() {
        let candidate = triangulation.triangle(index)?;
        let Some(parameter) = ray_parameter(triangulation, axis, candidate)? else {
            continue;
        };
        let better = if closest {
            parameter < best
        } else {
            parameter > best
        };
        if parameter > CONFUSION && better {
            best = parameter;
            *triangle = candidate;
        }
    }
    if CONFUSION < best && best < f64::MAX {
        *distance = best;
        return Ok(true);
    }
    Ok(false)
}
fn ray_parameter(
    triangulation: &Triangulation,
    axis: Axis1,
    triangle: Triangle,
) -> Result<Option<f64>, Failure> {
    let (a, b, c) = triangle.get();
    let v0 = triangulation.node(a)?.xyz();
    let v1 = triangulation.node(b)?.xyz();
    let v2 = triangulation.node(c)?.xyz();
    Ok(intersect_tri_line(
        axis.location().xyz(),
        axis.direction(),
        v0,
        v1,
        v2,
    ))
}
// The barycentric tests of the native branches, with `sign` +1 for a positive
// determinant and -1 for a negative one; the second minor is computed only when needed.
fn within(m: &[[f64; 4]; 3], d: f64, sign: f64) -> bool {
    const CONFUSION: f64 = 1E-15;
    let da = determinant(m, 0, 3, 2);
    sign * da > -CONFUSION && {
        let db = determinant(m, 0, 1, 3);
        sign * db > -CONFUSION && sign * (da + db) <= sign * d + CONFUSION
    }
}
// Minor of the 3x4 matrix over columns c1, c2, c3, in the native term order.
fn determinant(a: &[[f64; 4]; 3], c1: usize, c2: usize, c3: usize) -> f64 {
    a[0][c1] * a[1][c2] * a[2][c3] + a[0][c2] * a[1][c3] * a[2][c1] + a[0][c3] * a[1][c1] * a[2][c2]
        - a[0][c3] * a[1][c2] * a[2][c1]
        - a[0][c2] * a[1][c1] * a[2][c3]
        - a[0][c1] * a[1][c3] * a[2][c2]
}
/// Line parameter of the intersection of a line and a triangle, by Cramer's rule with a
/// 1e-15 margin; `intersect_tri_line(start, dir, v0, v1, v2)` is `IntersectTriLine`.
pub fn intersect_tri_line(
    start: Xyz,
    direction: Direction3,
    v0: Xyz,
    v1: Xyz,
    v2: Xyz,
) -> Option<f64> {
    const CONFUSION: f64 = 1E-15;
    let row = |d: f64, a: f64, b: f64, c: f64, s: f64| [-d, a - s, b - s, c - s];
    let m = [
        row(direction.x(), v1.x(), v2.x(), start.x(), v0.x()),
        row(direction.y(), v1.y(), v2.y(), start.y(), v0.y()),
        row(direction.z(), v1.z(), v2.z(), start.z(), v0.z()),
    ];
    let d = determinant(&m, 0, 1, 2);
    let dt = determinant(&m, 3, 1, 2);
    let inside = if d > CONFUSION {
        within(&m, d, 1.0)
    } else if d < -CONFUSION {
        within(&m, d, -1.0)
    } else {
        false
    };
    inside.then(|| dt / d)
}
/// Signed area (positive counter-clockwise) and perimeter of a closed polygon given by its
/// vertices; fewer than two points give zeros; `polygon_properties(&points)` is `(area, perimeter)`.
pub fn polygon_properties(points: &[Point2]) -> (f64, f64) {
    if points.len() < 2 {
        return (0.0, 0.0);
    }
    let reference = points[0].xy();
    let mut previous = points[1].xy() - reference;
    let mut area = 0.0;
    let mut perimeter = previous.modulus();
    for point in &points[2..] {
        let current = point.xy() - reference;
        area += previous ^ current;
        perimeter += (previous - current).modulus();
        previous = current;
    }
    perimeter += previous.modulus();
    (area * 0.5, perimeter)
}
