// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_BezierCurve.hxx and ModelingData/TKG3d/Geom/Geom_BezierCurve.hxx (the public members that forward to the shared state) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The public members of `Geom2d_BezierCurve` and `Geom_BezierCurve` that only lock
//! the shared [`BezierCurveCore`](crate::geom2d::bezier_curve_core::BezierCurveCore)
//! and forward to it: the same list, names and bodies in both classes.

/// The shared public members of the Bezier curve classes; expanded inside the
/// curve's `impl` with its point type;
/// `rustycad_tkg2d::bezier_curve_members!(Point3);`.
#[macro_export]
macro_rules! bezier_curve_members {
    ($point:ty) => {
        /// Raise the degree (`Increase`); `curve.increase(5)?`.
        pub fn increase(&self, degree: i32) -> Result<(), Failure> {
            self.state.write().increase(degree)
        }
        /// Remove the pole at `index` (`RemovePole`); `curve.remove_pole(2)?`.
        pub fn remove_pole(&self, index: i32) -> Result<(), Failure> {
            self.state.write().remove_pole(index)
        }
        /// Restrict to `[u1, u2]` and reparameterize on `[0, 1]` (`Segment`); `curve.segment(0.25, 0.75)?`.
        pub fn segment(&self, u1: f64, u2: f64) -> Result<(), Failure> {
            self.state.write().segment(u1, u2)
        }
        /// Replace a pole (`SetPole(Index, P)`); `curve.set_pole(2, p)?`.
        pub fn set_pole(&self, index: i32, pole: $point) -> Result<(), Failure> {
            self.state.write().set_pole(index, pole)
        }
        /// Replace a pole and its weight (`SetPole(Index, P, Weight)`); `curve.set_pole_weighted(2, p, 2.0)?`.
        pub fn set_pole_weighted(
            &self,
            index: i32,
            pole: $point,
            weight: f64,
        ) -> Result<(), Failure> {
            let mut state = self.state.write();
            state.set_pole(index, pole)?;
            state.set_weight(index, weight)
        }
        /// Replace a weight (`SetWeight`); `curve.set_weight(2, 2.0)?`.
        pub fn set_weight(&self, index: i32, weight: f64) -> Result<(), Failure> {
            self.state.write().set_weight(index, weight)
        }
        /// Whether the weights differ (`IsRational`); `curve.is_rational()`.
        pub fn is_rational(&self) -> bool {
            self.state.read().rational
        }
        /// Number of poles minus one (`Degree`); `curve.degree()`.
        pub fn degree(&self) -> i32 {
            self.state.read().degree()
        }
        /// Number of poles (`NbPoles`); `curve.nb_poles()`.
        pub fn nb_poles(&self) -> i32 {
            self.state.read().poles.len() as i32
        }
        /// Pole at `index` (`Pole`); `curve.pole(1)?`.
        pub fn pole(&self, index: i32) -> Result<$point, Failure> {
            self.state.read().pole(index)
        }
        /// Copy the poles into `target` of equal length (deprecated `Poles(P)`);
        /// `curve.poles_into(&mut poles)?`.
        pub fn poles_into(&self, target: &mut Array1<'_, $point>) -> Result<(), Failure> {
            self.state.read().poles_into(target)
        }
        /// The poles, indexed from 1 (`Poles()`); `curve.poles()`.
        pub fn poles(&self) -> Array1<'static, $point> {
            self.state.read().poles.clone()
        }
        /// Weight at `index`; 1 for a non-rational curve (`Weight`); `curve.weight(1)?`.
        pub fn weight(&self, index: i32) -> Result<f64, Failure> {
            self.state.read().weight(index)
        }
        /// Copy the weights into `target` of equal length, writing `target(1..=n)` for a
        /// non-rational curve (deprecated `Weights(W)`); `curve.weights_into(&mut w)?`.
        pub fn weights_into(&self, target: &mut Array1<'_, f64>) -> Result<(), Failure> {
            self.state.read().weights_into(target)
        }
        /// The weights of a rational curve, `None` otherwise (`Weights()`); `curve.weights()`.
        pub fn weights(&self) -> Option<Array1<'static, f64>> {
            self.state.read().weights_view().cloned()
        }
        /// Read the stored weights in place, unit weights (a non-owning shared view) for a
        /// non-rational curve (`WeightsArray`, which natively returns a stable reference);
        /// `curve.weights_array(|weights| weights.len())`.
        pub fn weights_array<R>(&self, read: impl FnOnce(&Array1<'static, f64>) -> R) -> R {
            read(&self.state.read().weights)
        }
        /// `BSplCLib::MaxDegree()` (`MaxDegree`); `curve.max_degree() == 25` (an associated function).
        pub fn max_degree() -> i32 {
            bsplclib::max_degree()
        }
        /// Parametric tolerance for a 2D or 3D tolerance, caching the inverse derivative bound
        /// (`Resolution`); `curve.resolution(1e-7)?`.
        pub fn resolution(&self, tolerance: f64) -> Result<f64, Failure> {
            self.state.write().resolution(tolerance)
        }
        /// The knots `{0, 1}` (`Knots`); `curve.knots()`.
        pub fn knots(&self) -> Array1<'static, f64> {
            $crate::geom2d::spline_data::bezier_knots()
        }
        /// The multiplicities `{Degree+1, Degree+1}` (`Multiplicities`); `curve.multiplicities()?`.
        pub fn multiplicities(&self) -> Result<Array1<'static, i32>, Failure> {
            self.state.read().multiplicities()
        }
        /// The flat knots (`KnotSequence`); `curve.knot_sequence()?`.
        pub fn knot_sequence(&self) -> Result<Array1<'static, f64>, Failure> {
            self.state.read().knot_sequence()
        }
    };
}
