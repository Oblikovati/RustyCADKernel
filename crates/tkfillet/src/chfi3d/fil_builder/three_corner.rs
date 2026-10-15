// Rust translation of ChFi3d_FilBuilder::PerformThreeCorner (ChFi3d_FilBuilder_C3.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The corner of three fillet contours at a vertex.
use super::ChFi3dFilBuilder;
use rustycad_tkernel::standard::Failure;

impl ChFi3dFilBuilder {
    /// `PerformThreeCorner(Index)`.
    pub(super) fn perform_three_corner_impl(&mut self, _index: i32) -> Result<(), Failure> {
        Err(Failure::new("PerformThreeCorner: pending"))
    }
}
