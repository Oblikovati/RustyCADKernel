//! Behaviour tests of the BRepExtrema distance group: DistShapeShape (solutions, their
//! supports and parameters, inner solutions, the multithread mode, the accessor errors,
//! Dump), DistanceSS, the Ext wrappers and Poly, on shapes built with BRepBuilderAPI.
//! The numeric parity with OCCT is checked by tools/verify_tktopalgo_numeric.py.

mod brep_extrema_support;

use brep_extrema_support::{BREPMESH_LIKE, mesh_shape};
use rustycad_tkbrep::topods::{self, Face, Shape, Vertex};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::extrema::{ExtAlgo, ExtFlag};
use rustycad_tkmath::bnd::BoundingBox3;
use rustycad_tkmath::gp::{Direction3, Plane3, Point3};
use rustycad_tktopalgo::brep_builder_api::{
    BRepBuilderAPIMakeEdge, BRepBuilderAPIMakeFace, BRepBuilderAPIMakePolygon,
    BRepBuilderAPIMakeSolid, BRepBuilderAPIMakeVertex, BRepBuilderAPISewing,
};
use rustycad_tktopalgo::brep_extrema::{
    BRepExtremaDistShapeShape, BRepExtremaDistanceSS, BRepExtremaExtCC, BRepExtremaExtCF,
    BRepExtremaExtFF, BRepExtremaExtPC, BRepExtremaExtPF, BRepExtremaPoly, BRepExtremaSolutionElem,
    BRepExtremaSupportType, uncompatible_shape,
};

const OPTIONS: (ExtFlag, ExtAlgo) = (ExtFlag::MinMax, ExtAlgo::Grad);

fn vertex(x: f64, y: f64, z: f64) -> Vertex {
    BRepBuilderAPIMakeVertex::new(Point3::new(x, y, z))
        .unwrap()
        .vertex()
        .unwrap()
}

fn segment(a: [f64; 3], b: [f64; 3]) -> Shape {
    let (pa, pb) = (Point3::new(a[0], a[1], a[2]), Point3::new(b[0], b[1], b[2]));
    BRepBuilderAPIMakeEdge::from_points(pa, pb)
        .unwrap()
        .edge()
        .unwrap()
        .into_shape()
}

fn square(z: f64, size: f64) -> Face {
    let plane = Plane3::new(
        Point3::new(0.0, 0.0, z),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    BRepBuilderAPIMakeFace::from_plane_bounds(plane, (0.0, size, 0.0, size))
        .unwrap()
        .face()
        .unwrap()
}

fn polygon_face(corners: [[f64; 3]; 4]) -> Shape {
    let mut polygon = BRepBuilderAPIMakePolygon::new();
    for c in corners {
        polygon.add_point(Point3::new(c[0], c[1], c[2])).unwrap();
    }
    polygon.close().unwrap();
    BRepBuilderAPIMakeFace::from_wire(&polygon.wire().unwrap(), true)
        .unwrap()
        .face()
        .unwrap()
        .into_shape()
}

/// The solid of the box [0, s]^3: six polygon faces with outward normals, sewn.
fn box_solid(s: f64) -> Shape {
    let faces = [
        [[0.0, 0.0, 0.0], [0.0, s, 0.0], [s, s, 0.0], [s, 0.0, 0.0]],
        [[0.0, 0.0, s], [s, 0.0, s], [s, s, s], [0.0, s, s]],
        [[0.0, 0.0, 0.0], [s, 0.0, 0.0], [s, 0.0, s], [0.0, 0.0, s]],
        [[0.0, s, 0.0], [0.0, s, s], [s, s, s], [s, s, 0.0]],
        [[0.0, 0.0, 0.0], [0.0, 0.0, s], [0.0, s, s], [0.0, s, 0.0]],
        [[s, 0.0, 0.0], [s, s, 0.0], [s, s, s], [s, 0.0, s]],
    ];
    let mut sewing = BRepBuilderAPISewing::new(1e-6, true, true, true, false);
    for corners in faces {
        sewing.add(&polygon_face(corners)).unwrap();
    }
    sewing.perform(&ProgressRange::default()).unwrap();
    let shell = topods::shell(sewing.sewed_shape()).unwrap();
    BRepBuilderAPIMakeSolid::from_shell(&shell)
        .unwrap()
        .solid()
        .unwrap()
        .into_shape()
}

fn dist(s1: &Shape, s2: &Shape) -> BRepExtremaDistShapeShape {
    BRepExtremaDistShapeShape::with_deflection(s1, s2, 1e-7, OPTIONS, &ProgressRange::default())
        .unwrap()
}

#[test]
fn solution_elem_keeps_its_support_and_parameters() {
    let v = vertex(1.0, 2.0, 3.0);
    let on_vertex = BRepExtremaSolutionElem::with_vertex(
        2.0,
        Point3::new(1.0, 2.0, 3.0),
        BRepExtremaSupportType::IsVertex,
        &v,
    );
    assert_eq!(on_vertex.support_kind(), BRepExtremaSupportType::IsVertex);
    assert!(on_vertex.vertex().is_same(&v));
    assert!(on_vertex.edge().is_null() && on_vertex.face().is_null());
    assert_eq!((on_vertex.dist(), on_vertex.edge_parameter()), (2.0, 0.0));
    let face = square(0.0, 1.0);
    let in_face = BRepExtremaSolutionElem::with_face(
        1.0,
        Point3::default(),
        BRepExtremaSupportType::IsInFace,
        &face,
        0.25,
        0.5,
    );
    assert_eq!(in_face.face_parameter(), (0.25, 0.5));
    assert_eq!(
        uncompatible_shape("x").exception_type(),
        "BRepExtrema_UnCompatibleShape"
    );
}

#[test]
fn vertex_to_edge_is_a_perpendicular_foot() {
    let d = dist(
        &vertex(0.5, 1.0, 0.0),
        &segment([0.0, 0.0, 0.0], [2.0, 0.0, 0.0]),
    );
    assert!(d.is_done());
    assert_eq!(d.nb_solution(), 1);
    assert_eq!(d.value().unwrap(), 1.0);
    assert_eq!(
        d.support_type_shape1(1).unwrap(),
        BRepExtremaSupportType::IsVertex
    );
    assert_eq!(
        d.support_type_shape2(1).unwrap(),
        BRepExtremaSupportType::IsOnEdge
    );
    assert_eq!(d.par_on_edge_s2(1).unwrap(), 0.5);
    assert_eq!(d.point_on_shape2(1).unwrap(), Point3::new(0.5, 0.0, 0.0));
    assert_eq!(
        d.par_on_edge_s1(1).unwrap_err().exception_type(),
        "BRepExtrema_UnCompatibleShape"
    );
    assert_eq!(
        d.par_on_face_s2(1).unwrap_err().exception_type(),
        "BRepExtrema_UnCompatibleShape"
    );
    assert_eq!(
        d.support_on_shape2(1).unwrap().shape_type().unwrap(),
        rustycad_tkg3d::top_abs::ShapeEnum::Edge
    );
}

#[test]
fn accessors_raise_outside_the_solutions() {
    let d = dist(&vertex(0.0, 0.0, 1.0), &square(0.0, 2.0).into_shape());
    assert_eq!(
        d.support_type_shape1(1).unwrap(),
        BRepExtremaSupportType::IsVertex
    );
    for n in [0, d.nb_solution() + 1] {
        assert_eq!(
            d.point_on_shape1(n).unwrap_err().exception_type(),
            "Standard_OutOfRange"
        );
        assert_eq!(
            d.support_on_shape2(n).unwrap_err().exception_type(),
            "Standard_OutOfRange"
        );
    }
    let empty = BRepExtremaDistShapeShape::new();
    assert_eq!(
        empty.value().unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
    assert_eq!(
        empty.par_on_edge_s1(1).unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
    let mut perform_empty = BRepExtremaDistShapeShape::new();
    assert!(!perform_empty.perform(&ProgressRange::default()).unwrap());
}

#[test]
fn a_vertex_inside_a_solid_is_an_inner_solution() {
    let solid = box_solid(2.0);
    let inside = vertex(1.0, 1.0, 0.5);
    let d = dist(&solid, &inside);
    assert!(d.inner_solution());
    assert_eq!(d.value().unwrap(), 0.0);
    assert_eq!(d.nb_solution(), 1);
    let outside = dist(&solid, &vertex(1.0, 1.0, 5.0));
    assert!(!outside.inner_solution());
    assert_eq!(outside.value().unwrap(), 3.0);
}

#[test]
fn the_multithread_mode_gives_the_single_thread_results() {
    let shapes = (box_solid(1.0), segment([-1.0, 3.0, 0.5], [3.0, 3.0, 0.7]));
    let single = dist(&shapes.0, &shapes.1);
    let mut multi = BRepExtremaDistShapeShape::new();
    multi.set_multi_thread(true);
    assert!(multi.is_multi_thread());
    multi.load_s1(&shapes.0).unwrap();
    multi.load_s2(&shapes.1).unwrap();
    assert!(multi.perform(&ProgressRange::default()).unwrap());
    assert_eq!(multi.value().unwrap(), single.value().unwrap());
    assert_eq!(multi.nb_solution(), single.nb_solution());
    for n in 1..=single.nb_solution() {
        assert_eq!(
            multi.point_on_shape1(n).unwrap(),
            single.point_on_shape1(n).unwrap()
        );
    }
}

#[test]
fn a_larger_deflection_keeps_more_solutions() {
    let (a, b) = (
        segment([0.0, 0.0, 0.0], [4.0, 0.0, 0.0]),
        segment([0.0, 1.0, 0.0], [4.0, 1.1, 0.0]),
    );
    let exact = dist(&a, &b);
    let loose =
        BRepExtremaDistShapeShape::with_deflection(&a, &b, 0.5, OPTIONS, &ProgressRange::default())
            .unwrap();
    assert!(loose.nb_solution() >= exact.nb_solution());
    assert_eq!(exact.value().unwrap(), 1.0);
}

#[test]
fn dump_writes_the_distance_and_the_solutions() -> Result<(), Failure> {
    let d = dist(&vertex(0.0, 0.0, 1.0), &vertex(0.0, 0.0, 3.0));
    let mut text: Vec<u8> = Vec::new();
    d.dump(&mut text)?;
    let text = String::from_utf8(text).unwrap();
    assert!(text.starts_with("the distance  value is :  2\nthe number of solutions is :1\n"));
    assert!(text.contains("X=0 Y=0 Z=3\n"));
    let mut partial: Vec<u8> = Vec::new();
    let error = BRepExtremaDistShapeShape::new()
        .dump(&mut partial)
        .unwrap_err();
    assert_eq!(error.exception_type(), "StdFail_NotDone");
    assert_eq!(partial, b"the distance  value is :  ");
    Ok(())
}

#[test]
fn distance_ss_prunes_by_the_boxes_and_the_start_distance() {
    let (v1, v2) = (vertex(0.0, 0.0, 0.0), vertex(3.0, 4.0, 0.0));
    let void = BoundingBox3::default();
    let near = BRepExtremaDistanceSS::new(&v1, &v2, (&void, &void), 10.0, 1e-7, OPTIONS).unwrap();
    assert!(near.is_done());
    assert_eq!((near.dist_value(), near.seq1_value().len()), (5.0, 1));
    let far = BRepExtremaDistanceSS::new(&v1, &v2, (&void, &void), 1.0, 1e-7, OPTIONS).unwrap();
    assert!(!far.is_done());
    assert_eq!(far.dist_value(), 1.0);
}

#[test]
fn ext_pc_and_ext_pf_find_the_projections() {
    let edge = topods::edge(&segment([0.0, 0.0, 0.0], [2.0, 0.0, 0.0])).unwrap();
    let pc = BRepExtremaExtPC::with_vertex_edge(&vertex(1.5, 1.0, 0.0), &edge).unwrap();
    assert!(pc.is_done());
    assert_eq!(pc.nb_ext().unwrap(), 1);
    assert_eq!(
        (pc.parameter(1).unwrap(), pc.square_distance(1).unwrap()),
        (1.5, 1.0)
    );
    assert!(pc.is_min(1).unwrap());
    assert_eq!(
        pc.point(2).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    let (d1, d2, _, _) = pc.trimmed_square_distances();
    assert_eq!((d1, d2), (3.25, 1.25));
    let face = square(0.0, 2.0);
    let pf = BRepExtremaExtPF::with_vertex_face(
        &vertex(1.0, 0.5, 3.0),
        &face,
        ExtFlag::MinMax,
        ExtAlgo::Grad,
    )
    .unwrap();
    assert!(pf.is_done());
    assert_eq!(pf.nb_ext(), 1);
    assert_eq!(pf.square_distance(1).unwrap(), 9.0);
    assert_eq!(pf.parameter(1).unwrap(), (1.0, 0.5));
    let mut not_initialized = BRepExtremaExtPF::new();
    not_initialized
        .perform(&vertex(0.0, 0.0, 0.0), &face)
        .unwrap();
    assert!(!not_initialized.is_done());
}

#[test]
fn ext_cc_ext_cf_and_ext_ff_report_parallel_and_crossing_cases() {
    let e1 = topods::edge(&segment([0.0, 0.0, 0.0], [2.0, 0.0, 0.0])).unwrap();
    let e2 = topods::edge(&segment([1.0, -1.0, 1.0], [1.0, 1.0, 1.0])).unwrap();
    let cc = BRepExtremaExtCC::with_edges(&e1, &e2).unwrap();
    assert!(!cc.is_parallel().unwrap());
    assert_eq!(cc.square_distance(1).unwrap(), 1.0);
    assert_eq!(cc.point_on_e2(1).unwrap(), Point3::new(1.0, 0.0, 1.0));
    let mut uninitialized = BRepExtremaExtCC::new();
    assert_eq!(
        uninitialized.perform(&e1).unwrap_err().exception_type(),
        "Standard_NullObject"
    );
    let face = square(3.0, 4.0);
    let cf = BRepExtremaExtCF::with_edge_face(&e1, &face).unwrap();
    assert!(cf.is_parallel().unwrap());
    assert_eq!((cf.nb_ext(), cf.square_distance(1).unwrap()), (1, 9.0));
    let ff = BRepExtremaExtFF::with_faces(&square(0.0, 1.0), &face).unwrap();
    assert!(ff.is_parallel().unwrap());
    assert_eq!(ff.square_distance(1).unwrap(), 9.0);
    assert_eq!(
        ff.point_on_face1(1).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
}

#[test]
fn poly_compares_the_triangulation_nodes() {
    let (low, high) = (square(0.0, 1.0).into_shape(), square(2.0, 1.0).into_shape());
    let (found, _) = BRepExtremaPoly::distance(&low, &high).unwrap();
    assert!(!found);
    mesh_shape(&low, 0.1, BREPMESH_LIKE).unwrap();
    mesh_shape(&high, 0.1, BREPMESH_LIKE).unwrap();
    let (found, nearest) = BRepExtremaPoly::distance(&low, &high).unwrap();
    assert!(found);
    assert_eq!(nearest.dist, 2.0);
    assert_eq!(nearest.p1, Point3::new(0.0, 0.0, 0.0));
}
