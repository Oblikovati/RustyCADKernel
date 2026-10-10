//! Polygon/triangle SAT preserving the native normal output timing.
use super::SelectMgrFrustumCore;
use rustycad_tkernel::standard::{
    Failure,
    exceptions::out_of_range,
    real::{real_first, real_last},
};
use rustycad_tkmath::{
    bvh::{std_max, std_min},
    gp::{Point3, Xyz},
};

fn polygon_normal(points: &[Point3]) -> Result<Xyz, Failure> {
    if points.len() < 3 {
        return Err(out_of_range(&format!(
            "polygon vertex count {}: expected at least 3",
            points.len()
        )));
    }
    let (first, second, third) = (points[0].xyz(), points[1].xyz(), points[2].xyz());
    Ok((third - second).crossed(first - second))
}
fn triangle_range(points: [Point3; 3], axis: Xyz) -> (f64, f64) {
    let first = axis.dot(points[0].xyz());
    points[1..].iter().fold((first, first), |(lo, hi), point| {
        let projection = axis.dot(point.xyz());
        (std_min(lo, projection), std_max(hi, projection))
    })
}
impl<const N: usize> SelectMgrFrustumCore<N> {
    fn polygon_planes_overlap(&self, points: &[Point3], ortho: bool) -> bool {
        for index in (0..N + 1).step_by(self.inc_factor(ortho)) {
            let (min, max) = Self::points_range(points, &self.planes[index]);
            if min > self.max_verts_projections[index] || max < self.min_verts_projections[index] {
                return false;
            }
        }
        true
    }
    fn polygon_cross_axes_overlap(&self, points: &[Point3], ortho: bool) -> bool {
        let count = Self::directions_nb(ortho);
        for index in 0..points.len() * count {
            let edge = index / count;
            let direction = points[(edge + 1) % points.len()].xyz() - points[edge].xyz();
            let axis = direction.crossed(self.edge_dirs[index % count]);
            if !self.polygon_axis_overlap(points, &axis) {
                return false;
            }
        }
        true
    }
    /// `hasPolygonOverlap`: computes the normal before testing any plane.
    /// Example: `core.has_polygon_overlap(&points, false, &mut normal)?`.
    pub fn has_polygon_overlap(
        &self,
        points: &[Point3],
        ortho: bool,
        normal: &mut Xyz,
    ) -> Result<bool, Failure> {
        *normal = polygon_normal(points)?;
        let projection = normal.dot(points[0].xyz());
        let (min, max) = self.vertex_range(normal, (real_last(), real_first()));
        if projection > max || projection < min || !self.polygon_planes_overlap(points, ortho) {
            return Ok(false);
        }
        Ok(self.polygon_cross_axes_overlap(points, ortho))
    }
    fn triangle_planes_overlap(&self, points: [Point3; 3], ortho: bool) -> bool {
        for index in (0..N + 1).step_by(self.inc_factor(ortho)) {
            let (min, max) = triangle_range(points, self.planes[index]);
            if min > self.max_verts_projections[index] || max < self.min_verts_projections[index] {
                return false;
            }
        }
        true
    }
    /// `isSeparated(triangle, axis)`: retains the native early overlap exit.
    fn is_separated_triangle(&self, points: [Point3; 3], axis: &Xyz) -> bool {
        let (low, high) = Self::points_range(&points, axis);
        let (mut min, mut max) = (real_last(), real_first());
        for vertex in &self.vertices {
            let projection = vertex.xyz().dot(*axis);
            min = std_min(min, projection);
            max = std_max(max, projection);
            if min <= high && max >= low {
                return false;
            }
        }
        min > high || max < low
    }
    fn triangle_cross_axes_overlap(
        &self,
        points: [Point3; 3],
        edges: [Xyz; 3],
        ortho: bool,
    ) -> bool {
        let count = Self::directions_nb(ortho);
        for index in 0..3 * count {
            let axis = self.edge_dirs[index % count].crossed(edges[index / count]);
            if self.is_separated_triangle(points, &axis) {
                return false;
            }
        }
        true
    }
    /// `hasTriangleOverlap`: leaves the caller's normal untouched on an early plane rejection.
    /// Example: `core.has_triangle_overlap(&a, &b, &c, false, &mut normal)`.
    pub fn has_triangle_overlap(
        &self,
        a: &Point3,
        b: &Point3,
        c: &Point3,
        ortho: bool,
        normal: &mut Xyz,
    ) -> bool {
        let edges = [b.xyz() - a.xyz(), c.xyz() - b.xyz(), a.xyz() - c.xyz()];
        let points = [*a, *b, *c];
        if !self.triangle_planes_overlap(points, ortho) {
            return false;
        }
        *normal = edges[2].crossed(edges[0]);
        if self.is_separated_triangle(points, normal) {
            return false;
        }
        self.triangle_cross_axes_overlap(points, edges, ortho)
    }
}
