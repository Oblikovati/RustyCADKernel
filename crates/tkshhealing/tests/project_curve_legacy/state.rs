#[test]
fn InitWithGeomSurface_G1() {
    let projector = ShapeConstructProjectCurveOnSurface::new();
    projector.init(&plane(0.), precision::confusion()).unwrap();
    successful(whole(&projector, &diagonal(0.)));
}
#[test]
fn SetSurfaceAndPrecision_G2() {
    let projector = ShapeConstructProjectCurveOnSurface::new();
    projector.set_surface(&plane(0.)).unwrap();
    projector.set_precision(precision::confusion());
    successful(whole(&projector, &diagonal(0.)));
}
#[test]
fn AdjustOverDegenModeAccessor_G3() {
    let projector = ShapeConstructProjectCurveOnSurface::new();
    assert_eq!(projector.adjust_over_degen_mode(), 1);
    projector.set_adjust_over_degen_mode(0);
    assert_eq!(projector.adjust_over_degen_mode(), 0);
    projector.set_adjust_over_degen_mode(1);
    assert_eq!(projector.adjust_over_degen_mode(), 1);
}
#[test]
fn StatusMethod_G4() {
    let projector = fixture(&plane(0.), precision::confusion());
    let _ = whole(&projector, &diagonal(0.));
    assert!(projector.status(ShapeExtendStatus::Ok) || projector.status(ShapeExtendStatus::Done));
}
#[test]
fn TightTolerance_H1() {
    let projector = fixture(&plane(0.), 1e-15);
    successful(whole(&projector, &diagonal(0.)));
}
#[test]
fn MultipleProjections_J1() {
    let projector = fixture(&plane(0.), precision::confusion());
    for index in 0..5 {
        let offset = f64::from(index) * 10.;
        let curve = segment(
            Point3::new(offset, 0., 0.),
            Point3::new(offset + 5., 5., 0.),
        );
        successful(whole(&projector, &curve));
    }
}
#[test]
fn SurfaceReinitialization_J2() {
    let projector = fixture(&plane(0.), precision::confusion());
    let (_, pcurve) = whole(&projector, &diagonal(0.));
    assert!(!pcurve.is_null());
    projector.init_analysis(
        &Handle::new(ShapeAnalysisSurface::new(&plane(5.)).unwrap()),
        precision::confusion(),
    );
    let (_, pcurve) = whole(&projector, &diagonal(5.));
    assert!(!pcurve.is_null());
}
