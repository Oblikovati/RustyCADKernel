//! Sphere separation and projected-boundary refinement for the frustum template.
use super::{SelectMgrBaseFrustum, SelectMgrFrustumCore};
use rustycad_tkernel::precision::angular;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Direction3, Point3, Vector3};

impl<const N: usize> SelectMgrFrustumCore<N> {
    fn sphere_plane_relation(&self, index: usize, center: Point3, radius: f64) -> (bool, bool) {
        let plane = self.planes[index];
        let norm = plane.dot(plane).sqrt();
        let projection = plane.dot(center.xyz()) / norm;
        let (max, min) = (
            self.max_verts_projections[index] / norm,
            self.min_verts_projections[index] / norm,
        );
        let separated = projection > max + radius || projection < min - radius;
        (
            separated,
            projection >= min + radius && projection <= max - radius,
        )
    }
    fn sphere_planes_overlap(
        &self,
        center: Point3,
        radius: f64,
        is_ortho: bool,
        inside: &mut Option<&mut bool>,
    ) -> (bool, bool) {
        let mut full = true;
        for index in (0..N).step_by(self.inc_factor(is_ortho)) {
            let (separated, enclosed) = self.sphere_plane_relation(index, center, radius);
            if separated {
                return (false, full);
            }
            if let Some(flag) = inside.as_mut() {
                **flag &= enclosed;
            }
            full &= enclosed;
        }
        (true, full)
    }
    /// Test sphere side planes, then the projected outline when no inside output is requested.
    /// Example: `core.has_sphere_overlap(&frustum, center, radius, false, None)?`.
    pub fn has_sphere_overlap<T: SelectMgrBaseFrustum + ?Sized>(
        &self,
        volume: &T,
        center: Point3,
        radius: f64,
        is_ortho: bool,
        mut inside: Option<&mut bool>,
    ) -> Result<bool, Failure> {
        let (overlaps, full) = self.sphere_planes_overlap(center, radius, is_ortho, &mut inside);
        if !overlaps {
            return Ok(false);
        }
        if inside.is_some() || full {
            return Ok(true);
        }
        self.sphere_boundary_overlap(volume, center, radius)
    }
    fn sphere_boundary_overlap<T: SelectMgrBaseFrustum + ?Sized>(
        &self,
        volume: &T,
        center: Point3,
        radius: f64,
    ) -> Result<bool, Failure> {
        let first = Vector3::between(self.vertices[0], self.vertices[2]);
        let second = Vector3::between(self.vertices[0], self.vertices[2 * N - 2]);
        if first.is_parallel(second, angular())? {
            return Ok(false);
        }
        let normal = Direction3::from_xyz(first.xyz().crossed(*second.xyz()))?;
        let mut boundary: Vec<Point3> = self.vertices.iter().step_by(2).copied().collect();
        boundary.push(Point3::default()); // Native allocates an unused final entry.
        let projected = Point3::from(center.xyz() - normal.xyz() * center.xyz().dot(normal.xyz()));
        volume.is_boundary_intersect_sphere(&projected, radius, &normal, &boundary, &mut false)
    }
}
