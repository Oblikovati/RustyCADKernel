//! Native closed-candidate gluing and its sticky subsequent merge flags.
mod candidates;
use super::{multiple_faces::free_edges, non_manifold::append_faces};
use rustycad_tkbrep::{topods::Shell, toptools::MapOfShape};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
pub(super) fn create_closed_shells(
    shells: &mut Vec<Shell>,
    multiple: &MapOfShape,
) -> Result<(), Failure> {
    let candidates = candidates::glue_candidates(shells, multiple)?;
    let mut index = 0;
    while index + 1 < shells.len() {
        merge_following(shells, index, multiple)?;
        index += 1;
    }
    shells.extend(candidates);
    Ok(())
}
fn merge_following(
    shells: &mut Vec<Shell>,
    index: usize,
    multiple: &MapOfShape,
) -> Result<(), Failure> {
    let mut next = index + 1;
    let (mut add, mut reversed) = (false, false);
    while next < shells.len() {
        let free = free_edges(&shells[index])?;
        if free.is_empty() {
            break;
        }
        let directions = edge_directions(&free, multiple, false);
        if directions.0.is_empty() && directions.1.is_empty() {
            break;
        }
        let other = free_edges(&shells[next])?;
        if other.is_empty() {
            next += 1;
            continue;
        }
        update_merge_flags(&other, multiple, &directions, &mut add, &mut reversed);
        if !add {
            next += 1;
            continue;
        }
        let source = shells.remove(next);
        append_faces(&mut shells[index], &source, reversed)?;
    }
    Ok(())
}
fn update_merge_flags(
    edges: &MapOfShape,
    multiple: &MapOfShape,
    directions: &(MapOfShape, MapOfShape),
    add: &mut bool,
    reversed: &mut bool,
) {
    for edge in edges.iter().filter(|edge| multiple.contains(edge)) {
        if !directions.0.contains(edge) && !directions.1.contains(edge) {
            continue;
        }
        *add = true;
        if (edge.orientation() == Orientation::Forward && directions.0.contains(edge))
            || (edge.orientation() == Orientation::Reversed && directions.1.contains(edge))
        {
            *reversed = true;
        }
    }
}
fn edge_directions(
    edges: &MapOfShape,
    multiple: &MapOfShape,
    stop_at_unshared: bool,
) -> (MapOfShape, MapOfShape) {
    let mut directions = (MapOfShape::default(), MapOfShape::default());
    for edge in edges {
        if !multiple.contains(edge) {
            if stop_at_unshared {
                break;
            }
            continue;
        }
        if edge.orientation() == Orientation::Forward {
            directions.0.add(edge.clone());
        } else if edge.orientation() == Orientation::Reversed {
            directions.1.add(edge.clone());
        }
    }
    directions
}
