// Copyright (c) 1998-1999 Matra Datavision
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

//! AppDef_SmoothCriterion and AppDef_LinearCriteria.
use super::{arc_line, kind};
use rustycad_tkernel::ncollection::{Array1, Array2, HArray1};
use rustycad_tkernel::standard::Handle;
use rustycad_tkgeombase::app_def::{LinearCriteria, SmoothCriterion};
use rustycad_tkgeombase::fem_tool::Curve;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::math::{Matrix, Vector};
use rustycad_tkmath::plib::HermitJacobi;

/// A 3D curve of `knots.len() - 1` elements of degree 6 (C1), every coefficient 0.1.
fn curve(knots: &[f64]) -> Handle<Curve> {
    let nb: i32 = knots.len() as i32 - 1;
    let c = Curve::new(3, nb, &HermitJacobi::new(6, Shape::C1).unwrap(), 1e-7).unwrap();
    for (i, k) in (1..).zip(knots) {
        c.knots()[i] = *k;
    }
    let mut coeff = Array2::with_bounds(0, 6, 1, 3).unwrap();
    coeff.init(0.1);
    (1..=nb).for_each(|el| c.set_element(el, &coeff).unwrap());
    Handle::new(c)
}

fn parameters(values: &[f64]) -> Handle<HArray1<f64>> {
    Handle::new(HArray1::from_array(&Array1::from_shared(values, 1)))
}

#[test]
fn members_needing_the_curve_are_domain_errors_before_set_curve() {
    let j = LinearCriteria::new(&arc_line(3), 1, 3).unwrap();
    assert_eq!(kind(j.assembly_table()), "Standard_DomainError");
    assert_eq!(kind(j.dependence_table()), "Standard_DomainError");
    assert_eq!(
        kind(j.quality_values(1e-8, 1e-6, 1e-6)),
        "Standard_DomainError"
    );
    let mut h = Matrix::with_bounds(0, 6, 0, 6).unwrap();
    assert_eq!(kind(j.hessian(1, 1, 1, &mut h)), "Standard_DomainError");
    // InputVector has no null check natively (a crash): the port's null error.
    let table = Handle::default();
    assert_eq!(
        kind(j.input_vector(&Vector::with_bounds(1, 1).unwrap(), &table)),
        "Standard_NullObject"
    );
}

#[test]
fn set_curve_of_a_null_first_curve_is_a_null_object_error() {
    let j = LinearCriteria::new(&arc_line(3), 1, 3).unwrap();
    assert_eq!(kind(j.set_curve(&Handle::default())), "Standard_NullObject");
}

#[test]
fn a_gradient_before_any_cache_on_an_empty_element_is_a_null_object_error() {
    // Element 2 of [0, 0.9, 0.95] holds no point: BuildCache leaves the cache null,
    // which the native Gradient dereferences.
    let j = LinearCriteria::new(&arc_line(3), 1, 3).unwrap();
    j.set_parameters(&parameters(&[0.0, 0.5, 0.8]));
    j.set_curve(&curve(&[0.0, 0.9, 0.95])).unwrap();
    let mut g = Vector::with_bounds(0, 6).unwrap();
    assert_eq!(kind(j.gradient(2, 1, &mut g)), "Standard_NullObject");
    // Element 1 builds it; element 2 then reuses it with an empty point range.
    j.gradient(1, 1, &mut g).unwrap();
    j.gradient(2, 1, &mut g).unwrap();
    assert_eq!(g[6], 0.0);
}

#[test]
fn dimensions_outside_the_line_are_domain_errors() {
    let j = LinearCriteria::new(&arc_line(3), 1, 3).unwrap();
    j.set_parameters(&parameters(&[0.0, 0.5, 1.0]));
    j.set_curve(&curve(&[0.0, 1.0])).unwrap();
    let mut g = Vector::with_bounds(0, 6).unwrap();
    assert_eq!(kind(j.gradient(1, 4, &mut g)), "Standard_DomainError");
    let mut h = Matrix::with_bounds(0, 6, 0, 6).unwrap();
    assert_eq!(kind(j.hessian(1, 1, 2, &mut h)), "Standard_DomainError");
}

#[test]
fn weights_are_normalized_and_checked() {
    let j = LinearCriteria::new(&arc_line(3), 1, 3).unwrap();
    assert_eq!(
        kind(j.set_weight(-1., 1., [1., 1., 1.])),
        "Standard_DomainError"
    );
    assert_eq!(
        kind(j.set_weight(1., 1., [1., -1., 1.])),
        "Standard_DomainError"
    );
    j.set_weight(2., 3., [1., 1., 2.]).unwrap();
    assert_eq!(j.get_weight(), (2., 3.));
    // SetWeight(Array1) takes the bounds of its argument, as the native assignment.
    j.set_point_weights(&Array1::from_shared(&[1., 2.], 7));
    j.set_est_length(4.5);
    assert_eq!(j.est_length(), 4.5);
}

#[test]
fn a_wrapped_point_range_is_out_of_memory() {
    // Natively the allocation of the wrapped size fails and the next write crashes.
    assert_eq!(
        kind(LinearCriteria::new(&arc_line(3), 3, 1)),
        "Standard_OutOfMemory"
    );
    assert_eq!(kind(LinearCriteria::new(&arc_line(3), 2, 1)), "none");
}

#[test]
fn quality_values_raise_the_estimations_of_underestimated_criteria() {
    let j = LinearCriteria::new(&arc_line(3), 1, 3).unwrap();
    j.set_curve(&curve(&[0.0, 1.0])).unwrap();
    j.set_estimation(0., 0., 0.);
    let q = j.quality_values(1e-8, 1e-6, 1e-6).unwrap();
    // A zero estimation below every criterion: the estimation grows by 10 (5) percent
    // of the value and ICDANA is 1 or 2.
    assert!(q.icdana >= 1);
    let e = j.get_estimation();
    assert_eq!(e[0], q.criteria[0] * 0.1);
    assert_eq!(e[2], q.criteria[2] * 0.05);
}
