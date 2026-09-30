// Rust translation of ModelingAlgorithms/TKHLR/TopCnx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT package TopCnx: the transition of an edge through a face along the boundary
//! curves it meets at one point (`EdgeFaceTransition`, over TopTrans_CurveTransition).
mod edge_face_transition;

pub use edge_face_transition::EdgeFaceTransition;
