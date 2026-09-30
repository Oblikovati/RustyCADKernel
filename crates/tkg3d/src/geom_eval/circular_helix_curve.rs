// Rust translation of ModelingData/TKG3d/GeomEval/GeomEval_CircularHelixCurve.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomEval_CircularHelixCurve`:
//! `C(t) = O + R*cos(t)*XDir + R*sin(t)*YDir + (P*t/(2*Pi))*ZDir`.
use super::curve_support::{AxisFrame, check_derivative_order, dump_analytic_curve};
use super::eval_failures::unsupported;
use crate::geom::errors;
use crate::geom::{GeomCurve, GeomCurveD1, GeomCurveD2, GeomCurveD3, GeomGeometry};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkmath::gp::{Axis2, Point3, Transform3, Vector3, Xyz};
use std::f64::consts::PI;
use std::io;

const CLASS: &str = "GeomEval_CircularHelixCurve";

/// A circular helix around the main direction of a frame, on `(-inf, +inf)`: one
/// turn every `2 Pi` of parameter rises by the pitch (a zero or negative pitch is
/// accepted). Immutable like every eval curve;
/// `Handle::new(GeomEvalCircularHelixCurve::new(axis, 1.0, 0.5)?)`.
pub struct GeomEvalCircularHelixCurve {
    parent: StandardTransient,
    position: Axis2,
    radius: f64,
    pitch: f64,
}
impl_transient!(
    GeomEvalCircularHelixCurve,
    "GeomEval_CircularHelixCurve",
    dyn GeomCurve,
    parent,
    interfaces: [dyn GeomGeometry, dyn GeomCurve]
);
impl Clone for GeomEvalCircularHelixCurve {
    fn clone(&self) -> Self {
        self.copied()
    }
}

// The trigonometric values and the frame of one evaluation.
struct HelixPoint {
    frame: AxisFrame,
    cos: f64,
    sin: f64,
}

impl GeomEvalCircularHelixCurve {
    /// Helix (`GeomEval_CircularHelixCurve(Position, Radius, Pitch)`);
    /// Standard_ConstructionError unless `radius > 0`;
    /// `GeomEvalCircularHelixCurve::new(axis, 1.0, 0.5)?`.
    pub fn new(position: Axis2, radius: f64, pitch: f64) -> Result<Self, Failure> {
        if radius <= 0.0 {
            return Err(errors::construction(&format!(
                "{CLASS}: radius = {radius}; expected > 0"
            )));
        }
        Ok(Self {
            parent: StandardTransient::default(),
            position,
            radius,
            pitch,
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
    /// The radius (`Radius`); `curve.radius()`.
    pub fn radius(&self) -> f64 {
        self.radius
    }
    /// The rise per full turn (`Pitch`); `curve.pitch()`.
    pub fn pitch(&self) -> f64 {
        self.pitch
    }
    fn at(&self, u: f64) -> HelixPoint {
        HelixPoint {
            frame: AxisFrame::new(self.position),
            cos: u.cos(),
            sin: u.sin(),
        }
    }
    // `anO + R*cos*aXD + R*sin*aYD + (P*U/(2 Pi))*aZD`, summed left to right.
    fn point_at(&self, h: &HelixPoint, u: f64) -> Point3 {
        let z: f64 = self.pitch * u / (2.0 * PI);
        let f: &AxisFrame = &h.frame;
        Point3::from(
            f.origin
                + f.x_dir * (self.radius * h.cos)
                + f.y_dir * (self.radius * h.sin)
                + f.z_dir * z,
        )
    }
    // `R*(-sin)*aXD + R*cos*aYD + (P/(2 Pi))*aZD`.
    fn first_derivative(&self, h: &HelixPoint) -> Vector3 {
        let rate: f64 = self.pitch / (2.0 * PI);
        let f: &AxisFrame = &h.frame;
        let planar: Xyz = f.x_dir * (self.radius * (-h.sin)) + f.y_dir * (self.radius * h.cos);
        Vector3::from(planar + f.z_dir * rate)
    }
    // `R*(-cos)*aXD + R*(-sin)*aYD`.
    fn second_derivative(&self, h: &HelixPoint) -> Vector3 {
        let f: &AxisFrame = &h.frame;
        Vector3::from(f.x_dir * (self.radius * (-h.cos)) + f.y_dir * (self.radius * (-h.sin)))
    }
    // `R*sin*aXD + R*(-cos)*aYD`.
    fn third_derivative(&self, h: &HelixPoint) -> Vector3 {
        let f: &AxisFrame = &h.frame;
        Vector3::from(f.x_dir * (self.radius * h.sin) + f.y_dir * (self.radius * (-h.cos)))
    }
}
impl GeomGeometry for GeomEvalCircularHelixCurve {
    fn transform(&self, _transform: Transform3) -> Result<(), Failure> {
        Err(unsupported("GeomEval_CircularHelixCurve::Transform", ""))
    }
    fn copy(&self) -> Result<Handle<dyn GeomGeometry>, Failure> {
        Ok(Handle::new(self.copied()).cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        let fields = [("Radius", self.radius), ("Pitch", self.pitch)];
        dump_analytic_curve(stream, depth, CLASS, self.position, &fields)
    }
}
impl GeomCurve for GeomEvalCircularHelixCurve {
    rustycad_tkg2d::unsupported_curve_reversal!(unsupported, "GeomEval_CircularHelixCurve");
    rustycad_tkg2d::open_curve_range!(-precision::infinite(), precision::infinite());
    rustycad_tkg2d::non_periodic_cn_curve!();
    fn eval_d0(&self, u: f64) -> Result<Point3, Failure> {
        Ok(self.point_at(&self.at(u), u))
    }
    fn eval_d1(&self, u: f64) -> Result<GeomCurveD1, Failure> {
        let h: HelixPoint = self.at(u);
        let point: Point3 = self.point_at(&h, u);
        let d1: Vector3 = self.first_derivative(&h);
        Ok(GeomCurveD1 { point, d1 })
    }
    fn eval_d2(&self, u: f64) -> Result<GeomCurveD2, Failure> {
        let h: HelixPoint = self.at(u);
        let point: Point3 = self.point_at(&h, u);
        let d1: Vector3 = self.first_derivative(&h);
        let d2: Vector3 = self.second_derivative(&h);
        Ok(GeomCurveD2 { point, d1, d2 })
    }
    fn eval_d3(&self, u: f64) -> Result<GeomCurveD3, Failure> {
        let h: HelixPoint = self.at(u);
        let point: Point3 = self.point_at(&h, u);
        let d1: Vector3 = self.first_derivative(&h);
        let d2: Vector3 = self.second_derivative(&h);
        let d3: Vector3 = self.third_derivative(&h);
        Ok(GeomCurveD3 { point, d1, d2, d3 })
    }
    // The planar part shifts the angle by `N * Pi / 2` (cos(U + phase), sin(U + phase)),
    // so its bits can differ from EvalD1..D3, which negate sin and cos instead.
    fn eval_dn(&self, u: f64, n: i32) -> Result<Vector3, Failure> {
        check_derivative_order("GeomEval_CircularHelixCurve::EvalDN", n)?;
        let f = AxisFrame::new(self.position);
        let phase: f64 = f64::from(n) * PI / 2.0;
        let coeff_x: f64 = self.radius * (u + phase).cos();
        let coeff_y: f64 = self.radius * (u + phase).sin();
        let mut result: Xyz = f.x_dir * coeff_x + f.y_dir * coeff_y;
        if n == 1 {
            result += f.z_dir * (self.pitch / (2.0 * PI));
        }
        Ok(Vector3::from(result))
    }
}
