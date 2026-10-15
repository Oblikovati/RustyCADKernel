// SPDX-License-Identifier: GPL-2.0-only
//! `MeshVS_MeshPrsBuilder::BuildNodes`: one point array of the visible nodes.
use super::MeshVSMeshPrsBuilderSource;
use crate::MeshVSDrawerAttribute as Key;
use crate::MeshVSTool;
use crate::data_source::MeshVSPackedMap;
use crate::flags::{MESHVS_DMF_HILIGHT_PRS, MESHVS_DMF_SELECTION_PRS, MESHVS_ET_NONE};
use crate::mesh::MeshVSMeshMembers;
use crate::prs_builder::{MeshVSPrsBuilderMembers, remaining_ids};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkservice::graphic3d::{
    Graphic3dArrayOfPoints, Graphic3dAspectsSource, Graphic3dGroupMembers as _,
    Graphic3dStructureMembers as _, Graphic3dStructureSource,
};

/// The native `MeshVS_MeshPrsBuilder::BuildNodes` body: the marker points of the nodes that
/// are displayed (`MeshVS_DA_DisplayNodes`), not hidden (outside selection and highlight)
/// and not excluded. Example: `meshvs_mesh_prs_build_nodes(&*builder, &prs, &ids, &mut excluded, 1)?`.
pub fn meshvs_mesh_prs_build_nodes<B: MeshVSMeshPrsBuilderSource + ?Sized>(
    builder: &B,
    prs: &Handle<dyn Graphic3dStructureSource>,
    ids: &MeshVSPackedMap,
    ids_to_exclude: &mut MeshVSPackedMap,
    display_mode: i32,
) -> Result<(), Failure> {
    let source = builder.get_data_source();
    let drawer = builder.get_drawer();
    let marker = MeshVSTool::create_aspect_marker3d(&builder.get_drawer(), true)?;
    let (Some(source), Some(drawer_value), false) = (source.get(), drawer.get(), marker.is_null())
    else {
        return Ok(());
    };
    let mut display_free_nodes = true;
    drawer_value.get_boolean(Key::DisplayNodes as i32, &mut display_free_nodes);
    let is_plain = (display_mode & (MESHVS_DMF_SELECTION_PRS | MESHVS_DMF_HILIGHT_PRS)) == 0;
    if !display_free_nodes {
        return Ok(());
    }
    let hidden = if is_plain {
        let parent = builder.parent_mesh();
        parent
            .get()
            .map(|mesh| mesh.get_hidden_nodes())
            .unwrap_or_default()
    } else {
        Handle::default()
    };
    let remaining = remaining_ids(ids, &hidden, ids_to_exclude);
    let upper = remaining.extent();
    if upper <= 0 {
        return Ok(());
    }
    let points = Handle::new(Graphic3dArrayOfPoints::new(upper, 0)?);
    let mut coords = Array1::<f64>::with_size(3);
    coords.update_lower_bound(1);
    let mut added = 0;
    for key in remaining.iter() {
        let (mut nb_nodes, mut kind) = (0, MESHVS_ET_NONE);
        if !source.get_geom(key, false, &mut coords, &mut nb_nodes, &mut kind) {
            continue;
        }
        if builder.is_excluding_on() {
            ids_to_exclude.add(key);
        }
        added += 1;
        points.add_vertex_f64(coords[1], coords[2], coords[3])?;
    }
    if added > 0 {
        let group = prs.new_group()?;
        group.set_primitives_aspect(&marker.cast::<dyn Graphic3dAspectsSource>());
        group.add_primitive_array(&points.cast(), true);
    }
    Ok(())
}
