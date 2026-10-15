// Rust port of DataExchange/TKXCAF/XCAFDoc/XCAFDoc_ShapeTool.cxx (Dump, DumpShape, the
// file-static DumpAssembly and DumpJson) at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The text and JSON dumps of `XCAFDoc_ShapeTool`.
use super::shape_tool::{XCAFDocShapeTool, shape_ref_node};
use super::support::{find, live};
use rustycad_tkbrep::topods::Shape;
use rustycad_tkcdf::json_dump::field_pointer;
use rustycad_tkernel::standard::Failure;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::tcollection::AsciiString;
use rustycad_tkg3d::top_abs::print_shape_type;
use rustycad_tklcaf::tdata_std::{TDataStdGenericExtString, TDataStdName};
use rustycad_tklcaf::tdf::{TdfAttributeState, TdfChildIterator, TdfLabel, TdfTool};
use rustycad_tkservice::json_dump::{field_integer, field_string};
use std::io::{self, Write};

/// An I/O error as a failure of the dump.
fn io_failure(error: io::Error) -> Failure {
    rustycad_tkernel::standard::exceptions::program_error(&format!(
        "XCAFDoc_ShapeTool dump: {error}"
    ))
}

/// The entry text of a label.
fn entry_of(label: &TdfLabel) -> String {
    let mut entry = AsciiString::default();
    TdfTool::entry(label, &mut entry);
    String::from_utf8_lossy(entry.as_bytes()).into_owned()
}

/// The common head of `DumpAssembly` and `DumpShape`: tabs, kind, type; then (with
/// `entry_separator` before the entry) the entry, the referred entry, the name and, deep,
/// the TShape and location addresses.
fn dump_head(
    stream: &mut dyn Write,
    label: &TdfLabel,
    shape: &Shape,
    level: i32,
    deep: bool,
    entry_separator: &str,
) -> Result<(), Failure> {
    for _ in 0..level {
        write!(stream, "\t").map_err(io_failure)?;
    }
    if XCAFDocShapeTool::is_assembly(label)? {
        write!(stream, "ASSEMBLY ").map_err(io_failure)?;
    } else if XCAFDocShapeTool::is_simple_shape(label)? {
        if label.father().father().father().is_root() {
            write!(stream, "PART ").map_err(io_failure)?;
        }
    } else {
        write!(stream, "INSTANCE ").map_err(io_failure)?;
    }
    print_shape_type(shape.shape_type()?, &mut WriteAdapter(stream)).map_err(io_failure)?;
    write!(stream, "{entry_separator}{}", entry_of(label)).map_err(io_failure)?;
    if XCAFDocShapeTool::is_reference(label)? {
        let node = shape_ref_node(label)?;
        let father = live(&node, "XCAFDoc_ShapeTool::Dump")?.father();
        let referred = live(&father, "XCAFDoc_ShapeTool::Dump")?.label();
        write!(stream, " (refers to {})", entry_of(&referred)).map_err(io_failure)?;
    }
    let name = find::<TDataStdName>(label, &TDataStdName::get_id())?;
    if let Some(name) = name.get() {
        write!(stream, " \"{}\" ", name.get().to_utf8()).map_err(io_failure)?;
    }
    if deep {
        write!(stream, "({:p}", shape.tshape().object_address()).map_err(io_failure)?;
        if !shape.location().is_identity() {
            write!(stream, ", {:p}", std::ptr::from_ref(shape.location())).map_err(io_failure)?;
        }
        write!(stream, ") ").map_err(io_failure)?;
    }
    Ok(())
}

/// A `&mut dyn Write` seen as an `impl Write`.
struct WriteAdapter<'a>(&'a mut dyn Write);
impl Write for WriteAdapter<'_> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.0.write(buffer)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

/// The file-static `DumpAssembly`: the head line of the label, then each child one level
/// deeper; an empty line after a top-level assembly.
fn dump_assembly(
    stream: &mut dyn Write,
    label: &TdfLabel,
    level: i32,
    deep: bool,
) -> Result<(), Failure> {
    let mut shape = Shape::default();
    XCAFDocShapeTool::get_shape(label, &mut shape)?;
    if shape.is_null() {
        return Ok(());
    }
    dump_head(stream, label, &shape, level, deep, " ")?;
    writeln!(stream).map_err(io_failure)?;
    let mut iterator = TdfChildIterator::new(label, false)?;
    while iterator.more() {
        dump_assembly(stream, &iterator.value(), level + 1, deep)?;
        iterator.next();
    }
    if level == 0 {
        writeln!(stream).map_err(io_failure)?;
    }
    Ok(())
}

impl XCAFDocShapeTool {
    /// `Dump(theDumpLog, deep)`: every shape as an assembly tree, then the free shapes.
    pub fn dump_shapes(&self, stream: &mut dyn Write, deep: bool) -> Result<(), Failure> {
        let shapes = self.get_shapes()?;
        if !shapes.is_empty() {
            writeln!(stream).map_err(io_failure)?;
        }
        for label in &shapes {
            dump_assembly(stream, label, 0, deep)?;
        }
        let free = self.get_free_shapes()?;
        write!(stream, "\nFree Shapes: {}\n", free.len()).map_err(io_failure)?;
        for label in &free {
            Self::dump_shape(stream, label, 0, deep)?;
            writeln!(stream).map_err(io_failure)?;
        }
        Ok(())
    }
    /// `DumpShape(theDumpLog, L, level, deep)`: the head line of a shape label, without a
    /// newline (two spaces before the entry).
    pub fn dump_shape(
        stream: &mut dyn Write,
        label: &TdfLabel,
        level: i32,
        deep: bool,
    ) -> Result<(), Failure> {
        let mut shape = Shape::default();
        if !Self::get_shape(label, &mut shape)? {
            return Ok(());
        }
        dump_head(stream, label, &shape, level, deep, "  ")
    }
}

/// The caches of `DumpJson`: the address and entry of each cached shape, then the flag.
pub(super) fn dump_caches(tool: &XCAFDocShapeTool, stream: &mut DumpStream) -> io::Result<()> {
    let entries: Vec<Vec<(usize, String)>> = {
        let caches = tool.state.read();
        [
            &caches.shape_labels,
            &caches.sub_shapes,
            &caches.simple_shapes,
        ]
        .iter()
        .map(|map| {
            map.items()
                .map(|(shape, label)| (std::ptr::from_ref(shape) as usize, entry_of(label)))
                .collect()
        })
        .collect()
    };
    let names = [
        ("&aShape", "aShapeLabel"),
        ("&aSubShape", "aSubShapeLabel"),
        ("&aSimpleShape", "aSimpleShapeLabel"),
    ];
    for (cache, (shape_field, label_field)) in entries.iter().zip(names) {
        for (address, entry) in cache {
            field_pointer(stream, shape_field, *address as *const ())?;
            field_string(stream, label_field, entry.as_bytes())?;
        }
    }
    field_integer(
        stream,
        "hasSimpleShapes",
        i32::from(tool.state.read().has_simple_shapes),
    )
}
