// Copyright (c) 1996-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

//! Behavior of the FairCurve port beyond the numeric probe, which compares the
//! densities, energies, Newton solves and computed curves with OCCT bit for bit.
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::Handle;
use rustycad_tkgeomalgo::fair_curve::*;
use rustycad_tkmath::gp::Point2;
use rustycad_tkmath::math::Vector;

fn origin() -> Point2 {
    Point2::new(0.0, 0.0)
}

fn unit() -> Point2 {
    Point2::new(1.0, 0.0)
}

#[test]
fn the_batten_law_is_linear_in_the_parameter() {
    let mut law = FairCurveBattenLaw::new(1.0, 0.5, 2.0);
    assert_eq!(law.height(0.0), (true, 0.5));
    assert_eq!(law.height(1.0), (true, 1.5));
    law.set_slope(-2.0);
    assert_eq!(law.height(1.0), (false, -1.0));
}

#[test]
fn a_batten_refuses_confused_points_and_a_null_height() {
    let confused = FairCurveBatten::new(origin(), origin(), 1.0, 0.0)
        .err()
        .unwrap();
    assert_eq!(confused.exception_type(), "Standard_NullValue");
    let flat = FairCurveBatten::new(origin(), unit(), 0.0, 0.0)
        .err()
        .unwrap();
    assert_eq!(flat.exception_type(), "Standard_NegativeValue");
    let mut batten = FairCurveBatten::new(origin(), unit(), 1.0, 0.0).unwrap();
    let error = batten.set_height(-1.0).unwrap_err();
    assert!(error.message().contains("-1"), "{}", error.message());
    assert_eq!(
        batten.set_p2(origin()).unwrap_err().exception_type(),
        "Standard_NullValue"
    );
}

#[test]
fn a_new_batten_is_the_segment_raised_to_degree_nine() {
    let batten = FairCurveBatten::new(origin(), Point2::new(3.0, 0.0), 1.0, 0.0).unwrap();
    let curve = batten.curve().unwrap();
    let curve = curve.get().unwrap();
    assert_eq!((curve.degree(), curve.nb_knots()), (9, 2));
    let poles = curve.poles();
    for i in 1..=10 {
        assert!((poles[i].x() - f64::from(i - 1) / 3.0).abs() <= 1e-15);
        assert_eq!(poles[i].y(), 0.0);
    }
}

#[test]
fn moving_an_end_point_turns_the_requested_angles_with_the_chord() {
    let mut batten = FairCurveBatten::new(origin(), unit(), 1.0, 0.0).unwrap();
    batten.set_p2(Point2::new(0.0, 1.0)).unwrap();
    let dump = dump_text(&batten);
    // The chord turned by +pi/2: Angle1 -= pi/2 and Angle2 += pi/2.
    assert!(dump.contains("      Angle1   |      0 | -1.5708"), "{dump}");
    assert!(dump.contains("      Angle2   |      0 | 1.5708"), "{dump}");
}

fn dump_text(batten: &FairCurveBatten) -> String {
    let mut text: Vec<u8> = Vec::new();
    batten.dump(&mut text).unwrap();
    String::from_utf8(text).unwrap()
}

#[test]
fn a_computed_batten_keeps_its_ends_and_constrained_angles() {
    let mut batten = FairCurveBatten::new(origin(), unit(), 1.0, 0.0).unwrap();
    batten.set_angle1(-0.4);
    batten.set_angle2(0.3);
    let mut code = FairCurveAnalysisCode::NullHeight;
    assert!(batten.compute(&mut code, 50, 1.0e-3).unwrap());
    assert_eq!(code, FairCurveAnalysisCode::Ok);
    assert_eq!((batten.get_angle1(), batten.get_angle2()), (-0.4, 0.3));
    let curve = batten.curve().unwrap();
    let poles = curve.get().unwrap().poles();
    assert!(poles[1].distance(origin()) <= 1e-12);
    assert!(poles[poles.upper()].distance(unit()) <= 1e-12);
    // The start tangent makes the requested angle with the chord.
    let tangent = (poles[2].y() - poles[1].y()).atan2(poles[2].x() - poles[1].x());
    assert!((tangent - -0.4).abs() <= 1e-9, "{tangent}");
    assert!(dump_text(&batten).ends_with("AnalysisCode : Ok\n"));
}

#[test]
fn a_steep_slope_leaves_no_matter_at_an_end() {
    let mut batten = FairCurveBatten::new(origin(), unit(), 0.1, 3.0).unwrap();
    batten.set_angle1(0.3);
    batten.set_angle2(0.3);
    let mut code = FairCurveAnalysisCode::Ok;
    assert!(batten.compute(&mut code, 50, 1.0e-3).unwrap());
    assert_eq!(code, FairCurveAnalysisCode::NullHeight);
}

#[test]
fn the_physical_ratio_must_lie_in_the_unit_interval() {
    let mut mvc = FairCurveMinimalVariation::new(origin(), unit(), 1.0, 0.0, 1.5).unwrap();
    let error = mvc.set_physical_ratio(-0.1).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_DomainError");
    assert_eq!(mvc.get_physical_ratio(), 1.5);
    // The constructor does not check; the minimal-variation energy refuses 1.5.
    let mut code = FairCurveAnalysisCode::Ok;
    let error = mvc.compute(&mut code, 50, 1.0e-3).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_DomainError");
    // NaN passes the native test.
    let mut other = FairCurveMinimalVariation::new(origin(), unit(), 1.0, 0.0, 0.0).unwrap();
    assert!(other.set_physical_ratio(f64::NAN).is_ok());
}

fn bezier_poles() -> (Handle<HArray1<f64>>, Handle<HArray1<Point2>>) {
    let mut flat: Array1<'static, f64> = Array1::with_bounds(1, 20).unwrap();
    let mut poles: Array1<'static, Point2> = Array1::with_bounds(1, 10).unwrap();
    for i in 1..=10 {
        (flat[i], flat[i + 10]) = (0.0, 1.0);
        poles[i] = Point2::new(f64::from(i - 1) / 3.0, 0.1 * f64::from((i - 1) % 3));
    }
    (
        Handle::new(HArray1::from_owned(flat)),
        Handle::new(HArray1::from_owned(poles)),
    )
}

#[test]
fn a_density_counts_the_value_gradient_and_packed_hessian() {
    let (flat, poles) = bezier_poles();
    let law = FairCurveBattenLaw::new(1.0, 0.0, 3.0);
    let mut tension =
        FairCurveDistributionOfTension::new(10, &flat, &poles, 0, 3.0, &law, 1, false);
    assert_eq!((tension.nb_variables(), tension.nb_equations()), (1, 1));
    tension.set_derivative_order(1);
    assert_eq!(tension.nb_equations(), 1 + 21);
    tension.set_derivative_order(2);
    assert_eq!(tension.nb_equations(), 22 + 21 * 22 / 2);
}

#[test]
fn the_energy_variables_round_trip_through_the_poles() {
    let (flat, poles) = bezier_poles();
    let law = FairCurveBattenLaw::new(1.0, 0.0, 3.0);
    let mut energy =
        FairCurveEnergyOfBatten::new(10, &flat, &poles, [0, 0], &law, 3.0, true, [0.0, 0.0]);
    let n = FairCurveEnergyModel::nb_variables(&energy);
    assert_eq!(n, 2 * 8 + 1);
    let mut x = Vector::filled(1, n, 0.0).unwrap();
    assert!(FairCurveEnergyModel::variable(&energy, &mut x));
    assert_eq!(x[n], 3.0);
    let snapshot = || -> Vec<Point2> { poles.get().unwrap().array1().iter().copied().collect() };
    let before = snapshot();
    let mut e = 0.0;
    assert!(FairCurveEnergyModel::value(&mut energy, &x, &mut e));
    assert!(e > 0.0);
    assert_eq!(snapshot(), before);
    // Beyond ten times the original sliding the energy refuses to evaluate.
    x[n] = 31.0;
    assert!(!FairCurveEnergyModel::value(&mut energy, &x, &mut e));
    assert_eq!(energy.status(), FairCurveAnalysisCode::InfiniteSliding);
}
