// Tests of BRepBlend_RstRstLineBuilder that need no blend function.
use super::*;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::Adaptor3dTopolTool;
use rustycad_tkg3d::geom::{GeomPlane, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkmath::gp::{Axis3, Direction2, Direction3, Point2, Point3};
use std::rc::Rc;

fn builder() -> BRepBlendRstRstLineBuilder {
    let frame = Axis3::new(Point3::default(), Direction3::Z, Direction3::X).expect("frame");
    let geom: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(frame)).cast();
    let s: Handle<dyn Adaptor3dSurface> =
        Handle::new(GeomAdaptorSurface::with_bounds(&geom, (0.0, 1.0, 0.0, 1.0)).expect("surface"))
            .cast();
    let line: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(
        Point2::new(0.0, 0.5),
        Direction2::new(1.0, 0.0).expect("dir"),
    ))
    .cast();
    let rst: Arc =
        Handle::new(Geom2dAdaptorCurve::with_range(&line, 0.0, 1.0).expect("curve")).cast();
    let d: WalkingDomain = Rc::new(Adaptor3dTopolTool::with_surface(&s).expect("domain"));
    BRepBlendRstRstLineBuilder::new((&s, &rst, &d), (&s, &rst, &d))
}

#[test]
fn a_builder_that_is_not_done_has_no_line() {
    let b = builder();
    assert!(!b.is_done());
    let error = b.line().err().map(|e| e.exception_type().to_string());
    assert_eq!(error.as_deref(), Some("StdFail_NotDone"));
}

#[test]
fn the_unhooking_flags_follow_the_status() {
    let mut b = builder();
    assert!(!b.decroch1_start() && !b.decroch2_end());
    b.decrochdeb = DecrochStatus::DecrochBoth;
    b.decrochfin = DecrochStatus::DecrochRst2;
    assert!(b.decroch1_start() && b.decroch2_start());
    assert!(!b.decroch1_end() && b.decroch2_end());
}
