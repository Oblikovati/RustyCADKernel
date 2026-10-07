//! The ChFi3d_Builder_0 edges of fillets, stripe ends, tolerances and solved points.
use rustycad_tkbrep::topexp;
use rustycad_tkernel::standard::Handle;
use rustycad_tkfillet::chfi_ds::{CommonPoint, Spine, Stripe, SurfData, shared};
use rustycad_tkfillet::chfi3d as b0;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomPlane, GeomSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkgeomalgo::int_surf::IntSurfTypeTrans;
use rustycad_tkmath::gp::{Axis1, Axis3, Direction3, Point2, Point3};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
use std::sync::{Arc, Mutex};

fn plane_xy() -> Handle<dyn GeomSurface> {
    Handle::new(GeomPlane::new(Axis3::default())).cast()
}

fn line(p: Point3, d: (f64, f64, f64)) -> Handle<dyn GeomCurve> {
    Handle::new(GeomLine::new(Axis1::new(
        p,
        Direction3::new(d.0, d.1, d.2).unwrap(),
    )))
    .cast()
}

#[test]
fn trsf_trans_and_search_pivot() {
    assert_eq!(b0::trsf_trans(IntSurfTypeTrans::In), Orientation::Forward);
    assert_eq!(
        b0::trsf_trans(IntSurfTypeTrans::Touch),
        Orientation::Internal
    );
    // Fillet 1 crosses the section of fillet 0 on the opposite face, so 0 is no pivot.
    let u = [[0.0, 0.0, 0.0], [-5.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
    assert_eq!(b0::search_pivot(&[1, 1, 1], &u, 1e-9), 1);
    // Each candidate has one neighbour section on the wrong side.
    let u = [[0.0, 1.0, 0.0], [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]];
    assert_eq!(b0::search_pivot(&[-1, -1, -1], &u, 0.0), -1);
}

#[test]
fn compute_arete_along_an_iso_of_a_plane() {
    let (p1, p2) = (CommonPoint::new(), CommonPoint::new());
    let mut arete = b0::Arete::default();
    b0::compute_arete(
        (&p1, Point2::new(1.0, 3.0)),
        (&p2, Point2::new(1.0, 0.0)),
        &plane_xy(),
        &mut arete,
        (1e-4, 1e-5),
        0,
    )
    .unwrap();
    assert!(!arete.c3d.is_null());
    let c = arete.c3d.get().unwrap();
    // The decreasing iso is reversed: it runs from V = 3 to V = 0.
    assert!(
        c.value(arete.pardeb)
            .unwrap()
            .distance(&Point3::new(1.0, 3.0, 0.0))
            < 1e-12
    );
    assert!(
        c.value(arete.parfin)
            .unwrap()
            .distance(&Point3::new(1.0, 0.0, 0.0))
            < 1e-12
    );
    let pc = arete.pcurv.get().unwrap();
    assert!(
        pc.value(arete.pardeb)
            .unwrap()
            .distance(Point2::new(1.0, 3.0))
            < 1e-12
    );
}

#[test]
fn stripe_ends_give_the_first_or_last_surface() {
    let pts = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
    ];
    let edges: Vec<_> = pts
        .windows(2)
        .map(|w| {
            BRepBuilderAPIMakeEdge::from_points(w[0], w[1])
                .unwrap()
                .edge()
                .unwrap()
        })
        .collect();
    let mut spine = Spine::new();
    edges.iter().for_each(|e| spine.set_edges(e));
    let mut stripe = Stripe::new();
    *stripe.change_spine() = Some(Arc::new(Mutex::new(spine)));
    *stripe.change_set_of_surf_data() =
        Some(shared((0..3).map(|_| shared(SurfData::new())).collect()));
    let st = shared(stripe);
    let start = topexp::first_vertex(&edges[0], false);
    let end = topexp::last_vertex(&edges[1], false);
    assert_eq!(b0::index_of_surf_data(&start, &st).unwrap(), (1, 1));
    assert_eq!(b0::index_of_surf_data(&end, &st).unwrap(), (3, -1));
    let (edge, sens) = b0::edge_from_v1(&end, &st).unwrap();
    assert!(edge.is_same(&edges[1]) && sens == -1);
    let inner = topexp::last_vertex(&edges[0], false);
    assert!(b0::edge_from_v1(&inner, &st).is_err());
}

#[test]
fn int_cs_keeps_the_asked_cut() {
    let s: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_bounds(&plane_xy(), (-5.0, 5.0, -5.0, 5.0)).unwrap())
            .cast();
    let c = line(Point3::new(1.0, 2.0, -3.0), (0.0, 0.0, 1.0));
    let hc: Handle<dyn Adaptor3dCurve> =
        Handle::new(GeomAdaptorCurve::with_range(&c, 0.0, 10.0).unwrap()).cast();
    let (p, w) = b0::int_cs(&s, &hc, 0.0).unwrap().expect("a cut");
    assert!(p.distance(Point2::new(1.0, 2.0)) < 1e-12 && (w - 3.0).abs() < 1e-12);
    let far: Handle<dyn Adaptor3dCurve> = Handle::new(
        GeomAdaptorCurve::with_range(
            &line(Point3::new(9.0, 0.0, -3.0), (0.0, 0.0, 1.0)),
            0.0,
            10.0,
        )
        .unwrap(),
    )
    .cast();
    assert!(
        b0::int_cs(&s, &far, 0.0).unwrap().is_none(),
        "the cut is outside the bounds of the surface"
    );
}

#[test]
fn parameters_and_trim_curve_on_elementary_geometry() {
    assert_eq!(
        b0::parameters(&plane_xy(), Point3::new(2.0, -1.0, 5.0)).unwrap(),
        (2.0, -1.0)
    );
    let c = line(Point3::new(0.0, 0.0, 0.0), (1.0, 0.0, 0.0));
    let t = b0::trim_curve(&c, Point3::new(1.0, 1.0, 0.0), Point3::new(4.0, -2.0, 0.0)).unwrap();
    let t = t.get().unwrap();
    assert_eq!((t.first_parameter(), t.last_parameter()), (1.0, 4.0));
    let s: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_surface(&plane_xy()).unwrap()).cast();
    assert!((b0::conv_tol2d_to_tol3d(&s, 1e-9).unwrap() - 1e-9).abs() < 1e-24);
}
