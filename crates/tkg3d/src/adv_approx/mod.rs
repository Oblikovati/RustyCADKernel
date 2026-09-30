// Rust translation of ModelingData/TKG3d/AdvApprox at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The AdvApprox package: approximation of a function of one parameter, with values
//! in a product of 1D, 2D and 3D spaces, by B-splines with shared knots.
//!
//! The native classes keep a `void*` to a caller-owned `AdvApprox_EvaluatorFunction`;
//! here the evaluator is a `&mut dyn AdvApproxEvaluatorFunction` borrowed for the
//! computation (closures implement it), and the cutting strategies implement
//! [`AdvApproxCutting`]. The native result handles become `Option` values that are
//! `None` where the native handle is null.
mod approx_a_function;
mod approximation;
mod cutting;
mod evaluator;
mod prepare_convert;
mod results;
mod simple_approx;

pub use approx_a_function::{AdvApproxApproxAFunction, SubSpaceTolerances};
pub use approximation::{AdvApproxApproximationInput, AdvApproxApproximationOutput, approximation};
pub use cutting::{AdvApproxCutting, AdvApproxDichoCutting, AdvApproxPrefAndRec};
pub use evaluator::AdvApproxEvaluatorFunction;
pub use simple_approx::AdvApproxSimpleApprox;

use rustycad_tkernel::tcollection::AsciiString;

/// A double as `std::ostream <<` prints it by default (six significant digits, `%g`);
/// `stream_real(0.5) == "0.5"`.
pub(crate) fn stream_real(value: f64) -> String {
    String::from_utf8_lossy(AsciiString::from_real(value).as_bytes()).into_owned()
}
