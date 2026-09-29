// Copyright (c) 2014-2014 OPEN CASCADE SAS
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
// Rust translation of ModelingData/TKGeomBase/GCPnts/GCPnts_DistFunction.hxx/.cxx and
// GCPnts_DistFunction2d.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GCPnts_DistFunction` and `GCPnts_DistFunction2d`: minus the square distance from
//! `C(u)`, `U1 <= u <= U2`, to the chord line through `C(U1)` and `C(U2)`.
//!
//! The two native classes have the same body over `gp_Lin` and `gp_Lin2d`; the port
//! has one generic class over the curve type and the native names as aliases.
use super::TCurveTypes;
use crate::failure_capture::FailureCapture;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::gp;
use rustycad_tkmath::math::{Function, MultipleVarFunction, Vector};

/// The chord deviation function of a curve type `C`
/// (`GCPnts_TCurveTypes<C>::DistFunction`); `TDistFunction::new(&adaptor, 0.0, 1.0)?`.
///
/// A curve evaluation that raises in [`Function::value`] is kept, reported as a
/// failed value, and read back with [`Self::take_failure`] after the optimizer.
pub struct TDistFunction<'a, C: TCurveTypes + ?Sized> {
    my_curve: &'a C,
    my_lin: C::Line,
    my_u1: f64,
    my_u2: f64,
    failure: FailureCapture,
}

/// `GCPnts_DistFunction` on an `Adaptor3d_Curve`.
pub type DistFunction<'a> = TDistFunction<'a, dyn Adaptor3dCurve>;
/// `GCPnts_DistFunction2d` on an `Adaptor2d_Curve2d`.
pub type DistFunction2d<'a> = TDistFunction<'a, dyn Adaptor2dCurve2d>;

impl<'a, C: TCurveTypes + ?Sized> TDistFunction<'a, C> {
    /// The function on `[u1, u2]` (`GCPnts_DistFunction(theCurve, U1, U2)`); when
    /// `C(U1)` and `C(U2)` coincide (square distance not above gp::Resolution()) the
    /// chord goes to `C(U1 + 0.01 (U2 - U1))` instead (OCCT #28812);
    /// Standard_ConstructionError when that point coincides too;
    /// `TDistFunction::new(&adaptor, 0.0, 1.0)?`.
    // The negated test keeps the native NaN behavior.
    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    pub fn new(curve: &'a C, u1: f64, u2: f64) -> Result<Self, Failure> {
        let p1: C::Point = curve.point_at(u1)?;
        let mut p2: C::Point = curve.point_at(u2)?;
        // Negated as natively, so a NaN distance also takes the #28812 branch.
        if !(C::point_square_distance(p1, p2) > gp::resolution()) {
            // For #28812
            p2 = curve.point_at(u1 + 0.01 * (u2 - u1))?;
        }
        Ok(Self {
            my_curve: curve,
            my_lin: C::chord_line(p1, p2)?,
            my_u1: u1,
            my_u2: u2,
            failure: FailureCapture::default(),
        })
    }
    /// `Err` with the first failure raised by the curve since the last call;
    /// `function.take_failure()?`.
    pub fn take_failure(&mut self) -> Result<(), Failure> {
        self.failure.take()
    }
}

/// `GCPnts_DistFunction(const GCPnts_DistFunction&)`: the same curve, chord and range.
impl<C: TCurveTypes + ?Sized> Clone for TDistFunction<'_, C> {
    fn clone(&self) -> Self {
        Self {
            my_curve: self.my_curve,
            my_lin: self.my_lin.clone(),
            my_u1: self.my_u1,
            my_u2: self.my_u2,
            failure: FailureCapture::default(),
        }
    }
}

impl<C: TCurveTypes + ?Sized> Function for TDistFunction<'_, C> {
    /// `Value(X, F)`: `-Lin.SquareDistance(C(X))`; false outside `[U1, U2]`.
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        if x < self.my_u1 || x > self.my_u2 {
            return false;
        }
        match self.failure.keep(self.my_curve.point_at(x)) {
            Some(point) => {
                *result = -C::line_square_distance(&self.my_lin, point);
                true
            }
            None => false,
        }
    }
}

/// The one-variable `math_MultipleVarFunction` view of a [`TDistFunction`]
/// (`GCPnts_TCurveTypes<C>::DistFunctionMV`), used by `math_PSO`;
/// `TDistFunctionMV::new(&mut function)`.
pub struct TDistFunctionMV<'f, 'a, C: TCurveTypes + ?Sized> {
    my_max_curv_lin_dist: &'f mut TDistFunction<'a, C>,
}

/// `GCPnts_DistFunctionMV` on an `Adaptor3d_Curve`.
pub type DistFunctionMV<'f, 'a> = TDistFunctionMV<'f, 'a, dyn Adaptor3dCurve>;
/// `GCPnts_DistFunction2dMV` on an `Adaptor2d_Curve2d`.
pub type DistFunction2dMV<'f, 'a> = TDistFunctionMV<'f, 'a, dyn Adaptor2dCurve2d>;

impl<'f, 'a, C: TCurveTypes + ?Sized> TDistFunctionMV<'f, 'a, C> {
    /// The view of `function` (`GCPnts_DistFunctionMV(theCurvLinDist)`);
    /// `TDistFunctionMV::new(&mut function)`.
    pub fn new(function: &'f mut TDistFunction<'a, C>) -> Self {
        Self {
            my_max_curv_lin_dist: function,
        }
    }
}

impl<C: TCurveTypes + ?Sized> MultipleVarFunction for TDistFunctionMV<'_, '_, C> {
    /// `NbVariables()`: 1.
    fn variables(&self) -> i32 {
        1
    }
    /// `Value(X, F)`: the function at `X(1)`.
    fn value(&mut self, point: &Vector<'_>, value: &mut f64) -> bool {
        self.my_max_curv_lin_dist.value(point[1], value)
    }
}
