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
// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_BSplineCurve.hxx and ModelingData/TKG3d/Geom/Geom_BSplineCurve.hxx (the public members that forward to the shared state) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The public members of `Geom2d_BSplineCurve` and `Geom_BSplineCurve` that only
//! lock the shared [`BSplineCurveCore`](super::BSplineCurveCore) and forward to it:
//! the same list, names and bodies in both classes, written once as macros.

/// The public read accessors of `Geom2d_BSplineCurve` and `Geom_BSplineCurve`, which
/// forward to the locked [`BSplineCurveCore`](crate::geom2d::bspline_curve_core::BSplineCurveCore); expanded inside the curve's `impl`
/// with its point and vector types;
/// `rustycad_tkg2d::bspline_curve_accessors!(Point3, Vector3);`.
#[macro_export]
macro_rules! bspline_curve_accessors {
    ($point:ty, $vector:ty) => {
        $crate::spline_pole_members!($point);
        /// Point located between knots `from_k1` and `to_k2` (`LocalValue`);
        /// `curve.local_value(0.5, 1, 2)?`.
        pub fn local_value(&self, u: f64, from_k1: i32, to_k2: i32) -> Result<$point, Failure> {
            self.local_d0(u, from_k1, to_k2)
        }
        /// `LocalD0`; `curve.local_d0(0.5, 1, 2)?`.
        pub fn local_d0(&self, u: f64, from_k1: i32, to_k2: i32) -> Result<$point, Failure> {
            self.state.read().local_d0(u, from_k1, to_k2)
        }
        /// `LocalD1`; `let (p, v1) = curve.local_d1(0.5, 1, 2)?;`.
        pub fn local_d1(
            &self,
            u: f64,
            from_k1: i32,
            to_k2: i32,
        ) -> Result<($point, $vector), Failure> {
            self.state.read().local_d1(u, from_k1, to_k2)
        }
        /// `LocalD2`; `let (p, v1, v2) = curve.local_d2(0.5, 1, 2)?;`.
        pub fn local_d2(
            &self,
            u: f64,
            from_k1: i32,
            to_k2: i32,
        ) -> Result<($point, $vector, $vector), Failure> {
            self.state.read().local_d2(u, from_k1, to_k2)
        }
        /// `LocalD3`; `let (p, v1, v2, v3) = curve.local_d3(0.5, 1, 2)?;`.
        pub fn local_d3(
            &self,
            u: f64,
            from_k1: i32,
            to_k2: i32,
        ) -> Result<($point, $vector, $vector, $vector), Failure> {
            self.state.read().local_d3(u, from_k1, to_k2)
        }
        /// `LocalDN`; `curve.local_dn(0.5, 1, 2, 2)?`.
        pub fn local_dn(
            &self,
            u: f64,
            from_k1: i32,
            to_k2: i32,
            n: i32,
        ) -> Result<$vector, Failure> {
            self.state.read().local_dn(u, from_k1, to_k2, n)
        }
        /// The degree (`Degree`); `curve.degree()`.
        pub fn degree(&self) -> i32 {
            self.state.read().deg
        }
        /// Index of the first knot of the parameter range (`FirstUKnotIndex`);
        /// `curve.first_u_knot_index()?`.
        pub fn first_u_knot_index(&self) -> Result<i32, Failure> {
            self.state.read().first_u_knot_index()
        }
        /// Index of the last knot of the parameter range (`LastUKnotIndex`);
        /// `curve.last_u_knot_index()?`.
        pub fn last_u_knot_index(&self) -> Result<i32, Failure> {
            self.state.read().last_u_knot_index()
        }
        /// Knot `index` (`Knot`); `curve.knot(1)?`.
        pub fn knot(&self, index: i32) -> Result<f64, Failure> {
            self.state.read().knot(index)
        }
        /// Copy the knots of `target`'s bounds (deprecated `Knots(K)`); `curve.knots_into(&mut k)?`.
        pub fn knots_into(&self, target: &mut Array1<'_, f64>) -> Result<(), Failure> {
            self.state.read().knots_into(target)
        }
        /// The knots, indexed from 1 (`Knots()`); `curve.knots()`.
        pub fn knots(&self) -> Array1<'static, f64> {
            self.state.read().knots.clone()
        }
        /// Copy the flat knots of `target`'s bounds (deprecated `KnotSequence(K)`);
        /// `curve.knot_sequence_into(&mut k)?`.
        pub fn knot_sequence_into(&self, target: &mut Array1<'_, f64>) -> Result<(), Failure> {
            self.state.read().knot_sequence_into(target)
        }
        /// The knots repeated by multiplicity (`KnotSequence()`); `curve.knot_sequence()`.
        pub fn knot_sequence(&self) -> Array1<'static, f64> {
            self.state.read().flat_knots.clone()
        }
        /// The knot distribution (`KnotDistribution`); `curve.knot_distribution()`.
        pub fn knot_distribution(&self) -> ::rustycad_tkmath::geom_abs::BSplKnotDistribution {
            self.state.read().knot_set
        }
        /// The knot interval `(I1, I2)` of `u`; equal indices when `u` is a knot within
        /// the tolerance, 0 or NbKnots+1 outside (`LocateU`);
        /// `curve.locate_u(0.5, 1e-9, false)?`.
        pub fn locate_u(
            &self,
            u: f64,
            parametric_tolerance: f64,
            with_knot_repetition: bool,
        ) -> Result<(i32, i32), Failure> {
            self.state
                .read()
                .locate_u(u, parametric_tolerance, with_knot_repetition)
        }
        /// Multiplicity of knot `index` (`Multiplicity`); `curve.multiplicity(1)?`.
        pub fn multiplicity(&self, index: i32) -> Result<i32, Failure> {
            self.state.read().multiplicity(index)
        }
        /// Copy the multiplicities into `target` of equal length (deprecated
        /// `Multiplicities(M)`); `curve.multiplicities_into(&mut m)?`.
        pub fn multiplicities_into(&self, target: &mut Array1<'_, i32>) -> Result<(), Failure> {
            self.state.read().multiplicities_into(target)
        }
        /// The multiplicities, indexed from 1 (`Multiplicities()`); `curve.multiplicities()`.
        pub fn multiplicities(&self) -> Array1<'static, i32> {
            self.state.read().mults.clone()
        }
        /// Number of knots (`NbKnots`); `curve.nb_knots()`.
        pub fn nb_knots(&self) -> i32 {
            self.state.read().knots.len() as i32
        }
    };
}

/// The public edits and queries of `Geom2d_BSplineCurve` and `Geom_BSplineCurve`
/// that forward to the locked [`BSplineCurveCore`](crate::geom2d::bspline_curve_core::BSplineCurveCore) under the same names; expanded
/// inside the curve's `impl` with its point and vector types;
/// `rustycad_tkg2d::bspline_curve_edits!(Point3, Vector3);`.
#[macro_export]
macro_rules! bspline_curve_edits {
    ($point:ty, $vector:ty) => {
        /// Raise the degree (`IncreaseDegree`); `curve.increase_degree(5)?`.
        pub fn increase_degree(&self, degree: i32) -> Result<(), Failure> {
            self.state.write().increase_degree(degree)
        }
        /// Raise the multiplicity of knot `index` to `mult` (`IncreaseMultiplicity(Index, M)`);
        /// `curve.increase_multiplicity(2, 3)?`.
        pub fn increase_multiplicity(&self, index: i32, mult: i32) -> Result<(), Failure> {
            self.state.write().increase_multiplicity(index, mult)
        }
        /// Raise the multiplicities of knots `i1..=i2` to `mult` (`IncreaseMultiplicity(I1, I2, M)`);
        /// `curve.increase_multiplicity_range(2, 3, 3)?`.
        pub fn increase_multiplicity_range(
            &self,
            i1: i32,
            i2: i32,
            mult: i32,
        ) -> Result<(), Failure> {
            self.state.write().raise_multiplicities(i1, i2, mult, false)
        }
        /// Add `step` to the multiplicities of knots `i1..=i2` (`IncrementMultiplicity`);
        /// `curve.increment_multiplicity(2, 3, 1)?`.
        pub fn increment_multiplicity(&self, i1: i32, i2: i32, step: i32) -> Result<(), Failure> {
            self.state.write().raise_multiplicities(i1, i2, step, true)
        }
        /// Insert knots (`InsertKnots(Knots, Mults, ParametricTolerance, Add)`); with `add` the
        /// multiplicities of existing knots are increased by `mults`, otherwise raised to them;
        /// `curve.insert_knots(&knots, &mults, 0.0, false)?`.
        pub fn insert_knots(
            &self,
            knots: &Array1<'_, f64>,
            mults: &Array1<'_, i32>,
            parametric_tolerance: f64,
            add: bool,
        ) -> Result<(), Failure> {
            self.state
                .write()
                .insert_knots(knots, mults, parametric_tolerance, add)
        }
        /// Reduce knot `index` to multiplicity `mult` if the curve moves less than
        /// `tolerance`; false when it would (`RemoveKnot`); `curve.remove_knot(2, 0, 1e-7)?`.
        pub fn remove_knot(&self, index: i32, mult: i32, tolerance: f64) -> Result<bool, Failure> {
            self.state.write().remove_knot(index, mult, tolerance)
        }
        /// Restrict to `[u1, u2]` (`Segment`, native default tolerance Precision::PConfusion());
        /// `curve.segment(0.2, 0.8, precision::p_confusion())?`.
        pub fn segment(&self, u1: f64, u2: f64, tolerance: f64) -> Result<(), Failure> {
            self.state.write().segment(u1, u2, tolerance)
        }
        /// Move knot `index` strictly between its neighbours (`SetKnot(Index, K)`); `curve.set_knot(2, 0.4)?`.
        pub fn set_knot(&self, index: i32, knot: f64) -> Result<(), Failure> {
            self.state.write().set_knot(index, knot)
        }
        /// Replace all knot values (`SetKnots`); `curve.set_knots(&knots)?`.
        pub fn set_knots(&self, knots: &Array1<'_, f64>) -> Result<(), Failure> {
            self.state.write().set_knots(knots)
        }
        /// Raise the multiplicity of knot `index` to `mult`, then move it (`SetKnot(Index, K, M)`);
        /// `curve.set_knot_with_multiplicity(2, 0.4, 2)?`.
        pub fn set_knot_with_multiplicity(
            &self,
            index: i32,
            knot: f64,
            mult: i32,
        ) -> Result<(), Failure> {
            let mut state = self.state.write();
            state.increase_multiplicity(index, mult)?;
            state.set_knot(index, knot)
        }
        /// Shift a parameter of a periodic curve into its range (`PeriodicNormalization`);
        /// `curve.periodic_normalization(7.0)?`.
        pub fn periodic_normalization(&self, u: f64) -> Result<f64, Failure> {
            self.state.read().periodic_normalization(u)
        }
        /// Make the curve periodic (`SetPeriodic`); `curve.set_periodic()?`.
        pub fn set_periodic(&self) -> Result<(), Failure> {
            self.state.write().set_periodic()
        }
        /// Make knot `index` the origin of a periodic curve (`SetOrigin`); `curve.set_origin(2)?`.
        pub fn set_origin(&self, index: i32) -> Result<(), Failure> {
            self.state.write().set_origin(index)
        }
        /// Make the curve non-periodic (`SetNotPeriodic`); `curve.set_not_periodic()?`.
        pub fn set_not_periodic(&self) -> Result<(), Failure> {
            self.state.write().set_not_periodic()
        }
        /// Move the point of parameter `u` to `point` by changing poles `index1..=index2`; returns
        /// `(FirstModifiedPole, LastModifiedPole)`, `(0, 0)` when nothing moved (`MovePoint`);
        /// `curve.move_point(0.5, p, 2, 3)?`.
        pub fn move_point(
            &self,
            u: f64,
            point: $point,
            index1: i32,
            index2: i32,
        ) -> Result<(i32, i32), Failure> {
            self.state.write().move_point(u, point, index1, index2)
        }
        /// Move the point and tangent of parameter `u`; returns `ErrorStatus`, 0 on success
        /// (`MovePointAndTangent`); `curve.move_point_and_tangent(0.5, p, t, 1e-7, 1, 1)?`.
        pub fn move_point_and_tangent(
            &self,
            u: f64,
            point: $point,
            tangent: $vector,
            tolerance: f64,
            starting_condition: i32,
            ending_condition: i32,
        ) -> Result<i32, Failure> {
            let conditions: (i32, i32) = (starting_condition, ending_condition);
            self.state
                .write()
                .move_point_and_tangent(u, point, tangent, tolerance, conditions)
        }
        /// Tangent continuity within `ang_tol` on `[tf, tl]` (`IsG1`); `curve.is_g1(0.0, 1.0, 1e-12)?`.
        pub fn is_g1(&self, tf: f64, tl: f64, ang_tol: f64) -> Result<bool, Failure> {
            self.state.read().is_g1(tf, tl, ang_tol)
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
    };
}
