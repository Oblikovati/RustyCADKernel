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
// Rust translation of GccAna_Lin2d2Tan.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! GccAna_Lin2d2Tan: lines through two points, or tangent to a circle and through
//! a point, or tangent to two circles.
mod two_circles;

use super::check_qualifier;
use super::solutions::{Slots, Tangencies, check_index, check_solution};
use crate::gcc_ent::{Position, QualifiedCirc, bad_qualifier};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Direction2, Line2, Point2, Vector2};

/// GccAna_Lin2d2Tan: the lines through two points (one solution), tangent to a
/// qualified circle and through a point (up to two) or tangent to two qualified
/// circles (up to four). Solutions keep the native order; see the progress notes
/// for the qualifier slots that native leaves unset.
/// ```
/// use rustycad_tkgeomalgo::gcc_ana::Lin2d2Tan;
/// use rustycad_tkmath::gp::Point2;
/// let lin = Lin2d2Tan::through_points(Point2::new(0.0, 0.0), Point2::new(1.0, 1.0), 1e-7)?;
/// assert_eq!(lin.nb_solutions(), 1);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Debug)]
pub struct Lin2d2Tan {
    well_done: bool,
    nbr_sol: i32,
    linsol: Slots<Line2>,
    qualifier1: Slots<Position>,
    qualifier2: Slots<Position>,
    tangency1: Tangencies,
    tangency2: Tangencies,
}

const MEMBER: &str = "GccAna_Lin2d2Tan";

impl Lin2d2Tan {
    fn empty(size: usize) -> Self {
        Self {
            well_done: false,
            nbr_sol: 0,
            linsol: Slots::new(size),
            qualifier1: Slots::new(size),
            qualifier2: Slots::new(size),
            tangency1: Tangencies::new(size),
            tangency2: Tangencies::new(size),
        }
    }

    // Store solution `index`: its line, qualifiers and the two tangency points.
    fn store(
        &mut self,
        index: i32,
        line: Line2,
        qualifiers: (Position, Position),
        points: (Point2, Point2),
    ) -> Result<(), Failure> {
        self.linsol.set(index, line)?;
        self.qualifier1.set(index, qualifiers.0)?;
        self.qualifier2.set(index, qualifiers.1)?;
        self.tangency1.point.set(index, points.0)?;
        self.tangency2.point.set(index, points.1)
    }

    /// GccAna_Lin2d2Tan(ThePoint1, ThePoint2, Tolerance): the line through two
    /// points at least |Tolerance| apart; `Lin2d2Tan::through_points(p1, p2, tol)?`.
    pub fn through_points(point1: Point2, point2: Point2, tolerance: f64) -> Result<Self, Failure> {
        let tol: f64 = tolerance.abs();
        let mut lin = Self::empty(1);
        lin.qualifier1.set(1, Position::NoQualifier)?;
        lin.qualifier2.set(1, Position::NoQualifier)?;
        if point1.distance(point2) >= tol {
            let dir = Direction2::new(point2.x() - point1.x(), point2.y() - point1.y())?;
            let line = Line2::new(point1, dir);
            lin.linsol.set(1, line)?;
            lin.well_done = true;
            lin.nbr_sol = 1;
            let par1: f64 = elclib::parameter(&line, point1)?;
            let par2: f64 = elclib::parameter(&line, point2)?;
            lin.tangency1.record(1, point1, par1, 0.0)?;
            lin.tangency2.record(1, point2, par2, 0.0)?;
        }
        Ok(lin)
    }

    /// GccAna_Lin2d2Tan(Qualified1, ThePoint, Tolerance): the lines tangent to a
    /// circle and through a point: none for a point inside, one for a point on the
    /// circle or an enclosing/outside circle, two for an unqualified circle;
    /// GccEnt_BadQualifier for an enclosed circle;
    /// `Lin2d2Tan::tangent_through_point(qualified, point, tol)?`.
    pub fn tangent_through_point(
        qualified1: QualifiedCirc,
        the_point: Point2,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let tol: f64 = tolerance.abs();
        let mut lin = Self::empty(2);
        check_qualifier(qualified1.qualifier(), MEMBER)?;
        let c1: Circle2 = qualified1.qualified();
        let r1: f64 = c1.radius();
        if qualified1.is_enclosed() {
            return Err(bad_qualifier(
                "GccAna_Lin2d2Tan: Qualified1 is GccEnt_enclosed; expected enclosing, outside or unqualified",
            ));
        }
        if tol < r1 - the_point.distance(c1.location()) {
            lin.well_done = true;
        } else if (the_point.distance(c1.location()) - r1).abs() <= tol {
            let dir = Direction2::from_vector(Vector2::between(c1.location(), the_point))?;
            let line = Line2::new(the_point, Direction2::new(-dir.y(), dir.x())?);
            let qualifiers = (qualified1.qualifier(), Position::NoQualifier);
            lin.store(1, line, qualifiers, (the_point, the_point))?;
            lin.well_done = true;
            lin.nbr_sol = 1;
        } else {
            lin.tangents_from_point(qualified1, the_point)?;
        }
        for i in 1..=lin.nbr_sol {
            lin.parameters(i, c1, None)?;
        }
        Ok(lin)
    }

    // The tangents from a point outside the circle: turn the centre around the
    // point by asin(+-R1/dist), then slide to the tangency point.
    fn tangents_from_point(
        &mut self,
        qualified1: QualifiedCirc,
        point: Point2,
    ) -> Result<(), Failure> {
        let c1: Circle2 = qualified1.qualified();
        let r1: f64 = c1.radius();
        let mut signe: f64 = 1.0;
        let dist: f64 = point.distance(c1.location());
        let d: f64 = dist - (dist * dist - r1 * r1).sqrt();
        if qualified1.is_enclosing() {
            self.nbr_sol = 1;
        } else if qualified1.is_outside() {
            signe = -1.0;
            self.nbr_sol = 1;
        } else if qualified1.is_unqualified() {
            self.nbr_sol = 2;
        }
        for i in 1..=self.nbr_sol {
            let p1: Point2 = c1.location().rotated(point, (signe * r1 / dist).asin());
            let d1 = Direction2::from_vector(Vector2::between(p1, point))?;
            let p1 = Point2::from(p1.xy() + d * d1.xy());
            let line = Line2::new(p1, Direction2::from_vector(Vector2::between(p1, point))?);
            let qualifiers = (qualified1.qualifier(), Position::NoQualifier);
            self.store(i, line, qualifiers, (p1, point))?;
            signe = -signe;
        }
        self.well_done = true;
        Ok(())
    }

    // The parameters of solution `i` on itself and on its arguments; the second
    // argument parameter is 0 for a point (`c2` = None).
    fn parameters(&mut self, i: i32, c1: Circle2, c2: Option<Circle2>) -> Result<(), Failure> {
        let line: Line2 = self.linsol.get(i)?;
        let (p1, p2) = (self.tangency1.point.get(i)?, self.tangency2.point.get(i)?);
        let par1: f64 = elclib::parameter(&line, p1)?;
        let par2: f64 = elclib::parameter(&line, p2)?;
        let pararg1: f64 = elclib::parameter(&c1, p1)?;
        let pararg2: f64 = match c2 {
            Some(c2) => elclib::parameter(&c2, p2)?,
            None => 0.0,
        };
        self.tangency1.record(i, p1, par1, pararg1)?;
        self.tangency2.record(i, p2, par2, pararg2)
    }

    /// IsDone: true when the construction ran to its end; `lin.is_done()`.
    pub fn is_done(&self) -> bool {
        self.well_done
    }

    /// NbSolutions (no WellDone test natively); `lin.nb_solutions()`.
    pub fn nb_solutions(&self) -> i32 {
        self.nbr_sol
    }

    /// ThisSolution(Index): Standard_OutOfRange outside [1, NbSolutions] (no
    /// WellDone test natively); `lin.this_solution(1)?`.
    pub fn this_solution(&self, index: i32) -> Result<Line2, Failure> {
        check_index(self.nbr_sol, index, "GccAna_Lin2d2Tan::ThisSolution")?;
        self.linsol.get(index)
    }

    /// WhichQualifier(Index, Qualif1, Qualif2); `let (q1, q2) = lin.which_qualifier(1)?;`.
    pub fn which_qualifier(&self, index: i32) -> Result<(Position, Position), Failure> {
        let member = "GccAna_Lin2d2Tan::WhichQualifier";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        Ok((self.qualifier1.get(index)?, self.qualifier2.get(index)?))
    }

    /// Tangency1(Index, ParSol, ParArg, PntSol): the tangency point on the first
    /// argument; `let (par_sol, par_arg, point) = lin.tangency1(1)?;`.
    pub fn tangency1(&self, index: i32) -> Result<(f64, f64, Point2), Failure> {
        let member = "GccAna_Lin2d2Tan::Tangency1";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        self.tangency1.raw(index)
    }

    /// Tangency2(Index, ParSol, ParArg, PntSol): the tangency point on the second
    /// argument; `let (par_sol, par_arg, point) = lin.tangency2(1)?;`.
    pub fn tangency2(&self, index: i32) -> Result<(f64, f64, Point2), Failure> {
        let member = "GccAna_Lin2d2Tan::Tangency2";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        self.tangency2.raw(index)
    }
}
