//! `BOPAlgo_ShellSplitter::SplitBlock`: the closed shells of one irregular block of faces.
use super::ShellSplitter;
use super::refine::refine_shell;
use crate::bop_tools::{ConnexityBlock, CoupleOfShape, get_edge_off, get_face_off};
use crate::int_tools::Context;
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{Edge, Face, Shape, ShapeIterator, Shell};
use rustycad_tkbrep::toptools::{IndexedDataMapOfShapeListOfShape, ShapeMapHasher};
use rustycad_tkernel::ncollection::{List, Map};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

/// The faces kept for the shells and the edge/face map of those faces.
struct ConnectedFaces {
    faces: List<Shape>,
    boundary: Map<Shape, ShapeMapHasher>,
    edge_faces: IndexedDataMapOfShapeListOfShape,
}

impl ShellSplitter {
    /// Splits one irregular connexity block of faces into closed shells (`SplitBlock`).
    /// `ShellSplitter::split_block(&mut block)?`.
    pub fn split_block(block: &mut ConnexityBlock) -> Result<(), Failure> {
        let context = Context::default();
        block.change_loops().clear();
        let Some(connected) = connected_faces(block.shapes())? else {
            return Ok(());
        };
        let total = connected.faces.len();
        let mut added: Map<Shape> = Map::new(1);
        for face in connected.faces.iter() {
            if !added.add(face.clone()) {
                continue;
            }
            let (shell, shell_edges) = grow_shell(face, &connected, &mut added, &context)?;
            let mut splits = List::default();
            refine_shell(&shell, &shell_edges, &mut splits)?;
            let mut open_shells = Vec::new();
            for split in splits.iter() {
                if BRepTool::is_closed(split)? {
                    split.set_closed(true)?;
                    block.change_loops().append(split.clone());
                } else {
                    open_shells.push(split.clone());
                }
            }
            if added.extent() == total {
                return Ok(());
            }
            if splits.len() == 1 {
                // Processing the open shell again would not bring any new result.
                continue;
            }
            // Faces of an open shell might be needed by another shell: free them.
            for open in &open_shells {
                for face in ShapeIterator::new(open, true, true) {
                    added.remove(&face);
                }
            }
        }
        Ok(())
    }
}

/// Removes the faces with free edges until none is left to remove; the kept faces in input
/// order with the boundary faces (met once) and their edge/face map, or `None` when no face
/// is kept.
fn connected_faces(shapes: &List<Shape>) -> Result<Option<ConnectedFaces>, Failure> {
    let mut kept: Map<Shape> = Map::new(1);
    for face in shapes.iter() {
        kept.add(face.clone());
    }
    let mut edge_faces = IndexedDataMapOfShapeListOfShape::default();
    loop {
        edge_faces.clear(false);
        for face in shapes.iter() {
            if kept.contains(face) {
                topexp::map_shapes_and_ancestors(
                    face,
                    ShapeEnum::Edge,
                    ShapeEnum::Face,
                    &mut edge_faces,
                )?;
            }
        }
        let before = kept.extent();
        remove_faces_with_free_edges(&edge_faces, &mut kept)?;
        if kept.extent() == before || kept.extent() == 0 {
            break;
        }
    }
    if kept.is_empty() {
        return Ok(None);
    }
    let mut faces = List::default();
    let mut boundary: Map<Shape, ShapeMapHasher> = Map::with_hasher(ShapeMapHasher, 1);
    for face in shapes.iter() {
        if kept.contains(face) {
            faces.append(face.clone());
            if !boundary.add(face.clone()) {
                boundary.remove(face);
            }
        }
    }
    Ok(Some(ConnectedFaces {
        faces,
        boundary,
        edge_faces,
    }))
}

fn remove_faces_with_free_edges(
    edge_faces: &IndexedDataMapOfShapeListOfShape,
    kept: &mut Map<Shape>,
) -> Result<(), Failure> {
    for index in 1..=edge_faces.extent() {
        let edge = Edge::unchecked(edge_faces.find_key(index)?.clone());
        if BRepTool::degenerated(&edge)? || edge.orientation() == Orientation::Internal {
            continue;
        }
        let faces = edge_faces.find_from_index(index)?;
        if faces.len() == 1 {
            kept.remove(faces.iter().next().expect("one face"));
        }
    }
    Ok(())
}

/// A shell grown from `seed` by adding, through each free edge, the neighbour face with
/// the smallest angle; with the edge/face map of the shell.
fn grow_shell(
    seed: &Shape,
    connected: &ConnectedFaces,
    added: &mut Map<Shape>,
    context: &Context,
) -> Result<(Shape, IndexedDataMapOfShapeListOfShape), Failure> {
    let builder = BRepBuilder::default();
    let mut shell = Shell::default();
    builder.make_shell(&mut shell);
    let mut shell = shell.into_shape();
    builder.add(&mut shell, seed)?;
    let mut shell_edges = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(&shell, ShapeEnum::Edge, ShapeEnum::Face, &mut shell_edges)?;
    let mut position = 0;
    while let Some(face) = ShapeIterator::new(&shell, true, true).nth(position) {
        position += 1;
        let face = Face::unchecked(face);
        let is_boundary = connected.boundary.contains(&face);
        for edge in Explorer::new(&face, ShapeEnum::Edge, ShapeEnum::Shape)? {
            let edge = Edge::unchecked(edge);
            let Some(selected) = select_neighbour(
                &edge,
                &face,
                is_boundary,
                connected,
                &shell_edges,
                added,
                context,
            )?
            else {
                continue;
            };
            if added.add(selected.clone().into_shape()) {
                builder.add(&mut shell, &selected)?;
                topexp::map_shapes_and_ancestors(
                    &selected,
                    ShapeEnum::Edge,
                    ShapeEnum::Face,
                    &mut shell_edges,
                )?;
            }
        }
    }
    Ok((shell, shell_edges))
}

/// The face to add to the shell through `edge` of `face`, if any.
fn select_neighbour(
    edge: &Edge,
    face: &Face,
    is_boundary: bool,
    connected: &ConnectedFaces,
    shell_edges: &IndexedDataMapOfShapeListOfShape,
    added: &Map<Shape>,
    context: &Context,
) -> Result<Option<Face>, Failure> {
    // Proceed only with the edges free in this shell.
    if shell_edges.seek(edge).is_some_and(|faces| faces.len() > 1)
        || edge.orientation() == Orientation::Internal
        || BRepTool::degenerated(edge)?
    {
        return Ok(None);
    }
    let candidates = connected.edge_faces.find_from_key(edge)?;
    if candidates.is_empty() {
        return Ok(None);
    }
    let mut couples: List<CoupleOfShape> = List::default();
    let mut inner_ways = 0;
    let mut selected = Face::default();
    for candidate in candidates.iter() {
        if face.is_same(candidate) || added.contains(candidate) {
            continue;
        }
        let candidate = Face::unchecked(candidate.clone());
        let mut edge_off = Edge::default();
        if !get_edge_off(edge, &candidate, &mut edge_off)? {
            continue;
        }
        if is_boundary && !connected.boundary.contains(&candidate) {
            inner_ways += 1;
            selected = candidate.clone();
        }
        let mut couple = CoupleOfShape::new();
        couple.set_shape1(&edge_off);
        couple.set_shape2(&candidate);
        couples.append(couple);
    }
    if couples.is_empty() {
        return Ok(None);
    }
    // Among the adjacent faces choose the one with the minimal angle to the current one.
    if !is_boundary || inner_ways != 1 {
        if couples.len() == 1 {
            selected = Face::unchecked(couples.iter().next().expect("one couple").shape2().clone());
        } else {
            get_face_off(edge, face, &couples, &mut selected, context)?;
        }
    }
    Ok((!selected.is_null()).then_some(selected))
}
