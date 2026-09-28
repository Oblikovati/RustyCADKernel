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
// Rust translation of Extrema_GFuncExtPC.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Extrema_GFuncExtPC`: `F(u) = (C(u) - P).C'(u) / |C'(u)|`, whose roots are the
//! extrema of the distance between a point and a curve, with the singular cases of a
//! vanishing first derivative (higher derivatives, then finite differences).
use crate::extrema::curve_tool::ExtremaCurveTool;
use crate::extrema::failures::{sequence_value, type_mismatch};
use crate::extrema::gp_space::{GpPoint, GpVector};
use crate::extrema::pon::POnCurve;
use crate::extrema::singular::{SingularSearch, singular_derivative, singular_step};
use crate::failure_capture::FailureCapture;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::native::std_max;
use rustycad_tkmath::geom_abs::CurveType;
use rustycad_tkmath::math::{Function, FunctionWithDerivative};

use crate::extrema::singular::{MAX_ORDER, MIN_TOL, TOL_FACTOR};

/// `Extrema_GFuncExtPC<TheCurve, TheCurveTool, ...>` over one [`ExtremaCurveTool`]:
/// the math function of the point-curve extrema. The native members raise; here
/// they return `Result`, and the math callbacks keep the first failure for the
/// algorithm that runs them ([`GFuncExtPC::take_failure`]);
/// `GFuncExtPC::<CurveTool>::with_point_curve(point, &adaptor)?`.
pub struct GFuncExtPC<'a, T: ExtremaCurveTool> {
    p: T::Point,
    curve: Option<&'a T::Curve>,
    u: f64,
    pc: T::Point,
    d1f: f64,
    sq_dist: Vec<f64>,
    is_min: Vec<i32>,
    points: Vec<POnCurve<T::Point>>,
    p_init: bool,
    c_init: bool,
    d1_init: bool,
    tol: f64,
    max_deriv_order: i32,
    uinfium: f64,
    usupremum: f64,
    failure: FailureCapture,
}

impl<T: ExtremaCurveTool> Default for GFuncExtPC<'_, T> {
    fn default() -> Self {
        Self {
            p: T::Point::default(),
            curve: None,
            u: 0.0,
            pc: T::Point::default(),
            d1f: 0.0,
            sq_dist: Vec::new(),
            is_min: Vec::new(),
            points: Vec::new(),
            p_init: false,
            c_init: false,
            d1_init: false,
            tol: MIN_TOL,
            max_deriv_order: 0,
            uinfium: 0.0,
            usupremum: 0.0,
            failure: FailureCapture::default(),
        }
    }
}

impl<'a, T: ExtremaCurveTool> GFuncExtPC<'a, T> {
    /// `Extrema_GFuncExtPC()`: neither point nor curve; `GFuncExtPC::<CurveTool>::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Extrema_GFuncExtPC(P, C)`; `GFuncExtPC::<CurveTool>::with_point_curve(point, &adaptor)?`.
    pub fn with_point_curve(point: T::Point, curve: &'a T::Curve) -> Result<Self, Failure> {
        let mut function: Self = Self {
            p: point,
            p_init: true,
            ..Self::default()
        };
        function.set_curve(curve)?;
        Ok(function)
    }
    /// `Initialize(C)`: sets the curve and clears the extrema;
    /// `function.initialize(&adaptor)?`.
    pub fn initialize(&mut self, curve: &'a T::Curve) -> Result<(), Failure> {
        self.points.clear();
        self.sq_dist.clear();
        self.is_min.clear();
        self.set_curve(curve)
    }
    // The curve part of the constructor and of Initialize: the interval and, for the
    // curves with a variable parametrization, the singularity tolerance.
    fn set_curve(&mut self, curve: &'a T::Curve) -> Result<(), Failure> {
        self.curve = Some(curve);
        self.c_init = true;
        self.sub_interval_initialize(T::first_parameter(curve)?, T::last_parameter(curve)?);
        match T::get_type(curve)? {
            CurveType::BezierCurve
            | CurveType::BSplineCurve
            | CurveType::OffsetCurve
            | CurveType::OtherCurve => {
                self.max_deriv_order = MAX_ORDER;
                self.tol = self.search_of_tolerance()?;
            }
            _ => {
                self.max_deriv_order = 0;
                self.tol = MIN_TOL;
            }
        }
        Ok(())
    }
    /// `SetPoint(P)`: sets the point and clears the extrema; `function.set_point(point)`.
    pub fn set_point(&mut self, point: T::Point) {
        self.p = point;
        self.p_init = true;
        self.points.clear();
        self.sq_dist.clear();
        self.is_min.clear();
    }
    /// `SubIntervalInitialize(Ufirst, Ulast)`; `function.sub_interval_initialize(0.0, 1.0)`.
    pub fn sub_interval_initialize(&mut self, ufirst: f64, ulast: f64) {
        self.uinfium = ufirst;
        self.usupremum = ulast;
    }
    /// The curve (`*myC`), Standard_NullObject before one is set.
    fn curve(&self) -> Result<&'a T::Curve, Failure> {
        self.curve
            .ok_or_else(|| crate::extrema::failures::null_object("Extrema_GFuncExtPC"))
    }
    fn check_init(&self, member: &str) -> Result<(), Failure> {
        if self.p_init && self.c_init {
            return Ok(());
        }
        Err(type_mismatch(&format!("Extrema_GFuncExtPC::{member}")))
    }
    /// `SearchOfTolerance()`: `max(1e-12 max |C'(u)|, 1e-20)` over 11 samples of the
    /// interval, skipping derivatives with an infinite X or Y;
    /// `function.search_of_tolerance()?`.
    pub fn search_of_tolerance(&self) -> Result<f64, Failure> {
        let curve: &T::Curve = self.curve()?;
        let step: f64 = (self.usupremum - self.uinfium) / 10.0;
        let mut max: f64 = -precision::infinite();
        for num in 0..=10 {
            let mut u: f64 = self.uinfium + f64::from(num) * step;
            if u > self.usupremum {
                u = self.usupremum;
            }
            let (_, derivative) = T::d1(curve, u)?;
            if is_infinite_xy(&derivative) {
                continue;
            }
            let magnitude: f64 = derivative.norm();
            if magnitude > max {
                max = magnitude;
            }
        }
        Ok(std_max(max * TOL_FACTOR, MIN_TOL))
    }
}

// `Precision::IsInfinite(V.X()) || Precision::IsInfinite(V.Y())` (Z is not checked).
fn is_infinite_xy<V: GpVector>(vector: &V) -> bool {
    precision::is_infinite(vector.coordinate(0)) || precision::is_infinite(vector.coordinate(1))
}

impl<'a, T: ExtremaCurveTool> GFuncExtPC<'a, T> {
    /// `Value(U, F)`: false (F unchanged) when the derivative vanishes; F is
    /// Precision::Infinite() when it is infinite; Standard_TypeMismatch without a point
    /// or curve; `function.value(0.5, &mut f)?`.
    pub fn value(&mut self, u: f64, f: &mut f64) -> Result<bool, Failure> {
        self.check_init("Value")?;
        self.u = u;
        let (pc, mut d1c) = T::d1(self.curve()?, self.u)?;
        self.pc = pc;
        if is_infinite_xy(&d1c) {
            *f = precision::infinite();
            return Ok(false);
        }
        let mut ndu: f64 = d1c.norm();
        if self.max_deriv_order != 0 && ndu <= self.tol {
            // Singular case
            d1c = self.singular_derivative()?;
            ndu = d1c.norm();
        }
        if ndu <= MIN_TOL {
            // Warning: 1st derivative is equal to zero!
            return Ok(false);
        }
        *f = self.p.vector_to(&self.pc).dot_with(&d1c) / ndu;
        Ok(true)
    }

    // The derivative of the singular case, on this function's interval.
    fn singular_derivative(&self) -> Result<T::Vector, Failure> {
        let search = SingularSearch {
            uinf: self.uinfium,
            usup: self.usupremum,
            tol: self.tol,
            max_order: self.max_deriv_order,
        };
        singular_derivative::<T>(self.curve()?, self.u, &search)
    }
}

impl<'a, T: ExtremaCurveTool> GFuncExtPC<'a, T> {
    /// `Derivative(U, DF)`; `function.derivative(0.5, &mut df)?`.
    pub fn derivative(&mut self, u: f64, df: &mut f64) -> Result<bool, Failure> {
        self.check_init("Derivative")?;
        let mut f: f64 = 0.0;
        self.values(u, &mut f, df)
    }
    /// `Values(U, F, DF)`: `DF = |C'| + (C - P).C''/|C'| - F C'.C''/|C'|²`, or a
    /// three-point difference of F when `|C'|` is below the tolerance (the point and
    /// parameter are then restored to their values before the call);
    /// `function.values(0.5, &mut f, &mut df)?`.
    pub fn values(&mut self, u: f64, f: &mut f64, df: &mut f64) -> Result<bool, Failure> {
        self.check_init("Values")?;
        let (pc_old, p_old) = (self.pc, self.p);
        if !self.value(u, f)? {
            self.d1_init = false;
            return Ok(false);
        }
        self.u = u;
        self.pc = pc_old;
        self.p = p_old;
        let (pc, d1c, d2c) = T::d2(self.curve()?, self.u)?;
        self.pc = pc;
        let ndu: f64 = d1c.norm();
        if ndu <= self.tol {
            // Singular case: the derivative is approximated by three points.
            let Some(derivative) = self.singular_values(u, *f)? else {
                self.d1_init = false;
                return Ok(false);
            };
            *df = derivative;
            self.u = u;
            self.pc = pc_old;
            self.p = p_old;
        } else {
            let ppc: T::Vector = self.p.vector_to(&self.pc);
            *df = ndu + (ppc.dot_with(&d2c) / ndu) - *f * (d1c.dot_with(&d2c)) / (ndu * ndu);
        }
        self.d1f = *df;
        self.d1_init = true;
        Ok(true)
    }

    // The three-point difference of F; None when one of the values fails.
    fn singular_values(&mut self, u: f64, f: f64) -> Result<Option<f64>, Failure> {
        let delta: f64 = singular_step(self.uinfium, self.usupremum, 0.01);
        let (mut f1, mut f2, mut f3): (f64, f64, f64) = (f, 0.0, f);
        if u - self.uinfium < 2.0 * delta {
            if !(self.value(u + delta, &mut f2)? && self.value(u + delta * 2.0, &mut f3)?) {
                return Ok(None);
            }
            return Ok(Some((-3.0 * f1 + 4.0 * f2 - f3) / (2.0 * delta)));
        }
        if !(self.value(u - delta, &mut f2)? && self.value(u - delta * 2.0, &mut f1)?) {
            return Ok(None);
        }
        Ok(Some((f1 - 4.0 * f2 + 3.0 * f3) / (2.0 * delta)))
    }

    /// `GetStateNumber()`: saves the current point as an extremum, a minimum when the
    /// recomputed derivative is positive; `function.get_state_number()?`.
    pub fn get_state_number(&mut self) -> Result<i32, Failure> {
        self.check_init("GetStateNumber")?;
        self.sq_dist.push(self.pc.square_distance_to(&self.p));
        // It is necessary to always compute myD1f.
        self.d1_init = true;
        let (mut ff, mut dd) = (0.0, 0.0);
        self.values(self.u, &mut ff, &mut dd)?;
        self.is_min.push(i32::from(self.d1f > 0.0));
        self.points.push(POnCurve::new(self.u, self.pc));
        Ok(0)
    }
    /// `NbExt()`; `function.nb_ext()`.
    pub fn nb_ext(&self) -> i32 {
        self.sq_dist.len() as i32
    }
    /// `SquareDistance(N)`; Standard_OutOfRange outside the saved extrema;
    /// `function.square_distance(1)?`.
    pub fn square_distance(&self, n: i32) -> Result<f64, Failure> {
        self.check_init("SquareDistance")?;
        sequence_value("Extrema_GFuncExtPC::SquareDistance()", &self.sq_dist, n)
    }
    /// `IsMin(N)`; `function.is_min(1)?`.
    pub fn is_min(&self, n: i32) -> Result<bool, Failure> {
        self.check_init("IsMin")?;
        Ok(sequence_value("Extrema_GFuncExtPC::IsMin()", &self.is_min, n)? == 1)
    }
    /// `Point(N)`; `function.point(1)?`.
    pub fn point(&self, n: i32) -> Result<POnCurve<T::Point>, Failure> {
        self.check_init("Point")?;
        sequence_value("Extrema_GFuncExtPC::Point()", &self.points, n)
    }
    /// The first failure a math callback kept since the last call; the algorithms
    /// return it where the native exception would leave them;
    /// `function.take_failure()?`.
    pub fn take_failure(&mut self) -> Result<(), Failure> {
        self.failure.take()
    }
}

impl<T: ExtremaCurveTool> Function for GFuncExtPC<'_, T> {
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        let value = GFuncExtPC::value(self, x, result);
        self.failure.keep(value).unwrap_or(false)
    }
    fn state_number(&mut self) -> i32 {
        let state = self.get_state_number();
        self.failure.keep(state).unwrap_or(0)
    }
}

impl<T: ExtremaCurveTool> FunctionWithDerivative for GFuncExtPC<'_, T> {
    fn derivative(&mut self, x: f64, derivative: &mut f64) -> bool {
        let value = GFuncExtPC::derivative(self, x, derivative);
        self.failure.keep(value).unwrap_or(false)
    }
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        let result = GFuncExtPC::values(self, x, value, derivative);
        self.failure.keep(result).unwrap_or(false)
    }
}
