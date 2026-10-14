// SPDX-License-Identifier: GPL-2.0-only
//! Independently authored scalar operands; these fixtures do not claim body admission.
use crate::sketch_region_geometry_capture::CapturedMetricCurve;
use crate::sketch_region_source_conformance::CapturedSpaceRepresentation;
use rustycad_tkmath::predicates::roots::{DyadicBound, RootIsolationOptions, metric_bounds::*};

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

pub(super) fn closure_source_query() -> TransportedResidualQuery<'static> {
    TransportedResidualQuery {
        residual: closure_residual(),
        maps: [
            MetricParameterMap::Endpoints([3., 5.]),
            MetricParameterMap::Endpoints([7., 11.]),
        ],
    }
}

fn closure_residual() -> PlanarResidualQuery<'static> {
    let curve = MetricCurve::Line {
        origin: [0.; 3],
        direction: [1., 0., 0.],
    };
    let pcurve = MetricCurve::Line {
        origin: [0.; 2],
        direction: [0.; 2],
    };
    PlanarResidualQuery {
        curve,
        pcurve,
        curve_embedding: identity(),
        plane_embedding: constant_plane(),
        plane_placement: identity(),
        interval: [0., 1.],
        spatial_limit: 0.125,
        options: closure_roots(),
    }
}

fn constant_plane() -> MetricEmbedding<2> {
    MetricEmbedding {
        linear: [[0.; 2]; 3],
        origin: [99.; 3],
        scale: 1.,
    }
}

pub(super) fn closure_roots() -> RootIsolationOptions {
    RootIsolationOptions {
        max_bisections: 0,
        max_degree: 5,
        ..RootIsolationOptions::new(0.125)
    }
}

pub(super) fn bounds(first: f64, last: f64) -> [DyadicBound; 2] {
    [
        DyadicBound::from_f64(first).unwrap(),
        DyadicBound::from_f64(last).unwrap(),
    ]
}
