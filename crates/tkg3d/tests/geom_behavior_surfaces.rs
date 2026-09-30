//! Regression tests for the native behaviour that the elementary and swept Geom
//! surfaces preserve (docs/tkg3d-progress.md); the numeric oracle compares them
//! bit for bit with OCCT.
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{
    GeomCircle, GeomConicalSurface, GeomCurve, GeomElementarySurface, GeomGeometry, GeomLine,
    GeomPlane, GeomSphericalSurface, GeomSurface, GeomSurfaceOfLinearExtrusion,
    GeomSurfaceOfRevolution, GeomToroidalSurface, GeomTrimmedCurve,
};
use rustycad_tkg3d::geom_eval::RepSurfaceDescFull;
use rustycad_tkmath::gp::{Axis1, Axis2, Axis3, Direction3, Point3, Transform3, Vector3};
use std::f64::consts::PI;

fn z() -> Direction3 {
    Direction3::new(0.0, 0.0, 1.0).unwrap()
}
fn frame() -> Axis3 {
    Axis3::from_normal(Point3::new(0.0, 0.0, 0.0), z()).unwrap()
}
fn z_line(x: f64) -> Handle<dyn GeomCurve> {
    Handle::new(GeomLine::from_point(Point3::new(x, 0.0, 0.0), z())).cast()
}

#[test]
fn plane_reverses_its_x_and_y_directions() {
    let plane = GeomPlane::new(frame());
    plane.u_reverse().unwrap();
    assert_eq!(plane.position().x_direction().x(), -1.0);
    plane.v_reverse().unwrap();
    assert_eq!(plane.position().y_direction().y(), -1.0);
    assert_eq!(plane.position().direction().z(), 1.0);
}

#[test]
fn cone_v_reverse_negates_the_semi_angle() {
    let cone = GeomConicalSurface::new(frame(), 0.3, 2.0).unwrap();
    cone.v_reverse().unwrap();
    assert_eq!(cone.semi_angle(), -0.3);
    assert_eq!(cone.position().direction().z(), -1.0);
    assert!(GeomConicalSurface::new(frame(), PI / 2.0, 1.0).is_err());
    assert!(cone.set_semi_angle(0.0).is_err());
}

#[test]
fn torus_constructor_accepts_what_the_setters_refuse() {
    let torus = GeomToroidalSurface::new(frame(), 1.0, 3.0).unwrap();
    assert!(torus.set_major_radius(2.0).is_err());
    assert!(torus.set_minor_radius(3.0).is_err());
    assert_eq!(torus.major_radius(), 1.0);
}

#[test]
fn sphere_u_iso_is_a_trimmed_meridian() {
    let sphere = GeomSphericalSurface::new(frame(), 2.0).unwrap();
    let iso: Handle<GeomTrimmedCurve> = sphere.u_iso(0.5).unwrap().cast();
    assert!(iso.is_some());
    // The periodic adjustment of the trimmed circle moves [-PI/2, PI/2] to [3PI/2, 5PI/2].
    assert_eq!(iso.first_parameter(), -PI / 2. + 2. * PI);
    let circle: Handle<GeomCircle> = sphere.v_iso(0.5).unwrap().cast();
    assert!(circle.is_some());
}

#[test]
fn revolution_reference_plane_is_not_implemented() {
    let axis = Axis1::new(Point3::new(0.0, 0.0, 0.0), z());
    let surface = GeomSurfaceOfRevolution::new(&z_line(2.0), axis).unwrap();
    let error = surface.reference_plane().unwrap_err();
    assert_eq!(error.exception_type(), "Standard_NotImplemented");
}

#[test]
fn revolution_mirror_reverses_u() {
    let axis = Axis1::new(Point3::new(0.0, 0.0, 0.0), z());
    let surface = GeomSurfaceOfRevolution::new(&z_line(2.0), axis).unwrap();
    let frame = Axis2::new(
        Point3::new(0.0, 0.0, 0.0),
        Direction3::new(1.0, 0.0, 0.0).unwrap(),
    );
    surface.mirror_plane(frame.unwrap()).unwrap();
    // The mirror reverses the axis direction twice: by the transformation and by UReverse.
    assert_eq!(surface.axis().direction().z(), -1.0);
}

#[test]
fn swept_basis_is_a_private_copy() {
    let basis = z_line(1.0);
    let surface =
        GeomSurfaceOfLinearExtrusion::new(&basis, Direction3::new(1.0, 0.0, 0.0).unwrap()).unwrap();
    basis.translate(Vector3::new(5.0, 0.0, 0.0)).unwrap();
    assert_eq!(surface.value(0.0, 0.0).unwrap().x(), 1.0);
}

#[test]
fn swept_moves_clear_the_evaluation_representation() {
    let surface =
        GeomSurfaceOfLinearExtrusion::new(&z_line(1.0), Direction3::new(1.0, 0.0, 0.0).unwrap())
            .unwrap();
    let rep: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(frame())).cast();
    surface
        .set_eval_representation(Handle::new(RepSurfaceDescFull::new(rep)).cast())
        .unwrap();
    assert!(surface.has_eval_representation());
    let copy: Handle<GeomSurfaceOfLinearExtrusion> = surface.copy().unwrap().cast();
    assert!(copy.has_eval_representation());
    surface.transform(Transform3::default()).unwrap();
    assert!(!surface.has_eval_representation());
}

#[test]
fn swept_evaluation_representation_rejects_itself() {
    let surface = Handle::new(
        GeomSurfaceOfLinearExtrusion::new(&z_line(1.0), Direction3::new(1.0, 0.0, 0.0).unwrap())
            .unwrap(),
    );
    let own = RepSurfaceDescFull::new(surface.cast());
    let error = surface
        .set_eval_representation(Handle::new(own).cast())
        .unwrap_err();
    assert_eq!(error.exception_type(), "Standard_ProgramError");
}

#[test]
fn trimmed_line_refuses_nan_bounds() {
    // Regression: natively SetTrim and Reverse call each other without end on the NaN
    // bounds of a non-periodic basis; the port overflowed its stack. A periodic basis
    // keeps the native NaN trim.
    for (u1, u2) in [(f64::NAN, f64::NAN), (0.0, f64::NAN), (f64::NAN, 1.0)] {
        let error = GeomTrimmedCurve::new(&z_line(1.0), u1, u2, true, true)
            .err()
            .unwrap();
        assert_eq!(error.exception_type(), "Standard_ConstructionError");
        assert!(error.message().contains("are NaN"));
    }
}
