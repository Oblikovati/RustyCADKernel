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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_ArcFunction.hxx/.lxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPatch_ArcFunction`: the signed distance of a quadric along an arc of a surface
//! domain, the `TheFunction` of `IntPatch_TheSOnBounds`.
use super::failures::null_object;
use super::h_inter_tool::{IntPatchHInterTool, arc_of};
use crate::int_start::{TheArc, TheArcFunction};
use crate::int_surf::IntSurfQuadric;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::Adaptor3dSurface;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkgeombase::failure_capture::FailureCapture;
use rustycad_tkmath::gp::{Point2, Point3, Vector3};
use rustycad_tkmath::math::{Function, FunctionWithDerivative};

/// `IntPatch_ArcFunction`: `F(X)` is the distance of the quadric to the surface point of
/// the arc at `X`. The points of the saved states are kept (`GetStateNumber`). A
/// failure of an arc or surface evaluation inside a solver is kept and raised when the
/// solver returns (docs/geomalgo-design.md section 6).
///
/// ```
/// use rustycad_tkgeomalgo::int_patch::IntPatchArcFunction;
/// let f = IntPatchArcFunction::new();
/// assert!(f.valpoint(1).is_err());
/// ```
#[derive(Default)]
pub struct IntPatchArcFunction {
    my_arc: TheArc,
    my_surf: Adaptor3dSurfaceHandle,
    my_quad: IntSurfQuadric,
    ptsol: Point3,
    seqpt: Vec<Point3>,
    capture: FailureCapture,
}

impl IntPatchArcFunction {
    /// `IntPatch_ArcFunction()`: no arc, no surface, the default quadric.
    /// `IntPatchArcFunction::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `SetQuadric(Q)`: clears the saved points. `f.set_quadric(&q)`.
    pub fn set_quadric(&mut self, q: &IntSurfQuadric) {
        self.my_quad = *q;
        self.seqpt.clear();
    }

    /// `Set(A)`: the arc; clears the saved points. `f.set_arc(&arc)`.
    pub fn set_arc(&mut self, a: &TheArc) {
        self.my_arc = a.clone();
        self.seqpt.clear();
    }

    /// `Set(S)`: the surface; clears the saved points. `f.set_surface(&s)`.
    pub fn set_surface(&mut self, s: &Adaptor3dSurfaceHandle) {
        self.my_surf = s.clone();
        self.seqpt.clear();
    }

    fn surface_ref(&self) -> Result<&dyn Adaptor3dSurface, Failure> {
        self.my_surf
            .get()
            .ok_or_else(|| null_object("IntPatch_ArcFunction", "Adaptor3d_Surface"))
    }

    fn arc_point(&self, x: f64) -> Result<Point2, Failure> {
        arc_of(&self.my_arc, "IntPatch_ArcFunction")?.value(x)
    }

    /// `Value(X, F)`: stores the surface point and returns the quadric distance; always
    /// true natively. `f.value_at(0.5)?`.
    pub fn value_at(&mut self, x: f64) -> Result<f64, Failure> {
        let p2d: Point2 = self.arc_point(x)?;
        self.ptsol = self.surface_ref()?.d0(p2d.x(), p2d.y())?;
        Ok(self.my_quad.distance(self.ptsol))
    }

    /// The tangent of the arc on the surface at `x` (`v.SetLinearForm(d2d.X(), d1u,
    /// d2d.Y(), d1v)`), storing the surface point.
    fn tangent(&mut self, x: f64) -> Result<Vector3, Failure> {
        let (p2d, d2d) = arc_of(&self.my_arc, "IntPatch_ArcFunction")?.d1(x)?;
        let d1 = self.surface_ref()?.d1(p2d.x(), p2d.y())?;
        self.ptsol = d1.point;
        let mut v = Vector3::default();
        v.set_linear_form(d2d.x(), d1.d1u, d2d.y(), d1.d1v);
        Ok(v)
    }

    /// `Derivative(X, D)`: the derivative of the distance along the arc.
    /// `f.derivative_at(0.5)?`.
    pub fn derivative_at(&mut self, x: f64) -> Result<f64, Failure> {
        let v: Vector3 = self.tangent(x)?;
        Ok(v.dot(self.my_quad.gradient(self.ptsol)))
    }

    /// `Values(X, F, D)`: the distance and its derivative. For a quadric that is not set
    /// the native `ValAndGrad` writes nothing: `F` keeps the value `previous` and the
    /// gradient is null. `f.values_at(0.5, 0.0)?`.
    pub fn values_at(&mut self, x: f64, previous: f64) -> Result<(f64, f64), Failure> {
        let v1: Vector3 = self.tangent(x)?;
        let (f, v2) = self
            .my_quad
            .val_and_grad(self.ptsol)
            .unwrap_or((previous, Vector3::default()));
        Ok((f, v1.dot(v2)))
    }

    /// `NbSamples()`: the greatest of the U and V sample counts of the surface and the
    /// sample count of the arc. `f.nb_samples()?`.
    pub fn nb_samples(&self) -> Result<i32, Failure> {
        let su: i32 = IntPatchHInterTool::nb_samples_u(&self.my_surf, 0.0, 0.0)?;
        let sv: i32 = IntPatchHInterTool::nb_samples_v(&self.my_surf, 0.0, 0.0)?;
        Ok(su
            .max(sv)
            .max(IntPatchHInterTool::nb_samples_on_arc(&self.my_arc)?))
    }

    /// `GetStateNumber()`: saves the last computed point and returns its index.
    /// `f.get_state_number()`.
    pub fn get_state_number(&mut self) -> i32 {
        self.seqpt.push(self.ptsol);
        self.seqpt.len() as i32
    }

    /// `Valpoint(Index)`: a saved point; Standard_OutOfRange outside the saved ones.
    /// `f.valpoint(1)?`.
    pub fn valpoint(&self, index: i32) -> Result<Point3, Failure> {
        crate::int_start::failures::sequence_value(
            &self.seqpt,
            index,
            "IntPatch_ArcFunction::Valpoint",
        )
        .copied()
    }

    /// `Quadric()`. `f.quadric()`.
    pub fn quadric(&self) -> &IntSurfQuadric {
        &self.my_quad
    }

    /// `Arc()`. `f.arc()`.
    pub fn arc(&self) -> TheArc {
        self.my_arc.clone()
    }

    /// `Surface()`. `f.surface()`.
    pub fn surface(&self) -> Adaptor3dSurfaceHandle {
        self.my_surf.clone()
    }

    /// `LastComputedPoint()`: the point of the last evaluation. `f.last_computed_point()`.
    pub fn last_computed_point(&self) -> Point3 {
        self.ptsol
    }
}

impl Function for IntPatchArcFunction {
    fn value(&mut self, x: f64, result: &mut f64) -> bool {
        let value = self.value_at(x);
        self.capture.keep(value).map(|f| *result = f).is_some()
    }
    fn state_number(&mut self) -> i32 {
        self.get_state_number()
    }
}

impl FunctionWithDerivative for IntPatchArcFunction {
    fn derivative(&mut self, x: f64, derivative: &mut f64) -> bool {
        let value = self.derivative_at(x);
        self.capture.keep(value).map(|d| *derivative = d).is_some()
    }
    fn values(&mut self, x: f64, value: &mut f64, derivative: &mut f64) -> bool {
        let values = self.values_at(x, *value);
        self.capture
            .keep(values)
            .map(|(f, d)| (*value, *derivative) = (f, d))
            .is_some()
    }
}

/// `TheFunction` of `IntPatch_TheSOnBounds`.
impl TheArcFunction for IntPatchArcFunction {
    fn set_arc(&mut self, arc: &TheArc) {
        IntPatchArcFunction::set_arc(self, arc);
    }
    fn checked_value(&mut self, x: f64) -> Result<Option<f64>, Failure> {
        Ok(Some(self.value_at(x)?))
    }
    // The native caller's `F` is uninitialized where a quadric that is not set would
    // leave it; 0 stands for it.
    fn checked_values(&mut self, x: f64) -> Result<Option<(f64, f64)>, Failure> {
        Ok(Some(self.values_at(x, 0.0)?))
    }
    fn nb_samples(&self) -> Result<i32, Failure> {
        IntPatchArcFunction::nb_samples(self)
    }
    fn get_state_number(&mut self) -> i32 {
        IntPatchArcFunction::get_state_number(self)
    }
    fn valpoint(&self, index: i32) -> Result<Point3, Failure> {
        IntPatchArcFunction::valpoint(self, index)
    }
    fn quadric(&self) -> &IntSurfQuadric {
        IntPatchArcFunction::quadric(self)
    }
    fn surface(&self) -> Adaptor3dSurfaceHandle {
        IntPatchArcFunction::surface(self)
    }
    fn last_computed_point(&self) -> Point3 {
        self.ptsol
    }
    fn take_failure(&mut self) -> Result<(), Failure> {
        self.capture.take()
    }
}
