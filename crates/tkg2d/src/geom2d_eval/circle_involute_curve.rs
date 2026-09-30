// Rust translation of ModelingData/TKG2d/Geom2dEval/Geom2dEval_CircleInvoluteCurve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2dEval_CircleInvoluteCurve`:
//! `C(t) = O + R*(cos(t) + t*sin(t))*XDir + R*(sin(t) - t*cos(t))*YDir`.
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

const CLASS: &str = "Geom2dEval_CircleInvoluteCurve";

/// The involute of a circle of radius R on `[0, +inf)`, neither closed nor periodic;
/// immutable like every eval curve; `Handle::new(Geom2dEvalCircleInvoluteCurve::new(axis, 1.0)?)`.
pub struct Geom2dEvalCircleInvoluteCurve {
    parent: StandardTransient,
    position: Axis2d,
    radius: f64,
}
impl_transient!(
    Geom2dEvalCircleInvoluteCurve,
    "Geom2dEval_CircleInvoluteCurve",
    dyn Geom2dCurve,
    parent,
    interfaces: [dyn Geom2dGeometry, dyn Geom2dCurve]
);
impl Clone for Geom2dEvalCircleInvoluteCurve {
    fn clone(&self) -> Self {
        self.copied()
    }
}

// The parameter, its trigonometric values and the base radius shared by the evaluators.
struct InvoluteTerms {
    radius: f64,
    u: f64,
    cos: f64,
    sin: f64,
}
impl InvoluteTerms {
    fn point(&self, frame: &LocalFrame) -> Point2 {
        let (r, u, c, s) = (self.radius, self.u, self.cos, self.sin);
        frame.point(r * (c + u * s), r * (s - u * c))
    }
    // `R t cos(t) XD + R t sin(t) YD`.
    fn d1(&self, frame: &LocalFrame) -> Vector2 {
        let (r, u, c, s) = (self.radius, self.u, self.cos, self.sin);
        frame.vector(r * u * c, r * u * s)
    }
    fn d2(&self, frame: &LocalFrame) -> Vector2 {
        let (r, u, c, s) = (self.radius, self.u, self.cos, self.sin);
        frame.vector(r * (c - u * s), r * (s + u * c))
    }
    fn d3(&self, frame: &LocalFrame) -> Vector2 {
        let (r, u, c, s) = (self.radius, self.u, self.cos, self.sin);
        frame.vector(r * (-2.0 * s - u * c), r * (2.0 * c - u * s))
    }
    // Leibniz on the t*trig(t) terms:
    // f^(N) = cos(t+N Pi/2) + t sin(t+N Pi/2) + N sin(t+(N-1) Pi/2),
    // g^(N) = sin(t+N Pi/2) - t cos(t+N Pi/2) - N cos(t+(N-1) Pi/2).
    fn dn(&self, frame: &LocalFrame, n: i32) -> Vector2 {
        let (sin_n, cos_n) = trig_shift(self.sin, self.cos, n);
        let (sin_nm1, cos_nm1) = trig_shift(self.sin, self.cos, n - 1);
        let order: f64 = f64::from(n);
        let f_n: f64 = cos_n + self.u * sin_n + order * sin_nm1;
        let g_n: f64 = sin_n - self.u * cos_n - order * cos_nm1;
        frame.vector(self.radius * f_n, self.radius * g_n)
    }
}

impl Geom2dEvalCircleInvoluteCurve {
    /// Involute (`Geom2dEval_CircleInvoluteCurve(Position, Radius)`);
    /// Standard_ConstructionError unless `radius > 0`;
    /// `Geom2dEvalCircleInvoluteCurve::new(axis, 1.0)?`.
    pub fn new(position: Axis2d, radius: f64) -> Result<Self, Failure> {
        if radius <= 0.0 {
            return Err(errors::construction(&format!(
                "{CLASS}: radius = {radius}; expected > 0"
            )));
        }
        Ok(Self {
            parent: StandardTransient::default(),
            position,
            radius,
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
    /// The base circle radius (`Radius`); `curve.radius()`.
    pub fn radius(&self) -> f64 {
        self.radius
    }
    fn terms(&self, u: f64) -> InvoluteTerms {
        InvoluteTerms {
            radius: self.radius,
            u,
            cos: u.cos(),
            sin: u.sin(),
        }
    }
}
impl Geom2dGeometry for Geom2dEvalCircleInvoluteCurve {
    fn transform(&self, _transform: Transform2) -> Result<(), Failure> {
        Err(unsupported("Geom2dEval_CircleInvoluteCurve::Transform", ""))
    }
    fn copy(&self) -> Result<Handle<dyn Geom2dGeometry>, Failure> {
        Ok(Handle::new(self.copied()).cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let fields = [("Radius", self.radius)];
        dump_analytic_curve(stream, depth, CLASS, self.position, &fields)
    }
}
impl Geom2dCurve for Geom2dEvalCircleInvoluteCurve {
    crate::unsupported_curve_reversal!(unsupported, "Geom2dEval_CircleInvoluteCurve");
    crate::open_curve_range!(0.0, precision::infinite());
    crate::non_periodic_cn_curve!();
    local_frame_evaluators!(InvoluteTerms);
    // Orders 1 to 3 delegate to the explicit evaluators, as natively.
    fn eval_dn(&self, u: f64, n: i32) -> Result<Vector2, Failure> {
        check_derivative_order("Geom2dEval_CircleInvoluteCurve::EvalDN", n)?;
        let frame = LocalFrame::new(self.position);
        let terms: InvoluteTerms = self.terms(u);
        Ok(match n {
            1 => terms.d1(&frame),
            2 => terms.d2(&frame),
            3 => terms.d3(&frame),
            _ => terms.dn(&frame, n),
        })
    }
}
