// SPDX-License-Identifier: GPL-2.0-only
//! Exact stored binary64 bytes bind observations without a numerical tolerance or hash.
use super::super::surface_charts::*;

pub(in crate::sketch_region_prism) use crate::sketch_region_source_conformance::encoding::{
    curve, numbers, placement, plane, space,
};

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
pub(in crate::sketch_region_prism) fn chart(value: &CapturedSurfaceChart) -> Vec<u8> {
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
pub(in crate::sketch_region_prism) fn sources(value: &CapturedPrismSources) -> Vec<u8> {
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
