//! Splitting and gluing on boxes, checked against the closed forms of the pieces: a face
//! cut in two by an edge across it, a box glued on top of another (fuse: the volumes
//! add) or sunk into it (cut: they subtract), and the confused edges of touching boxes.
mod support;

use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topods::{self, Edge, Shape, Wire};
use rustycad_tkfeat::loc_ope::{
    FindEdges, FindEdgesInFace, Gluer, Operation, SplitShape, Spliter, WiresOnShape,
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeomalgo::geom_api::ProjectPointOnCurve;
use rustycad_tkmath::gp::Point3;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeWire};
use support::{assert_close, count, edge_near, face_near, is_valid, make_box, sub_shapes, volume};

fn segment(a: Point3, b: Point3) -> Edge {
    BRepBuilderAPIMakeEdge::from_points(a, b)
        .unwrap()
        .edge()
        .unwrap()
}

fn param_on(edge: &Edge, p: Point3) -> f64 {
    let (c, f, l) = BRepTool::curve(edge).unwrap();
    let mut proj = ProjectPointOnCurve::new();
    proj.init_range(p, &c, f, l).unwrap();
    proj.lower_distance_parameter().unwrap()
}

#[test]
fn split_shape_cuts_the_top_face_of_a_box_in_two() {
    let cube = make_box((0., 0., 0.), (10., 10., 10.));
    let top = face_near(&cube, Point3::new(5., 5., 10.));
    let (a, b) = (Point3::new(5., 0., 10.), Point3::new(5., 10., 10.));
    let cut = segment(a, b);
    let mut split = SplitShape::with_shape(&cube).unwrap();
    for (p, near) in [
        (a, Point3::new(5., 0., 10.)),
        (b, Point3::new(5., 10., 10.)),
    ] {
        let on = edge_near(&cube, near);
        let v = sub_shapes(&cut, ShapeEnum::Vertex)
            .into_iter()
            .map(|v| topods::vertex(&v).unwrap())
            .find(|v| BRepTool::pnt(v).unwrap().distance(&p) < 1e-9)
            .unwrap();
        assert!(split.can_split(&on).unwrap());
        split.add_vertex(&v, param_on(&on, p), &on).unwrap();
    }
    let wire: Wire = BRepBuilderAPIMakeWire::from_edge(&cut)
        .unwrap()
        .wire()
        .unwrap();
    assert!(split.add_wire(&wire, &top).unwrap());
    let pieces = split.descendant_shapes(&top).unwrap().clone();
    assert_eq!(pieces.len(), 2);
    let areas: Vec<f64> = pieces.iter().map(support::area).collect();
    assert_close(areas[0], 50., 1e-9);
    assert_close(areas[1], 50., 1e-9);
    let solid = split
        .descendant_shapes(&cube)
        .unwrap()
        .first()
        .unwrap()
        .clone();
    assert_eq!(count(&solid, ShapeEnum::Face), 7);
    assert_close(volume(&solid), 1000., 1e-9);
    assert!(is_valid(&solid));
    let left = split.left_of(&wire, &top).unwrap();
    assert_eq!(left.len(), 1);
}

#[test]
fn spliter_splits_a_box_by_a_wire_on_its_top_face() {
    let cube = make_box((0., 0., 0.), (10., 10., 10.));
    let top = face_near(&cube, Point3::new(5., 5., 10.));
    let cut = segment(Point3::new(5., 0., 10.), Point3::new(5., 10., 10.));
    let mut pw = WiresOnShape::new(&cube);
    pw.bind_edge(&cut, &top).unwrap();
    let mut spliter = Spliter::with_shape(&cube);
    spliter.perform(&mut pw).unwrap();
    assert!(spliter.is_done());
    let res = spliter.resulting_shape().unwrap().clone();
    assert_eq!(count(&res, ShapeEnum::Face), 7);
    assert_close(volume(&res), 1000., 1e-9);
    assert!(is_valid(&res));
    assert_eq!(spliter.descendant_shapes(&top).unwrap().len(), 2);
    assert_eq!(spliter.direct_left().unwrap().len(), 1);
    // The left part grows across every edge that is not the cut: the whole box.
    assert_eq!(spliter.left().unwrap().len(), 7);
    assert_eq!(
        Spliter::new().left().err().unwrap().exception_type(),
        "StdFail_NotDone"
    );
}

/// A small box glued on the top face of a 10 x 10 x 10 cube, on top of it (fuse) or
/// below it (cut).
fn glue(z0: f64) -> (Gluer, Shape) {
    let cube = make_box((0., 0., 0.), (10., 10., 10.));
    let small = make_box((2., 2., z0), (2., 2., 2.));
    let top = face_near(&cube, Point3::new(5., 5., 10.));
    let glued_face = face_near(&small, Point3::new(3., 3., 10.));
    let mut gluer = Gluer::with_shapes(&cube, &small);
    gluer.bind_faces(&glued_face, &top).unwrap();
    gluer.perform().unwrap();
    (gluer, top.into_shape())
}

#[test]
fn gluer_fuses_a_box_on_top_of_a_cube() {
    let (gluer, top) = glue(10.);
    assert_eq!(gluer.ope_type(), Operation::Fuse);
    assert!(gluer.is_done());
    let res = gluer.resulting_shape().unwrap();
    assert_eq!(res.shape_type().unwrap(), ShapeEnum::Solid);
    assert_close(volume(res), 1008., 1e-9);
    assert!(is_valid(res));
    // 5 faces of the cube, its top with a square hole, and 5 faces of the small box.
    assert_eq!(count(res, ShapeEnum::Face), 11);
    assert_eq!(gluer.edges().len(), 4);
    let top_desc = gluer
        .descendant_faces(&topods::face(&top).unwrap())
        .unwrap();
    assert_eq!(top_desc.len(), 1);
    assert_close(support::area(top_desc.first().unwrap()), 96., 1e-9);
}

#[test]
fn gluer_cuts_a_box_sunk_into_a_cube() {
    let (gluer, _) = glue(8.);
    assert_eq!(gluer.ope_type(), Operation::Cut);
    let res = gluer.resulting_shape().unwrap();
    assert_close(volume(res), 992., 1e-9);
    assert!(is_valid(res));
    assert_eq!(count(res, ShapeEnum::Face), 11);
}

#[test]
fn gluer_refuses_to_perform_without_binding() {
    let cube = make_box((0., 0., 0.), (10., 10., 10.));
    let mut gluer = Gluer::with_shapes(&cube, &cube);
    assert_eq!(
        gluer.perform().unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
    assert_eq!(
        gluer.resulting_shape().unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
}

#[test]
fn find_edges_pairs_the_confused_edges_of_stacked_boxes() {
    let lower = make_box((0., 0., 0.), (10., 10., 10.));
    let upper = make_box((0., 0., 10.), (10., 10., 10.));
    let top = face_near(&lower, Point3::new(5., 5., 10.));
    let bottom = face_near(&upper, Point3::new(5., 5., 10.));
    let mut find = FindEdges::with_shapes(&top, &bottom).unwrap();
    find.init_iterator();
    let mut pairs = 0;
    while find.more() {
        let (from, to) = (find.edge_from().unwrap(), find.edge_to().unwrap());
        assert!(!from.is_same(&to));
        let (c, f, l) = BRepTool::curve(&from).unwrap();
        let mid = c.get().unwrap().value((f + l) / 2.).unwrap();
        assert!(edge_near(&upper, mid).is_same(&to));
        pairs += 1;
        find.next();
    }
    assert_eq!(pairs, 4);
}

#[test]
fn find_edges_in_face_keeps_the_edges_of_a_shape_lying_in_a_face() {
    let lower = make_box((0., 0., 0.), (10., 10., 10.));
    let inner = make_box((2., 2., 10.), (3., 3., 3.));
    let top = face_near(&lower, Point3::new(5., 5., 10.));
    let mut find = FindEdgesInFace::with_shape(&inner, &top).unwrap();
    find.init();
    let mut found = 0;
    while find.more() {
        let (c, f, l) = BRepTool::curve(&find.edge().unwrap()).unwrap();
        let mid = c.get().unwrap().value((f + l) / 2.).unwrap();
        assert!(mid.z().abs() - 10. < 1e-9);
        found += 1;
        find.next();
    }
    assert_eq!(found, 4);
}
