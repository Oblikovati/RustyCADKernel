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
// Rust translation of Extrema_GFuncExtCC.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Extrema_GFuncExtCC`: `F(u, v) = ((C2(v) - C1(u)).C1'(u)/|C1'|, (C2(v) - C1(u)).C2'(v)/|C2'|)`,
//! whose roots are the extrema of the distance between two curves, for
//! math_FunctionSetRoot.
use crate::extrema::curve_tool::ExtremaCurveTool;
use crate::extrema::failures::{null_object, out_of_range, sequence_value};
use crate::extrema::gp_space::{GpPoint, GpVector};
use crate::extrema::pon::POnCurve;
use crate::extrema::singular::{
    MAX_ORDER, MIN_TOL, SingularSearch, TOL_FACTOR, singular_derivative,
};
use crate::failure_capture::FailureCapture;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::math::Vector;

/// `Extrema_GFuncExtCC` over one [`ExtremaCurveTool`] (both curves of one tool, as
/// in every native instantiation: `Extrema_CCLocFOfLocECC` and the deprecated 2D
/// alias). The members that raise natively return `Result`; the math callbacks keep
/// the first failure ([`GFuncExtCC::take_failure`]);
/// `GFuncExtCC::<CurveTool>::with_curves(&c1, &c2, 1.0e-10)?`.
pub struct GFuncExtCC<'a, T: ExtremaCurveTool> {
    pub(super) c1: Option<&'a T::Curve>,
    pub(super) c2: Option<&'a T::Curve>,
    tol: f64,
    pub(super) u: f64,
    pub(super) v: f64,
    pub(super) p1: T::Point,
    pub(super) p2: T::Point,
    pub(super) du: T::Vector,
    pub(super) dv: T::Vector,
    sq_dist: Vec<f64>,
    points: Vec<POnCurve<T::Point>>,
    pub(super) tol_c1: f64,
    pub(super) tol_c2: f64,
    pub(super) max_order_c1: i32,
    pub(super) max_order_c2: i32,
    pub(super) uinfium: f64,
    pub(super) usupremum: f64,
    pub(super) vinfium: f64,
    pub(super) vsupremum: f64,
    pub(super) failure: FailureCapture,
}

impl<'a, T: ExtremaCurveTool> GFuncExtCC<'a, T> {
    /// `Extrema_GFuncExtCC(Tol)` (the native default is 1e-10): no curves, a null
    /// interval and the minimal tolerances; `GFuncExtCC::<CurveTool>::new(1.0e-10)`.
    pub fn new(tol: f64) -> Self {
        Self {
            c1: None,
            c2: None,
            tol,
            u: 0.0,
            v: 0.0,
            p1: T::Point::default(),
            p2: T::Point::default(),
            du: T::Vector::default(),
            dv: T::Vector::default(),
            sq_dist: Vec::new(),
            points: Vec::new(),
            tol_c1: MIN_TOL,
            tol_c2: MIN_TOL,
            max_order_c1: 0,
            max_order_c2: 0,
            uinfium: 0.0,
            usupremum: 0.0,
            vinfium: 0.0,
            vsupremum: 0.0,
            failure: FailureCapture::default(),
        }
    }
    /// `Extrema_GFuncExtCC(C1, C2, Tol)`: the intervals of both curves and, for the
    /// curves with a variable parametrization, their singularity tolerances;
    /// `GFuncExtCC::<CurveTool>::with_curves(&c1, &c2, 1.0e-10)?`.
    pub fn with_curves(c1: &'a T::Curve, c2: &'a T::Curve, tol: f64) -> Result<Self, Failure> {
        let mut function: Self = Self::new(tol);
        function.c1 = Some(c1);
        function.c2 = Some(c2);
        let mut first = Vector::with_bounds(1, 2)?;
        let mut last = Vector::with_bounds(1, 2)?;
        first[1] = T::first_parameter(c1)?;
        last[1] = T::last_parameter(c1)?;
        first[2] = T::first_parameter(c2)?;
        last[2] = T::last_parameter(c2)?;
        function.sub_interval_initialize(&first, &last);
        (function.max_order_c1, function.tol_c1) = function.singularity(c1)?;
        (function.max_order_c2, function.tol_c2) = function.singularity(c2)?;
        Ok(function)
    }
    // The maximal derivative order and tolerance of a curve's singular case.
    fn singularity(&self, curve: &T::Curve) -> Result<(i32, f64), Failure> {
        match T::get_type(curve)? {
            CurveType::BezierCurve
            | CurveType::BSplineCurve
            | CurveType::OffsetCurve
            | CurveType::OtherCurve => Ok((MAX_ORDER, self.search_of_tolerance(curve)?)),
            _ => Ok((0, MIN_TOL)),
        }
    }
    /// `SetCurve(theRank, C)`: Standard_OutOfRange for a rank other than 1 or 2;
    /// `function.set_curve(1, &c1)?`.
    pub fn set_curve(&mut self, rank: i32, curve: &'a T::Curve) -> Result<(), Failure> {
        if !(1..=2).contains(&rank) {
            return Err(out_of_range("Extrema_GFuncExtCC::SetCurve()", rank, 2));
        }
        if rank == 1 {
            self.c1 = Some(curve);
            (self.max_order_c1, self.tol_c1) = self.singularity(curve)?;
        } else {
            self.c2 = Some(curve);
            (self.max_order_c2, self.tol_c2) = self.singularity(curve)?;
        }
        Ok(())
    }
    /// `SetTolerance(theTol)`; `function.set_tolerance(1.0e-10)`.
    pub fn set_tolerance(&mut self, tol: f64) {
        self.tol = tol;
    }
    /// `NbVariables()`: 2; `function.nb_variables()`.
    pub fn nb_variables(&self) -> i32 {
        2
    }
    /// `NbEquations()`: 2; `function.nb_equations()`.
    pub fn nb_equations(&self) -> i32 {
        2
    }
    /// `CurvePtr(theRank)`: the curve of rank 1 or 2 (`None` before it is set);
    /// Standard_OutOfRange for another rank; `function.curve_ptr(1)?`.
    pub fn curve_ptr(&self, rank: i32) -> Result<Option<&'a T::Curve>, Failure> {
        if !(1..=2).contains(&rank) {
            return Err(out_of_range("Extrema_GFuncExtCC::CurvePtr()", rank, 2));
        }
        Ok(if rank == 1 { self.c1 } else { self.c2 })
    }
    /// `Tolerance()`; `function.tolerance()`.
    pub fn tolerance(&self) -> f64 {
        self.tol
    }
    /// `SubIntervalInitialize(theUfirst, theUlast)`: the `(U, V)` bounds of the
    /// finite differences; `function.sub_interval_initialize(&first, &last)`.
    pub fn sub_interval_initialize(&mut self, first: &Vector<'_>, last: &Vector<'_>) {
        self.uinfium = first[1];
        self.usupremum = last[1];
        self.vinfium = first[2];
        self.vsupremum = last[2];
    }
    /// `SearchOfTolerance(theC)`: `max(1e-12 max |C'|, 1e-20)` over 11 samples of the
    /// U interval when `curve` is the first curve (compared by address, as the native
    /// `void*`), of the V interval when it is the second, and 1e-20 otherwise;
    /// `function.search_of_tolerance(&c1)?`.
    pub fn search_of_tolerance(&self, curve: &T::Curve) -> Result<f64, Failure> {
        let (start, end) = if self.c1.is_some_and(|c| std::ptr::addr_eq(c, curve)) {
            (self.uinfium, self.usupremum)
        } else if self.c2.is_some_and(|c| std::ptr::addr_eq(c, curve)) {
            (self.vinfium, self.vsupremum)
        } else {
            return Ok(MIN_TOL);
        };
        let step: f64 = (end - start) / 10.0;
        let mut max: f64 = -precision::infinite();
        for num in 0..=10 {
            let mut u: f64 = start + f64::from(num) * step;
            if u > end {
                u = end;
            }
            let magnitude: f64 = T::d1(curve, u)?.1.norm();
            if magnitude > max {
                max = magnitude;
            }
        }
        Ok(std_max(max * TOL_FACTOR, MIN_TOL))
    }
    pub(super) fn curves(&self) -> Result<(&'a T::Curve, &'a T::Curve), Failure> {
        let missing = || null_object("Extrema_GFuncExtCC::Value()");
        Ok((self.c1.ok_or_else(missing)?, self.c2.ok_or_else(missing)?))
    }
}

impl<T: ExtremaCurveTool> GFuncExtCC<'_, T> {
    /// `Value(theUV, theF)`: false when a first derivative vanishes (after the
    /// singular replacement for spline-like curves); `function.value(&uv, &mut f)?`.
    pub fn value(&mut self, uv: &Vector<'_>, f: &mut Vector<'_>) -> Result<bool, Failure> {
        self.u = uv[1];
        self.v = uv[2];
        let (c1, c2) = self.curves()?;
        (self.p1, self.du) = T::d1(c1, self.u)?;
        (self.p2, self.dv) = T::d1(c2, self.v)?;
        let p1p2: T::Vector = self.p1.vector_to(&self.p2);
        let search1 = SingularSearch {
            uinf: self.uinfium,
            usup: self.usupremum,
            tol: self.tol_c1,
            max_order: self.max_order_c1,
        };
        let Some(ndu) = regular_norm::<T>(c1, self.u, &mut self.du, &search1)? else {
            return Ok(false);
        };
        let search2 = SingularSearch {
            uinf: self.vinfium,
            usup: self.vsupremum,
            tol: self.tol_c2,
            max_order: self.max_order_c2,
        };
        let Some(ndv) = regular_norm::<T>(c2, self.v, &mut self.dv, &search2)? else {
            return Ok(false);
        };
        f[1] = p1p2.dot_with(&self.du) / ndu;
        f[2] = p1p2.dot_with(&self.dv) / ndv;
        Ok(true)
    }
    /// `GetStateNumber()`: saves the current pair when both normalized projections
    /// are within the tolerance; `function.get_state_number()`.
    pub fn get_state_number(&mut self) -> i32 {
        let (mut du, mut dv) = (self.du, self.dv);
        let p1p2: T::Vector = self.p1.vector_to(&self.p2);
        let mut modulus: f64 = du.norm();
        if modulus > self.tol_c1 {
            du = du / modulus;
        }
        modulus = dv.norm();
        if modulus > self.tol_c2 {
            dv = dv / modulus;
        }
        if p1p2.dot_with(&du).abs() <= self.tol && p1p2.dot_with(&dv).abs() <= self.tol {
            self.sq_dist.push(self.p1.square_distance_to(&self.p2));
            self.points.push(POnCurve::new(self.u, self.p1));
            self.points.push(POnCurve::new(self.v, self.p2));
        }
        0
    }
    /// `NbExt()`; `function.nb_ext()`.
    pub fn nb_ext(&self) -> i32 {
        self.sq_dist.len() as i32
    }
    /// `SquareDistance(N)`; Standard_OutOfRange outside the saved extrema;
    /// `function.square_distance(1)?`.
    pub fn square_distance(&self, n: i32) -> Result<f64, Failure> {
        sequence_value("Extrema_GFuncExtCC::SquareDistance()", &self.sq_dist, n)
    }
    /// `Points(N, P1, P2)`, returned as `(P1, P2)`; `function.points(1)?`.
    // The native output pair (P1, P2) as one value.
    #[allow(clippy::type_complexity)]
    pub fn points(&self, n: i32) -> Result<(POnCurve<T::Point>, POnCurve<T::Point>), Failure> {
        let member: &str = "Extrema_GFuncExtCC::Points()";
        let first = sequence_value(member, &self.points, 2 * n - 1)?;
        Ok((first, sequence_value(member, &self.points, 2 * n)?))
    }
    /// The first failure a math callback kept since the last call;
    /// `function.take_failure()?`.
    pub fn take_failure(&mut self) -> Result<(), Failure> {
        self.failure.take()
    }
}

// The norm of the first derivative after the singular replacement of a spline-like
// curve, or None when it is not above Extrema's minimal tolerance.
fn regular_norm<T: ExtremaCurveTool>(
    curve: &T::Curve,
    u: f64,
    derivative: &mut T::Vector,
    search: &SingularSearch,
) -> Result<Option<f64>, Failure> {
    let mut norm: f64 = derivative.norm();
    if search.max_order != 0 && norm <= search.tol {
        *derivative = singular_derivative::<T>(curve, u, search)?;
        norm = derivative.norm();
    }
    if norm <= MIN_TOL {
        return Ok(None);
    }
    Ok(Some(norm))
}
