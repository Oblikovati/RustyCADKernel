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
// Rust translation of ModelingData/TKGeomBase/AppDef/AppDef_MyLineTool.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AppDef_MyLineTool`: the static tool through which the AppParCurves and Approx
//! generics read an AppDef_MultiLine, as the [`LineTool`] implementation of
//! [`MultiLine`] (`AppDef_MyLineTool::NbP3d(ML)` is `ml.nb_p3d()`).
use super::multi_line::MultiLine;
use super::multi_point_constraint::MultiPointConstraint;
use crate::app_par_curves::LineTool;
use crate::approx::Status;
use crate::native_failure::array_failure;
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};

// tab(i + low - 1) = read(i) for i in 1..=count (checked writes, as natively).
fn fill<T>(
    tab: &mut Array1<'_, T>,
    count: i32,
    first: i32,
    read: impl Fn(i32) -> Result<T, Failure>,
) -> Result<(), Failure> {
    let low: i32 = tab.lower();
    for i in 1..=count {
        let value: T = read(first + i - 1)?;
        tab.set_value(i + low - 1, value)
            .map_err(|error| array_failure(error, "AppDef_MyLineTool"))?;
    }
    Ok(())
}

// The 3D points, then the 2D ones read at NbPoints() + i.
fn fill_3d_2d<A, B>(
    mpc: &MultiPointConstraint,
    tabs: (&mut Array1<'_, A>, &mut Array1<'_, B>),
    read3d: impl Fn(i32) -> Result<A, Failure>,
    read2d: impl Fn(i32) -> Result<B, Failure>,
) -> Result<(), Failure> {
    let nbp3d: i32 = mpc.nb_points();
    fill(tabs.0, nbp3d, 1, read3d)?;
    fill(tabs.1, mpc.nb_points2d(), nbp3d + 1, read2d)
}

/// `AppDef_MyLineTool`: FirstPoint is 1, LastPoint `NbMultiPoints()`, NbP3d and NbP2d
/// the counts of the first multipoint; the vector reads answer false for a
/// multipoint without tangents (curvatures). The Approx members never cut the line:
/// WhatStatus is NoPointsAdded, MakeMLBetween the same line (a copy sharing its
/// array) and MakeMLOneMorePoint false.
///
/// ```
/// use rustycad_tkgeombase::app_def::MultiLine;
/// use rustycad_tkgeombase::app_par_curves::LineTool;
/// use rustycad_tkernel::ncollection::Array1;
/// use rustycad_tkmath::gp::Point2;
/// let p = [Point2::new(0.0, 0.0), Point2::new(1.0, 1.0)];
/// let line = MultiLine::from_points2d(&Array1::from_shared(&p, 1));
/// assert_eq!((line.nb_p3d()?, line.nb_p2d()?, line.last_point()?), (0, 1, 2));
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
impl LineTool for MultiLine {
    fn first_point(&self) -> i32 {
        1
    }
    fn last_point(&self) -> Result<i32, Failure> {
        self.nb_multi_points()
    }
    fn nb_p2d(&self) -> Result<i32, Failure> {
        Ok(self.value(1)?.nb_points2d())
    }
    fn nb_p3d(&self) -> Result<i32, Failure> {
        Ok(self.value(1)?.nb_points())
    }
    fn value_3d(&self, index: i32, points: &mut Array1<'_, Point3>) -> Result<(), Failure> {
        let mpc: MultiPointConstraint = self.value(index)?;
        fill(points, mpc.nb_points(), 1, |i| mpc.point(i))
    }
    fn value_2d(&self, index: i32, points2d: &mut Array1<'_, Point2>) -> Result<(), Failure> {
        let mpc: MultiPointConstraint = self.value(index)?;
        fill(points2d, mpc.nb_points2d(), mpc.nb_points() + 1, |i| {
            mpc.point2d(i)
        })
    }
    fn value(
        &self,
        index: i32,
        points: &mut Array1<'_, Point3>,
        points2d: &mut Array1<'_, Point2>,
    ) -> Result<(), Failure> {
        let mpc: MultiPointConstraint = self.value(index)?;
        fill_3d_2d(
            &mpc,
            (points, points2d),
            |i| mpc.point(i),
            |i| mpc.point2d(i),
        )
    }
    fn tangency_3d(&self, index: i32, tangents: &mut Array1<'_, Vector3>) -> Result<bool, Failure> {
        let mpc: MultiPointConstraint = self.value(index)?;
        if !mpc.is_tangency_point() {
            return Ok(false);
        }
        fill(tangents, mpc.nb_points(), 1, |i| mpc.tang(i)).map(|()| true)
    }
    fn tangency_2d(
        &self,
        index: i32,
        tangents2d: &mut Array1<'_, Vector2>,
    ) -> Result<bool, Failure> {
        let mpc: MultiPointConstraint = self.value(index)?;
        if !mpc.is_tangency_point() {
            return Ok(false);
        }
        fill(tangents2d, mpc.nb_points2d(), mpc.nb_points() + 1, |i| {
            mpc.tang2d(i)
        })
        .map(|()| true)
    }
    fn tangency(
        &self,
        index: i32,
        tangents: &mut Array1<'_, Vector3>,
        tangents2d: &mut Array1<'_, Vector2>,
    ) -> Result<bool, Failure> {
        let mpc: MultiPointConstraint = self.value(index)?;
        if !mpc.is_tangency_point() {
            return Ok(false);
        }
        fill_3d_2d(
            &mpc,
            (tangents, tangents2d),
            |i| mpc.tang(i),
            |i| mpc.tang2d(i),
        )
        .map(|()| true)
    }
    fn curvature_3d(
        &self,
        index: i32,
        curvatures: &mut Array1<'_, Vector3>,
    ) -> Result<bool, Failure> {
        let mpc: MultiPointConstraint = self.value(index)?;
        if !mpc.is_curvature_point() {
            return Ok(false);
        }
        fill(curvatures, mpc.nb_points(), 1, |i| mpc.curv(i)).map(|()| true)
    }
    fn curvature_2d(
        &self,
        index: i32,
        curvatures2d: &mut Array1<'_, Vector2>,
    ) -> Result<bool, Failure> {
        let mpc: MultiPointConstraint = self.value(index)?;
        if !mpc.is_curvature_point() {
            return Ok(false);
        }
        fill(curvatures2d, mpc.nb_points2d(), mpc.nb_points() + 1, |i| {
            mpc.curv2d(i)
        })
        .map(|()| true)
    }
    fn curvature(
        &self,
        index: i32,
        curvatures: &mut Array1<'_, Vector3>,
        curvatures2d: &mut Array1<'_, Vector2>,
    ) -> Result<bool, Failure> {
        let mpc: MultiPointConstraint = self.value(index)?;
        if !mpc.is_curvature_point() {
            return Ok(false);
        }
        fill_3d_2d(
            &mpc,
            (curvatures, curvatures2d),
            |i| mpc.curv(i),
            |i| mpc.curv2d(i),
        )
        .map(|()| true)
    }
    fn what_status(&self, _i1: i32, _i2: i32) -> Result<Status, Failure> {
        Ok(Status::NoPointsAdded)
    }
    // The native stub returns the line itself.
    fn make_ml_between(&self, _i1: i32, _i2: i32, _nb_p_min: i32) -> Result<Self, Failure> {
        Ok(self.clone())
    }
    fn make_ml_one_more_point(
        &self,
        _i1: i32,
        _i2: i32,
        _index_bad: i32,
    ) -> Result<Option<Self>, Failure> {
        Ok(None)
    }
}
