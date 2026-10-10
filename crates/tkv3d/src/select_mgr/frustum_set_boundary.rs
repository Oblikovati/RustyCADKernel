//! Outer polyline side faces and native finite segment predicates.
use super::SelectMgrTriangularFrustumSet;
use rustycad_tkernel::{ncollection::Array1, precision::confusion, standard::Failure};
use rustycad_tkmath::gp::{Point3, Xyz, resolution};

pub(super) fn frustum_set_face(
    boundary: &Array1<'_, Point3>,
    index: i32,
) -> Result<[Point3; 4], Failure> {
    let count = boundary.len() as i32 / 2;
    let indices = [
        index,
        index + count,
        index % count + 1 + count,
        index % count + 1,
    ];
    let mut face = [Point3::default(); 4];
    for (target, index) in face.iter_mut().zip(indices) {
        *target = *boundary
            .value(index)
            .map_err(|e| e.into_failure("TriangularFrustumSet boundary face"))?;
    }
    Ok(face)
}
pub(super) fn frustum_set_point_in_triangle(point: Point3, triangle: [Point3; 3]) -> bool {
    let [a, b, c] = triangle.map(|vertex| vertex.xyz() - point.xyz());
    let (u, v, w) = (b.crossed(c), c.crossed(a), a.crossed(b));
    u.dot(v) >= 0. && u.dot(w) >= 0.
}
pub(super) fn frustum_set_segments_cross(first: [Point3; 2], last: [Point3; 2]) -> bool {
    let a = first[1].xyz() - first[0].xyz();
    let b = last[1].xyz() - last[0].xyz();
    let ab = last[0].xyz() - first[0].xyz();
    let ba = first[0].xyz() - last[0].xyz();
    if ab.dot_cross(a, b).abs() > confusion() || ba.dot_cross(b, a).abs() > confusion() {
        return false;
    }
    let first = ab.crossed(b).dot(a.crossed(b)) / a.crossed(b).square_modulus();
    let last = ba.crossed(a).dot(b.crossed(a)) / b.crossed(a).square_modulus();
    (0. ..=1.).contains(&first) && (0. ..=1.).contains(&last)
}
fn frustum_set_segment_triangle(origin: Point3, direction: Xyz, triangle: [Point3; 3]) -> bool {
    let edge1 = triangle[1].xyz() - triangle[0].xyz();
    let edge2 = triangle[2].xyz() - triangle[0].xyz();
    let p = direction.crossed(edge2);
    let determinant = edge1.dot(p);
    if determinant.abs() < resolution() {
        return false;
    }
    let inverse = 1. / determinant;
    let offset = origin.xyz() - triangle[0].xyz();
    let u = inverse * offset.dot(p);
    if u < 0. || u > 1. {
        return false;
    }
    let q = offset.crossed(edge1);
    let v = inverse * direction.dot(q);
    if v < 0. || u + v > 1. {
        return false;
    }
    (0. ..=1.).contains(&(inverse * edge2.dot(q)))
}
impl SelectMgrTriangularFrustumSet {
    pub(super) fn frustum_set_crosses_segment(
        &self,
        first: Point3,
        last: Point3,
    ) -> Result<bool, Failure> {
        let boundary = self.boundary_points.read();
        let count = boundary.len() as i32 / 2;
        for index in boundary.lower()..boundary.lower() + count {
            let face = frustum_set_face(&boundary, index)?;
            let direction = last.xyz() - first.xyz();
            if frustum_set_segment_triangle(first, direction, [face[0], face[1], face[2]])
                || frustum_set_segment_triangle(first, direction, [face[0], face[2], face[3]])
            {
                return Ok(true);
            }
        }
        Ok(false)
    }
    pub(super) fn frustum_set_accepts_edges(&self, points: &[Point3]) -> Result<bool, Failure> {
        if *self.allow_overlap.read() {
            return Ok(true);
        }
        for (index, first) in points.iter().enumerate() {
            if self.frustum_set_crosses_segment(*first, points[(index + 1) % points.len()])? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
