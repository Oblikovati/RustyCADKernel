//! Real native box-part arrays, optional side outlines and group submission order.
use super::{AisViewCubePresentationSource, AisViewCubeTriangleCounts};
use crate::prs_mgr::PrsMgrPresentableObjectMembers;
use crate::v3d::V3dTypeOfOrientation as Orientation;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkservice::graphic3d::*;
pub(super) fn cube_part_capacity<O: AisViewCubePresentationSource + ?Sized>(
    object: &O,
    orientations: &[Orientation],
) -> Result<AisViewCubeTriangleCounts, Failure> {
    let mut counts = AisViewCubeTriangleCounts::default();
    for &orientation in orientations {
        object.create_box_part_triangles(&Handle::default(), &mut counts, orientation)?;
    }
    Ok(counts)
}
pub(super) fn submit_cube_parts<O: AisViewCubePresentationSource + ?Sized>(
    object: &O,
    prs: &Handle<dyn Graphic3dStructureSource>,
    orientations: &[Orientation],
    aspect: &Handle<Graphic3dAspectFillArea3d>,
    outlined: bool,
) -> Result<(), Failure> {
    let counts = cube_part_capacity(object, orientations)?;
    if counts.vertices <= 0 {
        return Ok(());
    }
    let triangles = Handle::new(Graphic3dArrayOfTriangles::new(
        counts.vertices,
        counts.triangles * 3,
        GRAPHIC3D_ARRAY_FLAGS_VERTEX_NORMAL,
    )?);
    let outlines = cube_side_outlines(counts.vertices, outlined)?;
    append_cube_parts(object, &triangles, &outlines, orientations)?;
    submit_cube_triangles(prs, &triangles, aspect)?;
    submit_cube_boundary(object, prs, &outlines)?;
    Ok(())
}
fn submit_cube_boundary<O: AisViewCubePresentationSource + ?Sized>(
    object: &O,
    prs: &Handle<dyn Graphic3dStructureSource>,
    outlines: &Handle<Graphic3dArrayOfSegments>,
) -> Result<(), Failure> {
    if outlines.is_null() {
        return Ok(());
    }
    let boundary = object.attributes().face_boundary_aspect();
    if boundary.is_null() {
        return Err(rustycad_tkernel::standard::exceptions::null_object(
            "AIS_ViewCube face-boundary aspect=null; expected own aspect or interactive-context drawer link",
        ));
    }
    let group = prs.new_group()?;
    group.set_group_primitives_aspect(&boundary.aspect().cast());
    group.add_primitive_array(&outlines.cast(), true);
    Ok(())
}

fn cube_side_outlines(
    vertices: i32,
    outlined: bool,
) -> Result<Handle<Graphic3dArrayOfSegments>, Failure> {
    if !outlined {
        return Ok(Handle::default());
    }
    Ok(Handle::new(Graphic3dArrayOfSegments::new(
        vertices,
        vertices * 2,
        0,
    )?))
}
fn append_cube_parts<O: AisViewCubePresentationSource + ?Sized>(
    object: &O,
    triangles: &Handle<Graphic3dArrayOfTriangles>,
    outlines: &Handle<Graphic3dArrayOfSegments>,
    orientations: &[Orientation],
) -> Result<(), Failure> {
    let mut counts = AisViewCubeTriangleCounts::default();
    for &orientation in orientations {
        let from_vertex = triangles.vertex_number();
        let from_triangle = counts.triangles;
        object.create_box_part_triangles(triangles, &mut counts, orientation)?;
        if !outlines.is_null() {
            append_cube_outline(
                triangles,
                outlines,
                from_vertex,
                counts.triangles - from_triangle,
            )?;
        }
    }
    Ok(())
}
fn append_cube_outline(
    triangles: &Graphic3dArrayOfTriangles,
    outlines: &Graphic3dArrayOfSegments,
    first: i32,
    triangle_count: i32,
) -> Result<(), Failure> {
    let first_outline = outlines.vertex_number();
    // OCCT 3d097a0 skips the fan center only when its reported triangle count exceeds two.
    let start = first + if triangle_count > 2 { 2 } else { 1 };
    for index in start..=triangles.vertex_number() {
        outlines.add_vertex_pnt(&triangles.vertice(index)?)?;
    }
    outlines.add_polyline_edges(first_outline + 1, outlines.vertex_number(), true)?;
    Ok(())
}
pub(super) fn submit_cube_triangles(
    prs: &Handle<dyn Graphic3dStructureSource>,
    triangles: &Handle<Graphic3dArrayOfTriangles>,
    aspect: &Handle<Graphic3dAspectFillArea3d>,
) -> Result<(), Failure> {
    let group = prs.new_group()?;
    group.set_closed(true);
    group.set_group_primitives_aspect(&aspect.cast());
    group.add_primitive_array(&triangles.cast(), true);
    Ok(())
}
