use rustycad_tkbrep::{
    topexp::{self, Explorer},
    topods::Shape,
    toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape},
};
use rustycad_tkernel::{ncollection::List, standard::Failure};
use rustycad_tkg3d::top_abs::ShapeEnum;
/// Append the first face's component while avoiding selected edges; `make_connexity_block(&faces,&avoided,&mut result)?`.
pub fn make_connexity_block(
    faces: &List<Shape>,
    avoided: &IndexedMapOfShape,
    output: &mut List<Shape>,
) -> Result<(), Failure> {
    let mut adjacent = IndexedDataMapOfShapeListOfShape::default();
    for face in faces.iter() {
        topexp::map_shapes_and_ancestors(face, ShapeEnum::Edge, ShapeEnum::Face, &mut adjacent)?;
    }
    let seed = faces.first().ok_or_else(|| {
        Failure::with_type(
            "Standard_NoSuchObject",
            "MakeConnexityBlock input has 0 faces; expected at least 1",
            "",
        )
    })?;
    let mut frontier = IndexedMapOfShape::default();
    frontier.add(seed.clone());
    let mut connected = IndexedMapOfShape::default();
    while !frontier.is_empty() {
        frontier = next_frontier(&frontier, &adjacent, avoided, &mut connected)?;
    }
    for index in 1..=connected.extent() {
        output.append(connected.find_key(index)?.clone());
    }
    Ok(())
}
fn next_frontier(
    frontier: &IndexedMapOfShape,
    adjacent: &IndexedDataMapOfShapeListOfShape,
    avoided: &IndexedMapOfShape,
    connected: &mut IndexedMapOfShape,
) -> Result<IndexedMapOfShape, Failure> {
    let mut next = IndexedMapOfShape::default();
    for index in 1..=frontier.extent() {
        let face = frontier.find_key(index)?;
        append_face_neighbors(face, adjacent, avoided, connected, &mut next)?;
        connected.add(face.clone());
    }
    Ok(next)
}
fn append_face_neighbors(
    face: &Shape,
    adjacent: &IndexedDataMapOfShapeListOfShape,
    avoided: &IndexedMapOfShape,
    connected: &IndexedMapOfShape,
    next: &mut IndexedMapOfShape,
) -> Result<(), Failure> {
    for edge in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
        if avoided.contains(&edge) {
            continue;
        }
        for neighbor in adjacent.find_from_key(&edge)?.iter() {
            if !neighbor.is_same(face) && !connected.contains(neighbor) {
                next.add(neighbor.clone());
            }
        }
    }
    Ok(())
}
