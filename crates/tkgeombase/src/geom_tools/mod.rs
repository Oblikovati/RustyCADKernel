// Created on: 1993-01-21
// Created by: Remi LEQUETTE
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
// Rust translation of ModelingData/TKGeomBase/GeomTools/GeomTools.hxx/.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! GeomTools: text dump, write and read of Geom curves and surfaces and of Geom2d
//! curves, the geometry sections of the BRep ASCII format.
//!
//! The static members of the `GeomTools` class are the free functions of this module
//! (`GeomTools::Dump(C, OS)` is [`dump_curve`]); the three set classes are
//! [`CurveSet`], [`Curve2dSet`] and [`SurfaceSet`]. The native `Standard_OStream` and
//! `Standard_IStream` are the TKernel model of a libstdc++ iostream, [`IoStream`],
//! so numbers are formatted and parsed byte for byte as natively. The text native
//! code writes to `std::cout` goes to an injected [`ConsoleOutput`].
//!
//! docs/tkgeombase-progress.md lists the native quirks the port keeps.
mod console;
mod curve2d_print;
mod curve2d_read;
mod curve2d_set;
mod curve_print;
mod curve_read;
mod curve_set;
mod curve_text;
mod debug_dump;
mod geometry_table;
mod surface_print;
mod surface_print_spline;
mod surface_read;
mod surface_set;
mod text_in;
mod text_out;
mod type_codes;
mod undefined_type_handler;

pub use console::{ConsoleOutput, StdoutConsole};
pub use curve_set::CurveSet;
pub use curve2d_set::Curve2dSet;
pub use debug_dump::geom_tools_dump;
pub use surface_set::SurfaceSet;
pub use text_in::get_real;
pub use undefined_type_handler::{UndefinedTypeHandler, UndefinedTypeHandlerBase};

use rustycad_tkernel::standard::{Failure, Handle};
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use std::sync::{OnceLock, PoisonError, RwLock};

/// `static theActiveHandler = new GeomTools_UndefinedTypeHandler`: the process-wide
/// handler every printer and reader consults, as natively.
fn active_handler() -> &'static RwLock<Handle<dyn UndefinedTypeHandler>> {
    static ACTIVE: OnceLock<RwLock<Handle<dyn UndefinedTypeHandler>>> = OnceLock::new();
    ACTIVE.get_or_init(|| RwLock::new(Handle::new(UndefinedTypeHandlerBase::default()).into_cast()))
}

/// The active handler, read once per dispatch as `GetUndefinedTypeHandler()` is.
pub(crate) fn undefined_type_handler() -> Handle<dyn UndefinedTypeHandler> {
    get_undefined_type_handler()
}

/// The Standard_NullObject a printer raises where the native code dereferences a
/// null handle.
pub(crate) fn null_geometry(member: &str, class: &str) -> Failure {
    Failure::with_type(
        "Standard_NullObject",
        &format!("{member}: null handle; expected a {class}"),
        "",
    )
}

/// `GeomTools::Dump(S, OS)`: the dump layout at the stream precision;
/// `geom_tools::dump_surface(&surface, &mut stream)?`.
pub fn dump_surface(
    surface: &Handle<dyn GeomSurface>,
    stream: &mut IoStream,
) -> Result<(), Failure> {
    SurfaceSet::print_surface(surface, stream, false)
}

/// `GeomTools::Write(S, OS)`: the compact layout at the stream precision (unlike the
/// set `Write`, precision 17 is not set); `geom_tools::write_surface(&surface, &mut stream)?`.
pub fn write_surface(
    surface: &Handle<dyn GeomSurface>,
    stream: &mut IoStream,
) -> Result<(), Failure> {
    SurfaceSet::print_surface(surface, stream, true)
}

/// `GeomTools::Read(S, IS)`: one compact surface record, null when it cannot be
/// read; `let surface = geom_tools::read_surface(&mut stream);`.
pub fn read_surface(stream: &mut IoStream) -> Handle<dyn GeomSurface> {
    SurfaceSet::read_surface(stream)
}

/// `GeomTools::Dump(C, OS)` for a `Geom_Curve`; `geom_tools::dump_curve(&curve, &mut stream)?`.
pub fn dump_curve(curve: &Handle<dyn GeomCurve>, stream: &mut IoStream) -> Result<(), Failure> {
    CurveSet::print_curve(curve, stream, false)
}

/// `GeomTools::Write(C, OS)` for a `Geom_Curve`, at the stream precision;
/// `geom_tools::write_curve(&curve, &mut stream)?`.
pub fn write_curve(curve: &Handle<dyn GeomCurve>, stream: &mut IoStream) -> Result<(), Failure> {
    CurveSet::print_curve(curve, stream, true)
}

/// `GeomTools::Read(C, IS)` for a `Geom_Curve`; `let curve = geom_tools::read_curve(&mut stream);`.
pub fn read_curve(stream: &mut IoStream) -> Handle<dyn GeomCurve> {
    CurveSet::read_curve(stream)
}

/// `GeomTools::Dump(C, OS)` for a `Geom2d_Curve`; `geom_tools::dump_curve2d(&curve, &mut stream)?`.
pub fn dump_curve2d(curve: &Handle<dyn Geom2dCurve>, stream: &mut IoStream) -> Result<(), Failure> {
    Curve2dSet::print_curve2d(curve, stream, false)
}

/// `GeomTools::Write(C, OS)` for a `Geom2d_Curve`, at the stream precision;
/// `geom_tools::write_curve2d(&curve, &mut stream)?`.
pub fn write_curve2d(
    curve: &Handle<dyn Geom2dCurve>,
    stream: &mut IoStream,
) -> Result<(), Failure> {
    Curve2dSet::print_curve2d(curve, stream, true)
}

/// `GeomTools::Read(C, IS)` for a `Geom2d_Curve`;
/// `let curve = geom_tools::read_curve2d(&mut stream);`.
pub fn read_curve2d(stream: &mut IoStream) -> Handle<dyn Geom2dCurve> {
    Curve2dSet::read_curve2d(stream)
}

/// `GeomTools::SetUndefinedTypeHandler`: installs `handler` for every later print and
/// read of an unknown class or type code; a null handle is ignored.
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkgeombase::geom_tools::{self, UndefinedTypeHandler};
/// geom_tools::set_undefined_type_handler(&Handle::default());
/// assert!(!geom_tools::get_undefined_type_handler().is_null());
/// ```
pub fn set_undefined_type_handler(handler: &Handle<dyn UndefinedTypeHandler>) {
    if handler.is_null() {
        return;
    }
    // A poisoned lock still holds a valid handle: handles are replaced atomically.
    let mut active = active_handler()
        .write()
        .unwrap_or_else(PoisonError::into_inner);
    *active = handler.clone();
}

/// `GeomTools::GetUndefinedTypeHandler`: the active handler, never null;
/// `geom_tools::get_undefined_type_handler()`.
pub fn get_undefined_type_handler() -> Handle<dyn UndefinedTypeHandler> {
    active_handler()
        .read()
        .unwrap_or_else(PoisonError::into_inner)
        .clone()
}
