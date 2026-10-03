//! ShapeFix_SplitTool range edits compared with pinned OCCT.
use rustycad_tkbrep::{
    brep::{BRepBuilder, BRepTool},
    topods::{Edge, Face},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomPlane, GeomSurface};
use rustycad_tkmath::gp::{Axis2d, Axis3, Direction3, Point3};
use rustycad_tkshhealing::shape_fix::ShapeFixSplitTool;
fn plane(z: f64) -> Face {
    let mut face = Face::default();
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(
        Axis3::new(Point3::new(0., 0., z), Direction3::Z, Direction3::X).unwrap(),
    ))
    .cast();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    face
}
fn trimmed_edge(face: &Face, other: &Face) -> Edge {
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    let curve: Handle<dyn GeomCurve> =
        Handle::new(GeomLine::from_point(Point3::default(), Direction3::X)).cast();
    builder
        .make_edge_with_curve(&mut edge, &curve, 1e-7)
        .unwrap();
    let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::default())).cast();
    let trimmed: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dTrimmedCurve::new(&line, 0., 10., true, true).unwrap()).cast();
    builder
        .update_edge_pcurve_on_face(&edge, &trimmed, face, 1e-7)
        .unwrap();
    builder
        .update_edge_pcurve_on_face(&edge, &line, other, 1e-7)
        .unwrap();
    builder.range(&edge, 0., 10., false).unwrap();
    builder.range(&edge, 0., 2., true).unwrap();
    builder.same_parameter(&edge, false).unwrap();
    edge
}
fn expected(name: &str) -> Vec<f64> {
    include_str!("../../../tools/oracle/tkshhealing_split_tool.expected")
        .lines()
        .find(|line| line.split_whitespace().next() == Some(name))
        .unwrap()
        .split_whitespace()
        .skip(1)
        .map(|v| v.parse().unwrap())
        .collect()
}
fn assert_cut(name: &str, endpoint: f64, cut: f64) {
    let face = plane(0.);
    let other = plane(1.);
    let edge = trimmed_edge(&face, &other);
    let mut line = true;
    let changed = ShapeFixSplitTool::new()
        .cut_edge(&edge, endpoint, cut, &face, &mut line)
        .unwrap();
    let (_, a, z) = BRepTool::curve(&edge).unwrap();
    let (c, d) = BRepTool::range_on_face(&edge, &face).unwrap();
    let (e, f) = BRepTool::range_on_face(&edge, &other).unwrap();
    let actual = [f64::from(changed), f64::from(line), a, z, c, d, e, f];
    for (a, e) in actual.iter().zip(expected(name)) {
        assert!(
            (a - e).abs() < 1e-12,
            "{name}: {actual:?}, expected {:?}",
            expected(name)
        );
    }
}
#[test]
fn trimmed_line_cuts_update_every_representation_before_3d_range() {
    assert_cut("trim_start", 10., 4.);
    assert_cut("trim_end", 0., 6.);
    assert_cut("middle", 2., 6.);
}
#[test]
fn tiny_cut_keeps_prior_mutations_but_early_rejection_keeps_outputs() {
    assert_cut("tiny", 0., 10. - 1e-10);
    assert_cut("early", 0., 1e-12);
}

fn assert_split(name: &str, parameter: f64, displacement: f64, reversed: bool, gap: bool) {
    use rustycad_tkg3d::top_abs::Orientation;
    use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
    let face = plane(0.);
    let mut edge = BRepBuilderAPIMakeEdge::from_points(Point3::default(), Point3::new(2., 0., 0.))
        .unwrap()
        .edge()
        .unwrap();
    if reversed {
        edge.set_orientation(Orientation::Reversed);
    }
    let mut vertex = rustycad_tkbrep::topods::Vertex::default();
    BRepBuilder::default()
        .make_vertex_at(&mut vertex, Point3::new(1., displacement, 0.), 1e-7)
        .unwrap();
    let (mut first, mut second) = (edge.clone(), edge.clone());
    let tool = ShapeFixSplitTool::new();
    let done = if gap {
        tool.split_edge_cut(
            &edge,
            0.6,
            1.4,
            &vertex,
            &face,
            &mut first,
            &mut second,
            0.01,
            1e-9,
        )
    } else {
        tool.split_edge(
            &edge,
            parameter,
            &vertex,
            &face,
            &mut first,
            &mut second,
            0.01,
            1e-9,
        )
    }
    .unwrap();
    let (_, a, b) = BRepTool::curve(&first).unwrap();
    let (_, c, d) = BRepTool::curve(&second).unwrap();
    let actual = [
        f64::from(done),
        a,
        b,
        c,
        d,
        f64::from(BRepTool::same_range(&first).unwrap()),
        f64::from(BRepTool::same_parameter(&first).unwrap()),
        BRepTool::pnt(&vertex).unwrap().y(),
        BRepTool::vertex_tolerance(&vertex).unwrap(),
        f64::from(first.orientation() == Orientation::Reversed),
    ];
    for (a, e) in actual.iter().zip(expected(name)) {
        assert!(
            (a - e).abs() < 1e-12,
            "{name}: {actual:?}, expected {:?}",
            expected(name)
        );
    }
}
#[test]
fn split_raises_tolerance_without_moving_vertex_and_repairs_flags() {
    assert_split("split", 1., 0., false, false);
    assert_split("displaced", 1., 0.2, false, false);
    assert_split("reversed", 1., 0., true, false);
    assert_split("rejected", 0., 0., false, false);
}
#[test]
fn gap_overload_cuts_forward_and_reversed_split_pieces() {
    assert_split("gap", 1., 0., false, true);
    assert_split("gap_reversed", 1., 0., true, true);
}

fn assert_sequence(name: &str, first: f64, last: f64, reversed: bool) {
    use rustycad_tkbrep::{
        brep_tools::ReShape,
        topods::{self, Vertex},
    };
    use rustycad_tkernel::ncollection::Sequence;
    use rustycad_tkg3d::top_abs::Orientation;
    use rustycad_tkshhealing::shape_build::ShapeBuildReShape;
    use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
    let face = plane(0.);
    let mut edge = BRepBuilderAPIMakeEdge::from_points(Point3::default(), Point3::new(2., 0., 0.))
        .unwrap()
        .edge()
        .unwrap();
    if reversed {
        edge.set_orientation(Orientation::Reversed);
    }
    let (mut v1, mut v2) = (Vertex::default(), Vertex::default());
    let builder = BRepBuilder::default();
    builder
        .make_vertex_at(&mut v1, Point3::new(first, 0.01, 0.), 1e-7)
        .unwrap();
    builder
        .make_vertex_at(&mut v2, Point3::new(last, 0.02, 0.), 1e-7)
        .unwrap();
    let mut pieces = Sequence::default();
    pieces.append(edge.clone().into_shape());
    let mut index = 17;
    let context = Handle::new(ShapeBuildReShape::new());
    let changed = ShapeFixSplitTool::new()
        .split_edge_sequence(
            &edge,
            first,
            &v1,
            last,
            &v2,
            &face,
            &mut pieces,
            &mut index,
            &context,
            0.001,
            1e-9,
        )
        .unwrap();
    let mut actual = vec![
        f64::from(changed),
        f64::from(index),
        pieces.length() as f64,
        BRepTool::vertex_tolerance(&v1).unwrap(),
        BRepTool::vertex_tolerance(&v2).unwrap(),
        f64::from(context.is_recorded(&edge)),
    ];
    for shape in pieces.iter() {
        let (_, a, b) = BRepTool::curve(&topods::edge(shape).unwrap()).unwrap();
        actual.extend([a, b]);
    }
    assert_eq!(actual.len(), expected(name).len(), "{name}");
    for (a, e) in actual.iter().zip(expected(name)) {
        assert!(
            (a - e).abs() < 1e-12,
            "{name}: {actual:?}, expected {:?}",
            expected(name)
        );
    }
}
#[test]
fn sequence_overload_covers_all_endpoint_combinations() {
    assert_sequence("seq_whole", 0., 2., false);
    assert_sequence("seq_start", 0., 1.4, false);
    assert_sequence("seq_end", 0.6, 2., false);
    assert_sequence("seq_middle", 0.6, 1.4, false);
}
#[test]
fn sequence_overload_preserves_direction_and_rejected_outputs() {
    assert_sequence("seq_backwards", 1.4, 0.6, false);
    assert_sequence("seq_reversed", 0.6, 1.4, true);
    assert_sequence("seq_reject", 1., 1., false);
}
