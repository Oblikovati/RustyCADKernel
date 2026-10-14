// SPDX-License-Identifier: GPL-2.0-only
//! Shared exact-engine entry; compatibility callers keep their original no-op observer.
use super::{DyadicBound, RootIsolation, RootIsolationError, RootIsolationOptions, chain, isolate};
pub(super) fn solve_polynomial(
    polynomial: super::polynomial::IntegerPolynomial,
    bounds: [DyadicBound; 2],
    width: DyadicBound,
    options: RootIsolationOptions,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<RootIsolation, RootIsolationError> {
    solve_polynomial_accounted(polynomial, bounds, width, options, cancelled, &mut || {})
}
pub(super) fn solve_polynomial_accounted(
    polynomial: super::polynomial::IntegerPolynomial,
    bounds: [DyadicBound; 2],
    width: DyadicBound,
    options: RootIsolationOptions,
    cancelled: &mut impl FnMut() -> bool,
    completed_split: &mut impl FnMut(),
) -> Result<RootIsolation, RootIsolationError> {
    let tower = chain::multiplicity_tower(polynomial, cancelled)?;
    isolate::isolate_accounted(&tower, bounds, width, options, cancelled, completed_split)
}
