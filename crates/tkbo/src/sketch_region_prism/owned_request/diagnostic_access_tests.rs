// SPDX-License-Identifier: GPL-2.0-only
//! The seam exposes exactly the existing local state, without synthesizing names or outcomes.
use super::{diagnostic_access::*, pipeline::*, progress::*, test_fixture::*};
#[test]
fn failed_admission_view_keeps_original_request_and_missing_plan() {
    let mut input = request();
    input.sources.clear();
    let failure = prepare_owned_polynomial_prism(input, &mut RequestObserver::default())
        .err()
        .unwrap();
    let view = failure.diagnostic_context();
    assert!(std::ptr::eq(view.request(), &failure.retained.request));
    assert_eq!(view.phase(), OwnedPrismPhase::Admission);
    assert!(view.plan().is_none());
    assert!(view.retained_stages().is_empty());
}
#[test]
fn later_failed_view_names_constructed_owner_without_publishing_it() {
    let mut observer = RequestObserver {
        fail: Some(2),
        ..RequestObserver::default()
    };
    let failure = prepare_owned_polynomial_prism(request(), &mut observer)
        .err()
        .unwrap();
    let view = failure.diagnostic_context();
    assert_eq!(view.phase(), OwnedPrismPhase::SourceCapture);
    assert_eq!(
        view.retained_stages(),
        [RetainedPreparationStage::ConstructedSource]
    );
    assert!(std::ptr::eq(
        view.plan().unwrap(),
        failure.retained.plan.as_ref().unwrap()
    ));
}
#[test]
fn prepared_view_names_outer_owners_after_children_move_into_contact_transaction() {
    let prepared = prepared(request());
    let view = prepared.diagnostic_context();
    use RetainedPreparationStage::*;
    assert_eq!(
        view.retained_stages(),
        [ConstructedSource, ContactTransport, Prism]
    );
    assert_eq!(view.phase(), prepared.retained.phase);
    assert!(std::ptr::eq(view.request(), &prepared.retained.request));
    assert!(std::ptr::eq(
        view.plan().unwrap(),
        prepared.retained.plan.as_ref().unwrap()
    ));
}

struct PhaseStopObserver(OwnedPrismPhase);
impl OwnedPrismProgress for PhaseStopObserver {
    fn checkpoint(
        &mut self,
        point: OwnedPrismCheckpoint,
    ) -> Result<bool, rustycad_tkernel::standard::Failure> {
        let stop = point.phase == self.0;
        Ok(stop)
    }
}
#[test]
fn intermediate_views_retain_inputs_then_census_and_partition_before_transport() {
    use RetainedPreparationStage::*;
    let phase = OwnedPrismPhase::Contact(
        crate::sketch_region_contact_transport::ContactTransportPhase::SourceBinding,
    );
    for (phase, expected) in [
        (
            OwnedPrismPhase::Census,
            vec![ConstructedSource, BoundaryInputs],
        ),
        (phase, vec![ConstructedSource, Census, Partition]),
    ] {
        let failure = prepare_owned_polynomial_prism(request(), &mut PhaseStopObserver(phase))
            .err()
            .unwrap();
        assert_eq!(failure.diagnostic_context().retained_stages(), expected);
    }
}
