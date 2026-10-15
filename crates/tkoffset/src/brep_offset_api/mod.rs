//! OCCT `BRepOffsetAPI`: the builder-API entry points of the offset, sweep and loft
//! algorithms.
mod find_contigous_edges;
mod make_draft;
mod make_evolved;
mod make_filling;
mod make_offset;
mod make_offset_domains;
mod make_offset_shape;
mod make_pipe;
mod make_pipe_shell;
mod normal_projection;
mod thru_sections;

pub use find_contigous_edges::BRepOffsetAPIFindContigousEdges;
pub use make_draft::BRepOffsetAPIMakeDraft;
pub use make_evolved::{BRepOffsetAPIMakeEvolved, EvolvedOptions};
pub use make_filling::{BRepOffsetAPIMakeFilling, FillingParameters};
pub use make_offset::BRepOffsetAPIMakeOffset;
pub use make_offset_shape::BRepOffsetAPIMakeOffsetShape;
pub use make_pipe::BRepOffsetAPIMakePipe;
pub use make_pipe_shell::BRepOffsetAPIMakePipeShell;
pub use normal_projection::BRepOffsetAPINormalProjection;
pub use thru_sections::BRepOffsetAPIThruSections;
