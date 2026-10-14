// SPDX-License-Identifier: GPL-2.0-only
//! Fresh aggregate copying keeps pcurve construction away from published arrangement storage.
use super::{boundaries::capture_candidate, outcome::*};
use crate::{bop_algo::BuilderFace, sketch_region::OwnedArrangement};
use rustycad_tkbrep::{
    topexp,
    topods::{Shape, edge},
    toptools::IndexedMapOfShape,
};
use rustycad_tkernel::ncollection::List;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::Plane3;
use rustycad_tktopalgo::{
    brep_builder_api::{BRepBuilderAPICopy, BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape},
    brep_lib,
};
pub(super) type CandidateParents = Vec<crate::sketch_region::ArrangementRelation>;
/// Build owned diagnostic candidates; `extract_candidates(&arrangement, plane, &mut |_| false)`.
pub(crate) fn extract_candidates(
    owned: &OwnedArrangement,
    plane: Plane3,
    cancelled: &mut dyn FnMut(CandidatePhase) -> bool,
) -> Result<CandidateBatch, CandidateCause> {
    poll(CandidatePhase::Copy, cancelled)?;
    let mut copier = BRepBuilderAPICopy::with_shape(&owned.arrangement, true, true)
        .map_err(CandidateCause::Mapped)?;
    let parents = copy_parents(owned, &mut copier)?;
    poll(CandidatePhase::Generation, cancelled)?;
    let builder = generate_faces(copier.shape().map_err(CandidateCause::Mapped)?, plane)?;
    poll(CandidatePhase::Capture, cancelled)?;
    finish_batch(owned, plane, &builder, &parents)
}
fn finish_batch(
    owned: &OwnedArrangement,
    plane: Plane3,
    builder: &BuilderFace,
    parents: &CandidateParents,
) -> Result<CandidateBatch, CandidateCause> {
    let candidates = capture_faces(builder, parents)?;
    let unrepresented = unrepresented_members(parents, &candidates)?;
    Ok(CandidateBatch {
        plane,
        candidates,
        input_deleted: owned.deleted.clone(),
        unrepresented,
        mapped_report: capture_report(builder),
    })
}
fn capture_faces(
    builder: &BuilderFace,
    parents: &CandidateParents,
) -> Result<Vec<RegionCandidate>, CandidateCause> {
    let mut candidates = Vec::new();
    let mut occurrence = 1;
    for candidate in builder.areas().iter() {
        candidates.push(capture_candidate(candidate, parents, &mut occurrence)?);
    }
    Ok(candidates)
}

pub(super) fn copy_parents(
    owned: &OwnedArrangement,
    copier: &mut BRepBuilderAPICopy,
) -> Result<CandidateParents, CandidateCause> {
    let mut parents = Vec::new();
    for relation in &owned.relations {
        for copied in copier
            .modified(&relation.output)
            .map_err(CandidateCause::Mapped)?
        {
            parents.push(composed_copy_relation(relation, copied));
        }
    }
    Ok(parents)
}
fn prepare_edges(
    shape: &Shape,
    support: &rustycad_tkbrep::topods::Face,
) -> Result<List<Shape>, CandidateCause> {
    let mut edges = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(shape, ShapeEnum::Edge, &mut edges)
        .map_err(CandidateCause::Mapped)?;
    let mut inputs = List::default();
    for index in 1..=edges.extent() {
        let segment = edge(edges.find_key(index).map_err(CandidateCause::Mapped)?)
            .map_err(CandidateCause::Mapped)?;
        brep_lib::build_pcurve_for_edge_on_plane(&segment, support)
            .map_err(CandidateCause::Mapped)?;
        inputs.append(segment.oriented(Orientation::Forward));
        inputs.append(segment.oriented(Orientation::Reversed));
    }
    Ok(inputs)
}
pub(super) fn check_builder(builder: &BuilderFace) -> Result<(), CandidateCause> {
    if !builder.options().has_errors() {
        return Ok(());
    }
    Err(CandidateCause::MappedReport(capture_report(builder)))
}
fn capture_report(builder: &BuilderFace) -> std::sync::Arc<rustycad_tkernel::message::Report> {
    let retained = std::sync::Arc::new(rustycad_tkernel::message::Report::new());
    retained.merge(builder.options().report());
    retained
}

fn poll(
    phase: CandidatePhase,
    cancelled: &mut dyn FnMut(CandidatePhase) -> bool,
) -> Result<(), CandidateCause> {
    if cancelled(phase) {
        return Err(CandidateCause::Cancelled(phase));
    }
    Ok(())
}

pub(crate) fn composed_copy_relation(
    relation: &crate::sketch_region::ArrangementRelation,
    copied: &Shape,
) -> crate::sketch_region::ArrangementRelation {
    let modified = relation
        .preserved
        .iter()
        .chain(&relation.modified)
        .cloned()
        .collect();
    crate::sketch_region::ArrangementRelation {
        output: copied.clone(),
        preserved: Vec::new(),
        modified,
        generated: relation.generated.clone(),
    }
}
pub(super) fn generate_faces(shape: &Shape, plane: Plane3) -> Result<BuilderFace, CandidateCause> {
    let support = BRepBuilderAPIMakeFace::from_plane(plane)
        .map_err(CandidateCause::Mapped)?
        .face()
        .map_err(CandidateCause::Mapped)?;
    let inputs = prepare_edges(shape, &support)?;
    let mut builder = BuilderFace::new();
    builder.set_face(&support);
    builder.set_shapes(&inputs);
    builder.perform().map_err(CandidateCause::Mapped)?;
    check_builder(&builder)?;
    Ok(builder)
}

fn unrepresented_members(
    parents: &CandidateParents,
    candidates: &[RegionCandidate],
) -> Result<CandidateParents, CandidateCause> {
    let mut unrepresented = Vec::new();
    for relation in parents {
        if !unrepresented_leaf(relation, candidates)? {
            continue;
        }
        unrepresented.push(composed_copy_relation(relation, &relation.output));
    }
    Ok(unrepresented)
}

fn unrepresented_leaf(
    relation: &crate::sketch_region::ArrangementRelation,
    candidates: &[RegionCandidate],
) -> Result<bool, CandidateCause> {
    if !rustycad_tkbrep::brep_tools::History::is_supported_type(&relation.output)
        .map_err(CandidateCause::Mapped)?
    {
        return Ok(false);
    }
    Ok(!candidates
        .iter()
        .flat_map(|c| &c.relations)
        .any(|r| r.output.is_same(&relation.output)))
}
