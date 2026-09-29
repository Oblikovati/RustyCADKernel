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
// Rust translation of Extrema_ExtElSS.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Extrema_ExtElSS`: the extrema between two elementary surfaces. Only two parallel
//! planes are computed natively; every sphere pair raises Standard_NotImplemented.
use super::failures::{extremum_index, not_done, not_implemented, null_object};
use super::pon::POnSurf;
use rustycad_tkernel::precision;
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::gp::{Cone3, Cylinder3, Plane3, Sphere3, Torus3};

/// `Extrema_ExtElSS`. The native class never assigns its point arrays, so `Points`
/// raises Standard_NullObject where the native member dereferences a null handle;
/// `ExtElSS::planes(plane1, plane2)?.is_parallel()?`.
#[derive(Clone, Debug, Default)]
pub struct ExtElSS {
    done: bool,
    is_par: bool,
    nb_ext: i32,
    sq_dist: Option<Vec<f64>>,
    points1: Option<Vec<POnSurf>>,
    points2: Option<Vec<POnSurf>>,
}

// The body of the sphere overloads: done, not parallel, no extremum, then
// Standard_NotImplemented.
macro_rules! not_implemented_pair {
    ($(#[$doc:meta])* $ctor:ident, $perform:ident, $first:ty, $second:ty, $name:literal) => {
        $(#[$doc])*
        pub fn $ctor(first: $first, second: $second) -> Result<Self, Failure> {
            let mut extrema: Self = Self::new();
            extrema.$perform(&first, &second)?;
            Ok(extrema)
        }
        /// The matching `Perform`: sets done with no extremum, then raises
        /// Standard_NotImplemented as natively.
        pub fn $perform(&mut self, _first: &$first, _second: &$second) -> Result<(), Failure> {
            self.done = true;
            self.is_par = false;
            self.nb_ext = 0;
            Err(not_implemented(concat!("Extrema_ExtElSS::Perform(", $name, ")")))
        }
    };
}

impl ExtElSS {
    /// `Extrema_ExtElSS()`: not done; `ExtElSS::new().is_done() == false`.
    pub fn new() -> Self {
        Self::default()
    }
    /// `Extrema_ExtElSS(gp_Pln, gp_Pln)`; `ExtElSS::planes(plane1, plane2)`.
    pub fn planes(first: Plane3, second: Plane3) -> Self {
        let mut extrema: Self = Self::new();
        extrema.perform_planes(&first, &second);
        extrema
    }
    /// `Perform(gp_Pln, gp_Pln)`: always done; parallel planes (axes within
    /// Precision::Angular()) give their square distance, others no extremum;
    /// `extrema.perform_planes(&plane1, &plane2)`.
    pub fn perform_planes(&mut self, first: &Plane3, second: &Plane3) {
        self.done = true;
        self.is_par = false;
        self.nb_ext = 0;
        let (d1, d2) = (first.axis().direction(), second.axis().direction());
        if d1.is_parallel(d2, precision::angular()) {
            self.is_par = true;
            self.nb_ext = 1;
            self.sq_dist = Some(vec![first.square_distance_plane(*second)]);
        }
    }
    not_implemented_pair!(
        /// `Extrema_ExtElSS(gp_Pln, gp_Sphere)`: Standard_NotImplemented.
        plane_sphere, perform_plane_sphere, Plane3, Sphere3, "gp_Pln, gp_Sphere");
    not_implemented_pair!(
        /// `Extrema_ExtElSS(gp_Sphere, gp_Sphere)`: Standard_NotImplemented.
        spheres, perform_spheres, Sphere3, Sphere3, "gp_Sphere, gp_Sphere");
    not_implemented_pair!(
        /// `Extrema_ExtElSS(gp_Sphere, gp_Cylinder)`: Standard_NotImplemented.
        sphere_cylinder, perform_sphere_cylinder, Sphere3, Cylinder3, "gp_Sphere, gp_Cylinder");
    not_implemented_pair!(
        /// `Extrema_ExtElSS(gp_Sphere, gp_Cone)`: Standard_NotImplemented.
        sphere_cone, perform_sphere_cone, Sphere3, Cone3, "gp_Sphere, gp_Cone");
    not_implemented_pair!(
        /// `Extrema_ExtElSS(gp_Sphere, gp_Torus)`: Standard_NotImplemented.
        sphere_torus, perform_sphere_torus, Sphere3, Torus3, "gp_Sphere, gp_Torus");
}

impl ExtElSS {
    /// `IsDone()`; `extrema.is_done()`.
    pub fn is_done(&self) -> bool {
        self.done
    }
    /// `IsParallel()`: StdFail_NotDone when not done; `extrema.is_parallel()?`.
    pub fn is_parallel(&self) -> Result<bool, Failure> {
        if !self.done {
            return Err(not_done("Extrema_ExtElSS::IsParallel()"));
        }
        Ok(self.is_par)
    }
    /// `NbExt()`: StdFail_NotDone when not done; `extrema.nb_ext()?`.
    pub fn nb_ext(&self) -> Result<i32, Failure> {
        if !self.done {
            return Err(not_done("Extrema_ExtElSS::NbExt()"));
        }
        Ok(self.nb_ext)
    }
    /// `SquareDistance(N)` (the native default is 1); `extrema.square_distance(1)?`.
    pub fn square_distance(&self, n: i32) -> Result<f64, Failure> {
        let member: &str = "Extrema_ExtElSS::SquareDistance()";
        extremum_index(member, n, || self.nb_ext())?;
        handle_value(member, self.sq_dist.as_deref(), n)
    }
    /// `Points(N, P1, P2)`, returned as `(P1, P2)`; `extrema.points(1)?`.
    pub fn points(&self, n: i32) -> Result<(POnSurf, POnSurf), Failure> {
        let member: &str = "Extrema_ExtElSS::Points()";
        extremum_index(member, n, || self.nb_ext())?;
        let first: POnSurf = handle_value(member, self.points1.as_deref(), n)?;
        Ok((first, handle_value(member, self.points2.as_deref(), n)?))
    }
}

/// `theHandle->Value(N)` of an `NCollection_HArray1`: Standard_NullObject for a null
/// handle, Standard_OutOfRange outside the array.
pub(super) fn handle_value<T: Copy>(
    member: &str,
    values: Option<&[T]>,
    n: i32,
) -> Result<T, Failure> {
    let values: &[T] = values.ok_or_else(|| null_object(member))?;
    let index: usize = super::failures::check_index(member, n, values.len() as i32)?;
    Ok(values[index])
}
