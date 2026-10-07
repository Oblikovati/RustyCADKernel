//! The `OCCT_DUMP_*` macros that the OCAF `DumpJson` members expand, over the TKernel
//! `DumpStream`. Each helper takes the C++ field expression of the macro argument
//! (`myVersion`, `aReference.get()`) and converts it with `Standard_Dump::DumpFieldToName`
//! (`Version`, `Reference`), as the macros do.
use rustycad_tkernel::standard::{
    Guid,
    dump::{DumpOutput, DumpStream, add_values_separator, dump_field_to_name, get_pointer_info},
};
use std::io;

/// A nested `DumpJson` call: writes one object into a stream at a depth.
pub type DumpChild<'a> = &'a dyn Fn(&mut DumpStream, i32) -> io::Result<()>;

/// `OCCT_DUMP_TRANSIENT_CLASS_BEGIN` / `OCCT_DUMP_CLASS_BEGIN`: `"className": "<name>"`;
/// `class_begin(&mut stream, "CDM_Document")?`.
pub fn class_begin(stream: &mut DumpStream, class_name: &str) -> io::Result<()> {
    field_string(stream, "className", class_name)
}

/// `OCCT_DUMP_FIELD_VALUE_STRING`: `"<name>": "<value>"`; `field_string(&mut s, "Name", "x")?`.
pub fn field_string(stream: &mut DumpStream, field: &str, value: &str) -> io::Result<()> {
    let name = key(field)?;
    add_values_separator(stream)?;
    stream.write_fragment(format!("\"{name}\": \"{value}\"").as_bytes())
}

/// `OCCT_DUMP_FIELD_VALUE_NUMERICAL` of an integer or a boolean (printed 0/1);
/// `field_integer(&mut stream, "Version", 1)?`.
pub fn field_integer(stream: &mut DumpStream, field: &str, value: i32) -> io::Result<()> {
    let name = key(field)?;
    add_values_separator(stream)?;
    stream.write_fragment(format!("\"{name}\": {value}").as_bytes())
}

/// `OCCT_DUMP_FIELD_VALUE_NUMERICAL` of a double, with the default stream precision;
/// `field_real(&mut stream, "myValue", 1.5)?`.
pub fn field_real(stream: &mut DumpStream, field: &str, value: f64) -> io::Result<()> {
    rustycad_tkernel::standard::dump::dump_real(stream, &key(field)?, value)
}

/// `OCCT_DUMP_FIELD_VALUE_GUID`: the 36-character GUID text; `field_guid(&mut s, "ID", id)?`.
pub fn field_guid(stream: &mut DumpStream, name: &str, value: Guid) -> io::Result<()> {
    field_string(stream, name, &value.to_string())
}

/// `OCCT_DUMP_FIELD_VALUE_POINTER`: the `GetPointerInfo` spelling of an address, empty for
/// null; `field_pointer(&mut stream, "Document", address)?`.
pub fn field_pointer(stream: &mut DumpStream, name: &str, address: *const ()) -> io::Result<()> {
    let pointer = (!address.is_null()).then_some(address as usize);
    let text = get_pointer_info(pointer, true).map_err(io::Error::other)?;
    field_string(stream, name, &String::from_utf8_lossy(text.as_bytes()))
}

/// `OCCT_DUMP_FIELD_VALUES_DUMPED`: `"<name>": {<child dump>}`, skipped at depth 0 and for a
/// null field (`child` is `None`); the child is dumped at `depth - 1`.
pub fn field_dumped(
    stream: &mut DumpStream,
    depth: i32,
    name: &str,
    child: Option<DumpChild<'_>>,
) -> io::Result<()> {
    let Some(child) = child.filter(|_| depth != 0) else {
        return Ok(());
    };
    let mut field = DumpStream::default();
    child(&mut field, depth - 1)?;
    key_to_class(stream, &key(name)?, field.bytes())
}

/// `OCCT_DUMP_BASE_CLASS`: a separator, then the parent-class dump written inline at
/// `depth - 1`; skipped at depth 0; `base_class(&mut stream, depth, &|s, d| parent(s, d))?`.
pub fn base_class(stream: &mut DumpStream, depth: i32, base: DumpChild<'_>) -> io::Result<()> {
    if depth == 0 {
        return Ok(());
    }
    add_values_separator(stream)?;
    base(stream, depth - 1)
}

/// `Standard_Dump::DumpKeyToClass`: `"<name>": {<text up to the first NUL>}`.
fn key_to_class(stream: &mut DumpStream, name: &str, text: &[u8]) -> io::Result<()> {
    let text = text.split(|&byte| byte == 0).next().unwrap_or_default();
    add_values_separator(stream)?;
    stream.write_fragment(format!("\"{name}\": {{").as_bytes())?;
    stream.write_fragment(text)?;
    stream.write_fragment(b"}")
}

/// `Standard_Dump::DumpFieldToName` of a macro argument, as UTF-8 text.
fn key(field: &str) -> io::Result<String> {
    let name = dump_field_to_name(field).map_err(io::Error::other)?;
    Ok(String::from_utf8_lossy(name.as_bytes()).into_owned())
}
