//! Anchored native pointer cases: no dangling buffer reads or invented text encoding.
#![allow(unsafe_code)] // The named single-thread buffer proves foreign writes between snapshot reads.
use rustycad_tkv3d::v3d::{V3dImageDumpOptions, V3dLightName};
use std::{
    cell::UnsafeCell,
    ffi::{CStr, c_char},
};

struct AnchoredLightBuffer {
    bytes: UnsafeCell<[u8; 8]>,
}
impl AnchoredLightBuffer {
    fn new() -> Self {
        let bytes = UnsafeCell::new(*b"sun\0\0\0\0\0");
        Self { bytes }
    }
    fn name(&self) -> V3dLightName<'_> {
        let pointer = self.bytes.get().cast::<c_char>();
        // The !Sync buffer stays allocated; this test never overlaps reads and writes.
        unsafe { V3dLightName::from_ptr(pointer) }
    }
    fn overwrite(&self, bytes: [u8; 8]) {
        let pointer = self.bytes.get();
        // Returned strings own bytes; no reference to the mutable storage escapes a read.
        unsafe { pointer.write(bytes) };
    }
}

fn light_row(label: &str, name: V3dLightName<'_>, anchor: *const c_char) -> String {
    let same = i32::from(name.as_ptr() == anchor);
    let bytes = match name.to_ascii_string() {
        Ok(bytes) => bytes,
        Err(failure) => return format!("{label} {same} {}\n", failure.exception_type()),
    };
    let mut row = format!("{label} {same} {}", bytes.len());
    for byte in bytes.as_bytes() {
        row.push_str(&format!(" {byte}"));
    }
    row.push('\n');
    row
}

fn alias_rows() -> String {
    let buffer = AnchoredLightBuffer::new();
    let mut options = V3dImageDumpOptions {
        light_name: buffer.name(),
        ..Default::default()
    };
    let copy = options;
    let anchor = buffer.name().as_ptr();
    let mut rows = light_row("original", options.light_name, anchor);
    rows.push_str(&light_row("copy", copy.light_name, anchor));
    rows.push_str(&mutated_alias_rows(
        &buffer,
        options.light_name,
        copy.light_name,
    ));
    options.light_name = V3dLightName::from_c_str(c"rebound");
    rows.push_str(&light_row("rebound", options.light_name, anchor));
    rows.push_str(&light_row("still-anchored", copy.light_name, anchor));
    rows
}

fn mutated_alias_rows(
    buffer: &AnchoredLightBuffer,
    original: V3dLightName<'_>,
    copy: V3dLightName<'_>,
) -> String {
    let anchor = buffer.name().as_ptr();
    buffer.overwrite(*b"moon\0xxx");
    let mut rows = light_row("mutated", original, anchor);
    rows.push_str(&light_row("mutated-copy", copy, anchor));
    buffer.overwrite(*b"m\0on\0xxx");
    rows.push_str(&light_row("truncated", copy, anchor));
    rows
}

fn immutable_light_row(label: &str, bytes: &[u8]) -> String {
    let text = CStr::from_bytes_until_nul(bytes).unwrap();
    let name = V3dLightName::from_c_str(text);
    light_row(label, name, text.as_ptr())
}

fn encoding_rows() -> String {
    let anchor = b"\xff\x80x\0y";
    let mut rows = immutable_light_row("bytes", anchor);
    rows.push_str(&immutable_light_row("interior", &anchor[1..]));
    rows.push_str(&immutable_light_row("empty-interior", &anchor[3..]));
    rows
}

type NativeDumpLayout = V3dImageDumpOptions<'static>;
const DUMP_OPTION_OFFSETS: [usize; 9] = [
    std::mem::offset_of!(NativeDumpLayout, width),
    std::mem::offset_of!(NativeDumpLayout, height),
    std::mem::offset_of!(NativeDumpLayout, buffer_type),
    std::mem::offset_of!(NativeDumpLayout, stereo_options),
    std::mem::offset_of!(NativeDumpLayout, tile_size),
    std::mem::offset_of!(NativeDumpLayout, to_adjust_aspect),
    std::mem::offset_of!(NativeDumpLayout, target_z_layer_id),
    std::mem::offset_of!(NativeDumpLayout, is_single_layer),
    std::mem::offset_of!(NativeDumpLayout, light_name),
];

fn layout_row() -> String {
    use std::mem::{align_of, size_of};
    let mut row = format!(
        "layout {} {}",
        size_of::<NativeDumpLayout>(),
        align_of::<NativeDumpLayout>()
    );
    for offset in DUMP_OPTION_OFFSETS {
        row.push_str(&format!(" {offset}"));
    }
    row.push('\n');
    row
}

#[test]
fn native_light_name_alias_copy_encoding_and_null_replay() {
    let default = V3dLightName::default();
    let mut rows = light_row("default", default, default.as_ptr());
    let null = V3dLightName::null();
    rows.push_str(&light_row("null", null, std::ptr::null()));
    rows.push_str(&alias_rows());
    rows.push_str(&encoding_rows());
    rows.push_str(&layout_row());
    assert_eq!(
        rows,
        include_str!("fixtures/image-dump-light-name-native.txt")
    );
}

#[test]
fn raw_null_distinct_empty_and_pointer_sized_native_field() {
    let null = unsafe { V3dLightName::from_ptr(std::ptr::null()) };
    assert_eq!(null, V3dLightName::null());
    let failure = null.to_ascii_string().unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_NullObject");
    assert!(failure.message().contains("LightName=null, expected"));
    assert_ne!(null, V3dLightName::default());
    assert_eq!(
        std::mem::size_of::<V3dLightName<'_>>(),
        std::mem::size_of::<*const c_char>()
    );
}
