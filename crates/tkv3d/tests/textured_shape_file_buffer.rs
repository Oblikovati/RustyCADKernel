//! Real captured filename allocation survives valid native in-place updates without field retargeting.
use rustycad_tkbrep::topods::Shape;
use rustycad_tkernel::{standard::Handle, tcollection::AsciiString};
use rustycad_tkv3d::ais::*;

#[test]
fn textured_filename_borrowed_alias_replays_native_defined_lifetime_rows() {
    let object = AisTexturedShape::new(&Shape::default());
    let mut rows = textured_static_file_rows(&object);
    rows.extend(textured_reused_file_rows(&object));
    object
        .set_texture_file_name(&"a-longer-reallocated-file.png".into())
        .unwrap();
    rows.push(FilenameBufferReceipt::new(&object).row("fresh-after-release"));
    rows.extend(textured_transferred_file_rows(&object));
    let actual = rows.join("\n") + "\n";
    let native = include_str!("fixtures/ais_textured_shape/native-file-buffer.txt");
    assert_eq!(actual, native);
}

struct FilenameBufferReceipt<'a> {
    object: &'a Handle<AisTexturedShape>,
    captured: AisTexturedShapeFileRef,
}
impl<'a> FilenameBufferReceipt<'a> {
    fn new(object: &'a Handle<AisTexturedShape>) -> Self {
        let captured = object.texture_file();
        Self { object, captured }
    }
    fn assign(&self, value: &str) {
        let value = AsciiString::from(value);
        self.object.set_texture_file_name(&value).unwrap();
    }
    fn row(&self, label: &str) -> String {
        let observed = self.captured.read().unwrap();
        let bytes = observed.as_bytes();
        let encoded = encode_filename_buffer(bytes);
        let same = self.captured.as_ptr() == self.object.texture_file().as_ptr();
        let base = self.object.textured_shape_base();
        let length = base.fields.read().texture_file.length();
        let row = format!(
            "{label} {} {} {} {} {encoded}",
            i32::from(same),
            self.object.ref_count(),
            length,
            bytes.len()
        );
        row.trim_end().to_owned()
    }
}
fn encode_filename_buffer(bytes: &[u8]) -> String {
    // Native byte receipts avoid interpreting arbitrary filenames as UTF-8.
    bytes
        .iter()
        .map(|byte| format!("{byte},"))
        .collect::<String>()
}
fn textured_static_file_rows(object: &Handle<AisTexturedShape>) -> Vec<String> {
    let trace = FilenameBufferReceipt::new(object);
    let mut rows = vec![trace.row("initial-static")];
    trace.assign("abc");
    rows.push(trace.row("retained-static"));
    trace.assign("0");
    rows.push(trace.row("predefined-static"));
    trace.assign("abc");
    object.set_texture_pix_map(&Handle::default());
    rows.push(trace.row("pixmap-static"));
    rows
}
fn textured_reused_file_rows(object: &Handle<AisTexturedShape>) -> Vec<String> {
    object
        .set_texture_file_name(&"abcdefghi.png".into())
        .unwrap();
    let trace = FilenameBufferReceipt::new(object);
    let mut rows = vec![trace.row("original")];
    for (label, value) in [("same-length", "qwertyuio.dat"), ("shorter", "good.dat")] {
        trace.assign(value);
        rows.push(trace.row(label));
    }
    let embedded = AsciiString::from_raw(b"ABC\0tail");
    object.set_texture_file_name(&embedded).unwrap();
    rows.push(trace.row("embedded-nul"));
    rows.extend(textured_direct_buffer_rows(&trace));
    rows
}
fn textured_direct_buffer_rows(trace: &FilenameBufferReceipt<'_>) -> Vec<String> {
    let base = trace.object.textured_shape_base();
    base.fields
        .write()
        .texture_file
        .set_value_char(1, b'Z')
        .unwrap();
    let changed = trace.row("protected-in-place");
    base.fields.write().texture_file.truncate(0).unwrap();
    vec![changed, trace.row("owned-truncated-empty")]
}

fn textured_transferred_file_rows(object: &Handle<AisTexturedShape>) -> Vec<String> {
    let value = AsciiString::from("move-owner.png");
    object.set_texture_file_name(&value).unwrap();
    let trace = FilenameBufferReceipt::new(object);
    let base = object.textured_shape_base();
    let mut moved = AsciiString::move_from(&mut base.fields.write().texture_file);
    let mut rows = vec![trace.row("moved-owner")];
    moved.copy("short.png");
    rows.push(trace.row("moved-owner-edit"));
    let mut swapped = AsciiString::from("swapped");
    moved.swap(&mut swapped);
    rows.push(trace.row("swapped-owner"));
    swapped.clear();
    assert!(trace.captured.read().is_err());
    rows.push(FilenameBufferReceipt::new(object).row("fresh-after-transferred-release"));
    rows
}

#[test]
fn textured_filename_alias_rejects_release_without_following_the_new_allocation() {
    let object = AisTexturedShape::new(&Shape::default());
    let initial = AsciiString::from("long-file.png");
    object.set_texture_file_name(&initial).unwrap();
    let captured = object.texture_file();
    let address = captured.as_ptr();
    object.set_texture_file_name(&"a".into()).unwrap();
    assert_eq!(captured.read().unwrap().as_bytes(), b"a");
    assert_eq!(captured.as_ptr(), address);
    let replacement = AsciiString::from("reallocated-file.png");
    object.set_texture_file_name(&replacement).unwrap();
    let failure = captured.read().unwrap_err();
    assert_eq!(failure.exception_type(), "Standard_NoSuchObject");
    assert!(failure.message().contains("captured allocation"));
    let current = object.texture_file().read().unwrap();
    assert_eq!(current.as_bytes(), replacement.as_bytes());
    assert_eq!(object.ref_count(), 1);
}

#[test]
fn textured_filename_observer_owns_neither_the_native_object_nor_its_buffer() {
    let object = AisTexturedShape::new(&Shape::default());
    let static_empty = object.texture_file();
    object
        .set_texture_file_name(&"owned-file.png".into())
        .unwrap();
    let captured = object.texture_file();
    assert_eq!(object.ref_count(), 1);
    drop(object);
    assert!(captured.read().is_err());
    assert_eq!(static_empty.read().unwrap().as_bytes(), b"");
}
