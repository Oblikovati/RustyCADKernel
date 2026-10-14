// SPDX-License-Identifier: GPL-2.0-only
//! One residual per canonical incidence retains every checked directed raw operand.
use super::{
    endpoint_uses::*, joined_progress::*, joined_types::*, scalar::certify_exact_endpoint,
};
use crate::sketch_region_prism::frozen::{proof_join::JoinedPrismProof, *};
use crate::sketch_region_source_conformance::CapturedSpaceRepresentation;
use std::collections::BTreeSet;

pub(super) fn certify_body_endpoints(
    join: &JoinedPrismProof,
    work: &mut ConformanceWork,
    progress: &mut impl JoinedConformanceProgress,
) -> ConformanceCheck {
    certify_projection_endpoints(
        join.projection(),
        join.raw_vertices(),
        (6, 12),
        work,
        progress,
    )
}

// Concrete sealed joins supply their admitted census; this helper issues no join authority.
pub(super) fn certify_projection_endpoints(
    projection: &FrozenPrismProjection,
    raw: &[proof_join::JoinedVertexOperand],
    expected: (usize, usize),
    work: &mut ConformanceWork,
    progress: &mut impl JoinedConformanceProgress,
) -> ConformanceCheck {
    for &(edge, first, last) in &projection.incidence().endpoints {
        let curve = endpoint_curve(projection, edge)?;
        for endpoint in obligations(edge, first, last, curve.interval) {
            append_endpoint(projection, raw, endpoint, curve, work, progress)?;
        }
    }
    count(
        "complete canonical endpoint proofs",
        work.evidence.endpoints.len(),
        expected.0,
    )?;
    complete_raw_contexts(raw, expected.1, work)
}

fn obligations(
    edge: FrozenPrismKey,
    first: FrozenPrismKey,
    last: FrozenPrismKey,
    interval: [f64; 2],
) -> [EndpointObligation; 2] {
    [
        (first, CanonicalEndpointSide::First, interval[0]),
        (last, CanonicalEndpointSide::Last, interval[1]),
    ]
    .map(|(vertex, side, parameter)| EndpointObligation {
        edge,
        vertex,
        side,
        parameter,
    })
}

fn append_endpoint(
    projection: &FrozenPrismProjection,
    raw: &[proof_join::JoinedVertexOperand],
    endpoint: EndpointObligation,
    curve: &CapturedSpaceRepresentation,
    work: &mut ConformanceWork,
    progress: &mut impl JoinedConformanceProgress,
) -> ConformanceCheck {
    work.entity = Some(endpoint.edge);
    work.occurrence = None;
    work.poll(progress)?;
    let uses = endpoint_uses(projection, raw, endpoint, work)?;
    work.evidence
        .endpoints
        .push(endpoint_evidence(endpoint, curve, &uses)?);
    Ok(())
}
fn endpoint_evidence(
    endpoint: EndpointObligation,
    curve: &CapturedSpaceRepresentation,
    uses: &[&proof_join::JoinedVertexOperand],
) -> ConformanceCheck<JoinedEndpointEvidence> {
    let raw = uses[0];
    let contexts = uses.iter().map(|row| row.occurrence.clone()).collect();
    let certificate =
        certify_exact_endpoint(curve, endpoint.parameter, raw.local_point, raw.placement)
            .map_err(|cause| endpoint_failure(endpoint, uses, cause))?;
    Ok(JoinedEndpointEvidence {
        edge: endpoint.edge,
        vertex: endpoint.vertex,
        side: endpoint.side,
        parameter: endpoint.parameter,
        contexts,
        local_point: raw.local_point,
        certificate,
    })
}
fn endpoint_failure(
    endpoint: EndpointObligation,
    uses: &[&proof_join::JoinedVertexOperand],
    cause: super::scalar::ExactEndpointFailure,
) -> JoinedConformanceCause {
    JoinedConformanceCause::Endpoint {
        vertex: endpoint.vertex,
        side: endpoint.side,
        contexts: uses.iter().map(|row| row.occurrence.clone()).collect(),
        cause: Box::new(cause),
    }
}

fn endpoint_curve(
    projection: &FrozenPrismProjection,
    edge: FrozenPrismKey,
) -> ConformanceCheck<&CapturedSpaceRepresentation> {
    let matching: Vec<_> = projection
        .nodes()
        .iter()
        .filter(|node| node.key == edge)
        .collect();
    one(&matching, "one captured canonical edge")?
        .curve
        .as_ref()
        .ok_or(missing("captured canonical edge curve"))
}

fn complete_raw_contexts(
    raw: &[proof_join::JoinedVertexOperand],
    total: usize,
    work: &ConformanceWork,
) -> ConformanceCheck {
    let expected: BTreeSet<_> = raw.iter().map(|row| row.occurrence.clone()).collect();
    count("unique directed joined raw vertices", expected.len(), total)?;
    let actual: Vec<_> = work
        .evidence
        .endpoints
        .iter()
        .flat_map(|row| row.contexts.iter().cloned())
        .collect();
    count(
        "all directed raw endpoint proof contexts",
        actual.len(),
        total,
    )?;
    same_context_set(actual.into_iter().collect(), expected)
}
fn same_context_set(
    actual: BTreeSet<FrozenUseKey>,
    expected: BTreeSet<FrozenUseKey>,
) -> ConformanceCheck {
    if actual != expected {
        return Err(JoinedConformanceCause::Mismatch {
            quantity: "complete directed endpoint/raw-vertex bijection",
        });
    }
    Ok(())
}
