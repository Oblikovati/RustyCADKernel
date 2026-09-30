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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_Boundary.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_Boundary`: the abstract boundary of a constrained filling.
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::standard::{Failure, SharedState, StandardTransient, Transient};
use rustycad_tkmath::gp::{Point3, Vector3};

/// The private tolerances of `GeomFill_Boundary` (`myT3d`, `myTang`), held by each
/// boundary behind a lock because the setters edit a shared boundary;
/// `GeomFillBoundaryTolerances::new(1e-4, 1e-2)`.
#[derive(Default)]
pub struct GeomFillBoundaryTolerances {
    values: SharedState<(f64, f64)>,
}

impl GeomFillBoundaryTolerances {
    /// The protected constructor `GeomFill_Boundary(Tol3d, Tolang)`;
    /// `GeomFillBoundaryTolerances::new(1e-4, 1e-2)`.
    pub fn new(tol3d: f64, tolang: f64) -> Self {
        Self {
            values: SharedState::new((tol3d, tolang)),
        }
    }
}

/// The Standard_Failure of the boundaries without normals.
fn undefined_normals() -> Failure {
    Failure::with_message(
        "Standard_Failure",
        "GeomFill_Boundary::Norm : Undefined normals; expected a boundary with normals",
    )
}

/// A boundary of a constrained filling (`GeomFill_Boundary`), reached through
/// `Handle<dyn GeomFillBoundary>`: a curve with a parametrization that the filling
/// can change, optionally with the normals of a constraint surface. Every evaluator
/// returns `Result`, because the curves and laws behind it raise natively.
/// `let (first, last) = boundary.points()?;`
pub trait GeomFillBoundary: Transient {
    /// `Value(U)`.
    fn value(&self, u: f64) -> Result<Point3, Failure>;
    /// `D1(U, P, V)`: the point and first derivative.
    fn d1(&self, u: f64) -> Result<(Point3, Vector3), Failure>;
    /// `HasNormals()`: false unless overridden.
    fn has_normals(&self) -> bool {
        false
    }
    /// `Norm(U)`: Standard_Failure unless overridden.
    fn norm(&self, _u: f64) -> Result<Vector3, Failure> {
        Err(undefined_normals())
    }
    /// `D1Norm(U, N, DN)`: the normal and its derivative; Standard_Failure unless
    /// overridden.
    fn d1_norm(&self, _u: f64) -> Result<(Vector3, Vector3), Failure> {
        Err(undefined_normals())
    }
    /// `Reparametrize(First, Last, HasDF, HasDL, DF, DL, Rev)`: a new parametrization
    /// on `[first, last]`, reversed when `rev`, with the end derivatives `df` and
    /// `dl` imposed when asked.
    #[allow(clippy::too_many_arguments)] // The native member signature.
    fn reparametrize(
        &self,
        first: f64,
        last: f64,
        has_df: bool,
        has_dl: bool,
        df: f64,
        dl: f64,
        rev: bool,
    ) -> Result<(), Failure>;
    /// `Points(PFirst, PLast)`: the points at the two bounds.
    fn points(&self) -> Result<(Point3, Point3), Failure> {
        let (f, l) = self.bounds()?;
        Ok((self.value(f)?, self.value(l)?))
    }
    /// `Bounds(First, Last)`.
    fn bounds(&self) -> Result<(f64, f64), Failure>;
    /// `IsDegenerated()`.
    fn is_degenerated(&self) -> bool;
    /// The stored tolerances of the boundary.
    fn tolerances(&self) -> &GeomFillBoundaryTolerances;
    /// `Tol3d()`.
    fn tol3d(&self) -> f64 {
        self.tolerances().values.read().0
    }
    /// `Tol3d(Tol)`.
    fn set_tol3d(&self, tol: f64) {
        self.tolerances().values.write().0 = tol;
    }
    /// `Tolang()`.
    fn tolang(&self) -> f64 {
        self.tolerances().values.read().1
    }
    /// `Tolang(Tol)`.
    fn set_tolang(&self, tol: f64) {
        self.tolerances().values.write().1 = tol;
    }
}
impl_transient_interface!(dyn GeomFillBoundary, "GeomFill_Boundary", StandardTransient);
