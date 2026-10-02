//! BOPDS records used by Boolean intersection bookkeeping, at OCCT 3d097a0.
mod common_block;
mod couple_of_pave_blocks;
mod face_info;
mod index_range;
mod pair;
mod pave;
mod pave_block;
mod point;
mod shape_info;
pub use common_block::{CommonBlock, pave_block_handle};
pub use couple_of_pave_blocks::CoupleOfPaveBlocks;
pub use face_info::{FaceInfo, PaveBlockIndexMap, VertexIndexSet};
pub use index_range::IndexRange;
pub use pair::Pair;
pub use pave::Pave;
pub use pave_block::{PaveBlock, PaveBlockHandle};
pub use point::Point;
pub use shape_info::ShapeInfo;
