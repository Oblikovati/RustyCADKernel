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
// Rust translation of ModelingData/TKG2d/Geom2dEval/Geom2dEval_ArchimedeanSpiralCurve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dEval_ArchimedeanSpiralCurve`: `C(t) = O + (a + b*t)*(cos(t)*XDir + sin(t)*YDir)`.
use super::analytic_dump::dump_analytic_curve;
use super::eval_basis::trig_shift;
use super::eval_math::{LocalFrame, check_derivative_order, local_frame_evaluators, unsupported};
use crate::geom2d::errors;
use crate::geom2d::{Geom2dCurve, Geom2dCurveD1, Geom2dCurveD2, Geom2dCurveD3, Geom2dGeometry};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkmath::gp::{Axis2d, Point2, Transform2, Vector2};
use std::io;

const CLASS: &str = "Geom2dEval_ArchimedeanSpiralCurve";

/// An Archimedean spiral `r = a + b*t` on `[0, +inf)`, neither closed nor periodic;
/// immutable like every eval curve;
/// `Handle::new(Geom2dEvalArchimedeanSpiralCurve::new(axis, 0.0, 1.0)?)`.
pub struct Geom2dEvalArchimedeanSpiralCurve {
    parent: StandardTransient,
    position: Axis2d,
    initial_radius: f64,
    growth_rate: f64,
}
impl_transient!(
    Geom2dEvalArchimedeanSpiralCurve,
    "Geom2dEval_ArchimedeanSpiralCurve",
    dyn Geom2dCurve,
    parent,
    interfaces: [dyn Geom2dGeometry, dyn Geom2dCurve]
);
impl Clone for Geom2dEvalArchimedeanSpiralCurve {
    fn clone(&self) -> Self {
        self.copied()
    }
}

// Radius and its derivative at `u`, and the trigonometric values the evaluators share.
struct SpiralTerms {
    r: f64,
    b: f64,
    cos: f64,
    sin: f64,
}
impl SpiralTerms {
    // `C = (r cos) XD + (r sin) YD`.
    fn point(&self, frame: &LocalFrame) -> Point2 {
        frame.point(self.r * self.cos, self.r * self.sin)
    }
    // `(b cos - r sin) XD + (b sin + r cos) YD`.
    fn d1(&self, frame: &LocalFrame) -> Vector2 {
        let (b, r, c, s) = (self.b, self.r, self.cos, self.sin);
        frame.vector(b * c - r * s, b * s + r * c)
    }
    // `(-2 b sin - r cos) XD + (2 b cos - r sin) YD`.
    fn d2(&self, frame: &LocalFrame) -> Vector2 {
        let (b, r, c, s) = (self.b, self.r, self.cos, self.sin);
        frame.vector(-2.0 * b * s - r * c, 2.0 * b * c - r * s)
    }
    // `(-3 b cos + r sin) XD + (-3 b sin - r cos) YD`.
    fn d3(&self, frame: &LocalFrame) -> Vector2 {
        let (b, r, c, s) = (self.b, self.r, self.cos, self.sin);
        frame.vector(-3.0 * b * c + r * s, -3.0 * b * s - r * c)
    }
}

impl Geom2dEvalArchimedeanSpiralCurve {
    /// Spiral (`Geom2dEval_ArchimedeanSpiralCurve(Position, InitialRadius, GrowthRate)`);
    /// Standard_ConstructionError unless `initial_radius >= 0` and `growth_rate > 0`;
    /// `Geom2dEvalArchimedeanSpiralCurve::new(axis, 0.0, 1.0)?`.
    pub fn new(position: Axis2d, initial_radius: f64, growth_rate: f64) -> Result<Self, Failure> {
        if initial_radius < 0.0 {
            return Err(errors::construction(&format!(
                "{CLASS}: initial radius = {initial_radius}; expected >= 0"
            )));
        }
        if growth_rate <= 0.0 {
            return Err(errors::construction(&format!(
                "{CLASS}: growth rate = {growth_rate}; expected > 0"
            )));
        }
        Ok(Self {
            parent: StandardTransient::default(),
            position,
            initial_radius,
            growth_rate,
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
    /// The radius at t = 0 (`InitialRadius`); `curve.initial_radius()`.
    pub fn initial_radius(&self) -> f64 {
        self.initial_radius
    }
    /// The radius growth per radian (`GrowthRate`); `curve.growth_rate()`.
    pub fn growth_rate(&self) -> f64 {
        self.growth_rate
    }
    fn radius_at(&self, u: f64) -> f64 {
        self.initial_radius + self.growth_rate * u
    }
    fn terms(&self, u: f64) -> SpiralTerms {
        SpiralTerms {
            r: self.radius_at(u),
            b: self.growth_rate,
            cos: u.cos(),
            sin: u.sin(),
        }
    }
}
impl Geom2dGeometry for Geom2dEvalArchimedeanSpiralCurve {
    fn transform(&self, _transform: Transform2) -> Result<(), Failure> {
        Err(unsupported(
            "Geom2dEval_ArchimedeanSpiralCurve::Transform",
            "",
        ))
    }
    fn copy(&self) -> Result<Handle<dyn Geom2dGeometry>, Failure> {
        Ok(Handle::new(self.copied()).cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let fields = [
            ("InitialRadius", self.initial_radius),
            ("GrowthRate", self.growth_rate),
        ];
        dump_analytic_curve(stream, depth, CLASS, self.position, &fields)
    }
}
impl Geom2dCurve for Geom2dEvalArchimedeanSpiralCurve {
    crate::unsupported_curve_reversal!(unsupported, "Geom2dEval_ArchimedeanSpiralCurve");
    crate::open_curve_range!(0.0, precision::infinite());
    crate::non_periodic_cn_curve!();
    local_frame_evaluators!(SpiralTerms);
    // Leibniz on (a + b t) f(t): only the k = 0 and k = 1 terms survive,
    // `r f^(N) + N b f^(N-1)`.
    fn eval_dn(&self, u: f64, n: i32) -> Result<Vector2, Failure> {
        check_derivative_order("Geom2dEval_ArchimedeanSpiralCurve::EvalDN", n)?;
        let frame = LocalFrame::new(self.position);
        let terms: SpiralTerms = self.terms(u);
        let (sin_n, cos_n) = trig_shift(terms.sin, terms.cos, n);
        let (sin_nm1, cos_nm1) = trig_shift(terms.sin, terms.cos, n - 1);
        let order: f64 = f64::from(n);
        let coeff_x: f64 = terms.r * cos_n + order * terms.b * cos_nm1;
        let coeff_y: f64 = terms.r * sin_n + order * terms.b * sin_nm1;
        Ok(frame.vector(coeff_x, coeff_y))
    }
}
