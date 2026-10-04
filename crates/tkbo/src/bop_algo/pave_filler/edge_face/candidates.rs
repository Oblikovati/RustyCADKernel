//! Edge/face candidate extraction from the initialized pave-filler data structure.
use super::{BOPAlgoPaveFiller, EdgeFaceCandidate};
use crate::{
    bop_ds::{DataStructure, PaveBlockHandle},
    bop_tools::correct_range_face,
    int_tools::{EdgeFace, Range},
};
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::ShapeEnum;

struct EdgeFaceInput {
    edge_index: i32,
    face_index: i32,
    edge: Edge,
    face: Face,
    face_box: rustycad_tkmath::bnd::BoundingBox3,
}

struct BlockRanges {
    parameters: (f64, f64),
    shrunk: (f64, f64),
}

pub(super) fn selected_edge_face_pairs(
    filler: &mut BOPAlgoPaveFiller,
) -> Result<Vec<(i32, i32)>, Failure> {
    let iterator = filler.iterator.as_mut().ok_or_else(|| {
        Failure::new("BOPAlgo_PaveFiller::PerformEF requires initialized iterator")
    })?;
    iterator.initialize(ShapeEnum::Edge, ShapeEnum::Face);
    let mut pairs = Vec::new();
    while iterator.more() {
        pairs.push(iterator.value()?);
        iterator.next();
    }
    Ok(pairs)
}

pub(super) fn initialize_glued_faces(
    filler: &mut BOPAlgoPaveFiller,
    pairs: &[(i32, i32)],
) -> Result<(), Failure> {
    let structure = filler.data_structure.as_mut().ok_or_else(|| {
        Failure::new("BOPAlgo_PaveFiller::PerformEF requires initialized data structure")
    })?;
    for &(edge, face) in pairs {
        if !structure.shape_info(edge)?.has_flag() {
            structure.change_face_info(face)?;
        }
    }
    Ok(())
}

pub(super) fn edge_face_candidates(
    filler: &mut BOPAlgoPaveFiller,
    pairs: &[(i32, i32)],
) -> Result<Vec<EdgeFaceCandidate>, Failure> {
    let mut candidates = Vec::new();
    for &(edge, face) in pairs {
        collect_face_edge_candidates(filler, edge, face, &mut candidates)?;
    }
    Ok(candidates)
}

fn collect_face_edge_candidates(
    filler: &mut BOPAlgoPaveFiller,
    edge_index: i32,
    face_index: i32,
    candidates: &mut Vec<EdgeFaceCandidate>,
) -> Result<(), Failure> {
    let structure = filler.data_structure.as_mut().ok_or_else(|| {
        Failure::new("BOPAlgo_PaveFiller::PerformEF requires initialized data structure")
    })?;
    let Some(input) = make_edge_face_input(structure, edge_index, face_index)? else {
        return Ok(());
    };
    structure.change_face_info(face_index)?;
    let blocks = structure.change_pave_blocks(edge_index)?.clone();
    collect_candidate_blocks(structure, &input, blocks, candidates)
}

fn make_edge_face_input(
    structure: &DataStructure,
    edge_index: i32,
    face_index: i32,
) -> Result<Option<EdgeFaceInput>, Failure> {
    if structure.shape_info(edge_index)?.has_flag() {
        return Ok(None);
    }
    Ok(Some(EdgeFaceInput {
        edge_index,
        face_index,
        edge: Edge::unchecked(structure.shape(edge_index)?.clone()),
        face: Face::unchecked(structure.shape(face_index)?.clone()),
        face_box: structure.shape_info(face_index)?.box_bounds().clone(),
    }))
}

fn collect_candidate_blocks(
    structure: &DataStructure,
    input: &EdgeFaceInput,
    blocks: Vec<PaveBlockHandle>,
    candidates: &mut Vec<EdgeFaceCandidate>,
) -> Result<(), Failure> {
    for block in blocks {
        collect_candidate_block(structure, input, block, candidates)?;
    }
    Ok(())
}

fn collect_candidate_block(
    structure: &DataStructure,
    input: &EdgeFaceInput,
    block: PaveBlockHandle,
    candidates: &mut Vec<EdgeFaceCandidate>,
) -> Result<(), Failure> {
    let Some(ranges) = candidate_block_bounds(structure, input, &block)? else {
        return Ok(());
    };
    let express_compute = block_endpoints_express_compute(structure, input.face_index, &block)?;
    let mut candidate = make_candidate(input, block, ranges)?;
    candidate.express_compute = express_compute;
    candidates.push(candidate);
    Ok(())
}

fn candidate_block_bounds(
    structure: &DataStructure,
    input: &EdgeFaceInput,
    block: &PaveBlockHandle,
) -> Result<Option<BlockRanges>, Failure> {
    if block_is_already_on_face(structure, input.face_index, block)? {
        return Ok(None);
    }
    let locked = block.lock().expect("pave-block lock must not be poisoned");
    let (shrunk_first, shrunk_last, shrunk_box, _) = locked.shrunk_data();
    if shrunk_box.is_void() || input.face_box.is_out_box(shrunk_box) {
        return Ok(None);
    }
    Ok(Some(BlockRanges {
        parameters: locked.range(),
        shrunk: (shrunk_first, shrunk_last),
    }))
}

fn block_is_already_on_face(
    structure: &DataStructure,
    face: i32,
    block: &PaveBlockHandle,
) -> Result<bool, Failure> {
    let real = structure.real_pave_block(block)?;
    Ok(structure.face_info(face)?.pave_blocks_on().contains(&real))
}

fn block_endpoints_express_compute(
    structure: &DataStructure,
    face: i32,
    block: &PaveBlockHandle,
) -> Result<bool, Failure> {
    let (first, last) = block
        .lock()
        .expect("pave-block lock must not be poisoned")
        .indices();
    let info = structure.face_info(face)?;
    Ok(face_has_classified_vertex(info, first) && face_has_classified_vertex(info, last))
}

fn face_has_classified_vertex(info: &crate::bop_ds::FaceInfo, vertex: i32) -> bool {
    info.vertices_in().contains(vertex) || info.vertices_on().contains(vertex)
}

fn make_candidate(
    input: &EdgeFaceInput,
    block: PaveBlockHandle,
    bounds: BlockRanges,
) -> Result<EdgeFaceCandidate, Failure> {
    let (parameter_range, shrunk_range) = corrected_ranges(input, bounds)?;
    Ok(EdgeFaceCandidate {
        edge_index: input.edge_index,
        face_index: input.face_index,
        edge: input.edge.clone(),
        face: input.face.clone(),
        block,
        parameter_range,
        shrunk_range,
        express_compute: false,
    })
}

fn corrected_ranges(input: &EdgeFaceInput, bounds: BlockRanges) -> Result<(Range, Range), Failure> {
    let parameter_source = Range::new(bounds.parameters.0, bounds.parameters.1);
    let shrunk_source = Range::new(bounds.shrunk.0, bounds.shrunk.1);
    let mut parameter_range = Range::default();
    let mut shrunk_range = Range::default();
    correct_range_face(
        &input.edge,
        &input.face,
        &parameter_source,
        &mut parameter_range,
    )?;
    correct_range_face(&input.edge, &input.face, &shrunk_source, &mut shrunk_range)?;
    Ok((parameter_range, shrunk_range))
}

pub(super) fn configured_edge_face(
    filler: &BOPAlgoPaveFiller,
    candidate: &EdgeFaceCandidate,
) -> EdgeFace {
    let mut solver = EdgeFace::default();
    solver.set_edge(&candidate.edge);
    solver.set_face(&candidate.face);
    solver.set_range(&candidate.parameter_range);
    solver.set_fuzzy_value(filler.options.fuzzy_value());
    solver.use_quick_coincidence_check(candidate.express_compute);
    solver.set_context(&filler.context);
    solver
}
