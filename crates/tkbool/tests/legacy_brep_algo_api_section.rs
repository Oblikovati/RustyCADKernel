//! Direct translation of src/ModelingAlgorithms/TKBool/GTests/BRepAlgoAPI_Section_Test.cxx (3d097a0).
#![allow(non_snake_case, clippy::excessive_precision)]

use rustycad_tkbo::brep_algo_api::BRepAlgoAPISection;
use rustycad_tkbrep::brep::BRepTool;
use rustycad_tkbrep::topexp::Explorer;
use rustycad_tkbrep::topods::{self, Face, Shape};
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{
    GeomBSplineCurve, GeomCylindricalSurface, GeomPlane, GeomRectangularTrimmedSurface, GeomSurface,
};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkgeombase::geom_convert;
use rustycad_tkgeombase::gprop::GProps;
use rustycad_tkmath::gp::{self, Axis3, Direction3, Plane3, Point3};
use rustycad_tkprim::brep_prim_api::{BRepPrimAPIMakeCylinder, BRepPrimAPIMakeSphere};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape};
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
use rustycad_tktopalgo::brep_gprop::BRepGProp;

/// `EXPECT_NEAR(actual, expected, tolerance)`.
fn expect_near(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "{actual} differs from {expected} by more than {tolerance}"
    );
}

/// `gp_Dir(x, y, z)`.
fn dir(x: f64, y: f64, z: f64) -> Direction3 {
    Direction3::new(x, y, z).unwrap()
}

/// `MakeHalfCylinderFace(theAxis, theRadius, theFirstU, theLastU)`.
fn MakeHalfCylinderFace(
    the_axis: Axis3,
    the_radius: f64,
    the_first_u: f64,
    the_last_u: f64,
) -> Face {
    let a_cylinder: Handle<dyn GeomSurface> =
        Handle::new(GeomCylindricalSurface::new(the_axis, the_radius).unwrap()).cast();
    let a_trimmed_cylinder: Handle<dyn GeomSurface> = Handle::new(
        GeomRectangularTrimmedSurface::with_one_direction(
            &a_cylinder,
            the_first_u,
            the_last_u,
            true,
            true,
        )
        .unwrap(),
    )
    .cast();
    BRepBuilderAPIMakeFace::from_surface(&a_trimmed_cylinder, precision::confusion())
        .unwrap()
        .face()
        .unwrap()
}

/// `CheckCylinderSection(theFirstAxis, theFirstRadius, theSecondAxis, theSecondRadius,
/// theExpectedLength)`.
fn CheckCylinderSection(
    the_first_axis: Axis3,
    the_first_radius: f64,
    the_second_axis: Axis3,
    the_second_radius: f64,
    the_expected_length: f64,
) {
    let a_pi = std::f64::consts::PI; // the literal 3.14159265358979323846
    let a_first_face = MakeHalfCylinderFace(the_first_axis, the_first_radius, 0.0, a_pi);
    let a_second_face = MakeHalfCylinderFace(the_second_axis, the_second_radius, 0.0, a_pi);
    assert!(!a_first_face.is_null());
    assert!(!a_second_face.is_null());
    let mut a_section =
        BRepAlgoAPISection::from_shapes(&a_first_face, &a_second_face, None, true).unwrap();
    a_section.build().unwrap();
    assert!(a_section.is_done());
    let a_result = a_section.shape().clone();
    assert!(!a_result.is_null());
    assert!(
        BRepCheckAnalyzer::new(&a_result, true, false, false)
            .unwrap()
            .is_valid()
            .unwrap()
    );
    let mut a_properties = GProps::default();
    BRepGProp::linear_properties(&a_result, &mut a_properties, false, false).unwrap();
    expect_near(a_properties.mass(), the_expected_length, 0.001);
}

/// OCCN2: BRepAlgoAPI_Section of a cylinder intersecting a sphere.
#[test]
fn occn2_cylinder_sphere_section_is_done() {
    let a_cylinder: Shape = BRepPrimAPIMakeCylinder::new(50., 200.)
        .shape()
        .unwrap()
        .clone();
    let a_sphere = BRepPrimAPIMakeSphere::at_center(Point3::new(60., 0., 100.), 50.)
        .unwrap()
        .shape()
        .unwrap()
        .clone();
    let a_section = BRepAlgoAPISection::from_shapes(&a_cylinder, &a_sphere, None, true).unwrap();
    assert!(a_section.is_done());
    assert!(!a_section.shape().is_null());
}

/// The section of the two faces, without or with approximation, with one edge whose
/// curve is a B-spline of two poles.
fn CheckTwoPoleSection(horizontal: &Face, vertical: &Face, the_approximation: bool) {
    let mut a_section = BRepAlgoAPISection::from_shapes(horizontal, vertical, None, false).unwrap();
    a_section.approximation(the_approximation);
    a_section.build().unwrap();
    assert!(a_section.is_done());
    let mut a_nb_edges = 0;
    for e in Explorer::new(a_section.shape(), ShapeEnum::Edge, ShapeEnum::Shape).unwrap() {
        let (a_curve, _, _) = BRepTool::curve(&topods::edge(&e).unwrap()).unwrap();
        let a_bspline: Handle<GeomBSplineCurve> = a_curve.cast();
        assert!(!a_bspline.is_null());
        assert_eq!(a_bspline.get().unwrap().nb_poles(), 2);
        a_nb_edges += 1;
    }
    assert_eq!(a_nb_edges, 1);
}

/// bug21564: intersections with a converted trimmed plane stay two-pole curves, with
/// and without approximation.
#[test]
fn occ21564_plane_intersection_keeps_two_poles() {
    let a_horizontal_plane: Handle<dyn GeomSurface> =
        Handle::new(GeomPlane::new(Axis3::from_axis2(gp::xoy()))).cast();
    let a_trimmed_plane: Handle<dyn GeomSurface> = Handle::new(
        GeomRectangularTrimmedSurface::new(
            &a_horizontal_plane,
            (-10.0, 10.0),
            (-10.0, 10.0),
            (true, true),
        )
        .unwrap(),
    )
    .cast();
    let a_bspline_plane = geom_convert::surface_to_bspline_surface(&a_trimmed_plane).unwrap();
    assert!(!a_bspline_plane.is_null());
    let a_horizontal_face_builder =
        BRepBuilderAPIMakeFace::from_surface(&a_bspline_plane.cast(), precision::confusion())
            .unwrap();
    assert!(a_horizontal_face_builder.is_done());
    let a_vertical_plane = Plane3::new(Point3::new(0.0, 0.0, 0.0), dir(1.0, 0.0, 0.0)).unwrap();
    let a_vertical_face_builder =
        BRepBuilderAPIMakeFace::from_plane_bounds(a_vertical_plane, (-10.0, 10.0, -10.0, 10.0))
            .unwrap();
    assert!(a_vertical_face_builder.is_done());
    let horizontal = a_horizontal_face_builder.face().unwrap();
    let vertical = a_vertical_face_builder.face().unwrap();
    CheckTwoPoleSection(&horizontal, &vertical, false);
    CheckTwoPoleSection(&horizontal, &vertical, true);
}

/// The frames of the bug23932 cylinders.
fn Frames() -> (Axis3, Axis3) {
    let first = Axis3::new(
        Point3::new(538.57646417050069, 347.77316708315834, 183.375),
        dir(0.0, -1.0, 0.0),
        dir(1.0, 0.0, 0.0),
    )
    .unwrap();
    let second = Axis3::new(
        Point3::new(859.00000000000011, 463.75990629028450, 80.865093709715211),
        dir(-1.0, 0.0, 0.0),
        dir(0.0, -1.0, 0.0),
    )
    .unwrap();
    (first, second)
}

/// bug23932_1: intersecting two trimmed cylinders completes without
/// Standard_NoSuchObject and preserves length.
#[test]
fn modalg_bug_23932_section_between_trimmed_cylinders1() {
    let (first, second) = Frames();
    CheckCylinderSection(
        first,
        44.194173824159222,
        second,
        186.78835506149036,
        326.401,
    );
}

/// bug23932_2: the nearby cylinder radii exercise the second historical section
/// configuration.
#[test]
fn modalg_bug_23932_section_between_trimmed_cylinders2() {
    let (first, second) = Frames();
    CheckCylinderSection(
        first,
        44.819173824159222,
        second,
        187.41335506149036,
        330.903,
    );
}
