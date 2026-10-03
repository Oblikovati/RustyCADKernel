//! Native-backed cases for `ShapeAnalysis_CheckSmallFace::CheckPin`.
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomBezierSurface, GeomPlane, GeomSurface};
use rustycad_tkmath::gp::{Axis3, Point3};
use rustycad_tkshhealing::shape_analysis::ShapeAnalysisCheckSmallFace;
use rustycad_tkshhealing::shape_extend::ShapeExtendStatus;

const ORACLE: &str = include_str!("../../../tools/oracle/shape_analysis_check_pin.expected");

#[test]
fn check_pin_matches_native_pole_classification_and_output_updates() {
    assert_case(
        "elementary",
        surface_face(Handle::new(GeomPlane::new(Axis3::default())).cast()),
        false,
        77,
        88,
        None,
    );
    assert_case(
        "regular",
        surface_face(bezier(false)),
        true,
        2,
        2,
        Some(ShapeExtendStatus::Done2),
    );
    assert_case(
        "sharp_marker",
        surface_face(bezier(true)),
        true,
        2,
        2,
        Some(ShapeExtendStatus::Done3),
    );
}

#[test]
fn check_pin_requires_initialized_native_precision_for_supported_surfaces() {
    let mut analyzer = ShapeAnalysisCheckSmallFace::new();
    let mut row = 77;
    let mut sense = 88;
    let failure = analyzer
        .check_pin(&surface_face(bezier(false)), &mut row, &mut sense)
        .unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_ConstructionError");
    assert_eq!((row, sense), (77, 88));
}

fn assert_case(
    name: &str,
    face: Face,
    expected_found: bool,
    expected_row: i32,
    expected_sense: i32,
    status: Option<ShapeExtendStatus>,
) {
    let mut analyzer = ShapeAnalysisCheckSmallFace::new();
    analyzer.set_tolerance(1.0e-4);
    let mut row = expected_row;
    let mut sense = expected_sense;
    let found = analyzer.check_pin(&face, &mut row, &mut sense).unwrap();
    let values = oracle_row(name);
    assert_eq!(found as i32, values[0] as i32, "{name}");
    assert_eq!(found, expected_found, "{name}");
    assert_eq!(row, values[1] as i32, "{name} row");
    assert_eq!(sense, values[2] as i32, "{name} sense");
    if let Some(status) = status {
        assert!(analyzer.status_pin(status), "{name} status");
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

fn surface_face(surface: Handle<dyn GeomSurface>) -> Face {
    let mut face = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1.0e-7)
        .unwrap();
    face
}

fn bezier(sharp_last: bool) -> Handle<dyn GeomSurface> {
    let mut poles = Array2::<Point3>::with_bounds(1, 2, 1, 2).unwrap();
    poles[(1, 1)] = Point3::new(0.0, 0.0, 0.0);
    poles[(1, 2)] = if sharp_last {
        Point3::new(0.0, 0.0, 0.0)
    } else {
        Point3::new(0.0, 5.0e-5, 0.0)
    };
    poles[(2, 1)] = Point3::new(1.0, 0.0, 0.0);
    poles[(2, 2)] = if sharp_last {
        Point3::new(5.0e-5, 0.0, 0.0)
    } else {
        Point3::new(1.0, 5.0e-5, 0.0)
    };
    Handle::new(GeomBezierSurface::new(&poles).unwrap()).cast()
}
