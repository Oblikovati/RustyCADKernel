//! The ChFi3d_Builder_0 helpers without the data structure: periods, boxes, the faces
//! around edges, the corner guide lines, the cut of a curve by a plane, and the straight
//! and cubic pcurves.
use rustycad_tkbrep::topexp;
use rustycad_tkbrep::topods::{self, Edge, Shape};
use rustycad_tkbrep::toptools::IndexedDataMapOfShapeListOfShape;
use rustycad_tkernel::standard::Handle;
use rustycad_tkfillet::chfi_ds::{Map, State};
use rustycad_tkfillet::chfi3d as b0;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomPlane, GeomSphericalSurface, GeomSurface};
use rustycad_tkg3d::geom_adaptor::{GeomAdaptorCurve, GeomAdaptorSurface};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::{Axis1, Axis3, Direction2, Direction3, Point2, Point3, Vector3};
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeBox, BRepPrimAPIMakeSphere};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

const TOL: f64 = 1e-12;

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

fn edge_map(shape: &Shape) -> Map {
    let mut map = Map::new();
    map.fill(shape, ShapeEnum::Edge, ShapeEnum::Face).unwrap();
    map
}

fn edges_of(shape: &Shape) -> Vec<Edge> {
    let mut m = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(shape, ShapeEnum::Edge, ShapeEnum::Face, &mut m).unwrap();
    (1..=m.extent())
        .map(|i| topods::edge(m.find_key(i).unwrap()).unwrap())
        .collect()
}

#[test]
fn in_period_and_boxes() {
    assert_eq!(b0::in_period(-3.0, 0.0, 5.0, 1e-9), 2.0);
    // ULast itself is one period back: `Eps > ULast - u` holds for u = ULast.
    assert_eq!(b0::in_period(5.0, 0.0, 5.0, 1e-9), 0.0);
    let p = |x, y| Point2::new(x, y);
    assert_eq!(
        b0::boite(p(3.0, -1.0), p(-2.0, 5.0)),
        (-2.0, 3.0, -1.0, 5.0)
    );
    assert_eq!(
        b0::boite4([p(0.0, 0.0), p(1.0, 1.0), p(2.0, 2.0), p(3.0, 3.0)]),
        (3.0, 3.0, 0.0, 3.0, 0.0, 3.0)
    );
}

#[test]
fn box_edges_have_two_faces_and_convex_corners() {
    let shape = BRepPrimAPIMakeBox::from_sizes(1.0, 2.0, 3.0)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    let map = edge_map(&shape);
    for e in edges_of(&shape) {
        let (f1, f2) = b0::conexfaces(&e, &map).unwrap();
        assert!(!f1.is_null() && !f2.is_null() && !f1.is_same(&f2));
        assert_eq!(b0::evalconti(&e, &f1, &f2).unwrap(), Continuity::G1);
    }
    let mut verts = IndexedDataMapOfShapeListOfShape::default();
    topexp::map_shapes_and_ancestors(&shape, ShapeEnum::Vertex, ShapeEnum::Edge, &mut verts)
        .unwrap();
    let corner: Vec<Edge> = verts
        .find_from_index(1)
        .unwrap()
        .iter()
        .map(|s| topods::edge(s).unwrap())
        .collect();
    let state = b0::edge_state(
        &[corner[0].clone(), corner[1].clone(), corner[2].clone()],
        &map,
    )
    .unwrap();
    assert_eq!(
        state,
        State::AllSame,
        "the three edges of a box corner are convex the same way"
    );
}

#[test]
fn the_seam_of_a_sphere_is_cn() {
    let shape = BRepPrimAPIMakeSphere::new(2.0)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    let map = edge_map(&shape);
    let seam = edges_of(&shape).into_iter().find(|e| {
        let (f1, f2) = b0::conexfaces(e, &map).unwrap();
        !f2.is_null() && f1.is_same(&f2)
    });
    let seam = seam.expect("the sphere has a seam");
    let (f1, f2) = b0::conexfaces(&seam, &map).unwrap();
    assert_eq!(b0::evalconti(&seam, &f1, &f2).unwrap(), Continuity::CN);
}

#[test]
fn circular_spine_joins_two_perpendicular_ends() {
    let (pd, vd) = (Point3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0));
    // The end tangents turn opposite ways around the axis, as at the ends of two fillets.
    let (pf, vf) = (Point3::new(0.0, 1.0, 0.0), Vector3::new(1.0, 0.0, 0.0));
    let (circle, w1, w2) = b0::circular_spine(pd, vd, pf, vf, 1.0)
        .unwrap()
        .expect("a circle");
    assert_eq!(w1, 0.0);
    assert!((w2 - std::f64::consts::FRAC_PI_2).abs() < TOL);
    let c = circle.get().unwrap();
    assert!(c.value(0.0).unwrap().distance(&pd) < 1e-9);
    assert!(c.value(w2).unwrap().distance(&pf) < 1e-9);
}

#[test]
fn inter_plane_edge_takes_the_cut_on_the_asked_side() {
    let plane: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_surface(&plane_xy()).unwrap()).cast();
    let c = line(Point3::new(0.0, 0.0, -2.0), (0.0, 0.0, 1.0));
    let hc: Handle<dyn Adaptor3dCurve> =
        Handle::new(GeomAdaptorCurve::with_range(&c, -5.0, 5.0).unwrap()).cast();
    assert_eq!(
        b0::inter_plane_edge(&plane, &hc, true, 1e-7).unwrap(),
        Some(2.0)
    );
    let short: Handle<dyn Adaptor3dCurve> =
        Handle::new(GeomAdaptorCurve::with_range(&c, 3.0, 5.0).unwrap()).cast();
    assert_eq!(
        b0::inter_plane_edge(&plane, &short, true, 1e-7).unwrap(),
        None
    );
}

#[test]
fn compute_pcurv_is_an_iso_line_at_unit_speed() {
    let pc = b0::compute_pcurv(
        Point2::new(1.0, 2.0),
        Point2::new(1.0, 7.0),
        (2.0, 7.0),
        false,
    )
    .unwrap();
    let curve = pc.get().unwrap();
    assert_eq!(
        (curve.first_parameter(), curve.last_parameter()),
        (2.0, 7.0)
    );
    assert!(curve.value(4.5).unwrap().distance(Point2::new(1.0, 4.5)) < TOL);
    let c = line(Point3::new(1.0, 0.0, 0.0), (0.0, 1.0, 0.0));
    let (mut reached, s) = (-1.0, plane_xy());
    let mut pc2: Handle<dyn Geom2dCurve> = pc.clone();
    assert!(b0::same_parameter_geom(&c, &mut pc2, &s, (2.0, 7.0), 1e-6, &mut reached).unwrap());
    assert_eq!(reached, rustycad_tkernel::precision::confusion());
}

#[test]
fn build_pcurve_leaves_and_arrives_along_the_tangents() {
    let (mut d1, mut d2) = (
        Direction2::new(1.0, 0.0).unwrap(),
        Direction2::new(0.0, 1.0).unwrap(),
    );
    let pc = b0::build_pcurve(
        Point2::new(0.0, 0.0),
        &mut d1,
        Point2::new(2.0, 2.0),
        &mut d2,
        true,
    )
    .unwrap();
    assert_eq!(
        (d1.x(), d2.y()),
        (1.0, -1.0),
        "the end tangent is turned against the chord"
    );
    let bezier = pc.get().unwrap();
    assert_eq!(bezier.nb_poles(), 4);
    // lambda = max(0.5 |p2 - p1| |dref . d1|, 1e-5) = 1.
    assert!(bezier.pole(2).unwrap().distance(Point2::new(1.0, 0.0)) < 1e-12);
}

#[test]
fn bound_srf_keeps_a_sphere_in_its_natural_bounds() {
    let sphere: Handle<dyn GeomSurface> =
        Handle::new(GeomSphericalSurface::new(Axis3::default(), 1.0).unwrap()).cast();
    let a = GeomAdaptorSurface::with_surface(&sphere).unwrap();
    b0::bound_srf(&a, (0.0, 1.0, 1.0, 1.5), true).unwrap();
    assert!(a.last_v_parameter().unwrap() <= std::f64::consts::FRAC_PI_2);
    assert!(
        a.first_u_parameter().unwrap() < 0.0,
        "the periodic U direction is enlarged"
    );
}
