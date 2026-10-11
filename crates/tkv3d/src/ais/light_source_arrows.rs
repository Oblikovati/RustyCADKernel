//! Native ambient and directional arrow arrays and their ordered shadow groups.
use super::light_source_geometry::{
    light_arrow_aspect, light_cone_frame, light_disabled_aspect, light_group, light_marker_aspect,
    light_point, light_shadow_aspect, light_sphere,
};
use super::{AisLightSourceMembers, AisLightSourceSource};
use crate::{prs_mgr::PrsMgrPresentableObjectMembers, prs3d::*};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::*;
use rustycad_tkservice::graphic3d::*;
type Presentation = Handle<dyn Graphic3dStructureSource>;
pub(super) fn offset_light_point(point: Point3, direction: Direction3, length: f64) -> Point3 {
    point.translated(Vector3::new(
        direction.x() * length,
        direction.y() * length,
        direction.z() * length,
    ))
}
fn arrow_tool<O: AisLightSourceSource + ?Sized>(object: &O, length: f64) -> Prs3dToolCylinder {
    Prs3dToolCylinder::new(
        length * 0.1,
        0.0,
        length * 0.2,
        object.nb_splits_arrow(),
        object.nb_splits_arrow(),
    )
}
fn arrow_triangles(
    tool: &Prs3dToolCylinder,
    count: i32,
) -> Result<Handle<Graphic3dArrayOfTriangles>, Failure> {
    Ok(Handle::new(Graphic3dArrayOfTriangles::new(
        count * tool.vertices_nb(true),
        count * tool.triangles_nb() * 3,
        GRAPHIC3D_ARRAY_FLAGS_VERTEX_NORMAL,
    )?))
}
pub(super) fn compute_light_ambient<O: AisLightSourceSource + ?Sized>(
    object: &O,
    prs: &Presentation,
    mode: i32,
) -> Result<(), Failure> {
    if mode == 0 {
        light_sphere(object, prs, object.size() * 0.25)?;
    }
    if mode == 0 || mode == 1 {
        ambient_arrows(object, prs)?;
    }
    let marker = if mode == 1 {
        light_marker_aspect(object)
    } else {
        light_disabled_aspect(object)
    };
    light_point(prs, marker, Point3::default())
}
fn ambient_arrows<O: AisLightSourceSource + ?Sized>(
    object: &O,
    prs: &Presentation,
) -> Result<(), Failure> {
    let tool = arrow_tool(object, object.size());
    let mut triangles = arrow_triangles(&tool, 6)?;
    let lines = Handle::new(Graphic3dArrayOfSegments::new(
        12,
        0,
        GRAPHIC3D_ARRAY_FLAGS_NONE,
    )?);
    fill_ambient_arrows(&tool, &mut triangles, &lines, object.size() * 0.25)?;
    light_group(prs, light_shadow_aspect(object), &lines.cast(), false)?;
    light_group(prs, light_arrow_aspect(object), &triangles.cast(), true)
}
fn fill_ambient_arrow(
    tool: &Prs3dToolCylinder,
    triangles: &mut Handle<Graphic3dArrayOfTriangles>,
    lines: &Graphic3dArrayOfSegments,
    direction: Direction3,
    length: f64,
) -> Result<(), Failure> {
    let point = offset_light_point(Point3::default(), direction, length);
    lines.add_vertex_pnt(&offset_light_point(point, direction, length * 0.5))?;
    lines.add_vertex_pnt(&offset_light_point(point, direction, length * 1.5))?;
    let frame = light_cone_frame(
        offset_light_point(point, direction, length),
        direction.reversed(),
    )?;
    tool.fill_array(triangles, &frame)
}
fn directional_count(requested: i32) -> i32 {
    match requested {
        9.. => 9,
        5..=8 => 5,
        3..=4 => 3,
        _ => 1,
    }
}
fn directional_positions<O: AisLightSourceSource + ?Sized>(
    object: &O,
) -> Result<Vec<Point3>, Failure> {
    let step = object.size() * 0.25;
    let frame = Axis2::new(Point3::default(), Direction3::Z.reversed())?;
    let offsets = &DIRECTIONAL_OFFSETS[..directional_count(object.nb_arrows()) as usize];
    Ok(offsets
        .iter()
        .map(|&offset| directional_position(&frame, step, offset))
        .collect())
}
fn directional_position(frame: &Axis2, step: f64, (dx, dy): (i32, i32)) -> Point3 {
    let origin = Point3::new(0.0, 0.0, step);
    let vertical = offset_light_point(origin, frame.y_direction(), step * f64::from(dy));
    offset_light_point(vertical, frame.x_direction(), step * f64::from(dx))
}

pub(super) fn compute_light_directional<O: AisLightSourceSource + ?Sized>(
    object: &O,
    prs: &Presentation,
    mode: i32,
) -> Result<(), Failure> {
    let arrays = directional_arrays(object, mode)?;
    directional_groups(object, prs, &arrays.0, &arrays.1, &arrays.2, mode)?;
    light_point(
        prs,
        light_disabled_aspect(object),
        Point3::new(0.0, 0.0, object.size() * 0.25),
    )
}
type DirectionalArrays = (
    Handle<Graphic3dArrayOfTriangles>,
    Handle<Graphic3dArrayOfPoints>,
    Handle<Graphic3dArrayOfSegments>,
);
fn directional_arrays<O: AisLightSourceSource + ?Sized>(
    object: &O,
    mode: i32,
) -> Result<DirectionalArrays, Failure> {
    let positions = directional_positions(object)?;
    let tool = arrow_tool(object, object.size() * 0.5);
    let triangles = if mode == 0 {
        arrow_triangles(&tool, positions.len() as i32)?
    } else {
        Handle::default()
    };
    let (points, lines) = directional_buffers(positions.len() as i32)?;
    let mut arrays = (triangles, points, lines);
    fill_directional_arrays(&tool, &mut arrays, &positions, object.size() * 0.5)?;
    Ok(arrays)
}
fn fill_directional_arrays(
    tool: &Prs3dToolCylinder,
    arrays: &mut DirectionalArrays,
    positions: &[Point3],
    length: f64,
) -> Result<(), Failure> {
    for &point in positions {
        fill_directional_arrow(tool, &mut arrays.0, &arrays.1, &arrays.2, point, length)?;
    }
    Ok(())
}

fn fill_directional_arrow(
    tool: &Prs3dToolCylinder,
    triangles: &mut Handle<Graphic3dArrayOfTriangles>,
    points: &Graphic3dArrayOfPoints,
    lines: &Graphic3dArrayOfSegments,
    point: Point3,
    length: f64,
) -> Result<(), Failure> {
    points.add_vertex_pnt(&point)?;
    lines.add_vertex_pnt(&point)?;
    let end = offset_light_point(point, Direction3::Z.reversed(), length);
    lines.add_vertex_pnt(&end)?;
    if !triangles.is_null() {
        tool.fill_array(triangles, &light_cone_frame(end, Direction3::Z.reversed())?)?;
    }
    Ok(())
}
fn directional_groups<O: AisLightSourceSource + ?Sized>(
    object: &O,
    prs: &Presentation,
    triangles: &Handle<Graphic3dArrayOfTriangles>,
    points: &Handle<Graphic3dArrayOfPoints>,
    lines: &Handle<Graphic3dArrayOfSegments>,
    mode: i32,
) -> Result<(), Failure> {
    let aspect = object.attributes().arrow_aspect().aspect();
    if mode == 0 {
        light_group(prs, light_shadow_aspect(object), &lines.cast(), false)?;
    }
    light_group(prs, aspect.cast(), &lines.cast(), false)?;
    if !triangles.is_null() {
        light_group(prs, aspect.cast(), &triangles.cast(), true)?;
    }
    light_group(prs, aspect.cast(), &points.cast(), false)
}

fn fill_ambient_arrows(
    tool: &Prs3dToolCylinder,
    triangles: &mut Handle<Graphic3dArrayOfTriangles>,
    lines: &Graphic3dArrayOfSegments,
    length: f64,
) -> Result<(), Failure> {
    for direction in [
        Direction3::X.reversed(),
        Direction3::X,
        Direction3::Y.reversed(),
        Direction3::Y,
        Direction3::Z.reversed(),
        Direction3::Z,
    ] {
        fill_ambient_arrow(tool, triangles, lines, direction, length)?;
    }
    Ok(())
}

const DIRECTIONAL_OFFSETS: [(i32, i32); 9] = [
    (0, 0),
    (1, 1),
    (-1, -1),
    (1, -1),
    (-1, 1),
    (0, 1),
    (1, 0),
    (0, -1),
    (-1, 0),
];

fn directional_buffers(
    count: i32,
) -> Result<
    (
        Handle<Graphic3dArrayOfPoints>,
        Handle<Graphic3dArrayOfSegments>,
    ),
    Failure,
> {
    let points = Handle::new(Graphic3dArrayOfPoints::new(
        count,
        GRAPHIC3D_ARRAY_FLAGS_NONE,
    )?);
    let lines = Handle::new(Graphic3dArrayOfSegments::new(
        count * 2,
        0,
        GRAPHIC3D_ARRAY_FLAGS_NONE,
    )?);
    Ok((points, lines))
}
