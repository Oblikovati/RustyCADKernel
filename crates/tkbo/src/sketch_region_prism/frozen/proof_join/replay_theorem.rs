// SPDX-License-Identifier: GPL-2.0-only
use super::*;

pub(super) fn regularity(
    proof: &PrismEmbeddingEvidence,
    links: &JoinedProofLinks,
    bytes: &mut Vec<u8>,
) {
    let report = proof.regularity();
    let source = report.source();
    replay_options::regularity(source.options(), bytes);
    replay::text(
        &format!("regularity/{:?}/{:?}", links.original, source.options()),
        bytes,
    );
    encoding::curve(
        &crate::sketch_region_geometry_capture::CapturedMetricCurve::Bezier {
            poles: source.poles().to_vec(),
            weights: source.weights().map(<[f64]>::to_vec),
        },
        bytes,
    );
    encoding::placement(source.placement(), bytes);
    encoding::numbers(source.interval(), bytes);
    replay::text(
        &format!(
            "{:?}/{:?}/{}/{:?}/{:?}/{}",
            report.completion(),
            report.denominator_isolation(),
            report.denominator_degree(),
            report.speed_degree(),
            report.decision(),
            report.bisections()
        ),
        bytes,
    );
}
pub(super) fn incidences(
    proof: &PrismEmbeddingEvidence,
    bound: &BoundProjection<'_>,
    bytes: &mut Vec<u8>,
) -> CaptureResult<()> {
    for row in proof.incidences() {
        domain(&row.first, proof, bound, bytes)?;
        domain(&row.second, proof, bound, bytes)?;
        conclusion(&row.conclusion, proof, bound, bytes)?;
    }
    Ok(())
}
fn domain(
    domain: &PrismFaceDomain,
    proof: &PrismEmbeddingEvidence,
    bound: &BoundProjection<'_>,
    bytes: &mut Vec<u8>,
) -> CaptureResult<()> {
    replay::text(
        &format!(
            "{:?}/{:?}/{:?}",
            face_use(&domain.face, proof, bound)?,
            domain.role,
            parents::original_multiset(bound.bindings, &domain.parents)?
        ),
        bytes,
    );
    replay::text(
        &format!(
            "source={:?}",
            source_identity_name(bound.bindings, &domain.source)?
        ),
        bytes,
    );
    match &domain.kind {
        PrismDomainKind::Cap {
            region,
            winding,
            sweep,
        } => {
            replay::text(
                &format!(
                    "cap/{:?}/{winding}",
                    source_identity_name(bound.bindings, region)?
                ),
                bytes,
            );
            encoding::numbers([*sweep], bytes);
        }
        PrismDomainKind::Side {
            boundary,
            native,
            stored_sweep,
            normalized_sweep,
            height,
        } => {
            replay::text(
                &format!("side/{:?}", source_identity_name(bound.bindings, boundary)?),
                bytes,
            );
            encoding::numbers(
                native
                    .iter()
                    .chain(stored_sweep)
                    .chain(normalized_sweep)
                    .copied()
                    .chain([*height]),
                bytes,
            );
        }
    }
    for use_ in &domain.trims {
        trim(use_, proof, bound, bytes)?;
    }
    Ok(())
}
fn conclusion(
    value: &PrismIncidenceConclusion,
    proof: &PrismEmbeddingEvidence,
    bound: &BoundProjection<'_>,
    bytes: &mut Vec<u8>,
) -> CaptureResult<()> {
    match value {
        PrismIncidenceConclusion::InjectivePlanarDomain => {
            replay::text("injective-planar-domain", bytes)
        }
        PrismIncidenceConclusion::Disjoint => replay::text("disjoint", bytes),
        PrismIncidenceConclusion::InjectiveModuloSeam {
            start,
            end,
            native_endpoints,
            normalized_sweep,
        } => {
            replay::text("injective-modulo-authored-seam", bytes);
            trim(start, proof, bound, bytes)?;
            trim(end, proof, bound, bytes)?;
            encoding::numbers(
                native_endpoints.iter().chain(normalized_sweep).copied(),
                bytes,
            );
        }
        PrismIncidenceConclusion::ExactlyRim {
            cap,
            side,
            native,
            normalized_sweep,
        } => {
            replay::text("exactly-authored-rim", bytes);
            trim(cap, proof, bound, bytes)?;
            trim(side, proof, bound, bytes)?;
            encoding::numbers(native.iter().copied().chain([*normalized_sweep]), bytes);
        }
    }
    Ok(())
}
fn trim(
    value: &PrismAuthoredTrimUse,
    proof: &PrismEmbeddingEvidence,
    bound: &BoundProjection<'_>,
    bytes: &mut Vec<u8>,
) -> CaptureResult<()> {
    let chart = chart_for(&value.face, proof)?;
    let index = links::face_index(bound, chart)?;
    let key = super::super::geometry::trim_occurrence(bound, index, &value.output)?;
    replay::text(
        &format!(
            "{:?}/{:?}/{:?}/{:?}",
            key,
            source_identity_name(bound.bindings, &value.source)?,
            value.role,
            parents::original_multiset(bound.bindings, &value.parents)?
        ),
        bytes,
    );
    encoding::numbers(value.interval, bytes);
    Ok(())
}
fn face_use(
    key: &crate::sketch_region::PartitionOccurrenceIdentity,
    proof: &PrismEmbeddingEvidence,
    bound: &BoundProjection<'_>,
) -> CaptureResult<FrozenUseKey> {
    let chart = chart_for(key, proof)?;
    let index = links::face_index(bound, chart)?;
    Ok(bound.use_names[index].clone())
}
fn chart_for<'a>(
    key: &crate::sketch_region::PartitionOccurrenceIdentity,
    proof: &'a PrismEmbeddingEvidence,
) -> CaptureResult<&'a BoundPrismChart> {
    let rows: Vec<_> = proof
        .binding()
        .charts
        .iter()
        .filter(|chart| &chart.output == key)
        .collect();
    only_bound(&rows, "one actual chart for theorem domain").copied()
}
