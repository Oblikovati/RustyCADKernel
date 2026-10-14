// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use crate::sketch_region::PartitionOccurrenceIdentity;

pub(super) fn link_proof(
    proof: &PrismEmbeddingEvidence,
    bound: &BoundProjection<'_>,
    projection: &FrozenPrismProjection,
    contact: &AuthoredClosureContact,
) -> CaptureResult<JoinedProofLinks> {
    check_projection_coverage(proof, bound, projection)?;
    check_contact(proof, bound.bindings, contact)?;
    check_copied_source(proof.binding(), bound)?;
    let mut faces = proof
        .binding()
        .charts
        .iter()
        .map(|chart| link_face(chart, bound, projection))
        .collect::<CaptureResult<Vec<_>>>()?;
    faces.sort_by(|a, b| a.occurrence.cmp(&b.occurrence));
    Ok(JoinedProofLinks {
        contact: contact.name,
        original: original_key(bound.bindings, &proof.binding().region.original)?,
        faces,
        parents: parents::current_dispositions(bound, projection)?,
        raw_vertices: raw_vertices::capture(bound, projection)?,
    })
}
fn check_contact(
    proof: &PrismEmbeddingEvidence,
    bindings: &PrismPublicationBindings,
    contact: &AuthoredClosureContact,
) -> CaptureResult<()> {
    let used = [bindings.operation, bindings.selected_region]
        .into_iter()
        .chain(
            bindings
                .originals
                .iter()
                .flat_map(|b| [b.member, b.directed_use]),
        )
        .chain(bindings.sources.iter().map(|b| b.member))
        .chain(bindings.directed.iter().map(|b| b.authored_use));
    if contact.name.0 == 0 || used.into_iter().any(|name| name == contact.name) {
        return Err(identity(
            "nonzero distinct caller-authored closure contact",
            0,
        ));
    }
    let region = &proof.binding().region;
    if contact.actual != region.contact || contact.original != region.original {
        return Err(identity(
            "actual source-bound closure contact and original",
            0,
        ));
    }
    Ok(())
}
fn check_copied_source(
    binding: &CurrentPrismBinding,
    bound: &BoundProjection<'_>,
) -> CaptureResult<()> {
    let copied = bound
        .receipt
        .copied
        .as_ref()
        .ok_or(missing("copied source"))?;
    let captured = copied.chart_sources.as_ref().map_err(|cause| {
        PrismFreezeCause::Surface(PrismSurfaceCause::SourceSnapshot(cause.clone()))
    })?;
    if binding.region.copied.face != PartitionOccurrenceIdentity::capture(&copied.base)
        || encoding::sources(&binding.region.copied) != encoding::sources(captured)
    {
        return Err(PrismFreezeCause::StaleCapture {
            representation: "same current copied proof source",
        });
    }
    Ok(())
}
pub(super) fn face_index(
    bound: &BoundProjection<'_>,
    chart: &BoundPrismChart,
) -> CaptureResult<usize> {
    let found: Vec<_> = bound
        .contexts
        .iter()
        .enumerate()
        .filter(|(_, context)| {
            PartitionOccurrenceIdentity::capture(&context.occurrence) == chart.output
        })
        .map(|(i, _)| i)
        .collect();
    only_bound(&found, "one actual face context for joined proof").copied()
}
/// Bind unchanged chart operands to frozen names; `link_frozen_chart(chart, bound, projection)`.
pub(in crate::sketch_region_prism::frozen) fn link_face(
    chart: &BoundPrismChart,
    bound: &BoundProjection<'_>,
    projection: &FrozenPrismProjection,
) -> CaptureResult<JoinedPrismFace> {
    let index = face_index(bound, chart)?;
    let key = bound.names[index];
    let frozen = projection
        .faces()
        .iter()
        .filter(|face| face.key == key)
        .collect::<Vec<_>>();
    let frozen = *only_bound(&frozen, "one frozen face for actual proof context")?;
    let source = source_identity_name(bound.bindings, &chart.source)?;
    if source != key.source || FrozenSweepRole::from(chart.role) != key.role {
        return Err(identity("same actual face source and sweep role", 0));
    }
    let trims = chart
        .captured
        .trims
        .iter()
        .map(|trim| link_trim(bound, index, trim))
        .collect::<CaptureResult<Vec<_>>>()?;
    check_face_operands(chart, key, &trims, frozen, projection)?;
    Ok(JoinedPrismFace {
        definition: key,
        occurrence: bound.use_names[index].clone(),
        source,
        role: chart.role,
        parents: parents::original_multiset(bound.bindings, &chart.parents)?,
        trims,
    })
}
fn link_trim(
    bound: &BoundProjection<'_>,
    face: usize,
    trim: &CapturedPrismTrim,
) -> CaptureResult<JoinedPrismTrim> {
    Ok(JoinedPrismTrim {
        occurrence: super::super::geometry::trim_occurrence(bound, face, &trim.output)?,
        source: source_identity_name(bound.bindings, &trim.source)?,
        role: trim.role,
        parents: parents::original_multiset(bound.bindings, &trim.parents)?,
        interval: trim.interval,
    })
}
fn check_face_operands(
    chart: &BoundPrismChart,
    key: FrozenPrismKey,
    links: &[JoinedPrismTrim],
    frozen: &FrozenPrismFace,
    projection: &FrozenPrismProjection,
) -> CaptureResult<()> {
    let captured = FrozenPrismFace {
        key,
        source: chart.captured.source.clone(),
        surface: chart.captured.actual.clone(),
        proof: chart.proof.clone(),
        trims: chart
            .captured
            .trims
            .iter()
            .zip(links)
            .map(|(trim, link)| projected_trim(trim, link))
            .collect(),
    };
    let mut expected = Vec::new();
    let mut actual = Vec::new();
    super::super::views::face_bytes(&captured, &mut expected);
    super::super::views::face_bytes(frozen, &mut actual);
    if expected != actual
        || chart.captured.extrusion.map(f64::to_bits)
            != projection.limits().extrusion.map(f64::to_bits)
    {
        return Err(PrismFreezeCause::StaleCapture {
            representation: "same actual frozen/proof chart operands",
        });
    }
    Ok(())
}
fn projected_trim(trim: &CapturedPrismTrim, link: &JoinedPrismTrim) -> FrozenPrismTrim {
    let mut parents = link.parents.clone();
    parents.dedup();
    FrozenPrismTrim {
        occurrence: link.occurrence.clone(),
        source: link.source,
        parents,
        role: trim.role,
        expected: trim.expected.clone(),
        pcurve: trim.pcurve.clone(),
        interval: trim.interval,
        space: trim.space.clone(),
    }
}

fn check_projection_coverage(
    proof: &PrismEmbeddingEvidence,
    bound: &BoundProjection<'_>,
    projection: &FrozenPrismProjection,
) -> CaptureResult<()> {
    if projection.faces().len() != proof.binding().charts.len() {
        return Err(identity(
            "exactly the current proof face universe",
            projection.faces().len(),
        ));
    }
    let mut actual = projection
        .uses()
        .iter()
        .map(|use_| use_.key.clone())
        .collect::<Vec<_>>();
    let mut expected = bound.use_names.clone();
    actual.sort();
    expected.sort();
    if actual != expected {
        return Err(identity(
            "all and only current authored directed contexts",
            actual.len(),
        ));
    }
    Ok(())
}
