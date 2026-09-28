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
// Rust translation of ModelingData/TKG2d/Adaptor2d/Adaptor2d_OffsetCurve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Adaptor2d_OffsetCurve`: an adapted curve moved along its normal by a distance.
use super::curve2d::Adaptor2dCurve2d;
use super::{offset_curve_kind as kind, offset_curve_shape as shape};
use crate::geom2d::cell::GeometryCell;
use crate::geom2d::errors::{not_implemented, null_object};
use crate::geom2d::offset_curve_utils::{calculate_d0, calculate_d1, calculate_d2, calculate_d3};
use crate::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCurveD1, Geom2dCurveD2, Geom2dCurveD3,
};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::{Circle2, Ellipse2, Hyperbola2, Line2, Parabola2, Point2, Vector2};

#[derive(Clone, Default)]
pub(super) struct OffsetState {
    pub(super) curve: Handle<dyn Adaptor2dCurve2d>,
    pub(super) offset: f64,
    pub(super) first: f64,
    pub(super) last: f64,
}
impl OffsetState {
    // The basis adaptor; natively a null one is dereferenced.
    pub(super) fn basis(&self) -> Result<&dyn Adaptor2dCurve2d, Failure> {
        self.curve.get().ok_or_else(|| {
            null_object("Adaptor2d_OffsetCurve: the basis curve is null; expected a loaded curve")
        })
    }
}

/// An offset of an adapted curve on `[WFirst, WLast]` (`Adaptor2d_OffsetCurve`).
///
/// The point is `C(u) + Offset * N / |N|` with `N = (C'.Y, -C'.X)`, so a positive
/// offset of a curve running along +X moves it towards -Y;
/// `Adaptor2dOffsetCurve::with_offset(&basis, 3.0)?`.
#[derive(Default)]
pub struct Adaptor2dOffsetCurve {
    parent: StandardTransient,
    state: GeometryCell<OffsetState>,
}
impl_transient!(
    Adaptor2dOffsetCurve,
    "Adaptor2d_OffsetCurve",
    dyn Adaptor2dCurve2d,
    parent,
    interfaces: [dyn Adaptor2dCurve2d]
);

impl Adaptor2dOffsetCurve {
    fn from_state(state: OffsetState) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: GeometryCell::new(state),
        }
    }
    /// The curve with a null offset and bounds `[0, 0]` (`Adaptor2d_OffsetCurve(C)`);
    /// `Adaptor2dOffsetCurve::with_curve(&basis)`.
    pub fn with_curve(curve: &Handle<dyn Adaptor2dCurve2d>) -> Self {
        Self::with_bounds(curve, 0.0, 0.0, 0.0)
    }
    /// The curve offset by `offset` on the curve's own bounds
    /// (`Adaptor2d_OffsetCurve(C, Offset)`); Standard_NullObject for a null curve,
    /// which the native constructor dereferences; `Adaptor2dOffsetCurve::with_offset(&basis, 3.0)?`.
    pub fn with_offset(curve: &Handle<dyn Adaptor2dCurve2d>, offset: f64) -> Result<Self, Failure> {
        let adaptor = Self::with_curve(curve);
        adaptor.load_offset(offset)?;
        Ok(adaptor)
    }
    /// The curve offset by `offset` on `[w_first, w_last]`
    /// (`Adaptor2d_OffsetCurve(C, Offset, WFirst, WLast)`);
    /// `Adaptor2dOffsetCurve::with_bounds(&basis, 2.0, 1.0, 8.0)`.
    pub fn with_bounds(
        curve: &Handle<dyn Adaptor2dCurve2d>,
        offset: f64,
        w_first: f64,
        w_last: f64,
    ) -> Self {
        Self::from_state(OffsetState {
            curve: curve.clone(),
            offset,
            first: w_first,
            last: w_last,
        })
    }
    /// Change the curve and reset the offset to 0; the bounds are kept
    /// (`Load(occ::handle<Adaptor2d_Curve2d>)`); `adaptor.load_curve(&other)`.
    pub fn load_curve(&self, curve: &Handle<dyn Adaptor2dCurve2d>) {
        let mut state = self.state.write();
        state.curve = curve.clone();
        state.offset = 0.0;
    }
    /// Change the offset and take the curve's bounds (`Load(double)`);
    /// Standard_NullObject without a curve; `adaptor.load_offset(7.5)?`.
    pub fn load_offset(&self, offset: f64) -> Result<(), Failure> {
        // Native order: the offset is stored before the basis bounds are read, so a
        // basis that raises leaves the new offset with the old bounds.
        let curve: Handle<dyn Adaptor2dCurve2d> = {
            let mut state = self.state.write();
            state.offset = offset;
            state.curve.clone()
        };
        let basis = curve.get().ok_or_else(|| {
            null_object("Adaptor2d_OffsetCurve::Load: the basis curve is null; expected a curve")
        })?;
        let first: f64 = basis.first_parameter()?;
        self.state.write().first = first;
        let last: f64 = basis.last_parameter()?;
        self.state.write().last = last;
        Ok(())
    }
    /// Change the offset and the bounds (`Load(double, double, double)`);
    /// `adaptor.load_offset_bounded(4.0, 2.0, 6.0)`.
    pub fn load_offset_bounded(&self, offset: f64, w_first: f64, w_last: f64) {
        let mut state = self.state.write();
        state.offset = offset;
        state.first = w_first;
        state.last = w_last;
    }
    /// The basis curve (`Curve`); `adaptor.curve()`.
    pub fn curve(&self) -> Handle<dyn Adaptor2dCurve2d> {
        self.state.read().curve.clone()
    }
    /// The offset distance (`Offset`); `adaptor.offset()`.
    pub fn offset(&self) -> f64 {
        self.state.read().offset
    }
    // A copy of the state, so that no call into the basis holds this lock.
    pub(super) fn snapshot(&self) -> OffsetState {
        self.state.snapshot()
    }
}

impl Adaptor2dCurve2d for Adaptor2dOffsetCurve {
    fn shallow_copy(&self) -> Result<Handle<dyn Adaptor2dCurve2d>, Failure> {
        let mut state: OffsetState = self.snapshot();
        if let Some(basis) = state.curve.get() {
            state.curve = basis.shallow_copy()?;
        }
        Ok(Handle::new(Self::from_state(state)).cast())
    }
    fn first_parameter(&self) -> Result<f64, Failure> {
        Ok(self.state.read().first)
    }
    fn last_parameter(&self) -> Result<f64, Failure> {
        Ok(self.state.read().last)
    }
    fn continuity(&self) -> Result<Shape, Failure> {
        shape::continuity(&self.snapshot())
    }
    fn nb_intervals(&self, continuity: Shape) -> Result<i32, Failure> {
        shape::nb_intervals(&self.snapshot(), continuity)
    }
    fn intervals(&self, bounds: &mut Array1<'_, f64>, continuity: Shape) -> Result<(), Failure> {
        shape::intervals(&self.snapshot(), bounds, continuity)
    }
    fn trim(
        &self,
        first: f64,
        last: f64,
        _tolerance: f64,
    ) -> Result<Handle<dyn Adaptor2dCurve2d>, Failure> {
        // The copy constructor shares the basis adaptor; it is not a shallow copy.
        let trimmed = Self::from_state(self.snapshot());
        let offset: f64 = trimmed.offset();
        trimmed.load_offset_bounded(offset, first, last);
        Ok(Handle::new(trimmed).cast())
    }
    fn is_closed(&self) -> Result<bool, Failure> {
        kind::is_closed(&self.snapshot())
    }
    fn is_periodic(&self) -> Result<bool, Failure> {
        self.snapshot().basis()?.is_periodic()
    }
    fn period(&self) -> Result<f64, Failure> {
        self.snapshot().basis()?.period()
    }
    fn resolution(&self, r3d: f64) -> Result<f64, Failure> {
        Ok(precision::p_confusion_with_speed(r3d))
    }
    fn get_type(&self) -> Result<CurveType, Failure> {
        kind::get_type(&self.snapshot())
    }
    fn line(&self) -> Result<Line2, Failure> {
        kind::line(self)
    }
    fn circle(&self) -> Result<Circle2, Failure> {
        kind::circle(&self.snapshot())
    }
    fn ellipse(&self) -> Result<Ellipse2, Failure> {
        kind::ellipse(&self.snapshot())
    }
    fn hyperbola(&self) -> Result<Hyperbola2, Failure> {
        kind::hyperbola(&self.snapshot())
    }
    fn parabola(&self) -> Result<Parabola2, Failure> {
        kind::parabola(&self.snapshot())
    }
    fn degree(&self) -> Result<i32, Failure> {
        kind::degree(&self.snapshot())
    }
    fn is_rational(&self) -> Result<bool, Failure> {
        let state: OffsetState = self.snapshot();
        if state.offset == 0.0 {
            return state.basis()?.is_rational();
        }
        Ok(false)
    }
    fn nb_poles(&self) -> Result<i32, Failure> {
        kind::nb_poles(&self.snapshot())
    }
    fn nb_knots(&self) -> Result<i32, Failure> {
        kind::nb_knots(&self.snapshot())
    }
    fn nb_samples(&self) -> Result<i32, Failure> {
        kind::nb_samples(&self.snapshot())
    }
    fn bezier(&self) -> Result<Handle<Geom2dBezierCurve>, Failure> {
        kind::bezier(&self.snapshot())
    }
    fn bspline(&self) -> Result<Handle<Geom2dBSplineCurve>, Failure> {
        kind::bspline(&self.snapshot())
    }
    fn eval_d0(&self, u: f64) -> Result<Point2, Failure> {
        let state: OffsetState = self.snapshot();
        if state.offset == 0.0 {
            return state.basis()?.eval_d0(u);
        }
        // The CalculateD0 flag is ignored: a degenerate normal leaves the basis point.
        let basis: Geom2dCurveD1 = state.basis()?.eval_d1(u)?;
        let mut point: Point2 = basis.point;
        calculate_d0(&mut point, basis.d1, state.offset)?;
        Ok(point)
    }
    fn eval_d1(&self, u: f64) -> Result<Geom2dCurveD1, Failure> {
        let state: OffsetState = self.snapshot();
        if state.offset == 0.0 {
            return state.basis()?.eval_d1(u);
        }
        let basis: Geom2dCurveD2 = state.basis()?.eval_d2(u)?;
        let (mut point, mut d1) = (basis.point, basis.d1);
        calculate_d1(&mut point, &mut d1, basis.d2, state.offset);
        Ok(Geom2dCurveD1 { point, d1 })
    }
    fn eval_d2(&self, u: f64) -> Result<Geom2dCurveD2, Failure> {
        let state: OffsetState = self.snapshot();
        if state.offset == 0.0 {
            return state.basis()?.eval_d2(u);
        }
        let basis: Geom2dCurveD3 = state.basis()?.eval_d3(u)?;
        let (mut point, mut d1, mut d2) = (basis.point, basis.d1, basis.d2);
        calculate_d2(&mut point, &mut d1, &mut d2, basis.d3, false, state.offset);
        Ok(Geom2dCurveD2 { point, d1, d2 })
    }
    fn eval_d3(&self, u: f64) -> Result<Geom2dCurveD3, Failure> {
        let state: OffsetState = self.snapshot();
        if state.offset == 0.0 {
            return state.basis()?.eval_d3(u);
        }
        // Native order: the fourth derivative is evaluated before EvalD3.
        let d4: Vector2 = state.basis()?.eval_dn(u, 4)?;
        let mut result: Geom2dCurveD3 = state.basis()?.eval_d3(u)?;
        let derivatives = [&mut result.d1, &mut result.d2, &mut result.d3];
        calculate_d3(&mut result.point, derivatives, d4, false, state.offset);
        Ok(result)
    }
    fn eval_dn(&self, _u: f64, _n: i32) -> Result<Vector2, Failure> {
        Err(not_implemented("Adaptor2d_OffsetCurve::EvalDN"))
    }
}
