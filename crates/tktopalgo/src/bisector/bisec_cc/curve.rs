// Rust translation of ModelingAlgorithms/TKTopAlgo/Bisector/Bisector_BisecCC.cxx (the Geom2d_Curve members) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `Geom2d_Geometry` and `Geom2d_Curve` members of `Bisector_BisecCC`.
use super::BisecCC;
use super::state::{CcState, OnCurves};
use super::values::CcValues;
use crate::bisector::bisec_pc::{copy_curve, lowered_continuity};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{
    Geom2dCurve, Geom2dCurveD1, Geom2dCurveD2, Geom2dCurveD3, Geom2dGeometry, dump_curve, errors,
};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Transform2, Vector2};
use std::io;

impl Geom2dGeometry for BisecCC {
    fn transform(&self, transform: Transform2) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.curve1.transform(transform)?;
        state.curve2.transform(transform)?;
        state.my_polygon.transform(transform);
        state.point_start.transform(transform);
        state.point_end.transform(transform);
        Ok(())
    }
    fn copy(&self) -> Result<Handle<dyn Geom2dGeometry>, Failure> {
        // The native copy sets every field but the shift and the current interval,
        // which stay 0 (the shift is never changed from 0 in this class).
        let state: CcState = self.state.snapshot();
        let copy = CcState {
            curve1: copy_curve(&state.curve1)?,
            curve2: copy_curve(&state.curve2)?,
            shift_parameter: 0.,
            current_interval: 0,
            ..state
        };
        Ok(Handle::new(BisecCC::from_state(copy)).cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        // The class inherits Geom2d_Curve::DumpJson, whose class name is static.
        dump_curve(stream, depth)
    }
}

impl BisecCC {
    fn values(&self, u: f64, n: i32) -> Result<CcValues, Failure> {
        self.state.read().values(u, n)
    }
}

impl Geom2dCurve for BisecCC {
    fn reverse(&self) -> Result<(), Failure> {
        Err(errors::not_implemented("Bisector_BisecCC::Reverse"))
    }
    fn reversed_parameter(&self, u: f64) -> Result<f64, Failure> {
        Ok(self.last_parameter() + self.first_parameter() - u)
    }
    fn first_parameter(&self) -> f64 {
        self.state.read().first_parameter().unwrap_or(f64::NAN)
    }
    fn last_parameter(&self) -> f64 {
        self.state.read().last_parameter().unwrap_or(f64::NAN)
    }
    fn is_closed(&self) -> Result<bool, Failure> {
        let state = self.state.read();
        if !state.curve1.is_closed()? {
            return Ok(false);
        }
        Ok(
            state.start_intervals.first()? == state.curve1.first_parameter()
                && state.end_intervals.last()? == state.curve1.last_parameter(),
        )
    }
    fn is_periodic(&self) -> bool {
        false
    }
    fn continuity(&self) -> Shape {
        lowered_continuity(self.state.read().curve1.continuity())
    }
    fn is_cn(&self, n: i32) -> Result<bool, Failure> {
        let state = self.state.read();
        Ok(state.curve1.is_cn(n + 1)? && state.curve2.is_cn(n + 1)?)
    }
    fn eval_d0(&self, u: f64) -> Result<Point2, Failure> {
        let mut on_curves = OnCurves::default();
        self.state.read().value_and_dist(u, &mut on_curves)
    }
    fn eval_d1(&self, u: f64) -> Result<Geom2dCurveD1, Failure> {
        let v = self.values(u, 1)?;
        Ok(Geom2dCurveD1 {
            point: v.p,
            d1: v.v1,
        })
    }
    fn eval_d2(&self, u: f64) -> Result<Geom2dCurveD2, Failure> {
        // Values computes the first derivative only: the second stays null.
        let v = self.values(u, 2)?;
        Ok(Geom2dCurveD2 {
            point: v.p,
            d1: v.v1,
            d2: v.v2,
        })
    }
    fn eval_d3(&self, u: f64) -> Result<Geom2dCurveD3, Failure> {
        let v = self.values(u, 3)?;
        Ok(Geom2dCurveD3 {
            point: v.p,
            d1: v.v1,
            d2: v.v2,
            d3: v.v3,
        })
    }
    fn eval_dn(&self, u: f64, n: i32) -> Result<Vector2, Failure> {
        let undefined = || {
            errors::undefined_derivative(&format!(
                "Bisector_BisecCC::EvalDN: N = {n}; expected 1..=3"
            ))
        };
        if n < 1 {
            return Err(undefined());
        }
        let v = self.values(u, n)?;
        match n {
            1 => Ok(v.v1),
            2 => Ok(v.v2),
            3 => Ok(v.v3),
            _ => Err(undefined()),
        }
    }
}
