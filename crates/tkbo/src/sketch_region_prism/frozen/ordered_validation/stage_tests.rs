// SPDX-License-Identifier: GPL-2.0-only
//! Independent control-flow expectations; these authored events do not stand in for geometry.
use super::*;
use OrderedPrismStageStatus::*;
use static_assertions::assert_impl_all;

#[test]
fn stage_trace_accepts_only_the_topology_geometry_intersection_tolerance_sequence() {
    let mut trace = OrderedPrismStageTrace::pending();
    for (index, stage) in OrderedPrismStage::ALL.into_iter().enumerate() {
        assert_eq!(stage.index(), index);
        assert_eq!(trace.next(), Some(stage));
        assert_eq!(trace.status(stage), Blocked);
        trace.record_prism_stage(stage, Passed).unwrap();
        assert_eq!(trace.status(stage), Passed);
    }
    assert_eq!(trace.next(), None);
}

#[test]
fn stage_trace_refusal_incomplete_and_cancel_keep_all_later_stages_blocked() {
    for stopping_stage in OrderedPrismStage::ALL {
        for terminal in [Failed, Incomplete, Cancelled] {
            check_stopped_prefix(stopping_stage, terminal);
        }
    }
}

fn check_stopped_prefix(stop: OrderedPrismStage, terminal: OrderedPrismStageStatus) {
    let mut trace = OrderedPrismStageTrace::pending();
    for stage in OrderedPrismStage::ALL {
        if stage.index() < stop.index() {
            trace.record_prism_stage(stage, Passed).unwrap();
        }
    }
    trace.record_prism_stage(stop, terminal).unwrap();
    assert_eq!(trace.next(), None);
    check_stopped_states(&mut trace, stop, terminal);
}

fn check_stopped_states(
    trace: &mut OrderedPrismStageTrace,
    stop: OrderedPrismStage,
    terminal: OrderedPrismStageStatus,
) {
    for stage in OrderedPrismStage::ALL {
        let expected = expected_stopped_status(stage, stop, terminal);
        assert_eq!(trace.status(stage), expected);
        let before = trace.clone();
        assert!(trace.record_prism_stage(stage, Passed).is_err());
        assert_eq!(*trace, before);
    }
}

fn expected_stopped_status(
    stage: OrderedPrismStage,
    stop: OrderedPrismStage,
    terminal: OrderedPrismStageStatus,
) -> OrderedPrismStageStatus {
    if stage.index() < stop.index() {
        return Passed;
    }
    if stage == stop {
        return terminal;
    }
    Blocked
}

#[test]
fn stage_trace_rejects_reordering_and_blocked_outcomes() {
    let mut trace = OrderedPrismStageTrace::pending();
    for stage in OrderedPrismStage::ALL.into_iter().skip(1) {
        assert!(trace.record_prism_stage(stage, Passed).is_err());
    }
    assert_eq!(trace, OrderedPrismStageTrace::pending());
    assert!(
        trace
            .record_prism_stage(OrderedPrismStage::Topology, Blocked)
            .is_err()
    );
    assert_eq!(trace, OrderedPrismStageTrace::pending());
}

#[test]
fn stage_trace_rejects_duplicate_completion() {
    let mut trace = OrderedPrismStageTrace::pending();
    trace
        .record_prism_stage(OrderedPrismStage::Topology, Passed)
        .unwrap();
    let before = trace.clone();
    let failure = trace
        .record_prism_stage(OrderedPrismStage::Topology, Passed)
        .unwrap_err();
    assert_eq!(failure.expected_stage, Some(OrderedPrismStage::Geometry));
    assert_eq!(failure.actual_stage, OrderedPrismStage::Topology);
    assert_eq!(failure.actual_status, Passed);
    assert_eq!(trace, before);
}

#[test]
fn stage_contracts_are_send_sync_without_unsafe_implementations() {
    assert_impl_all!(OrderedPrismStage: Send, Sync);
    assert_impl_all!(OrderedPrismStageStatus: Send, Sync);
    assert_impl_all!(OrderedPrismStageTrace: Send, Sync);
    assert_impl_all!(super::stages::PrismStageTransitionError: Send, Sync);
}
