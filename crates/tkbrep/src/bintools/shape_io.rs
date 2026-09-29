// Created on: 2004-05-18
// Created by: Sergey ZARITCHNY
// Copyright (c) 2004-2014 OPEN CASCADE SAS
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
// Rust translation of the static Write and Read members of
// ModelingData/TKBRep/BinTools/BinTools.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BinTools::Write` and `BinTools::Read`: one shape in the grouped binary format, on a
//! stream or in a file.
use super::format_version;
use super::shape_set::BinShapeSet;
use super::shape_set_base::BinShapeSetBase;
use super::stream_buffer_device::StreamBufDevice;
use crate::topods::Shape;
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::osd::{FileSystem, IosOpenMode, default_file_system};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::storage::IoStream;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkgeombase::geom_tools::{ConsoleOutput, StdoutConsole};
use std::sync::{Arc, PoisonError};

/// The options of `BinTools::Write`: with triangles, with normals, and the format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WriteOptions {
    /// `theWithTriangles`: write the triangulations and polygons (a face without a
    /// surface always has its triangulation written).
    pub with_triangles: bool,
    /// `theWithNormals`: write the normals of the triangulations.
    pub with_normals: bool,
    /// `theVersion`: a [`format_version`] number.
    pub version: i32,
}

impl Default for WriteOptions {
    /// The options of the short `Write` overloads: triangles, no normals, the current
    /// format.
    fn default() -> Self {
        Self {
            with_triangles: true,
            with_normals: false,
            version: format_version::CURRENT,
        }
    }
}

/// `BinTools::Write(theShape, theStream, theWithTriangles, theWithNormals, theVersion,
/// theRange)` (and, with default options, `Write(theShape, theStream, theRange)`): the
/// shape set of `shape`, then the reference to `shape` itself.
///
/// ```
/// use rustycad_tkbrep::bintools::{self, WriteOptions};
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkernel::message::ProgressRange;
/// use rustycad_tkernel::storage::IoStream;
/// let mut stream = IoStream::from_bytes(Vec::new());
/// bintools::write(&Shape::default(), &mut stream, WriteOptions::default(), &ProgressRange::default()).unwrap();
/// let mut shape = Shape::default();
/// stream.seekg(0);
/// bintools::read(&mut shape, &mut stream, &ProgressRange::default()).unwrap();
/// assert!(shape.is_null());
/// ```
pub fn write(
    shape: &Shape,
    stream: &mut IoStream,
    options: WriteOptions,
    progress: &ProgressRange<'_>,
) -> Result<(), Failure> {
    let mut set = BinShapeSet::new();
    set.set_with_triangles(options.with_triangles);
    set.set_with_normals(options.with_normals);
    set.set_format_nb(options.version);
    set.add(shape)?;
    set.write(stream, progress)?;
    set.write_shape(shape, stream)
}

/// `BinTools::Read(theShape, theStream, theRange)`: reads a shape set (with triangles)
/// and the shape reference after it. A stream without a known version line prints the
/// shape set's message on standard output and reads the reference from what follows;
/// `bintools::read(&mut shape, &mut stream, &ProgressRange::default())?`.
pub fn read(
    shape: &mut Shape,
    stream: &mut IoStream,
    progress: &ProgressRange<'_>,
) -> Result<(), Failure> {
    read_with_console(shape, stream, progress, Arc::new(StdoutConsole))
}

/// `BinTools::Read(theShape, theStream, theRange)` with the shape set's `std::cout`
/// text going to `console`; `bintools::read_with_console(&mut shape, &mut stream, &range, console)?`.
pub fn read_with_console(
    shape: &mut Shape,
    stream: &mut IoStream,
    progress: &ProgressRange<'_>,
    console: Arc<dyn ConsoleOutput>,
) -> Result<(), Failure> {
    let mut set = BinShapeSet::with_console(console);
    set.set_with_triangles(true);
    set.read(stream, progress)?;
    let count = set.nb_shapes();
    set.read_subs(shape, stream, count)
}

/// `BinTools::Write(theShape, theFile, ...)` through `OSD_FileSystem::DefaultFileSystem()`;
/// see [`write_file_on`]. `bintools::write_file(&shape, &"a.bin".into(), WriteOptions::default(), &range)?`.
pub fn write_file(
    shape: &Shape,
    file: &AsciiString,
    options: WriteOptions,
    progress: &ProgressRange<'_>,
) -> Result<bool, Failure> {
    write_file_on(&*default_file_system(), shape, file, options, progress)
}

/// `BinTools::Write(theShape, theFile, ...)` on the given file system: opens an output
/// stream (`out | binary`), writes the shape and flushes. False when the stream cannot
/// be opened, is not good, or fails while writing or flushing;
/// `bintools::write_file_on(&fs, &shape, &"a.bin".into(), WriteOptions::default(), &range)?`.
pub fn write_file_on(
    file_system: &dyn FileSystem,
    shape: &Shape,
    file: &AsciiString,
    options: WriteOptions,
    progress: &ProgressRange<'_>,
) -> Result<bool, Failure> {
    let Some(opened) = file_system.open_ostream(file, IosOpenMode::OUT | IosOpenMode::BINARY)?
    else {
        return Ok(false);
    };
    let buffer = {
        let opened = opened.lock().unwrap_or_else(PoisonError::into_inner);
        if !opened.good() {
            return Ok(false);
        }
        opened.rdbuf()
    };
    let device = StreamBufDevice::new(buffer, IosOpenMode::OUT);
    let mut stream = IoStream::with_device(Box::new(device), false, true);
    stream.set_precision(15);
    write(shape, &mut stream, options, progress)?;
    Ok(flushed_good(stream))
}

/// `aStream->flush(); return aStream->good();`: a failed `pubsync` makes it bad.
fn flushed_good(stream: IoStream) -> bool {
    let good = stream.good();
    let Some(mut device) = stream.into_device() else {
        return false;
    };
    device.flush().is_ok() && good
}

/// `BinTools::Read(theShape, theFile, theRange)` through
/// `OSD_FileSystem::DefaultFileSystem()`; see [`read_file_on`].
/// `bintools::read_file(&mut shape, &"a.bin".into(), &range)?`.
pub fn read_file(
    shape: &mut Shape,
    file: &AsciiString,
    progress: &ProgressRange<'_>,
) -> Result<bool, Failure> {
    let console: Arc<dyn ConsoleOutput> = Arc::new(StdoutConsole);
    read_file_on(&*default_file_system(), shape, file, progress, console)
}

/// `BinTools::Read(theShape, theFile, theRange)` on the given file system, with the
/// shape set's `std::cout` text going to `console`: opens an input stream
/// (`in | binary`), reads the shape and reports whether the stream is still good (a
/// shape that ends the file leaves it at the end, which is not good). False when the
/// stream cannot be opened; `bintools::read_file_on(&fs, &mut shape, &name, &range, console)?`.
pub fn read_file_on(
    file_system: &dyn FileSystem,
    shape: &mut Shape,
    file: &AsciiString,
    progress: &ProgressRange<'_>,
    console: Arc<dyn ConsoleOutput>,
) -> Result<bool, Failure> {
    let Some(opened) =
        file_system.open_istream(file, IosOpenMode::IN | IosOpenMode::BINARY, 0, None)?
    else {
        return Ok(false);
    };
    let buffer = opened
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .rdbuf();
    let device = StreamBufDevice::new(buffer, IosOpenMode::IN);
    let mut stream = IoStream::with_device(Box::new(device), true, false);
    read_with_console(shape, &mut stream, progress, console)?;
    Ok(stream.good())
}
