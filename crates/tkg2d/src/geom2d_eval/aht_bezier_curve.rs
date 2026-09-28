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
// Rust translation of ModelingData/TKG2d/Geom2dEval/Geom2dEval_AHTBezierCurve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dEval_AHTBezierCurve`: an algebraic-hyperbolic-trigonometric Bezier curve
//! over `{1, t, ..., t^k, sinh(alpha t), cosh(alpha t), sin(beta t), cos(beta t)}` on `[0, 1]`.
mod derivative_n;
mod evaluation;
use super::basis_sums::BasisSums;
use super::eval_math::{check_derivative_order, unsupported};
use crate::geom2d::errors;
use crate::geom2d::{
    Geom2dBoundedCurve, Geom2dCurve, Geom2dCurveD1, Geom2dCurveD2, Geom2dCurveD3, Geom2dGeometry,
    dump_base, dump_bounded_curve, dump_class_name, dump_field, dump_flag,
};
use evaluation::{AhtView, mixed, polynomial, quotient_d1, quotient_d2, quotient_d3};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Transform2, Vector2, Xy};
use std::io;

const CLASS: &str = "Geom2dEval_AHTBezierCurve";

/// A rational or non-rational AHT-Bezier curve; immutable like every eval curve
/// (`Reverse` and `Transform` raise Standard_NotImplemented);
/// `Handle::new(Geom2dEvalAHTBezierCurve::new(&poles, 1, 1.0, 1.0)?)`.
pub struct Geom2dEvalAHTBezierCurve {
    parent: StandardTransient,
    poles: Array1<'static, Point2>,
    weights: Array1<'static, f64>,
    alg_degree: i32,
    alpha: f64,
    beta: f64,
    rational: bool,
}
impl_transient!(
    Geom2dEvalAHTBezierCurve,
    "Geom2dEval_AHTBezierCurve",
    dyn Geom2dBoundedCurve,
    parent,
    interfaces: [dyn Geom2dGeometry, dyn Geom2dCurve, dyn Geom2dBoundedCurve]
);
impl Clone for Geom2dEvalAHTBezierCurve {
    fn clone(&self) -> Self {
        self.copied()
    }
}

/// The number of basis functions (`basisDimension`): `k + 1`, plus 2 for a hyperbolic
/// pair when alpha > 0 and 2 for a trigonometric pair when beta > 0;
/// `basis_dimension(1, 1.0, 1.0) == 6`.
pub(crate) fn basis_dimension(alg_degree: i32, alpha: f64, beta: f64) -> i32 {
    alg_degree + 1 + if alpha > 0.0 { 2 } else { 0 } + if beta > 0.0 { 2 } else { 0 }
}

// The checks both constructors share, in native order.
fn check_basis(count: usize, alg_degree: i32, alpha: f64, beta: f64) -> Result<(), Failure> {
    if alg_degree < 0 {
        return Err(errors::construction(&format!(
            "{CLASS}: algebraic degree = {alg_degree}; expected >= 0"
        )));
    }
    if alpha < 0.0 {
        return Err(errors::construction(&format!(
            "{CLASS}: alpha = {alpha}; expected >= 0"
        )));
    }
    if beta < 0.0 {
        return Err(errors::construction(&format!(
            "{CLASS}: beta = {beta}; expected >= 0"
        )));
    }
    let dimension: i32 = basis_dimension(alg_degree, alpha, beta);
    if count as i64 != i64::from(dimension) {
        return Err(errors::construction(&format!(
            "{CLASS}: {count} poles; expected basisDimension = {dimension}"
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

fn owned<T: Clone>(source: &Array1<'_, T>) -> Array1<'static, T> {
    let mut copy: Array1<'static, T> = Array1::default();
    copy.assign(source);
    copy
}

impl Geom2dEvalAHTBezierCurve {
    /// Non-rational curve (`Geom2dEval_AHTBezierCurve(Poles, AlgDegree, Alpha, Beta)`);
    /// Standard_ConstructionError unless `alg_degree >= 0`, `alpha >= 0`, `beta >= 0` and
    /// the pole count equals the basis dimension.
    ///
    /// The native constructor stores `myWeights(1, 1)`, one uninitialized double; the
    /// port stores 0.0 there; `Geom2dEvalAHTBezierCurve::new(&poles, 2, 0.0, 0.0)?`.
    pub fn new(
        poles: &Array1<'_, Point2>,
        alg_degree: i32,
        alpha: f64,
        beta: f64,
    ) -> Result<Self, Failure> {
        check_basis(poles.len(), alg_degree, alpha, beta)?;
        let weights = Array1::with_bounds(1, 1).map_err(errors::array_bounds)?;
        Ok(Self::stored(
            poles,
            weights,
            alg_degree,
            [alpha, beta],
            false,
        ))
    }
    /// Rational curve (`Geom2dEval_AHTBezierCurve(Poles, Weights, AlgDegree, Alpha, Beta)`);
    /// also refuses a weight count different from the pole count and weights <= 0;
    /// `Geom2dEvalAHTBezierCurve::with_weights(&poles, &weights, 2, 0.0, 0.0)?`.
    pub fn with_weights(
        poles: &Array1<'_, Point2>,
        weights: &Array1<'_, f64>,
        alg_degree: i32,
        alpha: f64,
        beta: f64,
    ) -> Result<Self, Failure> {
        check_basis(poles.len(), alg_degree, alpha, beta)?;
        check_weights(weights, poles.len())?;
        Ok(Self::stored(
            poles,
            owned(weights),
            alg_degree,
            [alpha, beta],
            true,
        ))
    }
    // The member initialization shared by both constructors.
    fn stored(
        poles: &Array1<'_, Point2>,
        weights: Array1<'static, f64>,
        alg_degree: i32,
        [alpha, beta]: [f64; 2],
        rational: bool,
    ) -> Self {
        Self {
            parent: StandardTransient::default(),
            poles: owned(poles),
            weights,
            alg_degree,
            alpha,
            beta,
            rational,
        }
    }
    // Copy() reruns the matching constructor on valid stored values.
    fn copied(&self) -> Self {
        let (weights, frequencies) = (self.weights.clone(), [self.alpha, self.beta]);
        Self::stored(
            &self.poles,
            weights,
            self.alg_degree,
            frequencies,
            self.rational,
        )
    }
    /// A copy of the poles with their original bounds (`Poles`); `curve.poles().len()`.
    pub fn poles(&self) -> Array1<'static, Point2> {
        self.poles.clone()
    }
    /// A copy of the weights (`Weights`); one element for a non-rational curve, whose
    /// native value is uninitialized; `curve.weights().len()`.
    pub fn weights(&self) -> Array1<'static, f64> {
        self.weights.clone()
    }
    /// The algebraic polynomial degree (`AlgDegree`); `curve.alg_degree()`.
    pub fn alg_degree(&self) -> i32 {
        self.alg_degree
    }
    /// The hyperbolic frequency, 0 without hyperbolic terms (`Alpha`); `curve.alpha()`.
    pub fn alpha(&self) -> f64 {
        self.alpha
    }
    /// The trigonometric frequency, 0 without trigonometric terms (`Beta`); `curve.beta()`.
    pub fn beta(&self) -> f64 {
        self.beta
    }
    /// The number of poles (`NbPoles`); `curve.nb_poles()`.
    pub fn nb_poles(&self) -> i32 {
        self.poles.len() as i32
    }
    /// Whether the curve was built with weights (`IsRational`); `curve.is_rational()`.
    pub fn is_rational(&self) -> bool {
        self.rational
    }
    fn view(&self) -> AhtView<'_> {
        AhtView {
            poles: self.poles.as_slice(),
            weights: self.rational.then(|| self.weights.as_slice()),
            alg_degree: self.alg_degree as usize,
            alpha: self.alpha,
            beta: self.beta,
        }
    }
    // The purely algebraic basis uses Horner's scheme; any pair uses the direct sums.
    fn sums<const MAX: usize>(&self, u: f64) -> BasisSums {
        if self.alpha <= 0.0 && self.beta <= 0.0 {
            return polynomial::<MAX>(&self.view(), u);
        }
        mixed::<MAX>(&self.view(), u)
    }
}
impl Geom2dGeometry for Geom2dEvalAHTBezierCurve {
    fn transform(&self, _transform: Transform2) -> Result<(), Failure> {
        Err(unsupported("Geom2dEval_AHTBezierCurve::Transform", ""))
    }
    fn copy(&self) -> Result<Handle<dyn Geom2dGeometry>, Failure> {
        Ok(Handle::new(self.copied()).cast())
    }
    // Each pole is dumped under the key "Pole" (DumpFieldToName("&aPole")).
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        dump_class_name(stream, CLASS)?;
        dump_base(stream, depth, dump_bounded_curve)?;
        dump::dump_integer(stream, "AlgDegree", self.alg_degree)?;
        dump::dump_real(stream, "Alpha", self.alpha)?;
        dump::dump_real(stream, "Beta", self.beta)?;
        dump_flag(stream, "Rational", self.rational)?;
        for pole in self.poles.as_slice() {
            dump_field(stream, depth, "Pole", |nested, _| pole.dump_json(nested))?;
        }
        Ok(())
    }
}
impl Geom2dCurve for Geom2dEvalAHTBezierCurve {
    fn reverse(&self) -> Result<(), Failure> {
        Err(unsupported("Geom2dEval_AHTBezierCurve::Reverse", ""))
    }
    fn reversed_parameter(&self, u: f64) -> Result<f64, Failure> {
        let detail: String = format!("(U = {u})");
        Err(unsupported(
            "Geom2dEval_AHTBezierCurve::ReversedParameter",
            &detail,
        ))
    }
    fn first_parameter(&self) -> f64 {
        0.0
    }
    fn last_parameter(&self) -> f64 {
        1.0
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
        let sums: BasisSums = self.sums::<0>(u);
        if !self.rational {
            return Ok(Point2::from(sums.n[0]));
        }
        Ok(Point2::from(sums.n[0] / sums.w[0]))
    }
    fn eval_d1(&self, u: f64) -> Result<Geom2dCurveD1, Failure> {
        let sums: BasisSums = self.sums::<1>(u);
        let [c, d1, _, _] = sums.n;
        let (c, d1) = if self.rational {
            quotient_d1(&sums)
        } else {
            (c, d1)
        };
        Ok(Geom2dCurveD1 {
            point: Point2::from(c),
            d1: Vector2::from(d1),
        })
    }
    fn eval_d2(&self, u: f64) -> Result<Geom2dCurveD2, Failure> {
        let sums: BasisSums = self.sums::<2>(u);
        let [mut c, mut d1, mut d2, _] = sums.n;
        if self.rational {
            (c, d1) = quotient_d1(&sums);
            d2 = quotient_d2(&sums, c, d1);
        }
        let [d1, d2] = [d1, d2].map(Vector2::from);
        Ok(Geom2dCurveD2 {
            point: Point2::from(c),
            d1,
            d2,
        })
    }
    fn eval_d3(&self, u: f64) -> Result<Geom2dCurveD3, Failure> {
        let sums: BasisSums = self.sums::<3>(u);
        let mut values: [Xy; 4] = sums.n;
        if self.rational {
            values = quotient_d3(&sums);
        }
        let [c, d1, d2, d3] = values;
        let [d1, d2, d3] = [d1, d2, d3].map(Vector2::from);
        Ok(Geom2dCurveD3 {
            point: Point2::from(c),
            d1,
            d2,
            d3,
        })
    }
    // Orders 1 to 3 delegate to the explicit evaluators, as natively.
    fn eval_dn(&self, u: f64, n: i32) -> Result<Vector2, Failure> {
        check_derivative_order("Geom2dEval_AHTBezierCurve::EvalDN", n)?;
        match n {
            1 => return Ok(self.eval_d1(u)?.d1),
            2 => return Ok(self.eval_d2(u)?.d2),
            3 => return Ok(self.eval_d3(u)?.d3),
            _ => {}
        }
        let view: AhtView<'_> = self.view();
        let value: Xy = match view.weights {
            None => derivative_n::non_rational_dn(&view, u, n),
            Some(weights) => derivative_n::rational_dn(&view, weights, u, n),
        };
        Ok(Vector2::from(value))
    }
}
impl Geom2dBoundedCurve for Geom2dEvalAHTBezierCurve {
    fn end_point(&self) -> Result<Point2, Failure> {
        self.eval_d0(1.0)
    }
    fn start_point(&self) -> Result<Point2, Failure> {
        self.eval_d0(0.0)
    }
}
