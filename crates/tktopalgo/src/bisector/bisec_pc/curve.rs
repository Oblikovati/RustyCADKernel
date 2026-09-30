// Rust translation of ModelingAlgorithms/TKTopAlgo/Bisector/Bisector_BisecPC.cxx (the Geom2d_Curve members) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `Geom2d_Geometry` and `Geom2d_Curve` members of `Bisector_BisecPC`.
use super::{BisecPC, PcState, PcValues, copy_curve, lowered_continuity};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{
    Geom2dCurve, Geom2dCurveD1, Geom2dCurveD2, Geom2dCurveD3, Geom2dGeometry, dump_curve, errors,
};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Transform2, Vector2};
use std::io;

impl BisecPC {
    // `Values(U, N, ...)` from the point of the bisector and null derivatives, as
    // every evaluator initialises them.
    fn eval(&self, u: f64, n: i32) -> Result<PcValues, Failure> {
        let state = self.state.read();
        let start = PcValues {
            p: state.point,
            ..PcValues::default()
        };
        state.values(u, n, start)
    }
}

impl Geom2dGeometry for BisecPC {
    fn transform(&self, transform: Transform2) -> Result<(), Failure> {
        let mut state = self.state.write();
        state.curve.transform(transform)?;
        state.point.transform(transform);
        state.point_start_bis.transform(transform);
        state.point_end_bis.transform(transform);
        Ok(())
    }
    fn copy(&self) -> Result<Handle<dyn Geom2dGeometry>, Failure> {
        let state: PcState = self.state.snapshot();
        let copy = PcState {
            curve: copy_curve(&state.curve)?,
            ..state
        };
        Ok(Handle::new(BisecPC::from_state(copy)).cast())
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        // The class inherits Geom2d_Curve::DumpJson, whose class name is static.
        dump_curve(stream, depth)
    }
}

impl Geom2dCurve for BisecPC {
    fn reverse(&self) -> Result<(), Failure> {
        Err(errors::not_implemented("Bisector_BisecPC::Reverse"))
    }
    fn reversed_parameter(&self, u: f64) -> Result<f64, Failure> {
        Ok(self.last_parameter() + self.first_parameter() - u)
    }
    fn first_parameter(&self) -> f64 {
        self.state
            .read()
            .start_intervals
            .first()
            .unwrap_or(f64::NAN)
    }
    fn last_parameter(&self) -> f64 {
        self.state.read().end_intervals.last().unwrap_or(f64::NAN)
    }
    fn is_closed(&self) -> Result<bool, Failure> {
        // The bisector is closed if the curve is closed and it has only one domain
        // of continuity equal to the one of the curve.
        let state = self.state.read();
        if !state.curve.is_closed()? {
            return Ok(false);
        }
        Ok(
            state.start_intervals.first()? == state.curve.first_parameter()
                && state.end_intervals.first()? == state.curve.last_parameter(),
        )
    }
    fn is_periodic(&self) -> bool {
        false
    }
    fn continuity(&self) -> Shape {
        lowered_continuity(self.state.read().curve.continuity())
    }
    fn is_cn(&self, n: i32) -> Result<bool, Failure> {
        self.state.read().curve.is_cn(n + 1)
    }
    fn eval_d0(&self, u: f64) -> Result<Point2, Failure> {
        Ok(self.eval(u, 0)?.p)
    }
    fn eval_d1(&self, u: f64) -> Result<Geom2dCurveD1, Failure> {
        let v = self.eval(u, 1)?;
        Ok(Geom2dCurveD1 {
            point: v.p,
            d1: v.v1,
        })
    }
    fn eval_d2(&self, u: f64) -> Result<Geom2dCurveD2, Failure> {
        let v = self.eval(u, 2)?;
        Ok(Geom2dCurveD2 {
            point: v.p,
            d1: v.v1,
            d2: v.v2,
        })
    }
    fn eval_d3(&self, u: f64) -> Result<Geom2dCurveD3, Failure> {
        // Values has no case for N = 3: the point and every derivative keep their
        // initial values (see docs/tktopalgo-progress.md).
        let v = self.eval(u, 3)?;
        Ok(Geom2dCurveD3 {
            point: v.p,
            d1: v.v1,
            d2: v.v2,
            d3: v.v3,
        })
    }
    fn eval_dn(&self, u: f64, n: i32) -> Result<Vector2, Failure> {
        if n < 1 {
            return Err(errors::undefined_derivative(&format!(
                "Bisector_BisecPC::EvalDN: N = {n}; expected 1..=3"
            )));
        }
        let v = self.eval(u, n)?;
        match n {
            1 => Ok(v.v1),
            2 => Ok(v.v2),
            3 => Ok(v.v3),
            _ => Err(errors::undefined_derivative(&format!(
                "Bisector_BisecPC::EvalDN: N = {n}; expected 1..=3"
            ))),
        }
    }
}
