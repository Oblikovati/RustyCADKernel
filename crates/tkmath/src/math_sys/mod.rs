//! Nonlinear equation systems using native rank-aware linear policies.
mod types;
pub use types::SystemResult;
pub mod linear;
mod newton;
pub use newton::{newton, newton_bounded, newton_uniform};

mod lm;
pub use lm::{LMConfig, levenberg_marquardt, levenberg_marquardt_bounded};

mod small;
pub use small::{
    CurveD2, CurveEvaluatorD2, NewtonBoundsN, NewtonOptions, NewtonResultN, SmallSystem, SurfaceD2,
    SurfaceEvaluatorD2, SymmetricFunction2D, solve_2d, solve_2d_symmetric, solve_3d, solve_4d,
    solve_curve_surface_extrema_3d, solve_surface_surface_extrema_4d,
};

pub mod utils;
