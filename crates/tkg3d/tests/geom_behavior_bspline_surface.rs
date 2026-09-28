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

//! Behaviour tests for every public member of Geom_BSplineSurface. The native
//! Geom_BSplineSurface_Test.cxx needs GeomAdaptor_Surface, so it is not mapped; the
//! numeric oracle (g3_bspline_surface*, surface kind 8) compares the members bit for
//! bit with OCCT, and these tests pin the preserved native quirks.
#![allow(non_snake_case)]
#[path = "../../tkg2d/tests/common/mod.rs"]
mod common;
#[path = "geom_behavior_bspline_surface/edits.rs"]
mod edits;
use common::{EXPECT_DOUBLE_EQ, EXPECT_NEAR};
use rustycad_tkernel::ncollection::{Array1, Array2};
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkg3d::geom::{
    GeomBSplineSurface, GeomCurve, GeomGeometry, GeomPlane, GeomSurface, KnotVector,
};
use rustycad_tkg3d::geom_eval::RepSurfaceDescFull;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Axis3, Direction3, Point3, Vector3};

/// A one-based array filled with `values`.
pub fn array<T: Copy + Default>(values: &[T]) -> Array1<'static, T> {
    let mut result: Array1<'static, T> = Array1::with_bounds(1, values.len() as i32).unwrap();
    result.as_mut_slice().copy_from_slice(values);
    result
}

/// A one-based `nu` x `nv` grid whose cell (i, j) is `cell(i, j)`.
pub fn grid<T: Copy + Default>(
    nu: i32,
    nv: i32,
    cell: impl Fn(i32, i32) -> T,
) -> Array2<'static, T> {
    let mut result: Array2<'static, T> = Array2::with_bounds(1, nu, 1, nv).unwrap();
    for i in 1..=nu {
        for j in 1..=nv {
            result.set_value(i, j, cell(i, j)).unwrap();
        }
    }
    result
}

/// A wavy pole over the unit grid.
pub fn wavy(i: i32, j: i32) -> Point3 {
    Point3::new(
        f64::from(i - 1),
        f64::from(j - 1),
        f64::from((i * j) % 3) * 0.25,
    )
}

/// The clamped bicubic Bezier patch of 4 x 4 wavy poles on [0, 1] x [0, 1].
pub fn bicubic() -> GeomBSplineSurface {
    let (knots, mults) = (array(&[0.0, 1.0]), array(&[4, 4]));
    let poles = grid(4, 4, wavy);
    let u: KnotVector = (&knots, &mults);
    GeomBSplineSurface::new(&poles, u, u, (3, 3), (false, false)).unwrap()
}

/// Cubic in U with an inner knot 0.5 (5 poles), quadratic Bezier in V (3 poles),
/// rational when `weights` is given.
pub fn nonuniform(weights: Option<fn(i32, i32) -> f64>) -> GeomBSplineSurface {
    let (uk, um) = (array(&[0.0, 0.5, 1.0]), array(&[4, 1, 4]));
    let (vk, vm) = (array(&[0.0, 1.0]), array(&[3, 3]));
    let poles = grid(5, 3, wavy);
    let Some(weight) = weights else {
        return GeomBSplineSurface::new(&poles, (&uk, &um), (&vk, &vm), (3, 2), (false, false))
            .unwrap();
    };
    let w = grid(5, 3, weight);
    GeomBSplineSurface::with_weights(&poles, &w, (&uk, &um), (&vk, &vm), (3, 2), (false, false))
        .unwrap()
}

/// Quadratic and U-periodic on knots 0..5 (5 poles), linear in V.
pub fn u_periodic() -> GeomBSplineSurface {
    let (uk, um) = (array(&[0.0, 1.0, 2.0, 3.0, 4.0, 5.0]), array(&[1; 6]));
    let (vk, vm) = (array(&[0.0, 1.0]), array(&[2, 2]));
    let poles = grid(5, 2, |i, j| {
        let angle = f64::from(i) * 1.2566370614359172;
        Point3::new(angle.cos(), angle.sin(), f64::from(j))
    });
    GeomBSplineSurface::new(&poles, (&uk, &um), (&vk, &vm), (2, 1), (true, false)).unwrap()
}

/// Assert that two points agree coordinate by coordinate within `tolerance`.
pub fn assert_same_point(a: Point3, b: Point3, tolerance: f64) {
    EXPECT_NEAR(a.x(), b.x(), tolerance);
    EXPECT_NEAR(a.y(), b.y(), tolerance);
    EXPECT_NEAR(a.z(), b.z(), tolerance);
}

#[test]
fn Geom_BSplineSurfaceBehavior__Constructors() {
    let surface = bicubic();
    assert_eq!((surface.nb_u_poles(), surface.nb_v_poles()), (4, 4));
    assert!(!surface.is_u_rational() && !surface.is_v_rational());
    // IsURational: the U isos (rows of weights, along V) are rational. Weights that
    // vary only with the U index make the V isos rational.
    let rational = nonuniform(Some(|i, _| 1.0 + f64::from(i) * 0.1));
    assert!(!rational.is_u_rational() && rational.is_v_rational());
    // Equal weights are not rational: Rational() compares them with Epsilon(|W|).
    let flat = nonuniform(Some(|_, _| 2.0));
    assert!(!flat.is_u_rational() && !flat.is_v_rational());
    let (knots, mults) = (array(&[0.0, 1.0]), array(&[4, 4]));
    let poles = grid(4, 4, wavy);
    let error = GeomBSplineSurface::new(
        &poles,
        (&knots, &mults),
        (&knots, &mults),
        (0, 3),
        (false, false),
    )
    .err()
    .unwrap();
    assert_eq!(error.exception_type(), "Standard_ConstructionError");
    let three = grid(3, 4, wavy);
    assert!(
        GeomBSplineSurface::new(
            &three,
            (&knots, &mults),
            (&knots, &mults),
            (3, 3),
            (false, false)
        )
        .is_err()
    );
    let zero = grid(4, 4, |i, j| if i == j { 0.0 } else { 1.0 });
    let refused = GeomBSplineSurface::with_weights(
        &poles,
        &zero,
        (&knots, &mults),
        (&knots, &mults),
        (3, 3),
        (false, false),
    );
    assert_eq!(
        refused.err().unwrap().exception_type(),
        "Standard_ConstructionError"
    );
    assert_eq!(GeomBSplineSurface::max_degree(), 25);
}

#[test]
fn Geom_BSplineSurfaceBehavior__CopyIsIndependent() {
    let surface = bicubic();
    let copy = GeomBSplineSurface::copy_of(&surface).unwrap();
    copy.set_pole(1, 1, Point3::new(9.0, 9.0, 9.0)).unwrap();
    assert_same_point(surface.pole(1, 1).unwrap(), wavy(1, 1), 0.0);
    let geometry: Handle<dyn GeomGeometry> = surface.copy().unwrap();
    let clone: Handle<GeomBSplineSurface> = geometry.cast();
    assert_eq!(clone.nb_u_poles(), 4);
    assert_same_point(clone.pole(2, 3).unwrap(), wavy(2, 3), 0.0);
}

#[test]
fn Geom_BSplineSurfaceBehavior__KnotGetters() {
    let surface = nonuniform(None);
    assert_eq!((surface.nb_u_knots(), surface.nb_v_knots()), (3, 2));
    assert_eq!((surface.u_degree(), surface.v_degree()), (3, 2));
    EXPECT_DOUBLE_EQ(surface.u_knot(2).unwrap(), 0.5);
    EXPECT_DOUBLE_EQ(surface.v_knot(2).unwrap(), 1.0);
    assert_eq!(surface.u_multiplicity(2).unwrap(), 1);
    assert_eq!(surface.v_multiplicity(1).unwrap(), 3);
    assert_eq!(
        surface.u_knot(4).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    assert_eq!(surface.u_knots().as_slice(), &[0.0, 0.5, 1.0]);
    assert_eq!(surface.v_knots().as_slice(), &[0.0, 1.0]);
    assert_eq!(surface.u_multiplicities().as_slice(), &[4, 1, 4]);
    assert_eq!(surface.v_multiplicities().as_slice(), &[3, 3]);
    assert_eq!(
        surface.u_knot_sequence().as_slice(),
        &[0.0, 0.0, 0.0, 0.0, 0.5, 1.0, 1.0, 1.0, 1.0]
    );
    assert_eq!(surface.v_knot_sequence().len(), 6);
    let mut knots: Array1<'static, f64> = Array1::with_bounds(1, 3).unwrap();
    surface.u_knots_into(&mut knots).unwrap();
    assert_eq!(knots.as_slice(), &[0.0, 0.5, 1.0]);
    assert!(surface.v_knots_into(&mut knots).is_err());
    let mut mults: Array1<'static, i32> = Array1::with_bounds(1, 2).unwrap();
    surface.v_multiplicities_into(&mut mults).unwrap();
    assert!(surface.u_multiplicities_into(&mut mults).is_err());
    let mut flat: Array1<'static, f64> = Array1::with_bounds(1, 6).unwrap();
    surface.v_knot_sequence_into(&mut flat).unwrap();
    assert!(surface.u_knot_sequence_into(&mut flat).is_err());
    assert_eq!(surface.first_u_knot_index().unwrap(), 1);
    assert_eq!(surface.last_u_knot_index().unwrap(), 3);
    assert_eq!(surface.first_v_knot_index().unwrap(), 1);
    assert_eq!(surface.last_v_knot_index().unwrap(), 2);
}

#[test]
fn Geom_BSplineSurfaceBehavior__KnotDistributionAndContinuity() {
    use rustycad_tkmath::geom_abs::BSplKnotDistribution;
    let surface = nonuniform(None);
    // Equally spaced knots with clamped ends and simple inner knots.
    assert_eq!(
        surface.u_knot_distribution(),
        BSplKnotDistribution::QuasiUniform
    );
    surface.set_u_knot(2, 0.4).unwrap();
    assert_eq!(
        surface.u_knot_distribution(),
        BSplKnotDistribution::NonUniform
    );
    assert_eq!(
        bicubic().v_knot_distribution(),
        BSplKnotDistribution::PiecewiseBezier
    );
    assert_eq!(
        u_periodic().u_knot_distribution(),
        BSplKnotDistribution::Uniform
    );
    // Continuity is the lower of the U and V smoothness: C2 at the simple inner U knot.
    assert_eq!(surface.continuity(), Shape::C2);
    assert_eq!(bicubic().continuity(), Shape::CN);
    assert!(surface.is_cn_u(2).unwrap() && !surface.is_cn_u(3).unwrap());
    assert!(surface.is_cn_v(7).unwrap());
    assert!(surface.is_cn_u(-1).is_err());
}

#[test]
fn Geom_BSplineSurfaceBehavior__PoleAndWeightGetters() {
    let surface = bicubic();
    assert_same_point(surface.pole(3, 2).unwrap(), wavy(3, 2), 0.0);
    assert_eq!(
        surface.pole(5, 1).unwrap_err().exception_type(),
        "Standard_OutOfRange"
    );
    let poles = surface.poles();
    assert_eq!((poles.upper_row(), poles.upper_col()), (4, 4));
    let mut target: Array2<'static, Point3> = Array2::with_bounds(1, 4, 1, 4).unwrap();
    surface.poles_into(&mut target).unwrap();
    assert_same_point(*target.value(4, 4).unwrap(), wavy(4, 4), 0.0);
    let mut small: Array2<'static, Point3> = Array2::with_bounds(1, 3, 1, 4).unwrap();
    assert_eq!(
        surface.poles_into(&mut small).unwrap_err().exception_type(),
        "Standard_DimensionError"
    );
    // Quirk: Weight() of a non-rational surface is 1 without any index check.
    EXPECT_DOUBLE_EQ(surface.weight(99, -3).unwrap(), 1.0);
    assert!(surface.weights().is_none());
    let mut weights: Array2<'static, f64> = Array2::with_bounds(1, 4, 1, 4).unwrap();
    surface.weights_into(&mut weights).unwrap();
    assert!(weights.iter().all(|w| *w == 1.0));
    let rational = nonuniform(Some(|i, j| f64::from(i + j)));
    EXPECT_DOUBLE_EQ(rational.weight(2, 3).unwrap(), 5.0);
    assert!(rational.weight(6, 1).is_err());
    assert_eq!(rational.weights().unwrap().upper_row(), 5);
    let sum: f64 = rational.weights_array(|w| w.iter().sum());
    assert_eq!(sum, 75.0);
}

#[test]
fn Geom_BSplineSurfaceBehavior__BoundsClosureAndPeriodicity() {
    let surface = nonuniform(None);
    assert_eq!(surface.bounds(), (0.0, 1.0, 0.0, 1.0));
    assert!(!surface.is_u_closed().unwrap() && !surface.is_v_closed().unwrap());
    assert!(!surface.is_u_periodic() && !surface.is_v_periodic());
    let periodic = u_periodic();
    assert_eq!(periodic.bounds(), (0.0, 5.0, 0.0, 1.0));
    assert!(periodic.is_u_periodic() && periodic.is_u_closed().unwrap());
    EXPECT_DOUBLE_EQ(periodic.u_period().unwrap(), 5.0);
    assert!(periodic.v_period().is_err());
    assert_eq!(periodic.first_u_knot_index().unwrap(), 1);
    assert_eq!(periodic.last_u_knot_index().unwrap(), 6);
}

#[test]
fn Geom_BSplineSurfaceBehavior__PeriodicNormalization() {
    let periodic = u_periodic();
    let (u, v) = periodic.periodic_normalization(7.5, 3.0).unwrap();
    EXPECT_DOUBLE_EQ(u, 2.5);
    EXPECT_DOUBLE_EQ(v, 3.0);
    let (u, _) = periodic.periodic_normalization(-1.0, 0.0).unwrap();
    EXPECT_DOUBLE_EQ(u, 4.0);
    assert_eq!(
        nonuniform(None).periodic_normalization(7.5, -2.0).unwrap(),
        (7.5, -2.0)
    );
}

#[test]
fn Geom_BSplineSurfaceBehavior__Evaluation() {
    let surface = bicubic();
    assert_same_point(surface.value(0.0, 0.0).unwrap(), wavy(1, 1), 1e-15);
    assert_same_point(surface.d0(1.0, 1.0).unwrap(), wavy(4, 4), 1e-15);
    let d1 = surface.d1(0.0, 0.0).unwrap();
    let edge = Vector3::between(wavy(1, 1), wavy(2, 1)).scaled(3.0);
    EXPECT_NEAR(d1.d1u.x(), edge.x(), 1e-14);
    EXPECT_NEAR(d1.d1u.z(), edge.z(), 1e-14);
    let d2 = surface.d2(0.3, 0.6).unwrap();
    let d3 = surface.d3(0.3, 0.6).unwrap();
    assert_eq!(d2.point, d3.point);
    let dn = surface.dn(0.3, 0.6, 1, 1).unwrap();
    EXPECT_NEAR(dn.z(), d2.d2uv.z(), 1e-13);
    let dn3 = surface.dn(0.3, 0.6, 2, 1).unwrap();
    EXPECT_NEAR(dn3.z(), d3.d3uuv.z(), 1e-12);
    let error = surface.dn(0.3, 0.6, 0, 0).unwrap_err();
    assert_eq!(error.exception_type(), "Geom_UndefinedDerivative");
}

#[test]
fn Geom_BSplineSurfaceBehavior__LocalEvaluation() {
    let surface = nonuniform(Some(|i, j| 1.0 + 0.2 * f64::from(i) + 0.1 * f64::from(j)));
    let span = ((1, 2), (1, 2));
    let (u, v) = (0.25, 0.4);
    assert_eq!(
        surface.local_value(u, v, span).unwrap(),
        surface.local_d0(u, v, span).unwrap()
    );
    assert_same_point(
        surface.local_d0(u, v, span).unwrap(),
        surface.d0(u, v).unwrap(),
        1e-14,
    );
    let d1 = surface.local_d1(u, v, span).unwrap();
    EXPECT_NEAR(d1.d1v.z(), surface.d1(u, v).unwrap().d1v.z(), 1e-13);
    let d2 = surface.local_d2(u, v, span).unwrap();
    EXPECT_NEAR(d2.d2u.y(), surface.d2(u, v).unwrap().d2u.y(), 1e-12);
    let d3 = surface.local_d3(u, v, span).unwrap();
    EXPECT_NEAR(d3.d3u.z(), surface.d3(u, v).unwrap().d3u.z(), 1e-11);
    let dn = surface.local_dn(u, v, span, (0, 2)).unwrap();
    EXPECT_NEAR(dn.z(), d2.d2v.z(), 1e-12);
    // The second U span evaluates its own polynomial outside [0, 0.5].
    let far = surface.local_d0(0.75, v, ((2, 3), (1, 2))).unwrap();
    assert_same_point(far, surface.d0(0.75, v).unwrap(), 1e-14);
}

#[test]
fn Geom_BSplineSurfaceBehavior__IsoCurves() {
    let surface = nonuniform(Some(|_, j| 1.0 + 0.5 * f64::from(j)));
    let u_iso = surface.u_iso_checked(0.0, true).unwrap();
    assert_eq!((u_iso.degree(), u_iso.nb_poles()), (2, 3));
    assert!(u_iso.is_rational());
    // V iso weights are constant along U (they vary only with j): CheckRational drops them.
    let v_iso = surface.v_iso_checked(0.2, true).unwrap();
    assert_eq!(
        (v_iso.degree(), v_iso.nb_poles(), v_iso.is_rational()),
        (3, 5, false)
    );
    assert!(surface.v_iso_checked(0.2, false).unwrap().is_rational());
    let iso: Handle<dyn GeomCurve> = surface.u_iso(0.7).unwrap();
    assert_same_point(
        iso.value(0.3).unwrap(),
        surface.value(0.7, 0.3).unwrap(),
        1e-14,
    );
    let iso: Handle<dyn GeomCurve> = surface.v_iso(0.3).unwrap();
    assert_same_point(
        iso.value(0.7).unwrap(),
        surface.value(0.7, 0.3).unwrap(),
        1e-14,
    );
}

#[test]
fn Geom_BSplineSurfaceBehavior__Locate() {
    let surface = nonuniform(None);
    assert_eq!(surface.locate_u(0.25, 1e-9, false).unwrap(), (1, 2));
    assert_eq!(surface.locate_u(0.5, 1e-9, false).unwrap(), (2, 2));
    assert_eq!(surface.locate_u(-1.0, 1e-9, false).unwrap(), (0, 1));
    assert_eq!(surface.locate_u(3.0, 1e-9, false).unwrap(), (3, 4));
    // With knot repetition the flat knots 0 0 0 1 1 1 are searched.
    assert_eq!(surface.locate_v(0.5, 1e-9, true).unwrap(), (3, 4));
    assert_eq!(surface.locate_v(-0.5e-9, 1e-9, false).unwrap(), (1, 1));
    assert_eq!(surface.locate_u(1.0 + 0.5e-9, 1e-9, false).unwrap(), (3, 3));
}

#[test]
fn Geom_BSplineSurfaceBehavior__Transform() {
    use rustycad_tkmath::gp::Transform3;
    let surface = bicubic();
    let mut shift = Transform3::default();
    shift.set_translation(Vector3::new(1.0, 2.0, 3.0));
    surface.transform(shift).unwrap();
    assert_same_point(
        surface.pole(1, 1).unwrap(),
        Point3::new(1.0, 2.0, 3.25),
        0.0,
    );
}

#[test]
fn Geom_BSplineSurfaceBehavior__Resolution() {
    let surface = bicubic();
    let (ur, vr) = surface.resolution(1e-3).unwrap();
    assert!(ur > 0.0 && vr > 0.0 && ur < 1e-3 && vr < 1e-3);
    // The inverse derivative bounds are cached: a second call scales the same values.
    let (ur2, vr2) = surface.resolution(2e-3).unwrap();
    EXPECT_DOUBLE_EQ(ur2, 2.0 * ur);
    EXPECT_DOUBLE_EQ(vr2, 2.0 * vr);
}

#[test]
fn Geom_BSplineSurfaceBehavior__EvalRepresentation() {
    let surface = Handle::new(bicubic());
    assert!(!surface.has_eval_representation() && surface.eval_representation().is_null());
    let frame = Axis3::from_normal(
        Point3::new(0.0, 0.0, 5.0),
        Direction3::new(0.0, 0.0, 1.0).unwrap(),
    )
    .unwrap();
    let plane: Handle<dyn GeomSurface> = Handle::new(GeomPlane::new(frame)).cast();
    surface
        .set_eval_representation(Handle::new(RepSurfaceDescFull::new(plane)).cast())
        .unwrap();
    assert!(surface.has_eval_representation());
    assert_same_point(
        surface.value(0.3, 0.4).unwrap(),
        Point3::new(0.3, 0.4, 5.0),
        0.0,
    );
    let copy: Handle<GeomBSplineSurface> = surface.copy().unwrap().cast();
    assert!(copy.has_eval_representation());
    let own = RepSurfaceDescFull::new(surface.cast());
    let error = surface
        .set_eval_representation(Handle::new(own).cast())
        .unwrap_err();
    assert_eq!(error.exception_type(), "Standard_ProgramError");
    surface.clear_eval_representation();
    assert!(!surface.has_eval_representation());
}

#[test]
fn Geom_BSplineSurfaceBehavior__DumpJson() {
    let mut stream = DumpStream::default();
    nonuniform(None).dump_json(&mut stream, -1).unwrap();
    let text = stream.text();
    assert!(
        text.starts_with("\"className\": \"Geom_BSplineSurface\""),
        "{text}"
    );
    assert!(
        text.contains("\"UDeg\": 3") && text.contains("\"VDeg\": 2"),
        "{text}"
    );
    // Weights.Size is dumped only for a rational surface.
    assert!(!text.contains("Weights.Size"), "{text}");
    let mut rational = DumpStream::default();
    nonuniform(Some(|i, _| f64::from(i)))
        .dump_json(&mut rational, -1)
        .unwrap();
    assert!(
        rational.text().contains("\"Weights.Size\": 15"),
        "{}",
        rational.text()
    );
}
