// SPDX-License-Identifier: GPL-2.0-only
//! Closed-form checks of the MeshVS data source defaults, the deformed source, the volume
//! topologies and the scratch buffer.
use crate::mesh_fixture::UnitSquareSource;
use martensite_tkmeshvs::flags::*;
use martensite_tkmeshvs::*;
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::Handle;
use rustycad_tkmath::gp::{Point3, Vector3};

fn normal(source: &UnitSquareSource, id: i32, max: i32) -> (bool, [f64; 3]) {
    let mut n = [-7.0; 3];
    let [x, y, z] = &mut n;
    (source.get_normal(id, max, x, y, z), n)
}

#[test]
fn face_normal_is_the_unit_cross_product_of_the_first_and_last_edges() {
    let source = UnitSquareSource::default();
    assert_eq!(normal(&source, 11, 10), (true, [0., 0., 1.]));
    assert_eq!(normal(&source, 12, 10), (true, [0., 0., 1.]));
}

#[test]
fn degenerate_face_writes_zeros_and_fails_while_other_kinds_keep_the_output() {
    let source = UnitSquareSource::default();
    assert_eq!(normal(&source, 16, 10), (false, [0., 0., 0.]));
    assert_eq!(normal(&source, 13, 10), (false, [-7.; 3]));
    assert_eq!(normal(&source, 11, 0), (false, [-7.; 3]));
    assert_eq!(normal(&source, 11, 2), (false, [-7.; 3]));
}

#[test]
fn face_normals_repeat_the_face_normal_for_every_node() {
    let source = UnitSquareSource::default();
    let mut normals = Handle::<HArray1<f64>>::default();
    assert!(source.get_normals_by_element(11, true, 10, &mut normals));
    let values: Vec<f64> = normals.get().unwrap().array1().iter().copied().collect();
    assert_eq!(values, [0., 0., 1., 0., 0., 1., 0., 0., 1.]);
    let mut untouched = Handle::<HArray1<f64>>::default();
    assert!(!source.get_normals_by_element(15, false, 10, &mut untouched));
    assert!(untouched.is_null());
    assert!(!source.get_normals_by_element(11, false, 0, &mut untouched));
}

#[test]
fn prism_face_normals_are_the_unnormalized_average_of_each_face() {
    let source = UnitSquareSource::default();
    let mut normals = Handle::<HArray1<f64>>::default();
    assert!(source.get_normals_by_element(14, false, 10, &mut normals));
    let array = normals.get().unwrap().array1();
    assert_eq!(array.length(), 15);
    // The first face reverses the base (2, 1, 0): its unit normal points down; the second
    // (the top) points up.
    assert_eq!([array[1], array[2], array[3]], [0.0, 0.0, -1.0]);
    assert_eq!([array[4], array[5], array[6]], [0.0, 0.0, 1.0]);
}

#[test]
fn bounding_box_covers_every_node() {
    let bounds = UnitSquareSource::default().get_bounding_box();
    let [x0, y0, z0, x1, y1, z1] = bounds.get().unwrap();
    assert!(x0 <= 0. && y0 <= 0. && z0 <= 0.);
    assert!(x1 >= 1. && y1 >= 1. && z1 >= 1.);
}

#[test]
fn default_groups_are_absent() {
    let source = UnitSquareSource::default();
    let mut kind = MESHVS_ET_ALL;
    let mut ids = MeshVSPackedMap::default();
    assert!(!source.get_group(1, &mut kind, &mut ids));
    assert_eq!(kind, MESHVS_ET_NONE);
    assert_eq!(source.get_group_addr(1), 0);
    assert!(!source.is_advanced_selection_enabled());
}

#[test]
fn prism_and_pyramid_topologies_follow_the_native_orientation() {
    let prism = MeshVSDataSource3DBase::create_prism_topology(3);
    let faces = prism.get().unwrap().array1();
    let face = |i: i32| faces[i].iter().copied().collect::<Vec<i32>>();
    assert_eq!(face(1), [2, 1, 0]);
    assert_eq!(face(2), [3, 4, 5]);
    assert_eq!(face(3), [1, 4, 3, 0]);
    let pyramid = MeshVSDataSource3DBase::create_pyramid_topology(4);
    let faces = pyramid.get().unwrap().array1();
    assert_eq!(faces[1].iter().copied().collect::<Vec<i32>>(), [4, 3, 2, 1]);
    assert_eq!(faces[5].iter().copied().collect::<Vec<i32>>(), [0, 4, 1]);
    assert!(MeshVSDataSource3DBase::create_prism_topology(2).is_null());
}

#[test]
fn cached_topologies_are_shared_handles() {
    let source = UnitSquareSource::default();
    let first = source.get_prism_topology(5);
    assert_eq!(first.object_address(), source.get_prism_topology(5).object_address());
    assert!(source.get_pyramid_topology(1).is_null());
}

fn deformed(magnify: f64) -> MeshVSDeformedDataSource {
    let source: Handle<dyn MeshVSDataSource> = Handle::new(UnitSquareSource::default()).cast();
    MeshVSDeformedDataSource::new(&source, magnify)
}

#[test]
fn deformed_nodes_move_by_the_magnified_vector() {
    let d = deformed(2.0);
    d.set_vector(2, &Vector3::new(0.25, 0.5, -1.0));
    let mut coords = Array1::<f64>::with_bounds(1, 3).unwrap();
    let (mut nb, mut kind) = (0, MESHVS_ET_NONE);
    assert!(d.get_geom(2, false, &mut coords, &mut nb, &mut kind));
    assert_eq!([coords[1], coords[2], coords[3]], [1.5, 1.0, -2.0]);
    assert!(!d.get_geom(3, false, &mut coords, &mut nb, &mut kind));
}

#[test]
fn deformed_element_needs_every_node_vector() {
    let d = deformed(1.0);
    for node in [2, 3] {
        d.set_vector(node, &Vector3::new(0., 0., 1.));
    }
    let mut coords = Array1::<f64>::with_bounds(1, 30).unwrap();
    let (mut nb, mut kind) = (0, MESHVS_ET_NONE);
    assert!(d.get_geom(13, true, &mut coords, &mut nb, &mut kind));
    assert_eq!((nb, kind, coords[3], coords[6]), (2, MESHVS_ET_LINK, 1.0, 1.0));
    assert!(!d.get_geom(11, true, &mut coords, &mut nb, &mut kind));
}

#[test]
fn non_positive_magnify_becomes_one_and_the_null_source_is_empty() {
    let d = deformed(-3.0);
    assert_eq!(d.get_magnify(), 1.0);
    d.set_magnify(0.25);
    assert_eq!(d.get_magnify(), 0.25);
    d.set_non_deformed_data_source(&Handle::default());
    assert!(d.get_all_nodes().is_empty());
    assert_eq!(d.get_addr(1, false), 0);
}

#[test]
fn buffer_views_alias_one_storage() {
    let mut buffer = MeshVSBuffer::new(4 * 3 * 8);
    assert!(!buffer.is_dynamic());
    buffer.set_point(1, &Point3::new(1.0, 2.0, 3.0));
    assert_eq!(buffer.real(4), 2.0);
    buffer.set_integer(0, 7);
    assert_eq!(buffer.as_bytes_mut()[..4], 7_i32.to_ne_bytes());
    assert!(MeshVSBuffer::new(MESHVS_BUF_SIZE + 1).is_dynamic());
}
