// SPDX-License-Identifier: GPL-2.0-only
//! Native parameter measurements retain typed evidence; they never prove contact identity.
use super::BoundaryCensus;
use crate::sketch_region_geometry_capture::CapturedMetricCurve;
use crate::sketch_region_source_conformance::*;
use rustycad_tkmath::predicates::roots::{
    bezier_line::CertifiedBezierIncidence, metric_bounds::MetricParameterMap,
};
pub(super) fn contact_native_membership(
    census: &BoundaryCensus,
    index: usize,
    contact: &CertifiedBezierIncidence,
    parameter: f64,
) -> Result<NativeParameterMembership, SourceConformanceCause> {
    let construction = current_construction(census, index)?;
    let map = construction_map(construction)?;
    let (lower, upper) = contact.parameter().bounds();
    native_parameter_membership(map, &[lower.clone(), upper.clone()], parameter)
        .map_err(SourceConformanceCause::Exact)
}

fn construction_map(
    construction: &CapturedBoundaryConstruction,
) -> Result<MetricParameterMap, SourceConformanceCause> {
    let pcurve = construction
        .pcurve
        .as_ref()
        .map_err(|cause| SourceConformanceCause::Capture(cause.clone()))?;
    Ok(match pcurve {
        CapturedMetricCurve::Line { .. } => {
            MetricParameterMap::Endpoints(construction.native_interval)
        }
        CapturedMetricCurve::Bezier { .. } => MetricParameterMap::Identity,
    })
}

fn current_construction(
    census: &BoundaryCensus,
    index: usize,
) -> Result<&CapturedBoundaryConstruction, SourceConformanceCause> {
    let segment = parameter_segment(census, index)?;
    let mut matches = census
        .inputs
        .constructions
        .iter()
        .filter(|c| c.occurrence == segment.occurrence);
    let capture = matches
        .next()
        .ok_or(SourceConformanceCause::MissingConstruction {
            occurrence: segment.occurrence,
        })?;
    if matches.next().is_some() || !binds_current_segment(capture, segment) {
        return Err(SourceConformanceCause::CurrentDescriptorMismatch {
            occurrence: segment.occurrence,
        });
    }
    current_construction_correspondence(&census.inputs, capture, segment)?;
    Ok(capture)
}

fn parameter_segment(
    census: &BoundaryCensus,
    index: usize,
) -> Result<&crate::sketch_region_candidates::CandidateSegment, SourceConformanceCause> {
    census
        .inputs
        .segments
        .get(index)
        .ok_or(SourceConformanceCause::InvalidSegmentIndex {
            received: index,
            expected_length: census.inputs.segments.len(),
        })
}
