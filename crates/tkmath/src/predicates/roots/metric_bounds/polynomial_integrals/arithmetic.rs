// SPDX-License-Identifier: GPL-2.0-only
use super::super::super::{
    RootIsolationError, certificate_arithmetic::rounding::outward_binary64,
    polynomial::IntegerPolynomial,
};
use super::super::{
    MetricCurve,
    arithmetic::{difference, product},
    coefficients,
};
use super::*;
use num_bigint::BigInt;

pub(super) fn admit_boundary(
    curves: &[OrientedMetricCurve<'_>],
    options: PolynomialIntegralOptions,
) -> Result<(), PolynomialIntegralFailure> {
    admit_options(options)?;
    if curves.is_empty() {
        return Err(PolynomialIntegralFailure::EmptyBoundary);
    }
    if curves.len() > usize::from(options.max_curves) {
        return Err(PolynomialIntegralFailure::CurveCount {
            actual: curves.len(),
            limit: options.max_curves,
        });
    }
    Ok(())
}
fn admit_curve(
    curve: MetricCurve<'_, 2>,
    options: PolynomialIntegralOptions,
) -> Result<(), PolynomialIntegralFailure> {
    let degree = stored_degree(curve)?;
    if degree > usize::from(options.max_degree) {
        return Err(PolynomialIntegralFailure::Degree {
            actual: degree,
            limit: options.max_degree,
        });
    }
    Ok(())
}
fn admit_options(options: PolynomialIntegralOptions) -> Result<(), PolynomialIntegralFailure> {
    if options.behavior_version != 1
        || options.max_degree > 16
        || options.max_curves == 0
        || options.max_curves > 64
        || !options.absolute_error_limit.is_finite()
        || options.absolute_error_limit < 0.
    {
        return Err(PolynomialIntegralFailure::Policy(options));
    }
    Ok(())
}
fn stored_degree(curve: MetricCurve<'_, 2>) -> Result<usize, PolynomialIntegralFailure> {
    Ok(match curve {
        MetricCurve::Bezier(curve) => {
            if let Some(weights) = curve.weights {
                return Err(PolynomialIntegralFailure::RationalWeights {
                    count: weights.len(),
                });
            }
            curve.poles.len().saturating_sub(1)
        }
        MetricCurve::Line { .. } => 1,
    })
}
pub(in super::super) fn native_plane() -> MetricEmbedding<2> {
    MetricEmbedding {
        linear: [[1., 0.], [0., 1.], [0., 0.]],
        origin: [0.; 3],
        scale: 1.,
    }
}
pub(super) fn curve_integral(
    source: MetricCurve<'_, 2>,
    options: PolynomialIntegralOptions,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<ExactRatio, PolynomialIntegralFailure> {
    poll(cancelled)?;
    admit_curve(source, options)?;
    let curve = coefficients::embedded(source, native_plane(), "integral curve", cancelled)?;
    let numerator = green_numerator(&curve, cancelled)?;
    let denominator = BigInt::from(2) * &curve[3][0] * &curve[3][0];
    Ok(integrate_unit(&numerator, denominator, cancelled)?)
}

pub(in super::super) fn green_numerator(
    curve: &coefficients::HomogeneousCurve,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<Vec<BigInt>, RootIsolationError> {
    // IntegerPolynomial's raw constructor/derivative preserve magnitude; sign normalization would not.
    let dx = IntegerPolynomial(curve[0].clone()).derivative();
    let dy = IntegerPolynomial(curve[1].clone()).derivative();
    let x_dy = product(&curve[0], &dy.0, cancelled)?;
    let y_dx = product(&curve[1], &dx.0, cancelled)?;
    Ok(difference(x_dy, y_dx))
}
pub(in super::super) fn integrate_unit(
    coefficients: &[BigInt],
    denominator: BigInt,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<ExactRatio, RootIsolationError> {
    let mut sum = ExactRatio::from_finite(0.);
    for (power, coefficient) in coefficients.iter().enumerate() {
        poll(cancelled)?;
        let contribution = ExactRatio::new(coefficient.clone(), &denominator * (power + 1));
        sum = sum.add(&contribution);
    }
    Ok(sum)
}
pub(super) fn rounded_measure(
    exact: ExactRatio,
    options: PolynomialIntegralOptions,
    operation: PolynomialIntegralOperation,
) -> Result<CertifiedPolynomialMeasure, PolynomialIntegralFailure> {
    let enclosure = finite_enclosure(&exact)?;
    let absolute_error_bound = certified_rounding_error(&exact, enclosure[0], options)?;
    Ok(CertifiedPolynomialMeasure {
        enclosure,
        absolute_error_bound,
        options,
        operation,
    })
}

fn certified_rounding_error(
    exact: &ExactRatio,
    representative: f64,
    options: PolynomialIntegralOptions,
) -> Result<f64, PolynomialIntegralFailure> {
    let error = exact.subtract(&ExactRatio::from_finite(representative));
    let absolute_error_bound = finite_enclosure(&error)?[1];
    if absolute_error_bound > options.absolute_error_limit {
        return Err(PolynomialIntegralFailure::ErrorLimit {
            requested: options.absolute_error_limit,
            achieved: absolute_error_bound,
        });
    }
    Ok(absolute_error_bound)
}

fn finite_enclosure(exact: &ExactRatio) -> Result<[f64; 2], PolynomialIntegralFailure> {
    outward_binary64(exact).ok_or_else(|| PolynomialIntegralFailure::Unrepresentable {
        exact_value: format!("{exact:?}"),
    })
}
