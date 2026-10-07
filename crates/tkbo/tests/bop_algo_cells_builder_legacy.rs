//! Direct translation of src/ModelingAlgorithms/TKBO/GTests/BOPAlgo_CellsBuilder_Test.cxx (3d097a0).
#![allow(non_snake_case)]

use rustycad_tkbo::bop_algo::BOPAlgoCellsBuilder;
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::ncollection::List;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::{self, Axis2, Circle3, Point3};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape, BRepBuilderAPIMakeWire,
};

fn CountShapes(shapes: &[Shape], kind: ShapeEnum) -> usize {
    let mut map = IndexedMapOfShape::default();
    for shape in shapes {
        topexp::map_shapes(shape, &mut map, true, true);
    }
    (1..=map.extent())
        .filter(|&index| map.find_key(index).unwrap().shape_type().unwrap() == kind)
        .count()
}

fn ExpectModified(builder: &BOPAlgoCellsBuilder, shape: &Shape, kind: ShapeEnum, count: usize) {
    assert_eq!(CountShapes(&builder.modified(shape).unwrap(), kind), count);
}

fn AddToResult(
    builder: &mut BOPAlgoCellsBuilder,
    take: &Shape,
    avoid: Option<&Shape>,
    material: i32,
) {
    let take: List<Shape> = [take.clone()].into_iter().collect();
    let avoid: List<Shape> = avoid.into_iter().cloned().collect();
    builder
        .add_to_result(&take, &avoid, material, false)
        .unwrap();
}

fn make_box(corner: Point3) -> Shape {
    BRepPrimAPIMakeBox::at_point(corner, 10.0, 10.0, 10.0)
        .unwrap()
        .shape()
        .unwrap()
        .clone()
}

fn MakeCircleFace(center: Point3, radius: f64) -> Shape {
    let frame = Axis2::new(center, gp::dz()).unwrap();
    let edge = BRepBuilderAPIMakeEdge::from_circ(Circle3::new(frame, radius).unwrap())
        .unwrap()
        .edge()
        .unwrap();
    let wire = BRepBuilderAPIMakeWire::from_edge(&edge)
        .unwrap()
        .wire()
        .unwrap();
    BRepBuilderAPIMakeFace::from_wire(&wire, false)
        .unwrap()
        .face()
        .unwrap()
        .into_shape()
}

#[test]
fn BOPAlgo_CellsBuilderTest__A5_CircleCellsHistory() {
    let face1 = MakeCircleFace(Point3::new(0.0, 0.0, 0.0), 10.0);
    let face2 = MakeCircleFace(Point3::new(10.0, 0.0, 0.0), 10.0);
    let face3 = MakeCircleFace(Point3::new(10.0, 10.0, 0.0), 10.0);
    let mut builder = BOPAlgoCellsBuilder::new();
    builder.set_arguments(
        &[face1.clone(), face2.clone(), face3.clone()]
            .into_iter()
            .collect(),
    );
    builder.options_mut().set_run_parallel(false);
    builder.perform();
    assert!(!builder.options().has_errors());

    AddToResult(&mut builder, &face1, None, 1);
    ExpectModified(&builder, &face1, ShapeEnum::Face, 4);
    ExpectModified(&builder, &face2, ShapeEnum::Face, 2);
    ExpectModified(&builder, &face3, ShapeEnum::Face, 2);
    builder.remove_internal_boundaries().unwrap();
    ExpectModified(&builder, &face1, ShapeEnum::Face, 1);
    ExpectModified(&builder, &face2, ShapeEnum::Face, 1);
    ExpectModified(&builder, &face3, ShapeEnum::Face, 1);

    builder.remove_all_from_result().unwrap();
    AddToResult(&mut builder, &face1, Some(&face3), 1);
    AddToResult(&mut builder, &face2, Some(&face3), 1);
    ExpectModified(&builder, &face1, ShapeEnum::Face, 2);
    ExpectModified(&builder, &face2, ShapeEnum::Face, 2);
    assert!(builder.is_deleted(&face3).unwrap());
    builder.remove_internal_boundaries().unwrap();
    ExpectModified(&builder, &face1, ShapeEnum::Face, 1);
    ExpectModified(&builder, &face2, ShapeEnum::Face, 1);

    builder.remove_all_from_result().unwrap();
    AddToResult(&mut builder, &face1, None, 1);
    AddToResult(&mut builder, &face2, Some(&face1), 2);
    AddToResult(&mut builder, &face3, Some(&face1), 2);
    ExpectModified(&builder, &face1, ShapeEnum::Face, 4);
    ExpectModified(&builder, &face2, ShapeEnum::Face, 4);
    ExpectModified(&builder, &face3, ShapeEnum::Face, 4);
    builder.remove_internal_boundaries().unwrap();
    ExpectModified(&builder, &face1, ShapeEnum::Face, 1);
    ExpectModified(&builder, &face2, ShapeEnum::Face, 2);
    ExpectModified(&builder, &face3, ShapeEnum::Face, 2);
}

// Equivalent to tests/boolean/history/A6.
#[test]
fn BOPAlgo_CellsBuilderTest__A6_BoxCellsHistory() {
    let box1 = make_box(Point3::new(0.0, 0.0, 0.0));
    let box2 = make_box(Point3::new(5.0, 0.0, 0.0));
    let box3 = make_box(Point3::new(2.5, 0.0, 5.0));
    let mut builder = BOPAlgoCellsBuilder::new();
    builder.set_arguments(
        &[box1.clone(), box2.clone(), box3.clone()]
            .into_iter()
            .collect(),
    );
    builder.options_mut().set_run_parallel(false);
    builder.perform();
    assert!(!builder.options().has_errors());
    AddToResult(&mut builder, &box1, None, 1);
    ExpectModified(&builder, &box1, ShapeEnum::Solid, 4);
    ExpectModified(&builder, &box2, ShapeEnum::Solid, 2);
    ExpectModified(&builder, &box3, ShapeEnum::Solid, 2);
    builder.remove_internal_boundaries().unwrap();
    ExpectModified(&builder, &box1, ShapeEnum::Solid, 1);
    ExpectModified(&builder, &box2, ShapeEnum::Solid, 1);
    ExpectModified(&builder, &box3, ShapeEnum::Solid, 1);
    builder.remove_all_from_result().unwrap();
    AddToResult(&mut builder, &box1, Some(&box3), 1);
    AddToResult(&mut builder, &box2, Some(&box3), 1);
    ExpectModified(&builder, &box1, ShapeEnum::Solid, 2);
    ExpectModified(&builder, &box2, ShapeEnum::Solid, 2);
    assert!(builder.is_deleted(&box3).unwrap());
    builder.remove_internal_boundaries().unwrap();
    ExpectModified(&builder, &box1, ShapeEnum::Solid, 1);
    ExpectModified(&builder, &box2, ShapeEnum::Solid, 1);
    builder.remove_all_from_result().unwrap();
    AddToResult(&mut builder, &box1, None, 1);
    AddToResult(&mut builder, &box2, Some(&box1), 2);
    AddToResult(&mut builder, &box3, Some(&box1), 2);
    ExpectModified(&builder, &box1, ShapeEnum::Solid, 4);
    ExpectModified(&builder, &box2, ShapeEnum::Solid, 4);
    ExpectModified(&builder, &box3, ShapeEnum::Solid, 4);
    builder.remove_internal_boundaries().unwrap();
    ExpectModified(&builder, &box1, ShapeEnum::Solid, 1);
    ExpectModified(&builder, &box2, ShapeEnum::Solid, 2);
    ExpectModified(&builder, &box3, ShapeEnum::Solid, 2);
}
