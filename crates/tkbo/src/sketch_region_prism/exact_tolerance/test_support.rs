// SPDX-License-Identifier: GPL-2.0-only
//! Independently authored scalar operands; these fixtures do not claim body admission.
use crate::sketch_region_geometry_capture::CapturedMetricCurve;
use crate::sketch_region_source_conformance::CapturedSpaceRepresentation;
use rustycad_tkmath::predicates::roots::metric_bounds::MetricEmbedding;

pub(super) fn identity() -> MetricEmbedding<3> {
    MetricEmbedding {
        linear: [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
        origin: [0.; 3],
        scale: 1.,
    }
}

pub(super) fn constant_curve(point: [f64; 3]) -> CapturedSpaceRepresentation {
    CapturedSpaceRepresentation {
        curve: CapturedMetricCurve::Line {
            origin: point,
            direction: [0.; 3],
        },
        interval: [0., 1.],
        placement: identity(),
        same_parameter: true,
    }
}
