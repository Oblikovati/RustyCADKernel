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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/GeomFill/GeomFill_Fixed.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomFill_Fixed`: a constant trihedron law.
use super::sweep_failures::construction_error;
use super::trihedron_law::{
    GeomFillTrihedronLaw, GeomFillTrihedronLawBase, TrihedronD0, TrihedronD1, TrihedronD2,
};
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg2d::geom2d::arrays::put;
use rustycad_tkmath::geom_abs::Shape;
use rustycad_tkmath::gp::Vector3;

/// `GeomFill_Fixed`: the same normalized frame at every parameter.
///
/// `let law = GeomFillFixed::new(Vector3::new(1.0, 0.0, 0.0), Vector3::new(0.0, 1.0, 0.0))?;`
pub struct GeomFillFixed {
    parent: StandardTransient,
    base: GeomFillTrihedronLawBase,
    frame: TrihedronD0,
}
impl_transient!(
    GeomFillFixed,
    "GeomFill_Fixed",
    dyn GeomFillTrihedronLaw,
    parent,
    interfaces: [dyn GeomFillTrihedronLaw]
);

impl GeomFillFixed {
    /// `GeomFill_Fixed(Tangent, Normal)`: Standard_ConstructionError when the vectors
    /// are parallel within 0.01 rad; a null vector raises from the gp checks.
    pub fn new(tangent: Vector3, normal: Vector3) -> Result<Self, Failure> {
        if tangent.is_parallel(normal, 0.01)? {
            return Err(construction_error(&format!(
                "GeomFill_Fixed : Two parallel vectors ! ({tangent:?}, {normal:?}; expected non-parallel)"
            )));
        }
        let t = tangent.normalized()?;
        let n = normal.normalized()?;
        let b = t.crossed(n).normalized()?;
        Ok(Self {
            parent: StandardTransient::default(),
            base: GeomFillTrihedronLawBase::default(),
            frame: TrihedronD0 {
                tangent: t,
                normal: n,
                binormal: b,
            },
        })
    }
}

impl GeomFillTrihedronLaw for GeomFillFixed {
    fn trihedron_base(&self) -> &GeomFillTrihedronLawBase {
        &self.base
    }
    fn copy(&self) -> Result<Handle<dyn GeomFillTrihedronLaw>, Failure> {
        let copy = GeomFillFixed::new(self.frame.tangent, self.frame.normal)?;
        copy.set_curve(&self.base.curve())?;
        Ok(Handle::new(copy).cast())
    }
    fn d0(&self, _param: f64, frame: &mut TrihedronD0) -> Result<bool, Failure> {
        *frame = self.frame;
        Ok(true)
    }
    fn d1(&self, _param: f64, frame: &mut TrihedronD1) -> Result<bool, Failure> {
        *frame = TrihedronD1 {
            tangent: self.frame.tangent,
            normal: self.frame.normal,
            binormal: self.frame.binormal,
            ..TrihedronD1::default()
        };
        Ok(true)
    }
    fn d2(&self, _param: f64, frame: &mut TrihedronD2) -> Result<bool, Failure> {
        *frame = TrihedronD2 {
            tangent: self.frame.tangent,
            normal: self.frame.normal,
            binormal: self.frame.binormal,
            ..TrihedronD2::default()
        };
        Ok(true)
    }
    /// Always 1.
    fn nb_intervals(&self, _shape: Shape) -> Result<i32, Failure> {
        Ok(1)
    }
    /// `-Infinite, +Infinite` at the array bounds.
    fn intervals(&self, bounds: &mut Array1<'_, f64>, _shape: Shape) -> Result<(), Failure> {
        let (lower, upper) = (bounds.lower(), bounds.upper());
        put(bounds, lower, -precision::infinite())?;
        put(bounds, upper, precision::infinite())
    }
    fn get_average_law(&self) -> Result<TrihedronD0, Failure> {
        Ok(self.frame)
    }
    /// Always true.
    fn is_constant(&self) -> Result<bool, Failure> {
        Ok(true)
    }
}
