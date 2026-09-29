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
// Rust translation of GccAna_Circ2d3Tan.hxx and the consultation members of
// GccAna_Circ2d3Tan.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! GccAna_Circ2d3Tan: circles tangent to three elements (qualified circles and
//! lines, and points). Each native constructor file `GccAna_Circ2d3Tan_<n>.cxx` is
//! one submodule; `circles3` holds the three-circle constructor of the main file.
mod candidates;
mod circ_circ_lin;
mod circ_circ_pnt;
mod circ_lin_lin;
mod circ_lin_pnt;
mod circ_pnt_pnt;
mod circles3;
mod circles3_systems;
mod lin_lin_lin;
mod lin_lin_pnt;
mod lin_pnt_pnt;
mod pnt_pnt_pnt;

use super::solutions::{Slots, Tangencies, check_solution, circle_at};
pub(super) use super::solutions::{line_qualifier, line_tangency};
use crate::gcc_ent::{Position, QualifiedLin};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Direction2, Line2, Point2};

/// GccAna_Circ2d3Tan: the circles tangent to three elements among qualified
/// circles, qualified lines and points, in the native solution order. Each
/// constructor names its argument kinds in order (`circ_circ_lin`...).
/// ```
/// use rustycad_tkgeomalgo::gcc_ana::Circ2d3Tan;
/// use rustycad_tkmath::gp::Point2;
/// let circ = Circ2d3Tan::pnt_pnt_pnt(
///     Point2::new(1.0, 0.0), Point2::new(0.0, 1.0), Point2::new(-1.0, 0.0), 1e-7)?;
/// assert_eq!(circ.nb_solutions(), 1);
/// assert!((circ.this_solution(1)?.radius() - 1.0).abs() < 1e-12);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Debug)]
pub struct Circ2d3Tan {
    well_done: bool,
    nbr_sol: i32,
    cirsol: Slots<Circle2>,
    qualifier1: Slots<Position>,
    qualifier2: Slots<Position>,
    qualifier3: Slots<Position>,
    tangency1: Tangencies,
    tangency2: Tangencies,
    tangency3: Tangencies,
}

/// The argument a tangency point lies on, for its parameter (`pararg<i>`).
#[derive(Clone, Copy)]
pub(super) enum Argument {
    Circle(Circle2),
    Line(Line2),
    /// A point argument: its parameter is 0.
    Point,
}

const NAME: &str = "GccAna_Circ2d3Tan";

impl Circ2d3Tan {
    fn empty(size: usize) -> Self {
        Self {
            well_done: false,
            nbr_sol: 0,
            cirsol: Slots::new(size),
            qualifier1: Slots::new(size),
            qualifier2: Slots::new(size),
            qualifier3: Slots::new(size),
            tangency1: Tangencies::new(size),
            tangency2: Tangencies::new(size),
            tangency3: Tangencies::new(size),
        }
    }

    fn tangency_mut(&mut self, which: usize) -> &mut Tangencies {
        match which {
            1 => &mut self.tangency1,
            2 => &mut self.tangency2,
            _ => &mut self.tangency3,
        }
    }

    fn qualifier_mut(&mut self, which: usize) -> &mut Slots<Position> {
        match which {
            1 => &mut self.qualifier1,
            2 => &mut self.qualifier2,
            _ => &mut self.qualifier3,
        }
    }

    // NbrSol++; cirsol(NbrSol) = gp_Circ2d(gp_Ax2d(center, dirx), radius).
    fn push_circle(&mut self, center: Point2, radius: f64) -> Result<i32, Failure> {
        self.nbr_sol += 1;
        self.cirsol.set(self.nbr_sol, circle_at(center, radius)?)?;
        Ok(self.nbr_sol)
    }

    fn set_qualifier(
        &mut self,
        which: usize,
        index: i32,
        position: Position,
    ) -> Result<(), Failure> {
        self.qualifier_mut(which).set(index, position)
    }

    // pnttg<which>sol(index) = point, par<which>sol = its parameter on the
    // solution, pararg<which> = its parameter on the argument, TheSame = 0.
    fn touch(
        &mut self,
        which: usize,
        index: i32,
        point: Point2,
        argument: Argument,
    ) -> Result<(), Failure> {
        let par_sol: f64 = elclib::parameter(&self.cirsol.get(index)?, point)?;
        let par_arg: f64 = match argument {
            Argument::Circle(circle) => elclib::parameter(&circle, point)?,
            Argument::Line(line) => elclib::parameter(&line, point)?,
            Argument::Point => 0.0,
        };
        self.tangency_mut(which)
            .record(index, point, par_sol, par_arg)
    }

    // TheSame<which>(index) = 1: the solution is the argument circle itself.
    fn mark_same(&mut self, which: usize, index: i32) -> Result<(), Failure> {
        self.tangency_mut(which).same.set(index, 1)
    }

    // The tangency on a line: the foot of the normal from the centre, found as
    // native does through the normal direction signed by the side of the centre.
    fn touch_line(&mut self, which: usize, index: i32, line: Line2) -> Result<(), Failure> {
        let solution: Circle2 = self.cirsol.get(index)?;
        let point: Point2 = line_tangency(line, solution.location(), solution.radius())?;
        self.touch(which, index, point, Argument::Line(line))
    }

    /// IsDone: true when the construction ran to its end; `circ.is_done()`.
    pub fn is_done(&self) -> bool {
        self.well_done
    }

    /// NbSolutions (no WellDone test natively); `circ.nb_solutions()`.
    pub fn nb_solutions(&self) -> i32 {
        self.nbr_sol
    }

    /// ThisSolution(Index); `circ.this_solution(1)?`.
    pub fn this_solution(&self, index: i32) -> Result<Circle2, Failure> {
        check_solution(
            self.well_done,
            self.nbr_sol,
            index,
            "GccAna_Circ2d3Tan::ThisSolution",
        )?;
        self.cirsol.get(index)
    }

    /// WhichQualifier(Index, Qualif1, Qualif2, Qualif3): the positions of solution
    /// `index` relative to the three arguments; `let (q1, q2, q3) = circ.which_qualifier(1)?;`.
    pub fn which_qualifier(&self, index: i32) -> Result<(Position, Position, Position), Failure> {
        let member = "GccAna_Circ2d3Tan::WhichQualifier";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        Ok((
            self.qualifier1.get(index)?,
            self.qualifier2.get(index)?,
            self.qualifier3.get(index)?,
        ))
    }

    /// Tangency1(Index, ParSol, ParArg, PntSol); StdFail_NotDone when the solution
    /// is the first argument itself; `let (par_sol, par_arg, point) = circ.tangency1(1)?;`.
    pub fn tangency1(&self, index: i32) -> Result<(f64, f64, Point2), Failure> {
        let member = "GccAna_Circ2d3Tan::Tangency1";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        self.tangency1.checked(index, member)
    }

    /// Tangency2(Index, ParSol, ParArg, PntSol); `circ.tangency2(1)?`.
    pub fn tangency2(&self, index: i32) -> Result<(f64, f64, Point2), Failure> {
        let member = "GccAna_Circ2d3Tan::Tangency2";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        self.tangency2.checked(index, member)
    }

    /// Tangency3(Index, ParSol, ParArg, PntSol); `circ.tangency3(1)?`.
    pub fn tangency3(&self, index: i32) -> Result<(f64, f64, Point2), Failure> {
        let member = "GccAna_Circ2d3Tan::Tangency3";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        self.tangency3.checked(index, member)
    }

    /// IsTheSame1(Index): whether solution `index` is the first argument;
    /// `circ.is_the_same1(1)?`.
    pub fn is_the_same1(&self, index: i32) -> Result<bool, Failure> {
        let member = "GccAna_Circ2d3Tan::IsTheSame1";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        self.tangency1.is_the_same(index)
    }

    /// IsTheSame2(Index); `circ.is_the_same2(1)?`.
    pub fn is_the_same2(&self, index: i32) -> Result<bool, Failure> {
        let member = "GccAna_Circ2d3Tan::IsTheSame2";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        self.tangency2.is_the_same(index)
    }

    /// IsTheSame3(Index); `circ.is_the_same3(1)?`.
    pub fn is_the_same3(&self, index: i32) -> Result<bool, Failure> {
        let member = "GccAna_Circ2d3Tan::IsTheSame3";
        check_solution(self.well_done, self.nbr_sol, index, member)?;
        self.tangency3.is_the_same(index)
    }
}

// The side of `center` relative to the line (origin, direction):
// (ox - cx) * (-dy) + (oy - cy) * dx, as the native qualifier tests write it.
pub(super) fn side_of(line: Line2, center: Point2) -> f64 {
    let (origin, dir) = (line.location(), line.direction());
    ((origin.x() - center.x()) * (-dir.y())) + ((origin.y() - center.y()) * (dir.x()))
}

// Dir(-dy, dx): the normal of a line direction, renormalized as gp_Dir2d does.
pub(super) fn normal_of(line: Line2) -> Result<Direction2, Failure> {
    let dir: Direction2 = line.direction();
    Direction2::new(-dir.y(), dir.x())
}

// Check the qualifier of every argument (GccEnt_BadQualifier), as each native
// constructor opens; lines accept no GccEnt_enclosing.
pub(super) fn check_line_qualifier(qualified: QualifiedLin) -> Result<(), Failure> {
    if qualified.is_enclosed() || qualified.is_outside() || qualified.is_unqualified() {
        return Ok(());
    }
    Err(crate::gcc_ent::bad_qualifier(&format!(
        "{NAME}: line qualifier {:?}; expected enclosed, outside or unqualified",
        qualified.qualifier()
    )))
}
