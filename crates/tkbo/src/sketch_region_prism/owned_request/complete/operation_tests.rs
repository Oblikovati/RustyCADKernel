// SPDX-License-Identifier: GPL-2.0-only
//! First real composition fixture uses caller operands, not fixture-issued correspondence.
use super::super::{OwnedPrismPhase, names::reservations::*, test_fixture};
use super::*;
use rustycad_tkernel::standard::Failure;
#[derive(Default)]
pub(super) struct CompleteObserver {
    points: Vec<CompletePrismCheckpoint>,
    stop: Option<usize>,
}
impl CompletePrismProgress for CompleteObserver {
    fn checkpoint(&mut self, point: CompletePrismCheckpoint) -> Result<bool, Failure> {
        self.points.push(point);
        Ok(self.stop == Some(point.ordinal))
    }
}
pub(super) fn accepted() -> (OwnedCompletePrism, CompleteObserver) {
    let mut request = test_fixture::request();
    request.policy.publication.version = 2;
    let mut progress = CompleteObserver::default();
    let result = with_owned_prism_operation(request, &mut progress, |result| {
        result.unwrap_or_else(|failure| panic!("composition refused at {}", stage(failure.cause())))
    });
    (result, progress)
}
fn stage(cause: &ScopedPrismCause<'_>) -> &'static str {
    match cause {
        ScopedPrismCause::Preparation(_) => "preparation",
        ScopedPrismCause::Resolution { .. } => "resolution",
        ScopedPrismCause::Publication(_) => "publication",
        ScopedPrismCause::Projection(_) => "projection",
    }
}
#[test]
fn actual_complete_owned_request_publishes_one_isolated_named_body() {
    let (result, progress) = accepted();
    assert_eq!(result.names().plan().version(), 2);
    assert_eq!(result.publication().requested().version, 2);
    assert_body_roster(result.body());
    assert!(result.body().binds(result.publication(), result.names()));
    assert!(result.reservations().binds(result.names()));
    assert_eq!(result.request().sources[0].revision.0, 71);
    assert_coverage(&result);
    assert_measures(&result);
    assert_outer_order(&progress.points);
}
fn assert_body_roster(
    body: &crate::sketch_region_prism::publication::body_view::OwnedNamedPrismBody,
) {
    assert_eq!((body.members().len(), body.occurrences().len()), (13, 26));
    let trims = body.faces().iter().map(|f| f.trims().len()).sum::<usize>();
    let tolerances = body
        .members()
        .iter()
        .filter(|m| m.tolerance().is_some())
        .count();
    assert_eq!((trims, tolerances), (6, 8));
}
fn assert_coverage(result: &OwnedCompletePrism) {
    let rows = result.reservations().rows();
    assert_eq!(rows.len(), 101);
    assert_eq!(
        rows.iter()
            .filter(|r| matches!(r.status, PrismReservationStatus::Applied { .. }))
            .count(),
        73
    );
    assert_eq!(
        rows.iter()
            .filter(|r| matches!(r.status, PrismReservationStatus::Inapplicable(_)))
            .count(),
        28
    );
}
fn assert_measures(result: &OwnedCompletePrism) {
    let bounds = result.volume().enclosure();
    assert!(bounds[0] <= 24. / 5. && bounds[1] >= 24. / 5.);
    let measures = result.publication().measures();
    let mut caps = 0;
    for face in measures.named_faces() {
        let named = measures.face_area(&face.occurrence).unwrap();
        if matches!(
            named.scalar,
            crate::sketch_region_prism::frozen::joined_measures::PrismAreaReport::Plane(_)
        ) {
            let bounds = named.scalar.enclosure();
            assert!(bounds[0] <= 12. / 5. && bounds[1] >= 12. / 5.);
            caps += 1;
        }
    }
    assert_eq!(caps, 2);
}
fn assert_outer_order(points: &[CompletePrismCheckpoint]) {
    assert_eq!(
        points.iter().map(|p| p.ordinal).collect::<Vec<_>>(),
        (0..points.len()).collect::<Vec<_>>()
    );
    let last = points.last().unwrap();
    assert_eq!(last.phase, CompletePrismPhase::Commit);
    assert_eq!(last.child_ordinal, None);
    for phase in [false, true] {
        let local: Vec<_> = points
            .iter()
            .filter(|p| matches!(p.phase, CompletePrismPhase::Publication(_)) == phase)
            .filter_map(|p| p.child_ordinal)
            .collect();
        assert_eq!(local, (0..local.len()).collect::<Vec<_>>());
    }
}
#[test]
fn construction_cancel_retains_v2_names_before_any_geometry_exists() {
    let mut observer = CompleteObserver {
        stop: Some(1),
        ..CompleteObserver::default()
    };
    with_owned_prism_operation(test_fixture::request(), &mut observer, |result| {
        assert_preconstruction_failure(result.err().unwrap());
    });
    assert_eq!(observer.points.len(), 2);
}
fn assert_preconstruction_failure(failure: ScopedPrismFailure<'_>) {
    let ScopedPrismCause::Preparation(original) = failure.cause() else {
        panic!("preparation required")
    };
    let plan = original.retained.plan.as_ref().unwrap();
    assert_eq!((plan.version(), plan.entries().len()), (2, 101));
    assert!(original.retained.source.is_none());
    let point = failure.callback().unwrap().checkpoint;
    assert_eq!(
        point.phase,
        CompletePrismPhase::Preparation(OwnedPrismPhase::Construction)
    );
    assert_eq!(failure.request().sources[0].revision.0, 71);
}
#[test]
fn complete_success_is_send_sync_and_operation_context_is_send() {
    fn send<T: Send>() {}
    fn send_sync<T: Send + Sync>() {}
    send_sync::<OwnedCompletePrism>();
    send_sync::<CompletePrismCallbackStop>();
    send::<context::CompletePrismContext<'_>>();
    send::<ScopedPrismFailure<'_>>();
}
