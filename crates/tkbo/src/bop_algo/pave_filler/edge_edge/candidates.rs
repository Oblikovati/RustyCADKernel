use super::EdgeBlockPair;
use crate::bop_algo::pave_filler::BOPAlgoPaveFiller;
use crate::bop_ds::{BOPDSIterator, DataStructure, PaveBlockHandle};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::bnd::BoundingBox3;

pub(super) fn collect(filler: &mut BOPAlgoPaveFiller) -> Result<Vec<EdgeBlockPair>, Failure> {
    let pairs = edge_pairs(filler)?;
    let structure = filler.data_structure.as_ref().ok_or_else(|| {
        Failure::new("BOPAlgo_PaveFiller::PerformEE requires initialized data structure")
    })?;
    collect_block_pairs(structure, &pairs)
}

fn edge_pairs(filler: &mut BOPAlgoPaveFiller) -> Result<Vec<(i32, i32)>, Failure> {
    let iterator = filler.iterator_mut().ok_or_else(|| {
        Failure::new("BOPAlgo_PaveFiller::PerformEE requires initialized iterator")
    })?;
    collect_iterator_pairs(iterator)
}

fn collect_iterator_pairs(iterator: &mut BOPDSIterator) -> Result<Vec<(i32, i32)>, Failure> {
    iterator.initialize(ShapeEnum::Edge, ShapeEnum::Edge);
    let mut pairs = Vec::new();
    while iterator.more() {
        pairs.push(iterator.value()?);
        iterator.next();
    }
    Ok(pairs)
}

fn collect_block_pairs(
    structure: &DataStructure,
    pairs: &[(i32, i32)],
) -> Result<Vec<EdgeBlockPair>, Failure> {
    let mut candidates = Vec::new();
    for (edge1, edge2) in pairs {
        append_edge_pair_candidates(structure, *edge1, *edge2, &mut candidates)?;
    }
    Ok(candidates)
}

fn append_edge_pair_candidates(
    structure: &DataStructure,
    edge1: i32,
    edge2: i32,
    output: &mut Vec<EdgeBlockPair>,
) -> Result<(), Failure> {
    if structure.shape_info(edge1)?.has_flag() || structure.shape_info(edge2)?.has_flag() {
        return Ok(());
    }
    let blocks1 = structure.pave_blocks(edge1)?;
    let blocks2 = structure.pave_blocks(edge2)?;
    for block1 in blocks1 {
        for block2 in blocks2 {
            append_block_pair(structure, edge1, edge2, block1, block2, output)?;
        }
    }
    Ok(())
}

fn append_block_pair(
    structure: &DataStructure,
    edge1: i32,
    edge2: i32,
    block1: &PaveBlockHandle,
    block2: &PaveBlockHandle,
    output: &mut Vec<EdgeBlockPair>,
) -> Result<(), Failure> {
    let (range1, bounds1) = block_bounds(structure, edge1, block1)?;
    let (range2, bounds2) = block_bounds(structure, edge2, block2)?;
    if range1.last() - range1.first() <= rustycad_tkernel::precision::p_confusion()
        || range2.last() - range2.first() <= rustycad_tkernel::precision::p_confusion()
        || !bounds1.intersects(&bounds2)
    {
        return Ok(());
    }
    output.push(EdgeBlockPair {
        edge1,
        edge2,
        block1: block1.clone(),
        block2: block2.clone(),
        range1,
        range2,
    });
    Ok(())
}

fn block_bounds(
    structure: &DataStructure,
    edge: i32,
    block: &PaveBlockHandle,
) -> Result<(crate::int_tools::Range, BoundingBox3), Failure> {
    let locked = block.lock().expect("pave-block lock must not be poisoned");
    let (first, last) = locked.range();
    let (_, _, shrunk_bounds, _) = locked.shrunk_data();
    let bounds = if locked.has_shrunk_data() {
        shrunk_bounds.clone()
    } else {
        structure.shape_info(edge)?.box_bounds().clone()
    };
    Ok((crate::int_tools::Range::new(first, last), bounds))
}
