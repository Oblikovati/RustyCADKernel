// SPDX-License-Identifier: GPL-2.0-only
use super::outcome::AffectedOccurrence;
pub(crate) use crate::sketch_region_geometry_capture::CapturedGeometryCause as PlanarMetricCause;
use rustycad_tkmath::predicates::roots::{RootIsolationOptions, metric_bounds::ResidualDecision};
pub(crate) struct PlanarMetricInspection {
    pub edge: AffectedOccurrence,
    pub face: AffectedOccurrence,
    pub requested_spatial_limit: f64,
    pub effective_spatial_limit: f64,
    pub root_policy: RootIsolationOptions,
    // Err is an inconclusive computation, never a valid metric certificate.
    pub decision: Result<ResidualDecision, PlanarMetricCause>,
    // Legacy root errors cannot reveal failed work; never fabricate a zero count.
    pub failed_subdivision_work: Option<u32>,
    // Bound belongs to these captured coefficients, not later edits of affected Shape aliases.
    pub coefficient_snapshot: Option<super::tolerance_capture::CapturedPlanarPair>,
}
