// Created on: 2016-10-25
// Created by: Andrey BETENEV
// Copyright (c) 2016 OPEN CASCADE SAS
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
// Rust translation of ModelingData/TKBRep/BRepTools/BRepTools_Debug.cxx at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The debugger helpers of BRepTools_Debug.cxx: global functions that no header
//! declares, meant to be called from a debugger prompt. Their text goes to the
//! injected console that stands for `std::cout`, and a failure is returned as its
//! message, as the native `catch` returns `what()`.
use super::io::{dump, write_file};
use crate::topods::Shape;
use crate::toptools::LocationSet;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::storage::IoStream;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkgeombase::geom_tools::ConsoleOutput;
use rustycad_tkmath::toploc::Location;
use std::io::{Read, Seek, SeekFrom};

/// `BRepTools_Write(theFileStr, theShape)`: the file name on success, "Error: write
/// failed" or the failure message otherwise;
/// `debug_write("a.brep", &shape)`.
pub fn debug_write(file: &str, shape: &Shape) -> String {
    match write_file(
        shape,
        &AsciiString::from_bytes(file.as_bytes()),
        &ProgressRange::default(),
    ) {
        Ok(true) => file.to_string(),
        Ok(false) => "Error: write failed".to_string(),
        Err(failure) => failure.message().to_string(),
    }
}

/// `BRepTools_Dump(theShape)`: the dump of the shape between blank lines on the console;
/// "Shape dumped to std::cout" or the failure message; `debug_dump(&shape, &console)`.
pub fn debug_dump(shape: &Shape, console: &dyn ConsoleOutput) -> String {
    console.write(b"\n\n");
    let mut stream = IoStream::from_bytes(Vec::new());
    let result = dump(shape, &mut stream);
    console.write(&stream_bytes(stream));
    if let Err(failure) = result {
        return failure.message().to_string();
    }
    console.write(b"\n");
    "Shape dumped to std::cout".to_string()
}

/// `BRepTools_DumpLoc(theShape)`: the location set of the shape's location on the
/// console; "Location dumped to std::cout"; `debug_dump_location(&location, &console)`.
pub fn debug_dump_location(location: &Location, console: &dyn ConsoleOutput) -> String {
    console.write(b"\n\n");
    let mut set = LocationSet::default();
    set.add(location);
    let mut stream = IoStream::from_bytes(Vec::new());
    set.dump(&mut stream);
    console.write(&stream_bytes(stream));
    console.write(b"\n");
    "Location dumped to std::cout".to_string()
}

/// The bytes written to an in-memory stream.
fn stream_bytes(stream: IoStream) -> Vec<u8> {
    let mut bytes = Vec::new();
    if let Some(mut device) = stream.into_device() {
        let rewound = device.seek(SeekFrom::Start(0)).is_ok();
        if rewound && device.read_to_end(&mut bytes).is_err() {
            bytes.clear();
        }
    }
    bytes
}
