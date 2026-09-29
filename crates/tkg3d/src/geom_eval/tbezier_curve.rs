// Copyright (c) 2025 OPEN CASCADE SAS
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
// Rust translation of ModelingData/TKG3d/GeomEval/GeomEval_TBezierCurve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomEval_TBezierCurve`: a trigonometric Bezier curve over the basis
//! `{1, sin(alpha t), cos(alpha t), ..., sin(n alpha t), cos(n alpha t)}` on `[0, Pi/alpha]`.
//!
//! The evaluation is the Geom2dEval algorithm with `gp_XYZ` sums; both run
//! `rustycad_tkg2d::geom2d_eval::eval_basis::tbezier`.
use super::curve_support::check_derivative_order;
use super::eval_failures::unsupported;
use crate::geom::errors;
use crate::geom::{
    GeomBoundedCurve, GeomCurve, GeomCurveD1, GeomCurveD2, GeomCurveD3, GeomGeometry,
    dump_bounded_curve,
};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg2d::geom2d::{dump_base, dump_class_name, dump_flag};
use rustycad_tkg2d::geom2d_eval::bezier_curve_data::{check_weights, owned};
use rustycad_tkg2d::geom2d_eval::eval_basis::sums::BasisSums;
use rustycad_tkg2d::geom2d_eval::eval_basis::tbezier::{
    TBezierView, curve_basis, non_rational_dn, rational_derivatives, rational_dn, rational_point,
};
use rustycad_tkmath::gp::{Point3, Transform3, Vector3, Xyz};
use std::f64::consts::PI;
use std::io;

const CLASS: &str = "GeomEval_TBezierCurve";

/// A rational or non-rational T-Bezier curve with 2n + 1 poles; immutable like every
/// eval curve (`Reverse` and `Transform` raise Standard_NotImplemented);
/// `Handle::new(GeomEvalTBezierCurve::new(&poles, 1.0)?)`.
pub struct GeomEvalTBezierCurve {
    parent: StandardTransient,
    poles: Array1<'static, Point3>,
    weights: Array1<'static, f64>,
    alpha: f64,
    rational: bool,
}
impl_transient!(
    GeomEvalTBezierCurve,
    "GeomEval_TBezierCurve",
    dyn GeomBoundedCurve,
    parent,
    interfaces: [dyn GeomGeometry, dyn GeomCurve, dyn GeomBoundedCurve]
);
impl Clone for GeomEvalTBezierCurve {
    fn clone(&self) -> Self {
        self.copied()
    }
}

// The checks both constructors share, in native order.
fn check_poles_and_alpha(count: usize, alpha: f64) -> Result<(), Failure> {
    if count < 3 || count.is_multiple_of(2) {
        return Err(errors::construction(&format!(
            "{CLASS}: {count} poles; expected an odd count >= 3"
        )));
    }
    if alpha <= 0.0 {
        return Err(errors::construction(&format!(
            "{CLASS}: alpha = {alpha}; expected > 0"
        )));
    }
    Ok(())
}

impl GeomEvalTBezierCurve {
    /// Non-rational curve (`GeomEval_TBezierCurve(Poles, Alpha)`); Standard_ConstructionError
    /// unless the pole count is odd and >= 3 and `alpha > 0`;
    /// `GeomEvalTBezierCurve::new(&poles, 1.0)?`.
    pub fn new(poles: &Array1<'_, Point3>, alpha: f64) -> Result<Self, Failure> {
        check_poles_and_alpha(poles.len(), alpha)?;
        Ok(Self::stored(poles, Array1::default(), alpha, false))
    }
    /// Rational curve (`GeomEval_TBezierCurve(Poles, Weights, Alpha)`); also refuses a
    /// weight count different from the pole count and weights <= 0. Equal weights still
    /// give a rational curve; `GeomEvalTBezierCurve::with_weights(&poles, &weights, 1.0)?`.
    pub fn with_weights(
        poles: &Array1<'_, Point3>,
        weights: &Array1<'_, f64>,
        alpha: f64,
    ) -> Result<Self, Failure> {
        check_poles_and_alpha(poles.len(), alpha)?;
        check_weights(CLASS, weights, poles.len())?;
        Ok(Self::stored(poles, owned(weights), alpha, true))
    }
    // The member initialization shared by both constructors.
    fn stored(
        poles: &Array1<'_, Point3>,
        weights: Array1<'static, f64>,
        alpha: f64,
        rational: bool,
    ) -> Self {
        Self {
            parent: StandardTransient::default(),
            poles: owned(poles),
            weights,
            alpha,
            rational,
        }
    }
    // Copy() reruns the matching constructor on valid stored values.
    fn copied(&self) -> Self {
        Self::stored(&self.poles, self.weights.clone(), self.alpha, self.rational)
    }
    /// A copy of the poles with their original bounds (`Poles`); `curve.poles().len()`.
    pub fn poles(&self) -> Array1<'static, Point3> {
        self.poles.clone()
    }
    /// A copy of the weights, empty for a non-rational curve (`Weights`); `curve.weights().len()`.
    pub fn weights(&self) -> Array1<'static, f64> {
        self.weights.clone()
    }
    /// The frequency parameter (`Alpha`); `curve.alpha()`.
    pub fn alpha(&self) -> f64 {
        self.alpha
    }
    /// The number of poles (`NbPoles`); `curve.nb_poles()`.
    pub fn nb_poles(&self) -> i32 {
        self.poles.len() as i32
    }
    /// The trigonometric order n, with NbPoles = 2n + 1 (`Order`); `curve.order()`.
    pub fn order(&self) -> i32 {
        (self.nb_poles() - 1) / 2
    }
    /// Whether the curve was built with weights (`IsRational`); `curve.is_rational()`.
    pub fn is_rational(&self) -> bool {
        self.rational
    }
    fn view(&self) -> TBezierView<'_, Point3> {
        TBezierView {
            poles: self.poles.as_slice(),
            weights: self.rational.then(|| self.weights.as_slice()),
            alpha: self.alpha,
        }
    }
    // The point and the first derivatives through the order-`MAX` sums.
    fn evaluate<const MAX: usize>(&self, u: f64) -> (Point3, [Vector3; 3]) {
        let sums: BasisSums<Xyz> = curve_basis::<MAX, Point3>(&self.view(), u);
        if !self.rational {
            let d = [sums.n[1], sums.n[2], sums.n[3]].map(Vector3::from);
            return (Point3::from(sums.n[0]), d);
        }
        let c: Xyz = rational_point(&sums);
        let d: [Xyz; 3] = rational_derivatives(&sums, c, MAX);
        (Point3::from(c), d.map(Vector3::from))
    }
}
impl GeomGeometry for GeomEvalTBezierCurve {
    fn transform(&self, _transform: Transform3) -> Result<(), Failure> {
        Err(unsupported("GeomEval_TBezierCurve::Transform", ""))
    }
    fn copy(&self) -> Result<Handle<dyn GeomGeometry>, Failure> {
        Ok(Handle::new(self.copied()).cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        dump_class_name(stream, CLASS)?;
        dump_base(stream, depth, dump_bounded_curve)?;
        dump::dump_real(stream, "Alpha", self.alpha)?;
        dump_flag(stream, "Rational", self.rational)?;
        dump::dump_integer(stream, "Poles.Length", self.nb_poles())
    }
}
impl GeomCurve for GeomEvalTBezierCurve {
    rustycad_tkg2d::unsupported_curve_reversal!(unsupported, "GeomEval_TBezierCurve");
    fn first_parameter(&self) -> f64 {
        0.0
    }
    fn last_parameter(&self) -> f64 {
        PI / self.alpha
    }
    fn is_closed(&self) -> Result<bool, Failure> {
        let distance: f64 = self.start_point()?.distance(&self.end_point()?);
        Ok(distance <= precision::confusion())
    }
    rustycad_tkg2d::non_periodic_cn_curve!();
    fn eval_d0(&self, u: f64) -> Result<Point3, Failure> {
        Ok(self.evaluate::<0>(u).0)
    }
    fn eval_d1(&self, u: f64) -> Result<GeomCurveD1, Failure> {
        let (point, [d1, _, _]) = self.evaluate::<1>(u);
        Ok(GeomCurveD1 { point, d1 })
    }
    fn eval_d2(&self, u: f64) -> Result<GeomCurveD2, Failure> {
        let (point, [d1, d2, _]) = self.evaluate::<2>(u);
        Ok(GeomCurveD2 { point, d1, d2 })
    }
    fn eval_d3(&self, u: f64) -> Result<GeomCurveD3, Failure> {
        let (point, [d1, d2, d3]) = self.evaluate::<3>(u);
        Ok(GeomCurveD3 { point, d1, d2, d3 })
    }
    // Non-rational curves differentiate the basis directly for every order; rational
    // ones delegate orders 1 to 3 to the explicit evaluators, as natively.
    fn eval_dn(&self, u: f64, n: i32) -> Result<Vector3, Failure> {
        check_derivative_order("GeomEval_TBezierCurve::EvalDN", n)?;
        let view: TBezierView<'_, Point3> = self.view();
        let Some(weights) = view.weights else {
            return Ok(Vector3::from(non_rational_dn(&view, u, n)));
        };
        Ok(match n {
            1 => self.eval_d1(u)?.d1,
            2 => self.eval_d2(u)?.d2,
            3 => self.eval_d3(u)?.d3,
            _ => Vector3::from(rational_dn(&view, weights, u, n)),
        })
    }
}
impl GeomBoundedCurve for GeomEvalTBezierCurve {
    fn end_point(&self) -> Result<Point3, Failure> {
        self.eval_d0(PI / self.alpha)
    }
    fn start_point(&self) -> Result<Point3, Failure> {
        self.eval_d0(0.0)
    }
}
