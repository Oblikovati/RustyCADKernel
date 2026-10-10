//! Polygon picks retain native boundary minimum and plane-only result fields.
use super::*;
use crate::select_basics::SelectBasicsPickResult;
use rustycad_tkernel::{precision::confusion, standard::Failure};
use rustycad_tkmath::gp::{Point3, Vector3, Xyz};
impl SelectMgrRectangularFrustum {
    pub(super) fn rectangular_boundary_pick(
        &self,
        points: &[Point3],
        test_overlap: bool,
        pick: &mut SelectBasicsPickResult,
    ) -> bool {
        let mut candidate = SelectBasicsPickResult::default();
        let mut matched = false;
        pick.invalidate();
        for (index, first) in points.iter().enumerate() {
            let last = &points[(index + 1) % points.len()];
            if test_overlap
                && !self.frustum_core.read().has_segment_overlap(
                    first,
                    last,
                    self.is_orthographic(),
                )
            {
                continue;
            }
            matched = true;
            self.segment_segment_distance(first, last, &mut candidate);
            *pick = *SelectBasicsPickResult::min(pick, &candidate);
        }
        matched
    }
    pub(super) fn rectangular_polygon_pick(
        &self,
        points: &[Point3],
        sensitivity: i32,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.rectangular_initialized("OverlapsPolygon")?;
        if sensitivity == 1 && !self.rectangular_boundary_pick(points, true, pick) {
            return Ok(false);
        }
        if sensitivity == 0 {
            return self.rectangular_polygon_interior(points, clip, pick);
        }
        Ok(!clip.is_clipped(pick.depth()))
    }
    fn rectangular_polygon_interior(
        &self,
        points: &[Point3],
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        let mut normal = Xyz::new(f64::MAX, f64::MAX, f64::MAX);
        if !self.frustum_core.read().has_polygon_overlap(
            points,
            self.is_orthographic(),
            &mut normal,
        )? {
            return Ok(false);
        }
        if normal.modulus() <= confusion() {
            return self.overlaps_point_pick(&points[0], clip, pick);
        }
        Ok(
            self.segment_plane_intersection(&Vector3::from(normal), &points[0], pick)
                && !clip.is_clipped(pick.depth()),
        )
    }
}
