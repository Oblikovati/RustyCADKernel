//! Replay true const-buffer fields with present and absent optional buffers.
use super::Graphic3dArrayOfPolylines;
use std::fmt::Write;
#[test]
fn native_const_array_buffer_fields_preserve_identity_and_nonowning_counts() {
    let mut rows = String::new();
    for (bounds, edges) in [(0, 0), (2, 0), (2, 4)] {
        record_buffer_reference_row(&mut rows, bounds, edges);
    }
    assert_eq!(
        rows,
        include_str!("../../tests/fixtures/array-buffer-reference/native.txt")
    );
}
fn record_buffer_reference_row(rows: &mut String, bounds: i32, edges: i32) {
    let array = Graphic3dArrayOfPolylines::new(4, bounds, edges, 0).unwrap();
    let attributes = array.attributes_ref();
    let indices = array.indices_ref();
    let bound_slot = array.bounds_ref();
    let counts = (
        attributes.ref_count(),
        indices.ref_count(),
        bound_slot.ref_count(),
    );
    let same = buffer_slot_identity(&array);
    writeln!(
        rows,
        "buffer {bounds} {edges} {} {} {} {} {} {}",
        counts.0, counts.1, counts.2, same.0, same.1, same.2
    )
    .unwrap();
}
fn buffer_slot_identity(array: &Graphic3dArrayOfPolylines) -> (i32, i32, i32) {
    let attributes = i32::from(std::ptr::eq(array.attributes_ref(), array.attributes_ref()));
    let indices = i32::from(std::ptr::eq(array.indices_ref(), array.indices_ref()));
    let bounds = i32::from(std::ptr::eq(array.bounds_ref(), array.bounds_ref()));
    (attributes, indices, bounds)
}
