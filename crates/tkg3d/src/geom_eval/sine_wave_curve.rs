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
// Rust translation of ModelingData/TKG3d/GeomEval/GeomEval_SineWaveCurve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomEval_SineWaveCurve`: `C(t) = O + t*XDir + A*sin(omega*t + phi)*YDir`, in the
//! plane of a `gp_Ax2`.
use super::curve_support::{AxisFrame, check_derivative_order, dump_analytic_curve};
use super::eval_failures::unsupported;
use crate::geom::errors;
use crate::geom::{GeomCurve, GeomCurveD1, GeomCurveD2, GeomCurveD3, GeomGeometry};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg2d::geom2d_eval::eval_basis::pow_int;
use rustycad_tkmath::gp::{Axis2, Point3, Transform3, Vector3, Xyz};
use std::f64::consts::PI;
use std::io;

const CLASS: &str = "GeomEval_SineWaveCurve";

/// A sine wave along the X direction of a frame, on `(-inf, +inf)`, not periodic.
///
/// Eval curves have no mutators: `Reverse` and `Transform` raise Standard_NotImplemented,
/// so the object needs no lock; `Handle::new(GeomEvalSineWaveCurve::new(axis, 2.0, 3.0, 0.0)?)`.
pub struct GeomEvalSineWaveCurve {
    parent: StandardTransient,
    position: Axis2,
    amplitude: f64,
    omega: f64,
    phase: f64,
}
impl_transient!(
    GeomEvalSineWaveCurve,
    "GeomEval_SineWaveCurve",
    dyn GeomCurve,
    parent,
    interfaces: [dyn GeomGeometry, dyn GeomCurve]
);
impl Clone for GeomEvalSineWaveCurve {
    fn clone(&self) -> Self {
        self.copied()
    }
}
impl GeomEvalSineWaveCurve {
    /// Sine wave (`GeomEval_SineWaveCurve(Position, Amplitude, Omega, Phase = 0)`);
    /// Standard_ConstructionError unless `amplitude > 0` and `omega > 0`;
    /// `GeomEvalSineWaveCurve::new(axis, 2.0, 3.0, 0.0)?`.
    pub fn new(position: Axis2, amplitude: f64, omega: f64, phase: f64) -> Result<Self, Failure> {
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
    pub fn position(&self) -> Axis2 {
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
    fn argument(&self, u: f64) -> f64 {
        self.omega * u + self.phase
    }
    // `anO + U * aXD + A * sin(aArg) * aYD`.
    fn point_at(&self, frame: &AxisFrame, u: f64) -> Point3 {
        let wave: f64 = self.amplitude * self.argument(u).sin();
        Point3::from(frame.origin + frame.x_dir * u + frame.y_dir * wave)
    }
    // `aXD + A * omega * cos(aArg) * aYD`: the X direction is added unscaled.
    fn first_derivative(&self, frame: &AxisFrame, u: f64) -> Vector3 {
        let b: f64 = self.amplitude * self.omega * self.argument(u).cos();
        Vector3::from(frame.x_dir + frame.y_dir * b)
    }
    // `A * (-omega^2) * sin(aArg) * aYD`.
    fn second_derivative(&self, frame: &AxisFrame, u: f64) -> Vector3 {
        let om2: f64 = self.omega * self.omega;
        Vector3::from(frame.y_dir * (self.amplitude * (-om2) * self.argument(u).sin()))
    }
    // `A * (-omega^3) * cos(aArg) * aYD`, with `aOm3 = aOm2 * omega`.
    fn third_derivative(&self, frame: &AxisFrame, u: f64) -> Vector3 {
        let om3: f64 = self.omega * self.omega * self.omega;
        Vector3::from(frame.y_dir * (self.amplitude * (-om3) * self.argument(u).cos()))
    }
}
impl GeomGeometry for GeomEvalSineWaveCurve {
    fn transform(&self, _transform: Transform3) -> Result<(), Failure> {
        Err(unsupported("GeomEval_SineWaveCurve::Transform", ""))
    }
    fn copy(&self) -> Result<Handle<dyn GeomGeometry>, Failure> {
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
impl GeomCurve for GeomEvalSineWaveCurve {
    rustycad_tkg2d::unsupported_curve_reversal!(unsupported, "GeomEval_SineWaveCurve");
    rustycad_tkg2d::open_curve_range!(-precision::infinite(), precision::infinite());
    rustycad_tkg2d::non_periodic_cn_curve!();
    fn eval_d0(&self, u: f64) -> Result<Point3, Failure> {
        Ok(self.point_at(&AxisFrame::new(self.position), u))
    }
    fn eval_d1(&self, u: f64) -> Result<GeomCurveD1, Failure> {
        let frame = AxisFrame::new(self.position);
        let point: Point3 = self.point_at(&frame, u);
        let d1: Vector3 = self.first_derivative(&frame, u);
        Ok(GeomCurveD1 { point, d1 })
    }
    fn eval_d2(&self, u: f64) -> Result<GeomCurveD2, Failure> {
        let frame = AxisFrame::new(self.position);
        let point: Point3 = self.point_at(&frame, u);
        let d1: Vector3 = self.first_derivative(&frame, u);
        let d2: Vector3 = self.second_derivative(&frame, u);
        Ok(GeomCurveD2 { point, d1, d2 })
    }
    fn eval_d3(&self, u: f64) -> Result<GeomCurveD3, Failure> {
        let frame = AxisFrame::new(self.position);
        let point: Point3 = self.point_at(&frame, u);
        let d1: Vector3 = self.first_derivative(&frame, u);
        let d2: Vector3 = self.second_derivative(&frame, u);
        let d3: Vector3 = self.third_derivative(&frame, u);
        Ok(GeomCurveD3 { point, d1, d2, d3 })
    }
    // Unlike Geom2dEval, the 3D curve shifts the sine argument by `N * Pi / 2` instead
    // of rotating (sin, cos) by quarter turns, so its bits can differ from EvalD1..D3.
    fn eval_dn(&self, u: f64, n: i32) -> Result<Vector3, Failure> {
        check_derivative_order("GeomEval_SineWaveCurve::EvalDN", n)?;
        let frame = AxisFrame::new(self.position);
        // The native local loop multiplies omega into 1.0 N times, as `powInt` does.
        let om_n: f64 = pow_int(self.omega, n);
        let shifted: f64 = self.omega * u + self.phase + f64::from(n) * PI / 2.0;
        let mut result: Xyz = frame.y_dir * (self.amplitude * om_n * shifted.sin());
        if n == 1 {
            result += frame.x_dir;
        }
        Ok(Vector3::from(result))
    }
}
