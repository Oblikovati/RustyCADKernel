//! `Select3D_SensitivePrimitiveArray::InitTriangulation`, `InitPoints` and the sub-array
//! groups (built in order; natively by `OSD_Parallel::For`, with the same result).
use super::primitive_array::{
    PrimArrayState, Select3DPrimArrayInit, Select3DSensitivePrimitiveArray, position_attribute,
    read_position, read_position_2d,
};
use super::{Select3DBndBox3d, Select3DSensitiveEntity, Select3DSensitiveEntityMembers as _};
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{GeneralTransform3, Point3};
use rustycad_tkservice::graphic3d::{Graphic3dBufferMembers as _, Graphic3dTypeOfPrimitiveArray};

/// The element count of an init (triangles or points), and whether it is split into groups.
pub(super) struct InitCounts {
    pub(super) first: i32,
    pub(super) count: i32,
    pub(super) has_groups: bool,
}

impl Select3DSensitivePrimitiveArray {
    /// The reset at the start of an init: dirty tree, no group, no buffers, the location.
    fn reset_for_init(
        &self,
        prim_type: Graphic3dTypeOfPrimitiveArray,
        init_loc: &rustycad_tkmath::toploc::Location,
    ) -> Result<(), Failure> {
        self.set.mark_dirty();
        self.bvh_indices.release()?;
        let mut state = self.state.write();
        state.groups = None;
        state.prim_type = prim_type;
        state.bnd_box.clear();
        (state.verts, state.indices) = (Handle::default(), Handle::default());
        reset_position_range(&mut state);
        state.is_3d = false;
        state.init_location = init_loc.clone();
        Ok(())
    }
    /// The checks of an init: vertices with a valid position attribute and an index range
    /// inside the index (or vertex) buffer; the position is kept as natively.
    fn check_init(&self, init: &Select3DPrimArrayInit) -> bool {
        let Some(verts) = init.verts.get().filter(|v| v.nb_elements() != 0) else {
            return false;
        };
        let Some((offset, stride, is_3d)) = position_attribute(verts) else {
            return false;
        };
        {
            let mut state = self.state.write();
            (state.pos_offset, state.pos_stride, state.is_3d) = (offset, stride, is_3d);
        }
        if offset.is_none() {
            return false;
        }
        init_range_is_valid(init, verts.nb_elements())
    }
    /// The BVH index buffer and the stored buffers of a checked init; false when the index
    /// buffer cannot be allocated.
    fn store_init(
        &self,
        init: &Select3DPrimArrayInit,
        counts: &InitCounts,
    ) -> Result<bool, Failure> {
        let patches = !counts.has_groups && self.patch_size_max() > 1;
        let capacity = if counts.has_groups {
            init.nb_groups
        } else {
            counts.count
        };
        if !self.bvh_indices.init(capacity, patches)? {
            return Ok(false);
        }
        self.store_initialized_buffers(init)?;
        Ok(true)
    }
    fn store_initialized_buffers(&self, init: &Select3DPrimArrayInit) -> Result<(), Failure> {
        let inverse = GeneralTransform3::from(init.init_loc.transformation().inverted()?);
        let mut state = self.state.write();
        (state.verts, state.indices) = (init.verts.clone(), init.indices.clone());
        (
            state.index_lower,
            state.index_upper,
            state.inv_init_location,
        ) = (init.index_lower, init.index_upper, inverse);
        Ok(())
    }
    /// `InitTriangulation`: the triangles of the index range, in patches of neighbours when the
    /// patch size is above 1, or in sub-arrays; false for invalid buffers or ranges.
    /// Example: `sens.init_triangulation(&init)?`.
    pub fn init_triangulation(&self, init: &Select3DPrimArrayInit) -> Result<bool, Failure> {
        self.reset_for_init(Graphic3dTypeOfPrimitiveArray::Triangles, &init.init_loc)?;
        self.state.write().cdg3d = Point3::new(0.0, 0.0, 0.0);
        if !self.check_init(init) {
            return Ok(false);
        }
        let count = (init.index_upper - init.index_lower + 1) / 3;
        let counts = InitCounts {
            first: init.index_lower / 3,
            count,
            has_groups: init.nb_groups > 1 && count / init.nb_groups > 10,
        };
        self.finish_primitive_initialization(init, &counts, (count / init.nb_groups.max(1)) * 3)
    }
    /// `InitPoints`: the points of the index range, in patches of close points when the patch
    /// size is above 1, or in sub-arrays. Example: `sens.init_points(&init)?`.
    pub fn init_points(&self, init: &Select3DPrimArrayInit) -> Result<bool, Failure> {
        self.reset_for_init(Graphic3dTypeOfPrimitiveArray::Points, &init.init_loc)?;
        if !self.check_init(init) {
            return Ok(false);
        }
        let count = init.index_upper - init.index_lower + 1;
        let counts = InitCounts {
            first: init.index_lower,
            count,
            has_groups: init.nb_groups > 1 && count / init.nb_groups > 10,
        };
        self.finish_primitive_initialization(init, &counts, count / init.nb_groups.max(1))
    }
    fn finish_primitive_initialization(
        &self,
        init: &Select3DPrimArrayInit,
        counts: &InitCounts,
        group_step: i32,
    ) -> Result<bool, Failure> {
        if counts.count < 1 || !self.store_init(init, counts)? {
            return Ok(false);
        }
        if counts.has_groups {
            return self.init_groups(init, group_step);
        }
        if self.read().prim_type == Graphic3dTypeOfPrimitiveArray::Triangles {
            self.index_triangles(counts);
        } else {
            self.index_points(counts);
        }
        self.finish_init(init.to_eval_min_max);
        Ok(true)
    }
    /// The box at the end of an init, when asked.
    fn finish_init(&self, to_eval_min_max: bool) {
        if to_eval_min_max {
            self.compute_bounding_box();
        }
    }
    /// The sub-arrays of an init split in `nb_groups` ranges of `div_step` indexes, then the
    /// combined box, the BVH indexes and the mean center; false when a sub-array fails.
    fn init_groups(&self, init: &Select3DPrimArrayInit, div_step: i32) -> Result<bool, Failure> {
        let mut groups = Vec::with_capacity(init.nb_groups as usize);
        let mut failures = 0;
        for index in 0..init.nb_groups {
            let (entity, ok) = self.init_group(init, index, div_step)?;
            groups.push(entity);
            failures += i32::from(!ok);
        }
        self.state.write().groups = Some(groups.clone());
        if failures != 0 {
            return Ok(false);
        }
        self.combine_initialized_groups(&groups);
        self.finish_init(init.to_eval_min_max);
        Ok(true)
    }
    /// Preserve the native center carry-over on grouped point reinitialization.
    fn combine_initialized_groups(&self, groups: &[Handle<Self>]) {
        // OCCT 3d097a0 resets this center only for InitTriangulation, not InitPoints.
        let mut center = self.state.read().cdg3d.xyz();
        for (index, group) in groups.iter().enumerate() {
            let Some(value) = group.get() else { continue };
            self.state.write().bnd_box.combine(&value.bounding_box());
            self.bvh_indices.set_index(index as i32, index as i32);
            center += value.center_of_geometry().xyz();
        }
        self.state.write().cdg3d = Point3::from(center / groups.len() as f64);
    }
    /// `Select3D_SensitivePrimitiveArray_InitFunctor`: one sub-array with the settings of this
    /// array, initialized and with its BVH built; false when its init fails.
    fn init_group(
        &self,
        init: &Select3DPrimArrayInit,
        index: i32,
        div_step: i32,
    ) -> Result<(Handle<Self>, bool), Failure> {
        let state = self.read();
        let entity = Handle::new(Self::new(&self.owner_id())?);
        self.copy_group_settings(&entity)?;
        let sub = self.group_initialization(init, index, div_step);
        let ok = match state.prim_type {
            Graphic3dTypeOfPrimitiveArray::Points => entity.init_points(&sub)?,
            Graphic3dTypeOfPrimitiveArray::Triangles => entity.init_triangulation(&sub)?,
            _ => false,
        };
        if ok {
            entity.bvh()?;
        }
        Ok((entity, ok))
    }
    fn group_initialization(
        &self,
        init: &Select3DPrimArrayInit,
        index: i32,
        div_step: i32,
    ) -> Select3DPrimArrayInit {
        let state = self.read();
        let lower = state.index_lower + index * div_step;
        let upper = (lower + div_step - 1).min(state.index_upper);
        Select3DPrimArrayInit {
            index_lower: lower,
            index_upper: upper,
            nb_groups: 1,
            ..init.clone()
        }
    }
    fn copy_group_settings(&self, entity: &Self) -> Result<(), Failure> {
        let state = self.read();
        entity.set_patch_size_max(state.patch_size_max);
        entity.set_patch_distance(state.patch_distance);
        entity.set_detect_elements(state.to_detect_elem);
        entity.set_detect_element_map(!state.detected_elem_map.is_null());
        entity.set_detect_nodes(state.to_detect_node);
        entity.set_detect_node_map(!state.detected_node_map.is_null());
        entity.set_sensitivity_factor(self.sensitivity_factor())
    }
    /// `computeBoundingBox` (protected): the combined box of the groups, or of every vertex.
    pub fn compute_bounding_box(&self) {
        let state = self.read();
        let mut bounds = Select3DBndBox3d::default();
        if let Some(groups) = &state.groups {
            for group in groups.iter().filter_map(Handle::get) {
                bounds.combine(&group.bounding_box());
            }
        } else if let Some(verts) = state.verts.get() {
            for index in 0..verts.nb_elements() {
                let p = point_vec(&state, index);
                bounds.add(&Vec3::new(f64::from(p.x), f64::from(p.y), f64::from(p.z)));
            }
        }
        self.state.write().bnd_box = bounds;
    }
}

/// The center of three vertices, preserving dimension-specific native rounding.
pub(super) fn triangle_center(state: &PrimArrayState, nodes: &[i32; 3]) -> Vec3<f32> {
    if state.is_3d {
        let [a, b, c] = nodes.map(|n| read_position(state, n).unwrap_or_default());
        return (a + b + c) / 3.0f32;
    }
    let [a, b, c] = nodes.map(|n| read_position_2d(state, n).unwrap_or_default());
    // Native Vec2 uses component division; Vec3 uses reciprocal multiplication.
    let sum = (a + b + c) / 3.0f32;
    Vec3::new(sum.x, sum.y, 0.0)
}

/// The single-precision position of a vertex (z = 0 in 2D).
pub(super) fn point_vec(state: &PrimArrayState, index: i32) -> Vec3<f32> {
    if state.is_3d {
        return read_position(state, index).unwrap_or_default();
    }
    let p = read_position_2d(state, index).unwrap_or_default();
    Vec3::new(p.x, p.y, 0.0)
}

fn init_range_is_valid(init: &Select3DPrimArrayInit, vertex_count: i32) -> bool {
    let (lower, upper) = (init.index_lower, init.index_upper);
    match init.indices.get() {
        Some(indices) => {
            !(lower < 0 || upper >= indices.nb_elements() || indices.nb_elements() == 0)
        }
        None => !(lower < 0 || upper >= vertex_count),
    }
}

fn reset_position_range(state: &mut PrimArrayState) {
    (
        state.index_lower,
        state.index_upper,
        state.pos_offset,
        state.pos_stride,
    ) = (0, 0, None, usize::MAX);
}
