//! Native boundary/sphere test, including virtual ray dispatch and untouched output flags.
use super::SelectMgrBaseIntersector;
use rustycad_tkernel::precision::confusion;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Direction3, Point3};

fn project_boundary_point(point: &Point3, normal: &Direction3) -> Point3 {
    let coordinates = point.xyz();
    Point3::from(coordinates - normal.xyz() * coordinates.dot(normal.xyz()))
}

fn segment_crosses_sphere<T: SelectMgrBaseIntersector + ?Sized>(
    volume: &T,
    center: &Point3,
    radius: f64,
    first: &Point3,
    last: &Point3,
) -> Result<bool, Failure> {
    let delta = last.xyz() - first.xyz();
    let ray = Direction3::new(delta.x(), delta.y(), delta.z())?;
    let Some((enter, leave)) = volume.ray_sphere_intersection(center, radius, first, &ray) else {
        return Ok(false);
    };
    let length = first.distance(last);
    Ok((enter > 0.0 && enter < length) || (leave > 0.0 && leave < length))
}

fn projected_edge_overlap<T: SelectMgrBaseIntersector + ?Sized>(
    volume: &T,
    center: &Point3,
    radius: f64,
    normal: &Direction3,
    edge: (&Point3, &Point3),
    inside: &mut bool,
) -> Result<bool, Failure> {
    if edge.0.distance(edge.1) < confusion() {
        return Ok(false);
    }
    let first = project_boundary_point(edge.0, normal);
    let last = project_boundary_point(edge.1, normal);
    if first.distance(center) < radius || last.distance(center) < radius {
        *inside = true;
        return Ok(true);
    }
    segment_crosses_sphere(volume, center, radius, &first, &last)
}

/// Preserve OCCT's closed-boundary convention and virtual ray intersection.
/// Example: `select_mgr_boundary_intersect_sphere(&volume, &center, 1., &normal, &points, &mut inside)?`.
pub fn select_mgr_boundary_intersect_sphere<T: SelectMgrBaseIntersector + ?Sized>(
    volume: &T,
    center: &Point3,
    radius: f64,
    normal: &Direction3,
    points: &[Point3],
    inside: &mut bool,
) -> Result<bool, Failure> {
    // OCCT 3d097a0 ignores the final entry: callers supply a repeated closing point.
    let last_index = points.len().saturating_sub(1);
    for index in 0..last_index {
        let next = if index + 1 == last_index {
            0
        } else {
            index + 1
        };
        if projected_edge_overlap(
            volume,
            center,
            radius,
            normal,
            (&points[index], &points[next]),
            inside,
        )? {
            return Ok(true);
        }
    }
    Ok(false)
}
