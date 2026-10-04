//! Native differential coverage for the ordinary edge/edge vertex crossing path.
use rustycad_tkbo::bop_algo::BOPAlgoPaveFiller;
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{Compound, Shape, Vertex},
};
use rustycad_tkmath::gp::{Axis2, Circle3, Point3, StandardDirection3};
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

#[test]
fn perform_ee_forces_vertex_edge_interference_at_a_pave_endpoint() {
    let first = edge_argument(Point3::new(0.0, 0.0, 0.0), Point3::new(2.0, 0.0, 0.0));
    let second = edge_argument(Point3::new(0.0, -1.0, 0.0), Point3::new(0.0, 1.0, 0.0));
    let mut filler = BOPAlgoPaveFiller::new();
    filler.add_argument(&first);
    filler.add_argument(&second);
    filler.options_mut().set_fuzzy_value(1.0e-6);
    filler.initialize().unwrap();
    filler.perform_ee().unwrap();
    assert_eq!(
        endpoint_snapshot(&mut filler),
        include_str!("../../../tools/oracle/tkbo_pave_filler_ee.endpoint.expected").trim()
    );
}

#[test]
fn perform_ee_fuses_generated_vertices_shared_by_three_edges() {
    let arguments = [
        edge_argument(Point3::new(-1.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0)),
        edge_argument(Point3::new(0.0, -1.0, 0.0), Point3::new(0.0, 1.0, 0.0)),
        edge_argument(Point3::new(-1.0, -1.0, 0.0), Point3::new(1.0, 1.0, 0.0)),
    ];
    let mut filler = BOPAlgoPaveFiller::new();
    for argument in &arguments {
        filler.add_argument(argument);
    }
    filler.options_mut().set_fuzzy_value(1.0e-6);
    filler.initialize().unwrap();
    filler.perform_ee().unwrap();
    assert_eq!(
        multi_snapshot(&mut filler),
        include_str!("../../../tools/oracle/tkbo_pave_filler_ee.multi.expected").trim()
    );
}

#[test]
fn perform_ee_uses_line_circle_common_range_to_fuse_near_intersections() {
    let line = edge_argument(Point3::new(-2.0, 0.0, 0.0), Point3::new(2.0, 0.0, 0.0));
    let circle = circle_argument();
    let near_crossing = edge_argument(
        Point3::new(1.0000005, -1.0, 0.0),
        Point3::new(1.0000005, 1.0, 0.0),
    );
    let mut filler = BOPAlgoPaveFiller::new();
    for argument in [&line, &circle, &near_crossing] {
        filler.add_argument(argument);
    }
    filler.initialize().unwrap();
    filler.perform_ee().unwrap();
    assert_eq!(
        near_line_circle_snapshot(&mut filler),
        include_str!("../../../tools/oracle/tkbo_pave_filler_ee.near_line_circle.expected").trim()
    );
}

#[test]
fn perform_ee_uses_second_common_range_when_circle_is_first() {
    let circle = circle_argument();
    let line = edge_argument(Point3::new(-2.0, 0.0, 0.0), Point3::new(2.0, 0.0, 0.0));
    let near_crossing = edge_argument(
        Point3::new(1.0000005, -1.0, 0.0),
        Point3::new(1.0000005, 1.0, 0.0),
    );
    let mut filler = BOPAlgoPaveFiller::new();
    for argument in [&circle, &line, &near_crossing] {
        filler.add_argument(argument);
    }
    filler.initialize().unwrap();
    filler.perform_ee().unwrap();
    assert_eq!(
        near_circle_line_snapshot(&mut filler),
        include_str!("../../../tools/oracle/tkbo_pave_filler_ee.near_circle_line.expected").trim()
    );
}

#[test]
fn perform_ee_constructs_intersection_at_far_origin() {
    let first = edge_argument(
        Point3::new(1.0e12 - 1.0, -2.0e12, 3.0e12),
        Point3::new(1.0e12 + 1.0, -2.0e12, 3.0e12),
    );
    let second = edge_argument(
        Point3::new(1.0e12, -2.0e12 - 1.0, 3.0e12),
        Point3::new(1.0e12, -2.0e12 + 1.0, 3.0e12),
    );
    let mut filler = BOPAlgoPaveFiller::new();
    filler.add_argument(&first);
    filler.add_argument(&second);
    filler.options_mut().set_fuzzy_value(1.0e-6);
    filler.initialize().unwrap();
    filler.perform_ee().unwrap();
    assert_eq!(
        far_origin_snapshot(&mut filler),
        include_str!("../../../tools/oracle/tkbo_pave_filler_ee.far_origin.expected").trim()
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

fn append_endpoint_blocks(
    output: &mut String,
    structure: &rustycad_tkbo::bop_ds::DataStructure,
    edge: i32,
) {
    let blocks = structure.pave_blocks(edge).unwrap();
    output.push_str(&format!("\nendpoint-edge {edge} {}", blocks.len()));
    for handle in blocks {
        let block = handle.lock().unwrap();
        let (first, last) = block.range();
        let (first_vertex, last_vertex) = block.indices();
        output.push_str(&format!(
            "\nendpoint-pb {first_vertex} {last_vertex} {first:.8} {last:.8}"
        ));
    }
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

fn endpoint_snapshot(filler: &mut BOPAlgoPaveFiller) -> String {
    let protected_vertices = filler.vertices_to_avoid_extension().to_vec();
    let structure = filler.data_structure_mut().unwrap();
    let mut output = format!(
        "endpoint {} {} {}",
        structure.nb_shapes(),
        structure.interf_ee().len(),
        structure.interf_ve_records().len()
    );
    let ee = structure.interf_ee().value(0).unwrap();
    output.push_str(&format!(
        "\nendpoint-ee {} {} {} {:.8} {:.8}",
        ee.index1(),
        ee.index2(),
        ee.index_new(),
        ee.common_part().vertex_parameter1(),
        ee.common_part().vertex_parameter2()
    ));
    for record in structure.interf_ve_records().iter() {
        output.push_str(&format!(
            "\nendpoint-ve {} {} {} {:.8}",
            record.index1(),
            record.index2(),
            record.index_new(),
            record.parameter()
        ));
    }
    append_endpoint_blocks(&mut output, structure, 1);
    append_endpoint_blocks(&mut output, structure, 5);
    output.push_str("\nendpoint-avoid");
    for vertex in &protected_vertices {
        output.push_str(&format!(" {vertex}"));
    }
    output
}

fn multi_snapshot(filler: &mut BOPAlgoPaveFiller) -> String {
    let structure = filler.data_structure_mut().unwrap();
    let shape_count = structure.nb_shapes();
    let record_lines = {
        let records = structure.interf_ee();
        let mut lines = String::new();
        for record in records.iter() {
            lines.push_str(&format!(
                "\nmulti-record {} {} {} {:.8} {:.8}",
                record.index1(),
                record.index2(),
                record.index_new(),
                record.common_part().vertex_parameter1(),
                record.common_part().vertex_parameter2()
            ));
        }
        format!("multi {shape_count} {}{lines}", records.len())
    };
    let mut output = record_lines;
    for edge in 0..shape_count {
        if structure.shape_info(edge).unwrap().shape_type()
            != rustycad_tkg3d::top_abs::ShapeEnum::Edge
        {
            continue;
        }
        append_multi_blocks(&mut output, structure, edge);
    }
    output
}

fn near_line_circle_snapshot(filler: &mut BOPAlgoPaveFiller) -> String {
    near_vertex_records(filler, "near-line-circle", "near-ee")
}

fn near_circle_line_snapshot(filler: &mut BOPAlgoPaveFiller) -> String {
    near_vertex_records(filler, "near-circle-line", "reversed-ee")
}

fn near_vertex_records(filler: &mut BOPAlgoPaveFiller, header: &str, prefix: &str) -> String {
    let structure = filler.data_structure_mut().unwrap();
    let shape_count = structure.nb_shapes();
    let records = copy_record_indices(structure);
    let mut output = format!("{header} {shape_count} {}", records.len());
    for (first, second, new_vertex) in records {
        let vertex = Vertex::unchecked(structure.shape(new_vertex).unwrap().clone());
        let point = BRepTool::pnt(&vertex).unwrap();
        let tolerance = BRepTool::vertex_tolerance(&vertex).unwrap();
        output.push_str(&format!(
            "\n{prefix} {first} {second} {new_vertex} {:.8} {:.8} {:.8} {:.8}",
            point.x(),
            point.y(),
            point.z(),
            tolerance
        ));
    }
    output
}

fn copy_record_indices(
    structure: &mut rustycad_tkbo::bop_ds::DataStructure,
) -> Vec<(i32, i32, i32)> {
    structure
        .interf_ee()
        .iter()
        .map(|record| (record.index1(), record.index2(), record.index_new()))
        .collect()
}

fn circle_argument() -> Shape {
    let circle = Circle3::new(Axis2::from_standard(StandardDirection3::Z), 1.0).unwrap();
    let edge = BRepBuilderAPIMakeEdge::from_circ(circle)
        .unwrap()
        .edge()
        .unwrap();
    edge_argument_shape(edge.into_shape())
}

fn far_origin_snapshot(filler: &mut BOPAlgoPaveFiller) -> String {
    let structure = filler.data_structure_mut().unwrap();
    let shape_count = structure.nb_shapes();
    let (record_count, first, second, new_vertex) = {
        let records = structure.interf_ee();
        let record = records.value(0).unwrap();
        (
            records.len(),
            record.index1(),
            record.index2(),
            record.index_new(),
        )
    };
    let vertex = Vertex::unchecked(structure.shape(new_vertex).unwrap().clone());
    let point = BRepTool::pnt(&vertex).unwrap();
    let tolerance = BRepTool::vertex_tolerance(&vertex).unwrap();
    format!(
        "far-origin {} {}\nfar-ee {} {} {} {:.8} {:.8} {:.8} {:.8}",
        shape_count,
        record_count,
        first,
        second,
        new_vertex,
        point.x(),
        point.y(),
        point.z(),
        tolerance
    )
}

fn append_multi_blocks(
    output: &mut String,
    structure: &rustycad_tkbo::bop_ds::DataStructure,
    edge: i32,
) {
    let blocks = structure.pave_blocks(edge).unwrap();
    output.push_str(&format!("\nmulti-edge {edge} {}", blocks.len()));
    for handle in blocks {
        let block = handle.lock().unwrap();
        let (first, last) = block.range();
        let (first_vertex, last_vertex) = block.indices();
        output.push_str(&format!(
            "\nmulti-pb {first_vertex} {last_vertex} {first:.8} {last:.8}"
        ));
    }
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
