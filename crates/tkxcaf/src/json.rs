//! The `OCCT_DUMP_*` expansions the `DumpJson` members of TKXCAF share, over the TKService
//! helpers: the value fields of geometry, shapes and `TCollection_HAsciiString` handles.
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::standard::Handle;
use rustycad_tkernel::standard::dump::DumpStream;
use rustycad_tkernel::tcollection::HAsciiString;
use rustycad_tkmath::gp::{Axis2, Direction3, Plane3, Point3};
use rustycad_tkservice::json_dump::{field_dumped, field_string};
use std::io;

/// `OCCT_DUMP_FIELD_VALUE_STRING(stream, aName)` with `aName = handle->ToCString()`, written
/// only when the handle is not null (the native `if (!handle.IsNull())` guard).
pub(crate) fn field_h_string(
    stream: &mut DumpStream,
    field: &str,
    text: &Handle<HAsciiString>,
) -> io::Result<()> {
    let Some(text) = text.get() else {
        return Ok(());
    };
    let bytes = text.string().as_bytes().to_vec();
    field_string(stream, field, &bytes)
}

/// `OCCT_DUMP_FIELD_VALUE_STRING(stream, myField.get())` of a handle: the stream form of
/// the raw pointer (hexadecimal, `0` for null), as natively.
pub(crate) fn field_raw_pointer(
    stream: &mut DumpStream,
    field: &str,
    address: *const (),
) -> io::Result<()> {
    let address = address as usize;
    let shown = if address == 0 {
        "0".to_owned()
    } else {
        format!("{address:#x}")
    };
    field_string(stream, field, shown.as_bytes())
}

/// `OCCT_DUMP_FIELD_VALUES_DUMPED(stream, depth, &myAxis)` of a `gp_Ax2`.
pub(crate) fn field_axis2(
    stream: &mut DumpStream,
    depth: i32,
    field: &str,
    axis: &Axis2,
) -> io::Result<()> {
    field_dumped(stream, depth, field, &|nested, _| axis.dump_json(nested))
}

/// `OCCT_DUMP_FIELD_VALUES_DUMPED(stream, depth, &myPnt)` of a `gp_Pnt`.
pub(crate) fn field_point(
    stream: &mut DumpStream,
    depth: i32,
    field: &str,
    point: &Point3,
) -> io::Result<()> {
    field_dumped(stream, depth, field, &|nested, _| point.dump_json(nested))
}

/// `OCCT_DUMP_FIELD_VALUES_DUMPED(stream, depth, &myDir)` of a `gp_Dir`.
pub(crate) fn field_direction(
    stream: &mut DumpStream,
    depth: i32,
    field: &str,
    direction: &Direction3,
) -> io::Result<()> {
    field_dumped(stream, depth, field, &|nested, _| {
        direction.dump_json(nested)
    })
}

/// `OCCT_DUMP_FIELD_VALUES_DUMPED(stream, depth, &myPln)` of a `gp_Pln`.
pub(crate) fn field_plane(
    stream: &mut DumpStream,
    depth: i32,
    field: &str,
    plane: &Plane3,
) -> io::Result<()> {
    field_dumped(stream, depth, field, &|nested, d| {
        plane.dump_json(nested, d)
    })
}

/// `OCCT_DUMP_FIELD_VALUES_DUMPED(stream, depth, &myShape)` of a `TopoDS_Shape`.
pub(crate) fn field_shape(
    stream: &mut DumpStream,
    depth: i32,
    field: &str,
    shape: &Shape,
) -> io::Result<()> {
    field_dumped(stream, depth, field, &|nested, d| {
        shape.dump_json(nested, d)
    })
}
