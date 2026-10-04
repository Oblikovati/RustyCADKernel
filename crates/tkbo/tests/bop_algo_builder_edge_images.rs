#![allow(missing_docs)]

use rustycad_tkbo::{bop_algo::BOPAlgoBuilder, bop_ds::DataStructure};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;

#[test]
fn edge_images_use_real_pave_blocks_and_skip_missing_references() {
    let (first, empty, unreferenced) = source_edges();
    let mut structure = initialized_structure(&[&first, &empty, &unreferenced]);
    let image = line_edge(0.0, 2.0);
    let first_index = structure.index(&first);
    append_image_block(&mut structure, first_index, &image);
    let empty_index = structure.index(&empty);
    structure.change_pave_blocks(empty_index).unwrap().clear();
    let mut builder = BOPAlgoBuilder::new();
    builder.fill_images_edges(&structure).unwrap();

    assert_one_image(&builder, &first, &image);
    assert!(builder.images().find(&empty).unwrap().is_empty());
    assert!(builder.images().find(&unreferenced).is_err());
    assert_origin(&builder, &image, &first);
}

#[test]
fn common_edge_blocks_share_the_real_image_and_bind_split_same_domain() {
    let (first, second, first_image, second_image) = common_edge_data();
    let mut structure = initialized_structure(&[&first, &second]);
    attach_common_edge_blocks(&mut structure, &first, &second, &first_image, &second_image);
    let mut builder = BOPAlgoBuilder::new();
    builder.fill_images_edges(&structure).unwrap();

    assert_one_image(&builder, &first, &first_image);
    assert_one_image(&builder, &second, &first_image);
    assert_eq!(builder.origins().find(&first_image).unwrap().len(), 2);
    assert_same_domain(&builder, &second_image, &first_image);
}

fn source_edges() -> (
    rustycad_tkbrep::topods::Edge,
    rustycad_tkbrep::topods::Edge,
    rustycad_tkbrep::topods::Edge,
) {
    (
        line_edge(0.0, 2.0),
        line_edge(1.0, 2.0),
        line_edge(2.0, 2.0),
    )
}

fn common_edge_data() -> (
    rustycad_tkbrep::topods::Edge,
    rustycad_tkbrep::topods::Edge,
    rustycad_tkbrep::topods::Edge,
    rustycad_tkbrep::topods::Edge,
) {
    (
        line_edge(0.0, 2.0),
        line_edge(0.0, 2.0),
        line_edge(0.0, 2.0),
        line_edge(0.0, 2.0),
    )
}

fn attach_common_edge_blocks(
    structure: &mut DataStructure,
    first: &rustycad_tkbrep::topods::Edge,
    second: &rustycad_tkbrep::topods::Edge,
    first_image: &rustycad_tkbrep::topods::Edge,
    second_image: &rustycad_tkbrep::topods::Edge,
) {
    let first_block = append_image_block(structure, structure.index(first), first_image);
    let second_block = append_image_block(structure, structure.index(second), second_image);
    let mut common = rustycad_tkbo::bop_ds::CommonBlock::default();
    common.set_pave_blocks(&[first_block.clone(), second_block.clone()]);
    bind_common_block(structure, first_block, second_block, common);
}

fn bind_common_block(
    structure: &mut DataStructure,
    first: rustycad_tkbo::bop_ds::PaveBlockHandle,
    second: rustycad_tkbo::bop_ds::PaveBlockHandle,
    common: rustycad_tkbo::bop_ds::CommonBlock,
) {
    let common = rustycad_tkbo::bop_ds::common_block_handle(common);
    structure.set_common_block(&first, &common);
    structure.set_common_block(&second, &common);
}

fn assert_one_image(builder: &BOPAlgoBuilder, source: &Shape, expected: &Shape) {
    let images = builder.images().find(source).unwrap();
    assert_eq!(images.len(), 1);
    assert!(images.first().unwrap().is_same(expected));
}

fn assert_origin(builder: &BOPAlgoBuilder, image: &Shape, source: &Shape) {
    let origins = builder.origins().find(image).unwrap();
    assert!(origins.first().unwrap().is_same(source));
}

fn assert_same_domain(builder: &BOPAlgoBuilder, split: &Shape, representative: &Shape) {
    assert!(
        builder
            .shapes_sd()
            .find(split)
            .unwrap()
            .is_same(representative)
    );
}

fn initialized_structure(edges: &[&rustycad_tkbrep::topods::Edge]) -> DataStructure {
    let shapes: Vec<Shape> = edges
        .iter()
        .map(|edge| (*edge).clone().into_shape())
        .collect();
    let mut structure = DataStructure::default();
    structure.set_arguments(&shapes);
    structure.init(1e-7).unwrap();
    structure
}

fn append_image_block(
    structure: &mut DataStructure,
    source_index: i32,
    image: &rustycad_tkbrep::topods::Edge,
) -> rustycad_tkbo::bop_ds::PaveBlockHandle {
    let image_index = structure.append_shape(&image.clone().into_shape());
    let block = structure.change_pave_blocks(source_index).unwrap()[0].clone();
    block
        .lock()
        .expect("pave block lock must not be poisoned")
        .set_edge(image_index);
    block
}

fn line_edge(y: f64, last_x: f64) -> rustycad_tkbrep::topods::Edge {
    let first = Point3::new(0.0, y, 0.0);
    let last = Point3::new(last_x, y, 0.0);
    BRepBuilderAPIMakeEdge::from_points(first, last)
        .unwrap()
        .edge()
        .unwrap()
}
