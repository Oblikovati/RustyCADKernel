//! Port of MathSys_NewtonTypes/2D/3D/4D at 3d097a0328e71b826377d4814ab05ec3c3d23871.
mod types;
pub use types::{NewtonBoundsN, NewtonOptions, NewtonResultN};
mod box_bounds;
mod checks;
mod engine;
mod linear;
mod symmetric;
use crate::math_utils::Status;
pub use box_bounds::{
    clamp_2d, clamp_3d, clamp_4d, is_bounds_valid_2d, is_bounds_valid_3d, is_bounds_valid_4d,
    is_options_valid, is_options_valid_3d, is_options_valid_4d, max_step_2d, max_step_3d,
    max_step_4d,
};
pub use checks::{
    ARMIJO_C1, CRITICAL_GRAD, CRITICAL_GRAD_SQ, LINE_SEARCH_MAX, NEWTON_PIVOT_TOL,
    is_armijo_accepted, is_finite_array, is_finite_system_n, is_step_within_tolerance,
    merit_directional_derivative, safe_distance_n, safe_norm_n,
};
pub use linear::{solve_2x2, solve_3x3, solve_4x4, solve_symmetric_2x2_svd};
pub use symmetric::{SymmetricFunction2D, solve_2d_symmetric};
/// Evaluates a fixed square residual and Jacobian; closures can implement this directly.
/// Example: `|x: &[f64; 2], f: &mut [f64; 2], j: &mut [[f64; 2]; 2]| { *f = *x; *j = [[1.0,0.0],[0.0,1.0]]; true }`.
pub trait SmallSystem<const N: usize> {
    /// Supplies every residual and Jacobian entry; `function.evaluate(&x, &mut f, &mut j)`.
    fn evaluate(&mut self, x: &[f64; N], f: &mut [f64; N], j: &mut [[f64; N]; N]) -> bool;
}
impl<const N: usize, F: FnMut(&[f64; N], &mut [f64; N], &mut [[f64; N]; N]) -> bool> SmallSystem<N>
    for F
{
    fn evaluate(&mut self, x: &[f64; N], f: &mut [f64; N], j: &mut [[f64; N]; N]) -> bool {
        self(x, f, j)
    }
}
struct General<'a, F>(&'a mut F);
impl<const N: usize, F: SmallSystem<N>> engine::Policy<N> for General<'_, F> {
    fn system(&mut self, x: &[f64; N]) -> Result<([f64; N], [[f64; N]; N]), Status> {
        let mut f: [f64; N] = [0.0; N];
        let mut j: [[f64; N]; N] = [[0.0; N]; N];
        if self.0.evaluate(x, &mut f, &mut j) && checks::is_finite_system_n(&f, &j) {
            Ok((f, j))
        } else {
            Err(Status::NumericalError)
        }
    }
    fn value_norm(&mut self, x: &[f64; N]) -> Result<f64, Status> {
        self.system(x).map(|(f, _)| checks::safe_norm_n(&f))
    }
    fn direction(&self, j: &[[f64; N]; N], f: &[f64; N], residual: f64) -> Option<[f64; N]> {
        linear::direction(j, f, residual)
    }
}
/// Native general 2D Newton method; `solve_2d(&mut function, [0.0; 2], &bounds, &options)`.
pub fn solve_2d<F: SmallSystem<2>>(
    function: &mut F,
    start: [f64; 2],
    bounds: &NewtonBoundsN<2>,
    options: &NewtonOptions,
) -> NewtonResultN<2> {
    engine::solve(&mut General(function), start, bounds, options)
}
/// Native general 3D Newton method; `solve_3d(&mut function, [0.0; 3], &bounds, &options)`.
pub fn solve_3d<F: SmallSystem<3>>(
    function: &mut F,
    start: [f64; 3],
    bounds: &NewtonBoundsN<3>,
    options: &NewtonOptions,
) -> NewtonResultN<3> {
    engine::solve(&mut General(function), start, bounds, options)
}
/// Native general 4D Newton method; `solve_4d(&mut function, [0.0; 4], &bounds, &options)`.
pub fn solve_4d<F: SmallSystem<4>>(
    function: &mut F,
    start: [f64; 4],
    bounds: &NewtonBoundsN<4>,
    options: &NewtonOptions,
) -> NewtonResultN<4> {
    engine::solve(&mut General(function), start, bounds, options)
}

mod geometry;
pub use geometry::*;
