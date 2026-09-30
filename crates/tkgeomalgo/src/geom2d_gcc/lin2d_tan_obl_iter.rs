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
// Rust translation of Geom2dGcc_Lin2dTanOblIter.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dGcc_Lin2dTanOblIter`: a line tangent to a curve at a given angle to a
//! line, found from a start parameter.
use super::{CurveTool, FunctionTanObl, QCurve, is_parallel};
use crate::gcc_ana::check_qualifier;
use crate::gcc_ana::solutions::not_done;
use crate::gcc_ent::Position;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::int_ana2d::AnaIntersection;
use rustycad_tkmath::gp::{Direction2, Line2, Point2, Vector2};
use rustycad_tkmath::math::FunctionRoot;
use std::f64::consts::PI;

/// `Geom2dGcc_Lin2dTanOblIter`: the line tangent to a qualified curve that makes the
/// angle `Angle` with `TheLin`, found by math_FunctionRoot from `Param1`, and its
/// intersection with `TheLin`.
/// ```
/// use rustycad_tkgeomalgo::geom2d_gcc::{Lin2dTanOblIter, unqualified};
/// use rustycad_tkg2d::geom2d::{Geom2dCircle, Geom2dCurve};
/// use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkmath::gp::{Axis2d, Direction2, Line2, Point2};
/// let axis = Axis2d::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0)?);
/// let circle: Handle<dyn Geom2dCurve> = Handle::new(Geom2dCircle::from_axis(axis, 1.0, true)?).cast();
/// let q = unqualified(&Geom2dAdaptorCurve::with_curve(&circle)?)?;
/// let x_axis = Line2::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0)?);
/// let lin = Lin2dTanOblIter::new(&q, x_axis, 1.0, 1e-9, std::f64::consts::FRAC_PI_2)?;
/// assert!(lin.is_done());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Debug, Default)]
pub struct Lin2dTanOblIter {
    well_done: bool,
    paral2: bool,
    linsol: Line2,
    qualifier1: Position,
    pnttg1sol: Point2,
    pntint2sol: Point2,
    par1sol: f64,
    par2sol: f64,
    pararg1: f64,
    pararg2: f64,
}

impl Lin2dTanOblIter {
    /// `Geom2dGcc_Lin2dTanOblIter(Qualified1, TheLin, Param1, TolAng, Angle)`: an
    /// angle within `TolAng` of 0 or PI makes the solution parallel to `TheLin`;
    /// GccEnt_BadQualifier for GccEnt_noqualifier.
    /// `Lin2dTanOblIter::new(&q, line, 0.0, 1e-9, 0.5)?`.
    pub fn new(
        qualified1: &QCurve,
        the_lin: Line2,
        param1: f64,
        tol_ang: f64,
        angle: f64,
    ) -> Result<Self, Failure> {
        let mut iter = Self::default();
        check_qualifier(qualified1.qualifier(), "Geom2dGcc_Lin2dTanOblIter")?;
        let cu1 = qualified1.curve();
        let u1: f64 = CurveTool::first_parameter(cu1)?;
        let u2: f64 = CurveTool::last_parameter(cu1)?;
        let the_direction: Direction2 = iter.direction(the_lin.direction(), tol_ang, angle)?;
        let mut func = FunctionTanObl::new(cu1, the_direction);
        let eps: f64 = CurveTool::eps_x(cu1, tol_ang.abs())?;
        let sol = FunctionRoot::bounded(&mut func, param1, eps, u1, u2, 100)?;
        func.raised()?;
        if !sol.is_done() {
            return Ok(iter);
        }
        iter.accept(qualified1, the_lin, the_direction, sol.root()?)?;
        Ok(iter)
    }

    // The direction of the solution: TheLin's direction turned by Angle, with the
    // exact quarter and half turns, and Paral2 for the parallel angles.
    fn direction(
        &mut self,
        dir: Direction2,
        tol_ang: f64,
        angle: f64,
    ) -> Result<Direction2, Failure> {
        let (a, b): (f64, f64) = (dir.x(), dir.y());
        // `if (|Angle| > |TolAng|) ... else Paral2`: a NaN angle is parallel.
        let turned: bool = angle.abs() > tol_ang.abs();
        if !turned {
            self.paral2 = true;
            return Ok(dir);
        }
        if (angle.abs() - PI).abs() <= tol_ang.abs() {
            self.paral2 = true;
            return Ok(dir.reversed());
        }
        if (angle - PI / 2.0).abs() <= tol_ang.abs() {
            return Direction2::new(-b, a);
        }
        if (angle + PI / 2.0).abs() <= tol_ang.abs() {
            return Direction2::new(b, -a);
        }
        Direction2::new(
            a * angle.cos() - b * angle.sin(),
            a * angle.sin() + b * angle.cos(),
        )
    }

    // The tangency at `usol` when the qualifier accepts it, and the intersection
    // with TheLin of a non-parallel solution.
    fn accept(
        &mut self,
        qualified1: &QCurve,
        the_lin: Line2,
        the_direction: Direction2,
        usol: f64,
    ) -> Result<(), Failure> {
        let (origine, vect1, vect2) = CurveTool::d2(qualified1.curve(), usol)?;
        let sign1: f64 = vect1.xy().dot(the_direction.xy());
        let sign2: f64 = vect2.xy().crossed(the_direction.xy());
        if !(qualified1.is_unqualified()
            || (qualified1.is_enclosing() && sign2 <= 0.0)
            || (qualified1.is_outside() && sign1 <= 0.0 && sign2 >= 0.0)
            || (qualified1.is_enclosed() && sign1 >= 0.0 && sign2 >= 0.0))
        {
            return Ok(());
        }
        self.well_done = true;
        self.linsol = Line2::new(origine, the_direction);
        self.pnttg1sol = origine;
        self.qualifier1 = qualified1.qualifier();
        self.pararg1 = usol;
        self.par1sol = 0.0;
        if !self.paral2 {
            self.intersect(the_lin)?;
        }
        Ok(())
    }

    // IntAna2d_AnaIntersection of the solution and TheLin, kept when it is one point.
    fn intersect(&mut self, the_lin: Line2) -> Result<(), Failure> {
        let intp = AnaIntersection::lines(self.linsol, the_lin);
        if !intp.is_done() || intp.is_empty()? || intp.nb_points()? != 1 {
            return Ok(());
        }
        self.pntint2sol = intp.point(1)?.value();
        let along = |line: Line2| -> f64 {
            Vector2::from(line.direction()).dot(Vector2::between(line.location(), self.pntint2sol))
        };
        self.par2sol = along(self.linsol);
        self.pararg2 = along(the_lin);
        Ok(())
    }

    /// `IsDone()`; `lin.is_done()`.
    pub fn is_done(&self) -> bool {
        self.well_done
    }

    /// `ThisSolution()`: StdFail_NotDone when the search failed; `lin.this_solution()?`.
    pub fn this_solution(&self) -> Result<Line2, Failure> {
        self.check_done("Geom2dGcc_Lin2dTanOblIter::ThisSolution")?;
        Ok(self.linsol)
    }

    /// `WhichQualifier(Qualif1)`; `lin.which_qualifier()?`.
    pub fn which_qualifier(&self) -> Result<Position, Failure> {
        self.check_done("Geom2dGcc_Lin2dTanOblIter::WhichQualifier")?;
        Ok(self.qualifier1)
    }

    /// `Tangency1(ParSol, ParArg, PntSol)`: the tangency on the curve;
    /// `let (par_sol, par_arg, point) = lin.tangency1()?;`.
    pub fn tangency1(&self) -> Result<(f64, f64, Point2), Failure> {
        self.check_done("Geom2dGcc_Lin2dTanOblIter::Tangency1")?;
        Ok((self.par1sol, self.pararg1, self.pnttg1sol))
    }

    /// `Intersection2(ParSol, ParArg, PntSol)`: the intersection with `TheLin`;
    /// Geom2dGcc_IsParallel for a parallel solution.
    /// `let (par_sol, par_arg, point) = lin.intersection2()?;`.
    pub fn intersection2(&self) -> Result<(f64, f64, Point2), Failure> {
        self.check_done("Geom2dGcc_Lin2dTanOblIter::Intersection2")?;
        if self.paral2 {
            return Err(is_parallel("Geom2dGcc_Lin2dTanOblIter::Intersection2"));
        }
        Ok((self.par2sol, self.pararg2, self.pntint2sol))
    }

    /// `IsParallel2()`: whether the solution is parallel to `TheLin`, set even when
    /// the search failed; `lin.is_parallel2()`.
    pub fn is_parallel2(&self) -> bool {
        self.paral2
    }

    fn check_done(&self, member: &str) -> Result<(), Failure> {
        if !self.well_done {
            return Err(not_done(member));
        }
        Ok(())
    }
}
