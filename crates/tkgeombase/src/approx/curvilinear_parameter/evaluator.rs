// Rust translation of ModelingData/TKGeomBase/Approx/Approx_CurvilinearParameter.cxx (EvalCurv, EvalCurvOnSurf, EvalCurvOn2Surf)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The three AdvApprox evaluators of `Approx_CurvilinearParameter` over
//! `Approx_CurvlinFunc`.
use crate::approx::curvlin_func::CurvlinFunc;
use crate::failure_capture::FailureCapture;
use crate::native_failure::array_failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adv_approx::AdvApproxEvaluatorFunction;

/// `EvalCurv` (case 1, 3 values), `EvalCurvOnSurf` (case 2, 5 values) or
/// `EvalCurvOn2Surf` (case 3, 7 values, which never trims the function: its trim is
/// commented out natively).
pub(super) struct CurvlinEvaluator {
    fonct: CurvlinFunc,
    start_end_sav: [f64; 2],
    case: i32,
    capture: FailureCapture,
}

impl CurvlinEvaluator {
    pub(super) fn new(fonct: CurvlinFunc, range: [f64; 2], case: i32) -> Self {
        Self {
            fonct,
            start_end_sav: range,
            case,
            capture: FailureCapture::default(),
        }
    }

    pub(super) fn take_failure(&mut self) -> Result<(), Failure> {
        self.capture.take()
    }

    // The number of values: 3, 5 or 7.
    fn dimension(&self) -> i32 {
        2 * self.case + 1
    }

    // Trim (cases 1 and 2), then EvalCase<n> into Res(0..dimension).
    fn eval(
        &mut self,
        start_end: [f64; 2],
        s: f64,
        order: i32,
    ) -> Result<(bool, Array1<'static, f64>), Failure> {
        if self.case != 3 && start_end != self.start_end_sav {
            self.fonct
                .trim(start_end[0], start_end[1], precision::confusion())?;
            self.start_end_sav = start_end;
        }
        let mut res: Array1<'static, f64> = Array1::with_bounds(0, self.dimension() - 1)
            .map_err(|e| array_failure(e, "Approx_CurvilinearParameter"))?;
        let done: bool = match self.case {
            1 => self.fonct.eval_case1(s, order, &mut res)?,
            2 => self.fonct.eval_case2(s, order, &mut res)?,
            _ => self.fonct.eval_case3(s, order, &mut res)?,
        };
        Ok((done, res))
    }
}

impl AdvApproxEvaluatorFunction for CurvlinEvaluator {
    fn evaluate(
        &mut self,
        dimension: i32,
        start_end: [f64; 2],
        s: f64,
        order: i32,
        result: &mut [f64],
    ) -> i32 {
        let mut error_code: i32 = 0;
        if dimension != self.dimension() {
            error_code = 1;
        }
        if s < start_end[0] || s > start_end[1] {
            error_code = 2;
        }
        let evaluated = self.eval(start_end, s, order);
        let Some((done, res)) = self.capture.keep(evaluated) else {
            return 1;
        };
        if !done {
            error_code = 3;
        }
        for (slot, value) in result.iter_mut().zip(res.iter()) {
            *slot = *value;
        }
        error_code
    }
}
