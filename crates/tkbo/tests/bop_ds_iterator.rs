//! Native-backed pair preparation and traversal for the BOPDS broad phase.
use rustycad_tkbo::bop_ds::{BOPDSIterator, DataStructure};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

#[test]
fn iterator_prepare_matches_native_overlapping_box_pairs() {
    let mut structure = DataStructure::default();
    structure.set_arguments(&[
        box_shape(Point3::default()),
        box_shape(Point3::new(0.5, 0.0, 0.0)),
    ]);
    structure.init(1.0e-7).unwrap();
    let mut iterator = BOPDSIterator::default();
    iterator.prepare(&structure).unwrap();
    assert_eq!(native_pair_output(&mut iterator).trim(), expected().trim());
}

#[test]
fn iterator_initialize_and_block_length_follow_selected_list() {
    let mut structure = DataStructure::default();
    structure.set_arguments(&[
        box_shape(Point3::default()),
        box_shape(Point3::new(0.5, 0.0, 0.0)),
    ]);
    structure.init(1.0e-7).unwrap();
    let mut iterator = BOPDSIterator::default();
    iterator.prepare(&structure).unwrap();
    iterator.initialize(ShapeEnum::Vertex, ShapeEnum::Edge);
    assert_eq!(iterator.expected_length(), 8);
    assert_eq!(iterator.block_length(), 4);
    iterator.set_run_parallel(true);
    assert!(iterator.run_parallel());
}

#[test]
fn preparing_again_clears_the_previous_pair_cursor() {
    let mut structure = DataStructure::default();
    structure.set_arguments(&[
        box_shape(Point3::default()),
        box_shape(Point3::new(0.5, 0.0, 0.0)),
    ]);
    structure.init(1.0e-7).unwrap();
    let mut iterator = BOPDSIterator::default();
    iterator.prepare(&structure).unwrap();
    iterator.initialize(ShapeEnum::Vertex, ShapeEnum::Edge);
    assert!(iterator.more());
    iterator.prepare(&structure).unwrap();
    assert!(!iterator.more());
    assert_eq!(iterator.expected_length(), 0);
}

fn box_shape(point: Point3) -> rustycad_tkbrep::topods::Shape {
    let mut maker = BRepPrimAPIMakeBox::at_point(point, 1.0, 1.0, 1.0).unwrap();
    maker.shape().unwrap().clone()
}

fn native_pair_output(iterator: &mut BOPDSIterator) -> String {
    let types = [
        ShapeEnum::Vertex,
        ShapeEnum::Edge,
        ShapeEnum::Face,
        ShapeEnum::Solid,
        ShapeEnum::Shell,
        ShapeEnum::Wire,
        ShapeEnum::Compound,
    ];
    let mut output = String::new();
    for first in types {
        for second in types {
            append_type_pairs(&mut output, iterator, first, second);
        }
    }
    output
}

fn append_type_pairs(
    output: &mut String,
    iterator: &mut BOPDSIterator,
    first: ShapeEnum,
    second: ShapeEnum,
) {
    iterator.initialize(first, second);
    output.push_str(&format!("{},{}:", first as i32, second as i32));
    while iterator.more() {
        let (left, right) = iterator.value().unwrap();
        output.push_str(&format!("{left}-{right},"));
        iterator.next();
    }
    output.push('\n');
}

fn expected() -> &'static str {
    include_str!("../../../tools/oracle/tkbo_bopds_iterator.expected")
}
