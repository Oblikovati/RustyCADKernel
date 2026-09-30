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
// Rust translation of the segment part of IntPatch_ImpPrmIntersection::Perform of
// ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ImpPrmIntersection.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The restrictions of the parametric surface that lie on the quadric become
//! restriction lines, with their polygon and the vertices of the lines at their ends.
// The negated comparisons keep the native result for NaN operands.
#![allow(clippy::neg_cmp_op_on_partial_ord)]
use super::super::IntPatchIType;
use super::super::path_point::{IntPatchThePathPointOfTheSOnBounds, IntPatchTheSegmentOfTheSOnBounds};
use super::super::point::IntPatchPoint;
use super::super::point_line::IntPatchPointLine;
use super::super::rline::IntPatchRLine;
use super::super::wline::IntPatchWLine;
use super::IntPatchImpPrmIntersection;
use super::perform::PrmInput;
use crate::int_start::TheArc;
use crate::int_surf::{IntSurfLineOn2S, IntSurfPntOn2S, IntSurfTransition, IntSurfTypeTrans};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Point2, Point3, Vector3};

/// The vertex at one end of the segment and the transitions of the line there.
struct SegmentEnd {
    point: IntPatchPoint,
    pstart: IntPatchThePathPointOfTheSOnBounds,
    param: f64,
    /// `(trans1, trans2)`, or None when the mixed product is null.
    transitions: Option<(IntSurfTypeTrans, IntSurfTypeTrans)>,
}

impl IntPatchImpPrmIntersection {
    /// The restriction lines of the segments (a degenerated segment is skipped).
    pub(super) fn segments(&mut self, input: &PrmInput<'_>) -> Result<(), Failure> {
        let nb_segm: i32 = self.solrst.nb_segments()?;
        for i in 1..=nb_segm {
            let thesegm: IntPatchTheSegmentOfTheSOnBounds = self.solrst.segment(i)?.clone();
            if is_degenerated(input, &thesegm)? {
                continue;
            }
            let rline = self.segment_line(input, &thesegm)?;
            self.slin.push(rline.cast());
        }
        Ok(())
    }

    /// The restriction line of one segment.
    fn segment_line(&mut self, input: &PrmInput<'_>, thesegm: &IntPatchTheSegmentOfTheSOnBounds) -> Result<Handle<IntPatchRLine>, Failure> {
        let arc: TheArc = thesegm.curve();
        let first: Option<SegmentEnd> = if thesegm.has_first_point() { Some(segment_end(input, &arc, thesegm.first_point()?)?) } else { None };
        let last: Option<SegmentEnd> = if thesegm.has_last_point() { Some(segment_end(input, &arc, thesegm.last_point()?)?) } else { None };
        // The transitions of the last end win (TransitionOK is set by each end).
        let transitions = match (&first, &last) {
            (_, Some(l)) => l.transitions,
            (Some(f), None) => f.transitions,
            (None, None) => None,
        };
        let rline = match transitions {
            Some((t1, t2)) => IntPatchRLine::new(false, t1, t2),
            None => IntPatchRLine::new_undecided(false),
        };
        if input.reversed {
            rline.set_arc_on_s1(&arc);
        } else {
            rline.set_arc_on_s2(&arc);
        }
        let rline = Handle::new(rline);
        if let Some(f) = &first {
            rline.add_vertex(&f.point, false);
            rline.set_first_point(rline.nb_vertex());
        }
        if let Some(l) = &last {
            rline.add_vertex(&l.point, false);
            rline.set_last_point(rline.nb_vertex());
        }
        if let (Some(f), Some(l)) = (&first, &last) {
            rline.add(&polygon(input, &arc, (f.param, l.param))?);
        }
        if first.is_some() || last.is_some() {
            self.segment_ends_on_lines(&rline, (&first, &last), input.tol_arc)?;
        }
        Ok(rline)
    }

    /// The vertices of the lines at the ends of the segment are made multiple and added
    /// to the restriction line.
    fn segment_ends_on_lines(&self, rline: &Handle<IntPatchRLine>, (first, last): (&Option<SegmentEnd>, &Option<SegmentEnd>), tol_arc: f64) -> Result<(), Failure> {
        let (mut procf, mut procl) = (false, false);
        for slinj in &self.slin {
            let vline = SegLine::of(slinj)?;
            for k in 1..=vline.line().nb_vertex() {
                let mut ptdeb: IntPatchPoint = vline.line().vertex(k)?;
                if let Some(f) = first
                    && ptdeb.value().distance(&f.pstart.value()) <= tol_arc
                {
                    mark_end(&vline, rline, (&mut ptdeb, k), f.param)?;
                    if !procf {
                        procf = true;
                        rline.set_first_point(rline.nb_vertex());
                    }
                }
                if let Some(l) = last {
                    if first.is_some() {
                        // The vertex is read again.
                        ptdeb = vline.line().vertex(k)?;
                    }
                    if ptdeb.value().distance(&l.pstart.value()) <= tol_arc {
                        mark_end(&vline, rline, (&mut ptdeb, k), l.param)?;
                        if !procl {
                            procl = true;
                            rline.set_last_point(rline.nb_vertex());
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

/// The vertex `k` of the line made multiple, replaced, and added to the restriction line
/// at the parameter of the end.
fn mark_end(vline: &SegLine, rline: &IntPatchRLine, (ptdeb, k): (&mut IntPatchPoint, i32), param: f64) -> Result<(), Failure> {
    ptdeb.set_multiple(true);
    vline.replace(k, ptdeb)?;
    ptdeb.set_parameter(param);
    rline.add_vertex(ptdeb, false);
    Ok(())
}

/// A walking or restriction line of the result (the native down cast by type).
enum SegLine {
    Walking(Handle<IntPatchWLine>),
    Restriction(Handle<IntPatchRLine>),
}

impl SegLine {
    fn of(l: &Handle<dyn super::super::line::IntPatchLine>) -> Result<Self, Failure> {
        if l.arc_type() == IntPatchIType::Walking {
            return Ok(Self::Walking(l.cast()));
        }
        let r: Handle<IntPatchRLine> = l.cast();
        if r.is_null() {
            return Err(super::super::failures::null_object("IntPatch_ImpPrmIntersection::Perform", "IntPatch_RLine"));
        }
        Ok(Self::Restriction(r))
    }
    fn line(&self) -> &dyn IntPatchPointLine {
        match self {
            Self::Walking(w) => w.get().expect("walking line"),
            Self::Restriction(r) => r.get().expect("restriction line"),
        }
    }
    fn replace(&self, k: i32, p: &IntPatchPoint) -> Result<(), Failure> {
        match self {
            Self::Walking(w) => w.replace(k, p),
            Self::Restriction(r) => r.replace(k, p),
        }
    }
}

/// Whether the segment is degenerated: its ends are the same point and so is its
/// point at `.57735 * f + .42265 * l`.
fn is_degenerated(input: &PrmInput<'_>, seg: &IntPatchTheSegmentOfTheSOnBounds) -> Result<bool, Failure> {
    if !(seg.has_first_point() && seg.has_last_point()) {
        return Ok(false);
    }
    let tol2: f64 = precision::confusion() * precision::confusion();
    let (pf, pl) = (seg.first_point()?, seg.last_point()?);
    if !(pf.value().square_distance(&pl.value()) <= tol2) {
        return Ok(false);
    }
    let p2d: Point2 = seg.curve().value(0.57735 * pf.parameter() + 0.42265 * pl.parameter())?;
    let pm: Point3 = input.psurf().0.d0(p2d.x(), p2d.y())?;
    Ok(pm.square_distance(&pf.value()) <= tol2)
}

/// The vertex at an end of the segment (the quadric parameters of its point, the
/// parametric ones of the arc) and the transitions of the line from the arc tangent.
fn segment_end(input: &PrmInput<'_>, arc: &TheArc, pstart: &IntPatchThePathPointOfTheSOnBounds) -> Result<SegmentEnd, Failure> {
    let param: f64 = pstart.parameter();
    let p2d: Point2 = arc.value(param)?;
    let pp: Point3 = pstart.value();
    let mut point = IntPatchPoint::new();
    point.set_value_tangent(pp, pstart.tolerance(), false);
    let (qu, qv) = input.quad.parameters(pp).unwrap_or((0.0, 0.0));
    let [u1, v1, u2, v2] = if !input.reversed { [qu, qv, p2d.x(), p2d.y()] } else { [p2d.x(), p2d.y(), qu, qv] };
    point.set_parameters(u1, v1, u2, v2);
    point.set_parameter(param);
    if !pstart.is_new() {
        point.set_vertex(input.reversed, &pstart.vertex()?);
    }
    let unk = IntSurfTransition::default();
    point.set_arc(input.reversed, arc, param, &unk, &unk);
    let transitions = end_transitions(input, arc, param, [u1, v1, u2, v2])?;
    Ok(SegmentEnd { point, pstart: pstart.clone(), param, transitions })
}

/// The mixed product of the arc tangent (on the parametric surface) and the normals.
fn end_transitions(input: &PrmInput<'_>, arc: &TheArc, param: f64, [u1, v1, u2, v2]: [f64; 4]) -> Result<Option<(IntSurfTypeTrans, IntSurfTypeTrans)>, Failure> {
    let s1 = input.surf1.d1(u1, v1)?;
    let norm1: Vector3 = s1.d1u.crossed(s1.d1v);
    let s2 = input.surf2.d1(u2, v2)?;
    let norm2: Vector3 = s2.d1u.crossed(s2.d1v);
    let (_, d2d) = arc.d1(param)?;
    let mut tgline = Vector3::default();
    if input.reversed {
        tgline.set_linear_form(d2d.x(), s1.d1u, d2d.y(), s1.d1v);
    } else {
        tgline.set_linear_form(d2d.x(), s2.d1u, d2d.y(), s2.d1v);
    }
    let u: f64 = tgline.dot_cross(norm2, norm1);
    if u > 0.00000001 {
        return Ok(Some((IntSurfTypeTrans::Out, IntSurfTypeTrans::In)));
    }
    if u < -0.00000001 {
        return Ok(Some((IntSurfTypeTrans::In, IntSurfTypeTrans::Out)));
    }
    Ok(None)
}

/// The polygon of 100 points of the segment on the parametric surface, with the
/// quadric parameters of its points.
fn polygon(input: &PrmInput<'_>, arc: &TheArc, (paramf, paraml): (f64, f64)) -> Result<Handle<IntSurfLineOn2S>, Failure> {
    let thelin = Handle::new(IntSurfLineOn2S::new(None));
    let nbsample: i32 = 100;
    let psurf = input.psurf().0;
    for j in 1..=nbsample {
        let prm: f64 = paramf + f64::from(j - 1) * (paraml - paramf) / f64::from(nbsample - 1);
        let p2d: Point2 = arc.d0(prm)?;
        let ptpoly: Point3 = psurf.d0(p2d.x(), p2d.y())?;
        let (qu, qv) = input.quad.parameters(ptpoly).unwrap_or((0.0, 0.0));
        let mut p2s = IntSurfPntOn2S::new();
        if !input.reversed {
            p2s.set_value_all(ptpoly, qu, qv, p2d.x(), p2d.y());
        } else {
            p2s.set_value_all(ptpoly, p2d.x(), p2d.y(), qu, qv);
        }
        thelin.add(&p2s);
    }
    Ok(thelin)
}
