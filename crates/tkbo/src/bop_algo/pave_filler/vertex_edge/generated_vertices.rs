use super::{append_block_pave, compute_vertex_edge_hit, edge_has_vertex_endpoint};
use crate::bop_ds::PaveBlockHandle;
use rustycad_tkernel::standard::Failure;
pub(super) fn intersect(
    filler: &mut super::BOPAlgoPaveFiller,
    associations: &[(PaveBlockHandle, i32)],
) -> Result<Vec<i32>, Failure> {
    let pairs = unique_edge_vertices(filler, associations)?;
    let mut changed_edges = Vec::new();
    for (edge_index, vertex_index) in pairs {
        intersect_generated_vertex(filler, edge_index, vertex_index, &mut changed_edges)?;
    }
    Ok(changed_edges)
}

fn unique_edge_vertices(
    filler: &super::BOPAlgoPaveFiller,
    associations: &[(PaveBlockHandle, i32)],
) -> Result<Vec<(i32, i32)>, Failure> {
    let structure = filler
        .data_structure
        .as_ref()
        .ok_or_else(missing_structure)?;
    let mut pairs = Vec::new();
    for (block, vertex) in associations {
        append_unique_edge_vertex(structure, block, *vertex, &mut pairs);
    }
    Ok(pairs)
}

fn append_unique_edge_vertex(
    structure: &crate::bop_ds::DataStructure,
    block: &PaveBlockHandle,
    vertex: i32,
    pairs: &mut Vec<(i32, i32)>,
) {
    let edge = block
        .lock()
        .expect("pave-block lock must not be poisoned")
        .original_edge();
    let representative = structure.shape_sd(vertex).unwrap_or(vertex);
    if !pairs.contains(&(edge, representative)) {
        pairs.push((edge, representative));
    }
}

fn intersect_generated_vertex(
    filler: &mut super::BOPAlgoPaveFiller,
    edge_index: i32,
    vertex_index: i32,
    changed_edges: &mut Vec<i32>,
) -> Result<(), Failure> {
    if generated_vertex_is_endpoint(filler, edge_index, vertex_index)? {
        return Ok(());
    }
    let Some((parameter, tolerance)) = generated_vertex_hit(filler, edge_index, vertex_index)?
    else {
        return Ok(());
    };
    append_generated_vertex_pave(
        filler,
        edge_index,
        vertex_index,
        parameter,
        tolerance,
        changed_edges,
    )
}

fn generated_vertex_is_endpoint(
    filler: &super::BOPAlgoPaveFiller,
    edge: i32,
    vertex: i32,
) -> Result<bool, Failure> {
    let structure = filler
        .data_structure
        .as_ref()
        .ok_or_else(missing_structure)?;
    edge_has_vertex_endpoint(structure, edge, vertex)
}

fn generated_vertex_hit(
    filler: &super::BOPAlgoPaveFiller,
    edge_index: i32,
    vertex_index: i32,
) -> Result<Option<(f64, f64)>, Failure> {
    let (vertex, edge, context) = filler.vertex_edge_inputs(vertex_index, edge_index)?;
    compute_vertex_edge_hit(&filler.options, &context, &vertex, &edge)
}

fn append_generated_vertex_pave(
    filler: &mut super::BOPAlgoPaveFiller,
    edge: i32,
    vertex: i32,
    parameter: f64,
    tolerance: f64,
    changed_edges: &mut Vec<i32>,
) -> Result<(), Failure> {
    let updated_vertex = filler.update_vertex(vertex, tolerance)?;
    let blocks = filler.structure_mut()?.pave_blocks(edge)?.to_vec();
    append_to_first_containing_block(&blocks, edge, updated_vertex, parameter, changed_edges);
    Ok(())
}

fn append_to_first_containing_block(
    blocks: &[PaveBlockHandle],
    edge: i32,
    vertex: i32,
    parameter: f64,
    changed_edges: &mut Vec<i32>,
) {
    for block in blocks {
        if !append_block_pave(block, vertex, parameter) {
            continue;
        }
        append_unique(changed_edges, edge);
        return;
    }
}

fn append_unique(edges: &mut Vec<i32>, edge: i32) {
    if !edges.contains(&edge) {
        edges.push(edge);
    }
}

fn missing_structure() -> Failure {
    Failure::new("BOPAlgo_PaveFiller data structure is None; expected initialized EF data")
}
