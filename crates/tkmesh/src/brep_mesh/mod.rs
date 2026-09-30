//! Concrete BRepMesh algorithms and topology support.
mod shape_tool;
pub use shape_tool::ShapeTool;

mod model_builder;
mod shape_visitor;
pub use model_builder::BRepMeshModelBuilder;
pub use shape_visitor::BRepMeshShapeVisitor;

mod deflection;
pub use deflection::Deflection;

mod post_processor;
pub use post_processor::BRepMeshModelPostProcessor;

mod edge_parameter;
pub use edge_parameter::{EdgeParameterCollection, EdgeParameterProvider};
