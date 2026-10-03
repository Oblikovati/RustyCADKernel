//! Native-backed cases for `ShapeAnalysis_CheckSmallFace::CheckSplittingVertices`.
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::{self, Builder, Compound, Face, Vertex, Wire};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkmath::gp::{Axis3, Point3};
use rustycad_tkshhealing::shape_analysis::{
    ShapeAnalysisCheckSmallFace, SplittingVertexEdges, SplittingVertexParameters,
};
use rustycad_tkshhealing::shape_extend::ShapeExtendStatus;
use rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge;

const ORACLE: &str =
    include_str!("../../../tools/oracle/shape_analysis_check_splitting_vertices.expected");

#[test]
fn splitting_vertex_projection_matches_native_maps_and_compound() {
    assert_case("split", 1, true, Some((1, 1)));
    assert_case("remote", 0, false, None);
    assert_case("no_vertices", 0, false, None);
}

#[test]
fn splitting_vertex_scan_rejects_uninitialized_native_precision() {
    let mut empty_analyzer = ShapeAnalysisCheckSmallFace::new();
    let empty_face = face_without_wires();
    assert_eq!(
        empty_analyzer
            .check_splitting_vertices(
                &empty_face,
                &mut SplittingVertexEdges::default(),
                &mut SplittingVertexParameters::default(),
                &mut Compound::default(),
            )
            .unwrap(),
        0
    );
    let (face, _) = fixture(Some(1.0e-4));
    let mut analyzer = ShapeAnalysisCheckSmallFace::new();
    let failure = analyzer
        .check_splitting_vertices(
            &face,
            &mut SplittingVertexEdges::default(),
            &mut SplittingVertexParameters::default(),
            &mut Compound::default(),
        )
        .unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_ConstructionError");
}

fn face_without_wires() -> Face {
    let mut face = Face::default();
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1.0e-7)
        .unwrap();
    face
}

fn assert_case(name: &str, count: i32, status: bool, split: Option<(i32, usize)>) {
    let native = oracle_row(name);
    assert_eq!(native[0] as i32, count, "{name} oracle count");
    let split_y = (name == "split").then_some(1.0e-4);
    let (face, split_vertex) = fixture(split_y);
    let (mut edges, mut parameters, mut all_vertices) = (
        SplittingVertexEdges::default(),
        SplittingVertexParameters::default(),
        Compound::default(),
    );
    Builder.make_compound(&mut all_vertices);
    let mut analyzer = ShapeAnalysisCheckSmallFace::new();
    analyzer.set_tolerance(1.0e-3);
    let found = analyzer
        .check_splitting_vertices(&face, &mut edges, &mut parameters, &mut all_vertices)
        .unwrap();
    assert_eq!(found, native[0] as i32, "{name} count");
    assert_eq!(
        all_vertices.nb_children(),
        native[4] as i32,
        "{name} compound"
    );
    assert_eq!(analyzer.status_split_vert(ShapeExtendStatus::Done), status);
    assert_eq!(status as i32, native[3] as i32, "{name} native status");
    assert_eq!(edges.len(), native[1] as usize, "{name} edge map");
    assert_eq!(parameters.len(), native[2] as usize, "{name} parameter map");
    if let Some((expected_edges, expected_params)) = split {
        let split_key = split_vertex.into_shape();
        assert_eq!(
            edges.seek(&split_key).unwrap().len(),
            expected_edges as usize
        );
        let found_parameters = parameters.seek(&split_key).unwrap();
        assert_eq!(found_parameters.len(), expected_params);
        let parameter = *found_parameters.first().unwrap();
        assert!((parameter - native[5]).abs() < 1.0e-12, "{name} parameter");
    }
}

fn oracle_row(name: &str) -> Vec<f64> {
    ORACLE
        .lines()
        .find(|line| line.starts_with(name))
        .unwrap()
        .split_whitespace()
        .skip(1)
        .map(|value| value.parse().unwrap())
        .collect()
}

fn fixture(split_y: Option<f64>) -> (Face, Vertex) {
    let builder = BRepBuilder::default();
    let mut face = Face::default();
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    builder
        .make_face_on_surface(&mut face, &surface, 1.0e-7)
        .unwrap();
    let edge =
        BRepBuilderAPIMakeEdge::from_points(Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0))
            .unwrap()
            .edge()
            .unwrap();
    let mut wire = Wire::default();
    Builder.make_wire(&mut wire);
    let mut wire_shape = wire.into_shape();
    Builder
        .add(&mut wire_shape, &edge.clone().into_shape())
        .unwrap();
    let mut face_shape = face.into_shape();
    Builder.add(&mut face_shape, &wire_shape).unwrap();
    let mut split_vertex = Vertex::default();
    if let Some(y) = split_y {
        builder
            .make_vertex_at(&mut split_vertex, Point3::new(0.5, y, 0.0), 1.0e-3)
            .unwrap();
        Builder
            .add(&mut face_shape, &split_vertex.clone().into_shape())
            .unwrap();
    }
    (topods::face(&face_shape).unwrap(), split_vertex)
}
