// SPDX-License-Identifier: GPL-2.0-only
//! Exact derivative nonvanishing; no curve injectivity, smooth closure or body acceptance (ADR 0025).
mod access;
mod arithmetic;
mod error;
mod solve;
mod types;
pub use error::{BezierRegularityCause, BezierRegularityFailure, BezierRegularityStage};
pub use types::*;

/// Classify regularity of the supplied placed Bézier over its complete closed interval.
///
/// Signed weights require a root-free denominator. Stationary outcomes are complete
/// classifications, never healthy geometry. Regularity alone does not prove injectivity.
/// ```
/// use rustycad_tkmath::predicates::roots::{RootIsolationOptions, metric_bounds::*};
/// let options = BezierRegularityOptions::v1(RootIsolationOptions::new(0.001));
/// let query = BezierRegularityQuery::new(7, &[[0.,0.,0.],[1.,0.,0.]], options);
/// let report = certify_bezier_regularity(query, || false).unwrap();
/// assert!(matches!(report.decision(), BezierRegularityDecision::Regular { .. }));
/// ```
pub fn certify_bezier_regularity(
    query: BezierRegularityQuery<'_>,
    mut cancelled: impl FnMut() -> bool,
) -> Result<BezierRegularityReport, BezierRegularityFailure> {
    let source = BezierRegularitySource::retain(query);
    let mut bisections = 0;
    match solve::classify(query, &mut cancelled, &mut bisections) {
        Ok(parts) => Ok(parts.report(source, bisections)),
        Err(cause) => Err(BezierRegularityFailure {
            source: Box::new(source),
            cause,
            bisections,
        }),
    }
}
