// Copyright (c) 1993-1999 Matra Datavision
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
// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_AxisPlacement.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2d_AxisPlacement`: an oriented axis (origin and unit direction).
use super::geometry::Geom2dGeometry;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, SharedState, StandardTransient};
use rustycad_tkmath::gp::{Axis2d, Direction2, Point2, Transform2};

/// An axis placement; `Handle::new(Geom2dAxisPlacement::new(axis))`.
pub struct Geom2dAxisPlacement {
    parent: StandardTransient,
    state: SharedState<Axis2d>,
}
impl_transient!(
    Geom2dAxisPlacement,
    "Geom2d_AxisPlacement",
    dyn Geom2dGeometry,
    parent,
    interfaces: [dyn Geom2dGeometry]
);
impl Clone for Geom2dAxisPlacement {
    fn clone(&self) -> Self {
        Self::new(self.ax2d())
    }
}
impl Geom2dAxisPlacement {
    /// Placement from a gp axis (`Geom2d_AxisPlacement(gp_Ax2d)`); `Geom2dAxisPlacement::new(axis)`.
    pub fn new(axis: Axis2d) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: SharedState::new(axis),
        }
    }
    /// Placement from origin and direction (`Geom2d_AxisPlacement(P, V)`);
    /// `Geom2dAxisPlacement::from_location(p, d)`.
    pub fn from_location(location: Point2, direction: Direction2) -> Self {
        Self::new(Axis2d::new(location, direction))
    }
    /// Reverse the direction in place (`Reverse`); `axis.reverse()`.
    pub fn reverse(&self) {
        self.state.write().reverse();
    }
    /// Reversed copy (`Reversed`); `axis.reversed()`.
    pub fn reversed(&self) -> Handle<Geom2dAxisPlacement> {
        let mut axis: Axis2d = self.ax2d();
        axis.reverse();
        Handle::new(Self::new(axis))
    }
    /// Replace the axis (`SetAxis`); `axis.set_axis(other)`.
    pub fn set_axis(&self, axis: Axis2d) {
        *self.state.write() = axis;
    }
    /// Replace the direction (`SetDirection`); `axis.set_direction(d)`.
    pub fn set_direction(&self, direction: Direction2) {
        self.state.write().set_direction(direction);
    }
    /// Replace the origin (`SetLocation`); `axis.set_location(p)`.
    pub fn set_location(&self, location: Point2) {
        self.state.write().set_location(location);
    }
    /// Angle in (-Pi, Pi] between the two directions (`Angle`); `axis.angle(&other)`.
    pub fn angle(&self, other: &Geom2dAxisPlacement) -> f64 {
        let own: Axis2d = self.ax2d();
        own.angle(other.ax2d())
    }
    /// The gp axis (`Ax2d`); `axis.ax2d()`.
    pub fn ax2d(&self) -> Axis2d {
        *self.state.read()
    }
    /// The direction (`Direction`); `axis.direction()`.
    pub fn direction(&self) -> Direction2 {
        self.state.read().direction()
    }
    /// The origin (`Location`); `axis.location()`.
    pub fn location(&self) -> Point2 {
        self.state.read().location()
    }
}
impl Geom2dGeometry for Geom2dAxisPlacement {
    fn transform(&self, transform: Transform2) -> Result<(), Failure> {
        self.state.write().transform(transform);
        Ok(())
    }
    fn copy(&self) -> Result<Handle<dyn Geom2dGeometry>, Failure> {
        Ok(Handle::new(self.clone()).cast())
    }
}
