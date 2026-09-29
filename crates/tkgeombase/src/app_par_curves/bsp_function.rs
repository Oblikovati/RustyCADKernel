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
// Rust translation of ModelingData/TKGeomBase/AppParCurves/AppParCurves_BSpFunction.gxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AppParCurves_BSpFunction` (AppParCurves_BSpFunction.gxx): F(u) and its gradient
//! for the B-spline fit of a MultiLine, for the parameter optimizer.
use super::Constraints;
use super::constraint::Constraint;
use super::fit_targets::{FitTargets, impl_par_function};
use super::function::point_constraint;
use super::least_square::LeastSquare;
use super::line_tool::LineTool;
use super::multi_bsp_curve::MultiBSpCurve;
use crate::native_failure::array_failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::{IntegerVector, Matrix, Vector};

/// `AppParCurves_BSpFunction`: generic over the MultiLine `L`.
///
/// The constrained branch is empty natively: with a constraint on an inner point,
/// Value leaves F unassigned and the errors keep their previous values (0 before
/// any evaluation here, indeterminate natively). The fit runs with the stored
/// lambdas (`SetFirstLambda`, `SetLastLambda`; 0 until set here, indeterminate
/// natively).
///
/// ```ignore
/// let mut f = BSpFunction::new(&line, 1, 10, &constraints, &u, &knots, &mults, 6)?;
/// let mut value = 0.0;
/// assert!(f.evaluate(&u, &mut value)?);
/// ```
pub struct BSpFunction<L: LineTool + Clone> {
    done: bool,
    multi_bsp_curve: MultiBSpCurve,
    parameters: Vector<'static>,
    fval: f64,
    val_grad_f: Vector<'static>,
    my_f: Matrix<'static>,
    least_square: LeastSquare,
    targets: FitTargets,
    nb_p: i32,
    marker: std::marker::PhantomData<L>,
    err3d: f64,
    err2d: f64,
    lambda1: f64,
    lambda2: f64,
    failure: Option<Failure>,
}

fn matrix(rows: [i32; 2], cols: i32) -> Result<Matrix<'static>, Failure> {
    Matrix::filled(rows[0], rows[1], 1, cols, 0.)
        .map_err(|error| array_failure(error, "AppParCurves_BSpFunction"))
}

impl<L: LineTool + Clone> BSpFunction<L> {
    /// `AppParCurves_BSpFunction(SSP, FirstPoint, LastPoint, TheConstraints, Parameters,
    /// Knots, Mults, NbPol)`; `BSpFunction::new(&line, 1, 10, &c, &u, &knots, &mults, 6)?`.
    #[allow(clippy::too_many_arguments)] // Mirrors the native eight-argument constructor.
    pub fn new(
        ssp: &L,
        first_point: i32,
        last_point: i32,
        constraints: &Constraints,
        parameters: &Vector<'_>,
        knots: &Array1<'_, f64>,
        mults: &Array1<'_, i32>,
        nb_pol: i32,
    ) -> Result<Self, Failure> {
        let (rows, curves) = ([first_point, last_point], ssp.nb_p3d() + ssp.nb_p2d());
        let least_square: LeastSquare = LeastSquare::with_knots_without_parameters(
            ssp,
            knots,
            mults,
            first_point,
            last_point,
            point_constraint(constraints, first_point)?,
            point_constraint(constraints, last_point)?,
            nb_pol,
        )?;
        let mut multi_bsp_curve: MultiBSpCurve = MultiBSpCurve::with_nb_poles(nb_pol)?;
        multi_bsp_curve.set_knots(knots);
        multi_bsp_curve.set_multiplicities(mults);
        Ok(Self {
            done: false,
            multi_bsp_curve,
            parameters: super::arrays::owned_vector(parameters)?,
            fval: 0.,
            val_grad_f: Vector::filled(first_point, last_point, 0.)?,
            my_f: matrix(rows, curves)?,
            least_square,
            targets: FitTargets::new(ssp, rows, constraints, "AppParCurves_BSpFunction")?,
            nb_p: last_point - first_point + 1,
            err3d: 0.,
            err2d: 0.,
            lambda1: 0.,
            lambda2: 0.,
            failure: None,
            marker: std::marker::PhantomData,
        })
    }

    /// `FirstConstraint(TheConstraints, FirstPoint)`; see [`point_constraint`].
    pub fn first_constraint(
        &self,
        constraints: &Constraints,
        first_point: i32,
    ) -> Result<Constraint, Failure> {
        point_constraint(constraints, first_point)
    }

    /// `LastConstraint(TheConstraints, LastPoint)`; see [`point_constraint`].
    pub fn last_constraint(
        &self,
        constraints: &Constraints,
        last_point: i32,
    ) -> Result<Constraint, Failure> {
        point_constraint(constraints, last_point)
    }

    /// `NbVariables()`; `f.nb_variables()`.
    pub fn nb_variables(&self) -> i32 {
        self.nb_p
    }

    /// `SetFirstLambda(l1)`; `f.set_first_lambda(1.0)`.
    pub fn set_first_lambda(&mut self, l1: f64) {
        self.lambda1 = l1;
    }

    /// `SetLastLambda(l2)`; `f.set_last_lambda(1.0)`.
    pub fn set_last_lambda(&mut self, l2: f64) {
        self.lambda2 = l2;
    }

    /// `Value(X, F)`: fits for `x` with the stored lambdas; false when the fit fails.
    /// F is left unassigned for a constrained function; `f.evaluate(&x, &mut value)?`.
    pub fn evaluate(&mut self, x: &Vector<'_>, f: &mut f64) -> Result<bool, Failure> {
        self.parameters.initialized(x)?;
        self.least_square
            .perform_lambdas(&self.parameters, self.lambda1, self.lambda2)?;
        if !self.least_square.is_done() {
            self.done = false;
            return Ok(false);
        }
        if !self.targets.contraintes {
            (self.fval, self.err3d, self.err2d) = self.least_square.error()?;
            *f = self.fval;
        }
        Ok(true)
    }

    /// `Perform(X)` (protected natively): F and its gradient for `x`.
    pub fn perform(&mut self, x: &Vector<'_>) -> Result<(), Failure> {
        self.parameters.initialized(x)?;
        self.least_square
            .perform_lambdas(&self.parameters, self.lambda1, self.lambda2)?;
        if !self.least_square.is_done() {
            self.done = false;
            return Ok(());
        }
        for j in self.parameters.lower()..=self.parameters.upper() {
            *self.val_grad_f.change_value(j)? = 0.;
        }
        if !self.targets.contraintes {
            (self.fval, self.err3d, self.err2d) =
                self.least_square.error_gradient(&mut self.val_grad_f)?;
        }
        Ok(())
    }

    /// `Gradient(X, G)`: Perform then G; always true; `f.evaluate_gradient(&x, &mut g)?`.
    pub fn evaluate_gradient(
        &mut self,
        x: &Vector<'_>,
        g: &mut Vector<'_>,
    ) -> Result<bool, Failure> {
        self.perform(x)?;
        g.initialized(&self.val_grad_f)?;
        Ok(true)
    }

    /// `Values(X, F, G)`: Perform then F and G; always true;
    /// `f.evaluate_values(&x, &mut value, &mut g)?`.
    pub fn evaluate_values(
        &mut self,
        x: &Vector<'_>,
        f: &mut f64,
        g: &mut Vector<'_>,
    ) -> Result<bool, Failure> {
        self.perform(x)?;
        *f = self.fval;
        g.initialized(&self.val_grad_f)?;
        Ok(true)
    }

    /// `CurveValue()`: the fitted curves (a copy sharing their poles with the fit);
    /// `f.curve_value()?`.
    pub fn curve_value(&mut self) -> Result<MultiBSpCurve, Failure> {
        if !self.targets.contraintes {
            self.multi_bsp_curve = self.least_square.bspline_value()?.clone();
        }
        Ok(self.multi_bsp_curve.clone())
    }

    /// `Error(IPoint, CurveIndex)`: the fit's distance (square roots taken once per
    /// fit by `Distance()`), or the constrained branch's (0); `f.error(3, 1)?`.
    pub fn error(&mut self, i_point: i32, curve_index: i32) -> Result<f64, Failure> {
        let d: f64 = *self.least_square.distance().value(i_point, curve_index)?;
        if !self.targets.contraintes {
            return Ok(d);
        }
        Ok(self.my_f.value(i_point, curve_index)?.sqrt())
    }

    /// `MaxError3d()`; `f.max_error_3d()`.
    pub fn max_error_3d(&self) -> f64 {
        self.err3d
    }

    /// `MaxError2d()`; `f.max_error_2d()`.
    pub fn max_error_2d(&self) -> f64 {
        self.err2d
    }

    /// `NewParameters()`; `f.new_parameters()`.
    pub fn new_parameters(&self) -> &Vector<'static> {
        &self.parameters
    }

    /// `FunctionMatrix()`: the fit's B-spline matrix; `f.function_matrix()?`.
    pub fn function_matrix(&self) -> Result<&Matrix<'static>, Failure> {
        self.least_square.function_matrix()
    }

    /// `DerivativeFunctionMatrix()`; `f.derivative_function_matrix()?`.
    pub fn derivative_function_matrix(&self) -> Result<&Matrix<'static>, Failure> {
        self.least_square.derivative_function_matrix()
    }

    /// `Index()`: the span index of each point; `f.index()`.
    pub fn index(&self) -> &IntegerVector<'static> {
        self.least_square.k_index()
    }
}

impl_par_function!(BSpFunction);
