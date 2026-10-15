// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeBox, BRepPrimAPIMakeCylinder};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

fn unit_box() -> Shape {
    let mut maker = BRepPrimAPIMakeBox::from_sizes(1.0, 2.0, 3.0).unwrap();
    maker.shape().unwrap().clone()
}

fn vertex_coordinates(topology: &BodyTopology) -> Vec<[f64; 3]> {
    (0..topology.vertex_count() as u32)
        .map(|index| {
            let vertex = topology.vertex(VertexIndex(index)).unwrap();
            let point = BRepTool::pnt(&vertex).unwrap();
            [point.x(), point.y(), point.z()]
        })
        .collect()
}

#[test]
fn box_enumerates_six_faces_twelve_edges_eight_vertices() {
    let topology = BodyTopology::enumerate(&unit_box()).unwrap();
    assert_eq!(topology.face_count(), 6);
    assert_eq!(topology.edge_count(), 12);
    assert_eq!(topology.vertex_count(), 8);
}

#[test]
fn separately_built_bodies_enumerate_in_the_same_order() {
    // Two constructions own distinct handles; the order must still agree.
    let first = BodyTopology::enumerate(&unit_box()).unwrap();
    let second = BodyTopology::enumerate(&unit_box()).unwrap();
    assert_eq!(vertex_coordinates(&first), vertex_coordinates(&second));
}

#[test]
fn every_entity_maps_back_to_its_index() {
    let topology = BodyTopology::enumerate(&unit_box()).unwrap();
    for index in 0..topology.face_count() as u32 {
        let face = topology.face(FaceIndex(index)).unwrap();
        assert_eq!(topology.face_index(&face), Some(FaceIndex(index)));
    }
    for index in 0..topology.edge_count() as u32 {
        let edge = topology.edge(EdgeIndex(index)).unwrap();
        assert_eq!(topology.edge_index(&edge), Some(EdgeIndex(index)));
    }
    let vertex = topology.vertex(VertexIndex(7)).unwrap();
    assert_eq!(topology.vertex_index(&vertex), Some(VertexIndex(7)));
}

#[test]
fn reversed_face_keeps_its_index_and_foreign_face_has_none() {
    let topology = BodyTopology::enumerate(&unit_box()).unwrap();
    let face = topology.face(FaceIndex(2)).unwrap();
    assert_eq!(topology.face_index(&face.reversed()), Some(FaceIndex(2)));
    let other = BodyTopology::enumerate(&unit_box()).unwrap();
    assert_eq!(
        topology.face_index(&other.face(FaceIndex(2)).unwrap()),
        None
    );
    assert!(topology.face(FaceIndex(6)).is_none());
}

#[test]
fn cylinder_counts_its_seam_once() {
    let mut maker = BRepPrimAPIMakeCylinder::new(1.0, 2.0);
    let topology = BodyTopology::enumerate(maker.shape().unwrap()).unwrap();
    assert_eq!(topology.face_count(), 3);
    assert_eq!(topology.edge_count(), 3);
    assert_eq!(topology.vertex_count(), 2);
}

#[test]
fn null_body_is_refused() {
    let error = BodyTopology::enumerate(&Shape::default()).err().unwrap();
    assert_eq!(error.operation, BodyQuery::Enumerate);
    assert_eq!(error.class, BodyQueryFailure::NullBody);
}
