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
// Rust translation of ModelingData/TKGeomBase/AppCont/AppCont_Function.hxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `AppCont_Function`: the continuous function `f(u)` of 3D and 2D points that the
//! user supplies to the continuous least-squares fit.
use rustycad_tkernel::ncollection::Array1;
use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};

/// The protected state of `AppCont_Function`: the number of 3D and 2D points that
/// Value and D1 return. The native constructor sets both to -1 and the derived
/// classes assign them.
///
/// ```
/// use rustycad_tkgeombase::app_cont::FunctionBase;
/// let mut base = FunctionBase::new();
/// assert_eq!(base.get_number_of_points(), (-1, -1));
/// base.set_number_of_points(1, 2);
/// assert_eq!((base.get_nb_of_3d_points(), base.get_nb_of_2d_points()), (1, 2));
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FunctionBase {
    my_nb_pnt: i32,
    my_nb_pnt2d: i32,
}

impl Default for FunctionBase {
    /// `AppCont_Function()`: both counts -1.
    fn default() -> Self {
        Self {
            my_nb_pnt: -1,
            my_nb_pnt2d: -1,
        }
    }
}

impl FunctionBase {
    /// `AppCont_Function()`: both counts -1, until the derived class sets them;
    /// `FunctionBase::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Assigns the protected `myNbPnt` and `myNbPnt2d`, as a derived constructor does;
    /// `base.set_number_of_points(1, 0)`.
    pub fn set_number_of_points(&mut self, nb_pnt: i32, nb_pnt2d: i32) {
        self.my_nb_pnt = nb_pnt;
        self.my_nb_pnt2d = nb_pnt2d;
    }

    /// `GetNumberOfPoints(theNbPnt, theNbPnt2d)`: the 3D and 2D counts;
    /// `base.get_number_of_points()`.
    pub fn get_number_of_points(&self) -> (i32, i32) {
        (self.my_nb_pnt, self.my_nb_pnt2d)
    }

    /// `GetNbOf3dPoints()`; `base.get_nb_of_3d_points()`.
    pub fn get_nb_of_3d_points(&self) -> i32 {
        self.my_nb_pnt
    }

    /// `GetNbOf2dPoints()`; `base.get_nb_of_2d_points()`.
    pub fn get_nb_of_2d_points(&self) -> i32 {
        self.my_nb_pnt2d
    }
}

/// `AppCont_Function`: the abstract class. An implementation owns a [`FunctionBase`]
/// (the protected counts) and gives the parameter range, the points and the
/// derivatives; the non-virtual count getters are provided.
///
/// The points arrive in arrays indexed from 1 with `max(count, 1)` entries, as the
/// fit allocates them. Value and D1 return false where the function cannot be
/// evaluated.
///
/// ```
/// use rustycad_tkernel::ncollection::Array1;
/// use rustycad_tkgeombase::app_cont::{Function, FunctionBase};
/// use rustycad_tkmath::gp::{Point2, Point3, Vector2, Vector3};
/// struct Segment(FunctionBase);
/// impl Function for Segment {
///     fn base(&self) -> &FunctionBase { &self.0 }
///     fn first_parameter(&self) -> f64 { 0.0 }
///     fn last_parameter(&self) -> f64 { 1.0 }
///     fn value(&self, u: f64, _: &mut Array1<'_, Point2>, pnt: &mut Array1<'_, Point3>) -> bool {
///         pnt[1] = Point3::new(u, 0.0, 0.0);
///         true
///     }
///     fn d1(&self, _: f64, _: &mut Array1<'_, Vector2>, vec: &mut Array1<'_, Vector3>) -> bool {
///         vec[1] = Vector3::new(1.0, 0.0, 0.0);
///         true
///     }
/// }
/// let mut base = FunctionBase::new();
/// base.set_number_of_points(1, 0);
/// assert_eq!(Segment(base).get_number_of_points(), (1, 0));
/// ```
pub trait Function {
    /// The protected counts of the base class; `function.base()`.
    fn base(&self) -> &FunctionBase;

    /// `GetNumberOfPoints(theNbPnt, theNbPnt2d)`; `function.get_number_of_points()`.
    fn get_number_of_points(&self) -> (i32, i32) {
        self.base().get_number_of_points()
    }

    /// `GetNbOf3dPoints()`; `function.get_nb_of_3d_points()`.
    fn get_nb_of_3d_points(&self) -> i32 {
        self.base().get_nb_of_3d_points()
    }

    /// `GetNbOf2dPoints()`; `function.get_nb_of_2d_points()`.
    fn get_nb_of_2d_points(&self) -> i32 {
        self.base().get_nb_of_2d_points()
    }

    /// `FirstParameter()`: the first parameter of the function; `function.first_parameter()`.
    fn first_parameter(&self) -> f64;

    /// `LastParameter()`: the last parameter of the function; `function.last_parameter()`.
    fn last_parameter(&self) -> f64;

    /// `Value(theU, thePnt2d, thePnt)`: the points at `u`;
    /// `function.value(0.5, &mut pnt2d, &mut pnt)`.
    fn value(&self, u: f64, pnt2d: &mut Array1<'_, Point2>, pnt: &mut Array1<'_, Point3>) -> bool;

    /// `D1(theU, theVec2d, theVec)`: the derivatives at `u`;
    /// `function.d1(0.5, &mut vec2d, &mut vec)`.
    fn d1(&self, u: f64, vec2d: &mut Array1<'_, Vector2>, vec: &mut Array1<'_, Vector3>) -> bool;

    /// `PeriodInformation(theDimIdx, IsPeriodic, thePeriod)`: whether output
    /// coordinate `dim_idx` (1 to `3 NbPnt + 2 NbPnt2d`) is periodic, and its period;
    /// not periodic (and period 0) by default; `function.period_information(1)`.
    fn period_information(&self, dim_idx: i32) -> (bool, f64) {
        let _ = dim_idx;
        (false, 0.0)
    }
}
