// Rust translation of ModelingData/TKG2d/Geom2d (NCollection_Array1 access shared by the Bezier and BSpline curves) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Checked `NCollection_Array1` access with the native exception types.
use super::errors;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

/// A new array with bounds `1..=count` (`NCollection_Array1<T>(1, count)`);
/// `one_based::<f64>(4)?.upper() == 4`.
pub fn one_based<T: Default>(count: i32) -> Result<Array1<'static, T>, Failure> {
    Array1::with_bounds(1, count).map_err(|error| {
        errors::out_of_range(&format!(
            "NCollection_Array1(1, {count}): {error}; expected count >= 0"
        ))
    })
}

/// `array(index)` with the native Standard_OutOfRange check; `at(&poles, 1)?`.
pub fn at<T: Copy>(array: &Array1<'_, T>, index: i32) -> Result<T, Failure> {
    array.value(index).copied().map_err(errors::array_bounds)
}

/// `array(index) = value` with the native Standard_OutOfRange check; `put(&mut poles, 1, p)?`.
pub fn put<T>(array: &mut Array1<'_, T>, index: i32, value: T) -> Result<(), Failure> {
    array.set_value(index, value).map_err(errors::array_bounds)
}

/// Copy `source` into a new owning array with bounds `1..=len`
/// (`Resize(1, n, false)` then `CopyValues`); `rebased(&poles)?`.
pub fn rebased<T: Default + Clone>(source: &Array1<'_, T>) -> Result<Array1<'static, T>, Failure> {
    let mut target: Array1<'static, T> = one_based(source.len() as i32)?;
    target.as_mut_slice().clone_from_slice(source.as_slice());
    Ok(target)
}

/// Whether consecutive weights differ by more than gp::Resolution, reading
/// `weights(i)` and `weights(i + 1)` for `i` in `first..last`; `any_weight_step(&w, 1, 3)?`.
pub fn any_weight_step(weights: &Array1<'_, f64>, first: i32, last: i32) -> Result<bool, Failure> {
    for i in first..last {
        let step: f64 = at(weights, i)? - at(weights, i + 1)?;
        if step.abs() > rustycad_tkmath::gp::resolution() {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Copy `source` into `target` (one more element) with `value` inserted after
/// one-based position `index`, as the native InsertPoleAfter loops do;
/// `insert_at(&poles, &mut new_poles, 2, p)?`.
pub fn insert_at<T: Copy>(
    source: &Array1<'_, T>,
    target: &mut Array1<'_, T>,
    index: i32,
    value: T,
) -> Result<(), Failure> {
    for i in 1..=index {
        put(target, i, at(source, i)?)?;
    }
    put(target, index + 1, value)?;
    for i in (index + 1)..=(source.len() as i32) {
        put(target, i + 1, at(source, i)?)?;
    }
    Ok(())
}

/// Copy `source` into `target` (one less element) without one-based position
/// `index`, as the native RemovePole loops do; `remove_at(&poles, &mut new_poles, 2)?`.
pub fn remove_at<T: Copy>(
    source: &Array1<'_, T>,
    target: &mut Array1<'_, T>,
    index: i32,
) -> Result<(), Failure> {
    for i in 1..index {
        put(target, i, at(source, i)?)?;
    }
    for i in index..=(target.len() as i32) {
        put(target, i, at(source, i + 1)?)?;
    }
    Ok(())
}

/// The weights that InsertPoleAfter reads: the stored ones of a rational curve,
/// unit weights otherwise (the native `IsRational() ? W(i) : 1.`);
/// `stored_or_unit_weights(false, &weights, 4)?`.
pub fn stored_or_unit_weights(
    rational: bool,
    weights: &Array1<'static, f64>,
    count: i32,
) -> Result<Array1<'static, f64>, Failure> {
    if rational {
        return Ok(weights.clone());
    }
    rustycad_tkmath::bsplclib::unit_weights(count)
}

/// The deprecated copying getters (`Poles(P)`, `Multiplicities(M)`): Standard_DimensionError
/// unless the lengths match, then the values in order; `copy_same_length(&poles, &mut p, "Geom2d_BezierCurve::Poles")?`.
pub fn copy_same_length<T: Copy>(
    source: &Array1<'_, T>,
    target: &mut Array1<'_, T>,
    member: &str,
) -> Result<(), Failure> {
    if target.len() != source.len() {
        return Err(errors::dimension(&format!(
            "{member}: length {}; expected {}",
            target.len(),
            source.len()
        )));
    }
    target.as_mut_slice().copy_from_slice(source.as_slice());
    Ok(())
}

/// The native `Index < 1 || Index > count` Standard_OutOfRange check of `member`;
/// `check_one_based(2, 4, "Geom2d_BezierCurve::SetPole")?`.
pub fn check_one_based(index: i32, count: usize, member: &str) -> Result<(), Failure> {
    if index < 1 || index > count as i32 {
        return Err(errors::out_of_range(&format!(
            "{member}: Index = {index}; expected in [1, {count}]"
        )));
    }
    Ok(())
}
