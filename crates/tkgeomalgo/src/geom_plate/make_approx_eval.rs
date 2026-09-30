// Rust translation of the evaluator class GeomPlate_MakeApprox_Eval of
// ModelingAlgorithms/TKGeomAlgo/GeomPlate/GeomPlate_MakeApprox.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomPlate_MakeApprox_Eval`: the plate surface as an AdvApp2Var function.
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkg3d::geom::GeomSurface;
use rustycad_tkgeombase::adv_app2var::{AdvApp2VarEvaluation, AdvApp2VarEvaluatorFunc2Var};
use rustycad_tkgeombase::failure_capture::FailureCapture;
use rustycad_tkmath::gp::Vector3;
use std::cell::RefCell;

/// The evaluator of `GeomPlate_MakeApprox` on a surface. A failure of the surface
/// evaluation is kept, reported by the error code 3 and raised by the caller when the
/// approximation returns (docs/geomalgo-design.md section 19).
pub(super) struct MakeApproxEval {
    surf: Handle<dyn GeomSurface>,
    capture: RefCell<FailureCapture>,
}

/// `Result[k + (jpar - 1) * Dimension] = v(k)` for the three coordinates.
fn store(result: &mut [f64], jpar: i32, dimension: i32, v: [f64; 3]) -> Result<(), Failure> {
    let start: usize = ((jpar - 1) * dimension).max(0) as usize;
    let length: usize = result.len();
    let slot = result.get_mut(start..start + 3).ok_or_else(|| {
        errors::out_of_range(&format!(
            "GeomPlate_MakeApprox_Eval: result of {length} values; expected at least {}",
            start + 3
        ))
    })?;
    slot.copy_from_slice(&v);
    Ok(())
}

fn coords(v: Vector3) -> [f64; 3] {
    [v.x(), v.y(), v.z()]
}

/// The native parameter checks: error code 2 when a parameter leaves its range.
fn parameter_code(request: &AdvApp2VarEvaluation<'_>) -> bool {
    let (constant, others) = if request.favor_iso == 1 {
        (request.u_start_end, request.v_start_end)
    } else {
        (request.v_start_end, request.u_start_end)
    };
    let mut out: bool = request.const_param < constant[0] || request.const_param > constant[1];
    for &t in request
        .parameters
        .iter()
        .take(request.nb_params.max(0) as usize)
    {
        out |= t < others[0] || t > others[1];
    }
    out
}

impl MakeApproxEval {
    /// `GeomPlate_MakeApprox_Eval(theSurf)`.
    pub(super) fn new(surf: &Handle<dyn GeomSurface>) -> Self {
        Self {
            surf: surf.clone(),
            capture: RefCell::new(FailureCapture::default()),
        }
    }

    /// The kept failure of an evaluation, if any; `ev.take_failure()?`.
    pub(super) fn take_failure(&self) -> Result<(), Failure> {
        self.capture.borrow_mut().take()
    }

    /// The value of the requested derivative at `(u, v)`: the point for order 0, the
    /// U or V derivative for order 1, and D2U, D2UV or D2V for U orders 2, 1 or 0.
    fn derivative(
        &self,
        u: f64,
        v: f64,
        order: i32,
        u_order: i32,
    ) -> Result<Option<[f64; 3]>, Failure> {
        let surface = super::live(&self.surf, "the plate surface")?;
        match order {
            0 => {
                let p = surface.value(u, v)?;
                Ok(Some([p.x(), p.y(), p.z()]))
            }
            1 => {
                let d1 = surface.d1(u, v)?;
                Ok(Some(coords(if u_order == 1 { d1.d1u } else { d1.d1v })))
            }
            2 => {
                let d2 = surface.d2(u, v)?;
                Ok(match u_order {
                    2 => Some(coords(d2.d2u)),
                    1 => Some(coords(d2.d2uv)),
                    0 => Some(coords(d2.d2v)),
                    _ => None,
                })
            }
            _ => Ok(None),
        }
    }

    /// Fills `result` as `Evaluate` does, after zeroing it.
    fn fill(&self, request: &AdvApp2VarEvaluation<'_>, result: &mut [f64]) -> Result<(), Failure> {
        let dimension: i32 = request.dimension;
        let count: usize = (dimension.max(0) * request.nb_params.max(0)) as usize;
        result.iter_mut().take(count).for_each(|value| *value = 0.0);
        let order: i32 = request.u_order + request.v_order;
        for jpar in 1..=request.nb_params {
            let t: f64 = request.parameters[(jpar - 1) as usize];
            let (u, v) = if request.favor_iso == 1 {
                (request.const_param, t)
            } else {
                (t, request.const_param)
            };
            if let Some(value) = self.derivative(u, v, order, request.u_order)? {
                store(result, jpar, dimension, value)?;
            }
        }
        Ok(())
    }
}

impl AdvApp2VarEvaluatorFunc2Var for MakeApproxEval {
    /// `Evaluate`: error code 1 for a dimension other than 3 and 2 for a parameter
    /// out of range (the values are computed anyway, as natively), 3 for a kept
    /// failure of the surface.
    fn evaluate(
        &self,
        request: &AdvApp2VarEvaluation<'_>,
        result: &mut [f64],
        error_code: &mut i32,
    ) {
        *error_code = 0;
        if request.dimension != 3 {
            *error_code = 1;
        }
        if parameter_code(request) {
            *error_code = 2;
        }
        let filled = self.fill(request, result);
        if self.capture.borrow_mut().keep(filled).is_none() {
            *error_code = 3;
        }
    }
}
