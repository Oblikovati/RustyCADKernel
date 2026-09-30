//! Port of MathSys_NewtonTypes/2D/3D/4D at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use crate::math_utils::{Config, Status};
/// Native fixed dimension result; `NewtonResultN::<2>::default()`.
#[derive(Clone, Copy, Debug)]
pub struct NewtonResultN<const N: usize> {
    /// Native status field; `value.status`.
    pub status: Status,
    /// Native x field; `value.x`.
    pub x: [f64; N],
    /// Native iterations field; `value.iterations`.
    pub iterations: u32,
    /// Native residual_norm field; `value.residual_norm`.
    pub residual_norm: f64,
    /// Native step_norm field; `value.step_norm`.
    pub step_norm: f64,
}
impl<const N: usize> Default for NewtonResultN<N> {
    fn default() -> Self {
        Self {
            status: Status::NotConverged,
            x: [0.0; N],
            iterations: 0,
            residual_norm: 0.0,
            step_norm: 0.0,
        }
    }
}
impl<const N: usize> NewtonResultN<N> {
    /// Tests native convergence status; `result.is_done()`.
    pub fn is_done(&self) -> bool {
        self.status == Status::Ok
    }
}
impl<const N: usize> From<&NewtonResultN<N>> for bool {
    fn from(result: &NewtonResultN<N>) -> Self {
        result.is_done()
    }
}
/// Native bounds default to a zero width box; `NewtonBoundsN::<3>::default()`.
#[derive(Clone, Copy, Debug)]
pub struct NewtonBoundsN<const N: usize> {
    /// Native min field; `value.min`.
    pub min: [f64; N],
    /// Native max field; `value.max`.
    pub max: [f64; N],
    /// Native has_bounds field; `value.has_bounds`.
    pub has_bounds: bool,
}
impl<const N: usize> Default for NewtonBoundsN<N> {
    fn default() -> Self {
        Self {
            min: [0.0; N],
            max: [0.0; N],
            has_bounds: true,
        }
    }
}
/// Specialized native tolerances and step policy; `NewtonOptions::default()`.
#[derive(Clone, Copy, Debug)]
pub struct NewtonOptions {
    /// Native solver field; `value.solver`.
    pub solver: Config,
    /// Native max_step_ratio field; `value.max_step_ratio`.
    pub max_step_ratio: f64,
    /// Native enable_line_search field; `value.enable_line_search`.
    pub enable_line_search: bool,
    /// Native allow_soft_bounds field; `value.allow_soft_bounds`.
    pub allow_soft_bounds: bool,
    /// Native soft_bounds_extension field; `value.soft_bounds_extension`.
    pub soft_bounds_extension: f64,
}
impl Default for NewtonOptions {
    fn default() -> Self {
        Self {
            solver: Config {
                x_tolerance: 1e-16,
                ..Config::default()
            },
            max_step_ratio: 0.5,
            enable_line_search: true,
            allow_soft_bounds: false,
            soft_bounds_extension: 1e-4,
        }
    }
}
impl NewtonOptions {
    /// Validates the fields used by the specialized solvers (`IsOptionsValid`); `options.is_valid()`.
    pub fn is_valid(&self) -> bool {
        super::box_bounds::is_options_valid(self)
    }
}
impl<const N: usize> NewtonBoundsN<N> {
    /// Inactive bounds ignore their coordinates, matching OCCT (`IsBoundsValid*D`); `bounds.is_valid()`.
    pub fn is_valid(&self) -> bool {
        super::box_bounds::bounds_valid(self)
    }
}
