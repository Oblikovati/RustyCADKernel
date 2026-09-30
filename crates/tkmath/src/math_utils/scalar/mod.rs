//! Scalar functors from MathUtils_FunctorScalar.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
use crate::math::{Function, FunctionWithDerivative, Vector};
mod arithmetic;
mod elementary;
mod lambda;
mod polynomial;
pub use arithmetic::*;
pub use elementary::*;
pub use lambda::*;
pub use polynomial::*;

mod power_gaussian;
pub use power_gaussian::*;
