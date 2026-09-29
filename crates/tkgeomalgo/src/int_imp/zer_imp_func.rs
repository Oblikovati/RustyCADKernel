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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/IntImp/IntImp_ZerImpFunc.gxx/.lxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `IntImp_ZerImpFunc`: the implicit function of one surface evaluated on a
//! parametric patch, whose zeros are the intersection of the two.
use super::captured::captured_function_set;
use super::{TheISurfaceTool, ThePSurfaceTool};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkgeombase::failure_capture::FailureCapture;
use rustycad_tkmath::gp::{Direction2, Point3, Vector3};
use rustycad_tkmath::math::{Matrix, Vector};

/// `EpsAng * EpsAng`: the squared sine under which the tangent is undefined.
const EPS_ANG2: f64 = 1.0e-16;
/// `Tolpetit`: the smallest 3d tangent.
const TOLPETIT: f64 = 1.0e-16;

/// The implicit surface of tool `I` on the patch of tool `P`, a function of the
/// patch parameters `(u, v)`.
///
/// `IntPatch_TheSurfFunction` is `IntImpZerImpFunc<HSurfaceTool, IntSurfQuadricTool>`;
/// the surfaces are held by value (the native class keeps pointers to them).
/// `IntImpZerImpFunc::<HSurfaceTool, IntSurfQuadricTool>::with_surfaces(&patch, &quadric)`.
pub struct IntImpZerImpFunc<P: ThePSurfaceTool, I: TheISurfaceTool> {
    surf: Option<P::Surface>,
    func: Option<I::Surface>,
    u: f64,
    v: f64,
    tol: f64,
    pntsol: Point3,
    valf: f64,
    computed: bool,
    tangent: bool,
    tgdu: f64,
    tgdv: f64,
    gradient: Vector3,
    derived: bool,
    d1u: Vector3,
    d1v: Vector3,
    d3d: Vector3,
    d2d: Direction2,
    pub(super) failure: FailureCapture,
}

impl<P: ThePSurfaceTool, I: TheISurfaceTool> Default for IntImpZerImpFunc<P, I> {
    fn default() -> Self {
        Self {
            surf: None,
            func: None,
            u: 0.0,
            v: 0.0,
            tol: 0.0,
            pntsol: Point3::default(),
            valf: 0.0,
            computed: false,
            tangent: false,
            tgdu: 0.0,
            tgdv: 0.0,
            gradient: Vector3::default(),
            derived: false,
            d1u: Vector3::default(),
            d1v: Vector3::default(),
            d3d: Vector3::default(),
            d2d: Direction2::default(),
            failure: FailureCapture::default(),
        }
    }
}

impl<P: ThePSurfaceTool, I: TheISurfaceTool> IntImpZerImpFunc<P, I> {
    /// The function without surfaces (the default constructor); set them before
    /// evaluating. `IntImpZerImpFunc::<P, I>::new()`.
    pub fn new() -> Self {
        Self::default()
    }
    /// The function of `is` on `ps`; `IntImpZerImpFunc::<P, I>::with_surfaces(&ps, &is)`.
    pub fn with_surfaces(ps: &P::Surface, is: &I::Surface) -> Self {
        Self {
            surf: Some(ps.clone()),
            func: Some(is.clone()),
            ..Self::default()
        }
    }
    /// The function of `is` without patch; `IntImpZerImpFunc::<P, I>::with_implicit(&is)`.
    pub fn with_implicit(is: &I::Surface) -> Self {
        Self {
            func: Some(is.clone()),
            ..Self::default()
        }
    }
    /// `Set(PS)`: the patch; `function.set_surface(&ps)`.
    pub fn set_surface(&mut self, ps: &P::Surface) {
        self.surf = Some(ps.clone());
    }
    /// `SetImplicitSurface(IS)`; `function.set_implicit_surface(&is)`.
    pub fn set_implicit_surface(&mut self, is: &I::Surface) {
        self.func = Some(is.clone());
    }
    /// `Set(Tol)`: the tolerance under which a root is null; `function.set_tolerance(1e-7)`.
    pub fn set_tolerance(&mut self, tol: f64) {
        self.tol = tol;
    }
    /// Two variables; `function.nb_variables()`.
    pub fn nb_variables(&self) -> i32 {
        2
    }
    /// One equation; `function.nb_equations()`.
    pub fn nb_equations(&self) -> i32 {
        1
    }
    /// The patch and the implicit surface; a missing one (a null pointer natively)
    /// raises `Standard_NullObject`.
    fn surfaces(&self) -> Result<(&P::Surface, &I::Surface), Failure> {
        let missing = || errors::null_object("IntImp_ZerImpFunc: the surfaces are not set");
        Ok((
            self.surf.as_ref().ok_or_else(missing)?,
            self.func.as_ref().ok_or_else(missing)?,
        ))
    }
    /// `Value(X, F)`: the implicit function at the patch point; `function.value(&x, &mut f)?`.
    pub fn value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> Result<bool, Failure> {
        (self.u, self.v) = (x[1], x[2]);
        let (surf, func) = self.surfaces()?;
        let pntsol: Point3 = P::value(surf, self.u, self.v)?;
        self.valf = I::value(func, pntsol.x(), pntsol.y(), pntsol.z());
        self.pntsol = pntsol;
        f[1] = self.valf;
        (self.computed, self.derived) = (false, false);
        Ok(true)
    }
    /// The patch derivatives at `(u, v)` into the state.
    fn patch_d1(&mut self) -> Result<(), Failure> {
        let (surf, _) = self.surfaces()?;
        let d1 = P::d1(surf, self.u, self.v)?;
        (self.pntsol, self.d1u, self.d1v) = (d1.point, d1.d1u, d1.d1v);
        Ok(())
    }
    fn store_jacobian(&mut self, d: &mut Matrix<'_>) {
        d[(1, 1)] = self.d1u.dot(self.gradient);
        d[(1, 2)] = self.d1v.dot(self.gradient);
        (self.computed, self.derived) = (false, true);
    }
    /// `Derivatives(X, D)`; `function.derivatives(&x, &mut d)?`.
    pub fn derivatives(&mut self, x: &Vector<'_>, d: &mut Matrix<'_>) -> Result<bool, Failure> {
        (self.u, self.v) = (x[1], x[2]);
        self.patch_d1()?;
        let (_, func) = self.surfaces()?;
        let p: Point3 = self.pntsol;
        self.gradient = I::gradient(func, p.x(), p.y(), p.z());
        self.store_jacobian(d);
        Ok(true)
    }
    /// `Values(X, F, D)`; `function.values(&x, &mut f, &mut d)?`.
    pub fn values(
        &mut self,
        x: &Vector<'_>,
        f: &mut Vector<'_>,
        d: &mut Matrix<'_>,
    ) -> Result<bool, Failure> {
        (self.u, self.v) = (x[1], x[2]);
        self.patch_d1()?;
        let (_, func) = self.surfaces()?;
        let p: Point3 = self.pntsol;
        let (mut valf, mut gradient) = (self.valf, self.gradient);
        I::value_and_gradient(func, [p.x(), p.y(), p.z()], &mut valf, &mut gradient);
        (self.valf, self.gradient) = (valf, gradient);
        f[1] = self.valf;
        self.store_jacobian(d);
        Ok(true)
    }
    /// `IsTangent()`: whether the 3d tangent of the intersection is undefined at the
    /// last point (computed once per evaluation; the gradient is the one of the last
    /// `Derivatives`/`Values`, as natively). `Direction2d` of a vanishing 2d tangent
    /// raises `Standard_ConstructionError`; `function.is_tangent()?`.
    pub fn is_tangent(&mut self) -> Result<bool, Failure> {
        if self.computed {
            return Ok(self.tangent);
        }
        self.computed = true;
        if !self.derived {
            self.patch_d1()?;
            self.derived = true;
        }
        self.tgdu = self.gradient.dot(self.d1v);
        self.tgdv = -self.gradient.dot(self.d1u);
        let n2grad_eps_ang2: f64 = self.gradient.square_magnitude() * EPS_ANG2;
        let (n2d1u, n2d1v) = (self.d1u.square_magnitude(), self.d1v.square_magnitude());
        self.tangent = (self.tgdu * self.tgdu <= n2grad_eps_ang2 * n2d1v)
            && (self.tgdv * self.tgdv <= n2grad_eps_ang2 * n2d1u);
        if !self.tangent {
            self.d3d
                .set_linear_form(self.tgdu, self.d1u, self.tgdv, self.d1v);
            self.d2d = Direction2::new(self.tgdu, self.tgdv)?;
            if self.d3d.magnitude() <= TOLPETIT {
                self.tangent = true;
            }
        }
        Ok(self.tangent)
    }
    /// `Root()`: the last value; `function.root()`.
    pub fn root(&self) -> f64 {
        self.valf
    }
    /// `Tolerance()`; `function.tolerance()`.
    pub fn tolerance(&self) -> f64 {
        self.tol
    }
    /// `Point()`: the last patch point; `function.point()`.
    pub fn point(&self) -> Point3 {
        self.pntsol
    }
    fn defined(&mut self, member: &str) -> Result<(), Failure> {
        if self.is_tangent()? {
            return Err(Failure::with_message(
                "StdFail_UndefinedDerivative",
                &format!("IntImp_ZerImpFunc::{member}: the tangent is undefined"),
            ));
        }
        Ok(())
    }
    /// `Direction3d()`: the 3d tangent of the intersection; `StdFail_UndefinedDerivative`
    /// when `IsTangent`; `function.direction3d()?`.
    pub fn direction3d(&mut self) -> Result<Vector3, Failure> {
        self.defined("Direction3d")?;
        Ok(self.d3d)
    }
    /// `Direction2d()`: the tangent in the patch parameters; `function.direction2d()?`.
    pub fn direction2d(&mut self) -> Result<Direction2, Failure> {
        self.defined("Direction2d")?;
        Ok(self.d2d)
    }
    /// `PSurface()`: the patch, None before it is set; `function.p_surface()`.
    pub fn p_surface(&self) -> Option<&P::Surface> {
        self.surf.as_ref()
    }
    /// `ISurface()`: the implicit surface, None before it is set; `function.i_surface()`.
    pub fn i_surface(&self) -> Option<&I::Surface> {
        self.func.as_ref()
    }
    /// The first failure raised by a solver callback since the last call;
    /// `function.take_failure()?`.
    pub fn take_failure(&mut self) -> Result<(), Failure> {
        self.failure.take()
    }
}

captured_function_set!([P: ThePSurfaceTool, I: TheISurfaceTool] IntImpZerImpFunc<P, I>);
