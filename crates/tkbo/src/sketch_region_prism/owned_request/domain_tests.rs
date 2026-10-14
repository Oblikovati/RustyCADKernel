// SPDX-License-Identifier: GPL-2.0-only
use super::{diagnostics::*, names::*, pipeline::*, test_fixture::*};
use crate::sketch_region_prism::PrismCompletion;
#[test]
fn outside_and_boundary_seed_are_explicit_refusals_not_completed_empty() {
    for seed in [[10., 10.], [0., 0.]] {
        let mut request = request();
        request.selection = SemanticSelectionIntent::BoundedRegionContaining { seed };
        let failure = prepare_owned_polynomial_prism(request, &mut RequestObserver::default())
            .err()
            .unwrap();
        assert_eq!(failure.class, OwnedPreparationClass::Incomplete);
        assert!(matches!(
            failure.retained.prism.as_ref().unwrap().completion,
            PrismCompletion::NoBody(_)
        ));
    }
}
#[test]
fn generated_pole_traversal_and_extrusion_reversals_keep_caller_native_names() {
    for (poles, traversal, sign) in [
        (false, false, 1.),
        (true, false, 1.),
        (false, true, -1.),
        (true, true, -1.),
    ] {
        let mut request = request();
        if poles {
            request.sources[0].world_poles.reverse();
        }
        request.sources[0].reversed_traversal = traversal;
        request.extrusion[2] *= sign;
        let result = prepared(request);
        let ids = result.retained.plan.as_ref().unwrap().identities();
        assert_eq!(ids.start, CallerDirectedUseId(901));
        assert_eq!(ids.end, CallerDirectedUseId(19));
    }
}
#[test]
fn changed_coefficients_under_same_caller_revision_do_not_reuse_old_evidence() {
    let original = prepared(request());
    let mut edited = request();
    edited.sources[0].world_poles[1][2] = 0.25;
    let failure = prepare_owned_polynomial_prism(edited, &mut RequestObserver::default())
        .err()
        .unwrap();
    assert_eq!(failure.class, OwnedPreparationClass::Failed);
    assert_geometry_refusal(&failure);
    assert_eq!(
        failure.retained.request.sources[0].revision,
        original.retained.request.sources[0].revision
    );
    assert_certified(&original);
}
#[test]
fn declared_plane_translation_and_world_coefficients_remain_exactly_bound() {
    let mut request = request();
    for point in &mut request.sources[0].world_poles {
        point[2] = 4.;
    }
    request
        .declared_plane
        .set_location(rustycad_tkmath::gp::Point3::new(0., 0., 4.));
    let result = prepared(request);
    assert_certified(&result);
    assert_eq!(
        result.retained.request.sources[0].world_poles[0],
        [0., 0., 4.]
    );
}

fn assert_certified(result: &PreparedPolynomialPrism) {
    let receipt = result.retained.prism.as_ref().unwrap();
    let region = receipt.region.as_ref().unwrap();
    assert!(region.decision.is_ok());
}
fn assert_geometry_refusal(failure: &OwnedPrismPreparationFailure) {
    use crate::sketch_region_prism::PrismCause;
    use crate::sketch_region_validation::ValidationStage;
    let receipt = failure.retained.prism.as_ref().unwrap();
    assert!(matches!(
        receipt.completion,
        PrismCompletion::Failed(PrismCause::ValidationRejected(ValidationStage::Geometry))
    ));
}
