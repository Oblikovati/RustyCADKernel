//! Native differential tests for BRepOffset_SimpleOffset::FillFaceData.
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{
    GeomBSplineSurface, GeomCylindricalSurface, GeomOffsetSurface, GeomPlane, GeomSurface,
};
use rustycad_tkg3d::top_abs::Orientation;
use rustycad_tkmath::gp::{Axis3, Direction3, Plane3, Point3, Transform3, Vector3};
use rustycad_tkmath::toploc::Location;
use rustycad_tkoffset::brep_offset::{BRepOffsetStatus, build_simple_offset_face_data};
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape};
use std::f64::consts::TAU;

#[test]
fn offsets_plane_and_preserves_native_face_data_flags() {
    let face = plane_face();
    assert_face_data(&face, 10.0);
}

#[test]
fn face_orientation_controls_signed_offset() {
    let mut face = plane_face();
    face.set_orientation(Orientation::Reversed);
    assert_face_data(&face, -10.0);
}

#[test]
fn face_location_is_applied_before_surface_offset() {
    let mut face = plane_face();
    face.set_location(
        &Location::new(Transform3::translation_vector(Vector3::new(0.0, 0.0, 3.0))),
        false,
    )
    .unwrap();
    assert_face_data(&face, 13.0);
}

#[test]
fn analytic_cylinder_face_uses_radius_offset_surface() {
    let surface: Handle<dyn GeomSurface> =
        Handle::new(GeomCylindricalSurface::new(Axis3::default(), 5.0).unwrap()).cast();
    let face = make_face(&surface, (0.0, TAU, 0.0, 2.0));
    let data = build_simple_offset_face_data(&face, 10.0, 1.0e-7).unwrap();
    assert_eq!(data.status, BRepOffsetStatus::Good);
    assert!(data.location.is_identity());
    assert!(!data.reverse_wires && !data.reverse_face);
    assert_close_point(
        data.offset_surface.value(0.0, 0.0).unwrap(),
        Point3::new(15.0, 0.0, 0.0),
    );
}

#[test]
fn bspline_face_data_offsets_the_collapsed_prepared_basis() {
    let basis = nearly_collapsed_surface();
    let base: Handle<dyn GeomSurface> = basis.cast();
    let face = make_face(&base, (0.0, 1.0, 0.0, 1.0));
    let data = build_simple_offset_face_data(&face, 0.25, 0.01).unwrap();
    let offset: Handle<GeomOffsetSurface> = data.offset_surface.cast();
    let prepared: Handle<GeomBSplineSurface> = offset.basis_surface().cast();
    assert!(
        prepared
            .pole(1, 1)
            .unwrap()
            .is_equal(&Point3::new(0.0, 0.0005, 0.0), 1.0e-15)
    );
    assert!(
        prepared
            .pole(1, 2)
            .unwrap()
            .is_equal(&Point3::new(0.0, 0.0005, 0.0), 1.0e-15)
    );
}

fn assert_face_data(face: &Face, expected_z: f64) {
    let data = build_simple_offset_face_data(face, 10.0, 1.0e-7).unwrap();
    assert_eq!(data.status, BRepOffsetStatus::Good);
    assert!(data.location.is_identity());
    assert_eq!(data.tolerance, 1.0e-7);
    assert!(!data.reverse_wires && !data.reverse_face);
    assert_close_point(
        data.offset_surface.value(0.0, 0.0).unwrap(),
        Point3::new(0.0, 0.0, expected_z),
    );
}

fn assert_close_point(actual: Point3, expected: Point3) {
    assert!(actual.distance(&expected) < 1.0e-12);
}

fn plane_face() -> Face {
    let plane = Plane3::new(Point3::default(), Direction3::new(0.0, 0.0, 1.0).unwrap()).unwrap();
    let surface: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(plane.position())).cast();
    make_face(&surface, (-1.0, 1.0, -1.0, 1.0))
}

fn make_face(surface: &Handle<dyn GeomSurface>, bounds: (f64, f64, f64, f64)) -> Face {
    let mut maker = BRepBuilderAPIMakeFace::from_surface_bounds(surface, bounds, 1.0e-7).unwrap();
    Face::unchecked(maker.shape().unwrap().clone())
}

fn nearly_collapsed_surface() -> Handle<GeomBSplineSurface> {
    let mut poles: Array2<'static, Point3> = Array2::with_bounds(1, 2, 1, 2).unwrap();
    poles.set_value(1, 1, Point3::new(0.0, 0.0, 0.0)).unwrap();
    poles.set_value(1, 2, Point3::new(0.0, 0.001, 0.0)).unwrap();
    poles.set_value(2, 1, Point3::new(1.0, 0.0, 0.0)).unwrap();
    poles.set_value(2, 2, Point3::new(1.0, 1.0, 0.0)).unwrap();
    let knots = array1(&[0.0, 1.0]);
    let multiplicities = array1(&[2, 2]);
    Handle::new(
        GeomBSplineSurface::new(
            &poles,
            (&knots, &multiplicities),
            (&knots, &multiplicities),
            (1, 1),
            (false, false),
        )
        .unwrap(),
    )
}

fn array1<T: Copy + Default>(values: &[T]) -> Array1<'static, T> {
    let mut array: Array1<'static, T> = Array1::with_bounds(1, values.len() as i32).unwrap();
    array.as_mut_slice().copy_from_slice(values);
    array
}
