//! Native differential coverage for the ordinary edge/edge vertex crossing path.
use rustycad_tkbo::bop_algo::BOPAlgoPaveFiller;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Compound, Shape, Vertex},
};
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;

#[test]
fn perform_ee_adds_crossing_vertex_and_splits_both_edge_blocks() {
    let first = edge_argument(Point3::new(-1.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0));
    let second = edge_argument(Point3::new(0.0, -1.0, 0.0), Point3::new(0.0, 1.0, 0.0));
    let mut filler = BOPAlgoPaveFiller::new();
    filler.add_argument(&first);
    filler.add_argument(&second);
    filler.options_mut().set_fuzzy_value(1.0e-6);
    filler.initialize().unwrap();
    filler.perform_ee().unwrap();
    assert_eq!(
        snapshot(&mut filler),
        include_str!("../../../tools/oracle/tkbo_pave_filler_ee.cross.expected").trim()
    );
}

#[test]
fn perform_ee_binds_identical_edge_parts_as_a_common_block() {
    let builder = BRepBuilder::default();
    let mut first_vertex = Vertex::default();
    let mut last_vertex = Vertex::default();
    builder
        .make_vertex_at(&mut first_vertex, Point3::new(0.0, 0.0, 0.0), 1.0e-7)
        .unwrap();
    builder
        .make_vertex_at(&mut last_vertex, Point3::new(2.0, 0.0, 0.0), 1.0e-7)
        .unwrap();
    let first = edge_argument_shape(
        BRepBuilderAPIMakeEdge::from_vertices(&first_vertex, &last_vertex)
            .unwrap()
            .edge()
            .unwrap()
            .into_shape(),
    );
    let second = edge_argument_shape(
        BRepBuilderAPIMakeEdge::from_vertices(&first_vertex, &last_vertex)
            .unwrap()
            .edge()
            .unwrap()
            .into_shape(),
    );
    let mut filler = BOPAlgoPaveFiller::new();
    filler.add_argument(&first);
    filler.add_argument(&second);
    filler.options_mut().set_fuzzy_value(1.0e-6);
    filler.initialize().unwrap();
    filler.perform_ee().unwrap();
    assert_eq!(
        overlap_snapshot(&mut filler),
        include_str!("../../../tools/oracle/tkbo_pave_filler_ee.overlap.expected").trim()
    );
}

fn snapshot(filler: &mut BOPAlgoPaveFiller) -> String {
    let structure = filler.data_structure_mut().unwrap();
    let (record_count, indices, new_index, kind, first_parameter, second_parameter) = {
        let records = structure.interf_ee();
        let record = records.value(0).unwrap();
        let part = record.common_part();
        (
            records.len(),
            (record.index1(), record.index2()),
            record.index_new(),
            part.shape_type() as i32,
            part.vertex_parameter1(),
            part.vertex_parameter2(),
        )
    };
    let mut output = format!(
        "ee {} {}\nrecord {} {} {} {} {:.8} {:.8}",
        structure.nb_shapes(),
        record_count,
        indices.0,
        indices.1,
        new_index,
        kind,
        first_parameter,
        second_parameter
    );
    append_blocks(&mut output, structure, 1);
    append_blocks(&mut output, structure, 5);
    output
}

fn append_blocks(output: &mut String, structure: &rustycad_tkbo::bop_ds::DataStructure, edge: i32) {
    let blocks = structure.pave_blocks(edge).unwrap();
    output.push_str(&format!("\nedge {edge} {}", blocks.len()));
    for handle in blocks {
        let block = handle.lock().unwrap();
        let (first, last) = block.range();
        let (first_vertex, last_vertex) = block.indices();
        output.push_str(&format!(
            "\npb {first_vertex} {last_vertex} {first:.8} {last:.8}"
        ));
    }
}

fn edge_argument(first: Point3, last: Point3) -> Shape {
    let edge = BRepBuilderAPIMakeEdge::from_points(first, last)
        .unwrap()
        .edge()
        .unwrap();
    edge_argument_shape(edge.into_shape())
}

fn edge_argument_shape(edge: Shape) -> Shape {
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    let mut shape = compound.into_shape();
    builder.add(&mut shape, &edge).unwrap();
    shape
}

fn overlap_snapshot(filler: &mut BOPAlgoPaveFiller) -> String {
    let structure = filler.data_structure_mut().unwrap();
    let shape_count = structure.nb_shapes();
    let (shape_count, record_count, record_indices, record_kind) = {
        let records = structure.interf_ee();
        let record = records.value(0).unwrap();
        (
            shape_count,
            records.len(),
            (record.index1(), record.index2()),
            record.common_part().shape_type() as i32,
        )
    };
    let mut output = format!(
        "overlap {shape_count} {record_count}\noverlap-record {} {} {record_kind}",
        record_indices.0, record_indices.1
    );
    append_common_blocks(&mut output, structure, 1);
    append_common_blocks(&mut output, structure, 5);
    output
}

fn append_common_blocks(
    output: &mut String,
    structure: &rustycad_tkbo::bop_ds::DataStructure,
    edge: i32,
) {
    let blocks = structure.pave_blocks(edge).unwrap();
    output.push_str(&format!("\noverlap-edge {edge} {}", blocks.len()));
    for handle in blocks {
        let block = handle.lock().unwrap();
        let (first, last) = block.indices();
        let count = structure
            .common_block(handle)
            .map(|common| common.lock().unwrap().pave_blocks().len())
            .unwrap_or(0);
        output.push_str(&format!("\noverlap-pb {first} {last} {count}"));
    }
}
