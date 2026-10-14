// SPDX-License-Identifier: GPL-2.0-only
//! A separate exact-zero query cannot widen or replace the operation's metric request.
use super::frames::*;
use crate::sketch_region_source_conformance::{
    CapturedSpaceRepresentation, captured_endpoint_root_options, certify_captured_placed_endpoint,
};
use rustycad_tkmath::predicates::roots::{DyadicBound, RootIsolationOptions, metric_bounds::*};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum EndpointFrameRole {
    CurveRepresentation,
    VertexOccurrence,
}

pub(super) enum ExactEndpointFailure {
    Frame {
        role: EndpointFrameRole,
        cause: ToleranceFrameFailure,
    },
    Residual(TransportedResidualReport),
}

pub(super) struct ExactEndpointCertificate {
    pub curve_frame: UsableToleranceFrame,
    pub vertex_frame: UsableToleranceFrame,
    pub requested_spatial: f64,
    pub effective_spatial: f64,
    pub effective_roots: RootIsolationOptions,
    pub proven_squared_upper: DyadicBound,
    pub bisections: u64,
}

/// Certify exact raw-point equality independently of operation limits; example: `certify_exact_endpoint(curve, 0., local, placement)`.
pub(super) fn certify_exact_endpoint(
    curve: &CapturedSpaceRepresentation,
    parameter: f64,
    local_point: [f64; 3],
    placement: MetricEmbedding<3>,
) -> Result<ExactEndpointCertificate, ExactEndpointFailure> {
    let curve_frame = endpoint_frame(curve.placement, EndpointFrameRole::CurveRepresentation)?;
    let vertex_frame = endpoint_frame(placement, EndpointFrameRole::VertexOccurrence)?;
    let report = certify_captured_placed_endpoint(curve, parameter, local_point, placement, 0.);
    let bisections = report.bisections;
    let proven_squared_upper =
        exact_zero_bound(&report).ok_or(ExactEndpointFailure::Residual(report))?;
    Ok(zero_certificate(
        curve_frame,
        vertex_frame,
        proven_squared_upper,
        bisections,
    ))
}

fn zero_certificate(
    curve_frame: UsableToleranceFrame,
    vertex_frame: UsableToleranceFrame,
    proven_squared_upper: DyadicBound,
    bisections: u64,
) -> ExactEndpointCertificate {
    ExactEndpointCertificate {
        curve_frame,
        vertex_frame,
        requested_spatial: 0.,
        effective_spatial: 0.,
        effective_roots: captured_endpoint_root_options(),
        proven_squared_upper,
        bisections,
    }
}

fn endpoint_frame(
    placement: MetricEmbedding<3>,
    role: EndpointFrameRole,
) -> Result<UsableToleranceFrame, ExactEndpointFailure> {
    admit_tolerance_frame(placement).map_err(|cause| ExactEndpointFailure::Frame { role, cause })
}

fn exact_zero_bound(report: &TransportedResidualReport) -> Option<DyadicBound> {
    let Ok(ResidualDecision::Within {
        proven_squared_upper,
    }) = &report.decision
    else {
        return None;
    };
    let zero = DyadicBound::from_f64(0.).expect("finite exact zero");
    (*proven_squared_upper == zero).then(|| proven_squared_upper.clone())
}
