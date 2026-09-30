// Rust translation of the continuity-interval reads repeated in the GCPnts sources
// (`NCollection_Array1<double> aTI(1, NbIntervals + 1); C.Intervals(aTI, S);`) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The continuity intervals of a curve, as the GCPnts algorithms read them.
use crate::cpnts::PntsCurve;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bsplclib;
use rustycad_tkmath::geom_abs::Shape;

/// The `NbIntervals(S) + 1` bounds of the intervals of continuity `shape`, indexed
/// from 1; `interval_bounds(curve, Shape::CN)?`.
pub(super) fn interval_bounds(
    curve: PntsCurve<'_>,
    shape: Shape,
) -> Result<Array1<'static, f64>, Failure> {
    let nb_intervals: i32 = curve.nb_intervals(shape)?;
    let mut bounds: Array1<'static, f64> = Array1::with_bounds(1, nb_intervals + 1)
        .map_err(|error| error.into_failure("NCollection_Array1"))?;
    curve.intervals(&mut bounds, shape)?;
    Ok(bounds)
}

/// `aTI(index)`, Standard_OutOfRange outside the bounds; `bound(&bounds, 1)?`.
pub(super) fn bound(bounds: &Array1<'_, f64>, index: i32) -> Result<f64, Failure> {
    bounds
        .value(index)
        .copied()
        .map_err(|error| error.into_failure("NCollection_Array1::Value"))
}

/// `BSplCLib::Hunt(aTI, U, Index)`: the interval of `u`, `Lower() - 1` before and
/// `Upper() + 1` after the bounds (the incoming index is not read natively);
/// `hunt_interval(&bounds, 0.5)?`.
pub(super) fn hunt_interval(bounds: &Array1<'_, f64>, u: f64) -> Result<i32, Failure> {
    let mut index: i32 = 0;
    bsplclib::hunt(bounds, u, &mut index)?;
    Ok(index)
}
