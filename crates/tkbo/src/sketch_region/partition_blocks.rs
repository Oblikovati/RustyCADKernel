// SPDX-License-Identifier: GPL-2.0-only
//! Own and representative pave parameters remain distinct native observations.
use super::partition_diagnostics::{native_failure, read_failure};
use super::partition_types::*;
use crate::bop_ds::{DataStructure, PaveBlockHandle};
pub(super) fn capture_blocks(
    index: i32,
    structure: &DataStructure,
) -> Result<Vec<PavePartitionObservation>, PartitionReadFailure> {
    let blocks = structure
        .pave_blocks(index)
        .map_err(|cause| native_failure(index, cause))?;
    blocks
        .iter()
        .enumerate()
        .map(|(position, block)| capture_block(index, position, block, structure))
        .collect()
}
fn capture_block(
    index: i32,
    position: usize,
    block: &PaveBlockHandle,
    structure: &DataStructure,
) -> Result<PavePartitionObservation, PartitionReadFailure> {
    let own = pave_scalars(index, position, block)?;
    let real = representative_block(index, position, block, structure)?;
    let real = pave_scalars(index, position, &real)?;
    Ok(pave_observation(own, real))
}
fn pave_observation(own: PaveScalars, real: PaveScalars) -> PavePartitionObservation {
    PavePartitionObservation {
        original_edge: own.0,
        own_edge: own.1,
        real_original_edge: real.0,
        real_edge: real.1,
        own_range: own.2,
        real_range: real.2,
        own_vertices: own.3,
        real_vertices: real.3,
    }
}
type PaveScalars = (i32, i32, [f64; 2], [i32; 2]);
fn pave_scalars(
    index: i32,
    position: usize,
    block: &PaveBlockHandle,
) -> Result<PaveScalars, PartitionReadFailure> {
    let guarded = block.lock().map_err(|_| poisoned_pave(index, position))?;
    let range = guarded.range();
    let vertices = guarded.indices();
    Ok((
        guarded.original_edge(),
        guarded.edge(),
        [range.0, range.1],
        [vertices.0, vertices.1],
    ))
}

fn poisoned_pave(index: i32, position: usize) -> PartitionReadFailure {
    read_failure(
        Some(index),
        Some(position),
        PartitionReadCause::PoisonedPaveBlock,
    )
}

fn representative_block(
    index: i32,
    position: usize,
    block: &PaveBlockHandle,
    structure: &DataStructure,
) -> Result<PaveBlockHandle, PartitionReadFailure> {
    structure.real_pave_block(block).map_err(|cause| {
        let mut failure = native_failure(index, cause);
        failure.block_position = Some(position);
        failure
    })
}
