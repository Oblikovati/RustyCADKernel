//! Virtual part dispatch and native side/edge frame construction.
use super::{
    AisViewCubePresentationSource, AisViewCubeTriangleCounts, ais_view_cube_is_box_corner,
    ais_view_cube_is_box_edge, ais_view_cube_is_box_side, ais_view_cube_round_rectangle_triangles,
};
use crate::v3d::{V3d, V3dTypeOfOrientation};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Axis2, Axis3, Direction3, Point2, Point3, Transform3, Xyz};
use rustycad_tkservice::graphic3d::Graphic3dArrayOfTriangles;
/// Dispatch virtual part geometry or count capacity for a null array.
/// Example: `ais_view_cube_create_box_part_triangles(&cube,&array,&mut counts,orientation)?`.
pub fn ais_view_cube_create_box_part_triangles<O: AisViewCubePresentationSource + ?Sized>(
    object: &O,
    array: &Handle<Graphic3dArrayOfTriangles>,
    counts: &mut AisViewCubeTriangleCounts,
    orientation: V3dTypeOfOrientation,
) -> Result<(), Failure> {
    if ais_view_cube_is_box_side(orientation) {
        return object.create_box_side_triangles(array, counts, orientation);
    }
    let settings = object.view_cube_presentation_base().settings.read();
    if ais_view_cube_is_box_edge(orientation) && settings.draw_edges {
        drop(settings);
        return object.create_box_edge_triangles(array, counts, orientation);
    }
    if ais_view_cube_is_box_corner(orientation) && settings.draw_corners {
        drop(settings);
        return object.create_box_corner_triangles(array, counts, orientation);
    }
    Ok(())
}
pub(super) fn cube_direction_transform(
    direction: Direction3,
    distance: f64,
) -> Result<Transform3, Failure> {
    let position = cube_point(direction.xyz() * distance);
    let frame = Axis2::new(position, direction.reversed())?;
    let mut transform = Transform3::default();
    transform.set_transformation_between(Axis3::from_axis2(frame), Axis3::default());
    Ok(transform)
}
/// Append a box face or count capacity for a null array.
/// Example: `ais_view_cube_create_box_side_triangles(&cube,&array,&mut counts,orientation)?`.
pub fn ais_view_cube_create_box_side_triangles<O: AisViewCubePresentationSource + ?Sized>(
    object: &O,
    array: &Handle<Graphic3dArrayOfTriangles>,
    counts: &mut AisViewCubeTriangleCounts,
    orientation: V3dTypeOfOrientation,
) -> Result<(), Failure> {
    let geometry = object
        .view_cube_presentation_base()
        .settings
        .read()
        .geometry
        .clone();
    let transform = cube_direction_transform(
        V3d::get_proj_axis(orientation),
        geometry.size * 0.5 + geometry.facet_extension,
    )?;
    ais_view_cube_round_rectangle_triangles(
        array,
        counts,
        Point2::new(geometry.size, geometry.size),
        geometry.round_radius * geometry.size,
        &transform,
    )
}
/// Append a box edge or count capacity for a null array.
/// Example: `ais_view_cube_create_box_edge_triangles(&cube,&array,&mut counts,orientation)?`.
pub fn ais_view_cube_create_box_edge_triangles<O: AisViewCubePresentationSource + ?Sized>(
    object: &O,
    array: &Handle<Graphic3dArrayOfTriangles>,
    counts: &mut AisViewCubeTriangleCounts,
    orientation: V3dTypeOfOrientation,
) -> Result<(), Failure> {
    let geometry = object
        .view_cube_presentation_base()
        .settings
        .read()
        .geometry
        .clone();
    let width = cube_native_max(
        geometry.facet_extension * 2.0_f64.sqrt() - geometry.edge_gap,
        geometry.edge_min_size,
    );
    let distance = geometry.size * 0.5 * 2.0_f64.sqrt()
        + geometry.facet_extension * std::f64::consts::FRAC_PI_4.cos();
    let transform = cube_direction_transform(V3d::get_proj_axis(orientation), distance)?;
    ais_view_cube_round_rectangle_triangles(
        array,
        counts,
        Point2::new(width, geometry.size),
        geometry.round_radius * geometry.size,
        &transform,
    )
}

pub(super) fn cube_point(xyz: Xyz) -> Point3 {
    Point3::new(xyz.x(), xyz.y(), xyz.z())
}

pub(super) fn cube_native_max(first: f64, second: f64) -> f64 {
    if first < second { second } else { first }
}
