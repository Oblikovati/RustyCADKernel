use rustycad_tkbrep::brep::{BRepBuilder, BRepTool};
use rustycad_tkbrep::brep_tools::{WireExplorer, outer_wire};
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{Edge, Face, Shape, Vertex, Wire};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dTrimmedCurve};
use rustycad_tkg3d::top_abs::{Orientation, ShapeEnum};
use rustycad_tkmath::gp::Point2;
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeCylinder, BRepPrimAPIMakeOneAxis};
use rustycad_tkshhealing::shape_upgrade::ShapeUpgradeUnifySameDomain;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;

const NATIVE_SEAM: &str =
    include_str!("../../../../tools/oracle/tkshhealing_unify_seam_pcurve.expected");

#[test]
fn adjacent_segments_of_a_native_cylinder_seam_merge_back_to_one_seam() {
    let input = split_cylinder_seam();
    assert_eq!(unique_edge_count(&input), native_value("input_edges"));
    assert!(is_valid(&input));
    assert!(NATIVE_SEAM.contains("input_valid 1"));
    let unifier = ShapeUpgradeUnifySameDomain::new(&input, true, false, true);
    unifier.build().unwrap();
    let result = unifier.shape();
    assert_eq!(unique_edge_count(&result), native_value("result_edges"));
    assert!(is_valid(&result));
    assert_native_seam_pair(&result);
}

fn split_cylinder_seam() -> Shape {
    let mut maker = BRepPrimAPIMakeCylinder::new(2.0, 5.0);
    let face = maker.face().unwrap();
    let seam = seam_edge(&face).unwrap();
    let segments = split_edge_pair(&seam, &face).unwrap();
    rebuild_lateral_face(&face, &seam, &segments)
        .unwrap()
        .into_shape()
}

fn seam_edge(face: &Face) -> Option<Edge> {
    Explorer::new(
        &face.clone().into_shape(),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )
    .ok()?
    .map(|candidate| rustycad_tkbrep::topods::edge(&candidate))
    .collect::<Result<Vec<_>, _>>()
    .ok()?
    .into_iter()
    .find(|edge| BRepTool::is_closed_on_face(edge, face).unwrap_or(false))
}

fn split_edge_pair(edge: &Edge, face: &Face) -> Result<(Edge, Edge), Failure> {
    let mut canonical = edge.clone();
    canonical.set_orientation(Orientation::Forward);
    let (curve, first, last) = BRepTool::curve(&canonical)?;
    let middle_parameter = (first + last) * 0.5;
    let mut middle_vertex = Vertex::default();
    BRepBuilder::default().make_vertex_at(
        &mut middle_vertex,
        curve
            .get()
            .ok_or_else(null_curve)?
            .value(middle_parameter)?,
        BRepTool::edge_tolerance(edge)?,
    )?;
    split_edge_segments(
        &canonical,
        face,
        &curve,
        &middle_vertex,
        (first, middle_parameter, last),
    )
}

fn split_edge_segments(
    edge: &Edge,
    face: &Face,
    curve: &Handle<dyn rustycad_tkg3d::geom::GeomCurve>,
    middle: &Vertex,
    range: (f64, f64, f64),
) -> Result<(Edge, Edge), Failure> {
    let (start, finish) = rustycad_tkbrep::topexp::vertices(edge, false);
    let lower = make_split_segment(curve, &start, middle, (range.0, range.1))?;
    let upper = make_split_segment(curve, middle, &finish, (range.1, range.2))?;
    set_split_pcurves(&lower, edge, face, (range.0, range.1))?;
    set_split_pcurves(&upper, edge, face, (range.1, range.2))?;
    Ok((lower, upper))
}

fn make_split_segment(
    curve: &Handle<dyn rustycad_tkg3d::geom::GeomCurve>,
    first: &Vertex,
    last: &Vertex,
    range: (f64, f64),
) -> Result<Edge, Failure> {
    BRepBuilderAPIMakeEdge::from_curve_vertices_params(curve, first, last, range.0, range.1)?.edge()
}

fn set_split_pcurves(
    segment: &Edge,
    source: &Edge,
    face: &Face,
    range: (f64, f64),
) -> Result<(), Failure> {
    let mut reversed = source.clone();
    reversed.reverse();
    let first = trimmed_pcurve(&BRepTool::curve_on_face(source, face)?.pcurve, range)?;
    let second = trimmed_pcurve(&BRepTool::curve_on_face(&reversed, face)?.pcurve, range)?;
    BRepBuilder::default().update_edge_pcurves_on_face(
        segment,
        (&first, &second),
        face,
        BRepTool::edge_tolerance(source)?,
    )
}

fn trimmed_pcurve(
    curve: &Handle<dyn Geom2dCurve>,
    range: (f64, f64),
) -> Result<Handle<dyn Geom2dCurve>, Failure> {
    Ok(Handle::new(Geom2dTrimmedCurve::new(
        curve, range.0, range.1, true, false,
    )?)
    .cast())
}

fn rebuild_lateral_face(
    source: &Face,
    seam: &Edge,
    segments: &(Edge, Edge),
) -> Result<Face, Failure> {
    let wire = rebuild_wire(source, seam, segments)?;
    let (surface, location) = BRepTool::surface_with_location(source)?;
    let mut result = Face::default();
    let builder = BRepBuilder::default();
    builder.make_face_on_located_surface(
        &mut result,
        &surface,
        &location,
        BRepTool::face_tolerance(source)?,
    )?;
    builder.add(&mut result, &wire.into_shape())?;
    Ok(result)
}

fn rebuild_wire(source: &Face, seam: &Edge, segments: &(Edge, Edge)) -> Result<Wire, Failure> {
    let mut wire = Wire::default();
    BRepBuilder::default().make_wire(&mut wire);
    let mut explorer = WireExplorer::with_wire_on_face(&outer_wire(source)?, source)?;
    while explorer.more() {
        add_wire_occurrence(&mut wire, explorer.current(), seam, segments)?;
        explorer.next()?;
    }
    Ok(wire)
}

fn add_wire_occurrence(
    wire: &mut Wire,
    current: &Edge,
    seam: &Edge,
    segments: &(Edge, Edge),
) -> Result<(), Failure> {
    let builder = BRepBuilder::default();
    if !current.is_same(seam) {
        return builder.add(wire, &current.clone().into_shape());
    }
    let (first, second) = ordered_segments(current, segments);
    builder.add(wire, &first.into_shape())?;
    builder.add(wire, &second.into_shape())
}

fn ordered_segments(current: &Edge, segments: &(Edge, Edge)) -> (Edge, Edge) {
    if current.orientation() == Orientation::Reversed {
        let mut first = segments.1.clone();
        let mut second = segments.0.clone();
        first.reverse();
        second.reverse();
        return (first, second);
    }
    (segments.0.clone(), segments.1.clone())
}

fn assert_native_seam_pair(shape: &Shape) {
    let face = rustycad_tkbrep::topods::face(
        &Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)
            .unwrap()
            .current()
            .clone(),
    )
    .unwrap();
    let edge = seam_edge(&face).expect("unified result retains its cylinder seam");
    let forward = BRepTool::curve_on_face(&edge, &face).unwrap();
    let mut reversed = edge.clone();
    reversed.reverse();
    let backward = BRepTool::curve_on_face(&reversed, &face).unwrap();
    assert_eq!((forward.first, forward.last), (0.0, 5.0));
    assert_seam_pcurve_bounds(&forward);
    assert_seam_pcurve_bounds(&backward);
    assert!((pcurve_value(&forward, 0.0).x() - pcurve_value(&backward, 0.0).x()).abs() > 6.0);
    assert!(NATIVE_SEAM.contains("seam 0 5 0 5 0 0 6.2831853071795862 5 6.2831853071795862 0"));
}

fn assert_seam_pcurve_bounds(curve: &rustycad_tkbrep::brep::EdgePCurve) {
    let first = pcurve_value(curve, curve.first);
    let last = pcurve_value(curve, curve.last);
    assert_eq!((first.y(), last.y()), (5.0, 0.0));
    assert!(is_native_seam_u(first.x()));
    assert_eq!(first.x(), last.x());
}

fn pcurve_value(curve: &rustycad_tkbrep::brep::EdgePCurve, parameter: f64) -> Point2 {
    curve.pcurve.get().unwrap().value(parameter).unwrap()
}

fn is_native_seam_u(value: f64) -> bool {
    value.abs() < 1e-12 || (value - std::f64::consts::TAU).abs() < 1e-12
}

fn unique_edge_count(shape: &Shape) -> usize {
    let mut edges: Vec<Edge> = Vec::new();
    for edge in Explorer::new(shape, ShapeEnum::Edge, ShapeEnum::Shape).unwrap() {
        let edge = rustycad_tkbrep::topods::edge(&edge).unwrap();
        if !edges.iter().any(|known| known.is_same(&edge)) {
            edges.push(edge);
        }
    }
    edges.len()
}

fn native_value(key: &str) -> usize {
    NATIVE_SEAM
        .split_whitespace()
        .collect::<Vec<_>>()
        .windows(2)
        .find(|pair| pair[0] == key)
        .unwrap()[1]
        .parse()
        .unwrap()
}

fn is_valid(shape: &Shape) -> bool {
    BRepCheckAnalyzer::new(shape, true, false, false)
        .unwrap()
        .is_valid()
        .unwrap()
}

fn null_curve() -> Failure {
    Failure::with_type("Standard_NullObject", "cylinder seam lacks a 3D curve", "")
}
