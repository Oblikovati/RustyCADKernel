//! Native planar predicates used by frustum/cylinder/circle overlap (OCCT 3d097a0).
use rustycad_tkernel::precision::{angular, confusion};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Direction3, Matrix3, Point3, Transform3, Vector3};

pub(crate) fn frustum_point_inside(point: Point3, vertices: &[Point3]) -> Result<bool, Failure> {
    let mut angle = 0.0;
    for index in 0..vertices.len() {
        let first = Vector3::between(point, vertices[index]);
        let last = Vector3::between(point, vertices[(index + 1) % vertices.len()]);
        angle += first.angle(last)?;
    }
    Ok((angle - 2.0 * std::f64::consts::PI).abs() < angular())
}

fn segment_axes(points: [Point3; 4]) -> ([f64; 4], [f64; 4]) {
    let mut first = points.map(|p| p.x());
    let mut second = points.map(|p| p.y());
    if first[0] == first[2] && first[1] == first[3] {
        first = points.map(|p| p.z());
    }
    if second[0] == second[2] && second[1] == second[3] {
        second = points.map(|p| p.z());
    }
    (first, second)
}

fn determinant_pair(first: (f64, f64), second: (f64, f64)) -> f64 {
    first.0 * second.1 - second.0 * first.1
}

pub(crate) fn frustum_segments_intersect(points: [Point3; 4]) -> bool {
    let rows = [1, 2, 3].map(|i| *(points[i].xyz() - points[0].xyz()).coordinates());
    if Matrix3::new(rows).determinant().abs() > confusion() {
        return false;
    }
    let (x, y) = segment_axes(points);
    let first = (x[0] - x[1], y[0] - y[1]);
    let second = (x[3] - x[2], y[3] - y[2]);
    let offset = (x[0] - x[2], y[0] - y[2]);
    let determinant = determinant_pair(first, second);
    if determinant == 0.0 {
        return false;
    }
    let u = determinant_pair(offset, second) / determinant;
    let v = determinant_pair(first, offset) / determinant;
    (0.0..=1.0).contains(&u) && (0.0..=1.0).contains(&v)
}

pub(crate) fn frustum_disk_projection(point: Point3, ray: Direction3, z: f64) -> (f64, f64) {
    let parameter = (z - point.z()) / ray.z();
    (
        point.x() + ray.x() * parameter,
        point.y() + ray.y() * parameter,
    )
}

fn projected_edge_hits_disk(radius: f64, first: (f64, f64), last: (f64, f64)) -> bool {
    let a = (first.0 - last.0) * (first.0 - last.0) + (first.1 - last.1) * (first.1 - last.1);
    let k = first.0 * (last.0 - first.0) + first.1 * (last.1 - first.1);
    let c = first.0 * first.0 + first.1 * first.1 - radius * radius;
    let discriminant = k * k - a * c;
    if discriminant
        .partial_cmp(&0.0)
        .is_none_or(|order| order.is_lt())
    {
        return false;
    }
    let first_root = (-k + discriminant.sqrt()) / a;
    let last_root = (-k - discriminant.sqrt()) / a;
    (0.0..=1.0).contains(&first_root) || (0.0..=1.0).contains(&last_root)
}

pub(crate) fn frustum_circle_intersection(
    radius: f64,
    center: Point3,
    transform: Transform3,
    vertices: &[Point3],
    edge: Vector3,
) -> Result<bool, Failure> {
    let inverse = transform.inverted()?;
    let ray = Direction3::from_vector(edge)?.transformed(inverse);
    if ray.z() == 0.0 {
        return Ok(false);
    }
    for index in 0..vertices.len() {
        let start = vertices[index].transformed(inverse);
        let finish = vertices[(index + 1) % vertices.len()].transformed(inverse);
        let first = frustum_disk_projection(start, ray, center.z());
        let last = frustum_disk_projection(finish, ray, center.z());
        if projected_edge_hits_disk(radius, first, last) {
            return Ok(true);
        }
    }
    Ok(false)
}

pub(crate) fn frustum_inside_end_faces(
    radii: (f64, f64),
    height: f64,
    transform: Transform3,
    vertices: &[Point3],
    edge: Vector3,
) -> Result<bool, Failure> {
    let inverse = transform.inverted()?;
    let ray = Direction3::from_vector(edge)?.transformed(inverse);
    if ray.z() == 0.0 {
        return Ok(false);
    }
    for vertex in vertices {
        let point = vertex.transformed(inverse);
        let (x1, y1) = frustum_disk_projection(point, ray, 0.0);
        let (x2, y2) = frustum_disk_projection(point, ray, height);
        if !(x1 * x1 + y1 * y1 <= radii.0 * radii.0 && x2 * x2 + y2 * y2 <= radii.1 * radii.1) {
            return Ok(false);
        }
    }
    Ok(true)
}
