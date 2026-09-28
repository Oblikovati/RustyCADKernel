// Created on: 1999-10-29
// Created by: Pavel DURANDIN
// Copyright (c) 1999-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/GeomTools/GeomTools_UndefinedTypeHandler.hxx/.cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomTools_UndefinedTypeHandler`: the extension point the GeomTools printers and
//! readers call for a geometry class, or a type code, they do not know.
use super::console::{ConsoleOutput, stdout_console};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient, Transient};
use rustycad_tkernel::storage::IoStream;
use rustycad_tkernel::{impl_transient, impl_transient_interface};
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use std::sync::Arc;

const UNKNOWN_CURVE: &[u8] = b"****** UNKNOWN CURVE TYPE ******";
const UNKNOWN_CURVE2D: &[u8] = b"****** UNKNOWN CURVE2d TYPE ******";
const UNKNOWN_SURFACE: &[u8] = b"****** UNKNOWN SURFACE TYPE ******";

/// The virtual members of `GeomTools_UndefinedTypeHandler`. The provided methods are
/// the native base behaviour: a print writes an "UNKNOWN ... TYPE" line (to the
/// stream for a dump, to the console for the compact form, which therefore stays
/// empty), and a read consumes nothing and returns a null handle. Override them to
/// store and restore other classes; `GeomTools::SetUndefinedTypeHandler` installs
/// the handler ([`super::set_undefined_type_handler`]).
///
/// Every member returns `Result` because an override may raise, and the readers
/// catch a failure as they catch a native Standard_Failure.
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkernel::storage::IoStream;
/// use rustycad_tkgeombase::geom_tools::{UndefinedTypeHandler, UndefinedTypeHandlerBase};
/// let handler = UndefinedTypeHandlerBase::default();
/// let mut stream = IoStream::from_bytes(Vec::new());
/// assert!(handler.read_curve(42, &mut stream).unwrap().is_null());
/// ```
pub trait UndefinedTypeHandler: Transient {
    /// Port-only seam: where the native `std::cout` writes of the provided members
    /// go; the process standard output unless overridden; `handler.console()`.
    fn console(&self) -> Arc<dyn ConsoleOutput> {
        stdout_console()
    }

    /// `PrintCurve(C, OS, compact)`; `handler.print_curve(&curve, &mut stream, false)?`.
    fn print_curve(
        &self,
        _curve: &Handle<dyn GeomCurve>,
        stream: &mut IoStream,
        compact: bool,
    ) -> Result<(), Failure> {
        report_unknown(&*self.console(), stream, compact, UNKNOWN_CURVE);
        Ok(())
    }

    /// `ReadCurve(ctype, IS, C)`: the curve read for type code `ctype` (the native
    /// output handle, null when nothing is read); `handler.read_curve(12, &mut stream)?`.
    fn read_curve(
        &self,
        _ctype: i32,
        _stream: &mut IoStream,
    ) -> Result<Handle<dyn GeomCurve>, Failure> {
        Ok(Handle::default())
    }

    /// `PrintCurve2d(C, OS, compact)`; `handler.print_curve2d(&curve, &mut stream, false)?`.
    fn print_curve2d(
        &self,
        _curve: &Handle<dyn Geom2dCurve>,
        stream: &mut IoStream,
        compact: bool,
    ) -> Result<(), Failure> {
        report_unknown(&*self.console(), stream, compact, UNKNOWN_CURVE2D);
        Ok(())
    }

    /// `ReadCurve2d(ctype, IS, C)`; `handler.read_curve2d(12, &mut stream)?`.
    fn read_curve2d(
        &self,
        _ctype: i32,
        _stream: &mut IoStream,
    ) -> Result<Handle<dyn Geom2dCurve>, Failure> {
        Ok(Handle::default())
    }

    /// `PrintSurface(S, OS, compact)`; `handler.print_surface(&surface, &mut stream, false)?`.
    fn print_surface(
        &self,
        _surface: &Handle<dyn GeomSurface>,
        stream: &mut IoStream,
        compact: bool,
    ) -> Result<(), Failure> {
        report_unknown(&*self.console(), stream, compact, UNKNOWN_SURFACE);
        Ok(())
    }

    /// `ReadSurface(ctype, IS, S)`; `handler.read_surface(12, &mut stream)?`.
    fn read_surface(
        &self,
        _ctype: i32,
        _stream: &mut IoStream,
    ) -> Result<Handle<dyn GeomSurface>, Failure> {
        Ok(Handle::default())
    }
}
impl_transient_interface!(
    dyn UndefinedTypeHandler,
    "GeomTools_UndefinedTypeHandler",
    StandardTransient
);

/// The native base behaviour of the print members: the dump gets the line, while the
/// compact form writes it to `std::cout` and leaves the stream untouched.
fn report_unknown(console: &dyn ConsoleOutput, stream: &mut IoStream, compact: bool, line: &[u8]) {
    if compact {
        console.write(&[line, b"\n"].concat());
    } else {
        stream.insert_bytes(line).insert_bytes(b"\n");
    }
}

/// An instance of the native class itself (`new GeomTools_UndefinedTypeHandler`), the
/// handler GeomTools starts with; its dynamic type is `GeomTools_UndefinedTypeHandler`.
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkgeombase::geom_tools::UndefinedTypeHandlerBase;
/// let handler = Handle::new(UndefinedTypeHandlerBase::default());
/// assert!(handler.is_instance_name("GeomTools_UndefinedTypeHandler"));
/// ```
pub struct UndefinedTypeHandlerBase {
    parent: StandardTransient,
    console: Arc<dyn ConsoleOutput>,
}
impl_transient!(
    UndefinedTypeHandlerBase,
    "GeomTools_UndefinedTypeHandler",
    StandardTransient,
    parent,
    interfaces: [dyn UndefinedTypeHandler]
);

impl Default for UndefinedTypeHandlerBase {
    /// `GeomTools_UndefinedTypeHandler()`, writing its console lines to standard output.
    fn default() -> Self {
        Self::with_console(stdout_console())
    }
}

impl UndefinedTypeHandlerBase {
    /// The native handler with its `std::cout` writes sent to `console`;
    /// `UndefinedTypeHandlerBase::with_console(Arc::new(StdoutConsole))`.
    pub fn with_console(console: Arc<dyn ConsoleOutput>) -> Self {
        Self {
            parent: StandardTransient::default(),
            console,
        }
    }
}

impl UndefinedTypeHandler for UndefinedTypeHandlerBase {
    fn console(&self) -> Arc<dyn ConsoleOutput> {
        Arc::clone(&self.console)
    }
}
