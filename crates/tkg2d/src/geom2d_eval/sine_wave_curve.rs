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
// Rust translation of ModelingData/TKG2d/Geom2dEval/Geom2dEval_SineWaveCurve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dEval_SineWaveCurve`: `C(t) = O + t*XDir + A*sin(omega*t + phi)*YDir`.
use super::analytic_dump::dump_analytic_curve;
use super::eval_math::{LocalFrame, check_derivative_order, pow_int, trig_shift, unsupported};
use crate::geom2d::errors;
use crate::geom2d::{Geom2dCurve, Geom2dCurveD1, Geom2dCurveD2, Geom2dCurveD3, Geom2dGeometry};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Axis2d, Point2, Transform2, Vector2, Xy};
use std::io;

const CLASS: &str = "Geom2dEval_SineWaveCurve";

/// A sine wave along the X direction of an axis, on `(-inf, +inf)`, not periodic.
///
/// Eval curves have no mutators: `Reverse` and `Transform` raise Standard_NotImplemented,
/// so the object needs no lock; `Handle::new(Geom2dEvalSineWaveCurve::new(axis, 2.0, 3.0, 0.0)?)`.
pub struct Geom2dEvalSineWaveCurve {
    parent: StandardTransient,
    position: Axis2d,
    amplitude: f64,
    omega: f64,
    phase: f64,
}
impl_transient!(
    Geom2dEvalSineWaveCurve,
    "Geom2dEval_SineWaveCurve",
    dyn Geom2dCurve,
    parent,
    interfaces: [dyn Geom2dGeometry, dyn Geom2dCurve]
);
impl Clone for Geom2dEvalSineWaveCurve {
    fn clone(&self) -> Self {
        self.copied()
    }
}
impl Geom2dEvalSineWaveCurve {
    /// Sine wave (`Geom2dEval_SineWaveCurve(Position, Amplitude, Omega, Phase = 0)`);
    /// Standard_ConstructionError unless `amplitude > 0` and `omega > 0`;
    /// `Geom2dEvalSineWaveCurve::new(axis, 2.0, 3.0, 0.0)?`.
    pub fn new(position: Axis2d, amplitude: f64, omega: f64, phase: f64) -> Result<Self, Failure> {
        if amplitude <= 0.0 {
            return Err(errors::construction(&format!(
                "{CLASS}: amplitude = {amplitude}; expected > 0"
            )));
        }
        if omega <= 0.0 {
            return Err(errors::construction(&format!(
                "{CLASS}: omega = {omega}; expected > 0"
            )));
        }
        Ok(Self {
            parent: StandardTransient::default(),
            position,
            amplitude,
            omega,
            phase,
        })
    }
    // The native Copy() reruns the constructor on valid stored values.
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
    /// The amplitude (`Amplitude`); `curve.amplitude()`.
    pub fn amplitude(&self) -> f64 {
        self.amplitude
    }
    /// The angular frequency (`Omega`); `curve.omega()`.
    pub fn omega(&self) -> f64 {
        self.omega
    }
    /// The phase shift (`Phase`); `curve.phase()`.
    pub fn phase(&self) -> f64 {
        self.phase
    }
    fn frame(&self) -> LocalFrame {
        LocalFrame::new(self.position)
    }
    fn argument(&self, u: f64) -> f64 {
        self.omega * u + self.phase
    }
    // `anO + U * aXD + A * sin(aArg) * aYD`.
    fn point_at(&self, frame: &LocalFrame, u: f64) -> Point2 {
        frame.point(u, self.amplitude * self.argument(u).sin())
    }
    // `aXD + A * omega * cos(aArg) * aYD`: the X direction is added unscaled.
    fn first_derivative(&self, frame: &LocalFrame, u: f64) -> Vector2 {
        let b: f64 = self.amplitude * self.omega * self.argument(u).cos();
        Vector2::from(frame.x_dir + b * frame.y_dir)
    }
    // `A * (-omega^2) * sin(aArg) * aYD`: no X term, so a zero X keeps the sign of the product.
    fn second_derivative(&self, frame: &LocalFrame, u: f64) -> Vector2 {
        let om2: f64 = self.omega * self.omega;
        Vector2::from(self.amplitude * (-om2) * self.argument(u).sin() * frame.y_dir)
    }
    fn third_derivative(&self, frame: &LocalFrame, u: f64) -> Vector2 {
        let om3: f64 = self.omega * self.omega * self.omega;
        Vector2::from(self.amplitude * (-om3) * self.argument(u).cos() * frame.y_dir)
    }
}
impl Geom2dGeometry for Geom2dEvalSineWaveCurve {
    fn transform(&self, _transform: Transform2) -> Result<(), Failure> {
        Err(unsupported("Geom2dEval_SineWaveCurve::Transform", ""))
    }
    fn copy(&self) -> Result<Handle<dyn Geom2dGeometry>, Failure> {
        Ok(Handle::new(self.copied()).cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let fields = [
            ("Amplitude", self.amplitude),
            ("Omega", self.omega),
            ("Phase", self.phase),
        ];
        dump_analytic_curve(stream, depth, CLASS, self.position, &fields)
    }
}
impl Geom2dCurve for Geom2dEvalSineWaveCurve {
    fn reverse(&self) -> Result<(), Failure> {
        Err(unsupported("Geom2dEval_SineWaveCurve::Reverse", ""))
    }
    fn reversed_parameter(&self, u: f64) -> Result<f64, Failure> {
        let detail: String = format!("(U = {u})");
        Err(unsupported(
            "Geom2dEval_SineWaveCurve::ReversedParameter",
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
        Ok(self.point_at(&self.frame(), u))
    }
    fn eval_d1(&self, u: f64) -> Result<Geom2dCurveD1, Failure> {
        let frame: LocalFrame = self.frame();
        let point: Point2 = self.point_at(&frame, u);
        let d1: Vector2 = self.first_derivative(&frame, u);
        Ok(Geom2dCurveD1 { point, d1 })
    }
    fn eval_d2(&self, u: f64) -> Result<Geom2dCurveD2, Failure> {
        let frame: LocalFrame = self.frame();
        let point: Point2 = self.point_at(&frame, u);
        let d1: Vector2 = self.first_derivative(&frame, u);
        let d2: Vector2 = self.second_derivative(&frame, u);
        Ok(Geom2dCurveD2 { point, d1, d2 })
    }
    fn eval_d3(&self, u: f64) -> Result<Geom2dCurveD3, Failure> {
        let frame: LocalFrame = self.frame();
        let point: Point2 = self.point_at(&frame, u);
        let d1: Vector2 = self.first_derivative(&frame, u);
        let d2: Vector2 = self.second_derivative(&frame, u);
        let d3: Vector2 = self.third_derivative(&frame, u);
        Ok(Geom2dCurveD3 { point, d1, d2, d3 })
    }
    fn eval_dn(&self, u: f64, n: i32) -> Result<Vector2, Failure> {
        check_derivative_order("Geom2dEval_SineWaveCurve::EvalDN", n)?;
        let frame: LocalFrame = self.frame();
        let argument: f64 = self.argument(u);
        let (sin_n, _) = trig_shift(argument.sin(), argument.cos(), n);
        let mut result: Xy = self.amplitude * pow_int(self.omega, n) * sin_n * frame.y_dir;
        if n == 1 {
            result += frame.x_dir;
        }
        Ok(Vector2::from(result))
    }
}
