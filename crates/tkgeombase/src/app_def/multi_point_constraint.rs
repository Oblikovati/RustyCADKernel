// Created on: 1991-03-16
// Created by: LPA
// Copyright (c) 1991-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/AppDef/AppDef_MultiPointConstraint.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AppDef_MultiPointConstraint`: an `AppParCurves_MultiPoint` with optional
//! tangents and curvatures for each of its points.
use crate::app_par_curves::{MultiPoint, copy_array};
use crate::native_failure::{array_failure, failure};
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};
use std::fmt;
use std::ops::Deref;

/// `AppDef_MultiPointConstraint`: the points of an [`MultiPoint`] (reached through
/// `Deref`) and four optional shared arrays: the 3D tangents and curvatures
/// (indexes `1..=NbPoints()`) and the 2D ones (indexes `NbPoints() + 1..`).
///
/// A copy (`clone`) shares the points and every allocated array, as the native copy
/// shares its handles: setting a tangent of a copy changes the original when the
/// array already exists, but not when the copy allocates it.
///
/// ```
/// use rustycad_tkgeombase::app_def::MultiPointConstraint;
/// use rustycad_tkmath::gp::{Point3, Vector3};
/// let mut point = MultiPointConstraint::with_counts(1, 0)?;
/// point.set_point(1, Point3::new(1.0, 2.0, 3.0))?;
/// assert!(!point.is_tangency_point());
/// point.set_tang(1, Vector3::new(0.0, 0.0, 1.0))?;
/// assert!(point.is_tangency_point());
/// assert_eq!(point.tang(1)?, Vector3::new(0.0, 0.0, 1.0));
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Default)]
pub struct MultiPointConstraint {
    base: MultiPoint,
    tang: Handle<HArray1<Vector3>>,
    curv: Handle<HArray1<Vector3>>,
    tang2d: Handle<HArray1<Vector2>>,
    curv2d: Handle<HArray1<Vector2>>,
}

impl Deref for MultiPointConstraint {
    type Target = MultiPoint;
    fn deref(&self) -> &MultiPoint {
        &self.base
    }
}

const CLASS: &str = "AppDef_MultiPointConstraint";

// Standard_ConstructionError when the vector arrays do not match their point arrays.
fn check_lengths(pairs: &[(i32, i32)]) -> Result<(), Failure> {
    if pairs.iter().all(|(points, vectors)| points == vectors) {
        return Ok(());
    }
    Err(failure(
        "Standard_ConstructionError",
        &format!(
            "{CLASS}: point and vector counts {pairs:?}; expected equal (points, vectors) pairs"
        ),
    ))
}

// Standard_OutOfRange outside `low..=high`.
fn check_index(member: &str, index: i32, low: i32, high: i32) -> Result<(), Failure> {
    if index < low || index > high {
        return Err(Failure::out_of_range(&format!(
            "{CLASS}::{member}: index {index}; expected {low}..={high}"
        )));
    }
    Ok(())
}

// The native reads dereference the array handle unchecked: a crash when the tangents
// or curvatures were never set; the port reports Standard_NullObject.
fn read<T: Copy + Send + Sync + 'static>(
    array: &Handle<HArray1<T>>,
    member: &str,
    index: i32,
) -> Result<T, Failure> {
    let Some(values) = array.get() else {
        return Err(failure(
            "Standard_NullObject",
            &format!(
                "{CLASS}::{member}: no vectors set; expected a Set call or a constructor with them"
            ),
        ));
    };
    let values = values.array1();
    values
        .value(index)
        .copied()
        .map_err(|error| array_failure(error, &format!("{CLASS}::{member}")))
}

/// Where a Set member writes: the array, its size, and the index offset (the 2D
/// members count the 3D points first).
struct VectorSlot<'a, T: Send + Sync + 'static> {
    array: &'a mut Handle<HArray1<T>>,
    member: &'static str,
    count: i32,
    offset: i32,
}

// SetValue on the array, allocated `1..=count` first when null (natively before the
// index check, so a failing Set still leaves an allocated array).
fn write<T: Copy + Default + Send + Sync + 'static>(
    slot: VectorSlot<'_, T>,
    index: i32,
    value: T,
) -> Result<(), Failure> {
    if slot.array.is_null() {
        let values: HArray1<T> =
            HArray1::with_bounds(1, slot.count).map_err(|error| array_failure(error, CLASS))?;
        *slot.array = Handle::new(values);
    }
    check_index(
        slot.member,
        index,
        slot.offset + 1,
        slot.offset + slot.count,
    )?;
    slot.array
        .change_array1()
        .set_value(index - slot.offset, value)
        .map_err(|error| array_failure(error, CLASS))
}

impl MultiPointConstraint {
    /// `AppDef_MultiPointConstraint()`: no points; `MultiPointConstraint::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `AppDef_MultiPointConstraint(NbPoints, NbPoints2d)`: origin points, no vectors;
    /// `MultiPointConstraint::with_counts(1, 2)?`.
    pub fn with_counts(nb_points: i32, nb_points2d: i32) -> Result<Self, Failure> {
        Ok(Self::from_base(MultiPoint::with_counts(
            nb_points,
            nb_points2d,
        )?))
    }

    /// `AppDef_MultiPointConstraint(tabP)`: 3D points only;
    /// `MultiPointConstraint::from_points(&points)`.
    pub fn from_points(points: &Array1<'_, Point3>) -> Self {
        Self::from_base(MultiPoint::from_points(points))
    }

    /// `AppDef_MultiPointConstraint(tabP2d)`: 2D points only;
    /// `MultiPointConstraint::from_points2d(&points2d)`.
    pub fn from_points2d(points2d: &Array1<'_, Point2>) -> Self {
        Self::from_base(MultiPoint::from_points2d(points2d))
    }

    /// `AppDef_MultiPointConstraint(tabP, tabP2d)`: 3D then 2D points;
    /// `MultiPointConstraint::from_points_and_2d(&points, &points2d)`.
    pub fn from_points_and_2d(points: &Array1<'_, Point3>, points2d: &Array1<'_, Point2>) -> Self {
        Self::from_base(MultiPoint::from_points_and_2d(points, points2d))
    }

    fn from_base(base: MultiPoint) -> Self {
        Self {
            base,
            ..Self::default()
        }
    }

    /// `AppDef_MultiPointConstraint(tabP, tabP2d, tabVec, tabVec2d)`: Standard_ConstructionError
    /// unless every tangent array matches its point array;
    /// `MultiPointConstraint::with_tangents(&p, &p2d, &v, &v2d)?`.
    pub fn with_tangents(
        points: &Array1<'_, Point3>,
        points2d: &Array1<'_, Point2>,
        tangents: &Array1<'_, Vector3>,
        tangents2d: &Array1<'_, Vector2>,
    ) -> Result<Self, Failure> {
        check_lengths(&[
            (points.length(), tangents.length()),
            (points2d.length(), tangents2d.length()),
        ])?;
        let mut constraint: Self = Self::from_points_and_2d(points, points2d);
        constraint.tang = copy_array(tangents);
        constraint.tang2d = copy_array(tangents2d);
        Ok(constraint)
    }

    /// `AppDef_MultiPointConstraint(tabP, tabP2d, tabVec, tabVec2d, tabCur, tabCur2d)`:
    /// Standard_ConstructionError unless every vector array matches its point array;
    /// `MultiPointConstraint::with_tangents_and_curvatures(&p, &p2d, &v, &v2d, &c, &c2d)?`.
    pub fn with_tangents_and_curvatures(
        points: &Array1<'_, Point3>,
        points2d: &Array1<'_, Point2>,
        tangents: &Array1<'_, Vector3>,
        tangents2d: &Array1<'_, Vector2>,
        curvatures: &Array1<'_, Vector3>,
        curvatures2d: &Array1<'_, Vector2>,
    ) -> Result<Self, Failure> {
        check_lengths(&[
            (points.length(), curvatures.length()),
            (points2d.length(), curvatures2d.length()),
        ])?;
        let mut constraint: Self = Self::with_tangents(points, points2d, tangents, tangents2d)?;
        constraint.curv = copy_array(curvatures);
        constraint.curv2d = copy_array(curvatures2d);
        Ok(constraint)
    }

    /// `AppDef_MultiPointConstraint(tabP, tabVec)`: 3D points and tangents;
    /// `MultiPointConstraint::with_tangents_3d(&p, &v)?`.
    pub fn with_tangents_3d(
        points: &Array1<'_, Point3>,
        tangents: &Array1<'_, Vector3>,
    ) -> Result<Self, Failure> {
        check_lengths(&[(points.length(), tangents.length())])?;
        let mut constraint: Self = Self::from_points(points);
        constraint.tang = copy_array(tangents);
        Ok(constraint)
    }

    /// `AppDef_MultiPointConstraint(tabP, tabVec, tabCur)`: 3D points, tangents and
    /// curvatures; `MultiPointConstraint::with_tangents_and_curvatures_3d(&p, &v, &c)?`.
    pub fn with_tangents_and_curvatures_3d(
        points: &Array1<'_, Point3>,
        tangents: &Array1<'_, Vector3>,
        curvatures: &Array1<'_, Vector3>,
    ) -> Result<Self, Failure> {
        check_lengths(&[(points.length(), curvatures.length())])?;
        let mut constraint: Self = Self::with_tangents_3d(points, tangents)?;
        constraint.curv = copy_array(curvatures);
        Ok(constraint)
    }

    /// `AppDef_MultiPointConstraint(tabP2d, tabVec2d)`: 2D points and tangents;
    /// `MultiPointConstraint::with_tangents_2d(&p2d, &v2d)?`.
    pub fn with_tangents_2d(
        points2d: &Array1<'_, Point2>,
        tangents2d: &Array1<'_, Vector2>,
    ) -> Result<Self, Failure> {
        check_lengths(&[(points2d.length(), tangents2d.length())])?;
        let mut constraint: Self = Self::from_points2d(points2d);
        constraint.tang2d = copy_array(tangents2d);
        Ok(constraint)
    }

    /// `AppDef_MultiPointConstraint(tabP2d, tabVec2d, tabCur2d)`: 2D points, tangents
    /// and curvatures; `MultiPointConstraint::with_tangents_and_curvatures_2d(&p, &v, &c)?`.
    pub fn with_tangents_and_curvatures_2d(
        points2d: &Array1<'_, Point2>,
        tangents2d: &Array1<'_, Vector2>,
        curvatures2d: &Array1<'_, Vector2>,
    ) -> Result<Self, Failure> {
        check_lengths(&[(points2d.length(), curvatures2d.length())])?;
        let mut constraint: Self = Self::with_tangents_2d(points2d, tangents2d)?;
        constraint.curv2d = copy_array(curvatures2d);
        Ok(constraint)
    }

    /// `SetTang(Index, Tang)`: allocates the 3D tangents on first use, then
    /// Standard_OutOfRange outside `1..=NbPoints()`; `point.set_tang(1, v)?`.
    pub fn set_tang(&mut self, index: i32, tang: Vector3) -> Result<(), Failure> {
        let count: i32 = self.nb_points();
        let slot = VectorSlot {
            array: &mut self.tang,
            member: "SetTang",
            count,
            offset: 0,
        };
        write(slot, index, tang)
    }

    /// `Tang(Index)`: Standard_OutOfRange outside `1..=NbPoints()`; Standard_NullObject
    /// without 3D tangents (a crash natively); `point.tang(1)?`.
    pub fn tang(&self, index: i32) -> Result<Vector3, Failure> {
        check_index("Tang", index, 1, self.nb_points())?;
        read(&self.tang, "Tang", index)
    }

    /// `SetTang2d(Index, Tang2d)`: `index` counts the 3D points first (the 2D tangent
    /// `index - NbPoints()`); allocates the 2D tangents on first use;
    /// `point.set_tang2d(2, v)?`.
    pub fn set_tang2d(&mut self, index: i32, tang2d: Vector2) -> Result<(), Failure> {
        let (offset, count) = (self.nb_points(), self.nb_points2d());
        let slot = VectorSlot {
            array: &mut self.tang2d,
            member: "SetTang2d",
            count,
            offset,
        };
        write(slot, index, tang2d)
    }

    /// `Tang2d(Index)`: see [`MultiPointConstraint::set_tang2d`]; `point.tang2d(2)?`.
    pub fn tang2d(&self, index: i32) -> Result<Vector2, Failure> {
        self.check_2d("Tang2d", index)?;
        read(&self.tang2d, "Tang2d", index - self.nb_points())
    }

    /// `SetCurv(Index, Curv)`: as [`MultiPointConstraint::set_tang`] for the 3D
    /// curvatures; `point.set_curv(1, v)?`.
    pub fn set_curv(&mut self, index: i32, curv: Vector3) -> Result<(), Failure> {
        let count: i32 = self.nb_points();
        let slot = VectorSlot {
            array: &mut self.curv,
            member: "SetCurv",
            count,
            offset: 0,
        };
        write(slot, index, curv)
    }

    /// `Curv(Index)`: as [`MultiPointConstraint::tang`]; `point.curv(1)?`.
    pub fn curv(&self, index: i32) -> Result<Vector3, Failure> {
        check_index("Curv", index, 1, self.nb_points())?;
        read(&self.curv, "Curv", index)
    }

    /// `SetCurv2d(Index, Curv2d)`: as [`MultiPointConstraint::set_tang2d`];
    /// `point.set_curv2d(2, v)?`.
    pub fn set_curv2d(&mut self, index: i32, curv2d: Vector2) -> Result<(), Failure> {
        let (offset, count) = (self.nb_points(), self.nb_points2d());
        let slot = VectorSlot {
            array: &mut self.curv2d,
            member: "SetCurv2d",
            count,
            offset,
        };
        write(slot, index, curv2d)
    }

    /// `Curv2d(Index)`: as [`MultiPointConstraint::tang2d`]; `point.curv2d(2)?`.
    pub fn curv2d(&self, index: i32) -> Result<Vector2, Failure> {
        self.check_2d("Curv2d", index)?;
        read(&self.curv2d, "Curv2d", index - self.nb_points())
    }

    /// `IsTangencyPoint()`: a 3D or 2D tangent array exists (an empty one counts);
    /// `point.is_tangency_point()`.
    pub fn is_tangency_point(&self) -> bool {
        !(self.tang.is_null() && self.tang2d.is_null())
    }

    /// `IsCurvaturePoint()`: a 3D or 2D curvature array exists;
    /// `point.is_curvature_point()`.
    pub fn is_curvature_point(&self) -> bool {
        !(self.curv.is_null() && self.curv2d.is_null())
    }

    /// `Dump(o)`: the native one-line header only; `point.dump(&mut text)?`.
    pub fn dump(&self, stream: &mut impl fmt::Write) -> fmt::Result {
        writeln!(stream, "{CLASS} dump:")
    }

    fn check_2d(&self, member: &str, index: i32) -> Result<(), Failure> {
        let nb_p: i32 = self.nb_points();
        check_index(member, index, nb_p + 1, nb_p + self.nb_points2d())
    }
}

/// `operator<<`: the Dump text.
impl fmt::Display for MultiPointConstraint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.dump(formatter)
    }
}
