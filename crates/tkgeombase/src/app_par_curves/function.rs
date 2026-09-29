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
// Rust translation of ModelingData/TKGeomBase/AppParCurves/AppParCurves_Function.gxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AppParCurves_Function` (AppParCurves_Function.gxx): F(u) = the sum of the
//! squared distances between a MultiLine and its Bezier fit for the parameters u,
//! and its gradient, for the parameter optimizer.
mod constrained;
mod correction;

use super::Constraints;
use super::constraint::Constraint;
use super::fit_targets::{FitTargets, impl_par_function};
use super::least_square::LeastSquare;
use super::line_tool::LineTool;
use super::multi_curve::MultiCurve;
use super::resol_constraint::couples;
use crate::native_failure::array_failure;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::{Matrix, Vector};

/// `AppParCurves_Function`: generic over the MultiLine `L` (kept by copy, as natively).
///
/// Only the constraints on points other than the extremities make the function
/// "constrained" (a Uzawa correction after each fit); `Error()` reads the squared
/// distances of that branch only and is 0 for an unconstrained function. The
/// errors of a native exception inside a callback of the optimizer are kept and
/// returned by [`Function::take_failure`].
///
/// ```ignore
/// let mut f = Function::new(&line, 1, 10, &constraints, &parameters, 3)?;
/// let mut value = 0.0;
/// assert!(f.evaluate(&parameters, &mut value)?);
/// ```
pub struct Function<L: LineTool + Clone> {
    done: bool,
    multi_line: L,
    multi_curve: MultiCurve,
    degre: i32,
    parameters: Vector<'static>,
    fval: f64,
    val_grad_f: Vector<'static>,
    my_f: Matrix<'static>,
    a: Matrix<'static>,
    da: Matrix<'static>,
    least_square: LeastSquare,
    targets: FitTargets,
    nb_p: i32,
    nb_cu: i32,
    err3d: f64,
    err2d: f64,
    first_p: i32,
    last_p: i32,
    constraints: Constraints,
    failure: Option<Failure>,
}

/// `FirstConstraint(TheConstraints, FirstPoint)`: the constraint of the first couple
/// on `point`, or of the *last* couple when none is (the loop variable keeps it);
/// `point_constraint(&constraints, 1)?`.
pub fn point_constraint(constraints: &Constraints, point: i32) -> Result<Constraint, Failure> {
    let mut cons: Constraint = Constraint::NoConstraint;
    for couple in couples(constraints)? {
        cons = couple.constraint();
        if couple.index() == point {
            break;
        }
    }
    Ok(cons)
}

fn matrix(rows: [i32; 2], cols: i32) -> Result<Matrix<'static>, Failure> {
    Matrix::filled(rows[0], rows[1], 1, cols, 0.)
        .map_err(|error| array_failure(error, "AppParCurves_Function"))
}

impl<L: LineTool + Clone> Function<L> {
    /// `AppParCurves_Function(SSP, FirstPoint, LastPoint, TheConstraints, Parameters, Deg)`:
    /// the fit of degree `deg` is initialized with the constraints of the extremities;
    /// `Function::new(&line, 1, 10, &constraints, &u, 3)?`.
    pub fn new(
        ssp: &L,
        first_point: i32,
        last_point: i32,
        constraints: &Constraints,
        parameters: &Vector<'_>,
        deg: i32,
    ) -> Result<Self, Failure> {
        let (rows, curves) = ([first_point, last_point], ssp.nb_p3d()? + ssp.nb_p2d()?);
        let least_square = LeastSquare::without_parameters(
            ssp,
            first_point,
            last_point,
            point_constraint(constraints, first_point)?,
            point_constraint(constraints, last_point)?,
            deg + 1,
        );
        Ok(Self {
            done: false,
            multi_line: ssp.clone(),
            multi_curve: MultiCurve::with_nb_poles(deg + 1)?,
            degre: deg,
            parameters: super::arrays::owned_vector(parameters)?,
            fval: 0.,
            val_grad_f: Vector::filled(first_point, last_point, 0.)?,
            my_f: matrix(rows, curves)?,
            a: matrix(rows, deg + 1)?,
            da: matrix(rows, deg + 1)?,
            least_square: least_square?,
            targets: FitTargets::new(ssp, rows, constraints, "AppParCurves_Function")?,
            nb_p: last_point - first_point + 1,
            nb_cu: curves,
            err3d: 0.,
            err2d: 0.,
            first_p: first_point,
            last_p: last_point,
            constraints: constraints.clone(),
            failure: None,
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

    /// `NbVariables()`: `LastPoint - FirstPoint + 1`; `f.nb_variables()`.
    pub fn nb_variables(&self) -> i32 {
        self.nb_p
    }

    /// `Value(X, F)`: fits for the parameters `x` (and corrects the poles by Uzawa when
    /// constrained); false when a fit fails; `f.evaluate(&x, &mut value)?`.
    pub fn evaluate(&mut self, x: &Vector<'_>, f: &mut f64) -> Result<bool, Failure> {
        self.parameters.initialized(x)?;
        self.least_square.perform(&self.parameters)?;
        if !self.least_square.is_done() {
            self.done = false;
            return Ok(false);
        }
        if !self.targets.contraintes {
            (self.fval, self.err3d, self.err2d) = self.least_square.error()?;
            *f = self.fval;
            return Ok(true);
        }
        if !self.constrained_value()? {
            return Ok(false);
        }
        *f = self.fval;
        Ok(true)
    }

    /// `Perform(X)` (protected natively): F and its gradient for the parameters `x`.
    pub fn perform(&mut self, x: &Vector<'_>) -> Result<(), Failure> {
        self.parameters.initialized(x)?;
        self.least_square.perform(&self.parameters)?;
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
            return Ok(());
        }
        self.constrained_gradient(x)
    }

    /// `Gradient(X, G)`: Perform then G = the gradient; always true;
    /// `f.evaluate_gradient(&x, &mut g)?`.
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

    /// `CurveValue()`: the fitted (and corrected) curves; `f.curve_value()?`.
    pub fn curve_value(&mut self) -> Result<&MultiCurve, Failure> {
        if !self.targets.contraintes {
            self.multi_curve = self.least_square.bezier_value()?;
        }
        Ok(&self.multi_curve)
    }

    /// `Error(IPoint, CurveIndex)`: the distance of the constrained branch (0 for an
    /// unconstrained function); `f.error(3, 1)?`.
    pub fn error(&self, i_point: i32, curve_index: i32) -> Result<f64, Failure> {
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

    /// `NewParameters()`: the parameters of the last evaluation; `f.new_parameters()`.
    pub fn new_parameters(&self) -> &Vector<'static> {
        &self.parameters
    }
}

impl_par_function!(Function);
