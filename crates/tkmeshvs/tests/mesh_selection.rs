// SPDX-License-Identifier: GPL-2.0-only
//! MeshVS selection on the unit-square fixture: owners and entities by mode, the native
//! centers of the quad and polyhedron, owner reuse, and the mesh owner detection state.
use crate::mesh_fixture::UnitSquareSource;
use martensite_tkmeshvs::flags::*;
use martensite_tkmeshvs::selection_policy::*;
use martensite_tkmeshvs::*;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::tcol_std::HPackedMapOfInteger;
use rustycad_tkmath::gp::Point3;
use rustycad_tkv3d::select_mgr::{SelectMgrEntityOwner, SelectMgrEntityOwnerMembers, SelectMgrEntityOwnerSource, SelectMgrSelectableObject, SelectMgrSelection};
use rustycad_tkv3d::select3d::{Select3DSensitiveEntity, Select3DSensitiveEntityMembers};

fn mesh() -> Handle<MeshVSMesh> {
    let mesh = MeshVSMesh::new(false).unwrap();
    mesh.set_data_source(&Handle::new(UnitSquareSource::default()).cast());
    mesh.add_builder(&Handle::new(MeshVSMeshPrsBuilder::with_defaults(&mesh.clone().cast())).cast(), true);
    mesh
}

fn entity_types(mesh: &Handle<MeshVSMesh>, mode: i32) -> Vec<String> {
    let selection = Handle::new(SelectMgrSelection::new(mode));
    mesh.compute_selection(&selection, mode).unwrap();
    selection
        .entities()
        .iter()
        .map(|e| e.base_sensitive().get().unwrap().dynamic_type().name().to_owned())
        .collect()
}

#[test]
fn hidden_nodes_get_dummy_entities_and_selectable_ones_points() {
    let mesh = mesh();
    assert_eq!(entity_types(&mesh, MESHVS_SMF_NODE.0), vec!["MeshVS_DummySensitiveEntity"; 8]);
    mesh.set_hidden_nodes(&Handle::new(HPackedMapOfInteger::default()));
    assert_eq!(entity_types(&mesh, MESHVS_SMF_NODE.0), vec!["Select3D_SensitivePoint"; 8]);
    assert_eq!(mesh.get_owner_maps(false).extent(), 8);
}

#[test]
fn faces_become_triangles_and_volumes_polyhedra() {
    let mesh = mesh();
    assert_eq!(entity_types(&mesh, MESHVS_SMF_FACE.0), ["Select3D_SensitiveTriangle"; 3]);
    assert_eq!(entity_types(&mesh, MESHVS_SMF_VOLUME.0), ["MeshVS_SensitivePolyhedron"]);
    assert_eq!(entity_types(&mesh, MESHVS_SMF_LINK.0), ["MeshVS_SensitiveSegment"]);
    // The 0D element of the default builder has no custom entity.
    assert!(entity_types(&mesh, MESHVS_SMF_0D.0).is_empty());
    // The element owners merge the four type maps once, the 0D owner without entity included.
    assert_eq!(mesh.get_owner_maps(true).extent(), 6);
}

#[test]
fn whole_mesh_owner_is_kept_across_computations() {
    let mesh = mesh();
    mesh.set_mesh_sel_method(MeshVSMeshSelectionMethod::Precise);
    let types = entity_types(&mesh, MESHVS_SMF_MESH.0);
    assert_eq!(types[0], "MeshVS_CommonSensitiveEntity");
    let owner = mesh.mesh_base().fields.read().whole_mesh_owner.clone();
    entity_types(&mesh, MESHVS_SMF_MESH.0);
    assert_eq!(owner.object_address(), mesh.mesh_base().fields.read().whole_mesh_owner.object_address());
    assert!(mesh.is_whole_mesh_owner(&owner));
}

#[test]
fn no_highlighter_or_no_face_size_means_no_selection() {
    let mesh = MeshVSMesh::new(false).unwrap();
    mesh.set_data_source(&Handle::new(UnitSquareSource::default()).cast());
    assert!(entity_types(&mesh, MESHVS_SMF_FACE.0).is_empty());
}

fn owner(mesh: &Handle<MeshVSMesh>) -> Handle<dyn SelectMgrEntityOwnerSource> {
    SelectMgrEntityOwner::with_selectable(&mesh.clone().cast(), 0).cast()
}

#[test]
fn quad_center_is_the_corner_mean() {
    let mesh = mesh();
    let corners = [Point3::new(0., 0., 0.), Point3::new(2., 0., 0.), Point3::new(2., 2., 0.), Point3::new(0., 2., 4.)];
    let quad = MeshVSSensitiveQuad::new(&owner(&mesh), corners);
    assert_eq!(quad.center_of_geometry(), Point3::new(1., 1., 1.));
    assert_eq!(quad.nb_sub_elements(), 1);
    assert!(MeshVSSensitiveQuad::from_points(&owner(&mesh), &corners[..3]).is_err());
}

#[test]
fn polyhedron_center_divides_the_face_vertex_sum_by_the_node_count() {
    let mesh = mesh();
    let nodes: Vec<Point3> = crate::mesh_fixture::ELEMENTS[3].1.iter().map(|&n| {
        let p = crate::mesh_fixture::NODES[(n - 1) as usize];
        Point3::new(p[0], p[1], p[2])
    }).collect();
    let topo = MeshVSDataSource3DBase::create_prism_topology(3);
    let polyhedron = MeshVSSensitivePolyhedron::new(&owner(&mesh), &nodes, 1, &topo).unwrap();
    // Faces 3+3+4+4+4 = 18 vertex visits over 6 nodes: each node appears 3 times.
    let mut sum = [0.0; 3];
    for p in &nodes {
        for (axis, value) in sum.iter_mut().zip([p.x(), p.y(), p.z()]) {
            *axis += 3.0 * value;
        }
    }
    let center = polyhedron.center_of_geometry();
    assert!((center.x() - sum[0] / 6.).abs() < 1e-15);
    assert!((center.z() - sum[2] / 6.).abs() < 1e-15);
    assert_eq!(polyhedron.nb_sub_elements(), 6);
}

#[test]
fn mesh_owner_unites_selected_maps_and_forgets_selection_on_detection() {
    let mesh = mesh();
    let source = mesh.get_data_source();
    let owner = MeshVSMeshOwner::new(&mesh.clone().cast(), &source, 5);
    let map = |ids: &[i32]| {
        let m = Handle::new(HPackedMapOfInteger::default());
        for id in ids {
            m.change_map().add(*id);
        }
        m
    };
    owner.add_selected_entities(&map(&[1, 2]), &Handle::default());
    owner.add_selected_entities(&map(&[3]), &map(&[11]));
    assert_eq!(owner.get_selected_nodes().get().unwrap().map().extent(), 3);
    assert_eq!(owner.get_selected_elements().get().unwrap().map().extent(), 1);
    owner.set_selected(true);
    owner.set_detected_entities(&map(&[2]), &Handle::default());
    assert!(!owner.is_selected());
    assert!(owner.is_forced_hilight());
    owner.clear_selected_entities();
    assert!(owner.get_selected_nodes().is_null());
}

#[test]
fn sensitive_mesh_reads_the_box_of_the_owner_source() {
    let mesh = mesh();
    let owner = MeshVSMeshOwner::new(&mesh.clone().cast(), &mesh.get_data_source(), 5);
    let entity = MeshVSSensitiveMesh::new(&owner.cast(), 4).unwrap();
    assert_eq!(entity.get_mode(), 4);
    assert_eq!(entity.nb_sub_elements(), 8);
    let plain = MeshVSSensitiveMesh::new(&super::mesh_selection::owner(&mesh), 1).unwrap();
    assert_eq!(plain.nb_sub_elements(), -1);
    assert_eq!(plain.center_of_geometry(), Point3::new(0., 0., 0.));
}

#[test]
fn selection_is_identical_across_threads() {
    let types = |_: usize| entity_types(&mesh(), MESHVS_SMF_FACE.0);
    let serial = types(0);
    let threads: Vec<_> = (0..4).map(|i| std::thread::spawn(move || types(i))).collect();
    for thread in threads {
        assert_eq!(thread.join().unwrap(), serial);
    }
}

#[test]
fn entity_owner_reports_its_entity() {
    let mesh = mesh();
    let selectable: Handle<dyn SelectMgrSelectableObject> = mesh.clone().cast();
    let owner = MeshVSMeshEntityOwner::new(&selectable, 12, 99, MESHVS_ET_FACE, 2, false);
    assert_eq!((owner.id(), owner.owner(), owner.kind(), owner.is_group()), (12, 99, MESHVS_ET_FACE, false));
    assert_eq!(owner.priority(), 2);
    let entity = MeshVSDummySensitiveEntity::new(&owner.cast());
    assert_eq!(entity.nb_sub_elements(), -1);
    assert!(!entity.owner_id().is_null());
}
