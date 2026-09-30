// Rust translation of ModelingData/TKBRep/TopoDS at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! TopoDS: the topological data structure.
//!
//! A [`Shape`] is a value: a handle on a shared [`TShape`], a location and an
//! orientation. The typed shapes ([`Vertex`] ... [`Compound`]) are newtypes over it and
//! the `TopoDS` namespace casts are the functions [`vertex`] ... [`compound`].
//! [`Builder`] makes the shapes without geometry and links sub-shapes, and
//! [`ShapeIterator`] (`TopoDS_Iterator`) walks the direct sub-shapes. The mapping rules
//! are in docs/topology-design.md.
mod alerts;
mod builder;
pub mod errors;
mod hshape;
mod iterator;
mod shape;
mod shape_flags;
mod shape_identity;
mod tshape;
mod tshapes;
mod typed;

pub use alerts::{AlertAttribute, AlertWithShape};
pub use builder::Builder;
pub use hshape::HShape;
pub use iterator::ShapeIterator;
pub use shape::Shape;
pub(crate) use tshape::dump_tshape;
pub use tshape::{TShape, TShapeBase, bit_layout};
pub use tshapes::{TCompSolid, TCompound, TEdge, TFace, TShell, TSolid, TVertex, TWire};
pub use typed::{
    CompSolid, Compound, Edge, Face, Shell, Solid, Vertex, Wire, comp_solid, compound, edge, face,
    shell, solid, vertex, wire,
};
