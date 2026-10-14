// SPDX-License-Identifier: GPL-2.0-only
//! Fresh actual-face coverage and existing chart proof, with explicit history parents.
mod history;
use super::super::super::surface_charts::{
    actual_faces, face_history, missing_body_images, same_context,
};
use super::super::types::{holds, need, require};
use super::*;
use crate::sketch_region::PartitionOccurrenceIdentity;

pub(super) fn current_charts(
    receipt: &PrismReceipt,
    cancelled: &mut impl FnMut() -> bool,
) -> PrismBindingCheck<Vec<BoundPrismChart>> {
    let charts = capture_actual_charts(receipt, cancelled, |faces| {
        require("single boundary prism face universe", faces.len(), 3)?;
        Ok(())
    })?;
    complete_roles(&charts)?;
    Ok(charts)
}

/// Capture individual chart operands; `capture_actual_charts(receipt,cancel,admit_faces)`.
/// The admission callback supplies domain policy at the historical V1 check position.
/// These operands alone certify neither a plural region nor a complete body binding.
pub(in crate::sketch_region_prism) fn capture_actual_charts(
    receipt: &PrismReceipt,
    cancelled: &mut impl FnMut() -> bool,
    admit_faces: impl FnOnce(&[PrismBodyContext]) -> PrismBindingCheck,
) -> PrismBindingCheck<Vec<BoundPrismChart>> {
    let diagnostic = need(receipt.diagnostic.as_ref(), "measurement actual body")?;
    let copied = need(receipt.copied.as_ref(), "measurement actual copied base")?;
    history::current_history(diagnostic, copied)?;
    let faces = actual_faces(diagnostic)?;
    admit_faces(&faces)?;
    complete_face_images(diagnostic, &faces)?;
    let mut charts = Vec::new();
    for face in &faces {
        charts.push(capture_current(face, receipt, cancelled)?);
    }
    Ok(charts)
}
fn capture_current(
    face: &PrismBodyContext,
    receipt: &PrismReceipt,
    cancelled: &mut impl FnMut() -> bool,
) -> PrismBindingCheck<BoundPrismChart> {
    let diagnostic = receipt.diagnostic.as_ref().unwrap();
    let (binding, relation) = face_history(face, diagnostic)?;
    let captured = capture_bound(binding, receipt)?;
    let mut proof = PrismSurfaceProof::default();
    prove_chart(&captured, binding.native.role, cancelled, &mut proof)?;
    Ok(BoundPrismChart {
        source: PartitionOccurrenceIdentity::capture(&binding.native.query),
        output: PartitionOccurrenceIdentity::capture(&face.occurrence),
        role: binding.native.role,
        parents: super::super::super::history::all_parents(relation),
        captured,
        proof,
    })
}

fn complete_face_images(
    diagnostic: &PrismDiagnostic,
    faces: &[PrismBodyContext],
) -> PrismBindingCheck {
    require(
        "all retained face images current",
        missing_body_images(diagnostic, faces)?.len(),
        0,
    )?;
    Ok(())
}
fn complete_roles(charts: &[BoundPrismChart]) -> PrismBindingCheck {
    for role in [
        PrismImageRole::First,
        PrismImageRole::Last,
        PrismImageRole::Generated,
    ] {
        require(
            "unique complete prism chart role",
            charts.iter().filter(|chart| chart.role == role).count(),
            1,
        )?;
    }
    Ok(())
}
fn capture_bound(
    binding: &PrismImageBinding,
    receipt: &PrismReceipt,
) -> PrismBindingCheck<CapturedSurfaceChart> {
    let diagnostic = receipt.diagnostic.as_ref().unwrap();
    let sources = &receipt.copied.as_ref().unwrap().chart_sources;
    let vector = receipt.limits.requested.extrusion;
    let extrusion = [vector.x(), vector.y(), vector.z()];
    Ok(capture_chart(
        binding,
        &diagnostic.bindings,
        extrusion,
        sources,
    )?)
}
