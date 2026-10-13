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
    let denominator = solve(
        projected.denominator,
        bounds.clone(),
        width.clone(),
        options,
        cancelled,
    );
    let used = check_denominator(denominator)?;
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
    let tower = chain::multiplicity_tower(polynomial, cancelled)?;
    isolate::isolate(&tower, bounds, width, options, cancelled)
}
fn check_denominator(
    result: Result<RootIsolation, RootIsolationError>,
) -> Result<u16, BezierLineError> {
    match result {
        Ok(roots) if roots.roots().is_empty() => Ok(roots.bisections()),
        Ok(roots) => Err(BezierLineError::DenominatorRoots(roots)),
        Err(RootIsolationError::ZeroPolynomial { .. }) => Err(BezierLineError::ZeroDenominator),
        Err(cause) => Err(BezierLineError::Root {
            stage: BezierStage::Denominator,
            cause,
        }),
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
