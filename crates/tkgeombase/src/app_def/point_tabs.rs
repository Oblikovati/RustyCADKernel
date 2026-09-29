// Copyright (c) 1996-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/AppDef/AppDef_LinearCriteria.cxx and
// AppDef_Variational.cxx (the TabP3d/TabP2d point reads)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `TabP3d(1, max(1, NbP3d))` and `TabP2d` arrays through which
//! AppDef_LinearCriteria and AppDef_Variational read the points of a MultiLine.
use super::multi_line::MultiLine;
use crate::app_par_curves::LineTool;
use crate::native_failure::array_failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Point3};

/// The point arrays of one multipoint, reused from point to point as natively
/// (entries a read does not write keep their previous values).
pub(super) struct PointTabs {
    /// `TabP3d`.
    pub(super) p3d: Array1<'static, Point3>,
    /// `TabP2d`.
    pub(super) p2d: Array1<'static, Point2>,
    nb_p3d: i32,
}

impl PointTabs {
    /// The arrays for `[NbP3d, NbP2d]` points; `PointTabs::new([1, 0])?`.
    pub(super) fn new([nb_p3d, nb_p2d]: [i32; 2]) -> Result<Self, Failure> {
        let context: &str = "AppDef point arrays";
        Ok(Self {
            p3d: Array1::with_bounds(1, nb_p3d.max(1))
                .map_err(|error| array_failure(error, context))?,
            p2d: Array1::with_bounds(1, nb_p2d.max(1))
                .map_err(|error| array_failure(error, context))?,
            nb_p3d,
        })
    }

    /// `Value(SSP, i, TabP3d)`; `tabs.read_3d(&line, 1)?`.
    pub(super) fn read_3d(&mut self, ssp: &MultiLine, i: i32) -> Result<(), Failure> {
        ssp.value_3d(i, &mut self.p3d)
    }

    /// The 2D points of multipoint `i` through the overload the native code picks:
    /// `Value(SSP, i, TabP2d)` without 3D points, else `Value(SSP, i, TabP3d, TabP2d)`;
    /// `tabs.read_2d(&line, 1)?`.
    pub(super) fn read_2d(&mut self, ssp: &MultiLine, i: i32) -> Result<(), Failure> {
        if self.nb_p3d == 0 {
            return ssp.value_2d(i, &mut self.p2d);
        }
        LineTool::value(ssp, i, &mut self.p3d, &mut self.p2d)
    }
}
