// SPDX-License-Identifier: GPL-2.0-only
//! Operation-local identity proofs reuse full scalar inputs, never a topology address alone.
use super::*;
use crate::sketch_region::OriginalMemberIdentity;
use crate::sketch_region_source_conformance::*;
use rustycad_tkmath::predicates::roots::metric_bounds::ResidualDecision;

pub(super) struct SourceCertificateKey {
    physical_source: usize,
    original: CapturedSpaceRepresentation,
    construction: CapturedBoundaryConstruction,
}
pub(super) struct SourceIdentityCertificate {
    key: SourceCertificateKey,
    pub entry: usize,
    pub decision: ResidualDecision,
    pub completed_bisections: u64,
}
pub(super) fn capture_key(
    retained: &PairedContactTransaction,
    construction: &CapturedBoundaryConstruction,
    parent: &OriginalMemberIdentity,
) -> Option<SourceCertificateKey> {
    let occurrences = &retained.census.paired.as_ref()?.occurrences;
    let source = occurrences
        .iter()
        .find(|row| row.source.occurrence == construction.occurrence)?;
    let original = super::exact_sources::single_original_representation(
        &retained.census.inputs.originals,
        parent,
    )
    .ok()?;
    Some(SourceCertificateKey {
        physical_source: source.physical_source,
        original: original.clone(),
        construction: construction.clone(),
    })
}
pub(super) fn same_key(a: &SourceCertificateKey, b: &SourceCertificateKey) -> bool {
    a.physical_source == b.physical_source
        && same_space_representation(&a.original, &b.original)
        && same_construction_basis(&a.construction, &b.construction)
}
pub(super) fn admit_certificate(
    cache: &mut Vec<SourceIdentityCertificate>,
    key: Option<SourceCertificateKey>,
    proof: &super::exact_sources::ExactContactSource,
    entry: usize,
) {
    let (Some(key), Ok(decision @ ResidualDecision::Within { .. })) = (key, &proof.decision) else {
        return;
    };
    cache.push(SourceIdentityCertificate {
        key,
        entry,
        decision: decision.clone(),
        completed_bisections: proof.completed_bisections,
    });
}
pub(super) fn lookup<'a>(
    cache: &'a [SourceIdentityCertificate],
    key: Option<&SourceCertificateKey>,
) -> Option<&'a SourceIdentityCertificate> {
    let key = key?;
    cache
        .iter()
        .find(|certificate| same_key(&certificate.key, key))
}
