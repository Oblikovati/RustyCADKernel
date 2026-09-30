// Rust translation of the curve-surface classes of Extrema at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The extrema of the distance between a curve and a surface: the function
//! FuncExtCS, the global-optimization functions (GlobOptFuncCS, GlobOptFuncConicS,
//! GlobOptFuncCQuadric), the global search GenExtCS and the general class ExtCS.
mod ext_cs;
mod func_ext_cs;
mod gen_ext_cs;
mod glob_opt_func_c_quadric;
mod glob_opt_func_conic_s;
mod glob_opt_func_cs;

pub use ext_cs::ExtCS;
pub use func_ext_cs::FuncExtCS;
pub use gen_ext_cs::GenExtCS;
pub use glob_opt_func_c_quadric::GlobOptFuncCQuadric;
pub use glob_opt_func_conic_s::GlobOptFuncConicS;
pub use glob_opt_func_cs::GlobOptFuncCS;
