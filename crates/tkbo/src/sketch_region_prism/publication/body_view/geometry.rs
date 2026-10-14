// SPDX-License-Identifier: GPL-2.0-only
//! Borrow captured scalar coefficients without exposing geometry handles or evaluating transforms.
use crate::sketch_region_prism::surface_charts::CapturedPrismSurface;
use crate::sketch_region_source_conformance::CapturedSpaceRepresentation;
use rustycad_tkmath::predicates::roots::metric_bounds::{MetricCurve, MetricEmbedding};

#[derive(Clone, Copy)]
pub(crate) struct PrismBodySpaceCurveView<'a> {
    pub(crate) curve: MetricCurve<'a, 3>,
    pub(crate) native_interval: [f64; 2],
    /// Already captured representation placement; occurrence placement is not applied again.
    pub(crate) representation_placement: MetricEmbedding<3>,
    pub(crate) same_parameter: bool,
}
#[derive(Clone, Copy)]
pub(crate) enum PrismBodySurfaceView<'a> {
    Plane {
        frame: MetricEmbedding<2>,
        representation_placement: MetricEmbedding<3>,
    },
    Extrusion {
        basis: MetricCurve<'a, 3>,
        direction: [f64; 3],
        representation_placement: MetricEmbedding<3>,
    },
}
impl<'a> PrismBodySpaceCurveView<'a> {
    pub(super) fn borrow(captured: &'a CapturedSpaceRepresentation) -> Self {
        Self {
            curve: captured.curve.borrow(),
            native_interval: captured.interval,
            representation_placement: captured.placement,
            same_parameter: captured.same_parameter,
        }
    }
}
impl<'a> PrismBodySurfaceView<'a> {
    pub(super) fn borrow(captured: &'a CapturedPrismSurface) -> Self {
        match captured {
            CapturedPrismSurface::Plane(plane) => Self::Plane {
                frame: plane.frame,
                representation_placement: plane.placement,
            },
            CapturedPrismSurface::Extrusion {
                basis,
                direction,
                placement,
            } => Self::Extrusion {
                basis: basis.borrow(),
                direction: *direction,
                representation_placement: *placement,
            },
        }
    }
}
