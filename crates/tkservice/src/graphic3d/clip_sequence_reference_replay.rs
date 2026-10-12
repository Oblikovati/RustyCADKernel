//! Complete pinned live-node/reference transcript, replayed without native libraries.
use super::clip_sequence_storage::ClipPlaneNodeLease;
use super::{
    Graphic3dClipPlane, Graphic3dSequenceOfHClipPlane, Graphic3dSequenceOfHClipPlaneIterator,
};
use rustycad_tkernel::standard::{Handle, Transient};
type ClipReferenceActors = (
    Handle<Graphic3dSequenceOfHClipPlane>,
    Handle<Graphic3dClipPlane>,
    Handle<Graphic3dClipPlane>,
    Handle<Graphic3dClipPlane>,
);
fn clip_reference_actors() -> ClipReferenceActors {
    (
        Handle::new(Graphic3dSequenceOfHClipPlane::default()),
        Handle::new(Graphic3dClipPlane::default()),
        Handle::new(Graphic3dClipPlane::default()),
        Handle::new(Graphic3dClipPlane::default()),
    )
}
fn record_native_values(rows: &mut Vec<String>, phase: &str, values: &[i32]) {
    let columns: Vec<String> = values.iter().map(ToString::to_string).collect();
    rows.push(format!("{phase} {}", columns.join(" ")));
}
fn record_reference_counts(rows: &mut Vec<String>, phase: &str, actors: &ClipReferenceActors) {
    let (sequence, first, second, _) = actors;
    record_native_values(
        rows,
        phase,
        &[
            sequence.size(),
            sequence.transient_base().ref_count(),
            first.ref_count(),
            second.ref_count(),
        ],
    );
}
fn replay_null_fields(rows: &mut Vec<String>) {
    let sequence = Handle::new(Graphic3dSequenceOfHClipPlane::default());
    let null = Handle::<Graphic3dClipPlane>::default();
    let first = sequence.append_reference(&null);
    let duplicate = sequence.append_reference(&null);
    record_null_insertion(rows, &sequence, first, duplicate);
    sequence.remove_reference(&null);
    assert_eq!(
        sequence.first_ref().err().unwrap().exception_type(),
        "Standard_NoSuchObject"
    );
    record_native_values(
        rows,
        "empty",
        &[i32::from(sequence.is_empty()), sequence.size()],
    );
}
fn record_alias_identity(
    rows: &mut Vec<String>,
    first: &ClipPlaneNodeLease,
    cursor: &ClipPlaneNodeLease,
    second: &Handle<Graphic3dClipPlane>,
) {
    record_native_values(
        rows,
        "alias",
        &[
            i32::from(first.object_address() == second.object_address()),
            i32::from(cursor.object_address() == second.object_address()),
            i32::from(first.reference_address() == cursor.reference_address()),
        ],
    );
}
fn replay_live_fields(rows: &mut Vec<String>) {
    let actors = clip_reference_actors();
    let (sequence, first, second, _) = &actors;
    sequence.append_reference(first);
    let alias = sequence.first_ref().unwrap();
    let iterator = Graphic3dSequenceOfHClipPlaneIterator::new(sequence);
    let cursor_alias = iterator.value_ref().unwrap();
    record_reference_counts(rows, "borrow", &actors);
    alias.assign(second).unwrap();
    record_alias_identity(rows, &alias, &cursor_alias, second);
    record_reference_counts(rows, "replace", &actors);
}
fn record_cursor_edit(
    rows: &mut Vec<String>,
    actors: &ClipReferenceActors,
    iterator: &Graphic3dSequenceOfHClipPlaneIterator,
) {
    let (sequence, first, second, third) = actors;
    let same = iterator.value_ref().unwrap().object_address() == third.object_address();
    record_native_values(
        rows,
        "edit",
        &[
            i32::from(same),
            sequence.size(),
            first.ref_count(),
            second.ref_count(),
            third.ref_count(),
        ],
    );
}
fn replay_cursor_edits(rows: &mut Vec<String>) {
    let actors = clip_reference_actors();
    let (sequence, first, second, third) = &actors;
    sequence.append_reference(first);
    sequence.append_reference(second);
    let mut iterator = Graphic3dSequenceOfHClipPlaneIterator::new(sequence);
    sequence.remove_reference(second);
    sequence.append_reference(third);
    iterator.next();
    record_cursor_edit(rows, &actors, &iterator);
    sequence.remove_at(&mut iterator).unwrap();
    record_removed_cursor(rows, &actors, &iterator);
}
fn record_iterator_borrow(
    rows: &mut Vec<String>,
    sequence: &Handle<Graphic3dSequenceOfHClipPlane>,
    iterator: &Graphic3dSequenceOfHClipPlaneIterator,
) {
    let count = iterator.value_ref().unwrap().observe().ref_count();
    record_native_values(
        rows,
        "iterator-borrow",
        &[i32::from(iterator.more()), sequence.ref_count(), count],
    );
}
fn replay_null_initialization(rows: &mut Vec<String>) {
    let sequence = Handle::new(Graphic3dSequenceOfHClipPlane::default());
    sequence.append(Handle::new(Graphic3dClipPlane::default()));
    let mut iterator = Graphic3dSequenceOfHClipPlaneIterator::from_handle(&sequence);
    record_iterator_borrow(rows, &sequence, &iterator);
    iterator.init_handle(&Handle::<Graphic3dSequenceOfHClipPlane>::default());
    record_empty_cursor(rows, "iterator-null", sequence.ref_count(), &iterator);
    sequence.remove_at(&mut iterator).unwrap();
    record_empty_cursor(rows, "remove-end", sequence.size(), &iterator);
}
#[test]
fn actual_clipping_fields_and_native_cursor_replay_all_ten_reference_records() {
    let mut rows = Vec::new();
    replay_null_fields(&mut rows);
    replay_live_fields(&mut rows);
    replay_cursor_edits(&mut rows);
    replay_null_initialization(&mut rows);
    assert_eq!(
        rows.join("\n") + "\n",
        include_str!("../../tests/fixtures/clip-collection-references-native.txt")
    );
}

fn record_null_insertion(
    rows: &mut Vec<String>,
    sequence: &Graphic3dSequenceOfHClipPlane,
    first: bool,
    duplicate: bool,
) {
    let empty_field = i32::from(sequence.first_ref().unwrap().is_null());
    record_native_values(
        rows,
        "null",
        &[
            i32::from(first),
            i32::from(duplicate),
            sequence.size(),
            empty_field,
        ],
    );
}
fn record_removed_cursor(
    rows: &mut Vec<String>,
    actors: &ClipReferenceActors,
    iterator: &Graphic3dSequenceOfHClipPlaneIterator,
) {
    record_native_values(
        rows,
        "remove-cursor",
        &[
            i32::from(iterator.more()),
            actors.0.size(),
            actors.3.ref_count(),
        ],
    );
}

fn record_empty_cursor(
    rows: &mut Vec<String>,
    phase: &str,
    count: i32,
    iterator: &Graphic3dSequenceOfHClipPlaneIterator,
) {
    // Count is the actual owner count or actual sequence size from the native row's caller.
    record_native_values(rows, phase, &[i32::from(iterator.more()), count]);
}
