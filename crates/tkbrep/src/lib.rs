// Rust translation of ModelingData/TKBRep at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT TKBRep: the boundary representation data structure.
//!
//! TKBRep depends on TKernel, TKMath, TKG2d, TKG3d and TKGeomBase natively; this
//! crate depends on their ports. Each OCCT package is one module, added when it is
//! ported: `topods` (the topological data structure), `topexp` (its exploration),
//! `toptools` (maps, hashers and the shape set), `brep` (the geometric shapes,
//! their builder and their queries), `brep_tools`, `brep_adaptor`, `brep_lprop` and
//! `bintools`. docs/topology-design.md fixes the Rust mapping
//! and docs/tkbrep-progress.md lists the ported packages.
pub mod bintools;
pub mod brep;
pub mod brep_adaptor;
pub mod brep_graph;
pub mod brep_graph_inc;
pub mod brep_lprop;
pub mod brep_tools;
pub mod topexp;
pub mod topods;
pub mod toptools;
