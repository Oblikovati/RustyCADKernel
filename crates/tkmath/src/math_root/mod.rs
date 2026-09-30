//! Modern scalar root solvers retaining OCCT status and optional result fields.
mod newton;
pub use newton::{newton, newton_bounded};

mod brent;
pub use brent::brent;

mod secant;
pub use secant::{secant, secant_auto};

mod bisection;
mod bracket;
pub use bisection::{bisection, bisection_newton};

mod trig;
pub mod utils;
pub use trig::{TrigResult, trigonometric, trigonometric_cde, trigonometric_linear};

mod multiple;
pub use multiple::{
    MultipleConfig, MultipleResult, find_all_roots, find_all_roots_samples,
    find_all_roots_with_derivative,
};
mod all;
pub use all::{
    AllRootsResult, NullInterval, find_all_roots_at_samples, find_all_roots_with_intervals,
};
