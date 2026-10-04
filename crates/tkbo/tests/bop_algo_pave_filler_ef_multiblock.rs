//! Native differential coverage for EF generated vertices on previously split edges.
use rustycad_tkbo::bop_algo::BOPAlgoPaveFiller;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Shape, Vertex},
};
use rustycad_tkmath::gp::{Direction3, Plane3, Point3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace};
use std::fmt::Write;

#[test]
fn perform_ef_projects_into_the_matching_block_after_ve_split() {
    let point = split_point();
    let edge = vertical_edge();
    let face = plane_face();
    let mut filler = prepared_filler(&point, &edge, &face);
    filler.perform_ve().unwrap();
    filler.perform_ef().unwrap();
    assert_eq!(
        snapshot(&mut filler, &edge).trim(),
        include_str!("../../../tools/oracle/tkbo_pave_filler_ef_multiblock.expected").trim()
    );
}

#[test]
fn perform_ef_reuses_common_block_shared_by_coincident_edges() {
    let (first_edge, second_edge) = coincident_edges();
    let face = horizontal_face();
    let mut filler = prepared_group_filler(&first_edge, &second_edge, &face);
    filler.perform_vf().unwrap();
    filler.perform_ee().unwrap();
    filler.perform_ef().unwrap();
    assert_eq!(
        common_block_snapshot(&mut filler, [&first_edge, &second_edge]).trim(),
        include_str!("../../../tools/oracle/tkbo_pave_filler_ef_common_group.expected").trim()
    );
}

#[test]
fn perform_ef_fuses_hits_using_candidate_tolerance_beyond_fuzzy_value() {
    let edges = tolerance_offset_edges();
    let face = horizontal_face();
    let mut filler = BOPAlgoPaveFiller::new();
    edges.iter().for_each(|edge| filler.add_argument(edge));
    filler.add_argument(&face);
    filler.options_mut().set_fuzzy_value(4.0e-4);
    filler.initialize().unwrap();
    filler.perform_ef().unwrap();
    assert_eq!(
        tolerance_fusion_snapshot(&mut filler, &edges).trim(),
        include_str!("../../../tools/oracle/tkbo_pave_filler_ef_candidate_tolerance.expected")
            .trim()
    );
}

fn tolerance_offset_edges() -> [Shape; 3] {
    [
        line_edge((-1.0, 0.0, -1.0), (1.0, 0.0, 1.0)),
        line_edge((-0.99959, 0.00041, -1.0), (1.00041, 0.00041, 1.0)),
        line_edge((-0.99918, 0.00082, -1.0), (1.00082, 0.00082, 1.0)),
    ]
}

fn line_edge(first: (f64, f64, f64), last: (f64, f64, f64)) -> Shape {
    BRepBuilderAPIMakeEdge::from_points(
        Point3::new(first.0, first.1, first.2),
        Point3::new(last.0, last.1, last.2),
    )
    .unwrap()
    .edge()
    .unwrap()
    .into_shape()
}

fn coincident_edges() -> (Shape, Shape) {
    let builder = BRepBuilder::default();
    let mut first = Vertex::default();
    let mut last = Vertex::default();
    builder
        .make_vertex_at(&mut first, Point3::new(-1.0, 0.0, 0.0), 1.0e-7)
        .unwrap();
    builder
        .make_vertex_at(&mut last, Point3::new(1.0, 0.0, 0.0), 1.0e-7)
        .unwrap();
    let first_edge = edge_from_vertices(&first, &last);
    let second_edge = edge_from_vertices(&first, &last);
    (first_edge, second_edge)
}

fn edge_from_vertices(first: &Vertex, last: &Vertex) -> Shape {
    BRepBuilderAPIMakeEdge::from_vertices(first, last)
        .unwrap()
        .edge()
        .unwrap()
        .into_shape()
}

fn horizontal_face() -> Shape {
    BRepBuilderAPIMakeFace::from_plane_bounds(Plane3::default(), (-2.0, 2.0, -2.0, 2.0))
        .unwrap()
        .face()
        .unwrap()
        .into_shape()
}

fn prepared_group_filler(first: &Shape, second: &Shape, face: &Shape) -> BOPAlgoPaveFiller {
    let mut filler = BOPAlgoPaveFiller::new();
    filler.add_argument(first);
    filler.add_argument(second);
    filler.add_argument(face);
    filler.initialize().unwrap();
    filler
}

fn split_point() -> Shape {
    let mut vertex = Vertex::default();
    BRepBuilder::default()
        .make_vertex_at(&mut vertex, Point3::new(0.0, 0.0, 0.5), 1.0e-7)
        .unwrap();
    vertex.into_shape()
}

fn vertical_edge() -> Shape {
    BRepBuilderAPIMakeEdge::from_points(Point3::default(), Point3::new(0.0, 0.0, 2.0))
        .unwrap()
        .edge()
        .unwrap()
        .into_shape()
}

fn plane_face() -> Shape {
    let mut plane =
        Plane3::new(Point3::default(), Direction3::new(0.0, 0.0, 1.0).unwrap()).unwrap();
    plane.set_location(Point3::new(0.0, 0.0, 1.0));
    BRepBuilderAPIMakeFace::from_plane_bounds(plane, (-1.0, 1.0, -1.0, 1.0))
        .unwrap()
        .face()
        .unwrap()
        .into_shape()
}

fn prepared_filler(point: &Shape, edge: &Shape, face: &Shape) -> BOPAlgoPaveFiller {
    let mut filler = BOPAlgoPaveFiller::new();
    filler.add_argument(point);
    filler.add_argument(edge);
    filler.add_argument(face);
    filler.initialize().unwrap();
    filler
}

fn common_block_snapshot(filler: &mut BOPAlgoPaveFiller, edges: [&Shape; 2]) -> String {
    let structure = filler.data_structure_mut().unwrap();
    let mut rows = format!(
        "after {} {} {}\n",
        structure.nb_shapes(),
        structure.interf_ee().len(),
        structure.interf_ef().len()
    );
    for edge in edges {
        append_common_block(structure, edge, &mut rows);
    }
    rows
}

fn tolerance_fusion_snapshot(filler: &mut BOPAlgoPaveFiller, edges: &[Shape; 3]) -> String {
    let structure = filler.data_structure_mut().unwrap();
    let mut rows = format!(
        "after {} {} {}\n",
        structure.nb_shapes(),
        structure.interf_ef().len(),
        structure.interf_ve_records().len()
    );
    append_tolerance_records(structure, edges, &mut rows);
    rows
}

fn append_tolerance_records(
    structure: &mut rustycad_tkbo::bop_ds::DataStructure,
    edges: &[Shape; 3],
    rows: &mut String,
) {
    for record in structure.interf_ef().iter() {
        writeln!(
            rows,
            "ef {} {} {}",
            record.index1(),
            record.index2(),
            record.index_new()
        )
        .unwrap();
    }
    for edge in edges {
        append_edge_blocks(structure, edge, rows);
    }
    append_generated_vertex(structure, rows);
}

fn append_edge_blocks(
    structure: &rustycad_tkbo::bop_ds::DataStructure,
    edge: &Shape,
    rows: &mut String,
) {
    let index = structure.index(edge);
    for handle in structure.pave_blocks(index).unwrap() {
        let block = handle.lock().unwrap();
        let (first, last) = block.indices();
        let (first_parameter, last_parameter) = block.range();
        writeln!(
            rows,
            "pb {index} {first} {last} {first_parameter:.12} {last_parameter:.12}"
        )
        .unwrap();
    }
}

fn append_generated_vertex(structure: &rustycad_tkbo::bop_ds::DataStructure, rows: &mut String) {
    let index = structure.nb_shapes() - 1;
    let vertex = Vertex::unchecked(structure.shape(index).unwrap().clone());
    let point = rustycad_tkbrep::brep::BRepTool::pnt(&vertex).unwrap();
    let tolerance = rustycad_tkbrep::brep::BRepTool::vertex_tolerance(&vertex).unwrap();
    writeln!(
        rows,
        "v {index} {:.12} {:.12} {:.12} {tolerance:.12}",
        point.x(),
        point.y(),
        point.z()
    )
    .unwrap();
}

fn append_common_block(
    structure: &rustycad_tkbo::bop_ds::DataStructure,
    edge: &Shape,
    rows: &mut String,
) {
    let edge_index = structure.index(edge);
    for handle in structure.pave_blocks(edge_index).unwrap() {
        let block = handle.lock().unwrap();
        let (first, last) = block.indices();
        let (first_parameter, last_parameter) = block.range();
        let is_common = structure.is_common_block(handle) as i32;
        writeln!(
            rows,
            "pb {edge_index} {first} {last} {first_parameter:.12} {last_parameter:.12} {is_common}"
        )
        .unwrap();
        drop(block);
        append_common_metadata(structure, handle, rows);
    }
}

fn append_common_metadata(
    structure: &rustycad_tkbo::bop_ds::DataStructure,
    block: &rustycad_tkbo::bop_ds::PaveBlockHandle,
    rows: &mut String,
) {
    let Some(common) = structure.common_block(block) else {
        return;
    };
    let common = common.lock().unwrap();
    writeln!(
        rows,
        "cb {} {} {:.12}",
        common.pave_blocks().len(),
        common.faces().len(),
        common.tolerance()
    )
    .unwrap();
    for member in common.pave_blocks() {
        let edge = member.lock().unwrap().original_edge();
        writeln!(rows, "member {edge}").unwrap();
    }
    for face in common.faces() {
        writeln!(rows, "face {face}").unwrap();
    }
}

fn snapshot(filler: &mut BOPAlgoPaveFiller, edge: &Shape) -> String {
    let structure = filler.data_structure_mut().unwrap();
    let edge_index = structure.index(edge);
    let mut rows = format!(
        "after {} {} {}\n",
        structure.nb_shapes(),
        structure.interf_ve_records().len(),
        structure.interf_ef().len()
    );
    append_paves(structure, edge_index, &mut rows);
    append_interferences(structure, &mut rows);
    rows
}

fn append_paves(structure: &rustycad_tkbo::bop_ds::DataStructure, edge: i32, rows: &mut String) {
    for handle in structure.pave_blocks(edge).unwrap() {
        let block = handle.lock().unwrap();
        let (first, last) = block.indices();
        let (first_parameter, last_parameter) = block.range();
        writeln!(
            rows,
            "pb {edge} {first} {last} {first_parameter:.12} {last_parameter:.12}"
        )
        .unwrap();
    }
}

fn append_interferences(structure: &mut rustycad_tkbo::bop_ds::DataStructure, rows: &mut String) {
    for record in structure.interf_ve_records().iter() {
        writeln!(
            rows,
            "ve {} {} {:.12}",
            record.index1(),
            record.index2(),
            record.parameter()
        )
        .unwrap();
    }
    for record in structure.interf_ef().iter() {
        writeln!(
            rows,
            "ef {} {} {}",
            record.index1(),
            record.index2(),
            record.index_new()
        )
        .unwrap();
    }
}
