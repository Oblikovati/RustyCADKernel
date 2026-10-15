// SPDX-License-Identifier: GPL-2.0-only
//! The MeshVS builders on the unit-square fixture: builder order and identity, display-mode
//! acceptance, excluded ids, the color texture and the element color maps.
use crate::mesh_fixture::UnitSquareSource;
use martensite_tkmeshvs::flags::*;
use martensite_tkmeshvs::selection_policy::*;
use martensite_tkmeshvs::two_colors::bind_two_colors;
use martensite_tkmeshvs::*;
use rustycad_tkernel::ncollection::Sequence;
use rustycad_tkernel::quantity::{Color, NamedColor};
use rustycad_tkernel::standard::{Handle, RuntimeTyped};
use rustycad_tkmath::gp::{Point3, Vector3};
use rustycad_tkservice::graphic3d::{
    Graphic3dAspectsRenderMembers, Graphic3dStructure, Graphic3dStructureMembers,
    Graphic3dStructureSource,
};
use rustycad_tkservice::null_driver::NullGraphicDriver;
use rustycad_tkv3d::prs_mgr::{PrsMgrPresentableObject, PrsMgrPresentableObjectMembers};
use rustycad_tkv3d::v3d::V3dViewer;

fn mesh() -> (Handle<MeshVSMesh>, Handle<dyn MeshVSMeshSource>) {
    let mesh = MeshVSMesh::new(false).unwrap();
    mesh.set_data_source(&Handle::new(UnitSquareSource::default()).cast());
    let parent = mesh.clone().cast();
    (mesh, parent)
}

fn presentation() -> (Handle<V3dViewer>, Handle<dyn Graphic3dStructureSource>) {
    let viewer = V3dViewer::new(&NullGraphicDriver::new_handle()).unwrap();
    let prs = Graphic3dStructure::new(&viewer.structure_manager(), &Handle::default())
        .unwrap()
        .cast();
    (viewer, prs)
}

fn ids(values: &[i32]) -> MeshVSPackedMap {
    let mut map = MeshVSPackedMap::default();
    for v in values {
        map.add(*v);
    }
    map
}

#[test]
fn constructor_sets_wireframe_white_plastic_and_three_drawers() {
    let (mesh, _) = mesh();
    assert_eq!(mesh.display_mode(), MESHVS_DMF_WIRE_FRAME);
    assert_eq!(mesh.hilight_mode(), MESHVS_DMF_WIRE_FRAME);
    assert!(mesh.has_color() && mesh.has_material());
    let fields = mesh.mesh_base().fields.read();
    let mut max_nodes = 0;
    assert!(
        fields
            .current_drawer
            .get()
            .unwrap()
            .get_integer(MeshVSDrawerAttribute::MaxFaceNodes as i32, &mut max_nodes)
    );
    assert_eq!(max_nodes, 10);
    let mut scale = 0.0;
    assert!(
        fields
            .hilight_drawer
            .get()
            .unwrap()
            .get_double(MeshVSDrawerAttribute::MarkerScale as i32, &mut scale)
    );
    assert_eq!(scale, 2.0);
    assert_eq!(fields.selection_method, MeshVSMeshSelectionMethod::Nodes);
}

#[test]
fn builders_are_ordered_by_decreasing_priority_and_take_free_ids() {
    let (mesh, parent) = mesh();
    mesh.add_builder(
        &Handle::new(MeshVSMeshPrsBuilder::with_defaults(&parent)).cast(),
        false,
    );
    mesh.add_builder(
        &Handle::new(MeshVSTextPrsBuilder::with_defaults(
            &parent,
            5.,
            &Color::named(NamedColor::RED),
        ))
        .cast(),
        true,
    );
    mesh.add_builder(
        &Handle::new(MeshVSNodalColorPrsBuilder::with_defaults(&parent)).cast(),
        false,
    );
    let order: Vec<(i32, i32)> = (1..=3)
        .map(|i| {
            let b = mesh.get_builder(i);
            (b.get_id(), b.get_priority())
        })
        .collect();
    assert_eq!(
        order,
        [
            (1, MESHVS_BP_TEXT),
            (2, MESHVS_BP_NODAL_COLOR),
            (0, MESHVS_BP_MESH)
        ]
    );
    assert_eq!(mesh.get_hilighter().get_id(), 1);
    assert_eq!(
        mesh.find_builder(MeshVSNodalColorPrsBuilder::static_type())
            .get_id(),
        2
    );
    mesh.remove_builder_by_id(1);
    assert!(mesh.get_hilighter().is_null());
    assert_eq!(mesh.get_free_id(), 1);
    assert!(mesh.get_builder(3).is_null());
}

#[test]
fn display_modes_need_a_builder_flag_once_builders_exist() {
    let (mesh, parent) = mesh();
    assert!(mesh.accept_display_mode(64) && !mesh.accept_display_mode(0));
    mesh.add_builder(
        &Handle::new(MeshVSMeshPrsBuilder::with_defaults(&parent)).cast(),
        false,
    );
    assert!(mesh.accept_display_mode(MESHVS_DMF_SHRINK));
    assert!(!mesh.accept_display_mode(MESHVS_DMF_TEXT_DATA_PRS));
}

#[test]
fn hidden_nodes_default_to_all_hidden_and_update_selectable_nodes() {
    let (mesh, _) = mesh();
    assert!(mesh.is_hidden_node(1) && !mesh.is_selectable_node(1));
    mesh.set_hidden_elems(&Handle::new(
        rustycad_tkernel::tcol_std::HPackedMapOfInteger::from_map(ids(&[12])),
    ));
    mesh.update_selectable_nodes();
    let selectable = mesh.get_selectable_nodes();
    let selectable: Vec<i32> = selectable.get().unwrap().map().iter().collect();
    // No hidden-node map: only the nodes of visible elements are selectable.
    assert!(selectable.contains(&1) && selectable.contains(&8) && !selectable.contains(&4));
}

#[test]
fn excluding_builders_exclude_the_elements_they_draw() {
    let (mesh, parent) = mesh();
    let builder = MeshVSElementalColorPrsBuilder::with_defaults(&parent);
    builder.set_color1(11, &Color::named(NamedColor::RED));
    builder.set_color1(13, &Color::named(NamedColor::RED));
    let (_viewer, prs) = presentation();
    let mut excluded = MeshVSPackedMap::default();
    builder
        .build(
            &prs,
            &ids(&[11, 12, 13]),
            &mut excluded,
            true,
            MESHVS_DMF_ELEMENTAL_COLOR_DATA_PRS,
        )
        .unwrap();
    assert_eq!(excluded.iter().collect::<Vec<_>>(), [11, 13]);
    // Groups: solids, links and edges.
    assert_eq!(prs.get().unwrap().groups().len(), 3);
    let _ = mesh;
}

#[test]
fn equal_color_pairs_move_to_the_single_color_map_during_build() {
    let (_, parent) = mesh();
    let builder = MeshVSElementalColorPrsBuilder::with_defaults(&parent);
    let green = Color::named(NamedColor::GREEN);
    builder.set_color2(12, &bind_two_colors(green, green));
    let (_viewer, prs) = presentation();
    builder
        .build(
            &prs,
            &ids(&[12]),
            &mut MeshVSPackedMap::default(),
            true,
            MESHVS_DMF_ELEMENTAL_COLOR_DATA_PRS,
        )
        .unwrap();
    assert!(!builder.has_colors2());
    let mut moved = Color::default();
    assert!(builder.get_color1(12, &mut moved));
}

#[test]
fn nodal_texture_rows_hold_the_padded_map_and_the_invalid_color() {
    assert_eq!(nodal_color_prs_builder::get_nearest_pow2(3), 4);
    assert_eq!(nodal_color_prs_builder::get_nearest_pow2(1), 1);
    assert_eq!(nodal_color_prs_builder::get_nearest_pow2(i32::MAX), 1 << 30);
    let (mesh, parent) = mesh();
    let builder = MeshVSNodalColorPrsBuilder::with_defaults(&parent);
    builder.use_texture(true);
    let mut map = Sequence::default();
    map.append(Color::named(NamedColor::RED));
    map.append(Color::named(NamedColor::BLUE));
    map.append(Color::named(NamedColor::GREEN));
    builder.set_color_map(&map);
    for node in 1..=8 {
        builder.set_texture_coord(node, 0.5);
    }
    let (_viewer, prs) = presentation();
    builder
        .build(
            &prs,
            &ids(&[11]),
            &mut MeshVSPackedMap::default(),
            true,
            MESHVS_DMF_NODAL_COLOR_DATA_PRS,
        )
        .unwrap();
    let aspect = mesh.attributes().shading_aspect().aspect();
    let texture = aspect.get().unwrap().texture_map();
    let image = texture.get().unwrap().get_image(&Handle::default());
    let image = image.get().unwrap().data();
    assert_eq!(image.data.value(0, 0).unwrap(), [255, 0, 0, 255]);
    assert_eq!(image.data.value(0, 3).unwrap(), [0, 255, 0, 255]);
    let gray = Color::named(NamedColor::GRAY);
    let channel = |c: f64| (255.0 * c) as u8;
    let expected = [
        channel(gray.red()),
        channel(gray.green()),
        channel(gray.blue()),
        255,
    ];
    assert_eq!(image.data.value(1, 2).unwrap(), expected);
}

#[test]
fn vector_arrow_and_extreme_magnitudes_have_closed_forms() {
    let mut points = [Point3::default(); 8];
    assert_eq!(
        vector_prs_builder::calculate_arrow(&mut points, 10.0, 0.1),
        1.0
    );
    assert_eq!(points[1].y(), -0.2);
    let (_, parent) = mesh();
    let builder =
        MeshVSVectorPrsBuilder::with_defaults(&parent, 1.0, &Color::named(NamedColor::GREEN));
    builder.set_vector(false, 1, &Vector3::new(3., 4., 0.));
    builder.set_vector(false, 2, &Vector3::new(0., 0., 2.));
    let (mut min, mut max) = (0.0, 0.0);
    builder.get_min_max_vector_value(false, &mut min, &mut max);
    assert_eq!((min, max), (2.0, 5.0));
}

#[test]
fn text_builder_writes_its_height_into_the_mesh_drawer() {
    let (mesh, parent) = mesh();
    let builder = MeshVSTextPrsBuilder::with_defaults(&parent, 7.5, &Color::named(NamedColor::RED));
    builder.set_text(
        false,
        3,
        &rustycad_tkernel::tcollection::AsciiString::new("three"),
    );
    assert!(builder.has_texts(false) && !builder.has_texts(true));
    let mut height = 0.0;
    assert!(
        mesh.get_drawer()
            .get()
            .unwrap()
            .get_double(MeshVSDrawerAttribute::TextHeight as i32, &mut height)
    );
    assert_eq!(height, 7.5);
}

#[test]
fn mesh_compute_draws_with_every_matching_builder() {
    let (mesh, parent) = mesh();
    mesh.add_builder(
        &Handle::new(MeshVSMeshPrsBuilder::with_defaults(&parent)).cast(),
        true,
    );
    let (_viewer, prs) = presentation();
    let manager = Handle::<rustycad_tkv3d::prs_mgr::PrsMgrPresentationManager>::default();
    mesh.compute(&manager, &prs, MESHVS_DMF_SHADING).unwrap();
    // Without hidden-node map the drawing keeps every node (only selection treats them as
    // hidden): node markers, then the polygons, edges and links of the elements.
    assert_eq!(prs.get().unwrap().groups().len(), 4);
}

#[test]
fn public_mesh_types_are_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<MeshVSMesh>();
    assert_send_sync::<MeshVSDeformedDataSource>();
    assert_send_sync::<MeshVSMeshPrsBuilder>();
    assert_send_sync::<MeshVSNodalColorPrsBuilder>();
    assert_send_sync::<MeshVSElementalColorPrsBuilder>();
    assert_send_sync::<MeshVSTextPrsBuilder>();
    assert_send_sync::<MeshVSVectorPrsBuilder>();
    assert_send_sync::<MeshVSMeshOwner>();
    assert_send_sync::<MeshVSMeshEntityOwner>();
    assert_send_sync::<MeshVSBuffer>();
}
