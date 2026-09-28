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

//! Regression tests of the native behaviour the LProp, GeomLProp, GProp and Hermit
//! ports preserve (docs/tkgeombase-progress.md); the values were read from the
//! pinned native build.
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use common::EXPECT_THROW;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg2d::geom2d::{Geom2dBezierCurve, Geom2dCurve};
use rustycad_tkg3d::geom::{GeomCurve, GeomLine, GeomPlane, GeomSphericalSurface, GeomSurface};
use rustycad_tkgeombase::geom_lprop::{self, CLProps2d, SLProps};
use rustycad_tkgeombase::gprop::{PEquation, PGProps};
use rustycad_tkgeombase::lprop::{CIType, CurAndInf};
use rustycad_tkmath::gp::{Axis3, Direction3, Point2, Point3};

fn plane() -> Handle<dyn GeomSurface> {
    Handle::new(GeomPlane::new(Axis3::default())).cast()
}

fn sphere() -> Handle<dyn GeomSurface> {
    Handle::new(GeomSphericalSurface::new(Axis3::default(), 2.0).unwrap()).cast()
}

#[test]
fn cur_and_inf_drops_a_parameter_equal_to_the_last_one() {
    let mut points = CurAndInf::new();
    points.add_inflection(1.0);
    points.add_ext_cur(1.0, true);
    points.add_ext_cur(0.5, false);
    assert_eq!(points.nb_points(), 2);
    assert_eq!(points.type_of(2).unwrap(), CIType::Inflection);
}

#[test]
fn slprops_first_tangent_u_orients_the_second_derivative() {
    // The fresh object passes significant order 0 by value: D2U of a plane is null.
    let mut props = SLProps::with_parameters(plane(), 0.3, 0.4, 2, 1e-7).unwrap();
    EXPECT_THROW(props.tangent_u(), "Standard_ConstructionError");
    let tangent = props.tangent_u().unwrap();
    assert_eq!(tangent.xyz().x(), 1.0);
}

#[test]
fn slprops_curvature_getters_return_the_value_read_before_the_computation() {
    let mut props = SLProps::with_parameters(sphere(), 0.3, 0.4, 2, 1e-7).unwrap();
    assert_eq!(props.max_curvature().unwrap(), 0.0);
    assert_eq!(props.max_curvature().unwrap(), -0.5);
    props.set_parameters(1.0, 0.2).unwrap();
    assert_eq!(props.min_curvature().unwrap(), -0.5);
}

#[test]
fn clprops_curvature_is_real_last_where_the_first_derivative_vanishes() {
    let mut poles = Array1::with_bounds(1, 3).unwrap();
    poles[1] = Point2::new(0.0, 0.0);
    poles[2] = Point2::new(0.0, 0.0);
    poles[3] = Point2::new(1.0, 1.0);
    let curve: Handle<dyn Geom2dCurve> =
        Handle::new(Geom2dBezierCurve::new(&poles).unwrap()).cast();
    let mut props = CLProps2d::with_parameter(curve, 0.0, 2, 1e-7).unwrap();
    assert_eq!(props.curvature().unwrap(), f64::MAX);
    // RealLast() passes the null-curvature test; the normal vector of a null D1 is
    // null, so its normalization raises.
    EXPECT_THROW(props.centre_of_curvature(), "Standard_ConstructionError");
    EXPECT_THROW(props.normal(), "LProp_NotDefined");
}

#[test]
fn continuity_refuses_curves_that_do_not_meet() {
    let x = Direction3::new(1.0, 0.0, 0.0).unwrap();
    let line: Handle<dyn GeomCurve> =
        Handle::new(GeomLine::from_point(Point3::default(), x)).cast();
    EXPECT_THROW(
        geom_lprop::continuity(&line, &line, 1.0, 0.0, false, false),
        "Standard_Failure",
    );
}

#[test]
fn pgprops_barycentre_of_no_point_is_out_of_range() {
    let points: Array1<'static, Point3> = Array1::default();
    EXPECT_THROW(PGProps::barycentre(&points), "Standard_OutOfRange");
}

#[test]
fn pequation_axis_index_is_checked() {
    let mut points = Array1::with_bounds(1, 2).unwrap();
    points[1] = Point3::new(0.0, 0.0, 0.0);
    points[2] = Point3::new(1.0, 0.0, 0.0);
    let equation = PEquation::new(&points, 1e-6);
    EXPECT_THROW(equation.principal_axis(4), "Standard_OutOfRange");
    EXPECT_THROW(equation.extent(0), "Standard_OutOfRange");
}
