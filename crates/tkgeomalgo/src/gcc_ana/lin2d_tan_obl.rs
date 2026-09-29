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
// Rust translation of GccAna_Lin2dTanObl.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! GccAna_Lin2dTanObl: lines through a point or tangent to a circle, at an angle
//! to a line.
use super::check_qualifier;
use super::solutions::{Slots, Tangencies, check_solution, not_done};
use crate::gcc_ent::{Position, QualifiedCirc, bad_qualifier};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::int_ana2d::AnaIntersection;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Direction2, Line2, Point2, Xy};

/// GccAna_Lin2dTanObl: the lines passing through a point, or tangent to a circle,
/// that make the angle `TheAngle` with a line. Solution `i` stores its tangency
/// (or passing) point and its intersection with the reference line; for a parallel
/// solution the intersection point stays at the origin, as the native default
/// gp_Pnt2d does.
/// ```
/// use rustycad_tkgeomalgo::gcc_ana::Lin2dTanObl;
/// use rustycad_tkmath::gp::{Line2, Point2};
/// let obl = Lin2dTanObl::through_point(Point2::new(0.0, 1.0), Line2::default(), 0.5)?;
/// assert_eq!(obl.nb_solutions()?, 1);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Debug)]
pub struct Lin2dTanObl {
    well_done: bool,
    nbr_sol: i32,
    linsol: Slots<Line2>,
    qualifier1: Slots<Position>,
    tangency1: Tangencies,
    intersection2: Tangencies,
}

// The direction of TheLine turned by TheAngle, in the native operation order.
fn turned(the_line: Line2, the_angle: f64) -> Xy {
    let (cosa, sina) = (the_line.direction().x(), the_line.direction().y());
    Xy::new(
        the_angle.cos() * cosa - the_angle.sin() * sina,
        the_angle.cos() * sina + the_angle.sin() * cosa,
    )
}

// The last intersection point of `line` and `the_line`, if any (the native loop
// keeps the last point).
fn last_intersection(line: Line2, the_line: Line2) -> Result<Option<Point2>, Failure> {
    let intp = AnaIntersection::lines(line, the_line);
    if !intp.is_done() || intp.is_empty()? {
        return Ok(None);
    }
    let mut last = None;
    for i in 1..=intp.nb_points()? {
        last = Some(intp.point(i)?.value());
    }
    Ok(last)
}

impl Lin2dTanObl {
    fn empty(size: usize) -> Self {
        Self {
            well_done: false,
            nbr_sol: 0,
            linsol: Slots::new(size),
            qualifier1: Slots::new(size),
            tangency1: Tangencies::new(size),
            intersection2: Tangencies::new(size),
        }
    }

    /// GccAna_Lin2dTanObl(ThePoint, TheLine, TheAngle): the line through
    /// `the_point` at `the_angle` from `the_line`;
    /// `Lin2dTanObl::through_point(p, line, angle)?`.
    pub fn through_point(
        the_point: Point2,
        the_line: Line2,
        the_angle: f64,
    ) -> Result<Self, Failure> {
        let mut obl = Self::empty(1);
        let (cosa, sina) = (the_line.direction().x(), the_line.direction().y());
        // Native writes `std::sin(TheAngle) * Cosa` in the second coordinate.
        let direction = Direction2::new(
            cosa * the_angle.cos() - sina * the_angle.sin(),
            sina * the_angle.cos() + the_angle.sin() * cosa,
        )?;
        let line = Line2::new(the_point, direction);
        obl.linsol.set(1, line)?;
        obl.qualifier1.set(1, Position::NoQualifier)?;
        let intp = AnaIntersection::lines(line, the_line);
        if !intp.is_done() {
            return Ok(obl);
        }
        let mut pntint2 = Point2::default();
        if let Some(point) = last_intersection(line, the_line)? {
            pntint2 = point;
        }
        let par1: f64 = elclib::parameter(&line, the_point)?;
        obl.tangency1.record(1, the_point, par1, 0.0)?;
        let par2: f64 = elclib::parameter(&line, pntint2)?;
        let pararg2: f64 = elclib::parameter(&the_line, pntint2)?;
        obl.intersection2.record(1, pntint2, par2, pararg2)?;
        obl.nbr_sol = 1;
        obl.well_done = true;
        Ok(obl)
    }

    /// GccAna_Lin2dTanObl(Qualified1, TheLine, TheAngle): the lines tangent to a
    /// circle at `the_angle` from `the_line`: one for an enclosing or outside
    /// circle, two for an unqualified one; GccEnt_BadQualifier for an enclosed
    /// circle; `Lin2dTanObl::tangent_to_circle(qualified, line, angle)?`.
    pub fn tangent_to_circle(
        qualified1: QualifiedCirc,
        the_line: Line2,
        the_angle: f64,
    ) -> Result<Self, Failure> {
        let member = "GccAna_Lin2dTanObl";
        check_qualifier(qualified1.qualifier(), member)?;
        if qualified1.is_enclosed() {
            return Err(bad_qualifier(
                "GccAna_Lin2dTanObl: Qualified1 is GccEnt_enclosed; expected enclosing, outside or unqualified",
            ));
        }
        let mut obl = Self::empty(2);
        let c1: Circle2 = qualified1.qualified();
        let xy: Xy = turned(the_line, the_angle);
        let right: Xy = Xy::new(xy.y(), -xy.x());
        let left: Xy = Xy::new(-xy.y(), xy.x());
        if qualified1.is_enclosing() {
            obl.add_tangent(c1, right, xy, qualified1.qualifier(), the_line)?;
        } else if qualified1.is_outside() {
            obl.add_tangent(c1, left, xy, qualified1.qualifier(), the_line)?;
        } else if qualified1.is_unqualified() {
            obl.add_tangent(c1, right, xy, Position::Enclosing, the_line)?;
            obl.add_second_unqualified(c1, left, xy, the_line)?;
        }
        obl.tangency_parameters(c1, the_line)?;
        Ok(obl)
    }

    // One solution tangent to `c1` on the side `normal`, parallel to `xy`.
    fn add_tangent(
        &mut self,
        c1: Circle2,
        normal: Xy,
        xy: Xy,
        qualifier: Position,
        the_line: Line2,
    ) -> Result<(), Failure> {
        let index: i32 = self.nbr_sol + 1;
        let point = Point2::from(c1.location().xy() + c1.radius() * normal);
        let line = Line2::new(point, Direction2::from_xy(xy)?);
        self.tangency1.point.set(index, point)?;
        self.linsol.set(index, line)?;
        self.qualifier1.set(index, qualifier)?;
        self.nbr_sol = index;
        self.well_done = true;
        if let Some(pntint2) = last_intersection(line, the_line)? {
            self.intersection2.point.set(index, pntint2)?;
        }
        Ok(())
    }

    // The outside solution of an unqualified circle. Native intersects the first
    // solution (linsol(1)), not the second, with TheLine; kept.
    fn add_second_unqualified(
        &mut self,
        c1: Circle2,
        left: Xy,
        xy: Xy,
        the_line: Line2,
    ) -> Result<(), Failure> {
        let point = Point2::from(c1.location().xy() + c1.radius() * left);
        self.tangency1.point.set(2, point)?;
        self.linsol
            .set(2, Line2::new(point, Direction2::from_xy(xy)?))?;
        self.qualifier1.set(2, Position::Outside)?;
        self.nbr_sol += 1;
        self.well_done = true;
        if let Some(pntint2) = last_intersection(self.linsol.get(1)?, the_line)? {
            self.intersection2.point.set(2, pntint2)?;
        }
        Ok(())
    }

    fn tangency_parameters(&mut self, c1: Circle2, the_line: Line2) -> Result<(), Failure> {
        for index in 1..=self.nbr_sol {
            let line: Line2 = self.linsol.get(index)?;
            let tangency: Point2 = self.tangency1.point.get(index)?;
            let pntint2: Point2 = self.intersection2.point.get(index)?;
            let par1: f64 = elclib::parameter(&line, tangency)?;
            let pararg1: f64 = elclib::parameter(&c1, tangency)?;
            self.tangency1.record(index, tangency, par1, pararg1)?;
            let par2: f64 = elclib::parameter(&line, pntint2)?;
            let pararg2: f64 = elclib::parameter(&the_line, pntint2)?;
            self.intersection2.record(index, pntint2, par2, pararg2)?;
        }
        Ok(())
    }

    /// IsDone: true when the construction succeeded; `obl.is_done()`.
    pub fn is_done(&self) -> bool {
        self.well_done
    }

    /// NbSolutions: StdFail_NotDone when the construction failed;
    /// `obl.nb_solutions()?`.
    pub fn nb_solutions(&self) -> Result<i32, Failure> {
        if !self.well_done {
            return Err(not_done("GccAna_Lin2dTanObl::NbSolutions"));
        }
        Ok(self.nbr_sol)
    }

    /// ThisSolution(Index); `obl.this_solution(1)?`.
    pub fn this_solution(&self, index: i32) -> Result<Line2, Failure> {
        let member = "GccAna_Lin2dTanObl::ThisSolution";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        self.linsol.get(index)
    }

    /// WhichQualifier(Index, Qualif1): the position of solution `index` relative
    /// to the circle (GccEnt_noqualifier for the point constructor);
    /// `obl.which_qualifier(1)?`.
    pub fn which_qualifier(&self, index: i32) -> Result<Position, Failure> {
        let member = "GccAna_Lin2dTanObl::WhichQualifier";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        self.qualifier1.get(index)
    }

    /// Tangency1(Index, ParSol, ParArg, PntSol): the tangency point on the circle
    /// (the passing point, of argument parameter 0, for the point constructor);
    /// `let (par_sol, par_arg, point) = obl.tangency1(1)?;`.
    pub fn tangency1(&self, index: i32) -> Result<(f64, f64, Point2), Failure> {
        let member = "GccAna_Lin2dTanObl::Tangency1";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        self.tangency1.raw(index)
    }

    /// Intersection2(Index, ParSol, ParArg, PntSol): the intersection with
    /// `TheLine`; `let (par_sol, par_arg, point) = obl.intersection2(1)?;`.
    pub fn intersection2(&self, index: i32) -> Result<(f64, f64, Point2), Failure> {
        let member = "GccAna_Lin2dTanObl::Intersection2";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        self.intersection2.raw(index)
    }
}
