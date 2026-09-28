// Copyright (c) 2025 OPEN CASCADE SAS
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

//! Geom2d_BSplineCurve edition, periodicity, location and access members.
mod common;
use common::EXPECT_NEAR;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dBSplineCurve, Geom2dBoundedCurve, Geom2dCurve, Geom2dGeometry};
use rustycad_tkmath::geom_abs::{BSplKnotDistribution, Shape};
use rustycad_tkmath::gp::{Point2, Transform2, Vector2};

fn reals(values: &[f64]) -> Array1<'static, f64> {
    let mut array = Array1::<f64>::with_bounds(1, values.len() as i32).unwrap();
    array.as_mut_slice().copy_from_slice(values);
    array
}

fn ints(values: &[i32]) -> Array1<'static, i32> {
    let mut array = Array1::<i32>::with_bounds(1, values.len() as i32).unwrap();
    array.as_mut_slice().copy_from_slice(values);
    array
}

fn points(count: usize) -> Array1<'static, Point2> {
    let mut array = Array1::<Point2>::with_bounds(1, count as i32).unwrap();
    for i in 0..count {
        array[i as i32 + 1] = Point2::new(i as f64, (i % 2) as f64 + 0.25 * (i * i) as f64);
    }
    array
}

fn curve(knots: &[f64], mults: &[i32], degree: i32, periodic: bool) -> Handle<Geom2dBSplineCurve> {
    let sum: i32 = mults.iter().sum();
    let count = if periodic {
        sum - mults[mults.len() - 1]
    } else {
        sum - degree - 1
    };
    let made = Geom2dBSplineCurve::new(
        &points(count as usize),
        &reals(knots),
        &ints(mults),
        degree,
        periodic,
    );
    Handle::new(made.unwrap())
}

// A cubic Bezier segment written as a B-spline.
fn cubic() -> Handle<Geom2dBSplineCurve> {
    curve(&[0.0, 1.0], &[4, 4], 3, false)
}

fn uniform() -> Handle<Geom2dBSplineCurve> {
    curve(&[0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0], &[1; 8], 2, false)
}

fn periodic() -> Handle<Geom2dBSplineCurve> {
    curve(&[0.0, 1.0, 2.0, 3.0, 4.0, 5.0], &[1; 6], 3, true)
}

fn near(a: Point2, b: Point2) {
    assert!(a.distance(b) <= 1e-13, "{a:?} != {b:?}");
}

#[test]
fn knot_insertion_and_removal_keep_the_shape() {
    let c = cubic();
    let before = c.value(0.3).unwrap();
    c.insert_knot(0.5, 1, 0.0).unwrap();
    assert_eq!((c.nb_knots(), c.nb_poles()), (3, 5));
    c.increase_multiplicity(2, 2).unwrap();
    assert_eq!((c.multiplicity(2).unwrap(), c.nb_poles()), (2, 6));
    c.increment_multiplicity(2, 2, 1).unwrap();
    assert_eq!(c.multiplicity(2).unwrap(), 3);
    assert_eq!(c.continuity(), Shape::C0);
    c.increase_multiplicity_range(2, 2, 3).unwrap();
    assert_eq!(c.multiplicity(2).unwrap(), 3);
    near(c.value(0.3).unwrap(), before);
    assert!(c.remove_knot(2, 0, 1e-9).unwrap());
    assert_eq!((c.nb_knots(), c.nb_poles()), (2, 4));
    assert!(c.remove_knot(1, -1, 1e-9).unwrap());
    assert_eq!(
        c.remove_knot(9, 0, 1e-9).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    near(c.value(0.3).unwrap(), before);
    c.insert_knots(&reals(&[0.25, 0.75]), &ints(&[1, 2]), 0.0, false)
        .unwrap();
    assert_eq!(c.nb_knots(), 4);
    assert_eq!(
        c.insert_knot(2.0, 1, 0.0).unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
}

#[test]
fn degree_elevation_keeps_the_shape() {
    let c = cubic();
    let before = c.eval_d1(0.4).unwrap();
    c.increase_degree(5).unwrap();
    assert_eq!((c.degree(), c.nb_poles()), (5, 6));
    let after = c.eval_d1(0.4).unwrap();
    near(after.point, before.point);
    EXPECT_NEAR((after.d1 - before.d1).magnitude(), 0.0, 1e-12);
    c.increase_degree(5).unwrap();
    assert_eq!(
        c.increase_degree(2).unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
    assert_eq!(
        c.increase_degree(26).unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
}

#[test]
fn knot_setters_check_the_neighbours() {
    let c = uniform();
    c.set_knot(2, 1.5).unwrap();
    assert_eq!(c.knot(2).unwrap(), 1.5);
    assert_eq!(
        c.set_knot(2, 3.0).unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
    assert_eq!(
        c.set_knot(1, 1.5).unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
    assert_eq!(
        c.set_knot(8, 5.5).unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
    assert_eq!(
        c.set_knot(0, 0.5).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    let shifted: Vec<f64> = c.knots().iter().map(|k| k + 1.0).collect();
    c.set_knots(&reals(&shifted)).unwrap();
    assert_eq!(c.knot(1).unwrap(), 1.0);
    let b = cubic();
    b.insert_knot(0.5, 1, 0.0).unwrap();
    b.set_knot_with_multiplicity(2, 0.4, 2).unwrap();
    assert_eq!((b.knot(2).unwrap(), b.multiplicity(2).unwrap()), (0.4, 2));
}

#[test]
fn uniform_curves_accept_pole_insertion_and_removal() {
    let c = uniform();
    assert_eq!(c.knot_distribution(), BSplKnotDistribution::Uniform);
    c.insert_pole_after(2, Point2::new(9.0, 9.0), 1.0).unwrap();
    assert_eq!(
        (c.nb_poles(), c.nb_knots(), c.knot(9).unwrap()),
        (6, 9, 8.0)
    );
    assert!(!c.is_rational());
    assert_eq!(c.pole(3).unwrap(), Point2::new(9.0, 9.0));
    c.insert_pole_before(1, Point2::new(-1.0, 0.0), 2.0)
        .unwrap();
    assert!(c.is_rational());
    assert_eq!((c.weight(1).unwrap(), c.weight(2).unwrap()), (2.0, 1.0));
    c.remove_pole(1).unwrap();
    assert!(!c.is_rational());
    assert_eq!(c.nb_poles(), 6);
    assert_eq!(
        c.remove_pole(9).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    let error = c
        .insert_pole_after(1, Point2::new(0.0, 0.0), 0.0)
        .unwrap_err();
    assert_eq!(error.exception_type(), "Standard_ConstructionError");
    let bezier = cubic();
    assert_eq!(
        bezier.knot_distribution(),
        BSplKnotDistribution::PiecewiseBezier
    );
    let error = bezier
        .insert_pole_after(1, Point2::new(0.0, 0.0), 1.0)
        .unwrap_err();
    assert_eq!(error.exception_type(), "Standard_ConstructionError");
    assert_eq!(
        bezier.remove_pole(2).unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
}

#[test]
fn periodic_curves_normalize_and_repeat() {
    let c = periodic();
    assert!(c.is_periodic() && c.is_closed().unwrap());
    assert_eq!(
        (c.first_parameter(), c.last_parameter(), c.period().unwrap()),
        (0.0, 5.0, 5.0)
    );
    assert_eq!(c.periodic_normalization(7.0).unwrap(), 2.0);
    assert_eq!(c.periodic_normalization(-1.0).unwrap(), 4.0);
    near(c.value(6.5).unwrap(), c.value(1.5).unwrap());
    assert_eq!(
        (
            c.first_u_knot_index().unwrap(),
            c.last_u_knot_index().unwrap()
        ),
        (1, 6)
    );
    assert_eq!(c.reversed_parameter(1.0).unwrap(), 4.0);
    let before = c.value(2.5).unwrap();
    c.set_origin(3).unwrap();
    assert_eq!(c.knots().as_slice(), &[2.0, 3.0, 4.0, 5.0, 6.0, 7.0]);
    near(c.value(2.5).unwrap(), before);
    c.set_not_periodic().unwrap();
    assert!(!c.is_periodic());
    near(c.value(2.5).unwrap(), before);
    assert_eq!(
        c.set_origin(2).unwrap_err().exception_type(),
        "Standard_NoSuchObject"
    );
    assert_eq!(
        periodic().set_origin(9).unwrap_err().exception_type(),
        "Standard_DomainError"
    );
}

#[test]
fn set_periodic_keeps_the_leading_poles() {
    let c = curve(&[0.0, 1.0, 2.0], &[3, 1, 3], 2, false);
    let first = c.pole(1).unwrap();
    c.set_periodic().unwrap();
    // End multiplicities become min(Degree, max(3, 3)) = 2: 2 + 1 + 2 - 2 = 3 poles.
    assert!(c.is_periodic());
    assert_eq!(c.multiplicities().as_slice(), &[2, 1, 2]);
    assert_eq!((c.nb_poles(), c.pole(1).unwrap()), (3, first));
}

#[test]
fn segment_restricts_the_parameter_range() {
    let c = cubic();
    let middle = c.value(0.5).unwrap();
    c.segment(0.25, 0.75, 1e-9).unwrap();
    assert_eq!((c.first_parameter(), c.last_parameter()), (0.25, 0.75));
    near(c.value(0.5).unwrap(), middle);
    assert_eq!(
        c.segment(0.7, 0.3, 1e-9).unwrap_err().exception_type(),
        "Standard_DomainError"
    );
    let p = periodic();
    let inside = p.value(4.5).unwrap();
    p.segment(3.0, 6.0, 1e-9).unwrap();
    assert!(!p.is_periodic());
    assert_eq!((p.first_parameter(), p.last_parameter()), (3.0, 6.0));
    near(p.value(4.5).unwrap(), inside);
    assert_eq!(
        periodic()
            .segment(0.0, 6.0, 1e-9)
            .unwrap_err()
            .exception_type(),
        "Standard_DomainError"
    );
}

#[test]
fn poles_weights_and_moves() {
    let c = cubic();
    c.set_weight(2, 2.0).unwrap();
    assert!(c.is_rational() && c.weights().is_some());
    c.set_weight(2, 1.0).unwrap();
    assert!(!c.is_rational() && c.weights().is_none());
    assert_eq!(
        c.set_weight(2, 0.0).unwrap_err().exception_type(),
        "Standard_ConstructionError"
    );
    assert_eq!(
        c.set_weight(7, 1.0).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    c.set_pole_weighted(1, Point2::new(0.0, -1.0), 3.0).unwrap();
    assert_eq!(
        (c.start_point().unwrap(), c.weight(1).unwrap()),
        (Point2::new(0.0, -1.0), 3.0)
    );
    let target = Point2::new(1.5, 3.0);
    let (first, last) = c.move_point(0.5, target, 2, 3).unwrap();
    assert_eq!((first, last), (2, 3));
    near(c.value(0.5).unwrap(), target);
    let error = c.move_point(0.5, target, 3, 2).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
    let u = cubic();
    u.insert_knots(&reals(&[0.25, 0.5, 0.75]), &ints(&[1, 1, 1]), 0.0, false)
        .unwrap();
    let tangent = Vector2::new(1.0, 1.0);
    assert_eq!(
        u.move_point_and_tangent(0.5, target, tangent, 1e-9, 0, 0)
            .unwrap(),
        0
    );
    let moved = u.eval_d1(0.5).unwrap();
    near(moved.point, target);
    EXPECT_NEAR((moved.d1 - tangent).magnitude(), 0.0, 1e-9);
}

#[test]
fn locate_u_on_knots_and_flat_knots() {
    let c = cubic();
    c.insert_knot(0.5, 1, 0.0).unwrap();
    assert_eq!(c.locate_u(0.5, 1e-9, false).unwrap(), (2, 2));
    assert_eq!(c.locate_u(0.25, 1e-9, false).unwrap(), (1, 2));
    assert_eq!(c.locate_u(-1.0, 1e-9, false).unwrap(), (0, 1));
    assert_eq!(c.locate_u(2.0, 1e-9, false).unwrap(), (3, 4));
    assert_eq!(c.locate_u(1.0, 1e-9, false).unwrap(), (3, 3));
    assert_eq!(c.locate_u(0.0, 1e-9, true).unwrap(), (1, 1));
    // Flat knots are 0 0 0 0 0.5 1 1 1 1.
    assert_eq!(c.locate_u(0.5, 1e-9, true).unwrap(), (5, 5));
    assert_eq!(c.locate_u(0.75, 1e-9, true).unwrap(), (5, 6));
}

#[test]
fn local_evaluation_matches_global_evaluation_in_its_span() {
    let c = cubic();
    c.insert_knot(0.5, 1, 0.0).unwrap();
    let u = 0.3;
    assert_eq!(c.local_value(u, 1, 2).unwrap(), c.value(u).unwrap());
    assert_eq!(c.local_d0(u, 1, 2).unwrap(), c.eval_d0(u).unwrap());
    let d3 = c.eval_d3(u).unwrap();
    assert_eq!(c.local_d1(u, 1, 2).unwrap(), (d3.point, d3.d1));
    assert_eq!(c.local_d2(u, 1, 2).unwrap(), (d3.point, d3.d1, d3.d2));
    assert_eq!(
        c.local_d3(u, 1, 2).unwrap(),
        (d3.point, d3.d1, d3.d2, d3.d3)
    );
    assert_eq!(c.local_dn(u, 1, 2, 2).unwrap(), c.eval_dn(u, 2).unwrap());
    assert_eq!(
        c.local_d0(u, 1, 1).unwrap_err().exception_type(),
        "Standard_DomainError"
    );
    assert_eq!(
        c.eval_dn(u, 0).unwrap_err().exception_type(),
        "Geom2d_UndefinedDerivative"
    );
}

#[test]
fn continuity_and_g1_checks() {
    let c = curve(&[0.0, 0.5, 1.0], &[6, 1, 6], 5, false);
    // Degree 5 with an interior multiplicity 1: C3 by the switch default, C4 by IsCN.
    assert_eq!(c.continuity(), Shape::C3);
    assert!(c.is_cn(4).unwrap() && !c.is_cn(5).unwrap());
    assert_eq!(
        c.is_cn(-1).unwrap_err().exception_type(),
        "Standard_RangeError"
    );
    assert!(c.is_g1(0.0, 1.0, 1e-9).unwrap());
    let kinked = curve(&[0.0, 0.5, 1.0], &[3, 2, 3], 2, false);
    assert!(!kinked.is_g1(0.0, 1.0, 1e-9).unwrap());
    assert!(kinked.is_g1(0.6, 1.0, 1e-9).unwrap());
    let p = curve(&[0.0, 1.0, 2.0, 3.0], &[2, 2, 2, 2], 2, true);
    assert_eq!(p.continuity(), Shape::C0);
    assert!(!p.is_g1(-1.0, 4.0, 1e-9).unwrap());
}

#[test]
fn deprecated_copying_getters_check_their_destination() {
    let c = uniform();
    let mut knots = Array1::<f64>::with_bounds(2, 8).unwrap();
    c.knots_into(&mut knots).unwrap();
    assert_eq!(knots[2], 1.0);
    let mut outside = Array1::<f64>::with_bounds(0, 8).unwrap();
    assert_eq!(
        c.knots_into(&mut outside).unwrap_err().exception_type(),
        "Standard_DomainError"
    );
    let mut flat = Array1::<f64>::with_bounds(1, 8).unwrap();
    c.knot_sequence_into(&mut flat).unwrap();
    assert_eq!(flat.as_slice(), c.knot_sequence().as_slice());
    let mut mults = Array1::<i32>::with_bounds(1, 8).unwrap();
    c.multiplicities_into(&mut mults).unwrap();
    let mut short = Array1::<i32>::with_bounds(1, 7).unwrap();
    assert_eq!(
        c.multiplicities_into(&mut short)
            .unwrap_err()
            .exception_type(),
        "Standard_DimensionError"
    );
    let mut poles = Array1::<Point2>::with_bounds(1, 5).unwrap();
    c.poles_into(&mut poles).unwrap();
    assert_eq!(poles.as_slice(), c.poles().as_slice());
    let mut weights = Array1::<f64>::with_bounds(0, 4).unwrap();
    c.weights_into(&mut weights).unwrap();
    assert_eq!(weights.as_slice(), &[1.0; 5]);
    c.weights_array(|stored| assert_eq!(stored.len(), 5));
    assert_eq!(
        c.knot(0).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    assert_eq!(
        c.multiplicity(9).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    assert_eq!(
        c.pole(0).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
}

#[test]
fn resolution_reverse_transform_and_copy() {
    let line = curve(&[0.0, 1.0], &[2, 2], 1, false);
    // BSplCLib::Resolution bounds the derivative by the L1 norm of the pole differences
    // over the knot span: |1 - 0| + |1.25 - 0| on the unit span of this segment.
    EXPECT_NEAR(line.resolution(1e-3).unwrap(), 1e-3 / 2.25, 1e-18);
    let p = periodic();
    assert!(p.resolution(1.0).unwrap() > 0.0);
    let c = cubic();
    let reference = cubic();
    c.reverse().unwrap();
    near(c.value(0.2).unwrap(), reference.value(0.8).unwrap());
    near(c.end_point().unwrap(), reference.start_point().unwrap());
    c.transform(Transform2::translation([1.0, 0.0])).unwrap();
    near(
        c.value(0.2).unwrap(),
        reference
            .value(0.8)
            .unwrap()
            .translated(Vector2::new(1.0, 0.0)),
    );
    let copy = Geom2dBSplineCurve::copy_of(&c).unwrap();
    assert_eq!(copy.poles().as_slice(), c.poles().as_slice());
    assert_eq!(Geom2dBSplineCurve::max_degree(), 25);
    let generic: Handle<dyn Geom2dCurve> = c.copy().unwrap().cast();
    assert_eq!(generic.value(0.5).unwrap(), c.value(0.5).unwrap());
}

#[test]
fn unclamped_ends_are_evaluated() {
    let c = uniform();
    assert_eq!(
        c.start_point().unwrap(),
        c.value(c.first_parameter()).unwrap()
    );
    assert_eq!(c.end_point().unwrap(), c.value(c.last_parameter()).unwrap());
    assert!(!c.is_closed().unwrap());
}
