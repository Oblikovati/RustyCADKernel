// SPDX-License-Identifier: GPL-2.0-only
use super::super::super::{
    DyadicBound, RootIsolation, RootIsolationError, RootIsolationOptions, checked_policy_interval,
    error::poll, polynomial::IntegerPolynomial, polynomial_solve::solve_polynomial_accounted,
};
use super::super::{MetricBezier, coefficients};
use super::*;

pub(super) struct RegularityParts {
    denominator: RootIsolation,
    denominator_degree: usize,
    speed_degree: Option<usize>,
    decision: BezierRegularityDecision,
}
impl RegularityParts {
    pub(super) fn report(
        self,
        source: BezierRegularitySource,
        bisections: u16,
    ) -> BezierRegularityReport {
        BezierRegularityReport {
            source,
            denominator_isolation: self.denominator,
            denominator_degree: self.denominator_degree,
            speed_degree: self.speed_degree,
            decision: self.decision,
            bisections,
        }
    }
}
struct IsolationWork<'a, F: FnMut() -> bool> {
    bounds: [DyadicBound; 2],
    width: DyadicBound,
    options: RootIsolationOptions,
    cancelled: &'a mut F,
    bisections: &'a mut u16,
}
pub(super) fn classify(
    query: BezierRegularityQuery<'_>,
    cancelled: &mut impl FnMut() -> bool,
    bisections: &mut u16,
) -> Result<RegularityParts, BezierRegularityCause> {
    let (bounds, width) = admit(query)?;
    let curve = construct(query, cancelled)?;
    let mut work = IsolationWork {
        bounds,
        width,
        options: query.options.roots,
        cancelled,
        bisections,
    };
    classify_curve(curve, &mut work)
}
fn construct(
    query: BezierRegularityQuery<'_>,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<coefficients::HomogeneousCurve, BezierRegularityCause> {
    admit_stored_degree(query)?;
    coefficients::embedded_signed_bezier(
        MetricBezier {
            poles: query.poles,
            weights: query.weights,
        },
        query.placement,
        "regularity curve",
        cancelled,
    )
    .map_err(BezierRegularityCause::Coefficients)
}
fn admit_stored_degree(query: BezierRegularityQuery<'_>) -> Result<(), BezierRegularityCause> {
    coefficients::validate_curve(
        MetricBezier {
            poles: query.poles,
            weights: query.weights,
        },
        "regularity curve",
    )
    .map_err(BezierRegularityCause::Coefficients)?;
    let actual = query.poles.len() - 1;
    let limit = query.options.max_stored_degree;
    if actual > usize::from(limit) {
        return Err(BezierRegularityCause::StoredDegree { actual, limit });
    }
    Ok(())
}
fn admit(
    query: BezierRegularityQuery<'_>,
) -> Result<([DyadicBound; 2], DyadicBound), BezierRegularityCause> {
    if query.options.version != 1 {
        return Err(BezierRegularityCause::Version(query.options.version));
    }
    let [lower, upper] = query.interval;
    if !lower.is_finite() || !upper.is_finite() || lower < 0. || upper > 1. || lower > upper {
        return Err(BezierRegularityCause::Domain(query.interval));
    }
    checked_policy_interval(query.interval, query.options.roots)
        .map_err(|cause| root_cause(BezierRegularityStage::Policy, cause))
}
fn classify_curve(
    curve: coefficients::HomogeneousCurve,
    work: &mut IsolationWork<'_, impl FnMut() -> bool>,
) -> Result<RegularityParts, BezierRegularityCause> {
    let denominator = denominator_polynomial(&curve[3], work.options.max_degree)?;
    let denominator_degree = denominator.degree();
    let isolation = work.isolate(denominator, BezierRegularityStage::Denominator)?;
    if !isolation.roots().is_empty() {
        return Err(BezierRegularityCause::DenominatorZeros(isolation));
    }
    let square = arithmetic::squared_speed(&curve, work.cancelled)
        .map_err(|cause| root_cause(BezierRegularityStage::SquaredSpeed, cause))?;
    let (speed_degree, decision) = classify_square(square, work)?;
    poll(work.cancelled).map_err(|cause| root_cause(BezierRegularityStage::SquaredSpeed, cause))?;
    Ok(RegularityParts {
        denominator: isolation,
        denominator_degree,
        speed_degree,
        decision,
    })
}
fn denominator_polynomial(
    coefficients: &[num_bigint::BigInt],
    limit: u16,
) -> Result<IntegerPolynomial, BezierRegularityCause> {
    match IntegerPolynomial::from_integers(coefficients.to_vec(), limit) {
        Err(RootIsolationError::ZeroPolynomial { .. }) => {
            Err(BezierRegularityCause::ZeroDenominator)
        }
        result => result.map_err(|cause| root_cause(BezierRegularityStage::Denominator, cause)),
    }
}
fn classify_square(
    coefficients: Vec<num_bigint::BigInt>,
    work: &mut IsolationWork<'_, impl FnMut() -> bool>,
) -> Result<(Option<usize>, BezierRegularityDecision), BezierRegularityCause> {
    let square = IntegerPolynomial::accept_zero(IntegerPolynomial::from_integers(
        coefficients,
        work.options.max_degree,
    ))
    .map_err(|cause| root_cause(BezierRegularityStage::SquaredSpeed, cause))?;
    if square.0.is_empty() {
        return Ok((None, BezierRegularityDecision::EverywhereStationary));
    }
    let degree = square.degree();
    let isolation = work.isolate(square, BezierRegularityStage::SquaredSpeed)?;
    Ok((Some(degree), isolated_decision(isolation)))
}
fn isolated_decision(isolation: RootIsolation) -> BezierRegularityDecision {
    if isolation.roots().is_empty() {
        return BezierRegularityDecision::Regular {
            zero_speed_isolation: isolation,
        };
    }
    BezierRegularityDecision::Stationary {
        zero_speed_isolation: isolation,
    }
}

impl<F: FnMut() -> bool> IsolationWork<'_, F> {
    fn isolate(
        &mut self,
        polynomial: IntegerPolynomial,
        stage: BezierRegularityStage,
    ) -> Result<RootIsolation, BezierRegularityCause> {
        let mut remaining = self.options;
        remaining.max_bisections -= *self.bisections;
        solve_polynomial_accounted(
            polynomial,
            self.bounds.clone(),
            self.width.clone(),
            remaining,
            self.cancelled,
            &mut || *self.bisections += 1,
        )
        .map_err(|cause| root_cause(stage, cause))
    }
}
fn root_cause(stage: BezierRegularityStage, cause: RootIsolationError) -> BezierRegularityCause {
    BezierRegularityCause::Roots { stage, cause }
}
