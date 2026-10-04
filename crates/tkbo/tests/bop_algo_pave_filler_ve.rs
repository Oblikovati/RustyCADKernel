//! Native differential coverage for the ordinary PaveFiller vertex/edge path.
use rustycad_tkbo::bop_algo::BOPAlgoPaveFiller;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{Compound, Shape, Vertex},
};
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;

#[test]
fn perform_ve_inserts_midpoint_pave_and_splits_edge_blocks() {
    let filler = perform_midpoint_ve(false);

    let structure = filler.data_structure().unwrap();
    let mut actual = format!(
        "{} {} {}",
        structure.nb_shapes(),
        structure.interf_ve_records().len(),
        structure.pave_blocks(3).unwrap().len()
    );
    let interference = structure.interf_ve_records().value(0).unwrap();
    actual.push_str(&format!(
        "\nve {} {} {:.8} {}",
        interference.index1(),
        interference.index2(),
        interference.parameter(),
        interference.index_new()
    ));
    for handle in structure.pave_blocks(3).unwrap() {
        let block = handle.lock().unwrap();
        let (first, last, bounds, splittable) = block.shrunk_data();
        actual.push_str(&format!(
            "\npb {} {} {:.8} {:.8} {:.8} {:.8} {} {}",
            block.indices().0,
            block.indices().1,
            block.range().0,
            block.range().1,
            first,
            last,
            splittable as i32,
            block.ext_paves().len()
        ));
        assert!(!bounds.is_void());
    }
    assert_eq!(
        actual,
        include_str!("../../../tools/oracle/tkbo_pave_filler_ve.expected").trim()
    );
}

#[test]
fn perform_ve_creates_shared_vertex_when_non_destructive_mode_is_enabled() {
    let filler = perform_midpoint_ve(true);
    let structure = filler.data_structure().unwrap();
    let interference = structure.interf_ve_records().value(0).unwrap();
    let blocks = structure.pave_blocks(3).unwrap();
    let mut actual = format!(
        "non-destructive {} {} {}\nve-nd {} {} {:.8} {}",
        structure.nb_shapes(),
        structure.interf_ve_records().len(),
        blocks.len(),
        interference.index1(),
        interference.index2(),
        interference.parameter(),
        interference.index_new()
    );
    append_vertex_box(&mut actual, structure, interference.index_new());
    append_block_summaries(&mut actual, "pb-nd", blocks);
    assert_eq!(
        actual,
        include_str!("../../../tools/oracle/tkbo_pave_filler_ve.nondestructive.expected").trim()
    );
}

#[test]
fn perform_ve_skips_a_micro_edge_whose_shrunk_interval_is_unsplittable() {
    let mut filler = BOPAlgoPaveFiller::new();
    filler.add_argument(&vertex_argument(3.5e-7));
    filler.add_argument(&edge_argument_with_length(7.0e-7));
    filler.options_mut().set_fuzzy_value(1.0e-8);
    filler.initialize().unwrap();
    filler.perform_ve().unwrap();
    let structure = filler.data_structure().unwrap();
    let block = structure.pave_blocks(3).unwrap()[0].lock().unwrap();
    let (first, last, _, splittable) = block.shrunk_data();
    assert_eq!(
        format!(
            "micro {} {} {first:.8} {last:.8}",
            structure.interf_ve_records().len(),
            splittable as i32
        ),
        include_str!("../../../tools/oracle/tkbo_pave_filler_ve.micro.expected").trim()
    );
}

fn append_vertex_box(
    output: &mut String,
    structure: &rustycad_tkbo::bop_ds::DataStructure,
    index: i32,
) {
    let bounds = structure.shape_info(index).unwrap().box_bounds();
    let gap = bounds.gap();
    let bounds = bounds.get().unwrap();
    output.push_str(&format!(
        "\nbox-nd {gap:.8} {:.8} {:.8} {:.8} {:.8} {:.8} {:.8}",
        bounds[0], bounds[1], bounds[2], bounds[3], bounds[4], bounds[5]
    ));
}

fn perform_midpoint_ve(non_destructive: bool) -> BOPAlgoPaveFiller {
    let mut filler = BOPAlgoPaveFiller::new();
    filler.add_argument(&vertex_argument(0.5));
    filler.add_argument(&edge_argument());
    filler.options_mut().set_fuzzy_value(1.0e-6);
    filler.set_non_destructive(non_destructive);
    filler.initialize().unwrap();
    filler.perform_ve().unwrap();
    filler
}

fn vertex_argument(x: f64) -> Shape {
    let mut vertex = Vertex::default();
    BRepBuilder::default()
        .make_vertex_at(&mut vertex, Point3::new(x, 0.0, 0.0), 1.0e-7)
        .unwrap();
    compound_with(vertex.into_shape())
}

fn edge_argument() -> Shape {
    edge_argument_with_length(1.0)
}

fn edge_argument_with_length(length: f64) -> Shape {
    let edge =
        BRepBuilderAPIMakeEdge::from_points(Point3::default(), Point3::new(length, 0.0, 0.0))
            .unwrap()
            .edge()
            .unwrap();
    compound_with(edge.into_shape())
}

fn compound_with(child: Shape) -> Shape {
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    let mut shape = compound.into_shape();
    builder.add(&mut shape, &child).unwrap();
    shape
}

fn append_block_summaries(
    output: &mut String,
    label: &str,
    blocks: &[rustycad_tkbo::bop_ds::PaveBlockHandle],
) {
    for handle in blocks {
        let block = handle.lock().unwrap();
        output.push_str(&format!(
            "\n{label} {} {} {:.8} {:.8}",
            block.indices().0,
            block.indices().1,
            block.range().0,
            block.range().1
        ));
    }
}
