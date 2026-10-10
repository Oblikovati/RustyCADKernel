//! Native axis/triangle branches: degeneration, parallel edges and barycentric interior.
use super::selecting_triangle::SelectingTriangle;
use super::{SelectMgrAxisIntersector, SelectMgrBaseIntersector, SelectMgrViewClipRange};
use crate::select_basics::SelectBasicsPickResult;
use rustycad_tkernel::{precision::angular, standard::Failure};
use rustycad_tkmath::gp::{Axis1, Direction3, Point3, Vector3, resolution};
impl SelectMgrAxisIntersector {
    pub(super) fn axis_triangle_pick(
        &self,
        points: [Point3; 3],
        sensitivity: i32,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        let axis = self.initialized_axis("OverlapsTriangle")?;
        if sensitivity == 1 {
            return self.axis_polygon_pick(
                &[points[0], points[1], points[2], points[0]],
                1,
                clip,
                pick,
            );
        }
        if sensitivity != 0 {
            return Ok(pick.is_valid() && !clip.is_clipped(pick.depth()));
        }
        self.axis_triangle_interior(&SelectingTriangle::new(points), axis, clip, pick)
    }
    fn axis_triangle_interior(
        &self,
        triangle: &SelectingTriangle,
        axis: Axis1,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        if triangle.normal.square_modulus() < resolution() {
            return self.axis_degenerate_triangle(triangle, clip, pick);
        }
        let alpha = triangle.normal.dot(axis.direction().xyz());
        if alpha.abs() < resolution() {
            return Ok(self.axis_parallel_triangle(triangle, clip, pick));
        }
        self.axis_triangle_plane(triangle, axis, alpha, pick)?;
        Ok(pick.is_valid() && !clip.is_clipped(pick.depth()))
    }
    fn axis_degenerate_triangle(
        &self,
        triangle: &SelectingTriangle,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        if triangle.edges[0].square_modulus() > resolution() {
            return self.overlaps_segment(&triangle.points[0], &triangle.points[1], clip, pick);
        }
        if triangle.edges[1].square_modulus() > resolution() {
            return self.overlaps_segment(&triangle.points[1], &triangle.points[2], clip, pick);
        }
        self.overlaps_point_pick(&triangle.points[0], clip, pick)
    }
    fn axis_parallel_triangle(
        &self,
        triangle: &SelectingTriangle,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> bool {
        self.axis_boundary_pick(&triangle.points, pick);
        pick.set_surface_normal_vec(&Vector3::from(triangle.normal));
        pick.is_valid() && !clip.is_clipped(pick.depth())
    }
    fn axis_triangle_plane(
        &self,
        triangle: &SelectingTriangle,
        axis: Axis1,
        alpha: f64,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<(), Failure> {
        let (point, interior) = triangle.ray_parameters(axis, alpha);
        if interior {
            pick.set_depth(axis.location().distance(&point));
            pick.set_picked_point(point);
            pick.set_surface_normal_vec(&Vector3::from(triangle.normal));
            return Ok(());
        }
        self.axis_triangle_edge_pick(triangle, axis, point, pick)
    }
    fn axis_triangle_edge_pick(
        &self,
        triangle: &SelectingTriangle,
        axis: Axis1,
        point: Point3,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<(), Failure> {
        let first = triangle.nearest_edge(point);
        let mut last = (first + 1) % 3;
        let edge = triangle.points[last].xyz() - triangle.points[first].xyz();
        if edge.square_modulus() > resolution()
            && axis
                .direction()
                .is_parallel(Direction3::from_xyz(edge)?, angular())
        {
            last = if first == 0 { 2 } else { first - 1 };
        }
        if self.ray_segment_distance(&triangle.points[first], &triangle.points[last], pick) {
            pick.set_surface_normal_vec(&Vector3::from(triangle.normal));
        }
        Ok(())
    }
}
