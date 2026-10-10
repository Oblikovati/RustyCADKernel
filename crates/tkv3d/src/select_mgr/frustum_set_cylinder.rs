//! Native six-point projected cylinder containment before child overlap fallback.
use super::*;
use rustycad_tkernel::{precision::angular, standard::Failure};
use rustycad_tkmath::gp::{Direction3, Plane3, Point3, Transform3, Vector3, Xyz};
impl SelectMgrTriangularFrustumSet {
    pub(super) fn frustum_set_near_plane(&self) -> Result<(Plane3, Direction3), Failure> {
        let children = self.frustums.read();
        let child = children.first().ok_or_else(|| {
            Failure::with_message(
                "Standard_NoSuchObject",
                "TriangularFrustumSet children=0; expected a first frustum",
            )
        })?;
        let core = child.frustum_core.read();
        let vertices = core.vertices();
        let normal = Direction3::from_vector(
            Vector3::between(vertices[0], vertices[1])
                .crossed(Vector3::between(vertices[0], vertices[2])),
        )?;
        Ok((Plane3::new(vertices[0], normal)?, normal))
    }
    pub(super) fn frustum_set_cylinder_overlap(
        &self,
        radii: (f64, f64),
        height: f64,
        transform: Transform3,
        hollow: bool,
        mut inside: Option<&mut bool>,
    ) -> Result<bool, Failure> {
        let (plane, normal) = self.frustum_set_near_plane()?;
        let points = frustum_set_cylinder_silhouette(radii, height, transform, plane, normal)?;
        let all_inside = self.frustum_set_contains_silhouette(points)?;
        if let Some(inside) = inside.as_deref_mut() {
            *inside &= all_inside;
        }
        if all_inside {
            return Ok(true);
        }
        self.frustum_set_any_child(|child| {
            child.overlaps_cylinder(
                radii.0,
                radii.1,
                height,
                &transform,
                hollow,
                inside.as_deref_mut(),
            )
        })
    }
    fn frustum_set_contains_silhouette(&self, points: [Point3; 6]) -> Result<bool, Failure> {
        let mut all_inside = true;
        for point in points {
            all_inside &= self.frustum_set_projected_inside(point)?;
        }
        Ok(all_inside)
    }
    fn frustum_set_projected_inside(&self, point: Point3) -> Result<bool, Failure> {
        for child in self.frustums.read().iter() {
            let core = child.frustum_core.read();
            if super::frustum_quadric_geometry::frustum_point_inside(point, &core.vertices()[..3])?
            {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
pub(super) fn frustum_set_plane_projection(
    point: Point3,
    plane: Plane3,
    normal: Direction3,
) -> Result<Point3, Failure> {
    let [a, b, c, d] = plane.coefficients()?;
    let parameter = -(point.xyz().dot(normal.xyz()) + d) / normal.dot(normal);
    Ok(Point3::new(
        a * parameter + point.x(),
        b * parameter + point.y(),
        c * parameter + point.z(),
    ))
}
fn frustum_set_cylinder_silhouette(
    radii: (f64, f64),
    height: f64,
    transform: Transform3,
    plane: Plane3,
    normal: Direction3,
) -> Result<[Point3; 6], Failure> {
    let axis = Direction3::Z.transformed(transform);
    let bottom = Point3::default().transformed(transform);
    let top = Point3::from(bottom.xyz() + axis.xyz() * height);
    let cosine = axis.angle(&normal).cos();
    let bottom = frustum_set_plane_projection(bottom, plane, normal)?;
    let top = frustum_set_plane_projection(top, plane, normal)?;
    let direction = frustum_set_projected_axis(bottom, top);
    let ends = if axis.is_parallel(normal, angular()) {
        Direction3::Y.transformed(transform)
    } else {
        axis.crossed(normal)?
    };
    Ok(frustum_set_silhouette_points(
        radii,
        (bottom, top),
        direction,
        ends,
        cosine,
    ))
}
fn frustum_set_silhouette_points(
    radii: (f64, f64),
    (bottom, top): (Point3, Point3),
    direction: Xyz,
    ends: Direction3,
    cosine: f64,
) -> [Point3; 6] {
    [
        Point3::from(bottom.xyz() - direction * radii.0 * cosine.abs()),
        Point3::from(top.xyz() + direction * radii.1 * cosine.abs()),
        Point3::from(top.xyz() + ends.xyz() * radii.1),
        Point3::from(top.xyz() - ends.xyz() * radii.1),
        Point3::from(bottom.xyz() + ends.xyz() * radii.0),
        Point3::from(bottom.xyz() - ends.xyz() * radii.0),
    ]
}

fn frustum_set_projected_axis(bottom: Point3, top: Point3) -> Xyz {
    let offset = top.xyz() - bottom.xyz();
    let length = offset.modulus();
    if length > 0. {
        offset / length
    } else {
        Xyz::default()
    }
}
