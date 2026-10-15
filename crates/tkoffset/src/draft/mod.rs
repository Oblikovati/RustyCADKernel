// Rust translation of the Draft package of ModelingAlgorithms/TKOffset at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Draft`: the draft angle modification (`Draft_Modification`) and its records
//! (`Draft_EdgeInfo`, `Draft_FaceInfo`, `Draft_VertexInfo`, `Draft_ErrorStatus`).
mod info;
mod modification;

pub use info::{DraftEdgeInfo, DraftErrorStatus, DraftFaceInfo, DraftVertexInfo};
pub use modification::DraftModification;
