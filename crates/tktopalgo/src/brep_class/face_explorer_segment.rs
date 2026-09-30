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
// Rust translation of BRepClass_FaceExplorer::OtherSegment (BRepClass_FaceExplorer.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepClass_FaceExplorer::OtherSegment`: the next segment from a point to a point
//! of a probe edge that is not tangent to the edge and does not pass through its ends.
// The native tests are written `x > tol` or `x <= tol`; their negations keep the
// native outcome of a NaN operand.
#![allow(clippy::neg_cmp_op_on_partial_ord)]
use super::face_explorer::{BRepClassFaceExplorer, PROBING_START};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::geom2d_api::ProjectPointOnCurve;
use rustycad_tkmath::gp::{Direction2, Line2, Point2, Vector2};

/// `Probing_End`: the probing of an edge stops at this fraction of its range.
const PROBING_END: f64 = 0.7;
/// `Probing_Step`: the step of the probed fraction.
const PROBING_STEP: f64 = 0.2111;
/// `SmallAngle`: below this sine the segment is tangent to the edge.
const SMALL_ANGLE: f64 = 0.001;
/// `aTolParConf2`: `Precision::PConfusion()` squared.
const TOL_PAR_CONF2: f64 = precision::p_confusion() * precision::p_confusion();

/// A segment found by the probing: its line and its (squared, until returned) length.
struct Found {
    line: Line2,
    param: f64,
}

/// The outcome of one probed fraction of an edge.
enum Probe {
    /// `continue`: the next fraction.
    Next,
    /// The segment to return.
    Segment(Found),
}

/// The pcurve of the probe edge and its (finite) probing range.
struct ProbedCurve<'a> {
    curve: &'a Handle<dyn Geom2dCurve>,
    first: f64,
    last: f64,
}

impl BRepClassFaceExplorer {
    /// `OtherSegment(P, L, Par)`: from the current probe edge and fraction, the next
    /// segment from `point` to a point of an edge (FORWARD or REVERSED, with a
    /// pcurve) that is not tangent to it and whose line misses the edge ends; with
    /// no such segment, the horizontal line from `point` and `RealLast`. Each call
    /// resumes after the previous segment. The pcurve exceptions are returned;
    /// `let (found, line, par) = explorer.other_segment(&point)?;`.
    pub fn other_segment(&mut self, point: &Point2) -> Result<(bool, Line2, f64), Failure> {
        while self.my_cur_edge_ind < self.my_probe_edges.len() {
            if let Some(found) = self.probe_current_edge(point)? {
                return Ok((true, found.line, found.param.sqrt()));
            }
            // This curve is not valid for line construction. Go to another edge.
            self.my_cur_edge_ind += 1;
            self.my_cur_edge_par = PROBING_START;
        }
        // Nothing found: a horizontal line.
        let horizontal = Line2::new(*point, Direction2::new(1.0, 0.0)?);
        Ok((false, horizontal, real_last()))
    }

    // The fractions of the current probe edge from the current one; the found
    // segment advances the fraction (and the edge past the last fraction).
    fn probe_current_edge(&mut self, point: &Point2) -> Result<Option<Found>, Failure> {
        let edge = &self.my_edges[self.my_probe_edges[self.my_cur_edge_ind]];
        let orientation: Orientation = edge.edge().orientation();
        if orientation != Orientation::Forward && orientation != Orientation::Reversed {
            return Ok(None);
        }
        let curve = edge.curve().clone();
        if curve.is_null() {
            return Ok(None);
        }
        let (first, last) = finite_range(edge.first_parameter(), edge.last_parameter());
        let probed = ProbedCurve {
            curve: &curve,
            first,
            last,
        };
        while self.my_cur_edge_par < PROBING_END {
            if let Probe::Segment(found) = self.probe_fraction(point, &probed)? {
                self.advance_after_segment();
                return Ok(Some(found));
            }
            self.my_cur_edge_par += PROBING_STEP;
        }
        Ok(None)
    }

    // After a found segment: the next fraction, or the next edge past the last one.
    fn advance_after_segment(&mut self) {
        self.my_cur_edge_par += PROBING_STEP;
        if self.my_cur_edge_par >= PROBING_END {
            self.my_cur_edge_ind += 1;
            self.my_cur_edge_par = PROBING_START;
        }
    }

    // One fraction: the segment from `point` to the curve point at the fraction,
    // unless it is too short, the tangent vanishes, the segment is tangent to the
    // curve (except at the last fraction) or its line passes through an end.
    fn probe_fraction(&self, point: &Point2, probed: &ProbedCurve<'_>) -> Result<Probe, Failure> {
        let fraction: f64 = self.my_cur_edge_par;
        let param_in: f64 = fraction * probed.first + (1.0 - fraction) * probed.last;
        let (on_curve, mut tangent) = probed.curve.d1(param_in)?;
        let param: f64 = on_curve.square_distance(*point);
        if !(param > TOL_PAR_CONF2) {
            return Ok(Probe::Next);
        }
        let lin_dir = Direction2::from_vector(Vector2::between(*point, on_curve))?;
        let tan_mod: f64 = tangent.square_magnitude();
        if tan_mod < TOL_PAR_CONF2 {
            return Ok(Probe::Next);
        }
        tangent /= tan_mod.sqrt();
        let sin_a: f64 = tangent.crossed(Vector2::new(lin_dir.x(), lin_dir.y()));
        let is_small_angle: bool = sin_a.abs() < SMALL_ANGLE;
        // The segment is tangent to the edge, which is bad for classification: try
        // another fraction, but keep the last one so that this edge still counts.
        if is_small_angle && fraction + PROBING_STEP < PROBING_END {
            return Ok(Probe::Next);
        }
        let line = Line2::new(*point, lin_dir);
        let first_on_curve: Point2 = probed.curve.d0(probed.first)?;
        if !(line.square_distance(first_on_curve) > TOL_PAR_CONF2) {
            return Ok(Probe::Next);
        }
        let last_on_curve: Point2 = probed.curve.d0(probed.last)?;
        if !(line.square_distance(last_on_curve) > TOL_PAR_CONF2) {
            return Ok(Probe::Next);
        }
        let found = Found { line, param };
        if !is_small_angle {
            return Ok(Probe::Segment(found));
        }
        let ends = (first_on_curve, last_on_curve);
        closest_approach(point, probed, ends, found)
    }
}

// The treatment of infinite ranges: a unit range at the finite end, [-1, 1] when
// both ends are infinite.
fn finite_range(first: f64, last: f64) -> (f64, f64) {
    if precision::is_negative_infinite(first) {
        if precision::is_positive_infinite(last) {
            return (-1.0, 1.0);
        }
        return (last - 1.0, last);
    }
    if precision::is_positive_infinite(last) {
        return (first, first + 1.0);
    }
    (first, last)
}

// A tangent segment: aims at the curve point closest to `point` (its projection or
// an end) when it is closer than the probed point; a closest point within
// PConfusion goes to the next fraction.
fn closest_approach(
    point: &Point2,
    probed: &ProbedCurve<'_>,
    (first_on_curve, last_on_curve): (Point2, Point2),
    mut found: Found,
) -> Result<Probe, Failure> {
    let mut projection = ProjectPointOnCurve::new();
    projection.init_range(*point, probed.curve, probed.first, probed.last)?;
    if projection.nb_points() <= 0 {
        return Ok(Probe::Segment(found));
    }
    let first_dist: f64 = point.square_distance(first_on_curve);
    let last_dist: f64 = point.square_distance(last_on_curve);
    let mut min_dist: f64 = projection.lower_distance()?;
    min_dist *= min_dist;
    let mut closest: Point2 = projection.nearest_point()?;
    if min_dist > first_dist {
        min_dist = first_dist;
        closest = first_on_curve;
    }
    if min_dist > last_dist {
        min_dist = last_dist;
        closest = last_on_curve;
    }
    if !(min_dist < found.param) {
        return Ok(Probe::Segment(found));
    }
    found.param = min_dist;
    if found.param < TOL_PAR_CONF2 {
        return Ok(Probe::Next);
    }
    let direction = Direction2::from_xy(closest.xy() - point.xy())?;
    found.line = Line2::new(*point, direction);
    Ok(Probe::Segment(found))
}
