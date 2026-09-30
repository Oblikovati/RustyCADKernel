//! Rust deviation: ComputeApprox refuses a range that the UV-bounds scan cannot
//! advance over (a zero-length range, or one shorter than the rounding step at its
//! ends), where the native `Function_SetUVBounds` loop never ends
//! (docs/tkgeombase-progress.md, Rust deviations).
use super::support::{cone, cylinder, dir, flat_patch_within, line, ranged, sphere, torus};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dSurface};
use rustycad_tkgeombase::proj_lib::{ComputeApprox, ProjectedCurve};
use rustycad_tkmath::gp::Point3;

// A segment of the line through (3, 0, 0) along (0.6, 0.8, 0) over `[first, last]`.
fn segment(first: f64, last: f64) -> Handle<dyn Adaptor3dCurve> {
    let origin: Point3 = Point3::new(3.0, 0.0, 0.0);
    ranged(line(origin, dir(0.6, 0.8, 0.0)), first, last)
}

// Every surface kind whose window is found by a sampling scan.
fn scanned_surfaces() -> [Handle<dyn Adaptor3dSurface>; 4] {
    [sphere(2.0), cylinder(2.0), cone(0.3), torus()]
}

fn assert_endless_scan_refused(failure: &Failure, range: &str) {
    assert_eq!(failure.exception_type(), "Standard_DomainError");
    let message: &str = failure.message();
    assert!(message.contains(range), "{message}");
    assert!(message.contains("never ends"), "{message}");
}

#[test]
fn compute_approx_refuses_a_zero_length_range_on_every_scanned_surface() {
    // Regression: the scan step of a zero-length range is 0, so the native sampling
    // loop `for (par = W1 + Step; par <= W2; par += Step)` never ends.
    for surface in scanned_surfaces() {
        let result = ComputeApprox::with_curve(&segment(1.0, 1.0), &surface, 1e-6);
        assert_endless_scan_refused(
            &result.err().expect("the endless scan is refused"),
            "[1, 1]",
        );
    }
}

#[test]
fn compute_approx_refuses_a_range_below_the_rounding_step() {
    // A range of one ulp gives a step of half an ulp, which rounds back to the same
    // parameter: the loop stalls as with a zero step.
    let last: f64 = 1.0 + f64::EPSILON;
    for surface in scanned_surfaces() {
        let result = ComputeApprox::with_curve(&segment(1.0, last), &surface, 1e-6);
        assert_endless_scan_refused(
            &result.err().expect("the endless scan is refused"),
            "[1, 1.0000000000000002]",
        );
    }
}

#[test]
fn compute_approx_keeps_a_short_range_whose_scan_advances() {
    // A short range whose step still moves the parameter is approximated as natively.
    for surface in scanned_surfaces() {
        let result = ComputeApprox::with_curve(&segment(1.0, 1.0 + 1e-6), &surface, 1e-6);
        assert!(result.is_ok());
    }
}

#[test]
fn projected_curve_reports_the_refused_zero_length_range() {
    // GeomProjLib::Curve2d reaches ComputeApprox through ProjectedCurve.
    let result = ProjectedCurve::with_tolerance(&sphere(2.0), &segment(1.0, 1.0), 1e-7);
    assert_endless_scan_refused(
        &result.err().expect("the endless scan is refused"),
        "[1, 1]",
    );
}

#[test]
fn projected_curve_refuses_a_zero_width_surface_range_in_the_pole_check() {
    // Regression: IsoIsDeg samples the U range of the patch in steps of a tenth of it;
    // with U1 == U2 the step is 0 and the native `for (T = U1; T <= U2; T += Step)`
    // never ends. The call must return, with the stalled scan named.
    let patch = flat_patch_within((0.5, 0.5, 0.0, 1.0));
    let result = ProjectedCurve::with_tolerance(&patch, &segment(0.0, 1.0), 1e-7);
    let failure: Failure = result.err().expect("the endless scan is refused");
    assert_endless_scan_refused(&failure, "[0.5, 0.5]");
    assert!(
        failure.message().contains("IsoIsDeg"),
        "{}",
        failure.message()
    );
}

#[test]
fn projected_curve_refuses_a_one_ulp_surface_range_in_the_pole_check() {
    // A range of one ulp gives a step of a tenth of an ulp, which rounds back.
    let high: f64 = 0.5 + f64::EPSILON / 2.;
    let patch = flat_patch_within((0.5, high, 0.0, 1.0));
    let result = ProjectedCurve::with_tolerance(&patch, &segment(0.0, 1.0), 1e-7);
    assert_endless_scan_refused(
        &result.err().expect("the endless scan is refused"),
        "[0.5, ",
    );
}
