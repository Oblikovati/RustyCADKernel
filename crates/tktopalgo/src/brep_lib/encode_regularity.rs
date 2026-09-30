// Rust translation of BRepLib::EncodeRegularity and its static helper (BRepLib.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepLib::EncodeRegularity`: the continuity between the faces of every edge shared by
//! two faces (or a seam), stored in the edge.
use super::continuity::continuity_of_faces;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Edge, Face, Shape, ShapeIterator};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, MapOfShape};
use rustycad_tkernel::ncollection::List;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::toploc::Location;

/// The default angular tolerance of `BRepLib::EncodeRegularity` (1.0e-10 rad).
pub const ENCODE_REGULARITY_TOL_ANG: f64 = 1.0e-10;

/// `BRepLib::EncodeRegularity(S, TolAng)`: the regularity of every edge of the shape
/// between its two faces (or the two sides of a seam) that has none; a compound (or a
/// compsolid) is processed child by child, each shape without its location once; an
/// exception stops the shape it came from, as natively caught;
/// `brep_lib::encode_regularity(&shape, 1e-10)?`.
pub fn encode_regularity(shape: &Shape, tol_ang: f64) -> Result<(), Failure> {
    let mut visited = MapOfShape::default();
    encode_in(shape, tol_ang, &mut visited, &MapOfShape::default())
}

/// `BRepLib::EncodeRegularity(S, LE, TolAng)`: the same restricted to the edges of the
/// list (compared without location and orientation);
/// `brep_lib::encode_regularity_of_edges(&shape, &edges, 1e-10)?`.
pub fn encode_regularity_of_edges(
    shape: &Shape,
    edges: &List<Shape>,
    tol_ang: f64,
) -> Result<(), Failure> {
    // Collect edges without location and orientation
    let mut pure_edges = MapOfShape::default();
    for edge in edges.iter() {
        let mut pure: Shape = edge.clone();
        pure.set_location(&Location::default(), false)?;
        pure.set_orientation(Orientation::Forward);
        pure_edges.add(pure);
    }
    let mut visited = MapOfShape::default();
    encode_in(shape, tol_ang, &mut visited, &pure_edges)
}

/// `BRepLib::EncodeRegularity(E, F1, F2, TolAng)`: stores ContinuityOfFaces between the
/// faces when the edge has no regularity above C0 there; an exception of the continuity
/// computation is ignored, as natively caught;
/// `brep_lib::encode_regularity_of_edge(&edge, &f1, &f2, 1e-10)?`.
pub fn encode_regularity_of_edge(
    edge: &Edge,
    face1: &Face,
    face2: &Face,
    tol_ang: f64,
) -> Result<(), Failure> {
    if BRepTool::continuity_on_faces(edge, face1, face2)? > Continuity::C0 {
        return Ok(());
    }
    let encoded = continuity_of_faces(edge, face1, face2, tol_ang).and_then(|continuity| {
        BRepBuilder::default().continuity_on_faces(edge, face1, face2, continuity)
    });
    // catch (Standard_Failure const&) {}
    let _ = encoded;
    Ok(())
}

/// The static `EncodeRegularity`: visits the shape without its location once.
fn encode_in(
    shape: &Shape,
    tol_ang: f64,
    visited: &mut MapOfShape,
    edges_to_encode: &MapOfShape,
) -> Result<(), Failure> {
    let mut located: Shape = shape.clone();
    located.set_location(&Location::default(), false)?; // nullify location
    if !visited.add(located.clone()) {
        return Ok(()); // do not need to process shape twice
    }
    if matches!(
        located.shape_type()?,
        ShapeEnum::Compound | ShapeEnum::CompSolid
    ) {
        let mut children = ShapeIterator::new(&located, true, true);
        while let Ok(child) = children.value() {
            encode_in(&child.clone(), tol_ang, visited, edges_to_encode)?;
            children.next();
        }
        return Ok(());
    }
    // catch (Standard_Failure const&): the exception only ends this shape.
    let _ = encode_edges(&located, tol_ang, edges_to_encode);
    Ok(())
}

/// The edges of one shape shared by two faces, or seams of one face.
fn encode_edges(shape: &Shape, tol_ang: f64, edges_to_encode: &MapOfShape) -> Result<(), Failure> {
    let mut ancestors = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(shape, ShapeEnum::Edge, ShapeEnum::Face, &mut ancestors)?;
    for index in 1..=ancestors.extent() {
        let edge = Edge::unchecked(ancestors.find_key(index)?.clone());
        if !edges_to_encode.is_empty() && !edges_to_encode.contains(&pure_edge(&edge)?) {
            continue; // process only the edges from the list to update their regularity
        }
        if let Some((face1, face2)) = adjacent_faces(&edge, ancestors.find_from_index(index)?)? {
            encode_regularity_of_edge(&edge, &face1, &face2, tol_ang)?;
        }
    }
    Ok(())
}

/// The edge without location, FORWARD.
fn pure_edge(edge: &Edge) -> Result<Shape, Failure> {
    let mut pure: Shape = edge.located(&Location::default(), false)?;
    pure.set_orientation(Orientation::Forward);
    Ok(pure)
}

/// The two faces of the edge: the first two distinct faces of the list, or the first
/// face twice when the edge is a seam of it ("is it a sewing edge?").
fn adjacent_faces(edge: &Edge, faces: &List<Shape>) -> Result<Option<(Face, Face)>, Failure> {
    let mut face1 = Face::default();
    for face in faces.iter() {
        if face1.is_null() {
            face1 = Face::unchecked(face.clone());
        } else if !face1.is_same(face) {
            return Ok(Some((face1, Face::unchecked(face.clone()))));
        }
    }
    if face1.is_null() {
        return Ok(None);
    }
    let orientation: Orientation = edge.orientation();
    let mut explorer = Explorer::new(&face1, ShapeEnum::Edge, ShapeEnum::Shape)?;
    while explorer.more() {
        let current = explorer.current();
        if edge.is_same(current) && orientation != current.orientation() {
            return Ok(Some((face1.clone(), face1)));
        }
        explorer.next()?;
    }
    Ok(None)
}
