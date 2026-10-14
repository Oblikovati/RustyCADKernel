// SPDX-License-Identifier: GPL-2.0-only
//! A zero-threshold identity proof is separate from the caller's nonzero metric admission.
use super::*;
use crate::sketch_region::OriginalMemberIdentity;
use crate::sketch_region_source_conformance::*;
use rustycad_tkmath::predicates::roots::metric_bounds::*;

pub(crate) struct ExactContactSource {
    pub occurrence: u64,
    pub original: OriginalMemberIdentity,
    pub decision: Result<ResidualDecision, SourceConformanceCause>,
    pub completed_bisections: u64,
    pub reused_certificate: Option<usize>,
    pub certificate_bisections: u64,
}

pub(super) fn capture_exact_sources(
    retained: &mut PairedContactTransaction,
    cancelled: &mut super::progress::ContactProgress<'_>,
) {
    let mut cache = Vec::new();
    for construction in &retained.census.inputs.constructions {
        for parent in &construction.parents {
            let proof =
                cached_source_identity(retained, construction, parent, &mut cache, cancelled);
            retained.inspection.completed_bisections += proof.completed_bisections;
            retained.exact_sources.push(proof);
            if cancelled.stopped().is_some() {
                return;
            }
        }
    }
}

fn certify_exact_source(
    retained: &PairedContactTransaction,
    construction: &CapturedBoundaryConstruction,
    parent: &OriginalMemberIdentity,
    cancelled: &mut super::progress::ContactProgress<'_>,
) -> ExactContactSource {
    let roots = retained
        .requested
        .remaining_roots(retained.inspection.completed_bisections);
    let (decision, completed_bisections) = exact_source_decision(
        &retained.census.inputs.originals,
        construction,
        parent,
        roots,
        cancelled,
    );
    fresh_identity_source(construction, parent, decision, completed_bisections)
}

fn fresh_identity_source(
    construction: &CapturedBoundaryConstruction,
    parent: &OriginalMemberIdentity,
    decision: Result<ResidualDecision, SourceConformanceCause>,
    completed_bisections: u64,
) -> ExactContactSource {
    ExactContactSource {
        occurrence: construction.occurrence,
        original: parent.clone(),
        decision,
        completed_bisections,
        reused_certificate: None,
        certificate_bisections: completed_bisections,
    }
}

fn exact_source_decision(
    originals: &[OriginalCurveCapture],
    construction: &CapturedBoundaryConstruction,
    parent: &OriginalMemberIdentity,
    roots: rustycad_tkmath::predicates::roots::RootIsolationOptions,
    cancelled: &mut super::progress::ContactProgress<'_>,
) -> (Result<ResidualDecision, SourceConformanceCause>, u64) {
    let query = exact_source_query(originals, construction, parent, roots);
    let Ok(query) = query else {
        return (Err(query.err().unwrap()), 0);
    };
    // WHY: a new zero-threshold query cannot inherit a rounded upper bound from admission metadata.
    let report = certify_transported_residual(query, || {
        cancelled.poll(ContactTransportPhase::SourceBinding)
    });
    (
        report.decision.map_err(SourceConformanceCause::Exact),
        report.bisections,
    )
}

pub(super) fn exact_source_query<'a>(
    originals: &'a [OriginalCurveCapture],
    construction: &'a CapturedBoundaryConstruction,
    parent: &OriginalMemberIdentity,
    roots: rustycad_tkmath::predicates::roots::RootIsolationOptions,
) -> Result<TransportedResidualQuery<'a>, SourceConformanceCause> {
    let representation = single_original_representation(originals, parent)?;
    source_query(
        representation,
        construction,
        ConstructionStage::CapturedBasis,
        0.,
        roots,
    )
}

fn reused_source(
    construction: &CapturedBoundaryConstruction,
    parent: &OriginalMemberIdentity,
    old: &super::source_cache::SourceIdentityCertificate,
) -> ExactContactSource {
    ExactContactSource {
        occurrence: construction.occurrence,
        original: parent.clone(),
        decision: Ok(old.decision.clone()),
        completed_bisections: 0,
        reused_certificate: Some(old.entry),
        certificate_bisections: old.completed_bisections,
    }
}

pub(super) fn single_original_representation<'a>(
    originals: &'a [OriginalCurveCapture],
    parent: &OriginalMemberIdentity,
) -> Result<&'a CapturedSpaceRepresentation, SourceConformanceCause> {
    let mut matching = originals.iter().filter(|row| row.original == *parent);
    let first = matching
        .next()
        .ok_or_else(|| SourceConformanceCause::MissingOriginal(parent.clone()))?;
    if matching.next().is_some() {
        return Err(SourceConformanceCause::MissingOriginal(parent.clone()));
    }
    first
        .representation
        .as_ref()
        .map_err(|cause| SourceConformanceCause::Capture(cause.clone()))
}

fn cached_source_identity(
    retained: &PairedContactTransaction,
    construction: &CapturedBoundaryConstruction,
    parent: &OriginalMemberIdentity,
    cache: &mut Vec<super::source_cache::SourceIdentityCertificate>,
    cancelled: &mut super::progress::ContactProgress<'_>,
) -> ExactContactSource {
    let key = super::source_cache::capture_key(retained, construction, parent);
    let proof = match super::source_cache::lookup(cache, key.as_ref()) {
        Some(old) => reused_source(construction, parent, old),
        None => certify_exact_source(retained, construction, parent, cancelled),
    };
    if proof.reused_certificate.is_none() {
        super::source_cache::admit_certificate(cache, key, &proof, retained.exact_sources.len());
    }
    proof
}
