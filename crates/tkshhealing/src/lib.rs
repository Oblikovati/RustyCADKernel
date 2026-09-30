// Rust translation of ModelingAlgorithms/TKShHealing at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT TKShHealing: the Shape Healing algorithms of ModelingAlgorithms (analysis of
//! shapes, fixing of their problems, conversion and division of their geometry, and the
//! operator sequences that data exchange runs on imported shapes).
//!
//! TKShHealing depends on TKBRep, TKernel, TKMath, TKG2d, TKTopAlgo, TKG3d, TKGeomBase
//! and TKGeomAlgo natively; this crate depends on their ports. Each of the 11 OCCT
//! packages is one module named after it in snake case (`ShapeExtend` is
//! `shape_extend`, `SHMessage` is `sh_message`, `ShapeProcessAPI` is
//! `shape_process_api`). Every module is declared here up front so that packages ported
//! in parallel never edit this file; a module whose package is not ported yet holds only
//! its documentation. docs/shhealing-design.md fixes how C++ constructs map, and
//! docs/tkshhealing-progress.md lists the ported packages.
pub mod sh_message;
pub mod shape_algo;
pub mod shape_analysis;
pub mod shape_build;
pub mod shape_construct;
pub mod shape_custom;
pub mod shape_extend;
pub mod shape_fix;
pub mod shape_process;
pub mod shape_process_api;
pub mod shape_upgrade;
