// Rust translation of ModelingAlgorithms/TKGeomAlgo at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT TKGeomAlgo: the geometric algorithms of ModelingAlgorithms (curve and
//! surface intersections, hatching, 2d constraint solving, filling and plates).
//!
//! TKGeomAlgo depends on TKernel, TKMath, TKG3d, TKG2d, TKGeomBase and TKBRep
//! natively; this crate depends on their ports. Each of the 33 OCCT packages is one
//! module named after it in snake case (`IntCurveSurface` is `int_curve_surface`,
//! `TopTrans` is `toptrans` as TKBRep's `topexp`). Every module is declared here up
//! front so that packages ported in parallel never edit this file; a module whose
//! package is not ported yet holds only its documentation. Generic C++ packages
//! (`.gxx`/`.pxx` templates) are ported once as Rust generics, as
//! docs/geomalgo-design.md fixes, and docs/tkgeomalgo-progress.md lists the ported
//! packages.
pub mod app_blend;
pub mod approx_int;
pub mod fair_curve;
pub mod gcc_ana;
pub mod gcc_ent;
pub mod gcc_int;
pub mod geom2d_api;
pub mod geom2d_gcc;
pub mod geom2d_hatch;
pub mod geom2d_int;
pub mod geom_api;
pub mod geom_fill;
pub mod geom_int;
pub mod geom_plate;
pub mod hatch;
pub mod hatch_gen;
pub mod int_curve;
pub mod int_curve_surface;
pub mod int_imp;
pub mod int_imp_par_gen;
pub mod int_patch;
pub mod int_polyh;
pub mod int_res2d;
pub mod int_start;
pub mod int_surf;
pub mod int_walk;
pub mod intf;
pub mod law;
pub mod local_analysis;
pub mod nl_plate;
pub mod plate;
pub mod topclass;
pub mod toptrans;
