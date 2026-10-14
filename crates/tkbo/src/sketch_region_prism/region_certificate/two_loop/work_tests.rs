// SPDX-License-Identifier: GPL-2.0-only
use super::{failure::*, request::*, work::*};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::predicates::roots::RootIsolationOptions;
struct FixedProgress {
    stop: bool,
}
impl TwoLoopRegionProgress for FixedProgress {
    fn checkpoint(&mut self, _: TwoLoopRegionStage) -> Result<bool, Failure> {
        Ok(self.stop)
    }
}
fn options(maximum_retained_units: usize) -> TwoLoopRegionOptions {
    TwoLoopRegionOptions {
        behavior: TwoLoopRegionBehavior::NestedPolynomialV2,
        winding: RootIsolationOptions::new(0.001),
        maximum_retained_units,
        maximum_poles: 33,
    }
}
#[test]
fn inclusive_operand_cap_refuses_before_mutating_achieved_units() {
    let mut progress = FixedProgress { stop: false };
    let mut work = TwoLoopWork::new(7, options(2), &mut progress);
    assert!(work.units("rows", 2).is_ok());
    let failure = work.units("rows", 1).err().unwrap();
    assert_eq!(work.retained_units, 2);
    assert!(matches!(
        failure.cause,
        TwoLoopRegionCause::Limit {
            requested: 2,
            measured: 3,
            ..
        }
    ));
}
#[test]
fn checked_operand_overflow_is_distinct_from_exhaustion() {
    let mut progress = FixedProgress { stop: false };
    let mut work = TwoLoopWork::new(7, options(usize::MAX), &mut progress);
    assert!(work.units("rows", usize::MAX).is_ok());
    assert!(matches!(
        work.units("rows", 1).err().unwrap().cause,
        TwoLoopRegionCause::Overflow { .. }
    ));
    assert_eq!(work.retained_units, usize::MAX);
}
#[test]
fn progress_and_hypothesis_refusals_retain_operation_stage_and_actual_values() {
    let mut progress = FixedProgress { stop: true };
    let mut work = TwoLoopWork::new(7, options(8), &mut progress);
    let failure = work.poll(TwoLoopRegionStage::Sources).err().unwrap();
    assert_eq!(failure.operation, 7);
    assert_eq!(failure.stage, TwoLoopRegionStage::Sources);
    assert!(matches!(failure.cause, TwoLoopRegionCause::Cancelled));
    assert!(work.count("arity", 2, 2).is_ok());
    assert!(matches!(
        work.count("arity", 3, 2).err().unwrap().cause,
        TwoLoopRegionCause::Count {
            actual: 3,
            expected: 2,
            ..
        }
    ));
    assert!(work.holds("bound", true).is_ok());
    assert!(matches!(
        work.holds("bound", false).err().unwrap().cause,
        TwoLoopRegionCause::Missing { .. }
    ));
}
