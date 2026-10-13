// SPDX-License-Identifier: GPL-2.0-only
use super::super::super::polynomial::IntegerPolynomial;
use super::super::BezierRayError;
use super::types::*;
pub(super) fn domain_limit(
    policy: BezierDomainSignOptions,
    basis: usize,
    domain: BezierLineDomain,
) -> Result<Option<u16>, BezierIncidenceCause> {
    if policy.version != 1 {
        return Err(BezierIncidenceCause::DomainPolicyVersion(policy.version));
    }
    if domain == BezierLineDomain::Support {
        return Ok(None);
    }
    match policy.degree_limit {
        BezierDomainDegreeLimit::Explicit(limit) => Ok(Some(limit)),
        BezierDomainDegreeLimit::SourceBasis => u16::try_from(basis)
            .map(Some)
            .map_err(|_| BezierIncidenceCause::DomainDegreeRepresentation(basis)),
    }
}
pub(super) fn domain_failure(cause: BezierRayError) -> BezierIncidenceCause {
    match cause {
        BezierRayError::Direction(_) => BezierIncidenceCause::DomainSignAdmission(cause),
        _ => BezierIncidenceCause::Classification(cause),
    }
}
pub(super) fn derived_degrees(
    basis: usize,
    normal: &IntegerPolynomial,
    denominator: &IntegerPolynomial,
    endpoints: Option<&[IntegerPolynomial; 2]>,
    limit: Option<u16>,
) -> BezierIncidenceDegrees {
    let [origin_sign_product, terminal_sign_product] =
        endpoints.map_or([None; 2], |ends| ends.each_ref().map(query_degree));
    let denominator = denominator.degree();
    BezierIncidenceDegrees {
        source_basis: basis,
        normal: normal.degree(),
        denominator,
        origin_conversion: origin_sign_product.map(|degree| degree - denominator),
        terminal_conversion: terminal_sign_product.map(|degree| degree - denominator),
        origin_sign_product,
        terminal_sign_product,
        effective_domain_limit: limit,
    }
}
fn query_degree(polynomial: &IntegerPolynomial) -> Option<usize> {
    if polynomial.0.is_empty() {
        return None;
    }
    Some(polynomial.degree())
}
