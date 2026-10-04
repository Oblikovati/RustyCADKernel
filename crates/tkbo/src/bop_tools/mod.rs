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

mod face_orientation;
pub use face_orientation::{face_sense, is_split_face_to_reverse, is_split_shape_to_reverse};

mod orientation_warning;
pub use orientation_warning::is_split_to_reverse_with_warning;

mod shape_dimensions;
pub use shape_dimensions::{dimension, dimensions, treat_compound};
mod open_shell;
pub use open_shell::is_open_shell;
