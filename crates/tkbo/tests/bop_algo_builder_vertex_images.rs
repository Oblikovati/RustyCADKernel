#![allow(missing_docs)]

use rustycad_tkbo::{bop_algo::BOPAlgoBuilder, bop_ds::DataStructure};
use rustycad_tkbrep::{topods::Shape, toptools::ShapeMapHasher};
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeVertex;

#[test]
fn fill_vertex_images_records_direct_same_domain_images_and_origins() {
    let source = vertex(0.0).into_shape();
    let intermediate = vertex(1.0).into_shape();
    let representative = vertex(2.0).into_shape();
    let mut structure = DataStructure::default();
    let source_index = structure.append_shape(&source);
    let intermediate_index = structure.append_shape(&intermediate);
    let representative_index = structure.append_shape(&representative);
    structure.add_shape_sd(source_index, intermediate_index);
    structure.add_shape_sd(intermediate_index, representative_index);

    let mut builder = BOPAlgoBuilder::new();
    builder.fill_images_vertices(&structure).unwrap();

    assert_single_mapping(builder.images(), &source, &intermediate);
    assert_single_mapping(builder.images(), &intermediate, &representative);
    assert_single_mapping(builder.origins(), &intermediate, &source);
    assert_single_mapping(builder.origins(), &representative, &intermediate);
    assert!(
        builder
            .shapes_sd()
            .find(&source)
            .unwrap()
            .is_same(&intermediate)
    );
    assert!(
        builder
            .shapes_sd()
            .find(&intermediate)
            .unwrap()
            .is_same(&representative)
    );
}

#[test]
fn clear_removes_vertex_image_state_and_keeps_builder_configuration() {
    let source = vertex(0.0).into_shape();
    let representative = vertex(0.0).into_shape();
    let mut structure = DataStructure::default();
    let source_index = structure.append_shape(&source);
    let representative_index = structure.append_shape(&representative);
    structure.add_shape_sd(source_index, representative_index);
    let mut builder = BOPAlgoBuilder::new();
    builder.fill_images_vertices(&structure).unwrap();
    builder.set_check_inverted(false);

    builder.clear();

    assert!(builder.images().is_empty());
    assert!(builder.origins().is_empty());
    assert!(builder.shapes_sd().is_empty());
    assert!(!builder.check_inverted());
}

fn vertex(x: f64) -> rustycad_tkbrep::topods::Vertex {
    BRepBuilderAPIMakeVertex::new(Point3::new(x, 0.0, 0.0))
        .unwrap()
        .vertex()
        .unwrap()
}

fn assert_single_mapping(
    map: &rustycad_tkernel::ncollection::DataMap<
        Shape,
        rustycad_tkernel::ncollection::List<Shape>,
        ShapeMapHasher,
    >,
    key: &Shape,
    expected: &Shape,
) {
    let values = map.find(key).unwrap();
    assert_eq!(values.len(), 1);
    assert!(values.first().unwrap().is_same(expected));
}
