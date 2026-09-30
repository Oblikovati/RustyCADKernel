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
// Rust translation of Geom2dGcc_Circ2d2TanOnGeo.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dGcc_Circ2d2TanOnGeo`: circles tangent to two lines, circles or points
//! with their centre on a curve: the bisecting loci of the two arguments
//! (GccAna) intersected with the curve (Geom2dInt).
mod circ_circ;
mod circ_lin;
mod lin_lin;
mod points;

use super::circle_solutions::CircleSolutions;
use super::domains::{clamped_domain, closed_domain, conic_domain, offset_curve};
use crate::gcc_ana::solutions::{Slots, circle_at};
use crate::gcc_ent::Position;
use crate::gcc_int::{Bisec, IType};
use crate::geom2d_int::TheIntConicCurveOfGInter;
use crate::int_res2d::Domain;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dOffsetCurve;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Circle2, Direction2, Line2, Point2, Xy};
use std::f64::consts::PI;

/// The largest number of solutions (`aNbSolMAX`).
const NB_SOL_MAX: i32 = 8;

/// `Geom2dGcc_Circ2d2TanOnGeo`: the circles tangent to two qualified lines or
/// circles (or through points) whose centres are the intersections of the curve
/// `OnCurv` with the bisecting loci of the arguments, at most 8.
/// ```
/// use rustycad_tkgeomalgo::geom2d_gcc::Circ2d2TanOnGeo;
/// use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
/// use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::{Direction2, Point2};
/// let line: Handle<dyn Geom2dCurve> =
///     Handle::new(Geom2dLine::from_point(Point2::new(0.0, -5.0), Direction2::new(0.0, 1.0)?)).cast();
/// let on = Geom2dAdaptorCurve::with_range(&line, 0.0, 10.0)?;
/// let circ = Circ2d2TanOnGeo::pnt_pnt(Point2::new(-1.0, 0.0), Point2::new(1.0, 2.0), &on, 1e-9)?;
/// assert_eq!(circ.nb_solutions(), 1);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Debug)]
pub struct Circ2d2TanOnGeo {
    solutions: CircleSolutions,
    pntcen: Slots<Point2>,
    parcen3: Slots<f64>,
}

// The curve `OnCurv` as an offset of 0 (`Adaptor2d_OffsetCurve(HCu2, 0.)`) and its
// clamped domain.
fn on_curve_of(
    on_curv: &Geom2dAdaptorCurve,
    tol: f64,
) -> Result<(Adaptor2dOffsetCurve, Domain), Failure> {
    let c2 = offset_curve(on_curv, 0.0)?;
    let d2: Domain = clamped_domain(&c2, tol)?;
    Ok((c2, d2))
}

// `Intp.Perform(Sol, D1, C2, D2, Tol1, Tol2)` for a bisecting locus of a kind in
// `kinds`, on its native domain: a whole circle or ellipse, the hyperbola on
// [-4, 4], the parabola on [-40, 40], the whole line; Standard_ConstructionError
// for another kind, as the native `default` case.
fn perform_bisector(
    intp: &mut TheIntConicCurveOfGInter,
    sol: &Handle<dyn Bisec>,
    kinds: &[IType],
    (c2, d2): (&Adaptor2dOffsetCurve, &Domain),
    (tol1, tol2): (f64, f64),
) -> Result<(), Failure> {
    let bisec = sol
        .get()
        .ok_or_else(|| super::null_object("GccInt_Bisec::ArcType"))?;
    let kind: IType = bisec.arc_type();
    if !kinds.contains(&kind) {
        return Err(Failure::with_type(
            "Standard_ConstructionError",
            &format!(
                "Geom2dGcc_Circ2d2TanOnGeo: bisecting locus of kind {kind:?}; expected one of {kinds:?}"
            ),
            "",
        ));
    }
    match kind {
        IType::Cir => {
            let circ = bisec.circle()?;
            intp.perform_circle(
                circ,
                &closed_domain(&circ, 2.0 * PI, (tol1, tol2))?,
                c2,
                d2,
                tol1,
                tol2,
            )
        }
        IType::Ell => {
            let elips = bisec.ellipse()?;
            intp.perform_ellipse(
                elips,
                &closed_domain(&elips, 2.0 * PI, (tol1, tol2))?,
                c2,
                d2,
                tol1,
                tol2,
            )
        }
        IType::Hpr => {
            let hypr = bisec.hyperbola()?;
            intp.perform_hyperbola(
                hypr,
                &conic_domain(&hypr, (-4.0, tol1), (4.0, tol2)),
                c2,
                d2,
                tol1,
                tol2,
            )
        }
        IType::Par => {
            let parab = bisec.parabola()?;
            intp.perform_parabola(
                parab,
                &conic_domain(&parab, (-40.0, tol1), (40.0, tol1)),
                c2,
                d2,
                tol1,
                tol2,
            )
        }
        _ => intp.perform_line(bisec.line()?, &Domain::new(), c2, d2, tol1, tol2),
    }
}

// `(((origin.X - Center.X) * (-dir.Y)) + ((origin.Y - Center.Y) * dir.X))`: the side
// of a centre relative to a line (<= 0 on the left).
fn line_side_value(line: &Line2, center: Point2) -> f64 {
    let (origin, dir) = (line.location(), line.direction());
    ((origin.x() - center.x()) * (-dir.y())) + ((origin.y() - center.y()) * dir.x())
}

// The qualifier of an unqualified line from the direction to its origin, else the
// given one; `dc` is gp_Dir2d(origin - Center), built first as natively.
fn line_qualifier(given: Position, dc: Direction2, normal: Direction2) -> Position {
    if given != Position::Unqualified {
        return given;
    }
    if dc.dot(normal) > 0.0 {
        Position::Outside
    } else {
        Position::Enclosed
    }
}

// The tangency point on a line of a circle of radius `radius` about `center`:
// dc = Dir(origin - Center); sign = dc . Dir(-dy, dx); Center + R Dir(sign (-dy, dx)).
fn line_touch(line: &Line2, center: Point2, radius: f64) -> Result<Point2, Failure> {
    let dir = line.direction();
    let dc = Direction2::from_xy(line.location().xy() - center.xy())?;
    let sign: f64 = dc.dot(Direction2::new(-dir.y(), dir.x())?);
    let dc = Direction2::from_xy(sign * Xy::new(-dir.y(), dir.x()))?;
    Ok(Point2::from(center.xy() + radius * dc.xy()))
}

// The tangency point on a circle of centre `c` seen from `center`:
// Center + R Dir(c - Center).
fn circle_touch(c: Point2, center: Point2, radius: f64) -> Result<Point2, Failure> {
    let dc = Direction2::from_xy(c.xy() - center.xy())?;
    Ok(Point2::from(center.xy() + radius * dc.xy()))
}

impl Circ2d2TanOnGeo {
    fn empty() -> Self {
        Self {
            solutions: CircleSolutions::new(NB_SOL_MAX as usize, 2),
            pntcen: Slots::new(NB_SOL_MAX as usize),
            parcen3: Slots::new(NB_SOL_MAX as usize),
        }
    }

    // Opens solution NbrSol + 1: its circle, centre and centre parameter.
    fn open(
        &mut self,
        center: Point2,
        radius: f64,
        parcen3: f64,
    ) -> Result<(i32, Circle2), Failure> {
        let s = &mut self.solutions;
        s.nbr_sol += 1;
        let index: i32 = s.nbr_sol;
        let circle: Circle2 = circle_at(center, radius)?;
        s.cirsol.set(index, circle)?;
        self.pntcen.set(index, center)?;
        self.parcen3.set(index, parcen3)?;
        Ok((index, circle))
    }

    // Records the tangency of argument `arg` at `point` (ParArg `par_arg`) on the
    // solution circle `circle`.
    fn touch(
        &mut self,
        arg: usize,
        index: i32,
        circle: &Circle2,
        point: Point2,
        par_arg: f64,
    ) -> Result<(), Failure> {
        let par_sol: f64 = elclib::parameter(circle, point)?;
        self.solutions
            .store_tangency(arg, index, false, (par_sol, par_arg, point))
    }

    /// `IsDone()`; `circ.is_done()`.
    pub fn is_done(&self) -> bool {
        self.solutions.well_done
    }

    /// `NbSolutions()`; `circ.nb_solutions()`.
    pub fn nb_solutions(&self) -> i32 {
        self.solutions.nbr_sol
    }

    /// `ThisSolution(Index)`; `circ.this_solution(1)?`.
    pub fn this_solution(&self, index: i32) -> Result<Circle2, Failure> {
        self.solutions
            .solution(index, "Geom2dGcc_Circ2d2TanOnGeo::ThisSolution")
    }

    /// `WhichQualifier(Index, Qualif1, Qualif2)`; `let (q1, q2) = circ.which_qualifier(1)?;`.
    pub fn which_qualifier(&self, index: i32) -> Result<(Position, Position), Failure> {
        let s = &self.solutions;
        s.check(index, "Geom2dGcc_Circ2d2TanOnGeo::WhichQualifier")?;
        Ok((s.qualifier_of(0, index)?, s.qualifier_of(1, index)?))
    }

    /// `Tangency1(Index, ParSol, ParArg, PntSol)`: StdFail_NotDone for a solution
    /// identical to the argument; `let (par_sol, par_arg, point) = circ.tangency1(1)?;`.
    pub fn tangency1(&self, index: i32) -> Result<(f64, f64, Point2), Failure> {
        self.solutions
            .tangency_of(0, index, "Geom2dGcc_Circ2d2TanOnGeo::Tangency1")
    }

    /// `Tangency2(Index, ParSol, ParArg, PntSol)`; `circ.tangency2(1)?`.
    pub fn tangency2(&self, index: i32) -> Result<(f64, f64, Point2), Failure> {
        self.solutions
            .tangency_of(1, index, "Geom2dGcc_Circ2d2TanOnGeo::Tangency2")
    }

    /// `CenterOn3(Index, ParArg, PntSol)`: the centre and its parameter on `OnCurv`;
    /// `let (par_arg, point) = circ.center_on3(1)?;`.
    pub fn center_on3(&self, index: i32) -> Result<(f64, Point2), Failure> {
        self.solutions
            .check(index, "Geom2dGcc_Circ2d2TanOnGeo::CenterOn3")?;
        Ok((self.parcen3.get(index)?, self.pntcen.get(index)?))
    }

    /// `IsTheSame1(Index)`; `circ.is_the_same1(1)?`.
    pub fn is_the_same1(&self, index: i32) -> Result<bool, Failure> {
        self.solutions
            .same_of(0, index, "Geom2dGcc_Circ2d2TanOnGeo::IsTheSame1")
    }

    /// `IsTheSame2(Index)`; `circ.is_the_same2(1)?`.
    pub fn is_the_same2(&self, index: i32) -> Result<bool, Failure> {
        self.solutions
            .same_of(1, index, "Geom2dGcc_Circ2d2TanOnGeo::IsTheSame2")
    }
}
