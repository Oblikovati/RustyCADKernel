use super::*;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomCylindricalSurface, GeomLine, GeomSurface};
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tkmath::gp::{Axis1, Axis3, Direction2, Direction3, Point2, Vector3};
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;
use rustycad_tktopalgo::brep_lib::build_curve3d;

const NATIVE_GLOBAL: &str =
    include_str!("../../../../tools/oracle/tkshhealing_unify_global_context.expected");

#[test]
fn shared_boundary_is_merged_once_across_sewn_faces() {
    let input = sewn_split_faces();
    let unifier = ShapeUpgradeUnifySameDomain::new(&input, true, false, false);
    unifier.build().unwrap();
    let result = unifier.shape();
    assert_global_counts(&result, "two_face_shell");
    assert_merged_shared_boundary(&result);
}

#[test]
fn one_context_rewrites_disconnected_faces_in_a_compound() {
    let input = compound_of_split_faces();
    let unifier = ShapeUpgradeUnifySameDomain::new(&input, true, false, false);
    unifier.build().unwrap();
    assert_global_counts(&unifier.shape(), "two_face_compound");
}

#[test]
fn shared_context_reaches_solid_faces_and_preserves_validity() {
    let input = prism_of_split_face();
    let unifier = ShapeUpgradeUnifySameDomain::new(&input, true, false, false);
    unifier.build().unwrap();
    let result = unifier.shape();
    assert_global_counts(&result, "prism_solid");
    assert!(
        BRepCheckAnalyzer::new(&result, true, false, false)
            .unwrap()
            .is_valid()
            .unwrap()
    );
}

#[test]
fn nonplanar_cylinder_keeps_native_edge_selection_and_validity() {
    let input = BRepPrimAPIMakeCylinder::new(2.0, 5.0)
        .shape()
        .unwrap()
        .clone();
    let unifier = ShapeUpgradeUnifySameDomain::new(&input, true, false, false);
    unifier.build().unwrap();
    assert_global_counts(&unifier.shape(), "cylinder_solid");
}

#[test]
fn nonplanar_face_merges_split_line_edges_and_retains_pcurves() {
    let input = split_cylinder_face();
    assert_cylinder_input_valid(&input);
    let unifier = ShapeUpgradeUnifySameDomain::new(&input, true, false, false);
    unifier.build().unwrap();
    let result = unifier.shape();
    assert_global_counts(&result, "cylinder_patch_split_lines");
    assert_native_cylinder_pcurve_row();
    assert_cylinder_merge_pcurve(&result);
}

fn assert_cylinder_input_valid(shape: &Shape) {
    let valid = BRepCheckAnalyzer::new(shape, true, false, false)
        .unwrap()
        .is_valid()
        .unwrap();
    assert!(valid);
    assert!(
        NATIVE_GLOBAL
            .lines()
            .any(|line| line == "cylinder_patch_input_valid 1")
    );
}

fn assert_native_cylinder_pcurve_row() {
    assert!(
        NATIVE_GLOBAL
            .lines()
            .any(|line| line == "cylinder_patch_pcurve 0 0 5 3.1415926535897931 2.5")
    );
}

fn split_cylinder_face() -> Shape {
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomCylindricalSurface::new(Axis3::default(), 2.0).unwrap()).cast();
    let edges = cylinder_patch_edges(&surface);
    let mut maker = BRepBuilderAPIMakeWire::new();
    for edge in edges {
        maker.add_edge(&edge).unwrap();
    }
    BRepBuilderAPIMakeFace::from_surface_wire(&surface, &maker.wire().unwrap(), true)
        .unwrap()
        .face()
        .unwrap()
        .into_shape()
}

fn cylinder_patch_edges(surface: &Handle<dyn GeomSurface>) -> Vec<rustycad_tkbrep::topods::Edge> {
    [
        (
            Point2::new(0.0, 0.0),
            Direction2::new(1.0, 0.0).unwrap(),
            std::f64::consts::PI,
        ),
        (
            Point2::new(std::f64::consts::PI, 0.0),
            Direction2::new(0.0, 1.0).unwrap(),
            2.5,
        ),
        (
            Point2::new(std::f64::consts::PI, 2.5),
            Direction2::new(0.0, 1.0).unwrap(),
            2.5,
        ),
        (
            Point2::new(std::f64::consts::PI, 5.0),
            Direction2::new(-1.0, 0.0).unwrap(),
            std::f64::consts::PI,
        ),
        (
            Point2::new(0.0, 5.0),
            Direction2::new(0.0, -1.0).unwrap(),
            2.5,
        ),
        (
            Point2::new(0.0, 2.5),
            Direction2::new(0.0, -1.0).unwrap(),
            2.5,
        ),
    ]
    .into_iter()
    .map(|(point, direction, last)| cylinder_surface_edge(surface, point, direction, last))
    .collect()
}

fn cylinder_surface_edge(
    surface: &Handle<dyn GeomSurface>,
    point: Point2,
    direction: Direction2,
    last: f64,
) -> rustycad_tkbrep::topods::Edge {
    let line: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dLine::from_point(point, direction)).cast();
    let edge = BRepBuilderAPIMakeEdge::from_pcurve_params(&line, surface, 0.0, last)
        .unwrap()
        .edge()
        .unwrap();
    assert!(build_curve3d(&edge, 1.0e-7, Continuity::C1, 14, 0).unwrap());
    if direction.x() == 0.0 {
        set_cylinder_vertical_curve(&edge, surface, point, direction, last);
    }
    edge
}

fn set_cylinder_vertical_curve(
    edge: &rustycad_tkbrep::topods::Edge,
    surface: &Handle<dyn GeomSurface>,
    point: Point2,
    direction: Direction2,
    last: f64,
) {
    let first = surface.value(point.x(), point.y()).unwrap();
    let last_point = surface
        .value(
            point.x() + direction.x() * last,
            point.y() + direction.y() * last,
        )
        .unwrap();
    let tangent = Vector3::between(first, last_point);
    let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(Axis1::new(
        first,
        Direction3::from_vector(tangent).unwrap(),
    )))
    .cast();
    rustycad_tkbrep::brep::BRepBuilder::default()
        .update_edge_curve(edge, &line, 1.0e-7)
        .unwrap();
}

fn assert_cylinder_merge_pcurve(shape: &Shape) {
    let faces = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap();
    let face = rustycad_tkbrep::topods::face(&faces.current().clone()).unwrap();
    let merged = edges_of_face(&face)
        .into_iter()
        .find(vertical_edge)
        .expect("the merged cylinder boundary retains a vertical line edge");
    assert_vertical_pcurve(&merged, &face);
}

fn assert_vertical_pcurve(
    edge: &rustycad_tkbrep::topods::Edge,
    face: &rustycad_tkbrep::topods::Face,
) {
    let pcurve = BRepTool::curve_on_face(edge, face).unwrap();
    assert!(pcurve.is_stored);
    assert_eq!((pcurve.first, pcurve.last), (0.0, 5.0));
    let adaptor =
        rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve2d::with_edge_on_face(edge, face).unwrap();
    assert_eq!(
        adaptor.get_type().unwrap(),
        rustycad_tkmath::geom_abs::CurveType::Line
    );
    assert_eq!(
        adaptor.value(2.5).unwrap(),
        Point2::new(std::f64::consts::PI, 2.5)
    );
}

fn vertical_edge(edge: &rustycad_tkbrep::topods::Edge) -> bool {
    let adaptor = rustycad_tkbrep::brep_adaptor::BRepAdaptorCurve::with_edge(edge).unwrap();
    adaptor.get_type().unwrap() == rustycad_tkmath::geom_abs::CurveType::Line
}

fn sewn_split_faces() -> Shape {
    let mut sewing = BRepBuilderAPISewing::default();
    sewing.add(&split_side(0., 1., true)).unwrap();
    sewing.add(&split_side(1., 2., false)).unwrap();
    sewing.perform(&ProgressRange::default()).unwrap();
    sewing.sewed_shape().clone()
}

fn split_side(x0: f64, x1: f64, split_right: bool) -> Shape {
    let points = if split_right {
        split_right_points(x0, x1)
    } else {
        split_left_points(x0, x1)
    };
    split_planar_face(&points)
}

fn split_right_points(x0: f64, x1: f64) -> [Point3; 6] {
    [
        Point3::new(x0, 0., 0.),
        Point3::new(x1, 0., 0.),
        Point3::new(x1, 0.5, 0.),
        Point3::new(x1, 1., 0.),
        Point3::new(x0, 1., 0.),
        Point3::new(x0, 0., 0.),
    ]
}

fn split_left_points(x0: f64, x1: f64) -> [Point3; 6] {
    [
        Point3::new(x0, 0., 0.),
        Point3::new(x1, 0., 0.),
        Point3::new(x1, 1., 0.),
        Point3::new(x0, 1., 0.),
        Point3::new(x0, 0.5, 0.),
        Point3::new(x0, 0., 0.),
    ]
}

fn split_planar_face(points: &[Point3]) -> Shape {
    let wire = wire_from_points(points);
    BRepBuilderAPIMakeFace::from_plane_wire(Plane3::default(), &wire, true)
        .unwrap()
        .face()
        .unwrap()
        .into_shape()
}

fn compound_of_split_faces() -> Shape {
    let builder = BRepBuilder::default();
    let mut compound = rustycad_tkbrep::topods::Compound::default();
    builder.make_compound(&mut compound);
    builder
        .add(&mut compound, &split_side(0., 1., true))
        .unwrap();
    builder
        .add(&mut compound, &split_side(1., 2., false))
        .unwrap();
    compound.into_shape()
}

fn prism_of_split_face() -> Shape {
    rustycad_tkprim::brep_prim_api::BRepPrimAPIMakePrism::new(
        &split_side(0., 2., true),
        rustycad_tkmath::gp::Vector3::new(0., 0., 1.),
        false,
        true,
    )
    .unwrap()
    .shape()
    .unwrap()
    .clone()
}

fn assert_global_counts(shape: &Shape, case: &str) {
    let actual = format!(
        "{case} faces {} wires {} edges {} unique_edges {} vertices {} valid {}",
        Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)
            .unwrap()
            .count(),
        Explorer::new(shape, ShapeEnum::Wire, ShapeEnum::Shape)
            .unwrap()
            .count(),
        edge_shapes(shape).len(),
        unique_edges(shape),
        Explorer::new(shape, ShapeEnum::Vertex, ShapeEnum::Shape)
            .unwrap()
            .count(),
        i32::from(
            BRepCheckAnalyzer::new(shape, true, false, false)
                .unwrap()
                .is_valid()
                .unwrap()
        ),
    );
    assert!(
        NATIVE_GLOBAL.lines().any(|expected| expected == actual),
        "{actual}"
    );
}

fn unique_edges(shape: &Shape) -> usize {
    let edges = edge_shapes(shape);
    edges
        .iter()
        .enumerate()
        .filter(|(index, edge)| !edges[..*index].iter().any(|earlier| earlier.is_same(edge)))
        .count()
}

fn assert_merged_shared_boundary(shape: &Shape) {
    let faces = Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape)
        .unwrap()
        .map(|face| rustycad_tkbrep::topods::face(&face).unwrap())
        .collect::<Vec<_>>();
    let first = edges_of_face(&faces[0]);
    let second = edges_of_face(&faces[1]);
    let shared = first
        .iter()
        .find(|edge| second.iter().any(|other| edge.is_same(other)))
        .expect("the sewn faces retain one shared boundary edge");
    let (start, end) = edge_endpoints(shared);
    let actual = endpoint_coordinates(start, end);
    let expected = vec![1., 0., 0., 1., 1., 0.];
    let reversed = vec![1., 1., 0., 1., 0., 0.];
    assert!(actual == expected || actual == reversed);
    assert_eq!(unique_edges(shape), 7);
}

fn edges_of_face(face: &rustycad_tkbrep::topods::Face) -> Vec<rustycad_tkbrep::topods::Edge> {
    Explorer::new(
        &face.clone().into_shape(),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )
    .unwrap()
    .map(|edge| rustycad_tkbrep::topods::edge(&edge).unwrap())
    .collect()
}
