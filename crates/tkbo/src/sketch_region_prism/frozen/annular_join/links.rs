// SPDX-License-Identifier: GPL-2.0-only
//! Every theorem face, trim and wire is joined through its actual full directed occurrence.
use super::*;
use crate::sketch_region::RegionObservedContext;

pub(super) fn capture(
    proof: &AnnularEmbeddingEvidence,
    bound: &BoundProjection<'_>,
    projection: &FrozenPrismProjection,
) -> AnnularFrozenCheck<'static, Vec<NamedAnnularFace>> {
    let observed = bound
        .contexts
        .iter()
        .map(snapshot_annular_context)
        .collect::<Result<Vec<_>, _>>()
        .map_err(AnnularFrozenCause::Context)?;
    let domains = proof.domains();
    let mut rows = [
        &domains.first,
        &domains.last,
        &domains.outer,
        &domains.inner,
    ]
    .into_iter()
    .map(|domain| link_domain(proof.binding(), domain, bound, projection, &observed))
    .collect::<CaptureResult<Vec<_>>>()?;
    rows.sort_by(|a, b| a.chart.occurrence.cmp(&b.chart.occurrence));
    check_coverage(&rows, projection)?;
    Ok(rows)
}
fn link_domain(
    binding: &AnnularPrismBinding,
    domain: &AnnularFaceDomain,
    bound: &BoundProjection<'_>,
    projection: &FrozenPrismProjection,
    observed: &[RegionObservedContext],
) -> CaptureResult<NamedAnnularFace> {
    let charts: Vec<_> = binding
        .charts()
        .iter()
        .filter(|c| c.identity == domain.identity)
        .collect();
    let chart = *only_bound(&charts, "exact bound chart for certified annular domain")?;
    let face = link_frozen_chart(&chart.chart, bound, projection)?;
    if context_key(&domain.identity.output, observed, bound)? != face.occurrence {
        return Err(identity("same full named theorem face context", 0));
    }
    check_trims(domain, &face, observed, bound)?;
    let kind = named_domain(binding, domain, &face, observed, bound)?;
    Ok(NamedAnnularFace {
        chart: face,
        domain: kind,
    })
}
fn context_key(
    actual: &RegionObservedContext,
    observed: &[RegionObservedContext],
    bound: &BoundProjection<'_>,
) -> CaptureResult<FrozenUseKey> {
    let found: Vec<_> = observed
        .iter()
        .enumerate()
        .filter(|(_, c)| *c == actual)
        .map(|(i, _)| &bound.use_names[i])
        .collect();
    Ok((*only_bound(
        &found,
        "one full named occurrence for actual annular context",
    )?)
    .clone())
}
fn check_trims(
    domain: &AnnularFaceDomain,
    face: &JoinedPrismFace,
    observed: &[RegionObservedContext],
    bound: &BoundProjection<'_>,
) -> CaptureResult<()> {
    let mut actual = domain
        .trims
        .iter()
        .map(|t| context_key(&t.identity.output, observed, bound))
        .collect::<CaptureResult<Vec<_>>>()?;
    let mut expected: Vec<_> = face.trims.iter().map(|t| t.occurrence.clone()).collect();
    actual.sort();
    expected.sort();
    if actual != expected || actual.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(identity(
            "bijection of theorem and frozen directed trim uses",
            actual.len(),
        ));
    }
    Ok(())
}
fn named_domain(
    binding: &AnnularPrismBinding,
    domain: &AnnularFaceDomain,
    face: &JoinedPrismFace,
    observed: &[RegionObservedContext],
    bound: &BoundProjection<'_>,
) -> CaptureResult<NamedAnnularDomain> {
    match domain.kind {
        AnnularDomainKind::Cap { outer, inner, .. } => Ok(NamedAnnularDomain::Cap {
            outer: cap_wire(binding, domain, face, outer, observed, bound)?,
            inner: cap_wire(binding, domain, face, inner, observed, bound)?,
        }),
        AnnularDomainKind::Side { source, .. } => Ok(NamedAnnularDomain::Side {
            boundary: NamedAnnularWire {
                source,
                occurrence: shared_wire(face)?,
            },
        }),
    }
}
fn cap_wire(
    binding: &AnnularPrismBinding,
    domain: &AnnularFaceDomain,
    face: &JoinedPrismFace,
    source: RegionSourceId,
    observed: &[RegionObservedContext],
    bound: &BoundProjection<'_>,
) -> CaptureResult<NamedAnnularWire> {
    let boundaries: Vec<_> = binding
        .boundaries()
        .iter()
        .filter(|b| b.source == source)
        .collect();
    let boundary = *only_bound(
        &boundaries,
        "one bound source for authored annular wire role",
    )?;
    let rims: Vec<_> = domain
        .trims
        .iter()
        .filter(|t| t.identity.source == boundary.copied)
        .collect();
    let rim = *only_bound(
        &rims,
        "one full-source cap rim for certified outer or inner boundary",
    )?;
    let key = context_key(&rim.identity.output, observed, bound)?;
    Ok(NamedAnnularWire {
        source,
        occurrence: parent_wire(&key, &face.occurrence)?,
    })
}
fn shared_wire(face: &JoinedPrismFace) -> CaptureResult<FrozenUseKey> {
    let wires = face
        .trims
        .iter()
        .map(|t| parent_wire(&t.occurrence, &face.occurrence))
        .collect::<CaptureResult<Vec<_>>>()?;
    let first = wires
        .first()
        .ok_or(missing("seam-cut side parameter wire"))?;
    if wires.iter().any(|wire| wire != first) {
        return Err(identity(
            "all side parameter trims on the same actual directed wire",
            wires.len(),
        ));
    }
    Ok(first.clone())
}
fn parent_wire(trim: &FrozenUseKey, face: &FrozenUseKey) -> CaptureResult<FrozenUseKey> {
    if trim.0.len() != face.0.len() + 2 || !trim.0.starts_with(&face.0) {
        return Err(identity("direct face/wire/trim ancestry", trim.0.len()));
    }
    Ok(FrozenUseKey(trim.0[..trim.0.len() - 1].to_vec()))
}
fn check_coverage(
    rows: &[NamedAnnularFace],
    projection: &FrozenPrismProjection,
) -> CaptureResult<()> {
    let mut actual = rows.iter().map(|r| r.chart.definition).collect::<Vec<_>>();
    let mut expected = projection.faces().iter().map(|f| f.key).collect::<Vec<_>>();
    actual.sort();
    expected.sort();
    if actual != expected || actual.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(identity(
            "all and only actual frozen faces in certified annular domain bijection",
            actual.len(),
        ));
    }
    Ok(())
}
