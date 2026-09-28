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
// Rust translation of ModelingData/TKG2d/Geom2dEval/Geom2dEval_TBezierCurve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dEval_TBezierCurve`: a trigonometric Bezier curve over the basis
//! `{1, sin(alpha t), cos(alpha t), ..., sin(n alpha t), cos(n alpha t)}` on `[0, Pi/alpha]`.
use super::eval_basis::sums::BasisSums;
use super::eval_basis::tbezier::{
    TBezierView, curve_basis, non_rational_dn, rational_derivatives, rational_dn, rational_point,
};
use super::eval_math::{check_derivative_order, unsupported};
use crate::geom2d::errors;
use crate::geom2d::{
    Geom2dBoundedCurve, Geom2dCurve, Geom2dCurveD1, Geom2dCurveD2, Geom2dCurveD3, Geom2dGeometry,
    dump_base, dump_bounded_curve, dump_class_name, dump_flag,
};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Transform2, Vector2, Xy};
use std::f64::consts::PI;
use std::io;

const CLASS: &str = "Geom2dEval_TBezierCurve";

/// A rational or non-rational T-Bezier curve with 2n + 1 poles; immutable like every
/// eval curve (`Reverse` and `Transform` raise Standard_NotImplemented);
/// `Handle::new(Geom2dEvalTBezierCurve::new(&poles, 1.0)?)`.
pub struct Geom2dEvalTBezierCurve {
    parent: StandardTransient,
    poles: Array1<'static, Point2>,
    weights: Array1<'static, f64>,
    alpha: f64,
    rational: bool,
}
impl_transient!(
    Geom2dEvalTBezierCurve,
    "Geom2dEval_TBezierCurve",
    dyn Geom2dBoundedCurve,
    parent,
    interfaces: [dyn Geom2dGeometry, dyn Geom2dCurve, dyn Geom2dBoundedCurve]
);
impl Clone for Geom2dEvalTBezierCurve {
    fn clone(&self) -> Self {
        self.copied()
    }
}

// The common checks of both constructors, in native order.
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

fn check_weights(weights: &Array1<'_, f64>, count: usize) -> Result<(), Failure> {
    if weights.len() != count {
        return Err(errors::construction(&format!(
            "{CLASS}: {} weights; expected {count} (one per pole)",
            weights.len()
        )));
    }
    let lower: i32 = weights.lower();
    for (offset, &weight) in weights.as_slice().iter().enumerate() {
        if weight <= 0.0 {
            let index: i64 = i64::from(lower) + offset as i64;
            return Err(errors::construction(&format!(
                "{CLASS}: weight({index}) = {weight}; expected every weight > 0"
            )));
        }
    }
    Ok(())
}

// An owning copy that keeps the source bounds, as the NCollection_Array1 copy constructor.
fn owned<T: Clone>(source: &Array1<'_, T>) -> Array1<'static, T> {
    let mut copy: Array1<'static, T> = Array1::default();
    copy.assign(source);
    copy
}

impl Geom2dEvalTBezierCurve {
    /// Non-rational curve (`Geom2dEval_TBezierCurve(Poles, Alpha)`); Standard_ConstructionError
    /// unless the pole count is odd and >= 3 and `alpha > 0`;
    /// `Geom2dEvalTBezierCurve::new(&poles, 1.0)?`.
    pub fn new(poles: &Array1<'_, Point2>, alpha: f64) -> Result<Self, Failure> {
        check_poles_and_alpha(poles.len(), alpha)?;
        Ok(Self {
            parent: StandardTransient::default(),
            poles: owned(poles),
            weights: Array1::default(),
            alpha,
            rational: false,
        })
    }
    /// Rational curve (`Geom2dEval_TBezierCurve(Poles, Weights, Alpha)`); also refuses a
    /// weight count different from the pole count and weights <= 0. Equal weights still
    /// give a rational curve; `Geom2dEvalTBezierCurve::with_weights(&poles, &weights, 1.0)?`.
    pub fn with_weights(
        poles: &Array1<'_, Point2>,
        weights: &Array1<'_, f64>,
        alpha: f64,
    ) -> Result<Self, Failure> {
        check_poles_and_alpha(poles.len(), alpha)?;
        check_weights(weights, poles.len())?;
        Ok(Self {
            parent: StandardTransient::default(),
            poles: owned(poles),
            weights: owned(weights),
            alpha,
            rational: true,
        })
    }
    // Copy() reruns the matching constructor on valid stored values.
    fn copied(&self) -> Self {
        Self {
            parent: StandardTransient::default(),
            poles: self.poles.clone(),
            weights: self.weights.clone(),
            alpha: self.alpha,
            rational: self.rational,
        }
    }
    /// A copy of the poles with their original bounds (`Poles`); `curve.poles().len()`.
    pub fn poles(&self) -> Array1<'static, Point2> {
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
    fn view(&self) -> TBezierView<'_, Point2> {
        TBezierView {
            poles: self.poles.as_slice(),
            weights: self.rational.then(|| self.weights.as_slice()),
            alpha: self.alpha,
        }
    }
    // The point and the first `count` derivatives through the order-`MAX` sums.
    fn evaluate<const MAX: usize>(&self, u: f64) -> (Point2, [Vector2; 3]) {
        let sums: BasisSums<Xy> = curve_basis::<MAX, Point2>(&self.view(), u);
        if !self.rational {
            let d = [sums.n[1], sums.n[2], sums.n[3]].map(Vector2::from);
            return (Point2::from(sums.n[0]), d);
        }
        let c: Xy = rational_point(&sums);
        let d: [Xy; 3] = rational_derivatives(&sums, c, MAX);
        (Point2::from(c), d.map(Vector2::from))
    }
}
impl Geom2dGeometry for Geom2dEvalTBezierCurve {
    fn transform(&self, _transform: Transform2) -> Result<(), Failure> {
        Err(unsupported("Geom2dEval_TBezierCurve::Transform", ""))
    }
    fn copy(&self) -> Result<Handle<dyn Geom2dGeometry>, Failure> {
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
impl Geom2dCurve for Geom2dEvalTBezierCurve {
    fn reverse(&self) -> Result<(), Failure> {
        Err(unsupported("Geom2dEval_TBezierCurve::Reverse", ""))
    }
    fn reversed_parameter(&self, u: f64) -> Result<f64, Failure> {
        let detail: String = format!("(U = {u})");
        Err(unsupported(
            "Geom2dEval_TBezierCurve::ReversedParameter",
            &detail,
        ))
    }
    fn first_parameter(&self) -> f64 {
        0.0
    }
    fn last_parameter(&self) -> f64 {
        PI / self.alpha
    }
    fn is_closed(&self) -> Result<bool, Failure> {
        let distance: f64 = self.start_point()?.distance(self.end_point()?);
        Ok(distance <= precision::confusion())
    }
    fn is_periodic(&self) -> bool {
        false
    }
    fn continuity(&self) -> Shape {
        Shape::CN
    }
    fn is_cn(&self, _n: i32) -> Result<bool, Failure> {
        Ok(true)
    }
    fn eval_d0(&self, u: f64) -> Result<Point2, Failure> {
        Ok(self.evaluate::<0>(u).0)
    }
    fn eval_d1(&self, u: f64) -> Result<Geom2dCurveD1, Failure> {
        let (point, [d1, _, _]) = self.evaluate::<1>(u);
        Ok(Geom2dCurveD1 { point, d1 })
    }
    fn eval_d2(&self, u: f64) -> Result<Geom2dCurveD2, Failure> {
        let (point, [d1, d2, _]) = self.evaluate::<2>(u);
        Ok(Geom2dCurveD2 { point, d1, d2 })
    }
    fn eval_d3(&self, u: f64) -> Result<Geom2dCurveD3, Failure> {
        let (point, [d1, d2, d3]) = self.evaluate::<3>(u);
        Ok(Geom2dCurveD3 { point, d1, d2, d3 })
    }
    // Non-rational curves differentiate the basis directly for every order; rational
    // ones delegate orders 1 to 3 to the explicit evaluators, as natively.
    fn eval_dn(&self, u: f64, n: i32) -> Result<Vector2, Failure> {
        check_derivative_order("Geom2dEval_TBezierCurve::EvalDN", n)?;
        let view: TBezierView<'_, Point2> = self.view();
        let Some(weights) = view.weights else {
            return Ok(Vector2::from(non_rational_dn(&view, u, n)));
        };
        Ok(match n {
            1 => self.eval_d1(u)?.d1,
            2 => self.eval_d2(u)?.d2,
            3 => self.eval_d3(u)?.d3,
            _ => Vector2::from(rational_dn(&view, weights, u, n)),
        })
    }
}
impl Geom2dBoundedCurve for Geom2dEvalTBezierCurve {
    fn end_point(&self) -> Result<Point2, Failure> {
        self.eval_d0(PI / self.alpha)
    }
    fn start_point(&self) -> Result<Point2, Failure> {
        self.eval_d0(0.0)
    }
}
