// Copyright (c) 1995-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/AppParCurves/AppParCurves_LeastSquare.gxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AppParCurves_LeastSquare` (AppParCurves_LeastSquare.gxx): the least-squares
//! fit of the poles of a MultiCurve (Bezier or B-spline) to a MultiLine, with the
//! extremity constraints eliminated from the unknowns.
mod errors;
pub use errors::FitErrors;
mod init;
mod normal_equations;
mod perform;
mod solve_blocks;

use super::constraint::Constraint;
use super::line_tool::LineTool;
use super::multi_bsp_curve::MultiBSpCurve;
use super::multi_curve::MultiCurve;
use crate::native_failure::{array_failure, failure};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tkmath::math::{IntegerVector, Matrix, Vector};

/// `AppParCurves_LeastSquare`: the native generic over `MultiLine`/`ToolLine`; the line
/// is only read by the constructors, so the struct is not generic and the
/// constructors take any [`LineTool`].
///
/// The native matrices start uninitialized (0 here). With curvature constraints,
/// `Perform(Parameters)` leaves pole rows unassigned (the native documentation says
/// the case is not treated there), and the lambdas are unassigned without a
/// tangency constraint: the port reads 0 where the native code reads indeterminate
/// values.
///
/// ```ignore
/// let mut fit = LeastSquare::new(&line, 1, 10, Constraint::PassPoint, Constraint::PassPoint, &parameters, 4)?;
/// assert!(fit.is_done());
/// let curve = fit.bezier_value()?;
/// ```
pub struct LeastSquare {
    first_constraint: Constraint,
    last_constraint: Constraint,
    scu: MultiBSpCurve,
    knots: Option<Array1<'static, f64>>,
    mults: Option<Array1<'static, i32>>,
    mypoles: Matrix<'static>,
    a: Matrix<'static>,
    da: Matrix<'static>,
    b2: Matrix<'static>,
    mypoints: Matrix<'static>,
    vflatknots: Vector<'static>,
    vec1t: Vector<'static>,
    vec1c: Vector<'static>,
    vec2t: Vector<'static>,
    vec2c: Vector<'static>,
    the_error: Matrix<'static>,
    myindex: IntegerVector<'static>,
    lambda1: f64,
    lambda2: f64,
    first_p: i32,
    last_p: i32,
    nlignes: i32,
    ninc: i32,
    na: i32,
    myfirstp: i32,
    mylastp: i32,
    resinit: i32,
    resfin: i32,
    nb_p2d: i32,
    nb_p: i32,
    nbpoles: i32,
    deg: i32,
    done: bool,
    iscalculated: bool,
    isready: bool,
}

// A matrix with the native bounds; bad bounds are Standard_RangeError.
fn matrix(rows: [i32; 2], cols: [i32; 2], value: f64) -> Result<Matrix<'static>, Failure> {
    Matrix::filled(rows[0], rows[1], cols[0], cols[1], value)
        .map_err(|error| array_failure(error, "AppParCurves_LeastSquare"))
}

/// `TheFirstPoint(FirstCons, FirstPoint)`: the first fitted point (after a constrained
/// first point); `the_first_point(Constraint::PassPoint, 1) == 2`.
pub fn the_first_point(first_cons: Constraint, first_point: i32) -> i32 {
    if first_cons == Constraint::NoConstraint {
        first_point
    } else {
        first_point + 1
    }
}

/// `TheLastPoint(LastCons, LastPoint)`: the last fitted point;
/// `the_last_point(Constraint::PassPoint, 9) == 8`.
pub fn the_last_point(last_cons: Constraint, last_point: i32) -> i32 {
    if last_cons == Constraint::NoConstraint {
        last_point
    } else {
        last_point - 1
    }
}

/// `NbBColumns(SSP)`: `3 NbP3d + 2 NbP2d`, the columns of the right-hand side;
/// `nb_b_columns(&line)`.
pub fn nb_b_columns(ssp: &impl LineTool) -> i32 {
    ssp.nb_p3d() * 3 + ssp.nb_p2d() * 2
}

impl LeastSquare {
    // The member initializers shared by the four constructors.
    fn allocate(
        ssp: &impl LineTool,
        [first_point, last_point]: [i32; 2],
        [first_cons, last_cons]: [Constraint; 2],
        nb_pol: i32,
        flat_length: i32,
    ) -> Result<Self, Failure> {
        let columns: i32 = nb_b_columns(ssp);
        let first_row: i32 = the_first_point(first_cons, first_point);
        let b2_rows: [i32; 2] = [
            first_row,
            first_row.max(the_last_point(last_cons, last_point)),
        ];
        Ok(Self {
            first_constraint: first_cons,
            last_constraint: last_cons,
            scu: MultiBSpCurve::with_nb_poles(nb_pol)?,
            knots: None,
            mults: None,
            mypoles: matrix([1, nb_pol], [1, columns], 0.)?,
            a: matrix([first_point, last_point], [1, nb_pol], 0.)?,
            da: matrix([first_point, last_point], [1, nb_pol], 0.)?,
            b2: matrix(b2_rows, [1, columns], 0.)?,
            mypoints: matrix([first_point, last_point], [1, columns], 0.)?,
            vflatknots: Vector::filled(1, flat_length, 0.)?,
            vec1t: Vector::filled(1, columns, 0.)?,
            vec1c: Vector::filled(1, columns, 0.)?,
            vec2t: Vector::filled(1, columns, 0.)?,
            vec2c: Vector::filled(1, columns, 0.)?,
            the_error: matrix(
                [first_point, last_point],
                [1, ssp.nb_p3d() + ssp.nb_p2d()],
                0.,
            )?,
            myindex: IntegerVector::filled(first_point, last_point, 0)?,
            lambda1: 0.,
            lambda2: 0.,
            first_p: 0,
            last_p: 0,
            nlignes: 0,
            ninc: 0,
            na: 0,
            myfirstp: 0,
            mylastp: 0,
            resinit: 0,
            resfin: 0,
            nb_p2d: 0,
            nb_p: 0,
            nbpoles: nb_pol,
            deg: 0,
            done: false,
            iscalculated: false,
            isready: false,
        })
    }

    /// The Bezier constructor with parameters: initializes and performs;
    /// `LeastSquare::new(&line, 1, 10, Constraint::PassPoint, Constraint::NoConstraint, &u, 4)?`.
    pub fn new(
        ssp: &impl LineTool,
        first_point: i32,
        last_point: i32,
        first_cons: Constraint,
        last_cons: Constraint,
        parameters: &Vector<'_>,
        nb_pol: i32,
    ) -> Result<Self, Failure> {
        let mut fit: Self =
            Self::without_parameters(ssp, first_point, last_point, first_cons, last_cons, nb_pol)?;
        fit.perform(parameters)?;
        Ok(fit)
    }

    /// The Bezier constructor without parameters: initializes only;
    /// `LeastSquare::without_parameters(&line, 1, 10, c1, c2, 4)?`.
    pub fn without_parameters(
        ssp: &impl LineTool,
        first_point: i32,
        last_point: i32,
        first_cons: Constraint,
        last_cons: Constraint,
        nb_pol: i32,
    ) -> Result<Self, Failure> {
        let mut fit: Self = Self::allocate(
            ssp,
            [first_point, last_point],
            [first_cons, last_cons],
            nb_pol,
            1,
        )?;
        fit.init(ssp, first_point, last_point)?;
        Ok(fit)
    }

    /// The B-spline constructor with parameters (`Deg = Sum(Mults) - NbPol - 1`);
    /// `LeastSquare::with_knots(&line, &knots, &mults, 1, 10, c1, c2, &u, 6)?`.
    #[allow(clippy::too_many_arguments)] // Mirrors the native nine-argument constructor.
    pub fn with_knots(
        ssp: &impl LineTool,
        knots: &Array1<'_, f64>,
        mults: &Array1<'_, i32>,
        first_point: i32,
        last_point: i32,
        first_cons: Constraint,
        last_cons: Constraint,
        parameters: &Vector<'_>,
        nb_pol: i32,
    ) -> Result<Self, Failure> {
        let mut fit: Self = Self::with_knots_without_parameters(
            ssp,
            knots,
            mults,
            first_point,
            last_point,
            first_cons,
            last_cons,
            nb_pol,
        )?;
        fit.perform(parameters)?;
        Ok(fit)
    }

    /// The B-spline constructor without parameters: initializes only;
    /// `LeastSquare::with_knots_without_parameters(&line, &knots, &mults, 1, 10, c1, c2, 6)?`.
    #[allow(clippy::too_many_arguments)] // Mirrors the native eight-argument constructor.
    pub fn with_knots_without_parameters(
        ssp: &impl LineTool,
        knots: &Array1<'_, f64>,
        mults: &Array1<'_, i32>,
        first_point: i32,
        last_point: i32,
        first_cons: Constraint,
        last_cons: Constraint,
        nb_pol: i32,
    ) -> Result<Self, Failure> {
        let flat_length: i32 = mults.iter().sum();
        let mut fit: Self = Self::allocate(
            ssp,
            [first_point, last_point],
            [first_cons, last_cons],
            nb_pol,
            flat_length,
        )?;
        fit.knots = Some(copy(knots));
        fit.mults = Some(copy(mults));
        fit.scu.set_knots(knots);
        fit.scu.set_multiplicities(mults);
        fit.init(ssp, first_point, last_point)?;
        Ok(fit)
    }

    /// `IsDone()`; `fit.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }

    /// `BezierValue()`: the fitted Bezier curves, sharing their pole array with the
    /// fit; Standard_NoSuchObject for a B-spline fit, StdFail_NotDone before a
    /// successful Perform; `fit.bezier_value()?`.
    pub fn bezier_value(&mut self) -> Result<MultiCurve, Failure> {
        if self.knots.is_some() {
            return Err(failure(
                "Standard_NoSuchObject",
                "AppParCurves_LeastSquare::BezierValue: a B-spline fit",
            ));
        }
        Ok(self.bspline_value()?.as_multi_curve())
    }

    /// `BSplineValue()`: stores the fitted poles `min(resinit, 2)..=max(resfin,
    /// NbPoles - 1)` into the result curve and returns it; StdFail_NotDone before a
    /// successful Perform; `fit.bspline_value()?`.
    pub fn bspline_value(&mut self) -> Result<&MultiBSpCurve, Failure> {
        self.check_done("BSplineValue")?;
        let (ideb, ifin) = (self.resinit.min(2), self.resfin.max(self.nbpoles - 1));
        for i in ideb..=ifin {
            let pole = super::multi_point::MultiPoint::with_counts(self.nb_p, self.nb_p2d)?;
            let mut j2: i32 = 1;
            for j in 1..=self.nb_p {
                let row = |k: i32| self.mypoles.value(i, k).copied();
                pole.set_point(j, Point3::new(row(j2)?, row(j2 + 1)?, row(j2 + 2)?))?;
                j2 += 3;
            }
            for j in self.nb_p + 1..=self.nb_p + self.nb_p2d {
                let row = |k: i32| self.mypoles.value(i, k).copied();
                pole.set_point2d(j, Point2::new(row(j2)?, row(j2 + 1)?))?;
                j2 += 2;
            }
            self.scu.set_value(i, &pole)?;
        }
        Ok(&self.scu)
    }

    fn check_done(&self, member: &str) -> Result<(), Failure> {
        if !self.done {
            return Err(failure(
                "StdFail_NotDone",
                &format!("AppParCurves_LeastSquare::{member}"),
            ));
        }
        Ok(())
    }

    /// `FunctionMatrix()`: the Bernstein or B-spline matrix; StdFail_NotDone before a
    /// successful Perform; `fit.function_matrix()?`.
    pub fn function_matrix(&self) -> Result<&Matrix<'static>, Failure> {
        self.check_done("FunctionMatrix")?;
        Ok(&self.a)
    }

    /// `DerivativeFunctionMatrix()`: its derivative; `fit.derivative_function_matrix()?`.
    pub fn derivative_function_matrix(&self) -> Result<&Matrix<'static>, Failure> {
        self.check_done("DerivativeFunctionMatrix")?;
        Ok(&self.da)
    }

    /// `Distance()`: the point-curve distances (square roots taken in place, once
    /// after each Perform); `fit.distance()`.
    pub fn distance(&mut self) -> &Matrix<'static> {
        if !self.iscalculated {
            for i in self.myfirstp..=self.mylastp {
                for j in 1..=self.nb_p + self.nb_p2d {
                    let value: &mut f64 = &mut self.the_error[(i, j)];
                    *value = value.sqrt();
                }
            }
            self.iscalculated = true;
        }
        &self.the_error
    }

    /// `FirstLambda()`: `(P2 - P1) / V1` of a tangency first point (0 before any, an
    /// indeterminate value natively); `fit.first_lambda()`.
    pub fn first_lambda(&self) -> f64 {
        self.lambda1
    }

    /// `LastLambda()`; `fit.last_lambda()`.
    pub fn last_lambda(&self) -> f64 {
        self.lambda2
    }

    /// `Points()`: the point coordinates, one row per point; `fit.points()`.
    pub fn points(&self) -> &Matrix<'static> {
        &self.mypoints
    }

    /// `Poles()`: the pole coordinates, one row per pole; `fit.poles()`.
    pub fn poles(&self) -> &Matrix<'static> {
        &self.mypoles
    }

    /// `KIndex()`: the span index of each point (0 for a Bezier fit); `fit.k_index()`.
    pub fn k_index(&self) -> &IntegerVector<'static> {
        &self.myindex
    }
}

fn copy<T: Clone + Default>(values: &Array1<'_, T>) -> Array1<'static, T> {
    let mut copy: Array1<'static, T> = Array1::with_size(values.len());
    copy.update_lower_bound(values.lower());
    copy.iter_mut()
        .zip(values.iter())
        .for_each(|(slot, value)| *slot = value.clone());
    copy
}
