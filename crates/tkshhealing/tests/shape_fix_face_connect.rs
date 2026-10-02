//! Translation of the pinned ShapeFix_FaceConnect_Test.cxx regression suite.
use rustycad_tkbrep::{
    topexp,
    topods::{self, Builder, Face, Shell},
    toptools::IndexedMapOfShape,
};
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::Point3;
use rustycad_tkshhealing::shape_fix::ShapeFixFaceConnect;
use rustycad_tktopalgo::{
    brep_check::BRepCheckAnalyzer,
    brep_gprop::BRepGProp,
    brep_lib::{BRepLibMakeFace, BRepLibMakePolygon},
};

fn rectangle(x_min: f64, x_max: f64) -> Face {
    let polygon = BRepLibMakePolygon::from_four_points(
        Point3::new(x_min, 0.0, 0.0),
        Point3::new(x_max, 0.0, 0.0),
        Point3::new(x_max, 1.0, 0.0),
        Point3::new(x_min, 1.0, 0.0),
        true,
    )
    .unwrap();
    BRepLibMakeFace::from_wire(&polygon.wire().unwrap(), true)
        .unwrap()
        .face()
        .unwrap()
}

fn shell_of(faces: &[Face]) -> Shell {
    let builder = Builder;
    let mut shell = Shell::default();
    builder.make_shell(&mut shell);
    let mut shape = shell.into_shape();
    for face in faces {
        builder.add(&mut shape, &face.clone().into_shape()).unwrap();
    }
    topods::shell(&shape).unwrap()
}

fn connector(first: &Face, second: &Face) -> ShapeFixFaceConnect {
    let mut connector = ShapeFixFaceConnect::new();
    assert!(connector.add(first, second));
    connector
}

fn edge_count(shape: &topods::Shape) -> usize {
    let mut edges = IndexedMapOfShape::default();
    topexp::map_shapes_of_type(shape, rustycad_tkg3d::top_abs::ShapeEnum::Edge, &mut edges)
        .unwrap();
    edges.extent()
}

fn area(shape: &topods::Shape) -> f64 {
    let mut properties = GProps::default();
    BRepGProp::surface_properties(shape, &mut properties, false, false).unwrap();
    properties.mass()
}

fn is_valid(shape: &topods::Shape) -> bool {
    BRepCheckAnalyzer::new(shape, true, false, false)
        .unwrap()
        .is_valid()
        .unwrap()
}

#[test]
fn rejects_null_faces() {
    let mut connector = ShapeFixFaceConnect::new();
    let face = rectangle(0.0, 1.0);
    let null = Face::default();
    assert!(!connector.add(&null, &face));
    assert!(!connector.add(&face, &null));
    assert!(!connector.add(&null, &null));
}

#[test]
fn accepts_self_and_duplicate_connections() {
    let mut connector = ShapeFixFaceConnect::new();
    let face1 = rectangle(0.0, 1.0);
    let face2 = rectangle(1.0, 2.0);
    assert!(connector.add(&face1, &face1));
    assert!(connector.add(&face1, &face2));
    assert!(connector.add(&face1, &face2));
}

#[test]
fn no_connections_return_original_shell() {
    let shell = shell_of(&[rectangle(0.0, 1.0), rectangle(1.0, 2.0)]);
    let result = ShapeFixFaceConnect::new()
        .build(&shell, 1.0e-6, 1.0e-7)
        .unwrap();
    assert!(result.is_same(&shell));
    assert_eq!(edge_count(&result.clone().into_shape()), 8);
}

#[test]
fn connects_coincident_boundaries() {
    let (face1, face2) = (rectangle(0.0, 1.0), rectangle(1.0, 2.0));
    let shell = shell_of(&[face1.clone(), face2.clone()]);
    let result = connector(&face1, &face2)
        .build(&shell, 1.0e-6, 1.0e-7)
        .unwrap();
    assert_eq!(edge_count(&result.clone().into_shape()), 7);
    assert_eq!(area(&shell.clone().into_shape()), 2.0);
    assert!((area(&result.clone().into_shape()) - 2.0).abs() <= 1.0e-12);
    assert!(is_valid(&result.clone().into_shape()));
}

#[test]
fn connects_boundaries_within_sewing_tolerance() {
    let (face1, face2) = (rectangle(0.0, 1.0), rectangle(1.0001, 2.0));
    let shell = shell_of(&[face1.clone(), face2.clone()]);
    let result = connector(&face1, &face2)
        .build(&shell, 1.0e-3, 1.0e-7)
        .unwrap();
    assert_eq!(edge_count(&result.clone().into_shape()), 7);
    assert!((area(&shell.clone().into_shape()) - 1.9999).abs() <= 1.0e-12);
    assert!((area(&result.clone().into_shape()) - 1.9999).abs() <= 1.0e-12);
    assert!(is_valid(&result.clone().into_shape()));
}

#[test]
fn leaves_boundaries_beyond_sewing_tolerance_separate() {
    let (face1, face2) = (rectangle(0.0, 1.0), rectangle(1.01, 2.0));
    let shell = shell_of(&[face1.clone(), face2.clone()]);
    let result = connector(&face1, &face2)
        .build(&shell, 1.0e-4, 1.0e-7)
        .unwrap();
    assert!(result.is_same(&shell));
    assert_eq!(edge_count(&result.clone().into_shape()), 8);
}

#[test]
fn clear_discards_connections() {
    let (face1, face2) = (rectangle(0.0, 1.0), rectangle(1.0, 2.0));
    let shell = shell_of(&[face1.clone(), face2.clone()]);
    let mut connector = connector(&face1, &face2);
    connector.clear();
    let result = connector.build(&shell, 1.0e-6, 1.0e-7).unwrap();
    assert!(result.is_same(&shell));
    assert_eq!(edge_count(&result.clone().into_shape()), 8);
}

#[test]
fn can_be_reused_after_clear() {
    let (face1, face2) = (rectangle(0.0, 1.0), rectangle(1.0, 2.0));
    let shell = shell_of(&[face1.clone(), face2.clone()]);
    let mut connector = connector(&face1, &face2);
    connector.clear();
    assert!(connector.add(&face1, &face2));
    let result = connector.build(&shell, 1.0e-6, 1.0e-7).unwrap();
    assert_eq!(edge_count(&result.clone().into_shape()), 7);
}

#[test]
fn empty_shell_is_returned_unchanged() {
    let shell = shell_of(&[]);
    let result = ShapeFixFaceConnect::new()
        .build(&shell, 1.0e-6, 1.0e-7)
        .unwrap();
    assert!(result.is_same(&shell));
    assert_eq!(edge_count(&result.clone().into_shape()), 0);
}

#[test]
fn self_connection_preserves_single_face_topology() {
    let face = rectangle(0.0, 1.0);
    let shell = shell_of(std::slice::from_ref(&face));
    let result = connector(&face, &face)
        .build(&shell, 1.0e-6, 1.0e-7)
        .unwrap();
    assert_eq!(edge_count(&result.clone().into_shape()), 4);
    assert!(is_valid(&result.into_shape()));
}

#[test]
fn connects_reversed_faces() {
    let mut face1 = rectangle(0.0, 1.0);
    let mut face2 = rectangle(1.0, 2.0);
    face1.reverse();
    face2.reverse();
    let shell = shell_of(&[face1.clone(), face2.clone()]);
    let result = connector(&face1, &face2)
        .build(&shell, 1.0e-6, 1.0e-7)
        .unwrap();
    assert_eq!(edge_count(&result.clone().into_shape()), 7);
    assert!(is_valid(&result.into_shape()));
}

#[test]
fn connects_three_face_chain() {
    let faces = [
        rectangle(0.0, 1.0),
        rectangle(1.0, 2.0),
        rectangle(2.0, 3.0),
    ];
    let shell = shell_of(&faces);
    let mut connector = ShapeFixFaceConnect::new();
    assert!(connector.add(&faces[0], &faces[1]));
    assert!(connector.add(&faces[1], &faces[2]));
    let result = connector.build(&shell, 1.0e-6, 1.0e-7).unwrap();
    assert_eq!(edge_count(&result.clone().into_shape()), 10);
    assert!((area(&result.clone().into_shape()) - 3.0).abs() <= 1.0e-12);
    assert!(is_valid(&result.into_shape()));
}

#[test]
fn leaves_unregistered_third_face_separate() {
    let faces = [
        rectangle(0.0, 1.0),
        rectangle(1.0, 2.0),
        rectangle(2.0, 3.0),
    ];
    let shell = shell_of(&faces);
    let result = connector(&faces[0], &faces[1])
        .build(&shell, 1.0e-6, 1.0e-7)
        .unwrap();
    assert_eq!(edge_count(&result.clone().into_shape()), 11);
}

#[test]
fn connects_two_independent_face_pairs() {
    let faces = [
        rectangle(0.0, 1.0),
        rectangle(1.0, 2.0),
        rectangle(10.0, 11.0),
        rectangle(11.0, 12.0),
    ];
    let shell = shell_of(&faces);
    let mut connector = ShapeFixFaceConnect::new();
    assert!(connector.add(&faces[0], &faces[1]));
    assert!(connector.add(&faces[2], &faces[3]));
    let result = connector.build(&shell, 1.0e-6, 1.0e-7).unwrap();
    assert_eq!(edge_count(&result.clone().into_shape()), 14);
    assert!(!is_valid(&shell.clone().into_shape()));
    assert!(!is_valid(&result.into_shape()));
}

#[test]
fn duplicate_connection_does_not_change_result() {
    let (face1, face2) = (rectangle(0.0, 1.0), rectangle(1.0, 2.0));
    let shell = shell_of(&[face1.clone(), face2.clone()]);
    let mut connector = ShapeFixFaceConnect::new();
    assert!(connector.add(&face1, &face2));
    assert!(connector.add(&face1, &face2));
    assert!(connector.add(&face2, &face1));
    let result = connector.build(&shell, 1.0e-6, 1.0e-7).unwrap();
    assert_eq!(edge_count(&result.into_shape()), 7);
}

#[test]
fn rebuilding_result_is_stable() {
    let (face1, face2) = (rectangle(0.0, 1.0), rectangle(1.0, 2.0));
    let shell = shell_of(&[face1.clone(), face2.clone()]);
    let connector = connector(&face1, &face2);
    let result1 = connector.build(&shell, 1.0e-6, 1.0e-7).unwrap();
    let result2 = connector.build(&result1, 1.0e-6, 1.0e-7).unwrap();
    assert_eq!(edge_count(&result1.clone().into_shape()), 7);
    assert_eq!(edge_count(&result2.clone().into_shape()), 7);
    assert!(is_valid(&result2.into_shape()));
}
