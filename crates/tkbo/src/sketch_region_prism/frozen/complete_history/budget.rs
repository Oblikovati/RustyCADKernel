// SPDX-License-Identifier: GPL-2.0-only
//! Check aggregate stored rows before projection; excess is refused without truncation or allocation.
use super::*;
use crate::sketch_region::{PartitionObservation, RegionCopyContextObservation, RegionContextObservation};
struct HistoryRowBudget { used: usize, maximum: usize }
impl HistoryRowBudget {
    fn add(&mut self, amount: usize) -> HistoryCheck<()> {
        self.used = self.used.checked_add(amount).ok_or(PrismHistoryCause::Bounds(HistoryBoundCause::Exhausted {
            quantity:"stored history rows", actual:usize::MAX, maximum:self.maximum }))?;
        if self.used > self.maximum { return Err(PrismHistoryCause::Bounds(HistoryBoundCause::Exhausted {
            quantity:"stored history rows", actual:self.used, maximum:self.maximum })); }
        Ok(())
    }
    fn relations(&mut self, rows: &[crate::sketch_region::ArrangementRelation]) -> HistoryCheck<()> {
        self.add(rows.len())?;
        for row in rows { self.add(row.preserved.len())?; self.add(row.modified.len())?; self.add(row.generated.len())?; }
        Ok(())
    }
    fn contexts(&mut self, rows: &[crate::sketch_region::RegionObservedContext]) -> HistoryCheck<()> {
        self.add(rows.len())?;
        for row in rows { self.add(row.ancestors.len())?; self.add(row.child_slots.len())?; }
        Ok(())
    }
    fn copy(&mut self, copied: &RegionCopyContextObservation) -> HistoryCheck<()> {
        if let RegionCopyContextObservation::Captured(rows) = copied {
            self.contexts(&rows.source)?; self.contexts(&rows.copied)?; self.add(rows.queries.len())?;
            for query in &rows.queries { self.contexts(std::slice::from_ref(&query.source))?;
                if let Ok(images) = &query.images { self.add(images.len())?; } }
        }
        Ok(())
    }
}
pub(super) fn admit(request: &PrismHistoryRequest<'_>) -> HistoryCheck<()> {
    let mut budget = HistoryRowBudget { used:0, maximum:request.requested.limits.maximum_rows };
    authored(request, &mut budget)?;
    partition(request, &mut budget)?;
    candidate(request, &mut budget)?;
    receipt(request, &mut budget)
}
fn authored(request: &PrismHistoryRequest<'_>, budget: &mut HistoryRowBudget) -> HistoryCheck<()> {
    for count in [request.sources.len(), request.publication.originals.len(), request.publication.sources.len(), request.publication.directed.len(),
        request.copied.selected_members.len(), request.copied.selected_uses.len(), request.copied.copied_uses.len(), request.historical.intermediate.len()] { budget.add(count)?; }
    for stage in &request.historical.intermediate {
        budget.add(stage.members.len())?; budget.add(stage.uses.len())?;
        for row in &stage.uses { budget.contexts(std::slice::from_ref(&row.actual))?; }
    }
    Ok(())
}
fn partition(request: &PrismHistoryRequest<'_>, budget: &mut HistoryRowBudget) -> HistoryCheck<()> {
    let rows = match &request.bridge.mapped.arrangement.partition {
        PartitionObservation::Captured(rows) | PartitionObservation::Incomplete { partial:rows, .. } => rows,
        PartitionObservation::NotReached => return Ok(()),
    };
    budget.add(rows.input_contexts.len())?; budget.add(rows.sources.len())?; budget.add(rows.vertices.len())?;
    for input in &rows.input_contexts { budget.copy(&input.observation)?; }
    for source in &rows.sources { partition_source(source, budget)?; }
    for vertex in &rows.vertices { budget.add(vertex.original_parents.len())?; }
    if let crate::sketch_region::PartitionMemberQueries::Captured { rows, .. } = &rows.member_queries {
        budget.add(rows.len())?;
        for row in rows { for answer in [&row.modified, &row.generated] { if let Ok(images) = answer { budget.add(images.len())?; } } }
    }
    if let crate::sketch_region::ArrangementCompletion::Arranged(rows) = &request.bridge.mapped.arrangement.completion { budget.relations(&rows.relations)?; budget.add(rows.deleted.len())?; }
    Ok(())
}
fn partition_source(source: &crate::sketch_region::SourcePartitionObservation, budget: &mut HistoryRowBudget) -> HistoryCheck<()> {
    budget.add(source.parents.len())?; budget.add(source.source_vertices.len())?; budget.add(source.blocks.len())?;
    if let Some(images) = &source.images { budget.add(images.len())?; }
    if let Some(Ok(space)) = &source.space {
        if let crate::sketch_region_geometry_capture::CapturedMetricCurve::Bezier { poles, weights } = &space.curve {
            budget.add(poles.len())?; if let Some(weights) = weights { budget.add(weights.len())?; }
        }
    }
    Ok(())
}
fn candidate(request: &PrismHistoryRequest<'_>, budget: &mut HistoryRowBudget) -> HistoryCheck<()> {
    let Some(copy) = &request.bridge.mapped.copied else { return Ok(()); };
    budget.copy(&copy.contexts)?; budget.add(copy.links.len())?; budget.add(copy.observation_failures.len())?;
    for row in &copy.links { budget.add(row.preserved.len())?; budget.add(row.modified.len())?; budget.add(row.generated.len())?; }
    match &copy.copied_tree {
        crate::sketch_region_candidates::CandidateCopyTreeObservation::Captured(rows) | crate::sketch_region_candidates::CandidateCopyTreeObservation::Incomplete { members: rows, .. } => budget.add(rows.len())?,
        crate::sketch_region_candidates::CandidateCopyTreeObservation::NotReached => (),
    }
    generated(copy, budget)?;
    if let Ok(batch) = &copy.completion {
        budget.add(batch.candidates.len())?; budget.add(batch.input_deleted.len())?; budget.relations(&batch.unrepresented)?;
        for candidate in &batch.candidates { budget.relations(&candidate.relations)?; }
    }
    Ok(())
}
fn generated(copy: &crate::sketch_region_candidates::CandidateCopyInspection, budget: &mut HistoryRowBudget) -> HistoryCheck<()> {
    let areas = match &copy.generated_topology {
        crate::sketch_region_candidates::GeneratedTopologyObservation::Observed(areas) | crate::sketch_region_candidates::GeneratedTopologyObservation::Incomplete { areas, .. } => areas,
        crate::sketch_region_candidates::GeneratedTopologyObservation::NotReached => return Ok(()),
    };
    budget.add(areas.len())?;
    for area in areas {
        if let RegionContextObservation::Captured(rows) = &area.contexts { budget.contexts(rows)?; }
        budget.add(area.members.len())?;
        for member in &area.members { budget.add(member.descendants.len())?; budget.add(member.copy_indices.len())?;
            budget.add(member.preserved.len())?; budget.add(member.modified.len())?; budget.add(member.generated.len())?; }
    }
    Ok(())
}
fn receipt(request: &PrismHistoryRequest<'_>, budget: &mut HistoryRowBudget) -> HistoryCheck<()> {
    let receipt = request.receipt;
    budget.add(receipt.input.rejected.len())?; budget.add(receipt.input.input_deleted.len())?; budget.relations(&receipt.input.unrepresented)?;
    for rejected in &receipt.input.rejected { budget.relations(rejected)?; }
    if let Some(selected) = &receipt.input.selected { budget.relations(&selected.relations)?; }
    if let Some(copy) = &receipt.copied { budget.relations(&copy.relations)?; budget.add(copy.observations.len())?; }
    if let Some(body) = &receipt.diagnostic {
        budget.relations(&body.relations)?; budget.relations(&body.native_retired)?; budget.add(body.bindings.len())?;
        if let crate::sketch_region_prism::PrismSweepQueries::Captured(rows) = &body.sweep_queries {
            budget.add(rows.len())?;
            for row in rows { budget.relations(std::slice::from_ref(&row.query))?; budget.add(row.generated.len())?; }
        }
    }
    Ok(())
}
