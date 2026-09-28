// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of Extrema_ExtCC::PrepareParallelResult at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The parallel results of Extrema_ExtCC on trimmed curves: kept for other curve
//! kinds; for a line and a circle when the circle point projects on the line range;
//! for two lines or two circles, reduced to the overlap of the projected ranges, to
//! a single middle pair or to the nearest range ends.
use super::ExtCC;
use crate::extrema::ExtPElC;
use crate::extrema::curve_tool::ExtremaCurveTool;
use crate::extrema::curve_tools::CurveTool;
use crate::extrema::pon::POnCurv;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::real;
use rustycad_tkg2d::geom2d::native::{std_max, std_min};
use rustycad_tkmath::bnd::Range1;
use rustycad_tkmath::elclib;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::gp::{Circle3, Line3, Point3, Vector3};
use std::f64::consts::PI;

impl ExtCC<'_> {
    /// `PrepareParallelResult(theUt11, theUt12, theUt21, theUt22, theSqDist)` (protected).
    pub(super) fn prepare_parallel_result(
        &mut self,
        ranges: [f64; 4],
        sq_dist: f64,
    ) -> Result<(), Failure> {
        if !self.is_parallel {
            return Ok(());
        }
        let [c1, c2] = self.curve_pair()?;
        let (type1, type2) = (CurveTool::get_type(c1)?, CurveTool::get_type(c2)?);
        let line_or_circle = |t: CurveType| t == CurveType::Line || t == CurveType::Circle;
        if !line_or_circle(type1) || !line_or_circle(type2) {
            self.sq_dist.push(sq_dist);
            self.done = true;
            self.is_parallel = true;
            return Ok(());
        }
        // Parallel case is only for line-line, circle-circle and circle-line!!!
        // But really for trimmed curves extremas can not exist!
        if type1 != type2 {
            return self.circle_line_parallel(type1, ranges, sq_dist);
        }
        if type1 == CurveType::Line {
            return self.lines_parallel(ranges, sq_dist);
        }
        self.circles_parallel(ranges, sq_dist)
    }

    // The projection of the circle point at its range start on the trimmed line
    // must exist.
    fn circle_line_parallel(
        &mut self,
        type1: CurveType,
        [ut11, ut12, ut21, ut22]: [f64; 4],
        sq_dist: f64,
    ) -> Result<(), Failure> {
        let [c1, c2] = self.curve_pair()?;
        let is_reversed: bool = type1 != CurveType::Circle;
        let (p_on_c, line, bounds) = if is_reversed {
            (CurveTool::value(c2, ut21)?, c1.line()?, (ut11, ut12))
        } else {
            (CurveTool::value(c1, ut11)?, c2.line()?, (ut21, ut22))
        };
        let projection =
            ExtPElC::with_line(p_on_c, line, precision::confusion(), bounds.0, bounds.1);
        if projection.is_done() {
            self.sq_dist.push(sq_dist);
        } else {
            self.is_parallel = false;
        }
        Ok(())
    }

    // Line - Line: the projected range of the first line on the second must meet
    // the second's range.
    fn lines_parallel(&mut self, ranges: [f64; 4], sq_dist: f64) -> Result<(), Failure> {
        let [ut11, ut12, ut21, ut22] = ranges;
        let is_first_infinite: bool = precision::is_infinite(ut11) && precision::is_infinite(ut12);
        let is_last_infinite: bool = precision::is_infinite(ut21) && precision::is_infinite(ut22);
        if is_first_infinite || is_last_infinite {
            // Infinite number of solution
            self.sq_dist.push(sq_dist);
            return Ok(());
        }
        self.is_parallel = false;
        let [c1, c2] = self.curve_pair()?;
        let (lin1, lin2) = (c1.line()?, c2.line()?);
        let is_opposite: bool = lin1.direction().dot(lin2.direction()) < 0.0;
        let mut range2 = Range1::new(ut21, ut22)?;
        let mut proj_rng12 = Range1::default();
        proj_rng12.add(projected_end(&lin1, &lin2, ut11, is_opposite)?);
        proj_rng12.add(projected_end(&lin1, &lin2, ut12, !is_opposite)?);
        range2.common(proj_rng12);
        if range2.delta() > precision::confusion() {
            self.clear_solutions();
            self.sq_dist.push(sq_dist);
            self.is_parallel = true;
        } else if !range2.is_void() {
            self.lines_touching(&lin1, &lin2, &range2, sq_dist)?;
        } else {
            self.nearest_ends(sq_dist)?;
        }
        Ok(())
    }

    // The ranges touch: the middle of the common range on the second line and its
    // projection on the first.
    fn lines_touching(
        &mut self,
        lin1: &Line3,
        lin2: &Line3,
        range2: &Range1,
        sq_dist: f64,
    ) -> Result<(), Failure> {
        self.clear_solutions();
        let (mut par1, mut par2): (f64, f64) = (0.0, 0.0);
        range2.get_bounds(&mut par1, &mut par2);
        par2 = 0.5 * (par1 + par2);
        let mut p: Point3 = elclib::value(par2, lin2);
        let p2 = POnCurv::new(par2, p);
        par1 = elclib::parameter(lin1, p)?;
        p = elclib::value(par1, lin1);
        self.points.extend([POnCurv::new(par1, p), p2]);
        self.sq_dist.push(sq_dist);
        Ok(())
    }

    // The ranges do not meet: the nearest pair of range ends (the first of equals).
    fn nearest_ends(&mut self, _sq_dist: f64) -> Result<(), Failure> {
        let (dmin, imin) = self.minimal_trimmed_distance();
        let (p1, p2) = self.end_pair(imin);
        self.clear_solutions();
        self.points.extend([p1, p2]);
        self.sq_dist.push(dmin);
        Ok(())
    }

    // The smallest of the four trimmed square distances and its index.
    fn minimal_trimmed_distance(&self) -> (f64, usize) {
        let (mut dmin, mut imin) = (self.dist[0], 0);
        for i in 1..4 {
            if dmin > self.dist[i] {
                dmin = self.dist[i];
                imin = i;
            }
        }
        (dmin, imin)
    }

    // The range ends of trimmed distance `index` (11, 12, 21, 22).
    fn end_pair(&self, index: usize) -> (POnCurv, POnCurv) {
        let first = if index < 2 {
            POnCurv::new(self.inf[0], self.p1f)
        } else {
            POnCurv::new(self.sup[0], self.p1l)
        };
        let second = if index.is_multiple_of(2) {
            POnCurv::new(self.inf[1], self.p2f)
        } else {
            POnCurv::new(self.sup[1], self.p2l)
        };
        (first, second)
    }
}

// The parameter on `lin2` of the point of `lin1` at `u`, or the infinite end
// (positive when `positive_infinity`) for an infinite `u`.
fn projected_end(
    lin1: &Line3,
    lin2: &Line3,
    u: f64,
    positive_infinity: bool,
) -> Result<f64, Failure> {
    if precision::is_infinite(u) {
        let infinite: f64 = precision::infinite();
        return Ok(if positive_infinity {
            infinite
        } else {
            -infinite
        });
    }
    elclib::parameter(lin2, elclib::value(u, lin1))
}

impl ExtCC<'_> {
    // Circle - Circle: the arcs are parallel when the projection of the first arc on
    // the second circle meets the second arc, possibly half a turn apart.
    fn circles_parallel(&mut self, ranges: [f64; 4], sq_dist: f64) -> Result<(), Failure> {
        self.is_parallel = false;
        let [c1, c2] = self.curve_pair()?;
        let [ut11, ut12, ut21, ut22] = ranges;
        let work_circ: Circle3 = c2.circle()?;
        let period: f64 = PI + PI;
        let p12: Point3 = CurveTool::value(c1, ut12)?;
        let (p11, v_tg1): (Point3, Vector3) = CurveTool::d1(c1, ut11)?;
        let range = Range1::new(ut21, ut22)?;
        // Precision of the calculation depends on circles radii
        let precision_r: f64 = std_max(
            real::epsilon(c1.circle()?.radius()),
            real::epsilon(c2.circle()?.radius()),
        );
        let par1: f64 = elclib::in_period(elclib::parameter(&work_circ, p11)?, ut21, ut21 + period);
        let v_tg2: Vector3 = CurveTool::dn(c2, par1, 1)?;
        // Check if circles have same/opposite directions
        let is_opposite: bool = v_tg1.dot(v_tg2) < 0.0;
        let mut par2: f64 =
            elclib::in_period(elclib::parameter(&work_circ, p12)?, ut21, ut21 + period);
        if is_opposite {
            // Must be aPar2 < aPar1
            if (range.delta() > precision::angular()) && ((par1 - par2) < precision::angular()) {
                par2 -= period;
            }
        } else if (range.delta() > precision::angular()) && ((par2 - par1) < precision::angular()) {
            // Must be aPar2 > aPar1
            par2 += period;
        }
        let mut proj_rng1 = Range1::default();
        proj_rng1.add(par1 - period);
        proj_rng1.add(par2 - period);
        let context = CircleArcs {
            work_circ,
            first: c1.circle()?,
            range,
            bounds: (ut11, ut12),
            precision: precision_r,
            sq_dist,
        };
        self.shifted_projections(&context, proj_rng1)
    }

    // The three half-turn shifts of the projected range (a `continue` skips the
    // shift, as natively).
    fn shifted_projections(
        &mut self,
        context: &CircleArcs,
        mut proj_rng1: Range1,
    ) -> Result<(), Failure> {
        let mut min_square_dist: f64 = f64::MAX;
        for _ in 0..3 {
            let mut rng: Range1 = proj_rng1;
            rng.common(context.range);
            if rng.delta() > precision::confusion() {
                match self.overlapping_arcs(context, &rng, min_square_dist)? {
                    ArcStep::Skip => continue,
                    ArcStep::Stop => break,
                    ArcStep::Next => {}
                }
            } else if !rng.is_void() {
                self.touching_arcs(context, &rng, &mut min_square_dist)?;
            } else {
                self.separate_arcs(context.precision, &mut min_square_dist);
            }
            proj_rng1.shift(PI);
        }
        Ok(())
    }
}

/// The data of the circle-circle parallel check.
struct CircleArcs {
    work_circ: Circle3,
    first: Circle3,
    range: Range1,
    bounds: (f64, f64),
    precision: f64,
    sq_dist: f64,
}

/// What the loop does after an overlapping range.
enum ArcStep {
    Skip,
    Stop,
    Next,
}

impl ExtCC<'_> {
    // The arcs overlap: the distance from the middle of the common range to the
    // first arc; a smaller one replaces the solutions as a parallel distance, and a
    // distance equal to the parallel one ends the search.
    fn overlapping_arcs(
        &mut self,
        context: &CircleArcs,
        rng: &Range1,
        min_square_dist: f64,
    ) -> Result<ArcStep, Failure> {
        let mut par: f64 = 0.0;
        rng.get_intermediate_point(0.5, &mut par);
        let p_circ2: Point3 = elclib::value(par, &context.work_circ);
        let (u1, u2) = context.bounds;
        let ext = ExtPElC::with_circle(p_circ2, context.first, precision::confusion(), u1, u2)?;
        if ext.nb_ext()? < 1 {
            return Ok(ArcStep::Skip);
        }
        let mut min_sq_d: f64 = ext.square_distance(1)?;
        for id in 2..=ext.nb_ext()? {
            min_sq_d = std_min(min_sq_d, ext.square_distance(id)?);
        }
        if min_sq_d <= min_square_dist + (1. + min_sq_d) * context.precision {
            self.clear_solutions();
            self.sq_dist.push(min_sq_d);
            self.is_parallel = true;
            let delta_sq_dist: f64 = min_sq_d - context.sq_dist;
            let sq_d: f64 = std_max(min_sq_d, context.sq_dist);
            if delta_sq_dist * delta_sq_dist < 4.0 * sq_d * precision::square_confusion() {
                // New solution is found
                return Ok(ArcStep::Stop);
            }
        }
        // Nearer solution can be found
        Ok(ArcStep::Next)
    }

    // The arcs touch: the extrema from the middle point, unless a parallel distance
    // is already smaller.
    fn touching_arcs(
        &mut self,
        context: &CircleArcs,
        rng: &Range1,
        min_square_dist: &mut f64,
    ) -> Result<(), Failure> {
        let mut par: f64 = 0.0;
        rng.get_intermediate_point(0.5, &mut par);
        let p_circ2: Point3 = elclib::value(par, &context.work_circ);
        let p2 = POnCurv::new(par, p_circ2);
        let (u1, u2) = context.bounds;
        let ext = ExtPElC::with_circle(p_circ2, context.first, precision::confusion(), u1, u2)?;
        let mut is_found: bool = !self.is_parallel;
        if !is_found {
            for id in 1..=ext.nb_ext()? {
                if ext.square_distance(id)? < *min_square_dist {
                    is_found = true;
                    break;
                }
            }
        }
        if is_found {
            self.clear_solutions();
            self.is_parallel = false;
            for id in 1..=ext.nb_ext()? {
                self.points.extend([ext.point(id)?, p2]);
                self.sq_dist.push(ext.square_distance(id)?);
                *min_square_dist = std_min(*min_square_dist, ext.square_distance(id)?);
            }
        }
        Ok(())
    }

    // The arcs do not meet: the nearest range ends, when not farther than the
    // distance found.
    fn separate_arcs(&mut self, precision_r: f64, min_square_dist: &mut f64) {
        self.is_parallel = false;
        let (dmin, imin) = self.minimal_trimmed_distance();
        if dmin <= *min_square_dist + (1. + dmin) * precision_r {
            let (p1, p2) = self.end_pair(imin);
            self.clear_solutions();
            self.points.extend([p1, p2]);
            self.sq_dist.push(dmin);
            *min_square_dist = std_min(*min_square_dist, dmin);
        }
    }
}
