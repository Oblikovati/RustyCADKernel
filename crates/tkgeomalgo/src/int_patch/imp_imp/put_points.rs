// Copyright (c) 1992-1999 Matra Datavision
// Copyright (c) 1999-2024 OPEN CASCADE SAS
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
// Rust translation of PutPointsOnLine of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpImpIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The points found on the restrictions of one domain become vertices of the lines
//! that pass through them, with their transitions.
use super::super::line::IntPatchLine;
use super::super::path_point::IntPatchThePathPointOfTheSOnBounds;
use super::super::point::IntPatchPoint;
use super::arc::recadre;
use super::find_line::{FindLineArgs, find_line};
use super::lines::VertexLine;
use super::multiple::{Marks, multiple_point, point_on_second_dom};
use super::transitions::line_arc_transitions;
use crate::int_start::{TheArc, TheTopolTool, TheVertex};
use crate::int_surf::{IntSurfQuadric, IntSurfTransition};
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3, resolution};

/// The fixed inputs of `PutPointsOnLine`.
pub(super) struct PutPointsInput<'a> {
    pub(super) s1: &'a Adaptor3dSurfaceHandle,
    pub(super) s2: &'a Adaptor3dSurfaceHandle,
    pub(super) listpnt: &'a [IntPatchThePathPointOfTheSOnBounds],
    pub(super) on_first: bool,
    pub(super) domain: &'a (dyn TheTopolTool + 'static),
    pub(super) quad_surf: &'a IntSurfQuadric,
    pub(super) other_quad: &'a IntSurfQuadric,
    pub(super) multpoint: bool,
    pub(super) tolarc: f64,
}

/// The locals of `PutPointsOnLine` that keep their values from one point (or line) to
/// the next, as natively: the vertex `solpnt` and the underlying vertex `vtx` above all.
#[derive(Default)]
struct PutState {
    solpnt: IntPatchPoint,
    vtx: TheVertex,
    normale: Vector3,
    vtgint: Vector3,
    vtgrst: Vector3,
    d1u: Vector3,
    d1v: Vector3,
    p2d: Point2,
    d2d: Vector2,
    transline: IntSurfTransition,
    transarc: IntSurfTransition,
    goon: bool,
}

/// The tangent of the restriction at the point of the arc: the arc derivative through
/// the quadric derivatives (kept in the state for the vertices that follow).
fn arc_tangent(st: &mut PutState, quad: &IntSurfQuadric, arc: &TheArc, prm: f64) -> Result<(), Failure> {
    (st.p2d, st.d2d) = arc.d1(prm)?;
    if let Some((_, d1u, d1v)) = quad.d1(st.p2d.x(), st.p2d.y()) {
        (st.d1u, st.d1v) = (d1u, d1v);
    }
    st.vtgrst.set_linear_form(st.d2d.x(), st.d1u, st.d2d.y(), st.d1v);
    Ok(())
}

/// `PutPointsOnLine(S1, S2, listpnt, slin, OnFirst, Domain, QuadSurf, OtherQuad,
/// multpoint, Tolarc)`: each point of `listpnt` not done yet is matched first with the
/// multiple points of the lines, then with every line not used by it (`FindLine`); the
/// matched line gets the point as a vertex, and the points of the same underlying
/// vertex after it too.
pub(super) fn put_points_on_line(
    input: &PutPointsInput<'_>,
    slin: &[Handle<dyn IntPatchLine>],
) -> Result<(), Failure> {
    let nbpnt: usize = input.listpnt.len();
    if slin.is_empty() || nbpnt == 0 {
        return Ok(());
    }
    let mut st = PutState::default();
    let mut marks = Marks { done: vec![0; nbpnt], used_line: vec![0; slin.len()] };
    for i in 0..nbpnt {
        if marks.done[i] == 1 {
            continue;
        }
        marks.used_line.fill(0);
        st.goon = true;
        if input.multpoint {
            st.goon = multiple_start(input, slin, &mut st, &mut marks, i)?;
        }
        if st.goon {
            for indiceline in 0..slin.len() {
                if marks.used_line[indiceline] == 0 {
                    try_line(input, slin, &mut st, &mut marks.done, (i, indiceline))?;
                }
            }
        }
    }
    Ok(())
}

/// The multiple-point step: the tangent at the point, then `MultiplePoint`.
fn multiple_start(
    input: &PutPointsInput<'_>,
    slin: &[Handle<dyn IntPatchLine>],
    st: &mut PutState,
    marks: &mut Marks,
    i: usize,
) -> Result<bool, Failure> {
    let current: &IntPatchThePathPointOfTheSOnBounds = &input.listpnt[i];
    st.normale = input.quad_surf.normale_at(current.value())?;
    arc_tangent(st, input.quad_surf, &current.arc(), current.parameter())?;
    let args = (input.listpnt, input.domain, input.quad_surf, st.normale);
    multiple_point(args, slin, marks, i, input.on_first, input.tolarc)
}

/// The tolerance of a point on a vertex: at least the vertex resolution, and its square
/// root where the two quadrics are tangent (OCC4455).
fn vertex_tolerance(
    input: &PutPointsInput<'_>,
    current: &IntPatchThePathPointOfTheSOnBounds,
    mut tolerance: f64,
) -> Result<f64, Failure> {
    let vtx: TheVertex = current.vertex()?;
    let vtx_tol: f64 = vtx.resolution(&current.arc());
    let tol_ang: f64 = 0.01 * tolerance;
    tolerance = std_max(tolerance, vtx_tol);
    let norm1: Vector3 = input.quad_surf.normale_at(current.value())?;
    let norm2: Vector3 = input.other_quad.normale_at(current.value())?;
    if norm1.magnitude() > resolution()
        && norm2.magnitude() > resolution()
        && norm1.is_parallel(norm2, tol_ang)?
    {
        tolerance = tolerance.sqrt();
    }
    Ok(tolerance)
}

/// One line for the point `i` (the loop body over `indiceline`).
fn try_line(
    input: &PutPointsInput<'_>,
    slin: &[Handle<dyn IntPatchLine>],
    st: &mut PutState,
    done: &mut [i32],
    (i, indiceline): (usize, usize),
) -> Result<(), Failure> {
    // The points can be moved: take the original point again.
    let current: &IntPatchThePathPointOfTheSOnBounds = &input.listpnt[i];
    let mut currentparameter: f64 = current.parameter();
    let mut psurf: Point3 = current.value();
    let mut tolerance: f64 = current.tolerance();
    if !current.is_new() {
        tolerance = vertex_tolerance(input, current, tolerance)?;
    }
    st.vtgint = Vector3::new(0.0, 0.0, 0.0);
    let mut args = FindLineArgs::new(input.tolarc, indiceline as i32 + 1);
    let found = find_line(
        &mut psurf,
        slin,
        tolerance,
        &mut args,
        (&current.arc(), &mut currentparameter),
        (input.quad_surf, input.other_quad),
    )?;
    let Some(linenumber) = found else {
        // To test whether the point is new or not, and act accordingly.
        done[i] = 1;
        return Ok(());
    };
    st.vtgint = args.vtgint;
    let found_line = FoundLine { psurf, currentparameter, linenumber, args };
    add_found_point(input, slin, st, done, i, found_line)
}

/// The result of `FindLine` for one point.
pub(super) struct FoundLine {
    pub(super) psurf: Point3,
    pub(super) currentparameter: f64,
    pub(super) linenumber: i32,
    pub(super) args: FindLineArgs,
}

/// The point becomes a vertex of the line found, at each of its parameters on it.
fn add_found_point(
    input: &PutPointsInput<'_>,
    slin: &[Handle<dyn IntPatchLine>],
    st: &mut PutState,
    done: &mut [i32],
    i: usize,
    found: FoundLine,
) -> Result<(), Failure> {
    let current: &IntPatchThePathPointOfTheSOnBounds = &input.listpnt[i];
    st.normale = input.quad_surf.normale_at(found.psurf)?;
    let currentarc: TheArc = current.arc();
    arc_tangent(st, input.quad_surf, &currentarc, found.currentparameter)?;
    let lin: &Handle<dyn IntPatchLine> = &slin[found.linenumber as usize - 1];
    let vline = VertexLine::analytic_or_conic(lin)?;
    if !input.on_first {
        // The correspondence between the points on the domains of the two surfaces.
        let args = (input.listpnt, input.domain, input.quad_surf, st.normale, st.vtgint);
        st.goon = point_on_second_dom(args, &vline, done, i, found.args.vert_tol)?;
    }
    if !st.goon {
        return Ok(());
    }
    set_solution(input, st, &found, (i, &currentarc))?;
    for &param in &found.args.lparams {
        st.solpnt.set_parameter(param);
        vline.add_vertex(&st.solpnt)?;
    }
    done[i] = 1;
    if st.goon {
        same_vertex_points(input, st, &vline, done, i)?;
    }
    Ok(())
}

/// The vertex `solpnt` of the point: the input tolerance (not the one of FindLine,
/// 4 April 97), the reframed parameters, the underlying vertex and the arc.
fn set_solution(
    input: &PutPointsInput<'_>,
    st: &mut PutState,
    found: &FoundLine,
    (i, currentarc): (usize, &TheArc),
) -> Result<(), Failure> {
    let current: &IntPatchThePathPointOfTheSOnBounds = &input.listpnt[i];
    st.solpnt.set_value_tangent(found.psurf, found.args.vert_tol, false);
    let (u1, v1) = (st.p2d.x(), st.p2d.y());
    let (u2, v2) = input.other_quad.parameters(found.psurf).unwrap_or((0.0, 0.0));
    if input.on_first {
        let [a, b, c, d] = recadre(input.s1, input.s2, [u1, v1, u2, v2])?;
        st.solpnt.set_parameters(a, b, c, d);
    } else {
        let [a, b, c, d] = recadre(input.s1, input.s2, [u2, v2, u1, v1])?;
        st.solpnt.set_parameters(a, b, c, d);
    }
    if !current.is_new() {
        st.vtx = current.vertex()?;
        st.solpnt.set_vertex(input.on_first, &st.vtx);
    }
    let (normale, vtgint, vtgrst) = (st.normale, st.vtgint, st.vtgrst);
    line_arc_transitions(vtgint, vtgrst, normale, &mut st.transline, &mut st.transarc)?;
    let prm: f64 = found.currentparameter;
    st.solpnt.set_arc(input.on_first, currentarc, prm, &st.transline, &st.transarc);
    Ok(())
}

/// The points after `i` on the same underlying vertex (`Domain->Identical`) become
/// vertices of the line too, with their own arcs; `vtx` may be the vertex of an earlier
/// point when the point `i` is new, as natively.
fn same_vertex_points(
    input: &PutPointsInput<'_>,
    st: &mut PutState,
    vline: &VertexLine,
    done: &mut [i32],
    i: usize,
) -> Result<(), Failure> {
    for k in i + 1..input.listpnt.len() {
        let other: &IntPatchThePathPointOfTheSOnBounds = &input.listpnt[k];
        if done[k] == 1 || other.is_new() {
            continue;
        }
        let vtxbis: TheVertex = other.vertex()?;
        if st.vtx.is_null() || !input.domain.identical(&st.vtx, &vtxbis)? {
            continue;
        }
        st.solpnt.set_vertex(input.on_first, &vtxbis);
        st.solpnt.set_tolerance(input.tolarc);
        let (arc, prm) = (other.arc(), other.parameter());
        (st.p2d, st.d2d) = arc.d1(prm)?;
        st.vtgrst.set_linear_form(st.d2d.x(), st.d1u, st.d2d.y(), st.d1v);
        let (normale, vtgint, vtgrst) = (st.normale, st.vtgint, st.vtgrst);
        line_arc_transitions(vtgint, vtgrst, normale, &mut st.transline, &mut st.transarc)?;
        st.solpnt.set_arc(input.on_first, &arc, prm, &st.transline, &st.transarc);
        vline.add_vertex(&st.solpnt)?;
        done[k] = 1;
    }
    Ok(())
}
