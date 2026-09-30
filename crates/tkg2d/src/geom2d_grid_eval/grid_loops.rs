// Rust translation of the parameter loops shared by ModelingData/TKG2d/Geom2dGridEval/*.cxx and ModelingData/TKG3d/GeomGridEval/*.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! Parameter loops of the grid evaluators: 1-based result arrays filled in parameter
//! order. Every Geom2dGridEval and GeomGridEval curve evaluator repeats these loops
//! natively; the port writes them once and TKG3d reuses them.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;

/// An empty result array (`NCollection_Array1<T>()`, bounds `1..0`);
/// `grid_loops::empty::<f64>().is_empty()`.
pub fn empty<T>() -> Array1<'static, T> {
    Array1::default()
}

/// A default-filled result array of bounds `1..len` (`NCollection_Array1<T>(1, len)`);
/// `grid_loops::result_array::<f64>(3).upper() == 3`.
pub fn result_array<T: Default>(len: usize) -> Array1<'static, T> {
    let mut array: Array1<'static, T> = Array1::with_size(len);
    array.update_lower_bound(1);
    array
}

/// Evaluates `eval` at each parameter from `Lower()` to `Upper()` into slot
/// `i - Lower() + 1`; the caller has already rejected an empty parameter array;
/// `grid_loops::map_grid(&params, |u| 2.0 * u)`.
pub fn map_grid<T: Default>(
    params: &Array1<'_, f64>,
    mut eval: impl FnMut(f64) -> T,
) -> Array1<'static, T> {
    let mut result: Array1<'static, T> = result_array(params.len());
    for (slot, &u) in result.iter_mut().zip(params.iter()) {
        *slot = eval(u);
    }
    result
}

/// `map_grid` for evaluators that can raise: the first failure aborts the grid, as
/// the native exception leaves the loop; `grid_loops::try_map_grid(&params, |u| curve.eval_d0(u))?`.
pub fn try_map_grid<T: Default>(
    params: &Array1<'_, f64>,
    mut eval: impl FnMut(f64) -> Result<T, Failure>,
) -> Result<Array1<'static, T>, Failure> {
    let mut result: Array1<'static, T> = result_array(params.len());
    for (slot, &u) in result.iter_mut().zip(params.iter()) {
        *slot = eval(u)?;
    }
    Ok(result)
}

/// Fills every slot of a `1..len` array with the same value (the constant-derivative
/// loops `for (i = 1; i <= aNb; ++i) aResult.SetValue(i, aConst)`);
/// `grid_loops::constant_grid(4, 0.0)`.
pub fn constant_grid<T: Default + Copy>(len: usize, value: T) -> Array1<'static, T> {
    let mut result: Array1<'static, T> = result_array(len);
    result.init(value);
    result
}

/// One derivative of every record of a grid, in order (the `aResult.SetValue(i,
/// aGrid.Value(i).DK)` loops of the offset and fallback DN);
/// `grid_loops::column(&d1_grid, |r| r.d1)`.
pub fn column<R, V: Default>(grid: &Array1<'_, R>, pick: impl Fn(&R) -> V) -> Array1<'static, V> {
    let mut result: Array1<'static, V> = result_array(grid.len());
    for (slot, record) in result.iter_mut().zip(grid.iter()) {
        *slot = pick(record);
    }
    result
}

/// The derivative coefficients of `(cos, sin)` for derivative order `n >= 1`, cyclic
/// with period 4 (the `switch ((theN - 1) % 4)` of the circle and ellipse DN);
/// `grid_loops::cyclic_coefficients(1, c, s) == (-s, c)`.
pub fn cyclic_coefficients(n: i32, cos_u: f64, sin_u: f64) -> (f64, f64) {
    match (n - 1) % 4 {
        0 => (-sin_u, cos_u),
        1 => (-cos_u, -sin_u),
        2 => (sin_u, -cos_u),
        _ => (cos_u, sin_u),
    }
}
