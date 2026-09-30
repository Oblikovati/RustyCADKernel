// Rust translation of the template helpers of ModelingData/TKG3d/GeomGridEval/GeomGridEval.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The public `GeomGridEval::Evaluate*Helper` templates: the U x V iteration of a
//! parametric surface grid, with the computation delegated to a functor.
use super::surface_grid::{any_empty, empty_grid, map_rows};
use super::{GeomGridEvalSurfD1, GeomGridEvalSurfD2, GeomGridEvalSurfD3};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkmath::gp::{Point3, Vector3};

// The shared body of the five templates: empty when either array is empty, otherwise
// `eval(u, v)` into cell `(iU, iV)`, U outer and V inner.
fn helper_grid<T: Default>(
    u_params: &Array1<'_, f64>,
    v_params: &Array1<'_, f64>,
    mut eval: impl FnMut(f64, f64) -> T,
) -> Array2<'static, T> {
    if any_empty(u_params, v_params) {
        return empty_grid();
    }
    map_rows(u_params, v_params, |u| u, |&u, v| eval(u, v))
}

/// Grid of points from a point functor (`GeomGridEval::EvaluateGridHelper`);
/// `evaluate_grid_helper(&u, &v, |u, v| Point3::new(u, v, 0.0))`.
pub fn evaluate_grid_helper(
    u_params: &Array1<'_, f64>,
    v_params: &Array1<'_, f64>,
    eval: impl FnMut(f64, f64) -> Point3,
) -> Array2<'static, Point3> {
    helper_grid(u_params, v_params, eval)
}

/// Grid of D1 records from a functor (`GeomGridEval::EvaluateGridD1Helper`);
/// `evaluate_grid_d1_helper(&u, &v, |u, v| surface_d1(u, v))`.
pub fn evaluate_grid_d1_helper(
    u_params: &Array1<'_, f64>,
    v_params: &Array1<'_, f64>,
    eval: impl FnMut(f64, f64) -> GeomGridEvalSurfD1,
) -> Array2<'static, GeomGridEvalSurfD1> {
    helper_grid(u_params, v_params, eval)
}

/// Grid of D2 records from a functor (`GeomGridEval::EvaluateGridD2Helper`);
/// `evaluate_grid_d2_helper(&u, &v, |u, v| surface_d2(u, v))`.
pub fn evaluate_grid_d2_helper(
    u_params: &Array1<'_, f64>,
    v_params: &Array1<'_, f64>,
    eval: impl FnMut(f64, f64) -> GeomGridEvalSurfD2,
) -> Array2<'static, GeomGridEvalSurfD2> {
    helper_grid(u_params, v_params, eval)
}

/// Grid of D3 records from a functor (`GeomGridEval::EvaluateGridD3Helper`);
/// `evaluate_grid_d3_helper(&u, &v, |u, v| surface_d3(u, v))`.
pub fn evaluate_grid_d3_helper(
    u_params: &Array1<'_, f64>,
    v_params: &Array1<'_, f64>,
    eval: impl FnMut(f64, f64) -> GeomGridEvalSurfD3,
) -> Array2<'static, GeomGridEvalSurfD3> {
    helper_grid(u_params, v_params, eval)
}

/// Grid of derivative vectors from a functor (`GeomGridEval::EvaluateGridDNHelper`);
/// `evaluate_grid_dn_helper(&u, &v, |u, v| Vector3::new(u, v, 0.0))`.
pub fn evaluate_grid_dn_helper(
    u_params: &Array1<'_, f64>,
    v_params: &Array1<'_, f64>,
    eval: impl FnMut(f64, f64) -> Vector3,
) -> Array2<'static, Vector3> {
    helper_grid(u_params, v_params, eval)
}
