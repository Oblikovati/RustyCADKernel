// Rust translation of ModelingData/TKBRep/BRep/BRep_CurveRepresentation.hxx/.cxx at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The `OCCT_DUMP_*` macro expansions that the BRep `DumpJson` bodies use.
//!
//! The class-name, base-class and nested-field macros are the TKG2d helpers
//! (`dump_class_name`, `dump_base`, `dump_field`); this module adds the pointer and
//! handle fields.
use rustycad_tkernel::standard::dump::{self, DumpStream};
use rustycad_tkernel::standard::{Handle, RuntimeTyped};
pub use rustycad_tkg2d::geom2d::{dump_base, dump_class_name, dump_field, dump_flag};
use rustycad_tkmath::toploc::Location;
use std::io;

/// `OCCT_DUMP_FIELD_VALUE_NUMERICAL(stream, myCurve.get())`: the pointer as
/// `operator<<(const void*)` prints it, `0` for null with libstdc++;
/// `dump_pointer(&mut stream, "Curve", handle.object_address())?`.
pub fn dump_pointer(stream: &mut DumpStream, name: &str, address: *const ()) -> io::Result<()> {
    dump::add_values_separator(stream)?;
    let address = address as usize;
    let text = if address == 0 {
        "0".to_owned()
    } else {
        format!("{address:#x}")
    };
    stream.append(&format!("\"{name}\": {text}"));
    Ok(())
}

/// `OCCT_DUMP_FIELD_VALUES_DUMPED(stream, depth, &myLocation)`;
/// `dump_location(&mut stream, depth, "Location", &location)?`.
pub fn dump_location(
    stream: &mut DumpStream,
    depth: i32,
    name: &str,
    location: &Location,
) -> io::Result<()> {
    dump_field(stream, depth, name, |nested, nested_depth| {
        location.dump_json(nested, nested_depth)
    })
}

/// `OCCT_DUMP_FIELD_VALUES_DUMPED(stream, depth, myHandle.get())`: nothing for a null
/// handle; `dump_handle(&mut stream, depth, "Curve", &curve, |c, s, d| c.dump_json(s, d))?`.
pub fn dump_handle<T: ?Sized + RuntimeTyped>(
    stream: &mut DumpStream,
    depth: i32,
    name: &str,
    handle: &Handle<T>,
    dumper: impl FnOnce(&T, &mut DumpStream, i32) -> io::Result<()>,
) -> io::Result<()> {
    let Some(object) = handle.get() else {
        return Ok(());
    };
    dump_field(stream, depth, name, |nested, nested_depth| {
        dumper(object, nested, nested_depth)
    })
}
