// Copyright (c) 1993-1999 Matra Datavision
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

//! Regression tests for the Adaptor3d curve-on-surface, iso-curve, InterFunc and
//! HSurfaceTool behaviour that the numeric oracle cannot reach: the paths where
//! native code dereferences a null handle or writes out of its buffers, and the
//! failure propagation that replaces native exceptions inside math functions.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::adaptor2d::{Adaptor2dCurve2d, Adaptor2dCurve2dBase, Adaptor2dLine2d};
use rustycad_tkg3d::adaptor3d::h_surface_tool;
use rustycad_tkg3d::adaptor3d::{
    Adaptor3dCurve, Adaptor3dCurveOnSurface, Adaptor3dInterFunc, Adaptor3dIsoCurve,
    Adaptor3dSurface,
};
use rustycad_tkg3d::geom::{GeomPlane, GeomSphericalSurface, GeomSurface};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorSurface;
use rustycad_tkmath::geom_abs::{CurveType, IsoType, Shape};
use rustycad_tkmath::gp::{Axis3, Direction2, Direction3, Point2, Point3};
use rustycad_tkmath::math::{Function, FunctionWithDerivative};

fn frame() -> Axis3 {
    let z = Direction3::new(0.0, 0.0, 1.0).unwrap();
    Axis3::from_normal(Point3::new(0.0, 0.0, 0.0), z).unwrap()
}
fn plane_adaptor() -> Handle<dyn Adaptor3dSurface> {
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(frame())).cast();
    Handle::new(GeomAdaptorSurface::with_surface(&plane).unwrap()).cast()
}
fn sphere_adaptor() -> Handle<dyn Adaptor3dSurface> {
    let sphere: Handle<dyn GeomSurface> =
        Handle::new(GeomSphericalSurface::new(frame(), 2.0).unwrap()).cast();
    Handle::new(GeomAdaptorSurface::with_surface(&sphere).unwrap()).cast()
}
fn segment(direction: (f64, f64)) -> Handle<dyn Adaptor2dCurve2d> {
    let d = Direction2::new(direction.0, direction.1).unwrap();
    Handle::new(Adaptor2dLine2d::new(Point2::new(0.3, 0.2), d, 0.0, 1.0)).cast()
}

#[test]
fn inter_func_rejects_a_fix_other_than_one_or_two() {
    let error = Adaptor3dInterFunc::new(&segment((1.0, 0.0)), 0.5, 3)
        .err()
        .unwrap();
    assert_eq!(error.exception_type(), "Standard_ConstructionError");
}

#[test]
fn inter_func_keeps_the_curve_failure_for_its_caller() {
    let base: Handle<dyn Adaptor2dCurve2d> = Handle::new(Adaptor2dCurve2dBase::default()).cast();
    let mut function = Adaptor3dInterFunc::new(&base, 0.5, 1).unwrap();
    let mut value = 0.0;
    assert!(!function.value(0.1, &mut value));
    assert_eq!(
        function.take_failure().unwrap().exception_type(),
        "Standard_NotImplemented"
    );
    let (mut f, mut d) = (0.0, 0.0);
    assert!(!function.values(0.1, &mut f, &mut d));
    assert!(function.take_failure().is_some());
    assert!(function.take_failure().is_none());
}

#[test]
fn inter_func_measures_the_fixed_coordinate() {
    let mut function = Adaptor3dInterFunc::new(&segment((0.6, 0.8)), 0.5, 2).unwrap();
    let (mut f, mut d) = (0.0, 0.0);
    assert!(function.values(1.0, &mut f, &mut d));
    assert_eq!((f, d), (0.2 + 0.8 - 0.5, 0.8));
}

#[test]
fn curve_on_surface_without_curve_reports_a_null_object() {
    // Natively the null 2D curve is dereferenced.
    let adaptor = Adaptor3dCurveOnSurface::with_surface(&plane_adaptor()).unwrap();
    assert_eq!(adaptor.get_type().unwrap(), CurveType::OtherCurve);
    let error = adaptor.first_parameter().err().unwrap();
    assert_eq!(error.exception_type(), "Standard_NullObject");
}

#[test]
fn curve_on_surface_intervals_check_the_array_length_and_bounds() {
    let adaptor =
        Adaptor3dCurveOnSurface::with_curve(&segment((1.0, 0.0)), &plane_adaptor()).unwrap();
    let mut wrong_length: Array1<'static, f64> = Array1::with_bounds(1, 3).unwrap();
    let error = adaptor
        .intervals(&mut wrong_length, Shape::C2)
        .err()
        .unwrap();
    assert_eq!(error.exception_type(), "Standard_ProgramError");
    // Natively a zero-based array is written past its end (raw indices from 1).
    let mut shifted: Array1<'static, f64> = Array1::with_bounds(0, 1).unwrap();
    let error = adaptor.intervals(&mut shifted, Shape::C2).err().unwrap();
    assert_eq!(error.exception_type(), "Standard_OutOfRange");
}

#[test]
fn curve_on_surface_change_curve_keeps_the_special_kind() {
    let adaptor =
        Adaptor3dCurveOnSurface::with_curve(&segment((1.0, 0.0)), &sphere_adaptor()).unwrap();
    assert_eq!(adaptor.get_type().unwrap(), CurveType::Circle);
    let oblique = segment((0.6, 0.8));
    adaptor.change_curve(|curve| *curve = oblique.clone());
    assert!(adaptor.get_curve() == oblique);
    assert_eq!(adaptor.get_type().unwrap(), CurveType::Circle);
    adaptor.load_curve(&oblique).unwrap();
    assert_eq!(adaptor.get_type().unwrap(), CurveType::OtherCurve);
}

/// The copy shares the curve and surface but not the fields: loading a new curve
/// into the original leaves the copy on the old one (GeomFill_BoundWithSurf keeps
/// such a copy).
#[test]
fn curve_on_surface_copy_keeps_its_own_fields() {
    let adaptor =
        Adaptor3dCurveOnSurface::with_curve(&segment((1.0, 0.0)), &sphere_adaptor()).unwrap();
    let copy = Adaptor3dCurveOnSurface::copy_of(&adaptor);
    assert!(copy.get_curve() == adaptor.get_curve());
    assert!(copy.get_surface() == adaptor.get_surface());
    adaptor.load_curve(&segment((0.6, 0.8))).unwrap();
    assert_eq!(adaptor.get_type().unwrap(), CurveType::OtherCurve);
    assert_eq!(copy.get_type().unwrap(), CurveType::Circle);
}

#[test]
fn curve_on_surface_eval_dn_stops_at_order_three() {
    let adaptor =
        Adaptor3dCurveOnSurface::with_curve(&segment((1.0, 0.0)), &plane_adaptor()).unwrap();
    let error = adaptor.eval_dn(0.5, 4).err().unwrap();
    assert_eq!(error.exception_type(), "Standard_NotImplemented");
    assert_eq!(
        adaptor.circle().err().unwrap().exception_type(),
        "Standard_NoSuchObject"
    );
}

#[test]
fn empty_iso_curve_reports_a_null_surface_for_its_kind() {
    // Natively GetType dereferences the null surface.
    let iso = Adaptor3dIsoCurve::default();
    assert_eq!(iso.iso(), IsoType::NoneIso);
    assert_eq!(
        iso.get_type().err().unwrap().exception_type(),
        "Standard_NullObject"
    );
    assert_eq!(
        iso.continuity().err().unwrap().exception_type(),
        "Standard_NoSuchObject"
    );
}

#[test]
fn iso_curve_on_a_sphere_is_a_circle_clipped_to_the_surface() {
    let iso = Adaptor3dIsoCurve::with_iso_range(&sphere_adaptor(), IsoType::IsoU, 0.5, (-9.0, 9.0))
        .unwrap();
    assert_eq!(iso.get_type().unwrap(), CurveType::Circle);
    assert_eq!(iso.first_parameter().unwrap(), -std::f64::consts::FRAC_PI_2);
    assert_eq!(iso.last_parameter().unwrap(), std::f64::consts::FRAC_PI_2);
    let error = Adaptor3dIsoCurve::with_iso(&sphere_adaptor(), IsoType::NoneIso, 0.0)
        .err()
        .unwrap();
    assert_eq!(error.exception_type(), "Standard_NoSuchObject");
}

#[test]
fn surface_tool_samples_and_g1_test() {
    let sphere = sphere_adaptor();
    assert_eq!(h_surface_tool::nb_samples_u(&sphere).unwrap(), 10);
    assert_eq!(h_surface_tool::nb_samples_v(&sphere).unwrap(), 15);
    assert_eq!(
        h_surface_tool::nb_samples_v_range(&sphere, 0.0, 1.0).unwrap(),
        5
    );
    assert!(!h_surface_tool::is_surf_g1(&sphere, true, 1e-12).unwrap());
    assert_eq!(h_surface_tool::nb_samples_u(&plane_adaptor()).unwrap(), 2);
}
