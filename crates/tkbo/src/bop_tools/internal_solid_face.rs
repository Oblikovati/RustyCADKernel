use super::{compute_state_face, get_edge_on_face, internal_face_between};
use crate::int_tools::Context;
use rustycad_tkbrep::{
    brep::BRepTool,
    topexp::{self, Explorer},
    topods::{Edge, Face, Shape, Solid},
    toptools::{IndexedDataMapOfShapeListOfShape, IndexedMapOfShape},
};
use rustycad_tkernel::{ncollection::List, standard::Failure};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum, State};
/// Classify through adjacent faces, then the solid classifier; `internal_face_in_solid(&face,&solid,&adjacent,1e-7,&context)?`.
pub fn internal_face_in_solid(
    face: &Face,
    solid: &Solid,
    adjacent: &IndexedDataMapOfShapeListOfShape,
    tolerance: f64,
    context: &Context,
) -> Result<bool, Failure> {
    for shape in Explorer::new(face, ShapeEnum::Edge, ShapeEnum::Shape)? {
        let edge = Edge::unchecked(shape);
        if !adjacent.contains(&edge)
            || edge.orientation() == Orientation::Internal
            || BRepTool::degenerated(&edge)?
        {
            continue;
        }
        let Some(state) = classify_adjacent(face, &edge, adjacent.find_from_key(&edge)?, context)?
        else {
            continue;
        };
        if state != 2 {
            return Ok(state == 1);
        }
        break;
    }
    let mut bounds = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(solid, ShapeEnum::Edge, &mut bounds)?;
    Ok(compute_state_face(face, solid, tolerance, &bounds, context)? == State::In)
}
fn classify_adjacent(
    face: &Face,
    edge: &Edge,
    adjacent: &List<Shape>,
    context: &Context,
) -> Result<Option<i32>, Failure> {
    if adjacent.len() == 1 {
        let support = Face::unchecked(
            adjacent
                .first()
                .expect("nonempty adjacency after length check")
                .clone(),
        );
        let mut occurrence = Edge::default();
        get_edge_on_face(edge, &support, &mut occurrence)?;
        if occurrence.orientation() != Orientation::Internal {
            return Ok(None);
        }
        return Ok(Some(internal_face_between(
            face, edge, &support, &support, context,
        )?));
    }
    if adjacent.len() != 2 {
        return Ok(None);
    }
    let first = Face::unchecked(
        adjacent
            .first()
            .expect("nonempty adjacency after length check")
            .clone(),
    );
    let last = Face::unchecked(
        adjacent
            .last()
            .expect("two-face adjacency after length check")
            .clone(),
    );
    let state = internal_face_between(face, edge, &first, &last, context)?;
    Ok((state != 2).then_some(state))
}
