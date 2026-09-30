// Created on: 1992-11-19
// Created by: Remi LEQUETTE
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
// Rust translation of the static functions of BRepClass_Intersector.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The static helpers of `BRepClass_Intersector`: the ON check, the tolerance
//! refinement on cylinders, the retry on the gap at a high-tolerance vertex, the 2d
//! tolerance of an edge, the box test of the segment and the chord tangent.
// The native tests are written `x > tol` or `x <= tol`; their negations keep the
// native outcome of a NaN operand.
#![allow(clippy::neg_cmp_op_on_partial_ord)]
use super::edge::BRepClassEdge;
use super::intersector::edge_domain;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::brep_adaptor::BRepAdaptorSurface;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{Face, Vertex};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::geom2d_int::GInter;
use rustycad_tkgeomalgo::int_res2d::{Domain, IntersectionPoint, Position, Transition};
use rustycad_tkgeombase::extrema::ExtPC2d;
use rustycad_tkgeombase::gc::MakeSegment2d;
use rustycad_tkgeombase::gce::Root;
use rustycad_tkmath::bnd::BoundingBox2;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::SurfaceType;
use rustycad_tkmath::gp::{Direction2, Line2, Point2, Vector2};

/// `Extrema_ExtPC2d`'s default function tolerance `TolF`.
const EXTREMA_TOL_F: f64 = 1.0e-10;

/// The unchanged arguments of `CheckSkip`: the segment line, the edge, its pcurve,
/// the segment domain and the adaptor of the segment line.
pub(super) struct SkipContext<'a> {
    pub(super) lin: &'a Line2,
    pub(super) edge: &'a BRepClassEdge,
    pub(super) curve: &'a Handle<dyn Geom2dCurve>,
    pub(super) dl: &'a Domain,
    pub(super) cga: &'a Geom2dAdaptorCurve,
}

/// `CheckOn`: the point of `curve` nearest to the origin of `lin` (Extrema_ExtPC2d);
/// within `tol_z` (refined on a cylinder, [`refine_tolerance`]) it is an ON point at
/// parameter 0 of the line, at the head or end of the curve when its parameter is
/// within Confusion of (or beyond) `deb` or `fin`. `check_on(&face, &lin, &c, &mut tol, 1.0, 0.0)?`.
pub(super) fn check_on(
    face: &Face,
    lin: &Line2,
    curve: &Geom2dAdaptorCurve,
    tol_z: &mut f64,
    fin: f64,
    deb: f64,
) -> Result<Option<IntersectionPoint>, Failure> {
    let extrema = ExtPC2d::with_curve(lin.location(), curve, EXTREMA_TOL_F)?;
    let (mut min_dist, mut min_ind): (f64, i32) = (real_last(), 0);
    if extrema.is_done() {
        for index in 1..=extrema.nb_ext()? {
            let dist: f64 = extrema.square_distance(index)?;
            if dist < min_dist {
                (min_dist, min_ind) = (dist, index);
            }
        }
    }
    if min_ind != 0 {
        min_dist = min_dist.sqrt();
    }
    if !(min_dist <= *tol_z) {
        return Ok(None);
    }
    let nearest = extrema.point(min_ind)?;
    let (exact, par): (Point2, f64) = (*nearest.value(), nearest.parameter());
    refine_tolerance(face, curve, par, tol_z)?;
    if !(min_dist <= *tol_z) {
        return Ok(None);
    }
    let on_curve = Transition::undecided(position_on_curve(par, deb, fin));
    let on_line = Transition::undecided(Position::Head);
    Ok(Some(IntersectionPoint::new(
        exact, 0.0, par, on_line, on_curve, false,
    )))
}

// The position of the ON point on the curve.
fn position_on_curve(par: f64, deb: f64, fin: f64) -> Position {
    if (par - deb).abs() <= precision::confusion() || par < deb {
        return Position::Head;
    }
    if (par - fin).abs() <= precision::confusion() || par > fin {
        return Position::End;
    }
    Position::Middle
}

/// `RefineTolerance`: on a cylindrical face, lowers `tol_z` to the UV resolution of
/// `tol_z` across the tangent of `curve` at `t` (at least Confusion);
/// `refine_tolerance(&face, &curve, 0.5, &mut tol)?`.
pub(super) fn refine_tolerance(
    face: &Face,
    curve: &Geom2dAdaptorCurve,
    t: f64,
    tol_z: &mut f64,
) -> Result<(), Failure> {
    let surface = BRepAdaptorSurface::with_face(face, false)?;
    if surface.get_type()? != SurfaceType::Cylinder {
        return Ok(());
    }
    let u_res: f64 = surface.u_resolution(*tol_z)?;
    let v_res: f64 = surface.v_resolution(*tol_z)?;
    let (_, tangent) = curve.d1(t)?;
    let direction = Direction2::from_vector(tangent)?;
    let mut tol_x: f64 = u_res * direction.y() + v_res * direction.x();
    if tol_x < 0.0 {
        tol_x = -tol_x;
    }
    if tol_x < precision::confusion() {
        tol_x = precision::confusion();
    }
    if tol_x < *tol_z {
        *tol_z = tol_x;
    }
    Ok(())
}

/// `CheckSkip`: when the last vertex of the edge has a tolerance above the edge's
/// maximum tolerance and the segment crosses the gap from the end of the edge to
/// the start of its next edge (both inside the segment domain), intersects the
/// segment with that gap instead: `curve` is loaded with the gap segment, `(fin, deb)`
/// and `(pdeb, pfin)` become its range and ends, and `inter` is replaced;
/// `check_skip(&mut inter, &context, &adaptor, (&mut fin, &mut deb), (&mut pdeb, &mut pfin))?`.
pub(super) fn check_skip(
    inter: &mut GInter,
    context: &SkipContext<'_>,
    curve: &Geom2dAdaptorCurve,
    (fin, deb): (&mut f64, &mut f64),
    (pdeb, pfin): (&mut Point2, &mut Point2),
) -> Result<(), Failure> {
    let Some((p1, p2)) = gap_ends(context, *fin, *deb)? else {
        return Ok(());
    };
    let (a, b, c) = context.lin.coefficients();
    let first_value: f64 = a * p1.x() + b * p1.y() + c;
    let second_value: f64 = a * p2.x() + b * p2.y() + c;
    // Check for getting into vertex with high tolerance.
    if first_value * second_value >= 0.0 {
        return Ok(());
    }
    let segment = MakeSegment2d::from_points(p1, p2)?;
    if !segment.is_done() {
        return Ok(());
    }
    let skip: Handle<dyn Geom2dCurve> = segment.value()?.cast();
    curve.load(&skip)?;
    if curve.curve().is_null() {
        return Ok(());
    }
    *deb = curve.first_parameter()?;
    *fin = curve.last_parameter()?;
    *pdeb = curve.d0(*deb)?;
    *pfin = curve.d0(*fin)?;
    let de: Domain = edge_domain(curve, (*pdeb, *deb), (*pfin, *fin))?;
    let tolerances = (precision::p_confusion(), precision::p_intersection());
    *inter = GInter::with_domains(
        context.cga,
        context.dl,
        curve,
        &de,
        tolerances.0,
        tolerances.1,
    )?;
    Ok(())
}

// The end of the edge and the start of its next edge (by their orientations), when
// the last vertex is above the maximum tolerance, the next edge has a pcurve on the
// face and both ends project strictly inside the segment domain.
fn gap_ends(
    context: &SkipContext<'_>,
    fin: f64,
    deb: f64,
) -> Result<Option<(Point2, Point2)>, Failure> {
    let edge: &BRepClassEdge = context.edge;
    if edge.edge().is_null() || edge.face().is_null() {
        return Ok(None);
    }
    let last_vertex: Vertex = topexp::last_vertex(edge.edge(), true);
    if last_vertex.is_null() {
        return Ok(None);
    }
    if !(BRepTool::vertex_tolerance(&last_vertex)? > edge.max_tolerance())
        || edge.next_edge().is_null()
    {
        return Ok(None);
    }
    let next = BRepTool::curve_on_face(edge.next_edge(), edge.face())?;
    if next.pcurve.is_null() {
        return Ok(None);
    }
    let at1: f64 = if edge.edge().orientation() != Orientation::Forward {
        deb
    } else {
        fin
    };
    let at2: f64 = if edge.next_edge().orientation() != Orientation::Forward {
        next.last
    } else {
        next.first
    };
    let p1: Point2 = context.curve.value(at1)?;
    let p2: Point2 = next.pcurve.value(at2)?;
    // Check if points belong to DL domain.
    let par1: f64 = elclib::parameter(context.lin, p1)?;
    let par2: f64 = elclib::parameter(context.lin, p2)?;
    let dl: &Domain = context.dl;
    if par1 <= dl.first_parameter()?
        || par1 >= dl.last_parameter()?
        || par2 <= dl.first_parameter()?
        || par2 >= dl.last_parameter()?
    {
        return Ok(None);
    }
    Ok(Some((p1, p2)))
}

/// `MaxTol2DCurEdge`: the UV resolution on the face of the larger vertex tolerance
/// (0 for a null vertex), at least `tolerance`;
/// `max_tol_2d_cur_edge(&v1, &v2, &face, 1e-7)?`.
pub(super) fn max_tol_2d_cur_edge(
    first: &Vertex,
    last: &Vertex,
    face: &Face,
    tolerance: f64,
) -> Result<f64, Failure> {
    let first_tol: f64 = if first.is_null() {
        0.0
    } else {
        BRepTool::vertex_tolerance(first)?
    };
    let last_tol: f64 = if last.is_null() {
        0.0
    } else {
        BRepTool::vertex_tolerance(last)?
    };
    let tol_v3d: f64 = std_max(first_tol, last_tol);
    let surface = BRepAdaptorSurface::with_face(face, false)?;
    let u_res: f64 = surface.u_resolution(tol_v3d)?;
    let v_res: f64 = surface.v_resolution(tol_v3d)?;
    Ok(std_max(std_max(u_res, v_res), tolerance))
}

/// `IsInter`: the box meets the segment `[0, param]` of `lin` (the whole line when
/// `param` is infinite); `is_inter(&bounds, &lin, 10.0)`.
pub(super) fn is_inter(bounds: &BoundingBox2, lin: &Line2, param: f64) -> bool {
    let is_out: bool = if precision::is_infinite(param) {
        bounds.is_out_line(*lin)
    } else {
        bounds.is_out_segment(lin.location(), elclib::value(param, lin))
    };
    !is_out
}

/// `GetTangentAsChord`: the direction of the chord from `param` to a point a tenth
/// of the range further (backwards at the last parameter, half the remaining range
/// near it); `tangent` is left unchanged when the chord is within PConfusion;
/// `tangent_as_chord(&pcurve, &mut tangent, 0.5, 0.0, 1.0)?`.
pub(super) fn tangent_as_chord(
    pcurve: &Handle<dyn Geom2dCurve>,
    tangent: &mut Direction2,
    param: f64,
    first: f64,
    last: f64,
) -> Result<(), Failure> {
    let mut offset: f64 = 0.1 * (last - first);
    if last - param < precision::p_confusion() {
        // param == last
        offset *= -1.0;
    } else if param + offset > last {
        // param is close to last
        offset = 0.5 * (last - param);
    }
    let point: Point2 = pcurve.value(param)?;
    let offset_point: Point2 = pcurve.value(param + offset)?;
    let mut chord = Vector2::between(point, offset_point);
    if offset < 0.0 {
        chord.reverse();
    }
    if chord.square_magnitude() > precision::square_p_confusion() {
        *tangent = Direction2::from_vector(chord)?;
    }
    Ok(())
}
