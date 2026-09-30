// Rust translation of ModelingAlgorithms/TKTopAlgo at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT TKTopAlgo: the topological algorithms of ModelingAlgorithms (point and
//! solid classification, shape construction, validity checks, extrema, bounding
//! boxes, global properties and the medial axis of faces).
//!
//! TKTopAlgo depends on TKMath, TKernel, TKG2d, TKG3d, TKGeomBase, TKBRep and
//! TKGeomAlgo natively; this crate depends on their ports. Each of the 16 OCCT
//! packages is one module named after it in snake case (`BRepClass3d` is
//! `brep_class3d`, `IntCurvesFace` is `int_curves_face`, `MAT2d` is `mat2d`). Every
//! module is declared here up front so that packages ported in parallel never edit
//! this file; a module whose package is not ported yet holds only its documentation.
//! Generic C++ packages (`.gxx`/`.pxx`/`.lxx` templates) are ported once as Rust
//! generics, as docs/topalgo-design.md fixes, and docs/tktopalgo-progress.md lists
//! the ported packages.
pub mod bisector;
pub mod brep_approx;
pub mod brep_bnd_lib;
pub mod brep_builder_api;
pub mod brep_check;
pub mod brep_class;
pub mod brep_class3d;
pub mod brep_extrema;
pub mod brep_gprop;
pub mod brep_int_curve_surface;
pub mod brep_lib;
pub mod brep_mat2d;
pub mod brep_top_adaptor;
pub mod int_curves_face;
pub mod mat;
pub mod mat2d;
