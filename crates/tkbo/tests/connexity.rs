//! Native connectivity order, repeated-map accumulation and regularity fixtures.
use rustycad_tkbo::bop_tools::{self, ConnexityBlock};
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topexp::Explorer,
    topods::{Compound, Shape},
    toptools::IndexedDataMapOfShapeListOfShape,
};
use rustycad_tkernel::ncollection::BaseAllocator;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeBox;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
#[test]
fn native_connexity_rows_match() {
    let mut rows = Vec::new();
    for (kind, (shape, connection, element)) in fixtures().into_iter().enumerate() {
        inspect(&mut rows, kind, &shape, connection, element);
    }
    assert_eq!(
        rows.join("\n") + "\n",
        include_str!("../../../tools/oracle/tkbo_connexity.expected")
    );
}
fn fixtures() -> Vec<(Shape, ShapeEnum, ShapeEnum)> {
    let box_shape = BRepPrimAPIMakeBox::from_sizes(1., 2., 3.)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    let face =
        Iterator::next(&mut Explorer::new(&box_shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap())
            .unwrap();
    let other = BRepPrimAPIMakeBox::from_sizes(1., 2., 3.)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    let other_face =
        Iterator::next(&mut Explorer::new(&other, ShapeEnum::Face, ShapeEnum::Shape).unwrap())
            .unwrap();
    let mut disconnected = Compound::default();
    let builder = BRepBuilder::default();
    builder.make_compound(&mut disconnected);
    builder.add(&mut disconnected, &face).unwrap();
    builder.add(&mut disconnected, &other_face).unwrap();
    vec![
        (box_shape.clone(), ShapeEnum::Edge, ShapeEnum::Face),
        (face, ShapeEnum::Vertex, ShapeEnum::Edge),
        (box_shape, ShapeEnum::Vertex, ShapeEnum::Edge),
        (empty_compound(), ShapeEnum::Vertex, ShapeEnum::Edge),
        (
            disconnected.into_shape(),
            ShapeEnum::Vertex,
            ShapeEnum::Edge,
        ),
    ]
}
fn empty_compound() -> Shape {
    let mut compound = Compound::default();
    BRepBuilder::default().make_compound(&mut compound);
    compound.into_shape()
}
fn inspect(
    rows: &mut Vec<String>,
    kind: usize,
    shape: &Shape,
    connection: ShapeEnum,
    element: ShapeEnum,
) {
    let elements: Vec<Shape> = Explorer::new(shape, element, ShapeEnum::Shape)
        .unwrap()
        .collect();
    let mut blocks = vec![Vec::new()];
    let mut map = IndexedDataMapOfShapeListOfShape::default();
    for pass in 0..2 {
        bop_tools::make_connexity_blocks(shape, connection, element, &mut blocks, &mut map)
            .unwrap();
        let mut row = format!("blocks {kind} {pass} {}", blocks.len());
        for block in &blocks {
            members(&mut row, block, &elements);
        }
        rows.push(row);
        let mut row = format!("map {kind} {pass} {}", map.extent());
        for index in 1..=map.extent() {
            members(
                &mut row,
                &map.find_from_index(index)
                    .unwrap()
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>(),
                &elements,
            );
        }
        rows.push(row);
    }
    inspect_compounds(rows, kind, shape, connection, element, &elements);
    inspect_regularity(rows, kind, connection, element, &elements);
}
fn inspect_compounds(
    rows: &mut Vec<String>,
    kind: usize,
    shape: &Shape,
    connection: ShapeEnum,
    element: ShapeEnum,
    elements: &[Shape],
) {
    let mut compounds = vec![empty_compound()];
    bop_tools::make_connexity_compounds(shape, connection, element, &mut compounds).unwrap();
    let mut row = format!("compounds {kind} {}", compounds.len());
    for compound in compounds {
        let children: Vec<Shape> = Explorer::new(&compound, element, ShapeEnum::Shape)
            .unwrap()
            .collect();
        members(&mut row, &children, elements);
    }
    rows.push(row);
}
fn inspect_regularity(
    rows: &mut Vec<String>,
    kind: usize,
    connection: ShapeEnum,
    element: ShapeEnum,
    elements: &[Shape],
) {
    for repeat in 0..2 {
        let mut input = elements.to_vec();
        if repeat == 1 && !elements.is_empty() {
            input.push(elements[0].reversed());
        }
        let mut blocks = Vec::new();
        bop_tools::make_connexity_blocks_from_shapes(&input, connection, element, &mut blocks)
            .unwrap();
        let mut row = format!("regular {kind} {repeat} {}", blocks.len());
        for block in blocks {
            row.push_str(&format!(" {}", i32::from(block.is_regular())));
            members(
                &mut row,
                &block.shapes().iter().cloned().collect::<Vec<_>>(),
                elements,
            );
        }
        rows.push(row);
    }
}
fn members(row: &mut String, shapes: &[Shape], elements: &[Shape]) {
    row.push_str(&format!(" {}", shapes.len()));
    for shape in shapes {
        let index = elements
            .iter()
            .position(|element| element.is_same(shape))
            .map_or(-1, |index| index as i32);
        row.push_str(&format!(" {index} {}", shape.orientation() as i32));
    }
}
#[test]
fn block_lists_share_supplied_allocator_and_preserve_mutations() {
    let allocator = BaseAllocator::common();
    let mut block = ConnexityBlock::with_allocator(allocator.clone());
    assert!(block.is_regular());
    assert!(block.shapes().is_empty());
    assert!(block.loops().is_empty());
    assert_eq!(block.shapes().allocator(), &allocator);
    assert_eq!(block.loops().allocator(), &allocator);
    let shape = empty_compound();
    block.change_shapes().append(shape.clone());
    block.change_loops().append(shape.clone());
    block.set_regular(false);
    assert!(!block.is_regular());
    assert!(block.shapes().first().unwrap().is_same(&shape));
    assert!(block.loops().first().unwrap().is_same(&shape));
}
