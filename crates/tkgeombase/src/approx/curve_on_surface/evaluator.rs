// Rust translation of ModelingData/TKGeomBase/Approx/Approx_CurveOnSurface.cxx (Approx_CurveOnSurface_Eval)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Approx_CurveOnSurface_Eval`: the 2D curve and the curve on surface together (5
//! values: u, v, x, y, z).
use crate::failure_capture::FailureCapture;
use crate::native_failure::failure;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::adv_approx::AdvApproxEvaluatorFunction;

/// The combined evaluator; the curve on surface and the 2D curve are trimmed
/// together when AdvApprox changes interval, and an adaptor exception is carried to
/// the end of the approximation.
pub(super) struct CurveAndPcurveEvaluator {
    fonct: Handle<dyn Adaptor3dCurve>,
    fonct2d: Handle<dyn Adaptor2dCurve2d>,
    start_end_sav: [f64; 2],
    capture: FailureCapture,
}

fn null_adaptor() -> Failure {
    failure(
        "Standard_NullObject",
        "Approx_CurveOnSurface_Eval: null adaptor; expected a curve",
    )
}

impl CurveAndPcurveEvaluator {
    pub(super) fn new(
        fonct: Handle<dyn Adaptor3dCurve>,
        fonct2d: Handle<dyn Adaptor2dCurve2d>,
        range: [f64; 2],
    ) -> Self {
        Self {
            fonct,
            fonct2d,
            start_end_sav: range,
            capture: FailureCapture::default(),
        }
    }

    pub(super) fn take_failure(&mut self) -> Result<(), Failure> {
        self.capture.take()
    }

    fn retrim(&mut self, [a, b]: [f64; 2]) -> Result<(), Failure> {
        let tol: f64 = precision::p_confusion();
        self.fonct = self.fonct.get().ok_or_else(null_adaptor)?.trim(a, b, tol)?;
        self.fonct2d = self
            .fonct2d
            .get()
            .ok_or_else(null_adaptor)?
            .trim(a, b, tol)?;
        self.start_end_sav = [a, b];
        Ok(())
    }

    // The derivative of `order` of (u, v, x, y, z); the 2D curve is evaluated first,
    // as natively.
    fn values(&self, par: f64, order: i32) -> Result<[f64; 5], Failure> {
        let c2d: &dyn Adaptor2dCurve2d = self.fonct2d.get().ok_or_else(null_adaptor)?;
        let c3d: &dyn Adaptor3dCurve = self.fonct.get().ok_or_else(null_adaptor)?;
        Ok(match order {
            0 => {
                let p2 = c2d.value(par)?;
                let p = c3d.value(par)?;
                [p2.x(), p2.y(), p.x(), p.y(), p.z()]
            }
            1 => {
                let (_, v2) = c2d.d1(par)?;
                let (_, v) = c3d.d1(par)?;
                [v2.x(), v2.y(), v.x(), v.y(), v.z()]
            }
            2 => {
                let (_, _, v2) = c2d.d2(par)?;
                let (_, _, v) = c3d.d2(par)?;
                [v2.x(), v2.y(), v.x(), v.y(), v.z()]
            }
            _ => [0.0; 5],
        })
    }
}

impl AdvApproxEvaluatorFunction for CurveAndPcurveEvaluator {
    fn evaluate(
        &mut self,
        dimension: i32,
        start_end: [f64; 2],
        par: f64,
        order: i32,
        result: &mut [f64],
    ) -> i32 {
        let error_code: i32 = if dimension != 5 { 1 } else { 0 };
        if start_end != self.start_end_sav {
            let trimmed = self.retrim(start_end);
            if self.capture.keep(trimmed).is_none() {
                return 1;
            }
        }
        let values = self.values(par, order);
        let Some(values) = self.capture.keep(values) else {
            return 1;
        };
        result[..5].copy_from_slice(&values);
        if !(0..=2).contains(&order) {
            return 3;
        }
        error_code
    }
}
