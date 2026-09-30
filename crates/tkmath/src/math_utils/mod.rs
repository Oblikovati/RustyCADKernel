//! OCCT modern solver utilities, separate from the legacy math package.
mod config;
mod convergence;
mod core;
mod domain;
mod extended;
mod glibc_cbrt;
mod types;
pub use config::*;
pub use convergence::*;
pub use core::*;
pub use domain::*;
pub use types::*;

mod bracket;
pub use bracket::*;

mod poly;
pub use poly::*;

mod deriv;
pub use deriv::*;

mod line_search;
pub use line_search::*;

mod functor_vector;
pub use functor_vector::*;

pub mod scalar;

pub mod vector;

pub(crate) use line_search::BrentState;

mod gauss;
pub use gauss::{
    get_gauss_points_and_weights, get_kronrod_points_and_weights,
    get_ordered_gauss_points_and_weights,
};

mod random;
pub use random::RandomGenerator;

/// `MathUtils::Utils` helpers (bracketing and finite-difference building blocks).
pub mod utils;
