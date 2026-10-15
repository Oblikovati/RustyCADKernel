// SPDX-License-Identifier: GPL-2.0-only
//! `MeshVS_MeshPrsBuilder::BuildHilightPrs`: the first entity of the ids as a point, a
//! segment, a polygon or the polygons of a volume, in an unlit group.
use super::MeshVSMeshPrsBuilderSource;
use super::primitives::node_xyz;
use crate::MeshVSDrawerAttribute as Key;
use crate::MeshVSTool;
use crate::data_source::{MeshVSPackedMap, MeshVSTopologyHandle};
use crate::flags::*;
use crate::prs_builder::{MeshVSPrsBuilderMembers, coords_of};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkservice::graphic3d::*;

fn aspect<T: Graphic3dAspectsSource + rustycad_tkernel::standard::RuntimeTyped>(
    value: &Handle<T>,
) -> Handle<dyn Graphic3dAspectsSource> {
    value.clone().cast()
}

/// The native `MeshVS_MeshPrsBuilder::BuildHilightPrs` body.
/// Example: `meshvs_mesh_prs_build_hilight_prs(&*builder, &prs, &ids, true)?`.
pub fn meshvs_mesh_prs_build_hilight_prs<B: MeshVSMeshPrsBuilderSource + ?Sized>(
    builder: &B,
    prs: &Handle<dyn Graphic3dStructureSource>,
    ids: &MeshVSPackedMap,
    is_element: bool,
) -> Result<(), Failure> {
    let source = builder.get_data_source();
    let Some(source) = source.get().filter(|_| !ids.is_empty()) else {
        return Ok(());
    };
    let drawer = builder.get_drawer();
    let mut max_nodes = 0;
    if !drawer
        .get()
        .is_some_and(|d| d.get_integer(Key::MaxFaceNodes as i32, &mut max_nodes))
        || max_nodes <= 0
    {
        return Ok(());
    }
    let mut coords = coords_of(max_nodes)?;
    let mut material = Graphic3dMaterialAspect::default();
    drawer
        .get()
        .expect("drawer")
        .get_material(Key::FrontMaterial as i32, &mut material);
    let fill = MeshVSTool::create_aspect_fill_area3d_with_material(&drawer, &material, true)?;
    fill.get()
        .ok_or_else(|| {
            Failure::with_type(
                "Standard_NullObject",
                "MeshVS_MeshPrsBuilder highlight fill=null: expected the drawer fill keys",
                "",
            )
        })?
        .set_shading_model(Graphic3dTypeOfShadingModel::Unlit);
    let beam = MeshVSTool::create_aspect_line3d(&drawer, true)?;
    let marker = MeshVSTool::create_aspect_marker3d(&drawer, true)?;
    let id = ids.iter().next().expect("non-empty ids");
    let (mut nb_nodes, mut kind) = (0, MESHVS_ET_NONE);
    if !source.get_geom(id, is_element, &mut coords, &mut nb_nodes, &mut kind) {
        return Ok(());
    }
    let group = prs.new_group()?;
    match kind {
        MESHVS_ET_NODE => {
            group.set_primitives_aspect(&aspect(&marker));
            let points = Handle::new(Graphic3dArrayOfPoints::new(1, 0)?);
            let p = node_xyz(&coords, 1)?;
            points.add_vertex_f64(p[0], p[1], p[2])?;
            group.add_primitive_array(&points.cast(), true);
        }
        MESHVS_ET_LINK => {
            group.set_primitives_aspect(&aspect(&beam));
            let segments = Handle::new(Graphic3dArrayOfSegments::new(2, 0, 0)?);
            for k in 1..=2 {
                let p = node_xyz(&coords, k)?;
                segments.add_vertex_f64(p[0], p[1], p[2])?;
            }
            group.add_primitive_array(&segments.cast(), true);
        }
        MESHVS_ET_FACE if nb_nodes > 0 => {
            group.set_primitives_aspect(&aspect(&fill));
            let polygons = Handle::new(Graphic3dArrayOfPolygons::new(nb_nodes, 0, 0, 0)?);
            for k in 1..=nb_nodes {
                let p = node_xyz(&coords, k)?;
                polygons.add_vertex_f64(p[0], p[1], p[2])?;
            }
            group.add_primitive_array(&polygons.cast(), true);
        }
        MESHVS_ET_VOLUME if nb_nodes > 0 => {
            group.set_primitives_aspect(&aspect(&fill));
            let mut topo = MeshVSTopologyHandle::default();
            if source.get_3d_geom(id, &mut nb_nodes, &mut topo) {
                group.add_primitive_array(&volume_polygons(&topo, &coords)?.cast(), true);
            }
        }
        MESHVS_ET_FACE | MESHVS_ET_VOLUME => {}
        _ => {
            let mut excluded = MeshVSPackedMap::default();
            builder.custom_build(prs, ids, &mut excluded, MESHVS_DMF_HILIGHT_PRS)?;
        }
    }
    Ok(())
}

/// One bound per volume face, the face nodes as polygon vertices.
fn volume_polygons(
    topo: &MeshVSTopologyHandle,
    coords: &Array1<'_, f64>,
) -> Result<Handle<Graphic3dArrayOfPolygons>, Failure> {
    let faces = topo.get().expect("volume topology").array1();
    let nb_vertices: i32 = faces.iter().map(|f| f.length()).sum();
    let polygons = Handle::new(Graphic3dArrayOfPolygons::new(
        nb_vertices,
        faces.length(),
        0,
        0,
    )?);
    for face in faces.iter() {
        polygons.add_bound(face.length())?;
        for &index in face.iter() {
            let p = node_xyz(coords, index + 1)?;
            polygons.add_vertex_f64(p[0], p[1], p[2])?;
        }
    }
    Ok(polygons)
}
