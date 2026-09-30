//! Vector functors from MathUtils_FunctorVector.hxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
//! Explicit dimension metadata adapts variable-size functors to the shared Rust objective trait.
use crate::math::{
    FunctionSet, Matrix, MultipleVarFunction, MultipleVarFunctionWithGradient, Vector,
};
use crate::math_utils::{EULER_NUMBER, TWO_PI};
mod linear;
mod planar;
mod system;
mod variable;
pub use linear::*;
pub use planar::*;
pub use system::*;
pub use variable::*;
