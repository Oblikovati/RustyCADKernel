//! Native corners: float-buffer winding for planar facets and circular fans for rounded corners.
use super::{AisViewCubeGeometry, AisViewCubePresentationSource, AisViewCubeTriangleCounts};
use crate::v3d::{V3d, V3dTypeOfOrientation};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Direction3, Point3};
use rustycad_tkservice::graphic3d::Graphic3dArrayOfTriangles;
use std::f64::consts::{FRAC_PI_4, PI};
/// Append a box corner or count capacity for a null array.
/// Example: `ais_view_cube_create_box_corner_triangles(&cube,&array,&mut counts,orientation)?`.
pub fn ais_view_cube_create_box_corner_triangles<O: AisViewCubePresentationSource + ?Sized>(
    object: &O,
    array: &Handle<Graphic3dArrayOfTriangles>,
    counts: &mut AisViewCubeTriangleCounts,
    orientation: V3dTypeOfOrientation,
) -> Result<(), Failure> {
    let base = object.view_cube_presentation_base();
    let geometry = base.settings.read().geometry.clone();
    let direction = V3d::get_proj_axis(orientation);
    let (vertices, triangles) = if geometry.round_radius > 0.0 {
        (21, 21)
    } else {
        (3, 1)
    };
    counts.vertices += vertices;
    counts.triangles += triangles;
    if array.is_null() {
        return Ok(());
    }
    let first = array.vertex_number();
    if geometry.round_radius > 0.0 {
        append_cube_round_corner(array, &geometry, direction)?;
    } else {
        append_cube_flat_corner(array, &geometry, direction)?;
    }
    set_cube_vertex_normals(array, first, &direction)
}
pub(super) fn set_cube_vertex_normals(
    array: &Graphic3dArrayOfTriangles,
    first: i32,
    normal: &Direction3,
) -> Result<(), Failure> {
    for index in first + 1..=array.vertex_number() {
        array.set_vertex_normal_dir(index, normal)?;
    }
    Ok(())
}
fn append_cube_flat_corner(
    array: &Graphic3dArrayOfTriangles,
    geometry: &AisViewCubeGeometry,
    direction: Direction3,
) -> Result<(), Failure> {
    let first = array.vertex_number();
    let origin = direction.xyz() * (geometry.size * 0.5 * 3.0_f64.sqrt());
    for axis in 0..3 {
        let mut point = origin;
        point.coordinates_mut()[axis] +=
            geometry.facet_extension * direction.coordinates()[axis].signum();
        array.add_vertex_pnt(&super::view_cube_geometry::cube_point(point))?;
    }
    append_cube_corner_winding(array, first, direction)
}
fn append_cube_corner_winding(
    array: &Graphic3dArrayOfTriangles,
    first: i32,
    direction: Direction3,
) -> Result<(), Failure> {
    // Native winding reads the narrowed Graphic3d vertices, preserving float rounding.
    let [a, b, c] = [
        array.vertice(first + 1)?,
        array.vertice(first + 2)?,
        array.vertice(first + 3)?,
    ];
    let normal = (b.xyz() - a.xyz()).crossed(c.xyz() - a.xyz());
    let (second, third) = if normal.dot(direction.xyz()) < 0.0 {
        (first + 3, first + 2)
    } else {
        (first + 2, first + 3)
    };
    array.add_triangle_edges(first + 1, second, third)?;
    Ok(())
}

fn append_cube_round_corner(
    array: &Graphic3dArrayOfTriangles,
    geometry: &AisViewCubeGeometry,
    direction: Direction3,
) -> Result<(), Failure> {
    let half_width = geometry.facet_extension * 2.0_f64.sqrt() * 0.5;
    let height = half_width * (2.0_f64 / 3.0).sqrt();
    let distance = geometry.size * 0.5 * 3.0_f64.sqrt() + height;
    let transform = super::view_cube_geometry::cube_direction_transform(direction, distance)?;
    let radius = super::view_cube_geometry::cube_native_max(
        geometry.facet_extension * 0.5 / FRAC_PI_4.cos(),
        geometry.corner_min_size,
    );
    let first = array.vertex_number();
    array.add_vertex_pnt(&Point3::default().transformed(transform))?;
    for index in 0..20 {
        let angle = (1.0 - f64::from(index) / 20.0) * (2.0 * PI);
        array.add_vertex_pnt(
            &Point3::new(radius * angle.cos(), radius * angle.sin(), 0.0).transformed(transform),
        )?;
    }
    array.add_triangle_fan_edges(first + 1, array.vertex_number(), true)?;
    Ok(())
}
