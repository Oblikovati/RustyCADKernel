// Rust translation of the weight checks of the Geom2dEval_TBezierCurve, Geom2dEval_AHTBezierCurve (ModelingData/TKG2d/Geom2dEval), GeomEval_TBezierCurve and GeomEval_AHTBezierCurve (ModelingData/TKG3d/GeomEval) constructors at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The pole and weight storage shared by the T-Bezier and AHT-Bezier curves of
//! Geom2dEval and GeomEval: the owning array copy of `myPoles(thePoles)` and the
//! weight checks of the rational constructors, which the four classes run in the
//! same order.
use crate::geom2d::errors;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

/// An owning copy that keeps the source bounds, as the NCollection_Array1 copy
/// constructor; `let stored: Array1<'static, f64> = owned(&weights);`.
pub fn owned<T: Clone>(source: &Array1<'_, T>) -> Array1<'static, T> {
    let mut copy: Array1<'static, T> = Array1::default();
    copy.assign(source);
    copy
}

/// Standard_ConstructionError unless there is one weight per pole and every weight
/// is > 0, checked from `Lower()` upwards; `check_weights("GeomEval_TBezierCurve", &w, 3)?`.
pub fn check_weights(class: &str, weights: &Array1<'_, f64>, count: usize) -> Result<(), Failure> {
    if weights.len() != count {
        return Err(errors::construction(&format!(
            "{class}: {} weights; expected {count} (one per pole)",
            weights.len()
        )));
    }
    let lower: i32 = weights.lower();
    for (offset, &weight) in weights.as_slice().iter().enumerate() {
        if weight <= 0.0 {
            let index: i64 = i64::from(lower) + offset as i64;
            return Err(errors::construction(&format!(
                "{class}: weight({index}) = {weight}; expected every weight > 0"
            )));
        }
    }
    Ok(())
}
