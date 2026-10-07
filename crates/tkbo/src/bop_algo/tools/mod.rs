//! `BOPAlgo_Tools`: the static helpers of the intersection part of Boolean operations.
mod classify_faces;
mod common_blocks;
mod connection;
mod intersect_vertices;
mod trsf;

pub use classify_faces::{InParts, ShapeBoxMap, SolidInternalFaces, classify_faces};
pub use common_blocks::{
    PaveBlockIndexLinks, PaveBlockLinks, compute_tolerance_of_cb, pave_block_index_links,
    pave_block_links, perform_common_blocks, perform_common_blocks_on_faces,
};
pub use connection::{ConnectionMap, fill_map, fill_map_one, make_blocks};
pub use intersect_vertices::intersect_vertices;
pub use trsf::{TRSF_TO_POINT_CRITERIA, trsf_to_point};
