//! Behavior tests of the native quirks the TKHelix port keeps (docs/tkhelix-progress.md);
//! the numeric oracle compares the same cases with OCCT bit for bit.
mod support;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkhelix::helix_brep::HelixBRepBuilderHelix;
use rustycad_tkhelix::helix_geom::{
    HelixGeomBuilderApproxCurve, HelixGeomBuilderHelix, HelixGeomBuilderHelixGen,
    HelixGeomHelixCurve,
};
use rustycad_tkmath::geom_abs::Shape;
use std::f64::consts::PI;
use support::brep::{array, my_axis};
use support::expect_throw;

#[test]
fn reload_with_a_tiny_taper_keeps_the_previous_taper() {
    let helix = HelixGeomHelixCurve::new();
    helix.load(0.0, 2.0 * PI, 5.0, 2.0, 0.1, true).unwrap();
    helix.load(0.0, 2.0 * PI, 5.0, 2.0, 0.0, true).unwrap();
    // A fresh untapered helix ends at radius 2; the reloaded one is still tapered.
    let end = helix.value(2.0 * PI).unwrap();
    assert!(end.x() > 2.0 + 0.1);
}

#[test]
fn a_failed_load_keeps_the_invalid_parameters() {
    let helix = HelixGeomHelixCurve::new();
    expect_throw(
        helix.load(3.0, 1.0, 5.0, 2.0, 0.0, true),
        "Standard_ConstructionError",
    );
    assert_eq!(helix.first_parameter().unwrap(), 3.0);
    expect_throw(helix.load_current(), "Standard_ConstructionError");
}

#[test]
fn intervals_write_absolute_indices() {
    let helix = HelixGeomHelixCurve::new();
    let mut short: Array1<'static, f64> = Array1::with_bounds(2, 3).unwrap();
    expect_throw(
        helix.intervals(&mut short, Shape::C0),
        "Standard_OutOfRange",
    );
    let mut wide: Array1<'static, f64> = Array1::with_bounds(0, 2).unwrap();
    helix.intervals(&mut wide, Shape::C0).unwrap();
    assert_eq!(wide.as_slice(), &[0.0, 0.0, 2.0 * PI]);
}

#[test]
fn a_nan_range_counts_int_min_turns_and_builds_nothing() {
    let mut builder = HelixGeomBuilderHelix::new();
    builder.set_curve_parameters(0.0, f64::NAN, 1.0, 1.0, 0.0, true);
    builder.perform().unwrap();
    assert_eq!(builder.error_status(), 0);
    assert_eq!(builder.curves().length(), 0);
    assert_eq!(builder.tolerance_reached(), -1.0);
}

#[test]
fn a_range_reversed_by_more_than_a_turn_raises_from_the_rest_coil() {
    let mut builder = HelixGeomBuilderHelix::new();
    builder.set_curve_parameters(5.0, -10.0, 3.0, 2.0, 0.0, true);
    expect_throw(builder.perform(), "Standard_ConstructionError");
}

#[test]
fn a_helix_without_part_raises_no_such_object() {
    let mut builder = HelixBRepBuilderHelix::new();
    let empty: Array1<'static, f64> = array(&[]);
    let no_flags: Array1<'static, bool> = array(&[]);
    builder
        .set_parameters(&my_axis(), &array(&[100.0]), &empty, &empty, &no_flags)
        .unwrap();
    expect_throw(builder.perform(), "Standard_NoSuchObject");
}

#[test]
fn arrays_keep_their_bounds_and_perform_reads_absolute_indices() {
    let mut builder = HelixBRepBuilderHelix::new();
    let mut heights: Array1<'static, f64> = Array1::with_bounds(2, 2).unwrap();
    heights.init(50.0);
    builder
        .set_parameters_diam(
            &my_axis(),
            100.0,
            &heights,
            &array(&[10.0]),
            &array(&[true]),
        )
        .unwrap();
    expect_throw(builder.perform(), "Standard_OutOfRange");
}

#[test]
fn zero_turns_raise_from_the_coil_load() {
    let mut builder = HelixBRepBuilderHelix::new();
    builder
        .set_parameters_diam(
            &my_axis(),
            100.0,
            &array(&[100.0]),
            &array(&[0.0]),
            &array(&[false]),
        )
        .unwrap();
    expect_throw(builder.perform(), "Standard_ConstructionError");
}

#[test]
fn wrong_dimensions_leave_the_builder_without_parameters() {
    let mut builder = HelixBRepBuilderHelix::new();
    let result = builder.set_parameters(
        &my_axis(),
        &array(&[100.0, 80.0, 60.0]),
        &array(&[30.0]),
        &array(&[10.0]),
        &array(&[true]),
    );
    expect_throw(result, "Standard_ConstructionError");
    assert_eq!((builder.error_status(), builder.warning_status()), (1, 1));
    builder.perform().unwrap();
    assert!(builder.shape().is_null());
}
