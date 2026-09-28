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

//! The dimension of a point-curve extrema computation: the point, vector and curve
//! adaptor types, and the few behaviors in which ExtremaPC2d and ExtremaPC differ.
//!
//! ExtremaPC2d evaluates through `Adaptor2d_Curve2d` and ExtremaPC through
//! `Adaptor3d_Curve`; apart from those types their distance functions and grid
//! evaluators are the same code, which [`PcSpace`] lets the port share.
use super::grid::GridFailurePolicy;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped};
use rustycad_tkg2d::adaptor2d::Adaptor2dCurve2d;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};
use rustycad_tkmath::math_utils::Domain1D;
use std::fmt::Debug;

/// A gp point of the extrema packages (`gp_Pnt2d` or `gp_Pnt`).
pub trait PcPoint: Copy + Default + Debug + PartialEq + 'static {
    /// The matching gp vector (`gp_Vec2d` or `gp_Vec`).
    type Vector: PcVector;
    /// `SquareDistance(theOther)`; `point.square_distance_to(&other)`.
    fn square_distance_to(&self, other: &Self) -> f64;
    /// `gp_Vec(*this, theTarget)`; `point.vector_to(&target)`.
    fn vector_to(&self, target: &Self) -> Self::Vector;
    /// True when every coordinate is finite and not beyond Precision::Infinite()
    /// (`ExtremaPC2d::IsFinitePoint`, `ExtremaPC::IsFinitePoint`); `point.is_finite_point()`.
    fn is_finite_point(&self) -> bool;
}

// One coordinate of IsFinitePoint.
fn is_finite_coordinate(value: f64) -> bool {
    value.is_finite() && !rustycad_tkernel::precision::is_infinite(value)
}

/// A gp vector of the extrema packages (`gp_Vec2d` or `gp_Vec`).
pub trait PcVector: Copy + Debug + 'static {
    /// `Dot(theOther)`; `vector.dot_with(&other)`.
    fn dot_with(&self, other: &Self) -> f64;
    /// `Magnitude()`; `vector.length()`.
    fn length(&self) -> f64;
}

impl PcPoint for Point2 {
    type Vector = Vector2;
    fn square_distance_to(&self, other: &Self) -> f64 {
        self.square_distance(*other)
    }
    fn vector_to(&self, target: &Self) -> Vector2 {
        Vector2::between(*self, *target)
    }
    fn is_finite_point(&self) -> bool {
        is_finite_coordinate(self.x()) && is_finite_coordinate(self.y())
    }
}

impl PcPoint for Point3 {
    type Vector = Vector3;
    fn square_distance_to(&self, other: &Self) -> f64 {
        self.square_distance(other)
    }
    fn vector_to(&self, target: &Self) -> Vector3 {
        Vector3::between(*self, *target)
    }
    fn is_finite_point(&self) -> bool {
        is_finite_coordinate(self.x())
            && is_finite_coordinate(self.y())
            && is_finite_coordinate(self.z())
    }
}

impl PcVector for Vector2 {
    fn dot_with(&self, other: &Self) -> f64 {
        self.dot(*other)
    }
    fn length(&self) -> f64 {
        self.magnitude()
    }
}

impl PcVector for Vector3 {
    fn dot_with(&self, other: &Self) -> f64 {
        self.dot(*other)
    }
    fn length(&self) -> f64 {
        self.magnitude()
    }
}

/// A point with its first derivative.
pub type PcD1<P> = (P, <P as PcPoint>::Vector);
/// A point with its first two derivatives.
pub type PcD2<P> = (P, <P as PcPoint>::Vector, <P as PcPoint>::Vector);

/// The curve adaptor of one dimension, as the numerical evaluators use it.
pub trait PcSpace: 'static {
    /// The point type.
    type Point: PcPoint;
    /// The adaptor trait object (`dyn Adaptor2dCurve2d` or `dyn Adaptor3dCurve`).
    type Curve: ?Sized + RuntimeTyped + 'static;
    /// How the grid evaluator reports failures of its classification.
    const FAILURES: GridFailurePolicy;
    /// The native grid evaluator class, for exception messages.
    const GRID_EVALUATOR_NAME: &'static str;
    /// `theCurve.Value(U)`.
    fn value(curve: &Self::Curve, u: f64) -> Result<Self::Point, Failure>;
    /// `theCurve.D1(U, P, V1)`.
    fn d1(curve: &Self::Curve, u: f64) -> Result<PcD1<Self::Point>, Failure>;
    /// `theCurve.D2(U, P, V1, V2)`.
    fn d2(curve: &Self::Curve, u: f64) -> Result<PcD2<Self::Point>, Failure>;
    /// `FirstParameter()`.
    fn first_parameter(curve: &Self::Curve) -> Result<f64, Failure>;
    /// `LastParameter()`.
    fn last_parameter(curve: &Self::Curve) -> Result<f64, Failure>;
    /// `NbIntervals(GeomAbs_C2)`.
    fn nb_c2_intervals(curve: &Self::Curve) -> Result<i32, Failure>;
    /// `Intervals(theArray, GeomAbs_C2)`.
    fn c2_intervals(curve: &Self::Curve, bounds: &mut Array1<'_, f64>) -> Result<(), Failure>;
    /// `Trim(First, Last, Tol)`.
    fn trim(
        curve: &Self::Curve,
        first: f64,
        last: f64,
        tol: f64,
    ) -> Result<Handle<Self::Curve>, Failure>;
    /// `IsPeriodic()`.
    fn is_periodic(curve: &Self::Curve) -> Result<bool, Failure>;
    /// `Period()`.
    fn period(curve: &Self::Curve) -> Result<f64, Failure>;
    /// `Extrema_CurveTool::DeflCurvIntervals` or `Extrema_Curve2dTool::DeflCurvIntervals`.
    fn defl_curv_intervals(curve: &Self::Curve) -> Result<Vec<f64>, Failure>;
    /// `constantSquareDistance` of the grid evaluator: the square distance when the
    /// geometry proves it constant, `None` otherwise (and on any failure).
    fn constant_square_distance(
        curve: &Self::Curve,
        point: &Self::Point,
        domain: &Domain1D,
    ) -> Option<f64>;
}

/// The ExtremaPC2d dimension.
pub struct Space2d;

/// The ExtremaPC dimension.
pub struct Space3d;

impl PcSpace for Space2d {
    type Point = Point2;
    type Curve = dyn Adaptor2dCurve2d;
    const FAILURES: GridFailurePolicy = GridFailurePolicy::CatchAll;
    const GRID_EVALUATOR_NAME: &'static str = "ExtremaPC2d_GridEvaluator";
    fn value(curve: &Self::Curve, u: f64) -> Result<Point2, Failure> {
        curve.value(u)
    }
    fn d1(curve: &Self::Curve, u: f64) -> Result<PcD1<Point2>, Failure> {
        curve.d1(u)
    }
    fn d2(curve: &Self::Curve, u: f64) -> Result<PcD2<Point2>, Failure> {
        curve.d2(u)
    }
    fn first_parameter(curve: &Self::Curve) -> Result<f64, Failure> {
        curve.first_parameter()
    }
    fn last_parameter(curve: &Self::Curve) -> Result<f64, Failure> {
        curve.last_parameter()
    }
    fn nb_c2_intervals(curve: &Self::Curve) -> Result<i32, Failure> {
        curve.nb_intervals(Shape::C2)
    }
    fn c2_intervals(curve: &Self::Curve, bounds: &mut Array1<'_, f64>) -> Result<(), Failure> {
        curve.intervals(bounds, Shape::C2)
    }
    fn trim(
        curve: &Self::Curve,
        first: f64,
        last: f64,
        tol: f64,
    ) -> Result<Handle<Self::Curve>, Failure> {
        curve.trim(first, last, tol)
    }
    fn is_periodic(curve: &Self::Curve) -> Result<bool, Failure> {
        curve.is_periodic()
    }
    fn period(curve: &Self::Curve) -> Result<f64, Failure> {
        curve.period()
    }
    fn defl_curv_intervals(curve: &Self::Curve) -> Result<Vec<f64>, Failure> {
        <crate::extrema::Curve2dTool as crate::extrema::ExtremaCurveTool>::defl_curv_intervals(
            curve,
        )
    }
    fn constant_square_distance(
        curve: &Self::Curve,
        point: &Point2,
        domain: &Domain1D,
    ) -> Option<f64> {
        super::constant_distance::constant_square_distance_2d(curve, *point, domain).ok()?
    }
}

impl PcSpace for Space3d {
    type Point = Point3;
    type Curve = dyn Adaptor3dCurve;
    const FAILURES: GridFailurePolicy = GridFailurePolicy::CatchSigns;
    const GRID_EVALUATOR_NAME: &'static str = "ExtremaPC_GridEvaluator";
    fn value(curve: &Self::Curve, u: f64) -> Result<Point3, Failure> {
        curve.value(u)
    }
    fn d1(curve: &Self::Curve, u: f64) -> Result<PcD1<Point3>, Failure> {
        curve.d1(u)
    }
    fn d2(curve: &Self::Curve, u: f64) -> Result<PcD2<Point3>, Failure> {
        curve.d2(u)
    }
    fn first_parameter(curve: &Self::Curve) -> Result<f64, Failure> {
        curve.first_parameter()
    }
    fn last_parameter(curve: &Self::Curve) -> Result<f64, Failure> {
        curve.last_parameter()
    }
    fn nb_c2_intervals(curve: &Self::Curve) -> Result<i32, Failure> {
        curve.nb_intervals(Shape::C2)
    }
    fn c2_intervals(curve: &Self::Curve, bounds: &mut Array1<'_, f64>) -> Result<(), Failure> {
        curve.intervals(bounds, Shape::C2)
    }
    fn trim(
        curve: &Self::Curve,
        first: f64,
        last: f64,
        tol: f64,
    ) -> Result<Handle<Self::Curve>, Failure> {
        curve.trim(first, last, tol)
    }
    fn is_periodic(curve: &Self::Curve) -> Result<bool, Failure> {
        curve.is_periodic()
    }
    fn period(curve: &Self::Curve) -> Result<f64, Failure> {
        curve.period()
    }
    fn defl_curv_intervals(curve: &Self::Curve) -> Result<Vec<f64>, Failure> {
        <crate::extrema::CurveTool as crate::extrema::ExtremaCurveTool>::defl_curv_intervals(curve)
    }
    fn constant_square_distance(
        curve: &Self::Curve,
        point: &Point3,
        domain: &Domain1D,
    ) -> Option<f64> {
        super::constant_distance::constant_square_distance_3d(curve, *point, domain).ok()?
    }
}
