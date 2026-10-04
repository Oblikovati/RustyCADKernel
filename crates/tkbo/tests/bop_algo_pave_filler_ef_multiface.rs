//! Native differential for deferred EF CommonBlock updates across incident faces.
use rustycad_tkbo::bop_algo::BOPAlgoPaveFiller;
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{Face, Shape, Vertex},
};
use rustycad_tkmath::gp::{Plane3, Point3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace};
use std::fmt::Write;

#[test]
fn perform_ef_defers_common_block_updates_until_all_faces_are_processed() {
    let edge = coplanar_edge();
    let first = plane_face(0.001);
    let second = plane_face(0.0005);
    let mut filler = BOPAlgoPaveFiller::new();
    filler.add_argument(&edge);
    filler.add_argument(&first);
    filler.add_argument(&second);
    filler.initialize().unwrap();
    let (edge_index, faces) = seed_face_endpoints(&mut filler, &edge, [&first, &second]);
    filler.perform_ef().unwrap();
    assert_eq!(
        append_result(&mut filler, edge_index, &faces),
        include_str!("../../../tools/oracle/tkbo_pave_filler_ef_multiface.expected")
    );
}

fn coplanar_edge() -> Shape {
    BRepBuilderAPIMakeEdge::from_points(Point3::new(-1.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0))
        .unwrap()
        .edge()
        .unwrap()
        .into_shape()
}

fn plane_face(tolerance: f64) -> Shape {
    let face = BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (-2.0, 2.0, -2.0, 2.0))
        .unwrap()
        .face()
        .unwrap()
        .into_shape();
    BRepBuilder::default()
        .update_face_tolerance(&Face::unchecked(face.clone()), tolerance)
        .unwrap();
    face
}

fn seed_face_endpoints(
    filler: &mut BOPAlgoPaveFiller,
    edge: &Shape,
    faces: [&Shape; 2],
) -> (i32, [i32; 2]) {
    let ds = filler.data_structure_mut().unwrap();
    let edge_index = ds.index(edge);
    let face_indices = faces.map(|face| ds.index(face));
    let endpoints = ds.shape_info(edge_index).unwrap().sub_shapes().to_vec();
    for face in face_indices {
        let info = ds.change_face_info(face).unwrap();
        for endpoint in &endpoints {
            info.change_vertices_on().add(*endpoint);
        }
    }
    (edge_index, face_indices)
}

fn append_result(filler: &mut BOPAlgoPaveFiller, edge: i32, faces: &[i32; 2]) -> String {
    let ds = filler.data_structure_mut().unwrap();
    let mut rows = String::new();
    append_summary(ds, faces, &mut rows);
    let block = &ds.pave_blocks(edge).unwrap()[0];
    append_common_block(ds, edge, block, &mut rows);
    append_endpoint_tolerances(ds, edge, &mut rows);
    rows
}

fn append_summary(
    ds: &mut rustycad_tkbo::bop_ds::DataStructure,
    faces: &[i32; 2],
    rows: &mut String,
) {
    writeln!(rows, "after {} {}", ds.nb_shapes(), ds.interf_ef().len()).unwrap();
    writeln!(rows, "faces {} {}", faces[0], faces[1]).unwrap();
    for face in faces {
        append_face_status(ds, *face, rows);
    }
}

fn append_face_status(ds: &rustycad_tkbo::bop_ds::DataStructure, face: i32, rows: &mut String) {
    let pave_blocks = ds.face_info(face).unwrap().pave_blocks_on().extent();
    writeln!(rows, "face {face} 1 {pave_blocks}").unwrap();
}

fn append_common_block(
    ds: &rustycad_tkbo::bop_ds::DataStructure,
    edge: i32,
    block: &rustycad_tkbo::bop_ds::PaveBlockHandle,
    rows: &mut String,
) {
    writeln!(rows, "pb {edge}").unwrap();
    let common = ds.common_block(block).unwrap();
    let common = common.lock().unwrap();
    writeln!(
        rows,
        "cb {} {:.9}",
        common.faces().len(),
        common.tolerance()
    )
    .unwrap();
    for face in common.faces() {
        writeln!(rows, "cf {face}").unwrap();
    }
}

fn append_endpoint_tolerances(
    ds: &rustycad_tkbo::bop_ds::DataStructure,
    edge: i32,
    rows: &mut String,
) {
    for vertex in ds.shape_info(edge).unwrap().sub_shapes() {
        let shape = ds.shape(*vertex).unwrap().clone();
        let tolerance = BRepTool::vertex_tolerance(&Vertex::unchecked(shape)).unwrap();
        writeln!(rows, "v {vertex} {tolerance:.9}").unwrap();
    }
}
