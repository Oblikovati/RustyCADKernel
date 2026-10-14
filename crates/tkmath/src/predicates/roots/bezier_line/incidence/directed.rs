// SPDX-License-Identifier: GPL-2.0-only
//! Retain endpoint signs before the shared constructor consumes the support isolation.
mod error;
#[cfg(test)]
mod tests;
mod types;
use super::{
    BezierIncidenceSource, BezierLineDomain, construct::ReportConstruction, domain::DomainIsolation,
};
pub use error::{DirectedBezierIncidenceCause, DirectedBezierIncidenceFailure};
use std::cmp::Ordering;
pub use types::*;

/// Certify exact positions and finite world construction using one complete support isolation.
///
/// Endpoint admission applies even for Support. Positions cover all support roots;
/// contacts retain the original domain filter. Neither order is nearest-hit order.
/// ```
/// use rustycad_tkmath::predicates::roots::{RootIsolationOptions, bezier_line::*};
/// let curve = BezierLineQuery { controls: &[[0., -1.], [1., 1.]], weights: None,
///     line: [[0., 0.], [1., 0.]], interval: [0., 1.], options: RootIsolationOptions::new(0.001) };
/// let query = BezierIncidenceQuery { curve_id: 7, line_id: 8, curve,
///     domain: BezierLineDomain::Support, domain_sign: BezierDomainSignOptions::source_basis(), max_world_error: 1e-8 };
/// let report = certify_directed_bezier_incidences(DirectedBezierIncidenceQuery::v1(query), || false).unwrap();
/// assert_eq!(report.positions(), &[DirectedBezierPosition::Interior]);
/// ```
pub fn certify_directed_bezier_incidences(
    query: DirectedBezierIncidenceQuery<'_>,
    mut cancelled: impl FnMut() -> bool,
) -> Result<DirectedBezierIncidenceReport, DirectedBezierIncidenceFailure> {
    certify_version(query)?;
    let (source, domain) =
        super::prepare_incidence(query.incidence, BezierLineDomain::Segment, &mut cancelled)
            .map_err(|cause| inherited_failure(query.version, cause))?;
    construct_directed(query, source, domain, &mut cancelled)
}
fn certify_version(
    query: DirectedBezierIncidenceQuery<'_>,
) -> Result<(), DirectedBezierIncidenceFailure> {
    if query.version == 1 {
        return Ok(());
    }
    Err(DirectedBezierIncidenceFailure {
        version: query.version,
        source: Box::new(BezierIncidenceSource::retain(query.incidence)),
        cause: DirectedBezierIncidenceCause::Version(query.version),
    })
}
fn inherited_failure(
    version: u32,
    cause: super::BezierIncidenceFailure,
) -> DirectedBezierIncidenceFailure {
    DirectedBezierIncidenceFailure {
        version,
        source: Box::new(cause.source().clone()),
        cause: DirectedBezierIncidenceCause::Incidence(cause),
    }
}
fn construct_directed(
    query: DirectedBezierIncidenceQuery<'_>,
    source: BezierIncidenceSource,
    domain: DomainIsolation,
    cancelled: &mut impl FnMut() -> bool,
) -> Result<DirectedBezierIncidenceReport, DirectedBezierIncidenceFailure> {
    let positions =
        endpoint_positions(&domain).map_err(|cause| DirectedBezierIncidenceFailure {
            version: query.version,
            source: Box::new(source.clone()),
            cause,
        })?;
    let incidence = ReportConstruction::new(query.incidence, source, domain)
        .construct(cancelled)
        .map_err(|cause| inherited_failure(query.version, cause))?;
    Ok(DirectedBezierIncidenceReport {
        version: query.version,
        incidence,
        positions,
    })
}
fn endpoint_positions(
    domain: &DomainIsolation,
) -> Result<Vec<DirectedBezierPosition>, DirectedBezierIncidenceCause> {
    verify_alignment(domain)?;
    domain
        .origin_signs
        .iter()
        .zip(&domain.end_signs)
        .enumerate()
        .map(|(index, (&origin, &terminal))| endpoint_position(index, origin, terminal))
        .collect()
}
fn verify_alignment(domain: &DomainIsolation) -> Result<(), DirectedBezierIncidenceCause> {
    let counts = (
        domain.isolation.roots().len(),
        domain.origin_signs.len(),
        domain.end_signs.len(),
    );
    if counts.0 != counts.1 || counts.0 != counts.2 {
        return Err(DirectedBezierIncidenceCause::Alignment {
            roots: counts.0,
            origins: counts.1,
            terminals: counts.2,
        });
    }
    Ok(())
}
fn endpoint_position(
    index: usize,
    origin: Ordering,
    terminal: Ordering,
) -> Result<DirectedBezierPosition, DirectedBezierIncidenceCause> {
    use DirectedBezierPosition::*;
    use Ordering::*;
    match (origin, terminal) {
        (Less, Less) => Ok(BeforeOrigin),
        (Equal, Less) => Ok(AtOrigin),
        (Greater, Less) => Ok(Interior),
        (Greater, Equal) => Ok(AtTerminal),
        (Greater, Greater) => Ok(BeyondTerminal),
        _ => Err(DirectedBezierIncidenceCause::EndpointOrder {
            support_root_index: index,
            origin,
            terminal,
        }),
    }
}
