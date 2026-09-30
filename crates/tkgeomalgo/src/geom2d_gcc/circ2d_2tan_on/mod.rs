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
// Rust translation of Geom2dGcc_Circ2d2TanOn.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dGcc_Circ2d2TanOn`: circles tangent to two curves (or through points)
//! with their centre on a curve.
mod dispatch;

use super::circle_solutions::{CircleSolutions, TwoTangencySolver};
use super::{Circ2d2TanOnGeo, Circ2d2TanOnIter, QualifiedCurve, point_of};
use crate::gcc_ana;
use crate::gcc_ana::solutions::Slots;
use crate::gcc_ent::Position;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::Geom2dPoint;
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkmath::gp::{Circle2, Point2};

/// `Geom2dGcc_Circ2d2TanOn`: the circles tangent to two qualified curves (or a
/// curve and a point, or through two points) with their centre on `OnCurve`, at
/// most 8: GccAna_Circ2d2TanOn when every curve is a line or a circle,
/// Geom2dGcc_Circ2d2TanOnGeo when only the locus is another curve, else one
/// Geom2dGcc_Circ2d2TanOnIter from the given parameters.
/// ```
/// use rustycad_tkgeomalgo::geom2d_gcc::Circ2d2TanOn;
/// use rustycad_tkg2d::geom2d::{Geom2dCartesianPoint, Geom2dCurve, Geom2dLine, Geom2dPoint};
/// use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::{Direction2, Point2};
/// let line: Handle<dyn Geom2dCurve> =
///     Handle::new(Geom2dLine::from_point(Point2::new(0.0, -5.0), Direction2::new(0.0, 1.0)?)).cast();
/// let on = Geom2dAdaptorCurve::with_curve(&line)?;
/// let point = |x: f64| -> Handle<dyn Geom2dPoint> { Handle::new(Geom2dCartesianPoint::from_pnt2d(Point2::new(x, 0.0))).cast() };
/// let circ = Circ2d2TanOn::with_points(&point(-1.0), &point(1.0), &on, 1e-9)?;
/// assert_eq!(circ.nb_solutions(), 1);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Debug)]
pub struct Circ2d2TanOn {
    solutions: CircleSolutions,
    pntcen: Slots<Point2>,
    parcen3: Slots<f64>,
    invert: bool,
}

// The CenterOn3 member of the solvers whose Results Circ2d2TanOn copies.
trait CenterSource: TwoTangencySolver {
    fn center(&self, index: i32) -> Result<(f64, Point2), Failure>;
}

impl CenterSource for gcc_ana::Circ2d2TanOn {
    fn center(&self, index: i32) -> Result<(f64, Point2), Failure> {
        self.center_on3(index)
    }
}

impl CenterSource for Circ2d2TanOnGeo {
    fn center(&self, index: i32) -> Result<(f64, Point2), Failure> {
        self.center_on3(index)
    }
}

impl Circ2d2TanOn {
    fn empty() -> Self {
        Self {
            solutions: CircleSolutions::new(8, 2),
            pntcen: Slots::new(8),
            parcen3: Slots::new(8),
            invert: false,
        }
    }

    /// `Geom2dGcc_Circ2d2TanOn(Qualified1, Qualified2, OnCurve, Tolerance, Param1,
    /// Param2, ParamOn)`. For two other curves on a line or circle, natively the
    /// iteration on that line or circle runs, then the iteration on the curve runs
    /// too and replaces its result; `Circ2d2TanOn::new(&q1, &q2, &on, 1e-9, [0.0; 3])?`.
    pub fn new(
        qualified1: &QualifiedCurve,
        qualified2: &QualifiedCurve,
        on_curve: &Geom2dAdaptorCurve,
        tolerance: f64,
        params: [f64; 3],
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty();
        if circ.two_analytic([qualified1, qualified2], on_curve, tolerance)? {
            return Ok(circ);
        }
        circ.iter_on_line_or_circle([qualified1, qualified2], on_curve, tolerance, params)?;
        let iter = Circ2d2TanOnIter::curve_curve_on_curve(
            qualified1, qualified2, on_curve, params, tolerance,
        )?;
        circ.store_iter(&iter, true, true)?;
        Ok(circ)
    }

    /// `Geom2dGcc_Circ2d2TanOn(Qualified1, Point, OnCurve, Tolerance, Param1,
    /// ParamOn)`; `Circ2d2TanOn::with_point(&q, &point, &on, 1e-9, [0.0; 2])?`.
    pub fn with_point(
        qualified1: &QualifiedCurve,
        point: &Handle<dyn Geom2dPoint>,
        on_curve: &Geom2dAdaptorCurve,
        tolerance: f64,
        params: [f64; 2],
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty();
        if !circ.one_analytic_point(qualified1, point, on_curve, tolerance)? {
            circ.iter_with_point(qualified1, point_of(point)?, on_curve, tolerance, params)?;
        }
        Ok(circ)
    }

    /// `Geom2dGcc_Circ2d2TanOn(Point1, Point2, OnCurve, Tolerance)`: GccAna on a line
    /// or circle, else Geom2dGcc_Circ2d2TanOnGeo;
    /// `Circ2d2TanOn::with_points(&p1, &p2, &on, 1e-9)?`.
    pub fn with_points(
        point1: &Handle<dyn Geom2dPoint>,
        point2: &Handle<dyn Geom2dPoint>,
        on_curve: &Geom2dAdaptorCurve,
        tolerance: f64,
    ) -> Result<Self, Failure> {
        let mut circ = Self::empty();
        circ.points_on(point1, point2, on_curve, tolerance)?;
        Ok(circ)
    }

    // The single solution of an iteration when it is done: its circle, TheSame
    // flags (the second one only with `same2`), qualifiers (with `qualifiers`) and
    // tangencies (natively of parameters 0).
    fn store_iter(
        &mut self,
        iter: &Circ2d2TanOnIter,
        same2: bool,
        qualifiers: bool,
    ) -> Result<(), Failure> {
        let s = &mut self.solutions;
        s.well_done = iter.is_done();
        if !s.well_done {
            return Ok(());
        }
        s.nbr_sol = 1;
        s.cirsol.set(1, iter.this_solution())?;
        s.tangency[0].same.set(1, i32::from(iter.is_the_same1()?))?;
        if same2 {
            s.tangency[1].same.set(1, i32::from(iter.is_the_same2()?))?;
        }
        if qualifiers {
            let (q1, q2) = iter.which_qualifier()?;
            s.qualifier[0].set(1, q1)?;
            s.qualifier[1].set(1, q2)?;
        }
        let same = [
            s.tangency[0].same.get(1)? != 0,
            s.tangency[1].same.get(1)? != 0,
        ];
        s.store_tangency(0, 1, same[0], iter.tangency1()?)?;
        s.store_tangency(1, 1, same[1], iter.tangency2()?)
    }

    // WellDone, and when done NbrSol, the qualifiers and the Results of a GccAna or
    // geometric solver.
    fn take(
        &mut self,
        done: bool,
        count: impl FnOnce() -> i32,
        source: &impl CenterSource,
    ) -> Result<(), Failure> {
        self.solutions.well_done = done;
        if !done {
            return Ok(());
        }
        self.solutions.nbr_sol = count();
        self.solutions.store_qualifiers(source)?;
        self.results_of(source)
    }

    // `Results(Circ)`: the circles, TheSame flags, qualifiers, tangencies and centres
    // of solutions 1 to NbrSol.
    fn results_of(&mut self, source: &impl CenterSource) -> Result<(), Failure> {
        self.solutions.store_results(source)?;
        self.solutions.store_qualifiers(source)?;
        for j in 1..=self.solutions.nbr_sol {
            let (parcen3, pntcen) = source.center(j)?;
            self.parcen3.set(j, parcen3)?;
            self.pntcen.set(j, pntcen)?;
        }
        Ok(())
    }

    /// `Results(const GccAna_Circ2d2TanOn&)`; `solver.results_ana(&ana)?`.
    pub fn results_ana(&mut self, circ: &gcc_ana::Circ2d2TanOn) -> Result<(), Failure> {
        self.results_of(circ)
    }

    /// `Results(const Geom2dGcc_Circ2d2TanOnGeo&)`; `solver.results_geo(&geo)?`.
    pub fn results_geo(&mut self, circ: &Circ2d2TanOnGeo) -> Result<(), Failure> {
        self.results_of(circ)
    }

    fn stored(&self, arg: usize) -> usize {
        if self.invert { 1 - arg } else { arg }
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
            .solution(index, "Geom2dGcc_Circ2d2TanOn::ThisSolution")
    }

    /// `WhichQualifier(Index, Qualif1, Qualif2)`; `let (q1, q2) = circ.which_qualifier(1)?;`.
    pub fn which_qualifier(&self, index: i32) -> Result<(Position, Position), Failure> {
        let s = &self.solutions;
        s.check(index, "Geom2dGcc_Circ2d2TanOn::WhichQualifier")?;
        Ok((
            s.qualifier_of(self.stored(0), index)?,
            s.qualifier_of(self.stored(1), index)?,
        ))
    }

    /// `Tangency1(Index, ParSol, ParArg, PntSol)`;
    /// `let (par_sol, par_arg, point) = circ.tangency1(1)?;`.
    pub fn tangency1(&self, index: i32) -> Result<(f64, f64, Point2), Failure> {
        self.solutions
            .tangency_of(self.stored(0), index, "Geom2dGcc_Circ2d2TanOn::Tangency1")
    }

    /// `Tangency2(Index, ParSol, ParArg, PntSol)`;
    /// `let (par_sol, par_arg, point) = circ.tangency2(1)?;`.
    pub fn tangency2(&self, index: i32) -> Result<(f64, f64, Point2), Failure> {
        self.solutions
            .tangency_of(self.stored(1), index, "Geom2dGcc_Circ2d2TanOn::Tangency2")
    }

    /// `CenterOn3(Index, ParArg, PntSol)`: the centre and its parameter on the locus
    /// (never written by the iterations: 0 and the origin here);
    /// `let (par_arg, point) = circ.center_on3(1)?;`.
    pub fn center_on3(&self, index: i32) -> Result<(f64, Point2), Failure> {
        self.solutions
            .check(index, "Geom2dGcc_Circ2d2TanOn::CenterOn3")?;
        Ok((self.parcen3.get(index)?, self.pntcen.get(index)?))
    }

    /// `IsTheSame1(Index)`; `circ.is_the_same1(1)?`.
    pub fn is_the_same1(&self, index: i32) -> Result<bool, Failure> {
        self.solutions
            .same_of(self.stored(0), index, "Geom2dGcc_Circ2d2TanOn::IsTheSame1")
    }

    /// `IsTheSame2(Index)`; `circ.is_the_same2(1)?`.
    pub fn is_the_same2(&self, index: i32) -> Result<bool, Failure> {
        self.solutions
            .same_of(self.stored(1), index, "Geom2dGcc_Circ2d2TanOn::IsTheSame2")
    }
}
