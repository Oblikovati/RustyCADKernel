//! Modern optimization algorithms with native status and callback semantics.
use crate::math::{Matrix, MultipleVarFunction, MultipleVarFunctionWithGradient, Vector};
use crate::math_utils::{
    Config, LineSearchResult, Status, VectorResult, ZERO_TOL, exact_line_search,
};
mod bfgs;
mod powell;
pub mod utils;
pub use bfgs::{bfgs, bfgs_numerical};
pub use powell::{powell, powell_with_directions};

mod scalar;
pub use scalar::{brent, brent_with_bracket, golden};

mod frpr;
pub use frpr::{ConjugateGradientFormula, FrprConfig, frpr, frpr_numerical};

mod bounds;
mod newton;
pub use bfgs::bfgs_bounded;
pub use newton::{
    HessianFunction, NewtonConfig, newton, newton_bounded, newton_modified, newton_numerical,
    newton_numerical_hessian,
};

mod lbfgs;
pub use lbfgs::lbfgs;

mod pso;
pub use pso::*;

mod global;
pub use global::*;

mod uzawa;
pub use uzawa::{UzawaConfig, UzawaResult, uzawa, uzawa_equality};
