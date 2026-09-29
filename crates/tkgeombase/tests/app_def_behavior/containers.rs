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

//! AppDef_MultiPointConstraint, AppDef_MultiLine and AppDef_MyLineTool.
use super::{arc_line, kind};
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkgeombase::app_def::{MultiLine, MultiPointConstraint};
use rustycad_tkgeombase::app_par_curves::LineTool;
use rustycad_tkgeombase::approx::Status;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};

fn p3(n: usize) -> Vec<Point3> {
    (0..n).map(|i| Point3::new(i as f64, 1.0, 2.0)).collect()
}

#[test]
fn reading_vectors_never_set_is_a_null_object_error() {
    // Natively Tang(1) dereferences the null tangent array.
    let m = MultiPointConstraint::from_points(&Array1::from_shared(&p3(2), 1));
    assert_eq!(kind(m.tang(1)), "Standard_NullObject");
    assert_eq!(kind(m.curv(2)), "Standard_NullObject");
    assert_eq!(kind(m.tang(3)), "Standard_OutOfRange");
}

#[test]
fn a_failing_setter_still_allocates_its_array() {
    // SetTang allocates `1..=NbPoints()` before its index check: a 2D-only point
    // becomes a tangency point although the call raises.
    let mut m =
        MultiPointConstraint::from_points2d(&Array1::from_shared(&[Point2::new(0., 0.)], 1));
    assert!(!m.is_tangency_point());
    assert_eq!(
        kind(m.set_tang(1, Vector3::new(1., 0., 0.))),
        "Standard_OutOfRange"
    );
    assert!(m.is_tangency_point());
    m.set_tang2d(1, Vector2::new(0., 1.)).unwrap();
    assert_eq!(m.tang2d(1).unwrap(), Vector2::new(0., 1.));
}

#[test]
fn vector_constructors_refuse_mismatched_counts() {
    let (p, v1, v2) = (
        p3(2),
        [Vector3::new(1., 0., 0.)],
        [Vector3::new(1., 0., 0.); 2],
    );
    let points = Array1::from_shared(&p, 1);
    let one = Array1::from_shared(&v1, 1);
    assert_eq!(
        kind(MultiPointConstraint::with_tangents_3d(&points, &one)),
        "Standard_ConstructionError"
    );
    let two = Array1::from_shared(&v2, 5);
    let m = MultiPointConstraint::with_tangents_3d(&points, &two).unwrap();
    // The vectors are reindexed from 1 whatever their lower bound.
    assert_eq!(m.tang(2).unwrap(), Vector3::new(1., 0., 0.));
}

#[test]
fn copies_share_allocated_arrays_only() {
    let mut m = MultiPointConstraint::from_points(&Array1::from_shared(&p3(1), 1));
    let mut copy = m.clone();
    copy.set_tang(1, Vector3::new(0., 0., 1.)).unwrap();
    // The copy allocated its own array: the original has none.
    assert!(!m.is_tangency_point());
    m.set_tang(1, Vector3::new(1., 0., 0.)).unwrap();
    let mut shared = m.clone();
    shared.set_tang(1, Vector3::new(0., 1., 0.)).unwrap();
    assert_eq!(m.tang(1).unwrap(), Vector3::new(0., 1., 0.));
    assert_eq!(m.to_string(), "AppDef_MultiPointConstraint dump:\n");
}

#[test]
fn the_default_line_has_no_array() {
    let line = MultiLine::new();
    assert_eq!(kind(line.nb_multi_points()), "Standard_NullObject");
    assert_eq!(kind(line.last_point()), "Standard_NullObject");
    assert_eq!(line.first_point(), 1);
    assert_eq!(
        kind(MultiLine::with_nb_multi_points(-1)),
        "Standard_ConstructionError"
    );
}

#[test]
fn an_empty_line_raises_on_its_first_multipoint() {
    let line = MultiLine::with_nb_multi_points(0).unwrap();
    assert_eq!(line.last_point().unwrap(), 0);
    assert_eq!(kind(line.nb_p3d()), "Standard_OutOfRange");
    assert_eq!(kind(line.nb_points()), "Standard_OutOfRange");
    let mut text = String::new();
    line.dump(&mut text).unwrap();
    assert_eq!(
        text,
        "AppDef_MultiLine dump:\nIt contains 0 MultiPointConstraint\n"
    );
}

#[test]
fn line_copies_share_their_multipoints() {
    let line = arc_line(3);
    let copy = line.clone();
    copy.set_value(2, &MultiPointConstraint::with_counts(0, 1).unwrap())
        .unwrap();
    assert_eq!(line.value(2).unwrap().nb_points2d(), 1);
    assert_eq!(
        kind(line.set_value(4, &MultiPointConstraint::new())),
        "Standard_OutOfRange"
    );
}

#[test]
fn the_line_tool_writes_from_the_lower_bound_and_checks_the_size() {
    let line = arc_line(3);
    let sentinels = [Point3::new(-7., -7., -7.); 2];
    let mut points = Array1::from_shared(&sentinels, -3).clone();
    line.value_3d(2, &mut points).unwrap();
    assert_eq!(points[-3], Point3::new(0.5, 0.25, 0.0125));
    let mut empty: Array1<'static, Point3> = Array1::default();
    assert_eq!(kind(line.value_3d(1, &mut empty)), "Standard_OutOfRange");
    let mut tangents: Array1<'static, Vector3> = Array1::default();
    assert!(!line.tangency_3d(1, &mut tangents).unwrap());
}

#[test]
fn the_line_tool_never_cuts_the_line() {
    let line = arc_line(4);
    assert_eq!(line.what_status(1, 4).unwrap(), Status::NoPointsAdded);
    assert_eq!(
        line.make_ml_between(1, 4, 10)
            .unwrap()
            .nb_multi_points()
            .unwrap(),
        4
    );
    assert!(line.make_ml_one_more_point(1, 4, 2).unwrap().is_none());
}
