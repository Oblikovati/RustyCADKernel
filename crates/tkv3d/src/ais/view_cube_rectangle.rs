//! Native rounded rectangles retain fan order, capacity overestimates and outward normals.
use super::AisViewCubeTriangleCounts;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Direction3, Point2, Point3, Transform3};
use rustycad_tkservice::graphic3d::Graphic3dArrayOfTriangles;
use std::f64::consts::PI;
/// Native rectangle factory: null array counts capacity; otherwise appends geometry.
/// Example: `ais_view_cube_round_rectangle_triangles(&array,&mut counts,size,radius,&transform)?`.
pub fn ais_view_cube_round_rectangle_triangles(
    array: &Handle<Graphic3dArrayOfTriangles>,
    counts: &mut AisViewCubeTriangleCounts,
    size: Point2,
    radius: f64,
    transform: &Transform3,
) -> Result<(), Failure> {
    let radius = cube_native_min(radius, cube_native_min(size.x(), size.y()) * 0.5);
    let half = Point2::new(size.x() * 0.5 - radius, size.y() * 0.5 - radius);
    let (vertices, triangles) = if radius > 0.0 { (37, 37) } else { (4, 2) };
    counts.vertices += vertices;
    counts.triangles += triangles;
    if array.is_null() {
        return Ok(());
    }
    let first = array.vertex_number();
    if radius > 0.0 {
        append_cube_round_rectangle(array, half, radius, transform)?;
    } else {
        append_cube_square_rectangle(array, half, transform)?;
    }
    let normal = Direction3::Z.transformed(*transform).reversed();
    super::view_cube_corners::set_cube_vertex_normals(array, first, &normal)
}
fn cube_native_min(first: f64, second: f64) -> f64 {
    if second < first { second } else { first }
}
fn append_cube_round_rectangle(
    array: &Graphic3dArrayOfTriangles,
    half: Point2,
    radius: f64,
    transform: &Transform3,
) -> Result<(), Failure> {
    let first = array.vertex_number();
    array.add_vertex_pnt(&Point3::default().transformed(*transform))?;
    for quadrant in 0..4 {
        append_cube_rectangle_arc(array, half, radius, transform, quadrant)?;
    }
    array.add_triangle_fan_edges(first + 1, array.vertex_number(), true)?;
    Ok(())
}
fn append_cube_rectangle_arc(
    array: &Graphic3dArrayOfTriangles,
    half: Point2,
    radius: f64,
    transform: &Transform3,
    quadrant: i32,
) -> Result<(), Failure> {
    let from = PI * 0.5 * (1 - quadrant) as f64;
    let to = PI * 0.5 * (-quadrant) as f64;
    let x = if quadrant < 2 { half.x() } else { -half.x() };
    let y = if quadrant == 0 || quadrant == 3 {
        half.y()
    } else {
        -half.y()
    };
    for index in 0..=8 {
        let t = f64::from(index) / 8.0;
        let angle = (1.0 - t) * from + t * to;
        let point = Point3::new(x + radius * angle.cos(), y + radius * angle.sin(), 0.0);
        array.add_vertex_pnt(&point.transformed(*transform))?;
    }
    Ok(())
}
fn append_cube_square_rectangle(
    array: &Graphic3dArrayOfTriangles,
    half: Point2,
    transform: &Transform3,
) -> Result<(), Failure> {
    let first = array.vertex_number();
    for (x, y) in [
        (-half.x(), -half.y()),
        (-half.x(), half.y()),
        (half.x(), half.y()),
        (half.x(), -half.y()),
    ] {
        array.add_vertex_pnt(&Point3::new(x, y, 0.0).transformed(*transform))?;
    }
    array.add_quad_triangle_edges(first + 1, first + 2, first + 3, first + 4)?;
    Ok(())
}
