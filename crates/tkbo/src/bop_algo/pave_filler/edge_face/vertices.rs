//! Fuses coincident vertex hits into common topology for intersecting edge blocks.
use super::PendingEFVertex;
use crate::bop_ds::{Pave, PaveBlockHandle};
use crate::bop_tools::{compute_vertices, make_vertex};
use rustycad_tkbrep::topods::Vertex;
use rustycad_tkernel::{ncollection::List, standard::Failure};

pub(super) fn append_pave(block: &PaveBlockHandle, pave: Pave) {
    block
        .lock()
        .expect("pave-block lock must not be poisoned")
        .append_ext_pave1(pave);
}

pub(super) fn pending_vertex_groups(
    pending: &[PendingEFVertex],
    fuzzy: f64,
) -> Result<Vec<Vec<usize>>, Failure> {
    let mut groups = Vec::new();
    for current in 0..pending.len() {
        let joined = overlapping_groups(current, &groups, pending, fuzzy)?;
        merge_pending_groups(&mut groups, current, &joined);
    }
    Ok(groups)
}

fn overlapping_groups(
    current: usize,
    groups: &[Vec<usize>],
    pending: &[PendingEFVertex],
    fuzzy: f64,
) -> Result<Vec<usize>, Failure> {
    let mut joined = Vec::new();
    for (index, group) in groups.iter().enumerate() {
        if group_intersects(current, group, pending, fuzzy)? {
            joined.push(index);
        }
    }
    Ok(joined)
}

fn group_intersects(
    current: usize,
    group: &[usize],
    pending: &[PendingEFVertex],
    fuzzy: f64,
) -> Result<bool, Failure> {
    for other in group {
        if compute_vertices(&pending[current].vertex, &pending[*other].vertex, fuzzy)? == 0 {
            return Ok(true);
        }
    }
    Ok(false)
}

fn merge_pending_groups(groups: &mut Vec<Vec<usize>>, current: usize, joined: &[usize]) {
    if joined.is_empty() {
        groups.push(vec![current]);
        return;
    }
    let keep = joined[0];
    groups[keep].push(current);
    for index in joined.iter().skip(1).rev() {
        let absorbed = groups.remove(*index);
        groups[keep].extend(absorbed);
    }
}

pub(super) fn fused_vertex(
    pending: &[PendingEFVertex],
    group: &[usize],
) -> Result<Vertex, Failure> {
    let mut shapes: List<rustycad_tkbrep::topods::Shape> = List::default();
    for index in group {
        shapes.append(pending[*index].vertex.clone().into_shape());
    }
    let mut vertex = Vertex::default();
    make_vertex(&shapes, &mut vertex)?;
    Ok(vertex)
}

pub(super) fn append_unique(indices: &mut Vec<i32>, index: i32) {
    if !indices.contains(&index) {
        indices.push(index);
    }
}
