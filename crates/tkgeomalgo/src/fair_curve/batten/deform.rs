// Copyright (c) 1996-1999 Matra Datavision
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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/FairCurve/FairCurve_Batten.cxx and
// FairCurve_MinimalVariation.cxx (the deformation of the private Compute) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The deformation of a homotopy step: the Hermite polynomial that moves the ends
//! (points, tangents, second derivatives) by the requested deltas, added to the
//! current poles.
use super::FairCurveBatten;
use super::step::HomotopyStep;
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, arrays::one_based, errors};
use rustycad_tkmath::gp::{Point2, Vector2, Xy};
use rustycad_tkmath::math::Matrix;
use rustycad_tkmath::plib;

/// The end derivative of the current curve and the knot gap of its end span.
pub(in crate::fair_curve) struct EndDerivative {
    pub(in crate::fair_curve) derivative: Vector2,
    pub(in crate::fair_curve) knot_gap: f64,
}

impl FairCurveBatten {
    /// The number of Hermite conditions: 2 + both requested constraint orders.
    fn hermite_size(&self) -> i32 {
        2 + self.new.constraint_order1 + self.new.constraint_order2
    }
    /// `PLib::HermiteCoefficients(0, 1, Order1, Order2)`; None where the native code
    /// returns false.
    pub(in crate::fair_curve) fn hermite(&self) -> Result<Option<Matrix<'static>>, Failure> {
        let size: i32 = self.hermite_size();
        let mut coef: Matrix<'static> = Matrix::with_bounds(1, size, 1, size).map_err(|error| {
            errors::dimension(&format!("Hermite matrix {size}x{size}: {error}"))
        })?;
        let (order1, order2) = (self.new.constraint_order1, self.new.constraint_order2);
        let done: bool = plib::hermite_coefficients(0.0, 1.0, order1, order2, &mut coef)?;
        Ok(done.then_some(coef))
    }
    /// `DAngleRef`: the rotation of the chord by the moves of the end points.
    pub(in crate::fair_curve) fn reference_angle(
        &self,
        step: &HomotopyStep,
    ) -> Result<f64, Failure> {
        let v_old = Vector2::between(self.old.p1, self.old.p2);
        let moved: Xy =
            -(self.old.p1.xy() + step.delta_p1.xy()) + (self.old.p2.xy() + step.delta_p2.xy());
        Vector2::from(moved).angle(v_old)
    }
    /// The first derivative of the current curve at P1 (`OldDerive`).
    pub(in crate::fair_curve) fn start_derivative(&self) -> EndDerivative {
        let poles = self.poles.get().expect("batten poles").array1();
        let knots = self.knots.get().expect("batten knots").array1();
        let lower: i32 = poles.lower();
        let derivative = Vector2::between(poles[lower], poles[lower + 1]);
        let knot_gap: f64 = knots[knots.lower() + 1] - knots[knots.lower()];
        self.scaled_derivative(derivative, knot_gap)
    }
    /// The first derivative of the current curve at P2 (`OldDerive`).
    pub(in crate::fair_curve) fn end_derivative(&self) -> EndDerivative {
        let poles = self.poles.get().expect("batten poles").array1();
        let knots = self.knots.get().expect("batten knots").array1();
        let upper: i32 = poles.upper();
        let derivative = Vector2::between(poles[upper - 1], poles[upper]);
        let knot_gap: f64 = knots[knots.upper()] - knots[knots.upper() - 1];
        self.scaled_derivative(derivative, knot_gap)
    }
    /// `OldDerive *= Degree / KnotGap` unless the gap is below the computational
    /// precision.
    fn scaled_derivative(&self, mut derivative: Vector2, knot_gap: f64) -> EndDerivative {
        if knot_gap.abs() > precision::computational() {
            derivative *= f64::from(self.degree) / knot_gap;
        }
        EndDerivative {
            derivative,
            knot_gap,
        }
    }
    /// The current poles plus the Hermite deformation of `adelta` converted to a
    /// B-spline of the batten's structure (`NPoles`).
    pub(in crate::fair_curve) fn deformed_poles(
        &self,
        hermite: &Matrix<'_>,
        adelta: &[Xy],
    ) -> Result<Handle<HArray1<Point2>>, Failure> {
        let size: i32 = self.hermite_size();
        let mut interpolation: Array1<'static, Point2> = one_based(size)?;
        for ii in 1..=size {
            let mut aux: Xy = Xy::new(0.0, 0.0);
            for kk in 1..=size {
                aux += hermite[(kk, ii)] * adelta[(kk - 1) as usize];
            }
            interpolation[ii] = Point2::from(aux);
        }
        let mut hermite_poles: Array1<'static, Point2> = one_based(size)?;
        plib::coefficients_poles(&interpolation, None, &mut hermite_poles, None)?;
        let delta = self.delta_curve(&hermite_poles)?;
        let delta_poles: Array1<'static, Point2> = delta.poles();
        let poles = self.poles.get().expect("batten poles").array1();
        if delta_poles.len() != poles.len() {
            return Err(errors::dimension(&format!(
                "FairCurve: {} deformation poles; expected {} like the batten",
                delta_poles.len(),
                poles.len()
            )));
        }
        let mut npoles: Array1<'static, Point2> = one_based(poles.len() as i32)?;
        for kk in 1..=poles.len() as i32 {
            let delta_xy: Xy = delta_poles[delta_poles.lower() + kk - 1].xy();
            npoles[kk] = Point2::from(delta_xy + poles[poles.lower() + kk - 1].xy());
        }
        Ok(Handle::new(HArray1::from_owned(npoles)))
    }
    /// The Bezier of the Hermite poles raised to the batten degree and knots.
    fn delta_curve(
        &self,
        hermite_poles: &Array1<'_, Point2>,
    ) -> Result<Geom2dBSplineCurve, Failure> {
        let size: i32 = hermite_poles.len() as i32;
        let mut knots: Array1<'static, f64> = one_based(2)?;
        (knots[1], knots[2]) = (0.0, 1.0);
        let mut mults: Array1<'static, i32> = one_based(2)?;
        (mults[1], mults[2]) = (size, size);
        let delta = Geom2dBSplineCurve::new(hermite_poles, &knots, &mults, size - 1, false)?;
        delta.increase_degree(self.degree)?;
        let batten_mults = self.mults.get().expect("batten mults").array1();
        if batten_mults.len() > 2 {
            let batten_knots = self.knots.get().expect("batten knots").array1();
            delta.insert_knots(&batten_knots, &batten_mults, 1.0e-10, false)?;
        }
        Ok(delta)
    }
}
