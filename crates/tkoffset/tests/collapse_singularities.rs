//! Native differential tests for BRepOffset::CollapseSingularities.
use rustycad_tkbrep::topods::Face;
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomBSplineSurface, GeomPlane, GeomSurface};
use rustycad_tkmath::gp::{Axis3, Point3};
use rustycad_tkoffset::brep_offset::collapse_singularities;
use rustycad_tktopalgo::brep_builder_api::{BRepBuilderAPIMakeFace, BRepBuilderAPIMakeShape};

#[test]
fn non_bspline_surface_is_returned_unchanged() {
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    let result = collapse_singularities(&plane, &Face::default(), 1.0e-7).unwrap();
    assert_eq!(result.object_address(), plane.object_address());
}

#[test]
fn only_singular_bspline_boundary_poles_are_collapsed_to_their_centroid() {
    let surface = nearly_collapsed_surface();
    let face = surface_face(&surface);
    let input: Handle<dyn GeomSurface> = surface.clone().cast();
    let result = collapse_singularities(&input, &face, 0.01).unwrap();
    let result: Handle<GeomBSplineSurface> = result.cast();
    assert_ne!(result.object_address(), input.object_address());
    assert_pole(&result, 1, 1, Point3::new(0.0, 0.0005, 0.0));
    assert_pole(&result, 1, 2, Point3::new(0.0, 0.0005, 0.0));
    assert!(
        surface
            .pole(1, 1)
            .unwrap()
            .is_equal(&Point3::new(0.0, 0.0, 0.0), 0.0)
    );
}

#[test]
fn nonsingular_bspline_surface_keeps_its_original_handle() {
    let surface = nearly_collapsed_surface();
    let face = surface_face(&surface);
    let input: Handle<dyn GeomSurface> = surface.cast();
    let result = collapse_singularities(&input, &face, 1.0e-7).unwrap();
    assert_eq!(result.object_address(), input.object_address());
}

fn assert_pole(surface: &GeomBSplineSurface, u_index: i32, v_index: i32, expected: Point3) {
    assert!(
        surface
            .pole(u_index, v_index)
            .unwrap()
            .is_equal(&expected, 1.0e-15)
    );
}

fn surface_face(surface: &Handle<GeomBSplineSurface>) -> Face {
    let base: Handle<dyn GeomSurface> = surface.clone().cast();
    let mut maker =
        BRepBuilderAPIMakeFace::from_surface_bounds(&base, (0.0, 1.0, 0.0, 1.0), 1.0e-7).unwrap();
    Face::unchecked(maker.shape().unwrap().clone())
}

fn nearly_collapsed_surface() -> Handle<GeomBSplineSurface> {
    let poles = surface_poles();
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

fn surface_poles() -> Array2<'static, Point3> {
    let mut poles: Array2<'static, Point3> = Array2::with_bounds(1, 2, 1, 2).unwrap();
    poles.set_value(1, 1, Point3::new(0.0, 0.0, 0.0)).unwrap();
    poles.set_value(1, 2, Point3::new(0.0, 0.001, 0.0)).unwrap();
    poles.set_value(2, 1, Point3::new(1.0, 0.0, 0.0)).unwrap();
    poles.set_value(2, 2, Point3::new(1.0, 1.0, 0.0)).unwrap();
    poles
}

fn array1<T: Copy + Default>(values: &[T]) -> Array1<'static, T> {
    let mut array: Array1<'static, T> = Array1::with_bounds(1, values.len() as i32).unwrap();
    array.as_mut_slice().copy_from_slice(values);
    array
}
