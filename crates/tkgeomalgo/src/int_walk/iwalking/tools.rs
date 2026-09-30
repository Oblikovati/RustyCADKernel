// Copyright (c) 1995-1999 Matra Datavision
// Copyright (c) 1999-2024 OPEN CASCADE SAS
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
// Rust translation of the macro types of ModelingAlgorithms/TKGeomAlgo/IntWalk/IntWalk_IWalking.gxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The type parameters of `IntWalk_IWalking.gxx`: the members its body calls on
//! `TheIWFunction` and on `IntWalk_TheIWLine`, under their native names, and the
//! bundle [`IWalkingTypes`] that an instance names.
use crate::int_imp::{IntImpZerImpFunc, TheISurfaceTool, ThePSurfaceTool};
use crate::int_surf::{IntSurfLineOn2S, IntSurfPathPoint, IntSurfPntOn2S};
use rustycad_tkernel::standard::{Failure, Handle, RuntimeTyped, Transient};
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkmath::gp::{Direction2, Point3, Vector3};
use rustycad_tkmath::math::{FunctionSetWithDerivatives, Matrix, Vector};

/// `TheIWFunction`: the implicit function of two variables `(u, v)` on the walked
/// surface. Its solver side is `math_FunctionSetWithDerivatives`; the members below
/// are the ones the body calls directly, fallible where the native ones can raise.
///
/// `IntPatch_TheSurfFunction` (`IntImpZerImpFunc<HSurfaceTool, IntSurfQuadricTool>`)
/// implements it; `function.checked_value(&x, &mut f)?`.
pub trait TheIWFunction: FunctionSetWithDerivatives + Clone {
    /// `ThePSurface`.
    type Surface;
    /// `Set(Caro)`: the walked surface.
    fn set_surface(&mut self, surface: &Self::Surface);
    /// `Value(X, F)`.
    fn checked_value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> Result<bool, Failure>;
    /// `Values(X, F, D)`.
    fn checked_values(
        &mut self,
        x: &Vector<'_>,
        f: &mut Vector<'_>,
        d: &mut Matrix<'_>,
    ) -> Result<bool, Failure>;
    /// `Derivatives(X, D)`.
    fn checked_derivatives(&mut self, x: &Vector<'_>, d: &mut Matrix<'_>) -> Result<bool, Failure>;
    /// `Root()`: the value of the last evaluation.
    fn root(&self) -> f64;
    /// `Tolerance()`: the bound under which a root is null.
    fn tolerance(&self) -> f64;
    /// `IsTangent()` at the last evaluation.
    fn is_tangent(&mut self) -> Result<bool, Failure>;
    /// `Point()`: the surface point of the last evaluation.
    fn point(&self) -> Point3;
    /// `Direction3d()`: the 3d tangent of the section.
    fn direction3d(&mut self) -> Result<Vector3, Failure>;
    /// `Direction2d()`: the tangent in the surface parameters.
    fn direction2d(&mut self) -> Result<Direction2, Failure>;
    /// `PSurface()->Value(U, V)`: the walked surface evaluated directly.
    fn p_surface_value(&self, u: f64, v: f64) -> Result<Point3, Failure>;
    /// The first failure a solver callback raised since the last call (the native
    /// exception unwinding through `math_FunctionSetRoot`).
    fn take_failure(&mut self) -> Result<(), Failure>;
}

/// `IntWalk_TheIWLine`: a walking line, shared through handles and edited through
/// them, so every member takes `&self` (docs/geom-design.md).
///
/// `IntPatch_TheIWLineOfTheIWalking` implements it; `line.add_point(&point)`.
pub trait TheIWLine: Transient + RuntimeTyped + Sized {
    /// `new IntWalk_TheIWLine(new NCollection_IncAllocator())`: an empty line.
    fn with_inc_allocator() -> Self;
    /// `Reverse()`.
    fn reverse(&self);
    /// `Cut(Index)`: keeps the points before `index`.
    fn cut(&self, index: i32) -> Result<(), Failure>;
    /// `AddPoint(P)`.
    fn add_point(&self, p: &IntSurfPntOn2S);
    /// `AddStatusFirst(Closed, HasFirst)`.
    fn add_status_first(&self, closed: bool, has_first: bool);
    /// `AddStatusFirst(Closed, HasLast, Index, P)`.
    fn add_status_first_point(
        &self,
        closed: bool,
        has_first: bool,
        index: i32,
        p: &IntSurfPathPoint,
    );
    /// `AddStatusFirstLast(Closed, HasFirst, HasLast)`.
    fn add_status_first_last(&self, closed: bool, has_first: bool, has_last: bool);
    /// `AddStatusLast(HasLast)`.
    fn add_status_last(&self, has_last: bool);
    /// `AddStatusLast(HasLast, Index, P)`.
    fn add_status_last_point(&self, has_last: bool, index: i32, p: &IntSurfPathPoint);
    /// `AddIndexPassing(Index)`.
    fn add_index_passing(&self, index: i32);
    /// `SetTangentVector(V, Index)`.
    fn set_tangent_vector(&self, v: Vector3, index: i32);
    /// `SetTangencyAtBegining(IsTangent)`.
    fn set_tangency_at_begining(&self, is_tangent: bool);
    /// `SetTangencyAtEnd(IsTangent)`.
    fn set_tangency_at_end(&self, is_tangent: bool);
    /// `NbPoints()`.
    fn nb_points(&self) -> i32;
    /// `Value(Index)`, a copy; Standard_OutOfRange outside `1..=NbPoints`.
    fn value(&self, index: i32) -> Result<IntSurfPntOn2S, Failure>;
    /// `Line()`: the points as an `IntSurf_LineOn2S`.
    fn line(&self) -> Handle<IntSurfLineOn2S>;
    /// `TangentVector(Index)`: the vector and its index.
    fn tangent_vector(&self) -> (Vector3, i32);
    /// `IsTangentAtBegining()`.
    fn is_tangent_at_begining(&self) -> bool;
    /// `IsTangentAtEnd()`.
    fn is_tangent_at_end(&self) -> bool;
}

/// The macro types of one instance of `IntWalk_IWalking.gxx`: `ThePSurfaceTool`,
/// `TheIWFunction` and `IntWalk_TheIWLine` (the path and loop points are the IntSurf
/// classes in every instance).
///
/// `impl IWalkingTypes for TheIWalkingTypes { type SurfaceTool = HSurfaceTool; ... }`.
pub trait IWalkingTypes {
    /// `ThePSurfaceTool`.
    type SurfaceTool: ThePSurfaceTool;
    /// `TheIWFunction`, over the surfaces of the tool.
    type Function: TheIWFunction<Surface = <Self::SurfaceTool as ThePSurfaceTool>::Surface>;
    /// `IntWalk_TheIWLine`.
    type Line: TheIWLine;
}

impl<P: ThePSurfaceTool, I: TheISurfaceTool> TheIWFunction for IntImpZerImpFunc<P, I> {
    type Surface = P::Surface;
    fn set_surface(&mut self, surface: &P::Surface) {
        IntImpZerImpFunc::set_surface(self, surface);
    }
    fn checked_value(&mut self, x: &Vector<'_>, f: &mut Vector<'_>) -> Result<bool, Failure> {
        IntImpZerImpFunc::value(self, x, f)
    }
    fn checked_values(
        &mut self,
        x: &Vector<'_>,
        f: &mut Vector<'_>,
        d: &mut Matrix<'_>,
    ) -> Result<bool, Failure> {
        IntImpZerImpFunc::values(self, x, f, d)
    }
    fn checked_derivatives(&mut self, x: &Vector<'_>, d: &mut Matrix<'_>) -> Result<bool, Failure> {
        IntImpZerImpFunc::derivatives(self, x, d)
    }
    fn root(&self) -> f64 {
        IntImpZerImpFunc::root(self)
    }
    fn tolerance(&self) -> f64 {
        IntImpZerImpFunc::tolerance(self)
    }
    fn is_tangent(&mut self) -> Result<bool, Failure> {
        IntImpZerImpFunc::is_tangent(self)
    }
    fn point(&self) -> Point3 {
        IntImpZerImpFunc::point(self)
    }
    fn direction3d(&mut self) -> Result<Vector3, Failure> {
        IntImpZerImpFunc::direction3d(self)
    }
    fn direction2d(&mut self) -> Result<Direction2, Failure> {
        IntImpZerImpFunc::direction2d(self)
    }
    // `PSurface()->Value(U, V)` calls the adaptor member, which the tool's Value forwards
    // to; a missing surface is the native null pointer.
    fn p_surface_value(&self, u: f64, v: f64) -> Result<Point3, Failure> {
        let surface = self
            .p_surface()
            .ok_or_else(|| errors::null_object("IntImp_ZerImpFunc::PSurface: no surface"))?;
        P::value(surface, u, v)
    }
    fn take_failure(&mut self) -> Result<(), Failure> {
        IntImpZerImpFunc::take_failure(self)
    }
}
