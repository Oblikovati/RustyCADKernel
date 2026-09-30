//! Meshing parameters, strategy contracts, and topology visitation.
mod interfaces;
mod parameters;
mod shape_explorer;
pub use interfaces::{CurveTessellator, MeshAlgo, MeshAlgoFactory, ModelAlgo, ShapeVisitor};
pub use parameters::{MeshAlgoType, MeshParameters};
pub use shape_explorer::ShapeExplorer;

mod context;
mod mesh_builder;
mod model_builder;
pub use context::{Context, MeshContext};
pub use mesh_builder::MeshBuilder;
pub use model_builder::ModelBuilder;
