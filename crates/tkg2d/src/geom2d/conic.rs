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
// Rust translation of ModelingData/TKG2d/Geom2d/Geom2d_Conic.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom2d_Conic`: circles, ellipses, hyperbolas and parabolas placed by a gp_Ax22d.
use super::curve::{Geom2dCurve, dump_curve};
use super::geometry::{dump_base, dump_class_name, dump_field};
use super::protected::{PROTECTED, ProtectedAccess};
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkmath::gp::{Axis2d, Axis22d, Point2};
use std::io;

/// A conic in its local coordinate system (the native protected `pos`).
///
/// The origin is the center (the apex of a parabola) and the X axis is the
/// origin of the parameter; `circle.set_location(Point2::new(1.0, 2.0))`.
pub trait Geom2dConic: Geom2dCurve {
    /// The local coordinate system (`Position`); `conic.position()`.
    fn position(&self) -> Axis22d;
    /// Change the protected `pos` under the object's lock; only Geom2d calls it.
    #[doc(hidden)]
    fn change_position(&self, access: ProtectedAccess, change: &mut dyn FnMut(&mut Axis22d));
    /// Change the protected `pos` by a step that can raise, keeping what it already
    /// changed, as the native member does; only Geom2d calls it.
    #[doc(hidden)]
    fn try_change_position(
        &self,
        access: ProtectedAccess,
        change: &mut dyn FnMut(&mut Axis22d) -> Result<(), Failure>,
    ) -> Result<(), Failure>;
    /// Eccentricity: 0 circle, < 1 ellipse, 1 parabola, > 1 hyperbola (`Eccentricity`);
    /// `conic.eccentricity()?`.
    fn eccentricity(&self) -> Result<f64, Failure>;
    /// Replace the local coordinate system (`SetAxis`); `conic.set_axis(frame)`.
    fn set_axis(&self, axis: Axis22d) {
        self.change_position(PROTECTED, &mut |position| position.set_axis(axis));
    }
    /// Set origin and X direction, keeping the handedness (`SetXAxis`); `conic.set_x_axis(axis)?`.
    fn set_x_axis(&self, axis: Axis2d) -> Result<(), Failure> {
        self.try_change_position(PROTECTED, &mut |position| position.set_x_axis(axis))
    }
    /// Set origin and Y direction, keeping the handedness (`SetYAxis`); `conic.set_y_axis(axis)?`.
    fn set_y_axis(&self, axis: Axis2d) -> Result<(), Failure> {
        self.try_change_position(PROTECTED, &mut |position| position.set_y_axis(axis))
    }
    /// Move the origin (`SetLocation`); `conic.set_location(Point2::new(1.0, 2.0))`.
    fn set_location(&self, point: Point2) {
        self.change_position(PROTECTED, &mut |position| position.set_location(point));
    }
    /// X axis, the origin of the parameterization (`XAxis`); `conic.x_axis()`.
    fn x_axis(&self) -> Axis2d {
        let position: Axis22d = self.position();
        Axis2d::new(position.location(), position.x_direction())
    }
    /// Y axis (`YAxis`); `conic.y_axis()`.
    fn y_axis(&self) -> Axis2d {
        let position: Axis22d = self.position();
        Axis2d::new(position.location(), position.y_direction())
    }
    /// Center, or apex of a parabola (`Location`); `conic.location()`.
    fn location(&self) -> Point2 {
        self.position().location()
    }
}
impl_transient_interface!(dyn Geom2dConic, "Geom2d_Conic", dyn Geom2dCurve);

/// `Geom2d_Conic::Reverse`: reverse the Y direction of the local system;
/// `reverse_conic_position(&mut position)?`.
pub fn reverse_conic_position(position: &mut Axis22d) -> Result<(), Failure> {
    let mut temp = position.y_direction();
    temp.reverse();
    let reversed = Axis22d::with_directions(position.location(), position.x_direction(), temp)?;
    position.set_axis(reversed);
    Ok(())
}

/// `Geom2d_Conic::DumpJson`; `dump_conic(&mut stream, -1, position)?`.
pub fn dump_conic(stream: &mut DumpStream, depth: i32, position: Axis22d) -> io::Result<()> {
    dump_class_name(stream, "Geom2d_Conic")?;
    dump_base(stream, depth, dump_curve)?;
    dump_field(stream, depth, "pos", |nested, _| position.dump_json(nested))
}

/// Implement the protected position accessors of a conic whose state has a `pos` field.
macro_rules! impl_conic_position {
    () => {
        fn position(&self) -> rustycad_tkmath::gp::Axis22d {
            self.state.read().pos
        }
        fn change_position(
            &self,
            _access: $crate::geom2d::protected::ProtectedAccess,
            change: &mut dyn FnMut(&mut rustycad_tkmath::gp::Axis22d),
        ) {
            change(&mut self.state.write().pos);
        }
        fn try_change_position(
            &self,
            _access: $crate::geom2d::protected::ProtectedAccess,
            change: &mut dyn FnMut(
                &mut rustycad_tkmath::gp::Axis22d,
            ) -> Result<(), rustycad_tkernel::standard::Failure>,
        ) -> Result<(), rustycad_tkernel::standard::Failure> {
            change(&mut self.state.write().pos)
        }
    };
}
pub(crate) use impl_conic_position;
