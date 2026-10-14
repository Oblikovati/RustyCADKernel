// SPDX-License-Identifier: GPL-2.0-only
//! Refusal contexts own authored keys without losing exact typed numerical/native causes.
use super::{tests::*, *};
fn incomplete(outcome: PrismFreezeOutcome) -> PrismFreezeFailure {
    match outcome {
        PrismFreezeOutcome::Incomplete(failure) => failure,
        _ => panic!("expected attributed incomplete diagnostic"),
    }
}
pub(super) fn assert_raw_context(outcome: PrismFreezeOutcome, expected: FrozenPrismKey, raw: f64) {
    let failure = incomplete(outcome);
    let PrismFreezeCause::Entities {
        primary,
        related,
        occurrence,
        cause,
    } = failure.cause
    else {
        panic!("expected owned raw-tolerance entity context")
    };
    assert_eq!(failure.entity, Some(primary));
    assert_eq!(primary, expected);
    assert!(related.is_empty());
    assert_eq!(occurrence.unwrap().0.last().unwrap().0, primary);
    assert!(
        matches!(*cause, PrismFreezeCause::InvalidScalar { actual, .. }
        if actual.to_bits() == raw.to_bits())
    );
}
pub(super) fn assert_endpoint_context(
    outcome: PrismFreezeOutcome,
    before: &FrozenPrismProjection,
    vertex: FrozenPrismKey,
) {
    let failure = incomplete(outcome);
    assert!(failure.entity.is_some());
    assert_endpoint_pair(&failure.cause, before);
    let PrismFreezeCause::Entities {
        primary,
        related,
        cause,
        ..
    } = failure.cause
    else {
        unreachable!()
    };
    assert_eq!(failure.entity, Some(primary));
    assert_eq!(related, vec![vertex]);
    assert!(matches!(*cause, PrismFreezeCause::EndpointMismatch { .. }));
}
fn assert_endpoint_pair(cause: &PrismFreezeCause, projection: &FrozenPrismProjection) {
    let PrismFreezeCause::Entities {
        primary,
        related,
        occurrence,
        ..
    } = cause
    else {
        panic!("expected exact endpoint entity pair")
    };
    assert_eq!(related.len(), 1);
    assert!(occurrence.is_none());
    assert!(projection.incidence().endpoints.iter().any(
        |(edge, first, last)| edge == primary && (*first == related[0] || *last == related[0])
    ));
}
#[test]
fn endpoint_numerical_error_keeps_edge_vertex_pair_and_original_cause() {
    let projection = fresh_projection();
    let mut nodes = projection.nodes().to_vec();
    let edge = invalidate_authored_first_endpoint(&mut nodes, &projection);
    let cause = super::endpoints::certify_endpoints(&nodes, projection.incidence()).unwrap_err();
    assert_endpoint_pair(&cause, &projection);
    assert_eq!(cause.primary_entity(), Some(edge));
    let PrismFreezeCause::Entities { cause, .. } = cause else {
        unreachable!()
    };
    assert!(matches!(*cause, PrismFreezeCause::Endpoint(_)));
}
#[test]
fn native_classification_and_message_survive_owned_context() {
    let projection = fresh_projection();
    let (edge, vertex, _) = projection.incidence().endpoints[0];
    let original = rustycad_tkernel::standard::Failure::new("owned native cause");
    let cause = PrismFreezeCause::Native(original).at_endpoint(edge, vertex);
    assert!(cause.native_failure());
    assert_eq!(cause.primary_entity(), Some(edge));
    let PrismFreezeCause::Entities { cause, .. } = cause else {
        unreachable!()
    };
    let PrismFreezeCause::Native(failure) = *cause else {
        unreachable!()
    };
    assert_eq!(failure.message(), "owned native cause");
}

fn invalidate_authored_first_endpoint(
    nodes: &mut [FrozenPrismNode],
    projection: &FrozenPrismProjection,
) -> FrozenPrismKey {
    let edge = projection
        .incidence()
        .endpoints
        .iter()
        .find(|(key, _, _)| {
            key.source == AuthoredPrismId(703) && key.role == FrozenSweepRole::First
        })
        .unwrap()
        .0;
    let curve = nodes
        .iter_mut()
        .find(|n| n.key == edge)
        .unwrap()
        .curve
        .as_mut()
        .unwrap();
    curve.interval[0] = f64::NAN;
    edge
}
pub(super) fn actual_key(
    receipt: &super::super::PrismReceipt,
    bindings: &PrismPublicationBindings,
    actual: &rustycad_tkbrep::topods::Shape,
) -> FrozenPrismKey {
    let bound = super::bindings::bind_projection(receipt, bindings)
        .ok()
        .unwrap();
    let index = bound
        .contexts
        .iter()
        .position(|c| c.occurrence.is_same(actual))
        .unwrap();
    bound.names[index]
}
