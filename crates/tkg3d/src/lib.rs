// Rust translation of ModelingData/TKG3d/(Geom, AdvApprox, Adaptor3d, GeomAdaptor, GeomEval, GeomGridEval, GeomHash and TopAbs packages) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT TKG3d: transient 3D geometry.
//!
//! The Geom class hierarchy maps to Rust as described in docs/geom-design.md,
//! with the rules of the Geom2d port in `rustycad-tkg2d`: abstract classes are
//! traits reached through `Handle<dyn Trait>`, and each object keeps its state
//! behind one lock so that mutation through any handle is shared, as in OCCT.
//! TKG3d depends on TKG2d natively; this crate reuses the Geom2d array and dump
//! helpers, the Geom2dEval basis evaluators, the Geom2dHash helpers and the
//! Adaptor2d curves, and the Geom2dGridEval parameter loops. The AdvApprox,
//! Adaptor3d, GeomAdaptor, GeomEval, GeomGridEval, GeomHash and TopAbs packages are
//! ported in full.
pub mod adaptor3d;
pub mod adv_approx;
pub mod geom;
pub mod geom_adaptor;
pub mod geom_eval;
pub mod geom_grid_eval;
pub mod geom_hash;
pub mod top_abs;
