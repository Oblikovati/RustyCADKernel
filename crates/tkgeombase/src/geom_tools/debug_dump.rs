// Created on: 1994-07-25
// Created by: Remi LEQUETTE
// Copyright (c) 1994-1999 Matra Datavision
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
// Rust translation of ModelingData/TKGeomBase/GeomTools/GeomTools_Debug.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `GeomTools_Dump`: the debugger helper that dumps any geometric transient.
use super::console::ConsoleOutput;
use super::{curve_print, curve2d_print, surface_print};
use rustycad_tkernel::standard::{Failure, Handle, StandardTransient};
use rustycad_tkernel::storage::IoStream;
use rustycad_tkg2d::geom2d::Geom2dCurve;
use rustycad_tkg3d::geom::{GeomCurve, GeomSurface};
use std::io::{Read, Seek, SeekFrom};

/// `const char* GeomTools_Dump(const occ::handle<Standard_Transient>&)`: writes
/// `"\n\n"`, the dump of a `Geom_Surface`, `Geom_Curve` or `Geom2d_Curve` (tested in
/// that order) at the default precision, and `"\n"` to the console (the native
/// `std::cout`), and returns what it found. A failure while printing returns its
/// message, as the native `catch (Standard_Failure)` returns `what()`. The native
/// `void*` overload, whose null argument gives "Error: argument is null", exists for
/// debuggers only and has no Rust counterpart.
///
/// ```
/// use rustycad_tkernel::standard::Handle;
/// use rustycad_tkgeombase::geom_tools::{StdoutConsole, geom_tools_dump};
/// assert_eq!(geom_tools_dump(&Handle::default(), &StdoutConsole), "Error: Not a geometric object");
/// ```
pub fn geom_tools_dump(object: &Handle<StandardTransient>, console: &dyn ConsoleOutput) -> String {
    let result = dump_geometry(object, console);
    result.unwrap_or_else(|failure| failure.message().to_string())
}

fn dump_geometry(
    object: &Handle<StandardTransient>,
    console: &dyn ConsoleOutput,
) -> Result<String, Failure> {
    let surface: Handle<dyn GeomSurface> = object.cast();
    if surface.is_some() {
        dump_to(console, |stream| {
            surface_print::print_surface(&surface, stream, false)
        })?;
        return Ok("Found Geom_Surface, see dump in std::cout".into());
    }
    let curve: Handle<dyn GeomCurve> = object.cast();
    if curve.is_some() {
        dump_to(console, |stream| {
            curve_print::print_curve(&curve, stream, false)
        })?;
        return Ok("Found Geom_Curve, see dump in std::cout".into());
    }
    let curve2d: Handle<dyn Geom2dCurve> = object.cast();
    if curve2d.is_some() {
        dump_to(console, |stream| {
            curve2d_print::print_curve2d(&curve2d, stream, false)
        })?;
        return Ok("Found Geom2d_Curve, see dump in std::cout".into());
    }
    Ok("Error: Not a geometric object".into())
}

/// `std::cout << "\n\n"; Print(..., std::cout); std::cout << '\n';` through a fresh
/// stream at the default precision; the console receives what was printed before a
/// failure too, as `std::cout` would have.
fn dump_to(
    console: &dyn ConsoleOutput,
    print: impl FnOnce(&mut IoStream) -> Result<(), Failure>,
) -> Result<(), Failure> {
    let mut stream = IoStream::from_bytes(Vec::new());
    stream.insert_bytes(b"\n\n");
    let printed = print(&mut stream);
    if printed.is_ok() {
        stream.insert_bytes(b"\n");
    }
    console.write(&written_bytes(stream));
    printed
}

/// The bytes written to an in-memory stream.
fn written_bytes(stream: IoStream) -> Vec<u8> {
    let mut bytes = Vec::new();
    if let Some(mut device) = stream.into_device() {
        // An in-memory cursor cannot fail to seek or read.
        let _ = device.seek(SeekFrom::Start(0));
        let _ = device.read_to_end(&mut bytes);
    }
    bytes
}
