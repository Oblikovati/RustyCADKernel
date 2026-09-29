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
// Rust translation of ModelingData/TKGeomBase/AppParCurves/AppParCurves_*.gxx (the MultiLine/ToolLine parameters)
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `MultiLine`/`ToolLine` template parameters of the AppParCurves generics.
use crate::approx::Status;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};

/// The static `ToolLine` members the AppParCurves generics call on their
/// `MultiLine`, as methods of the line itself (`ToolLine::NbP3d(SSP)` is
/// `ssp.nb_p3d()`). A MultiLine is a sequence of multipoints `FirstPoint..=LastPoint`,
/// each with `NbP3d` 3D and `NbP2d` 2D points, and optional tangents and curvatures.
///
/// The output arrays are filled from their lower bound. The Approx drivers
/// (`Approx_ComputeLine`) also call `WhatStatus`, `MakeMLBetween` and
/// `MakeMLOneMorePoint`, which a line that is never cut may answer as
/// AppDef_MyLineTool does (no points added, the same line, no extra point).
///
/// `LastPoint`, `NbP2d` and `NbP3d` are fallible: AppDef_MyLineTool reads the line's
/// array (a null one natively crashes) and its first multipoint (Standard_OutOfRange
/// on an empty line), and the generics propagate that exception.
///
/// ```
/// use rustycad_tkgeombase::app_def::MultiLine;
/// use rustycad_tkgeombase::app_par_curves::LineTool;
/// use rustycad_tkernel::ncollection::Array1;
/// use rustycad_tkmath::gp::Point3;
/// fn points(line: &impl LineTool) -> Result<i32, rustycad_tkernel::standard::Failure> {
///     Ok(line.last_point()? - line.first_point() + 1)
/// }
/// let p = [Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 0.0, 0.0)];
/// assert_eq!(points(&MultiLine::from_points(&Array1::from_shared(&p, 1)))?, 2);
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub trait LineTool {
    /// `FirstPoint(ML)`: the index of the first multipoint.
    fn first_point(&self) -> i32;
    /// `LastPoint(ML)`: the index of the last multipoint.
    fn last_point(&self) -> Result<i32, Failure>;
    /// `NbP2d(ML)`: the number of 2D points of a multipoint.
    fn nb_p2d(&self) -> Result<i32, Failure>;
    /// `NbP3d(ML)`: the number of 3D points of a multipoint.
    fn nb_p3d(&self) -> Result<i32, Failure>;
    /// `Value(ML, MPointIndex, tabPt)`: the 3D points of a multipoint.
    fn value_3d(&self, index: i32, points: &mut Array1<'_, Point3>) -> Result<(), Failure>;
    /// `Value(ML, MPointIndex, tabPt2d)`: the 2D points of a multipoint.
    fn value_2d(&self, index: i32, points2d: &mut Array1<'_, Point2>) -> Result<(), Failure>;
    /// `Value(ML, MPointIndex, tabPt, tabPt2d)`: the 3D and 2D points.
    fn value(
        &self,
        index: i32,
        points: &mut Array1<'_, Point3>,
        points2d: &mut Array1<'_, Point2>,
    ) -> Result<(), Failure>;
    /// `Tangency(ML, MPointIndex, tabV)`: the 3D tangents; false without tangents.
    fn tangency_3d(&self, index: i32, tangents: &mut Array1<'_, Vector3>) -> Result<bool, Failure>;
    /// `Tangency(ML, MPointIndex, tabV2d)`: the 2D tangents.
    fn tangency_2d(
        &self,
        index: i32,
        tangents2d: &mut Array1<'_, Vector2>,
    ) -> Result<bool, Failure>;
    /// `Tangency(ML, MPointIndex, tabV, tabV2d)`: the 3D and 2D tangents.
    fn tangency(
        &self,
        index: i32,
        tangents: &mut Array1<'_, Vector3>,
        tangents2d: &mut Array1<'_, Vector2>,
    ) -> Result<bool, Failure>;
    /// `Curvature(ML, MPointIndex, tabV)`: the 3D curvatures; false without them.
    fn curvature_3d(
        &self,
        index: i32,
        curvatures: &mut Array1<'_, Vector3>,
    ) -> Result<bool, Failure>;
    /// `Curvature(ML, MPointIndex, tabV2d)`: the 2D curvatures.
    fn curvature_2d(
        &self,
        index: i32,
        curvatures2d: &mut Array1<'_, Vector2>,
    ) -> Result<bool, Failure>;
    /// `Curvature(ML, MPointIndex, tabV, tabV2d)`: the 3D and 2D curvatures.
    fn curvature(
        &self,
        index: i32,
        curvatures: &mut Array1<'_, Vector3>,
        curvatures2d: &mut Array1<'_, Vector2>,
    ) -> Result<bool, Failure>;
    /// `WhatStatus(ML, I1, I2)`: what the line can do for the points `i1..=i2` that
    /// the approximation cannot fit.
    fn what_status(&self, i1: i32, i2: i32) -> Result<Status, Failure>;
    /// `MakeMLBetween(ML, I1, I2, NbPMin)`: a new line with more points (at least
    /// `nb_p_min`) between the points `i1` and `i2`.
    fn make_ml_between(&self, i1: i32, i2: i32, nb_p_min: i32) -> Result<Self, Failure>
    where
        Self: Sized;
    /// `MakeMLOneMorePoint(ML, I1, I2, indbad, OtherLine)`: the line of the points
    /// `i1..=i2` with one more point before `index_bad`, or None when the tool
    /// cannot add it (the native false result).
    fn make_ml_one_more_point(
        &self,
        i1: i32,
        i2: i32,
        index_bad: i32,
    ) -> Result<Option<Self>, Failure>
    where
        Self: Sized;
}
