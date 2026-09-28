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

//! Regression tests of the native behaviour the GeomBndLib and BndLib ports preserve
//! (docs/tkgeombase-progress.md); the bitwise values are compared with the pinned native
//! build by tools/tkgeombase_corpus_bndlib.py.
mod bnd_support;
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
use bnd_support::{array1, array2, curve, dir, get3, pnt, surface};
use common::EXPECT_THROW;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Handle;
use rustycad_tkg3d::geom::{
    GeomBSplineSurface, GeomBezierCurve, GeomCircle, GeomOffsetCurve, GeomOffsetSurface, GeomPlane,
    GeomTrimmedCurve,
};
use rustycad_tkg3d::geom_adaptor::GeomAdaptorCurve;
use rustycad_tkgeombase::bnd_lib;
use rustycad_tkgeombase::geom_bnd_lib::{
    Circle, Curve, Hyperbola2d, Line, Plane, Surface, sampling,
};
use rustycad_tkmath::bnd::{BoundingBox2, BoundingBox3};
use rustycad_tkmath::geom_abs::{CurveType, SurfaceType};
use rustycad_tkmath::gp::{self, Axis2d, Axis3, Circle3, Cylinder3, Hyperbola2, Line3, Torus3};
use std::f64::consts::PI;

const INF: f64 = 1e101;

#[test]
fn equal_infinite_ends_raise_standard_failure() {
    let line = Line3::new(pnt(0.0, 0.0, 0.0), gp::dx());
    EXPECT_THROW(Line::lin_box(&line, -INF, -INF, 0.0), "Standard_Failure");
    EXPECT_THROW(Line::lin_box(&line, INF, INF, 0.0), "Standard_Failure");
    let mut bounds = BoundingBox3::default();
    EXPECT_THROW(
        bnd_lib::add_lin(&line, INF, INF, 0.0, &mut bounds),
        "Standard_Failure",
    );
    let cylinder = Cylinder3::new(gp::xoy().axis3(), 1.0).unwrap();
    EXPECT_THROW(
        bnd_lib::add_cylinder(&cylinder, -INF, -INF, 0.0, &mut bounds),
        "Standard_Failure",
    );
}

#[test]
fn doubly_infinite_line_adds_the_origin_point_and_opens_its_axis() {
    let line = Line3::new(pnt(1.0, 2.0, 3.0), gp::dy());
    let bounds = Line::lin_box(&line, INF, -INF, 0.0).unwrap();
    assert!(bounds.is_open_ymin() && bounds.is_open_ymax());
    assert!(!bounds.is_open_xmin() && !bounds.is_open_zmax());
    let [x_min, x_max, _, _, z_min, z_max] = get3(&bounds);
    assert_eq!((x_min, x_max, z_min, z_max), (1.0, 1.0, 3.0, 3.0));
}

#[test]
fn bndlib_circle_arc_widens_the_box_it_is_given() {
    // The arc reads the accumulated finite part back (gap removed) before adding the
    // extrema: a point already in the box stays a bound.
    let circle = Circle3::new(gp::xoy(), 1.0).unwrap();
    let mut bounds = BoundingBox3::default();
    bounds.add_point(pnt(-5.0, 0.5, 0.0));
    bnd_lib::add_circ_arc(&circle, 0.0, PI / 2.0, 0.0, &mut bounds).unwrap();
    let [x_min, x_max, y_min, y_max, _, _] = get3(&bounds);
    assert_eq!((x_min, x_max, y_min, y_max), (-5.0, 1.0, 0.0, 1.0));
}

#[test]
fn bndlib_hyperbola2d_skips_the_coordinate_extrema() {
    // A rotated branch whose X extremum lies inside the arc: GeomBndLib adds it, the
    // BndLib 2D member (the parabola rule) does not.
    let axis = Axis2d::new(bnd_support::pnt2d(0.0, 0.0), bnd_support::dir2d(1.0, 1.0));
    let hyperbola = Hyperbola2::from_x_axis(axis, 3.0, 2.0, true).unwrap();
    let full = Hyperbola2d::hypr_box(&hyperbola, -1.0, 2.0, 0.0).unwrap();
    let mut loose = BoundingBox2::default();
    bnd_lib::add_hypr2d(&hyperbola, -1.0, 2.0, 0.0, &mut loose).unwrap();
    assert!(full.limits().x_min < loose.limits().x_min);
}

#[test]
fn offset_surface_fallback_enlarges_by_the_larger_of_offset_and_tolerance() {
    let poles = array2(
        2,
        2,
        &[
            pnt(0.0, 0.0, 0.0),
            pnt(0.0, 1.0, 0.5),
            pnt(1.0, 0.0, 0.5),
            pnt(1.0, 1.0, 0.0),
        ],
    );
    let knots = array1(&[0.0, 1.0]);
    let mults = array1(&[2, 2]);
    let basis = surface(
        GeomBSplineSurface::new(
            &poles,
            (&knots, &mults),
            (&knots, &mults),
            (1, 1),
            (false, false),
        )
        .unwrap(),
    );
    let offset = surface(GeomOffsetSurface::new(&basis, 0.75, false).unwrap());
    let evaluator = Surface::from_surface(&offset).unwrap();
    assert_eq!(evaluator.get_type(), SurfaceType::OffsetSurface);
    let bounds = evaluator
        .bounding_box_range(0.0, 1.0, 0.0, 1.0, 0.25)
        .unwrap();
    assert_eq!(bounds.gap(), 0.75);
}

#[test]
fn null_geometry_raises_standard_null_object() {
    let evaluator = Circle::new(Handle::default());
    EXPECT_THROW(evaluator.bounding_box(0.0), "Standard_NullObject");
}

#[test]
fn torus_patch_beyond_int_range_returns_a_void_box() {
    // floor(VMax / (Pi/4)) overflows int (INT_MIN on x86-64), so the section range is
    // empty and BndLib returns before its Enlarge.
    let torus = Torus3::new(gp::xoy().axis3(), 4.0, 1.0).unwrap();
    let mut bounds = BoundingBox3::default();
    bnd_lib::add_torus_patch(&torus, [0.0, 1.0, 0.0, 1e300], 0.5, &mut bounds).unwrap();
    assert!(bounds.is_void());
    assert_eq!(bounds.gap(), 0.0);
}

#[test]
fn tilted_infinite_plane_is_the_whole_space_without_enlarge() {
    let axis = Axis3::from_normal(pnt(0.0, 0.0, 0.0), dir(1.0, 1.0, 1.0)).unwrap();
    let plane = Plane::new(Handle::new(GeomPlane::new(axis)));
    let bounds = plane.bounding_box(1.0).unwrap();
    assert!(bounds.is_whole());
    assert_eq!(bounds.gap(), 0.0);
}

#[test]
fn trimmed_offset_curve_uses_the_offset_evaluator() {
    let circle = curve(GeomCircle::new(gp::xoy(), 2.0).unwrap());
    let offset = curve(GeomOffsetCurve::new(&circle, 0.5, gp::dz(), false).unwrap());
    let trimmed = curve(GeomTrimmedCurve::new(&offset, 0.0, 1.0, true, true).unwrap());
    let evaluator = Curve::from_curve(&trimmed).unwrap();
    assert_eq!(evaluator.get_type(), CurveType::OffsetCurve);
    // The offset circle has radius 2.5: its first-quadrant arc box is exact.
    let bounds = evaluator.bounding_box(0.0).unwrap();
    assert!((bounds.limits().x_max - 2.5).abs() <= precision::confusion());
}

#[test]
fn bezier_sample_count_shrinks_on_a_short_range() {
    let poles = array1(&[
        pnt(0.0, 0.0, 0.0),
        pnt(1.0, 2.0, 0.0),
        pnt(3.0, 2.0, 0.0),
        pnt(4.0, 0.0, 0.0),
    ]);
    let bezier = curve(GeomBezierCurve::new(&poles).unwrap());
    let adaptor = GeomAdaptorCurve::with_curve(&bezier).unwrap();
    // 2 * NbPoles = 8; a range of 0.5 gives RealToInt(0.5 * 8) + 1 = 5, and 0.1 the floor 5.
    assert_eq!(sampling::compute_nb_samples(&adaptor, 0.0, 1.0).unwrap(), 8);
    assert_eq!(sampling::compute_nb_samples(&adaptor, 0.0, 0.5).unwrap(), 5);
    assert_eq!(sampling::compute_nb_samples(&adaptor, 0.2, 0.3).unwrap(), 5);
}
