//! Native parity for invalid marked-degenerate edge removal.
use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::topods::{self, Builder, Edge, Face, Vertex};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::geom::{
    GeomCircle, GeomConicalSurface, GeomCurve, GeomCylindricalSurface, GeomPlane,
    GeomSphericalSurface, GeomSurface, GeomSurfaceOfRevolution,
};
use rustycad_tkmath::gp::{Axis1, Axis2, Axis2d, Axis3, Direction2, Direction3, Point2, Point3};
use rustycad_tkshhealing::shape_analysis::{ShapeAnalysisEdge, ShapeAnalysisWire};
use rustycad_tkshhealing::shape_extend::{ShapeExtendStatus, ShapeExtendWireData};
use rustycad_tkshhealing::shape_fix::ShapeFixWire;

const NATIVE_RESULT: &str =
    include_str!("../../../tools/oracle/tkshhealing_wire_degenerated.expected");

#[test]
fn invalid_marked_degenerated_edge_is_removed_with_native_status() {
    let (analyzer, data) = invalid_marked_degenerated_wire();
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);
    assert!(fixer.fix_degenerated_at(1).unwrap());
    assert_eq!(data.nb_edges(), 0);
    assert!(fixer.last_fix_status(ShapeExtendStatus::Done3));
    assert!(analyzer.last_check_status(ShapeExtendStatus::Fail2));
    assert_eq!(
        NATIVE_RESULT.lines().next(),
        Some("changed=1 count=0 done3=1 fail2=1")
    );
}

#[test]
fn closed_edge_at_cone_apex_is_replaced_as_degenerated() {
    let (analyzer, data) = cone_apex_wire();
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);
    assert!(fixer.fix_degenerated_at(1).unwrap());
    let repaired = data.edge(1).unwrap();
    assert!(BRepTool::degenerated(&repaired).unwrap());
    assert!(fixer.last_fix_status(ShapeExtendStatus::Done2));
    assert_eq!(
        BRepTool::range_on_face(&repaired, &analyzer.face()).unwrap(),
        (0.0, 2.0 * std::f64::consts::PI)
    );
    assert!(
        NATIVE_RESULT
            .lines()
            .any(|line| line == "replace=1 count=1 degenerated=1 done2=1")
    );
}

#[test]
fn bulk_degenerated_fix_tracks_single_edge_replacement_status() {
    let (analyzer, data) = cone_apex_wire();
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);
    assert!(fixer.fix_degenerated().unwrap());
    assert_eq!(data.nb_edges(), 1);
    assert!(BRepTool::degenerated(&data.edge(1).unwrap()).unwrap());
    assert!(fixer.status_degenerated(ShapeExtendStatus::Done2));
    assert!(
        NATIVE_RESULT
            .lines()
            .any(|line| line == "bulk=1 count=1 done2=1")
    );
}

#[test]
fn unready_fix_preserves_the_analyzer_check_status() {
    let (analyzer, _) = invalid_marked_degenerated_wire();
    analyzer.check_degenerated_at(1).unwrap();
    assert!(analyzer.last_check_status(ShapeExtendStatus::Fail2));
    analyzer.set_face(&Face::default()).unwrap();
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);
    assert!(!fixer.fix_degenerated_at(1).unwrap());
    assert!(analyzer.last_check_status(ShapeExtendStatus::Fail2));
    assert!(
        NATIVE_RESULT
            .lines()
            .any(|line| { line == "unready=0 prior_fail2=1 after_fail2=1" })
    );
}

#[test]
fn duplicate_degenerated_edges_at_wire_wrap_are_collapsed() {
    let (analyzer, data) = cone_apex_wire();
    let edge = data.edge(1).unwrap();
    data.add_edge(&edge, 0).unwrap();
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);
    assert!(fixer.fix_degenerated().unwrap());
    assert_eq!(data.nb_edges(), 1);
    assert!(BRepTool::degenerated(&data.edge(1).unwrap()).unwrap());
    assert!(fixer.status_degenerated(ShapeExtendStatus::Done2));
    assert!(
        NATIVE_RESULT
            .lines()
            .any(|line| { line == "duplicate=1 count=1 degenerated=1 status2=1" })
    );
}

#[test]
fn periodic_cylinder_pcurves_shift_to_join_adjacent_edges() {
    let (analyzer, data) = periodic_cylinder_wire();
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);
    assert!(fixer.fix_shifted().unwrap());
    let edge_analysis = ShapeAnalysisEdge::new();
    let first = edge_analysis
        .pcurve_on_face(&data.edge(1).unwrap(), &analyzer.face(), true)
        .unwrap();
    let second = edge_analysis
        .pcurve_on_face(&data.edge(2).unwrap(), &analyzer.face(), true)
        .unwrap();
    let first_curve = first.0.get().unwrap();
    let second_curve = second.0.get().unwrap();
    assert!((first_curve.value(first.1).unwrap().x() - 5.8).abs() < 1e-9);
    assert!((first_curve.value(first.2).unwrap().x() - 6.0).abs() < 1e-9);
    assert!(
        (second_curve.value(second.1).unwrap().x() - (std::f64::consts::TAU - 0.2)).abs() < 1e-9
    );
    assert!((second_curve.value(second.2).unwrap().x() - std::f64::consts::TAU).abs() < 1e-9);
    assert!(fixer.last_fix_status(ShapeExtendStatus::Done1));
    assert!(fixer.last_fix_status(ShapeExtendStatus::Done2));
    assert!(
        NATIVE_RESULT
            .lines()
            .any(|line| line == "shift=1 done1=1 done2=1 uv=5.8,6,6.08319,6.28319")
    );
}

#[test]
fn sphere_v_closed_override_matches_native_period_shift() {
    let (analyzer, data) = spherical_wire();
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);
    assert!(fixer.fix_shifted().unwrap());
    let edge_analysis = ShapeAnalysisEdge::new();
    let values = [1, 2].map(|index| {
        let (curve, first, last) = edge_analysis
            .pcurve_on_face(&data.edge(index).unwrap(), &analyzer.face(), true)
            .unwrap();
        let curve = curve.get().unwrap();
        (curve.value(first).unwrap(), curve.value(last).unwrap())
    });
    assert!((values[1].0.y() - (-1.5 + std::f64::consts::PI)).abs() < 1e-9);
    assert!((values[1].1.y() - (-1.4 + std::f64::consts::PI)).abs() < 1e-9);
    assert!(fixer.last_fix_status(ShapeExtendStatus::Done1));
    assert!(fixer.last_fix_status(ShapeExtendStatus::Done2));
    assert!(NATIVE_RESULT.lines().any(|line| {
        line == "sphere=1 done1=1 done2=1 e1=0.2,1.4,0.4,1.5 e2=0.4,1.64159,0.6,1.74159"
    }));
}

#[test]
fn spherical_bimeridian_range_matches_native_translation() {
    let (analyzer, data) = spherical_bimeridian_wire();
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);
    assert!(fixer.fix_shifted().unwrap());
    let edge_analysis = ShapeAnalysisEdge::new();
    let values = [1, 2, 3, 4].map(|index| {
        let (curve, first, last) = edge_analysis
            .pcurve_on_face(&data.edge(index).unwrap(), &analyzer.face(), true)
            .unwrap();
        let curve = curve.get().unwrap();
        (curve.value(first).unwrap(), curve.value(last).unwrap())
    });
    assert!((values[0].0.y() + 1.841592653589793).abs() < 1e-8);
    assert!((values[0].1.y() + 1.641592653589793).abs() < 1e-8);
    assert!((values[1].0.y() + 1.641592653589793).abs() < 1e-8);
    assert!((values[1].1.y() + 1.841592653589793).abs() < 1e-8);
    assert!((values[2].0.y() + 1.3).abs() < 1e-9);
    assert!(fixer.last_fix_status(ShapeExtendStatus::Done1));
    assert!(!fixer.last_fix_status(ShapeExtendStatus::Done2));
    assert!(NATIVE_RESULT.lines().any(|line| {
        line == "bimeridian=1 done1=1 done2=0 e1=0,-1.84159,1,-1.64159 e2=5,-1.64159,5.5,-1.84159 e3=4.5,-1.3,5,-1.5 e4=1,-1.5,1.5,-1.3"
    }));
}

#[test]
fn revolution_uses_period_of_its_periodic_basis_curve() {
    let (analyzer, data) = revolution_wire();
    let fixer = ShapeFixWire::new();
    fixer.init(&analyzer);
    assert!(fixer.fix_shifted().unwrap());
    let edge_analysis = ShapeAnalysisEdge::new();
    let (curve, first, last) = edge_analysis
        .pcurve_on_face(&data.edge(2).unwrap(), &analyzer.face(), true)
        .unwrap();
    let curve = curve.get().unwrap();
    assert!((curve.value(first).unwrap().y() - (std::f64::consts::TAU - 0.2)).abs() < 1e-9);
    assert!((curve.value(last).unwrap().y() - std::f64::consts::TAU).abs() < 1e-9);
    assert!(fixer.last_fix_status(ShapeExtendStatus::Done1));
    assert!(fixer.last_fix_status(ShapeExtendStatus::Done2));
    assert!(NATIVE_RESULT.lines().any(|line| {
        line == "revolution=1 done1=1 done2=1 e1=0.2,5.8,0.4,6 e2=0.4,6.08319,0.6,6.28319"
    }));
}

fn periodic_cylinder_wire() -> (Handle<ShapeAnalysisWire>, Handle<ShapeExtendWireData>) {
    let cylinder = Handle::new(GeomCylindricalSurface::new(Axis3::default(), 1.0).unwrap());
    let surface: Handle<dyn GeomSurface> = cylinder.cast();
    let face = face_on(&surface);
    let first = cylinder_edge(&face, &cylinder, 5.8, 6.0);
    let second = cylinder_edge(&face, &cylinder, -0.2, 0.0);
    let data = Handle::new(ShapeExtendWireData::new());
    data.add_edge(&first, 0).unwrap();
    data.add_edge(&second, 0).unwrap();
    let analyzer = Handle::new(ShapeAnalysisWire::from_wire_data(&data, &face, 1e-7).unwrap());
    (analyzer, data)
}

fn spherical_wire() -> (Handle<ShapeAnalysisWire>, Handle<ShapeExtendWireData>) {
    let sphere = Handle::new(GeomSphericalSurface::new(Axis3::default(), 1.0).unwrap());
    let surface: Handle<dyn GeomSurface> = sphere.cast();
    let face = face_on(&surface);
    let first = surface_edge(
        &face,
        &surface,
        Point2::new(0.2, 1.4),
        Point2::new(0.4, 1.5),
    );
    let second = surface_edge(
        &face,
        &surface,
        Point2::new(0.4, -1.5),
        Point2::new(0.6, -1.4),
    );
    let data = Handle::new(ShapeExtendWireData::new());
    data.add_edge(&first, 0).unwrap();
    data.add_edge(&second, 0).unwrap();
    let analyzer = Handle::new(ShapeAnalysisWire::from_wire_data(&data, &face, 1e-7).unwrap());
    (analyzer, data)
}

fn spherical_bimeridian_wire() -> (Handle<ShapeAnalysisWire>, Handle<ShapeExtendWireData>) {
    let sphere = Handle::new(GeomSphericalSurface::new(Axis3::default(), 1.0).unwrap());
    let surface: Handle<dyn GeomSurface> = sphere.cast();
    let face = face_on(&surface);
    let edges = [
        surface_edge(
            &face,
            &surface,
            Point2::new(0.0, 1.3),
            Point2::new(1.0, 1.5),
        ),
        surface_edge(
            &face,
            &surface,
            Point2::new(5.0, 1.5),
            Point2::new(5.5, 1.3),
        ),
        surface_edge(
            &face,
            &surface,
            Point2::new(4.5, -1.3),
            Point2::new(5.0, -1.5),
        ),
        surface_edge(
            &face,
            &surface,
            Point2::new(1.0, -1.5),
            Point2::new(1.5, -1.3),
        ),
    ];
    let data = Handle::new(ShapeExtendWireData::new());
    for edge in edges {
        data.add_edge(&edge, 0).unwrap();
    }
    let analyzer = Handle::new(ShapeAnalysisWire::from_wire_data(&data, &face, 1e-7).unwrap());
    (analyzer, data)
}

fn revolution_wire() -> (Handle<ShapeAnalysisWire>, Handle<ShapeExtendWireData>) {
    let basis_frame = Axis2::new(Point3::new(2.0, 0.0, 0.0), Direction3::X).unwrap();
    let basis: Handle<dyn GeomCurve> =
        Handle::new(GeomCircle::new(basis_frame, 1.0).unwrap()).cast();
    let revolution = Handle::new(GeomSurfaceOfRevolution::new(&basis, Axis1::default()).unwrap());
    let surface: Handle<dyn GeomSurface> = revolution.cast();
    let face = face_on(&surface);
    let first = surface_edge(
        &face,
        &surface,
        Point2::new(0.2, 5.8),
        Point2::new(0.4, 6.0),
    );
    let second = surface_edge(
        &face,
        &surface,
        Point2::new(0.4, -0.2),
        Point2::new(0.6, 0.0),
    );
    let data = Handle::new(ShapeExtendWireData::new());
    data.add_edge(&first, 0).unwrap();
    data.add_edge(&second, 0).unwrap();
    let analyzer = Handle::new(ShapeAnalysisWire::from_wire_data(&data, &face, 1e-7).unwrap());
    (analyzer, data)
}

fn surface_edge(
    face: &Face,
    surface: &Handle<dyn GeomSurface>,
    first: Point2,
    last: Point2,
) -> Edge {
    let builder = BRepBuilder::default();
    let vector = Point2::new(last.x() - first.x(), last.y() - first.y());
    let length = vector.x().hypot(vector.y());
    let direction = Direction2::new(vector.x(), vector.y()).unwrap();
    let pcurve: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dLine::new(Axis2d::new(first, direction))).cast();
    let mut edge = Edge::default();
    builder.make_edge(&mut edge).unwrap();
    builder
        .update_edge_pcurve_on_face(&edge, &pcurve, face, 1e-7)
        .unwrap();
    builder.range_on_face(&edge, face, 0.0, length).unwrap();
    let mut first_vertex = Vertex::default();
    let mut last_vertex = Vertex::default();
    builder
        .make_vertex_at(
            &mut first_vertex,
            surface.value(first.x(), first.y()).unwrap(),
            1e-7,
        )
        .unwrap();
    builder
        .make_vertex_at(
            &mut last_vertex,
            surface.value(last.x(), last.y()).unwrap(),
            1e-7,
        )
        .unwrap();
    let mut shape = edge.clone().into_shape();
    let topology = Builder;
    topology
        .add(
            &mut shape,
            &first_vertex.oriented(rustycad_tkg3d::top_abs::Orientation::Forward),
        )
        .unwrap();
    topology
        .add(
            &mut shape,
            &last_vertex.oriented(rustycad_tkg3d::top_abs::Orientation::Reversed),
        )
        .unwrap();
    topods::edge(&shape).unwrap()
}

fn face_on(surface: &Handle<dyn GeomSurface>) -> Face {
    let mut face = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, surface, 1e-7)
        .unwrap();
    face
}

fn cylinder_edge(
    face: &Face,
    cylinder: &GeomCylindricalSurface,
    first_u: f64,
    last_u: f64,
) -> Edge {
    let builder = BRepBuilder::default();
    let mut edge = Edge::default();
    builder.make_edge(&mut edge).unwrap();
    let pcurve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::new(
        Point2::new(first_u, 0.0),
        Direction2::new(1.0, 0.0).unwrap(),
    )))
    .cast();
    builder
        .update_edge_pcurve_on_face(&edge, &pcurve, face, 1e-7)
        .unwrap();
    builder
        .range_on_face(&edge, face, 0.0, last_u - first_u)
        .unwrap();
    let mut first = Vertex::default();
    let mut last = Vertex::default();
    builder
        .make_vertex_at(&mut first, cylinder.value(first_u, 0.0).unwrap(), 1e-7)
        .unwrap();
    builder
        .make_vertex_at(&mut last, cylinder.value(last_u, 0.0).unwrap(), 1e-7)
        .unwrap();
    let mut shape = edge.clone().into_shape();
    let topology = Builder;
    topology
        .add(
            &mut shape,
            &first.oriented(rustycad_tkg3d::top_abs::Orientation::Forward),
        )
        .unwrap();
    topology
        .add(
            &mut shape,
            &last.oriented(rustycad_tkg3d::top_abs::Orientation::Reversed),
        )
        .unwrap();
    topods::edge(&shape).unwrap()
}

fn cone_apex_wire() -> (Handle<ShapeAnalysisWire>, Handle<ShapeExtendWireData>) {
    let face = cone_face();
    let edge = cone_apex_edge(&face);
    let data = Handle::new(ShapeExtendWireData::new());
    data.add_edge(&edge, 0).unwrap();
    let analyzer = Handle::new(ShapeAnalysisWire::from_wire_data(&data, &face, 1e-7).unwrap());
    (analyzer, data)
}

fn cone_face() -> Face {
    let cone = Handle::new(GeomConicalSurface::new(Axis3::default(), 0.5, 0.0).unwrap());
    let surface: Handle<dyn GeomSurface> = cone.cast();
    let mut face = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    face
}

fn cone_apex_edge(face: &Face) -> Edge {
    let builder = BRepBuilder::default();
    let mut vertex = Vertex::default();
    builder
        .make_vertex_at(&mut vertex, Point3::default(), 1e-7)
        .unwrap();
    let mut edge = Edge::default();
    builder.make_edge(&mut edge).unwrap();
    add_edge_vertex(&mut edge, &vertex);
    let pcurve: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::new(Axis2d::new(
        Point2::default(),
        Direction2::new(1.0, 0.0).unwrap(),
    )))
    .cast();
    builder
        .update_edge_pcurve_on_face(&edge, &pcurve, face, 1e-7)
        .unwrap();
    builder
        .range_on_face(&edge, face, 0.0, 2.0 * std::f64::consts::PI)
        .unwrap();
    edge
}

fn invalid_marked_degenerated_wire() -> (Handle<ShapeAnalysisWire>, Handle<ShapeExtendWireData>) {
    let face = plane_face();
    let edge = marked_degenerated_edge();
    let data = Handle::new(ShapeExtendWireData::new());
    data.add_edge(&edge, 0).unwrap();
    let analyzer = Handle::new(ShapeAnalysisWire::from_wire_data(&data, &face, 1e-7).unwrap());
    (analyzer, data)
}

fn plane_face() -> Face {
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let mut face = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1e-7)
        .unwrap();
    face
}

fn marked_degenerated_edge() -> Edge {
    let builder = BRepBuilder::default();
    let mut vertex = Vertex::default();
    builder
        .make_vertex_at(&mut vertex, Point3::new(1.0, 0.0, 0.0), 1e-7)
        .unwrap();
    let mut edge = Edge::default();
    builder.make_edge(&mut edge).unwrap();
    builder.degenerated(&edge, true).unwrap();
    add_edge_vertex(&mut edge, &vertex);
    edge
}

fn add_edge_vertex(edge: &mut Edge, vertex: &Vertex) {
    let builder = Builder;
    let first = vertex
        .clone()
        .oriented(rustycad_tkg3d::top_abs::Orientation::Forward);
    let last = vertex
        .clone()
        .oriented(rustycad_tkg3d::top_abs::Orientation::Reversed);
    let mut shape = edge.clone().into_shape();
    builder.add(&mut shape, &first).unwrap();
    builder.add(&mut shape, &last).unwrap();
    *edge = topods::edge(&shape).unwrap();
}
