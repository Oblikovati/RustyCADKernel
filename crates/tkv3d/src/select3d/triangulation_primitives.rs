//! Triangulation primitive tests preserve the upstream boundary box index quirk.
use super::sensitive_triangle::points_box;
use super::sensitive_triangulation::Select3DSensitiveTriangulation;
use super::triangulation_failure::native_checked;
use super::{
    Select3DBndBox3d, Select3DSensitiveSet, Select3DSensitiveSetBase, Select3DTypeOfSensitivity,
};
use crate::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use crate::select_mgr::SelectMgrSelectionType;
use rustycad_tkernel::ncollection::{Array1, Vec3};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bvh::{BvhBox, std_max, std_min};
impl Select3DSensitiveSet for Select3DSensitiveTriangulation {
    fn set_base(&self) -> &Select3DSensitiveSetBase {
        // Dirty state belongs to this entity independently of connected copies.
        &self.set
    }
    fn size(&self) -> i32 {
        // Free-edge alias edits do not recompute the captured primitive count.
        self.state.read().primitives_nb
    }
    /// `Box`: the triangle box, or for a free edge the box of the nodes at `Lower + prim` and
    /// `Lower + prim + 1` (natively not `2 * prim`: the boundary boxes are those of other edges).
    fn element_box(&self, idx: i32) -> Select3DBndBox3d {
        // Nonfallible BVH hooks must throw the checked native Failure instead of inventing a box.
        native_checked(self.checked_element_box(idx))
    }
    fn center(&self, idx: i32, axis: i32) -> Result<f64, Failure> {
        let bounds = self.checked_element_box(idx)?;
        let center = (*bounds.corner_min() + *bounds.corner_max()) * 0.5;
        Ok(match axis {
            0 => center.x,
            1 => center.y,
            _ => center.z,
        })
    }
    fn swap(&self, idx1: i32, idx2: i32) {
        native_checked(self.primitive(idx1));
        native_checked(self.primitive(idx2));
        if let Some(indexes) = self.state.write().bvh_prim_indexes.as_mut() {
            indexes.swap(idx1 as usize, idx2 as usize);
        }
    }
    /// `overlapsElement`: the free edge as a boundary polygon, or the interior triangle.
    fn overlaps_element(
        &self,
        pick_result: &mut SelectBasicsPickResult,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        elem_idx: i32,
        is_full_inside: bool,
    ) -> Result<bool, Failure> {
        if is_full_inside {
            return Ok(true);
        }
        self.overlap_triangulation_primitive(pick_result, manager, self.primitive(elem_idx)?)
    }
    /// `elementIsInside`: the segment or triangle (polyline) or all its points.
    fn element_is_inside(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        elem_idx: i32,
        is_full_inside: bool,
    ) -> Result<bool, Failure> {
        if is_full_inside {
            return Ok(true);
        }
        self.include_triangulation_primitive(manager, self.primitive(elem_idx)?)
    }
    fn distance_to_cog(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
    ) -> Result<f64, Failure> {
        manager.dist_to_geometry_center(&self.cdg3d)
    }
}

impl Select3DSensitiveTriangulation {
    fn checked_element_box(&self, idx: i32) -> Result<Select3DBndBox3d, Failure> {
        let prim = self.primitive(idx)?;
        if self.sens_type == Select3DTypeOfSensitivity::Interior {
            return Ok(points_box(&self.prim_triangle(prim)?));
        }
        let (a, b) = self.boundary_box_nodes(prim)?;
        Ok(boundary_segment_box(a, b))
    }

    fn overlap_triangulation_primitive(
        &self,
        pick: &mut SelectBasicsPickResult,
        volume: &dyn SelectBasicsSelectingVolumeManager,
        primitive: i32,
    ) -> Result<bool, Failure> {
        if self.sens_type == Select3DTypeOfSensitivity::Boundary {
            return self.overlap_boundary_primitive(pick, volume, primitive);
        }
        let [first, middle, last] = self.prim_triangle(primitive)?;
        volume.overlaps_triangle(
            &first,
            &middle,
            &last,
            Select3DTypeOfSensitivity::Interior as i32,
            pick,
        )
    }
    fn include_triangulation_primitive(
        &self,
        volume: &dyn SelectBasicsSelectingVolumeManager,
        primitive: i32,
    ) -> Result<bool, Failure> {
        let polyline = volume.get_active_selection_type() == SelectMgrSelectionType::Polyline;
        if self.sens_type == Select3DTypeOfSensitivity::Boundary {
            return self.include_boundary_primitive(volume, primitive, polyline);
        }
        self.include_interior_primitive(volume, primitive, polyline)
    }
    fn overlap_boundary_primitive(
        &self,
        pick: &mut SelectBasicsPickResult,
        volume: &dyn SelectBasicsSelectingVolumeManager,
        primitive: i32,
    ) -> Result<bool, Failure> {
        let (first, last) = self.edge_nodes(primitive)?;
        let points = [first, last];
        volume.overlaps_polygon(
            &Array1::from_shared(&points, 1),
            Select3DTypeOfSensitivity::Boundary as i32,
            pick,
        )
    }
    fn include_interior_primitive(
        &self,
        volume: &dyn SelectBasicsSelectingVolumeManager,
        primitive: i32,
        polyline: bool,
    ) -> Result<bool, Failure> {
        let [a, b, c] = self.prim_triangle(primitive)?;
        let mut dummy = SelectBasicsPickResult::default();
        if polyline {
            return volume.overlaps_triangle(&a, &b, &c, self.sens_type as i32, &mut dummy);
        }
        Ok(volume.overlaps_point_only(&a)?
            && volume.overlaps_point_only(&b)?
            && volume.overlaps_point_only(&c)?)
    }
    fn include_boundary_primitive(
        &self,
        volume: &dyn SelectBasicsSelectingVolumeManager,
        primitive: i32,
        polyline: bool,
    ) -> Result<bool, Failure> {
        let (a, b) = self.edge_nodes(primitive)?;
        if polyline {
            return volume.overlaps_segment(&a, &b, &mut SelectBasicsPickResult::default());
        }
        Ok(volume.overlaps_point_only(&a)? && volume.overlaps_point_only(&b)?)
    }
}

fn boundary_segment_box(
    a: rustycad_tkmath::gp::Point3,
    b: rustycad_tkmath::gp::Point3,
) -> Select3DBndBox3d {
    let min = Vec3::new(
        std_min(a.x(), b.x()),
        std_min(a.y(), b.y()),
        std_min(a.z(), b.z()),
    );
    let max = Vec3::new(
        std_max(a.x(), b.x()),
        std_max(a.y(), b.y()),
        std_max(a.z(), b.z()),
    );
    BvhBox::from_corners(min, max)
}
