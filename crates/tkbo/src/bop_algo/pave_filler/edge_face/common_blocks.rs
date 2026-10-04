//! EF common-block face membership and the native sampled tolerance calculation.
use crate::bop_ds::{
    CommonBlock, CommonBlockHandle, DataStructure, PaveBlockHandle, common_block_handle,
};
use crate::int_tools::{Cached, Context};
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Edge, Face},
};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkgeomalgo::geom_api::{ProjectPointOnCurve, ProjectPointOnSurf};
use rustycad_tkmath::gp::Point3;
use std::sync::Arc;

type CurveSample = (Handle<dyn GeomCurve>, (f64, f64));

pub(super) struct PendingCommonFace {
    block: PaveBlockHandle,
    face: i32,
}

impl PendingCommonFace {
    pub(super) fn new(block: PaveBlockHandle, face: i32) -> Self {
        Self { block, face }
    }
}

pub(super) fn apply_pending_common_faces(
    structure: &mut DataStructure,
    pending: &[PendingCommonFace],
    context: &Context,
) -> Result<(), Failure> {
    for (block, faces) in grouped_faces(pending) {
        apply_common_face_group(structure, &block, &faces, context)?;
    }
    Ok(())
}

fn grouped_faces(pending: &[PendingCommonFace]) -> Vec<(PaveBlockHandle, Vec<i32>)> {
    let mut groups: Vec<(PaveBlockHandle, Vec<i32>)> = Vec::new();
    for item in pending {
        let Some((_, faces)) = groups
            .iter_mut()
            .find(|(block, _)| Arc::ptr_eq(block, &item.block))
        else {
            groups.push((Arc::clone(&item.block), vec![item.face]));
            continue;
        };
        if !faces.contains(&item.face) {
            faces.push(item.face);
        }
    }
    groups
}

fn apply_common_face_group(
    structure: &mut DataStructure,
    block: &PaveBlockHandle,
    faces: &[i32],
    context: &Context,
) -> Result<(), Failure> {
    let common = find_or_create_common_block(structure, block);
    for face in faces {
        append_new_face(&common, *face);
    }
    structure.set_common_block(block, &common);
    let tolerance = compute_common_block_tolerance(structure, &common, context)?;
    common
        .lock()
        .expect("common-block lock must not be poisoned")
        .set_tolerance(tolerance);
    Ok(())
}

pub(super) fn common_block_vertices(structure: &DataStructure) -> Vec<(i32, f64)> {
    let mut visited: Vec<PaveBlockHandle> = Vec::new();
    let mut vertices = Vec::new();
    for blocks in structure.pave_blocks_pool() {
        for block in blocks {
            collect_common_block_vertices(structure, block, &mut visited, &mut vertices);
        }
    }
    vertices
}

fn collect_common_block_vertices(
    structure: &DataStructure,
    block: &PaveBlockHandle,
    visited: &mut Vec<PaveBlockHandle>,
    vertices: &mut Vec<(i32, f64)>,
) {
    let Some(common) = structure.common_block(block) else {
        return;
    };
    let Some((real, first, last, tolerance)) = common_block_vertex_data(&common) else {
        return;
    };
    if visited.iter().any(|seen| Arc::ptr_eq(seen, &real)) {
        return;
    }
    visited.push(real);
    if tolerance > 0.0 {
        vertices.extend([(first, tolerance), (last, tolerance)]);
    }
}

fn common_block_vertex_data(
    common: &CommonBlockHandle,
) -> Option<(PaveBlockHandle, i32, i32, f64)> {
    let common = common
        .lock()
        .expect("common-block lock must not be poisoned");
    let real = Arc::clone(common.pave_block1().ok()?);
    let (first, last) = real
        .lock()
        .expect("pave-block lock must not be poisoned")
        .indices();
    Some((real, first, last, common.tolerance()))
}

fn find_or_create_common_block(
    structure: &DataStructure,
    block: &PaveBlockHandle,
) -> CommonBlockHandle {
    if let Some(common) = structure.common_block(block) {
        return common;
    }
    let mut common = CommonBlock::default();
    common.set_pave_blocks(std::slice::from_ref(block));
    common_block_handle(common)
}

fn append_new_face(common: &CommonBlockHandle, face: i32) {
    let mut common = common
        .lock()
        .expect("common-block lock must not be poisoned");
    if !common.contains_face(face) {
        common.add_face(face);
    }
}

fn compute_common_block_tolerance(
    structure: &DataStructure,
    common: &CommonBlockHandle,
    context: &Context,
) -> Result<f64, Failure> {
    let snapshot = common_block_snapshot(common)?;
    let edge = block_edge(structure, &snapshot.first_block)?;
    let tolerance = BRepTool::edge_tolerance(&edge)?;
    sample_block_tolerance(structure, &edge, &snapshot, context, tolerance)
}

pub(super) fn block_edge(
    structure: &DataStructure,
    block: &PaveBlockHandle,
) -> Result<Edge, Failure> {
    let index = block
        .lock()
        .expect("pave-block lock must not be poisoned")
        .original_edge();
    Ok(Edge::unchecked(structure.shape(index)?.clone()))
}

struct CommonBlockSnapshot {
    first_block: PaveBlockHandle,
    blocks: Vec<PaveBlockHandle>,
    faces: Vec<i32>,
}

fn common_block_snapshot(common: &CommonBlockHandle) -> Result<CommonBlockSnapshot, Failure> {
    let common = common
        .lock()
        .expect("common-block lock must not be poisoned");
    Ok(CommonBlockSnapshot {
        first_block: Arc::clone(common.pave_block1()?),
        blocks: common.pave_blocks().to_vec(),
        faces: common.faces().to_vec(),
    })
}

fn sample_block_tolerance(
    structure: &DataStructure,
    edge: &Edge,
    snapshot: &CommonBlockSnapshot,
    context: &Context,
    initial: f64,
) -> Result<f64, Failure> {
    if snapshot.blocks.len() == 1 && snapshot.faces.is_empty() {
        return Ok(initial);
    }
    let (curve, range) = sample_curve_range(edge, &snapshot.first_block)?;
    sample_block_entities(structure, &curve, range, snapshot, context, initial)
}

fn sample_block_entities(
    structure: &DataStructure,
    curve: &Handle<dyn GeomCurve>,
    range: (f64, f64),
    snapshot: &CommonBlockSnapshot,
    context: &Context,
    initial: f64,
) -> Result<f64, Failure> {
    let maximum = sample_edge_group(
        structure,
        &snapshot.blocks,
        &snapshot.first_block,
        curve,
        range,
        initial,
        context,
    )?;
    sample_face_group(structure, &snapshot.faces, curve, range, maximum, context)
}

fn sample_curve_range(edge: &Edge, block: &PaveBlockHandle) -> Result<CurveSample, Failure> {
    let (curve, _, _) = BRepTool::curve(edge)?;
    let (first, last) = block
        .lock()
        .expect("pave-block lock must not be poisoned")
        .range();
    Ok((curve, (first, (last - first) / 12.0)))
}

fn sample_edge_group(
    structure: &DataStructure,
    blocks: &[PaveBlockHandle],
    source: &PaveBlockHandle,
    curve: &Handle<dyn GeomCurve>,
    range: (f64, f64),
    initial: f64,
    context: &Context,
) -> Result<f64, Failure> {
    let mut maximum = initial;
    for block in blocks {
        maximum = sample_other_edge(structure, block, source, curve, range, maximum, context)?;
    }
    Ok(maximum)
}

fn sample_face_group(
    structure: &DataStructure,
    faces: &[i32],
    curve: &Handle<dyn GeomCurve>,
    range: (f64, f64),
    initial: f64,
    context: &Context,
) -> Result<f64, Failure> {
    let mut maximum = initial;
    for face in faces {
        maximum = sample_face(structure, *face, curve, range, maximum, context)?;
    }
    Ok(maximum)
}

fn sample_other_edge(
    structure: &DataStructure,
    other: &PaveBlockHandle,
    source: &PaveBlockHandle,
    curve: &Handle<dyn GeomCurve>,
    sample: (f64, f64),
    maximum: f64,
    context: &Context,
) -> Result<f64, Failure> {
    if Arc::ptr_eq(other, source) {
        return Ok(maximum);
    }
    let edge = block_edge(structure, other)?;
    let tolerance = BRepTool::edge_tolerance(&edge)?;
    let projector = context.proj_pc(&edge)?;
    project_edge_samples(&projector, curve, sample, tolerance, maximum)
}

fn project_edge_samples(
    projector: &Cached<ProjectPointOnCurve>,
    curve: &Handle<dyn GeomCurve>,
    range: (f64, f64),
    tolerance: f64,
    mut maximum: f64,
) -> Result<f64, Failure> {
    for point in sampled_points(curve, range)? {
        let mut projection = projector.write();
        projection.perform(point)?;
        if projection.nb_points() > 0 {
            maximum = maximum.max(tolerance + projection.lower_distance()?);
        }
    }
    Ok(maximum)
}

fn sample_face(
    structure: &DataStructure,
    index: i32,
    curve: &Handle<dyn GeomCurve>,
    sample: (f64, f64),
    maximum: f64,
    context: &Context,
) -> Result<f64, Failure> {
    let face = Face::unchecked(structure.shape(index)?.clone());
    let tolerance = BRepTool::face_tolerance(&face)?;
    let projector = context.proj_ps(&face)?;
    project_face_samples(&projector, curve, sample, tolerance, maximum)
}

fn project_face_samples(
    projector: &Cached<ProjectPointOnSurf>,
    curve: &Handle<dyn GeomCurve>,
    range: (f64, f64),
    tolerance: f64,
    mut maximum: f64,
) -> Result<f64, Failure> {
    for point in sampled_points(curve, range)? {
        let mut projection = projector.write();
        projection.perform(point)?;
        if projection.nb_points()? > 0 {
            maximum = maximum.max(tolerance + projection.lower_distance()?);
        }
    }
    Ok(maximum)
}

fn sampled_points(
    curve: &Handle<dyn GeomCurve>,
    (first, step): (f64, f64),
) -> Result<Vec<Point3>, Failure> {
    (1..=11)
        .map(|sample| curve.eval_d0(first + f64::from(sample) * step))
        .collect()
}
