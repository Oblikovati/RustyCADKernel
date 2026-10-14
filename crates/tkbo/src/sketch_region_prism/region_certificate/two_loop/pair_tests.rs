// SPDX-License-Identifier: GPL-2.0-only
use super::{failure::*, fixture::*, pair_capture::domains, request::*, work::*};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::predicates::roots::RootIsolationOptions;
struct PairProgress;
impl TwoLoopRegionProgress for PairProgress {
    fn checkpoint(&mut self, _: TwoLoopRegionStage) -> Result<bool, Failure> {
        Ok(false)
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
fn actual_complete_pair_projection_keeps_exceptional_and_rejected_root_evidence() {
    let fixture = TwoLoopFixture::nested();
    let mut progress = PairProgress;
    let mut work = TwoLoopWork::new(77, options(4096), &mut progress);
    let rows = domains(&fixture.bridge.census, [OUTER, INNER], &mut work)
        .unwrap_or_else(|_| panic!("complete actual pair projection"));
    assert_eq!(rows.len(), 3);
    assert_eq!(rows.iter().filter(|row| row.closure.is_some()).count(), 2);
    let cross = rows
        .iter()
        .find(|row| row.sources == [OUTER, INNER])
        .unwrap();
    assert!(cross.closure.is_none());
    assert!(work.retained_units > 3);
    assert_send_sync::<super::pair_types::TwoLoopPairProof>();
}
fn assert_send_sync<T: Send + Sync>() {}
#[test]
fn pair_projection_exact_limit_and_one_short_are_distinct_without_large_allocations() {
    let fixture = TwoLoopFixture::nested();
    let mut progress = PairProgress;
    let mut work = TwoLoopWork::new(77, options(4096), &mut progress);
    assert!(domains(&fixture.bridge.census, [OUTER, INNER], &mut work).is_ok());
    let units = work.retained_units;
    let mut exact = TwoLoopWork::new(77, options(units), &mut progress);
    assert!(domains(&fixture.bridge.census, [OUTER, INNER], &mut exact).is_ok());
    let mut short = TwoLoopWork::new(77, options(units - 1), &mut progress);
    assert!(matches!(
        domains(&fixture.bridge.census, [OUTER, INNER], &mut short)
            .err()
            .unwrap()
            .cause,
        TwoLoopRegionCause::Limit { .. }
    ));
}
#[test]
fn missing_or_duplicate_pair_domains_cannot_become_certified_empty() {
    let mut fixture = TwoLoopFixture::nested();
    fixture.bridge.census.domains.remove(1);
    let mut progress = PairProgress;
    let mut work = TwoLoopWork::new(77, options(4096), &mut progress);
    assert!(domains(&fixture.bridge.census, [OUTER, INNER], &mut work).is_err());
    let mut fixture = TwoLoopFixture::nested();
    fixture.bridge.census.domains[1].occurrences = [0, 0];
    assert!(domains(&fixture.bridge.census, [OUTER, INNER], &mut work).is_err());
}
