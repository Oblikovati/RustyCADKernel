// Rust translation of ChFi3d_FilBuilder::PerformTwoCorner (ChFi3d_FilBuilder_C2.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The corner of two fillet contours at a vertex.
use super::ChFi3dFilBuilder;
use rustycad_tkernel::standard::Failure;

impl ChFi3dFilBuilder {
    /// `PerformTwoCorner(Index)`.
    pub(super) fn perform_two_corner_impl(&mut self, _index: i32) -> Result<(), Failure> {
        Err(Failure::new("PerformTwoCorner: pending"))
    }
}
