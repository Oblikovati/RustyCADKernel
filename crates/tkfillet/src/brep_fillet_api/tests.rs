//! Unit tests of BRepFilletAPI_MakeFillet2d on a square face.
use super::*;
use crate::chfi2d::ConstructionError;
use rustycad_tkbrep::topexp::{self, Explorer};
use rustycad_tkbrep::topods::{self, Edge, Face, Shape, Vertex};
use rustycad_tkbrep::toptools::IndexedMapOfShape;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeFace, BRepBuilderAPIMakePolygon, BRepBuilderAPIMakeShape,
};

fn square_face() -> Face {
    let mut poly = BRepBuilderAPIMakePolygon::new();
    for (x, y) in [(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)] {
        poly.add_point(Point3::new(x, y, 0.0)).expect("point");
    }
    poly.close().expect("close");
    let wire = poly.wire().expect("wire");
    BRepBuilderAPIMakeFace::from_wire(&wire, false)
        .and_then(|m| m.face())
        .expect("face")
}

fn indexed(shape: &Face, kind: ShapeEnum) -> IndexedMapOfShape {
    let mut map = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(shape, kind, &mut map).expect("map");
    map
}

fn vertex(face: &Face, i: usize) -> Vertex {
    topods::vertex(indexed(face, ShapeEnum::Vertex).find_key(i).expect("index")).expect("vertex")
}

fn edge(face: &Face, i: usize) -> Edge {
    topods::edge(indexed(face, ShapeEnum::Edge).find_key(i).expect("index")).expect("edge")
}

fn edge_count(shape: &Shape) -> usize {
    let mut ex = Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape).expect("explorer");
    let mut n = 0;
    while ex.more() {
        n += 1;
        ex.next().expect("next");
    }
    n
}

#[test]
fn build_takes_the_face_with_a_fillet_and_a_chamfer() {
    let face = square_face();
    let mut api = BRepFilletAPIMakeFillet2d::with_face(&face).expect("init");
    assert_eq!(api.status(), ConstructionError::Ready);
    let fillet = api.add_fillet(&vertex(&face, 1), 1.0).expect("fillet");
    let chamfer = api
        .add_chamfer_angle(&edge(&face, 2), &vertex(&face, 3), 1.0, 0.5)
        .expect("chamfer");
    assert_eq!(api.status(), ConstructionError::IsDone);
    api.build(&ProgressRange::default()).expect("build");
    assert!(api.is_done());
    assert_eq!(edge_count(api.shape().expect("shape")), 6);
    assert_eq!(
        (api.nb_fillet(), api.nb_chamfer(), api.nb_curves()),
        (1, 1, 2)
    );
    let first = api
        .new_edges(1)
        .expect("fillet edges")
        .first()
        .expect("one edge")
        .clone();
    assert!(first.is_same(&fillet));
    let second = api
        .new_edges(2)
        .expect("chamfer edges")
        .first()
        .expect("one edge")
        .clone();
    assert!(second.is_same(&chamfer));
    assert!(api.new_edges(3).is_err());
}

#[test]
fn modified_gives_the_descendant_of_a_cut_edge() {
    let face = square_face();
    let mut api = BRepFilletAPIMakeFillet2d::with_face(&face).expect("init");
    api.add_fillet(&vertex(&face, 1), 1.0).expect("fillet");
    let cut = (1..=4)
        .map(|i| edge(&face, i))
        .find(|e| api.is_modified(e))
        .expect("a cut edge");
    let descendant = api.descendant_edge(&cut).expect("descendant");
    let modified = api
        .modified(&cut.clone().into_shape())
        .expect("modified")
        .first()
        .expect("one")
        .clone();
    assert!(modified.is_same(&descendant));
    assert!(api.basis_edge(&descendant).expect("basis").is_same(&cut));
    assert!(api.modified(&vertex(&face, 1).into_shape()).is_err());
}

#[test]
fn build_is_not_done_after_a_refused_edit() {
    let face = square_face();
    let mut api = BRepFilletAPIMakeFillet2d::with_face(&face).expect("init");
    api.add_chamfer(&edge(&face, 1), &edge(&face, 2), -1.0, 1.0)
        .expect("refused");
    assert_eq!(api.status(), ConstructionError::ParametersError);
    api.build(&ProgressRange::default()).expect("build");
    assert!(!api.is_done());
}
