//! Native circle overlap, retaining the untransformed center used by OCCT 3d097a0.
use super::SelectMgrFrustumCore;
use super::frustum_quadric_geometry::{frustum_circle_intersection, frustum_point_inside};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Direction3, Plane3, Point3, Transform3};

impl<const N: usize> SelectMgrFrustumCore<N> {
    fn circle_center_projection(&self) -> Result<Point3, Failure> {
        let ray = Direction3::from_vector(self.view_edge())?;
        let [a, b, c, d] = Plane3::new(self.vertices[0], ray)?.coefficients()?;
        // Preserve the native origin dot product, including signed-zero arithmetic.
        let t = -(Point3::default().xyz().dot(ray.xyz()) + d);
        Ok(Point3::new(a * t, b * t, c * t))
    }
    fn circle_containment(
        &self,
        vertices: &[Point3],
        radius: f64,
        filled: bool,
    ) -> Result<(bool, bool), Failure> {
        let center = self.circle_center_projection()?;
        let center_inside = frustum_point_inside(center, vertices)?;
        let has_vertex_outside = vertices
            .iter()
            .any(|vertex| vertex.distance(&center) > radius);
        let inside = has_vertex_outside && center_inside;
        let overlap = if filled {
            !has_vertex_outside || (center_inside && has_vertex_outside)
        } else {
            inside
        };
        Ok((overlap, inside))
    }
    /// Test an outlined or filled circle with the native projected-center convention.
    /// Example: `core.has_circle_overlap(1., transform, true, false, None)?`.
    pub fn has_circle_overlap(
        &self,
        radius: f64,
        transform: Transform3,
        filled: bool,
        is_ortho: bool,
        inside: Option<&mut bool>,
    ) -> Result<bool, Failure> {
        let vertices = self.outline_vertices(is_ortho);
        let intersects = frustum_circle_intersection(
            radius,
            Point3::default(),
            transform,
            &vertices,
            self.view_edge(),
        )?;
        let (overlap, enclosed) = if intersects {
            (true, false)
        } else {
            self.circle_containment(&vertices, radius, filled)?
        };
        if let Some(flag) = inside {
            *flag = enclosed;
        }
        Ok(overlap)
    }
}
