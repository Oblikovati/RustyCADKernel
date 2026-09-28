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
// Rust translation of ModelingData/TKGeomBase/GProp/GProp_PrincipalProps.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GProp_PrincipalProps`: principal moments, axes and radii of gyration.
use rustycad_tkernel::standard::real::epsilon;
use rustycad_tkmath::gp::{Point3, Vector3};

// The relative tolerance of the symmetry tests without argument.
const SYMMETRY_REL_TOL: f64 = 1.0e-10;

/// `GProp_PrincipalProps`: the principal properties of inertia returned by
/// [`GProps::principal_properties`](super::GProps::principal_properties).
///
/// ```
/// use rustycad_tkgeombase::gprop::PrincipalProps;
/// let undefined = PrincipalProps::default();
/// assert_eq!(undefined.moments().0, f64::MAX);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PrincipalProps {
    i1: f64,
    i2: f64,
    i3: f64,
    r1: f64,
    r2: f64,
    r3: f64,
    v1: Vector3,
    v2: Vector3,
    v3: Vector3,
    g: Point3,
}

impl Default for PrincipalProps {
    /// `GProp_PrincipalProps()`: RealLast() moments, radii and centre, the
    /// coordinate axes as axes of inertia.
    fn default() -> Self {
        Self {
            i1: f64::MAX,
            i2: f64::MAX,
            i3: f64::MAX,
            r1: f64::MAX,
            r2: f64::MAX,
            r3: f64::MAX,
            v1: Vector3::new(1.0, 0.0, 0.0),
            v2: Vector3::new(0.0, 1.0, 0.0),
            v3: Vector3::new(0.0, 0.0, 1.0),
            g: Point3::new(f64::MAX, f64::MAX, f64::MAX),
        }
    }
}

impl PrincipalProps {
    /// `GProp_PrincipalProps()`: an undefined presentation; `PrincipalProps::undefined()`.
    pub fn undefined() -> Self {
        Self::default()
    }

    // The private constructor used by GProp_GProps::PrincipalProperties.
    pub(crate) fn new(moments: [f64; 3], radii: [f64; 3], axes: [Vector3; 3], g: Point3) -> Self {
        let [i1, i2, i3] = moments;
        let [r1, r2, r3] = radii;
        let [v1, v2, v3] = axes;
        Self {
            i1,
            i2,
            i3,
            r1,
            r2,
            r3,
            v1,
            v2,
            v3,
            g,
        }
    }

    /// `HasSymmetryAxis()`: two moments equal within 1e-10 relative (of I1, and of
    /// I2 for the pair I2, I3); `props.has_symmetry_axis()`.
    pub fn has_symmetry_axis(&self) -> bool {
        let eps1: f64 = self.i1.abs() * SYMMETRY_REL_TOL;
        let eps2: f64 = self.i2.abs() * SYMMETRY_REL_TOL;
        self.axis_test(eps1, eps2)
    }

    /// `HasSymmetryAxis(aTol)`: as [`Self::has_symmetry_axis`] with the tolerance
    /// `|I * tol| + |Epsilon(I)|`; `props.has_symmetry_axis_with_tolerance(1e-6)`.
    pub fn has_symmetry_axis_with_tolerance(&self, tol: f64) -> bool {
        let eps1: f64 = (self.i1 * tol).abs() + epsilon(self.i1).abs();
        let eps2: f64 = (self.i2 * tol).abs() + epsilon(self.i2).abs();
        self.axis_test(eps1, eps2)
    }

    fn axis_test(&self, eps1: f64, eps2: f64) -> bool {
        (self.i1 - self.i2).abs() <= eps1
            || (self.i1 - self.i3).abs() <= eps1
            || (self.i2 - self.i3).abs() <= eps2
    }

    /// `HasSymmetryPoint()`: the three moments equal within 1e-10 relative to I1;
    /// `props.has_symmetry_point()`.
    pub fn has_symmetry_point(&self) -> bool {
        self.point_test(self.i1.abs() * SYMMETRY_REL_TOL)
    }

    /// `HasSymmetryPoint(aTol)`: as [`Self::has_symmetry_point`] with the tolerance
    /// `|I1 * tol| + |Epsilon(I1)|`; `props.has_symmetry_point_with_tolerance(1e-6)`.
    pub fn has_symmetry_point_with_tolerance(&self, tol: f64) -> bool {
        self.point_test((self.i1 * tol).abs() + epsilon(self.i1).abs())
    }

    fn point_test(&self, eps1: f64) -> bool {
        (self.i1 - self.i2).abs() <= eps1 && (self.i1 - self.i3).abs() <= eps1
    }

    /// `Moments(Ixx, Iyy, Izz)`: the principal moments;
    /// `let (ixx, iyy, izz) = props.moments();`.
    pub fn moments(&self) -> (f64, f64, f64) {
        (self.i1, self.i2, self.i3)
    }

    /// `FirstAxisOfInertia()`: the first eigenvector; `props.first_axis_of_inertia()`.
    pub fn first_axis_of_inertia(&self) -> Vector3 {
        self.v1
    }

    /// `SecondAxisOfInertia()`: the second eigenvector; `props.second_axis_of_inertia()`.
    pub fn second_axis_of_inertia(&self) -> Vector3 {
        self.v2
    }

    /// `ThirdAxisOfInertia()`: the third eigenvector; `props.third_axis_of_inertia()`.
    pub fn third_axis_of_inertia(&self) -> Vector3 {
        self.v3
    }

    /// `RadiusOfGyration(Rxx, Ryy, Rzz)`: the principal radii of gyration;
    /// `let (rxx, ryy, rzz) = props.radius_of_gyration();`.
    pub fn radius_of_gyration(&self) -> (f64, f64, f64) {
        (self.r1, self.r2, self.r3)
    }

    /// The centre of mass the axes pass through (the private native `g`, kept for
    /// the presentation); `props.centre()`.
    pub fn centre(&self) -> Point3 {
        self.g
    }
}
