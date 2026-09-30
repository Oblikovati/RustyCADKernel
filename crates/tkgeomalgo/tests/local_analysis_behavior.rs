//! Behavior of the LocalAnalysis port beyond the numeric probe, which compares every
//! query and the Dump text with OCCT.
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine, Geom2dTrimmedCurve};
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkgeomalgo::local_analysis::*;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Axis3, Direction2, Point2};

fn plane() -> Handle<dyn GeomSurface> {
    Handle::new(GeomPlane::new(Axis3::default())).cast()
}

#[test]
fn the_tolerance_defaults_are_the_native_ones() {
    let curve = CurveContinuityTolerances::default();
    assert_eq!(
        (curve.eps_nul, curve.eps_g2, curve.percent, curve.maxlen),
        (0.001, 0.001, 0.01, 10000.0)
    );
    let surface = SurfaceContinuityTolerances::default();
    assert_eq!(
        (surface.eps_c2, surface.percent, surface.maxlen),
        (0.001, 0.01, 10000.0)
    );
}

#[test]
fn an_analysis_without_order_has_no_status() {
    let analysis =
        LocalAnalysisSurfaceContinuity::with_tolerances(&SurfaceContinuityTolerances::default());
    assert!(analysis.is_done() && analysis.status_error().is_none());
    assert_eq!(
        analysis.continuity_status().unwrap_err().exception_type(),
        "Standard_NoSuchObject"
    );
    let mut text: Vec<u8> = Vec::new();
    dump_surface(&analysis, &mut text).unwrap();
    assert!(text.is_empty());
}

#[test]
fn a_parameter_outside_the_curves_fails_without_reason() {
    let line = Geom2dLine::from_point(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0).unwrap());
    let basis: Handle<dyn Geom2dCurve> = Handle::new(line).cast();
    let trimmed: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dTrimmedCurve::new(&basis, 0.0, 1.0, true, true).unwrap()).cast();
    let eps = SurfaceContinuityTolerances::default();
    let analysis = LocalAnalysisSurfaceContinuity::on_curves(
        (&trimmed, &trimmed),
        2.0,
        &plane(),
        &plane(),
        Shape::G1,
        &eps,
    )
    .unwrap();
    assert!(!analysis.is_done() && analysis.status_error().is_none());
    assert_eq!(
        analysis.c0_value().unwrap_err().exception_type(),
        "StdFail_NotDone"
    );
    let mut text: Vec<u8> = Vec::new();
    dump_surface(&analysis, &mut text).unwrap();
    assert_eq!(
        String::from_utf8(text).unwrap(),
        "Problem in the computation \n"
    );
}
