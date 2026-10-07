//! The TNaming package: the topological naming of OCAF (ocaf-design.md section 10).
//!
//! A [`TNamingBuilder`] records shape evolutions in the [`TNamingNamedShape`] of a label;
//! the root [`TNamingUsedShapes`] maps every recorded shape to the evolutions that use it,
//! which the iterators and [`TNamingTool`] walk.
mod builder;
mod copy_shape;
mod deltas;
mod evolution;
mod iterator;
mod name;
mod named_shape;
mod named_shape_attribute;
mod naming;
mod naming_tool;
mod node;
mod node_chains;
mod same_shape_iterator;
mod scope;
mod shape_iterators;
mod shapes_set;
mod tool;
mod tool_find;
mod tool_shapes;
mod translate_tool;
mod translator;
mod used_shapes;

pub use builder::TNamingBuilder;
pub use copy_shape::{TNamingCopyShape, TNamingTransientMap};
pub use deltas::{TNamingDeltaOnModification, TNamingDeltaOnRemoval};
pub use evolution::{TNamingEvolution, TNamingNameType};
pub use iterator::TNamingIterator;
pub use name::TNamingName;
pub use named_shape::TNamingNamedShape;
pub use naming::TNamingNaming;
pub use naming_tool::TNamingNamingTool;
pub use node::{TNamingNode, TNamingNodePtr, TNamingRefShape, TNamingRefShapePtr};
pub use same_shape_iterator::TNamingSameShapeIterator;
pub use scope::TNamingScope;
pub use shape_iterators::{TNamingNewShapeIterator, TNamingOldShapeIterator};
pub use shapes_set::{TNamingIteratorOnShapesSet, TNamingShapesSet};
pub use tool::TNamingTool;
pub use tool_shapes::TNamingNamedShapeMap;
pub use translate_tool::TNamingTranslateTool;
pub use translator::{TNamingShapeShapeMap, TNamingTranslator};
pub use used_shapes::{TNamingShapeRefMap, TNamingUsedShapes};
