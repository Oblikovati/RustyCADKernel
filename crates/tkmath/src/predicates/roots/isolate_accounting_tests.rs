// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use crate::predicates::roots::polynomial::IntegerPolynomial;
use std::cell::Cell;

fn accounting_tower() -> Vec<SturmChain> {
    let polynomial = IntegerPolynomial::from_binary64(&[-2., 0., 1.], 8).unwrap();
    super::super::chain::multiplicity_tower(polynomial, &mut || false).unwrap()
}
fn accounted_attempt(
    limit: u16,
    cancelled: &mut impl FnMut() -> bool,
    split: &mut impl FnMut(),
) -> Result<RootIsolation, RootIsolationError> {
    let options = RootIsolationOptions {
        max_bisections: limit,
        ..RootIsolationOptions::new(0.001)
    };
    let bounds = [
        DyadicBound::from_f64(0.).unwrap(),
        DyadicBound::from_f64(2.).unwrap(),
    ];
    isolate_accounted(
        &accounting_tower(),
        bounds,
        DyadicBound::from_f64(0.001).unwrap(),
        options,
        cancelled,
        split,
    )
}
#[test]
fn failed_limit_preserves_actual_completed_split_events() {
    let mut actual = 0;
    let result = accounted_attempt(1, &mut || false, &mut || actual += 1);
    assert_eq!(
        result,
        Err(RootIsolationError::SubdivisionLimit { limit: 1 })
    );
    assert_eq!(actual, 1);
}
#[test]
fn cancellation_after_one_split_preserves_that_work() {
    let actual = Cell::new(0);
    let result = accounted_attempt(20, &mut || actual.get() > 0, &mut || {
        actual.set(actual.get() + 1)
    });
    assert_eq!(result, Err(RootIsolationError::Cancelled));
    assert_eq!(actual.get(), 1);
}
#[test]
fn successful_accounting_equals_the_original_result_count() {
    let mut actual = 0;
    let result = accounted_attempt(100, &mut || false, &mut || actual += 1).unwrap();
    assert_eq!(actual, result.bisections());
    assert_eq!(result.roots().len(), 1);
}
#[test]
fn cancellation_before_first_split_charges_no_work() {
    let mut actual = 0;
    let result = accounted_attempt(100, &mut || true, &mut || actual += 1);
    assert_eq!(result, Err(RootIsolationError::Cancelled));
    assert_eq!(actual, 0);
}
