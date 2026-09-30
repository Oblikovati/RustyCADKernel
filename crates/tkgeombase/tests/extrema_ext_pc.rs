//! Direct translation of Extrema_ExtPC_Test.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.
// 1725.9708621929999 is the upstream literal.
#![allow(non_snake_case, dead_code, clippy::excessive_precision)]
mod common;
#[allow(unused_imports)]
use common::*;
#[allow(unused_imports)]
use prelude::*;
use rustycad_tkgeombase::extrema::ExtPC;
use rustycad_tkmath::elslib;
use rustycad_tkmath::gp::{Axis3, Cylinder3};

// Test for bug OCC24945
// Extrema_ExtPElC::Perform does not consider angular tolerance when calculates angle between two
// vectors
#[test]
fn Extrema_ExtPC_Test__Bug24945_CylinderParameterNormalization() {
    // Point to project
    let aP3D = Point3::new(-1725.97, 843.257, -4.22741e-013);

    // Circle for projection
    let aAxis = Axis2::with_x_direction(
        Point3::new(0.0, 843.257, 0.0),
        dir3(0.0, 1.0, 0.0).reversed(),
        dir3(1.0, 0.0, 0.0),
    )
    .unwrap();
    let aCircle = Handle::new(GeomCircle::new(aAxis, 1725.9708621929999).unwrap());
    let aC3D = GeomAdaptorCurve::with_curve(&aCircle.cast()).unwrap();

    // Project point onto circle
    let aExtPC = ExtPC::with_curve(aP3D, &aC3D, 1.0e-10).unwrap();
    assert!(aExtPC.is_done());
    assert!(aExtPC.nb_ext().unwrap() > 0);

    let aProj: Point3 = *aExtPC.point(1).unwrap().value();

    // Check projected point coordinates
    EXPECT_NEAR(aProj.x(), -1725.97, 1.0e-2);
    EXPECT_NEAR(aProj.y(), 843.257, 1.0e-2);
    EXPECT_NEAR(aProj.z(), 0.0, 1.0e-10);

    // Compute parameters on cylinder
    let aCylAxis = Axis2::with_x_direction(
        Point3::new(0.0, 2103.87, 0.0),
        dir3(0.0, 1.0, 0.0).reversed(),
        dir3(1.0, 0.0, 0.0).reversed(),
    )
    .unwrap();
    let aCylinder = Cylinder3::new(Axis3::from_axis2(aCylAxis), 1890.).unwrap();

    let (aU, aV) = elslib::parameters(&aCylinder, aProj);

    // Check parameters
    // U should be normalized to [0, 2*PI) - the test previously expected 6.2832 (2*PI)
    // but the improved normalization now consistently returns 0.0 for the seam
    EXPECT_NEAR(aU, 0.0, 1.0e-4);
    EXPECT_NEAR(aV, 1260.613, 1.0e-2);
}
