// SPDX-License-Identifier: GPL-2.0-only
//! Exact winding of a chart point with respect to one authored loop, through the
//! existing certified multi-occurrence Bézier wire classification.
use super::binding::SegmentBinding;
use super::types::*;
use rustycad_tkmath::predicates::roots::RootIsolationOptions;
use rustycad_tkmath::predicates::roots::bezier_line::{
    BezierWireLocation, BezierWireQuery, BezierWireSegment, classify_bezier_wire,
};

/// Classify `point` against `authored`; the wire endpoints must close exactly.
pub(super) fn locate(
    request: &LoopRegionRequest<'_, '_>,
    binding: &SegmentBinding<'_>,
    authored: &AuthoredLoop,
    point: [f64; 2],
    cancelled: &mut impl FnMut() -> bool,
) -> LoopRegionCheck<BezierWireLocation> {
    let options = request.options.winding;
    locate_with(options, binding, authored, point, cancelled)
}
/// `locate` under explicit root isolation options (shared with ADR 0084).
pub(super) fn locate_with(
    options: RootIsolationOptions,
    binding: &SegmentBinding<'_>,
    authored: &AuthoredLoop,
    point: [f64; 2],
    cancelled: &mut impl FnMut() -> bool,
) -> LoopRegionCheck<BezierWireLocation> {
    let segments: Vec<BezierWireSegment<'_>> = authored
        .uses
        .iter()
        .map(|directed| {
            let segment = binding.segment(directed.source);
            BezierWireSegment {
                id: directed.source.0,
                controls: &segment.controls,
                weights: segment.weights.as_deref(),
                interval: [0., 1.],
                reversed: directed.reversed,
            }
        })
        .collect();
    let query = BezierWireQuery {
        segments: &segments,
        line: [point, [point[0] + 1., point[1]]],
        options,
    };
    // Nested numerical stages report cancellation inside their own causes; the observed
    // caller stop is the authoritative classification.
    let mut stopped = false;
    let result = classify_bezier_wire(query, || {
        stopped = stopped || cancelled();
        stopped
    });
    match result {
        Ok(result) => Ok(result.location().clone()),
        Err(_) if stopped => refuse(LoopRegionStage::Nesting, LoopRegionCause::Cancelled),
        Err(cause) => {
            let loop_name = authored.name;
            refuse(
                LoopRegionStage::Nesting,
                LoopRegionCause::Winding { loop_name, cause },
            )
        }
    }
}
/// Jordan winding of a point known to be off the loop: 0, +1 or -1.
pub(super) fn off_loop(
    authored: &AuthoredLoop,
    point: [f64; 2],
    location: BezierWireLocation,
) -> LoopRegionCheck<i64> {
    match location {
        BezierWireLocation::Outside => Ok(0),
        BezierWireLocation::Inside {
            winding: value @ (-1 | 1),
        } => Ok(value),
        actual => {
            let loop_name = authored.name;
            let cause = LoopRegionCause::NonJordanWinding {
                loop_name,
                point,
                actual,
            };
            refuse(LoopRegionStage::Nesting, cause)
        }
    }
}
/// Exact start point of the loop's first directed use, in the census chart.
pub(super) fn witness(binding: &SegmentBinding<'_>, authored: &AuthoredLoop) -> [f64; 2] {
    let first = authored.uses[0];
    let controls = &binding.segment(first.source).controls;
    if first.reversed {
        return *controls.last().expect("nonempty controls");
    }
    controls[0]
}
