//! Projected cylinder/cone overlap of SelectMgr_Frustum (OCCT 3d097a0).
use super::SelectMgrFrustumCore;
use super::frustum_quadric_geometry::{
    frustum_circle_intersection, frustum_inside_end_faces, frustum_point_inside,
    frustum_segments_intersect,
};
use rustycad_tkernel::precision::angular;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::bvh::std_max;
use rustycad_tkmath::gp::{Direction3, Plane3, Point3, Transform3, Vector3, Xyz};

#[derive(Clone, Copy)]
struct CylinderQuery {
    radii: (f64, f64),
    height: f64,
    transform: Transform3,
    hollow: bool,
}

struct CylinderProjection {
    points: [Point3; 6],
    axis: Xyz,
}

fn plane_projection(point: Point3, ray: Direction3, coefficients: [f64; 4]) -> Point3 {
    let t = -(point.xyz().dot(ray.xyz()) + coefficients[3]);
    Point3::new(
        coefficients[0] * t + point.x(),
        coefficients[1] * t + point.y(),
        coefficients[2] * t + point.z(),
    )
}

fn projected_axis(bottom: Point3, top: Point3) -> Xyz {
    if top.distance(&bottom) > 0.0 {
        return (top.xyz() - bottom.xyz()) / top.distance(&bottom);
    }
    Xyz::new(0.0, 0.0, 0.0)
}

fn cylinder_projection(
    origin: Point3,
    ray: Direction3,
    transform: Transform3,
    radii: (f64, f64),
    height: f64,
) -> Result<CylinderProjection, Failure> {
    let cylinder_normal = Direction3::Z.transformed(transform);
    let bottom = Point3::default().transformed(transform);
    let top = Point3::from(bottom.xyz() + cylinder_normal.xyz() * height);
    let coefficients = Plane3::new(origin, ray)?.coefficients()?;
    let bottom = plane_projection(bottom, ray, coefficients);
    let top = plane_projection(top, ray, coefficients);
    let axis = projected_axis(bottom, top);
    let end_direction = if cylinder_normal.is_parallel(ray, angular()) {
        Direction3::Y.transformed(transform)
    } else {
        cylinder_normal.crossed(ray)?
    };
    let cosine = cylinder_normal.angle(&ray).cos().abs();
    let points = cylinder_silhouette(bottom, top, axis, end_direction, radii, cosine);
    Ok(CylinderProjection { points, axis })
}

fn cylinder_silhouette(
    bottom: Point3,
    top: Point3,
    axis: Xyz,
    end_direction: Direction3,
    radii: (f64, f64),
    cosine: f64,
) -> [Point3; 6] {
    [
        Point3::from(bottom.xyz() - axis * radii.0 * cosine),
        Point3::from(bottom.xyz() + end_direction.xyz() * radii.0),
        Point3::from(top.xyz() + end_direction.xyz() * radii.1),
        Point3::from(top.xyz() + axis * radii.1 * cosine),
        Point3::from(top.xyz() - end_direction.xyz() * radii.1),
        Point3::from(bottom.xyz() - end_direction.xyz() * radii.0),
    ]
}

fn vertices_enter_cylinder(
    vertices: &[Point3],
    projection: &CylinderProjection,
    radius: f64,
) -> Result<bool, Failure> {
    for vertex in vertices {
        if (projection.axis.dot(projection.axis) == 0.0
            && vertex.distance(&projection.points[0]) <= radius)
            || frustum_point_inside(*vertex, &projection.points)?
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn cylinder_edges_cross(vertices: &[Point3], points: &[Point3; 6]) -> bool {
    for index in 0..vertices.len() {
        let first = vertices[index];
        let last = vertices[(index + 1) % vertices.len()];
        if [(1, 2), (4, 5), (4, 2), (1, 5)]
            .iter()
            .any(|&(a, b)| frustum_segments_intersect([points[a], points[b], first, last]))
        {
            return true;
        }
    }
    false
}

fn cylinder_caps_cross(
    vertices: &[Point3],
    edge: Vector3,
    transform: Transform3,
    radii: (f64, f64),
    height: f64,
) -> Result<bool, Failure> {
    Ok(
        frustum_circle_intersection(radii.0, Point3::default(), transform, vertices, edge)?
            || frustum_circle_intersection(
                radii.1,
                Point3::new(0., 0., height),
                transform,
                vertices,
                edge,
            )?,
    )
}

fn cylinder_enclosed(points: &[Point3; 6], vertices: &[Point3]) -> Result<bool, Failure> {
    let mut enclosed = true;
    // Native &= evaluates every point, preserving later direction failures after a miss.
    for point in points {
        enclosed &= frustum_point_inside(*point, vertices)?;
    }
    Ok(enclosed)
}

impl<const N: usize> SelectMgrFrustumCore<N> {
    pub(crate) fn outline_vertices(&self, is_ortho: bool) -> Vec<Point3> {
        if N == 4 && is_ortho {
            return [0, 2, 6, 4].map(|i| self.vertices[i]).to_vec();
        }
        self.vertices[..N].to_vec()
    }
    pub(crate) fn view_edge(&self) -> Vector3 {
        Vector3::from(self.edge_dirs[if N == 4 { 4 } else { 0 }])
    }
    /// Test all projected vertices against both end faces; `core.is_inside_cylinder_end_face(1., 2., 3., transform, &vertices)?`.
    pub fn is_inside_cylinder_end_face(
        &self,
        bottom: f64,
        top: f64,
        height: f64,
        transform: Transform3,
        vertices: &[Point3],
    ) -> Result<bool, Failure> {
        frustum_inside_end_faces((bottom, top), height, transform, vertices, self.view_edge())
    }
    /// Test a coplanar point by its accumulated vertex angles; `core.is_dot_inside(point, &vertices)?`.
    pub fn is_dot_inside(&self, point: Point3, vertices: &[Point3]) -> Result<bool, Failure> {
        frustum_point_inside(point, vertices)
    }
    fn cylinder_relation(
        &self,
        query: CylinderQuery,
        vertices: &[Point3],
    ) -> Result<(bool, bool), Failure> {
        let CylinderQuery {
            radii,
            height,
            transform,
            hollow,
        } = query;
        if hollow && frustum_inside_end_faces(radii, height, transform, vertices, self.view_edge())?
        {
            return Ok((false, false));
        }
        let ray = Direction3::from_vector(self.view_edge())?;
        let projection = cylinder_projection(self.vertices[0], ray, transform, radii, height)?;
        let partial = vertices_enter_cylinder(vertices, &projection, std_max(radii.1, radii.0))?
            || cylinder_edges_cross(vertices, &projection.points)
            || (!hollow
                && cylinder_caps_cross(vertices, self.view_edge(), transform, radii, height)?);
        if partial {
            return Ok((true, false));
        }
        let enclosed = cylinder_enclosed(&projection.points, vertices)?;
        Ok((enclosed, enclosed))
    }
    /// Test a cylinder or cone, preserving the native full-inside output updates.
    /// Example: `core.has_cylinder_overlap(1., 2., 3., transform, false, true, None)?`.
    #[allow(clippy::too_many_arguments)]
    pub fn has_cylinder_overlap(
        &self,
        bottom: f64,
        top: f64,
        height: f64,
        transform: Transform3,
        hollow: bool,
        is_ortho: bool,
        inside: Option<&mut bool>,
    ) -> Result<bool, Failure> {
        let query = CylinderQuery {
            radii: (bottom, top),
            height,
            transform,
            hollow,
        };
        let (overlap, enclosed) =
            self.cylinder_relation(query, &self.outline_vertices(is_ortho))?;
        if let Some(flag) = inside {
            *flag &= enclosed;
        }
        Ok(overlap)
    }
}
