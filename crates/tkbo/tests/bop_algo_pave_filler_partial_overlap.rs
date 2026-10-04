//! Native differential coverage for endpoint-split partial edge overlaps.
use rustycad_tkbo::bop_algo::BOPAlgoPaveFiller;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Compound, Shape},
};
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;

#[test]
fn perform_ve_then_ee_builds_common_block_for_partial_overlap() {
    let (first, second) = partial_overlap_arguments();
    let mut filler = initialized_filler(&first, &second);
    filler.perform_ve().unwrap();
    filler.perform_ee().unwrap();
    assert_eq!(
        partial_overlap_snapshot(&mut filler),
        native_overlap_expected()
    );
}

#[test]
fn perform_ee_alone_keeps_unsplit_partial_overlap_unbound() {
    let (first, second) = partial_overlap_arguments();
    let mut filler = initialized_filler(&first, &second);
    filler.perform_ee().unwrap();
    assert_eq!(
        partial_ee_only_snapshot(&mut filler),
        native_ee_only_expected()
    );
}

fn initialized_filler(first: &Shape, second: &Shape) -> BOPAlgoPaveFiller {
    let mut filler = BOPAlgoPaveFiller::new();
    filler.add_argument(first);
    filler.add_argument(second);
    filler.options_mut().set_fuzzy_value(1.0e-6);
    filler.initialize().unwrap();
    filler
}

fn partial_overlap_arguments() -> (Shape, Shape) {
    let first = edge_argument(Point3::new(0.0, 0.0, 0.0), Point3::new(2.0, 0.0, 0.0));
    let second = edge_argument(Point3::new(1.0, 0.0, 0.0), Point3::new(3.0, 0.0, 0.0));
    (first, second)
}

fn edge_argument(first: Point3, last: Point3) -> Shape {
    let edge = BRepBuilderAPIMakeEdge::from_points(first, last)
        .unwrap()
        .edge()
        .unwrap();
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    let mut shape = compound.into_shape();
    builder.add(&mut shape, &edge.into_shape()).unwrap();
    shape
}

fn partial_ee_only_snapshot(filler: &mut BOPAlgoPaveFiller) -> String {
    let structure = filler.data_structure_mut().unwrap();
    let shape_count = structure.nb_shapes();
    let record_count = structure.interf_ee().len();
    format!("partial-ee-only {shape_count} {record_count}")
}

fn partial_overlap_snapshot(filler: &mut BOPAlgoPaveFiller) -> String {
    let structure = filler.data_structure_mut().unwrap();
    let mut output = format!("partial-overlap {}", structure.nb_shapes());
    append_interferences(&mut output, structure);
    append_pave_blocks(&mut output, structure, 1);
    append_pave_blocks(&mut output, structure, 5);
    output
}

fn append_interferences(output: &mut String, structure: &mut rustycad_tkbo::bop_ds::DataStructure) {
    output.push_str(&format!(
        " {} {}",
        structure.interf_ve_records().len(),
        structure.interf_ee().len()
    ));
    for record in structure.interf_ve_records().iter() {
        output.push_str(&format!(
            "\npartial-ve {} {} {} {:.8}",
            record.index1(),
            record.index2(),
            record.index_new(),
            record.parameter()
        ));
    }
    for record in structure.interf_ee().iter() {
        output.push_str(&format!(
            "\npartial-ee {} {} {} {}",
            record.index1(),
            record.index2(),
            record.index_new(),
            record.common_part().shape_type() as i32
        ));
    }
}

fn append_pave_blocks(
    output: &mut String,
    structure: &rustycad_tkbo::bop_ds::DataStructure,
    edge: i32,
) {
    let blocks = structure.pave_blocks(edge).unwrap();
    output.push_str(&format!("\npartial-edge {edge} {}", blocks.len()));
    for handle in blocks {
        let block = handle.lock().unwrap();
        let (first_vertex, last_vertex) = block.indices();
        let (first, last) = block.range();
        let common = structure
            .common_block(handle)
            .map(|entry| entry.lock().unwrap().pave_blocks().len())
            .unwrap_or(0);
        output.push_str(&format!(
            "\npartial-pb {first_vertex} {last_vertex} {first:.8} {last:.8} {common}"
        ));
    }
}

fn native_overlap_expected() -> String {
    include_str!("../../../tools/oracle/tkbo_pave_filler_ee.partial_overlap.expected")
        .trim()
        .to_owned()
}

fn native_ee_only_expected() -> String {
    include_str!("../../../tools/oracle/tkbo_pave_filler_ee.partial_ee_only.expected")
        .trim()
        .to_owned()
}
