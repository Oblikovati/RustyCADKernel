//! Native-backed cases for `ShapeAnalysis_CheckSmallFace::CheckTwisted`.
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomBezierSurface, GeomPlane, GeomSurface};
use rustycad_tkmath::gp::{Axis3, Point3};
use rustycad_tkshhealing::shape_analysis::ShapeAnalysisCheckSmallFace;
use rustycad_tkshhealing::shape_extend::ShapeExtendStatus;

const ORACLE: &str = include_str!("../../../tools/oracle/shape_analysis_check_twisted.expected");

#[test]
fn twisted_support_sampling_matches_native_first_reversal() {
    assert_case(
        "elementary",
        Handle::new(GeomPlane::new(Axis3::default())).cast(),
    );
    assert_case("flat", bezier(false));
    assert_case("twisted", bezier(true));
}

#[test]
fn twisted_scan_requires_precision_only_after_elementary_fast_path() {
    let mut analyzer = ShapeAnalysisCheckSmallFace::new();
    let mut parameter_u = 17.0;
    let mut parameter_v = 18.0;
    assert!(
        !analyzer
            .check_twisted(
                &face(Handle::new(GeomPlane::new(Axis3::default())).cast()),
                &mut parameter_u,
                &mut parameter_v
            )
            .unwrap()
    );
    let failure = analyzer
        .check_twisted(&face(bezier(true)), &mut parameter_u, &mut parameter_v)
        .unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_ConstructionError");
}

fn assert_case(name: &str, surface: Handle<dyn GeomSurface>) {
    let expected = oracle_row(name);
    let mut analyzer = ShapeAnalysisCheckSmallFace::new();
    analyzer.set_tolerance(1.0e-4);
    let (mut parameter_u, mut parameter_v) = (77.0, 88.0);
    let found = analyzer
        .check_twisted(&face(surface), &mut parameter_u, &mut parameter_v)
        .unwrap();
    assert_eq!(found as i32, expected[0] as i32, "{name} found");
    assert_parameter(name, "U", parameter_u, expected[1]);
    assert_parameter(name, "V", parameter_v, expected[2]);
    assert_eq!(
        analyzer.status_twisted(ShapeExtendStatus::Done) as i32,
        expected[3] as i32
    );
}

fn assert_parameter(name: &str, axis: &str, actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 1.0e-15,
        "{name} {axis} parameter: {actual} != {expected}"
    );
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

fn face(surface: Handle<dyn GeomSurface>) -> Face {
    let mut face = Face::default();
    BRepBuilder::default()
        .make_face_on_surface(&mut face, &surface, 1.0e-7)
        .unwrap();
    face
}

fn bezier(twisted: bool) -> Handle<dyn GeomSurface> {
    let mut poles = Array2::<Point3>::with_bounds(1, 3, 1, 2).unwrap();
    for u in 1..=3 {
        let x = if twisted {
            if u == 2 { 10.0 } else { 0.0 }
        } else {
            f64::from(u - 1)
        };
        poles[(u, 1)] = Point3::new(x, 0.0, 0.0);
        poles[(u, 2)] = Point3::new(x, 1.0, 0.0);
    }
    Handle::new(GeomBezierSurface::new(&poles).unwrap()).cast()
}
