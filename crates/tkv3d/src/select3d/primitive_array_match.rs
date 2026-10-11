//! The detection members of `Select3D_SensitivePrimitiveArray`: `Matches`, the element boxes
//! and the overlap and inclusion tests of the points, triangles, nodes and edges.
use super::primitive_array::{
    PrimArrayState, Select3DSensitivePrimitiveArray, point_index, position_point, tri_indices,
};
use super::{Select3DSensitiveEntity, Select3DTypeOfSensitivity};
use crate::select_basics::{SelectBasicsPickResult, SelectBasicsSelectingVolumeManager};
use crate::select_mgr::SelectMgrSelectionType;
use rustycad_tkernel::ncollection::packed_map_algo;
use rustycad_tkernel::standard::real::real_last;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::Point3;
use rustycad_tkservice::graphic3d::Graphic3dBufferMembers as _;

impl Select3DSensitivePrimitiveArray {
    /// The sub-array at a BVH index.
    pub(super) fn group(&self, idx: i32) -> Option<Handle<Self>> {
        let index = self.bvh_indices.index(idx);
        self.state
            .read()
            .groups
            .as_ref()?
            .get(index as usize)
            .cloned()
    }
    /// The detection state reset at the start of `Matches`.
    pub(super) fn reset_detection(&self) {
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
    pub(super) fn take_group_detection(&self) {
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
    pub(super) fn matches_groups(
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
    pub(super) fn overlaps_points(
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
    pub(super) fn overlaps_triangles(
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
    pub(super) fn points_inside(
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
    pub(super) fn triangles_inside(
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
}
