//! OCCT `BRepOffsetAPI`: the builder-API entry points of the offset, sweep and loft
//! algorithms.
mod make_offset;
mod make_offset_domains;
mod make_offset_shape;
mod make_pipe_shell;
mod thru_sections;

pub use make_offset::BRepOffsetAPIMakeOffset;
pub use make_offset_shape::BRepOffsetAPIMakeOffsetShape;
pub use make_pipe_shell::BRepOffsetAPIMakePipeShell;
pub use thru_sections::BRepOffsetAPIThruSections;
