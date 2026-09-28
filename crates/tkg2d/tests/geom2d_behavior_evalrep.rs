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

//! Geom2dEval_RepCurveDesc descriptors and Geom2dEval_RepUtils on Geom2d curves.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{
    Geom2dBSplineCurve, Geom2dBezierCurve, Geom2dCircle, Geom2dCurve, Geom2dGeometry, Geom2dLine,
    Geom2dOffsetCurve,
};
use rustycad_tkg2d::geom2d_eval::rep_utils::{
    clone_curve_desc, try_eval_curve_d0, try_eval_curve_d1, try_eval_curve_d2, try_eval_curve_d3,
    try_eval_curve_dn, validate_curve_desc,
};
use rustycad_tkg2d::geom2d_eval::{
    Domain1d, Map1d, RepCurveDesc, RepCurveDescDerivBounded, RepCurveDescFull, RepCurveDescMapped,
    RepCurveKind,
};
use rustycad_tkmath::gp::{Axis2d, Direction2, Point2, Transform2, Vector2};

fn line() -> Handle<dyn Geom2dCurve> {
    let direction = Direction2::new(0.6, 0.8).unwrap();
    Handle::new(Geom2dLine::from_point(Point2::new(1.0, 1.0), direction)).cast()
}

fn circle() -> Handle<dyn Geom2dCurve> {
    let axis = Axis2d::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0).unwrap());
    Handle::new(Geom2dCircle::from_axis(axis, 2.0, true).unwrap()).cast()
}

fn bezier() -> Handle<Geom2dBezierCurve> {
    let mut poles = Array1::<Point2>::with_bounds(1, 3).unwrap();
    poles.as_mut_slice().copy_from_slice(&[
        Point2::new(0.0, 0.0),
        Point2::new(1.0, 2.0),
        Point2::new(2.0, 0.0),
    ]);
    Handle::new(Geom2dBezierCurve::new(&poles).unwrap())
}

fn mapped(map: Map1d, domain: Option<Domain1d>) -> Handle<dyn RepCurveDesc> {
    Handle::new(RepCurveDescMapped::new(circle(), 3, domain, map)).cast()
}

#[test]
fn maps_and_domains() {
    let identity = Map1d::default();
    assert!(identity.is_identity() && identity.is_valid());
    let map = Map1d {
        scale: 2.0,
        offset: 0.5,
    };
    assert!(!map.is_identity());
    assert_eq!(map.map(1.0), 2.5);
    assert!(
        !Map1d {
            scale: 0.0,
            offset: 0.0
        }
        .is_valid()
    );
    let domain = Domain1d::default();
    assert!(domain.contains(1.0 + 1e-8) && !domain.contains(1.1) && !domain.contains(-0.1));
}

#[test]
fn full_descriptor_redirects_every_order() {
    let owner = bezier();
    let full: Handle<dyn RepCurveDesc> = Handle::new(RepCurveDescFull::new(line())).cast();
    assert_eq!(
        (full.kind(), full.max_deriv_order(), full.mapping()),
        (RepCurveKind::Full, None, None)
    );
    owner.set_eval_representation(full.clone()).unwrap();
    assert!(owner.has_eval_representation());
    assert!(owner.eval_representation() == full);
    let l = line();
    assert_eq!(owner.value(3.0).unwrap(), l.value(3.0).unwrap());
    assert_eq!(owner.eval_d3(3.0).unwrap(), l.eval_d3(3.0).unwrap());
    assert_eq!(owner.eval_dn(3.0, 1).unwrap(), l.eval_dn(3.0, 1).unwrap());
    // The copy gets its own copy of the representation.
    let copy = Geom2dBezierCurve::copy_of(&owner).unwrap();
    assert!(copy.eval_representation().representation() != full.representation());
    assert_eq!(copy.value(3.0).unwrap(), l.value(3.0).unwrap());
    owner.clear_eval_representation();
    assert_eq!(owner.value(0.5).unwrap(), Point2::new(1.0, 1.0));
}

#[test]
fn bounded_descriptor_limits_the_order() {
    let owner = bezier();
    let desc = Handle::new(RepCurveDescDerivBounded::new(line(), 1));
    owner.set_eval_representation(desc.cast()).unwrap();
    assert_eq!(desc.max_deriv_order(), Some(1));
    assert_eq!(owner.eval_d1(0.5).unwrap(), line().eval_d1(0.5).unwrap());
    // Above the limit the Bezier evaluates itself: B''(t) = 2 (P0 - 2 P1 + P2) = (0, -8).
    assert_eq!(owner.eval_d2(0.5).unwrap().d2, Vector2::new(0.0, -8.0));
    assert_eq!(owner.eval_dn(0.5, 2).unwrap(), Vector2::new(0.0, -8.0));
    desc.set_max_deriv_order(3);
    assert_eq!(owner.eval_d2(0.5).unwrap().d2, Vector2::new(0.0, 0.0));
    desc.set_representation(circle());
    assert_eq!(owner.value(0.0).unwrap(), Point2::new(2.0, 0.0));
}

#[test]
fn mapped_descriptor_scales_derivatives() {
    let desc = mapped(
        Map1d {
            scale: 2.0,
            offset: 0.5,
        },
        None,
    );
    let c = circle();
    let direct = c.eval_d3(2.0 * 0.25 + 0.5).unwrap();
    let result = try_eval_curve_d3(&desc, 0.25).unwrap().unwrap();
    assert_eq!(result.point, direct.point);
    assert_eq!(
        (result.d1, result.d2, result.d3),
        (direct.d1 * 2.0, direct.d2 * 4.0, direct.d3 * 8.0)
    );
    assert_eq!(
        try_eval_curve_d2(&desc, 0.25).unwrap().unwrap().d2,
        direct.d2 * 4.0
    );
    assert_eq!(
        try_eval_curve_d1(&desc, 0.25).unwrap().unwrap().d1,
        direct.d1 * 2.0
    );
    assert_eq!(
        try_eval_curve_d0(&desc, 0.25).unwrap().unwrap(),
        direct.point
    );
    assert_eq!(
        try_eval_curve_dn(&desc, 0.25, 3).unwrap().unwrap(),
        direct.d3 * 8.0
    );
    assert_eq!(try_eval_curve_dn(&desc, 0.25, 4).unwrap(), None);
    assert_eq!(try_eval_curve_dn(&desc, 0.25, 0).unwrap(), None);
    let mapped = desc.cast::<RepCurveDescMapped>();
    mapped.set_domain(Some(Domain1d {
        first: 0.0,
        last: 1.0,
    }));
    assert!(try_eval_curve_d0(&desc, 0.25).unwrap().is_some());
    assert!(try_eval_curve_d0(&desc, 1.0).unwrap().is_none());
    mapped.set_param_map(Map1d::default());
    mapped.set_max_deriv_order(0);
    assert!(try_eval_curve_d1(&desc, 0.5).unwrap().is_none());
    assert_eq!(
        desc.mapping(),
        Some((
            Some(Domain1d {
                first: 0.0,
                last: 1.0
            }),
            Map1d::default()
        ))
    );
    let none: Handle<dyn RepCurveDesc> = Handle::default();
    assert!(try_eval_curve_d0(&none, 0.0).unwrap().is_none());
}

#[test]
fn validation_rejects_bad_descriptors() {
    let owner = bezier();
    let null: Handle<dyn RepCurveDesc> =
        Handle::new(RepCurveDescFull::new(Handle::default())).cast();
    assert_eq!(
        owner
            .set_eval_representation(null.clone())
            .unwrap_err()
            .exception_type(),
        "Standard_ProgramError"
    );
    let invalid = mapped(
        Map1d {
            scale: 0.0,
            offset: 1.0,
        },
        None,
    );
    assert_eq!(
        owner
            .set_eval_representation(invalid)
            .unwrap_err()
            .exception_type(),
        "Standard_ProgramError"
    );
    let own: Handle<dyn RepCurveDesc> = Handle::new(RepCurveDescFull::new(owner.cast())).cast();
    assert!(owner.set_eval_representation(own.clone()).is_err());
    assert!(validate_curve_desc(&Handle::default(), std::ptr::null()).is_ok());
    assert!(validate_curve_desc(&own, std::ptr::null()).is_ok());
    // A representation cleared after validation fails instead of crashing.
    let full = Handle::new(RepCurveDescFull::new(line()));
    owner.set_eval_representation(full.cast()).unwrap();
    full.set_representation(Handle::default());
    assert_eq!(
        owner.value(0.5).unwrap_err().exception_type(),
        "Standard_ProgramError"
    );
    assert!(clone_curve_desc(&null).unwrap().is_null());
    assert!(clone_curve_desc(&Handle::default()).unwrap().is_null());
}

#[test]
fn clones_keep_the_descriptor_kind() {
    let bounded: Handle<dyn RepCurveDesc> =
        Handle::new(RepCurveDescDerivBounded::new(line(), 2)).cast();
    let copy = clone_curve_desc(&bounded).unwrap();
    assert_eq!(
        (copy.kind(), copy.max_deriv_order()),
        (RepCurveKind::DerivBounded, Some(2))
    );
    let map = Map1d {
        scale: -0.5,
        offset: 0.25,
    };
    let domain = Some(Domain1d {
        first: 0.1,
        last: 0.9,
    });
    let copy = clone_curve_desc(&mapped(map, domain)).unwrap();
    assert_eq!(
        (copy.kind(), copy.mapping()),
        (RepCurveKind::Mapped, Some((domain, map)))
    );
}

#[test]
fn bspline_and_offset_accept_representations() {
    let mut poles = Array1::<Point2>::with_bounds(1, 2).unwrap();
    poles
        .as_mut_slice()
        .copy_from_slice(&[Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)]);
    let mut knots = Array1::<f64>::with_bounds(1, 2).unwrap();
    knots.as_mut_slice().copy_from_slice(&[0.0, 1.0]);
    let mut mults = Array1::<i32>::with_bounds(1, 2).unwrap();
    mults.init(2);
    let bspline = Handle::new(Geom2dBSplineCurve::new(&poles, &knots, &mults, 1, false).unwrap());
    bspline
        .set_eval_representation(Handle::new(RepCurveDescFull::new(line())).cast())
        .unwrap();
    assert_eq!(bspline.value(2.0).unwrap(), line().value(2.0).unwrap());
    // B-spline changes keep the representation, Transform included.
    bspline
        .transform(Transform2::translation([1.0, 0.0]))
        .unwrap();
    assert!(bspline.has_eval_representation());
    assert!(
        Geom2dBSplineCurve::copy_of(&bspline)
            .unwrap()
            .has_eval_representation()
    );
    bspline.clear_eval_representation();
    assert!(!bspline.has_eval_representation());
    let offset = Handle::new(Geom2dOffsetCurve::new(&circle(), 1.0, false).unwrap());
    offset
        .set_eval_representation(Handle::new(RepCurveDescFull::new(line())).cast())
        .unwrap();
    assert!(offset.eval_representation().is_some());
    assert_eq!(offset.eval_d2(0.5).unwrap(), line().eval_d2(0.5).unwrap());
    assert!(
        Geom2dOffsetCurve::copy_of(&offset)
            .unwrap()
            .has_eval_representation()
    );
    offset.set_offset_value(2.0);
    assert!(!offset.has_eval_representation());
    offset
        .set_eval_representation(Handle::new(RepCurveDescFull::new(line())).cast())
        .unwrap();
    offset.clear_eval_representation();
    assert!(!offset.has_eval_representation());
}
