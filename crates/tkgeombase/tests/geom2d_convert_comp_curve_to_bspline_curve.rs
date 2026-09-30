//! Direct translation of Geom2dConvert_CompCurveToBSplineCurve_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
#![allow(non_snake_case)]
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBoundedCurve, Geom2dCircle, Geom2dCurve, Geom2dTrimmedCurve,
};
use rustycad_tkgeombase::gc::MakeCircle2d as GC_MakeCircle2d;
use rustycad_tkgeombase::geom2d_convert::CompCurveToBSplineCurve as Geom2dConvert_CompCurveToBSplineCurve;
use rustycad_tkmath::convert::ParameterisationType;
use rustycad_tkmath::gp::Point2;

fn trimmed(circle: &Handle<Geom2dCircle>, u1: f64, u2: f64) -> Handle<Geom2dTrimmedCurve> {
    let basis: Handle<dyn Geom2dCurve> = circle.cast();
    Handle::new(Geom2dTrimmedCurve::new(&basis, u1, u2, true, true).unwrap())
}

#[test]
fn Geom2dConvert_CompCurveToBSplineCurveTest__OCC30747_ClosedContourFromCircleArcs() {
    // OCC30747: 2d Curves concatenator must properly handle closed contours.
    // Split a full circle into 10 arcs and assemble them into a closed BSpline.
    let aCirc: Handle<Geom2dCircle> =
        Handle::try_from(&GC_MakeCircle2d::from_center(Point2::new(0.0, 0.0), 50.0, true).unwrap())
            .unwrap();
    assert!(!aCirc.is_null());

    let aF: f64 = aCirc.first_parameter();
    let aL: f64 = aCirc.last_parameter();
    let aNb: i32 = 10;
    let aDelta: f64 = (aF + aL) / f64::from(aNb);

    let aFTrim = trimmed(&aCirc, aF, aDelta);
    let mut aRes = Geom2dConvert_CompCurveToBSplineCurve::with_curve(
        &aFTrim.cast::<dyn Geom2dBoundedCurve>(),
        ParameterisationType::TgtThetaOver2,
    )
    .unwrap();

    for anId in 1..aNb {
        let aLTrim: Handle<Geom2dTrimmedCurve> = if anId == (aNb - 1) {
            trimmed(&aCirc, f64::from(anId) * aDelta, aF)
        } else {
            trimmed(
                &aCirc,
                f64::from(anId) * aDelta,
                f64::from(anId + 1) * aDelta,
            )
        };
        aRes.add(
            &aLTrim.cast::<dyn Geom2dBoundedCurve>(),
            precision::p_confusion(),
            false,
        )
        .unwrap();
    }

    let aBSpline: Handle<Geom2dBSplineCurve> = aRes.bspline_curve();
    assert!(!aBSpline.is_null());
    assert!(
        aBSpline.is_closed().unwrap(),
        "Assembled BSpline curve from closed circle arcs must be closed"
    );
}
