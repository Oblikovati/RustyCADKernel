//! Meshing parameters, strategy contracts, and topology visitation.
mod interfaces;
mod parameters;
mod shape_explorer;
pub use interfaces::{CurveTessellator, MeshAlgo, MeshAlgoFactory, ModelAlgo, ShapeVisitor};
pub use parameters::{MeshAlgoType, MeshParameters};
pub use shape_explorer::ShapeExplorer;
