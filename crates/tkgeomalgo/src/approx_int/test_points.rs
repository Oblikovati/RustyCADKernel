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
// Rust translation of the point loops of ApproxInt_Approx::buildKnots
// (ModelingAlgorithms/TKGeomAlgo/ApproxInt/ApproxInt_Approx.gxx) and
// ApproxInt_KnotTools::DefineParType (ApproxInt_KnotTools.cxx)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The points of a test multi-line split into the three point sets of
//! `ApproxInt_KnotTools` (the same loop in `buildKnots` and `DefineParType`).
use super::line::ApproxIntLine;
use super::multi_line::ApproxIntMultiLine;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::arrays;
use rustycad_tkmath::gp::{Point2, Point3};

/// An array of bounds `first..=last`.
pub(crate) fn bounded<T: Default + Clone>(
    first: i32,
    last: i32,
) -> Result<Array1<'static, T>, Failure> {
    Array1::with_bounds(first, last).map_err(|e| e.into_failure("ApproxInt"))
}

/// The points `first..=last` of a multi-line (`aPntXYZ`, `aPntU1V1`, `aPntU2V2`); an
/// unused set keeps its default points.
pub(crate) struct TestPoints {
    pub(crate) xyz: Array1<'static, Point3>,
    pub(crate) u1v1: Array1<'static, Point2>,
    pub(crate) u2v2: Array1<'static, Point2>,
}

impl TestPoints {
    /// Reads the multipoints of `line`; with one parameter curve, it is on the first
    /// surface when `approx_u1v1`.
    pub(crate) fn read<L: ApproxIntLine>(
        line: &ApproxIntMultiLine<L>,
        (first, last): (i32, i32),
        approx_u1v1: bool,
    ) -> Result<Self, Failure> {
        let mut points = Self {
            xyz: bounded(first, last)?,
            u1v1: bounded(first, last)?,
            u2v2: bounded(first, last)?,
        };
        let (nbp3d, nbp2d) = (line.nb_p3d(), line.nb_p2d());
        let mut tab3d: Array1<'static, Point3> = arrays::one_based(nbp3d.max(1))?;
        let mut tab2d: Array1<'static, Point2> = arrays::one_based(nbp2d.max(1))?;
        for i in first..=last {
            if nbp3d != 0 && nbp2d != 0 {
                line.value(i, &mut tab3d, &mut tab2d)?;
            } else if nbp2d != 0 {
                line.value_2d(i, &mut tab2d)?;
            } else if nbp3d != 0 {
                line.value_3d(i, &mut tab3d)?;
            }
            points.store(i, (nbp3d, nbp2d, approx_u1v1), (&tab3d, &tab2d))?;
        }
        Ok(points)
    }

    fn store(
        &mut self,
        i: i32,
        (nbp3d, nbp2d, approx_u1v1): (i32, i32, bool),
        (tab3d, tab2d): (&Array1<'_, Point3>, &Array1<'_, Point2>),
    ) -> Result<(), Failure> {
        if nbp3d > 0 {
            arrays::put(&mut self.xyz, i, arrays::at(tab3d, 1)?)?;
        }
        if nbp2d > 1 {
            arrays::put(&mut self.u1v1, i, arrays::at(tab2d, 1)?)?;
            arrays::put(&mut self.u2v2, i, arrays::at(tab2d, 2)?)?;
        } else if nbp2d > 0 {
            let target = if approx_u1v1 {
                &mut self.u1v1
            } else {
                &mut self.u2v2
            };
            arrays::put(target, i, arrays::at(tab2d, 1)?)?;
        }
        Ok(())
    }
}
