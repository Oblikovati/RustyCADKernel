// Copyright (c) 1995-1999 Matra Datavision
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

//! Behavior of the Law port beyond the numeric probe, which compares the evaluators,
//! edits and class functions with OCCT bit for bit.
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Handle, RuntimeTyped};
use rustycad_tkgeomalgo::law::*;
use rustycad_tkmath::geom_abs::Shape;

fn array<T: Copy + Default>(values: &[T]) -> Array1<'static, T> {
    let mut result: Array1<'static, T> = Array1::with_bounds(1, values.len() as i32).unwrap();
    result.as_mut_slice().copy_from_slice(values);
    result
}

fn cubic() -> LawBSpline {
    let poles = array(&[0.0, 1.0, 3.0, 2.0, 4.0]);
    LawBSpline::new(
        &poles,
        &array(&[0.0, 0.5, 1.0]),
        &array(&[4, 1, 4]),
        3,
        false,
    )
    .unwrap()
}

#[test]
fn bspline_accessors_read_the_stored_arrays() {
    let law = cubic();
    assert_eq!((law.pole(3).unwrap(), law.knot(2).unwrap()), (3.0, 0.5));
    assert_eq!(
        (law.multiplicity(1).unwrap(), law.weight(2).unwrap()),
        (4, 1.0)
    );
    assert!(law.pole(6).is_err() && law.knot(0).is_err() && law.multiplicity(4).is_err());
    let mut flat = array(&[0.0; 9]);
    law.knot_sequence(&mut flat).unwrap();
    assert_eq!(
        flat.as_slice(),
        &[0.0, 0.0, 0.0, 0.0, 0.5, 1.0, 1.0, 1.0, 1.0]
    );
    let mut short = array(&[0.0; 2]);
    let error = law.knots(&mut short).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_DimensionError");
    assert_eq!(LawBSpline::max_degree(), 25);
    assert_eq!(LawBSpline::static_type().name(), "Law_BSpline");
}

#[test]
fn bspline_local_evaluators_agree_with_the_global_ones_inside_a_span() {
    let law = cubic();
    let value = law.d0(0.25).unwrap();
    assert_eq!(law.local_d0(0.25, 1, 2).unwrap(), value);
    assert_eq!(law.local_d1(0.25, 1, 2).unwrap().0, value);
    assert_eq!(law.local_d2(0.25, 1, 2).unwrap(), law.d2(0.25).unwrap());
    assert_eq!(
        law.local_d0(0.25, 2, 2).unwrap_err().exception_type(),
        "Standard_DomainError"
    );
}

#[test]
fn bspline_setters_keep_the_law_valid() {
    let law = cubic();
    law.set_pole(1, 0.5).unwrap();
    assert_eq!(law.value(0.0).unwrap(), 0.5);
    assert!(law.set_pole(0, 1.0).is_err());
    law.set_knots(&array(&[0.0, 0.4, 1.0])).unwrap();
    assert_eq!(law.knot(2).unwrap(), 0.4);
    assert!(law.set_knots(&array(&[0.0, 1.0, 0.5])).is_err());
    law.insert_knots(&array(&[0.25, 0.75]), &array(&[1, 2]), 0.0, false)
        .unwrap();
    assert_eq!(law.nb_knots(), 5);
    assert!(!law.is_rational() && law.weight(1).unwrap() == 1.0);
    law.set_weight(2, 1.0).unwrap();
    assert!(!law.is_rational());
}

#[test]
fn bsp_func_without_curve_raises_null_object() {
    let law = LawBSpFunc::new();
    assert!(law.curve().is_null());
    assert_eq!(
        law.value(0.5).unwrap_err().exception_type(),
        "Standard_NullObject"
    );
    assert!(law.set_curve(&Handle::default()).is_err());
    law.set_curve(&Handle::new(cubic())).unwrap();
    assert_eq!(law.bounds().unwrap(), (0.0, 1.0));
    assert_eq!(law.continuity().unwrap(), Shape::C2);
    assert_eq!(LawBSpFunc::static_type().name(), "Law_BSpFunc");
    let function: Handle<dyn LawFunction> = Handle::new(LawS::new()).cast();
    assert!(function.is_kind_name("Law_BSpFunc") && function.is_kind_name("Law_Function"));
}

#[test]
fn composite_lends_its_current_law() {
    let composite = LawComposite::new();
    for (a, b) in [(0.0, 1.0), (1.0, 2.0)] {
        let piece = LawLinear::new();
        piece.set(a, a, b, b);
        composite.change_laws(|laws| {
            laws.append(Handle::new(piece).cast());
        });
    }
    let bounds = composite.change_elementary_law(1.5, |law| law.get().unwrap().bounds().unwrap());
    assert_eq!(bounds.unwrap(), (1.0, 2.0));
    assert_eq!(composite.bounds().unwrap(), (0.0, 2.0));
    assert!(!composite.is_periodic());
    composite.set_periodic();
    assert!(composite.is_periodic());
    assert_eq!(
        composite.continuity().unwrap_err().exception_type(),
        "Standard_NotImplemented"
    );
    let empty = LawComposite::default();
    assert_eq!(
        empty.value(0.0).unwrap_err().exception_type(),
        "Standard_NoSuchObject"
    );
}

#[test]
fn interpolate_reports_its_state() {
    let points = Handle::new(HArray1::from_owned(array(&[0.0, 1.0, 0.5, 2.0])));
    let mut inter = LawInterpolate::new(&points, false, 1e-9).unwrap();
    assert!(!inter.is_done());
    assert_eq!(
        inter.curve().err().unwrap().exception_type(),
        "StdFail_NotDone"
    );
    let flags = Handle::new(HArray1::from_owned(array(&[false, false])));
    let error = inter.load(&array(&[0.0, 0.0]), &flags).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_ConstructionError");
    let flags = Handle::new(HArray1::from_owned(array(&[false; 4])));
    inter.load(&array(&[0.0; 4]), &flags).unwrap();
    inter.perform().unwrap();
    assert!(inter.is_done());
    // Perform computed the end tangents and marked them in the shared flags.
    let marked = flags.get().unwrap().array1().as_slice().to_vec();
    assert_eq!(marked, vec![true, false, false, true]);
    let bad = Handle::new(HArray1::from_owned(array(&[0.0, 0.0, 1.0, 2.0])));
    assert!(LawInterpolate::with_parameters(&points, &bad, false, 1e-9).is_err());
}

#[test]
fn knot_splitting_rejects_a_negative_range() {
    let error = LawBSplineKnotSplitting::new(&Handle::new(cubic()), -1).unwrap_err();
    assert_eq!(error.exception_type(), "Standard_RangeError");
    let split = LawBSplineKnotSplitting::new(&Handle::new(cubic()), 3).unwrap();
    assert_eq!((split.nb_splits(), split.split_value(2).unwrap()), (3, 2));
    assert!(split.split_value(0).is_err());
}
