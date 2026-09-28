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
// Rust translation of ModelingData/TKG3d/Geom/Geom_Axis1Placement.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_Axis1Placement`: a transient gp_Ax1.
use super::axis_placement::GeomAxisPlacement;
use super::geometry::GeomGeometry;
use super::protected::ProtectedAccess;
use rustycad_tkernel::impl_transient;
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkg2d::geom2d::cell::GeometryCell;
use rustycad_tkmath::gp::{Axis1, Direction3, Point3, Transform3};

/// An axis; `Handle::new(GeomAxis1Placement::new(axis))`.
pub struct GeomAxis1Placement {
    parent: StandardTransient,
    state: GeometryCell<Axis1>,
}
impl_transient!(
    GeomAxis1Placement,
    "Geom_Axis1Placement",
    dyn GeomAxisPlacement,
    parent,
    interfaces: [dyn GeomGeometry, dyn GeomAxisPlacement]
);
impl Clone for GeomAxis1Placement {
    fn clone(&self) -> Self {
        Self::new(self.ax1())
    }
}
impl GeomAxis1Placement {
    /// Placement from a gp axis (`Geom_Axis1Placement(gp_Ax1)`); `GeomAxis1Placement::new(axis)`.
    pub fn new(axis: Axis1) -> Self {
        Self {
            parent: StandardTransient::default(),
            state: GeometryCell::new(axis),
        }
    }
    /// Placement from origin and direction (`Geom_Axis1Placement(P, V)`);
    /// `GeomAxis1Placement::from_location(p, d)`.
    pub fn from_location(location: Point3, direction: Direction3) -> Self {
        Self::new(Axis1::new(location, direction))
    }
    /// The gp axis (`Ax1`); `placement.ax1()`.
    pub fn ax1(&self) -> Axis1 {
        *self.state.read()
    }
    /// Reverse the direction in place (`Reverse`); `placement.reverse()`.
    pub fn reverse(&self) {
        self.state.write().reverse();
    }
    /// Reversed copy (`Reversed`); `placement.reversed()`.
    pub fn reversed(&self) -> Handle<GeomAxis1Placement> {
        let mut axis: Axis1 = self.ax1();
        axis.reverse();
        Handle::new(Self::new(axis))
    }
}
impl GeomGeometry for GeomAxis1Placement {
    fn transform(&self, transform: Transform3) -> Result<(), Failure> {
        self.state.write().transform(transform);
        Ok(())
    }
    fn copy(&self) -> Result<Handle<dyn GeomGeometry>, Failure> {
        Ok(Handle::new(self.clone()).cast())
    }
}
impl GeomAxisPlacement for GeomAxis1Placement {
    fn axis(&self) -> Axis1 {
        *self.state.read()
    }
    fn change_axis(&self, _access: ProtectedAccess, change: &mut dyn FnMut(&mut Axis1)) {
        change(&mut self.state.write());
    }
    fn set_direction(&self, direction: Direction3) -> Result<(), Failure> {
        self.state.write().set_direction(direction);
        Ok(())
    }
}
