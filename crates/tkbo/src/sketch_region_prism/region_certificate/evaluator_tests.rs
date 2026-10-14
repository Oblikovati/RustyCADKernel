// SPDX-License-Identifier: GPL-2.0-only
//! Unchanged poles cannot certify geometry whose evaluator has been redirected.
use super::{tests::*, *};
use rustycad_tkbrep::{
    brep::BRepTool,
    topods::{Shape, edge},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{GeomBezierCurve, GeomCurve, GeomLine};
use rustycad_tkmath::gp::{Direction2, Direction3, Point2, Point3};

pub(super) fn redirect_space(shape: &Shape) {
    let raw = BRepTool::curve(&edge(shape).unwrap()).unwrap().0;
    let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_point(
        Point3::new(9., 0., 0.),
        Direction3::new(0., 1., 0.).unwrap(),
    ))
    .cast();
    let descriptor = Handle::new(rustycad_tkg3d::geom_eval::RepCurveDescFull::new(line)).cast();
    raw.cast::<GeomBezierCurve>()
        .set_eval_representation(descriptor)
        .unwrap();
    assert_eq!(
        raw.cast::<GeomBezierCurve>().pole(2).unwrap(),
        Point3::new(4., 0., 0.)
    );
    assert_eq!(raw.value(0.).unwrap(), Point3::new(9., 0., 0.));
}
pub(super) fn redirect_pcurve(shape: &Shape, face: &rustycad_tkbrep::topods::Face) {
    let raw = BRepTool::curve_on_face(&edge(shape).unwrap(), face).unwrap();
    let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(
        Point2::new(9., 0.),
        Direction2::new(0., 1.).unwrap(),
    ))
    .cast();
    let descriptor = Handle::new(rustycad_tkg2d::geom2d_eval::RepCurveDescFull::new(line)).cast();
    raw.pcurve
        .cast::<Geom2dBezierCurve>()
        .set_eval_representation(descriptor)
        .unwrap();
    assert_eq!(
        raw.pcurve.cast::<Geom2dBezierCurve>().pole(2).unwrap(),
        Point2::new(4., 0.)
    );
    assert_eq!(raw.pcurve.value(0.).unwrap(), Point2::new(9., 0.));
}
#[test]
fn alternate_current_original_evaluator_cannot_reuse_coefficients_only_receipt() {
    let (bridge, shapes) = fixture();
    redirect_space(&shapes[0]);
    assert!(bridge.census.source_representation_certified());
    let receipt = run(&bridge, &shapes);
    assert_four_present(&receipt);
    assert_alternate(&receipt.region.unwrap().decision.err().unwrap());
}
#[test]
fn alternate_census_space_evaluator_is_refused() {
    let (bridge, shapes) = fixture();
    redirect_space(&bridge.census.inputs.segments[0].edge);
    assert!(bridge.census.source_representation_certified());
    let receipt = run(&bridge, &shapes);
    assert_four_present(&receipt);
    assert_alternate(&receipt.region.unwrap().decision.err().unwrap());
}
#[test]
fn alternate_census_pcurve_evaluator_is_refused() {
    let (bridge, shapes) = fixture();
    redirect_pcurve(
        &bridge.census.inputs.segments[0].edge,
        &bridge.census.inputs.construction_support,
    );
    assert!(bridge.census.source_representation_certified());
    let receipt = run(&bridge, &shapes);
    assert_four_present(&receipt);
    assert_alternate(&receipt.region.unwrap().decision.err().unwrap());
}
#[test]
fn alternate_candidate_evaluators_are_refused_even_with_unchanged_metadata() {
    for space in [false, true] {
        let (bridge, shapes) = fixture();
        let candidate = &borrowed_batch(&bridge).unwrap().candidates[0];
        let shape = &candidate.wires[0].segments[0].edge;
        if space {
            redirect_space(shape);
        } else {
            redirect_pcurve(shape, &candidate.face);
        }
        let decision = boundary::candidate_boundary(candidate, &bridge.census);
        assert_alternate(&decision.err().unwrap());
        assert_four_present(&run(&bridge, &shapes));
    }
}
fn assert_alternate(cause: &RegionCertificateCause) {
    let RegionCertificateCause::Capture(cause) = cause else {
        panic!("{cause:?}; expected alternate evaluator refusal");
    };
    assert!(matches!(
        cause.as_ref(),
        super::super::surface_charts::PrismSurfaceCause::AlternateEvaluator { .. }
    ));
}

#[test]
fn copied_evaluator_mutations_remove_previously_retained_region_certificates() {
    for space in [false, true] {
        let (bridge, shapes) = fixture();
        let mut receipt = run(&bridge, &shapes);
        let face = rustycad_tkbrep::topods::face(&receipt.copied.as_ref().unwrap().base).unwrap();
        let edge = super::super::isolation_tests::first_edge(&face);
        if space {
            redirect_space(&edge);
        } else {
            redirect_pcurve(&edge, &face);
        }
        retain_certificate(
            &bridge,
            &crate::sketch_region_partition_bindings::binding_sources(&shapes),
            &mut receipt,
        );
        assert_four_present(&receipt);
        assert_alternate(&receipt.region.unwrap().decision.err().unwrap());
    }
}
