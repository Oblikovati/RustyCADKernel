//! Plane-pair preparation from BOPAlgo_PaveFiller_6.cxx at OCCT 3d097a0.
use super::BOPAlgoPaveFiller;
use crate::bop_ds::FaceInfo;
use rustycad_tkernel::standard::Failure;

impl BOPAlgoPaveFiller {
    /// Initializes face classifications and checks native shared-vertex multiplicity.
    /// `let intersect = filler.check_planes(first_face, second_face)?;`.
    pub fn check_planes(&mut self, first_face: i32, second_face: i32) -> Result<bool, Failure> {
        let structure = self.structure_mut()?;
        structure.change_face_info(first_face)?;
        structure.change_face_info(second_face)?;
        let first = structure.face_info(first_face)?;
        let second = structure.face_info(second_face)?;
        Ok(two_shared_occurrences(first, second))
    }
}

fn two_shared_occurrences(first: &FaceInfo, second: &FaceInfo) -> bool {
    // Native counts a vertex twice when it occurs in both second-face sets.
    second
        .vertices_in()
        .iter()
        .chain(second.vertices_on().iter())
        .copied()
        .filter(|index| {
            first.vertices_in().contains(*index) || first.vertices_on().contains(*index)
        })
        .take(2)
        .count()
        == 2
}
