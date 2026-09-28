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
// Rust translation of ModelingData/TKGeomBase/GProp/GProp_CelGProps.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GProp_CelGProps`: global properties of arcs of lines and circles.
use super::GProps;
use super::elementary::{assemble_inertia, local_inertia};
use rustycad_tkernel::standard::Failure;
use rustycad_tkmath::elclib;
use rustycad_tkmath::gp::{Axis1, Circle3, Line3, Matrix3, Point3, Xyz};
use std::f64::consts::PI;
use std::ops::Deref;

/// `GProp_CelGProps`: the length, centre and inertia of a bounded line or circle
/// arc; the inherited [`GProps`] members are reached through `Deref`.
///
/// ```
/// use rustycad_tkgeombase::gprop::CelGProps;
/// use rustycad_tkmath::gp::{Line3, Point3, Direction3};
/// let line = Line3::new(Point3::default(), Direction3::default());
/// let props = CelGProps::from_line(line, 0.0, 2.0, Point3::default());
/// assert_eq!(props.mass(), 2.0);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CelGProps {
    base: GProps,
}

impl Deref for CelGProps {
    type Target = GProps;
    fn deref(&self) -> &GProps {
        &self.base
    }
}

impl CelGProps {
    /// `GProp_CelGProps()`: an empty framework at the origin; `CelGProps::new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `GProp_CelGProps(C, CLocation)`: the whole circle `circle`;
    /// `CelGProps::from_circle(circle, location)?`.
    pub fn from_circle(circle: Circle3, location: Point3) -> Result<Self, Failure> {
        Self::from_circle_arc(circle, 0.0, 2.0 * PI, location)
    }

    /// `GProp_CelGProps(C, U1, U2, CLocation)`: the arc `[u1, u2]` of `circle`;
    /// `CelGProps::from_circle_arc(circle, 0.0, 1.0, location)?`.
    pub fn from_circle_arc(
        circle: Circle3,
        u1: f64,
        u2: f64,
        location: Point3,
    ) -> Result<Self, Failure> {
        let mut props: Self = Self::new();
        props.set_location(location);
        props.perform_circle(circle, u1, u2)?;
        Ok(props)
    }

    /// `GProp_CelGProps(C, U1, U2, CLocation)`: the segment `[u1, u2]` of `line`;
    /// `CelGProps::from_line(line, 0.0, 1.0, location)`.
    pub fn from_line(line: Line3, u1: f64, u2: f64, location: Point3) -> Self {
        let mut props: Self = Self::new();
        props.set_location(location);
        props.perform_line(line, u1, u2);
        props
    }

    /// `SetLocation(CLocation)`: the point the inertia is accumulated at;
    /// `props.set_location(p)`.
    pub fn set_location(&mut self, location: Point3) {
        self.base.loc = location;
    }

    /// `Perform(C, U1, U2)` on a circle: arc length, centroid in the circle plane
    /// (the native centre keeps the Z of the circle location) and inertia;
    /// StdFail_NotDone when math_Jacobi fails; `props.perform_circle(c, 0.0, 1.0)?`.
    pub fn perform_circle(&mut self, circle: Circle3, u1: f64, u2: f64) -> Result<(), Failure> {
        let (x0, y0, z0) = coordinates(&circle.location().xyz());
        let xa1: Xyz = circle.x_axis().direction().xyz();
        let xa2: Xyz = circle.y_axis().direction().xyz();
        let ray: f64 = circle.radius();
        let base: &mut GProps = &mut self.base;
        base.dim = ray * (u2 - u1).abs();
        let xloc: f64 = ray * (u2.sin() - u1.sin()) / (u2 - u1);
        let yloc: f64 = ray * (u1.cos() - u2.cos()) / (u2 - u1);
        base.g.set_coordinates(
            xloc * xa1.x() + yloc * xa2.x() + x0,
            xloc * xa1.y() + yloc * xa2.y() + y0,
            z0,
        );
        let ray3: f64 = ray * ray * ray;
        let dm = local_inertia(
            [
                ray3 * (u2 / 2.0 - u1 / 2.0 - (2.0 * u2).sin() / 4.0 + (2.0 * u1).sin() / 4.0),
                ray3 * (u2 / 2.0 - u1 / 2.0 + (2.0 * u2).sin() / 4.0 - (2.0 * u1).sin() / 4.0),
                ray * ray * base.dim,
            ],
            [
                -ray3 * ((2.0 * u1).cos() / 4.0 - (2.0 * u2).cos() / 4.0),
                0.0,
                0.0,
            ],
        );
        let frame = [
            circle.x_axis().direction(),
            circle.y_axis().direction(),
            circle.axis().direction(),
        ];
        assemble_inertia(base, dm, frame, 1.0)
    }

    /// `Perform(C, U1, U2)` on a line: length, midpoint and the inertia integrated
    /// in closed form (at the origin, without the location);
    /// `props.perform_line(line, 0.0, 1.0)`.
    pub fn perform_line(&mut self, line: Line3, u1: f64, u2: f64) {
        let position: Axis1 = line.position();
        let p1: Point3 = elclib::line_value(u1, position);
        self.base.dim = (u2 - u1).abs();
        let p2: Point3 = elclib::line_value(u2, position);
        self.base.g.set_coordinates(
            (p1.x() + p2.x()) / 2.0,
            (p1.y() + p2.y()) / 2.0,
            (p1.z() + p2.z()) / 2.0,
        );
        let (vx, vy, vz) = coordinates(&position.direction().xyz());
        let (x0, y0, z0) = coordinates(&position.location().xyz());
        let integral = |alfa: [f64; 3]| cubic_integral(alfa, u1, u2);
        let ixx = integral([
            (vz * vz + vy * vy) / 3.0,
            vy * y0 + vz * z0,
            y0 * y0 + z0 * z0,
        ]);
        let iyy = integral([
            (vz * vz + vx * vx) / 3.0,
            vx * x0 + vz * z0,
            x0 * x0 + z0 * z0,
        ]);
        // The native Izz repeats the Ixx linear and constant terms (Y and Z).
        let izz = integral([
            (vy * vy + vx * vx) / 3.0,
            vy * y0 + vz * z0,
            y0 * y0 + z0 * z0,
        ]);
        let ixy = integral([(vy * vx) / 3.0, (vy * x0 + vx * y0) / 2.0, y0 * x0]);
        let ixz = integral([(vz * vx) / 3.0, (vz * x0 + vx * z0) / 2.0, z0 * x0]);
        let iyz = integral([(vy * vz) / 3.0, (vy * z0 + vz * y0) / 2.0, y0 * z0]);
        self.base.inertia = Matrix3::from_columns(
            Xyz::new(ixx, -ixy, -ixz),
            Xyz::new(-ixy, iyy, -iyz),
            Xyz::new(-ixz, -iyz, izz),
        );
    }
}

fn coordinates(xyz: &Xyz) -> (f64, f64, f64) {
    (xyz.x(), xyz.y(), xyz.z())
}

// `U2 (U2 (U2 a1 + a2) + a3) - U1 (U1 (U1 a1 + a2) + a3)`, in the native order.
fn cubic_integral([a1, a2, a3]: [f64; 3], u1: f64, u2: f64) -> f64 {
    (u2 * (u2 * (u2 * a1 + a2) + a3)) - (u1 * (u1 * (u1 * a1 + a2) + a3))
}
