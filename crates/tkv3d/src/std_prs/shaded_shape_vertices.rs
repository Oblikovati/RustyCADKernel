//! Native face-local UV bounds and topology/mirror-aware normal transformation.
use super::shaded_shape_triangles::{ShadedFaceMesh, ShadedTexelParameters};
use rustycad_tkbrep::brep_tools::{self, UVBounds};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Direction3, Point2, Point3};
use rustycad_tkservice::graphic3d::*;

pub(super) fn append_shaded_vertices(
    array: &Handle<Graphic3dArrayOfTriangles>,
    face: &ShadedFaceMesh,
    texels: ShadedTexelParameters,
) -> Result<(), Failure> {
    let bounds = if texels.enabled {
        brep_tools::uv_bounds(&face.face)?
    } else {
        (0., 0., 0., 0.)
    };
    for index in 1..=face.mesh.nb_nodes() {
        append_shaded_node(array, face, texels, bounds, index)?;
    }
    Ok(())
}
fn located_shaded_vertex(
    face: &ShadedFaceMesh,
    index: i32,
) -> Result<(Point3, Direction3), Failure> {
    let transform = face.location.transformation();
    let mirrored = transform.vectorial_part().determinant() < 0.;
    let mut normal = face.mesh.normal(index)?;
    if (face.face.orientation() == Orientation::Reversed) ^ mirrored {
        normal.reverse();
    }
    let mut point = face.mesh.node(index)?;
    if !face.location.is_identity() {
        point.transform(transform);
        normal.transform(transform);
    }
    Ok((point, normal))
}
fn normalized_shaded_texel(
    node: Point2,
    (umin, umax, vmin, vmax): UVBounds,
    parameters: ShadedTexelParameters,
) -> Point2 {
    let du = umax - umin;
    let dv = vmax - vmin;
    if du == 0. || dv == 0. {
        return node;
    }
    Point2::new(
        (-parameters.origin.x() + parameters.repeat.x() * (node.x() - umin) / du)
            / parameters.scale.x(),
        (-parameters.origin.y() + parameters.repeat.y() * (node.y() - vmin) / dv)
            / parameters.scale.y(),
    )
}

fn append_shaded_node(
    array: &Graphic3dArrayOfTriangles,
    face: &ShadedFaceMesh,
    texels: ShadedTexelParameters,
    bounds: UVBounds,
    index: i32,
) -> Result<(), Failure> {
    let (point, normal) = located_shaded_vertex(face, index)?;
    if texels.enabled && face.mesh.has_uv_nodes() {
        let uv = normalized_shaded_texel(face.mesh.uv_node(index)?, bounds, texels);
        array.add_vertex_pnt_normal_texel(&point, &normal, &uv)?;
    } else {
        array.add_vertex_pnt_normal(&point, &normal)?;
    }
    Ok(())
}
