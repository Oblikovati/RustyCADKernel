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

//! Regression and behavior tests of the Geom2d port beyond the legacy suites. Expected
//! bits come from the native oracle (tools/verify_tkg2d_numeric.py).
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Handle, RuntimeTyped};
use rustycad_tkg2d::geom2d::{
    Geom2dBezierCurve, Geom2dCircle, Geom2dConic, Geom2dCurve, Geom2dDirection, Geom2dGeometry,
    Geom2dOffsetCurve, Geom2dTrimmedCurve, Geom2dVector,
};
use rustycad_tkg2d::geom2d_eval::{Map1d, RepCurveDescFull, RepCurveDescMapped};
use rustycad_tkmath::gp::{Axis2d, Direction2, Point2, Vector2};

fn circle(radius: f64) -> Handle<Geom2dCircle> {
    let axis = Axis2d::new(Point2::new(0.0, 0.0), Direction2::new(1.0, 0.0).unwrap());
    Handle::new(Geom2dCircle::from_axis(axis, radius, true).unwrap())
}

// Geom2d_Direction::Copy passes gpVec2d through the gp_Dir2d constructor, which
// normalizes the stored unit vector again; the copy differs in the last bit here.
#[test]
fn direction_copy_renormalizes_like_the_native_copy() {
    let direction =
        Handle::new(Geom2dDirection::new(-8.016135604395185, -8.707126967791696).unwrap());
    let copy: Handle<dyn Geom2dVector> = direction.copy().unwrap().cast();
    assert_eq!(direction.x().to_bits(), 13827647938583653192);
    assert_eq!(copy.x().to_bits(), 13827647938583653193);
    assert_eq!(copy.y().to_bits(), 13828173817748843949);
}

// Geom2d_TrimmedCurve::SetTrim stores the new bounds before it raises the out of
// range failure, and Copy then re-trims them and fails too.
#[test]
fn failed_set_trim_keeps_the_rejected_bounds() {
    let mut poles = Array1::<Point2>::with_bounds(1, 3).unwrap();
    poles[1] = Point2::new(0.0, 0.0);
    poles[2] = Point2::new(1.0, 1.0);
    poles[3] = Point2::new(2.0, 0.0);
    let bezier: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dBezierCurve::new(&poles).unwrap()).cast();
    let trimmed = Handle::new(Geom2dTrimmedCurve::new(&bezier, 0.2, 0.8, true, true).unwrap());
    let error = trimmed.set_trim(0.5, 2.0, true, true).err().unwrap();
    assert_eq!(error.exception_type(), "Standard_ConstructionError");
    assert_eq!(
        (trimmed.first_parameter(), trimmed.last_parameter()),
        (0.5, 2.0)
    );
    let copy_error = trimmed.copy().err().unwrap();
    assert_eq!(copy_error.exception_type(), "Standard_ConstructionError");
    // Reversed bounds on a non-periodic basis reverse the basis copy: [1-0.8, 1-0.2].
    trimmed.set_trim(0.8, 0.2, true, true).unwrap();
    assert_eq!(trimmed.first_parameter(), 1.0 - 0.8);
    assert_eq!(trimmed.last_parameter(), 1.0 - 0.2);
    let (start, expected) = (
        trimmed.value(1.0 - 0.8).unwrap(),
        bezier.value(0.8).unwrap(),
    );
    assert!(start.distance(expected) <= 1e-15);
}

// SetBasisCurve with the offset curve itself unwraps it and adds its own offset.
#[test]
fn offset_of_itself_doubles_the_offset() {
    let basis: Handle<dyn Geom2dCurve> = circle(2.0).cast();
    let offset = Handle::new(Geom2dOffsetCurve::new(&basis, 0.5, false).unwrap());
    offset.set_basis_curve(&offset.cast(), false).unwrap();
    assert_eq!(offset.offset(), 1.0);
    assert!(
        offset
            .basis_curve()
            .is_kind(<Geom2dCircle as RuntimeTyped>::static_type())
    );
    assert_eq!(offset.value(0.0).unwrap(), Point2::new(3.0, 0.0));
}

// Mutation through one handle is visible through another handle of a base class.
#[test]
fn shared_mutation_through_base_handles() {
    let concrete = circle(2.0);
    let conic: Handle<dyn Geom2dConic> = concrete.cast();
    concrete.set_radius(5.0).unwrap();
    assert_eq!(conic.value(0.0).unwrap(), Point2::new(5.0, 0.0));
    conic.set_location(Point2::new(1.0, 0.0));
    assert_eq!(concrete.value(0.0).unwrap(), Point2::new(6.0, 0.0));
    assert_eq!(conic.dynamic_type().unwrap().name(), "Geom2d_Circle");
}

// Evaluation representations: validation, mapped evaluation and clearing on Transform.
#[test]
fn evaluation_representation_is_validated_and_used() {
    let owner = Handle::new(Geom2dOffsetCurve::new(&circle(1.0).cast(), 1.0, false).unwrap());
    let own = Handle::new(RepCurveDescFull::new(owner.cast()));
    let error = owner.set_eval_representation(own.cast()).err().unwrap();
    assert_eq!(error.exception_type(), "Standard_ProgramError");
    let map = Map1d {
        scale: 2.0,
        offset: 0.0,
    };
    let mapped = Handle::new(RepCurveDescMapped::new(circle(2.0).cast(), 3, None, map));
    owner.set_eval_representation(mapped.cast()).unwrap();
    let d1: Vector2 = owner.dn(0.0, 1).unwrap();
    assert_eq!((d1.x(), d1.y()), (0.0, 4.0));
    owner.translate(Vector2::new(1.0, 0.0)).unwrap();
    assert!(!owner.has_eval_representation());
}
