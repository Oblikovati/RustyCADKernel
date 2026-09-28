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
// Rust translation of ModelingData/TKG3d/Geom/Geom_Conic.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_Conic`: circles, ellipses, hyperbolas and parabolas placed by a gp_Ax2.
use super::curve::{GeomCurve, dump_curve};
use super::protected::{PROTECTED, ProtectedAccess};
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkg2d::geom2d::{dump_base, dump_class_name, dump_field};
use rustycad_tkmath::gp::{Axis1, Axis2, Direction3, Point3};
use std::io;

/// A conic in its local coordinate system (the native protected `pos`).
///
/// The origin is the center (the apex of a parabola), the main direction is the
/// normal of the plane of the conic and the X axis is the origin of the
/// parameter; `circle.set_location(Point3::new(1.0, 2.0, 3.0))`.
pub trait GeomConic: GeomCurve {
    /// The local coordinate system (`Position`); `conic.position()`.
    fn position(&self) -> Axis2;
    /// Change the protected `pos` under the object's lock; only Geom calls it.
    #[doc(hidden)]
    fn change_position(&self, access: ProtectedAccess, change: &mut dyn FnMut(&mut Axis2));
    /// Change the protected `pos` by a step that can raise, keeping what it already
    /// changed, as the native member does; only Geom calls it.
    #[doc(hidden)]
    fn try_change_position(
        &self,
        access: ProtectedAccess,
        change: &mut dyn FnMut(&mut Axis2) -> Result<(), Failure>,
    ) -> Result<(), Failure>;
    /// Eccentricity: 0 circle, < 1 ellipse, 1 parabola, > 1 hyperbola (`Eccentricity`);
    /// `conic.eccentricity()?`.
    fn eccentricity(&self) -> Result<f64, Failure>;
    /// Change the main axis, keeping the X direction as close as possible
    /// (`SetAxis`); Standard_ConstructionError when it is parallel to the old X direction;
    /// `conic.set_axis(axis)?`.
    fn set_axis(&self, axis: Axis1) -> Result<(), Failure> {
        self.try_change_position(PROTECTED, &mut |position| position.set_axis(axis))
    }
    /// Move the origin (`SetLocation`); `conic.set_location(Point3::new(1.0, 2.0, 3.0))`.
    fn set_location(&self, point: Point3) {
        self.change_position(PROTECTED, &mut |position| position.set_location(point));
    }
    /// Replace the local coordinate system (`SetPosition`); `conic.set_position(frame)`.
    fn set_position(&self, frame: Axis2) {
        self.change_position(PROTECTED, &mut |position| *position = frame);
    }
    /// The main axis, normal to the plane of the conic (`Axis`); `conic.axis()`.
    fn axis(&self) -> Axis1 {
        self.position().axis()
    }
    /// Center, or apex of a parabola (`Location`); `conic.location()`.
    fn location(&self) -> Point3 {
        self.position().location()
    }
    /// X axis, the origin of the parameterization (`XAxis`); `conic.x_axis()`.
    fn x_axis(&self) -> Axis1 {
        let position: Axis2 = self.position();
        Axis1::new(position.location(), position.x_direction())
    }
    /// Y axis (`YAxis`); `conic.y_axis()`.
    fn y_axis(&self) -> Axis1 {
        let position: Axis2 = self.position();
        Axis1::new(position.location(), position.y_direction())
    }
}
impl_transient_interface!(dyn GeomConic, "Geom_Conic", dyn GeomCurve);

/// `Geom_Conic::Reverse`: reverse the main direction of the local system, which
/// reverses its Y direction too; `reverse_conic_position(&mut position)?`.
pub fn reverse_conic_position(position: &mut Axis2) -> Result<(), Failure> {
    let mut vz: Direction3 = position.direction();
    vz.reverse();
    position.set_direction(vz)
}

/// `Geom_Conic::DumpJson`; `dump_conic(&mut stream, -1, position)?`.
pub fn dump_conic(stream: &mut DumpStream, depth: i32, position: Axis2) -> io::Result<()> {
    dump_class_name(stream, "Geom_Conic")?;
    dump_base(stream, depth, dump_curve)?;
    dump_field(stream, depth, "pos", |nested, _| position.dump_json(nested))
}

/// Implement the protected position accessors of a conic whose state has a `pos` field.
macro_rules! impl_conic_position {
    () => {
        fn position(&self) -> rustycad_tkmath::gp::Axis2 {
            self.state.read().pos
        }
        fn change_position(
            &self,
            _access: $crate::geom::protected::ProtectedAccess,
            change: &mut dyn FnMut(&mut rustycad_tkmath::gp::Axis2),
        ) {
            change(&mut self.state.write().pos);
        }
        fn try_change_position(
            &self,
            _access: $crate::geom::protected::ProtectedAccess,
            change: &mut dyn FnMut(
                &mut rustycad_tkmath::gp::Axis2,
            ) -> Result<(), rustycad_tkernel::standard::Failure>,
        ) -> Result<(), rustycad_tkernel::standard::Failure> {
            change(&mut self.state.write().pos)
        }
    };
}
pub(crate) use impl_conic_position;
