// Rust translation of the file and progress commands of Draw/TKDraw/DBRep/DBRep.cxx
// (writebrep, readbrep, XProgress) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! `writebrep`/`binsave`, `readbrep`/`binrestore` and `XProgress`.
use super::CommandEntry;
use crate::draw::{
    DrawInterpretor, DrawProgressIndicator, DrawSession, default_console_mode, default_graph_mode,
    default_tcl_mode, parse_on_off_iterator, parse_on_off_no_iterator, set_default_console_mode,
    set_default_graph_mode, set_default_tcl_mode, set_stop_indicator,
};
use rustycad_tkbrep::bintools;
use rustycad_tkbrep::brep::BRepBuilder;
use rustycad_tkbrep::brep_tools::{self, BRepWriteOptions};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkbrep::toptools::format_version;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::tcollection::AsciiString;
use std::io::Read;

/// `writebrep`.
pub(super) const WRITEBREP: CommandEntry = (
    "writebrep",
    "writebrep shape filename [-binary {0|1}]=0 [-version Version]=4\n\t\t:                          [-triangles {0|1}]=1 [-normals {0|1}]=0\n\t\t: Save the shape in the ASCII (default) or binary format file.\n\t\t:  -binary  write into the binary format (ASCII when unspecified)\n\t\t:  -version a number of format version to save;\n\t\t:           ASCII  versions: 1, 2 and 3    (3 for ASCII  when unspecified);\n\t\t:           Binary versions: 1, 2, 3 and 4 (4 for Binary when unspecified).\n\t\t:  -triangles write triangulation data (TRUE when unspecified).\n\t\t:           Ignored (always written) if face defines only triangulation (no surface).\n\t\t:  -normals include vertex normals while writing triangulation data (FALSE when unspecified).",
    writebrep,
);
/// `readbrep`.
pub(super) const READBREP: CommandEntry = (
    "readbrep",
    "readbrep filename shape\n\t\t: Restore the shape from the binary or ASCII format file.",
    readbrep,
);
/// `binsave`.
pub(super) const BINSAVE: CommandEntry = ("binsave", "binsave shape filename", writebrep);
/// `binrestore`.
pub(super) const BINRESTORE: CommandEntry = ("binrestore", "alias to readbrep command", readbrep);
/// `XProgress`.
pub(super) const XPROGRESS: CommandEntry = (
    "XProgress",
    "XProgress [+|-t] [+|-c] [+|-g]\n\t\t The options are:\n\t\t   +|-t :  switch on/off output to tcl of Progress Indicator\n\t\t   +|-c :  switch on/off output to cout of Progress Indicator\n\t\t   +|-g :  switch on/off graphical mode of Progress Indicator",
    xprogress,
);

/// The parsed `writebrep` arguments.
#[derive(Default)]
struct WriteRequest {
    shape_name: String,
    shape: Shape,
    file_name: String,
    binary: bool,
    version: i32,
    with_triangles: bool,
    with_normals: bool,
}

/// `writebrep shape file [options]` / `binsave shape file`: writes the shape in the
/// ASCII or binary BRep format.
fn writebrep(session: &DrawSession, di: &DrawInterpretor, a: &[&str]) -> Result<i32, Failure> {
    let Some(request) = parse_write(session, di, a)? else {
        return Ok(1);
    };
    let progress = DrawProgressIndicator::new(di, 1.0);
    let range = progress.start();
    let file = AsciiString::new(&request.file_name);
    let written = if request.binary {
        if request.version > bintools::format_version::UPPER {
            di.append("Syntax error: unknown format version")?;
            return Ok(1);
        }
        if request.with_normals
            && request.version > 0
            && request.version < bintools::format_version::VERSION_4
        {
            di.append("Error: vertex normals require binary format version 4 or later")?;
            return Ok(1);
        }
        let version = if request.version > 0 {
            request.version
        } else {
            bintools::format_version::CURRENT
        };
        let options = bintools::WriteOptions {
            with_triangles: request.with_triangles,
            with_normals: request.with_normals,
            version,
        };
        bintools::write_file(&request.shape, &file, options, &range)?
    } else {
        if request.version > format_version::UPPER {
            di.append("Syntax error: unknown format version")?;
            return Ok(1);
        }
        if request.with_normals
            && request.version > 0
            && request.version < format_version::VERSION_3
        {
            di.append("Error: vertex normals require ascii format version 3 or later")?;
            return Ok(1);
        }
        let version = if request.version > 0 {
            request.version
        } else {
            format_version::CURRENT
        };
        let options = BRepWriteOptions {
            with_triangles: request.with_triangles,
            with_normals: request.with_normals,
            version,
        };
        brep_tools::write_file_with_options(&request.shape, &file, options, &range)?
    };
    if !written {
        di.append(format!("Cannot write to the file {}", request.file_name))?;
        return Ok(1);
    }
    di.append(&request.shape_name)?;
    Ok(0)
}

/// The `writebrep` options; `None` (the command returns 1) after a reported error.
fn parse_write(
    session: &DrawSession,
    di: &DrawInterpretor,
    a: &[&str],
) -> Result<Option<WriteRequest>, Failure> {
    let mut request = WriteRequest {
        binary: a[0].eq_ignore_ascii_case("binsave"),
        version: -1,
        with_triangles: true,
        ..WriteRequest::default()
    };
    let mut index = 1;
    while index < a.len() {
        let parameter = a[index].to_ascii_lowercase();
        match parameter.as_str() {
            "-binary" => request.binary = parse_on_off_no_iterator(a, &mut index),
            "-version" if index + 1 < a.len() => {
                index += 1;
                request.version = session.atoi(a[index]);
                if request.version <= 0 {
                    di.append("Syntax error: unknown version")?;
                    return Ok(None);
                }
            }
            "-notriangles" | "-triangles" => {
                request.with_triangles = parse_on_off_no_iterator(a, &mut index)
            }
            "-nonormals" | "-normals" => {
                let value = parse_on_off_iterator(a, &mut index);
                request.with_normals = if parameter == "-nonormals" {
                    !value
                } else {
                    value
                };
            }
            _ if request.shape_name.is_empty() => {
                request.shape_name = a[index].to_string();
                request.shape = session.shape(a[index])?;
                if request.shape.is_null() {
                    di.append(format!(
                        "Syntax error: {} is not a shape",
                        request.shape_name
                    ))?;
                    return Ok(None);
                }
            }
            _ if request.file_name.is_empty() => request.file_name = a[index].to_string(),
            _ => {
                di.append(format!("Syntax error: unknown argument '{parameter}'"))?;
                return Ok(None);
            }
        }
        index += 1;
    }
    if request.file_name.is_empty() {
        di.append("Syntax error: wrong number of arguments")?;
        return Ok(None);
    }
    Ok(Some(request))
}

/// The header probe of `readbrep`: true for the binary format, `None` when the file
/// cannot be opened or holds fewer than 255 bytes (the C++ `read` of 255 bytes then
/// fails).
fn is_binary_file(file_name: &str) -> Option<bool> {
    let mut file = std::fs::File::open(file_name).ok()?;
    let mut header = [0u8; 255];
    let mut filled = 0;
    while filled < header.len() {
        match file.read(&mut header[filled..]) {
            Ok(0) | Err(_) => return None,
            Ok(count) => filled += count,
        }
    }
    Some(!header.starts_with(b"DBRep_DrawableShape"))
}

/// `readbrep file shape` / `binrestore file shape`: reads a binary or ASCII BRep file
/// (recognized by its header) into a shape variable.
fn readbrep(session: &DrawSession, di: &DrawInterpretor, a: &[&str]) -> Result<i32, Failure> {
    if a.len() != 3 {
        di.append("Syntax error: wrong number of arguments")?;
        return Ok(1);
    }
    let (file_name, shape_name) = (a[1], a[2]);
    let Some(binary) = is_binary_file(file_name) else {
        di.append(format!("Error: cannot read the file '{file_name}'"))?;
        return Ok(1);
    };
    let progress = DrawProgressIndicator::new(di, 1.0);
    let range = progress.start();
    let file = AsciiString::new(file_name);
    let mut shape = Shape::default();
    let read = if binary {
        bintools::read_file(&mut shape, &file, &range)?
    } else {
        brep_tools::read_file(&mut shape, &file, &BRepBuilder::default(), &range)?
    };
    if !read {
        di.append(format!("Error: cannot read from the file '{file_name}'"))?;
        return Ok(1);
    }
    session.set_shape(shape_name, &shape)?;
    di.append(shape_name)?;
    Ok(0)
}

/// `sscanf("%p")`: a hexadecimal number with an optional `0x`.
fn parse_pointer(text: &str) -> Option<u64> {
    let digits = text
        .strip_prefix("0x")
        .or_else(|| text.strip_prefix("0X"))
        .unwrap_or(text);
    let end = digits
        .find(|c: char| !c.is_ascii_hexdigit())
        .unwrap_or(digits.len());
    u64::from_str_radix(&digits[..end], 16).ok()
}

/// `XProgress [+|-t] [+|-c] [+|-g]` / `XProgress -stop id`: sets the default output
/// modes of the progress indicator (or asks one to stop) and prints the modes.
fn xprogress(_session: &DrawSession, di: &DrawInterpretor, a: &[&str]) -> Result<i32, Failure> {
    let mut index = 1;
    while index < a.len() {
        let argument = a[index].as_bytes();
        let turn = argument.first() == Some(&b'+');
        if argument.first() != Some(&b'-') && !turn {
            index += 1;
            continue;
        }
        match argument.get(1) {
            Some(b't') => set_default_tcl_mode(turn),
            Some(b'c') => set_default_console_mode(turn),
            Some(b'g') => set_default_graph_mode(turn),
            _ if a[index] == "-stop" && index + 1 < a.len() => {
                if let Some(id) = parse_pointer(a[index + 1]) {
                    set_stop_indicator(id);
                }
                return Ok(0);
            }
            _ => {}
        }
        index += 1;
    }
    let state = |on: bool| if on { "ON" } else { "OFF" };
    di.append(format!(
        "Progress Indicator defaults: tcl mode is {}, console mode is {}, graphical mode is {}\n",
        state(default_tcl_mode()),
        state(default_console_mode()),
        state(default_graph_mode())
    ))?;
    Ok(0)
}
