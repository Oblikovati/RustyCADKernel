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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_SimpleBound.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_SimpleBound`: a boundary on a 3d curve, without normals.
use super::bound_parameter::BoundParameter;
use super::{GeomFillBoundary, GeomFillBoundaryTolerances};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg2d::geom2d::errors;
use rustycad_tkg3d::adaptor3d::Adaptor3dCurve;
use rustycad_tkmath::gp::{Point3, Vector3};

/// A boundary on an adapted 3d curve (`GeomFill_SimpleBound`), reparametrized by a
/// law once the filling asks for it.
///
/// `let bound = GeomFillSimpleBound::new(&curve, 1e-4, 1e-2); bound.value(0.5)?`
pub struct GeomFillSimpleBound {
    parent: StandardTransient,
    tolerances: GeomFillBoundaryTolerances,
    my_c3d: Handle<dyn Adaptor3dCurve>,
    my_par: BoundParameter,
}
impl_transient!(
    GeomFillSimpleBound,
    "GeomFill_SimpleBound",
    dyn GeomFillBoundary,
    parent,
    interfaces: [dyn GeomFillBoundary]
);

impl GeomFillSimpleBound {
    /// `GeomFill_SimpleBound(Curve, Tol3d, Tolang)`: the curve is shared;
    /// `GeomFillSimpleBound::new(&curve, 1e-4, 1e-2)`.
    pub fn new(curve: &Handle<dyn Adaptor3dCurve>, tol3d: f64, tolang: f64) -> Self {
        Self {
            parent: StandardTransient::default(),
            tolerances: GeomFillBoundaryTolerances::new(tol3d, tolang),
            my_c3d: curve.clone(),
            my_par: BoundParameter::default(),
        }
    }
    /// The curve, or the Standard_NullObject of the native null dereference.
    fn curve(&self) -> Result<&dyn Adaptor3dCurve, Failure> {
        self.my_c3d
            .get()
            .ok_or_else(|| errors::null_object("GeomFill_SimpleBound: null curve"))
    }
}

impl GeomFillBoundary for GeomFillSimpleBound {
    /// The curve at `myPar(U)`.
    fn value(&self, u: f64) -> Result<Point3, Failure> {
        let x: f64 = self.my_par.value(u)?;
        self.curve()?.value(x)
    }
    /// The curve derivative times the law derivative.
    fn d1(&self, u: f64) -> Result<(Point3, Vector3), Failure> {
        let (x, dx) = self.my_par.d1(u)?;
        let (p, v) = self.curve()?.d1(x)?;
        Ok((p, v * dx))
    }
    fn reparametrize(
        &self,
        first: f64,
        last: f64,
        has_df: bool,
        has_dl: bool,
        df: f64,
        dl: f64,
        rev: bool,
    ) -> Result<(), Failure> {
        let curve: &dyn Adaptor3dCurve = self.curve()?;
        self.my_par
            .reparametrize(curve, first, last, (has_df, has_dl), (df, dl), rev)
    }
    /// The law bounds once reparametrized, the curve range before.
    fn bounds(&self) -> Result<(f64, f64), Failure> {
        self.my_par.bounds(|| {
            let curve: &dyn Adaptor3dCurve = self.curve()?;
            Ok((curve.first_parameter()?, curve.last_parameter()?))
        })
    }
    fn is_degenerated(&self) -> bool {
        false
    }
    fn tolerances(&self) -> &GeomFillBoundaryTolerances {
        &self.tolerances
    }
}
