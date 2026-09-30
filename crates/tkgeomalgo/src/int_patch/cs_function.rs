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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntPatch/IntPatch_CSFunction.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntPatch_CSFunction`: the distance between a surface and a curve on another surface,
//! the function of `IntPatch_CurvIntSurf`.
use super::h_curve2d_tool::IntPatchHCurve2dTool;
use crate::int_imp::captured::captured_function_set;
use crate::int_imp::{HSurfaceTool, TheFunction, ThePSurfaceTool};
use crate::int_start::TheArc;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::adaptor3d::h_surface_tool::Adaptor3dSurfaceHandle;
use rustycad_tkgeombase::failure_capture::FailureCapture;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};
use rustycad_tkmath::math::{Matrix, Vector};

/// `IntPatch_CSFunction`: `F(u, v, w) = S1(u, v) - S2(C(w))`, three equations in three
/// variables, with the last point and squared distance. The native class keeps
/// pointers to the caller's handles; the port keeps copies of the handles.
///
/// ```ignore
/// let mut f = IntPatchCSFunction::new(&s1, &arc, &s2);
/// f.value(&x, &mut fx)?;
/// ```
pub struct IntPatchCSFunction {
    surface1: Adaptor3dSurfaceHandle,
    curve: TheArc,
    surface2: Adaptor3dSurfaceHandle,
    p: Point3,
    f: f64,
    failure: FailureCapture,
}

impl Clone for IntPatchCSFunction {
    /// The native copy: the handles, the last point and root; no pending failure.
    fn clone(&self) -> Self {
        Self {
            surface1: self.surface1.clone(),
            curve: self.curve.clone(),
            surface2: self.surface2.clone(),
            p: self.p,
            f: self.f,
            failure: FailureCapture::default(),
        }
    }
}

impl IntPatchCSFunction {
    /// `IntPatch_CSFunction(S1, C, S2)`: `s1` is the surface searched, `c` a curve on
    /// `s2`. The point is left at its default origin, the root at 0.
    /// `IntPatchCSFunction::new(&s1, &c, &s2)`.
    pub fn new(s1: &Adaptor3dSurfaceHandle, c: &TheArc, s2: &Adaptor3dSurfaceHandle) -> Self {
        Self {
            surface1: s1.clone(),
            curve: c.clone(),
            surface2: s2.clone(),
            p: Point3::default(),
            f: 0.0,
            failure: FailureCapture::default(),
        }
    }

    /// `NbVariables()`: 3. `f.nb_variables()`.
    pub fn nb_variables(&self) -> i32 {
        3
    }

    /// `NbEquations()`: 3. `f.nb_equations()`.
    pub fn nb_equations(&self) -> i32 {
        3
    }

    /// Writes `Psurf - Pcurv`, keeps its squared norm and the middle point.
    fn store(&mut self, psurf: Point3, pcurv: Point3, f: &mut Vector<'_>) {
        f[1] = psurf.x() - pcurv.x();
        f[2] = psurf.y() - pcurv.y();
        f[3] = psurf.z() - pcurv.z();
        self.f = f[1] * f[1] + f[2] * f[2] + f[3] * f[3];
        self.p = Point3::from((psurf.xyz() + pcurv.xyz()) / 2.0);
    }

    /// `Value(X, F)`: always true natively. `f.value(&x, &mut fx)?`.
    pub fn value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> Result<bool, Failure> {
        let psurf: Point3 = HSurfaceTool::value(&self.surface1, x[1], x[2])?;
        let p2d: Point2 = IntPatchHCurve2dTool::value(&self.curve, x[3])?;
        let pcurv: Point3 = HSurfaceTool::value(&self.surface2, p2d.x(), p2d.y())?;
        self.store(psurf, pcurv, f);
        Ok(true)
    }

    /// The Jacobian `(D1u, D1v, -D1w)` and the two points.
    fn jacobian(&self, x: &Vector<'_>, d: &mut Matrix<'_>) -> Result<(Point3, Point3), Failure> {
        let s1 = HSurfaceTool::d1(&self.surface1, x[1], x[2])?;
        let (p2d, d2d): (Point2, Vector2) = IntPatchHCurve2dTool::d1(&self.curve, x[3])?;
        let s2 = HSurfaceTool::d1(&self.surface2, p2d.x(), p2d.y())?;
        let mut d1w = Vector3::default();
        d1w.set_linear_form(d2d.x(), s2.d1u, d2d.y(), s2.d1v);
        let (d1u, d1v) = (s1.d1u, s1.d1v);
        (d[(1, 1)], d[(1, 2)], d[(1, 3)]) = (d1u.x(), d1v.x(), -d1w.x());
        (d[(2, 1)], d[(2, 2)], d[(2, 3)]) = (d1u.y(), d1v.y(), -d1w.y());
        (d[(3, 1)], d[(3, 2)], d[(3, 3)]) = (d1u.z(), d1v.z(), -d1w.z());
        Ok((s1.point, s2.point))
    }

    /// `Derivatives(X, D)`: always true natively. `f.derivatives(&x, &mut d)?`.
    pub fn derivatives(&mut self, x: &Vector<'_>, d: &mut Matrix<'_>) -> Result<bool, Failure> {
        self.jacobian(x, d)?;
        Ok(true)
    }

    /// `Values(X, F, D)`: always true natively. `f.values(&x, &mut fx, &mut d)?`.
    pub fn values(
        &mut self,
        x: &Vector<'_>,
        f: &mut Vector<'_>,
        d: &mut Matrix<'_>,
    ) -> Result<bool, Failure> {
        let (psurf, pcurv) = self.jacobian(x, d)?;
        self.store(psurf, pcurv, f);
        Ok(true)
    }

    /// `Point()`: the middle of the last surface and curve points. `f.point()`.
    pub fn point(&self) -> Point3 {
        self.p
    }

    /// `Root()`: the squared distance of the last points. `f.root()`.
    pub fn root(&self) -> f64 {
        self.f
    }

    /// `AuxillarSurface()`: the surface searched (`S1`). `f.auxillar_surface()`.
    pub fn auxillar_surface(&self) -> &Adaptor3dSurfaceHandle {
        &self.surface1
    }

    /// `AuxillarCurve()`: the curve on `S2`. `f.auxillar_curve()`.
    pub fn auxillar_curve(&self) -> &TheArc {
        &self.curve
    }

    /// The first failure raised by a solver callback since the last call.
    /// `f.take_failure()?`.
    pub fn take_failure(&mut self) -> Result<(), Failure> {
        self.failure.take()
    }
}

captured_function_set!([] IntPatchCSFunction);

/// `TheFunction` of `IntPatch_CurvIntSurf` (docs/geomalgo-design.md section 6).
impl TheFunction for IntPatchCSFunction {
    type SurfaceTool = HSurfaceTool;
    type CurveTool = IntPatchHCurve2dTool;
    fn auxillar_surface(&self) -> &Adaptor3dSurfaceHandle {
        &self.surface1
    }
    fn auxillar_curve(&self) -> &TheArc {
        &self.curve
    }
    fn point(&self) -> Point3 {
        self.p
    }
    fn root(&self) -> f64 {
        self.f
    }
    fn take_failure(&mut self) -> Result<(), Failure> {
        self.failure.take()
    }
}
