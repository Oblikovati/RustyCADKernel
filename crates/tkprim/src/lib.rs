// Rust translation of ModelingAlgorithms/TKPrim at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT TKPrim: the primitive solids of ModelingAlgorithms (boxes, wedges, cylinders,
//! cones, spheres, tori, revolved bodies, half-spaces) and the linear and rotational
//! sweeps (prisms and revolutions) of any shape.
//!
//! TKPrim depends on TKBRep, TKernel, TKMath, TKG2d, TKGeomBase, TKG3d and TKTopAlgo
//! natively (TKTopAlgo brings TKGeomAlgo); this crate depends on their ports. Each of
//! the 5 OCCT packages is one module named after it in snake case (`BRepPrimAPI` is
//! `brep_prim_api`, `BRepPreviewAPI` is `brep_preview_api`); every package is ported.
//! docs/prim-design.md fixes how the C++ constructs of the toolkit map to Rust, and
//! docs/tkprim-progress.md records the verification and the preserved native behaviour.
pub mod brep_preview_api;
pub mod brep_prim;
pub mod brep_prim_api;
pub mod brep_sweep;
pub mod sweep;
