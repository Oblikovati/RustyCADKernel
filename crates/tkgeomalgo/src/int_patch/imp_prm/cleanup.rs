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
// Rust translation of the restriction part of IntPatch_ImpPrmIntersection::Perform,
// CheckSegmSegm and IsCoincide of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpPrmIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The vertices on the restrictions of the quadric, the lines too short, and the lines
//! that coincide with a restriction line.
use super::super::IntPatchIType;
use super::super::line::IntPatchLine;
use super::super::point_line::IntPatchPointLine;
use super::super::rline::IntPatchRLine;
use super::super::rst_int::IntPatchRstInt;
use super::super::IntPatchTheSurfFunction;
use super::IntPatchImpPrmIntersection;
use super::perform::PrmInput;
use crate::int_start::TheArc;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Line2, Point2};
use rustycad_tkmath::math::Vector;

/// The point line of a line of the result.
pub(super) fn point_line(l: &Handle<dyn IntPatchLine>) -> Result<Handle<dyn IntPatchPointLine>, Failure> {
    let p: Handle<dyn IntPatchPointLine> = l.cast();
    if p.is_null() {
        return Err(super::super::failures::null_object("IntPatch_ImpPrmIntersection::Perform", "IntPatch_PointLine"));
    }
    Ok(p)
}

/// The arc of a restriction line.
fn arc_of(r: &IntPatchRLine) -> TheArc {
    if r.is_arc_on_s1() { r.arc_on_s1() } else { r.arc_on_s2() }
}

impl IntPatchImpPrmIntersection {
    /// The vertices of every line on the restrictions of the quadric (IntPatch_RstInt);
    /// a line of fewer than two distinct points is removed, a walking line goes to the
    /// end (the restriction lines come first); then the lines that coincide with an
    /// isoline restriction line are removed.
    pub(super) fn restrictions(&mut self, input: &PrmInput<'_>, func: &mut IntPatchTheSurfFunction) -> Result<(), Failure> {
        self.put_vertices_on_lines(input)?;
        let tol_3d: f64 = std_max(func.tolerance(), input.tol_tang);
        self.remove_coincident(input, func, tol_3d)
    }

    fn put_vertices_on_lines(&mut self, input: &PrmInput<'_>) -> Result<(), Failure> {
        let mut i: usize = 0;
        let mut nb_lin: usize = self.slin.len();
        while i < nb_lin {
            let l: Handle<dyn IntPatchLine> = self.slin[i].clone();
            if !input.reversed {
                IntPatchRstInt::put_vertex_on_line(&l, input.surf1, input.d1, input.surf2, true, input.tol_tang)?;
            } else {
                IntPatchRstInt::put_vertex_on_line(&l, input.surf2, input.d2, input.surf1, false, input.tol_tang)?;
            }
            let pl = point_line(&l)?;
            let nb: i32 = pl.nb_pnts()?;
            if nb <= 2 && (nb < 2 || pl.point(1)?.is_same(&pl.point(2)?, precision::confusion(), -1.0)) {
                self.slin.remove(i);
                nb_lin -= 1;
                continue;
            }
            if l.arc_type() == IntPatchIType::Walking {
                self.slin.push(l);
                self.slin.remove(i);
                nb_lin -= 1;
                continue;
            }
            i += 1;
        }
        Ok(())
    }

    /// The restriction lines (first in the list) against the lines after them: a
    /// walking line along an isoline restriction is removed, of two coincident
    /// restriction lines the shorter one.
    fn remove_coincident(&mut self, input: &PrmInput<'_>, func: &mut IntPatchTheSurfFunction, tol_3d: f64) -> Result<(), Failure> {
        let other = if input.reversed { input.surf1 } else { input.surf2 };
        let mut i: usize = 0;
        while i < self.slin.len() {
            let arl1: Handle<IntPatchRLine> = self.slin[i].cast();
            let Some(rl1) = arl1.get() else {
                // Walking-walking cases are not supported.
                break;
            };
            let arc: TheArc = arc_of(rl1);
            if arc.get_type()? != CurveType::Line {
                // The restriction line must be an isoline.
                break;
            }
            let mut is_first_deleted: bool = false;
            let mut j: usize = i + 1;
            while j < self.slin.len() {
                let l2: Handle<dyn IntPatchLine> = self.slin[j].clone();
                let arl2: Handle<IntPatchRLine> = l2.cast();
                if let Some(rl2) = arl2.get()
                    && arc_of(rl2).get_type()? != CurveType::Line
                {
                    j += 1;
                    continue;
                }
                // The direction is {0, +-1} or {+-1, 0}: the domain is rectangular.
                let dir = arc.line()?.direction();
                let (mut tol_2d, mut period) = (other.u_resolution(tol_3d)?, if other.is_v_periodic()? { other.v_period()? } else { 0.0 });
                if dir.x().abs() < 0.5 {
                    // The restriction goes along V.
                    tol_2d = other.v_resolution(tol_3d)?;
                    period = if other.is_u_periodic()? { other.u_period()? } else { 0.0 };
                }
                let coincide = is_coincide(func, &point_line(&l2)?, &arc, rl1.is_arc_on_s1(), (tol_3d, tol_2d, period))?;
                if !coincide {
                    j += 1;
                    continue;
                }
                let Some(rl2) = arl2.get() else {
                    // Delete the walking line.
                    self.slin.remove(j);
                    continue;
                };
                let arc2: TheArc = arc_of(rl2);
                let range2: f64 = arc2.last_parameter()? - arc2.first_parameter()?;
                let range1: f64 = arc.last_parameter()? - arc.first_parameter()?;
                if range2 > range1 {
                    is_first_deleted = true;
                    break;
                }
                self.slin.remove(j);
            }
            if is_first_deleted {
                self.slin.remove(i);
                continue;
            }
            i += 1;
        }
        Ok(())
    }
}

/// `CheckSegmSegm(theRefParF, theRefParL, theParF, theParL)`: whether `[f, l]` is in
/// the reference segment.
fn check_segm_segm(ref_f: f64, ref_l: f64, f: f64, l: f64) -> bool {
    if f < ref_f || f > ref_l {
        return false;
    }
    !(l < ref_f || l > ref_l)
}

/// `IsCoincide(theFunc, theLine, theArc, isTheSurface1Using, theToler3D, theToler2D,
/// thePeriod)`: whether the line lies on the arc (a 2d line) of the parametric surface:
/// a parallel restriction line at a null (or period) distance whose segment is in the
/// arc one or conversely; or a walking line whose points project inside the arc and are
/// on it, or in a tangent zone (7 points toward the arc on the quadric).
pub(super) fn is_coincide(
    func: &mut IntPatchTheSurfFunction,
    line: &Handle<dyn IntPatchPointLine>,
    arc: &TheArc,
    is_surface1_using: bool,
    (toler_3d, toler_2d, period): (f64, f64, f64),
) -> Result<bool, Failure> {
    if line.arc_type() == IntPatchIType::Restriction {
        let r: Handle<IntPatchRLine> = line.cast();
        return rline_coincide(arc, r.get().expect("restriction line"), (toler_2d, period));
    }
    let arc_lin: Line2 = arc.line()?;
    let (uaf, ual) = (arc.first_parameter()?, arc.last_parameter()?);
    for pt_id in 1..=line.nb_pnts()? {
        let (uf, vf) = line.point(pt_id)?.parameters_on_surface(is_surface1_using);
        // A 2d point of the parametric surface, whose 2d line is the arc.
        let ploc = Point2::new(uf, vf);
        let rparam: f64 = elclib::parameter(&arc_lin, ploc)?;
        if rparam < uaf || rparam > ual {
            return Ok(false);
        }
        let pmin: Point2 = elclib::value(rparam, &arc_lin);
        let dist: f64 = ploc.distance(pmin);
        if dist < toler_2d || (dist - period).abs() < toler_2d {
            // The point is on the restriction line.
            continue;
        }
        if !in_tangent_zone(func, (uf, vf), pmin, toler_3d)? {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Whether the points between the line point and its projection on the arc are on the
/// quadric (a tangent zone of the two surfaces).
fn in_tangent_zone(func: &mut IntPatchTheSurfFunction, (uf, vf): (f64, f64), pmin: Point2, toler_3d: f64) -> Result<bool, Failure> {
    // sin(x)*sin(x)
    let coeffs: [f64; 7] = [0.02447174185, 0.09549150281, 0.20610737385, 0.34549150281, 0.5, 0.65450849719, 0.79389262615];
    let (du, dv) = (pmin.x() - uf, pmin.y() - vf);
    let mut x = Vector::with_bounds(1, 2)?;
    let mut val = Vector::with_bounds(1, 1)?;
    for c in coeffs {
        x[1] = uf + c * du;
        x[2] = vf + c * dv;
        if !func.value(&x, &mut val)? {
            return Ok(false);
        }
        if func.root().abs() > toler_3d {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Two restriction lines: parallel isolines at a null (or period) distance with one
/// segment in the other.
fn rline_coincide(arc: &TheArc, rl2: &IntPatchRLine, (toler_2d, period): (f64, f64)) -> Result<bool, Failure> {
    let arc2: TheArc = arc_of(rl2);
    if arc2.get_type()? != CurveType::Line {
        return Ok(false);
    }
    let (lin1, lin2): (Line2, Line2) = (arc.line()?, arc2.line()?);
    if !lin1.direction().is_parallel(lin2.direction(), precision::angular()) {
        return Ok(false);
    }
    let dist: f64 = arc.line()?.distance_line(arc2.line()?);
    if !(dist < toler_2d || (dist - period).abs() < toler_2d) {
        return Ok(false);
    }
    let (rf, rl) = (arc.first_parameter()?, arc.last_parameter()?);
    let (parf, parl) = (arc2.first_parameter()?, arc2.last_parameter()?);
    let (p1, p2) = (elclib::value(parf, &lin2), elclib::value(parl, &lin2));
    let (param1, param2) = (elclib::parameter(&lin1, p1)?, elclib::parameter(&lin1, p2)?);
    if check_segm_segm(rf, rl, param1, param2) {
        return Ok(true);
    }
    // The lines are parallel: there is no point in projecting the points again.
    Ok(check_segm_segm(param1, param2, rf, rl))
}
