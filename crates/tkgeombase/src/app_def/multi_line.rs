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
// Rust translation of ModelingData/TKGeomBase/AppDef/AppDef_MultiLine.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AppDef_MultiLine`: the sequence of multipoints the AppDef approximations fit.
use super::multi_point_constraint::MultiPointConstraint;
use crate::native_failure::{array_failure, failure};
use rustycad_tkernel::ncollection::{Array1, HArray1};
use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkmath::gp::{Point2, Point3};
use std::fmt;

/// `AppDef_MultiLine`: the multipoints `1..=NbMultiPoints()` in a shared array.
///
/// A copy (`clone`) shares the array, as the native copy shares its handle:
/// `SetValue` on a copy changes the original. The default line has no array (the
/// native null handle); its members report Standard_NullObject where the native
/// ones dereference it.
///
/// ```
/// use rustycad_tkgeombase::app_def::MultiLine;
/// use rustycad_tkernel::ncollection::Array1;
/// use rustycad_tkmath::gp::Point3;
/// let points = [Point3::new(0.0, 0.0, 0.0), Point3::new(1.0, 2.0, 0.0)];
/// let line = MultiLine::from_points(&Array1::from_shared(&points, 1));
/// assert_eq!((line.nb_multi_points()?, line.nb_points()?), (2, 1));
/// assert_eq!(line.value(2)?.point(1)?, Point3::new(1.0, 2.0, 0.0));
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
#[derive(Clone, Default)]
pub struct MultiLine {
    tab_mult: Handle<HArray1<MultiPointConstraint>>,
}

const CLASS: &str = "AppDef_MultiLine";

// The line of `multipoints`, reindexed from 1.
fn line_of(multipoints: Vec<MultiPointConstraint>) -> MultiLine {
    let mut values: Array1<'static, MultiPointConstraint> = Array1::with_size(multipoints.len());
    values.update_lower_bound(1);
    values
        .iter_mut()
        .zip(multipoints)
        .for_each(|(slot, point)| *slot = point);
    MultiLine {
        tab_mult: Handle::new(HArray1::from_owned(values)),
    }
}

impl MultiLine {
    /// `AppDef_MultiLine()`: no multipoint array (a null handle natively);
    /// `MultiLine::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `AppDef_MultiLine(NbMult)`: `nb_mult` empty multipoints to be set with
    /// SetValue; Standard_ConstructionError when negative;
    /// `MultiLine::with_nb_multi_points(4)?`.
    pub fn with_nb_multi_points(nb_mult: i32) -> Result<Self, Failure> {
        if nb_mult < 0 {
            return Err(failure(
                "Standard_ConstructionError",
                &format!("{CLASS}: NbMult {nb_mult}; expected >= 0"),
            ));
        }
        let values: HArray1<MultiPointConstraint> =
            HArray1::with_bounds(1, nb_mult).map_err(|error| array_failure(error, CLASS))?;
        Ok(Self {
            tab_mult: Handle::new(values),
        })
    }

    /// `AppDef_MultiLine(tabMultiP)`: a copy of the multipoints (each sharing its
    /// arrays with the input); `MultiLine::from_multi_points(&points)`.
    pub fn from_multi_points(tab_multi_p: &Array1<'_, MultiPointConstraint>) -> Self {
        line_of(tab_multi_p.iter().cloned().collect())
    }

    /// `AppDef_MultiLine(tabP3d)`: one 3D point per multipoint;
    /// `MultiLine::from_points(&points)`.
    pub fn from_points(tab_p3d: &Array1<'_, Point3>) -> Self {
        line_of(tab_p3d.iter().map(|&p| one_point_3d(p)).collect())
    }

    /// `AppDef_MultiLine(tabP2d)`: one 2D point per multipoint;
    /// `MultiLine::from_points2d(&points)`.
    pub fn from_points2d(tab_p2d: &Array1<'_, Point2>) -> Self {
        line_of(tab_p2d.iter().map(|&p| one_point_2d(p)).collect())
    }

    /// `NbMultiPoints()`; Standard_NullObject for the default line (a crash
    /// natively); `line.nb_multi_points()?`.
    pub fn nb_multi_points(&self) -> Result<i32, Failure> {
        Ok(self.array("NbMultiPoints")?.array1().length())
    }

    /// `NbPoints()`: the 3D plus 2D points of the first multipoint;
    /// Standard_OutOfRange on an empty line; `line.nb_points()?`.
    pub fn nb_points(&self) -> Result<i32, Failure> {
        let first: MultiPointConstraint = self.value(1)?;
        Ok(first.nb_points() + first.nb_points2d())
    }

    /// `SetValue(Index, MPoint)`: Standard_OutOfRange outside `1..=NbMultiPoints()`;
    /// the line keeps a copy sharing the arrays of `mpoint`; `line.set_value(1, &p)?`.
    pub fn set_value(&self, index: i32, mpoint: &MultiPointConstraint) -> Result<(), Failure> {
        let values = self.array("SetValue")?;
        let mut values = values.change_array1();
        check_index("SetValue", index, values.length())?;
        values
            .set_value(index, mpoint.clone())
            .map_err(|error| array_failure(error, CLASS))
    }

    /// `Value(Index)`: a copy of the multipoint (sharing its arrays);
    /// Standard_OutOfRange outside `1..=NbMultiPoints()`; `line.value(1)?`.
    pub fn value(&self, index: i32) -> Result<MultiPointConstraint, Failure> {
        let values = self.array("Value")?;
        let values = values.array1();
        check_index("Value", index, values.length())?;
        values
            .value(index)
            .cloned()
            .map_err(|error| array_failure(error, CLASS))
    }

    /// `Dump(o)`: the header and the multipoint count; Standard_NullObject for the
    /// default line; `line.dump(&mut text)?`.
    pub fn dump(&self, stream: &mut impl fmt::Write) -> Result<(), Failure> {
        let count: i32 = self.nb_multi_points()?;
        let text: Result<(), fmt::Error> = writeln!(stream, "{CLASS} dump:")
            .and_then(|()| writeln!(stream, "It contains {count} MultiPointConstraint"));
        text.map_err(|error| Failure::new(&format!("{CLASS}::Dump: {error}")))
    }

    fn array(&self, member: &str) -> Result<&HArray1<MultiPointConstraint>, Failure> {
        self.tab_mult.get().ok_or_else(|| {
            failure(
                "Standard_NullObject",
                &format!(
                    "{CLASS}::{member}: the default line has no multipoints; expected a sized line"
                ),
            )
        })
    }
}

// Standard_OutOfRange outside 1..=count.
fn check_index(member: &str, index: i32, count: i32) -> Result<(), Failure> {
    if index <= 0 || index > count {
        return Err(Failure::out_of_range(&format!(
            "{CLASS}::{member}: index {index}; expected 1..={count}"
        )));
    }
    Ok(())
}

// AppDef_MultiPointConstraint MP(1, 0); MP.SetPoint(1, p).
fn one_point_3d(point: Point3) -> MultiPointConstraint {
    MultiPointConstraint::from_points(&Array1::from_shared(&[point], 1))
}

// AppDef_MultiPointConstraint MP(0, 1); MP.SetPoint2d(1, p).
fn one_point_2d(point: Point2) -> MultiPointConstraint {
    MultiPointConstraint::from_points2d(&Array1::from_shared(&[point], 1))
}
