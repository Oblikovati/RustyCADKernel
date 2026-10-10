//! Point and segment/frustum separating-axis tests.
use super::SelectMgrFrustumCore;
use rustycad_tkernel::precision::confusion;
use rustycad_tkmath::{
    bvh::{std_max, std_min},
    gp::{Point3, Xyz},
};
impl<const N: usize> SelectMgrFrustumCore<N> {
    /// `hasPointOverlap`: between every pair of planes; `core.has_point_overlap(&point, false)`.
    pub fn has_point_overlap(&self, point: &Point3, ortho: bool) -> bool {
        (0..N + 1).step_by(self.inc_factor(ortho)).all(|index| {
            let projection = self.planes[index].dot(point.xyz());
            !(projection > self.max_verts_projections[index]
                || projection < self.min_verts_projections[index])
        })
    }
    fn segment_planes_overlap(&self, start: &Point3, end: &Point3, ortho: bool) -> bool {
        for index in (0..N + 1).step_by(self.inc_factor(ortho)) {
            let (first, last) = (
                self.planes[index].dot(start.xyz()),
                self.planes[index].dot(end.xyz()),
            );
            if std_min(first, last) > self.max_verts_projections[index]
                || std_max(first, last) < self.min_verts_projections[index]
            {
                return false;
            }
        }
        true
    }
    fn segment_axis_overlap(&self, start: &Point3, end: &Point3, direction: Xyz) -> bool {
        let (min, max) = self.vertex_range(&direction, (f64::MAX, -f64::MAX));
        let (first, last) = (direction.dot(start.xyz()), direction.dot(end.xyz()));
        !(std_min(first, last) > max || std_max(first, last) < min)
    }
    /// `hasSegmentOverlap`: a degenerate segment overlaps even outside the volume.
    /// Example: `core.has_segment_overlap(&start, &end, false)`.
    pub fn has_segment_overlap(&self, start: &Point3, end: &Point3, ortho: bool) -> bool {
        let direction = end.xyz() - start.xyz();
        if direction.modulus() < confusion() {
            return true;
        }
        if !self.segment_planes_overlap(start, end, ortho)
            || !self.segment_axis_overlap(start, end, direction)
        {
            return false;
        }
        (0..Self::directions_nb(ortho)).all(|index| {
            self.segment_axis_overlap(start, end, direction.crossed(self.edge_dirs[index]))
        })
    }
}
