//! The coherent (editable, connected) triangulation of OCCT Poly.
mod link;
mod node;
mod tri_ptr;
mod triangle;
mod triangle_connect;
mod triangulation;
pub use link::CoherentLink;
pub use node::CoherentNode;
pub use tri_ptr::CoherentTriPtr;
pub use triangle::CoherentTriangle;
pub use triangulation::{CoherentTriangulation, TwoIntegers};
