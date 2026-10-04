//! BOPTools_AlgoTools.cxx connectivity overloads, OCCT 3d097a0328e71b826377d4814ab05ec3c3d23871.
use super::ConnexityBlock;
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topexp::{self, Explorer},
    topods::{Compound, Shape},
    toptools::{IndexedDataMapOfShapeListOfShape, MapOfShape},
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
/// Append breadth-first element blocks and ancestor bindings; `make_connexity_blocks(&shape, ShapeEnum::Vertex, ShapeEnum::Edge, &mut blocks, &mut map)?`.
pub fn make_connexity_blocks(
    shape: &Shape,
    connection: ShapeEnum,
    element: ShapeEnum,
    blocks: &mut Vec<Vec<Shape>>,
    map: &mut IndexedDataMapOfShapeListOfShape,
) -> Result<(), Failure> {
    topexp::map_shapes_and_ancestors(shape, connection, element, map)?;
    let mut visited = MapOfShape::default();
    for shape in Explorer::new(shape, element, ShapeEnum::Shape)? {
        if !visited.add(shape.clone()) {
            continue;
        }
        blocks.push(connected_block(shape, connection, map, &mut visited)?);
    }
    Ok(())
}
fn connected_block(
    seed: Shape,
    connection: ShapeEnum,
    map: &IndexedDataMapOfShapeListOfShape,
    visited: &mut MapOfShape,
) -> Result<Vec<Shape>, Failure> {
    let mut block = vec![seed];
    let mut current = 0;
    while current < block.len() {
        let shape = block[current].clone();
        append_neighbors(&shape, connection, map, visited, &mut block)?;
        current += 1;
    }
    Ok(block)
}
fn append_neighbors(
    shape: &Shape,
    connection: ShapeEnum,
    map: &IndexedDataMapOfShapeListOfShape,
    visited: &mut MapOfShape,
    block: &mut Vec<Shape>,
) -> Result<(), Failure> {
    for subshape in Explorer::new(shape, connection, ShapeEnum::Shape)? {
        for neighbor in map.find_from_key(&subshape)?.iter() {
            if visited.add(neighbor.clone()) {
                block.push(neighbor.clone());
            }
        }
    }
    Ok(())
}
/// Append one compound per block; `make_connexity_compounds(&shape, connection, element, &mut compounds)?`.
pub fn make_connexity_compounds(
    shape: &Shape,
    connection: ShapeEnum,
    element: ShapeEnum,
    compounds: &mut Vec<Shape>,
) -> Result<(), Failure> {
    let mut blocks = Vec::new();
    let mut map = IndexedDataMapOfShapeListOfShape::default();
    make_connexity_blocks(shape, connection, element, &mut blocks, &mut map)?;
    for block in blocks {
        compounds.push(compound_from_shapes(&block)?);
    }
    Ok(())
}
fn compound_from_shapes(shapes: &[Shape]) -> Result<Shape, Failure> {
    let builder = BRepBuilder::default();
    let mut compound = Compound::default();
    builder.make_compound(&mut compound);
    for shape in shapes {
        builder.add(&mut compound, shape)?;
    }
    Ok(compound.into_shape())
}
/// Append blocks with repeated shapes and non-two-valent connections marked irregular; `make_connexity_blocks_from_shapes(&shapes, connection, element, &mut blocks)?`.
pub fn make_connexity_blocks_from_shapes(
    shapes: &[Shape],
    connection: ShapeEnum,
    element: ShapeEnum,
    blocks: &mut Vec<ConnexityBlock>,
) -> Result<(), Failure> {
    let (unique, repeated) = unique_and_repeated(shapes);
    let compound = compound_from_shapes(&unique)?;
    let mut connected = Vec::new();
    let mut map = IndexedDataMapOfShapeListOfShape::default();
    make_connexity_blocks(&compound, connection, element, &mut connected, &mut map)?;
    for block in connected {
        blocks.push(classify_block(&block, connection, &map, &repeated)?);
    }
    Ok(())
}
fn unique_and_repeated(shapes: &[Shape]) -> (Vec<Shape>, MapOfShape) {
    let mut visited = MapOfShape::default();
    let mut repeated = MapOfShape::default();
    let mut unique = Vec::new();
    for shape in shapes {
        if visited.add(shape.clone()) {
            unique.push(shape.clone());
        } else {
            repeated.add(shape.clone());
        }
    }
    (unique, repeated)
}
fn classify_block(
    shapes: &[Shape],
    connection: ShapeEnum,
    map: &IndexedDataMapOfShapeListOfShape,
    repeated: &MapOfShape,
) -> Result<ConnexityBlock, Failure> {
    let mut block = ConnexityBlock::default();
    let mut regular = true;
    for shape in shapes {
        if repeated.contains(shape) {
            regular = false;
            block
                .change_shapes()
                .append(shape.oriented(Orientation::Forward));
            block
                .change_shapes()
                .append(shape.oriented(Orientation::Reversed));
            continue;
        }
        block.change_shapes().append(shape.clone());
        if regular {
            regular = has_regular_connections(shape, connection, map)?;
        }
    }
    block.set_regular(regular);
    Ok(block)
}
fn has_regular_connections(
    shape: &Shape,
    connection: ShapeEnum,
    map: &IndexedDataMapOfShapeListOfShape,
) -> Result<bool, Failure> {
    for subshape in Explorer::new(shape, connection, ShapeEnum::Shape)? {
        if map.find_from_key(&subshape)?.len() != 2 {
            return Ok(false);
        }
    }
    Ok(true)
}
