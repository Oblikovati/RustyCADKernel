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
// Rust translation of ModelingData/TKG3d/Geom/Geom_ElementarySurface.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `Geom_ElementarySurface`: planes and quadrics placed by a gp_Ax3 (the native
//! protected `pos`).
use super::protected::{PROTECTED, ProtectedAccess};
use super::surface::{GeomSurface, dump_surface};
use rustycad_tkernel::impl_transient_interface;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkg2d::geom2d::{dump_base, dump_class_name, dump_field};
use rustycad_tkmath::gp::{Axis1, Axis3, Point3};
use std::io;

/// A surface in its local coordinate system; the main axis is the axis of
/// revolution or the normal of a plane; `surface.set_location(Point3::new(1.0, 0.0, 0.0))`.
pub trait GeomElementarySurface: GeomSurface {
    /// The local coordinate system (`Position`); `surface.position()`.
    fn position(&self) -> Axis3;
    /// Change the protected `pos` under the object's lock; only Geom calls it.
    #[doc(hidden)]
    fn change_position(&self, access: ProtectedAccess, change: &mut dyn FnMut(&mut Axis3));
    /// Change the protected `pos` by a step that can raise; only Geom calls it.
    #[doc(hidden)]
    fn try_change_position(
        &self,
        access: ProtectedAccess,
        change: &mut dyn FnMut(&mut Axis3) -> Result<(), Failure>,
    ) -> Result<(), Failure>;
    /// Change the main axis, keeping X and Y as close as possible (`SetAxis`);
    /// Standard_ConstructionError when it is parallel to the old X direction;
    /// `surface.set_axis(axis)?`.
    fn set_axis(&self, axis: Axis1) -> Result<(), Failure> {
        self.try_change_position(PROTECTED, &mut |position| position.set_axis(axis))
    }
    /// Move the origin (`SetLocation`); `surface.set_location(p)`.
    fn set_location(&self, point: Point3) {
        self.change_position(PROTECTED, &mut |position| position.set_location(point));
    }
    /// Replace the local coordinate system (`SetPosition`); `surface.set_position(frame)`.
    fn set_position(&self, frame: Axis3) {
        self.change_position(PROTECTED, &mut |position| *position = frame);
    }
    /// The main axis (`Axis`); `surface.axis()`.
    fn axis(&self) -> Axis1 {
        self.position().axis()
    }
    /// The origin (`Location`); `surface.location()`.
    fn location(&self) -> Point3 {
        self.position().location()
    }
}
impl_transient_interface!(
    dyn GeomElementarySurface,
    "Geom_ElementarySurface",
    dyn GeomSurface
);

/// `Geom_ElementarySurface::UReverse`: reverse the Y direction of the frame;
/// `elementary_u_reverse(&mut pos)`.
pub fn elementary_u_reverse(position: &mut Axis3) {
    position.y_reverse();
}

/// `Geom_ElementarySurface::VReverse`: reverse the main direction of the frame;
/// `elementary_v_reverse(&mut pos)`.
pub fn elementary_v_reverse(position: &mut Axis3) {
    position.z_reverse();
}

/// `Geom_ElementarySurface::DumpJson`; `dump_elementary_surface(&mut stream, -1, pos)?`.
pub fn dump_elementary_surface(
    stream: &mut DumpStream,
    depth: i32,
    position: Axis3,
) -> io::Result<()> {
    dump_class_name(stream, "Geom_ElementarySurface")?;
    dump_base(stream, depth, dump_surface)?;
    dump_field(stream, depth, "pos", |nested, _| position.dump_json(nested))
}

/// Implement the protected position accessors of an elementary surface whose
/// state has a `pos` field.
macro_rules! impl_elementary_position {
    () => {
        fn position(&self) -> rustycad_tkmath::gp::Axis3 {
            self.state.read().pos
        }
        fn change_position(
            &self,
            _access: $crate::geom::protected::ProtectedAccess,
            change: &mut dyn FnMut(&mut rustycad_tkmath::gp::Axis3),
        ) {
            change(&mut self.state.write().pos);
        }
        fn try_change_position(
            &self,
            _access: $crate::geom::protected::ProtectedAccess,
            change: &mut dyn FnMut(
                &mut rustycad_tkmath::gp::Axis3,
            ) -> Result<(), rustycad_tkernel::standard::Failure>,
        ) -> Result<(), rustycad_tkernel::standard::Failure> {
            change(&mut self.state.write().pos)
        }
    };
}
pub(crate) use impl_elementary_position;
