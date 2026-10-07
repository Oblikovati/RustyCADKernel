// Tests of BRepBlend_SurfRstLineBuilder on the floor and the top edge of a step wall.
use super::*;
use crate::blend_func::ConstRadInv;
use crate::brep_blend::{
    BRepBlendSurfCurvConstRadInv, BRepBlendSurfPointConstRadInv, BRepBlendSurfRstConstRad,
    SurfRstInverses, SurfRstWalkSettings,
};
use crate::chfi_ds::ElSpine;
use rustycad_tkg2d::geom2d::{Geom2dCurve, Geom2dLine};
use rustycad_tkg2d::geom2d_adaptor::Geom2dAdaptorCurve;
use rustycad_tkg3d::adaptor3d::{Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dTopolTool};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomPlane, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkmath::gp::{Axis3, Direction2, Direction3, Point3};
use std::rc::Rc;

fn plane(
    origin: Point3,
    n: Direction3,
    bounds: Option<(f64, f64, f64, f64)>,
) -> Handle<dyn Adaptor3dSurface> {
    let frame = Axis3::new(origin, n, Direction3::X).expect("frame");
    let geom: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(frame)).cast();
    let adaptor = match bounds {
        Some(b) => GeomAdaptorSurface::with_bounds(&geom, b),
        None => GeomAdaptorSurface::with_surface(&geom),
    };
    Handle::new(adaptor.expect("surface")).cast()
}

fn line2d(v: f64, last: f64) -> Arc {
    let geom: Handle<dyn Geom2dCurve> = Handle::new(Geom2dLine::from_point(
        Point2::new(0.0, v),
        Direction2::new(1.0, 0.0).expect("dir"),
    ))
    .cast();
    Handle::new(Geom2dAdaptorCurve::with_range(&geom, 0.0, last).expect("curve")).cast()
}

/// The functions of the step of edge length `l2` and the builder.
struct Step {
    builder: BRepBlendSurfRstLineBuilder,
    f: BRepBlendSurfRstConstRad,
    finv: ConstRadInv,
    finvp: BRepBlendSurfPointConstRadInv,
    finvc: BRepBlendSurfCurvConstRadInv,
}

fn step(l2: f64) -> Step {
    let floor = plane(
        Point3::default(),
        Direction3::Z,
        Some((0.0, 10.0, 0.0, 5.0)),
    );
    let wall = plane(Point3::default(), Direction3::Y, Some((0.0, l2, -0.5, 0.0)));
    let top = plane(Point3::new(0.0, 0.0, 0.5), Direction3::Z, None);
    let (rst, pcref) = (line2d(-0.5, l2), line2d(0.0, l2));
    let line: Handle<dyn GeomCurve> =
        Handle::new(GeomLine::from_point(Point3::default(), Direction3::X)).cast();
    let spine = ElSpine::new();
    spine.set_curve(&line).expect("spine");
    spine.set_first_parameter(-1.0);
    spine.set_last_parameter(11.0);
    let guide: Handle<dyn Adaptor3dCurve> = Handle::new(spine).cast();
    let mut f = BRepBlendSurfRstConstRad::new((&floor, &wall, &rst), &guide).expect("function");
    f.set_refs(&top, &pcref);
    f.set_radius(1.0, 2);
    let hc: Handle<dyn Adaptor3dCurve> =
        Handle::new(Adaptor3dCurveOnSurface::with_curve(&rst, &wall).expect("cos")).cast();
    let mut finvc = BRepBlendSurfCurvConstRadInv::new(&floor, &hc, &guide);
    finvc.set_radius(1.0, 2);
    let mut finvp = BRepBlendSurfPointConstRadInv::new(&floor, &guide);
    finvp.set_radius(1.0, 2);
    let mut finv = ConstRadInv::new(&floor, &top, &guide);
    crate::blend::BlendFuncInv::set_curve(&mut finv, false, &pcref).expect("inverse");
    finv.set_radius(1.0, 2);
    let d1: WalkingDomain = Rc::new(Adaptor3dTopolTool::with_surface(&floor).expect("domain"));
    let d2: WalkingDomain = Rc::new(Adaptor3dTopolTool::with_surface(&wall).expect("domain"));
    let builder = BRepBlendSurfRstLineBuilder::new(&floor, &d1, &wall, &rst, &d2);
    Step {
        builder,
        f,
        finv,
        finvp,
        finvc,
    }
}

fn settings() -> SurfRstWalkSettings {
    SurfRstWalkSettings {
        max_step: 2.0,
        tol3d: 1e-6,
        tol2d: 1e-6,
        tol_guide: 1e-5,
        fleche: 0.01,
    }
}

fn run(s: &mut Step, (pdep, pmax): (f64, f64)) {
    let contact = (1.0f64 - 0.25).sqrt();
    let start = marching::math_vector(&[pdep, contact, pdep]).expect("start");
    let inv = SurfRstInverses {
        finv: &mut s.finv,
        finv_p: &mut s.finvp,
        finv_c: &mut s.finvc,
    };
    s.builder
        .perform(&mut s.f, inv, (pdep, pmax), settings(), &start, true)
        .expect("perform");
}

#[test]
fn a_builder_that_is_not_done_has_no_line() {
    let s = step(10.0);
    let error = s
        .builder
        .line()
        .err()
        .map(|e| e.exception_type().to_string());
    assert_eq!(error.as_deref(), Some("StdFail_NotDone"));
}

#[test]
fn the_step_walks_to_the_end_of_the_edge() {
    let mut s = step(8.0);
    run(&mut s, (3.0, 12.0));
    assert!(s.builder.is_done());
    let line = lock(s.builder.line().expect("line"));
    let last = *line.point(line.nb_points()).expect("last point");
    assert!(
        (last.parameter() - 8.0).abs() < 1e-6,
        "{}",
        last.parameter()
    );
    assert!(line.end_point_on_second().is_vertex());
    assert!(line.end_point_on_second().nb_point_on_rst() >= 1);
    assert!(!s.builder.decroch_end());
}

#[test]
fn the_vertex_flag_keeps_its_vertex_until_one_is_found() {
    let mut flag = VertexFlag::default();
    flag.set(None);
    assert!(flag.vertex().is_none());
    flag.is_vtx = true;
    assert!(flag.vertex().is_some());
}
