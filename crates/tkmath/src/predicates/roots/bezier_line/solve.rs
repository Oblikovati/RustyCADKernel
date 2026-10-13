// SPDX-License-Identifier: GPL-2.0-only
use super::super::{
    DyadicBound, RootIsolation, RootIsolationError, RootIsolationOptions, bernstein, chain, isolate,
};
use super::{BezierLineContact, BezierLineError, BezierStage, projection::Projection};
use num_bigint::BigInt;

pub(super) fn classify(
    projected: Projection,
    bounds: [DyadicBound; 2],
    width: DyadicBound,
    options: RootIsolationOptions,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<BezierLineContact, BezierLineError> {
    let denominator = regular_denominator(
        projected.denominator,
        bounds.clone(),
        width.clone(),
        options,
        cancelled,
    );
    let (_, used) = denominator?;
    let mut remaining = options;
    remaining.max_bisections -= used;
    let numerator = solve(projected.numerator, bounds, width, remaining, cancelled);
    complete_numerator(numerator, used)
}
fn solve(
    controls: Vec<BigInt>,
    bounds: [DyadicBound; 2],
    width: DyadicBound,
    options: RootIsolationOptions,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<RootIsolation, RootIsolationError> {
    let polynomial = bernstein::from_integer_controls(controls, options.max_degree, cancelled)?;
    solve_polynomial(polynomial, bounds, width, options, cancelled)
}
pub(super) fn solve_polynomial(
    polynomial: super::super::polynomial::IntegerPolynomial,
    bounds: [DyadicBound; 2],
    width: DyadicBound,
    options: RootIsolationOptions,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<RootIsolation, RootIsolationError> {
    solve_polynomial_accounted(polynomial, bounds, width, options, cancelled, &mut || {})
}
pub(super) fn solve_polynomial_accounted(
    polynomial: super::super::polynomial::IntegerPolynomial,
    bounds: [DyadicBound; 2],
    width: DyadicBound,
    options: RootIsolationOptions,
    cancelled: &mut impl FnMut() -> bool,
    completed_split: &mut impl FnMut(),
) -> Result<RootIsolation, RootIsolationError> {
    let tower = chain::multiplicity_tower(polynomial, cancelled)?;
    isolate::isolate_accounted(&tower, bounds, width, options, cancelled, completed_split)
}
pub(super) fn regular_denominator(
    controls: Vec<BigInt>,
    bounds: [DyadicBound; 2],
    width: DyadicBound,
    options: RootIsolationOptions,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<(super::super::polynomial::IntegerPolynomial, u16), BezierLineError> {
    regular_denominator_certified(controls, bounds, width, options, cancelled)
        .map(|(polynomial, isolation)| (polynomial, isolation.bisections()))
}
pub(super) fn regular_denominator_certified(
    controls: Vec<BigInt>,
    bounds: [DyadicBound; 2],
    width: DyadicBound,
    options: RootIsolationOptions,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<(super::super::polynomial::IntegerPolynomial, RootIsolation), BezierLineError> {
    regular_denominator_accounted(controls, bounds, width, options, cancelled, &mut || {})
}
pub(super) fn regular_denominator_accounted(
    controls: Vec<BigInt>,
    bounds: [DyadicBound; 2],
    width: DyadicBound,
    options: RootIsolationOptions,
    cancelled: &mut impl FnMut() -> bool,
    completed_split: &mut impl FnMut(),
) -> Result<(super::super::polynomial::IntegerPolynomial, RootIsolation), BezierLineError> {
    let polynomial = bernstein::from_integer_controls(controls, options.max_degree, cancelled)
        .map_err(denominator_error)?;
    let result = solve_polynomial_accounted(
        polynomial.clone(),
        bounds,
        width,
        options,
        cancelled,
        completed_split,
    );
    Ok((polynomial, check_denominator(result)?))
}
fn denominator_error(cause: RootIsolationError) -> BezierLineError {
    match cause {
        RootIsolationError::ZeroPolynomial { .. } => BezierLineError::ZeroDenominator,
        cause => BezierLineError::Root {
            stage: BezierStage::Denominator,
            cause,
        },
    }
}
fn check_denominator(
    result: Result<RootIsolation, RootIsolationError>,
) -> Result<RootIsolation, BezierLineError> {
    match result {
        Ok(roots) if roots.roots().is_empty() => Ok(roots),
        Ok(roots) => Err(BezierLineError::DenominatorRoots(roots)),
        Err(cause) => Err(denominator_error(cause)),
    }
}
fn complete_numerator(
    result: Result<RootIsolation, RootIsolationError>,
    used: u16,
) -> Result<BezierLineContact, BezierLineError> {
    match result {
        Ok(mut roots) => {
            roots.bisections += used;
            Ok(BezierLineContact::Isolated(roots))
        }
        Err(RootIsolationError::ZeroPolynomial { .. }) => {
            Ok(BezierLineContact::Coincident { bisections: used })
        }
        Err(cause) => Err(BezierLineError::Root {
            stage: BezierStage::Numerator,
            cause,
        }),
    }
}
