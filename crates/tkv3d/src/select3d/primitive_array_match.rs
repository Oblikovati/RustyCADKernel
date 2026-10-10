//! The detection members of `Select3D_SensitivePrimitiveArray`: `Matches`, the element boxes
//! and the overlap and inclusion tests of the points, triangles, nodes and edges.
use super::primitive_array::{
    PrimArrayState, Select3DSensitivePrimitiveArray, point_index, position_point, read_position,
    read_position_2d, tri_indices,
};
use super::{
    Select3DBndBox3d, Select3DPrimArrayInit, Select3DSensitiveEntity, Select3DSensitiveEntityBase,
    Select3DSensitiveEntityMembers as _, Select3DSensitiveSet, Select3DSensitiveSetBase,
    Select3DTypeOfSensitivity, select3d_set_build_bvh, select3d_set_dump_json,
    select3d_set_matches,
};
use crate::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use crate::select_mgr::{SelectMgrEntityOwnerSource, SelectMgrSelectionType};
use rustycad_tkernel::ncollection::{Vec3, packed_map_algo};
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{GeneralTransform3, Point3};
use rustycad_tkservice::graphic3d::{Graphic3dBufferMembers as _, Graphic3dTypeOfPrimitiveArray};
use rustycad_tkservice::json_dump::{
    base_class, class_begin, field_dumped, field_integer, field_real,
};
use std::io;

impl Select3DSensitivePrimitiveArray {
    /// The sub-array at a BVH index.
    fn group(&self, idx: i32) -> Option<Handle<Self>> {
        let index = self.bvh_indices.index(idx);
        self.state
            .read()
            .groups
            .as_ref()?
            .get(index as usize)
            .cloned()
    }
    /// The detection state reset at the start of `Matches`.
    fn reset_detection(&self) {
        let mut state = self.state.write();
        for map in [&state.detected_elem_map, &state.detected_node_map]
            .into_iter()
            .filter_map(Handle::get)
        {
            map.change_map().clear();
        }
        (
            state.min_depth_elem,
            state.min_depth_node,
            state.min_depth_edge,
        ) = (real_last(), real_last(), real_last());
        (
            state.detected_elem,
            state.detected_node,
            state.detected_edge_node1,
            state.detected_edge_node2,
        ) = (-1, -1, -1, -1);
    }
    /// Copies the detection of the last detected sub-array.
    fn take_group_detection(&self) {
        let detected = self.set.detected_idx();
        let Some(group) = self.group(detected).filter(|_| detected != -1) else {
            return;
        };
        let from = group.read();
        let mut state = self.state.write();
        (
            state.min_depth_elem,
            state.min_depth_node,
            state.min_depth_edge,
        ) = (
            from.min_depth_elem,
            from.min_depth_node,
            from.min_depth_edge,
        );
        (state.detected_elem, state.detected_node) = (from.detected_elem, from.detected_node);
        (state.detected_edge_node1, state.detected_edge_node2) =
            (from.detected_edge_node1, from.detected_edge_node2);
    }
    /// The range detection of a box or polyline over every sub-array, uniting their maps.
    fn matches_groups(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        pick_result: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        let mut pick = SelectBasicsPickResult::default();
        let mut has_results = false;
        for iter in 0..self.bvh_indices.nb_elements() {
            let Some(child) = self.group(iter) else {
                continue;
            };
            if !child.matches(manager, &mut pick)? {
                continue;
            }
            has_results = true;
            self.unite_maps(&child);
            if pick_result.depth() > pick.depth() {
                self.set.set_detected_idx(iter);
                *pick_result = pick;
            }
        }
        if !has_results {
            return Ok(false);
        }
        pick_result
            .set_dist_to_geom_center(manager.dist_to_geometry_center(&self.center_of_geometry())?);
        Ok(true)
    }
    /// Unites the detection maps of a sub-array into this array's.
    fn unite_maps(&self, child: &Self) {
        let (own, from) = (self.read(), child.read());
        for (target, source) in [
            (&own.detected_elem_map, &from.detected_elem_map),
            (&own.detected_node_map, &from.detected_node_map),
        ] {
            if let (Some(target), Some(source)) = (target.get(), source.get()) {
                packed_map_algo::unite(&mut target.change_map(), &source.map());
            }
        }
    }
    /// Adds an element or node to a detection map of a range selection.
    fn add_detected(map: &Handle<rustycad_tkernel::tcol_std::HPackedMapOfInteger>, value: i32) {
        if let Some(map) = map.get() {
            map.change_map().add(value);
        }
    }
    /// `overlapsElement` for the points of a patch.
    fn overlaps_points(
        &self,
        state: &PrimArrayState,
        pick_result: &mut SelectBasicsPickResult,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        elem_idx: i32,
        patch_size: i32,
        is_full_inside: bool,
    ) -> Result<bool, Failure> {
        let (mut result, mut pick) = (false, SelectBasicsPickResult::default());
        let is_range = manager.get_active_selection_type() != SelectMgrSelectionType::Point;
        for iter in 0..patch_size {
            let index = point_index(&state.indices, elem_idx + iter);
            let point = position_point(state, index);
            let (detect_node, detect_elem) = {
                let s = self.state.read();
                (s.to_detect_node, s.to_detect_elem)
            };
            if (detect_node || detect_elem)
                && (is_full_inside || manager.overlaps_point(&point, &mut pick)?)
            {
                let mut own = self.state.write();
                if pick.depth() <= own.min_depth_node {
                    (own.detected_elem, own.detected_node) = (index, index);
                    (own.min_depth_elem, own.min_depth_node) = (pick.depth(), pick.depth());
                }
                if is_range {
                    Self::add_detected(&own.detected_elem_map, index);
                    Self::add_detected(&own.detected_node_map, index);
                }
                result = true;
            }
            *pick_result = *SelectBasicsPickResult::min(pick_result, &pick);
        }
        Ok(result)
    }
    /// `overlapsElement` for the triangles of a patch: the elements, nodes and edges asked for.
    fn overlaps_triangles(
        &self,
        state: &PrimArrayState,
        pick_result: &mut SelectBasicsPickResult,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        elem_idx: i32,
        patch_size: i32,
        is_full_inside: bool,
    ) -> Result<bool, Failure> {
        let (mut result, mut pick) = (false, SelectBasicsPickResult::default());
        for iter in 0..patch_size {
            let tri_index = elem_idx + iter;
            let nodes = tri_indices(&state.indices, tri_index * 3);
            let points = nodes.map(|n| position_point(state, n));
            result |=
                self.detect_element(manager, &mut pick, tri_index, &points, is_full_inside)?;
            result |= self.detect_nodes(manager, &mut pick, &nodes, &points, is_full_inside)?;
            result |= self.detect_edges(manager, &mut pick, &nodes, &points, is_full_inside)?;
            *pick_result = *SelectBasicsPickResult::min(pick_result, &pick);
        }
        Ok(result)
    }
    /// The element detection of a triangle.
    fn detect_element(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        pick: &mut SelectBasicsPickResult,
        tri_index: i32,
        p: &[Point3; 3],
        is_full_inside: bool,
    ) -> Result<bool, Failure> {
        if !self.state.read().to_detect_elem {
            return Ok(false);
        }
        if !is_full_inside
            && !manager.overlaps_triangle(
                &p[0],
                &p[1],
                &p[2],
                Select3DTypeOfSensitivity::Interior as i32,
                pick,
            )?
        {
            return Ok(false);
        }
        let mut own = self.state.write();
        if pick.depth() <= own.min_depth_elem {
            (own.detected_elem, own.min_depth_elem) = (tri_index, pick.depth());
        }
        if manager.get_active_selection_type() != SelectMgrSelectionType::Point {
            Self::add_detected(&own.detected_elem_map, tri_index);
        }
        Ok(true)
    }
    /// The node detection of a triangle.
    fn detect_nodes(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        pick: &mut SelectBasicsPickResult,
        nodes: &[i32; 3],
        p: &[Point3; 3],
        is_full_inside: bool,
    ) -> Result<bool, Failure> {
        if !self.state.read().to_detect_node {
            return Ok(false);
        }
        let mut result = false;
        for k in 0..3 {
            if !is_full_inside && !manager.overlaps_point(&p[k], pick)? {
                continue;
            }
            let mut own = self.state.write();
            if pick.depth() <= own.min_depth_node {
                (own.detected_node, own.min_depth_node) = (nodes[k], pick.depth());
            }
            if manager.get_active_selection_type() != SelectMgrSelectionType::Point {
                Self::add_detected(&own.detected_node_map, nodes[k]);
            }
            result = true;
        }
        Ok(result)
    }
    /// The edge detection of a triangle (the edges (2,0), (0,1), (1,2)).
    fn detect_edges(
        &self,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        pick: &mut SelectBasicsPickResult,
        nodes: &[i32; 3],
        p: &[Point3; 3],
        is_full_inside: bool,
    ) -> Result<bool, Failure> {
        if !self.state.read().to_detect_edge {
            return Ok(false);
        }
        let mut result = false;
        for k in 0..3 {
            let first = if k == 0 { 2 } else { k - 1 };
            if !is_full_inside && !manager.overlaps_segment(&p[first], &p[k], pick)? {
                continue;
            }
            let mut own = self.state.write();
            if pick.depth() <= own.min_depth_edge {
                (
                    own.detected_edge_node1,
                    own.detected_edge_node2,
                    own.min_depth_edge,
                ) = (nodes[first], nodes[k], pick.depth());
            }
            result = true;
        }
        Ok(result)
    }
    /// `elementIsInside` for the points of a patch.
    fn points_inside(
        &self,
        state: &PrimArrayState,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        elem_idx: i32,
        patch_size: i32,
        is_full_inside: bool,
    ) -> Result<bool, Failure> {
        for iter in 0..patch_size {
            let index = point_index(&state.indices, elem_idx + iter);
            if !is_full_inside && !manager.overlaps_point_only(&position_point(state, index))? {
                return Ok(false);
            }
            if manager.get_active_selection_type() != SelectMgrSelectionType::Point {
                let own = self.state.read();
                Self::add_detected(&own.detected_elem_map, index);
                Self::add_detected(&own.detected_node_map, index);
            }
        }
        Ok(true)
    }
    /// `elementIsInside` for the triangles of a patch.
    fn triangles_inside(
        &self,
        state: &PrimArrayState,
        manager: &dyn SelectBasicsSelectingVolumeManager,
        elem_idx: i32,
        patch_size: i32,
        is_full_inside: bool,
    ) -> Result<bool, Failure> {
        for iter in 0..patch_size {
            let tri_index = elem_idx + iter;
            let nodes = tri_indices(&state.indices, tri_index * 3);
            let p = nodes.map(|n| position_point(state, n));
            if !is_full_inside
                && (!manager.overlaps_point_only(&p[0])?
                    || !manager.overlaps_point_only(&p[1])?
                    || !manager.overlaps_point_only(&p[2])?)
            {
                return Ok(false);
            }
            if manager.get_active_selection_type() != SelectMgrSelectionType::Point {
                let own = self.state.read();
                Self::add_detected(&own.detected_elem_map, tri_index);
                nodes
                    .iter()
                    .for_each(|&n| Self::add_detected(&own.detected_node_map, n));
            }
        }
        Ok(true)
    }
    /// `applyTransformation` (protected): the box of the eight corners at the initial location.
    fn apply_transformation(&self) -> Select3DBndBox3d {
        let state = self.read();
        if state.init_location.is_identity() {
            return state.bnd_box;
        }
        let (min, max, trsf) = (
            *state.bnd_box.corner_min(),
            *state.bnd_box.corner_max(),
            state.init_location.transformation(),
        );
        let mut bounds = Select3DBndBox3d::default();
        for (x, y, z) in (0..8).map(|i| (i / 4, (i / 2) % 2, i % 2)) {
            let corner = Point3::new(
                if x == 0 { min.x } else { max.x },
                if y == 0 { min.y } else { max.y },
                if z == 0 { min.z } else { max.z },
            );
            let p = corner.transformed(trsf);
            bounds.add(&Vec3::new(p.x(), p.y(), p.z()));
        }
        bounds
    }
    /// The box of a patch of points or triangles.
    fn patch_box(state: &PrimArrayState, elem_idx: i32, patch_size: i32) -> Select3DBndBox3d {
        let mut bounds = Select3DBndBox3d::default();
        let as_f64 = |v: Vec3<f32>| Vec3::new(f64::from(v.x), f64::from(v.y), f64::from(v.z));
        for iter in 0..patch_size {
            match state.prim_type {
                Graphic3dTypeOfPrimitiveArray::Points => {
                    let index = point_index(&state.indices, elem_idx + iter);
                    let p = if state.is_3d {
                        read_position(state, index).unwrap_or_default()
                    } else {
                        let q = read_position_2d(state, index).unwrap_or_default();
                        Vec3::new(q.x, q.y, 0.0)
                    };
                    bounds.add(&as_f64(p));
                }
                Graphic3dTypeOfPrimitiveArray::Triangles => {
                    let nodes = tri_indices(&state.indices, (elem_idx + iter) * 3);
                    let (min, max) = triangle_min_max(state, &nodes);
                    bounds.add(&as_f64(min));
                    bounds.add(&as_f64(max));
                }
                _ => return bounds,
            }
        }
        bounds
    }
}

/// The single-precision component-wise corners of a triangle (z = 0 in 2D).
fn triangle_min_max(state: &PrimArrayState, nodes: &[i32; 3]) -> (Vec3<f32>, Vec3<f32>) {
    if state.is_3d {
        let [a, b, c] = nodes.map(|n| read_position(state, n).unwrap_or_default());
        return (a.cwise_min(&b).cwise_min(&c), a.cwise_max(&b).cwise_max(&c));
    }
    let [a, b, c] = nodes.map(|n| read_position_2d(state, n).unwrap_or_default());
    let (min, max) = (a.cwise_min(&b).cwise_min(&c), a.cwise_max(&b).cwise_max(&c));
    (Vec3::new(min.x, min.y, 0.0), Vec3::new(max.x, max.y, 0.0))
}

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
