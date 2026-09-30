// Rust translation of ModelingAlgorithms/TKGeomAlgo/AppBlend/AppBlend_Debug.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AppBlend_Debug.cxx`: the process-wide switches of the approximation, file
//! statics natively (both false at start).
use std::sync::atomic::{AtomicBool, Ordering};

static CONTEXT_SPLINE_APPROX: AtomicBool = AtomicBool::new(false);
static CONTEXT_APPROX_WITH_NO_TGT: AtomicBool = AtomicBool::new(false);

/// `AppBlend_SetContextSplineApprox`: a switch the blending algorithms read to
/// choose the B-spline approximation; `set_context_spline_approx(true)`.
pub fn set_context_spline_approx(b: bool) {
    CONTEXT_SPLINE_APPROX.store(b, Ordering::Relaxed);
}

/// `AppBlend_GetContextSplineApprox`; `get_context_spline_approx()`.
pub fn get_context_spline_approx() -> bool {
    CONTEXT_SPLINE_APPROX.load(Ordering::Relaxed)
}

/// `AppBlend_SetContextApproxWithNoTgt`: makes the approximation ignore the section
/// tangents; `set_context_approx_with_no_tgt(true)`.
pub fn set_context_approx_with_no_tgt(b: bool) {
    CONTEXT_APPROX_WITH_NO_TGT.store(b, Ordering::Relaxed);
}

/// `AppBlend_GetContextApproxWithNoTgt`; `get_context_approx_with_no_tgt()`.
pub fn get_context_approx_with_no_tgt() -> bool {
    CONTEXT_APPROX_WITH_NO_TGT.load(Ordering::Relaxed)
}
