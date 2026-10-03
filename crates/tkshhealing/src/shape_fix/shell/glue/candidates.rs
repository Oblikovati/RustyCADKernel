use super::{append_faces, edge_directions, free_edges};
use crate::shape_fix::shell::{build_groups::empty_shell, groups::ShapeMap};
use rustycad_tkbrep::{
    brep::BRepBuilder,
    topods::{self, ShapeIterator, Shell},
    toptools::MapOfShape,
};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tktopalgo::brep_bnd_lib::BRepBndLib;
pub(super) fn glue_candidates(
    shells: &mut Vec<Shell>,
    multiple: &MapOfShape,
) -> Result<Vec<Shell>, Failure> {
    let mut merged = Vec::new();
    let mut index = 0;
    while index + 1 < shells.len() {
        let edges = free_edges(&shells[index])?;
        let directions = edge_directions(&edges, multiple, true);
        if edges.is_empty() || edges.extent() > directions.0.extent() + directions.1.extent() {
            index += 1;
            continue;
        }
        let choices = select_other_shells(&shells[index + 1..], multiple, &directions)?;
        if choices.is_empty() {
            index += 1;
            continue;
        }
        let candidates = combine_candidates(&shells[index], &choices)?;
        merged.extend(inner_shells(candidates)?);
        let mut next = index + 1;
        while next < shells.len() {
            if choices.is_bound(&shells[next]) {
                shells.remove(next);
            } else {
                next += 1;
            }
        }
        shells.remove(index);
    }
    Ok(merged)
}
fn select_other_shells(
    shells: &[Shell],
    multiple: &MapOfShape,
    directions: &(MapOfShape, MapOfShape),
) -> Result<ShapeMap<bool>, Failure> {
    let mut result = ShapeMap::default();
    for shell in shells {
        let edges = free_edges(shell)?;
        if edges.is_empty() {
            continue;
        }
        if let Some(reverse) = candidate_orientation(&edges, multiple, directions) {
            result.bind(shell.clone().into_shape(), reverse);
        }
    }
    Ok(result)
}
fn candidate_orientation(
    edges: &MapOfShape,
    multiple: &MapOfShape,
    directions: &(MapOfShape, MapOfShape),
) -> Option<bool> {
    let mut reverse = false;
    for edge in edges {
        if !multiple.contains(edge)
            || (!directions.0.contains(edge) && !directions.1.contains(edge))
        {
            return None;
        }
        reverse |= (edge.orientation() == Orientation::Forward && directions.0.contains(edge))
            || (edge.orientation() == Orientation::Reversed && directions.1.contains(edge));
    }
    Some(reverse)
}
fn combine_candidates(first: &Shell, choices: &ShapeMap<bool>) -> Result<Vec<Shell>, Failure> {
    let mut candidates = vec![(first.clone(), false)];
    for (shell, reverse) in choices.items() {
        candidates.push((topods::shell(shell)?, *reverse));
    }
    let mut result = Vec::new();
    for first_index in 0..candidates.len() {
        for second_index in first_index + 1..candidates.len() {
            let reverse = if first_index == 0 {
                candidates[second_index].1
            } else {
                candidates[first_index].1 == candidates[second_index].1
            };
            result.push(combine_pair(
                &candidates[first_index].0,
                &candidates[second_index].0,
                reverse,
            )?);
        }
    }
    Ok(result)
}
fn combine_pair(first: &Shell, second: &Shell, reverse: bool) -> Result<Shell, Failure> {
    let mut shell = empty_shell();
    for face in ShapeIterator::new(first, false, true) {
        BRepBuilder::default().add(&mut shell, &face)?;
    }
    append_faces(&mut shell, second, reverse)?;
    Ok(shell)
}
fn inner_shells(shells: Vec<Shell>) -> Result<Vec<Shell>, Failure> {
    let bounds = shells
        .iter()
        .map(shell_bounds)
        .collect::<Result<Vec<_>, _>>()?;
    let mut remove = vec![false; shells.len()];
    for first in 0..bounds.len() {
        for second in first + 1..bounds.len() {
            if bounds[first] == bounds[second] {
                continue;
            }
            if enclosed(&bounds[first], &bounds[second]) {
                remove[second] = true;
            } else if enclosed(&bounds[second], &bounds[first]) {
                remove[first] = true;
            }
        }
    }
    Ok(shells
        .into_iter()
        .enumerate()
        .filter_map(|(index, shell)| (!remove[index]).then_some(shell))
        .collect())
}
fn shell_bounds(shell: &Shell) -> Result<[f64; 6], Failure> {
    let mut bounds = BoundingBox3::default();
    BRepBndLib::add_close(shell, &mut bounds)?;
    bounds.get()
}
fn enclosed(inner: &[f64; 6], outer: &[f64; 6]) -> bool {
    (0..3).all(|axis| inner[axis] >= outer[axis] && inner[axis + 3] <= outer[axis + 3])
}
