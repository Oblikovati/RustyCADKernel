// Rust translation of ModelingData/TKGeomBase/AdvApp2Var at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The AdvApp2Var package: approximation of a function of two parameters by
//! polynomial patches and B-spline surfaces.
//!
//! The package has two layers. The C++ classes drive the cutting of the domain
//! into patches and iso-curves; the numerical work is f2c translations of
//! Fortran routines in [`math_base`], [`approx_f2var`] and [`sys_base`], over the
//! common-block tables of [`data`].
//!
//! The kernels keep the native routine names, argument order and loop structure,
//! so every floating-point operation happens in the native order. Scalars that the
//! native code only reads are passed by value, scalars it writes are `&mut`, and an
//! array argument is the slice that starts at the native pointer; inside, the f2c
//! pointer shift is kept by the views of the `f2c` module. The calls to the trace
//! routines (`mgenmsg_`, `mgsomsg_`, `maermsg_`) are left out of the kernels: the
//! debug level [`sys_base::mnfndeb_`] is 0 and the routines do nothing.
mod approx_a_func2var;
mod approx_a_func2var_perform;
mod approx_a_func2var_results;
mod arrays;
mod context;
mod criterion;
mod evaluator;
mod f2c;
mod framework;
mod framework_update;
mod iso;
mod iso_approx;
mod network;
mod node;
mod patch;
mod patch_constraints;
mod patch_discretise;
mod patch_make;

pub mod approx_f2var;
pub mod data;
pub mod math_base;
pub mod sys_base;

pub use approx_a_func2var::AdvApp2VarApproxAFunc2Var;
pub use context::{AdvApp2VarContext, FrontierTolerances, SubspaceTolerances};
pub use criterion::{AdvApp2VarCriterion, AdvApp2VarCriterionRepartition, AdvApp2VarCriterionType};
pub use evaluator::{AdvApp2VarEvaluation, AdvApp2VarEvaluatorFunc2Var};
pub use framework::{AdvApp2VarFramework, AdvApp2VarIsoStrip};
pub use iso::AdvApp2VarIso;
pub use network::AdvApp2VarNetwork;
pub use node::AdvApp2VarNode;
pub use patch::AdvApp2VarPatch;
