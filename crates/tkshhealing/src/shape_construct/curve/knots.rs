use super::ShapeConstructCurve;
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Failure, Handle, real::epsilon};

impl ShapeConstructCurve {
    /// `FixKnots(knots)` on a handle: [`Self::fix_knots`] on the shared array;
    /// Standard_NullObject for a null handle; `ShapeConstructCurve::fix_knots_handle(&knots)?`.
    pub fn fix_knots_handle(knots: &Handle<HArray1<f64>>) -> Result<bool, Failure> {
        let array = knots.get().ok_or_else(|| {
            Failure::with_message(
                "Standard_NullObject",
                "ShapeConstruct_Curve::FixKnots: null knots; expected an HArray1 handle",
            )
        })?;
        Self::fix_knots(&mut array.change_array1())
    }
    /// `FixKnots(knots)`: moves each knot that is not above the previous one by more than
    /// `Epsilon(previous)` to `previous + 2 * Epsilon(previous)`; true when a knot moved.
    /// The knots are read from rank 1 to `Length()`, whatever the lower bound, as natively
    /// (Standard_OutOfRange when that leaves the array); `ShapeConstructCurve::fix_knots(&mut knots)?`.
    pub fn fix_knots(knots: &mut Array1<'_, f64>) -> Result<bool, Failure> {
        let mut fixed = false;
        let mut previous = knot_value(knots, 1)?;
        for index in 2..=knots.length() {
            let (next, changed) = fix_next_knot(knots, index, previous)?;
            fixed |= changed;
            previous = next;
        }
        Ok(fixed)
    }
}

fn knot_value(knots: &Array1<'_, f64>, index: i32) -> Result<f64, Failure> {
    let value = knots.value(index);
    value
        .copied()
        .map_err(|_| knot_bounds_failure(knots, index))
}

fn knot_bounds_failure(knots: &Array1<'_, f64>, index: i32) -> Failure {
    let bounds = (knots.lower(), knots.upper());
    Failure::with_message(
        "Standard_OutOfRange",
        &format!(
            "ShapeConstruct_Curve::FixKnots: index {index}, array {}..{}; expected index within array bounds",
            bounds.0, bounds.1
        ),
    )
}

fn fix_next_knot(
    knots: &mut Array1<'_, f64>,
    index: i32,
    previous: f64,
) -> Result<(f64, bool), Failure> {
    let next = knot_value(knots, index)?;
    if next - previous <= epsilon(previous) {
        let next = previous + 2.0 * epsilon(previous);
        knots
            .set_value(index, next)
            .map_err(|_| knot_bounds_failure(knots, index))?;
        return Ok((next, true));
    }
    Ok((next, false))
}
