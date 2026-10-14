// SPDX-License-Identifier: GPL-2.0-only
use super::super::super::{
    DyadicBound, RootIsolation, bernstein, polynomial::IntegerPolynomial, signs,
};
use super::super::{BezierLineQuery, BezierRayError, ray, solve};
use super::domain_policy::{derived_degrees, domain_failure, domain_limit};
use super::types::*;
use crate::predicates::{exact_dot, exact_point};
use num_bigint::BigInt;
use std::cmp::Ordering;

pub(super) struct DomainIsolation {
    pub normal: IntegerPolynomial,
    pub isolation: RootIsolation,
    pub denominator_isolation: RootIsolation,
    pub origin_signs: Vec<Ordering>,
    pub end_signs: Vec<Ordering>,
    pub degrees: BezierIncidenceDegrees,
}
struct PreparedDomain {
    normal: IntegerPolynomial,
    endpoints: Option<[IntegerPolynomial; 2]>,
    denominator_isolation: RootIsolation,
    degrees: BezierIncidenceDegrees,
}
pub(super) fn isolate_domain(
    query: BezierLineQuery<'_>,
    policy: BezierDomainSignOptions,
    line_domain: BezierLineDomain,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<DomainIsolation, BezierIncidenceCause> {
    let (bounds, width) = super::super::checked_query(query)
        .map_err(|cause| BezierIncidenceCause::Classification(BezierRayError::Line(cause)))?;
    let limit = domain_limit(policy, query.controls.len().saturating_sub(1), line_domain)?;
    let prepared = prepare_domain(query, bounds.clone(), width.clone(), limit, cancelled)
        .map_err(domain_failure)?;
    let mut remaining = query.options;
    remaining.max_bisections -= prepared.denominator_isolation.bisections();
    let mut isolation =
        solve::solve_polynomial(prepared.normal.clone(), bounds, width, remaining, cancelled)
            .map_err(|cause| BezierIncidenceCause::Classification(ray::normal_error(cause)))?;
    isolation.bisections += prepared.denominator_isolation.bisections();
    finish_domain(prepared, isolation, cancelled).map_err(BezierIncidenceCause::Classification)
}
fn prepare_domain(
    query: BezierLineQuery<'_>,
    bounds: [DyadicBound; 2],
    width: DyadicBound,
    limit: Option<u16>,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<PreparedDomain, BezierRayError> {
    let (projection, terminal) =
        captured_projection(query, bounds, width, limit.is_some(), cancelled)?;
    prepare_polynomials(
        projection,
        terminal,
        query.options.max_degree,
        limit,
        query.controls.len() - 1,
        cancelled,
    )
}
fn captured_projection(
    query: BezierLineQuery<'_>,
    bounds: [DyadicBound; 2],
    width: DyadicBound,
    signs_required: bool,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<(ray::RegularProjection, Option<Vec<BigInt>>), BezierRayError> {
    ray::regular_projection_captured(
        query,
        bounds,
        width,
        cancelled,
        |projected, longitudinal, _, _| {
            if !signs_required {
                return Ok(None);
            }
            terminal_controls(query, &projected.denominator, longitudinal).map(Some)
        },
    )
}
fn prepare_polynomials(
    mut projection: ray::RegularProjection,
    terminal: Option<Vec<BigInt>>,
    root_limit: u16,
    limit: Option<u16>,
    basis: usize,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<PreparedDomain, BezierRayError> {
    let normal = normal_polynomial(&mut projection, root_limit, cancelled)?;
    let endpoints = optional_endpoints(&mut projection, terminal, limit, cancelled)?;
    let degrees = derived_degrees(
        basis,
        &normal,
        &projection.denominator,
        endpoints.as_ref(),
        limit,
    );
    Ok(PreparedDomain {
        normal,
        degrees,
        endpoints,
        denominator_isolation: projection.denominator_isolation,
    })
}
fn optional_endpoints(
    projection: &mut ray::RegularProjection,
    terminal: Option<Vec<BigInt>>,
    limit: Option<u16>,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<Option<[IntegerPolynomial; 2]>, BezierRayError> {
    match (terminal, limit) {
        (Some(terminal), Some(limit)) => {
            endpoint_polynomials(projection, terminal, limit, cancelled).map(Some)
        }
        (None, None) => Ok(None),
        (terminal, limit) => {
            unreachable!("terminal {terminal:?}, limit {limit:?}; expected both applied or skipped")
        }
    }
}
fn normal_polynomial(
    projection: &mut ray::RegularProjection,
    limit: u16,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<IntegerPolynomial, BezierRayError> {
    let controls = std::mem::take(&mut projection.normal);
    bernstein::from_integer_controls(controls, limit, cancelled).map_err(ray::normal_error)
}
fn endpoint_polynomials(
    projection: &mut ray::RegularProjection,
    terminal: Vec<BigInt>,
    limit: u16,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<[IntegerPolynomial; 2], BezierRayError> {
    let origin = ray::directed_product(
        std::mem::take(&mut projection.longitudinal),
        projection.denominator.clone(),
        limit,
        cancelled,
    )?;
    let end = ray::directed_product(terminal, projection.denominator.clone(), limit, cancelled)?;
    Ok([origin, end])
}
fn finish_domain(
    prepared: PreparedDomain,
    isolation: RootIsolation,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<DomainIsolation, BezierRayError> {
    let [origin_signs, end_signs] = applied_signs(&prepared, &isolation, cancelled)?;
    Ok(DomainIsolation {
        normal: prepared.normal,
        degrees: prepared.degrees,
        isolation,
        denominator_isolation: prepared.denominator_isolation,
        origin_signs,
        end_signs,
    })
}
fn applied_signs(
    prepared: &PreparedDomain,
    isolation: &RootIsolation,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<[Vec<Ordering>; 2], BezierRayError> {
    let Some([origin, end]) = &prepared.endpoints else {
        return Ok([Vec::new(), Vec::new()]);
    };
    let origin = endpoint_signs(&prepared.normal, origin.clone(), isolation, cancelled)?;
    let end = endpoint_signs(&prepared.normal, end.clone(), isolation, cancelled)?;
    Ok([origin, end])
}
fn endpoint_signs(
    normal: &IntegerPolynomial,
    endpoint: IntegerPolynomial,
    isolation: &RootIsolation,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<Vec<Ordering>, BezierRayError> {
    let result = signs::evaluate_isolated(normal.clone(), endpoint, isolation.clone(), cancelled)
        .map_err(BezierRayError::Signs)?;
    Ok(result.signs().to_vec())
}
fn terminal_controls(
    query: BezierLineQuery<'_>,
    denominator: &[BigInt],
    longitudinal: &[BigInt],
) -> Result<Vec<BigInt>, BezierRayError> {
    // Projection already validated the same endpoints before calling this capture.
    let origin = exact_point(query.line[0], 0).expect("validated line origin");
    let end = exact_point(query.line[1], 1).expect("validated line end");
    let direction = [&end[0] - &origin[0], &end[1] - &origin[1]];
    let squared_length = exact_dot(&direction, &direction);
    Ok(longitudinal
        .iter()
        .zip(denominator)
        .map(|(along, weight)| along - &squared_length * weight)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::super::super::super::RootIsolationOptions;
    use super::*;
    #[test]
    fn rational_quadratic_certifies_two_support_roots_but_one_segment_contact() {
        let controls = [[0., 1.], [0.5, -1.], [1., 1.]];
        let query = BezierLineQuery {
            controls: &controls,
            weights: Some(&[1., 2., 1.]),
            line: [[0., 0.], [0.5, 0.]],
            interval: [0., 1.],
            options: RootIsolationOptions::new(0.001),
        };
        let result = segment_domain(query);
        assert_eq!(result.isolation.roots().len(), 2);
        assert_eq!(result.origin_signs, [Ordering::Greater, Ordering::Greater]);
        assert_eq!(result.end_signs, [Ordering::Less, Ordering::Greater]);
        verify_reversed_weight_signs(query);
    }
    fn segment_domain(query: BezierLineQuery<'_>) -> DomainIsolation {
        isolate_domain(
            query,
            BezierDomainSignOptions::source_basis(),
            BezierLineDomain::Segment,
            &mut || false,
        )
        .unwrap()
    }
    fn verify_reversed_weight_signs(query: BezierLineQuery<'_>) {
        for weight in [1., -1.] {
            let weights = [weight, 2. * weight, weight];
            let reversed = BezierLineQuery {
                weights: Some(&weights),
                line: [query.line[1], query.line[0]],
                ..query
            };
            let reverse = segment_domain(reversed);
            assert_eq!(reverse.origin_signs, [Ordering::Greater, Ordering::Less]);
            assert_eq!(reverse.end_signs, [Ordering::Less, Ordering::Less]);
        }
    }
}
