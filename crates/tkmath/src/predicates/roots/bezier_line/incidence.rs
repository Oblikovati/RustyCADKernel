// SPDX-License-Identifier: GPL-2.0-only
//! Source-bound incidence certificates extend, rather than replace, mapped solvers.
mod access;
mod construct;
mod domain;
mod domain_policy;
mod error;
mod refine;
#[cfg(test)]
mod tests;
mod types;
use super::super::RootBracket;
pub use types::*;

/// Certify complete bounded or supporting-line incidences and world construction error.
///
/// The owned report binds every certificate to the supplied coefficients; occurrence
/// keys identify sources without asserting confidence. Coincident curves are declined.
/// ```
/// use rustycad_tkmath::predicates::roots::{RootIsolationOptions, bezier_line::*};
/// let curve = BezierLineQuery { controls: &[[0., -1.], [1., 1.]], weights: None,
///     line: [[0., 0.], [1., 0.]], interval: [0., 1.],
///     options: RootIsolationOptions::new(0.001) };
/// let query = BezierIncidenceQuery { curve_id: 7, line_id: 8, curve,
///     domain: BezierLineDomain::Segment, domain_sign: BezierDomainSignOptions::source_basis(), max_world_error: 1e-8 };
/// assert_eq!(certify_bezier_line_incidences(query, || false).unwrap().contacts().len(), 1);
/// ```
pub fn certify_bezier_line_incidences(
    query: BezierIncidenceQuery<'_>,
    mut cancelled: impl FnMut() -> bool,
) -> Result<BezierIncidenceReport, BezierIncidenceFailure> {
    let source = BezierIncidenceSource::retain(query);
    validate_error(query.max_world_error)
        .map_err(|cause| failure(source.clone(), cause, None, None))?;
    let domain =
        domain::isolate_domain(query.curve, query.domain_sign, query.domain, &mut cancelled)
            .map_err(|cause| failure(source.clone(), cause, None, None))?;
    construct::ReportConstruction::new(query, source, domain).construct(&mut cancelled)
}
fn validate_error(value: f64) -> Result<(), BezierIncidenceCause> {
    if !value.is_finite() || value <= 0. {
        return Err(BezierIncidenceCause::WorldErrorLimit(value));
    }
    Ok(())
}
fn failure(
    source: BezierIncidenceSource,
    cause: BezierIncidenceCause,
    support_isolation: Option<super::super::RootIsolation>,
    last_root: Option<RootBracket>,
) -> BezierIncidenceFailure {
    BezierIncidenceFailure {
        evidence: Box::new(BezierIncidenceFailureEvidence {
            source,
            cause,
            support_isolation,
            last_root,
        }),
    }
}
