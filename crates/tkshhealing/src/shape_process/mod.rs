// Rust translation of ModelingAlgorithms/TKShHealing/ShapeProcess at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package ShapeProcess: the operators and contexts that run sequences of Shape
//! Healing operations (docs/tkshhealing-progress.md).
mod context;
mod oper_library;
mod operator;
mod package;
mod shape_context;

pub use context::{ShapeProcessContext, ShapeProcessResource};
pub use oper_library::ShapeProcessOperLibrary;
pub use operator::{ShapeProcessOperFunc, ShapeProcessOperator, ShapeProcessUOperator};
pub use package::{ShapeProcess, ShapeProcessOperation, ShapeProcessOperationsFlags};
pub use shape_context::{ShapeProcessShapeContext, ShapeProcessShapeMap};
