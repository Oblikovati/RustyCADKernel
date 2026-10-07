//! The `OCCT_DUMP_*` macros that the TKService `DumpJson` members expand, over the TKernel
//! `DumpStream`. Each helper takes the C++ field expression of the macro argument
//! (`myXOrigin`, `&myColor`) and converts it with `Standard_Dump::DumpFieldToName`
//! (`XOrigin`, `Color`), as the macros do.
//!
//! The same helpers exist in rustycad-tkcdf (`json_dump`); TKService does not depend on
//! TKCDF natively, so this crate keeps its own private copy until they move into
//! `rustycad_tkernel::standard::dump`.
use rustycad_tkernel::quantity::Color;
use rustycad_tkernel::standard::dump::{
    DumpOutput, DumpStream, add_values_separator, dump_field_to_name,
};
use std::io;

/// A nested `DumpJson` call: writes one object into a stream at a depth.
pub(crate) type DumpChild<'a> = &'a dyn Fn(&mut DumpStream, i32) -> io::Result<()>;

/// `OCCT_DUMP_TRANSIENT_CLASS_BEGIN` / `OCCT_DUMP_CLASS_BEGIN`: `"className": "<name>"`.
pub(crate) fn class_begin(stream: &mut DumpStream, class_name: &str) -> io::Result<()> {
    add_values_separator(stream)?;
    stream.write_fragment(format!("\"className\": \"{class_name}\"").as_bytes())
}

/// `OCCT_DUMP_FIELD_VALUE_NUMERICAL` of an integer, an enumeration or a boolean (0/1).
pub(crate) fn field_integer(stream: &mut DumpStream, field: &str, value: i32) -> io::Result<()> {
    let name = key(field)?;
    add_values_separator(stream)?;
    stream.write_fragment(format!("\"{name}\": {value}").as_bytes())
}

/// `OCCT_DUMP_FIELD_VALUE_NUMERICAL` of an unsigned (`size_t`) value.
pub(crate) fn field_unsigned(stream: &mut DumpStream, field: &str, value: u64) -> io::Result<()> {
    let name = key(field)?;
    add_values_separator(stream)?;
    stream.write_fragment(format!("\"{name}\": {value}").as_bytes())
}

/// `OCCT_DUMP_FIELD_VALUE_NUMERICAL` of a double, with the default stream precision.
pub(crate) fn field_real(stream: &mut DumpStream, field: &str, value: f64) -> io::Result<()> {
    rustycad_tkernel::standard::dump::dump_real(stream, &key(field)?, value)
}

/// `OCCT_DUMP_FIELD_VALUE_STRING`: `"<name>": "<value>"` with the value bytes as they are.
pub(crate) fn field_string(stream: &mut DumpStream, field: &str, value: &[u8]) -> io::Result<()> {
    let name = key(field)?;
    add_values_separator(stream)?;
    stream.write_fragment(format!("\"{name}\": \"").as_bytes())?;
    stream.write_fragment(value)?;
    stream.write_fragment(b"\"")
}

/// `OCCT_DUMP_FIELD_VALUE_POINTER`: the short `GetPointerInfo` text of an address, empty for
/// null; `field_pointer(&mut s, "Colors", 0)?`.
pub(crate) fn field_pointer(
    stream: &mut DumpStream,
    field: &str,
    address: usize,
) -> io::Result<()> {
    let text = rustycad_tkernel::standard::dump::get_pointer_info(Some(address), true)
        .map_err(io::Error::other)?;
    field_string(stream, field, text.as_bytes())
}

/// `OCCT_DUMP_FIELD_VALUES_DUMPED`: `"<name>": {<child dump>}`, skipped at depth 0; the child
/// is dumped at `depth - 1`.
pub(crate) fn field_dumped(
    stream: &mut DumpStream,
    depth: i32,
    field: &str,
    child: DumpChild<'_>,
) -> io::Result<()> {
    if depth == 0 {
        return Ok(());
    }
    let mut nested = DumpStream::default();
    child(&mut nested, depth - 1)?;
    key_to_class(stream, &key(field)?, nested.bytes())
}

/// `OCCT_DUMP_FIELD_VALUES_DUMPED` of a `Quantity_Color` field (`"RGB": [r, g, b]`).
pub(crate) fn field_color(
    stream: &mut DumpStream,
    depth: i32,
    field: &str,
    color: Color,
) -> io::Result<()> {
    field_dumped(stream, depth, field, &|nested, _| color.dump_json(nested))
}

/// `OCCT_DUMP_BASE_CLASS`: a separator, then the parent-class dump written inline at
/// `depth - 1`; skipped at depth 0.
pub(crate) fn base_class(
    stream: &mut DumpStream,
    depth: i32,
    base: DumpChild<'_>,
) -> io::Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numerical_fields_use_the_converted_key() {
        let mut stream = DumpStream::default();
        class_begin(&mut stream, "Aspect_Grid").unwrap();
        field_real(&mut stream, "myXOrigin", 1.5).unwrap();
        field_integer(&mut stream, "myIsActive", 1).unwrap();
        assert_eq!(
            stream.text(),
            "\"className\": \"Aspect_Grid\", \"XOrigin\": 1.5, \"IsActive\": 1"
        );
    }

    #[test]
    fn dumped_fields_stop_at_depth_zero() {
        let mut stream = DumpStream::default();
        field_color(&mut stream, 0, "&myColor", Color::default()).unwrap();
        assert_eq!(stream.text(), "");
        base_class(&mut stream, 0, &|_, _| Ok(())).unwrap();
        assert_eq!(stream.text(), "");
    }
}
