// SPDX-License-Identifier: GPL-2.0-only
use super::{diagnostics::*, names::*, pipeline::*, progress::*, test_fixture::*};
use crate::sketch_region_prism::PrismCompletion;
use rustycad_tkbrep::{brep::BRepTool, topods::edge};
#[test]
fn owned_coefficients_drive_actual_pipeline_and_keep_body_publication_pending() {
    let result = prepared(request());
    let retained = &result.retained;
    assert_eq!(
        retained.plan.as_ref().unwrap().identities().boundary,
        CallerBoundaryId(0)
    );
    let receipt = retained.prism.as_ref().unwrap();
    assert!(matches!(receipt.completion, PrismCompletion::Incomplete));
    let certificate = receipt.region.as_ref().unwrap().decision.as_ref().unwrap();
    assert_eq!(certificate.selected_winding, 1);
    assert!(receipt.diagnostic.is_some());
    assert!(retained.bridge.is_some());
}
#[test]
fn constructor_retains_one_authored_vertex_with_native_zero_and_one_uses() {
    for reversed in [false, true] {
        let mut request = request();
        request.sources[0].reversed_traversal = reversed;
        let result = prepared(request);
        let source = result.retained.source.as_ref().unwrap();
        assert!(source.native_start.is_same(&source.closure));
        assert!(source.native_end.is_same(&source.closure));
        assert_actual_constructor_uses(source);
        assert_eq!(native_parameters(source, false), [0., 1.]);
        let expected = if reversed { [1., 0.] } else { [0., 1.] };
        assert_eq!(native_parameters(source, true), expected);
    }
}
fn native_parameters(
    source: &super::construction::ConstructedPolynomialSource,
    cumulative: bool,
) -> [f64; 2] {
    let shape = if cumulative {
        source.edge.clone()
    } else {
        source.native_edge.clone()
    };
    let edge = edge(&shape).unwrap();
    [&source.native_start, &source.native_end]
        .map(|v| BRepTool::parameter(&rustycad_tkbrep::topods::vertex(v).unwrap(), &edge).unwrap())
}
#[test]
fn owned_source_creation_is_isolated_from_caller_buffer_edits() {
    let mut caller = request();
    let snapshot = caller.clone();
    let result = prepared(caller.clone());
    caller.sources[0].world_poles[1][0] = 99.;
    assert_eq!(
        result.retained.request.sources[0].world_poles,
        snapshot.sources[0].world_poles
    );
    assert_eq!(
        actual_source_poles(&result),
        snapshot.sources[0].world_poles
    );
}
#[test]
fn failed_stage_keeps_original_requested_policy_and_real_diagnostics() {
    let mut request = request();
    request.policy.contacts.world_endpoint_limit = -1.;
    let failure = prepare_owned_polynomial_prism(request, &mut RequestObserver::default())
        .err()
        .unwrap();
    assert_eq!(failure.class, OwnedPreparationClass::Failed);
    assert_eq!(
        failure
            .retained
            .request
            .policy
            .contacts
            .world_endpoint_limit,
        -1.
    );
    assert_contact_policy_failure(&failure.cause);
    assert!(failure.retained.source.is_some());
}
#[test]
fn constant_curve_and_tangent_extrusion_cannot_prepare_a_certified_region() {
    let mut constant = request();
    constant.sources[0].world_poles.fill([0.; 3]);
    let mut tangent = request();
    tangent.extrusion = [1., 0., 0.];
    for request in [constant, tangent] {
        assert!(prepare_owned_polynomial_prism(request, &mut RequestObserver::default()).is_err());
    }
}
#[test]
fn callback_failure_keeps_typed_error_and_never_becomes_plain_cancelled() {
    let mut observer = RequestObserver {
        fail: Some(2),
        ..RequestObserver::default()
    };
    let failure = prepare_owned_polynomial_prism(request(), &mut observer)
        .err()
        .unwrap();
    assert_eq!(failure.class, OwnedPreparationClass::Failed);
    assert!(matches!(
        failure.callback.unwrap().cause,
        OwnedPrismCallbackCause::Failed(_)
    ));
    assert_eq!(observer.seen.len(), 3);
    assert!(failure.retained.source.is_some());
}
#[test]
fn owned_request_operation_state_and_callback_context_are_send() {
    fn send<T: Send>() {}
    fn send_sync<T: Send + Sync>() {}
    send_sync::<super::request::OwnedPolynomialPrismRequest>();
    send::<PreparedPolynomialPrism>();
    send::<OwnedPrismPreparationFailure>();
    send::<super::progress::OwnedPrismCallbacks<'static>>();
}

fn assert_contact_policy_failure(cause: &OwnedPreparationCause) {
    assert!(matches!(
        cause,
        OwnedPreparationCause::Contact(
            crate::sketch_region_contact_transport::ContactTransportStop::Policy { value: -1., .. }
        )
    ));
}

fn actual_source_poles(result: &PreparedPolynomialPrism) -> Vec<[f64; 3]> {
    use crate::sketch_region_geometry_capture::CapturedMetricCurve;
    let source = result.retained.source.as_ref().unwrap();
    let capture =
        crate::sketch_region_source_conformance::capture_originals(&[source.source()]).unwrap();
    let space = capture[0].representation.as_ref().ok().unwrap();
    assert_eq!(space.interval, [0., 1.]);
    match &space.curve {
        CapturedMetricCurve::Bezier { poles, weights } => {
            assert!(weights.is_none());
            poles.clone()
        }
        _ => panic!("expected actual polynomial Bezier source"),
    }
}
fn assert_actual_constructor_uses(source: &super::construction::ConstructedPolynomialSource) {
    let edge = edge(&source.native_edge).unwrap();
    let first = rustycad_tkbrep::topexp::first_vertex(&edge, false);
    let last = rustycad_tkbrep::topexp::last_vertex(&edge, false);
    assert!(first.is_equal(&source.native_start));
    assert!(last.is_equal(&source.native_end));
}
