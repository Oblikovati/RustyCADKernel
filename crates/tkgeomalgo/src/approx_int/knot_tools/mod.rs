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
// Rust translation of ModelingAlgorithms/TKGeomAlgo/ApproxInt/ApproxInt_KnotTools.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `ApproxInt_KnotTools`: the knot sequence of a discrete point set for its B-spline
//! approximation (A. Razdan, "Knot Placement for B-Spline curve Approximation"): the
//! discrete curvature (`curvature`), the draft knots (`knot_inds`), their filtering
//! (`filter`) and the choice of the parametrization (`par_type`).
mod curvature;
mod filter;
mod knot_inds;
mod par_type;

use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkg2d::geom2d::arrays;
use rustycad_tkmath::gp::{Point2, Point3};
use rustycad_tkmath::math::Vector;

/// `ApproxInt_KnotTools`: static members only (docs/geomalgo-design.md section 20).
///
/// ```
/// use rustycad_tkgeomalgo::approx_int::ApproxIntKnotTools;
/// use rustycad_tkmath::math::Vector;
/// let pars = Vector::filled(1, 3, 0.0)?;
/// let mut curv = rustycad_tkernel::ncollection::Array1::<f64>::with_bounds(1, 2).unwrap();
/// assert_eq!(ApproxIntKnotTools::build_curvature(&[0.0; 6], 3, &pars, &mut curv)?, 0.0);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct ApproxIntKnotTools;

/// The point sets of `BuildKnots` (`thePntsXYZ`, `thePntsU1V1`, `thePntsU2V2`) and their
/// flags (`theApproxXYZ`, `theApproxU1V1`, `theApproxU2V2`).
#[derive(Clone, Copy)]
pub struct ApproxIntKnotPointSets<'a> {
    /// The 3d points.
    pub xyz: &'a Array1<'a, Point3>,
    /// The points on the first surface.
    pub u1v1: &'a Array1<'a, Point2>,
    /// The points on the second surface.
    pub u2v2: &'a Array1<'a, Point2>,
    /// Whether the 3d points are used.
    pub approx_xyz: bool,
    /// Whether the points on the first surface are used.
    pub approx_u1v1: bool,
    /// Whether the points on the second surface are used.
    pub approx_u2v2: bool,
}

impl ApproxIntKnotPointSets<'_> {
    /// The dimension of the used sets (`aDim`).
    fn dim(&self) -> usize {
        let mut dim: usize = 0;
        if self.approx_xyz {
            dim += 3;
        }
        if self.approx_u1v1 {
            dim += 2;
        }
        if self.approx_u2v2 {
            dim += 2;
        }
        dim
    }

    /// The coordinates of the points `first..=last` in the used sets, `dim` values per
    /// point (`aCoords`).
    fn coords(&self, first: i32, last: i32) -> Result<Vec<f64>, Failure> {
        let mut coords: Vec<f64> =
            Vec::with_capacity((last - first + 1).max(0) as usize * self.dim());
        for i in first..=last {
            if self.approx_xyz {
                let p: Point3 = arrays::at(self.xyz, i)?;
                coords.extend_from_slice(&[p.x(), p.y(), p.z()]);
            }
            if self.approx_u1v1 {
                let p: Point2 = arrays::at(self.u1v1, i)?;
                coords.extend_from_slice(&[p.x(), p.y()]);
            }
            if self.approx_u2v2 {
                let p: Point2 = arrays::at(self.u2v2, i)?;
                coords.extend_from_slice(&[p.x(), p.y()]);
            }
        }
        Ok(coords)
    }
}

/// The 1-based `NCollection_Sequence<int>::Value(Index)` (Standard_OutOfRange outside
/// `1..=len`).
pub(super) fn seq_at(inds: &[i32], index: i32) -> Result<i32, Failure> {
    if index < 1 || index as usize > inds.len() {
        return Err(Failure::with_message(
            "Standard_OutOfRange",
            &format!(
                "NCollection_Sequence::Value: index {index} outside 1..={}",
                inds.len()
            ),
        ));
    }
    Ok(inds[(index - 1) as usize])
}

impl ApproxIntKnotTools {
    /// `BuildKnots(thePntsXYZ, thePntsU1V1, thePntsU2V2, thePars, theApproxXYZ,
    /// theApproxU1V1, theApproxU2V2, theMinNbPnts, theKnots)`: appends to `knots` the
    /// indices of the knots of the points (at least one set is used), with at least
    /// `min_nb_pnts` points per knot interval; clears `knots` when no set is used or
    /// there are less than two parameters;
    /// `ApproxIntKnotTools::build_knots(sets, &pars, 30, &mut knots)?`.
    pub fn build_knots(
        sets: ApproxIntKnotPointSets<'_>,
        pars: &Vector<'_>,
        min_nb_pnts: i32,
        knots: &mut Vec<i32>,
    ) -> Result<(), Failure> {
        let dim: usize = sets.dim();
        if dim == 0 || pars.length() < 2 {
            knots.clear();
            return Ok(());
        }
        // I: the coordinates of the points.
        let coords: Vec<f64> = sets.coords(pars.lower(), pars.upper())?;
        // II: the draft knot sequence.
        let mut draft: Vec<i32> = Vec::new();
        Self::compute_knot_inds(&coords, dim as i32, pars, &mut draft)?;
        // III: the output knot sequence.
        Self::filter_knots(&mut draft, min_nb_pnts, knots)
    }
}
