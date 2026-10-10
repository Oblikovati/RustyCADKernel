//! Axis/polygon behavior retains native plane-only interior testing and pick updates.
use super::{SelectMgrAxisIntersector, SelectMgrBaseIntersector, SelectMgrViewClipRange};
use crate::select_basics::SelectBasicsPickResult;
use rustycad_tkernel::{precision::confusion, standard::Failure};
use rustycad_tkmath::gp::{Point3, Vector3};
impl SelectMgrAxisIntersector {
    pub(super) fn axis_boundary_pick(
        &self,
        points: &[Point3],
        pick: &mut SelectBasicsPickResult,
    ) -> bool {
        let mut matched = false;
        let mut candidate = SelectBasicsPickResult::default();
        pick.invalidate();
        for (index, start) in points.iter().enumerate() {
            let end = &points[(index + 1) % points.len()];
            if self.ray_segment_distance(start, end, &mut candidate) {
                matched = true;
                *pick = *SelectBasicsPickResult::min(pick, &candidate);
            }
        }
        matched
    }
    pub(super) fn axis_polygon_pick(
        &self,
        points: &[Point3],
        sensitivity: i32,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.initialized_axis("OverlapsPolygon")?;
        if sensitivity == 1 && !self.axis_boundary_pick(points, pick) {
            return Ok(false);
        }
        if sensitivity == 0 {
            return self.axis_polygon_interior(points, clip, pick);
        }
        Ok(!clip.is_clipped(pick.depth()))
    }
    fn axis_polygon_interior(
        &self,
        points: &[Point3],
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        if points.len() < 3 {
            return Err(Failure::out_of_range(&format!(
                "axis polygon points={}; expected at least 3",
                points.len()
            )));
        }
        let first = points[0].xyz() - points[1].xyz();
        let second = points[2].xyz() - points[1].xyz();
        let normal = Vector3::from(second.crossed(first));
        if normal.magnitude() <= confusion() {
            return self.overlaps_point_pick(&points[0], clip, pick);
        }
        if !self.ray_plane_intersection(&normal, &points[0], pick) {
            return Ok(false);
        }
        // OCCT 3d097a0 deliberately does not check containment in the polygon here.
        Ok(!clip.is_clipped(pick.depth()))
    }
}
