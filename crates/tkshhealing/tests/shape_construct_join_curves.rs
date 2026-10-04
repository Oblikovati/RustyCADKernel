//! Native differential tests for ShapeConstruct curve joining.
use rustycad_tkbrep::{
    brep_tools::ReShape,
    topexp::Explorer,
    topods::{self, Builder, Compound, Shape},
    toptools::MapOfShape,
};
use rustycad_tkernel::{ncollection::List, standard::Handle};
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::{
    geom::{GeomCurve, GeomLine},
    top_abs::Orientation,
};
use rustycad_tkmath::gp::{Axis1, Axis2d, Direction2, Direction3, Point2, Point3};
use rustycad_tkshhealing::{
    shape_build::ShapeBuildReShape,
    shape_construct::{ShapeConstructCurve, ShapeConstructCurveJoin2dRequest},
    shape_extend::ShapeExtendStatus,
    shape_fix::{FixRoot, ShapeFixWireframe, ShapeFixWireframeShapeLists},
};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPIMakePolygon};

#[test]
fn join_curves_3d_matches_native_output_and_range_updates() {
    assert_eq!(joined_3d_observation(), expected("join3d"));
}

#[test]
fn join_curves_2d_matches_native_output_and_range_updates() {
    assert_eq!(joined_2d_observation(), expected("join2d"));
}

#[test]
fn reversed_2d_input_matches_native_parameter_and_orientation_changes() {
    assert_eq!(
        joined_reversed_2d_observation(),
        expected("join2d_reversed")
    );
}

fn joined_3d_observation() -> Vec<f64> {
    let line = line_3d();
    let mut first_range = (0., 1.);
    let mut second_range = (1., 2.);
    let joined = ShapeConstructCurve
        .join_curves_3d(
            &line,
            &line,
            Orientation::Forward,
            Orientation::Forward,
            &mut first_range,
            &mut second_range,
        )
        .unwrap()
        .unwrap();
    let (start, end) = curve_endpoints_3d(&joined.curve);
    observed_3d(&joined, first_range, second_range, start, end)
}

fn joined_2d_observation() -> Vec<f64> {
    let line = line_2d();
    let mut request = forward_2d_request(line.clone(), line, (0., 1.), (1., 2.));
    request.is_error = true;
    observe_2d_request(&mut request)
}

fn joined_reversed_2d_observation() -> Vec<f64> {
    let mut request = reversed_2d_request();
    observe_2d_request(&mut request)
}

fn observe_2d_request(request: &mut ShapeConstructCurveJoin2dRequest) -> Vec<f64> {
    let joined = ShapeConstructCurve
        .join_curves_2d(request)
        .unwrap()
        .unwrap();
    let start = joined.curve.value(joined.curve.first_parameter()).unwrap();
    let end = joined.curve.value(joined.curve.last_parameter()).unwrap();
    observed_2d(
        &joined,
        request.first_range,
        request.second_range,
        start,
        end,
    )
}

fn curve_endpoints_3d(curve: &rustycad_tkg3d::geom::GeomBSplineCurve) -> (Point3, Point3) {
    (
        curve.value(curve.first_parameter()).unwrap(),
        curve.value(curve.last_parameter()).unwrap(),
    )
}

fn forward_2d_request(
    first: Handle<dyn Geom2dCurve>,
    second: Handle<dyn Geom2dCurve>,
    first_range: (f64, f64),
    second_range: (f64, f64),
) -> ShapeConstructCurveJoin2dRequest {
    ShapeConstructCurveJoin2dRequest {
        first,
        second,
        first_orientation: Orientation::Forward,
        second_orientation: Orientation::Forward,
        first_range,
        second_range,
        is_error: false,
    }
}

fn reversed_2d_request() -> ShapeConstructCurveJoin2dRequest {
    let first = line_2d();
    let mut request = forward_2d_request(first.clone(), first, (0., 1.), (0., 1.));
    request.first_orientation = Orientation::Reversed;
    request
}

#[test]
fn shared_face_pair_order_preserves_native_fail1_rejection() {
    assert_eq!(
        SharedFaceCase::new(false).run(),
        shared_face_expected("shared_face_forward")
    );
}

#[test]
fn opposite_shared_face_order_allows_native_merge() {
    assert_eq!(
        SharedFaceCase::new(true).run(),
        shared_face_expected("shared_face_reversed")
    );
}

fn tiny_wire() -> rustycad_tkbrep::topods::Wire {
    BRepBuilderAPIMakePolygon::from_four_points(
        Point3::new(0., 0., 0.),
        Point3::new(0.01, 0., 0.),
        Point3::new(1., 0., 0.),
        Point3::new(1., 1., 0.),
        true,
    )
    .unwrap()
    .wire()
    .unwrap()
}

fn plane_face(wire: &rustycad_tkbrep::topods::Wire) -> rustycad_tkbrep::topods::Face {
    BRepBuilderAPIMakeFace::from_plane_wire(rustycad_tkmath::gp::Plane3::default(), wire, true)
        .unwrap()
        .face()
        .unwrap()
}

fn two_face_compound(
    first: &rustycad_tkbrep::topods::Face,
    second: &rustycad_tkbrep::topods::Face,
) -> Shape {
    let mut compound = Compound::default();
    Builder.make_compound(&mut compound);
    Builder.add(&mut compound, first).unwrap();
    Builder.add(&mut compound, second).unwrap();
    compound.into_shape()
}

struct SharedFaceCase {
    wire: rustycad_tkbrep::topods::Wire,
    first: rustycad_tkbrep::topods::Face,
    second: rustycad_tkbrep::topods::Face,
    fixer: ShapeFixWireframe,
}

impl SharedFaceCase {
    fn new(reverse_second: bool) -> Self {
        let wire = tiny_wire();
        let first = plane_face(&wire);
        let mut second = plane_face(&wire);
        if reverse_second {
            second.set_orientation(Orientation::Reversed);
        }
        let root = two_face_compound(&first, &second);
        let fixer = ShapeFixWireframe::from_shape(&root);
        fixer.set_precision(0.1);
        fixer.set_context(&Handle::new(ShapeBuildReShape::new()));
        Self {
            wire,
            first,
            second,
            fixer,
        }
    }

    fn run(&self) -> Vec<f64> {
        let edges = edges(&self.wire.clone().into_shape());
        let mut small = small_edge_map(&edges[0]);
        let mut edge_faces = linked_edges(&edges, &self.first, &self.second);
        let mut face_small = small_edge_face_map(&self.first, &edges[0]);
        let mut multiple = MapOfShape::default();
        let changed = self
            .fixer
            .merge_small_edges(
                &mut small,
                &mut edge_faces,
                &mut face_small,
                &mut multiple,
                false,
                -1.,
            )
            .unwrap();
        shared_face_result(&self.fixer, &self.first, &self.second, changed)
    }
}

fn small_edge_map(edge: &Shape) -> MapOfShape {
    let mut small = MapOfShape::default();
    small.add(edge.clone());
    small
}

fn small_edge_face_map(
    face: &rustycad_tkbrep::topods::Face,
    edge: &Shape,
) -> ShapeFixWireframeShapeLists {
    let mut edges = List::default();
    edges.append(edge.clone());
    let mut links = ShapeFixWireframeShapeLists::default();
    links.bind(face.clone().into_shape(), edges);
    links
}

fn edges(shape: &Shape) -> Vec<Shape> {
    Explorer::new(
        shape,
        rustycad_tkg3d::top_abs::ShapeEnum::Edge,
        rustycad_tkg3d::top_abs::ShapeEnum::Shape,
    )
    .unwrap()
    .collect()
}

fn linked_edges(
    edges: &[Shape],
    first: &rustycad_tkbrep::topods::Face,
    second: &rustycad_tkbrep::topods::Face,
) -> ShapeFixWireframeShapeLists {
    let mut links = ShapeFixWireframeShapeLists::default();
    for edge in edges {
        links.bind(edge.clone(), face_pair(first, second));
    }
    links
}

fn face_pair(
    first: &rustycad_tkbrep::topods::Face,
    second: &rustycad_tkbrep::topods::Face,
) -> List<Shape> {
    let mut faces = List::default();
    faces.append(first.clone().into_shape());
    faces.append(second.clone().into_shape());
    faces
}

fn shared_face_result(
    fixer: &ShapeFixWireframe,
    first: &rustycad_tkbrep::topods::Face,
    second: &rustycad_tkbrep::topods::Face,
    changed: bool,
) -> Vec<f64> {
    let first_result = applied_face(fixer, first);
    let second_result = applied_face(fixer, second);
    vec![
        f64::from(changed),
        edges(&first_result).len() as f64,
        face_edge_curves(&topods::face(&second_result).unwrap())
            .1
            .len() as f64,
        f64::from(fixer.status_small_edges(ShapeExtendStatus::Done)),
        f64::from(fixer.status_small_edges(ShapeExtendStatus::Fail)),
    ]
}

fn applied_face(fixer: &ShapeFixWireframe, face: &rustycad_tkbrep::topods::Face) -> Shape {
    ReShape::apply(
        &*fixer.context(),
        &face.clone().into_shape(),
        rustycad_tkg3d::top_abs::ShapeEnum::Shape,
    )
    .unwrap()
}

fn face_edge_curves(
    face: &rustycad_tkbrep::topods::Face,
) -> (usize, Vec<rustycad_tkbrep::topods::Edge>) {
    let face_shape = face.clone().into_shape();
    let edges = Explorer::new(
        &face_shape,
        rustycad_tkg3d::top_abs::ShapeEnum::Edge,
        rustycad_tkg3d::top_abs::ShapeEnum::Shape,
    )
    .unwrap()
    .collect::<Vec<_>>();
    let curves = edges
        .into_iter()
        .filter_map(|edge| {
            let edge = topods::edge(&edge).ok()?;
            let pcurve = rustycad_tkbrep::brep::BRepTool::curve_on_face(&edge, face).ok()?;
            (!pcurve.pcurve.is_null()).then_some(edge)
        })
        .collect::<Vec<_>>();
    (curves.len(), curves)
}

fn observed_3d(
    joined: &rustycad_tkshhealing::shape_construct::ShapeConstructCurveJoin3d,
    first_range: (f64, f64),
    second_range: (f64, f64),
    start: Point3,
    end: Point3,
) -> Vec<f64> {
    vec![
        1.,
        f64::from(joined.reversed_first),
        f64::from(joined.reversed_second),
        first_range.0,
        first_range.1,
        second_range.0,
        second_range.1,
        f64::from(joined.curve.degree()),
        f64::from(joined.curve.nb_poles()),
        start.x(),
        start.y(),
        end.x(),
        end.y(),
    ]
}

fn observed_2d(
    joined: &rustycad_tkshhealing::shape_construct::ShapeConstructCurveJoin2d,
    first_range: (f64, f64),
    second_range: (f64, f64),
    start: Point2,
    end: Point2,
) -> Vec<f64> {
    vec![
        1.,
        f64::from(joined.reversed_first),
        f64::from(joined.reversed_second),
        first_range.0,
        first_range.1,
        second_range.0,
        second_range.1,
        f64::from(joined.curve.degree()),
        f64::from(joined.curve.nb_poles()),
        start.x(),
        start.y(),
        end.x(),
        end.y(),
    ]
}

fn line_3d() -> Handle<dyn GeomCurve> {
    let axis = Axis1::new(Point3::default(), Direction3::new(1., 0., 0.).unwrap());
    Handle::new(GeomLine::new(axis)).cast()
}

fn line_2d() -> Handle<dyn Geom2dCurve> {
    let axis = Axis2d::new(Point2::default(), Direction2::new(1., 0.).unwrap());
    Handle::new(Geom2dLine::new(axis)).cast()
}

fn expected(name: &str) -> Vec<f64> {
    include_str!("../../../tools/oracle/tkshhealing_shapeconstruct_join_curves.expected")
        .lines()
        .find(|line| line.starts_with(&format!("{name} ")))
        .unwrap()
        .split_whitespace()
        .skip(1)
        .map(|value| value.parse().unwrap())
        .collect()
}

fn shared_face_expected(name: &str) -> Vec<f64> {
    include_str!("../../../tools/oracle/tkshhealing_wireframe_shared_face_join.expected")
        .lines()
        .find(|line| line.starts_with(&format!("{name} ")))
        .unwrap()
        .split_whitespace()
        .skip(1)
        .map(|value| value.parse().unwrap())
        .collect()
}
