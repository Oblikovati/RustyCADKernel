//! Boolean topology utilities from the pinned OCCT BOPTools package.
mod connexity;
mod connexity_block;
pub use connexity::{
    make_connexity_blocks, make_connexity_blocks_from_shapes, make_connexity_compounds,
};
pub use connexity_block::ConnexityBlock;
pub mod algo_tools2d;
pub mod algo_tools3d;

mod split_reverse;
pub use split_reverse::is_split_edge_to_reverse;
