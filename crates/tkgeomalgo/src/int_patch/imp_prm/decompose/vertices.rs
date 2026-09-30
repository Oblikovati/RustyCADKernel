// Copyright (c) 1992-1999 Matra Datavision
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
// Rust translation of GetVertices, SearchVertices, TestMiddleOnPrm, VerifyVertices,
// AddVertices, PutIntVertices and HasInternals of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpPrmIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The vertices of a line during its decomposition.
use super::super::super::point::IntPatchPoint;
use super::super::super::point_line::IntPatchPointLine;
use super::super::super::rline::IntPatchRLine;
use super::seam::{adjust_u, is_seam_parameter};
use crate::int_start::{TheArc, TheTopolTool};
use crate::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::State;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Line2, Point2, Point3};
use std::f64::consts::PI;

const TWO_PI: f64 = 2.0 * PI;

/// `GetVertices(thePLine, TOL3D, TOL2D)`: the points of the vertices of the line,
/// without the vertices equal (in 3d and in the four parameters) to an earlier one.
pub(super) fn get_vertices(
    pline: &dyn IntPatchPointLine,
    tol_3d: f64,
    tol_2d: f64,
) -> Result<Handle<IntSurfLineOn2S>, Failure> {
    let vertices = Handle::new(IntSurfLineOn2S::new(None));
    let nb_vrt: i32 = pline.nb_vertex();
    let mut an_vrts: Vec<i32> = vec![0; nb_vrt as usize + 1];
    for i in 1..=nb_vrt {
        if an_vrts[i as usize] == -1 {
            continue;
        }
        let pi: IntPatchPoint = pline.vertex(i)?;
        for k in i + 1..=nb_vrt {
            if an_vrts[k as usize] == -1 {
                continue;
            }
            let pk: IntPatchPoint = pline.vertex(k)?;
            if pi.value().distance(&pk.value()) <= tol_3d && same_parameters(&pi, &pk, tol_2d) {
                an_vrts[k as usize] = -1;
            }
        }
    }
    for i in 1..=nb_vrt {
        if an_vrts[i as usize] != -1 {
            vertices.add(&pline.vertex(i)?.pnt_on_2s());
        }
    }
    Ok(vertices)
}

/// Whether the four parameters of two vertices are within `tol_2d`.
fn same_parameters(pi: &IntPatchPoint, pk: &IntPatchPoint, tol_2d: f64) -> bool {
    let (a, b, c, d) = pi.parameters();
    let (e, f, g, h) = pk.parameters();
    (a - e).abs() <= tol_2d
        && (b - f).abs() <= tol_2d
        && (c - g).abs() <= tol_2d
        && (d - h).abs() <= tol_2d
}

/// `SearchVertices(Line, Vertices, PTypes)`: for each point of the line the rank of the
/// first vertex it is (1-based `ptypes`, 0 for none).
pub(super) fn search_vertices(
    line: &IntSurfLineOn2S,
    vertices: &IntSurfLineOn2S,
) -> Result<Vec<i32>, Failure> {
    let (nbp, nbv) = (line.nb_points(), vertices.nb_points());
    let mut ptypes: Vec<i32> = vec![0; nbp as usize + 1];
    for ip in 1..=nbp {
        let ap: IntSurfPntOn2S = line.value(ip)?;
        for iv in 1..=nbv {
            if ap.is_same(
                &vertices.value(iv)?,
                precision::confusion(),
                precision::p_confusion(),
            ) {
                ptypes[ip as usize] = iv;
                break;
            }
        }
    }
    Ok(ptypes)
}

/// `TestMiddleOnPrm(aP, aV, IsReversed, ArcTol, PDomain)`: whether the middle of the
/// point and the vertex on the parametric surface is in (or on) its domain.
fn test_middle_on_prm(
    ap: &IntSurfPntOn2S,
    av: &IntSurfPntOn2S,
    is_reversed: bool,
    arc_tol: f64,
    pdomain: &(dyn TheTopolTool + 'static),
) -> Result<bool, Failure> {
    let (up, vp) = ap.parameters_on_surface(is_reversed);
    let (uv, vv) = av.parameters_on_surface(is_reversed);
    let pm = Point2::new((up + uv) * 0.5, (vp + vv) * 0.5);
    let pos: State = pdomain.classify(pm, arc_tol, true)?;
    Ok(pos == State::On || pos == State::In)
}

/// The vertex nearest to an end of the line: the same one, or the nearest (in the
/// quadric parameters, also through the seam: `conjugated`).
struct Nearest {
    same: i32,
    near: i32,
    dist_min: f64,
    conjugated: i32,
}

fn nearest_vertex(
    end: &IntSurfPntOn2S,
    vertices: &IntSurfLineOn2S,
    is_reversed: bool,
    tol_2d: f64,
) -> Result<Nearest, Failure> {
    let (ue, ve) = end.parameters_on_surface(!is_reversed);
    let p2d = Point2::new(ue, ve);
    let mut n = Nearest {
        same: 0,
        near: 0,
        dist_min: 1.0e+100,
        conjugated: 0,
    };
    for iv in 1..=vertices.nb_points() {
        let av: IntSurfPntOn2S = vertices.value(iv)?;
        if end.is_same(&av, precision::confusion(), precision::p_confusion()) {
            n.same = iv;
            break;
        }
        let (uv, vv) = av.parameters_on_surface(!is_reversed);
        let dist: f64 = Point2::new(uv, vv).distance(p2d);
        if dist < n.dist_min {
            n.dist_min = dist;
            n.near = iv;
            n.conjugated = 0;
        }
        if is_seam_parameter(uv, tol_2d) {
            let ucv: f64 = if uv.abs() < (TWO_PI - uv).abs() {
                TWO_PI
            } else {
                0.0
            };
            let cdist: f64 = Point2::new(ucv, vv).distance(p2d);
            if cdist < n.dist_min {
                n.dist_min = cdist;
                n.conjugated = iv;
                n.near = iv;
            }
        }
    }
    Ok(n)
}

/// The vertex to add at an end: a seam vertex whose middle with the end is in the
/// domain, farther than 1.5e-7 and than `tol_2d` (in the quadric parameters).
fn end_vertex(
    end: &IntSurfPntOn2S,
    vertices: &IntSurfLineOn2S,
    n: &Nearest,
    (is_reversed, tol_2d, arc_tol, pdomain): (bool, f64, f64, &(dyn TheTopolTool + 'static)),
    vrt: &mut IntSurfPntOn2S,
) -> Result<bool, Failure> {
    if n.same != 0 || n.near == 0 {
        return Ok(false);
    }
    let av: IntSurfPntOn2S = vertices.value(n.near)?;
    let (uv, vv) = av.parameters_on_surface(!is_reversed);
    if !is_seam_parameter(uv, tol_2d) {
        // To do: analyze an internal vertex.
        return Ok(false);
    }
    let ucv: f64 = if uv.abs() < (TWO_PI - uv).abs() {
        TWO_PI
    } else {
        0.0
    };
    if !test_middle_on_prm(end, &av, is_reversed, arc_tol, pdomain)? {
        return Ok(false);
    }
    vrt.set_point(av.value());
    let (u2, v2) = av.parameters_on_surface(is_reversed);
    vrt.set_parameters_on(is_reversed, u2, v2);
    vrt.set_parameters_on(!is_reversed, if n.conjugated == 0 { uv } else { ucv }, vv);
    let dist_3d: f64 = vrt.value().distance(&end.value());
    Ok(dist_3d > 1.5e-7 && n.dist_min > tol_2d)
}

/// `VerifyVertices(Line, IsReversed, Vertices, TOL2D, ArcTol, PDomain, VrtF, AddFirst,
/// VrtL, AddLast)`: the quadric U of the vertices in `[0, 2 PI]`, then the vertices to
/// add at the first and last points of the line.
#[allow(clippy::too_many_arguments)]
pub(super) fn verify_vertices(
    line: &IntSurfLineOn2S,
    is_reversed: bool,
    vertices: &IntSurfLineOn2S,
    tol_2d: f64,
    arc_tol: f64,
    pdomain: &(dyn TheTopolTool + 'static),
    vrt_f: &mut IntSurfPntOn2S,
    vrt_l: &mut IntSurfPntOn2S,
) -> Result<(bool, bool), Failure> {
    let nbp: i32 = line.nb_points();
    let (apf, apl) = (line.value(1)?, line.value(nbp)?);
    for iv in 1..=vertices.nb_points() {
        let (uv, vv) = vertices.value(iv)?.parameters_on_surface(!is_reversed);
        vertices.set_uv(iv, !is_reversed, adjust_u(uv), vv)?;
    }
    let nf: Nearest = nearest_vertex(&apf, vertices, is_reversed, tol_2d)?;
    let nl: Nearest = nearest_vertex(&apl, vertices, is_reversed, tol_2d)?;
    let ctx = (is_reversed, tol_2d, arc_tol, pdomain);
    let add_first: bool = end_vertex(&apf, vertices, &nf, ctx, vrt_f)?;
    let add_last: bool = end_vertex(&apl, vertices, &nl, ctx, vrt_l)?;
    Ok((add_first, add_last))
}

/// `AddVertices(Line, VrtF, AddFirst, VrtL, AddLast, D3DF, D3DL)`: the end vertices
/// inserted when they are nearer than twice the mean step and farther than 1.5e-7.
pub(super) fn add_vertices(
    line: &IntSurfLineOn2S,
    (vrt_f, add_first): (&IntSurfPntOn2S, bool),
    (vrt_l, add_last): (&IntSurfPntOn2S, bool),
    (d3df, d3dl): (f64, f64),
) -> Result<bool, Failure> {
    let mut result: bool = false;
    if add_first {
        let df: f64 = line.value(1)?.value().distance(&vrt_f.value());
        if (d3df * 2.0) > df && df > 1.5e-7 {
            line.insert_before(1, vrt_f)?;
            result = true;
        }
    }
    if add_last {
        let dl: f64 = line
            .value(line.nb_points())?
            .value()
            .distance(&vrt_l.value());
        if (d3dl * 2.0) > dl && dl > 1.5e-7 {
            line.add(vrt_l);
            result = true;
        }
    }
    Ok(result)
}

/// `PutIntVertices(Line, Result, theIsReversed, Vertices, ArcTol)`: a vertex on the line
/// at each inner point that is a vertex (its parameter is the point index, or the
/// parameter on the arc of a restriction line).
pub(super) fn put_int_vertices(
    pline: &dyn IntPatchPointLine,
    rline: Option<&IntPatchRLine>,
    result: &IntSurfLineOn2S,
    is_reversed: bool,
    (vertices, arc_tol): (&IntSurfLineOn2S, f64),
) -> Result<(), Failure> {
    let (nbp, nbv) = (result.nb_points(), vertices.nb_points());
    if nbp < 3 {
        return Ok(());
    }
    let mut the_pnt = IntPatchPoint::new();
    for ip in 2..nbp {
        let ap: IntSurfPntOn2S = result.value(ip)?;
        for iv in 1..=nbv {
            if !ap.is_same(
                &vertices.value(iv)?,
                precision::confusion(),
                precision::p_confusion(),
            ) {
                continue;
            }
            let pnt: Point3 = ap.value();
            let (u1, v1, u2, v2) = ap.parameters();
            the_pnt.set_value_tangent(pnt, arc_tol, false);
            the_pnt.set_parameters(u1, v1, u2, v2);
            let param: f64 = match rline {
                // The restriction line is on the parametric surface.
                Some(r) => {
                    let arc: TheArc = if r.is_arc_on_s1() {
                        r.arc_on_s1()
                    } else {
                        r.arc_on_s2()
                    };
                    let lin: Line2 = arc.line()?;
                    let ps = if is_reversed {
                        Point2::new(u1, v1)
                    } else {
                        Point2::new(u2, v2)
                    };
                    elclib::parameter(&lin, ps)?
                }
                None => f64::from(ip),
            };
            the_pnt.set_parameter(param);
            pline.add_vertex(&the_pnt, false);
        }
    }
    Ok(())
}

/// `HasInternals(Line, Vertices)`: whether an inner point of the line is a vertex.
pub(super) fn has_internals(
    line: &IntSurfLineOn2S,
    vertices: &IntSurfLineOn2S,
) -> Result<bool, Failure> {
    let (nbp, nbv) = (line.nb_points(), vertices.nb_points());
    if nbp < 3 {
        return Ok(false);
    }
    for ip in 2..nbp {
        let ap: IntSurfPntOn2S = line.value(ip)?;
        for iv in 1..=nbv {
            if ap.is_same(
                &vertices.value(iv)?,
                precision::confusion(),
                precision::p_confusion(),
            ) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
