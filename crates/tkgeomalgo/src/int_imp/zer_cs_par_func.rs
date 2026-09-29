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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntImp/IntImp_ZerCSParFunc.gxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntImp_ZerCSParFunc`: the difference between a patch point and a curve point,
//! a function of `(u, v, w)` whose zeros are the curve-surface intersections.
use super::captured::captured_function_set;
use super::{TheCurveTool, TheFunction, ThePSurfaceTool};
use rustycad_tkernel::standard::Failure;
use rustycad_tkgeombase::failure_capture::FailureCapture;
use rustycad_tkmath::gp::{Point3, Vector3};
use rustycad_tkmath::math::{Matrix, Vector};

/// The curve-surface distance function of a patch of tool `P` and a curve of tool
/// `C`.
///
/// `IntCurveSurface_TheCSFunctionOfHInter` is an instance;
/// `IntImpZerCSParFunc::<HSurfaceTool, MyCurveTool>::new(&surface, &curve)`.
pub struct IntImpZerCSParFunc<P: ThePSurfaceTool, C: TheCurveTool> {
    surface: P::Surface,
    curve: C::Curve,
    p: Point3,
    f: f64,
    pub(super) failure: FailureCapture,
}

impl<P: ThePSurfaceTool, C: TheCurveTool> Clone for IntImpZerCSParFunc<P, C> {
    /// The native copy: the surfaces, the last point and root; no pending failure.
    fn clone(&self) -> Self {
        Self {
            surface: self.surface.clone(),
            curve: self.curve.clone(),
            p: self.p,
            f: self.f,
            failure: FailureCapture::default(),
        }
    }
}

impl<P: ThePSurfaceTool, C: TheCurveTool> IntImpZerCSParFunc<P, C> {
    /// The function of `s` and `c` at the origin with a null root;
    /// `IntImpZerCSParFunc::<P, C>::new(&s, &c)`.
    pub fn new(s: &P::Surface, c: &C::Curve) -> Self {
        Self {
            surface: s.clone(),
            curve: c.clone(),
            p: Point3::new(0.0, 0.0, 0.0),
            f: 0.0,
            failure: FailureCapture::default(),
        }
    }
    /// Three variables `(u, v, w)`; `function.nb_variables()`.
    pub fn nb_variables(&self) -> i32 {
        3
    }
    /// Three equations; `function.nb_equations()`.
    pub fn nb_equations(&self) -> i32 {
        3
    }
    /// Writes `Psurf - Pcurv`, keeps its squared norm and the middle point.
    fn store(&mut self, psurf: Point3, pcurv: Point3, f: &mut Vector<'_>) {
        let (f1, f2, f3) = (
            psurf.x() - pcurv.x(),
            psurf.y() - pcurv.y(),
            psurf.z() - pcurv.z(),
        );
        (f[1], f[2], f[3]) = (f1, f2, f3);
        self.f = f1 * f1 + f2 * f2 + f3 * f3;
        self.p = Point3::from((psurf.xyz() + pcurv.xyz()) * 0.5);
    }
    /// `Value(X, F)`; `function.value(&x, &mut f)?`.
    pub fn value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> Result<bool, Failure> {
        let psurf: Point3 = P::value(&self.surface, x[1], x[2])?;
        let pcurv: Point3 = C::value(&self.curve, x[3])?;
        self.store(psurf, pcurv, f);
        Ok(true)
    }
    /// The Jacobian `(D1u, D1v, -D1w)`, and the two points.
    fn jacobian(&self, x: &Vector<'_>, d: &mut Matrix<'_>) -> Result<(Point3, Point3), Failure> {
        let surface = P::d1(&self.surface, x[1], x[2])?;
        let (pcurv, d1w): (Point3, Vector3) = C::d1(&self.curve, x[3])?;
        let (d1u, d1v) = (surface.d1u, surface.d1v);
        (d[(1, 1)], d[(1, 2)], d[(1, 3)]) = (d1u.x(), d1v.x(), -d1w.x());
        (d[(2, 1)], d[(2, 2)], d[(2, 3)]) = (d1u.y(), d1v.y(), -d1w.y());
        (d[(3, 1)], d[(3, 2)], d[(3, 3)]) = (d1u.z(), d1v.z(), -d1w.z());
        Ok((surface.point, pcurv))
    }
    /// `Derivatives(X, D)`; `function.derivatives(&x, &mut d)?`.
    pub fn derivatives(&mut self, x: &Vector<'_>, d: &mut Matrix<'_>) -> Result<bool, Failure> {
        self.jacobian(x, d)?;
        Ok(true)
    }
    /// `Values(X, F, D)`; `function.values(&x, &mut f, &mut d)?`.
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
    /// `Point()`: the middle of the last patch and curve points; `function.point()`.
    pub fn point(&self) -> Point3 {
        self.p
    }
    /// `Root()`: the squared distance of the last points; `function.root()`.
    pub fn root(&self) -> f64 {
        self.f
    }
    /// `AuxillarSurface()`; `function.auxillar_surface()`.
    pub fn auxillar_surface(&self) -> &P::Surface {
        &self.surface
    }
    /// `AuxillarCurve()`; `function.auxillar_curve()`.
    pub fn auxillar_curve(&self) -> &C::Curve {
        &self.curve
    }
    /// The first failure raised by a solver callback since the last call;
    /// `function.take_failure()?`.
    pub fn take_failure(&mut self) -> Result<(), Failure> {
        self.failure.take()
    }
}

captured_function_set!([P: ThePSurfaceTool, C: TheCurveTool] IntImpZerCSParFunc<P, C>);

impl<P: ThePSurfaceTool, C: TheCurveTool> TheFunction for IntImpZerCSParFunc<P, C> {
    type SurfaceTool = P;
    type CurveTool = C;
    fn auxillar_surface(&self) -> &P::Surface {
        IntImpZerCSParFunc::auxillar_surface(self)
    }
    fn auxillar_curve(&self) -> &C::Curve {
        IntImpZerCSParFunc::auxillar_curve(self)
    }
    fn point(&self) -> Point3 {
        IntImpZerCSParFunc::point(self)
    }
    fn root(&self) -> f64 {
        IntImpZerCSParFunc::root(self)
    }
    fn take_failure(&mut self) -> Result<(), Failure> {
        IntImpZerCSParFunc::take_failure(self)
    }
}
