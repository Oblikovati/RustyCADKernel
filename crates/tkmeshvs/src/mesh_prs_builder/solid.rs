// SPDX-License-Identifier: GPL-2.0-only
//! `MeshVS_MeshPrsBuilder::AddFaceSolidPrs`: the fan triangles of a face, with the nodal
//! normals of the data source (smooth shading) or the face normal.
use super::MeshVSMeshPrsBuilderSource;
use super::primitives::{face_points, fan_index};
use crate::data_source::MeshVSDataSource;
use crate::mesh::MeshVSMeshMembers;
use crate::prs_builder::MeshVSPrsBuilderMembers;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkservice::graphic3d::Graphic3dArrayOfTriangles;

/// The shading options of `AddFaceSolidPrs`: `theIsShaded`, `theIsShrinked`,
/// `theShrinkingCoef` and `theIsSmoothShading`.
#[derive(Clone, Copy, Debug)]
pub struct MeshVSFaceSolidOptions {
    /// Vertex normals are written.
    pub is_shaded: bool,
    /// The face is shrunk around its center.
    pub is_shrinked: bool,
    /// The shrink coefficient.
    pub shrink_coef: f64,
    /// The nodal normals of the data source are used when they all exist.
    pub is_smooth_shading: bool,
}

/// The native protected `MeshVS_MeshPrsBuilder::AddFaceSolidPrs` body (`theID`, `theCoords`,
/// `theNbNodes`, `theMaxNodes`, `theTriangles` and the options); natively it reads the data
/// source of the parent mesh (not the builder's own one) and draws nothing without it.
/// Example: `meshvs_mesh_prs_add_face_solid_prs(&*builder, 101, &coords, 3, 10, &triangles, options)?`.
pub fn meshvs_mesh_prs_add_face_solid_prs<B: MeshVSMeshPrsBuilderSource + ?Sized>(
    builder: &B,
    id: i32,
    coords: &Array1<'_, f64>,
    nb_nodes: i32,
    max_nodes: i32,
    triangles: &Graphic3dArrayOfTriangles,
    options: MeshVSFaceSolidOptions,
) -> Result<(), Failure> {
    let parent = builder.parent_mesh();
    let source = parent
        .get()
        .map(|mesh| mesh.get_data_source())
        .unwrap_or_default();
    let Some(source) = source.get() else {
        return Ok(());
    };
    let mut normal = [0.0; 3];
    let mut vertex_normals = Vec::new();
    if options.is_shaded {
        vertex_normals = face_normals(
            source,
            (id, nb_nodes, max_nodes),
            options.is_smooth_shading,
            &mut normal,
        );
    }
    let smooth = options.is_smooth_shading && vertex_normals.len() as i32 == nb_nodes;
    let points = face_points(coords, nb_nodes, options.is_shrinked, options.shrink_coef)?;
    for node in 0..nb_nodes - 2 {
        for sub in 0..3 {
            let index = fan_index(node, sub) as usize;
            let p = points[index];
            if !options.is_shaded {
                triangles.add_vertex_f64(p[0], p[1], p[2])?;
                continue;
            }
            if smooth {
                normal = vertex_normals[index];
            }
            triangles.add_vertex_normal_f64(p, normal)?;
        }
    }
    Ok(())
}

/// The nodal normals up to the first missing one (smooth shading), and the face normal in
/// `normal` unless every nodal normal exists; the outputs are written in place, as natively.
fn face_normals(
    source: &dyn MeshVSDataSource,
    (id, nb_nodes, max_nodes): (i32, i32, i32),
    is_smooth: bool,
    normal: &mut [f64; 3],
) -> Vec<[f64; 3]> {
    let mut vertex_normals = Vec::new();
    if is_smooth {
        for node in 1..=nb_nodes {
            let [x, y, z] = &mut *normal;
            if !source.get_node_normal(node, id, x, y, z) {
                break;
            }
            vertex_normals.push(*normal);
        }
    }
    if !is_smooth || vertex_normals.len() as i32 != nb_nodes {
        let [x, y, z] = &mut *normal;
        source.get_normal(id, max_nodes, x, y, z);
    }
    vertex_normals
}
