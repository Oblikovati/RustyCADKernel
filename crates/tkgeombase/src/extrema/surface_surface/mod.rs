// Rust translation of the surface-surface classes of Extrema at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The extrema of the distance between two surfaces: the function FuncExtSS, the
//! sampled search GenExtSS and the general class ExtSS.
mod ext_ss;
mod func_ext_ss;
mod gen_ext_ss;

pub use ext_ss::ExtSS;
pub use func_ext_ss::FuncExtSS;
pub use gen_ext_ss::GenExtSS;
