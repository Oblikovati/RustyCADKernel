//! Shared curve ownership preserves the borrowed point-curve projector behavior.
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::{
    adaptor3d::Adaptor3dCurve,
    geom::{GeomCurve, GeomLine},
    geom_adaptor::GeomAdaptorCurve,
};
use rustycad_tkgeombase::extrema::LocateExtPC;
use rustycad_tkmath::gp::{Axis1, Point3};
fn owned_line() -> Handle<dyn Adaptor3dCurve> {
    let line: Handle<dyn GeomCurve> = Handle::new(GeomLine::new(Axis1::default())).cast();
    Handle::new(GeomAdaptorCurve::with_range(&line, -10.0, 10.0).unwrap()).cast()
}
#[test]
fn owned_projector_retains_adaptor_after_caller_release() {
    let curve = owned_line();
    let weak = curve.downgrade();
    let mut projector = LocateExtPC::new();
    projector
        .initialize_owned(curve, (-10.0, 10.0), 1e-9)
        .unwrap();
    assert!(!weak.upgrade().is_null());
    projector.perform(Point3::new(1.0, 2.0, 3.0), 0.0).unwrap();
    assert_eq!(projector.point().unwrap().parameter(), 3.0);
    assert_eq!(projector.square_distance().unwrap(), 5.0);
    drop(projector);
    assert!(weak.upgrade().is_null());
}
#[test]
fn repeated_projection_and_reinitialization_match_borrowed() {
    let curve = owned_line();
    let mut owned = LocateExtPC::new();
    let mut borrowed = LocateExtPC::new();
    for range in [(-10.0, 10.0), (-2.0, 2.0), (-5.0, 7.0)] {
        owned.initialize_owned(curve.clone(), range, 1e-9).unwrap();
        borrowed.initialize(&*curve, range, 1e-9).unwrap();
        for z in [-3.0, 1.0, 0.0, 4.0] {
            let point = Point3::new(1.0, 2.0, z);
            owned.perform(point, 0.0).unwrap();
            borrowed.perform(point, 0.0).unwrap();
            assert_eq!(owned.is_done(), borrowed.is_done());
            if owned.is_done() {
                assert_eq!(owned.point().unwrap(), borrowed.point().unwrap());
                assert_eq!(owned.is_min().unwrap(), borrowed.is_min().unwrap());
            }
        }
    }
}
#[test]
fn null_owned_adaptor_rejected_without_resetting_previous_state() {
    let mut projector = LocateExtPC::new();
    projector
        .initialize_owned(owned_line(), (-10.0, 10.0), 1e-9)
        .unwrap();
    projector.perform(Point3::new(1.0, 2.0, 3.0), 0.0).unwrap();
    assert!(
        projector
            .initialize_owned(Handle::default(), (-1.0, 1.0), 1e-9)
            .is_err()
    );
    assert_eq!(projector.point().unwrap().parameter(), 3.0);
}
