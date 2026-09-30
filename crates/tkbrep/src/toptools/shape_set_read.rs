// Rust translation of TopTools_ShapeSet::Read(IS, theProgress)
// (ModelingData/TKBRep/TopTools/TopTools_ShapeSet.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The base body of the virtual `Read` of `TopTools_ShapeSet`.
use super::format_version;
use super::shape_set::{ShapeSet, THE_ASCII_VERSIONS};
use super::shape_set_io::{WORD_BUFFER, read_shape_among};
use super::shape_set_text::shape_enum_of;
use crate::topods::Shape;
use rustycad_tkernel::message::{ProgressRange, ProgressScope};
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::storage::IoStream;

/// `char vers[101]` read with `getline(vers, 100, '\n')`.
const VERSION_LINE: i64 = 100;

/// The `char buffer[255]` of `Read`: it holds the `TShapes` tag, then the flags of
/// each shape. A failed extraction leaves it unchanged and a shorter word leaves the
/// bytes after its terminator in place, so the flags are read from this persistent
/// copy exactly as from the native array.
struct WordBuffer {
    bytes: [u8; WORD_BUFFER],
}

impl WordBuffer {
    /// `IS >> buffer`: the word is copied with its terminator when one is extracted.
    fn extract(&mut self, stream: &mut IoStream) {
        let word = stream.extract_word(WORD_BUFFER);
        if word.is_empty() {
            return;
        }
        self.bytes[..word.len()].copy_from_slice(&word);
        self.bytes[word.len()] = 0;
    }

    /// The C string (`strcmp` stops at the terminator).
    fn text(&self) -> &[u8] {
        let end = self
            .bytes
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(WORD_BUFFER);
        &self.bytes[..end]
    }

    /// `buffer[i] == '1'`.
    fn flag(&self, position: usize) -> bool {
        self.bytes[position] == b'1'
    }
}

/// `TopTools_ShapeSet::Read(IS, theProgress)`: clears the set, finds the version line,
/// reads the locations, the geometry and the `TShapes n` table (each shape made by
/// `read_shape_geometry`, filled by `add_shapes`, flagged, checked for version 1).
/// A missing version or table writes a console line and stops; "Interrupted by the
/// user" is written when the progress indicator breaks. The base hooks make no shape,
/// so a non-empty table raises Standard_NullObject on the flags (a crash natively);
/// `shape_set_read::read(&mut set, &mut stream, &ProgressRange::default())?`.
pub fn read<S: ShapeSet + ?Sized>(
    set: &mut S,
    stream: &mut IoStream,
    progress: &ProgressRange<'_>,
) -> Result<(), Failure> {
    set.clear();
    if !find_version(set, stream) {
        console(
            set,
            b"File was not written with this version of the topology\n",
        );
        return Ok(());
    }
    let scope = ProgressScope::with_literal(progress, "Reading", 100.0, false);
    read_sections(set, stream, &scope)
}

/// The locations, the geometry and the TShapes, each followed by the user-break check.
fn read_sections<S: ShapeSet + ?Sized>(
    set: &mut S,
    stream: &mut IoStream,
    scope: &ProgressScope<'_>,
) -> Result<(), Failure> {
    set.change_locations().read(stream, &scope.next(10.0))?;
    if interrupted(set, scope) {
        return Ok(());
    }
    set.read_geometry(stream, &scope.next(75.0))?;
    if interrupted(set, scope) {
        return Ok(());
    }
    let mut buffer = WordBuffer {
        bytes: [0; WORD_BUFFER],
    };
    buffer.extract(stream);
    if buffer.text() != b"TShapes" {
        console(set, b"Not a TShape table\n");
        return Ok(());
    }
    read_tshapes(set, stream, &mut buffer, &scope.next(15.0))?;
    interrupted(set, scope);
    Ok(())
}

/// `if (aPS.UserBreak()) std::cout << "Interrupted by the user" << '\n';`.
fn interrupted<S: ShapeSet + ?Sized>(set: &S, scope: &ProgressScope<'_>) -> bool {
    if !scope.user_break() {
        return false;
    }
    console(set, b"Interrupted by the user\n");
    true
}

fn console<S: ShapeSet + ?Sized>(set: &S, text: &[u8]) {
    set.shape_set_data().console.write(text);
}

/// The version loop: reads lines until one, without its trailing `\r` and `\n`
/// (never the first byte), is a known version line, and sets the format number.
fn find_version<S: ShapeSet + ?Sized>(set: &mut S, stream: &mut IoStream) -> bool {
    loop {
        let mut line = Vec::new();
        stream.getline(&mut line, VERSION_LINE, b'\n');
        // `strlen(vers)`: the line is a C string.
        if let Some(end) = line.iter().position(|byte| *byte == 0) {
            line.truncate(end);
        }
        while line.len() > 1 && matches!(line.last(), Some(b'\r' | b'\n')) {
            line.pop();
        }
        let known = (format_version::LOWER..=format_version::UPPER)
            .find(|version| line == THE_ASCII_VERSIONS[*version as usize].as_bytes());
        if let Some(version) = known {
            set.set_format_nb(version);
            return !stream.fail();
        }
        if stream.fail() {
            return false;
        }
    }
}

/// The `TShapes n` records.
fn read_tshapes<S: ShapeSet + ?Sized>(
    set: &mut S,
    stream: &mut IoStream,
    buffer: &mut WordBuffer,
    progress: &ProgressRange<'_>,
) -> Result<(), Failure> {
    let mut count = 0;
    stream.extract_int(&mut count);
    let scope = ProgressScope::with_literal(progress, "Shapes", f64::from(count), false);
    let mut index = 1;
    while index <= count && scope.more() {
        let shape = read_tshape(set, stream, buffer, count)?;
        set.shape_set_data_mut().shapes.add(shape);
        index += 1;
        drop(scope.next(1.0));
    }
    Ok(())
}

/// One record: type code, geometry, flags word, sub-shapes until `*`, then the flags
/// and, for version 1, `check`.
fn read_tshape<S: ShapeSet + ?Sized>(
    set: &mut S,
    stream: &mut IoStream,
    buffer: &mut WordBuffer,
    count: i32,
) -> Result<Shape, Failure> {
    let kind = shape_enum_of(&stream.extract_word(WORD_BUFFER));
    let mut shape = Shape::default();
    set.read_shape_geometry(kind, stream, &mut shape)?;
    buffer.extract(stream);
    read_sub_shapes(set, stream, &mut shape, count)?;
    apply_flags(set.format_nb(), &shape, buffer)?;
    if set.format_nb() == format_version::VERSION_1 {
        set.check(kind, &mut shape)?;
    }
    Ok(shape)
}

/// The sub-shape references up to `*`, each added with `add_shapes`.
fn read_sub_shapes<S: ShapeSet + ?Sized>(
    set: &mut S,
    stream: &mut IoStream,
    shape: &mut Shape,
    count: i32,
) -> Result<(), Failure> {
    loop {
        let child = read_shape_among(set.shape_set_data(), stream, count)?;
        if child.is_null() {
            return Ok(());
        }
        set.add_shapes(shape, &child)?;
    }
}

/// `S.Free(buffer[0] == '1')` ... `S.Convex(buffer[6] == '1')`; Checked only from
/// version 2 on.
fn apply_flags(format_nb: i32, shape: &Shape, buffer: &WordBuffer) -> Result<(), Failure> {
    shape.set_free(buffer.flag(0))?;
    shape.set_modified(buffer.flag(1))?;
    shape.set_checked(format_nb >= format_version::VERSION_2 && buffer.flag(2))?;
    shape.set_orientable(buffer.flag(3))?;
    shape.set_closed(buffer.flag(4))?;
    shape.set_infinite(buffer.flag(5))?;
    shape.set_convex(buffer.flag(6))
}
