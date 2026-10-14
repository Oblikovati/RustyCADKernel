// SPDX-License-Identifier: GPL-2.0-only
//! The authored cap roles and extrusion independently specify the six endpoint operands.
use super::super::{PrismImageRole, PrismReceipt, contexts::body_contexts};
use super::{
    census::*,
    census_tests::{assert_count, fixture},
    census_types::*,
};
use crate::sketch_region::PartitionOccurrenceIdentity;
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};

#[test]
fn six_endpoint_operands_retain_twelve_directed_uses_including_reversed_seam() {
    let (receipt, binding) = fixture();
    let census = capture_census(&receipt, &binding).ok().unwrap();
    assert_eq!(census.endpoints.len(), 6);
    let mut signatures = Vec::new();
    for row in &census.endpoints {
        assert_eq!(row.contexts.len(), 2);
        signatures.push(endpoint_signature(&receipt, row));
    }
    signatures.sort();
    assert_eq!(signatures, expected_endpoint_signatures());
}

type EndpointSignature = (u8, u64, [u64; 3]);

fn endpoint_signature(receipt: &PrismReceipt, row: &ToleranceEndpointCensus) -> EndpointSignature {
    let role = edge_role(receipt, &row.edge);
    let range = if role == PrismImageRole::Generated {
        [0., 2.]
    } else {
        [0., 1.]
    };
    assert_eq!(row.space.interval, range);
    let expected_parameter = match row.vertex.orientation {
        Orientation::Forward => range[0],
        Orientation::Reversed => range[1],
        actual => panic!("endpoint direction {actual:?}; expected Forward/Reversed"),
    };
    assert_eq!(row.parameter, expected_parameter);
    (
        role_code(role),
        row.parameter.to_bits(),
        row.point.map(f64::to_bits),
    )
}

fn edge_role(receipt: &PrismReceipt, edge: &PartitionOccurrenceIdentity) -> PrismImageRole {
    let roles: Vec<_> = receipt
        .diagnostic
        .as_ref()
        .unwrap()
        .bindings
        .iter()
        .filter(|row| {
            edge.same_physical(&PartitionOccurrenceIdentity::capture(
                &row.context.occurrence,
            ))
        })
        .map(|row| row.native.role)
        .collect();
    assert!(!roles.is_empty());
    assert!(roles.iter().all(|role| *role == roles[0]));
    roles[0]
}

fn role_code(role: PrismImageRole) -> u8 {
    match role {
        PrismImageRole::First => 1,
        PrismImageRole::Last => 2,
        PrismImageRole::Generated => 3,
    }
}

fn expected_endpoint_signatures() -> Vec<EndpointSignature> {
    [
        (1, 0., 0.),
        (1, 1., 0.),
        (2, 0., 2.),
        (2, 1., 2.),
        (3, 0., 0.),
        (3, 2., 2.),
    ]
    .into_iter()
    .map(|(role, parameter, z): (u8, f64, f64)| {
        (
            role,
            parameter.to_bits(),
            [0_f64.to_bits(), 0_f64.to_bits(), z.to_bits()],
        )
    })
    .collect()
}

#[test]
fn missing_vertex_context_does_not_disappear_into_shared_physical_incidence() {
    let (receipt, binding) = fixture();
    let diagnostic = receipt.diagnostic.as_ref().unwrap();
    let mut contexts = body_contexts(&diagnostic.body).ok().unwrap();
    let index = contexts
        .iter()
        .position(|row| row.occurrence.shape_type().unwrap() == ShapeEnum::Vertex)
        .unwrap();
    contexts.remove(index);
    let error = capture_census_from_contexts(&contexts, &binding, &diagnostic.relations)
        .err()
        .unwrap();
    assert_count(error, "direct vertex contexts per edge use", 1, 2, true);
}

#[test]
fn internal_direction_remains_an_explicit_unsupported_endpoint_incidence() {
    let result = super::endpoint_contexts::require_directed(Orientation::Internal);
    assert!(matches!(
        result.err().unwrap().cause,
        ToleranceCensusCause::Orientation {
            actual: Orientation::Internal,
            ..
        }
    ));
}
