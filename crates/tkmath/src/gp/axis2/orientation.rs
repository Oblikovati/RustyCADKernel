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
// Rust translation of FoundationClasses/TKMath/gp/gp_Ax2.hxx inline setters at 3d097a0328e71b826377d4814ab05ec3c3d23871.

use super::super::direction::{CheckedProducts, DirectionProducts, UncheckedProducts};
use super::super::{Axis1, Axis2, Axis3, Direction3, Failure, Point3};
use rustycad_tkernel::precision;
impl Axis2 {
    pub(in crate::gp) const fn from_parts(
        location: Point3,
        direction: Direction3,
        x_direction: Direction3,
        y_direction: Direction3,
    ) -> Self {
        Self {
            frame: Axis3::from_stored(location, direction, x_direction, y_direction),
        }
    }
    /// Returns the main axis (origin and normal); `frame.axis()`.
    pub fn axis(&self) -> Axis1 {
        Axis1::new(self.location(), self.direction())
    }
    /// Moves only the origin; `frame.set_location(point)`.
    pub fn set_location(&mut self, location: Point3) {
        *self = Self::from_parts(
            location,
            self.direction(),
            self.x_direction(),
            self.y_direction(),
        );
    }
    /// Replaces the main axis, swapping stored directions when the new normal is the X
    /// direction within `Precision::Angular()`; `frame.set_axis(axis)?`.
    pub fn set_axis(&mut self, axis: Axis1) -> Result<(), Failure> {
        self.set_normal::<CheckedProducts>(axis.location(), axis.direction())
    }
    /// Replaces the normal with gp_Ax2's collinear-X swap; `frame.set_direction(normal)?`.
    pub fn set_direction(&mut self, normal: Direction3) -> Result<(), Failure> {
        self.set_normal::<CheckedProducts>(self.location(), normal)
    }
    /// gp_Ax2::SetDirection as compiled with `No_Exception` (ElSLib); `frame.set_direction_unchecked(normal)`.
    pub(crate) fn set_direction_unchecked(&mut self, normal: Direction3) {
        let Ok(()) = self.set_normal::<UncheckedProducts>(self.location(), normal);
    }
    fn set_normal<P: DirectionProducts>(
        &mut self,
        location: Point3,
        normal: Direction3,
    ) -> Result<(), P::Error> {
        let cosine = normal.dot(self.x_direction());
        let (x, y, old) = (self.x_direction(), self.y_direction(), self.direction());
        if (cosine.abs() - 1.0).abs() <= precision::angular() {
            *self = if cosine > 0.0 {
                Self::from_parts(location, normal, y, old)
            } else {
                Self::from_parts(location, normal, old, y)
            };
            return Ok(());
        }
        // The axis is committed before the cross products, matching the native failure state.
        *self = Self::from_parts(location, normal, x, y);
        let x = P::cross_crossed(normal, x, normal)?;
        *self = Self::from_parts(location, normal, x, y);
        let y = P::crossed(normal, x)?;
        *self = Self::from_parts(location, normal, x, y);
        Ok(())
    }
    /// gp_Ax2(P, N, Vx) with a build-specific zero-norm policy; `Axis2::frame_using::<P>(p,n,x)`.
    pub(crate) fn frame_using<P: DirectionProducts>(
        location: Point3,
        normal: Direction3,
        x: Direction3,
    ) -> Result<Self, P::Error> {
        let x = P::cross_crossed(normal, x, normal)?;
        let y = P::crossed(normal, x)?;
        Ok(Self::from_parts(location, normal, x, y))
    }
    /// Projects X onto the plane and rebuilds Y; `frame.set_x_direction(direction)?`.
    pub fn set_x_direction(&mut self, direction: Direction3) -> Result<(), Failure> {
        let normal = self.direction();
        let x = normal.cross_crossed(direction, normal)?;
        *self = Self::from_parts(self.location(), normal, x, self.y_direction());
        let y = normal.crossed(x)?;
        *self = Self::from_parts(self.location(), normal, x, y);
        Ok(())
    }
    /// Derives X from Y and the normal, then rebuilds Y; `frame.set_y_direction(direction)?`.
    pub fn set_y_direction(&mut self, direction: Direction3) -> Result<(), Failure> {
        let normal = self.direction();
        let x = direction.crossed(normal)?;
        *self = Self::from_parts(self.location(), normal, x, self.y_direction());
        let y = normal.crossed(x)?;
        *self = Self::from_parts(self.location(), normal, x, y);
        Ok(())
    }
    /// Returns the angle between main directions; `frame.angle(other)`.
    pub fn angle(&self, other: Self) -> f64 {
        self.axis().angle(other.axis())
    }
    /// Tests the other origin against this plane and parallel normals; `frame.is_coplanar(other,1e-7,1e-12)`.
    pub fn is_coplanar(&self, other: Self, linear: f64, angular: f64) -> bool {
        plane_offset(self, other.location()) <= linear
            && self.axis().is_parallel(other.axis(), angular)
    }
    /// Tests an axis lying in this plane via a normal direction; `frame.is_coplanar_axis(axis,1e-7,1e-12)`.
    pub fn is_coplanar_axis(&self, axis: Axis1, linear: f64, angular: f64) -> bool {
        plane_offset(self, axis.location()) <= linear && self.axis().is_normal(axis, angular)
    }
}
fn plane_offset(frame: &Axis2, point: Point3) -> f64 {
    let normal = frame.direction();
    let origin = frame.location();
    let offset = normal.x() * (point.x() - origin.x())
        + normal.y() * (point.y() - origin.y())
        + normal.z() * (point.z() - origin.z());
    if offset < 0.0 { -offset } else { offset }
}
