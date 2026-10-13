// SPDX-License-Identifier: GPL-2.0-only
use super::outcome::AffectedOccurrence;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::predicates::roots::{
    RootIsolationOptions,
    metric_bounds::{ResidualDecision, ResidualError},
};
#[derive(Debug)]
pub(crate) enum PlanarMetricCause {
    Native(Failure),
    MissingCurve {
        representation: &'static str,
    },
    Unsupported {
        representation: &'static str,
        dynamic_type: String,
    },
    ParameterRanges {
        curve: [f64; 2],
        pcurve: [f64; 2],
    },
    MissingSameParameter,
    MissingStoredPCurve,
    Exact(ResidualError),
}
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
impl From<Failure> for PlanarMetricCause {
    fn from(cause: Failure) -> Self {
        Self::Native(cause)
    }
}
impl From<ResidualError> for PlanarMetricCause {
    fn from(cause: ResidualError) -> Self {
        Self::Exact(cause)
    }
}
