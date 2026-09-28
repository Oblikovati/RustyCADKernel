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
// Rust translation of ModelingData/TKG2d/Geom2dEval/Geom2dEval_LogarithmicSpiralCurve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dEval_LogarithmicSpiralCurve`: `C(t) = O + a*e^(b*t)*(cos(t)*XDir + sin(t)*YDir)`.
use super::analytic_dump::dump_analytic_curve;
use super::eval_math::{LocalFrame, check_derivative_order, unsupported};
use crate::geom2d::errors;
use crate::geom2d::{Geom2dCurve, Geom2dCurveD1, Geom2dCurveD2, Geom2dCurveD3, Geom2dGeometry};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Axis2d, Point2, Transform2, Vector2};
use std::io;

const CLASS: &str = "Geom2dEval_LogarithmicSpiralCurve";

/// A logarithmic spiral on `(-inf, +inf)`, neither closed nor periodic; immutable
/// like every eval curve; `Handle::new(Geom2dEvalLogarithmicSpiralCurve::new(axis, 1.0, 0.1)?)`.
pub struct Geom2dEvalLogarithmicSpiralCurve {
    parent: StandardTransient,
    position: Axis2d,
    scale: f64,
    growth_exponent: f64,
}
impl_transient!(
    Geom2dEvalLogarithmicSpiralCurve,
    "Geom2dEval_LogarithmicSpiralCurve",
    dyn Geom2dCurve,
    parent,
    interfaces: [dyn Geom2dGeometry, dyn Geom2dCurve]
);
impl Clone for Geom2dEvalLogarithmicSpiralCurve {
    fn clone(&self) -> Self {
        self.copied()
    }
}

/// `(b + i)^n` by binary powering in the native operation order (`powComplexInt`);
/// `pow_complex_int(0.5, 1.0, 1) == (0.5, 1.0)`.
fn pow_complex_int(base_re: f64, base_im: f64, exponent: i32) -> (f64, f64) {
    let (mut res_re, mut res_im) = (1.0_f64, 0.0_f64);
    let (mut pow_re, mut pow_im) = (base_re, base_im);
    let mut exp: i32 = exponent;
    while exp > 0 {
        if exp & 1 != 0 {
            let tmp_re: f64 = res_re * pow_re - res_im * pow_im;
            let tmp_im: f64 = res_re * pow_im + res_im * pow_re;
            (res_re, res_im) = (tmp_re, tmp_im);
        }
        let next_re: f64 = pow_re * pow_re - pow_im * pow_im;
        let next_im: f64 = 2.0 * pow_re * pow_im;
        (pow_re, pow_im) = (next_re, next_im);
        exp >>= 1;
    }
    (res_re, res_im)
}

// `a e^(b t)`, `b` and the trigonometric values shared by the evaluators.
struct SpiralTerms {
    exp: f64,
    b: f64,
    cos: f64,
    sin: f64,
}
impl SpiralTerms {
    fn point(&self, frame: &LocalFrame) -> Point2 {
        frame.point(self.exp * self.cos, self.exp * self.sin)
    }
    fn d1(&self, frame: &LocalFrame) -> Vector2 {
        let (e, b, c, s) = (self.exp, self.b, self.cos, self.sin);
        frame.vector(e * (b * c - s), e * (b * s + c))
    }
    fn d2(&self, frame: &LocalFrame) -> Vector2 {
        let (e, b, c, s) = (self.exp, self.b, self.cos, self.sin);
        let b2: f64 = b * b;
        frame.vector(
            e * ((b2 - 1.0) * c - 2.0 * b * s),
            e * ((b2 - 1.0) * s + 2.0 * b * c),
        )
    }
    fn d3(&self, frame: &LocalFrame) -> Vector2 {
        let (e, b, c, s) = (self.exp, self.b, self.cos, self.sin);
        let b2: f64 = b * b;
        let b3: f64 = b2 * b;
        frame.vector(
            e * ((b3 - 3.0 * b) * c - (3.0 * b2 - 1.0) * s),
            e * ((b3 - 3.0 * b) * s + (3.0 * b2 - 1.0) * c),
        )
    }
    // `a e^(b t) (b + i)^N (cos t + i sin t)`.
    fn dn(&self, frame: &LocalFrame, n: i32) -> Vector2 {
        let (pow_re, pow_im) = pow_complex_int(self.b, 1.0, n);
        let coeff_x: f64 = pow_re * self.cos - pow_im * self.sin;
        let coeff_y: f64 = pow_re * self.sin + pow_im * self.cos;
        frame.vector(self.exp * coeff_x, self.exp * coeff_y)
    }
}

impl Geom2dEvalLogarithmicSpiralCurve {
    /// Spiral (`Geom2dEval_LogarithmicSpiralCurve(Position, Scale, GrowthExponent)`);
    /// Standard_ConstructionError unless `scale > 0` and `growth_exponent > 0`;
    /// `Geom2dEvalLogarithmicSpiralCurve::new(axis, 1.0, 0.1)?`.
    pub fn new(position: Axis2d, scale: f64, growth_exponent: f64) -> Result<Self, Failure> {
        if scale <= 0.0 {
            return Err(errors::construction(&format!(
                "{CLASS}: scale = {scale}; expected > 0"
            )));
        }
        if growth_exponent <= 0.0 {
            return Err(errors::construction(&format!(
                "{CLASS}: growth exponent = {growth_exponent}; expected > 0"
            )));
        }
        Ok(Self {
            parent: StandardTransient::default(),
            position,
            scale,
            growth_exponent,
        })
    }
    fn copied(&self) -> Self {
        Self {
            parent: StandardTransient::default(),
            ..*self
        }
    }
    /// The local coordinate system (`Position`); `curve.position()`.
    pub fn position(&self) -> Axis2d {
        self.position
    }
    /// The scale factor a (`Scale`); `curve.scale()`.
    pub fn scale(&self) -> f64 {
        self.scale
    }
    /// The growth exponent b (`GrowthExponent`); `curve.growth_exponent()`.
    pub fn growth_exponent(&self) -> f64 {
        self.growth_exponent
    }
    fn terms(&self, u: f64) -> SpiralTerms {
        SpiralTerms {
            exp: self.scale * (self.growth_exponent * u).exp(),
            b: self.growth_exponent,
            cos: u.cos(),
            sin: u.sin(),
        }
    }
}
impl Geom2dGeometry for Geom2dEvalLogarithmicSpiralCurve {
    fn transform(&self, _transform: Transform2) -> Result<(), Failure> {
        Err(unsupported(
            "Geom2dEval_LogarithmicSpiralCurve::Transform",
            "",
        ))
    }
    fn copy(&self) -> Result<Handle<dyn Geom2dGeometry>, Failure> {
        Ok(Handle::new(self.copied()).cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let fields = [
            ("Scale", self.scale),
            ("GrowthExponent", self.growth_exponent),
        ];
        dump_analytic_curve(stream, depth, CLASS, self.position, &fields)
    }
}
impl Geom2dCurve for Geom2dEvalLogarithmicSpiralCurve {
    fn reverse(&self) -> Result<(), Failure> {
        Err(unsupported(
            "Geom2dEval_LogarithmicSpiralCurve::Reverse",
            "",
        ))
    }
    fn reversed_parameter(&self, u: f64) -> Result<f64, Failure> {
        let detail: String = format!("(U = {u})");
        Err(unsupported(
            "Geom2dEval_LogarithmicSpiralCurve::ReversedParameter",
            &detail,
        ))
    }
    fn first_parameter(&self) -> f64 {
        -precision::infinite()
    }
    fn last_parameter(&self) -> f64 {
        precision::infinite()
    }
    fn is_closed(&self) -> Result<bool, Failure> {
        Ok(false)
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
        Ok(self.terms(u).point(&LocalFrame::new(self.position)))
    }
    fn eval_d1(&self, u: f64) -> Result<Geom2dCurveD1, Failure> {
        let frame = LocalFrame::new(self.position);
        let terms: SpiralTerms = self.terms(u);
        let (point, d1) = (terms.point(&frame), terms.d1(&frame));
        Ok(Geom2dCurveD1 { point, d1 })
    }
    fn eval_d2(&self, u: f64) -> Result<Geom2dCurveD2, Failure> {
        let frame = LocalFrame::new(self.position);
        let terms: SpiralTerms = self.terms(u);
        let (point, d1, d2) = (terms.point(&frame), terms.d1(&frame), terms.d2(&frame));
        Ok(Geom2dCurveD2 { point, d1, d2 })
    }
    fn eval_d3(&self, u: f64) -> Result<Geom2dCurveD3, Failure> {
        let frame = LocalFrame::new(self.position);
        let terms: SpiralTerms = self.terms(u);
        let (point, d1, d2) = (terms.point(&frame), terms.d1(&frame), terms.d2(&frame));
        let d3: Vector2 = terms.d3(&frame);
        Ok(Geom2dCurveD3 { point, d1, d2, d3 })
    }
    // Orders 1 to 3 delegate to the explicit evaluators, as natively.
    fn eval_dn(&self, u: f64, n: i32) -> Result<Vector2, Failure> {
        check_derivative_order("Geom2dEval_LogarithmicSpiralCurve::EvalDN", n)?;
        let frame = LocalFrame::new(self.position);
        let terms: SpiralTerms = self.terms(u);
        Ok(match n {
            1 => terms.d1(&frame),
            2 => terms.d2(&frame),
            3 => terms.d3(&frame),
            _ => terms.dn(&frame, n),
        })
    }
}
