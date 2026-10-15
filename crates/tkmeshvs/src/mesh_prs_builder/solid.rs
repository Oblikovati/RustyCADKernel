// SPDX-License-Identifier: GPL-2.0-only
//! `MeshVS_MeshPrsBuilder::AddFaceSolidPrs`: the fan triangles of a face, with the nodal
//! normals of the data source (smooth shading) or the face normal.
use super::MeshVSMeshPrsBuilderSource;
use super::elements::{Element, ElementSettings};
use super::primitives::{face_points, fan_index};
use crate::data_source::MeshVSDataSource;
use crate::mesh::MeshVSMeshMembers;
use crate::prs_builder::MeshVSPrsBuilderMembers;
use rustycad_tkernel::standard::Failure;
use rustycad_tkservice::graphic3d::Graphic3dArrayOfTriangles;

/// The native body; natively it reads the data source of the parent mesh (not the builder's
/// own one) and draws nothing without it.
pub(super) fn add_face_solid_prs<B: MeshVSMeshPrsBuilderSource + ?Sized>(
    builder: &B,
    element: &Element<'_, '_>,
    triangles: &Graphic3dArrayOfTriangles,
    settings: &ElementSettings,
) -> Result<(), Failure> {
    let parent = builder.parent_mesh();
    let source = parent
        .get()
        .map(|mesh| mesh.get_data_source())
        .unwrap_or_default();
    let Some(source) = source.get() else {
        return Ok(());
    };
    let is_shrinked = settings.is_shrink || settings.has_select;
    let is_shaded = settings.is_reflect;
    let mut normal = [0.0; 3];
    let mut vertex_normals = Vec::new();
    if is_shaded {
        vertex_normals = face_normals(source, element, settings, &mut normal);
    }
    let smooth = settings.is_smooth && vertex_normals.len() as i32 == element.nb_nodes;
    let points = face_points(
        element.coords,
        element.nb_nodes,
        is_shrinked,
        settings.shrink_coef,
    )?;
    for node in 0..element.nb_nodes - 2 {
        for sub in 0..3 {
            let index = fan_index(node, sub) as usize;
            let p = points[index];
            if !is_shaded {
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
    element: &Element<'_, '_>,
    settings: &ElementSettings,
    normal: &mut [f64; 3],
) -> Vec<[f64; 3]> {
    let mut vertex_normals = Vec::new();
    if settings.is_smooth {
        for node in 1..=element.nb_nodes {
            let [x, y, z] = normal;
            if !source.get_node_normal(node, element.key, x, y, z) {
                break;
            }
            vertex_normals.push(*normal);
        }
    }
    if !settings.is_smooth || vertex_normals.len() as i32 != element.nb_nodes {
        let [x, y, z] = normal;
        source.get_normal(element.key, settings.max_nodes, x, y, z);
    }
    vertex_normals
}
