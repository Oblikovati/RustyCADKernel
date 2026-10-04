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

pub(super) fn append_face_common_block(
    structure: &mut DataStructure,
    block: &PaveBlockHandle,
    face: i32,
    context: &Context,
) -> Result<(f64, (i32, i32)), Failure> {
    let common = find_or_create_common_block(structure, block);
    append_new_face(&common, face);
    structure.set_common_block(block, &common);
    let tolerance = compute_common_block_tolerance(structure, &common, context)?;
    common
        .lock()
        .expect("common-block lock must not be poisoned")
        .set_tolerance(tolerance);
    let endpoint_indices = common_block_endpoints(&common)?;
    Ok((tolerance, endpoint_indices))
}

fn common_block_endpoints(common: &CommonBlockHandle) -> Result<(i32, i32), Failure> {
    let common = common
        .lock()
        .expect("common-block lock must not be poisoned");
    let block = common.pave_block1()?;
    let endpoints = block
        .lock()
        .expect("pave-block lock must not be poisoned")
        .indices();
    Ok(endpoints)
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
    let (first_block, blocks, faces) = common_block_snapshot(common)?;
    let edge = block_edge(structure, &first_block)?;
    let tolerance = BRepTool::edge_tolerance(&edge)?;
    if blocks.len() == 1 && faces.is_empty() {
        return Ok(tolerance);
    }
    sample_block_tolerance(
        structure,
        &edge,
        &first_block,
        &blocks,
        &faces,
        context,
        tolerance,
    )
}

fn block_edge(structure: &DataStructure, block: &PaveBlockHandle) -> Result<Edge, Failure> {
    let index = block
        .lock()
        .expect("pave-block lock must not be poisoned")
        .original_edge();
    Ok(Edge::unchecked(structure.shape(index)?.clone()))
}

fn common_block_snapshot(
    common: &CommonBlockHandle,
) -> Result<(PaveBlockHandle, Vec<PaveBlockHandle>, Vec<i32>), Failure> {
    let common = common
        .lock()
        .expect("common-block lock must not be poisoned");
    Ok((
        Arc::clone(common.pave_block1()?),
        common.pave_blocks().to_vec(),
        common.faces().to_vec(),
    ))
}

fn sample_block_tolerance(
    structure: &DataStructure,
    edge: &Edge,
    block: &PaveBlockHandle,
    blocks: &[PaveBlockHandle],
    faces: &[i32],
    context: &Context,
    initial: f64,
) -> Result<f64, Failure> {
    let (curve, range) = sample_curve_range(edge, block)?;
    let maximum = sample_edge_group(structure, blocks, block, &curve, range, initial, context)?;
    sample_face_group(structure, faces, &curve, range, maximum, context)
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
    let edge = other_block_edge(structure, other)?;
    let tolerance = BRepTool::edge_tolerance(&edge)?;
    let projector = context.proj_pc(&edge)?;
    project_edge_samples(&projector, curve, sample, tolerance, maximum)
}

fn other_block_edge(structure: &DataStructure, block: &PaveBlockHandle) -> Result<Edge, Failure> {
    let index = block
        .lock()
        .expect("pave-block lock must not be poisoned")
        .original_edge();
    Ok(Edge::unchecked(structure.shape(index)?.clone()))
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
