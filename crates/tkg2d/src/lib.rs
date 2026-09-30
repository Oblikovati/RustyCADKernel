// Rust translation of ModelingData/TKG2d (Geom2d, Adaptor2d, Geom2dAdaptor, Geom2dEval, Geom2dGridEval and Geom2dHash packages) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT TKG2d: transient 2D geometry and the 2D curve adaptors.
//!
//! The Geom2d class hierarchy maps to Rust as described in docs/geom-design.md:
//! abstract classes are traits reached through `Handle<dyn Trait>`, and each
//! object keeps its state behind one lock so that mutation through any handle is
//! shared, as in OCCT. The Adaptor2d and Geom2dAdaptor adaptors follow the same
//! rules. The toolkit API and legacy-test acceptance remain in progress.
pub mod adaptor2d;
pub mod geom2d;
pub mod geom2d_adaptor;
pub mod geom2d_eval;
pub mod geom2d_grid_eval;
pub mod geom2d_hash;
