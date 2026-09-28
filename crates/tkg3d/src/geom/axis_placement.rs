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
// Rust translation of ModelingData/TKG3d/Geom/Geom_AxisPlacement.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_AxisPlacement`: an oriented axis (origin and main direction) with the
//! native protected `axis` field.
use super::geometry::GeomGeometry;
use super::protected::{PROTECTED, ProtectedAccess};
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkmath::gp::{Axis1, Direction3, Point3};

/// An axis placement; `placement.set_location(Point3::new(1.0, 2.0, 3.0))`.
pub trait GeomAxisPlacement: GeomGeometry {
    /// The main axis (`Axis`); `placement.axis()`.
    fn axis(&self) -> Axis1;
    /// Change the protected `axis` under the object's lock; only Geom calls it.
    #[doc(hidden)]
    fn change_axis(&self, access: ProtectedAccess, change: &mut dyn FnMut(&mut Axis1));
    /// Replace the main direction (`SetDirection`, pure virtual); `placement.set_direction(d)?`.
    fn set_direction(
        &self,
        direction: Direction3,
    ) -> Result<(), rustycad_tkernel::standard::Failure>;
    /// Replace the main axis only; an Axis2Placement keeps its X and Y directions
    /// (`SetAxis`, non-virtual); `placement.set_axis(axis)`.
    fn set_axis(&self, axis: Axis1) {
        self.change_axis(PROTECTED, &mut |stored| *stored = axis);
    }
    /// Move the origin (`SetLocation`); `placement.set_location(p)`.
    fn set_location(&self, point: Point3) {
        self.change_axis(PROTECTED, &mut |stored| stored.set_location(point));
    }
    /// Angle in [0, Pi] between the main directions (`Angle`); `placement.angle(&*other)`.
    fn angle(&self, other: &dyn GeomAxisPlacement) -> f64 {
        let own: Axis1 = self.axis();
        own.angle(other.axis())
    }
    /// The main direction (`Direction`); `placement.direction()`.
    fn direction(&self) -> Direction3 {
        self.axis().direction()
    }
    /// The origin (`Location`); `placement.location()`.
    fn location(&self) -> Point3 {
        self.axis().location()
    }
}
impl_transient_interface!(
    dyn GeomAxisPlacement,
    "Geom_AxisPlacement",
    dyn GeomGeometry
);
