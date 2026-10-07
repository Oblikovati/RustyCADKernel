//! HLRBRep_Intersector: edge intersections in the view plane and sight lines on faces.
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
use rustycad_tkhlr::hlr_algo::{Projector, init_min_max};
use rustycad_tkhlr::hlr_brep::{EdgeData, Intersector, Surface};
use rustycad_tkmath::gp::{Direction3, Plane3, Point3};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace};
use std::sync::Arc;

fn edge_data(origin: Point3, direction: Direction3, p: &Arc<Projector>) -> EdgeData {
    let c: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_point(origin, direction)).cast();
    let e = BRepBuilderAPIMakeEdge::from_curve_params(&c, 0., 2.)
        .unwrap()
        .edge()
        .unwrap();
    let mut d = EdgeData::new();
    d.set(
        false, false, &e, 1, 2, false, false, false, false, 0., 1e-7, 2., 1e-7,
    )
    .unwrap();
    let g = d.change_geometry();
    g.set_projector(Some(p.clone()));
    let (mut mn, mut mx) = ([0.; 16], [0.; 16]);
    init_min_max(1e100, &mut mn, &mut mx);
    g.update(&mut mn, &mut mx).unwrap();
    d
}

fn crossing_pair() -> (EdgeData, EdgeData) {
    let p = Arc::new(Projector::new().unwrap());
    let x = Direction3::new(1., 0., 0.).unwrap();
    let y = Direction3::new(0., 1., 0.).unwrap();
    (
        edge_data(Point3::new(0., 1., 0.), x, &p),
        edge_data(Point3::new(1., 0., 0.), y, &p),
    )
}

#[test]
fn two_crossing_edges_meet_once_and_a_line_never_meets_itself() {
    let (e1, e2) = crossing_pair();
    let mut i = Intersector::new();
    i.perform_edges(1, &e1, 0., 0., 2, &e2, 0., 0., false)
        .unwrap();
    assert!(i.is_done().unwrap());
    assert_eq!(i.nb_points().unwrap(), 1);
    let pt = i.point(1).unwrap();
    assert!((pt.param_on_first() - 1.).abs() < 1e-9);
    assert!((pt.param_on_second() - 1.).abs() < 1e-9);
    i.perform_self(&e1, 0., 0.).unwrap();
    assert_eq!(i.nb_points().unwrap(), 0);
    assert_eq!(i.nb_segments().unwrap(), 0);
}

#[test]
fn a_simulated_point_keeps_its_parameters() {
    let (e1, e2) = crossing_pair();
    let mut i = Intersector::new();
    i.simulate_one_point(&e1, 0.5, &e2, 1.5).unwrap();
    assert!(i.is_done().unwrap());
    assert_eq!(i.nb_points().unwrap(), 1);
    assert_eq!(i.nb_segments().unwrap(), 0);
    let pt = i.point(1).unwrap();
    assert_eq!((pt.param_on_first(), pt.param_on_second()), (0.5, 1.5));
}

#[test]
fn a_sight_line_crosses_a_plane_face_once() {
    let p = Arc::new(Projector::new().unwrap());
    let plane = Plane3::new(
        Point3::new(0., 0., 0.),
        Direction3::new(0., 0., 1.).unwrap(),
    )
    .unwrap();
    let f = BRepBuilderAPIMakeFace::from_plane_bounds(plane, (-1., 1., -1., 1.))
        .unwrap()
        .face()
        .unwrap();
    let mut s = Surface::new();
    s.set_projector(Some(p.clone()));
    s.set_surface(&f).unwrap();
    let mut i = Intersector::new();
    i.load();
    i.perform_line(&s, &p.shoot(0.25, 0.5).unwrap(), 100.)
        .unwrap();
    assert!(i.is_done().unwrap());
    assert_eq!(i.nb_points().unwrap(), 1);
    let cs = i.cs_point(1).unwrap();
    assert!((cs.u() - 0.25).abs() < 1e-9 && (cs.v() - 0.5).abs() < 1e-9);
}
