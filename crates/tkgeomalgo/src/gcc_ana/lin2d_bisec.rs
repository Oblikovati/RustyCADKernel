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
// Rust translation of GccAna_Lin2dBisec.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! GccAna_Lin2dBisec: the bisecting lines of two lines.
use super::solutions::{Slots, Tangencies, check_solution, not_done};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::int_ana2d::AnaIntersection;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Direction2, Line2, Point2, Vector2, Xy};

/// GccAna_Lin2dBisec: the bisecting lines of two lines. Secant lines give the
/// interior bisector (direction D1 + D2) then the exterior one (D1 - D2, reversed
/// when the angle from Lin1 to Lin2 is not negative); parallel lines give the
/// equidistant line; identical lines give Lin1 itself.
/// ```
/// use rustycad_tkgeomalgo::gcc_ana::Lin2dBisec;
/// use rustycad_tkmath::gp::{Direction2, Line2, Point2};
/// let x = Line2::default();
/// let y = Line2::new(Point2::default(), Direction2::new(0.0, 1.0)?);
/// assert_eq!(Lin2dBisec::new(x, y)?.nb_solutions()?, 2);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Debug)]
pub struct Lin2dBisec {
    well_done: bool,
    nbr_sol: i32,
    linsol: Slots<Line2>,
    // The intersection points and parameters on each argument (pntint<i>sol,
    // par<i>sol, pararg<i>); TheSame is unused.
    inter1: Tangencies,
    inter2: Tangencies,
}

impl Lin2dBisec {
    /// GccAna_Lin2dBisec(Lin1, Lin2); `Lin2dBisec::new(lin1, lin2)?`.
    pub fn new(lin1: Line2, lin2: Line2) -> Result<Self, Failure> {
        let mut bisec = Self {
            well_done: false,
            nbr_sol: 0,
            linsol: Slots::new(2),
            inter1: Tangencies::new(2),
            inter2: Tangencies::new(2),
        };
        let intp = AnaIntersection::lines(lin1, lin2);
        if intp.is_done() {
            if intp.parallel_elements()? {
                bisec.parallel(lin1, lin2, intp.identical_elements()?)?;
            } else if !intp.is_empty()? {
                bisec.secant(lin1, lin2, &intp)?;
            }
        }
        bisec.intersections(lin1, lin2)?;
        Ok(bisec)
    }

    // Parallel lines: Lin1 when identical, else the line halfway between them.
    fn parallel(&mut self, lin1: Line2, lin2: Line2, identical: bool) -> Result<(), Failure> {
        self.well_done = true;
        self.nbr_sol = 1;
        if identical {
            return self.linsol.set(1, lin1);
        }
        // Attention: do not use Lin1.Distance(Lin2), as the lines can be concurrent
        // for gp_Lin2d, so dist = 0 (test of the angle too strict?).
        let mut dist: f64 = lin1.distance(lin2.location()) / 2.0;
        let (dx, dy) = (lin2.direction().x(), lin2.direction().y());
        let cross: f64 =
            Vector2::new(-dy, dx).dot(Vector2::between(lin2.location(), lin1.location()));
        if cross < 0.0 {
            dist = -dist;
        }
        let origin = Point2::from(lin2.location().xy() + Xy::new(-dy * dist, dx * dist));
        self.linsol.set(1, Line2::new(origin, lin2.direction()))
    }

    // Secant lines: the interior then the exterior bisector at each intersection.
    fn secant(&mut self, lin1: Line2, lin2: Line2, intp: &AnaIntersection) -> Result<(), Failure> {
        let (d1, d2) = (lin1.direction().xy(), lin2.direction().xy());
        for i in 1..=intp.nb_points()? {
            let apex: Point2 = intp.point(i)?.value();
            self.nbr_sol += 1;
            let interior = Line2::new(apex, Direction2::from_xy(d1 + d2)?);
            self.linsol.set(self.nbr_sol, interior)?;
            self.nbr_sol += 1;
            let mut exterior = Line2::new(apex, Direction2::from_xy(d1 - d2)?);
            if lin1.angle(lin2) >= 0.0 {
                exterior.reverse();
            }
            self.linsol.set(self.nbr_sol, exterior)?;
            self.well_done = true;
        }
        Ok(())
    }

    // Each solution meets both arguments at its own location.
    fn intersections(&mut self, lin1: Line2, lin2: Line2) -> Result<(), Failure> {
        for i in 1..=self.nbr_sol {
            let line: Line2 = self.linsol.get(i)?;
            let point: Point2 = line.location();
            let par_sol: f64 = elclib::parameter(&line, point)?;
            let par1: f64 = elclib::parameter(&lin1, point)?;
            let par2: f64 = elclib::parameter(&lin2, point)?;
            self.inter1.record(i, point, par_sol, par1)?;
            self.inter2.record(i, point, par_sol, par2)?;
        }
        Ok(())
    }

    /// IsDone: true when a bisector was built; `bisec.is_done()`.
    pub fn is_done(&self) -> bool {
        self.well_done
    }

    /// NbSolutions: 1 or 2; StdFail_NotDone when the algorithm failed;
    /// `bisec.nb_solutions()?`.
    pub fn nb_solutions(&self) -> Result<i32, Failure> {
        if !self.well_done {
            return Err(not_done("GccAna_Lin2dBisec::NbSolutions"));
        }
        Ok(self.nbr_sol)
    }

    /// ThisSolution(Index): the interior (1) or exterior (2) bisector;
    /// `bisec.this_solution(1)?`.
    pub fn this_solution(&self, index: i32) -> Result<Line2, Failure> {
        check_solution(
            self.well_done,
            self.nbr_sol,
            index,
            "GccAna_Lin2dBisec::ThisSolution",
        )?;
        self.linsol.get(index)
    }

    /// Intersection1(Index, ParSol, ParArg, PntSol): where solution `index` meets
    /// Lin1; `let (par_sol, par_arg, point) = bisec.intersection1(1)?;`.
    pub fn intersection1(&self, index: i32) -> Result<(f64, f64, Point2), Failure> {
        check_solution(
            self.well_done,
            self.nbr_sol,
            index,
            "GccAna_Lin2dBisec::Intersection1",
        )?;
        self.inter1.raw(index)
    }

    /// Intersection2(Index, ParSol, ParArg, PntSol): where solution `index` meets
    /// Lin2; `let (par_sol, par_arg, point) = bisec.intersection2(1)?;`.
    pub fn intersection2(&self, index: i32) -> Result<(f64, f64, Point2), Failure> {
        check_solution(
            self.well_done,
            self.nbr_sol,
            index,
            "GccAna_Lin2dBisec::Intersection2",
        )?;
        self.inter2.raw(index)
    }
}
