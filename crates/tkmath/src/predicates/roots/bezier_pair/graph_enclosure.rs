// SPDX-License-Identifier: GPL-2.0-only
//! Enclose the actual algebraic graph without replacing correspondence by point proximity.
use super::super::{
    RootBracket, RootIsolationError,
    certificate_arithmetic::{ExactRatio, RatioInterval, rounding::outward_binary64},
    error::poll,
};
use super::{polynomial::ExactCoefficientPolynomial, projection::LinearFiberProjection};
use num_bigint::BigInt;
#[derive(Clone, Debug, PartialEq)]
/// Original algebraic graph enclosure noncompletion.
pub enum GraphEnvelopeFailure {
    /// Original polynomial evaluation or cancellation cause.
    Root(RootIsolationError),
    /// Alpha interval still contains zero; refinement could not separate it.
    UnresolvedLeadingInterval,
    /// A finite binary64 outward graph enclosure could not be constructed.
    Binary64Overflow,
}
pub(super) struct PairedParameterEnvelope {
    pub exact: RatioInterval,
    pub rounded: [f64; 2],
}
pub(super) fn enclose_u(
    projection: &LinearFiberProjection,
    root: &RootBracket,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<PairedParameterEnvelope, GraphEnvelopeFailure> {
    let exact = exact_graph_interval(projection, root, cancelled)?;
    let rounded = rounded_bounds(&exact)?;
    Ok(PairedParameterEnvelope { exact, rounded })
}
fn exact_graph_interval(
    projection: &LinearFiberProjection,
    root: &RootBracket,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<RatioInterval, GraphEnvelopeFailure> {
    let interval = parameter_interval(root);
    let alpha = power_enclosure(&projection.alpha, &interval, cancelled)?;
    let reciprocal = alpha
        .reciprocal()
        .ok_or(GraphEnvelopeFailure::UnresolvedLeadingInterval)?;
    let beta = power_enclosure(
        &projection.beta.scaled(&BigInt::from(-1)),
        &interval,
        cancelled,
    )?;
    Ok(beta.multiply(&reciprocal))
}

pub(super) fn parameter_interval(root: &RootBracket) -> RatioInterval {
    RatioInterval {
        lower: ExactRatio::from_dyadic(root.bounds().0),
        upper: ExactRatio::from_dyadic(root.bounds().1),
    }
}
fn rounded_bounds(value: &RatioInterval) -> Result<[f64; 2], GraphEnvelopeFailure> {
    let lower = outward_binary64(&value.lower).ok_or(GraphEnvelopeFailure::Binary64Overflow)?;
    let upper = outward_binary64(&value.upper).ok_or(GraphEnvelopeFailure::Binary64Overflow)?;
    Ok([lower[0], upper[1]])
}
fn power_enclosure(
    polynomial: &ExactCoefficientPolynomial,
    parameter: &RatioInterval,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<RatioInterval, GraphEnvelopeFailure> {
    let mut value = RatioInterval::point(ExactRatio::from_finite(0.));
    for coefficient in polynomial.0.iter().rev() {
        poll(cancelled).map_err(GraphEnvelopeFailure::Root)?;
        let term = RatioInterval::point(ExactRatio::new(coefficient.clone(), BigInt::from(1)));
        value = value.multiply(parameter).add(&term);
    }
    Ok(value)
}
