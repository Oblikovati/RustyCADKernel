// SPDX-License-Identifier: GPL-2.0-only
//! Complete owned scalar descriptors; topology identity cannot substitute for coefficient equality.
use super::types::*;
use crate::sketch_region_geometry_capture::{CapturedGeometryCause, CapturedMetricCurve};
use rustycad_tkmath::predicates::roots::{RootIsolationOptions, metric_bounds::MetricEmbedding};
use std::sync::Arc;
pub(super) fn equal_float_bits<'a>(
    first: impl Iterator<Item = &'a f64>,
    second: impl Iterator<Item = &'a f64>,
) -> bool {
    first.map(|v| v.to_bits()).eq(second.map(|v| v.to_bits()))
}
/// Compare complete stored coefficient bits; `same_metric_curve(&first, &second)`.
pub(crate) fn same_metric_curve<const DIM: usize>(
    first: &CapturedMetricCurve<DIM>,
    second: &CapturedMetricCurve<DIM>,
) -> bool {
    match (first, second) {
        (
            CapturedMetricCurve::Line {
                origin: a,
                direction: b,
            },
            CapturedMetricCurve::Line {
                origin: c,
                direction: d,
            },
        ) => equal_float_bits(a.iter().chain(b), c.iter().chain(d)),
        _ => same_bezier_coefficients(first, second),
    }
}
fn same_bezier_coefficients<const DIM: usize>(
    a: &CapturedMetricCurve<DIM>,
    b: &CapturedMetricCurve<DIM>,
) -> bool {
    let (
        CapturedMetricCurve::Bezier {
            poles: a,
            weights: b,
        },
        CapturedMetricCurve::Bezier {
            poles: c,
            weights: d,
        },
    ) = (a, b)
    else {
        return false;
    };
    equal_float_bits(a.iter().flatten(), c.iter().flatten())
        && b.is_some() == d.is_some()
        && equal_float_bits(b.iter().flatten(), d.iter().flatten())
}

/// Compare separate affine coefficient bits without rounded composition; `same_embedding(&a, &b)`.
pub(crate) fn same_embedding<const DIM: usize>(
    a: &MetricEmbedding<DIM>,
    b: &MetricEmbedding<DIM>,
) -> bool {
    equal_float_bits(
        a.linear
            .iter()
            .flatten()
            .chain(a.origin.iter())
            .chain(std::iter::once(&a.scale)),
        b.linear
            .iter()
            .flatten()
            .chain(b.origin.iter())
            .chain(std::iter::once(&b.scale)),
    )
}
/// Compare captured coefficients and placements without numerical rounding; `same_space_representation(a, b)`.
pub(crate) fn same_space_representation(
    a: &CapturedSpaceRepresentation,
    b: &CapturedSpaceRepresentation,
) -> bool {
    same_metric_curve(&a.curve, &b.curve)
        && equal_float_bits(a.interval.iter(), b.interval.iter())
        && same_embedding(&a.placement, &b.placement)
        && a.same_parameter == b.same_parameter
}
pub(super) fn same_geometry<T>(
    a: &CapturedGeometry<T>,
    b: &CapturedGeometry<T>,
    equal: impl Fn(&T, &T) -> bool,
) -> bool {
    match (a, b) {
        (Ok(a), Ok(b)) => equal(a, b),
        (Err(a), Err(b)) => Arc::<CapturedGeometryCause>::ptr_eq(a, b),
        _ => false,
    }
}
pub(super) fn same_root_policy(a: RootIsolationOptions, b: RootIsolationOptions) -> bool {
    a.version == b.version
        && a.max_degree == b.max_degree
        && a.max_bisections == b.max_bisections
        && a.max_width.to_bits() == b.max_width.to_bits()
}
pub(super) fn same_source_policy(a: SourceConformancePolicy, b: SourceConformancePolicy) -> bool {
    a.original_plane_limit.to_bits() == b.original_plane_limit.to_bits()
        && a.construction_limit.to_bits() == b.construction_limit.to_bits()
        && same_root_policy(a.roots, b.roots)
}
