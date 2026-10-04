use super::PendingVertexPart;
use rustycad_tkbrep::topods::Vertex;
use rustycad_tkernel::standard::Failure;

pub(super) fn connected_vertex_groups(
    pending: &[PendingVertexPart],
    fuzzy: f64,
) -> Result<Vec<Vec<usize>>, Failure> {
    let mut groups: Vec<Vec<usize>> = Vec::new();
    for current in 0..pending.len() {
        let mut joined = Vec::new();
        for (group_index, group) in groups.iter().enumerate() {
            if vertex_group_interferes(&pending[current].vertex, group, pending, fuzzy)? {
                joined.push(group_index);
            }
        }
        merge_vertex_group(&mut groups, current, &joined);
    }
    Ok(groups)
}

fn vertex_group_interferes(
    vertex: &Vertex,
    group: &[usize],
    pending: &[PendingVertexPart],
    fuzzy: f64,
) -> Result<bool, Failure> {
    for index in group {
        if crate::bop_tools::compute_vertices(vertex, &pending[*index].vertex, fuzzy)? == 0 {
            return Ok(true);
        }
    }
    Ok(false)
}

fn merge_vertex_group(groups: &mut Vec<Vec<usize>>, current: usize, joined: &[usize]) {
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

pub(super) fn merged_vertex(
    pending: &[PendingVertexPart],
    group: &[usize],
) -> Result<Vertex, Failure> {
    let mut shapes = rustycad_tkernel::ncollection::List::default();
    for index in group {
        shapes.append(pending[*index].vertex.clone().into_shape());
    }
    let mut vertex = Vertex::default();
    crate::bop_tools::make_vertex(&shapes, &mut vertex)?;
    Ok(vertex)
}
