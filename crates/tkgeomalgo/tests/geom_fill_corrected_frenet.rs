//! All declarations of GeomFill_CorrectedFrenet_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871, including the original wire reproducer.
#![allow(non_snake_case)]
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkg3d::geom::{GeomBSplineCurve, GeomCurve};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeomalgo::geom_fill::{GeomFillCorrectedFrenet, GeomFillTrihedronLaw, TrihedronD0};
use rustycad_tkmath::gp::Point3;

/// An adaptor on the B-spline of `poles` on `[0, 1]` without inner knots.
fn bspline_adaptor(poles: &[Point3], degree: i32) -> Handle<dyn Adaptor3dCurve> {
    let mut p: Array1<'static, Point3> = Array1::with_bounds(1, poles.len() as i32).unwrap();
    p.as_mut_slice().copy_from_slice(poles);
    let mut knots: Array1<'static, f64> = Array1::with_bounds(1, 2).unwrap();
    knots[2] = 1.0;
    let mut mults: Array1<'static, i32> = Array1::with_bounds(1, 2).unwrap();
    mults.init(degree + 1);
    let curve: Handle<dyn GeomCurve> =
        Handle::new(GeomBSplineCurve::new(&p, &knots, &mults, degree, false).unwrap()).cast();
    Handle::new(GeomAdaptorCurve::with_curve(&curve).unwrap()).cast()
}

fn frame_at(law: &GeomFillCorrectedFrenet, param: f64) -> TrihedronD0 {
    let mut frame = TrihedronD0::default();
    law.d0(param, &mut frame).unwrap();
    frame
}

fn law_on(poles: &[Point3], degree: i32) -> GeomFillCorrectedFrenet {
    let law = GeomFillCorrectedFrenet::with_evaluation(false);
    law.set_curve(&bspline_adaptor(poles, degree)).unwrap();
    law
}

#[test]
fn GeomFill_CorrectedFrenet__EndlessLoopPrevention() {
    let poles = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(1.0, 0.0, 0.0),
        Point3::new(1.0, 1.0, 0.0),
        Point3::new(0.0, 1.0, 0.0),
    ];
    let law = law_on(&poles, 3);
    for param in [0.0, 1.0] {
        let f = frame_at(&law, param);
        assert!(f.tangent.magnitude() > 1e-10);
        assert!(f.normal.magnitude() > 1e-10);
        assert!(f.binormal.magnitude() > 1e-10);
    }
}

#[test]
fn GeomFill_CorrectedFrenet__SmallStepHandling() {
    let poles = [Point3::new(0.0, 0.0, 0.0), Point3::new(1e-10, 0.0, 0.0)];
    let law = law_on(&poles, 1);
    let mut frame = TrihedronD0::default();
    assert!(law.d0(0.5, &mut frame).is_ok());
}

#[test]
fn GeomFill_CorrectedFrenet__ParameterProgressionGuarantee() {
    let poles = [
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(0.5, 0.5, 0.0),
        Point3::new(1.0, 0.0, 0.0),
    ];
    let law = law_on(&poles, 2);
    let mut param = 0.1;
    while param <= 0.9 {
        let f = frame_at(&law, param);
        assert!(f.tangent.magnitude() > 1e-12);
        assert!(f.normal.magnitude() > 1e-12);
        assert!(f.binormal.magnitude() > 1e-12);
        param += 0.1;
    }
}

/// Preserve the actual segment/make-edge/wire-builder path of the loop reproducer.
fn corrected_frenet_reproducer_wire() -> rustycad_tkbrep::topods::Wire {
    let points = [
        Point3::new(-1.0, -1.0, 0.0),
        Point3::new(0.0, -2.0, 0.0),
        Point3::new(0.0, -2.0, -1.0),
        Point3::new(0.0, -1.0, -1.0),
    ];
    let extend = rustycad_tkshhealing::shape_extend::ShapeExtendWireData::new();
    for ends in points.windows(2) {
        let curve = rustycad_tkgeombase::gc::MakeSegment::from_points(ends[0], ends[1])
            .unwrap()
            .value()
            .unwrap();
        let edge =
            rustycad_tktopalgo::brep_builder_api::BRepBuilderAPIMakeEdge::from_curve(&curve.cast())
                .unwrap()
                .edge()
                .unwrap();
        extend.add_edge(&edge, 0).unwrap();
    }
    extend.wire_api_make().unwrap()
}
#[test]
fn GeomFill_CorrectedFrenet__ActualReproducerCase() {
    let adaptor = rustycad_tkbrep::brep_adaptor::BRepAdaptorCompCurve::with_wire(
        &corrected_frenet_reproducer_wire(),
        false,
    )
    .unwrap();
    let law = GeomFillCorrectedFrenet::with_evaluation(false);
    // This SetCurve call caused the original endless loop.
    law.set_curve(&adaptor.shallow_copy().unwrap()).unwrap();
    let mut frame = TrihedronD0::default();
    law.d0(0.0, &mut frame).unwrap();
    law.d0(0.5, &mut frame).unwrap();
    law.d0(1.0, &mut frame).unwrap();
}
