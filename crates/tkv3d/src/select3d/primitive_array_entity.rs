//! The detection members of `Select3D_SensitivePrimitiveArray`: `Matches`, the element boxes
//! and the overlap and inclusion tests of the points, triangles, nodes and edges.
use super::primitive_array::Select3DSensitivePrimitiveArray;
use super::{
    Select3DBndBox3d, Select3DPrimArrayInit, Select3DSensitiveEntity, Select3DSensitiveEntityBase,
    Select3DSensitiveEntityMembers as _, Select3DSensitiveSet, Select3DSensitiveSetBase,
    select3d_set_build_bvh, select3d_set_dump_json, select3d_set_matches,
};
use crate::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use crate::select_mgr::{SelectMgrEntityOwnerSource, SelectMgrSelectionType};
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{GeneralTransform3, Point3};
use rustycad_tkservice::graphic3d::{Graphic3dBufferMembers as _, Graphic3dTypeOfPrimitiveArray};
use rustycad_tkservice::json_dump::{
    base_class, class_begin, field_dumped, field_integer, field_real,
};
use std::io;

impl Select3DSensitiveEntity for Select3DSensitivePrimitiveArray {
    fn sensitive_base(&self) -> &Select3DSensitiveEntityBase {
        &self.entity
    }
    /// `Matches`: the BVH traversal (checking all elements for a range detection), the
    /// detection of the last detected sub-array copied; a range detection with sub-arrays
    /// tests every sub-array.
    fn matches(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.reset_detection();
        let state = self.read();
        let to_detect_range =
            !state.detected_elem_map.is_null() || !state.detected_node_map.is_null();
        if state.groups.is_none()
            || manager.get_active_selection_type() == SelectMgrSelectionType::Point
            || !to_detect_range
        {
            if !select3d_set_matches(self, manager, pick_result, to_detect_range)? {
                return Ok(false);
            }
            self.take_group_detection();
            return Ok(true);
        }
        self.matches_groups(manager, pick_result)
    }
    fn nb_sub_elements(&self) -> i32 {
        self.state
            .read()
            .groups
            .as_ref()
            .map_or_else(|| self.bvh_indices.nb_elements(), |g| g.len() as i32)
    }
    fn bounding_box(&self) -> Select3DBndBox3d {
        if !self.state.read().bnd_box.is_valid() {
            self.compute_bounding_box();
        }
        self.apply_transformation()
    }
    fn center_of_geometry(&self) -> Point3 {
        self.state.read().cdg3d
    }
    /// `Set`: the owner of the array and of its sub-arrays.
    fn set(&self, owner: &Handle<dyn SelectMgrEntityOwnerSource>) {
        self.entity.fields.write().owner_id = owner.clone();
        for group in self
            .read()
            .groups
            .unwrap_or_default()
            .iter()
            .filter_map(Handle::get)
        {
            group.set(owner);
        }
    }
    /// `GetConnected`: a new array of the same buffers, range and groups.
    fn get_connected(&self) -> Result<Handle<dyn Select3DSensitiveEntity>, Failure> {
        let entity = Self::new(&self.owner_id())?;
        let state = self.read();
        let init = Select3DPrimArrayInit {
            verts: state.verts.clone(),
            indices: state.indices.clone(),
            init_loc: state.init_location.clone(),
            index_lower: state.index_lower,
            index_upper: state.index_upper,
            to_eval_min_max: true,
            nb_groups: state.groups.as_ref().map_or(1, |g| g.len() as i32),
        };
        match state.prim_type {
            Graphic3dTypeOfPrimitiveArray::Points => {
                entity.init_points(&init)?;
            }
            Graphic3dTypeOfPrimitiveArray::Triangles => {
                entity.init_triangulation(&init)?;
            }
            _ => {}
        }
        Ok(Handle::new(entity).cast())
    }
    /// `BVH`: the tree of this array (when dirty), then the dirty trees of the sub-arrays.
    fn bvh(&self) -> Result<(), Failure> {
        if !self.set.is_dirty() {
            return Ok(());
        }
        select3d_set_build_bvh(self)?;
        for group in self
            .read()
            .groups
            .unwrap_or_default()
            .iter()
            .filter_map(Handle::get)
        {
            if group.set.is_dirty() {
                group.bvh()?;
            }
        }
        Ok(())
    }
    fn to_build_bvh(&self) -> bool {
        self.set.is_dirty()
    }
    fn clear(&self) {}
    fn has_init_location(&self) -> bool {
        !self.state.read().init_location.is_identity()
    }
    fn inv_init_location(&self) -> GeneralTransform3 {
        self.state.read().inv_init_location
    }
    fn dump_json(&self, stream: &mut DumpStream, depth: i32) -> io::Result<()> {
        class_begin(stream, "Select3D_SensitivePrimitiveArray")?;
        base_class(stream, depth, &|s, d| select3d_set_dump_json(self, s, d))?;
        let s = self.read();
        field_integer(stream, "myPosStride", s.pos_stride as i32)?;
        field_integer(stream, "myPrimType", s.prim_type as i32)?;
        field_integer(stream, "myIndexLower", s.index_lower)?;
        field_integer(stream, "myIndexUpper", s.index_upper)?;
        field_integer(stream, "myPatchSizeMax", s.patch_size_max)?;
        field_real(stream, "myPatchDistance", f64::from(s.patch_distance))?;
        field_integer(stream, "myIs3d", i32::from(s.is_3d))?;
        let location = s.init_location.clone();
        field_dumped(stream, depth, "&myInitLocation", &|o, d| {
            location.dump_json(o, d)
        })?;
        let bounds = s.bnd_box;
        field_dumped(stream, depth, "&myBndBox", &|o, _| bounds.dump_json(o))?;
        for (name, value) in [
            ("myMinDepthElem", s.min_depth_elem),
            ("myMinDepthNode", s.min_depth_node),
            ("myMinDepthEdge", s.min_depth_edge),
        ] {
            field_real(stream, name, value)?;
        }
        for (name, value) in [
            ("myDetectedElem", s.detected_elem),
            ("myDetectedNode", s.detected_node),
            ("myDetectedEdgeNode1", s.detected_edge_node1),
            ("myDetectedEdgeNode2", s.detected_edge_node2),
        ] {
            field_integer(stream, name, value)?;
        }
        for (name, value) in [
            ("myToDetectElem", s.to_detect_elem),
            ("myToDetectNode", s.to_detect_node),
            ("myToDetectEdge", s.to_detect_edge),
        ] {
            field_integer(stream, name, i32::from(value))?;
        }
        Ok(())
    }
}

impl Select3DSensitiveSet for Select3DSensitivePrimitiveArray {
    fn set_base(&self) -> &Select3DSensitiveSetBase {
        &self.set
    }
    fn size(&self) -> i32 {
        self.bvh_indices.nb_elements()
    }
    fn element_box(&self, idx: i32) -> Select3DBndBox3d {
        let has_groups = self.state.read().groups.is_some();
        if let Some(group) = has_groups.then(|| self.group(idx)) {
            return group
                .and_then(|g| g.get().map(|v| v.bounding_box()))
                .unwrap_or_default();
        }
        let state = self.read();
        Self::patch_box(
            &state,
            self.bvh_indices.index(idx),
            self.bvh_indices.patch_size(idx),
        )
    }
    fn center(&self, idx: i32, axis: i32) -> Result<f64, Failure> {
        let center = if self.state.read().groups.is_some() {
            let c = self
                .group(idx)
                .and_then(|g| g.get().map(|v| v.center_of_geometry()))
                .unwrap_or_default();
            Vec3::new(c.x(), c.y(), c.z())
        } else {
            let bounds = self.element_box(idx);
            (*bounds.corner_min() + *bounds.corner_max()) * 0.5
        };
        Ok(match axis {
            0 => center.x,
            1 => center.y,
            _ => center.z,
        })
    }
    fn swap(&self, idx1: i32, idx2: i32) {
        let (e1, e2) = (self.bvh_indices.index(idx1), self.bvh_indices.index(idx2));
        if self.bvh_indices.has_patches() {
            let (p1, p2) = (
                self.bvh_indices.patch_size(idx1),
                self.bvh_indices.patch_size(idx2),
            );
            self.bvh_indices.set_index_patch(idx1, e2, p2);
            self.bvh_indices.set_index_patch(idx2, e1, p1);
        } else {
            self.bvh_indices.set_index(idx1, e2);
            self.bvh_indices.set_index(idx2, e1);
        }
    }
    fn overlaps_element(
        &self,
        pick_result: &mut SelectBasicsPickResult,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        elem_idx: i32,
        is_full_inside: bool,
    ) -> Result<bool, Failure> {
        if self.state.read().groups.is_some() {
            return self
                .group(elem_idx)
                .and_then(|g| g.get().map(|v| v.matches(manager, pick_result)))
                .unwrap_or(Ok(false));
        }
        let (state, index, patch) = (
            self.read(),
            self.bvh_indices.index(elem_idx),
            self.bvh_indices.patch_size(elem_idx),
        );
        match state.prim_type {
            Graphic3dTypeOfPrimitiveArray::Points => {
                self.overlaps_points(&state, pick_result, manager, index, patch, is_full_inside)
            }
            Graphic3dTypeOfPrimitiveArray::Triangles => {
                self.overlaps_triangles(&state, pick_result, manager, index, patch, is_full_inside)
            }
            _ => Ok(false),
        }
    }
    fn element_is_inside(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        elem_idx: i32,
        is_full_inside: bool,
    ) -> Result<bool, Failure> {
        if self.state.read().groups.is_some() {
            let mut dummy = SelectBasicsPickResult::default();
            return self
                .group(elem_idx)
                .and_then(|g| g.get().map(|v| v.matches(manager, &mut dummy)))
                .unwrap_or(Ok(false));
        }
        let (state, index, patch) = (
            self.read(),
            self.bvh_indices.index(elem_idx),
            self.bvh_indices.patch_size(elem_idx),
        );
        match state.prim_type {
            Graphic3dTypeOfPrimitiveArray::Points => {
                self.points_inside(&state, manager, index, patch, is_full_inside)
            }
            Graphic3dTypeOfPrimitiveArray::Triangles => {
                self.triangles_inside(&state, manager, index, patch, is_full_inside)
            }
            _ => Ok(false),
        }
    }
    fn distance_to_cog(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
    ) -> Result<f64, Failure> {
        manager.dist_to_geometry_center(&self.state.read().cdg3d)
    }
}
