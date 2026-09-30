//! 1:1 translation of src/ModelingAlgorithms/TKHelix/GTests/HelixBRep_BuilderHelix_Integration_Test.cxx.
#![allow(non_snake_case)]

mod support;
use rustycad_tkbrep::topods::{self, Shape, Wire};
use rustycad_tkg3d::top_abs::ShapeEnum;
use rustycad_tkhelix::helix_brep::HelixBRepBuilderHelix;
use rustycad_tkmath::geom_abs::Shape as Continuity;
use rustycad_tktopalgo::brep_check::BRepCheckAnalyzer;
use std::f64::consts::PI;
use support::brep::{array, compute_wire_length, count_shapes, my_axis};
use support::{expect_near, expect_throw};

/// The fixture `HelixBRepTest`: `myTolerance = 1.e-4` (and `myAxis`, see `my_axis`).
const MY_TOLERANCE: f64 = 1.0e-4;

/// `ValidateHelixWire(theWire, theExpectedLength)` with the default tolerance 0.05.
fn validate_helix_wire(the_wire: &Wire, the_expected_length: f64) {
    let the_tolerance = 0.05;
    assert!(!the_wire.is_null());
    assert_eq!(the_wire.shape_type().unwrap(), ShapeEnum::Wire);

    // Check that wire has edges
    let a_nb_edges = count_shapes(the_wire, ShapeEnum::Edge);
    assert!(a_nb_edges > 0);

    // Compute wire length
    let a_wire_length = compute_wire_length(the_wire);
    if the_expected_length > 0.0 {
        expect_near(
            a_wire_length,
            the_expected_length,
            the_expected_length * the_tolerance,
        );
    }
}

/// `TopoDS::Wire(theShape)`.
fn as_wire(shape: &Shape) -> Wire {
    topods::wire(shape).unwrap()
}

#[test]
fn HelixBRepTest__TCL_Test_A1_PureCylindricalHelix() {
    let mut a_builder = HelixBRepBuilderHelix::new();

    // Parameters from TCL test A1: helix, D1 = 100, 1 part, pitch = 5, number of turns, PFi=0
    let a_heights = array(&[100.0]); // H1 = 100
    let a_pitches = array(&[5.0]); // P1 = 5 (number of turns)
    let a_is_pitches = array(&[false]); // PF1 = 0 means number of turns

    a_builder
        .set_parameters_diam(&my_axis(), 100.0, &a_heights, &a_pitches, &a_is_pitches)
        .unwrap();
    a_builder.set_approx_parameters(MY_TOLERANCE, 8, Continuity::C1);

    a_builder.perform().unwrap();

    assert_eq!(a_builder.error_status(), 0);

    let a_helix_wire = a_builder.shape();
    assert!(!a_helix_wire.is_null());
    assert_eq!(a_helix_wire.shape_type().unwrap(), ShapeEnum::Wire);

    // Calculate expected helix length
    let a_circumference = PI * 100.0; // D1 = 100
    let a_turns = 100.0 / 20.0; // Height/Pitch for number of turns mode (20 = 100/5)
    let a_expected_length = a_turns * (a_circumference * a_circumference + 20.0 * 20.0).sqrt();

    validate_helix_wire(&as_wire(a_helix_wire), a_expected_length);
}

#[test]
fn HelixBRepTest__TCL_Test_B1_CompositeCylindricalHelix() {
    let mut a_builder = HelixBRepBuilderHelix::new();

    // Parameters from TCL test B1: comphelix, D1 = D2 = 100, 1 part, pitch = 20, PFi=1
    let a_diams = array(&[100.0, 100.0]); // D1, D2
    let a_heights = array(&[100.0]); // H1
    let a_pitches = array(&[20.0]); // P1
    let a_is_pitches = array(&[true]); // PF1 = 1 means pitch

    a_builder
        .set_parameters(&my_axis(), &a_diams, &a_heights, &a_pitches, &a_is_pitches)
        .unwrap();
    a_builder.set_approx_parameters(MY_TOLERANCE, 8, Continuity::C1);

    a_builder.perform().unwrap();

    assert_eq!(a_builder.error_status(), 0);

    let a_helix_wire = a_builder.shape();
    assert!(!a_helix_wire.is_null());

    // Calculate expected helix length - same as A1 since geometry is identical
    let a_circumference = PI * 100.0;
    let a_turns = 100.0 / 20.0; // Height/Pitch
    let a_expected_length = a_turns * (a_circumference * a_circumference + 20.0 * 20.0).sqrt();

    validate_helix_wire(&as_wire(a_helix_wire), a_expected_length);
}

#[test]
fn HelixBRepTest__TCL_Test_C1_SpiralHelix() {
    let mut a_builder = HelixBRepBuilderHelix::new();

    // Parameters from TCL test C1: spiral, D1 = 100, D2 = 20, 3 parts
    let a_heights = array(&[20.0, 60.0, 20.0]); // H1, H2, H3
    let a_pitches = array(&[2.0, 6.0, 2.0]); // P1, P2, P3 (number of turns)
    let a_is_pitches = array(&[false, false, false]); // PF1 = PF2 = PF3 = 0

    a_builder
        .set_parameters_spiral(
            &my_axis(),
            100.0,
            20.0,
            &a_heights,
            &a_pitches,
            &a_is_pitches,
        )
        .unwrap();
    a_builder.set_approx_parameters(MY_TOLERANCE, 8, Continuity::C1);

    a_builder.perform().unwrap();

    assert_eq!(a_builder.error_status(), 0);

    let a_helix_wire = a_builder.shape();
    assert!(!a_helix_wire.is_null());

    // For spiral helix, length calculation is more complex due to changing diameter
    // Just validate basic properties
    validate_helix_wire(&as_wire(a_helix_wire), 0.0); // Don't check exact length for complex spiral
}

#[test]
fn HelixBRepTest__TCL_Test_F1_Helix2Interface() {
    let mut a_builder = HelixBRepBuilderHelix::new();

    // Parameters from TCL test F1: comphelix2 with turns interface
    let a_pitches = array(&[10.0, 10.0, 10.0]); // P1, P2, P3
    let a_nb_turns = array(&[2.0, 6.0, 2.0]); // N1, N2, N3

    a_builder
        .set_parameters_diam_turns(&my_axis(), 100.0, &a_pitches, &a_nb_turns)
        .unwrap();
    a_builder.set_approx_parameters(MY_TOLERANCE, 8, Continuity::C1);

    a_builder.perform().unwrap();

    assert_eq!(a_builder.error_status(), 0);

    let a_helix_wire = a_builder.shape();
    assert!(!a_helix_wire.is_null());

    // Calculate expected length for cylindrical helix with multiple parts
    let a_circumference = PI * 100.0;
    let a_total_turns = 2.0 + 6.0 + 2.0; // Sum of turns
    let a_expected_length =
        a_total_turns * (a_circumference * a_circumference + 10.0 * 10.0).sqrt();

    validate_helix_wire(&as_wire(a_helix_wire), a_expected_length);
}

#[test]
fn HelixBRepTest__TCL_Test_E1_CustomerExample() {
    let mut a_builder = HelixBRepBuilderHelix::new();

    // Parameters from TCL test E1: Customer example with 5 parts
    let a_diams = array(&[150.0, 150.0, 150.0, 123.0, 123.0, 123.0]); // D1..D6

    // Calculate heights from pitches and turns
    let a_heights = array(&[
        0.75 * 13.0, // N1 * P1
        2.1 * 64.0,  // N2 * P2
        2.25 * 50.0, // N3 * P3
        2.5 * 45.0,  // N4 * P4
        0.75 * 13.0, // N5 * P5
    ]);

    let a_pitches = array(&[0.75, 2.1, 2.25, 2.5, 0.75]); // N1..N5 (number of turns)
    let a_is_pitches = array(&[false, false, false, false, false]); // Number of turns mode

    a_builder
        .set_parameters(&my_axis(), &a_diams, &a_heights, &a_pitches, &a_is_pitches)
        .unwrap();
    a_builder.set_approx_parameters(MY_TOLERANCE, 8, Continuity::C1);

    a_builder.perform().unwrap();

    assert_eq!(a_builder.error_status(), 0);

    let a_helix_wire = a_builder.shape();
    assert!(!a_helix_wire.is_null());

    // Complex customer example - just validate basic properties
    validate_helix_wire(&as_wire(a_helix_wire), 0.0); // Don't check exact length for complex multi-part helix
}

#[test]
fn HelixBRepTest__ErrorConditions_InvalidDimensions() {
    let mut a_builder = HelixBRepBuilderHelix::new();

    // Test mismatched array dimensions
    let a_diams = array(&[100.0, 100.0, 100.0]); // 3 elements = 2 parts
    let a_heights = array(&[50.0]); // 1 element - should cause error (need 2 for 2 parts)
    let a_pitches = array(&[10.0]); // 1 element - should cause error (need 2 for 2 parts)
    let a_is_pitches = array(&[true]); // 1 element - should cause error (need 2 for 2 parts)

    expect_throw(
        a_builder.set_parameters(&my_axis(), &a_diams, &a_heights, &a_pitches, &a_is_pitches),
        "Standard_ConstructionError",
    );
}

#[test]
fn HelixBRepTest__ErrorConditions_TurnsInterface() {
    let mut a_builder = HelixBRepBuilderHelix::new();

    // Test mismatched dimensions in turns interface
    let a_pitches = array(&[10.0, 10.0]);
    let a_nb_turns = array(&[5.0]); // Mismatched size

    expect_throw(
        a_builder.set_parameters_diam_turns(&my_axis(), 100.0, &a_pitches, &a_nb_turns),
        "Standard_ConstructionError",
    );
}

#[test]
fn HelixBRepTest__ApproximationQuality() {
    let mut a_builder = HelixBRepBuilderHelix::new();

    let a_heights = array(&[50.0]);
    let a_pitches = array(&[10.0]);
    let a_is_pitches = array(&[true]);

    a_builder
        .set_parameters_diam(&my_axis(), 50.0, &a_heights, &a_pitches, &a_is_pitches)
        .unwrap();

    // Test different tolerance levels
    let a_tight_tolerance = 1.0e-6;
    a_builder.set_approx_parameters(a_tight_tolerance, 8, Continuity::C2);

    a_builder.perform().unwrap();

    assert_eq!(a_builder.error_status(), 0);

    let a_tolerance_reached = a_builder.tolerance_reached();
    assert!(a_tolerance_reached > 0.0);

    // For complex geometry, tolerance reached might be higher than requested
    // but should be reasonable
    assert!(a_tolerance_reached < a_tight_tolerance * 1000.0);
}

#[test]
fn HelixBRepTest__ParameterValidation() {
    let mut a_builder = HelixBRepBuilderHelix::new();

    // Test very small height - should cause error during Perform()
    let mut a_heights = array(&[1.0e-8]);
    let mut a_pitches = array(&[10.0]);
    let a_is_pitches = array(&[true]);

    a_builder
        .set_parameters_diam(&my_axis(), 50.0, &a_heights, &a_pitches, &a_is_pitches)
        .unwrap();
    a_builder.set_approx_parameters(MY_TOLERANCE, 8, Continuity::C1);

    a_builder.perform().unwrap();

    assert_ne!(a_builder.error_status(), 0); // Should have error status 12 (invalid height)

    // Test zero pitch - should cause error
    *a_heights.change_value(1).unwrap() = 50.0;
    *a_pitches.change_value(1).unwrap() = 0.0;

    a_builder
        .set_parameters_diam(&my_axis(), 50.0, &a_heights, &a_pitches, &a_is_pitches)
        .unwrap();

    a_builder.perform().unwrap();

    assert_ne!(a_builder.error_status(), 0); // Should have error status 11 (invalid pitch)
}

#[test]
fn HelixBRepTest__MultiPartContinuity() {
    let mut a_builder = HelixBRepBuilderHelix::new();

    // 2-part helix with different parameters
    let a_diams = array(&[80.0, 60.0, 40.0]);
    let a_heights = array(&[40.0, 40.0]);
    let a_pitches = array(&[8.0, 12.0]);
    let a_is_pitches = array(&[true, true]);

    a_builder
        .set_parameters(&my_axis(), &a_diams, &a_heights, &a_pitches, &a_is_pitches)
        .unwrap();
    a_builder.set_approx_parameters(MY_TOLERANCE, 8, Continuity::C1);

    a_builder.perform().unwrap();

    assert_eq!(a_builder.error_status(), 0);

    let a_helix_wire = a_builder.shape();
    assert!(!a_helix_wire.is_null());
    assert_eq!(a_helix_wire.shape_type().unwrap(), ShapeEnum::Wire);

    // Check that wire has multiple edges
    let a_nb_edges = count_shapes(a_helix_wire, ShapeEnum::Edge);
    assert!(a_nb_edges > 1); // Should have multiple edges for multiple parts

    // Verify wire continuity by checking connectivity
    let a_wire: Shape = as_wire(a_helix_wire).into();
    let a_check = BRepCheckAnalyzer::new(&a_wire, true, false, false).unwrap();
    assert!(a_check.is_valid().unwrap());
}
