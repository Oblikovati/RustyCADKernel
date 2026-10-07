//! `BOPAlgo_Tools::PerformCommonBlocks` (both overloads) and `ComputeToleranceOfCB`.
use super::connection::{ConnectionMap, make_blocks};
use crate::bop_ds::{
    CommonBlock, CommonBlockHandle, DataStructure, PaveBlock, PaveBlockHandle,
    PaveBlockHandleIdentity, common_block_handle,
};
use crate::int_tools::Context;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{Edge, Face};
use rustycad_tkernel::ncollection::{DefaultHashPolicy, Map};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomCurve;
use std::sync::{Arc, MutexGuard};

/// Pave blocks linked to the pave blocks they coincide with.
pub type PaveBlockLinks = ConnectionMap<PaveBlockHandle, PaveBlockHandle, PaveBlockHandleIdentity>;
/// Pave blocks linked to indices: the faces they lie on, or the vertices to put on them.
pub type PaveBlockIndexLinks = ConnectionMap<PaveBlockHandle, i32, PaveBlockHandleIdentity>;

/// An empty pave-block connection map; `let mut links = pave_block_links();`.
pub fn pave_block_links() -> PaveBlockLinks {
    PaveBlockLinks::with_hasher(PaveBlockHandleIdentity, 1)
}

/// An empty pave-block/face map; `let mut on_faces = pave_block_index_links();`.
pub fn pave_block_index_links() -> PaveBlockIndexLinks {
    PaveBlockIndexLinks::with_hasher(PaveBlockHandleIdentity, 1)
}

/// Makes one common block per chain of at least two connected pave blocks, reusing the first
/// existing common block of the chain and merging the faces of all of them; then stores the
/// computed tolerance. `perform_common_blocks(&links, &mut ds, Some(&context))?`.
pub fn perform_common_blocks(
    links: &PaveBlockLinks,
    structure: &mut DataStructure,
    context: Option<&Context>,
) -> Result<(), Failure> {
    if links.extent() == 0 {
        return Ok(());
    }
    for chain in make_blocks(links, PaveBlockHandleIdentity)? {
        if chain.len() < 2 {
            continue;
        }
        let common = merged_common_block(structure, &chain)?;
        for block in &chain {
            structure.set_common_block(block, &common);
        }
        let tolerance = compute_tolerance_of_cb(Some(&common), structure, context)?;
        lock_common(&common).set_tolerance(tolerance);
    }
    Ok(())
}

/// Adds the linked faces to each pave block's common block (created on demand), skipping faces
/// already stored, and recomputes its tolerance; `perform_common_blocks_on_faces(&links, &mut ds, None)?`.
pub fn perform_common_blocks_on_faces(
    links: &PaveBlockIndexLinks,
    structure: &mut DataStructure,
    context: Option<&Context>,
) -> Result<(), Failure> {
    for index in 1..=links.extent() {
        let block = links.find_key(index)?.clone();
        let common = structure.common_block(&block).unwrap_or_else(|| {
            let mut created = CommonBlock::new();
            created.add_pave_block(Arc::clone(&block));
            common_block_handle(created)
        });
        let mut new_faces: Vec<i32> = Vec::new();
        for face in links.find_from_index(index)? {
            if !lock_common(&common).faces().contains(face) {
                new_faces.push(*face);
            }
        }
        lock_common(&common).append_faces(&mut new_faces);
        structure.set_common_block(&block, &common);
        let tolerance = compute_tolerance_of_cb(Some(&common), structure, context)?;
        lock_common(&common).set_tolerance(tolerance);
    }
    Ok(())
}

/// Returns the maximal distance (plus own tolerance) from 11 samples of the common block's
/// real pave block to its other edges and faces, or the real edge tolerance when alone.
/// `let tolerance = compute_tolerance_of_cb(Some(&common), &ds, Some(&context))?;`.
pub fn compute_tolerance_of_cb(
    common: Option<&CommonBlockHandle>,
    structure: &DataStructure,
    context: Option<&Context>,
) -> Result<f64, Failure> {
    let Some(common) = common else {
        return Ok(0.0);
    };
    let (real, blocks, faces) = {
        let locked = lock_common(common);
        let real = Arc::clone(locked.pave_block1()?);
        (real, locked.pave_blocks().to_vec(), locked.faces().to_vec())
    };
    let edge = block_edge(structure, &real)?;
    let mut maximum = BRepTool::edge_tolerance(&edge)?;
    if blocks.len() < 2 && faces.is_empty() {
        return Ok(maximum);
    }
    let (curve, _, _) = BRepTool::curve(&edge)?;
    let (first, last) = lock_block(&real).range();
    let step = (last - first) / f64::from(SAMPLE_COUNT + 1);
    let owned_context;
    let context = match context {
        Some(context) => context,
        None => {
            owned_context = Context::default();
            &owned_context
        }
    };
    let sampling = Sampling {
        curve: &curve,
        first,
        step,
    };
    if blocks.len() > 1 {
        maximum = sample_edges(structure, &blocks, &real, &sampling, context, maximum)?;
    }
    sample_faces(structure, &faces, &sampling, context, maximum)
}

/// Native sample count: 11 interior points at a fixed step of the range divided by 12.
const SAMPLE_COUNT: i32 = 11;

struct Sampling<'a> {
    curve: &'a Handle<dyn GeomCurve>,
    first: f64,
    step: f64,
}

impl Sampling<'_> {
    /// The parameters accumulate the step, `aT += aDt`, exactly as the native loop does.
    fn points(&self) -> Result<Vec<rustycad_tkmath::gp::Point3>, Failure> {
        let mut parameter = self.first;
        let mut points = Vec::with_capacity(SAMPLE_COUNT as usize);
        for _ in 0..SAMPLE_COUNT {
            parameter += self.step;
            points.push(self.curve.eval_d0(parameter)?);
        }
        Ok(points)
    }
}

fn sample_edges(
    structure: &DataStructure,
    blocks: &[PaveBlockHandle],
    real: &PaveBlockHandle,
    sampling: &Sampling<'_>,
    context: &Context,
    mut maximum: f64,
) -> Result<f64, Failure> {
    for block in blocks {
        if Arc::ptr_eq(block, real) {
            continue;
        }
        let edge = block_edge(structure, block)?;
        let tolerance = BRepTool::edge_tolerance(&edge)?;
        let projector = context.proj_pc(&edge)?;
        for point in sampling.points()? {
            let mut projection = projector.write();
            projection.perform(point)?;
            if projection.nb_points() > 0 {
                let candidate = tolerance + projection.lower_distance()?;
                if candidate > maximum {
                    maximum = candidate;
                }
            }
        }
    }
    Ok(maximum)
}

fn sample_faces(
    structure: &DataStructure,
    faces: &[i32],
    sampling: &Sampling<'_>,
    context: &Context,
    mut maximum: f64,
) -> Result<f64, Failure> {
    for face in faces {
        let face = Face::unchecked(structure.shape(*face)?.clone());
        let tolerance = BRepTool::face_tolerance(&face)?;
        let projector = context.proj_ps(&face)?;
        for point in sampling.points()? {
            let mut projection = projector.write();
            projection.perform(point)?;
            if projection.nb_points()? > 0 {
                let candidate = tolerance + projection.lower_distance()?;
                if candidate > maximum {
                    maximum = candidate;
                }
            }
        }
    }
    Ok(maximum)
}

fn merged_common_block(
    structure: &DataStructure,
    chain: &[PaveBlockHandle],
) -> Result<CommonBlockHandle, Failure> {
    let mut fence: Map<i32, DefaultHashPolicy> = Map::new(1);
    let mut faces: Vec<i32> = Vec::new();
    let mut reused: Option<CommonBlockHandle> = None;
    for block in chain {
        let Some(existing) = structure.common_block(block) else {
            continue;
        };
        for face in lock_common(&existing).faces() {
            if fence.add(*face) {
                faces.push(*face);
            }
        }
        reused.get_or_insert(existing);
    }
    let common = reused.unwrap_or_else(|| common_block_handle(CommonBlock::new()));
    {
        let mut locked = lock_common(&common);
        locked.set_pave_blocks(chain);
        locked.set_faces(&faces);
    }
    Ok(common)
}

fn block_edge(structure: &DataStructure, block: &PaveBlockHandle) -> Result<Edge, Failure> {
    let index = lock_block(block).original_edge();
    Ok(Edge::unchecked(structure.shape(index)?.clone()))
}

fn lock_common(common: &CommonBlockHandle) -> MutexGuard<'_, CommonBlock> {
    common
        .lock()
        .expect("common-block lock must not be poisoned")
}

fn lock_block(block: &PaveBlockHandle) -> MutexGuard<'_, PaveBlock> {
    block.lock().expect("pave-block lock must not be poisoned")
}
