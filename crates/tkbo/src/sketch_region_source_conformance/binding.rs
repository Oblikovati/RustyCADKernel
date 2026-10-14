// SPDX-License-Identifier: GPL-2.0-only
//! The required matrix is generated from current inputs, never from a partial proof's own rows.
use super::{binding_types::*, descriptor_bits::*, snapshot_binding::*, types::*};
use crate::sketch_region_certification::BoundaryInputs;
/// Attach immutable evidence to the complete current matrix; `bind_source_conformance(inputs, policy, report)`.
pub(crate) fn bind_source_conformance(
    inputs: &BoundaryInputs,
    requested: SourceConformancePolicy,
    inspection: SourceConformanceInspection,
) -> BoundSourceConformance {
    let faults = source_binding_faults(inputs, requested, &inspection);
    BoundSourceConformance { inspection, faults }
}
/// Revalidate inputs without trusting cached acceptance; `source_binding_faults(inputs, policy, report)`.
pub(crate) fn source_binding_faults(
    inputs: &BoundaryInputs,
    requested: SourceConformancePolicy,
    inspection: &SourceConformanceInspection,
) -> Vec<SourceBindingFault> {
    let mut faults = Vec::new();
    if !same_source_policy(requested, inspection.requested) {
        faults.push(SourceBindingFault::Policy);
    }
    check_snapshots(inputs, inspection, &mut faults);
    check_required_matrix(inputs, inspection, &mut faults);
    faults
}

fn check_snapshots(
    inputs: &BoundaryInputs,
    report: &SourceConformanceInspection,
    faults: &mut Vec<SourceBindingFault>,
) {
    if inputs.segments.is_empty() || inputs.originals.is_empty() || inputs.constructions.is_empty()
    {
        faults.push(SourceBindingFault::EmptyUniverse);
    }
    check_original_snapshots(inputs, report, faults);
    check_construction_snapshots(inputs, report, faults);
    for segment in &inputs.segments {
        check_segment(inputs, segment, faults);
    }
    if inputs.segments.len() != inputs.constructions.len() {
        faults.push(SourceBindingFault::ConstructionSnapshot);
    }
}
fn check_original_snapshots(
    inputs: &BoundaryInputs,
    report: &SourceConformanceInspection,
    faults: &mut Vec<SourceBindingFault>,
) {
    for (index, source) in inputs.originals.iter().enumerate() {
        if inputs.originals[..index]
            .iter()
            .any(|old| old.original == source.original)
        {
            faults.push(SourceBindingFault::OriginalSnapshot);
        }
    }
    if inputs.originals.len() != report.originals.len()
        || !inputs
            .originals
            .iter()
            .zip(&report.originals)
            .all(|(a, b)| same_original(a, b))
    {
        faults.push(SourceBindingFault::OriginalSnapshot);
    }
}
fn check_construction_snapshots(
    inputs: &BoundaryInputs,
    report: &SourceConformanceInspection,
    faults: &mut Vec<SourceBindingFault>,
) {
    if inputs.constructions.len() != report.constructions.len()
        || !inputs
            .constructions
            .iter()
            .zip(&report.constructions)
            .all(|(a, b)| same_construction(a, b))
    {
        faults.push(SourceBindingFault::ConstructionSnapshot);
    }
}
fn check_segment(
    inputs: &BoundaryInputs,
    segment: &crate::sketch_region_candidates::CandidateSegment,
    faults: &mut Vec<SourceBindingFault>,
) {
    let matches: Vec<_> = inputs
        .constructions
        .iter()
        .filter(|c| c.occurrence == segment.occurrence)
        .collect();
    if matches.len() != 1 || !binds_current_segment(matches[0], segment) {
        faults.push(SourceBindingFault::CurrentSegment(segment.occurrence));
        return;
    }
    if let Err(cause) = current_construction_correspondence(inputs, matches[0], segment) {
        faults.push(SourceBindingFault::CurrentGeometry {
            occurrence: segment.occurrence,
            cause: std::sync::Arc::new(cause),
        });
    }
}
fn check_required_matrix(
    inputs: &BoundaryInputs,
    report: &SourceConformanceInspection,
    faults: &mut Vec<SourceBindingFault>,
) {
    let mut expected = Vec::new();
    for construction in &inputs.constructions {
        if construction.parents.is_empty() {
            faults.push(SourceBindingFault::CurrentSegment(construction.occurrence));
        }
        for original in &construction.parents {
            let key = (construction.occurrence, original.clone());
            if expected.contains(&key) {
                faults.push(matrix_fault(&key));
            }
            expected.push(key);
        }
    }
    check_original_coverage(inputs, &expected, faults);
    check_matrix_rows(&expected, report, faults);
}
fn check_original_coverage(
    inputs: &BoundaryInputs,
    expected: &[(u64, crate::sketch_region::OriginalMemberIdentity)],
    faults: &mut Vec<SourceBindingFault>,
) {
    for source in &inputs.originals {
        if !expected
            .iter()
            .any(|(_, original)| *original == source.original)
        {
            faults.push(SourceBindingFault::OriginalSnapshot);
        }
    }
}
fn matrix_fault(key: &(u64, crate::sketch_region::OriginalMemberIdentity)) -> SourceBindingFault {
    SourceBindingFault::Matrix {
        occurrence: key.0,
        original: key.1.clone(),
    }
}
fn check_matrix_rows(
    expected: &[(u64, crate::sketch_region::OriginalMemberIdentity)],
    report: &SourceConformanceInspection,
    faults: &mut Vec<SourceBindingFault>,
) {
    if expected.len() != report.entries.len() {
        faults.push(SourceBindingFault::ConstructionSnapshot);
    }
    for (key, row) in expected.iter().zip(&report.entries) {
        if key.0 != row.occurrence || key.1 != row.original {
            faults.push(matrix_fault(key));
        }
    }
    for key in expected {
        check_one_matrix_row(key, report, faults);
    }
    if !super::stage_binding::work_matches(report) {
        faults.push(SourceBindingFault::WorkReceipt);
    }
}

fn check_one_matrix_row(
    key: &(u64, crate::sketch_region::OriginalMemberIdentity),
    report: &SourceConformanceInspection,
    faults: &mut Vec<SourceBindingFault>,
) {
    let rows: Vec<_> = report
        .entries
        .iter()
        .filter(|row| row.occurrence == key.0 && row.original == key.1)
        .collect();
    if rows.len() != 1 {
        faults.push(matrix_fault(key));
        return;
    }
    if !super::stage_binding::complete_stages(rows[0], report.requested) {
        faults.push(SourceBindingFault::Stages {
            occurrence: key.0,
            original: key.1.clone(),
        });
    }
}

/// Check current mutable BRep fields separately from historical scalar proof.
/// Example: `current_construction_correspondence(inputs, capture, segment)?`.
pub(crate) fn current_construction_correspondence(
    inputs: &BoundaryInputs,
    historical: &CapturedBoundaryConstruction,
    segment: &crate::sketch_region_candidates::CandidateSegment,
) -> Result<(), SourceConformanceCause> {
    // Topology identity does not freeze mutable BRep curves, pcurves or support.
    let current = super::capture::capture_construction(segment, &inputs.construction_support);
    require_current_capture(&current.copied)?;
    require_current_capture(&current.pcurve)?;
    require_current_capture(&current.support)?;
    if !same_construction(historical, &current) {
        return Err(SourceConformanceCause::CurrentDescriptorMismatch {
            occurrence: segment.occurrence,
        });
    }
    Ok(())
}

fn require_current_capture<T>(capture: &CapturedGeometry<T>) -> Result<(), SourceConformanceCause> {
    capture
        .as_ref()
        .map(|_| ())
        .map_err(|cause| SourceConformanceCause::Capture(cause.clone()))
}
