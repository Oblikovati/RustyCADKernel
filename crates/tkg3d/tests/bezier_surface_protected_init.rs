//! Regressions for the protected native `Geom_BezierSurface::init` reuse path.
use rustycad_tkernel::ncollection::Array2;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{GeomBezierSurface, GeomPlane, GeomSurface};
use rustycad_tkg3d::geom_eval::RepSurfaceDescFull;
use rustycad_tkmath::gp::{Axis3, Point3};

fn poles(rows: i32, columns: i32, scale: f64) -> Array2<'static, Point3> {
    let mut values = Array2::with_bounds(1, rows, 1, columns).unwrap();
    for row in 1..=rows {
        for column in 1..=columns {
            values
                .set_value(
                    row,
                    column,
                    Point3::new(row as f64 * scale, column as f64 * scale, 0.),
                )
                .unwrap();
        }
    }
    values
}

fn weights(rows: i32, columns: i32, first: f64, second: f64) -> Array2<'static, f64> {
    let mut values = Array2::with_bounds(1, rows, 1, columns).unwrap();
    for row in 1..=rows {
        for column in 1..=columns {
            values
                .set_value(row, column, if column == 1 { first } else { second })
                .unwrap();
        }
    }
    values
}

fn attach_plane_representation(surface: &GeomBezierSurface) {
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(Axis3::default())).cast();
    surface
        .set_eval_representation(Handle::new(RepSurfaceDescFull::new(plane)).cast())
        .unwrap();
}

#[test]
fn protected_init_replaces_geometry_and_invalidates_caches() {
    let surface = GeomBezierSurface::new(&poles(2, 2, 1.)).unwrap();
    let before = surface.resolution(1.).unwrap();
    attach_plane_representation(&surface);
    surface
        .init(&poles(3, 2, 4.), Some(&weights(3, 2, 1., 2.)))
        .unwrap();
    assert_eq!((surface.nb_u_poles(), surface.nb_v_poles()), (3, 2));
    assert!(surface.is_u_rational());
    assert!(!surface.is_v_rational());
    assert!(!surface.has_eval_representation());
    assert_eq!(surface.pole(1, 1).unwrap(), Point3::new(4., 4., 0.));
    assert_ne!(surface.resolution(1.).unwrap(), before);
}

#[test]
fn protected_init_preserves_constructor_validation_distinction() {
    let poles = poles(3, 2, 1.);
    let zero_weights = weights(3, 2, 0., 0.);
    let error = GeomBezierSurface::with_weights(&poles, &zero_weights)
        .err()
        .unwrap();
    assert_eq!(error.exception_type(), "Standard_ConstructionError");
    let surface = GeomBezierSurface::new(&poles).unwrap();
    surface.init(&poles, Some(&zero_weights)).unwrap();
    assert!(!surface.is_u_rational() && !surface.is_v_rational());
    assert_eq!(surface.weight(1, 1).unwrap(), 1.);
}

#[test]
fn protected_init_accepts_null_weights_and_clears_representation() {
    let poles = poles(2, 2, 2.);
    let surface = GeomBezierSurface::new(&poles).unwrap();
    attach_plane_representation(&surface);
    surface.init(&poles, None).unwrap();
    assert!(!surface.is_u_rational() && !surface.is_v_rational());
    assert_eq!(surface.weight(1, 1).unwrap(), 1.);
    assert!(!surface.has_eval_representation());
}

#[test]
fn protected_init_keeps_native_partial_state_on_dimension_error() {
    let surface =
        GeomBezierSurface::with_weights(&poles(2, 2, 1.), &weights(2, 2, 1., 2.)).unwrap();
    let before = surface.resolution(1.).unwrap();
    attach_plane_representation(&surface);
    let error = surface
        .init(&poles(3, 2, 5.), Some(&weights(2, 3, 1., 2.)))
        .unwrap_err();
    assert_eq!(error.exception_type(), "Standard_DimensionMismatch");
    assert_eq!((surface.nb_u_poles(), surface.nb_v_poles()), (3, 2));
    assert_eq!(surface.pole(1, 1).unwrap(), Point3::new(5., 5., 0.));
    assert!(surface.is_u_rational());
    assert!(!surface.is_v_rational());
    assert!(surface.has_eval_representation());
    assert_eq!(surface.resolution(1.).unwrap(), before);
}
