//! Direct translation of ModelingData/TKG3d/GTests/GeomAPI_Interpolate_Test.cxx at
//! 3d097a0328e71b826377d4814ab05ec3c3d23871. The suite belongs to the TKG3d test
//! directory but tests GeomAPI_Interpolate (TKGeomAlgo), so it runs in this crate.
#![allow(non_snake_case)]
use rustycad_tkernel::ncollection::HArray1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::GeomCurve;
use rustycad_tkgeomalgo::geom_api::Interpolate;
use rustycad_tkmath::gp::Point3;

// Test BUC60902: GeomAPI_Interpolate tangent computation
#[test]
fn GeomAPI_Interpolate_Test__BUC60902_TangentPreservation() {
    // Create 5 points along a sine wave
    let aPnts: Handle<HArray1<Point3>> = Handle::new(HArray1::with_bounds(1, 5).unwrap());
    let mut aP = Point3::new(0.0, 0.0, 0.0);
    for i in 1..=5 {
        aP.set_x(f64::from(i - 1) * 1.57);
        aP.set_y((f64::from(i - 1) * 1.57).sin());
        aPnts
            .get()
            .unwrap()
            .change_array1()
            .set_value(i, aP)
            .unwrap();
    }

    // First interpolation without tangents
    let mut anInterpolater = Interpolate::new(&aPnts, false, precision::confusion()).unwrap();
    anInterpolater.perform().unwrap();
    assert!(
        anInterpolater.is_done(),
        "First interpolation should succeed"
    );

    // Get the generated tangents
    let aCur = anInterpolater.curve().unwrap();
    assert!(!aCur.is_null(), "Interpolated curve should not be null");

    let (_, aFirstTang) = aCur.d1(aCur.first_parameter()).unwrap();
    let (_, aLastTang) = aCur.d1(aCur.last_parameter()).unwrap();

    // Second interpolation with the same tangents explicitly provided
    let mut anInterpolater1 = Interpolate::new(&aPnts, false, precision::confusion()).unwrap();
    anInterpolater1
        .load_ends(aFirstTang, aLastTang, false)
        .unwrap();
    anInterpolater1.perform().unwrap();
    assert!(
        anInterpolater1.is_done(),
        "Second interpolation should succeed"
    );

    // Get the tangents after recomputation
    let aCur = anInterpolater1.curve().unwrap();
    assert!(
        !aCur.is_null(),
        "Second interpolated curve should not be null"
    );

    let (_, aFirstTang1) = aCur.d1(aCur.first_parameter()).unwrap();
    let (_, aLastTang1) = aCur.d1(aCur.last_parameter()).unwrap();

    // Verify that the tangents are preserved
    assert!(
        aFirstTang
            .is_equal(aFirstTang1, precision::confusion(), precision::angular())
            .unwrap(),
        "First tangent should be preserved after recomputation"
    );

    assert!(
        aLastTang
            .is_equal(aLastTang1, precision::confusion(), precision::angular())
            .unwrap(),
        "Last tangent should be preserved after recomputation"
    );
}
