// SPDX-License-Identifier: GPL-2.0-only
use super::*;
use crate::sketch_region_geometry_capture::CapturedMetricCurve;
use crate::sketch_region_prism::surface_charts::CapturedPrismSurface;
use crate::sketch_region_source_conformance::{CapturedPlaneSupport, CapturedSpaceRepresentation};
use rustycad_tkmath::predicates::roots::metric_bounds::{MetricCurve, MetricEmbedding};

fn placement<const D: usize>() -> MetricEmbedding<D> {
    MetricEmbedding {
        linear: [[-0.; D], [2.; D], [-3.; D]],
        origin: [9007199254740992., -0., 7.],
        scale: -2.,
    }
}
fn placement_bits<const D: usize>(value: MetricEmbedding<D>) -> Vec<u64> {
    value
        .linear
        .into_iter()
        .flatten()
        .chain(value.origin)
        .chain([value.scale])
        .map(f64::to_bits)
        .collect()
}
fn assert_placement<const D: usize>(actual: MetricEmbedding<D>, expected: MetricEmbedding<D>) {
    let actual = placement_bits(actual);
    let expected = placement_bits(expected);
    assert_eq!(actual, expected);
}
fn captured_line() -> CapturedSpaceRepresentation {
    CapturedSpaceRepresentation {
        curve: CapturedMetricCurve::Line {
            origin: [1., -0., 3.],
            direction: [0., 2., 0.],
        },
        interval: [-0., 5.],
        placement: placement(),
        same_parameter: true,
    }
}
#[test]
fn space_view_retains_native_range_and_separate_affine_operands() {
    let captured = captured_line();
    let view = PrismBodySpaceCurveView::borrow(&captured);
    assert_eq!(
        view.native_interval.map(f64::to_bits),
        captured.interval.map(f64::to_bits)
    );
    assert_placement(view.representation_placement, captured.placement);
    let MetricCurve::Line { origin, direction } = view.curve else {
        panic!("expected captured line");
    };
    assert_eq!(origin.map(f64::to_bits), [1., -0., 3.].map(f64::to_bits));
    assert_eq!(direction, [0., 2., 0.]);
    assert!(view.same_parameter);
}
#[test]
fn plane_view_preserves_frame_then_placement_without_multiplying() {
    let plane = CapturedPlaneSupport {
        frame: placement(),
        placement: placement(),
    };
    let captured = CapturedPrismSurface::Plane(plane.clone());
    let PrismBodySurfaceView::Plane {
        frame,
        representation_placement,
    } = PrismBodySurfaceView::borrow(&captured)
    else {
        panic!("expected captured plane");
    };
    assert_placement(frame, plane.frame);
    assert_placement(representation_placement, plane.placement);
}
#[test]
fn extrusion_view_borrows_owned_poles_and_keeps_direction_bits() {
    let captured = extrusion();
    let PrismBodySurfaceView::Extrusion {
        basis,
        direction,
        representation_placement,
    } = PrismBodySurfaceView::borrow(&captured)
    else {
        panic!("expected captured extrusion");
    };
    assert_borrowed_basis(basis, &captured);
    assert_eq!(
        direction.map(f64::to_bits),
        [0., -0., -2.].map(f64::to_bits)
    );
    assert_placement(representation_placement, placement::<3>());
}
fn assert_borrowed_basis(basis: MetricCurve<'_, 3>, captured: &CapturedPrismSurface) {
    let MetricCurve::Bezier(curve) = basis else {
        panic!("expected captured Bezier");
    };
    let CapturedPrismSurface::Extrusion {
        basis: CapturedMetricCurve::Bezier { poles, .. },
        ..
    } = captured
    else {
        panic!("expected stored Bezier extrusion");
    };
    assert!(std::ptr::eq(curve.poles, poles.as_slice()));
    assert!(curve.weights.is_none());
}
fn extrusion() -> CapturedPrismSurface {
    CapturedPrismSurface::Extrusion {
        basis: CapturedMetricCurve::Bezier {
            poles: vec![[0.; 3], [1.; 3]],
            weights: None,
        },
        direction: [0., -0., -2.],
        placement: placement(),
    }
}
#[test]
fn scalar_read_views_are_send_sync_without_geometry_owners() {
    static_assertions::assert_impl_all!(PrismBodySurfaceView<'static>: Send, Sync);
    static_assertions::assert_impl_all!(PrismBodySpaceCurveView<'static>: Send, Sync);
}
