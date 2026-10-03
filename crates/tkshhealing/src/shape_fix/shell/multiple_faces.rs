//! Attach faces bounded only by multiply shared edges, preserving native order.
use super::build_groups::{ShellGroups, build_shell_group};
use super::groups::ShapeMap;
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topexp::{self, Explorer},
    topods::{Face, Shape, ShapeIterator, Shell},
    toptools::{IndexedDataMapOfShapeListOfShape, MapOfShape},
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
pub(super) fn add_multiple_faces(
    source: &Shell,
    faces: &mut Vec<Face>,
    multiple: &MapOfShape,
    groups: &mut ShellGroups,
    non_manifold: bool,
) -> Result<bool, Failure> {
    let mut ancestors = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(source, ShapeEnum::Edge, ShapeEnum::Face, &mut ancestors)?;
    for face in faces.iter() {
        groups.face_shells.unbind(face);
    }
    let candidates = select_candidates(faces, multiple)?;
    let mut changed = false;
    for candidate in candidates {
        let free = free_edges(&candidate)?;
        if free.is_empty() {
            continue;
        }
        let choices = matching_shells(
            &free,
            multiple,
            &ancestors,
            groups,
            non_manifold,
            &candidate,
        )?;
        if choices.is_empty() {
            continue;
        }
        changed = true;
        attach_candidate(&candidate, &choices, &mut groups.shells, non_manifold)?;
        let added =
            Explorer::new(&candidate, ShapeEnum::Face, ShapeEnum::Shape)?.collect::<Vec<_>>();
        faces.retain(|face| !added.iter().any(|part| part.is_same(face)));
    }
    Ok(changed)
}
fn select_candidates(faces: &[Face], multiple: &MapOfShape) -> Result<Vec<Shape>, Failure> {
    let (mut chosen, mut possible) = (Vec::new(), Vec::new());
    for face in faces {
        classify_wires(face, multiple, &mut chosen, &mut possible);
    }
    let mut trial = ShellGroups::default();
    build_shell_group(&mut possible, &MapOfShape::default(), &mut trial)?;
    if trial.changed {
        for shell in trial.shells {
            let free = free_edges(&shell)?;
            if !free.is_empty() && free.iter().all(|edge| multiple.contains(edge)) {
                chosen.push(shell.into_shape());
            }
        }
    }
    Ok(chosen)
}
fn classify_wires(
    face: &Face,
    multiple: &MapOfShape,
    chosen: &mut Vec<Shape>,
    possible: &mut Vec<Face>,
) {
    let mut shared_count = 0;
    for wire in ShapeIterator::new(face, false, true) {
        let edges = ShapeIterator::new(&wire, false, true).collect::<Vec<_>>();
        shared_count += edges.iter().filter(|edge| multiple.contains(edge)).count();
        if shared_count == 0 {
            continue;
        }
        if shared_count == edges.len() {
            chosen.push(face.clone().into_shape());
        } else {
            possible.push(face.clone());
        }
    }
}
pub(super) fn free_edges(shape: &Shape) -> Result<MapOfShape, Failure> {
    let mut edges = MapOfShape::default();
    for face in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)? {
        for edge in Explorer::new(&face, ShapeEnum::Edge, ShapeEnum::Shape)? {
            if !edges.remove(&edge) {
                edges.add(edge);
            }
        }
    }
    Ok(edges)
}
fn matching_shells(
    free: &MapOfShape,
    multiple: &MapOfShape,
    ancestors: &IndexedDataMapOfShapeListOfShape,
    groups: &mut ShellGroups,
    non_manifold: bool,
    candidate: &Shape,
) -> Result<ShapeMap<bool>, Failure> {
    let mut choices = ShapeMap::default();
    let mut directions = (MapOfShape::default(), MapOfShape::default());
    let mut faces = Vec::new();
    for edge in free.iter().filter(|edge| multiple.contains(edge)) {
        if edge.orientation() == Orientation::Forward {
            directions.0.add(edge.clone());
        } else {
            directions.1.add(edge.clone());
        }
        faces.extend(ancestors.find_from_key(edge)?.iter().cloned());
    }
    for face in faces {
        let Some(shell) = groups.face_shells.seek(&face) else {
            continue;
        };
        if choices.is_bound(shell) || (!non_manifold && BRepTool::is_closed(shell)?) {
            continue;
        }
        let boundary = free_edges(shell)?;
        if !free.iter().all(|edge| boundary.contains(edge)) {
            continue;
        }
        let (direct, reversed) = count_orientation(&boundary, multiple, &directions);
        if direct > 0 && reversed > 0 {
            groups.errors.push(candidate.clone());
            continue;
        }
        if direct > 0 || reversed > 0 {
            choices.bind(shell.clone().into_shape(), reversed > 0);
        }
    }
    Ok(choices)
}
fn count_orientation(
    boundary: &MapOfShape,
    multiple: &MapOfShape,
    directions: &(MapOfShape, MapOfShape),
) -> (usize, usize) {
    let (mut direct, mut reversed) = (0, 0);
    for edge in boundary.iter().filter(|edge| multiple.contains(edge)) {
        let orientation = edge.orientation();
        if (orientation == Orientation::Forward && directions.0.contains(edge))
            || (orientation == Orientation::Reversed && directions.1.contains(edge))
        {
            reversed += 1;
        } else if (orientation == Orientation::Forward && directions.1.contains(edge))
            || (orientation == Orientation::Reversed && directions.0.contains(edge))
        {
            direct += 1;
        }
    }
    (direct, reversed)
}
fn attach_candidate(
    candidate: &Shape,
    choices: &ShapeMap<bool>,
    shells: &mut Vec<Shell>,
    non_manifold: bool,
) -> Result<(), Failure> {
    let mut first: Option<(usize, bool)> = None;
    let mut index = 0;
    while index < shells.len() {
        let Some(&reverse) = choices.seek(&shells[index]) else {
            index += 1;
            continue;
        };
        if let Some((first_index, first_reverse)) = first.filter(|_| non_manifold) {
            let source = shells[index].clone();
            for face in ShapeIterator::new(&source, false, true) {
                let face = if reverse != first_reverse {
                    face.reversed()
                } else {
                    face
                };
                BRepBuilder::default().add(&mut shells[first_index], &face)?;
            }
            shells.remove(index);
            continue;
        }
        if first.is_none() {
            first = Some((index, reverse));
        }
        let addition = if reverse {
            candidate.reversed()
        } else {
            candidate.clone()
        };
        for face in Explorer::new(&addition, ShapeEnum::Face, ShapeEnum::Shape)? {
            BRepBuilder::default().add(&mut shells[index], &face)?;
        }
        index += 1;
    }
    Ok(())
}
