//! SAT first, then native degeneracy, parallel-edge and barycentric picking branches.
use super::selecting_triangle::SelectingTriangle;
use super::*;
use crate::select_basics::SelectBasicsPickResult;
use rustycad_tkernel::{precision::angular, standard::Failure};
use rustycad_tkmath::gp::{Axis1, Direction3, Point3, Vector3, Xyz, resolution};
impl SelectMgrRectangularFrustum {
    pub(super) fn rectangular_triangle_pick(
        &self,
        points: [Point3; 3],
        sensitivity: i32,
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        self.rectangular_initialized("OverlapsTriangle")?;
        if sensitivity == 1 {
            return self.overlaps_polygon(
                &[points[0], points[1], points[2], points[0]],
                1,
                clip,
                pick,
            );
        }
        if sensitivity != 0 {
            return Ok(!clip.is_clipped(pick.depth()));
        }
        self.rectangular_triangle_sat(points, clip, pick)
    }
    fn rectangular_triangle_sat(
        &self,
        points: [Point3; 3],
        clip: &SelectMgrViewClipRange,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<bool, Failure> {
        let mut normal = Xyz::new(f64::MAX, f64::MAX, f64::MAX);
        if !self.frustum_core.read().has_triangle_overlap(
            &points[0],
            &points[1],
            &points[2],
            self.is_orthographic(),
            &mut normal,
        ) {
            return Ok(false);
        }
        let triangle = SelectingTriangle::new(points);
        if normal.square_modulus() < resolution() {
            return self.rectangular_degenerate_triangle(&triangle, clip, pick);
        }
        self.rectangular_triangle_interior(&triangle, pick)?;
        Ok(!clip.is_clipped(pick.depth()))
    }
    fn rectangular_degenerate_triangle(
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
    fn rectangular_triangle_interior(
        &self,
        triangle: &SelectingTriangle,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<(), Failure> {
        let ray = *self.ray.read();
        let alpha = triangle.normal.dot(ray.direction.xyz());
        if alpha.abs() < resolution() {
            self.rectangular_boundary_pick(&triangle.points, false, pick);
        } else {
            self.rectangular_triangle_plane(triangle, alpha, pick)?;
        }
        pick.set_surface_normal_vec(&Vector3::from(triangle.normal));
        Ok(())
    }
    fn rectangular_triangle_plane(
        &self,
        triangle: &SelectingTriangle,
        alpha: f64,
        pick: &mut SelectBasicsPickResult,
    ) -> Result<(), Failure> {
        let ray = *self.ray.read();
        let (point, interior) = triangle.ray_parameters(Axis1::new(ray.near, ray.direction), alpha);
        if interior {
            pick.set_depth(ray.near.distance(&point) * ray.scale);
            pick.set_picked_point(point);
            return Ok(());
        }
        let first = triangle.nearest_edge(point);
        let mut last = (first + 1) % 3;
        let edge = triangle.points[last].xyz() - triangle.points[first].xyz();
        if edge.square_modulus() > resolution()
            && ray
                .direction
                .is_parallel(Direction3::from_xyz(edge)?, angular())
        {
            last = if first == 0 { 2 } else { first - 1 };
        }
        self.segment_segment_distance(&triangle.points[first], &triangle.points[last], pick);
        Ok(())
    }
}
