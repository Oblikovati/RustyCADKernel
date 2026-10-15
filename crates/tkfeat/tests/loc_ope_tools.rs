//! The LocOpe tools on boxes and cylinders, checked against their geometry: sample
//! points on the edges, closed wires, tangent faces, line and circle intersections with
//! their entering and leaving orientations, shells and solids from faces, and wires
//! chained from edges.
mod support;

use rustycad_tkbrep::brep_tools;
use rustycad_tkbrep::topods::{self, Shape};
use rustycad_tkernel::ncollection::{List, Sequence};
use rustycad_tkfeat::loc_ope::{
    self, BuildShape, BuildWires, CSIntersector, CurveShapeIntersector, WiresOnShape,
};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::{Axis1, Axis2, Circle3, Direction3, Line3, Point3};
use rustycad_tkprim::brep_prim_api::BRepPrimAPIMakeCylinder;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeShape;
use support::{assert_close, count, edge_near, face_near, make_box, sub_shapes, volume};

fn cube() -> Shape {
    make_box((0., 0., 0.), (10., 10., 10.))
}

#[test]
fn sample_edges_takes_nine_interior_points_per_edge_and_every_vertex() {
    let mut points = Sequence::default();
    loc_ope::sample_edges(&cube(), &mut points).unwrap();
    assert_eq!(points.len(), 12 * 9 + 8);
    for p in points.iter() {
        let on_face = [p.x(), p.y(), p.z()]
            .iter()
            .filter(|c| c.abs() < 1e-9 || (*c - 10.).abs() < 1e-9)
            .count();
        assert!(on_face >= 2, "{p:?} is not on an edge of the cube");
    }
}

#[test]
fn closed_wire_and_edge_on_a_box_face() {
    let shape = cube();
    let top = face_near(&shape, Point3::new(5., 5., 10.));
    let wire = brep_tools::outer_wire(&top).unwrap();
    assert!(loc_ope::closed_wire(&wire, &top).unwrap());
    let edge = edge_near(&shape, Point3::new(5., 0., 10.));
    assert!(!loc_ope::closed_edge(&edge, &top).unwrap());
}

#[test]
fn tgt_faces_is_false_at_a_box_edge_and_true_along_a_cylinder_seam() {
    let shape = cube();
    let top = face_near(&shape, Point3::new(5., 5., 10.));
    let front = face_near(&shape, Point3::new(5., 0., 5.));
    let edge = edge_near(&shape, Point3::new(5., 0., 10.));
    assert!(!loc_ope::tgt_faces(&edge, &top, &front).unwrap());
    let mut cylinder = BRepPrimAPIMakeCylinder::new(2., 5.);
    let cylinder = cylinder.shape().unwrap().clone();
    let lateral = face_near(&cylinder, Point3::new(0., 0., 2.5));
    let seam = edge_near(&cylinder, Point3::new(2., 0., 2.5));
    assert!(loc_ope::tgt_faces(&seam, &lateral, &lateral).unwrap());
}

#[test]
fn cs_intersector_sorts_the_entering_and_leaving_points_of_a_line() {
    let mut inter = CSIntersector::with_shape(&cube());
    let mut lines = Sequence::default();
    lines.append(Line3::new(
        Point3::new(-5., 5., 5.),
        Direction3::new(1., 0., 0.).unwrap(),
    ));
    inter.perform_lines(&lines).unwrap();
    assert!(inter.is_done());
    assert_eq!(inter.nb_points(1).unwrap(), 2);
    let (p1, p2) = (inter.point(1, 1).unwrap(), inter.point(1, 2).unwrap());
    assert_close(p1.parameter(), 5., 1e-9);
    assert_close(p2.parameter(), 15., 1e-9);
    assert_eq!(p1.orientation(), Orientation::Forward);
    assert_eq!(p2.orientation(), Orientation::Reversed);
    let (mut or, mut from, mut to) = (Orientation::External, 0, 0);
    assert!(
        inter
            .localize_after(1, 6., 1e-7, &mut or, &mut from, &mut to)
            .unwrap()
    );
    assert_eq!((or, from, to), (Orientation::Reversed, 2, 2));
    assert!(
        inter
            .localize_before(1, 6., 1e-7, &mut or, &mut from, &mut to)
            .unwrap()
    );
    assert_eq!((or, from, to), (Orientation::Forward, 1, 1));
    assert!(
        !inter
            .localize_after_index(1, 2, 1e-7, &mut or, &mut from, &mut to)
            .unwrap()
    );
    assert_eq!(
        inter.point(2, 1).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
}

#[test]
fn cs_intersector_refuses_an_empty_request_and_reports_not_done() {
    let mut inter = CSIntersector::new();
    assert_eq!(
        inter.nb_points(1).unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
    let lines: Sequence<Line3> = Sequence::default();
    assert_eq!(
        inter.perform_lines(&lines).unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
}

#[test]
fn cs_intersector_meets_a_circle_eight_times_through_a_box() {
    let mut inter = CSIntersector::with_shape(&cube());
    let mut circles = Sequence::default();
    let frame = Axis2::with_x_direction(
        Point3::new(5., 5., 5.),
        Direction3::new(0., 0., 1.).unwrap(),
        Direction3::new(1., 0., 0.).unwrap(),
    )
    .unwrap();
    circles.append(Circle3::new(frame, 7.).unwrap());
    inter.perform_circles(&circles).unwrap();
    assert_eq!(inter.nb_points(1).unwrap(), 8);
    let params: Vec<f64> = (1..=8)
        .map(|i| inter.point(1, i).unwrap().parameter())
        .collect();
    assert!(params.windows(2).all(|w| w[0] <= w[1]));
}

#[test]
fn curve_shape_intersector_on_an_axis_through_a_box() {
    let axis = Axis1::new(
        Point3::new(5., 5., -1.),
        Direction3::new(0., 0., 1.).unwrap(),
    );
    let inter = CurveShapeIntersector::with_axis(&axis, &cube()).unwrap();
    assert_eq!(inter.nb_points().unwrap(), 2);
    assert_close(inter.point(1).unwrap().parameter(), 1., 1e-9);
    assert_close(inter.point(2).unwrap().parameter(), 11., 1e-9);
    assert_eq!(inter.point(1).unwrap().orientation(), Orientation::Forward);
    assert_eq!(inter.point(2).unwrap().orientation(), Orientation::Reversed);
    let (mut or, mut from, mut to) = (Orientation::External, 0, 0);
    assert!(
        inter
            .localize_after_index(1, &mut or, &mut from, &mut to)
            .unwrap()
    );
    assert_eq!((or, from, to), (Orientation::Reversed, 2, 2));
    assert!(
        inter
            .localize_before_index(2, &mut or, &mut from, &mut to)
            .unwrap()
    );
    assert_eq!((or, from, to), (Orientation::Forward, 1, 1));
    let empty = CurveShapeIntersector::with_axis(&axis, &Shape::default()).unwrap();
    assert!(!empty.is_done());
}

#[test]
fn build_shape_makes_the_solid_of_the_box_faces_and_an_open_shell_of_five() {
    let faces: List<Shape> = sub_shapes(&cube(), ShapeEnum::Face).into_iter().collect();
    let build = BuildShape::with_faces(&faces).unwrap();
    assert_eq!(build.shape().shape_type().unwrap(), ShapeEnum::Solid);
    assert_close(volume(build.shape()), 1000., 1e-9);
    let five: List<Shape> = faces.iter().skip(1).cloned().collect();
    let open = BuildShape::with_faces(&five).unwrap();
    assert_eq!(open.shape().shape_type().unwrap(), ShapeEnum::Shell);
    assert_eq!(open.shape().orientation(), Orientation::Internal);
    assert!(
        BuildShape::with_faces(&List::default())
            .unwrap()
            .shape()
            .is_null()
    );
}

#[test]
fn build_wires_chains_the_edges_of_a_face_into_a_closed_wire() {
    let shape = cube();
    let top = face_near(&shape, Point3::new(5., 5., 10.));
    let edges: List<Shape> = sub_shapes(&top, ShapeEnum::Edge).into_iter().collect();
    let pw = WiresOnShape::new(&Shape::default());
    let build = BuildWires::with_edges(&edges, &pw).unwrap();
    let wires = build.result().unwrap();
    assert_eq!(wires.len(), 1);
    let wire = wires.first().unwrap();
    assert_eq!(count(wire, ShapeEnum::Edge), 4);
    assert!(wire.closed().unwrap());
    assert_eq!(
        BuildWires::new().result().err().unwrap().exception_type(),
        "StdFail_NotDone"
    );
}

#[test]
fn wires_on_shape_projects_the_ends_of_a_cut_on_the_face_edges() {
    let shape = cube();
    let top = face_near(&shape, Point3::new(5., 5., 10.));
    let cut = rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge::from_points(
        Point3::new(5., 0., 10.),
        Point3::new(5., 10., 10.),
    )
    .unwrap()
    .edge()
    .unwrap();
    let mut pw = WiresOnShape::new(&shape);
    pw.bind_edge(&cut, &top).unwrap();
    assert_eq!(
        pw.bind_edge(&cut, &top).unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
    pw.init_edge_iterator().unwrap();
    assert!(pw.is_done());
    let mut edges = 0;
    while pw.more_edge() {
        assert!(pw.edge().unwrap().is_same(&cut));
        assert!(pw.on_face().unwrap().is_same(&top));
        assert!(pw.on_edge().unwrap().is_none());
        edges += 1;
        pw.next_edge();
    }
    assert_eq!(edges, 1);
    for v in sub_shapes(&cut, ShapeEnum::Vertex) {
        let vertex = topods::vertex(&v).unwrap();
        let (mut e, mut p) = (topods::Edge::default(), 0.);
        assert!(pw.on_edge_of_vertex(&vertex, &mut e, &mut p).unwrap());
        assert!(pw.on_vertex(&vertex).unwrap().is_none());
        let middle = edge_near(
            &shape,
            rustycad_tkbrep::brep::BRepTool::pnt(&vertex).unwrap(),
        );
        assert!(e.is_same(&middle));
        assert_close(p, 5., 1e-7);
    }
}
