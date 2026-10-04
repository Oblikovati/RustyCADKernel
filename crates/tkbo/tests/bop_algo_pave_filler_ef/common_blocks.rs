//! Native EF common-block face membership and tolerance snapshots.
use super::*;

#[test]
fn perform_ef_common_edge_records_face_membership_and_tolerance() {
    let edge = coplanar_edge_argument();
    let face = face_argument();
    BRepBuilder::default()
        .update_face_tolerance(&Face::unchecked(face.clone()), 0.001)
        .unwrap();
    let mut filler = configured_filler(&edge, &face, false, 0.0);
    let (edge_index, face_index) = seed_face_endpoint_membership(&mut filler, &edge, &face);
    filler.perform_ef().unwrap();
    let mut rows = String::new();
    append_common_block_result(&mut filler, edge_index, face_index, &mut rows);
    assert_eq!(
        rows,
        include_str!("../../../../tools/oracle/tkbo_pave_filler_ef_commonblock.expected")
    );
}

fn seed_face_endpoint_membership(
    filler: &mut BOPAlgoPaveFiller,
    edge: &Shape,
    face: &Shape,
) -> (i32, i32) {
    let ds = filler.data_structure_mut().unwrap();
    let edge_index = ds.index(edge);
    let face_index = ds.index(face);
    let endpoints = ds.shape_info(edge_index).unwrap().sub_shapes().to_vec();
    let info = ds.change_face_info(face_index).unwrap();
    for endpoint in endpoints {
        info.change_vertices_on().add(endpoint);
    }
    (edge_index, face_index)
}

fn append_common_block_result(
    filler: &mut BOPAlgoPaveFiller,
    edge: i32,
    face: i32,
    rows: &mut String,
) {
    let ds = filler.data_structure_mut().unwrap();
    writeln!(rows, "after {} {}", ds.nb_shapes(), ds.interf_ef().len()).unwrap();
    append_coplanar_record(ds, rows);
    writeln!(
        rows,
        "face {} {}",
        ds.has_face_info(face).unwrap() as u8,
        ds.face_info(face).unwrap().pave_blocks_on().extent()
    )
    .unwrap();
    let block = ds.pave_blocks(edge).unwrap()[0].clone();
    append_common_metadata(ds, edge, &block, rows);
}

fn append_common_metadata(
    ds: &rustycad_tkbo::bop_ds::DataStructure,
    edge: i32,
    block: &rustycad_tkbo::bop_ds::PaveBlockHandle,
    rows: &mut String,
) {
    writeln!(rows, "pb {edge} {}", ds.is_common_block(block) as u8).unwrap();
    if let Some(common) = ds.common_block(block) {
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
    append_endpoint_tolerances(ds, edge, rows);
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
