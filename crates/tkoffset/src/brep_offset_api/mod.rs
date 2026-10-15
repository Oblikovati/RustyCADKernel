//! OCCT `BRepOffsetAPI`: the builder-API entry points of the offset, sweep and loft
//! algorithms.
mod make_offset;
mod make_offset_domains;
mod make_offset_shape;

pub use make_offset::BRepOffsetAPIMakeOffset;
pub use make_offset_shape::BRepOffsetAPIMakeOffsetShape;
