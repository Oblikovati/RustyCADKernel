//! Native sphere projection and boundary containment, including the final far successor.
use super::*;
use rustycad_tkernel::{ncollection::Array1, precision::confusion, standard::Failure};
use rustycad_tkmath::gp::{Direction3, Point3, Vector3};
impl SelectMgrTriangularFrustumSet {
    pub(super) fn frustum_set_sphere_overlap(
        &self,
        center: Point3,
        radius: f64,
    ) -> Result<bool, Failure> {
        if !self.frustum_set_any(|child| child.overlaps_sphere(&center, radius, None))? {
            return Ok(false);
        }
        self.frustum_set_sphere_boundary(center, radius)
    }
    fn frustum_set_sphere_boundary(&self, center: Point3, radius: f64) -> Result<bool, Failure> {
        let boundary = self.boundary_points.read();
        let normal = frustum_set_sphere_normal(&boundary)?;
        let projection = Point3::from(center.xyz() - normal.xyz() * center.xyz().dot(normal.xyz()));
        let (angle, points) = frustum_set_sphere_angles(&boundary, projection, normal)?;
        let center_inside = (angle - 2. * std::f64::consts::PI).abs() < confusion();
        let mut boundary_inside = false;
        let crosses = self.is_boundary_intersect_sphere(
            &projection,
            radius,
            &normal,
            points.as_slice(),
            &mut boundary_inside,
        )?;
        Ok(if *self.allow_overlap.read() {
            crosses || center_inside
        } else {
            !crosses && center_inside && !boundary_inside
        })
    }
}
#[derive(Default)]
struct SpherePlaneCandidates {
    second: usize,
    third: usize,
    max_first: f64,
    max_second: f64,
}
impl SpherePlaneCandidates {
    fn consider(&mut self, points: &[Point3], index: usize) {
        let distance = points[0].distance(&points[index]);
        if distance < confusion() {
            return;
        }
        if self.max_first < distance {
            self.replace_farthest(index, distance);
            return;
        }
        let other_distance = points[self.second].distance(&points[index]);
        if self.max_second < other_distance {
            self.third = index;
            self.max_second = other_distance;
        }
    }
    fn replace_farthest(&mut self, index: usize, distance: f64) {
        if self.second != self.third {
            self.third = self.second;
            self.max_second = self.max_first;
        }
        self.second = index;
        self.max_first = distance;
    }
}
fn frustum_set_sphere_normal(boundary: &Array1<'_, Point3>) -> Result<Direction3, Failure> {
    let origin = *boundary
        .value(boundary.lower())
        .map_err(|e| e.into_failure("frustum sphere plane"))?;
    let points = boundary.as_slice();
    let mut candidates = SpherePlaneCandidates::default();
    for index in 0..points.len() / 2 {
        candidates.consider(points, index);
    }
    let a = Vector3::between(origin, points[candidates.second]);
    let b = Vector3::between(origin, points[candidates.third]);
    Direction3::from_vector(a.crossed(b))
}
fn frustum_set_sphere_angles(
    boundary: &Array1<'_, Point3>,
    center: Point3,
    normal: Direction3,
) -> Result<(f64, Array1<'static, Point3>), Failure> {
    let upper = boundary.lower() + boundary.len() as i32 / 2;
    let mut points = Array1::with_bounds(boundary.lower(), upper)
        .map_err(|e| e.into_failure("frustum sphere boundary"))?;
    let mut angle = 0.;
    for index in boundary.lower()..upper {
        let first = *boundary
            .value(index)
            .map_err(|e| e.into_failure("frustum sphere angle start"))?;
        let last = *boundary
            .value(index + 1)
            .map_err(|e| e.into_failure("frustum sphere angle finish"))?;
        points
            .set_value(index, first)
            .map_err(|e| e.into_failure("frustum sphere boundary"))?;
        let a = Point3::from(first.xyz() - normal.xyz() * first.xyz().dot(normal.xyz()));
        let b = Point3::from(last.xyz() - normal.xyz() * last.xyz().dot(normal.xyz()));
        angle += Vector3::between(center, a).angle(Vector3::between(center, b))?;
    }
    Ok((angle, points))
}
