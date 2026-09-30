// Created on: 1993-04-07
// Created by: Laurent BUCHARD
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
// Rust translation of IntCurveSurface_TheQuadCurvExactHInter.hxx/.cxx,
// IntCurveSurface_TheQuadCurvFuncOfTheQuadCurvExactHInter.hxx/.cxx and
// IntCurveSurface_QuadricCurveExactInterUtils.pxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The intersection of a curve with a quadric by the roots of the signed distance,
//! generic over the curve and surface tools (docs/geomalgo-design.md section 8).
use super::failures::out_of_range;
use super::tools::{InterCurveTool, InterSurfaceTool};
use crate::int_surf::IntSurfQuadric;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::failure_capture::FailureCapture;
use rustycad_tkmath::geom_abs::{Shape, SurfaceType};
use rustycad_tkmath::gp::{Point3, Vector3};
use rustycad_tkmath::math::{Function, FunctionAllRoots, FunctionSample, FunctionWithDerivative};
use std::marker::PhantomData;

/// `EPSX`, `EPSDIST` and `EPSNUL` of the root search.
const EPS: [f64; 3] = [0.00000000000001, 0.00000001, 0.00000001];

/// The signed distance from a quadric to the point at a curve parameter, with its
/// derivative. `IntCurveSurface_TheQuadCurvFuncOfTheQuadCurvExactHInter` is
/// [`TheQuadCurvFuncOfTheQuadCurvExactHInter`](super::TheQuadCurvFuncOfTheQuadCurvExactHInter).
///
/// A curve exception inside a callback is kept and reported as `false`, as the IntImp
/// functions do (docs/geomalgo-design.md section 6); the inherent members return it.
///
/// ```
/// use rustycad_tkgeomalgo::int_curve_surface::{HCurve, TheQuadCurvFuncOfTheQuadCurvExactHInter};
/// use rustycad_tkgeomalgo::int_surf::IntSurfQuadric;
/// fn distance(q: &IntSurfQuadric, c: &HCurve) -> f64 {
///     TheQuadCurvFuncOfTheQuadCurvExactHInter::new(q, c).value(0.5).unwrap()
/// }
/// ```
pub struct QuadCurvFunc<C: InterCurveTool> {
    my_quadric: IntSurfQuadric,
    my_curve: C::Curve,
    failure: FailureCapture,
}

impl<C: InterCurveTool> QuadCurvFunc<C> {
    /// Constructor (Q, C); `TheQuadCurvFuncOfTheQuadCurvExactHInter::new(&q, &c)`.
    pub fn new(q: &IntSurfQuadric, c: &C::Curve) -> Self {
        Self {
            my_quadric: *q,
            my_curve: c.clone(),
            failure: FailureCapture::default(),
        }
    }
    /// `Value(Param, F)`: the signed distance of the curve point to the quadric;
    /// `function.value(0.5)?`.
    pub fn value(&mut self, param: f64) -> Result<f64, Failure> {
        Ok(self.my_quadric.distance(C::value(&self.my_curve, param)?))
    }
    /// `Derivative(Param, D)`: the curve tangent along the quadric gradient;
    /// `function.derivative(0.5)?`.
    pub fn derivative(&mut self, param: f64) -> Result<f64, Failure> {
        let (p, t): (Point3, Vector3) = C::d1(&self.my_curve, param)?;
        Ok(t.dot(self.my_quadric.gradient(p)))
    }
    /// `Values(Param, F, D)`: both, from one `ValAndGrad`; an undefined quadric
    /// leaves `f` unchanged and gives a null gradient, as natively;
    /// `function.values(0.5, &mut f)?`.
    pub fn values(&mut self, param: f64, f: &mut f64) -> Result<f64, Failure> {
        let (p, t): (Point3, Vector3) = C::d1(&self.my_curve, param)?;
        let mut grad = Vector3::new(0.0, 0.0, 0.0);
        if let Some((value, gradient)) = self.my_quadric.val_and_grad(p) {
            (*f, grad) = (value, gradient);
        }
        Ok(t.dot(grad))
    }
    /// The first failure a solver callback raised since the last call.
    fn take_failure(&mut self) -> Result<(), Failure> {
        self.failure.take()
    }
}

impl<C: InterCurveTool> Function for QuadCurvFunc<C> {
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        let value = QuadCurvFunc::value(self, x);
        self.failure.keep(value).map(|v| *result = v).is_some()
    }
}

impl<C: InterCurveTool> FunctionWithDerivative for QuadCurvFunc<C> {
    fn derivative(&mut self, x: f64, derivative: &mut f64) -> bool {
        let value = QuadCurvFunc::derivative(self, x);
        self.failure.keep(value).map(|d| *derivative = d).is_some()
    }
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        let d = QuadCurvFunc::values(self, x, value);
        self.failure.keep(d).map(|d| *derivative = d).is_some()
    }
}

/// The roots (points and intervals) of the distance function of a curve and a
/// quadric patch. `IntCurveSurface_TheQuadCurvExactHInter` is
/// [`TheQuadCurvExactHInter`](super::TheQuadCurvExactHInter).
///
/// ```
/// use rustycad_tkgeomalgo::int_curve_surface::{HCurve, TheQuadCurvExactHInter};
/// use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
/// fn roots(s: &Adaptor3dSurfaceHandle, c: &HCurve) -> i32 {
///     TheQuadCurvExactHInter::new(s, c).map(|e| e.nb_roots()).unwrap_or(-1)
/// }
/// ```
pub struct QuadCurvExact<S: InterSurfaceTool, C: InterCurveTool> {
    nbpnts: i32,
    pnts: Vec<f64>,
    nbintv: i32,
    intv: Vec<f64>,
    tools: PhantomData<fn() -> (S, C)>,
}

impl<S: InterSurfaceTool, C: InterCurveTool> QuadCurvExact<S, C> {
    /// Constructor (S, C): the roots on every C1 interval of the curve; not done
    /// (-1 roots) for a surface that is not a plane, cylinder, cone or sphere, or when
    /// a root search fails; `TheQuadCurvExactHInter::new(&s, &c)?`.
    pub fn new(surface: &S::Surface, curve: &C::Curve) -> Result<Self, Failure> {
        let mut exact = Self {
            nbpnts: -1,
            pnts: Vec::new(),
            nbintv: -1,
            intv: Vec::new(),
            tools: PhantomData,
        };
        exact.perform_intersection(surface, curve)?;
        Ok(exact)
    }
    /// `PerformIntersection` of QuadricCurveExactInterUtils.
    fn perform_intersection(
        &mut self,
        surface: &S::Surface,
        curve: &C::Curve,
    ) -> Result<(), Failure> {
        let Some(quadric) = quadric_of::<S>(surface)? else {
            return Ok(());
        };
        let nb_intervals: i32 = C::nb_intervals(curve, Shape::C1)?;
        let mut intervals: Array1<'static, f64> = Array1::with_bounds(1, nb_intervals + 1)
            .map_err(|e| e.into_failure("NCollection_Array1"))?;
        C::intervals(curve, &mut intervals, Shape::C1)?;
        for ii in 1..=nb_intervals {
            if !self.interval_roots(&quadric, curve, [intervals[ii], intervals[ii + 1]])? {
                return Ok(());
            }
        }
        self.nbpnts = self.pnts.len() as i32;
        self.nbintv = (self.intv.len() / 2) as i32;
        Ok(())
    }
    /// The roots on `[u1, u2]`, appended; false when the search is not done.
    fn interval_roots(
        &mut self,
        quadric: &IntSurfQuadric,
        curve: &C::Curve,
        [u1, u2]: [f64; 2],
    ) -> Result<bool, Failure> {
        let sample = FunctionSample::new(u1, u2, C::nb_samples(curve, u1, u2)?);
        let mut function: QuadCurvFunc<C> = QuadCurvFunc::new(quadric, curve);
        let roots = FunctionAllRoots::new(&mut function, &sample, EPS);
        function.take_failure()?;
        let roots: FunctionAllRoots = roots?;
        if !roots.is_done() {
            return Ok(false);
        }
        for i in 1..=roots.number_of_points()? {
            self.pnts.push(roots.point(i)?);
        }
        for i in 1..=roots.number_of_intervals()? {
            self.intv.extend(roots.interval(i)?);
        }
        Ok(true)
    }
    /// `IsDone()`; `exact.is_done()`.
    pub fn is_done(&self) -> bool {
        self.nbpnts != -1
    }
    /// `NbRoots()`: -1 when not done; `exact.nb_roots()`.
    pub fn nb_roots(&self) -> i32 {
        self.nbpnts
    }
    /// `Root(Index)`: Standard_OutOfRange outside the roots; `exact.root(1)?`.
    pub fn root(&self, index: i32) -> Result<f64, Failure> {
        sequence_value(
            &self.pnts,
            index,
            "IntCurveSurface_TheQuadCurvExactHInter::Root",
        )
    }
    /// `NbIntervals()`: -1 when not done; `exact.nb_intervals()`.
    pub fn nb_intervals(&self) -> i32 {
        self.nbintv
    }
    /// `Intervals(Index, U1, U2)`: the curve segment of interval `index`;
    /// Standard_OutOfRange outside them; `let (u1, u2) = exact.intervals(1)?;`.
    pub fn intervals(&self, index: i32) -> Result<(f64, f64), Failure> {
        let index2: i32 = index + index - 1;
        let member = "IntCurveSurface_TheQuadCurvExactHInter::Intervals";
        Ok((
            sequence_value(&self.intv, index2, member)?,
            sequence_value(&self.intv, index2 + 1, member)?,
        ))
    }
}

/// The quadric of a plane, cylinder, cone or sphere patch, `None` otherwise.
fn quadric_of<S: InterSurfaceTool>(
    surface: &S::Surface,
) -> Result<Option<IntSurfQuadric>, Failure> {
    let mut quadric = IntSurfQuadric::new();
    match S::get_type(surface)? {
        SurfaceType::Plane => quadric.set_plane(&S::plane(surface)?),
        SurfaceType::Cylinder => quadric.set_cylinder(&S::cylinder(surface)?),
        SurfaceType::Cone => quadric.set_cone(&S::cone(surface)?),
        SurfaceType::Sphere => quadric.set_sphere(&S::sphere(surface)?),
        _ => return Ok(None),
    }
    Ok(Some(quadric))
}

/// `NCollection_Sequence<double>::operator()(index)`.
fn sequence_value(items: &[f64], index: i32, member: &str) -> Result<f64, Failure> {
    let found = usize::try_from(index - 1).ok().and_then(|at| items.get(at));
    found
        .copied()
        .ok_or_else(|| out_of_range(member, index, items.len()))
}
