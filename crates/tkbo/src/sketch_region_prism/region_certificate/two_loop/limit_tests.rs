// SPDX-License-Identifier: GPL-2.0-only
use super::{
    failure::*, fixture::*, fixture_bindings, operation::*, operation_tests::*, outcome::*,
    request::*,
};
fn with_options<'a>(
    fixture: &'a TwoLoopFixture,
    requested: TwoLoopRegionOptions,
) -> TwoLoopRegionOutcome<'a> {
    let sources = fixture.sources();
    let identities = fixture_bindings::identities(fixture);
    inspect_two_loop_region(
        &TwoLoopRegionRequest {
            bridge: &fixture.bridge,
            sources: &sources,
            identities: &identities,
            inner_interior: [1., 1.],
            material_seed: [0.25, 0.25],
            requested,
        },
        &mut OperationProgress,
    )
}
#[test]
fn inclusive_whole_operation_operand_budget_and_pole_limit_are_enforced() {
    let fixture = TwoLoopFixture::nested();
    let first = accepted(inspect(&fixture, &mut OperationProgress));
    let mut requested = fixture_bindings::options();
    requested.maximum_retained_units = first.retained_units;
    assert_eq!(
        accepted(with_options(&fixture, requested)).retained_units,
        first.retained_units
    );
    requested.maximum_retained_units -= 1;
    let TwoLoopRegionOutcome::Incomplete(failure) = with_options(&fixture, requested) else {
        panic!("one short; expected typed limit")
    };
    assert!(matches!(failure.cause, TwoLoopRegionCause::Limit { .. }));
    check_pole_limit(&fixture);
}
struct CancelAt {
    cutoff: usize,
    polls: usize,
}
impl TwoLoopRegionProgress for CancelAt {
    fn checkpoint(
        &mut self,
        _: TwoLoopRegionStage,
    ) -> Result<bool, rustycad_tkernel::standard::Failure> {
        self.polls += 1;
        Ok(self.polls == self.cutoff)
    }
}
#[test]
fn every_actual_cancellation_cutoff_preserves_source_and_later_reuse() {
    let fixture = TwoLoopFixture::nested();
    let baseline = accepted(inspect(&fixture, &mut OperationProgress)).replay;
    let mut counter = CancelAt {
        cutoff: usize::MAX,
        polls: 0,
    };
    accepted(inspect(&fixture, &mut counter));
    check_cancellation_cutoffs(&fixture, counter.polls);
    assert_eq!(
        accepted(inspect(&fixture, &mut OperationProgress)).replay,
        baseline
    );
}

fn check_pole_limit(fixture: &TwoLoopFixture) {
    let mut requested = fixture_bindings::options();
    requested.maximum_poles = 3;
    let TwoLoopRegionOutcome::Incomplete(failure) = with_options(fixture, requested) else {
        panic!("four poles; expected limit3 refusal")
    };
    assert!(matches!(
        failure.cause,
        TwoLoopRegionCause::Limit {
            operand: "polynomial poles",
            requested: 3,
            measured: 4
        }
    ));
}

fn check_cancellation_cutoffs(fixture: &TwoLoopFixture, polls: usize) {
    for cutoff in 1..=polls {
        let mut progress = CancelAt { cutoff, polls: 0 };
        assert!(
            matches!(
                inspect(fixture, &mut progress),
                TwoLoopRegionOutcome::Cancelled(_)
            ),
            "cutoff {cutoff}"
        );
    }
}
