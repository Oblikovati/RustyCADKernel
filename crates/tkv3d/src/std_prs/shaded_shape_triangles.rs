//! Native triangle assembly preserves located normals, reversed winding and UV normalization.
use rustycad_tkbrep::{
    brep::BRepTool,
    topexp::Explorer,
    topods::{self, Face, Shape},
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::{
    gp::Point2,
    poly::{Triangulation, mesh_purpose},
    toploc::Location,
};
use rustycad_tkservice::graphic3d::*;

#[derive(Clone, Copy)]
pub(super) struct ShadedTexelParameters {
    pub enabled: bool,
    pub origin: Point2,
    pub repeat: Point2,
    pub scale: Point2,
}
pub(super) struct ShadedFaceMesh {
    pub face: Face,
    pub mesh: Handle<Triangulation>,
    pub location: Location,
}
/// Native triangle filler family; `StdPrsShadedShape::fill_triangles(&shape)?` retains mesh normals.
pub struct StdPrsShadedShape;
impl StdPrsShadedShape {
    /// Fill already-triangulated topology without texture coordinates; `Self::fill_triangles(&shape)?`.
    pub fn fill_triangles(shape: &Shape) -> Result<Handle<Graphic3dArrayOfTriangles>, Failure> {
        let dummy = Point2::default();
        Self::fill_triangles_with_texels(shape, false, dummy, dummy, dummy)
    }
    /// Apply native per-face UV origin/repeat/scale; `Self::fill_triangles_with_texels(&shape,true,origin,repeat,scale)?`.
    pub fn fill_triangles_with_texels(
        shape: &Shape,
        enabled: bool,
        origin: Point2,
        repeat: Point2,
        scale: Point2,
    ) -> Result<Handle<Graphic3dArrayOfTriangles>, Failure> {
        let meshes = shaded_face_meshes(shape)?;
        let texels = ShadedTexelParameters {
            enabled,
            origin,
            repeat,
            scale,
        };
        fill_shaded_meshes(&meshes, texels)
    }
}
fn shaded_face_meshes(shape: &Shape) -> Result<Vec<ShadedFaceMesh>, Failure> {
    let mut meshes = Vec::new();
    for entry in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)? {
        let face = topods::face(&entry)?;
        let (mesh, location) = BRepTool::triangulation(&face, mesh_purpose::NONE)?;
        if !mesh.is_null() {
            meshes.push(ShadedFaceMesh {
                face,
                mesh,
                location,
            });
        }
    }
    Ok(meshes)
}
fn shaded_mesh_capacity(meshes: &[ShadedFaceMesh]) -> (i32, i32) {
    meshes.iter().fold((0, 0), |(vertices, triangles), face| {
        (
            vertices + face.mesh.nb_nodes(),
            triangles + face.mesh.nb_triangles(),
        )
    })
}
fn append_shaded_face(
    array: &Handle<Graphic3dArrayOfTriangles>,
    face: &ShadedFaceMesh,
    texels: ShadedTexelParameters,
) -> Result<(), Failure> {
    rustycad_tktopalgo::brep_lib::BRepLibToolTriangulatedShape::compute_normals(
        &face.face, &face.mesh,
    )?;
    let offset = array.vertex_number();
    super::shaded_shape_vertices::append_shaded_vertices(array, face, texels)?;
    super::shaded_shape_indices::append_shaded_indices(array, face, offset)
}

fn fill_shaded_meshes(
    meshes: &[ShadedFaceMesh],
    texels: ShadedTexelParameters,
) -> Result<Handle<Graphic3dArrayOfTriangles>, Failure> {
    let array = allocated_shaded_triangles(meshes, texels.enabled)?;
    if array.is_null() {
        return Ok(array);
    }
    for face in meshes {
        if !face.mesh.has_geometry() {
            continue;
        }
        append_shaded_face(&array, face, texels)?;
    }
    Ok(array)
}

fn allocated_shaded_triangles(
    meshes: &[ShadedFaceMesh],
    enabled: bool,
) -> Result<Handle<Graphic3dArrayOfTriangles>, Failure> {
    let (vertices, triangles) = shaded_mesh_capacity(meshes);
    if vertices < 3 || triangles <= 0 {
        return Ok(Handle::default());
    }
    let array = Handle::new(Graphic3dArrayOfTriangles::with_options(
        vertices,
        3 * triangles,
        true,
        false,
        enabled,
    )?);
    Ok(array)
}
