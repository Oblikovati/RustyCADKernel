// SPDX-License-Identifier: GPL-2.0-only
use super::{error::*, members, replay, test_fixture::*};
use crate::sketch_region_prism::{
    exact_tolerance::JoinedToleranceMetadata, frozen::FrozenSweepRole::First,
    tolerance_inspection::*,
};
pub(super) fn metadata() -> JoinedToleranceMetadata {
    JoinedToleranceMetadata {
        entity: key(2051, First, 6),
        contexts: vec![],
        observation: PrismToleranceRequest::new(StoredToleranceCap::report(1e-6).unwrap(), 0.)
            .unwrap()
            .observe(0., 1e-7)
            .unwrap(),
    }
}
#[test]
fn member_retains_named_geometry_range_and_tolerance_policy_bits() {
    let names = names();
    let source = node(key(2051, First, 6));
    let member = members::project(&source, &names, &[metadata()]).unwrap();
    assert_eq!(member.key(), names.member(&source.key).unwrap());
    assert_eq!(
        member
            .space_curve()
            .unwrap()
            .native_interval
            .map(f64::to_bits),
        [(-0_f64).to_bits(), 1_f64.to_bits()]
    );
    assert_eq!(member.tolerance().unwrap(), metadata().observation);
    assert_eq!(
        member.output_slot().unwrap().role,
        crate::sketch_region_prism::owned_request::names::SemanticNameRole::FirstRim
    );
}
#[test]
fn member_coefficients_remain_owned_after_input_mutation_and_drop() {
    let names = names();
    let mut source = node(key(2051, First, 6));
    let member = members::project(&source, &names, &[metadata()]).unwrap();
    let mut expected = Vec::new();
    replay::member(&member, &mut expected);
    source.curve.as_mut().unwrap().interval = [3., 5.];
    corrupt_poles(source.curve.as_mut().unwrap());
    source.raw_tolerance = Some(99.);
    drop(source);
    drop(names);
    let mut actual = Vec::new();
    replay::member(&member, &mut actual);
    assert_eq!(actual, expected);
}
#[test]
fn duplicate_missing_or_inconsistent_tolerance_observations_refuse() {
    let names = names();
    let mut source = node(key(2051, First, 6));
    assert!(members::project(&source, &names, &[]).is_err());
    assert!(members::project(&source, &names, &[metadata(), metadata()]).is_err());
    source.raw_tolerance = Some(-0.);
    let failed = members::project(&source, &names, &[metadata()])
        .err()
        .unwrap();
    assert!(matches!(
        failed.cause,
        PrismBodyProjectionCause::OperandMismatch { .. }
    ));
    assert!(matches!(
        failed.entity,
        Some(PrismBodyProjectionEntity::Member(_))
    ));
}

fn corrupt_poles(space: &mut crate::sketch_region_source_conformance::CapturedSpaceRepresentation) {
    use crate::sketch_region_geometry_capture::CapturedMetricCurve;
    let CapturedMetricCurve::Bezier { poles, .. } = &mut space.curve else {
        panic!("expected owned Bezier fixture");
    };
    poles[0][0] = 99.;
}
