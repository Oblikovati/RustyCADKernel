//! ChFi3d_PerformElSpine on an open polyline: the guide follows the spine on its range
//! and is extended straight beyond the spine ends.
use rustycad_tkernel::standard::Handle;
use rustycad_tkfillet::chfi_ds::{ElSpine, Spine, SpineHandle, lock};
use rustycad_tkfillet::chfi3d::perform_el_spine;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::geom_abs::{CurveType, Shape as Continuity};
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
use std::sync::{Arc, Mutex};

/// The spine of the polyline (0,0,0) (3,0,0) (3,4,0), loaded.
fn polyline_spine() -> SpineHandle {
    let pts = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(3.0, 0.0, 0.0),
        Point3::new(3.0, 4.0, 0.0),
    ];
    let mut spine = Spine::new();
    for w in pts.windows(2) {
        spine.set_edges(
            &BRepBuilderAPIMakeEdge::from_points(w[0], w[1])
                .unwrap()
                .edge()
                .unwrap(),
        );
    }
    spine.load().unwrap();
    Arc::new(Mutex::new(spine))
}

/// The elementary spine [wf, wl] with its ends taken on the spine.
fn el_spine(spine: &SpineHandle, (wf, wl): (f64, f64)) -> Handle<ElSpine> {
    let es = Handle::new(ElSpine::new());
    let e = es.get().unwrap();
    e.set_first_parameter(wf);
    e.set_last_parameter(wl);
    let mut guard = lock(spine);
    let (p, v) = guard.spine_mut().d1(wf).unwrap();
    e.set_first_point_and_tgt(p, v);
    let (p, v) = guard.spine_mut().d1(wl).unwrap();
    e.set_last_point_and_tgt(p, v);
    es
}

#[test]
fn the_guide_of_one_edge_follows_the_edge() {
    let spine = polyline_spine();
    let es = el_spine(&spine, (0.5, 2.5));
    perform_el_spine(&es, &spine, Continuity::C2, 1e-4, false).unwrap();
    let e = es.get().unwrap();
    assert_eq!(e.get_type().unwrap(), CurveType::BSplineCurve);
    assert!(e.value(1.5).unwrap().distance(&Point3::new(1.5, 0.0, 0.0)) < 1e-9);
}

#[test]
fn the_guide_across_the_corner_is_one_smooth_curve() {
    let spine = polyline_spine();
    let es = el_spine(&spine, (1.0, 6.0));
    perform_el_spine(&es, &spine, Continuity::C2, 1e-4, false).unwrap();
    let e = es.get().unwrap();
    let bs = e.bspline().unwrap();
    let b = bs.get().unwrap();
    assert!(b.degree() >= 5, "smoothed to C2 at least");
    assert_eq!(e.nb_vertices(), 1, "the corner vertex with its tangent");
    assert!(
        e.value(e.first_parameter().unwrap())
            .unwrap()
            .distance(&Point3::new(1.0, 0.0, 0.0))
            < 1e-3
    );
}
