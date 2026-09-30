//! 1:1 translation of src/ModelingAlgorithms/TKHelix/GTests/HelixGeom_Tools_Test.cxx.
#![allow(non_snake_case)]

use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve};
use rustycad_tkhelix::helix_geom::{HelixGeomHelixCurve, HelixGeomTools};
use rustycad_tkmath::geom_abs::Shape;
use std::f64::consts::PI;

/// The fixture `HelixGeom_Tools_Test`: `myTolerance = 1.e-6`.
const MY_TOLERANCE: f64 = 1.0e-6;

#[test]
fn HelixGeom_Tools_Test__ApprHelix() {
    let mut a_bspline: Handle<GeomBSplineCurve> = Handle::default();
    let mut a_max_error: f64 = 0.0;

    let a_result = HelixGeomTools::appr_helix(
        0.0,          // T1
        2.0 * PI,     // T2
        10.0,         // Pitch
        5.0,          // Start radius
        0.0,          // Taper angle
        true,         // Clockwise
        MY_TOLERANCE, // Tolerance
        &mut a_bspline,
        &mut a_max_error,
    )
    .unwrap();

    assert_eq!(a_result, 0); // Success
    assert!(!a_bspline.is_null());
    assert!(a_max_error <= MY_TOLERANCE);

    // Test curve properties
    let a_bspline = a_bspline.get().unwrap();
    assert!(a_bspline.degree() > 0);
    assert!(a_bspline.nb_poles() > 0);
}

#[test]
fn HelixGeom_Tools_Test__ApprCurve3D() {
    // Create a helix curve adaptor
    let a_helix = HelixGeomHelixCurve::new();
    a_helix.load(0.0, 2.0 * PI, 10.0, 3.0, 0.0, true).unwrap();
    let a_h_adaptor = a_helix.to_adaptor();

    let mut a_bspline: Handle<GeomBSplineCurve> = Handle::default();
    let mut a_max_error: f64 = 0.0;

    let a_result = HelixGeomTools::appr_curve_3d(
        &a_h_adaptor,
        MY_TOLERANCE,
        Shape::C1,
        50, // Max segments
        6,  // Max degree
        &mut a_bspline,
        &mut a_max_error,
    )
    .unwrap();

    assert_eq!(a_result, 0);
    assert!(!a_bspline.is_null());
    assert!(a_max_error <= MY_TOLERANCE * 10.0);

    // Verify approximation quality by sampling points
    let a_bspline = a_bspline.get().unwrap();
    let a_nb_samples = 10;
    for i in 0..=a_nb_samples {
        let (first, last) = (
            a_helix.first_parameter().unwrap(),
            a_helix.last_parameter().unwrap(),
        );
        let a_param = first + f64::from(i) * (last - first) / f64::from(a_nb_samples);

        let a_orig_pnt = a_helix.value(a_param).unwrap();

        // Map parameter to B-spline parameter space
        let a_bspline_param = a_bspline.first_parameter()
            + f64::from(i) * (a_bspline.last_parameter() - a_bspline.first_parameter())
                / f64::from(a_nb_samples);

        let a_approx_pnt = a_bspline.d0(a_bspline_param).unwrap();

        let a_distance = a_orig_pnt.distance(&a_approx_pnt);
        assert!(a_distance <= a_max_error * 2.0); // Allow some margin
    }
}

#[test]
fn HelixGeom_Tools_Test__DifferentContinuity() {
    let a_helix = HelixGeomHelixCurve::new();
    a_helix.load(0.0, 6.0 * PI, 15.0, 4.0, 0.05, true).unwrap();
    let a_h_adaptor = a_helix.to_adaptor();

    // Test C0 continuity
    let mut a_bspline_c0: Handle<GeomBSplineCurve> = Handle::default();
    let mut a_max_error_c0: f64 = 0.0;
    let a_result_c0 = HelixGeomTools::appr_curve_3d(
        &a_h_adaptor,
        MY_TOLERANCE,
        Shape::C0,
        30,
        4,
        &mut a_bspline_c0,
        &mut a_max_error_c0,
    )
    .unwrap();

    assert_eq!(a_result_c0, 0);
    assert!(!a_bspline_c0.is_null());

    // Test C2 continuity
    let mut a_bspline_c2: Handle<GeomBSplineCurve> = Handle::default();
    let mut a_max_error_c2: f64 = 0.0;
    let a_result_c2 = HelixGeomTools::appr_curve_3d(
        &a_h_adaptor,
        MY_TOLERANCE,
        Shape::C2,
        30,
        6,
        &mut a_bspline_c2,
        &mut a_max_error_c2,
    )
    .unwrap();

    assert_eq!(a_result_c2, 0);
    assert!(!a_bspline_c2.is_null());

    // C2 curve should generally have higher degree
    assert!(a_bspline_c2.get().unwrap().degree() >= a_bspline_c0.get().unwrap().degree());
}
