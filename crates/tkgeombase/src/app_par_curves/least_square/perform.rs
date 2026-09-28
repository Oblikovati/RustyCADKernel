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

//! The four `AppParCurves_LeastSquare::Perform` overloads.
use super::LeastSquare;
use crate::app_par_curves::bernstein::bernstein;
use crate::app_par_curves::constraint::Constraint;
use crate::app_par_curves::spline_function::spline_function;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::math::{Householder, Vector};

impl LeastSquare {
    /// `Perform(Parameters)`: the fit for the stored constraints. The unconstrained
    /// case is a Householder QR; otherwise the pass-point system is solved and the
    /// code falls through to the full system with the tangency unknowns (lambda1,
    /// lambda2), which recomputes the poles, as natively. Curvature constraints are
    /// not treated here (see [`LeastSquare::perform_curvatures`]).
    /// `fit.perform(&parameters)?`.
    pub fn perform(&mut self, parameters: &Vector<'_>) -> Result<(), Failure> {
        self.done = false;
        if !self.isready {
            return Ok(());
        }
        self.iscalculated = false;
        self.compute_function(parameters)?;
        let (first, last) = (self.first_constraint, self.last_constraint);
        if first != Constraint::TangencyPoint && last != Constraint::TangencyPoint {
            if first == Constraint::NoConstraint && last == Constraint::NoConstraint {
                return self.householder();
            }
            self.pass_point_rhs()?;
            if self.resfin - self.resinit + 1 < 1 {
                self.done = true;
                return Ok(());
            }
            self.solve_blocks()?;
            self.done = true;
        }
        self.solve_full_system()
    }

    // The unconstrained fit: mypoles = the QR solution of A X = mypoints.
    fn householder(&mut self) -> Result<(), Failure> {
        let solution = Householder::from_matrix(&self.a, &self.mypoints, 1.0e-20)?;
        if !solution.is_done() {
            self.done = false;
            return Ok(());
        }
        self.done = true;
        self.mypoles.initialized(solution.all_values()?)
    }

    // B2 = mypoints - (the fixed first and last pole terms), for the pass-point cases of
    // Perform(Parameters); a curvature constraint leaves B2 as it is, as natively.
    fn pass_point_rhs(&mut self) -> Result<(), Failure> {
        let n: i32 = self.nbpoles;
        let fixed: Vec<i32> = match (self.first_constraint, self.last_constraint) {
            (Constraint::NoConstraint, _) => vec![n],
            (Constraint::PassPoint, Constraint::NoConstraint) => vec![1],
            (Constraint::PassPoint, Constraint::PassPoint) => vec![1, n],
            _ => return Ok(()),
        };
        self.subtract_fixed(&fixed)
    }

    /// `Perform(Parameters, V1t, V2t, l1, l2)`: both ends become tangency points with
    /// the tangents `v1t`, `v2t` (read from their lower bounds);
    /// `fit.perform_tangents(&u, &v1, &v2, 1.0, 1.0)?`.
    pub fn perform_tangents(
        &mut self,
        parameters: &Vector<'_>,
        v1t: &Vector<'_>,
        v2t: &Vector<'_>,
        l1: f64,
        l2: f64,
    ) -> Result<(), Failure> {
        self.done = false;
        if !self.isready {
            return Ok(());
        }
        self.resinit = 3;
        self.resfin = self.nbpoles - 2;
        self.ninc = self.na * (self.resfin - self.resinit + 1) + 2;
        self.first_constraint = Constraint::TangencyPoint;
        self.last_constraint = Constraint::TangencyPoint;
        for i in 1..=self.vec1t.upper() {
            self.vec1t[i] = *v1t.value(i + v1t.lower() - 1)?;
            self.vec2t[i] = *v2t.value(i + v2t.lower() - 1)?;
        }
        self.perform_lambdas(parameters, l1, l2)
    }

    /// `Perform(Parameters, V1t, V2t, V1c, V2c, l1, l2)`: both ends become curvature
    /// points with the tangents and curvatures given;
    /// `fit.perform_curvatures(&u, [&v1t, &v2t, &v1c, &v2c], 1.0, 1.0)?`.
    pub fn perform_curvatures(
        &mut self,
        parameters: &Vector<'_>,
        [v1t, v2t, v1c, v2c]: [&Vector<'_>; 4],
        l1: f64,
        l2: f64,
    ) -> Result<(), Failure> {
        self.done = false;
        if !self.isready {
            return Ok(());
        }
        self.resinit = 4;
        self.resfin = self.nbpoles - 3;
        self.ninc = self.na * (self.resfin - self.resinit + 1) + 2;
        self.first_constraint = Constraint::CurvaturePoint;
        self.last_constraint = Constraint::CurvaturePoint;
        for i in 1..=self.vec1t.upper() {
            self.vec1t[i] = *v1t.value(i + v1t.lower() - 1)?;
            self.vec2t[i] = *v2t.value(i + v2t.lower() - 1)?;
            self.vec1c[i] = *v1c.value(i + v1c.lower() - 1)?;
            self.vec2c[i] = *v2c.value(i + v2c.lower() - 1)?;
        }
        self.perform_lambdas(parameters, l1, l2)
    }

    /// `Perform(Parameters, l1, l2)`: the fit with the tangency (and curvature) poles
    /// fixed by the given lambdas; without tangency constraints it is
    /// `Perform(Parameters)`; `fit.perform_lambdas(&u, 1.0, 1.0)?`.
    pub fn perform_lambdas(
        &mut self,
        parameters: &Vector<'_>,
        l1: f64,
        l2: f64,
    ) -> Result<(), Failure> {
        self.done = false;
        if !self.isready {
            return Ok(());
        }
        if self.first_constraint < Constraint::TangencyPoint
            && self.last_constraint < Constraint::TangencyPoint
        {
            return self.perform(parameters);
        }
        self.iscalculated = false;
        (self.lambda1, self.lambda2) = (l1, l2);
        self.compute_function(parameters)?;
        self.lambda_poles(l1, l2)?;
        if self.resinit > self.resfin {
            self.done = true;
            return Ok(());
        }
        self.lambda_rhs()?;
        self.solve_blocks()?;
        self.done = true;
        Ok(())
    }

    // The poles fixed by the tangents (poles 2 and NbPoles - 1) and the curvatures
    // (poles 3 and NbPoles - 2).
    fn lambda_poles(&mut self, l1: f64, l2: f64) -> Result<(), Failure> {
        let (first, last, n, deg) = (
            self.first_constraint,
            self.last_constraint,
            self.nbpoles,
            f64::from(self.deg),
        );
        let (l11, l22) = (deg * l1, deg * l2);
        let degree_factor: f64 = f64::from(self.deg * (self.deg - 1));
        for i in 1..=self.mypoles.col_number() {
            if first >= Constraint::TangencyPoint {
                self.mypoles[(2, i)] = self.mypoints[(self.myfirstp, i)] + l1 * self.vec1t[i];
            }
            if first == Constraint::CurvaturePoint {
                self.mypoles[(3, i)] = 2. * self.mypoles[(2, i)] - self.mypoles[(1, i)]
                    + l11 * l11 * self.vec1c[i] / degree_factor;
            }
        }
        for i in 1..=self.mypoles.col_number() {
            if last >= Constraint::TangencyPoint {
                self.mypoles[(n - 1, i)] = self.mypoints[(self.mylastp, i)] - l2 * self.vec2t[i];
            }
            if last == Constraint::CurvaturePoint {
                self.mypoles[(n - 2, i)] = 2. * self.mypoles[(n - 1, i)] - self.mypoles[(n, i)]
                    + l22 * l22 * self.vec2c[i] / degree_factor;
            }
        }
        Ok(())
    }

    // B2 = mypoints - (all the fixed pole terms) for Perform(Parameters, l1, l2).
    fn lambda_rhs(&mut self) -> Result<(), Failure> {
        let (first, last, n) = (self.first_constraint, self.last_constraint, self.nbpoles);
        let tangent_last: bool = last == Constraint::TangencyPoint;
        let fixed: Vec<i32> = match (first, last) {
            (
                Constraint::NoConstraint | Constraint::PassPoint,
                Constraint::NoConstraint | Constraint::PassPoint,
            ) => return Ok(()),
            (Constraint::NoConstraint, _) if tangent_last => vec![n, n - 1],
            (Constraint::NoConstraint, _) => vec![n, n - 1, n - 2],
            (Constraint::PassPoint, _) if tangent_last => vec![1, n, n - 1],
            (Constraint::PassPoint, _) => vec![1, n, n - 1, n - 2],
            (Constraint::TangencyPoint, Constraint::NoConstraint) => vec![1, 2],
            (Constraint::TangencyPoint, Constraint::PassPoint) => vec![1, n, 2],
            (Constraint::TangencyPoint, Constraint::TangencyPoint) => vec![1, n, 2, n - 1],
            (Constraint::TangencyPoint, Constraint::CurvaturePoint) => return Ok(()),
            (Constraint::CurvaturePoint, Constraint::NoConstraint) => vec![1, 2, 3],
            (Constraint::CurvaturePoint, Constraint::PassPoint) => vec![1, 2, 3, n],
            (Constraint::CurvaturePoint, Constraint::TangencyPoint) => vec![1, 2, 3, n, n - 1],
            (Constraint::CurvaturePoint, Constraint::CurvaturePoint) => {
                vec![1, 2, 3, n, n - 1, n - 2]
            }
        };
        self.subtract_fixed(&fixed)
    }

    // B2(j, i) = mypoints(j, i) - sum over the fixed poles p (in the native order) of
    // A(j, p) mypoles(p, i), for the fitted points FirstP..=LastP.
    pub(super) fn subtract_fixed(&mut self, fixed: &[i32]) -> Result<(), Failure> {
        for j in self.first_p..=self.last_p {
            let weights: Vec<f64> = fixed
                .iter()
                .map(|p| self.a.value(j, *p).copied())
                .collect::<Result<_, _>>()?;
            for i in 1..=self.b2.col_number() {
                let mut value: f64 = *self.mypoints.value(j, i)?;
                for (p, weight) in fixed.iter().zip(&weights) {
                    value -= weight * *self.mypoles.value(*p, i)?;
                }
                *self.b2.change_value(j, i)? = value;
            }
        }
        Ok(())
    }

    /// `ComputeFunction(Parameters)`: A and DA from the Bernstein basis, or from the
    /// B-spline basis with the span indexes.
    pub(super) fn compute_function(&mut self, parameters: &Vector<'_>) -> Result<(), Failure> {
        if self.knots.is_none() {
            return bernstein(self.nbpoles, parameters, &mut self.a, &mut self.da);
        }
        spline_function(
            self.nbpoles,
            self.deg,
            parameters,
            &self.vflatknots,
            &mut self.a,
            &mut self.da,
            &mut self.myindex,
        )
    }
}
