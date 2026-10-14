// SPDX-License-Identifier: GPL-2.0-only
//! A failed later request does not acquire or mutate an earlier receipt's geometry.
use super::*;
use crate::sketch_region::{ArrangementCompletion, equal_occurrences};
use crate::sketch_region_certification::census_curved_pair_fixture;
use crate::sketch_region_partition_bindings::binding_sources;
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Shape, edge},
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;

pub(super) fn retain_new_request(
    edges: &[Shape],
    invalid: bool,
    cancel: bool,
) -> PairedContactTransaction {
    let (census, mapped, _) = super::test_fixture::actual_inputs_for(edges.to_vec());
    let mut policy = super::test_fixture::contact_policy();
    if invalid {
        policy.world_endpoint_limit = -1.;
    }
    match transport_partition_contacts(census, mapped, policy, &binding_sources(edges), |_| cancel)
    {
        Ok(receipt) => receipt,
        Err(
            ContactTransportStop::Policy { completed, .. }
            | ContactTransportStop::Cancelled { completed, .. },
        ) => *completed,
    }
}

fn curve_sample(shape: &Shape) -> Point3 {
    let (curve, first, last) = BRepTool::curve(&edge(shape).unwrap()).unwrap();
    curve.value(first + (last - first) * 0.5).unwrap()
}

fn retained_geometry(receipt: &PairedContactTransaction) -> Vec<Point3> {
    let ArrangementCompletion::Arranged(owned) = &receipt.mapped.arrangement.completion else {
        panic!("expected actual owned arrangement");
    };
    let mut samples = receipt
        .census
        .inputs
        .segments
        .iter()
        .map(|segment| curve_sample(&segment.edge))
        .collect::<Vec<_>>();
    for member in equal_occurrences(&owned.arrangement).unwrap() {
        if member.shape_type().unwrap() == ShapeEnum::Edge {
            samples.push(curve_sample(&member));
        }
    }
    samples
}

#[test]
fn earlier_receipt_survives_later_policy_failure_cancellation_retry_and_source_mutation() {
    let edges = census_curved_pair_fixture();
    let published = retain_new_request(&edges, false, false);
    let before = retained_geometry(&published);
    let failed = retain_new_request(&edges, true, false);
    assert!(failed.inspection.contacts.is_empty());
    let stopped = retain_new_request(&edges, false, true);
    assert!(stopped.inspection.contacts.is_empty());
    let retried = retain_new_request(&edges, false, false);
    assert!(!super::test_fixture::bounded_contacts(&retried).is_empty());
    super::mutation_tests::change_contact_curve(&edges[0]);
    assert_eq!(retained_geometry(&published), before);
    assert_eq!(retained_geometry(&retried), before);
}
