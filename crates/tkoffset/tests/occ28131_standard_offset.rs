//! Faithful port of `GeomFill_BSplineCurvesTest.OCC28131_StandardOffsetOfFilledFace`.
#[path = "../../tktopalgo/tests/support/occ28131.rs"]
mod fixture;

use rustycad_tkbrep::{
    brep::BRepTool,
    topexp::{self, Explorer},
    topods::{Edge, Face, Shape},
};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::{
    geom::{GeomSphericalSurface, GeomSurface},
    top_abs::{Orientation, ShapeEnum},
};
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::geom_abs::JoinType;
use rustycad_tkoffset::brep_offset::{BRepOffsetMakeOffset, BRepOffsetMode};
use rustycad_tkshhealing::shape_fix::ShapeFixShape;
use rustycad_tktopalgo::{
    brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape},
    brep_check::BRepCheckAnalyzer,
    brep_gprop::BRepGProp,
};

#[test]
fn geom_fill_bspline_curves_occ28131_standard_offset_of_filled_face() {
    let source = occ28131_face();
    assert!(!source.is_null());
    let mut maker = BRepOffsetMakeOffset::new();
    initialize_standard_maker(&mut maker, &source);
    maker.make_offset_shape().unwrap();
    assert!(maker.is_done());
    assert!(!maker.shape().is_null());
    assert_native_raw_result(maker.shape());
    assert_raw_singular_edge_connectivity(&maker, &source);
    assert_fixed_result(maker.shape());
}

fn initialize_standard_maker(maker: &mut BRepOffsetMakeOffset, source: &Shape) {
    maker.initialize(
        source,
        10.0,
        1.0e-7,
        BRepOffsetMode::Skin,
        false,
        false,
        JoinType::Arc,
        false,
        false,
    );
}

fn assert_native_raw_result(shape: &Shape) {
    assert_eq!(count(shape, ShapeEnum::Face), 2);
    assert_eq!(count(shape, ShapeEnum::Edge), 8);
    assert_eq!(edge_endpoint_count(shape), 16);
    assert!(!is_valid(shape));
    assert_near(
        surface_area(shape),
        1_695.591_173_430_126_2,
        1.0e-7,
        "raw area",
    );
}

fn assert_raw_singular_edge_connectivity(maker: &BRepOffsetMakeOffset, source: &Shape) {
    let source_edge = source_singular_edge(source);
    let image_edge = history_edge(maker, &source_edge);
    let offset_face = history_face(maker, source);
    let patch_face = spherical_face(maker.shape());
    let endpoints = topexp::vertices(&image_edge, false);
    assert!(!endpoints.0.is_same(&endpoints.1));
    assert_endpoint_shared_count(&offset_face, &image_edge, &endpoints.0, 1);
    assert_endpoint_shared_count(&offset_face, &image_edge, &endpoints.1, 1);
    assert_endpoint_shared_count(&patch_face, &image_edge, &endpoints.0, 1);
    assert_endpoint_shared_count(&patch_face, &image_edge, &endpoints.1, 1);
    assert_edge_orientation(&offset_face, &image_edge, Orientation::Reversed);
    assert_edge_orientation(&patch_face, &image_edge, Orientation::Forward);
}

fn source_singular_edge(source: &Shape) -> Edge {
    let mut edges = Explorer::new(source, ShapeEnum::Edge, ShapeEnum::Shape).unwrap();
    while edges.more() {
        let edge = Edge::unchecked(edges.current().clone());
        if BRepTool::degenerated(&edge).unwrap() {
            return edge;
        }
        edges.next().unwrap();
    }
    panic!("OCC28131 source face has no degenerated edge");
}

fn history_edge(maker: &BRepOffsetMakeOffset, source: &Edge) -> Edge {
    let images = maker
        .offset_edges_from_shapes()
        .last_image(&source.clone().into_shape());
    assert_eq!(images.len(), 1);
    Edge::unchecked(images[0].clone())
}

fn history_face(maker: &BRepOffsetMakeOffset, source: &Shape) -> Face {
    let images = maker.offset_faces_from_shapes().last_image(source);
    assert_eq!(images.len(), 1);
    Face::unchecked(images[0].clone())
}

fn assert_endpoint_shared_count(
    face: &Face,
    singular_edge: &Edge,
    expected: &rustycad_tkbrep::topods::Vertex,
    expected_matches: usize,
) {
    let matches = shared_endpoint_count(face, singular_edge, expected);
    assert_eq!(
        matches,
        expected_matches,
        "corrected spherical endpoint {:?} joined {matches} face edges",
        BRepTool::pnt(expected).unwrap()
    );
}

fn shared_endpoint_count(
    face: &Face,
    singular_edge: &Edge,
    expected: &rustycad_tkbrep::topods::Vertex,
) -> usize {
    let mut matches = 0;
    let shape = face.clone().into_shape();
    let edges = Explorer::new(&shape, ShapeEnum::Edge, ShapeEnum::Shape).unwrap();
    for edge in edges {
        if edge.is_same(&singular_edge.clone().into_shape()) {
            continue;
        }
        let (first, last) = topexp::vertices(&Edge::unchecked(edge), false);
        matches += usize::from(first.is_same(expected)) + usize::from(last.is_same(expected));
    }
    matches
}

fn assert_edge_orientation(face: &Face, singular_edge: &Edge, expected: Orientation) {
    for edge in Explorer::new(
        &face.clone().into_shape(),
        ShapeEnum::Edge,
        ShapeEnum::Shape,
    )
    .unwrap()
    {
        if edge.is_same(&singular_edge.clone().into_shape()) {
            assert_eq!(edge.orientation(), expected);
            return;
        }
    }
    panic!("corrected singular edge is absent from the offset face");
}

fn spherical_face(shape: &Shape) -> Face {
    for face in Explorer::new(shape, ShapeEnum::Face, ShapeEnum::Shape).unwrap() {
        let face = Face::unchecked(face);
        if BRepTool::surface(&face)
            .unwrap()
            .cast::<GeomSphericalSurface>()
            .get()
            .is_some()
        {
            return face;
        }
    }
    panic!("OCC28131 standard offset must include a spherical correction face");
}

fn edge_endpoint_count(shape: &Shape) -> usize {
    count(shape, ShapeEnum::Edge) * 2
}

fn assert_fixed_result(shape: &Shape) {
    let fixer = ShapeFixShape::new(shape.clone());
    fixer.perform().unwrap();
    let result = fixer.shape();
    assert!(!result.is_null());
    assert!(is_valid(&result));
    assert_near(max_tolerance(&result), 0.408, 0.408 * 0.01, "max tolerance");
    assert_near(
        surface_area(&result),
        1693.76,
        1693.76 * 0.01,
        "surface area",
    );
}

fn occ28131_face() -> Shape {
    let surface = fixture::occ28131_surface().unwrap();
    let surface: Handle<dyn GeomSurface> = surface.cast();
    let mut maker = BRepBuilderAPIMakeFace::from_surface(&surface, 0.0).unwrap();
    maker.shape().unwrap().clone()
}

fn count(shape: &Shape, kind: ShapeEnum) -> usize {
    let mut explorer = Explorer::new(shape, kind, ShapeEnum::Shape).unwrap();
    let mut count = 0;
    while explorer.more() {
        count += 1;
        explorer.next().unwrap();
    }
    count
}

fn max_tolerance(shape: &Shape) -> f64 {
    [ShapeEnum::Vertex, ShapeEnum::Edge, ShapeEnum::Face]
        .into_iter()
        .map(|kind| BRepTool::max_tolerance(shape, kind).unwrap())
        .fold(0.0_f64, f64::max)
}

fn surface_area(shape: &Shape) -> f64 {
    let mut properties = GProps::new();
    BRepGProp::surface_properties(shape, &mut properties, false, false).unwrap();
    properties.mass()
}

fn is_valid(shape: &Shape) -> bool {
    BRepCheckAnalyzer::new(shape, true, false, false)
        .unwrap()
        .is_valid()
        .unwrap()
}

fn assert_near(actual: f64, expected: f64, tolerance: f64, label: &str) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{label}={actual}, expected {expected} +/- {tolerance}"
    );
}
