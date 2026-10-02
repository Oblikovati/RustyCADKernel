//! Face orientation propagation for a rebuilt closed shell.
use rustycad_tkbrep::brep_tools::ReShape;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Builder, Face, Shape, ShapeIterator, Shell};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

use crate::shape_build::ShapeBuildReShape;

/// Repairs shell face directions by propagating edge-use constraints.
pub(super) fn repair_shell_orientations(shape: &Shape) -> Result<Shape, Failure> {
    let shells = shell_shapes(shape)?;
    if shells.is_empty() {
        return Ok(shape.clone());
    }
    let context = Handle::new(ShapeBuildReShape::new());
    for shell in shells {
        let rebuilt = orient_shell(&shell)?;
        if !rebuilt.is_null() {
            context.replace(&shell, &rebuilt)?;
        }
    }
    context.apply(shape, ShapeEnum::Shape)
}

/// Collects each distinct shell below the result.
fn shell_shapes(shape: &Shape) -> Result<Vec<Shape>, Failure> {
    Explorer::new(shape, ShapeEnum::Shell, ShapeEnum::Shape).map(Iterator::collect)
}

/// Rebuilds a shell after solving opposite edge-use constraints.
fn orient_shell(shell_shape: &Shape) -> Result<Shape, Failure> {
    let faces = direct_faces(shell_shape)?;
    if faces.len() < 2 {
        return Ok(Shape::default());
    }
    let flips = orientation_flips(&faces)?;
    if flips.iter().all(|flip| !flip) {
        return Ok(Shape::default());
    }
    rebuild_shell(shell_shape, &faces, &flips)
}

/// Rebuilds one shell with the propagated face reversal choices.
fn rebuild_shell(shell_shape: &Shape, faces: &[Face], flips: &[bool]) -> Result<Shape, Failure> {
    let location = shell_shape.location().clone();
    let orientation = shell_shape.orientation();
    let builder = Builder;
    let mut shell = Shell::default();
    builder.make_shell(&mut shell);
    let mut shell_shape = shell.into_shape();
    add_oriented_faces(&builder, &mut shell_shape, faces, flips)?;
    shell_shape.set_location(&location, false)?;
    Ok(shell_shape.oriented(orientation))
}

/// Adds each face, toggling its orientation when the constraint graph requires it.
fn add_oriented_faces(
    builder: &Builder,
    shell: &mut Shape,
    faces: &[Face],
    flips: &[bool],
) -> Result<(), Failure> {
    for (index, face) in faces.iter().enumerate() {
        let oriented = if flips[index] {
            face.clone().into_shape().reversed()
        } else {
            face.clone().into_shape()
        };
        builder.add(shell, &oriented)?;
    }
    Ok(())
}

/// Reads direct face children from the shell in their stored order.
fn direct_faces(shell: &Shape) -> Result<Vec<Face>, Failure> {
    ShapeIterator::new(shell, true, true)
        .filter(|child| child.shape_type().ok() == Some(ShapeEnum::Face))
        .map(|child| topods::face(&child))
        .collect()
}

/// Solves XOR constraints where equal edge directions require one face reversal.
fn orientation_flips(faces: &[Face]) -> Result<Vec<bool>, Failure> {
    let edge_uses = faces
        .iter()
        .map(face_edges)
        .collect::<Result<Vec<_>, _>>()?;
    let mut graph = vec![Vec::new(); faces.len()];
    add_pair_constraints(&edge_uses, &mut graph);
    propagate_flips(&graph)
}

/// Adds the XOR constraint for each shared edge between a pair of faces.
fn add_pair_constraints(edge_uses: &[Vec<(Shape, Orientation)>], graph: &mut [Vec<(usize, bool)>]) {
    for first in 0..edge_uses.len() {
        add_constraints_for_face(first, edge_uses, graph);
    }
}

/// Connects one face to later faces that use the same edge.
fn add_constraints_for_face(
    first: usize,
    edge_uses: &[Vec<(Shape, Orientation)>],
    graph: &mut [Vec<(usize, bool)>],
) {
    for second in first + 1..edge_uses.len() {
        add_constraints_for_pair(first, second, edge_uses, graph);
    }
}

/// Stores opposite-use parity constraints for each shared edge of a face pair.
fn add_constraints_for_pair(
    first: usize,
    second: usize,
    edge_uses: &[Vec<(Shape, Orientation)>],
    graph: &mut [Vec<(usize, bool)>],
) {
    for (edge_a, orientation_a) in &edge_uses[first] {
        let Some((_, orientation_b)) = edge_uses[second]
            .iter()
            .find(|(edge_b, _)| edge_a.is_same(edge_b))
        else {
            continue;
        };
        let reverse = orientation_a == orientation_b;
        graph[first].push((second, reverse));
        graph[second].push((first, reverse));
    }
}

/// Collects edge identities with their cumulative orientations on a face.
fn face_edges(face: &Face) -> Result<Vec<(Shape, Orientation)>, Failure> {
    Explorer::new(
        &face.clone().into_shape(),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )
    .map(|edges| {
        edges
            .map(|edge| (edge.clone(), edge.orientation()))
            .collect()
    })
}

/// Propagates face reversal parity through every connected component.
fn propagate_flips(graph: &[Vec<(usize, bool)>]) -> Result<Vec<bool>, Failure> {
    let mut flips = vec![None; graph.len()];
    for root in 0..graph.len() {
        if flips[root].is_some() {
            continue;
        }
        flips[root] = Some(false);
        let mut queue = vec![root];
        while let Some(face) = queue.pop() {
            let current = flips[face].unwrap_or(false);
            for &(neighbor, reverse) in &graph[face] {
                let expected = current ^ reverse;
                if let Some(existing) = flips[neighbor] {
                    if existing != expected {
                        return Err(Failure::new(
                            "ShapeFix_Shell::FixFaceOrientation found inconsistent edge constraints",
                        ));
                    }
                    continue;
                }
                flips[neighbor] = Some(expected);
                queue.push(neighbor);
            }
        }
    }
    Ok(flips.into_iter().map(Option::unwrap_or_default).collect())
}
