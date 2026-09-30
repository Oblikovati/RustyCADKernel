// Rust translation of BRepTools::Dump, Write and Read
// (ModelingData/TKBRep/BRepTools/BRepTools.hxx/.cxx) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `BRepTools::Dump`, `Write` and `Read`: one shape in the BRep ASCII format, on a
//! stream or in a file.
use super::BRepToolsShapeSet;
use crate::bintools::stream_buffer_device::StreamBufDevice;
use crate::brep::BRepBuilder;
use crate::topods::Shape;
use crate::toptools::{ShapeSet, format_version};
use rustycad_tkernel::message::ProgressRange;
use rustycad_tkernel::osd::{FileSystem, IosOpenMode, default_file_system};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::storage::IoStream;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkgeombase::geom_tools::{ConsoleOutput, StdoutConsole};
use std::sync::{Arc, PoisonError};

/// The options of `BRepTools::Write`: with triangles, with normals, and the format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BRepWriteOptions {
    /// `theWithTriangles`: write the triangulations and polygons (a face without a
    /// surface always has its triangulation written).
    pub with_triangles: bool,
    /// `theWithNormals`: write the normals of the triangulations (from version 3 on).
    pub with_normals: bool,
    /// `theVersion`: a TopTools [`format_version`] number.
    pub version: i32,
}

impl Default for BRepWriteOptions {
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

/// `BRepTools::Dump(Sh, S)`: the shape reference, then the dump of its shape set;
/// `brep_tools::dump(&shape, &mut stream)?`.
pub fn dump(shape: &Shape, stream: &mut IoStream) -> Result<(), Failure> {
    let mut set = BRepToolsShapeSet::default();
    set.add(shape)?;
    set.dump_shape(shape, stream)?;
    set.dump(stream)
}

/// `BRepTools::Write(theShape, theStream, theProgress)`: [`write_with_options`] with
/// the default options; `brep_tools::write(&shape, &mut stream, &ProgressRange::default())?`.
pub fn write(
    shape: &Shape,
    stream: &mut IoStream,
    progress: &ProgressRange<'_>,
) -> Result<(), Failure> {
    write_with_options(shape, stream, BRepWriteOptions::default(), progress)
}

/// `BRepTools::Write(theShape, theStream, theWithTriangles, theWithNormals, theVersion,
/// theProgress)`: the shape set of `shape`, then the reference to `shape` itself;
/// `brep_tools::write_with_options(&shape, &mut stream, BRepWriteOptions::default(), &range)?`.
///
/// ```
/// use rustycad_tkbrep::brep::BRepBuilder;
/// use rustycad_tkbrep::brep_tools;
/// use rustycad_tkbrep::topods::Shape;
/// use rustycad_tkernel::message::ProgressRange;
/// use rustycad_tkernel::storage::IoStream;
/// let mut stream = IoStream::from_bytes(Vec::new());
/// brep_tools::write(&Shape::default(), &mut stream, &ProgressRange::default())?;
/// stream.seekg(0);
/// let mut shape = Shape::default();
/// brep_tools::read(&mut shape, &mut stream, &BRepBuilder::default(), &ProgressRange::default())?;
/// assert!(shape.is_null());
/// # Ok::<(), rustycad_tkernel::standard::Failure>(())
/// ```
pub fn write_with_options(
    shape: &Shape,
    stream: &mut IoStream,
    options: BRepWriteOptions,
    progress: &ProgressRange<'_>,
) -> Result<(), Failure> {
    let mut set = prepared_set(shape, options)?;
    set.write(stream, progress)?;
    set.write_shape(shape, stream)
}

/// The shape set of `shape` with the options set.
fn prepared_set(shape: &Shape, options: BRepWriteOptions) -> Result<BRepToolsShapeSet, Failure> {
    let mut set = BRepToolsShapeSet::new(options.with_triangles, options.with_normals);
    set.set_format_nb(options.version);
    set.add(shape)?;
    Ok(set)
}

/// `BRepTools::Read(Sh, S, B, theProgress)`: reads a shape set with a copy of `builder`
/// and the shape reference after it; `brep_tools::read(&mut shape, &mut stream, &builder, &range)?`.
pub fn read(
    shape: &mut Shape,
    stream: &mut IoStream,
    builder: &BRepBuilder,
    progress: &ProgressRange<'_>,
) -> Result<(), Failure> {
    read_with_console(shape, stream, builder, progress, Arc::new(StdoutConsole))
}

/// [`read`] with the shape set's `std::cout` messages going to `console`;
/// `brep_tools::read_with_console(&mut shape, &mut stream, &builder, &range, console)?`.
pub fn read_with_console(
    shape: &mut Shape,
    stream: &mut IoStream,
    builder: &BRepBuilder,
    progress: &ProgressRange<'_>,
    console: Arc<dyn ConsoleOutput>,
) -> Result<(), Failure> {
    let mut set = BRepToolsShapeSet::with_console(*builder, true, false, console);
    set.read(stream, progress)?;
    *shape = set.read_shape(stream)?;
    Ok(())
}

/// `BRepTools::Write(theShape, theFile, theProgress)`: [`write_file_on`] on the default
/// file system with the default options; `brep_tools::write_file(&shape, &name, &range)?`.
pub fn write_file(
    shape: &Shape,
    file: &AsciiString,
    progress: &ProgressRange<'_>,
) -> Result<bool, Failure> {
    write_file_on(
        &*default_file_system(),
        shape,
        file,
        BRepWriteOptions::default(),
        progress,
    )
}

/// `BRepTools::Write(theShape, theFile, theWithTriangles, theWithNormals, theVersion,
/// theProgress)` on the default file system; see [`write_file_on`].
pub fn write_file_with_options(
    shape: &Shape,
    file: &AsciiString,
    options: BRepWriteOptions,
    progress: &ProgressRange<'_>,
) -> Result<bool, Failure> {
    write_file_on(&*default_file_system(), shape, file, options, progress)
}

/// `BRepTools::Write(theShape, theFile, ...)` on `file_system`: opens `out | binary`,
/// writes `DBRep_DrawableShape` (for DRAW), the shape set and, when the stream is still
/// good, the shape reference, then flushes. False when the stream cannot be opened, is
/// not good or ends bad; `brep_tools::write_file_on(&fs, &shape, &name, options, &range)?`.
pub fn write_file_on(
    file_system: &dyn FileSystem,
    shape: &Shape,
    file: &AsciiString,
    options: BRepWriteOptions,
    progress: &ProgressRange<'_>,
) -> Result<bool, Failure> {
    let Some(opened) = file_system.open_ostream(file, IosOpenMode::OUT | IosOpenMode::BINARY)?
    else {
        return Ok(false);
    };
    let buffer = {
        let opened = opened.lock().unwrap_or_else(PoisonError::into_inner);
        if !opened.good() || opened.eof() {
            return Ok(false);
        }
        opened.rdbuf()
    };
    let mut set = prepared_set(shape, options)?;
    let device = StreamBufDevice::new(buffer, IosOpenMode::OUT);
    let mut stream = IoStream::with_device(Box::new(device), false, true);
    stream.insert_bytes(b"DBRep_DrawableShape\n");
    set.write(&mut stream, progress)?;
    let is_good = stream.good();
    if is_good {
        set.write_shape(shape, &mut stream)?;
    }
    Ok(flushed_good(stream) && is_good)
}

/// `aStream->flush(); isGood = aStream->good() && isGood && !errno` with `errno` just
/// reset: a failed `pubsync` makes the stream bad.
fn flushed_good(stream: IoStream) -> bool {
    let good = stream.good();
    let Some(mut device) = stream.into_device() else {
        return false;
    };
    device.flush().is_ok() && good
}

/// `BRepTools::Read(Sh, File, B, theProgress)`: [`read_file_on`] on the default file
/// system; `brep_tools::read_file(&mut shape, &name, &builder, &range)?`.
pub fn read_file(
    shape: &mut Shape,
    file: &AsciiString,
    builder: &BRepBuilder,
    progress: &ProgressRange<'_>,
) -> Result<bool, Failure> {
    let console: Arc<dyn ConsoleOutput> = Arc::new(StdoutConsole);
    read_file_on(
        &*default_file_system(),
        shape,
        file,
        builder,
        progress,
        console,
    )
}

/// `BRepTools::Read(Sh, File, B, theProgress)` on `file_system`, with the shape set's
/// `std::cout` messages going to `console`: opens `in`, reads the shape set and, when it
/// holds shapes, the shape reference. False when the stream cannot be opened or the set
/// is empty; `brep_tools::read_file_on(&fs, &mut shape, &name, &builder, &range, console)?`.
pub fn read_file_on(
    file_system: &dyn FileSystem,
    shape: &mut Shape,
    file: &AsciiString,
    builder: &BRepBuilder,
    progress: &ProgressRange<'_>,
    console: Arc<dyn ConsoleOutput>,
) -> Result<bool, Failure> {
    let Some(opened) = file_system.open_istream(file, IosOpenMode::IN, 0, None)? else {
        return Ok(false);
    };
    let buffer = opened
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .rdbuf();
    let device = StreamBufDevice::new(buffer, IosOpenMode::IN);
    let mut stream = IoStream::with_device(Box::new(device), true, false);
    let mut set = BRepToolsShapeSet::with_console(*builder, true, false, console);
    set.read(&mut stream, progress)?;
    if set.nb_shapes() == 0 {
        return Ok(false);
    }
    *shape = set.read_shape(&mut stream)?;
    Ok(true)
}
