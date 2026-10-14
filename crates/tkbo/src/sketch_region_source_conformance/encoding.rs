// SPDX-License-Identifier: GPL-2.0-only
//! Exact stored scalar bytes shared by source-bound inspections; no numerical tolerance or hash.
use super::{CapturedPlaneSupport, CapturedSpaceRepresentation};
use crate::sketch_region_geometry_capture::CapturedMetricCurve;
use rustycad_tkmath::predicates::roots::metric_bounds::MetricEmbedding;

pub(crate) fn numbers(values: impl IntoIterator<Item = f64>, bytes: &mut Vec<u8>) {
    for value in values {
        bytes.extend(value.to_bits().to_le_bytes());
    }
}
pub(crate) fn curve<const D: usize>(value: &CapturedMetricCurve<D>, bytes: &mut Vec<u8>) {
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
pub(crate) fn placement<const D: usize>(value: MetricEmbedding<D>, bytes: &mut Vec<u8>) {
    numbers(value.linear.into_iter().flatten(), bytes);
    numbers(value.origin, bytes);
    numbers([value.scale], bytes);
}
pub(crate) fn plane(value: &CapturedPlaneSupport, bytes: &mut Vec<u8>) {
    placement(value.frame, bytes);
    placement(value.placement, bytes);
}
pub(crate) fn space(value: &CapturedSpaceRepresentation, bytes: &mut Vec<u8>) {
    curve(&value.curve, bytes);
    numbers(value.interval, bytes);
    placement(value.placement, bytes);
    bytes.push(u8::from(value.same_parameter));
}
