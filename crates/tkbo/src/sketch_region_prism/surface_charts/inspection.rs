// SPDX-License-Identifier: GPL-2.0-only
use super::super::{
    PrismBodyContext, PrismCompletion, PrismDiagnostic, PrismImageBinding, PrismPhase,
    PrismProgress, PrismReceipt, history::all_parents,
};
use super::{capture::capture_chart, progress::ChartProgress, types::*};
use crate::sketch_region::{ArrangementRelation, PartitionOccurrenceIdentity};

pub(in crate::sketch_region_prism) fn retain_surface_charts(
    receipt: &mut PrismReceipt,
    observer: &mut dyn PrismProgress,
) -> Result<(), PrismCompletion> {
    if receipt.diagnostic.is_none() {
        return Ok(());
    }
    let diagnostic = receipt.diagnostic.as_mut().unwrap();
    let sources = &receipt.copied.as_ref().unwrap().chart_sources;
    let extrusion = receipt.limits.requested.extrusion;
    let mut progress = ChartProgress::new(observer);
    diagnostic.surface_charts = Some(inspect_diagnostic(
        diagnostic,
        sources,
        [extrusion.x(), extrusion.y(), extrusion.z()],
        &mut progress,
    )?);
    progress.completion()
}
struct InspectionContext<'a, 'b> {
    diagnostic: &'a PrismDiagnostic,
    sources: &'a PrismChartSources,
    extrusion: [f64; 3],
    progress: &'a mut ChartProgress<'b>,
}
fn inspect_diagnostic(
    diagnostic: &PrismDiagnostic,
    sources: &PrismChartSources,
    extrusion: [f64; 3],
    progress: &mut ChartProgress<'_>,
) -> Result<PrismSurfaceInspection, PrismCompletion> {
    let faces = super::coverage::actual_faces(diagnostic)?;
    let mut result = empty_inspection(faces.len());
    result.uncovered_faces = super::coverage::missing_body_images(diagnostic, &faces)?;
    let mut context = InspectionContext {
        diagnostic,
        sources,
        extrusion,
        progress,
    };
    result.scanned_all_faces = scan_faces(faces, &mut context, &mut result);
    Ok(result)
}
fn scan_faces(
    faces: Vec<PrismBodyContext>,
    context: &mut InspectionContext<'_, '_>,
    result: &mut PrismSurfaceInspection,
) -> bool {
    for face in faces {
        inspect_face(&face, context, result);
        if context.progress.stopped || context.progress.failure.is_some() {
            return false;
        }
    }
    true
}
fn empty_inspection(face_count: usize) -> PrismSurfaceInspection {
    PrismSurfaceInspection {
        version: 1,
        entries: Vec::new(),
        uncovered_faces: Vec::new(),
        face_count,
        scanned_all_faces: false,
    }
}
fn inspect_face(
    face: &PrismBodyContext,
    context: &mut InspectionContext<'_, '_>,
    result: &mut PrismSurfaceInspection,
) {
    let (binding, relation) = match super::coverage::face_history(face, context.diagnostic) {
        Ok(history) => history,
        Err(cause) => {
            result.uncovered_faces.push((
                PartitionOccurrenceIdentity::capture(&face.occurrence),
                cause,
            ));
            return;
        }
    };
    result
        .entries
        .push(inspect_entry(binding, relation, context));
}
fn inspect_entry(
    binding: &PrismImageBinding,
    face: &ArrangementRelation,
    context: &mut InspectionContext<'_, '_>,
) -> PrismSurfaceEntry {
    let mut entry = initial_entry(binding, face);
    match capture_chart(
        binding,
        &context.diagnostic.bindings,
        context.extrusion,
        context.sources,
    ) {
        Ok(chart) => inspect_captured(&mut entry, chart, context.progress),
        Err(cause) => entry.decision = Err(cause),
    }
    entry
}
fn inspect_captured(
    entry: &mut PrismSurfaceEntry,
    chart: CapturedSurfaceChart,
    progress: &mut ChartProgress<'_>,
) {
    entry.decision = super::prove_chart(
        &chart,
        entry.role,
        &mut || progress.poll(),
        &mut entry.proof,
    );
    entry.captured = Some(chart);
}
fn initial_entry(binding: &PrismImageBinding, face: &ArrangementRelation) -> PrismSurfaceEntry {
    PrismSurfaceEntry {
        source: PartitionOccurrenceIdentity::capture(&binding.native.query),
        output: PartitionOccurrenceIdentity::capture(&face.output),
        role: binding.native.role,
        parents: all_parents(face),
        captured: None,
        proof: PrismSurfaceProof::default(),
        decision: Err(PrismSurfaceCause::MissingHistory {
            entity: "chart inspection not executed",
        }),
    }
}
