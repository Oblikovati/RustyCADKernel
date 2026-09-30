// Rust translation of ModelingAlgorithms/TKHelix at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT TKHelix: the helix curves of ModelingAlgorithms (the analytic helix, its
//! B-spline approximation coil by coil, and composite helix wires of several parts).
//!
//! TKHelix depends on TKBRep, TKernel, TKMath, TKGeomBase, TKG2d, TKG3d, TKGeomAlgo
//! and TKTopAlgo natively (its EXTERNLIB); this crate depends on their ports. Each of
//! the 2 OCCT packages is one module named after it in snake case (`HelixGeom` is
//! `helix_geom`, `HelixBRep` is `helix_brep`), declared here up front; a module whose
//! package is not ported yet holds only its documentation. docs/helix-design.md fixes
//! how the C++ builder hierarchy maps to Rust, and docs/tkhelix-progress.md lists the
//! ported packages.
pub mod helix_brep;
pub mod helix_geom;
