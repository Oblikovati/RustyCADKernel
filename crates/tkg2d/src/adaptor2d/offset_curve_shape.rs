// Rust translation of ModelingData/TKG2d/Adaptor2d/Adaptor2d_OffsetCurve.cxx (Continuity, NbIntervals, Intervals) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Continuity and continuity intervals of `Adaptor2d_OffsetCurve`.
//!
//! An offset loses two orders of continuity, so the intervals of order `S` are the
//! basis intervals of order `S + 2` that meet the offset bounds.
use super::offset_curve::OffsetState;
use crate::geom2d::errors::{array_bounds, type_mismatch};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::geom_abs::Shape;

/// The offset continuity from the basis one (`Continuity`); Standard_TypeMismatch on a
/// C0 basis; `continuity(&state)?`.
pub(super) fn continuity(state: &OffsetState) -> Result<Shape, Failure> {
    Ok(match state.basis()?.continuity()? {
        Shape::CN => Shape::CN,
        Shape::C3 => Shape::C2,
        Shape::C2 => Shape::G2,
        Shape::G2 => Shape::C1,
        Shape::C1 => Shape::G1,
        Shape::G1 => Shape::C0,
        // No continuity: the native message names IntervalContinuity.
        Shape::C0 => return Err(type_mismatch("Adaptor2d_OffsetCurve::IntervalContinuity")),
    })
}

// The basis continuity whose intervals bound those of `shape`: two orders higher, CN
// from C2 on (the native `(GeomAbs_Shape)((int)S + 2)`).
fn basis_shape(shape: Shape) -> Shape {
    match shape {
        Shape::C0 => Shape::C1,
        Shape::G1 => Shape::G2,
        Shape::C1 => Shape::C2,
        Shape::G2 => Shape::C3,
        Shape::C2 | Shape::C3 | Shape::CN => Shape::CN,
    }
}

// The basis intervals of `shape` in an array `1..=count+1`.
fn basis_intervals(
    state: &OffsetState,
    shape: Shape,
    count: i32,
) -> Result<Array1<'static, f64>, Failure> {
    let mut bounds: Array1<'static, f64> =
        Array1::with_bounds(1, count + 1).map_err(array_bounds)?;
    state.basis()?.intervals(&mut bounds, shape)?;
    Ok(bounds)
}

// The first basis bound above `first` and the last one below `last`, scanning as the
// native loops do; an index leaving the array raises Standard_OutOfRange.
fn clip_indices(
    bounds: &Array1<'_, f64>,
    first: f64,
    last: f64,
    count: i32,
) -> Result<(i32, i32), Failure> {
    let mut first_index: i32 = 1;
    while *bounds.value(first_index).map_err(array_bounds)? <= first {
        first_index += 1;
    }
    let mut last_index: i32 = count + 1;
    while *bounds.value(last_index).map_err(array_bounds)? >= last {
        last_index -= 1;
    }
    Ok((first_index, last_index))
}

/// Number of intervals of continuity `shape` (`NbIntervals`); `nb_intervals(&state, Shape::C1)?`.
pub(super) fn nb_intervals(state: &OffsetState, shape: Shape) -> Result<i32, Failure> {
    let basis: Shape = basis_shape(shape);
    let count: i32 = state.basis()?.nb_intervals(basis)?;
    if count == 1 {
        return Ok(count);
    }
    let bounds = basis_intervals(state, basis, count)?;
    let (first, last) = clip_indices(&bounds, state.first, state.last, count)?;
    Ok(last - first + 2)
}

/// Interval bounds of continuity `shape`, the first and last replaced by the offset
/// bounds (`Intervals`); `intervals(&state, &mut out, Shape::C1)?`.
pub(super) fn intervals(
    state: &OffsetState,
    out: &mut Array1<'_, f64>,
    shape: Shape,
) -> Result<(), Failure> {
    let basis: Shape = basis_shape(shape);
    let count: i32 = state.basis()?.nb_intervals(basis)?;
    let lower: i32 = out.lower();
    if count == 1 {
        out.set_value(lower, state.first).map_err(array_bounds)?;
        return out.set_value(lower + 1, state.last).map_err(array_bounds);
    }
    let bounds = basis_intervals(state, basis, count)?;
    let (first, last) = clip_indices(&bounds, state.first, state.last, count)?;
    for (i, j) in (lower..).zip(first - 1..=last + 1) {
        let value: f64 = *bounds.value(j).map_err(array_bounds)?;
        out.set_value(i, value).map_err(array_bounds)?;
    }
    out.set_value(lower, state.first).map_err(array_bounds)?;
    out.set_value(lower + last - first + 2, state.last)
        .map_err(array_bounds)
}
