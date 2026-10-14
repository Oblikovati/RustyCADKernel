// SPDX-License-Identifier: GPL-2.0-only
//! Exact stored binary64 bytes bind observations without a numerical tolerance or hash.
use super::super::surface_charts::*;
use crate::sketch_region_geometry_capture::CapturedMetricCurve;
use crate::sketch_region_source_conformance::{CapturedPlaneSupport, CapturedSpaceRepresentation};
use rustycad_tkmath::predicates::roots::metric_bounds::MetricEmbedding;

pub(in crate::sketch_region_prism) fn numbers(
    values: impl IntoIterator<Item = f64>,
    bytes: &mut Vec<u8>,
) {
    for value in values {
        bytes.extend(value.to_bits().to_le_bytes());
    }
}
pub(super) fn curve<const D: usize>(value: &CapturedMetricCurve<D>, bytes: &mut Vec<u8>) {
    match value {
        CapturedMetricCurve::Bezier { poles, weights } => {
            bytes.push(0);
            bytes.extend((poles.len() as u64).to_le_bytes());
            numbers(poles.iter().flatten().copied(), bytes);
            bytes.push(u8::from(weights.is_some()));
            if let Some(weights) = weights {
                bytes.extend((weights.len() as u64).to_le_bytes());
                numbers(weights.iter().copied(), bytes);
            }
        }
        CapturedMetricCurve::Line { origin, direction } => {
            bytes.push(1);
            numbers(*origin, bytes);
            numbers(*direction, bytes);
        }
    }
}
pub(in crate::sketch_region_prism) fn placement<const D: usize>(
    value: MetricEmbedding<D>,
    bytes: &mut Vec<u8>,
) {
    numbers(value.linear.into_iter().flatten(), bytes);
    numbers(value.origin, bytes);
    numbers([value.scale], bytes);
}
pub(super) fn plane(value: &CapturedPlaneSupport, bytes: &mut Vec<u8>) {
    placement(value.frame, bytes);
    placement(value.placement, bytes);
}
pub(super) fn space(value: &CapturedSpaceRepresentation, bytes: &mut Vec<u8>) {
    curve(&value.curve, bytes);
    numbers(value.interval, bytes);
    placement(value.placement, bytes);
    bytes.push(u8::from(value.same_parameter));
}
pub(super) fn surface(value: &CapturedPrismSurface, bytes: &mut Vec<u8>) {
    match value {
        CapturedPrismSurface::Plane(value) => {
            bytes.push(0);
            plane(value, bytes);
        }
        CapturedPrismSurface::Extrusion {
            basis,
            direction,
            placement: affine,
        } => {
            bytes.push(1);
            curve(basis, bytes);
            numbers(*direction, bytes);
            placement(*affine, bytes);
        }
    }
}
pub(super) fn chart(value: &CapturedSurfaceChart) -> Vec<u8> {
    let mut bytes = Vec::new();
    chart_source(&value.source, &mut bytes);
    surface(&value.actual, &mut bytes);
    numbers(value.extrusion, &mut bytes);
    bytes.extend((value.trims.len() as u64).to_le_bytes());
    for trim in &value.trims {
        trim_values(trim, &mut bytes);
    }
    bytes
}
pub(super) fn chart_source(value: &CapturedChartSource, bytes: &mut Vec<u8>) {
    match value {
        CapturedChartSource::Cap(value) => {
            bytes.push(0);
            plane(value, bytes);
        }
        CapturedChartSource::Side(value) => {
            bytes.push(1);
            space(value, bytes);
        }
    }
}
fn trim_values(value: &CapturedPrismTrim, bytes: &mut Vec<u8>) {
    bytes.push(value.role as u8);
    curve(&value.pcurve, bytes);
    numbers(value.interval, bytes);
    space(&value.space, bytes);
    expectation(&value.expected, bytes);
}
pub(super) fn expectation(value: &CapturedTrimExpectation, bytes: &mut Vec<u8>) {
    match value {
        CapturedTrimExpectation::Cap {
            pcurve,
            interval,
            space: expected,
        } => {
            bytes.push(0);
            curve(pcurve, bytes);
            numbers(*interval, bytes);
            space(expected, bytes);
        }
        CapturedTrimExpectation::SideGenerating => bytes.push(1),
        CapturedTrimExpectation::SideDirecting { parameter } => {
            bytes.push(2);
            numbers([*parameter], bytes);
        }
    }
}
pub(super) fn sources(value: &CapturedPrismSources) -> Vec<u8> {
    let mut bytes = Vec::new();
    plane(&value.support, &mut bytes);
    bytes.extend((value.boundaries.len() as u64).to_le_bytes());
    for boundary in &value.boundaries {
        space(&boundary.space, &mut bytes);
        curve(&boundary.pcurve, &mut bytes);
        numbers(boundary.interval, &mut bytes);
        bytes.extend((boundary.endpoints.len() as u64).to_le_bytes());
        numbers(
            boundary.endpoints.iter().map(|(_, parameter)| *parameter),
            &mut bytes,
        );
    }
    bytes
}
