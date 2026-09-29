// Copyright (c) 1997-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/Approx/Approx_SweepFunction.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Approx_SweepFunction`: the section of a sweep as a function of the path
//! parameter, which `Approx_SweepApproximation` approximates.
use crate::native_failure::failure;
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::{Failure, StandardTransient, Transient};
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};

/// The Standard_NotImplemented of a member the implementation does not provide.
fn not_implemented(member: &str) -> Failure {
    failure(
        "Standard_NotImplemented",
        &format!("Approx_SweepFunction::{member}"),
    )
}

/// The section and its first derivatives: poles, 2D poles and weights, each with its
/// derivative (`D1` arguments).
pub struct SweepD1<'a, 'b> {
    /// `Poles`.
    pub poles: &'a mut Array1<'b, Point3>,
    /// `DPoles`.
    pub d_poles: &'a mut Array1<'b, Vector3>,
    /// `Poles2d`.
    pub poles2d: &'a mut Array1<'b, Point2>,
    /// `DPoles2d`.
    pub d_poles2d: &'a mut Array1<'b, Vector2>,
    /// `Weigths`.
    pub weights: &'a mut Array1<'b, f64>,
    /// `DWeigths`.
    pub d_weights: &'a mut Array1<'b, f64>,
}

/// The second derivatives added to [`SweepD1`] (`D2` arguments).
pub struct SweepD2<'a, 'b> {
    /// The value and first derivatives.
    pub d1: SweepD1<'a, 'b>,
    /// `D2Poles`.
    pub d2_poles: &'a mut Array1<'b, Vector3>,
    /// `D2Poles2d`.
    pub d2_poles2d: &'a mut Array1<'b, Vector2>,
    /// `D2Weigths`.
    pub d2_weights: &'a mut Array1<'b, f64>,
}

/// `Approx_SweepFunction`: the abstract class. Shared through handles, so every
/// member takes `&self` (an implementation keeps its evaluation state behind a
/// lock). The members with a native body raise Standard_NotImplemented unless
/// overridden.
///
/// ```ignore
/// struct Section { /* ... */ }
/// impl SweepFunction for Section { /* d0, nb_2d_curves, section_shape... */ }
/// let approx = SweepApproximation::new(&Handle::new(Section::new()).cast());
/// ```
pub trait SweepFunction: Transient {
    /// `D0(Param, First, Last, Poles, Poles2d, Weigths)`: the section at `param` of
    /// the interval `[first, last]`; false when it cannot be computed.
    fn d0(
        &self,
        param: f64,
        first: f64,
        last: f64,
        poles: &mut Array1<'_, Point3>,
        poles2d: &mut Array1<'_, Point2>,
        weights: &mut Array1<'_, f64>,
    ) -> Result<bool, Failure>;

    /// `D1(Param, First, Last, Poles, DPoles, Poles2d, DPoles2d, Weigths,
    /// DWeigths)`; Standard_NotImplemented by default.
    fn d1(
        &self,
        param: f64,
        first: f64,
        last: f64,
        values: SweepD1<'_, '_>,
    ) -> Result<bool, Failure> {
        let _ = (param, first, last, values);
        Err(not_implemented("D1"))
    }

    /// `D2(...)`: up to the second derivatives; Standard_NotImplemented by default.
    fn d2(
        &self,
        param: f64,
        first: f64,
        last: f64,
        values: SweepD2<'_, '_>,
    ) -> Result<bool, Failure> {
        let _ = (param, first, last, values);
        Err(not_implemented("D2"))
    }

    /// `Nb2dCurves()`: the 2D curves of the section.
    fn nb_2d_curves(&self) -> Result<i32, Failure>;

    /// `SectionShape(NbPoles, NbKnots, Degree)`.
    fn section_shape(&self) -> Result<(i32, i32, i32), Failure>;

    /// `Knots(TKnots)`: the knots of the section.
    fn knots(&self, knots: &mut Array1<'_, f64>) -> Result<(), Failure>;

    /// `Mults(TMults)`: the multiplicities of the section.
    fn mults(&self, mults: &mut Array1<'_, i32>) -> Result<(), Failure>;

    /// `IsRational()`: whether the section has weights.
    fn is_rational(&self) -> Result<bool, Failure>;

    /// `NbIntervals(S)`: the intervals of continuity `shape` of the path.
    fn nb_intervals(&self, shape: Shape) -> Result<i32, Failure>;

    /// `Intervals(T, S)`: their bounds.
    fn intervals(&self, t: &mut Array1<'_, f64>, shape: Shape) -> Result<(), Failure>;

    /// `SetInterval(First, Last)`: the interval the next evaluations belong to.
    fn set_interval(&self, first: f64, last: f64) -> Result<(), Failure>;

    /// `Resolution(Index, Tol, TolU, TolV)`: the parametric tolerances of the 2D curve
    /// `index`; Standard_NotImplemented by default.
    fn resolution(&self, index: i32, tol: f64) -> Result<(f64, f64), Failure> {
        let _ = (index, tol);
        Err(not_implemented("Resolution"))
    }

    /// `GetTolerance(BoundTol, SurfTol, AngleTol, Tol3d)`: the 3D tolerance of each
    /// pole of the section.
    fn get_tolerance(
        &self,
        bound_tol: f64,
        surf_tol: f64,
        angle_tol: f64,
        tol3d: &mut Array1<'_, f64>,
    ) -> Result<(), Failure>;

    /// `SetTolerance(Tol3d, Tol2d)`: the precision the function must reach.
    fn set_tolerance(&self, tol3d: f64, tol2d: f64) -> Result<(), Failure>;

    /// `BarycentreOfSurf()`; Standard_NotImplemented by default.
    fn barycentre_of_surf(&self) -> Result<Point3, Failure> {
        Err(not_implemented("BarycentreOfSurf"))
    }

    /// `MaximalSection()`: the largest section length; Standard_NotImplemented by
    /// default.
    fn maximal_section(&self) -> Result<f64, Failure> {
        Err(not_implemented("MaximalSection()"))
    }

    /// `GetMinimalWeight(Weigths)`: the smallest weight of each pole;
    /// Standard_NotImplemented by default.
    fn get_minimal_weight(&self, weights: &mut Array1<'_, f64>) -> Result<(), Failure> {
        let _ = weights;
        Err(not_implemented("GetMinimalWeight"))
    }
}

impl_transient_interface!(dyn SweepFunction, "Approx_SweepFunction", StandardTransient);
