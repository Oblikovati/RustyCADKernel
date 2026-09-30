//! ProjectOnPlane, ComputeApprox, ComputeApproxOnPolarSurface, ProjectedCurve and
//! CompProjectedCurve.
use super::support::{
    curve, cylinder, dir, extrusion, horizontal_circle, kind, line, plane, ranged, sphere,
    vertical_parabola, xoy,
};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkgeombase::proj_lib::{
    self as projlib, CompProjectedCurve, ComputeApprox, ComputeApproxOnPolarSurface,
    ProjectOnPlane, ProjectedCurve,
};
use rustycad_tkmath::geom_abs::{CurveType, Shape};
use rustycad_tkmath::gp::Point3;

#[test]
fn project_on_plane_keeps_a_circle_a_circle() {
    let projection = ProjectOnPlane::with_direction(&xoy(), dir(0.0, 0.0, 1.0));
    projection
        .load(&curve(horizontal_circle(4.0, 2.0)), 1e-7, true)
        .unwrap();
    assert_eq!(projection.get_type().unwrap(), CurveType::Circle);
    let circle = projection.circle().unwrap();
    assert_eq!(circle.radius(), 2.0);
    assert_eq!(circle.location(), Point3::new(0.0, 0.0, 0.0));
    assert!(projection.is_periodic().unwrap());
}

#[test]
fn project_on_plane_of_a_parabola_along_its_axis_is_a_null_object() {
    // The parabola axis is Z: its projection along Z is a line, and the native code
    // trims the null parabola handle.
    let projection = ProjectOnPlane::with_direction(&xoy(), dir(0.0, 0.0, 1.0));
    let parabola = ranged(vertical_parabola(), -1.0, 1.0);
    assert_eq!(
        kind(projection.load(&parabola, 1e-7, true)),
        "Standard_NullObject"
    );
}

#[test]
fn project_on_plane_defaults_to_x_onto_xoy() {
    let projection = ProjectOnPlane::new();
    assert_eq!(projection.get_direction(), dir(1.0, 0.0, 0.0));
    assert_eq!(projection.get_plane().direction(), dir(0.0, 0.0, 1.0));
    assert!(projection.get_curve().is_null());
}

#[test]
fn project_on_plane_refuses_a_kept_line_along_a_direction_in_the_plane() {
    // Regression: the default projection runs along X onto XOY, a direction in the
    // plane, so a line normal to the plane projects to NaN parameters. Natively the
    // kept parametrization of that NaN segment never returns; the port overflowed the
    // stack. Every line is refused alike (docs/tkgeombase-progress.md).
    let vertical = ranged(
        line(Point3::new(1.0, 2.0, 3.0), dir(0.0, 0.0, 1.0)),
        0.0,
        3.0,
    );
    let in_plane = ranged(
        line(Point3::new(0.0, 0.0, 0.0), dir(1.0, 1.0, 0.0)),
        -1.0,
        2.0,
    );
    for segment in [vertical, in_plane] {
        let projection = ProjectOnPlane::new();
        let error = projection.load(&segment, 1e-7, true).unwrap_err();
        assert_eq!(error.exception_type(), "Standard_ConstructionError");
        assert!(
            error
                .message()
                .contains("the direction (1, 0, 0) is parallel to the plane")
        );
        // Without the kept parametrization the NaN line is the native result.
        projection.load(&segment, 1e-7, false).unwrap();
        assert_eq!(projection.get_type().unwrap(), CurveType::Line);
        assert!(projection.first_parameter().unwrap().is_nan());
    }
}

#[test]
fn compute_approx_fits_a_circle_on_a_sphere() {
    let equator = ranged(horizontal_circle(1.0, 3.0_f64.sqrt()), 0.5, 2.0);
    let approx = ComputeApprox::with_curve(&equator, &sphere(2.0), 1e-6).unwrap();
    assert!(!approx.bspline().is_null());
    assert!(approx.bezier().is_null());
    assert!(approx.tolerance() < 1e-6);
}

#[test]
fn polar_projection_has_no_initial_curve_on_a_plane() {
    let mut polar = ComputeApproxOnPolarSurface::new();
    let segment = ranged(
        line(Point3::new(0.0, 0.0, 1.0), dir(1.0, 0.0, 0.0)),
        0.0,
        1.0,
    );
    assert_eq!(
        kind(polar.build_initial_curve2d(&segment, &plane())),
        "Standard_NoSuchObject"
    );
    assert_eq!(
        kind(ComputeApproxOnPolarSurface::with_curve(
            &segment,
            &plane(),
            1e-4
        )),
        "Standard_NoSuchObject"
    );
}

#[test]
fn polar_projection_on_a_cylinder_is_done() {
    let arc = ranged(horizontal_circle(1.0, 2.0), 0.25, 2.0);
    let polar = ComputeApproxOnPolarSurface::with_curve(&arc, &cylinder(2.0), 1e-4).unwrap();
    assert!(polar.is_done());
    assert!(!polar.bspline().is_null());
    assert!(polar.curve2d().is_ok());
}

#[test]
fn projected_curve_gives_the_analytic_line_on_a_cylinder() {
    let generatrix = ranged(
        line(Point3::new(2.0, 0.0, 0.0), dir(0.0, 0.0, 1.0)),
        0.0,
        3.0,
    );
    let projected = ProjectedCurve::with_curve(&cylinder(2.0), &generatrix).unwrap();
    assert_eq!(projected.get_type().unwrap(), CurveType::Line);
    let pcurve = projlib::make_pcurve_of_type(&projected).unwrap();
    assert!(!pcurve.is_null());
}

#[test]
fn projected_curve_evaluators_are_not_implemented() {
    let generatrix = ranged(
        line(Point3::new(2.0, 0.0, 0.0), dir(0.0, 0.0, 1.0)),
        0.0,
        3.0,
    );
    let projected = ProjectedCurve::with_curve(&cylinder(2.0), &generatrix).unwrap();
    assert_eq!(kind(projected.eval_d0(1.0)), "Standard_NotImplemented");
    assert_eq!(kind(projected.continuity()), "Standard_NotImplemented");
    assert_eq!(
        kind(projected.nb_intervals(Shape::C0)),
        "Standard_NotImplemented"
    );
    assert_eq!(kind(projected.resolution(1e-7)), "Standard_NotImplemented");
}

#[test]
fn projected_curve_without_a_part_keeps_the_null_bspline() {
    let far = ranged(
        line(Point3::new(10.0, 0.0, 1.0), dir(0.0, 1.0, 0.0)),
        0.0,
        1.0,
    );
    let projected = ProjectedCurve::with_surface(&extrusion());
    projected.set_max_dist(0.1);
    projected.perform(&far).unwrap();
    assert_eq!(projected.get_type().unwrap(), CurveType::BSplineCurve);
    assert!(projected.result().bspline().is_null());
    assert_eq!(kind(projected.degree()), "Standard_NullObject");
    assert_eq!(kind(projlib::make_pcurve_of_type(&projected)), "ok");
}

#[test]
fn comp_projected_curve_second_init_appends_the_parts() {
    let arc = ranged(horizontal_circle(1.0, 3.0), 0.0, 1.0);
    let projection = CompProjectedCurve::with_tolerances(&extrusion(), &arc, 1e-7, 1e-7).unwrap();
    let parts: i32 = projection.nb_curves();
    assert!(parts >= 1);
    projection.init().unwrap();
    assert_eq!(projection.nb_curves(), 2 * parts);
}

#[test]
fn comp_projected_curve_raises_on_high_orders_and_g1() {
    let arc = ranged(horizontal_circle(1.0, 3.0), 0.0, 1.0);
    let projection = CompProjectedCurve::with_tolerances(&extrusion(), &arc, 1e-7, 1e-7).unwrap();
    assert_eq!(kind(projection.eval_dn(0.5, 3)), "Standard_NotImplemented");
    assert_eq!(kind(projection.eval_dn(0.5, 0)), "Standard_OutOfRange");
    assert_eq!(
        kind(projection.nb_intervals(Shape::G1)),
        "Standard_OutOfRange"
    );
    assert_eq!(kind(projection.bounds(0)), "Standard_NoSuchObject");
    let (first, last) = projection.bounds(1).unwrap();
    assert!(first <= last);
}

#[test]
fn comp_projected_curve_default_has_a_null_sequence() {
    let projection = CompProjectedCurve::new();
    assert_eq!(projection.nb_curves(), 0);
    assert!(projection.get_sequence().is_null());
    assert!(projection.get_proj2d() && !projection.get_proj3d());
}
