// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use crate::sketch_region_certification::census_bezier_fixture;

#[test]
fn unsupported_behavior_version_has_typed_admission_failure() {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    let mut policy = options();
    policy.version = 2;
    let failure = attempt(&bridge, &shapes, &receipt, policy, || false)
        .err()
        .unwrap();
    assert_eq!(failure.phase, PrismEmbeddingPhase::Admission);
    assert_eq!(failure.requested, policy);
    assert!(failure.original.is_none());
    assert!(matches!(
        failure.cause,
        PrismEmbeddingCause::Version {
            actual: 2,
            expected: 1
        }
    ));
}
#[test]
fn regularity_budget_and_version_refusals_retain_actual_source_and_policy() {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    for mode in 0..3 {
        let mut policy = options();
        match mode {
            0 => policy.regularity.max_stored_degree = 2,
            1 => policy.regularity.roots.max_degree = 1,
            _ => policy.regularity.version = 2,
        }
        let failure = attempt(&bridge, &shapes, &receipt, policy, || false)
            .err()
            .unwrap();
        assert_eq!(failure.phase, PrismEmbeddingPhase::Regularity);
        assert!(failure.original.is_some());
        assert_eq!(failure.requested, policy);
        let PrismEmbeddingCause::Regularity(cause) = failure.cause else {
            panic!("expected numerical refusal")
        };
        assert_eq!(cause.source().options(), policy.regularity);
        assert_eq!(cause.source().poles().len(), 4);
        assert!(!matches!(
            cause.completion(),
            BezierRegularityCompletion::Complete
        ));
    }
}
#[test]
fn stationary_quartic_keeps_genuine_census_refusal() {
    let poles = [[0., 0.], [0., 0.], [4., 0.], [0., 4.], [0., 0.]];
    let (bridge, shapes) = fixture_for(vec![census_bezier_fixture(&poles, None)]);
    let receipt = run(&bridge, &shapes);
    let failure = attempt(&bridge, &shapes, &receipt, options(), || false)
        .err()
        .unwrap();
    assert_eq!(failure.phase, PrismEmbeddingPhase::CurrentBinding);
    assert!(matches!(failure.cause, PrismEmbeddingCause::Binding(_)));
    super::super::super::tests::assert_four_present(&receipt);
}
#[test]
fn incidence_table_refuses_missing_or_unrelated_authored_uses() {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    let report = embedded(&bridge, &shapes, &receipt);
    let mut domains = report.domains().clone();
    domains[0].trims.clear();
    assert!(table::six_rows(&domains).is_err());
    let mut domains = report.domains().clone();
    domains[0].trims[0].output.topology_address = usize::MAX;
    assert!(table::six_rows(&domains).is_err());
    let mut domains = report.domains().clone();
    let side = domains
        .iter_mut()
        .find(|domain| domain.role == PrismImageRole::Generated)
        .unwrap();
    side.trims
        .iter_mut()
        .find(|trim| trim.role == PrismTrimRole::End)
        .unwrap()
        .output
        .topology_address = usize::MAX;
    assert!(table::six_rows(&domains).is_err());
}
#[test]
fn missing_extra_face_roles_and_duplicate_trim_roles_cannot_form_table() {
    let (bridge, shapes) = fixture();
    let receipt = run(&bridge, &shapes);
    let report = embedded(&bridge, &shapes, &receipt);
    let mut domains = report.domains().clone();
    domains[2].role = PrismImageRole::First;
    assert!(table::six_rows(&domains).is_err());
    let mut domains = report.domains().clone();
    let duplicate = domains[0].trims[0].clone();
    domains[0].trims.push(duplicate);
    assert!(table::six_rows(&domains).is_err());
}
#[test]
fn normalized_oblique_chart_mismatch_stays_refused() {
    use super::super::super::super::tests::request;
    use super::super::super::measurements::tests::domain_tests::with_request;
    let (bridge, shapes) = fixture();
    let mut requested = request([1., 1.]);
    requested.extrusion = rustycad_tkmath::gp::Vector3::new(1., 1., 2.);
    let receipt = with_request(&bridge, &shapes, requested);
    let error = attempt(&bridge, &shapes, &receipt, options(), || false)
        .err()
        .unwrap();
    let PrismEmbeddingCause::Binding(cause) = error.cause else {
        panic!("expected binding failure")
    };
    assert!(matches!(
        *cause,
        PrismBindingFailure::Chart(PrismSurfaceCause::Identity {
            stage: PrismChartStage::SideDirection,
            ..
        })
    ));
}
#[test]
fn retained_nonzero_jacobian_cannot_certify_current_in_plane_sweep() {
    let (bridge, shapes) = fixture();
    let mut receipt = run(&bridge, &shapes);
    receipt.limits.requested.extrusion = rustycad_tkmath::gp::Vector3::new(1., 0., 0.);
    receipt.affine_chart.as_mut().unwrap().extrusion = [1., 0., 0.];
    let error = attempt(&bridge, &shapes, &receipt, options(), || false)
        .err()
        .unwrap();
    let PrismEmbeddingCause::Binding(cause) = error.cause else {
        panic!("expected binding failure")
    };
    assert!(matches!(
        *cause,
        PrismBindingFailure::Region(RegionCertificateCause::Hypothesis {
            name: "fresh nonzero measurement sweep determinant",
            ..
        })
    ));
}
