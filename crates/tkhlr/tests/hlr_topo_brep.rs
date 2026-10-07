//! HLRTopoBRep: the data structure walks and the outlines of simple solids.
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Edge, Shape, Vertex};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomCurve, GeomLine};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkhlr::hlr_algo::Projector;
use rustycad_tkhlr::hlr_topo_brep::{Data, MapOfShapeTool, OutLiner};
use rustycad_tkmath::gp::{Axis2, Direction3, Point3};
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeBox, BRepPrimAPIMakeSphere};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;

fn vertex_at(x: f64) -> Vertex {
    let mut v = Vertex::default();
    BRepBuilder::default()
        .make_vertex_at(&mut v, Point3::new(x, 0., 0.), 1e-7)
        .unwrap();
    v
}

fn line_edge() -> Edge {
    let c: Handle<dyn GeomCurve> = Handle::new(GeomLine::from_point(
        Point3::new(0., 0., 0.),
        Direction3::new(1., 0., 0.).unwrap(),
    ))
    .cast();
    let mut e = Edge::default();
    BRepBuilder::default()
        .make_edge_with_curve(&mut e, &c, 1e-7)
        .unwrap();
    e
}

#[test]
fn insert_before_keeps_the_current_vertex() {
    let mut ds = Data::new();
    let e = line_edge();
    ds.init_vertex(&e);
    ds.append(&vertex_at(2.), 2.).unwrap();
    ds.init_vertex(&e);
    ds.insert_before(&vertex_at(1.), 1.).unwrap();
    assert!(ds.more_vertex());
    assert_eq!(ds.parameter().unwrap(), 2.);
    ds.init_vertex(&e);
    let mut pars = Vec::new();
    while ds.more_vertex() {
        pars.push(ds.parameter().unwrap());
        ds.next_vertex();
    }
    assert_eq!(pars, vec![1., 2.]);
}

#[test]
fn the_edge_walk_skips_edges_without_vertices() {
    let mut ds = Data::new();
    let (e1, e2) = (line_edge(), line_edge());
    ds.init_vertex(&e1);
    ds.init_vertex(&e2);
    ds.append(&vertex_at(1.), 1.).unwrap();
    ds.init_edge();
    assert!(ds.more_edge());
    assert!(ds.edge().unwrap().is_same(&e2));
    ds.next_edge();
    assert!(!ds.more_edge());
}

fn outlined(s: &Shape) -> (Handle<OutLiner>, Shape) {
    let o = Handle::new(OutLiner::with_original(s));
    let mut mst = MapOfShapeTool::default();
    o.fill(&Projector::new().unwrap(), &mut mst, 0).unwrap();
    let out = o.out_lined_shape();
    (o, out)
}

#[test]
fn a_sphere_seen_from_the_top_has_one_internal_outline() {
    let s = BRepPrimAPIMakeSphere::with_axis(Axis2::default(), 2.)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    let (o, _) = outlined(&s);
    let f = topods::face(
        Explorer::new(&s, ShapeEnum::Face, ShapeEnum::Shape)
            .unwrap()
            .current(),
    )
    .unwrap();
    let ds = o.data_structure();
    assert!(ds.face_has_int_l(&f));
    assert!(ds.face_int_l(&f).unwrap().extent() >= 1);
}

#[test]
fn a_box_seen_along_an_axis_keeps_its_six_faces() {
    let s = BRepPrimAPIMakeBox::with_axes(Axis2::default(), 1., 2., 3.)
        .shape()
        .unwrap()
        .clone();
    let (_, out) = outlined(&s);
    let mut ex = Explorer::new(&out, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    let mut n = 0;
    while ex.more() {
        n += 1;
        ex.next().unwrap();
    }
    assert_eq!(n, 6);
}
