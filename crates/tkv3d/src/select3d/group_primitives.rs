//! Primitive hooks expose the group map through its independently reordered BVH indices.
use super::{
    Select3DBndBox3d, Select3DSensitiveGroup, Select3DSensitiveSet, Select3DSensitiveSetBase,
};
use crate::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use rustycad_tkernel::standard::Failure;
impl Select3DSensitiveSet for Select3DSensitiveGroup {
    fn set_base(&self) -> &Select3DSensitiveSetBase {
        // Traversal retains the native detected primitive index independently of map keys.
        &self.set
    }
    fn size(&self) -> i32 {
        // Failed list addition may retain a member before appending its primitive index.
        self.state.read().bvh_prim_indexes.len() as i32
    }
    fn element_box(&self, idx: i32) -> Select3DBndBox3d {
        self.primitive_member(idx)
            .get()
            .expect("Select3D_SensitiveGroup::Box null member; expected a sensitive entity")
            .bounding_box()
    }
    fn center(&self, idx: i32, axis: i32) -> Result<f64, Failure> {
        let center = self
            .primitive_member(idx)
            .get()
            .expect("Select3D_SensitiveGroup::Center null member; expected a sensitive entity")
            .center_of_geometry();
        Ok(match axis {
            0 => center.x(),
            1 => center.y(),
            _ => center.z(),
        })
    }
    fn swap(&self, idx1: i32, idx2: i32) {
        self.state
            .write()
            .bvh_prim_indexes
            .swap(idx1 as usize, idx2 as usize);
    }
    fn overlaps_element(
        &self,
        pick_result: &mut SelectBasicsPickResult,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        elem_idx: i32,
        _is_full_inside: bool,
    ) -> Result<bool, Failure> {
        super::select3d_group_overlaps_element(
            self,
            pick_result,
            manager,
            elem_idx,
            _is_full_inside,
        )
    }

    fn element_is_inside(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        elem_idx: i32,
        is_full_inside: bool,
    ) -> Result<bool, Failure> {
        super::select3d_group_element_is_inside(self, manager, elem_idx, is_full_inside)
    }

    fn distance_to_cog(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
    ) -> Result<f64, Failure> {
        super::select3d_group_distance_to_cog(self, manager)
    }
}
