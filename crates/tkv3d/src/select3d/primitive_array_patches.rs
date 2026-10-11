//! Native patch assembly preserves the upstream boundary and rounding rules.
use super::primitive_array::{
    PrimArrayState, Select3DSensitivePrimitiveArray, point_index, tri_indices, vec3_to_pnt,
};
use super::primitive_array_init::{InitCounts, point_vec, triangle_center};
use rustycad_tkernel::ncollection::Vec3;
use rustycad_tkernel::standard::short_real::short_real_last;
use rustycad_tkservice::graphic3d::Graphic3dBufferMembers as _;

#[derive(Default)]
struct PatchCursor {
    first: i32,
    size: i32,
    previous_point: Option<Vec3<f32>>,
}

impl Select3DSensitivePrimitiveArray {
    /// The float center and BVH indexes of triangle patches.
    pub(super) fn index_triangles(&self, counts: &InitCounts) {
        let state = self.read();
        let mut center = Vec3::<f32>::new(0.0, 0.0, 0.0);
        let mut previous = [-1; 3];
        let mut cursor = PatchCursor::default();
        self.begin_patch_assembly();
        for iter in 0..counts.count {
            let mut current = tri_indices(&state.indices, (counts.first + iter) * 3);
            center += triangle_center(&state, &current);
            std::mem::swap(&mut current, &mut previous);
            self.record_triangle_patch(counts, iter, &current, &previous, &mut cursor);
        }
        self.finish_patch_assembly(counts, &cursor, center);
    }
    /// Shared-node boundaries include the native empty first triangle patch.
    fn record_triangle_patch(
        &self,
        counts: &InitCounts,
        iter: i32,
        current: &[i32; 3],
        previous: &[i32; 3],
        cursor: &mut PatchCursor,
    ) {
        if !self.bvh_indices.has_patches() {
            self.bvh_indices.set_index(iter, counts.first + iter);
            return;
        }
        if cursor.size < self.patch_size_max() && previous.iter().any(|n| current.contains(n)) {
            cursor.size += 1;
            return;
        }
        self.push_patch(counts.first + cursor.first, cursor.size);
        (cursor.first, cursor.size) = (iter, 0);
    }
    /// The float center and BVH indexes of point patches.
    pub(super) fn index_points(&self, counts: &InitCounts) {
        let state = self.read();
        let mut center = Vec3::<f32>::new(0.0, 0.0, 0.0);
        let mut cursor = PatchCursor::default();
        self.begin_patch_assembly();
        for iter in 0..counts.count {
            let index = point_index(&state.indices, counts.first + iter);
            let point = point_vec(&state, index);
            center += point;
            self.record_point_patch(&state, counts, iter, point, &mut cursor);
        }
        self.finish_patch_assembly(counts, &cursor, center);
    }
    /// Preserve the strict consecutive-point distance comparison.
    fn record_point_patch(
        &self,
        state: &PrimArrayState,
        counts: &InitCounts,
        iter: i32,
        point: Vec3<f32>,
        cursor: &mut PatchCursor,
    ) {
        if !self.bvh_indices.has_patches() {
            self.bvh_indices.set_index(iter, counts.first + iter);
            return;
        }
        if cursor.extend_point(point, state.patch_size_max, state.patch_distance) {
            return;
        }
        self.push_patch(counts.first + cursor.first, cursor.size.max(1));
        (cursor.first, cursor.size) = (iter, 0);
    }
    fn begin_patch_assembly(&self) {
        if self.bvh_indices.has_patches() {
            self.bvh_indices.set_nb_elements(0);
        }
    }
    fn finish_patch_assembly(&self, counts: &InitCounts, cursor: &PatchCursor, center: Vec3<f32>) {
        if cursor.size != 0 {
            self.push_patch(counts.first + cursor.first, cursor.size);
        }
        self.state.write().cdg3d = vec3_to_pnt(center / counts.count as f32);
    }
    /// `SetIndex(NbElements++, index, patchSize)`.
    fn push_patch(&self, index: i32, patch_size: i32) {
        let position = self.bvh_indices.nb_elements();
        self.bvh_indices.set_nb_elements(position + 1);
        self.bvh_indices
            .set_index_patch(position, index, patch_size);
    }
}
fn point_patch_distance_squared(distance: f32) -> f32 {
    if distance < short_real_last() {
        return distance * distance;
    }
    distance
}

impl PatchCursor {
    fn extend_point(&mut self, point: Vec3<f32>, maximum: i32, distance: f32) -> bool {
        let last = self.previous_point.replace(point);
        if self.size >= maximum
            || !last.is_some_and(|p| {
                (point - p).square_modulus() < point_patch_distance_squared(distance)
            })
        {
            return false;
        }
        self.size += 1;
        true
    }
}
